# Las pruebas son sondas para tu código: vitest

## Resumen

La flota sondea cincuenta targets y publica lo que encuentra. Nada sondea la flota. Esta lección le da la vuelta a eso: escribes una suite de vitest que fija las fronteras del clasificador como una tabla, afirma el cronograma de backoff sin esperar un solo milisegundo real, y deja afuera para siempre el bug de config de m02-l2, después cablea la suite entera al workflow de pulse.yml para que un clasificador roto no pueda volver a publicar status.json. Barrera #2 en el pipeline, y la estación empieza a monitorearse a sí misma.

## Primero, rompe algo

m02-l3 le enseñó modales a la flota bajo carga: un pool hecho a mano, backoff con jitter, timeouts de AbortController. Cincuenta targets sondeados con cero 429s, cada resultado aterrizando tipado. La flota funciona. Todavía nada demuestra que siga funcionando.

Esa distinción es toda la lección, así que hagámosla concreta antes de cualquier teoría. Instala la herramienta:

```bash
npm i -D vitest@4.1.11
```

(Re-sondeado 2026-09-06, y esto es una lección sobre dist-tags por sí sola: cuando esta lección se escribió el 2026-09-02, la v5 todavía era un release candidate y `latest` servía la línea v4. Vitest 5.0.0 llegó a GA el 2026-09-03 y se quedó con `latest`; la línea v4 ahora vive en su propio tag `V4`, en 4.1.11. El pin de arriba es exactamente por qué la instalación nombra una versión en vez de confiar en `latest`: cada transcripción de esta lección es de la v4. Arranca en v5 si prefieres, lee primero sus notas de migración, y todo lo que esta lección enseña sobre las pruebas como barrera sigue aplicando. `npm view vitest dist-tags` imprime el mapa actual en una línea.)

Una pieza de setup para que la sonda tenga un target estable, y trae consigo un renombrado que tienes que hacer a propósito, porque los dos clasificadores que has escrito hasta ahora quieren el mismo nombre. Tu clasificador de l1 sigue viviendo donde lo dejó ese lab, llamado `classifyProbe` y atado al objeto de la unión; el challenge de m02-l1 calificó una firma distinta bajo el mismo nombre. Mueve el archivo a `src/classify.ts` y parte el nombre en dos:

- **`classify(result: ProbeResult): Verdict`** — la forma de unión de l1, renombrada. Se queda, y no por sentimentalismo: es la única forma que puede juzgar un `dns-error`, que lleva un hostname y ningún número, y el panel de M3 va a necesitar exactamente eso.
- **`classifyProbe(kind: string, value: number)`** — la forma de frontera que calificó el challenge. Corre `parseProbe` primero (los kinds desconocidos vuelven como `'invalid'`), después le pasa el resultado parseado a `classify`.

Dos nombres exportados, un switch exhaustivo por debajo: escribe la forma de frontera sobre la forma de unión, nunca al lado, o acabas de construir adentro de un solo archivo la deriva contra la que este curso te viene advirtiendo. Haz el renombrado en una sola pasada y deja que `npx tsc --noEmit` te camine hasta cada sitio de llamada, incluida la línea del driver de l1; ese mandado es lo que m02-l1 te vendió y esta es la primera vez que lo gastas. Estos son también los nombres exactos que mueve la extracción de paquete de m03-l1 y que importa el panel de m03-l2, así que acertarlos hoy es un renombrado que no haces después. Diez minutos, nada de lógica nueva, y cada prueba de esta lección importa ese único archivo. (El `probe.ts` de la raíz se encoge hasta ser una CLI que importa de `./src/classify.js`. Una arruga honesta para notar en vez de arreglar: `src/fleet.ts` se queda con la copia local de `ProbeResult` que declaró en l3, así que el repo ahora guarda dos uniones estructuralmente idénticas. `src/classify.ts` es la canónica desde hoy, la duplicación es exactamente el riesgo de deriva que la extracción de paquete de M3 existe para cerrar, y tienes permiso de que te moleste hasta entonces.)

Ahora escribe una prueba. Crea `tests/classify.test.ts` al lado de tu flota:

```ts
import { expect, test } from 'vitest';
import { classifyProbe } from '../src/classify.js';

test('a 400ms probe is degraded, not up', () => {
  expect(classifyProbe('ok', 400)).toBe('degraded');
});
```

Córrelo:

```bash
npx vitest run
```

Si tu lógica de fronteras está bien, te sale un check verde. Si tu verificación de frontera dice `> 400` donde la spec de m02-l1 decía que la banda degraded empieza EN 400, acabas de atrapar el tipo de bug que el cron habría publicado cada treinta minutos, para siempre, con un checkmark verde al lado. De cualquier manera aprendiste algo de verdad en cinco minutos, que es todo el argumento de venta.

Dos ortografías de ese comando, y la diferencia importa toda la lección: `npx vitest run` ejecuta la suite una vez y sale, que es lo que quiere el CI. `npx vitest` a secas arranca el modo watch: se queda vivo, vuelve a correr las pruebas afectadas cada vez que guardas un archivo, y convierte la suite en una lectura en vivo mientras trabajas. Usa el modo watch en tu escritorio por el resto de este lab; la forma `run` es la que va en el workflow después. Y fíjate que invocamos vitest con `npx` en todas partes este módulo, nunca a través de un script `test` de `package.json`: el stub de npm-init en `scripts.test` se queda intacto hoy, a propósito, y m03-l1 cablea `"test": "vitest run"` en el momento en que el `pnpm -r test` del workspace de verdad necesita un script que encontrar.

Acá está la síntesis que le gana a esta lección su título: una suite de pruebas es un monitor de uptime apuntado a tu propio código. La versión hacia afuera ya la construiste. Una aserción es una sonda con una lectura esperada. Una prueba que falla es un 429 de tu propia lógica. Misma disciplina, apuntada hacia adentro, y ya te sabes la disciplina.

## Sondas apuntadas hacia adentro

Vienes haciendo "testing" a mano desde m01-l2: corre la flota, mírale la salida, asiente. Eso funciona hasta que el código cambia mientras no estás mirando la salida, que es lo que es el resto de este curso. Cada módulo de acá en adelante agrega código del que depende otro código. La suite es cómo un cambio descuidado en la forma de salida de la flota queda atrapado antes de que rompa el panel que vas a entregar en el módulo 3, que lee esa forma y nada más.

![Cinco conceptos de monitoreo como las sondas y las lecturas esperadas mapean uno a uno sobre conceptos de testing como las llamadas a función y las aserciones.](assets/v01-diagram.webp)

vitest es el runner que usa este curso: habla TypeScript nativamente con cero config, y encuentra cualquier cosa que coincida con `*.test.ts`. Alrededor de 99.9 millones de descargas por semana al momento de escribir esto, por lo que valgan los conteos de descargas; no es un veredicto del ecosistema de Solana, eso sí, y esta lección te va a mostrar el otro campamento antes de terminar. Los patrones de abajo son el 80% diario: tablas, fake timers, fixtures, cobertura. Todo lo demás queda como bookmark al final de esta sección.

### La tabla es la spec

Tu clasificador tiene un contrato, y ya lo sabes de memoria porque el challenge de m02-l1 te calificó sobre él: latencia por debajo de 400 es `up`, de 400 a 1000 es `degraded`, por encima de 1000 es `down`, un 429 quiere decir que el target respondió así que es `degraded` y no `down`, los kinds desconocidos son `invalid`. Cinco reglas de frontera. Podrías escribir cinco funciones de prueba separadas y repetir la ceremonia cinco veces, o podrías fijarte en que son todas la misma oración con números distintos:

```ts
import { expect, test } from 'vitest';
import { classifyProbe } from '../src/classify.js';

const rows: Array<[kind: string, value: number, expected: string]> = [
  ['ok', 399, 'up'],
  ['ok', 400, 'degraded'],
  ['ok', 1000, 'degraded'],
  ['ok', 1001, 'down'],
  ['http-error', 429, 'degraded'],
  ['http-error', 500, 'down'],
  ['timeout', 0, 'down'],
  ['gopher', 200, 'invalid'],
];

test.each(rows)('classifyProbe(%s, %d) is %s', (kind, value, expected) => {
  expect(classifyProbe(kind, value)).toBe(expected);
});
```

`test.each` (una prueba de tabla: un solo cuerpo de prueba, corrido una vez por fila) convierte el contrato en datos. Lee las filas en voz alta y estás leyendo la spec. Esa es la victoria de verdad, no lo que te ahorras de escribir: cuando el challenge de m02-l1 agregó la regla de que 429 es degraded, eso fue una fila. Cuando un bug de frontera alguna vez aparezca en producción, el pin de regresión es una fila. Las pruebas más baratas de extender son las más propensas a extenderse, y una tabla cuesta una línea por lección aprendida.

Fíjate qué filas están acá. No entradas al azar: los valores exactos donde cambia el comportamiento. 399 y 400. 1000 y 1001. Las fronteras son donde viven los bugs corridos en uno, así que las fronteras son donde las sondas apuntan.

Una palabra rápida sobre la aserción misma, porque vas a recurrir a ella doscientas veces en este curso. `toBe` verifica identidad: la respuesta correcta para cadenas, números, booleanos, cualquier cosa que devuelva el clasificador. En el momento en que afirmas sobre un objeto o un array, cámbiate a `toEqual`, que compara estructura. `expect({ a: 1 }).toBe({ a: 1 })` falla, dos objetos distintos, la misma forma; `toEqual` pasa. Ese par cubre la mayor parte de tu vida de aserciones. El catálogo de matchers va mucho más a fondo (`toMatchObject`, `toThrow`, `resolves`, y vas a conocer `resolves` en el paso 3), pero toBe-para-valores y toEqual-para-formas es el reflejo diario que vale la pena instalar ahora.

### Fake timers: afirma el cronograma, sáltate la espera

El cronograma de backoff de m02-l3 es un contrato también: el intento n espera `min(capMs, baseMs * 2^n)`. Con una base de 500ms, un techo de 8000ms y cinco retries, eso es 500, 1000, 2000, 4000, 8000. (La flota del lab corría un techo de 5000ms; la prueba lo sube a 8000 para que cada delay ejercite la duplicación antes del clamp.) Pruébalo con timers reales y cada ejecución de la suite se pasa quince segundos reales durmiendo, lo que quiere decir que dejas de correr la suite, lo que quiere decir que ya no tienes una suite.

![Cinco barras se duplican de 500 a 8000 milisegundos, totalizando más de quince segundos de espera que los fake timers eliminan.](assets/v02-chart.webp)

`vi.useFakeTimers()` (el reemplazo de reloj de vitest: intercepta `setTimeout` y compañía para que los callbacks programados se disparen cuando TÚ adelantas el reloj, no cuando lo hace el reloj de pared) está construido exactamente para esta forma de código. El modelo mental clave, y el que el quiz va a picar: los fake timers no encogen los delays. Los tiempos programados mantienen sus valores exactos. Saltas el reloj a cada instante programado y afirmas qué se disparó. Esa precisión es por qué la prueba puede fijar el cronograma valor por valor en vez de afirmar "pasaron como cinco esperas".

El patrón, primero sobre un juguete:

```ts
import { afterEach, beforeEach, expect, test, vi } from 'vitest';

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

test('the callback fires at 500ms, not before', async () => {
  const fired = vi.fn();
  setTimeout(fired, 500);

  await vi.advanceTimersByTimeAsync(499);
  expect(fired).not.toHaveBeenCalled();

  await vi.advanceTimersByTimeAsync(1);
  expect(fired).toHaveBeenCalledTimes(1);
});
```

`vi.fn()` es un spy: una función falsa que registra cómo fue llamada. `advanceTimersByTimeAsync` mueve el reloj falso y deja que se resuelvan las promises que estaban esperando esos timers. El baile de 499-y-después-1 es la misma disciplina de fronteras que la tabla: afirma que nada pasa un milisegundo antes, después afirma que pasa exactamente a tiempo. Vas a hacer esto contra el loop de retry de verdad en el lab.

Una trampa antes de que te la encuentres: los fake timers solo ayudan si el código bajo prueba es lo bastante puro para poder manejarse. Tu clasificador y tu fórmula de backoff toman valores y devuelven valores; nunca tocan la red. Esa fue una decisión de diseño que m02-l1 y m02-l3 tomaron antes de que supieras por qué, y este es el por qué. Sondear URLs de verdad dentro de pruebas unitarias vuelve la suite flaky, lenta y rate-limited, y ya sabes exactamente qué siente el target por las ráfagas. El motor de Rust de M4 va a hacer la misma jugada de mantener-el-núcleo-puro a propósito, y lo vamos a decir otra vez ahí.

### Fixtures: el bug que no puede volver nunca

En m02-l2 le metiste un typo a un campo de la config, `intervalSeconds` donde el código lee `intervalSecs`, y rastreaste cómo la versión sin parsear de la flota se lo habría tragado con cortesía. Después construiste la frontera de zod y viste el mismo archivo morir en el arranque con un error a nivel de campo. Ese archivo con typo está por recibir un ascenso: de anécdota de guerra a fixture (un archivo de entrada commiteado que las pruebas cargan; evidencia congelada, repetida para siempre).

La jugada es chica y es uno de los hábitos de más valor del curso: cada bug que arreglas se convierte en una prueba que falla si el bug vuelve. En el lab vas a volver a cometer ese crimen contra el schema ACTUAL de la flota (m02-l3 le cambió la forma a la config para la flota concurrente, así que el archivo original es un schema desactualizado), vas a estacionar la copia saboteada en `tests/fixtures/`, y vas a afirmar que `safeParse` la RECHAZA. No "la flota parece estar bien". El rechazo, afirmado, por nombre. Si alguien alguna vez aflojara el schema y la mentira cortés volviera a ser representable, la suite se pone en rojo antes de que el cron pueda publicar una sola línea equivocada.

![Un pipeline de cinco etapas que muestra un bug siendo arreglado, congelado como archivo de fixture, fijado por una prueba de rechazo, y atrapado permanentemente si alguna vez vuelve.](assets/v03-diagram.webp)

### Cobertura: señal, no ídolo

Corre una suite con cobertura y te sale un porcentaje: cuántas líneas de tu fuente se ejecutaron mientras corrían las pruebas. Una pregunta útil de hacer, un número terrible para adorar, y necesitas las dos mitades de esa oración.

La mitad útil: una rama sin cubrir es un target de sonda que nadie está mirando. Tu clasificador tiene brazos para `timeout`, para `http-error`, para los kinds desconocidos. Si la cobertura muestra que el brazo de `timeout` nunca corrió, ninguna prueba de la suite notaría si empezara a devolver `up`. Ese es exactamente el instinto de monitoreo hacia afuera que ya tienes: un target sin ninguna sonda apuntada puede estar down una semana sin que nadie se entere. Lee el reporte de cobertura como lees la lista de targets de la flota, buscando el hueco que importa.

La mitad del ídolo: 100% de cobertura de líneas demuestra que cada línea CORRIÓ bajo alguna prueba. No demuestra nada sobre si las aserciones de esas líneas atraparían una respuesta equivocada. Una suite que llama a cada función y no afirma nada saca puntaje perfecto. Y las últimas ramas sin cubrir son a menudo inalcanzables a propósito: tu brazo `assertNever` existe precisamente para que NO PUEDA correr, y perseguir un número que lo penaliza quiere decir borrar tu propia baranda de seguridad para complacer a una métrica. La cobertura marca qué mirar. Los humanos deciden qué importa.

### El ecosistema, nombrado con honestidad

Un compás para cada uno, porque vas a encontrarte con los tres en el mundo real:

![Tres runners de pruebas comparados por lo que son y por cuándo se los encuentra un desarrollador, con vitest escrito acá y jest leído en el mundo real.](assets/v04-comparison.webp)

`node:test` es el aparte de cero dependencias: un runner de pruebas de verdad que ya viene dentro de Node mismo, estable desde Node 20, sin ninguna instalación. Su historia de cobertura sigue siendo experimental, que es por qué es el aparte y no la lección. Vale saber que existe; algunas herramientas chicas genuinamente no necesitan nada más.

jest es el establecido, y la honestidad importa más que la lealtad tribal acá: vas a encontrarte con la línea jest 30 en los repos de anza, kit y gill los dos prueban con él. Las APIs son parecidas por diseño, así que leer sus pruebas se va a sentir familiar. Las configs no son parecidas, y esa es la trampa: copiar config de jest desde un repo del mundo real hacia un proyecto de vitest produce fallas misteriosas, porque el parecido está en los archivos de prueba, no en la plomería.

Si la escala de décadas de los ecosistemas te sorprende, a esta altura no debería: Express 5 siguió a Express 4 después de diez años (2014-04-09 a 2024-09-10), y el ecosistema corrió la major vieja felizmente todo ese tiempo. El testing es la misma historia. Aprendes la herramienta actual y lees al establecido, porque el mundo real corre las dos, y "leer el mundo real" es una habilidad que este curso te viene comprando a propósito.

### La parte honesta

Las pruebas son código que también tienes que mantener, y pretender lo contrario es cómo los equipos terminan odiando sus suites. Cada cambio del clasificador ahora rompe filas de la tabla. La prueba de backoff fija los delays tan apretado que reafinar el cronograma a propósito quiere decir editar pruebas, lo cual es medianamente molesto exactamente cuando estás apurado. La barrera de CI que estás por construir agrega minutos entre el merge y la publicación. Toda esa fricción es el punto: la fricción en el camino a publicar una mentira es el producto.

Pero nombra dónde se invierte, porque se invierte. Las pruebas sobre-especificadas, del tipo que afirma sobre cadenas de log incidentales o se mete en las tripas privadas, vuelven caro cada refactor sin atrapar ningún bug de verdad: la prueba se rompe con cada reescritura de texto y nunca con un error de lógica. Y una META de cobertura, "exigimos 95%", persigue líneas en vez de riesgo y te consigue pruebas que ejecutan código sin verificarlo. La brújula para las dos: prueba el contrato, no la implementación; pon barrera a la publicación, no a cada tecla.

**Profundiza (el 20%).** los patrones diarios de arriba son lo que necesita la flota; el resto de vitest es una caja de herramientas profunda que deberías saquear a demanda, no memorizar. La [guía de vitest](https://vitest.dev/guide/) (verificada en vivo 2026-09-02) cubre lo que dejamos deliberadamente como bookmark: la taxonomía de mocking (mocks, spies, module mocks), el snapshot testing y el modo browser. Aparte para el camino de cero dependencias: la [documentación de node:test](https://nodejs.org/api/test.html). Nada del lab de abajo depende del material guardado como bookmark.

## Lab: la suite, después la barrera

El cronograma de rueditas de M2 se cierra en este lab, y acá está el repliegue, en voz alta: las pruebas de tabla las construimos juntos línea por línea, la prueba de fake timers y la prueba de config las completas desde setups dados, y el cableado de CI se trabaja otra vez PERO tú manejas cada push. El próximo módulo los apoyos empiezan a adelgazar de verdad.

### 1. La primera prueba que falla, a propósito

Escribiste `tests/classify.test.ts` en la apertura. Ahora hazla mentirte deliberadamente, porque nunca deberías confiar en una prueba que no has visto fallar. Da vuelta la expectativa:

```ts
expect(classifyProbe('ok', 400)).toBe('up'); // wrong on purpose
```

```bash
npx vitest run
```

```
 FAIL  tests/classify.test.ts > a 400ms probe is degraded, not up
AssertionError: expected 'degraded' to be 'up'
```

Lee esa falla como lees un resultado de sonda: lectura esperada, lectura real, delta. Da vuelta la expectativa de nuevo a `'degraded'`, míralo ponerse en verde. Ese ritmo de rojo-y-después-verde es el loop de confianza, y lo vas a correr sobre el pipeline entero al final de este lab.

![Un mensaje de falla de vitest anotado para mostrar dónde aparecen el nombre de la prueba que falla, el valor real y el valor esperado.](assets/v05-annotated-code.webp)

### 2. La tabla del clasificador, fila por fila

Reemplaza la prueba única con la tabla de `test.each` de la sección de teoría, las ocho filas. Constrúyela en este orden y mira qué compra cada agregado: primero las cuatro filas `ok` (los dos lados de las dos fronteras de latencia), córrela, verde. Las dos filas `http-error` (429 versus 500, la regla que calificó el challenge), córrela, verde. Después `timeout` y el kind desconocido. Ocho filas, ocho checks verdes, y el contrato que vienes cargando en la cabeza desde m02-l1 ahora vive en algún lugar que el compilador y el runner pueden alcanzar los dos.

### 3. La prueba de backoff (el reloj lo escribes tú)

Setup dado, las aserciones son tuyas. Primero una extracción: m02-l3 dejó el loop de retry inline en `probeWithRetry`, con `Math.random()` incrustado en la línea del jitter, y un cronograma con un término aleatorio adentro no se puede fijar. Saca el loop a `src/backoff.ts` como `retryOn429(fn, { baseMs, capMs, retries, jitter, isRetryable })`: curva base determinista en el código, y las dos decisiones de criterio inyectadas en el sitio de llamada. La inyección del jitter te la esperabas; el predicado `isRetryable` es el que hace la extracción posible en absoluto, porque el helper es genérico sobre lo que sea que devuelva `fn` y no puede saber cómo se ve "ocupado" para él. La flota pasa la función de jitter equitativo más `isRetryable: (r) => r.kind === 'http-error' && r.status === 429`, mapeando el `maxRetries` de su config sobre la ortografía más corta `retries` de la opción en la llamada (el nombre de la config vive en la frontera de parseo; el helper es libre de escribir sus opciones a su manera). La prueba pasa `jitter: () => 0`, un `fn` de juguete que responde la cadena `'429'`, y un predicado de una línea que coincide, después fija valores exactos. Apunta `probeWithRetry` al nuevo helper, confirma que la flota todavía corre, y después vuelve. Acá está el harness:

```ts
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { retryOn429 } from '../src/backoff.js';

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

test('five retries wait exactly 500, 1000, 2000, 4000, 8000 ms', async () => {
  const alwaysBusy = vi.fn(async () => '429' as const);
  const run = retryOn429(alwaysBusy, {
    baseMs: 500,
    capMs: 8000,
    retries: 5,
    jitter: () => 0,
    isRetryable: (r) => r === '429',
  });

  await vi.advanceTimersByTimeAsync(0); // flush the first attempt
  expect(alwaysBusy).toHaveBeenCalledTimes(1);

  // YOUR TURN from here: advance to one ms BEFORE the first retry,
  // assert nothing fired, then land each retry on its exact instant.
  await vi.advanceTimersByTimeAsync(499);
  expect(alwaysBusy).toHaveBeenCalledTimes(1);

  await vi.advanceTimersByTimeAsync(1); // t = 500
  expect(alwaysBusy).toHaveBeenCalledTimes(2);

  // ... continue: t = 1500, 3500, 7500, 15500 ...

  await expect(run).resolves.toBe('429');
});
```

(Si tu archivo de m02-l3 escribe el helper distinto, quédate con tus nombres. La prueba fija comportamiento, no ortografía.)

Resuelve tú mismo los avances de reloj que quedan antes de correr: cada retry aterriza en el instante anterior más el siguiente delay del cronograma, así que 500, después 1500, después 3500, después 7500, después 15500. Seis llamadas en total: el primer intento más cinco retries. Cuando tus aserciones aterricen en verde, mira el tiempo de ejecución que vitest reporta para el archivo. Milisegundos. Acabas de verificar quince segundos y medio de comportamiento programado sin esperar nada de eso.

![Seis intentos de retry aterrizan en instantes exactos del reloj falso desde cero hasta 15500 milisegundos mientras el tiempo real apenas pasa.](assets/v06-timeline.webp)

### 4. La prueba de frontera de la config (el fixture se gana el sueldo)

Recrea el crimen de m02-l2 contra el schema actual de la flota: toma tu `pulse.config.json` bueno, renombra un campo obligatorio, `timeoutMillis` por `timeoutMs`, y guarda la copia saboteada como `tests/fixtures/pulse.bad.json`. Es la misma clase de typo que el original `intervalSeconds`; desde entonces el schema se reconstruyó para la flota concurrente, así que el pin apunta a un campo del que todavía es dueño. Setup dado, la aserción es tuya:

```ts
import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { configSchema } from '../src/config.js';

test('the m02-l2 typo class is refused at the boundary', () => {
  const raw = JSON.parse(
    readFileSync(new URL('./fixtures/pulse.bad.json', import.meta.url), 'utf8'),
  );

  const result = configSchema.safeParse(raw);

  expect(result.success).toBe(false);
  if (!result.success) {
    const paths = result.error.issues.map((issue) => issue.path.join('.'));
    expect(paths.join('\n')).toContain('timeoutMs');
  }
});
```

Dos aserciones, dos garantías distintas. La primera dice que el schema sí rechaza el archivo. La segunda dice que el rechazo NOMBRA el campo que falta, porque una frontera que falla sin decir dónde es apenas mejor que una que miente. Este es el pin de regresión: la mentira cortés de m02-l2 ahora no puede volver sin que esta prueba se ponga en rojo primero.

Agrega tú mismo la prueba espejo antes de seguir: un segundo fixture, `pulse.good.json` (una copia de tu config real), y una prueba que afirme que `safeParse` la ACEPTA, `expect(result.success).toBe(true)`. Hoy se siente redundante. Deja de sentirse redundante la primera vez que alguien ajusta un refinamiento y sin querer deja afuera la config de producción; una frontera que rechaza todo está igual de rota que una que admite todo, y ahora las dos direcciones tienen una sonda.

Voy a confesar el origen de mi entusiasmo acá: una vez corrí un monitor por meses con un bug de frontera del clasificador casi idéntico al del paso 1, y lo encontré no por ninguna alarma sino leyendo el log crudo sin nada que hacer un domingo. Cada reporte que había publicado en esa ventana estaba sutilmente equivocado, y cada uno se había entregado con un deploy verde al lado. No me costó nada más que confianza, que es lo caro. El hábito del fixture es lo que hago con esa memoria.

### 5. Lee la cobertura, arregla un hueco

```bash
npm i -D @vitest/coverage-v8@4.1.11
npx vitest run --coverage
```

(El paquete de cobertura se versiona en sincronía con vitest mismo; mantén los dos fijados juntos.)

Lee el reporte como una lista de targets, no como un marcador de puntajes. Mira `src/classify.ts` primero, y una nota de honestidad sobre el toolchain antes de que salgas a cazar: con los pins actuales (vitest 4.1.11 con coverage-v8 4.1.11 sobre Node 24), `src/backoff.ts` puede simplemente no aparecer en la tabla de cobertura en absoluto, incluso cuando su archivo de prueba corre, así que si la fila falta, eso es el hueco de reporte de la herramienta, no una demostración de cobertura perfecta ni de cobertura cero, y `src/classify.ts` es donde gastar el ejercicio. En algún lugar de tu flota hay una rama que la suite nunca ejecuta; en la mayoría de los builds de este proyecto es un brazo de mapeo de errores (el brazo `dns-error` del clasificador es un hallazgo común) o, si tu tabla sí muestra backoff, el borde de la fórmula con el techo por debajo de la base, el caso donde `capMs` es más chico que `baseMs` y cada delay queda aplastado plano. Encuentra TU hueco, pregúntate si un bug ahí llegaría a status.json, y si sí, agrega la fila o el caso que lo cubra. Si la línea sin cubrir es tu brazo `assertNever`, déjala sin cubrir y disfruta el recordatorio de por qué el número es una señal y no una meta.

### 6. La re-entrega: las pruebas le ponen barrera al cron

Ahora la costura del módulo: la suite se suma al pipeline que construiste en m01-l3, y este workflow es el único artefacto que este curso hace crecer hasta el capstone mismo. Las barreras de Rust se le suman en M4, los builds de release todavía después. Hoy aprende a rechazar.

Abre `.github/workflows/pulse.yml`. Estás agregando un job y una arista:

```yaml
name: pulse

on:
  push:
    branches: [main]
  schedule:
    - cron: "*/30 * * * *"

permissions:
  contents: write

jobs:
  typecheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm ci
      - run: npx tsc --noEmit

  test: # NEW: the suite as a job
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm ci
      - run: npx vitest run

  probe:
    needs: [typecheck, test] # CHANGED: was `needs: typecheck`
    runs-on: ubuntu-latest
    steps:
      # your existing probe-and-commit steps, unchanged
```

(Los tags de las actions `checkout@v7` y `setup-node@v7` fueron sondeados como las majors actuales el 2026-09-02; Node 24 es el LTS Activo al momento de escribir esto, con la v26 planificada para tomar el relevo el 2026-10-28.)

La línea estructural es `needs: [typecheck, test]` (la clave `needs:` declara dependencias de jobs: `probe` no va a arrancar a menos que cada job que nombra haya tenido éxito). Sin esa arista, el job test corre AL LADO de probe y no le pone barrera a nada; una suite en rojo y un commit fresco de status.json aterrizarían en la misma ejecución, que es una decoración, no una barrera. El diff es la lección. Léelo.

![Los triggers de push y de calendario alimentan las barreras de typecheck y de test cuyas flechas de needs apuntan las dos al job probe que le hace commit a status json.](assets/v07-flowchart.webp)

Ahora demuestra que la barrera existe, en rojo primero, porque el paso 1 te enseñó a nunca confiar en un check sin probar. Planta el bug del clasificador a propósito: da vuelta tu frontera de 400 a `> 400` en `src/classify.ts`, hazle commit a un branch, mergéalo (o haz push directo a main; es tu estación, y esta es la única ocasión para romper main deliberadamente). Mira la ejecución de Actions: el job test se pone en rojo en la fila `classifyProbe(ok, 400) is degraded`, y el job probe aparece como salteado. Abre el repo: status.json no tiene commit nuevo. El pipeline se negó a publicar la mentira. Toma el screenshot; esta ejecución en rojo es la evidencia de aceptación, y honestamente es un artefacto satisfactorio de guardar.

Después revierte el bug plantado, haz push, y mira la secuencia verde: typecheck pasa, test pasa, la sonda corre, status.json se actualiza. Una cosa más que ahora sabes y que la mayoría de la gente nunca verifica: como los workflows programados corren el último commit del branch por defecto, el SIGUIENTE tick del cron después de un merge rojo habría corrido la misma suite roja y habría rechazado otra vez, cada treinta minutos, hasta que alguien lo arreglara. Un merge rojo sin barrera se vuelve un status.json equivocado según el calendario, sin nadie mirando. Uno con barrera se vuelve una publicación estancada y una X roja que alguien va a ver. Estancado le gana a mentir. Ese es todo el diseño.

Ahora la nota honesta al pie sobre qué exactamente quedó con barrera, porque un módulo que gastó cuatro lecciones en tipos veraces no debería ser vago sobre su propio camino de publicación. El job llamado `probe` todavía corre el escritor original de `fleet.ts` de m01-l3, intacto: forma v0, `latencyMs` todavía tipado flojo, todavía lo único que escribe `status.json`. Todo a lo que le volviste a poner tipos en l1 y reconstruiste de forma concurrente en l3 vive al lado, y lo que las dos barreras protegen es ese código. Eso es deliberado, no un descuido: la forma `{ url, status, latencyMs, checkedAt }` de `status.json` es un contrato congelado que el panel de m03-l2 está por renderizar, y cambiar el publicador por debajo de un consumidor que todavía no existe es cómo rompes los dos a la vez. El escritor se recablea en m03-l2, del otro lado de la mudanza al workspace, como la primera jugada de lab de esa lección, una vez que hay un schema de panel en pantalla para mantener en verde mientras lo haces.

Verifica local y remotamente antes de seguir: `npx vitest run` verde en tu máquina, y la ejecución de Actions del commit que empujaste mostrando el job test completándose antes de que arranque el step de probe.

## Challenge

No hay challenge calificado en esta lección; los más fuertes del módulo viven en l1 y l3, y escribir pruebas se demuestra con la suite a la que el lab le acaba de poner barrera. En cambio, una rep sin guía con algo de verdad en juego: planta un bug DISTINTO, uno que la suite actual NO atrape. Rompe el sitio de llamada del jitter, o haz que el schema de config acepte un `timeoutMs` negativo, y confirma que el pipeline se queda verde todo el camino hasta un status.json publicado. Quédate un rato con cómo se siente eso. Después escribe la prueba que lo habría atrapado, mírala fallar contra el bug plantado, revierte el bug, mírala pasar, y deja la prueba puesta. Acabas de hacer el loop profesional completo: encuentra el target que nadie mira, apúntale una sonda, quédate con la sonda. Repite para siempre, en cada trabajo que tengas en tu vida.

## Dónde deja esto a la estación

La victoria de 30 segundos antes de cerrar la terminal: en una oración, ¿por qué tiene que ponerle barrera al CRON el job test, específicamente, y no solo correr en los pushes? Dilo en voz alta. Si tu oración contiene "branch por defecto", "según el calendario" y "status.json equivocado sin nadie mirando", tienes la lección entera. Y la promesa hecha hacia adelante, textualmente para que la reconozcas cuando aterrice: en M4 vas a conocer cargo test. Misma idea, flag distinta.

Si la suite atrapó un bug de verdad tuyo hoy, aunque sea uno cercano a los plantados, de verdad quiero que me lo cuentes; ese primer rescate es el momento en que este hábito deja de ser tarea.

La flota está tipada, parseada, disciplinada y auto-monitoreada, y todo eso vive en un único árbol de archivos que crece y que solo tú puedes usar. El próximo módulo el motor se convierte en un paquete de verdad: workspaces, package.json como contrato, un panel de React, y la primera URL que un desconocido puede abrir. Las rueditas se salen en la puerta del workspace.
