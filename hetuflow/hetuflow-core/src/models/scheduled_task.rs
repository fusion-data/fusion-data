use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ScheduleEntity, TaskEntity};

/// 调度任务组合结构体
/// 包含任务信息和对应的调度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
  /// 任务实体
  pub task: TaskEntity,
  /// 调度实体（可选，手动触发的任务可能没有调度信息）
  pub schedule: Option<ScheduleEntity>,
}

impl ScheduledTask {
  /// 创建新的调度任务
  pub fn new(task: TaskEntity, schedule: Option<ScheduleEntity>) -> Self {
    Self { task, schedule }
  }

  /// 获取任务ID
  pub fn task_id(&self) -> Uuid {
    self.task.id
  }

  /// 获取Job ID
  pub fn job_id(&self) -> Uuid {
    self.task.job_id
  }

  /// 获取调度ID
  pub fn schedule_id(&self) -> Option<Uuid> {
    self.task.schedule_id
  }

  /// 获取任务优先级
  pub fn priority(&self) -> i32 {
    self.task.priority
  }

  /// 获取任务标签
  pub fn tags(&self) -> &Vec<String> {
    &self.task.tags
  }

  /// 检查任务是否匹配指定的标签
  pub fn matches_tags(&self, required_tags: &[String]) -> bool {
    if required_tags.is_empty() {
      return true;
    }

    required_tags.iter().all(|tag| self.task.tags.contains(tag))
  }

  /// 检查任务是否为定时任务
  pub fn is_scheduled(&self) -> bool {
    self.schedule.is_some()
  }

  /// 检查任务是否为手动触发任务
  pub fn is_manual(&self) -> bool {
    self.schedule.is_none()
  }

  /// 获取调度类型描述
  pub fn schedule_type_description(&self) -> String {
    match &self.schedule {
      Some(schedule) => format!("{:?}", schedule.schedule_kind),
      None => "Manual".to_string(),
    }
  }
}
