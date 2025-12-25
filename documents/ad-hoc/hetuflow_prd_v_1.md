# hetuflow（河图流动）PRD v1.0

## 1. 产品概述

### 1.1 产品名称
- 英文名：**hetuflow**
- 中文名：**河图流动**

### 1.2 一句话定位
> **AI Workflow 的生产级控制与执行平台** —— AI 帮你设计流程，但永远不能越权执行。

### 1.3 产品愿景
让 AI Workflow 像代码一样：**可审计、可回滚、可上线到生产环境**。

### 1.4 核心理念（融合 n8n + Refly）
- Refly：自然语言 + AI 生成工作流
- n8n：工程级、确定性执行
- hetuflow：**AI 生成 + 人类接管 + 系统级运行**

---

## 2. 目标用户与使用场景

### 2.1 目标用户（按优先级）

#### P1：AI 创业公司 / AI SaaS 团队（10–50 人）
- 需要将 LLM + Workflow 上线到生产
- 强烈关注稳定性、成本、审计

#### P2：内容 / 增长 / 运营团队
- 需要 AI 自动生成内容，但不敢全自动发布

#### P3：企业 IT / 自动化团队（中后期）
- 内部流程自动化 + AI 能力

---

## 3. 核心问题与产品目标

### 3.1 核心问题
> **AI Workflow 在上线前缺乏安全、可控、可审计的机制**。

### 3.2 产品目标（MVP）
- 让用户 **敢于点击 Run**
- 明确 AI 能做什么、不能做什么
- 降低 AI Workflow 的生产事故率

---

## 4. 产品整体架构

### 4.1 系统分层

1. **AI Copilot 层**
   - 自然语言 → Workflow 草稿
   - Workflow 修改建议（Patch）

2. **Workflow 表达层**
   - AI 视图（意图式）
   - 工程视图（DAG）

3. **Workflow Runtime 层**
   - 确定性执行
   - 重试 / 幂等 / 状态管理

4. **Control & Safety 层**
   - Sandbox / Dry-run
   - Gate / 审批
   - 权限 / 审计

---

## 5. 功能需求（FRD）

### 5.1 Workflow 双视图系统（核心）

#### 5.1.1 AI 视图（Vibe View）
- 自然语言描述目标、约束
- 支持多轮对话修改
- 不暴露 DAG 细节

**示例**：
> 每天早上 9 点，基于 Hacker News 生成一条 AI 行业快讯，但需要人工确认后发布。

---

#### 5.1.2 工程视图（Flow View）
- DAG 节点编排
- 节点参数可配置
- 可插入代码节点

---

### 5.2 AI Copilot

#### 5.2.1 功能
- 从自然语言生成 Workflow Draft
- 将用户修改转为 Workflow Patch
- 解释 workflow 的执行逻辑

#### 5.2.2 约束（强制）
- AI 不可直接执行生产 Workflow
- AI 生成内容必须进入 Draft 状态

---

### 5.3 Workflow Runtime

#### 5.3.1 执行能力
- DAG 执行
- Step 级状态持久化
- 重试与失败回滚

#### 5.3.2 兼容性
- 支持 n8n Workflow JSON 导入（v1）

---

### 5.4 Gate / 审批系统（杀手锏）

#### 5.4.1 Gate 类型

1. **人工审批 Gate**
2. **规则 Gate**
   - 成本阈值
   - 风险操作（发布 / 删除 / 调用支付）
3. **AI Explain Gate**
   - 仅解释，不放行

#### 5.4.2 UI 要求
- 明确展示：
  - 本次执行的影响
  - 预计成本
  - 风险等级

---

### 5.5 Sandbox / Dry-run

#### 5.5.1 功能
- 模拟执行 Workflow
- 不调用真实外部 API
- 返回 Mock 输出

#### 5.5.2 输出
- 每一步的模拟结果
- Token / 成本估算

---

### 5.6 Workflow 版本管理

- 每次修改生成新版本
- 支持 Diff 查看
- 支持一键回滚

---

### 5.7 执行观测（Observability）

- Execution Timeline
- Step 级输入 / 输出
- 执行耗时
- AI Token / Cost

---

## 6. 权限与安全

### 6.1 权限模型（RBAC）

- Viewer：只读
- Editor：修改 workflow
- Operator：执行 / 审批
- Admin：权限管理

---

### 6.2 安全原则

- AI 永不越权
- 高风险节点必须 Gate
- 所有操作可审计

---

## 7. 非功能需求（NFR）

### 7.1 稳定性
- Workflow 执行失败率 < 0.1%

### 7.2 可扩展性
- 节点插件化
- 多模型路由

### 7.3 可审计性
- 所有 AI 输出 / 修改可追溯

---

## 8. MVP 范围定义

### 8.1 必须有
- AI Copilot
- Workflow 双视图
- Sandbox
- Gate / 审批
- 版本管理

### 8.2 明确不做
- Marketplace
- Agent 自动执行
- 大规模集成生态

---

## 9. 成功指标（MVP）

- ≥ 10 家设计伙伴
- ≥ 30 个真实生产 Workflow
- ≥ 1 次 AI 修改被回滚（说明系统被信任）

---

## 10. 长期演进方向（不进入 MVP）

- Agent → Workflow Patch
- 企业私有化部署
- Workflow 模板市场
- AI 自动优化建议

---

**文档状态**：Draft v1.0
**负责人**：CEO / Product
**适用阶段**：Seed / MVP

