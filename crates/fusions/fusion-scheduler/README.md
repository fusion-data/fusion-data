# fusion-scheduler

## 运行

```sh
# Master & Scheduler 节点
cargo run -p fusion-scheduler

# Worker 节点
cargo run -p fusion-scheduler-worker
```

## grpcurl

```sh
grpcurl -plaintext localhost:58050 describe


grpcurl -plaintext -proto fusion_scheduler_api/v1/scheduler_api.proto \
  -import-path ./crates/fusions/fusion-scheduler-api/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..Bt2vANjcUF7aPaAE5EcA7A.mB5lmVMla0_vCFNB7lSp-X5NzR-IdH6uEzjAmYZomeU.uott3BNUzXPY8Vz9bTfxaA' \
  -d '{}' \
  localhost:58050 fusion_scheduler_api.v1.SchedulerApi/CreateJob
```
