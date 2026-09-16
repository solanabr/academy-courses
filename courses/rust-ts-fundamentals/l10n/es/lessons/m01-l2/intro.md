# Node 24, TypeScript 7 y tu primera sonda

La lección pasada corriste dos sondas sin instalar una sola cosa: un one-liner con `fetch` en la consola de devtools contra rust-lang.org, y un snippet del Playground donde el compilador atrapó un bug antes de que el código llegara a correr. Las dos funcionaron. Las dos también desaparecieron. Cierra la pestaña y ninguna de las dos sondas existió nunca: sin archivo, sin historial, sin forma de correrla otra vez mañana y comparar números.

Ese es el problema con los entornos prestados. La medición de verdad necesita una casa: un runtime que es tuyo, un compilador que atrapa el bug ANTES de que la sonda te mienta, y un archivo en un directorio al que le puedes hacer commit. De aquí a diez minutos esa casa existe, y `pulse` v0 imprime su primera latencia.

Así que revisemos sobre qué estás parado. Abre una terminal y corre:

```bash
node --version
```

Si eso imprime `v24.x.x`, ya estás en casa. Si no imprime nada, o algo más viejo, la siguiente sección lo arregla en dos minutos. De cualquier forma, deja la terminal abierta — todo lo que viene pasa ahí.

Y en cuanto corra la sonda, vamos a hablar de algo genuinamente raro: el compilador que estás por instalar fue reescrito él mismo en otro lenguaje para ganar 10x de velocidad. Toda la tesis de este curso, TypeScript para las superficies y un lenguaje de sistemas para los hot paths, está corriendo en tu propia máquina antes de que hayas escrito cincuenta líneas.

## Resumen

- Instalas Node 24 LTS y TypeScript 7.0.2, y entregas `pulse` v0: una sonda que busca una URL e imprime su latencia, corrida con tsx, dentro de los primeros diez minutos.
- Recorres las ~5 flags estrictas de tsconfig con las que el código de este curso de verdad va a chocar, cada una con el error exacto del compilador que lanza, y guardas el resto como bookmark.
- Recibes la historia sin adornos de TypeScript 7: qué compró el compilador nativo, qué cuesta hoy, y por qué los repos de producción todavía fijan 5.x.
- El lab planta un bug real a propósito y deja que el compilador lo atrape. El challenge te manda a múltiples URL, solo.

## El toolchain de diez minutos

### Node 24 LTS, no "el más nuevo"

¿Por qué 24 y no el número que sea más grande en la página de descargas? Porque Node se publica en dos vías. Las majors de número par se promueven a LTS, soporte de largo plazo: reciben arreglos por años y son lo que corren de verdad los servidores de producción. Las majors de número impar son experimentos con vida útil corta, un lugar para que el proyecto pruebe cambios antes de que una línea LTS los herede. "El más nuevo" es una beta rodante; LTS es el suelo sobre el que construyes.

Esto no es trivia de Node, es un hábito que estás formando para cada runtime y cada toolchain de este curso. Cuando algo que despliegas se rompe a las 3 de la mañana, quieres estar en la línea que recibe parches de seguridad por años, contra la que prueban las plataformas de nube, que cada paquete de tu árbol de dependencias dice soportar. Esa línea tiene un nombre y un calendario publicado, y revisar el calendario antes de instalar es un acto de treinta segundos que ahorra dolor real. "Maintenance" en ese calendario, por cierto, no es la muerte: un LTS en mantenimiento todavía recibe arreglos críticos, solo deja de recibir features nuevas, que para un servidor suele ser exactamente lo que quieres.

Ahora mismo la línea Active LTS es Node 24, nombre en clave "Krypton". Una nota al pie con fecha, porque este relevo está programado: Node 26 se vuelve el nuevo Active LTS el 2026-10-28, con Node 24 deslizándose a mantenimiento una semana antes, el 2026-10-20, según el cronograma de lanzamientos publicado (todavía seguro, todavía parchado hasta el 2028-04-30, solo que ya no es el titular). Al 2026-09-02, Node 24 LTS es la instalación correcta, y todo en este curso corre sin cambios en 26 cuando actualices después.

![Una línea de tiempo donde Node 24 mantiene Active LTS hasta hoy, entra en mantenimiento el 20 de octubre de 2026, y le pasa el título de Active LTS a Node 26 el 28 de octubre de 2026.](assets/v01-timeline.webp)

Instálalo desde nodejs.org (elige el botón LTS, apunta a 24) o, si ya usas un gestor de versiones como nvm, `nvm install 24`. Después verifica:

```bash
node --version
# v24.x.x
```

Ese único binario trae más que un motor de JavaScript. `fetch` viene integrado. `performance.now()` viene integrado. Incluso hay una flag `node --env-file` para cargar variables de entorno de forma nativa, sin paquete; recuerda ese nombre, se gana su propio momento cuando lleguemos a la higiene de secretos más adelante en el curso. La sonda que estás por escribir usa cero dependencias para su trabajo real. Todo lo que instalas después es para los *tipos*, no para el runtime.

### TypeScript 7.0.2 y tsx

Convierte la sonda en un proyecto de verdad:

```bash
mkdir pulse-station && cd pulse-station
npm init -y
npm pkg set type=module
```

Tres comandos, una decisión que vale nombrar. El nombre del directorio es `pulse-station`, no `pulse`, porque es la estación entera del diagrama de m01-l1, no solo la sonda de hoy: el repo de GitHub de la próxima lección toma este nombre, `npm init -y` lo estampa en `package.json`, y los dos vuelven después (el panel busca `https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json`, y m03-l1 le pasa el nombre a la raíz del workspace). `npm init -y` escribe un `package.json`, el archivo que hace de este directorio un proyecto que npm entiende. `npm pkg set type=module` declara que los archivos de acá son módulos ES, el sabor moderno de import/export. Por qué eso importa recibe un párrafo después; por ahora es una casilla que marcamos para que el `await` de nivel superior funcione.

Ahora el toolchain:

```bash
npm i -D typescript@7.0.2 tsx@4.23.13 @types/node@24
```

Versiones verificadas contra el registro de npm el 2026-09-02; `latest` para typescript resuelve exactamente a 7.0.2 hoy. Una nota cosmética antes de la trampa: npm 11 puede imprimir algunas líneas `npm warn install-scripts esbuild...` durante esta instalación. Eso es la barrera de npm sobre los scripts de instalación hablando de más sobre una dependencia que eligió no correr, no una falla; si el comando sale sin una línea `npm error`, el toolchain aterrizó bien. Y ese dígito merece su propia caja de advertencia, porque es una trampa genuina: **no hay un typescript 7.0.0 estable en el registro.** GA llegó con arreglos de parche ya incorporados, así que el 7.x estable empieza, y por ahora termina, en 7.0.2. Un script de setup que dice `typescript@7.0.0` falla todas y cada una de las veces que corre. La prosa puede decir "TypeScript 7.0"; las líneas de instalación tienen que decir 7.0.2.

Tres paquetes, tres trabajos:

- **typescript** te da `tsc`, el verificador. Lee tu código, aplica las reglas de tipos y te dice qué está mal. En este curso lo corremos como `tsc --noEmit`: verifica todo, emite nada.
- **tsx** es el runner. Ejecuta un archivo `.ts` directamente, sin etapa de compilación que tengas que ver. Herramienta de dev-loop, y la forma en que `pulse` corre todo el curso.
- **@types/node** le enseña al verificador cómo se ven los globals propios de Node, así `process.argv` tiene un tipo en vez de ser un misterio.

La división es lo que hay que internalizar: tsx corre tu código y no le importan tus tipos; tsc verifica tus tipos y nunca corre tu código. Necesitas los dos, y confundirlos está detrás de la mitad de la confusión de "¡pero corrió bien!" en los equipos de TypeScript. Un archivo puede correr perfecto bajo tsx mientras carga un error de tipos que va a morder a la próxima persona que llame a tu función de otra forma, y por eso el lab te hace correr los dos, cada vez, hasta que el par sea memoria muscular.

Una cosa más, pequeña, porque la vas a escribir todo el tiempo: el prefijo `npx` corre un binario del `node_modules` propio de tu proyecto en vez de andar cazando una instalación global. `npx tsc` es *tu* 7.0.2 fijado, no lo que sea que otro proyecto dejó en tu máquina. Pins por proyecto, invocados por proyecto: esa disciplina es la razón por la que dos proyectos con versiones distintas de TypeScript pueden coexistir en paz en una sola laptop.

![Un archivo fuente fluye por tsx hasta un programa corriendo y por separado por tsc hasta errores de tipos, con las definiciones de tipos de Node alimentando solo al verificador.](assets/v02-diagram.webp)

### La primera latencia en un solo archivo

Hora de la recompensa. Crea `smoke.ts`:

```typescript
const started = performance.now();
const res = await fetch("https://www.rust-lang.org");
const elapsed = performance.now() - started;
console.log(`${res.status} in ${elapsed.toFixed(1)}ms`);
```

Cuatro líneas. `performance.now()` da un timestamp de milisegundos de alta resolución; llámalo antes y después del fetch y la diferencia es tu latencia. ¿Por qué no `Date.now()`, que quizás conozcas de JavaScript en otra parte? Porque `Date.now()` lee el reloj de pared, y los relojes de pared se ajustan: tu sistema operativo sincroniza la hora en segundo plano, y un reloj que salta a mitad de medición te puede dar una latencia negativa. `performance.now()` es monotónico, solo avanza, que es la propiedad que una herramienta de medición de verdad necesita. Decisión pequeña, pero `pulse` es una herramienta de medición para el resto del curso, así que arranca con el reloj correcto. Córrelo:

```bash
npx tsx smoke.ts
# 200 in 254.5ms
```

Ese número es de mi ejecución mientras redactaba esto, y acá hay un detalle que merece tu atención: mi primerísima ejecución imprimió 498.4ms, la segunda 254.5ms. Misma URL, segundos de diferencia, la mitad de la latencia. Caché de DNS, reuso de conexión, humor de la red. Una muestra es ruido. Guarda ese pensamiento, porque convertir ruido en señal es exactamente hacia dónde van las herramientas de este curso, y también es el coding challenge de esta lección.

Acabas de hacer todo lo que hizo la consola de devtools ayer, salvo que es un archivo, en un proyecto, en tu máquina, y va a correr idéntico mañana. Ahora hagamos que el compilador se gane su asiento.

### Las cinco flags que de verdad vas a hacer saltar

Corre el generador de config:

```bash
npx tsc --init
```

Esto escribe `tsconfig.json`, el libro de reglas del verificador. Y quiero ser directo sobre el método acá, porque yo mismo hice la versión equivocada de esto: copiar una config en modo estricto sacada de un post de blog de 2023 en vez de leer lo que emite el `tsc --init` actual. Culpable, más de una vez. Las configs desactualizadas desactivan en silencio las flags exactas de las que depende este curso. La salida de init ES el canon actual; la genera el mismo equipo que entrega el compilador, y cambia a medida que cambia el lenguaje. Lee la tuya, no la de un blog.

Dos ediciones pequeñas antes del recorrido, las dos sugeridas por comentarios dentro del archivo generado mismo: pon `"types": ["node"]` (el valor por defecto de init es una lista vacía, que le esconde `process` al verificador) y descomenta `"lib": ["esnext"]`. Eso es todo. Cero config inventada en este curso; todo lo demás queda exactamente como lo escribió init.

Ahora, el archivo prende un montón de cosas. No vas a hacer saltar la mayoría. Estas son las cinco flags con las que el código propio de este curso de verdad va a chocar, cada una con su choque:

**1. `strict`** es el paraguas: prende una familia de verificaciones, sobre todo "null y undefined son tipos reales que tienes que manejar". Sin ella, esto compila y explota en tiempo de ejecución:

```typescript
function firstChar(s: string | null): string {
  return s.charAt(0); // strict says: s might be null. Handle it.
}
```

Bajo `strict`, eso es un error hasta que verifiques `s` primero. Toda lección de acá en adelante asume que está prendida.

**2. `noUncheckedIndexedAccess`** hace que la indexación diga la verdad. `process.argv[2]` no tiene ninguna garantía de existir; el usuario podría correr tu sonda sin ninguna URL. Así que bajo esta flag su tipo es `string | undefined`, no `string`, y pasarlo directo a una función que quiere un `string` es un error:

```text
error TS2345: Argument of type 'string | undefined' is not
assignable to parameter of type 'string'.
```

Ese es un error real del compilador, y lo vas a hacer saltar a propósito en el lab. La clase de bug que borra: un batch vacío, un argumento que falta, un índice corrido en uno, cada uno un crash en tiempo de ejecución que ahora no se puede escribir.

**3. `exactOptionalPropertyTypes`** gobierna una mentira sutil. Un campo opcional como `label?: string` quiere decir "puede estar ausente". Escribir `undefined` en él no es ausencia; es presencia con un hueco adentro, y el código que itera claves o serializa a JSON trata a los dos de forma distinta. La sonda va a crecer un objeto de opciones bastante pronto, así que acá está el choque en miniatura:

```typescript
type ProbeOptions = { label?: string };
const opts: ProbeOptions = { label: undefined };
```

```text
error TS2375: Type '{ label: undefined; }' is not assignable to
type 'ProbeOptions' with 'exactOptionalPropertyTypes: true'.
```

Si un campo es opcional, omítelo. Si de verdad puede contener undefined, dilo en el tipo. El bug que esto borra es silencioso y feo: `JSON.stringify` descarta los campos ausentes pero un spread copia los que son `undefined`, así que los dos objetos "vacíos" se comportan distinto en el momento en que cruzan una frontera.

**4. `verbatimModuleSyntax`** mantiene honestos a los tipos y a los valores en la línea de import. Cuando `pulse` crezca un `types.ts` en una lección posterior, este import parece inocente:

```typescript
import { ProbeResult } from "./types.js";
```

```text
error TS1484: 'ProbeResult' is a type and must be imported using
a type-only import when 'verbatimModuleSyntax' is enabled.
```

El arreglo es `import type { ProbeResult }`. ¿Por qué te debería importar? Los tipos desaparecen en tiempo de ejecución. Un import que solo carga un tipo tiene que ser borrable, y esta flag garantiza que la salida compilada nunca lleve un import fantasma que se rompa en tiempo de ejecución.

**5. `module: "nodenext"`** alinea al verificador con cómo Node resuelve módulos de verdad. Su tropiezo más común: los imports relativos necesitan el nombre de archivo completo, extensión incluida, y la extensión es `.js` incluso en un archivo `.ts`, porque eso es lo que existe después de la compilación:

```typescript
import { probe } from "./probe";
```

```text
error TS2835: Relative import paths need explicit file extensions
in ECMAScript imports when '--moduleResolution' is 'node16' or
'nodenext'. Did you mean './probe.js'?
```

El compilador incluso sugiere el arreglo. Tómalo. Esta se siente pedante exactamente una vez, y después notas que tus imports ahora quieren decir lo mismo para el verificador, para Node y para cada bundler más abajo, y toda la categoría de "funciona en dev, se rompe en la resolución de prod" desaparece.

Cada uno de esos mensajes de error es de mi terminal, no parafraseado. Ese es el estándar que mantiene este curso: cuando una lección dice "el compilador atrapa esto", recibes el texto de error real, y puedes reproducirlo.

![Una tabla que empareja cada una de las cinco flags estrictas con lo que impone y el bug exacto de la sonda que la hace saltar.](assets/v03-comparison.webp)

¿Y el resto del archivo generado? Es real y vale conocerlo, y no vale una caminata pesada flag por flag. Acá está la salida restante de init en 7.0.2, una fila de referencia para cada una, para que sepas qué estás cargando:

| Ajuste | Una línea de por qué está ahí |
|---|---|
| `target: "esnext"` | emite y verifica contra el JavaScript actual; el runtime es moderno, actúa como tal |
| `isolatedModules` | cada archivo tiene que ser traducible solo, que es lo que requieren las herramientas rápidas por archivo |
| `moduleDetection: "force"` | trata cada archivo como un módulo, sin scripts globales accidentales |
| `noUncheckedSideEffectImports` | un `import "./x"` pelado tiene que apuntar a algo que existe |
| `jsx: "react-jsx"` | cómo compilar JSX si aparece algo; inerte acá e inerte para todo el curso, porque el panel del módulo tres llega desde create-vite cargando su propio tsconfig en vez de extender este |
| `skipLibCheck` | no vuelvas a verificar los tipos en los archivos de declaración de tus dependencias en cada ejecución |
| `sourceMap`, `declaration`, `declarationMap` | salidas para depuradores y consumidores de librerías; inerte hasta que emitas |

Ninguna de estas va a interrumpir tu semana como sí lo van a hacer las cinco de arriba. Cuando una sí te sorprenda, la referencia del Handbook lo explica mejor de lo que puede una oración acá, y esa es la división de trabajo correcta.

Acá está la síntesis, y es la franca: las flags son el code review que no puedes saltarte. Un revisor humano atrapa el bug del argumento que falta en un buen día, si no está cansado, si el diff no es enorme. Este revisor corre en milisegundos, en cada guardado, para siempre, y nunca se cansa. La estrictez es fricción que compras a propósito. Las cinco flags te van a interrumpir todo el curso, y cada interrupción es un bug que nunca se entregó.

### El compilador que se volvió diez veces más rápido

Una historia antes del lab, porque acabas de instalar su final.

En marzo de 2025 Microsoft anunció "A 10x Faster TypeScript": el compilador de TypeScript, escrito él mismo en TypeScript por más de una década, se estaba portando a Go. Dieciséis meses después, el 2026-07-08, ese port llegó a GA como TypeScript 7. El benchmark del titular: en VS Code el build completo de verificación de tipos bajó de 125.7s a 10.6s. El uso de memoria bajó entre 6% y 26% en los codebases probados. (Vas a ver "~18% menos memoria" citado en posts de segunda mano; ese punto medio no aparece en ninguna parte del anuncio de GA. La cifra real es el rango. Aprendí a desconfiar de los números sospechosamente redondos, y tú también deberías.)

![Un gráfico de barras que muestra la verificación de tipos completa de VS Code bajando de 125.7 segundos con TypeScript 6 a 10.6 segundos con TypeScript 7.](assets/v04-chart.webp)

Quédate un rato con lo que implica esa historia, porque es la tesis de este curso vestida con las release notes de otro. El equipo de TypeScript, la gente mejor posicionada del planeta para hacer rápido a TypeScript, concluyó que el hot path del compilador iba en un lenguaje de sistemas. No porque TypeScript sea malo; porque capas distintas tienen físicas distintas. TypeScript para las superficies donde los tipos te compran corrección, un lenguaje nativo para los caminos donde el layout de memoria te compra velocidad. Estás aprendiendo los dos lenguajes en este curso exactamente por esta razón.

Ahora la advertencia, y es una de verdad, una sola caja, sin enterrar nada:

> **Lo que cuesta TS 7 hoy.** TypeScript 7 se entregó SIN una API programática; el post de GA lo dice sin rodeos: "Esperamos que TypeScript 7.1 se entregue con una API nueva (y distinta)". Las herramientas que manejan el compilador de forma programática, typescript-eslint, plugins de lenguaje de frameworks, todavía no pueden sentarse en 7, así que los repos del mundo real siguen fijando 5.x. El ejemplo emblemático es uno en el que este curso se va a apoyar por semanas: @solana/kit, la librería cliente moderna de Solana, se compila con typescript ^5.9.3. Al 2026-09-02, 7.1 no se ha entregado; el tag `next` del registro es un build de desarrollo 7.1.0. Verificado en vivo; cuando llegue 7.1, este párrafo se reescribe.

No me creas sobre el estado de las cosas, pregúntale al registro tú mismo; este hábito de verificar versiones contra la fuente en vez de asumirlas es uno que el curso va a ejercitar:

```bash
npm view typescript dist-tags.latest dist-tags.next
# dist-tags.latest = '7.0.2'
# dist-tags.next = '7.1.0-dev.20260902.1'
```

¿Entonces instalar 7.0.2 es un error? No, y la distinción importa: para *tus* proyectos, donde corres `tsc` y `tsx` directamente, 7 es el TypeScript más rápido jamás entregado y está completamente listo. El atraso está en el ecosistema de herramientas *alrededor* del compilador. Los repos de producción fijan 5.x no por flojera sino porque la compatibilidad de herramientas es parte de lo que quiere decir una versión: una versión es una promesa sobre todo lo que se enchufa en ella, no solo sobre el binario mismo. La velocidad del compilador y la madurez de su ecosistema son, ahora mismo, un trade-off.

La regla de decisión, ya que la vas a enfrentar en tus propios proyectos pronto: repo greenfield donde controlas el toolchain y sobre todo necesitas `tsc` más un runner, toma 7 y disfruta la velocidad. Repo que se apoya en typescript-eslint, en un plugin de lenguaje de framework, o en cualquier otra cosa que maneje el compilador a través de su API, quédate en 5.x hasta que llegue 7.1 y las herramientas se pongan al día. Ninguna de las dos opciones está mal; son respuestas a preguntas distintas. Este curso toma el lado rápido, te dice dónde está la costura, y vas a reconocer el patrón cada vez que el producto emblemático de un ecosistema se entregue antes que sus herramientas otra vez, que en esta industria es como cada trimestre.

Un párrafo sobre sistemas de módulos, porque es todo lo que 2026 le debe al tema: por una década JavaScript tuvo dos sabores de módulo compitiendo, CommonJS (`require`) y los módulos ES (`import`), y el dolor de interoperar generó mil hilos de discusión furiosos. Esa guerra terminó. `require(esm)` es estable desde Node 24, `tsc --init` emite ajustes ESM-first, y tú pusiste `"type": "module"` hace diez minutos sin ceremonia. Escribe ESM-first, consume lo que necesites, y si un tutorial de 2022 te advierte sobre peligros de paquete dual, revisa su fecha y cierra la pestaña.

**Profundiza (el 20%).** esta lección recorrió las flags que vas a hacer saltar y se saltó el tour del lenguaje a propósito; los recursos canónicos lo hacen mejor. El TypeScript Handbook, https://www.typescriptlang.org/docs/handbook/intro.html, es la fuente oficial de la verdad y se lee en unas pocas noches. La introducción a TypeScript propia de Node, https://nodejs.org/learn/typescript/introduction, cubre la mirada del runtime sobre la misma historia. Guarda las dos como bookmark; este curso enlaza capítulos, nunca los vuelve a enseñar.

## Lab: entrega pulse v0

Tier completamente resuelto, y voy a decir en voz alta la parte que no se dice: esto es lo más que este curso te va a llevar de la mano. Cada comando está impreso, la sonda se construye paso a paso, y tus únicos blancos son dos TODOs. El próximo módulo recibes esqueletos; para los módulos tardíos, specs. El challenge de múltiples URL después del lab es tu primer pequeño paso solo. Ese repliegue es deliberado, y es así como te vuelves fuerte.

El contrato del artefacto, porque las lecciones siguientes te lo van a exigir: `pulse` v0 es un archivo de TypeScript donde `probe(url)` busca el target con el fetch integrado, lo cronometra con `performance.now()`, e imprime URL, estado HTTP y latencia en ms. Deliberadamente tipado con cadenas y de un solo target. Eso no es un cumplido: en la lección de tipos de TypeScript que viene, le vamos a dar a esta sonda un target malformado, verla mentir cortésmente, y reemplazar sus cadenas por una unión tipada. La versión 0 se supone que tiene espacio para crecer.

**1. Confirma el scaffold.** Deberías estar dentro de `pulse-station/` con `package.json` (que contiene `"type": "module"`), `tsconfig.json` (con tus dos ediciones), y `node_modules` de la instalación. Demuéstralo:

```bash
npx tsc --noEmit && echo ready
# ready
```

**2. Crea `probe.ts` con el esqueleto.** Dos TODOs, todo lo demás completo:

```typescript
// probe.ts - pulse v0
type ProbeResult = {
  url: string;
  status: number;
  latencyMs: number;
};

async function probe(url: string): Promise<ProbeResult> {
  // TODO 1: capture performance.now() into `started`,
  // await fetch(url) into `res`,
  // then compute latencyMs as the difference from a second performance.now()
  return { url, status: res.status, latencyMs };
}

const target = process.argv[2];

const result = await probe(target);
// TODO 2: print one line: the url, the status, and latencyMs
// with one decimal place, space-separated
```

Lee la forma antes de llenarla. `ProbeResult` es el registro que devuelve cada sonda: qué URL, qué estado HTTP, cuánto tardó. `process.argv` es el array de Node con los pedazos de la línea de comandos; el índice 0 es node mismo, el índice 1 el script, el índice 2 el primer argumento que de verdad pasaste.

Vale la pena pausar en una decisión de diseño: `probe()` devuelve un registro en vez de imprimir su propia salida. Esa división, medir en un lugar, presentar en otro, parece ceremonia en un archivo de veinte líneas, y es la razón por la que el challenge de abajo es fácil en vez de una reescritura. Una función que devuelve valores `ProbeResult` se puede llamar diez veces y sus resultados se pueden recoger, ordenar, resumir; una función que imprime ya gastó su respuesta. Las lecciones siguientes le exigen a `probe(url)` exactamente este contrato, así que la forma que escribes ahora es estructural.

**3. Llena el TODO 1: el par de tiempos.** El patrón es timestamp, esperar el trabajo con await, timestamp, restar:

```typescript
  const started = performance.now();
  const res = await fetch(url);
  const latencyMs = performance.now() - started;
```

El orden es todo acá. Las dos llamadas a `performance.now()` tienen que encerrar al `await`; pon la segunda llamada antes del await y estarías midiendo el costo de arrancar el request, no de terminarlo. Este par de encerrar-el-await es el patrón más reusado de todo este curso. Lo vas a escribir en Rust con `std::time::Instant` dentro de poco, misma forma, lenguaje distinto.

![Tres líneas de código anotadas que muestran un timestamp antes de un fetch, la llamada esperada con await, y la resta que da la latencia.](assets/v05-annotated-code.webp)

**4. Llena el TODO 2: la línea de salida.**

```typescript
console.log(`${result.url} ${result.status} ${result.latencyMs.toFixed(1)}ms`);
```

`toFixed(1)` mantiene un decimal: los dígitos por debajo del milisegundo son ruido a escala de red. Una línea por sonda, divisible por máquina en espacios. Esa es una decisión de diseño diminuta que paga en dos lecciones cuando un workflow parsea esta salida.

**5. Haz saltar la flag a propósito.** Ahora verifica el archivo:

```bash
npx tsc --noEmit
```

Falla, y debería:

```text
probe.ts: error TS2345: Argument of type 'string | undefined' is
not assignable to parameter of type 'string'.
```

Esto es `noUncheckedIndexedAccess` haciendo su trabajo, y yo planté el choque: `process.argv[2]` podría no existir. Corre la sonda sin URL y, sin esta flag, `fetch(undefined)` produciría un error desconcertante en tiempo de ejecución tres capas más abajo. El compilador se niega a dejar que la situación exista. Esto no es una novatada. La flag encontró un hueco real en un programa real de once líneas.

**6. Arréglalo a la manera de la flag.** Pon una guarda antes de usarlo:

```typescript
const target = process.argv[2];
if (!target) {
  console.error("usage: npx tsx probe.ts <url>");
  process.exit(1);
}
```

Después del `if`, TypeScript estrecha `target` a `string` simple: el caso undefined sale del proceso, así que no puede llegar a `probe()`. No silenciaste al verificador; manejaste el caso, y conseguiste un mensaje de uso gratis. Verifica otra vez:

```bash
npx tsc --noEmit
# (silence; silence is a pass)
```

**7. Córrelo de verdad.**

```bash
npx tsx probe.ts https://www.rust-lang.org
# https://www.rust-lang.org 200 498.4ms
```

Tu número va a ser distinto; el mío lo fue entre dos ejecuciones a segundos de diferencia. Lo que tiene que coincidir es la forma: URL, estado, latencia con un decimal. Prueba una segunda ejecución y mira cómo baja la latencia a medida que las conexiones se calientan. Prueba `npx tsx probe.ts` sin argumento y mira tu línea de uso en vez de un crash.

Si en cambio algo te peleó, las dos fallas clásicas de setup producen errores inconfundibles, las dos de mi terminal:

- `error TS2304: Cannot find name 'performance'` (o `'fetch'`, o `'process'`): tu `tsconfig.json` todavía tiene el valor por defecto de init `"types": []`. Ponlo en `"types": ["node"]` y vuelve a correr.
- `error TS1309: The current file is a CommonJS module and cannot use 'await' at the top level`: a `package.json` le falta `"type": "module"`. Corre `npm pkg set type=module` y vuelve a correr.

Cualquier otra cosa, lee el error despacio antes de buscarlo. Los mensajes de TypeScript 7 suelen nombrar la flag o el arreglo sin rodeos, y construir el reflejo de leer-el-error ahora paga interés compuesto todo el curso.

![Un diagrama de flujo desde el argumento de comando pasando por una guarda y un fetch cronometrado hasta una sola línea de resultado impresa.](assets/v06-flowchart.webp)

**Checkpoint, la barrera de la lección:** ahora deberías tener una línea de latencia real para una URL real impresa por tu propio toolchain con verificación estricta, más la memoria de un error del compilador que disparaste y arreglaste. Una línea de terminal y un mensaje de error. Ese par es toda la victoria: la sonda funciona, y viste la maquinaria que la mantiene honesta.

## Challenge: sondea una flota

Solo ahora. Extiende `probe.ts` para aceptar múltiples URL:

```bash
npx tsx probe.ts https://www.rust-lang.org https://www.typescriptlang.org https://nodejs.org
```

Requisitos:

- Imprime una línea de latencia por target, mismo formato que v0.
- Ordena la salida por latencia, el más lento al final.
- El caso sin argumentos todavía imprime la línea de uso y sale.
- `npx tsc --noEmit` se queda en silencio.

Pistas, no pasos: `process.argv.slice(2)` te da todas las URL de una. Ya tienes un `probe()` que devuelve un `ProbeResult`; un array de esos se puede ordenar con un comparador sobre `latencyMs`. Que sondees secuencialmente o dispares todos los fetches al mismo tiempo es tu decisión, pero fíjate que cambia lo que quieren decir los números: las sondas secuenciales tienen la red para ellas solas, mientras que las concurrentes comparten tu conexión y pueden inflarse la latencia entre ellas. Ninguna está mal, miden cosas distintas, y saber qué pregunta estás haciendo es la habilidad de verdad. Vamos a formalizar exactamente ese trade-off en la lección de async.

Y cuenta con encontrarte con la flag otra vez. En el momento en que indexes tu array de resultados, `noUncheckedIndexedAccess` te va a recordar que el array podría estar vacío, y esta vez no hay arreglo impreso que copiar. Ya conoces su jugada; maneja el caso a la manera de la flag.

Si quieres un segundo entrenamiento, la página de esta lección en la plataforma del curso trae un coding challenge complementario en su panel de editor interactivo (código starter y calificador incluidos, nada que descargar): `latencyStats`, que convierte un batch de muestras en min, max, media y p95. Una muestra es ruido, un resumen es señal; esa función exacta se entrega en la flota el próximo módulo, cuando el reporte de flota de m02-l3 la pone en servicio en la estación. Las lecciones siguientes reparten sus challenges de la misma forma, así que cuando una dice "el starter", ese panel es donde vive.

## A dónde va el latido después

`pulse` v0 funciona, y lo viste funcionar. Esa última parte es el problema. Un latido que tienes que estar cuidando no es un latido; mide solo cuando te acuerdas de preguntar. La próxima lección tu sonda se mueve a una máquina que no es tuya y corre según un calendario: git para versionarla, GitHub para alojarla, y tu primera ejecución verde de Actions para ejecutarla. La primera entrega del curso.

![Un diagrama de escalera que muestra pulse v0 hoy, su paso a ejecuciones programadas, y la reescritura tipada posterior, con un solo contrato sostenido de principio a fin.](assets/v07-diagram.webp)

Antes de irte: corre la sonda contra un sitio que de verdad te importe y mira el número. Si algo de arriba te peleó — un desajuste de versión, un error de flag que no pudiste descifrar — ese es exactamente el feedback que quiero; tráelo a la discusión del curso, en el peor caso se vuelve la caja de troubleshooting de la próxima cohorte. Nos vemos en el primer check verde.
