# Checkout con un código QR: el spec de Solana Pay, en vivo

El módulo pasado construiste transfer-kit: detecta si un mint pertenece al Token clásico o a Token-2022 y puede mover cualquiera de las stablecoins principales con un memo y una reference adjuntos. Esa es una capacidad real, y tiene un hueco que se siente. Nada en él le pide a un cliente que pague. Solo envía. Puedes pagarle a cualquiera desde código; un cliente no puede pasarte dinero corriendo tu TypeScript.

Lo que un cliente sí puede hacer es apuntar un teléfono a un cuadrado de píxeles. Hoy construyes ese cuadrado, y el servidor que se entera cuando alguien lo paga. Al terminar esta lección una venta en devnet aterriza en tu máquina: código QR escaneado, transferencia liquidada, pedido emparejado, monto confirmado.

Empieza la instalación ahora para que termine mientras lees. Este es un workspace nuevo al lado de transfer-kit, no dentro de él, por una razón que la sección de teoría va a hacer concreta. Corre el `mkdir` desde la raíz de `wavelength` para que la carpeta nueva quede junto a `transfer-kit`:

```bash
mkdir wavelength-checkout && cd wavelength-checkout
npm init -y
npm pkg set type=module
npm i @solana/pay@1.0.26 @solana/kit@6.10.0
npm i -D tsx esbuild
```

Los pins, con su nota de frescura: `@solana/pay` 1.0.26 es el `latest` de npm (publicado 2026-07-31, revisado de nuevo 2026-08-22) y tiene como peer a `@solana/kit ^6.9.0`, lo que te mantiene en la misma línea kit v6 que el curso usa desde el módulo 2; 6.10.0 es el último release de esa línea. Un piso duro que revisar antes de depurar cualquier otra cosa: pay 1.0.26 declara `engines.node >= 20`, porque necesita el soporte de Ed25519 de Node en `crypto.subtle` — el Node 24+ que este curso asume desde el módulo 1 lo pasa con espacio de sobra. `tsx` corre archivos TypeScript directamente, `esbuild` empaqueta un archivo para el navegador más adelante. npm 7+ te instala automáticamente el resto de las dependencias peer de pay.

Esa línea `npm pkg set type=module` no es decoración. `npm init -y` escribe un manifiesto CommonJS, y cada archivo del workspace es un módulo ES: el servidor y la prueba de humo usan los dos `import.meta.url`, que es un error de sintaxis bajo CommonJS, así que sin esa única línea el lab muere en su primera corrida con un error que no dice nada sobre módulos. Cada workspace que este curso crea de aquí en adelante lo pone, y el bloque de scaffold de cada lección incluye la línea en vez de asumir que te acuerdas.

Mientras eso corre, la versión de una frase de dónde estás en la escalera del módulo: esta lección es el checkout más simple posible, una URL que la billetera del cliente convierte en una transacción, y sus límites son exactamente lo que arregla la próxima lección.

## Resumen

Lo que esta lección establece, una línea accionable por punto:

- Un transfer request de Solana Pay —una solicitud de transferencia— es una URL: el esquema `solana:`, una dirección de destinatario y parámetros de consulta para `amount`, `spl-token`, `reference`, `label`, `message` y `memo`. La billetera la lee y construye la transferencia ella misma.
- `amount` va en unidades decimales del token. `12.5` quiere decir 12.5 USDC. Escribir unidades base (`12500000`) le cobra al cliente 12.5 millones de USDC, y es el bug más común que existe en un checkout de Solana Pay.
- La reference es una clave base58 de 32 bytes, aleatoria y nueva, que generas por cada checkout, antes de que exista pago alguno. Es la clave de unión entre tu libro mayor de pedidos y la blockchain: no puedes conocer la firma de antemano, pero la reference la elegiste tú.
- `encodeURL` construye la URL, `createQR` la convierte en píxeles (solo en el navegador, necesita un DOM), `watchReference` abre una suscripción WebSocket que se resuelve cuando aterriza una transacción que menciona tu reference, y `validateTransfer` confirma después que la transferencia pagó el monto correcto del mint correcto al destinatario correcto.
- La librería clásica de checkout vive en `typescript/packages/solana-pay/` dentro del repo de pay. El titular del repo se movió a una CLI de pagos agénticos; el subpaquete es lo que le haces `npm i`.
- La página del spec está congelada más o menos en 2022. Todavía nombra a Phantom, FTX y Slope. El spec mismo sigue siendo el estándar vivo; la página es vintage. La firma de mensajes es una extensión alpha, no parte de v1.
- Shopify anunció Solana Pay el 2023-08-23; la ruta Shopify viva hoy es el plugin de MoonPay Commerce.

Cómo se reparte el trabajo, dicho en voz alta: el lab es una construcción guiada con todos los archivos dados, salvo dos huecos deliberados. La generación de la clave de reference y las expectativas de `validateTransfer` vienen como TODOs, y llenarlos es el peldaño de completion del Challenge, con las respuestas a plena vista en la teoría de abajo. El peldaño solo agrega un segundo disco y demuestra que las dos ventas se distinguen. Te están dando menos que el módulo pasado, a propósito.

## Una URL que una billetera puede pagar

### El transfer request, campo por campo

Aquí tienes una URL de transfer-request completa, la cadena exacta que produjo mi propia corrida del código de esta lección:

```
solana:4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM?amount=12.5&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=CD9GnkB9qKo3XWX2UkDMNBc9YBTFe29ofLtASYbprvDh&label=Wavelength+Records&message=Slow+Tides%2C+Wavelength+pressing+041&memo=LP-041
```

Ningún servidor participó en hacer eso pagable. El esquema `solana:` le dice a la billetera que esto es una URL de pago. La dirección justo después es el destinatario, la billetera de tu comercio. Luego, los parámetros de consulta:

- `amount=12.5`: cuánto, en unidades decimales del token.
- `spl-token=4zMM...ncDU`: qué token, por dirección de mint. Ese es el USDC de devnet, el mint que transfer-kit viene enviando desde el módulo 2. Omite este parámetro y la billetera lee el monto como SOL nativo.
- `reference=CD9G...rvDh`: una clave de cuenta extra que la billetera adjunta a la transferencia, para que puedas encontrarla después. Más sobre esto en un momento, es la idea que carga el peso de toda la lección.
- `label`, `message`: lo que la billetera le muestra al cliente al momento de aprobar. El nombre de tu tienda, el artículo.
- `memo=LP-041`: escrito dentro de la transacción misma a través del programa memo, así que el SKU viaja on-chain junto con el pago.

Un par de esa lista merece desenredarse ahora, porque sus nombres invitan a la confusión. `message` y `memo` suenan a sinónimos y se comportan de formas que no se parecen en nada. `message` es solo de pantalla: la billetera se lo muestra al cliente al momento de aprobar, y se evapora. Nunca toca la blockchain. `memo` es lo contrario: la billetera lo escribe dentro de la transacción a través del programa memo, así que se liquida on-chain, permanentemente, al lado del pago, donde tu back office (y cualquier otro, esto es un libro mayor público) puede leerlo para siempre. La regla de oro: `message` es para el humano que aprueba, `memo` es para los sistemas que concilian. Pon el título del disco en `message` y el SKU en `memo`, nunca al revés, y nunca pongas nada en `memo` que no imprimirías en un comprobante público.

El cliente escanea, su billetera parsea esos parámetros, construye una instrucción `TransferChecked` con ellos y pide una huella. Nunca ves una clave privada, nunca construyes una transacción. La URL es todo el protocolo entre tu tienda y su billetera.

Ahora la trampa, porque merece su propio párrafo y una comparación lado a lado. Te pasaste todo el módulo 2 convirtiendo montos decimales a unidades base con `toBaseUnits`, porque las instrucciones de transferencia on-chain hablan unidades base. Una URL de Solana Pay no. El spec define `amount` como una cantidad de UI, y la billetera multiplica por los decimales del mint por ti. Los dos hábitos chocan de frente:

![Comparación lado a lado de 12.5 USDC como monto decimal en la URL frente a 12500000 unidades base en una instrucción, advirtiendo que las unidades base en una URL cobran millones.](assets/v01-comparison.webp)

Las dos convenciones son correctas donde viven. La URL habla humano, la instrucción habla unidades base, y la billetera es la traductora. Mantén `toBaseUnits` completamente fuera del código de tu URL.

### La clave de reference: elegida antes de que exista el pago

Este es el problema que resuelve la reference. La firma de un pago es el identificador único obvio para una venta, y no puedes usarla, porque no existe hasta que el cliente paga. Necesitas un identificador que controles antes de que nazca la transacción, algo que puedas escribir en tu libro mayor de pedidos al momento del checkout y después usar para reconocer el pago cuando aterrice.

Ese identificador es la reference: una clave base58 de 32 bytes, aleatoria y nueva, que generas por cada checkout. La billetera la adjunta a la transferencia como una clave de cuenta extra. No firma nada, no guarda nada, nunca es una billetera. Existe para que la transacción la mencione, porque la capa RPC de Solana puede buscar transacciones por cualquier cuenta que mencionen. Acuñas una clave que nadie ha visto jamás, la incrustas en el código QR, y la única transacción del mundo que la lleva es tu venta.

Vale la pena ser preciso sobre la mecánica, ya que explica tanto por qué esto funciona como por qué no cuesta nada. La billetera agrega tu reference a la lista de cuentas en la instrucción de transferencia como una clave que no firma ni es escribible. La cuenta detrás de esa dirección no existe y nunca existirá; sin rent, sin creación, sin estado. Es puro grafiti sobre la lista de cuentas de la transacción. Pero Solana indexa las transacciones por cada cuenta que mencionan, exista o no, que es lo que `getSignaturesForAddress` consulta por debajo. Pregúntale al RPC "¿qué transacciones mencionan `CD9G...rvDh`?" y la respuesta es tu venta y nada más en toda la historia de la blockchain. Un índice de transacciones gratis, a prueba de colisiones, preasignable, construido a partir de una dirección que nadie fondeó. Compara eso con el rodeo tradicional, una dirección de depósito única por pedido con toda la gestión de claves que eso arrastra, y la reference empieza a verse como la mejor idea que es.

![Flujo de datos: una clave de reference acuñada por el servidor viaja a través del código QR y la billetera hasta la transacción, y luego se consulta de vuelta vía getSignaturesForAddress para emparejar el pedido.](assets/v02-diagram.webp)

Si alguna vez integraste Stripe, ya te encontraste con esta forma: es tu clave de idempotencia y tu id de pedido fusionados en un solo valor, elegido del lado del cliente antes del cobro. Piénsalo como el ticket de un guardarropa. El ticket se imprime antes de que llegue el abrigo, el número corresponde a exactamente un abrigo, y tener el ticket es como lo reclamas después. Misma disciplina aquí: un checkout, una reference, nunca reutilizada. Reutiliza uno y dos ventas distintas se vuelven indistinguibles, que es precisamente la falla que el desafío solo te hace demostrar que evitaste.

Generar uno toma una sola línea con kit, y sí, es un par de claves completo del que tiramos la mitad privada de inmediato. Solo importa la dirección:

```typescript
import { generateKeyPairSigner } from '@solana/kit';

const reference = (await generateKeyPairSigner()).address;
```

Esta línea es uno de los dos huecos TODO del lab. Ya viste la respuesta.

![La URL de transfer-request dividida en partes etiquetadas: esquema solana, destinatario, monto decimal, mint de spl-token, clave de reference, label y message, y memo on-chain.](assets/v03-diagram.webp)

### Dónde vive la librería en realidad, y qué tan vieja es la página del spec

Dos advertencias honestas antes de que leas cualquier material oficial, que de otro modo te costarían una noche cada una.

Primero, el repo. El repositorio canónico de Solana Pay es `solana-foundation/pay` (la vieja URL `solana-labs/solana-pay` redirige ahí). Abre su README y no vas a encontrar tu librería de checkout. El producto titular ahora es una CLI para pagos agénticos, flujos HTTP entre máquinas, e instalar `@solana/pay` de forma global hasta te da el binario de esa CLI. La librería clásica de checkout que acabas de instalar vive en un subpaquete: `typescript/packages/solana-pay/`. No está deprecada, no está congelada y sí que está publicada: 1.0.26 salió el 2026-07-31, reconstruida sobre kit. La energía de pagos de la Foundation se mudó a otra puerta de entrada; la librería se quedó en la casa. Marca la ruta del subpaquete, no la raíz del repo.

![Árbol del repo solana-foundation/pay que muestra el README raíz como el titular de la CLI agéntica y la librería clásica de checkout viviendo en typescript/packages/solana-pay con sus cinco exports centrales.](assets/v04-diagram.webp)

Segundo, el spec. El spec de Solana Pay en docs.solanapay.com sigue siendo el estándar que toda billetera implementa, y la página en sí está congelada en algún punto de la era 2022-2023: su línea de copyright dice 2023, su reparto es puro 2022. Su línea de apertura, textual, es "Rough consensus on this spec has been reached, and implementations exist in Phantom, FTX, and Slope." Uno de esos tres colapsó de forma espectacular y otro ya no existe. Lee el spec por el protocolo, que ha envejecido bien, e ignora el reparto, que no. El repo lleva el mismo texto en `typescript/packages/solana-pay/spec/SPEC.md`, junto a dos hermanos que vale la pena conocer por nombre: `SPEC1.1.md` y `message-signing-spec.md`, los dos abren con la línea "This spec is currently alpha and subject to change." La firma de mensajes es esa extensión alpha, no parte del estándar vivo de transfer/transaction-request v1, y ningún checkout de este curso se apoya en ella.

¿Por qué importa este vintage más allá de la trivia? Porque en algún momento vas a pegar código de tutoriales viejos, todo el mundo lo hace, y el ecosistema alrededor de este spec tiene tres eras distintas: los originales de 2022 (tipos de web3.js 1.x, montos `BigNumber`), el largo medio 0.2.x (solo polling), y la línea 1.x que instalaste (tipos de kit, montos `number` simples, un watcher basado en push). El código de la era equivocada te va a tirar errores de tipos de formas confusas. Ante la duda, confía en los archivos `.d.ts` de tu propio `node_modules` por encima de cualquier blog post, incluido el yo futuro de este.

### De la URL a los píxeles: encodeURL y createQR

`encodeURL` es una función pura: campos adentro, `URL` afuera. Sin red, sin RPC, nada asíncrono:

```typescript
import { encodeURL } from '@solana/pay';
import { address } from '@solana/kit';

const url = encodeURL({
  recipient: address('4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM'),
  amount: 12.5,
  splToken: address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU'),
  reference,
  label: 'Wavelength Records',
  message: 'Slow Tides, Wavelength pressing 041',
  memo: 'LP-041',
});
```

Cada campo mapea uno a uno con las partes de la URL que acabas de leer. `recipient`, `splToken` y `reference` son valores `Address` de kit, y por eso `address(...)` envuelve las cadenas: valida el base58 a nivel de tipos en vez de a la hora del escaneo.

`createQR(url, size, background, color)` convierte la URL en un objeto QR con estilos. Un hecho estructural decide tu arquitectura: necesita un DOM. Llámalo en Node y lanza `document is not defined`; yo lo hice, lo lanza. Así que el código QR pertenece a un archivo del navegador, y la construcción de la URL pertenece a tu servidor, y esa división es de todos modos la frontera de confianza correcta. El servidor decide qué está en venta y a qué precio; el navegador solo convierte una URL terminada en píxeles. Para pruebas del lado del servidor está `createQROptions`, que construye la configuración del código QR sin tocar el DOM. La prueba de humo del lab ni siquiera llega tan lejos: como el código QR se dibuja en el navegador, lo único que una comprobación del lado de Node puede afirmar honestamente es que el bundle del navegador se construyó, que es exactamente lo que afirma.

La librería también trae el espejo de `encodeURL`: `parseURL`. Pásale una URL y te devuelve los campos tipados, destinatario y monto y todo lo demás, o lanza si la cadena está malformada. Esta es literalmente la mitad del protocolo que le toca a la billetera; cuando un teléfono escanea tu código QR, algo con exactamente la forma de `parseURL` corre del otro lado del vidrio. Lo que la vuelve una herramienta de prueba silenciosamente perfecta para tu lado: si tu URL recién codificada sobrevive un viaje de ida y vuelta por el parser de la propia librería con el monto intacto, una billetera que cumpla el spec la va a leer como la pensaste. No hace falta teléfono. La prueba de humo se apoya en esto, y es un hábito que vale la pena robarse para cualquier trabajo de protocolo: cuando una librería trae las dos direcciones de un códec, el viaje de ida y vuelta es la comprobación de correctitud más barata que vas a escribir.

### Escuchar el pago aterrizar: watchReference

Tu página ahora muestra un código QR. En algún lugar allá afuera un cliente aprueba el pago en su teléfono. Tu servidor necesita enterarse. No la pestaña del navegador, tu servidor: un callback de "pago exitoso" en el frontend es falsificable por cualquiera con las devtools abiertas, y construir el reflejo de comercio de nunca confiar en él es la mitad de lo que trata el módulo 4.

La librería te da dos herramientas de detección, una de forma vieja y una nueva.

`findReference(rpc, reference)` le pregunta al RPC por HTTP: ¿alguna transacción ha mencionado esta clave de reference? Devuelve la firma más antigua que coincida y lanza `FindReferenceError` cuando todavía no ha aterrizado nada. Es una pregunta de un solo tiro, así que usarla sola quiere decir preguntar una y otra vez con un temporizador. Ese loop de polling era toda la historia de la línea legacy 0.2.x, y la propia app de punto de venta todavía funciona así.

`watchReference(rpcSubscriptions, reference, options)` es el camino 1.x y honestamente una bendición para cualquiera que haya escrito la versión con polling. Abre una suscripción WebSocket (`logsNotifications`, filtrada a transacciones que mencionan tu reference) y devuelve una promesa que se resuelve en el momento en que aterriza una transacción que coincide, con la firma y su estado de error on-chain. Sin temporizador, sin aritmética de reintentos, sin peticiones desperdiciadas. Lo armas cuando se abre el checkout y esperas:

```typescript
import { watchReference } from '@solana/pay';
import { createSolanaRpcSubscriptions } from '@solana/kit';

const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

const { signature, err } = await watchReference(rpcSubscriptions, reference, {
  commitment: 'confirmed',
  abortSignal: controller.signal,
});
```

El `abortSignal` importa en código real: un checkout del que el cliente se aleja no debería sostener una suscripción para siempre. Abórtala y la promesa se rechaza con `FindReferenceError`, que tu código trata como "venta expirada", no como un crash. En mi propia corrida contra devnet, armar el watcher sobre una reference nueva y abortar tres segundos después hizo exactamente eso, limpiamente.

La opción `commitment` es una decisión de política para la que ya tienes el vocabulario. El módulo 1 construyó la escalera: `confirmed` quiere decir que una supermayoría del clúster votó por el bloque de la transacción, `finalized` quiere decir que la blockchain ya construyó tan por encima de él que el rollback queda descartado. Para un disco de 12.5 USDC, `confirmed` es la decisión de comercio correcta, el mismo juicio al que llega la tabla de políticas del módulo 1 para bienes de monto bajo: el riesgo residual es minúsculo y el cliente está ahí parado esperando. ¿Vendes algo que no puedes recuperar a un precio que dolería? Observa en `confirmed` para el comprobante rápido, entrega la mercancía en `finalized`. El watcher acepta cualquiera de los dos; el punto es que el parámetro es una decisión de negocio con un nombre técnico puesto, y dejarlo por defecto sin decidir sigue siendo una decisión.

Suscripción WebSocket, en una frase para cualquiera que solo haya hecho polling contra APIs REST: en vez de que preguntes una y otra vez "¿ya llegó algo?", mantienes abierta una sola conexión de larga vida y el nodo RPC te empuja la respuesta en el momento en que existe. Igual mantén `findReference` en tu caja de herramientas. Un WebSocket que se cae durante el pago se pierde la notificación, y un poll es cómo barres lo que se le escapó a una suscripción. Los checkouts de producción del módulo 4 corren los dos: suscríbete por velocidad, barre por verdad.

![Comparación de findReference como un loop de polling HTTP repetido frente a watchReference como una sola suscripción WebSocket que empuja la firma cuando la transferencia aterriza.](assets/v05-comparison.webp)

### La confianza llega al final: validateTransfer

Que el watcher se resuelva no quiere decir que te pagaron. Quiere decir que aterrizó una transacción que menciona tu reference. Esas son afirmaciones distintas. Cualquiera puede enviar una transacción que mencione tu clave de reference: un pago del monto equivocado, del token equivocado, al destinatario equivocado, o una transferencia que falló on-chain pero que igual lleva la cuenta.

`validateTransfer` cierra la brecha. Le pasas la firma que atrapó el watcher más una declaración de lo que se suponía que era esta venta, y busca la transacción liquidada y contrasta la historia con la blockchain:

```typescript
import { validateTransfer } from '@solana/pay';
import { createSolanaRpc } from '@solana/kit';

const rpc = createSolanaRpc('https://api.devnet.solana.com');

await validateTransfer(
  rpc,
  signature,
  {
    recipient: MERCHANT,      // the money went to you
    amount: 12.5,             // decimal units, same convention as the URL
    splToken: USDC_DEVNET,    // it was actually USDC, not a lookalike mint
    reference,                // and it is THIS sale, not another one
  },
  { commitment: 'confirmed' },
);
```

Si falla cualquier expectativa, lanza `ValidateTransferError` y no hiciste una venta, sea lo que sea que muestre el navegador. El objeto de expectativas es el segundo hueco TODO del lab, y ahora también viste esta respuesta. El modelo mental para llevarte al módulo 4: el watcher es tu timbre, `validateTransfer` es revisar que el dinero es real antes de entregar el disco. Un timbre no es un pago.

Armado todo junto, una venta fluye así:

![Flujo de venta de cuatro carriles donde el servidor acuña una reference y arma un watcher, el navegador renderiza el código QR, la billetera envía la transferencia y validateTransfer la confirma.](assets/v06-flowchart.webp)

### Quién ha corrido esta forma a escala

Este patrón de URL y reference no es un juguete de salón de clases. El 2023-08-23, Shopify anunció Solana Pay como opción de pago en toda su red de comercios, con MonkeDAO, Mad Lads y Helius entre los primeros usuarios. El argumento que hizo quien lidera las integraciones en Shopify fue pura economía de comercio, la misma aritmética del módulo 1: sin comisiones bancarias, sin chargebacks, sin retenciones de varios días sobre tus propios ingresos. Una venta con tarjeta es un préstamo que la red puede revertir durante meses; una transferencia de stablecoin liquidada es final en segundos, y para un comercio que corre con márgenes delgados esa diferencia es todo el argumento. La ruta de integración ha cambiado de manos desde entonces, como suele pasar con la plomería del comercio: la ruta viva de hoy para una tienda de Shopify es el plugin de MoonPay Commerce. El spec de abajo es el que estás implementando ahora mismo.

![Línea de tiempo desde el spec de Solana Pay de 2022 pasando por el anuncio de Shopify de 2023 con sus tres primeros usuarios nombrados, hasta la librería basada en kit de 2026 y la ruta de MoonPay Commerce.](assets/v07-timeline.webp)

Ahora la contrapartida, porque el peldaño de esta lección tiene un techo afilado y deberías sentirlo antes de construir. Todo lo que la billetera sabe de esta venta vino de una URL que imprimiste en una pantalla, y una vez que está en el lado del vidrio que le toca al cliente no controlas nada de eso. El monto, el mint, la reference: todo eso son datos en las manos del cliente antes de volverse una transacción. `validateTransfer` quiere decir que la manipulación no puede engañarte, un pago alterado simplemente falla la validación. Pero tampoco puede expresarte. Sin totales de carrito calculados del lado del servidor, sin lógica de cupones, sin memo dinámico, sin ningún estado de pedido más allá de una clave de reference por carga de página. Un transfer request es simplísimo y trustless, y es exactamente un disco a un precio. Ese techo es el problema de apertura de la próxima lección.

## Lab: vende un disco en devnet

Construcción guiada. Cada archivo de abajo va en el workspace `wavelength-checkout` que creaste arriba. Dos archivos vienen con huecos TODO, marcados a gritos; la construcción corre hasta una falla específica y nombrada con ellos puestos, y el Challenge los cierra.

1. **El disco.** Crea `checkout/record.ts`. Un disco, un precio, más las dos direcciones que importa todo lo demás:

   ```typescript
   import { address } from '@solana/kit';

   // Your devnet merchant wallet. Paste the address you funded in module 2.
   export const MERCHANT = address('4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM');

   // Devnet USDC, the same mint transfer-kit has been sending since module 2.
   export const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');

   export const RECORD = {
     sku: 'LP-041',
     title: 'Slow Tides, Wavelength pressing 041',
     priceUsdc: 12.5, // decimal token units. NOT base units. Never 12500000.
   };
   ```

   Reemplaza `MERCHANT` con tu propia dirección de devnet o la venta no te va a llegar para nada.

2. **La URL de pago, con el hueco número uno.** Crea `checkout/payment.ts`:

   ```typescript
   import { encodeURL } from '@solana/pay';
   import { generateKeyPairSigner, type Address } from '@solana/kit';
   import { MERCHANT, USDC_DEVNET, RECORD } from './record.ts';

   // One fresh reference per checkout: a random 32-byte base58 key chosen
   // BEFORE the payment exists. The join key between this sale and the chain.
   export async function newReference(): Promise<Address> {
     // TODO(completion): mint a fresh key and return its address.
     // One line. The theory section shows it, generateKeyPairSigner is
     // already imported, and the smoke test will fail here until you do.
     throw new Error('TODO: newReference');
   }

   export function buildPaymentURL(reference: Address): URL {
     return encodeURL({
       recipient: MERCHANT,
       amount: RECORD.priceUsdc,
       splToken: USDC_DEVNET,
       reference,
       label: 'Wavelength Records',
       message: RECORD.title,
       memo: RECORD.sku,
     });
   }
   ```

3. **El lado del navegador.** Crea `checkout/page.ts`. Este es el único archivo que llega a tocar `createQR`, por la razón del DOM que vimos en la teoría:

   ```typescript
   import { createQR } from '@solana/pay';

   // The server stamped the encoded URL onto the mount node; all this file
   // does is turn it into pixels. createQR needs a DOM, which is why it
   // lives here and not in server.ts.
   const mount = document.getElementById('qr')!;
   const qr = createQR(mount.dataset.url!, 512, 'white', 'black');
   qr.append(mount);
   ```

   Empaquétalo para el navegador:

   ```bash
   npx esbuild checkout/page.ts --bundle --format=esm --outfile=checkout/public/page.js
   ```

   Checkpoint: esbuild reporta el archivo de salida. El mío salió en 94.7kb, la librería de estilos del código QR es la mayor parte.

4. **El watcher, con el hueco número dos.** Crea `checkout/watcher.ts`:

   ```typescript
   import { watchReference, validateTransfer } from '@solana/pay';
   import {
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     type Address,
     type Signature,
   } from '@solana/kit';
   import { MERCHANT, USDC_DEVNET, RECORD } from './record.ts';

   const rpc = createSolanaRpc('https://api.devnet.solana.com');
   const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

   // Resolves with the signature once a transfer carrying `reference` lands
   // on devnet AND survives validation against what this sale should be.
   export async function awaitSale(
     reference: Address,
     abortSignal?: AbortSignal,
   ): Promise<Signature> {
     const { signature, err } = await watchReference(rpcSubscriptions, reference, {
       commitment: 'confirmed',
       abortSignal,
     });
     if (err) {
       throw new Error(`transfer ${signature} landed but failed on-chain`);
     }

     const expected = {
       recipient: MERCHANT,
       amount: 0, // TODO(completion): the record's real price, decimal units
       // TODO(completion): add splToken and reference. Without the mint
       // check, 12.5 of any token passes. Without the reference, any sale
       // passes as this one.
     };
     await validateTransfer(rpc, signature, expected, { commitment: 'confirmed' });
     return signature;
   }
   ```

   Tal como viene, este watcher va a atrapar un pago y después lo va a rechazar, porque ninguna transferencia real valida contra un monto esperado de cero. Eso es deliberado. El timbre funciona; la comprobación del dinero te toca a ti escribirla.

5. **El servidor.** Crea `checkout/server.ts`. `node:http` pelado, nada más; cada GET de la página es un checkout, así que cada visita acuña una reference, codifica una URL y arma un watcher antes de que el HTML siquiera salga del socket:

   ```typescript
   import { createServer } from 'node:http';
   import { readFileSync } from 'node:fs';
   import { newReference, buildPaymentURL } from './payment.ts';
   import { awaitSale } from './watcher.ts';
   import { RECORD } from './record.ts';

   const pageJs = readFileSync(new URL('./public/page.js', import.meta.url), 'utf8');

   const server = createServer(async (req, res) => {
     if (req.url === '/page.js') {
       res.writeHead(200, { 'content-type': 'text/javascript' });
       res.end(pageJs);
       return;
     }

     // Every GET / is one checkout: fresh reference, fresh QR, fresh watcher.
     const reference = await newReference();
     const url = buildPaymentURL(reference);

     awaitSale(reference)
       .then((signature) => {
         console.log(`SOLD ${RECORD.sku} ref=${reference} sig=${signature}`);
       })
       .catch((err) => {
         console.error(`checkout ${reference} did not validate:`, err.message);
       });
     console.log(`checkout open ref=${reference}`);

     res.writeHead(200, { 'content-type': 'text/html' });
     res.end(`<!doctype html>
   <title>Wavelength Records</title>
   <h1>${RECORD.title}</h1>
   <p>${RECORD.priceUsdc} USDC (devnet)</p>
   <div id="qr" data-url="${url.toString()}"></div>
   <script type="module" src="/page.js"></script>`);
   });

   server.listen(3010, () => {
     console.log('Wavelength checkout on http://localhost:3010');
   });
   ```

   Fíjate en lo que el servidor nunca hace: nunca le dice al navegador si el pago ocurrió. El comprobante es una línea de log del lado del servidor. Cablear el estado de la venta de vuelta a la página es trabajo real con decisiones de confianza reales, y le pertenece al módulo del back office.

   Fíjate también en lo que esta versión de juguete deja escapar, porque ver la fuga ahora te ahorra una sesión de depuración en el módulo 4. Cada GET arma un watcher sin señal de abort y sin expiración. Recarga la página cinco veces y tienes cinco suscripciones WebSocket vivas, cuatro de ellas huérfanas que se van a quedar sentadas sobre la conexión RPC hasta que el proceso muera. Bien para un lab en devnet, un problema real con cualquier tráfico. Un checkout de producción tiene un ciclo de vida: se abre, expira después de algunos minutos, su watcher se aborta, y un barrido periódico de `findReference` atrapa cualquier cosa que haya pagado después de que la suscripción se cerró. Ya construiste la maquinaria de abort (`awaitSale` acepta una señal, la prueba de humo la ejercita); este servidor simplemente no la usa todavía. El módulo del back office les da a los checkouts ese ciclo de vida como se debe, junto con la persistencia que esta línea de log está supliendo.

![Manejador de peticiones anotado que muestra la secuencia por checkout: acuñar una reference nueva, codificar la URL de pago, armar el watcher, luego servir la página.](assets/v08-annotated-code.webp)

6. **La prueba de humo.** Crea `checkout/smoke.ts`, la verificación estándar por lección del módulo:

   ```typescript
   import { parseURL } from '@solana/pay';
   import { newReference, buildPaymentURL } from './payment.ts';
   import { awaitSale } from './watcher.ts';
   import { RECORD } from './record.ts';
   import { existsSync } from 'node:fs';

   const main = async () => {
     // 1. The URL round-trips through the library's own parser.
     const reference = await newReference();
     const url = buildPaymentURL(reference);
     const parsed = parseURL(url);
     if (!('recipient' in parsed) || parsed.amount !== RECORD.priceUsdc) {
       throw new Error(`URL did not round-trip: ${url}`);
     }
     console.log('transfer-request URL valid');

     // 2. The browser bundle that draws the QR exists. We cannot
     //    render a QR in Node, so this is the honest check.
     if (!existsSync(new URL('./public/page.js', import.meta.url))) {
       throw new Error('checkout/public/page.js missing: run the esbuild step');
     }
     console.log('QR bundle built');

     // 3. The watcher arms against devnet and cancels cleanly.
     const controller = new AbortController();
     const armed = awaitSale(reference, controller.signal).catch(() => 'aborted');
     await new Promise((r) => setTimeout(r, 2000));
     controller.abort();
     if ((await armed) !== 'aborted') {
       throw new Error('watcher resolved without a payment?');
     }
     console.log('reference watcher armed');
     process.exit(0);
   };
   main();
   ```

7. **Córrelo hasta la falla diseñada.**

   ```bash
   npx tsx checkout/smoke.ts
   ```

   Checkpoint: muere de inmediato con `TODO: newReference`. Ese error exacto es la línea de meta de este lab. El scaffold está completamente cableado, la página se empaqueta, el código del servidor está completo, y los dos huecos entre tú y una tienda que funciona son los dos conceptos que esta lección existe para enseñar. Cerrarlos es el Challenge.

## Challenge

Tres peldaños, y el del medio es donde la tienda cobra vida.

**Worked.** Listo: el lab de arriba era eso, terminando en la falla TODO nombrada. Si tu prueba de humo falla con cualquier cosa que no sea `TODO: newReference`, arregla eso primero; los dos sospechosos de siempre son un `checkout/public/page.js` faltante (vuelve a correr el paso de esbuild) y un typo en una dirección literal, que `address(...)` rechaza al momento del import con un error de base58.

**Completion.** Llena los dos huecos. En `payment.ts`, haz que `newReference` acuñe y devuelva una dirección nueva; en `watcher.ts`, reemplaza el objeto `expected` con la historia verdadera de la venta: el `amount` real, más `splToken` y `reference`. Las dos respuestas aparecen textuales en la sección de teoría. La aceptación, en dos etapas. Primero, `npx tsx checkout/smoke.ts` imprime exactamente tres líneas: `transfer-request URL valid`, `QR bundle built`, `reference watcher armed`.

Segundo, la venta misma, y necesita un cliente que no seas tú. `validateTransfer` mide el *cambio* de saldo en la cuenta de token del destinatario, así que una billetera que se paga a sí misma no mueve nada y la venta se rechaza con `amount not transferred` por más correcto que esté tu código. Tu par de claves de comercio es el destinatario, así que no puede ser también el comprador. Abastece una segunda billetera antes de arrancar el servidor, desde la raíz de `wavelength`:

```bash
CUSTOMER=$(solana-keygen pubkey /tmp/customer.json)   # the pretend customer from module 2 lesson 1
solana transfer "$CUSTOMER" 0.05 --allow-unfunded-recipient --url devnet
npm run --workspace transfer-kit pay -- "$CUSTOMER" 13
```

La primera línea le da SOL al cliente para pagar las comisiones de transacción; la segunda mueve 13 USDC de devnet fuera del saldo de tu comercio, suficiente para una venta de 12.5 con vuelto. Fondear una billetera en la moneda con la que está a punto de pagarte se siente circular, y lo es: en devnet eres los dos lados del mostrador y el dinero vuelve a casa al final de la corrida. Si tu saldo de comercio no cubre 13, el faucet de Circle te deja otra gota una vez que pasó su cooldown.

Ahora corre `npx tsx checkout/server.ts`, abre `http://localhost:3010`, anota la línea `checkout open ref=...` que imprime, y paga ese código QR en devnet por una de dos rutas.

*Teléfono.* Escanéalo con una billetera móvil cambiada a devnet — la billetera que creaste en el módulo 1, abastecida exactamente de la misma forma, sustituyendo `$CUSTOMER` por la dirección que te muestra la app de la billetera en los dos comandos de arriba. La mayoría de las billeteras esconden el cambio a devnet detrás de un toggle en los ajustes de desarrollador; si la tuya no cambia de red para nada, sáltate el teléfono, la ruta por la CLI pasa exactamente la misma barrera.

*CLI.* Deja que tu propio tooling haga de cliente. El script `pay` de transfer-kit ganó la lección pasada exactamente las dos perillas que esto necesita: `PAYER_KEYFILE` elige la billetera que firma, y un cuarto argumento acepta una reference que acuñó alguien más. Desde la raíz de `wavelength`:

```bash
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5 <ref-from-the-server-log>
```

`$(solana address)` es la dirección de comercio que pegaste en `record.ts`, `12.5` es la cadena decimal que el kit convierte a unidades base por ti (la URL y el kit coinciden en unidades decimales aquí; solo la instrucción de abajo habla unidades base), y la reference es lo que le deja al watcher que tu servidor ya armó reconocer esta transferencia como su venta.

De cualquier forma, la aceptación es una línea de log del servidor: `SOLD LP-041 ref=<your reference> sig=<signature>`. Esa firma es una transacción de devnet real; búscala en cualquier explorador y encuentra tu memo `LP-041` ahí sentado on-chain.

**Solo.** Wavelength surte un segundo disco. Agrégalo a `record.ts` a un precio distinto, sírvelo en `/lp-042` con su propio checkout, y vende los dos. Primero recárgale saldo a la billetera del cliente con la misma línea `pay` — su vuelto de la primera venta no alcanza para una segunda — y después cobra las dos. Aceptación: dos líneas de log `SOLD` con dos references distintos, cada una validada contra su propio precio, y puedes decir en voz alta cuál reference pertenece a cuál disco sin leer los montos. Si reutilizaste una reference para las dos, ya sabes cuál frase de la teoría te saltaste. Las decisiones de diseño son tuyas: funciones de watcher por disco, un mapa de discos, lo que sea que sostenga dos SKUs honestamente.

Si la validación sigue rechazando un pago que estás seguro de haber enviado, lee el mensaje de `ValidateTransferError` antes de tocar código, ese mensaje nombra la expectativa que falló. `amount not transferred` sobre una transferencia que puedes ver en un explorador quiere decir que el pagador y el destinatario eran la misma billetera: revisa que `PAYER_KEYFILE` esté de verdad en el entorno del comando que corriste, porque sin él el script firma como el comercio y el delta es cero. Un *desajuste* genuino de monto es normalmente el bug de unidades decimales apareciendo en un lugar nuevo: revisa si algo en tu ruta de envío multiplicó por un millón una vez de más.

Una petición antes de que cierres la terminal: si algún paso te peleó, anota cuál. Las lecciones que le quedan a este módulo asumen que este scaffold entró limpio, y la lista de fricciones de los lectores es cómo el curso se vuelve más afilado. Donde salió suave, tómate la victoria; levantaste un riel de pago a partir de un spec de URL y cinco funciones, y la mayoría de la gente que integra checkouts con tarjeta nunca ha visto su propio dinero confirmarse ni una sola vez.

Tu tienda ahora vende un disco a un precio hardcodeado, y cada dato de la venta todavía viaja por una URL que controla la billetera del cliente. Un carrito real tiene un total calculado a partir de líneas de artículo, un cupón que lo cambia, un id de pedido de tu base de datos, y nada de eso pertenece a una cadena que el cliente puede editar. La próxima lección, "Transaction requests: el servidor construye la transacción", invierte la dirección de la confianza: la billetera deja de construir la transferencia desde tu URL y empieza a pedirle a tu servidor una transacción que construiste tú. El watcher que acabas de escribir se viene contigo sin cambios.
