# El mapa: qué enseñamos, qué dejamos como bookmark, y por qué los dos lenguajes

Lección uno. Todavía no hay nada construido. Llegas sabiendo programar en algún lenguaje, nuevo en Rust, nuevo en TypeScript, nuevo en web3, y no voy a abrir con una definición. Voy a hacerte medir internet con el navegador que ya tienes abierto.

Abre una pestaña nueva, ve a https://rust-lang.org (la dirección apex, sin www), presiona F12 (o clic derecho, Inspect), haz clic en la pestaña Console y pega esto:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

Presiona Enter. En un segundo o así recibes una línea como `rust-lang.org: 238.5 ms (status 200)`. Eso es lo que imprimió la mía mientras escribía esto, desde una conexión casera en medio del día. (Dos cosas en ese snippet son deliberadas. `location.origin` es la dirección de la pestaña en la que estás parado, así que estás sondeando el sitio en el que estás y no una URL que yo dejé hardcodeada, y una página siempre tiene permiso de leer respuestas de su propio origen; leer una respuesta de un origen *distinto* solo funciona cuando ese servidor lo habilita con un header CORS, una regla de seguridad del navegador que esquivas por completo al preguntarle a la pestaña sobre sí misma. Y la instrucción de no poner www importa: la escritura con www de este sitio hace calladamente un 301-redirect al apex, `fetch` sigue los redirects en silencio, y una sonda que cruza orígenes en pleno vuelo puede quedar bloqueada por esa misma regla incluso cuando el sitio está perfectamente sano.) La tuya será distinta, porque es una medición real de un servidor real sobre tu red real. Sin instalación, sin cuenta, sin framework. Acabas de sondear infraestructura en vivo y leer su latencia, y ese único reflejo, apuntar una sonda a algo real y leer el número, es todo el curso en miniatura.

Ahora la segunda demo, porque este curso tiene dos lenguajes y cada uno recibe un argumento de apertura. Ve a https://www.typescriptlang.org/play/ (la barra final importa, esa es la URL final), limpia el editor y escribe estas tres líneas:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeout;
console.log(`waiting ${wait} ms`);
```

Mira la línea dos. Antes de que corras nada, aparece un subrayado rojo debajo de `timeout`, y al pasarle el mouse por encima muestra:

```
Property 'timeout' does not exist on type '{ url: string; timeoutMs: number; }'.
Did you mean 'timeoutMs'?
```

JavaScript puro correría esto alegremente e imprimiría `waiting undefined ms`. Mentiría con cortesía, y te enterarías en producción, a las 3 a.m., cuando el timeout que creías haber puesto nunca se disparó. El compilador atrapó el bug mientras todavía escribías, y encima adivinó el arreglo. Ese subrayado rojo es la tesis de toda la mitad de TypeScript en este curso: la máquina puede demostrar cosas sobre tu código antes de que el código exista en otro lugar que no sea tu editor.

Dos sondas, dos minutos, cero instalaciones. Todo lo que sigue es un mapa.

## Resumen

Esta lección te da tres cosas y te pide una decisión honesta. Primero, el mapa: una tabla completa de lo que este curso enseña frente a lo que deja como bookmark hacia recursos gratuitos canónicos, y por qué esa división es deliberada y no floja. Segundo, la evidencia para enseñar dos lenguajes a la vez, con conteos reales de bytes desde un repo emblemático de Solana y una proporción de portal de empleos leída con cuidado. Tercero, la promesa: el artefacto que vas a construir a lo largo de diez módulos, una estación personal de uptime y latencia llamada Pulse Station, dibujada como el diagrama exacto que vas a volver a dibujar de memoria en el módulo final. La decisión honesta es el chequeo de prerrequisitos al final de la sección de teoría: un recurso gratuito específico, una pregunta específica y permiso para irte y volver.

Una cosa dicha en voz alta antes de empezar, porque este curso dice sus reglas en voz alta. Ahora mismo, en la lección uno, todo está completamente resuelto: cada comando mostrado, cada snippet completo, corres en lugar de derivar. Esa ayuda se repliega según un cronograma. Unos módulos más adelante vas a recibir interfaces y constraints en lugar de archivos terminados, y para el capstone vas a recibir un spec y silencio. El repliegue es el currículo.

Ni Node, ni compiladores, ni cuentas hoy. El toolchain llega la próxima lección. Hoy es orientación, y la orientación bien hecha vale más que cualquier instalación suelta.

## El mapa es el producto

Aquí está el problema que esta lección existe para resolver. Ya puedes hacer que un programa funcione en algún lenguaje. Entre ti y entregar software de verdad sobre un stack de blockchain, internet ofrece alrededor de mil horas de material: un libro de Rust de 20 capítulos, un handbook de TypeScript, cuatro sitios de documentación de plataformas y una blockchain que cambia bajo tus pies. Nadie termina ese montón. La gente que entrega nunca lo hizo. Un montón de tutoriales no es un currículo; es un backlog, y los backlogs no enseñan. Lo que la gente que entrega hizo en realidad fue aprender un 80 por ciento específico y dejar el resto como bookmark, y la mayoría armó ese mapa por prueba, error y años.

Este curso te pasa el mapa por adelantado y después lo camina contigo. El trato, dicho en los propios términos del dueño: ya existen recursos excelentes para aprender Rust y TypeScript, así que los enlazamos en lugar de volver a enseñarlos. Lo que este curso agrega es la parte que esos recursos no cargan: cómo usas cada patrón en el mundo real, y por qué necesitas cada concepto, por lo que se rompe sin él. Selección por encima de cobertura. A la frontera entre los dos le llamamos la costura 80/20: el lado enseñado cubre los patrones que cargan el ciclo de vida diario del dev, escribir, probar, empaquetar, desplegar, operar; el lado con bookmarks es el material profundo canónico, enlazado a nivel de capítulo en el momento exacto en que podrías quererlo. Cada lección de lenguaje de aquí en adelante lleva una caja fija para profundizar, le decimos la caja del 20%, que contiene el bookmark de esa lección.

![Dos carriles muestran patrones del mundo real enseñados junto a recursos canónicos con bookmark, unidos por enlaces a nivel de capítulo puestos exactamente donde una lección los necesita.](assets/v01-diagram.webp)

El costo de ese trato es real, y deberías escucharlo ahora en lugar de descubrirlo en medio de un error. VAS a encontrarte con sintaxis de Rust y de TypeScript que este curso nunca enseñó, a veces dentro de un mensaje del compilador, tres módulos más adelante, cuando una anotación de tiempo de vida aparezca en un error de código que no escribiste. El mapa es la mitigación, no una exención mágica. El trato solo funciona si de verdad abres el capítulo con bookmark cuando una lección lo señaliza. Leer todo el libro de Rust de cabo a rabo antes de empezar es la trampa de mil horas; negarse a leer el único capítulo al que apunta el mapa, cuando apunta a él, es la falla opuesta y sale igual de caro.

### La tabla completa, impresa como contenido

Esta tabla no es un apéndice. Es el gemelo honesto del temario, y el módulo final te va a traer de vuelta a ella para preguntarte qué bookmarks se volvieron urgentes. Cada URL de abajo fue verificada en vivo el 2026-09-02, y esta única tabla se vuelve a verificar antes de que el curso entregue actualizaciones.

| Concepto | Enseñado aquí | Con bookmark a |
|---|---|---|
| Fundamentos de JS | Nunca. Este es el prerrequisito | MDN Learn Core Scripting, gratis, 34 lecciones: https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting |
| Config estricta de TypeScript | M1: las ~5 flags que dispara nuestro propio código | Intro del TS Handbook: https://www.typescriptlang.org/docs/handbook/intro.html |
| Uniones discriminadas, narrowing, exhaustividad | M2 | Capítulo de narrowing del Handbook; el repo type-challenges como el ejercicio después |
| Genéricos que de verdad usas | M2, justo a tiempo, motivados por zod | Capítulo de genéricos del Handbook |
| Validación en las fronteras | M2, zod | El tutorial gratuito de Zod de Total TypeScript: https://www.totaltypescript.com/tutorials |
| Async en serio: límites, backoff, cancelación | M2 | La unidad de async de MDN |
| Práctica de pruebas | M2 (vitest), M4 (cargo test) | Guía de vitest; docs de node:test |
| Alfabetización en empaquetado y publicación | M3 | La ruta de aprendizaje de Node: https://nodejs.org/learn |
| React, a nivel de consumidor | M3 | Quick start de react.dev; la profundidad le pertenece a un curso hermano, nombrado abajo |
| El ownership (propiedad) y el préstamo, el por qué y el uso diario | M4 | El fork interactivo de Brown del Rust Book, con quizzes y visualizaciones de ownership: https://rust-book.cs.brown.edu/ |
| Errores como valores | M4 | The Rust Book, capítulo de manejo de errores: https://doc.rust-lang.org/book/ |
| Enums, structs, traits en la práctica | M4 | Capítulos de enums y traits del Rust Book |
| serde, dominio de cargo, clap, un CLI de verdad | M5 | serde.rs; el Cargo book; Rustlings después de cada módulo de Rust: https://rustlings.rust-lang.org/ |
| tokio, a profundidad de saber-cuándo-lo-necesitas | M6 | Capítulo de async del Rust Book (el Book ahora cubre async de forma nativa) |
| Amplitud de la sintaxis de Rust | Nunca se vuelve a enseñar | Rust by Example: https://doc.rust-lang.org/rust-by-example/ ; Comprehensive Rust, el curso con el que Google hace onboarding a su propio equipo de Android: https://google.github.io/comprehensive-rust/ ; el speed-run de sintaxis de media hora en fasterthanli.me |
| Tiempos de vida más allá de leerlos, unsafe, autoría de macros | Nunca, señalizado | The Rustonomicon, que abre diciéndote que todavía no lo leas: https://doc.rust-lang.org/nomicon/ |
| Contenedores | M6 | Docker Get Started: https://docs.docker.com/get-started/ |
| Desplegar la app de TS | M3 | El getting started de Vercel: https://vercel.com/docs/getting-started-with-vercel |
| El edge, los dos lenguajes | M7 | Cloudflare Workers get started: https://developers.cloudflare.com/workers/get-started/guide/ |
| Profundidad de Solana, Anchor, UX de billetera | Solo el modelo de cliente mínimo, M8 | Cursos hermanos en este catálogo, nombrados en la prosa de abajo |

Mira las filas de nunca. Un curso que puede decir "nunca" en voz alta, y decirte exactamente dónde vive ese material en cambio, te está haciendo una promesa: nada del lado enseñado es relleno, y nada del lado con bookmarks es secretamente obligatorio para pasar un lab. Cada lab de este curso corre con material enseñado más, como máximo, un capítulo señalizado. Ese es el ajá que vale la pena dejar reposar un segundo: un curso puede ser honesto sobre lo que no enseña. La lista de bookmarks ES el producto.

![Un diagrama de fronteras muestra que los labs pueden requerir solo material enseñado más un capítulo señalizado, mientras el resto del mapa con bookmarks queda afuera.](assets/v02-diagram.webp)

### Por qué los dos lenguajes, con comprobantes

Pregunta justa: ¿por qué no solo Rust, si la blockchain corre Rust? ¿O solo TypeScript, si esa es la rampa de entrada más corta? Porque un producto de Solana de verdad no está escrito en un lenguaje. Está escrito en dos.

El mejor número de todos los que encontró la investigación: drift-labs/protocol-v2, un repo de producción emblemático de Solana, se divide casi exactamente a la mitad por bytes. TypeScript 5,746,893 bytes, Rust 5,533,370 bytes. Eso no es una migración atrapada en pleno vuelo. Rust es el programa on-chain; TypeScript es su SDK, sus clientes, sus pruebas, toda la superficie que usuarios e integradores de verdad tocan. Las dos mitades son estructurales. Un producto, los dos lenguajes, casi cincuenta-cincuenta.

![Una sola barra para un repositorio de producción de Solana se divide casi en partes iguales entre TypeScript y Rust por bytes.](assets/v03-chart.webp)

El mercado laboral cuenta la misma historia desde un ángulo distinto, si lo lees con cuidado. En web3.career el 2026-09-01, una consulta de Rust más Solana devolvió 950 empleos y una consulta de TypeScript más Solana devolvió 505. No leas esos como conteos absolutos; los portales de empleos multi-etiquetan con agresividad y la misma publicación aparece bajo varias consultas. Lee la proporción de alrededor de 2:1 a favor de Rust por lo que dice sin rodeos: el trabajo central de la blockchain es Rust, y ahí es donde se concentran el volumen de publicaciones, y los peldaños de arriba de la escalera. El 505 de TypeScript no es una oportunidad más chica sino una rampa de entrada más corta: como acaba de mostrar la división por bytes, cada uno de esos equipos de Rust también entrega una superficie de TypeScript, así que TS es la mitad por la que te pueden contratar más pronto mientras la mitad de Rust se acumula. Un dev que sostiene los dos extremos de esa escalera es la forma que los equipos de verdad necesitan. Este curso existe porque esa forma no tiene un curso dedicado. Lo que me lleva a los vecinos.

### Los vecinos, nombrados con honestidad

Quiero nombrar los otros cursos de este espacio, porque varios son buenos, y porque tres de ellos están literalmente dentro de nuestro 20 por ciento con bookmark. Eso no es algo normal que haga un curso, y el hecho de que se sienta inusual vale la pena notarlo.

Cyfrin entrega un curso gratuito de Solana en Updraft, anunciado a través del Codex de Colosseum el 2026-01-16, y es trabajo serio: cada programa construido dos veces, una vez en Anchor y una vez en Rust nativo. Su prerrequisito es el propio curso gratuito Rust Programming Basics de Cyfrin, al que nuestros módulos de Rust van a apuntar con gusto. La School of Solana de Ackee también es gratis, una cohorte de nueve semanas, con acceso por postulación, y su oración de audiencia es casi palabra por palabra la nuestra. Ninguno enseña TypeScript como pista de primera clase, ninguno toca Docker, Vercel ni Cloudflare, y los dos van más profundo on-chain que nosotros. Mapas distintos, dibujados con honestidad.

El mercado también cobra dinero de verdad por el material que nosotros enlazamos gratis. RareSkills le pone precio a su bootcamp de Rust en $900 por 3 semanas, a su bootcamp de ZK en $2,600 por 14 semanas, y a su material de Circom en hasta $5,500. No me estoy burlando de esos precios; las cohortes y el code review valen lo que cuestan. Te estoy calibrando: el conocimiento crudo es gratis y está enlazado en la tabla de arriba, y lo que elijas pagar, aquí o en cualquier parte, debería ser selección, secuenciación y feedback, nunca acceso.

![Una columna de pago con bootcamps que van de novecientos a varios miles de dólares está al lado de una columna gratuita de recursos canónicos que este curso enlaza.](assets/v04-comparison.webp)

Un recurso de pago se gana una declaración, una sola vez, porque le apunta exactamente a la audiencia de este curso. El 2023-04-25, ThePrimeagen entregó "Rust for TypeScript Developers" en lo que ahora es Master.dev, 5 horas 19 minutos, de pago con un preview gratuito. El mercado nombró a nuestra audiencia tres años antes de que lo hiciera este curso. Si terminas aquí y quieres una segunda voz sobre la mitad de Rust, esa es la opción de pago nombrada, y es la única que este curso va a nombrar jamás.

### Pulse Station: la cosa que de verdad vas a construir

Los mapas no motivan a nadie. Los artefactos sí. Así que aquí está la promesa, concreta como para exigirme que la cumpla.

A lo largo de diez módulos construyes Pulse Station, una estación personal de uptime y latencia. Empieza vergonzosamente chica: la próxima lección, un CLI de TypeScript que sondea una URL e imprime la latencia, que vas a notar es exactamente el one-liner que corriste en la consola hoy, promovido a programa de verdad. Después crece. La sonda pasa a un calendario en GitHub Actions, en una máquina que no es tuya, y hace commit de sus mediciones a un archivo. Los resultados reciben tipos, después validación, después concurrencia disciplinada, después pruebas que le ponen una barrera al calendario. Un panel en Vercel le da una cara que cualquier desconocido puede abrir. El motor se reescribe en Rust, lado a lado con el TypeScript que refleja, le crece un CLI de verdad, después un poller de larga duración en un contenedor de Docker publicado en GHCR. Las sondas pasan al edge en Cloudflare Workers, en los dos lenguajes, uno de ellos compilado a WebAssembly. Y después la estación apunta al target más interesante disponible: Solana misma, lecturas en vivo y una transacción real, una blockchain cuyo propio latido es una historia de latencia. Cada sonda, desde el one-liner de la consola de hoy hasta el capstone, le pega a un target real. No hay feeds sintéticos en ninguna parte de este curso. Y cuando los diagramas de abajo digan "Solana RPC" y "devnet transaction", léelos por ahora en sentido amplio como el endpoint público de consultas de la blockchain y su red gratuita de práctica; el módulo 8 define bien los dos antes de que toques cualquiera.

![Una línea de tiempo de diez paradas hace crecer una sonda de consola hasta una flota tipada, un panel, un motor de Rust, un contenedor, edge workers y una estación que vigila Solana.](assets/v05-timeline.webp)

La estación terminada tiene una forma, y la forma importa lo suficiente como para que el capstone te pida volver a dibujarla de memoria. Este es ese dibujo. Estúdialo ahora, a la ligera; lo vas a volver a encontrar en el módulo diez.

![Un sistema de hub y radios se centra en un repositorio público alimentado por sondas programadas y leído por un panel, edge workers y un poller en contenedor que vigila targets reales.](assets/v06-diagram.webp)

Nota la palabra public en el hub, porque es un requisito, no un default que se me olvidó cambiar. El repo de tu estación va a ser público, y tres rutas críticas del curso dependen de eso: GitHub Actions es gratis y sin medición para repositorios públicos en runners estándar, que es toda la matemática de CI gratis del calendario en el que vive tu sonda; el panel trae `status.json` directo desde la URL raw pública del repo; y el lab de deploy en Vercel conecta un repo personal público en el tier gratuito. Un repo privado rompe calladamente las tres, con semanas de diferencia, de formas confusas. El costo es igual de real y está dicho en voz alta: el código de tu estación y su historial completo de status son públicos. Para un artefacto de portafolio, y este es uno, ese costo es una feature. Los empleadores pueden ver tu estación corriendo.

### La caja de honestidad, y las puertas que este curso abre

Primero, el chequeo de prerrequisitos, y lo voy a citar exactamente como el curso lo dice en todas partes: abre MDN Learn Core Scripting, incluida su unidad de async. Si puedes seguirlo cómodamente, estás listo. Si no, empieza ahí. Es gratis, y este curso va a seguir aquí.

Este curso no es para principiantes absolutos en programación; la portada lo dice y lo estoy repitiendo. El chequeo incluye la unidad de async a propósito. El módulo dos construye disciplina de concurrencia directamente sobre alfabetización en promises, con solo un repaso de cinco minutos sobre el modelo de promises, y un repaso no es un primer curso de async. Saltarse el chequeo no te hace más rápido. Reubica la demora al módulo dos y la hace más cara.

![Un flujo de decisión corto dirige a los lectores cómodos hacia el curso y a los lectores que honestamente dicen "todavía no" hacia un desvío gratuito por MDN antes de volver.](assets/v07-flowchart.webp)

Segundo, las puertas. Este es el curso alimentador del catálogo, lo que quiere decir que termina donde empiezan los cursos más profundos, y esas fronteras están trazadas a propósito. La profundidad de Solana, el modelo de cuentas como sistema, los programas, los PDA, la historia de la blockchain, vive en el curso btc-to-sol-evolution; nosotros enseñamos solo el modelo mínimo del lado del cliente en el módulo ocho. La UX de billetera, el aterrizaje de transacciones y todo lo que tiene que ver con lograr que una transacción entre bajo presión le pertenece al curso de dominio del lado del cliente, que está en producción mientras escribo esto; hasta que se entregue, la puerta es el tema mismo, y el panel de nuestro módulo tres construye el piso de React a nivel de consumidor sobre el que se para ese tipo de trabajo. Anchor y la autoría de Rust on-chain viven en el curso Anchor V2, y esa puerta merece un letrero franco: su propia vara es que ya entregas programas de Anchor y quieres el delta de V2, así que nuestra salida de Rust, leer y escribir structs, enums, traits y Result, con los tiempos de vida señalizados, te compra el nivel de lectura para su prosa, no un lugar por encima de su vara. Cuando una lección de aquí se niega a ir más profundo en uno de esos temas, va a nombrar la puerta en cambio. Misma costura, a escala de catálogo.

## Lab: corre las sondas, toma la decisión

Numerado y corto, porque el punto de hoy es la decisión, no el tooling. Todo aquí es cero instalación por diseño; ningún paso requiere Node, una cuenta ni una descarga.

1. **Corre la sonda de latencia.** Abre https://rust-lang.org en una pestaña (dirección apex, sin www, igual que en la apertura), abre devtools (F12, pestaña Console), pega el one-liner de `fetch` que está al principio de esta lección, presiona Enter. Aquí está de nuevo para que no tengas que hacer scroll:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

   Copia la línea impresa, algo con la forma `rust-lang.org: 238.5 ms (status 200)`, a una nota borrador. Esa línea es tu primer artefacto. Como el snippet sondea `location.origin`, el sitio en el que está la pestaña, puedes pegarlo en la consola de cualquier otro sitio y simplemente sondea ese sitio en cambio, y sigue funcionando ahí por la misma razón por la que funcionó aquí: una página siempre puede leer las respuestas de su propio origen. Sondear una URL de terceros desde la pestaña de alguien más es otro juego. El navegador solo te da una respuesta cross-origin legible cuando el servidor target lo habilita con un header CORS, y la mayoría no lo hace; agregar `mode: "no-cors"` junto a `cache` detiene la falla rotunda, pero entonces el navegador te devuelve una respuesta opaca y el snippet imprime `status 0`. Un 0 ahí quiere decir "opaco a propósito", no un sitio muerto. El sondeo del mismo origen es la versión que siempre funciona y siempre muestra un status real, y es por eso que el paso uno empieza ahí.

2. **Córrelo cuatro veces más.** El mismo pegado, cuatro Enters más. Mira cómo se mueve el número. Conexiones frías, caché de DNS, el clima de la ruta; la latencia es una distribución, no un valor, y acabas de descubrir eso con la paciencia equivalente a un for-loop. Anota tu más rápida y tu más lenta. El CLI de la próxima lección convierte exactamente esta repetición en código.

3. **Dispara la captura del compilador.** Abre https://www.typescriptlang.org/play/ y escribe el snippet de tres líneas de la apertura. No lo pegues; escríbelo, y mira qué temprano aparece el subrayado. Pásale el mouse y lee el error completo, incluido el arreglo sugerido. Haz un screenshot o copia el texto del error a la misma nota borrador. Segundo artefacto.

4. **Rómpelo peor.** Todavía en el Playground, cambia la línea dos para que el archivo diga:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeoutMs + probe.url;
console.log(`waiting ${wait} ms`);
```

   Lee lo que dice el compilador sobre sumar un número a un string. Lo permite (las reglas de JavaScript lo permiten) pero pásale el mouse al tipo del resultado y nota que `wait` ahora es un string. El compilador no es un linter gritando no; es un contador que siempre sabe qué tipo tienes en realidad. Dos minutos de hurgar aquí rinden a lo largo de todo el módulo dos.

5. **Abre el chequeo de honestidad.** Ve a la página de MDN Learn Core Scripting de la tabla, échale un ojo a la lista de lecciones y abre su unidad de async específicamente. Lee una página o dos. Después toma la decisión, una palabra en tu nota borrador: listo, o MDN-primero. Las dos respuestas pasan este lab. La única respuesta que falla es la que no se examinó.

6. **Guarda el mapa.** Deja como bookmark la tabla de esta lección en el sistema que de verdad revisitas. Es la única pieza de hoy que vas a usar por meses.

Checkpoint: tu nota borrador guarda una línea real de latencia, un error real del compilador y un veredicto de una palabra. Esa es toda la barrera. Treinta segundos de artefactos, una decisión honesta.

## Challenge

**Completion (todos):** apunta la sonda a otro lado. Cualquier sitio que te importe: tu propio proyecto, tu sitio de docs favorito, tu banco. El mismo one-liner, pestaña nueva, target nuevo, número nuevo. Si el fetch falla donde rust-lang.org tuvo éxito, lee el error de la consola y forma una hipótesis sobre por qué; ahora tienes un misterio que la discusión de políticas de seguridad en un módulo más adelante va a resolver.

**Solo (opcional, sin walkthrough):** en el Playground, escribe un objeto `probe` que guarde un array de URLs target y un `timeoutMs`, después escribe una línea que acceda a una propiedad que no definiste y otra que indexe más allá de lo que sabes que guarda el array. Mira cuál de las dos atrapa el compilador bajo los settings default y cuál deja pasar. Acabas de encontrar, por tu cuenta, el hueco exacto que una flag de strict mode en el recorrido del tsconfig de la próxima lección existe para cerrar. Trae tu hallazgo contigo.

## Revisa tu rumbo

Tres preguntas honestas antes de que avances. ¿Puedes volver a correr las dos sondas en frío, sin esta lección abierta? ¿Puedes explicarle la costura 80/20 a otro dev en dos oraciones, incluido qué es la caja del 20%? ¿De verdad abriste MDN y tomaste la decisión, o le asentiste al párrafo y seguiste haciendo scroll? El quiz de esta lección sondea los mismos bordes: qué dice el contrato que hagas cuando un error del compilador menciona territorio con bookmark, de qué es evidencia la división por bytes de drift, cuál es la jugada honesta cuando la unidad de async es nueva para ti y qué depende en realidad de que el repo de la estación sea público. Si alguna respuesta se siente blanda, el lab se vuelve a correr en cinco minutos.

Me fui largo en la sección del mapa; eso fue a propósito, y no va a ser el patrón. Acabas de sondear internet desde una consola de navegador, y esa medición muere cuando la pestaña se cierra. La próxima lección instalas el toolchain de verdad, Node 24 LTS y TypeScript 7 (la antorcha LTS de Node pasa a 26 el 2026-10-28; la lección fija lo que verifica el día en que la corres), y conviertes ese one-liner en `pulse` v0: una sonda que vive en un repo, con tipos y todo, sin nada entre ti y ella más que una instalación. Deja la pestaña de la consola abierta hasta entonces si quieres; para el final de la próxima lección no la vas a necesitar.
