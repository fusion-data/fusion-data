use rmcp::{Peer, RoleClient, ServiceExt, model::CallToolRequestParams, transport::StreamableHttpClientTransport};
use serde_json::json;
use std::error::Error;

/// 腾讯位置服务 MCP 客户端 DEMO
///
/// 此示例演示如何使用 rmcp 客户端连接腾讯位置服务 MCP 服务器
/// 并调用地理位置相关的工具
///
/// ## 使用前准备
///
/// 1. **注册腾讯位置服务账号**
///    - 访问: https://lbs.qq.com/
///    - 注册并登录控制台
///
/// 2. **创建应用并获取 API Key**
///    - 在控制台中创建新应用
///    - 开启 WebService API 功能
///    - 获取 API Key
///
/// 3. **配置 API Key**
///    - 将下方代码中的 YOUR_API_KEY_HERE 替换为实际的 API Key
///
/// 4. **运行示例**
///    ```bash
///    cargo run -p fusion-ai --example tencent-map
///    ```
///
/// ## 支持的功能
///
/// - **IP 定位**: 根据 IP 地址获取地理位置信息
/// - **地址解析**: 将地址转换为经纬度坐标
/// - **地点搜索**: 在指定城市搜索 POI 信息
/// - **路线规划**: 计算两点间的驾车路线
///
/// ## 相关文档
///
/// - 官方 MCP 文档: https://lbs.qq.com/service/MCPServer/MCPServerGuide/userGuide
/// - WebService API 文档: https://lbs.qq.com/service/webService/webServiceGuide/overview
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenvy::dotenv()?;
  println!("🚀 腾讯位置服务 MCP 客户端 DEMO - Streamable HTTP Client");
  println!("📖 使用 RMCP (Rust MCP Client) 调用腾讯位置服务");

  // 腾讯位置服务 MCP 服务器地址
  // 注意：需要替换 <YourKey> 为实际的 API Key
  // 官方文档: https://lbs.qq.com/service/MCPServer/MCPServerGuide/userGuide
  // Streamable HTTP 方式接入地址: https://mcp.map.qq.com/mcp?key=<YourKey>&format=0
  // SSE 方式接入地址: https://mcp.map.qq.com/sse?key=<YourKey>&format=0
  let api_key = std::env::var("TENCENT_MAP_KEY")?; // 请替换为您的腾讯位置服务 API Key
  let server_url = format!("https://mcp.map.qq.com/mcp?key={}&format=0", api_key);

  println!("正在连接腾讯位置服务 MCP 服务器: {}", server_url);

  // 创建 Streamable HTTP 客户端传输层
  let transport = StreamableHttpClientTransport::from_uri(server_url);

  // 创建并启动 MCP 客户端服务
  let service = ().serve(transport).await?;

  println!("✅ 成功连接到腾讯位置服务 MCP 服务器");

  // 获取服务器信息
  let server_info = service.peer_info().unwrap();
  println!("📋 服务器信息: {}", serde_json::to_string_pretty(server_info).unwrap());

  // 获取可用工具列表
  match service.list_tools(Default::default()).await {
    Ok(tools) => {
      println!("\n🔧 可用工具列表:");
      for tool in &tools.tools {
        println!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("无描述"));
      }
    }
    Err(e) => {
      eprintln!("❌ 获取工具列表失败: {}", e);
    }
  }

  // 示例1: IP 定位
  println!("\n🌍 示例1: IP 定位");
  if let Err(e) = demo_ip_location(&service).await {
    eprintln!("❌ IP 定位示例失败: {}", e);
  }

  // 示例2: 地址解析
  println!("\n📍 示例2: 地址解析");
  if let Err(e) = demo_geocoding(&service).await {
    eprintln!("❌ 地址解析示例失败: {}", e);
  }

  // 示例3: 城市搜索
  println!("\n🔍 示例3: 城市搜索");
  if let Err(e) = demo_city_search(&service).await {
    eprintln!("❌ 城市搜索示例失败: {}", e);
  }

  // 示例4: 驾车路线规划
  println!("\n🚗 示例4: 驾车路线规划");
  if let Err(e) = demo_driving_route(&service).await {
    eprintln!("❌ 驾车路线规划示例失败: {}", e);
  }

  println!("\n✅ DEMO 演示完成");

  // 优雅关闭连接
  service.cancel().await?;

  Ok(())
}

/// IP 定位示例
/// 根据 IP 地址获取地理位置信息
async fn demo_ip_location(service: &Peer<RoleClient>) -> Result<(), Box<dyn Error>> {
  let params = CallToolRequestParams::new("ipLocation").with_arguments(
    json!({
      "ip": "117.59.114.33",
      "coord_type": "5",  // 坐标类型：5-腾讯坐标
      "get_poi": "1"      // 是否返回周边POI列表：0-不返回，1-返回
    })
    .as_object()
    .unwrap()
    .clone(),
  );

  println!("  📍 正在查询 IP: 117.59.114.33 的地理位置...");
  let result = service.call_tool(params).await?;
  println!("  结果: {}", serde_json::to_string_pretty(&result).unwrap());
  Ok(())
}

/// 地址解析示例（地理编码）
/// 将地址转换为经纬度坐标
async fn demo_geocoding(service: &Peer<RoleClient>) -> Result<(), Box<dyn Error>> {
  let params = CallToolRequestParams::new("geocoder").with_arguments(
    json!({
      "address": "北京市海淀区北四环西路66号",
      "region": "北京",     // 指定地址所在城市
      "coord_type": "1"     // 返回坐标类型：1-GPS坐标
    })
    .as_object()
    .unwrap()
    .clone(),
  );

  println!("  🗺️ 正在解析地址: 北京市海淀区北四环西路66号...");
  let result = service.call_tool(params).await?;
  println!("  结果: {}", serde_json::to_string_pretty(&result).unwrap());
  Ok(())
}

/// 地点搜索示例
/// 在指定城市搜索地点POI信息
async fn demo_city_search(service: &Peer<RoleClient>) -> Result<(), Box<dyn Error>> {
  let params = CallToolRequestParams::new("placeSearchNearby").with_arguments(
    json!({
      "keyword": "洪崖洞",
      "location": "重庆",            // 搜索范围：重庆市
      "page_size": "10",            // 每页条目数
      "page_index": "1"             // 页码
    })
    .as_object()
    .unwrap()
    .clone(),
  );

  println!("  🔍 正在搜索: 洪崖洞 (重庆)...");
  let result = service.call_tool(params).await?;
  println!("  结果: {}", serde_json::to_string_pretty(&result).unwrap());
  Ok(())
}

/// 驾车路线规划示例
/// 计算两点间的驾车路线
async fn demo_driving_route(service: &Peer<RoleClient>) -> Result<(), Box<dyn Error>> {
  let params = CallToolRequestParams::new("directionDriving").with_arguments(
    json!({
      "from": "39.908823,116.397470", // 天安门坐标
      "to": "39.999718,116.326364",   // 鸟巢坐标
      "policy": "0",                  // 路线策略：0-参考实时路况的最快捷路线
      "from_poi": "天安门",           // 起点POI名称
      "to_poi": "鸟巢"                // 终点POI名称
    })
    .as_object()
    .unwrap()
    .clone(),
  );

  println!("  🚗 正在规划路线: 天安门 → 鸟巢...");
  let result = service.call_tool(params).await?;
  println!("  结果: {}", serde_json::to_string_pretty(&result).unwrap());
  Ok(())
}
