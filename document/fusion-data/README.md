# Fusion Data 数据平台

- [商业智能](./bi.md)
- [数据治理](./governance.md)
- [调度系统](./scheduler/scheduler.md)
- [数据集成](./integration.md)
- [身份认证和授权](./iam/index.md)

## 子系统

### 通用库 fusiondata-common

数据库访问，缓存系统，消息系统，WEB/gRPC 等的实际工具封装库

### 系统监控 fusiondata-system

### IAM fusiondata-iam

作为主应用模块，负责认证和授权，菜单，权限，路由配置，系统监控功能?

### 任务调度 fusiodata-scheduler

批量/定时任务

### 通知服务 fusiondata-notification

email、phone、站内消息、……

### 文件服务 fusiondata-file

文件上传下载、云对象存储访问、……

### 数据集成 fusiondata-integration

各种数据数据（源）的访问/操作/转换/加载

### 元数据管理 fusiondata-metadata

### 主数据管理 fusiondata-maindata

### 数据资产 fusiondata-asset

### 数据标准 fusiondata-standard

### 数据质量 fusiondata-quality

- 数据稽查报告: 约束条件、指标
- 数据质量报告: 对稽查数据的报告

### 数据市场 fusiondata-market

- 提供基于SQL的服务：SQL as Services
- market作为数据服务模块，market模块引用market-api模块，market-execute模块、market-log模块分别承担数据服务的不同功能。

### 数据对比 fusiondata-comparison

作为数据比对模块，通过数据比对sql来校验数据源和数据目的的数据差异和总数情况。

### BI fusiondata-bi

商业智能应用: 报表、可视化

### 流程编排 fusiodata-workflow

对业务流进行编排，通常会由“任务调度”子系统进行调度执行

