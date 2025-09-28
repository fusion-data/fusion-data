# Hetuflow

Hetuflow 是一个高性能的分布式任务调度系统，提供了完整的任务调度、管理和监控解决方案。

## 系统架构

详细的系统架构设计请参考 [架构文档](architecture.md)。

## 核心组件

### 调度核心

- [核心模块](core/core.md) - 调度系统的核心逻辑

### 服务端组件

- [服务端概述](server/server.md) - 服务端整体架构
- [网关服务](server/server-gateway.md) - API 网关实现
- [网关 API](server/server-gateway-api.md) - API 接口定义
- [负载均衡](server/server-load_balance.md) - 负载均衡策略
- [调度器](server/server-scheduler.md) - 任务调度器
- [类型定义](server/server-types-entities.md) - 数据类型和实体
- [分布式锁](server/distributed_lock.md) - 分布式锁实现

### 代理组件

- [代理服务](agent/agent.md) - 任务执行代理管理

## 开发计划

当前的开发任务和计划请参考 [TODO 列表](todo.md)。

## 特性

- **高可用性**: 支持多节点部署，确保系统稳定运行
- **分布式锁**: 提供可靠的分布式锁机制
- **负载均衡**: 智能的任务分发和负载均衡
- **实时监控**: 完整的监控和管理界面
- **灵活配置**: 支持多种调度策略和配置选项
