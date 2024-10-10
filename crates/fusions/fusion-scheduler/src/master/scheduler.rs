use std::time::Duration;

use chrono::Utc;
use fusion_server::app::AppState;
use hierarchical_hash_wheel_timer::{
  thread_timer::{self, TimerWithThread},
  ClosureTimer, OneShotClosureState, PeriodicClosureState, TimerReturn,
};
use modql::filter::{OpValInt32, OpValString, OpValValue};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::error;
use ultimate::Result;
use ultimate_grpc::GrpcStartInfo;
use uuid::Uuid;

use crate::{
  service::{
    sched_namespace::{AssociateNamespaceWithScheduler, SchedNamespaceSvc},
    sched_node::{NodeKind, SchedNode, SchedNodeFilter, SchedNodeForCreate, SchedNodeForUpdate, SchedNodeSvc},
  },
  NODE_ALIVE_TIMEOUT_SECONDS,
};

use super::{db_runner::loop_db_runner, DbCmd, DbRunner, SchedulerConfig};

pub type TimerCore = TimerWithThread<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;
pub type TimerRef =
  thread_timer::TimerRef<uuid::Uuid, OneShotClosureState<uuid::Uuid>, PeriodicClosureState<uuid::Uuid>>;

pub struct Scheduler {
  timer_core: TimerCore,
  scheduler_config: SchedulerConfig,
  db_tx: mpsc::Sender<DbCmd>,

  // XXX 在 init 方法调用后不可用
  _db_rx: Option<mpsc::Receiver<DbCmd>>,

  db_runner_handle: Option<JoinHandle<()>>,

  app_state: AppState,
}

impl Scheduler {
  pub fn new(app_state: AppState, grpc_start_info: GrpcStartInfo) -> Result<Self> {
    let mut scheduler_config: SchedulerConfig = app_state.configuration_state().underling().get("fusion-scheduler")?;
    if scheduler_config.advertised_addr.is_none() {
      scheduler_config.advertised_addr = Some(grpc_start_info.local_addr.to_string());
    }

    let timer_core = TimerWithThread::for_uuid_closures();

    let (db_tx, rx) = mpsc::channel(512);

    Ok(Self { timer_core, scheduler_config, db_tx, _db_rx: Some(rx), db_runner_handle: None, app_state })
  }

  pub async fn init(&mut self) -> Result<()> {
    heartbeat(self.timer_core.timer_ref(), self.db_tx.clone(), &self.scheduler_config);

    // 注册 master 节点到集群
    self.register().await?;

    let masters = self.list_masters().await?;
    let sched_node = masters.iter().find(|sn| &sn.id == self.scheduler_config.node_id).unwrap();

    // 遍历 namespaces 关关联。
    //     需要吗？namespace 在任何 master 节点运行其实都可以。我们只需要保证同一个 namespace 下的所有流程都由同一个 master 节点调度即可。

    let db_runner = DbRunner::new(self.app_state.clone(), self._db_rx.take().unwrap());
    let handle = loop_db_runner(db_runner);
    self.db_runner_handle = Some(handle);

    self.associate_namespaces(sched_node).await?;

    Ok(())
  }

  async fn list_masters(&self) -> ultimate::Result<Vec<SchedNode>> {
    let ctx = self.app_state.create_super_admin_ctx();
    let valid_check_time = Utc::now() - Duration::from_secs(NODE_ALIVE_TIMEOUT_SECONDS);
    let filter = SchedNodeFilter {
      kind: Some(OpValInt32::Eq(NodeKind::Scheduler as i32).into()),
      last_check_time: Some(OpValValue::Gt(serde_json::to_value(valid_check_time)?).into()),
      ..Default::default()
    };
    SchedNodeSvc::find_many(&ctx, vec![filter]).await
  }

  pub async fn shutdown(self) {
    match self.db_tx.send(DbCmd::Stop).await {
      Ok(_) => {}
      Err(e) => error!("Send Stop command to db_runner error: {:?}", e),
    };

    if let Some(handle) = self.db_runner_handle {
      if !handle.is_finished() {
        match handle.await {
          Ok(_) => {}
          Err(e) => {
            error!("Join db_runner_handle await error: {:?}", e)
          }
        };
      }
    }

    match self.timer_core.shutdown() {
      Ok(_) => {}
      Err(err) => error!("{:?}", err),
    };
  }

  /// 注册节点。当节点ID不存在则插入。若节点已存在，先判断节点是否存活，若存活则返回错误，不存活则更新。
  async fn register(&self) -> Result<()> {
    let node_id = self.scheduler_config.node_id;
    let node_kind = NodeKind::Scheduler;
    let node_addr = self.scheduler_config.advertised_addr.as_deref().expect("获取节点地址失败");

    let ctx = self.app_state.create_super_admin_ctx();
    let ctx = {
      let mm = ctx.mm().get_or_clone_with_txn()?;
      mm.dbx().begin_txn().await?;
      ctx.with_mm(mm)
    };

    match SchedNodeSvc::find(
      &ctx,
      vec![SchedNodeFilter { id: Some(OpValString::Eq(node_id.to_string()).into()), ..Default::default() }],
    )
    .await?
    {
      Some(sched_node) => {
        if self.is_alive_node(&sched_node.addr).await {
          error!("Node '{}|{}' is in a alive state.", node_id, node_addr);
          panic!("Node is offline");
        } else {
          // 先前节点已下线，更新为自己
          let entity_u = SchedNodeForUpdate {
            kind: Some(node_kind),
            addr: Some(node_addr.to_string()),
            status: Some(100),
            last_check_time: Some(Utc::now()),
          };
          SchedNodeSvc::update_by_id(&ctx, node_id, entity_u).await?;
        }
      }
      None => {
        let entity_c = SchedNodeForCreate { id: node_id.to_string(), kind: node_kind, addr: node_id.to_string() };
        SchedNodeSvc::create(&ctx, entity_c).await?;
      }
    };

    ctx.mm().dbx().commit_txn().await?;

    Ok(())
  }

  async fn is_alive_node(&self, node_addr: &str) -> bool {
    todo!()
  }

  /// 关联调度命名空间
  async fn associate_namespaces(&self, sched_node: &SchedNode) -> Result<()> {
    let ctx = self.app_state.create_super_admin_ctx().into_tx_mm_ctx()?;
    ctx.mm().dbx().begin_txn().await?;

    let datas = self
      .scheduler_config
      .namespaces
      .iter()
      .map(|namespace| AssociateNamespaceWithScheduler {
        namespace: namespace.to_string(),
        scheduler_id: sched_node.id.clone(),
      })
      .collect();

    let (associate_ns, _unassociate_ns): (Vec<_>, Vec<_>) = SchedNamespaceSvc::associate_scheduler(&ctx, datas)
      .await?
      .into_iter()
      .partition(|sn| sn.node_id == sched_node.id);

    // 保存 associate_ns 列表，定期查询相关流程并计算下次运行时间
    self.db_tx.send(DbCmd::ListenNamespaces(associate_ns)).await?;

    // TODO 保存 unassociate_ns 列表，定期查询 sched_namespace 尝试关联

    ctx.mm().dbx().commit_txn().await?;
    todo!()
  }
}

/// 创建 master 心跳定时任务
fn heartbeat(mut timer_ref: TimerRef, tx: mpsc::Sender<DbCmd>, conf: &SchedulerConfig) {
  let node_id = conf.node_id;
  let period = match duration_str::parse_std(conf.heartbeat_interval) {
    Ok(d) => d,
    Err(e) => panic!("Invalid heartbeat interval: {}", e),
  };
  timer_ref.schedule_action_periodic(Uuid::nil(), Duration::from_secs(17), period, move |_| {
    tx.blocking_send(DbCmd::Heartbeat(node_id)).unwrap();
    TimerReturn::Reschedule(())
  });
}
