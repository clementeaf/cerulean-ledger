# Tesseract — Consenso por Convergencia Geometrica

**Estado:** Prototipo conceptual / investigacion activa
**Relacion con Cerulean Ledger:** Modulo experimental, no activo en produccion

---

## En una frase

Tesseract es un modelo de consenso donde la verdad emerge porque la geometria converge, no porque alguien la verifique.

---

## El problema que aborda

Todos los mecanismos de consenso existentes dependen de supuestos computacionales o de confianza:

| Mecanismo | Supuesto |
|---|---|
| Proof of Work | La mayoria del hashrate es honesta |
| BFT (PBFT, HotStuff) | Menos de 1/3 de los nodos son bizantinos |
| Proof of Stake | La mayoria del stake es honesto |
| DAGs (IOTA, Narwhal) | Tiempo como dimension privilegiada + validacion computacional |

Tesseract elimina estos supuestos. No asume mayoria honesta, no usa validacion computacional, y no depende de primitivas criptograficas que un computador cuantico pueda romper.

---

## Como funciona

### Campo de probabilidad 4D

Los datos no se almacenan — emergen. Cada evento genera una distribucion de probabilidad en un espacio toroidal de 4 dimensiones simetricas:

- **Temporal** — cuando ocurrio
- **Contexto** — que tipo de evento es
- **Organizacion** — quien lo origino
- **Version** — iteracion del dato

Ninguna dimension es privilegiada. La probabilidad converge hacia un punto fijo unico.

### Cristalizacion

Cuando la convergencia supera un umbral (sigma >= 0.85), el estado cristaliza — se vuelve permanente sin que nadie lo decida explicitamente. Es el primer mecanismo formal de estado emergente en sistemas distribuidos.

### Auto-sanacion

Destruir un dato no lo elimina. El campo regenera las celdas destruidas desde la geometria de sus vecinos ortogonales. Con soporte ortogonal sigma = 4, la recuperacion toma 2-5 pasos. El atacante necesita destruir el campo completo (O(S^4) celdas) para tener efecto.

### Rechazo de falsedad

Un dato falso inyectado no se propaga. Sin soporte orbital (sigma <= 1), no genera resonancia y queda aislado geometricamente. 7 tipos de ataque adversarial probados: Sybil, eclipse, timing, cuantico, entre otros.

---

## Las 4 leyes fisicas

Tesseract mapea propiedades criptograficas a leyes fundamentales de la fisica:

### 1. Causalidad — "Nada viaja mas rapido que la luz"

Como los conos de luz en relatividad: un evento solo puede influir lo que podria haber alcanzado. El orden causal se deriva del grafo de dependencias (SHA-256), no de un reloj global. Reordenar la causalidad requiere invertir SHA-256.

### 2. Conservacion — "La energia no se crea ni se destruye"

La cantidad total en el campo es invariante. Compromisos Pedersen sobre Curve25519 garantizan que sum(inputs) == sum(outputs) sin revelar valores. El doble gasto es una imposibilidad fisica, no una violacion detectada.

### 3. Entropia — "La flecha del tiempo solo apunta en una direccion"

El sistema cristaliza cuando es energeticamente favorable. Cada cristalizacion produce un sello: S(n) = SHA-256(S(n-1) || evidencia). Revertir cuesta energia proporcional a binding_energy x edad.

### 4. Gravedad — "La masa curva el espacio"

La masa ES el conteo de eventos causales de un participante. Sin registro que hackear, sin balance que forjar. La influencia decae con el cuadrado de la distancia, previniendo monopolio.

---

## Comparativa

### vs Bitcoin

| Aspecto | Bitcoin | Tesseract |
|---|---|---|
| Seguridad | Computacional (hashrate) | Geometrica (convergencia) |
| Finalidad | Probabilistica (nunca 100%) | Determinista (punto fijo) |
| Costo de ataque | O(hashrate) — lineal | O(S^4) — exponencial |
| Auto-sanacion | Requiere nodos backup | Automatica desde geometria |
| Post-cuantico | Vulnerable (preimagen) | Inmune (sin primitiva computacional) |

### vs BFT (PBFT, HotStuff)

| Aspecto | BFT | Tesseract |
|---|---|---|
| Supuesto | Mayoria honesta (2f+1) | Sin supuesto de mayoria |
| Finalidad | Condicional a honestidad | Incondicional |
| Tolerancia | Menos de n/3 bizantinos | Cualquier numero de destrucciones |
| Estado emergente | No — todo explicito | Si — emerge por proximidad |

### vs DAGs (IOTA, Narwhal)

| Aspecto | DAGs | Tesseract |
|---|---|---|
| Tiempo | Dimension privilegiada | 4 ejes simetricos |
| Validacion | Computacional (PoW/tip) | Convergencia geometrica |
| Estructura | Grafo dirigido aciclico | Campo probabilistico toroidal |
| Sanacion | Manual / redundancia | Automatica desde el campo |

---

## Estado actual

- **Conceptos formalizados:** Campo 4D, cristalizacion, auto-sanacion, rechazo de falsedad
- **4 leyes con pruebas criptograficas:** Causalidad (SHA-256), Conservacion (Pedersen), Entropia (hash chain), Gravedad (DAG causal)
- **Simulacion interactiva:** Disponible en el Block Explorer de Cerulean Ledger (`/tesseract`)
- **Ataques probados:** Sybil, eclipse, timing, cuantico, destruccion masiva, inyeccion, particion
- **No implementado en produccion:** Es investigacion. Cerulean Ledger usa Raft/BFT para consenso en produccion.

---

## Relevancia para la Camara

Tesseract representa la linea de investigacion a largo plazo de Cerulean Ledger. Mientras la plataforma hoy opera con consenso probado (Raft + BFT), Tesseract explora si es posible un mecanismo de consenso que:

1. No dependa de supuestos de honestidad
2. Sea inmune a computacion cuantica por diseno (no por parche criptografico)
3. Ofrezca auto-sanacion sin redundancia
4. Escale sin coordinacion

Es trabajo en progreso. Lo compartimos para dar contexto sobre la vision a largo plazo del proyecto y la profundidad tecnica del equipo.

---

*Tesseract es un modulo experimental de Cerulean Ledger. No esta activo en redes de produccion.*
