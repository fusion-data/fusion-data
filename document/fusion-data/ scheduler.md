# 调度系统

## 设计

### 名词

- JobDefinition: 作业定义
- TriggerDefinition: 触发器定义

一个 JobDefinition 可以被多个 TriggerDefinition 关联，但一个 TriggerDefinition 只能关联一个 JobDefinition。

- JobTask: JobDefinition 在关联的某个 TriggerDefinition 被触发时，将定义/声明一个 JobTask。要调度的一个作业任务，至少执行一个 JobTaskInstance。当 JobTaskInstance 执行失败，可以选择重新调度。
- JobTaskInstance: 实践执行的作业任务实例

### 集群

- 基于 PG 数据库实现 HA
- 伪去中心化，通过向 scheduler.coordinator 表记录注册自己 node，确定一个 Coordinator node。将由 Coordinator node 负责作业任务分配
- 每个 node 都可以执行 job task
- 每个 node 向 coordinator 发送心跳，发送当前 node 负责情况
  - 负责情况由 cpu + memory 使用率计算得出
    - coordinator 考虑在计分上添加一些值。以确保它可以正常进行 job 协调

### 实现

#### JobTask 获取

coordinator 从 PG 中获取 N 分钟内待执行的 TriggerDefinition ，并根据 TriggerDefinition 获取 JobDefinition 后在内存中创建 JobTask 列表。
