# Publica algo real: tsdown y las señales de una dep que se muere

## Resumen

La lección pasada entregó la primera URL: pulse-board en vivo en Vercel, tanteado desde el teléfono de un desconocido. La estación tiene una cara pública, pero todavía no un motor público: `pulse-core`, el clasificador con el que cada parte de la estación está de acuerdo, sigue viviendo solo adentro de tu workspace, importable por tus paquetes y por los de nadie más. Hoy es la vuelta de honor del módulo. Construyes `pulse-core` con una herramienta de build real, lees el tarball que estás por entregarle al mundo, lo publicas en npm como un paquete público con scope, y demuestras que se instala para un desconocido. Después la lección enseña la habilidad escondida adentro de la elección de herramienta: leer si una dependencia está viva antes de adoptarla, usando la herramienta de build que acabamos de elegir como ejemplo trabajado, porque la herramienta que reemplazó está enseñando esa lección sobre sí misma en su propio README.

La mayoría de quienes desarrollan publican su primer paquete sin mirar nunca adentro. Tú no vas a ser uno de ellos. Haz esto ahora mismo, desde la raíz del workspace:

```bash
cd packages/pulse-core
pnpm add -D tsdown@0.22.14 typescript
npx tsdown src/index.ts
npm pack --dry-run
```

Ese pin está al día al 2026-09-02; tsdown ya tiene un release candidate 0.23 tagueado, así que espera un dígito más alto para cuando tipees esto, y toma lo que `pnpm add -D tsdown` te dé si el pin envejeció. La línea de `typescript` está ahí porque el layout estricto de pnpm significa que un paquete tiene que declarar lo que usa su build, incluso cuando la raíz del workspace ya lo tiene.

El último comando imprimió un listado de archivos. Léelo despacio, cada línea, como lo leería un desconocido, porque en unos veinte minutos un desconocido PUEDE. El mío mostró `dist/index.mjs`, `package.json`, y después cada archivo de `src/` viajando de arriba sin invitación (tres en un repo que siguió los labs: `index.ts`, el clasificador, los helpers de backoff). El tuyo se va a ver parecido. Fíjate también en lo que NO está ahí: ninguna declaración de tipos, lo que para un paquete cuyo valor entero son sus tipos unión sería una catástrofe silenciosa; la sección de build de abajo explica por qué la corrida zero-config no pudo emitirlas y lo arregla. Ese listado es el contenido exacto de lo que `npm publish` subiría hoy, y hoy es un desastre. Toda la primera mitad de esta lección es convertir ese listado en un contrato.

El repliegue de la ayuda, dicho en voz alta: la configuración del build y el cableado de los exports los hacemos juntos, recorridos línea por línea. El publish y la demostración en el proyecto borrador los corres tú desde una checklist. El ejercicio de cierre, leer los signos vitales de tres paquetes reales, es completamente sin guía, y es el último peldaño de la escalera del tier de TypeScript. El módulo que viene reinicia la escalera desde abajo para un lenguaje nuevo.

## El tarball y los signos vitales

### Lo que compra un paso de build, y qué es tsdown

Hasta ahora `pulse-core` entregaba TypeScript crudo y se salía con la suya, porque cada consumidor vivía en el mismo workspace y hablaba TypeScript por el mismo tooling. m03-l1 dijo que la lección de publish iba a cambiar eso, y esta es la lección de publish. No se puede asumir que el proyecto de un desconocido compile tus archivos `.ts`; algunos runtimes experimentan con correr TypeScript directo, pero una librería publicada que lo requiere encogió su audiencia sin razón. Así que un paquete que va al registro entrega dos artefactos: JavaScript compilado para todo runtime, y declaraciones de tipos `.d.ts` para que los consumidores de TypeScript conserven cada garantía que se ganaron los tipos unión en M2. Una fuente, dos salidas, y una herramienta cuyo trabajo entero es emitir las dos correctamente.

Esa herramienta, para nosotros, es tsdown. Es la sucesora de tsup, el default que reinó por mucho tiempo para exactamente este trabajo, y la razón por la que pasamos de largo a la titular es la segunda mitad de esta lección, así que aguanta la pregunta unas secciones. Mecánicamente: tsdown empaqueta tu punto de entrada con Rolldown, el bundler en Rust que también mueve a Vite 8, y puede emitir declaraciones al lado del JavaScript. Ya lo corriste una vez con cero config. Dos observaciones de esa corrida vale fijarlas antes de configurarlo.

Primero, escribió `index.mjs`, no `index.js`. tsdown va por default a extensiones que gritan "ESM" sin importar lo que diga el `package.json` de alrededor, que es ingeniería defensiva para paquetes que entregan formatos duales. Nuestro paquete declaró `"type": "module"` allá en m03-l1, un `.js` pelado ya es ESM acá, y hacer coincidir la extensión que espera el resto de nuestro workspace mantiene aburrido el mapa de exports. Así que apagamos ese default.

Segundo, las declaraciones que faltan. Emitir declaraciones es trabajo de TypeScript, tsdown solo lo orquesta, y la maquinaria de TypeScript se niega a correr sin un `tsconfig.json` en el paquete. `pulse-core` nunca tuvo uno: la extracción de m03-l1 movió el único tsconfig del repo a `pulse-fleet` junto con todo lo demás, y nada desde entonces necesitó uno acá, porque vitest y los consumidores del workspace resolvían todos el código fuente crudo. El paso de build es donde termina ese viaje gratis. Pide declaraciones sin un tsconfig y el build muere con `ERROR Error: tsgo generator requires a tsconfig file to be specified.` Así que el paquete recibe primero su propio contrato-con-el-compilador, `packages/pulse-core/tsconfig.json`, mínimo y estricto, coincidiendo con la configuración que la flota usó todo el curso:

```json
{
  "compilerOptions": {
    "target": "es2023",
    "module": "nodenext",
    "moduleResolution": "nodenext",
    "strict": true,
    "declaration": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

Ahora la configuración de tsdown, `packages/pulse-core/tsdown.config.ts`:

```ts
import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/index.ts"],
  format: "esm",
  dts: true,
  fixedExtension: false,
});
```

Cuatro líneas, cada una ganándose su lugar: el entry es el mismo `src/index.ts` curado que ha sido la superficie pública desde la extracción, `format` es ESM porque este curso entrega un solo sistema de módulos, `dts` pide declaraciones (que es por qué el tsconfig de arriba tenía que existir), y `fixedExtension: false` es el opt-out que nos consigue `.js` y `.d.ts`. Una nota cosmética para esta corrida y para todas las que vienen: espera una línea `WARN TypeScript 7.0 does not yet have a stable API and is experimental`. Al momento de escribir esto (2026-09-02, tsdown 0.22.14 contra typescript 7.0.2, la estable actual) es exactamente lo que dice, una advertencia; las declaraciones se emiten bien. Anótalo como nota de versión y sigue. Agrega el script a `packages/pulse-core/package.json`:

```json
{
  "scripts": {
    "build": "tsdown"
  }
}
```

Corre `pnpm build` desde el directorio del paquete y `dist/` ahora tiene `index.js` e `index.d.ts`. Salida total para nuestro motorcito: dos o tres kilobytes. Chico es lo correcto; este paquete son tres módulos de lógica pura, y el tamaño del tarball está por volverse algo que lees, no algo que adivinas.

![El código fuente fluye por tsdown hacia dist, se empaqueta en un tarball, y se entrega al registro, con una inspección de corrida en seco antes de subirlo.](assets/v01-flowchart.webp)

### La preocupación dual: los tipos tienen que viajar con el JavaScript

El mapa de exports ha sido la puerta de entrada del paquete desde m03-l1: una allowlist de puntos de entrada, con los imports profundos rechazados con `ERR_PACKAGE_PATH_NOT_EXPORTED`. Ahora mismo apunta al código fuente de TypeScript. Apúntalo a la salida construida en cambio, y fíjate en que el valor para `"."` crece de un string a un objeto con dos condiciones:

```json
{
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./dist/index.js"
    }
  }
}
```

Esta es la preocupación dual en el corazón de publicar paquetes tipados, y merece el énfasis. Bajo la resolución moderna (la familia `nodenext` que usan los tsconfig de tus consumidores), TypeScript recorre el mapa de exports para encontrar tipos, el mismo mapa que Node recorre para encontrar JavaScript. Dos resolvedores, un mapa. La condición `types` tiene que estar justo ahí al lado del punto de entrada de JavaScript que describe, y el orden importa: `types` va primero en el objeto de condiciones, porque los resolvedores toman la primera condición que coincide y `default` coincide con todo.

Sáltatelo y fabricas el reporte de bug más confuso que recibe quien escribe una librería: el JavaScript se importa y corre perfecto, y los consumidores de TypeScript no pueden encontrar tu módulo. Rompí esto deliberadamente en un proyecto borrador mientras escribía esta lección, estacionando las declaraciones en algún lugar al que el mapa no apunta, y el error vale leerlo completo porque algún día un consumidor te lo va a pegar:

```text
error TS7016: Could not find a declaration file for module '@kaue/pulse-core'.
  There are types at '.../node_modules/@kaue/pulse-core/types/index.d.ts',
  but this result could not be resolved when respecting package.json "exports".
  The '@kaue/pulse-core' library may need to update its package.json or typings.
```

Lee la línea del medio dos veces. TypeScript ENCONTRÓ las declaraciones. Se negó a usarlas, porque cuando existe un mapa de exports este gobierna todo, y un archivo que el mapa no expone bien podría no existir. El JavaScript siguió corriendo todo el tiempo. Esa asimetría, JS que funciona con tipos invisibles, es exactamente lo que la demostración en el proyecto borrador del lab existe para atrapar antes de que lo haga un desconocido.

![El mismo paquete funciona para un consumidor de Node en cualquier caso, pero los consumidores de TypeScript pierden todos los tipos cuando el mapa de exports omite su condición types.](assets/v02-comparison.webp)

Una consecuencia de reapuntar el mapa, dicha sin adornos para que nunca te sorprenda: tus consumidores del workspace ahora también resuelven `dist/`. El panel y la flota necesitan `pulse-core` construido antes de que su propio tooling pueda verlo, así que un clon fresco corre el build del core una vez antes que nada, y mientras editas activamente código del core mantienes `npx tsdown --watch` corriendo para que los consumidores siempre vean salida fresca. Y "clon fresco" no es hipotético: dos de tus consumidores automatizados son clones frescos en cada corrida, así que cablea el build en los dos AHORA, antes de que el próximo push los ponga en rojo. En `.github/workflows/pulse.yml`, agrega un paso justo después de la línea `pnpm install --frozen-lockfile` en los tres jobs:

```yaml
      - run: pnpm --filter pulse-core build
```

(El filtro coincide con el campo `name` del paquete; cuando el lab renombre el core a tu scope de npm, actualiza los tres filtros al nombre con scope.) Y para el deploy de Vercel conectado a git, que construye el panel desde su propio clon fresco, prefija el script de build del panel en `packages/pulse-board/package.json` para que el core siempre se construya primero:

```json
{
  "scripts": {
    "build": "pnpm --filter pulse-core build && tsc -b && vite build"
  }
}
```

Esa única edición también arregla el primer `npm run build` local de cualquier colaborador futuro. Sáltate cualquiera de los dos cableados y la falla llega en el próximo push, en los logs de alguien más, dos superficies más allá de este párrafo. Ese es el costo honesto de un mapa sirviendo a las dos audiencias. El patrón más profundo, dejar que el workspace resuelva el código fuente mientras solo el artefacto publicado apunta a dist, existe en los overrides de `publishConfig` de pnpm, y es territorio de bookmark, no el camino de hoy.

### El tarball es el producto

Ahora arregla el listado que leíste arriba. Lo que `npm publish` sube es exactamente lo que `npm pack` ensambla, y lo que npm empaqueta es, por default, casi todo lo que está en la carpeta: código fuente, archivos de config, notas sueltas, cualquier cosa con forma de `.env` que se haya metido. Nada se quita después. El tarball ES el entregable; lo que muestre el listado es lo que aterriza, byte por byte, en el `node_modules` de cada consumidor, y gracias a la política de unpublish fuertemente restringida de npm es efectivamente para siempre. El arreglo es el campo `files`, una allowlist, la misma filosofía que el mapa de exports una capa más abajo:

```json
{
  "files": ["dist"]
}
```

`package.json` en sí, el README y los archivos de licencia se entregan siempre igual, que es lo que quieres. Corre el prevuelo otra vez, `npm pack --dry-run`, y el listado se colapsa a tres entradas:

```text
npm notice 📦  pulse-core@0.1.0
npm notice Tarball Contents
npm notice 510B dist/index.d.ts
npm notice 607B dist/index.js
npm notice 374B package.json
```

Los tamaños van a diferir; la forma no debería. Salida compilada, declaraciones, manifest, nada más. Ningún `src/`, ninguna config, ningún archivo de test, nada con forma de entorno. Este hábito de corrida-en-seco-antes-de-publicar cuesta veinte segundos y es el hábito de profesionalismo-y-seguridad más barato de todo este curso. La historia de lo que pasa cuando el lado del registro de esto sale mal es la apertura de la lección de auditoría de dependencias, allá al final del curso; por ahora, la regla alcanza: lee el listado, cada vez, antes de que el tarball se vuelva permanente.

Dos hechos de publicación completan el patrón. Primero, los nombres: `pulse-core` como nombre pelado le pertenece a quien lo registró primero, así que publicas bajo tu scope, el prefijo `@username/` que recibe gratis toda cuenta de npm, donde el namespace es tuyo por completo. Segundo, un gotcha de lab que vale anticipar: los paquetes con scope van a PRIVADO por default en el primer publish, los paquetes privados son una feature paga, y así un `npm publish` pelado de un paquete con scope en una cuenta gratis falla con un error sobre pagos que se lee como un bug de facturación. No lo es. Es npm preguntando qué visibilidad quisiste decir. La flag `--access public` es la respuesta, y olvidarla es un rito de paso del que este párrafo acaba de salvarte.

**Profundiza (el 20%).** esta lección enseñó la mecánica de publicación que ejercita nuestro artefacto: build, exports, files, pack, publish. El resto del mundo de entregar-una-librería, los workflows de watch, múltiples puntos de entrada, targets de plataforma, decisiones de no-empaquetar, vive en los [docs de tsdown](https://tsdown.dev/guide/) (URL sondeada el 2026-09-02), y la capa de automatización arriba de todo eso, changesets, publicación manejada por CI, atestaciones de procedencia, es maquinaria real que este curso deliberadamente señaliza en vez de enseñar. Nada en el lab depende de nada de eso.

### Leer los signos vitales de una dependencia

Ahora la pregunta que estacioné: ¿por qué tsdown y no tsup, la herramienta con años de reinado y, hasta el día de hoy, más descargas?

Porque el propio README de tsup la responde en su primera oración. Arriba, sobre el nombre del proyecto, hay una advertencia de quien lo mantiene: "Este proyecto ya no se mantiene activamente. Por favor considera usar tsdown en su lugar." Una oración, escrita por la persona que sabría, en 2025, y puede ser la oración más honesta que produjo el ecosistema ese año. Los números a su alrededor vuelven perfecto el caso de estudio. El último publish de tsup es 8.5.1, fechado 2025-11-12. Sus descargas para la semana que terminó el 2026-08-29: unos 8.5 millones. tsdown, la sucesora nombrada, publicando activamente, la misma semana: unos 5.7 millones. La herramienta abandonada todavía le gana en descargas a su propio reemplazo, diez meses después de su release final, y repos serios todavía dependen de ella; el workspace de kit de anza y el SDK de gill los dos construyen con tsup hoy.

Quédate con esa tensión, porque es la meta-habilidad de toda esta lección: los conteos de descargas son un indicador rezagado con años de inercia horneada adentro. Cada lockfile, tutorial y plantilla existente sigue trayendo tsup mucho después de que su autor le dijera a todos que se fueran. El voto del mercado es información vieja. El README es la de hoy. Siempre estás a una oración de README de una dependencia abandonada, y la habilidad entera es saber buscar la oración antes de instalar, no después.

Y una cosa hay que decir en defensa de tsup, porque la lección es sobre leer señales, no sobre burlarse de los caídos: esa advertencia es BUEN mantenimiento. El autor entregó una herramienta sobre la que se apoyaba todo el ecosistema, y cuando dejó de mantenerla lo dijo, sin adornos, arriba de todo, con una sucesora nombrada y una guía de migración enlazada. Compara la alternativa que vas a conocer constantemente en el mundo real: paquetes que simplemente paran calladitos, sin aviso, con los issues acumulándose, las descargas siguiendo su marcha. Una nota honesta de abdicación es un regalo. Aprende a recibirla.

![La tsup sin mantenimiento todavía registra más descargas semanales que su sucesora tsdown, mostrando a la popularidad sobreviviendo al adiós de quien la mantenía.](assets/v03-chart.webp)

Así que sistematízalo. Antes de adoptar una dependencia, cinco señales, en orden de ranking:

1. **Avisos del README.** Las propias palabras de quien mantiene le ganan a toda métrica de esta lista. Advertencias de deprecación, "buscando mantenedores", punteros a sucesores. Treinta segundos en la página del repo.
2. **Fecha del último publish, leída contra la cadencia natural del proyecto.** `npm view <pkg> time.modified` da la fecha; el juicio es contextual. Una utilidad completa puede quedarse callada y estar bien; un bundler que sigue un ecosistema en movimiento y se queda en silencio un año es otra historia. La fecha es el dato, la cadencia es el lente.
3. **Tendencia de descargas contra conteo absoluto.** `curl -s https://api.npmjs.org/downloads/point/last-week/<pkg>` para la foto, el gráfico de la página de npm para la forma. Los 5.7 millones de tsdown como sucesora joven son una señal de vitalidad más fuerte que el número más grande, más viejo y en declive de tsup.
4. **Punteros a sucesores.** Cuando el README, los issues o la conversación del ecosistema apuntan todos a algún lugar específico, la sucesión ya pasó socialmente incluso si los números no se pusieron al día.
5. **Preferencias reveladas de los repos en los que confías.** ¿De qué dependen las bases de código que ya lees? Cuando los repos que respetas empiezan a migrar, ese es el ecosistema votando con sus lockfiles, adelantado al gráfico de descargas.

![Cinco señales rankeadas para juzgar una dependencia, desde las propias palabras de quien la mantiene hasta las preferencias reveladas del ecosistema, cada una con su comando de revisión.](assets/v04-table.webp)

Dos lecturas trabajadas más calibran la checklist contra sus modos de falla, porque una checklist que solo corriste sobre un paquete enseña la lección equivocada.

Express, el modo de falla de la impaciencia. Express 4.0.0 se publicó el 2014-04-09. Express 5.0.0: 2024-09-10, fechas del registro, una década y cinco meses entre majors. Por una prueba de "versión major reciente", Express pasó diez años pareciendo muerto mientras movía lo que ahora son unos 133 millones de descargas por semana, nueve cifras, con releases de mantenimiento continuando justo hasta este verano. Lento no es muerto. Un paquete maduro en reposo muchas veces solo está terminado, y la checklist lee las señales de quien mantiene contra la cadencia precisamente para nunca confundir estabilidad con abandono.

esbuild, el modo de falla de la superstición del número de versión. esbuild está en 0.28.2 al momento de escribir esto, cero-punto-x después de años como una de las herramientas más estructurales del mundo de JavaScript; es el motor sobre el que tsup misma estaba construida, y publicó dentro del último mes. Mientras tanto un montón de paquetes con un confiado 2.x o 3.x no han visto un commit en años. Los dígitos de versión son branding. No son signos vitales, y nada en la lista de cinco señales pide uno.

![Express se arrastra entre majors y sin embargo prospera, tsup termina en una nota de despedida a pesar de su uso enorme, y esbuild se mantiene sana sin salir nunca del cero punto x.](assets/v05-timeline.webp)

El trade-off que completa el cuadro, y ahora te apunta a ti: publicar es un compromiso vestido de hito. En el momento en que la versión 0.1.0 existe en el registro, el lockfile de cada consumidor es una promesa que estás cumpliendo, disciplina de semver, changelogs, respuesta de seguridad, todo el paquete, y un paquete abandonado CON usuarios es peor que ningún paquete, porque en ese punto te volviste el README de tsup, ojalá con su honestidad. Así que nombra el alcance honesto de hoy: publicas para aprender la mecánica y para reclamar una pieza de portafolio real, algo defendible para un paquete 0.x con un consumidor conocido, tú. Adopta la carga de mantenimiento deliberadamente solo para código que genuinamente quieras que corran desconocidos. La checklist corta para los dos lados; algún día alguien la corre sobre ti, y lo más amable que puede hacer tu futuro paquete fantasma es decirlo arriba de su README.

Esta lectura de cinco señales vuelve al final del curso, sistematizada en un veredicto de auditoría escrito en el lab de auditoría de dependencias, donde lo que está en juego deja de ser la elección de herramienta y empieza a ser la cadena de suministro.

### La barrera del tier: lo que M1 a M3 saltearon, y dónde vive

El tier de TypeScript termina en esta lección, así que el curso te debe el mapa que prometió en m01-l1: lo que deliberadamente no enseñamos, y el hogar nombrado de cada pieza. Este es territorio debido, no una disculpa. El 80% que ahora tienes es real: uniones y narrowing, fronteras y zod, disciplina de async, un cron con pruebas, un workspace, un panel, una URL, y desde hoy un paquete publicado. El 20% nunca faltó. Estaba archivado:

- **Programación a nivel de tipos y escribir genéricos.** Consumes genéricos con fluidez (`z.infer`, `ReturnType`, el toolkit de m02-l2); todavía no escribes tipos condicionales y mapeados. El patio de ejercicios es type-challenges (el gimnasio post-M2 de m02-l1) más el capítulo de Generics del Handbook. Ve cuando la firma de tipos de una librería te dé curiosidad en vez de cansancio.
- **La superficie completa de tsconfig.** Tienes el canon estricto de la tabla de m01-l2; las varias docenas de flags restantes se buscan por flag, a demanda, para siempre. Nadie las memoriza. Ahora sabes eso.
- **node:test.** El runner sin dependencias tuvo su barra lateral honesta en m02-l4; vitest es el carril de este curso. Si un contexto libre de dependencias quiere pruebas, la barra lateral es la rampa de entrada.
- **Bun y Deno.** Los dos vivos y entregando (Bun 1.4, Deno 2.9, los dos con releases de fines de agosto de 2026). Este curso corre Node porque el ecosistema de Solana relevado lo hace: cada repo estudiado declara engines de Node y un pin de packageManager de pnpm, ninguno declara Bun o Deno. Posicionamiento, no desdén; revisa la señal cinco en un año y mira si los lockfiles se movieron.
- **React más allá de la porción de consumidor-de-datos, y todo lo del cliente.** Ruteo, formularios, librerías de estado, UX de wallet, aterrizaje de transacciones: ese es el territorio del curso de dominio del lado del cliente, en producción mientras escribo esto, y la fuerza en TS que ahora tienes es exactamente el cimiento sobre el que se construye ese tipo de trabajo.
- **jest.** Nombrado, no enseñado: es la hermana mayor de vitest y te la vas a encontrar en los repositorios de anza. La superficie de API es lo bastante cercana como para que tu fluidez con vitest se transfiera casi toda.

Esa es toda la barrera. Cada bookmark tiene una dirección, cada dirección tiene un disparador de cuándo visitarla, y el mapa que recibiste en la apertura del curso acaba de ganar su primer pin de "estás acá": fuerte en TS, con las ubicaciones del 20% memorizadas.

![Una región central asentada de habilidades enseñadas está rodeada por seis territorios con bookmark, cada uno etiquetado con exactamente a dónde ir cuando haga falta.](assets/v06-diagram.webp)

## Lab: pulse-core, publicado

La mitad trabajada está lista: el build corre, el mapa de exports lleva tipos al lado del JavaScript, el tarball está limpio. Lo que queda es tuyo, desde una checklist. Vas a necesitar una cuenta gratis de npm: regístrate en npmjs.com si no lo hiciste, y anota tu usuario, porque está por volverse un namespace.

1. **Reclama tu scope en el nombre.** En `packages/pulse-core/package.json`, cambia el nombre a tu scope: `"name": "@YOUR_NPM_USERNAME/pulse-core"`. Los nombres del registro tienen que ser únicos; tu scope es el rincón del registro donde la unicidad es tu problema y de nadie más.

2. **Re-enlaza los consumidores sin tocar un import.** La flota y el panel importan de `"pulse-core"`, y el protocolo de workspace de pnpm tiene una forma de alias hecha para exactamente este renombre. En `packages/pulse-fleet/package.json` y `packages/pulse-board/package.json`, cambia la línea de dependencia:

   ```json
   {
     "dependencies": {
       "pulse-core": "workspace:@YOUR_NPM_USERNAME/pulse-core@*"
     }
   }
   ```

   Después `pnpm install` desde la raíz. El alias dice: el especificador local `pulse-core` resuelve al paquete del workspace ahora llamado `@YOUR_NPM_USERNAME/pulse-core`. Cada línea `import { classifyProbe } from "pulse-core"` de toda la estación sigue funcionando, textualmente. Checkpoint: `pnpm -r test` en verde desde la raíz, cero líneas de import cambiadas.

3. **Construye y prevuelo.** Desde `packages/pulse-core`: `pnpm build`, después `npm pack --dry-run`. La aceptación es la forma limpia de la sección de teoría: `dist/index.js`, `dist/index.d.ts`, `package.json`, nada más. Si aparece algo extra, el campo `files` es tu allowlist; arréglalo y vuelve a correr la corrida en seco hasta que el listado sea aburrido.

4. **Publica.** Dos comandos, una flag que importa:

   ```bash
   npm login
   npm publish --access public
   ```

   `npm login` rebota por el navegador. La flag `--access public` es el gotcha de la sección de teoría: sin ella, un primer publish con scope falla con un error sobre planes de pago, porque los paquetes con scope van a privado por default y privado es pago. Con ella, la terminal imprime el nombre y la versión de tu paquete, y esa es toda la ceremonia. `@YOUR_NPM_USERNAME/pulse-core@0.1.0` ahora existe en el registro público. Ve a mirar su página en npmjs.com; ahora tienes una página de artefactos entregados. Va a decir que no se encontró README, con verdad, porque el tarball limpio que inspeccionaste en el paso 3 no tiene ninguno; escribir uno es el primer pulido post-entrega que esta lección te deja.

5. **Demuéstralo como un desconocido.** En algún lugar AFUERA del workspace, tu directorio home, donde sea:

   ```bash
   mkdir pulse-scratch && cd pulse-scratch
   npm init -y
   npm pkg set type=module
   npm i @YOUR_NPM_USERNAME/pulse-core
   ```

   Después `smoke.mjs`:

   ```js
   import { classifyProbe } from "@YOUR_NPM_USERNAME/pulse-core";

   console.log(classifyProbe("ok", 240));
   console.log(classifyProbe("ok", 700));
   ```

   `node smoke.mjs` debería imprimir `up` y después `degraded`, directo del contrato de m02-l1: menos de 400 es up, de 400 a 1000 es degraded. Este es tu código, instalado desde la internet pública, corriendo el mismo juicio con el que publica tu cron. Si quieres la demostración completa, corre `npm i -D typescript` (un `npx tsc` pelado en un proyecto que no lo tiene resuelve el paquete equivocado del registro, un stub deprecado llamado `tsc`), agrega un `tsconfig.json` con configuración estricta de `"module": "nodenext"` y un archivo `.ts` importando `ProbeResult`; que `npx tsc --noEmit` pase demuestra que los tipos viajaron. Este paso es el que atrapa la falla de tipos-invisibles de la sección de teoría, que es por qué es un paso y no una sugerencia.

6. **Di la verdad del cableado en voz alta.** De vuelta en el workspace: `pnpm -r test`, todavía en verde. Abre el panel, todavía renderizando. Y como el renombre acaba de pasar, termina el cableado del pipeline de la sección de teoría: actualiza el `pnpm --filter` en los tres jobs de `pulse.yml` y en el script de build del panel al nuevo nombre con scope, empuja, y mira cómo la corrida de Actions y el deploy de Vercel se mantienen los dos en verde desde sus propios clones frescos. Solo entonces la afirmación es honesta en todos lados, no solo en la laptop donde `dist/` ya existe. Nada de la estación consume la copia de npm; el cron y el panel resuelven la copia del workspace por el alias, el mismo symlink que ayer. Publicar cambió el ALCANCE del paquete, no el cableado de la estación. Ahora existen dos copias de la verdad, el workspace para ti, el registro para desconocidos, y mantenerlas honestas entre sí es para lo que sirven los números de versión, una disciplina que el tier de Rust va a volver a encontrar desde el lado de cargo.

![El mismo paquete llega en vivo a los consumidores del workspace por un symlink, mientras los desconocidos reciben la copia congelada del registro publicada en npm.](assets/v07-diagram.webp)

## Challenge: tres veredictos, sin barandas

El ejercicio sin guía, y la última rep del tier. Corre la checklist de cinco señales sobre estos tres paquetes reales: `request`, `body-parser` y `zod`. Para cada uno, escribe un veredicto de un párrafo, adoptar, evitar, o adoptar-con-los-ojos-abiertos, citando al menos dos señales concretas que revisaste personalmente: un aviso de README o del registro, una fecha de último publish leída contra la cadencia, una cifra de descargas con su fecha, un puntero a un sucesor, o la preferencia revelada de un repo nombrado. Los comandos ya están en tus manos: `npm view <pkg>`, `npm view <pkg> time.modified`, el endpoint de descargas, y la página del repo. Uno de estos tres está en plena salud, uno viene diciéndole a la gente que se vaya desde hace años mientras millones siguen llegando cada semana, y uno está en algún lugar más interesante; no te voy a decir cuál es cuál, porque leer eso en frío es la habilidad entera. Aceptación: tres párrafos, cada afirmación verificable, y al menos un veredicto que te sorprendiera lo bastante para revisarlo dos veces.

## Checkpoint, y el relevo de lenguaje

Lo que ahora puedes hacer, concretamente: llevar un paquete del workspace desde código fuente de TypeScript hasta un artefacto de npm público, con scope e instalable, con tipos que viajan; leer un listado de tarball como prevuelo y mantenerlo limpio con una allowlist; y leer los signos vitales de una dependencia en orden de ranking antes de adoptarla. La recuperación de 30 segundos antes de cerrar la pestaña: nombra las dos señales que le ganan a los conteos de descargas cuando juzgas la salud de una dependencia. Dilas en voz alta. El aviso de README de quien la mantiene, y la fecha del último publish leída contra la cadencia natural del proyecto. Si esas dos llegaron al instante, la meta-habilidad está instalada.

Dos pedidos mientras está fresco. Primero, pon la URL de npm de tu paquete al lado de la URL de Vercel de la lección pasada, bio o README, donde sea que fue la primera; el portafolio se acumula. Segundo, anota en tu diario del curso cuál paso del lab te peleó, el renombre, la flag de access, la demostración en el borrador, porque el tier de Rust también publica a un registro y tu lista de fricción es la checklist que vas a querer abierta cuando crates.io haga las mismas preguntas con otra ortografía.

El tier de TypeScript está completo: flota tipada, cron con pruebas, workspace, panel, URL, paquete publicado. Cada promesa del mapa de m01-l1, cumplida y entregada. El módulo que viene el mismo motor de sondeo se reconstruye bajo un compilador que se niega a adivinar: el ownership, el verificador de préstamos, y tu primer E0382, a propósito, adentro de diez minutos. Va a sentirse como el code review más estricto de tu vida, y es el único review que no puedes saltarte. Trae el tag del paquete publicado y una piel gruesa.
