# Cerulean Ledger — Presentación ante la Cámara de Blockchain Chile

**Fecha:** 24 de abril de 2026

---

## Cerulean Ledger en una frase

Una plataforma DLT empresarial permisionada, desarrollada en Chile, construida en Rust, con ~95% de paridad funcional con Hyperledger Fabric y la primera DLT empresarial permisionada con firmas ML-DSA-65 (FIPS 204) integradas end-to-end — bloques, transacciones, endorsements e identidades.

---

## Por qué le importa a la Cámara

### Chile puede liderar en DLT empresarial post-cuántica

Hoy ninguna plataforma DLT empresarial en el mundo — ni Fabric, ni Corda, ni Ethereum privado, ni IOTA — ofrece firmas post-cuánticas en producción. Cerulean Ledger sí, y fue desarrollada íntegramente en Chile.

Esto posiciona al ecosistema blockchain chileno como referente en una capacidad que las grandes plataformas aún no ofrecen:

| Plataforma | País/Org | PQC en producción (2026) |
|---|---|---|
| Hyperledger Fabric | Linux Foundation (EE.UU.) | No — requiere reescribir BCCSP |
| R3 Corda | R3 (Reino Unido) | No |
| Ethereum privado | Consensys (EE.UU.) | No |
| IOTA Rebased | IOTA Foundation (Alemania) | No |
| **Cerulean Ledger** | **Chile** | **Sí — ML-DSA-65 (FIPS 204)** |

### Relevancia regulatoria inmediata

- **NIST FIPS 204** — Estándar publicado en agosto 2024. Cerulean Ledger lo implementa.
- **CNSS Policy 15 (NSA)** — Exige migración a algoritmos quantum-safe antes de 2030. Cerulean Ledger ya cumple.
- **eIDAS 2.0 (UE)** — Dirección regulatoria europea apunta a firmas PQC. Chile exporta a la UE.
- **CMF / SII** — La plataforma provee audit trail inmutable con trazabilidad por organización, exportable para reguladores.

---

## Qué es Cerulean Ledger

### No es una criptomoneda

Es infraestructura para redes empresariales donde múltiples organizaciones que no confían entre sí necesitan compartir datos de forma segura, auditable e inmutable, sin un intermediario central.

### Modelo probado: Execute-Order-Validate

El mismo modelo arquitectónico de Hyperledger Fabric (la plataforma DLT empresarial más desplegada del mundo):

```
Cliente ──► Gateway ──► Endorsers (simulan)
        ──► Orderer (Raft) ──► Commit (todos)
```

1. Los peers endorsantes simulan la transacción y firman el resultado
2. El servicio de ordenamiento establece un orden total y crea bloques
3. Todos los peers validan y commitean

### Capacidades completas

| Área | Capacidad |
|---|---|
| **Consenso** | Raft (crash fault) + BFT (byzantine fault), seleccionable |
| **Privacidad** | Channels multi-ledger + private data collections con TTL |
| **Smart contracts** | WebAssembly (Rust SDK) + EVM (Solidity) + externos (HTTP) |
| **Identidad** | DID (`did:cerulean:`) + credenciales verificables + X.509 MSP |
| **Criptografía** | Ed25519 + ML-DSA-65 (FIPS 204), migración gradual sin flag day |
| **Persistencia** | RocksDB con column families, índices secundarios |
| **Ejecución** | Paralela con wave scheduling (56K TPS medidos) |
| **Tokenomics** | Supply cap, halving, fee burn, storage deposits, base fee dinámico |
| **Bridge** | Framework cross-chain con escrow, Merkle proofs, replay protection |
| **Governance** | Propuestas on-chain, votación stake-weighted, timelock |
| **Light client** | Verificación via Merkle proofs para IoT/móvil |
| **Monitoreo** | Prometheus + Grafana incluidos |

---

## La ventaja post-cuántica en detalle

### El problema: "harvest now, decrypt later"

Los datos firmados hoy con criptografía clásica pueden ser interceptados y almacenados. Cuando exista una computadora cuántica suficientemente poderosa, esas firmas se rompen. Para registros que deben ser válidos por décadas — títulos de propiedad, contratos, historiales médicos — la protección debe empezar ahora.

### La solución: ML-DSA-65 (FIPS 204) end-to-end

Cerulean Ledger integra firmas post-cuánticas en toda la stack:

- Firmas de bloques y endorsements
- Propuestas de transacción
- Mensajes gossip P2P
- Identidades DID

Cada nodo elige su algoritmo: `SIGNING_ALGORITHM=ml-dsa-65`. Redes mixtas (nodos clásicos + PQC) operan simultáneamente. La migración es gradual, organización por organización, sin coordinar un cambio global.

### Por qué Fabric no puede hacer esto rápido

Agregar PQC a Fabric requiere:
- Reescribir el módulo criptográfico BCCSP (Go)
- Reconstruir toda la infraestructura de certificados X.509
- Actualizar peers, orderers y SDKs simultáneamente
- Manejar backward compatibility con datos ya firmados

Es un proyecto de años. Cerulean Ledger lo tiene hoy porque fue diseñado desde el inicio con `Vec<u8>` para firmas de longitud variable (Ed25519: 64 bytes, ML-DSA-65: 3,309 bytes).

---

## Casos de uso para Chile

### 1. Sector público: documentos verificables

**Problema:** Títulos universitarios, certificados de antecedentes, licencias profesionales — todos requieren verificación manual, son lentos de verificar y susceptibles a falsificación.

**Solución:** La universidad/institución emite una credencial verificable firmada con su DID. El empleador o entidad verificadora la valida criptográficamente en segundos, sin llamar a la institución emisora. Si hay fraude, la credencial se revoca y la revocación se propaga automáticamente.

**Actores:** Universidades, Registro Civil, municipalidades, colegios profesionales, empleadores.

### 2. Agroexportación: trazabilidad de cadena de suministro

**Problema:** Chile exporta fruta, salmón, vino a mercados que exigen trazabilidad completa. Hoy se maneja con papeles y PDFs falsificables.

**Solución:** Cada punto de la cadena (cosecha → transporte → SAG → aduana → importador) registra una transacción firmada. Las certificaciones fitosanitarias son credenciales verificables, no PDFs. El importador verifica toda la cadena criptográficamente.

**Actores:** Productores, logística, SAG, Aduana, importadores.

### 3. Salud: historial médico transfronterizo

**Problema:** Paciente chileno en emergencia en el extranjero. Sin acceso a historial, alergias, procedimientos previos.

**Solución:** El paciente tiene un DID con credenciales verificables selectivas. El médico extranjero verifica contra la blockchain. El paciente decide qué comparte.

**Actores:** Hospitales, MINSAL, redes médicas internacionales.

### 4. Finanzas: conciliación interinstitucional

**Problema:** Bancos, AFP, aseguradoras mantienen cada uno su versión de la verdad. Discrepancias = costos operativos enormes.

**Solución:** Registro compartido donde ambas partes firman cada transacción. La CMF opera un nodo observador con auditoría en tiempo real.

### 5. RRHH: verificación de documentos laborales

**Demo funcional** en el block explorer — flujo de 5 pasos:
1. Registrar issuer (empresa/institución)
2. Registrar candidato (DID)
3. Emitir credencial verificable
4. Verificar credencial
5. Perfil completo verificado

---

## Rendimiento medido (no estimado)

Benchmarks ejecutados con Criterion en Apple M-series, reproducibles.

| Métrica | Resultado |
|---|---|
| Ordering throughput | 23 millones tx/s (in-memory) |
| Ejecución paralela | 56K TPS (independientes), 39K TPS (mixtas) |
| RocksDB write | 104K bloques/s |
| Endorsement validation (10 orgs) | 43K validaciones/s |
| BFT rounds | 100 rounds/s |
| Footprint por nodo | ~50 MB RAM |
| Startup | ~2 segundos |

**Comparación con Fabric:** Un nodo Cerulean Ledger usa ~50 MB RAM vs ~500 MB+ de Fabric. El pipeline completo (BFT + ejecución + persistencia) procesa consistentemente sobre 5K TPS en escenarios realistas.

---

## Madurez del proyecto

| Métrica | Valor |
|---|---|
| Tests totales | **2,741** passing, 0 failed |
| Tests E2E (Docker) | 71 assertions, 20 categorías |
| Tests de penetración | 22 (OWASP top vectors) |
| Tests BFT adversarios | 16 escenarios |
| Auditoría de seguridad | 10/10 hallazgos remediados |
| Clippy warnings | 0 |
| CVEs en dependencias | 0 |
| Endpoints API documentados | 68 |
| Paridad con Fabric | ~95% (34 capacidades verificadas) |

### Infraestructura lista

- Docker deployment: 6 nodos + Prometheus + Grafana
- CI/CD: GitHub Actions, 4 jobs green
- SDK TypeScript publicado
- CLI operador (bcctl): 14 comandos
- Block explorer: React + Vite, UI en español
- Documentación completa: quick-start, API, deployment, benchmarks, security audit

---

## Alineación regulatoria

| Estándar / Marco | Relevancia para Chile | Estado |
|---|---|---|
| **NIST FIPS 204** | Estándar internacional de firmas PQC | Implementado |
| **CNSS Policy 15** | Exigencia NSA de quantum-safe para 2030 | Cumple |
| **FIPS 140-3** | Módulo criptográfico auditable | Self-tests KAT en startup |
| **SOC 2** | Estándar para servicios cloud/fintech | 13 criterios mapeados (11 Done) |
| **ISO 27001** | Gestión de seguridad de la información | 17 controles mapeados |
| **CMF** | Regulador financiero chileno | Audit trail inmutable, trazabilidad por org |
| **SII** | Servicio de Impuestos Internos | Registros verificables, timestamping |
| **eIDAS 2.0** | Regulación europea (mercados de exportación) | Firmas PQC alineadas |
| **Ley 19.628** | Protección de datos personales (Chile) | Private data collections con TTL |

---

## Qué pedimos a la Cámara

### Oportunidades de colaboración

1. **Visibilidad** — Incluir Cerulean Ledger en el catálogo de tecnologías DLT desarrolladas en Chile
2. **Pilotos** — Conectar con empresas miembro interesadas en pilotos (agroexportación, fintech, sector público)
3. **Reguladores** — Facilitar conversaciones con CMF/SII sobre estándares de audit trail y firma digital
4. **Ecosistema** — Acceso a desarrolladores para contribución open source y testing
5. **Eventos** — Espacio para demos técnicos en eventos de la Cámara

### Lo que ofrecemos

- Plataforma open source (MIT) sin costo de licencia
- Soporte técnico directo para pilotos
- Demo funcional disponible en minutos
- Documentación en español
- Capacitación para equipos técnicos de empresas miembro

---

## Roadmap

| Período | Foco | Relevancia Chile |
|---|---|---|
| **Q2 2026** | MVP completado | SDK TS, CLI, Docker, benchmarks, docs |
| **Q3 2026** | Ecosistema Chile | Docs español, SDK Python, onboarding multi-org en < 10 min |
| **Q4 2026** | Enterprise hardening | FIPS 140-3 completo, encryption at rest, audit trail regulatorio |
| **2027** | Certificaciones | SOC 2, ISO 27001, pilotos en producción |

---

## Comparación competitiva resumida

| Criterio | Cerulean Ledger | Hyperledger Fabric | IOTA Rebased | Ethereum privado |
|---|---|---|---|---|
| Origen | **Chile** | Linux Foundation (EE.UU.) | IOTA Foundation (Alemania) | Consensys (EE.UU.) |
| Tipo | Permisionada | Permisionada | Pública (DPoS) | Permisionada |
| Lenguaje | Rust | Go/Java | Rust/Move | Go/Solidity |
| **PQC producción** | **Sí (FIPS 204)** | **No** | **No** | **No** |
| Channels privados | Sí | Sí | No | No |
| BFT | Sí | Solo Raft | Sí | IBFT/Clique |
| Ejecución paralela | Sí | No | Sí | No |
| Deployment | 1 binario, Docker Compose | Múltiples containers + CAs | Validator node | Múltiples containers |
| Footprint | ~50 MB | ~500 MB+ | ~200 MB | ~300 MB+ |
| Tests | 2,741 | — | — | — |
| Licencia | MIT | Apache 2.0 | Apache 2.0 | Varias |

---

## Datos clave para la discusión

- **Primera DLT empresarial con PQC** — ventana de oportunidad antes de que las grandes plataformas alcancen
- **Desarrollada en Chile** — soberanía tecnológica, sin dependencia de proveedores extranjeros
- **Open source (MIT)** — sin vendor lock-in, cualquier organización puede auditar y contribuir
- **2,741 tests, 0 failures** — rigor técnico demostrable
- **Demo en vivo disponible** — `docker compose up` → red de 6 nodos en 4 minutos
- **Casos chilenos documentados** — agroexportación, sector público, salud, finanzas, RRHH

---

*Cerulean Ledger — Tecnología DLT empresarial. Desarrollada en Chile. Segura ante computación cuántica.*
