# 工作流组件测试文档

本目录包含了 Hetumind Web 工作流系统的全面测试套件。

## 📋 目录结构

```
__tests__/
├── setup.ts                 # 测试环境配置
├── README.md               # 本文档
├── WorkflowCanvas.test.tsx # 工作流画布组件测试
├── TriggerNode.test.tsx    # 触发器节点测试
├── AIAgentNode.test.tsx    # AI Agent 节点测试
├── WorkflowEngine.test.tsx # 工作流执行引擎测试
├── PerformanceOptimizer.test.tsx # 性能优化组件测试
└── integration/
    └── WorkflowIntegration.test.tsx # 集成测试
```

## 🧪 测试类型

### 1. 单元测试 (Unit Tests)

**位置**: `__tests__/*.test.tsx`

**覆盖范围**:
- 组件渲染和基本功能
- 用户交互处理
- 数据传递和状态管理
- 错误处理和边界情况
- 可访问性功能

**示例**:
```typescript
describe('WorkflowCanvas', () => {
  it('renders workflow canvas correctly', () => {
    renderWithProviders(<WorkflowCanvas {...defaultProps} />);
    expect(screen.getByTestId('react-flow')).toBeInTheDocument();
  });
});
```

### 2. 集成测试 (Integration Tests)

**位置**: `__tests__/integration/*.test.tsx`

**覆盖范围**:
- 多组件协作
- 工作流创建和执行流程
- 数据流和状态同步
- 性能和内存管理
- 错误恢复机制

**示例**:
```typescript
describe('Workflow Integration', () => {
  it('creates and executes complete workflow', async () => {
    renderWithProviders(<WorkflowEditor />);
    // 测试完整的工作流创建和执行流程
  });
});
```

### 3. 性能测试 (Performance Tests)

**覆盖范围**:
- 渲染性能
- 大数据处理能力
- 内存使用优化
- 响应时间测试

## 🛠️ 测试工具和配置

### 核心依赖

- **Jest**: 测试框架
- **React Testing Library**: React 组件测试
- **Jest DOM**: DOM 断言扩展
- **TypeScript**: 类型安全测试

### Mock 策略

#### React Flow Mock
```typescript
jest.mock('@xyflow/react', () => ({
  ReactFlow: ({ children, onNodesChange }) => (
    <div data-testid="react-flow">
      {children}
    </div>
  ),
  // 其他组件...
}));
```

#### Ant Design Mock
```typescript
jest.mock('antd', () => ({
  Button: ({ children, onClick }) => (
    <button onClick={onClick}>{children}</button>
  ),
  // 其他组件...
}));
```

#### 图标 Mock
```typescript
jest.mock('@ant-design/icons', () => ({
  PlayCircleOutlined: () => <span>▶️</span>,
  // 其他图标...
}));
```

## 🚀 运行测试

### 基本命令

```bash
# 运行所有测试
npm run test

# 运行单元测试
npm run test:unit

# 运行集成测试
npm run test:integration

# 运行端到端测试
npm run test:e2e

# 运行性能测试
npm run test:performance

# 生成覆盖率报告
npm run test:coverage

# 监视模式
npm run test:watch

# CI 模式
npm run test:ci
```

### 自定义测试运行器

使用自定义测试运行器获得更好的报告：

```bash
# 使用测试脚本
node scripts/test-runner.js

# 指定测试类型
node scripts/test-runner.js unit integration

# 查看帮助
node scripts/test-runner.js --help
```

## 📊 测试覆盖率

### 覆盖率目标

- **整体覆盖率**: ≥ 70%
- **分支覆盖率**: ≥ 70%
- **函数覆盖率**: ≥ 70%
- **行覆盖率**: ≥ 70%

### 覆盖率报告

运行 `npm run test:coverage` 后查看：

- **控制台报告**: 实时显示覆盖率统计
- **HTML 报告**: `coverage/lcov-report/index.html`
- **JSON 报告**: `coverage/coverage-final.json`
- **LCOV 报告**: `coverage/lcov.info`

### 覆盖率配置

```javascript
// jest.config.js
coverageThresholds: {
  global: {
    branches: 70,
    functions: 70,
    lines: 70,
    statements: 70,
  },
},
```

## 🎯 测试最佳实践

### 1. 测试命名

```typescript
// ✅ 好的命名
describe('WorkflowCanvas', () => {
  it('renders nodes correctly when data is provided', () => {});
  it('handles node selection when clicked', () => {});
});

// ❌ 避免的命名
describe('WorkflowCanvas', () => {
  it('test1', () => {});
  it('should work', () => {});
});
```

### 2. 测试结构 (AAA 模式)

```typescript
it('adds node when add button is clicked', () => {
  // Arrange (准备)
  const mockOnNodesChange = jest.fn();
  renderWithProviders(
    <WorkflowCanvas {...defaultProps} onNodesChange={mockOnNodesChange} />
  );

  // Act (执行)
  fireEvent.click(screen.getByTestId('add-node'));

  // Assert (断言)
  expect(mockOnNodesChange).toHaveBeenCalledWith([
    { type: 'add', item: expect.any(Object) }
  ]);
});
```

### 3. Mock 使用

```typescript
// ✅ 有限 Mock
jest.mock('./api', () => ({
  fetchWorkflow: jest.fn().mockResolvedValue(mockWorkflow),
}));

// ✅ 功能性 Mock
jest.spyOn(api, 'fetchWorkflow').mockResolvedValue(mockWorkflow);

// ❌ 过度 Mock
jest.mock('react', () => ({
  createElement: jest.fn(),
  // 不应该 Mock 整个 React
}));
```

### 4. 异步测试

```typescript
// ✅ 使用 async/await
it('loads workflow data asynchronously', async () => {
  render(<WorkflowComponent />);

  await waitFor(() => {
    expect(screen.getByText('Workflow loaded')).toBeInTheDocument();
  });
});

// ✅ 使用 findBy
it('displays loading state', async () => {
  render(<WorkflowComponent />);

  expect(await screen.findByText('Loading...')).toBeInTheDocument();
});
```

### 5. 测试隔离

```typescript
beforeEach(() => {
  jest.clearAllMocks();
  // 清理副作用
});

afterEach(() => {
  // 清理 DOM
  cleanup();
});
```

## 🔧 调试测试

### 1. 使用 screen.debug()

```typescript
test('debug example', () => {
  render(<Component />);
  screen.debug(); // 打印当前 DOM 状态
  screen.debug(screen.getByTestId('specific-element')); // 打印特定元素
});
```

### 2. 使用 logRoles

```typescript
import { logRoles } from '@testing-library/dom';

test('accessibility check', () => {
  const { container } = render(<Component />);
  logRoles(container); // 打印可访问的角色
});
```

### 3. VS Code 调试

```json
// .vscode/launch.json
{
  "name": "Debug Jest Tests",
  "type": "node",
  "request": "launch",
  "program": "${workspaceFolder}/node_modules/.bin/jest",
  "args": ["--runInBand", "--no-cache", "${file}"],
  "console": "integratedTerminal",
  "internalConsoleOptions": "neverOpen"
}
```

## 📈 持续集成

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '18'
      - run: npm ci
      - run: npm run test:ci
      - uses: codecov/codecov-action@v1
        with:
          file: ./coverage/lcov.info
```

### 预提交钩子

```json
// package.json
{
  "husky": {
    "hooks": {
      "pre-commit": "lint-staged && npm run test:unit"
    }
  },
  "lint-staged": {
    "*.{ts,tsx}": [
      "eslint --fix",
      "jest --bail --findRelatedTests"
    ]
  }
}
```

## 🐛 常见问题

### 1. Mock 不生效

**问题**: Mock 没有替换实际实现
**解决**: 确保 Mock 在测试文件顶部，且在 import 之前

```typescript
// ✅ 正确
jest.mock('./module');
import { Component } from './module';

// ❌ 错误
import { Component } from './module';
jest.mock('./module');
```

### 2. 异步测试超时

**问题**: 异步测试失败或超时
**解决**: 增加超时时间或使用正确的等待策略

```typescript
// 增加超时
test('slow test', async () => {
  // 测试代码
}, 10000); // 10秒超时

// 或者在 jest.config.js 中设置
testTimeout: 10000,
```

### 3. React 更新警告

**问题**: React 更新相关的警告
**解决**: 使用 `act` 包装状态更新

```typescript
import { act, renderHook } from '@testing-library/react';

test('hook test', () => {
  const { result } = renderHook(() => useHook());

  act(() => {
    result.current.updateState();
  });
});
```

## 📚 参考资源

- [Jest 官方文档](https://jestjs.io/docs/getting-started)
- [React Testing Library 文档](https://testing-library.com/docs/react-testing-library/intro)
- [Testing Playground](https://testing-playground.com/)
- [React 测试最佳实践](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)

## 🤝 贡献指南

### 添加新测试

1. 为新功能编写对应的单元测试
2. 确保测试覆盖率不低于目标值
3. 添加集成测试验证组件协作
4. 更新本文档说明新增测试

### 测试规范

- 每个组件至少有一个测试文件
- 测试文件命名: `ComponentName.test.tsx`
- 测试描述清晰，使用 "should/when/then" 格式
- 保持测试独立性和可重复性

---

如有测试相关问题，请查看 [Jest 配置](../../jest.config.js) 或联系开发团队。