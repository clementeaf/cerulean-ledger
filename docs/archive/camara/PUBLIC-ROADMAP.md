# Roadmap Publico — Cerulean Ledger

Hoja de ruta orientada a partners, evaluadores y la comunidad blockchain chilena.

**Ultima actualizacion:** 2026-04-24

---

## Estado actual: Core completo

La plataforma blockchain esta funcional con ~95% de paridad respecto a Hyperledger Fabric 2.5:

- Ciclo completo: endorse → order → commit
- Raft persistente con crash recovery + BFT (HotStuff-inspired)
- Channels, private data, chaincode Wasm + EVM (Solidity)
- Identidad DID + credenciales verificables
- Firmas post-cuanticas ML-DSA-65 (FIPS 204)
- Governance on-chain con votacion stake-weighted
- Bridge cross-chain con escrow y Merkle proofs
- Light client para IoT/movil
- 2,700+ tests, 71 E2E, CI verde
- Docker deployment con 6 nodos + Prometheus + Grafana
- Block Explorer (Cerulean Ledger) + App de voto electronico (Cerulean Voto)

---

## Q2 2026 (abril–junio) — Producto minimo viable

Transicion de plataforma tecnica a producto entregable.

| Entregable | Fecha objetivo | Estado |
|---|---|---|
| Graceful shutdown con flush RocksDB | 2026-04-01 | Completado |
| 9 servicios persistentes en RocksDB | 2026-04-01 | Completado |
| Health check avanzado (RocksDB, peers, ordering) | 2026-04-01 | Completado |
| SDK TypeScript (submit, evaluate, private data, events) | 2026-04-08 | Completado |
| CLI operador `bcctl` (14 comandos) | 2026-04-10 | Completado |
| Documentacion completa (68 endpoints, deployment, quick-start) | 2026-04-12 | Completado |
| Benchmarks publicados (Criterion + comparacion Fabric) | 2026-04-15 | Completado |
| Block Explorer — Cerulean Ledger UI | 2026-04-17 | Completado |
| Governance HTTP API (7 endpoints) | 2026-04-23 | Completado |
| Cerulean Voto — frontend de voto electronico | 2026-04-24 | Completado |
| Documentacion para Camara de Blockchain Chile | 2026-04-24 | Completado |
| Documentacion en espanol (quick-start, API, deployment) | 2026-06-15 | Pendiente |

## Q3 2026 (julio–septiembre) — Ecosistema Chile

Foco en adopcion en el mercado chileno y ecosistema de desarrolladores.

| Entregable | Fecha objetivo | Estado |
|---|---|---|
| SDK Python (paridad con SDK TS, type hints, PyPI) | 2026-07-31 | Pendiente |
| Onboarding multi-org automatizado (< 10 min) | 2026-08-15 | Pendiente |
| Audit trail para reguladores (middleware, export CSV) | 2026-08-31 | Pendiente |
| Primer piloto con empresa/organizacion chilena | 2026-09-30 | Pendiente |

## Q4 2026 (octubre–diciembre) — Enterprise hardening

Preparacion para pilotos en produccion y evaluaciones de seguridad.

| Entregable | Fecha objetivo | Estado |
|---|---|---|
| FIPS 140-3 preparacion (KAT, zeroizacion, doc modulo) | 2026-10-31 | En progreso |
| Property-based testing (proptest + fuzzing) | 2026-11-15 | Pendiente |
| Encryption at rest (RocksDB/LUKS) | 2026-11-30 | Pendiente |
| Vulnerability disclosure process | 2026-12-15 | Pendiente |

## H1 2027 (enero–junio) — Certificaciones

| Entregable | Fecha objetivo | Estado |
|---|---|---|
| SOC 2 Type II — inicio periodo de observacion | 2027-01-15 | Pendiente |
| ISO 27001 — framework de politicas | 2027-03-31 | Pendiente |
| ML-KEM (FIPS 203) — key exchange post-cuantico para TLS | 2027-04-30 | Pendiente |
| Hybrid signatures (Ed25519 + ML-DSA-65) | 2027-05-31 | Pendiente |

## H2 2027 (julio–diciembre) — Expansion

| Entregable | Fecha objetivo | Estado |
|---|---|---|
| SOC 2 Type II — auditoria completa | 2027-08-31 | Pendiente |
| ISO 27001 — certificacion | 2027-10-31 | Pendiente |
| Fabric CA integration (enrolamiento, registro) | 2027-09-30 | Pendiente |
| gRPC protocol (interop con redes Fabric existentes) | 2027-12-31 | Pendiente |

---

## Metricas de exito

| Hito | Criterio | Fecha limite |
|---|---|---|
| MVP entregable | Red de 6 nodos sobrevive restart sin perdida de datos | Q2 2026 |
| Adopcion Chile | Desarrollador de cero a transaccion en < 10 min con docs en espanol | Q3 2026 |
| Enterprise-ready | Piloto con al menos 1 organizacion en produccion | Q4 2026 |
| Certificable | SOC 2 Type II completado | H2 2027 |

---

## Como participar

- **Evaluadores tecnicos:** Clone el repo, siga el [Quick Start](QUICK-START.md), ejecute `./scripts/e2e-test.sh`
- **Partners potenciales:** Revise la [Comparacion con Fabric](FABRIC-COMPARISON.md) y los [Benchmarks](BENCHMARKS-RESULTS.md)
- **Camara de Blockchain:** Vea la [Presentacion](PRESENTACION.md) y el [FAQ](FAQ.md)

---

*Cerulean Ledger es un proyecto activo en desarrollo. Este roadmap refleja la direccion actual y puede ajustarse segun feedback de la comunidad y partners.*
