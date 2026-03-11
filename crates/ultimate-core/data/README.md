# Data

## Signing a JWT by ECDSA

### Generate a new private key

#### for ES256

```shell
openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out private.pem
```

#### for ES384

```shell
openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-384 -out private.pem
```

#### for ES512

```shell
openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-521 -out private.pem
```

#### for ES256K

```shell
openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:secp256k1 -out private.pem
```

#### Generate a public key from the private key

```shell
openssl pkey -in private.pem -pubout -out public.pem
```
