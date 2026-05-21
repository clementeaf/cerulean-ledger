# Onboarding de organizaciones

Cómo agregar una nueva organización a una red rust-bc existente.

## Requisitos previos

- Red rust-bc corriendo (al menos `node1`, `node2`, `node3`, `orderer1`)
- Docker y Docker Compose instalados
- Acceso a la CA (archivo `deploy/tls/ca-key.pem`)
- Conectividad al API de un nodo existente

## Método rápido (1 comando)

```bash
./scripts/onboard-org.sh <org_id> <node_name> <api_port> <p2p_port> [network_url]
```

### Ejemplo: agregar org3 con node4

```bash
./scripts/onboard-org.sh org3 node4 8088 8089
```

El script ejecuta automáticamente:

1. Genera certificado TLS para el nuevo nodo (firmado por la CA existente)
2. Registra la organización en la red via API
3. Genera un archivo `docker-compose.node4.yml` con la configuración del nodo
4. Levanta el contenedor Docker
5. Registra el peer en el servicio de discovery
6. Verifica que el nodo esté healthy y sincronizado

### Ejemplo: agregar org4 con node5 apuntando a otra red

```bash
./scripts/onboard-org.sh org4 node5 8090 8091 https://node1.miempresa.cl:8080
```

## Método manual (paso a paso)

### Paso 1: Generar certificado TLS

```bash
cd deploy/tls

# Generar key + CSR
openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 \
  -nodes \
  -keyout node4-key.pem \
  -out node4.csr \
  -subj "/CN=node4/O=org3"

# Firmar con la CA
openssl x509 -req \
  -in node4.csr \
  -CA ca-cert.pem \
  -CAkey ca-key.pem \
  -CAcreateserial \
  -days 365 \
  -out node4-cert.pem

chmod 644 node4-key.pem node4-cert.pem
rm node4.csr
```

### Paso 2: Registrar organización

```bash
curl -sk https://localhost:8080/api/v1/store/organizations -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "org_id": "org3",
    "name": "Organización 3",
    "msp_id": "Org3MSP"
  }'
```

### Paso 3: Crear docker-compose override

Crear `docker-compose.node4.yml`:

```yaml
version: '3.8'

services:
  node4:
    build: .
    container_name: rust-bc-node4
    restart: unless-stopped
    networks:
      - bc-net
    ports:
      - "8088:8088"
      - "8089:8089"
    environment:
      BIND_ADDR: "0.0.0.0"
      P2P_EXTERNAL_ADDRESS: "node4:8089"
      API_PORT: "8088"
      P2P_PORT: "8089"
      DIFFICULTY: "1"
      NETWORK_ID: "local-test"
      ORG_ID: "org3"
      NODE_ROLE: "peer"
      STORAGE_BACKEND: "rocksdb"
      STORAGE_PATH: "/app/data/rocksdb"
      TLS_CERT_PATH: "/tls/node4-cert.pem"
      TLS_KEY_PATH: "/tls/node4-key.pem"
      TLS_CA_CERT_PATH: "/tls/ca-cert.pem"
      BOOTSTRAP_NODES: "node1:8081,node2:8081,node3:8081"
    volumes:
      - node4-data:/app/data
      - ./deploy/tls:/tls:ro

volumes:
  node4-data:

networks:
  bc-net:
    external: true
    name: rust-bc_bc-net
```

### Paso 4: Levantar el nodo

```bash
docker compose -f docker-compose.yml -f docker-compose.node4.yml up -d node4
```

### Paso 5: Registrar peer en discovery

```bash
curl -sk https://localhost:8080/api/v1/discovery/register -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "peer_address": "node4:8089",
    "org_id": "org3",
    "role": "Peer",
    "chaincodes": [],
    "channels": ["default"]
  }'
```

### Paso 6: Verificar

```bash
# Health check del nuevo nodo
curl -sk https://localhost:8088/api/v1/health | jq .data.status

# Verificar que se sincronizó
curl -sk https://localhost:8088/api/v1/chain/info | jq .data.block_count

# Verificar peers conectados
./scripts/bcctl.sh status
```

## Unirse a un canal existente

Después del onboarding, el nuevo nodo puede unirse a canales:

```bash
# Escribir una transacción en un canal específico
curl -sk https://localhost:8088/api/v1/gateway/submit -X POST \
  -H 'Content-Type: application/json' \
  -H 'X-Org-Id: org3' \
  -H 'X-Channel-Id: mychannel' \
  -d '{
    "chaincode_id": "mycc",
    "channel_id": "mychannel",
    "transaction": {
      "id": "tx-org3-001",
      "input_did": "did:bc:org3-admin",
      "output_recipient": "did:bc:recipient",
      "amount": 50
    }
  }'
```

## Instalar chaincode

```bash
# Instalar Wasm chaincode en el nuevo nodo
curl -sk "https://localhost:8088/api/v1/chaincode/install?chaincode_id=mycc&version=1.0" \
  -X POST --data-binary @chaincode.wasm \
  -H 'Content-Type: application/octet-stream'

# Aprobar como org3
curl -sk "https://localhost:8088/api/v1/chaincode/mycc/approve?version=1.0" \
  -X POST -H 'X-Org-Id: org3'
```

## Desconectar una organización

```bash
# Detener el nodo
docker compose -f docker-compose.yml -f docker-compose.node4.yml stop node4

# Eliminar completamente (incluyendo datos)
docker compose -f docker-compose.yml -f docker-compose.node4.yml down -v node4

# Limpiar certificados
rm deploy/tls/node4-*.pem
rm docker-compose.node4.yml
```

## Troubleshooting

| Problema | Solución |
|----------|----------|
| "connection refused" al verificar | Esperar más tiempo; el nodo necesita ~15s para arrancar |
| "Permission denied" en TLS | Verificar `chmod 644` en los archivos `.pem` |
| No se sincroniza con la red | Verificar `BOOTSTRAP_NODES` apunta a nodos activos |
| "ACL denied" al enviar transacciones | El nodo existente debe tener `ACL_MODE=permissive` o registrar ACL para org3 |
| El nodo aparece como "degraded" | Normal si aún no tiene peers; revisar `checks.peers` en `/health` |
