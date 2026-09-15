# Pagos diferidos: nonces durables en la feria (y el apagón que los dejó marcados)

## Resumen

La lección pasada patrocinaste comisiones: un paymaster de Kora co-firmó el checkout para que un comprador con USDC y cero SOL todavía pudiera irse con el prensado. Esta lección quita la otra suposición que hace calladamente cada checkout que has construido hasta ahora, que es una red viva que está arriba y alcanzable en el momento exacto en que un comprador dice sí. La feria de discos está en un sótano. No hay señal. Igual tomas una venta.

Esto es lo que establece hoy, por adelantado:

- Una transacción normal lleva un plazo de reloj de pared en el que nunca has tenido que pensar: su blockhash es válido por 150 bloques (`MAX_PROCESSING_AGE = 150`), que al tiempo de slot objetivo actual de 300ms sale a 45 segundos. No "alrededor de un minuto". Cuarenta y cinco segundos, y encogiéndose cada vez que baja el tiempo de slot.
- Un nonce durable reemplaza el blockhash con un valor guardado en una cuenta on-chain que no se mueve hasta que tú lo avanzas. Firma en el sótano al mediodía, envía desde la banqueta a las seis, la transacción sigue siendo válida.
- La primitiva tiene dos cicatrices, y las dos son estructurales: los docs oficiales advierten que puede quedar deprecada, y un bug de doble procesamiento de nonce durable detuvo mainnet de Solana por unas 4.5 horas el 2022-06-01. Vas a construir la fila alrededor de las dos.
- El artefacto es `fair-queue`, un nuevo workspace hermano al lado de pos-stall (el puesto mismo no se edita hoy): ventas firmadas offline contra un pool de nonces, drenadas cuando vuelve la conectividad, con un paso de drain que se niega, estructuralmente, a retransmitir un nonce gastado.

Primero, pon el workspace en pie. `fair-queue` se sienta al lado de `checkout-txreq` y `pos-stall` en tu workspace de Wavelength:

```bash
cd ~/wavelength   # the workspace root; last lesson left you inside gasless-checkout
mkdir fair-queue && cd fair-queue
npm init -y && npm pkg set type=module
npm install @solana/kit@6.10.0 @solana-program/system@0.12.2
npm install -D tsx typescript @types/node
```

Notas de pins, comprobadas el 2026-08-31: esto se queda en el workspace de kit v6 del curso, y `@solana-program/system` 0.12.2 es el último release que hace peer con `@solana/kit ^6.x`. El release 0.13.0 (2026-07-15) saltó a kit ^7 y el 0.14.0 a kit ^8, así que `npm install @solana-program/system` sin versión te entregaría un error de peer-dependency contra este workspace. Fíjalo.

## Un cheque sin fecha

### El plazo con el que has estado viviendo

Cada transacción que este curso ha construido hasta ahora, el checkout, el puesto, el crank de las suscripciones, la compra patrocinada, llevaba un `recentBlockhash`. Lo has tratado como un sello de frescura y has seguido adelante, y eso era correcto: para un checkout online el plazo nunca muerde. Hoy muerde, así que vamos a derivarlo de verdad.

La regla: el blockhash de una transacción no puede ser más viejo que 150 bloques, una constante que el código del validador nombra `MAX_PROCESSING_AGE`. La profundidad de la fila está fijada en bloques, no en segundos, así que la ventana de reloj de pared es `150 x slot time`. En los viejos slots de 400ms eso eran alrededor de 60 segundos; en la etapa de 350ms de SIMD-0525 (en vigor el 2026-08-21) eran ~53. El objetivo de hoy es 300ms, con la siguiente etapa habiendo entrado en vigor en la epoch 1024 el 2026-08-28, así que la ventana es 150 x 0.30 = 45 segundos. Dos cortes más por etapas, 250ms y después 200ms, ya están detrás de feature gates en el código y los dos entraron en vigor en devnet, que va una escalera entera adelante de mainnet: en mainnet las cuentas de feature de 250ms y 200ms todavía no existen. Así que la ventana de mainnet se encoge otra vez el día en que cada una se active ahí, y 200ms es el fondo de la escalera de SIMD-0525. Por eso este curso no deja de decir derívalo, nunca lo memorices — y, como la sonda de abajo está a punto de mostrar, por eso lo derivas de tu cluster y no de la escalera.

No me creas la aritmética. Crea `probe.ts` y mira morir un blockhash en tiempo real:

```typescript
import { createSolanaRpc } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

const {
  value: { blockhash, lastValidBlockHeight },
} = await rpc.getLatestBlockhash().send();
const start = Date.now();
console.log(`got ${blockhash}, valid until block height ${lastValidBlockHeight}`);

let height = await rpc.getBlockHeight().send();
while (height <= lastValidBlockHeight) {
  await new Promise((r) => setTimeout(r, 5000));
  height = await rpc.getBlockHeight().send();
}
console.log(`expired after ~${Math.round((Date.now() - start) / 1000)}s`);
```

Córrelo con `npx tsx probe.ts`. El mío imprimió 28 segundos, más o menos la granularidad de 5 segundos del polling, y esa sorpresa es la lección misma: la sonda corre contra devnet, que va adelante de mainnet en tiempo de slot y por lo tanto quema su ventana de 150 bloques más rápido — medido el 2026-09-07, devnet estaba produciendo un slot aproximadamente cada 165ms contra los ~320ms de mainnet, en `solana-core` 4.3.0-beta.3. Lee 165ms dos veces, porque está por debajo de 200ms, el fondo de la escalera de arriba, y no es un typo. Devnet tiene activados los cuatro feature gates, así que su *objetivo* es 200ms; estaba corriendo más rápido que su propio objetivo. Mainnet, en la etapa de 300ms, midió ~320ms — más lento que su propio objetivo. Ninguno de los dos clusters se sienta en su número declarado, y SIMD-0525 lo dice él mismo: la tabla fija el trabajo por slot, los hashes por tick y los límites de bloque, y la SIMD anota que ninguno de sus valores de tiempo "are explicitly in protocol and may deviate from reality." La etapa es a lo que la red apunta, no un reloj al que se la sostiene. Que es el argumento entero a favor de la sonda: `getRecentPerformanceSamples` devuelve `numSlots` y `samplePeriodSecs`, y dividir uno entre el otro es el tiempo de slot actual de tu cluster, con escalera o sin escalera. No tomes ninguno de los dos números de esta página. Diga lo que diga, la ventana de devnet es más apretada que los ~45 segundos de mainnet, y las dos siguen encogiéndose. Diga lo que diga tu sonda, ese número es el planteo entero del problema: en el sótano, la brecha entre "el comprador firmó" y "vuelves a tener barras" se mide en horas, y la paciencia de la transacción se mide en segundos. Ningún loop de reintentos arregla esto, porque retransmitir una transacción vencida no le extiende la vida; el hash simplemente es demasiado viejo, y el RPC te lo va a seguir diciendo no importa con qué amabilidad preguntes otra vez.

![Dos líneas de tiempo horizontales: una transacción con blockhash muere a los 45 segundos aproximadamente mientras una transacción con nonce durable sigue siendo válida por horas hasta que su nonce se avanza.](assets/v01-timeline.png)

Si vienes de los pagos con tarjeta, ya has visto este problema resuelto antes. Las terminales store-and-forward han tomado tarjetas en aviones y en sótanos por décadas: la terminal registra la autorización offline y reenvía el lote cuando se reconecta, y el adquirente resuelve el riesgo después. La razón de que Solana necesite una primitiva dedicada para la misma jugada es que no hay adquirente que absorba la ambigüedad. La validación es global y mecánica, cada nodo tiene que estar de acuerdo sobre si una transacción está fresca, y la regla de frescura es esa ventana de 150 bloques. Así que la versión nativa de la blockchain del store-and-forward no puede solo guardar bytes y esperar; tiene que cambiar lo que "fresco" quiere decir para esa transacción.

¿La bala de plata? Una transacción cuyo sello de frescura controlas tú. Eso es exactamente lo que es un nonce durable: en vez de apuntar a un bloque reciente, la transacción apunta a un valor guardado en una cuenta, y ese valor solo cambia cuando su autoridad lo dice. Los docs oficiales lo ponen en una oración: las transacciones con nonce durable reemplazan el blockhash reciente con un valor de nonce guardado, quitando la ventana de vencimiento de 150 slots, lo que habilita la firma offline y el envío diferido. Un cheque sin fecha encima.

### La cuenta de nonce, campo por campo

El valor guardado vive en una cuenta de nonce: una cuenta propiedad del System Program, 80 bytes de datos, que tiene que estar exenta de rent (alrededor de 0.00106 SOL a ese tamaño en devnet el 2026-09-07, y bajando; tu código le pregunta al RPC en vez de hardcodearlo, que es la razón entera de que esa cifra sea segura de imprimir aquí). Trae una con el cliente generado y te devuelve el estado completo, tipado:

```typescript
import { fetchNonce } from '@solana-program/system';

const { data } = await fetchNonce(rpc, nonceAccountAddress);
// data.version               -> account format version
// data.state                 -> must be Initialized to back a transaction
// data.authority             -> the pubkey allowed to advance, withdraw, reauthorize
// data.blockhash             -> THE nonce value: a hash derived from a recent blockhash
// data.lamportsPerSignature  -> the fee rate captured when the nonce last advanced
```

Dos campos hacen el trabajo. `authority` es la autoridad del nonce, la cuenta que tiene que firmar cualquier instrucción que mueva este nonce; para la fila de la feria esa es la llave del comercio, el mismo firmante que corre checkout-txreq. Y `blockhash` es el valor del nonce mismo. El nombre no es un accidente: el valor es un hash derivado de un blockhash real en el momento del último advance, y va al campo `recentBlockhash` de la transacción como los mismos 32 bytes, así que el formato de cable nunca cambia. Lo que cambia es cómo el runtime lo valida. El campo restante, `lamportsPerSignature`, registra la tasa de comisión capturada cuando el nonce avanzó por última vez, un resabio del rol de la cuenta en la contabilidad de comisiones que vas a leer y nunca tocar.

El costo, dado que un comercio siempre debería conocerlo. El depósito de rent para 80 bytes era de 1,056,640 lamports en devnet y 1,317,264 en mainnet cuando lo leí el 2026-09-07, y los dos siguen bajando a medida que SIMD-0437 escalona la tasa por byte hacia abajo — así que lee el tuyo de `getMinimumBalanceForRentExemption(80)` y no de esta oración. Es un depósito, no una comisión: `WithdrawNonceAccount` le devuelve cada lamport a la autoridad el día en que retires un slot, así que un pool de cuatro slots amarra alrededor de cuatro veces ese número por todo el tiempo que corras el puesto y no te cuesta nada deshacerlo. Por transacción, el camino del nonce durable es un poco más pesado que uno con blockhash, dado que cada venta lleva la instrucción extra de advance y sus cuentas. Para un flujo de pagos ese canje es invisible; la matemática de la comisión base que hiciste en el módulo 1 sigue dominando.

![Una cuenta de nonce de 80 bytes desplegada campo por campo: version, state, la pubkey de authority de 32 bytes mapeada a la llave del comercio, el valor de nonce guardado de 32 bytes, y la tasa de comisión.](assets/v02-diagram.png)

### Instrucción 0 o nada

¿Cómo sabe un validador que una transacción está usando un nonce durable y no un blockhash rancio? Mira exactamente un lugar: la instrucción 0. Si, y solo si, la primera instrucción de la transacción es el `AdvanceNonceAccount` del System Program, con la cuenta de nonce como la primera cuenta de esa instrucción y escribible, el runtime cambia de modo de validación. Carga la cuenta de nonce, comprueba que parsea como `Initialized`, y comprueba que el valor guardado coincide con el campo `recentBlockhash` de la transacción. Coincide, y la transacción sigue adelante; la instrucción de advance entonces rueda el valor guardado a un hash nuevo, que es lo que hace a este nonce de un solo uso. Pon el advance en cualquier lugar que no sea el primero y no tienes una transacción con nonce durable para nada, solo una normal que lleva un blockhash que la fila habrá vencido hace mucho.

La instrucción misma es minúscula: tres cuentas (la cuenta de nonce, el sysvar de recent-blockhashes, la autoridad como firmante) y cuatro bytes de datos, `[4, 0, 0, 0]`, el discriminador del System Program para AdvanceNonceAccount. Esos cuatro bytes vale memorizarlos porque tu lab hace asserts sobre ellos.

![La instrucción AdvanceNonceAccount anotada: System Program, la cuenta de nonce escribible, el sysvar de recent-blockhashes, la autoridad como firmante, y los bytes de datos 4,0,0,0, válida solo en la posición cero de instrucción.](assets/v03-annotated-code.png)

En kit nunca construyes a mano esa instrucción para tus propias transacciones, porque el helper de lifetime lo hace por ti. Donde el builder de la lección pasada llamaba a `setTransactionMessageLifetimeUsingBlockhash`, la fila de la feria llama a su hermano:

```typescript
import { setTransactionMessageLifetimeUsingDurableNonce, type Nonce } from '@solana/kit';

const message = setTransactionMessageLifetimeUsingDurableNonce(
  {
    nonce: nonceValue as Nonce,          // data.blockhash from fetchNonce, cached
    nonceAccountAddress,                  // the pool account backing this sale
    nonceAuthorityAddress,                // the merchant key
  },
  transactionMessage,
);
```

Una sola llamada hace dos trabajos: pone la restricción de lifetime del mensaje en el valor del nonce, y antepone la instrucción AdvanceNonceAccount para que aterrice en el índice 0. Crédito donde toca, este es el tipo de eliminación de trampas que kit hace bien; en los días del web3.js legado, olvidarse de poner el advance primero era una falla silenciosa clásica. Un requisito sigue siendo tuyo, eso sí: la autoridad del nonce tiene que firmar de verdad. En fair-queue el comercio es a la vez fee payer y autoridad del nonce, así que el único firmante embebido cubre los dos roles y `signTransactionMessageWithSigners` resuelve todo sin una red a la vista. Divide esos roles entre dos llaves y las dos tienen que estar presentes al momento de firmar. Esa división es también la postura de producción: la lección de la barrera de salida al aire va a pedir una llave dedicada de autoridad del nonce, para que una laptop de puesto robada comprometa una fila y no la caja, y la construcción de una sola llave de este lab es una concesión a la testeabilidad que te va a hacer arreglar, no una recomendación.

Fíjate en lo que quiere decir firmar offline aquí. La transacción firmada es un blob de bytes autocontenido. Quien tenga esos bytes puede enviarlos, desde cualquier máquina, en cualquier momento, hasta que el nonce avance. El planteo propio de los docs es directo: no hay restricciones sobre qué tan viejo es el nonce. Tu archivo de fila por lo tanto no es un log; es un cajón lleno de cheques firmados y sin fecha. Cualquiera que tenga uno puede enviarlo — aunque no redirigirlo: el beneficiario está horneado en los bytes firmados, así que un ladrón no gana nada más que el poder de cobrar tu cheque en tu propia caja en un momento que tú no elegiste, y un cheque cobrado a tus espaldas es lo que convierte tu re-firma posterior en un cargo doble. Trata `queue.json` con exactamente la seriedad con la que tratarías ese cajón, porque de aquí en adelante el peligro en esta lección no es el vencimiento sino su opuesto.

### Las dos cicatrices

La primera cicatriz está escrita directamente en la documentación oficial, y quiero que la leas como está escrita en vez de mi paráfrasis: "Durable nonces may be deprecated in a future release," con un puntero a una discusión SIMD abierta (docs de solana.com, vueltos a obtener el 2026-08-31). Esa discusión ahora tiene una propuesta numerada detrás: SIMD-0571, "Soft Deprecation of Durable Nonce Transactions", un pull request abierto y sin mergear en el repo de SIMD al 2026-08-31, que es decir propuesto, disputado y no aceptado. Quédate con lo inusual que es eso, porque la página de docs que enseña la feature abre advirtiéndote que la feature puede desaparecer. Construye la fila de la feria, entrégala, pero arquitecta de modo que el módulo de la fila sea intercambiable: el día en que aterrice la deprecación, quieres reemplazar un directorio, no tu checkout.

La segunda cicatriz es la razón de que exista la primera. El 2022-06-01, un bug en el manejo de nonces durables dejó que ciertas transacciones de nonce se procesaran dos veces. Los validadores no se pusieron de acuerdo sobre el resultado, el consenso se estancó, y mainnet se detuvo por unas 4.5 horas. La respuesta después fue drástica: la feature se desactivó temporalmente en toda la red mientras se arreglaba la lógica del runtime. Lee eso como comercio, no como historiador de protocolos. Procesar dos veces una transacción de pago quiere decir un comprador cobrado dos veces, y la clase de falla no era exótica: transacciones que deberían haber sido irrepetibles se honraron otra vez. Fíjate en el tiempo verbal, eso sí — arreglada. La reparación de 2022 separó los dominios de validación de nonce durable y de blockhash, así que hoy un validador conforme rechaza determinísticamente los bytes cuyo nonce ha avanzado: el valor guardado ya no coincide con el `recentBlockhash` de la transacción, la comprobación exacta que leíste en la sección de la instrucción 0. ¿Así que por qué tu drain sigue tratando un nonce gastado como radioactivo? No porque la retransmisión pueda hacer un cargo doble — el runtime cerró esa puerta. Porque cada retransmisión de bytes muertos es un envío desperdiciado y un borrón en el libro mayor, y porque el cargo doble que sigue vivo te pertenece a ti por completo: RE-FIRMAR la misma venta contra un nonce fresco. La regla del nonce gastado del drain existe para que nadie tome esa decisión a las 6 p.m. con un archivo de fila abierto y sin registro de lo que ya aterrizó.

![Línea de tiempo del apagón de mainnet de Solana del 2022-06-01: un bug de doble procesamiento de nonce durable detiene la red por unas 4.5 horas, el origen de la regla del nonce gastado del drain.](assets/v04-timeline.png)

Una pieza más de honestidad comercial antes del libro mayor. Una venta en fila no es una venta liquidada. La firma en tu cajón demuestra que el comprador autorizó el pago al mediodía; no demuestra que el pago vaya a aterrizar a las seis, porque un envío al momento del drain todavía puede fallar como cualquier otra transacción, más llanamente cuando el saldo del comprador se gastó en otra parte durante la tarde. Los comercios con tarjeta han vivido con exactamente esto desde que existe el store-and-forward, y la postura es la misma: entrega el disco en el puesto si tus márgenes toleran el riesgo, o guarda los artículos de alto valor para recoger después de que el drain confirme. De cualquier modo tu libro mayor necesita dos estados, en fila y aterrizada, y solo el segundo es ingreso. El back office que construiste en el módulo 4 ya tiene la mitad de aterrizada; el archivo de fila es la otra.

Así que este es el libro mayor honesto sobre esta primitiva. Un nonce durable quita el plazo de reloj de pared, y para una fila offline eso es una bendición. Es también exactamente por lo que es peligroso: los bytes firmados son instrumentos al portador hasta que su nonce avanza, un drain que no puede distinguir aterrizada de perdida invita al cargo doble por re-firma, y la feature se entrega con su propia advertencia de deprecación. Para el checkout online que construiste en el módulo 3, nada de este canje vale la pena; un blockhash fresco es más simple, más seguro y se vence solo, y eso sigue siendo el default correcto. Echa mano de los nonces durables solo cuando la firma tenga que pasar lejos de la red, que es precisamente, y solamente, lo que es la feria. Una nota de frontera antes de construir: los emisores también usan nonces durables online, como una política deliberada de aterrizaje de transacciones para reintentos y firmantes lentos. Ese es otro asiento con otra matemática, y los nonces durables como política del emisor son territorio del curso Client-Side Mastery. Aquí nos quedamos en el puesto.

## Lab: la fila de la feria

Tu parte del trabajo, en voz alta: este es el módulo 8, territorio en solitario. Los pasos trabajados de abajo te entregan la infraestructura, el pool de nonces y la captura matutina, completos y corribles, porque la plomería de cuentas no es la lección de esta lección. El firmante offline y el drain están especificados como criterios de aceptación con sus fragmentos estructurales a la vista, y los archivos los armas tú. El clasificador del drain al final es el desafío de código sin guía. Para el capstone, todo es tuyo de todos modos.

El día tiene una forma, y los scripts la siguen:

![Ciclo de cuatro etapas: crea el pool de nonces una vez estando online, captura los valores de los nonces cada mañana, firma ventas offline hacia una fila durante la feria, y drena la fila de forma segura cuando vuelvas a estar online.](assets/v05-flowchart.png)

1. **Llaves y fondeo.** `merchant.json` tiene que ser *la* llave del comercio, la que checkout-txreq le paga, porque las ventas de la fila acreditan esa billetera y el drain concilia contra el mismo libro mayor. Esa llave es la identidad de la CLI que creó el módulo 2, así que cópiala en vez de acuñar una nueva — `solana-keygen new -o merchant.json` te entregaría otra billetera, y nada río abajo te lo diría: las ventas aterrizarían, en la tienda equivocada. El comprador de demo, en cambio, es genuinamente fresco; hace de suplente de la billetera del cliente que firmaría en un puesto real:

   ```bash
   cp ~/.config/solana/id.json merchant.json    # THE merchant key, not a new one
   solana-keygen new -o buyer.json --no-bip39-passphrase
   solana airdrop 2 "$(solana-keygen pubkey merchant.json)" --url devnet
   solana airdrop 2 "$(solana-keygen pubkey buyer.json)" --url devnet
   ```

   Confirma la copia antes de construir sobre ella: `solana-keygen pubkey merchant.json` tiene que imprimir exactamente lo que imprime `solana address`. Una nota hacia adelante, porque la próxima lección audita esto: hoy esa única llave lleva tres sombreros — receptor del pago, fee payer y autoridad del nonce — lo que está bien para un lab y es exactamente lo que la fila de separación de llaves de la barrera de prod te va a pedir dividir.

   Sé claro sobre qué es `buyer.json`: un suplente de demo, y está escondiendo el único problema genuinamente sin resolver de este diseño, así que déjame nombrarlo en vez de agitar la mano. En un puesto real la billetera del comprador tiene que firmar en su propio dispositivo, pero el flujo de QR del módulo 3 no puede entregar esa firma aquí: una transaction request hace que la billetera traiga la transacción de tu servidor por la red, y la premisa entera de esta lección es que no hay red. Llevar un mensaje sin firmar al teléfono del comprador y traer de vuelta los bytes firmados sin señal necesita un transporte local, un viaje de ida y vuelta por QR, NFC o BLE, y construir uno está fuera del alcance de este curso, que es la razón de que el keypair de demo sea el único camino de comprador soportado aquí. Lo que la fila misma necesita sobrevive intacto a esa limitación: no le importa quién produjo la firma del comprador, solo que la llave del comercio tenga los dos roles que importan, fee payer y autoridad del nonce, así que cuando exista una integración de billetera con transporte local, el diseño de la fila no cambia.

   Checkpoint: los dos airdrops confirman, y `solana balance "$(solana-keygen pubkey merchant.json)" --url devnet` y el mismo comando contra `buyer.json` imprimen cada uno 2 SOL.

2. **Crea el pool.** Este es el hecho que le da forma al artefacto entero: un nonce respalda una transacción. El advance que valida la venta A también invalida cualquier otra cosa firmada contra ese mismo valor, así que una fila de N ventas pendientes necesita N cuentas de nonce. Eso es un pool de nonces, y el nuestro es de cuatro slots, cada uno una cuenta de 80 bytes creada e inicializada en una sola transacción de dos instrucciones. Cuatro es una decisión de dimensionamiento, no un número mágico: el pool pone el tope de cuántas ventas puedes sostener entre drains, así que dimensiónalo al tramo offline más largo que esperes por tu tasa de ventas, y acuérdate de que cada slot extra cuesta solo un depósito de rent reembolsable. Una feria en un sótano con un hueco de escalera con señal cada hora o dos hace de cuatro más que suficiente para un demo; un fin de semana de festival querría más. Crea `create-pool.ts`:

   ```typescript
   import { readFileSync, writeFileSync } from 'node:fs';
   import {
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     generateKeyPairSigner,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
   } from '@solana/kit';
   import {
     getCreateAccountInstruction,
     getInitializeNonceAccountInstruction,
     getNonceSize,
     SYSTEM_PROGRAM_ADDRESS,
   } from '@solana-program/system';

   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
   const WS_URL = process.env.WS_URL ?? 'wss://api.devnet.solana.com';
   const POOL_SIZE = 4;

   const rpc = createSolanaRpc(RPC_URL);
   const rpcSubscriptions = createSolanaRpcSubscriptions(WS_URL);
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

   const merchant = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync('merchant.json', 'utf8'))),
   );

   const space = BigInt(getNonceSize());
   const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

   const pool: string[] = [];
   for (let i = 0; i < POOL_SIZE; i++) {
     const nonceAccount = await generateKeyPairSigner();
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (tx) => setTransactionMessageFeePayerSigner(merchant, tx),
       (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
       (tx) =>
         appendTransactionMessageInstructions(
           [
             getCreateAccountInstruction({
               payer: merchant,
               newAccount: nonceAccount,
               lamports: rent,
               space,
               programAddress: SYSTEM_PROGRAM_ADDRESS,
             }),
             getInitializeNonceAccountInstruction({
               nonceAccount: nonceAccount.address,
               nonceAuthority: merchant.address,
             }),
           ],
           tx,
         ),
     );
     const signed = await signTransactionMessageWithSigners(message);
     assertIsTransactionWithBlockhashLifetime(signed);
     await sendAndConfirm(signed, { commitment: 'confirmed' });
     console.log(`nonce ${i}: ${nonceAccount.address} (${getSignatureFromTransaction(signed)})`);
     pool.push(nonceAccount.address);
   }

   writeFileSync('nonce-pool.json', JSON.stringify(pool, null, 2));
   console.log(`pool of ${POOL_SIZE} written to nonce-pool.json`);
   ```

   Corre `npx tsx create-pool.ts`. Checkpoint: cuatro direcciones imprimen con firmas, y `nonce-pool.json` existe. Fíjate en lo que usa esta transacción misma: un lifetime de blockhash pelado. La creación del pool pasa en casa, online, así que no necesita ningún nonce; el punto del pool es lo que pasa después. Fíjate también en `nonceAuthority: merchant.address`, el patrón de autoridad separada: la cuenta de nonce tiene su propio keypair desechable, pero el control le pertenece a la llave del comercio. El keypair de la cuenta firma una vez en la creación y nunca se vuelve a necesitar.

3. **La captura matutina.** Firmar offline requiere conocer cada valor de nonce antes de perder la señal, así que el último acto online de la mañana es cachearlos. Crea `snapshot.ts`:

   ```typescript
   import { readFileSync, writeFileSync, existsSync } from 'node:fs';
   import { address, createSolanaRpc } from '@solana/kit';
   import { fetchNonce } from '@solana-program/system';

   type QueueSlot = { nonceAccount: string; nonceValue: string };

   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
   const rpc = createSolanaRpc(RPC_URL);

   const pool = JSON.parse(readFileSync('nonce-pool.json', 'utf8')) as string[];
   const pending = existsSync('queue.json')
     ? (JSON.parse(readFileSync('queue.json', 'utf8')) as { pending: unknown[] }).pending
     : [];
   const busy = new Set(
     (pending as { nonceAccount: string }[]).map((e) => e.nonceAccount),
   );

   const free: QueueSlot[] = [];
   for (const nonceAccount of pool) {
     if (busy.has(nonceAccount)) continue; // still backing a queued sale
     const { data } = await fetchNonce(rpc, address(nonceAccount));
     free.push({ nonceAccount, nonceValue: data.blockhash });
   }

   writeFileSync('queue.json', JSON.stringify({ free, pending }, null, 2));
   console.log(`snapshot: ${free.length} free nonces cached, ${pending.length} sales still pending`);
   ```

   Checkpoint: `npx tsx snapshot.ts` imprime `4 free nonces cached, 0 sales still pending`, y `queue.json` sostiene cuatro pares `{ nonceAccount, nonceValue }`. Esos valores cacheados son tu inventario del día de cheques sin fecha, en blanco y refrendados.

4. **El firmante offline: tu construcción.** Fuera los scaffolds. Escribe `sign-sale.ts` de modo que `npx tsx sign-sale.ts <lamports> "<label>"` tome una venta enteramente offline. Criterios de aceptación:

   - Carga `merchant.json` y `buyer.json` como firmantes, saca el siguiente slot libre de `queue.json`, y lanza un error con un mensaje claro cuando el pool esté agotado (cuatro ventas pendientes y ningún drain quiere decir que el puesto deja de tomar pedidos; dilo).
   - Construye el mensaje con el comercio como fee payer (el puesto cubre la comisión de red; fíjate en que esta es una postura distinta de la de la lección pasada, donde una llave dedicada de patrocinador Kora pagaba y la llave del comercio se quedaba fuera del camino caliente) y el lifetime de nonce durable a partir del `nonceValue` cacheado del slot, usando la llamada a `setTransactionMessageLifetimeUsingDurableNonce` de la sección de teoría, con el comercio como autoridad del nonce.
   - Agrega un `getTransferSolInstruction({ source: buyer, destination: merchant.address, amount: lamports(saleLamports) })` como la venta misma. Una transferencia de SOL pelada, deliberadamente: el lifetime del nonce es ortogonal a lo que la transacción hace de verdad, así que el payload más simple posible mantiene sin estorbos la única idea nueva, y es por eso que `buyer.json` recibió un airdrop en vez de el USDC que usa cada otro peldaño. Cambia eso por el `TransferChecked` del módulo 2 y no cambia ni una línea de la captura, de la guarda ni del drain.
   - Firma con `signTransactionMessageWithSigners`, y agrega al array `pending` de `queue.json` una entrada con exactamente esta forma, porque el drain, la prueba de humo y después el harness de journey del capstone la leen toda:

   ```typescript
   type QueueEntry = {
     kind: 'durable-nonce';
     nonceAccount: string;   // the pool slot backing this sale
     nonceValue: string;     // the cached nonce it was signed against
     signature: string;      // getSignatureFromTransaction(signed)
     wire: string;           // getBase64EncodedWireTransaction(signed)
     signedAtSeconds: number;
     label: string;
   };
   ```

   Y una guarda que no voy a dejar al azar, pégala verbatim entre construir y firmar. Vuelve a derivar la regla de la sección de teoría contra el mensaje que construiste de verdad:

   ```typescript
   // The runtime only treats this as a nonce transaction if
   // AdvanceNonceAccount sits at instruction 0. Prove it before signing.
   const [ix0] = message.instructions;
   const isAdvance =
     ix0 !== undefined &&
     ix0.programAddress === SYSTEM_PROGRAM_ADDRESS &&
     ix0.data !== undefined &&
     ix0.data.length === 4 &&
     ix0.data[0] === 4;
   if (!isAdvance) throw new Error('instruction 0 is not AdvanceNonceAccount: refusing to sign');
   ```

   Checkpoint, y es el divertido: apaga tu wifi. De verdad apagado. Después `npx tsx sign-sale.ts 1000000 "Kind of Blue, original pressing"`. Entra a la fila. Nada en el camino de la firma toca la red, lo que acabas de demostrar por construcción. Firma una segunda venta mientras estás allá abajo; la fila sostiene las dos, cada una contra su propio slot del pool.

5. **La prueba de 90 segundos y el drain.** Wifi todavía apagado, mira tu reloj: cualquier transacción con blockhash firmada cuando te quedaste a oscuras murió en la marca de los 45 segundos (más pronto en devnet, como mostró tu sonda). Espera hasta que hayan pasado al menos 90 segundos desde tu primera corrida de `sign-sale.ts`, que es la demora que también usa el harness de verify, cómodamente pasada la ventana. Después reconéctate y escribe `drain.ts` con estos criterios:

   - Para cada entrada pendiente, llama a `fetchNonce` sobre el `nonceAccount` de la entrada y compara el `data.blockhash` vivo con el `nonceValue` cacheado de la entrada. Esta comparación es la historia entera de seguridad, así que pasa antes que cualquier otra cosa.
   - Los valores coinciden: el cheque sigue sin cobrarse. Envía los bytes guardados exactamente como se firmaron y confirma sondeando el estado. Dos fragmentos cargan el peso de la API aquí; se apoyan en `type Base64EncodedWireTransaction` y `signature` de `@solana/kit`, más el import de `SYSTEM_PROGRAM_ADDRESS` que usó la guarda del paso 4, así que los tres pertenecen al bloque de imports de drain.ts:

   ```typescript
   const sig = await rpc
     .sendTransaction(entry.wire as Base64EncodedWireTransaction, {
       encoding: 'base64',
       preflightCommitment: 'confirmed',
     })
     .send();
   ```

   ```typescript
   // A nonce tx has no blockhash deadline, so the blockhash-based confirm
   // helper does not apply. Poll the signature status instead.
   // searchTransactionHistory is load-bearing: without it the RPC consults
   // only its short-lived status cache (about a minute of recent slots), so
   // a sale that landed in an earlier drain reads back null here and would
   // be misrouted to UNSAFE -- whose remedy, re-selling against a fresh
   // nonce, is a double-charge. Same guard the refund builder uses.
   const { value: statuses } = await rpc
     .getSignatureStatuses([signature(entry.signature)], { searchTransactionHistory: true })
     .send();
   const s = statuses[0];
   const landed = s !== null && s !== undefined && s.err === null &&
     (s.confirmationStatus === 'confirmed' || s.confirmationStatus === 'finalized');
   ```

   - Los valores difieren: el nonce está GASTADO, y esta rama no puede contener ninguna llamada de envío. Concilia en su lugar: si `getSignatureStatuses` — preguntado con `searchTransactionHistory: true`, exactamente como en el sondeo de arriba, dado que la entrada puede haber aterrizado hace muchos minutos — muestra la firma propia de la entrada confirmada y sin error, un envío anterior ya la aterrizó, regístrala como `RECONCILED` y libera el slot. Si el nonce se movió pero tu firma no está en ninguna parte, regístrala como `UNSAFE` y guarda la venta para una decisión humana (revenderla contra un nonce fresco); de cualquier modo los bytes guardados están muertos. Esta rama es la lección de 2022 como camino de código.
   - Las entradas aterrizadas y conciliadas salen de `pending`; un `snapshot.ts` posterior vuelve a traer los valores nuevos de sus slots y los devuelve a `free`. Eso es reciclaje de nonces: el pool son cuatro cuentas para siempre, no cuatro cuentas por día.

   Checkpoint: `npx tsx drain.ts` imprime `LANDED` con una firma de devnet para cada venta, minutos después de firmar, y el explorador muestra la instrucción 0 de tu transacción como AdvanceNonceAccount con datos `[4, 0, 0, 0]`. Un blockhash nunca podría haber cruzado esa brecha.

6. **El intento de repetición.** Corre `npx tsx drain.ts` una segunda vez sin capturar. Cada entrada que acaba de aterrizar ahora pega en la rama del nonce gastado, imprime `RECONCILED`, y, esta es la aserción que importa, no envía nada. Después cierra el loop con un harness que escribes tú: `verify/fair-queue.smoke.ts`, de cuatro asserts de largo, manejando tus propios scripts de punta a punta. Firma una venta, la sostiene 90 segundos, la drena, decodifica la instrucción 0 para confirmar AdvanceNonceAccount (reusa la guarda del paso 4), y repite el drain esperando cero reenvíos. Verde quiere decir que fair-queue es real.

![Diagrama de flujo de decisión para drenar una entrada: los valores de nonce que coinciden envían y confirman, mientras que un nonce gastado se rutea a conciliación o a un veredicto inseguro que nunca reenvía.](assets/v06-flowchart.png)

## Challenge

El drain que escribiste maneja una fila en una tarde feliz. El desafío en solitario, `nonce-queue-drain`, es la versión general, lógica pura, sin RPC: llega una fila mezclada que sostiene tanto entradas con lifetime de blockhash como entradas con nonce durable (un puesto realista firma online cuando tiene señal y offline cuando no), y clasificas cada entrada antes de que se envíe cualquier cosa.

```typescript
declare function drainFairQueue(
  nowSeconds: number,
  windowSeconds: number, // ~45 today; a parameter because slot time moves
  queueJson: string // the queue, flattened to one JSON string (see below)
): { submit: string[]; expired: string[]; unsafe: string[] };
```

Fíjate en la forma antes de empezar: tres argumentos posicionales de entrada, y tres listas de strings de `id` de salida, no tres listas de entradas. El calificador no le puede entregar a tu función un objeto ni un array literal, así que la fila llega serializada:

- **Formato de cable.** `queueJson` es un solo string JSON que sostiene un array plano con cuatro huecos por entrada, en orden de fila — `[id, signedAtSeconds, kind, nonceAdvanced]` repetido — donde `kind` es `'blockhash'` o `'nonce'` (la entrada de nonce durable de tu fila, aplanada) y `nonceAdvanced` es `0` o `1`. El starter ya le hace `JSON.parse` a ese string y te reconstruye ítems tipados como se debe; el decode es plomería, no la lección, y tu trabajo empieza en el loop de clasificación.
- **Regla uno.** Las entradas con blockhash vencen por reloj de pared: un id aterriza en `submit` solo cuando `nowSeconds - signedAtSeconds <= windowSeconds`, si no aterriza en `expired`.
- **Regla dos.** Las entradas con nonce durable nunca vencen por reloj de pared, pero `nonceAdvanced === true` rutea a `unsafe`, nunca a `submit`, no importa lo joven que sea la entrada.
- **Regla tres.** El orden se preserva dentro de cada grupo, porque la fila de un puesto es también su orden de despacho.
- **La respuesta equivocada clásica.** Aplicarles la ventana de edad a las entradas con nonce, venciendo calladamente cheques que no tienen fecha. Ramifica sobre `kind` antes de tocar cualquier matemática de edad.

El starter y los tests están en el widget de desafío de código de nonce-queue-drain; la solución pasa cada caso, y el starter falla al menos uno, así que sabes que los tests muerden.

![Comparación de tres filas de los grupos del clasificador: submit para las entradas frescas, expired para las entradas con blockhash fuera de ventana, y unsafe para las entradas con nonce gastado que nunca deben reenviarse.](assets/v07-comparison.png)

## Checkpoint, y el cajón

Si el lab te peleó, haz el triage en este orden. Que `fetchNonce` lance cuenta-no-encontrada quiere decir que la transacción de creación del pool no confirmó; vuelve a correr `create-pool.ts` y confía en las firmas impresas por encima de tus suposiciones. Un `sign-sale.ts` que falla dentro de `signTransactionMessageWithSigners` estando offline es casi siempre un desajuste de firmantes: la autoridad del nonce que nombraste en la llamada del lifetime tiene que ser una dirección cuyo firmante esté embebido en el mensaje, y en esta construcción eso quiere decir que el comercio es a la vez fee payer y autoridad. Y si el drain no aterriza nada mientras el explorador muestra tus cuentas de nonce sin tocar, decodifica tú mismo la instrucción 0 de una entrada en fila, como hace la guarda del paso 4: si AdvanceNonceAccount no está sentado en el índice 0, esa entrada se construyó sin la llamada del lifetime de nonce durable, normalmente porque un camino de código cayó calladamente de vuelta a un lifetime de blockhash, y la guarda se está negando a firmar exactamente como se diseñó.

Corrí mi propia fila por el ciclo completo mientras escribía esto: dos ventas firmadas con el wifi apagado, una demora del valor de un café, las dos aterrizaron en el primer drain, y el segundo drain concilió las dos sin enviar un byte. La parte satisfactoria no es el aterrizaje para nada; es ver a la repetición negarse a enviar.

Mira lo que el puesto sobrevive ahora. El módulo 3 le dio un código QR y precios del lado del servidor. La lección pasada quitó la necesidad de que el comprador tuviera SOL. Hoy quitó la necesidad de una red en el momento de la venta, y lo hizo sobre una primitiva que ahora manejas como su historia exige: advance en la instrucción 0, un nonce por venta, un cajón tratado como papel al portador, y un drain que preferiría escalar a un humano antes que cobrar un cheque gastado. El puesto puede tomar dinero sin SOL y sin señal.

Lo que quiere decir que la construcción está hecha y la duda empieza. Antes de que Wavelength dé vuelta el cartel a abierto, queda una barrera: demostrar que todo este stack, del checkout a la fila, está de verdad listo para dinero real. La próxima lección es esa barrera, una checklist de producción que puedes fallar, corrida contra todo lo que has construido. Trae los artefactos; son ellos los que están siendo examinados.
