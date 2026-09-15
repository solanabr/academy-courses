# HTTP 402, revivido: el protocolo x402

## Resumen

Acabas de comparar corredores fiat y escribiste el registro de decisión de corredores de Wavelength para sus compradores de EE. UU., la UE y Brasil. Cada cliente hasta ahora ha sido un humano con un teléfono en la mano. En esta lección, el cliente deja de ser humano.

Esta es la escena. Una API responde una petición con HTTP 402 Payment Required, un código de estado que estuvo muerto veinticinco años. Esta vez el llamador no es un humano haciendo clic en un paywall sino un bot: lee el 402, paga y reintenta exactamente la misma petición, todo antes de la próxima línea de tu log. Nadie hizo clic en nada. El protocolo que hace que ese viaje de ida y vuelta funcione se llama x402, y para el final de esta lección puedes leer su tráfico v2 como un nativo.

Lo que te llevas hoy, por adelantado:

- La superficie de x402 v2 a nivel de spec: los tres headers que llevan todo el intercambio, PAYMENT-REQUIRED, PAYMENT-SIGNATURE y PAYMENT-RESPONSE, más los ids de red CAIP-2, cuatro esquemas, tres transportes.
- El flujo exact-SVM de punta a punta: quién construye la transacción, quién firma parcialmente, y por qué un facilitador agrega la última firma y la envía.
- El panorama de facilitadores de Solana, incluida una corrección a una suposición que vas a oír repetida: Helius no es un facilitador.
- Disciplina de fuentes para los números de tráfico de x402, porque dos cifras publicadas piden a gritos que las mezclen en una sola estadística equivocada.

Lección de conceptos, así que el lab te entrega un scaffold y te deja a ti cada decisión de criterio; la próxima lección se quitan las rueditas cuando construyas el agente que paga. Antes de cualquier teoría, levanta un endpoint 402 falso para hurgar. Node está en tu máquina desde el módulo 2, donde los clientes `@solana-program/*` pusieron el piso en Node 24; el mock de hoy no importa nada más que el `node:http` incorporado de Node, y curl viene con macOS y Linux:

```bash
mkdir -p ~/wavelength/x402-lab && cd ~/wavelength/x402-lab
node --version
```

Deja esa terminal abierta. En unos cuatro minutos va a estar hablando el mismo código de estado que tu API de precios de prensado va a hablar por dinero.

## El cuatrocientos dos, desmitificado

### Por qué volvió un código de estado muerto

HTTP ha llevado un espacio para pagos desde los noventa. El código de estado 402, Payment Required, quedó reservado en las primeras specs de HTTP y después nunca recibió un comportamiento definido: un terreno con zonificación comercial en el que nadie construyó por veinticinco años. Cada intento de monetizar un endpoint HTTP le dio la vuelta en su lugar. Los paywalls te redirigen a una página de checkout. Las API keys mueven el pago a un portal de facturación y a una tarjeta de crédito en archivo. Los dos patrones comparten una suposición: en algún punto del flujo, un humano con un navegador va a aparecer a teclear cosas.

El comercio agéntico rompe esa suposición. Cuando el llamador es un programa, una página de checkout es un callejón sin salida; no hay nadie que le haga clic. Lo que un llamador máquina necesita es un desafío de pago dentro del viaje de ida y vuelta de HTTP mismo: una respuesta legible por máquina a "esto cuesta dinero" que lleve todo lo necesario para pagar, para que el llamador pueda liquidar y reintentar sin salirse nunca del protocolo. Ese es precisamente el hueco para el que 402 estaba zonificado, y x402 es el protocolo que finalmente construyó en el terreno. El colapso honesto en una línea: x402 es un header de paywall con un comprobante. El servidor dice "pago requerido, estos son los términos" en un header, el cliente reenvía la petición con prueba de pago en un segundo, y el servidor responde con la mercancía más un comprobante de liquidación en un tercero. Tres headers, un viaje de ida y vuelta, y el lado del pago del intercambio no toca nunca el cuerpo de una respuesta. Todo lo demás en esta lección es el detalle detrás de esos tres momentos.

La gobernanza detrás de la spec vale treinta segundos, porque te dice que esto es infraestructura, no el SDK de una startup. x402 se originó dentro de Coinbase, incubado por su equipo de Development Platform, y desde entonces se mudó a una x402 Foundation que opera bajo la Linux Foundation. La Solana Foundation se unió a ella. Esa trayectoria, del experimento de una empresa a la tutela de una casa neutral, es el camino estándar para los protocolos que pretenden sobrevivir a sus creadores.

![Línea de tiempo que rastrea HTTP 402 desde su reserva sin usar de la década de 1990, pasando por la incubación de x402 en Coinbase, hasta una casa en la Linux Foundation y la superficie v2 estable de hoy.](assets/v01-timeline.png)

Una advertencia de fechas antes de la mecánica, dado que vas a conocer números de versión de inmediato: la spec v2 es lo que enseñamos aquí porque su superficie es estable, pero su fecha de entrega a mainnet no está publicada al momento de escribir esto (2026-08-22), y v1 sigue viva en el mundo real. Estás aprendiendo la spec actual mientras el mundo desplegado se para a horcajadas sobre dos versiones. Guarda esa idea; se vuelve una trampa de interoperabilidad de verdad más abajo.

### La superficie v2: headers, redes, esquemas, transportes

Empieza por los nombres que van en el cable, porque v2 movió la conversación entera hacia los headers y confundirlos produce fallas silenciosas. En v1, la prueba de pago del cliente viajaba en un header llamado X-PAYMENT, y el comprobante de liquidación del servidor volvía en X-PAYMENT-RESPONSE. La convención del prefijo X- se ha desalentado formalmente en HTTP por más de una década, y v2 retiró los dos nombres: el header de petición ahora es PAYMENT-SIGNATURE, y el header de respuesta es PAYMENT-RESPONSE. Los mismos trabajos, nombres nuevos.

El tercer header no es un renombre, es una reubicación, y es el que agarra a la gente. En v1 el desafío mismo, los términos del pago, llegaba como el cuerpo JSON de la respuesta 402. En v2 no. El servidor serializa el desafío a JSON, lo codifica en base64 y lo pone como un header de respuesta llamado PAYMENT-REQUIRED; el cuerpo de un 402 de v2 son dos bytes, `{}`, bajo un `Content-Length: 2`. El documento de transporte HTTP de v2 es directo sobre por qué: los cuerpos de las respuestas son un asunto de implementación del servidor, y toda la información del protocolo x402 se comunica a través de los tres headers. Así que toma esto como una regla y aplícala a cada respuesta de x402 que inspecciones en tu vida: **lee el header, nunca el cuerpo.** Eso vale en el desafío inicial, en un pago que el facilitador rechazó y en una liquidación que falló; el cuerpo está vacío en los tres casos, y todo lo que quieres saber, incluido por qué falló la petición, está sentado en un header.

La trampa ahora se escribe sola dos veces. Un cliente que manda X-PAYMENT a un servidor v2 está hablando el dialecto del año pasado, y el servidor ve una petición sin ninguna prueba de pago: responde 402 otra vez, tu agente paga otra vez, y te pasas una tarde aprendiendo lo que este párrafo te acaba de decir. Y un cliente que parsea el cuerpo del 402 buscando los términos encuentra un objeto vacío, concluye que el servidor está roto, y se pasa esa misma tarde depurando un servidor que se está portando perfectamente.

Después, cómo un pago nombra su cadena. x402 es deliberadamente multi-chain, así que el campo `network` de sus términos de pago usa ids de red CAIP-2, un estándar de nombres agnóstico de la cadena en el que cada red recibe un id de la forma `namespace:reference`. Las redes de Solana viven bajo el namespace `solana:` con una referencia derivada del hash de génesis del cluster. Lo práctico para un ingeniero de pagos: nunca supongas que un 402 está pidiendo pago en la blockchain que esperas. Lee el campo `network`, cotéjalo contra el id CAIP-2 en el que piensas pagar, y rechaza cualquier otra cosa. Comprobación barata, protección real.

Por encima del formato de cable se sientan dos ejes de variedad. Primero, cuatro esquemas de pago, que responden "qué forma toma el pago":

- **exact**: paga un monto preciso por llamada, liquidado por llamada. El esquema de API medida, y el foco de esta lección.
- **upto**: autoriza hasta un techo, liquida lo que de verdad se usó.
- **auth-capture**: el patrón de los rieles de tarjeta que conoces del módulo 1, autorización ahora, captura después, como esquema de primera clase.
- **batch-settlement**: acumula muchas obligaciones pequeñas y liquídalas juntas.

Segundo, tres transportes, que responden "qué protocolo lleva el desafío": **http** pelado, que es el que has estado imaginando todo este tiempo; **mcp**, el Model Context Protocol que usan los frameworks de agentes para llamar herramientas; y **a2a**, mensajería agente-a-agente. La cuadrícula de esquemas-por-transportes es la razón de que la spec se lea más grande de lo que se siente. Tu asiento de comercio le importa una sola celda hoy: el esquema exact sobre http, apuntando a la SVM. La spec llama a esa combinación exact-SVM.

![Tarjeta de referencia que le da a cada uno de los tres headers de v2 su dirección, su trabajo y su origen en v1, bajo una tira de regla que dice el header nunca el cuerpo, al lado del formato de red CAIP-2 y de una cuadrícula de cuatro esquemas por tres transportes con exact sobre HTTP resaltado.](assets/v02-comparison.png)

### Una petición, de punta a punta: el flujo exact-SVM

Ahora sigue una llamada hasta el final, porque el flujo es donde x402 deja de ser una spec y empieza a ser un sistema de pagos. Imagina el setup de la próxima lección un peldaño antes: la API de precios de prensado de Wavelength cotiza costos de prensado de vinilo, y el bot de compras de un distribuidor quiere una cotización.

**Momento uno, el desafío.** El bot llama a `GET /price`. El servidor responde 402 con un cuerpo vacío y un header PAYMENT-REQUIRED; decodifica ese header de base64 y tienes el desafío, legible por máquina. Tres campos se sientan en su nivel superior. `x402Version` es el entero `2`, y es cómo sabes qué dialecto estás leyendo antes de tocar cualquier otra cosa. `error` es un string que dice por qué pasó este 402, que en el desafío inicial es solo un replanteo de que se requiere pago y en un pago rechazado es la razón real de la falla. `resource` es un objeto que nombra lo que el llamador estaba tratando de comprar, su `url`, su `description` y su `mimeType`.

Debajo de esos se sienta `accepts`, un array de uno o más objetos PaymentRequirements, los términos del pago mismos, y es aquí donde el resto de la lección no deja de volver: `scheme` ("exact"), `network` (un id CAIP-2), `amount` (un string de monto en unidades base; aprendiste en el módulo 2 por qué el dinero viaja como unidades base enteras, y fíjate en el nombre, porque v1 llamaba a este mismo campo `maxAmountRequired` y te vas a topar con los dos), `asset` (el mint del token que liquida el pago, USDC para nosotros), `payTo` (la dirección de dueño del comercio, no una cuenta de token), `maxTimeoutSeconds` (cuánto tiempo va a mantener el servidor estos términos abiertos para que el pago se complete), y un objeto `extra` con dos miembros que hacen funcionar el sabor SVM. `extra.feePayer` nombra la cuenta que va a pagar la comisión de la transacción, y no es el bot. `extra.memo`, con un tope de 256 bytes, lleva el id de factura que el comercio va a usar para la conciliación; el nuestro diría algo como `WVL-PRESS-0042`, y cuando la transacción liquidada aterriza on-chain, ese memo es cómo tu back office empareja el pago con el pedido. Construiste exactamente este patrón de conciliación con el verificador en el módulo 4; x402 solo estandariza dónde viaja el id.

**Momento dos, el pago.** El bot lee los términos y construye una transacción versionada de Solana que transfiere `amount` de `asset` al dueño de `payTo`, con el memo adjunto, y con el slot de fee payer puesto en la cuenta nombrada en `extra.feePayer`. Después hace algo que merece su propia definición, porque es la bisagra de todo el diseño. Una transacción de Solana lista cada cuenta que tiene que firmarla, y es inerte hasta que todas lo han hecho. **Firmar parcialmente** quiere decir firmar tus propios slots requeridos y dejar el de alguien más vacío: el bot firma como dueño del token autorizando la transferencia, pero no puede firmar como fee payer, porque el fee payer es la llave de alguien más. Lo que el bot sostiene ahora es una transacción completa en cada detalle y válida en ninguno, como un contrato con una línea de firma todavía en blanco. Codifica en base64 esa transacción parcialmente firmada y reintenta la petición original con ella en el header PAYMENT-SIGNATURE.

**Momento tres, la liquidación.** El servidor no toca la blockchain él mismo. Reenvía el payload a un **facilitador**, un servicio que expone dos endpoints. `/verify` inspecciona la transacción parcialmente firmada y responde una pregunta: si esto se completara y se enviara, ¿satisfaría los términos del pago? Monto correcto, asset correcto, red correcta, destinatario correcto, memo intacto. Contra el SDK fijado (`@x402/svm` 2.23.0), responde ensayando, no solo leyendo: el verificador llena el slot de fee payer en su propia copia y *simula* la transacción completada contra el estado actual de la blockchain. Firmada, sí — enviada, nunca; una simulación no mueve dinero y no gasta comisión. Después `/settle` hace la parte irreversible: el facilitador agrega la firma de fee payer que falta, la que corresponde a `extra.feePayer` del momento uno, y envía a la red la transacción ahora completamente firmada. El bot pagó el precio; el facilitador pagó la comisión. Eso es patrocinio de comisiones, la misma jugada económica que vas a volver a conocer en el checkout sin gas del módulo 8, empaquetada aquí como infraestructura de protocolo.

**Momento cuatro, el comprobante.** Liquidación confirmada, el servidor finalmente hace lo que el bot le pidió en primer lugar: responde 200 con la cotización, más un header PAYMENT-RESPONSE que lleva los detalles de la liquidación. El bot recibió sus datos, el comercio recibió su pago, y el intercambio entero cupo dentro de una sola petición HTTP reintentada. Sin creación de cuenta, sin emisión de API key, sin tarjeta en archivo. Desde el punto de vista de tu log, un 402 seguido milisegundos después por un 200.

![Diagrama de secuencia que sigue una llamada medida desde un 402 de cuerpo vacío que lleva su desafío en el header PAYMENT-REQUIRED, pasando por la firma parcial del agente, hasta los pasos de verify y settle del facilitador y el header de comprobante.](assets/v03-flowchart.png)

La coreografía de las firmas es la parte que la gente entiende mal en la primera lectura, así que fíjala. El cliente firma como dueño y nunca como fee payer. `/verify` nunca envía: llena el slot de fee payer en una copia de prueba puramente para simular, y una simulación no puede mover dinero. `/settle` es el único broadcast: firma de fee payer puesta, transacción afuera. Si puedes recitar esa oración, puedes depurar la mitad de los hilos confundidos sobre x402 que vayas a leer en tu vida.

![Diagrama de firmas que muestra al agente llenando el slot de dueño, a verify firmando una copia de prueba puramente para simular, y al facilitador llenando el slot de fee payer para el broadcast solo en settle, mientras el comercio no firma nada.](assets/v04-diagram.png)

Un número que no vas a encontrar aquí, a propósito. La spec exige que la transacción de settle lleve instrucciones de límite de ComputeBudget, y acota el precio por unidad de cómputo, pero no declara ningún conteo de unidades de cómputo para una liquidación, ninguno. Una cifra de "alrededor de 20,000 CU por settle" circula de todos modos, y cuando la persiguimos mientras investigábamos este curso se deshizo a favor del lector: los 20,000 son reales pero no son un costo. Es `DEFAULT_COMPUTE_UNIT_LIMIT` en el SDK de referencia (`@x402/svm` 2.23.0, leído el 2026-08-22), el techo que el cliente pide cuando antepone la instrucción SetComputeUnitLimit, que es un presupuesto que pides, no una cuenta que pagas. Citarlo como consumo es como citar tu límite de crédito como tu alquiler. Así que esta lección no imprime ningún costo de CU para la transacción de settle, y tus docs de API tampoco deberían. Si un número te importa, mídelo en tus propias transacciones liquidadas y fecha la medición. Cualquier absoluto que imprimas necesita una fuente independiente y fechada, o no debería imprimirse. Esa regla está a punto de cargar mucho más peso en la sección de tráfico.

### El facilitador en el que tienes que confiar

Hora de ser adultos sobre la contrapartida, porque la versión desmitificada de x402 no puede ser "y después la magia lo liquida". El facilitador es un tercero parado dentro de tu camino de pago, y deberías ver la superficie de confianza sin foco suave. Ve cada transacción antes del envío. Sostiene la llave del fee payer, lo que quiere decir que decide qué se envía y qué no: un facilitador puede negarse a liquidar, que es censura cuando estás en el extremo equivocado, y los alojados corren screening KYT, comprobaciones de compliance a nivel de transacción, por diseño. Esto no es una falla que alguien olvidó arreglar. El patrocinio de comisiones requiere un fee payer, la verificación requiere un inspector, y poner los dos en un solo servicio es lo que hace que el lado del bot en el flujo sea un solo header. Estás comprando conveniencia con confianza, el canje más viejo de los pagos. Tu registro de decisión de corredores de la lección pasada hizo explícito el mismo canje para las rampas fiat; la columna del facilitador pertenece a la misma tabla.

El encaje del esquema es el segundo límite honesto. exact es liquidación por llamada en la blockchain, que es exactamente lo correcto para una API medida donde cada llamada vale dinero real, y exactamente lo equivocado para streaming de alta frecuencia, miles de eventos de menos de un centavo por minuto, donde el overhead de la liquidación por llamada en la blockchain domina al pago mismo. Cuando el encaje por llamada se rompe, para eso existen los esquemas upto y batch-settlement. Ajusta el esquema a la medición, no al revés.

¿Así que quiénes son las opciones reales de facilitador en Solana? Este es el panorama, y contiene una corrección que vale nombrar en voz alta. Las notas tempranas de planeación de este curso suponían que Helius estaría en esta lista. No está: Helius no aparece en ningún roster de facilitadores, y sus trabajos en este curso siguen siendo los que han sido, webhooks, RPC y los rieles de suscripción del módulo 5. La lista que sí existe:

- **Corbits**, **PayAI** y **Solvador**: facilitadores de Solana, cada uno corriendo el servicio de /verify-y-/settle.
- **Dexter**: el mismo asiento, y gratis, lo que lo vuelve la primera parada obvia para los experimentos de un comercio pequeño.
- **Coinbase CDP**: la opción alojada del actor establecido, con screening KYT y OFAC incorporado. Si tu postura de compliance requiere un camino de liquidación con screening, este es el diseñado para ese requisito; si la resistencia a la censura es tu prioridad, la misma feature se lee como un bug.
- **Faremeter**: no un facilitador alojado sino un framework de código abierto, y notable por auto-negociar v1 y v2, más MPP (el Machine Payments Protocol, la familia de pagos por auth de HTTP cuya spec de método de Solana escribe la Foundation, y a la que le vas a poner un gate junto con x402 en dos lecciones). ¿Te acuerdas del mundo de dos dialectos de más arriba? Faremeter es el adaptador para vivir en él.
- **El facilitador de x402.org**: solo devnet y testnet. Perfecto para el lab que estás a punto de correr, y una trampa si cableas producción contra él.

![Tabla de roster de las opciones de facilitador de x402 en Solana con sus advertencias de confianza y de red, más una fila corregida que anota que Helius no es un facilitador.](assets/v05-comparison.png)

¿Cómo eliges? De la misma manera en que elegiste corredores la lección pasada: nombra la restricción que domina. Liquidación con screening de compliance requerida, CDP. Presupuesto cero y mainnet, el asiento gratis, Dexter, después de tu propia diligencia sobre él. Contrapartes mezcladas de v1 y v2, Faremeter delante de cualquier facilitador que liquide. Banco de pruebas, el de x402.org, y nada más. No hay un ganador absoluto, que es la señal más sana posible para un panorama tan joven.

### Leer el tráfico sin inventar números

x402 dejó de ser una curiosidad en 2026, y los números son genuinamente grandes. También están publicados en dos ventanas distintas en dos páginas distintas, y el pecado analítico más común en los comentarios sobre pagos agénticos es mezclarlas. Vas a aprender las dos cifras con sus fuentes engrapadas, porque un ingeniero de pagos que cita mal números de volumen quema una credibilidad que es lenta de reconstruir.

Cifra uno, del dashboard de x402.org, una ventana móvil de 30 días, obtenida el 2026-08-21: 75.41 millones de transacciones, 24.24 millones de dólares en volumen, 94.06 mil compradores, 22 mil vendedores. Quédate un segundo con la forma de eso, porque la forma es la historia. Divide el volumen entre las transacciones y el pago promedio anda por los 32 centavos. Eso no es gente comprando discos; eso son máquinas comprando llamadas de API, exactamente el tráfico de micropagos por llamada para el que se diseñó el esquema exact. Aproximadamente noventa y cuatro mil compradores contra veintidós mil vendedores te dice que el lado comprador supera a los vendedores cuatro a uno, y honestamente, eso es todo lo que te dice: un conteo de compradores no dice nada sobre concentración, un puñado de bots pesados podría estar impulsando la mayoría de esas 75 millones de llamadas, y el dashboard no publica ese corte. Fíjate en lo que el número no puede sostener antes de citarlo.

Cifra dos, de la página de x402 de solana.com: 37 millones o más de transacciones en Solana, y una afirmación de que Solana lleva el 70 por ciento del volumen mensual de x402. Página distinta, ventana distinta, métrica distinta: eso se lee como una captura acumulada en Solana con una afirmación de participación de mercado adjunta, no como un total móvil de 30 días.

Ahora la disciplina, dicha como una regla que puedes hacer cumplir en una revisión de docs. Cita cada cifra con su propia fuente y su propia fecha, y nunca las combines. No las sumes; 75.41M más 37M es igual a un número que ninguna fuente en la tierra sostiene. No divides una entre la otra para derivar una participación; las ventanas no coinciden. Y féchalo todo, porque los dos son números vivos de dashboard que derivan a diario; las cifras de arriba eran ciertas el 2026-08-21 y ya están obsoletas mientras las lees. Si dos números no se midieron en la misma ventana por la misma fuente, no pertenecen a la misma aritmética. Esa oración es la regla entera.

![Dos tarjetas de fuente que mantienen apartados los totales de 30 días de x402.org y las cifras acumuladas de Solana de solana.com, con un panel que prohíbe cualquier aritmética entre ellas.](assets/v06-comparison.png)

Quién está parado detrás de este tráfico importa tanto como su tamaño. El roster de socios de x402.org incluye a AWS, Cloudflare, Stripe y Vercel, que es el establishment de la infraestructura, no una porra nativa de cripto. Las historias de migración ya empezaron: atxp.ai movió su stack a x402 más MPP en Solana. Y la competencia llegó en la forma más halagadora, con OKX entregando un protocolo rival de pagos entre máquinas al que llama APP. Los estándares que nadie usa no consiguen competidores.

Stripe merece su propio momento, porque su posición es la señal más clara que hay de hacia dónde creen los actores establecidos que va esto. Cuenta sus frentes. Está en el roster de trusted-by de x402.org. Co-escribió ACP, la spec de checkout agéntico, con OpenAI. Como viste en el trabajo de corredores del módulo 6, opera un adquirente de USDC-en-Solana que liquida a los comercios en fiat. Y con Tempo Labs co-escribió `draft-httpauth-payment-00`, el esquema de autenticación HTTP "Payment" sobre el que está construido MPP, que conoces en dos lecciones. Un actor establecido, cuatro asientos en cuatro mesas distintas del comercio nativo de máquinas. Stripe no le está apostando a un ganador; está comprando la cartelera entera de la carrera. Para Wavelength la lectura es más simple y más útil: los rieles que estás aprendiendo este módulo son los mismos rieles alrededor de los cuales se está posicionando el actor establecido de pagos más grande del planeta, y tu API de precios de prensado va a hablar la versión de protocolo abierto de ellos la próxima lección.

![Diagrama de centro que ubica a x402 entre sus socios de la Linux Foundation, con un llamado para los cuatro frentes de Stripe, siendo el cuarto el esquema base de auth de Payment de HTTP detrás de MPP, y flechas de borde para los desafiantes.](assets/v07-diagram.png)

## Lab: anota un 402 como si la spec te estuviera mirando

La barrera de esta lección es la anotación, no la construcción: toma una respuesta 402 con forma de v2 y etiqueta cada header y cada campo de PaymentRequirements con su rol, y después declara quién firma en /verify frente a /settle. Vas a generar la respuesta tú mismo desde un servidor mock, así que también sientes el viaje de ida y vuelta desde la silla del servidor. El scaffold está dado; cada anotación es tuya.

1. En el directorio `~/wavelength/x402-lab` del principio de la lección, crea `x402-mock.mjs`. Este es un stub de enseñanza de la capa de pago de la API de precios de prensado: habla el sobre de desafío completo de v2, los tres campos de nivel superior más un PaymentRequirements de siete campos, sobre los tres headers de v2. No verifica nada; un facilitador real hace la comprobación en producción, y la spec sigue siendo la fuente de verdad para el formato de cable.

```js
// x402-mock.mjs - a v2-shaped 402 teaching stub. Zero dependencies.
import { createServer } from "node:http";

// The challenge. In v2 this never travels in the body: it is JSON, base64'd,
// and set as the PAYMENT-REQUIRED response header.
const paymentRequired = {
  x402Version: 2, // which dialect this challenge speaks
  error: "Payment required", // WHY the 402 happened; the only place a reason appears in the challenge
  resource: {
    // what the caller was trying to buy
    url: "http://localhost:4021/price",
    description: "Wavelength pressing-price quote",
    mimeType: "application/json",
  },
  accepts: [
    {
      scheme: "exact",
      network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", // CAIP-2: solana namespace + mainnet genesis-hash reference
      amount: "10000", // base units: 0.01 USDC at 6 decimals
      asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
      payTo: "MerchantownerPubkeyGoesRightHere11111111111", // the merchant OWNER; the scheme derives the ATA
      maxTimeoutSeconds: 300, // how long these terms stay payable
      extra: {
        feePayer: "FaciLitatorFeePayerPubkeyGoesRightHere11111", // stand-in: the sponsor who signs LAST
        memo: "WVL-PRESS-0042", // invoice id for reconciliation; 256-byte ceiling
      },
    },
  ],
};

const b64 = (value) => Buffer.from(JSON.stringify(value)).toString("base64");

createServer((req, res) => {
  const proof = req.headers["payment-signature"]; // Node lowercases incoming header names
  if (!proof) {
    res.writeHead(402, {
      "Content-Type": "application/json",
      "Content-Length": "2",
      "PAYMENT-REQUIRED": b64(paymentRequired),
    });
    res.end("{}"); // the body is empty on purpose; everything is in the header
    return;
  }
  // A real server forwards `proof` to a facilitator: /verify inspects, /settle signs + submits.
  // This stub accepts anything, so the header choreography is visible end to end.
  const receipt = b64({
    success: true,
    transaction: "5xSettLedSignatureStandin",
    network: paymentRequired.accepts[0].network,
    payer: "AgentPubkeyStandin111111111111111111111111",
  });
  res.writeHead(200, { "Content-Type": "application/json", "PAYMENT-RESPONSE": receipt });
  res.end(JSON.stringify({ quote: { sku: "12in-180g-black", unitPriceUsd: 7.4 } }));
}).listen(4021, () => console.log("mock pressing-price API on :4021"));
```

2. Córrelo, y después juega el primer momento del bot desde una segunda terminal:

```bash
node x402-mock.mjs
```

```bash
curl -i http://localhost:4021/price
```

```text
HTTP/1.1 402 Payment Required
Content-Type: application/json
Content-Length: 2
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJQYXltZW50IHJlcXVpcmVkIiwicmVzb3VyY2Ui...
Date: Fri, 21 Aug 2026 16:41:09 GMT
Connection: keep-alive
Keep-Alive: timeout=5

{}
```

Quédate con esa pantalla: un 402, dos bytes de cuerpo, y un header que lleva varios cientos de caracteres de base64, elidido arriba en los puntos suspensivos. Decodifícalo y los términos aparecen:

```bash
curl -sD - -o /dev/null http://localhost:4021/price \
  | grep -i '^payment-required:' | sed 's/^[^:]*: *//' | tr -d '\r' \
  | base64 -d | node -p "JSON.stringify(JSON.parse(require('fs').readFileSync(0,'utf8')),null,2)"
```

```json
{
  "x402Version": 2,
  "error": "Payment required",
  "resource": {
    "url": "http://localhost:4021/price",
    "description": "Wavelength pressing-price quote",
    "mimeType": "application/json"
  },
  "accepts": [
    {
      "scheme": "exact",
      "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
      "amount": "10000",
      "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "payTo": "MerchantownerPubkeyGoesRightHere11111111111",
      "maxTimeoutSeconds": 300,
      "extra": {
        "feePayer": "FaciLitatorFeePayerPubkeyGoesRightHere11111",
        "memo": "WVL-PRESS-0042"
      }
    }
  ]
}
```

Este es el momento exacto en que un agente que paga empieza a leer, y fíjate dónde lee.

3. Juega el momento del reintento. El valor del header aquí es un blob de relleno, no una transacción parcialmente firmada de verdad; el /verify de un facilitador lo rebotaría al instante, que es algo lindo para demostrarte a ti mismo más adelante en el facilitador solo-devnet de x402.org:

```bash
curl -i -H "PAYMENT-SIGNATURE: c3R1Yg==" http://localhost:4021/price
```

Deberías ver `HTTP/1.1 200 OK`, el cuerpo de la cotización y un header `PAYMENT-RESPONSE` que lleva un comprobante en base64.

4. Ahora la barrera de verdad. Crea `annotations.md` y etiqueta, en tus propias palabras, una línea cada uno, trabajando hacia afuera desde el cable: los tres headers PAYMENT-REQUIRED, PAYMENT-SIGNATURE y PAYMENT-RESPONSE; los tres campos de nivel superior del desafío `x402Version`, `error` y `resource`; y los siete campos de requisitos `scheme`, `network`, `amount`, `asset`, `payTo`, `maxTimeoutSeconds` y `extra` (divide ese último en `feePayer` y `memo`), cada uno con su rol en el flujo. Sin copiar frases de esta lección; el punto es que las etiquetas sobrevivan en tus palabras.

5. Cierra el archivo con la nota de dos líneas que exige la barrera, respondiendo con precisión: qué firmas existen antes de que el facilitador toque la transacción, qué hace /verify con ellas, y cuál es la única firma que /settle agrega antes de enviar.

6. Auto-comprobación contra la sección del flujo. La condición de aprobación: un compañero de equipo que nunca haya visto x402 podría leer tu `annotations.md` al lado de la salida de curl y predecir correctamente qué haría un facilitador con un pago real.

## Challenge

Wavelength va a necesitar una decisión de facilitador antes de la construcción de la próxima lección, así que redáctala ahora, cinco líneas, en el mismo formato que el registro de decisión de corredores: una línea que nombre la restricción dominante para un comercio pequeño que mide una API de precios de prensado en mainnet, una línea para tu elección principal con la razón, una línea para la alternativa de compliance y cuándo te cambiarías a ella, una línea para lo que usas en CI y por qué nunca puede ser el ajuste de producción, y una línea que declare la confianza que estás aceptando, en tus propias palabras, a partir de la sección de la superficie de confianza. No hay una única respuesta correcta; hay una defendible, y la próxima lección vas a construir contra la que hayas elegido.

Objetivo extra, si el mundo de dos dialectos te molestó tanto como debería: agrega cinco líneas a `x402-mock.mjs` que detecten un header `X-PAYMENT` entrante y respondan el 402 de siempre, cuerpo vacío y todo, con el campo `error` del desafío reescrito para nombrar el header de v2 que el cliente debería haber mandado. Manda ese cable trampa por el canal que un cliente v2 ya está leyendo y habrás construido el traspaso de v1 a v2 más amable del ecosistema.

## Checkpoint: lo que ahora puedes hacer

Si la anotación te peleó en algún punto, el enganche suele estar en uno de dos lugares. Confundir qué parte firma en /settle quiere decir volver a leer el momento tres; el comercio nunca firma, y /verify nunca envía (su firma de fee payer vive en una copia de prueba, puramente para simular), así que hay exactamente un lugar por donde el dinero puede salir: /settle. Y si tu etiqueta de `extra.feePayer` dice algo como "la cuenta desde la que el bot paga comisiones", ese es el cerebro de v1 hablando: el punto entero es que el bot no paga comisiones, las paga el patrocinador nombrado en ese campo.

Esto es con lo que entraste sin tener y con lo que sales sosteniendo. Puedes leer en frío una respuesta 402 de v2, y arrancas en el lugar correcto, porque sabes que el cuerpo nunca te va a decir nada y que el header PAYMENT-REQUIRED te lo dice todo, hasta por qué falló la petición. Puedes nombrar cada parte móvil del desafío que lleva ese header. Puedes rastrear un pago entre máquinas desde el desafío hasta el comprobante y decir con precisión dónde está la confianza y quién firma qué. Puedes nombrar las opciones reales de facilitador en Solana, incluida la que no está en la lista no importa con qué frecuencia la oigas suponer. Y puedes citar tráfico de x402 sin cometer el pecado del número mezclado, lo que te pone adelante de la mayoría de la gente que escribe sobre este protocolo para ganarse la vida. No está mal para una lección donde lo único que desplegaste fue un mock de cincuenta líneas.

La próxima lección tu cliente es un bot. Le pones precio a la API de precios de prensado de Wavelength y construyes el agente que la paga, llamada por llamada: el 402 que mockeaste hoy se vuelve un desafío real, el header de relleno se vuelve una transacción parcialmente firmada real, y la columna de facilitador de tu registro de decisión se hace efectiva.
