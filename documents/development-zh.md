# 开发环境配置

本项目为多语言多模块工程，包含 Rust、Node.js 相关依赖，并通过 Docker 管理部分服务。请按照以下步骤完成开发环境配置。

## 1. 基础环境要求

- **Rust**：使用 [rustup](https://www.rust-lang.org/tools/install)，建议使用 [rsproxy](https://rsproxy.cn/)
  以加速在车内访问。版本需 ≥ 1.87（见 `Cargo.toml`）。
- **Node.js**：官网[下载](https://nodejs.org/zh-cn/download)，建议使用 [nvm](https://github.com/nvm-sh/nvm)
  管理，推荐版本 ≥ 18。
- **pnpm**：推荐使用 [pnpm](https://pnpm.io/zh/installation) 作为 Node 包管理器。
- **Docker & Docker Compose**：用于本地数据库及相关服务的启动。

## 2. Rust 依赖安装

在项目根目录下执行：

```shell
cargo check
cargo build
```

如需格式化和代码检查：

```shell
cargo fmt
cargo clippy --workspace --all-targets --all-features -- -D
```

### 如需运行测试：

```shell
cargo test
```

`cargo` 提供了细粒度的命令来选择要运行的测试类型：

- `--lib`：只运行库（library）的单元测试和文档测试。
- `--bins`：只运行二进制文件（binary executables）的单元测试。
- `--doc`：只运行文档测试。

如： `hetumind` 包主要是一个库（即 `src/lib.rs` 是主要入口），您可以使用以下命令：

```shell
cargo test -p hetumind-studio --lib
```

这个命令会运行 `hetumind` 包中定义在 `src` 目录下的所有单元测试和文档测试。

如果您的项目中同时包含库（`src/lib.rs`）和二进制文件（如 `src/main.rs`），并且您想同时测试它们（但不包括 `tests`
目录下的集成测试），可以使用：

```shell
cargo test -p hetumind-studio --lib --bins
```

这条命令会运行库和所有二进制目标中的单元测试和文档测试，但会排除 `tests` 目录中的所有集成测试。

### 运行程序

```shell
cargo run -p hetumind-studio --bin hetumind-studio
```

`-p` 指定 crate 包，`--bin` 指定要运行的程序（不指定 `--bin` 时，默认运行与 crate 包同名程序）

## 3. Node.js 依赖安装

如有前端或 Node.js 相关子项目（如 `web`、`admin` 等），请进入对应目录，执行：

```shell
pnpm install
```

> **注意**：`pnpm-lock.yaml` 已锁定依赖版本，首次安装请勿删除或修改该文件。

## 4. 数据库与服务启动（Docker）

项目根目录下有 `docker-compose.yml`，用于启动开发所需的数据库和服务。常见服务如 PostgreSQL、Redis、MinIO、OSS/S3 等。

启动命令：

```shell
docker-compose up -d
```

查看服务状态：

```shell
docker-compose ps
```

关闭服务：

```shell
docker-compose down
```

如需重建服务（清空数据）：

```shell
docker-compose down -v
docker-compose up -d
```

## 5. 运行后端服务

以 `hetumind` 为例，运行命令如下：

```shell
cargo run -p hetumind-studio --bin hetumind-studio
```

如有其它二进制或子项目，参考 `Cargo.toml` 的 `[workspace]` 配置。

## 6. 其它说明

- **环境变量**：如需自定义数据库、Redis 等连接信息，可在根目录下创建 `.env` 文件，参考 `.env.example` 或
  `docker-compose.yml` 中的环境变量配置。
- **依赖管理**：Rust 依赖统一在 `Cargo.toml` 的 `[workspace.dependencies]` 管理，Node 依赖由各子项目的 `package.json` 和
  `pnpm-lock.yaml` 管理。
- **常用命令**：见本项目 AI 助手推荐的命令模式。

## 7. 常见问题

- **依赖编译失败**：请确保 Rust、Node、pnpm、Docker 版本均符合要求，并已正确安装。
- **端口冲突**：如本地端口被占用，可修改 `docker-compose.yml` 或相关配置文件中的端口映射。
- **数据库连接失败**：确认 Docker 服务已启动，且 `.env` 配置正确。

---

如有更多问题，请查阅各子模块 README 或联系项目维护者。
