# Transaction requests: el servidor construye la transacción

La lección pasada, el checkout puso un código QR en una página y un watcher detrás: la reference key que incrustaste dejó que tu servidor emparejara la transferencia de devnet entrante y la validara. Progreso de verdad. Pero fíjate en quién hizo la construcción. La billetera del cliente armó esa transacción a partir de una URL fija, lo que quiere decir que un disco a un precio es todo lo que tu checkout puede expresar.

Ahora imagínate la tienda de verdad. Un cliente tiene tres discos en un carrito, un código de cupón que llegó en el newsletter del mes pasado, y un id de pedido que tu libro mayor necesita estampado on-chain. ¿Dónde viven esas cosas? En una URL no. Una URL es una cadena que sostiene el cliente, y cada carácter de ella es editable antes de que su billetera actúe siquiera sobre ella. Un total en una URL es una sugerencia. Un cupón en una URL es una invitación.

¿La bala de plata? Basta de dejar que la billetera construya la transacción. Esta lección invierte el protocolo: la billetera te trae la cuenta del cliente, y tu servidor te devuelve una transacción completamente construida con el carrito ya con precio, el memo estampado y la reference inyectada, todo por código que controlas tú. Genera el scaffold del workspace ahora para que la instalación corra mientras lees. Corre el `mkdir` desde la raíz de `wavelength`, para que la carpeta nueva quede junto a `transfer-kit` y los imports relativos de abajo resuelvan:

```bash
mkdir -p checkout-txreq/src checkout-txreq/public
cd checkout-txreq
npm init -y
npm pkg set type=module
npm install @solana/kit@6.10.0 @solana-program/token@0.14.0 express@5
npm install -D tsx@4 typescript @types/express @types/node
```

Los pins, revisados el 2026-08-22: `@solana/kit` 6.10.0 es el último release de la línea v6, y este workspace se queda en v6 porque `@solana/pay` 1.0.26 (publicado el 2026-07-31, todavía el `latest` de npm) tiene como peer a kit ^6.9. `@solana-program/token` 0.14.0 es el último minor compatible con kit v6; la ola 0.15.x tiene como peer a kit ^7, así que rompería este workspace. `express` 5 es el major actual. `tsx` es el runner que has usado todo el curso. Una ausencia deliberada: NO agregues `@solana/pay` a este workspace. Su rango de peers quiere `@solana-program/token` ^0.12, que pelea con el pin 0.14.0, y de todos modos este servidor nunca codifica una URL; la librería de pay se queda allá en el proyecto de checkout, donde vive la página.

## Resumen

El índice de hallazgos, cada línea accionable:

- Un transaction request —una solicitud de transacción— le da vuelta al esquema de la URL: `solana:<https-link>` en vez de `solana:<recipient>`. Una diferencia de forma, un protocolo completamente distinto por debajo.
- La billetera le habla dos verbos a tu link. GET devuelve `{ label, icon }` para que la billetera pueda mostrar quién está pidiendo dinero. POST `{ account }` devuelve `{ transaction }`, una transacción en base64 construida para ese pagador específico.
- Entregas **checkout-txreq**: un endpoint de Express en `/txreq` (puerto 3100, http pelado en local; las billeteras van a querer https, y la lección del punto de venta le pone un proxy) más `buildOrderTransaction`, el núcleo de precio y ensamblado que reusa cada superficie posterior de este curso.
- El precio pasa por completo al lado del servidor. La URL solo lleva un id de pedido opaco; el total, la aritmética del cupón, el memo y la reference key los calcula e inyecta tu código. Nunca confíes en un precio que manda el cliente.
- El canje es una nueva frontera de confianza: la billetera ahora firma una transacción que escribió tu servidor. La regla del spec la vigila: una solicitud que exige la firma de una cuenta que el usuario no envió es maliciosa y debe rechazarse. Ese rechazo lo vas a implementar tú mismo.
- Una transacción construida lleva incrustado un blockhash, válido por 150 bloques. Con el tiempo de slot objetivo actual de 300ms eso son alrededor de 45 segundos de reloj de pared, y SIMD-0525 ya tiene dos recortes más de tiempo de slot preparados; el número se deriva del tiempo de slot, así que derívalo, nunca lo memorices.

Tu parte del trabajo hoy: los handlers de GET y POST vienen resueltos, con cada línea dada. El cálculo del total en el servidor y la inyección del memo y la reference vienen como TODOs en el scaffold, y la sección de teoría te enseña exactamente qué va adentro de ellos. El camino del cupón y la guarda contra solicitudes maliciosas son solo tuyos al final.

## Quién construye la transacción ahora

### Un carácter de diferencia, un protocolo invertido

Pon las dos formas de URL lado a lado. Un transfer request es `solana:<recipient>?amount=...&spl-token=...&reference=...`: el destinatario es la URL, los parámetros son la transacción, y la billetera hace el ensamblado. Un transaction request es `solana:<https-link>`: el payload es un link a tu servidor, y el trabajo de la billetera se encoge a buscar, mostrar y firmar. El mismo prefijo `solana:`. Si lo que sigue parsea como una URL https, la billetera lo trata como un transaction request; si parsea como una dirección, como un transfer request. Las billeteras despachan sobre esa forma, y tu modelo mental también debería, porque todo lo demás en este módulo cuelga de en qué lado de esa bifurcación estás.

![Las dos formas de URL que define Solana Pay, lado a lado: el transfer request construido por la billetera exponiendo cada parámetro, frente al transaction request construido por el servidor que solo lleva un link y un id de pedido.](assets/v01-comparison.webp)

¿Por qué vino primero la lección del QR, entonces? Porque el transfer request es trustless de una forma a la que esta lección renuncia a propósito. No había servidor que comprometer: la billetera construía la transferencia ella misma a partir de parámetros que el cliente podía inspeccionar. Simplísimo, y exactamente tan limitado como suena. En el momento en que quieres lógica de carrito, necesitas código corriendo en algún lado que el cliente no puede editar, y código corriendo en algún lado quiere decir confianza en ese algún lado. Guarda ese pensamiento; se vuelve la sección de seguridad.

Si cargas cicatrices de Stripe, esta inversión se va a sentir familiar. El flujo solo de cliente, con un precio horneado adentro de un widget del frontend, es justo de lo que cada guía de integración de Stripe te advierte en la primera página. El flujo adulto es un PaymentIntent: tu backend decide el monto, adjunta los metadatos y le entrega al cliente algo ya con precio. Un transaction request es esa misma forma sobre rieles de Solana. La lógica de precios vuelve a tu backend, donde siempre perteneció, y la billetera se vuelve la superficie de confirmación en vez de la calculadora.

### El viaje de ida y vuelta: GET, después POST

La billetera hace dos llamadas a tu link, en un orden fijo, y el orden es el punto.

Primero le hace GET a tu endpoint. Tu respuesta es chica: una cadena `label` y una URL `icon`. Eso no es decoración. La billetera está a punto de pedirle a un humano que apruebe un pago construido por un servidor desconocido, y el GET le da algo honesto que mostrar antes de que pase nada financiero: a quién le estás pagando, con una cara. Honesto, con una advertencia que vale la pena decir en voz alta: el label es autodeclarado. Cualquier servidor puede contestar "Wavelength Records", que es por lo que las billeteras bien construidas muestran tu dominio al lado de tu label, y por lo que la identidad que un cliente puede verificar de verdad es el origen https, no la cadena que elegiste tú. Tu trabajo es mantener esas dos apuntando al mismo negocio. Recién después de renderizar ese contexto la billetera hace POST de `{ "account": "<base58 pubkey>" }`, la clave pública del cliente, a la misma URL. Tu servidor ahora sabe lo único que no podía saber de antemano, quién está pagando, y puede construir la transacción para exactamente ese pagador: su cuenta como fee payer, su cuenta de token como la fuente de los fondos.

![Diagrama de flujo desde un carrito guardado pasando por el QR, el GET de la billetera por label e icon, el POST de la cuenta, el precio del lado del servidor, la firma y el watcher de la reference confirmando.](assets/v02-flowchart.webp)

Fíjate en lo que te compra la división. El GET es cacheable, sin autenticación, seguro de golpear cien veces. El POST es por cliente y devuelve una transacción que solo es válida por poco tiempo, porque tu servidor le estampa un blockhash reciente. Un blockhash no puede tener más de 150 bloques de antigüedad cuando la transacción aterriza; con el tiempo de slot objetivo actual de 300ms eso da alrededor de 45 segundos. Deriva esa ventana del tiempo de slot cada vez que la cites, porque el tiempo de slot es lo que se mueve: era de ~60 segundos con los viejos slots de 400ms y de ~53 en la etapa de 350ms de SIMD-0525, la etapa de 300ms entró en vigor en la epoch 1024, y ya hay dos recortes más detrás de un feature gate en el código. En la práctica: construye en el POST, nunca en el GET, y nunca cachees una transacción construida. Un cliente que escanea, se va a dar una vuelta y firma dos minutos después se lleva un blockhash vencido y un envío fallido, que es molesto pero seguro; su billetera simplemente vuelve a pedir.

¿No podría la billetera mandarte el carrito entero en ese POST, y saltarse el id de pedido? No, y la restricción es una ventaja. La billetera habla el spec, y el body del POST del spec es `{account}`, nada más. Cualquier otra cosa que tu build necesite tiene que viajar en la URL que acuñaste, que es exactamente por lo que la URL lleva un id de pedido opaco que apunta a estado que tu servidor ya tiene. La billetera se queda como un firmante tonto y auditable; tu servidor se queda como el único autor de la lógica de negocio. En el momento en que te encuentres deseando que la billetera te mandara más, normalmente estás tratando de mover el precio de vuelta al cliente, y ya sabes cómo termina esa historia.

Una consecuencia de construir-en-el-POST merece su propio párrafo: el POST no es idempotente, y está bien. Golpea tu endpoint dos veces por el mismo pedido y te llevas dos transacciones distintas, cada una con su propia reference key nueva, porque `buildOrderTransaction` acuña una por llamada. Una billetera que vuelve a pedir después de un blockhash vencido hace exactamente esto. Relájate: como máximo una de esas transacciones llega a liquidar, el cliente firma una, y tu watcher empareja la reference que de verdad aterrice on-chain. Lo que NO tienes que hacer es tratar "construí una transacción" como "hice una venta". Un build es una cotización. La liquidación es la venta, y el módulo del back office formaliza esa distinción con idempotencia por firma del lado del libro mayor.

### Qué ensambla el servidor

Hora de ser concreto sobre la cosa que devuelve tu handler de POST. Es una transacción versión 0 con el cliente como fee payer y dos instrucciones adentro.

La última instrucción es el pago mismo: un `TransferChecked` que mueve el total del carrito en USDC de devnet (mint `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`, 6 decimals) desde la cuenta de token asociada del cliente hasta la tuya. Esto es territorio de transfer-kit del módulo 2, y lo reusas directo: `resolveAta(owner, mint, tokenProgram)` encuentra las dos cuentas de token, pasándole el programa Token clásico para el USDC de devnet, `toBaseUnits` mantiene la aritmética del dinero en bigints exactos. Los totales se calculan a partir de cadenas decimales, se suman como unidades base, nunca se flotan. El único truco nuevo es que el cliente es la `authority` pero todavía no ha firmado nada; tu servidor construye una transacción sin firmar y la firma llega después, en su billetera. Kit está cómodo con eso: ensamblas el mensaje, lo compilas y lo serializas con el espacio de la firma vacío.

![Diagrama de la transacción devuelta: un sobre v0, el cliente como fee payer sin firmar, un memo con el id de pedido, y una instrucción TransferChecked que lleva una cuenta de reference de solo lectura.](assets/v03-diagram.webp)

Antes de seguir, el modo de falla que de verdad te va a morder en una demo: cuentas de token que no existen. `TransferChecked` mueve fondos entre cuentas de token asociadas, y una ATA solo existe una vez que alguien pagó su rent —el mínimo exento de rent para una cuenta de token de 165 bytes, 1,488,440 lamports leídos de devnet el 2026-09-07 y bajando a medida que SIMD-0437 escalona la tasa hacia abajo—, la línea de costo que costeaste en el módulo 2 preguntándole al clúster en vez de confiar en una constante. Tu ATA de destino del comercio existe porque transfer-kit la creó cuando recibiste USDC por primera vez. La ATA de origen del cliente es la riesgosa: una billetera que nunca ha tenido USDC de devnet no tiene cuenta de USDC, tu transacción construida referencia una dirección sin nada detrás, y la simulación previa a la firma de la billetera falla con un error que el cliente te va a leer en voz alta. No hay arreglo del lado del servidor adentro de esta transacción sin hacerte cargo del rent de desconocidos; el manejo honesto es una comprobación en la tienda (su saldo es visible on-chain antes de que siquiera renderices el QR) y un mensaje claro. Los checkouts de producción hacen exactamente esto, y el tuyo lo va a hacer en el capstone.

Justo antes de la transferencia se sienta un SPL Memo que lleva `wavelength:<orderId>:<description>`. En la lección del QR el memo viajaba de acompañante como un parámetro de la URL y la billetera lo incluía; ahora tu servidor lo escribe directo, lo que quiere decir que puede llevar tu id de pedido real y un resumen del carrito parseable por máquina, y el cliente no puede editar ninguno de los dos. Tu módulo de back office se va a apoyar en esto, y fuerte.

¿Y la reference? La misma disciplina que aprendiste con `findReference`: una clave nueva de un solo uso por pago, generada antes de que el pago exista, para que puedas ubicar la transacción después. Lo que cambia es dónde vive. La billetera ya no está ensamblando nada, así que tu servidor inyecta la reference él mismo, como una cuenta extra agregada a la instrucción de transferencia: de solo lectura, no firmante, marcador puro. Cualquier cuenta listada en una transacción queda indexada por los nodos RPC, que es todo el truco detrás de las reference keys y siempre lo ha sido. Aquí están las dos construcciones, exactamente como van a los TODOs de completion más adelante:

![Las dos construcciones de código para los TODOs de completion: agregar la reference como una cuenta de solo lectura no firmante, y construir una instrucción de memo cuyos datos son la cadena utf8 del pedido.](assets/v04-annotated-code.webp)

Quiero señalar la decisión silenciosa de ese snippet, porque es del tipo que vas a tomar cada semana como ingeniero de pagos. La reference podría haber ido en la instrucción de memo en cambio; la indexación on-chain igual la encontraría. Va en la transferencia porque ahí es donde la propia lógica de validación de `@solana/pay` busca references adyacentes a la transferencia, y coincidir con la convención que tus herramientas esperan le gana a una astucia privada todas las veces. Ya me quemé antes con la elección opuesta, un layout "mejor" que hizo que cada librería aguas abajo me peleara. La convención es una ventaja.

### La frontera de confianza que acabas de crear

Aquí está la contrapartida, dicha sin adornos. Las transacciones construidas por el servidor te dan control total: precios, memos, references, cualquier instrucción que tu código pueda ensamblar. A cambio, la billetera ahora recibe una transacción completamente construida desde un endpoint https y se le pide que la firme. La protección del cliente ya no es "puedo leer los parámetros de la URL". Es la inspección que hace la billetera de lo que volvió. Moviste la frontera de confianza, y algo tiene que vigilar la nueva línea.

El spec la vigila con una sola regla contundente: una solicitud que exige la firma de una cuenta que el usuario no envió es maliciosa, y la billetera debe rechazarla. Siéntate con lo que eso atrapa. Un servidor hostil, o el tuyo comprometido, podría devolver un pago de aspecto perfectamente válido que además exige la firma de alguna otra cuenta: una clave de tesorería que el usuario casualmente controla, un miembro de un multisig, cualquier cosa que el atacante espere que se apruebe sin mirar. La billetera sabe que se ofreció exactamente una cuenta, la que ella misma mandó por POST. Todo otro firmante requerido en la transacción devuelta es una exigencia que nadie aceptó, con una excepción legítima: si el servidor firmó parcialmente la transacción él mismo, esas firmas llegan ya provistas, y la billetera las verifica en vez de que le pidan producirlas.

![Diagrama de confianza de tres zonas: el servidor del comercio en el que solo se confía para construir, la billetera haciendo cumplir que todo firmante requerido sin firma sea la cuenta enviada, y el cliente apoyándose en esa comprobación.](assets/v05-diagram.webp)

¿Cómo comprueba esto una billetera en realidad? Decodificando los bytes del cable antes de firmar, y tú vas a hacer lo mismo en el cliente de humo de esta lección, porque tu cliente de humo está haciendo de billetera. El layout hace el trabajo por ti: un mensaje compilado declara `numSignerAccounts` en su header, y su lista de cuentas estáticas está ordenada con los firmantes primero. Rebana las primeras `numSignerAccounts` direcciones y tienes en la mano la lista completa de firmantes exigidos; cualquier cosa en ese recorte que no sea tu cuenta enviada, y que no lleve ya una firma de verdad, es motivo de rechazo. Kit trae los decoders (`getTransactionDecoder`, `getCompiledTransactionMessageDecoder`), así que la comprobación son una docena de líneas, y escribirla una vez va a hacer más por tu intuición que cualquier diagrama, que es por lo que el peldaño solo te hace escribirla.

Una nota honesta más ya que estamos: esta regla protege firmas, no criterio. Una billetera que la hace cumplir perfectamente igual va a presentar alegremente una transacción que le paga el monto equivocado al comercio equivocado, y la defensa del cliente ahí es el resumen decodificado de la billetera más tu label y tu icon. La regla es el piso de la seguridad de los transaction requests; el techo está en algún lugar bastante más arriba. Trátala como el mínimo que verificas, y deja que tu back office (el próximo módulo) verifique todo lo demás después de la liquidación.

### A dónde se fue la energía del spec

Una pausa corta de realidad antes del lab, porque el piso debajo de esta lección se movió hace poco y deberías saber para qué lado. Ve a mirar el repositorio canónico de Solana Pay hoy y el README que te recibe no es una librería de checkout por QR. Al 2026-08-21, el repo abre con una CLI de pagos agénticos construida alrededor de x402 y MPP, pagos HTTP entre máquinas, con la librería clásica de checkout siguiendo viva como un subdirectorio. La energía de pagos de la Foundation se movió visiblemente desde "la billetera escanea un QR" hacia flujos de pago programáticos, manejados por el servidor.

![Línea de tiempo desde el spec de Solana Pay de 2022, pasando por la librería de checkout volviéndose un subpaquete, hasta el repo de 2026 abriendo con una CLI de pagos agénticos.](assets/v06-timeline.webp)

Lee ese arco desde donde estás sentado hoy, con una lección de profundidad en transacciones construidas por el servidor. Un agente pagando por HTTP y una billetera contestando un transaction request son la misma jugada: un servidor que pone el precio, construye y devuelve algo para firmar. El QR siempre fue un mecanismo de entrega. Lo que estás construyendo esta lección, el par GET-POST alrededor de un constructor del lado del servidor, es la forma sobre la que el ecosistema está redoblando la apuesta, que es por lo que este endpoint, no la página del QR, es el artefacto que el resto de este curso sigue consumiendo. El módulo 7 se enfrenta al extremo agéntico de ese arco, de frente.

## Lab: construye checkout-txreq

El layout que estás por llenar, y dónde se sienta en el workspace de Wavelength:

![Diagrama del workspace, con transfer-kit y checkout alimentando a checkout-txreq con su catálogo, su constructor, su servidor y su prueba de humo, que el puesto del POS, el blink del drop y el capstone consumen en lecciones posteriores.](assets/v07-diagram.webp)

1. **El catálogo y las reglas de precios.** Crea `src/catalog.ts`. Las interfaces y los datos de la tienda vienen dados; `priceOrder` viene como tu primer TODO de completion, con las reglas que tiene que implementar sentadas justo encima:

   ```typescript
   // checkout-txreq/src/catalog.ts
   // The store's source of truth. Prices are decimal strings, parsed exactly;
   // no float ever touches money in this course.
   import { toBaseUnits, fromBaseUnits } from '../../transfer-kit/src/index';

   export interface CatalogEntry {
     title: string;
     priceUsdc: string; // decimal string, e.g. "22.5"
   }

   export interface OrderLine {
     sku: string;
     quantity: number;
   }

   export interface PricedOrder {
     baseUnits: bigint;   // final total in USDC base units (6 decimals)
     totalUsdc: string;   // display form of the same number
     description: string; // "1x WVL-001, 2x WVL-002"
   }

   export const CATALOG: Record<string, CatalogEntry> = {
     'LP-041': { title: 'Slow Tides, Wavelength pressing 041', priceUsdc: '12.5' }, // last lesson's record, now priced server-side
     'WVL-001': { title: 'Wavelength LP, first press', priceUsdc: '18' },
     'WVL-002': { title: 'Late Static Night, 12-inch', priceUsdc: '22.5' },
     'WVL-045': { title: 'The August pressing, limited', priceUsdc: '30' },
   };

   export function priceOrder(lines: OrderLine[], coupon?: string): PricedOrder {
     // Rule 1: reject an empty cart, an unknown sku, and any quantity outside 1..20.
     // Rule 2: subtotal in base units: toBaseUnits(entry.priceUsdc, 6) * BigInt(quantity),
     //         summed as bigint. fromBaseUnits(total, 6) gives you totalUsdc back.
     // Rule 3: description joins the lines: "1x WVL-001, 2x WVL-002".
     // (coupon is unused for now; it is your solo rung.)
     throw new Error('Your turn: compute the total per the three rules above.');
   }
   ```

   La ruta del import asume que transfer-kit está un directorio más allá, como lo ha estado desde el módulo 2; ajústala a tu layout. Deja `WVL-045` en 30 exacto, una lección posterior vende ese prensado a través de este mismo catálogo.

2. **El constructor.** Crea `src/build-order-transaction.ts`. Este archivo es el núcleo de pago que el resto del curso importa, así que sus dos nombres exportados importan tanto como su comportamiento: `buildOrderTransaction` para quienes lo llaman, `finalizeTransaction` como el tramo final compartido. Todo viene dado salvo las dos inyecciones que ya viste en la sección de teoría:

   ```typescript
   // checkout-txreq/src/build-order-transaction.ts
   // The payment core: price an order server-side, stamp the memo and reference,
   // return a base64 transaction for the submitted account to sign.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstructions,
     compileTransaction,
     createSolanaRpc,
     createTransactionMessage,
     generateKeyPairSigner,
     getBase64EncodedWireTransaction,
     pipe,
     setTransactionMessageFeePayer,
     setTransactionMessageLifetimeUsingBlockhash,
     type Address,
     type Instruction,
   } from '@solana/kit';
   import { getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta } from '../../transfer-kit/src/index';
   import { priceOrder, type OrderLine } from './catalog';

   const USDC_MINT = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU'); // devnet USDC, 6 decimals
   const USDC_DECIMALS = 6;
   const MEMO_PROGRAM = address('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'); // SPL Memo v2
   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';

   const rpc = createSolanaRpc(RPC_URL);

   function merchantAddress(): Address {
     const configured = process.env.MERCHANT_ADDRESS;
     if (!configured) throw new Error('set MERCHANT_ADDRESS to the wallet checkout already pays');
     return address(configured);
   }

   export interface BuildOrderInput {
     account: string;      // base58 pubkey the wallet POSTed; the ONLY signer we may demand
     sku?: string;         // single-line form (a later lesson calls it this way)
     quantity?: number;
     lines?: OrderLine[];  // multi-line cart form
     coupon?: string;      // wired on the solo rung
     orderId?: string;
   }

   export interface BuiltOrder {
     transactionBase64: string;
     reference: Address;
     memo: string;
     totalUsdc: string;
   }

   export async function buildOrderTransaction(input: BuildOrderInput): Promise<BuiltOrder> {
     const lines =
       input.lines ?? (input.sku ? [{ sku: input.sku, quantity: input.quantity ?? 1 }] : []);
     const priced = priceOrder(lines, input.coupon);

     const payer = address(input.account);
     const reference = (await generateKeyPairSigner()).address;
     const orderId = input.orderId ?? `ord-${Date.now().toString(36)}`;
     const memo = `wavelength:${orderId}:${priced.description}`;

     // resolveAta takes the owning token program as its third seed since the
     // roster lesson. Devnet USDC is a classic Token mint, so the program is
     // static here; no per-request detection round trip needed.
     const sourceAta = await resolveAta(payer, USDC_MINT, TOKEN_PROGRAM_ADDRESS);
     const destinationAta = await resolveAta(merchantAddress(), USDC_MINT, TOKEN_PROGRAM_ADDRESS);

     const transferIx = getTransferCheckedInstruction({
       source: sourceAta,
       mint: USDC_MINT,
       destination: destinationAta,
       authority: payer,
       amount: priced.baseUnits,
       decimals: USDC_DECIMALS,
     });

     const transactionBase64 = await finalizeTransaction({
       feePayer: payer,
       transferIx,
       reference,
       memo,
     });

     return { transactionBase64, reference, memo, totalUsdc: priced.totalUsdc };
   }

   export interface FinalizeInput {
     feePayer: Address;
     transferIx: Instruction;
     reference: Address;
     memo: string;
   }

   // The shared tail every surface reuses: inject the reference, stamp the memo,
   // set the lifetime, serialize. Later lessons call this directly.
   export async function finalizeTransaction(input: FinalizeInput): Promise<string> {
     // TODO(completion) 1: rebuild the transfer instruction with ONE extra account
     // appended: { address: input.reference, role: AccountRole.READONLY }.
     // Spread the existing accounts; never mutate the instruction you were given.
     const transferWithReference: Instruction = input.transferIx; // replace me

     // TODO(completion) 2: the memo instruction: programAddress MEMO_PROGRAM,
     // no accounts, data = new TextEncoder().encode(input.memo).
     const memoIx: Instruction = { programAddress: MEMO_PROGRAM, data: new Uint8Array() }; // replace me

     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayer(input.feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions([memoIx, transferWithReference], m),
     );

     return getBase64EncodedWireTransaction(compileTransaction(message));
   }
   ```

   Lee el pipe una vez, de arriba abajo, porque es todo el modelo de transacciones de kit en cinco líneas: un mensaje versión 0 vacío, un fee payer, un lifetime, instrucciones, después compilar y serializar. `generateKeyPairSigner` acuña la reference; te quedas solo con su dirección, la clave privada nunca se usa y nunca se guarda. Fíjate también en lo que `buildOrderTransaction` se niega a aceptar: no existe ningún campo de monto en su input. No hay forma de que un llamador, o un cliente, le pase un precio a esta función.

3. **El servidor.** Crea `src/server.ts`. Completamente resuelto; este es el par GET y POST de la sección de teoría hecho literal:

   ```typescript
   // checkout-txreq/src/server.ts
   // The transaction-request endpoint: GET answers with display metadata,
   // POST {account} answers with a base64 transaction built server-side.
   import express from 'express';
   import type { NextFunction, Request, Response } from 'express';
   import { buildOrderTransaction } from './build-order-transaction';
   import type { OrderLine } from './catalog';

   const app = express();

   // Cross-origin access, before any other middleware. Next lesson the
   // point-of-sale page -- served from https://localhost:3001 -- POSTs to
   // this endpoint itself, and a browser preflights that cross-origin JSON
   // POST with an OPTIONS request. Without these headers the preflight
   // fails and the real POST never leaves the page. A wallet scanning a QR
   // is not a browser and never preflights; the POS page is, and does.
   app.use((req: Request, res: Response, next: NextFunction) => {
     res.setHeader('Access-Control-Allow-Origin', '*');
     res.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS');
     res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
     if (req.method === 'OPTIONS') {
       res.sendStatus(204);
       return;
     }
     next();
   });

   app.use(express.json());
   app.use(express.static('public'));

   const BASE_URL = process.env.BASE_URL ?? 'http://localhost:3100';
   const PORT = Number(process.env.PORT ?? 3100);

   interface Order {
     lines: OrderLine[];
     coupon?: string;
   }

   // The web-cart path: your storefront creates the order BEFORE showing the QR,
   // so the URL only ever carries an opaque order id. In production this map is
   // your database; the demo cart below is what smoke.ts buys.
   const ORDERS = new Map<string, Order>([
     ['demo-cart', { lines: [{ sku: 'WVL-001', quantity: 1 }, { sku: 'WVL-002', quantity: 2 }] }],
   ]);

   app.get('/txreq', (_req: Request, res: Response) => {
     res.json({
       label: 'Wavelength Records',
       icon: `${BASE_URL}/icon.png`,
     });
   });

   app.post('/txreq', async (req: Request, res: Response) => {
     const account: unknown = (req.body as { account?: unknown } | undefined)?.account;
     if (typeof account !== 'string' || account.length === 0) {
       res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
       return;
     }

     const orderId = typeof req.query.order === 'string' ? req.query.order : 'demo-cart';
     const order = ORDERS.get(orderId);
     if (!order) {
       res.status(404).json({ message: `unknown order: ${orderId}` });
       return;
     }

     try {
       const built = await buildOrderTransaction({
         account,
         lines: order.lines,
         coupon: order.coupon,
         orderId,
       });
       console.log(`[txreq] order ${orderId}: ${built.totalUsdc} USDC, ref ${built.reference}`);
       res.json({
         transaction: built.transactionBase64,
         message: `Wavelength Records: ${built.totalUsdc} USDC`,
       });
     } catch (err) {
       res.status(400).json({
         message: err instanceof Error ? err.message : 'could not build the order transaction',
       });
     }
   });

   app.listen(PORT, () => {
     console.log(`checkout-txreq listening on :${PORT}`);
   });
   ```

   Tira cualquier PNG cuadrado adentro de `public/icon.png` (cualquier arte de relleno sirve; las billeteras solo necesitan que la URL resuelva) para que la URL de icon del GET funcione. El endpoint loguea la reference en cada build; mantén ese hábito, es la clave de unión sobre la que viven tanto tu watcher como tu back office.

   El bloque de CORS de arriba merece su propia oración, porque su ausencia sería invisible hoy y fatal la próxima lección. Cada consumidor que este servidor ha conocido hasta ahora —curl, smoke.ts, una billetera resolviendo un QR— le habla sin ningún navegador en el medio, así que nada hace cumplir CORS y el servidor parecería funcionar sin esos headers. La lección del punto de venta cambia el cliente: la *página* del POS busca la transacción ella misma, cross-origin, y un navegador se niega a mandar ese POST salvo que el preflight OPTIONS vuelva estampado con `Access-Control-Allow-Origin`. El middleware contesta el preflight con un 204 vacío y estampa cada respuesta. Guárdate el patrón; la lección de los blinks vuelve la misma regla estructural para todo un canal de distribución.

4. **Córrelo.** En una terminal, con tu billetera de comercio del módulo 2:

   ```bash
   MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
   ```

   Checkpoint: `checkout-txreq listening on :3100`, y `curl http://localhost:3100/txreq` devuelve el JSON de tu label e icon. Fíjate que esto corre en http pelado, lo que está bien para el cliente local de hoy; las billeteras de verdad exigen https para los links de transaction request, y la lección del punto de venta pone este mismo servidor detrás de un proxy SSL local en vez de enseñar certificados dos veces.

5. **El cliente de humo.** Crea `smoke.ts` en la raíz del proyecto. Hace de billetera: GET, POST, después decodifica la respuesta y verifica que tu trabajo de verdad aterrizó adentro de los bytes:

   ```typescript
   // checkout-txreq/smoke.ts
   // Plays the wallet: GET the metadata, POST {account}, then decode what came
   // back and prove the memo and reference made it into the transaction.
   import {
     generateKeyPairSigner,
     getBase64Encoder,
     getCompiledTransactionMessageDecoder,
     getTransactionDecoder,
   } from '@solana/kit';

   const TXREQ_URL = process.env.TXREQ_URL ?? 'http://localhost:3100/txreq';
   const MEMO_PROGRAM = 'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr';

   function fail(msg: string): never {
     console.error(`SMOKE FAIL: ${msg}`);
     process.exit(1);
   }

   async function main(): Promise<void> {
     const customer = (await generateKeyPairSigner()).address;

     // 1. GET: the display step
     const get = await fetch(TXREQ_URL);
     if (get.status !== 200) fail(`GET returned ${get.status}`);
     const meta = (await get.json()) as { label?: unknown; icon?: unknown };
     if (typeof meta.label !== 'string' || typeof meta.icon !== 'string') {
       fail('GET must return { label, icon }');
     }

     // 2. POST {account}: the build step
     const post = await fetch(TXREQ_URL, {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ account: customer }),
     });
     if (post.status !== 200) fail(`POST returned ${post.status}: ${await post.text()}`);
     const body = (await post.json()) as { transaction?: unknown; message?: unknown };
     if (typeof body.transaction !== 'string' || body.transaction.length === 0) {
       fail('POST must return a base64 transaction');
     }

     // 3. Decode the wire bytes and check your work landed inside them
     const wireBytes = getBase64Encoder().encode(body.transaction);
     const tx = getTransactionDecoder().decode(wireBytes);
     const message = getCompiledTransactionMessageDecoder().decode(tx.messageBytes);
     if (message.version !== 0) {
       // Solana has two envelope shapes, legacy and version 0. Kit decodes
       // both, but our server builds version 0 and that is what this smoke
       // asserts came back.
       fail('server did not return a version 0 transaction envelope');
     }

     const memoPresent = message.instructions.some(
       (ix) => message.staticAccounts[ix.programAddressIndex] === MEMO_PROGRAM,
     );
     if (!memoPresent) fail('no memo instruction in the built transaction: the finalize TODOs are still open');

     const transfer = message.instructions.find(
       (ix) => message.staticAccounts[ix.programAddressIndex] !== MEMO_PROGRAM,
     );
     if (!transfer || (transfer.accountIndices ?? []).length !== 5) {
       fail('transfer instruction has no injected reference account (expected 5: source, mint, destination, authority, reference)');
     }

     const lastIx = message.instructions[message.instructions.length - 1];
     if (message.staticAccounts[lastIx.programAddressIndex] === MEMO_PROGRAM) {
       fail('memo is the last instruction: validateTransfer pops the last instruction and expects the transfer');
     }

     console.log(
       `GET returned label/icon; POST {account} returned a base64 transaction for the cart total (${String(body.message ?? '')})`,
     );
   }

   main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
   ```

   Corre `npx tsx smoke.ts` en una segunda terminal. Con los TODOs todavía abiertos falla, primero en el `priceOrder` vacío, después en la instrucción de transferencia a la que le falta su cuenta de reference inyectada. (El `memoIx` de relleno ya nombra la dirección de programa correcta, así que la comprobación del memo solo se dispara si borras el relleno de plano; el conteo de cinco cuentas es lo que atrapa el TODO abierto.) Esa secuencia de fallas es la lista de pendientes que te deja el peldaño de completion, en el orden correcto. El bloque de decodificación vale una lectura lenta incluso antes de que arregles nada: es un tercio de la guarda que mantiene segura a la billetera y que vas a terminar solo, y `numSignerAccounts` más el orden de firmantes-primero es todo el conocimiento extra que ese paso necesita.

6. **Apunta hacia él la página que ya tienes.** Tu página de checkout de la lección pasada todavía codifica un transfer request. Dos ediciones chicas allá en el proyecto de checkout (donde `@solana/pay` 1.0.26 ya vive) la cambian a la forma de link. Primero, en `checkout/server.ts`, estampa un id de pedido en vez de una URL terminada: cambia el div del QR a `<div id="qr" data-order="demo-cart"></div>`. Tres cosas de arriba pueden irse con él, y deberían: la construcción de `url`, la llamada a `newReference()` y el bloque `awaitSale(...)` con su log `checkout open ref=`. Las tres le pertenecían al flujo de transfer request. Bajo un transaction request la reference la acuña del lado del servidor `buildOrderTransaction` en el *otro* workspace, así que una reference que acuñe esta página no llega a ninguna billetera ni a ninguna transacción, y un watcher armado sobre ella no puede resolver nunca —una suscripción WebSocket viva por carga de página, esperando para siempre, más una línea de log que nombra una reference que ninguna venta va a llevar jamás. La vigilancia se muda con la reference: ahora es trabajo de `checkout-txreq`. Después reemplaza el cuerpo de `checkout/page.ts`:

   ```typescript
   // wavelength-checkout/checkout/page.ts: the transaction-request swap
   import { encodeURL, createQR } from '@solana/pay';

   // The server now stamps only an opaque order id; this file builds the
   // solana:<https-link> URL and turns it into pixels.
   const mount = document.getElementById('qr')!;
   const link = new URL(`http://localhost:3100/txreq?order=${mount.dataset.order}`);
   const url = encodeURL({ link });
   const qr = createQR(url.toString(), 360, 'white', 'black');
   qr.append(mount);
   ```

   Vuelve a empaquetarlo (`npx esbuild checkout/page.ts --bundle --format=esm --outfile=checkout/public/page.js`), después recarga la página con los dos servidores arriba: la página de checkout en :3010, este endpoint en :3100. El mismo `encodeURL`, distinto campo: pásale `link` en vez de `recipient` y la librería emite `solana:<https-link>` en vez de `solana:<recipient>`. Ese cambio de un solo campo es toda la migración del lado del cliente, lo que te dice a dónde se mudó todo el trabajo de verdad. Checkpoint: la página renderiza un QR cuyo texto decodificado empieza con `solana:http`. Sé honesto contigo mismo sobre qué va a hacer un teléfono con él, eso sí: las billeteras exigen https para los links de transaction request, así que una billetera estricta va a rechazar este QR de http pelado. Hoy tu cliente de humo es la billetera; la próxima lección un proxy SSL se pone delante del puerto 3100 y este mismo QR se vuelve escaneable en un puesto de mercado. La página queda cableada ahora para que esa lección solo tenga que agregar el proxy.

## Challenge

**Completion.** Llena los dos sitios TODO: `priceOrder` en `src/catalog.ts` según sus tres reglas, y las dos construcciones en `finalizeTransaction` según las formas anotadas de la sección de teoría. Aceptación: `npx tsx smoke.ts` imprime `GET returned label/icon; POST {account} returned a base64 transaction for the cart total` con el total del carrito demo en el message. Haz la aritmética tú mismo antes de confiar en la salida: 1 a 18 más 2 a 22.5 es 63 USDC, y si tu servidor dice cualquier otra cosa, tu aritmética de unidades base tiene un float adentro en algún lado. Mi primera corrida imprimió exactamente `Wavelength Records: 63 USDC`, y la reference logueada al lado es lo que el watcher de la lección pasada emparejaría una vez que esta venta liquide.

**Solo, en tres partes, sin guía paso a paso.** Primero, el camino del cupón: agrega una tabla `COUPONS` (`CRATEDIG10` con 10 por ciento de descuento es el código demo), extiende `priceOrder` para aplicarlo a un carrito de varios artículos en aritmética de bigint (redondea el descuento hacia abajo; la deriva de redondeo a favor de la tienda es un ticket de reembolso esperando a pasar), y agrega a `ORDERS` un pedido que lleve el cupón. Segundo, la guarda: escribe `assertNoForeignSignatureDemand(transactionBase64, submittedAccount)` en tu cliente de humo usando los decoders que ya importaste. Rebana las primeras `numSignerAccounts` entradas de `staticAccounts`; cada dirección en ese recorte tiene que ser o bien la cuenta enviada o bien llevar ya una firma distinta de cero en `tx.signatures`, y cualquier otra cosa lanza. Después demuestra que la guarda se dispara: construye localmente un fixture con forma maliciosa llamando a `finalizeTransaction` con una instrucción trucada que liste una segunda cuenta como `WRITABLE_SIGNER`, y afirma que tu guarda lo rechaza. Tercero, liquida una: haz POST como tu cliente de mentira (`/tmp/customer.json`, la billetera que la lección pasada abasteció) en vez de como el comercio, firma con esa clave (cárgala con `createKeyPairSignerFromBytes`, firma con `signTransaction`, manda el base64 por `rpc.sendTransaction`), y míralo aterrizar en devnet. Dos razones por las que tiene que ser el cliente y no tú: la transferencia que construye este endpoint corre desde la cuenta de token de la cuenta que hizo POST hasta la tuya, así que postear tu propia dirección construye una transferencia de comercio a comercio que no demuestra nada, y la billetera de origen tiene que tener de verdad el total del carrito, que son 63 USDC en el carrito demo. Si tu billetera de cliente está corta, o le recargas desde el comercio con el `pay` de transfer-kit (y le pides otra gota al faucet si el comercio también está corto) o liquidas un pedido más chico que hayas agregado tú mismo a `ORDERS` —la guarda y la aritmética del cupón son lo que este peldaño está juzgando, no el tamaño del ticket.

Aceptación, directo de la barrera de esta lección: el POST devuelve una transacción en base64 que se envía en devnet por el total exacto calculado por el servidor, con el cupón aplicado, y tu guarda rechaza la solicitud con forma maliciosa mientras deja pasar la honesta. Si la guarda llega a dejar pasar el fixture alguna vez, revisa si recortaste los firmantes desde el frente de `staticAccounts`; recortar desde cualquier otro lado es el error clásico, porque el layout pone a los firmantes primero y nada te avisa si ignoras eso.

Cuando pasen las dos mitades, fíjate en lo que tienes en la mano: una tienda cuyos precios no se pueden manipular desde una URL, y un cliente que se niega a firmar por cuentas que nadie ofreció. Ese par, control del servidor más escepticismo de la billetera, es todo el modelo de confianza de los transaction requests, y construiste los dos lados. Vale una pausa para el café.

Antes de que cierres la terminal: las lecciones posteriores asumen que este endpoint entró limpio, así que si algún paso te peleó —el orden de los TODOs, el recorte de firmantes-primero, el redondeo del cupón— repórtalo a la comunidad del curso mientras está fresco. Y si tu guarda atrapó el fixture en la primera corrida, tómate la victoria en voz alta; acabas de escribir la misma comprobación que entregan los equipos de billeteras.

Tu endpoint solo se ha manejado desde una pestaña del navegador y un script de humo, eso sí. Este endpoint está a punto de dejar el navegador por completo y golpear una mesa plegable en una feria de discos de fin de semana. La próxima lección le apuntas un punto de venta de verdad, y te encuentras con la realidad del hardware móvil a la hora de cobrar pagos de Solana en persona.
