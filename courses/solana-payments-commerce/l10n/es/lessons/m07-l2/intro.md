# Ponle un paywall a la API de precios de prensado y después construye el bot que la paga

## Resumen

La lección pasada te entregó el flujo de x402 v2 en papel: los tres headers que lo llevan, PAYMENT-REQUIRED de salida, PAYMENT-SIGNATURE de vuelta, PAYMENT-RESPONSE de salida otra vez, el esquema exact-SVM y la respuesta exacta a quién firma en /verify frente a /settle. Se acabó el tiempo de papel. Hoy construyes las dos puntas y cableas los memos en el libro mayor que ya corres.

Esta es la situación que hace que valga la pena construirlo. Wavelength publica una API de precios de prensado: le das un disco y un tamaño de tirada y te cotiza lo que cuesta el prensado en vinilo. Es gratis, y un bot coleccionista la está martillando diez mil veces al día por nada, inflando tu factura de RPC mientras te paga cero. El arreglo no es una prohibición. El arreglo es un precio. Un middleware delante del endpoint, y el mismo bot que ayer se colaba gratis o empieza a pagar hoy, porque las cotizaciones valen más para su operador que los centavos que ahora cuestan, o se va, y de cualquiera de las dos formas el colarse gratis se acaba.

Primero, levanta el workspace. Va al lado del workspace `backoffice` del módulo 4, porque la ruta de import entre los dos es justamente el punto de esta lección:

```bash
mkdir -p ~/wavelength/x402/src && cd ~/wavelength/x402
npm init -y && npm pkg set type="module"
npm install @x402/express@2.23.0 @x402/svm@2.23.0 @x402/core@2.23.0 \
  express@4.21.2 @solana/kit@6.10.0
npm install -D typescript tsx @types/node @types/express
```

Notas de versión, porque esta línea se mueve rápido: 2.23.0 es la línea `@x402/*` estampada — publicada en npm el 2026-08-18, verificada como instalable el 2026-09-05 — y los nombres con scope son los únicos que hay que instalar; vuelve a comprobar el tag antes de fijarlo. `@x402/svm` declara como peer `@solana/kit >=5.1.0`, y este workspace fija kit 6.10.0, la misma línea v6 que tus workspaces de checkout y de ops. Express queda fijado dentro del rango de peers `^4 || ^5` de `@x402/express` — y sí, 4.21.2 es un major de express distinto del 5.x que corren tus workspaces de checkout, back office y capstone, que es la misma regla por workspace que los majors de kit: cada workspace fija dentro de los rangos de peers de sus propias dependencias, y las dos líneas de express nunca se encuentran en un mismo `node_modules`.

Espera tres líneas `npm warn ERESOLVE overriding peer dependency` en esa instalación, y espéralas cada vez: `@x402/svm` depende de `@solana-program/token` 0.9, `@solana-program/token-2022` 0.6 y `@solana-program/compute-budget` 0.11, y las tres todavía declaran un peer `@solana/kit ^5.0`. npm las resuelve contra tu kit raíz 6.10.0 y avisa en vez de fallar (`npm ls @solana/kit` las va a imprimir como `invalid: "^5.0"`). Advertencias, no errores: la instalación se completa y el lab corre. También es una foto justa de lo joven que es este stack, y una razón para releer esas advertencias en cada bump en vez de entrenar el ojo para saltárselas.

Con lo que te vas, mientras corre esa instalación:

- **La punta del servidor**: el endpoint de precios de prensado de Wavelength detrás de `@x402/express`, apuntado al facilitador de devnet, con un id de factura `extra.memo` por llamada en cada desafío.
- **La punta del cliente**: un agente que paga sobre `@x402/svm`, que se come el 402, firma parcialmente, reintenta con el header de pago y se queda con un comprobante.
- **La conciliación como acumulación**: los ids de factura del memo aterrizan en el mismo libro mayor de backoffice que construiste en el módulo 4. Una venta de máquina y una venta humana fluyen por un solo camino de código.
- **Dos guardas**: el tope de spendControls del cliente (y su trampa de rechazo silencioso), y la historia de por qué el verificador de x402 trae una allowlist hardcodeada para un programa de guarda de billetera.

La división del trabajo, en voz alta: la plomería del servidor y el hook de conciliación los camino contigo de punta a punta. La hoja de precios del middleware y el loop de pagar-y-reintentar del agente te toca llenarlos contra reglas declaradas, modo completion con las llamadas nombradas. La guarda pre-vuelo `decidePayment` del Challenge va en solitario, sin guía.

## Medir el consumo de una llamada desde los dos lados

### La hoja de precios que el middleware hace cumplir

Arranca en el servidor, porque el servidor es donde vive la decisión del dinero. `@x402/express` expone `paymentMiddleware(routes, resourceServer)`: un objeto de rutas que dice qué cuesta cuánto, y un servidor de recursos que sabe cómo verificar y liquidar. Esta es la forma de las rutas, la de verdad, tomada de los tipos de 2.23.0 y recorrida campo por campo:

```ts
// The shape of one protected route (RoutesConfig entry, @x402/core 2.23.0)
const example = {
  'GET /price': {
    accepts: {
      scheme: 'exact',                                   // per-call, precise amount
      network: 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1', // CAIP-2 id: devnet
      payTo: 'YOUR_MERCHANT_ADDRESS',                    // the OWNER address; the scheme derives the ATA
      price: '$0.05',                                    // money form; resolved to devnet USDC
      extra: { memo: 'WVL-INV-0001' },                   // the invoice id, 256-byte ceiling
    },
    description: 'Wavelength pressing-price quote',
  },
};
```

Cada campo es una decisión para la que ya tienes el vocabulario. `scheme: 'exact'` es el esquema de API con medición de consumo de la lección pasada, una llamada, una liquidación. `network` es el id CAIP-2 de devnet; el id de mainnet `solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp` está a un cambio de string de distancia, y todo lo demás de esta lección sobrevive ese cambio menos el facilitador, al que ya vamos a llegar. `payTo` toma la dirección del dueño del comercio, no una cuenta de token: el esquema de SVM deriva él mismo la cuenta de token asociada, la misma distinción dueño-contra-ATA que hace cumplir tu verificador del módulo 4. `price` en la forma de dinero `'$0.05'` lo parsea el esquema contra su tabla de activos incorporada, que en devnet resuelve al mint de USDC de devnet `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`, seis decimales, el mismo mint que tu checkout usa desde el módulo 2. Puedes pasar un par `{ asset, amount }` explícito en su lugar cuando le pones precio en un token específico; la forma de dinero es el valor por defecto porque una hoja de precios en dólares es lo que un comercio de verdad mantiene.

Y `extra.memo` es el campo alrededor del cual orbita esta lección: un string de a lo sumo 256 bytes que el agente que paga tiene que incrustar en la transacción de pago como instrucción de memo, verificado byte por byte por el facilitador. Es tu id de factura, viajando sobre el pago mismo. Fíjate en que el techo se mide en bytes UTF-8, no en caracteres; un id de factura con caracteres multibyte gasta el presupuesto más rápido de lo que su largo sugiere, y por eso el Challenge te hace medirlo como se debe.

![Mapeo campo por campo desde la config de rutas del comercio hasta las PaymentRequirements que el agente decodifica del header PAYMENT-REQUIRED, con el activo en unidades base, un maxTimeoutSeconds al que el SDK le pone 300 por defecto, un fee payer que aporta el facilitador, y un pie que marca maxAmountRequired y amount como frontera de v1 a v2.](assets/v01-comparison.webp)

Una cosa para absorber antes de que te cueste una tarde, y absórbela como una frontera de versión y no como una deriva: el campo del monto se llama `maxAmountRequired` en v1 y `amount` en v2. Eso no es la spec en desacuerdo con el SDK. Abre `@x402/core` 2.23.0 y los dos esquemas están sentados en el mismo build, `PaymentRequirementsV1Schema` con `maxAmountRequired` y `PaymentRequirementsV2Schema` con `amount`, porque el paquete habla los dos dialectos a propósito. Así que el nombre del campo es en sí mismo una señal de versión: si estás mirando `maxAmountRequired`, estás mirando términos de v1, y todo lo demás sobre ese desafío, incluido el hecho de que llegó en un cuerpo de respuesta y no en un header, se desprende de eso.

¿De dónde sale el fee payer? De ti no. El servidor de recursos se sincroniza con el facilitador al arrancar y aprende, por esquema y por red, la dirección del patrocinador con la que el facilitador va a firmar. Por eso la secuencia de arranque tiene un paso `initialize()` explícito, y por eso tu servidor de comercio nunca tiene una llave de fee payer:

```ts
// x402/src/gateway.ts - one facilitator client, one resource server, boot-time work
import { x402ResourceServer } from '@x402/express';
import { HTTPFacilitatorClient } from '@x402/core/server';
import { registerExactSvmScheme } from '@x402/svm/exact/server';

const FACILITATOR_URL = 'https://x402.org/facilitator'; // devnet/testnet ONLY

const facilitator = new HTTPFacilitatorClient({ url: FACILITATOR_URL });
export const resourceServer = new x402ResourceServer(facilitator);
registerExactSvmScheme(resourceServer);
// the server calls resourceServer.initialize() once at boot
```

Esa URL merece su propia oración en negrita en tus notas de despliegue. El facilitador de x402.org es el despliegue de referencia y sirve solo devnet y testnet. Es perfecto para este lab y para CI, y es una trampa si sobrevive hasta una config de producción, porque no te va a liquidar un pago de mainnet, nunca.

Este es también el momento en que se cobra el Challenge de la lección pasada. Redactaste un registro de decisión de facilitador de cinco líneas: una restricción dominante, una elección primaria, una alternativa de compliance, un ajuste de CI y una aceptación de confianza nombrada. Ábrelo. La línea de CI de ese registro es lo que `FACILITATOR_URL` implementa hoy, y si tu registro dice cualquier otra cosa que no sea el facilitador de x402.org para CI, este lab es tu oportunidad de discutir con tu propio yo pasado. La línea de la elección primaria, Corbits, Dexter, PayAI o Solvador, o el CDP de Coinbase si la liquidación con screening es tu restricción, es el cambio de un solo string que haces al salir al aire. Salir al aire es cambiar una URL más la decisión de confianza que esa URL representa, y la mitad de confianza es la mitad difícil, y por eso lo escribiste antes de tener código al que apegarte. Una cláusula más le corresponde a esa decisión: la precaución de versión de la lección pasada sigue aplicando en el cambio, así que antes de salir al aire, confirma con el facilitador que elegiste que hoy liquida v2 en mainnet, o que negocia v1 por ti, porque el mundo desplegado sigue parado a horcajadas sobre las dos versiones.

![Diagrama de secuencia de una llamada pagada, desde el 402 que lleva un memo de factura, pasando por la liquidación del facilitador, hasta la escritura en el libro mayor que precede al comprobante 200.](assets/v02-flowchart.webp)

### Un memo por llamada, o la conciliación colapsa

Ahora la parte que parece un detalle y en realidad es el problema de diseño de la lección. El objeto de rutas de arriba es estático: constrúyelo una vez y cada 402 lleva el mismo memo. Entrega eso y tu conciliación llega muerta, porque tres llamadas pagadas aterrizan en el libro mayor como tres pagos contra un solo id de factura, y tu fila de despacho reporta una venta. Reusar un memo entre llamadas colapsa la conciliación. El memo es un id de factura solo si es único por llamada.

Entonces, ¿de dónde puede salir un id único por llamada? Piensa en lo que el middleware ve de verdad. El desafío 402 y el reintento pagado son dos peticiones HTTP separadas, posiblemente con segundos de diferencia. Cuando llega el reintento, el middleware reconstruye los términos de pago desde la config de la ruta y comprueba que los términos contra los que pagó el agente coincidan con los términos que cotizaría ahora mismo; solo los campos que el esquema declara dinámicos, las pistas de blockhash, quedan exentos de esa comparación. Un memo acuñado al azar en el servidor en el momento del desafío falla esta prueba: el reintento acuñaría uno distinto, la comparación fallaría, y el agente enfrentaría un loop infinito de 402s mientras de su billetera no se drena nada. El id de factura tiene que ser algo que las dos peticiones compartan. Y la única cosa que las dos peticiones comparten, byte por byte, es la URL.

Ese es el patrón: el agente acuña el id de factura y lo lleva en el query string, y el servidor lo pliega en la config de la ruta de esa petición. Determinista en los dos tramos, único por llamada porque el agente lo hace así:

```ts
// x402/src/routes.ts - the price sheet, one invoice id at a time
import type { RoutesConfig } from '@x402/core/server';

const SOLANA_DEVNET = 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1';
const MERCHANT = process.env.MERCHANT_ADDRESS ?? '';

// The price sheet for ONE invoice id. Rebuilt per request: every call carries
// its own extra.memo, and the 402 challenge and the paid retry agree on it
// because the invoice id rides the query string of both.
// The three TODO(config) marks are your completion rung: reason each value out
// from the rules above before reading the ones printed here, then compare.
export function routesFor(invoiceId: string): RoutesConfig {
  return {
    'GET /price': {
      accepts: {
        scheme: 'exact',
        network: SOLANA_DEVNET,     // TODO(config): the CAIP-2 network id
        payTo: MERCHANT,
        price: '$0.05',             // TODO(config): the per-call price
        extra: { memo: invoiceId }, // TODO(config): the per-call invoice id
      },
      description: 'Wavelength pressing-price quote',
    },
    'GET /price/rush': {
      accepts: {
        scheme: 'exact',
        network: SOLANA_DEVNET,
        payTo: MERCHANT,
        price: '$1.50', // deliberately over the client's default cap; see the guard section
        extra: { memo: invoiceId },
      },
      description: 'Rush quote: a human calls the pressing plant',
    },
  };
}
```

Antes de que objetes que un id de factura acuñado por el comprador tiene que ser un agujero de seguridad, juega el ataque hasta el final. El agente controla el string, así que lo peor que puede hacer es reusar uno, y entonces colapsó sus propios comprobantes, no los tuyos: tu libro mayor registra cada pago liquidado bajo el memo que llevaba, cada uno con su propia firma de transacción, y un comprador que paga tres veces bajo un solo id te pagó tres veces igual. El memo es una llave de correlación, no una autorización. La autorización vive en la verificación que el facilitador hace del monto, el activo, el destinatario y el memo contra los términos que produjo tu servidor. Si tu lógica de despacho llega a tratar un memo como prueba de algo por sí solo, reconstruiste exactamente el bug de confianza en webhooks que el módulo 4 se pasó una lección sacándote a golpes.

Ya viste esta idea de conciliación antes bajo otro nombre. Las reference keys de Solana Pay y los ids de factura del memo de x402 son la misma idea: un marcador por pago que viaja sobre la transacción para que el comercio pueda emparejar dinero con pedidos sin emitir una dirección de depósito única por venta. El módulo 3 estampó la reference key en tus transacciones de checkout; x402 estandariza dónde viaja el marcador para los pagos entre máquinas. Dos rieles, un patrón de conciliación.

![Diagrama que contrasta un memo derivado del query string compartido, que sobrevive al reintento, contra un memo aleatorio acuñado en el servidor y uno estático reusado.](assets/v03-diagram.webp)

### El mismo libro mayor, un cliente nuevo

La conciliación es un solo hook. El servidor de recursos dispara `onAfterSettle` después de que el facilitador confirma la liquidación, y el contexto que le entrega a ese hook lleva todo lo que necesita tu libro mayor del módulo 4: los requirements que se pagaron (incluidos el memo y el monto) y el resultado del settle (incluida la firma de la transacción on-chain). Así la venta de máquina aterriza exactamente donde aterriza la venta humana:

```ts
// x402/src/reconcile.ts - the machine-sale path into the module 4 ledger
import type { x402ResourceServer } from '@x402/express';
import type { Ledger } from '../../backoffice/src/ledger.ts'; // the module 4 artifact, unchanged

export function wireReconciliation(resourceServer: x402ResourceServer, ledger: Ledger): void {
  resourceServer.onAfterSettle(async (ctx) => {
    if (!ctx.result.success) return;
    const memo = ctx.requirements.extra['memo'];
    if (typeof memo !== 'string' || memo.length === 0) return;
    ledger.record(
      {
        orderId: memo,                    // the invoice id IS the order id
        recipient: ctx.requirements.payTo,
        recipientAta: '',                 // derivable from (payTo, mint); record() never reads it
        mint: ctx.requirements.asset,
        amountBaseUnits: BigInt(ctx.requirements.amount),
      },
      ctx.result.transaction,             // the settled signature, same column as every webhook sale
    );
    console.log(`reconciled ${memo} -> ${ctx.result.transaction}`);
  });
}
```

Lee la forma que se está pasando y fíjate en que es el `ExpectedOrder` que congelaste en el módulo 4, armado con el vocabulario de x402: memo pasa a ser `orderId`, `payTo` pasa a ser `recipient`, y el mint del activo y el monto en unidades base mapean directo. `recipientAta` queda vacío aquí porque el comprobante del settle no lo lleva y `record()` nunca lo lee; es derivable a partir del dueño más el mint cuando el herramental de ops lo quiera, exactamente como lo anotó la lección del módulo 4 cuando agregó el campo. Forzar el mapeo a través de la misma llamada a `record()` paga el mes que viene, cuando concilies una semana de ventas: el checkout con QR del módulo 3, la transferencia confirmada por webhook del módulo 4 y el bot que pagó por x402 esta mañana son filas en un solo archivo con una sola forma, y cada herramienta de auditoría que escribas alguna vez funciona en las tres.

Voy a confesar dónde se equivocó mi propia primera versión de este hook: registré desde `ctx.paymentPayload`, lo que mandó el cliente, en vez de `ctx.requirements`, lo que exigió el servidor y verificó el facilitador. Los mismos datos en el camino feliz, la dirección de confianza equivocada. El hábito del módulo 4 se transfiere textual: el despacho registra lo que se verificó, nunca lo que se afirmó.

![Dos caminos de venta, una reference key de checkout humano y un id de factura de memo x402 de máquina, convergiendo en una sola forma de fila del libro mayor de backoffice.](assets/v04-diagram.webp)

### El agente, su loop y su límite

Cruza al otro lado del mostrador. El agente que paga necesita tres cosas: una identidad que pueda firmar, un loop que responda 402s y un límite que le impida pagar cualquier cosa que le pongan enfrente.

La identidad primero, y fíjate en que a propósito no es el manejo de llaves del módulo 2. Ahí cargaste el archivo `solana-keygen` que ya tenía el comercio, sus 64 bytes enteros, con `createKeyPairSignerFromBytes`. El agente es el comprador, no el comercio, así que necesita una identidad propia, y no tiene ningún archivo de keygen que cargar. El constructor hermano de Kit cubre ese caso: `createKeyPairSignerFromPrivateKeyBytes` toma una semilla de llave privada de 32 bytes en vez de un archivo de keypair de 64 bytes. Genera la semilla una vez, persístela, y recárgala en una llave de Web Crypto no extraíble en cada corrida. El mismo tipo de firmante al final, envuelto para x402:

```ts
// x402/src/signer.ts - a persistent agent identity
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { webcrypto } from 'node:crypto';
import { createKeyPairSignerFromPrivateKeyBytes } from '@solana/kit';
import { toClientSvmSigner, type ClientSvmSigner } from '@x402/svm';

const SEED_FILE = 'agent-seed.bin';

export async function loadSigner(): Promise<ClientSvmSigner> {
  if (!existsSync(SEED_FILE)) {
    const seed = new Uint8Array(32);
    webcrypto.getRandomValues(seed);
    writeFileSync(SEED_FILE, seed);
  }
  const seed = new Uint8Array(readFileSync(SEED_FILE));
  return toClientSvmSigner(await createKeyPairSignerFromPrivateKeyBytes(seed));
}
```

Fíjate en lo que este agente no necesita: SOL. El facilitador es el fee payer, así que la billetera del agente tiene USDC de devnet y nada más. Esa asimetría es el diseño de exact-SVM haciendo su trabajo, los clientes máquina no manejan gas.

El objeto cliente registra el esquema de SVM contra el id de red comodín, y el límite vive en ese cliente. No aparece ninguna llamada a `setSpendControls` en el código del agente, lo cual es una decisión, no una omisión: los valores por defecto están en vigor, y los valores por defecto tienen dientes. Decláralos con precisión, porque son el hecho congelado sobre el que gira esta sección: de fábrica, los spendControls del cliente reconocen solo activos anclados al dólar de la tabla por defecto del esquema, con un tope de un dólar estadounidense por pago, a menos que los sobrescribas. El USDC de devnet a cinco centavos pasa de largo. La cotización rush a un dólar cincuenta no, y la forma en que no pasa es la trampa: la comprobación corre dentro de la creación del pago, antes de que se firme nada, y lanza. Si el código de tu agente no atrapa y loguea ese throw, el agente simplemente nunca paga, tu fila de despacho se queda vacía, y nada en ningún lado dice por qué. Un tope que no puedes ver rechazando es indistinguible de una integración rota. Cuando una llamada vale legítimamente más de un dólar, sube el tope deliberadamente con `setSpendControls({ maxAmountPerPayment: '$2' })` y escribe por qué; cuando no, quédate con el valor por defecto y haz que el rechazo sea ruidoso.

El loop en sí es tu peldaño de completion, el TODO en el medio del archivo del agente. `wrapFetchWithPayment(fetch, client)` de `@x402/fetch` hace el baile entero en una línea, y lo vas a usar en producción (a propósito no está en la línea de instalación de este workspace; agrega el paquete cuando lo necesites). Hoy escribes el baile a mano una vez, porque el ingeniero que construyó el loop puede debuguear el wrapper, y el ingeniero que solo usó el wrapper no:

```ts
// x402/src/agent.ts - the collector bot, taught to pay
import { x402Client } from '@x402/core/client';
import {
  decodePaymentRequiredHeader,
  decodePaymentResponseHeader,
  encodePaymentSignatureHeader,
} from '@x402/core/http';
import type { PaymentRequired } from '@x402/core/types';
import { ExactSvmScheme } from '@x402/svm/exact/client';
import { loadSigner } from './signer.ts';

const API = process.env.API_URL ?? 'http://localhost:4021';

const signer = await loadSigner();
console.log(`agent pays as ${signer.address}`);

const client = new x402Client().register('solana:*', new ExactSvmScheme(signer));
// No setSpendControls call: the DEFAULTS are in force. Pegged assets, $1 per payment.

// Your turn (completion rung). The four rules:
// 1. fetch(url); anything but HTTP 402 returns as-is, already paid or free.
// 2. Read the challenge out of the HEADER, never the body:
//    decodePaymentRequiredHeader(res.headers.get('PAYMENT-REQUIRED'))
//    returns the PaymentRequired object. The 402's body is the two bytes '{}'.
// 3. const payload = await client.createPaymentPayload(paymentRequired);
//    spendControls run INSIDE this call: an over-cap quote throws here,
//    before anything is signed. Let the throw escape to the caller.
// 4. Retry the SAME url with the header:
//    { 'PAYMENT-SIGNATURE': encodePaymentSignatureHeader(payload) }
async function payAndRetry(url: string, client: x402Client): Promise<Response> {
  throw new Error('Your turn: implement the four rules above.');
}

// The driver is worked: three paid calls, then one deliberate refusal.
for (let call = 1; call <= 3; call++) {
  const invoiceId = `WVL-INV-${Date.now()}-${call}`;
  const res = await payAndRetry(`${API}/price?run=500&invoice=${invoiceId}`, client);
  if (!res.ok) {
    // Failures are silent in the body and loud in the headers; the checkpoint
    // section at the end of this lesson reads them field by field.
    console.error(`call ${call} failed: HTTP ${res.status}`);
    continue;
  }
  const receiptHeader = res.headers.get('PAYMENT-RESPONSE');
  console.log(`call ${call}: paid ${invoiceId}`, await res.json());
  if (receiptHeader) {
    console.log(`  settled: ${decodePaymentResponseHeader(receiptHeader).transaction}`);
  }
}

// The over-cap call: $1.50 against the $1 default. Decline LOUDLY or not at all.
try {
  await payAndRetry(`${API}/price/rush?run=500&invoice=WVL-INV-RUSH-1`, client);
} catch (error) {
  const reason = error instanceof Error ? error.message : String(error);
  console.log(`rush call declined by spendControls: ${reason}`);
}
```

Sostén el camino de decisión entero en una sola imagen antes del lab, porque la posición de la barrera de spendControls, dentro de la creación del pago y antes de cualquier firma, es el hecho al que la sección de debugging te va a seguir mandando de vuelta.

![Diagrama de flujo del agente manejando un 402, donde la comprobación de spendControls dentro de la creación del pago o bien habilita la llamada para firmarse o bien lanza antes de cualquier firma.](assets/v05-flowchart.webp)

### Verifica lo que de verdad se firmó

Una historia de adentro del SDK antes del lab, porque te va a recalibrar para siempre cómo piensas sobre la verificación: las billeteras rompen la verificación ingenua.

Supón que decidiste comprobar los pagos por segunda vez tú mismo, del lado del servidor. El diseño obvio: reconstruir la transacción que esperas, la transferencia, el memo, el compute budget, y compararla byte por byte con lo que mandó el agente. Riguroso, ¿no? Despliégalo, y los pagos legítimos empiezan a fallar. No por maldad: por funciones de seguridad. Phantom y Solflare inyectan instrucciones de guarda de Lighthouse en las transacciones que firman. Lighthouse es un programa de aserciones; las billeteras le agregan instrucciones para que, si los efectos de la transacción divergen de lo que se simuló para el usuario, la ejecución falle on-chain. Un airbag de protección, y quiere decir que la transacción firmada es un superconjunto de la transacción que construiste.

El verificador de SVM de x402 lleva el tejido cicatricial en su código fuente: `mechanisms/svm/src/constants.ts` hardcodea una allowlist para el programa de guarda de Lighthouse, `L2TExMFKdjpN9kozasaurPirfHy9P8sbXoAN1qA3S95`, agregada después de que el issue #828 documentara exactamente esta falla. El verificador recorre la transacción que de verdad se firmó, comprueba la transferencia, el memo y el aislamiento del fee payer, y tolera instrucciones de ese único programa de guarda de la allowlist mientras rechaza cualquier otra adición.

El principio es más grande que el incidente, así que fíjalo: verifica lo que de verdad se firmó, nunca la transacción idealizada que habrías construido. Tu verificador del módulo 4 ya vive según esta regla sin que tú la nombres, lee la transacción desde la blockchain y comprueba propiedades, dueño, mint, delta, memo, en vez de exigir igualdad byte a byte con una plantilla. Las comprobaciones de propiedades toleran adiciones benignas; las comparaciones byte a byte le declaran la guerra a cada función de seguridad de billetera que se haya entregado alguna vez. Cuando escribas código de verificación en cualquier parte de tu stack, estás eligiendo entre esas dos posturas, y este incidente es el argumento a favor de la primera.

![Comparación entre el emparejamiento ingenuo byte por byte y la verificación basada en propiedades que x402 hace de la transacción firmada, con su allowlist para las instrucciones de guarda de Lighthouse inyectadas por la billetera.](assets/v06-comparison.webp)

El mismo paquete esconde una segunda guarda que vale la pena conocer porque construiste a su prima en el módulo 4. El lado del facilitador mantiene una caché de liquidación: una tabla en memoria de las transacciones que se están liquidando en ese momento, para que una llamada /settle duplicada para el mismo pago sea rechazada como `duplicate_settlement` en vez de competirle al primer envío. Las entradas se desalojan con un temporizador que el paquete ata al tiempo de vida del blockhash; sus docs llaman a esa ventana de 60 a 90 segundos más o menos y desalojan a los 120, cerca del doble del tiempo de vida, con el razonamiento de que, una vez que el blockhash de un pago ya no puede aterrizar, un settle repetido de ese pago ya no puede tener éxito, así que recordarlo no sirve de nada. Si esa oración te dio déjà vu, debería: es la misma aritmética de desalojo que la de tu store de firmas procesadas del módulo 4, que olvida una firma una vez que su transacción no podría confundirse de ninguna manera con una fresca. Tu store resguarda el despacho contra webhooks repetidos; la caché de liquidación resguarda el envío contra settles repetidos. La misma forma, otra puerta. Y el hábito del módulo 4 de derivar el reloj de pared a partir del tiempo de slot actual aplica a los dos: con el tiempo de slot objetivo de 300ms — puesto por etapas por SIMD-0525 mientras se escribe esto, con la epoch 1024 (2026-08-28) fijada para cerrarlo días después — la ventana de 150 bloques corre unos 45 segundos, bastante por debajo del redondo 60 que la gente cita, y los cortes por etapas que le quedan a SIMD-0525 la van a encoger todavía más, que es exactamente por qué el margen de la caché es generoso, y por qué este curso sigue diciendo derívalo, nunca lo memorices.

![Línea de tiempo de un pago, desde la firma parcial pasando por el vencimiento del blockhash hasta el desalojo de la caché de liquidación, puesta al lado del store de firmas procesadas del módulo 4 en el mismo horizonte.](assets/v07-timeline.webp)

### El cobrador de peaje que alquilas

Nombra el canje antes de entregarlo, porque este es estructural — y es el mismo canje que la lección pasada ya nombró: el facilitador es fee payer y frontera de confianza de la liquidación en uno solo, ve cada transacción, puede negarse, y negarse a escala es censura le diga como le diga el contrato de servicio; los alojados corren screening KYT por diseño, la misma moneda dada vuelta hacia el compliance. Nada de esto es un bug que alguien vaya a arreglar; un patrocinador es una contraparte. Lo que esta lección agrega es la disciplina del libro mayor: hiciste este mismo canje con las rampas fiat en el módulo 6 y lo escribiste en un registro de decisión, la columna del facilitador pertenece a la misma tabla, y el facilitador de devnet de x402.org pertenece específicamente a la fila nunca-en-producción de esa tabla.

El segundo límite honesto es económico, y es la versión de comercio entre máquinas de una lección que toda persona de pagos aprende tarde o temprano: los costos de liquidación tienen que caber dentro de la cosa que se vende. exact liquida cada llamada on-chain, así que cada llamada carga costo real de blockchain, la comisión de transferencia que se come el patrocinador más el costo operativo de las idas y vueltas de verify y settle. A cinco centavos la cotización, ese overhead es un error de redondeo y la medición de consumo por llamada es exactamente lo correcto. A mil pings de telemetría de menos de un centavo por minuto, la liquidación por llamada cuesta más que el producto, y ninguna cantidad de entusiasmo de ingeniería cambia la aritmética; ese tráfico quiere el modelo de autorizar-un-techo del esquema upto o la liquidación por lotes, los dos existen precisamente porque exact no se estira hasta ahí. Empareja el esquema con la economía unitaria de la llamada, y desconfía de cualquier plan de medición de consumo cuyo margen dependa de que el riel de liquidación sea gratis. Esta es la mitad optimista, y es la mitad que importa para Wavelength: el bot coleccionista que esta mañana era un puro centro de costos ahora es un cliente con una economía unitaria que funciona, a un nivel de precio que ninguna red de tarjetas podría liquidar con ganancia. Los clientes máquina no son una amenaza para la hoja de precios: son el primer segmento de clientes de la historia que la lee a la perfección y nunca abandona un carrito.

Y el tercer límite con el que convives toda la lección: la línea `@x402/*` está fijada en 2.23.0 aquí (publicada el 2026-08-18), y ya se movió dos veces desde entonces — 2.24.0 el 2026-08-27, 2.25.0 el 2026-09-04, comprobado el 2026-09-07 — que es justamente el punto y no una vergüenza: nada de este ecosistema sugiere que se vaya a quedar quieto. Cada hecho de cable de esta lección se leyó de ese build exacto y no de un documento: el transporte por headers, `amount` en vez de `maxAmountRequired` en el requirement de v2, y el `maxTimeoutSeconds` que el servidor de recursos llena por ti. El estar a horcajadas entre versiones es lo que hace que esa disciplina no sea opcional, porque un solo paquete trae los esquemas de los dos dialectos uno al lado del otro, así que "qué forma estoy sosteniendo" sigue siendo una pregunta viva en cada bump en vez de una cerrada. Vuelve a verificar en cada toque, como lo hizo esta lección, no como lo hace un bookmark.

![Tarjeta de tres columnas del canje de la medición de consumo: la frontera de confianza del facilitador, la economía de la liquidación por llamada, y el pin de paquete que se mueve rápido y hay que volver a verificar.](assets/v08-comparison.webp)

## Lab: ponle la cancela, págala, concíliala

El criterio de esta lección: una llamada sin pagar responde 402, tu agente liquida tres llamadas pagadas en devnet, tres ids de factura distintos se concilian en el libro mayor del módulo 4, y la llamada rush por encima del tope se rechaza con una razón logueada. La plomería está caminada arriba; tú llenas la hoja de precios y el loop.

1. **Fondea al agente.** Guarda `src/signer.ts` y `src/agent.ts` de la sección de teoría, después corre el agente una vez. Acuña su semilla, imprime su dirección, y se cae de inmediato en el loop sin implementar, que es lo correcto: hoy la caída es tu marcador de TODO. Mándale a la dirección impresa unos dólares de USDC de devnet desde el faucet de Circle en faucet.circle.com (elige Solana Devnet), el mismo faucet que usaste en el módulo 2. No hace falta airdrop de SOL: el facilitador paga las comisiones, lo que ahora puedes explicar en vez de solamente disfrutar.

```bash
cd ~/wavelength/x402
npx tsx src/agent.ts   # prints: agent pays as <address>, then throws 'Your turn'; fund that address
```

2. **Arma el servidor.** Tres de sus cuatro archivos vinieron de la sección de teoría: `src/gateway.ts` (cliente del facilitador más servidor de recursos), `src/routes.ts` (tu hoja de precios completada, los tres campos TODO(config)), y `src/reconcile.ts` (el hook del libro mayor). El cuarto es el cableado de Express de abajo, trabajado porque sus dos decisiones son sutiles y ninguna de las dos es la lección:

```ts
// x402/src/server.ts - Wavelength's pressing-price API, gate in front
import express from 'express';
import { paymentMiddleware } from '@x402/express';
import { Ledger } from '../../backoffice/src/ledger.ts';
import { resourceServer } from './gateway.ts';
import { routesFor } from './routes.ts';
import { wireReconciliation } from './reconcile.ts';

const ledger = new Ledger(process.env.LEDGER_FILE ?? 'orders.jsonl');
wireReconciliation(resourceServer, ledger);

const app = express();

app.use((req, res, next) => {
  const invoiceId = typeof req.query.invoice === 'string' ? req.query.invoice : '';
  if (!invoiceId) {
    res.status(400).json({ error: 'invoice query param required, e.g. ?invoice=WVL-INV-0001' });
    return;
  }
  // Fresh middleware per request so the routes carry THIS call's memo.
  // syncFacilitatorOnStart=false: the shared resourceServer synced once at boot.
  void paymentMiddleware(routesFor(invoiceId), resourceServer, undefined, undefined, false)(
    req,
    res,
    next,
  );
});

function quote(runSize: number): { runSize: number; unitPriceUsd: number; totalUsd: number } {
  const unit = runSize >= 500 ? 6.1 : 7.4;
  return { runSize, unitPriceUsd: unit, totalUsd: Math.round(unit * runSize * 100) / 100 };
}

app.get('/price', (req, res) => {
  res.json(quote(Number(req.query.run ?? 100)));
});
app.get('/price/rush', (req, res) => {
  res.json({ ...quote(Number(req.query.run ?? 100)), rush: true });
});

await resourceServer.initialize(); // learn supported kinds + the fee payer from the facilitator
app.listen(4021, () => console.log('pressing-price API on :4021, gate armed'));
```

Las dos decisiones, para que sean tuyas: el middleware se construye por petición puramente para que el objeto de rutas pueda llevar el memo de factura de esta llamada, con la sincronización cara del facilitador hecha una sola vez al arrancar y deshabilitada por petición con ese `false` final. Y los handlers de negocio se quedan completamente ignorantes de los pagos; la función de cotización correría idéntica con el middleware borrado, que es la propiedad que le deja a la próxima lección agregar un segundo protocolo de pago sin tocarla.

3. **Levanta el servidor** con tu dirección de comercio del módulo 2:

```bash
MERCHANT_ADDRESS=$(solana address) LEDGER_FILE=../backoffice/orders.jsonl npx tsx src/server.ts
```

Lo esperado: `pressing-price API on :4021, gate armed`. Si lanza en `initialize()`, el facilitador está inalcanzable o no soporta el par esquema/red; comprueba la URL y tu red antes de sospechar de tu código.

4. **Demuestra la cancela desde una segunda terminal.** Sin header de pago, no hay cotización:

```bash
curl -i "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST"
```

Lo esperado, y esta es la forma que prometió la lección pasada (las líneas Date, ETag y keep-alive quitadas, y el base64 elidido en los puntos suspensivos):

```text
HTTP/1.1 402 Payment Required
X-Powered-By: Express
Content-Type: application/json; charset=utf-8
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJQYXltZW50IHJlcXVpcmVkIiwicmVzb3VyY2Ui...
Cache-Control: no-store
Content-Length: 2

{}
```

La cancela está viva, exactamente como se anunció. Decodifica el header:

```bash
curl -sD - -o /dev/null "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST" \
  | grep -i '^payment-required:' | sed 's/^[^:]*: *//' | tr -d '\r' \
  | base64 -d | node -p "JSON.stringify(JSON.parse(require('fs').readFileSync(0,'utf8')),null,2)"
```

```json
{
  "x402Version": 2,
  "error": "Payment required",
  "resource": {
    "url": "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST",
    "description": "Wavelength pressing-price quote",
    "mimeType": ""
  },
  "accepts": [
    {
      "scheme": "exact",
      "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
      "amount": "50000",
      "asset": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
      "payTo": "<the MERCHANT_ADDRESS your server was started with>",
      "maxTimeoutSeconds": 300,
      "extra": {
        "memo": "WVL-INV-TEST",
        "feePayer": "CKPKJWNdJEqa81x7CkZ14BVPiY6y16Sxs7owznqtWYp5"
      }
    }
  ]
}
```

Léelo contra la tarjeta de mapeo de la sección de teoría: tu `'$0.05'` se volvió `"50000"` unidades base del mint de USDC de devnet, tu memo pasó byte por byte, y `resource.description` es la `description` que escribiste en la ruta. Llegaron dos campos que nunca configuraste. `extra.feePayer` vino de la sincronización con el facilitador. Y `maxTimeoutSeconds` es `300` porque la config de tu ruta lo dejó afuera y `@x402/core` rellena 300 segundos cuando eso pasa; pon `maxTimeoutSeconds: 90` al lado de `price` en el `accepts` de esa ruta y el desafío dice `90` en su lugar. El `mimeType` está vacío por la misma razón al revés: nada declaró uno, así que nada se inventó. Puedes ver la misma respuesta sin correr nada: `curl -s https://x402.org/facilitator/supported` lista los kinds que ese facilitador liquida, y el 2026-08-22 su entrada `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1` anunciaba el esquema `exact` con `extra.feePayer` puesto en `CKPKJWNdJEqa81x7CkZ14BVPiY6y16Sxs7owznqtWYp5` (los patrocinadores rotan, así que empareja la forma, no el string). Cada red de esa lista es una testnet, que es la advertencia de solo-devnet repetida por el facilitador mismo. Encontrar ese fee payer en los términos es tu evidencia de que la secuencia de arranque funcionó; un `feePayer` faltante quiere decir que `initialize()` nunca corrió o que el facilitador no soporta tu par de esquema y red.

5. **Completa `payAndRetry`** en `src/agent.ts` contra las cuatro reglas de la sección de teoría, después corre el agente:

```bash
npx tsx src/agent.ts
```

Salida esperada, formas y no strings exactos: tres líneas `call N: paid WVL-INV-...` cada una seguida por una firma de transacción liquidada, y después `rush call declined by spendControls:` con el tope nombrado en la razón.

Las tres firmas son transacciones de devnet reales, así que dedica un minuto a leer una en cualquier explorador, porque la lección entera está sentada adentro. La instrucción de transferencia mueve `50000` unidades base de USDC de devnet desde la ATA del agente hasta la tuya. La instrucción de memo lleva tu string de factura, la historia de conciliación visible en un libro mayor público. Las instrucciones de compute budget están ahí porque la spec las exige en una transacción de settle. Y el fee payer de la transacción no eres tú ni el agente: es la dirección del patrocinador del facilitador que sale de `extra.feePayer`, la cuenta cuya firma fue la última en agregarse. Dos firmas, dos momentos de firma, uno de los cuales pasó en tu máquina y el otro no. Eso es la firma parcial, ya no en papel.

6. **Audita el libro mayor.** Tres filas nuevas, tres ids de factura distintos, junto a las ventas del módulo 4 que el archivo ya tenga:

```bash
tail -n 3 ../backoffice/orders.jsonl
```

Cada fila: el memo como `orderId`, la firma liquidada, `"50000"` unidades base, el mint de USDC de devnet. Un archivo, una forma, dos clases de cliente. Ese es el criterio de esta lección, comprobado a mano desde tu propia terminal: la cancela viva (una llamada sin pagar devuelve 402), al menos una llamada de devnet liquidada, y su memo conciliado en el libro mayor.

## Challenge: la guarda pre-vuelo decidePayment

Peldaño en solitario, sin guía. El throw de spendControls es la guarda del SDK; un agente de producción quiere su propia decisión pre-vuelo con razones que sus logs puedan agrupar, tomada antes incluso de preguntarle al SDK. Implementa `decidePayment` en el widget de coding-challenge decide-payment, que te entrega el starter y sus tests. El widget lo llama posicionalmente, un argumento por campo: `decidePayment(scheme, network, asset, amount, feePayer, memo, maxUsd, allowedAssetsJson)`. Los primeros seis son los términos de pago que cotizó un 402, desagrupados del objeto de requirements que decodificaste del header PAYMENT-REQUIRED; los últimos dos son los controles propios del agente, y la allowlist llega serializada como un string JSON que mapea mint a decimales, así que `JSON.parse(allowedAssetsJson)` antes de comprobar nada contra ella. Devuelve un objeto de decisión que o bien pasa el fee payer y el memo para el pago o bien rechaza con una razón precisa. El monto llega como `amount` porque estos son términos de v2; una guarda escrita contra una contraparte de v1 estaría leyendo `maxAmountRequired` para el mismo número, que es la frontera de versión de la sección de teoría apareciendo en tu primera línea de código.

Rechaza con una razón distinta para cada uno, y corre las comprobaciones en este orden fijo, para que los mismos términos malos produzcan siempre la misma razón, que es lo que hace que los rechazos sean agrupables:

```text
1. scheme     not "exact"                                    -> unsupported scheme
2. network    not a known Solana CAIP-2 id (mainnet/devnet)  -> unknown network
3. asset      absent from the agent's pegged allowlist       -> asset not allowed
4. memo       over 256 UTF-8 BYTES (not string length)       -> memo too large
5. fee payer  no feePayer named                              -> missing fee payer
6. amount     USD-converted value above the cap              -> exceeds spend cap
```

Seis filas, y un campo del desafío queda a propósito fuera de ellas. Cada requirement que decodificaste lleva `maxTimeoutSeconds`, `300` en los términos que cotiza este lab, y la guarda nunca lo mira. Las seis comprobaciones responden todas una sola pregunta, si voy a pagar estos términos, y cada una es una política sobre la que el agente tiene una opinión. `maxTimeoutSeconds` no es una política: es la fecha límite del comercio para completar el pago, puesta en la ruta del comercio y entregada a ti como un hecho. Ahí no hay nada que el agente apruebe o rechace, y un agente que paga de inmediato, como hace este, no puede perderse una ventana de cinco minutos de todas formas. La versión de esta guarda que sí lo comprobaría le pertenece a un agente que encola 402s y los paga después: compara la ventana contra la latencia de peor caso de tu fila y descarta cualquier cosa que no llegue. Fíjate igual en lo que le cuesta a un agente que encola saltarse esa comprobación, porque no es un rechazo. Los términos simplemente dejan de ser pagables, tu guarda no dice nada de nada, y lo que sea que aprendas de eso lo aprendes del `error` de la respuesta y no de tus propios logs.

La barrera de aceptación, que coincide con el criterio de la lección: una llamada dentro de política devuelve `willPay: true` con el fee payer y el memo pasados; una llamada por encima del tope rechaza con una razón de tope; una llamada con el activo equivocado, un esquema que no es exact y un memo de tamaño excesivo rechazan cada uno con su propia razón; y el id CAIP-2 de devnet se acepta como conocido. Los tests del widget corren todo el lote; verde quiere decir listo.

Si terminas temprano, cabléalo: llama a tu guarda al principio de `payAndRetry` y compara sus veredictos con los throws del SDK a lo largo de las cuatro llamadas del lab. Deberían coincidir en todas, y ahora tienes dos opiniones independientes sobre cada pago que hace tu agente, que es exactamente cuánta paranoia merece un bot que tiene una billetera.

Y un ejercicio que cobra una deuda del módulo 4, que vale diez minutos porque es el único lugar donde este curso puede pagarla honestamente. Tu lección de conciliación dejó `tryMatchByReferenceOrMemo` en `backoffice-refunds/src/sweep.ts` con la mitad del memo abierta, con la promesa de que el tráfico del módulo 7 la iba a necesitar. La necesita — pero solo en el camino infeliz, así que arma uno: corre una llamada pagada del agente con tu hook de liquidación deshabilitado (comenta el `ledger.record` de `onAfterSettle`), para que dinero real aterrice on-chain llevando su id de factura `extra.memo` y tu libro mayor nunca se entere. Eso es un worker caído, reproducido a propósito. Ahora corre el barrido de tesorería contra la ATA de tu comercio. Con la mitad del memo llena encuentra el crédito huérfano, parsea el id de factura del memo, y lo empareja con el pedido abierto que el camino de la reference nunca habría podido encontrar, porque una liquidación de x402 no lleva ninguna reference key. Aceptación: una fila recuperada, y el mismo barrido corrido dos veces la escribe una sola vez. Vuelve a habilitar el hook después.

## Checkpoint: tres filas y una negativa

Dónde suele trabarse esto, en el orden en que te lo encontrarías:

1. **Un loop de 402 que nunca se resuelve** — el agente firma y reintenta para siempre mientras nada se liquida y ningún USDC sale de su billetera. Tu memo no es determinista entre el desafío y el reintento; relee la sección del query string, la URL es el único terreno compartido.
2. **Un agente que termina en silencio con una fila de despacho vacía** — dejaste que el throw de spendControls se escapara sin loguearlo, exactamente la trampa del rechazo silencioso de la que avisó la sección de teoría; atraparla no es opcional en nada que entregues.
3. **Una falla de verify en una llamada que tus spendControls aprobaron felices** — no adivines, y no leas el cuerpo, que es `{}` aquí como lo es en todos lados. Decodifica el header PAYMENT-REQUIRED de la respuesta que falla y lee su campo `error`, las palabras propias del facilitador para lo que salió mal. Una ATA sin fondos aparece ahí como `transaction_simulation_failed`, porque el verificador de exact-SVM agrega la firma del fee payer, simula el resultado y mira fallar la transferencia; ese solo string es la diferencia entre comprobar el balance de USDC de devnet de tu agente en diez segundos y sospechar de tu propio código durante una hora.
4. **Pasada la verificación, muerta en la liquidación** — la razón viaja en `PAYMENT-RESPONSE` como `errorReason`, y el cuerpo sigue siendo `{}`. La guarda comprueba la política, el facilitador comprueba la realidad, y la realidad reporta de vuelta por un header.
5. **Filas aterrizando bajo un solo id de pedido** — reusaste un id de factura, que tu libro mayor va a registrar tan feliz y tu conciliación va a malinterpretar como una sola venta; el colapso está río abajo de la escritura, no en ella.

Ahora cuenta lo que tienes. Un patrón de API de producción donde el paywall es un solo middleware y la lógica de negocio nunca se enteró de que el dinero existe. Un agente que paga por HTTP como los navegadores lo hacen fetch, con un límite que hace cumplir antes de firmar. Un camino de conciliación donde las ventas de máquina y las ventas humanas son un solo libro mayor, una sola forma de fila, una sola historia de auditoría, porque ruteaste el memo de x402 por el mismo `record()` que toma una venta de webhook. Y dos instintos de verificación afilados en incidentes reales: comprueba propiedades de lo que se firmó, nunca igualdad byte a byte con lo que construiste, y olvida el estado de repetición solo cuando la blockchain misma hace imposible la repetición.

El hook hacia adelante ya está sentado en tu código, en los handlers que nunca se enteraron de que el dinero existe. La misma API está a punto de responder un segundo protocolo de pago sin cambiar una línea de lógica de negocio: la próxima lección le pones delante el gate de la CLI de pay, así los bots que hablan x402 y los bots que hablan MPP quedan servidos los dos por una sola puerta. Hoy construiste la puerta. La próxima lección aprende más idiomas.
