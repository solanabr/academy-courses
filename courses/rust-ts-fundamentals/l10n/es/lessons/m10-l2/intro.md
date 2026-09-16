# Conclusión: el mapa que ahora es tuyo

La lección pasada armaste la estación entera y lo probaste: el script de demo pasó de punta a punta, la extensión en solitario se entregó sin apoyo en ningún lado cerca, y el README y el runbook significan que otro dev podría operar lo que construiste sin ti en la sala. No queda nada por construir. Así que esta lección abre como abrió el curso: haciéndote medir algo. Esta vez, ese algo eres tú.

Abre una pestaña nueva, ve a https://rust-lang.org (apex, sin www, igual que en la lección uno), aprieta F12, haz clic en la pestaña Console y pega el snippet exacto de la lección uno:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

Después ve a https://www.typescriptlang.org/play/ (con slash final, igual que siempre) y pega el segundo:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeout;
console.log(`waiting ${wait} ms`);
```

Las mismas pocas líneas. El mismo subrayado rojo debajo de `timeout`. El código no cambió ni un carácter desde el módulo uno. Ahora haz la parte que importa: agarra cualquier cosa sobre la que puedas escribir y anota cinco líneas sobre lo que ves AHORA y no podías ver entonces. No las pulas. Las mías, corriéndolo de nuevo mientras escribo esto: la forma de la respuesta es una unión que modelaría como variantes ok o error antes de tocarla; este fetch no tiene timeout, ni retry, ni backoff, y sé exactamente cuál de mis propias funciones arregla eso; `performance.now` es un reloj monótono y sé por qué eso importa para medir; este código corre en un solo lugar y yo entregué la misma sonda a cuatro; y el subrayado no es un linter siendo quisquilloso, es una demostración sobre mi programa que ahora diseño a propósito. El código no cambió. Tú sí. El resto de esta lección es el mapa de cuánto, exactamente, y de a dónde llevan los caminos de acá en adelante.

## Resumen

Esta es la conclusión, y no enseña nada nuevo a propósito. Hace cuatro cosas. Repasa la construcción peldaño por peldaño, una oración honesta cada uno, que es recuperación espaciada disfrazada de vuelta de honor. Reimprime la tabla de enseñado-contra-bookmark de la lección uno, cerrando la promesa que hizo esa lección, y después la reordena por urgencia según la ruta que elijas. Lee tu nivel de salida contra la oración de prerrequisito que cada curso hermano declara, con evidencia en vez de sensaciones. Y entrega la última habilidad: el hábito de reverificar cada número que este curso imprimió y que se va a podrir. El lab produce tres artefactos chicos y ninguno es pasivo. Después, la puerta.

## El mapa, cerrado

### La construcción, peldaño por peldaño

Esto es lo que hiciste de verdad, módulo por módulo. Léelo despacio; cada línea es algo que puedes abrir en tu propio GitHub ahora mismo.

R0, módulo uno: `pulse` v0, una sonda en TypeScript estricto que hacía fetch a una URL e imprimía su latencia, promovida a un cron de GitHub Actions que hace commit de `status.json` de vuelta a un repo público. Tu primera entrega fue un latido en una máquina que no es tuya. R1, módulo dos: la flota ganó tipos, una unión discriminada para resultados de sondas, validación con zod en la frontera de config, un pool de concurrencia hecho a mano con cancelación real, y una suite de vitest que le pone una barrera al cron. R2 y R3, módulo tres: el motor se extrajo a `pulse-core` dentro de un workspace de pnpm, un panel en React salió en vivo en una URL de Vercel que cualquier desconocido puede abrir, y el paquete se publicó en npm donde cualquiera puede instalarlo. R4, módulo cuatro: el motor se reescribió en Rust, la unión se volvió un enum, los errores se volvieron `Result` con thiserror, la máquina de estados ganó tests de transición, y clippy más fmt se sumaron a la barrera de CI. R5, módulo cinco: serde leyó el mismo archivo de config que lee la flota de TypeScript, el workspace se dividió en crates, una CLI con clap le puso frente, y CI produjo un binario de release que una máquina limpia puede correr. R6, módulo seis: el poller se mudó a una imagen Docker multi-stage, un orden de magnitud más chica que la ingenua, publicada en GHCR, con compose corriendo la estación local. R7, módulo siete: las sondas llegaron al edge en Cloudflare Workers dos veces, una en TypeScript y una en Rust compilado a WebAssembly, ambas guardando el último estado conocido en KV. R8, módulo ocho: Solana se sumó a la lista de targets, lecturas con kit en el panel, sondas de la blockchain en el poller, y una transacción de devnet real confirmada con tu propia clave desechable. R9, módulo nueve: auditorías sobre todo el árbol, logs estructurados que puedes grepear durante un incidente, y una alarma sobre el monitor mismo. R10, la lección pasada: la estación entera, verificada arista por arista desde una sola máquina, con transcripción guardada y con un runbook.

![Una escalera sube desde una sola sonda agendada, pasando por flotas tipadas, motores en Rust, contenedores, edge workers y targets de Solana, hasta una estación armada.](assets/v01-timeline.webp)

Fíjate en lo que la escalera no es: no son diez proyectos. Es un solo artefacto que nunca se tiró a la basura. El one-liner que volviste a correr hace diez minutos sigue estando reconociblemente adentro del binario de release, de los workers y del contenedor. Ese es el argumento de acumulación al que este curso le apostó, y tú eres la evidencia de que funcionó.

La forma de la estación, una última vez, porque la lección uno te prometió que te ibas a volver a encontrar con este dibujo y la lección pasada te hizo redibujarlo de memoria. Dos notas honestas antes de que lo pongas frente a la hoja de respuestas de la lección pasada, porque los dos dibujos deliberadamente no son la misma imagen. Esta reimpresión es el dibujo del ABRIDOR DEL CURSO, centrado en el repo: el repo público en el hub, cuatro radios, dibujado antes de que existiera el módulo ocho, así que no hay radio `tx-check`. La referencia operativa de m10-l1 redibujó el mismo sistema centrado en el pipeline: Actions en el hub, los workers separados en dos radios, `tx-check` contado, cinco radios en total. Mismos componentes, mismas aristas reales, dos centros de gravedad, y los dos son ciertos de un solo sistema: el abridor pregunta dónde viven los datos (el repo), el runbook pregunta qué late (el pipeline). Ver que los dos dibujos describen tu única estación, y saber cuál agarrar según la pregunta, ya es de por sí una habilidad de nivel graduación; si quieres que este coincida con el del runbook, agregar el radio `tx-check` es el único delta.

![Un sistema de hub y radios se centra en un repositorio público, con cada radio ya construido, verificado y marcado como completo alrededor de targets de sondeo reales.](assets/v02-diagram.webp)

### La tabla, reimpresa

La lección uno imprimió una tabla de lo que este curso enseña contra lo que deja como bookmark y la llamó el gemelo honesto del programa del curso. También prometió que el módulo final te iba a traer de vuelta a ella. Esta es esa visita, y la costura 80/20 es contenido del curso hasta el final. Es una reimpresión con un solo delta honesto, señalado en vez de colado: la fila de concurrencia sin miedo de más abajo es nueva de hoy, agregada a la salida porque diez módulos de práctica de ownership finalmente le ganaron un lugar; la tabla de la lección uno nunca la traía. Los links de abajo son los mismos recursos canónicos, verificados en vivo durante la pasada de investigación de este curso y vueltos a sondear el 2026-09-02, la fecha que imprime la lección uno. Uno de ellos es además un chiste chico a costa del curso: a mediados de junio de 2026, mientras se armaba nuestra investigación, Frontend Masters pasó a ser Master.dev y todas las URLs viejas empezaron a redirigir. Un rebrand aterrizó adentro de nuestra propia lista de links y demostró, en vivo, por qué el hábito de reverificar aplica también a los bookmarks. Los links se pudren. Los links verificados se pudren con retraso.

| Concepto | Dónde vivió | Bookmark a |
|---|---|---|
| Fundamentos de JS | Nunca se enseñó; es el prerrequisito | MDN Learn Core Scripting: https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting |
| Config estricta de TypeScript | M1 | TS Handbook intro: https://www.typescriptlang.org/docs/handbook/intro.html y Everyday Types: https://www.typescriptlang.org/docs/handbook/2/everyday-types.html |
| Uniones discriminadas, narrowing, exhaustividad | M2 | Handbook narrowing, sección de uniones: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions ; el repo type-challenges en GitHub como ejercicio |
| Genéricos que de verdad usas | M2 | Handbook generics: https://www.typescriptlang.org/docs/handbook/2/generics.html |
| Validación en las fronteras | M2 | El tutorial gratuito de Zod de Total TypeScript: https://www.totaltypescript.com/tutorials |
| Async, concurrencia, cancelación | M2 | La unidad de async de MDN; TRPL ch17 para el lado de Rust: https://doc.rust-lang.org/book/ch17-00-async-await.html |
| Práctica de testing | M2 (vitest), M4 (cargo test) | vitest guide; node:test docs |
| Alfabetización en empaquetar y publicar | M3 | El learn path de Node: https://nodejs.org/learn |
| React, nivel consumidor | M3 | react.dev quick start; la profundidad de cliente le pertenece al curso hermano del lado del cliente (en producción), leído contra la sección de puertas de más abajo |
| Ownership y borrowing | M4 | El fork interactivo de Brown, quizzes y visualizaciones de ownership: https://rust-book.cs.brown.edu/ |
| Errores como valores | M4 | El capítulo de manejo de errores del Rust Book: https://doc.rust-lang.org/book/ch09-00-error-handling.html |
| Enums, structs, traits en la práctica | M4 | El capítulo de enums del Book: https://doc.rust-lang.org/book/ch06-00-enums.html y generics and traits: https://doc.rust-lang.org/book/ch10-00-generics.html |
| serde, cargo, clap, una CLI de verdad | M5 | serde.rs; el Cargo book; Rustlings después de cada módulo de Rust: https://rustlings.rust-lang.org/ |
| tokio, profundidad de saber-cuándo-lo-necesitas | M6 | TRPL ch17 async, el mismo capítulo que arriba: https://doc.rust-lang.org/book/ch17-00-async-await.html |
| Concurrencia sin miedo, threads de verdad | Nunca se enseñó; fila agregada hoy | El ch16 del Book, fearless concurrency, desde la tabla de contenidos en https://doc.rust-lang.org/book/ |
| Amplitud de sintaxis de Rust | Nunca se volvió a enseñar | Rust by Example: https://doc.rust-lang.org/rust-by-example/ ; Comprehensive Rust: https://google.github.io/comprehensive-rust/ |
| Lifetimes más allá de leerlos, unsafe, macros | Nunca, señalizado | The Rustonomicon, que abre diciéndote que todavía no lo leas: https://doc.rust-lang.org/nomicon/ |
| Contenedores | M6 | Docker Get Started: https://docs.docker.com/get-started/ |
| Desplegar la app de TS | M3 | Vercel getting started: https://vercel.com/docs/getting-started-with-vercel |
| El edge, los dos lenguajes | M7 | Cloudflare Workers get started: https://developers.cloudflare.com/workers/get-started/guide/ |
| Profundidad de Solana, Anchor, UX de wallet | Modelo mínimo de cliente, M8 | Cursos hermanos de este catálogo, leídos contra sus propios letreros de más abajo |

En la lección uno esta tabla era una promesa. Hoy es un orden de lectura, y los órdenes de lectura son personales. Así que reordénala. Cuáles bookmarks se volvieron urgentes recién depende por completo de a dónde vas, pero tres ascensos valen para casi todo el que sale de este curso.

Primero: TRPL ch16, fearless concurrency. En el momento en que tu próximo servicio en Rust necesite threads de verdad en vez de tasks de tokio, ese capítulo deja de ser opcional. Ya tienes todo lo que asume, ownership incluido, que es justamente por lo que puede entrar hoy al mapa como bookmark y no como capítulo. Segundo: lifetimes más allá de leerlos. Si tu ruta es escribir programas, lee la relación real que el curso de Anchor tiene con ellos antes de ponerte a estudiar en pánico: sus labs de V2 quitan las viejas anotaciones `<'info>` de los structs de cuentas en vez de exigir nuevas, y lo que sus checkpoints sí exigen es razonar sobre borrows, seguir el argumento del compilador sobre cuál handle está vivo y cuándo. Su costumbre de rampas de entrada justo a tiempo cubre traits y tipos marcadores, no lifetimes, así que el capítulo profundo de lifetimes sigue siendo tu bookmark, y vence cuando lo necesitas, no antes. Tercero: type-challenges, el gimnasio de TS. Ahora escribes uniones discriminadas a diario; los ejercicios convierten eso de un patrón que usas en un músculo que es tuyo. Elige tus tres. Ata cada uno a una ruta. Ignora el resto hasta que venzan, porque un acopio de bookmarks es solo la trampa de las mil horas con mejor organización.

![Los mismos bookmarks aparecen primero como una lista plana de promesas, y después reordenados en niveles de urgencia que un graduado asigna según la ruta que eligió.](assets/v03-comparison.webp)

### Leer las puertas contra sus propios letreros

Ahora la parte que me niego a tratar a la ligera, porque sobrevender tu salida le quitaría el mérito a todo lo que las cajas de honestidad construyeron desde la lección uno. Cada curso hermano de este catálogo declara su propio prerrequisito. La forma correcta de irte de acá es leer esas oraciones como ahora lees un Cargo.toml: como afirmaciones que hay que verificar contra evidencia que es tuya. Así que hagamos exactamente eso, puerta por puerta.

El curso de dominio del lado del cliente, la pista avanzada de este catálogo para wallets, aterrizaje de transacciones y datos en el frontend, está en producción mientras escribo esto, lo que significa que su letrero todavía no está impreso y no voy a inventar uno para citarlo. Lo que puedes poner frente a cualquier vara de pista de cliente son recibos, no adjetivos: una flota de sondas tipada con un núcleo de unión discriminada, fronteras con zod que fallan fuerte al arrancar, un pool de concurrencia con cancelación que escribiste a mano y después probaste con timers falsos, y el panel de React que entregaste en Vercel en el módulo tres y volviste a entregar con paneles nuevos dos veces después de eso. Si tu meta es la mitad de cliente de Solana, ese curso es la puerta; hasta que se entregue, la puerta es el tema mismo, UX de wallet y aterrizaje de transacciones, y llegas cargando evidencia.

El curso de Anchor, la pista de framework para escribir programas on-chain, lleva el letrero más franco del catálogo, y leerlo como está escrito es el momento de integridad de esta lección: asume que ya entregas programas de Anchor y quieres el delta de V2. Esa es una vara de entrega, y ningún curso de fundamentos supera una vara de entrega por sí solo. Lo que tienes desde acá es real y parcial: escribes structs, enums, traits, `Result` con thiserror y serde a diario, y lees Rust idiomático lo bastante bien como para revisarlo, que es el nivel de lectura que asume la prosa de ese curso. Lo que te falta, se parte en dos: cuentas, PDAs y transacciones como conceptos, que viven en el curso de evolución de Bitcoin a Solana, el curso de conceptos que recorre cómo funciona el modelo de datos de una blockchain y por qué el de Solana tiene la forma que tiene; y reps de programas entregados, que ningún curso te regala. Así que la ruta honesta para escribir programas pasa primero por el curso de conceptos, después por primeros programas escritos y entregados, y solo después por la puerta de Anchor V2. La confianza en Rust por sí sola no supera una vara de entrega, y pretender lo contrario te manda a labs avanzados sin el modelo ni las reps que asumen.

Lee con cuidado el propio letrero del curso de Bitcoin a Solana, porque es más angosto de lo que lo pinta el folclore: el curso nunca afirma que no haya código, su descripción promete que vas a "correr, scriptear y explicar" y entregar un bot de ops, y la tranquilidad que de verdad imprime tiene alcance de lección y es solo de Rust: "¿Nunca escribiste una línea de Rust? Bien, no te hace falta." Contra tu evidencia el veredicto sigue siendo cómodo: estás sobrecalificado en todos los pisos de programación que tiene, en Rust más que en ninguno, donde su pedido es cero y tu práctica es diaria. Tómalo por los conceptos, muévete rápido por lo que ya sabes, y trata sus labs como reps rápidas en vez de reps salteadas; que un curso esté por debajo de tu nivel de código no pone sus ideas por debajo de tu nivel de ideas.

![Un diagrama de flujo rutea a un graduado por tres puertas de curso, y muestra una vara cumplida, una vara de dos partes con un desvío por conceptos, y un camino de conceptos abierto.](assets/v04-flowchart.webp)

Dos puertas más del catálogo merecen que se lean sus letreros, porque la estación mapea sobre ellas de manera despareja y decir cómo es para lo que existe esta sección. El curso de pagos y comercio en Solana es la puerta siguiente que mejor calza con el conjunto de habilidades de la estación: su medio de back-office, un servicio de checkout, manejo de webhooks, un worker de liquidación con retries y backoff, corre sobre exactamente los músculos que este curso ejercitó, un servicio de TS tipado con parseo en la frontera, un loop de worker agendado, backoff con jitter y disciplina de retry en el camino de lectura, así que llegarías ahí gastando tu atención en semántica de pagos en vez de en plomería. El curso de activos digitales es una puerta real con un letrero más alto, y la lectura honesta es un orden de lectura y no un muro: la mitad de PDAs de su vara le pertenece al curso de conceptos, y los conocimientos previos sobre token accounts en los que se apoya su propio primer módulo son algo que este curso conscientemente no dio, así que llegas ahí debiéndolo. m04-l2 gastó la cifra de rent de una ATA como constante trabajada y dijo en el momento que lo que una ATA ES de verdad le pertenece a ese curso, no a este. No te voy a inventar una ruta: nada acá llena ese hueco y no revisé cada programa hermano buscando uno, así que trátalo como una noche de lectura que agendas antes de su módulo uno y no durante.

Antes de la última puerta, la parte del mapa que hace confiable a todo el resto: lo que todavía NO eres. No eres autor de programas; ese camino pasa por conceptos que este curso nunca enseñó y por reps de programas entregados que nunca asignó, antes incluso de que se abra la puerta del curso de Anchor. No eres un desarrollador de sistemas en Rust con fluidez en lifetimes; lees lifetimes cuando el compilador los imprime, y el material profundo está en bookmark, no absorbido. No eres ingeniero de UX de wallet ni de aterrizaje de transacciones; entregaste un panel que lee una blockchain, que es una cosa distinta de llevar de la mano la transacción de un usuario hasta un bloque, y el curso del lado del cliente existe porque esa diferencia es una disciplina entera. Escribir esas tres oraciones me cuesta algo, porque todo curso quiere afirmar que sus graduados pueden hacer todo. Pero el valor de un mapa ESTÁ en sus bordes. Un mapa de salida sin columna de todavía-no es un aviso publicitario, y pasaste diez módulos aprendiendo a desconfiar de esos.

Una puerta más merece su mención con nombre, y una declaración repetida de la lección uno. El 2023-04-25, ThePrimeagen entregó Rust for TypeScript Developers en lo que ahora es Master.dev, 5 horas 19 minutos, pago con preview gratuito. El mercado le puso nombre a la audiencia exacta de este curso tres años antes de que este curso existiera. Acabas de caminar todo el camino al que apunta ese título, en los dos sentidos. Si quieres una segunda voz sobre la mitad de Rust ahora que ya entregaste con ella, ese sigue siendo el único recurso pago que este curso nombra.

### Los números que se van a podrir, y el hábito que no

Cada dígito de versión que este curso imprimió fue verificado el día en que se escribió, y todos y cada uno se están muriendo en un calendario que nadie publica. Eso no es una falla del curso. Ese es el terreno, y lo último que este curso enseña es el reflejo que lo sobrevive. Tres casos, cada uno con su regla.

Caso uno, el más filoso de todo el archivo de investigación: entre el 2026-06-16 y el 2026-08-21, @solana/kit entregó un minor y después dos majors. 6.10.0 a mediados de junio, 7.0.0 dos semanas después, 8.0.0 a fines de agosto. Poco más de nueve semanas, dos saltos de major. Durante la propia ventana de investigación de este curso, nuestras notas internas quedaron desactualizadas en ese dígito dos veces. Me tocó ver nuestra propia documentación podrirse en tiempo real mientras escribía un curso sobre no confiar en documentación desactualizada, que es más o menos tan humillante como suena. La regla que sobrevive: fija lo que tus dependencias declaran como peer, por workspace, y vuelve a sondear antes de cada instalación nueva. El dígito nunca fue el conocimiento. La sonda es el conocimiento.

![Un minor y dos majors de un mismo paquete aterrizan en poco más de nueve semanas sobre una línea de tiempo de verano, cada marcador fechado desde el registro.](assets/v05-chart.webp)

Caso dos, los runtimes. Este curso fijó Node 24 LTS y te dijo, con una nota al pie fechada, que la antorcha de LTS pasa a v26 el 2026-10-28. Esa fecha sale del calendario de releases publicado de Node, que es justamente el punto: los runtimes se pudren con educación, en calendarios que puedes leer. Así que lee el calendario, no un blog post sobre el calendario. Cuando armas un proyecto en marzo, la pregunta nunca es qué dijo mi curso, es qué dice el calendario hoy.

Caso tres, la blockchain misma. Solana apunta a slots de 300ms y, cuando este curso midió veinte muestras recientes el 2026-09-01, la red promedió 316ms. Las metas son marketing hasta que se miden, y tu estación mide. Ese hábito, correr tu propia sonda en vez de citar el número de alguien, es el mismo reflejo en otra capa, y ya lo tienes metido en una máquina que lo ejercita cada treinta minutos sin ti.

¿Dónde viven los dígitos, entonces? En tu tabla de pins, la que tu runbook viene cargando desde la lección pasada: cada versión de la que depende esta estación, en un solo lugar, cada una con la fecha en que la verificaste. Algunos de los tuyos se van a ver más o menos así, con tus propias fechas en la última columna:

```markdown
| surface        | pin                        | why                              | verified   |
|----------------|----------------------------|----------------------------------|------------|
| Node           | 24 LTS                     | active LTS line; v26 2026-10-28  | 2026-09-02 |
| @solana/kit    | what deps peer against     | probe peers before install       | 2026-09-02 |
| rust toolchain | current stable via rustup  | six-week train; clippy in CI     | 2026-09-02 |
```

La columna del medio es la que hace el trabajo real. Una tabla de pins que solo guarda dígitos es una lista de mentiras futuras; una tabla de pins que guarda la regla al lado de cada dígito es un manual de mantenimiento. Volver a fijar es una edición de una línea y el CI que construiste juzga cada re-fijado gratis. El hábito, dicho una vez, sin adornos, para que se lo puedas repetir a alguien más: los números en sistemas que corren son fotos; guárdalos en un archivo, féchalos, y vuelve a sondear cuando lo necesites en vez de confiar en cualquier dígito congelado, incluidos los de este curso. Sobre todo los de este curso. En un año, desconfía de cada número de versión impreso acá y confía en el método que los produjo.

![Un loop chico corre desde un disparador, pasando por una sonda en vivo y una comparación, y actualiza o vuelve a sellar una tabla de pins fechada en cualquiera de los dos casos.](assets/v06-flowchart.webp)

## Lab: tres artefactos, ninguno pasivo

El apoyo se fue desde la lección pasada; este lab es instrucciones, no pasos que sigues conmigo. Produce tres artefactos escritos chicos. Treinta minutos, nada que instalar, y todo lo que escribes aterriza en el repo de la estación así que se entrega como se entregó todo lo demás.

1. **Termina el diff then-vs-now.** Escribiste cinco líneas en bruto al principio de esta lección. Límpialas hasta dejar cinco de verdad y guárdalas como `docs/then-vs-now.md` en el repo de la estación. La prueba para cada línea: tiene que nombrar algo específico que ahora ves en esos dos snippets, una unión, un backoff que falta, una elección de runtime, un costo, una demostración. "Ahora sé más TypeScript" no pasa la prueba. "La forma de esta respuesta es una unión y la modelaría antes de tocarla" sí pasa.

2. **Escribe la declaración de ruta.** Un curso hermano, nombrado, con una declaración de dos líneas de que cumples con la vara que declara, o de exactamente cómo la vas a cumplir. Evidencia significa apuntar a cosas: el panel entregado, la máquina de estados con enum, el curso de conceptos que vas a tomar primero. Agrégala al mismo archivo. Si no puedes escribir la declaración en dos líneas, todavía no elegiste ruta, y eso vale saberlo hoy en vez de tres semanas dentro del curso equivocado.

3. **Armá la lista de lectura de los próximos tres.** Agrégala debajo de la declaración de ruta, mismo archivo, `docs/then-vs-now.md`; los tres artefactos viajan como un solo registro de graduación. De la tabla reimpresa, elige exactamente tres bookmarks. Para cada uno, una línea: el bookmark, y una cláusula de porque que lo ate a tu ruta. "TRPL ch16, porque mi ruta es el poller de Rust creciendo threads de verdad" es la forma. Tres, no siete. La disciplina es el artefacto.

4. **Haz el commit.**

   ```bash
   git add docs/then-vs-now.md
   git commit -m "docs: graduation record (then-vs-now, route, next three)"
   git push
   ```

   Tu estación ahora carga su propio registro de graduación, públicamente, al lado del código que se lo ganó.

5. **Revisa el latido.** Abre la pestaña Actions de tu repo y confirma que el cron sigue en verde y que `status.json` se movió en la última hora. La estación sigue latiendo estés mirando o no. Eso es lo que construiste.

## Challenge

Corre el hábito de reverificar una vez, en serio, antes de que se desdibuje. Abre el workspace de TS de tu estación y sondea qué declaran como peer tus dependencias hoy: `npm view` (npm está en tu máquina desde que llegó Node en el módulo uno) contra los `peerDependencies` de tus paquetes adyacentes a kit, la jugada exacta del módulo ocho. Compara la respuesta con tu tabla de pins. Si coinciden, agrega la fecha de hoy a la columna verified de la tabla y terminaste en cinco minutos. Si no coinciden, detectaste tu primera deriva de verdad, y sabes precisamente qué hacer: actualiza el pin, anota la fecha, corre la suite, deja que CI lo juzgue. Cualquiera de los dos resultados es una victoria; el punto es que corriste la sonda un día en que nadie te lo pidió.

## Revisa tu rumbo

El último checkpoint del curso es de treinta segundos y en voz alta. Léele tus tres artefactos a alguien, o a la sala vacía: cinco líneas de diff, una declaración de ruta, tres cláusulas de porque. Si suenan como evidencia, acá terminaste. Esa lectura en voz alta es además el autoexamen honesto: una línea de diff que murmuras por encima es una que deberías afilar, y una declaración de ruta que no puedes decir sin que se te escape la risa es una ruta que en realidad no elegiste. El quiz de esta lección recorre el mismo terreno: la lectura honesta de la puerta de Anchor, qué dice el hábito de reverificar cuando una línea de instalación vieja falla, y qué significa un error sorpresa de lifetimes para un graduado que es dueño del mapa.

Y con eso, el aha que todo este curso fue construido para entregar, dicho sin adornos: el 20% nunca faltó. Nunca fue un hueco en tu educación. Ahora sabes exactamente dónde vive cada pieza de él, capítulo por capítulo, y, más importante, sabes cuándo vas a necesitar cada pieza, porque tu ruta te lo dice. Un error de lifetimes en un lab de Anchor no es un hueco. Es un bookmark que vence, y lo vas a retirar como un ítem reservado.

No hay lección siguiente. Hay un curso siguiente, y ahora puedes leer su oración de prerrequisito con evidencia en la mano. Mientras tanto la estación sigue latiendo en su cron mientras te vas: pública, en tu GitHub, una pieza de portafolio que responde la única pregunta de entrevista que importa, sabes entregar, con una URL en vez de un párrafo. Elige tu ruta. Abre tu primer bookmark. Entrega.
