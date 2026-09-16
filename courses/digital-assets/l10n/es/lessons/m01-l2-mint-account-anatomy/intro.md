# Anatomía de mint y cuenta desde bytes crudos

En m01-l1 corriste un script provisto que decodificó las ocho extensiones de PYUSD y viste una transferencia clásica costar 76 CU. Leíste números que todavía no podías explicar. Peor aún, le creíste al decodificador en todo: el modo `jsonParsed` del RPC te entregó una lista ordenada de nombres de extensión, y no tenías forma de comprobar si te decía la verdad. Eso es una caja negra, y hoy la abrimos.

Empieza por mirar la materia prima con tus propios ojos. Nada de librerías todavía. Guarda esto como `peek.ts` donde sea:

```typescript
// peek.ts - three bytes of PYUSD's mint, before any parser exists.
// Zero npm dependencies. Run: npx tsx@4.20.5 peek.ts
const RPC = "https://api.mainnet-beta.solana.com";
const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

async function peek() {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0", id: 1, method: "getAccountInfo",
      params: [PYUSD, { encoding: "base64" }],
    }),
  });
  const { result } = await res.json();
  const data = Buffer.from(result.value.data[0], "base64");

  console.log(`length:        ${data.length} bytes`);
  console.log(`byte 165:      ${data[165]}   (the account-type discriminator)`);
  console.log(`bytes 166-169: ${[...data.subarray(166, 170)].join(" ")}  (first TLV header)`);
}

// Wrapped in a function deliberately: a .ts file with no imports is a script,
// not a module, and top-level await is a module-only privilege.
peek();
```

```bash
npx tsx@4.20.5 peek.ts
# tsx pinned at 4.20.5, verified 2026-08-22; npx fetches it on first run.
```

Mi ejecución, hoy, 2026-08-22:

```text
length:        866 bytes
byte 165:      1   (the account-type discriminator)
bytes 166-169: 3 0 32 0  (first TLV header)
```

Los mismos 866 bytes que la lección pasada, pero esta vez nada los parseó por ti. Ese `1` parado en el byte 165 y esa pequeña tira `3 0 32 0` son toda la anatomía de esta lección, y al terminarla los vas a leer como lees español. La diferencia con la vez anterior es la diferencia entre confiar en un decodificador y ser dueño de la lectura: cuando tu inspector imprime el conjunto exacto de extensiones de un mint que nunca hiciste, calculado por tu propia aritmética de cursor, ya nadie puede salirte con vaguedades.

## Resumen

Esta es una lección de construcción, y entrega la primera herramienta real del kit Overgrowth: **R1, el inspector `decode-mint`**. Dada una dirección de mint y un RPC, imprime los campos base (autoridad de mint, supply como BigInt, decimales, autoridad de congelamiento), reporta si la cuenta es un mint pelado de 82 bytes o un mint extendido de 165 más 1 bytes, y enumera cada extensión TLV como `{name, type, length}`. Ya viene con `test-decode-mint.ts`, un script de asserts que decodifica un mint conocido y fijado y falla ruidosamente ante cualquier discrepancia. Ese script es el primer criterio de aceptación del curso, y el patrón que establece, asserts simples contra un objetivo fijado, se reutiliza en cada peldaño posterior.

La teoría cubre toda la anatomía de bytes crudos sobre la que se para el curso: el layout del mint pelado de 82 bytes, por qué los mints extendidos rellenan hasta 165 bytes más un byte discriminador, el formato de entrada TLV, la matemática de longitud de cuenta detrás de `try_calculate_account_len`, y por qué cada u64 de este curso es un BigInt. La ayuda se repliega según un calendario: la lección pasada corriste scripts terminados; hoy te muestro el slice base y el chequeo de si es extendido de punta a punta, pero el loop TLV es un TODO que llenas tú. El challenge posterior es totalmente solo. El siguiente peldaño es una lección conceptual, guiada de punta a punta por diseño; el repliegue se retoma en m01-l4, donde cuatro de las cinco reglas del validador son tuyas para portar.

## La anatomía, desde el offset cero

Los datos de una cuenta son apenas un array de bytes. El programa dueño de la cuenta decide qué significan esos bytes, y para los mints el significado está publicado en el código fuente del programa. Todo lo de abajo es ese layout publicado, y nada en él es secreto ni ingenioso. Es el plano de un edificio en el que has estado viviendo.

### La base de 82 bytes

Un mint SPL clásico es exactamente 82 bytes, y los dos programas de token usan los mismos cinco campos en el mismo orden. Aquí está el mapa, offsets incluidos, porque estás a punto de escribir código contra ellos:

![Un mapa horizontal de bytes que muestra el layout del mint de 82 bytes: una autoridad de mint opcional de 36 bytes, un supply little-endian de 8 bytes, un byte de decimales, un byte de inicializado y una autoridad de congelamiento opcional de 36 bytes.](assets/v01-diagram.webp)

Dos de estos campos merecen una mirada más de cerca porque hacen tropezar a la gente.

**COption es de 36 bytes, no de 33.** Una pubkey opcional se serializa como una etiqueta u32 little-endian completa (0 para None, 1 para Some) seguida de la clave de 32 bytes, que siempre está presente en el buffer incluso cuando la etiqueta dice None. Así que la autoridad de mint abarca los bytes 0 a 35, y la autoridad de congelamiento abarca del 46 al 81. Si internalizaste el `Option` de Rust como un solo byte de etiqueta, este layout te va a desfasar en tres. El formato serializado gasta cuatro bytes en la etiqueta.

**El supply es un u64, y un u64 no cabe en un number de JavaScript.** `Number.MAX_SAFE_INTEGER` es 2^53 menos 1, unos 9.0 mil billones; un u64 llega como máximo a unos 18.4 trillones, tres órdenes de magnitud más arriba. Esta no es una preocupación teórica que puedas postergar. Cuando decodifiqué hoy el mint del USDC clásico su supply marcó 7,923,463,957,481,104 unidades base, que es cerca del 88 por ciento del camino hacia el entero más grande que JavaScript puede representar con exactitud. Un orden de magnitud más en el crecimiento de stablecoins, o cualquier token de 9 decimales con un supply grande, y `Number(supply)` redondea en silencio. En silencio es la palabra clave: sin excepción, sin advertencia, apenas un saldo equivocado en producción. Así que la regla de este curso es absoluta y aburrida: **el supply y los montos u64 son BigInt de punta a punta**, leídos con `DataView.getBigUint64`, impresos con la `n` todavía conceptualmente pegada, nunca rebotados por `Number`. He entregado la otra versión de esta decisión, hace años, en un dashboard que mostraba supplies de tokens. Funcionaba en todas las pruebas, porque los supplies de prueba son chicos. Ese es exactamente el tipo de bug que es.

![Una comparación que muestra el supply de PYUSD en el 7.6 por ciento del límite de enteros seguros que tiene JavaScript, el de USDC en el 88 por ciento, y el máximo de u64 mucho más allá, terminando con la regla de mantener todo en BigInt.](assets/v02-comparison.webp)

Esa es la base. En un mint clásico pelado, ese también es el final: el byte 82 es el borde de la cuenta. La longitud misma es tu primera bifurcación de parser, y es una respuesta completa por sí sola. Una cuenta que mide exactamente 82 bytes es un mint pelado sin extensiones, sin discriminador, sin región TLV, punto final. Nada más que revisar.

### Por qué los mints extendidos vuelven a empezar en el byte 165

Ahora la parte rara. El mint de PYUSD es de 866 bytes, y su primera extensión no empieza en el byte 82. Empieza en el byte 166. Entre la base y las extensiones hay 83 bytes de relleno en cero y un byte misterioso. ¿Por qué un formato desperdiciaría 83 bytes por mint?

Por una decisión tomada años antes de que existieran las extensiones. El programa de token clásico verifica el tipo de sus cuentas solo por la longitud: un mint es de 82 bytes, una cuenta de token es de 165, un multisig es de 355. Esa es toda la prueba. Longitud 82, trata los bytes como un mint; longitud 165, trátalos como una cuenta de token; 355, un multisig; cualquier otra cosa, rechaza. Barato, simple, y completamente dependiente de que esos tres números nunca choquen.

Las extensiones rompen eso. Una vez que las cuentas tienen colas opcionales de longitud variable, las longitudes dejan de ser distintivas: algún mint extendido acabaría cayendo exactamente en 165 bytes, y cualquier programa que usara la prueba de longitud leería alegremente un mint como si fuera el saldo de tokens de alguien. Leer mal los bytes en un límite de tipos es como se roban los fondos, así que Token-2022 cerró la puerta de forma estructural. Toda cuenta extendida, mint o cuenta de token, se rellena más allá de la zona de colisión: primero los datos base, ceros hasta el offset 165, y después **un byte discriminador de tipo de cuenta en el offset 165**. El valor 1 significa mint. El valor 2 significa cuenta de token. Tu `peek.ts` imprimió ese `1` exacto. Después del discriminador, y solo después de él, la región de extensiones empieza en el byte 166.

El costo de este diseño es sincero y visible: un mint extendido gasta 83 bytes en relleno, con el rent pagado por todos ellos, puramente para que ninguna longitud pueda volver a ser ambigua nunca. La alternativa era un formato donde la confusión de tipos es posible y cada programa aguas abajo carga con el peso de no cometer nunca el error. Pagar 83 bytes una sola vez, al nivel del formato, para borrar una clase entera de bugs en todas partes, es un canje que los diseñadores aceptaron sin dudar, y habiendo pasado tiempo con la literatura de exploits tenían razón en hacerlo.

Esto también resuelve una pregunta práctica sobre las cuentas de token que usas a diario. Tus ATAs, las cuentas de token asociadas que guardan tus saldos, son cuentas de token de 165 bytes en el programa clásico. Una cuenta de token de Token-2022 con cualquier extensión recibe el mismo trato que un mint: se rellena (ya está en 165), y después el byte 165 lleva un 2 en vez de un 1. Mismo slot de discriminador, valor distinto. Un byte, y los mints y las cuentas de token nunca más pueden confundirse sin importar lo que las extensiones le hagan a sus longitudes.

![Dos barras de bytes que comparan un mint pelado de 82 bytes con el mint de 866 bytes de PYUSD, cuya base idéntica va seguida de relleno, el discriminador de tipo de cuenta y una región TLV de 700 bytes.](assets/v03-diagram.webp)

### El recorrido TLV

Todo lo que va del byte 166 al final de la cuenta es una secuencia de **entradas TLV**: tipo, longitud, valor, repetido. Cada entrada es un código de tipo u16 little-endian que dice de qué extensión se trata, una longitud u16 little-endian que dice cuántos bytes de valor siguen, y después exactamente esa cantidad de bytes de valor. Cuatro bytes de header, después el payload. La siguiente entrada inmediatamente después. Sin separadores, sin campo de conteo por delante, sin índice. La lista es el recorrido.

La salida de tu `peek.ts` ya contenía un ejemplo completo y resuelto. Los cuatro bytes en 166 eran `3 0 32 0`. Léelos como dos u16 little-endian: tipo = 3, longitud = 32. El tipo 3 es MintCloseAuthority, y su valor es una pubkey de 32 bytes. Así que los bytes 170 a 201 son esa autoridad, y el header de la siguiente entrada empieza en 166 + 4 + 32 = 202. En 202 encontrarías `12 0 32 0`: PermanentDelegate, otra pubkey de 32 bytes, siguiente header en 238. Y así sucesivamente, ocho veces, hasta que la última entrada termina exactamente en el byte 866, el borde de la cuenta. Cuando tu cursor cae justo en el límite de la cuenta sin que sobre nada, esa es la verificación de reconciliación que prueba que tu recorrido leyó cada byte.

![Una tabla que recorre las ocho entradas TLV de PYUSD del byte 166 al 866, mostrando el tipo, el nombre, la longitud de cada entrada, y la aritmética de cursor que cae exactamente en el límite de la cuenta.](assets/v04-annotated-code.webp)

Recorre con la vista la columna de tipo un segundo, porque mata calladamente una suposición tentadora. Los códigos van 3, 12, 1, 4, 16, 14, 18, 19. Sin ordenar. Las entradas TLV aparecen en el orden en que el emisor las inicializó, no en orden de tipo, así que tu parser nunca debe hacer búsqueda binaria ni suponer la posición. Recorres, siempre.

Ahora la trampa que le gana a esta lección su lugar en el curso, la que ni el brief ni yo vamos a dejar que aprendas en producción. Cuando terminas de leer una entrada, avanzas el cursor hasta el siguiente header. El valor mide `length` bytes, pero la entrada mide `4 + length` bytes, porque los campos de tipo y longitud se llevan dos bytes cada uno. Avanza solo por `length` y tu cursor queda 4 bytes corto, en el medio del valor que acabas de leer. El siguiente "tipo" que leas son dos bytes de la pubkey de alguna autoridad. La siguiente "longitud" son dos más. Los dos parsean bien, porque dos bytes cualesquiera parsean como un u16. De ahí en adelante, cada entrada que decodificas es basura que parece datos, y nada lanza un error. Una constante equivocada, una lectura corrupta en silencio de cada entrada siguiente. Este es el impuesto de fragilidad del parseo a mano, y por eso existe el script de asserts.

![Dos recorridos sobre los mismos bytes TLV, uno avanzando por cuatro más longitud hasta el siguiente header, el otro quedando cuatro bytes corto, así que cada lectura posterior está mal en silencio.](assets/v05-diagram.webp)

Las longitudes mismas ya te están diciendo qué vive dentro de cada valor, incluso antes de que estudiemos los mecanismos. Las dos entradas de 32 bytes, MintCloseAuthority y PermanentDelegate, son, cada una, una sola pubkey: una autoridad, nada más. TransferHook y MetadataPointer leen 64 las dos, y las dos son un par de pubkeys: una autoridad con permiso para actualizar la entrada, más la dirección a la que apunta (un programa de hook en un caso, una cuenta de metadatos en el otro). El 65 raro de ConfidentialTransferMint son dos pubkeys más un único byte de política de aprobación en el medio. Y el 174 de TokenMetadata es la única entrada de longitud variable del conjunto: guarda cadenas de verdad (nombre, símbolo, URI), así que su longitud difiere por mint mientras que la longitud de cada otra entrada está fijada por su struct. Todavía no puedes decodificar los valores, y esta lección deliberadamente no lo hace: los layouts de valor son conocimiento por extensión, y el módulo 2 los toma un mecanismo a la vez. Pero ya puedes hacer un análisis forense sorprendentemente afilado con `{name, type, length}` solo, que es exactamente la interfaz que exporta tu inspector.

Una nota de nomenclatura antes de que construyas, porque, si no, tres grafías de una misma extensión te van a costar media hora de confusión. La salida `jsonParsed` del RPC de la lección pasada llamaba a la quinta extensión de PYUSD `confidentialTransferFeeConfig`. El código fuente de Rust la llama `ConfidentialTransferFeeConfig`. El enum del cliente JS fijado, que tu inspector va a usar para los nombres, imprime `ConfidentialTransferFee`. La misma extensión, código de tipo 16 en las tres. El u16 es la identidad; los nombres son apenas fachadas por toolchain encima de él, que es exactamente por qué tu inspector imprime tanto el nombre como el número.

Y no es la única: en el pin de arriba, el cliente también llama al tipo 25 `ScaledUiAmountConfig` donde el código fuente dice `ScaledUiAmount`, y al tipo 26 `PausableConfig` donde el código fuente dice `Pausable`. Tres códigos, tres desacuerdos, una regla — compara números, no cadenas, siempre que dos toolchains tengan que ponerse de acuerdo sobre la misma extensión. m01-l4 te hace ponerlo en práctica: su validador es un port del Rust, así que darle de comer los nombres del lado cliente de tu inspector es exactamente cómo consigues que una regla correcta rechace un mint que es perfectamente legal.

### La matemática del espacio: try_calculate_account_len

Ahora puedes leer cualquier mint existente. Crear uno plantea el problema inverso: antes de que la cuenta exista tienes que decirle al programa del sistema cuántos bytes asignar y cubrir con rent, y después no puedes escaparte de una respuesta equivocada a fuerza de realloc, porque las extensiones de un mint se fijan en la creación. El modelo de ese cálculo vive en el código fuente Rust de Token-2022 como `ExtensionType::try_calculate_account_len`, y lo leemos en vez de escribirlo (en esta lección no se escribe Rust). Lo que hace el código fuente: si el conjunto de extensiones está vacío, devuelve la longitud base pelada. Si no, empieza desde 165 más 1, la base rellenada más el discriminador, y suma `4 + value_length` por cada extensión pedida, exactamente la aritmética que tu recorrido TLV acaba de verificar al revés.

Los números resultantes son agradablemente no redondos, y cada uno detalla su propio costo. Un mint con solo NonTransferable, una extensión marcadora cuyo valor es de cero bytes, es de 165 + 1 + 4 + 0 = 170 bytes. Un mint con solo TransferFeeConfig, cuyo valor es de 108 bytes de autoridades, montos retenidos, y dos calendarios de comisiones, es de 165 + 1 + 4 + 108 = 278. Esas dos cifras salen directamente de los ejemplos token-2022 de program-examples de solana-developers, y ahora también salen de tu propia aritmética. Cada extensión se paga a sí misma en rent de cuenta, y la longitud sola te dice la factura. El lab calcula estos con el espejo del mismo modelo que tiene el cliente JS, así que nunca los memorizas, los regeneras.

![Un gráfico de barras con los tamaños de mint: 82 bytes pelado, 170 con NonTransferable, 278 con una config de comisión de transferencia, y 866 para PYUSD, contra el piso de 166 bytes de la base extendida.](assets/v06-chart.webp)

Aquí está el trade-off honesto de toda esta lección, dicho una vez antes de que construyas. Leer bytes crudos te da una verdad de base que ninguna UI de billetera, ningún parser de RPC y ningún SDK pueden ocultarte. También es frágil por naturaleza: los offsets se corren a medida que se agregan extensiones, los códigos de tipo tienen que mapear bien, y ya viste cómo un avance de cursor equivocado corrompe en silencio todo lo que viene después. El parseo a mano es la jugada correcta para enseñar y la jugada equivocada para producción. Los clientes publicados existen precisamente para que rara vez hagas esto a mano, y el paso 6 del lab usa uno para revisar tu trabajo. Pero cuando una billetera muestra una cosa y un explorador muestra otra, los bytes son el desempate, y después de hoy estás calificado para consultarlos. Ese es el punto: no reemplazar las herramientas, sino dejar de ser rehén de ellas.

Una nota de honestidad relacionada, y tu segundo toque de color del día: ahora conoces el costo exacto en bytes de cada extensión, pero nadie publica el costo de cómputo de cada extensión. Los números de CU por extensión no aparecen en ninguna parte de la documentación ni del código de Token-2022; revisé los dos (2026-08-21) y volví con las manos vacías, que es por lo que una lección posterior de este curso mide esos costos en un lab en vez de citar una tabla. La anatomía la puedes leer; el costo de runtime lo tienes que medir.

## Lab: construye R1, el inspector de mints

Manos al teclado, unos cuarenta minutos. Este workspace lo consumen lecciones posteriores, así que ponlo donde vive el curso.

**1. Levanta el workspace y fija el toolchain.** La convención de workspace del curso, que se sostiene desde aquí hasta el capstone, es una carpeta por lección bajo `labs/`. Las lecciones posteriores importan entre esas carpetas por ruta relativa, así que los nombres son estructurales y no decorativos:

```bash
mkdir -p labs/m01-l2 && cd labs/m01-l2
npm init -y
npm pkg set type=module
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.20.5
```

tsx pasa al workspace como dependencia de desarrollo en el mismo pin 4.20.5 que usaban los one-liners de m01-l1, así que los comandos `npx tsx` pelados de abajo resuelven a esta copia local fijada en vez de a lo que npx bajaría hoy.

Los pins merecen un párrafo, porque aquí tuve que tomar una decisión de verdad y deberías verla. Al 2026-09-05, el `latest` de npm para `@solana/kit` es 8.2.0, pero "instalar latest" no es como se fija un workspace de Solana. La regla que de verdad decide el número: fija el major de kit contra el que hacen peer los clientes `@solana-program/*` de tu workspace. El cliente de este workspace es `@solana-program/token-2022`, cuya línea actual es **0.15.0** y hace peer con kit ^7.0.0 — 0.16.0 ya saltó su rango de peers a ^8, así que instalar eso contra kit 7 falla el chequeo de peers de plano. El kit más nuevo dentro del rango ^7 es 7.1.1, y con eso queda resuelto. Verifiqué esos rangos de peers contra npm hoy en vez de confiar en cualquier documento, incluido este: van a derivar, y `npm view @solana-program/token-2022 peerDependencies` toma diez segundos. Así que el par es kit 7.1.1 más token-2022 0.15.0, versiones exactas, nada de carets en serio. Si la instalación de arriba terminó sin una queja de `ERESOLVE`, tu workspace coincide con el mío.

**2. Escribe el inspector, primero la parte resuelta.** Antes del código, sostén una vez todo el flujo de decisión en tu cabeza. Es corto, y cada rama es algo que la teoría acaba de enseñar:

![Un diagrama de flujo para el inspector de mints: trae los bytes, parsea la base, reporta pelado en exactamente 82 bytes, si no revisa el byte 165 y recorre la región TLV desde el byte 166.](assets/v07-flowchart.webp)

Ahora crea `decode-mint.ts`. Todo aquí se muestra completo excepto una región: el slice base y el chequeo de si es extendido son tuyos para copiar, y el loop TLV es tuyo para escribir.

```typescript
// decode-mint.ts - R1, the Overgrowth mint inspector.
// Parses a mint account's raw bytes: 82-byte base, bare-vs-extended, TLV walk.
// Run: npx tsx decode-mint.ts <MINT_ADDRESS> [RPC_URL]
// Pins (verified 2026-09-05): @solana/kit 7.1.1, @solana-program/token-2022 0.15.0

import { pathToFileURL } from "node:url";
import { createSolanaRpc, address, getBase58Decoder } from "@solana/kit";
import { ExtensionType } from "@solana-program/token-2022";

const BARE_MINT_LEN = 82; // classic mint: full layout, nothing after
const EXTENDED_BASE_LEN = 165; // extended mint: base padded to token-account length
const ACCOUNT_TYPE_LEN = 1; // one discriminator byte at offset 165
const TLV_START = EXTENDED_BASE_LEN + ACCOUNT_TYPE_LEN; // 166

export interface TlvEntry {
  name: string;
  type: number;
  length: number;
}

export interface ParsedMint {
  kind: "bare-82" | "extended-165+1";
  mintAuthority: string | null;
  supply: bigint;
  decimals: number;
  freezeAuthority: string | null;
  accountType: number | null; // 1 = Mint; null on a bare mint
  extensions: TlvEntry[];
}

const b58 = getBase58Decoder();

/** Read a COption<Pubkey>: u32 LE tag (0 = None, 1 = Some) + 32-byte pubkey. */
function readCOptionPubkey(data: Uint8Array, offset: number): string | null {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const tag = view.getUint32(offset, true);
  if (tag === 0) return null;
  return b58.decode(data.subarray(offset + 4, offset + 36));
}

/** Parse the 82-byte base + (if present) the account-type byte and TLV region. */
export function parseMint(data: Uint8Array): ParsedMint {
  if (data.length < BARE_MINT_LEN) {
    throw new Error(`account is ${data.length} bytes; a mint is at least 82`);
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);

  // The 82-byte base, same in both programs:
  const mintAuthority = readCOptionPubkey(data, 0); //  0..36 COption<Pubkey>
  const supply = view.getBigUint64(36, true); // 36..44 u64 LE
  const decimals = data[44]; // 44     u8
  const isInitialized = data[45]; // 45     bool
  const freezeAuthority = readCOptionPubkey(data, 46); // 46..82 COption<Pubkey>
  if (isInitialized !== 1) throw new Error("mint is not initialized");

  // The length IS the first tell: exactly 82 bytes means bare, no TLV region.
  if (data.length === BARE_MINT_LEN) {
    return {
      kind: "bare-82",
      mintAuthority,
      supply,
      decimals,
      freezeAuthority,
      accountType: null,
      extensions: [],
    };
  }

  // Extended: base padded to 165 bytes, then one account-type byte.
  const accountType = data[EXTENDED_BASE_LEN];
  if (accountType !== 1) {
    throw new Error(`account-type byte is ${accountType}, expected 1 (Mint)`);
  }

  const extensions: TlvEntry[] = [];
  let cursor = TLV_START;
  // TODO(you): walk the TLV region.
  // While there is room for a 4-byte header (cursor + 4 <= data.length):
  //   - read type as u16 LE at cursor, and length as u16 LE at cursor + 2
  //   - a type of 0 (Uninitialized) is padding, not an entry: stop the walk
  //   - push { name: ExtensionType[type] ?? `unknown(${type})`, type, length }
  //   - advance the cursor past the header AND the value. Not by `length`.

  return {
    kind: "extended-165+1",
    mintAuthority,
    supply,
    decimals,
    freezeAuthority,
    accountType,
    extensions,
  };
}

/** The tool's interface: given a mint address and an RPC, return the parse. */
export async function decodeMint(
  mint: string,
  rpcUrl = "https://api.mainnet-beta.solana.com",
): Promise<ParsedMint & { owner: string; dataLength: number }> {
  const rpc = createSolanaRpc(rpcUrl);
  const { value: account } = await rpc
    .getAccountInfo(address(mint), { encoding: "base64" })
    .send();
  if (!account) throw new Error(`no account at ${mint}`);
  const data = Uint8Array.from(Buffer.from(account.data[0], "base64"));
  return { ...parseMint(data), owner: account.owner, dataLength: data.length };
}

async function main() {
  const [mint, rpcUrl] = process.argv.slice(2);
  if (!mint) {
    console.error("usage: npx tsx decode-mint.ts <MINT_ADDRESS> [RPC_URL]");
    process.exit(1);
  }
  const parsed = await decodeMint(mint, rpcUrl);

  console.log(`mint:          ${mint}`);
  console.log(`owner program: ${parsed.owner}`);
  console.log(`data length:   ${parsed.dataLength} bytes -> ${parsed.kind}`);
  console.log(`mintAuthority: ${parsed.mintAuthority}`);
  console.log(`supply:        ${parsed.supply} (bigint)`);
  console.log(`decimals:      ${parsed.decimals}`);
  console.log(`freezeAuth:    ${parsed.freezeAuthority}`);
  console.log(`extensions (${parsed.extensions.length}):`);
  for (const e of parsed.extensions) {
    console.log(`  { name: ${e.name}, type: ${e.type}, length: ${e.length} }`);
  }
}

// Run the CLI only when invoked directly, so tests can import parseMint.
// (String suffix checks are a trap here: "test-decode-mint.ts" also ends
// with "decode-mint.ts". Compare resolved URLs instead.)
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
```

Tres decisiones de diseño que vale la pena nombrar, y después las partes rutinarias pueden quedarse rutinarias. `parseMint` es una función pura de bytes a estructura, sin red adentro, que es lo que la hace testeable y sobre lo que se construye el challenge. `decodeMint` la envuelve con el fetch y es la interfaz que importan las lecciones posteriores; el validador `check-combo` de m01-l4 llama exactamente a esta función, así que su forma (`extensions` como `{name, type, length}[]`) es un contrato ahora, no una preferencia de estilo. Y el mapa de nombres es el propio enum `ExtensionType` del cliente fijado y no una tabla escrita a mano: el paquete que instalamos para el paso 6 ya viene con el mapeo de u16 a nombre, generado desde el código fuente del programa, y no voy a mantener a mano una copia peor de él. (La última vez que copié a mano un enum así a un proyecto, salió una variante nueva aguas arriba y mi tabla me mintió durante un mes. Reutiliza las tablas del ecosistema.)

**3. Apúntalo primero a un mint pelado.** El USDC clásico, el mismo mint que leíste a través del parser del RPC la lección pasada:

```bash
npx tsx decode-mint.ts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

Mi ejecución de hoy:

```text
mint:          EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
owner program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
data length:   82 bytes -> bare-82
mintAuthority: BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG
supply:        7923463957481104 (bigint)
decimals:      6
freezeAuth:    7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar
extensions (0):
```

La rama de 82 bytes funciona de punta a punta con el código tal como está dado: campos base decodificados desde bytes crudos por tus propios offsets, `bare-82` reportado solo por el chequeo de longitud, supply impreso como BigInt (ese es el número del 88 por ciento de la teoría, en vivo). Tu supply va a diferir del mío; es una cuenta viva.

**4. Ahora PYUSD, y te topas con el TODO.** Córrelo contra el mint extendido:

```bash
npx tsx decode-mint.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

Con el TODO sin llenar vas a ver que los campos base decodifican bien, `866 bytes -> extended-165+1`, y después `extensions (0):`. El inspector ve la región y no puede leerla. Este es el problema de completado: **llena el loop TLV.** Tienes todo lo que necesitas: el layout del header de la teoría, el recorrido resuelto sobre estos exactos 866 bytes en la tabla de arriba, `view.getUint16(offset, true)` para lecturas de u16 little-endian, y las dos condiciones de parada en los comentarios del TODO. Son menos de diez líneas. La única línea que importa es el avance del cursor, y ya sabes por qué.

**5. Vuelve a correr y reconcilia.** Cuando tu loop está bien, el mismo comando imprime:

```text
mint:          2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
data length:   866 bytes -> extended-165+1
mintAuthority: 8Jornc27vtAYPkwDzsZVgLQchAYyC8nD7aCNPCDV8Qk2
supply:        688176370728435 (bigint)
decimals:      6
freezeAuth:    2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk
extensions (8):
  { name: MintCloseAuthority, type: 3, length: 32 }
  { name: PermanentDelegate, type: 12, length: 32 }
  { name: TransferFeeConfig, type: 1, length: 108 }
  { name: ConfidentialTransferMint, type: 4, length: 65 }
  { name: ConfidentialTransferFee, type: 16, length: 129 }
  { name: TransferHook, type: 14, length: 64 }
  { name: MetadataPointer, type: 18, length: 64 }
  { name: TokenMetadata, type: 19, length: 174 }
```

Ocho entradas, códigos de tipo sin ordenar, longitudes que coinciden con la tabla del recorrido. Si en cambio te salió una entrada correcta seguida de nombres sin sentido como `unknown(53421)`, felicitaciones, construiste el bug de cursor que sale en la sección de teoría; avanzaste solo por `length`. Arregla el avance y mira cómo la basura vuelve de golpe a ocho entradas limpias. Con franqueza, tropezar con esto a propósito una vez vale la pena, solo para ver lo plausible que se ve el desastre.

**6. Contrasta contra el cliente publicado.** Tu parser coincide con los bytes; ahora confirma que coincide con el ecosistema. Crea `crosscheck.ts`:

```typescript
// crosscheck.ts - decode the same mint with the shipped client and compare.
// Run: npx tsx crosscheck.ts
import { createSolanaRpc, address } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

async function main() {
  const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");
  const mint = await fetchMint(rpc, address(PYUSD));
  console.log(`supply:   ${mint.data.supply} (${typeof mint.data.supply})`);
  console.log(`decimals: ${mint.data.decimals}`);
  const ext = mint.data.extensions;
  if (ext.__option === "Some") {
    console.log(`extensions (${ext.value.length}):`);
    for (const e of ext.value) console.log(`  - ${e.__kind}`);
  }
}
main().catch((e) => { console.error(e); process.exit(1); });
```

Corre `npx tsx crosscheck.ts`. El cliente reporta el mismo supply (como bigint, ojo: kit hizo la misma llamada de u64 que nosotros), los mismos decimales, y los mismos ocho tipos de extensión en el mismo orden. Dos decodificadores independientes, uno de ellos tuyo, coincidiendo byte por byte. Esta es la relación sana con las herramientas publicadas: úsalas a diario, y sé capaz de auditarlas cuando dos fuentes no coinciden.

**7. Calcula las longitudes de cuenta en vez de memorizarlas.** El cliente JS espeja `try_calculate_account_len` como `getMintSize`. Crea `compute-len.ts`:

```typescript
// compute-len.ts - the account-length model, computed instead of memorized.
// Run: npx tsx compute-len.ts
import { address } from "@solana/kit";
import { getMintSize, type Extension } from "@solana-program/token-2022";

// A pubkey placeholder: sizes depend on the SET, never on the values.
const ANY = address("11111111111111111111111111111111");

// Bare classic mint: call it with no extension list at all.
console.log(`bare mint:              ${getMintSize()} bytes`);

// NonTransferable is a zero-length marker: 165 + 1 + (2 + 2 + 0) = 170.
const nonTransferable: Extension = { __kind: "NonTransferable" };
console.log(`+ NonTransferable:      ${getMintSize([nonTransferable])} bytes`);

// TransferFeeConfig carries 108 value bytes: 165 + 1 + (2 + 2 + 108) = 278.
const fee = { epoch: 0n, maximumFee: 0n, transferFeeBasisPoints: 0 };
const transferFee: Extension = {
  __kind: "TransferFeeConfig",
  transferFeeConfigAuthority: ANY,
  withdrawWithheldAuthority: ANY,
  withheldAmount: 0n,
  olderTransferFee: fee,
  newerTransferFee: fee,
};
console.log(`+ TransferFeeConfig:    ${getMintSize([transferFee])} bytes`);

// Both at once: one 165+1 base, then each entry pays its own 4-byte header.
console.log(`+ both:                 ${getMintSize([nonTransferable, transferFee])} bytes`);
```

```text
bare mint:              82 bytes
+ NonTransferable:      170 bytes
+ TransferFeeConfig:    278 bytes
+ both:                 282 bytes
```

El 170 y el 278 de la teoría, regenerados a demanda, más el caso combinado: 166 + (4 + 0) + (4 + 108) = 282, una base compartida, cada extensión pagando su propio header. Cuando dimensiones el mint de SPROUT en el trabajo de diseño del próximo módulo, esta es la herramienta que le pone precio a cada conjunto candidato de extensiones antes de que le comprometas rent.

**8. El criterio de aceptación.** Último archivo: `test-decode-mint.ts`, el script de asserts. Este es el patrón del hilo de pruebas que todo el curso reutiliza: `node:assert` pelado, un objetivo fijado, exit 1 a la primera falla.

```typescript
// test-decode-mint.ts - acceptance gate for R1, the mint inspector.
// Decodes a pinned known mint (PYUSD) and asserts the expected extension set.
// Run: npx tsx test-decode-mint.ts
import assert from "node:assert/strict";
import { createSolanaRpc, address } from "@solana/kit";
import { parseMint } from "./decode-mint.ts";

const RPC_URL = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

// The pinned expectation: PYUSD's eight TLV entries, in on-chain order,
// with the exact value lengths read on 2026-08-22. The extension SET is fixed
// at mint creation, but TokenMetadata's value holds updatable strings (name,
// symbol, URI), so that one entry's length, and the account total, can
// legitimately change. If this ever fails on `length`, re-read the live
// account before blaming your parser.
const EXPECTED = [
  { name: "MintCloseAuthority", type: 3, length: 32 },
  { name: "PermanentDelegate", type: 12, length: 32 },
  { name: "TransferFeeConfig", type: 1, length: 108 },
  { name: "ConfidentialTransferMint", type: 4, length: 65 },
  { name: "ConfidentialTransferFee", type: 16, length: 129 },
  { name: "TransferHook", type: 14, length: 64 },
  { name: "MetadataPointer", type: 18, length: 64 },
  { name: "TokenMetadata", type: 19, length: 174 },
];

async function main() {
  const rpc = createSolanaRpc(RPC_URL);
  const { value: account } = await rpc
    .getAccountInfo(address(PYUSD), { encoding: "base64" })
    .send();
  assert.ok(account, "PYUSD mint account not found");

  const data = Uint8Array.from(Buffer.from(account.data[0], "base64"));
  const parsed = parseMint(data);

  // Base: correctly reported as extended, not bare.
  assert.equal(parsed.kind, "extended-165+1", "PYUSD must report extended");
  assert.equal(parsed.accountType, 1, "account-type byte must be 1 (Mint)");
  assert.equal(parsed.decimals, 6, "PYUSD has 6 decimals");

  // Supply is a bigint end to end: u64 does not fit a JS number safely.
  assert.equal(typeof parsed.supply, "bigint", "supply must be a bigint");
  assert.ok(parsed.supply > 0n, "live PYUSD supply is positive");

  // Every printed TLV entry matches the expected {name, type, length} set.
  assert.deepEqual(parsed.extensions, EXPECTED, "TLV extension set mismatch");

  // The lengths must reconcile with the account size: nothing skipped.
  const tlvBytes = parsed.extensions.reduce((n, e) => n + 4 + e.length, 0);
  assert.equal(166 + tlvBytes, data.length, "TLV walk must cover the account");

  console.log(`ok - ${parsed.extensions.length} extensions on ${PYUSD}`);
  console.log(`ok - base reported as ${parsed.kind}, supply is bigint`);
  console.log(`ok - 166 + ${tlvBytes} TLV bytes = ${data.length} bytes total`);
}

main().catch((e) => {
  console.error("FAIL:", e.message);
  process.exit(1);
});
```

**9. Checkpoint.** Corre el criterio:

```bash
npx tsx test-decode-mint.ts
```

```text
ok - 8 extensions on 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
ok - base reported as extended-165+1, supply is bigint
ok - 166 + 700 TLV bytes = 866 bytes total
```

Si la ejecución muere antes de cualquier assert con un error de fetch o un `429`, ese es el RPC público gratuito limitándote la tasa, no tu código: espera treinta segundos, o pon `RPC_URL` en cualquier endpoint que ya uses (el script lo lee del entorno exactamente para este momento). Si muere en el primer assert con `account not found`, revisa la constante del mint carácter por carácter; las direcciones base58 no sobreviven a un copy-paste parcial, y el error es indistinguible de que la cuenta genuinamente no exista.

Tres líneas `ok` y un código de salida cero, y R1 es real: el inspector enumera el conjunto TLV completo de un mint vivo, reporta pelado contra extendido correctamente, y lo prueba contra una expectativa fijada. Fíjate en lo que te compra el último assert y que mirar a ojo nunca te va a dar: el reduce sobre `4 + e.length` recalcula el tamaño de la cuenta desde tu propio parseo, así que el bug de cursor exacto del paso 5 no puede pasar este criterio ni aunque los nombres basura hayan quedado plausibles. Si la ejecución falla en un `length` en cambio, vuelve a leer el comentario arriba de `EXPECTED` antes de tocar tu parser: las pruebas fijadas contra cuentas vivas cargan una suposición con fecha, y la jugada honesta es volver a verificar el objetivo, no debilitar el assert.

**10. Suéltalo de la correa.** Antes del challenge, pasa cinco minutos apuntando `decode-mint` a mints que nadie te asignó: algo de tu propia billetera, algo que esté de moda en un explorador, el mint de Token-2022 más raro que puedas encontrar.

```bash
# any mint, any RPC; the second argument is optional
npx tsx decode-mint.ts <MINT_ADDRESS_FROM_YOUR_WALLET>
```
 Cada ejecución es uno de tres resultados, y los tres enseñan. Un `bare-82` con campos familiares: el mundo clásico, del todo legible para ti ahora. Un mint extendido cuya lista de extensiones explica su comportamiento antes de que llegues a leer su documentación. O un nombre `unknown(N)`: un código de tipo más nuevo que el enum del cliente fijado, que no es un bug sino una marca de tiempo, prueba de que el catálogo se mueve y tu parser se degrada con elegancia en vez de mentir. Guarda una nota del conjunto más raro que encuentres; el validador de m01-l4 va a tener opiniones sobre él.

## Challenge

Solo, sin guía paso a paso, sin espiar el código del lab. Esta es la versión de coding challenge de lo que acabas de construir: las dos computaciones centrales como funciones puras, comprobadas con pruebas, sin red en ninguna parte.

Escribe `tlv.ts` exportando dos funciones:

1. `parseTlv(data: Uint8Array, start: number): TlvEntry[]`, un parser TLV puro: recorre headers desde `start`, para en el borde de la cuenta o en un tipo de 0, devuelve entradas `{name, type, length}`. Tiene que lanzar si una longitud declarada se pasa del final del buffer (tu loop del lab nunca revisó eso; los parsers de producción tienen que hacerlo).
2. `mintLen(valueLengths: number[]): number`, el modelo de longitud de cuenta como aritmética: un array vacío da 82, si no 166 más los headers y valores por entrada.

Después `test-tlv.ts` comprobándolas con arrays de bytes construidos a mano, sin RPC: una región TLV sintética de dos entradas que construyes tú con tipos y longitudes conocidos; una entrada marcadora de longitud cero; un buffer truncado que tiene que lanzar; y las aserciones de que `mintLen([])` es 82, `mintLen([0])` es 170, y `mintLen([108])` es 278. Acepta cuando los dos archivos corren limpios bajo `npx tsx test-tlv.ts` y tu `parseTlv`, apuntado a los bytes crudos de la cuenta de PYUSD en el offset 166 (tráelos en base64 como lo hizo `peek.ts`; `decodeMint` devuelve el parseo, no los bytes), reproduce exactamente las ocho entradas que imprime tu inspector.

Si tus pruebas sintéticas pasan pero la reproducción de PYUSD difiere, no confíes en ninguna de las dos: haz diff de las dos listas de entradas y encuentra qué función está mintiendo. Esa disciplina de diff, dos caminos independientes a la misma respuesta, es lo que enseñó el paso 6, y es el hábito que sobrevive a cualquier toolchain.

## Lo que ya es tuyo

Momento de feedback, puntuado con honestidad. Construiste la primera herramienta del curso y su primer criterio de aceptación. Puedes parsear la base de 82 bytes a ciegas, sabes por qué existe el byte 165 y qué significan sus dos valores, puedes recorrer una región TLV sin librería, sabes la única constante de cursor que separa un parser que funciona de basura plausible, y tus supplies son BigInts porque has visto un supply de mainnet vivo parado en el 88 por ciento de la línea de enteros seguros que tiene JavaScript. Lo que todavía no puedes hacer: explicar qué hace realmente cualquiera de esas ocho extensiones en el momento de la transferencia, o elegir un conjunto para un mint propio. Justo a tiempo; los mecanismos son los módulos 2 y 3, y elegir está a dos lecciones de distancia.

Ahora puedes leer los bytes de cualquier mint y ver la base clásica de 82 bytes sentada debajo de las extensiones. Pero esto es lo que pasa con esa base: el programa que la sirve no es el programa que la sirvió durante seis años. La misma interfaz, los mismos offsets que acabas de memorizar, un motor completamente reescrito por debajo, y tu medición de 76 CU de m01-l1 es el recibo. Ese es p-token, el cambio de motor que ocurrió bajo los pies de todos, y es la siguiente lección.

¡Feliz parseo! 🌱
