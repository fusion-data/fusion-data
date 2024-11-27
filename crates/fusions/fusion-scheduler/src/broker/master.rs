use std::{
  collections::{BTreeMap, BTreeSet, VecDeque},
  time::Duration,
};

use fusiondata_context::ctx::CtxW;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use ultimate::{application::Application, Result};
use ultimate_api::v1::{Pagination, SortBy, SortDirection};
use ultimate_db::modql::filter::OpValInt32;

use crate::service::{
  global_path::{GlobalPath, GlobalPathSvc},
  sched_namespace::{SchedNamespace, SchedNamespaceFilter, SchedNamespaceForUpdate, SchedNamespaceSvc},
  sched_node::{SchedNode, SchedNodeSvc},
};

use super::{SchedCmd, SchedulerConfig};

pub async fn loop_master(app: Application, db_tx: mpsc::Sender<SchedCmd>) -> Result<()> {
  let scheduler_config = SchedulerConfig::try_new(&app)?;
  let mut master_opt: Option<SchedNode> = None;
  while master_opt.is_none() {
    let alive_timeout: Duration = *scheduler_config.alive_timeout();
    let ctx = CtxW::new_with_app(app.clone());
    let node_id = scheduler_config.node_id();
    let node = match SchedNodeSvc::find_active_master(&ctx, alive_timeout).await? {
      Some(node) => node,
      None => bind_or_get_master(&ctx, node_id).await?,
    };
    if node.id == node_id {
      master_opt = Some(node);
    } else {
      tokio::time::sleep(Duration::from_secs(30)).await;
    }
  }

  let master = Master::new(app, scheduler_config, db_tx);
  loop {
    master.run().await;
    tokio::time::sleep(Duration::from_secs(60)).await;
  }
}

pub struct Master {
  app: Application,
  config: SchedulerConfig,
  cmd_tx: mpsc::Sender<SchedCmd>,
  schedulers: RwLock<Vec<SchedNode>>,
}

impl Master {
  pub fn new(app: Application, scheduler_config: SchedulerConfig, db_tx: mpsc::Sender<SchedCmd>) -> Self {
    Self { app, config: scheduler_config, cmd_tx: db_tx, schedulers: RwLock::new(vec![]) }
  }

  pub async fn run(&self) {
    match self.scan_schedulers().await {
      Ok(need_balance) if need_balance => match self.balance_namespaces().await {
        Ok(_) => (),
        Err(e) => error!("Balance namespaces error: {}", e),
      },
      Ok(_) => (),
      Err(e) => error!("Scan schedulers error: {}", e),
    };
  }

  /// 获取所有 scheduler 节点，并更新本地缓存。
  ///
  /// # Returns:
  ///   - `true`: 需要重新平衡 namespaces
  async fn scan_schedulers(&self) -> Result<bool> {
    let ctx = CtxW::new_with_app(self.app.clone());

    let healthy_nodes = SchedNodeSvc::check_healthy_schedulers(&ctx, *self.config.alive_timeout()).await?;

    // 移除 inalive list
    let mut schedulers = self.schedulers.write().await;
    let need_balance = healthy_nodes.len() != schedulers.len() || {
      let ids: BTreeSet<i64> = healthy_nodes.iter().map(|n| n.id).collect();
      schedulers.iter().any(|n| !ids.contains(&n.id))
    };
    *schedulers = healthy_nodes;

    Ok(need_balance)
  }

  /// 平衡 namespace
  async fn balance_namespaces(&self) -> Result<()> {
    let ctx = CtxW::new_with_app(self.app.clone());

    let schedulers = self.schedulers.read().await;
    let node_len = schedulers.len();
    if node_len == 0 {
      warn!("No active schedulers found, skip namespace balance.");
      return Ok(());
    }

    let ns_len = SchedNamespaceSvc::count(
      &ctx,
      vec![SchedNamespaceFilter { status: Some(OpValInt32::Eq(100).into()), ..Default::default() }],
    )
    .await? as usize;
    if ns_len == 0 {
      warn!("No enable namespaces, skip namespace balance.");
      return Ok(());
    }

    let ns_len_per_node = ns_len / node_len;
    // 构建 node -> namespace len
    let mut nodes: BTreeMap<i64, usize> = schedulers.iter().map(|node| (node.id, ns_len_per_node)).collect();

    // 获取所有有效 namespaces
    let mut namespaces: VecDeque<SchedNamespace> = self.get_batch_namespace(500).await?;
    // let ns_len = namespaces.len();

    if namespaces.is_empty() {
      return Ok(());
    }

    let mut unrelateds = vec![];
    while let Some(namespace) = namespaces.pop_back() {
      let node_id = if let Some(node_id) = namespace.node_id {
        node_id
      } else {
        unrelateds.push(namespace);
        continue;
      };

      let capacity = if let Some(ns) = nodes.get_mut(&node_id) {
        ns
      } else {
        unrelateds.push(namespace);
        continue;
      };

      if *capacity > 0 {
        *capacity -= 1;
      } else {
        // 节点无可关联容量
        nodes.remove(&node_id);
        unrelateds.push(namespace);
      }
    }

    let mut updates: Vec<(Vec<i32>, i64)> = nodes
      .iter()
      .map(|(node_id, len)| {
        let ns_ids: Vec<i32> = (0..*len).flat_map(|_| namespaces.pop_back().map(|ns| ns.id)).collect();

        (ns_ids, *node_id)
      })
      .collect();

    // 若还有剩余的 namespace，则分配给第一个 update
    updates[0].0.extend(namespaces.iter().map(|n| n.id));

    for (ns_ids, node_id) in updates {
      info!("Balance namespace {:?} to node {}", ns_ids, node_id);
      SchedNamespaceSvc::update(
        &ctx,
        vec![SchedNamespaceFilter { id: Some(OpValInt32::In(ns_ids).into()), ..Default::default() }],
        SchedNamespaceForUpdate { node_id: Some(node_id), ..Default::default() },
      )
      .await?;
    }

    Ok(())
  }

  async fn get_batch_namespace(&self, batch_size: i64) -> Result<VecDeque<SchedNamespace>> {
    let ctx = CtxW::new_with_app(self.app.clone());
    let items = SchedNamespaceSvc::find_many(
      &ctx,
      vec![SchedNamespaceFilter { status: Some(OpValInt32::Eq(100).into()), ..Default::default() }],
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
}

// async fn ping_node(node: SchedNode) -> NodeHealth {
//   let is_alive = true;
//   let last_check_time = Utc::now();
//   let mut unhealth_count = node.unhealth_count;

//   if !is_alive {
//     unhealth_count += 1;
//   }

//   NodeHealth { node_id: node.id, is_alive, inalive_count: unhealth_count, last_check_time }
// }

/// 尝试绑定并获取最新主节点
async fn bind_or_get_master(ctx: &CtxW, node_id: i64) -> Result<SchedNode> {
  // 最多尝试 5 次
  let mut count = 5;
  while count > 0 {
    let opt_path = GlobalPathSvc::find_unique(ctx, GlobalPath::MASTER).await?;
    let ret = GlobalPathSvc::obtain_optimistic_lock(
      ctx,
      GlobalPath::MASTER,
      opt_path.map(|p| p.revision).unwrap_or_default(),
      Some(node_id.to_string()),
    )
    .await?;

    if ret {
      let node = SchedNodeSvc::heartbeat_and_return(ctx, node_id).await?;
      return Ok(node);
    } else {
      match SchedNodeSvc::find_active_master(ctx, Duration::from_secs(10)).await? {
        Some(node) => return Ok(node),
        None => count -= 1,
      }
    }
  }

  panic!("Failed to bind & get active master");
}
