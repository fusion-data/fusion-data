# hetumind-studio

## 运行测试

### 运行所有凭证测试

```
cargo test --test api_credentials_test -- --nocapture --ignored
```

### 运行特定测试

```
cargo test --test api_credentials_test test_create_get_delete_credential -- --nocapture --ignored
```

所有测试都使用 `#[ignore]` 标记，需要添加 `--ignored` 参数才能运行，这样可以避免在常规测试中执行集成测试。
