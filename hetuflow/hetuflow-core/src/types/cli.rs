use clap::{ValueEnum, builder::PossibleValue};

use super::{AgentStatus, JobStatus, TaskStatus};

impl ValueEnum for AgentStatus {
  fn value_variants<'a>() -> &'a [Self] {
    &[
      Self::Idle,
      Self::Busy,
      Self::Connecting,
      Self::Disconnecting,
      Self::Offline,
      Self::Error,
      Self::Online,
    ]
  }

  fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
    let b = PossibleValue::new(format!("{}", *self as i32));
    Some(match self {
      Self::Idle => b.help("空闲状态"),
      Self::Busy => b.help("忙碌状态"),
      Self::Connecting => b.help("连接中"),
      Self::Disconnecting => b.help("断开连接中"),
      Self::Offline => b.help("离线状态"),
      Self::Error => b.help("错误状态"),
      Self::Online => b.help("在线状态"),
    })
  }
}

impl ValueEnum for JobStatus {
  fn value_variants<'a>() -> &'a [Self] {
    &[JobStatus::Created, JobStatus::Disabled, JobStatus::Enabled]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    let b = PossibleValue::new(format!("{}", *self as i32));
    Some(match self {
      JobStatus::Created => b.help("Created"),
      JobStatus::Disabled => b.help("Disabled"),
      JobStatus::Enabled => b.help("Enabled"),
    })
  }
}

impl ValueEnum for TaskStatus {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Pending, Self::Doing, Self::Failed, Self::Cancelled, Self::Succeeded]
  }

  fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
    let b = PossibleValue::new(format!("{}", *self as i32));
    Some(match self {
      TaskStatus::Pending => b.help("Pending"),
      TaskStatus::Doing => b.help("Doing"),
      TaskStatus::Failed => b.help("Failed"),
      TaskStatus::Cancelled => b.help("Cancelled"),
      TaskStatus::Succeeded => b.help("Succeeded"),
    })
  }
}
