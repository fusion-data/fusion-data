use chrono::{DateTime, Utc};
use fusion_scheduler_api::v1::trigger_definition::TriggerStatus;
use fusiondata_context::ctx::CtxW;
use tracing::error;
use ultimate::Result;
use ultimate_api::v1::PagePayload;

use crate::service::trigger_definition::TriggerSchedule;

use super::{
  trigger_definition_bmc::TriggerDefinitionBmc, util::cron_to_next_occurrence, TriggerDefinition,
  TriggerDefinitionForCreate, TriggerDefinitionForPage, TriggerDefinitionForUpdate,
};

pub struct TriggerDefinitionSvc;

impl TriggerDefinitionSvc {
  pub async fn page(ctx: &CtxW, for_page: TriggerDefinitionForPage) -> Result<PagePayload<TriggerDefinition>> {
    TriggerDefinitionBmc::page(ctx.mm(), for_page.filter, for_page.pagination).await.map_err(Into::into)
  }

  pub async fn create(ctx: &CtxW, entity_c: TriggerDefinitionForCreate) -> Result<i64> {
    let entity_c = entity_c.improve()?;
    TriggerDefinitionBmc::create(ctx.mm(), entity_c).await.map_err(Into::into)
  }

  pub async fn scan_next_triggers(ctx: &CtxW, node_id: i64) -> Result<()> {
    let triggers = TriggerDefinitionBmc::scan_next_triggers(ctx.mm(), node_id).await?;
    let now = Utc::now();

    for trigger in triggers {
      let id = trigger.id;
      if let Some(entity_u) = Self::compute_next_trigger_update(trigger, &now) {
        TriggerDefinitionBmc::update_by_id(ctx.mm(), id, entity_u).await?;
        // TODO 创建 process instance 及 task
      }
    }
    Ok(())
  }

  /// 计算下次触发时间，并返回更新实体
  fn compute_next_trigger_update(td: TriggerDefinition, now: &DateTime<Utc>) -> Option<TriggerDefinitionForUpdate> {
    if td.status != TriggerStatus::Active as i32 {
      return None;
    }

    let mut u = TriggerDefinitionForUpdate::default();
    if td.invalid_time.is_some_and(|d| &d >= now) {
      // 已到失效时间
      u.status = Some(TriggerStatus::Completed as i32);
      return Some(u);
    }

    match td.schedule {
      TriggerSchedule::Simple { interval, first_delay, execution_count } => {
        if execution_count.is_some_and(|ec| td.executed_count >= ec) {
          // 达到次数限制
          u.status = Some(TriggerStatus::Completed as i32);
        } else {
          u.refresh_occurrence = Some(td.refresh_occurrence + first_delay.map(|d| d + interval).unwrap_or(interval));
        }
      }
      TriggerSchedule::Cron { cron, tz } => {
        match cron_to_next_occurrence(&cron, tz.as_deref(), now) {
          Ok(d) => u.refresh_occurrence = Some(d),
          Err(e) => error!("Cron to next occurrence Error: {}", e),
        };
      }
      TriggerSchedule::Depend => {}
    }

    Some(u)
  }
}
