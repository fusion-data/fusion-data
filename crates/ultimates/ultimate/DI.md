# 编译期依赖注入

提供了一种特殊的 `Component`，它支持在编译期注入依赖的组件。

像下面的例子 `UserService` 只需派生 `Service` 特征，为了区分注入的依赖，你需要通过属性宏 `#[inject(component)]` 和 `#[inject(config)]` 指定依赖是一个 `Component` 还是一个 `Config`。

```rust
use std::sync::Arc;

use ultimate::{application::Application, component::Component};

#[derive(Component, Clone)]
pub struct AuthSvc {
  #[component]
  user_svc: UserSvc,
  #[component]
  pwd_svc: PwdSvc,
}

#[derive(Clone, Component)]
pub struct UserSvc {
  #[component]
  db: Db,
}

#[derive(Clone, Component)]
pub struct PwdSvc {
  pwd_generator: Arc<PwdGenerator>,
}

// #[derive(Debug, Clone)]
// pub struct PwdSvc2 {
//   pwd_generator: Arc<PwdGenerator>,
// }

#[derive(Debug, Default)]
pub struct PwdGenerator {}

#[derive(Clone, Component)]
pub struct Db {}

#[tokio::main]
async fn main() -> ultimate::Result<()> {
  Application::builder().build().await?;

  let _auth_svc: AuthSvc = Application::global().component();

  // let _pwd_svc = PwdSvc2 { pwd_generator: Arc::default() };
  // println!("{:?}", _pwd_svc);

  Ok(())
}
```
