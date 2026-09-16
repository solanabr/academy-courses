# package.json es un contrato: workspaces, engines, peers

M2 cerró con una flota probada: resultados de sonda tipados, config pasada por zod, concurrencia disciplinada, y una suite de vitest que le pone barrera al cron de Actions antes de que sondee. La flota funciona. También es una sola masa indiferenciada de código, y en tres lecciones parte de ella se entrega a npm, donde la máquina de cada consumidor puede hacer cumplir promesas que todavía no hiciste conscientemente.

Así que antes de cualquier teoría, rompe una de esas promesas a propósito. Abre el `package.json` de la flota y agrega una cláusula engines que declare una major de Node que no existe:

```json
{
  "engines": {
    "node": ">=99"
  }
}
```

Ahora corre una instalación fresca con la verificación de engines prendida (npm solo avisa por defecto; la flag hace que la imponga):

```bash
rm -rf node_modules
npm install --engine-strict
```

```
npm error code EBADENGINE
npm error engine Unsupported engine
npm error engine Not compatible with your version of node/npm: pulse-station@1.0.0
npm error notsup Not compatible with your version of node/npm: pulse-station@1.0.0
npm error notsup Required: {"node":">=99"}
npm error notsup Actual:   {"node":"v24.20.0","npm":"11.19.0"}
```

(Transcripción de npm 11.19; el nombre del paquete es el que `npm init -y` tomó de tu directorio `pulse-station` allá en m01-l2, tus dígitos van a diferir, y las versiones más viejas de npm escriben la etiqueta `notsup` más larga.) Léelo como un dev que trabaja. `Required` es lo que afirma tu package.json; `Actual` es la máquina en la que aterrizó; la instalación se negó porque una promesa escrita y un entorno real no coincidieron. No un mensaje de error, una cláusula de contrato disparándose, y cada campo que cubrimos hoy se dispara exactamente así en la máquina de alguien, con el tiempo. Revierte el sabotaje y reinstala antes de seguir.

## Resumen

Los hallazgos primero:

- `package.json` es un contrato con máquinas y consumidores que nunca vas a conocer: `exports` promete una superficie de API, `engines` un rango de runtime, `packageManager` fija la herramienta, los rangos de dependencias eligen qué futuros aceptas, `peerDependencies` nombra lo que tiene que proveer el CONSUMIDOR.
- Vas a extraer el motor de la flota (la unión `ProbeResult`, `classifyProbe`, los helpers de backoff) hacia `pulse-core`, un paquete de workspace de pnpm, con la suite de m02-l4 en verde antes y después: la estructura cambia, el comportamiento no.
- La extracción se gana su ceremonia porque el segundo consumidor es real, no especulativo: la lección que viene un panel de React importa el clasificador, y en M7 un edge worker importa el core puro.
- El artefacto de peer en vivo: `helius-sdk` 3.1.0 declara peer con `@solana/kit ^6.9.0` mientras que el latest de kit está en 8.2.0 (los dos sondeados el 2026-09-02). Instálalos juntos y npm se niega. La regla durable, que se cobra en M8: fija lo que tus deps declaran peer, por workspace, nunca un dígito memorizado.
- La ayuda se repliega según un cronograma: yo manejo el primer movimiento de extracción diff por diff, tú mueves los módulos restantes desde la misma receta, y el diagnóstico del conflicto de peers más el coding challenge son solo tuyos.

## El contrato, cláusula por cláusula

Acá va la síntesis que hace que todo el archivo tenga sentido: un paquete es una promesa sobre entornos que nunca vas a ver. Tu código va a correr en una laptop que nunca tocaste, un Node que nunca instalaste, al lado de versiones que nunca elegiste, importado por una persona que nunca vas a conocer. `package.json` deja escrito cuáles de esos futuros prometes sobrevivir; cada campo de abajo es una cláusula, y el sabotaje de apertura ya te mostró la imposición.

### Por qué extraer ahora: la prueba del segundo consumidor

Todo en la flota vive en un solo `src/`. El clasificador que decide "up" contra "degraded", el tipo unión que hace que los estados equivocados sean irrepresentables, la matemática de backoff que mantiene educada a la flota: todo eso se sienta al lado del cableado de la CLI y el punto de entrada del cron. Eso era correcto. Un consumidor, una masa, cero ceremonia.

Estuve del otro lado de esto, y en grande: dos repos, cada uno con su propia copia pegada de la misma función de clasificación, y la semana en que una copia recibió un arreglo de frontera y la otra no, nuestros paneles discreparon sobre si producción estaba sana. Nadie lo notó por días porque las dos copias estaban en verde en sus propias pruebas. Las copias se desvían. Ese es el argumento entero.

La regla honesta no es "siempre extraer". Una frontera de paquete cuesta ceremonia: dos archivos `package.json` que mantener veraces, un mapa de exports que mantener, cada refactor preguntando "¿es pública esta API?" El copy-paste no tiene ninguno de esos costos, justo hasta que aparece el segundo consumidor. Entonces: extrae cuando el segundo consumidor sea REAL. El nuestro está planificado: la lección que viene el panel renderiza `status.json` con el mismo clasificador con el que publica la flota, y si los dos discrepan sobre "degraded", el panel le miente a ojos humanos. El edge worker de M7 hace tres. La extracción pasa ahora porque la ventana de deriva se abre ahora.

![El repo de la flota se parte en pulse-core que guarda los módulos del motor y pulse-fleet que guarda la app, unidos por una sola flecha de dependencia de workspace.](assets/v01-diagram.webp)

La forma que usamos es deliberadamente chica, y tiene un nombre que vale acuñar una vez: monorepo-lite. Un repo, un directorio `packages/`, un archivo de workspace de dos líneas, y nada más. Nada de Nx, nada de turborepo, nada de grafo de tareas. Esas herramientas resuelven la orquestación de builds para docenas de paquetes; nosotros tenemos dos. Agarrarlas acá sería adoptar un tren de carga para cruzar la calle. Cuando tu workspace crezca más allá del punto donde `pnpm -r` se siente lento, vas a saberlo, y las herramientas van a seguir ahí.

### El workspace y el protocolo

El cableado son dos archivos. Primero, `pnpm-workspace.yaml` en la raíz del repo le dice a pnpm dónde viven los paquetes:

```yaml
packages:
  - "packages/*"
```

Segundo, el consumidor declara su dependencia usando el protocolo de workspace:

```json
{
  "dependencies": {
    "pulse-core": "workspace:*"
  }
}
```

`workspace:*` quiere decir: resuelve esto desde el workspace, nunca desde el registro, sea la versión que sea en este momento. Al instalar, pnpm enlaza `packages/pulse-core` con un symlink adentro del `node_modules` de la flota, así que la línea de import en el código de la flota se lee exactamente como cualquier dependencia de terceros:

```ts
import { classifyProbe, type ProbeResult } from "pulse-core";
```

Con honestidad: este link es una bendición. Edita un archivo en `pulse-core`, y la flota ve el cambio al instante, sin publish, sin bump de versión, sin reinstalar. Recibes la disciplina de la frontera de paquete sin ninguno de los viajes de ida y vuelta del registro. Al publicar, `pnpm publish` reescribe `workspace:*` a un rango de versión real, así que la ergonomía se queda local y el artefacto publicado se queda honesto. Una salvedad que se vuelve estructural en m03-l4, la vuelta de honor del módulo: el `npm publish` pelado no hace ninguna reescritura de esas, y esa lección publica con npm; se sale con la suya solo porque pulse-core viene con cero dependencias de runtime, así que no existe ninguna línea `workspace:*` que se filtre al registro. Publica un paquete que dependa de un hermano del workspace, y la grafía de pnpm deja de ser opcional.

Pregunta justa antes de comprometernos: npm también tiene workspaces, ¿entonces por qué pnpm? Dos razones honestas. El ecosistema eligió: cada repo serio de Solana en TypeScript que vas a leer es un workspace de pnpm, y leer el mundo real con fluidez es una meta declarada acá. Y el layout más estricto de pnpm (los paquetes solo ven lo que declaran, no lo que sea que quedó elevado al alcance) significa que una línea de dependencia que falta falla en tu máquina hoy en vez de en la máquina de un consumidor después del publish; para un paquete que va camino a npm en tres lecciones, esa estrictez es una feature apuntada a nosotros mismos.

Una nota de la casa, dicha sin adornos para que nunca se lea como descuido: las líneas de instalación que usan los labs de este curso se estandarizan en `npm i`, porque nuestro toolchain de verificación las cosecha y las replica. pnpm es lo que corre de verdad el ecosistema de Solana en TypeScript, así que esta lección lo enseña como verdad del ecosistema y lo usa donde el workspace lo exige (`workspace:*` y `pnpm -r` son idioms de pnpm). Estás aprendiendo las dos grafías a propósito: alfabetización de npm para cualquier lado, fluidez de pnpm para los repos que de verdad vas a leer.

### exports: la frontera de API pública hecha literal

Antes de que el mapa de exports existiera en la práctica, "API pública" era un comentario y una esperanza. Cualquiera podía meter la mano en las tripas de tu paquete con `import { thing } from "pulse-core/src/classify"` y ahora el layout interno de tus archivos es estructural para desconocidos. Renombra un archivo, rompe el mundo.

El campo `exports` prohíbe exactamente esto. Es una allowlist de puntos de entrada; cualquier cosa no listada no resuelve, punto final:

```json
{
  "exports": {
    ".": "./src/index.ts"
  }
}
```

Con ese mapa, `import ... from "pulse-core"` funciona y `import ... from "pulse-core/src/classify"` lanza `ERR_PACKAGE_PATH_NOT_EXPORTED`. Tu superficie pública ahora es un solo archivo que curas tú, `src/index.ts`, re-exportando exactamente lo que los consumidores pueden tocar. Todo lo demás es privado por mecanismo en vez de por etiqueta. Exportamos fuente de TypeScript por ahora porque cada consumidor de este workspace habla TypeScript; la lección de publish agrega una etapa de build y apunta este mapa a la salida compilada, y nada de la frontera cambia.

### engines, packageManager, y el runtime que de verdad corres

`engines` declara qué runtimes afirmas soportar. El verbo importa: afirmar. Nada prueba tu código en Node 20 porque escribiste `>=20`. El mundo real muestra el hueco: el ecosistema corre Node 24 (el LTS actual al 2026-09-02; el relevo de Node 26 aterriza el 2026-10-28), mientras que los pisos de las librerías relevadas se sientan en `>=20` o `^22`, porque quienes mantienen siguen soportando runtimes que sus usuarios todavía no dejaron. `@solana/kit` mismo declara `engines.node >=20.18.0` mientras sus mantenedores seguro desarrollan en algo más nuevo.

¿Por qué los pisos van dos líneas de LTS detrás del runtime? Porque un campo engines es la intersección de cada entorno al que todavía despliegan los usuarios de un paquete. Un equipo sobre una imagen base de Node 20 lee `>=20` como "esto no nos va a dejar varados"; un bump por capricho corta a esos equipos de los arreglos sin ninguna razón técnica. El piso se mueve cuando el código necesita una API más nueva o la línea vieja sale de LTS, no antes. El `>=20.18.0` de kit dice "todavía cargamos con la flota que no migró", una amabilidad deliberada y costosa.

![Una columna muestra el runtime Node 24 que los desarrolladores usan de verdad mientras la otra apila los pisos de soporte más viejos que las librerías todavía prometen, con la regla de afirma-lo-que-pruebas abajo.](assets/v02-comparison.webp)

Para `pulse-core` la afirmación honesta es la angosta: `"node": ">=24"`, porque Node 24 es el único runtime que tocaron nuestras pruebas. Ensancharla a `>=20` sin probar en 20 sería decorar el contrato con una promesa que nadie verificó. Cuando al CI le crezca una matriz de versiones (ese hilo sigue en M6), la afirmación puede ensancharse para coincidir con la evidencia.

`packageManager` es un tipo de pin distinto: no qué runtime, sino qué gestor de paquetes y exactamente qué versión, fijable por hash, legible por herramientas:

```json
{
  "packageManager": "pnpm@11.25.0"
}
```

Este campo es la convención real del ecosistema. En el relevamiento de investigación detrás de este curso, 5 de 5 repos de Solana en TypeScript relevados fijan pnpm vía `packageManager`, y kit va más allá: su repo bloquea `npm install` y `yarn` de plano con una guarda de preinstall cuyo nombre lo dice todo, "please-use-pnpm". Copia las líneas de lab con `npm i` de este curso en un repo así y te va a rechazar. Lee el campo `packageManager` primero; el repo te dice sus reglas.

Ahora el beat de honestidad sobre el camino de instalación, porque cambió hace poco y la mayoría de los tutoriales no se pusieron al día. La herramienta que históricamente auto-activaba el pnpm correcto desde este campo era corepack, que venía adentro de Node; el 2025-03-19 el TSC de Node lo votó afuera (se fue de Node 25+, y queda solo en Node 24 LTS). El runtime decidió que los gestores de paquetes no son su trabajo, así que la línea de instalación durable es la aburrida:

```bash
npm i -g pnpm@11.25.0
```

Fijado, explícito, funciona en cada Node que tenga npm, que son todos. (Frescura de versión, y la respuesta cambió mientras se escribía este curso: 11.25.0 tenía el dist-tag `latest` el 2026-09-02, pero pnpm 12 se lo llevó el 2026-09-04, y un re-chequeo el 2026-09-06 lee `latest` = 12.3.4 con la línea 11 sobreviviendo bajo `latest-11` en 11.26.0. El curso sigue fijando 11.25.0 igual, a propósito: este dígito tiene que coincidir con el campo `packageManager` de abajo, el job de CI del paso 7, y el Dockerfile de M6, y `npm i -g pnpm@11.25.0` lo instala sin importar por dónde haya andado `latest`. Para eso sirve un pin. Para un repo tuyo, corre `npm view pnpm dist-tags` y elige deliberadamente.) `corepack enable` todavía funciona hoy en tu laptop con Node 24 y muere en la próxima imagen base; un hábito con fecha de vencimiento es un mal hábito, y el Dockerfile de M6 va a usar la línea de npm exactamente por esta razón.

![Una línea de tiempo corre desde el voto de 2025 que eliminó corepack, pasando por Node 25 dejándolo caer, hasta el comando de instalación de npm fijado que sobrevive al cambio.](assets/v03-timeline.webp)

### Rangos de semver: qué futuros aceptas

Cada línea de dependencia en `package.json` es un rango, y un rango es una política sobre el futuro: ¿qué versiones, publicadas después de que dejaste de mirar, tiene permitido pasarte el resolvedor? Cuatro formas cubren el uso real.

**Exacto** (`6.9.0`): esta versión y nada más. Máxima protección, cero arreglos.

**Caret** (`^6.9.0`): en o por encima de la base, adentro de la misma major. `6.9.1`, `6.10.0`, `6.44.0` la satisfacen todas; `7.0.0` nunca. El caret confía en la promesa central de semver, que los cambios de ruptura solo se entregan detrás de un bump de major, así que para exactamente en la frontera donde la ruptura tiene permitido vivir. Esta es la forma que npm escribe por defecto, y la forma que más vas a leer.

**Tilde** (`~6.9.0`): en o por encima de la base, misma major Y misma minor. Solo caminatas de patch: `6.9.4` sí, `6.10.0` no. La confianza más ajustada para cuando quieres arreglos pero no features.

**Piso** (`>=6.9.0`): cualquier cosa en o por encima, majors incluidas. Casi nunca lo que quiere una app, porque acepta futuros por los que semver explícitamente se niega a responder. Aparece en `engines` (donde "este runtime o más nuevo" es la intención real) mucho más que en dependencias.

Fíjate cuáles de estas escribes tú de verdad. Casi ninguna: `npm i some-dep` y `pnpm add some-dep` escriben un rango con caret por ti, así que la mayoría de las líneas de dependencia en la mayoría de los repos son una política que su autor nunca eligió conscientemente. El default es defendible (los arreglos fluyen, las majors bloqueadas), pero los equipos que quieren pins exactos lo dan vuelta deliberadamente con `npm config set save-exact true` y se apoyan en el lockfile más una herramienta de actualización. Las dos posturas son coherentes; irse a la deriva hacia una porque una herramienta la escribió por ti es la única opción equivocada.

Y la trampa adentro del caret, que se gana su propio párrafo porque leer `^` como "más o menos esta versión" te va a doler con el tiempo: bajo major cero, la minor se vuelve el slot de ruptura. `^0.3.9` admite `0.3.10` y rechaza `0.4.0`, porque los paquetes pre-1.0 reservan los bumps de minor para cambios de ruptura y el caret respeta eso. `^0.3.9` y `^1.3.9` parecen hermanos y admiten futuros completamente distintos. La regla tiene un piso más abajo de eso: con major Y minor las dos en cero, npm trata el patch como el slot de ruptura, así que `^0.0.3` admite `0.0.3` y nada más. Los paquetes `@solana-program/*` que vas a conocer en M8 viven en tierra 0.x, así que esta regla no es trivia.

![Una matriz muestra los rangos exacto, tilde, caret y piso admitiendo progresivamente más versiones desde una base 6.9.0, con una nota al pie sobre la regla del caret cero.](assets/v04-comparison.webp)

Nombra el trade-off antes de seguir, porque los rangos son política y toda política cuesta algo. Los pins ajustados te protegen de majors sorpresa y te dejan sin alimentar de arreglos; los rangos anchos traen arreglos y de vez en cuando traen una rotura de martes a la mañana que no planificaste. De cualquier modo el rango en `package.json` es solo la mitad de la historia: el lockfile registra la resolución exacta que produjo de verdad tu instalación, y es el pin de verdad en los dos mundos. Ese hilo, y qué debería hacer el CI con él, se retoma como corresponde en m09-l1.

### peerDependencies: la cláusula más profunda

Las dependencias regulares dicen "necesito esto, instálamelo." Las peer dependencies dicen algo más extraño y más fuerte: "trabajo al lado de una dependencia que provees TÚ, en ESTE rango." Una librería que declara peer con `@solana/kit` te está diciendo que va a llamar a las APIs de kit en tiempo de ejecución pero se niega a ser dueña de qué instancia de kit existe en tu app, porque tiene que haber exactamente una y tiene que ser la tuya.

¿Por qué negarse a ser dueño siquiera? Supón que helius-sdk declarara kit como dependencia regular. Tu app instala su propio kit, helius-sdk un segundo kit privado, y cada valor que cruza entre los dos (un objeto rpc, un tipo de dirección) lo construyó una copia y lo inspeccionó la otra; las verificaciones de identidad se desarman con los errores menos útiles del ecosistema, porque el objeto SÍ es válido, solo que de la copia equivocada. La declaración de peer dice "tenemos que compartir la única instancia"; el rango dice qué instancias se probó de verdad compartiendo.

Acá va el artefacto en vivo, sondeado el 2026-09-02, no de memoria:

```bash
npm view helius-sdk version peerDependencies
```

```
3.1.0
{
  '@solana-program/compute-budget': '^0.15.0',
  '@solana-program/stake': '^0.6.1',
  '@solana-program/system': '^0.12.0',
  '@solana-program/token': '^0.13.0',
  '@solana/kit': '^6.9.0'
}
```

Mientras tanto el latest de `@solana/kit` está en 8.2.0, dos majors adelante de ese rango de peer `^6.9.0`. Instala `helius-sdk` y después pide kit latest en el mismo paquete, y el resolvedor de npm se niega con un error `ERESOLVE`. El lab dispara esto a propósito, porque la negativa protege algo real: helius-sdk llama a APIs de kit desde la línea 6.x contra la que se probó, y forzar kit 8 al lado solo corre una librería contra una superficie de API que nunca vio, moviendo la falla del tiempo de instalación (ruidoso) al tiempo de ejecución (silencioso, en producción).

![Un diagrama de flujo traza a npm verificando una versión de kit pedida contra un rango de peer instalado y bifurcándose hacia una negativa segura o una instalación forzada riesgosa.](assets/v05-flowchart.webp)

¿Entonces qué fijas de verdad? No lo más nuevo de todo, y no un dígito que alguien memorizó. La regla, y es la oración más durable de todo el curso: **fija lo que tus deps declaran peer, por workspace.** Lee los rangos de peer de tus dependencias, y dale a cada workspace la versión en la que esos rangos coinciden. Por workspace importa porque la flota, el panel y un bot futuro son paquetes separados con conjuntos de dependencias separados; un solo dígito para todo el repo es cómo fabricas un conflicto que ningún paquete individual tiene.

¿Por qué una regla en vez de un número? Porque los dígitos se pudren en una escala de tiempo que prueban los propios timestamps de npm: kit entregó 6.10.0 el 2026-06-16, 7.0.0 el 2026-06-30, 8.0.0 el 2026-08-21. Cualquier wiki que congeló "usa kit 6" estuvo equivocada dos veces antes de que cambiara la estación. Los rangos de peer en tu `node_modules` de verdad son el único consejo de versión que se actualiza solo. La lección dos de M8 construye el workspace de Solana donde esta regla se vuelve el paso de setup.

![Marcadores de release muestran las majors siete y ocho de kit aterrizando con semanas de diferencia mientras el rango de peer de una librería se queda anclado a la línea seis debajo de ellas.](assets/v06-chart.webp)

### Lee un manifest del mundo real antes de escribir el tuyo

La habilidad que esta lección de verdad está instalando es alfabetización de manifests, así que cierra la teoría leyendo uno real, solo con los ojos. Levanta cualquier repo serio de Solana en TypeScript (kit es el que este curso sigue citando) y lee su `package.json` haciendo una pregunta por campo: ¿qué promete esta línea, y a quién? Vas a encontrar el pin de `packageManager` (los cinco repos relevados cargan uno), un piso de engines más viejo que tu Node (dato de audiencia, no descuido), una guarda de preinstall rechazando el gestor de paquetes equivocado, y mapas de `exports` con entradas condicionales por entorno (profundidad dejada como bookmark, no enseñada hoy). Dos minutos de esto por repo desconocido: el repo te dice sus reglas antes de que corras un comando adentro.

**Profundiza (el 20%).** esta lección enseñó los campos que ejercita nuestro camino de entrega y el porqué de cada uno. El resto de la superficie de npm, la semántica de scripts, la precedencia de config, las flags de publicación, los dist-tags, los casos borde del protocolo de workspace, es material real que deliberadamente dejamos como bookmark en vez de re-enseñar. El camino canónico es lo que el track de aprendizaje de Node.js enseña sobre gestores de paquetes: [An introduction to the npm package manager](https://nodejs.org/learn/getting-started/an-introduction-to-the-npm-package-manager) (URL sondeada el 2026-09-02). Léelo después del lab, no en vez de él; nada de abajo depende de él.

## Lab: extrae pulse-core

El repliegue de la ayuda, en voz alta: los pasos 1 al 4 están completamente trabajados, con diffs en pantalla, porque el primer movimiento de extracción es la receta. Los pasos 5 y 6 te pasan la misma receta sin narrar para los módulos restantes; el paso 7 recablea el pipeline de CI como un diff trabajado, porque romper el latido de la estación no es un lugar para practicar. La rep de diagnóstico del paso 8 y el challenge después de eso son enteramente tuyos. Para m03-l4 vas a hacer este baile sin la partitura.

1. **Instala pnpm, fijado.** Primera herramienta de la lección, así que acá va su instalación (frescura: `latest` era 11.25.0 el 2026-09-02; vuelve a chequear con `npm view pnpm version`):

   ```bash
   npm i -g pnpm@11.25.0
   pnpm --version
   ```

2. **Declara el workspace y mueve la flota adentro.** Desde la raíz del repo, crea el layout y reubica todo lo que tenga forma de flota adentro de `packages/pulse-fleet` (tus nombres de archivo pueden diferir; mueve lo que tengas):

   ```bash
   mkdir -p packages/pulse-fleet packages/pulse-core/src
   git mv src tests package.json tsconfig.json pulse.config.json probe.ts fleet.ts smoke.ts packages/pulse-fleet/
   ```

   Tres notas sobre el movimiento:

   - `tests` está en la lista: la suite de m02-l4 importa `../src/config.js` y lee `./fixtures/`, así que tiene que quedarse como hermano de `src/` o el checkpoint de abajo corre cero pruebas.
   - Los tres archivos `.ts` sueltos son los scripts a nivel raíz de la flota. Todo lo que tenga forma de flota se mueve; `status.json` y `.github/` se quedan en la raíz a propósito.
   - Si un archivo listado no existe en tu repo, sácalo del comando en vez de dejar que `git mv` rechace el batch entero.

   Después abre el `packages/pulse-fleet/package.json` movido y haz dos ediciones:

   1. Pon `"name"` en `"pulse-fleet"`. El manifest raíz de abajo está por reusar el nombre viejo para el pegamento privado, y pnpm indexa todo por el campo name, no por el directorio: los prefijos de `pnpm -r`, los selectores `--filter` (el paso 7 necesita uno), y el build-skip de Vercel de m03-l3 quieren todos un nombre único por paquete.
   2. Reemplaza el stub de npm-init en `scripts` por `"test": "vitest run"`. m02-l4 corría la suite como `npx vitest run` y nunca necesitó el script; `pnpm -r` abajo corre el script `test` de cada paquete, y sin esta línea correría el stub, que imprime `Error: no test specified` y sale con 1.

   Crea `pnpm-workspace.yaml` en la raíz:

   ```yaml
   packages:
     - "packages/*"
   ```

   Y un `package.json` raíz mínimo (la raíz hereda el nombre viejo a nivel repo, `pulse-station`, liberado por el renombre que acabas de hacer; es pegamento privado, nunca publicado):

   ```json
   {
     "name": "pulse-station",
     "private": true,
     "packageManager": "pnpm@11.25.0"
   }
   ```

   Tu directorio `.github/workflows` se queda en la raíz, porque Actions solo lee workflows de ahí. Pero fíjate qué le hizo el movimiento al pipeline: cada job en `pulse.yml` sigue corriendo `npm ci` contra una raíz cuyo `package.json` ahora es pegamento privado sin lockfile. Empuja ahora mismo y los tres jobs se ponen en rojo; eso es lo esperado, y el paso 7 lo recablea antes de que se empuje nada. Ya que estás acá, borra el `package-lock.json` desactualizado de la raíz: pnpm escribe su propio `pnpm-lock.yaml` en la próxima instalación, y ese archivo (commitéalo) es, de ahora en más, el pin de verdad del workspace. Checkpoint: `pnpm install` desde la raíz completa y `pnpm -r test` corre la suite de m02-l4 en verde desde su casa nueva. Un freno probable: pnpm 11 se niega a correr scripts de build de dependencias que no le dijeron que confíe, así que la primera instalación puede abortar con `ERR_PNPM_IGNORED_BUILDS` nombrando `esbuild` (el motor de vitest, que compila un binario nativo al instalar). El arreglo es un comando, `pnpm approve-builds esbuild`, que anota una entrada `allowBuilds` en `pnpm-workspace.yaml`; commitea ese archivo y vuelve a correr la instalación. (Docs más viejos mencionan una clave `onlyBuiltDependencies`; pnpm 11.25 la ignora en silencio, así que usa el comando.) Nada está extraído todavía; solo demostramos que el movimiento no rompió nada antes de cambiar cualquier otra cosa.

3. **Crea pulse-core y mueve el primer módulo del motor.** El clasificador y su unión van primero, porque son el módulo que importa la lección que viene. Si seguiste la consolidación de m02-l4, todo eso vive en un solo archivo, `src/classify.ts`: la unión `ProbeResult`, `parseProbe`, el `classify` con forma de unión, el `classifyProbe` con forma de frontera, y `assertNever`. (El `probe.ts` de la raíz es el envoltorio de CLI alrededor de ellos; es cableado de app y ya se movió con la flota en el paso 2.) Un archivo, un movimiento:

   ```bash
   git mv packages/pulse-fleet/src/classify.ts packages/pulse-core/src/
   ```

   Dale a `pulse-core` su contrato, cada campo de la sección de teoría llenado con honestidad:

   ```json
   {
     "name": "pulse-core",
     "version": "0.1.0",
     "private": false,
     "type": "module",
     "exports": {
       ".": "./src/index.ts"
     },
     "engines": {
       "node": ">=24"
     },
     "packageManager": "pnpm@11.25.0"
   }
   ```

   Y la superficie pública curada, `packages/pulse-core/src/index.ts`:

   ```ts
   export type { ProbeResult, Verdict } from "./classify.js";
   export { classify, classifyProbe, parseProbe, assertNever } from "./classify.js";
   ```

   Ese `.js` en los especificadores no es un typo y no es opcional: bajo la resolución `nodenext` que corren los tsconfig de este curso, los imports relativos de ESM tienen que nombrar la extensión emitida, exactamente como ya hacía cada archivo de m02. Escribe `"./classify"` pelado y vitest igual va a correr contento (su bundler resuelve más suelto), lo que vuelve el error extra traicionero: la primera herramienta en rechazarlo es `tsc --noEmit`, con un TS2835 por import, en la barrera de CI del paso 7, a dos pasos del archivo donde te equivocaste al escribir.

![Cada campo del manifest de pulse-core carga una nota al margen explicando la promesa que esa línea les hace a las herramientas y a los consumidores.](assets/v07-annotated-code.webp)

4. **Cablea la flota para que importe cruzando la frontera.** En `packages/pulse-fleet/package.json`, agrega la dependencia de workspace:

   ```json
   {
     "dependencies": {
       "pulse-core": "workspace:*"
     }
   }
   ```

   Después corre `pnpm install` desde la raíz para crear el symlink, y actualiza cada import de la flota del módulo movido. La CLI de sonda en `packages/pulse-fleet/probe.ts`:

   ```ts
   // before
   import { classify, type ProbeResult } from "./src/classify.js";

   // after
   import { classify, type ProbeResult } from "pulse-core";
   ```

   Fíjate en el efecto secundario de la frontera sobre la grafía: los imports relativos de tus propios archivos necesitan la extensión `.js`, pero un especificador de paquete pelado nunca carga una; el mapa de exports la resuelve. Tus archivos de prueba cambian esa misma única línea y nada más (`../src/classify.js` se vuelve `pulse-core`), así que la suite de m02-l4 ahora ejercita `pulse-core` a través de su frontera pública, exactamente como lo va a hacer el panel de la lección que viene. Checkpoint: `pnpm -r test` en verde de nuevo, y la salida te enseña cómo piensa `-r`: el script `test` de cada paquete del workspace, con la salida prefijada con el nombre del paquete, con esta forma:

   ```
   Scope: all 2 workspace projects
   packages/pulse-fleet test$ vitest run
   ...
   Test Files  3 passed (3)
        Tests  14 passed (14)
   ```

   Tus conteos van a coincidir con lo que sea que creció tu suite de m02-l4; la forma es lo que hay que reconocer. Solo `pulse-fleet` corre pruebas porque solo él tiene un script `test`, y eso está bien hoy: la suite cruza la frontera, así que el core se ejercita, y cuando a `pulse-core` le crezca su propio script `-r` lo levanta con cero config. Si TypeScript no puede resolver `pulse-core`, te salteaste el `pnpm install` de la raíz que crea el link; si resuelve pero se queja del punto de entrada, tu path de `exports` no coincide con dónde se sienta `index.ts`.

5. **Mueve tú mismo los módulos restantes del motor.** Los helpers de backoff de m02-l3 van en el core (el edge worker de M7 los va a querer; la config de zod no se mueve, porque parsear config es cableado de app, no motor). La misma receta que los pasos 3 y 4: `git mv` al archivo, re-exporta las piezas públicas desde `index.ts`, actualiza los imports de la flota, `pnpm -r test`. Esta vez no hay diff provisto; tienes el patrón.

![Un loop de cinco pasos mueve un módulo, lo re-exporta, recablea imports, reinstala, y prueba, repitiendo por módulo hasta que el código del motor viva solo en el paquete core.](assets/v08-flowchart.webp)

6. **Demuestra que la frontera prohíbe la puerta de atrás.** Desde cualquier archivo de la flota, prueba el import profundo que el mapa de exports existe para matar, `import { classifyProbe } from "pulse-core/src/classify"`, y corre las pruebas. La negativa usa dos disfraces: a través de vitest recibes la redacción de Vite, `"./src/classify" is not exported under the conditions ["node", "development", "import"]`, mientras que la resolución pelada de Node (corre el import por `npx tsx` y mira) lanza el canónico `ERR_PACKAGE_PATH_NOT_EXPORTED`. La misma ley, dos tribunales; reconoce las dos grafías, después borra la línea. Aceptación para la extracción: `pnpm -r test` en verde desde la raíz, y `grep -r "classifyProbe" packages/pulse-fleet/src` muestra solo líneas de import, cero cuerpos de función. El código del motor vive en exactamente un solo lugar.

7. **Recablea el pipeline (el tercer consumidor del workspace).** La advertencia del paso 2 vence: los jobs de `pulse.yml` todavía instalan con `npm ci` y corren sus herramientas desde una raíz que ya no guarda la flota. Enséñale al workflow el layout que acabas de enseñarte a ti mismo. Los tres jobs cambian `npm ci` por un par de líneas; el job probe, el único que carga una línea `cache: npm`, borra esa también:

   ```yaml
         - uses: actions/setup-node@v7
           with:
             node-version: 24              # CHANGED (probe job): cache: npm line deleted
         - run: npm i -g pnpm@11.25.0        # CHANGED: was `npm ci`
         - run: pnpm install --frozen-lockfile
   ```

   La eliminación del cache no es limpieza opcional: `cache: npm` hace que setup-node vaya a buscar `package-lock.json`, el archivo que el paso 2 borró deliberadamente, y en un runner de verdad falla duro con `Dependencies lock file is not found` antes de que se ejecute el primer step `run:` de ese job. Grepea tu propio `pulse.yml` antes de editar: esa línea existe en exactamente un lugar, el job probe que escribió m01-l3, porque a los jobs de typecheck y test que agregó m02-l4 nunca se la dieron. Y fíjate cuándo te llegaría la falla, porque `needs: [typecheck, test]` decide eso: el job probe no arranca hasta que las dos barreras se ponen en verde, así que esta aparece como una ejecución en rojo cuyos primeros dos jobs pasaron, varios minutos adentro. (`cache: pnpm` existe, pero setup-node le pregunta a pnpm por el path de su store, así que solo funciona si pnpm está instalado ANTES de setup-node; omitir la línea es el mínimo honesto hoy.) Después apunta cada job al workspace: la barrera de typecheck se vuelve `- run: pnpm --filter pulse-fleet exec tsc --noEmit`, la barrera de test `- run: pnpm -r test` (el comando exacto que vienes corriendo localmente), y los steps de run del job probe ganan `working-directory: packages/pulse-fleet`, con un contrato deliberadamente sin cambiar: `status.json` se queda en la RAÍZ DEL REPO. Ese path es estructural, porque el panel de m03-l2 y la config de m03-l3 los dos traen el archivo crudo en `.../main/status.json`, así que apunta la escritura de la flota a la raíz (escribe a `../../status.json`, o toma el path de salida como argumento) y mantén el step de commit agregando `status.json` desde la raíz del repo; si alguna vez aparece una segunda copia adentro de `packages/pulse-fleet/`, la escritura está mal apuntada y el panel renderizaría en silencio la copia congelada de la raíz. `--frozen-lockfile` es el trabajo de `npm ci` en la grafía de pnpm: toma exactamente lo que registró `pnpm-lock.yaml` o falla a gritos. Commitea el cambio del workflow junto con `pnpm-lock.yaml`, empuja, y mira la ejecución. Checkpoint: las dos barreras en verde, el job probe commitea un `status.json` fresco, y las aristas `needs: [typecheck, test]` de m02-l4 sobreviven intactas. El pipeline es el tercer consumidor de la extracción, y como `-r` camina lo que sea que declare el workspace, cada paquete futuro ya está adentro de la barrera.

8. **La rep de diagnóstico (sin guía).** En un directorio borrador, reproduce el conflicto de la sección de teoría con tus propias manos y lee la negativa:

   ```bash
   mkdir peer-scratch && cd peer-scratch
   npm init -y
   npm i helius-sdk
   npm i @solana/kit@latest
   ```

   La primera instalación tiene éxito (npm auto-instala los peers declarados, todos de las líneas compatibles con 6.x). La segunda se niega, más ruidosa que la salida prolija de `npm view` en la sección de teoría: espera primero unas treinta líneas de `npm warn ERESOLVE overriding peer dependency` (la línea de helius que ya conoces, `peer @solana/kit@"^6.9.0" from helius-sdk@3.1.0`, pasa de largo entre ellas), y después el bloque que importa, que abre con `npm error code ERESOLVE`. Nombra el conflicto a través de los peers TRANSITIVOS de helius-sdk, líneas con forma de `peer @solana/kit@"^6.4.0" from @solana-program/system@0.12.2`, y cierra con `Conflicting peer dependency: @solana/kit@6.10.0`, el kit más nuevo en el que puede coincidir todo el árbol: los paquetes `@solana-program/*` con los que helius-sdk declara peer cargan sus propios rangos de kit, y cualquiera de ellos alcanza para rechazar kit 8. Tu entregable es UNA oración escrita que diagnostique el arreglo en términos de la regla del pin. Tiene que nombrar el rango y la regla; un dígito de versión pelado como respuesta va a estar desactualizado antes de que termine el módulo.

## Challenge

La lógica de semver que acabas de usar a ojo se vuelve código: implementa `satisfiesRange(version, range)` para las cuatro formas de rango que enseñó esta lección, exacto, `>=`, tilde y caret, incluyendo la regla del caret cero donde la major 0 vuelve a la minor el slot de ruptura. `parseSemver` y `compare` vienen provistos en el starter, en el coding-challenge panel de la página de esta lección; el brazo de coincidencia exacta está hecho por ti. Once pruebas lo califican, una de ellas el lab de esta lección en miniatura: ¿`7.0.2` satisface `^6.9.0`? Tu implementación debería coincidir con el resolvedor de npm en esa llamada: no lo hace. El ejercicio no es todo npm: la regla del caret para en el piso de major cero; la sub-regla `^0.0.z`, donde el patch se vuelve el slot de ruptura, no está modelada ni probada acá, así que no trates el ejercicio como una reimplementación completa del matcher de npm. Las pistas escalan de ordenar operadores a la rama del caret cero; gástalas en orden.

Una nota de diseño antes de empezar, la primera pista disfrazada: el ORDEN en el que pruebas los operadores es estructural. Verifica `>=` antes que cualquier cosa de un solo carácter, o vas a cortar el prefijo equivocado y cada prueba de piso falla de una, un bug de cadenas vestido de bug de lógica. Después de este challenge, un caret en cualquier manifest es algo que computas, no algo que mires entrecerrando los ojos: la diferencia entre leer un conflicto de peers y que uno te lea a ti.

## Checkpoint, y el primer consumidor de afuera

Lo que ya puedes hacer: leer cualquier `package.json` y decir qué promete cada campo y a quién; correr un workspace de pnpm de dos paquetes donde una suite en verde demuestra que la estructura cambió y el comportamiento no; diagnosticar un conflicto de peers leyendo rangos en vez de pasarles por encima a la fuerza con una flag. La recuperación de 30 segundos, en voz alta: ¿qué admite `^6.9.0`, y dónde para? (Cualquier 6.x en o por encima de 6.9.0. Nunca 7. Con una base `^0.9.0`, la parada se mueve a la minor.)

Dos pedidos mientras está fresco: si un paso del lab te peleó, anota DÓNDE te peleó (¿el symlink? ¿el path de exports? ¿el bloque ERESOLVE?) en tus notas del curso, y si la oración del conflicto de peers te tomó más de un intento, guarda tus borradores fallidos; M8 te va a mostrar el mismo diagnóstico con más en juego y tu redacción vieja es evidencia útil de cómo mejoró tu modelo.

`pulse-core` ahora es un paquete real con una frontera real, y la frontera recibe su primera prueba de afuera inmediatamente: la lección que viene un panel de React importa el clasificador cruzándola y renderiza el `status.json` del cron para ojos humanos, lo que quiere decir que la extracción que acabas de hacer deja de ser un argumento y se vuelve un pixel. Nos vemos en el render.
