# Webhooks que mienten: ingesta e idempotencia

## Resumen

La lección pasada construiste el verificador: un `verify(signature, expectedOrder)` del lado del servidor que trae la transacción, revisa el programa de tokens, el mint, el delta de saldo y el memo, y guarda un conjunto de firmas procesadas. Es el harness de aceptación del curso. Lo que no puede hacer es notar un pago por su cuenta. Alguien tiene que decirle que existe una firma. Hoy construimos al que avisa, sabiendo que el que avisa miente.

Los hallazgos por delante:

- Entregas **backoffice**: un receptor Express para eventos TRANSFER Enhanced de Helius que deduplica sobre la firma de la transacción, pasa cada evento por el verificador de la lección pasada antes de despachar nada, y escribe un libro mayor de pedidos donde una fila significa un pago real.
- Las entregas duplicadas no son un bug que manejas; son el contrato que firmaste. La propia documentación de Helius dice que puede reintentar las entregas de webhook si tu servidor no responde con éxito, y que podrías recibir eventos duplicados (revisado 2026-08-22). Contestar lento cuenta como no contestar.
- La clave de dedup es la firma de la transacción. Esa elección es inferencia nuestra, sólida pero nuestra: Helius documenta los reintentos, no la clave. La vamos a defender como se debe más abajo.
- Un webhook es una notificación, no una demostración. Cada evento pasa por verificación on-chain antes de que el libro mayor registre una venta; los números del payload nunca son entradas del despacho.
- Los webhooks tienen un techo, y se hace cumplir: un webhook que falla al 95 por ciento o más durante 7 días queda deshabilitado automáticamente en los planes pagos (una ventana de 24 horas en el plan gratis). Pasado el volumen de merchant-ops, la ingesta pasa a la indexación con Yellowstone gRPC, que le pertenece al curso planeado Client-Side Mastery.

Siente la falla primero. Guarda esto como `naive.ts` donde sea (asume que `express` está instalado; si estás en una carpeta nueva, `npm install express@5.1.0` primero):

```ts
// naive.ts - the receiver you must never ship
import express from 'express';

const app = express();
app.use(express.json());

let fulfilled = 0;
app.post('/webhooks/helius', (req, res) => {
  for (const event of req.body) {
    fulfilled++;
    console.log(`shipped order ${fulfilled} for signature ${event.signature.slice(0, 8)}...`);
  }
  res.status(200).end();
});

app.listen(4000, () => console.log('naive receiver on :4000'));
```

Córrelo con `npx tsx naive.ts`, y después hazte pasar por Helius un momento. Un pago real, entregado tres veces, exactamente como lo entregaría una tormenta de reintentos:

```bash
for i in 1 2 3; do
  curl -s -X POST localhost:4000/webhooks/helius \
    -H 'Content-Type: application/json' \
    -d '[{"signature":"5KtPn1abcDEF","type":"TRANSFER"}]'
done
```

Tres pedidos despachados. Un pago. Si este receptor manejara el club del disco del mes que tiene Wavelength, acabas de mandarle al mismo cliente tres copias de una tirada de 200 prensados y te comiste el costo de dos. El pipeline que estás por construir existe para que ese loop imprima `shipped order 1` y después se quede callado.

![Pipeline que muestra una entrega de webhook pasando auth, un filtro de forma y un ack 200 inmediato, después un claim de la firma, la resolución del pedido y la verificación on-chain antes de una sola fila del libro mayor.](assets/v01-diagram.webp)

## Notificaciones, no demostraciones

### Higiene de Stripe, claves nuevas

Si ya integraste Stripe, esta lección ya la hiciste una vez. Stripe reintenta los webhooks que tu endpoint no reconoce. Los integradores de Stripe deduplican sobre una clave de idempotencia para que un `checkout.session.completed` reentregado no despache dos veces. Stripe te dice, en negrita, que verifiques el evento contra su API en vez de confiar en el body del POST, porque cualquiera puede hacerle POST de JSON a tu endpoint. Cada una de esas oraciones sobrevive el viaje a Solana con una sola sustitución: la clave de idempotencia pasa a ser la firma de la transacción, y "verificar contra la API de Stripe" pasa a ser "verificar contra la blockchain."

Vale la pena tomarse ese mapeo en serio y no como un eslogan, porque Stripe ya no es un espectador en esta historia. A 2026, Stripe ocupa una posición de cuatro frentes en los pagos cripto: su logo está en el muro de trusted-by de x402.org, coescribió el Agentic Commerce Protocol con OpenAI, coescribió el esquema de autenticación HTTP "Payment" sobre el que está construido el Machine Payments Protocol, y opera como adquirente de USDC-en-Solana que liquida a los comercios en fiat (x402.org, agenticcommerce.dev, el datatracker del IETF y la propia documentación de Stripe, revisado 2026-08-21). La empresa que escribió el manual de higiene de webhooks ahora está procesando el riel exacto sobre el que estás construyendo. Cuando tu disciplina de webhooks acá coincide con la de ellos, eso es evolución convergente bajo el mismo depredador: el evento duplicado.

![Tabla que mapea cinco hábitos de webhook de Stripe a sus equivalentes en Solana, con la clave de idempotencia pasando a ser la firma de la transacción y la verificación por API pasando a ser una verificación on-chain del pago.](assets/v02-table.webp)

Una asimetría no se traslada, y sube lo que está en juego en vez de bajarlo. Cuando un integrador de Stripe despacha dos veces, hay una API de reembolsos y, detrás de ella, una red de tarjetas que puede revertir el dinero. Acá, el módulo 1 ya te enseñó la verdad cruda del riel: no hay chargebacks. Un disco despachado dos veces no es un ticket de soporte incómodo, es inventario perdido. La higiene es la misma que la de Stripe; el precio de saltártela es más alto. Que es la versión de buena noticia que le toca al constructor, honestamente. La disciplina que ya conoces alcanza. Solo que de verdad tienes que hacerla.

### El evento, Enhanced o Raw

Los webhooks de Helius vienen en dos sabores principales, elegidos al momento de crearlos con `webhookType`. **Enhanced** entrega eventos parseados y legibles para humanos: Helius pasa la transacción cruda por su parser y te entrega un objeto tipado con un campo `type` como `TRANSFER`, una `signature`, una `description` y `tokenTransfers` estructurados. **Raw** entrega la transacción más cerca del formato de cable, con menos parseo y menor latencia de entrega. También hay variantes de Discord y gemelos de devnet de cada uno (`enhancedDevnet`, `rawDevnet`), que es lo que usa el lab para que tus pagos de prueba se queden en devnet.

Para merchant ops, Enhanced es el valor por defecto correcto. La diferencia de latencia importa cuando estás corriendo contra los bloques; a una tienda de discos que despacha dentro del minuto no le importa, y la forma parseada quiere decir que tu código de filtro se lee como inglés. El canje: estás consumiendo la interpretación que Helius hace de la transacción, una razón más para que el payload siga siendo una pista y no una fuente de verdad.

Crear uno es un solo POST autenticado, autenticado con la `HELIUS_API_KEY` que exportaste durante el setup del módulo 2. (Tus llamadas RPC no la usan; este curso corre su RPC contra endpoints públicos, y la clave existe solo para los webhooks.) Si `echo $HELIUS_API_KEY` no imprime nada, vuelve a ese paso antes de continuar:

```bash
curl -s -X POST "https://mainnet.helius-rpc.com/v0/webhooks?api-key=$HELIUS_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
    "webhookURL": "https://backoffice.wavelength.example/webhooks/helius",
    "webhookType": "enhancedDevnet",
    "transactionTypes": ["TRANSFER"],
    "accountAddresses": ["<YOUR_MERCHANT_USDC_ATA>"],
    "authHeader": "wavelength-webhook-secret"
  }'
```

Una cosa sobre esa URL antes de los campos: `mainnet.helius-rpc.com` acá es el host de la propia API de webhooks, no una declaración sobre qué cluster estás mirando. El cluster lo elige `webhookType`, que es por lo que un webhook de devnet se crea contra el mismo host.

Campo por campo, porque cada uno es una decisión. `webhookURL` tiene que ser una URL que Helius pueda alcanzar, así que localhost necesita un túnel durante el desarrollo (sirve cualquier túnel HTTPS; el lab anota una opción). `transactionTypes: ["TRANSFER"]` acota la entrega al tipo parseado que nos importa. `accountAddresses` es la lista de vigilancia: tu ATA de comercio, la cuenta a la que le paga cada checkout del módulo 3. Y `authHeader` es un valor que eliges tú y que Helius devuelve en el header Authorization de cada entrega, para que tu receptor pueda descartar tráfico que nunca vino de Helius. El dashboard puede crear el mismo webhook por formulario si prefieres hacer clic. No hay SDK en este camino de ninguna manera: el curl de arriba es todo el registro, y `helius-sdk` nunca se instala en este curso (si igual lo buscaste, ten en cuenta que 3.1.0 pide como peer `@solana/kit` ^6.9, así que te fijaría a la misma línea de kit v6 en la que ya están los workspaces de ops).

Lo que llega a tu endpoint es un **array** de eventos de transacción enhanced, incluso para una sola transacción. Cada elemento lleva `signature`, `type`, `slot`, `timestamp`, `feePayer`, un array `tokenTransfers` con mints y montos, y más. Acá está la parte que debería sentirse rara hasta que hace clic: de todo ese objeto rico, nuestro receptor va a leer exactamente dos campos, `signature` y `type`. Todo lo demás es decorado. No porque los datos estén mal habitualmente, sino porque "habitualmente" no es una política de despacho, y tenemos un verificador cuyo trabajo entero es establecer esos mismos hechos desde la blockchain misma.

![Comparación de los webhooks Enhanced y Raw por forma del payload, latencia, filtrado, postura de confianza y encaje, con Enhanced marcado como el valor por defecto de merchant-ops y los dos costando un crédito por evento.](assets/v03-comparison.webp)

### Los duplicados son el contrato

Ahora el corazón del asunto. La promesa de entrega de Helius es deliberadamente modesta: si tu servidor no responde con éxito, puede reintentar, y podrías recibir eventos duplicados. "Responder con éxito" es, a grandes rasgos, "contestar con un 2xx, a tiempo." Léelo como diseñador de sistemas y caen dos consecuencias.

Primero: la velocidad de tu endpoint es parte de su corrección. Un handler que verifica on-chain antes de contestar puede tardar segundos bajo carga; segundos alcanzan para parecer una falla, y una falla quiere decir reentrega. Así que el receptor hace el ack primero y trabaja después. Acepta el POST, revisa las cosas baratas (header de auth, forma), contesta 200, y después procesa cada evento de forma asíncrona. Esto invierte el instinto que tenemos casi todos, que es contestar solo cuando el trabajo está hecho. Acá, contestar ES un trabajo separado del trabajo, y confundirlos fabrica exactamente los duplicados que después tienes que sobrevivir.

Segundo: como los duplicados son esperables, el despacho exactamente-una-vez no puede vivir en la capa de entrega en absoluto. Tiene que vivir en tu estado, indexado sobre algo que sea idéntico en cada duplicado del mismo pago y distinto en cada pago distinto. Mira el evento y pregunta qué califica. ¿`timestamp`? Los duplicados idénticos podrían no coincidir si se vuelven a parsear, y dos pagos distintos pueden compartir uno. ¿Los montos de `tokenTransfers`? Dos clientes comprando el mismo disco de 28 USDC producen montos idénticos. ¿El payload entero hasheado? Un reparseo con un campo agregado y tu clave cambia mientras el pago no. ¿La firma de la transacción? Única por transacción por construcción, inmutable una vez que la transacción aterriza, presente en cada entrega de esa transacción y, lo mejor de todo, es la entrada exacta que tu verificador ya consume.

Así que: indexa la idempotencia sobre la firma. Déjame etiquetar esto como lo exigen las reglas de honestidad de este curso. Helius documenta los reintentos; no documenta "deduplica sobre la firma." Ese paso es inferencia nuestra. Es sólida, apoyada en lo que una firma es on-chain y no en ningún comportamiento del proveedor, y es la misma inferencia que hace todo integrador serio. Pero si Helius alguna vez cambiara lo que contiene una entrega, la inferencia es lo que volverías a revisar, que es un argumento más a favor del hábito que esta lección entrena: nunca dejes que la palabra del webhook llegue al libro mayor sin que la blockchain la confirme.

El mecanismo es un registro de claims con tres estados. Antes de que ocurra cualquier trabajo sobre un evento, el receptor intenta tomar el claim de su firma. Un claim fresco marca la firma como `processing` y sigue adelante. Una segunda entrega que llega a mitad de la verificación encuentra el claim y se para en seco: por eso el claim tiene que escribirse antes de que empiece la verificación, no después de que tenga éxito. Cuando el trabajo termina, el claim se asienta en `fulfilled` o `rejected`, los dos terminales. Y cuando el trabajo falla por una razón que no es culpa del pago, un timeout de RPC, un crash en nuestro propio código, el claim se libera por completo, así que la firma se lee como nunca-vista otra vez.

El registro entero es suficientemente chico como para leerlo de un tirón:

```ts
// claims.ts - three-state claim registry keyed on the transaction signature
type ClaimState = 'processing' | 'fulfilled' | 'rejected';

const claims = new Map<string, ClaimState>();

export function claim(signature: string): 'fresh' | 'seen' {
  if (claims.has(signature)) return 'seen';
  claims.set(signature, 'processing');
  return 'fresh';
}

export function settle(signature: string, outcome: 'fulfilled' | 'rejected'): void {
  claims.set(signature, outcome);
}

export function release(signature: string): void {
  claims.delete(signature); // transient failure: the next retry claims fresh again
}
```

¿Por qué liberar en vez de rechazar? Por lo que pasan a ser los reintentos entonces. Si tu proceso se muere a mitad de verificar un pago real, Helius va a golpear la puerta de nuevo, el claim liberado contesta `fresh`, y el pago se despacha en la segunda pasada con cero código escrito por ti. El comportamiento de reintento contra el que te pasaste toda esta sección defendiéndote resulta ser tu recuperación ante crashes, gratis, una vez que dejas de pelearte con él. El modo de falla que esto mata es real: rechaza ante un error transitorio y el pago de ese cliente queda varado para siempre en un estado terminal mientras su dinero está sentado en tu ATA. Lo encontrarías durante la conciliación de la próxima lección, pero "el libro mayor se cura solo" le gana a "el libro mayor es auditado."

Antes de dejar esta sección, págale al patrón ack-y-después-trabajar honestamente, porque no es gratis y fingir lo contrario sería exactamente el tipo de palabrería que este curso vive jurando dejar. En el momento en que contestas 200 antes de que el trabajo esté hecho, le dijiste a Helius que la entrega salió bien, lo que quiere decir que Helius nunca la va a reintentar, lo que quiere decir que cualquier evento que se muera entre tu ack y tu settle simplemente desapareció desde el punto de vista del sistema de entrega. Un crash de proceso en esa ventana, un deploy que reinicia el servidor a mitad de lote, un rejection no manejado en `processEvent`: el pago aterrizó on-chain, la notificación fue entregada y reconocida, y tu libro mayor no sabe nada. La válvula `release` no te puede salvar acá, porque liberar solo ayuda cuando viene un reintento, y tú firmaste la renuncia al reintento con tu 200. ¿Entonces qué respalda de verdad el hueco? Dos cosas. Dentro de una sola vida de proceso, el registro de claims más release maneja todo lo que falla en voz alta. A través de las muertes de proceso, nada en esta lección lo hace, a propósito: la red de seguridad para el trabajo perdido en silencio es el barrido de conciliación que construyes la próxima lección, que recorre la historia propia de la blockchain contra el libro mayor y saca a la superficie cada pago que nunca recibió una fila. Los sistemas de producción achican más la ventana empujando los eventos aceptados a una fila de mensajes durable antes de hacer el ack, así que la fila sobrevive al crash aunque el intercambio HTTP ya terminó. Para el volumen de merchant-ops, ack-y-después-trabajar más conciliación es el canje honesto y proporcionado: aceptas una ventana chica de pérdida silenciosa que un barrido nocturno repara, a cambio de un endpoint suficientemente rápido como para que la tormenta de reintentos nunca empiece. Solo ten presente que hiciste ese canje, porque la falla que permite es invisible hasta que vas a buscarla.

![Diagrama de flujo que muestra una firma con su claim tomado antes de cualquier trabajo, duplicados que caen en el claim, eventos verificados que se asientan en fulfilled o rejected, y errores transitorios que liberan el claim para el siguiente reintento.](assets/v04-flowchart.webp)

### El libro mayor que lo dice en serio

El despacho necesita un registro, y el registro es su propio artefacto: el **libro mayor de pedidos**. El nuestro es un archivo JSONL append-only, una línea por pedido despachado, que lleva el id del pedido, la firma que lo pagó, el monto y el mint que el pedido esperaba, y un timestamp. Eso es deliberadamente aburrido. La parte interesante es el invariante que el pipeline le concede: una fila se escribe solo después de un claim fresco y una verificación on-chain que pasa, así que una fila es igual a un pago real, siempre. El libro mayor nunca registra intentos, notificaciones ni esperanzas. Registra dinero.

Lo aburrido también es estructural para lo que viene. Este archivo es el artefacto que la conciliación de la próxima lección lee, línea por línea, contra la historia on-chain. Mantén la forma estable, porque una lección futura llama a `rows()` sobre él y espera exactamente estos campos.

En producción este par, registro más libro mayor, es una sola base de datos con dos tablas, y el claim es un insert en una tabla con un índice único sobre la firma. La garantía de unicidad de la base de datos reemplaza nuestro Map en proceso, y el patrón claim-antes-de-trabajar sobrevive sin cambios: insertar primero, trabajar segundo, borrar la fila ante una falla transitoria. Nuestro registro en memoria pierde los claims en `processing` al reiniciar, y eso es por diseño y no por flojera. Las filas despachadas persisten en el archivo; los claims en vuelo mueren con el proceso; los reintentos reentregan todo lo que murió en vuelo. También puedes reconstruir las entradas `fulfilled` del registro desde el libro mayor al arrancar, un loop sobre `ledger.rows()` llamando a `settle(row.signature, 'fulfilled')`; vale la pena agregarlo el día que cablees el servidor real, y quedó fuera del lab porque la prueba de humo arranca desde un archivo vacío de todos modos.

También está la pregunta silenciosa de qué tan grande se pone este estado, y la lección pasada ya te dio el vocabulario para eso. El conjunto de firmas procesadas del verificador necesitaba un horizonte de desalojo porque guardar cada firma para siempre es crecimiento sin límite, y el registro hereda la misma aritmética: cada pago que Wavelength recibe alguna vez deja atrás una entrada `fulfilled`. La lógica de desalojo se traslada casi sin cambios. Una firma solo puede ser reentregada mientras su transacción todavía pueda confundirse con una fresca, y una vez que un pago es lo bastante viejo como para que el tiempo de vida de su blockhash más un margen cómodo haya pasado, y su fila está a salvo en el libro mayor, la entrada del registro ya hizo su trabajo y puede irse. El libro mayor mismo, en cambio, nunca se desaloja; es el libro del negocio, crece una línea chica por venta, y un año de una tienda de discos ocupada cabe en unos pocos megabytes. Mantén la distinción nítida en la cabeza: el registro es memoria operativa con un horizonte, el libro mayor es historia sin ninguno.

Un hábito más de la lección pasada se traslada: el conjunto de firmas procesadas adentro del verificador sigue existiendo y sigue corriendo. El registro del receptor es la reja rápida de la puerta de entrada; el dedup del verificador es defensa en profundidad detrás de ella. Dos capas indexadas sobre la misma firma cuestan casi nada, y el día en que una de ellas tenga un bug es el día en que aprendes a querer a la otra.

![Una fila JSON anotada del libro mayor con id del pedido, firma, monto en unidades base como string, mint y timestamp, escrita solo después de un claim fresco y una verificación que pasa.](assets/v05-annotated-code.webp)

### Las falsificaciones mueren en el verificador

Hora de pensar como el atacante, porque tu endpoint es una URL pública y el JSON es gratis. Importan dos clases de falsificación.

Clase uno: pura ficción. Un POST con un payload Enhanced bien formado y una firma que no existe on-chain, o existe pero es la transacción no relacionada de otra persona. El header de auth detiene la versión floja de esto, que es por lo que lo revisamos, pero un secreto compartido se filtra, queda en un volcado de config, o lo adivinan, así que es un portero, no una demostración. La pared de verdad es que el despacho requiere que `verify()` pase, y `verify()` arranca desde getTransaction contra la blockchain. Una firma ficticia no resuelve a nada. Una firma real no relacionada falla las comprobaciones de programa-de-tokens, mint, delta o memo. De cualquier manera el claim se asienta en `rejected` y el libro mayor nunca se entera.

Clase dos, la sutil: un pago real, mal descrito. El atacante manda una transacción genuina, digamos 0.01 USDC a tu ATA de comercio de verdad, y después te hace POST de un evento con forma de Enhanced para esa firma donde `tokenTransfers` dice 28 USDC y la descripción nombra tu prensado más caro. Cada campo cuadra salvo los que importan. Si tu receptor leyera los montos del payload, este ataque despacha discos por un centavo. El nuestro no lee nada más que la firma; el verificador trae la transacción real y calcula el delta real, 0.01 USDC, contra los 28 USDC que el pedido esperaba, y lo rechaza como underpaid. La lección se comprime a una sola oración que a esta altura deberías poder recitar: el payload rutea, la blockchain decide.

Eso también es por lo que `resolveOrder` en nuestro pipeline funciona como funciona. Mapear una firma a un pedido vía la descripción del payload le entregaría el ruteo Y la decisión al atacante. En cambio el resolver hace lo que hace el verificador: trae la transacción y lee nuestro propio id de pedido del memo on-chain que tu checkout estampó en el módulo 3. La entrada no confiable llega a nominar una firma para inspección. Eso es todo lo que llega a hacer.

![Tres carriles a través de las mismas rejas: una falsificación ficticia muere en la verificación, un pago real mal descrito muere en la comprobación del monto, y solo el pago honesto llega al libro mayor.](assets/v06-diagram.webp)

### El techo, y lo que hay más allá

Los webhooks fallan de una manera que la plataforma nota. Cada entrega que tu endpoint arruina, deja expirar o responde con un 500 cuenta en tu contra, y la barrera de protección está publicada y es automática: sostén una tasa de falla de 95 por ciento o más durante 7 días en un plan pago y Helius deshabilita el webhook (los planes gratis se juzgan con una ventana de 24 horas). Esto no es castigo por un mal deploy; 95 por ciento durante una semana es un endpoint que está efectivamente muerto. El auto-disable es la plataforma negándose a hacerle DDoS a tu cadáver. Tus defensas son las que ya construiste: haz el ack rápido para que la lentitud no se lea como falla, mantén el handler delgado, y monitorea las estadísticas de entrega del webhook en el dashboard de la misma manera en que mirarías la tasa de error de un endpoint Stripe.

Haz las cuentas mientras estamos acá, porque deciden la arquitectura más honestamente que el gusto. La entrega cuesta 1 crédito por evento. Una tienda de discos que hace incluso mil ventas al día gasta mil créditos en ingesta, error de redondeo contra tu uso de RPC, y el webhook se dispara solo cuando una cuenta vigilada de verdad se mueve. Este es el régimen para el que están diseñados los webhooks: eventos de baja frecuencia y alto valor donde el precio por evento es despreciable y un minuto de latencia es invisible. Ahora inviértelo. Indexar cada transferencia que toca un programa popular, decenas de millones de eventos, requisitos de frescura por debajo del segundo: el precio de entrega por evento y el overhead de un HTTP por evento dejan los dos de tener sentido, y ninguna cantidad de higiene de reintentos arregla un desajuste de arquitectura. La respuesta equivocada clásica es un loop de polling martillando rangos de getTransaction, que quema créditos para volver a traer estado que casi no cambió y aun así va atrasado. La respuesta correcta es una suscripción de streaming directo desde la manguera del validador: Yellowstone gRPC y sus parientes. Ese mundo, la ingesta por gRPC, la plomería de Geyser, el backfill, el asiento entero de infraestructura de datos, es territorio del curso planeado Client-Side Mastery — todavía en producción cuando este curso se publica, así que trata esto como un mapa de dónde vive ese asiento, no como un enlace para hacer clic ya. Wavelength no tiene ese problema. Un back office de comercio es precisamente el asiento de merchant-ops, y para él, el humilde webhook más la disciplina que ahora tienes es la ingeniería correcta, no la versión para principiantes de algo más sofisticado.

![Línea de tiempo de un handler de webhook lento acumulando entregas fallidas y reintentos a lo largo de una semana hasta que la tasa de falla de 95 por ciento durante siete días dispara la deshabilitación automática del endpoint.](assets/v07-timeline.webp)

## Lab: construye el backoffice

Cómo se divide el trabajo de hoy: recorro el servidor y el pipeline contigo de punta a punta (worked). Tú implementas las dos jugadas que esta lección existe para enseñar, el claim de la firma y la escritura en el libro mayor, contra reglas enunciadas y con la prueba de humo como tu juez (completion). Después la repetición de triple entrega y una falsificación hecha a mano son solo tuyas (solo). Terminado quiere decir que `npx tsx smoke.ts` imprime `SMOKE PASS`.

**1. Genera el scaffold del workspace.** `backoffice` vive al lado de tu proyecto `verifier` de la lección pasada para que pueda importar `verify`. El mismo stack de servidor que el resto del curso:

```bash
mkdir -p backoffice/src
cd backoffice
npm init -y
npm pkg set type=module
npm install express@5.1.0
npm install -D tsx@4 typescript @types/express @types/node
```

Los pins y sus notas de frescura: `express` está fijado en 5.1.0 acá, pero cualquier 5.x sirve y nada en este receptor depende de la diferencia (la lección del blink instaló 5.2.1, que es el 5.x actual de npm al 2026-08-22). `tsx` 4 es el runner que usa todo el curso; esta línea de install es su instalación si la máquina está limpia. En esta línea de install no aparece ningún SDK de Helius, y este curso nunca instala uno: el webhook se crea con un curl y el receptor lee JSON plano de un POST HTTP, así que `HELIUS_API_KEY` del setup del módulo 2 es la única dependencia de Helius que tienes. Ese es más bien el punto — la ingesta es solo HTTP, y un receptor de webhooks que necesita el SDK de un proveedor para parsear el body de una petición se echó encima una dependencia para nada.

**2. Tipos: el contrato que consumimos y los dos campos que leemos.**

```ts
// backoffice/src/types.ts

// The verifier's contract, frozen in the last lesson and restated here
// verbatim so backoffice compiles standalone. The real wiring imports the
// verifier package directly; these shapes must keep matching it exactly.
export interface ExpectedOrder {
  orderId: string;         // the id your txreq builder stamps into the spl-memo
  recipient: string;       // the merchant owner address (base58)
  recipientAta: string;    // the merchant token account for the expected mint
  mint: string;            // the mint you price in (base58)
  amountBaseUnits: bigint; // the exact price, integer base units, never a float
}

export type RejectReason =
  | 'duplicate'
  | 'wrong-token-program'
  | 'wrong-mint'
  | 'underpaid'
  | 'wrong-reference';

export type VerifyResult =
  | { ok: true; reason: 'verified'; signature: string }
  | { ok: false; reason: RejectReason | 'not-found'; signature: string };

export type VerifyFn = (
  signature: string,
  expectedOrder: ExpectedOrder,
) => Promise<VerifyResult>;

// Maps a signature to the open order it claims to pay, by reading OUR memo
// out of the transaction on-chain. Returns undefined when the transaction is
// visible but matches no open order. If the transaction is not visible yet,
// THROW instead: the catch in processEvent releases the claim, and the next
// Helius retry resolves it cleanly.
export type ResolveOrderFn = (signature: string) => Promise<ExpectedOrder | undefined>;

// The minimum we read from a Helius Enhanced event: the signature and the type.
// Everything else in the payload is a hint, never an input to fulfillment.
export interface EnhancedEvent {
  signature: string;
  type: string;
}
```

**3. El registro, tu primer peldaño de completion.** Las reglas están en los comentarios; la implementación es tuya. Toda la lección pende de que `claim` haga set-antes-de-trabajar, así que gánatelo:

```ts
// backoffice/src/registry.ts

export type SigState = 'processing' | 'fulfilled' | 'rejected';

export class SignatureRegistry {
  private states = new Map<string, SigState>();

  // The idempotency gate. Called BEFORE any verification work.
  // Rule 1: if the signature is already tracked (any state), return 'seen'.
  // Rule 2: otherwise record it as 'processing' and return 'fresh'.
  // The set-before-work order is the whole trick: a duplicate delivery that
  // arrives while the first is still verifying must land on 'seen'.
  claim(signature: string): 'fresh' | 'seen' {
    throw new Error('Your turn: implement the claim per the two rules above.');
  }

  // Terminal states. A settled signature is never processed again.
  settle(signature: string, state: 'fulfilled' | 'rejected'): void {
    this.states.set(signature, state);
  }

  // Transient-failure escape hatch: forget the claim so the NEXT redelivery
  // gets a clean 'fresh'. This is what turns Helius retries from a nuisance
  // into your crash recovery.
  release(signature: string): void {
    this.states.delete(signature);
  }

  stateOf(signature: string): SigState | undefined {
    return this.states.get(signature);
  }
}
```

**4. El libro mayor, tu segundo peldaño de completion.** JSONL append-only; `record` es tuyo, `rows` viene dado porque la conciliación de la próxima lección depende de su comportamiento exacto:

```ts
// backoffice/src/ledger.ts
import { appendFileSync, existsSync, readFileSync } from 'node:fs';
import type { ExpectedOrder } from './types';

export interface LedgerRow {
  orderId: string;
  signature: string;
  amountBaseUnits: string; // the bigint price serialized; JSON has no bigint
  mint: string;
  fulfilledAt: string; // ISO timestamp
}

// Append-only JSONL. One row = one fulfillment = one real payment.
// Exactly-once is enforced UPSTREAM by the registry claim; the ledger's own
// invariant is that record() is only ever reached through a fresh claim.
export class Ledger {
  constructor(private file: string) {}

  record(order: ExpectedOrder, signature: string): LedgerRow {
    // Rule 1: build a LedgerRow from the ORDER's fields (orderId, mint, and
    //         amountBaseUnits via .toString()) plus the signature. Never from
    //         any webhook payload.
    // Rule 2: fulfilledAt is new Date().toISOString().
    // Rule 3: append the row to this.file as one JSON line ending in '\n',
    //         then return the row.
    throw new Error('Your turn: write the row per the three rules above.');
  }

  rows(): LedgerRow[] {
    if (!existsSync(this.file)) return [];
    return readFileSync(this.file, 'utf8')
      .split('\n')
      .filter((line) => line.length > 0)
      .map((line) => JSON.parse(line) as LedgerRow);
  }
}
```

**5. El receptor, worked.** Lee dos veces la ubicación del ack y el bloque catch; son las dos decisiones en las que la sección de teoría gastó más palabras:

```ts
// backoffice/src/server.ts
import express from 'express';
import type { EnhancedEvent, ResolveOrderFn, VerifyFn } from './types';
import type { SignatureRegistry } from './registry';
import type { Ledger } from './ledger';

export interface BackofficeDeps {
  authSecret: string; // the authHeader value you set at webhook creation
  verify: VerifyFn; // the lesson-1 verifier
  resolveOrder: ResolveOrderFn; // signature -> open order, via the on-chain memo
  registry: SignatureRegistry;
  ledger: Ledger;
  onSettled?: (signature: string) => void; // test hook; unused in production
}

function isEnhancedEvent(value: unknown): value is EnhancedEvent {
  if (typeof value !== 'object' || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.signature === 'string' && typeof v.type === 'string';
}

export function createApp(deps: BackofficeDeps): express.Express {
  const app = express();
  app.use(express.json({ limit: '1mb' }));

  app.post('/webhooks/helius', (req, res) => {
    // Gate 1: the shared secret. A bouncer, not proof.
    if (req.get('authorization') !== deps.authSecret) {
      res.status(401).json({ error: 'bad auth header' });
      return;
    }

    // Helius posts an ARRAY of events. Anything that isn't one is malformed.
    if (!Array.isArray(req.body)) {
      res.status(400).json({ error: 'expected an array of events' });
      return;
    }

    const events = req.body.filter(isEnhancedEvent).filter((e) => e.type === 'TRANSFER');

    // Ack FIRST, work after. A slow answer counts as a failed delivery,
    // and enough failed deliveries kill the webhook.
    res.status(200).json({ received: events.length });

    for (const event of events) {
      void processEvent(deps, event.signature);
    }
  });

  return app;
}

async function processEvent(deps: BackofficeDeps, signature: string): Promise<void> {
  try {
    // The idempotency gate: claim before any work. (Inside the try so a
    // throw here, including the lab's placeholder, fails loudly in the catch
    // instead of tearing the process down as an unhandled rejection.)
    if (deps.registry.claim(signature) === 'seen') return;

    // The payload told us a signature. The CHAIN tells us which order it pays.
    const order = await deps.resolveOrder(signature);
    if (!order) {
      deps.registry.settle(signature, 'rejected');
      deps.onSettled?.(signature);
      return;
    }

    // A webhook is a notification, not proof. The verifier is the proof.
    const result = await deps.verify(signature, order);
    if (result.ok) {
      deps.ledger.record(order, signature);
      deps.registry.settle(signature, 'fulfilled');
    } else if (result.reason === 'not-found') {
      // The last lesson's contract: not-found is transient, not a verdict.
      // At confirmed commitment the transaction can lag a fast webhook.
      // Release, and the next retry re-verifies against a caught-up chain.
      deps.registry.release(signature);
      return;
    } else {
      deps.registry.settle(signature, 'rejected');
    }
    deps.onSettled?.(signature);
  } catch (err) {
    // Transient failure (RPC hiccup, our own bug): log it with its reason so
    // the ops log reads like a diagnosis, then release the claim so the next
    // redelivery retries cleanly. Crashing here without releasing would
    // strand the signature in 'processing' forever.
    console.error(
      `[backoffice] transient failure for ${signature.slice(0, 8)}...:`,
      err instanceof Error ? err.message : err,
    );
    deps.registry.release(signature);
  }
}
```

Recorre la parte worked conmigo. El cuerpo del handler antes del ack hace solo trabajo de tiempo constante: comparar el header, revisar el array, filtrar la forma. Todo lo que puede ser lento o puede fallar vive después de `res.status(200)`, adentro de `processEvent`, lanzado con `void` porque la respuesta HTTP no le debe nada. `processEvent` es la sección de teoría hecha código: claim, resolve, verify, settle, con `release` en el catch como la válvula que cura los reintentos. Una rama merece una segunda mirada: el `not-found` del verificador libera en vez de rechazar, honrando el contrato de la lección pasada de que not-found es transitorio, porque con commitment `confirmed` un webhook rápido le puede ganar a la visibilidad de `getTransaction`. La siguiente reentrega vuelve a verificar contra una blockchain que se puso al día. Y fíjate en lo que está ausente: ni un solo campo del evento más allá de `signature` y `type` se lee nunca. La falsificación por descripción falsa no tiene con qué hablar.

**6. La prueba de humo.** Esta es la barrera de aceptación del lab y la misma repetición de triple entrega que corriste contra el receptor ingenuo, ahora con una falsificación agregada. Pone stubs del verificador y del resolver para que corra offline; los stubs honran los contratos reales exactamente:

```ts
// backoffice/smoke.ts
// Triple-delivers a real event and one spoof against the receiver.
// Pass = exactly one ledger row, spoof rejected. Run: npx tsx smoke.ts
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createApp } from './src/server';
import { SignatureRegistry } from './src/registry';
import { Ledger } from './src/ledger';
import type { ExpectedOrder, VerifyResult } from './src/types';

const REAL_SIG = 'RealSig1111111111111111111111111111111111111111111111111111111111111111111111111111111111';
const SPOOF_SIG = 'SpoofSig111111111111111111111111111111111111111111111111111111111111111111111111111111111';

const order: ExpectedOrder = {
  orderId: 'ord-0088',
  recipient: 'WVLmerchantOwner1111111111111111111111111111', // stub base58
  recipientAta: 'WVLmerchantUsdcAta11111111111111111111111111', // stub base58
  mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  amountBaseUnits: 28_000_000n, // 28 USDC
};

// Stub the chain so the smoke runs offline. The real wiring imports the
// lesson-1 verifier and the on-chain memo resolver instead of these.
const resolveOrder = async (_signature: string): Promise<ExpectedOrder | undefined> =>
  order; // both real event and spoof resolve to this order; the spoof dies at verify, not here
const verify = async (signature: string, _expected: ExpectedOrder): Promise<VerifyResult> =>
  signature === REAL_SIG
    ? { ok: true, reason: 'verified', signature }
    : { ok: false, reason: 'wrong-reference', signature };

const registry = new SignatureRegistry();
const ledger = new Ledger(join(mkdtempSync(join(tmpdir(), 'backoffice-')), 'ledger.jsonl'));

const settled = new Set<string>();
const app = createApp({
  authSecret: 'wavelength-webhook-secret',
  verify,
  resolveOrder,
  registry,
  ledger,
  onSettled: (sig) => settled.add(sig),
});

const server = app.listen(0, async () => {
  const address = server.address();
  if (address === null || typeof address === 'string') throw new Error('no port');
  const url = `http://127.0.0.1:${address.port}/webhooks/helius`;

  const post = (signature: string, auth = 'wavelength-webhook-secret') =>
    fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json', authorization: auth },
      body: JSON.stringify([{ signature, type: 'TRANSFER' }]),
    });

  // 1. Triple delivery of the same real event: Helius retry behavior, replayed.
  for (let i = 0; i < 3; i++) {
    const res = await post(REAL_SIG);
    if (res.status !== 200) throw new Error(`delivery ${i + 1}: expected 200, got ${res.status}`);
  }
  // 2. A spoofed event whose signature fails on-chain verification.
  await post(SPOOF_SIG);
  // 3. A delivery with the wrong auth header never even reaches processing.
  const unauth = await post(REAL_SIG, 'wrong-secret');
  if (unauth.status !== 401) throw new Error(`expected 401 for bad auth, got ${unauth.status}`);

  // Wait for the async processing to settle both signatures.
  for (let i = 0; i < 100 && settled.size < 2; i++) {
    await new Promise((r) => setTimeout(r, 10));
  }

  const rows = ledger.rows();
  if (rows.length !== 1) throw new Error(`expected exactly 1 ledger row, found ${rows.length}`);
  if (rows[0].signature !== REAL_SIG) throw new Error('ledger row carries the wrong signature');
  if (registry.stateOf(SPOOF_SIG) !== 'rejected') throw new Error('spoof was not rejected');
  if (registry.stateOf(REAL_SIG) !== 'fulfilled') throw new Error('real payment not fulfilled');

  console.log('backoffice: triple-delivered webhook -> exactly-once ledger row; spoofed event rejected');
  console.log('SMOKE PASS');
  server.close();
});
```

Córrelo:

```bash
npx tsx smoke.ts
```

Con los dos throws de relleno todavía en su lugar, la prueba de humo falla en el conteo de filas del libro mayor (cada throw se atrapa, el bloque catch del receptor lo loguea con su mensaje `Your turn`, y el claim se libera, así que nunca se escribe ninguna fila), que es el lab diciéndote que los peldaños de completion son genuinamente tuyos. Cuando tu `claim` y tu `record` estén bien, imprime la línea de aprobación. Cablea `npm run verify:backoffice` a este script en `package.json` (`"verify:backoffice": "tsx smoke.ts"`), para que el verify de cada peldaño siga siendo ejecutable por nombre — el hábito que paga cuando el harness de journey del capstone sacude el stack ensamblado y necesitas volver a revisar un peldaño aislado.

![Comparación del verificador, el resolver y el libro mayor temporal con stubs de la prueba de humo contra el cableado en vivo de devnet, con el registro de firmas idéntico de los dos lados.](assets/v08-comparison.webp)

**7. Apúntale un webhook real.** La prueba de humo puso un stub de la blockchain; la corrida en vivo necesita el cableado real, y `createApp` solo construye la app, así que dale un punto de entrada. Crea `backoffice/src/main.ts`:

```ts
// backoffice/src/main.ts - the live wiring for step 7. Before you pay,
// register the order your checkout is about to mint in OPEN_ORDERS.
import { createSolanaRpc, signature as asSignature } from '@solana/kit';
import { createVerifier } from '../../verifier/src/verify.ts';
import { createMemoryStore } from '../../verifier/src/store.ts';
import { createRpcFetchTransaction } from '../../verifier/src/rpc.ts';
import { createApp } from './server';
import { SignatureRegistry } from './registry';
import { Ledger } from './ledger';
import type { ExpectedOrder } from './types';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// The one open order this live run expects, keyed by orderId. A toy on
// purpose: next lesson replaces it with a real open-orders store.
const OPEN_ORDERS = new Map<string, ExpectedOrder>();

// The real resolver: fetch the transaction, read OUR order id out of the
// on-chain memo (wavelength:<orderId>:<description>), map it to an open
// order. Throws when the tx is not visible yet, so processEvent's catch
// releases the claim and the next Helius retry resolves it cleanly.
async function resolveOrder(sig: string): Promise<ExpectedOrder | undefined> {
  const tx = await rpc
    .getTransaction(asSignature(sig), {
      encoding: 'jsonParsed',
      maxSupportedTransactionVersion: 1,
      // Match the verifier's commitment: the default here is `finalized`,
      // which would make every fresh payment "not visible yet" for ~10 extra
      // seconds and lean on the retry loop to paper over the lag.
      commitment: 'confirmed',
    })
    .send();
  if (!tx) throw new Error('transaction not visible yet');
  for (const ix of tx.transaction.message.instructions) {
    const p = ix as { program?: string; parsed?: unknown };
    if (p.program === 'spl-memo' && typeof p.parsed === 'string') {
      const order = OPEN_ORDERS.get(p.parsed.split(':')[1] ?? '');
      if (order) return order;
    }
  }
  return undefined; // visible, but matches no open order
}

const app = createApp({
  authSecret: process.env.WEBHOOK_SECRET ?? 'wavelength-webhook-secret',
  verify: createVerifier({
    fetchTransaction: createRpcFetchTransaction(),
    store: createMemoryStore(),
  }),
  resolveOrder,
  registry: new SignatureRegistry(),
  ledger: new Ledger('orders.jsonl'), // lands at backoffice/orders.jsonl; later modules read this exact path
});
app.listen(4000, () => console.log('backoffice listening on :4000'));
```

Agrega tu pedido a `OPEN_ORDERS` (el id de pedido que tu checkout va a estampar, tu dirección de comercio y tu ATA, el mint, el precio exacto en unidades base), arráncalo con `npx tsx src/main.ts`, y expón el puerto 4000 por un túnel HTTPS (cualquier túnel; si no tienes ninguno instalado, `npx localtunnel --port 4000` es una opción sin configuración). Después corre el curl de creación que está en la sección de teoría con `webhookType: "enhancedDevnet"`, la URL de tu túnel, y tu ATA de comercio de devnet en `accountAddresses`. Después marca una venta en el checkout del módulo 3, pagando desde `/tmp/customer.json` de la manera en que lo hace el peldaño de completion de esa lección (`PAYER_KEYFILE=/tmp/customer.json npm run --workspace transfer-kit pay -- $(solana address) 12.5 <ref>`) — el comercio es el que está siendo pagado, así que el comercio no puede ser el que paga — y mira llegar un evento Enhanced real, tomar el claim, resolver, verificar contra devnet, y aterrizar una fila en el libro mayor. La página de webhooks del dashboard muestra la entrega de cualquier manera, que es tu ventana de debugging cuando el túnel se cae.

## Challenge

**El peldaño de completion** ya quedó atrás si la prueba de humo pasa: tu `claim` y tu `record`, juzgados por la triple entrega.

**El peldaño solo, dos partes, sin recorrido guiado.**

Primero, repetición hostil. Extiende `smoke.ts` (o escribe `attack.ts` al lado) para cubrir los dos casos que la prueba de humo básica no cubre: los tres duplicados del evento real adentro de UN solo array de entrega, que ejercita claim-antes-de-trabajar dentro de un solo lote, y un simulacro de recuperación ante crash donde tu stub de `verify` hace throw en su primera llamada y tiene éxito en la segunda, demostrando que una reentrega después de `release` despacha exactamente una vez. Aceptación: una fila del libro mayor en los dos casos, y el estado del registro del simulacro termina en `fulfilled`.

Segundo, una falsificación real contra el cableado real. En devnet, manda una transferencia genuina a tu ATA de comercio por un monto de tokens muy por debajo de cualquier pedido abierto, arma a mano un evento con forma de Enhanced para esa firma real que afirma un pago a precio completo, y hazle POST a tu receptor con el header de auth correcto. Aceptación: el verificador lo rechaza por el delta on-chain, el registro lee `rejected`, y el libro mayor no ganó nada. Si tu falsificación de alguna manera despacha, no arregles la falsificación. Arregla el receptor, porque ese agujero era real.

## Checkpoint, y lo que el libro mayor no te puede decir

Si la prueba de humo falla en el conteo de filas, tu `claim` está revisando después del trabajo en vez de antes, o `record` está escribiendo más de lo que se le dice; los dos son visibles en menos de un minuto de leer tu propio diff contra las reglas. Si el paso del webhook real no entrega nada, casi siempre es la URL del túnel o la ATA en `accountAddresses`, en ese orden, y el log de entregas del dashboard decide cuál. Cuando pasa, fíjate en lo que ahora tienes en la mano, porque es más que un handler de webhooks: reintentos, crashes, duplicados y dos clases enteras de falsificación colapsan todos en un solo invariante silencioso, una fila del libro mayor por pago real. Ese invariante es la diferencia entre una demo y un back office.

Pero siéntate un momento con el libro mayor y sus puntos ciegos te devuelven la mirada. Sabe lo que fue pagado y verificado, nada más. Un cliente que pagó de más un dólar: una fila, el sobrepago invisible. Un pago que aterrizó on-chain mientras tu webhook estaba deshabilitado: ninguna fila en absoluto, y el dinero sigue siendo tuyo, sin registrar. Y en ningún lado de nada de lo que construiste hay una manera de mandar dinero de vuelta. El libro mayor registra la verdad; no puede notar las verdades que le faltan, y no puede deshacer ninguna. La próxima lección es conciliación, barrer la blockchain contra este archivo para encontrar cada discrepancia, y después el flujo de reembolso que ningún doc oficial te va a enseñar. El riel no tiene chargebacks, así que construimos el devolver nosotros mismos. Nos vemos allá.
