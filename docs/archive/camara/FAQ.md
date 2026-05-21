# Preguntas Frecuentes — Cerulean Ledger

Preparado para la presentación ante la directiva de la Cámara de Blockchain Chile (abril 2026). Organizado por audiencia: directiva de la Cámara, empresas miembro, evaluadores técnicos y reguladores.

---

## Para la directiva de la Cámara

### Qué es Cerulean Ledger?

Una plataforma DLT empresarial permisionada, desarrollada en Chile, con ~95% de paridad funcional con Hyperledger Fabric y la primera DLT empresarial permisionada con firmas ML-DSA-65 (FIPS 204) integradas end-to-end — bloques, transacciones, endorsements e identidades.

### Es una criptomoneda?

No. Es infraestructura para redes empresariales privadas donde múltiples organizaciones necesitan compartir datos de forma segura, auditable e inmutable. Los participantes son organizaciones conocidas y autorizadas. Sin token público, sin exchange, sin participación anónima.

### Por qué le importa a la Cámara?

Tres razones:

1. **Posicionamiento de Chile** — Es la primera DLT empresarial permisionada con firmas FIPS 204 integradas end-to-end. Ninguna plataforma comparable — ni Fabric, ni Corda, ni IOTA — ofrece esta capacidad hoy. Esto posiciona al ecosistema blockchain chileno como referente.

2. **Casos de uso locales listos** — Tiene casos documentados y demostrables para sectores chilenos: agroexportación (trazabilidad SAG→Aduana→importador), sector público (documentos verificables), salud (historial médico transfronterizo), finanzas (conciliación CMF), RRHH (credenciales laborales con demo funcional).

3. **Soberanía tecnológica** — Open source (MIT), sin dependencia de proveedores extranjeros, sin vendor lock-in. Cualquier empresa miembro de la Cámara puede auditarlo, desplegarlo y contribuir.

### Cómo se compara con lo que ya usan las empresas miembro?

| Aspecto | Cerulean Ledger | Hyperledger Fabric |
|---|---|---|
| PQC | Sí (ML-DSA-65, FIPS 204) | No — años de desarrollo por delante |
| Deployment | 1 binario, `docker compose up` (4 min) | Múltiples containers, CAs Go/Java, 30+ min |
| RAM por nodo | ~50 MB | ~500 MB+ |
| TPS | 56K (medidos) | ~3K |
| Complejidad | Variables de entorno | YAML extenso, configtxgen, cryptogen |

### Qué le piden a la Cámara?

1. **Visibilidad** — Incluirlo en el catálogo de tecnologías DLT chilenas
2. **Pilotos** — Conectar con empresas miembro interesadas
3. **Reguladores** — Facilitar conversaciones con CMF/SII
4. **Ecosistema** — Acceso a desarrolladores para contribución y testing
5. **Eventos** — Espacio para demos en eventos de la Cámara

### Qué ofrecen a cambio?

- Plataforma open source sin costo de licencia
- Soporte técnico directo para pilotos con empresas miembro
- Demo funcional disponible en minutos
- Capacitación para equipos técnicos
- Documentación en español

### Si ya existe Fabric, por qué necesitamos otra plataforma?

Tres razones concretas:

1. **Criptografía post-cuántica** — Fabric no la tiene y tardaría años en integrarla (requiere reescribir su módulo criptográfico en Go, reconstruir infraestructura de certificados, actualizar todos los peers y SDKs). Cerulean Ledger la tiene hoy.
2. **Barrera de entrada** — Fabric requiere equipos especializados en Go/Java, Docker, CAs. Cerulean Ledger se despliega con un solo comando. Esto baja el costo de adopción para empresas chilenas.
3. **Rendimiento** — 10x menos RAM, órdenes de magnitud más rápido en ordering. Para un mercado que recién adopta DLT, empezar con la tecnología más eficiente tiene sentido.

### Cuál es el modelo de negocio?

Open source (MIT) + servicios:

- Soporte empresarial y SLAs
- Consultoría de implementación y pilotos
- Servicios managed (nodos como servicio)
- Capacitación y certificación de desarrolladores
- Desarrollo de aplicaciones verticales

### Qué tan maduro está?

| Métrica | Valor |
|---|---|
| Paridad con Fabric | ~95% (34 capacidades verificadas punto a punto) |
| Tests | 2,741 passing, 0 failed |
| Auditoría de seguridad | 10/10 hallazgos remediados |
| Endpoints API | 68 documentados |
| Docker deployment | 6 nodos + Prometheus + Grafana |
| Block explorer | UI completa en español |
| Compliance mapeado | SOC 2 (13 criterios), ISO 27001 (17 controles) |

### Qué falta?

El core técnico está listo. Los gaps restantes para enterprise son de producto, no de tecnología:

- Documentación completa en español (en progreso)
- SDK Python (ecosistema fintech/gobierno chileno)
- Onboarding automatizado multi-org (< 10 min por organización)
- Certificaciones formales (SOC 2, ISO 27001) — requieren período de observación
- Audit trail exportable para reguladores (SII, CMF)

---

## Para evaluadores técnicos

### Qué modelo de consenso usa?

Dos backends seleccionables:

- **Raft** (crash fault tolerance) — para redes donde los nodos son confiables pero pueden fallar. Persistente en RocksDB con crash recovery. Idéntico al modelo de Fabric.
- **BFT** (byzantine fault tolerance) — protocolo HotStuff-inspired con 3 fases (Prepare → PreCommit → Commit), tolerancia f = (n-1)/3 fallas bizantinas, rotación de líder, backoff exponencial.

Se selecciona con `CONSENSUS_MODE=raft|bft`.

### Cómo se ejecutan los smart contracts?

Dos runtimes:

1. **Wasm (WebAssembly)** — Runtime principal via Wasmtime v36. Los contratos se escriben en Rust (chaincode-sdk disponible), se compilan a `wasm32-unknown-unknown`, y se ejecutan con fuel metering y límites de memoria configurables. Host functions: `put_state`, `get_state`, `emit_event`, `invoke` (cross-chaincode), `history_for_key`.

2. **EVM** — Compatibilidad Solidity via revm. Deploy, call, static-call de contratos Ethereum. ABI encoding/decoding completo.

3. **External** — Chaincode-as-a-service via HTTP POST. Compatible con microservicios existentes.

### Cómo maneja la ejecución paralela de transacciones?

El executor analiza los read-write sets de las transacciones y construye un grafo de dependencias basado en conflictos RAW/WAW/WAR. Luego agrupa las transacciones no-conflictivas en "waves" que se ejecutan concurrentemente (tokio tasks intra-wave). La validación MVCC se realiza por wave. Los writes se aplican en orden determinista.

Benchmark: 56K TPS para 500 transacciones independientes en 1 wave.

### Qué algoritmos criptográficos soporta?

| Operación | Algoritmos |
|---|---|
| Firmas digitales | Ed25519 (default), ML-DSA-65 (FIPS 204) |
| Hash | SHA-256 |
| TLS | TLS 1.3 (rustls), mTLS, certificate pinning |
| KAT self-tests | Ed25519, ML-DSA-65, SHA-256 en startup |
| Key zeroization | Ed25519 via ZeroizeOnDrop, ML-DSA-65 via custom Drop |

ML-DSA-65 produce firmas de 3,309 bytes (vs 64 bytes Ed25519). Todos los campos de firma en el codebase usan `Vec<u8>` para soportar ambos tamaños.

### Cómo funciona la persistencia?

- **RocksDB** — Backend principal con Column Families: `blocks`, `transactions`, `identities`, `credentials`, `meta`, `tx_by_block`, `organizations`, `policies`, `collections`, `chaincode_defs`, `private_*`. Índices secundarios con claves zero-padded para prefix scans eficientes.
- **MemoryStore** — Default para desarrollo y tests, basado en HashMap.
- **CouchDB** — Implementación disponible para world state.

Se selecciona con `STORAGE_BACKEND=rocksdb`.

### Cuántos tests tiene y qué cubren?

2,741 tests totales:

| Categoría | Tests | Qué validan |
|---|---|---|
| Unit tests (lib) | ~2,439 | Todos los módulos: storage, consensus, identity, endorsement, ordering, gateway, channels, chaincode, bridge, governance, tokenomics, light client, EVM |
| Integration tests | ~302 | BFT E2E (16), bridge E2E (11), TPS benchmarks (15), penetration (22), multi-node P2P, store API, fuzz |
| E2E (Docker) | 71 | 20 categorías: orgs, policies, channels, mining, tx lifecycle, private data, discovery, gateway, chain integrity, Prometheus, store CRUD |

Además: property-based tests (proptest) para criptografía, stress tests de 1000+ operaciones.

### Puedo usarlo con Docker?

Sí. `docker compose up -d` levanta:

- 3 peers (2 orgs)
- 3 orderers (Raft cluster)
- Prometheus (métricas)
- Grafana (dashboards)
- TLS mutuo auto-generado

```bash
docker compose build && docker compose up -d
curl -sk https://localhost:8080/api/v1/health
```

### La API es REST o gRPC?

REST (HTTP/JSON) con Actix-Web 4. 68 endpoints documentados. Response envelope consistente:

```json
{
  "status": "success",
  "status_code": 200,
  "message": "...",
  "data": { ... },
  "timestamp": "2026-04-23T...",
  "trace_id": "..."
}
```

gRPC está en roadmap solo si hay demanda de interoperabilidad con Fabric.

### Cómo se compara con IOTA Rebased?

| Aspecto | IOTA Rebased | Cerulean Ledger |
|---|---|---|
| Red | Pública (DPoS) | Permisionada |
| VM | MoveVM | Wasm + EVM |
| Consenso | Mysticeti (DAG BFT) | Raft + BFT selectable |
| TPS claim | 50K+ | 56K (medido, independientes) |
| PQC | No | Sí (ML-DSA-65) |
| Channels privados | No | Sí |
| Private data | No | Sí (collections con ACL) |
| Governance | On-chain | On-chain (propuestas + voting + timelock) |
| Bridge | 150+ chains producción | Framework listo, sin chains conectadas |
| Target | dApps públicas | Enterprise B2B |

---

## Para clientes enterprise

### Cuánto cuesta implementarlo?

El software es open source (MIT). Los costos son:

- **Infraestructura** — Cada nodo requiere ~50 MB RAM. Una red mínima (4 nodos) corre en instancias cloud mínimas (~$50/mes total).
- **Implementación** — Depende de la complejidad del caso de uso. Un piloto básico (DID + credenciales + 2 orgs) se configura en días.
- **No hay gas fees** — Las transacciones no tienen costo por operación.

### Necesito conocer Rust para usarlo?

No. Los smart contracts se pueden escribir en Rust (SDK disponible) o Solidity (EVM compatible), y también se pueden implementar como servicios HTTP externos en cualquier lenguaje. Los SDKs cliente están en TypeScript (Python en roadmap).

### Cómo migro desde Hyperledger Fabric?

El modelo arquitectónico es idéntico (endorse→order→commit, channels, private data, chaincode lifecycle), pero el protocolo de wire es HTTP/JSON, no gRPC/Protobuf. Esto significa:

- **Lógica de negocio** — Se porta directamente (mismos conceptos: endorsement policies, channels, private data)
- **Smart contracts** — Se reescriben en Rust/Wasm o Solidity (no Go/Java)
- **SDKs** — Se cambia `fabric-network` por `@cerulean/sdk` (API similar)
- **Infraestructura** — Se simplifica significativamente (menos componentes)

La interoperabilidad directa con redes Fabric existentes requiere gRPC (en roadmap).

### Qué pasa si el proyecto se abandona?

- Código 100% open source (MIT)
- Sin dependencia de servicios cloud propietarios
- Rust y todas las dependencias son open source
- Una organización puede fork y mantener independientemente
- La documentación completa permite continuidad

### Tienen un demo funcional?

Sí. El block explorer (React + Vite) incluye un demo de 5 pasos de verificación de credenciales RRHH:

1. Registrar issuer (organización)
2. Registrar candidato (DID)
3. Emitir credencial verificable
4. Verificar credencial
5. Ver perfil completo

Disponible en `http://localhost:5173/demo` con el nodo corriendo.

También: `./scripts/try-it.sh` ejecuta un demo interactivo sin Docker.

### Soporta alta disponibilidad?

Sí:

- **Raft cluster** — 3 orderers con persistencia en disco. Tolera 1 caída sin pérdida de servicio.
- **BFT** — Tolera f = (n-1)/3 nodos bizantinos.
- **Pull-sync** — Nodos se re-sincronizan automáticamente al reconectarse.
- **Graceful shutdown** — SIGTERM drena conexiones y flushea RocksDB.
- **Health check** — `/health` reporta estado de storage, peers y ordering; status "degraded" cuando hay problemas.

---

## Para reguladores

### Cumple con estándares NIST?

- **FIPS 204 (ML-DSA-65)** — Firmas digitales post-cuánticas implementadas según el estándar NIST, security level 3.
- **FIPS 140-3** — Self-tests KAT (Known Answer Tests) ejecutados en cada startup para Ed25519, ML-DSA-65 y SHA-256. El nodo se niega a arrancar si fallan.
- **SHA-256** — Usado para hashing en toda la plataforma.

### Cómo se audita?

- **Audit trail inmutable** — Cada transacción queda registrada con timestamp, firma del autor (DID), y es irrevocable.
- **Export CSV** — Los registros de auditoría se pueden exportar para revisión.
- **Regulador como observador** — Una organización reguladora puede operar un nodo con permisos de solo lectura para auditar en tiempo real.
- **Trazabilidad por org** — Cada acción está asociada a una organización identificada via X.509/mTLS.

### Los datos son inmutables?

Sí. Una vez que un bloque es commiteado:

- No se puede modificar retroactivamente (hash encadenado)
- No se puede eliminar (append-only)
- Cada bloque tiene la firma del orderer
- Los endorsements de las transacciones son verificables por cualquier nodo

### Cómo maneja datos personales (GDPR, Ley 19.628)?

- **Private data collections** — Los datos personales se almacenan en colecciones privadas accesibles solo por organizaciones autorizadas. En el ledger público solo queda un hash.
- **TTL/Purge** — Las colecciones privadas soportan `blocks_to_live`: los datos se purgan automáticamente después de N bloques.
- **Selective disclosure** — Las credenciales verificables permiten al titular decidir qué información compartir.
- **Nota importante** — El ledger principal es inmutable. Los datos sensibles nunca deben escribirse directamente en el ledger; siempre en private data collections con TTL apropiado.

### Existe documentación de compliance?

Sí, disponible en `docs/`:

| Documento | Contenido |
|---|---|
| `COMPLIANCE-FRAMEWORK.md` | Mapeo SOC 2 (13 criterios) + ISO 27001 (17 controles) |
| `FIPS-140-MODULE.md` | Boundary del módulo criptográfico, algoritmos aprobados, gestión de claves |
| `CERTIFICATION-ROADMAP.md` | Roadmap de certificación en 3 niveles |
| `SECURITY-AUDIT.md` | Auditoría de seguridad completa con estado de remediación |
| `ENCRYPTION-AT-REST.md` | Guía de cifrado en disco (LUKS, Docker, cloud) |
| `PQC-ENTERPRISE.md` | Posicionamiento PQC para industrias reguladas |

---

## Preguntas técnicas adicionales

### Qué pasa si un nodo se cae?

- **Raft ordering** — Si un orderer se cae, el cluster sigue operando mientras haya mayoría (2/3). Al volver, se re-sincroniza automáticamente desde el log persistido en RocksDB.
- **Peers** — Al reconectarse, el gossip protocol detecta el gap de altura y ejecuta pull-sync para recuperar bloques faltantes.
- **Datos** — RocksDB es crash-safe. Los datos persisten en disco con write-ahead log.

### Se puede correr en ARM? En cloud?

- **ARM** — Sí, compilado y testeado en Apple M-series (ARM64).
- **x86** — Sí, el Dockerfile usa multi-stage build.
- **Cloud** — Cualquier proveedor con Docker support. El footprint bajo (~50 MB) permite instancias mínimas.
- **On-premise** — Sí, sin dependencias de servicios cloud.

### El código está auditado por seguridad?

Sí. `docs/SECURITY-AUDIT.md` documenta la auditoría completa:

- 10 hallazgos identificados, 10 remediados
- ACL deny-by-default en todas las rutas
- Rate limiting
- Input validation middleware
- 22 tests de penetración (OWASP top vectors)
- Checkpoint HMAC para integridad de archivos
- Wasmtime v36 (15 CVEs resueltos vs v21)
- Zero clippy warnings

### Cuál es la licencia?

MIT — permite uso comercial, modificación, distribución y uso privado sin restricciones.
