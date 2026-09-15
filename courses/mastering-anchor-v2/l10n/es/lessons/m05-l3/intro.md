# Token-2022 desde el asiento del framework

La lección pasada entregaste el swap de token-a-ticket: matemática de producto constante sobre dos reservas de token SPL, el invariante sosteniéndose a través de un canje, la guarda de slippage disparando en el momento en que un fill entró por debajo de la cotización. Funciona. Demostraste que funciona. Después un jugador se acerca a la máquina arcade con un token contra el que no probaste.

Su mint es Token-2022. Lleva una tarifa de transferencia y un transfer hook. Y acá está la pregunta honesta, la que hace girar esta lección entera: ¿tu llamada a `transfer_checked` todavía funciona, entrega menos en silencio, o falla de entrada?

No respondas de memoria. Abre una terminal, porque vamos a apuntar un lector minúsculo a un mint Token-2022 de verdad y dejar que nos lo cuente. Esa es la jugada entera de esta lección: corre, observa, razona. Empiézalo ahora, antes de seguir leyendo, con un comando:

```bash
# PYUSD, a live Token-2022 mint. A classic SPL mint is 82 bytes. Watch this one.
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta
```

Lee la línea `Length:` que imprime. Si dice cualquier cosa distinta de 82, ya tienes la consecuencia uno en las manos, y el resto de esta lección es el por qué. Vas a leer un mint vivo y reportar dos cosas de vuelta, el número de bytes que de verdad ocupa y si una transferencia contra él necesita cuentas extra que ahora mismo no estás mandando. Si puedes reportar esas dos cosas desde la lectura y no desde un post de blog, entendiste el asiento en el que estás sentado.

## Resumen

Tu swap ya apunta a los dos programas de token. Lo construiste sobre `InterfaceAccount<T>` y `transfer_checked` allá en la lección del swap, y esa no fue una elección descartable: el mismo camino de código ya corre contra el Token clásico de SPL y contra Token-2022 sin una rama. Así que el sistema de tipos está listo. Esa es la buena noticia, y es gratis.

La mala noticia es que Token-2022 calladamente mueve dos piezas de trabajo real hacia tu programa, y el sistema de tipos no te va a recordar ninguna de las dos:

- **Los tamaños de cuenta dejan de ser constantes.** Un mint o una cuenta de token de Token-2022 no tiene largo fijo. Las extensiones lo hacen crecer. Asume el tamaño clásico y tus lecturas se van más allá del final de los datos.
- **Un mint con hook quiere decir que una transferencia puede necesitar cuentas extra.** Si el mint declara un transfer hook, una transferencia contra él corre un segundo programa, y ese programa necesita sus propias cuentas reenviadas en tu instrucción. Sáltatelas y la transferencia no se completa.

Esa es la lección entera. Dos consecuencias, observadas en vivo, razonadas desde el asiento del programa. Voy a recorrer la primera lectura contigo paso a paso en el lab. El Challenge del final lo corres por tu cuenta, contra un mint que eliges tú. Ese es el repliegue: guiado ahora, Solo en quince minutos.

Una frontera dura, dicha de entrada para que ninguno de los dos se desvíe. Esta lección enseña Token-2022 como *consecuencia para tu programa*. No te enseña a diseñar extensiones, y no recorre la interfaz de transfer hook. El curso de Digital Assets recorre la interfaz de transfer hook de punta a punta y es dueño de la profundidad de estándares de extensión — su módulo 3 es donde escribes el hook que esta lección solo lee. Cuando peguemos en esa línea, paramos y apuntamos hacia allá. A propósito.

## Qué cambia de verdad para tu programa

Lo que no cambia es más de lo que supondrías, así que toma eso primero. Un mint Token-2022 usa el *mismo layout base* que un mint clásico de SPL. El mismo campo de supply, los mismos decimals, los mismos slots de autoridad, los mismos primeros 82 bytes. Una cuenta de token Token-2022 comparte también la base clásica de 165 bytes. Si Token-2022 hubiera reescrito el layout base, cada billetera y cada indexador de la red se habría roto el primer día. No lo hizo. Mantuvo la base y agregó datos nuevos después de ella.

Ese agregado es la cosa que hay que internalizar. Todo lo que Token-2022 suma vive en una sección TLV pegada a la cola de la cuenta: tipo, largo, valor, repetido. Un mint que opta por una tarifa de transferencia, un puntero de metadatos y un transfer hook lleva tres entradas TLV después de su base. Un mint que no opta por nada no lleva ninguna y se lee exactamente como un mint clásico.

![Los layouts base y la primitiva transfer_checked vienen del SPL clásico; el largo total de la cuenta y la necesidad de cuentas extra de transferencia tienen que observarse en vivo sobre Token-2022.](assets/v01-comparison.png)

### Consecuencia uno: el tamaño ahora es dato, no constante

Un número lo vuelve concreto. Un mint clásico de SPL tiene 82 bytes. Punto final, siempre, para siempre. Una cuenta de token clásica tiene 165 bytes, el mismo trato. Esas son constantes que puedes dejar hardcoded y no volver a pensar nunca más.

Ahora la lectura en vivo que estás a punto de correr tú mismo: el mint PYUSD de mainnet, un mint Token-2022 de verdad, ocupa 866 bytes ahora mismo. La misma base, los mismos 82 bytes al frente, más una cola TLV que lleva sus extensiones. Eso es más de diez veces el tamaño clásico, y no es un número mágico que quiera que memorices. Es un número que *lees*, porque un mint Token-2022 distinto lleva un conjunto distinto de extensiones y aterriza en un largo distinto.

![Un mint clásico de SPL tiene 82 bytes y una cuenta de token clásica 165 bytes, los dos fijos, mientras el mint PYUSD Token-2022 en vivo tiene 866 bytes por su cola de extensiones.](assets/v02-chart.png)

¿Por qué el estándar lo hizo de esta forma, agregando una cola auto-descriptiva en vez de solo ensanchar la struct? Porque no puedes renumerar un formato binario que el ecosistema entero ya está parseando. Los campos base quedan en offsets fijos de los que dependen miles de clientes. Así que las features nuevas no podían ir *adentro* del layout viejo, tenían que ir *después* de él, cada una anunciando su propio tipo y su propio largo para que un lector pueda caminar la cola sin un schema horneado de antemano. La elegancia es real. El costo es igual de real y cae sobre ti: el largo ahora es un valor que llevan los datos, no una constante en la que puedas confiar desde el header. Léelo.

Esta es también exactamente la razón por la que te dije que no respondieras la pregunta de apertura de memoria. El largo de un mint no es estable ni para un solo mint a lo largo del tiempo: una autoridad puede agregar una extensión después de la emisión y la cuenta se vuelve más larga, así que un número que era correcto cuando escribiste tu programa puede estar equivocado cuando corre. La disciplina es aburrida y correcta: lee la cuenta viva, razona desde lo que dice.

El framework sí ayuda acá, y ayuda más que lo que ayudaba el camino clásico. En tu swap, el mint y los vaults están tipados como `anchor_spl::token_interface::InterfaceAccount<Mint>` e `InterfaceAccount<TokenAccount>`. Ese tipo acepta una cuenta cuyo dueño es *cualquiera de los dos*, el programa Token clásico o Token-2022, y deserializa los campos base correctamente en los dos. Cuando alcanzas más allá de la base, hacia la cola de extensiones, la segunda capa es `anchor_spl::extensions`, que parsea desde la cuenta las structs de extensión TLV de tamaño fijo que están soportadas. Dos capas, una para la base y una para la cola.

![Un mint Token-2022 mantiene el layout clásico de 82 bytes, rellena hasta 165 bytes, marca el tipo de cuenta en el byte 165, y después lleva una cola de extensiones TLV.](assets/v03-annotated-code.png)

Un detalle de ese diagrama sorprende a la gente, así que nómbralo antes de que muerda: la cola no empieza en el byte 82. Un mint extendido se rellena hasta 165 bytes, el tamaño de la *cuenta de token* clásica, y el byte 165 lleva una etiqueta de tipo de cuenta de un byte, `1` para un mint. Solo entonces corre el TLV, del byte 166 hasta el final. El relleno existe para que un lector nunca pueda confundir un mint extendido con una cuenta de token por el largo solo: un mint pelado de 82 bytes nunca fue ambiguo, pero una base de 82 bytes más una cola TLV podría aterrizar en exactamente 165 bytes — el largo de una cuenta de token — y esa es la colisión que descartan el relleno más la etiqueta de tipo. En PYUSD eso deja 700 bytes de cola debajo de los 866.

¿Entonces qué se rompe de verdad si ignoras esto y asumes los 82 bytes clásicos? De dos formas, las dos feas. Si rebanas la cuenta a un largo fijo y lees un campo por offset, o aterrizas en un byte que ahora quiere decir otra cosa o te vas limpio más allá del buffer, y tu programa está tomando decisiones sobre basura. Si deserializas con un decodificador de tamaño fijo, o rechaza la cuenta o te entrega una struct base que calladamente suelta todo lo que está en la cola. Ninguna de las dos fallas se anuncia como "asumiste el tamaño equivocado". Salen a la superficie como valores basura y rechazos misteriosos, que es el peor tipo de bug para perseguir.

Puedes ver la constante romperse en una línea. Apunta la misma lectura a un mint clásico de SPL y al de Token-2022, uno atrás del otro:

```bash
# USDC, a classic SPL mint. Then PYUSD, Token-2022. Compare the Length: lines.
solana account EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v --url mainnet-beta | grep Length
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta | grep Length
```

El primero imprime la constante que podrías haber dejado hardcoded. El segundo imprime un número que tuviste que leer. La misma superficie de instrucción, el mismo `transfer_checked`, dos largos distintos: esa brecha es la consecuencia uno, y ninguna cantidad de disciplina de tipado en tu struct de cuentas la cierra por ti.

Esta es también la razón por la que tu swap tipa sus cuentas como `InterfaceAccount<T>` y no como `Account<TokenAccount>`. `Account<TokenAccount>` amarra el dueño de la cuenta al programa Token clásico. Entrégale una cuenta Token-2022 y no lee la cola mal, rechaza la cuenta de entrada, porque el dueño es un id de programa distinto. `InterfaceAccount<T>` es la versión que acepta una cuenta cuyo dueño es cualquiera de los dos programas y lee la base compartida de los dos. La trampa de la que te salva es la más vieja de este rincón de Solana: dejar hardcoded el id del programa Token clásico en algún lugar de tus cuentas o de tus verificaciones, así que en el instante en que un usuario de verdad trae un mint Token-2022 tu programa lo rebota por una razón que él no puede ver. Tipéalo como la interfaz y esa clase entera de bug nunca llega a escribirse.

### Consecuencia dos: un hook puede exigir cuentas que no estás mandando

El segundo cambio es el que falla fuerte en vez de calladamente. Un mint puede declarar un *transfer hook*: un programa al que el programa Token-2022 llama en cada transferencia de ese token, después de que corre la lógica propia de la transferencia. Quien crea el token escribe y despliega ese programa, y después apunta el mint hacia él a través de la extensión TransferHook.

Para tu swap, la consecuencia es angosta y específica. Cuando una transferencia corre contra un mint con hook, el programa de hook se ejecuta, y necesita sus propias cuentas. Esas cuentas extra no son cuentas que conozcas en tiempo de compilación. Se resuelven desde una lista on-chain que el hook publica para ese mint. Tu instrucción tiene que reenviarlas. Si armas un `transfer_checked` con solo las cuatro cuentas que siempre tomó, from, to, mint, authority, y el mint tiene un hook vivo, la transferencia no se va a completar. `transfer_checked` sigue siendo la primitiva correcta. No es la instrucción la que está equivocada. Es la lista de cuentas la que está corta.

Así que desde el asiento del programa hay exactamente una pregunta que tienes que poder responder antes de una transferencia: ¿este mint declara un hook, y si lo hace, ese hook está vivo? Y el campo sobre el que lo llaveas es el `programId` de la extensión TransferHook. Si el mint no lleva ninguna extensión TransferHook, no hay nada que reenviar. Si lleva la extensión pero el `programId` está sin valor, el hook está *declarado pero dormido*, todavía nada que reenviar. Solo cuando el `programId` es una dirección de verdad una transferencia necesita que las cuentas extra del hook se resuelvan y se agreguen.

Sin valor es una cosa específica acá, no un gesto vago. En el wire ese campo es una pubkey opcional-no-cero: siempre treinta y dos bytes, todos ceros queriendo decir "ninguno". Así que "sin valor" se lee de vuelta como la dirección default de todos ceros, `11111111111111111111111111111111`, que importa en el momento en que escribes la verificación, porque no es `null` y no está vacía.

Ese caso dormido no es un rincón que inventé para ser minucioso. El mint PYUSD vivo lleva una extensión TransferHook ahora mismo, y su `programId` es el default de todos ceros. Ocho extensiones presentes, el hook entre ellas, y aun así un `transfer_checked` simple contra él no necesita cuentas extra, porque el hook está armado y quieto en vez de activo. Por esto "¿tiene la extensión?" es la pregunta equivocada y "¿está el `programId` con valor?" es la correcta. Vas a ver exactamente eso en un minuto cuando corras el lector.

![Una rama de sí/no: un mint sin hook, o con un programId de hook dejado en la dirección default de todos ceros, transfiere normalmente; un programId de verdad exige resolver cuentas extra primero.](assets/v04-flowchart.png)

Dos capas, otra vez, y vale nombrarlas juntas porque son la forma de la historia entera de Token-2022 en Anchor.

![Anchor te da compatibilidad con los dos programas gratis a través de InterfaceAccount y transfer_checked, pero leer la cola de extensiones para el largo y el estado del transfer hook es trabajo que tu programa tiene que hacer.](assets/v05-diagram.png)

Esa etiqueta de costura es el trade-off, dicho sin adornos. `token_interface` te compra compatibilidad con los dos programas gratis, al nivel del tipo, y es genuinamente un alivio comparado con dejar hardcoded un id de programa y ramificar. Pero Token-2022 mueve trabajo real hacia ti a cambio: no puedes asumir un tamaño fijo, y un mint con hook quiere decir que una transferencia que *parece* completa puede fallar a menos que reenvíes las cuentas del hook. El framework te entrega el asiento. No te entrega el estándar.

Otra trampa de lee-no-asumas va justo al lado del hook, porque el mint del jugador de la apertura llevaba las dos. Un mint también puede declarar una tarifa de transferencia, y cuando lo hace, la cantidad que de verdad aterriza en el destino es más chica que la cantidad que le entregaste a `transfer_checked`. Para un envío simple de billetera a billetera eso es una molestia de redondeo. Para tu swap es un bug de corrección: tu matemática de producto constante asume que el vault recibió exactamente lo que le mandaste, y una tarifa calladamente anula esa suposición, así que tu invariante se corre y tu precio sale mal. La mecánica es idéntica a todo lo demás acá, lee el mint, no asumas la cantidad. La matemática propia de la tarifa, cómo computan de verdad los basis points y el tope máximo, es el catálogo de extensiones, y ese catálogo es del curso de Digital Assets. Notar que el neto-recibido puede diferir de la cantidad-mandada es la parte que es tuya.

![Un mint Token-2022 que lleva tarifa entrega menos que la cantidad mandada, así que el invariante del swap se computa sobre la reserva equivocada y el saldo almacenado del vault sobreestima la custodia real.](assets/v06-diagram.png)

Y esa es la línea que no cruzamos. Diseñar una extensión, escribir un programa de transfer hook, cablear su interfaz de resolución de cuentas de punta a punta, eso es profundidad de estándares, y vive en un solo lugar por diseño. El curso de Digital Assets recorre la interfaz de transfer hook de punta a punta y enseña profundidad de estándares de extensión. Acá, desde el asiento del framework, tu trabajo se detiene en notar que el hook existe y en saber que tendrías que reenviar sus cuentas. Notar es mecánica. La autoría es el estándar. Curso distinto, a propósito.

![Esta lección enseña leer el largo consciente de extensiones, detectar un transfer hook y razonar sobre cuentas extra; diseñar extensiones y escribir la interfaz de transfer hook le pertenecen al curso de Digital Assets.](assets/v07-comparison.png)

Podrías tentarte de archivar todo esto como "caso de borde que voy a manejar cuando alguien se queje". Resístete a eso. Los mints Token-2022 que andan sueltos por ahí son desproporcionadamente los que menos quieres que te fallen. Las stablecoins reguladas y los activos de mayor valor recurren a delegados permanentes, tarifas de transferencia y hooks precisamente porque hay dinero real y compliance real montados sobre ellos. La memecoin descartable nunca va a ejercitar este camino. El mint que de verdad le importa a tu tesorería sí. Esa asimetría es el argumento entero: un hábito de lee-no-asumas es un seguro barato, y un tamaño hardcoded es una bomba de tiempo con el nombre de tu contraparte más grande escrito encima.

## Lab: lee el mint antes de confiar en él

Hora de volver visibles las dos consecuencias. Vamos a escribir un lector chico, apuntarlo al mint Token-2022 provisto, y reportar el largo observado y el estado del hook. Este es el compás de corre-observa-razona, y es deliberadamente un script de cliente: la pregunta es sobre lo que el mint *es*, y la forma más limpia de responderla es leer la cuenta viva.

Estoy recorriendo cada paso acá. Copia junto conmigo.

**1. Instala las dependencias de cliente.** Nada de Rust y nada de Anchor en este lab: la pregunta es sobre lo que un mint *es*, así que el lector es un script de Node y tu toolchain V2 se sienta esta afuera. Usamos `@solana/kit` y el cliente Token-2022 nativo de kit. Mira el pin, y mira el *par*: al 2026-09-07 el `latest` de kit en npm es 8.2.0 mientras el `latest` de `@solana-program/token-2022` es 0.16.1, que hace peer con kit `^8`. La instalación de abajo deliberadamente sostiene el par más viejo — `token-2022@^0.15` hace peer con kit `^7` — porque un par emparejado es lo que tiene que resolver, no el más nuevo de cualquiera de los dos. Corre la verificación de frescura de abajo antes de tocar estos, y muévelos a los dos juntos o a ninguno.

```bash
npm install @solana/kit@^7 @solana-program/token-2022@^0.15
npm install -D tsx    # runs a TypeScript file directly
# Freshness check before you ever bump these:
#   npm view @solana-program/token-2022 peerDependencies
```

Fija ese rango de cliente explícitamente. Deja `@solana-program/token-2022` sin versión y el resolvedor de npm puede retroceder hacia un `0.11.x` más viejo, que hace peer con kit `^6`, y la instalación muere en un conflicto de peer `ERESOLVE` contra el `^7` que acabas de pedir. Nombrar los dos majors es lo que hace que el par resuelva.

Checkpoint: la instalación termina sin ningún `ERESOLVE` y `npm ls @solana/kit` imprime un único `7.x` en el nivel de arriba. Dos versiones de kit en ese árbol quieren decir que algo trajo el `^6` de vuelta, y el lector va a fallar por un desajuste de tipos y no por el mint.

**2. Toma el lector.** Este script se te da para correrlo, no es TypeScript que se te esté pidiendo escribir. En este curso escribes Rust; la única lección en la que escribes un cliente en TS es m08-l1, y esta no es esa. Cópialo tal como está, y léelo como leerías el script de un colega: dos lecturas, que se corresponden con las dos consecuencias. Primero la cuenta cruda, para medir su largo real. Después el mint decodificado, para inspeccionar la extensión de transfer hook. Guarda esto como `inspect.ts`:

```typescript
import {
  address,
  createSolanaRpc,
  fetchEncodedAccount,
  unwrapOption,
} from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const RPC_URL = "https://api.mainnet-beta.solana.com";
const rpc = createSolanaRpc(RPC_URL);

// The provided specimen: a live Token-2022 mint on mainnet.
const MINT = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");

// On the wire the hook's program is an "optional non-zero" pubkey: 32 bytes,
// all-zero meaning unset. The client decodes those zero bytes to the default
// address below, NOT to null, so this is what "no hook" actually looks like.
const UNSET_HOOK = address("11111111111111111111111111111111");

async function inspect(): Promise<void> {
  // Consequence one: read the real length. Never assume the classic 82 bytes.
  const raw = await fetchEncodedAccount(rpc, MINT);
  if (!raw.exists) {
    throw new Error(`mint ${MINT} not found`);
  }
  console.log("owner program:", raw.programAddress);
  console.log("account bytes:", raw.data.length);

  // Consequence two: does this mint declare a transfer hook, and is it live?
  const mint = await fetchMint(rpc, MINT);
  const extensions = unwrapOption(mint.data.extensions) ?? [];
  console.log("extensions present:", extensions.length);

  const hook = extensions.find((ext) => ext.__kind === "TransferHook");

  if (hook === undefined) {
    console.log("transfer hook: none -> transfer needs no extra accounts");
    return;
  }

  if (hook.programId === UNSET_HOOK) {
    console.log("transfer hook: present but programId unset -> dormant, no extra accounts");
  } else {
    console.log(`transfer hook: ACTIVE (${hook.programId}) -> forward its extra accounts`);
  }
}

inspect().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

**3. Córrelo.**

```bash
npx tsx inspect.ts
```

**4. Lee la salida.** Deberías ver algo que se alinea con esto, con alguna variación en las direcciones exactas:

```text
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
account bytes: 866
extensions present: 8
transfer hook: present but programId unset -> dormant, no extra accounts
```

Detente y lee eso, porque es la lección entera aterrizando de una vez. El `owner program` es el programa Token-2022, que es por lo que los campos base decodificaron y la cola siquiera existe. El `account bytes` es 866, no 82, que es la consecuencia uno en una línea: esta cuenta tiene más de diez veces el tamaño clásico, y tu programa corrompería cada lectura si asumiera la constante. Volvieron ocho extensiones, que son la cola de la que están hechos esos bytes extra. Y la línea del hook es la consecuencia dos con el matiz horneado adentro: la extensión está *presente*, pero su `programId` está sin valor, así que un `transfer_checked` contra este mint no necesita cuentas extra. Que la extensión estuviera ahí no respondió la pregunta. El `programId` sí.

Esa comparación contra `UNSET_HOOK` vale otro compás, porque es el lugar exacto donde un lector descuidado entrega un bug. "Sin valor" en el wire es treinta y dos bytes cero, y el cliente te los entrega de vuelta como la dirección default a la que codifican esos ceros, `11111111111111111111111111111111`, no como `null` y no como `undefined`. Así que una prueba de veracidad sobre `hook.programId` siempre da verdadero y reportaría cada hook dormido como vivo:

```typescript
// WRONG: a 32-zero-byte pubkey decodes to a non-empty string, so this is
// always truthy and reports every dormant hook as active.
if (hook.programId) { /* resolve extra accounts */ }

// RIGHT: compare against the address those zero bytes actually encode to.
if (hook.programId !== UNSET_HOOK) { /* resolve extra accounts */ }
```

Mete la línea equivocada en `inspect.ts` y vuelve a correrlo si quieres ver el modo de falla: el mismo hook dormido de PYUSD vuelve `ACTIVE`, y un programa que confiara en eso empezaría a reenviar cuentas que nadie pidió. Compara contra la dirección default explícitamente, como lo hace el script, o escribiste una verificación que solo puede responder sí.

Si tu lector imprimió una dirección de verdad en esa última línea en vez del mensaje de dormido, estarías mirando un mint cuyas transferencias no puedes completar con cuatro cuentas, y el campo que te lo dijo es el mismo, el `programId`. Esa es la decisión entera, y acabas de hacer que tu programa la declare en voz alta desde una lectura viva en vez de una adivinanza.

Dos notas antes de que salgas a cazar por tu cuenta. Primera, todo lo que acabas de hacerle a un mint se aplica también a las cuentas de token. Una cuenta de token Token-2022 es la base clásica de 165 bytes más su propia cola de extensiones, así que la regla de lee-el-largo-real vale cada vez que tu programa toca un saldo, no solo cuando inspecciona un mint. Apunta la misma llamada a `fetchEncodedAccount` a una cuenta de token y vas a ver el mismo largo variable. Segunda, mantén claro por qué esto fue un script de cliente y no una lectura on-chain. On-chain, `InterfaceAccount<T>` y `anchor_spl::extensions` hacen este trabajo exacto adentro de tu handler. Off-chain, lo hace kit. Las mismas dos preguntas, las mismas dos respuestas, un asiento distinto. El punto nunca fue el lenguaje. Fue el hábito de preguntarle a la cuenta en vez de a tu memoria.

## Challenge: encuentra un mint que diga sí

El lab te entregó un mint cuya respuesta es "ninguna cuenta extra". Ahora haces uno que diga "sí". Cazar en mainnet un hook vivo es un ejercicio de aguja en un pajar sin forma de distinguir la falla de la mala suerte, así que vas a acuñar el especimen tú mismo en devnet, donde controlas cada entrada.

El `programId` de la extensión TransferHook es solo una pubkey almacenada; nada valida que apunte a un programa de hook de verdad al momento de acuñar. Así que cualquier id de programa que sea tuyo hace que el campo se lea como con valor, que es exactamente el estado que estás tratando de observar — no tiene ni que estar desplegado. El id de tu swap desde `Anchor.toml` funciona aunque ese programa solo haya corrido adentro de LiteSVM, y el greeter R0 que de verdad entregaste en m01-l2 funciona igual de bien:

```bash
solana config set --url devnet
solana airdrop 2

# Any program id works as the hook target; use your own swap's, from Anchor.toml.
spl-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb \
  create-token --transfer-hook <YOUR_SWAP_PROGRAM_ID> --decimals 6
```

Eso imprime una dirección de mint nueva. Ponla en `inspect.ts` como `MINT`, cambia `RPC_URL` a `https://api.devnet.solana.com`, y córrelo. Si `spl-token` no está en tu máquina, `cargo install spl-token-cli` lo pone ahí.

Aceptación: la última línea lee `ACTIVE` e imprime el id de programa que pasaste. Si lee `none` en cambio, creaste el mint sin `--transfer-hook`. Si el lector da error antes de imprimir cualquier cosa — una falla de dueño o de decodificación — creaste un mint clásico; verifica la flag `--program-id`, que es lo que selecciona Token-2022. (`dormant` no deberías verlo acá: `create-token --transfer-hook` escribe un id de programa de verdad en el campo.)

Cuando la última línea lea `ACTIVE`, quédate con lo que quiere decir para el swap que construiste. Tu instrucción actual manda cuatro cuentas. Las transferencias de este mint necesitan más, resueltas desde la lista on-chain del hook, y tu swap tal como está escrito fallaría contra él. No tienes que arreglar eso hoy. Construir de verdad la resolución le pertenece a la profundidad de estándares que estamos dejando deliberadamente al curso de Digital Assets. Notar que tendrías que hacerlo es la habilidad para la que existió esta lección.

## Qué deberías poder decir ahora

Acá está el checkpoint, y es la forma exacta de la respuesta que esta lección se construyó para producir. Apunta tu lector al mint provisto y, desde la lectura y no desde la memoria, reporta de vuelta:

- el largo de cuenta consciente de extensiones que observaste, en bytes, y
- sí o no sobre si una transferencia necesita cuentas extra de hook, nombrando el único campo sobre el que lo llaveaste.

Para el mint del lab eso es: 866 bytes, ninguna cuenta extra necesaria, llaveado sobre el `programId` de la extensión TransferHook todavía sentado en la dirección default de todos ceros. Si puedes producir esa forma para el mint que hiciste en el Challenge también, terminaste.

Lo que finalmente responde al jugador en la puerta de la máquina arcade. Su mint llevaba una tarifa y un hook, así que los tres resultados estaban vivos y ahora puedes decir cuál es cuál. Si el `programId` del hook está con valor, tu `transfer_checked` **falla de entrada**, y falla limpio en vez de mover cualquier cosa a medias, porque la CPI del hook aborta la transferencia. Si el hook está dormido pero hay un `TransferFeeConfig` presente, **entrega menos en silencio**: la llamada tiene éxito, llega menos de lo que mandaste, y tu invariante es la cosa que lo nota, tarde. Y si ninguno de los dos está armado, simplemente **funciona**, que es el caso PYUSD que acabas de leer. Tres respuestas, una lectura, y la lectura es la habilidad entera. Puedes mover un token a través de los dos programas y puedes mirar un mint que nunca viste y decir, desde el asiento de tu propio programa, exactamente qué exigiría una transferencia contra él.

La próxima lección dejamos de leer las cuentas de otros y empezamos a hacerle rayos X a las nuestras. Prendes los instrumentos first-party de V2 sobre este swap exacto, flamegraphs, un debugger de step y cobertura, para ver a dónde se va de verdad su compute. Mediste cuánto te cuesta un mint en bytes. Después mides cuánto cuesta tu programa en compute.
