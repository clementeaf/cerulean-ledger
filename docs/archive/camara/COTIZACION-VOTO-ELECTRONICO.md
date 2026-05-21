# Cotizacion — Sistema de Voto Electronico sobre Cerulean Ledger

**Fecha:** 24 de abril de 2026
**Vigencia:** 30 dias corridos desde la fecha de emision
**Moneda:** Pesos chilenos (CLP), valores netos (liquidos)

---

## 1. Resumen ejecutivo

Propuesta de desarrollo de un sistema de voto electronico construido sobre la plataforma blockchain **Cerulean Ledger**, una DLT propia con consenso BFT, identidad descentralizada (DIDs), credenciales verificables y firmas post-cuanticas (ML-DSA-65, FIPS 204).

El sistema se entrega en tres fases incrementales, cada una funcional de forma independiente. El cliente puede optar por contratar solo la Fase 1 y evaluar antes de continuar.

---

## 2. Alcance por fase

### Fase 1 — MVP: Voto abierto para asambleas y juntas (Mes 1)

Ideal para juntas de accionistas, asambleas gremiales, votaciones de directorio o cualquier escenario donde el voto no requiera ser secreto.

**Entregables:**

| # | Componente | Descripcion |
|---|---|---|
| 1 | Padron electoral digital | Registro de votantes via credenciales verificables (1 DID = 1 voto). Verificacion de elegibilidad automatica. |
| 2 | Ciclo de vida de eleccion | Crear eleccion, definir opciones, abrir/cerrar votacion, escrutinio automatico, publicacion de resultados. |
| 3 | API REST completa | Endpoints para gestion de elecciones, emision de votos, consulta de resultados y auditoria. |
| 4 | Interfaz web de votacion | Aplicacion React para que votantes emitan su voto y vean resultados en tiempo real. |
| 5 | Panel de administracion | Crear elecciones, gestionar padron, monitorear participacion. |
| 6 | Registro inmutable | Cada voto queda registrado en la cadena con consenso BFT (no se puede alterar ni eliminar). |
| 7 | Documentacion | Manual de uso, guia de despliegue, documentacion de API. |

**Infraestructura reutilizada (sin costo adicional):**
- Modulo de gobernanza existente (voting, proposals, tally)
- Sistema de identidad con DIDs y credenciales verificables
- Almacenamiento persistente (RocksDB)
- Consenso BFT (tolerancia a fallas bizantinas)
- Rate limiting y control de acceso

**Limitaciones de esta fase:**
- El voto es abierto (visible quien voto que) — apropiado para asambleas pero no para elecciones con voto secreto.

---

### Fase 2 — Voto secreto con verificabilidad (Mes 2)

Agrega privacidad al voto sin sacrificar la auditabilidad del resultado.

**Entregables:**

| # | Componente | Descripcion |
|---|---|---|
| 1 | Esquema commit-reveal | El votante compromete su voto cifrado; solo se revela al cierre de la eleccion. Nadie puede ver votos antes del escrutinio. |
| 2 | Aislamiento por eleccion | Cada eleccion opera en un canal independiente (privacidad total entre elecciones). |
| 3 | Verificacion individual | Cada votante puede verificar que su voto fue contado correctamente, sin revelar su eleccion a terceros. |
| 4 | Auditoria publica | Cualquier observador puede validar el escrutinio total sin acceder a votos individuales. |
| 5 | Pruebas de integridad | Suite de tests automatizados para escenarios adversariales (doble voto, manipulacion, revelacion anticipada). |

---

### Fase 3 — Produccion y acceso movil (Mes 3)

Preparacion para despliegue en entorno productivo con acceso desde dispositivos moviles.

**Entregables:**

| # | Componente | Descripcion |
|---|---|---|
| 1 | Cliente liviano movil | Votacion desde celular sin necesidad de nodo completo (via light client). |
| 2 | Firma post-cuantica | Activacion de ML-DSA-65 (FIPS 204) en produccion — proteccion contra amenazas de computacion cuantica. |
| 3 | Hardening de seguridad | Anti-DoS reforzado durante ventana electoral, monitoreo, alertas. |
| 4 | Despliegue Docker | Infraestructura containerizada lista para nube (AWS, GCP, Azure o on-premise). |
| 5 | Capacitacion | Sesion de capacitacion para administradores del sistema. |
| 6 | Soporte post-lanzamiento | 2 semanas de soporte incluido despues del despliegue. |

---

## 3. Cronograma y costos

| Fase | Duracion | Costo neto (CLP) |
|---|---|---|
| Fase 1 — MVP voto abierto | 4 semanas | $1.500.000 |
| Fase 2 — Voto secreto | 4 semanas | $1.400.000 |
| Fase 3 — Produccion y movil | 4 semanas | $1.200.000 |
| **Total proyecto completo** | **12 semanas** | **$4.100.000** |

**Condiciones de pago:**
- Pago mensual al inicio de cada fase.
- Cada fase es independiente: el cliente puede pausar o no continuar despues de cualquier fase.
- Si se contrata el proyecto completo por adelantado: **$3.800.000** (descuento de $300.000).

---

## 4. Mantencion y soporte (opcional, post-proyecto)

| Plan | Incluye | Costo mensual (CLP) |
|---|---|---|
| Basico | Correccion de bugs criticos, actualizaciones de seguridad, soporte por email (48h respuesta) | $250.000 |
| Estandar | Basico + mejoras menores, soporte prioritario (24h), monitoreo de nodos | $450.000 |

---

## 5. Diferenciadores tecnicos

| Caracteristica | Cerulean Ledger | Soluciones convencionales |
|---|---|---|
| Firmas post-cuanticas (ML-DSA-65) | Si, FIPS 204 | No |
| Consenso BFT (tolerancia a fallas bizantinas) | Si, HotStuff-inspired | Generalmente no |
| Identidad descentralizada (DIDs) | Nativo | Requiere integracion externa |
| Credenciales verificables | Nativo | Requiere integracion externa |
| Verificabilidad publica del escrutinio | Si | Parcial o nula |
| Aislamiento por eleccion (canales) | Si | No |
| Codigo abierto / auditable | Si | Raramente |
| Rendimiento comprobado | 56K TPS (benchmark) | Variable |

---

## 6. Requisitos del cliente

- Designar un punto de contacto para validaciones semanales.
- Proveer el padron de votantes en formato digital (CSV, Excel o API).
- Definir las reglas de cada eleccion (quorum, opciones, duracion).
- Infraestructura de hosting (o contratar hosting como servicio adicional).

---

## 7. Exclusiones

- Desarrollo de apps nativas iOS/Android (la solucion movil es web responsive via light client).
- Integracion con sistemas de identidad gubernamentales (ClaveUnica, SII) — cotizable como adenda.
- Hosting e infraestructura cloud (cotizable por separado segun proveedor).
- Certificaciones formales ante organismos electorales (requiere proceso independiente).

---

## 8. Garantias

- Todo el codigo fuente se entrega al cliente.
- Suite de tests automatizados con cobertura superior al 80%.
- Documentacion tecnica y de usuario incluida.
- 2 semanas de soporte post-lanzamiento incluidas en Fase 3.

---

## 9. Contacto

**Cerulean Ledger**
Contacto: [nombre]
Email: [email]
Telefono: [telefono]

---

*Este documento es una cotizacion referencial. Los plazos y costos pueden ajustarse segun los requisitos especificos del cliente una vez definido el alcance detallado.*
