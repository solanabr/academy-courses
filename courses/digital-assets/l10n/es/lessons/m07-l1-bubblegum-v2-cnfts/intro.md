# Bubblegum v2: acuña un millón de NFTs por el precio de una laptop

## Resumen

La lección pasada cerró el arco de los estándares de NFT. Token Metadata es legacy, Metaplex Core es el estándar recomendado para el trabajo nuevo de NFT, y el rule set todo-Pass que decodificaste sobre el emblemático legacy resultó no imponer nada en absoluto. Dos lecciones atrás, en m06-l2, construiste el artefacto en el que se apoya esta lección: una colección Core Overgrowth Almanac verificada, un activo cuyo plugin Royalties sí pasa por una verificación de programa, una impresión Edition numerada, y un badge Founding-Farmer congelado en su billetera para siempre. Cuatro activos. Acuñados a mano, uno por uno.

Ahora Overgrowth necesita repartir un crate Harvest a cada jugador. Digamos que un millón de ellos.

Al rent de cuenta clásica ese airdrop cuesta una casa, y no lo has entregado porque nadie puede pagarlo. Así que antes de cualquier teoría, ponle precio tú mismo. Crea un directorio de trabajo, instala los SDKs, y deja que el dimensionador de account-compression responda la pregunta:

```bash
mkdir -p overgrowth && cd overgrowth
npm install @metaplex-foundation/mpl-bubblegum@5.1.0 \
  @metaplex-foundation/mpl-account-compression@0.0.1 \
  @metaplex-foundation/mpl-core@1.10.0 \
  @metaplex-foundation/umi@1.5.1 \
  @metaplex-foundation/umi-bundle-defaults@1.5.1 \
  @metaplex-foundation/digital-asset-standard-api@2.0.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Pins leídos de npm el 2026-08-22, la semana en que se escribió esta lección. `mpl-bubblegum` 5.1.0 se publicó el 2026-08-17, así que es reciente; el lado on-chain es una línea Rust aparte y se mueve según su propio calendario. Revisa los dos antes de fijar nada de larga vida.

```typescript
// overgrowth/scratch-cost.ts
import { getMerkleTreeSize } from "@metaplex-foundation/mpl-account-compression";

const RPC = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";

// Rent is (128 bytes of account header + your data) x a per-byte rate, and
// that rate is a NETWORK PARAMETER, not a constant a course gets to bake in.
// SIMD-0437 is stepping it down and the clusters are on different steps, so
// ask the one you will actually pay on: a 0-byte account's rent-exempt
// minimum is exactly the header times the rate.
async function lamportsPerByte(): Promise<number> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMinimumBalanceForRentExemption", params: [0] }),
  });
  const { result } = (await res.json()) as { result: number };
  return result / 128;
}

async function main(): Promise<void> {
  const rate = await lamportsPerByte();
  const bytes = getMerkleTreeSize(20, 256, 14);
  const sol = ((128 + bytes) * rate) / 1_000_000_000;

  console.log(`${rate.toLocaleString()} lamports per byte on ${RPC}`);
  console.log(`${bytes.toLocaleString()} bytes of account`);
  console.log(`${sol.toFixed(3)} SOL of rent for ${(2 ** 20).toLocaleString()} leaves`);
  console.log(`${(sol / 2 ** 20).toFixed(8)} SOL per cNFT`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

`npx tsx scratch-cost.ts` imprime cuatro líneas. Las mías, contra devnet el 2026-09-06:

```
5,080 lamports per byte on https://api.devnet.solana.com
1,223,352 bytes of account
6.215 SOL of rent for 1,048,576 leaves
0.00000593 SOL per cNFT
```

Una cuenta. Seis SOL y pico. Un millón de NFTs. Esa tercera línea son unos 0.000006 SOL cada uno. Ponle precio al ancla de $150/SOL que esta lección usa de punta a punta y el árbol entero sale por menos de mil dólares: el precio de una laptop decente, para un airdrop que las cuentas por activo cotizan en cientos de miles. (Si prefieres el ancla chica: lo que cuesta un café en SOL cubre unos cuantos miles de hojas en crates.)

Tu primera línea puede que no diga 5,080, y si no lo dice, aquí no hay nada roto. Metaplex publica 8.5012 SOL para un árbol de un millón de hojas, y esa era la respuesta correcta a 6,960 lamports por byte, la tasa que cobraba cada cluster antes de que el SIMD-0437 empezara a bajarla por etapas. Su fila no es exactamente este árbol, y el paso 3 hace explícita la diferencia: ellos publican profundidad 20 con buffer 1024 y canopy 13, esta lección construye buffer 256 y canopy 14, y los dos quedan a 0.17% de distancia por coincidencia y no por acuerdo. Mainnet pasó a 6,333 el 2026-09-03; devnet está un paso más adelante, en 5,080; un fork de surfpool todavía servía 6,960 cuando lo revisé el 2026-09-06, porque un fork hereda las cuentas de mainnet y no necesariamente su esquema de rent. Los mismos bytes, tres facturas distintas. Este es el primero de muchos números de este curso que pertenece al mundo y no a la página, y la razón por la que el script pregunta en vez de afirmar.

Hoy construyes ese árbol de verdad. El repliegue de la ayuda, en voz alta: la teoría y la derivación del costo del árbol están trabajadas por completo, la especificación del árbol es tuya para elegirla y defenderla (el paso 5 te hace derivar la profundidad y justificar el buffer y el canopy antes de acuñar tu primer crate), y el crate de logro soulbound más su demostración de rechazo de transferencia son enteramente tuyos, en solitario, con solo las dos firmas de función fijadas para ti.

## Sin cuenta, solo una hoja

### El número que fuerza el diseño

Aquí está el dolor en dólares, tasado desde el piso hacia arriba. De todas las cuentas por activo de Solana, la más barata es una cuenta de token SPL pelada de 165 bytes; pregúntale a un cluster cuánto cuesta una (`solana rent 165`) y mainnet respondió 0.00185 SOL el 2026-09-06. Ese es el comparador más caritativo posible: cualquier forma real de NFT cuesta más. Acuña un millón de cualquier cosa que necesite siquiera esa cuenta mínima y estás reteniendo como rehén del orden de 1,900 SOL de rent; a $150 por SOL eso es casi $300,000, bloqueados, para repartir un crate. Y los activos Core que entregaste el módulo pasado no te rescatan: a los ~0.003 SOL por activo que publica el proveedor, un millón de crates Core sale como 3,000 SOL. Barato por activo, unos cuarenta centavos de dólar cada uno a ese precio de SOL, sigue siendo una casa a escala de flota. Cualquier cosa por activo es el problema.

Ese número no es un supuesto que alguien inventó para un curso. Solana ya se había pasado por mucho de los 500 millones de cuentas y sumaba como un millón al día para noviembre de 2024, que es exactamente la presión que produjo la compresión de estado en primer lugar (Helius lo documentó cerca de su keynote de compresión ZK, 2024-11-25; toma los conteos como su instantánea, no como una lectura en vivo). Cada una de esas cuentas es la RAM de un validador. La blockchain estaba haciendo crecer un problema de almacenamiento más rápido de lo que hacía crecer usuarios.

Así que, el replanteo. En realidad no necesitas un millón de cuentas. Necesitas poder *demostrar*, para cualquier crate, que existe y quién es su dueño. Esos son requisitos distintos, y solo el segundo es estructural.

![Una comparación a dos columnas entre un millón de cuentas de token clásicas que cuestan unos dos mil SOL de rent y un árbol Bubblegum v2 de la misma capacidad que cuesta SOL de un solo dígito, con lecturas solo por DAS y escrituras que cargan prueba.](assets/v01-comparison.png)

### Qué es en realidad una hoja

La compresión de estado guarda un hash de tu activo, no tu activo.

En concreto: Bubblegum toma los metadatos del crate (nombre, URI, comisión del vendedor, creators, colección, dueño y un nonce), los hashea en un solo valor de 32 bytes, y escribe ese valor en un slot de un **árbol de Merkle concurrente** on-chain. Ese slot es la **hoja**. Cada par de hojas se hashea junto en un padre, cada par de padres en un abuelo, hasta llegar a una sola **raíz** de 32 bytes guardada en la cuenta del árbol. Nada más de tu crate está on-chain. No hay cuenta de mint, no hay cuenta de token, no hay PDA de metadatos, no hay cuenta de activo Core. Hay un hash, sentado en un slot, en una cuenta grande que pagaste una vez.

Si has usado git, ya tienes la intuición. Un hash de commit no contiene tu repositorio. Se compromete con él, tan precisamente que cambiar un byte en cualquier lado cambia el hash, y tan barato que puedes nombrar un millón de archivos con 32 bytes. Una raíz de Merkle es ese mismo truco con una prueba adosada: dados una hoja y el hash hermano en cada nivel hacia arriba del árbol, cualquiera puede recomputar la raíz y contrastarla con la que está on-chain. Veinte niveles para un millón de hojas, porque eso es lo que `log2` hace por ti.

Esa prueba es todo el trato. Dejaste de pagar por almacenamiento y empezaste a pagar por pruebas.

![Un camino resaltado sube por un árbol de Merkle de veinte niveles desde una hoja hasta la raíz on-chain, con los veinte hashes hermanos sombreados formando una prueba de 640 bytes.](assets/v02-diagram.png)

Vale la pena ser preciso sobre lo que ese intercambio cuesta de verdad, porque la asimetría es toda la razón por la que la compresión es viable y no apenas ingeniosa. Guardar un activo como cuenta es O(1) para leer y O(n) en rent sobre n activos, y el rent es el recurso caro porque es memoria de validador retenida para siempre. Guardar un activo como hoja es O(1) en rent sobre n activos, porque la cuenta del árbol tiene un tamaño fijo sin importar qué tan llena esté, y O(log n) por escritura, porque una prueba es un hash hermano por nivel. Duplicar tu supply de un millón a dos millones no duplica el rent. Suma un nivel, lo que suma un hash hermano a cada prueba y treinta y dos bytes a cada escritura. Ese es el canje que hace el diseño: convierte un costo lineal de almacenamiento en un costo logarítmico de ancho de banda. El ancho de banda lo puedes agrupar, cachear y acortar con un canopy. El rent solo lo puedes pagar.

El corolario es lo que la gente se pierde. La compresión no es "NFTs, más baratos." Es una curva de costo distinta, y las curvas se cruzan. Por debajo de unos pocos miles de activos una cuenta Core por activo es genuinamente competitiva y muchísimo más simple de operar. Por encima de cien mil no hay discusión que tener. Todas las decisiones interesantes viven en el medio, y dependen de la tasa de escritura más que del conteo.

Un nombre del próximo visual necesita su presentación antes de que te lo encuentres ahí: el **programa Noop**. Es un programa desplegado que a propósito no hace nada cuando se lo invoca; todo su valor está en que los datos que le pasas terminan en los logs de la transacción. Al momento del mint Bubblegum le hace CPI al programa Noop con los metadatos legibles completos del crate, así que los logs se vuelven el único lugar donde esos datos llegan a escribirse, y los indexadores reconstruyen todo lo que te sirven reproduciendo esos logs. La blockchain misma solo guarda el hash.

El **asset id** sale del mismo diseño. Un cNFT no tiene cuenta, así que necesita alguna dirección canónica por la cual ser referido, y Bubblegum la deriva: el asset id es `PDA(tree, leaf index)`. Determinista, derivable offline, estable para siempre. Vas a derivar uno en el lab con `findLeafAssetIdPda` y después ver cómo un proveedor de DAS te devuelve la misma cadena.

![Una sola cuenta de árbol contiene la raíz, el canopy, el changelog buffer y los slots de hojas junto a un índice DAS separado de metadatos legibles, con la inexistente cuenta por activo tachada.](assets/v03-diagram.png)

### El changelog buffer, y por qué las pruebas se ponen viejas

Ahora la parte que muerde a la gente en producción.

Una prueba de Merkle es una instantánea. Dice: *dada esta raíz, mi hoja hashea hasta ella.* En el momento en que cualquier otro acuña, transfiere o quema algo en el mismo árbol, la raíz cambia, y cada prueba que alguien tuviera en mano es ahora una prueba contra una raíz que ya no existe. En un árbol que hace una escritura al día, nadie lo nota. En un árbol que hace un drop de crates, todos lo notan a la vez.

Para eso está `max_buffer_size`. La cuenta del árbol guarda un **changelog buffer** de los últimos N cambios de raíz, así que una prueba que era válida hace unas pocas escrituras todavía puede reproducirse hacia adelante y ser aceptada. El buffer 64 quiere decir que como 64 escrituras concurrentes pueden caer en un slot antes de que las pruebas empiecen a rebotar. El buffer 256 te compra más margen y te cuesta bytes. Por esto la estructura se llama árbol de Merkle *concurrente* y no solo árbol de Merkle: sin el buffer, un árbol se serializaría a una escritura por slot, y al objetivo de ~400ms por slot de Solana, tu drop de un millón de crates tardaría como cuatro días y medio.

La trampa dicha sin rodeos: **vuelve a pedir la prueba inmediatamente antes de cada escritura.** No al principio de tu script. No una vez por lote. Inmediatamente antes. El helper que vas a usar en el challenge, `getAssetWithProof`, hace una ida y vuelta fresca a DAS en cada llamada exactamente por esta razón, y si cacheas su resultado a lo largo de un lote de transferencias vas a recibir un chorro de errores de hashing-mismatch que parecen un bug en tu código y no lo son.

![Un diagrama de flujo de tres carriles que contrasta una prueba que verifica directamente, una reproducida hacia adelante desde el changelog buffer tras escrituras concurrentes, y una rechazada porque la raíz se salió del buffer por antigüedad.](assets/v04-flowchart.png)

### El canopy es el dinero

Veinte niveles de profundidad quiere decir que una escritura de cliente tiene que aportar veinte hashes hermanos. Veinte claves públicas, a 32 bytes cada una, viajando en la transacción. Eso son 640 bytes de prueba antes de que hayas escrito una sola instrucción, y el presupuesto de transacción de Solana no es lo bastante generoso como para que lo ignores. Empuja los nodos de prueba más allá del límite y tu escritura no se pone más lenta, deja de caber.

El **canopy** es el arreglo: cachea los K niveles superiores de nodos internos dentro de la propia cuenta del árbol. Si la blockchain ya conoce los 14 niveles de arriba, el cliente solo envía los `max_depth - canopy_depth` nodos de abajo. Seis, en ese caso, en vez de veinte.

Lo que quiere decir que el canopy no es una perilla de ajuste, es una compra. Compras pruebas de cliente más cortas con rent. Nadie en este curso midió la fórmula exacta, así que en vez de darte una tabla en la que confiar, derívala. Esta es la parte donde el número de titular del proveedor deja de ser folclore y empieza a ser aritmética:

```typescript
// overgrowth/tree-size.ts
import { getMerkleTreeSize } from "@metaplex-foundation/mpl-account-compression";

export type TreeSpec = {
  maxDepth: number;
  maxBufferSize: number;
  canopyDepth: number;
};

/**
 * The (maxDepth, maxBufferSize) pairs the on-chain account layout is generated for.
 * Legality is per PAIR, not per field: depth 14 accepts buffer 64 but never 512.
 */
const DEPTH_BUFFER_PAIRS: ReadonlyArray<readonly [number, number]> = [
  [3, 8], [5, 8],
  [6, 16], [7, 16], [8, 16], [9, 16],
  [10, 32], [11, 32], [12, 32], [13, 32],
  [14, 64], [14, 256], [14, 1024], [14, 2048],
  [15, 64], [16, 64], [17, 64], [18, 64], [19, 64],
  [20, 64], [20, 256], [20, 1024], [20, 2048],
  [24, 64], [24, 256], [24, 512], [24, 1024], [24, 2048],
  [26, 512], [26, 1024], [26, 2048],
  [30, 512], [30, 1024], [30, 2048],
];

const DEPTHS = [...new Set(DEPTH_BUFFER_PAIRS.map(([d]) => d))].sort((a, b) => a - b);

export function depthForSupply(targetSupply: number): number {
  const depth = DEPTHS.find((d) => 2 ** d >= targetSupply);
  if (depth === undefined) throw new Error(`no supported depth holds ${targetSupply} leaves`);
  return depth;
}

export function treeBytes(spec: TreeSpec): number {
  const legal = DEPTH_BUFFER_PAIRS.some(
    ([d, b]) => d === spec.maxDepth && b === spec.maxBufferSize,
  );
  if (!legal) {
    throw new Error(
      `depth ${spec.maxDepth} / buffer ${spec.maxBufferSize} is not a supported pair`,
    );
  }
  return getMerkleTreeSize(spec.maxDepth, spec.maxBufferSize, spec.canopyDepth);
}

/** The account header the runtime charges rent on, on top of your data. */
export const ACCOUNT_HEADER_BYTES = 128;

/**
 * Read the per-byte rent rate off a live cluster instead of trusting a
 * literal. A 0-byte account's rent-exempt minimum is exactly the header times
 * the rate, so one RPC call gives you the whole schedule. SIMD-0437 is
 * stepping this number down in stages and the clusters are not in step with
 * each other, which is why no function here takes a default.
 */
export async function readLamportsPerByte(rpcUrl: string): Promise<number> {
  const res = await fetch(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMinimumBalanceForRentExemption", params: [0] }),
  });
  const { result } = (await res.json()) as { result: number };
  return result / ACCOUNT_HEADER_BYTES;
}

/** Rent-exempt minimum in SOL: (header + data) x the rate you just read. */
export function rentSol(bytes: number, lamportsPerByte: number): number {
  return ((ACCOUNT_HEADER_BYTES + bytes) * lamportsPerByte) / 1_000_000_000;
}

export function treeCostSol(spec: TreeSpec, lamportsPerByte: number): number {
  return rentSol(treeBytes(spec), lamportsPerByte);
}

/** Proof nodes a client must ship per write, once the canopy covers its top levels. */
export function proofNodesOnTheWire(spec: TreeSpec): number {
  return Math.max(spec.maxDepth - spec.canopyDepth, 0);
}
```

Barre el canopy contra una profundidad y un buffer fijos y el trade-off se imprime solo:

```typescript
// overgrowth/tree-cost.ts
import {
  depthForSupply,
  proofNodesOnTheWire,
  readLamportsPerByte,
  treeBytes,
  treeCostSol,
} from "./tree-size";

const RPC = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
const targetSupply = Number(process.argv[2] ?? 1_000_000);
const maxBufferSize = Number(process.argv[3] ?? 256);
const maxDepth = depthForSupply(targetSupply);

// Metaplex's published million-leaf row, copied exactly: depth 20 with buffer
// 1024 and canopy 13, at 8.5012 SOL. Note the spec, because it is NOT the one
// this sweep runs. The rate is the pre-SIMD-0437 6,960 their page was written
// under, kept only so the comparison below can rescale their number to yours.
const VENDOR_SPEC = { maxDepth: 20, maxBufferSize: 1024, canopyDepth: 13 };
const RATE_BEHIND_THE_VENDOR_FIGURE = 6960;
const VENDOR_SOL_AT_THAT_RATE = 8.5012;

async function main(): Promise<void> {
  const rate = await readLamportsPerByte(RPC);
  console.log(`rent rate: ${rate.toLocaleString()} lamports/byte on ${RPC}`);
  console.log(`target supply ${targetSupply.toLocaleString()} -> maxDepth ${maxDepth} (${(2 ** maxDepth).toLocaleString()} leaves)`);
  console.log("canopy   bytes        SOL      proof nodes on the wire");
  for (const canopyDepth of [0, 6, 8, 10, 12, 13, 14]) {
    const spec = { maxDepth, maxBufferSize, canopyDepth };
    const line = [
      String(canopyDepth).padStart(4),
      String(treeBytes(spec)).padStart(10),
      treeCostSol(spec, rate).toFixed(3).padStart(9),
      String(proofNodesOnTheWire(spec)).padStart(10),
    ].join("  ");
    console.log(line);
  }

  // Check the derivation against the vendor by deriving THEIR spec, not ours.
  // Their number is quoted for exactly one configuration AND at one rent rate,
  // so reproduce the configuration and rescale the rate before comparing;
  // holding a 6,960-era number against a 5,080-era derivation would be a unit
  // error, not a finding. Doing it this way makes the check real: it agrees to
  // four decimal places, which is a verification. Pointing the same check at
  // OUR spec would also have printed MATCH, and that would have been luck.
  if (maxDepth === 20) {
    const vendorAtYourRate = VENDOR_SOL_AT_THAT_RATE * (rate / RATE_BEHIND_THE_VENDOR_FIGURE);
    const theirs = treeCostSol(VENDOR_SPEC, rate);
    const ours = treeCostSol({ maxDepth: 20, maxBufferSize, canopyDepth: 14 }, rate);
    console.log(`\nvendor row : ${VENDOR_SOL_AT_THAT_RATE} SOL, depth 20 / buffer 1024 / canopy 13, quoted at ${RATE_BEHIND_THE_VENDOR_FIGURE} lamports/byte`);
    console.log(`  rescaled to your rate: ${vendorAtYourRate.toFixed(4)} SOL`);
    console.log(`derived, their spec  : ${theirs.toFixed(4)} SOL`);
    console.log(
      Math.abs(theirs - vendorAtYourRate) < 0.005 * vendorAtYourRate
        ? "MATCH - the derivation reproduces the published row"
        : "MISMATCH - re-check the spec",
    );
    console.log(`derived, this sweep  : ${ours.toFixed(4)} SOL at depth 20 / buffer ${maxBufferSize} / canopy 14`);
    console.log(`  a DIFFERENT tree that lands ${(((ours - theirs) / theirs) * 100).toFixed(2)}% away - near, not the same`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

`npx tsx tree-cost.ts` te da esto. Mi corrida, devnet, 2026-09-06 — tu columna de SOL se mueve con la tasa de tu cluster y la columna de bytes no:

```
rent rate: 5,080 lamports/byte on https://api.devnet.solana.com
target supply 1,000,000 -> maxDepth 20 (1,048,576 leaves)
canopy   bytes        SOL      proof nodes on the wire
   0      174840      0.889          20
   6      178872      0.909          14
   8      191160      0.972          12
  10      240312      1.221          10
  12      436920      2.220           8
  13      699064      3.552           7
  14     1223352      6.215           6

vendor row : 8.5012 SOL, depth 20 / buffer 1024 / canopy 13, quoted at 6960 lamports/byte
  rescaled to your rate: 6.2049 SOL
derived, their spec  : 6.2049 SOL
MATCH - the derivation reproduces the published row
derived, this sweep  : 6.2153 SOL at depth 20 / buffer 256 / canopy 14
  a DIFFERENT tree that lands 0.17% away - near, not the same
```

Lee esa tabla dos veces, porque es lo más útil de toda esta lección, y léela primero por la columna de BYTES, porque esa columna es física y la columna de SOL es política. Un árbol de un millón de hojas sin ningún canopy son 174,840 bytes; con canopy 14 son 1,223,352. Siete veces la cuenta, y por lo tanto siete veces el rent, a cualquier tasa que la red llegue a cobrar. Seis séptimos de esa cifra famosa, en lo que sea que esté denominada este mes, son canopy. Lo que compras con eso es una caída de veinte nodos de prueba a seis, que es la diferencia entre "mi instrucción de transferencia cabe" y "mi instrucción de transferencia no cabe." Y fíjate en lo que el chequeo de arriba acaba de mostrarte: la propia fila del proveedor compra casi exactamente la misma cuenta por otra vía, canopy 13 con un buffer cuatro veces más profundo, que es el mismo dinero gastado en concurrencia de escritura en vez de en largo de prueba. Su cifra de titular codifica en silencio las dos decisiones, ahora sabes cuáles son, y puedes elegir distinto a propósito.

Mi propio sesgo, por lo que valga: he visto más proyectos quemarse por un canopy subdimensionado que por uno sobredimensionado, porque el rent es un número que ves el día cero y un tamaño de transacción reventado es un número que ves el día del drop. Si no estás seguro, compra el canopy.

![Un gráfico de doble eje donde un árbol de un millón de hojas crece siete veces en bytes de cuenta a medida que el canopy se profundiza de 0 a 14, mientras los nodos de prueba por escritura caen de 20 a 6.](assets/v05-chart.png)

### El dimensionamiento es una puerta sin vuelta

`max_depth` es permanente. Un árbol de profundidad 14 contiene 16,384 hojas y contendrá 16,384 hojas mientras exista. No hay realloc, no hay migración, no hay "ya lo agrandaremos después." Cuando la última hoja se llena, ese árbol está terminado, y tu única jugada es crear otro y enseñarle a cada sistema río abajo que tu colección ahora abarca dos árboles.

Eso no es fatal, y un montón de drops de producción corren multiárbol a propósito. Pero es una decisión que quieres tomar deliberadamente el día cero en vez de descubrirla el día del drop, así que corre los números contra tu ambición real en vez de contra tu roadmap real.

| max_depth | hojas | bytes, sin canopy | bytes, canopy 8 | nodos de prueba con canopy 8 |
|---|---|---|---|---|
| 14 | 16,384 | 31,800 | 48,120 | 6 |
| 17 | 131,072 | 38,040 | 54,360 | 9 |
| 20 | 1,048,576 | 44,280 | 60,600 | 12 |
| 24 | 16,777,216 | 52,600 | 68,920 | 16 |

Esas son filas de `maxBufferSize: 64`, producidas con los helpers de `tree-size.ts` que acabas de escribir, en un loop de cinco líneas sobre profundidades a un canopy fijo de 8 (`tree-cost.ts` barre el canopy a profundidad fija, así que este barrido es su hermano de un minuto). Están en bytes y no en SOL a propósito: los bytes son una propiedad del layout de la cuenta y se te van a leer igual que a mí, mientras que el SOL es la tasa de tu cluster multiplicada por esa columna y se mueve debajo de los dos. Multiplica la columna tú mismo y aparece la parte sorprendente. La profundidad casi no cuesta nada. Pasar de dieciséis mil hojas a dieciséis *millones* suma como 20,800 bytes, unos 0.11 SOL a la tasa de devnet el día que escribí esto, porque la profundidad solo afecta el ancho de fila del changelog buffer, no el almacenamiento de hojas. Nada guarda tus hojas. No hay nada que guardar.

Así que la guía casi se escribe sola: sé generoso con la profundidad, sé deliberado con el canopy, y sé honesto con el buffer. La profundidad es casi gratis y permanente, así que sobredimensiónala. El canopy es caro y permanente, así que ponle precio contra el largo de prueba que tus escrituras pueden cargar de verdad. El buffer se sienta entre los dos y es igual de permanente: más barato que el canopy, pero dinero de verdad a escala (el salto a profundidad 20 de buffer 64 a buffer 256 suma 130,560 bytes, como 0.66 SOL a la tasa de devnet), así que ajústalo a tu peor concurrencia de escritura esperada en vez de subirlo por defecto.

No toda combinación de profundidad y buffer es legal, de paso, y esta es la razón por la que `tree-size.ts` lleva esa tabla de pares en vez de dos listas independientes de valores permitidos. El layout de la cuenta on-chain está generado para un conjunto fijo de combinaciones: la profundidad 14 acepta buffer 64, 256, 1024 o 2048 y nada más, la profundidad 26 arranca en 512, y buffer 128 no es un tamaño legal en ninguna profundidad. Chequear los dos campos por separado dejaría pasar media docena de combinaciones que el programa va a rechazar. Una mala combinación tampoco falla con gracia en runtime, falla como un error inútil de tamaño de cuenta después de que ya pagaste el rent, así que deja que la guarda tire el error antes de que gastes.

![Una tabla de cuatro filas en bytes de cuenta donde subir la profundidad del árbol de 16 mil a 16 millones de hojas suma solo 20,800 bytes, ubicando el costo real de un árbol comprimido en el canopy.](assets/v06-table.png)

### Qué cambió en v2

Bubblegum se publicó en 2023, y mucho de lo que la gente todavía repite con confianza sobre los NFTs comprimidos describe el programa V1. La versión 2 rompió la mayor parte.

La reversión que más importa: **ahora existen los cNFTs soulbound.** Durante dos años la sabiduría recibida decía que la compresión te compraba mints baratos y te costaba toda primitiva de imposición, que un cNFT no podía congelarse y nunca podía volverse no transferible, y que si querías soulbound echabas mano de un mint NonTransferable de Token-2022 en su lugar. Bubblegum v2 ya viene con `set_non_transferable_v2`, más congelar y descongelar, y cuando la colección Core del árbol lleva un `PermanentFreezeDelegate` la cosa entera impone a nivel de programa (documentación de Bubblegum v2 de Metaplex; el conjunto de instrucciones está ahí mismo en la propia interfaz del programa, que vas a llamar en el challenge). El folclore simplemente está desactualizado.

Vale la pena ver el mecanismo, porque es un byte:

```typescript
// what the client library exposes for the v2 leaf's flags field
import { LeafSchemaV2Flags } from "@metaplex-foundation/mpl-bubblegum";

// LeafSchemaV2Flags.None                 === 0
// LeafSchemaV2Flags.FrozenByOwner        === 1
// LeafSchemaV2Flags.FrozenByPermDelegate === 2
// LeafSchemaV2Flags.NonTransferable      === 4

const soulboundAndFrozen =
  LeafSchemaV2Flags.NonTransferable | LeafSchemaV2Flags.FrozenByPermDelegate; // 6
```

Soulbound es un bit. El bit 2, en un byte `flags` que las hojas V1 no tienen. Lo que explica el próximo hecho, el que te va a costar una tarde si se te pasa: **las hojas V2 no son retrocompatibles con las hojas V1.** Esquema de hoja distinto, hash distinto, superficie de programa distinta. Un árbol V1 no se puede llevar a un flujo v2 y no hay upgrade en el lugar. Si heredas un árbol V1, lo lees con instrucciones V1 y acuñas supply nuevo en un árbol v2 nuevo.

El resto del delta de v2, rápido:

| | Bubblegum V1 | Bubblegum v2 |
|---|---|---|
| Estándar de colección | colecciones de Token Metadata | colecciones MPL Core |
| Esquema de hoja | `LeafSchema` V1, sin byte de flags | `LeafSchemaV2`, con un bitfield de flags |
| Congelar / descongelar | no disponible | `freeze_v2`, `thaw_v2`, variantes con delegado |
| Soulbound | no disponible | `set_non_transferable_v2` |
| Programa de compresión | SPL account-compression | `mpl-account-compression` forkeado |
| Verificación de colección | instrucción verify, paso aparte | la colección es una pubkey simple, siempre verificada |

Esa fila del programa de compresión no es cosmética. Bubblegum v2 corre sobre el propio fork de account compression de Metaplex, `mpl-account-compression`, cuya línea de crate 0.4.2 es de donde salió el programa forkeado, con el program id `mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW`. Esa es una dirección distinta de la del programa account-compression de SPL, y el propio programa Bubblegum se sienta en `BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY`. Las líneas de crate de los programas se mueven más rápido que los borradores de lección, así que toma el número de crate como la procedencia del fork y no como el estado de npm de hoy; los pins del cliente en el bloque de instalación de arriba son los que tu lab corre de verdad.

Una cosa que v2 conservó de V1, y merece una oración porque es la pieza que la gente olvida hasta que la necesita: el árbol tiene una autoridad propia. `create_tree_v2` escribe una PDA de config del árbol junto a la cuenta Merkle registrando quién creó el árbol, y los mints pasan por un firmante `treeCreatorOrDelegate`. Puedes delegar ese slot a un servicio de minteo sin entregar tu keypair, y puedes crear el árbol `public`, en cuyo caso cualquiera puede agregarle una hoja. Los árboles públicos son cómo funcionan las experiencias de mint abierto y también cómo un desconocido te llena el supply cap, así que el `public: false` por defecto es el default correcto y deberías tener una razón antes de cambiarlo. Hay dos autoridades separadas en juego en esta lección, lo que hace tropezar a la gente: la autoridad del *árbol* firma los mints hacia el árbol, y la autoridad de la *colección* firma la membresía en el Almanac. Tu billetera del lab resulta ser las dos. En producción normalmente no son la misma clave, y la instrucción de mint quiere las dos firmas.

La fila de la colección es la que se estira hacia atrás hasta el trabajo de la lección pasada. `MetadataArgsV2` lleva `collection` como un `Option<PublicKey>` pelado, con el propio comentario de la librería cliente diciendo que en V2 "es solo una `Pubkey` y siempre se considera verificada." Sin paso de verify, sin limbo de no verificado. La cuenta de colección Almanac que creaste en m06-l2 es el valor literal que pasas, y el mint falla si la autoridad de la colección no firma. Primero la colección, después los miembros, un nivel más abajo. La misma regla que aprendiste con los activos Core.

![Una línea de tiempo de cuatro paradas desde Bubblegum V1 en 2023 hasta Bubblegum v2 en 2026, donde la afirmación de que los cNFTs no pueden ser soulbound queda finalmente tachada.](assets/v07-timeline.png)

### El trade-off, nombrado

La compresión cambia mints baratos por complejidad de lectura y acoplamiento de escritura. Las dos mitades son propiedades permanentes del diseño, no asperezas que alguien vaya a limar.

**Las lecturas necesitan un RPC que soporte DAS.** No hay cuenta que buscar, así que `getAccountInfo` sobre un asset id de cNFT no devuelve nada en absoluto. Tu crate es real, es demostrablemente tuyo, y un endpoint RPC público por defecto no puede decirte ni una sola cosa sobre él. Ahora dependes del índice de un proveedor, con la frescura y la completitud que ese proveedor ofrezca, para el acto básico de leer tu propio activo. Esa es una dependencia real y deberías sentirla en el lab cuando tu primera llamada a `getAsset` vuelva vacía por cuatro segundos porque el indexador todavía no se puso al día.

**Las escrituras necesitan una prueba fresca.** Cada transferencia, quema, congelamiento y actualización de metadatos carga nodos de prueba, lo que quiere decir que cada escritura está acoplada al estado actual del árbol y compite contra toda otra escritura sobre el mismo árbol.

**Y el rent no es toda la factura.** Lo que sea que el árbol te haya costado, compró el árbol. No compra el millón de transacciones que lo llenan. Cada `mint_v2` es una transacción con su propia comisión base y, en cualquier día que valga la pena para un drop, su propia comisión de prioridad, y ninguna cantidad de dimensionamiento astuto del árbol hace que eso desaparezca. Agrupar varios mints en una transacción ayuda mucho y es la jugada estándar para un drop real, pero el techo de cuántos caben lo fija el tamaño de transacción, que lo fija el largo de prueba, que lo fija tu canopy. La decisión del canopy llega más lejos de lo que sugiere la línea del rent. Aterrizar un lote grande de forma confiable es una disciplina de lado cliente por derecho propio, y el curso planificado Client-Side Mastery se encarga de ese territorio: comisiones de prioridad, reintentos, y cómo evitar que un lote tenga éxito a medias en silencio.

**Y las decisiones de dimensionamiento son de ida sin vuelta.** Subdimensiona `max_depth` y tu supply cap es permanente. Subdimensiona el canopy y cada transacción de cliente carga pruebas más largas para siempre, lo que puede empujar una escritura más allá del límite de tamaño de transacción exactamente en el momento en que tienes la mayor cantidad de escrituras.

¿Vale la pena? Para un millón de crates que ningún jugador va a cambiar individualmente jamás, obviamente. Para cuatro volúmenes del Almanac con regalías y números de edición, obviamente no, que es por lo que la lección pasada usó cuentas Core y esta no. La regla honesta es aburrida: comprime cuando el conteo de activos es grande y la tasa de escritura por activo es baja. Ninguna mitad sola alcanza.

## Lab: planta el árbol de crates

Siete pasos. Los pasos 1 al 4 están trabajados por completo, el paso 5 te hace elegir y defender la especificación del árbol antes de que nada gaste, y el criterio es el propio script de mint corriendo limpio de punta a punta (el Checkpoint lo reformula). El árbol que construimos aquí deliberadamente no es de un millón de hojas; es de 16,384, que es el tamaño que el proveedor cotiza como 0.34 SOL (a la tasa pre-SIMD-0437 bajo la que se escribió su página) y que tu propia derivación va a confirmar, en el dinero de tu propio cluster, antes de que gastes nada.

Una cosa cambia en tu setup, y cambia por una razón con la que deberías sentarte. **Esta lección corre contra devnet, a través de un RPC que soporte DAS.** El surfnet que has estado usando desde m02-l1 es un validador real, va a crear tu árbol y acuñar tus crates con toda alegría, y después va a ser completamente incapaz de decirte qué acuñaste, porque un validador local no corre un indexador. Ese es el trade-off de complejidad de lectura, y te lo encuentras el día uno. Consigue un endpoint de devnet en un proveedor de DAS (Helius, QuickNode, Triton y Shyft sirven todos la interfaz; el listado de proveedores y cómo elegir son asunto de la próxima lección), luego:

```bash
export DAS_RPC_URL="https://devnet.helius-rpc.com/?api-key=YOUR_KEY"
```

1. **El setup compartido.** Un helper es dueño de la conexión, la billetera y la libreta de direcciones. Fíjate en los tres plugins de Umi: `mplCore()` para la colección, `mplBubblegum()` para el árbol, y `dasApi()` para las lecturas. Sin ese tercero, `umi.rpc.getAsset` no existe. (Una nota de honestidad sobre el sistema de tipos: con estos pins exactos la augmentación de das-api no llega a `umi.rpc` bajo una pasada estricta de resolución de módulos con `tsc`, así que tu editor puede subrayarte `getAsset` en rojo; las corridas documentadas del lab con `npx tsx` no se ven afectadas, y el subrayado es el hueco del stack fijado, no de tu código.)

    ```typescript
    // overgrowth/umi.ts
    import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
    import { keypairIdentity, sol, type Umi } from "@metaplex-foundation/umi";
    import { mplCore } from "@metaplex-foundation/mpl-core";
    import { mplBubblegum } from "@metaplex-foundation/mpl-bubblegum";
    import { dasApi } from "@metaplex-foundation/digital-asset-standard-api";
    import fs from "node:fs";

    export async function getUmi(): Promise<Umi> {
      const endpoint = process.env.DAS_RPC_URL;
      if (!endpoint) throw new Error("set DAS_RPC_URL to a devnet DAS endpoint");

      const umi = createUmi(endpoint).use(mplCore()).use(mplBubblegum()).use(dasApi());

      let secret: Uint8Array;
      if (fs.existsSync("wallet.json")) {
        secret = Uint8Array.from(JSON.parse(fs.readFileSync("wallet.json", "utf8")));
      } else {
        const fresh = umi.eddsa.generateKeypair();
        fs.writeFileSync("wallet.json", JSON.stringify(Array.from(fresh.secretKey)));
        secret = fresh.secretKey;
      }
      umi.use(keypairIdentity(umi.eddsa.createKeypairFromSecretKey(secret)));

      const balance = await umi.rpc.getBalance(umi.identity.publicKey);
      if (balance.basisPoints < sol(1).basisPoints) {
        throw new Error(`fund ${umi.identity.publicKey} with devnet SOL, then re-run`);
      }
      return umi;
    }

    export function book(): Record<string, string> {
      return fs.existsSync("crates.json") ? JSON.parse(fs.readFileSync("crates.json", "utf8")) : {};
    }

    export function remember(key: string, value: string): void {
      fs.writeFileSync("crates.json", JSON.stringify({ ...book(), [key]: value }, null, 2));
    }
    ```

    Corre todo desde `overgrowth/` para que `wallet.json` y la libreta compartida de direcciones `crates.json` caigan juntas. Fondea la dirección impresa desde un faucet de devnet en la primera corrida; el script te dice la dirección y se detiene en vez de construir a medias un árbol con un pagador sin fondos.

2. **Pon el Almanac en este cluster.** Tu Almanac de m06-l2 vive en el surfnet, y devnet nunca ha oído de él. Así que el helper o encuentra la colección registrada en `crates.json` o la crea, y esta vez lleva un `PermanentFreezeDelegate` desde el nacimiento, porque `set_non_transferable_v2` exige que la autoridad firmante sea un delegado de congelamiento permanente sobre la colección. Ármala ahora o el challenge es imposible de ganar después.

    ```typescript
    // overgrowth/collection.ts
    import { generateSigner, publicKey, type PublicKey, type Umi } from "@metaplex-foundation/umi";
    import { createCollection, fetchCollection } from "@metaplex-foundation/mpl-core";
    import { book, remember } from "./umi";

    export async function getOrCreateAlmanac(umi: Umi): Promise<PublicKey> {
      const saved = book().collection;
      if (saved) {
        const existing = await fetchCollection(umi, publicKey(saved));
        return existing.publicKey;
      }

      const collection = generateSigner(umi);
      await createCollection(umi, {
        collection,
        name: "Overgrowth Almanac",
        uri: "https://overgrowth.example/almanac/collection.json",
        plugins: [
          {
            type: "PermanentFreezeDelegate",
            frozen: false,
            authority: { type: "Address", address: umi.identity.publicKey },
          },
        ],
      }).sendAndConfirm(umi);

      remember("collection", collection.publicKey);
      return collection.publicKey;
    }
    ```

    `frozen: false` es deliberado. Un congelamiento a nivel de colección con `frozen: true` congelaría a cada miembro en el momento en que se uniera, que no es lo que quiere un crate Harvest. Lo que necesitas es el *slot de delegado ocupado por una autoridad que controlas*, para que un crate específico pueda marcarse después como no transferible mientras el resto del drop sigue siendo negociable.

3. **Escribe la matemática de dimensionamiento y el chequeo del costo del árbol.** Ya tienes `tree-size.ts` de la sección de teoría. Córrelo contra tu objetivo real antes de gastar:

    ```bash
    npx tsx tree-cost.ts 16384 64
    ```

    Eso imprime el barrido de canopy para un árbol de 16,384 hojas. Este sí lo puedes igualar exactamente, a diferencia de la fila del millón de hojas: Metaplex publica profundidad 14 con buffer 64 y canopy 8 a 0.3358 SOL, que es precisamente la especificación que este lab construye. Su página es anterior al recorte de rent, así que iguala por BYTES y no por SOL: canopy 8 son 48,120 bytes, que son 0.3358 SOL a 6,960 lamports/byte y se imprimen como 0.245 SOL en devnet hoy. El barrido también te va a mostrar canopy 0 en 31,800 bytes, dos tercios de la cuenta y catorce nodos de prueba en cada escritura de cliente en vez de seis. La misma forma que la tabla del millón de hojas, dos órdenes de magnitud más abajo.

4. **Crea el árbol y acuña un crate (trabajado; el paso 5 te entrega las decisiones de especificación).** Aquí está el script principal. Léelo entero antes de correrlo.

    ```typescript
    // overgrowth/mint-harvest-crates.ts
    import { generateSigner, publicKey, type PublicKey, type Umi } from "@metaplex-foundation/umi";
    import {
      createTreeV2,
      findLeafAssetIdPda,
      mintV2,
      parseLeafFromMintV2Transaction,
    } from "@metaplex-foundation/mpl-bubblegum";
    import type { DasApiAsset } from "@metaplex-foundation/digital-asset-standard-api";
    import assert from "node:assert/strict";
    import { getOrCreateAlmanac } from "./collection";
    import { depthForSupply, readLamportsPerByte, treeCostSol } from "./tree-size";
    import { book, getUmi, remember } from "./umi";

    const CRATE_SUPPLY = 16_384;
    const MAX_BUFFER_SIZE = 64;
    const CANOPY_DEPTH = 8;

    async function createCrateTree(umi: Umi): Promise<PublicKey> {
      const saved = book().tree;
      if (saved) return publicKey(saved);

      const spec = {
        maxDepth: depthForSupply(CRATE_SUPPLY),
        maxBufferSize: MAX_BUFFER_SIZE,
        canopyDepth: CANOPY_DEPTH,
      };
      const rate = await readLamportsPerByte(process.env.DAS_RPC_URL!);
      console.log(`tree spec ${JSON.stringify(spec)} -> ${treeCostSol(spec, rate).toFixed(4)} SOL of rent at ${rate} lamports/byte`);

      const merkleTree = generateSigner(umi);
      const builder = await createTreeV2(umi, { merkleTree, ...spec });
      await builder.sendAndConfirm(umi);

      remember("tree", merkleTree.publicKey);
      return merkleTree.publicKey;
    }

    export async function mintCrate(
      umi: Umi,
      merkleTree: PublicKey,
      coreCollection: PublicKey,
      name: string,
    ): Promise<PublicKey> {
      const { signature } = await mintV2(umi, {
        merkleTree,
        coreCollection,
        leafOwner: umi.identity.publicKey,
        metadata: {
          name,
          uri: "https://overgrowth.example/crates/harvest.json",
          sellerFeeBasisPoints: 500,
          collection: coreCollection,
          creators: [{ address: umi.identity.publicKey, verified: true, share: 100 }],
        },
      }).sendAndConfirm(umi);

      const leaf = await parseLeafFromMintV2Transaction(umi, signature);
      const [assetId] = findLeafAssetIdPda(umi, { merkleTree, leafIndex: leaf.nonce });
      return assetId;
    }

    async function resolveThroughDas(umi: Umi, assetId: PublicKey): Promise<DasApiAsset> {
      for (let attempt = 0; attempt < 20; attempt += 1) {
        try {
          return await umi.rpc.getAsset(assetId);
        } catch {
          await new Promise((r) => setTimeout(r, 3000));
        }
      }
      throw new Error(`DAS never indexed ${assetId} - is DAS_RPC_URL a DAS provider?`);
    }

    async function main(): Promise<void> {
      const umi = await getUmi();
      const collection = await getOrCreateAlmanac(umi);
      const merkleTree = await createCrateTree(umi);
      console.log(`tree      ${merkleTree}`);
      console.log(`collection ${collection}`);

      const crate = await mintCrate(umi, merkleTree, collection, "Harvest Crate");
      remember("crate", crate);
      console.log(`crate     ${crate}`);

      const asset = await resolveThroughDas(umi, crate);
      assert.equal(asset.compression.compressed, true, "crate is not compressed");
      assert.equal(asset.compression.tree, merkleTree, "crate is in the wrong tree");
      assert.equal(asset.grouping[0]?.group_value, collection, "crate is not in the Almanac");
      console.log(`OK: getAsset resolved ${asset.content.metadata.name} under the Almanac collection`);

      const achievement = book().achievement;
      assert.ok(achievement, 'crates.json is missing "achievement" - the soulbound crate is yours to mint');
      const soulbound = await resolveThroughDas(umi, publicKey(achievement));
      assert.equal(soulbound.compression.compressed, true);
      console.log(`OK: soulbound crate ${achievement} still resolves through DAS`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Tres detalles ahí adentro se ganan su porqué. `createTreeV2` es async y devuelve un builder en vez de una transacción, porque tiene que preguntarle al RPC cuánto rent necesita un árbol de ese tamaño antes de poder componer la creación de la cuenta con la instrucción de config del árbol. `parseLeafFromMintV2Transaction` vuelve a sacar la hoja de los logs de la propia transacción de mint, que es cómo te enteras del índice de hoja sin consultar nada. Y `findLeafAssetIdPda(umi, { merkleTree, leafIndex })` es `PDA(tree, leaf index)` en código: pura derivación, sin llamada de red, el asset id calculado a partir de dos valores que ya tienes.

    El loop de reintentos alrededor de `getAsset` no es relleno defensivo. Un proveedor de DAS indexa desde los logs de transacción, así que hay un hueco real entre "tu mint confirmó" y "tu crate es consultable," normalmente un par de segundos en devnet. Sin el loop tu primera corrida falla y te pasas veinte minutos depurando un árbol que estaba bien.

5. **Elige y defiende la especificación.** Antes de correr nada: la especificación del árbol es la parte que te pertenece. Saca qué `maxDepth` necesita un supply de 16,384 crates y contrástalo con `DEPTHS` en `tree-size.ts`. Después decide `maxBufferSize` y `canopyDepth` por tu cuenta a partir del barrido del paso 3 en vez de aceptar las constantes del principio del archivo, y ten listo decir en voz alta por qué las elegiste. Las constantes mostradas son una respuesta defendible, no la respuesta.

    Luego córrelo:

    ```bash
    npx tsx mint-harvest-crates.ts
    ```

    Deberías ver la línea de especificación del árbol con su cifra en SOL, la dirección del árbol, la dirección de la colección, el asset id del crate, y después, tras una pausa, `OK: getAsset resolved Harvest Crate under the Almanac collection`. Luego se detiene, con `crates.json is missing "achievement"`. Eso es correcto. La barrera te está diciendo qué falta.

6. **Demuestra que el activo no tiene cuenta.** Una línea, y es toda la lección en un solo chequeo. Con el id del crate de `crates.json`:

    ```bash
    solana account $(node -p "require('./crates.json').crate") --url devnet
    ```

    El CLI reporta que la cuenta no existe. Tu crate es real, está en una colección verificada, DAS acaba de describírtelo por completo, y no hay cuenta. Siéntate con eso un segundo, porque la próxima lección está construida exactamente sobre este hueco.

![Un diagrama de flujo de seis pasos que traza un crate Harvest desde la llamada a mintV2 pasando por el hasheo de la hoja, el log del Noop, el parseo del índice de hoja, la derivación offline del asset id, y finalmente el indexado DAS donde getAsset lo resuelve.](assets/v08-flowchart.png)

7. **Lee lo que el índice te dio.** Imprime la respuesta cruda de `getAsset` una vez, solo para ver la forma:

    ```typescript
    // overgrowth/show-crate.ts
    import { publicKey } from "@metaplex-foundation/umi";
    import { book, getUmi } from "./umi";

    async function main(): Promise<void> {
      const umi = await getUmi();
      const asset = await umi.rpc.getAsset(publicKey(book().crate));
      console.log(JSON.stringify(asset, null, 2));
      console.log(`compressed: ${asset.compression.compressed}`);
      console.log(`tree:       ${asset.compression.tree}`);
      console.log(`collection: ${asset.grouping[0]?.group_value}`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Mira tres campos en específico: `compression.compressed` es `true`, `compression.tree` es la dirección de tu árbol, y `grouping[0].group_value` es la colección Almanac. Esos tres son los asserts que hace la barrera, y son el eco del lado DAS de las tres decisiones que tomaste al momento del mint.

    Fíjate en lo que la respuesta *no* contiene: ninguna pista de que esto es barato. El JSON parece un NFT. Las billeteras lo renderizan como un NFT, los marketplaces lo listan como un NFT, y tu cliente de juego lee `content.metadata.name` exactamente como lee el de un volumen del Almanac. La compresión es invisible en la capa de lectura, que es precisamente por qué funciona como decisión de producto y precisamente por qué es fácil subestimar el compromiso operativo que hay debajo. El resto de ese JSON es material de la próxima lección.

## Challenge

Solo. Acuña el crate de logro Founding-Farmer y hazlo soulbound. Nota de namespace antes de que objetes que ya tienes uno de estos: el BADGE Founding-Farmer de m06-l2 es un activo Core en tu surfnet, y este CRATE de logro es una hoja cNFT en el árbol. El mismo honorífico, dos estándares distintos, los dos soulbound a propósito; Overgrowth le entrega a sus fundadores uno de cada uno, y tener el par es un lindo souvenir de las dos arquitecturas.

Escribe `overgrowth/soulbound.ts` exportando exactamente estas dos funciones, porque el script de la barrera y el trabajo posterior las llaman por esta interfaz:

```typescript
// overgrowth/soulbound.ts
import type { PublicKey, Umi } from "@metaplex-foundation/umi";

export async function markSoulbound(
  umi: Umi,
  assetId: PublicKey,
  coreCollection: PublicKey,
): Promise<void> {
  throw new Error(`implement me: ${umi.identity.publicKey} ${assetId} ${coreCollection}`);
}

export async function transferMustFail(
  umi: Umi,
  assetId: PublicKey,
  coreCollection: PublicKey,
): Promise<void> {
  throw new Error(`implement me: ${umi.identity.publicKey} ${assetId} ${coreCollection}`);
}
```

`markSoulbound` llama a `set_non_transferable_v2`. La instrucción quiere una raíz, un hash de datos, un hash de creadores, un nonce, un índice y una prueba, que son seis cosas que no quieres ensamblar a mano. `getAssetWithProof(umi, assetId, { truncateCanopy: true })` devuelve un objeto cuyos campos se esparcen directo al input de la instrucción, y `truncateCanopy` le dice que suelte los nodos de prueba que el árbol ya cachea para que envíes seis en vez de catorce. La `authority` firmante tiene que ser el delegado de congelamiento permanente que armaste sobre la colección en el paso 2 del lab.

`transferMustFail` busca una prueba **fresca** (la escritura de `set_non_transferable_v2` cambió la raíz, así que la que acabas de usar ya está vieja), intenta un `transferV2` hacia un firmante descartable, y hace assert de que el envío se rechaza. Atrapa el rechazo y haz assert de que lo atrapaste. Un test que pasa porque tu transferencia no hizo nada en silencio no es un test.

Luego conecta las dos a `mint-harvest-crates.ts`: acuña un segundo crate llamado `Founding Farmer` con `mintCrate`, haz `remember("achievement"...)` de su asset id, márcalo soulbound, y demuestra que la transferencia falla. Para que la demostración sea honesta, transfiere el crate Harvest común al mismo firmante descartable en la misma corrida y haz assert de que ese sí *tiene éxito*. Si solo falla el crate de logro, el congelamiento es una propiedad de ese crate. Si fallan los dos, tienes una colección mal configurada y estabas a punto de entregarla.

Aceptado cuando: `npx tsx mint-harvest-crates.ts` corre limpio de punta a punta; `getAsset` resuelve los dos crates bajo la colección Almanac; la transferencia del crate de logro se rechaza mientras la transferencia del crate Harvest llega bien en la misma corrida; y tu salida de `tree-cost.ts` para el árbol que realmente creaste coincide con tu propia cifra derivada del dimensionador dentro del redondeo. Si tomaste la especificación por defecto de profundidad 14 / buffer 64 / canopy 8, esa cifra también coincide con el número publicado del proveedor; si ejercitaste la libertad del paso 5 y elegiste una especificación legal distinta, no hay número de proveedor con el cual coincidir, y tu dimensionador ES la fuente, que es el punto de haberlo construido.

## Checkpoint

El criterio: `npx tsx mint-harvest-crates.ts` imprime la dirección de su árbol, la dirección de su colección, dos asset ids, y las dos líneas `OK`. Con eso en verde, `harvest-crates` está completo: un árbol Bubblegum v2 dimensionado por tu propia matemática, un crate Harvest acuñado en la colección Core Almanac verificada, y un crate de logro soulbound que DAS todavía describe con toda alegría y que nadie puede mover.

Tres respuestas que deberías poder dar sin consultar nada. ¿Dónde viven on-chain los metadatos de un crate? No viven; un hash de 32 bytes de ellos se sienta en una hoja, y la versión legible vive en el índice de un proveedor. ¿Qué compra un árbol de SOL de un solo dígito? Como un millón de hojas, unas pocas millonésimas de SOL cada una, y seis séptimos de esa cuenta son canopy y no árbol. ¿Y puede un cNFT ser soulbound? Sí, desde v2, vía `set_non_transferable_v2` sobre una colección que lleva un delegado de congelamiento permanente, sin importar lo que te haya dicho un post de blog de 2024.

Si tu corrida del challenge está en rojo ahora mismo, el arreglo casi siempre es una de dos cosas. Un hashing mismatch quiere decir una prueba vieja: vuelve a pedirla inmediatamente antes de la escritura, en cada escritura, sin excepciones. Un error de autoridad en `set_non_transferable_v2` quiere decir que el firmante no es un delegado de congelamiento permanente sobre la colección Core, lo que quiere decir que el paso 2 creó tu colección sin el plugin y necesitas una nueva.

![Un diagrama de hub con harvest-crates en el centro, alimentado por la colección Core Almanac y alimentando la lección del lector DAS, la lección de la frontera de compresión, y el capstone.](assets/v09-diagram.png)

Ahora tienes SPROUT, activos Core Almanac y cNFTs de crate Harvest esparcidos entre tres formas on-chain distintas, y uno de ellos ni siquiera lo puedes buscar con `getAccountInfo`. Próxima lección, un script lee los tres.
