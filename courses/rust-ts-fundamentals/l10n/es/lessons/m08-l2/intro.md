# Lecturas en producción: el panel de Solana

## Resumen

La lección pasada construiste el medidor de banco: `chain-probe.ts` mide el tiempo de slot contra el target de 300ms, deriva la cuenta regresiva de epoch desde el epoch fijo de 432,000 slots, y te dio el modelo de cliente de cuatro ideas, con todo lo más profundo pasado al curso btc-to-sol por su nombre. Funciona. También corre en exactamente una máquina, la tuya, en una terminal que nadie más va a ver nunca. Un medidor que nadie puede ver es un medidor que no existe.

Hoy es la re-entrega más grande del curso, y quiero decir primero la parte que se calla: acá no hay nada nuevo excepto el target. La lectura es una línea que ya entiendes. El polling es el de m03-l2. El cache es el KV de m07-l1. El backoff es el de m02-l3. Lo que cambia es dónde corre todo: la misma lectura de la blockchain, promovida a cada superficie desplegada de la que la estación es dueña. Al final, el panel de Vercel renderiza un panel de Solana en vivo y el JSON público del worker lleva un snapshot cacheado de la blockchain, los dos en las URLs que ya entregaste.

Demuestra la lectura primero. En el repo de la estación, donde `@solana/kit@^8` está instalado desde la lección pasada, mete esto en `packages/pulse-fleet/balance.ts` al lado de `chain-probe.ts` y córrelo desde ese directorio (el hogar de los scripts de banco de la lección pasada, con el `"type": "module"` y el tsx que los scripts necesitan):

```ts
import { createSolanaRpc, address } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");
const watched = address("So11111111111111111111111111111111111111112");

const { value: lamports } = await rpc.getBalance(watched).send();
console.log(lamports);
```

```bash
npx tsx balance.ts
# 1807515117625n
```

Esa dirección es el mint de wrapped-SOL, una cuenta ocupada de mainnet que va a seguir existiendo el año que viene; el número que te toque va a diferir del de mi ejecución del 2026-09-02. Fíjate en la `n`. Ese balance es un `bigint`, está en lamports, y la razón por la que kit se niega a darte un número pelado es lo primero que enseña producción hoy.

Cómo corre esta lección, en voz alta: la lectura canónica y el esqueleto del panel se trabajan en pantalla una vez; el cableado del polling, la clave del cache de KV, y el presupuesto de backoff son tuyos para componer desde patrones de los que ya eres dueño; la extensión de la segunda dirección vigilada al final es totalmente en solitario. Ese es el repliegue de M8 y no se revierte.

## La misma lectura, cada superficie

### Una lectura de la blockchain es solo otra sonda

Acá va la síntesis que hace chica esta lección: una lectura de la blockchain es solo otra sonda. Es un POST HTTP a un endpoint con rate limit que normalmente responde rápido, a veces responde lento, y ocasionalmente te rechaza. Lo que quiere decir que cada pregunta de producción que levanta ya se respondió hace semanas, en el módulo dos, antes de que este curso hubiera dicho la palabra Solana. ¿Cada cuánto puedo llamarlo? Presupuesto. ¿Y si falla? Backoff, después degrada a último estado conocido. ¿Quién paga cuando cincuenta navegadores preguntan a la vez? Todos los que están detrás del tope compartido, juntos. El único material genuinamente nuevo de hoy es un tipo de dato y una disciplina, y los dos entran en una sección cada uno.

![Una sola lectura de la blockchain alimenta tres superficies, con flechas en negrita promoviéndola al panel desplegado y al edge worker mientras el script de banco se queda local.](assets/v01-diagram.webp)

La forma de la estación después de hoy, concretamente: `pulse-board` (Vercel) sondea la blockchain directo y renderiza el slot, el tiempo de slot medido, y un balance vigilado. `pulse-edge-ts` (Cloudflare) pliega las lecturas de slot y de balance adentro de su cron de 15 minutos existente, escribe el resultado en KV bajo una clave `chain`, y lo sirve en el JSON público al lado del bloque `targets` que ya publica. Dos deploys, cero plataformas nuevas, y la prueba de aceptación es dos pestañas de navegador mostrando el mismo slot con un refresh de diferencia.

### El número que miente cortésmente

JavaScript tiene un solo tipo de número, un float de 64 bits, y es exacto para enteros solo hasta `Number.MAX_SAFE_INTEGER`: 2^53 menos 1, que es 9,007,199,254,740,991. En lamports, eso es unos 9 millones de SOL. Por encima de esa línea, la aritmética de `number` no lanza, no avisa, ni siquiera se tambalea visiblemente. Redondea. `9007199254740993` se vuelve `9007199254740992` y la consola lo imprime con total confianza. Para un contador de vistas, a quién le importa. Para un display de balance, esa es la mentira cortés que entrega un número equivocado a la pantalla de alguien.

Las cuentas reales de mainnet están por encima de esa línea, billeteras de exchange y stake pools entre ellas, que es por lo que kit tipa cada u64 como `bigint` y nunca como `number`. Esto no es una elección de performance; el parseo de bigint es, si acaso, más lento. Es una política de puerta sobre la corrección: ningún balance puede corromperse en silencio al entrar a tu programa. La oportunidad de corrupción se mueve a tu lado de la puerta, y tiene exactamente una forma: en el momento en que ruteas un valor de lamports a través de `Number` para poder hacerle matemática de floats, reintrodujiste el bug que kit existe para prevenir.

![Una línea numérica en escala logarítmica muestra balances por debajo del límite de dos-a-la-cincuenta-y-tres renderizando exacto mientras valores más grandes redondean en silencio.](assets/v02-chart.webp)

Así que el contrato de formato del panel es matemática de BigInt en todo el camino hasta la cadena. La división da los SOL enteros, el resto da la fracción, y la convención de display es fija: nueve dígitos fraccionarios, rellenados a la izquierda, ceros finales recortados, sin punto decimal en los valores enteros.

```ts
export function lamportsToSol(lamports: bigint): string {
  const whole = lamports / 1_000_000_000n;
  const frac = lamports % 1_000_000_000n;
  if (frac === 0n) return whole.toString();
  const digits = frac.toString().padStart(9, "0").replace(/0+$/, "");
  return `${whole}.${digits}`;
}
```

El `padStart` es estructural y es donde esta función se escribe mal habitualmente: un resto de `2_500_000n` no es ".25", son nueve dígitos de fracción con dos ceros adelante, ".0025". Mi ejecución de esta función exacta contra el balance de mainnet de hoy: `1807515117625n` adentro, `"1807.515117625"` afuera, y `123456789123456789n` hace ida y vuelta a `"123456789.123456789"` sin perder un dígito. Ese último valor está por encima de 2^53 a propósito; es el fixture con el que el coding challenge va a golpear tu versión. Una trampa adyacente ya que estamos, porque muerde en la sección del worker: `JSON.stringify` lanza un `TypeError` sobre bigint. En cualquier lugar donde un valor de lamports cruce hacia JSON, lo conviertes a string explícitamente primero. El panel nunca hace JSON de sus bigints, pero el snapshot de KV sí tiene que hacerlo.

### El panel: el panel de Vercel aprende a dar la hora

El panel de Vercel ya sabe sondear: m03-l2 construyó el intervalo de `useEffect` con limpieza y una flag cancelled, y ese esqueleto se transfiere entero. Lo que cambia es la fuente de datos (el RPC, a través de kit, en vez de un archivo de status crudo) y una métrica derivada: el tiempo de slot medido. Dos muestras consecutivas de `getSlot` te dan un delta de slots y un delta de reloj de pared; divide y tienes el latido de la blockchain tal como lo observa tu panel, sentado al lado del target de 300ms. Y sí, este es el método ingenuo de dos muestras que la lección pasada construyó como algo descartable, revivido a propósito, así que di el trade-off en voz alta en vez de dejar que parezca amnesia: el medidor de banco quería veinte minutos de la historia registrada del nodo mismo para JUZGAR la blockchain, y el borrón de tu ida y vuelta habría contaminado ese veredicto; el panel quiere un latido vivo aproximado a un refresh de diez segundos por cero requests extra, y `getRecentPerformanceSamples` por tick gastaría un tercer request por pestaña contra los topes compartidos para comprar una precisión que un panel de estado no necesita. El borrón viaja adentro de cada lectura del panel, unos pocos milisegundos de ruido en estas ventanas, y el trabajo de la etiqueta es presentar el número como la observación que es, no como la medición de la lección pasada. La aritmética de slots pasa en bigint, y solo el delta chico final cruza a `number` para la división, que es la dirección correcta para cruzar: un delta de unas pocas docenas de slots no está ni cerca del acantilado.

Instala kit donde vive el panel (desde `packages/pulse-board`):

```bash
pnpm add @solana/kit
# resolved to 8.2.0 on 2026-09-02; the digit rule below explains why yours may differ
```

El esqueleto trabajado, `src/SolanaPanel.tsx`; compila limpio bajo modo estricto contra el kit que esa línea de instalación acaba de resolver, porque lo verifiqué antes de pegarlo acá:

```tsx
import { useEffect, useState } from "react";
import { createSolanaRpc, address } from "@solana/kit";
import { lamportsToSol } from "./lamports";

const RPC_URL = "https://api.mainnet.solana.com";
const WATCHED = address("So11111111111111111111111111111111111111112");
const POLL_MS = 10_000;
const TARGET_SLOT_MS = 300;

type PanelState =
  | { phase: "loading" }
  | { phase: "error"; message: string }
  | { phase: "ready"; slot: bigint; slotTimeMs: number | null; lamports: bigint };

const rpc = createSolanaRpc(RPC_URL);

export function SolanaPanel() {
  const [state, setState] = useState<PanelState>({ phase: "loading" });

  useEffect(() => {
    let cancelled = false;
    let last: { slot: bigint; at: number } | null = null;

    async function poll() {
      try {
        const slot = await rpc.getSlot().send();
        const { value: lamports } = await rpc.getBalance(WATCHED).send();
        const now = Date.now();
        let slotTimeMs: number | null = null;
        if (last && slot > last.slot) {
          slotTimeMs = (now - last.at) / Number(slot - last.slot);
        }
        last = { slot, at: now };
        if (!cancelled) setState({ phase: "ready", slot, slotTimeMs, lamports });
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

  if (state.phase === "loading") return <p>reading the chain...</p>;
  if (state.phase === "error") return <p>chain read failed: {state.message}</p>;

  return (
    <section>
      <h2>Solana</h2>
      <p>slot {state.slot.toString()}</p>
      <p>
        slot time{" "}
        {state.slotTimeMs === null
          ? "measuring..."
          : `${state.slotTimeMs.toFixed(0)}ms (target ${TARGET_SLOT_MS}ms)`}
      </p>
      <p>watched balance {lamportsToSol(state.lamports)} SOL</p>
    </section>
  );
}
```

Cada compás estructural es el panel de m03-l2: el `PanelState` discriminado, la flag cancelled, la limpieza que mantiene el hot reload en exactamente un poller vivo. El primer tick muestra "measuring..." para el tiempo de slot porque una tasa necesita dos muestras; eso es honestidad, no un bug.

Ahora el presupuesto, porque este componente gasta un recurso compartido en cada tick. Los topes del RPC público son 100 requests cada 10 segundos por IP, y 40 por método en la misma ventana. Este panel hace 2 requests por tick, y con un `POLL_MS` de diez segundos eso es 2 requests cada 10 segundos por pestaña abierta, 1 por método. Bien para ti, bien para el puñado de gente a la que le mandas la URL. Pero corre la aritmética a la manera de m02-l3 antes de confiar en ella: el carril por método se llena alrededor de las 40 pestañas sondeando con la misma cadencia detrás de un solo NAT, una oficina, un local, una residencia estudiantil. Pasado eso, cada visitante detrás de esa IP empieza a comerse 429s, y tu panel muestra errores causados por su propia popularidad. La lección pasada llamó a este desajuste el problema del panel desplegado y lo prometió; la sección del worker es la respuesta, y el status franco de ESTE panel es: las lecturas directas son correctas con tu tráfico, y en el momento en que no lo sean, el snapshot cacheado que estás por construir es a donde apunta el panel en cambio. Ese recableado se deja deliberadamente para el capstone, que sondea el JSON público del worker desde el panel como su única arista nueva.

![Un tick de polling gasta dos requests contra topes compartidos por IP, hace loop cada diez segundos, y se desvía por un estado de error cuando falla.](assets/v03-flowchart.webp)

### El worker: cachea la blockchain, sirve el cache

La relación del worker con la blockchain es distinta en naturaleza, y viene de la arquitectura de m07-l1 y no de nada específico de Solana. El panel sondea mientras un humano lo mira. El worker corre en un cron sin nadie mirando, escribe lo que aprendió en KV, y sirve la última verdad conocida a quien pregunte, desde la memoria del mundo y no desde una mirada fresca a él. Los datos de la blockchain encajan en ese modelo sin un solo cambio estructural: un refresh más en el handler scheduled, una clave más en KV, un bloque más en el JSON público.

El contrato del snapshot, y el único lugar donde la disciplina de bigint se encuentra con JSON:

```ts
export interface ChainSnapshot {
  slot: string;      // bigint, stringified: JSON.stringify throws on raw bigint
  balances: Record<string, string>;  // address -> lamports as string, same reason
  fetchedAt: string; // ISO timestamp: the snapshot's honesty field
}
```

`fetchedAt` no es decoración. Un cache que esconde su edad es un panel que miente; el consumidor del JSON decide qué quiere decir desactualizado, y solo puede decidir si lo estampas. El refresh en sí es el kata de m02-l3 vestido de blockchain: envuelve cada lectura en backoff (jitter parejo, el `backoffDelay` que extrajiste a pulse-core en m03-l1), y cuando falla al final no escribas nada, así el snapshot anterior sigue sirviendo mientras el próximo tick de cron reintenta. Degrada a desactualizado, nunca a vacío. Para una superficie de estado esa propiedad es todo el juego, y la construiste en el módulo dos sin saber que esta lección venía.

![Un carril manejado por cron refresca el snapshot de la blockchain hacia KV con backoff y dos salidas de falla, mientras un carril aparte sirve el snapshot cacheado a los requests.](assets/v04-flowchart.webp)

Esta arquitectura asume que kit corre adentro de workerd siquiera, una suposición de la que vale la pena desconfiar: el worker no es node, y la mitad del registro de npm se entera de eso por las malas. Corrí la prueba de humo mientras escribía esto: un worker hello-world, `npm i @solana/kit`, `createSolanaRpc(...).getSlot().send()` en el handler de fetch, `wrangler dev`. Kit cargó, resolvió, y ejecutó limpiamente adentro del isolate. El paquete trae una condición de export `workerd` explícita, y el runtime provee el Web Crypto que kit quiere, así que este es territorio soportado, no suerte. La respuesta real de mi sonda igual fue un error, y vale la pena leerla: HTTP 403, "Your IP or provider is blocked from this endpoint". El mismo 403 volvió para un POST de `fetch` pelado desde el mismo isolate, así que no fue kit, fue la blocklist del endpoint público a la que no le gustó el egress del entorno de mi sonda. Dos lecciones en un cuerpo de error. Primero, el transporte no es tu riesgo; la política del endpoint compartido lo es, y un proveedor bloqueado es un modo de falla más que tu camino de degradación ya absorbe. Segundo, esto es precisamente por lo que gana la arquitectura de snapshot: cuando un camino de lectura es rechazado, el JSON público del worker sigue sirviendo la última verdad que aprendió en vez de reenviarle el rechazo a cada visitante.

Dos cuestiones prácticas antes de que el lab lo cablee. El build de workerd de kit resuelve a su build de node, así que la fecha de compatibilidad de tu `wrangler.jsonc` importa: fechas del 2026-08-04 o posteriores habilitan la compatibilidad con node por defecto, y el scaffold de m07-l1 es más nuevo que eso, así que estás cubierto; una fecha más vieja necesita la flag `nodejs_compat` agregada a mano. Y la escotilla de escape, dicha porque una promesa le gana a un misterio: si kit-en-el-worker alguna vez te pelea, el fallback es una edición de una sola tabla, no una reescritura. Tu worker ya habla JSON-RPC crudo a este endpoint exacto, ya que la sonda `getHealth` de m07-l1 es un POST de fetch pelado. `getSlot` y `getBalance` son dos cadenas de método más en ese mismo estilo, y la arquitectura (cron, KV, backoff, degradar) no cambia ni una línea. La razón por la que kit es el camino enseñado igual es el tipo de dato: kit te da los lamports como bigint, mientras que `JSON.parse` sobre una respuesta cruda de RPC te da un `number` que ya redondeó cualquier cosa por encima del acantilado antes de que tu código llegue siquiera a verla.

### Fija la regla, no el dígito

Quizá notaste que esta lección no te ha dicho qué versión de kit instalar, solo te mostró a qué resolvió `pnpm add` en mi máquina en una fecha declarada. Eso es deliberado, y es la lección de peers de m03-l1 llegando a su pago. Entre el 2026-06-16 y el 2026-08-21, kit entregó tres releases en poco más de nueve semanas, dos de ellos majors: la minor 6.10.0, después 7.0.0, después 8.0.0, según los timestamps propios de npm. Cualquier tutorial que congeló un dígito en prosa durante ese tramo se pudrió antes del siguiente café de su autor, dos veces, y este curso se niega a sumarse a ellos. La línea de instalación que corres está verificada el día que la corres; los dígitos en prosa no.

![Una línea de tiempo de diez semanas marca una minor y dos releases majors sucesivos de la librería kit, terminando en la versión observada en la fecha de escritura.](assets/v05-timeline.webp)

Así que el pin se reduce a una regla: lee contra qué declaran peer las dependencias propias de tu workspace, y fija a eso. Los paquetes de cliente generado bajo `@solana-program/*` declaran, en sus `peerDependencies`, exactamente contra qué majors de kit fueron construidos, y npm hace cumplir ese contrato en tiempo de instalación. La lectura toma un comando. Acá va la salida en vivo de mi sonda al momento de escribir:

```bash
npm view @solana-program/system version peerDependencies
# version = '0.14.1'
# peerDependencies = { '@solana/kit': '^8.0.0' }
```

Hoy, esa salida nombra la major a la que resolvieron mis líneas de instalación. El día que la corras, puede nombrar la siguiente, y entonces ESA es tu respuesta, diga lo que diga cualquier tutorial o esta mismísima página. La regla corta para los dos lados, que es lo que la vuelve una regla en vez de un consejo: fija por debajo del rango de peer y la instalación falla a gritos; fuerza el paso más allá de un error de peer con una flag de override y entregas un desajuste de versión que falla en tiempo de ejecución en cambio, que es estrictamente peor, porque el gestor de paquetes estaba leyendo el contrato por ti y le dijiste que parara. Tu estación todavía no tiene paquetes `@solana-program/*`, las lecturas no necesitan ninguno, así que hoy el pin honesto es simplemente lo que resuelva `pnpm add @solana/kit`. La regla está en tus manos para el día en que un cliente generado entre al árbol, y el hábito más profundo generaliza más allá de Solana por completo: el hecho durable en cualquier ecosistema rápido nunca es el dígito, es dónde está escrito el dígito con autoridad. Sigue adelante en este catálogo y vas a encontrar la misma regla enseñada dos veces más, a propósito: el curso de pagos la enseña como una costura por workspace, dos workspaces en un repo fijados a majors de kit distintas porque sus dependencias declaran peer distinto, y el m08 del curso Anchor V2 la ejercita contra el rango de peer declarado de un cliente generado. Tres cursos, una regla, desde tres direcciones, porque es el único hábito que sobrevive al ritmo de releases de este ecosistema.

![Fijar al dígito de versión de un tutorial se pudre bajo el churn del ecosistema mientras que fijar a los rangos de peer de tus propias dependencias se actualiza con el contrato.](assets/v06-comparison.webp)

### Hacia dónde va kit, y cuándo lo gratis deja de alcanzar

Un cuadro sobre el futuro, para que los docs no te embosquen. La guía de upgrade del sitio de kit ahora abre con una API estilo plugin: `createClient()` con composición `.use(...)`. Es trabajo propio de kit, es hacia donde va la librería, y el panel no la usa. Los paquetes de plugin que la respaldan, `@solana/plugin-core` y `@solana/plugin-interfaces`, estaban en 8.2.0 en mi sonda al momento de escribir, al unísono con el propio kit, así que ningún dígito 0.x los va a señalar jamás como jóvenes; lo que fecha la superficie es ser la más nueva de la librería, en un ecosistema que entregó dos majors de kit en poco más de nueve semanas. El estilo pipe-y-RPC que enseña esta lección es el camino estable documentado, el dialecto que hablan hoy los ejemplos propios del ecosistema de kit y sus clientes generados, así que todo lo que cableaste acá es cimiento, no un callejón sin salida. Cuando los ejemplos propios de kit y sus clientes generados muevan su canon a createClient(), vuelve a correr este cálculo; el orden de la página de docs es marketing, lo que habla el código despachado del ecosistema es evidencia.

Y el libro de costos de lo que construiste, porque toda arquitectura es una cuenta. El snapshot que sirve el worker está desactualizado hasta por un intervalo de cron, quince minutos con tu calendario actual, más lo que sea que le agregue encima la consistencia eventual. Ese es el precio, y lo que compra es supervivencia: la alternativa era gastar el presupuesto compartido de 100-por-10-segundos en vistas de página, que con cualquier tráfico real convierte tu panel en un generador de 429 para todos los que están detrás de la misma IP. Para un panel de estado, los últimos-valores-conocidos con un `fetchedAt` honesto son el lado correcto de ese trade-off. Ten claro, eso sí, lo que NO construiste: en el momento en que tu producto necesite datos de la blockchain frescos por push, websockets, streaming, historial indexado, el polling-y-cache ya te quedó chico por completo. Esa profundidad, junto con todo lo de aterrizar transacciones, le pertenece al curso de dominio del lado del cliente, en producción mientras escribo; hasta que se entregue, esos nombres de tema son tus términos de búsqueda. Esta lección lee estado, y punto.

Cuando el endpoint público en sí deja de alcanzar, hay un próximo paso avalado que encaja con la regla de sin-tarjeta de este curso: Helius ofrece un tier gratuito a $0 con 1,000,000 de créditos al mes y 10 requests por segundo, sin tarjeta de crédito, según sus precios publicados al 2026-09-02. Cambiarlo es un cambio de URL en una constante, aplica la misma disciplina de sondeo, y ese único párrafo es todo lo que este curso tiene que decir sobre proveedores. La migración entera, cuando llegue el día:

```typescript
// the one line that changes when you outgrow the public door
const RPC_URL = "https://api.mainnet.solana.com"; // -> your provider URL, nothing else moves
console.log(new URL(RPC_URL).host);
```

**Profundiza (el 20%).** esta lección te enseñó el patrón de lectura de producción; la superficie completa de la API, cada método de RPC, las suscripciones, y el roadmap de plugins viven en la documentación de kit en https://solanakit.com, que es el recurso para dejar como bookmark, sondeado en vivo el 2026-09-02. Todo lo que te encuentres de acá en adelante con forma de RPC es una variación de la forma leer-sondear-cachear-degradar de la que ahora eres dueño.

## Lab: promueve la lectura

Estimados 45 minutos de construcción. Los pasos 1 y 2 están trabajados arriba; desde el paso 3 estás componiendo patrones de los que eres dueño contra contratos declarados.

1. **Chequeo de banco (hecho).** Si `npx tsx balance.ts` imprimió un bigint en la apertura, la lectura funciona desde tu máquina y tu install de kit está al día. Si imprimió un 403 con "blocked from this endpoint", relee la historia de la prueba de humo de la sección del worker: el egress de tu red está en la blocklist del endpoint, y el lab igual funciona porque las superficies desplegadas corren desde otras redes. Anota qué error te tocó; esa alfabetización es la lección.

2. **El formateador.** Crea `packages/pulse-board/src/lamports.ts` con `lamportsToSol` exactamente como lo especifica el contrato (nueve dígitos rellenados, recortados, sin punto en los enteros). Este archivo es también el target del coding challenge, y su lista de fixtures es la prueba de aceptación: `2500000n` renderiza `0.0025`, `1n` renderiza `0.000000001`, `123456789123456789n` hace ida y vuelta exacta.

3. **El panel.** Agrega `SolanaPanel.tsx` desde el esqueleto trabajado, móntalo en `App.tsx` al lado del panel existente, y móntalo AFUERA de los retornos tempranos del panel: el App trabajado retorna temprano en loading y en error, y un panel puesto debajo de esos retornos desaparece cada vez que la fuente de status está caída, que es precisamente cuando quieres el panel de la blockchain todavía visible. Después corre `npm run dev`. Checkpoint: el slot renderiza dentro de un tick, el tiempo de slot lee "measuring..." una vez y después un número en los 300 bajos, y el balance vigilado se muestra con una fracción plausible. Después empuja. El pipeline de m03-l3 hace el resto, y tu URL de producción de vercel.app ahora es un medidor de la blockchain. Ábrela en tu teléfono.

4. **El refresh del worker, tuyo para escribir.** En `pulse-edge-ts`: `npm i @solana/kit` (npm, no pnpm, y eso es correcto acá: el worker es su propio proyecto npm afuera del pnpm workspace de la estación, exactamente como lo armó m07-l1), después un `refreshChain(kv)` que implemente el contrato `ChainSnapshot`. La composición está completamente especificada por cosas de las que eres dueño: cada lectura envuelta en retries manejados por el `backoffDelay` de pulse-core con jitter parejo (base 500, cap 5000, la base y el cap de m02-l3; 3 retries máximo es el presupuesto más ajustado propio de este worker, ya que un tick de cron no tiene razón para esperar los cinco de la flota), los bigints pasados a string antes de `JSON.stringify`, `fetchedAt` estampado, `PULSE_KV.put("chain"...)` al tener éxito, y con los retries agotados: retorna sin escribir. Llámalo desde el handler scheduled después del loop de targets existente.

5. **Sírvelo.** Extiende el JSON del handler de fetch: lee la clave `chain` y devuélvela al lado de `targets`. Verificación local primero: `npx wrangler dev`, golpea la ruta de prueba scheduled (`curl "http://localhost:8787/cdn-cgi/handler/scheduled"`), después `curl http://localhost:8787/` y encuentra el bloque chain. Después `npx wrangler deploy`.

6. **Pincha producción.** La barrera, textualmente del verificador del curso:

   ```bash
   curl -s https://pulse-edge-ts.<your-subdomain>.workers.dev/ | grep -o '"chain"'
   ```

   Dos pestañas: el panel de Vercel y el JSON de workers.dev. El mismo slot, con un refresh de diferencia. Esa es la victoria de 30 segundos.

   Una escotilla de escape, porque la historia de la prueba de humo también puede venir por tu deploy: si el grep no encuentra nada y `npx wrangler tail` muestra cada refresh muriendo en errores 403 "blocked from this endpoint", el egress de tu worker está en la blocklist del endpoint público, y degradar-a-desactualizado no tiene a qué degradarse, ya que nunca se escribió ningún snapshot en un deploy fresco. Tres salidas honestas. Cambia `RPC_URL` por el fallback sin claves que documentó m07-l1, `https://solana-rpc.publicnode.com` (sin cuenta, una constante, verificado respondiendo getHealth desde adentro de workerd el 2026-09-04); o cámbialo por una URL de proveedor (el tier gratuito de Helius de la sección de costos: registro, sin tarjeta, cambia una constante); o quédate con el endpoint público y manda en cambio la salida de tail que muestra los 403s como tu evidencia de barrera. Un worker correctamente construido rechazado por la política de IP de un endpoint ha demostrado todo lo que este paso existe para probar, incluida la alfabetización en fallas, y un worker que hace failover a un fallback documentado ha demostrado una cosa más.

7. **Mata la blockchain, mírala degradarse.** En dev local, métele un typo a la constante de la URL del RPC, dispara la ruta scheduled, y confirma que el JSON servido todavía lleva el snapshot anterior con su `fetchedAt` más viejo. Un aviso de ruido para que no caces un bug fantasma: con el host irresoluble, la ruta de prueba scheduled de wrangler puede responder `exception` y workerd puede loguear líneas de "internal error" no capturadas incluso mientras tu `refreshChain` captura y degrada correctamente; la condición de aprobación es que el JSON servido siga llevando el snapshot viejo, no una respuesta limpia de la ruta de prueba. Saca el typo. Si en cambio un refresh fallido vació tu bloque chain, tu `refreshChain` escribió algo en el camino de falla; arregla eso antes que nada, es la propiedad para la que existe todo el diseño.

![El JSON público del worker gana un bloque chain que lleva un slot pasado a string, un mapa de balances, y su propio timestamp de fetched-at al lado de los targets existentes.](assets/v07-annotated-code.webp)

## Challenge: la segunda dirección viaja gratis

Solo, sin apoyo. Agrega una segunda dirección vigilada al panel y al worker, elegida por ti, cualquier cuenta de mainnet que te parezca interesante. La restricción que lo vuelve un ejercicio de diseño, dicha con precisión para que la solución perezosa falle: el worker mantiene exactamente UNA pasada de refresh, un presupuesto de backoff, y una escritura de KV por tick, con cada balance vigilado viajando adentro de esa misma pasada. Atornillar una segunda llamada a `refreshChain`, un segundo sobre de backoff, o una segunda clave de KV satisface la letra de "funciona" y reprueba el ejercicio; la llamada extra a `getBalance` en sí es el único request nuevo legítimo. Vas a encontrar la costura en tu código del paso 4 dentro del minuto de mirarlo: reestructurar `WATCHED` de constante a lista, si no lo construiste ya de esa forma, es todo el truco, y decidir si el polling directo del panel también debería hacer batch es tu decisión para tomar y defender en un comentario de código.

Al lado de eso, el coding challenge `lamports-to-sol` está en vivo en el runner del curso: el starter trae el clásico bug del resto sin rellenar, renderizando `2500000n` como `0.2500000`, equivocado por dos órdenes de magnitud en un display de dinero. Las pruebas incluyen el fixture de por-encima-de-2^53, así que una conversión furtiva a `Number()` no puede pasar. División de BigInt, resto, trabajo de cadenas, nada más.

Aceptación para toda la lección: las dos URLs desplegadas responden con datos en vivo de la blockchain; un target de RPC matado se degrada a último-estado-conocido en vez de dar error; un balance por encima de 2^53 lamports renderiza correctamente a través de tu formateador.

## Checkpoint, y en qué se acaba de convertir la estación

Di lo que ahora puedes hacer, porque es mucho vestido de poco: leer estado de la blockchain con kit en cada superficie de la que eres dueño, mantener honestos los enteros del tamaño del dinero desde el RPC hasta el pixel, presupuestar un tope de tasa compartido a través de una flota de navegadores de desconocidos, cachear una lectura detrás de un cron con un timestamp honesto y un camino de degradación, y elegir una versión de dependencia leyendo el contrato que declara tu propio árbol en vez de confiar en el dígito congelado de un desconocido. La pregunta de recuperación antes de cerrar la pestaña: tu panel muestra un balance de exactamente `9007199254740993` lamports; ¿por qué puedes confiar en él? Di la respuesta en una oración, y si la palabra bigint no está adentro, relee la sección del acantilado.

Pedido de feedback, específico esta vez: el paso 4 fue el bloque más grande de composición sin guía que el curso te ha dado. Dime dónde crujió. Si fuiste a la lección m02-l3 a re-derivar la forma del backoff, eso es el repliegue funcionando; si fuiste a ella porque el contrato de acá subespecificó algo, eso es un bug de esta lección, y quiero el número de línea.

La mitad de TS de la estación ahora lee la blockchain en producción, de banco a navegador a edge. Pero la estación tiene un segundo lenguaje y una tercera superficie: el poller de Docker de M6 sigue ciego a la blockchain, y no recibe kit, porque nadie envolvió el cable para Rust como kit lo envuelve para TS, y en la lección que viene eso resulta ser lo mejor que tiene. El camino de Rust va directo al cable de JSON-RPC con reqwest y serde_json, y el descubrimiento que espera ahí es que todo lo que M4 y M5 te enseñaron sobre parsear JSON no confiable y modelar errores ES código de cliente de Solana.
