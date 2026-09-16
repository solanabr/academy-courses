# Conciliación, reembolsos y pagos parciales: el manual que falta

## Resumen

La lección pasada el back office ganó oídos: ingesta webhooks de Helius de forma idempotente, verifica cada evento on-chain antes de creerle, y escribe un libro mayor de pedidos con clave en la firma de la transacción, despachando exactamente una vez. Hoy un comprador pone a prueba lo que todavía no puede hacer. Compró un prensado de una sesión en vivo, el disco llegó alabeado, y quiere su dinero de vuelta. Abres los docs de pagos de Solana buscando el flujo de reembolso y no encuentras nada, porque los chargebacks se diseñaron fuera de los rieles a propósito. La reversión te toca construirla a ti.

Antes de cualquier cosa nueva, dos minutos de mantenimiento que este módulo viene postergando, y después demuestra que la línea base sigue respondiendo. Las carpetas de ops que creaste como hermanas sueltas se suman ahora al workspace `wavelength`, así pueden importarse entre sí por nombre (`verifier`, `transfer-kit`) en vez de por ruta relativa. Importar por nombre pide dos cosas, y el roster solo te da la primera. Registrar las carpetas como workspaces hace que npm cree un symlink de cada paquete dentro del `node_modules` raíz bajo su nombre de paquete (`npm init -y` nombró cada uno según su carpeta). Pero cuando un archivo dice después `from 'verifier'`, Node sigue ese symlink hasta el `package.json` del paquete y abre lo que sea que nombre `main`, y `npm init -y` escribió `"main": "index.js"`, un archivo que ninguno de los dos paquetes tiene. En vez de eso, apunta `main` a la puerta de entrada real de cada paquete. `transfer-kit` tiene un archivo barrel en `src/index.ts` desde el módulo 2; el verificador nunca tuvo uno, porque hasta hoy cada consumidor importaba sus archivos por ruta relativa. Dale la misma puerta de entrada:

```ts
// verifier/src/index.ts
export { createVerifier } from './verify.ts';
export { createMemoryStore } from './store.ts';
export { createRpcFetchTransaction } from './rpc.ts';
export type { ExpectedOrder, VerifyResult, RejectReason } from './types.ts';
```

Después, desde la raíz de `wavelength`, registra el roster y apunta los dos campos `main` (tsx, que corre cada script de este curso, resuelve una entrada TypeScript directamente):

```bash
cd ~/wavelength
npm pkg set --json workspaces='["transfer-kit","verifier","backoffice","backoffice-refunds"]'
npm pkg set type="module"
npm pkg set main="src/index.ts" --workspace transfer-kit --workspace verifier
npm install
npm run --workspace backoffice verify:backoffice
```

Deberías ver el webhook entregado por triplicado colapsar a exactamente una fila del libro mayor y el evento falsificado rebotar contra el verificador. Ese libro mayor, con clave en la firma, es el sustrato sobre el que se construye todo lo de hoy. Si la comprobación de humo falla, arregla primero la lección pasada; un conciliador sobre un libro mayor roto solo automatiza la confusión.

Con la línea base respondiendo, de entrada, lo que hoy establece:

- Entregas **backoffice-refunds**: un conciliador que empareja pagos con pedidos por la reference key (la mitad del memo se entrega como un TODO cableado que completa el tráfico del módulo 7), y un constructor de reembolsos que empuja stablecoins de vuelta a la billetera de origen a través de transfer-kit, registrando cada reembolso en el libro mayor enlazado a su firma de origen.
- La conciliación converge. Una reference key de Solana Pay y un id de factura en el `extra.memo` de x402 son la misma idea, un id de pedido llevado en un campo buscable, sin necesidad de una dirección de depósito única. Un solo conciliador sirve a los dos rieles. Un barrido de tesorería atrapa el dinero que no empareja con nada.
- Los reembolsos son construcción original, no cita de documentación. Seguimos el único precedente que existe en la industria: Stripe devuelve los reembolsos cripto como stablecoins a la billetera de origen, y nosotros también.
- El pago de más, el pago de menos y los pagos parciales reciben políticas con nombre, nunca valores por defecto silenciosos. Cada elección aquí es una contrapartida declarada.
- Una regla le gana a todo lo demás: un reembolso nunca sale de tu billetera hasta que el pago de origen haya alcanzado finality. Un reembolso push es irreversible en el instante en que aterriza.

Cómo se reparte el trabajo en este módulo: este es el tramo que se inclina hacia completion. El conciliador y el barrido llegan como esqueletos con TODOs marcados, igual que el registro de claims y la escritura del libro mayor de la lección pasada, y el constructor de reembolsos llega trabajado, sobre la teoría de que el único archivo donde un error gasta dinero de verdad es un archivo que deberías leer antes de escribir. La política de pago parcial del final es solo, sin scaffold, porque una política que otro escribió por ti es exactamente el valor por defecto silencioso que esta lección existe para matar.

## El conciliador y el pago inverso, de cerca

### Un campo, dos rieles

Empieza por el problema que resuelve la conciliación. El dinero llega a tu ATA de tesorería como un flujo de transferencias. Los pedidos viven en tu libro mayor como filas. Nada en una transferencia cruda dice qué pedido paga, y tienes una sola cuenta de USDC, no una por cliente. El arreglo clásico, el que usan los exchanges, es una dirección de depósito única por usuario: derivar miles de cuentas, vigilarlas todas, barrerlas constantemente. Infraestructura de verdad, costo de verdad, y para una tienda de discos, absurdo.

Los rieles que vienes usando desde la lección de transfer-kit ya llevan la mejor respuesta. Cada pago que arma tu checkout incluye una **reference key**: un valor base58 de 32 bytes, nuevo, adjunto a la transacción como cuenta no firmante. No hace nada on-chain. Existe para que puedas encontrar la transacción después, porque la búsqueda de firmas por cuenta es una capacidad nativa del RPC. Genera una por pedido, guárdala en la fila del pedido, y el pago lleva su propia ficha de reclamo.

Esta es la parte donde vale la pena detenerse. En el módulo 7 vas a conocer x402, el protocolo de pagos nativo de HTTP, y sus facturas llevan un campo `extra.memo` con un id de factura adentro. Riel distinto, spec distinto, exactamente la misma idea: un identificador de pedido guardado en una ranura buscable del pago mismo, así el comercio necesita una cuenta y una consulta en vez de una fábrica de direcciones. La convergencia no es una coincidencia. Cualquier riel de pago push sin direcciones de depósito tiene que resolver el emparejamiento, y un id de pedido buscable es la solución mínima. Lo que quiere decir que el conciliador que escribes hoy es el patrón general y no plomería de Solana Pay, y el módulo 7 le va a enchufar la mitad de x402 sin una reescritura. (Crédito a quien corresponde: el spec de reference de Solana Pay y su helper `findReference` entregaron este patrón primero; nosotros construimos el nuestro directamente sobre el kit para que un solo conciliador sirva a los dos rieles.)

![Las reference keys de Solana Pay y los ids de factura del memo x402 llevan cada uno un id de pedido en un campo buscable, así un solo conciliador empareja cualquiera de los dos rieles con el libro mayor de pedidos.](assets/v01-diagram.webp)

### De la reference al pedido

Mecánicamente, la conciliación es un recorrido de tres pasos. Toma la reference key del pedido. Pídele al RPC las firmas que mencionan esa dirección; `getSignaturesForAddress` las devuelve con la más nueva primero, incluidas las transacciones fallidas, así que el recorrido filtra por `err === null`. Después, y este es el paso que te mantiene honesto, pasa cada candidata por el verificador que construiste al principio de este módulo. La reference key demuestra que alguien le adjuntó tu ficha de reclamo a una transacción. No demuestra que la transacción te haya pagado el monto correcto del token correcto desde el programa correcto. Adjuntar una cuenta arbitraria a una transacción es gratis; eso es lo que hace buscables a las references, y también lo que las hace falsificables como evidencia de pago. El verificador es el juez, la reference es apenas la dirección del juzgado.

La salida del conciliador es deliberadamente más rica que pagado-o-no. El verificador calcula el delta de saldo real en tu ATA en unidades base, y comparar ese delta con el precio del pedido parte el mundo en cuatro estados honestos: **paid** (exacto), **underpaid**, **overpaid** y **unmatched** (no se encontró nada verificable). El libro mayor de la lección pasada nunca registró más que el camino feliz, porque el despacho estaba condicionado a la verificación exacta. Hoy los otros tres estados dejan de ser errores y pasan a ser insumos de la política.

![La conciliación recorre desde la reference key de un pedido, pasando por la búsqueda de firmas y el verificador, hasta una comparación en unidades base que sale como paid, underpaid, overpaid o unmatched.](assets/v02-flowchart.webp)

### El riel del memo, y el dinero sin historia

La reference key es la puerta de entrada, pero la consigna de este artefacto dice reference o memo, y la mitad del memo no es una redundancia. Tu checkout viene estampando las dos en cada pago desde la lección de transaction request: una reference key nueva como cuenta buscable, y el id del pedido dentro de un spl-memo como intención legible por humanos. El verificador ya parsea ese memo; es una de sus cinco comprobaciones. Para el tráfico de tu propio checkout, la reference sola alcanza. El camino del memo existe para todo lo demás, y todo lo demás viene en camino. Un pago x402 del módulo 7 va a llegar llevando su id de factura en `extra.memo`, sin ninguna reference de Solana Pay a la vista, y un comprador que paga una factura compartida desde un retiro de exchange puede desbaratar el flujo de la reference por completo y aun así escribir tu id de pedido en un campo de memo. Un conciliador que puede emparejar por cualquiera de los dos campos es lo que lo hace un conciliador en vez de dos.

Lo que levanta la pregunta en dirección inversa que todo comercio se come en el primer mes: ¿qué pasa con una transferencia que llega a tu ATA de tesorería y no empareja con nada? Ninguna reference conocida, ningún memo parseable, solo dinero. No te vas a enterar por un webhook que no estabas esperando, así que la respuesta honesta es un barrido: recorre el historial de firmas de tu propia ATA y comprueba cada transferencia entrante contra el libro mayor. La misma llamada a `getSignaturesForAddress` hace el trabajo apuntada a la cuenta de tesorería misma, con dos mecánicas que vale la pena conocer. El RPC limita cada página a 1,000 firmas, y paginas hacia atrás pasando la firma más vieja que recibiste como cursor `before` en la siguiente llamada. Y no recorres hasta el principio de los tiempos: persiste la firma más nueva que cada barrido completa, y el siguiente barrido se detiene cuando llega a ella.

```ts
// backoffice-refunds/src/sweep.ts (core walk)
export async function sweepTreasury(treasuryAta: Address, stopAt?: Signature) {
  let before: Signature | undefined;
  while (true) {
    const page = await rpc
      .getSignaturesForAddress(treasuryAta, { limit: 1000, before })
      .send();
    if (page.length === 0) return;
    for (const entry of page) {
      if (entry.signature === stopAt) return; // reached last sweep's frontier
      if (entry.err !== null || isProcessed(entry.signature)) continue;
      const matched = await tryMatchByReferenceOrMemo(entry.signature);
      if (!matched) recordOrphan(entry.signature); // money with no story
    }
    before = page[page.length - 1].signature;
  }
}
```

Una fila huérfana no es un problema para auto-resolver; es un problema para sacar a la luz. El valor por defecto tentador, empujarlo derecho de vuelta a donde vino, es un auto-reembolso sin barrera, una trampa que la sección de política de abajo disecciona, y quemaría comisiones devolviéndoles polvo a los bots. Las huérfanas van a una fila de revisión, y si alguna llega a devolverse, pasa por el mismo camino de reembolso con guarda que todo lo demás, comprobación de finality y enlace de origen incluidos. El verdadero regalo del barrido es una propiedad, no una funcionalidad: córrelo después de cualquier caída y el libro mayor converge a lo que dice la blockchain, porque la blockchain, no tu bandeja de webhooks, siempre fue el reporte de liquidación.

### El reembolso que te toca inventar

Ahora el prensado alabeado. En los rieles de tarjeta con los que creciste integrando, este momento está muy pavimentado. Stripe tiene una API de reembolsos, el emisor tiene un proceso de chargeback detrás, y una máquina entera de disputas está detrás de eso. La máquina existe porque los pagos con tarjeta son pagos pull: el comercio metió la mano en la cuenta del comprador, así que el sistema trae una palanca para meter la mano de vuelta. Los pagos push invierten la geometría. El comprador te entregó tokens en una transferencia final y atómica; no hay palanca, no hay contraparte que pueda meter la mano en tu cuenta, y por eso nadie escribió una página de reembolsos, porque no hay primitiva de reembolso que documentar. El pitch de Shopify que desempacó la lección de apertura de este módulo —chargebacks eliminados por construcción— estaba vendiendo exactamente este momento. ¡Cierto! Y es precisamente por eso que lo que viene después es construcción original de nuestra parte, ensamblada con piezas que ya tienes, no algo que puedas ir a buscar.

Entonces, ¿qué es un reembolso, estructuralmente? Es un pago. Esa es toda la idea. Tienes la transacción de origen, que nombra la billetera del comprador como fuente. Tienes transfer-kit, que empuja stablecoins a cualquier dirección. Un reembolso es un pago push inverso: mismo mint, monto menor o igual a lo que de verdad pagaron, destino leído desde la transacción de origen, más una cosa que los rieles de pago no te van a dar gratis, un enlace de auditoría. La fila del reembolso en el libro mayor registra la firma de origen que revierte, y su memo on-chain lleva esa firma también, así cualquiera que audite cualquiera de los dos lados del libro mayor puede caminar de la venta al reembolso y de vuelta sin confiar en tu base de datos.

Esto no es nosotros improvisando en el vacío. Stripe cruzó este puente primero para su flujo de pago con cripto, y su comportamiento documentado es el precedente que reflejamos: los reembolsos se devuelven como stablecoins a la billetera de origen. No a una dirección que el comprador te manda por correo, no a quien pida amablemente con un ticket de soporte convincente. La billetera de origen, leída desde la blockchain. Esa sola regla borra una clase entera de fraude donde "el comprador" que pide el reembolso no es la billetera que pagó. Vale la pena notar con cuánto cuidado los grandes acotaron este territorio: el mismo riel de Stripe limita a los clientes a 10,000 USD por transacción. Cuando la empresa de pagos con más experiencia del planeta pone barreras de protección así de ajustadas alrededor de dinero irreversible, capta la indirecta sobre cuánto respeto merece la dirección inversa.

![Comparación entre los rieles de tarjeta y los rieles push: las tarjetas traen una máquina de chargebacks documentada, mientras que los rieles push no traen ninguna primitiva de reversión, así que el comercio construye el reembolso como un pago nuevo.](assets/v03-comparison.webp)

La asimetría corta para los dos lados, y aquí es donde te muerde a ti y no al comprador. Ningún chargeback protege al comercio tampoco. El primer reembolso que puse en fila en estos rieles, comprobé el destino tres veces como si estuviera desactivando algo. Buen instinto, blanco equivocado: la dirección estaba bien, el problema era que el pago de origen tenía segundos de vida. Piensa en lo que eso quiere decir. Un pago con commitment `confirmed` puede, rara vez, quedar en un fork que después se descarta. Si lo reembolsas y el fork muere, el "pago" se evapora mientras tu reembolso, una transacción totalmente independiente, aterriza y alcanza finality igual. Acabas de pagar dinero de verdad para revertir un pago que nunca pasó, y no hay palanca con la que meter la mano de vuelta, porque construiste sobre el riel que no tiene una. La guarda es una sola llamada RPC: la firma de origen tiene que reportar `finalized`, el commitment que la red no va a deshacer, antes de que el constructor de reembolsos firme nada. Alcanzar finality cuesta unos diez segundos según la estimación del ecosistema, los mismos ~10s que derivó la lección de apertura de este módulo al objetivo de slot de 300ms. Un reembolso nunca es tan urgente como para no poder esperar diez segundos; la lección de apertura del módulo 4 ya hizo este mismísimo argumento según el valor para el despacho, y el caso del reembolso es más fuerte, porque ahora tú eres el pagador.

![Una solicitud de reembolso pasa las comprobaciones del libro mayor y una barrera de finality sobre la firma de origen antes de que se empujen stablecoins a la billetera de origen; un origen sin finality es rechazado.](assets/v04-flowchart.webp)

### Política, no valores por defecto

Los pagos exactos eran el 95 por ciento fácil. Los otros tres estados esconden cada uno una decisión, y la disciplina en la que insiste esta lección es que tomes cada decisión a propósito, la escribas y la codifiques, porque todo valor por defecto silencioso es uno de dos modos de falla con gabardina puesta.

Recorre el caso de juguete. Un prensado cuesta 30 USDC. Un comprador manda 12. ¿Qué debería pasar? Despachar en silencio es la primera trampa: la intención no es un pago, acabas de vender un disco de 30 dólares por 12, y se corre la voz de que Wavelength despacha con pago parcial. Auto-reembolsar al instante es la segunda trampa, y es más sutil. Imagínate un adversario que no quiere tus discos, solo quiere hacerte daño. Programa un loop: paga de menos por una fracción, deja que tu auto-reembolsador empuje el dinero de vuelta, repite. Cada ciclo le cuesta casi nada y te cuesta a ti una comisión de transacción, una verificación RPC, una construcción de reembolso y una escritura en el libro mayor, para siempre, al ritmo al que pueda mandar transferencias. Un camino de auto-reembolso sin barrera es una invitación a que otro gaste tu dinero en comisiones. El camino del reembolso necesita una guarda: un límite de tasa, un umbral mínimo, una fila de revisión manual, algo que haga que el loop le cueste al atacante más de lo que te cuesta a ti.

Entre las trampas está el espacio de las políticas defendibles, y defendible quiere decir que la contrapartida está declarada. Retén el pedido como underpaid y dale al comprador una ventana para completar el pago: lo más gentil para los errores honestos, te cuesta un disco retenido y la contabilidad de los vencimientos. Reembolso menos una comisión de procesamiento: el cierre más limpio, te cuesta buena voluntad del comprador y necesita la comisión publicada de entrada, además de que igual pasa por el camino de reembolso con guarda. El pago de más lo refleja: quédate con el excedente como crédito de tienda registrado (simple, pero ahora tienes un pasivo), o reembolsa el excedente después de la finality por el mismo camino con guarda, con `amountBaseUnits` puesto en el excedente, nunca en el pago entero.

Un **pago parcial** es pago de menos con un nombre más respetable, y merece su propio caso trabajado porque los pedidos de varios ítems son donde la política vaga se va a morir. Digamos que un carrito de Wavelength lleva el prensado a 30 USDC, una bolsa tote a 10 y un slipmat a 5, y llegan 40 de los 45. Existen dos lecturas defendibles. La lectura atómica por pedido dice que el pedido es una sola cosa, retén todo por los 5 que faltan o reembolsa los 40, lo que mantiene el despacho binario al costo de demorar dos ítems que el comprador pagó completos. La lectura de llenado por línea dice despacha el prensado y la bolsa, después rutea el slipmat corto por 5 hacia tu bifurcación de pago de menos, que es más amable y te cuesta lógica de asignación: ¿qué ítems cubrieron los 40? ¿Los de mayor valor primero? ¿Según el orden del comprador? Tu memo de política tiene que nombrar la regla de asignación, porque "obviamente el prensado" deja de ser obvio el día que dos ítems empatan. Cualquiera de las dos lecturas es defendible. Decidir a las 2 a.m. por ticket no lo es.

El tl;dr es: ninguna de estas respuestas es correcta, y ese es el punto. Lo correcto es que tu libro mayor pueda mostrar, para cada pago no exacto, qué política declarada lo ruteó y cuándo. Esa auditabilidad es cómo se ve una disputa en rieles sin máquina de disputas.

![Tabla de los tres estados de pago no exacto, la trampa del valor por defecto silencioso para cada uno, y dos políticas defendibles por estado con el costo que carga cada política.](assets/v05-table.webp)

## Lab: construye backoffice-refunds

Seis pasos. El conciliador y el constructor de reembolsos vienen andamiados con TODOs; el módulo de política es tuyo. Todo corre contra devnet con los mismos pins de workspace que el curso congeló en el módulo 2: `@solana/kit` ^6.10.0 con `@solana-program/token` 0.14.0 (fijado en 2026-08; el estándar peer del ecosistema kit se movió desde entonces a la línea v7, y estos workspaces se quedan en v6 porque `@solana/pay` tiene a kit ^6.9 como peer; el módulo de suscripciones recorre esa costura como se debe). Ninguna dependencia nueva hoy, lo que es su propia pequeña lección: un sistema de reembolsos es una composición, no un paquete.

**Paso 1: el scaffold.** Crea el workspace e instala su tooling:

```bash
mkdir -p backoffice-refunds/src
cd backoffice-refunds && npm init -y && npm pkg set type=module
npm pkg set scripts.build="tsc --noEmit"
npm pkg set scripts.verify:refunds="tsx verify-refunds.ts"
npm install -D tsx typescript @types/node
cd .. && npm install
```

Crea estos siete archivos bajo `src/` a medida que recorres los pasos de abajo: `reconcile.ts`, `refund.ts`, `policy.ts`, `origin.ts`, `sweep.ts`, `ledger.ts` (una extensión delgada del libro mayor del backoffice) y `reconcile-demo.ts`, el driver de checkpoint impreso en el paso 2. El harness de verificación que la línea de script acaba de cablear, `verify-refunds.ts`, vive en la raíz del paquete exactamente como vivía el del verificador, y el paso 6 lo imprime entero. El build pasa con los scaffolds puestos porque cada TODO lanza en vez de dar error de tipos; el harness es lo que te va a decir que están sin terminar.

**Paso 2: el conciliador.** Primero va una enmienda al verificador, porque la clasificación de abajo lee un campo que el contrato de la lección 1 no lleva. El congelamiento de `VerifyResult` era sobre la forma de la llamada, `verify(signature, expectedOrder)`, y sobre nunca cambiar un campo existente; agregar uno no es ninguna de las dos cosas. En `verifier/src/types.ts`, el resultado gana el delta observado:

```ts
// verifier/src/types.ts: the one amendment to the frozen contract, additive.
export type VerifyResult =
  | { ok: true; reason: 'verified'; signature: string; paidBaseUnits: bigint }
  | {
      ok: false;
      reason: RejectReason | 'not-found';
      signature: string;
      paidBaseUnits?: bigint;
    };
```

Obligatorio en la rama de aprobación, opcional en la rama de rechazo, porque los rechazos río arriba de la comprobación de delta (`duplicate`, `not-found`) nunca se enteraron del número. En `verifier/src/verify.ts`, los dos returns río abajo del cálculo del delta lo llevan:

```ts
// verifier/src/verify.ts: both returns that know the delta now report it.
    if (credit.delta < expected.amountBaseUnits) {
      return { ok: false, reason: 'underpaid', signature, paidBaseUnits: credit.delta };
    }
    // ...memo check unchanged...
    deps.store.add(signature);
    return { ok: true, reason: 'verified', signature, paidBaseUnits: credit.delta };
```

Todos los llamadores anteriores siguen pasando la comprobación de tipos, porque ninguno de ellos leía un campo que no existía; vuelve a correr `npm run --workspace verifier verify:verifier` para demostrar que la enmienda no rompió nada. Ahora abre `backoffice-refunds/src/reconcile.ts`. El recorrido es el de la sección de teoría: la búsqueda de firmas y el filtro de transacciones fallidas vienen dados, y el TODO es la clasificación, el paso donde el delta observado del verificador se vuelve uno de los cuatro estados:

```ts
// backoffice-refunds/src/reconcile.ts
import { createSolanaRpc } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';
import { createVerifier, createRpcFetchTransaction, createMemoryStore } from 'verifier';
import { getOrderByReference, recordPayment } from './ledger';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// Reconciliation gets its OWN verifier with a fresh, empty store, on purpose.
// The ingestion path's processed-signatures set exists to stop double
// fulfillment; reconciliation's whole job is to re-derive truth from chain
// state, including for payments the ledger already knows. Share the ingestion
// store and every already-fulfilled payment would come back 'duplicate' and
// read as unmatched. And "fresh per run" must mean per CALL, not per import:
// the verifier is built inside reconcileOrder below, because a module-scope
// store in a long-lived process would mark a re-reconciled payment
// 'duplicate' -> unmatched, the exact failure this isolation exists to avoid.

export type ReconcileResult =
  | { status: 'paid'; signature: Signature }
  | { status: 'underpaid'; signature: Signature; paidBaseUnits: bigint }
  | { status: 'overpaid'; signature: Signature; paidBaseUnits: bigint }
  | { status: 'unmatched' };

export async function reconcileOrder(reference: Address): Promise<ReconcileResult> {
  const verify = createVerifier({
    fetchTransaction: createRpcFetchTransaction(),
    store: createMemoryStore(), // fresh per call; see the note above
  });
  const order = getOrderByReference(reference);
  if (!order) return { status: 'unmatched' };

  // Given: candidate signatures for the reference key, newest first.
  const candidates = await rpc
    .getSignaturesForAddress(reference, { limit: 10 })
    .send();

  for (const entry of candidates) {
    if (entry.err !== null) continue; // failed txs appear in this list too

    // The lesson-1 checks, unchanged: program, mint, delta, memo. Dedup runs
    // against this call's fresh store, so history stays re-inspectable.
    const check = await verify(entry.signature, order);
    if (!check.ok && check.reason !== 'underpaid') continue;

    // TODO: classify by the observed delta, in base units, never floats.
    // The verifier hands you check.paidBaseUnits (always present on the
    // 'verified' and 'underpaid' branches; the optionality covers rejections
    // upstream of the delta check). Record the payment against order.id, then
    // return 'paid' when it equals order.amountBaseUnits, 'underpaid' when it
    // is short, 'overpaid' when it exceeds. Bigints only: one float
    // comparison here misroutes every borderline amount.
    throw new Error('TODO: classify the observed delta');
  }
  return { status: 'unmatched' };
}
```

Dos notas de diseño, porque son las decisiones interesantes. Primero, el verificador sigue siendo el único juez; la enmienda que acabas de hacer le entrega al conciliador el `paidBaseUnits` observado para que pueda clasificar en vez de solo aceptar o rechazar. Un rechazo `underpaid` ya no es un callejón sin salida, es dato. Una advertencia para llevarte al módulo 7: en el orden del verificador la comprobación de monto corre antes que la comprobación de memo, así que un veredicto `underpaid` no lleva ninguna atadura al pedido por sí solo. En el camino de la reference eso es seguro, la búsqueda de firmas ya ató cada candidata a la reference de este pedido, pero el camino del memo tiene que volver a comprobar el memo antes de clasificar un pago corto, o la transferencia de un desconocido podría rutearse a tu fila de política. Segundo, fíjate en que el conciliador no confía en el camino del webhook en absoluto. Los webhooks te dijeron que probablemente pasó algo; la conciliación es el proceso por lotes que reconstruiría la verdad solo desde el estado de la blockchain si se perdieran todos los webhooks. Los comercios que hayan corrido un cierre de mes contra un reporte de liquidación de un PSP ya conocen esta forma: el mismo trabajo, salvo que tu reporte de liquidación es la blockchain, consultable en cualquier momento.

El camino del memo y el barrido de tesorería que salieron en la sección de teoría viven en `sweep.ts`, cableados pero con el TODO de `tryMatchByReferenceOrMemo` abierto: primero la búsqueda por reference, el parseo del memo como fallback, fila huérfana cuando las dos fallan. Llénalo después de que funcione el recorrido principal. El harness de hoy solo ejercita el camino de la reference, y ten claro cuándo se gana el sueldo de verdad la mitad del memo, porque no es el camino feliz en ningún lado: las liquidaciones x402 del módulo 7 concilian por su propio hook `onAfterSettle` derecho al libro mayor, con id de memo y todo, sin acercarse a este barrido. Para lo que sirve el barrido es para el dinero que llegó mientras no había nadie escuchando —un worker caído, un webhook deshabilitado, un settle que aterrizó después de que el proceso murió— y el Challenge del módulo 7 te hace demostrarlo en exactamente ese caso. Escríbelo para la caída, no para el tráfico.

El driver de checkpoint es `src/reconcile-demo.ts`, totalmente trabajado. Siembra la fila de pedidos abiertos que el conciliador va a buscar (en producción, tu servidor de checkout hace esa llamada a `recordOrder` en el momento en que acuña la reference; aquí la demo hace las veces de eso), después corre el recorrido e imprime el veredicto. Lee las mismas variables de entorno que te enseñó el harness en vivo del módulo 4, así que nada nuevo que recordar:

```ts
// backoffice-refunds/src/reconcile-demo.ts
// Usage: npx tsx backoffice-refunds/src/reconcile-demo.ts <reference>
// Seeds an open order against the reference, then reconciles it.
import { address } from '@solana/kit';
import { recordOrder } from './ledger';
import { reconcileOrder } from './reconcile';

function req(name: string): string {
  const v = process.env[name];
  if (!v) throw new Error(`set ${name}, same variable as the module 4 live harness`);
  return v;
}

if (!process.argv[2]) throw new Error('usage: reconcile-demo.ts <reference>');
const reference = address(process.argv[2]);

const order = {
  orderId: process.env.ORDER_ID ?? 'ord-0231',
  recipient: req('MERCHANT'),
  recipientAta: req('MERCHANT_ATA'),
  mint: process.env.MINT ?? '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
  amountBaseUnits: BigInt(process.env.AMOUNT_BASE_UNITS ?? '30000000'),
};
recordOrder(order, reference);

const result = await reconcileOrder(reference);
if (result.status === 'unmatched') {
  console.log(`reconcile: order ${order.orderId} unmatched`);
  process.exit(1);
}
const paid =
  'paidBaseUnits' in result ? result.paidBaseUnits : order.amountBaseUnits;
console.log(
  `reconcile: order ${order.orderId} ${result.status}, signature ${result.signature}, ${paid} base units`,
);
```

Importa `recordOrder` desde `ledger.ts`, que el paso 3 desarrolla; la mitad de pedidos abiertos son dos operaciones de Map, así que si vas estrictamente en orden, salta adelante, cablea esas dos funciones y vuelve.

Checkpoint antes de seguir. Paga uno de tus propios pedidos de devnet desde una segunda billetera —una segunda billetera específicamente, porque el verificador lee el cambio de saldo en tu cuenta de token de comercio y un comercio que se paga a sí mismo no cambia nada. La CLI de transfer-kit hace las dos mitades, desde la raíz de `wavelength` (si a `/tmp/customer.json` no le queda SOL, `solana transfer $(solana-keygen pubkey /tmp/customer.json) 0.05 --allow-unfunded-recipient --url devnet` lo recarga):

```bash
npm run --workspace transfer-kit pay -- $(solana-keygen pubkey /tmp/customer.json) 13
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5
```

La segunda corrida imprime la `reference` que adjuntó. Pásale eso a la demo, con el precio que de verdad pagaste:

```bash
MERCHANT=$(solana address) MERCHANT_ATA=<your usdc ata> AMOUNT_BASE_UNITS=12500000 \
  npx tsx backoffice-refunds/src/reconcile-demo.ts <reference-from-your-order>
```

```
reconcile: order ord-0231 paid, signature 5Kd...w2, 12500000 base units
```

Montos exactos, coincidencia exacta, estado `paid`. Si te sale `unmatched` en un pago que puedes ver en el explorador, tu verificador lo rechazó; corre la demo con `DEBUG=verify` y lee cuál de las cinco comprobaciones dijo que no. Ese loop de falla, el conciliador dice unmatched, el verificador dice por qué, es el ritmo de depuración para el resto del curso.

**Paso 3: el libro mayor gana un segundo tipo de fila.** Abre `ledger.ts`. Crece en dos direcciones, y la primera es un store que este módulo viene necesitando calladamente desde siempre: una **tabla de pedidos abiertos**. `recordOrder(order, reference)` guarda un `ExpectedOrder` contra la reference key que acuñó tu checkout, y `getOrderByReference` es su búsqueda; el script reconcile-demo siembra una fila para el pago que estás por hacer, y tu servidor de checkout es donde va esa llamada en producción. Sin ella, un pedido sin pagar o pagado de menos sería invisible para el conciliador, porque el libro mayor de la lección pasada nunca registró más que pagos despachados. Segundo, la fila de pago de la lección pasada quedó congelada en cinco campos, así que di la extensión en voz alta antes de usarla: las filas de pago ganan `paidBaseUnits`, el delta que el verificador de verdad observó on-chain, que es un número distinto del `amountBaseUnits` que esperaba el pedido en el momento en que alguien paga de menos, más un `refundSignature` nullable que se queda vacío hasta que una reversión lo nombra. Los dos son aditivos, así que todo lector de `rows()` de la lección pasada sigue funcionando. Di el escritor en voz alta, porque es la mitad que todo el mundo olvida y la omisión es silenciosa: **`recordRefund` escribe dos veces** —agrega la fila de reembolso de abajo, y estampa `refundSignature` en la fila de pago que revierte. Sáltate la segunda escritura y la guarda de `already refunded` en el constructor de reembolsos lee un campo que nada llena nunca, así que no dispara nunca, y cada nueva corrida de tu barrera empuja otro reembolso de verdad en devnet —el mismísimo doble envío que la guarda existe para prevenir. Los reembolsos agregan una fila enlazada propia:

```ts
// backoffice-refunds/src/ledger.ts (additions)
export type RefundRow = {
  originSignature: string;  // the payment this reverses: the audit link
  refundSignature: string;  // the refund's own on-chain signature
  refundReference: string;  // fresh reference key, so refunds reconcile too
  amountBaseUnits: string;  // bigint serialized as string on disk
  reason: string;
  at: string;               // ISO timestamp
};
```

La firma de origen es todo el diseño. Una fila de reembolso que no puede nombrar el pago que revierte es dinero que se va sin historia, no auditable por ti e indistinguible de un robo para cualquier otro que lea tus libros. Una verruga práctica que vale el paréntesis: `bigint` no sobrevive a `JSON.stringify`, así que las unidades base viven como cadenas en disco y reviven en los bordes. Te encontraste con esta mismísima verruga en la lección de transfer-kit desde la otra dirección; misma regla, cadena exacta adentro, bigint exacto afuera.

![La fila de reembolso del libro mayor guarda la firma del pago de origen y la suya propia, y el memo de la transacción de reembolso repite el origen, así un auditor puede recorrer el enlace en los dos sentidos.](assets/v06-diagram.webp)

**Paso 4: el constructor de reembolsos.** Este es el archivo que no existía en ningún doc que pudieras haber copiado. Abre `refund.ts`:

```ts
// backoffice-refunds/src/refund.ts
import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { createSolanaRpc, createKeyPairSignerFromBytes, generateKeyPairSigner } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';
import { sendStablecoin } from 'transfer-kit';
import { getPayment, recordRefund } from './ledger';

const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
const RPC_WS_URL = process.env.RPC_WS_URL ?? 'wss://api.devnet.solana.com';
const rpc = createSolanaRpc(RPC_URL);

// The merchant wallet that received the sale is the wallet that funds the
// reversal: the same keypair file the module 2 lab created.
async function loadMerchantKeyBytes(): Promise<Uint8Array> {
  const keyfile = process.env.MERCHANT_KEYFILE ?? `${homedir()}/.config/solana/id.json`;
  return new Uint8Array(JSON.parse(await readFile(keyfile, 'utf8')));
}
const merchant = await createKeyPairSignerFromBytes(await loadMerchantKeyBytes());

export async function refundPayment(
  originSignature: Signature,
  opts: { to: Address; mint: Address; amountBaseUnits: bigint; reason: string },
) {
  // Guard 1: our own ledger must know this payment, once.
  const payment = getPayment(originSignature);
  if (!payment) throw new Error(`no ledger row for ${originSignature}; refusing to refund`);
  if (payment.refundSignature) throw new Error(`already refunded in ${payment.refundSignature}`);
  if (opts.amountBaseUnits > BigInt(payment.paidBaseUnits))
    throw new Error('refund exceeds the amount actually paid');

  // Guard 2, given because it is the one rule this lesson will not let you
  // get wrong: the origin payment must be FINALIZED before we push.
  const { value } = await rpc
    .getSignatureStatuses([originSignature], { searchTransactionHistory: true })
    .send();
  if (value[0]?.confirmationStatus !== 'finalized') {
    throw new Error('origin payment not finalized; a push refund is irreversible, wait');
  }

  // The refund is a plain push payment, built by the kit that sent the sale,
  // called with the exact signature transfer-kit has had since the roster lesson: the
  // CALLER mints the reference key, so we give the refund its own claim ticket.
  const refundReference = (await generateKeyPairSigner()).address;
  const { signature } = await sendStablecoin({
    rpcUrl: RPC_URL,
    rpcSubscriptionsUrl: RPC_WS_URL,
    payer: merchant,
    mint: opts.mint,
    recipient: opts.to,
    amount: opts.amountBaseUnits, // already base units, never a float
    memo: `refund:${originSignature}:${opts.reason}`,
    reference: refundReference,
  });

  // Writes twice: the refund row, AND refundSignature back onto the payment
  // row. That back-stamp is what arms the `already refunded` guard above.
  recordRefund({
    originSignature,
    refundSignature: signature,
    refundReference,
    amountBaseUnits: opts.amountBaseUnits.toString(),
    reason: opts.reason,
    at: new Date().toISOString(),
  });

  return { signature, reference: refundReference };
}
```

Lee la forma que tiene. Las guardas primero, de la más barata a la más cara: tres comprobaciones del libro mayor que no cuestan nada, después una llamada RPC de estado, y solo entonces la transacción. El destino merece su propio momento, porque es por donde entraría un ataque de ingeniería social. `opts.to` nunca lo escribe un humano y nunca se lee de un ticket de soporte; el llamador lo extrae de la transacción de origen misma. El scaffold trae el helper, y es lo bastante corto como para leerlo entero:

```ts
// backoffice-refunds/src/origin.ts
import { createSolanaRpc } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

export async function getOriginatingWallet(
  originSignature: Signature,
  mint: Address,
): Promise<Address> {
  const tx = await rpc
    .getTransaction(originSignature, {
      encoding: 'jsonParsed',
      maxSupportedTransactionVersion: 1,
    })
    .send();
  if (!tx?.meta) throw new Error('origin transaction not found');

  // The account whose token balance for our mint went DOWN is the payer's ATA;
  // its owner field is the wallet the refund returns to.
  for (const post of tx.meta.postTokenBalances ?? []) {
    if (post.mint !== mint) continue;
    const pre = tx.meta.preTokenBalances?.find(b => b.accountIndex === post.accountIndex);
    const preAmount = BigInt(pre?.uiTokenAmount.amount ?? '0');
    const postAmount = BigInt(post.uiTokenAmount.amount);
    if (postAmount < preAmount && post.owner) return post.owner as Address;
  }
  throw new Error('no debited token account for this mint in the origin transaction');
}
```

Los mismos saldos de token pre y post que lee el verificador para el delta de tu lado, recorridos desde el otro extremo: la cuenta que fue debitada nombra a su dueño, y ese dueño es el destino del reembolso. Esta es la regla de Stripe sobre la billetera de origen, hecha estructural en vez de procedimental; no hay ningún camino de código donde un correo persuasivo cambie a dónde va el dinero.

![El destino del reembolso se lee de la transacción de origen misma: la cuenta de token debitada por el mint del pago nombra a su dueño, y ese dueño es la billetera de origen.](assets/v07-diagram.webp)

De vuelta al constructor mismo. La comprobación de `already refunded` importa más de lo que parece: las solicitudes de reembolso van a llegar desde el tooling de soporte, y el tooling de soporte reintenta, así que la idempotencia por firma de origen aquí es la misma disciplina que la idempotencia por firma en el handler de webhooks. Y el memo hace la reversión legible on-chain, no solo en tu base de datos. El reembolso lleva su propia reference key nueva, acuñada aquí y entregada a transfer-kit exactamente como ha sido la reference de cada venta desde el módulo 2, lo que quiere decir que un reembolso se puede localizar con una búsqueda de firmas exactamente igual que un pago. El dinero que sale va sobre los mismos rieles buscables que el dinero que entra, gratis, porque compusiste en vez de escribir un segundo sistema. Esa composición es la recompensa silenciosa de toda la escalera de artefactos hasta aquí.

**Paso 5: codifica una política.** `policy.ts` es el peldaño solo, pero la superficie de tipos está fija para que el harness pueda rutear a través de ella:

```ts
// backoffice-refunds/src/policy.ts
import type { ExpectedOrder } from 'verifier';
import type { ReconcileResult } from './reconcile';

export type UnderpayPolicy =
  | { kind: 'hold-for-topup'; windowMinutes: number }
  | { kind: 'refund-minus-fee'; feeBaseUnits: bigint };

export type OverpayPolicy =
  | { kind: 'credit-to-order' }
  | { kind: 'refund-surplus-after-finality' };

// Yours to choose, and to defend in writing.
export const policy: { underpay: UnderpayPolicy; overpay: OverpayPolicy } = {
  underpay: { kind: 'hold-for-topup', windowMinutes: 60 },
  overpay: { kind: 'credit-to-order' },
};

// The ledger event a routed non-exact payment produces. No fulfillment
// member exists here on purpose.
export interface PolicyEvent {
  orderId: string;
  policy: UnderpayPolicy['kind'] | OverpayPolicy['kind'];
  paidBaseUnits: bigint;
  at: string;
}

// Yours to write: take an underpaid reconcile result and the order it
// shorts, and return the PolicyEvent your stated policy dictates.
export function routeUnderpaid(
  result: Extract<ReconcileResult, { status: 'underpaid' }>,
  order: ExpectedOrder,
): PolicyEvent {
  throw new Error('TODO: route by your stated policy');
}
```

Fíjate en lo que el sistema de tipos se niega a expresar: no hay miembro `fulfill-anyway` y no hay miembro de auto-reembolso instantáneo sin barrera. Las dos trampas son irrepresentables, que es la guarda más barata que vas a entregar en tu vida. `routeUnderpaid` toma un resultado de conciliación underpaid y devuelve el evento de libro mayor que dicta tu política; el harness solo comprueba que un pedido underpaid produzca un evento estampado con la política en vez de un despacho.

**Paso 6: la barrera.** Guárdalo como `backoffice-refunds/verify-refunds.ts`, el archivo al que ya apunta la línea de script del paso 1. Lee el orden de las operaciones antes de correrlo: la comprobación de underpaid va primero porque su conciliación registra la fila de pago que la mitad del reembolso después revierte, y las dos sondas que-deben-fallar encierran la única llamada que gasta dinero.

```ts
// backoffice-refunds/verify-refunds.ts
// The lesson's acceptance gate, wired to `npm run verify:refunds`.
// Env: REFERENCE (the reference of the devnet payment you made in step 2's
// checkpoint), plus MERCHANT / MERCHANT_ATA / MINT / AMOUNT_BASE_UNITS from
// the module 4 live harness. Optional: FRESH_SIGNATURE, see below.
import { address, signature } from '@solana/kit';
import { getPayment, recordOrder } from './src/ledger';
import { reconcileOrder } from './src/reconcile';
import { policy, routeUnderpaid } from './src/policy';
import { refundPayment } from './src/refund';
import { getOriginatingWallet } from './src/origin';

function fail(msg: string): never {
  console.error(`REFUNDS FAIL: ${msg}`);
  process.exit(1);
}

function req(name: string): string {
  const v = process.env[name];
  return v ?? fail(`set ${name}, same variables as the module 4 live harness`);
}

const reference = address(req('REFERENCE'));
const priceBaseUnits = BigInt(process.env.AMOUNT_BASE_UNITS ?? '30000000');

// 1. Underpaid routing. An order for double the price turns the real payment
// into a short one, which must land in policy, never in fulfillment.
const order = {
  orderId: 'ord-underpaid-check',
  recipient: req('MERCHANT'),
  recipientAta: req('MERCHANT_ATA'),
  mint: process.env.MINT ?? '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
  amountBaseUnits: priceBaseUnits * 2n,
};
recordOrder(order, reference);
const short = await reconcileOrder(reference);
if (short.status !== 'underpaid') {
  fail(`expected underpaid, got ${short.status}: is your classification comparing base units?`);
}
const event = routeUnderpaid(short, order);
if (event.policy !== policy.underpay.kind) {
  fail(`routed to ${event.policy}, but your stated policy is ${policy.underpay.kind}`);
}
console.log(`  underpaid order ${order.orderId}: routed to ${event.policy}`);

// 2. Must fail: an origin the ledger has never seen.
const mintAddress = address(order.mint);
try {
  await refundPayment(signature('1'.repeat(64)), {
    to: address(order.recipient),
    mint: mintAddress,
    amountBaseUnits: 1n,
    reason: 'must-never-send',
  });
  fail('a refund against an origin the ledger has never seen went through');
} catch (err) {
  if (!(err instanceof Error && err.message.includes('refusing to refund'))) throw err;
  console.log('  unknown origin: refused, as it must be');
}

// 3. The real reversal: refund the payment part 1 just classified.
const originSignature = short.signature;
const payment = getPayment(originSignature);
if (!payment) fail('classification never recorded the payment row; re-read the step 2 TODO');
try {
  const to = await getOriginatingWallet(originSignature, mintAddress);
  const refund = await refundPayment(originSignature, {
    to,
    mint: mintAddress,
    amountBaseUnits: BigInt(payment.paidBaseUnits),
    reason: 'verify-harness',
  });
  console.log(`  refund ${refund.signature}: recorded, linked to origin`);
} catch (err) {
  const msg = err instanceof Error ? err.message : String(err);
  if (msg.includes('already refunded')) {
    console.log('  refund: idempotency guard held on re-run; linkage already in the ledger');
  } else if (msg.includes('not finalized')) {
    fail('origin payment not finalized yet; wait a few seconds and re-run');
  } else {
    throw err;
  }
}

// 4. Must fail, env-gated: a refund requested before the origin finalized.
const freshSig = process.env.FRESH_SIGNATURE;
if (freshSig) {
  const fresh = signature(freshSig);
  const freshRow = getPayment(fresh);
  if (!freshRow) fail('FRESH_SIGNATURE has no ledger row: reconcile it first');
  try {
    await refundPayment(fresh, {
      to: await getOriginatingWallet(fresh, mintAddress),
      mint: mintAddress,
      amountBaseUnits: BigInt(freshRow.paidBaseUnits),
      reason: 'must-never-send',
    });
    fail('refund went through: either the finality guard is missing, or the payment finalized before the gate ran; use a fresher signature');
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (!msg.includes('not finalized')) throw err;
    console.log('  finality guard: refused the pre-finality refund');
  }
} else {
  console.log('  finality guard: SKIP (set FRESH_SIGNATURE to exercise it)');
}

console.log(
  'refunds: refund recorded against origin signature; underpaid order routed to policy',
);
```

Córrelo desde el paquete:

```bash
npm run --workspace backoffice-refunds verify:refunds
```

Salida esperada, primera corrida:

```
  underpaid order ord-underpaid-check: routed to hold-for-topup
  unknown origin: refused, as it must be
  refund <signature>: recorded, linked to origin
  finality guard: SKIP (set FRESH_SIGNATURE to exercise it)
refunds: refund recorded against origin signature; underpaid order routed to policy
```

Las dos sondas que-deben-fallar son los dientes. El rechazo de origen desconocido corre todas las veces: `signature('1'.repeat(64))` es la firma de todos ceros, y si empujar dinero contra ella hace cualquier cosa que no sea lanzar, tu guarda del libro mayor es ornamental —vuelve al paso 4. El rechazo por finality necesita un pago que todavía no haya alcanzado finality, que ningún harness puede conjurar a pedido, así que está condicionado a una variable de entorno: paga un pedido, corre `reconcile-demo` sobre él de inmediato, exporta la firma como `FRESH_SIGNATURE`, y vuelve a correr la barrera dentro de la ventana de finality de ~10 segundos que derivó la lección de apertura de este módulo (31 o más bloques al objetivo de 300ms, alrededor de 9.3s, redondeado hacia arriba porque el final es "o más"). Reembolsarlo tiene que ser rechazado. Fíjate también en lo que demuestra gratis una nueva corrida: la parte 3 pega en la guarda de `already refunded` y lo reporta como aprobación, porque que la idempotencia sobreviva una segunda corrida es la propiedad, no un inconveniente.

## Challenge

El trabajo de completion queda atrás si el harness pasa: conciliación por reference con clasificación juzgada por el verificador, y el constructor de reembolsos con guarda. El peldaño solo tiene dos partes, y la segunda es la que va a sentirse desconocida.

Primero, la mitad operativa. Emite un reembolso de verdad en devnet: paga uno de tus propios pedidos desde una segunda billetera, espera la finality, reembólsalo con `refundPayment`, y después demuestra que el reembolso se puede encontrar por su propia ficha de reclamo: pídele `getSignaturesForAddress(refundReference)` y trae la transacción que devuelve. Deberías ver exactamente una firma, la del reembolso, llevando el memo `refund:<origin>:<reason>`, porque el dinero que sale va sobre los mismos rieles de reference buscable que el dinero que entra. (No lo pases por `reconcileOrder`: esa función clasifica créditos a tu tesorería contra un pedido abierto, y un reembolso es un débito sin fila de pedido, así que `unmatched` es la respuesta correcta ahí, no un bug.) Después rómpelo a propósito: pide un segundo reembolso del mismo origen y confirma que la guarda de idempotencia lanza.

Segundo, el memo de política. Elige tu política de pago de menos y de pago parcial, codifícala en `policy.ts`, y escríbela en el repo como un documento corto que un humano de soporte pueda aplicar: qué pasa con 12 de 30 USDC, qué pasa cuando dos de tres ítems de línea están cubiertos, qué se le dice al comprador, y cuál es la contrapartida declarada, es decir qué te cuesta esta política y por qué aceptas ese costo. Después rutea por ella el pedido underpaid sembrado por el harness. Si tu memo no puede justificarle la política a un comprador escéptico y a un contador escéptico al mismo tiempo, no está terminado. No hay respuesta de referencia; la barrera de aceptación es el enlace y la explicitud, no estar de acuerdo con la mía.

Aceptación: un reembolso aparece en el libro mayor atado a la firma del pago original, lleva el origen en su memo on-chain, y se resuelve desde su propia reference key en una sola búsqueda de firmas; el intento de doble reembolso lanza; un pedido underpaid rutea a tu política declarada, no a un despacho silencioso; el memo de política existe y nombra su contrapartida.

![Línea de tiempo de un pedido de 30 USDC pagado con 12: el conciliador lo marca underpaid, se abre una ventana de 60 minutos para completar el pago, y el pedido o se completa o rutea a un reembolso con guarda.](assets/v08-timeline.webp)

## Checkpoint: el back office, completo

Si el harness se niega a ponerse en verde, los sospechosos de siempre, en orden: el conciliador clasificando con matemática de floats en vez de unidades base (la comparación rutea mal en silencio los montos al límite; conoces este bug del módulo 2), la guarda de finality comprobando `confirmed` en vez de `finalized` (el caso de pago fresco del harness atrapa exactamente esto), o el memo del reembolso sin la firma de origen, así que la aserción de enlace falla aunque el dinero se haya movido. Los tres son arreglos de cinco minutos una vez nombrados. Cuando pase, detente en lo que de verdad construiste, porque es más raro de lo que parece: un flujo de reembolsos para rieles que no traen ninguno, con un rastro de papel más fuerte que el que te dan los rieles de tarjeta. Cada reversión nombra su causa on-chain. Muy pocos comercios en producción sobre estos rieles pueden decir eso hoy. Tú puedes.

Y con eso, el back office está completo: verificar, ingestar, conciliar, reembolsar. El dinero que entra está demostrado, el dinero que sale está con guarda y enlazado, y cada pago no exacto aterriza en una política que elegiste en voz alta. Lo que la tienda todavía no puede hacer es volver el mes que viene. Las ventas sueltas son todo el negocio hasta ahora, y la mejor idea de Wavelength es un club del disco del mes, lo que quiere decir ingresos recurrentes, lo que sobre rieles push levanta una pregunta genuinamente picante: ¿cómo le facturas a alguien todos los meses sin tomar nunca custodia de sus fondos? Ese es el próximo módulo. Nos vemos allá.
