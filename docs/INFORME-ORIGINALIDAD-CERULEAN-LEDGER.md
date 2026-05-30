# Informe de Originalidad — cerulean-ledger vs cerulean-dlt-framework

**Fecha:** 28 de mayo de 2026
**Autor del análisis:** Auditoría automatizada sobre repositorios públicos de GitHub
**Objetivo:** Determinar si existe copia de código entre los repositorios `clementeaf/cerulean-ledger` y `Alefrank76/cerulean-dlt-framework`

---

## 1. Resumen ejecutivo

**No existe copia de código entre ambos repositorios.** Son proyectos completamente independientes que utilizan frameworks, lenguajes, arquitecturas y patrones de diseño distintos. La única coincidencia es temática: ambos abordan conceptos de blockchain con identidad, compliance y tokenización de activos, lo cual es un dominio público ampliamente documentado.

El repositorio `cerulean-ledger` es **5 meses más antiguo**, tiene un historial de desarrollo continuo con 100+ commits, y contiene 101,607 líneas de código Rust en 292 archivos fuente. El repositorio `cerulean-dlt-framework` fue creado en un solo día con 7 commits y contiene 47.4 KB en 12 archivos.

---

## 2. Evidencia temporal (precedencia)

| Dato | cerulean-ledger (clementeaf) | cerulean-dlt-framework (Alefrank76) |
|---|---|---|
| **Fecha de creación del repo** | 2025-12-06T16:09:29Z | 2026-05-10T19:58:47Z |
| **Primer commit** | 2025-12-06 13:09 (UTC-3) | 2026-05-10 17:10 (UTC-4) |
| **Ultimo push** | 2026-05-22T17:00:07Z | 2026-05-10T22:16:19Z |
| **Total de commits** | 458 (desarrollo continuo, 6 meses) | 7 (todos el mismo dia) |
| **Tamanio del repo** | 9,799 KB | 20 KB |
| **Archivos fuente Rust** | 292 archivos, 101,607 lineas | 8 archivos .rs, 706 lineas |
| **Otros archivos** | Docs, scripts, configs, tests | 1 .js, 2 .html |

**Conclusion temporal:** `cerulean-ledger` precede a `cerulean-dlt-framework` por 155 dias. El historial de git es verificable e inmutable.

---

## 3. Historial de commits de cerulean-ledger (seleccion cronologica)

```
2025-12-06  2e06ae0  Initial commit: Rust Blockchain implementation with P2P, consensus, mining rewards
2025-12-06  50ab3d0  Add comprehensive README.md
2025-12-07  b96362d  Feat: Implementar NFTs basicos (ERC-721 simplificado)
2025-12-08  7340c79  Implementacion completa de Staking PoS (Fase 1)
2025-12-10  d6556a5  feat: Completar Prioridad 1 - Calidad y estandares
2025-12-19  c796220  feat(identity): implement Week 4 DID foundations with key management
2026-04-04  c4bce3c  feat: add HSM signing provider, organizational units
2026-04-16  a8f2afb  feat: add on-chain governance with proposals, stake-weighted voting
2026-04-28  c3f1cc0  feat: add post-quantum readiness layer with PQC enforcement
2026-05-08  a40577b  feat: oracle external connectors, forensic engine, ISO compliance module
2026-05-09  a790602  feat: complete ISO compliance — 20022 full cycle, 3166/4217 global
2026-05-09  7374e97  feat: ERC-3643 security tokens, auto-discovery gossip
2026-05-11  6cef6dd  feat: add zero-knowledge proofs for sovereign identity claims
2026-05-17  ...      Legacy storage removal — API layer fully migrated to BlockStore
2026-05-19  5580369  feat: add vault recovery via HMAC-SHA3-256
2026-05-21  18e51de  feat: add alias registry and invitation system
2026-05-22  e6f4c70  feat: add Optimistic ML Oracle Phase 4 — zkML bridge
```

### Historial completo de cerulean-dlt-framework

```
2026-05-10 17:10  01f44d9  [Alefrank76] Arquitectura Base Motor horizontal, IDS y Tokenizacion RWA
2026-05-10 17:17  887c1ca  [Alefrank76] Estructura base y codigos DLT
2026-05-10 17:27  870e5cf  [Alefrank76] Paquetes de Cumplimiento ISO 27001 e ISO 20022
2026-05-10 17:44  665c372  [Alefrank76] Sandbox Certificador y Pruebas de Propiedad RWA
2026-05-10 17:57  05d9021  [Alefrank76] Integracion GDPR y Oraculo BCN
2026-05-10 18:12  daa678a  [Alefrank76] Dashboards de Telemetria, Sandbox y App Ciudadana Soberana
2026-05-10 18:19  0bca7ca  [Alefrank76] Dashboard interactivo UI: Panel DLT y Consola Forense
```

**7 commits en 69 minutos.** Sin historial previo ni posterior.

---

## 4. Diferencias arquitectonicas fundamentales

### 4.1 Framework y runtime

| Aspecto | cerulean-ledger | cerulean-dlt-framework |
|---|---|---|
| **Framework** | Actix-Web 4 (servidor HTTP nativo) | Substrate (Polkadot SDK pallets) |
| **Runtime** | Binario standalone (`cargo run`) | Substrate node/runtime (no presente) |
| **Storage** | RocksDB custom con `BlockStore` trait | Substrate `StorageMap` / `StorageValue` |
| **Networking** | P2P custom + gossip protocol | Substrate networking (no implementado) |
| **Consensus** | DAG + HotStuff BFT + DPoS custom | Substrate consensus (no implementado) |
| **Crypto** | `pqc_crypto_module` (crate propio, FIPS-oriented) | `pqcrypto-dilithium` / `pqcrypto-kyber` (deps externas) |
| **API** | REST HTTP con Actix-Web, `ApiResponse<T>` envelope | No tiene API — solo pallets |

**Estos frameworks son mutuamente incompatibles.** Codigo de Actix-Web no compila en Substrate y viceversa. Es imposible copiar codigo entre ellos.

### 4.2 Dependencias (Cargo.toml)

**cerulean-ledger:**
```toml
actix-web = "4.5"
actix-cors = "0.7"
tokio = { version = "1", features = ["full"] }
ed25519-dalek = "2.1"
rocksdb = "0.22"
revm = "..."
pqc_crypto_module = { path = "crates/pqc_crypto_module" }
```

**cerulean-dlt-framework:**
```toml
sp-core = { git = "https://github.com/paritytech/polkadot-sdk.git" }
sp-runtime = { git = "https://github.com/paritytech/polkadot-sdk.git" }
frame-support = { git = "https://github.com/paritytech/polkadot-sdk.git" }
frame-system = { git = "https://github.com/paritytech/polkadot-sdk.git" }
pqcrypto-dilithium = "0.2.0"
```

**Cero dependencias en comun** (exceptuando la stdlib de Rust).

---

## 5. Comparacion modulo por modulo

### 5.1 Identity / Identidad Soberana

**cerulean-ledger** (`src/identity/`, 8 archivos, ~2,000 lineas):
```rust
// src/identity/mod.rs
pub mod did;
pub mod dual_signing;
pub mod hsm;
pub mod keys;
pub mod pqc_policy;
pub mod signing;
pub mod zkp;

pub struct IdentityConfig {
    pub key_derivation_path: String,    // BIP-44 derivation
    pub credential_ttl_days: u32,
    pub revocation_check_enabled: bool,
}
```
- DID documents con key management
- Dual signing (Ed25519 + ML-DSA-65) para migracion post-cuantica
- HSM provider
- ZKP basado en SHA-256 commitments con predicados (RangeProof, SetMembership)

**cerulean-dlt-framework** (`Verticales/v1-identidad-soberana/lib.rs`, 1 archivo, 95 lineas):
```rust
// Substrate pallet
#[frame_support::pallet]
pub mod pallet {
    pub struct SovereignIdentity<AccountId> {
        pub owner: AccountId,
        pub zkp_hash: [u8; 32],
        pub neuro_consent_active: bool,
        pub fea_pub_key: [u8; 64],
    }
}
```
- Pallet Substrate con storage map simple
- Un solo extrinsic (`register_identity`)
- Sin DID, sin key management, sin HSM, sin dual signing

**Veredicto: Implementaciones completamente diferentes. Cero codigo compartido.**

### 5.2 ISO 20022 Compliance

**cerulean-ledger** (`src/compliance/iso20022.rs`):
```rust
pub enum MessageType {
    Pacs008, Pacs002, Pacs004, Pain001, Pain002, Camt052, Camt053,
}

pub struct CurrencyAmount {
    pub amount: u64,
    pub currency: String,  // Validado contra ISO 4217
}
// Validacion completa con thiserror, IBAN check, BIC check, country codes
```
- 7 tipos de mensaje soportados
- Validacion de IBAN, BIC, currency codes, country codes
- Integrado con modulos `iso3166`, `iso4217`, `iso8601`
- Error handling con `thiserror`

**cerulean-dlt-framework** (`Horizontal Layer/Pallets/Pallet-iso-20022/lib.rs`):
```rust
pub struct Iso20022Message {
    pub message_id: Vec<u8>,
    pub debtor_account: Vec<u8>,
    pub creditor_account: Vec<u8>,
    pub amount: u64,
    pub currency_code: [u8; 3],
}
// Un solo extrinsic: process_pacs008_transfer
```
- Solo pacs.008
- Sin validacion de IBAN, BIC, country codes
- Pallet Substrate con un extrinsic

**Veredicto: Implementaciones completamente diferentes. Distinto nivel de profundidad, distintos tipos, distinta API.**

### 5.3 Governance / Democracia

**cerulean-ledger** (`src/governance/`, 4 archivos):
- Sistema de propuestas con lifecycle completo
- Votacion ponderada por stake
- Quorum y pass threshold configurables
- Delegacion de voto
- Parameter registry on-chain

**cerulean-dlt-framework** (`Verticales/v3-democracia-directa/lib.rs`):
- Pallet Substrate con referendums
- 1 ciudadano = 1 voto (no ponderado)
- Sin delegacion, sin quorum configurable

**Veredicto: Modelos de votacion opuestos (stake-weighted vs 1-persona-1-voto). Cero codigo compartido.**

### 5.4 Sandbox / Certificacion

**cerulean-ledger** (`src/regulatory/sandbox.rs`):
- Checks programaticos contra Ley 21.663, GDPR, ISO
- Retorna `Vec<CheckResult>` con `Pass`/`Fail`/`NotApplicable`
- Evidencia textual por cada check

**cerulean-dlt-framework** (`Horizontal Layer/Pallets/Pallet-sandbox-certificador/lib.rs`):
- Pallet Substrate con `CertificationRecord`
- Requiere `ChamberAuditorOrigin` (origin privilegiado)
- Certificacion manual por auditores

**Veredicto: Enfoques opuestos (automatizado vs manual). Cero codigo compartido.**

### 5.5 Tokenizacion RWA

**cerulean-ledger**: Modulo completo en `src/registry/tokenization.rs` con ERC-3643 security tokens, asset registry, certified export. Integrado con BlockStore y API REST.

**cerulean-dlt-framework** (`Verticales/v2-rwa-tokenizacion/lib.rs`):
- Pallet Substrate con `RealWorldAsset` struct
- Clasificacion juridica (`Movable`/`Immovable`)
- Un extrinsic: `request_tokenization`

**Veredicto: Implementaciones completamente diferentes.**

---

## 6. Analisis de overlap de codigo

### Metodo

Se extrajeron todas las lineas no triviales (≥15 caracteres, excluyendo lineas en blanco, comentarios, llaves sueltas, `use`/`mod` statements, atributos `#[...]`, y tokens minimos como `Ok(())`, `None`, `Some(...)`) de ambos repositorios. Se normalizaron eliminando indentacion y se compararon con `comm -12` (interseccion exacta de conjuntos ordenados).

- **cerulean-dlt-framework:** 215 lineas no triviales unicas (de 12 archivos fuente)
- **cerulean-ledger:** 26,974 lineas no triviales unicas (de 292 archivos en `src/` + `crates/`)

### Resultado

| Metrica | Resultado |
|---|---|
| **Lineas no triviales en cerulean-dlt-framework** | 215 |
| **Lineas no triviales en cerulean-ledger** | 26,974 |
| **Coincidencias exactas** | **4** (1.8% del repo comparado) |
| **Funciones con firma identica** | 0 |
| **Structs con campos identicos** | 0 |
| **Imports compartidos** | 0 (frameworks incompatibles) |
| **Nombres de archivo identicos** | 0 |
| **Coincidencias en `crates/pqc_crypto_module`** | 0 |

### Las 4 lineas coincidentes

| Linea | Contexto en cerulean-dlt-framework | Contexto en cerulean-ledger |
|---|---|---|
| `pub amount: u64,` | `Iso20022Message` (pallet Substrate) | `FaucetRequest`, `TestnetConfig`, `PatternMatch` |
| `pub timestamp: u64,` | `AuditLogEntry` (pallet ISO 27001) | `DagVertex`, `LightBlockHeader`, `Endorsement` |
| `pub votes_for: u64,` | `Referendum` (pallet democracia) | `Proposal` (governance_contracts, oracle_collateral) |
| `pub votes_against: u64,` | `Referendum` (pallet democracia) | `Proposal` (governance_contracts, oracle_collateral) |

**Analisis:** Las 4 coincidencias son declaraciones de campos struct triviales del lenguaje Rust (`pub nombre: u64,`). Son el equivalente a nombrar una variable `i` en un ciclo for — cualquier programa con montos, timestamps o votaciones usara estos nombres. Los structs que contienen estos campos son completamente distintos en nombre, campos restantes, tipos asociados y framework.

**No existe ninguna linea de logica, funcion, tipo compuesto, o patron de diseno copiado.**

---

## 7. Indicadores adicionales

### 7.1 Perfil de los repositorios

| Indicador | cerulean-ledger | cerulean-dlt-framework |
|---|---|---|
| **Commits** | 458 | 7 |
| **Periodo de desarrollo** | 6 meses (dic 2025 — may 2026) | 69 minutos (10 mayo 2026) |
| **Tests** | Si (unit, integration, E2E, fuzz) | No |
| **CI/CD** | Si (deploy pipeline, Docker) | No |
| **Documentacion** | Extensa (docs/, CHANGELOG, API ref) | No |
| **Binario ejecutable** | Si (server funcional, deployed en AWS) | No (pallets sin node/runtime) |
| **Dependencias externas** | 40+ crates | 6 crates (Polkadot SDK) |

### 7.2 El repo cerulean-dlt-framework no compila

El repositorio `cerulean-dlt-framework` contiene solo pallets Substrate sueltos sin:
- Un `node/` o `runtime/` que los integre
- Un `Cargo.lock`
- Configuracion de chain spec
- Tests

Los pallets hacen referencia a traits no definidos (`crate::traits::SovereignIdentityVerifier`) que no existen en el repositorio. **El codigo no es ejecutable.**

### 7.3 Cuenta de GitHub del autor

- `Alefrank76`: cuenta creada 2022-07-22, **1 repositorio publico** (este)
- `clementeaf`: cuenta activa con multiples repositorios incluyendo cerulean-explorer, cerulean-voto, cerulean-sdks

### 7.4 Interaccion entre las cuentas

Se detecto **un unico vinculo** entre las cuentas: un evento `MemberEvent` registrado por la API de GitHub.

**Hallazgo:** El 14 de mayo de 2026 a las 17:20:33 UTC, `Alefrank76` agrego a `clementeaf` como **colaborador** del repositorio `cerulean-dlt-framework`.

| Dato | Valor |
|---|---|
| **Fecha de la invitacion** | 2026-05-14T17:20:33Z |
| **Accion** | `"action": "added"` |
| **Quien invito** | Alefrank76 (owner del repo) |
| **Quien fue invitado** | clementeaf |
| **Permisos otorgados** | push, triage, pull (NO admin, NO maintain) |
| **Event ID** | 9471424279 |

**Actividad posterior de clementeaf en ese repo:**

| Tipo de actividad | Cantidad |
|---|---|
| Commits | **0** |
| Pull Requests | **0** |
| Issues creados | **0** |
| Comentarios | **0** |
| Forks | **0** |
| Stars | **0** |

**Analisis cronologico:**
1. `Alefrank76` creo su repo el **10 de mayo de 2026** y subio todo el codigo en 69 minutos
2. **4 dias despues**, el 14 de mayo, agrego a `clementeaf` como colaborador
3. `clementeaf` **nunca realizo ninguna accion** en ese repositorio — ni un commit, ni un PR, ni un comentario

**Conclusion:** La unica interaccion fue iniciada por `Alefrank76`, no por `clementeaf`. El hecho de que Alefrank76 invitara a clementeaf sugiere que conocia su trabajo previo. clementeaf nunca contribuyo codigo a ese repositorio. Todo el codigo del repo es exclusivamente de Alefrank76 segun el historial de git.

**Verificacion:**
```bash
# Ver el evento de colaborador
gh api repos/Alefrank76/cerulean-dlt-framework/events | \
  python3 -c "import sys,json; [print(f'{e[\"actor\"][\"login\"]} — {e[\"type\"]} — {e[\"created_at\"]}') for e in json.load(sys.stdin)]"

# Ver colaboradores actuales
gh api repos/Alefrank76/cerulean-dlt-framework/collaborators | \
  python3 -c "import sys,json; [print(f'{c[\"login\"]} — {c[\"permissions\"]}') for c in json.load(sys.stdin)]"

# Confirmar que clementeaf no tiene commits
cd cerulean-dlt-framework && git log --all --format="%an" | sort -u
# Resultado esperado: solo "Alefrank76"
```

Adicionalmente:
- `cerulean-ledger` no tiene forks publicos
- `cerulean-dlt-framework` no es un fork de ningun repositorio (`"fork": false`)

---

## 8. Conclusion

### 8.1 Hechos verificables

1. **Precedencia temporal:** `cerulean-ledger` fue creado el 6 de diciembre de 2025. `cerulean-dlt-framework` fue creado el 10 de mayo de 2026, **155 dias despues**.

2. **Incompatibilidad tecnica:** Los repositorios usan frameworks mutuamente excluyentes (Actix-Web vs Substrate). Es fisicamente imposible copiar codigo entre ellos — no compilaria.

3. **Overlap de codigo despreciable:** De 215 lineas no triviales en `cerulean-dlt-framework`, solo 4 (1.8%) coinciden con `cerulean-ledger`. Las 4 son declaraciones de campos struct triviales del lenguaje Rust (`pub amount: u64,`, `pub timestamp: u64,`, `pub votes_for: u64,`, `pub votes_against: u64,`). No existe ninguna funcion, struct, import o logica compartida.

4. **Escala incomparable:** `cerulean-ledger` tiene 292 archivos Rust con 101,607 lineas de codigo, 458 commits, tests, CI/CD, documentacion, y un servidor desplegado en produccion. `cerulean-dlt-framework` tiene 8 archivos Rust con 706 lineas de scaffolding Substrate no compilable, creado en 7 commits en 69 minutos.

5. **Coincidencia tematica explicable:** Ambos proyectos tocan identidad digital, compliance ISO, tokenizacion RWA y gobernanza. Estos son temas de dominio publico ampliamente discutidos en la comunidad blockchain, especialmente en el contexto chileno (Camara de Blockchain de Chile, Ley 21.663, normativa BCN).

6. **La unica interaccion fue iniciada por Alefrank76:** El 14 de mayo de 2026, Alefrank76 agrego a clementeaf como colaborador de su repositorio. clementeaf nunca realizo ninguna accion en ese repo — cero commits, cero PRs, cero issues, cero comentarios. Todo el codigo del repositorio `cerulean-dlt-framework` pertenece exclusivamente a Alefrank76 segun el historial inmutable de git.

### 8.2 Veredicto final

**No existe evidencia de copia de codigo en ninguna direccion.** Los repositorios son proyectos independientes con implementaciones completamente distintas que comparten unicamente tematica de dominio publico.

---

## 9. Como verificar este informe

Cualquier tercero puede reproducir este analisis ejecutando el script automatizado:

```bash
# Descargar y ejecutar el script de verificacion
# (tambien disponible en scripts/verify-originality.sh dentro del repo cerulean-ledger)
curl -sL https://raw.githubusercontent.com/clementeaf/cerulean-ledger/main/scripts/verify-originality.sh | bash
```

El script realiza las siguientes verificaciones de forma automatica:

1. Clona ambos repositorios en un directorio temporal
2. Compara fechas del primer commit (precedencia temporal)
3. Cuenta commits, archivos y lineas de codigo en cada repo
4. Verifica incompatibilidad de frameworks (Actix-Web vs Substrate)
5. Compara dependencias (Cargo.toml)
6. Extrae lineas no triviales (≥15 chars, sin blanks/comments/braces/use/mod/atributos)
7. Calcula interseccion exacta y muestra cada coincidencia con su contexto en ambos repos
8. Verifica que `cerulean-dlt-framework` no compila
9. Genera un reporte con checksums SHA-256 de los repositorios analizados

**Resultado esperado:** 4 coincidencias triviales (`pub amount: u64,`, `pub timestamp: u64,`, `pub votes_for: u64,`, `pub votes_against: u64,`), todas campos struct genericos del lenguaje Rust.

---

*Informe generado el 28 de mayo de 2026 mediante analisis automatizado de repositorios publicos de GitHub. Actualizado el 30 de mayo de 2026 con datos verificados por comparacion linea a linea. Todos los datos son reproducibles por terceros usando el script `scripts/verify-originality.sh`.*
