# ¿Qué es este token, en realidad?

Esta es la lección uno, así que todavía no hay nada construido. Llegas con los fundamentos de Solana en la cabeza: cuentas, PDAs, transacciones, comisiones, ATAs. Llegas también, lo más probable, con una creencia que yo sostuve muchísimo más tiempo del debido: que "un token" quiere decir el mint SPL clásico que ya usas. Vamos a romper esa creencia en los próximos cinco minutos, usando un token que PayPal les entrega a millones de personas. Si llegaste acá desde el curso Solana Payments and Commerce, ya conociste este mint desde el lado del que paga; ahora vas a leer cada byte de él.

Sin instalar toolchain. Necesitas Node 20 o más nuevo (`node --version` para verificarlo; yo estoy en la 23.9) y nada más. Guarda este archivo como `read-pyusd.ts`:

```typescript
// read-pyusd.ts - read PayPal USD's live mint account and list what it carries.
// Zero npm dependencies. Run: npx tsx@4.20.5 read-pyusd.ts
// Read-only: one RPC call against mainnet, nothing signed, nothing sent.

const RPC = "https://api.mainnet-beta.solana.com";
const PYUSD_MINT = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
const CLASSIC_MINT_SIZE = 82; // a classic SPL mint is exactly this many bytes

async function main() {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      params: [PYUSD_MINT, { encoding: "jsonParsed" }],
    }),
  });
  const { result, error } = await res.json();
  if (error) throw new Error(JSON.stringify(error));

  const account = result.value;
  const info = account.data.parsed.info;

  console.log(`mint:          ${PYUSD_MINT}`);
  console.log(`owner program: ${account.owner}`);
  console.log(`account size:  ${account.space} bytes (a classic mint is ${CLASSIC_MINT_SIZE})`);
  console.log(`decimals:      ${info.decimals}`);
  console.log(`supply:        ${info.supply} base units`);
  console.log(`mintAuthority: ${info.mintAuthority}`);
  console.log(`freezeAuth:    ${info.freezeAuthority}`);

  const extensions = info.extensions ?? [];
  console.log(`\nextensions (${extensions.length}):`);
  for (const ext of extensions) {
    console.log(`  - ${ext.extension}`);
  }

  const hook = extensions.find((e: any) => e.extension === "transferHook");
  const fee = extensions.find((e: any) => e.extension === "transferFeeConfig");
  console.log(`\ntransferHook.programId: ${hook?.state.programId}`);
  console.log(
    `transferFee: ${fee?.state.newerTransferFee.transferFeeBasisPoints} bps, ` +
    `max ${fee?.state.newerTransferFee.maximumFee}`
  );
  if (hook && hook.state.programId === null) {
    console.log(`\n=> a transfer hook slot exists, but no hook program is set.`);
    console.log(`   configured, but dormant.`);
  }
}

main().catch((e) => { console.error(e); process.exit(1); });
```

Córrelo:

```bash
npx tsx@4.20.5 read-pyusd.ts
# tsx pinned at 4.20.5, verified working 2026-08-22; npx fetches it on first
# run, so there is genuinely nothing to install.
```

Cuando corrí esto hoy, 2026-08-22, contra el RPC público gratuito, me dio:

```text
mint:          2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
account size:  866 bytes (a classic mint is 82)
decimals:      6
supply:        688176370728435 base units
mintAuthority: 8Jornc27vtAYPkwDzsZVgLQchAYyC8nD7aCNPCDV8Qk2
freezeAuth:    2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk

extensions (8):
  - mintCloseAuthority
  - permanentDelegate
  - transferFeeConfig
  - confidentialTransferMint
  - confidentialTransferFeeConfig
  - transferHook
  - metadataPointer
  - tokenMetadata

transferHook.programId: null
transferFee: 0 bps, max 0

=> a transfer hook slot exists, but no hook program is set.
   configured, but dormant.
```

Ocho extensiones, posadas sobre un mint que millones de usuarios de PayPal mueven de acá para allá sin sospechar ni una vez que tiene una forma distinta de cualquier otro token de dólar. Un transfer hook que existe pero no apunta a ningún programa. Una comisión de transferencia de cero basis points que, aun así, está ahí mismo en los bytes, esperando. Tu línea de supply va a diferir de la mía, porque esta es una cuenta viva y PayPal acuña y quema contra ella todos los días. Todo lo demás debería coincidir.

Espera, un momento. Tú "usas SPL tokens" todos los días. Entonces, ¿por qué este lleva un `permanentDelegate`, una autoridad que puede mover el PYUSD de cualquiera fuera de la cuenta de cualquiera? ¿Por qué su mint es diez veces el tamaño de los mints que tú hiciste? ¿Y por qué hay un slot de hook configurado pero apagado?

Todavía no puedes responder. Esa brecha es este curso.

## Resumen

Acabas de decodificar un token que no hiciste, usando un decodificador que no escribiste. Esta lección trata de lo que viste: qué es realmente una cuenta de mint, qué quieren decir como forma las ocho entradas de extensión en el mint de PYUSD (todavía no en mecanismo), y por qué "SPL token" dejó de nombrar una sola cosa hace años. En el lab vas a correr un segundo script sin configuración que simula una transferencia clásica común y lee su costo de cómputo directo del runtime: un segundo número que todavía no puedes explicar. Los dos números se resuelven a lo largo de este módulo. La tesis con la que te vas hoy es corta: una primitiva de activo es una decisión, y no puedes tomar una decisión que no puedes leer.

Reglas de la casa, dichas una sola vez y válidas para todo el curso:

- **Lee la teoría, no escribas código a la par.** Tus manos se mueven en el Lab numerado, y solo ahí. La apertura que acabas de correr es la única excepción que recibe cada lección: algo que hacer antes de algo que creer.
- **Haz el Challenge solo.** Sin guía, sin pasos de solución. Ahí es donde el aprendizaje se acumula.
- **Cada herramienta muestra su instalación la primera vez que aparece**, y cada versión fijada lleva una fecha. Hoy eso fue `tsx@4.20.5`, verificado 2026-08-22.
- **La ayuda se repliega según un calendario.** Hoy te doy dos scripts completos y tú los corres. La próxima lección construyes el decodificador: el tramo de los campos base se trabaja contigo, y el recorrido de extensiones te toca escribirlo a ti. Para el final del módulo estás eligiendo primitivas y defendiendo la elección. Las rueditas se quitan a propósito, no por sorpresa.

## Un token es una decisión que todavía no puedes tomar

Vamos a nombrar lo que de verdad leíste, porque dos de las palabras importan enormemente y los desarrolladores de Solana las mezclan a diario.

Una **cuenta de mint** es la cuenta que define el token en sí: su supply, sus decimales, quién puede crear más, quién puede congelarlo. Un mint por token, para siempre. No es donde viven los saldos. Los saldos viven en las **cuentas de token** (normalmente las cuentas de token asociadas, ATAs, que ya conoces), una por tenedor por token. Cuando "consultas tu saldo de PYUSD" lees una cuenta de token. Cuando preguntas qué *es* PYUSD, lees el mint. Hoy leemos el mint, y vamos a seguir leyendo mints todo el módulo, porque el mint es donde están escritas la identidad de un token y sus reglas.

Ahora el campo que debería haberte frenado: `owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`. Ese no es el programa de tokens que conoces. El programa SPL Token clásico vive en `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`. El mint de PYUSD es propiedad de un programa distinto en una dirección distinta: Token-2022, también llamado Token Extensions. Mismo trabajo, reglamento distinto, y los dos no se mezclan. Una instrucción de token clásico apuntada a un mint Token-2022 falla, y al revés.

![Dos programas de tokens separados, SPL Token clásico siendo dueño del mint USDC de 82 bytes y Token-2022 siendo dueño del mint PYUSD de 866 bytes con ocho extensiones, con instrucciones que no pueden cruzar entre ellos.](assets/v01-diagram.webp)

Esta es la trampa número uno de todo este curso, así que lo voy a decir sin rodeos: **"SPL token" no es una sola cosa.** Me pasé un tramo vergonzoso de mis primeros trabajos en Solana diciendo "SPL token" como si nombrara un estándar único, y entregué integraciones sobre esa suposición. Culpable, y con todas las letras. Me costó un fin de semana de debugging la primera vez que un mint Token-2022 chocó con código que tenía `Tokenkeg` hardcodeado. Los dos programas coexisten en mainnet, para siempre, y cada billetera, DEX e indexador tiene que manejar los dos.

### Los 82 bytes y los 866

El tamaño es la forma más rápida de sentir la diferencia. Un mint SPL clásico tiene exactamente 82 bytes, siempre: autoridad de mint, supply, decimales, bandera de inicializado, autoridad de congelación. Layout fijo, nada opcional, nada más, y los mismos cinco campos ya sea que el mint sea USDC o algo que levantaste un martes por la tarde para practicar. Tu script imprimió PYUSD en 866 bytes. ¿A dónde se fueron los otros 784 bytes?

Se fueron a las **extensiones**: paquetes opcionales de estado y comportamiento extra que Token-2022 le deja a un emisor adjuntar a un mint (o a cuentas de token individuales) en el momento de la creación. Cada extensión queda asentada en los datos de la cuenta con un esquema llamado **TLV**, de tipo-longitud-valor: una etiqueta chica que dice qué extensión es esta, un largo que dice cuántos bytes ocupa, y después los bytes del valor en sí. Una tras otra, como cajas etiquetadas en fila. Un decodificador recorre la fila: lee una etiqueta, lee un largo, salta adelante, repite. Ese recorrido es exactamente lo que el parser del RPC hizo por ti hoy, y exactamente lo que vas a implementar tú mismo la próxima lección.

![Un mint clásico son cinco campos fijos que suman 82 bytes, mientras que el mint de PYUSD tiene los mismos campos base seguidos de ocho entradas de extensión tipo-longitud-valor, que suman 866 bytes.](assets/v02-diagram.webp)

Lee de nuevo las ocho entradas de PYUSD, esta vez como una historia. El conjunto completo es mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer y tokenMetadata. Eso es un equipamiento con forma de compliance. Un delegado permanente quiere decir que Paxos, el emisor regulado, puede mover o quemar PYUSD desde cualquier cuenta: poderes de incautación, lo que exige una orden judicial. El par de transferencia confidencial son rieles de privacidad. El par de metadatos pone el nombre y el símbolo del token on-chain en el mint mismo en vez de en la cuenta de un ecosistema aparte. Y dos de las entradas están cargadas pero no disparan: la comisión de transferencia está configurada en 0 basis points con un máximo de 0, y el transfer hook, la extensión que dejaría correr un programa en todas y cada una de las transferencias, tiene `programId: null`.

**Configurado, pero dormido.** Guárdate esa frase; hace trabajo real todo el curso. Que una extensión esté presente en los bytes no es lo mismo que tenga un efecto activo. PYUSD lleva un slot de hook y una tabla de comisiones igual que un edificio lleva conducto vacío: hoy no corre nada por ahí, pero nadie tiene que romper las paredes con un martillo neumático para cambiar eso después. Presente no es activo, y esas son dos preguntas separadas que una sola mirada a una lista de extensiones va a mezclar alegremente. Cuando evalúes cualquier token de ahora en adelante, preguntas primero si una extensión existe en los bytes y después, por separado, si actualmente hace algo en absoluto.

Una advertencia honesta antes de la tesis, porque es el trade-off sobre el que se apoya toda esta lección. Leer un mint te dice lo que un token **es**. No te dice lo que tu plataforma objetivo va a **aceptar**. Las mismas ocho extensiones que hacen que PYUSD cumpla lo suficiente para PayPal harían que algunos programas de DEX rechazaran un token de plano, porque rechazan extensiones Token-2022 desconocidas antes que arriesgarse a un comportamiento que no modelaron. La visibilidad es necesaria y no suficiente. Ya puedes decodificar un token y aun así no poder entregar uno; la mitad de esa habilidad que es elegir-y-verificar viene más adelante en el curso, y no voy a fingir que el script de hoy te la da.

### El menú detrás de la pregunta

Ahora, la razón por la que esta lección existe en el día uno, antes de construir nada: todo lo que vas a hacer en este curso arranca de una decisión que la mayoría de la gente toma por defecto en vez de a propósito: **¿qué primitiva de activo emites?**

En Solana en 2026 ese menú tiene cuatro entradas serias. Un mint SPL clásico: 82 bytes, sin extensiones, aburrido a propósito, soportado por literalmente todo. Un mint Token-2022 con un conjunto de extensiones elegido: comportamiento programable al costo de preguntas de compatibilidad plataforma por plataforma. Un activo Metaplex Core: el estándar recomendado actual para trabajo con NFT, una familia de programas completamente distinta. Y un NFT comprimido: estado que vive en un árbol de Merkle en vez de en su propia cuenta, una reducción de costo de mil veces con sus propias consecuencias en el camino de lectura.

![Una comparación a cuatro bandas de mints SPL clásicos, mints Token-2022 con extensiones, activos Metaplex Core y NFT comprimidos, cada uno resumido por forma on-chain, reputación y cobertura del curso.](assets/v03-comparison.webp)

Acá está la tesis, y es lo más parecido a filosofía que vas a tener hoy. Cada uno de esos cuatro no es un nivel de producto; es una respuesta distinta a la pregunta "¿qué debería hacer cumplir la blockchain sobre este activo?". Un mint clásico responde "casi nada más allá de supply y congelación". El conjunto de extensiones de PYUSD responde "incautación, comisiones, privacidad y metadatos, parte de eso precableado y dormido". Una decisión así solo es real si puedes verificar lo que de verdad se decidió, y el único lugar donde la decisión está escrita son los bytes que leíste hoy. Los whitepapers describen intenciones. Los mints son la ley. No puedes elegir una primitiva de activo que no puedes leer, y hasta esta mañana no podías leer ninguna. Por eso decodificar vino antes que todo, incluido el toolchain.

### Tokens más nuevos que los tutoriales

Hay una objeción justa acechando acá, y es la que yo habría levantado hace unos años: quizá Token-2022 es sobre todo una especificación, impresionante en papel y poco desplegada en la práctica. El mint que decodificaste esta mañana es el contraargumento, y es uno pesado. PayPal y Paxos entregaron PYUSD sobre Token-2022 en mayo de 2024, el despliegue emblemático del programa, y para finales de mayo de 2025 la propia nota de Solana para desarrolladores sobre PYUSD contaba $215.9 millones de él en manos de apenas 20,400 cuentas de token. La cifra de supply que tu script imprimió hoy es la que sea hoy; la mía leyó unos 688 millones de dólares en unidades base. Número vivo, cuenta viva, que es exactamente por qué el script lo lee en vez de que yo lo afirme.

Mientras tanto la educación oficial sobre todo esto se congeló a mitad de la trama. El repositorio developer-content de solana-foundation, la fuente detrás de toda una generación de cursos oficiales, fue archivado el 24 de enero de 2025. Cada curso construido a partir de él es anterior a ScaledUiAmount, Pausable, ConfidentialMintBurn y el cambio de motor de p-token. Piensa un momento en lo que eso quiere decir: los tokens que decodificas hoy son más nuevos que los tutoriales que se suponía que los explicaban.

![Una línea de tiempo desde el lanzamiento de PYUSD en mayo de 2024, pasando por el archivado del contenido oficial para desarrolladores en enero de 2025, hasta las funcionalidades de 2026 que ningún tutorial cubre, y que termina con el lector decodificando el mint.](assets/v04-timeline.webp)

Esa fecha de archivado es la razón por la que este curso tiene una disciplina de pruebas como hilo conductor que vas a encontrar una y otra vez: **mide, no memorices.** Los números sobre un sistema vivo se pudren. Lo que me lleva al segundo número que te prometí, el que vas a producir tú mismo en el lab. Una transferencia común en el programa SPL Token clásico, la aburrida del mint de 82 bytes, cuesta actualmente 76 unidades de cómputo. Las **unidades de cómputo**, CU, son el medidor de Solana para el trabajo on-chain: cada instrucción corre contra un presupuesto (200,000 por defecto, y vas a ver esa cifra exacta en una línea de log en un rato), y lo que consume lo reporta el runtime mismo. Durante años esa misma transferencia costó 4,645 CU. En 2026 la implementación detrás del programa de token clásico fue cambiada por debajo de la interfaz, y el precio se desplomó. Misma dirección de programa, mismos bytes de instrucción, una caída de sesenta veces. Cómo ese cambio fue siquiera posible sin que se rompiera la billetera de nadie es la lección tres de este módulo, y es una de las mejores historias de sistemas en Solana. Hoy solo mides las secuelas, y te niegas a memorizarlas, porque un número que cayó sesenta veces una vez puede volver a moverse.

### El camino desde acá: Overgrowth

Todo en este curso construye una sola cosa. **Overgrowth** es un juego cooperativo de cultivo y crafteo cuya economía on-chain entera vas a levantar, primitiva por primitiva: SPROUT, su moneda, como un mint Token-2022 cuyo conjunto de extensiones vas a elegir y defender; los Almanac NFTs, los libros de conocimiento coleccionables que restringen el acceso a las recetas de crafteo; y los Harvest crates, recompensas de drop estacionales acuñadas como NFT comprimidos porque van a ser muchísimos, demasiados como para pagar rent por cuenta. Para el módulo final vas a haber emitido activos en todas las formas del menú de arriba, y el punto de hoy es que lo vas a hacer como alguien que lee bytes antes de confiar en nombres.

Este módulo es la rampa de entrada, y corre a propósito al revés: lo concreto primero, los fundamentos después. Hoy pediste prestado un decodificador y sentiste dos números que no puedes explicar. La próxima lección dejas de pedir prestado: construyes el decodificador tú mismo, empezando por el mint pelado de 82 bytes y subiendo por el recorrido TLV, y ese inspector se vuelve la primera herramienta real del kit de Overgrowth, la que las lecciones posteriores llaman. La lección tres explica el 76. La lección cuatro convierte el catálogo de extensiones en un framework de elección y cierra el módulo con SPROUT como su ejemplo resuelto; la decisión real de spec y tamaño para SPROUT abre la conversación de diseño del próximo módulo, con el framework en tus manos.

![Un mapa del módulo en cuatro pasos que muestra las dos mediciones sin explicar de hoy resueltas por la construcción del inspector en la lección dos, el cambio de motor en la lección tres y el framework de elección en la lección cuatro.](assets/v05-flowchart.webp)

Suficiente teoría. Ve a medir el segundo número.

## Lab: dos números desde un arranque en frío

Ahora escribe código a la par; esta es la parte que haces, no que lees. Unos quince minutos.

**1. Confirma tu runtime.** Necesitas Node 20+ para el `fetch` incorporado sobre el que se apoyan estos scripts.

```bash
node --version
# v20.x or newer. Mine printed v23.9.0.
```

**2. Vuelve a correr el lector de PYUSD, y esta vez léelo como un auditor.** Lo corriste en la apertura; ahora extráele afirmaciones. Corre `npx tsx@4.20.5 read-pyusd.ts` de nuevo y marca cuatro hechos contra tu propia salida: el owner program empieza con `Tokenz` (Token-2022, no el clásico); la cuenta tiene 866 bytes contra los 82 del clásico; el conteo de extensiones es 8; y `transferHook.programId` es `null` mientras la comisión lee 0 bps con máximo 0. Esos dos últimos son el par "configurado, pero dormido". Si tu conteo de extensiones difiere de 8, no asumas que la lección está equivocada ni que lo estás tú. El conjunto de extensiones de un mint queda fijo cuando se crea el mint, salvo una excepción estrecha de metadatos (la lección cuatro hace todo un argumento con eso), así que un conteo distinto casi siempre quiere decir que el parser de tu RPC expone las entradas con otros nombres, que el endpoint te sirvió un parseo viejo o parcial, o que la dirección se escribió mal. Lee la lista que imprimió tu corrida y compárala entrada por entrada.

**3. Apunta el mismo decodificador a un mint clásico.** Copia el archivo a `read-usdc.ts` y cambia una constante, para que puedas ver cómo se ve un mint del programa clásico a través del mismo lente:

```typescript
// in read-usdc.ts, replace the PYUSD address with classic USDC's mint:
const PYUSD_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
```

```bash
npx tsx@4.20.5 read-usdc.ts
```

Mi corrida de hoy:

```text
mint:          EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
owner program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
account size:  82 bytes (a classic mint is 82)
decimals:      6
supply:        7923463957481104 base units
mintAuthority: BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG
freezeAuth:    7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar

extensions (0):

transferHook.programId: undefined
transferFee: undefined bps, max undefined
```

El owner empieza con `Tokenkeg`, el tamaño es exactamente 82, el conteo de extensiones es cero. Y mira esas dos líneas `undefined`: ese es nuestro script haciéndole preguntas de Token-2022 a un mint clásico. Ni dormido, ni cero. No hay ningún slot en el que estar dormido. Un mint clásico ni siquiera puede representar los conceptos, que es la demostración más limpia que vas a tener de que estos son dos programas distintos, no un programa con opciones.

**4. Ahora el segundo cliffhanger. Guarda esto como `transfer-cu.ts`.** Simula una transferencia real de token clásico entre dos cuentas vivas de mainnet y le pregunta al runtime cuánto costó. No se firma nada, no se pagan comisiones, nada llega a la blockchain; `simulateTransaction` con `sigVerify: false` es una pregunta gratis, y es la misma disciplina de medir primero que vas a usar todo el curso. El medio del archivo ensambla a mano una transacción cruda para que no necesitemos ninguna librería; trata esa parte como una caja negra sellada hoy. Tú eres la persona que corre el instrumento, todavía no la persona que lo construyó.

```typescript
// transfer-cu.ts - simulate a classic SPL Token transfer and read its compute cost.
// Zero npm dependencies. Run: npx tsx@4.20.5 transfer-cu.ts
// Builds a real Transfer instruction against live mainnet accounts and asks the RPC
// to simulate it. No signatures, no fees paid, nothing lands on chain.

const RPC = "https://api.mainnet-beta.solana.com";
const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // classic SPL USDC
const TOKEN_PROGRAM = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // classic SPL Token
// Any wallet that owns two USDC accounts and some SOL works. This is a well-known,
// long-lived exchange hot wallet; we borrow it on paper, simulation-only.
const WALLET = "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9";

async function rpc(method: string, params: unknown[]): Promise<any> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const json = await res.json();
  if (json.error) throw new Error(`${method}: ${JSON.stringify(json.error)}`);
  return json.result;
}

// base58 -> bytes (Solana address alphabet)
const ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function b58decode(s: string): Uint8Array {
  let n = 0n;
  for (const c of s) {
    const i = ALPHABET.indexOf(c);
    if (i < 0) throw new Error(`bad base58 char ${c}`);
    n = n * 58n + BigInt(i);
  }
  const out: number[] = [];
  while (n > 0n) { out.unshift(Number(n & 0xffn)); n >>= 8n; }
  for (const c of s) { if (c === "1") out.unshift(0); else break; }
  return new Uint8Array(out);
}

// shortvec length prefix used by the legacy transaction format
function compactU16(n: number): number[] {
  const out: number[] = [];
  do { let b = n & 0x7f; n >>= 7; if (n > 0) b |= 0x80; out.push(b); } while (n > 0);
  return out;
}

async function main() {
  // 1. Find two of the wallet's live USDC token accounts, funded one first.
  const res = await rpc("getTokenAccountsByOwner", [
    WALLET, { mint: USDC_MINT }, { encoding: "jsonParsed" },
  ]);
  const accounts = res.value
    .sort((a: any, b: any) =>
      Number(BigInt(b.account.data.parsed.info.tokenAmount.amount) -
             BigInt(a.account.data.parsed.info.tokenAmount.amount)));
  if (accounts.length < 2) throw new Error("need a wallet with two token accounts");
  const source = accounts[0].pubkey;
  const dest = accounts[1].pubkey;

  // 2. Hand-assemble a legacy transaction: one classic Transfer of 1 base unit.
  //    Keys in required order: writable signer, writables, then read-only.
  const keys = [WALLET, source, dest, TOKEN_PROGRAM].map(b58decode);
  const ixData = new Uint8Array(9);
  ixData[0] = 3; // Transfer discriminator in the classic SPL Token interface
  ixData[1] = 1; // amount = 1 as u64 little-endian (0.000001 USDC)

  const msg: number[] = [
    1, 0, 1, // header: 1 signer, 0 read-only signers, 1 read-only non-signer
    ...compactU16(keys.length), ...keys.flatMap((k) => [...k]),
    ...new Uint8Array(32), // blockhash placeholder; the RPC replaces it
    ...compactU16(1), // one instruction
    3, // program id index -> TOKEN_PROGRAM
    ...compactU16(3), 1, 2, 0, // accounts: source, dest, authority
    ...compactU16(ixData.length), ...ixData,
  ];
  const tx = new Uint8Array([...compactU16(1), ...new Uint8Array(64), ...msg]);

  // 3. Simulate. sigVerify:false means our 64 zero bytes pass as a "signature".
  const sim = await rpc("simulateTransaction", [
    Buffer.from(tx).toString("base64"),
    { sigVerify: false, replaceRecentBlockhash: true, encoding: "base64" },
  ]);

  console.log(`simulated: Transfer of 1 base unit (0.000001 USDC)`);
  console.log(`  from ${source}`);
  console.log(`  to   ${dest}`);
  console.log(`err:           ${JSON.stringify(sim.value.err)}`);
  console.log(`unitsConsumed: ${sim.value.unitsConsumed}`);
  for (const line of sim.value.logs ?? []) console.log(`  ${line}`);
}

main().catch((e) => { console.error(e); process.exit(1); });
```

**5. Córrelo y lee el medidor.**

```bash
npx tsx@4.20.5 transfer-cu.ts
```

Mi corrida, el mismo día:

```text
simulated: Transfer of 1 base unit (0.000001 USDC)
  from 7KJjY7rArbydeLBF7gQ5LdqXRKRYyPArT99NEctsHsgU
  to   FzbcyEZ9m8xjtergWgWDq7mfPoHEbboBF791B6cTpzbq
err:           null
unitsConsumed: 76
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 76 of 200000 compute units
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
```

Ahí está, de la propia boca del runtime: `consumed 76 of 200000 compute units`. Ya no es mi afirmación. Es tuya. `err: null` quiere decir que la transferencia de verdad tendría éxito si se firmara; las direcciones de cuenta específicas de tu corrida pueden diferir de las mías, ya que el script elige en vivo las dos cuentas de USDC más grandes de la billetera.

**6. Checkpoint.** Terminaste el lab cuando puedes señalar cuatro cosas en la salida de tu propia terminal: los dos owner programs (`Tokenz...` y `Tokenkeg...`), la brecha de tamaño de 866 contra 82, el hook `null` en una extensión que existe, y la línea donde el runtime reporta 76 CU. Si en cambio alguno de los scripts falló, las causas abrumadoramente probables son Node por debajo de 20 (sin `fetch`) o el RPC público limitándote la tasa; espera treinta segundos y vuelve a correrlo, o mete cualquier endpoint RPC que ya uses.

![Un gráfico de barras que muestra Transfer cayendo de 4,645 a 76 CU y TransferChecked de 6,200 a 105 después del cambio de motor, con las cifras actuales medidas en vivo y las cifras históricas citadas.](assets/v06-chart.webp)

## Challenge

Solo, sin guía. Escribe el informe sobre el que un compañero de equipo podría actuar, cuatro líneas, con tus propias palabras:

1. ¿Cuántas extensiones TLV lleva el mint de PYUSD, y cuáles dos están configuradas pero dormidas? Di qué quiso decir "dormida" concretamente en los bytes que leíste.
2. ¿Cuánto cuesta en CU un Transfer SPL clásico, según tu propia simulación, y por qué este curso se niega a dejarte memorizar ese número?
3. Elige un token más, cualquier dirección de mint de tu propia billetera o de un explorador. Apunta `read-pyusd.ts` a él y clasifícalo: ¿qué programa es su dueño, y cuántas extensiones lleva?
4. Una frase: ¿por qué no puedes elegir una primitiva de activo que no puedes leer?

Si la línea 4 sale parecida a "porque las reglas reales de la primitiva viven en los bytes del mint, no en su nombre ni en sus docs", tienes la lección. Si sale "porque leer en general es bueno", corre de nuevo la comparación con USDC y mira más de cerca las dos líneas `undefined`.

## Lo que decodificaste, y lo que no

Honestidad rápida sobre el día. No aprendiste los mecanismos de las extensiones, no escribiste un decodificador, y todavía no puedes decir por qué un hook o un delegado permanente pondrían nervioso a un DEX. Lo que sí hiciste: leíste un mint de producción vivo que la mayoría de sus tenedores nunca va a mirar, cazaste una distinción de programas que todavía muerde a ingenieros en ejercicio, y mediste un costo del runtime en vez de citar uno. Produjiste dos números que no puedes explicar, a propósito, desde un arranque en frío, en menos de una hora. Esa es una habilidad real, y es aquella sobre la que se apoya todo lo demás de acá.

La próxima lección, se acaban los préstamos. Construyes el decodificador tú mismo, desde el mint pelado de 82 bytes hasta el recorrido TLV completo, y se vuelve R1, el inspector de mints, la primera herramienta del kit de Overgrowth y la que el resto del módulo sigue llamando. Trae los dos scripts de hoy; vamos a abrir la caja negra.

Si algo de esta lección se leyó mal contra lo que imprimió tu propia terminal, confía en la terminal y avísame. Ese reflejo también es el curso.

¡Feliz decodificación! 🌱
