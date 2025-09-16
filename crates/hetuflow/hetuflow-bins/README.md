# hetuflow-bins

## 项目结构

```shell
├── bin
│   ├── hetuflow-agent.rs     # Agent 端：hetuflow 任务执行、任务进程管理、……
│   └── hetuflow-server.rs    # Server 端：hetuflow 管理、任务调度、……
├── Cargo.toml
├── README.md
├── resources
│   ├── hetuflow-agent.toml   # Agent 端配置文件
│   └── hetuflow-server.toml  # Server 端配置文件
├── src
│   └── lib.rs
├── tests
│   ├── common                # 测试用的环境
│   ├── integration           # 集成测试模块
│   └── mod.rs
```

- [hetuflow-ddl.sql](../../scripts/software/postgres/sqls/hetuflow-ddl.sql): 数据库初始化脚本
