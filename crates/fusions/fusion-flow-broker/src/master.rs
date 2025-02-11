use std::{
  collections::{BTreeMap, BTreeSet, VecDeque},
  sync::Arc,
  time::Duration,
};

use fusiondata_context::ctx::CtxW;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use ultimate::{application::Application, component::Component, Result};
use ultimate_api::v1::{Pagination, SortBy, SortDirection};
use ultimate_db::{
  modql::filter::{OpValInt32, OpValInt64},
  ModelManager,
};

use fusion_flow::{
  core::config::SchedulerConfig,
  service::{
    global_path::{GlobalPath, GlobalPathSvc},
    sched_namespace::{SchedNamespace, SchedNamespaceFilter, SchedNamespaceForUpdate, SchedNamespaceSvc},
    sched_node::{SchedNode, SchedNodeSvc},
  },
};

pub async fn loop_master(app: Application) -> Result<()> {
  let master: Master = app.component();

  match master.attempt_bind_master().await {
    Ok(_) => (),
    Err(err) => error!("Failed to attempt bind master: {:?}", err),
  };

  loop {
    master.execute().await;
    tokio::time::sleep(Duration::from_secs(10)).await;
  }
}

#[derive(Clone, Component)]
pub struct Master {
  #[config]
  flow_config: SchedulerConfig,

  #[component]
  mm: ModelManager,

  #[component]
  sched_namespace_svc: SchedNamespaceSvc,

  #[component]
  sched_node_svc: SchedNodeSvc,

  #[component]
  global_path_svc: GlobalPathSvc,

  schedulers: Arc<RwLock<Vec<SchedNode>>>,
}

impl Master {
  pub async fn execute(&self) {
    match self.scan_schedulers().await {
      Ok(rebalance) if rebalance => match self.rebalance_namespaces().await {
        Ok(_) => info!("Namespaces rebalanced"),
        Err(e) => error!("Balance namespaces error: {}", e),
      },
      Ok(_) => debug!("No need to rebalance namespaces"),
      Err(e) => error!("Scan schedulers error: {}", e),
    };
  }

  /// 获取所有 scheduler 节点，并更新本地缓存。
  ///
  /// # Returns:
  ///   - `true`: 需要重新平衡 namespaces
  async fn scan_schedulers(&self) -> Result<bool> {
    let ctx = CtxW::new_super_admin(self.mm.clone());
    let healthy_schedulers =
      self.sched_node_svc.check_healthy_schedulers(&ctx, *self.flow_config.alive_timeout()).await?;

    let mut schedulers = self.schedulers.write().await;

    // 判断是否需要 rebalance
    let need_rebalance = healthy_schedulers.len() != schedulers.len() || {
      // 存在 unhealthy
      let healthy_ids: BTreeSet<&str> = healthy_schedulers.iter().map(|n| n.id.as_str()).collect();
      schedulers.iter().any(|n| !healthy_ids.contains(n.id.as_str()))
    };

    *schedulers = healthy_schedulers;

    Ok(need_rebalance)
  }

  /// 平衡 namespace
  async fn rebalance_namespaces(&self) -> Result<()> {
    let schedulers = self.schedulers.read().await;
    let node_len = schedulers.len();
    if node_len == 0 {
      warn!("No active schedulers found, skip namespace balance.");
      return Ok(());
    }

    let ctx = CtxW::new_super_admin(self.mm.clone());
    let ns_len = self
      .sched_namespace_svc
      .count(&ctx, vec![SchedNamespaceFilter { status: Some(OpValInt32::Eq(100).into()), ..Default::default() }])
      .await? as usize;
    if ns_len == 0 {
      warn!("No enable namespaces, skip namespace balance.");
      return Ok(());
    }

    let ns_len_per_node = ns_len / node_len;
    // 构建 node -> namespace len
    let mut nodes: BTreeMap<&str, usize> = schedulers.iter().map(|node| (node.id.as_str(), ns_len_per_node)).collect();

    let mut cursor_id: Option<i64> = None;
    loop {
      // 获取所有有效 namespaces
      let mut namespaces: VecDeque<SchedNamespace> = self.get_batch_namespace(500, cursor_id.take()).await?;
      // let ns_len = namespaces.len();

      if namespaces.is_empty() {
        return Ok(());
      }

      // 注：在下一次 loop 时使用
      cursor_id = namespaces.back().map(|ns| ns.id);

      let mut unrelateds = vec![]; // 未关联 scheduler 的 namespace
      while let Some(namespace) = namespaces.pop_back() {
        let node_id = if let Some(node_id) = namespace.node_id.as_deref() {
          node_id
        } else {
          unrelateds.push(namespace); // 未关联 scheduler
          continue;
        };

        let remaining = if let Some(remaining) = nodes.get_mut(node_id) {
          remaining
        } else {
          unrelateds.push(namespace); // 关联的 scheduler 已无效
          continue;
        };

        if *remaining > 0 {
          *remaining -= 1;
        } else {
          // 节点可关联 namespace 容量已满
          nodes.remove(node_id);
          unrelateds.push(namespace);
        }
      }
      namespaces.extend(unrelateds);

      let mut updates: Vec<(Vec<i64>, String)> = nodes
        .iter()
        .map(|(node_id, remaining)| {
          let ns_ids = (0..*remaining).flat_map(|_| namespaces.pop_back().map(|ns| ns.id)).collect();
          (ns_ids, node_id.to_string())
        })
        .collect();

      // XXX 若还有剩余的 namespace，则分配给第一个 update
      updates[0].0.extend(namespaces.iter().map(|n| n.id));

      for (ns_ids, node_id) in updates {
        info!("Balance namespace {:?} to node {}", ns_ids, node_id);
        let ret = self
          .sched_namespace_svc
          .update(
            &ctx,
            vec![SchedNamespaceFilter { id: Some(OpValInt64::In(ns_ids.clone()).into()), ..Default::default() }],
            SchedNamespaceForUpdate { node_id: Some(node_id.clone()), ..Default::default() },
          )
          .await;
        if let Err(e) = ret {
          error!("Balance namespace {:?} to node {}: {}", ns_ids, node_id, e);
        }
      }
    }
  }

  async fn get_batch_namespace(&self, batch_size: i64, cursor_id: Option<i64>) -> Result<VecDeque<SchedNamespace>> {
    let ctx = CtxW::new_super_admin(self.mm.clone());
    let items = self
      .sched_namespace_svc
      .find_many(
        &ctx,
        vec![SchedNamespaceFilter {
          id: cursor_id.map(|id| OpValInt64::Gt(id).into()),
          status: Some(OpValInt32::Eq(100).into()),
          ..Default::default()
        }],
        Some(&Pagination {
          page_size: batch_size,
          sort_bys: vec![SortBy::new("id", SortDirection::Asc)],
          ..Default::default()
        }),
      )
      .await?
      .into_iter()
      .collect();
    Ok(items)
  }

  /// 查找 master 的 SchedNode
  ///
  /// - 找到 SchedNode
  ///    - 1.1. 判断 SchedNode 是否为当前 node_id，若是则返回
  ///    - 1.2. 不是，则 sleep
  /// - 未找到 SchedNode
  ///    - 2.1. 尝试注册自己为 Master
  ///        - 2.1.1. 注册成功，返回
  ///    - 2.2. 注册失败，则 sleep
  pub async fn attempt_bind_master(&self) -> Result<SchedNode> {
    let alive_timeout: Duration = *self.flow_config.alive_timeout();
    let ctx = CtxW::new_super_admin(self.mm.clone());
    let node_id = self.flow_config.node_id();

    loop {
      let node = match self.sched_node_svc.find_active_master(&ctx, alive_timeout).await? {
        Some(node) => Some(node),
        None => self.bind_or_get_master(&ctx, &node_id).await?,
      };
      match node {
        Some(node) if node.id == node_id => {
          debug!("Current node is master: {:?}", node);
          return Ok(node);
        }
        Some(node) => debug!("Current node isn't master: [{:?}]", node),
        _ => {
          debug!("Master node not found, sleeop 30s.");
          tokio::time::sleep(Duration::from_secs(30)).await
        }
      }
    }
  }

  /// 尝试注册为主节点或者获取最新主节点
  async fn bind_or_get_master(&self, ctx: &CtxW, node_id: &str) -> Result<Option<SchedNode>> {
    let mut count = 5; // 最多尝试 5 次
    while count > 0 {
      let revision = self
        .global_path_svc
        .find_unique(ctx, GlobalPath::MASTER)
        .await?
        .map(|p| p.revision)
        .unwrap_or_default();

      if self
        .global_path_svc
        .obtain_optimistic_lock(ctx, GlobalPath::MASTER, revision, Some(node_id.to_string()))
        .await?
      {
        let node = self.sched_node_svc.heartbeat_and_return(ctx, node_id).await?;
        info!("Register master success: {:?}", node);
        return Ok(Some(node));
      }

      if let Some(node) = self.sched_node_svc.find_active_master(ctx, Duration::from_secs(10)).await? {
        debug!("Get master success: {:?}", node);
        return Ok(Some(node));
      }

      count -= 1
    }

    Ok(None)
  }
}
