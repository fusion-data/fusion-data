# 腾讯位置服务 MCP 客户端示例

本目录包含使用 RMCP (Rust MCP Client) 调用腾讯位置服务的示例代码。

## 示例列表

### tencent-map.rs

腾讯位置服务 MCP 客户端 DEMO，演示如何连接腾讯位置服务 MCP 服务器并调用各种地理位置相关的工具。

**支持的功能：**
- IP 定位：根据 IP 地址获取地理位置信息
- 地址解析：将地址转换为经纬度坐标
- 地点搜索：在指定城市搜索 POI 信息
- 路线规划：计算两点间的驾车路线

## 使用前准备

1. **注册腾讯位置服务账号**
   - 访问 [腾讯位置服务](https://lbs.qq.com/)
   - 注册并登录控制台

2. **创建应用并获取 API Key**
   - 在控制台中创建新应用
   - 开启 WebService API 功能
   - 获取 API Key

3. **配置 API Key**
   - 编辑 `tencent-map.rs` 文件
   - 将 `YOUR_API_KEY_HERE` 替换为实际的 API Key

## 运行示例

```bash
# 运行腾讯位置服务 MCP 客户端示例
cargo run -p fusion-ai --example tencent-map
```

## 相关文档

- [腾讯位置服务 MCP Server 用户指南](https://lbs.qq.com/service/MCPServer/MCPServerGuide/userGuide)
- [腾讯位置服务 WebService API 概览](https://lbs.qq.com/service/webService/webServiceGuide/overview)
- [RMCP 库文档](https://docs.rs/rmcp/)

## 注意事项

- 使用前请确保已获取有效的腾讯位置服务 API Key
- 请遵守腾讯位置服务的使用条款和配额限制
- 示例中的坐标和地址仅供演示使用