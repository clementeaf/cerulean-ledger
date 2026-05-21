# Inicio Rápido

Red blockchain de 4 nodos funcionando en menos de 5 minutos.

## Requisitos

- Docker y Docker Compose
- curl (para pruebas)
- Node.js 18+ (opcional, para el SDK JS)
- Python 3.10+ (opcional, para el SDK Python)

## 1. Clonar y generar certificados TLS

```bash
git clone https://github.com/clementeaf/rust-bc.git
cd rust-bc
cd deploy && bash generate-tls.sh && cd ..
```

## 2. Levantar la red

```bash
docker compose build
docker compose up -d node1 node2 node3 orderer1
```

Esperar ~20 segundos para que los nodos arranquen y se descubran entre sí.

## 3. Verificar salud

```bash
curl -sk https://localhost:8080/api/v1/health | jq .
```

Respuesta esperada:

```json
{
  "status": "Success",
  "status_code": 200,
  "data": {
    "status": "healthy",
    "uptime_seconds": 25,
    "blockchain": { "height": 1, "last_block_hash": "...", "validators_count": 0 },
    "checks": { "storage": "ok", "peers": "ok (3 connected)", "ordering": "ok" }
  }
}
```

## 4. Crear una wallet y minar un bloque

```bash
# Crear wallet
WALLET=$(curl -sk https://localhost:8080/api/v1/wallets/create -X POST | jq -r '.data.address')
echo "Wallet: $WALLET"

# Minar un bloque
curl -sk https://localhost:8080/api/v1/mine -X POST \
  -H 'Content-Type: application/json' \
  -d "{\"miner_address\": \"$WALLET\"}" | jq .
```

## 5. Enviar una transacción via gateway

```bash
curl -sk https://localhost:8080/api/v1/gateway/submit -X POST \
  -H 'Content-Type: application/json' \
  -d '{
    "chaincode_id": "mycc",
    "channel_id": "",
    "transaction": {
      "id": "tx-001",
      "input_did": "did:bc:alice",
      "output_recipient": "did:bc:bob",
      "amount": 100
    }
  }' | jq .
```

Respuesta:

```json
{
  "data": { "tx_id": "tx-001", "block_height": 2, "valid": true }
}
```

## 6. Verificar propagación multi-nodo

```bash
for port in 8080 8082 8084; do
  echo -n "Puerto $port: "
  curl -sk "https://localhost:$port/api/v1/chain/info" | jq -r '.data.block_count'
done
```

Todos los nodos deben reportar la misma altura de blockchain.

## 7. Usar el SDK JS/TS (opcional)

```bash
cd sdk-js && npm install && npm run build && cd ..
```

```typescript
import { BlockchainClient } from '@rust-bc/sdk';

const client = new BlockchainClient({
  baseUrl: 'https://localhost:8080/api/v1',
});

const health = await client.health();
console.log(health.status); // "healthy"

const result = await client.submitTransaction('mycc', '', {
  id: 'tx-002',
  inputDid: 'did:bc:alice',
  outputRecipient: 'did:bc:bob',
  amount: 50,
});
console.log(result.block_height); // 3
```

## 8. Monitoreo (opcional)

```bash
docker compose up -d prometheus grafana
```

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)

## 9. CLI operador

```bash
./scripts/bcctl.sh status       # Salud de todos los nodos
./scripts/bcctl.sh consistency  # Comparar tips de cadena entre peers
./scripts/bcctl.sh mine         # Crear wallet + minar bloque
./scripts/bcctl.sh orgs         # Listar organizaciones registradas
```

## Siguientes pasos

- [Referencia de API](REFERENCIA-API.md) — los 68 endpoints con ejemplos
- [Guía de despliegue](DESPLIEGUE.md) — configuración de producción
- [Comparación con Fabric](COMPARACION-FABRIC.md) — análisis de paridad
