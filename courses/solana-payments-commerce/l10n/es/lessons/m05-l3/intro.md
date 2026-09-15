# Dunning, ciclo de vida, y quién más hace esto

## Resumen

La lección pasada el club empezó a facturar de verdad: muchos suscriptores, de forma no custodial, sobre el programa oficial Subscriptions, con cada pull exitoso conciliado en el libro mayor del backoffice como cualquier otro pago. Esa máquina funciona de maravilla cuando la ATA del suscriptor tiene fondos. Hoy una no la tiene. El pull mensual dispara, la transferencia del delegado pega contra una cuenta de token vacía, y la renovación falla. En los rieles de tarjetas es aquí donde empieza el dunning: el calendario de reintentos, el email de tarjeta actualizada, la escalera de gracia que corre toda empresa de SaaS. En estos rieles agarras ese playbook y tu mano se cierra en el aire. No hay tarjeta que reintentar ni banco al que hacerle dunning. No puedes sacar dinero a la fuerza de una billetera que no controlas.

Entonces, ¿qué hace un sistema adulto de facturación no custodial con una renovación fallida? La respuesta que el operador más visible del ecosistema usa en sus propias facturas: nada automático. Los hallazgos primero:

- **Retry-never es la política, no un hueco.** Helius factura sus propias suscripciones sobre este mismo programa, y su postura documentada es que una renovación fallida no se reintenta automáticamente contra la billetera. Se vuelve una factura abierta ordinaria que el suscriptor puede liquidar después.
- Entregas **dunning-loop**: una máquina de estados del backoffice sobre el libro mayor de facturación. Los resultados del pull manejan las transiciones: el éxito avanza el ciclo, la falla abre una factura, la liquidación reanuda, la cancelación encola la recuperación de rent.
- **RevokeAbandonedSubscription y RevokeAbandonedDelegation** cierran las PDAs muertas de suscripción y de delegación y devuelven sus lamports de rent al pagador registrado. Los acuerdos cancelados dejan de costarte dinero.
- El panorama de proveedores de 2026, negativos incluidos: los pay links de MoonPay Commerce soportan suscripciones, Stripe Billing soporta suscripciones en stablecoin, y Sphere no tiene ningún producto recurrente.

Arranca el reloj antes de leer, porque la corrida en solitario de esta lección es la espera más larga del curso en reloj de pared y es una espera que no puedes comprimir. La corrida necesita dos ciclos de facturación completos, una hora es el piso que permite la unidad `periodHours` del plan, así que dos ciclos son dos horas de tiempo real. Ponla en marcha ahora y lee la teoría mientras corre. Desde la carpeta `subscriptions/` de la lección pasada:

1. Crea el plan id 2 con `periodHours: 1n` (el Challenge explica por qué tiene que ser un segundo plan y no una edición: los términos del plan son inmutables una vez que alguien se suscribe).
2. Suscríbele tu oyente de prueba fondeado, por el mismo flujo de subscribe de la lección pasada, y asegúrate de que su dirección sea la que está en `subscribers.json`.
3. Apunta `04-pull.ts` al plan nuevo — su `PLAN_ID` está hardcodeado a `1n` desde la lección pasada — y después arranca el crank y déjalo corriendo: `CLUB_MINT=<your devnet mint> npx tsx 05-crank.ts`.

Para cuando llegues al Challenge, el ciclo uno ya debería haber facturado y el ciclo dos debería estar cerca, y el drenaje va entre el ciclo dos y el ciclo tres. Si prefieres leer primero y correr después, está bien — solo que sepas que estás eligiendo quedarte dos horas sentado al final en vez de durante.

Mientras tanto, demuestra la línea base: desde esa misma carpeta `subscriptions/`, su directorio de trabajo para cada comando, vuelve a correr su barrera:

```bash
npx tsx pull.test.ts
```

Deberías ver el pull aterrizar en el libro mayor como una factura conciliada. Esa fila del libro mayor es la materia prima de hoy: el dunning es lo que le pasa a las filas que nunca se escriben. Cómo se reparte el trabajo hoy, dicho en voz alta: la máquina de estados y su adaptador van trabajados, tecleados conmigo contra las seis transiciones, el harness de la barrera los demuestra, y la corrida completa de dos-ciclos-más-una-falla del final es en solitario, sin scaffold.

## La falla de la que tu libro mayor es dueño

### Nada a lo que hacerle dunning

Empieza por qué existe siquiera el playbook de los rieles de tarjetas. Los pagos con tarjeta son pagos pull contra un instrumento que la red puede volver a intentar: el comercio guarda un token de la tarjeta del comprador, y cuando un cargo rebota, el procesador puede presentarlo de nuevo mañana, y el jueves, y la semana que viene, escalando por lo que la industria llama educadamente una estrategia de dunning. Los reintentos funcionan ahí porque la falla suele ser transitoria del lado del instrumento: un límite que se reinicia, un sueldo que aterrizó, una tarjeta nueva en el archivo. La maquinaria para meter la mano en la cuenta del comprador existe, así que reintentar es barato y muchas veces funciona.

Ahora mira lo que tu crank tiene en realidad: una PDA de delegación que permite una transferencia acotada, si y solo si el límite y el saldo del suscriptor lo cubren los dos. Cuando la ATA está vacía, un reintento cinco minutos después pega contra la misma cuenta vacía. Un reintento a las 3 a.m. pega contra la misma cuenta vacía. No hay emisor al que volver a presentarle ni instrumento permanente que pueda haberse recargado por detrás, solo una billetera cuyo dueño tiene que actuar. Cada reintento automático es una comisión de transacción gastada para volver a aprender un hecho que ya registraste. Mi primer instinto aquí seguía siendo escribir la fila de reintentos. Tenía el archivo abierto, `retry.ts`, la cadencia del cron esbozada en un comentario, antes de admitir que el archivo entero era un error de categoría importado de otro riel de pagos.

La política honesta invierte el valor por defecto, y aquí es donde importa el dogfooding. Helius corre su propia facturación de suscripciones sobre el programa Subscriptions de la Foundation, y su política para una renovación fallida es que el cargo no se reintenta automáticamente contra la billetera. El pull fallido se vuelve una factura abierta, al suscriptor se le avisa, y el dinero llega cuando recarga y liquida. Retry-never. No retry-with-backoff, no retry-thrice-then-flag. La renovación se convierte de un pull automatizado en una cuenta por cobrar ordinaria, que es algo que tu backoffice ya sabe manejar, porque el módulo 4 le enseñó a emparejar los pagos entrantes contra los pedidos abiertos.

![Los rieles de tarjetas reintentan porque las fallas suelen ser transitorias y el procesador puede volver a presentar, mientras que una billetera vacía sigue vacía hasta que el dueño actúa, así que la renovación se vuelve una factura.](assets/v01-comparison.png)

### La máquina, y dónde vive

Si los reintentos se fueron, lo que queda es estado. Una suscripción en cualquier momento está en exactamente uno de tres lugares: **active**, facturando normal; **open-invoice**, una renovación falló y hay una cuenta por cobrar pendiente (este es tu estado de gracia, el período donde la política de acceso es tuya para elegir); o **cancelled**, muerta por elección o por la blockchain. Lo que la mueve entre ellos es una lista corta de eventos: un resultado de pull de club-billing, una liquidación, un cancel explícito.

Fíjate dónde corre esta máquina. No on-chain. La blockchain te da la primitiva del pull y sus topes duros, y va a seguir dándote exactamente eso; no tiene opinión sobre períodos de gracia, emails de dunning, ni sobre si un suscriptor atrasado mantiene el acceso tres días. El único lugar donde controlas el estado es tu propio libro mayor, así que la falla y el ciclo de vida viven ahí, como datos. Esta es la tesis silenciosa de toda la segunda mitad de este curso: la blockchain es la capa de liquidación, y las decisiones de producto son filas que escribes. Un pull o tiene éxito, porque el límite delegado y el saldo lo cubrieron los dos, o no lo tiene, y una falla irrecuperable no tiene a dónde auto-reintentar, así que aterriza en tu libro mayor como una factura y un estado.

Las transiciones, exhaustivamente, porque lo exhaustivo es el punto de una máquina de estados:

- **active + pull ok** avanza el ciclo: una fila de factura pagada, el mismo camino de conciliación que la lección pasada.
- **active + pull insufficient-funds** mueve a open-invoice y escribe la fila de la factura. No se agenda ningún reintento. No hay efecto de reintento que agendar.
- **active + pull cancelled** (la blockchain reporta la suscripción revocada o la razón canceled de la guarda) refleja la realidad: marca cancelled, encola la recuperación de rent.
- **open-invoice + pull** se rechaza de plano. El crank no debe ni intentarlo; la máquina de estados lanza. Esto es retry-never como invariante y no como comentario.
- **open-invoice + settle** reanuda: la factura se liquida, la suscripción vuelve a active, y el próximo ciclo factura normal.
- **any + explicit cancel** marca cancelled y encola un RevokeAbandoned para que el rent vuelva a casa.

![Tres estados, active, open-invoice y cancelled, con los resultados del pull, la liquidación y el cancel explícito manejando las transiciones; un pull contra open-invoice se rechaza de plano y la cancelación encola la recuperación de rent con RevokeAbandoned.](assets/v02-flowchart.png)

### Qué concede la gracia en realidad

El estado open-invoice tiene un segundo nombre en las transiciones de arriba, gracia, y la gracia es una decisión de producto y no una técnica. La blockchain no pausó nada cuando el pull falló. Tu libro mayor cambió una fila. Si el suscriptor igual recibe el disco de este mes es enteramente una cuestión de qué hace tu código de despacho cuando lee esa fila, lo que quiere decir que tienes que decidir, por escrito, qué vive un suscriptor atrasado. La política del club, y la que asume el lab: el acceso continúa durante la ventana de gracia. El período perdido se despacha, la factura por él queda abierta, y se confía en que el suscriptor liquide. Eso se lee generoso hasta que notas que también es la opción barata: pausar el despacho a mitad de ciclo quiere decir construir una pausa, y recuperar el acceso a bienes digitales que ya entregaste quiere decir construir una mentira. Un negocio con costo marginal real por período, digamos un alquiler de hardware, trazaría la línea distinto y pausaría en la primera falla. Cualquiera de las dos políticas está bien. Una política sin decidir no, porque sin decidir en la práctica quiere decir lo que sea que tu código de despacho haga por casualidad, descubierto por un cliente.

La historia de las notificaciones es la otra mitad de la gracia, y sobre rieles retry-never deja de ser una cortesía. En los rieles de tarjetas a un suscriptor se le puede hacer dunning hasta devolverlo a la salud sin que lea nunca un email. Aquí el suscriptor es el único actor que puede arreglar la falla, así que el mensaje es el mecanismo de recuperación. Tres momentos alcanzan. En la falla: qué pasó, qué se debe, y el enlace de pago que lleva la reference key de la factura, porque un mensaje sin un camino de liquidación adjunto es solo malas noticias. Un recordatorio con una cadencia declarada mientras la factura envejece. Un aviso final cuando se acerca el horizonte de abandono, nombrando la fecha y qué pasa después de ella. Mándalas desde los efectos del libro mayor, no desde el crank: el efecto open-invoice es el disparador natural, lo que mantiene la lógica de notificación fuera de la máquina de estados y dentro de los consumidores, que es donde van los efectos secundarios.

### Liquidar y reanudar, sobre maquinaria que ya es tuya

Aquí es donde la escalera de artefactos te paga de vuelta. Una factura abierta necesita una forma de que la paguen, y ese camino entero lo construiste en el módulo 4. Dale a la factura una reference key fresca al crearla, exactamente como un pedido de checkout. Dile al suscriptor qué debe y dónde, con esa reference adjunta. Cuando recarga su billetera y paga, el pago lleva la ficha de reclamo, tu conciliador la encuentra con el mismo recorrido de búsqueda-de-firma-más-verificador que cualquier venta, y el resultado conciliado se vuelve un evento settle hacia la máquina de estados. Reanudar es `reconcileOrder` apuntado a una factura en vez de a un pedido, alimentando un tipo de evento más a un reducer.

Bueno, una arruga honesta. El pago de liquidación es un pago push simple del suscriptor, no un pull de delegado, así que no toca en absoluto la ventana de facturación de la suscripción. Tu próximo pull agendado igual dispara con la cadencia propia del plan. Decide a propósito si una liquidación a mitad de ciclo cubre solo el período perdido (la lectura simple, y la que codifica el lab) o además corre el ancla de la facturación futura; cualquiera de las dos es defendible, pero el libro mayor tiene que decir cuál corres.

Y nombra la contrapartida de frente, porque retry-never es honesto pero no es gratis. Una renovación fallida no se cura sola. Los ingresos que los rieles de tarjetas habrían recuperado calladamente en el reintento del martes ahora quedan como una cuenta por cobrar hasta que un humano actúe, así que tu camino de liquidación y tu historia de notificaciones dejan de ser un lujo y se vuelven la diferencia entre un estado de gracia y una máquina silenciosa de churn. Estás canjeando la automatización de recuperación de ingresos por cero custodia y cero cargos sorpresa. Para un club de discos cuyos suscriptores eligieron los rieles cripto a propósito, ese canje se lee bien. Para un negocio cuyo margen depende de la recuperación pasiva, es un costo real, y pretender lo contrario es como esta política se gana mala fama.

![Una factura abierta lleva una reference key fresca; el pago de recarga se encuentra por búsqueda de firma, se verifica, y se concilia en un evento settle que reactiva la suscripción.](assets/v03-diagram.png)

### Recuperar el rent

Cada suscripción que alguna vez existió dejó cuentas on-chain: la PDA de suscripción, las PDAs de delegación, cada una guardando un depósito exento de rent medido en lamports. Pequeño individualmente, del orden de un par de millones de lamports por cuenta, y deberías leer la cifra exacta de la cuenta en vez de confiar en el número redondo de nadie. Pero un club con churn de cientos de suscriptores es, calladamente, un casero que paga alquiler por apartamentos vacíos. La salida de ciclo de vida del programa existe exactamente para esto: la familia de instrucciones RevokeAbandoned. `RevokeAbandonedDelegation` sirve tanto para delegaciones fijas como recurrentes; `RevokeAbandonedSubscription` cierra una suscripción a plan abandonada. Cada una cierra la cuenta muerta y devuelve sus lamports de rent.

Dos detalles de los docs del propio programa son estructurales. Primero, el firmante es el **pagador registrado**, no el delegador, el delegatario ni el suscriptor. Si tu billetera de comercio patrocinó la creación de la cuenta allá en el flujo de subscribe, tu billetera de comercio es quien reclama el rent, que es la razón entera de que esto vaya en la fila de tu backoffice y no en un botón de cara al usuario. Segundo, las dos instrucciones **fallan si la Subscription Authority sigue viva** para esa cuenta. Un suscriptor que solo se atrasó todavía tiene una SA viva; abandonado quiere decir que la autoridad misma se fue o está obsoleta. Mientras la SA viva, el camino de revocación ordinario es la salida correcta en su lugar. Tu consumidor de limpieza comprueba esa precondición antes de enviar, y trata el caso de falla como rutina, no como un bug.

Lo que saca a la superficie la segunda contrapartida de la lección: reclamar el rent es dinero real de vuelta, pero solo después de que decides que un acuerdo está de verdad muerto, y esa decisión es irreversible de una manera que los lamports no capturan. Una vez revocada, la cuenta de suscripción queda cerrada y un suscriptor que vuelve tiene que suscribirse otra vez desde cero: ceremonia de firma nueva, cuenta nueva, fricción de onboarding nueva. (El programa sí trae un `resumeSubscription` on-chain para una cancelación que el suscriptor agendó y después lamentó antes del revoke, con la guarda del vencimiento que observó al firmar, pero esa es otra puerta, más angosta; no puede resucitar una cuenta cerrada.) Reclama una semana después de un pull fallido y convertiste a un cliente en estado de gracia en un problema de re-adquisición para ahorrar dos millones de lamports. Trata el abandono como una transición explícita y meditada: en la política del club, una factura abierta que envejece más allá de un horizonte declarado, o un cancel explícito, y nada más blando.

![Un pull fallido pasa por la gracia y los recordatorios hasta un horizonte declarado o un cancel explícito, después del cual RevokeAbandoned devuelve el rent; reclamar durante la gracia fuerza un re-subscribe completo.](assets/v04-timeline.png)

### Quién más hace esto

Aleja la cámara del libro mayor de Wavelength hacia el mercado, porque construir-o-comprar es una pregunta real y 2026 por fin le da respuestas reales. Imagínate el panorama de la facturación recurrente como tres puestos de una feria callejera, cada uno vendiendo algo distinto, y uno de ellos, importante, no vendiendo lo que su cartel te haría suponer.

Antes del recorrido, una definición, porque toda la pregunta de construir-o-comprar gira sobre ella. Un merchant of record es la entidad que legalmente le vende al comprador: toma el pago a su propio nombre, es dueña de las obligaciones de reembolso y de disputa, maneja los impuestos donde los impuestos aplican, y te paga a ti después. Cuando tercerizas la facturación recurrente a un proveedor hosteado casi siempre estás comprando algún recorte de ese acuerdo, y el precio del recorte es que alguien se para entre tu cliente y tu dinero. La facturación no custodial es la esquina opuesta: tú eres el merchant of record, los fondos del suscriptor se mueven directo de su billetera a la tuya, y cada obligación que el proveedor habría absorbido, el dunning muy incluido, te toca construirla a ti, que es lo que este módulo ha sido. Ninguna de las dos esquinas es la opción adulta en general. La jugada adulta es saber cuál estás corriendo, porque los modos de falla difieren: un proveedor puede retener o congelar tus payouts, mientras que tus propios rieles pueden fallar una renovación sin nadie más que tú posicionado para notarlo.

![Un merchant of record vende a su propio nombre y es dueño del dunning y de los reembolsos, aceptando retenciones de payouts; la facturación no custodial mueve los fondos de billetera a billetera y te deja cada obligación a ti.](assets/v05-comparison.png)

**MoonPay Commerce**, la plataforma antes conocida como Helio, vende checkout hosteado: sus pay links soportan suscripciones, así que un comercio puede levantar facturación recurrente sin ninguna integración de programa. El puesto está concurrido; el resumen de solana.com de abril de 2026 reporta a MoonPay Commerce en más de cuarenta millones de dólares en volumen de pagos únicos desde su lanzamiento de octubre de 2025, 88 por ciento de eso sobre Solana. Fíjate qué mide ese número fechado, pagos únicos, lo que te dice que el checkout hosteado es la mitad probada y lo recurrente es el estante más nuevo encima. **Stripe Billing** vende el paquete adyacente a las tarjetas: suscripciones en stablecoin dentro del mismo producto de facturación que corre la mitad de las facturas de SaaS en internet, lo que quiere decir lógica de dunning, prorrateo y manejo de impuestos que no escribes, a cambio de que Stripe se siente entre tú y el riel. Y **Sphere** vende infraestructura de pagos, ramps, OTC, y el corredor PIX para liquidación instantánea sobre riel bancario, y aquí está el resultado negativo que vale más que la mayoría de los positivos: Sphere no tiene ningún producto recurrente. Nada de malo con Sphere; mucho de bueno para flujos de una sola vez. Pero suponer que todo proveedor de pagos ofrece facturación recurrente es precisamente cómo un equipo quema un sprint de integración descubriendo que la funcionalidad que dimensionó no existe. Verifica el soporte de recurrente por proveedor, por escrito, antes de que diseñes la arquitectura alrededor de eso.

La regla de decisión sale limpia. Construye sobre el programa de la Foundation, como hizo este curso, cuando quieres no custodial y on-chain: los fondos del suscriptor nunca se quedan con un intermediario, los límites los hace cumplir el programa, y el ciclo de vida es tuyo, que es exactamente por qué tuviste que escribir tú mismo la máquina de estados de esta lección. Agarra un proveedor cuando quieres un producto hosteado, adyacente a las tarjetas, y estás contento de heredar su política de ciclo de vida junto con sus emails de dunning. Qué mirar, si tomas el camino del proveedor: si el producto recurrente del proveedor es una primitiva de primera clase o un loop de pay links, quién tiene la custodia entre el cargo y la liquidación, y cuál es en realidad su política de renovación fallida, porque ahora sabes que es una política, no física.

![Cuatro opciones de facturación recurrente comparadas: el programa de la Foundation hace cumplir lo recurrente no custodial on-chain, MoonPay Commerce ofrece pay links de suscripción, Stripe Billing factura suscripciones en stablecoin, y Sphere no ofrece ninguna.](assets/v06-table.png)

## Lab: construye dunning-loop

Cinco pasos. Este artefacto es una máquina de estados pura sobre datos del libro mayor, lo que te compra algo raro en este curso: cero dependencias nuevas y cero drama de workspaces. El workspace `dunning` no importa nada de `@solana/kit`, así que queda enteramente fuera de los dos pins, el de kit ^6 y el de kit ^7 (esos pins, congelados en 2026-08, viven donde viven las llamadas a la blockchain: checkout y ops en ^6, subscriptions en ^7). Corre con `tsx`, que está en las devDependencies del repo desde la lección de transfer-kit; si de algún modo estás empezando limpio, `npm install -D tsx typescript` lo pone de vuelta.

**Paso 1: el scaffold.** Un workspace nuevo al lado de los otros, y como es TypeScript puro la instalación es diminuta. Desde la raíz `wavelength`:

```bash
mkdir dunning && cd dunning
npm init -y
npm pkg set type=module
npm install -D tsx typescript @types/node
```

Después agrega `"dunning"` al array `workspaces` de la raíz como lo has hecho para cada workspace desde la lección de conciliación. Tres archivos se crean en los pasos de abajo, todos por ti: `statemachine.ts` (la máquina), `outcome.ts` (el puente desde el vocabulario de club-billing), y `statemachine.test.ts` (la barrera).

**Paso 2: la máquina de estados.** Abre `dunning/statemachine.ts`. Los tipos van primero, porque en este archivo los tipos son la mitad de la política:

```ts
// dunning/statemachine.ts
export type SubscriptionState = 'active' | 'open-invoice' | 'cancelled';

export type PullOutcome =
  | { kind: 'ok'; signature: string; amountBaseUnits: bigint }
  | { kind: 'insufficient-funds'; amountBaseUnits: bigint }
  | { kind: 'cancelled' };

export type DunningEvent =
  | { type: 'pull'; period: number; outcome: PullOutcome }
  | { type: 'settle'; invoiceId: string; signature: string }
  | { type: 'cancel'; reason: string };

// Note what this vocabulary refuses to express: there is no
// 'schedule-retry' effect. Retry-never is unrepresentable-by-type,
// the same trick policy.ts pulled on fulfill-anyway last module.
export type LedgerEffect =
  | { effect: 'record-paid-invoice'; invoiceId: string; period: number; signature: string; amountBaseUnits: string }
  | { effect: 'open-invoice'; invoiceId: string; period: number; amountBaseUnits: string }
  | { effect: 'settle-invoice'; invoiceId: string; signature: string }
  | { effect: 'mark-cancelled'; reason: string }
  | { effect: 'enqueue-revoke-abandoned'; subscriptionPda: string; instruction: 'RevokeAbandonedSubscription' };

export type Subscription = {
  id: string;
  subscriptionPda: string;
  state: SubscriptionState;
  openInvoiceId?: string;
};

export type Transition = { next: SubscriptionState; effects: LedgerEffect[] };
```

Los montos serializan como strings, la misma regla de bigint-en-disco que el libro mayor hace cumplir desde el módulo 4. Después el reducer, trabajado en vez de retenido, porque las seis transiciones de la sección de teoría ya dicen la respuesta entera; tecléalo brazo por brazo, comprobando cada uno contra su línea de transición, y guarda el pensamiento reservado para la corrida en solitario de devnet donde ningún listado te va a ayudar.

```ts
const invoiceId = (sub: Subscription, period: number) => `inv:${sub.id}:${period}`;

export function transition(sub: Subscription, event: DunningEvent): Transition {
  switch (sub.state) {
    case 'active': {
      if (event.type === 'pull') {
        const { outcome, period } = event;
        if (outcome.kind === 'ok') {
          return {
            next: 'active',
            effects: [{
              effect: 'record-paid-invoice',
              invoiceId: invoiceId(sub, period),
              period,
              signature: outcome.signature,
              amountBaseUnits: outcome.amountBaseUnits.toString(),
            }],
          };
        }
        if (outcome.kind === 'insufficient-funds') {
          return {
            next: 'open-invoice',
            effects: [{
              effect: 'open-invoice',
              invoiceId: invoiceId(sub, period),
              period,
              amountBaseUnits: outcome.amountBaseUnits.toString(),
            }],
          };
        }
        // outcome.kind === 'cancelled': the chain said no; mirror it.
        return cancelTransition(sub, 'revoked-on-chain');
      }
      if (event.type === 'cancel') return cancelTransition(sub, event.reason);
      throw new Error(`no ${event.type} transition from active`);
    }

    case 'open-invoice': {
      if (event.type === 'pull') {
        // THE load-bearing refusal: an open invoice is never retried
        // against the wallet. The crank must not even ask.
        throw new Error(
          `refusing pull for ${sub.id}: open invoice ${sub.openInvoiceId}; retry-never`,
        );
      }
      if (event.type === 'settle') {
        if (event.invoiceId !== sub.openInvoiceId) {
          throw new Error(`settle for unknown invoice ${event.invoiceId}`);
        }
        return {
          next: 'active', // resume
          effects: [{ effect: 'settle-invoice', invoiceId: event.invoiceId, signature: event.signature }],
        };
      }
      return cancelTransition(sub, event.reason);
    }

    case 'cancelled': {
      if (event.type === 'settle' && event.invoiceId === sub.openInvoiceId) {
        // A receivable collected after cancellation still settles;
        // the subscription itself stays dead.
        return {
          next: 'cancelled',
          effects: [{ effect: 'settle-invoice', invoiceId: event.invoiceId, signature: event.signature }],
        };
      }
      throw new Error(`no ${event.type} transition from cancelled`);
    }
  }
}

function cancelTransition(sub: Subscription, reason: string): Transition {
  return {
    next: 'cancelled',
    effects: [
      { effect: 'mark-cancelled', reason },
      {
        effect: 'enqueue-revoke-abandoned',
        subscriptionPda: sub.subscriptionPda,
        instruction: 'RevokeAbandonedSubscription',
      },
    ],
  };
}
```

Dos decisiones aquí son deliberadas. El brazo del pull en open-invoice lanza en vez de devolver un no-op, porque un no-op silencioso invita a un bug del crank donde los reintentos pasan y nada lo nota; un invariante lanzado convierte el mismo bug en una alerta. Una consecuencia para el loop del crank de la lección pasada: su `catch` por suscriptor loguea y sigue, lo que se tragaría calladamente exactamente este throw. Cuando rutees el crank por la máquina en la corrida en solitario, o comprueba el estado del libro mayor antes de llamar siquiera a `pullOnce` (más limpio), o vuelve a lanzar desde el catch cuando el mensaje empieza con `refusing pull`; un invariante tragado es la falla de las 3 a.m. que este brazo existe para prevenir. Y la cancelación desde cualquier estado vivo se rutea por un solo `cancelTransition`, así que hay exactamente un lugar en el código base donde se encola la recuperación de rent, que es el lugar donde después vas a agregar la comprobación del horizonte de abandono.

**Paso 3: el adaptador de outcome.** La máquina de estados habla `PullOutcome`; club-billing habla decisiones de `decidePull` y resultados de send. `dunning/outcome.ts` es el puente de diez líneas:

```ts
// dunning/outcome.ts
import type { PullOutcome } from './statemachine';

// The shape last lesson's period-window guard returns, verbatim.
export interface PullDecision {
  shouldPull: boolean;
  reason: string; // 'due' when pulling, else 'canceled' | 'expired' | 'too-early'
  nextEligibleTs: number;
}

export type PullAttempt =
  | { ok: true; signature: string }
  | { ok: false; error: 'insufficient-funds' | 'other'; detail?: string };

export function outcomeFromPull(
  decision: PullDecision,
  attempt: PullAttempt | null,
  amountBaseUnits: bigint,
): PullOutcome | null {
  if (!decision.shouldPull) {
    // Cancelled on-chain is a state-machine event; too-early and
    // expired are non-events for dunning (the crank just moves on).
    // NOTE the spelling seam, on purpose and documented: the guard's
    // reason string is 'canceled' (one L, the client's US spelling);
    // the state machine's own vocabulary is 'cancelled' (two Ls).
    // The comparison below is on the guard's side. Keep it one-L.
    return decision.reason === 'canceled' ? { kind: 'cancelled' } : null;
  }
  if (!attempt) throw new Error('decision said pull, but no attempt was made');
  if (attempt.ok) return { kind: 'ok', signature: attempt.signature, amountBaseUnits };
  if (attempt.error === 'insufficient-funds') {
    return { kind: 'insufficient-funds', amountBaseUnits };
  }
  // RPC hiccups and blockhash expiry are NOT dunning events: the pull
  // never legally failed, so the crank retries the SUBMISSION next
  // tick. Retry-never bans retrying the debt, not the plumbing.
  return null;
}
```

Esa última rama es la línea más sutil del lab, así que, para que quede claro: retry-never gobierna la renovación, no la red. Una transacción que venció antes de aterrizar nunca le cobró a nadie y nunca falló contra un saldo; volver a enviarla es plomería, no dunning. Solo un pull que la blockchain de verdad rechazó por fondos insuficientes se convierte en una factura abierta.

**Paso 4: cablea la liquidación y la fila de revocación.** El lado de la liquidación es un puente, no un sistema, y vive en el workspace ops de kit-v6 porque mira a la blockchain (el reducer de dunning que alimenta es TypeScript puro y no importa de ningún kit). Crea `backoffice-refunds/src/settle-invoices.ts`:

```ts
// backoffice-refunds/src/settle-invoices.ts: open invoices get paid through
// the exact machinery a checkout order uses. Two functions, one timer.
import { generateKeyPairSigner, address, type Address } from '@solana/kit';
import { reconcileOrder } from './reconcile';
import { recordOrder } from './ledger';

const MERCHANT = address(process.env.MERCHANT_ADDRESS!);
const MERCHANT_ATA = address(process.env.MERCHANT_ATA!);
const CLUB_MINT = address(process.env.CLUB_MINT!);

// Called from the open-invoice effect: mint the invoice a claim ticket and
// register it as an open order, exactly like a checkout does at page load.
export async function openInvoiceReference(
  invoiceId: string,
  amountBaseUnits: bigint,
): Promise<Address> {
  const reference = (await generateKeyPairSigner()).address;
  recordOrder(
    {
      orderId: invoiceId,
      recipient: MERCHANT,
      recipientAta: MERCHANT_ATA,
      mint: CLUB_MINT,
      amountBaseUnits,
    },
    reference,
  );
  return reference;
}

// Called on a timer for every open invoice: a paid reconcile becomes the
// settle event your dunning reducer consumes.
export async function settleTick(
  invoiceId: string,
  reference: Address,
): Promise<{ type: 'settle'; invoiceId: string; signature: string } | null> {
  const result = await reconcileOrder(reference);
  return result.status === 'paid'
    ? { type: 'settle', invoiceId, signature: result.signature }
    : null;
}
```

El loop del temporizador en sí son tres líneas de `setInterval` alrededor de `settleTick`, y el evento devuelto va directo a `transition`. Nada de esto es maquinaria nueva: `recordOrder` y `reconcileOrder` son los exports de la lección de conciliación, haciendo por una factura exactamente lo que hacen por un pedido. El lado del revoke drena los efectos encolados. El consumidor construye la instrucción con el cliente que instalaste la lección pasada (`@solana/subscriptions`, fijado en 0.5.0 en el workspace de kit ^7, pin congelado en 2026-08), exactamente como lo moldean los docs del programa:

```ts
import { getRevokeAbandonedSubscriptionInstruction } from '@solana/subscriptions';

const ix = getRevokeAbandonedSubscriptionInstruction({
  payer: payerSigner,             // the recorded payer signs, nobody else
  subscriptionAccount: subscriptionPda,
  subscriptionAuthority: subscriptionAuthorityPda,
  planPda,
});
```

Antes de enviar, el consumidor comprueba la precondición sobre la que advierten los docs: si la Subscription Authority sigue viva para esa cuenta, la instrucción falla por diseño, y el camino de revocación ordinario es la salida correcta en su lugar. Loguea el rechazo y vuelve a encolar con la razón; una fila de limpieza que descarta las fallas calladamente es como las cuentas se filtran para siempre.

El mismo patrón de drenaje de efectos lleva la historia de las notificaciones de la mitad teórica. Suscribe un segundo consumidor a los efectos `open-invoice` y haz que mande el mensaje de falla con la reference key de la factura adjunta; la cadencia del recordatorio y el horizonte del aviso final corren sobre la edad de la factura, leída del libro mayor en el mismo temporizador que corre el conciliador. Ninguna arquitectura nueva, solo un lector más de los efectos que ya emites, que es el argumento entero para hacer que la máquina de estados devuelva efectos en vez de ejecutarlos.

**Paso 5: escribe y corre la barrera.** Crea `dunning/statemachine.test.ts`. Maneja dos ciclos exitosos, fuerza un outcome de insufficient-funds, afirma que el rechazo del pull durante open-invoice de verdad lanza, afirma que no existe ningún efecto de reintento en ninguna parte de la traza, liquida, y cancela:

```ts
// dunning/statemachine.test.ts: the lesson's gate. Pure logic, no chain.
import { transition, type Subscription, type LedgerEffect } from './statemachine';

function fail(msg: string): never {
  console.error(`FAIL: ${msg}`);
  process.exit(1);
}

let sub: Subscription = { id: 'club-77', subscriptionPda: 'SubPda11111111111111111111111111111111111111', state: 'active' };
const trace: LedgerEffect[] = [];
function apply(t: { next: Subscription['state']; effects: LedgerEffect[] }, openInvoiceId?: string): void {
  sub = { ...sub, state: t.next, openInvoiceId: openInvoiceId ?? sub.openInvoiceId };
  trace.push(...t.effects);
}

// 1. Two successful cycles keep the subscription active.
for (const period of [1, 2]) {
  const t = transition(sub, {
    type: 'pull',
    period,
    outcome: { kind: 'ok', signature: `sig${period}`, amountBaseUnits: 15_000_000n },
  });
  if (t.next !== 'active') fail(`cycle ${period} should stay active`);
  apply(t);
}

// 2. Insufficient funds opens an invoice, schedules nothing.
const t3 = transition(sub, {
  type: 'pull',
  period: 3,
  outcome: { kind: 'insufficient-funds', amountBaseUnits: 15_000_000n },
});
if (t3.next !== 'open-invoice') fail('failed pull must open an invoice');
const opened = t3.effects.find((e) => e.effect === 'open-invoice');
if (!opened || opened.effect !== 'open-invoice') fail('open-invoice effect missing');
apply(t3, opened.invoiceId);

// 3. Retry-never is an invariant: a pull against open-invoice throws.
let threw = false;
try {
  transition(sub, {
    type: 'pull',
    period: 4,
    outcome: { kind: 'ok', signature: 'sigX', amountBaseUnits: 15_000_000n },
  });
} catch {
  threw = true;
}
if (!threw) fail('pull during open-invoice must throw (retry-never)');

// 4. No retry effect exists anywhere in the trace (or the vocabulary).
if (trace.some((e) => e.effect.includes('retry'))) fail('a retry effect leaked into the ledger');

// 5. Settlement resumes.
const t5 = transition(sub, { type: 'settle', invoiceId: sub.openInvoiceId!, signature: 'sigSettle' });
if (t5.next !== 'active') fail('settle must resume the subscription');
apply(t5);

// 6. Explicit cancel enqueues rent recovery.
const t6 = transition(sub, { type: 'cancel', reason: 'user-request' });
if (t6.next !== 'cancelled') fail('cancel must land in cancelled');
if (!t6.effects.some((e) => e.effect === 'enqueue-revoke-abandoned')) {
  fail('cancel must enqueue RevokeAbandoned');
}
apply(t6);

console.log('failed renewal -> open-invoice (no wallet retry); settle -> resume; cancel -> RevokeAbandoned enqueued');
```

Córrelo desde la carpeta `dunning`:

```bash
npx tsx statemachine.test.ts
```

Salida esperada:

```
failed renewal -> open-invoice (no wallet retry); settle -> resume; cancel -> RevokeAbandoned enqueued
```

Si la afirmación del rechazo falla, tu brazo de open-invoice se ablandó; si la afirmación de cero-reintentos falla, un efecto se coló en tu vocabulario que debería ser irrepresentable. Las dos son la lección, hecha cumplir.

## Challenge

Las transiciones fueron trabajo de completion. La corrida es en solitario, sobre devnet, contra el club real de la lección pasada.

Los términos del plan son inmutables una vez que alguien se suscribe (los campos `expected*` que firmaste fijaron los términos que aceptaste; la inmutabilidad misma es obra del programa — UpdatePlan nunca toca los términos), así que comprimir el período quiere decir un segundo plan, no una edición: crea el plan id 2 con `periodHours: 1n` (una hora es el piso que permite la unidad del plan), suscríbele tu oyente de prueba fondeado, y deja en paz la delegación del plan viejo; mantiene su propio reloj, y puedes darlo de baja después. Sé honesto contigo mismo sobre el reloj de pared que esto compra: dos ciclos completos en un plan por hora son dos horas, así que arranca el crank temprano en la sesión, déjalo correr, y haz el drenaje entre el ciclo dos y el ciclo tres. Corre esos dos ciclos y mira aterrizar dos filas de factura pagada en el libro mayor del backoffice. Fíjate quién las escribe, porque este es el único lugar donde los dos escritores del libro mayor del curso son fáciles de confundir: el crank escribe las filas de facturación él mismo, indexadas sobre la firma del pull, exactamente como estableció la lección pasada. El conciliador del módulo 4 nunca las ve — empareja facturas OPEN por reference key, y un pull oficial no lleva ni reference ni memo. Mirar `reconcileOrder` aquí y concluir que tu pipeline está roto es el giro equivocado predecible; mira `orders.jsonl` en su lugar. Ahora fuerza la falla: drena la ATA del suscriptor de prueba (manda su saldo de USDC-dev a otra parte desde la billetera del suscriptor), deja que el crank dispare el tercer pull, y demuestra que la falla aterriza como una factura abierta y no como un reintento contra la billetera: tu libro mayor tiene que mostrar una fila de factura abierta y cero intentos de pull más contra esa suscripción, que es lo que garantiza el rechazo lanzado de la máquina de estados si tu crank se rutea por ella. Después recorre las dos salidas. Salida uno: vuelve a recargar la ATA, paga la factura por su reference key, y confirma que el evento settle reanuda la suscripción y que el próximo ciclo factura normal. Salida dos: levanta un segundo oyente de prueba para eso — un par de claves nuevo con un poco de SOL de devnet y USDC-dev, por el mismo flujo de subscribe de la lección pasada — y suscríbelo al plan por hora. Después recorre el desmontaje completo en orden: cancela su suscripción de plano, revoca su Subscription Authority desde la billetera de ese suscriptor (la puerta de salida unilateral de la lección pasada; este es el paso que hace que las cuentas queden *abandonadas* en vez de solo atrasadas, y fíjate que revocar la SA no devuelve por sí solo el rent de las PDAs — solo limpia la precondición que estaba bloqueando RevokeAbandoned), corre el consumidor de la fila de revocación, y confirma que el RevokeAbandoned aterriza, comprobando los lamports de rent que vuelven al saldo del pagador registrado — tu billetera de comercio, si patrocinó el subscribe. Mantén a tu primer oyente fuera de esta salida: su Subscription Authority tiene que seguir viva para la facturación reanudada de la Salida uno, y un consumidor apuntado a ella va a loguear el rechazo de SA-viva y volver a encolar — la precondición diseñada, no un bug.

Aceptación: una traza del libro mayor que muestre dos ciclos pagados, una factura abierta sin intentos automáticos de reintento contra la billetera, un liquida-y-reanuda, y un cancel cuya firma de recuperación de rent puedas pegar. Esa traza, los cinco momentos completos, es el artefacto.

![La traza de aceptación corre cinco momentos: dos ciclos pagados, una falla de ATA drenada aterrizando como una factura abierta con cero reintentos, una liquidación con reference key, y una cancelación reclamando rent.](assets/v07-diagram.png)

## Checkpoint, y lo que el club por fin puede sobrevivir

Si el harness o la corrida de devnet te pelearon, los sospechosos probables en orden: el brazo del pull en open-invoice devolviendo un valor en vez de lanzar (la afirmación lo agarra, pero el costo real habría sido un crank reintentando alegremente para siempre), el adaptador de outcome convirtiendo una falla de RPC en una factura abierta (plomería confundida con deuda; a tu suscriptor le hacen dunning por un timeout), o el consumidor de revoke enviando mientras la Subscription Authority sigue viva y malinterpretando la falla diseñada como un bug. Cada uno es un arreglo de dos líneas una vez nombrado en voz alta.

Da un paso atrás y mira la forma de lo que construiste a lo largo de este módulo, porque ahora es una sola máquina. El club le factura a muchos suscriptores sin custodia, cada pull se concilia en el mismo libro mayor que cada venta, una renovación fallida se degrada a una cuenta por cobrar ordinaria en vez de a una tormenta de reintentos, la liquidación la reanuda por los rieles exactos que usa un pago de checkout, y los acuerdos muertos devuelven su rent. Facturando, fallando con elegancia, limpiando detrás de sí. El club del disco del mes es un negocio de suscripción real sobre rieles que no traen ninguna lógica de negocio de suscripción, y sabes con precisión qué partes son física y qué partes son tu política, porque escribiste la política como tipos.

Cada comprador hasta ahora, eso sí, llegó precargado: USDC en la billetera, billetera en la mano. El próximo módulo abre la puerta por la que camina el resto del mundo: la frontera fiat. Meter y sacar dinero, y saber exactamente quién es merchant-of-record en cada costura.
