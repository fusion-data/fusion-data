# Hetumind CLI

强大而灵活的 Hetumind 工作流引擎命令行工具。

## 🚀 概述

`hetumind-cli` 是 Hetumind 工作流引擎的官方命令行工具，为开发者和管理员提供高效、自动化的方式来管理和执行工作流。它支持从本地开发、测试到服务器部署的全过程。

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/guixuflow/guixuflow.git
cd guixuflow

# 构建 CLI 工具
cargo build --release --bin hetumind-studio-cli

# 将二进制文件添加到 PATH
cp target/release/hetumind-studio-cli /usr/local/bin/
```

### 验证安装

```bash
hetumind-studio-cli --version
```

## ⚙️ 配置

### 初始配置

CLI 工具需要连接到 Hetumind API 服务器。配置文件位于 `~/.hetumind/config.toml`：

```toml
[api]
endpoint = "http://127.0.0.1:8080"
token = "your-api-token-here"
```

### 环境变量

你也可以通过环境变量指定配置文件路径：

```bash
export GUIXU_CONFIG_PATH="/path/to/your/config.toml"
```

## 🛠️ 使用指南

### 基本语法

```bash
hetumind-studio-cli <COMMAND> [OPTIONS]
```

或使用短别名：

```bash
hetumind-studio-cli wf <SUBCOMMAND> [OPTIONS]
```

### 命令概览

| 命令       | 描述                 | 需要 API |
| ---------- | -------------------- | -------- |
| `new`      | 创建新的工作流文件   | ❌       |
| `validate` | 验证工作流文件       | ❌       |
| `list`     | 列出服务器上的工作流 | ✅       |
| `run`      | 运行工作流           | ✅       |
| `import`   | 导入工作流到服务器   | ✅       |
| `export`   | 从服务器导出工作流   | ✅       |

## 📋 命令详解

### 1. 创建新工作流

从模板创建新的工作流定义文件：

```bash
# 使用默认模板创建工作流
hetumind-studio-cli workflow new --name "my-workflow"

# 使用空模板创建工作流
hetumind-studio-cli workflow new --name "empty-workflow" --template empty

# 指定输出路径
hetumind-studio-cli workflow new --name "custom-workflow" --output "/path/to/workflow.json"
```

#### 可用模板

- **`default`**: 包含基本结构的默认模板
- **`empty`**: 空工作流模板，只包含基本字段

### 2. 验证工作流

验证本地工作流文件的语法和结构：

```bash
# 验证工作流文件
hetumind-studio-cli workflow validate ./my-workflow.json

# 验证会检查：
# - JSON 语法正确性
# - 工作流结构完整性
# - 节点连接有效性
```

### 3. 列出工作流

列出服务器上所有可用的工作流：

```bash
# 列出所有工作流（默认20个）
hetumind-studio-cli workflow list

# 按状态过滤
hetumind-studio-cli workflow list --status active

# 指定返回数量
hetumind-studio-cli workflow list --limit 50

# 组合使用
hetumind-studio-cli workflow list --status draft --limit 10
```

### 4. 运行工作流

执行工作流，支持通过 ID 或文件路径：

```bash
# 通过工作流ID运行
hetumind-studio-cli workflow run "550e8400-e29b-41d4-a716-446655440000"

# 通过本地文件运行
hetumind-studio-cli workflow run ./my-workflow.json

# 提供输入数据
hetumind-studio-cli workflow run "workflow-id" --input ./input-data.json

# 同步执行（等待完成）
hetumind-studio-cli workflow run "workflow-id" --sync

# 组合使用
hetumind-studio-cli workflow run ./my-workflow.json --input ./data.json --sync
```

#### 输入数据格式

输入数据文件应为 JSON 格式：

```json
{
  "user_id": "12345",
  "process_type": "batch",
  "data_source": "/path/to/data",
  "options": {
    "parallel": true,
    "timeout": 300
  }
}
```

### 5. 导入工作流

将本地工作流文件导入到服务器：

```bash
# 导入工作流文件
hetumind-studio-cli workflow import ./my-workflow.json
```

### 6. 导出工作流

从服务器导出工作流定义：

```bash
# 导出到标准输出
hetumind-studio-cli workflow export "workflow-id"

# 导出到文件
hetumind-studio-cli workflow export "workflow-id" --output ./exported-workflow.json

# 指定输出格式
hetumind-studio-cli workflow export "workflow-id" --format json --output ./workflow.json
```

## 🔧 高级用法

### 批量操作

结合 shell 脚本进行批量操作：

```bash
#!/bin/bash

# 批量验证多个工作流文件
for file in workflows/*.json; do
    echo "验证 $file..."
    hetumind-studio-cli workflow validate "$file"
done

# 批量导入工作流
for file in workflows/*.json; do
    echo "导入 $file..."
    hetumind-studio-cli workflow import "$file"
done
```

### 配置管理

为不同环境使用不同配置：

```bash
# 开发环境
export GUIXU_CONFIG_PATH="~/.hetumind/dev-config.toml"
hetumind-studio-cli workflow list

# 生产环境
export GUIXU_CONFIG_PATH="~/.hetumind/prod-config.toml"
hetumind-studio-cli workflow list
```

### 调试和故障排除

```bash
# 显示详细版本信息
hetumind-studio-cli --version

# 显示帮助信息
hetumind-studio-cli --help
hetumind-studio-cli workflow --help
hetumind-studio-cli workflow new --help

# 检查配置文件位置
ls -la ~/.hetumind-studio/
```

## 📝 示例工作流

### 完整示例：创建、验证、导入和运行

```bash
# 1. 创建新工作流
hetumind-studio-cli workflow new --name "data-processing" --template default

# 2. 验证工作流（在编辑后）
hetumind-studio-cli workflow validate ./data-processing.json

# 3. 导入到服务器
hetumind-studio-cli workflow import ./data-processing.json

# 4. 准备输入数据
cat > input.json << EOF
{
  "source_path": "/data/input",
  "output_path": "/data/output",
  "batch_size": 1000
}
EOF

# 5. 运行工作流
hetumind-studio-cli workflow run "data-processing-id" --input ./input.json --sync
```

## 🚨 常见问题

### Q: 如何解决"配置验证失败"错误？

**A:** 确保配置文件格式正确且包含有效的 API 令牌：

```toml
[api]
endpoint = "http://your-server:8080"
token = "your-valid-token"
```

### Q: 工作流验证失败怎么办？

**A:** 检查以下几点：

- JSON 语法是否正确
- 必需字段是否完整
- 节点连接是否有效
- 节点类型是否支持

### Q: 如何获取 API 令牌？

**A:** 请联系你的 Hetumind 管理员获取 API 访问令牌。

### Q: 支持哪些输出格式？

**A:** 目前支持：

- `json` (默认)
- `yaml` (计划中)

## 🔗 相关链接

- [Hetumind 文档](https://your-docs-site.com)
- [API 参考](https://your-api-docs.com)
- [GitHub 仓库](https://github.com/guixuflow/guixuflow)
- [问题反馈](https://github.com/guixuflow/guixuflow/issues)

## 📄 许可证

本项目采用 [MIT License](../../LICENSE) 许可证。

---

**提示**: 使用 `hetumind-cli workflow --help` 查看最新的命令帮助信息。
