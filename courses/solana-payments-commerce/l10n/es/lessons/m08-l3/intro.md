# El gate de producción: una checklist de salida al aire que va en serio

## Resumen

Ahora puedes tomar pagos sin SOL del comprador (la penúltima lección) y sin conectividad (la lección pasada). Checkout, verificador, back office, suscripciones, rampas, x402, gasless, la fila offline: cada pieza del stack de Wavelength existe y pasó su propia prueba de humo. Esta lección comprueba si la cosa entera es de verdad apta para abrir.

Aquí viene la parte incómoda. "Funciona" y "está listo" son afirmaciones distintas, y solo una de las dos se ha probado alguna vez. Cada peldaño se verificó en aislamiento, en el camino feliz, por ti, en un buen día. Nadie ha preguntado todavía qué pasa cuando el worker del webhook se muere un viernes en la noche, o cuando un pedido mayorista de $9,000 se liquida al mismo nivel de commitment que un disco de $12.

Así que hoy construyes el último artefacto antes del capstone: `prod-gate`, una auditoría puntuada que corre contra todo lo que tienes. Es una checklist que puedes fallar de verdad. Las fallas no se vuelven sentimientos; se vuelven tareas de arreglo con tu nombre encima, y cada una se cierra antes del módulo 9.

Haz esto ahora mismo, antes de cualquier teoría:

```bash
cd ~/wavelength && mkdir -p gate && npx tsx --version
```

Si falta `tsx`, instálalo en el repo que has estado construyendo todo el curso: `npm i -D tsx` (no hace falta pin; es un runner, no una superficie de API). Esa carpeta `gate/` vacía es donde va a vivir el veredicto sobre tus últimos siete módulos para el final del lab.

Cómo se reparte el trabajo hoy, dicho en voz alta: la teoría de abajo es de servicio completo, el lab te entrega el runner del gate y cuatro filas terminadas y te hace escribir el resto tú mismo, y el Challenge es criterio puro sin guía, porque decidir lo que "listo" quiere decir para tu propio stack es la habilidad que esta lección de verdad enseña. Este es el módulo 8. Los scaffolds se fueron.

## El gate

### Por qué una checklist y no una sensación

En 1935 el US Army Air Corps evaluó el Model 299 de Boeing, el avión que se volvió el B-17. Era la aeronave más sofisticada jamás construida hasta ese punto, y en su vuelo de evaluación se estrelló en el despegue, matando al piloto de pruebas, porque la tripulación olvidó liberar un seguro de los mandos. La respuesta del Ejército no fue "contratar mejores pilotos". Fue la checklist del piloto: una lista corta, aburrida y puntuada de cosas que tienen que ser ciertas antes de que las ruedas dejen el suelo. El avión era demasiado complejo para la competencia sola; la competencia más una checklist lo volaron por treinta años.

Tu stack de comercio acaba de cruzar la misma línea de complejidad. Ocho subsistemas, tres protocolos de pago, dos arreglos de fee payer, una fila offline que sostiene transacciones firmadas casi al portador. Ningún script de verify de una sola lección ve el conjunto. El gate es la comprobación del seguro de los mandos para la tienda.

Dos de esos ocho se quedan fuera de la auditoría del gate a propósito, así que dilo antes de que el diagrama lo implique. Los caminos de dinero del embed de la rampa corren sobre la infraestructura de Coinbase detrás de las llaves de Coinbase, así que su única pregunta con forma de gate, la higiene del secreto de CDP, se dobla hacia la fila de separación de llaves en vez de ganarse su propio artefacto. Y la puerta de x402/MPP se liquida en el mismo libro mayor del back office que las filas del back office ya auditan; su único punto abierto, el memo por llamada que falta en el camino del gate, ya está en la lista de tareas de arreglo que te llevaste de esa lección. Seis artefactos bajo auditoría, ocho subsistemas en el stack, y el capstone cablea todos.

Y lo que está en juego no es hipotético. Solana procesó más de un billón de dólares en volumen de stablecoins en 2025, la cifra con la que abre la documentación de pagos de solana.com (obtenida el 2026-08-23, la misma afirmación que fechó la lección inicial). Ese es el estanque al que tu tiendita de discos se está cableando. El dinero a esa escala no le importa que tu demo funcionara; encuentra el webhook que nunca monitoreaste y el nivel de commitment en el que nunca pensaste, y los encuentra a la peor hora posible. La realidad dura es: una checklist de salida al aire solo vale escribirla si puede fallar, y la tuya va a fallar hoy, al menos una vez, en tu primera corrida. Ese es el punto.

![Diagrama de seis artefactos del curso alimentando al auditor prod-gate, que emite un reporte puntuado que le pone gate a la entrada al capstone del módulo 9.](assets/v01-diagram.webp)

Las filas del gate se agrupan en cuatro familias: la verdad de la liquidación, la falla silenciosa, el manejo del dinero, y qué pasa cuando se rompe de todos modos. Recórrelas en orden; cada familia termina con la pregunta que van a hacer sus filas.

### La verdad de la liquidación: commitment escalado al valor

Cada camino de pago que construiste termina en el mismo momento: alguna línea de código decide que el dinero llegó y libera la mercancía. La primera familia del gate audita lo que esa línea comprueba de verdad.

La política viene directo de la guía oficial, y escala el commitment al valor. Confirmed es el commitment para la mayoría de los pagos: una confirmación optimista, alcanzada en bastante menos de un segundo, que en la práctica casi nunca se revierte. Finalized está reservado para pagos de alto valor o sensibles al compliance: te cuesta varios segundos extra de latencia, y en cambio la transacción queda más allá de todo rollback. Y processed es solo para la UI y puede descartarse durante forks, lo que quiere decir que nunca es, bajo ninguna circunstancia, un gate de liquidación. Muestra un spinner en processed si quieres. Despacha un disco sobre eso y puedes estar despachando un disco por una transacción que ya no existe.

Concretamente, para Wavelength: la venta del disco de $12 pone el gate en confirmed, porque hacer que cada comprador espere la finality para salvarte de una reversión que esencialmente nunca pasa es latencia gastada en nada. El pedido mayorista de $9,000 pone el gate en finalized, porque a ese tamaño los segundos extra son más baratos que la conversación con tu contador. Tú eliges el umbral; el gate solo exige que exista un umbral y que el código lo haga cumplir.

![Comparación de processed, confirmed y finalized: processed es solo para la UI y descartable en forks, confirmed es el gate de menos de un segundo para la mayoría de los pagos, finalized el gate más lento para los pedidos de alto valor.](assets/v02-comparison.webp)

Una sutileza de todo el stack que esta familia también atrapa: tu drain de fair-queue de la lección pasada liquida transacciones que se firmaron horas antes. El comprador dejó la feria hace mucho. Si alguna de esas ventas cruza tu umbral de alto valor, el drain tiene que sostenerla en finalized antes de marcar el pedido como despachado, porque no hay ningún comprador parado frente a ti para volver a pasar la tarjeta. Escribe la fila de modo que te fuerce a hacer grep de cada argumento de commitment en el código, no solo de los que te acuerdas de haber escrito.

### La falla silenciosa: las tuberías que se mueren sin hacer ruido

Un stack de pagos tiene dos clases de falla. Las ruidosas lanzan errores, te despiertan y se arreglan. Las calladas simplemente se detienen, y te enteras por un cliente. La segunda familia caza las calladas.

El webhook es el clásico. Tu back office despacha pedidos a partir de las entregas del webhook de Helius, y un webhook de Helius que falla por suficiente tiempo se desactiva automáticamente: fallas de entrega sostenidas y la plataforma deja de mandar, por diseño, para protegerse de martillar un endpoint muerto. Comportamiento razonable para una infraestructura. Catastrófico para una tienda sin alertas, porque desde afuera nada parece estar mal. Los checkouts se completan, el dinero llega on-chain, el libro mayor se llena de pagos verificados. El despacho simplemente se detiene calladamente, y la primera señal es un correo furioso.

Voy a admitir que esta es personal: una vez me enteré de que una fila de despacho estaba caída por el DM de un cliente, no por ningún dashboard, y la brecha llevaba dos días creciendo. Salir de eso a fuerza de reembolsos es exactamente tan divertido como suena. El arreglo es una alerta sobre la tasa de fallas del webhook, probada disparándola a propósito, más un sondeo de respaldo (construiste el verificador por sondeo en el módulo 4; es tu red de seguridad aquí) para que un webhook desactivado se degrade a despacho lento en vez de a ninguno.

![Diagrama de flujo que muestra fallas del worker del webhook acumulándose hasta que Helius desactiva automáticamente el webhook, después de lo cual los pagos siguen teniendo éxito pero el despacho se detiene calladamente, con dos mitigaciones marcadas, una alerta de tasa de fallas y un sondeo de respaldo.](assets/v03-flowchart.webp)

La misma familia cubre los reintentos y la idempotencia, porque una tormenta de reintentos es la gemela de la falla silenciosa: todo se ve bien mientras despachas doble. Antes de escribir la fila, ten clara una distinción, dado que decide lo que "seguro de reintentar" quiere decir en cada camino que tienes. Retransmitir la misma transacción firmada es inofensivo: la firma es la llave de deduplicación, y la red no va a procesar dos veces bytes idénticos mientras el blockhash viva. Construir y firmar una transacción nueva para la misma compra es una autorización nueva, y nada on-chain sabe que es "la misma" venta. Ese segundo caso es aquel para el que existen tus defensas, y ya las construiste. El libro mayor de pedidos indexa la idempotencia sobre la firma de la transacción, así que un evento de webhook repetido aterriza en una fila existente y no hace nada. La reference key de cada checkout quiere decir que un comprador que reintenta un pago estancado no puede pagar dos veces por un pedido, porque la segunda transacción lleva la misma reference y el verificador la empareja con un registro ya liquidado. Y el clasificador del drain de fair-queue rutea cualquier nonce gastado a `unsafe`, sin reenviarlo nunca — no porque la red aceptaría los bytes viejos (un nonce gastado se rechaza determinísticamente, y así ha sido desde el arreglo de 2022), sino porque un nonce avanzado quiere decir que esa venta puede haberse liquidado ya una vez, y el único reintento que queda es el de firma nueva: exactamente el cargo doble por autorización nueva que toda esta familia de defensas existe para detener.

Las filas del gate para esta familia no preguntan si construiste estas cosas. Te piden demostrarlas, ahora, repitiendo un evento real grabado y pegando la única fila de pedido resultante en el campo de evidencia. Una defensa que nunca has disparado es una hipótesis.

![Comparación de dos casos: retransmitir los mismos bytes firmados se deduplica por firma y es seguro, mientras que firmar una segunda transacción para esa venta es una autorización nueva que las defensas tienen que atrapar.](assets/v04-comparison.webp)

### El manejo del dinero: las llaves y el presupuesto de comisiones

Tercera familia, dos preocupaciones: quién puede mover el dinero, y qué te cuesta moverlo.

Las llaves primero, y la postura es la separación de poderes. El firmante de Kora que patrocina los checkouts gasless está acotado por su allowlist y sus reglas de validación; la fila del gate te pide demostrar la negación, repitiendo la petición fuera de la allowlist que armaste hace dos lecciones y mostrando el rechazo. La autoridad del nonce de la fila de la feria es su propia llave, no tu llave de tesorería, así que comprometer el puesto compromete una fila, no la tienda. Y la billetera receptora del comercio debería ser exactamente eso: una receptora, con su llave guardada en ningún lugar caliente, barrida con la cadencia que te deje dormir. Sé honesto sobre dónde te deja eso: si construiste la fila de la feria exactamente como se entregó su lección, hoy fallas esta fila, porque ese lab corrió deliberadamente la llave del comercio como fee payer y autoridad del nonce a la vez para mantener una laptop testeable, y lo dijo. El arreglo es mecánico, dado que la autoridad de una cuenta de nonce puede ser cualquier llave: acuña un keypair dedicado de autoridad del nonce, entrégale las cuentas del pool con la instrucción AuthorizeNonceAccount del system program (o recrea el pool bajo ella), guarda esa llave en la laptop del puesto, y mueve el receptor del comercio a frío. Si una sola llave filtrada puede simultáneamente drenar el paymaster, avanzar los nonces y vaciar la caja, el gate te falla, y debe hacerlo.

Ahora las comisiones, donde la contabilidad honesta importa más que la optimización. Tu piso de costo por venta es la comisión base, 5000 lamports por firma: irrelevante para el margen en cualquier venta con precio en dólares. El número que de verdad aparece en los libros es el patrocinio. Cada checkout gasless donde el comprador necesita una cuenta de token nueva te cuesta el mínimo exento de rent de 165 bytes — el número que devuelve `getMinimumBalanceForRentExemption(165)` en el cluster al que estás desplegando, que la lección de gasless te hizo leer en vez de memorizar — y el comprador puede reclamarlo después cerrando la cuenta; lo presupuestaste como gasto, no como préstamo, en la primera lección de este módulo, y la fila del gate simplemente comprueba que el presupuesto existe y que tiene un tope diario.

Corre la aritmética una vez para que el tope de la fila sea un número y no un encogimiento de hombros — es la servilleta de cien compradores de la lección que abrió el módulo a escala de feria: 200 ventas cuestan 0.001 SOL de comisiones base para el día entero (un pelo más una vez que las ventas patrocinadas de abajo cuenten su segunda firma), mientras que 50 primerizos gasless que necesitan cuentas de token de USDC nuevas cuestan alrededor de 0.074 SOL de rent patrocinado — 50 × los 1,488,440 lamports que devolvió `getMinimumBalanceForRentExemption(165)` en devnet el 2026-09-07, una tasa que baja por calendario, así que corre la consulta para tu propia cifra en vez de confiar en esta — unas setenta veces la línea de comisiones, con cada lamport del rent reclamable por los compradores y ninguno por ti. La línea de comisiones en tus libros es en realidad una línea de patrocinio, y el tope diario de la fila del gate es lo que evita que una ola guionada de compradores primerizos falsos convierta tu presupuesto de crecimiento en su granja de rent.

Después está la comisión que agregas a propósito. Bajo congestión, una transacción que lleva solo la comisión base puede quedarse en la fila y caerse; una priority fee pequeña le compra a tu pago un lugar en la línea. Las priority fees se cotizan por unidad de cómputo, así que la receta tiene dos diales: un límite de unidades de cómputo cerca de lo que la transacción de verdad usa (una transacción puede pedir hasta 1.4M CU; tu transferencia de token más el memo usa una astilla minúscula de eso, y cada CU que reservas sin necesidad multiplica el precio que pagas), y un precio por CU leído del mercado reciente sobre las cuentas que tocas. Esta es la receta entera, la única caja que este curso entrega:

```typescript
// gate/fee-recipe.ts
import { address, createSolanaRpc } from '@solana/kit';
import {
  getSetComputeUnitLimitInstruction,
  getSetComputeUnitPriceInstruction,
} from '@solana-program/compute-budget';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// The one-box recipe: two instructions, one RPC read, three constants.
const CU_LIMIT = 60_000; // generous ceiling for a token transfer + memo
const FLOOR_MICROLAMPORTS = 1_000; // never send a bare base-fee tx
const CAP_MICROLAMPORTS = 1_000_000; // never let a spike eat the margin

export async function paymentFeeInstructions(writableAccounts: string[]) {
  const recent = await rpc
    .getRecentPrioritizationFees(writableAccounts.map(address))
    .send();

  const fees = recent
    .map((entry) => Number(entry.prioritizationFee))
    .sort((a, b) => a - b);

  const p75 = fees.length > 0 ? fees[Math.floor(fees.length * 0.75)] : 0;
  const microLamports = Math.min(Math.max(p75, FLOOR_MICROLAMPORTS), CAP_MICROLAMPORTS);

  return [
    getSetComputeUnitLimitInstruction({ units: CU_LIMIT }),
    getSetComputeUnitPriceInstruction({ microLamports }),
  ];
}
```

Nota de instalación para el único paquete nuevo en este workspace: `npm i @solana-program/compute-budget@0.16.0` en checkout-txreq, el workspace cuyo builder ensambla cada transacción de pago sobre la que viajan estas instrucciones — el mismo 0.16.0 que fijó el workspace de gasless antes en este módulo, un solo número de compute-budget para el repo entero. El pin está haciendo trabajo real. Este workspace corre kit ^6.10, y 0.16.0 es el último minor de compute-budget cuyo rango de peers acepta un kit v6 (verificado contra npm el 2026-08-31: 0.16.0 hace peer con kit ^6.4.0, 0.17.0 saltó a ^7 y el actual 0.18.0 a ^8, así que agarrar latest aquí rompe la instalación). Regla de frescura como siempre: vuelve a comprobar el rango de peers el día en que construyas.

![Versión anotada del código de la receta de comisiones, etiquetando el límite de CU, el piso, el tope y la lectura del mercado de comisiones recientes, con la salida siendo dos instrucciones de compute-budget por transacción de pago.](assets/v05-annotated-code.webp)

Una nota hacia adelante sobre ese campo `microLamports`, porque es la parte de esta receta que no sobrevive al cambio de formato. Una priority fee de v0 es un *precio*: micro-lamports por unidad de cómputo, facturados contra las unidades que de verdad consumes. El formato de transacción v1 lo reemplaza con un *total*: lamports absolutos para la transacción entera, llevados en la config propia del mensaje en vez de en una instrucción de ComputeBudget. Esas son dimensiones distintas, así que cualquier serie de comisiones que promedie entre las dos sin convertir no tiene sentido — multiplica un precio de v0 por el límite de CU y divide entre 1,000,000 para ponerlo en las unidades de v1. Dos cosas viajan con ese cambio. En v1 las instrucciones de compute-budget de arriba se vuelven no-ops, aceptadas e ignoradas mientras siguen costando un hueco de instrucción y unas 150 CU. Y v1 no entrega ningún límite de recursos por defecto, así que una transacción que los omite aterriza y después falla, donde v0 habría caído de vuelta a 200,000 CU. Nada de esto vuelve equivocada a la receta de esta página: construye transacciones v0 y es correcta para ellas. Te dice lo que vas a tener que volver a derivar en vez de copiar cuando te muevas.

Ahora la frontera, nombrada en voz alta porque fingir profundidad aquí sería peor que la superficialidad. Esta receta es deliberadamente superficial. La estrategia real de aterrizaje de transacciones es una disciplina: estimación de comisiones por cuenta, curvas de reintento, bundles de Jito, ciencia del compute budget, y el curso Client-Side Mastery es dueño de todo eso de punta a punta, del mismo modo que este curso es dueño del comercio. La contrapartida que estás aceptando es exacta: sobre-provisiona la comisión y quemas margen en cada venta; sub-provisiona y las transacciones se caen bajo congestión, y afinar esa curva como se debe es ciencia del aterrizaje, no comercio. La receta de una caja aterriza transacciones de pago de forma confiable y paga un poco de más. Para una tienda, ese es el lado correcto del canje. Cuando tu volumen haga que el pago de más duela, ese curso es a donde vas; el mismo traspaso aplica a la indexación a escala, a la que tu configuración de webhook-más-sondeo con el tiempo le va a quedar chica.

### Cuando se rompe de todos modos: el playbook y los libros

Última familia. Todo lo de arriba reduce las probabilidades de un incidente; nada las reduce a cero. La diferencia entre una mala hora y un mal trimestre es si la respuesta se escribió mientras estabas tranquilo.

Un playbook de incidentes para una tienda de este tamaño cabe en una página, y la fila del gate comprueba que existan cuatro cosas en él. Un interruptor de congelación: un comando o una bandera que deja de aceptar pagos nuevos mientras sigue registrando los que ya están on-chain en vuelo, porque los peores incidentes son los que sigues vendiendo hacia adentro. Una escalera de severidad: qué haces cuando el despacho se atrasa (cae de vuelta al sondeo), frente a cuando el paymaster se está drenando (congela el patrocinio, mantén el checkout normal), frente a cuando sospechas una filtración de llave (congela todo, rota, barre). Una línea de comunicación: la oración que les publicas a los compradores, pre-escrita, porque no vas a escribir una oración tranquila durante el incidente. Y un dueño: a quién le suena el teléfono. Si la respuesta a alguna de estas vive ahora mismo en tu cabeza, no existe.

![Árbol de decisión del playbook de incidentes: un despacho atrasado rutea a un respaldo por sondeo, un paymaster drenándose a una congelación del patrocinio, una filtración de llave sospechada a rotación y comunicación a los compradores, cada uno terminando en postmortem.](assets/v06-flowchart.webp)

Y después la fila que no puedo poner verde por ti, porque el ecosistema no puede. Cuando tu tienda esté en vivo, alguien tarde o temprano tiene que cerrar los libros: emparejar cada liquidación on-chain con un pedido, un reembolso, un tick de suscripción, y entregarle a un contador algo que reconozca. Ve a buscar un SaaS nativo de Solana de contabilidad y reportes para comercios que haga esto y, a la fecha de esta construcción, el 2026-08-31, no vas a encontrar uno hecho para ese propósito. Ese es un hueco de mercado observado, declarado con su fecha porque los mercados jóvenes se mueven: existen herramientas generales de impuestos cripto, existen dashboards de exchanges, el dashboard de Stripe cubre las ventas que se liquidaron en los rieles propios de Stripe y nada más. Tu libro mayor on-chain de Solana Pay y x402 es tuyo para conciliar.

La postura honesta del gate no es ni la desesperación ni la negación. Ya tienes la materia prima: cada venta de tu libro mayor lleva una reference key o un id de factura en el memo, emparejado con una firma de transacción por el verificador. Así que la fila exige la respuesta intermedia que sí puedes entregar, una exportación de conciliación: un comando que emite un CSV fechado de liquidaciones unidas a pedidos, la cosa que le entregarías a un contador hoy y la cosa que vas a diferir contra el SaaS real el mes en que alguien finalmente lo construya. Un hueco declarado con una fecha y un rodeo es un plan. Un hueco que supusiste que un proveedor tenía cubierto es la emergencia del año que viene. (Si eres el lector que ha estado esperando una idea de startup todo este curso: esta fila es una.)

## Lab: corre el gate

El lab construye `prod-gate` y lo corre contra tu stack. Te llevas el runner y cuatro filas terminadas; las filas restantes son tuyas para escribirlas a partir de las cuatro familias de arriba. Presupuesta la mayor parte de tu tiempo para el paso 5, que es la auditoría de verdad.

1. Define la forma de la fila y las primeras cuatro filas en `gate/rows.ts`. Estas cuatro son la semilla; fíjate en que cada campo `evidence` exige un artefacto que puedas pegar, nunca una sensación:

```typescript
// gate/rows.ts
export type Rung =
  | 'checkout'
  | 'verifier'
  | 'backoffice'
  | 'club-billing'
  | 'gasless-checkout'
  | 'fair-queue'
  | 'stack-wide';

export interface GateRow {
  id: string;
  rung: Rung;
  question: string;
  evidence: string;
}

export const rows: GateRow[] = [
  {
    id: 'commit-policy',
    rung: 'stack-wide',
    question:
      'Does every settlement path gate on confirmed, escalate to finalized above your high-value threshold, and never treat processed as settlement?',
    evidence:
      'grep every commitment argument in the verifier and the fair-queue drain; list each occurrence with its value and the payment path it guards',
  },
  {
    id: 'webhook-monitoring',
    rung: 'backoffice',
    question:
      'Do you alert on the webhook failure rate before sustained failures can auto-disable the webhook?',
    evidence:
      'show the alert rule and the last test alert it fired; a dashboard nobody watches is a fail',
  },
  {
    id: 'idempotency',
    rung: 'backoffice',
    question:
      'Does a replayed webhook event, or a client retry of the same checkout, fulfill exactly once?',
    evidence:
      'replay one recorded event twice against the orders ledger and paste the resulting single order row',
  },
  {
    id: 'fee-recipe',
    rung: 'checkout',
    question:
      'Does every payment transaction carry the one-box priority-fee pair (CU limit near real usage, floored and capped CU price)?',
    evidence: 'decode one live checkout tx and show both compute-budget instructions',
  },
];
```

Checkpoint: `npx tsx -e "import('./gate/rows.ts').then((m) => console.log(m.rows.length))"` imprime `4`.

2. Escribe tú mismo las filas restantes, a partir de las cuatro familias. Apunta a un total de diez a catorce. Como mínimo el conjunto tiene que cubrir: el presupuesto de patrocinio con un tope diario en el peldaño gasless (el rent de la ATA como gasto presupuestado, dimensionado desde una lectura de rent en vivo y no desde una constante recordada), la repetición de la negación fuera de la allowlist para el firmante de Kora, la separación de llaves entre paymaster, autoridad del nonce y receptor, la exclusión `unsafe` del nonce gastado en el drain de fair-queue, el playbook de incidentes de cuatro partes, la exportación de conciliación, una alerta de falla del tick de facturación para club-billing, y las reglas de hosting del blink — `Access-Control-Allow-Origin: *` en cada ruta de action Y en `actions.json`, con `actions.json` servido desde la raíz del dominio. Esa última fila existe porque el blink es la única superficie cuya falla es invisible desde tu terminal: curl no hace cumplir CORS y un montaje en sub-ruta le responde a curl perfectamente bien, así que los dos defectos dan limpio en las pruebas locales y no renderizan nada en una billetera. La evidencia ya es regenerable — los dumps de headers de la checklist de listo-para-el-registro de la lección del blink — y el ensamblaje es exactamente cuando se rompe, porque montar varias apps en un solo servidor es el momento en que un archivo montado en la raíz deja de estar en la raíz. Mantén cada `question` respondible con sí o no, y cada campo `evidence` un artefacto que se pueda pegar.

3. Escribe el runner. Se niega a saltarse filas, puntúa lo que lee, y convierte cada falla en una tarea de arreglo:

```typescript
// gate/run.ts
import { readFileSync, writeFileSync } from 'node:fs';
import { rows } from './rows.ts';

type Verdict = 'pass' | 'fail';

interface VerdictEntry {
  verdict: Verdict;
  note: string;
}

const verdicts: Record<string, VerdictEntry> = JSON.parse(
  readFileSync(new URL('./verdicts.json', import.meta.url), 'utf8'),
);

let failures = 0;
const reportLines: string[] = [];
const fixTasks: string[] = [];

for (const row of rows) {
  const entry = verdicts[row.id];
  if (!entry) {
    throw new Error(`no verdict recorded for row "${row.id}": the gate does not skip rows`);
  }
  reportLines.push(`${entry.verdict.toUpperCase().padEnd(4)}  ${row.id} (${row.rung}): ${entry.note}`);
  if (entry.verdict === 'fail') {
    failures += 1;
    fixTasks.push(`- [ ] ${row.id}: ${entry.note}`);
  }
}

const report = [
  `# prod-gate report, ${new Date().toISOString().slice(0, 10)}`,
  '',
  ...reportLines,
  '',
  failures === 0
    ? 'GATE: GREEN. Proceed to the capstone.'
    : `GATE: RED. ${failures} failing row(s). Every fix-task below closes before m09.`,
  '',
  '## Fix tasks',
  ...(fixTasks.length > 0 ? fixTasks : ['(none)']),
  '',
].join('\n');

writeFileSync(new URL('./report.md', import.meta.url), report);
console.log(report);
```

Fíjate en lo que el runner no es: no es verificación automatizada. No puede hacerle grep a tu verificador ni disparar tu alerta. El archivo de veredictos eres tú, bajo juramento, con la evidencia adjunta. El trabajo del runner es contabilidad y negativa: ninguna fila sin responder, ninguna falla sin una tarea de arreglo.

Checkpoint, y demuestra la mitad de la negativa antes de confiar en la mitad de la contabilidad: pon un `{}` vacío en `gate/verdicts.json` y corre `npx tsx gate/run.ts`. Tiene que lanzar `no verdict recorded for row "commit-policy"` y no escribir ningún reporte. Un runner que renderiza un reporte parcial no es un gate.

4. Cablea la receta de comisiones antes de auditar su fila. Instala `@solana-program/compute-budget@0.16.0` (la razón del pin está en la teoría de arriba), suelta `gate/fee-recipe.ts` ahí, y antepón la salida de `paymentFeeInstructions(...)` a la construcción de la transacción del servidor de checkout, pasándole las cuentas escribibles que toca el pago (la ATA del comercio es la disputada en un día de ventas ocupado). Concretamente, la edición vive en un solo lugar: dale a `finalizeTransaction` un array opcional `extraInstructions` que ponga por delante de las instrucciones de pago, y haz que el servidor de checkout pase ahí las dos instrucciones de compute-budget. Cada superficie que reusa el builder, el blink y el checkout patrocinado incluidos, hereda la perilla; fíjate en que la estimación de comisiones de Kora ahora también ve la priority fee, que es exactamente el caso que el tope del patrocinador de esa lección existe para absorber. Vuelve a correr la prueba de humo del checkout del módulo 3 para confirmar que nada se rompió.

5. Ahora audita. Para cada fila, en orden, realiza de verdad la acción de evidencia: corre el grep, repite el evento, arma y repite la petición de patrocinio fuera de la allowlist, decodifica la transacción en vivo, abre el playbook. Registra veredictos honestos en `gate/verdicts.json`:

```json
{
  "commit-policy": { "verdict": "pass", "note": "verifier gates on confirmed; drain escalates to finalized over 500 USDC; no processed anywhere" },
  "webhook-monitoring": { "verdict": "fail", "note": "no alert on webhook failure rate yet" },
  "idempotency": { "verdict": "pass", "note": "replayed event produced one order row (signature-keyed)" },
  "fee-recipe": { "verdict": "fail", "note": "checkout txs still ship with no compute-budget instructions" }
}
```

Esa muestra es de mi propia primera corrida contra una construcción de referencia de este stack, y sí, salió dos de cuatro. Un número de ahí merece una etiqueta de advertencia: el umbral de escalada a finalized de 500 USDC es la elección de esa construcción, no una recomendación del curso; el Challenge te hace derivar el tuyo a partir de lo que un pago caído de verdad te cuesta. Si tu primera corrida sale toda verde, la explicación más probable es una calificación generosa, no un stack impecable; vuelve a leer los campos de evidencia y sé más malo.

6. Córrelo:

```bash
npx tsx gate/run.ts
```

Checkpoint: deberías ver el reporte puntuado en la terminal y en `gate/report.md`, terminando en `GATE: GREEN` o en `GATE: RED` con una lista de tareas de arreglo. Un gate RED aquí es un lab aprobado. El lab prueba la auditoría, no el stack.

7. Cierra el loop. Trabaja las tareas de arreglo (las dos de arriba son una tarde: una regla de alerta, una edición de servidor que ya cableaste en el paso 4), vuelve a auditar solo las filas falladas, y vuelve a correr hasta GREEN. Guarda cada `report.md` fechado; el capstone abre leyendo el tuyo más reciente.

![Comparación de los estados de veredicto de prod-gate: pass guarda la evidencia, fail fuerza una tarea de arreglo con nombre, una fila saltada hace que el runner lance un error, y solo un gate todo verde abre el capstone.](assets/v07-comparison.webp)

## Challenge

Ninguna caminata guiada en este. Tres juicios, escritos en `gate/DECISIONS.md`:

1. Fija tu umbral de alto valor: el valor de pago por encima del cual Wavelength escala de confirmed a finalized. Defiende el número en tres oraciones usando tu lista de precios real (discos, suscripciones, mayoreo) y lo que una reversión en cada nivel te costaría en dinero y en confianza.
2. Escribe una fila del gate que esta lección nunca mencionó, sacada de un modo de falla específico de tu construcción del stack (cada implementación deriva; la tuya tiene un punto blando que la mía no). Fila completa: id, peldaño, pregunta de sí o no, evidencia que se pueda pegar.
3. Argumenta contra tu propio gate: nombra la falla con más probabilidad de lastimarte que ninguna fila puede atrapar, y di por qué una checklist no puede sostenerla. Si no se te ocurre nada, la fila de nonces durables y las palabras "instrumento al portador" son un lugar por donde empezar a pensar.

Aceptación: tu `report.md` final está GREEN con cada tarea de arreglo cerrada, `DECISIONS.md` sostiene los tres juicios, y la fila que escribiste tú aparece puntuada en el reporte.

## Antes de dar vuelta el cartel

Publica tu reporte puntuado y tu fila de autoría propia en el canal del curso, y lee las filas de otras dos personas antes de comparar veredictos; las filas que otros constructores inventan para sus propios puntos blandos son la mejor auditoría gratis que tu stack va a recibir. Si tu gate atrapó algo sobre lo que esta lección nunca te advirtió, eso es el sistema funcionando, y quiero que me lo cuentes.

El gate está verde y las tareas de arreglo están cerradas. Eso hace de esta la última lección que trata al stack como partes. El próximo módulo es el capstone: cablear cada peldaño de la escalera en una sola tienda corriendo y ver un journey de comprador completo, navegar, pagar, despachar, conciliar, corriendo de punta a punta. Hiciste la inspección. Ahora abre las puertas.
