# Async que sobrevive al contacto: límites, backoff, cancelación

## Resumen

m02-l2 le puso un parser a cada frontera: la config está pasada por zod (tipada por `z.infer`, refinada, verificada con `satisfies`), y la flota hasta parseó una respuesta real de `getBalance` con los lamports como bigint. Los datos ya no pueden colarse malformados. Pero la flota todavía sondea un target a la vez, y en el momento en que la apuntas a cincuenta targets al mismo tiempo, descubres que internet tiene opiniones sobre cómo preguntas. Esta lección construye a mano la disciplina de concurrencia de la flota: un pool de workers que acota cuántas sondas están en vuelo, backoff exponencial con jitter para los 429s, timeouts de AbortController que convierten sockets colgados en resultados tipados, y un reporte agregado donde cada target termina en exactamente una variante de `ProbeResult`. Vas a causar un muro de 429s a propósito, después lo vas a hacer desaparecer, y vas a medir las dos cosas.

## La concurrencia es un presupuesto

Causa el problema primero. Nada que instalar hoy; todo corre sobre lo que ya tienes (Node 24 LTS de m01-l2, `tsx` como el runner, zod de la lección pasada). Dos archivos, tres minutos.

Primero, un target que tienes permiso para machacar. Este es un servidor local que se comporta como cada API rate-limited que te vayas a encontrar en tu vida: sirve un número tope de requests por ventana, después responde 429 hasta que la ventana da la vuelta. Guárdalo como `src/limited-server.ts`:

```ts
// A local target that behaves like every rate-limited API you will ever meet.
// CAP requests per WINDOW_MS, then 429s until the window rolls over.
import { createServer } from "node:http";

const CAP = Number(process.env.CAP ?? 25);
const WINDOW_MS = Number(process.env.WINDOW_MS ?? 1000);
const LATENCY_MS = Number(process.env.LATENCY_MS ?? 250);

let windowStart = Date.now();
let seen = 0;

const server = createServer((req, res) => {
  const now = Date.now();
  if (now - windowStart >= WINDOW_MS) {
    windowStart = now;
    seen = 0;
  }
  seen += 1;
  if (seen > CAP) {
    res.writeHead(429, { "content-type": "application/json", "retry-after": "1" });
    res.end(JSON.stringify({ error: "too many requests" }));
    return;
  }
  setTimeout(() => {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify({ ok: true, path: req.url }));
  }, LATENCY_MS);
});

server.listen(8787, () => {
  console.log(`limited target on http://localhost:8787 (cap ${CAP}/${WINDOW_MS}ms, latency ${LATENCY_MS}ms)`);
});
```

Arráncalo en una terminal (`npx tsx src/limited-server.ts`) y déjalo corriendo. Ahora la flota ingenua, `src/burst.ts`:

```ts
// The naive fleet: fifty probes, one instant.
const targets = Array.from({ length: 50 }, (_, i) => `http://localhost:8787/t/${i}`);

const statuses = await Promise.all(
  targets.map(async (url) => {
    const res = await fetch(url);
    return res.status;
  }),
);

const walls = statuses.filter((s) => s === 429).length;
console.log(`429s: ${walls} / ${targets.length}`);
```

Córrelo:

```bash
npx tsx src/burst.ts
```

En mi máquina:

```text
429s: 25 / 50
```

La mitad de la flota quedó rechazada. Mira lo que pasó del lado del target: cincuenta requests llegaron en el mismo instante. El servidor admite 25 por segundo, así que los primeros 25 pasaron y los otros 25 chocaron contra el muro, todo adentro de un solo tick del loop de eventos. La flota que existe para medir disponibilidad acaba de volverse ella misma el problema de disponibilidad. Tu "monitoreo" llegó con la forma exacta de un ataque, y el servidor lo trató como tal.

Acá está la oración que toda esta lección desarma: la concurrencia es un presupuesto que gastas, no una velocidad que recibes. La versión ingenua gastó el presupuesto entero en un instante. Hoy aprendes a dosificarlo.

### Cinco minutos sobre el modelo de promises

Con el tiempo acotado, un diagrama, y seguimos. Si vienes de un lenguaje solo-síncrono (Python sin asyncio, PHP, Java del de toda la vida), este es el modelo mental sobre el que se para todo lo de abajo. Si las promises ya te resultan cómodas, pasa rápido al diagrama y sigue.

Una promise es un valor que representa un resultado que todavía no existe. No el resultado: el ticket de reclamo por él. Está en exactamente uno de tres estados: pending (trabajo en vuelo), fulfilled (acá está tu valor), o rejected (acá está tu error). Se asienta una vez, de una sola forma, y no cambia nunca más.

`await` es donde el cerebro síncrono se quema, así que dilo con precisión: `await` suspende esta función hasta que la promise se asienta. No bloquea el programa. La función se estaciona sola, el loop de eventos sigue corriendo todo lo demás (timers, otros fetches, el servidor que acabas de escribir), y cuando la promise se asienta, la función retoma desde esa línea exacta con el valor en la mano. Un thread, muchas funciones suspendidas, e I/O que se superpone porque nadie está parado esperando un socket.

![Una promise pasa de pending a fulfilled o a rejected, mientras await pausa solo la función que espera y el loop de eventos sigue corriendo.](assets/v01-diagram.webp)

Ese es todo el repaso. Si algo de eso te resultó nuevo en vez de oxidado, esta lección va a seguir acá mañana: la caja de honestidad de m01-l1 apuntaba al track Learn Core Scripting de MDN (developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting) exactamente por esta razón, y sus lecciones de async son la ruta respetable más rápida hacia la alfabetización en promises. Haz esas, vuelve, y todo lo de abajo se va a leer con la mitad del esfuerzo.

### Promise.all no arranca nada

Ahora la ráfaga de la apertura, explicada en una línea: para cuando `Promise.all` corre, cada sonda ya arrancó.

La gente trata a `Promise.all` como un scheduler, algún dispatcher inteligente que va a ir soltando requests a un ritmo razonable. No lo es. Es un join. El `targets.map(...)` creó cincuenta promises, lo que quiere decir que cincuenta llamadas a `fetch` ya se dispararon, en el mismo instante síncrono, antes de que `Promise.all` siquiera recibiera el array. Todo lo que hace el join es esperar a todas y pasarte los resultados en orden. La estampida pasó en `.map`. `Promise.all` solo miró.

Una propiedad más mientras estamos siendo precisos, porque decide la forma de agregación de la flota. `Promise.all` es todo-o-nada: en el momento en que cualquier promise se rechaza, el join entero se rechaza con ese primer error, y los otros cuarenta y nueve resultados, incluidos los que ya habían tenido éxito, simplemente se perdieron. Para una flota cuyo trabajo entero es "un resultado por cada target", eso está exactamente al revés; una búsqueda de DNS flaky no debería vaporizar cuarenta y nueve mediciones buenas. La respuesta de la plataforma es `Promise.allSettled`, que espera todo y te pasa un objeto wrapper por promise, `{ status: "fulfilled", value }` o `{ status: "rejected", reason }`, donde `reason` está tipado como un unknown que igual tienes que interrogar. La nuestra es mejor para este codebase, y ya la construiste: sondas que nunca se rechazan, porque cada desenlace aterriza en la unión `ProbeResult` con su propio brazo nombrado y su propio payload tipado. Mismo espíritu de allSettled, sin ningún `reason` sin tipar que haya que pescar. Ten presente que `allSettled` existe para el día en que estés agregando promises que no controlas; adentro de la flota, la unión es la agregación.

![Cincuenta requests simultáneos desbordan un tope de tasa y la mitad rebota, mientras que los mismos cincuenta en olas de cinco pasan todos.](assets/v02-comparison.webp)

### El pool: N workers, una fila

El arreglo es vergonzosamente chico, y construirlo a mano es el punto. Librerías como `p-limit` existen y están bien; después de hoy vas a saber exactamente qué hacen, que son unas doce líneas:

```ts
export async function probeAll(targets: string[], config: FleetConfig): Promise<ProbeReport[]> {
  const reports = new Array<ProbeReport>(targets.length);
  let next = 0;

  async function worker(): Promise<void> {
    while (next < targets.length) {
      const i = next;
      next += 1;
      const url = targets[i]!; // i < length, but noUncheckedIndexedAccess can't see it
      reports[i] = await probeWithRetry(url, config);
    }
  }

  const size = Math.min(config.concurrency, targets.length);
  await Promise.all(Array.from({ length: size }, worker));
  return reports;
}
```

Léelo como una obra: una fila compartida de trabajo (`next` es apenas un índice sobre los targets), y `size` workers, cada uno corriendo un loop de "agarra el próximo índice, haz la sonda, guarda el resultado en ese índice, repite". Cada worker es una función async, así que mientras su sonda está suspendida en `await`, las sondas de los otros workers también están en vuelo. Como mucho existen `size` sondas en cualquier instante. Los mismos cincuenta targets, el mismo trabajo total, pero la tasa de ráfaga ahora está acotada por un número que elegiste.

Fíjate que `Promise.all` volvió, y ahora se usa para lo que es: un join sobre exactamente `size` promises de worker, no cincuenta fetches sin acotar. Y fíjate que los workers nunca lanzan. `probeWithRetry` (lo construimos a continuación) devuelve un resultado tipado para cada desenlace, así que un rechazo nunca puede derribar el join. Esa es la forma de agregación que la flota necesita: cada target termina en exactamente un `ProbeResult`, fallas incluidas.

Ya que estamos con los rechazos, una trampa específica de Node se merece su propio párrafo, porque no falla con cortesía. A una promise que nadie espera se le dice fire-and-forget, y cuando se rechaza, no hay ningún catch en toda su cadena. La respuesta por defecto de Node ante un rechazo no manejado es imprimir el error y matar el proceso. No la sonda. El proceso. Una flota de monitoreo que se muere porque la sonda 37 se topó con un hipo de DNS que nadie estaba escuchando es un reporte de incidente genuinamente vergonzoso, y yo escribí una versión más suave: un scraper mío de los primeros corrió bien dos días, después un solo retry sin esperar se rechazó a las 3am y se llevó el loop entero abajo con él. La estructura de pool que acabas de leer es la cura tanto como el medidor: cada promise de sonda se crea adentro de un worker, cada worker es esperado por el join, así que cada rechazo tiene dónde aterrizar. Si alguna vez te descubres escribiendo `void somePromise()` o llamando a una función async sin esperarla ni recolectarla, para y pregúntate quién es dueño de esa promise cuando se rechaza. En esta flota la respuesta es siempre: el agregado.

Dos notas honestas sobre ese snippet. El contador `next` es seguro sin locks porque JavaScript es de un solo thread: las dos líneas que lo leen y lo incrementan corren sincrónicamente, y ningún otro worker puede intercalarse entre ellas. Ese razonamiento es un regalo del modelo de loop de eventos, disfrútalo, no viaja a Rust. Y el `!` sobre `targets[i]` somos nosotros pasando por encima de `noUncheckedIndexedAccess` de nuestro tsconfig estricto: la flag no puede demostrar `i < targets.length` a través de las dos sentencias, nosotros sí, y un comentario de una línea lleva la demostración.

La perilla importa más que el mecanismo. ¿Cuánto debería ser `concurrency`? Acá está el replanteo que separa a la gente que ya quedó rate-limited de la gente que está por quedar: el pool no se dimensiona según tu máquina. Node va a sostener miles de sockets sin quejarse. La restricción es el presupuesto del target, el tope publicado de quien sea que estés sondeando. Un pool de 5 contra nuestro tope local de 25/seg, con cada request tardando 250ms, produce como mucho 20 requests por segundo: por debajo del muro por diseño, no por suerte. Tu CPU nunca entró en el cálculo.

![Cinco loops de worker idénticos sacan índices de una fila compartida y alimentan un solo join que produce un resultado por target.](assets/v03-flowchart.webp)

### Backoff, y por qué el jitter no es opcional

El pool acota tu tasa de ráfaga, pero los topes se chocan igual: otro proceso comparte tu IP, la ventana se monta sobre tus olas, alguien baja el tope un martes. Cuando llega un 429, la respuesta cortés es esperar y reintentar, y el cronograma de esas esperas es donde pasa la ingeniería.

El cronograma canónico es exponencial: espera un delay base, después duplícalo en cada falla sucesiva, con un techo para que no pueda crecer para siempre. El intento 0 espera `baseMs`, el intento 1 espera `2 * baseMs`, el intento n espera `min(capMs, baseMs * 2^n)`. Con una base de 500ms y un techo de 5000ms, el cronograma corre 500, 1000, 2000, 4000, 5000. La duplicación le da al target lugar para respirar; el techo impide que una caída larga produzca esperas de horas. Determinista, diez líneas, lo vas a escribir tú mismo en el challenge:

```ts
export function backoffDelay(attempt: number, baseMs: number, capMs: number): number {
  return Math.min(capMs, baseMs * 2 ** attempt);
}
```

Ahora deriva la pieza que falta en vez de memorizarla. Imagínate el desastre de tope bajo: cinco workers disparan, los cinco reciben 429 en la misma ventana, porque el mismo tope los rechazó a todos de una. Los cinco calculan el mismo delay de intento 0, de 500ms. Los cinco duermen 500ms. Los cinco despiertan en el mismo instante y disparan otra vez, una estampida sincronizada de cinco, contra el mismo tope que acaba de rechazar una estampida de cinco. Fallan juntos, duermen 1000ms juntos, estampida otra vez. Las fallas sincronizadas reintentan sincronizadas, y la manada vuelve a disparar el límite exacto que la creó, para siempre. El cronograma es perfecto y la flota nunca se vacía.

El arreglo es ruido. Jitter quiere decir que cada worker aleatoriza su espera alrededor del delay programado, así la manada se esparce a lo largo de la ventana en vez de llegar como una sola. Usamos el sabor común de "jitter equitativo" en el sitio de llamada: quédate con la mitad del delay, aleatoriza la otra mitad.

```ts
const delay = backoffDelay(attempt, baseMs, capMs);
const jittered = delay / 2 + Math.random() * (delay / 2);
await sleep(jittered);
```

Fíjate que el jitter no hizo nada más rápido. Esparcida alrededor del delay base, la espera promedio es más o menos la que era. El jitter no es una herramienta de latencia, es una herramienta de desincronización: existe para impedir que tus propios clientes coordinen un ataque accidental contra la cosa que están reintentando. Más adelante, en la corrida del lab con tope bajo, vas a ver el borrón en tu propio log: un grumo de 429s aterriza junto, y los retries vuelven dispersos (los míos aterrizaron en 437, 282 y 382ms) en vez de en un solo bloque. Tus dígitos exactos van a diferir, eso es `Math.random()` haciendo su único trabajo; la forma, que no haya dos esperas iguales, es lo que estás buscando.

![Los retries sin jitter llegan en grumos simultáneos repetidos que fallan todos, mientras que los retries con jitter se esparcen a lo largo de la línea de tiempo y tienen éxito.](assets/v04-timeline.webp)

Una regla más antes de cablearlo, porque acá es donde el trabajo de tipos de l1 paga renta: la política de retry es por variante. Un 429 es el servidor diciendo "ahora no", así que se gana el backoff. Un 404 es el servidor diciendo "nunca", y reintentarlo es un bug que cuesta cinco delays descubrir. Un timeout es ambiguo y para una flota de estado la jugada honesta es registrarlo y dejar que la próxima ronda programada decida. La unión discriminada es lo que vuelve esta política expresable como código en vez de como sensaciones: haz match en la variante, reintenta exactamente una de ellas.

### AbortController: un timeout es una cancelación

El último modo de falla es el peor: el target que ni responde ni rechaza. Un socket colgado te mantiene secuestrado el slot del pool; con cinco workers, cinco sockets colgados son una flota muerta. Los timeouts son cómo un slot recupera su vida, y en `fetch` un timeout se escribe AbortController.

El cableado son tres jugadas: crea un controller, pásale su signal a `fetch`, y arregla que `abort()` se llame cuando el presupuesto se vence. El abort hace que el `fetch` en vuelo se rechace, y nosotros capturamos ese rechazo y lo convertimos en un desenlace tipado de primera clase:

```ts
async function probeOnce(url: string, timeoutMs: number): Promise<ProbeResult> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  const started = performance.now();
  try {
    const res = await fetch(url, { signal: controller.signal });
    if (res.ok) {
      return { kind: "ok", latencyMs: Math.round(performance.now() - started) };
    }
    return { kind: "http-error", status: res.status };
  } catch {
    if (controller.signal.aborted) {
      return { kind: "timeout", budgetMs: timeoutMs };
    }
    return { kind: "dns-error", host: new URL(url).hostname };
  } finally {
    clearTimeout(timer);
  }
}
```

Recorre las salidas, porque cada una de ellas es una lección. El camino feliz devuelve `ok` con una latencia medida. Una respuesta que no es 2xx devuelve `http-error` con el status (el loop de retry de arriba decide si un 429 se gana otro intento). Si el bloque catch encuentra `controller.signal.aborted` en true, el rechazo fue nuestro propio timer disparándose, y se convierte en la variante `timeout` con el presupuesto que reventó, no en un rechazo no manejado traqueteando hacia arriba por la stack. Cualquier otra cosa en el catch es la red misma fallando (DNS, conexión rechazada), y aterriza en el brazo `dns-error` que agregaste en el ejercicio de l1. Si ese nombre te molesta ahora que las fallas de conexión también viven ahí, renómbralo `network-error` y deja que el compilador te camine hasta cada switch que necesite actualización. Que ese mandado cueste minutos en vez de una tarde es exactamente lo que compraste en m02-l1.

Y el `finally`: `clearTimeout` corre en cada salida. Sáltatelo y la sonda funciona igual, que es lo que vuelve a esta trampa tan bien escondida. El timer sobrevive al request terminado, se dispara más tarde, y aborta un controller que nadie está usando. Inofensivo hoy; después alguien reusa el controller, o el proceso debería haber salido y no salió porque había un timer pendiente. La limpieza es parte del patrón, no un adorno.

![Cuatro líneas anotadas que muestran la creación del controller, el cableado del timer, el pase del signal, y el clearTimeout en finally que se olvida fácil.](assets/v05-annotated-code.webp)

Ahora la parte honesta, y el trade-off de verdad de esta lección. La cancelación es cooperativa y local. Abortar el fetch libera tu slot, asienta tu promise, y le da a tu reporte una fila tipada limpia. No cruza el cable para des-mandar nada: el request que ya mandaste igual puede completarse en el servidor. Yo medí esto en el lab que estás por correr: con el timeout apretado a 100ms, las cincuenta sondas volvieron `timeout`, y el servidor igual quemó presupuesto sirviendo requests que nadie estaba esperando, lo bastante como para que se dispararan 10 retries en el camino. Cada disciplina de esta lección cambia latencia por cortesía. El pool termina en diez olas en vez de una. El backoff vuelve más lentos para reportar a los targets que fallan. Un timeout convierte a un target lento-pero-vivo en una falla declarada en un umbral que elegiste tú. No hay un tamaño de pool ni un timeout correctos, hay el presupuesto del target, tu deadline, y una perilla; el pecado es no saber contra qué límite estás negociando.

![Los cuatro desenlaces posibles de una sonda, éxito, error HTTP, timeout, y falla de red, fluyen cada uno hacia una fila de resultado tipada, con solo el camino del 429 volviendo en loop por el backoff.](assets/v06-flowchart.webp)

### El muro bajo el que esta flota vive de verdad

Todo hasta acá usó un tope de juguete para que pudieras medirlo sin molestar a nadie. Ahora el número real, porque esta flota apunta a infraestructura de Solana desde M8 en adelante. Los endpoints públicos de RPC de Solana publican sus límites en la referencia oficial de clusters (solana.com/docs/references/clusters, verificada el 2026-09-02): 100 requests cada 10 segundos por IP, y 40 cada 10 segundos para cualquier método de RPC individual, con 40 conexiones concurrentes por IP y 100 MB de datos cada 30 segundos. La misma página dice sin vueltas que estos endpoints "no están pensados para aplicaciones de producción". Esos cuatro números son un presupuesto de target, exactamente como lo era `CAP=25`, y el único fetch de `getBalance` de la lección pasada ya vivía debajo de ellos sin saberlo.

Haz la aritmética como dimensionarías cualquier pool. Una flota de estado que sondea vía `getBalance` quema primero el presupuesto por método: 40 cada 10 segundos. Cincuenta targets a través de un pool de 5 con nuestras latencias de 250ms empujarían 20 requests por segundo, cinco veces por encima de ese tope por método. La misma flota con `concurrency: 3` y una pausa modesta por ronda se queda por debajo. El punto no son estos dígitos en particular; el punto es que la perilla tiene un input correcto, y es el presupuesto publicado del target, nunca el apetito de tu máquina.

Acá está el hábito que este curso no para de ejercitar, y vale la pena nombrarlo como hábito: metas documentadas, realidad medida. Solana apunta a slots de 300ms y una sonda de 20 muestras el 2026-09-01 midió 316ms de promedio. Los docs dicen 100 requests cada 10 segundos; tu log dice dónde arrancaron los 429s de verdad. Los sistemas publican intenciones, y los ingenieros las verifican con sus propios instrumentos. Hoy tu instrumento es un contador en una flota de cincuenta líneas. En M8 vas a apuntar el mismo hábito a la blockchain misma y construir un medidor que mide tiempos de slot en vivo.

Una frontera, dicha sin vueltas para que nadie sobre-aplique los patrones de hoy: todo en esta lección son modales del camino de lectura, la etiqueta de los GETs y las lecturas de JSON-RPC que puedes volver a mandar a ciegas. Reintentar una transacción es otro deporte con otra cosa en juego (¿el primer envío de verdad aterrizó?), y esa profundidad, el aterrizaje de transacciones y todo lo que está cerca de las billeteras, es territorio del curso de dominio del lado del cliente; ese curso está en producción mientras escribo esto, y hasta que se entregue, el puntero honesto es el tema mismo. Las sondas son idempotentes; los pagos no; no portes este loop de retry al dinero.

**Profundiza (el 20%).** todo lo de acá fue la capa de uso diario. Los internals del loop de eventos, microtasks contra macrotasks, los iteradores async y `for await`, los combinadores de promises más allá de `all`: guarda como bookmark la unidad Asynchronous JavaScript de MDN (developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Async_JS, gratis, verificada en vivo el 2026-09-02) y profundiza cuando un bug te mande. Esta lección deliberadamente no vuelve a enseñar lo que esas páginas ya tienen.

## Lab: la flota, concurrente y cortés

El repliegue que este módulo viene corriendo continúa: el pool y la ráfaga quedaron completamente trabajados arriba; el loop de backoff del paso 3 es un completion, esqueleto dado, dos órganos tuyos; el cableado del abort del paso 4 es evocación, escrito de memoria contra un listado que ya leíste; el challenge de después es solo tuyo.

1. **Reconstruye el schema de config para la flota concurrente.** Las perillas de la flota van en `pulse.config.json`, detrás del parser de la lección pasada, no hardcodeadas. El lab sondea una lista plana de URLs locales bajo un presupuesto compartido, así que remodela el schema de l2 en `src/config.ts`: los targets se vuelven URLs planas, `timeoutMs` se mueve al nivel superior, y dos campos nuevos, `concurrency` y `retry`, llevan las perillas. Misma disciplina de fronteras, forma nueva, sigue siendo `strictObject` porque una config es una forma de la que eres dueño y la regla de l2 se sostiene: una clave desconocida adentro es un error, no un encogimiento de hombros. Y `z.infer` actualiza `FleetConfig` gratis:

   ```ts
   import { z } from "zod";

   const retrySchema = z.strictObject({
     maxRetries: z.number().int().min(0),
     baseMs: z.number().int().positive(),
     capMs: z.number().int().positive(),
   });

   export const configSchema = z.strictObject({
     targets: z.array(z.url()).min(1),
     timeoutMs: z.number().int().positive(),
     concurrency: z.number().int().min(1).max(50),
     retry: retrySchema,
   });

   export type FleetConfig = z.infer<typeof configSchema>;
   ```

   Tres consecuencias de la remodelación, atendidas ahora para que `npx tsc --noEmit` y tu barrera de CI de m01-l3 se queden en verde en vez de podrirse en silencio. Primero, mantén `parseOrExit` en `src/config.ts` cuando cambies el schema; el snippet de arriba muestra solo lo que cambia, y tanto el paso 5 de este lab como tus scripts de l2 siguen importando el helper. Segundo, `src/check-config.ts` imprime `config.fleetName` y `t.intervalSecs`, campos que el schema nuevo ya no tiene, así que o lo recortas a la forma nueva (una línea: conteo de targets, tamaño del pool, timeout) o lo borras junto con `pulse.config.broken.json` y, si su schema peleó con la remodelación, con el `src/check-status.ts` de tu challenge de l2; esos eran los props de enseñanza de l2, y la disciplina que enseñaban ahora vive adentro de la flota misma. Tercero, fíjate qué se fue calladamente de la config y por qué: el `intervalSecs` por target (y el refine construido sobre él) no tiene a qué engancharse en una flota que sondea cada target en una sola ronda compartida; la cadencia ahora le pertenece al cron que dispara la ronda, no a los targets individuales, y el `timeoutMs` compartido es el presupuesto que sobrevivió.

2. **Genera la config del lab.** Cincuenta targets locales, pool de 5, el cronograma de backoff de la sección de teoría. Un generador descartable le gana a escribir cincuenta URLs a mano:

   ```ts
   // src/make-targets.ts
   import { writeFileSync } from "node:fs";

   const targets = Array.from({ length: 50 }, (_, i) => `http://localhost:8787/t/${i}`);
   const config = {
     targets,
     timeoutMs: 3000,
     concurrency: 5,
     retry: { maxRetries: 5, baseMs: 500, capMs: 5000 },
   };
   writeFileSync("pulse.config.json", JSON.stringify(config, null, 2));
   console.log("wrote pulse.config.json");
   ```

   Corre `npx tsx src/make-targets.ts` una vez.

3. **Escribe el loop de retry (completion).** En `src/fleet.ts`, el loop de abajo viene con dos huecos. Todo alrededor está completo; llénalos desde la sección de teoría sin volver a scrollear si puedes.

   ```ts
   async function probeWithRetry(url: string, config: FleetConfig): Promise<ProbeReport> {
     const { maxRetries, baseMs, capMs } = config.retry;
     let retries = 0;
     for (let attempt = 0; ; attempt++) {
       const result = await probeOnce(url, config.timeoutMs);
       // TODO 1: set `retryable` from the variant. Retry policy is per-variant,
       // and exactly one of the four earns another attempt.
       if (!retryable || attempt >= maxRetries) {
         return { url, result, retries };
       }
       retries += 1;
       // TODO 2: compute `jittered` from `backoffDelay(attempt, baseMs, capMs)`
       // run through the equal-jitter line: keep half, randomize the other half.
       console.log(`  429 from ${url}: attempt ${attempt}, waiting ${Math.round(jittered)}ms`);
       await sleep(jittered);
     }
   }
   ```

   Las versiones llenas, una vez que hayas escrito las tuyas:

   ```ts
   const retryable = result.kind === "http-error" && result.status === 429;
   ```

   ```ts
   const delay = backoffDelay(attempt, baseMs, capMs);
   const jittered = delay / 2 + Math.random() * (delay / 2);
   ```

   (`ProbeReport` es `{ url: string; result: ProbeResult; retries: number }`: la unión de l1 llevando su target y su costo. `sleep` es el de dos líneas `new Promise((resolve) => setTimeout(resolve, ms))`. Una nota de cableado antes de que el compilador pregunte: declara tú mismo la unión `ProbeResult` de cuatro variantes y este tipo `ProbeReport` al principio de `src/fleet.ts`. Todavía no hay nada que importar, a propósito: la unión de l1 vive en el `probe.ts` de la raíz, que es un script de CLI, no un módulo, así que la flota se lleva su propia copia local hoy. m02-l4 mueve la copia canónica a `src/classify.ts` y M3 la extrae a un paquete; esta copia local es la duplicación que motiva las dos cosas.)

4. **Cablea el abort (de memoria).** `probeOnce` está impreso completo en la sección de teoría, así que este es evocación, no completion: cierra esta página o scrollea más allá de ella y escribe la función en `src/fleet.ts` tú mismo desde las tres jugadas, la creación del controller, el `signal` en las opciones del fetch, y el `clearTimeout` en `finally`, más el mapeo de salidas (`signal.aborted` en el catch se vuelve la variante `timeout`, todo lo demás en el catch se vuelve `dns-error`). Después vuelve a scrollear y haz el diff del tuyo contra el listado. La línea que lo más probable es que te hayas saltado es el `clearTimeout`, y la sección de teoría dice por qué esa se esconde.

5. **Ensambla y corre.** `probeAll` de la sección de teoría más un pie de CLI chico: parsea la config con el `parseOrExit` de l2, llama a `probeAll`, después pliega los reportes en conteos. Un objeto indexado por el kind de la variante es la versión rápida que se muestra abajo; reescribir el pliegue como el switch exhaustivo de l1 con `assertNever` es la versión más sólida, y vale los cinco minutos.

   ```ts
   const path = process.argv[2];
   if (!path) {
     console.error("usage: npx tsx src/fleet.ts pulse.config.json");
     process.exit(1);
   }

   const config = parseOrExit(configSchema, JSON.parse(readFileSync(path, "utf8")));

   const startedAt = performance.now();
   const reports = await probeAll(config.targets, config);
   const elapsed = Math.round(performance.now() - startedAt);

   const counts = { ok: 0, timeout: 0, "http-error": 0, "dns-error": 0 };
   let retriesTotal = 0;
   let finished429 = 0;
   for (const { result, retries } of reports) {
     counts[result.kind] += 1;
     retriesTotal += retries;
     if (result.kind === "http-error" && result.status === 429) finished429 += 1;
   }

   console.log(`${reports.length} targets in ${elapsed}ms with pool of ${config.concurrency}`);
   console.log(`ok: ${counts.ok}  timeout: ${counts.timeout}  http-error: ${counts["http-error"]}  dns-error: ${counts["dns-error"]}`);
   console.log(`429s in final report: ${finished429}  (retries spent absorbing them: ${retriesTotal})`);
   ```

   Una línea más se suma al pie, y es un regreso a casa. El challenge compañero de m01-l2, `latencyStats`, se vendió con una promesa: esa función exacta se entrega adentro de la flota. Se entrega ahora. Pega tu solución calificada en `src/fleet.ts` (o escríbela de cero desde el mismo contrato: min, max, media redondeada a dos decimales, p95 de nearest-rank), después pliega las sondas exitosas a través de ella debajo de los conteos. Su contrato toma una cadena separada por comas, así que el adaptador es un solo `join`:

   ```ts
   const okLatencies = reports.flatMap(({ result }) =>
     result.kind === "ok" ? [result.latencyMs] : [],
   );
   if (okLatencies.length > 0) {
     const s = latencyStats(okLatencies.join(","));
     console.log(`latency: min ${s.min}  max ${s.max}  mean ${s.mean}  p95 ${s.p95}  (ms, over ${okLatencies.length} ok probes)`);
   }
   ```

   La función de stats que construiste en m01-l2, ahora de turno en la estación: una muestra era ruido, y cincuenta por barrido es exactamente el batch que fue construido para resumir. La guarda de `length` no es cortesía, y vale la pena saber exactamente de qué te salva. Su starter dice "se garantiza que el input no está vacío", y esa promesa es del que llama para mantenerla: corre `latencyStats([].join(","))` y no te da ningún error, te da `{ min: 0, max: 0, mean: 0, p95: 0 }`, porque `"".split(",")` es `[""]` y `Number("")` es `0`. Así que un barrido donde cada una de las sondas falló imprimiría una línea confiada de `min 0 max 0 mean 0 p95 0`, el tipo de error más peligroso: un monitor reportando latencia perfecta para una flota que no respondió nada. Saltarse la línea es el reporte honesto. Las funciones de frontera heredan sus precondiciones de quien sea que las llame, y esta guarda es donde esa se mantiene.

   Con el servidor de la apertura todavía corriendo:

   ```bash
   npx tsx src/fleet.ts pulse.config.json
   ```

   Mi ejecución:

   ```text
   50 targets in 2562ms with pool of 5
   ok: 50  timeout: 0  http-error: 0  dns-error: 0
   429s in final report: 0  (retries spent absorbing them: 0)
   latency: min 251  max 279  mean 254.7  p95 263  (ms, over 50 ok probes)
   ```

   La dispersión de latencia se sienta un pelo por encima del piso de 250ms del servidor, siendo el setup de conexión lo que es; tus dígitos van a diferir, la forma no.

   Ponlo al lado de la ejecución de ráfaga de la apertura y su `429s: 25 / 50`. Los mismos cincuenta targets, el mismo servidor, el mismo tope. Lo único que cambió es quién acota el trabajo en vuelo: nadie, o tú. Ese lado-a-lado es el artefacto de esta lección y tu barrera de verificación: cincuenta targets, un resultado tipado cada uno, cero 429s.

6. **Haz visible el backoff.** Cero retries es una victoria aburrida, así que baja el muro hasta que la disciplina tenga que trabajar. Reinicia el servidor con un quinto del presupuesto (`CAP=5 npx tsx src/limited-server.ts`), corre la flota otra vez, y lee el log mientras pelea:

   ```text
   429 from http://localhost:8787/t/6: attempt 0, waiting 437ms
   429 from http://localhost:8787/t/5: attempt 0, waiting 282ms
   429 from http://localhost:8787/t/8: attempt 0, waiting 382ms
   429 from http://localhost:8787/t/7: attempt 1, waiting 816ms
   429 from http://localhost:8787/t/8: attempt 2, waiting 1920ms
   ```

   Ahí está toda la sección de teoría en cinco líneas de log: las esperas del intento 0 se agrupan alrededor de la base de 500ms, las del intento 1 alrededor del doble de eso, el intento 2 duplica otra vez, y no hay dos esperas iguales porque el jitter esparció la manada. Mi ejecución terminó las cincuenta en 13647ms con 59 retries absorbidos y, otra vez, cero 429s en el reporte final. Más lento, y civilizado: ese es el trade-off que elegiste cuando pusiste las perillas.

7. **Dos ejercicios de falla, treinta segundos cada uno.** Mata el servidor y corre la flota: cincuenta filas de `dns-error`, al instante, sin crash, porque cada salida está tipada. Reinícialo, pon `timeoutMs` en 100 en la config (por debajo de la latencia de 250ms del servidor), corre otra vez: cincuenta filas de `timeout` en unos 2.3 segundos, sin que el pool se trabe nunca porque cada abort liberó su slot. Una flota que reporta sus fallas con la misma forma calma que sus éxitos es la cosa contra la que se construyen las pruebas de la próxima lección.

![A lo largo de tres ejecuciones medidas la ráfaga ingenua falla la mitad de sus sondas mientras que las dos ejecuciones con pool no fallan ninguna, pagando en cambio con tiempos de reloj más largos.](assets/v07-chart.webp)

## Challenge: el cronograma de backoff, exacto

La rep sin guía. El grader te pasa un cronograma ingenuo, del tipo que le ganaría a una flota su ban si alguna vez lo entregaras: empieza a duplicar de inmediato, así que la primera espera es el doble de la base, y nunca aplica el techo, así que los intentos tardíos esperan absurdamente largo. Arregla las dos cosas. El contrato: el intento n (basado en 0) espera `min(capMs, baseMs * 2^n)` milisegundos, los delays unidos en una cadena separada por comas, y `retries = 0` devuelve la cadena vacía. El cronograma se queda determinista; no hay `Math.random()` en la función calificada, y el archivo starter dice por qué: el jitter vive en el sitio de llamada para que el cronograma mismo se pueda probar hasta el dígito, que es precisamente lo que hacen las seis pruebas. Mira el borde que miran las pruebas: un techo por debajo de la base aplasta cada delay contra el techo.

Si quieres el chequeo de cordura de quince segundos antes de enviar, base 500 y techo 5000 sobre cinco retries tiene que imprimir `500,1000,2000,4000,5000`, el cronograma exacto que corrió tu flota en el lab.

## Dónde estás parado

El triunfo de treinta segundos, en voz alta: la concurrencia es un presupuesto que gastas, no una velocidad que recibes. Y las dos perillas que lo gastan: el tamaño del pool, y el cronograma de retry. Si además puedes decir por qué existe el jitter sin que la palabra "aleatorio" aparezca antes de la palabra "estampida", tienes la lección entera.

La flota ahora sondea cincuenta targets tipada, parseada, acotada y cortés: respeta un tope publicado, absorbe 429s con backoff con jitter, convierte sockets colgados en timeouts tipados, y termina con un resultado para cada uno de los targets, sin importar qué hizo la red. Y acá está la parte incómoda: está completamente sin demostrar. Cada frontera del clasificador, cada delay de backoff, cada mapeo de timeout es una afirmación que nadie probó; la evidencia hasta ahora es "se veía bien en mi terminal", que es exactamente el estándar que rechazarías de cualquier otro. La próxima lección: vitest. Sondas para tu propio código, cableadas al cron antes de que publique otro status.json. Hora de demostrarlo.
