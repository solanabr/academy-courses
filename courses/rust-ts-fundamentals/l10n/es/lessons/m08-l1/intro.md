# Una blockchain es un target: el modelo mínimo + las primeras lecturas

## Resumen

M7 cerró el tier de edge: el motor de Rust corre como WASM en una segunda URL de workers.dev, y los dos edge workers ya sondean el endpoint getHealth de Solana como un target entre varios. La rampa estructural está lista. Hoy abre el tier de Solana, y abre de la misma forma en que abrió este curso: midiendo algo. Vas a construir `chain-probe.ts`, un script de bench que lee los signos vitales de una blockchain en vivo, mide su latido real sobre 20 muestras, lo compara contra la meta de 300ms publicada por la red, y deriva una cuenta regresiva de epoch en tiempo de reloj a partir de una sola constante fija. En el camino te llevas el modelo útil más chico de qué es siquiera una blockchain para un cliente: cuatro ideas, una oración cada una, con todo lo más profundo traspasado por nombre. Una palabra sobre cómo corre M8: guiado pero liderado por quien aprende. Yo trabajo el setup de RPC y una muestra en pantalla; tú escribes la agregación de 20 muestras, la línea de meta-contra-medido, y la aritmética de epoch tú mismo desde contratos declarados, y la extensión de min/max del final es solo tuya. Has entregado siete módulos de sondas. No necesitas que te esté encima.

## Mide algo primero

Tu estación viene sondeando `https://api.mainnet.solana.com` desde que m07-l1 lo puso en la lista de targets del worker, y hasta ahora la relación ha sido superficial a nivel de transporte: un POST que o responde rápido o, como documentó m07-l1, saluda a tu isolate con un 403 y se cambia por el fallback de publicnode. De cualquier forma, todavía nadie le hizo una pregunta de verdad. Pega esto en tu terminal ahora mismo:

```bash
curl -sS https://api.mainnet.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

```
{"jsonrpc":"2.0","result":"ok","id":1}
```

Eso es una blockchain respondiendo una pregunta JSON sobre HTTP simple. Sin billetera, sin clave, sin comisión, sin cuenta en ninguna parte. La misma forma de POST-un-cuerpo-JSON que vienes mandando desde las lecciones de HTTP de M2, apuntada a una red que ha procesado más de medio billón de transacciones (su propio getTransactionCount me lo dijo mientras escribía esto), y le responde a cualquiera que pregunte. Ahora hazlo desde TypeScript, en el repo de la estación. La estación es un workspace de pnpm desde m03-l1, así que las instalaciones pasan por pnpm (la flag `-w` dice "sí, me refiero a la raíz del workspace", así que cada paquete del workspace puede resolver la instalación; un `npm i` perdido acá garabatearía un package-lock.json rival dentro de un repo de pnpm):

```bash
pnpm add -w @solana/kit@^8
```

Nota de frescura: eso resuelve a 8.2.0 al 2026-09-02, y el dígito importa más de lo habitual acá. Kit entregó dos majors en poco más de nueve semanas este verano (7.0.0 a fines de junio, apenas semanas después de la minor 6.10.0, después 8.0.0 a fines de agosto), que es exactamente por qué la regla de este curso es fijar contra lo que tus dependencias declaran peer y re-verificar las líneas de instalación el día que las corres, no el día que se escribió un tutorial.

El script mismo va donde viven los scripts de bench de la estación: `packages/pulse-fleet/`, al lado de `probe.ts` y `fleet.ts`. Esa ubicación es estructural, no prolijidad: pulse-fleet carga el campo `"type": "module"` y el runner `tsx` que estos scripts necesitan, y la raíz del workspace no tiene ninguno de los dos, así que `npx tsx` corrido ahí muere en el top-level await ("Top-level await is currently not supported with the cjs output format"). Crea `packages/pulse-fleet/first-read.ts`:

```typescript
// packages/pulse-fleet/first-read.ts
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

console.log(await rpc.getHealth().send());
console.log(await rpc.getSlot().send());
```

```bash
cd packages/pulse-fleet
npx tsx first-read.ts
```

```
ok
443692227n
```

Tres líneas que funcionan. El sufijo `n` en ese segundo número es kit pasándote un `bigint`, porque los conteos de slots son valores u64 y kit se niega a mentir sobre eso a nivel de tipos. Guárdate eso: hoy es una gentileza, la próxima lección, cuando los números sean saldos, es la diferencia entre correcto y silenciosamente equivocado. ¿Y ese número grande en sí? Ese es el contador del latido de la blockchain, y el resto de esta lección es sobre qué quiere decir y qué tan rápido late.

## El modelo mínimo, derivado

### ¿Qué necesita saber un cliente en realidad?

Acá va la pregunta que le da forma a todo este tier. Tu estación quiere mostrar datos de Solana en vivo. ¿Cuál es el mínimo que tienes que entender sobre una blockchain para hacer eso con honestidad?

La respuesta máxima es un currículo: consenso, validadores, el modelo de cuentas como sistema, ejecución de programas, firmas criptográficas, mercados de comisiones. Temas reales, y este catálogo los enseña, pero no acá. Exigir todo eso antes de una primera lectura es así como los tutoriales pierden gente en la semana uno, y peor, es innecesario: acabas de leer estado de la blockchain en vivo con tres líneas y nada de ese conocimiento.

La respuesta mínima ingenua también falla, igual. "Es solo una API" te dio la sonda de getHealth en M7, pero se derrumba en el momento en que haces una segunda pregunta. ¿Por qué esa lectura no costó nada si todo el mundo dice que las blockchains tienen comisiones? ¿Por qué la página de docs habla de cuentas y slots y epochs? Una API cuyo vocabulario no puedes parsear es una API que vas a usar mal. Así que el mínimo honesto queda en el medio: necesitas exactamente las ideas que vuelven legible el lado de lectura de la API, y nada de lo que necesita quien escribe programas. Son cuatro, y vale la pena declarar la regla de selección porque es la misma regla 80/20 que este curso ya aplicó a dos lenguajes: una idea entra a la lista solo si una pregunta con la que te vas a topar personalmente antes del final de este módulo la fuerza. No "importante para las blockchains". Forzada, por tu propio código, esta semana.

**Las cuentas guardan lamports.** Una cuenta es una dirección con un saldo y algunos datos. El saldo se denomina en lamports, la unidad más chica de SOL: 1 SOL son 1,000,000,000 lamports, un entero, sin decimales a nivel del ledger (la lección de M2 sobre matemática de dinero te dijo por qué). Ese es el sustantivo del sistema. Una línea en node demuestra la forma:

```typescript
const LAMPORTS_PER_SOL = 1_000_000_000n;
console.log(2n * LAMPORTS_PER_SOL); // 2000000000n: two SOL, as the ledger stores it
```

**Los programas son código.** La lógica que mueve saldos de un lado a otro vive en programas. Un programa también es una cuenta, una cuyos datos resultan ser código ejecutable. Anotado, no explorado: esa sola oración es todo lo que necesita el lado del cliente.

**Las transacciones mutan.** El estado cambia exactamente de una forma: una transacción firmada. Las lecturas son preguntas gratis; las escrituras son eventos firmados, con comisión, que pasan por el consenso. Ese es el verbo del sistema, y esta lección contiene cero de ellas.

**El RPC es la puerta de lectura.** Un nodo de RPC guarda una copia del estado de la blockchain y responde preguntas JSON sobre él, que es lo que acabas de hacer dos veces. Sin transacción, sin comisión, sin billetera. La puerta a la que le vienes golpeando desde M7.

Ese es el modelo entero, y el conteo aguanta presión de los dos lados. Trata de encogerlo a tres sacando "los programas son código" y la primera página de explorer que abras deja de parsear: la mitad de las cuentas ahí están marcadas como ejecutables y no tienes ningún casillero en la cabeza para lo que eso quiere decir. Trata de crecerlo a cinco, con PDAs, digamos, o cuentas de token, y vas a encontrar que nada del código de este módulo toca nunca la quinta idea, lo que por la regla de selección la descalifica. Cuatro no es un número redondo que me haya gustado; es lo que fuerzan las preguntas.

¿Dónde viven de verdad los saldos de tokens, por qué se pueden derivar direcciones, qué puede y qué no puede hacer exactamente un programa, cómo funciona la historia de la blockchain? Todas preguntas reales, todas deliberadamente fuera de estas cuatro oraciones, y todas con un dueño que tiene nombre: el curso de evolución de Bitcoin a Solana de este catálogo recorre el modelo de cuentas como sistema, la ejecución de programas, los PDAs y la historia de la blockchain desde primeros principios. Esta lección enseña lo que necesita un cliente. Pretender que cuatro oraciones cubren el resto es así como los tutoriales producen confusión confiada, así que no lo voy a hacer, y el traspaso tiene un nombre en cambio.

![Cuatro ideas dispuestas alrededor de un hub, cuentas, programas, transacciones y RPC, con una flecha punteada pasándole temas más profundos a otro curso.](assets/v01-diagram.webp)

### Las lecturas son preguntas, las escrituras son eventos

De las cuatro ideas, la asimetría entre leer y escribir es la que le da forma a este módulo entero, así que se gana una segunda mirada antes de que empecemos a medir.

Cuando tu sonda llamó a getSlot, ningún validador registró que pasó. Nada se firmó, nada se pagó, nada tocó el consenso. Un nodo de RPC miró su copia del estado y respondió, de la forma en que cualquier servidor web responde un GET. Por eso el endpoint público puede ser gratis y abierto: responder preguntas es barato. Las comisiones existen para pagar la cosa cara, que es mutar estado replicado sobre el que miles de máquinas tienen que ponerse de acuerdo. Una escritura es una transacción firmada que compite por entrar en un bloque; una lectura nunca llega siquiera a ser una transacción.

La consecuencia práctica para la estación: tres lecciones de este módulo son lecturas gratis desde dos lenguajes, y después una escritura cuidadosamente ganada. La devnet, el cluster de práctica donde va a pasar esa escritura, es la única plataforma nueva de este módulo, y llega en m08-l4 con un par de claves y un faucet (un firmante basado en archivo, deliberadamente no una billetera; m08-l4 vuelve filosa esa distinción). Hoy y la próxima lección se quedan en el lado de lectura de la puerta, en mainnet, donde lo peor que puedes hacer es preguntar demasiado y demasiado rápido. Cosa sobre la que, como vas a ver en breve, el endpoint tiene opiniones.

![Dos carriles comparan una lectura gratis sin firmar respondida desde el estado del nodo contra una escritura firmada y con comisión que pasa por el consenso.](assets/v02-flowchart.webp)

### El latido se aceleró la semana en que se investigó este curso

Ahora el vocabulario para lo que getSlot devolvió en realidad. Un slot es la unidad de planificación de la red, su latido: cada slot, un validador tiene el derecho de producir un bloque. El slot y el bloque no son sinónimos, el slot es el tick del reloj y el bloque es lo que aterriza en él, pero para los fines de un cliente el contador de slots es el reloj, y solo sube. Un epoch son 432,000 slots, una constante fija de la red, y clusters es la palabra para las redes mismas: mainnet, donde vive el valor, más devnet y testnet para práctica y staging.

¿Qué tan rápido es el latido? Acá es donde la lección se pone una fecha encima. El 2026-08-28, cuatro días antes del barrido de investigación de este curso, la etapa 2 de SIMD-0525 se activó en mainnet y bajó la meta de tiempo de slot de 400ms a 300ms. Eso es un cuarto menos de intervalo, que es un tercio más de slots por segundo: la blockchain que estás sondeando se aceleró la semana en que se escribió este material. Un curso de fundamentos que se lanza ahora enseña una blockchain cuyo latido cambió la semana pasada, lo que te dice algo sobre por qué cada número de este curso carga una fecha.

Una trampa en cómo se cita ese hecho, porque le va a morder a cualquiera que revise las fuentes. El propio frontmatter del documento SIMD-0525 seguía diciendo Draft mientras el cambio estaba en vivo en mainnet. La demostración no es la spec, es la blockchain, y los dos números que el challenge va a picotear merecen líneas propias:

- El feature gate de la etapa 2 registra un slot de activación de **441,936,000**. Divide por 432,000 y te da exactamente 1023, sin resto: ese slot es el tick de apertura del epoch 1023.
- El comportamiento se encendió al inicio del epoch **1024**, una frontera después, porque así funcionan los feature gates de Solana: la activación aterriza durante un epoch, el interruptor se da vuelta en la frontera siguiente. Mantén ese retraso de un epoch en la cabeza.

Cualquiera puede decodificar la cuenta del gate desde el RPC público; lo re-verifiqué mientras escribía esto el 2026-09-02, sigue ahí, sigue activo, una sola llamada a getAccountInfo. Cita el gate on-chain, nunca la línea de estado de una spec. También hay una etapa 3 que apunta a 250ms; al momento de escribir esto su gate no estaba en vivo, y la medición que estás por tomar va a confirmar que la blockchain todavía corre al ritmo de la etapa 2. Si la etapa 3 aterriza después de que esta lección se entregue, tu medidor se vuelve más interesante, no equivocado. Ese es el punto de construir un medidor en vez de memorizar un número.

![Una línea de tiempo corre desde la activación de agosto pasando por dos mediciones fechadas hasta una flecha abierta para la propia sonda del lector.](assets/v03-timeline.webp)

### Metas contra mediciones

Así que la red dice 300ms. ¿Es eso lo que hace?

El barrido de investigación preguntó, a la manera honesta: getRecentPerformanceSamples, un método de RPC que devuelve los propios tiempos de slot registrados por el nodo en ventanas de 60 segundos. Veinte muestras, promediadas, el 2026-09-01: 316ms. No 300. Alrededor de cinco por ciento por encima de la meta, y ese hueco no es un escándalo, es como se ven los sistemas en vivo. Una meta es una intención de ingeniería; una medición es lo que pasó, clima de red incluido. Mi propia re-ejecución mientras escribía esta lección volvió con 313.6ms. Misma historia, otro día.

La objeción filosa primero, porque deberías estar levantándola: ¿ese hueco de 16ms es real, o es tu propio viaje de ida y vuelta HTTP filtrándose en los números? Es real, y vale la pena hacer tuya la razón antes del lab. getRecentPerformanceSamples no cronometra nada de tu lado del cable; devuelve el propio historial registrado del nodo, cuántos slots pasaron de verdad en cada ventana de 60 segundos que ya registró. La latencia de tu conexión decide cuándo recibes esos registros, no los valores que hay adentro. Existe un esquema de medición donde tu latencia sí contamina el resultado, y lo vas a construir en el lab como un descartable deliberado, precisamente para que sientas la diferencia entre cronometrar algo tú mismo y pedirle sus logs a un sistema.

Si esto se siente familiar, debería. Es la lección más vieja de este curso vestida de blockchain: M1 te hizo medir tu propia latencia en vez de confiar en un número de un README, y M2 te enseñó que afirmado y medido son columnas distintas. Ahora el sistema bajo prueba es una blockchain, y la disciplina se transfiere sin cambios. Tu propio panel hace afirmaciones de uptime; la versión honesta de tu estación publica lo que mide, no lo que espera. La realidad dura es: cada sistema que vayas a operar alguna vez tiene un hueco entre su meta y su comportamiento, y los equipos que saben el tamaño de su hueco son en los que puedes confiar.

Para una estación que pasó siete módulos aprendiendo a medir los endpoints de otra gente, una meta que ya viene con su propia API pública de medición se siente como que te pasan la hoja de respuestas. La mayor parte de la infraestructura te hace adivinar. Esta blockchain te pasa getRecentPerformanceSamples y te desafía a revisar.

![Tres barras muestran la meta de 300 milisegundos al lado de mediciones de 316 y 313.6, un hueco de alrededor de cinco por ciento.](assets/v04-chart.webp)

### El reloj de epoch sale de la aritmética

Acá va la parte que me parece calladamente deliciosa. Los epochs se definen en slots, no en tiempo: 432,000 slots, siempre, antes de la aceleración y después de ella. Lo que quiere decir que la duración del epoch en tiempo de reloj es una cantidad derivada, y cuando el tiempo de slot se movió, cada epoch del calendario se encogió en silencio.

Haz la aritmética una vez a mano, porque el lab hace que tu script la haga para siempre después. Con la vieja meta de 400ms: 432,000 slots por 0.4 segundos son 172,800 segundos, que son 48 horas. A 300ms: 432,000 por 0.3 son 129,600 segundos, 36 horas. Nadie redimensionó los epochs, nadie anunció un cambio de calendario, y sin embargo todo lo que está atado a las fronteras de epoch, los ciclos de staking, los cronogramas de validadores, ahora da la vuelta medio día antes. Una constante fija, una variable medida, y la respuesta en tiempo de reloj sale de una multiplicación. Tu medidor va a calcular las horas que quedan del epoch actual desde el tiempo de slot que acaba de medir, lo que quiere decir que tu reloj de epoch se mantiene correcto incluso si la etapa 3 aterriza y encoge los epochs de nuevo a 36 por cinco sextos, la meta de 250ms sobre la de 300ms, que ahora puedes sacar tú mismo.

![Dos columnas multiplican el conteo fijo de slots por dos tiempos de slot, convirtiendo 48 horas en 36 sin que nada más cambie.](assets/v05-comparison.webp)

### La puerta de lectura es gratis, tiene tope y es honesta al respecto

Última pieza del modelo antes de construir: la puerta misma. La URL que imprime este curso es `https://api.mainnet.solana.com`, la forma actual en los propios docs de clusters de Solana. Los tutoriales más viejos, y hay muchos, usan `api.mainnet-beta.solana.com`; ese alias legacy todavía responde, así que reconócelo cuando lo veas, pero escribe la forma actual. De acá en adelante, este curso escribe solo el nombre actual.

El endpoint público es gratis, y es honesto sobre qué quiere decir gratis. Los topes documentados:

| Tope | Límite |
| --- | --- |
| Requests por IP | 100 por 10 segundos |
| Requests por IP, un solo método | 40 por 10 segundos |
| Conexiones concurrentes por IP | 40 |
| Datos por IP | 100 MB por 30 segundos |

Los docs después dicen la parte callada con palabras simples: estos endpoints "no están pensados para aplicaciones de producción". Gratis, abierto, con tope de tasa, explícitamente una herramienta de bench. Que es precisamente el trade-off que estás aceptando hoy, y lo quiero nombrado en vez de descubierto. Haz la aritmética de presupuesto a la manera de m02-l3: una ejecución del medidor que estás por construir gasta tres requests, así que el tope por IP toleraría el lab entero, el challenge, y treinta re-ejecuciones paranoicas dentro de una sola ventana de diez segundos. Un panel desplegado con cincuenta visitantes, cada navegador refrescando un panel en vivo, revienta 100 requests por 10 segundos antes de que termines de leer esta oración. Mismo endpoint, mismos topes, veredictos opuestos. Ese desajuste es el problema de apertura de la próxima lección, no una nota al pie. Mientras tanto la disciplina de bench: muestrea con un retraso, nunca martilles getSlot en un loop apretado, y trata los 429 como el endpoint diciéndote la verdad sobre para qué es.

**Profundiza (el 20%).** la referencia canónica para los clusters y sus endpoints públicos, incluyendo cada rate limit de arriba, las URLs de devnet y testnet, y los links de explorer, es la propia página de clusters de Solana: https://solana.com/docs/references/clusters (verificada en vivo el 2026-09-02). Déjala como bookmark; esta lección enseñó deliberadamente solo el camino de lectura de mainnet, y esa página es dueña del resto.

## Lab: solana-probes-v0, el medidor de la blockchain

El artefacto que este tier empieza a construir es `solana-probes-v0`, y su primera pieza es `chain-probe.ts`: un script de bench independiente en `packages/pulse-fleet` que imprime los signos vitales de la blockchain, su latido medido, y la cuenta regresiva del epoch. Deliberadamente v0, deliberadamente un prototipo de bench. La próxima lección lleva a producción estas lecturas hacia el panel desplegado y el edge worker; el trabajo de hoy es lograr que la medición esté bien en tu propia máquina primero, el mismo ritmo de bench-y-después-deploy que la estación viene siguiendo desde M3.

El repliegue, dicho sin rodeos: los pasos 1 a 3 están trabajados, yo muestro el código. Los pasos 4 y 5 te dan un contrato y tú escribes el código. Si quieres la versión honesta de esta lección, no hagas scroll hacia adelante para revisarte hasta que tu versión corra.

![Tres superficies desplegadas se sientan encima de un script de bench solitario, con una flecha prometiendo que las lecturas suben la próxima lección.](assets/v06-diagram.webp)

**1. Arma el archivo.** En el repo de la estación, crea `packages/pulse-fleet/chain-probe.ts` al lado de `first-read.ts` y los otros scripts de bench, y corre todo lo de este lab desde ese directorio, por la razón de módulo-y-tsx que nombró la apertura. Ya instalaste `@solana/kit@^8` en la raíz en la apertura; la única otra herramienta es `tsx`, que está en las dev dependencies de pulse-fleet desde que los scripts de bench se mudaron al workspace (si de algún modo estás en una carpeta fresca fuera de la estación: `npm i -D tsx`, y dale a su package.json `"type": "module"`). Empieza con los signos vitales que ya sabes leer:

```typescript
import { createSolanaRpc } from "@solana/kit";

const SLOTS_PER_EPOCH = 432_000n;
const TARGET_SLOT_MS = 300;

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

const health = await rpc.getHealth().send();
const slot = await rpc.getSlot().send();
console.log(`health: ${health} | slot: ${slot}`);
```

Las dos constantes de arriba son los dos anclajes de la lección: la duración fija del epoch como un `bigint` (va a dividir el número de slot `bigint`, y la aritmética mezclada de bigint y number es un error de compilación de TypeScript, que es el sistema de tipos haciéndote un favor), y la meta como número simple, porque la matemática de milisegundos se queda cómodamente en el rango de `number`.

**2. Toma una muestra ingenua.** Antes de agarrar el instrumento correcto, mide el latido de la forma en que medirías cualquier cosa: dos lecturas y un reloj. Este es el loop de muestreo trabajado, y también es uno que respeta el tope de tasa, dos llamadas con diez segundos de diferencia, no un loop caliente:

```typescript
// packages/pulse-fleet/naive.ts - a throwaway, not part of the gauge
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

const before = await rpc.getSlot().send();
await new Promise((r) => setTimeout(r, 10_000));
const after = await rpc.getSlot().send();

const slotMs = 10_000 / Number(after - before);
console.log(`${after - before} slots in 10s -> ~${slotMs.toFixed(0)}ms per slot`);
```

Mi ejecución: `34 slots in 10s -> ~294ms per slot`. Cerca de la meta, y ruidosa, porque diez segundos es una ventana chiquita y tu propia latencia de request embarra los dos extremos. Demuestra el concepto y muestra la debilidad en un solo archivo descartable. Para hacerlo mejor tendrías que muestrear por minutos, y resulta que no tienes que hacerlo, porque el nodo viene muestreando para ti.

**3. Trae las muestras reales.** `getRecentPerformanceSamples` devuelve las propias ventanas de rendimiento registradas por el nodo, cada una 60 segundos de tiempo de la blockchain con la cantidad de slots que pasaron de verdad adentro. Veinte muestras son veinte minutos de historia medida en un solo request, sin loop, sin ansiedad por el tope de tasa, y sin embarre de latencia de request, porque los tiempos adentro de las muestras son los registros del nodo, no tus viajes de ida y vuelta. Agrega a `chain-probe.ts`:

```typescript
const samples = await rpc.getRecentPerformanceSamples(20).send();
console.log(samples[0]);
```

Corre `npx tsx chain-probe.ts` una vez para ver la forma de una muestra. Los campos que necesitas: `numSlots` (un `bigint`, los slots producidos en la ventana) y `samplePeriodSecs` (un `number`, la duración de la ventana). Borra la línea `console.log(samples[0])` una vez que hayas mirado; el medidor imprime conclusiones, no materia prima.

**4. Escribe la agregación y la línea del medidor (tú).** El contrato que tu código tiene que satisfacer:

- Suma `numSlots` y `samplePeriodSecs` en las 20 muestras, después calcula el promedio de milisegundos por slot como segundos totales sobre slots totales, por 1000. Suma primero, después divide una sola vez: promediar los promedios por muestra le daría el mismo peso a una ventana corta que a una larga.
- `numSlots` es un `bigint`; convierte con `Number()` en la división. Los conteos de slots por ventana son unos pocos cientos, para nada cerca de la pérdida de precisión, y dilo en un comentario para que el tú del futuro no entre en panic.
- Imprime una línea de medidor: el promedio medido a un decimal, la meta de 300ms, el hueco porcentual con signo, y el conteo de muestras. La mía dice: `slot time: 313.6ms measured vs 300ms target (+4.5%) over 20 samples`.

**5. Escribe el reloj de epoch (tú).** Segundo contrato, y es la aritmética de la sección de teoría hecha ejecutable:

- El epoch actual: el número de slot dividido por `SLOTS_PER_EPOCH`, en aritmética de `bigint`, que redondea hacia abajo gratis.
- Los slots que quedan: la duración del epoch menos la posición del slot dentro del epoch, vía el operador `%`, que `bigint` también soporta.
- Las horas que quedan: los slots que quedan por tus milisegundos medidos, dividido por 3,600,000. Usa el valor medido, no la meta; el reloj debería decir la verdad que tu medidor acaba de establecer.
- Imprímelo como una línea con el número de epoch, los slots que quedan, y las horas a un decimal.

Antes de correr el script terminado, revisa su forma contra el mapa de abajo. No el código, la estructura: cuatro etapas, tres líneas impresas, y una regla estricta sobre qué número alimenta a cuál. Si tu versión calcula el reloj de epoch desde la meta de 300ms en vez del promedio medido, compila, corre, y calladamente dice una verdad peor.

![Cuatro etapas del script del medidor se apilan verticalmente, con el tiempo de slot medido alimentando el reloj de epoch mientras la constante de la meta queda excluida de él.](assets/v07-annotated-code.webp)

**6. Córrelo.** Salida completa de mi ejecución al momento de escribir, 2026-09-02:

```
health: ok | slot: 443692920
slot time: 313.6ms measured vs 300ms target (+4.5%) over 20 samples
epoch 1027: 403080 slots / ~35.1h remaining
```

Tus números de slot y de epoch van a ser más altos, tu promedio medido debería caer más o menos en la banda de 300-330ms al momento de la investigación, y la línea del hueco debería mostrar un porcentaje positivo chico. Cualquier cosa salvajemente fuera de esa banda, revisa primero el orden de suma-después-divide; es donde la mayoría de las versiones de este script salen mal.

**7. Demuestra que es un reloj.** El checkpoint que separa un medidor de una impresión con suerte: córrelo de nuevo unos minutos después. El número de slot debería haber avanzado más o menos tu tiempo transcurrido dividido por tu tiempo de slot medido. Cinco minutos son 300,000ms, que a ~314ms por slot son alrededor de 950 slots. Si tus dos ejecuciones encierran esa aritmética, tu script no está solo leyendo un contador, midió la tasa del contador, y ahora sabes la velocidad de reloj de una blockchain igual que sabías la latencia de tu API en M1: porque la mediste tú mismo.

![Una hoja de trabajo de seis filas: dos lecturas de slot más los minutos transcurridos divididos por el tiempo de slot medido predicen el avance, y la coincidencia es la demostración.](assets/v08-table.webp)

Una cosa más antes de darlo por hecho: las tres líneas de salida de este script son su interfaz, así que mantenlas limpias, etiquetadas, y de un-hecho-por-línea. Sé preciso sobre qué se promueve, igual, para que la próxima lección no pueda decepcionarte: m08-l2 sube las lecturas de clase signos-vitales (el slot, más las lecturas de saldo que introduce) a las superficies desplegadas, mientras que el medidor de 20 muestras y el reloj de epoch se quedan como instrumentos de bench a propósito, demasiado hambrientos de requests para un panel por visitante viviendo bajo los topes públicos. El medidor sigue pagando el alquiler justo acá, cada vez que lo re-corres para revisar el ritmo de la blockchain contra una afirmación, que es una cosa que ahora vas a hacer por años. v0 es un prototipo, no una excusa.

## Challenge

Dos escalones, sin guía. Primero, extiende el medidor: reporta el tiempo de slot mínimo y máximo a lo largo de la ventana de 20 muestras junto al promedio, y marca cualquier muestra individual que haya corrido más lento que el doble de la meta. Ese es el pensamiento de latency-stats de M1, dispersión y outliers, apuntado a datos de la blockchain; mi ventana de hoy fue de 309.3ms a 326.1ms sin nada marcado, y una muestra marcada en un día tranquilo vale la pena sospecharla (empieza por tu propia aritmética antes de culpar a la blockchain).

Segundo, el challenge calificado `epoch-clock` de la plataforma te da una función `epochClock(slot, slotMs)` con tres bugs plantados: redondea el epoch en vez de hacer floor, reporta los slots transcurridos en vez de los que quedan, y arruina la conversión de milisegundos a horas. Los tres son bugs que tu código del lab acaba de evitar; el arreglo es transferir lo que hiciste en el paso 5 al borrador roto de otra persona. El anclaje de aceptación vale la pena internalizarlo antes de que empieces: en el slot 441,936,000 con slots de 300ms tiene que reportar el epoch 1023 con un epoch entero de 432,000 slots y 36.0 horas restantes, porque 441,936,000 dividido por 432,000 es exactamente 1023 y un slot de frontera pertenece al epoch que abre. Si puedes explicar eso, el bug del floor ya está resuelto en tu cabeza. (Y sí, este es a propósito el slot del gate que nombró la sección de teoría: el slot ESTÁ en el epoch 1023, mientras que la feature que activó se encendió en el epoch 1024, el retraso de un epoch que marca la línea de tiempo. Tu `epochClock` responde dónde está un slot, no cuándo una feature entra en efecto.)

## Checkpoint

Ahora puedes hacer cuatro cosas concretas: explicarle una blockchain a otro desarrollador en cuatro oraciones sin vaguedades, y nombrar dónde vive la historia más profunda; leer estado de la blockchain en vivo desde TypeScript con kit contra el RPC público de mainnet; medir el tiempo de slot real de una red y declarar con un número el hueco respecto de su meta; y convertir un conteo crudo de slots en una cuenta regresiva de epoch en tiempo de reloj a partir de una sola constante fija. Tu `chain-probe.ts` corre, y su segunda ejecución demostró su propia aritmética.

La recuperación de 30 segundos, en voz alta antes de cerrar la pestaña: ¿cuál de las cuatro ideas explica por qué hoy no te costó nada? (El RPC es la puerta de lectura; las lecturas nunca se vuelven transacciones.) ¿Y por qué los epochs se acortaron en agosto cuando nadie cambió el epoch? (Los epochs son 432,000 slots, fijos; el tiempo de slot es la variable, así que la duración en tiempo de reloj se movió con él.)

Un pedido mientras está fresco: esta es la primera lección del curso donde el sistema bajo prueba es una blockchain en vez de algo que desplegaste, y quiero saber si el modelo de cuatro ideas aguantó o si una quinta pregunta te estuvo molestando durante el lab. Dime cuál. Si el modelo necesita una quinta oración, ese es exactamente el feedback que le cambia la forma a este tier.

Puedes medir el latido de la blockchain desde un script de bench. Pero mira lo que está desplegado: el panel de Vercel, los edge workers, cada superficie con una URL sigue siendo chain-blind, sondeando getHealth como si fuera cualquier otro endpoint. La próxima lección las lecturas van a producción, un panel de Solana en vivo en cada superficie desplegada, y la disciplina que construiste para HTTP inestable en M2 resulta ser exactamente lo que exige un RPC público con tope de tasa. Trae tu medidor.
