# Onramps, y el camino de vuelta

## Resumen

El módulo de suscripciones cerró con la máquina de estados de dunning convirtiendo una renovación fallida en una factura abierta; el club del disco del mes ahora se maneja solo. Pero cada peldaño, desde la primera transferencia hasta el club, dio por sentado que el USDC ya había llegado a una billetera. En una feria de discos de verdad, la mitad del público tiene tarjeta y nada de cripto, y uno de tus artistas quiere retirar las regalías del mes pasado a una cuenta bancaria. El dinero tiene que entrar, y tiene que volver a salir. Ninguna de las dos direcciones es tuya para custodiar, y esta lección se trata de cablear las dos sin tocar nunca ninguna.

Antes de cualquier teoría, corre esta sola línea en tu terminal (`node` está en tu máquina desde el módulo 1):

```bash
node -e "const u = new URL('https://pay.example/buy?address=BuyerWa11etXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX&asset=USDC'); u.searchParams.set('address', 'AttackerXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'); console.log(u.toString())"
```

Una línea de JavaScript acaba de reescribir a dónde irían los fondos de un comprador, en cualquier flujo lo bastante descuidado como para llevar el destino en una URL. Esa es toda la lección de seguridad del día en miniatura: cualquier cosa que esté en una URL del cliente es escribible por el atacante, así que la dirección de destino nunca puede viajar en una. El arreglo se llama session token, y es la columna vertebral de todo lo que construimos abajo.

Los hallazgos por delante:

- **El camino de entrada** es el flujo embebido headless de Coinbase Onramp. Tu servidor acuña un **session token** de un solo uso que vincula la dirección de billetera del comprador y el activo por cobrar; la URL del cliente lleva solo ese token. Una URL manipulada no puede redirigir fondos, porque la dirección no está en la URL para manipularla.
- Desde el 31 de julio de 2025, toda URL de Coinbase Onramp y Offramp tiene que inicializarse con un session token. La era de la dirección en la URL se terminó por mandato, no solo por buen gusto.
- **El onramp de fiat a cripto de Stripe** es la otra forma de integración: Stripe es merchant-of-record, y se come el fraude, las disputas y el KYC. Está en public preview, lo que quiere decir que sus límites son tus límites.
- **El camino de salida** es Coinbase Offramp, y es solo alojado: rediriges al usuario al flujo propio de Coinbase en vez de embeberlo. El riel de payout depende de dónde esté el artista: ACH es un riel bancario de EE. UU., PayPal sirve a una lista de países selectos, y el menú en vivo por región es config que consultas, no un dato que recuerdas.
- La distinción estructural del módulo: un procesador que liquida tus ventas en un saldo fiat y un usuario que hace el off-ramp de su propio USDC a su banco son dos actores distintos con dos superficies de KYC distintas. Saber cuál es cuál en cada costura es todo tu trabajo de compliance como dev. Tampoco es asesoría legal, y lo voy a decir de nuevo donde importa.

El artefacto es `ramp-embed`, crecido a partir del mismo workspace `wavelength-checkout`: una ruta de servidor que acuña el session token, un traspaso al cliente que abre la URL del onramp, y una prueba de humo que demuestra que la dirección nunca se filtra. Cómo se reparte el trabajo de hoy: el handler de sesión se entrega como un esqueleto con dos huecos TODO bien a la vista y la teoría tiene las dos respuestas textuales; el offramp es un recorrido guiado porque no puedes automatizar de forma significativa el flujo de KYC alojado de otro; y el desafío de código en solitario te entrega una integración que funciona pero tiene fugas para reparar, en vez de un archivo en blanco.

![El checkout existente está en el centro, con un comprador con tarjeta entrando por el Coinbase Onramp embebido y un artista saliendo por el Coinbase Offramp alojado hacia un banco.](assets/v01-diagram.webp)

## La frontera fiat

### Las rampas, y a quién sirven

Una rampa es una casa de cambio con un departamento de cumplimiento. La dirección **onramp** toma un pago con tarjeta o bancario de un usuario y entrega cripto a una dirección que ese usuario nombra; la dirección **offramp** toma su cripto y entrega fiat a una cuenta bancaria que le pertenece. En las dos direcciones, el cliente de la rampa es la persona que quiere cambiar de activo, no tú. Wavelength nunca sostiene el número de tarjeta, nunca sostiene el fiat, y nunca sostiene el USDC del comprador. Estás cableando una puerta, no una bóveda.

Existen dos formas de integración, y el vocabulario importa para el resto del módulo. Una integración **alojada** quiere decir que rediriges al usuario a las páginas propias del proveedor y que vuelve cuando terminó. Una integración **headless** (o embebida) quiere decir que el flujo del proveedor corre adentro de tu producto, bajo tu estilo y tus pantallas, mientras el proveedor sigue manejando el dinero y el KYC por debajo. Lo alojado es una derivación; lo headless es un componente. Coinbase Onramp ofrece el camino headless, que es la razón por la que se lleva la construcción de hoy. Coinbase Offramp, como vamos a ver, a propósito no.

¿Por qué le importa siquiera a una tienda de discos? Porque la matemática del comprador es brutal. Cada lección hasta ahora dio por sentada una billetera que ya sostiene USDC, que en una feria física describe quizá la porción cripto-nativa del público. Todos los demás tienen una tarjeta. Si la respuesta a "¿puedo pagar?" es "primero ve a crear una cuenta de exchange, completa el KYC ahí, compra USDC, retíralo a una billetera que además tienes que instalar, y después vuelve," no perdiste una venta, perdiste el segmento entero. El onramp colapsa todo eso en: toca comprar, Apple Pay, el USDC aparece en la billetera, paga el checkout. Los mismos rieles que ya construiste. Público nuevo.

Y esto dejó de ser hipotético para los consumidores hace rato. La ola de rieles de consumo de abril de 2026 volvió el patrón mainstream: Meta empezó a pagarles a los creadores en USDC sobre Solana en Colombia y Filipinas, MetaMask Card empezó a gastar USDC de Solana sobre Mastercard en terminales comunes, y Solflare entregó onramps de Coinbase Apple Pay directamente adentro de la billetera (los tres momentos salen del resumen de ecosistema de abril de 2026 de la Solana Foundation en solana.com, el mismo resumen que citan lecciones posteriores; afirmaciones fechadas, así que vuelve a comprobarlas antes de repetirlas). La plomería que estás por construir es la misma plomería, una tienda más chica.

![Una línea de tiempo de tres rieles de consumo de abril de 2026, los payouts a creadores de Meta, el gasto con MetaMask Card, y los onramps de Solflare con Apple Pay, con flechas hacia adelante hasta el embed de la tienda de esta lección.](assets/v02-timeline.webp)

### El session token: vincula del lado del servidor, nunca filtres

Aquí está la columna vertebral didáctica de la lección, y es una propiedad de seguridad de verdad, no relleno de integración.

El embed ingenuo se ve como la sola línea que corriste arriba: construye una URL en el cliente, agrega la dirección de billetera del comprador y tu app ID como parámetros de consulta, ábrela. Incluso funciona. El problema es que una URL es dato en manos del atacante. Una extensión de navegador maliciosa, una dependencia comprometida, un man-in-the-middle en una red mala, cualquiera de ellos puede reescribir `address=` antes de que se abra la ventana, y la tarjeta del comprador financia alegremente la billetera de un desconocido. El comprador le echa la culpa a tu tienda, y la disputa aterriza en algún lugar caro. Voy a ser honesto: la primera integración de onramp que bosquejé tenía la dirección en el query string, porque todos los quickstarts de esa época la tenían. La industria aprendió mejor en público.

El flujo de session-token saca la dirección de la superficie de ataque por completo. Tres pasos:

1. Tu **servidor** llama a la API de session token de Coinbase, autenticada con tu CDP API key, y el cuerpo del request vincula el destino: la dirección de billetera del comprador, las blockchains en las que es válida, y opcionalmente qué activos puede recibir la sesión.
2. Coinbase devuelve un **session token**: una credencial de un solo uso, que vence a los cinco minutos, que internamente referencia todo lo que el request vinculó.
3. El **cliente** abre la URL del onramp llevando solo ese token. La dirección nunca aparece en la URL, así que no hay nada que manipular. Reescribe el token y la sesión simplemente falla; no se la puede redirigir, solo romper.

Sé preciso sobre lo que el token protege y lo que no, porque una afirmación de seguridad que se pasa de rosca invita a su propia refutación de treinta segundos. El token saca la dirección de todo lo que está aguas abajo de la acuñación: la URL, la ventana abierta, cualquier link copiado, logueado o filtrado, que es donde los destinos viven más tiempo y se reescriben más fácil. Lo que no hace es autenticar la intención. La ruta `/session` del lab sigue aceptando la dirección de un POST del cliente, así que un atacante que pueda reescribir requests adentro del navegador del comprador podría manipular un salto antes, en el cuerpo en vez de en la URL. Una tienda de producción cierra ese salto vinculando la dirección a algo que el cliente no puede falsificar, una sesión logueada cuya billetera se enlazó en el registro, o una firma de billetera conectada que demuestre control de la dirección; el lab deja afuera esa capa de auth porque le pertenece a tu app, no a la rampa. Afirmación más angosta, igual vale la construcción.

La propiedad de la que agarrarse: **vincula del lado del servidor, nunca filtres**. El valor sensible vive en una llamada autenticada de servidor a servidor; el cliente lleva una referencia opaca. Si usaste los PaymentIntents de Stripe, esta es la misma forma (un intent creado en el servidor, un secreto del lado del cliente que lo referencia), y el solapamiento es la respuesta estándar a "el cliente quiere arrancar un flujo que el cliente no tiene que poder dirigir."

![Un flujo de cuatro saltos donde el cliente pide una sesión, tu servidor vincula la dirección dentro de un token de Coinbase, y el cliente abre una URL en la que la manipulación no lleva a ningún lado.](assets/v03-flowchart.webp)

Las formas concretas, verificadas contra los docs en vivo de Coinbase hoy, son lo bastante chicas como para memorizarlas. La acuñación es un POST a `https://api.developer.coinbase.com/onramp/v1/token` con un Bearer JWT generado a partir de tu CDP API key, y el cuerpo que vincula un destino de USDC en Solana es exactamente este:

```json
{
  "addresses": [{ "address": "<buyer wallet address>", "blockchains": ["solana"] }],
  "assets": ["USDC"]
}
```

La respuesta es un `{ "token": "...", "channel_id": "..." }` plano (`channel_id` son metadatos del flujo de guest-checkout de Coinbase; el camino del widget necesita solo `token`, que es la razón por la que la ruta de servidor del lab descarta el resto). Y la URL del cliente que abre tu tienda se construye a partir del token más los valores por defecto de pantalla:

```
https://pay.coinbase.com/buy/select-asset
  ?sessionToken=<token>
  &defaultNetwork=solana
  &defaultAsset=USDC
  &presetFiatAmount=12.5
  &fiatCurrency=USD
```

Lee esa URL dos veces y fíjate en lo que falta: ninguna dirección, ningún app ID. `defaultNetwork` y `defaultAsset` son presets de experiencia de usuario (eligen en qué pantalla de activo abre el widget), y `presetFiatAmount` pre-completa la compra con el precio del disco para que el comprador aterrice en una pantalla que ya dice el número correcto. Ninguno de ellos es relevante para la seguridad. El único parámetro estructural es `sessionToken`, y es opaco. Esa asimetría, presets aburridos en la URL, la vinculación sensible detrás del token, es el diseño.

![La URL del onramp anotada línea por línea, con sessionToken marcado como estructural, cuatro presets de pantalla marcados como cosméticos, y la dirección de billetera y el app ID ausentes por diseño.](assets/v04-annotated-code.webp)

Los dos tiempos de vida del token también son parte de la propiedad, no trivia. De un solo uso quiere decir que una URL capturada no se puede repetir para abrir una segunda sesión de fondeo contra la misma vinculación, y el vencimiento a los cinco minutos quiere decir que un link filtrado muere antes de poder circular. Tu servidor acuña por clic, en el momento de la intención. Cachea un session token como cachearías una cotización de precio y el mejor caso es un link muerto, vencido o ya consumido, servido a un comprador de verdad en el momento de la compra; el peor caso es una vinculación acuñada para un comprador y entregada a otro. La acuñación cuesta un solo viaje de ida y vuelta autenticado, así que no hay nada que valga la pena ahorrar.

![La vida de un session token desde la acuñación por clic pasando por un solo uso y el vencimiento a los cinco minutos, con los tokens repetidos y cacheados mostrados sin salida fuera de la línea.](assets/v05-timeline.webp)

Una nota práctica sobre lo que recibe el comprador. El destino que vinculas es la dirección de billetera del comprador, y Coinbase entrega USDC a la associated token account derivada de ella, la misma derivación de ATA que aprendiste cuando Wavelength recibió USDC por primera vez en el módulo 2. El comprador no necesita pre-crear nada. Sale del flujo sosteniendo exactamente el saldo que tu checkout sabe cobrar. El USDC de mainnet en Solana es el mint `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`; tu checkout de devnet cobra el mint sustituto de devnet, que es la razón por la que la corrida en vivo del lab es una sesión en sandbox y no una de devnet. Los onramps son un producto de mainnet. Las tarjetas de verdad compran dólares de verdad.

Otras dos piezas de la superficie de Coinbase pertenecen a tu mapa, las dos dichas a nivel de concepto porque el camino del widget de arriba es el que construimos. Primero, el **camino headless de Apple Pay**: más allá del widget, la Onramp Order API te deja ir totalmente headless. Tu servidor cotiza y crea el pedido y recibe de vuelta un link de pago, tú renderizas ese link en un webview o iframe, y lo que el comprador ve es el botón de Apple Pay o Google Pay de Coinbase parado adentro de tu propia UI. Una corrección que vale la pena cargar, porque es fácil suponer lo contrario: ese camino se autentica con la misma CDP API key firmada dentro de un Bearer JWT, pero no toma un session token. Los session tokens inicializan las URLs del widget alojado; la order API es una llamada de API autenticada y común. Cuando tu tienda le queda chica a las pantallas del widget, esa es la puerta; comprueba la referencia de API actual cuando la atravieses, porque la superficie headless es la parte que más rápido se mueve del producto. Segundo, el **trial mode**: los docs de onboarding de Coinbase describen un nivel de prueba para integraciones nuevas, con tamaños de transacción limitados antes de la aprobación completa. Trata eso como una característica y no como una molestia: le da a este lab una compra con tarjeta real de punta a punta, con ruedas de apoyo en los montos. Para qué está autorizada tu app vive en tu dashboard de CDP, no en esta lección.

### La otra forma: Stripe como merchant-of-record

El flujo headless de Coinbase no es la única manera de atornillarle una puerta fiat a una tienda, y la alternativa enseña por contraste la segunda palabra de vocabulario de la lección.

**Merchant-of-record** es la entidad cuyo nombre está en el cargo: la que la red de tarjetas hace responsable, contra la que se presenta la disputa, la que tiene que conocer a su cliente. Stripe corre su onramp de fiat a cripto, actualmente en public preview, con Stripe mismo como merchant-of-record. Embebe el onramp de Stripe y la compra de cripto con tarjeta es la venta de Stripe, no la tuya. Stripe corre el KYC, Stripe se come el fraude, Stripe absorbe el chargeback cuando un comprador disputa el cargo tres días después. Lo que cediste a cambio es control: la cobertura geográfica de Stripe es tu cobertura geográfica, la lista de activos de Stripe es tu lista de activos, y public preview quiere decir que las dos pueden desplazarse debajo de ti, con la etiqueta de preview como tu única advertencia. Los techos específicos, qué países, qué activos, qué montos, viven detrás de los docs de preview actuales de Stripe y se mueven sin aviso, que es exactamente por qué esta lección no los imprime; la próxima lección convierte ve-a-leer-la-matriz-actual-del-proveedor en un hábito calificado en vez de un encogimiento de hombros.

Para sentir por qué importa ese asiento, corre un escenario de juguete con números redondos, ilustrativos e inventados a propósito: la feria vende 1,000 discos a $12.50 a través de onramps fondeados con tarjeta, el fraude de tarjeta no presente corre a un 1% más o menos típico, y las redes de tarjetas le cobran al merchant-of-record una comisión fija de disputa por cada una, a menudo más que la venta misma; digamos $15.

```typescript
// dispute-math.ts - who eats the chargebacks? Whoever is merchant-of-record.
const sales = 1_000; // records sold
const price = 12.5; // USD each
const disputeRate = 0.01; // card-not-present, typical-ish
const feePerDispute = 15; // fixed network fee, USD - often more than the sale

const disputes = sales * disputeRate;
const reversed = disputes * price;
const fees = disputes * feePerDispute;
console.log(`${disputes} disputes -> $${reversed} reversed sales + $${fees} in network fees`);
// 10 disputes -> $125 reversed sales + $150 in network fees
```

Diez compras disputadas. Quien esté sentado en el asiento pierde los $125 de ventas revertidas más $150 en comisiones más el tiempo de ops de pelear diez disputas, y si el ratio de disputas sube lo bastante alto, las redes de tarjetas meten la cuenta entera en un programa de monitoreo. Ahora fíjate quién es ese quien: no tú. La compra de cripto fue la venta de Coinbase o la venta de Stripe, y la maquinaria de disputas los mastica a ellos. Lo que tú vendiste fue un disco, pagado en USDC que ya se había compensado con la finality de Solana por debajo. Esa es la razón callada y estructural por la que una tienda liquidada en cripto quiere un socio de rampa delante en vez de su propio adquirente de tarjetas: la máquina de chargebacks sigue existiendo, solo que no está apuntada a ti.

El instinto aquí es preguntar qué forma gana, y es un poco el instinto equivocado. Las dos formas ponen al proveedor en el asiento de merchant-of-record para la compra de cripto; estás eligiendo profundidad de integración y superficie de proveedor, no responsabilidad. La comparación honesta:

![Coinbase Onramp y el onramp de Stripe comparados: los dos hacen al proveedor merchant-of-record y dueño del KYC; se diferencian en la forma de integración, el estado del producto, y contra cuál construye esta lección.](assets/v06-comparison.webp)

El patrón se generaliza más allá de estos dos proveedores, que es por qué vale la pena internalizarlo ahora: quien sea merchant-of-record es dueño del fraude, de las disputas y de las comprobaciones de identidad, y a cambio es dueño del mapa de cobertura. Vas a encontrarte el mismo canje con otro conjunto de proveedores la próxima lección, y para el final de ella, "¿quién es merchant-of-record aquí?" debería ser la primera pregunta que le haces a cualquier proveedor de pagos, justo antes de "¿y en qué corredores?"

### El camino de vuelta

Ahora el artista con las regalías del mes pasado. Wavelength les paga a sus artistas de la misma manera en que le pagan, en USDC a una billetera que el artista controla, así que el saldo de regalías ya está sentado en su custodia, sus claves, nada tuyo queda adentro. El paso que falta es USDC a una cuenta bancaria, y la herramienta es Coinbase Offramp.

La restricción de integración que le da forma a todo: **Offramp es solo alojado.** No hay embed headless para el camino de salida. Tu producto acuña un session token exactamente como antes, mismo endpoint, misma disciplina de vinculación, y después redirige al artista al flujo alojado propio de Coinbase en `https://pay.coinbase.com/v3/sell/input`, llevando el `sessionToken`, un `partnerUserRef` (tu referencia opaca por usuario, de menos de 50 caracteres), y un `redirectUrl` en tu dominio de allowlist al que Coinbase manda de vuelta al artista cuando termina. Adentro del flujo alojado, el artista se autentica con Coinbase, el KYC corre como su relación con Coinbase y no contigo, manda el USDC para adentro, y el payout aterriza en el riel de retiro que Coinbase ofrezca donde esté el artista. Ese menú es regional, y los docs propios de Coinbase lo dicen exactamente en la forma en que deberías anotarlo: **transferencias bancarias ACH (US)** y **PayPal (países selectos)**, más un saldo simple de Coinbase, con una API de config que devuelve la lista por país (re-comprobado 2026-09-02). Un artista de EE. UU. y un artista de São Paulo que caminan el mismo flujo ven salidas distintas, así que nunca prometas un riel específico en tu UI de payout; promete la caminata. Después vuelven a tu `redirectUrl` y tu producto retoma el hilo.

¿Por qué Coinbase embebería el camino de entrada y alojaría el camino de salida? Sigue el riesgo. El fraude de onramp es fraude de tarjeta, un problema al que los proveedores pueden ponerle precio y comerse a escala. El offramp es por donde el lavado de dinero sale hacia el sistema bancario, y el proveedor quiere ese flujo por completo en sus propias páginas, bajo su propia sesión, sin ninguna UI controlada por un socio cerca. Pierdes la UX embebida para la salida; a cambio el compliance del payout no toca tu producto en absoluto. Como canjes van, tómalo, todas las veces.

![La caminata del offramp, donde tu producto acuña un token y redirige hacia afuera, después de lo cual el inicio de sesión, el KYC, el envío de USDC y el payout al banco pasan todos en las páginas de Coinbase.](assets/v07-flowchart.webp)

### Dos flujos que se parecen y no lo son

Aquí está la distinción que este módulo no te va a dejar difuminar, porque difuminarla es como los desarrolladores se convencen de que tienen superficies de compliance que no tienen, o peor, de que no tienen las que sí.

**La liquidación fiat del comercio** es un procesador actuando por ti, el comercio: acepta tus ventas y las liquida en un saldo fiat a tu nombre. El KYC que importa ahí es el procesador dando de alta a *tu negocio* (las comprobaciones de know-your-business que aceptas cuando abres la cuenta), y el procesador está en el asiento de merchant-of-record o de adquirente para esas ventas. **Un off-ramp de usuario** es el comprador o el artista actuando por sí mismos: su activo, su cuenta bancaria, su verificación de identidad con el proveedor de la rampa. Distinto actor, distinta dirección, distinta superficie de KYC, distinta responsabilidad. El offramp que acabas de caminar es del segundo tipo. El flujo de liquidación de Stripe que vas a encontrarte la próxima lección es del primer tipo. Los dos "convierten cripto en fiat," y esa frase es precisamente la confusión que hay que rechazar: nombra al actor y los flujos se separan limpiamente.

Tu superficie de compliance como dev se dice honestamente en una sola oración: tienes que poder decir, en cada costura de tu producto donde fiat y cripto se tocan, qué actor está moviendo dinero y quién es merchant-of-record para ese movimiento. Ese es un trabajo de describir, no un trabajo de operar. No corres ninguno de los dos flujos. Y para decirlo sin vueltas, porque este rincón del curso roza territorio regulado: esta es una lectura de ingeniería sobre dónde están las costuras, no asesoría legal, y un producto de servicios de dinero de verdad trae un abogado de verdad.

![Tres costuras mapeadas, con Coinbase como merchant-of-record tanto de su onramp como de su offramp, Stripe del suyo propio, y Wavelength describiendo cada costura sin operar ninguna.](assets/v08-diagram.webp)

### Lo que verifiqué, y en lo que no me tienes que creer

> **Verificado al escribir, 2026-08-23.** La API de session token (`POST https://api.developer.coinbase.com/onramp/v1/token`, auth con Bearer JWT, cuerpo `addresses`/`blockchains`/`assets`, respuesta plana `{ token }`, de un solo uso, vencimiento a los cinco minutos), el mandato de session-token en todas las URLs de Onramp y Offramp desde 2025-07-31, la URL base del offramp y sus parámetros obligatorios `sessionToken`, `partnerUserRef` y `redirectUrl`, ACH más PayPal entre los métodos de retiro del offramp, el host de sandbox, y el hecho de que la Order API headless se autentica con un JWT de CDP y no toma ningún session token fueron todos comprobados contra los docs de desarrollador en vivo de Coinbase en esta fecha.
>
> **A propósito no congelado aquí: la cobertura de activos y geográfica de Coinbase en Solana.** Qué activos vende el onramp en Solana, en qué países y estados de EE. UU., con qué límites, es configuración del proveedor que se mueve sin aviso, y cualquier lista impresa en un curso queda obsoleta a la semana siguiente. En el momento de integrar, consúltala: las Onramp APIs exponen un endpoint de buy-options (`GET https://api.developer.coinbase.com/onramp/v1/buy/options?country=US`, mismo host y misma auth con Bearer JWT que la acuñación del token) que devuelve la matriz en vivo de activos y métodos de pago para un país dado. Trata la cobertura como config que consultas, nunca como un dato que recuerdas. Un corredor que el proveedor no sirve es un corredor que tú no puedes servir, y quieres aprender eso de una respuesta de API en staging, no de un comprador en producción.

Esa caja también es la contrapartida de la lección dicha honestamente, así que no la voy a enterrar: las rampas te dan alcance al precio del control. El procesador es dueño del KYC, de la cobertura geográfica, de los rieles de payout, del estado de preview y de los límites de trial, y cada uno de esos es un techo sobre tu producto que no llegas a negociar en código. Embeber una rampa también planta una costura de compliance adentro de tu tienda que tienes que poder describir aunque no la operes. La alternativa, convertirte tú mismo en el transmisor de dinero regulado, es tanto peor para una tienda de discos que el canje apenas merece el nombre. Pero es un canje, y la caja de cobertura es donde muerde.

## Lab: cablea la puerta fiat

Construcción numerada, en el workspace `wavelength-checkout` que creaste en el módulo 3. `ramp-embed/` es una carpeta adentro de `wavelength-checkout`, no un workspace nuevo: va al lado de la carpeta `checkout/` que el módulo 3 construyó ahí, que es lo que le deja importar el precio del disco directamente; los workspaces de ops y de facturación de los módulos 4 y 5 están en otra parte del repo y hoy no participan. El handler de sesión se entrega con dos huecos TODO, y la construcción corre hasta una falla nombrada con ellos puestos; el Challenge los cierra. Una dependencia nueva, necesaria solo para la ruta de servidor (la prueba de humo corre limpia sin ella):

```bash
npm install @coinbase/cdp-sdk
```

Ese es el CDP SDK de Coinbase (línea 1.x a agosto de 2026; comprueba npm antes de fijar), usado aquí para exactamente una cosa: generar el JWT de vida corta que autentica tu servidor contra el endpoint del token. Firmarlos tú mismo es posible y no vale la pena.

1. **El módulo de sesión, con los dos huecos.** Crea `ramp-embed/session.ts`. Funciones puras, sin I/O, que es lo que hace posible la prueba de humo:

   ```typescript
   export interface SessionTokenRequest {
     addresses: { address: string; blockchains: string[] }[];
     assets?: string[];
   }

   export interface OnrampUrlOptions {
     presetFiatAmount: number;
     fiatCurrency?: string;
     defaultAsset?: string;
   }

   // The server-side binding: this body is what pins the destination.
   export function buildSessionRequest(destinationAddress: string): SessionTokenRequest {
     // TODO(completion): return the body that binds destinationAddress to
     // USDC on Solana. The exact shape appears in the theory section.
     throw new Error('TODO: buildSessionRequest');
   }

   // The client handoff: the URL carries the token and display presets ONLY.
   export function buildOnrampUrl(sessionToken: string, opts: OnrampUrlOptions): string {
     // TODO(completion): build the pay.coinbase.com URL. sessionToken,
     // defaultNetwork, defaultAsset, presetFiatAmount, fiatCurrency.
     // The address does not appear. The theory section shows every param.
     throw new Error('TODO: buildOnrampUrl');
   }
   ```

2. **La ruta de servidor.** Crea `ramp-embed/server.ts`. La misma forma simple de `node:http` que el servidor del checkout, una sola ruta POST: el cliente manda la dirección de billetera del comprador, el servidor la vincula dentro de un session token y responde con la URL de onramp terminada. Las llaves vienen del dashboard de CDP (crea una API key bajo tu proyecto; tiene un ID y un secreto) y viven en variables de entorno, nunca en el código:

   ```typescript
   import { createServer } from 'node:http';
   import { generateJwt } from '@coinbase/cdp-sdk/auth';
   import { RECORD } from '../checkout/record.ts';
   import { buildOnrampUrl, buildSessionRequest } from './session.ts';

   const KEY_ID = process.env.CDP_API_KEY_ID;
   const KEY_SECRET = process.env.CDP_API_KEY_SECRET;

   async function mintSessionToken(destinationAddress: string): Promise<string> {
     if (!KEY_ID || !KEY_SECRET) {
       throw new Error('set CDP_API_KEY_ID and CDP_API_KEY_SECRET');
     }
     const jwt = await generateJwt({
       apiKeyId: KEY_ID,
       apiKeySecret: KEY_SECRET,
       requestMethod: 'POST',
       requestHost: 'api.developer.coinbase.com',
       requestPath: '/onramp/v1/token',
       expiresIn: 120,
     });
     const res = await fetch('https://api.developer.coinbase.com/onramp/v1/token', {
       method: 'POST',
       headers: {
         Authorization: `Bearer ${jwt}`,
         'Content-Type': 'application/json',
       },
       body: JSON.stringify(buildSessionRequest(destinationAddress)),
     });
     if (!res.ok) {
       throw new Error(`session token mint failed: ${res.status} ${await res.text()}`);
     }
     const body = (await res.json()) as { token: string };
     return body.token;
   }

   const server = createServer((req, res) => {
     if (req.method !== 'POST' || req.url !== '/session') {
       res.writeHead(404);
       res.end();
       return;
     }
     let raw = '';
     req.on('data', (chunk) => {
       raw += chunk;
     });
     req.on('end', async () => {
       try {
         const { address } = JSON.parse(raw) as { address: string };
         const token = await mintSessionToken(address);
         const url = buildOnrampUrl(token, { presetFiatAmount: RECORD.priceUsdc });
         res.writeHead(200, { 'Content-Type': 'application/json' });
         res.end(JSON.stringify({ url }));
       } catch (err) {
         res.writeHead(500, { 'Content-Type': 'application/json' });
         res.end(JSON.stringify({ error: (err as Error).message }));
       }
     });
   });

   server.listen(3200, () => {
     console.log('ramp-embed session route on :3200');
   });
   ```

   Fíjate en la única línea que hace el trabajo de escalera de artefactos: `presetFiatAmount: RECORD.priceUsdc`. La sesión de onramp toma su precio de la misma definición de disco que cobra el checkout, siendo 12.5 USDC igual a 12.5 dólares porque un peg al dólar cobrado 1:1 no necesita paso de conversión. Una comprobación de unidades antes de seguir, porque este curso viene machacando unidades base desde hace cinco módulos: `presetFiatAmount` va en unidades fiat enteras, y `RECORD.priceUsdc` en tu checkout es el `12.5` a escala humana, nunca unidades base; si tu archivo de disco alguna vez tuvo `12_500_000`, convierte antes de esta línea o el widget te va a ofrecer educadamente un prensado de doce millones de dólares. Adentro del workspace `wavelength-checkout`, `ramp-embed/` consume a su hermano `checkout/`; nada está duplicado.

3. **La prueba de humo.** Crea `ramp-embed/smoke.ts`, la verificación offline estándar del módulo. Ejercita las dos funciones puras y afirma la propiedad de seguridad directamente, sin red y sin llaves:

   ```typescript
   import { RECORD } from '../checkout/record.ts';
   import { buildOnrampUrl, buildSessionRequest } from './session.ts';

   const DEMO_ADDRESS = 'Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS';

   const request = buildSessionRequest(DEMO_ADDRESS);
   const binds = request.addresses.some(
     (entry) => entry.address === DEMO_ADDRESS && entry.blockchains.includes('solana'),
   );
   if (!binds) {
     throw new Error('session request must bind the address on solana');
   }

   const url = new URL(
     buildOnrampUrl('demo-session-token', { presetFiatAmount: RECORD.priceUsdc }),
   );
   if (url.hostname !== 'pay.coinbase.com') {
     throw new Error('onramp URL must live on pay.coinbase.com');
   }
   if (url.searchParams.get('sessionToken') !== 'demo-session-token') {
     throw new Error('client URL must carry the sessionToken');
   }
   if (url.searchParams.get('defaultNetwork') !== 'solana') {
     throw new Error('defaultNetwork must be solana');
   }
   if (url.searchParams.get('presetFiatAmount') !== String(RECORD.priceUsdc)) {
     throw new Error('the preset fiat amount must carry through');
   }
   if (url.toString().includes(DEMO_ADDRESS)) {
     throw new Error('the wallet address leaked into the client URL');
   }

   console.log('session request binds the address server-side');
   console.log('wallet address absent from the client URL');
   console.log(url.toString());
   ```

4. **Córrelo hasta la falla nombrada.**

   ```bash
   npx tsx ramp-embed/smoke.ts
   ```

   Con los huecos puestos esto muere con `TODO: buildSessionRequest`, y esa falla exacta es el checkpoint de la parte trabajada. Cualquier otra cosa quiere decir un typo más arriba: el sospechoso de siempre es la ruta de import relativa a `checkout/record.ts`, que tiene que trepar fuera de `ramp-embed/` con `../`.

![Los tres archivos de ramp-embed, constructores de sesión puros, una ruta de servidor autenticada, y una prueba de humo que demuestra que la dirección está vinculada del lado del servidor y ausente de la URL del cliente.](assets/v09-diagram.webp)

5. **La caminata de sesión, sandbox por defecto.** Este paso necesita llaves de CDP, y la caminata del offramp del paso 6 también, así que guárdate el par junto para cuando tengas credenciales; solo la caminata opcional con tarjeta real al final necesita además la autorización de trial mode de tu app. Si hoy estás offline o sin llaves, la prueba de humo sola completa la construcción de la lección, y los pasos 5 y 6 pueden esperar. Exporta `CDP_API_KEY_ID` y `CDP_API_KEY_SECRET`, arranca la ruta con `npx tsx ramp-embed/server.ts`, y después juega al cliente de la tienda desde una segunda terminal:

   ```bash
   curl -s -X POST http://localhost:3200/session \
     -H 'Content-Type: application/json' \
     -d '{"address":"<your own mainnet wallet address>"}'
   ```

   El JSON que vuelve sostiene una URL de onramp en vivo, y el token crudo está sentado adentro como el parámetro de consulta `sessionToken`. Acuérdate de lo que dijo la teoría sobre sus tiempos de vida: de un solo uso, cinco minutos. Cada curl acuña un token bueno para un intento, así que vuelve a correr el curl cuando necesites otro; cada caminata de abajo y el paso 6 reciben todos su propia acuñación fresca.

   La caminata por defecto es el sandbox, porque nada en esta lección necesita tu tarjeta: acuña un token fresco y entrégaselo a `https://pay-sandbox.coinbase.com/?sessionToken=<token>` con los mismos presets de pantalla. No se mueven fondos reales, y el punto de integración que se califica es el mismo que califica el host en vivo: la URL que abriste salió de tu ruta de servidor, vinculada antes de que el navegador la viera siquiera. Una nota de honestidad antes de que escribas nada en ese formulario de tarjeta. Esta lección imprimía los valores de prueba aceptados por el sandbox, y dejó de hacerlo, porque son datos de proveedor exactamente del tipo que este módulo te sigue diciendo que feches: los docs del sandbox de guest-checkout que verificamos al escribir (2026-08-23) ya no se podían encontrar en el índice de docs de Coinbase en una re-comprobación del 2026-09-02. Así que toma los valores de prueba de lo que digan los docs actuales de Coinbase, y si el host del sandbox rechaza tu token o el flujo se movió por completo, acuña de nuevo, vuelve a comprobar los docs, y cae de vuelta a la caminata opcional de abajo o a la prueba de humo; el trabajo calificado de esta lección nunca dependió de este formulario.

   La sesión con tarjeta real es la opcional, no la que viene por defecto. Si tienes autorización de trial mode y quieres ver producción, abre en un navegador la URL que viene del JSON y camínala como el comprador: el widget abre en USDC sobre Solana con 12.5 dólares pre-completados, y la hoja de Apple Pay o de tarjeta que muestra Coinbase es exactamente la superficie que vería un comprador de Wavelength. Si completas o no la compra chica es tu decisión y tu límite de trial. Cualquiera de los dos hosts completa este paso de forma legítima.

6. **La caminata del offramp, guiada.** Sin código nuevo, a propósito: el camino de salida está alojado, así que la construcción es una redirección que puedes armar a mano y el aprendizaje está en caminarla. Como el paso 5, esto necesita tus llaves de CDP. Acuña un session token fresco de la misma manera (mismo endpoint; la ruta de servidor del paso 5 ya lo hace, y para una sesión de venta la dirección vinculada es la billetera de la que sale el USDC en vez de un destino), y después arma la URL alojada: `https://pay.coinbase.com/v3/sell/input` con tu `sessionToken`, un `partnerUserRef` que nombre al artista en tus libros (cualquier string opaco de menos de 50 caracteres, nunca su identidad real), y un `redirectUrl` que tú controles. Un prerrequisito se esconde en ese último parámetro: los dominios de redirect se ponen en la allowlist en la configuración de Onramp de tu dashboard de CDP antes de que Coinbase los acepte, así que registra `http://localhost:3200` ahí primero, y cuando el flujo alojado se niegue a abrir, un redirect rechazado y no registrado es lo primero que hay que comprobar. Ábrela y narra lo que ves contra la teoría: el inicio de sesión es la relación del artista con Coinbase, las comprobaciones de identidad son de él y no tuyas, el envío de USDC sale de su billetera, y las opciones de payout que te muestran son la porción de tu región del roster de retiro de la teoría — ACH si estás sentado sobre una cuenta bancaria de EE. UU., PayPal donde Coinbase lo ofrezca, rara vez el roster entero de una sola vez. Después cierra el ciclo en voz alta, porque este es el músculo de la evaluación: di quién es merchant-of-record para el onramp embebido, para este offramp alojado, y para una compra con tarjeta por onramp de Stripe, una línea cada uno. Si alguno de los tres te lleva más de una oración, vuelve a leer el mapa de costuras antes de seguir.

## Challenge

**Completion.** Cierra los dos huecos en `session.ts`. `buildSessionRequest` devuelve el cuerpo de vinculación, `buildOnrampUrl` construye la URL de solo token; las dos formas están textuales en la sección de teoría, y el punto de escribirlas tú mismo es notar qué mitad de vincular-del-lado-del-servidor-nunca-filtrar impone cada una. La aceptación es que la prueba de humo pase entera:

```bash
npx tsx ramp-embed/smoke.ts
```

Tres líneas: la confirmación de vinculación, la confirmación de no-fuga, y una URL de `pay.coinbase.com` que contiene `sessionToken` y `defaultNetwork=solana` sin ninguna dirección de billetera en ningún lado. Código de salida 0.

**Solo.** El desafío de código colapsa las dos mitades del lab en una sola función, llamada en el momento del orden de tres saltos que acabas de caminar donde la acuñación ya respondió: `initHeadlessOnramp(destinationAddress, sessionToken, fiatAmount)` toma la billetera de destino, el token que tu servidor recibió de vuelta de la acuñación de sesión, y el monto fiat preestablecido, en ese orden, y devuelve `{ requestBody, onrampUrl }`, el cuerpo de vinculación que tu servidor POSTeó para ganarse ese token más la URL de solo token que abre el navegador del comprador. Lo que te entregan no es un esqueleto sino la integración ingenua del principio de esta lección, hecha concreta. Guárdala como `ramp-embed/naive-onramp.ts` y repárala en el lugar:

```typescript
// ramp-embed/naive-onramp.ts: the working-but-leaky integration, as promised.
// Four repairs, and the grader checks all of them: it binds the wrong chain,
// it leaks the raw address into the client URL, the URL never carries the
// sessionToken, and defaultNetwork points at the wrong network. The first two
// are the conceptual sins from the top of the lesson; the last two are what
// the fix has to put in their place.

interface OnrampInit {
  requestBody: {
    addresses: { address: string; blockchains: string[] }[];
    assets: string[];
  };
  onrampUrl: string;
}

function initHeadlessOnramp(
  destinationAddress: string,
  sessionToken: string,
  fiatAmount: number
): OnrampInit {
  const requestBody = {
    addresses: [{ address: destinationAddress, blockchains: ['ethereum'] }],
    assets: ['USDC'],
  };
  // Query built by hand rather than URLSearchParams: the challenge grader
  // runs in a bare JS realm without the web URL APIs.
  const params: [string, string][] = [
    ['address', destinationAddress],
    ['defaultNetwork', 'ethereum'],
    ['presetFiatAmount', String(fiatAmount)],
  ];
  const query = params
    .map(([k, v]) => `${k}=${encodeURIComponent(v)}`)
    .join('&');
  const onrampUrl = `https://pay.coinbase.com/buy/select-asset?${query}`;
  return { requestBody, onrampUrl };
}
```

Arregla las dos mitades contra los mismos criterios de aceptación que usó el lab. El request tiene que vincular el destino en Solana, la URL del cliente tiene que llevar el `sessionToken` con `defaultNetwork=solana` y el monto fiat preestablecido, y la dirección cruda nunca puede aparecer en la URL. Reescribir una integración con fugas es la versión de este ejercicio que te vas a encontrar de verdad en el trabajo, y el patrón viaja mucho más allá de este curso: todo proveedor que te entrega una API de "crea una sesión del lado del servidor, referénciala del lado del cliente" es exactamente esta forma con otros nombres de campo.

## Checkpoint, y la puerta se abre para los dos lados

Si la prueba de humo te pelea después de la completion, los mensajes de falla son el diagnóstico: una aserción de `blockchains` quiere decir que el cuerpo de vinculación está malformado (es un array de entradas de dirección, cada una con su propio array `blockchains`, un anidamiento fácil de aplanar sin querer), una aserción de `sessionToken` o `defaultNetwork` quiere decir que un nombre de parámetro se fue a la deriva (distinguen mayúsculas y minúsculas, `sessionToken` y no `sessiontoken`), y que dispare la aserción de fuga quiere decir que la dirección se abrió camino hasta los argumentos del constructor de la URL, que es la fuga que esta lección existe para volver estructuralmente imposible en todo lo que está aguas abajo de la acuñación. Y si la ruta de sandbox en vivo responde 401, tus JWT claims no coinciden con el request: el método, el host y el path en `generateJwt` tienen que ser exactamente el método, el host y el path que después consultas.

Mira la tienda funcionar de punta a punta: entra un comprador que no tiene más que una tarjeta, tu servidor vincula una sesión, Coinbase convierte la tarjeta en USDC en la propia ATA del comprador, y tu checkout, tu watcher y tu libro mayor existentes lo toman desde ahí sin aprender nada nuevo, una vez que mainnet es donde corren: la maquinaria es agnóstica al cluster, y hoy tu checkout todavía cobra el mint sustituto de devnet, así que el ciclo completo de tarjeta a checkout se cierra cuando el capstone mueve el stack a configuración de mainnet. Las regalías de un artista salen caminando por el otro lado hacia un banco sobre rieles que nunca tocas. Dinero adentro, dinero afuera, custodia en ningún lugar cerca de ti, y puedes nombrar al merchant-of-record en cada costura en una línea cada uno. Esa última habilidad suena a trivia y en realidad es el criterio de contratación: montones de devs pueden montar un widget, pocos te pueden decir quién se come el chargeback.

Ahora puedes mover dinero para adentro y para afuera, pero la puerta que construiste da por sentado un solo tipo de comprador. Para un comprador de EE. UU. con tarjeta de crédito, un comprador de la UE que vive sobre SEPA, y un comprador brasileño que espera PIX, el riel correcto es distinto cada vez, y elegir mal o pierde la venta o se come el margen. La próxima lección es procesadores de aceptación y corredores: tres proveedores comparados sobre números, y un registro de decisión escrito de qué riel sirve a qué comprador.
