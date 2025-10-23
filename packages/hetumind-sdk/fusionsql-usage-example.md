# 使用 @fusion-data/fusionsql 包的示例

## 导入类型

```typescript
// 从 fusionsql 导入分页和操作符类型
import { PageResult, OpValString, OpValNumber, OpValBool } from '@fusion-data/fusionsql';
import { HetumindSDK } from '@fusion-data/hetumind-sdk';

// 在 hetumind-sdk 中使用 fusionsql 类型
const sdk = new HetumindSDK({
  baseURL: 'http://localhost:3000',
  token: 'your-token'
});

// 使用 fusionsql 的 OpValString 进行字符串过滤
const workflowQuery = {
  options: { page: 1, limit: 10 },
  filter: {
    name: { $eq: 'my-workflow', $contains: 'test' },
    status: { $eq: 100 }, // Active
    is_archived: { $eq: false }
  }
};

// 使用 fusionsql 的 OpValNumber 进行数字过滤
const executionQuery = {
  options: { page: 1, limit: 20 },
  filter: {
    status: { $eq: 100 }, // Success
    duration: { $gt: 1000, $lt: 60000 } // 1-60秒
  }
};

// 使用 fusionsql 的 PageResult 处理分页结果
const workflows = await sdk.workflows.queryWorkflows(workflowQuery);
console.log(`找到 ${ workflows.page.page.total } 个工作流`);
```

## 类型兼容性

fusionsql 包提供了以下主要类型：

- **PageResult<T>**: 分页结果
- **OpValString**: 字符串操作符 (`$eq`, `$ne`, `$like`, `$in`, `$contains` 等)
- **OpValNumber**: 数字操作符 (`$eq`, `$ne`, `$gt`, `$gte`, `$lt`, `$lte`, `$in`)
- **OpValBool**: 布尔操作符 (`$eq`, `$ne`, `$null`)
- **OpValDateTime**: 日期时间操作符
- **OpValUuid**: UUID 操作符

这些类型可以直接在 hetumind-sdk 中使用，提供类型安全的查询构建。