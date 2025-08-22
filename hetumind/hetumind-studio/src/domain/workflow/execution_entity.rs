use hetumind_core::{
  user::UserId,
  workflow::{Execution, ExecutionId, ExecutionMode, ExecutionStatus, WorkflowId},
};
use modelsql::{field::Fields, postgres::PgRowType};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ultimate_common::time::OffsetDateTime;
use ultimate_core::DataError;
use uuid::Uuid;

/// Workflow 执行实体主表,存储一次执行的宏观状态和生命周期信息。
/// 核心实体，代表了一次工作流的整体执行过程。可以把它想象成每次点击 "Execute Workflow" 后生成的主记录。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_entity")]
pub struct ExecutionEntity {
  /// PK
  pub id: ExecutionId,
  /// FK: [Workflow]
  pub workflow_id: WorkflowId,
  pub mode: ExecutionMode,
  /// 状态
  pub status: ExecutionStatus,
  /// 开始时间
  pub started_at: Option<OffsetDateTime>,
  /// 结束时间
  pub finished_at: Option<OffsetDateTime>,
  /// 触发者ID
  pub triggered_by: Option<UserId>,
  /// 等待时间
  pub wait_till: Option<OffsetDateTime>,
  /// 重试执行 ID
  pub retry_of: Option<ExecutionId>,
  /// 重试成功执行 ID
  pub retry_success_id: Option<ExecutionId>,
  pub deleted_at: Option<OffsetDateTime>,
  pub created_at: OffsetDateTime,
  pub created_by: UserId,
  pub updated_at: Option<OffsetDateTime>,
  pub updated_by: Option<UserId>,
}
impl PgRowType for ExecutionEntity {}

impl TryFrom<ExecutionEntity> for Execution {
  type Error = DataError;

  fn try_from(entity: ExecutionEntity) -> Result<Self, Self::Error> {
    Ok(Execution {
      id: entity.id,
      workflow_id: entity.workflow_id,
      status: entity.status,
      started_at: entity.started_at,
      finished_at: entity.finished_at,
      data: None,  // TODO: 需要从 ExecutionData 表中获取数据
      error: None, // TODO: 需要从 ExecutionMetadata 表中获取数据
      mode: entity.mode,
      triggered_by: entity.triggered_by,
    })
  }
}

/// 执行数据表
///
/// - 作用: 存储工作流执行过程中产生的实际业务数据。这通常是工作流运行到最后一步时，所有节点输出数据的 JSON 集合。为了数据库性能，这个可能很大的数据体被分离到单独的表中。
/// - 核心内容: 存储一个大的 JSON 对象，包含了所有分支、所有 item 的最终数据。
/// - 关系: 通过 1:1 关系与 ExecutionEntity 关联。当你在 hetumind 编辑器中查看一次成功执行的结果时，你看到的数据就来源于此。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_data")]
pub struct ExecutionDataEntity {
  pub execution_id: ExecutionId,
  /// 1:1 [Workflow]
  /// - 存储执行快照
  ///   - 问题: 如果用户在没有保存工作流的情况下点击执行，或者在执行后修改了工作流，我们如何保证能复现或审查当初的执行情况？
  ///   - 解决方案: 在每次执行时，hetumind 会将当时完整的工作流结构 (一个 [Workflow] 对象) 序列化成 JSON，并存储在 ExecutionData 实体的 workflow_data 字段中。
  ///     当次执行中的每个节点在执行时直接使用该快照，而不是使用数据库中的工作流。
  ///   - 意义: 这相当于为每次执行都拍下了一张“执行快照”。无论原来的工作流后来如何变化甚至被删除，这张快照都保证了执行记录的不可变性和可追溯性。
  pub workflow_data: serde_json::Value,

  /// 存储最终结果
  /// - 存储内容: 只有工作流最后一个（或所有未连接输出的）节点的最终输出数据才会被完整地序列化成一个大的 JSON 字符串，保存在 ExecutionData 表的 data 字段中。
  /// - 目的: 让用户可以在执行日志中看到工作流最终产出了什么结果。
  /// - 限制: 这里不包含中间节点的输入/输出数据。例如，在一个 Start -> Set -> Google Sheets 的工作流中，ExecutionData 只会保存 Google Sheets 节点的输出，而不会保存 Set 节点的输出
  pub data: String,
}
impl PgRowType for ExecutionDataEntity {}

/// 执行元数据表
/// - 作用: 存储关于【整个工作流执行】的、非结构化的上下文元信息。这是一个灵活的键值对集合，用于补充 ExecutionEntity 中的标准字段。
/// - 关系: 通过 OneToMany 关系与 ExecutionEntity 关联。这使得 hetumind 可以在 UI 上展示每个节点的执行耗时、成功与否等详细信息，对于调试和监控至关重要。
/// - 注意: 【不要】用它来存储任何与单个节点执行相关的过程信息（如状态、时间、错误等），这类信息应由专门的 `NodeExecution` 实体负责。
/// - 核心内容
///   - 节点状态: 每个节点的开始时间、结束时间、状态（成功/失败）。
///   - 错误信息: 如果某个节点失败，具体的错误信息会记录在这里。
///   - 重试信息: 节点的重试次数和策略。
///   - 数据来源: 追踪数据是如何从一个节点传递到另一个节点的。
/// - 示例:
///   - key: "triggering_ip", value: "192.168.1.1"
///   - key: "runner_version", value: "v0.2.1"
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_metadata")]
pub struct ExecutionMetadata {
  pub id: Uuid,
  pub execution_id: ExecutionId,
  pub key: String,
  pub value: String,
}
impl PgRowType for ExecutionMetadata {}

/// 执行注解表
/// - 作用: 存储用户或系统为某次执行添加的人工注释或标记。这是一个企业版功能，用于增强团队协作和问题追溯。
/// - 核心内容:
///   - content: 注释的文本内容。
///   - userId: 添加该注释的用户 ID。
///   - createdAt: 创建时间。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_annotation")]
pub struct ExecutionAnnotationEntity {
  pub id: Uuid,
  pub execution_id: ExecutionId,
  pub vote: Option<String>,
  pub note: Option<String>,
  pub created_at: OffsetDateTime,
  pub updated_at: OffsetDateTime,
}
impl PgRowType for ExecutionAnnotationEntity {}

/// 执行注解标签关联表
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
#[enum_def(table_name = "execution_annotation_tags")]
pub struct ExecutionAnnotationTags {
  pub annotation_id: Uuid,
  pub tag_id: Uuid,
}
impl PgRowType for ExecutionAnnotationTags {}
