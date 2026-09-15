# MPP, AP2, ACP: la guerra de estándares (y ponerle gate a la API con pay)

## Resumen

La lección pasada cerró el loop de ventas a máquinas: la API de precios de prensado de Wavelength está detrás de @x402/express, un agente que paga liquida cada llamada a través del facilitador de devnet, y cada id de factura de extra.memo aterriza en el mismo libro mayor del back office por el que fluye una venta humana. Una API, un protocolo, conciliado.

Antes de cualquier teoría, demuestra que todavía tienes la herramienta de la que depende esta lección. La instalaste en la primerísima lección de este curso, cuando parecía una curiosidad:

```bash
npm i -g @solana/pay   # the opening lesson installed it locally; go global so `pay` is on your PATH
pay --version
```

Lee con cuidado el número que te sale, porque te enseña todo el tema de la lección antes de que la lección empiece. El 2026-08-22 el release más nuevo de la CLI estaba etiquetado `pay-v0.27.0` (2026-08-02) y `pay-v0.28.0` siguió el 2026-08-26, pero el wrapper de npm `@solana/pay` 1.0.26 fija el build de la CLI que descarga en 0.26.0, así que una instalación global fresca todavía imprime `pay 0.26.0`. Tres números de versión para una herramienta, todos ellos vigentes, ninguno equivocado: la versión del paquete de npm, el build fijado de la CLI y el tag del release más nuevo. Todo lo de abajo está escrito contra 0.26.0 porque eso es lo que el comando de instalación de este curso te entrega de verdad; si tu `pay --version` es más alto, trata las flags y los campos de aquí como una hipótesis de partida y deja que el propio `--help` de la herramienta resuelva cualquier desacuerdo. (Quedarte con el `npx pay` local de la lección uno también funciona; solo prefija con `npx` cada `pay` de esta lección.) Esa identidad doble, librería de checkout y CLI agéntica en un solo paquete, fue la escena inicial de todo este curso, y hoy rinde.

Un solo protocolo es el problema. Esta semana un agente construido por Google, un flujo de checkout de OpenAI y un cliente nativo de Solana pueden todos golpear esa misma puerta, cada uno con una credencial distinta y una suposición distinta sobre quién está vendiendo el disco en realidad. Esto es con lo que te vas a quedar:

- **MPP (Machine Payments Protocol)** viene en dos capas, y colapsarlas en una es lo más común que se dice mal sobre él. La base es `draft-httpauth-payment-00`, "The 'Payment' HTTP Authentication Scheme", un Internet-Draft de la IETF salido de Tempo Labs y Stripe; la contribución de la Solana Foundation es `draft-solana-charge-00`, una spec de método de pago registrada bajo esa base que define el método `solana/charge`. Juntas mueven el pago hacia la maquinaria de autenticación nativa de HTTP: un desafío `WWW-Authenticate: Payment`, una credencial `Authorization`, una prueba `Payment-Receipt`.
- **AP2** (el Agent Payments Protocol de Google, v0.2, estandarizado a través de grupos de trabajo de la FIDO Alliance) y **ACP** (el Agentic Commerce Protocol de Stripe y OpenAI, Apache-2.0, con ChatGPT como su primera plataforma) son primero tarjetas, y ACP está diseñado explícitamente para que el comercio siga siendo merchant-of-record.
- El momento práctico que ningún otro curso tiene: `pay gate api paywall.yml` pone un solo gate delante de la API de precios de prensado para que la misma API responda tanto x402 como MPP, y `pay curl` del lado del cliente auto-negocia el protocolo que el servidor ofrezca.
- La recomendación honesta de 2026, argumentada en vez de afirmada: no le apuestes en exclusiva a ninguno todavía. Pon el gate una sola vez y negocia, y acepta los dos costos de esa cobertura: una dependencia de CLI que cambia rápido, y ningún memo por llamada en el camino del gate, así que las ventas ruteadas por el gate son trabajo de libro mayor todavía pendiente.

Cómo se reparte el trabajo hoy, dicho en voz alta: la teoría de abajo es de servicio completo, el lab es una transcripción guiada que tecleas tú mismo, y el Challenge es puro trabajo de criterio sin ninguna caminata guiada, porque evaluar estándares bajo incertidumbre es la habilidad de verdad que esta lección enseña.

## La guerra de las velocidades

En 1948, Columbia Records introdujo el disco de larga duración de 33⅓ rpm. Un año después RCA respondió con el 45. Los dos eran mejoras reales sobre el 78 de goma laca, los dos eran incompatibles entre sí, y los dos bandos gastaron dinero de marketing insistiendo en que el otro estaba condenado. Los compradores de discos hicieron lo racional: muchos dejaron de comprar reproductores por completo y esperaron. La guerra no terminó con un ganador. Terminó cuando los fabricantes de tocadiscos entregaron reproductores multivelocidad, y entonces cada formato encontró el nicho para el que de verdad era mejor, el LP para álbumes, el 45 para sencillos.

Guarda ese tocadiscos en la cabeza durante la próxima media hora. Los pagos agénticos en 2026 son una guerra de las velocidades: más de cuatro estándares, cada uno respaldado por alguien enorme, cada uno girando a sus propias rpm. Tu trabajo como integrador de Wavelength no es elegir la velocidad ganadora sino entregar el tocadiscos multivelocidad. Lo que está en juego es concreto: adivinas mal y reescribes tu integración de pagos cuando el mercado se mueva; te niegas a elegir y le sirves a cada agente que aparezca mientras tus competidores siguen leyendo borradores de specs. Así que esta es la ruta: primero el desafiante nativo de Solana de cerca, después los dos actores establecidos que van primero por tarjetas, después la única fila de la comparación que decide todo (quién es merchant-of-record), después la apuesta.

![Línea de tiempo de dos carriles que separa el esquema base de MPP registrado en la IETF, con su presentación y su vencimiento, de la spec del método de Solana rastreada en git, sobre una banda de marcadores de x402, AP2, ACP y desafiantes.](assets/v01-timeline.png)

### MPP: el pago como credencial HTTP

MPP es el Machine Payments Protocol, y lo primero que hay que interiorizar es que son dos documentos con dos dueños, y no tienen la misma posición. La base es `draft-httpauth-payment-00`, "The 'Payment' HTTP Authentication Scheme": un Internet-Draft genuino de la IETF, en el datatracker, presentado el 2026-06-19 y venciendo el 2026-12-21, salido de Tempo Labs y Stripe en vez de de algún lugar cerca de Solana. Define el baile desafío-credencial-comprobante en abstracto y se mantiene deliberadamente agnóstico del método de pago; en sus propias palabras, "specific payment methods are defined in separate payment method specifications." La pieza de Solana es una de esas: `draft-solana-charge-00`, "Solana Charge Intent for HTTP Payment Authentication", escrita por Ludo Galabru e Ilan Gitter de la Solana Foundation, que define el método `solana/charge`, qué lleva un charge intent, cómo se firma y cómo se liquida.

Lo que esa spec de método *no* es, es un Internet-Draft de la IETF. Busca `draft-solana-` en el datatracker y devuelve cero documentos; solo el esquema base se presenta ahí. Eso vale un hábito, y te va a evitar citar algo mal en tus propios docs. Las specs de método se renderizan a páginas elegantes con estilo RFC, con una fecha de publicación y un vencimiento impresos arriba, y el build del sitio es lo que produce esas fechas: reconstruye y se mueven, porque nada las registró en ninguna parte. Así que no las cites. Cita el historial de git en cambio: `solana/charge` aterrizó en el repo de specs el 2026-03-24, y su cambio sustantivo más reciente fue el soporte de transferencias confidenciales de Token-2022 el 2026-08-07. Yo congelé eso el 2026-08-22 y lo volvería a comprobar el día en que construyas.

Mecánicamente, MPP hace algo que x402 deliberadamente no hizo: mueve el pago hacia la maquinaria de autenticación nativa de HTTP en vez de a headers propios. El flujo se lee como Basic Auth con dinero adentro. Tu servidor rechaza una petición sin pagar con un desafío `WWW-Authenticate: Payment` que describe lo que quiere. El cliente responde reintentando con un header `Authorization` que lleva un charge intent de Solana firmado como su credencial. Cuando el pago aterriza, la respuesta del servidor incluye un header `Payment-Receipt`, la prueba que el cliente archiva. Desafío, credencial, comprobante. Cualquier librería HTTP que entienda flujos de auth ya entiende la forma de este baile, y esa es la apuesta de diseño: hacer que los pagos entre máquinas sean aburridos para cada proxy, caché y stack de middleware que ha manejado `WWW-Authenticate` durante treinta años.

![Diagrama de secuencia de una llamada MPP en modo pull, desde el desafío Payment pasando por el charge intent firmado del agente hasta el broadcast co-firmado del servidor y el header de comprobante.](assets/v02-flowchart.png)

Dos modos, y el default importa. En **modo pull**, el cliente firma el charge intent y lo entrega; el servidor puede co-firmar como fee payer y hacer el broadcast de la transacción él mismo. En **modo push**, el cliente lleva la transacción a la blockchain por su cuenta y presenta el resultado. Pull es el default, y fíjate en lo que la cláusula de co-firma mete de contrabando: el patrocinio de comisiones viene incorporado en el camino feliz del protocolo. Aquí, que el servidor pague la comisión de red de su propio cliente es la postura por defecto. Guarda esa idea una lección más; está a punto de volverse el tema entero del módulo 8.

La spec de método de Solana lleva dos rasgos más con forma de Solana que vale la pena nombrar. Los payment splits dejan que un solo cargo se abra en abanico hacia múltiples destinatarios, con un tope de 8 transferencias adicionales, que es un sello pagándole a un artista y a una planta de prensado en la misma liquidación sin un segundo salto. Y las transferencias confidenciales de Token-2022 viajan a través de un cargo `type='bundle'`, así que un agente puede pagar sin transmitirle el monto al mundo (las transferencias confidenciales en sí son territorio del curso Digital Assets, Tokenization and Token Extensions; aquí solo necesitas saber que MPP les dejó la puerta abierta).

Pon MPP al lado del protocolo que ya entregaste. x402, del que construiste los dos extremos la lección pasada, lleva su pago en los headers propios `PAYMENT-SIGNATURE` y `PAYMENT-RESPONSE` y liquida a través de un facilitador al que le tienes que confiar visibilidad de cada transacción. MPP dobla ese mismo momento 402 hacia la semántica de auth estándar y, en modo pull, hace que tu propio servidor sea el co-firmante en vez de un tercero. Forma de confianza distinta, mismo instinto comercial: cobrar por llamada, sobre HTTP, en stablecoins, sin registro de cuenta.

### AP2 y ACP: los actores establecidos construyen para tarjetas

Ahora el otro lado de la tienda. AP2 es el Agent Payments Protocol de Google, en v0.2, en estandarización a través de grupos de trabajo de la FIDO Alliance, el mismo organismo que convirtió las passkeys de un demo en un default de la industria. Sus primitivas centrales son credenciales digitales verificables y un par de mandatos firmados: un Checkout Mandate que captura lo que el humano autorizó al agente a comprar, y un Payment Mandate que captura cómo se le permite pagar. El centro del diseño es la responsabilidad para tarjetas. Cuando un agente compra la cosa equivocada con tu Visa, la respuesta de AP2 es un rastro criptográfico de papel de quién autorizó qué. Los rieles de pago mismos siguen siendo lo que eran, lo que hoy quiere decir sobre todo redes de tarjetas.

ACP es el Agentic Commerce Protocol, co-escrito por Stripe y OpenAI, liberado bajo Apache-2.0, con ChatGPT como su primera plataforma desplegada. Su jugada característica es el Shared Payment Token: una credencial acotada que representa el método de pago del comprador, que la plataforma le pasa al comercio para que el comercio, y esta es la cláusula estructural, **siga siendo merchant-of-record**. El cliente compra dentro de ChatGPT, pero el vendedor del disco sigue siendo Wavelength: tu nombre en el estado de cuenta, tu política de reembolsos, tus obligaciones fiscales, tu relación con el cliente. Para cualquiera que haya corrido una tienda a través de un procesador de tarjetas, ACP es el menos ajeno de los cuatro estándares a propósito.

![Diagrama de dos carriles que contrasta los mandatos firmados por humanos de AP2, presentados como credenciales verificables, con el Shared Payment Token de ACP, que deja al comercio cobrando como merchant-of-record.](assets/v03-diagram.png)

Vale la pena detenerse en quién está parado dónde, porque uno de esos nombres está parado en un lugar que puede que no hayas registrado. Mapeaste la cobertura en cuatro frentes de Stripe hace dos lecciones: el muro de trusted-by de x402, ACP co-escrito con OpenAI, el adquirente de USDC que liquida en fiat de la lección del corredor, y el esquema de autenticación HTTP "Payment". Ese cuarto frente es el borrador base que acabas de leer. Stripe co-escribió `draft-httpauth-payment-00`, lo que la pone en cada riel de esta lección, incluido el que suele describirse como la respuesta nativa de Solana. Cuando la empresa de infraestructura de pagos más grande del campo se niega a elegir un único ganador, eso te dice algo sobre lo resuelta que está esta guerra.

Y es una guerra de verdad, no una de diapositivas. OKX entregó un estándar competidor al que llama APP. Mientras tanto atxp.ai, una plataforma de pagos para agentes, migró a x402 más MPP en Solana, una jugada que la Foundation destacó en su resumen del ecosistema de abril de 2026. Estándares multiplicándose en un flanco mientras los integradores se consolidan en otro es exactamente lo que parece un campo en disputa visto desde dentro.

### Merchant-of-record: la fila que decide tu apuesta

Quítale la criptografía y cada estándar es una respuesta a una sola pregunta comercial: cuando un agente compra un disco, ¿quién lo vendió? Conociste merchant-of-record en las lecciones de fiat de este curso; es la entidad que vende legalmente, el nombre en la disputa, la parte que carga con las obligaciones de reembolso y de compliance. Pon a los cuatro en fila sobre esa fila y la guerra se vuelve mucho más fácil de leer.

![Matriz que compara x402, MPP, AP2 y ACP en merchant-of-record, transporte y liquidación, separando el par nativo de cripto del par que va primero por tarjetas, con la columna de MPP marcada como una spec de método bajo el borrador base en vez de un documento de la IETF por derecho propio.](assets/v04-comparison.png)

Lee las columnas y los bandos se ordenan solos. Bajo x402 y MPP estás vendiendo directamente: el agente le paga a tu dirección en stablecoins, y la pregunta interesante es a quién le confías el medio (un facilitador para x402; a nadie más que a tu propio servidor co-firmante en el modo pull por defecto de MPP). El borrador de MPP ni se molesta en replantear merchant-of-record, porque la autenticación-de-pago-sobre-HTTP no cambia quién es el vendedor. Bajo AP2, tu relación con el procesador persiste y la contribución del protocolo es evidencia de autorización; Google no está entrando como el vendedor de tus discos. Bajo ACP, mantenerte como merchant-of-record no es un accidente del diseño, es el titular: Stripe y OpenAI construyeron la maquinaria de credenciales específicamente para que las plataformas puedan alojar el checkout sin absorber el rol legal del comercio.

### La apuesta

¿Así que a qué velocidad le comprometes la tienda? Camina cada apuesta exclusiva hasta su modo de falla.

Apuéstale todo a MPP y le estás apostando a dos documentos a la vez, que es una apuesta más delgada de lo que parece al principio. La Foundation controla `solana/charge`, pero no controla el esquema de abajo: `draft-httpauth-payment-00` le pertenece a Tempo Labs y a Stripe, vence en el datatracker el 2026-12-21, y define el desafío, la credencial y el comprobante por los que tu integración está realmente moldeada. Un `-01` de cualquiera de las dos capas puede mover un campo contra el que construiste, y la spec de método ni está en el reloj de la IETF, así que su sucesión es una decisión de repo en vez de un proceso que puedas mirar desde afuera. La autoría de la Foundation es una señal real. Es una señal sobre una de las dos capas.

Apuéstale todo a x402 y te llevas el tráfico, el alcance cross-chain y el ecosistema de facilitadores que conociste la lección pasada, más la frontera de confianza que también conociste la lección pasada: un intermediario de liquidación que ve cada transacción y puede filtrar lo que liquida. Un buen asiento, honestamente. Simplemente no uno neutral.

Apuéstale todo a AP2 o a ACP y habrás apostado una API nativa de Solana con medición de consumo sobre rieles que van primero por tarjetas, gobernados por Google o por Stripe-y-OpenAI. Para la tienda de Wavelength en ChatGPT algún día, ACP es probablemente la puerta correcta, y la cláusula de merchant-of-record la hace una genuinamente amigable con el comercio. Para la API de precios de prensado, donde el comprador es un script con una billetera y sin tarjeta, es el vendedor correcto y la herramienta equivocada.

El tl;dr es: apostarle a un solo estándar en agosto de 2026 es prematuro, y no tienes por qué. El tocadiscos multivelocidad existe. `pay gate` pone un único gate delante de tu API que responde x402 y MPP simultáneamente, y `pay curl` del lado del cliente negocia el que el servidor ofrezca. Dejas de predecir al ganador y empiezas a servirle a quien aparezca.

Nombra el costo, eso sí, porque la cobertura no es gratis. Estás tomando una dependencia de la CLI de pay: esta lección está escrita contra el build fijado en npm 0.26.0, y el repo ya había etiquetado `pay-v0.28.0` para el 2026-08-26 (comprobado el 2026-09-07). Su feed de releases muestra versiones menores aterrizando con días de diferencia a lo largo de junio, julio y agosto, y va a seguir cambiando sin parar — lee tu propio `pay --version` y la página de releases del repo en vez de esta oración. Así que no incrustes el subcomando en tu aplicación: mantén `pay gate` en la capa de despliegue, un proceso que lanzan tus scripts de ops, nunca un string al que tu lógica de negocio le haga shell out. Si un release futuro renombra o reforma el gate, tu radio de impacto es un archivo de config y una unidad de systemd, y en el peor caso esta lección se degrada con elegancia: el demo del lado del cliente de `pay curl` todavía funciona contra el middleware de x402 pelado que entregaste la lección pasada. Esa es la diferencia entre depender de una herramienta que se mueve y ser estructural sobre ella.

![Mapa de decisión que pesa las apuestas exclusivas a MPP, x402, AP2 o ACP contra ponerle gate una sola vez con la CLI de pay, cada nodo cargando su propio modo de falla o costo.](assets/v05-diagram.png)

## Lab: pon el gate una vez, responde a los dos

La transcripción objetivo: un gate delante de la API de precios de prensado, una llamada sin pagar rechazada con los desafíos de los dos protocolos, y una llamada pagada negociada por MPP completada por `pay curl`. Teclea junto conmigo; la transcripción es el artefacto que la barrera de aceptación de abajo te pide sostener — la plataforma califica el quiz, la transcripción es tu propia evidencia.

Una nota de honestidad sobre el esquema antes del paso uno, en el mismo espíritu que cada pin de versión de este curso: la superficie de config del gate le pertenece a una CLI que entrega rápido. Los campos de abajo se leyeron de `pay 0.26.0` el 2026-08-22 pidiéndole a la herramienta que escribiera su propia config, que es el paso 3, y ese archivo generado le gana a esta página en todas partes excepto en un bug conocido del scaffold de 0.26.0 (un campo `forward_url` obsoleto) que el paso 3 te guía a arreglar.

**1. Confirma el toolchain.** `pay --version`, del principio de la lección, responde por la CLI. Dos cosas más que necesita antes de poder pagar por nada: una cuenta, y algún lugar a donde mandar dinero. `pay setup` genera un keypair, lo guarda en el keystore de tu sistema operativo y ofrece fondearlo; en macOS el backend es el keychain, así que un shell no interactivo necesita `pay setup --backend keychain` (Linux: `gnome-keyring`, Windows: `windows-hello`, CI headless: `file`). Sáltatelo y el primer `pay curl` se va a detener y te va a correr setup a media clase. La API pelada de abajo necesita el mismo tooling de workspace que la lección pasada: Node con `express` (`npm i express`) y `npx tsx` para correr TypeScript directamente.

**2. Levanta la API pelada de precios de prensado.** No el servidor envuelto en x402 de la lección pasada: la ruta de cotización desnuda que está debajo. El gate está a punto de ser dueño de la capa de pago, así que el upstream tiene que estar libre de pagos.

```bash
mkdir -p ~/wavelength/pay-gate && cd ~/wavelength/pay-gate
```

```ts
// ~/wavelength/pay-gate/price-api.ts
// The pressing-price quote, with zero payment code. The gate in front handles that.
import express from 'express';

const app = express();

app.get('/price', (req, res) => {
  const record = String(req.query.record ?? 'WVL-UNSPECIFIED');
  // `run` is the alias last lesson's x402 agent already sends; accepting both
  // is what lets that agent hit this API through the gate in step 7 unchanged.
  const runSize = Number(req.query.runSize ?? req.query.run ?? 0);
  if (!Number.isInteger(runSize) || runSize < 100) {
    res.status(400).json({ error: 'runSize (min 100) required' });
    return;
  }
  // The price model is reworked from last lesson on purpose: the setup fee is
  // now explicit and amortized over the run instead of folded into a flat unit
  // rate, so quotes for the same run WILL differ from last lesson's numbers.
  const setupFeeUsd = 900;
  const perUnitUsd = runSize >= 500 ? 4.1 : 5.6;
  const totalUsd = setupFeeUsd + perUnitUsd * runSize;
  res.json({
    record,
    runSize,
    unitPriceUsd: Number((totalUsd / runSize).toFixed(2)),
    totalUsd: Number(totalUsd.toFixed(2)),
    currency: 'USD',
  });
});

app.listen(3000, () => console.log('pressing-price API (bare) on :3000'));
```

Córrela y pásale la prueba de humo:

```bash
npx tsx ~/wavelength/pay-gate/price-api.ts &
curl -s 'http://localhost:3000/price?record=WVL-014&runSize=500'
```

Deberías ver una cotización en JSON. Fíjate en lo que acabas de demostrar: el endpoint por ahora regala su respuesta gratis, a cualquiera, que es el problema inicial de la lección pasada otra vez.

**3. Escribe la config del gate.** No la escribas a mano desde cero: pídele a la herramienta que escriba la suya, después aplica el único arreglo conocido de abajo y compara contra mi versión editada:

```bash
cd ~/wavelength/pay-gate
pay server scaffold          # writes paywall.yml
pay gate api --help          # the flags, which are a separate surface from the file
```

Dos superficies, dos fuentes de verdad: el archivo del scaffold define los nombres de los campos, `--help` define las flags. Ahora edita el scaffold hasta dejar el único endpoint de Wavelength. La versión de abajo es lo que 0.26.0 acepta, e incluye un arreglo con el que de otro modo te toparías como un error, porque la plantilla del scaffold en este build omite un bloque que el binario requiere:

```yaml
# paywall.yml - edited from `pay server scaffold` output, pay 0.26.0, 2026-08-22
name: wavelength-price
subdomain: wavelength
title: "Wavelength pressing-price API"
description: "Vinyl pressing quotes, per call"
category: other
version: v1
accounting: pooled
routing:                        # the scaffold writes a flat `forward_url:` here;
  type: proxy                   # the 0.26.0 binary wants this block instead and
  url: http://localhost:3000    # errors "Invalid paywall: missing field `routing`"
endpoints:
  - method: GET
    path: "price"
    description: "Pressing-price quote"
    metering:
      dimensions:
        - direction: usage
          unit: requests
          scale: 1
          tiers:
            - price_usd: 0.10   # per call, deliberately under the $1
                                # spendControls default you met last lesson
```

Ese desajuste entre el propio scaffold de una herramienta y su propio parser vale treinta segundos de atención, porque es el bug más instructivo del camino hacia un gate que funciona: hasta la plantilla de primera mano se va a la deriva del binario de primera mano en un proyecto que entrega así de rápido. El `forward_url` de la plantilla es un renombre que el parser ya dejó atrás. Lee el error, cambia el bloque, sigue adelante. Fíjate también en lo que no está en este archivo: ningún campo de memo por llamada. La conciliación por id de factura en esta capa le pertenece al protocolo que negocie el pago, `extra.memo` del lado de x402 como lo construiste la lección pasada, y el gate no te ofrece una perilla para eso. Y ningún campo de beneficiario tampoco: nada en este archivo nombra la billetera en la que aterriza el dinero, así que antes de confiarle al gate cualquier cosa real, abre en un explorador la firma de una llamada liquidada del paso 6 y confirma qué cuenta fue acreditada de verdad. Trata cablear eso a la dirección del comercio de Wavelength como un pendiente de salida al aire, no como una suposición.

**4. Levanta el gate.** Primero libera su puerto: el servidor de middleware de la lección pasada, y la API mock de la lección anterior a esa, los dos escuchaban en `:4021`, así que detén cualquiera de ellos que siga corriendo (ctrl-C en sus terminales) o el gate se muere con `EADDRINUSE` en el momento en que hace bind.

```bash
pay gate api paywall.yml --bind 127.0.0.1:4021 --rpc-url https://api.devnet.solana.com
```

Las dos flags son deliberadas. `--bind` mueve el gate de su default `0.0.0.0:1402` al puerto que usa esta lección, y hacer bind a localhost mantiene fuera de tu red un paywall con el que estás experimentando. `--rpc-url` importa más: el gate valida el destinatario del pago contra el RPC de mainnet a menos que lo apuntes a otro lado, y este lab es un lab de devnet. (Si fondear una cuenta de devnet es una molestia, la CLI también tiene un modo `pay --sandbox gate api ...` que corre contra un Surfpool alojado con billeteras efímeras auto-fondeadas; necesita que ese host esté alcanzable, así que trátalo como la alternativa, no como el default.) El gate hace de proxy en `:4021` delante de tu API pelada en `:3000`. Tu lógica de negocio no cambió ni una línea; la capa de pago ahora vive completamente delante de ella.

**5. Golpea sin pagar.** Pégale al gate con curl pelado y lee el rechazo de cerca:

```bash
curl -i 'http://localhost:4021/price?record=WVL-014&runSize=500'
```

El estado es 402, y la parte interesante es que la respuesta habla dos veces, las dos veces en headers (la redacción exacta le toca cambiarla a la CLI; la forma es lo que estás comprobando):

```text
HTTP/1.1 402 Payment Required
WWW-Authenticate: Payment ...challenge fields: amount, asset, recipient...
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6Miwi...   <- base64 x402 v2 challenge
Content-Length: 2

{}
```

La línea `WWW-Authenticate` es el desafío de MPP, en el header de auth que esta lección acaba de introducir. La línea `PAYMENT-REQUIRED` es el desafío de x402 v2 que ya sabes decodificar de la lección pasada, y viaja en un header por la razón que dio la lección pasada. Así que aquí tampoco vayas a buscar un array `accepts` en el cuerpo. El cuerpo es `{}`, exactamente como lo era contra tu propio middleware. Un solo rechazo, dos protocolos, dos headers, los dos anunciando el mismo precio. Ese doble discurso es el producto entero de esta lección.

![Topología que muestra agentes de x402, clientes de MPP y llamadores sin pagar pegándole todos a un solo pay gate en el puerto 4021, que reenvía solo las llamadas liquidadas a la API pelada de precios de prensado en el puerto 3000.](assets/v06-diagram.png)

**6. Deja que la CLI negocie.** Ahora la llamada pagada, con el lado del cliente de la misma herramienta:

```bash
pay curl 'http://localhost:4021/price?record=WVL-014&runSize=500'
```

Mira la secuencia que narra: primera petición, 402 recibido, protocolo elegido (MPP aquí, dado que `pay curl` es hablante nativo), charge intent firmado, reintento con la credencial `Authorization`, y después tu cotización JSON con un header `Payment-Receipt` en la respuesta. Captura la transcripción completa; es el primer punto de la barrera de aceptación de abajo. Voy a admitir que la primera vez que corrí este flujo de punta a punta, lo que me agarró no fue que el pago aterrizara, fue lo aburrida que se ve la transcripción. Un desafío de auth, una credencial, un comprobante. Treinta años de memoria muscular de HTTP, ahora con dinero adentro.

**7. Demuestra que la otra velocidad todavía suena.** El gate dice servir x402 también, así que verifícalo con el agente que paga que construiste la lección pasada, apuntado al gate en vez de al viejo middleware. Su URL base es la variable de entorno `API_URL` que cableaste la lección pasada, y él le agrega su propio `?run=...&invoice=...`, que es exactamente por lo que la API del paso 2 acepta `run` además de `runSize`:

```bash
cd ~/wavelength/x402
API_URL=http://localhost:4021 npx tsx src/agent.ts
```

El agente debería liquidar exactamente como lo hizo la lección pasada, headers y facilitador y todo, sin notar nunca que el servidor detrás de la puerta cambió. Tres cosas sí difieren, así que espéralas en vez de depurarlas.

Primero, el gate mide $0.10 por llamada contra el precio de ruta de $0.05 de la lección pasada (sigue muy por debajo del tope de spendControls), y los números de la cotización reflejan el modelo de precios reformado del paso 2.

Segundo, la cuarta llamada del agente es la que pasa el tope a propósito, y va a `/price/rush`. Esa ruta no existe aquí: la API pelada del paso 2 sirve solo `/price`, y `paywall.yml` declara solo el endpoint `price`, así que el gate responde 404 y la llamada nunca llega lo bastante lejos para ser rechazada por `spendControls`. Ese es el comportamiento correcto para esta lección, cuyo tema es la negociación de protocolos en vez de los límites — el rechazo por tope es el checkpoint de la lección pasada y ya pasó ahí. Si de todos modos quieres la transcripción idéntica de cuatro líneas, son dos agregados pequeños: un `app.get('/price/rush', ...)` en `price-api.ts` que devuelva la misma forma de cotización, y una segunda entrada bajo `endpoints:` en `paywall.yml` con `path: "price/rush"` y un `price_usd` por encima del tope del agente. Opcional, y vale hacerlo una vez si quieres ver una ruta con gate rechazar en lugar de dar 404.

Tercero, acuérdate de la ausencia que nombró el paso 3: el camino del gate no escribe ninguna fila de libro mayor, así que esta liquidación aparece en tu transcripción y en el explorador, no en `orders.jsonl`. La misma API, los dos protocolos, un solo archivo de config. Checkpoint: ahora sostienes una transcripción de terminal con un 402 sin pagar que muestra los dos desafíos, una llamada negociada por MPP con un `Payment-Receipt`, y una liquidación de x402 a través del mismo gate.

Fíjate en lo que no le pasó a tu escalera mientras hacías eso. Ningún artefacto nuevo nació hoy.

![La API de precios de prensado como un solo peldaño de artefacto en dos capas: el middleware de pago que todavía es dueño de la conciliación por memo, y el pay gate de hoy respondiendo los dos protocolos.](assets/v07-diagram.png)

## Challenge

Ninguna caminata guiada en este. Tú eres el integrador, y el criterio de un integrador es el entregable.

Escríbele a Wavelength un **memo de apuesta**, en tres partes:

1. **La transcripción.** Tu captura del lab: el 402 de doble desafío, la liquidación por MPP de `pay curl`, el agente de x402 liquidando a través del mismo gate.
2. **La tabla de merchant-of-record.** Cuatro líneas, una por estándar (x402, MPP, AP2, ACP), cada una nombrando quién es merchant-of-record y una cláusula sobre por qué. Escríbela a partir de la comparación que acabas de estudiar, en tus propias palabras; se la vas a defender a un comercio que nunca ha oído ninguno de estos acrónimos.
3. **La elección.** Una sola oración que nombre a qué debería apostarle Wavelength hoy, con su razón. Hay una respuesta defendible en los cuatro estándares que acabas de comparar, y no es el nombre de un protocolo. Si tu oración nombra un solo estándar, vuelve a leer los modos de falla de la sección de la apuesta y discute conmigo en el margen del memo sobre por qué el tuyo los sobrevive.

Acepta cuando: la transcripción muestre los dos protocolos liquidando a través de un solo gate, las cuatro afirmaciones de merchant-of-record de la tabla sean correctas, y la oración de la elección nombre tanto la estrategia como su costo.

## Checkpoint, y la pregunta que hay debajo

Si el lab te peleó, haz el triage en este orden. Que `pay --version` falle quiere decir que la instalación global falta o está sombreada; reinstala y comprueba tu PATH. Un comando que se detiene a correr `pay setup` quiere decir que te saltaste la cuenta del paso 1; dale un backend de keystore y vuelve a correrlo. Un gate que rechaza `paywall.yml` con `Invalid paywall: missing field <name>` es deriva de esquema, y el arreglo es mecánico: vuelve a correr `pay server scaffold` hacia un archivo de prueba, diféralo contra el tuyo y reconcilia campo por campo, porque el archivo generado sigue al binario mucho mejor de lo que puede cualquier tutorial. Y si `pay gate` mismo ha cambiado sin parar hasta quedar irreconocible para cuando leas esto, degrádate con elegancia exactamente como estaba planeado: corre `pay curl` contra el middleware de x402 de la lección pasada en su lugar, y todavía te llevas la mitad del lado del cliente de la lección, una CLI negociando un 402 sin que tú escribas código de protocolo. Documenta el camino que hayas tomado; una nota fechada de "esto funcionó el 2026-08-22 con pay 0.26.0 de `@solana/pay` 1.0.26" es precisamente la disciplina que todo este módulo ha estado enseñando.

Da un paso atrás y mira lo que la tienda puede hacer ahora. Los humanos pagan a través de páginas de checkout, puestos con QR y blinks. Las máquinas pagan a través de x402 y MPP, negociados por un gate que configuras en vez de código que mantienes. Una costura sigue honestamente abierta, y deberías poder nombrarla: las ventas que corren por el middleware de la lección pasada se concilian en el libro mayor por id de factura, porque cableaste ese hook tú mismo, mientras que la contabilidad `pooled` del gate no te entrega ningún memo por llamada, así que las llamadas ruteadas por el gate son trabajo de libro mayor todavía pendiente antes de salir al aire. Todo lo demás está armado. El comprador de discos de 1949 esperó a que pasara la guerra de formatos; el fabricante de tocadiscos de 1950 vendió a través de ella. Acabas de construir el reproductor multivelocidad, y sabes cuál tornillo sigue flojo.

Una pregunta sigue abierta, y ha estado escondida a plena vista desde la sección de MPP. El default del modo pull deja que el servidor co-firme como fee payer, lo que quiere decir que alguien distinto del comprador cubrió la comisión de red. Generaliza eso y te sale el tema entero del próximo módulo: ¿quién paga cuando el cliente tiene cero SOL? El rol de co-firmante que acabas de conocer crece hacia el patrocinio de Kora y el checkout sin gas, porque nadie lleva SOL a una feria de discos. Nos vemos allá.
