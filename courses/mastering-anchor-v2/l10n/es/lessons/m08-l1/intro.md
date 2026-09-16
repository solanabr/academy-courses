# Entrega un cliente: el IDL, Program Metadata, y un cliente Codama/kit

La lección pasada corriste el checklist de auditoría contra el swap con un número de línea en cada fila, y después manejaste `anchor fuzz` hasta que un bug sembrado produjo un artefacto de crash que repetiste, parchaste y volviste a fuzzear limpio. R4 está endurecido. Sobrevive los canjes que le tiras y revierte los que debe. Y es completamente inalcanzable por cualquiera que no seas tú, sentado en esta terminal, corriendo un banco de pruebas de Rust. Un programa endurecido que solo tus propias pruebas pueden llamar es un vault cerrado con la llave todavía en tu bolsillo.

Así que el trabajo ahora es un cliente. Un llamador de verdad. Y acá está la trampa en la que caminas en el momento en que recurres a uno, así que déjame hacerte sentirla antes de explicarla. Abre una terminal en tu workspace y corre estos dos comandos:

```bash
npm view @solana/kit version
npm view @solana-program/token@0.15.0 peerDependencies
```

El primero te dice el `@solana/kit` más nuevo de npm, que al 2026-08-22 es `8.0.0`, y el segundo te dice qué pide de verdad `@solana-program/token`: `{ '@solana/kit': '^7.0.0' }`. Lee esas dos salidas una al lado de la otra. El kit más nuevo es la única versión que tus dependencias no quieren. Instala el número que npm llama `latest` y tu `npm install` tira un error de peer-dependency antes de que escribas una sola línea de código de cliente. Esa brecha, y cómo entregar igual sin mentirte a ti mismo sobre ella, es la mayor parte de esta lección.

## Resumen

Vas a darle a R4 un llamador. Cuatro jugadas, en orden de dependencia, porque cada una necesita la anterior:

1. Construye el **IDL** del programa, el contrato JSON que describe cada instrucción y cada cuenta.
2. **Publica ese IDL on-chain** a través del Program Metadata Program, para que cualquier cliente pueda traer tu interfaz desde el cluster en vez de desde tu repositorio.
3. **Genera un cliente `@solana/kit` tipado** a partir del IDL con `anchor codama`, porque no existe un paquete de TypeScript oficial de Anchor V2 y la cosa que parece uno no lo es.
4. **Fija kit correctamente** y manda exactamente un swap a través del builder generado contra tu deploy de devnet.

El repliegue de esta lección corre así: los pasos 1 a 4 del Lab están completamente trabajados, cada comando real y cada checkpoint verificable. El paso 5, el tubo de send de kit, es un problema de Completion: recibes el esqueleto entero con las tres líneas estructurales en blanco. Después el Challenge es Solo, un archivo autocontenido donde resuelves qué versión de kit fijar a partir de un rango de peer y armas la llamada sin ninguna respuesta trabajada adelante.

Una nota que colorea todo: Anchor V2 es una candidata a release de semanas de edad, y las herramientas de cliente a su alrededor se mueven más rápido que el framework. Cada número de versión de acá lleva la fecha en que lo verifiqué. Cuando llegues a esta lección, vuelve a correr los dos comandos de arriba. Los números se van a haber movido. La *regla* no, y la regla es la cosa por la que estás acá.

## El camino de entregar-un-cliente

Antes de que manejes la ruta, mira el mapa. Las cuatro jugadas no son independientes. El IDL es la entrada para publicarlo y la entrada para generar el cliente. El cliente generado es la entrada para el send. Sácalas de orden y vas a estar regenerando un cliente contra un IDL que nunca actualizaste, que es la forma más común en que un cliente generado entrega una llamada que ya no coincide con el programa.

![El IDL de anchor idl build alimenta tanto la publicación on-chain como la generación de Codama; el cliente generado fuerza el pin de kit, y el pin vuelve posible el send.](assets/v01-flowchart.webp)

Nota la forma. Publicar (B) y generar (C) se ramifican los dos desde el IDL, y son independientes entre sí. Puedes generar un cliente sin publicar nunca el IDL on-chain, y puedes publicar sin generar. Hacemos los dos porque sirven a llamadores distintos: publicar sirve a *cualquiera*, generar sirve a *ti*. Mantén esa división en mente, es la respuesta a dos de las preguntas de verificación del final.

### El IDL es el contrato, y v2 deliberadamente lo dejó en paz

Un archivo IDL (Interface Description Language) es una descripción en JSON de tu programa: su dirección, sus instrucciones con sus argumentos y sus cuentas requeridas, sus layouts de cuenta, sus códigos de error, y los discriminators de cada uno. Es la versión legible por máquina de todo lo que de otra forma tendrías que leer de `lib.rs` a mano. Constrúyelo con el CLI:

```bash
anchor idl build --program-name token_ticket_swap -o target/idl/token_ticket_swap.json
```

Acá está la parte que importa para un curso de framework, porque la expectativa corre para el otro lado. Una reescritura `no_std` desde cero suena como si debiera haber forkeado el formato de interfaz, y no lo hizo: el IDL de v2 mantiene los **mismos discriminators de 8 bytes** que v1 por defecto, y la *spec* del IDL en sí está intacta — el código fuente de la spec en el tag fijado v2.0.0-rc.1 es byte por byte idéntico al de la baseline 1.1.2. Esa identidad incluye los campos `serialization` y `repr`, los que describen cómo serializan los tipos y cómo se disponen los enums en memoria: los dos anteceden a V2 (llegaron con la reescritura de spec de la era 0.30) y las dos líneas los llevan en el mismo lugar del mismo archivo. El delta de verdad de V2 en este módulo no está en el JSON para nada; está en cómo el JSON llega al mundo — el camino de publicación del CLI a través del Program Metadata Program, que recorre la próxima sección. La estabilidad es la feature: una llamada armada contra un IDL de la era v1 sigue apuntando a la instrucción correcta debajo de un programa v2, y un IDL de v2 entra en cualquier herramienta que ya lea la spec moderna.

Si que el discriminator siga estable suena como una nota al pie, acuérdate de lo que hiciste cuando computaste un preimage de discriminator a mano antes en este curso: hasheaste el nombre con namespace de la instrucción y viste los primeros ocho bytes volverse el selector sobre el que rutea el runtime. Esos ocho bytes exactos son lo que el IDL lleva en el array `discriminator` de cada instrucción. Por eso una llamada de la era v1 sigue aterrizando contra un programa v2: el selector no se movió, y tampoco la descripción envuelta a su alrededor. Un cliente generado lee esos bytes directo del IDL, así que nunca vuelves a tipear un discriminator a mano, y nunca te equivocas de dedo con uno dentro de una llamada que calladamente apunta a la instrucción equivocada.

![La spec del IDL es idéntica entre la baseline 1.x y el tag de v2 — discriminators, campos de serialization y de repr todos sin cambios; un generador de cliente que ignora serialization/repr es seguro solo sobre tipos borsh default en cualquiera de las dos líneas.](assets/v02-annotated-code.webp)

Así que este es un sondeo de tiempo de escritura, no un hecho que puedas congelar de mí. Antes de que confíes en un cliente generado para un programa que usa serialización no default o un `repr` propio — campos que la spec moderna lleva desde la era 0.30, en las dos líneas — confirma que tu versión de generador los consume. Para el swap que estás entregando, `Pool` es una struct borsh simple y `swap_arcade_for_tickets` toma dos `u64`, así que estás seguramente dentro de lo que maneja cada generador. En el momento en que entregues un programa que no lo está, ese sondeo es tuyo.

### Pon el IDL on-chain para que cualquiera pueda llamarte

Tienes un archivo JSON. Un archivo JSON en tu repositorio ayuda exactamente a la gente que tiene tu repositorio. Para dejar que una billetera, un explorador, o el script de un extraño resuelva tu interfaz desde nada más que el id de tu programa, el IDL tiene que vivir en la cadena.

Piénsalo como una ciudad trata un edificio. Cualquiera puede dibujar un plano, pero el plano que *cuenta*, el que un contratista puede sacar y usar para construir, es el que está archivado en la municipalidad bajo la dirección del edificio, enmendable solo por el propietario de registro. La versión de Solana de ese archivador es el **Program Metadata Program**. Anchor soltó sus propias instrucciones de IDL incorporadas allá en 1.0, y V2 hereda esa remoción: el IDL se guarda a través de este programa, en una dirección determinística derivada del id de tu programa, escribible solo por la autoridad de upgrade del programa. Así que cuando tipees `anchor idl init` en un momento, los verbos son viejos pero la maquinaria no — los comandos de la era 0.x del mismo nombre escribían en las propias cuentas de IDL on-chain de Anchor, el mecanismo que 1.0 sacó, mientras este CLI reusa los nombres de verbo como un front end para el Program Metadata Program, que es por lo que enseñarlos acá no resucita el camino retirado.

![El Program Metadata Program guarda el IDL en un PDA canónico derivado del id del programa; cualquiera puede leerlo, pero solo la autoridad de upgrade puede escribirlo o darle upgrade.](assets/v03-diagram.webp)

Los comandos son el subcomando `idl` del CLI de Anchor. La primera publicación crea la cuenta on-chain; las ediciones posteriores le dan upgrade:

```bash
# First time: create the on-chain IDL account and write the IDL into it
anchor idl init -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet

# Later, after any program change that touches the interface
anchor idl upgrade -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet

# Prove it: fetch the IDL back from the chain, by program id alone
anchor idl fetch -o fetched.json <YOUR_SWAP_PROGRAM_ID> --provider.cluster devnet
```

Por debajo del capó estos escriben a través del Program Metadata Program. Checkpoint: después de `idl init`, ese último comando de `idl fetch` saca tu IDL del cluster hacia `fetched.json` usando nada más que el id del programa. Diffealo contra `target/idl/token_ticket_swap.json` y debería coincidir. Si `idl init` falla diciendo que la cuenta ya existe, el culpable de siempre no es alguna sesión olvidada tuya: el `anchor deploy` de la RC sube el IDL por defecto siempre que `target/idl/<name>.json` existe, así que un deploy simple ya publicó por ti. O pasa `--no-idl` a la hora del deploy para mantener publicar como un paso explícito (lo que hace el lab de este curso), o acepta la subida automática y usa `idl upgrade` para cada edición posterior. Si falla en la autoridad, la billetera con la que estás firmando no es la autoridad de upgrade del programa, y solo esa clave puede escribir.

Hay un trade-off de verdad en publicar, y prefiero que lo escuches de mí y no de un hilo de soporte. La cuenta de IDL cuesta alquiler, y la copia on-chain es actual solo en la medida de su última publicación. En el flujo de `--no-idl` que corre este lab, eso quiere decir tu último `idl upgrade`: publica una vez, cambia el programa, olvídate de darle upgrade al IDL, y ahora cada cliente que confía en la cadena arma llamadas contra una interfaz obsoleta. El IDL on-chain es un compromiso de mantenerlo fresco, no un tira-y-olvida.

### El estado honesto del cliente de TypeScript de Anchor

Ahora el cliente en sí, y acá es donde he visto a más gente perder una tarde que en cualquier otro lado de la historia de V2. Déjame llevarte escalera arriba de la forma en que de verdad la subirías, peldaño equivocado primero.

Ya escribiste clientes de Anchor antes. En el mundo 0.x importabas el paquete de TypeScript de Anchor, le entregabas tu IDL, y llamabas `program.methods.swap(...)`. Así que la jugada obvia es encontrar la versión V2 de ese paquete y hacer lo mismo. Buscas, encuentras `@anchor-lang/core`, latest en npm `1.1.2` al 2026-08-22, es el sucesor renombrado del viejo `@coral-xyz/anchor`, y se ve exactamente correcto.

No es correcto. `@anchor-lang/core` en `1.1.2` todavía depende de `@solana/web3.js` v1. Es el cliente de la era v1 vistiendo un nombre nuevo. Anchor publica un cliente de TypeScript, pero no uno de V2: `@anchor-lang/core` es real, oficial, y está en la línea v1, y nada lo reemplazó para V2. Recurre a la cosa que parece oficial y calladamente te fijaste de vuelta a web3.js v1, la línea de SDK de la que V2 existe para salir.

La jugada siguiente más obvia es escribir el cliente a mano: codifica el discriminator, serializa en borsh `amountIn` y `minOut`, arma la lista de `AccountMeta` en el orden exacto que espera el programa, deriva el PDA del pool tú mismo. Funciona. Es también cómo una llamada de swap termina llevando las cuentas en el orden equivocado o un off-by-one en los account metas, una clase de bug que compila limpio y falla solo on-chain. El cliente hecho a mano es boilerplate que reescribes, y en el que te equivocas sutilmente, para cada instrucción, y ese costo es una parte grande de por qué existe la generación first-party de clientes.

Llévate una disidencia hacia el camino que estamos a punto de tomar, porque apunta a ese camino y no al que acabas de rechazar. ChewingGlass, en la discusión #3742 de Anchor: "Codama doesn't resolve has_ones. Anchor does... I still feel quite boilerplate-y fetching and passing heaps of accounts in codama. Boilerplate kills new devs." Esa es una acusación justa y vale sostenerla mientras usas la herramienta. Un cliente kit generado todavía te hace traer y pasar cuentas que una llamada de `program.methods` de la era v1 habría resuelto desde constraints por ti. La generación compra corrección en el orden de las cuentas, en las flags y en la codificación. No compra de vuelta la ergonomía del cliente de TypeScript viejo. Toma la corrección, y quédate molesto con el resto.

Así que el camino de verdad, el que entrega el CLI de Anchor, es **Codama**. Codama es un generador de clientes: lee un IDL y emite un cliente tipado. El CLI de Anchor lo envuelve en dos subcomandos, así que no instalas ni configuras Codama por separado, el CLI fija la versión que usa (`CODAMA_VERSION = 1.6.0` adentro del CLI al 2026-08-22) y lo maneja por ti.

![@anchor-lang/core es la línea de SDK equivocada, hacerlo a mano invita bugs silenciosos de cuentas, y anchor codama generate produce un cliente kit cuyo único costo de verdad es disciplina de regeneración.](assets/v04-comparison.webp)

Dos comandos. El primero convierte el IDL de Anchor en el propio árbol de IDL de Codama. El segundo corre esa conversión in-process y después renderiza el cliente:

```bash
# Convert the Anchor IDL to a Codama IDL (inspect it if you like)
anchor codama convert target/idl/token_ticket_swap.json --out codama-idl.json

# Convert + render a JavaScript/TypeScript client into clients/
anchor codama generate -l js -p clients target/idl/token_ticket_swap.json
```

Dos comandos, no uno, y la división es deliberada. `convert` es la mitad honesta que puedes inspeccionar: escribe el IDL de Codama en un archivo que puedes abrir y diffear, así que cuando una llamada generada se ve mal puedes ver si la culpa está en la conversión o en tu programa. `generate` corre esa misma conversión in-process y después se la entrega a los renderizadores de Codama, así que en el trabajo del día a día corres solo `generate`. Recurre a `convert` el día en que un builder generado te sorprenda y quieras leer qué piensa Codama que es tu programa.

Lo que aterriza debajo de `clients/` no es un blob. La flag `-p` nombra un directorio base y el CLI escribe cada lenguaje en `<base>/<language>`, así que `-p clients -l js` renderiza hacia `clients/js/`. Adentro, Codama emite un directorio que puedes leer, una carpeta por tipo de cosa de tu programa:

![El cliente generado es un directorio de instructions, accounts, pdas, types, programs y errors; quien llama usa el builder de instrucción de instructions, el buscador de PDA del pool de pdas, y el decodificador de cuenta de accounts.](assets/v05-diagram.webp)

El builder se nombra según tu instrucción. `swap_arcade_for_tickets` se vuelve `getSwapArcadeForTicketsInstructionAsync`. El sufijo `-Async` es la convención de Codama para la variante que resuelve lo que puede por ti: deriva el PDA de `pool` a partir de sus seeds y llena direcciones de programa default, así que pasas las cuentas que solo tú puedes saber (el trader, los mints, las cuentas de token de reserva y de trader) y él arma el resto. Ese es el punto entero de la generación. El orden de las cuentas, el discriminator, la codificación borsh de `amountIn` y `minOut`, la derivación del PDA, todo sale de tu IDL en vez de salir de tu memoria.

Pon la razón en la página, porque acá es donde hacerlo a mano de verdad muerde. El `#[derive(Accounts)]` del swap lista nueve cuentas en un orden fijo, y el runtime las hace coincidir posicionalmente, por slot, no por nombre. Hazlo a mano y estás retipeando ese orden hacia un array de `AccountMeta` de memoria, donde intercambiar `reserve_arcade` y `reserve_ticket`, o marcar `trader` como solo lectura cuando tiene que firmar, compila limpio y falla solo cuando el canje pega en la cadena. El builder generado lee el orden y las flags de escribible/firmante del IDL y te pide cada cuenta por nombre. Las dos reservas que nunca puedes confundir llegan como `reserveArcade` y `reserveTicket`, etiquetadas, en el único lugar donde un typo de otra forma sería invisible.

![Hacer a mano las nueve cuentas posicionales del swap falla en silencio cuando dos slots de reserva se intercambian, mientras el builder generado toma cuentas por nombre y deriva el PDA del pool él mismo.](assets/v06-comparison.webp)

### El pin de kit: coincide con tus peers, nunca persigas latest

Ahora de vuelta a la trampa que sentiste al principio de esta lección, porque ahora tienes las piezas para entenderla. El cliente generado es un cliente `@solana/kit`, y se apoya en los paquetes `@solana-program/*` (`@solana-program/system`, `@solana-program/token`) para las piezas de system y de token. Esos paquetes declaran una peer dependency sobre kit. Al 2026-08-22, `@solana-program/system@0.13.0` y `@solana-program/token@0.15.0` hacen peer los dos sobre `@solana/kit` `^7.0.0`. Y el kit `latest` de npm es `8.0.0`, publicado el 2026-08-21.

Así que el kit más nuevo y el kit que quieren tus dependencias son majors distintos. Esto no es una casualidad de una semana mala, es la textura normal de un SDK que se mueve rápido: la biblioteca central entrega un major nuevo antes que el ecosistema que hace peer sobre ella. Mira la semana en que pasó.

![El 2026-08-21 el web3.js heredado todavía le ganaba a kit 1,882,726 contra 1,738,844 en descargas semanales, y kit entregó 8.0.0 el mismo día, mientras el ecosistema todavía hacía peer sobre kit ^7.](assets/v07-chart.webp)

Las dos mitades de ese gráfico son verdad en la misma semana: el cruce de descargas dice que kit es hacia donde va el ecosistema, y los rangos de peer dicen que no persigas su número de versión.

Así que la regla durable, la única cosa para llevarte de esta lección si no te llevas nada más: **fija `@solana/kit` al major que declaran tus dependencias de `@solana-program`, nunca a `latest`.** Es la misma ley que enseñan dos cursos hermanos desde sus propios asientos — Payments y Commerce la aplica por workspace, donde dos workspaces de un mismo repositorio legítimamente fijan majors de kit distintos porque cada uno coincide con sus propios peers, y Rust & TypeScript Fundamentals la deriva de cómo resuelven los rangos de peer de npm para empezar. Hoy ese major es 7. Instálalo explícitamente:

```bash
# Freshness: verified 2026-08-22. kit latest is 8.0.0, but the @solana-program
# packages below peer on ^7, so we pin ^7. Re-run `npm view ... peerDependencies`
# when you reach this; the numbers move, the rule does not.
npm install @solana/kit@^7 @solana-program/system@0.13.0 @solana-program/token@0.15.0
```

¿Por qué no fijar solo `latest` y dejar que npm lo resuelva? Por lo que hace npm con una peer dependency. Cuando `@solana-program/token@0.15.0` declara `peerDependencies: { '@solana/kit': '^7.0.0' }`, le está diciendo a tu gestor de paquetes "solo voy a correr contra un kit 7". Instala kit 8 al lado y npm no puede satisfacer esa restricción, así que se detiene con un error de peer-conflict `ERESOLVE` antes de que nada compile. `latest` es un blanco en movimiento que una subida de major de kit puede convertir en exactamente ese conflicto de la noche a la mañana, y lo hizo, el día en que se entregó kit 8. Fija el número en el que tus peers están de acuerdo y tu instalación es reproducible hasta que *tú* decidas moverla, a propósito, después de haber verificado que los rangos de peer también se movieron. Ese es el trato que estás haciendo en todas partes de este curso: renuncias a "siempre el más nuevo" y obtienes "siempre resoluble".

### Una interfaz más, ya en tus manos: declare_program!

Hay un segundo consumidor del IDL que ya encontraste del otro lado. En v1, un programa podía consumir el IDL de *otro* programa en tiempo de compilación a través de `declare_program!`, generando una interfaz de CPI a partir de un IDL vendoreado — un programa Anchor llamando a otro por su interfaz publicada en vez de por versiones de código que coinciden. Eso es exactamente lo que tu escrow le viene haciendo al vault desde m04-l3, y está verificado funcionando sobre la línea 2.0.0-rc.1 que fija este curso: el JSON que has estado cosechando hacia `idls/` es un IDL haciendo la mitad de tiempo de compilación del trabajo que describe la mitad del lado del cliente de esta lección. El capstone se apoya en él con cuatro peldaños de ancho en m09-l3.

Vale saberlo mientras estás acá: la razón por la que el curso consume programas por IDL en vez de por código no es gusto. La feature `cpi` de nivel de código del scaffold tope en un programa consumido por binario sobre la RC — un segundo colisiona a la hora del link sobre un símbolo de despacho sin mangling — así que el camino del IDL es tanto el propio mecanismo de V2 como el único que escala al salón de cuatro peldaños. Dónde se usa la misma jugada en serio más allá de este curso: consumir el IDL publicado de un protocolo vivo directamente, que es territorio del curso de DeFi y RWA Engineering.

## El Lab

Chequeo de repliegue antes de empezar: los pasos 1 a 4 están trabajados, los corres y ves cada checkpoint ponerse verde. El paso 5 es el problema de Completion, el tubo de kit con las mismas tres líneas en blanco para que llenes. Después el Challenge es Solo.

Instalación de herramientas en el primer uso. Construiste la RC de Anchor V2 desde su canal git allá en R0, así que esto es un confirmar, no una instalación nueva. Acuérdate de por qué acá no hay línea de `avm use`: no se cortó ningún GitHub Release para el tag v2, así que el binario precompilado que descarga `avm install` no está ahí y el fetch da 404.

```bash
# If the RC is not on this machine, rebuild it from the documented channel:
# cargo install --git https://github.com/otter-sec/anchor.git \
#   --tag v2.0.0-rc.1 anchor-cli --locked --force
which anchor       # ~/.cargo/bin/anchor either way (the avm shim lives at the same path) — only proves it's on PATH
anchor --version   # the real check: anchor-cli 2.0.0-rc.1 (freshness 2026-08-22; RC, re-check)
node --version     # anchor codama drives @codama/cli via npx, so Node must be present
```

**1. Haz el deploy del swap, y después construye y publica su IDL.** R4 solo corrió en LiteSVM y en Surfpool, así que necesita estar en el cluster antes de que nada de acá funcione: la cuenta de IDL está llaveada a un id de programa real, y un cliente necesita una dirección para llamar. `anchor deploy` usa el keypair de programa del workspace, así que el id que imprime sigue siendo tuyo por el resto del curso.

```bash
# -p: deploy ONLY the swap. A bare `anchor deploy` loops every program in the
#     workspace, prints one Program Id per program, and pays ProgramData rent for
#     each — far more than one airdrop covers at this point in the course.
# --no-idl: the RC uploads target/idl/<name>.json during deploy by default; skip
#     that here so the publish stays the explicit `idl init` two lines down.
anchor deploy -p token_ticket_swap --no-idl --provider.cluster devnet
                                          # prints Program Id -> <YOUR_SWAP_PROGRAM_ID>
                                          # short on funds? solana airdrop 2 -u devnet, retry
anchor idl build --program-name token_ticket_swap -o target/idl/token_ticket_swap.json
anchor idl init -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet
anchor idl fetch -o fetched.json <YOUR_SWAP_PROGRAM_ID> --provider.cluster devnet
```

Checkpoint: `anchor deploy` imprime un Program Id, regístralo en tu archivo de pins como `<YOUR_SWAP_PROGRAM_ID>`; después `fetched.json` existe y diffea limpio contra `target/idl/token_ticket_swap.json`. Cualquier cliente de la tierra puede ahora resolver la interfaz de tu swap desde el id del programa solo.

**1b. Levanta un workspace de node para el cliente.** El cliente generado es TypeScript, y nada en un workspace de Anchor crea uno. Hazlo ahora, en la raíz del repositorio, para que `npx tsc` tenga algo que mirar:

```bash
npm init -y
cat > tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "target": "es2022",
    "module": "es2022",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["clients/**/*.ts", "app/**/*.ts"]
}
JSON
mkdir -p app
```

Checkpoint: `tsconfig.json` y `package.json` existen en la raíz del repositorio. Ese `include` es lo que hace que el `tsc --noEmit` del paso 4 verifique los tipos del cliente generado en vez de reportar "No inputs were found", y `app/` es donde aterriza tu script de send en el paso 5, que es lo que hace que su import de `../clients/js` resuelva.

**2. Genera el cliente kit.** Convierte y renderiza en un comando.

```bash
anchor codama generate -l js -p clients target/idl/token_ticket_swap.json
```

Checkpoint: `clients/js/instructions/` contiene un builder `getSwapArcadeForTicketsInstructionAsync`, `clients/js/pdas/` contiene `findPoolPda`, `clients/js/accounts/` contiene `fetchPool`, y `clients/js/errors/` contiene tus variantes de `SwapError` — el renderizador JS de Codama separa los buscadores de PDA en su propia carpeta `pdas/`, así que no salgas a cazar el buscador debajo de `accounts/`. Si la carpeta está vacía, `npx` no pudo traer `@codama/cli`, verifica que Node esté en tu PATH y vuelve a correrlo.

**3. Fija kit al major de los peers.** Instala las dependencias de runtime del cliente, fijadas al major sobre el que hacen peer tus paquetes `@solana-program`.

```bash
npm install @solana/kit@^7 @solana-program/system@0.13.0 @solana-program/token@0.15.0
```

Checkpoint: la instalación completa sin error de peer-dependency. Si tira uno que menciona `@solana/kit@8`, algo tiró `latest`, arréglalo de vuelta a `^7`.

**4. Verifica los tipos del cliente generado.** Esta es la barrera de verificación de la mitad de cliente entera de la lección. TypeScript es la única herramienta que este lab todavía no instaló, así que agrégala como dependencia de dev en vez de dejar que `npx` traiga una flotante:

```bash
npm install -D typescript@5.9.2   # pinned on purpose; use whatever your workspace already pins
npx tsc --noEmit
```

Checkpoint: cero errores. Un cliente tipado que no pasa la verificación de tipos no es un cliente, es un pasivo. Este es el mismo `tsc --noEmit` que barrera la tarea, así que ponerlo verde acá es ponerlo verde allá.

**5. Manda un swap (problema de Completion).** Acá está el tubo de send de kit, y va en `app/send-swap.ts`. Tres líneas estructurales están en blanco. Llénalas: quien paga la tarifa es quien llama, el lifetime es el blockhash reciente, y la única instrucción agregada es la que arma tu cliente generado. Este es el esqueleto exacto que sigue el send.

Antes de que pueda correr, seis de esas direcciones tienen que existir en devnet, y nada hasta ahora las creó. Haz eso primero: el mismo CLI de `spl-token` que usaste para la lectura de Token-2022 del módulo 5, después una llamada al propio `init_pool` del swap, después una lectura de dos líneas para descubrir qué creó `init_pool`, y después dos líneas más de `spl-token` que necesitan lo que la lectura te dijo.

```bash
solana config set --url devnet

# The two mints, and the trader's token accounts.
spl-token create-token --decimals 6            # -> <ARCADE_MINT>
spl-token create-token --decimals 6            # -> <TICKET_MINT>
spl-token create-account <ARCADE_MINT>         # -> <TRADER_ARCADE_ATA>
spl-token create-account <TICKET_MINT>         # -> <TRADER_TICKET_ATA>
spl-token mint <ARCADE_MINT> 1000              # give the trader something to swap
# Read that last line precisely: `spl-token mint` takes an optional third argument,
# the recipient TOKEN ACCOUNT, and its default is "the mint authority's own ATA" —
# you. Nothing you have typed so far puts a single token inside the pool.

# The pool and its two reserves. `init_pool` is the instruction you wrote in the
# swap lab; the generated client has a builder for it too, so send it the same way
# the pipe below sends the swap — same pipe, different builder. It prints nothing,
# which is exactly why the next step exists: you cannot fund what you cannot name.
```

`init_pool` creó dos cuentas de token de reserva y no te dijo ninguna de las dos direcciones, y las próximas dos líneas de shell necesitan las dos. Léelas del registro del pool con la otra mitad del cliente que acabas de generar — `findPoolPda` de `pdas/`, `fetchPool` de `accounts/`, los dos re-exportados desde la raíz del cliente. Sin borsh manual, sin explorer:

```typescript
// app/read-pool.ts — run this once, right here, before you fund anything.
import { createSolanaRpc } from '@solana/kit';
import { fetchPool, findPoolPda } from '../clients/js';

const rpc = createSolanaRpc('https://api.devnet.solana.com');
const [poolPda] = await findPoolPda();
const pool = await fetchPool(rpc, poolPda);
// pool.data.arcadeMint / .ticketMint / .arcadeReserve / .ticketReserve / .bump, all typed
console.log(pool.data.arcadeReserve, pool.data.ticketReserve);
```

Esas dos direcciones impresas son el `<POOL_ARCADE_RESERVE>` y el `<POOL_TICKET_RESERVE>` de abajo — el arreglo de auditoría de m07-l3 es lo que hizo que el pool guardara las dos direcciones de reserva al lado de los mints y del bump, y esta es la lección donde lo cobras. El decode dobla como tu prueba de que `init_pool` aterrizó: si `pool.data.bump` se lee de vuelta como el bump canónico guardado y los dos mints coinciden con lo que desplegaste, el registro es real. Es también la mitad de lectura del cliente generado haciendo trabajo de verdad en vez de demo — la necesitaste para avanzar, no para admirarla.

```bash
# Now fund the reserves, because `init_pool` CREATES the two reserve token accounts
# and leaves them empty, and R4 has no deposit instruction — you never wrote one.
# An empty reserve makes swap_out return 0 and the `require!(out > 0, ZeroOutput)`
# guard reject every trade, so skip these two lines and the swap below cannot land.
# You are still both mints' authority, so mint straight in by naming the reserve as
# the recipient: the third argument the `mint 1000` line above deliberately left off.
spl-token mint <ARCADE_MINT> 1 <POOL_ARCADE_RESERVE>   # 1.000000 -> 1_000_000 base units
spl-token mint <TICKET_MINT> 1 <POOL_TICKET_RESERVE>   # the same, so the pool starts balanced
```

Esas dos líneas de mint dejan al pool sosteniendo 1,000,000 / 1,000,000 unidades base, que es deliberadamente el par de reserva que usó el ejemplo trabajado de m05-l2: un `amountIn` de `10_000` cotiza 9,871 tickets de salida, cómodamente libre del piso de `minOut` de `9_800` del send de abajo. Siembra otra profundidad y recomputa ese piso antes de mandar, o tu propia guarda de slippage te va a rechazar — que es la guarda funcionando, no un bug. También: `secretKey` en la firma de abajo son los 64 bytes de tu archivo de keypair de devnet, que puedes cargar con `new Uint8Array(JSON.parse(fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf8')))`.

```typescript
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromBytes,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
  getSignatureFromTransaction,
  assertIsTransactionWithBlockhashLifetime,
  address,
} from '@solana/kit';
import { getSwapArcadeForTicketsInstructionAsync } from '../clients/js';

async function sendSwap(secretKey: Uint8Array): Promise<string> {
  const rpc = createSolanaRpc('https://api.devnet.solana.com');
  const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');
  const trader = await createKeyPairSignerFromBytes(secretKey);

  // The generated async builder derives the pool PDA and default programs for us.
  const swapIx = await getSwapArcadeForTicketsInstructionAsync({
    trader,
    mintArcade: address('<ARCADE_MINT>'),
    mintTicket: address('<TICKET_MINT>'),
    reserveArcade: address('<POOL_ARCADE_RESERVE>'),
    reserveTicket: address('<POOL_TICKET_RESERVE>'),
    traderArcade: address('<TRADER_ARCADE_ATA>'),
    traderTicket: address('<TRADER_TICKET_ATA>'),
    amountIn: 10_000n,
    minOut: 9_800n,
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => /* FILL: set the fee payer to the trader */ m,
    (m) => /* FILL: set the lifetime to latestBlockhash */ m,
    (m) => /* FILL: append the generated swapIx */ m,
  );

  const signedTx = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signedTx);

  const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  await sendAndConfirm(signedTx, { commitment: 'confirmed' });
  return getSignatureFromTransaction(signedTx);
}
```

Los tres rellenos, para que te verifiques una vez que los hayas intentado: `(m) => setTransactionMessageFeePayerSigner(trader, m)`, después `(m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m)`, después `(m) => appendTransactionMessageInstruction(swapIx, m)`. Nota que `amountIn` y `minOut` son `bigint`s, no números; ese sufijo `n` es cómo kit carga un `u64` sin perder precisión por encima de 2^53.

Dos especificidades de kit que vale nombrar mientras están adelante. `sendAndConfirmTransactionFactory` toma tanto `rpc` como `rpcSubscriptions`, porque kit confirma escuchando en un websocket por la firma en vez de consultando, que es por lo que creaste un cliente de subscriptions al lado del de RPC. Y `assertIsTransactionWithBlockhashLifetime` no es ceremonia: es una guarda de tipos que se niega a compilar el send a menos que el mensaje de verdad lleve un lifetime de blockhash, así que olvidarse de la línea del lifetime se vuelve un error de tipos en tu escritorio en vez de una transacción caída en devnet. Lograr que el canje *aterrice* de forma confiable bajo carga real es un oficio separado, y queda más allá de la frontera del lado cliente. Acá estás demostrando que la llamada es bien formada y confirmable, no afinándola para un líder congestionado.

![Una transacción de kit se arma definiendo quien paga la tarifa, el lifetime de blockhash y la instrucción, y después se firma, se protege, se manda y se confirma, y su firma se lee de vuelta.](assets/v08-flowchart.webp)

Checkpoint para el lab entero: `sendSwap` devuelve una firma, y esa firma resuelve en un explorador de devnet como un swap confirmado. Eso es un llamador, distinto de ti, moviendo R4. La llave del vault está fuera de tu bolsillo.

Y la mitad de lectura ya está demostrada, porque no podrías haber llegado hasta acá sin ella: `findPoolPda` derivó el pool y `fetchPool` lo decodificó allá en el paso 5, tipado y sin borsh manual, y las dos direcciones de reserva que te entregó son las cuentas que fondeaste y contra las que acabas de operar. El builder escribe llamadas, el decodificador lee estado, y los dos salieron del único IDL.

## El Challenge

Ahora Solo, sin ninguna respuesta trabajada adelante. El challenge es `wire-kit-swap-client`, un archivo de TypeScript autocontenido (`starter.ts` y su `tests.json` debajo del directorio de challenge de esta lección). Pela el send hasta las dos decisiones que de verdad son tuyas, para que califique determinísticamente sin RPC y sin firma.

Tu `planSwapClient` toma seis argumentos posicionales, en este orden: `owner`, la dirección de quien llama; los dos números del canje, `amountIn` y `minOut` (los dos `bigint`); `recentBlockhash`; `splPeer`, una string como `"^7.0.0"` (el rango que declara `@solana-program/token`); y `kitLatest`, una string como `"8.0.0"` (el `latest` de npm, la trampa). Debajo, en el mismo archivo, está `swapInstruction(owner, amountIn, minOut)`, un builder provisto que hace de sustituto del generado — trátalo como un dado. Devuelves cuatro cosas:

- `pinnedKitMajor`: el major de `splPeer`, incluso cuando `kitLatest` es más nuevo. El pin viene del rango de peer, nunca de latest.
- `feePayer`: quien llama (`owner`).
- `lifetime`: el blockhash reciente.
- `instructions`: exactamente una, el swap del builder provisto `swapInstruction(owner, amountIn, minOut)`, que lleva tanto `amountIn` como `minOut`.

No vuelvas a armar la instrucción de swap a mano, esa es la razón entera por la que generaste un cliente. Llama al builder provisto. Aceptación: `pinnedKitMajor` es igual al major de `splPeer` incluso cuando `kitLatest` es `9.9.9`, quien paga la tarifa es el owner y el lifetime es el blockhash reciente, y exactamente una instrucción se agrega llevando tanto el monto de entrada como el piso de slippage. Las pruebas verifican cada uno de los cuatro campos devueltos por separado, así que una respuesta a medio llenar te dice exactamente qué subobjetivo te perdiste en vez de solo ponerse roja.

## Antes de seguir adelante

Detente y verifica la forma de respuesta que debías producir: un directorio `clients/` que pasa la verificación de tipos bajo `npx tsc --noEmit`, un `package.json` que fija `@solana/kit` en `^7`, y una firma de swap de devnet confirmada del builder generado. Si tienes esas tres, entregaste un cliente honestamente sobre herramientas de frontera. Si solo leíste esta lección, no lo hiciste, esto es un build, y el swap tiene que aterrizar.

El trato que hiciste, dicho sin adornos para que lo lleves hacia adelante: un cliente generado es actual solo en la medida del IDL que publicaste y de la versión de Codama que fijaste. Sáltate un `idl upgrade` después de un cambio de programa y los llamadores arman contra una interfaz obsoleta. Persigue `latest` en kit y rompes el grafo de peers de `@solana-program` el día en que se entrega un major nuevo. Cambiaste control escrito a mano por disciplina de regeneración, y esa disciplina es la fecha `verified` de cada pin.

Una dirección más, para que sepas a dónde va este cliente después. Lograr que una transacción bien formada quede *armada* es esta lección. Lograr que *aterrice* de forma confiable bajo carga, tarifas de prioridad, reintentos, el arte entero del aterrizaje de transacciones, queda más allá de la frontera del lado cliente. Este curso es dueño del framework y de la interfaz, y entrega la estrategia de aterrizaje al otro lado de esa línea.

Puedes llamar al swap ahora. ¿Pero alguien puede demostrar que los bytes que corren en devnet se armaron desde tu código, y no se cambiaron por otra cosa después de que mirabas para otro lado? La próxima lección corres un build determinístico en Docker, le haces deploy, y verificas los bytes on-chain contra tu código con un comando, y después te encuentras con el hecho incómodo de que la cadena entera de verify se apoya en un solo guardián.
