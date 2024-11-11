# Backend

## 开发环境设置

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

- Rocky/Fedora, `sudo dnf install lld clang`
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
docker compose up -d --build
```

可以使用 `docker compose logs -f db` 命令实时查看 PG 数据库日志，以方便查看 SQL 语句。

#### Opentelemetry

docker compose 默认将启动 jaeger 作为 opentelemetry 的 collector，端口为 4317。在 Rust 服务中要启用 opentelemtry 功能需要启用 `tracing.otel` 模块，在 `app.toml` 中添加以下配置：

```toml
[ultimate.tracing.otel]
enable = true
exporter_otlp_endpoint = "http://localhost:4317"
traces_sample = "always_on"
```
