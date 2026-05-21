# Cerulean Ledger — Talking Points para la Cámara de Blockchain Chile

**Preparado para:** Presentación ante directiva — 24 de abril de 2026

---

## Apertura (30 segundos)

> Cerulean Ledger es una plataforma DLT empresarial desarrollada en Chile, con ~95% de paridad funcional con Hyperledger Fabric, y la primera DLT empresarial permisionada con firmas ML-DSA-65 (FIPS 204) integradas end-to-end. Ninguna plataforma comparable — ni Fabric, ni Corda, ni IOTA — ofrece esta capacidad hoy. Y fue construida aquí.

---

## Pitch para la directiva (3 minutos)

> Las empresas chilenas que necesitan registro distribuido entre múltiples organizaciones hoy dependen de Hyperledger Fabric — una plataforma estadounidense compleja, que requiere equipos especializados en Go/Java, y que no tiene protección contra la amenaza cuántica.
>
> Cerulean Ledger resuelve ambos problemas. Es una DLT permisionada que implementa el mismo modelo de Fabric — channels, private data, smart contracts, endorsement policies, Raft — con ~95% de paridad verificada. Pero con tres diferencias fundamentales:
>
> **Primero, criptografía post-cuántica.** Implementamos ML-DSA-65 (FIPS 204, NIST security level 3) en toda la stack: bloques, transacciones, endorsements, identidades. NIST publicó el estándar en agosto 2024. La NSA exige migración para 2030. La UE apunta en la misma dirección con eIDAS 2.0. Somos los primeros en DLT empresarial en implementarlo. Esto posiciona a Chile como referente.
>
> **Segundo, simplicidad.** Un nodo usa 50 MB de RAM. Una red de 6 nodos se levanta con `docker compose up` en 4 minutos. Sin Java, sin Go, sin Certificate Authorities complejas. Esto baja drásticamente la barrera de entrada para empresas chilenas.
>
> **Tercero, casos chilenos.** Tenemos casos documentados y demostrables: trazabilidad agroexportadora (SAG → Aduana → importador), documentos verificables para el sector público, historial médico transfronterizo, conciliación financiera con CMF como observador, y un demo funcional de credenciales laborales en el block explorer.
>
> Todo respaldado por 2,741 tests, auditoría de seguridad completa, compliance SOC 2 e ISO 27001 mapeados, y documentación de 68 endpoints API.
>
> Lo que pedimos a la Cámara: visibilidad, conexión con empresas miembro para pilotos, y puente con reguladores. Lo que ofrecemos: tecnología open source sin costo de licencia, soporte directo, y la oportunidad de que Chile lidere en DLT post-cuántica antes de que las grandes plataformas alcancen.

---

## Pitch técnico ampliado (si hay preguntas de profundidad)

> Además de la paridad con Fabric, Cerulean Ledger va más allá en varias capacidades:
>
> - **Consenso BFT** — No solo Raft (crash faults). Tenemos un protocolo HotStuff-inspired con tolerancia a fallas bizantinas, validado con 16 tests adversarios (equivocación, particiones de red, crash faults simultáneos).
> - **Ejecución paralela** — Wave scheduling con detección de conflictos RAW/WAW/WAR. 56K TPS medidos con Criterion.
> - **Compatibilidad EVM** — Deploy y ejecución de contratos Solidity via revm, además de Wasm nativo.
> - **Bridge cross-chain** — Framework con escrow, pruebas Merkle, protección anti-replay. Infraestructura lista para conectar con otras redes.
> - **Governance on-chain** — Propuestas, votación ponderada por stake, timelock antes de ejecución.
> - **Light client** — Verificación de estados para dispositivos IoT o móviles sin nodo completo.

---

## One-liners según quién pregunte

| Perfil en la sala | Mensaje clave |
|---|---|
| **Presidente / Directiva** | "Chile tiene la oportunidad de liderar en DLT post-cuántica. Esta plataforma es la evidencia técnica de que podemos." |
| **Miembro técnico** | "95% paridad Fabric, consenso BFT + Raft seleccionable, 56K TPS, firmas FIPS 204 — todo en un binario de 50 MB." |
| **Miembro de empresa financiera** | "Conciliación interinstitucional con la CMF como nodo observador. Audit trail inmutable exportable. Sin gas fees." |
| **Miembro de agroexportación** | "Trazabilidad completa de la parcela al importador. Certificaciones SAG como credenciales verificables, no PDFs falsificables." |
| **Miembro de sector público** | "Títulos, certificados, licencias verificables en segundos. Sin llamar a la institución emisora. Revocación automática." |
| **Preocupado por regulación** | "FIPS 204, SOC 2 mapeado, ISO 27001 mapeado, audit trail para CMF/SII. La Ley 19.628 se cumple con private data collections + TTL." |

---

## Datos clave para memorizar

| Dato | Número |
|---|---|
| Origen | Chile |
| Paridad con Fabric | ~95% (34 capacidades verificadas) |
| Tests | 2,741 passing, 0 failed |
| TPS (ejecución paralela) | 56,000 (independientes), 39,000 (mixtas) |
| Ordering throughput | 23 millones tx/s (in-memory) |
| RAM por nodo | ~50 MB (Fabric: ~500 MB+) |
| Endpoints API | 68 documentados |
| Startup | ~2 segundos |
| Auditoría seguridad | 10/10 remediados |
| PQC estándar | ML-DSA-65 (FIPS 204, NIST level 3) |
| Firma PQC tamaño | 3,309 bytes |
| BFT tolerancia | f = (n-1)/3 byzantine faults |
| Docker deployment | 6 nodos + Prometheus + Grafana |
| Licencia | MIT (open source) |
| Competidores con PQC | 0 (Fabric, Corda, IOTA: ninguno) |

---

## Objeciones probables y respuestas

### "Las empresas ya usan Fabric. Por qué cambiarían?"

No proponemos reemplazar Fabric donde ya funciona. Proponemos que los nuevos proyectos DLT en Chile — especialmente los que manejan datos de largo plazo (propiedad, salud, contratos) — empiecen con protección cuántica desde el día uno. Migrar después es más caro que empezar bien. Y para empresas que aún no usan DLT, la barrera de entrada es 10x menor.

### "La computación cuántica está lejos."

Los datos firmados hoy pueden ser interceptados y almacenados para descifrarlos después ("harvest now, decrypt later"). Para Chile, esto importa en agroexportación (certificados SAG válidos por años), sector público (títulos, registros de propiedad), y finanzas (contratos de largo plazo). NIST, la NSA (CNSS Policy 15), y la UE (eIDAS 2.0) ya exigen migración. Chile exporta a la UE — la alineación regulatoria es ventaja competitiva.

### "Es un proyecto pequeño."

El código habla: 2,741 tests, 68 endpoints, 22 tests de penetración, auditoría de seguridad completa, compliance SOC 2/ISO 27001 mapeado, block explorer funcional, documentación exhaustiva. La productividad de Rust + tooling moderno permite resultados de enterprise-grade. Y al ser un solo punto de contacto técnico, las decisiones son rápidas — ideal para pilotos.

### "No tiene comunidad como Fabric."

Correcto, es más joven. Pero para un piloto enterprise, un equipo que conoce cada línea de código y responde en horas es más predecible que depender del roadmap de la Linux Foundation. La Cámara puede ayudar a construir esa comunidad en Chile.

### "Qué pasa si el proyecto se abandona?"

Open source (MIT). Cualquier organización puede fork, auditar y mantener. Sin dependencia de servicios cloud propietarios. La documentación completa (CLAUDE.md, API reference, deployment guide, security audit) permite continuidad total.

### "Los reguladores chilenos lo aceptarían?"

La plataforma ya tiene:
- Audit trail inmutable con trazabilidad por organización (requisito CMF/SII)
- Compliance SOC 2 (13 criterios mapeados) e ISO 27001 (17 controles)
- FIPS 204 (estándar NIST), FIPS 140-3 (self-tests KAT en startup)
- Private data collections con TTL (Ley 19.628 de protección de datos)
- Export CSV para entrega a reguladores

La Cámara puede facilitar las conversaciones con CMF/SII para validar el enfoque.

### "Por qué Rust y no Go como Fabric?"

Rust ofrece memory safety sin garbage collector, lo que elimina una clase entera de vulnerabilidades (buffer overflows, use-after-free) y da rendimiento predecible. Un nodo usa ~50 MB vs ~500 MB+ de Fabric. Para un mercado como Chile donde el costo de infraestructura importa, esto es significativo.

---

## Flujo recomendado para demo en la Cámara

Si hay oportunidad de demo en vivo o en un evento futuro:

1. **Setup** (2 min) — `docker compose up`, mostrar 6 nodos healthy con monitoreo
2. **Block explorer** (2 min) — Dashboard de red, estadísticas, UI en español
3. **Demo credenciales RRHH** (5 min) — Flujo de 5 pasos: registrar empresa → registrar candidato → emitir credencial → verificar → perfil completo. Este es el caso más visual y entendible.
4. **Caso agroexportación** (3 min) — Explicar el flujo SAG→Aduana→importador con credenciales verificables
5. **PQC** (2 min) — Mostrar firma de 3,309 bytes vs 64 bytes clásica, explicar por qué importa para datos de largo plazo
6. **Números** (1 min) — 2,741 tests, 56K TPS, 50 MB RAM, auditoría completa

**Tiempo total:** ~15 minutos incluyendo preguntas.

---

## Mensaje de cierre sugerido

> Chile tiene una ventana de oportunidad para liderar en DLT empresarial post-cuántica. Ningún otro país tiene una plataforma equivalente en producción hoy. Cerulean Ledger es la evidencia técnica de que podemos. Lo que necesitamos de la Cámara es el puente con la industria y los reguladores para convertir esta ventaja técnica en adopción real.

---

*Cerulean Ledger — DLT empresarial. Desarrollada en Chile. Segura ante computación cuántica.*
