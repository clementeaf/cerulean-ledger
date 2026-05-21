# Referencia de API

URL base: `https://localhost:8080/api/v1`

Todas las respuestas usan el envelope gateway: `{ status, status_code, message, data, error, timestamp, trace_id }`.

Headers: `Content-Type: application/json`. Endpoints con scope de canal aceptan `X-Channel-Id` (default: `"default"`). Endpoints con scope de organización requieren `X-Org-Id`.

---

## Salud y utilidades

### GET /health

```bash
curl -sk https://localhost:8080/api/v1/health | jq .data
```

```json
{
  "status": "healthy",
  "uptime_seconds": 120,
  "blockchain": { "height": 5, "last_block_hash": "abc...", "validators_count": 0 },
  "checks": { "storage": "ok", "peers": "ok (3 connected)", "ordering": "ok" }
}
```

### GET /version

```json
{ "api_version": "1.0.0", "rust_bc_version": "0.1.0", "blockchain_height": 5 }
```

### GET /openapi.json

Especificación OpenAPI 3.0.

### GET /metrics

Formato texto Prometheus (fuera del scope `/api/v1`).

---

## Gateway (Endorse -> Order -> Commit)

### POST /gateway/submit

Enviar una transacción a través del pipeline completo.

```bash
curl -sk https://localhost:8080/api/v1/gateway/submit -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "chaincode_id": "mycc",
    "channel_id": "mychannel",
    "transaction": {
      "id": "tx-001",
      "input_did": "did:bc:alice",
      "output_recipient": "did:bc:bob",
      "amount": 100
    }
  }'
```

```json
{ "tx_id": "tx-001", "block_height": 3, "valid": true }
```

---

## Bloques

### GET /blocks

Retorna la blockchain completa como array.

### GET /blocks/index/{index}

Obtener bloque por altura.

### GET /blocks/{hash}

Obtener bloque por hash.

### POST /blocks

Minar un bloque. Body: `{ data, miner_address }`.

### GET /store/blocks?page=1&limit=10

Lista paginada de bloques desde la capa de almacenamiento.

### GET /store/blocks/latest

Retorna la altura del último bloque.

### GET /store/blocks/{height}

Obtener bloque por altura desde almacenamiento.

### GET /store/blocks/{height}/transactions

Listar transacciones de un bloque (consulta por índice secundario).

---

## Transacciones

### POST /transactions

Crear, validar y encolar una transacción.

```bash
curl -sk https://localhost:8080/api/v1/transactions -X POST \
  -d '{ "from": "addr1", "to": "addr2", "amount": 50, "fee": 1 }'
```

### GET /mempool

```json
{ "count": 2, "transactions": [...] }
```

### POST /store/transactions

Persistir transacción en el store. Retorna 201.

### GET /store/transactions/{tx_id}

Leer una transacción del store.

---

## Canales

### POST /channels

```bash
curl -sk https://localhost:8080/api/v1/channels -X POST \
  -d '{ "channel_id": "mychannel" }'
```

Retorna 201: `{ "channel_id": "mychannel" }`.

### GET /channels

Listar todos los canales.

### POST /channels/{channel_id}/config

Actualizar configuración del canal (requiere firmas de endorsement).

### GET /channels/{channel_id}/config

Obtener configuración actual del canal.

### GET /channels/{channel_id}/config/history

Historial de versiones de configuración.

---

## Organizaciones

### POST /store/organizations

```bash
curl -sk https://localhost:8080/api/v1/store/organizations -X POST \
  -d '{ "org_id": "org1", "name": "Organización 1", "msp_id": "Org1MSP" }'
```

### GET /store/organizations

Listar todas las organizaciones.

### GET /store/organizations/{org_id}

Obtener organización por ID.

---

## Políticas de endorsement

### POST /store/policies

```bash
curl -sk https://localhost:8080/api/v1/store/policies -X POST \
  -d '{ "resource_id": "mychannel/mycc", "policy": { "NOutOf": { "n": 2, "orgs": ["org1", "org2"] } } }'
```

### GET /store/policies/{resource_id}

Obtener política para un recurso.

---

## Ciclo de vida de chaincode

### POST /chaincode/install?chaincode_id={id}&version={v}

Subir binario Wasm. Content-Type: `application/octet-stream`.

```bash
curl -sk "https://localhost:8080/api/v1/chaincode/install?chaincode_id=basic&version=1.0" \
  -X POST --data-binary @chaincode.wasm -H 'Content-Type: application/octet-stream'
```

```json
{ "chaincode_id": "basic", "version": "1.0", "size_bytes": 529 }
```

### POST /chaincode/{id}/approve?version={v}

Aprobar chaincode para tu organización. Requiere header `X-Org-Id`.

### POST /chaincode/{id}/commit?version={v}

Commit de chaincode (requiere que la política esté satisfecha).

### POST /chaincode/{id}/simulate?version={v}

Simular invocación de chaincode (solo lectura).

```json
{ "result": "...", "rwset": { "reads": [...], "writes": [...] } }
```

---

## Colecciones de datos privados

### POST /private-data/collections

```bash
curl -sk https://localhost:8080/api/v1/private-data/collections -X POST \
  -d '{ "name": "datos-secretos", "member_org_ids": ["org1", "org2"] }'
```

### PUT /private-data/{collection}/{key}

Requiere header `X-Org-Id` (debe ser miembro de la colección).

```bash
curl -sk https://localhost:8080/api/v1/private-data/datos-secretos/clave1 -X PUT \
  -H 'X-Org-Id: org1' -d '{ "value": "valor-secreto" }'
```

```json
{ "collection": "datos-secretos", "key": "clave1", "hash": "abc123..." }
```

### GET /private-data/{collection}/{key}

Requiere header `X-Org-Id`. Retorna 403 para no-miembros.

---

## Servicio de discovery

### POST /discovery/register

Registrar un peer en el servicio de discovery.

### GET /discovery/endorsers?chaincode={id}&channel={ch}

Obtener plan de endorsement (lista de peers que pueden endorsar).

### GET /discovery/peers?channel={id}

Listar peers en un canal.

---

## ACL (Control de acceso)

### POST /acls

```bash
curl -sk https://localhost:8080/api/v1/acls -X POST \
  -d '{ "resource": "peer/ChaincodeToChaincode", "policy_ref": "mycc_policy" }'
```

### GET /acls

Listar todas las entradas ACL.

### GET /acls/{resource}

Obtener ACL para un recurso específico.

---

## MSP (Proveedor de servicios de membresía)

### POST /msp/{msp_id}/revoke

Agregar serial de certificado a la lista de revocación (CRL).

```bash
curl -sk https://localhost:8080/api/v1/msp/Org1MSP/revoke -X POST \
  -H 'X-MSP-Role: admin' -d '{ "serial": "ABC123" }'
```

### GET /msp/{msp_id}

Obtener info del MSP (tamaño CRL).

---

## Identidad y credenciales

### POST /identity/create

Crear DID + par de claves Ed25519. Body: `{ "name": "Alice" }`.

### GET /identity/{did}

Obtener documento DID.

### POST /store/identities

Persistir registro de identidad.

### GET /store/identities/{did}

Leer registro de identidad.

### POST /credentials/issue

Emitir una credencial verificable.

### POST /credentials/{id}/verify

Verificar firma y expiración de credencial.

### POST /store/credentials

Persistir credencial en el store.

### GET /store/credentials/{cred_id}

Leer credencial.

### GET /store/credentials/by-subject/{subject_did}

Listar credenciales por DID del sujeto.

---

## Eventos (WebSocket)

### GET /events/blocks?from_height=N

Long-poll o upgrade WebSocket. Retorna eventos de bloques desde `from_height`.

### GET /events/blocks/filtered

Stream WebSocket de resúmenes de bloques filtrados (IDs de TX + códigos de validación, sin payloads).

### GET /events/blocks/private

Stream WebSocket con datos privados para orgs autorizadas. Requiere header `X-Org-Id`.

---

## Snapshots

### POST /snapshots/{channel_id}

Crear snapshot de estado.

### GET /snapshots/{channel_id}

Listar snapshots.

### GET /snapshots/{channel_id}/{snapshot_id}

Descargar binario de snapshot.

---

## Wallets (legacy)

### POST /wallets/create

Crear nueva wallet. Retorna `{ address, balance, public_key }`.

### GET /wallets/{address}

Obtener balance e info de wallet.

---

## Minado (legacy)

### POST /mine

```bash
curl -sk https://localhost:8080/api/v1/mine -X POST \
  -d '{ "miner_address": "abc123" }'
```

```json
{ "hash": "...", "reward": 50, "transactions_count": 3 }
```

---

## Info de cadena (legacy)

### GET /chain/verify

```json
{ "valid": true, "block_count": 10 }
```

### GET /chain/info

```json
{ "block_count": 10, "difficulty": 1, "latest_block_hash": "...", "is_valid": true }
```
