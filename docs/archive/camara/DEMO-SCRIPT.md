# Cerulean Ledger — Guion de Demo (5 minutos)

**Preparado para:** Presentacion ante la Camara de Blockchain Chile
**Requisitos previos:** Docker instalado, repo clonado, `npm install` en `block-explorer-vite/`

---

## Antes de la presentacion

```bash
# Levantar la red (4 minutos, hacer ANTES de entrar a la sala)
docker compose up -d

# Verificar que los nodos estan sanos
./scripts/bcctl.sh status
```

Tener abiertos en pestanas del navegador:
1. Block Explorer: `http://localhost:5173`
2. Terminal con el repo abierto

---

## Minuto 0:00 — Apertura (30 seg)

**Decir:** "Les voy a mostrar Cerulean Ledger funcionando. Una red de 6 nodos blockchain con criptografia post-cuantica, levantada con un solo comando."

**Mostrar:** Terminal con `docker compose ps` — los 6 nodos + Prometheus + Grafana corriendo.

---

## Minuto 0:30 — La red en vivo (1 min)

**Mostrar:** Block Explorer → Dashboard (`/dashboard`)

- Senalar: bloques, peers conectados, estadisticas de red
- "Esto es una red permisionada real con 3 peers, 3 orderers, TLS mutuo, todo en Rust."

**Ejecutar en terminal:**

```bash
# Crear una wallet y minar un bloque
./scripts/bcctl.sh mine
```

**Mostrar:** El bloque nuevo aparece en el dashboard.

- "Ese bloque fue firmado con Ed25519. Pero miren esto..."

---

## Minuto 1:30 — Criptografia post-cuantica (1 min)

**Mostrar:** Terminal

```bash
# El nodo soporta ML-DSA-65 (FIPS 204)
# Basta cambiar una variable de entorno:
echo "SIGNING_ALGORITHM=ml-dsa-65"
```

**Decir:**
- "Cerulean Ledger es la primera DLT empresarial permisionada con firmas ML-DSA-65 (FIPS 204) integradas end-to-end."
- "Fabric no lo tiene. Corda no lo tiene. IOTA no lo tiene. Nosotros si, y fue desarrollado en Chile."
- "Cada nodo elige su algoritmo. Redes mixtas clasica/PQC coexisten. Migracion gradual, sin flag day."

---

## Minuto 2:30 — Demo RRHH: credenciales verificables (1.5 min)

**Mostrar:** Block Explorer → Demo RRHH (`/demo`)

Ejecutar los 5 pasos en vivo:

1. **Registrar emisor** — "La universidad se registra como emisor de credenciales"
2. **Registrar candidato** — "El candidato obtiene su identidad digital (DID)"
3. **Emitir credencial** — "La universidad emite el titulo como credencial verificable"
4. **Verificar credencial** — "RRHH verifica en segundos, sin llamar a la universidad"
5. **Perfil completo** — "Todo el historial del candidato, verificable e inmutable"

**Decir:** "Esto reemplaza llamadas telefonicas, PDFs falsificables y semanas de espera. Verificacion instantanea, criptograficamente segura, con audit trail."

---

## Minuto 4:00 — Tesseract: el futuro (30 seg)

**Mostrar:** Block Explorer → Tesseract (`/tesseract`)

- Senalar la simulacion interactiva del campo de probabilidad
- "Esto es Tesseract: un prototipo de consenso basado en geometria 4D, no en computacion."
- "Es investigacion en curso. La seguridad emerge de la convergencia geometrica, no de asumir que la mayoria es honesta."
- No profundizar — dejar como teaser para preguntas.

---

## Minuto 4:30 — Cierre (30 seg)

**Decir:**
- "Lo que acaban de ver: red empresarial de 6 nodos, credenciales verificables, criptografia post-cuantica FIPS 204, todo en un binario de 50 MB de RAM."
- "Pedi a la Camara: visibilidad, pilotos con empresas miembro, y puente con reguladores."
- "Todo es open source. Pueden auditarlo hoy."

---

## Preguntas frecuentes durante el demo

| Pregunta | Respuesta corta |
|---|---|
| "Cuantos TPS?" | 56K medidos con Criterion (wave-parallel scheduling) |
| "Y si se cae un nodo?" | Raft con crash recovery. Los demas siguen. |
| "Que pasa con datos privados?" | Private data collections con ACL por org y TTL |
| "Funciona con Solidity?" | Si, compatibilidad EVM completa via revm |
| "Cuanto cuesta?" | Open source, sin licencia. Soporte directo para pilotos. |
| "Que es Tesseract?" | Prototipo de consenso geometrico. Investigacion activa. Ver docs/TESSERACT.md |

---

## Plan B: sin Docker

Si Docker falla, usar el demo interactivo local:

```bash
./scripts/try-it.sh
```

Esto levanta un nodo local, crea wallets, mina bloques y ejecuta transacciones — todo desde terminal. Menos visual pero demuestra la funcionalidad core.

---

## Checklist pre-demo

- [ ] `docker compose up -d` exitoso (todos los contenedores healthy)
- [ ] Block Explorer corriendo en `localhost:5173`
- [ ] `./scripts/bcctl.sh status` muestra 6 nodos
- [ ] Pestana de Tesseract cargada
- [ ] Pestana de Demo RRHH cargada
- [ ] Terminal visible para comandos en vivo
- [ ] Conexion a internet NO requerida (todo es local)
