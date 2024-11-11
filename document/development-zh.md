# Backend

开发环境设置

## Rust 开发环境

### 安装 Rust

Linux、MacOS 使用以下命案安装：

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Windows 下载 [rustup-init.exe](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe) 进行安装。

> 更多 rustup-init 安装方法见：[Other Rust Installation Methods](https://forge.rust-lang.org/infra/other-installation-methods.html)

### 安装 LLVM 编译器

MacOS:

```bash
brew install llvm
```

Linux:

- Rocky/Fedora/RHEL, `sudo dnf install lld clang`
- Debian/Ubuntu, `sudo apt-get install lld clang`
- Arch, `sudo pacman -S lld clang`

Windows:

```powershell
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```

## 使用 Docker 启动开发环境依赖

### 安装 Docker

- Windows/Linux 推荐使用 [Docker Desktop](https://www.docker.com/products/docker-desktop/)。
- Mac 系统推荐使用 [orbstack](https://orbstack.dev/download)。

### 启动 docker compose

进入 `fusion-data` 项目根目录，执行以下命令使用 `docker compose` 启动依赖服务：

```bash
docker compose up -d
```

可以使用 `docker compose logs -f db` 命令实时查看 PG 数据库日志，以方便查看 SQL 语句。

> 在开发阶段，也许你不需要启动所有服务依赖。比如：jaeger 和 opentelemetry。这时可以使用 `docker compose up -d db` 命令只启动数据库。

### Opentelemetry

docker compose 默认将启动 jaeger 作为 opentelemetry 的 collector，端口为 4317。在 Rust 服务中要启用 opentelemtry 功能需要启用 `tracing.otel` 模块，在 `app.toml` 中添加以下配置：

```toml
[ultimate.log.otel]
enable = true
exporter_otlp_endpoint = "http://localhost:4317"
traces_sample = "always_on"
```

### 重建 Docker Containers

有时候需要重建 Docker Containers，比如：更新 [SQL 脚本](../scripts/software/postgres/sqls/)、…… 可以通过 `docker compose down -v` 命令删除所有容器及关联的容器磁盘卷，然后重新启动。

```sh
docker compose up -d
```

> 根据需要，可以添加 `--build` 选项以重新构建 docker images。

## IDE

_需要提前安装好 Rust 编译环境，见：[Rust 开发环境](#rust-开发环境)_

推荐使用 [VSCode](https://code.visualstudio.com/) 作为开发工具。使用 VSCode 打开项目时将会自动识别并安装依赖的插件。

你也可以手动安装以下插件：

- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [dependi](https://marketplace.visualstudio.com/items?itemName=fill-labs.dependi)
- [ReneSaarsoo.sql-formatter-vsc](https://marketplace.visualstudio.com/items?itemName=ReneSaarsoo.sql-formatter-vsc)（可选）

> 也可以使用 [Jetbrains RustRover](https://www.jetbrains.com/rust/)，使用 RustRover 不需要额外安装插件。
