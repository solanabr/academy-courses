# Demuéstralo: los bytes on-chain coinciden con tu código fuente

La lección pasada generaste un cliente kit y publicaste el IDL on-chain, así que ahora cualquier cosa puede llamar al swap a través de un builder tipado. El cliente confía en el IDL. El IDL confía en el programa desplegado. ¿Y el programa desplegado? Ahora mismo le estás pidiendo a todo el mundo que te crea que los bytes que corren en el cluster son los bytes de tu repo.

Esa es la brecha que quiero cerrar. Cualquiera puede desplegar un programa, apuntar a un repo público de GitHub, y decir "este es el código fuente". Nada en un programa desplegado obliga a que esa afirmación sea verdadera. La cuenta on-chain es nada más un blob de bytecode sBPF. No lleva ningún link de vuelta a un commit, ninguna firma de un compilador, nada. Así que el lector de tu repo y el usuario de tu programa están confiando en dos artefactos distintos y esperando que sean el mismo.

Antes de cualquier teoría, demuéstrate a ti mismo que los bytes siquiera tienen una huella. El programa bajo el microscopio por el resto de esta lección es R4, el crate de swap `token_ticket_swap` cuyo IDL publicaste la lección pasada. Sácale el hash a tu build local ahora mismo:

```bash
# Install once (Rust toolchain required). Pin the version. To see what is newer:
#   cargo search solana-verify   (reads crates.io, i.e. what is available)
# `solana-verify --version` only tells you what YOU installed, so it is the
# confirmation step, not the freshness check.
cargo install solana-verify --version 0.5.1   # latest as of 2026-08-22, re-check before you pin

# Fingerprint your compiled program. This is the "before" of everything that follows.
solana-verify get-executable-hash target/deploy/token_ticket_swap.so
```

Ese comando imprime un único hash sha256 del ejecutable. Anótalo. Es una string de 64 caracteres que cambia si cambia siquiera una instrucción del binario. La lección entera está construida sobre una sola idea: si dos hashes producidos de forma independiente coinciden, los dos builds son idénticos byte a byte, y si difieren, algo se movió. Todo lo demás es plomería alrededor de esa comparación.

## Resumen

Vas a demostrar, en devnet, que un programa desplegado se construyó desde un código fuente específico con un toolchain fijado. Después lo vas a romper a propósito y vas a ver fallar la demostración. Después vas a mirar con dureza lo que la demostración no cubre, porque esa brecha es donde vive la confianza de verdad.

Acá está la forma de esto:

- **Un build verificable es un build determinista.** Compila el mismo código fuente en dos máquinas distintas con el CLI de Solana y puedes sacar dos binarios distintos, porque las rutas de build y las versiones de toolchain se filtran hacia los bytes. `solana-verify build` corre la compilación dentro de una imagen de Docker fijada, así que la salida es reproducible. Mismo código fuente más mismo toolchain fijado es igual a mismo hash, en cualquier máquina.
- **`verify-from-repo` es la demostración entera, y funciona en devnet.** Vuelve a construir tu programa desde un repo público dentro de esa imagen fijada, le saca el hash al resultado, trae el hash del programa on-chain desde el cluster que le nombres, y reporta coincidencia o desajuste. Apúntalo a devnet y demuestra tu deploy de devnet de forma local y trustless. No hace falta ningún tercero.
- **Una coincidencia demuestra procedencia, no seguridad.** Demuestra que los bytes desplegados vinieron exactamente de este código fuente con este toolchain. No dice nada sobre si el código fuente es correcto, y nada sobre quién controla los upgrades. Esas son demostraciones aparte que ya hiciste (tu auditoría, tu pasada de fuzz) o que vas a hacer (la autoridad).
- **La cadena de verificación entera se apoya en un solo guardián.** OtterSec construye el framework de Anchor, publica sus crates, y corre el registry de builds verificados contra el que Anchor verifica. Ese es un punto único de confianza real y honesto, y te voy a mostrar cómo verlo tú mismo en los metadatos propios de npm.
- **El submit al registry remoto y el traspaso de autoridad de Squads son exclusivos de mainnet.** Los vas a leer, narrados de punta a punta, claramente etiquetados como más allá del cluster de este curso. Nada de la sección cercada corre en devnet, y lo voy a decir cada vez.

El repliegue de esta lección: yo corro el ciclo completo de build, deploy y verificación de punta a punta en el lab, con la coincidencia de hash de devnet en pantalla, y cada comando que hay ahí es uno que corres contra tu propio program id y tu propio repo. El peldaño solo es el desajuste: cambia una línea, vuelve a construir, vuelve a desplegar, y haz que la demostración se ponga en rojo, después di en una oración qué te compra y qué no te compra un resultado verde. Esta lección es un build y un juicio, sin ningún problema de completion en el medio.

![Una cadena de cuatro cajas muestra al cliente kit confiando en el IDL publicado, que confía en el programa desplegado, cuyo link de vuelta al repo del código fuente queda sin demostrar.](assets/v01-flowchart.webp)

## Del código fuente a los bytes y de vuelta

Empieza con el dolor, porque no es obvio hasta que te lo topas. Construyes `token_ticket_swap` en tu laptop, tu compañero construye el mismo commit en la suya, y los dos archivos `.so` hashean distinto. Nadie editó el código fuente. ¿Entonces qué se movió?

El build normal del CLI de Solana empotra detalles específicos de la máquina dentro del binario. Rutas de build absolutas, la versión exacta del compilador, ordenamientos incidentales, todo eso puede filtrarse hacia los bytes. Esto no es una rareza de Solana, es cómo funciona la compilación nativa. La consecuencia es que "acá está mi código fuente" y "acá está mi binario" no se pueden verificar uno contra el otro salvo que todo el mundo se ponga de acuerdo, hasta la versión, en cómo se produjo el binario. Una comparación de hashes solo tiene sentido si el build es determinista.

Puede que primero eches mano de los arreglos obvios, y vale la pena ver por qué cada uno se queda corto, porque es eso lo que fuerza la solución de verdad. ¿Commitear un `Cargo.lock` y fijar cada dependencia? Necesario, pero no suficiente: dos máquinas con builds distintos de rustc siguen divergiendo, y el lockfile no dice nada sobre el compilador. ¿Publicar tus versiones exactas de rustc y de Solana en el README y pedirle a la gente que las igualen a mano? Mejor, pero ahora estás confiando en que cada verificador reconstruya un entorno paso a paso, y cualquier deriva en una biblioteca de sistema transitiva todavía puede mover un byte. El patrón es claro. El fijado parcial siempre deja una variable libre, y una sola variable libre rompe el hash. El único arreglo que las cierra todas de una vez es entregar el entorno mismo.

La bala de plata es Docker. `solana-verify build` corre la compilación dentro de una imagen fijada con un toolchain fijo y un entorno fijo, así que el mismo código fuente produce los mismos bytes sin importar de quién sea la máquina que está debajo. Ya no estás confiando en el build, estás confiando en los pins. Es la misma jugada que hace un ingeniero de puentes cuando especifica el grado exacto del acero en vez de "algún metal fuerte": el determinismo viene de quitar las variables libres, no de tener cuidado.

El costo es real y lo quiero sobre la mesa. Un build verificable es más lento que uno nativo, necesita Docker corriendo, y el primer build se baja una imagen grande. Estás comprando reproducibilidad con tiempo de build y una dependencia local más pesada. Para la iteración del día a día todavía usas el `cargo build-sbf` nativo y rápido. Echas mano del build verificable cuando estás a punto de desplegar algo en lo que la gente va a confiar.

![Un build normal bifurca un código fuente en dos hashes distintos en dos máquinas; un build de Docker fijado encauza el mismo código fuente hacia un único hash reproducible.](assets/v02-flowchart.webp)

Lo que nos trae a los pins, y a un número que quiero desactivar antes de que te confunda. Un build verificable registra el toolchain exacto que usó, y ese toolchain incluye una versión de Solana. Esa versión es el entorno de build de estos bytes. No es una afirmación sobre qué es la "Solana actual". Esos son dos hechos distintos y confundirlos es una trampa de verdad.

| Pin | Valor para el build de `token_ticket_swap` | Nota de frescura |
|---|---|---|
| `solana-verify` | 0.5.1 | La más nueva en crates.io al 2026-08-22; `cargo search solana-verify` para ver si eso se movió, `solana-verify --version` para confirmar lo que tienes |
| Docker | 27.x o más nuevo | El build falla rápido si el daemon no está corriendo |
| Anchor | 2.0.0-rc.1, git `otter-sec/anchor` rev `e4878b6d` (= tag `v2.0.0-rc.1`) | No es un release cortado que avm pueda bajar; fija el commit, no la rama, y vuelve a chequear en el build |
| Toolchain de build de Solana (dentro de la imagen) | 3.1.10 | PIN DE LOCAL-CI / DOCKER. Este es el ancla determinista de los bytes, NO una afirmación sobre la Solana actual |

Lee esa última fila dos veces. Solana 3.1.10 es el toolchain horneado dentro de este build para que el hash sea reproducible. El release estable actual de Solana es una cosa completamente distinta: Agave v4.2.1 al 2026-08-22 (vuelve a verificar, se mueve). La RC de V2 de Anchor apunta a la línea 3.x de Solana, así que un pin de build en 3.1.10 es exactamente lo correcto para estos bytes y no dice nada sobre el release de nodo más nuevo. Si alguna vez te encuentras leyendo una tabla de pins y pensando "entonces la Solana actual es 3.1", para. El pin es un hecho de build, el release es un hecho de red, y se separan a propósito.

Una nota rápida sobre herramientas, ya que vas a conocer las dos. Anchor entrega `anchor build --verifiable`, que envuelve la misma idea usando una imagen `solanafoundation/anchor:v<version>`, y el `anchor verify` de V2 tira directo por debajo hacia el binario `solana-verify`. Acá usamos `solana-verify` directo porque es la herramienta sobre la que el ecosistema se estandarizó para el paso de verificación, y porque mantener el build y la demostración en una sola herramienta quiere decir una sola versión que fijar y un solo set de flags que aprender.

Una arruga que nombrar antes de que compares cualquier hash, porque hace tropezar a la gente. El `.so` de tu disco y el programa como vive on-chain no están dispuestos de forma idéntica. Un programa actualizable se despliega a través de dos cuentas: una cuenta de programa, y una cuenta ProgramData aparte que de verdad guarda los bytes del ejecutable. `solana-verify get-executable-hash` le saca la huella a tu `.so` local. `solana-verify get-program-hash` le saca la huella al ejecutable traído de esa cuenta ProgramData on-chain. Las herramientas los normalizan para que los dos sean directamente comparables, que es exactamente por qué un deploy limpio deja los dos hashes iguales. Cuando no están de acuerdo y sabes que no editaste nada, la causa habitual es mundana: hasheaste un build fresco pero desplegaste un `.so` obsoleto de una compilación anterior. Vuelve a construir, vuelve a desplegar, vuelve a hashear, y se alinean.

Ahora la demostración en sí. `verify-from-repo` hace cuatro cosas:

```bash
# The frozen skeleton. Fill in your program id, your library name, and your repo URL.
# Two of those are flags; the repo URL is a positional argument at the end.
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  <REPO_URL>

# For a workspace with several programs, name the one you are proving:
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  --library-name token_ticket_swap \
  <REPO_URL>
```

Vuelve a construir el repo dentro de la imagen fijada, le saca el hash a ese binario fresco, trae el programa on-chain del cluster que está en `-u`, y hashea lo que está desplegado de verdad. Dos hashes, computados de forma independiente desde dos fuentes: tu código público y el cluster en vivo. Si son iguales, los bytes desplegados vinieron demostrablemente de ese código fuente con ese toolchain. Si difieren, no. Eso es todo. No hay ningún intermediario de confianza en este camino, que es exactamente por qué funciona en devnet: tú eres quien corre el rebuild y quien corre la comparación.

![Verify-from-repo hashea de forma independiente un rebuild de Docker del repo y el programa on-chain traído, y después compara los dos hashes localmente para sacar verificado o desajuste.](assets/v03-diagram.webp)

Vuélvelo concreto un segundo. Digamos que tu build local hashea a `9f3c...a1` y que `get-program-hash` sobre tu deploy de devnet devuelve el mismo `9f3c...a1`. Después `verify-from-repo` vuelve a construir desde el repo público, computa `9f3c...a1` una tercera vez, y lo compara contra el valor on-chain. Tres computaciones independientes, un solo valor, y cada una de ellas es algo que un escéptico puede reproducir sin pedirte nada. Ahora mueve un punto base de la comisión, vuelve a construir, y el hash local se vuelve `2b77...e0` mientras el repo sigue produciendo `9f3c...a1`. El desajuste es aritmética: bytes distintos, sha256 distinto, cero superposición.

Acá es donde me doy vuelta y nombro la parte honesta, porque una línea verde es seductora y miente por omisión si la dejas. Una coincidencia demuestra que los bytes en devnet se construyeron desde este código fuente con este toolchain. Demuestra procedencia. No demuestra que el código fuente sea seguro. Un programa perfectamente verificable puede drenar cada vault que tenga, porque la verificación nunca lee la lógica, solo le saca la huella a la salida compilada. La procedencia y la seguridad son ortogonales, y la razón por la que tu programa es confiable es el checklist de auditoría y la pasada de fuzz que corriste en el módulo de seguridad, no este hash. La verificación vuelve portables esos resultados. Deja que un extraño confirme que el código que auditaste es el código que está corriendo. Eso es enorme, y también es estrictamente menos que "seguro".

![Un build verificado demuestra que los bytes vinieron de este código fuente con el toolchain fijado y es re-ejecutable de forma trustless, pero no demuestra nada sobre ausencia de bugs, seguridad para otorgar permisos, o autoridad de upgrade.](assets/v04-comparison.webp)

## El guardián debajo de toda la cadena

Hasta acá la historia está limpia. Build determinista, comparación local, resultado trustless. Ahora quiero derivar la pregunta incómoda que un lector cuidadoso ya debería estar formando, porque "no confíes, verifica" corta para los dos lados y no te voy a entregar la herramienta sin la salvedad.

La pregunta es en qué estás confiando exactamente cuando verificas un programa de Anchor. Se siente como nada, porque el build es determinista, y esa lectura es verdadera para la comparación y falsa para el entorno. Sigue la cadena. Confías en que la imagen de Docker fijada sea un toolchain honesto. Confías en los crates de Anchor contra los que compilaste. Y si usas el registry remoto, confías en quien sea que lo corra. Que esos puntos de confianza existan no tiene nada de raro; todo toolchain los tiene. Lo que importa es que acá se colapsan en una sola parte.

OtterSec construye el framework de Anchor, publica sus paquetes, y corre el registry de builds verificados contra el que verifican las herramientas propias de Anchor. Un solo guardián abarca el framework, los artefactos y el registry. Esto no es un rumor y no tienes que creerme, que es el punto entero: los dos registries lo registran, y lo puedes leer de cualquiera de los dos.

Sé preciso sobre cuál comando demuestra cuál mitad, porque las dos son artefactos aparte. La caminata de npm de abajo lee el campo `repository` en `@anchor-lang/core`, el cliente de TypeScript, y lo que demuestra es *hacia dónde se movió el repositorio del código fuente*. Tu programa no compila contra ese paquete; compila contra los crates de Rust. Para esos, pregúntale directo a crates.io:

```bash
# The Rust side: who owns the crate your program actually links against.
cargo owner --list anchor-lang
cargo info anchor-lang                      # the `repository:` row names the source repo
# (cargo search won't do here: it prints only name/version/description,
#  never the repository field — `cargo info` is the command that reads it.)

# The npm side: the repository field, version by version, is where the two
# custody transfers are legible.
npm view @anchor-lang/core repository.url
npm view @anchor-lang/core@1.1.1 repository.url   # the version where it changes
```

El campo repository de `@anchor-lang/core` apunta a otter-sec a partir de la versión 1.1.1, publicada el 2026-06-25. Recorre la historia y puedes ver moverse la custodia: el campo va arrastrándose de coral-xyz a solana-foundation a otter-sec, sin ningún anuncio en ninguna parte. Dos transferencias silenciosas de custodia, registradas solo en un campo de metadatos que casi nadie lee. Cuando rastreé esto por primera vez lo hice exactamente como lo acabas de hacer tú, un `npm view` a la vez, porque yo tampoco me lo creía de una afirmación de segunda mano. Esa es la costura que quiero que te quedes: verifica la procedencia de tu herramienta de procedencia.

![El campo repository de npm para @anchor-lang/core camina de coral-xyz a solana-foundation a otter-sec, con dos transferencias no anunciadas y otter-sec tomando el control en v1.1.1 el 2026-06-25.](assets/v05-timeline.webp)

¿Comparado con qué, eso sí? Esa es la pregunta que mantiene esto honesto en vez de alarmista. Comparado con ninguna verificación en absoluto, donde le crees a un extraño que su deploy coincide con su repo, un solo guardián bien considerado corriendo un pipeline reproducible es un paso grande hacia arriba. Comparado con una cadena de suministro completamente diversificada, varias partes independientes construyendo el framework, publicando los crates y corriendo registries que compiten entre sí, es un paso corto. Las dos comparaciones son verdaderas al mismo tiempo. La respuesta correcta no es desconfiar de la herramienta sino conocer la forma exacta de aquello en lo que estás confiando, para que si la custodia alguna vez cambia de manos otra vez lo notes, igual que acabas de notar las últimas dos transferencias.

Un solo guardián custodia el framework, publica los artefactos, corre el registry contra el que verifica Anchor, y firma con GPG el tag v2 bajo la clave trixter-osec. Eso es un montón de la cadena de suministro apoyándose en una sola parte competente y bien considerada. "Bien considerada" está haciendo trabajo de verdad en esa oración, y no es lo mismo que "trustless". Un build verificable te quita la necesidad de confiar en quien construyó tu programa específico. No te quita la necesidad de confiar en quien construyó el framework. Los dos hechos son verdaderos a la vez, y un ingeniero de seguridad sostiene los dos sin pestañear.

![OtterSec se sienta en el centro de tres radios, construyendo el framework, publicando los crates, y corriendo el registry de builds verificados, así que un solo guardián abarca toda la cadena de suministro.](assets/v06-diagram.webp)

## Exclusivo de mainnet, y de solo lectura acá

Dos piezas más del flujo de trabajo real pertenecen a tu cabeza aunque no las vayas a correr esta lección. Las estoy cercando explícitamente. **Todo lo de esta sección es exclusivo de mainnet y está más allá del cluster de este curso.** Lo vas a leer, no lo vas a ejecutar.

La primera es el submit al registry remoto. Junto al `verify-from-repo` local que acabas de correr, `solana-verify` puede encolar un job de verificación con los workers remotos de OtterSec, que escriben un registro en el registry on-chain. La vieja flag `--remote` de `verify-from-repo` está deprecada y ahora nada más imprime el camino actual: sube tu PDA de verify con la autoridad de upgrade del programa, y después `solana-verify remote submit-job --program-id <PROGRAM_ID> --uploader <UPLOADER>`. Ese job toma en algún punto entre uno y treinta minutos y escribe un registro de verificación que los exploradores y las billeteras leen para mostrar la pequeña insignia de "verified". **El submit-job remoto es exclusivo de mainnet.** Si lo apuntas a un programa de devnet esperando un resultado, no vas a sacar ninguno, y la razón no es un deploy que falta ni un daemon de Docker parado, es que el camino del registry solo cubre mainnet. En devnet, el `verify-from-repo` local es la demostración, punto final.

Sé preciso sobre qué agrega el job remoto, porque es una capa de conveniencia, no una demostración más fuerte. La demostración trustless es la local que ya corriste: cualquiera puede volver a construir y comparar. Lo que compra el registry es descubribilidad. OtterSec corre el build en su infraestructura, escribe el resultado en un registro on-chain, y cada explorador y cada billetera que lee ese registro puede mostrar una insignia de verificado sin que cada usuario vuelva a construir tu programa él mismo. Así que el camino remoto canjea un poco más de confianza en el guardián por mucho más alcance, y es el mismo OtterSec que ya conociste corriendo el registry. En mainnet ese canje en general vale la pena. En devnet simplemente no se ofrece, que es la razón entera de que el job vuelva vacío ahí.

La segunda es el traspaso de la autoridad de upgrade, y acá es donde la verificación se topa con la gobernanza. La autoridad de upgrade es la cuenta que tiene permiso para reemplazar los bytes de un programa. Un programa recién desplegado tiene una, normalmente un solo keypair, que puede cambiar el ejecutable a voluntad. Un build verificado con una autoridad caliente de una sola clave es un programa que es demostrablemente este código fuente ahora mismo y que podría ser silenciosamente distinto mañana. El objetivo final recomendado es mover esa autoridad a un multisig de Squads v4, un programa que requiere firmas de M de N miembros antes de autorizar una acción, así que ninguna clave sola puede empujar un upgrade por su cuenta. **Este flujo es exclusivo de mainnet para este curso; lo estoy narrando, no corriendo.** El programa Squads v4 es `SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf` (vuelve a verificarlo antes de que actúes sobre él alguna vez), y el traspaso tiene un orden específico:

![El traspaso de Squads v4, exclusivo de mainnet, corre desde crear el Squad, a escribir un buffer, a transferir la autoridad de upgrade, a una propuesta aprobada hasta el umbral y ejecutada.](assets/v07-flowchart.webp)

El orden no es arbitrario, y hacerlo al revés es la trampa clásica. Escribes los bytes nuevos en un buffer y le pones al Squad la autoridad de ese buffer antes de entregar la autoridad de upgrade del programa mismo. Si transferiste la autoridad del programa al Squad primero y solo entonces descubriste que el buffer era propiedad de la clave equivocada, quedarías atorado necesitando una propuesta de multisig para arreglar un error al que el multisig todavía no puede llegar. Buffer primero, programa segundo, ejecutar al final. Cada paso te deja en algún lugar del que todavía puedes recuperarte, hasta justo antes de la aprobación final.

La salvedad honesta se acumula de dos maneras, y las dos pertenecen a la mesa antes de que alguien toque mainnet. Primero, el titular recomendado de la autoridad es él mismo no actualizable: el programa Squads v4 es inmutable desde noviembre de 2024, lo cual es un feature, el multisig en el que confías no te lo pueden cambiar por debajo, y también un hecho que deberías decir en voz alta. Segundo, el objetivo final más allá del multisig es poner la autoridad del programa en `None`, volviendo inmutable tu propio programa. Esa es la garantía más fuerte que les puedes ofrecer a los usuarios y es irreversible. No hay vuelta atrás. Una jugada de autoridad que no puedes deshacer es un canje, no una victoria gratis. Vuélvelo inmutable después de haberlo verificado, nunca antes, porque la inmutabilidad congela lo que haya ahí, seguro o no.

![La escalera de autoridad corre desde un solo keypair a un multisig de Squads v4 a inmutable, canjeando control por certeza en cada peldaño, con el peldaño final irreversible.](assets/v08-comparison.webp)

## Lab: demuestra el swap en devnet

Hora de correr todo el asunto. El repliegue de la ayuda es explícito acá. Yo corro el ciclo completo del camino verde de punta a punta, del build a la verificación en devnet, con los comandos y los checkpoints escritos. Después tú corres el completion sobre tu propio deploy llenando las flags, y el desajuste solo es tuyo en el challenge.

Primero, confirma tu toolchain. Cada herramienta muestra su instalación la primera vez que la necesitas.

```bash
# solana-verify (installed above): confirm the version you pinned
solana-verify --version

# Docker must be running; solana-verify builds inside it.
# Install Docker Desktop or the engine from the official docs at docker.com/get-started.
docker info >/dev/null && echo "docker up" || echo "start docker first"

# Anchor V2 RC, if you have not already installed it for this course. `avm install`
# 404s on the RC (no GitHub Release cut for the v2 tag, so its binary is missing), so
# the documented channel is the git build you ran in m01-l2.
# macOS needs LTO off or the release build blows up at link; harmless elsewhere.
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --rev e4878b6d anchor-cli --locked --force
anchor --version   # expect anchor-cli 2.0.0-rc.1 (freshness 2026-08-22; RC, re-check)
```

Fíjate en el `--rev` donde cada lección anterior escribió `--tag v2.0.0-rc.1`. Esos dos resuelven al *mismo* código fuente — `v2.0.0-rc.1` es un tag anotado cuyo commit es `e4878b6d`, lo que puedes confirmar tú mismo:

```bash
git ls-remote https://github.com/otter-sec/anchor.git 'refs/tags/v2.0.0-rc.1*'
# 2f77733f...  refs/tags/v2.0.0-rc.1       <- the tag object
# e4878b6d...  refs/tags/v2.0.0-rc.1^{}    <- the commit it points at
```

¿Entonces por qué escribirlo de la forma más difícil acá? Porque un tag es un *ref* y un commit es un *hecho*. Un tag se puede mover o borrar y volver a cortar en un commit distinto; el hash `e4878b6d` nombra un solo objeto inmutable y nada más puede responder nunca por él. En todos los demás lugares de este curso el tag es lo bastante preciso, y se lee mejor. Acá el entregable entero es un hash que un extraño reproduce, así que el toolchain se nombra en la granularidad más apretada que existe — la misma disciplina que aplica `solana-verify` cuando fija su imagen de build de stock en vez de dejarla flotando. Si `anchor --version` reporta algo distinto de la RC después de esto, el commit se reescribió y vuelves a fijar desde el tag.

También vale enunciarlo sin adornos, ya que la tabla de pins lo matiza: el CLI de Anchor que corres localmente no está dentro del sobre determinista. `solana-verify build` compila dentro de la imagen de Docker fijada, usando el toolchain de esa imagen, así que el hash es una función de la imagen y de tu código fuente, no de tu `anchor` de host. Fijar tu CLI local te mantiene *a ti* consistente entre lecciones. Fijar la imagen es lo que hace funcionar la demostración.

Checkpoint: `solana-verify --version` imprime `solana-verify 0.5.1`, y la línea de docker imprime `docker up`. Si docker no está arriba, arréglalo ahora, porque el paso del build va a fallar con un error de daemon, no con un error de código fuente, y eso etiqueta mal el problema.

1. **Construye de forma determinista.** Desde la raíz del workspace:

```bash
solana-verify build --library-name token_ticket_swap
```

Esto levanta la imagen de Docker fijada y compila `token_ticket_swap` dentro de ella. La primera corrida se baja la imagen y es lenta. Checkpoint: termina con `target/deploy/token_ticket_swap.so` escrito y sin error.

2. **Sácale la huella al binario determinista.**

```bash
solana-verify get-executable-hash target/deploy/token_ticket_swap.so
```

Checkpoint: sacas un sha256 de 64 caracteres. Este es el hash que el verificador va a reproducir de forma independiente.

3. **Apunta a devnet y financia el deploy.** Un deploy de programa no es gratis, así que asegúrate de que el CLI esté en devnet con SOL para gastar:

```bash
solana config set -u devnet
solana airdrop 2        # devnet faucet; retry if the faucet rate-limits you
solana balance
```

Checkpoint: `solana balance` muestra al menos un par de SOL. Si el deploy después dice "insufficient funds", eso es un problema de balance, no un problema de build, y acá es donde lo arreglas.

4. **Despliega a devnet.** Ya desplegaste el swap la lección pasada, así que este es un upgrade en el lugar, no un programa nuevo: pasa el keypair de programa del workspace para que los bytes deterministas aterricen en la misma dirección a la que ya apuntan tu IDL publicado y tu cliente generado.

```bash
solana program deploy target/deploy/token_ticket_swap.so -u devnet \
  --program-id target/deploy/token_ticket_swap-keypair.json
```

Checkpoint: el comando imprime tu `Program Id`, el mismo de la lección pasada. Ese es tu `<SWAP_PROGRAM_ID>`. Confirma que el hash on-chain coincide con tu hash local:

```bash
solana-verify get-program-hash -u devnet <SWAP_PROGRAM_ID>
```

Checkpoint: este hash es igual al del paso 2. Si difieren, desplegaste un binario distinto del que hasheaste, normalmente un `.so` obsoleto, así que vuelve a construir y a desplegar antes de seguir.

5. **Verifica desde el repo, contra devnet.** Commitea y sube tu código fuente a un repo público primero, y después:

```bash
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  --library-name token_ticket_swap \
  <REPO_URL>
```

Checkpoint: reporta una coincidencia, una línea "verified" para el programa de devnet. Esa única línea es el objetivo de evaluación de esta lección. Ya demostraste, de forma local y trustless, que los bytes en devnet se construyeron desde tu código fuente público con el toolchain fijado.

![Una tabla de checkpoints que empareja cada paso del lab con cómo se ve el éxito y el arreglo específico si sale mal, terminando con verify-from-repo reportando una coincidencia en devnet.](assets/v09-table.webp)

## Challenge: haz que la demostración se ponga en rojo, después di qué quiere decir el verde

El peldaño solo tiene dos partes, y las dos son el punto.

Primero, rómpelo. Cambia exactamente una línea del código fuente de `token_ticket_swap`. La opción más limpia es la guarda de slippage que escribiste en la lección del swap, porque cambia el comportamiento y por lo tanto los bytes sin tocar la interfaz, las cuentas, ni el IDL:

```diff
- require!(out >= min_out, SwapError::SlippageExceeded);
+ require!(out > min_out, SwapError::SlippageExceeded);   // one character, on purpose
```

**No commitees ni subas esa edición.** El ejercicio entero depende de que el repo y la cadena no estén de acuerdo, y el paso 5 del lab te dijo que subieras tu código fuente, así que el reflejo está justo ahí. Deja el cambio local. Después vuelve a construir con `solana-verify build --library-name token_ticket_swap`, vuelve a desplegar ese binario editado al *mismo* `<SWAP_PROGRAM_ID>` (el upgrade con `--program-id target/deploy/token_ticket_swap-keypair.json` del paso 4, así que estás reemplazando los bytes que acabas de demostrar en vez de acuñar un programa fresco), y corre el mismo `verify-from-repo` contra tu repo público todavía sin editar. Si sacas una coincidencia en vez de un desajuste, subiste. Los bytes on-chain ahora rechazan un fill exacto de `min_out`; el repo todavía lo acepta. Aceptación: `verify-from-repo` reporta un MISMATCH. Después revierte la línea, vuelve a construir, vuelve a desplegar, y míralo volver a una coincidencia. Ya viste los dos resultados con tus propias manos, que es la única forma en que la línea verde quiere decir algo alguna vez.

Segundo, escribe una oración. En tus propias palabras, enuncia qué demuestra y qué no demuestra una coincidencia verificada. Una respuesta que pasa nombra las dos mitades: demuestra que los bytes desplegados vinieron de este código fuente exacto con el toolchain fijado, y no demuestra que el código fuente sea seguro ni que la autoridad de upgrade esté trabada. Si tu oración solo tiene la primera mitad, aprendiste la herramienta y te perdiste la lección.

Tercero, haz la única jugada de autoridad que devnet *sí* puede ejecutar, porque el módulo anterior te prometió que ibas a razonar sobre quién tiene la clave de upgrade y leer un flujo exclusivo de mainnet no es eso. Tu swap de devnet ahora mismo tiene una autoridad de upgrade de un solo keypair: la tuya. Míralo, muévelo, y mira otra vez:

```bash
solana program show <SWAP_PROGRAM_ID> -u devnet     # read the Authority line
solana-keygen new -o /tmp/new-authority.json --no-bip39-passphrase
# Pass the new authority as a KEYPAIR, not a pubkey: the CLI requires the
# incoming authority to co-sign the handoff, the guard that stops you from
# typo-ing your program away to an address nobody holds. (A bare pubkey makes
# the command fail on a missing signature unless you add
# --skip-new-upgrade-authority-signer-check — a flag for hardware-wallet flows,
# and exactly the guard you should not rehearse turning off.)
solana program set-upgrade-authority <SWAP_PROGRAM_ID> -u devnet \
  --new-upgrade-authority /tmp/new-authority.json
solana program show <SWAP_PROGRAM_ID> -u devnet     # read it again
```

Ese es el primer peldaño de la escalera, ejecutado en vez de narrado: la clave que puede reemplazar silenciosamente tus bytes verificados es ahora una clave que elegiste a propósito. El peldaño de Squads y el peldaño `None` se sientan arriba de él y son exclusivos de mainnet para este curso, pero la forma es la misma jugada cada vez. Quédate con `/tmp/new-authority.json` si quieres seguir actualizando este programa, y fíjate en lo que acaba de pasar si no: le entregaste tu programa a un keypair en `/tmp`, que es un ensayo chico, seguro e instructivo de exactamente el error irreversible del que advierte la escalera.

Finalmente, en una línea cada uno, identifica cuáles dos pasos de esta lección son exclusivos de mainnet y no se pueden demostrar en devnet. Si nombraste el submit-job al registry remoto y el traspaso de autoridad de Squads, tienes el alcance correcto.

## Antes de seguir adelante

Chéquate contra las cuatro formas en que esto sale mal en la práctica, porque son las cuatro que está buscando la evaluación.

¿Tu `verify-from-repo` corrió de verdad contra devnet con tu propio program id, y reportó una coincidencia real? Leer los flujos narrados no cuenta como la demostración. La barrera es una coincidencia de hash que produjiste tú, más el desajuste que produjiste después de editar una línea. Si solo leíste, todavía no pasaste.

¿El `verify-from-repo` de tu terminal usó `--library-name`? En un workspace de un solo programa es opcional y en el tuyo no, porque `quarter-vault` tiene tres programas ya — cinco una vez que aterrice el capstone — y la herramienta no tiene forma de adivinar a cuál de ellos pertenece tu program id.

¿Mantuviste en su carril la versión de Solana que está en la tabla de pins? Hecho de build, no hecho de red — si esa distinción no es instantánea ya, vuelve a leer la caminata por la tabla de pins de arriba antes de seguir.

¿Y probaste el submit-job remoto en devnet y te confundiste cuando no devolvió nada? Ese silencio es exactamente lo que predice el alcance de cluster, porque el camino del registry remoto es exclusivo de mainnet. En devnet, el `verify-from-repo` local es la demostración entera, y la razón por la que el job falla ahí es su alcance de cluster, no un deploy que falta y no un daemon parado.

Último check, y este es del módulo, no de la lección: cierra las notas y di toda la secuencia de entrega de memoria, en orden. El IDL fuera del programa, el IDL sobre la cadena, el cliente fuera del IDL, kit fijado al major del peer, build determinista, deploy a devnet, `verify-from-repo`. Si se pierde un paso, ese es el que hay que volver a correr, no volver a leer.

Entregaste el swap y lo demostraste byte por byte, y le miraste de frente al único guardián sobre el que se apoya toda la cadena de verificación. El próximo módulo le arrancas el framework a un peldaño por completo. Vuelves a construir R2, el quarter-vault del módulo 3, sobre pinocchio crudo: sin macros, sin struct de cuentas generada, así que puedes ver exactamente qué te estaba escribiendo Anchor debajo de todo esto. La verificación demostró que los bytes coincidían con el código fuente. Pinocchio te muestra qué estaba escondiendo el código fuente.

Nos vemos en el módulo de pinocchio.
