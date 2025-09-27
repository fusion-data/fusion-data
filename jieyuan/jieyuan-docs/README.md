# 界垣

## Access Token

- JSON WEB Token（JWT 令牌）:
- JSON WEB Encryption 令牌）: 对 JWT 的 payload 进行加密，应首选。但要考虑到客户端对加解密库（及 publich key）的支持情况。若客户端不需要关心 payload 内容，也许使用 不透明令牌即可。
- Opaque Access Token（不透明令牌）: 适合 Session 类型访问令牌。
  - 在撰写本文时，获取安全不透明令牌的最佳实践是使用加密安全的随机字符串生成器生成至少 128 位熵的令牌。
