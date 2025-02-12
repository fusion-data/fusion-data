# fusion-flow

## 运行

```sh
# Master & Scheduler 节点
cargo run -p fusion-flow

# Worker 节点
cargo run -p fusion-flow-worker
```

## grpcurl

```sh
grpcurl -plaintext localhost:58050 describe


grpcurl -plaintext -proto fusion_flow_api/v1/flow_api.proto \
  -import-path ./crates/fusions/fusion-flow-api/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..Bt2vANjcUF7aPaAE5EcA7A.mB5lmVMla0_vCFNB7lSp-X5NzR-IdH6uEzjAmYZomeU.uott3BNUzXPY8Vz9bTfxaA' \
  -d '{}' \
  localhost:58050 fusion_flow_api.v1.SchedulerApi/CreateJob
```
