# Parse, don't validate: zod en la frontera

## Resumen

La lección pasada reemplazaste los resultados de sonda tipados con cadenas de v0 por la unión `ProbeResult`. Los estados imposibles perdieron su codificación, y el switch del clasificador ahora es exhaustivo demostrado por el compilador. Pero esa demostración cubre solo los valores nacidos dentro del programa. Hoy la extiendes a valores nacidos afuera: el archivo de config de la flota, y su primera respuesta RPC de Solana de verdad. Vas a construir la capa de config de la flota pulse: un schema de zod v4 que rechaza basura en el arranque con un error a nivel de campo, un tipo `FleetConfig` derivado de ese schema para que el tipo y la validación nunca puedan desajustarse, un helper genérico honesto usado en las dos fronteras, y un parser de `getBalance` que expone los lamports como `bigint` porque un número de JavaScript te mentiría sin hacer ruido. Para el final, `npx tsx src/check-config.ts pulse.config.json` imprime un resumen tipado para una config buena y muere a los gritos, nombrando el campo roto exacto, para una mala.

## El patrón: un parser devuelve el tipo

Antes de cualquier teoría, corre el bug que esta lección existe para matar. Las perillas de la flota pasan hoy de arrays hardcodeados a un `pulse.config.json`, y la forma obvia de cargarlo es como lo hace la mitad de la producción: `JSON.parse` más un cast. Comete el crimen en miniatura, un archivo desechable, `src/naive-load.ts`. (Una nota de layout, ya que esta es la primera vez que ves `src/`: desde este módulo en adelante, el código nuevo de la flota vive en un directorio `src/`, así que `mkdir -p src` si no tienes uno. `probe.ts` y `fleet.ts` se quedan en la raíz del repo, donde los esperan las rutas de `npx tsx` que trae el workflow de m01-l3; los dos se van tejiendo juntos a medida que avanza el módulo.) El archivo es un loader basado en cast que lleva una config con un campo con typo, `"intervalSeconds"` donde el código lee `intervalSecs`:

```ts
type FleetConfig = { intervalSecs: number };
const raw = '{ "intervalSeconds": 60 }'; // the typo: Seconds, not Secs
const config = JSON.parse(raw) as FleetConfig;
const waitMs = (config.intervalSecs || 0) * 1000;
console.log(`waiting ${waitMs}ms between probes`);
```

```bash
npx tsx src/naive-load.ts   # prints: waiting 0ms between probes
npx tsc --noEmit            # exits clean. green.
```

Nada falla. No en la carga, no en el cast, no en la compilación. El `intervalSecs` que falta se lee como `undefined`, el fallback `|| 0` lo convierte en cero, y el loop de sondeo ahora no espera nada. Pon ese loader dentro del cron de GitHub Actions que viene del módulo uno y la flota sondearía alegremente en un loop caliente, martillando tus targets tan rápido como fetch pueda disparar, y cada línea individual de código involucrada parece correcta.

Acá está la parte que debería molestarte. La unión de la lección pasada no puede salvarte acá. `ProbeResult` protege los valores que construye tu propio código. La config nunca la construyó tu código. Fue leída del disco, pasada por `JSON.parse` hasta quedar en papilla con forma de `any`, y después en algún lugar hay una línea como esta:

```ts
const config = JSON.parse(readFileSync(path, "utf8")) as FleetConfig;
```

Ese `as FleetConfig` es el bug. He entregado exactamente esta línea, más veces de las que quiero contar, y siempre se siente segura porque el editor autocompleta hermoso después. Pero una aserción de tipo es un comentario que el compilador está obligado a creer. Los tipos de TypeScript se borran en tiempo de ejecución; la aserción no verifica nada, no convierte nada, no protege nada. Es una promesa que nadie verifica, y la basura del runtime desfila ante el compilador con la credencial de tu tipo puesta.

¿La bala de plata? Un parser.

La distinción tiene nombre, y es el título de esta lección. Un **validador** bendice los datos en su lugar: mira un valor, tal vez lanza, y te devuelve la misma cosa sin tipo que le diste, más una sensación reconfortante. Un **parser** es una función que o devuelve el valor tipado o rechaza. Después de que corre un parser, el tipo es VERDADERO, no aseverado. Ese es el patrón entero: parsea donde entran los datos, confía en los tipos después.

![Un archivo de config fluye a través de una aserción sin verificar hasta un bug en tiempo de ejecución, mientras el camino del parser se bifurca en un valor tipado o un rechazo ruidoso.](assets/v01-flowchart.webp)

### zod v4, el subconjunto de trabajo

Podrías escribir parsers a mano, y en Rust más adelante en este curso efectivamente lo harás. En TypeScript el ecosistema ya zanjó la pregunta. Instala zod:

```bash
npm i zod@4.5.4
```

Ese dígito es el último release al 2026-09-02; vuelve a chequear con `npm view zod version` antes de fijar. La prosa de este curso dice "zod v4" y cualquier 4.x que instales hoy va a correr este lab.

Un schema de zod es un valor que describe una forma y sabe cómo verificarla. Acá está el schema de config de la flota, entero, porque lo vas a construir en el lab y quiero que hayas visto el destino primero:

```ts
import { z } from "zod";

export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine((t) => t.timeoutMs < t.intervalSecs * 1000, {
    message: "timeoutMs must be under the probe interval, or probes pile up on a slow target",
    path: ["timeoutMs"],
  });

export const configSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(targetSchema).min(1),
});
```

Léelo de arriba abajo. `z.strictObject` describe un objeto y rechaza las claves que no conoce, que es exactamente lo que atrapa el typo de `intervalSeconds`: una clave desconocida es un error, no un encogimiento de hombros. `z.url()` y `z.number().int().positive()` empujan reglas dentro del schema que si no vivirían como sentencias `if` dispersas en cinco consumidores. Y `.refine` es la escotilla de escape entre campos: cualquier predicado sobre todo el objeto, con un mensaje que escribes tú y un `path` para que el error caiga en el campo que miraría un humano. Fíjate en las unidades haciendo trabajo de verdad en ese refinamiento: el intervalo está en segundos, el timeout en milisegundos, así que la comparación multiplica por 1000. Las reglas entre campos son precisamente donde se pudre primero la validación hecha a mano, porque ningún campo por sí solo es dueño de ellas.

### Error maps: escribe el mensaje para el operador a las 2am

Los mensajes por defecto de zod son correctos y un poco robóticos: "Invalid input: expected number, received undefined" te dice qué vio la máquina, no qué debería hacer el humano al respecto. En una frontera interna eso está bien, nadie los lee. En una frontera de CLI el texto del error ES la interfaz de usuario, y zod v4 te deja reemplazar cualquier mensaje en el punto donde se declara la regla, con un parámetro `error`:

```ts
const s = z.strictObject({
  url: z.url({ error: "url must be a full URL, scheme included (https://...)" }),
  intervalSecs: z.number({ error: "intervalSecs is the probe cadence in SECONDS, as a number" })
    .int()
    .positive({ error: "intervalSecs must be a positive number of seconds" }),
});
```

Dale a eso una config con `"url": "example.com"` y `"intervalSecs": -5` y el árbol impreso se lee como si lo hubiera escrito un colega:

```
✖ url must be a full URL, scheme included (https://...)
  → at url
✖ intervalSecs must be a positive number of seconds
  → at intervalSecs
```

La regla de la casa que uso para estos: un buen mensaje de frontera nombra la unidad, la restricción y, cuando puede, el POR QUÉ, porque la persona que lo lee está editando un archivo de config bajo presión de tiempo y tiene cero interés en tu sistema de tipos. Mete errores personalizados en el `targetSchema` del lab donde sea que un mensaje por defecto dejaría al operador adivinando; los criterios de aceptación no dependen de ellos, tu yo futuro sí. Hay todo un tier más de esta maquinaria (error maps por schema, localización) que la flota no necesita; si alguna vez entregas un producto donde los errores de validación llegan a usuarios finales, ese es el momento de ir a leerlo.

Dos maneras de correr un schema, y la diferencia importa en una frontera de CLI:

![Tarjetas lado a lado contrastan parse, que lanza al fallar, con safeParse, que devuelve un objeto de resultado que quien llama tiene que inspeccionar y manejar.](assets/v02-comparison.webp)

El arranque de la flota quiere `safeParse`. Un typo de config no es una excepción en tu lógica, es el error del operador, y lo más amable que puede hacer una CLI es imprimir exactamente qué está mal y salir con código distinto de cero para que el cron marque la ejecución en rojo. Vas a cablear eso en el lab con `z.prettifyError`, que convierte el error de zod en el árbol legible con el que un humano arregla un archivo de config.

### El schema es de donde viene el tipo

Ahora la jugada que hace este patrón sistemático en vez de solo ordenado. No escribes una interfaz `FleetConfig` al lado del schema. La derivas:

```ts
export type FleetConfig = z.infer<typeof configSchema>;
```

`z.infer` lee el tipo estático del schema de runtime. Una sola fuente de verdad. Esto honestamente es una bendición, y acá está el bug de desajuste que mata. Supón que el schema y una interfaz escrita a mano viven lado a lado. Alguien renombra `intervalSecs` a `intervalMs` en la interfaz durante un refactor, actualiza cada consumidor que el compilador marca, entrega. El schema todavía valida el nombre de campo VIEJO. Ahora configs válidas fallan la validación, o peor, la interfaz reclama un campo que el validador nunca verifica. Dos fuentes de verdad no se desajustan porque tu equipo sea descuidado; se desajustan porque son dos, y cada renombrado es una moneda al aire sobre cuál de las dos se actualiza. Con `z.infer` no hay moneda. Renombra el campo en el schema y cada consumidor del tipo se pone en rojo de inmediato, porque el tipo ES el schema. Vas a correr este ejercicio a propósito en el lab y mirar los errores en cascada.

![El schema de zod en el centro alimenta un tipo de TypeScript derivado hacia arriba a todos los consumidores y datos validados en tiempo de ejecución hacia abajo, reemplazando una interfaz separada escrita a mano.](assets/v03-diagram.webp)

Vale un momento de historia, porque esta idea es más grande que esta librería. zod entregó v3 el 2021-05-17 y tardó cuatro años en entregar un major, v4 el 2025-07-09. En esa ventana "parse, don't validate" (parsea, no valides) pasó de slogan de blog post a la cultura de fronteras de todo un ecosistema, 274.7M de descargas semanales de valor. La idea le quedó chica a la librería. Estás aprendiendo la idea; zod es solo la mejor herramienta actual para ella en TypeScript, y cuando este curso llegue a Rust vas a encontrar la misma disciplina corriendo en tiempo de compilación con serde.

![Una línea de tiempo corre desde el lanzamiento de zod 3 en 2021, pasando por cuatro años de adopción del ecosistema, hasta zod 4 en julio de 2025 y 274.7 millones de descargas semanales hoy.](assets/v04-timeline.webp)

### Generics, justo a tiempo

Este es el momento justo-a-tiempo que prometió el módulo: los genéricos aterrizan acá, exactamente cuando los necesitas, porque los has estado consumiendo durante tres párrafos sin un nombre.

Mira otra vez lo que escribiste. `z.array(targetSchema)`: le pasaste un valor con forma de tipo a una función y te devolvió un schema para arrays DE esa forma. `z.infer<typeof configSchema>`: le aplicaste una función a nivel de tipos a un tipo y te salió un tipo nuevo. Los dos son APIs genéricas, y en los dos casos CONSUMISTE el parámetro de tipo que declaró alguien más. Acá está la proporción honesta que nadie pone en la etiqueta: leer y aplicar los genéricos de alguien más es cerca del 90% de los genéricos que toca un desarrollador que trabaja. `Array<string>`, `Promise<Response>`, `Map<string, ProbeResult>`, `z.infer<typeof T>`. Lo has estado haciendo desde el módulo uno, cada vez que `await fetch(...)` te pasaba un `Promise<Response>` y el compilador sabía qué salía del `await`.

El modelo mental que hace legibles las firmas genéricas en los docs de cualquier librería: un parámetro de tipo es un argumento de función que resulta ser un tipo. `Array<T>` es una fábrica que toma un tipo y devuelve un tipo de array; `z.ZodType<T>` toma el tipo de salida y devuelve "un schema que produce eso". Cuando una firma parece intimidante, léela como lees una llamada a función: encuentra qué entra, encuentra dónde vuelve a salir, ignora la maquinaria del medio. Esa habilidad de lectura, no la habilidad de escritura, es lo que te desbloquea en codebases de verdad.

El otro 10% es escribir los tuyos, y esta lección necesita exactamente uno, porque la flota ahora tiene dos fronteras haciendo el mismo baile: leer datos crudos, pasarlos por safeParse, imprimir el árbol y morir si falla, devolver el valor tipado si hay éxito. Dos veces es un patrón:

```ts
export function parseOrExit<T>(schema: z.ZodType<T>, raw: unknown): T {
  const result = schema.safeParse(raw);
  if (!result.success) {
    console.error("boundary refused this input:");
    console.error(z.prettifyError(result.error));
    process.exit(1);
  }
  return result.data;
}
```

Lee la firma despacio, es la lección de genéricos entera. `<T>` declara una variable de tipo. `schema: z.ZodType<T>` dice "un schema que produce T", y `raw: unknown` es la disciplina de la lección pasada sosteniendo la línea: la entrada es intocable hasta que se demuestre. El tipo de retorno `T` cierra el círculo: lo que sea que produzca el schema, lo recibe quien llama, completamente tipado. Llámalo con `configSchema` y `T` se vuelve `FleetConfig`; llámalo con un schema de balance más adelante y `T` se vuelve eso. Un helper, las dos fronteras, cero casts. Esta también es la promesa de cierre de la lección pasada cobrada: `parseProbe` protegía una sola forma de par hecha a mano, y `parseOrExit` más un schema es esa misma guarda de frontera generalizada a cualquier frontera que puedas describir, que es lo que resulta significar "proteger toda la frontera". Fíjate en lo que NO hicimos: ni torres de `<T extends ...>`, ni tipos condicionales, ni trucos ingeniosos de inferencia. Las firmas genéricas elaboradas son una habilidad que puedes adquirir cuando una librería te obligue; hoy aprendes la forma que vas a usar de verdad cada semana.

### `satisfies`: verificado, no ensanchado

Una herramienta más y la teoría está lista. La flota quiere una config por defecto en el código, para desarrollo local cuando no se da ningún archivo. Tres maneras de escribirla:

```ts
// 1. No annotation: narrow types, zero shape checking. A typo'd key sails through
//    until something consumes it.
export const defaultConfig = { ... };

// 2. Annotation: shape checked, but WIDENED. fleetName is now just `string`;
//    the compiler forgot what you wrote.
export const defaultConfig: FleetConfig = { ... };

// 3. satisfies: shape checked AND every field keeps its narrow literal type.
export const defaultConfig = {
  fleetName: "pulse-dev",
  targets: [
    { name: "local", url: "http://localhost:3000/health", intervalSecs: 30, timeoutMs: 2000 },
  ],
} satisfies FleetConfig;
```

`satisfies` es verificación sin ensanchamiento. El compilador verifica que el literal se ajuste a `FleetConfig`, exactamente como lo haría la anotación, pero el tipo inferido propio del valor sobrevive: pon el cursor sobre `defaultConfig.fleetName` y ves el literal `"pulse-dev"`, no `string`. Con la anotación te llevas el peor trade-off en una constante: querías precisión Y la verificación, y calladamente vendió la precisión. Por qué le importa a la flota: el código aguas abajo puede ramificar en `cfg.fleetName === "pulse-dev"` con el compilador siguiendo el valor exacto, y una clave con typo en el default todavía no compila, algo que la opción 1 habría dejado pasar hasta que algún consumidor se tropezara con ella en producción. Para ser claros sobre lo que `satisfies` no es: es puramente en tiempo de compilación. No corre nada, no refina nada en tiempo de ejecución, nunca llama a tu `.refine`. El schema protege el archivo en disco; `satisfies` protege el literal en tu fuente. Fronteras distintas, herramientas distintas, y la flota ahora usa las dos sobre la misma forma.

![Tres versiones del mismo literal de config muestran que sin anotación queda sin verificar, con anotación queda verificado pero ensanchado, y con satisfies queda verificado conservando los tipos estrechos.](assets/v05-annotated-code.webp)

**Profundiza (el 20%).** esta lección te enseñó la disciplina de frontera y los genéricos que consumes a diario. El resto de la superficie de zod (transforms, brands, refinamientos async) y el oficio de escribir firmas genéricas elaboradas quedan deliberadamente como bookmark. Cuando los quieras: el tutorial gratis de Zod de Total TypeScript (totaltypescript.com/tutorials, 10 ejercicios, gratis al momento de escribir esto) es el mejor conjunto de ejercicios sobre la librería, y el capítulo Generics del TypeScript Handbook (typescriptlang.org/docs/handbook/2/generics.html) es el tratamiento canónico de cómo escribirlas. Haz el tutorial después de este módulo, no en vez del lab.

## Lab: la frontera que rechaza

El repliegue de la ayuda, dicho en voz alta: el paso 1 está completamente trabajado, escribes a la par y yo explico cada línea. Los pasos 2 y 3 son completions, te paso un esqueleto y escribes tú la parte estructural. El paso 4 es un ejercicio guiado donde el compilador es el que enseña. Ningún challenge sin guía en esta lección; los coding challenges del módulo están en las lecciones de uno y otro lado de esta, así que el lab y el quiz cargan la evaluación acá.

Estás trabajando en el repo de la flota del módulo uno. Se asume Node 24 LTS (ese es el LTS activo hoy; Node 26 toma la línea LTS el 2026-10-28, y nada en este lab cambia con eso). `tsx` y `typescript` han sido dependencias de desarrollo desde el build de v0; si te estás sumando de cero:

```bash
npm i -D tsx typescript @types/node
npm i zod@4.5.4
```

### 1. Ponle un schema a la config, cablea la frontera, mata el bug de la apertura (trabajado)

Crea `src/config.ts` con el schema que viste en la sección de teoría, más el tipo derivado, el default y el helper. Archivo completo, nada elidido:

```ts
import { z } from "zod";

export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine((t) => t.timeoutMs < t.intervalSecs * 1000, {
    message: "timeoutMs must be under the probe interval, or probes pile up on a slow target",
    path: ["timeoutMs"],
  });

export const configSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(targetSchema).min(1),
});

export type FleetConfig = z.infer<typeof configSchema>;

export const defaultConfig = {
  fleetName: "pulse-dev",
  targets: [
    { name: "local", url: "http://localhost:3000/health", intervalSecs: 30, timeoutMs: 2000 },
  ],
} satisfies FleetConfig;

export function parseOrExit<T>(schema: z.ZodType<T>, raw: unknown): T {
  const result = schema.safeParse(raw);
  if (!result.success) {
    console.error("boundary refused this input:");
    console.error(z.prettifyError(result.error));
    process.exit(1);
  }
  return result.data;
}
```

Dos líneas merecen comentario. `path: ["timeoutMs"]` apunta el error del refinamiento al campo que un operador realmente editaría; sin eso el mensaje cae en todo el objeto target, que es técnicamente cierto y prácticamente inútil. Y `process.exit(1)` dentro de `parseOrExit` es lo que hace que el helper sea honesto sobre su nombre: el tipo dice "returns T", y la única forma de que eso sea siempre cierto es que la rama de falla no retorne jamás. TypeScript entiende esto porque `process.exit` devuelve `never`, así que el compilador demuestra que la rama de éxito es la única rama que llega a `return`.

Ahora el script de frontera, `src/check-config.ts`:

```ts
import { readFileSync } from "node:fs";
import { configSchema, parseOrExit, type FleetConfig } from "./config.js";

const path = process.argv[2] ?? "pulse.config.json";
const raw: unknown = JSON.parse(readFileSync(path, "utf8"));

const config: FleetConfig = parseOrExit(configSchema, raw);

console.log(`fleet "${config.fleetName}": ${config.targets.length} target(s)`);
for (const t of config.targets) {
  console.log(`  ${t.name} -> ${t.url} every ${t.intervalSecs}s, timeout ${t.timeoutMs}ms`);
}
```

Dos detalles antes de correrlo, y los dos se han comido una tarde de alguien. El import dice `./config.js` aunque el archivo en disco sea `config.ts`; esas son las reglas de resolución de ESM, donde los especificadores de import nombran el archivo de SALIDA, y `tsx` lo resuelve correctamente, así que no "arregles" la extensión. Y fíjate en el tipo de `raw`: `unknown`, nunca `any`. Esa es la regla de la lección pasada encontrándose con la herramienta de esta lección; `JSON.parse` devuelve `any`, y anotar el binding como `unknown` lo desenvenena así nada aguas abajo puede tocarlo sin parsear, lo que quiere decir que la ÚNICA forma de llegar desde acá a una config usable es a través del parser. El compilador ahora hace cumplir el patrón que le da nombre a esta lección. Y un `pulse.config.json` de verdad en la raíz del repo:

```json
{
  "fleetName": "pulse-prod",
  "targets": [
    {
      "name": "docs",
      "url": "https://example.com",
      "intervalSecs": 60,
      "timeoutMs": 3000
    },
    {
      "name": "api",
      "url": "https://example.org/health",
      "intervalSecs": 30,
      "timeoutMs": 2000
    }
  ]
}
```

Corre la frontera:

```bash
npx tsx src/check-config.ts pulse.config.json
```

Deberías ver el resumen parseado y tipado:

```
fleet "pulse-prod": 2 target(s)
  docs -> https://example.com every 60s, timeout 3000ms
  api -> https://example.org/health every 30s, timeout 2000ms
```

Ahora el momento por el que existe este lab. Copia la config a `pulse.config.broken.json` y comete de verdad el typo de la apertura: renombra la clave `intervalSecs` del primer target a `intervalSeconds`. Corre la frontera contra ella:

```bash
npx tsx src/check-config.ts pulse.config.broken.json
```

```
boundary refused this input:
✖ Unrecognized key: "intervalSeconds"
  → at targets[0]
✖ Invalid input: expected number, received undefined
  → at targets[0].intervalSecs
```

Código de salida 1. Compara esto contra la ejecución de naive-load de la apertura, porque este es el antes/después que importa: el mismo archivo con el que v0 corría en verde ahora no puede entrar al programa. La falla no se mudó a un lugar más lindo; dejó de existir en tiempo de ejecución y se volvió un rechazo en el arranque con el campo exacto nombrado dos veces, una como la clave desconocida que escribiste y otra como la clave requerida que dejaste sin alimentar. Mi ejecución rota imprimió exactamente esos dos errores y nada más, que es la otra cosa que te compra un buen árbol de errores: sin scroll, sin arqueología de stack traces, solo el arreglo.

![El arranque de la flota fluye desde el cron, pasando por la lectura y el parseo del archivo, hasta parseOrExit, que o admite una config tipada al loop de sondeo o sale en rojo para el operador.](assets/v06-flowchart.webp)

### 2. El refinamiento entre campos (completion)

Tu turno de escribir la regla. Borra el `.refine` de `targetSchema` y reconstrúyelo tú mismo a partir de este esqueleto:

```ts
export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine(
    (t) => /* your predicate: the timeout budget must fit inside the probe interval */,
    {
      message: /* your message: say WHY, not just what */,
      path: [/* aim it at the field the operator should edit */],
    },
  );
```

Ojo con las unidades; el intervalo es segundos y el timeout es milisegundos, así que el predicado honesto es `t.timeoutMs < t.intervalSecs * 1000`. Un predicado con unidades mezcladas es exactamente el tipo de regla que nunca sobrevive como conocimiento tribal, y por eso vive en el schema y no en un comentario de code review. Después demuestra que funciona. Haz una copia de la config buena con un target puesto en `"intervalSecs": 2, "timeoutMs": 5000` y corre el verificador contra ella:

```
boundary refused this input:
✖ timeoutMs must be under the probe interval, or probes pile up on a slow target
  → at targets[0].timeoutMs
```

Aceptación: la ejecución sale con código distinto de cero y el error cae en `targets[0].timeoutMs` con tu mensaje, como la salida de arriba. Si tu mensaje solo repite la matemática, reescríbelo; dentro de seis meses el operador que lo lea no va a recordar por qué existe la regla, y "probes pile up on a slow target" es la diferencia entre un arreglo y un workaround.

### 3. La frontera con forma de blockchain: getBalance como bigint (completion)

Ahora la segunda frontera, y la razón por la que este curso está derivando hacia Solana. La flota va a vigilar infraestructura de blockchain con el tiempo, así que su primera lectura de blockchain pasa acá, sin SDK, porque una llamada JSON-RPC es solo un POST y ya eres dueño de una disciplina de parser.

Una sola oración para orientarte y nada más: en Solana, los balances viven en cuentas y están denominados en lamports, un conteo entero de la unidad más pequeña de la blockchain, y todo lo más profundo sobre qué ES una cuenta le pertenece al curso de evolución de Bitcoin a Solana, que recorre ese modelo de punta a punta. La llamada en sí es la forma JSON-RPC que adivinarías: haz POST de un nombre de método y params, recibe un result de vuelta. Acá está un cuerpo de respuesta de verdad, del endpoint que estás por golpear, capturado mientras escribía esta lección:

```json
{"jsonrpc":"2.0","result":{"context":{"apiVersion":"4.2.1","slot":443610065},"value":1},"id":1}
```

Ese `"value":1` es el balance en lamports, y llega como un número JSON pelado, lo que nos lleva a lo que de verdad importa en este paso: el tipo de ese entero. Los balances de lamports son u64 en el cable: un entero sin signo de 64 bits cuyo máximo es 18446744073709551615. Los números de JavaScript son doubles, exactos solo hasta `Number.MAX_SAFE_INTEGER`, que es 9007199254740991. Cualquier u64 más allá de eso se redondea en silencio. Un balance de un lamport pasa por `JSON.parse` intacto; el balance de una ballena no tiene por qué. Corre la mentira tú mismo, una línea:

```bash
node -e "console.log(JSON.parse('{\"value\":9007199254740993}').value)"
```

```
9007199254740992
```

Uno de diferencia, sin error, sin advertencia, y pasó dentro de `JSON.parse` antes de que ningún schema pudiera mirar el valor. Un balance equivocado por unos pocos lamports sin error en ninguna parte es la mentira más cortés de este curso hasta ahora. Así que el arreglo no puede ser "validar el número después"; el daño precede a la validación. El arreglo es interceptar el texto crudo antes de que se vuelva un double.

![Una línea numérica marca el límite de entero seguro de JavaScript, el primer valor que se redondea en silencio, y el máximo u64 mucho más grande que los balances de lamports pueden alcanzar.](assets/v07-chart.webp)

Node 21 y más nuevos dan el punto de intercepción: `JSON.parse` le pasa a tu reviver un objeto de contexto que lleva el texto fuente crudo de cada primitivo, así puedes construir un `BigInt` a partir de los dígitos antes de que el double exista. Los tipos que trae TypeScript todavía no se pusieron al día con ese tercer argumento del reviver, así que el archivo lo salva con un solo alias tipado, que es en sí una pequeña lección franca: los runtimes se entregan antes que los tipos. Crea `src/balance.ts` a partir de este esqueleto y completa las dos partes marcadas:

```ts
import { z } from "zod";
import { parseOrExit } from "./config.js";

// Plain z.object here, not strictObject, on purpose: this boundary reads
// someone ELSE's shape, and the RPC server may add fields (apiVersion already
// rides along) without that being your bug. Strictness is for shapes you own,
// like the config; tolerance of unknown keys is for shapes you only consume.
const balanceResponseSchema = z.object({
  jsonrpc: z.literal("2.0"),
  id: z.number(),
  result: z.object({
    context: z.object({ slot: z.number() }),
    // YOUR SCHEMA (a): the balance field. It must come out as bigint, not number.
  }),
});

const RPC_URL = "https://api.mainnet.solana.com";
const address = process.argv[2] ?? "Vote111111111111111111111111111111111111111";

const res = await fetch(RPC_URL, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "getBalance",
    params: [address],
  }),
});

const text = await res.text();

// Node 21+ passes a { source } context to the reviver; TypeScript's lib types
// have not caught up yet, so bridge the gap with one typed alias.
type ReviverWithSource = (
  this: unknown,
  key: string,
  value: unknown,
  context?: { source?: string },
) => unknown;

const parseWithSource = JSON.parse as (text: string, reviver?: ReviverWithSource) => unknown;

const raw: unknown = parseWithSource(text, (key, value, ctx) =>
  /* YOUR REVIVER (b): when the key is "value" and ctx.source exists,
     build the BigInt from ctx.source; otherwise return value unchanged */
);

const body = parseOrExit(balanceResponseSchema, raw);

console.log(`slot ${body.result.context.slot}`);
console.log(`balance: ${body.result.value} lamports (${typeof body.result.value})`);
```

Para (a) la respuesta es una línea, `value: z.bigint()`, y hace más de lo que parece: si tu reviver alguna vez deja de correr, el schema falla a los gritos en vez de dejar que un double redondeado se haga pasar por un balance. Para (b): `key === "value" && ctx?.source !== undefined ? BigInt(ctx.source) : value`. El reviver ve cada clave llamada `value` en el documento; acá solo el balance coincide, y la guarda sobre `ctx?.source` te mantiene honesto porque el contexto solo lleva texto fuente para primitivos.

Córrelo exactamente una vez:

```bash
npx tsx src/balance.ts
```

```
slot 443609276
balance: 1 lamports (bigint)
```

Tu slot va a diferir; el balance del programa de voto genuinamente es 1 lamport, y la palabra entre paréntesis es la verificación de aceptación: `bigint`. Dale una dirección más movida como primer argumento si quieres un número grande, pero ojo con el exactamente una vez: `https://api.mainnet.solana.com` es el endpoint público, tiene rate limit, y la documentación de Solana dice sin vueltas que no está pensado para aplicaciones de producción. Un fetch en este lab es una visita de cortesía. Cincuenta targets en un cron es un ban, y la forma de ese límite es precisamente donde arranca la próxima lección.

### 4. El ejercicio de desajuste (descubrimiento guiado)

Último paso, y el punto de `z.infer` hecho físico. En `src/config.ts`, renombra el campo del schema `intervalSecs` a `intervalMs`. No cambies nada más. Ahora corre el compilador sobre el proyecto:

```bash
npx tsc --noEmit
```

Mira la cascada: `check-config.ts` se pone en rojo donde imprime `t.intervalSecs`, el refinamiento se pone en rojo dentro del schema mismo, `defaultConfig` se pone en rojo bajo su `satisfies`. Cada consumidor del tipo se enteró del renombrado al instante, porque hay exactamente un lugar de donde viene el tipo. Este es el bug de desajuste que viste en la sección de teoría, corriendo al revés: con dos fuentes de verdad este renombrado habría sido una divergencia silenciosa; con una sola fuente es una checklist escrita por el compilador de cada sitio que tiene que decidir. Revierte el renombrado, corre `npx tsc --noEmit` otra vez, confirma verde.

**Verifica antes de seguir**: `npx tsx src/check-config.ts pulse.config.json` imprime el resumen tipado de dos targets y sale con 0. El mismo comando contra `pulse.config.broken.json` imprime el árbol de errores nombrando `intervalSeconds` e `intervalSecs` y sale con código distinto de cero. Tu copia con timeout malo es rechazada con tu mensaje de refinamiento en `targets[0].timeoutMs`. Y `npx tsx src/balance.ts` imprime una línea de lamports que termina en `(bigint)`.

## Challenge

La flota publica `status.json` en cada ejecución del cron; construiste ese archivo en el módulo uno y desde entonces has estado confiando en tu propia salida. Basta. Escribe `src/check-status.ts`: un schema de zod para `status.json` tal como tu flota lo escribe de verdad, un tipo `StatusReport` derivado con `z.infer`, y un parseo del archivo a través del mismo helper `parseOrExit`, imprimiendo una línea de resumen por target en caso de éxito. Restricciones: el schema debe ser estricto, al menos un campo necesita una regla más ajustada que su tipo primitivo (un formato de timestamp vía `z.iso.datetime()`, el único validador de acá que la lección no enseñó, así que esa opción te cuesta una búsqueda en la documentación; un array no vacío; una latencia que no pueda ser negativa), y ningún helper nuevo; `parseOrExit` fue escrito genérico precisamente para que esta tercera frontera te cueste cero plomería nueva.

```bash
npx tsx src/check-status.ts status.json
```

Aceptación: tu `status.json` real actual parsea limpio, y editar a mano un campo hasta volverlo basura queda rechazado con un error legible a nivel de campo. Corre enteramente local, así que no hay RPC involucrado. Si el schema te da pelea porque tu propio formato de salida es inconsistente entre ejecuciones, felicitaciones: el parser acaba de encontrar un bug de verdad, y arreglar el escritor es parte del challenge.

## Donde termina la frontera

Hora de ser franco sobre qué compraste y qué costó. Un parser en cada frontera cuesta una dependencia, un schema que mantener junto a cada cambio de config, y trabajo en tiempo de arranque, y los árboles de error de zod pueden de verdad abrumar cuando los schemas se anidan profundo: una falla cuatro niveles abajo en una unión anidada imprime un árbol que exige lectura de verdad, y por eso la flota mantiene su config plana y sus mensajes escritos a mano. La disciplina también tiene una frontera, y saber dónde se detiene importa tanto como adoptarla. Parsea en las FRONTERAS, los lugares donde los datos entran desde afuera de tu sistema de tipos: un archivo de config en disco, una respuesta HTTP, el entorno (cuando la flota tenga secretos en los módulos de deploy, `process.env` también recibe un schema, y por la misma razón). En ningún otro lado. Las funciones internas que se pasan entre sí valores ya parseados por un schema deberían confiar en sus tipos; revalidar entre tus propias funciones es ruido que dice que no le crees a tu propio compilador, y si eso es cierto los tipos eran inútiles. Y un parseo aprobado demuestra forma, nunca verdad. Una config bien formada todavía puede apuntar sondas a la URL equivocada; una respuesta RPC bien formada todavía puede estar desactualizada para cuando actúes sobre ella; el schema no puede conocer tu intención, solo tu estructura. Parsear te compra exactamente una oración: "estos datos tienen la forma sobre la que razoné". Resulta que esa es la única oración que el compilador necesitaba para volver real cada garantía aguas abajo.

Tu victoria de treinta segundos, dila en voz alta antes de cerrar la pestaña: un validador bendice los datos en su lugar; un parser DEVUELVE el valor tipado, así que después de que corre el tipo es verdadero por construcción. Si puedes decir eso y señalar la línea en `parseOrExit` donde pasa, tienes esta lección.

Si algo del lab te dio pelea, o el truco del reviver te pareció que merecía un por qué más profundo, dime: ese feedback dirige dónde gasta el curso su profundidad, y las lecciones de frontera son las que quiero más afinadas a donde la gente realmente resbala. Tus fronteras ahora rechazan basura. Así que apunta la flota a cincuenta targets reales a la vez, y descubre que internet te rechaza a TI: rate limits, sockets colgados y un muro de 429. La próxima lección es async que sobrevive al contacto: concurrencia como presupuesto, backoff con jitter, y cancelación. Llévate un rate limit.
