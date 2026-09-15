# Contenedores 101: mete el poller en una caja

## Resumen

m06-l1 convirtió el CLI en `pulse-pollerd`: un loop de poll de tokio que envuelve el crate del motor, con el drop-in trabajado de axum `/status` respondiendo en el puerto 8080. Corre para siempre, pero solo en tu máquina. Hoy arreglamos la parte de "solo en tu máquina". Vas a instalar un runtime de contenedores, correr tu primer contenedor dentro del primer cuarto de esta lección, construir el único modelo mental que hace que Docker deje de ser magia, y después escribir el Dockerfile más honesto posible para el poller: ingenuo, de una sola etapa y gloriosamente pasado de peso. Una nota sobre el apoyo, en voz alta: este curso asume que nunca tocaste Docker, así que esto es un primer contacto completamente trabajado. El Dockerfile es un problema de completion con exactamente tres TODOs, y el challenge en Solo es un único cambio de variable de entorno. Las rueditas se vuelven a sacar la lección que viene.

Primero, treinta segundos de reconocimiento. Abre una terminal y pregunta si ya hay un runtime de contenedores en tu máquina:

```bash
docker version
```

Números de versión tanto para Client como para Server quieren decir que ya hay un runtime instalado y corriendo; vas a pasar rápido por el paso de instalación de abajo. `command not found`, o un cliente que responde mientras la mitad del servidor tira error, es el resultado esperado para la mayoría de ustedes, y instalar un runtime es el primer trabajo de esta lección. De cualquier forma, ahora sabes en qué camino estás.

## La caja en la que se puso de acuerdo la industria

Agarra el binario de release que CI te construyó en m05-l3 y pásaselo a un amigo. Si corre otra distro de Linux, hay una buena probabilidad de que muera con una versión de glibc más vieja que aquella contra la que enlazó tu runner de CI, con un mensaje de error que nombra una versión de símbolo y no le sirve a nadie. Si está en macOS y CI construyó para Linux, no arranca en absoluto; formato de ejecutable equivocado, punto. Y aun cuando el binario corre, tu poller lee `pulse.config.json` desde su directorio de trabajo y espera que el puerto 8080 esté libre, supuestos que tu máquina cumple y la suya quizás no. "Funciona en mi máquina" deja de ser un chiste en el momento en que alguien lo abre como reporte de bug. El arreglo en el que convergió la industria no es entregar el binario. Es entregar la caja con forma de máquina en la que el binario corre: el sistema de archivos, las librerías, la config, las expectativas de puerto, todo congelado junto para que lo único que aporte la máquina destino sea un kernel.

Primero, el que corre la caja. Instálalo ahora.

### Elige un runtime, corre un contenedor

Docker el *formato* es abierto y estándar. Docker la *app de escritorio* es un producto con licencia, y hay tres maneras sensatas de conseguir un runtime en tu máquina:

![Docker Desktop, OrbStack y colima comparados por precio y plataforma, con los tres gratis para quien aprende por su cuenta.](assets/v01-comparison.webp)

Por qué existen tres opciones es una historia con fecha. El 2021-08-31, Docker anunció que Docker Desktop dejaría de ser gratis en el trabajo: cualquier empresa de más de 250 empleados o más de $10M de facturación necesitaría una suscripción paga, con un período de gracia hasta el 2022-01-31. Ese único anuncio es la razón por la que OrbStack y colima se volvieron nombres conocidos entre los desarrolladores de mac. Los umbrales siguen en pie hoy y, para ti, ahora mismo, son irrelevantes: uso personal, educación y empresas chicas se quedan gratis en las tres. Elige por gusto (versiones verificadas el 2026-09-02; Docker entrega mensualmente, así que tus dígitos pueden correr más alto):

**Docker Desktop** (macOS, Windows, Linux) es el camino por defecto y lo que asumo en esta lección. Descarga el instalador desde la página Get Started de Docker enlazada al final de esta sección, córrelo, abre la app una vez para que arranque el engine. El release actual es 4.89.0, corriendo Engine 29.7.2 por debajo.

**OrbStack** (macOS): `brew install orbstack`. Gratis para uso personal, $8/usuario/mes cuando paga una empresa.

**colima** (macOS/Linux): `brew install colima docker`, después `colima start`. OSS gratis, sin GUI, y el CLI de `docker` le habla exactamente como le hablaría a Desktop.

En Linux el cálculo es más simple: Docker Engine en sí es open source y gratis en todos lados, incluido el trabajo, y se instala desde el camino de paquetes de tu distro según los docs de Get Started. La historia de licencias de arriba es sobre la app de escritorio, que en Linux es una comodidad más que una necesidad.

Cualquiera que hayas elegido, la prueba de vida es la misma:

```bash
docker run hello-world
```

Primera corrida, te salen barras de progreso del pull, después un mensaje que empieza:

```text
Hello from Docker!
This message shows that your installation appears to be working correctly.
```

Córrelo una segunda vez. Sin barras de progreso, salida instantánea. Esa diferencia no es una cache tibia del lado de Docker; es la arquitectura entera de la cosa mostrándose en tu primer comando, y vale un diagrama antes de seguir adelante.

Mientras la evidencia está fresca, un comando más:

```bash
docker ps -a
```

`docker ps` solo lista los contenedores *corriendo*, y ahora mismo esa lista está vacía, porque el proceso de hello-world imprimió y salió. El `-a` muestra también los que salieron, y hay dos: uno por cada `docker run`, cada uno con un nombre autogenerado, los dos detenidos, los dos creados desde la misma única imagen. Nada se reusó entre las corridas salvo la plantilla. Límpialos con `docker rm` y los nombres o IDs que muestra el listado, o empieza a formar el hábito que usa el lab: `--rm` en la corrida misma, para que el cadáver nunca quede dando vueltas.

![El primer docker run le hace pull a la imagen desde el registro y la cachea, mientras la segunda corrida reusa la imagen cacheada y solo crea un contenedor nuevo.](assets/v02-flowchart.webp)

### Imagen, contenedor, capa, registro

Cuatro palabras cargan este módulo entero, así que fijémoslas mientras la salida de hello-world sigue en tu pantalla.

Una **imagen** es un sistema de archivos inmutable y en capas, más algo de metadatos: qué comando correr, qué puertos pensó el autor, qué variables de entorno poner. Es una plantilla. No hace nada por sí sola.

Un **contenedor** es un proceso corriendo al que le pasaron ese sistema de archivos como su raíz, más una capa escribible delgada encima para que pueda garabatear sin tocar la plantilla. `docker run` estampa uno a partir de una imagen igual que `cargo run` estampa un proceso a partir de un binario. Tres corridas, tres contenedores, una imagen.

Una **capa** es una etapa cacheada del build de una imagen. Cada instrucción de un Dockerfile produce una, apilada de solo lectura encima de la anterior, y la capa escribible del contenedor que corre se sienta arriba de toda la stack: cuando el proceso escribe un archivo, el cambio aterriza ahí, y cuando modifica un archivo de una capa más abajo, el archivo se copia hacia arriba primero y se cambia en la copia. La plantilla de abajo nunca se toca, que es por qué tres contenedores pueden compartir una imagen sin pisarse entre sí, y por qué todo lo que un contenedor escribe muere con él a menos que deliberadamente dispongas otra cosa. Vas a ver las capas pasar volando en el lab, con precio individual.

Un **registro** es donde viven las imágenes para que otras máquinas puedan hacerles pull. Ya conoces esta forma dos veces: npm es un registro de paquetes y crates.io es un registro de crates; Docker Hub es un registro de sistemas de archivos. Publica una vez, haz pull en cualquier parte, resuelve por nombre y tag en vez de por nombre y semver. La analogía es lo bastante cercana para apoyarse en ella y lo bastante honesta para acotarla: los tags de imagen son etiquetas mutables, no versiones inmutables, así que `rust:1.98` puede apuntar mañana a una imagen reconstruida de una forma en que `serde@1.0.229` nunca lo hará. Docker Hub es el registro por defecto, es de donde vienen `hello-world` y la imagen base `rust`, y es un servicio con rate limits con los que vamos a lidiar con honestidad en un minuto. Un **tag** es la etiqueta legible por humanos después de los dos puntos, el `:naive` de `pulse-pollerd:naive`. Una **imagen base** es simplemente la imagen desde la que arranca tu imagen, el argumento de la línea `FROM`, que aporta sus capas como tu cimiento.

![Un registro sirve una imagen hecha de capas de solo lectura apiladas, y cada contenedor corriendo es un proceso aparte con su propia capa escribible delgada sobre esa misma imagen.](assets/v03-diagram.webp)

Ahora el modelo que hace que todo esto se condense en algo sobre lo que puedes razonar. El dibujo tentador, el que la palabra "contenedor" te planta en la cabeza, es una máquina virtual chica: una computadora chiquita que arrancas, a la que le haces login y en la que hurgas. Desarrolla ese dibujo y esperas hacer ssh adentro, instalar cosas, reiniciarla.

No es eso. Un contenedor es un proceso vistiendo un sistema de archivos. Un proceso, arrancado por tu kernel como cualquier otro, salvo que el kernel le muestra un directorio raíz distinto y una vista del mundo cercada. Acá está la prueba que lo zanja: ¿qué pasa cuando el proceso sale? El contenedor se terminó. Nada quedó arriba, porque no había máquina, solo el proceso. `hello-world` imprimió su mensaje, salió, y su contenedor terminó en el mismo respiro. Eso es también por qué el segundo `docker run` creó un contenedor *nuevo* en vez de reengancharse al viejo: los contenedores son tan descartables como los procesos, porque eso es lo que son.

Si todo este montaje necesita un dibujo de afuera del software: el contenedor de carga intermodal. Antes de la caja de acero estandarizada, cargar un barco de carga significaba estibadores apilando a mano barriles y cajones, cada barco un caso especial. La caja estandarizó la *interfaz*, y de repente a la grúa, al barco, al camión y al puerto dejó de importarles qué había adentro. Docker es esa caja para el software: el registro es el puerto, la imagen es el contenedor sellado, y cualquier host con un runtime es un barco que puede cargarlo. Donde la analogía se rompe, y se rompe: una caja de acero es carga inerte, mientras que nuestra caja viene con la instrucción de arrancar exactamente un proceso. Lleva la analogía hasta la logística y suéltala antes del comportamiento.

Una nota al pie de honestidad antes de que alguien en una Mac me agarre: en macOS y Windows, los contenedores de Linux no pueden correr directo sobre el kernel del host, así que Desktop, OrbStack y colima manejan cada uno en silencio una VM de Linux y corren tus contenedores adentro de ella. El modelo se sostiene igual; tus contenedores son procesos en *ese* kernel. Acabas de pagar la ilusión con algo de RAM, que es parte de la cuenta que vamos a sumar en breve. Y fíjate en lo que el modelo cambia sobre tus reflejos de debugging: el sistema de archivos, el entorno y la red de la caja son el mundo del autor de la imagen, no el de tu máquina, así que "funciona en el contenedor" y "funciona en mi host" ahora son afirmaciones separadas con evidencia separada. Esa separación es la victoria de portabilidad con ropa de trabajo.

### Haz login antes del primer pull de verdad

El lab de abajo le hace pull a la imagen base `rust` desde Docker Hub, y quiero que hagas login primero, porque la falla que te ahorras es genuinamente fea de diagnosticar. Escenario: estás en un coworking, en un campus, o adentro de un runner de CI. Tu build muere haciendo pull de una imagen base con un error 429. En tu casa, el build idéntico funciona. Nada en el error menciona por qué.

Lo que está pasando: los pulls no autenticados de Docker Hub están topeados en 100 pulls cada 6 horas *por dirección IPv4* (o por subred IPv6 /64), y detrás de un NAT compartido, todos en el edificio están gastando el mismo cupo. Tú no hiciste nada; las cincuenta laptops a tu alrededor sí. Una cuenta gratis de Docker Hub te pasa a tu propio cupo de 200 pulls cada 6 horas, atado a tu cuenta en vez de a la IP del edificio. Las dos cifras son del propio Docker, leídas en docs.docker.com/docker-hub/usage/ el 2026-09-06, y son las que sobrevivieron a la saga de políticas de 2025: solo los planes pagos Pro, Team y Business tienen una tasa de pulls ilimitada. Revisa la tabla tú mismo antes de citarle un número a un colega; ese es el hábito, no el dígito. La variante de CI de esta falla es la que muerde a los equipos: los runners hosteados comparten las direcciones de egreso de su proveedor de nube con miles de desconocidos, así que un pull anónimo que funcionó todo el sprint empieza a fallar de a ratos la semana en que se entrega algo popular. Misma causa, mismo arreglo, y ahora puedes diagnosticarlo solo por el patrón de síntomas: depende de la ubicación, es reproducible, y es 429 en vez de not-found.

![Muchas laptops detrás de una IP compartida agotan un cupo común de pulls de cien cada seis horas, mientras un usuario logueado recibe sus propios doscientos.](assets/v04-diagram.webp)

Así que: crea la cuenta gratis en hub.docker.com, después:

```bash
docker login
```

Pon el usuario y el token o la contraseña que te pida; `Login Succeeded` es tu checkpoint. (El plan Personal gratis además trae un repositorio privado de imágenes, que es más hosting del que este curso le va a pedir.) Ese es todo el arreglo. Cuando ni 200 cada 6 horas alcanzan, o quieres tus propias imágenes hospedadas al lado de tu código, GHCR, el registro de GitHub, es la escotilla de escape, y es exactamente adonde empujamos la imagen del poller en m06-l4. Hoy no.

Voy a confesar adónde se fueron mis propias horas en este territorio, porque no fue el rate limit. Fue correr un contenedor, hacerle curl a `localhost:8080`, no conseguir nada, y concluir que mi app estaba rota. La app estaba bien. Me había olvidado del mapeo de puertos, así que mi request nunca entró en la caja en absoluto. Vas a cablear ese mapeo deliberadamente en el lab, y cuando lleguemos ahí vas a ver por qué el adentro y el afuera de un contenedor son redes distintas.

### Lo que cuesta la caja

El trade-off, dicho antes de que construyas nada, porque este se mide en gigabytes. Una imagen ingenua entrega tu entorno de build entero: toolchain, caches de dependencias, fuente. La imagen base `rust:1.98` sola pesa alrededor de 600 MB comprimida en Docker Hub (la miré bajar por el cable mientras verificaba esta lección el 2026-09-02), y se descomprime a bastante más; agrega tu directorio `target/` y la imagen de un binario de pocos megabytes aterriza en los gigabytes. Los mapeos de puertos y las caches de capas son lugares nuevos para que vivan bugs, que no existían cuando corrías `cargo run`. Y en macOS está esa VM de Linux administrada, ociosa, en tu RAM. Aceptas todo eso porque "corre igual en todos lados" es el cimiento sobre el que se paran CI, los registros y cada módulo posterior de este curso. El peso, al menos, tiene arreglo, y arreglarlo es literalmente la lección que viene.

**Profundiza (el 20%).** esta lección te da el modelo mental, el login y tu primera imagen; el tour guiado de la plataforma más amplia, volúmenes, redes de contenedores, el CLI completo, vive en el camino oficial Get Started de Docker: [https://docs.docker.com/get-started/](https://docs.docker.com/get-started/). Déjalo como bookmark, recorre sus primeras dos secciones esta semana. Una meta-lección que viene gratis: Docker alojaba este material en una URL de "workshop", y el propio verificador de links de este curso agarró esa URL redirigiendo sin avisar a otro lado mientras se verificaban los hechos de esta lección. Verifica los links antes de confiar en los bookmarks del año pasado, incluidos los míos. El lab de abajo no necesita nada del material que quedó como bookmark.

## Lab: pulse-pollerd:naive

Terminal abierta en la raíz del workspace `pulse-rs`, la que contiene `Cargo.toml` con `[workspace]`, `crates/pulse-engine`, `crates/pulse-cli`, `crates/pulse-pollerd` y `pulse.config.json`. Vamos a meter el poller en una caja con el Dockerfile más obvio que pueda llegar a funcionar, a propósito, y a leer los restos con honestidad.

1. **Ponle cerca al contexto de build primero.** Cuando Docker construye una imagen, le manda el "contexto de build", por defecto tu directorio actual entero, al engine. Tu workspace contiene un directorio `target/` con gigabytes de cache de build, y `COPY . .` lo arrastraría alegremente adentro de la imagen. Crea `.dockerignore` en la raíz del workspace:

```text
target/
.git/
```

Misma idea que `.gitignore`, público distinto: esto recorta lo que el build siquiera puede ver. (Nota honesta sobre la segunda línea: el `.git/` de la estación en realidad vive un nivel más arriba, en la raíz del repo, desde que m04-l3 movió `pulse-rs/` adentro del repo de la estación, así que nunca entra en absoluto a este contexto de build. La línea de ignore no cuesta nada y salva a quien construya desde un clone independiente, que es por qué se queda.) Hazlo antes del primer build y nunca vas a saber qué tan lenta era la alternativa.

2. **Completa el Dockerfile.** Crea un archivo llamado `Dockerfile` en la raíz del workspace. Acá está el esqueleto, con los tres TODOs de la lección:

```dockerfile
# TODO 1: pick the base image. We need a full Rust toolchain to compile,
# and we pin the stable minor: rust:1.98
FROM ???

WORKDIR /app

# TODO 2: what does the build need copied in? The whole workspace: every
# crate, the root Cargo.toml, and pulse.config.json the poller reads.
COPY ??? ???

RUN cargo build --release --bin pulse-pollerd

EXPOSE 8080

# TODO 3: the command the container runs when it starts. One process,
# remember: this IS the container.
CMD ???
```

Trabaja los tres TODOs contra lo que sabes, después verifica contra la versión llena:

```dockerfile
FROM rust:1.98

WORKDIR /app

COPY . .

RUN cargo build --release --bin pulse-pollerd

EXPOSE 8080

CMD ["/app/target/release/pulse-pollerd"]
```

![Cada instrucción del Dockerfile anotada con su propósito, mostrando que la imagen base y la etapa de cargo build aportan las capas pesadas mientras EXPOSE y CMD son metadatos.](assets/v05-annotated-code.webp)

Una verificación de prevuelo antes de las glosas, y a algunos de ustedes les va a ahorrar un crash desconcertante. Allá en m05-l1 creaste el `pulse.config.json` de la raíz de la estación y apuntaste el lado de Rust ahí, un archivo, no dos copias, y la jugada sugerida fue un symlink. `COPY` copia el link, no los bytes: el destino del link vive fuera de este contexto de build, así que adentro de la imagen queda colgando, y el poller muere al arrancar sin poder abrir un archivo que `ls` jura que está justo ahí. Corre `ls -l pulse.config.json` en la raíz de `pulse-rs`; si ves una flecha, reemplaza el link con una copia de verdad (copia el destino encima con `cp`) antes de construir, y COMMITEA el reemplazo, no solo el arreglo local: el job de CI de m06-l4 construye esta misma imagen desde un checkout fresco en un runner, donde un symlink trackeado que apunta fuera del contexto de build queda colgando igual de mal, dos lecciones más tarde, cuando ya nadie se acuerda de por qué. La disciplina de un-solo-archivo termina con honestidad en el límite de la imagen, porque un sistema de archivos sellado no puede seguir un puntero de vuelta a tu laptop; mantener las dos copias de acuerdo ahora es una tarea de mantenimiento real (chica), y es el precio de la caja, no el bug de la caja.

Cuatro glosas que los comentarios del esqueleto no podían meter. `rust:1.98` fija la minor del toolchain igual que `rust-toolchain` la fija localmente; el tag existe en Docker Hub y sigue al stable actual, así que súbelo cuando suba tu máquina. `WORKDIR /app` es calladamente estructural para nosotros: el poller lee `pulse.config.json` relativo a su directorio de trabajo, y como `COPY . .` deja el archivo en `/app` y el proceso de `CMD` arranca ahí, el mismo path relativo que funcionaba en tu host resuelve adentro de la caja; cambia el WORKDIR sin mover la config y construiste una imagen que arranca y de inmediato no puede encontrar sus propios targets. `EXPOSE 8080` es pura documentación, y esto importa: *no* abre ningún puerto. Publicar un puerto es una decisión de runtime que se toma con `-p`, que es el trabajo entero del paso 4. Y `CMD` usa la forma de array JSON para que tu binario corra directo como el único proceso del contenedor, sin shell en el medio.

3. **Constrúyela y lee las capas.** Desde la raíz del workspace:

```bash
docker build -t pulse-pollerd:naive .
```

El `-t` le pone tag al resultado; el `.` es el contexto de build que acabas de cercar. Las primerísimas líneas de salida son tu recibo del `.dockerignore`: una línea `transferring context` con un tamaño. Un workspace limpio transfiere en megabytes; si ves cientos de megabytes o peor, `target/` o `.git/` se filtraron al contexto, y arreglar el archivo de ignore ahora salva a cada build de acá al capstone. Después espera minutos: el pull de la imagen base, seguido de un `cargo build --release` en frío de todo el workspace adentro de la caja. Mira la estructura de la salida mientras corre: un paso numerado por cada instrucción que toca el sistema de archivos, `[1/4] FROM`, `[2/4] WORKDIR`, y así, cada uno volviéndose una capa, con el paso de cargo haciendo esencialmente toda la espera. (Seis instrucciones, un denominador de 4: `EXPOSE` y `CMD` son metadatos, así que BuildKit no les da paso numerado, la misma historia de 0B que cuenta `docker history` en el paso 5.) La cola debería terminar con algo como:

```text
 => exporting to image
 => => naming to docker.io/library/pulse-pollerd:naive
```

Guarda esa cola; el checkpoint la quiere. Después reconstruye de inmediato sin cambiar nada: `docker build -t pulse-pollerd:naive .` otra vez. Segundos, no minutos, con `CACHED` impreso al lado de los pasos. Las capas son la cache, y el log de build es donde la miras trabajar.

4. **Córrelo con la puerta abierta.** El poller escucha en 8080 *adentro* del contenedor, y adentro es una red distinta de la de tu host. `curl localhost:8080` en tu máquina golpea el puerto 8080 de tu host, donde no hay nada escuchando. La flag `-p` construye el puente:

```bash
docker run --rm -p 8080:8080 pulse-pollerd:naive
```

Lee `-p 8080:8080` como `host:container`: los requests al puerto 8080 del host se reenvían al 8080 del contenedor. Los dos números no tienen que coincidir, que es exactamente la costura de la que tira el challenge. (`--rm` borra el contenedor cuando sale, un hábito de cortesía que vale formar ahora.) Mientras corre, un `docker ps` pelado en otra terminal muestra el contenedor vivo con su mapeo de puertos impreso en la columna PORTS, que es donde miro primero cada vez que un servicio en contenedor "roto" cruza mi escritorio. Después, desde esa segunda terminal:

```bash
curl -s localhost:8080/status
```

Deberías conseguir el mismo JSON de `/status` que en m06-l1: estado por target, latencia, timestamp del último poll, ahora servido desde adentro de la caja. Si curl se cuelga o se resetea mientras los logs del contenedor se ven sanos, revisa dos sospechosos en orden. Primero, ¿de verdad pasaste `-p`? (La confesión de arriba es tuya para saltártela ahora.) Segundo, la dirección de bind: un servidor que hace bind a `127.0.0.1` adentro del contenedor solo se alcanza desde adentro del contenedor, que para un proceso es un lugar muy silencioso. El drop-in de m06-l1 hace bind a `0.0.0.0:8080`; si el tuyo dice `127.0.0.1`, cámbialo a `0.0.0.0` y reconstruye. En tu host esa distinción apenas importaba. En la caja lo es todo.

![Un request desde el host llega al poller solo a través del mapeo de puertos publicado y un bind a cero-punto-cero-punto-cero-punto-cero, con callejones sin salida cuando falta cualquiera de los dos.](assets/v06-flowchart.webp)

5. **Pésala y anota el número.** Detén el contenedor (Ctrl-C en su terminal), después:

```bash
docker images pulse-pollerd
```

Mira la columna SIZE. Tu binario del poller son unos pocos megabytes. La imagen en la que lo acabas de entregar va a sentarse en los gigabytes, tres órdenes de magnitud de empaque alrededor de la cosa que de verdad hiciste. Registra el número exacto en algún lado donde lo vuelvas a encontrar la lección que viene; volvemos a medir esta misma imagen después de la reconstrucción multi-stage, y quiero que tu antes sea tuyo.

Tampoco tomes el total por fe; pregúntale a la imagen misma dónde vive el peso:

```bash
docker history pulse-pollerd:naive
```

Una fila por capa, la más nueva primero, cada una con su propio tamaño. Léela contra el Dockerfile que escribiste: la fila `RUN cargo build` es la más pesada que causaste TÚ, alrededor de medio gigabyte de salida congelada de `target/`, mientras las filas del toolchain base debajo de ella son las verdaderas gigantes, cada una en la misma clase de medio gigabyte o más grande, porque un userland de Debian más rustc más cargo pesan más que el build de cualquier proyecto. La fila `COPY . .` lleva tu árbol de fuentes, y las filas de clase `EXPOSE`, `CMD` y `ENV` reportan todas 0B porque los metadatos no pesan nada. Esta es la misma contabilidad por capa que el log de build insinuaba, ahora con una balanza al lado.

¿De dónde vino el peso? Nada misterioso: la imagen final es cada capa que viste pasar volando. El toolchain completo de Rust desde `FROM`. Tu árbol de fuentes entero desde `COPY . .`. Y la más pesada, la capa `RUN cargo build`, que congeló el directorio `target/` entero, artefactos de dependencias y todo, adentro del sistema de archivos entregado. Nada de eso hace falta para *correr* el binario; todo eso hacía falta para *construir* el binario, y un Dockerfile de una sola etapa no puede notar la diferencia. Esa oración es la lección que viene.

![La imagen ingenua apila un toolchain completo, el árbol de fuentes y toda la cache de build alrededor del único binario chico que el contenedor de verdad necesita correr.](assets/v07-diagram.webp)

Eso es el lab: el poller responde desde adentro de una caja que cualquier host con Docker en la Tierra puede correr, y la caja está cómicamente pasada de peso. Las dos mitades de esa oración son el punto.

## Challenge

Solo, una costura, sin conceptos nuevos: haz que el puerto del poller sea configurable por variable de entorno. ¿Por qué variables de entorno, si el poller ya tiene un archivo de config perfectamente bueno? Porque una imagen es inmutable y una imagen debería servir a muchos deployments: misma caja, puerto distinto en tu laptop, en CI, y en cualquier host que con el tiempo lo corra. Las variables de entorno son la perilla que el operador de un contenedor puede girar sin reconstruir, que es por qué son el idioma de config de todo servicio en contenedor que vayas a leer alguna vez. Tres jugadas. En el arranque de `pulse-pollerd`, lee `POLLER_PORT` y recurre a 8080 cuando falta o no se puede parsear; `std::env::var("POLLER_PORT")` te da un arranque con forma de `Result`, y la cola de la cadena es una que ya escribiste: la demo de overflow de m05-l3 leía su argumento con `.nth(1).and_then(|s| s.parse().ok()).unwrap_or(250)`. Roba eso y adáptalo, porque dos cosas difieren y las dos se siguen de dónde viene el valor. `args().nth(1)` te da un `Option` mientras `env::var` te da un `Result`, así que el tuyo necesita un `.ok()` adelante para subirse a la misma vía; y el fallback es 8080, no 250. Aterrizar en `.ok().and_then(|s| s.parse().ok()).unwrap_or(8080)` es el punto del ejercicio, no la línea de partida. En el Dockerfile, agrega `ENV POLLER_PORT=8080` arriba del `CMD` para documentar el default en la imagen misma. Reconstruye, después córrelo mudado:

```bash
docker run --rm -e POLLER_PORT=9090 -p 9090:9090 pulse-pollerd:naive
```

Aceptación: `curl -s localhost:9090/status` responde, y correr sin `-e` sigue respondiendo en 8080. Si 9090 se cuelga, relee los dos sospechosos del paso 4; el segundo no puede lastimarte dos veces, pero el primero absolutamente sí.

## Checkpoint

Barrera sobre hacer, cuatro cosas para pegar: la salida `Hello from Docker!` de tu verificación de instalación; la cola del log de build ingenuo; una respuesta de `curl -s localhost:8080/status` del lado del host servida desde el contenedor; y la línea SIZE de `docker images` que registraste para la lección que viene. `docker login` debería haber dicho `Login Succeeded` en el camino, aunque nada te obligara a hacerlo.

Lo que ahora puedes hacer, concretamente: instalar y verificar un runtime de contenedores y explicar qué pagaste por él en tu SO; leer `docker ps -a`, un log de build y `docker history` como evidencia en vez de ruido; explicar imagen, contenedor, capa, registro y tag con un solo diagrama; y llevar cualquier binario del que seas dueño desde `cargo run` hasta responder por un puerto publicado desde adentro de una caja.

La recuperación de 30 segundos antes de cerrar la terminal: ¿cuál de los dos crea `docker run`, una imagen o un contenedor? (Un contenedor; las imágenes las crean únicamente los builds.) Y los cuatro sustantivos en un respiro: el registro guarda imágenes, la imagen es la plantilla inmutable en capas, la capa es una etapa de build cacheada, el contenedor es un proceso corriendo que viste ese sistema de archivos.

Si la instalación del runtime te peleó, y en algunas máquinas corporativas genuinamente lo hace, dime cuál SO y cuál runtime en el feedback del curso; la tabla de elección de runtime al principio de esta lección es la sección que más espero tener que ajustar por cohorte, y los reportes de fallas reales son cómo se gana el sueldo.

Entregaste una caja que funciona y que pesa gigabytes para un binario medido en megabytes, y tienes el número exacto anotado. La lección que viene: dos Dockerfiles, una idea. Los builds multi-stage recortan esa imagen en un orden de magnitud, con tu propio antes-y-después de Rust haciendo el argumento, y la misma técnica llevada directo a una imagen slim de Node para la flota. Ten ese número a mano.
