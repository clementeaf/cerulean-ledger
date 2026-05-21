# Guía de Despliegue

Despliegue en producción de una red blockchain rust-bc.

## Arquitectura

Una red mínima de producción consiste en:

| Componente | Cantidad | Rol |
|-----------|----------|-----|
| Nodos peer | 2+ | Ejecutar chaincode, endorsar transacciones, almacenar ledger |
| Orderer | 1+ | Ordenar transacciones en bloques (Solo o Raft) |
| Prometheus | 1 | Recolección de métricas |
| Grafana | 1 | Dashboards |

## Despliegue con Docker Compose

### 1. Generar certificados TLS

```bash
cd deploy && bash generate-tls.sh
```

Para producción, reemplazar con certificados de una CA de confianza o la PKI de la organización.

### 2. Configurar variables de entorno

Variables principales por nodo:

| Variable | Default | Descripción |
|----------|---------|-------------|
| `API_PORT` | 8080 | Puerto de la API HTTP |
| `P2P_PORT` | 8081 | Puerto del protocolo P2P |
| `BIND_ADDR` | `127.0.0.1` | Dirección de escucha (`0.0.0.0` en contenedores) |
| `STORAGE_BACKEND` | *(memory)* | Usar `rocksdb` para almacenamiento persistente |
| `STORAGE_PATH` | `./data/blocks` | Directorio de datos RocksDB |
| `DIFFICULTY` | 1 | Dificultad de minado |
| `NETWORK_ID` | `mainnet` | Identificador de red |
| `ACL_MODE` | *(strict)* | `permissive` desactiva control de acceso |
| `JWT_SECRET` | `change-me-in-production` | Secreto para firma JWT |

### Configuración TLS

| Variable | Descripción |
|----------|-------------|
| `TLS_CERT_PATH` | Certificado TLS del nodo (activa HTTPS + TLS P2P) |
| `TLS_KEY_PATH` | Clave privada TLS del nodo |
| `TLS_CA_CERT_PATH` | Certificado CA para verificación de peers |
| `TLS_PINNED_CERTS` | Fingerprints SHA-256 separados por coma (opcional) |

### Configuración P2P

| Variable | Default | Descripción |
|----------|---------|-------------|
| `P2P_EXTERNAL_ADDRESS` | — | Dirección de anuncio (ej. `node1:8081`) |
| `BOOTSTRAP_NODES` | — | Lista `host:port` separada por comas |
| `SEED_NODES` | — | Lista de peers que siempre se intentan |
| `P2P_RESPONSE_BUFFER_BYTES` | 262144 | Buffer de respuesta (256 KB) |
| `P2P_HANDLER_BUFFER_BYTES` | 65536 | Buffer del handler (64 KB) |
| `P2P_SYNC_BUFFER_BYTES` | 4194304 | Buffer de sync de estado (4 MB) |

### Configuración Raft (ordering)

| Variable | Default | Descripción |
|----------|---------|-------------|
| `ORDERING_BACKEND` | `solo` | `solo` o `raft` |
| `RAFT_NODE_ID` | 1 | ID Raft de este nodo |
| `RAFT_PEERS` | — | Entradas `id:host:port` (ej. `1:orderer1:8087,2:orderer2:8087`) |

Con `STORAGE_BACKEND=rocksdb`, el log Raft se persiste en `{STORAGE_PATH}/raft/` y sobrevive reinicios.

### 3. Levantar la red

```bash
docker compose up -d
```

### 4. Verificar

```bash
./scripts/bcctl.sh status       # Salud de todos los nodos
./scripts/bcctl.sh consistency  # Comparar estado de cadena entre peers
```

## Checklist de producción

### Seguridad

- [ ] Reemplazar certificados TLS auto-firmados con certificados de CA
- [ ] Configurar `JWT_SECRET` con un valor aleatorio fuerte
- [ ] Mantener `ACL_MODE` en strict (default) — nunca usar permissive en producción
- [ ] Revisar `PEER_ALLOWLIST` para restringir conexiones P2P entrantes
- [ ] Activar `TLS_PINNED_CERTS` para certificate pinning
- [ ] Rotar certificados TLS via SIGHUP o `TLS_RELOAD_INTERVAL`

### Almacenamiento

- [ ] Configurar `STORAGE_BACKEND=rocksdb` en todos los nodos
- [ ] Montar `/app/data` como volumen Docker nombrado o disco persistente
- [ ] Si RocksDB falla al abrir, el nodo sale con error (sin fallback silencioso)

### Monitoreo

- [ ] Prometheus raspando `/metrics` en cada nodo
- [ ] Dashboards Grafana para altura de bloques, cantidad de peers, throughput de transacciones
- [ ] Alertar cuando `/api/v1/health` retorne estado `"degraded"`

### Red

- [ ] Usar `P2P_EXTERNAL_ADDRESS` cuando los nodos están detrás de NAT/load balancer
- [ ] Configurar `BOOTSTRAP_NODES` en cada peer para discovery inicial
- [ ] Ajustar `P2P_SYNC_BUFFER_BYTES` para redes con bloques grandes

### Respaldo

- [ ] Hacer snapshot del directorio de datos RocksDB (`/app/data`) periódicamente
- [ ] Usar `POST /api/v1/snapshots/{channel_id}` para snapshots a nivel de aplicación
- [ ] Probar restauración desde snapshot antes de confiar en él

## Referencia de puertos

| Servicio | Puerto default | Protocolo |
|---------|---------------|----------|
| API (HTTPS) | 8080 | HTTPS (TLS) |
| P2P | 8081 | TCP (TLS) |
| Prometheus | 9090 | HTTP |
| Grafana | 3000 | HTTP |

## Shutdown graceful

El nodo maneja SIGTERM y Ctrl-C:

1. Deja de aceptar nuevas conexiones HTTP
2. Drena requests en vuelo (timeout 10s)
3. Aborta tareas de fondo (gossip, discovery, sync, purge)
4. Hace flush del WAL de RocksDB
5. Sale con código 0

```bash
docker compose stop   # envía SIGTERM, espera 10s, luego SIGKILL
```
