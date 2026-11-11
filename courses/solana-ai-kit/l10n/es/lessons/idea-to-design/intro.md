# De una idea a un diseño

En la lección pasada el investigador respondió una pregunta por su nombre. En esta lección dos agentes llevan Stamp de **tres líneas de idea a un diseño desde el que un ingeniero puede construir**, y aprendes qué te debe cada uno cuando termina. Unos veinte minutos, y nada de esto necesita un toolchain.

## Por qué importa

La mayoría de los proyectos de hackathon que vi fracasar perdieron su mes en la primera semana, con el modelo de cuentas equivocado y una idea copiada. El kit pone dos especialistas en ese punto. **solana-researcher** responde preguntas del ecosistema con fuentes citadas y fechadas y una etiqueta de confianza en cada hallazgo, porque el ecosistema se mueve más rápido que la memoria de cualquier modelo. **solana-architect** fija cuentas, PDAs, límites de programa, firmantes y el modelo de confianza *antes* de que exista código, para que los ingenieros nunca los reabran. Los juegos tienen su propio arquitecto, **game-architect**, que escribe un archivo de concepto y un archivo de plan y decide qué vive on-chain. La salida de esta lección es un archivo que el resto del curso lee.

## Manos a la obra

1. **Escribe la idea en tres líneas.** Crea `idea.md` en el proyecto con una persona, lo que no puede hacer hoy y lo que Stamp hace al respecto. La nuestra:

```text
Stamp
Rita tiene una cafetería y regala un café cada décima compra, con tarjetas de papel que los clientes pierden.
Stamp guarda la tarjeta en Solana: la tienda sella la billetera del cliente en la caja, diez sellos canjean un café, y el cliente ve su tarjeta en cualquier app de billetera.
```

2. **Pregúntale al investigador qué existe.** Escribe dentro de Claude Code:

```text
Usa el agente solana-researcher. ¿Qué existe hoy en Solana para tarjetas de fidelidad o tarjetas de sellos de pequeños negocios, incluyendo proyectos que cerraron y por qué? Cita una fuente por cada hallazgo con fecha y etiqueta de confianza, y termina con lo que no pudiste verificar.
```

Su archivo fija la forma de la respuesta: **primero una respuesta de dos o tres oraciones**, después hallazgos cada uno con un enlace y una etiqueta, **High** cuando varias fuentes autorizadas coinciden, **Medium** para una, **Low** para fuentes comunitarias solamente o fuentes que se contradicen, **Speculative** para una inferencia, después una recomendación y una lista de lo que no pudo verificar. Lee los ítems Low como pistas, no como hechos. *El agente tiene permitido decir que no encontró nada sólido, y esa respuesta vale tenerla antes de construir.*

### Si quieres ir más a fondo en la investigación

Los hallazgos del investigador se detienen en lo que es público en la web. El kit también trae **Colosseum Copilot** como pack de skills central, que responde desde los archivos de hackathons e historiales de proyectos de Colosseum: qué afirmaron las entregas pasadas, qué entregaron después y qué sigue sin saberse. Necesita una cuenta de Colosseum y un inicio de sesión una vez por máquina, en una terminal, con Node 20 o más nuevo:

```bash
npx @colosseum-org/copilot-connect login
```

El inicio de sesión abre el navegador. Agrega `--device` en una máquina donde no se pueda abrir un navegador, y revisa la conexión con `status`, que es también lo que `/doctor` corre en su quinto chequeo. Después pregunta dentro de Claude Code:

```text
Usa el agente solana-researcher con el skill de Colosseum copilot. ¿Qué entregas de hackathons pasados construyeron tarjetas de fidelidad o de sellos para pequeños negocios? Para cada una, separa lo que afirmó al entregar de lo que pasó después, con fechas, y termina con lo que no pudiste encontrar.
```

Lee la entrada del registro del kit antes de iniciar sesión. Marca el pack como *caution* (precaución): el helper es un paquete npm sin versión fijada, el skill consulta la API de Colosseum como tú, y tus preguntas y las respuestas del agente van a Colosseum cuando marcas la casilla opcional de compartir al iniciar sesión, que viene apagada por defecto. *Para un taller, los hallazgos públicos alcanzan. Para una entrada a un hackathon, este es el archivo que quieres revisado.*

3. **Entrégale la idea al arquitecto.** Escribe:

```text
Usa el agente solana-architect para diseñar Stamp a partir de idea.md. Quiero cada cuenta con su dueño, seeds, tamaño, quién paga la renta y quién puede cambiarla o cerrarla, cada instrucción con sus firmantes, los invariantes, el modelo de autoridad y los riesgos abiertos. Usa Anchor por defecto salvo que puedas mostrar una razón medida para Pinocchio. Escribe el resultado en design.md.
```

Esa lista es la propia sección **Output** del arquitecto, citada de vuelta, que es un truco que vale guardar: *un agente al que se le pide exactamente lo que su archivo promete rara vez deja algo afuera.* Espera algo así para Stamp:

- Una **cuenta de tienda**, un PDA a partir de un prefijo fijo y la billetera de la tienda, que guarda el número de sellos por premio y la autoridad de la tienda.
- Una **cuenta de tarjeta por cliente**, un PDA a partir del prefijo, la tienda y el cliente, que guarda el conteo de sellos y el conteo de canjes. Una cuenta por cliente, porque el archivo del arquitecto dice que las cuentas escribibles compartidas serializan cada transacción que las toca, y dos clientes en la caja al mismo tiempo no deberían esperarse uno al otro.
- Tres instrucciones: **create shop** (firma la tienda), **stamp** (firma la tienda, se escribe la tarjeta del cliente), **redeem** (firma el cliente, diez sellos salen de la tarjeta). El arquitecto, o `/plan-feature` después de él, va a preguntar quién paga la renta de una tarjeta nueva, y *la respuesta honesta para una cafetería es la tienda.*
- Invariantes como que los sellos nunca quedan negativos y un canje siempre cuesta exactamente el número configurado, un modelo de autoridad para la clave de la tienda, y una lista de riesgos que empieza con la clave de la tienda estando en un teléfono.

![El carril de diseño: tres líneas de idea van a solana-researcher para hallazgos citados con etiquetas de confianza, después a solana-architect para cuentas, instrucciones, invariantes y riesgos en design.md, después a /plan-feature para un plan por fases, con game-architect tomando el paso de diseño para juegos.](assets/v01-design-lane.webp)

4. **Convierte el diseño en fases.** Escribe:

```text
/plan-feature Stamp, a partir de design.md
```

El comando no escribe código. Lee el diseño, pregunta por las decisiones que lo cambian y devuelve un plan de nueve secciones, resumen, cuentas, instrucciones, integraciones, clientes, fases, riesgos, pruebas y despliegue, descartando las que no aplican y **terminando en preguntas abiertas para ti**. Respóndelas en `design.md` para que los próximos agentes lean un solo archivo.

5. **Si vas a un hackathon, agrega el pack de startup.** `/add-skill startup-builder` instala el pack completo de diez skills, idea-sprint, positioning, pricing, pitch-deck, el skill de hackathon, build-status, fundraising, launch, incident-comms y ecosystem-bd, porque se enlazan entre sus archivos. Su idea-sprint escribe `.claude/context/idea.md`, y `/plan-feature` arranca desde ese archivo cuando existe. Colosseum Copilot, del aparte de investigación de arriba, es lo que su skill de hackathon lee para buscar antecedentes.

## Tres semillas de idea para probar en vez de Stamp

Los mismos dos prompts funcionan con cualquier idea. Reemplaza las tres líneas y deja el resto:

- **Vaquinha** (una colecta entre amigos): amigos juntan SOL para un regalo y el fondo paga cuando se alcanza la meta, o devuelve a todos en una fecha límite. Primera pregunta de investigación: qué existe para fondos grupales y escrow, y cómo manejan el camino de la devolución.
- **Biblioteca de herramientas**: un galpón de barrio presta taladros y escaleras contra un depósito que vuelve al devolverlos. Primera pregunta de investigación: patrones de depósito y devolución, y quién guarda el depósito.
- **Lista de la puerta**: un evento pequeño vende un número fijo de entradas que no se pueden revender en absoluto, solo devolver al organizador. Primera pregunta de investigación: ticketing en Solana, y qué proyectos intentaron frenar la reventa.

## Está listo cuando

- `idea.md` tiene tres líneas y `design.md` tiene cuentas, instrucciones, invariantes, autoridad y riesgos.
- Cada hallazgo de la respuesta de investigación trae un enlace y una etiqueta.
- Las preguntas abiertas del plan están respondidas en `design.md`.

## Ojo con

- **El arquitecto diseña y no programa.** Si empieza a escribir Rust, detenlo y pídele el archivo de diseño.
- **Pinocchio necesita una razón medida.** El archivo del arquitecto lo dice: Anchor por defecto, y Pinocchio cuando una instrucción sigue por encima de su presupuesto de cómputo después de las propias palancas de Anchor, cuando las comisiones de cómputo a volumen determinan el costo del producto, o cuando el tamaño del binario o la renta del deploy son una restricción dura. Una instrucción caliente se optimiza primero en su lugar. Un programa donde la mayoría de las instrucciones son calientes se escribe en Pinocchio desde el inicio.
- **Una etiqueta Low es una pista.** No pongas un número Low en un deck.
- **Los juegos van a game-architect.** Pregunta por la jugabilidad, la plataforma y qué debe estar on-chain, después escribe `concept.md` y `plan.md` y espera tu aprobación antes de que alguien implemente.

## Lo que queda

La investigación vuelve **citada y etiquetada**, y un diseño vuelve como **cuentas, instrucciones, invariantes, autoridad y riesgos** en un archivo. *Pídele a cada agente la salida que su propio archivo promete, y guarda el archivo, porque los próximos agentes lo leen.*

## Siguiente

En la próxima lección el anchor-engineer construye las tres instrucciones de Stamp desde `design.md`, el ingeniero de QA las prueba, y ves los tres comandos que cierran cada tarea de programa. Rust, Anchor y la Solana CLI entran ahí, así que instálalos antes de la sesión si quieres seguir al teclado. `/doctor` ya imprimió el comando de la Solana CLI, y el paso de scaffold del ingeniero pone a la vista el toolchain de Anchor.
