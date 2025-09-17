# Apps

这个目录用于存放应用程序。

每个子目录应该是一个完整的应用，可以是：

- Web 应用（React、Vue、Angular 等）
- 服务端应用（Express、Fastify 等）
- CLI 工具
- 桌面应用

## 示例结构

```
apps/
├── web-app/          # 前端 Web 应用
├── admin-panel/      # 管理后台
├── api-server/       # API 服务器
└── cli-tool/         # 命令行工具
```

## 创建新应用

要创建新的应用，请运行：

```bash
mkdir apps/your-app-name
cd apps/your-app-name
pnpm init
```
