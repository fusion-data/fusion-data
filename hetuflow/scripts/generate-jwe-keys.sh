#!/bin/bash

# JWE 密钥生成脚本
# 用于生成 Hetuflow JWE Token 认证所需的 ECDH-ES 密钥对

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 openssl 是否可用
check_openssl() {
  if ! command -v openssl &> /dev/null; then
    print_error "OpenSSL 未安装或不在 PATH 中"
    print_info "请安装 OpenSSL: brew install openssl (macOS) 或 apt-get install openssl (Ubuntu)"
    exit 1
  fi
  print_info "OpenSSL 版本: $(openssl version)"
}

# 生成 ECDH-ES 密钥对
generate_keys() {
  local output_dir="${1:-./keys}"
  local private_key_file="$output_dir/jwe-private.pem"
  local public_key_file="$output_dir/jwe-public.pem"
  
  print_info "创建输出目录: $output_dir"
  mkdir -p "$output_dir"
  
  print_info "生成 ECDH-ES P-256 私钥..."
  openssl ecparam -genkey -name prime256v1 -noout -out "$private_key_file"
  
  print_info "从私钥提取公钥..."
  openssl ec -in "$private_key_file" -pubout -out "$public_key_file"
  
  print_success "密钥对生成完成!"
  print_info "私钥文件: $private_key_file"
  print_info "公钥文件: $public_key_file"
}

# 显示密钥内容（用于配置文件）
show_keys() {
  local output_dir="${1:-./keys}"
  local private_key_file="$output_dir/jwe-private.pem"
  local public_key_file="$output_dir/jwe-public.pem"
  
  if [[ ! -f "$private_key_file" ]] || [[ ! -f "$public_key_file" ]]; then
    print_error "密钥文件不存在，请先生成密钥"
    return 1
  fi
  
  echo
  print_info "=== 配置文件格式 ==="
  echo
  
  print_info "Server 端配置 (hetuflow-server/resources/app.toml):"
  echo "[hetuflow.jwe]"
  echo -n "private_key = \""
  # 将私钥内容转换为单行，用\n表示换行
  sed ':a;N;$!ba;s/\n/\\n/g' "$private_key_file"
  echo "\""
  echo -n "public_key = \""
  sed ':a;N;$!ba;s/\n/\\n/g' "$public_key_file"
  echo "\""
  echo "key_agreement_algorithm = \"ECDH-ES\""
  echo "content_encryption_algorithm = \"A256GCM\""
  echo "token_ttl = 3600"
  
  echo
  print_info "Agent 端配置 (hetuflow-agent/resources/app.toml):"
  echo "[hetuflow.agent]"
  echo "# JWE Token (由 Server 端生成)"
  echo "# jwe_token = \"eyJhbGciOiJFQ0RILUVTK...\""
  
  echo
  print_warning "注意事项:"
  echo "1. 私钥仅用于 Server 端解密 JWE Token"
  echo "2. 公钥用于 Server 端生成 JWE Token"
  echo "3. Agent 端只需要配置由 Server 生成的 JWE Token"
  echo "4. 请妥善保管私钥文件，不要提交到版本控制系统"
  echo "5. 建议在生产环境中使用环境变量或密钥管理系统"
}

# 生成示例配置文件
generate_config_examples() {
  local output_dir="${1:-./keys}"
  local server_config="$output_dir/server-jwe-config.toml"
  local agent_config="$output_dir/agent-jwe-config.toml"
  local private_key_file="$output_dir/jwe-private.pem"
  local public_key_file="$output_dir/jwe-public.pem"
  
  if [[ ! -f "$private_key_file" ]] || [[ ! -f "$public_key_file" ]]; then
    print_error "密钥文件不存在，请先生成密钥"
    return 1
  fi
  
  print_info "生成配置文件示例..."
  
  # Server 配置示例
  cat > "$server_config" << EOF
# Hetuflow Server JWE 配置示例
# 将以下内容添加到 hetuflow-server/resources/app.toml

[hetuflow.jwe]
# 私钥（用于解密 JWE Token）
private_key = "$(sed ':a;N;$!ba;s/\n/\\n/g' "$private_key_file")"
# 公钥（用于加密 JWE Token）
public_key = "$(sed ':a;N;$!ba;s/\n/\\n/g' "$public_key_file")"
# 密钥协商算法
key_agreement_algorithm = "ECDH-ES"
# 内容加密算法
content_encryption_algorithm = "A256GCM"
# Token 有效期（秒）
token_ttl = 3600
EOF
  
  # Agent 配置示例
  cat > "$agent_config" << EOF
# Hetuflow Agent JWE 配置示例
# 将以下内容添加到 hetuflow-agent/resources/app.toml

[hetuflow.agent]
# JWE Token（由 Server 端生成和分发）
# 取消注释并填入实际的 JWE Token
# jwe_token = "eyJhbGciOiJFQ0RILUVTK0EyNTZLVyIsImVuYyI6IkEyNTZHQ00iLCJ0eXAiOiJKV0UifQ..."
EOF
  
  print_success "配置文件示例已生成:"
  print_info "Server 配置: $server_config"
  print_info "Agent 配置: $agent_config"
}

# 显示帮助信息
show_help() {
  echo "JWE 密钥生成脚本"
  echo
  echo "用法: $0 [选项] [输出目录]"
  echo
  echo "选项:"
  echo "  -h, --help     显示此帮助信息"
  echo "  -g, --generate 生成新的密钥对（默认操作）"
  echo "  -s, --show     显示现有密钥的配置格式"
  echo "  -c, --config   生成配置文件示例"
  echo "  -a, --all      执行所有操作（生成密钥、显示配置、生成示例）"
  echo
  echo "参数:"
  echo "  输出目录       密钥文件的输出目录（默认: ./keys）"
  echo
  echo "示例:"
  echo "  $0                    # 在 ./keys 目录生成密钥"
  echo "  $0 /path/to/keys      # 在指定目录生成密钥"
  echo "  $0 -s ./keys          # 显示现有密钥的配置格式"
  echo "  $0 -a ./keys          # 执行所有操作"
}

# 主函数
main() {
  local action="generate"
  local output_dir="./keys"
  
  # 解析命令行参数
  while [[ $# -gt 0 ]]; do
    case $1 in
      -h|--help)
        show_help
        exit 0
        ;;
      -g|--generate)
        action="generate"
        shift
        ;;
      -s|--show)
        action="show"
        shift
        ;;
      -c|--config)
        action="config"
        shift
        ;;
      -a|--all)
        action="all"
        shift
        ;;
      -*)
        print_error "未知选项: $1"
        show_help
        exit 1
        ;;
      *)
        output_dir="$1"
        shift
        ;;
    esac
  done
  
  print_info "Hetuflow JWE 密钥生成工具"
  print_info "输出目录: $output_dir"
  echo
  
  check_openssl
  
  case $action in
    "generate")
      generate_keys "$output_dir"
      ;;
    "show")
      show_keys "$output_dir"
      ;;
    "config")
      generate_config_examples "$output_dir"
      ;;
    "all")
      generate_keys "$output_dir"
      echo
      show_keys "$output_dir"
      echo
      generate_config_examples "$output_dir"
      ;;
  esac
  
  echo
  print_success "操作完成!"
}

# 执行主函数
main "$@"