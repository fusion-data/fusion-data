# fusion-scheduler

## grpcurl

```sh
grpcurl -plaintext localhost:58010 describe


grpcurl -plaintext -proto fusion_scheduler_api/v1/scheduler_api.proto \
  -import-path ./crates/fusions/fusion-scheduler-api/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..Bt2vANjcUF7aPaAE5EcA7A.mB5lmVMla0_vCFNB7lSp-X5NzR-IdH6uEzjAmYZomeU.uott3BNUzXPY8Vz9bTfxaA' \
  -d '{}' \
  localhost:58010 fusion_scheduler_api.v1.SchedulerApi/CreateJob
```
