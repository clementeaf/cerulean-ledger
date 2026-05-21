# Polygon Comparison

Cómo Cerulean Ledger se diferencia de Polygon. Last updated: 2026-05-08.

---

## Son categorías distintas

| Dimensión | Polygon | Cerulean Ledger |
|---|---|---|
| Tipo | L2 público sobre Ethereum | L1 permisionado standalone |
| Modelo de acceso | Abierto, permissionless | Cerrado, permissioned (MSP + ACL) |
| Finalidad | Escalar Ethereum (fees bajos, throughput) | Infraestructura enterprise soberana |
| Usuarios típicos | DeFi, NFTs, gaming, dApps públicas | Gobierno, banca, supply chain, voto |
| Token | MATIC/POL (público, tradeable) | NOTA (interno, supply cap 100M, no público) |
| Visibilidad de datos | Todo público en el explorer | Aislamiento por organización, deny-by-default |

Compararlos directamente es como comparar un servicio cloud público con un datacenter privado de un banco. Ambos procesan datos, pero sirven propósitos fundamentalmente distintos.

---

## Donde Polygon es superior

| Capacidad | Detalle |
|---|---|
| Ecosistema | Miles de dApps, billones en TVL, millones de usuarios |
| zkEVM | Zero-knowledge proofs para validación matemáticamente verificable |
| Interop Ethereum | Bridge nativo, hereda seguridad de Ethereum L1 |
| Tooling | Hardhat, Foundry, Etherscan, The Graph, OpenZeppelin |
| Liquidez | Token en todos los exchanges, DeFi nativo |
| Battle-testing | Millones de transacciones diarias en producción desde 2021 |

---

## Donde Cerulean tiene ventajas reales

### 1. Privacidad y control de acceso

Polygon es público — todo es visible en el explorer. Cerulean ofrece:

- Canales aislados (ledgers separados por organización)
- Private data collections con ACL y TTL purge
- mTLS + MSP roles (admin/peer/client)
- `enforce_acl()` deny-by-default

Para banca o gobierno, privacidad no es opcional — es requisito legal.

### 2. Criptografía post-cuántica

Polygon usa ECDSA secp256k1 (misma curva que Bitcoin, 2009). Vulnerable a computación cuántica.

Cerulean implementa:
- ML-DSA-65 (FIPS 204, 2024) — firmas digitales
- SHA3-256 (FIPS 202) — hash
- ML-KEM-768 (FIPS 203) — intercambio de claves
- Dual-signing para migración gradual
- Módulo cripto con camino FIPS 140-3

Polygon necesitaría que todo Ethereum migre para cambiar su criptografía. Cerulean ya está ahí.

### 3. Soberanía operacional

| Polygon | Cerulean |
|---|---|
| Dependes de Ethereum L1 | No dependes de nadie |
| Sequencer centralizado (en transición) | Raft/BFT distribuido entre tus nodos |
| Gas en MATIC (precio de mercado) | Sin token público, costos controlados |
| Regulador ve "cripto pública" | Regulador ve "infraestructura privada" |

Para un gobierno latinoamericano o un banco regulado, "correr sobre Polygon" es una conversación regulatoria difícil. "Correr tu propia infraestructura" es aceptable.

### 4. Consenso flexible

Polygon PoS tiene un set fijo de validadores. Cerulean ofrece Raft (crash-fault), BFT (byzantine-fault), DAG, o DPoS, seleccionable por variable de entorno según el caso de uso.

### 5. Dual runtime de smart contracts

Cerulean ofrece Wasm chaincode (estilo Fabric) Y EVM via revm. Polygon solo tiene EVM. Para lógica empresarial compleja, Wasm es más expresivo y portable.

---

## Cuándo elegir cada uno

| Escenario | Elección |
|---|---|
| DeFi, NFTs, dApp pública | **Polygon** |
| Voto electrónico gubernamental | **Cerulean** |
| Supply chain entre empresas | **Cerulean** |
| Gaming con miles de usuarios | **Polygon** |
| Banco que necesita audit trail | **Cerulean** |
| Identidad digital soberana | **Cerulean** |
| Interoperabilidad con Ethereum | **Polygon** |
| Compliance Ley 21.663 Chile | **Cerulean** |

---

## Veredicto

No son competidores — atienden mercados distintos. Polygon resuelve escalabilidad pública. Cerulean resuelve infraestructura enterprise soberana con criptografía post-cuántica y privacidad por diseño.

Donde sí hay superioridad técnica puntual de Cerulean: criptografía post-cuántica y privacidad nativa. Polygon no tiene ninguna de las dos y no las tendrá sin cambios fundamentales en Ethereum.

---

## Test coverage summary

| Category | Count |
|---|---|
| Unit + integration tests | 1,427+ |
| E2E tests (Docker network) | 71 |
| CI status | All green |
