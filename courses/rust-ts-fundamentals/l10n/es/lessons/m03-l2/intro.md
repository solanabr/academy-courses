# React como consumidor: el panel

La lección pasada partió el repo en un workspace real: `pulse-core` extraído con un `package.json` honesto, la flota importándolo cruzando la frontera, la suite de m02-l4 todavía en verde, y una promesa al salir de que venía un segundo consumidor. Esta es esa lección. La estación viene commiteando `status.json` cada 30 minutos desde el módulo 1, noches y fines de semana, y en todo ese tiempo ni un solo pixel lo mostró nunca. Hoy recibe una cara.

Y la cara la hacemos primero, la teoría después. Desde la raíz del repo:

```bash
cd packages
pnpm create vite pulse-board --template react-ts
```

(Eso corre `create-vite`, 9.2.0 mientras escribo esto el 2026-09-02; la flag `--template react-ts` lo vuelve no interactivo.) Ahora destripa la demo y reemplaza `packages/pulse-board/src/App.tsx` con la cosa más chica que pueda mostrar tus datos. Pon tu propio usuario de GitHub:

```tsx
import { useEffect, useState } from "react";

const RAW_URL =
  "https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json";

export default function App() {
  const [raw, setRaw] = useState("loading...");

  useEffect(() => {
    fetch(RAW_URL)
      .then((res) => res.text())
      .then(setRaw)
      .catch((err) => setRaw(String(err)));
  }, []);

  return <pre>{raw}</pre>;
}
```

Instálalo y córrelo:

```bash
cd pulse-board
pnpm install
npm run dev
```

(Sobre las grafías mezcladas, invocando una vez la nota de la casa de m03-l1 para que no pique más: las instalaciones adentro del workspace son trabajo de pnpm, mientras `npm run` y `pnpm run` leen el mismo bloque `scripts` y son intercambiables; las líneas de script de este curso usan la que replicó el toolchain de verificación, y `pnpm run dev` acá se comportaría idéntico.)

Abre la URL de localhost que se imprimió. Ese muro de JSON en tu navegador no son datos de muestra, es tu flota: los targets que elegiste, latencias que midió tu cron en una máquina que no es tuya, traídas cross-origin desde tu repo público con cero backend y cero claves. Semanas de sondeo sin tripulación, en una página, adentro de quince minutos. Deja esa pestaña abierta; toda la lección es sobre convertirla de un volcado de `<pre>` en un panel que le mostrarías a alguien.

## Resumen

Los hallazgos primero:

- Un panel es una función pura de un archivo JSON; React es solo el loop de render. Componente = función de props a UI, el estado el único input que dispara un repintado. Esta lección enseña React solo a nivel consumidor, a propósito: aterriza el piso de consumidor sobre el que se sostiene el trabajo real de cliente de dApps, y la profundidad se queda con el curso de dominio del lado del cliente.
- Tu repo público ya es una API de datos. `raw.githubusercontent.com` manda `access-control-allow-origin: *` sin condiciones (sondeado el 2026-09-02), cachea por 5 minutos (`cache-control: max-age=300`, Fastly), y sirve `.json` como `text/plain`. Los tres hechos le dan forma al código de hoy.
- El primer movimiento del lab cobra el pagaré de m02-l4: el escritor de la flota se recablea para emitir filas de la unión `ProbeResult`, así que `status.json` finalmente habla el dialecto tipado que el resto de la estación habla desde M2.
- Los bytes traídos cruzaron una frontera de red, así que pasan por un schema de zod como toda frontera desde m02-l2: un archivo corrupto produce un estado de error visible, nunca una página en blanco.
- El panel importa `classifyProbe` de `pulse-core`, así que la flota y el panel demostrablemente corren el mismo código de clasificación: la extracción de m03-l1 demostrada, no afirmada.
- La ayuda se repliega según un cronograma: yo manejo el scaffold y el efecto de polling, tú construyes `StatusRow` desde una especificación a nivel de firma, y el indicador de desactualización del challenge es solo tuyo.

## El loop de render y el camino de los datos

Una cerca antes que nada, dicha sin disculpas: este no es un curso de React. React es un tema de tamaño carrera, y el hogar de este catálogo para él, el curso de dominio del lado del cliente, está en producción mientras escribo esto; espera UX de wallet, aterrizaje de transacciones y profundidad real de cliente de dApps ahí. Nuestro trabajo es el nivel de consumidor del que arranca ese tipo de trabajo: componentes, props, estado, un efecto. Eso resulta ser suficiente para entregar un panel real, lo que te dice algo sobre dónde vive de verdad el 80 por ciento.

### Un componente es una función, el estado es el timbre

Primero saca la mística. El mejor modelo de un componente de React es la cosa que vienes escribiendo todo el curso: una función pura. Toma un objeto de inputs, llamados props, y devuelve una descripción de UI. Mismos inputs, misma UI. Sin estados de ánimo escondidos.

```tsx
function Greeting({ name }: { name: string }) {
  return <p>hello, {name}</p>;
}
```

La sintaxis de corchetes angulares es JSX, y merece exactamente un párrafo: es azúcar compilado para llamadas de función. `<p>hello, {name}</p>` se vuelve una llamada que construye `{ type: "p", props: { children: [...] } }`, un objeto simple que describe qué debería existir. El toolchain de Vite hace la compilación; nunca lo configuras. Esa es toda la ceremonia que JSX recibe en este curso.

Entonces si los componentes son funciones puras, ¿qué hace que la página cambie alguna vez? Una cosa: el estado. `useState` te da un valor más un setter, y llamar al setter es el único timbre que React atiende. Setea estado, React vuelve a correr tu función con el valor nuevo, hace el diff de la descripción contra el DOM, parcha la diferencia. Los datos fluyen en un sentido, siempre: estado adentro, render afuera, pixeles último. React nunca vuelve a leer tu tabla desde el DOM, y reasignar alguna variable a nivel de módulo al lado del componente le es invisible. Solo el setter agenda un repintado.

![Los bytes traídos fluyen por el parseo hacia el estado y de ahí a los pixeles parchados, mientras las flechas que van del DOM o de variables de módulo de vuelta al loop están tachadas.](assets/v01-flowchart.webp)

Lo que da el aha por el que se llama esta lección: un panel es una función pura de un archivo JSON. `status.json` es el estado del mundo; el panel es `render(state)`. Todo lo demás, traer, sondear, cachear, es plomería para mantener fresco ese único input. Agárrate de ese modelo y la mayoría de los tutoriales de React se colapsan en detalles sobre la plomería.

Una primitiva más, porque "traer un archivo cada minuto" es un efecto secundario, no un cálculo puro. `useEffect` es el contenedor de React para exactamente eso: código que corre después del render, tocando el mundo. Toma una closure y un array de dependencias; el array vacío `[]` significa correr una vez al montar. Un asterisco de modo dev antes de que cuentes nada en devtools: la plantilla de create-vite envuelve la app en el StrictMode de React, que en desarrollo deliberadamente monta, desmonta y vuelve a montar cada componente una vez para sacudir las limpiezas que falten, así que "una vez al montar" aparece como dos veces en la pestaña Network mientras desarrollas; los builds de producción lo corren una vez. Crucialmente, la closure puede devolver una función de limpieza, llamada al desmontar o en el hot reload; sáltatela en un interval y cada save de dev apila otro poller, un bug que el lab te hace conocer a propósito. Esa es toda la API de hooks que este curso enseña: `useState`, `useEffect`, listo. Context, reducers, refs, server components, suspense: todo real, todo postergado por nombre al curso de dominio del lado del cliente.

Una trampa preventiva. Tipea "react fetch data" en una caja de búsqueda y te van a decir que los efectos hechos a mano son cosa de amateurs y que una librería de fetching de datos es lo mínimo. Esas librerías son excelentes, y resuelven problemas que esta lección no tiene: deduplicación entre docenas de componentes, invalidación de cache, escrituras optimistas. Tu requisito es una URL, un interval, un schema; `useState` más `useEffect` más zod es todo el trabajo, y saber eso es la habilidad. El caso de la librería se argumenta como corresponde cuando llegan las mutaciones y el estado compartido de servidor, y ese es territorio del dominio del lado del cliente.

### El camino de los datos: tu repo público ya es una API

Ahora la plomería, y acá es donde rinde la decisión de tu-repo-es-público del módulo 1. El repo de tu estación es público, lo que significa que cada archivo en él se sirve en `raw.githubusercontent.com/<user>/<repo>/<branch>/<path>`. Sin token, sin SDK, sin ningún servidor que tú corras. La pregunta que hace quien trabaja de esto antes de confiar en ese camino: ¿qué hace de verdad ese endpoint? No qué dice un blog post que hace. Así que lo sondeé, y todo en esta sección se enseña desde los headers observados, fechados el 2026-09-02.

![Un cron commitea datos de estado a un repositorio público mientras un navegador los lee de vuelta pasando por un cache de CDN de cinco minutos, una revisión de schema y el estado de React.](assets/v02-diagram.webp)

Tres hallazgos importan. Primero, CORS. Los navegadores bloquean los fetch cross-origin salvo que el servidor opte por permitirlos, y raw.githubusercontent opta por todo: `access-control-allow-origin: *`, sin condiciones. La sonda revisó el caso furtivo, mandando un request pelado y después uno etiquetado con Origin, y los headers volvieron idénticos, así que la permisividad no está reflejada por origen, está simplemente abierta. Por esto tu volcado de `<pre>` de quince minutos funcionó al primer intento en vez de morir con un error de CORS en la consola.

Segundo, el cache, que cambia tu modelo mental de "en vivo". El endpoint devuelve `cache-control: max-age=300` con un `etag` fuerte, servido por Fastly; la sonda vio un MISS volverse un HIT un segundo después. Cinco minutos de cache de CDN, apilados sobre un cron de 30 minutos: tu panel puede ir detrás de la realidad por el intervalo del cron más la ventana de cache, y ningún código de React cambia eso, porque los bytes desactualizados llegan desactualizados. Cuando tu panel "no se actualiza", mira los headers de respuesta en devtools primero, no el componente.

![Una línea de tiempo muestra un intervalo de cron de treinta minutos con una ventana de cache de cinco minutos superponiéndose a un commit fresco, así que quien mira puede leer filas viejas después de que existen datos nuevos.](assets/v03-timeline.webp)

Tercero, el content type. El endpoint sirve tu archivo `.json` como `content-type: text/plain; charset=utf-8`. Y sin embargo `await res.json()` lo parsea sin quejarse, porque la especificación de fetch de WHATWG parsea el cuerpo que le pediste parsear; el header de MIME viaja de arriba sin leerse. Conveniente, y un poco deshonesto. El día en que cambies a una librería HTTP que olfatea el content-type antes de parsear, esta respuesta exacta se vuelve un reporte de bug, así que aprendes el hecho ahora, mientras es barato.

Una línea de honestidad para completar el cuadro. GitHub no documenta ningún rate limit para este endpoint, y no voy a inventar uno; lo que estás consumiendo son bytes estáticos sin autenticar detrás de un CDN con controles de abuso no documentados. Y GitHub no bendice raw.* como producto de hosting para nada. En este curso solo es el endpoint de datos, siempre. El panel en sí, el HTML y el JS, se entrega a Vercel la lección que viene, que es el camino sancionado.

Así que los bytes llegan: CORS abierto, posiblemente desactualizados, con el MIME mal etiquetado. ¿Les confías? Ya sabes la respuesta, porque es la misma respuesta que en m02-l2: cruzaron una frontera de red, así que se parsean, no se afirman. Un schema de zod que espeja la unión `ProbeResult` de `pulse-core` se sienta en la frontera del fetch, y un archivo corrompido a mano muere ahí como estado de error visible en vez de hondo en un render como página en blanco. "Parse, don't validate", ahora cuidando pixeles.

Nombra el trade-off mientras está fresco: un trato genuinamente buenísimo con fecha de vencimiento impresa. Sondear un archivo crudo no cuesta nada (sin backend, sin claves, sin cuenta, el uptime de tu repo) al precio que acaba de mostrarte la matemática del CDN: hasta cron-más-cinco-minutos detrás de la realidad, sin push, sin auth, solo público. En el momento en que necesites actualizaciones en tiempo real, filas privadas, o un camino de escritura, necesitas una API de verdad; el edge worker de M7 empieza esa historia. Hasta entonces, un backend acá sería pura ceremonia.

Hay un trade-off con forma de React escondido acá también. Un loop de render de framework compra UI declarativa: describe cómo se ve el panel para un estado dado, y hacer el diff es problema de alguien más. El precio es un paso de build y una dependencia que sobrevive a tu interés en ella. Para una tabla estática, el DOM pelado alcanzaría; para un panel al que le crecen paneles de M8 a M10 (un carril de Solana, un gráfico de latencia, una tira de salud de workers), tomas el trato, porque cada panel nuevo es otra función pura del mismo estado. Elige frameworks por a dónde va el artefacto, no por lo que necesita el commit actual.

### El segundo consumidor: el import que demuestra la frontera

El movimiento tres es corto porque m03-l1 hizo el trabajo pesado. El panel tiene que decidir qué color recibe cada fila, y "up contra degraded" es un juicio que la estación ya hace, en `classifyProbe`, adentro de `pulse-core`. Reimplementar esas tres líneas localmente funcionaría hoy y se desviaría mañana: alguien reajusta la banda de latencia en la flota, se olvida del panel, y los pixeles empiezan a discrepar con las alertas sobre qué significa "degraded". Así que el panel hace la única cosa defendible:

```ts
import { classifyProbe, type ProbeResult } from "pulse-core";
```

La misma línea de import que usa la flota, resuelta por el mismo symlink `workspace:*`, ejecutando el mismo código. La lección pasada la extracción era un argumento; esta línea la vuelve un pixel. Un clasificador, dos consumidores, cero deriva posible.

![El paquete pulse core se sienta en el centro mientras la flota, el panel nuevo y dos consumidores fantasma futuros importan todos el mismo clasificador.](assets/v04-diagram.webp)

Un movimiento chico de TypeScript a nivel consumidor se gana su lugar acá. El panel quiere un tipo para las claves de su mapa de colores: el veredicto que devuelve `classifyProbe`. Tu propio `index.ts` da la casualidad de que re-exporta `Verdict`, así que un import directo funciona, pero haz el movimiento que necesitarías contra un paquete de terceros cuya superficie no controlas: `type Verdict = ReturnType<typeof classifyProbe>`. `ReturnType` es un tipo utilitario incorporado (un genérico que consumes, exactamente la habilidad de m02-l2) que extrae el tipo de retorno de una función, y acá también caza algo que el alias exportado no publicita, como muestra el próximo párrafo. El mapa de colores queda tipado en cualquier caso.

Un detalle honesto sale de extraer el tipo de esta forma. `classifyProbe` es la forma de frontera que congeló m02-l4: toma el par no confiable `(kind, value)` y responde `'invalid'` para un kind que no reconoce. Así que el tipo que acabas de extraer tiene cuatro miembros, no los tres que carga la unión `Verdict` exportada, y un `Record` sobre él necesita una entrada `invalid` o el compilador va a nombrar la clave que falta.

Dos pasajes fechados cierran la teoría. Tu `pulse-board` corre sobre Vite 8, y Vite 8.0.0 (entregado el 2026-03-12) está movido por Rolldown: el bundler que tritura tu TypeScript está escrito en Rust, fijado como `rolldown ~1.2.4` en el propio manifest de Vite. La tesis de los dos lenguajes está sentada en tu `node_modules` ahora mismo, y ese es todo el recorrido de bundlers que recibes. Y cuando a este panel le crezca su panel de Solana en m08-l2, su `@solana/kit` se va a fijar leyendo rangos de peer, no de memoria: la regla de m03-l1, ya acumulándose.

**Profundiza (el 20%).** esta lección enseñó la porción componentes-props-estado-efecto que necesita un consumidor de datos, y para. La profundidad de hooks, context, ruteo, formularios, todo lo que tenga forma de framework, queda deliberadamente como bookmark. La rampa de entrada canónica es el propio [Quick Start](https://react.dev/learn) de React (URL sondeada el 2026-09-02): interactivo, gratis, mantenido por el equipo de React. Léelo después del lab si React te hizo clic y quieres el vocabulario completo; nada de abajo depende de él, y la profundidad seria del lado del cliente vive en el curso de dominio del lado del cliente igual.

## Lab: pulse-board

Meta: el volcado de `<pre>` se vuelve un panel de estado tipado, parseado y coloreado por el clasificador que sondea en un interval y falla a gritos con basura. Yo manejo los pasos 1 al 4 con devtools abierto; el paso 5 te entrega una especificación en vez de un diff; el challenge después del lab es sin guía.

1. **Cablea el scaffold al workspace.** El `pnpm create vite` de la apertura ya creó `packages/pulse-board`, y como `pnpm-workspace.yaml` globea `packages/*`, ya es miembro del workspace. Limpia la demo (`src/App.css`, `src/assets`, los imports del logo) y agrega las dos dependencias que el panel de verdad necesita, desde `packages/pulse-board`:

   ```bash
   pnpm add zod
   pnpm add pulse-core --workspace
   ```

   (Frescura: `pnpm add zod` resolvió a 4.5.4 el 2026-09-02; la flag `--workspace` fuerza el protocolo `workspace:*` así que `pulse-core` se enlaza desde tu repo, nunca desde el registro.) Checkpoint: `packages/pulse-board/package.json` ahora lista `"pulse-core": "workspace:*"`, y `npm run dev` sigue sirviendo.

2. **Cobra el pagaré de m02-l4: recablea el escritor de la flota a la unión.** La nota al pie de m02-l4 congeló las filas planas v0 de `status.json` y prometió que el escritor se recablearía en M3 una vez que hubiera un panel que mantener en verde. El panel está a veinte minutos, así que el recableado pasa ahora, primero, o el schema que escribas en el próximo paso va a rechazar tu propio archivo. Abre `packages/pulse-fleet/fleet.ts`, el escritor del cron. Su lista `TARGETS`, el sobre del reporte y el path de escritura `../../status.json` se quedan todos; lo que cambia es el tipo de fila, del mentiroso `{ url, status, latencyMs: number | string, checkedAt }` a un envoltorio que carga la unión real:

   ```ts
   import { writeFile } from "node:fs/promises";
   import type { ProbeResult } from "pulse-core";

   type TargetStatus = {
     url: string;
     checkedAt: string;
     result: ProbeResult;
   };

   async function probeOne(url: string): Promise<TargetStatus> {
     const checkedAt = new Date().toISOString();
     const start = performance.now();
     try {
       const res = await fetch(url, { signal: AbortSignal.timeout(10_000) });
       const latencyMs = Math.round((performance.now() - start) * 10) / 10;
       if (!res.ok) {
         return { url, checkedAt, result: { kind: "http-error", status: res.status } };
       }
       return { url, checkedAt, result: { kind: "ok", latencyMs } };
     } catch (err) {
       if (err instanceof Error && err.name === "TimeoutError") {
         return { url, checkedAt, result: { kind: "timeout", budgetMs: 10_000 } };
       }
       return { url, checkedAt, result: { kind: "dns-error", host: new URL(url).hostname } };
     }
   }
   ```

   Recorre el catch, porque es el mapa de salidas de m02-l3 comprimido a dos brazos. `AbortSignal.timeout` rechaza con un error *llamado* `TimeoutError`, así que esa revisión de nombre es la salida de "se disparó nuestro propio temporizador" y se vuelve la variante `timeout` con el budget que reventó. Todo lo demás en el catch es la red misma fallando, DNS, conexión rechazada, TLS, antes de que ningún budget pudiera expirar; `fetch` rechaza esos inmediatamente como un `TypeError`, y aterrizan en el brazo `dns-error` exactamente como enseñó el ejercicio de m02-l1. Un `catch` pelado acá publicaría un host muerto como un timeout de diez segundos, la mentira chica y precisa que este recableado existe para desalojar. Borra la declaración local v0 de `ProbeResult` mientras estás ahí; el tipo ahora llega de `pulse-core`, solo como tipo, sin costarle nada al runtime. Renombra el tipo del elemento del array de resultados del escritor a `TargetStatus` y el resto compila sin tocarlo. Córrelo una vez desde `packages/pulse-fleet`, `npx tsx fleet.ts`, y abre el `status.json` fresco en la raíz del repo: cada fila ahora dice `{ "url", "checkedAt", "result": { "kind": ... } }`. El dialecto v0 que m02-l1 volvió inrepresentable en `probe.ts` finalmente fue desalojado del único archivo al que todavía se le permitía mentir en él. Commitea y empuja antes de construir el panel, para que la próxima corrida del cron publique filas de la unión para tu URL en vivo también.

3. **Ponle schema a la frontera.** Crea `src/status.ts`, el checkpoint de frontera del panel. El schema espeja el reporte que emite el escritor que acabas de recablear: `generatedAt`, más una entrada por target envolviendo la unión `ProbeResult`:

   ```ts
   import { z } from "zod";

   const probeResultSchema = z.discriminatedUnion("kind", [
     z.object({ kind: z.literal("ok"), latencyMs: z.number() }),
     z.object({ kind: z.literal("timeout"), budgetMs: z.number() }),
     z.object({ kind: z.literal("http-error"), status: z.number() }),
     z.object({ kind: z.literal("dns-error"), host: z.string() }),
   ]);

   const targetStatusSchema = z.object({
     url: z.string(),
     checkedAt: z.string(),
     result: probeResultSchema,
   });

   export const statusFileSchema = z.object({
     generatedAt: z.string(),
     targets: z.array(targetStatusSchema),
   });

   export type StatusFile = z.infer<typeof statusFileSchema>;
   export type TargetStatus = StatusFile["targets"][number];
   ```

   Fíjate en lo que compra `z.infer` en esta frontera: el `result` parseado es estructuralmente idéntico al `ProbeResult` de `pulse-core`, así que hacer narrowing sobre él en el próximo paso es narrowing real contra la unión real, sin casts en ninguna parte. Si los nombres de campo de tu flota difieren de los míos, el schema es el único lugar donde los reconcilias; para eso sirve un checkpoint de frontera.

4. **El efecto de polling, con devtools abierto.** Reemplaza `src/App.tsx`. Antes de pegar, abre la pestaña Network de los devtools del navegador y mantenla visible; el punto de este paso es ver la teoría pasar.

   ```tsx
   import { useEffect, useState } from "react";
   import { statusFileSchema, type StatusFile } from "./status";
   import { StatusBoard } from "./StatusBoard";

   const RAW_URL =
     "https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json";
   const POLL_MS = 60_000;

   type BoardState =
     | { phase: "loading" }
     | { phase: "error"; message: string }
     | { phase: "ready"; data: StatusFile };

   export default function App() {
     const [state, setState] = useState<BoardState>({ phase: "loading" });

     useEffect(() => {
       let cancelled = false;

       async function poll() {
         try {
           const res = await fetch(RAW_URL);
           if (!res.ok) throw new Error(`HTTP ${res.status}`);
           const parsed = statusFileSchema.safeParse(await res.json());
           if (cancelled) return;
           if (parsed.success) {
             setState({ phase: "ready", data: parsed.data });
           } else {
             const message = parsed.error.issues
               .map((i) => `${i.path.join(".")}: ${i.message}`)
               .join("; ");
             setState({ phase: "error", message });
           }
         } catch (err) {
           if (!cancelled) setState({ phase: "error", message: String(err) });
         }
       }

       poll();
       const id = setInterval(poll, POLL_MS);
       return () => {
         cancelled = true;
         clearInterval(id);
       };
     }, []);

     if (state.phase === "loading") return <p>loading fleet status...</p>;
     if (state.phase === "error") return <p>board error: {state.message}</p>;
     return <StatusBoard data={state.data} />;
   }
   ```

   Una cosa para notar antes de que me acuses de contradecirme: esos dos imports locales no llevan extensión `.js`, y m03-l1 fue enfática en que los tsconfig de este curso vuelven la extensión no opcional. Las dos cosas son ciertas, porque son tsconfig distintos. La regla de m03-l1 es sobre la resolución `nodenext`, donde un especificador nombra el archivo emitido. El panel es un paquete de create-vite con su propio tsconfig puesto en resolución `bundler`, donde el bundler resuelve el especificador y sin extensión es el idioma. La regla durable no es "siempre escribe `.js`", es "escribe lo que exija tu modo de resolución", y la forma rápida de saber en cuál estás es leer `moduleResolution` en el tsconfig bajo el que de verdad estás compilando. No "arregles" estas dos líneas.

   Huesos familiares, deliberadamente: `BoardState` es una unión discriminada (el movimiento de m02-l1, ahora dándole forma a la UI), y el render de abajo es solo narrowing. `StatusBoard` todavía no existe, así que el servidor de dev muestra un error de import; está bien por un paso. Dos lecturas antes de seguir. En la pestaña Network, haz clic en el request de `status.json` y lee los headers de respuesta tú mismo: `cache-control: max-age=300`, el `etag`, la línea `via` de varnish. (¿Sin navegador a mano? `curl -s -D - -o /dev/null https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json` vuelca los headers idénticos, `access-control-allow-origin: *` incluido.) Segunda lectura: ¿por qué 60 segundos? Los datos cambian cada 30 minutos y el CDN reusa una copia por 5, así que sondear más rápido solo compra re-lecturas cacheadas; 60s mantiene la pestaña honesta dentro de un minuto de que el cache se ponga fresco. El interval y el cron son relojes distintos, y confundirlos es la trampa.

   Ahora el bug que tienes que conocer una vez. Comenta las dos líneas de limpieza (`cancelled = true; clearInterval(id);`), guarda, y edita cualquier archivo unas cuantas veces para disparar hot reloads. Mira la pestaña Network llenarse: cada reload apiló otro poller, ninguno de los viejos murió. Yo entregué exactamente esto, y el reporte de bug de la-pestaña-se-come-un-core que sigue no es divertido. Restaura la limpieza, mira los requests caer de vuelta a uno por minuto, y nunca más escribas un efecto con interval sin su return. (Este ejercicio necesita un navegador real, porque el hot reload es el disparador; si trabajas sin interfaz, lee el diagrama de flujo de abajo como la transcripción del ejercicio y mete un `console.count("poll tick")` adentro de `poll` para que el apilado se muestre como un contador disparándose la próxima vez que sí tengas devtools.)

![Con limpieza, cada hot reload reemplaza el interval de polling, mientras que sin limpieza cada reload agrega otro poller vivo hasta que los requests se apilan.](assets/v05-flowchart.webp)

5. **StatusRow y los colores del clasificador, desde una especificación.** Tu turno, solo a nivel de firma. Construye `src/StatusRow.tsx` exportando `StatusRow({ target }: { target: TargetStatus })`, una fila de tabla que: deriva su veredicto del `classifyProbe` importado, que toma el par congelado `(kind, value)`, así que haces narrowing sobre `target.result.kind` y le pasas la lectura de esa variante; colorea la celda del veredicto desde un mapa `Record<Verdict, string>` (consigue `Verdict` vía `ReturnType<typeof classifyProbe>`, y recuerda que carga `'invalid'`); renderiza un string de detalle humano por variante haciendo narrowing sobre ese mismo `target.result.kind` (latencia para `ok`, budget para `timeout`, código para `http-error`, host para `dns-error`); y muestra `checkedAt` como hora local. Una variante necesita una decisión tuya: un `dns-error` carga un hostname, no una lectura numérica, así que no tiene nada que pasarle al clasificador. La mía, para después de que lo hayas intentado:

   ```tsx
   import { classifyProbe } from "pulse-core";
   import type { TargetStatus } from "./status";

   type Verdict = ReturnType<typeof classifyProbe>;

   const VERDICT_COLOR: Record<Verdict, string> = {
     up: "#22c55e",
     degraded: "#eab308",
     down: "#ef4444",
     invalid: "#a1a1aa",
   };

   export function StatusRow({ target }: { target: TargetStatus }) {
     // classifyProbe is the frozen (kind, value) boundary form. A dns-error
     // has a hostname and no reading, so the board decides that one here.
     const verdict: Verdict =
       target.result.kind === "ok"
         ? classifyProbe("ok", target.result.latencyMs)
         : target.result.kind === "timeout"
           ? classifyProbe("timeout", target.result.budgetMs)
           : target.result.kind === "http-error"
             ? classifyProbe("http-error", target.result.status)
             : "down";

     const detail =
       target.result.kind === "ok"
         ? `${target.result.latencyMs} ms`
         : target.result.kind === "timeout"
           ? `no answer in ${target.result.budgetMs} ms`
           : target.result.kind === "http-error"
             ? `HTTP ${target.result.status}`
             : `DNS failed for ${target.result.host}`;

     return (
       <tr>
         <td>{target.url}</td>
         <td style={{ color: VERDICT_COLOR[verdict] }}>{verdict}</td>
         <td>{detail}</td>
         <td>{new Date(target.checkedAt).toLocaleTimeString()}</td>
       </tr>
     );
   }
   ```

   Y el panel que mapea las filas, `src/StatusBoard.tsx`, que honestamente es demasiado simple para especificarlo:

   ```tsx
   import type { StatusFile } from "./status";
   import { StatusRow } from "./StatusRow";

   export function StatusBoard({ data }: { data: StatusFile }) {
     return (
       <table>
         <thead>
           <tr>
             <th>target</th>
             <th>verdict</th>
             <th>detail</th>
             <th>checked</th>
           </tr>
         </thead>
         <tbody>
           {data.targets.map((t) => (
             <StatusRow key={t.url} target={t} />
           ))}
         </tbody>
       </table>
     );
   }
   ```

   Checkpoint, y es todo el punto de la lección: el servidor de dev ahora muestra filas de tus targets reales, latencias que midió tu cron, coloreadas por la función exacta que le pone barrera a las publicaciones de la flota. Un `solana.com` verde en tu pantalla y un `solana.com` verde en los logs del cron nunca pueden discrepar, porque son una sola función.

6. **El ejercicio del archivo corrupto.** Demuestra la frontera antes de confiar en ella. Copia una respuesta real a `public/corrupt.json`, y después rómpela a mano: cambia el `"kind": "ok"` de una fila a `"kind": "okay"` (la falsificación de m02-l1, de vuelta por venganza). Apunta `RAW_URL` a `/corrupt.json` temporalmente y recarga. Esperado: ninguna página en blanco, ningún lloriqueo solo en la consola, sino tu estado de error, en pantalla, nombrando el path y el discriminante que falló. Ese mensaje es zod rechazando en el borde, exactamente como se diseñó. Apunta `RAW_URL` de vuelta a tu URL cruda y confirma que las filas vuelven.

7. **Build limpio.** Desde `packages/pulse-board`:

   ```bash
   npm run build
   ```

   El script de build de la plantilla react-ts corre `tsc -b` antes de `vite build`, así que esto es la barrera de tipos y el bundler en una sola línea. Esperado: cero errores de tipo y una carpeta `dist/`. Esa carpeta es un sitio completamente estático, que es precisamente lo que vuelve tan corta la próxima lección.

## Challenge

Sin guía, cerrando el loop de la matemática de la desactualización. Agrega un indicador de "última actualización" que: (a) muestre `generatedAt` como hora local; (b) calcule la edad de los datos más nuevos desde los valores traídos, no desde cuándo los trajiste; (c) cambie visualmente a desactualizado (color, badge, tú decides) pasados dos intervalos de cron, 60 minutos, porque una corrida perdida es un hipo y dos son un incidente. Mientras estás ahí, agrega un query string que rompa el cache al fetch (`` `${RAW_URL}?t=${Date.now()}` `` vuelve cada poll una clave de cache distinta, cambiando la amabilidad con el CDN por frescura). ¿La bala de plata para los paneles que se ven desactualizados? No hay una; hay dos estrategias honestas, y ahora tu panel hace las dos.

![Una tarjeta de dos columnas compara romper el cache contra un indicador honesto de desactualización en frescura, costo y modos de falla, terminando con las dos adoptadas.](assets/v06-comparison.webp)

Aceptación, las cuatro: el panel renderiza filas reales de la flota coloreadas por el clasificador importado; el ejercicio del archivo corrupto muestra el estado de error de zod en pantalla; el indicador de desactualización cambia con datos viejos (pruébalo alimentándole un archivo local trucado con timestamps de hace una hora); `npm run build` sale limpio.

## Checkpoint, y la primera URL visible para desconocidos

Haz el balance de lo que quedó demostrado. La extracción sobrevivió un segundo consumidor real: una línea de import, y la flota y el panel nunca pueden desviarse sobre qué significa "degraded". La apuesta del repo público pagó su dividendo: una app de navegador con cero backend, leyendo datos reales cruzando orígenes porque los headers sondeados lo permiten. Y la disciplina de fronteras se mantuvo en territorio nuevo: los bytes basura mueren en el schema con un mensaje, no en el render con una página en blanco. La pregunta de recuperación, en frío, sin notas: nombra las dos demoras apiladas entre que corre una sonda y que cambia un pixel. Si dijiste el cronograma del cron y el cache de 5 minutos del CDN, el modelo mental está instalado.

Si el panel renderiza pero las filas se ven mal, confía en el orden de depuración que enseñó la lección: los headers de respuesta primero (¿es el cache?), el estado de error del schema segundo (¿es la forma?), el componente último. El componente casi nunca es el mentiroso; es una función pura de lo que sea que le hayan pasado.

El panel corre hermoso, en localhost, donde exactamente una persona en la Tierra puede verlo. Esa carpeta `dist/` del paso 7 está ahí sentada, completamente estática, sin necesitar nada más que un host. La lección que viene: `vercel login`, `vercel`, y una URL que le puedes mandar por mensaje a un desconocido. La primera URL del curso, en la lección diez. Trae un teléfono.
