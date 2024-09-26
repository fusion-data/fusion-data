# fusion-scheduler

## grpcurl

```sh
grpcurl -plaintext -import-path ./crates/fusions/fusion-scheduler-api/proto \
  -proto fusion_scheduler_api/v1/scheduler.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..Bt2vANjcUF7aPaAE5EcA7A.mB5lmVMla0_vCFNB7lSp-X5NzR-IdH6uEzjAmYZomeU.uott3BNUzXPY8Vz9bTfxaA' \
  -d '{}' \
  localhost:58010 fusion_scheduler_api.v1.Scheduler/CreateJob
```
