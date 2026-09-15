# Anatomía de un pago en USDC: mints, ATAs y el transfer-kit

El módulo 1 te dio el modelo de los rieles del dinero. Mapeaste la autorización, la captura y la liquidación de tarjetas sobre `processed`, `confirmed` y `finalized`, decodificaste una transferencia de USDC en vivo desde el libro mayor público, y escaneaste un código QR de pago que generó tu propia terminal. Lo único que produjiste que sobrevive a la lección es un documento: `commitment-policy.md`, la política de confirmación que escribiste y defendiste. Tenlo abierto al lado; el paso 6 te pide revisar una línea de código contra él, y el módulo 4 es donde se convierte en código que corre.

Una nota sobre la carpeta, porque esta lección empieza una nueva. `wavelength-rails` del módulo 1 era espacio de borrador: un puñado de scripts de sondeo desechables y el archivo de la política. Ya hizo su trabajo. Copia `commitment-policy.md` dentro del workspace `wavelength` que estás a punto de crear, en el nivel superior, y después deja `wavelength-rails` en paz o bórralo; nada posterior en el curso lee de ahí. De aquí en adelante, `wavelength` es el único directorio en el que vive este curso, y cada comando dice si corre desde esa raíz o desde dentro de una carpeta de workspace.

Antes de un solo concepto, corre esto en cualquier terminal con Node instalado:

```bash
node -e "console.log(2.01 * 10 ** 6, Math.trunc(2.01 * 10 ** 6))"
```

Te devuelve:

```
2009999.9999999998 2009999
```

Lee eso dos veces. Un cliente escribió $2.01, multiplicaste por un millón para llegar a la unidad más pequeña de USDC, y JavaScript te entregó un número que queda una unidad corto. Trúncalo, como hace la mitad de los tutoriales de internet, y acabas de pagar de menos. Ningún error lanzado, ninguna advertencia, nada en tus logs. Las dos trampas de las que nadie le avisa a un ingeniero de producto son exactamente lo que estás a punto de dejar atrás: matemática de dinero en punto flotante que manda en silencio el número equivocado de centavos, y el hecho de que la billetera a la que le pagas quizá todavía no tenga una cuenta de USDC, así que el pago falla antes de empezar. Para el final de esta lección las dos trampas están muertas, matadas por un módulo que vas a reusar en cada lección que queda de este curso.

## Resumen

Esta es la primera lección de construcción del curso. El artefacto es `transfer-kit`: un módulo de TypeScript que exporta `toBaseUnits` y `fromBaseUnits` (matemática de dinero con enteros exactos), `resolveAta` (encontrar dónde una billetera guarda un token, offline) y `sendStablecoin` (una transferencia verificada que lleva un memo y una reference key). Vas a enviar un pago real de USDC en devnet, imprimir su firma y su reference key, y después probar que el pago existe ubicándolo solo a partir de la reference con una verificación de humo de verify. Cada peldaño de checkout posterior importa este kit, así que las interfaces que escribas hoy son estructurales.

Cómo se reparte el trabajo hoy: esta lección es un ejemplo resuelto con todo el apoyo, así que escribe conmigo de principio a fin. El desafío de código del final, parsear montos escritos por el cliente, es tu primera pieza de trabajo solo. Agregar un segundo token al kit es el cierre sin guía. La próxima lección te da menos apoyo, a propósito: el apoyo llega con un hueco.

## La forma de un pago con stablecoin

### Un mint es el token; tu billetera nunca lo guarda

Primera definición, justo a tiempo: un **mint** es la cuenta on-chain que ES un token. Una sola cuenta define USDC en Solana: su suministro, sus lugares decimales, quién puede crear más. Cada saldo de USDC en cualquier parte de la blockchain apunta de vuelta a ella. En mainnet esa cuenta es `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`, y declara 6 decimals. USDT es otro mint, `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`. En devnet, donde construimos, Circle publica un mint de USDC de prueba en `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`. Tres direcciones, una forma. Tu kit va a tratar "cuál stablecoin" como un parámetro, y por eso agregar USDT más tarde esta noche te va a tomar cinco minutos.

Aquí está la parte que rompe el modelo mental que trajiste de los rieles de tarjetas. La dirección de billetera de tu cliente no guarda USDC. No puede. Una billetera guarda SOL de forma nativa, y eso es todo lo que guarda. Los saldos de tokens viven en cuentas separadas, una por dueño y mint, y la casa estándar de un saldo se llama **associated token account**, la ATA. (El dueño y el mint son los dos seeds que te van a importar hoy; hay un tercero, el token program que es dueño del mint, y se vuelve estructural la próxima lección. El código de abajo lo pasa explícitamente, así que no te sorprenda ver tres argumentos donde la prosa dice dos.) La billetera de Alice es una dirección; el USDC de Alice vive en una segunda dirección; el USDT de Alice viviría en una tercera. Cuando le pagas a Alice, los tokens se mueven entre token accounts, y su billetera firma por la que ella posee.

![La billetera de Alice y el mint de USDC apuntan cada uno a una tercera cuenta, su associated token account, que se deriva de ese par y guarda el saldo.](assets/v01-diagram.png)

¿Cómo encuentras la cuenta de USDC de Alice si ella nunca te dijo su dirección? La calculas. Una ATA es una dirección derivada de programa: determinista, calculada a partir del dueño, del mint y del token program del mint, propiedad de un programa, sin clave privada, guárdate esa idea para el módulo 5. Por hoy, la consecuencia práctica es todo el punto: dada cualquier billetera y cualquier mint, tu código puede derivar la dirección exacta de la token account sin preguntarle a nadie, sin búsqueda, sin ida y vuelta de RPC, sin mensaje al cliente. `resolveAta` va a ser cuatro líneas.

### La cuenta que tu cliente podría no tener

Ahora la trampa. Derivable no es lo mismo que existente. Si Bob nunca ha tenido USDC, la dirección donde viviría su USDC es calculable pero está vacía: ahí no existe ninguna cuenta. Manda tokens a una dirección vacía y la transferencia falla. En los rieles de tarjetas el banco adquirente garantiza que el destino existe; aquí, nadie lo hace. Cada cliente que llega por primera vez llega sin plataforma de aterrizaje, y tu checkout es lo que se da cuenta.

El arreglo es que quien paga puede crear la cuenta en la misma transacción, y crearla cuesta dinero real: una ATA son 165 bytes y necesita suficientes lamports depositados para quedar exenta de rent. El módulo pasado te prometí la partida de una sola vez que los rieles de tarjetas nunca te mostraron. Aquí está, el alquiler de la terminal de estos rieles: un costo fijo de montar la caja, no un corte de cada venta. El programa associated-token-account incluso trae una versión idempotente de su instrucción de creación, create-if-missing, que convierte "¿existe la cuenta?" de una pregunta que haces en una pregunta que nunca necesitas hacer. Tu kit se la va a poner delante a cada pago sin condiciones: si la cuenta existe es un no-op, si no existe acabas de financiarle a tu cliente su plataforma de aterrizaje.

Pregúntale a la red cuál es ese depósito en vez de confiar en cualquier número que leas, incluido el mío:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getMinimumBalanceForRentExemption","params":[165]}'
```

En devnet, el 2026-09-07, eso respondió `1488440` lamports, alrededor de 0.0015 SOL; la misma llamada contra mainnet respondió `1855569`, alrededor de 0.0019 SOL. Dos cosas que sacar de que esos números sean distintos. Primero, la tasa del rent es un parámetro de runtime del clúster, no una constante del token program, así que una cifra copiada de un tutorial es una cifra del clúster de otra persona en el día de otra persona. Segundo, está cayendo activamente: SIMD-0437 está bajando por pasos la tasa por byte (el mínimo exento de rent es `(128 + bytes) × rate`, y 165 + 128 = 293, que es la razón por la que cada tamaño de cuenta escala por el mismo factor cuando la tasa se mueve), mainnet dio su primer paso el 2026-09-03, devnet ya va un paso adelante, y hay más pasos programados. Material más viejo — y frases más viejas en la propia historia de este curso — cita 2,039,280 lamports para esta cuenta, lo cual era correcto antes de ese primer paso y ahora está alrededor de 37% alto en el clúster contra el que corren tus labs — el mismo movimiento dicho al revés es una reducción de 27%, y los dos números son fáciles de confundir. Corre el curl.

Ese es el trade-off, y lo quiero sobre la mesa antes de construir. Las unidades base enteras son exactas pero implacables: ahora tú eres dueño de la contabilidad de los decimales que el atajo del punto flotante escondía. Y pagarle a un cliente completamente nuevo puede requerir crear y financiar el rent de su ATA primero, un costo real de alrededor de una milésima de SOL y un modo de falla extra en cada comprador que llega por primera vez. Ninguno de los dos costos está escondido ya. Ese es el trato en estos rieles, una y otra vez: la maquinaria está expuesta, y tú eres el que la sostiene.

![Un diagrama de flujo que muestra la derivación, un create idempotente sin condiciones y la transferencia agrupados como una sola transacción atómica, con la comprobación de existencia tachada.](assets/v02-flowchart.png)

### El dinero como enteros, o el bug de $2.01 disecado

El one-liner de la apertura falló porque la blockchain no guarda 2.01. Guarda una cuenta entera de la unidad más pequeña. Con 6 decimals, un USDC son 1,000,000 **unidades base**, así que $2.01 son exactamente 2,010,000 de ellas. Los enteros son la única representación honesta del dinero, y el token program no habla nada más que enteros.

Los números ordinarios de JavaScript son floats de 64 bits, y los floats no pueden representar la mayoría de las fracciones decimales de forma exacta. `2.01` se guarda como algo un pelo por debajo de 2.01, multiplícalo por un millón y te da `2009999.9999999998`, trúncalo y quedas corto una unidad. Podrías parcharlo con `Math.round` y tus pruebas van a pasar, y aquí está por qué ese parche es una bomba de tiempo y no un arreglo: los floats solo son exactos para enteros hasta 2 a la 53ª potencia, que es 9,007,199,254,740,992. Más allá de eso, los doubles físicamente no pueden representar cada entero, así que el error de redondeo deja de ser fraccionario y se vuelve una deriva silenciosa de unidades enteras que ninguna llamada de redondeo puede recuperar. Con 6 decimals ese techo queda alrededor de nueve mil millones de USDC, lo cual suena inalcanzable hasta que recuerdas que las unidades base son también cómo vas a sumar volumen diario, mantener saldos de tesorería y procesar pagos por lote. Y para tokens de 9 decimales, el techo baja a unos nueve millones, bien dentro del saldo de una sola ballena. La regla que sobrevive a toda escala: la matemática de dinero nunca toca un float, ni una vez, ni en el medio. Parsea el string decimal del cliente directamente a un `bigint`.

![Tres caminos que convierten 2.01 a unidades base: el truncamiento en punto flotante cae una unidad corto, el redondeo aguanta hasta dos a la quincuagésima tercera y el parseo exacto de strings se mantiene correcto.](assets/v03-annotated-code.png)

Una asimetría completa el modelo del dinero. La conversión corre en dos direcciones, y solo una de ellas es peligrosa. Ir hacia adentro, del string del cliente a unidades base, es donde los pagos se hacen o se corrompen, así que recibe el trato paranoico. Ir hacia afuera, de unidades base a un string de despliegue, es formateo honesto: `12500000` con 6 decimals se renderiza como `12.5`, hecho con rebanado de strings sobre el mismo principio de cero floats, y si una UI después elige mostrar `12.50` con un cero al final, esa es una decisión de presentación que no toca nada. El kit entrega las dos direcciones como un par, `toBaseUnits` y `fromBaseUnits`, porque un sistema que solo puede codificar dinero y nunca auditarlo de vuelta hacia afuera es medio sistema, y el paso de verify al final del lab se apoya en el viaje hacia afuera para reportar lo que de verdad se movió.

### TransferChecked, el memo y la reference key

Tres piezas más y la teoría está lista. Léelas como respuestas a tres preguntas de producto: ¿cómo evito mandar la cosa equivocada?, ¿cómo etiqueto un pago? y ¿cómo lo vuelvo a encontrar después?

No mandar la cosa equivocada: el token program tiene dos instrucciones de transferencia, y tu kit usa la estricta. El `Transfer` simple toma un origen, un destino y un monto entero, y te cree en todo lo demás. **TransferChecked** toma además la dirección del mint y los decimals, y el programa verifica los dos contra las cuentas on-chain, rechazando la transacción ante cualquier discrepancia. Manda el mint equivocado en tu config y la transacción es rechazada. Deja que los decimals se desvíen entre tu código y la realidad y es rechazada otra vez, en vez de mandar un pago errado por órdenes de magnitud. El `Transfer` simple también está deprecado en el tooling actual, así que la elección se hace sola: una instrucción que lleva su propia verificación de sensatez es exactamente lo que el dinero merece.

![Una tabla comparativa que muestra que TransferChecked verifica el mint y los decimals on-chain y rechaza las discrepancias mientras que el Transfer simple le cree a tu configuración, y que Transfer está deprecado en el tooling actual.](assets/v04-comparison.png)

Etiquetar: un **memo** es una instrucción diminuta del memo program que le pega un string corto a la transacción, de forma permanente y pública. IDs de pedido, referencias de factura, "wavelength-order-0001". Público es la palabra operativa: nunca pongas nombres ni correos de clientes en uno, el mundo entero puede leerlo. Piénsalo como el campo de referencia de una transferencia bancaria, menos la privacidad.

Volver a encontrarlo: esta es la estrella silenciosa de todo el curso. Una **reference key** —una clave de referencia— es una dirección nueva y única, de 32 bytes, que le adjuntas a la instrucción de transferencia como una cuenta extra. No firma nada, no recibe nada, no hace nada. Pero cada cuenta que aparece en una transacción se vuelve buscable, así que una búsqueda de firmas sobre esa dirección devuelve exactamente un pago: el tuyo. Genera una reference única por pago y le has dado a cada checkout un número de seguimiento que la red indexa gratis. Cuando tu cliente dice "ya pagué", no escaneas el libro mayor esperando hacer coincidir montos; buscas la reference. Todo el motor de conciliación del módulo 4 se para sobre este truco, y el código QR que escaneaste el módulo pasado ya llevaba una.

![Una dirección de reference nueva viaja en la transferencia como una cuenta extra inerte, el libro mayor la indexa, y una sola búsqueda sobre esa dirección devuelve exactamente un pago.](assets/v05-diagram.png)

Una pieza de historia, porque este trío exacto es más viejo de lo que parece. Cuando Shopify anunció soporte para Solana Pay el 2023-08-23, con MonkeDAO, Mad Lads y Helius entre los primeros usuarios, la propuesta era eliminar comisiones bancarias, chargebacks y tiempos de retención, y el pago por debajo era precisamente este: una transferencia verificada de stablecoin más una reference key para hacer coincidencias. La vía en vivo hoy pasa por el plugin de MoonPay Commerce, pero el primitivo nunca cambió. Estás a punto de construir el mismo pago push que se le entregó a los comercios de Shopify, lo bastante pequeño para caber en un archivo.

## Lab: construye transfer-kit

Numerado y con las manos en la masa de aquí en adelante. Los pasos 1 y 2 son configuración, escuetos a propósito; las decisiones interesantes reciben su por qué en línea.

### 1. Herramientas, claves y SOL de devnet

Necesitas Node 24 o posterior (`node --version` para comprobar) y la CLI de Solana. Node 24 no es un piso arbitrario: los clientes `@solana-program/*` fijados en el paso siguiente declaran `node >= 24`, y un runtime más viejo se gana una advertencia `EBADENGINE` en cada instalación de aquí en adelante. Instala la CLI con el instalador de Anza:

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Reinicia tu shell, después crea un par de claves y apunta la CLI a devnet:

```bash
solana-keygen new --no-bip39-passphrase
solana config set --url devnet
solana airdrop 2
```

`solana-keygen new` escribe un archivo de par de claves en `~/.config/solana/id.json`; ese archivo es tu identidad de comercio por el resto del curso, y el kit lo va a cargar directamente. Una regla de higiene, dicha una vez y temprano: esta es una clave de desarrollo en texto plano para el dinero de juguete de devnet, así que nunca le mandes fondos reales de mainnet, y cuando este curso toque mainnet más adelante la configuración de firma cambia antes que cualquier otra cosa. El airdrop te da SOL de devnet gratis para comisiones y rent. Si se queja de límites de tasa, espera un minuto y reintenta, o usa el faucet web en faucet.solana.com. Confirma con `solana balance`; cualquier número distinto de cero quiere decir que estás listo.

Una cuenta más antes del código, porque el módulo 4 asume que ya la tienes. Ve a dashboard.helius.dev, regístrate (el tier gratis cubre todo lo que hace este curso) y copia la API key del dashboard. Ponla en tu perfil de shell para que sobreviva a una terminal nueva:

```bash
echo 'export HELIUS_API_KEY=<paste-your-key>' >> ~/.zshrc   # or ~/.bashrc
source ~/.zshrc
echo $HELIUS_API_KEY   # must print your key, not an empty line
```

Hoy no la vas a usar. La lección de webhooks del módulo 4 crea un webhook de devnet con ella, y ese es el único lugar de este curso que necesita uno; ninguna llamada RPC que escribas la usa. Tu RPC va a endpoints públicos en todo momento: devnet para todo lo que mandas, y mainnet en modo de solo lectura en el par de lecciones que inspeccionan cuentas en vivo.

### 2. El workspace, con versiones fijadas

Crea el proyecto en el que vive todo el curso:

```bash
cd ~ && mkdir wavelength && cd wavelength
npm init -y
npm pkg set type="module"
mkdir -p transfer-kit/src
npm init -y --workspace transfer-kit
npm pkg set type="module" --workspace transfer-kit
npm install --workspace transfer-kit @solana/kit@^6.10.0 @solana-program/token@0.14.0 @solana-program/memo@0.11.2
npm install --workspace transfer-kit --save-dev typescript@^5.6.0 tsx@^4.19.0 @types/node
```

Estos pins no son los números más nuevos de npm, y me niego a entregarte un misterio, así que aquí está de dónde sale cada uno. Al 2026-08-22, verificado contra el registro mientras escribía esto: el `@solana/kit` más reciente de npm es la línea 8.x, y el estándar amplio de peers del ecosistema es v7. Fijamos la línea v6, que terminó en 6.10.0, porque `@solana/pay`, la librería que adoptan nuestros peldaños de checkout en el módulo 3, declara una dependencia de peer sobre kit ^6.9, y cada lección posterior importa este kit. `@solana-program/token@0.14.0` es el último release compatible con kit v6 (0.15.0 salta su rango de peers a kit ^7, y instalarlo al lado de kit 6 es un error de npm, no una advertencia), y `@solana-program/memo@0.11.2` es la misma historia para el cliente de memo. Los pins de versión se ponen viejos, así que trata este párrafo como fechado: cuando construyas fuera de este curso, lee los rangos de peers en npm y fija a lo que tu grafo de dependencias de verdad exige. Agrega un `tsconfig.json` en la raíz del workspace:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["transfer-kit/src"]
}
```

Checkpoint: `npm ls --workspace transfer-kit @solana/kit` resuelve a `@solana/kit@6.10.x` y no imprime ningún error de dependencia de peer. Si npm reporta un conflicto `ERESOLVE` en su lugar, uno de los tres pins se desvió; arréglalo aquí, porque cada archivo de abajo está construido contra exactamente estos.

### 3. Matemática de dinero exacta: `amounts.ts`

Primer archivo, y es aquel en el que muere el bug de la apertura. Crea `transfer-kit/src/amounts.ts`:

```typescript
/** Exact decimal-string -> integer base units. No floats anywhere. */
export function toBaseUnits(amount: string, decimals: number): bigint {
  const match = /^(\d+)(?:\.(\d+))?$/.exec(amount.trim());
  if (!match) {
    throw new Error(`unparseable amount: "${amount}"`);
  }
  const [, whole, fraction = ''] = match;
  if (fraction.length > decimals) {
    throw new Error(
      `"${amount}" has ${fraction.length} decimal places; this mint supports ${decimals}`,
    );
  }
  const padded = fraction.padEnd(decimals, '0');
  return BigInt(whole) * 10n ** BigInt(decimals) + BigInt(padded || '0');
}

/** Integer base units -> display string, exact, trailing zeros trimmed. */
export function fromBaseUnits(units: bigint, decimals: number): string {
  const digits = units.toString().padStart(decimals + 1, '0');
  const whole = digits.slice(0, digits.length - decimals);
  const fraction = decimals === 0 ? '' : digits.slice(digits.length - decimals);
  const trimmed = fraction.replace(/0+$/, '');
  return trimmed ? `${whole}.${trimmed}` : whole;
}
```

El diseño en un solo aliento: la entrada del cliente se queda como string hasta el último momento posible, la partimos en el punto decimal, rellenamos la fracción a exactamente `decimals` dígitos, y ensamblamos el resultado con aritmética de `bigint` que no puede redondear. La entrada con exceso de precisión, siete lugares decimales contra los seis de USDC, se rechaza en voz alta en vez de redondearse en silencio, porque un checkout que inventa redondeo de sub-centavo es un checkout que concilia mal para siempre. Fíjate en lo que falta: `parseFloat`, `Number`, cualquier float, en cualquier parte.

Checkpoint, directamente desde la raíz del workspace:

```bash
npx tsx -e "
import { toBaseUnits } from './transfer-kit/src/amounts.ts';
console.log(toBaseUnits('2.01', 6));
console.log(toBaseUnits('0.000001', 6));
console.log(toBaseUnits('90071992547.409934', 6));
"
```

Deberías ver `2010000n`, `1n` y `90071992547409934n`. Ese tercer valor está muy por encima del techo del float y es exacto hasta el último dígito; la vía del float da `90071992547409920` para la misma entrada. (`tsx`, que instalamos en el paso 2, corre TypeScript directamente; es también lo que usan los scripts `pay` y `verify`.)

### 4. Los mints que aceptas: `mints.ts`

```typescript
import { address } from '@solana/kit';

export const USDC_MAINNET = address('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
export const USDT_MAINNET = address('Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB');
export const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');

export const USDC_DECIMALS = 6;
export const USDT_DECIMALS = 6;
```

Escueto a propósito. `address()` valida el string en la construcción y te da un tipo marcado en el que confía el resto de kit, así que un typo en una dirección de mint muere aquí en vez de dentro de una transacción. Este archivo es también la puerta de entrada de tu desafío solo: aceptar una nueva stablecoin empieza con dos constantes.

### 5. Donde vive el dinero del cliente: `ata.ts`

```typescript
import type { Address } from '@solana/kit';
import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';

/** Where `owner` holds `mint`. Pure derivation: no network call, no signer. */
export async function resolveAta(owner: Address, mint: Address): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({
    owner,
    mint,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  return ata;
}
```

Cuatro líneas, como prometí, y el comentario es la lección: esto es derivación, no búsqueda. `findAssociatedTokenPda` calcula la dirección determinista localmente a partir de tres seeds, el dueño, el mint y el token program que es dueño del mint; nunca se consulta la red.

El `async` te va a molestar, y debería, porque en todos los demás lugares de este curso `await` quiere decir una ida y vuelta. Aquí no. Derivar la dirección quiere decir hashear los seeds, y kit va por la API de digest Web Crypto de la plataforma, que es asíncrona por especificación, haga o no el trabajo de forma local. Así que `await findAssociatedTokenPda(...)` es una promesa resuelta por tu propia CPU, no por un nodo RPC, y los dos `await` dentro del `Promise.all` en `send.ts` no te cuestan nada más que un tick de microtask. Que exista o no una cuenta en la dirección derivada es una pregunta aparte, y la vía de envío está a punto de volverla irrelevante.

### 6. El envío: `send.ts`

La pieza central. Crea `transfer-kit/src/send.ts`:

```typescript
import {
  AccountRole,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type KeyPairSigner,
  type Rpc,
  type RpcSubscriptions,
  type Signature,
  type SolanaRpcApi,
  type SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import {
  getCreateAssociatedTokenIdempotentInstruction,
  getTransferCheckedInstruction,
} from '@solana-program/token';
import { getAddMemoInstruction } from '@solana-program/memo';
import { toBaseUnits } from './amounts.js';
import { resolveAta } from './ata.js';

export interface SendStablecoinParams {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  payer: KeyPairSigner;
  /** The recipient's WALLET address. We derive their token account ourselves. */
  recipient: Address;
  mint: Address;
  decimals: number;
  /** Human-typed decimal string, e.g. "12.50". Never a float. */
  amount: string;
  memo: string;
}

export interface StablecoinReceipt {
  signature: Signature;
  reference: Address;
  destinationAta: Address;
  baseUnits: bigint;
}

export async function sendStablecoin(
  params: SendStablecoinParams,
): Promise<StablecoinReceipt> {
  const { rpc, rpcSubscriptions, payer, recipient, mint, decimals, amount, memo } = params;

  const baseUnits = toBaseUnits(amount, decimals);
  const reference = (await generateKeyPairSigner()).address;
  const [sourceAta, destinationAta] = await Promise.all([
    resolveAta(payer.address, mint),
    resolveAta(recipient, mint),
  ]);

  // First-time recipient: this creates and rent-funds their ATA (payer pays,
  // the rent-exempt minimum for 165 bytes). If it already exists, the
  // idempotent variant is a no-op, not an error.
  const createDestination = getCreateAssociatedTokenIdempotentInstruction({
    payer,
    ata: destinationAta,
    owner: recipient,
    mint,
  });

  const transfer = getTransferCheckedInstruction({
    source: sourceAta,
    mint,
    destination: destinationAta,
    authority: payer,
    amount: baseUnits,
    decimals,
  });

  // The reference key rides along as an extra readonly non-signer account:
  // it changes nothing on-chain, but the payment is now findable by this address.
  const transferWithReference = {
    ...transfer,
    accounts: [
      ...transfer.accounts,
      { address: reference, role: AccountRole.READONLY },
    ],
  };

  const memoInstruction = getAddMemoInstruction({ memo });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) =>
      appendTransactionMessageInstructions(
        [createDestination, memoInstruction, transferWithReference],
        m,
      ),
  );

  const transaction = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(transaction);

  // maxRetries: 0n - WE own the retry policy. A retry must resubmit these same
  // signed bytes, never rebuild with a fresh blockhash (that mints a second payment).
  const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  await sendAndConfirm(transaction, { commitment: 'confirmed', maxRetries: 0n });

  return {
    signature: getSignatureFromTransaction(transaction),
    reference,
    destinationAta,
    baseUnits,
  };
}
```

Archivo largo, tres decisiones que merecen tu atención; el resto es el pipeline estándar de construir-firmar-enviar de kit y puede ir escueto.

Decisión uno, el orden de las instrucciones: create-if-missing, después el memo, después la transferencia verificada. La transferencia va **al final** y el memo inmediatamente antes, y eso no es estética: el `validateTransfer` de `@solana/pay` saca la última instrucción y exige que sea la transferencia de token, después saca la anterior y exige que sea el memo. Cada validador que está más abajo en este curso, incluida la barrera de aceptación del módulo 3 y el POS en `m03-l3`, hereda esa regla. Las tres siguen viajando en una sola transacción, así que se mantiene atómica: o Bob recibe una cuenta y el dinero y la etiqueta, o no pasa nada. No hay ningún estado en el que hayas financiado el rent de una cuenta para un pago que falló.

Decisión dos, la colocación de la reference. Generamos un par de claves desechable puramente para cosechar una dirección nueva y única, y después lo agregamos, como no firmante de solo lectura, a la lista de cuentas que lleva la instrucción de transferencia. El programa la ignora; el libro mayor la indexa. Diez caracteres de código, y cada pago que este kit llegue a mandar tiene un número de seguimiento.

Decisión tres, la postura frente a los reintentos, y esta siembra una regla en la que se apoya todo el curso. Necesita una definición primero, porque `setTransactionMessageLifetimeUsingBlockhash` te coló una palabra estructural.

Un **blockhash** es un hash de un bloque reciente, y cada transacción tiene que llevar uno. No es un nonce y no es un timestamp; es un vencimiento. La red recuerda los últimos 150 blockhashes y rechaza cualquier transacción cuyo blockhash se haya caído del final de esa lista, que es lo que impide que una transacción firmada se quede dando vueltas para siempre y aterrice meses después. Así que el blockhash que traes cuando construyes es un tiempo de vida: esta transacción es válida hasta que ese hash caduque, y después está muerta, de forma permanente y segura.

Ahora la regla de reintentos. Pasamos `maxRetries: 0n`, diciéndole al RPC que no reenvíe por su cuenta, porque el reenvío a ciegas le pertenece a quien guarda el comprobante, y ese eres tú. Aquí está la propiedad que hace seguros los reintentos: la firma de una transacción firmada es una función de sus bytes exactos, así que reenviar los mismos bytes firmados siempre produce la misma firma, y la red va a aterrizar esa firma como máximo una vez. Reconstruir el pago con un blockhash nuevo en cambio crea una firma nueva, y si el primer envío de verdad aterrizó mientras se te vencía el tiempo, felicidades, pagaste dos veces. Así que la regla: al vencer el tiempo, reenvía los bytes idénticos, y reconstruye solo después de que el blockhash haya vencido de verdad Y la blockchain confirme que el original nunca aterrizó. Dos condiciones, las dos comprobadas abajo.

"Vencido de verdad" es algo que compruebas, no una duración que adivinas. El RPC responde directamente:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"isBlockhashValid","params":["<YOUR_BLOCKHASH>",{"commitment":"finalized"}]}'
```

Mientras `result.value` sea `true`, tus bytes originales todavía pueden aterrizar, así que no reconstruyas. En el momento en que cambie a `false`, esa transacción en particular está muerta: nunca puede volver a aterrizar. Muerta no es lo mismo que nunca-aterrizada, eso sí, y esta es la mitad de la regla que le cuesta dinero a la gente. El blockhash pudo haber vencido *después* de que tu transacción se liquidara en silencio mientras se te vencía el tiempo. Así que el vencimiento es la primera de dos comprobaciones, no la comprobación completa — antes de reconstruir, pregúntale a la blockchain si el original ya aterrizó:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSignatureStatuses","params":[["<YOUR_SIGNATURE>"],{"searchTransactionHistory":true}]}'
```

Un null en `result.value[0]` con la búsqueda de historial activada quiere decir que nunca aterrizó, y solo entonces es seguro reconstruir. Un objeto de estado quiere decir que te pagaron y que el reintento que estabas a punto de mandar le habría cobrado a tu cliente una segunda vez. Vencido más nunca-aterrizado: los dos, cada vez. La cifra de reloj de pared, alrededor de 150 bloques y por lo tanto unos 45 segundos con el tiempo de slot objetivo actual de 300ms, sirve para dimensionar un timeout pero no es la comprobación; derívala del tiempo de slot en vez de memorizar los segundos, porque el tiempo de slot es el número que se mueve, y se ha estado moviendo rápido: SIMD-0525 ya lo cortó dos veces en una semana, con dos cortes más por etapas ya condicionados en el código. La fila de pagos offline del módulo 8 se salta este plazo por completo — un puesto de mercado no puede volver a comprobar un blockhash al que no puede llegar, así que firma contra un durable nonce en su lugar; hoy el kit simplemente se niega a ser tramposo con el plazo, y si alguna vez necesitas la comprobación, el curl de arriba es eso.

![Dos líneas de tiempo sobre la ventana de 150 bloques: reenviar los mismos bytes mantiene una sola firma que aterriza como máximo una vez, mientras que reconstruir crea una segunda firma y le cobra al cliente dos veces.](assets/v06-timeline.png)

Una pasada más sobre la plomería, porque este es tu primer contacto con el pipeline de kit y se va a repetir en cada archivo que toque la red que escriba este curso. La cadena de `pipe` construye un *message* de transacción: una descripción de lo que debería pasar, de quién paga la comisión y de cuál blockhash ancla su tiempo de vida. Nada en el message es final; puedes seguir transformándolo. `signTransactionMessageWithSigners` es la puerta de un solo sentido: recorre el message, encuentra cada cuenta que tiene que firmar (nuestro payer cubre tanto la comisión como la transferencia de token), firma, y congela los bytes. Después de esa puerta, la transacción ES sus bytes, que es exactamente por qué funciona la regla de reenvío: la firma se calculó sobre ellos, así que los bytes y la firma nunca pueden separarse. `sendAndConfirmTransactionFactory` después empaqueta las dos conexiones RPC que le pasaste, HTTP para mandar y una suscripción por websocket para escuchar de vuelta, y bloquea hasta que la red reporte tu nivel de commitment solicitado. Pedimos `confirmed`. Ahora abre `commitment-policy.md` y compáralo con lo que escribiste bajo `Everyday payments`: un pago de prueba de $1.25 cae de lleno bajo ese encabezado, y si tu archivo decía `finalized` ahí, cambia el string en el código para que empate con tu política en vez de cambiar tu política para que empate con mi código. Este es un momento pequeño y es la única vez que esta lección toca el archivo, pero es el hábito que el módulo 4 automatiza: el nivel de commitment en el código está aguas abajo de una política escrita, nunca un valor por defecto que alguien escribió una vez.

![Un pipeline de tres fases: el message mutable gana un fee payer, un tiempo de vida por blockhash e instrucciones, después la firma congela los bytes, y la transacción inmutable se manda y se confirma.](assets/v07-flowchart.png)

Ata los exports juntos en `transfer-kit/src/index.ts`:

```typescript
export { toBaseUnits, fromBaseUnits } from './amounts.js';
export { resolveAta } from './ata.js';
export { sendStablecoin } from './send.js';
export type { SendStablecoinParams, StablecoinReceipt } from './send.js';
export * from './mints.js';
```

Checkpoint antes de gastar nada: corre `npx tsc --noEmit` desde la raíz de `wavelength`. El silencio es el aprobado. Cinco archivos, ninguna salida, el kit compila.

### 7. Manda un pago real: `pay.ts`

Dos cosas antes del script. Primero, USDC de devnet: Circle corre un faucet en faucet.circle.com, elige Solana Devnet, pega tu dirección de `solana address`, y te manda una pequeña cuota de prueba a la ATA correcta. Segundo, una segunda billetera a la que pagarle: `solana-keygen new --no-bip39-passphrase -o /tmp/customer.json`, y después `solana-keygen pubkey /tmp/customer.json` imprime la dirección de billetera de tu cliente de mentiras. No tiene cuenta de USDC, que es exactamente lo que queremos: tu primer pago ejercita la vía del comprador que llega por primera vez a propósito.

Crea `transfer-kit/src/pay.ts`:

```typescript
import { readFile, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
} from '@solana/kit';
import { sendStablecoin } from './send.js';
import { USDC_DEVNET, USDC_DECIMALS } from './mints.js';

const recipientArg = process.argv[2];
const amountArg = process.argv[3] ?? '1.25';
if (!recipientArg) {
  console.error('usage: npm run --workspace transfer-kit pay -- <recipient-wallet> [amount]');
  process.exit(1);
}

const keyfile = `${homedir()}/.config/solana/id.json`;
const bytes = new Uint8Array(JSON.parse(await readFile(keyfile, 'utf8')));
const payer = await createKeyPairSignerFromBytes(bytes);

const rpc = createSolanaRpc('https://api.devnet.solana.com');
const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

const receipt = await sendStablecoin({
  rpc,
  rpcSubscriptions,
  payer,
  recipient: address(recipientArg),
  mint: USDC_DEVNET,
  decimals: USDC_DECIMALS,
  amount: amountArg,
  memo: `wavelength-order-0001`,
});

console.log('signature :', receipt.signature);
console.log('reference :', receipt.reference);
console.log('base units:', receipt.baseUnits.toString());

await writeFile(
  new URL('../receipt.json', import.meta.url),
  JSON.stringify(
    {
      signature: receipt.signature,
      reference: receipt.reference,
      destinationAta: receipt.destinationAta,
      baseUnits: receipt.baseUnits.toString(),
    },
    null,
    2,
  ),
);
```

Conecta los scripts y córrelo:

```bash
npm pkg set scripts.pay="tsx src/pay.ts" scripts.verify="tsx src/verify.ts" --workspace transfer-kit
npm run --workspace transfer-kit pay -- $(solana-keygen pubkey /tmp/customer.json) 1.25
```

Salida esperada, con tus propios valores:

```
signature : 3Qx...long base58 string
reference : 7pK...a fresh address
base units: 1250000
```

Esa firma es un pago de devnet en vivo que puedes pegar en cualquier explorador, y deberías, porque ya has estado del otro lado de este mismo vidrio antes. En el módulo 1 decodificaste la transferencia de USDC de otra persona; ahora decodifica la tuya. Pon el explorador en devnet, abre la transacción, y lee la anatomía que acabas de escribir: tres instrucciones en orden, el create aterrizando la cuenta de Bob, el string del memo ahí sentado en público exactamente como se advirtió, y la transferencia verificada con una dirección extra colgándole sin hacer nada (saluda a tu reference key). El script también deja un `receipt.json` al lado del código fuente, que es deliberadamente primitivo: hace de sustituto de la tabla orders que va a mantener tu backend real, y el paso de verify está a punto de consumirlo. Aprovecha el momento si quieres; el módulo 1 fue todo preparación para esas tres líneas de salida.

### 8. Demuestra que aterrizó: `verify.ts`

Un pago que no puedes verificar de forma independiente es un pago que no tienes. La verificación de humo hace el papel de tu yo de mañana por la mañana: ignora todo excepto la reference key y el monto esperado, encuentra el pago solo a partir de la reference, y recalcula lo que de verdad llegó desde los propios registros de saldo de la blockchain. Crea `transfer-kit/src/verify.ts`:

```typescript
import { readFile } from 'node:fs/promises';
import { address, createSolanaRpc, signature as asSignature } from '@solana/kit';
import { fromBaseUnits } from './amounts.js';
import { USDC_DECIMALS } from './mints.js';

const rpc = createSolanaRpc('https://api.devnet.solana.com');

const receipt = JSON.parse(
  await readFile(new URL('../receipt.json', import.meta.url), 'utf8'),
) as { signature: string; reference: string; destinationAta: string; baseUnits: string };

// 1. Locate the payment by its reference key, NOT by the signature we stored.
//    This is the whole point: reconciliation must work from the reference alone.
const reference = address(receipt.reference);
const found = await rpc.getSignaturesForAddress(reference, { limit: 10 }).send();
if (found.length === 0) {
  throw new Error(`no transaction found for reference ${receipt.reference}`);
}
const sig = found[0].signature;
if (sig !== receipt.signature) {
  throw new Error(`reference resolves to ${sig}, expected ${receipt.signature}`);
}

// 2. Fetch it and confirm it succeeded.
const tx = await rpc
  .getTransaction(asSignature(sig), { encoding: 'json', maxSupportedTransactionVersion: 1 })
  .send();
if (!tx || !tx.meta) throw new Error('transaction not found on devnet');
if (tx.meta.err) throw new Error(`transaction failed: ${JSON.stringify(tx.meta.err)}`);
const meta = tx.meta;

// 3. Recompute the received amount from token balances, in exact base units.
const keys = tx.transaction.message.accountKeys;
const destinationIndex = keys.findIndex((k) => k === receipt.destinationAta);
const balanceAt = (balances: typeof meta.preTokenBalances): bigint => {
  const entry = balances?.find((b) => b.accountIndex === destinationIndex);
  return entry ? BigInt(entry.uiTokenAmount.amount) : 0n;
};
const delta = balanceAt(meta.postTokenBalances) - balanceAt(meta.preTokenBalances);

if (delta !== BigInt(receipt.baseUnits)) {
  throw new Error(`recipient received ${delta}, expected ${receipt.baseUnits}`);
}

console.log('devnet transfer confirmed');
console.log(`located by reference ${receipt.reference}`);
console.log(`amount: ${delta} base units (${fromBaseUnits(delta, USDC_DECIMALS)} USDC)`);
```

Dos notas de diseño antes de correrlo. La búsqueda pide hasta diez firmas pero todo el esquema solo funciona si alguna vez encuentra una, y eso es una disciplina, no una esperanza: el kit genera una reference nueva por pago y nunca la reusa. Reusa una reference entre pagos y `getSignaturesForAddress` empieza a devolver una lista que tienes que desambiguar, que es conciliación con pasos extra y un criadero de bugs sutiles; un pago, una reference, para siempre. Segundo, la verificación deliberadamente no confía en el monto de tu propio comprobante como verdad de referencia. Extrae el saldo de la cuenta de destino antes y después desde los metadatos de la transacción y toma la diferencia, en strings crudos de unidades base convertidos directamente a `bigint`, con los floats excluidos del rastro de auditoría con la misma minuciosidad que de la vía de pago. Córrelo:

```bash
npm run --workspace transfer-kit verify
```

```
devnet transfer confirmed
located by reference 7pK...your reference
amount: 1250000 base units (1.25 USDC)
```

Ese es el checkpoint que toda la lección pone como barrera. Si la búsqueda no devuelve nada, el sospechoso de siempre es un RPC que no se ha puesto al día; espera unos segundos y vuelve a correrlo. Si la comprobación del monto lanza un error, lee los dos números del error, porque uno de ellos vino de tu código y uno vino de la blockchain, y la blockchain no es la que está equivocada.

![La verificación fluye desde la reference key, pasando por la búsqueda de firmas y la obtención de la transacción, hasta un delta de saldo en unidades base, con salidas de falla por firma ausente, transacción fallida o discrepancia de monto.](assets/v08-flowchart.png)

### 9. Lo que ese pago costó de verdad

Cierra el círculo con el repaso de costos que el módulo 1 te prometió. Tu pago llevó una firma, así que la comisión base fue de 5000 lamports, un cargo fijo, una fracción pequeña de un centavo a cualquier precio del SOL que te den ganas de meter. Y como tu cliente de mentiras nunca había tenido USDC, la transacción también depositó el mínimo exento de rent de 165 bytes que consultaste al principio de esta lección — la partida del alquiler de la terminal, pagada por ti como fee payer. No me creas a mí cuánto salió de tu billetera: `solana balance` antes y después de un pago a una dirección nueva te muestra la comisión y el rent juntos, y restar la respuesta del curl deja la comisión. Manda el mismo pago otra vez al mismo cliente y mira cambiar la anatomía: el create idempotente se vuelve un no-op, sin rent, solo la comisión de 5000 lamports. Los compradores que llegan por primera vez te cuestan una fracción de centavo más la plataforma de aterrizaje; los compradores que vuelven cuestan la fracción sola.

Aléjate un párrafo, porque la forma aquí importa más que los tamaños. El costo recurrente de tomar un pago en estos rieles es plano y diminuto a cualquier tamaño de ticket, y el único costo con significado es un gasto de capital de una sola vez, por cliente, que compra una pieza permanente de infraestructura on-chain para esa relación. Los rieles de tarjetas te cobran un porcentaje para siempre y no te devuelven nada durable. Aquí pagas centavos una vez y cada pago posterior de ese cliente viaja casi gratis. Para un negocio con clientes que vuelven, eso no es un descuento, es un modelo de costos distinto — llévatelo contigo al módulo 6, donde se pesan modelos de costos como este cuando elegimos un riel por corredor.

![Barras apiladas comparan a un cliente que llega por primera vez y paga la comisión base de 5000 lamports más el depósito único de rent de la ATA, alrededor de una milésima de SOL, contra un cliente que vuelve y paga solo esa comisión fija.](assets/v09-chart.png)

## Challenge

Dos partes, en orden creciente de soledad.

**El desafío de código, tu primera repetición en solitario.** `toBaseUnits` maneja un número que produjo tu código. Ahora maneja un número que escribió un humano. El peldaño de abajo pide un segundo export para `amounts.ts`, `parseAmount(input: string, decimals: number)`, que toma la entrada cruda del formulario de checkout y devuelve o las unidades base exactas o la razón por la que se negó: `{ ok: true, baseUnits }` o `{ ok: false, reason }`, nunca un error lanzado. Ese tipo de retorno es toda la decisión de diseño, así que sé deliberado con él. `toBaseUnits` lanza porque el código de kit lo llama con un string que produjo kit, donde un string malo es un bug y detener el programa es la respuesta correcta. `parseAmount` se para en la frontera del cliente, donde la entrada mala es tráfico ordinario, y negarse es un resultado normal y no una emergencia, así que devuelve algo que el checkout puede renderizar al lado del campo en vez de una excepción que la UI tiene que atrapar para seguir viva. Las razones son un conjunto cerrado, `empty`, `malformed`, `negative`, `too-precise` y `too-large`, precisamente para que el formulario pueda mapear cada una a una frase que un cliente entienda. Este es trabajo genuinamente nuevo, no un retecleo del lab: `toBaseUnits` puede asumir un string decimal limpio, y este recibe lo que sea que produjo el teclado de un cliente.

Algunos límites a los que te obliga. `" 12.5 "` parsea a `12500000n` con 6 decimals, porque el espacio en blanco al inicio y al final es un artefacto del pegado, no un error, y `"0012.50"` parsea a lo mismo, ya que los ceros al inicio son feos pero no están mal. `"$12.50"` y `"12abc"` son `malformed`; también lo es `"."`, que no tiene dígitos a ninguno de los dos lados. Un campo sin tocar, o uno que solo tiene espacios, es `empty`, una razón aparte para que el formulario pueda decir "ingresa un monto" en vez de "eso no es un número". `"-5"` es `negative`, porque un checkout nunca cobra por debajo de cero. `"7.071"` en el mint de 2 decimales que estás a punto de crear es `too-precise`, y fíjate que ese mismísimo string está perfectamente bien en USDC: la precisión es una propiedad del mint, no del parser. Y hay un techo igual que hay un piso, porque los montos de token SPL son `u64`: `18446744073709.551615` con 6 decimals es el monto más grande que cualquier transferencia puede llevar, y una unidad base más es `too-large` antes de que siquiera llegue a una instrucción.

El límite que la mayoría de las implementaciones se equivoca es el de exceso de precisión, y vale la pena bajar el ritmo por él. `"12.5000001"` en un mint de 6 decimales es `too-precise`, porque ese séptimo dígito es dinero que el mint no puede guardar y truncarlo inventa redondeo de sub-centavo que concilia mal para siempre. Pero `"1.5000000"` **no** tiene exceso de precisión. También tiene siete dígitos de fracción, y el séptimo es un cero, así que el monto es exactamente 1500000 unidades base y es exactamente representable. Contar los caracteres de la fracción es la comprobación fácil y rechaza un pago que está perfectamente bien; preguntar si alguno de los dígitos extra es distinto de cero es la comprobación que de verdad es correcta. (El `toBaseUnits` del lab sí cuenta caracteres, deliberadamente: lo alimenta tu propio código, así que un dígito extra ahí quiere decir un bug y debería ser ruidoso. La regla indulgente pertenece a la frontera humana y solo ahí.)

Después el caso que el peldaño no puede probar, y que deberías implementar de todas formas: `"12,50"`. Un teclado pt-BR produce esa coma por defecto y tus clientes brasileños la van a escribir todo el día. Es `malformed`, y el parser tiene que negarse en vez de quitar la coma servicialmente para dejar `1250` y cobrar cien veces el ticket. Está ausente de los casos de abajo por una razón de plomería y no pedagógica: el formato de caso que le pasa los argumentos a tu función está él mismo separado por comas, así que una coma dentro de un monto entre comillas no puede sobrevivir el viaje. La regla vale de todas formas. Y una regla más que ningún caso puede comprobar tampoco, que te impones a ti mismo: nada de `parseFloat` ni de `Number` en ninguna parte del parseo, porque una versión que pasa con floats pasa por suerte.

**El cierre solo, sin guía.** Haz real el segundo token del kit. `mints.ts` ya lleva `USDT_MAINNET`, pero Tether no publica ningún mint oficial de devnet, así que sustitúyelo como lo hacen los equipos de integración, y hazlo una prueba de verdad mientras estás ahí: crea tu propio mint de práctica con **2 decimals**, no 6, para que tu kit tenga que respetar de verdad la precisión propia del mint en vez de estar de acuerdo en silencio con la de USDC. Usa la CLI de spl-token (`spl-token --version` para confirmar que la tienes, `cargo install spl-token-cli` si tu instalación no la trajo; después `spl-token create-token --decimals 2 --url devnet`, `spl-token create-account` y `spl-token mint` para acuñarte un saldo), regístralo al lado de las constantes reales, y manda `7.07` a la billetera de tu cliente. La aceptación es la misma barrera que puso la lección, ahora para dos precisiones distintas: montos correctos en unidades base exactas, y cada pago ubicado por su propia reference a través de la verificación de verify. Mira `verify.ts` en particular, porque formatea su salida con un `USDC_DECIMALS` hardcodeado, y un mint de 2 decimales va a hacer que esa línea te mienta. Arreglarlo es el refactor de esta noche.

Cuando las dos partes pasen, has superado la barrera del módulo: dos tokens, montos exactos, cada pago localizable por reference.

Este kit es pequeño, y eso es el logro, no una limitación; todo el checkout de la tienda de discos se va a parar sobre estos cuatro exports. Si la vía de envío te dio pelea, los dos culpables de siempre son un saldo vacío de USDC en devnet (el faucet arregla eso) y un timeout de RPC a mitad de la confirmación (reenvía los mismos bytes, ya sabes por qué). Trae cualquier cosa más extraña a la comunidad del curso, idealmente con tu reference key adjunta, porque ahora hablas en números de seguimiento.

Así que transfer-kit mueve USDC, y tienes comprobantes. La próxima lección entra un cliente con PYUSD en la mano, y PYUSD no es un token clásico: vive en Token-2022 con ocho extensiones colgando del mint, y tu kit como está escrito le apunta el pago al programa completamente equivocado. Vamos a leer ese mint en vivo y a enseñarle al kit algunos modales.
