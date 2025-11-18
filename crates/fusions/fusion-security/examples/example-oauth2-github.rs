use std::env;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use url::Url;

use fusion_security::oauth::{OAuthClient, OAuthConfig, OAuthProvider};

/// ```sh
/// cargo run -p fusion-security --features="with-oauth" --example example-oauth2-github
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenvy::dotenv().unwrap();

  println!("=== GitHub OAuth 2.0 授权码模式示例 ===\n");

  // 配置环境变量或直接设置
  let client_id = env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| {
    eprintln!("❌ 错误: 缺少 GITHUB_CLIENT_ID 环境变量");
    eprintln!("请在 .env 文件中设置: GITHUB_CLIENT_ID=your_client_id");
    std::process::exit(1);
  });

  let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap_or_else(|_| {
    eprintln!("❌ 错误: 缺少 GITHUB_CLIENT_SECRET 环境变量");
    eprintln!("请在 .env 文件中设置: GITHUB_CLIENT_SECRET=your_client_secret");
    std::process::exit(1);
  });

  // 使用固定的本地回调地址
  let redirect_url = "http://localhost:8080".to_string();

  // 创建 OAuth 配置，包含 public_repo 和 user:email 权限
  let config = OAuthConfig {
    client_id,
    client_secret,
    redirect_url: redirect_url.clone(),
    scopes: vec!["public_repo".to_string(), "user:email".to_string()],
  };

  // 创建 GitHub OAuth 提供者
  let provider = OAuthProvider::github();

  // 创建 OAuth 客户端
  let oauth_client = OAuthClient::new(provider, &config)?;

  // 生成随机的状态参数用于 CSRF 防护
  let state = format!(
    "random_state_{}",
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()
  );
  let auth_url = oauth_client.get_authorize_url(&state);

  println!("1. 请在浏览器中打开以下 URL 进行授权:");
  println!("{}\n", auth_url);

  println!("2. 授权后，GitHub 会重定向到: {}", redirect_url);
  println!("   本地服务器会自动捕获授权码和状态参数\n");

  println!("正在等待 OAuth 回调...");

  let (code, returned_state) = {
    // 一个简单的本地服务器实现来捕获 OAuth 回调
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
      if let Ok((mut stream, _)) = listener.accept().await {
        let mut reader = BufReader::new(&mut stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        let redirect_url = request_line.split_whitespace().nth(1).unwrap_or("/");
        let url = Url::parse(&("http://localhost".to_string() + redirect_url))?;

        let code = url
          .query_pairs()
          .find(|(key, _)| key == "code")
          .map(|(_, code)| code.into_owned())
          .ok_or_else(|| "未找到授权码参数".to_string())?;

        let returned_state = url
          .query_pairs()
          .find(|(key, _)| key == "state")
          .map(|(_, state)| state.into_owned())
          .ok_or_else(|| "未找到状态参数".to_string())?;

        let message = "授权成功！请返回终端查看结果。";
        let response = format!(
          "HTTP/1.1 200 OK\r\ncontent-type: text/html; charset=utf-8\r\ncontent-length: {}\r\n\r\n{}",
          message.len(),
          message
        );
        stream.write_all(response.as_bytes()).await?;

        // 服务器在收集第一个代码后自动终止
        break (code, returned_state);
      }
    }
  };

  println!("\n✅ GitHub 返回了以下授权码:");
  println!("{}\n", code);
  println!("✅ GitHub 返回了以下状态参数:");
  println!("{} (预期值: `{}`)\n", returned_state, state);

  // 验证状态参数防止 CSRF 攻击
  if returned_state != state {
    println!("❌ 错误: 状态参数不匹配，可能存在 CSRF 攻击");
    return Ok(());
  }

  println!("正在使用授权码换取访问令牌...");

  // 使用授权码换取访问令牌
  let user_id = "demo_user_github";
  match oauth_client.exchange_code(&code, &state, user_id).await {
    Ok(token_response) => {
      println!("✅ 成功获取访问令牌:");
      println!("  Access Token: {}", token_response.access_token);
      println!("  Token Type: {}", token_response.token_type);
      if let Some(expires_in) = token_response.expires_in {
        println!("  Expires In: {} 秒", expires_in);
      }
      if let Some(refresh_token) = token_response.refresh_token {
        println!("  Refresh Token: {}", refresh_token);
      }
      if let Some(scope) = token_response.scope {
        println!("  Scope: {}", scope);
      }

      println!("\n获取用户信息...");

      // 使用访问令牌获取用户信息（自动处理令牌续期）
      match oauth_client.get_user_info(user_id).await {
        Ok(user_info) => {
          println!("✅ 成功获取用户信息:");
          println!("  ID: {}", user_info.id);
          println!("  Username: {}", user_info.username);
          if let Some(name) = user_info.name {
            println!("  Name: {}", name);
          }
          if let Some(email) = user_info.email {
            println!("  Email: {}", email);
          }
          if let Some(avatar_url) = user_info.avatar_url {
            println!("  Avatar URL: {}", avatar_url);
          }
          println!("  Provider: {}", user_info.provider);
        }
        Err(e) => {
          println!("❌ 获取用户信息失败: {}", e);
        }
      }
    }
    Err(e) => {
      println!("❌ 换取访问令牌失败: {}", e);
    }
  }

  println!("\n=== OAuth 流程完成 ===");

  Ok(())
}
