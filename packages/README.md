# Packages

这个目录用于存放可复用的库和组件包。

每个子目录应该是一个独立的 npm 包，具有自己的 `package.json` 文件。

## 示例结构

```
packages/
├── ui-components/     # UI 组件库
├── utils/            # 工具函数库
├── shared-types/     # 共享类型定义
└── config/           # 共享配置
```

## 创建新包

要创建新的包，请运行：

```bash
mkdir packages/your-package-name
cd packages/your-package-name
pnpm init
```
