# fruitbox-iam

## 运行

```sh
cargo run -p fruitbox-iam --bin fruitbox-iam
```

## grpc

Enter `backend` directory:

```sh
grpcurl -plaintext localhost:8889 describe


grpcurl -plaintext -import-path ./crates/fusions/fusion-iam/proto \
  -proto fusion_iam/v1/auth.proto \
  -d '{"email":"admin@fusiondata.com", "password":"2024.Fusiondata"}' \
  localhost:8889 fusion_iam.v1.Auth/Signin


grpcurl -plaintext -proto fusion_iam/v1/user.proto \
  -import-path ./crates/fusions/fusion-iam/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..iru12eF7KSCntUw3-d3fQg.WdhmXWkrIJr9eQ03Mrk4ByPgYXStVr35esIrV14RIO0.H-HkIh3rQQ17C_6efRye0A' \
  -d '{"id":1}' \
  localhost:8889 fusion_iam.v1.User/Find


grpcurl -plaintext -proto fusion_iam/v1/role.proto \
  -import-path ./crates/fusions/fusion-iam/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..iru12eF7KSCntUw3-d3fQg.WdhmXWkrIJr9eQ03Mrk4ByPgYXStVr35esIrV14RIO0.H-HkIh3rQQ17C_6efRye0A' \
  -d '{
       "field_mask":{ "paths": ["role", "permissions"]},
       "create_role": {
         "name":"test2",
         "description":"测试角色2",
         "status":"ROLE_STATUS_DISABLED"
       },
       "permission_ids":[1,2]
     }'
  localhost:8889 fusion_iam.v1.Role/Create


grpcurl -plaintext -proto fusion_iam/v1/role.proto \
  -import-path ./crates/fusions/fusion-iam/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..iru12eF7KSCntUw3-d3fQg.WdhmXWkrIJr9eQ03Mrk4ByPgYXStVr35esIrV14RIO0.H-HkIh3rQQ17C_6efRye0A' \
  -d '{
       "field_mask":{ "paths": ["permissions"]},
       "id": 1
     }' \
  localhost:8889 fusion_iam.v1.Role/Get


grpcurl -plaintext -proto fusion_iam/v1/access_control.proto \
  -import-path ./crates/fusions/fusion-iam/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..iru12eF7KSCntUw3-d3fQg.WdhmXWkrIJr9eQ03Mrk4ByPgYXStVr35esIrV14RIO0.H-HkIh3rQQ17C_6efRye0A' \
  -d '{
       "policy": "{\"version\":\"v1.0\",\"statement\":[{\"effect\":\"Allow\",\"action\":[\"GET\"],\"resource\":[\"*\"]}]}"
     }' \
  localhost:8889 fusion_iam.v1.AccessControl/CreatePolicyStatement


## 需要把 id 替换成正确的值（可以使用上一个 CreatePolicyStatement 的返回值）
grpcurl -plaintext -proto fusion_iam/v1/access_control.proto \
  -import-path ./crates/fusions/fusion-iam/proto \
  -import-path ./crates/ultimates/ultimate-api/proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0..iru12eF7KSCntUw3-d3fQg.WdhmXWkrIJr9eQ03Mrk4ByPgYXStVr35esIrV14RIO0.H-HkIh3rQQ17C_6efRye0A' \
  -d '{
       "id": "<policy statement uuid>"
     }' \
  localhost:8889 fusion_iam.v1.AccessControl/GetPolicyStatement
```
