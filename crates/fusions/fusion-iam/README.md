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


grpcurl -plaintext -import-path ./crates/fusion-iam/proto \
  -proto fusion_iam/v1/user.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0.._QTBCD_ZH49y3Rh5teJQNQ.Tst-zxGVa4VjbRHqIp2VWgHvozCFlmBNANfqO2ljdrg.9CtL86ZhdUrFNleXaSaMhQ' \
  -d '{"id":1}' \
  localhost:8889 fusion_iam.v1.User/Find


grpcurl -plaintext -import-path ./crates/fusion-iam/proto \
  -proto fusion_iam/v1/role.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0.._QTBCD_ZH49y3Rh5teJQNQ.Tst-zxGVa4VjbRHqIp2VWgHvozCFlmBNANfqO2ljdrg.9CtL86ZhdUrFNleXaSaMhQ' \
  -d '{
       "field_mask":{ "paths": ["role", "permissions"]},
       "create_role": {
         "name":"test2",
         "description":"测试角色2",
         "status":"ROLE_STATUS_DISABLED"
       },
       "permission_ids":[1,2]
     }' \
  localhost:8889 fusion_iam.v1.Role/Create


grpcurl -plaintext -import-path ./crates/fusion-iam/proto \
  -proto fusion_iam/v1/role.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0.._QTBCD_ZH49y3Rh5teJQNQ.Tst-zxGVa4VjbRHqIp2VWgHvozCFlmBNANfqO2ljdrg.9CtL86ZhdUrFNleXaSaMhQ' \
  -d '{
       "field_mask":{ "paths": ["permissions"]},
       "id": 1
     }' \
  localhost:8889 fusion_iam.v1.Role/Get


grpcurl -plaintext -import-path ./crates/fusion-iam/proto \
  -proto fusion_iam/v1/access_control.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0.._QTBCD_ZH49y3Rh5teJQNQ.Tst-zxGVa4VjbRHqIp2VWgHvozCFlmBNANfqO2ljdrg.9CtL86ZhdUrFNleXaSaMhQ' \
  -d '{
       "policy": "{\"version\":\"v1.0\",\"statement\":[{\"effect\":\"Allow\",\"action\":[\"GET\"],\"resource\":[\"*\"]}]}"
     }' \
  localhost:8889 fusion_iam.v1.AccessControl/CreatePolicyStatement


grpcurl -plaintext -import-path ./crates/fusion-iam/proto \
  -proto fusion_iam/v1/access_control.proto \
  -H 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiZGlyIn0.._QTBCD_ZH49y3Rh5teJQNQ.Tst-zxGVa4VjbRHqIp2VWgHvozCFlmBNANfqO2ljdrg.9CtL86ZhdUrFNleXaSaMhQ' \
  -d '{
       "id": "0191efc2-b13b-7932-8468-8470648c17db"
     }' \
  localhost:8889 fusion_iam.v1.AccessControl/GetPolicyStatement
```
