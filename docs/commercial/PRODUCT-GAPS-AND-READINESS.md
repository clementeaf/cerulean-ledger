# Cerulean Ledger — Gaps y Estado de Preparación

**Fecha:** 2026-05-08
**Propósito:** Consolidar todos los gaps identificados entre el estado actual del producto y lo necesario para posicionamiento comercial, adopción enterprise, y compliance regulatorio en Chile.

---

## Resumen ejecutivo

Cerulean Ledger tiene el motor técnico completo: 1,427+ tests, criptografía post-cuántica, ejecución paralela, consenso multi-modal, y 4 verticales funcionales. Lo que separa al producto de la adopción real son gaps de **visibilidad, validación externa, y operación comercial** — no de tecnología.

| Categoría | Gaps críticos | Gaps importantes | Gaps menores | Cerrados |
|---|---|---|---|---|
| Go-to-market | 2 | ~~2~~ 0 | 1 | 2 (estrategia, one-pager) |
| Compliance / regulatorio | 0 | ~~3~~ 2 | 1 | 1 (gobernanza) |
| Técnico / producto | 1 | 2 | 2 | 1 (Polygon comparison) |
| Comercial / ventas | 1 | ~~3~~ 1 | 1 | 2 (SLA, matriz) |
| **Total** | **4** | **5 abiertos** | **5** | **6 cerrados** |

---

## Gaps críticos (bloquean posicionamiento)

### 1. No existe sandbox público

**Problema:** El producto solo es demostrable para quien lo instala localmente. No hay URL compartible. Esto bloquea cualquier demo remota, presentación a inversores, o evaluación por parte de un cliente.

**Qué se necesita:**
- VPS o instancia cloud con dominio (ej: sandbox.cerulean.cl)
- Block Explorer + Demo de credenciales + Cerulean Voto accesibles
- API docs interactivos (OpenAPI/Swagger)
- Rate limiting para proteger de abuso
- TLS con certificado válido

**Esfuerzo:** 1-2 semanas
**Impacto:** Sin esto, el producto no existe para nadie fuera del equipo.

---

### 2. No hay caso de éxito publicable

**Problema:** Ninguna organización ha usado Cerulean en producción (o piloto) de forma documentada. Sin un caso real, toda afirmación de valor es teórica.

**Qué se necesita:**
- Identificar una institución dispuesta a pilotear (municipio, universidad, cámara de comercio)
- Ejecutar piloto en una vertical (credenciales o votación)
- Documentar resultados con métricas concretas
- Obtener permiso para publicar el caso

**Esfuerzo:** 4-8 semanas (depende del cliente)
**Impacto:** Es la diferencia entre "proyecto técnico" y "producto validado".

---

### 3. No hay auditoría de seguridad independiente

**Problema:** Existe un self-audit completo (10/10 en security audit interno) y un pre-lab FIPS, pero ninguna firma externa ha validado la seguridad. Para enterprise y gobierno, esto es requisito.

**Qué se necesita:**
- Contratar pentest externo (firma reconocida)
- Publicar resumen de resultados (no el reporte completo)
- Remediar hallazgos críticos si los hay

**Esfuerzo:** 2-4 semanas + presupuesto ($5K-$20K USD según alcance)
**Impacto:** Sin esto, un CISO corporativo no aprueba la adopción.

---

### 4. Frontends no desplegados en URL pública

**Problema:** Las landing pages ya existen y son profesionales — `block-explorer-vite/` (hero + conceptos + comparativa vs Fabric/IOTA/Hedera) y `cerulean-voto/` (hero + 3 pilares de voto). Pero solo corren en localhost. Un cliente potencial no puede verlas.

**Qué se necesita:**
- Deploy de ambos frontends a URL pública (VPS, Vercel, Cloudflare Pages, etc.)
- Dominio propio (p. ej. cerulean.cl u otro TLD disponible)
- Backend API accesible para que el explorer y el demo funcionen en vivo

**Esfuerzo:** 2-3 días (infra, no código)
**Impacto:** Se fusiona con Gap 1 (sandbox). Resolver uno resuelve ambos.

---

## Gaps importantes (frenan aceleración)

### 5. ~~Resultados de stress test no publicados~~ CERRADO

**Resolución:** `docs/architecture/benchmarks/STRESS-TEST-RESULTS.md` — 4 niveles de ramp-up (500→5K creds, concurrencia 10→100). ~42 TPS end-to-end via HTTP, p50 14ms, 0 errores reales, 8,500 credenciales escritas. Rate limiter es el cuello de botella, no el nodo. Punto de quiebre no alcanzado.

---

### 6. ~~Integración con CSIRT / ANCI no implementada~~ CERRADO

**Resolución:** `src/events/webhook.rs` — `WebhookNotifier` se suscribe al `EventBus`, filtra eventos de seguridad (ACL denied, equivocation, rate limit, invalid signature, slashing), y los envía como JSON POST al endpoint configurado via `CSIRT_WEBHOOK_URL`. Soporte para `X-Webhook-Secret`, timeout configurable, backoff exponencial. 5 nuevas variantes en `BlockEvent`, 22 tests nuevos, 1438 total.

---

### 7. ~~Política de retención de datos no definida~~ CERRADO

**Resolución:** `RetentionPolicy` struct en `src/channel/config.rs` — configurable por canal via `ConfigUpdateType::SetRetention`. Tres controles: `block_retention_count`, `private_data_ttl_blocks`, `transaction_retention_secs`. Default: retención indefinida (0 = no purge). Backwards-compatible con JSON existente (`#[serde(default)]`). 7 tests nuevos, 1445 total.

---

### 8. ~~Documentación de gobernanza operacional ausente~~ CERRADO

**Resolución:** `docs/commercial/GOVERNANCE-OPERATIONAL.md` — roles, gestión de incidentes, continuidad, control de acceso, monitoreo, mapeo Ley 21.663.

---

### 9. ~~No hay SLA documentado~~ CERRADO

**Resolución:** `docs/commercial/SLA.md` — 3 tiers (SaaS 99.5%, on-premise, consorcio 99.9%), compensaciones, métricas, exclusiones.

---

### 10. No hay pricing público

**Problema:** Los docs comerciales existentes mencionan rangos ($49-$499/mes), pero no hay pricing oficial publicado ni estructura clara por vertical.

**Acción:** Definir pricing por vertical y modelo (SaaS / on-premise / consorcio).
**Esfuerzo:** 1 semana (decisión de negocio, no técnica)

---

### 11. ~~Comparación vs Polygon no documentada~~ CERRADO

**Resolución:** `docs/architecture/comparisons/POLYGON-COMPARISON.md` — categorías distintas, ventajas mutuas, tabla de cuándo elegir cada uno.

---

### 12. ~~Estrategia de adopción no documentada~~ CERRADO

**Resolución:** `docs/commercial/GO-TO-MARKET-STRATEGY.md` — playbook 5 pasos, vocabulario comercial, canal por vertical, certificaciones como moat, riesgos y mitigaciones.

---

### 13. ~~Explicación simple del producto no documentada~~ CERRADO

**Resolución:** `docs/commercial/ONE-PAGER-PRODUCTO.md` — sin jerga, problema/solución, 4 verticales, números, siguiente paso.

---

### 14. ~~Matriz vertical × horizontal no documentada~~ CERRADO

**Resolución:** `docs/commercial/VERTICAL-HORIZONTAL-MATRIX.md` — matriz de consumo, priorización de verticales, leverage de plataforma.

---

## Gaps menores (mejoran pero no bloquean)

### 15. No hay gRPC

**Problema:** Toda comunicación es HTTP/JSON. Incompatible con SDKs nativos de Fabric si se quisiera interop.
**Impacto:** Solo relevante si se busca interoperabilidad con redes Fabric existentes. Como producto standalone, no bloquea.
**Esfuerzo:** Alto

---

### 16. No hay Fabric CA

**Problema:** No existe servicio de enrollment/registration de identidades. Certificados se gestionan externamente.
**Impacto:** Medio. Resoluble con integración a CA existente del cliente.
**Esfuerzo:** Alto

---

### 17. Service discovery manual

**Problema:** El discovery de peers es por registro manual, no gossip-based automático.
**Impacto:** Bajo para redes pequeñas (3-10 nodos). Relevante si la red crece.
**Esfuerzo:** Medio

---

### 18. FIPS 140-3 no certificado aún

**Problema:** El pre-lab está hecho, el módulo crypto cumple, pero la certificación formal no está completa (12-24 meses, $80K-$250K).
**Impacto:** Solo bloquea si el cliente exige certificación formal (gobierno USA, defensa).
**Esfuerzo:** Alto (tiempo + presupuesto)

---

### 19. No hay soporte 24/7

**Problema:** No existe estructura de soporte para producción.
**Impacto:** Resoluble post-primer-cliente. No bloquea piloto.
**Esfuerzo:** Operacional, no técnico

---

## Ruta de cierre priorizada

### Semana 1-2 (desbloqueantes)

| # | Gap | Acción |
|---|---|---|
| 1 | Sandbox público | Deploy a VPS + dominio + TLS |
| 4 | Landing page | Página simple con link al sandbox |
| 5 | Stress test | Correr, capturar, publicar resultados |

### Semana 3-4 (compliance + comercial)

| # | Gap | Acción |
|---|---|---|
| 6 | CSIRT webhook | Implementar notificación configurable |
| ~~8~~ | ~~Gobernanza operacional~~ | ~~CERRADO~~ |
| ~~9~~ | ~~SLA~~ | ~~CERRADO~~ |
| 10 | Pricing | Estructurar por vertical |
| ~~11-14~~ | ~~Docs faltantes~~ | ~~CERRADOS (Polygon, adopción, one-pager, matriz)~~ |

### Mes 2-3 (validación)

| # | Gap | Acción |
|---|---|---|
| 2 | Caso de éxito | Ejecutar piloto con institución ancla |
| 3 | Auditoría externa | Contratar pentest, publicar resumen |
| 7 | Retención de datos | Política + TTL configurable |

### Mediano plazo (6+ meses)

| # | Gap | Acción |
|---|---|---|
| 18 | FIPS 140-3 | Iniciar proceso formal con laboratorio |
| 15-17 | gRPC, Fabric CA, auto-discovery | Solo si el mercado lo demanda |
| 19 | Soporte 24/7 | Estructurar post-revenue |

---

## Gaps identificados por auditoría Cámara Blockchain Chile (2026-05-08)

### 20. ~~Oráculo sin conectores a fuentes externas~~ CERRADO

**Resolución:** `src/oracle_connector.rs` — `DataSource` trait, HTTP connector con JSON path extraction, multi-source aggregation (N-of-M consensus), spread validation, background poller con `spawn_oracle_poller()`. Env vars: `ORACLE_SOURCES`, `ORACLE_MIN_SOURCES`, `ORACLE_POLL_INTERVAL_SECS`. 11 tests.

---

### 21. ~~Sandbox forense ausente~~ CERRADO

**Resolución:** `src/forensic.rs` — `ForensicEngine` con: `build_timeline()` (audit + block events correlados y ordenados), `security_timeline()` (solo Warning/Critical), `correlate_security_events()` (vincula requests fallidos con eventos de seguridad), `build_evidence_package()` (paquete firmable con SHA-256 content hash, cadena de custodia), `severity_summary()`. Tipos: `TimelineEntry`, `Severity`, `ReplayResult`, `EvidencePackage`. 10 tests.

---

### 22. ~~ISO compliance para RWA y finanzas~~ CERRADO (fase 1)

**Resolución:** `src/compliance/` — módulo con 3 sub-módulos:
- `iso20022.rs`: tipos `Pacs008`, `Pain001`, `Camt053` con validadores (`validate_pacs008`, `validate_pain001`, `validate_camt053`). Valida BIC, IBAN, moneda, país, montos.
- `iso3166.rs`: 27 códigos de país LATAM + principales, `is_valid_country()`, `country_name()`.
- `iso4217.rs`: 15 monedas con decimales, `is_valid_currency()`, `format_amount()`.
25 tests. Pendiente fase 2: ERC-3643 security tokens.

---

## Conclusión

El core de la plataforma fue validado por la Cámara Blockchain Chile como "flagship que supera expectativas" en las 4 fases de auditoría (motor, transaccional, contratos, seguridad). Los gaps restantes son de **conectividad con el mundo exterior** (oráculo), **herramientas de auditoría avanzada** (forense), y **compliance financiero** (ISOs). Resolverlos posiciona a Cerulean para entrada en banca y gobierno a nivel nacional.

---

*Documento generado: 2026-05-08. Actualizado con feedback de Cámara Blockchain Chile.*
