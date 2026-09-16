# Haz que los estados imposibles sean irrepresentables

**Resumen:** El módulo 1 entregó el latido: pulse v0 sondea una URL con el fetch integrado, y el cron de Actions de m01-l3 lo corre según el calendario y le hace commit a status.json, una máquina que no es tuya, corriendo tu código. También corre tus bugs. Esta lección es sobre un bug en particular, del tipo que nunca se cae. Vas a forjar un registro de sonda malformado, vas a ver a v0 publicarlo como healthy sin quejarse, y después vas a borrar la categoría entera de ese bug reconstruyendo el tipo de resultado de la flota como una unión discriminada con un switch exhaustivo. Al final, un estado equivocado no va a ser atrapado. No va a ser construible.

Mide primero. Abre tu repo de pulse y pega estas cuatro líneas al principio del archivo de la sonda (el principio importa: la guarda de uso de v0 sale temprano cuando no se le pasa ninguna URL), después córrelo con `npx tsx probe.ts`:

```ts
type ProbeRecord = { status: string; latencyMs?: number };

const forged: ProbeRecord = { status: 'okay' };
console.log(forged.status !== 'timeout' ? 'healthy' : 'down');
```

Imprime `healthy`, después la guarda de uso de v0 se queja por la URL que falta y sale con 1. Ignora la línea de la guarda; está haciendo su trabajo viejo. La línea que importa es la primera. Vuelve a leer el registro forjado. Su status es la cadena `'okay'`, no `'ok'`. Ninguna sonda corrió. No hay latencia. Y aun así: healthy. Sin excepción, sin texto rojo, nada en lo que el cron pueda fallar. Si este registro estuviera en status.json ahora mismo, tu workflow le haría commit en verde, y el panel que vas a construir en el módulo 3 renderizaría como up un target que nunca fue sondeado de verdad.

Nada se cayó. Ese es el problema. Quédate un segundo con esa frase, porque es toda la lección: en v0, un estado equivocado es representable, así que fluye. Cada consumidor de abajo o lo re-verifica o confía en él, y el que confía le miente a quien mire.

Tu instinto podría ser "entonces agrega una verificación". Guarda ese pensamiento. Vamos a hacer algo mejor que verificar. Vamos a hacer que el estado equivocado sea imposible de escribir.

## El patrón: estados que no puedes escribir mal

### Una forma que puede mentir

Aquí está la forma del resultado de v0, reducida a los dos campos que deciden la salud. Es prima cercana del registro que tu sonda viene escribiendo desde m01-l2, ese también lleva `url` y un timestamp, y comete el mismo pecado con otra ortografía: m01-l3 entregó `latencyMs: number | string`, un timeout registrado como prosa:

```ts
type ProbeRecord = { status: string; latencyMs?: number };
```

Dos campos. Parece inofensivo. Ahora cuenta lo que puede decir. `status` es `string`, lo que quiere decir que puede guardar `'ok'`, `'timeout'`, `'okay'`, `'OK '` con un espacio al final, o el texto completo de Moby Dick. `latencyMs` es opcional, lo que quiere decir que cada uno de esos status viene en dos sabores: con un número, o sin uno. El tipo codifica alegremente todos estos:

- `{ status: 'ok' }` sin latencia. Una sonda "ok" que no midió nada.
- `{ status: 'okay', latencyMs: 200 }` el typo que acabas de forjar, ahora vistiendo una latencia plausible.
- `{ status: 'timeout', latencyMs: 143 }` un timeout que de alguna forma tiene latencia.

Ninguno de estos estados puede pasar en la realidad. Una sonda exitosa siempre tiene latencia. Un timeout nunca. Pero el tipo no puede decir eso, así que cada consumidor de `ProbeRecord` tiene que volver a derivar la realidad en el punto de uso: verificar la cadena de status, verificar si la latencia está ahí, decidir qué quiere decir un campo que falta. Cada verificación es un lugar para olvidar una verificación. Tu registro forjado pasó de largo porque un consumidor, esa pequeña línea `!== 'timeout'`, hizo una suposición de apariencia razonable que el tipo nunca prometió.

![La realidad permite tres resultados de sonda mientras que el tipo con cadenas también acepta muchas combinaciones inválidas, como un timeout que lleva una latencia.](assets/v01-comparison.webp)

### Por qué el código que mueve dinero no puede encogerse de hombros

Ahora el por qué, porque este curso te prometió el por qué y esta es la lección que se lo gana.

Empieza por cómo web2 maneja este bug exacto. El registro forjado se entrega. Algún panel muestra un mosaico verde desactualizado. Un usuario abre un ticket, alguien grepea los logs, un arreglo sale el martes. El estado equivocado te costó una disculpa. Esta es una forma válida de vivir, y es por eso que un montón de empresas de web2 todavía entregan felices sobre JavaScript a secas: cuando los errores son baratos y reversibles, la demostración en tiempo de compilación es un impuesto que puedes rechazar racionalmente.

Web3 rompe esa aritmética. El código hacia el que este curso te va caminando, el código del que están hechos los codebases web3 de verdad, mueve valor. Una transacción que pasa la verificación de tipos hasta existir con el estado equivocado no produce un ticket. Produce una transferencia que ya pasó, en un ledger cuya meta de diseño entera es que nadie pueda deshacerla en silencio. Una cuenta drenada no se des-drena. El costo de un estado equivocado deja de ser "una disculpa" y se vuelve ilimitado e irreversible, y una vez que eso es verdad, el seguro más barato del mercado es una demostración que el compilador va a hacer gratis, en cada build, para siempre.

Así que haz la pregunta más filosa: ¿qué compra exactamente una demostración en tiempo de compilación que una verificación en tiempo de ejecución no puede? Una verificación en tiempo de ejecución es una frase que alguien tiene que acordarse de escribir, en cada sitio, en cada refactor, para siempre. Corre cuando llega el valor malo, lo que quiere decir que corre en producción, en el peor momento posible, si llegó a escribirse. Una demostración en tiempo de compilación es distinta en clase, no en grado: es validación que nunca tienes que recordar, porque el programa que contiene el error no es un programa. Nunca compila, así que nunca existe, así que nunca corre. La validación que nunca tienes que recordar le gana a la validación que alguien tarde o temprano va a olvidar. Ese es todo el trade-off, y web3 es el ambiente donde el precio de olvidar finalmente hizo que todos pagaran por la demostración.

![Un estado equivocado se convierte en un ticket arreglable en web2, en una transferencia irreversible en web3, y no se entrega nunca cuando el compilador lo rechaza.](assets/v02-flowchart.webp)

El ecosistema viene votando esto con los pies desde hace un rato, y 2026 nos dio la boleta más ruidosa hasta ahora. Entre marzo de 2025 y 2026-07-08, Microsoft portó el compilador de TypeScript a Go y lo entregó como TS 7: un ecosistema tan comprometido con la demostración en tiempo de compilación que reconstruyó el demostrador mismo por velocidad. Ese no es el comportamiento de una comunidad que cree que los tipos son lint. Los tipos son infraestructura estructural en 2026, y el compilador que verifica tu sonda hoy es el más rápido jamás construido precisamente porque ahora hay tanto apoyado en él.

![Una línea de tiempo que va desde el anuncio del port a Go, en marzo de 2025, hasta TypeScript 7 entregado el 8 de julio de 2026.](assets/v03-timeline.webp)

### La unión: cada variante lleva exactamente sus propios datos

Antes del arreglo, descarta los no-arreglos tentadores, porque te vas a topar con los tres en codebases de verdad y cada uno falla por una razón que vale la pena hacer propia.

Arreglo ingenuo uno: validar en todas partes. Escribe un helper `isValidRecord` y llámalo en cada sitio de uso. Esto funciona hasta el día en que alguien agrega un sitio de uso y no sabe que el helper existe, que en un codebase que crece es el martes que viene. Convertiste un problema de diseño en una prueba de memoria, y el modo de falla es silencioso, exactamente como el que acabas de ver.

Arreglo ingenuo dos: probar más duro. Escribe una prueba unitaria para el caso `'okay'`. Buen instinto, herramienta equivocada: una prueba demuestra que las entradas que sí pensaste se portan bien, y toda la identidad de este bug es ser la entrada que nadie pensó. Las pruebas muestrean el espacio de estados. Tenemos que encogerlo.

Arreglo ingenuo tres: comentarios y disciplina. Documenta que status tiene que ser `'ok'` o `'timeout'` y confía en el equipo. Este es el que todo el mundo entrega de verdad, y es el hábitat natural de la mentira cortés, porque un comentario compila sin importar lo que haga el código.

Fíjate en la forma de las tres fallas: cada una deja el estado equivocado representable y después pone una guarda en algún lado, esperando que la guarda esté siempre despierta. Así que el arreglo de verdad invierte el enfoque. No más guardas. Una forma que no tiene codificación para las mentiras.

Pregúntate qué es en realidad un resultado de sonda. Es una de exactamente tres historias: el target respondió a tiempo, el target nunca respondió, o el target respondió con un error HTTP. Cada historia viene con su propia evidencia y, esto es crucial, solo con su propia evidencia. TypeScript te deja escribir eso directamente:

```ts
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };
```

Esta es una unión discriminada. Tres formas de objeto unidas por `|`, que comparten una propiedad, `kind`, cuyo tipo en cada rama no es `string` sino un único literal: exactamente `'ok'`, exactamente `'timeout'`, exactamente `'http-error'`. Esa propiedad literal compartida se llama el discriminante, y es lo que hace que todo el patrón encaje, porque el compilador puede distinguir las variantes mirándola.

Recorre las variantes como un auditor. `'ok'` lleva `latencyMs`, obligatorio, porque un éxito sin medición no es un éxito. `'timeout'` lleva `budgetMs`, el presupuesto que reventó, y no tiene ningún campo de latencia, ni uno opcional, ninguno. `'http-error'` lleva el código `status` que devolvió el servidor. Nada opcional en ninguna parte. Ahora vuelve a correr las mentiras de ayer contra este tipo. ¿Un ok sin latencia? Falta una propiedad obligatoria, no hay codificación. ¿Un `'okay'` con typo? No es uno de los tres literales, no hay codificación. ¿Un timeout con latencia? `latencyMs` no existe en esa variante, no hay codificación. Las combinaciones incoherentes no quedaron atrapadas. Dejaron de tener ortografía.

Te lo confieso: he entregado la versión con campo opcional de esto más veces de las que quiero admitir. `latencyMs?: number` se siente tan razonable cuando lo escribes, un campo, cubre los dos casos, vas rápido. La opcionalidad es exactamente por donde los estados imposibles se vuelven a colar. Cada `?` en un campo que en realidad es "presente en algunas variantes" es una puertita que dejaste abierta, y algo acabará entrando por ahí vistiendo la cadena de status equivocada. La disciplina de la unión, cada variante lleva exactamente sus propios datos, es el hábito que mantiene la puerta cerrada.

### Narrowing en serio

Acá aterriza una objeción justa: bueno, el tipo es honesto, pero `result.latencyMs` ya no compila en absoluto, porque `latencyMs` solo existe en una de las tres ramas. ¿Acabamos de volver el tipo inutilizable?

No. Lo hicimos exigir una demostración antes de usarlo, y el narrowing de TypeScript (el estrechamiento de tipos) es cómo das esa demostración. Verifica el discriminante y el compilador estrecha la unión al único brazo que coincide, dentro de ese bloque nada más:

```ts
function describe(result: ProbeResult): string {
  if (result.kind === 'ok') {
    return `${result.latencyMs.toFixed(1)}ms`;
  }
  if (result.kind === 'http-error') {
    return `HTTP ${result.status}`;
  }
  return `no answer in ${result.budgetMs}ms`;
}
```

Dentro de la primera rama, `result` es `{ kind: 'ok'; latencyMs: number }` y nada más, así que `latencyMs` está garantizado, sin verificación opcional, sin undefined. Dentro de la segunda, `status` está garantizado. Y mira la última línea: después de que dos verificaciones eliminaron dos variantes, el compilador estrechó lo que quedaba a `'timeout'` él solito, así que `budgetMs` simplemente funciona. Nunca se lo dijiste. Él hizo la eliminación.

El mismo motor de narrowing corre con combustible más simple también. `typeof value === 'string'` estrecha un `unknown` a `string` dentro del bloque. `value === null` estrecha un `T | null` a `T` en la rama else. Las verificaciones de discriminante, las verificaciones de `typeof`, las verificaciones de igualdad: son todas la misma jugada, una prueba en tiempo de ejecución que el compilador mira y convierte en información de tipos. Vas a usar las tres en los bordes de la flota antes de que esta lección termine.

Y acá hay una recompensa que ya te ganaste sin darte cuenta. ¿Te acuerdas del `'okay'` forjado de la apertura, el typo que arrancó toda esta lección? Escribe el mismo typo contra la unión y mira qué pasa:

```ts
function isUp(result: ProbeResult): boolean {
  return result.kind === 'okay';
}
```

```
error TS2367: This comparison appears to be unintentional because the types
  '"http-error" | "ok" | "timeout"' and '"okay"' have no overlap.
```

Contra la forma de v0, `record.status === 'okay'` era una comparación perfectamente legal entre dos cadenas, y el compilador no tenía nada que decir. No podía. `string === string` es siempre una pregunta razonable. Contra la unión, el tipo del discriminante son tres literales específicos, así que compararlo con `'okay'` es demostrablemente siempre falso, y el compilador marca la comparación misma como un bug. La clase de typo no se volvió más difícil de escribir. Se volvió imposible de compilar. Ese antes y después vale la pena repetirlo en tu cabeza, porque es la demostración más clara que hay de lo que compraste: la verificación que antes tenías que hacer a ojo en el code review, ahora una máquina la hace con cada tecla.

![Cada verificación del discriminante pela una variante de la unión hasta que la rama final se conoce solo por eliminación.](assets/v04-diagram.webp)

### La exhaustividad es una feature que eliges activar

El narrowing te da acceso seguro. Hay una segunda garantía disponible, y tienes que ir a buscarla a propósito: la garantía de que manejaste cada variante. Acá es donde el patrón pasa de ordenado a genuinamente estructural, y el precio de entrada es una función de cuatro líneas:

```ts
function assertNever(value: never): never {
  throw new Error(`Unhandled variant: ${JSON.stringify(value)}`);
}
```

Nada mágico. Una función común y corriente cuyo tipo de parámetro es `never`, el tipo sin valores. Cualquier función que tome `never` se comportaría igual. La magia está enteramente en el narrowing: haz switch sobre el discriminante, maneja cada variante, y en el brazo `default` el compilador eliminó todo, así que el tipo que queda es `never`, y la llamada pasa la verificación de tipos. Sáltate una variante, o agrega una nueva después, y lo que queda ya no es `never`. La llamada deja de compilar, y el error nombra la variante exacta que no manejaste, en la línea exacta.

![Un switch anotado cuyo brazo default compila mientras todas las variantes están manejadas y falla nombrando la variante cuando aparece una nueva.](assets/v05-annotated-code.webp)

Vuelve a leer esa consecuencia, porque invierte cómo se siente refactorizar. Agregar una variante `'dns-error'` a un codebase lleno de estos switches no crea una cacería por cada lugar que necesita actualizarse. Crea una checklist de errores de compilación: cada sitio sin manejar falla, por nombre, hasta que cada uno decide qué quiere decir dns-error para él. El compilador escribe la hoja de trabajo del refactor por ti. Con honestidad, de todo lo que hay en esta lección, esta es, más que ninguna otra, la parte que ojalá alguien me hubiera mostrado antes, y tampoco es sabiduría solo de TypeScript. Esta jugada exacta vuelve en el módulo 4 vistiendo una bandera de Rust, donde `match` es exhaustivo por defecto y el compilador tiene la pluma desde el principio. Apréndela acá, recógela otra vez allá.

Una advertencia, y es la trampa más filosa de la lección. Un brazo `default:` que haga cualquier cosa que no sea `assertNever`, digamos `default: return 'down'`, silencia al compilador y vende sin hacer ruido la garantía que acabas de comprar. Las variantes futuras pasan de largo por ahí, sin clasificar, para siempre, y tsc no dice nada, porque le dijiste que no. Un default atrapa-todo es la mentira cortés con firma de tipo. Ve a buscar uno solo en una frontera donde de verdad no te importe, y ten claro qué estás vendiendo cuando lo hagas.

### unknown en vez de any en la frontera

Hay una disciplina más para instalar antes del lab, y vive en los bordes de tu programa, donde llegan los archivos JSON, las respuestas de RPC y los datos de otras personas.

TypeScript te da dos tipos para "no sé qué es esto", y son opuestos vestidos con nombres parecidos. `any` es el compilador rindiéndose: cada operación sobre un `any` está permitida, sin verificar, y cada valor que toca hereda el encogimiento de hombros, esparciéndose por tu grafo de llamadas como un solvente. `unknown` es el compilador exigiendo una demostración: ninguna operación está permitida hasta que lo estreches, con las herramientas exactas que acabas de aprender. Mismo runtime, los dos borrados a nada, defaults opuestos. Uno permite todo y no pide nada. El otro no permite nada hasta que hayas mostrado tu trabajo.

![Comparación lado a lado que muestra que any permite todo sin verificar mientras unknown bloquea todo uso hasta que se demuestra la forma del valor.](assets/v06-comparison.webp)

Esto le importa a la flota ahora mismo porque `JSON.parse` te devuelve datos sobre los que no demostraste nada, y su tipo de retorno debería tratarse como `unknown` en cada frontera de la que eres dueño. Decláralo `any` para ir rápido y un solo sitio de fetch infecta a cada consumidor de abajo con acceso sin verificar. Decláralo `unknown` y el compilador te obliga a la disciplina de parsear en la frontera que vas a construir dentro de `parseProbe` en el lab: demuestra la forma una vez, en el borde, y todo lo de adentro opera sobre tipos honestos. Esa disciplina es exactamente lo que va a necesitar el archivo de config que la flota hace crecer la próxima lección, y cada respuesta de RPC después de él, y es precisamente donde la próxima lección arranca.

Usémoslo en serio una vez, para que sea un hábito y no un slogan. Supón que llega un registro crudo de un archivo, forma desconocida, confianza cero. Acá está la guarda de frontera, construida enteramente con las jugadas de narrowing que ya tienes:

```ts
function readRecord(raw: unknown): ProbeResult | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const kind = (raw as Record<string, unknown>)['kind'];
  const value = (raw as Record<string, unknown>)['value'];
  if (typeof kind !== 'string' || typeof value !== 'number') return null;
  return parseProbe(kind, value);
}
```

Rastrea las demostraciones a medida que se acumulan. La verificación `typeof raw !== 'object'` más la verificación de igualdad `raw === null` juntas demuestran que tenemos un objeto de verdad en la mano antes de tocarlo, y ojo que las dos son necesarias: `typeof null` es `'object'`, una verruga de JavaScript de veinte años que la verificación de igualdad parcha. Después cada campo se saca como `unknown` y se interroga con `typeof` hasta que confiesa ser un `string` o un `number`. Solo entonces, con cada afirmación demostrada, el valor se gana el derecho de entrar a `parseProbe`. Intenta saltarte cualquier verificación y el compilador te detiene en la línea siguiente, porque estás operando sobre un valor cuya forma todavía no demostraste. Esa demanda constante de recibos es molesta por como un día. Después llega algún registro malformado a las tres de la mañana, rebota en esta función como un `null`, y dejas de notar la molestia para siempre.

Sí, esto es verboso. Cinco líneas de interrogatorio para dos campos, y un objeto de config de verdad tiene veinte. Siente esa fricción y recuérdala, porque es el dolor exacto que hace que la herramienta de la próxima lección aterrice: una librería de schemas escribe toda esta función a partir de una declaración, y el tipo sale de regalo. Tienes permiso de estar molesto. La molestia es el currículum.

Lo que nos trae al límite honesto, y merece su propio párrafo en vez de una nota al pie. Los tipos de TypeScript se borran en tiempo de ejecución. La unión demuestra teoremas sobre los valores que construye tu propio código, pero no demuestra nada, nada en absoluto, sobre los bytes que llegan de un archivo o de la red. Declarar `const data: ProbeResult = JSON.parse(raw)` no es una demostración, es un disfraz. Las garantías del sistema de tipos empiezan solo después de que una verificación real en tiempo de ejecución se las ha ganado, y por eso `parseProbe` devuelve `ProbeResult | null` en vez de hacer una aserción, y por eso la versión sistemática de esa idea, schemas que generan tanto la verificación en tiempo de ejecución como el tipo desde una sola fuente, es todo el tema de la próxima lección.

![Una línea de frontera separa los tipos borrados en tiempo de compilación de los bytes sin tipo en tiempo de ejecución, con parseProbe como la única barrera que convierte uno en el otro.](assets/v07-diagram.webp)

### El trade-off

Cada lección de este curso nombra el costo, así que acá está. Modelar con uniones es ceremonia que pagas por adelantado: más declaraciones de tipos que la forma v0 de una línea, un paso de parseo en cada frontera, y una variante nueva rompe cada switch del codebase hasta que cada sitio decide qué hacer con ella. En un domingo de hackathon, ese ruido es fricción real, y `{ status: string }` de verdad te va a llevar más rápido a la demo. La cuenta llega después, exactamente en el momento en que menos te la puedes permitir, y en el dominio de este curso la cuenta no viene con política de devolución. Ruidoso para la velocidad, magnífico para la corrección: ahora sabes de qué lado de ese trade-off se para este curso y, más útil todavía, sabes cómo elegir por proyecto en vez de por costumbre.

**Profundiza (el 20%).** esta lección te enseñó por qué existen las uniones y los patrones que vas a usar a diario: el discriminante, el switch exhaustivo, `unknown` en los bordes. La taxonomía completa del narrowing, los type guards, el operador `in`, las funciones de aserción, vive en el capítulo Narrowing del TypeScript Handbook, y su sección discriminated-unions es el tratamiento canónico del patrón de hoy: [https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions). Guárdalo como bookmark, léelo esta semana, y cuando termine este módulo, el repo type-challenges es el patio de ejercicios donde estos músculos se construyen de verdad. El lab de abajo no necesita nada del material del bookmark.

## Lab: borra la categoría

Así se sientan las rueditas este módulo: voy a trabajar los pasos 1 al 8 contigo, con cada archivo mostrado y explicado. El paso 9 lo completas con los errores del compilador como única guía, sin walkthrough en prosa. El challenge de después es todo tuyo. Ese repliegue es deliberado, y se vuelve más empinado el próximo módulo.

Todo esto pasa en tu repo de pulse del módulo 1, en `probe.ts`, el mismo archivo de la sonda que vienes haciendo crecer desde m01-l2. Una nota de orden antes de empezar: ese archivo ya declara el registro `ProbeResult` de v0 (la forma `{ url, status, latencyMs }`, con un timestamp viajando de acompañante en la copia de la flota). La unión del paso 2 se queda con el nombre, así que borra la declaración vieja cuando agregues la nueva, dos tipos con un nombre es un error de compilación, y esta vez el error tendría razón.

Una segunda nota de orden, sobre el archivo que NO estás editando. `fleet.ts`, el escritor del cron de m01-l3, declara su propio tipo de resultado con forma v0 aparte (ahí es donde viven `url` y `checkedAt`) y no importa nada de `probe.ts`, así que nada de lo que hagas hoy lo toca: sigue compilando, la barrera de typecheck en el CI de m01-l3 sigue en verde, y `status.json` mantiene su contrato congelado `{ url, status, latencyMs, checkedAt }` hasta que m03-l2 recablee el escritor, justo antes de que el panel le ponga un schema al archivo. Sí, eso quiere decir que el escritor de la flota todavía habla el dialecto mentiroso de v0 después de hoy. A propósito: esta lección borra la categoría dentro de `probe.ts`; el lado de la flota se reconstruye sobre schemas en las próximas dos lecciones.

1. **Reproduce la mentira primero.** Si te salteaste el registro forjado de la apertura, hazlo ahora: agrega `const forged: ProbeRecord = { status: 'okay' };` y la verificación `!== 'timeout'`, después corre estas dos:

```bash
npx tsx probe.ts     # prints: healthy
                     # then: usage: npx tsx probe.ts <url>, exit 1
npx tsc --noEmit     # exits clean. green.
```

   El error de uso y el exit 1 son la guarda sin-argumentos de v0 haciendo su trabajo normal; la línea forjada imprime antes de que la guarda se dispare, y ese primer `healthy` es la mentira que nos importa. Míralo imprimir, y confirma que el compilador también está en verde. No tiene ninguna objeción, porque el tipo que le diste genuinamente permite esto. Ese check verde es tu foto de antes.

2. **Modela la unión.** Arriba del archivo, borra el registro `ProbeResult` viejo de v0 y agrega en su lugar la nueva capa de tipos de la flota. Borra también las líneas forjadas del paso 1, las tres (el tipo `ProbeRecord`, la const `forged` y su `console.log`); eran la foto de antes, y dejadas ahí imprimirían un `healthy` perdido antes de cada ejecución de la sonda para siempre. (tsc se pone en rojo en el momento en que haces estas ediciones, porque la sonda de v0 todavía devuelve la forma vieja; los pasos 5 al 7 llevan cada uno de esos errores de vuelta al verde, que es exactamente el workflow que esta lección está vendiendo.)

```ts
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };

type Verdict = 'up' | 'degraded' | 'down';

function assertNever(value: never): never {
  throw new Error(`Unhandled variant: ${JSON.stringify(value)}`);
}
```

   Ojo que `Verdict` es en sí misma una pequeña unión de literales. El mismo truco que arregló el tipo de resultado también evita que `'degarded'` salga jamás de tu clasificador.

3. **Pon la forma vieja y la nueva en una sola pantalla.** Este lado a lado es toda la lección en dos declaraciones, así que míralas de verdad juntas antes de borrar nada:

```ts
// v0: one shape, many lies
type ProbeRecord = { status: string; latencyMs?: number };

// typed fleet: three shapes, no spare states
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };
```

   El diff es la lección. El campo con cadenas se convirtió en tres discriminantes literales. El campo opcional se convirtió en un campo obligatorio que existe solo donde es verdad. Todo lo que v0 podía escribir mal, la unión ni lo puede escribir.

4. **Parsea en la frontera.** Los pares `(kind, value)` no confiables vienen de afuera; esta función es el checkpoint de frontera que los convierte en demostración o los rechaza:

```ts
function parseProbe(kind: string, value: number): ProbeResult | null {
  switch (kind) {
    case 'ok':
      return { kind: 'ok', latencyMs: value };
    case 'timeout':
      return { kind: 'timeout', budgetMs: value };
    case 'http-error':
      return { kind: 'http-error', status: value };
    default:
      return null;
  }
}
```

   Ojo lo que el tipo de retorno dice en voz alta: `ProbeResult | null`. Parsear puede fallar, así que el tipo lo admite, y el compilador obliga a cada llamador a manejar el `null` antes de tocar el resultado. El target malformado de la apertura muere justo acá, en la frontera, como un `null` que tienes que manejar a los gritos, en vez de en el fondo de un panel como un mosaico verde.

![Los registros no confiables pasan por un único checkpoint de parseo donde las entradas forjadas quedan fuera como null y solo los resultados demostrados siguen hacia adentro.](assets/v08-flowchart.webp)

5. **Reescribe el clasificador de forma exhaustiva.** Reemplaza con esto cualquier lógica de v0 que decidiera la salud:

```ts
function classifyProbe(result: ProbeResult): Verdict {
  switch (result.kind) {
    case 'ok':
      if (result.latencyMs < 400) return 'up';
      if (result.latencyMs <= 1000) return 'degraded';
      return 'down';
    case 'timeout':
      return 'down';
    case 'http-error':
      return result.status === 429 ? 'degraded' : 'down';
    default:
      return assertNever(result);
  }
}
```

   Dos decisiones de criterio acá adentro se ganan su por qué. Las bandas: por debajo de 400ms es healthy, de 400 a 1000 inclusive es degraded, y solo estrictamente por encima de 1000 es down, porque una respuesta lenta sigue siendo una respuesta. Y 429: una respuesta de límite de tasa quiere decir que el target está vivo y hablando, nada más que cansado de ti, así que es `'degraded'`, no `'down'`. El resto del switch es plomería, y fíjate qué poco código defensivo tiene. Dentro de cada brazo, los campos simplemente existen. El narrowing ya los demostró.

6. **Conecta la sonda misma a la unión.** La función de sonda ahora devuelve el tipo honesto de punta a punta:

```ts
async function probe(url: string, budgetMs = 5000): Promise<ProbeResult> {
  const started = performance.now();
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(budgetMs) });
    const latencyMs = performance.now() - started;
    if (!res.ok) {
      return { kind: 'http-error', status: res.status };
    }
    return { kind: 'ok', latencyMs };
  } catch {
    return { kind: 'timeout', budgetMs };
  }
}
```

   Intenta, nada más como experimento, devolver el estado forjado desde esta función: `return { kind: 'ok' }` sin latencia. El compilador se niega antes de que puedas guardar el archivo. Ese es el antes y después de toda esta lección comprimido en un solo subrayado rojo: la mentira que viste imprimir `healthy` en el paso 1 ahora no puede salir de la función que la habría contado.

7. **Reescribe el driver, el último bastión de v0.** Corre `npx tsc --noEmit` ahora y los errores que quedan, cinco, apuntan todos al fondo del archivo: el loop del driver de m01-l2 todavía junta `results`, ordena por `latencyMs`, e imprime `result.url`/`result.status`/`result.latencyMs.toFixed(1)`, y ninguno de esos existe en todos los brazos de la unión (y `url` en ninguno). Ese es el compilador diciéndote que el formato de salida de la CLI se diseñó para la forma vieja, así que el driver se rediseña, no se parcha. Primero, si todavía no lo hiciste, agrega al archivo la función `describe` de la sección de teoría, textualmente de la sección de narrowing; está a punto de convertirse en el formateador de detalle de la CLI. Después mantén las líneas de `targets`/guarda de uso y reemplaza todo lo que está debajo con:

```ts
for (const target of targets) {
  const result = await probe(target);
  console.log(`${target} ${classifyProbe(result)} (${describe(result)})`);
}
```

   El array `results`, el ordenamiento y la vieja línea de log se van todos. La nueva salida es la línea con el veredicto primero que la flota quiere de verdad, con la evidencia entre paréntesis:

```bash
npx tsx probe.ts https://www.rust-lang.org
# https://www.rust-lang.org up (88.7ms)
```

   Tu latencia va a ser distinta; la forma no. Fíjate qué sacó el rediseño: ordenar por latencia tenía sentido cuando cada registro tenía un `latencyMs`, y con la unión honesta ya no, porque un timeout no tiene latencia por la que ordenar. El tipo no solo encontró el bug, retiró la feature en la que el bug vivía.

8. **Verifica, después extrae la demostración negativa.** Primero el check positivo: `npx tsc --noEmit` debería salir limpio. Verde. Pero verde-cuando-está-correcto es solo la mitad de lo que compraste, así que ahora demuestra que la garantía es real rompiéndola a propósito. Comenta el brazo `case 'http-error':` entero en `classifyProbe` y corre `npx tsc --noEmit` otra vez:

```
probe.ts: error TS2345: Argument of type '{ kind: "http-error"; status: number; }'
  is not assignable to parameter of type 'never'.
```

   Mira lo que hizo. No dijo "algo está mal en algún lado". Nombró la variante que faltaba, en la línea de `assertNever`, en la función que dejó de manejarla. El compilador está haciendo el review. Restaura el brazo, confirma que está limpio, y ese es tu checkpoint: ahora deberías poder producir los dos estados a demanda, verde cuando es exhaustivo, un error con nombre cuando no.

9. **El ejercicio de dns-error. Tu turno, el compilador como guía.** Ahora mismo, una falla de DNS, sondeando `https://definitely-not-a-real-host.example`, aterriza en el `catch` y queda registrada como un timeout, que es una pequeña mentira propia: el host no dio timeout, no existe. Agrega una cuarta variante, `{ kind: 'dns-error'; host: string }`, a `ProbeResult`, después corre `npx tsc --noEmit` y arregla nada más que lo que el compilador nombra, un error a la vez, hasta que esté en verde. Sin walkthrough para este, y no hace falta: cuenta con que los errores te marchen a exactamente dos sitios, el `assertNever` del clasificador y la salida por descarte de la función `describe` (se apoyaba en que `'timeout'` fuera lo único que quedaba, y el paso 7 hizo a `describe` parte de la CLI). Cuando tsc esté en verde otra vez, cada switch de tu flota decidió conscientemente qué quiere decir una falla de DNS. Esa hoja de trabajo que acabas de seguir la escribió el compilador, y es la experiencia exacta de mantener código tipado en un equipo de verdad.

![Agregar una variante irradia errores del compilador con nombre hacia cada sitio sin manejar hasta que cada uno se arregla y el build se pone en verde.](assets/v09-diagram.webp)

## Challenge: parsea una vez, clasifica exhaustivamente

Ahora la rep sin guía. El challenge classify-probe-result vive en el coding-challenge panel interactivo de la página de esta lección, igual que el `latencyStats` de m01-l2: starter, grader y hints, todo en el editor del navegador, nada que descargar. El starter que te da es puro pensamiento v0: solo se consideran las sondas `'ok'`, la banda degraded no existe, y todo lo demás se amontona en `'down'`. Reconstrúyelo como acabas de reconstruir la flota. Modela `ProbeResult` como una unión discriminada, parsea el par `(kind, value)` entrante una vez en la frontera (los kinds desconocidos parsean a `null`, exactamente como `parseProbe` en el paso 4), después clasifica con un switch exhaustivo cerrado por `assertNever`. Para ser preciso con `'invalid'`, dado que no es un cuarto veredicto: mantén `Verdict` como la unión de tres miembros del paso 2, y haz que la función que ve el grader devuelva `Verdict | 'invalid'`, donde `'invalid'` es lo que responde cuando el parseo volvió `null`. La falla de parseo y la clasificación siguen siendo dos hechos distintos, y el tipo de retorno lo dice. Ojo con los bordes que le importan al grader: 400 y 1000 aterrizan los dos en `'degraded'`, un 429 quiere decir que el target respondió así que también es `'degraded'`, y las ocho pruebas incluyen los valores de borde y el caso de kind desconocido. Todo lo que necesitas está arriba; nada de lo que guardaste como bookmark es necesario. Si quieres el flex extra después, borra un brazo y predice el error antes de correr tsc.

## Checkpoint

Antes de cerrar la pestaña, la recuperación de 30 segundos, en voz alta o en una nota, sin espiar: ¿qué demuestra `assertNever`, y cuándo se dispara? Estás buscando algo como: si cada variante está manejada, el valor del brazo default se estrecha a `never`, así que la llamada compila; una variante nueva lo vuelve no-never y la compilación falla justo ahí. Si esa frase salió limpia, el mecanismo es tuyo. Si no salió, relee el paso 8, corre la demostración negativa una vez más, y va a salir.

Y cuéntame cómo te fue con el ejercicio, sin adornos: ¿los errores del compilador de verdad te caminaron hasta cada sitio en el paso 9, o encontraste un hueco donde la hoja de trabajo se perdió algo? Ese feedback le da forma a qué tan fuerte se apoyan los próximos módulos en esta jugada. Donde te trabes, dilo en el thread de la comunidad de esta lección; un punto de traba nombrado temprano salva a cinco estudiantes detrás de ti.

Tus resultados ahora están tipados. Pero la flota está a punto de hacer crecer un archivo de config, JSON en disco que llega como datos no confiables y sin tipo, y lo mismo hace cada respuesta de RPC que vayas a traer en tu vida, y ahora sabes exactamente por qué una unión no puede ayudar ahí: los tipos se borran, y un disfraz no es una demostración. Una unión no puede demostrar nada sobre bytes que no parseaste. La próxima lección: zod en la frontera, donde el schema es a la vez la verificación en tiempo de ejecución y la única fuente del tipo, y `parseProbe` crece hasta convertirse en algo que puede vigilar toda la frontera. La guarda de frontera está a punto de recibir un schema.
