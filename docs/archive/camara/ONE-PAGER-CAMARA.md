# Cerulean Ledger — Blockchain Enterprise con Criptografia Post-Cuantica

**Alternativa moderna a Hyperledger Fabric, desarrollada en Chile en Rust, con soporte nativo FIPS 204 (ML-DSA-65).**

---

## El problema

Las plataformas blockchain empresariales actuales (Fabric, Corda, Ethereum privado) dependen de criptografía clásica vulnerable a computación cuántica. Ninguna ofrece firmas post-cuánticas en producción. Además, su complejidad operativa (Go, Java, Docker pesado, configuración extensa) frena la adopción en mercados como Chile.

## La solución

Cerulean Ledger es un nodo blockchain permisionado que implementa el ciclo completo Fabric (endorse → order → commit) con:

- **Firmas post-cuánticas ML-DSA-65 (FIPS 204)** integradas end-to-end — bloques, endorsements, transacciones, gossip
- **Despliegue en minutos** — Docker Compose, 4 nodos, TLS mutuo, sin dependencias Java/Go
- **~95% paridad funcional** con Hyperledger Fabric 2.5

## Capacidades principales

| Capacidad | Detalle |
|---|---|
| Endorsement policies | AnyOf, AllOf, NOutOf — misma semántica que Fabric |
| Ordering service | Solo + Raft persistente con crash recovery |
| Channels | Aislamiento multi-ledger con governance de configuración |
| Private data | Colecciones con ACL por organización y TTL de purga |
| Chaincode (Wasm) | Lifecycle completo: install → approve → commit → simulate |
| Identidad (DID) | Credenciales verificables, X.509 MSP, mTLS |
| Gossip P2P | Alive, pull-sync, anchor peers, TLS |
| Persistencia | RocksDB con column families e índices secundarios |
| SDKs | TypeScript/JavaScript (Python en roadmap) |
| CLI operador | `bcctl` con 14 comandos |
| Monitoreo | Prometheus + Grafana incluidos |

## Ventaja competitiva: Post-Quantum

| Plataforma | PQC en produccion (2026) |
|---|---|
| Hyperledger Fabric | No — solo investigación, requiere reescribir BCCSP |
| R3 Corda | No |
| Ethereum privado | No |
| **Cerulean Ledger** | **Si — ML-DSA-65 (FIPS 204) across full stack** |

Cada nodo selecciona su algoritmo (`SIGNING_ALGORITHM=ml-dsa-65`). Redes mixtas clásicas/PQC operan simultáneamente. Migración gradual, sin flag day.

## Rendimiento medido

| Métrica | Resultado |
|---|---|
| Ordering throughput | ~18,700 TX/s (Solo, batch 100) |
| RocksDB write | ~103,000 bloques/s (batch 100) |
| Endorsement validation (10 orgs) | ~27 µs |
| Event fan-out (50 suscriptores) | ~612 ns |
| Footprint por nodo | ~50 MB RAM |
| Startup | ~2 segundos |

## Caso de uso: Hoktus (gestión de personal)

Credenciales verificables para validación de documentos laborales, antecedentes con private data collections entre empresas y entidades, contratos con firmas PQC de largo plazo, y trazabilidad inmutable del proceso de contratación.

## Estado del proyecto

- **2,040+ tests** unitarios e integración, 71 E2E
- **CI completamente verde**
- **Auditoría de seguridad** con 10/10 hallazgos remediados
- **Docker deployment** funcional con 6 nodos + monitoring
- **Documentación** completa: API (68 endpoints), deployment, quick-start, storage, benchmarks

## Alineación regulatoria

- **NIST FIPS 204** — ML-DSA-65 implementado según estándar
- **CNSS Policy 15** — Cumple requerimiento NSA de algoritmos quantum-safe para 2030
- **CMF / SII (Chile)** — Audit trail inmutable, trazabilidad por organización
- **eIDAS 2.0 (EU)** — Firmas post-cuánticas alineadas con dirección regulatoria europea

## Contacto

Repositorio: `github.com/clementeaf/rust-bc`

---

*Cerulean Ledger — desarrollado en Rust. Sin dependencias Java, Go o Node.js en el core. Open source (MIT).*
