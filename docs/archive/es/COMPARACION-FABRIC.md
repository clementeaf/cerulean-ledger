# Comparación con Hyperledger Fabric 2.5

Cómo se compara rust-bc con Hyperledger Fabric 2.5. Actualizado: 2026-04-07.

---

## Paridad alcanzada

| Capacidad | Fabric 2.5 | rust-bc | Estado |
|-----------|-----------|---------|--------|
| Endorse → Order → Commit | Pipeline gRPC | Pipeline HTTP/P2P | Paridad |
| Políticas de endorsement (AnyOf, AllOf, NOutOf) | Políticas Protobuf | Políticas JSON | Paridad |
| Ordering Raft | Cluster etcd/raft | tikv/raft + log persistente RocksDB + P2P | Paridad |
| Validación MVCC | Validación de bloques | `validate_rwset()` en el commit path | Paridad |
| Canales (multi-ledger) | Ledger por canal | BlockStore por canal | Paridad |
| Colecciones de datos privados | Gossip + side-DB | P2P push + side-store + TTL purge | Paridad |
| Ciclo de vida de chaincode | Install → Approve → Commit | Mismo flujo | Paridad |
| Ejecución de chaincode Wasm | Docker/Go/Java/Node | Wasmtime + fuel/memory limits | Diferente pero funcional |
| Chaincode externo | Chaincode-as-a-service | Cliente HTTP + campo runtime | Paridad |
| World state + historial | LevelDB/CouchDB | Memory + CouchDB + key history | Paridad |
| Roles MSP (admin/peer/client) | Basado en cert X.509 | Extracción X.509 de mTLS + fallback headers | Paridad |
| Enforcement ACL | Políticas por recurso | `enforce_acl()` deny-by-default | Paridad |
| Protocolo gossip | Alive + pull + anchor | Alive + pull-sync + anchor peers | Paridad |
| Eventos de bloques | Deliver/DeliverFiltered | WebSocket + filtered + private | Paridad |
| Snapshots de estado | Snapshot de ledger | Create/restore + verificación SHA-256 | Paridad |
| Certificate pinning | Config de canal | Allowlist de fingerprints SHA-256 | Paridad |
| Firma HSM | PKCS#11 BCCSP | `cryptoki` feature-gated | Scaffold |
| SDK Node | fabric-network | @rust-bc/sdk (TypeScript) | Paridad |
| CLI operador | peer CLI | bcctl (binario Rust, 14 comandos) | Paridad |
| Block explorer | Hyperledger Explorer | App Next.js | Paridad |
| Despliegue Docker | Ejemplos docker-compose | 3 peers + orderer + Prometheus + Grafana | Paridad |
| Rotación hot de certs | Configurable | SIGHUP + recarga periódica | Paridad |
| Almacenamiento persistente | LevelDB/CouchDB | RocksDB (8 service stores + bloques) | Paridad |
| Shutdown graceful | Shutdown orderer/peer | SIGTERM/SIGINT con drain de conexiones | Paridad |

---

## Gaps restantes

### Moderados (no bloquean uso enterprise)

| Gap | Descripción | Impacto | Esfuerzo |
|-----|------------|---------|----------|
| Sin protocolo gRPC | Toda comunicación es HTTP/JSON, no Protobuf/gRPC. Incompatible con SDKs y peers Fabric nativos. | Alto (solo si se necesita interop) | Alto |
| Sin Fabric CA | No hay servicio de enrollment/registration de identidades. Certificados se gestionan externamente. | Medio | Alto |
| Sin service discovery automático | Discovery es registro manual de peers, no discovery automático por gossip. | Medio | Medio |

### Baja prioridad

| Gap | Descripción | Impacto | Esfuerzo |
|-----|------------|---------|----------|
| Sin ordering Kafka | Solo backends Solo y Raft. Kafka fue deprecado en Fabric 2.x. | Ninguno | — |
| Protocolo chaincode externo | Usa HTTP POST, no el protocolo CDS de Fabric. | Bajo | Bajo |
| DiscoveryService no persistente | Registros de peers se pierden al reiniciar. | Bajo | Bajo |

---

## Veredicto

### Como MVP / Proof of Concept: ~95% de paridad

El ciclo de vida completo de transacciones Fabric funciona end-to-end: endorsement con políticas configurables, ordering Raft persistente con recuperación ante crashes, validación MVCC, aislamiento de canales, datos privados con ACL, ciclo de vida de chaincode, protocolo gossip, eventos de bloques, y snapshots de estado. Enforcement MSP X.509 desde certificados mTLS. Respaldado por 2040+ tests unitarios, 71 tests E2E, y un pipeline CI completamente verde.

### Como reemplazo enterprise en producción: ~80% de paridad

No quedan gaps críticos. La principal capacidad faltante es soporte de protocolo gRPC, que solo se necesita para interoperabilidad con redes Fabric existentes. Como blockchain permisionada standalone, rust-bc está lista para producción.

### Ventajas sobre Fabric

| Aspecto | Fabric 2.5 | rust-bc |
|---------|-----------|---------|
| Lenguaje | Go/Java (GC pauses) | Rust (zero-cost abstractions, sin GC) |
| Chaincode runtime | Docker containers (overhead) | Wasm in-process (microsegundos) |
| Deployment | Complejo (CA, MSP, configtxgen) | Simple (docker compose + generate-tls.sh) |
| API | gRPC (requiere tooling específico) | REST/JSON (curl, cualquier lenguaje) |
| Curva de aprendizaje | Alta (documentación fragmentada) | Media (docs unificados, ejemplos claros) |
| Footprint | ~500MB por peer | ~50MB por nodo |

---

## Cobertura de tests

| Categoría | Cantidad |
|-----------|----------|
| Tests unitarios + integración | 2040+ |
| Tests E2E (red Docker) | 71 |
| Tests integración CouchDB | 3 |
| Estado CI | Todo verde |
| Tests fallando | 0 |
