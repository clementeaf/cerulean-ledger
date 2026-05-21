# Que es una DLT Empresarial Permisionada?

**Documento introductorio para la directiva de la Camara de Blockchain Chile**

---

## DLT: Distributed Ledger Technology

Una DLT es un registro digital compartido entre multiples organizaciones que no confian entre si, donde:

- Cada participante tiene una copia identica del registro
- Ningun participante puede alterar el registro unilateralmente
- Los cambios se propagan automaticamente a todos los participantes
- El historial completo es inmutable y auditable

No es una base de datos compartida. Es un sistema donde la **verdad es colectiva** — no la controla ninguna de las partes.

---

## Permisionada vs publica

Existen dos modelos fundamentales de DLT:

| Aspecto | DLT publica | DLT permisionada |
|---|---|---|
| Quien participa | Cualquier persona, anonimamente | Solo organizaciones autorizadas |
| Ejemplo conocido | Bitcoin, Ethereum | Hyperledger Fabric, Cerulean Ledger |
| Identidad | Pseudonima (direcciones) | Verificada (certificados, DIDs) |
| Gobernanza | Algoritmica (mineria, staking) | Contractual (politicas, acuerdos) |
| Privacidad | Todo es publico | Datos privados entre partes |
| Rendimiento | Lento (7-30 TPS en Bitcoin) | Rapido (miles de TPS) |
| Costo por transaccion | Gas fees variables | Sin costo por transaccion |
| Regulacion | Dificil de cumplir | Disenada para cumplimiento |

**En resumen:** Una DLT permisionada es una red privada donde las organizaciones participantes se conocen, tienen identidad verificada, y operan bajo reglas acordadas. No hay mineria, no hay tokens publicos, no hay participacion anonima.

---

## Que implica "empresarial"

Una DLT empresarial no es solo permisionada — esta disenada para operar en contextos de negocio reales. Esto significa:

### 1. Identidad verificable

Cada participante tiene una identidad criptografica emitida por una autoridad reconocida. No hay anonimato. Cada transaccion es atribuible a una organizacion y un rol especifico.

**Implicancia:** Cumplimiento regulatorio (CMF, SII, GDPR) es posible porque siempre se sabe quien hizo que y cuando.

### 2. Privacidad controlada

No todos ven todo. Los datos se comparten selectivamente:

- **Channels:** Redes separadas dentro de la misma infraestructura. Solo los miembros del channel ven sus transacciones.
- **Private data collections:** Datos que solo comparten dos o tres organizaciones, sin que el resto de la red los vea.

**Implicancia:** Una empresa puede compartir datos con un regulador sin exponer informacion comercial a competidores en la misma red.

### 3. Politicas de endorsement

Antes de que una transaccion se registre, debe ser aprobada por las organizaciones que la politica defina:

- "Al menos 2 de 3 organizaciones deben aprobar"
- "Todas las organizaciones del channel deben aprobar"
- "Cualquier organizacion puede aprobar"

**Implicancia:** El nivel de confianza requerido se configura por tipo de operacion. Una transferencia de fondos puede requerir unanimidad; una lectura de datos, solo una firma.

### 4. Smart contracts con lifecycle

Los contratos inteligentes (chaincode) no se despliegan libremente. Siguen un proceso controlado:

1. Una organizacion propone una nueva version
2. Las demas organizaciones la revisan y aprueban
3. Solo cuando hay consenso, se activa en la red

**Implicancia:** Ningun participante puede cambiar las reglas unilateralmente. Los cambios al software de la red son gobernados colectivamente.

### 5. Inmutabilidad auditable

Cada transaccion queda registrada permanentemente con:

- Quien la ejecuto (identidad)
- Cuando (timestamp)
- Que organizacion la endorso
- Que datos cambio (antes y despues)

**Implicancia:** Audit trail completo para reguladores, auditores externos y compliance interno. No se necesita reconstruir la historia — el registro ES la historia.

### 6. Tolerancia a fallas

Si un nodo se cae, la red sigue operando. Si una organizacion se desconecta, se reconecta y sincroniza automaticamente. Los datos no se pierden.

**Implicancia:** Operacion continua sin punto unico de falla. Ningun participante puede bloquear la red desconectandose.

---

## Cerulean Ledger vs Hyperledger Fabric: misma arquitectura, diferente implementacion

Cerulean Ledger usa el mismo modelo arquitectonico de Fabric — Execute-Order-Validate (EOV). No reinventamos el modelo porque funciona. Lo que hicimos fue reimplementarlo con ventajas tecnicas concretas. Pero seamos honestos: Fabric tiene ventajas que nosotros aun no tenemos.

### Donde Cerulean Ledger supera a Fabric

| Dimension | Fabric | Cerulean Ledger |
|---|---|---|
| Criptografia post-cuantica | No — requiere reescribir BCCSP (anos de trabajo) | ML-DSA-65 (FIPS 204) end-to-end |
| Consenso bizantino | Solo Raft (tolera crashes, no nodos maliciosos) | Raft + BFT (tolera nodos maliciosos) |
| Ejecucion de transacciones | Secuencial | Paralela con wave scheduling y deteccion de conflictos |
| Rendimiento | ~3,000 TPS | 56,000 TPS medidos |
| Memoria por nodo | ~500 MB (Go + Java + Docker) | ~50 MB (un binario Rust) |
| Compatibilidad EVM | No soporta Solidity | Si — deploy y ejecucion via revm |
| Despliegue | 30+ min, CAs en Go/Java, YAML extenso, cryptogen | `docker compose up`, 4 minutos, variables de entorno |
| Light client | No nativo | Si — verificacion Merkle para IoT/movil |
| Seguridad de memoria | Go (con garbage collector, sin garantias en tiempo de compilacion) | Rust (garantias de memoria en tiempo de compilacion, sin GC) |

### Donde Fabric supera a Cerulean Ledger

| Dimension | Fabric | Cerulean Ledger |
|---|---|---|
| Madurez en produccion | 7+ anos operando en empresas reales | Sin pilotos en produccion aun |
| Ecosistema | SDKs en 5 lenguajes, comunidad global, documentacion extensa | SDK TypeScript (Python en roadmap) |
| Casos de uso probados | Walmart, Maersk, JPMorgan, HSBC, De Beers | Sin casos reales todavia |
| Certificate Authority | Fabric CA madura con enrolamiento y registro | Identidad via DIDs (sin CA tradicional) |
| Soporte comercial | IBM, multiple vendors | Equipo fundador |
| Integracion con sistemas legacy | Conectores maduros para SAP, Oracle, etc. | Por desarrollar |

### Lo que esto significa

Cerulean Ledger no pretende tener la madurez de Fabric — seria deshonesto. Lo que ofrecemos es una implementacion tecnica superior del mismo modelo probado, con una ventaja que Fabric no puede agregar facilmente: criptografia post-cuantica integrada desde el diseno.

Fabric tendria que reescribir su modulo criptografico (BCCSP), reconstruir su infraestructura de certificados X.509, actualizar todos los SDKs en 5 lenguajes, y coordinar la migracion de redes en produccion. Son anos de trabajo. Cerulean Ledger lo tiene hoy porque se diseno desde cero con esa capacidad.

La madurez de ecosistema se construye con tiempo, pilotos y comunidad. La arquitectura criptografica se elige al principio — cambiarla despues es ordenes de magnitud mas dificil.

---

## Donde se usa hoy en el mundo

| Sector | Uso | Quien |
|---|---|---|
| Comercio internacional | Trazabilidad de contenedores y documentos | Maersk + IBM (TradeLens) |
| Alimentos | Origen y cadena de frio desde granja a supermercado | Walmart + IBM (Food Trust) |
| Finanzas | Conciliacion interbancaria y liquidacion | JPMorgan (Onyx), HSBC |
| Salud | Historial medico compartido entre instituciones | Avaneer Health |
| Gobierno | Registro de propiedad, titulos, certificados | Dubai Land Department |
| Cadena de suministro | Verificacion de proveedores y compliance | De Beers (Tracr) |

Todos estos usan DLT empresariales permisionadas — no Bitcoin ni Ethereum publico.

---

## Por que Rust

Cerulean Ledger esta escrito en Rust. No es una eleccion estetica — es una decision de ingenieria con consecuencias directas:

| Propiedad | Que significa | Impacto en una DLT |
|---|---|---|
| Seguridad de memoria | El compilador impide accesos invalidos, buffer overflows y data races en tiempo de compilacion | Elimina la clase de vulnerabilidades mas explotada en software de infraestructura (70% de CVEs en C/C++ segun Microsoft) |
| Sin garbage collector | No hay pausas impredecibles por recoleccion de basura | Latencia consistente bajo carga — critico para consenso y ordering |
| Rendimiento nativo | Compila a codigo maquina, sin VM ni interprete | 50 MB de RAM por nodo vs 500+ MB en Fabric (Go + Java) |
| Concurrencia segura | El sistema de ownership previene data races en tiempo de compilacion | Ejecucion paralela de transacciones sin bugs de concurrencia |
| Binario unico | Un solo ejecutable sin dependencias de runtime | Despliegue simple: un binario, un contenedor, sin JVM ni runtime de Go |

**Comparacion con las alternativas:**

- **Go** (Fabric): Mas simple de escribir, pero el garbage collector introduce pausas. Sin garantias de seguridad de memoria en tiempo de compilacion.
- **Java** (Corda): JVM pesada, garbage collector, superficie de ataque amplia. Requiere gestion de dependencias compleja.
- **C++**: Rendimiento equivalente, pero sin garantias de seguridad de memoria. Historicamente propenso a vulnerabilidades criticas.

Rust da el rendimiento de C++ con las garantias de seguridad que una DLT empresarial necesita. No es casualidad que proyectos como Solana, Polkadot, Diem (Meta) y NEAR eligieron Rust para sus implementaciones blockchain.

---

## Que cambia con la criptografia post-cuantica

Las DLT empresariales actuales (Fabric, Corda) usan criptografia clasica (RSA, ECDSA). Esta criptografia es vulnerable a computadores cuanticos:

- **Amenaza "harvest now, decrypt later":** Un adversario puede interceptar datos firmados hoy y romper las firmas cuando tenga acceso a un computador cuantico.
- **Horizonte:** NIST estima que computadores cuanticos criptograficamente relevantes podrian existir entre 2030 y 2040.
- **Problema:** Los registros de una DLT empresarial deben ser validos por decadas — titulos de propiedad, contratos, historiales medicos.

Cerulean Ledger resuelve esto implementando firmas ML-DSA-65 (FIPS 204, NIST 2024) en toda la stack. Los datos firmados hoy seran verificables incluso en un futuro post-cuantico.

### FIPS 204 en simple

**FIPS** = Federal Information Processing Standard. Son los estandares de seguridad del gobierno de Estados Unidos, publicados por el NIST (National Institute of Standards and Technology). Cuando algo es "FIPS", significa que paso por anos de revision publica, ataques academicos y validacion formal.

**FIPS 204** es el estandar publicado en agosto 2024 que define **ML-DSA** (Module-Lattice-Based Digital Signature Algorithm) — un algoritmo de firma digital que resiste ataques de computadores cuanticos.

En terminos simples:

- **Firma digital** = el equivalente criptografico de una firma notarial. Prueba que un documento fue creado por quien dice ser, y que no fue alterado despues.
- **ML-DSA-65** = una firma digital cuya seguridad se basa en problemas matematicos de reticulados (lattices), no en factorizacion de numeros primos (RSA) ni curvas elipticas (ECDSA).
- **Por que importa:** Los computadores cuanticos pueden resolver factorizacion y curvas elipticas eficientemente (algoritmo de Shor). No pueden resolver problemas de reticulados eficientemente. ML-DSA-65 sobrevive a la era cuantica.

**El "65" en ML-DSA-65** indica NIST security level 3 — equivalente a 128 bits de seguridad clasica. Suficiente para proteger informacion clasificada segun la NSA.

| Algoritmo | Base matematica | Resiste computador cuantico | Estandar |
|---|---|---|---|
| RSA-2048 | Factorizacion de primos | No | FIPS 186 |
| ECDSA (P-256) | Curvas elipticas | No | FIPS 186 |
| **ML-DSA-65** | **Reticulados (lattices)** | **Si** | **FIPS 204** |

Cerulean Ledger usa ML-DSA-65 para firmar bloques, transacciones, endorsements e identidades. Es seleccionable por nodo — una red puede operar con firmas clasicas y post-cuanticas simultaneamente, permitiendo migracion gradual.

### ML-DSA-65 end-to-end: como funciona en la practica

"End-to-end" suena tecnico, pero la idea es simple: **cada vez que algo se firma en Cerulean Ledger, se firma con ML-DSA-65**. No hay excepciones, no hay puntos ciegos.

Para entender que significa, sigamos el recorrido de una transaccion desde que un usuario la crea hasta que queda registrada permanentemente:

```
1. CREAR       El usuario firma la transaccion con su clave ML-DSA-65
               "Yo, Empresa A, transfiero X a Empresa B"
                         |
                         v
2. ENDORSAR    Los nodos endorsantes verifican la firma del usuario,
               simulan la transaccion, y firman el resultado con ML-DSA-65
               "Confirmamos que esta transaccion es valida"
                         |
                         v
3. ORDENAR     El servicio de ordering recibe la transaccion endorsada,
               la incluye en un bloque, y firma el bloque con ML-DSA-65
               "Este es el bloque #547 con estas 12 transacciones"
                         |
                         v
4. DISTRIBUIR  Los nodos se comunican via P2P con mensajes firmados ML-DSA-65
               "Aqui va el bloque #547, verificalo"
                         |
                         v
5. REGISTRAR   Cada nodo valida todas las firmas (usuario + endorsers + bloque),
               y almacena el bloque permanentemente
               "Bloque #547 verificado y almacenado"
```

**Cada flecha del diagrama es una firma ML-DSA-65.** Si cualquiera de esos pasos usara criptografia clasica, seria el punto debil que un adversario cuantico atacaria. End-to-end significa: no hay punto debil.

**En numeros:**

- Una transaccion simple genera al menos 3 firmas ML-DSA-65 (usuario + endorser + bloque)
- En una red de 6 nodos con 3 endorsers, un bloque de 10 transacciones produce ~40 firmas PQC
- Cada firma ML-DSA-65 ocupa 3,309 bytes (vs 64 bytes de Ed25519) — mas grande, pero el costo es asumible con el rendimiento de Rust

**Lo que el operador ve:**

Nada especial. Cambia una variable de entorno:

```
SIGNING_ALGORITHM=ml-dsa-65
```

El nodo arranca y todo funciona igual. La complejidad criptografica queda dentro del binario. El operador no necesita entender reticulados ni FIPS — solo elige el algoritmo y opera.

**Lo que el auditor ve:**

Cada firma en el registro indica que algoritmo se uso. Si una firma es Ed25519, es clasica. Si es ML-DSA-65, es post-cuantica. El auditor puede verificar que toda la cadena esta protegida con PQC sin herramientas especiales — la informacion esta en los datos.

### Firmas post-cuanticas en produccion: que significa

"En produccion" no es lo mismo que "en un paper" o "en un prototipo de laboratorio". Significa que las firmas post-cuanticas estan integradas en el software que se despliega, ejecuta y opera en una red real. La diferencia es importante:

| Nivel | Que implica | Quien esta ahi hoy |
|---|---|---|
| Investigacion | Papers academicos, pruebas de concepto aisladas | Muchas universidades y empresas |
| Prototipo | Modulo funcional pero no integrado en la stack completa | Algunas startups, labs de Google/IBM |
| **Produccion** | **Integrado end-to-end en el software que se despliega y opera** | **Cerulean Ledger** |

**"End-to-end" significa que TODA firma en el sistema usa ML-DSA-65:**

- Bloques — el bloque que sella un conjunto de transacciones
- Transacciones — cada operacion individual dentro de un bloque
- Endorsements — la aprobacion de cada organizacion antes de registrar
- Identidades — el registro DID de cada participante
- Gossip P2P — los mensajes entre nodos para sincronizar estado

Si una sola de estas capas usa criptografia clasica, el sistema tiene un eslabon debil. Un adversario ataca el punto mas fragil. Produccion significa que no hay eslabones debiles.

### Que implica para una organizacion

**Hoy:**

- Los datos que firman con su DLT actual (Fabric, Corda, o cualquier otra) quedan protegidos con ECDSA o RSA.
- Un adversario sofisticado (estado-nacion, competidor con recursos) puede interceptar y almacenar esos datos firmados. No los puede leer ni alterar hoy.

**Manana (2030-2040):**

- Cuando ese adversario tenga acceso a un computador cuantico, puede romper las firmas retroactivamente.
- Puede probar que un contrato fue alterado, falsificar la autoria de una transaccion, o invalidar un registro de propiedad.
- Los datos que firmaron hace 10 anos dejan de ser confiables.

**Con firmas PQC en produccion hoy:**

- Los datos firmados ahora resisten tanto ataques clasicos como cuanticos.
- No hay ventana de vulnerabilidad. No hay deuda criptografica que pagar despues.
- Cuando llegue la computacion cuantica, los registros siguen siendo validos.

### Como impacta en sectores concretos

| Sector | Dato critico | Vida util | Riesgo sin PQC |
|---|---|---|---|
| Inmobiliario | Titulo de propiedad | Decadas | Falsificacion retroactiva de transferencias |
| Salud | Historial medico | Toda la vida del paciente | Alteracion de diagnosticos o tratamientos |
| Financiero | Contratos y conciliaciones | 10-30 anos (regulacion) | Repudio de transacciones firmadas |
| Gobierno | Certificados, licencias, votaciones | Indefinido | Invalidacion de actos oficiales |
| Comercio exterior | Certificados de origen, fitosanitarios | 5-15 anos | Falsificacion de trazabilidad |
| Legal | Contratos firmados digitalmente | Segun prescripcion | Disputas sobre autenticidad |

**La pregunta no es si la computacion cuantica llegara, sino si los datos que firmas hoy seguiran siendo confiables cuando llegue.** Cerulean Ledger es la primera DLT empresarial permisionada que integra firmas FIPS 204 end-to-end para responder esa pregunta hoy.

---

## Verificabilidad del claim

Cerulean Ledger afirma ser "la primera DLT empresarial permisionada con firmas ML-DSA-65 (FIPS 204) integradas end-to-end". Este claim es especifico y verificable:

**Lo que afirmamos:**
- Firmas ML-DSA-65 (FIPS 204, NIST 2024) integradas en bloques, transacciones, endorsements e identidades
- Seleccionable por nodo via variable de entorno (`SIGNING_ALGORITHM=ml-dsa-65`)
- Coexistencia de firmas clasicas (Ed25519) y post-cuanticas en la misma red

**Como verificarlo:**
- El codigo fuente es open source — auditable en `src/identity/` (SigningProvider trait, ML-DSA-65 implementation)
- 2,700+ tests automatizados incluyen tests especificos de firma/verificacion PQC
- `docs/PQC-TEST-EVIDENCE.md` documenta el inventario completo de tests PQC

**Lo que NO afirmamos:**
- No afirmamos ser "la unica blockchain con PQC" — QRL (Quantum Resistant Ledger) usa XMSS desde 2018, pero es una blockchain publica, no una DLT empresarial permisionada
- No afirmamos tener deployments en produccion — tenemos la capacidad implementada y testeada, no pilotos reales aun
- No afirmamos que ninguna otra plataforma este trabajando en PQC — NIST publico FIPS 204 en agosto 2024 y es esperable que otros proyectos lo integren progresivamente

**El claim preciso es:** primera DLT empresarial permisionada con FIPS 204 integrado end-to-end. Es verificable, acotado y honesto.

---

## Estandares y reguladores relevantes

### NIST — National Institute of Standards and Technology

- **Que es:** Agencia del gobierno de EE.UU. que define los estandares tecnicos que usa el mundo. Cuando el NIST publica un estandar, se convierte en referencia global — no solo para EE.UU.
- **Por que importa:** NIST publico FIPS 204 (ML-DSA) en agosto 2024 como el estandar oficial de firmas post-cuanticas. Es el resultado de 8 anos de competencia publica donde se evaluaron 82 algoritmos candidatos. ML-DSA-65 fue uno de los ganadores.
- **Relacion con Cerulean Ledger:** Implementamos ML-DSA-65 exactamente como lo define FIPS 204. No es una interpretacion ni una variante — es el estandar tal cual.

### CNSS Policy 15 — Committee on National Security Systems

- **Que es:** La directiva de la NSA (Agencia de Seguridad Nacional de EE.UU.) que define los plazos de migracion a criptografia post-cuantica para sistemas de seguridad nacional.
- **Que exige:** Todos los sistemas que manejan informacion clasificada deben migrar a algoritmos quantum-safe antes de 2030. Los sistemas no clasificados pero sensibles, antes de 2035.
- **Por que importa para Chile:** Cualquier organizacion chilena que interactue con cadenas de suministro estadounidenses, contratos de defensa, o exporte a EE.UU. estara sujeta a estas exigencias. La compliance se propaga por la cadena.
- **Relacion con Cerulean Ledger:** Ya cumple. No hay migracion pendiente.

### eIDAS 2.0 — Electronic Identification, Authentication and Trust Services

- **Que es:** El reglamento de la Union Europea que define como funcionan las identidades digitales, firmas electronicas y servicios de confianza en los 27 paises miembro.
- **Version 2.0 (2024):** Introduce la European Digital Identity Wallet — cada ciudadano europeo tendra una billetera digital con credenciales verificables (titulo universitario, licencia de conducir, historial medico). La direccion regulatoria apunta a aceptar firmas post-cuanticas.
- **Por que importa para Chile:** Chile exporta a la UE (sector agroalimentario, mineria, vinos). Los certificados de origen, fitosanitarios y de trazabilidad que acompanan esas exportaciones tendran que ser compatibles con los estandares europeos. Si Europa exige firmas PQC, los exportadores chilenos necesitan emitirlas.
- **Relacion con Cerulean Ledger:** Las credenciales verificables de Cerulean Ledger (DID + ML-DSA-65) estan alineadas con la direccion de eIDAS 2.0.

### CMF — Comision para el Mercado Financiero (Chile)

- **Que es:** El regulador financiero chileno. Supervisa bancos, aseguradoras, fondos de inversion, bolsas de valores y fintech.
- **Que exige:** Trazabilidad completa de operaciones financieras, audit trail inmutable, reporte regulatorio periodico, y cumplimiento de estandares de seguridad de la informacion.
- **Por que importa:** Cualquier solucion DLT que opere en el sector financiero chileno necesita satisfacer las exigencias de la CMF en materia de auditoria, seguridad y trazabilidad.
- **Relacion con Cerulean Ledger:** Audit trail inmutable por organizacion, identidades verificables, channels privados para datos sensibles, y export de registros para reportes regulatorios. La CMF podria participar como nodo observador (solo lectura) en una red.

### SII — Servicio de Impuestos Internos (Chile)

- **Que es:** La autoridad tributaria chilena. Controla la emision de documentos tributarios electronicos (facturas, boletas, guias de despacho).
- **Que exige:** Integridad y trazabilidad de documentos tributarios, firma electronica avanzada, y conservacion de registros por los plazos legales (6 anos minimo para documentos tributarios).
- **Por que importa:** Los documentos tributarios firmados con criptografia clasica podrian ser cuestionados en el futuro si las firmas se vuelven vulnerables. El SII aun no exige PQC, pero los plazos de conservacion (6+ anos) se superponen con el horizonte de amenaza cuantica.
- **Relacion con Cerulean Ledger:** Documentos tributarios firmados con ML-DSA-65 mantienen su validez criptografica durante todo el periodo de conservacion legal, incluso en un escenario post-cuantico.

### Resumen de alineacion

| Estandar / Regulador | Exigencia | Cerulean Ledger |
|---|---|---|
| NIST FIPS 204 | Firmas post-cuanticas estandarizadas | Implementado (ML-DSA-65) |
| CNSS Policy 15 | Migracion PQC antes de 2030 | Ya cumple |
| eIDAS 2.0 | Identidad digital + firmas avanzadas | Alineado (DID + credenciales verificables) |
| CMF | Audit trail, trazabilidad, seguridad | Audit trail inmutable, channels privados |
| SII | Integridad de documentos tributarios | Firmas PQC validas por todo el periodo legal |

---

## Analogia simple

Imaginen un libro notarial compartido entre 10 notarias:

- Cada notaria tiene una copia identica del libro
- Para agregar una entrada, al menos 2 notarias deben firmarla
- Ninguna notaria puede borrar o modificar entradas anteriores
- Algunas paginas solo son visibles para las notarias involucradas en ese acto
- Si una notaria cierra, las demas siguen operando con el libro completo
- Un auditor puede verificar cualquier entrada en cualquier momento

Eso es una DLT empresarial permisionada. Cerulean Ledger es el software que lo hace posible — con la garantia adicional de que las firmas seran seguras incluso contra computadores cuanticos.

---

*Documento preparado como material introductorio para la Camara de Blockchain Chile — abril 2026.*
