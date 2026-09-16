# Aceptación y corredores: Stripe, MoonPay Commerce y PIX

## Resumen

La lección pasada incrustaste un onramp headless de Coinbase en la tienda y recorriste el offramp alojado para el payout de un artista. Ahora el dinero entra y sale de Wavelength, y puedes decir quién es merchant-of-record en cada costura. Así que la pregunta de plomería está resuelta. La pregunta de hoy es una pregunta de estructura de mercado, y es la que decide si la tienda realmente vende discos: un comprador de EE. UU. con una tarjeta de crédito, un comprador de la UE que vive sobre SEPA y un comprador brasileño que hace años no toca más que PIX son tres problemas distintos detrás de un solo botón de checkout. Elige el procesador equivocado para un corredor y o pierdes la venta de plano o te comes en silencio el margen de cada unidad. Esa es toda la lección: qué riel para qué comprador, y en qué liquidas de verdad.

Piénsalo como lo piensa un sello pequeño con la distribución. Nadie en su sano juicio firma un solo distribuidor exclusivo mundial para un prensado de vinilo. Firmas un distribuidor de EE. UU. que conoce las tiendas de EE. UU., un distribuidor de la UE que conoce las tiendas de la UE y un distribuidor brasileño que conoce Brasil, cada uno con sus propios términos, cada uno llevándose su propia tajada, cada uno pagándote según su propio calendario. Los procesadores de aceptación son distribuidores territoriales de dinero. Esta lección compara tres de ellos con números, y después te hace firmar los acuerdos por escrito.

Antes de nada de eso, haz una cosa ahora mismo. El primer dato del día es comprobable desde tu terminal, así que compruébalo. `curl` ya viene preinstalado en macOS y en casi toda distro de Linux (`brew install curl` si la tuya es la excepción):

```bash
curl -sIL https://hel.io | grep -i '^location'
```

Deberías ver la cadena de redirección salir del dominio viejo de Helio y aterrizar en una propiedad de MoonPay. Si aterriza en otro lado, alguna historia de adquisición se volvió a mover desde que esto se escribió, y acabas de aprender la regla más profunda de esta lección un paso antes de tiempo: en pagos, todo dato de proveedor lleva una fecha encima.

Los hallazgos, por delante:

- **Stripe pay-with-crypto** acepta USDC en Solana en el checkout y liquida en fiat a tu saldo de Stripe. Disponibilidad general en EE. UU., un límite de cliente de $10,000 por transacción, y reembolsos devueltos como stablecoins a la billetera de origen, que es exactamente la construcción de reembolso que armaste hace dos módulos, ahora corriendo dentro del adquirente de otro.
- **MoonPay Commerce** es el antiguo Helio: checkout, pay links, card-to-crypto, con la liquidación configurable a cripto, stablecoin o fiat convertido. Wavelength gira ese dial a stablecoin y se queda con su tesorería. Su cifra de volumen la reporta el proveedor y está en movimiento, así que la citamos con fuente y fecha, nunca como una constante.
- **Sphere** es el ancla de Brasil: rieles que incluyen PIX junto a SEPA y ACH, liquidación cotizada en menos de 30 minutos.
- **La frontera que no es técnica**: un método de cinco preguntas para ubicar la línea regulatoria de un corredor, corrido sobre Brasil como caso trabajado — dos fronteras, dos fechas de entrada en vigor, dos plazos de agenda y un flujo de comercio que se sienta sobre la línea en vez de dentro de ella.
- **El precio de exhibición** para discos con precio en USD cobrados en USDC no necesita ningún oráculo, porque USDC es un peg a USD cobrado 1:1. El caso del activo volátil queda nombrado y traspasado.
- El entregable es el **registro de decisión de corredores**: una tabla escrita de tres filas más un esqueleto de config, un riel por geografía de comprador, con el activo de liquidación y el merchant-of-record declarados por fila.

Cómo se reparte el trabajo: esta es una lección de concepto ya avanzado el curso, así que la proporción se invierte. Recorremos juntos los tres procesadores y la lógica de precios, con números. El registro de decisión y el esqueleto de config son tuyos y de nadie más, sin apoyo, porque una tabla de corredores que llenó otra persona no decide nada por ti. Es un valor por defecto, y los valores por defecto son la forma en que mueren los márgenes.

## Tres territorios, tres distribuidores

### Un procesador de aceptación no es un onramp

Primero, una definición que al ecosistema le encanta difuminar, puesta justo a tiempo. Las rampas de la lección pasada mueven el fiat de un comprador hacia cripto que él posee, o la cripto de un comercio hacia fiat que él posee. El cliente de una rampa es quien sea que quiera que se le intercambie el activo. Un **procesador de aceptación** se sienta en otro lugar por completo: su cliente es el comercio, y su trabajo es tomar lo que sea que tenga el comprador y entregar lo que sea que el comercio quiera tener, cobrando una comisión por pararse en el medio. Un **corredor** es el emparejamiento que esta lección no para de puntuar: una geografía de comprador más el riel que llega hasta ella. Y el **activo de liquidación** es lo que de verdad aterriza en tu cuenta al final, fiat o stablecoin, que es la columna más consecuente de la tabla de hoy porque decide si siquiera operas una tesorería cripto.

![Una rampa intercambia activos para quien sea su dueño, mientras un procesador de aceptación se sienta entre comprador y comercio entregando el activo de liquidación del comercio; un corredor empareja geografía con riel.](assets/v01-diagram.webp)

¿Por qué se gana la distinción su propia sección? Porque la mala lectura más común en este mercado es mirar un producto de liquidación en fiat para comercios y describirlo como "la tienda hace off-ramp de sus fondos." No lo hace. Cuando un procesador te liquida en fiat, tu tienda nunca tiene cripto por esa venta, así que no hay nada de lo que hacer off-ramp. El offramp que recorriste la lección pasada y la liquidación en fiat que vas a conocer en un momento son máquinas distintas que dan la casualidad de terminar en la misma moneda, y confundirlas te va a hacer construir infraestructura de tesorería que no necesitas. He visto a equipos hacer exactamente eso. No es un desperdicio pequeño.

### Stripe: el adquirente que liquida en fiat

Empieza por el territorio que ya sabes pensar, porque integraste Stripe en una vida anterior. Stripe pay-with-crypto acepta USDC en Solana en el checkout y liquida en fiat a tu saldo de Stripe. Está disponible de forma general en EE. UU. Quédate un rato con lo que hace esa frase: el comprador paga en stablecoin on-chain, y a la mañana siguiente tu dashboard de Stripe muestra dólares, en el mismo saldo donde aterrizan tus ventas con tarjeta, según el calendario de Stripe. Conservas todos tus libros actuales, tu contador nunca aprende qué es una ATA, y el tramo cripto se vuelve un detalle de implementación del adquirente.

Las barreras de protección te dicen lo que Stripe piensa del dinero irreversible. Hay un límite de cliente de $10,000 por transacción. Y los reembolsos se devuelven como stablecoins a la billetera de origen, lo que debería sonar fuerte: ese es el pago push inverso que construiste a mano en la lección de conciliación, la misma regla de billetera de origen, la misma geometría sin chargebacks. No llegaste a esa forma por tu cuenta, y sería halagador fingir lo contrario: la lección de conciliación copió el precedente de Stripe a propósito, porque era el único que existía. Lo nuevo aquí es ver la misma regla sostenerse en un riel donde el comercio no toca cripto en absoluto, lo que te dice que la devolución a la billetera de origen es una propiedad del dinero push y no una preferencia de tesorería que heredaste. Toma el tope de $10,000 como la otra mitad del mensaje: cuando Stripe acota un riel así de fuerte, le está poniendo precio a la irreversibilidad que ya sabes que está ahí.

![El USDC del comprador en Solana fluye por Stripe hacia el saldo fiat del comercio bajo un tope por transacción, mientras los reembolsos vuelven como stablecoins enviadas por push a la billetera de origen.](assets/v02-diagram.webp)

La economía de este acuerdo es la economía de la comodidad. La liquidación en fiat te ahorra una tesorería cripto, te ahorra la costura del offramp, te ahorra cada pregunta de conciliación sobre tener stablecoins en un balance. A cambio aceptas el tope, el calendario de liquidación del procesador y una huella geográfica que es, según los datos congelados para esta lección, disponibilidad general en EE. UU. Tu comprador de la UE no está en esa frase. Tu comprador brasileño no está ni cerca. Volver a sondear los propios docs de Stripe el 2026-08-22 agrega una arruga viva que vale la pena llevarse al registro sin cambiar la fila: los compradores pueden pagar desde donde sea, es la ubicación del *negocio* la que está restringida, y junto a la disponibilidad general en EE. UU. Stripe lista la UE, Hong Kong, México y Suiza en preview privado. El preview privado es una lista de espera, no un corredor, así que no se gana una celda en una tabla contra la que vas a entregar. Ponlo en tu lista de re-verificación, porque es exactamente el tipo de fila que se da vuelta. Y una cláusula del contrato de distribución importa aquí para la tienda de discos: sigues siendo merchant-of-record de la venta. Stripe es tu adquirente, no el vendedor de tus discos. Sostén eso junto al mapa de costuras de la lección pasada sin pestañear, porque las dos lecciones no se están contradiciendo: allá, el *onramp* de Stripe puso a Stripe en el asiento de merchant-of-record, porque lo que se vendía era la cripto misma; aquí, lo que se vende es tu disco, y la *aceptación pay-with-crypto* de Stripe es un producto distinto en un asiento distinto, adquirente de tu venta. Mismo logo, dos asientos. Nombrar el producto antes de nombrar el asiento es exactamente la disciplina que el mapa de costuras estaba instalando. La queja del comprador por un prensado alabeado es con Wavelength, en este riel y en todos los rieles de la tabla de hoy.

Una cosa más sobre este distribuidor en particular, porque explica el mercado en el que estás operando. Stripe sostiene hoy una posición en cuatro frentes: aparece como logo de trusted-by de x402 (el estándar HTTP-nativo para pagos entre máquinas que conoces el módulo que viene), coescribió ACP, el Agentic Commerce Protocol para checkout de agentes de IA, con OpenAI, coescribió el esquema de autenticación HTTP "Payment" sobre el que está construido MPP (el otro riel de pagos entre máquinas del módulo que viene), y corre este riel de adquirencia USDC-en-Solana que liquida fiat. Una empresa así de paciente te está diciendo hacia dónde cree que van los pagos. Guárdate esa idea hasta el final de esta lección; el frente x402 es justo lo próximo que construyes.

![Stripe sostiene cuatro posiciones a la vez, respaldo de x402, coautor de ACP con OpenAI, coautor del esquema de auth HTTP Payment detrás de MPP, y adquirente USDC-en-Solana que liquida a fiat.](assets/v03-diagram.webp)

### MoonPay Commerce: el checkout con un dial de liquidación

Ahora el acuerdo opuesto. MoonPay Commerce es el antiguo Helio, y verificaste la adquisición tú mismo en los primeros cinco minutos: hel.io ahora redirige al brazo de comercio de MoonPay. La forma del producto es un checkout cripto con pay links y un camino card-to-crypto, y aquí el activo de liquidación es un dial y no un dato dado: su propia página de producto ofrece liquidación en cripto, en stablecoins, o autoconvertida a fiat (USD, EUR, GBP) en regiones soportadas, comprobado 2026-08-22. Ese dial es toda la razón por la que se gana una columna. Gíralo a fiat y compraste un segundo Stripe con otra geografía; gíralo a stablecoin, que es lo que hace Wavelength, y la venta aterriza como tokens que tienes tú, alimentando la maquinaria exacta de tesorería y conciliación que construiste en el módulo 4. Tu conciliador, tu libro mayor, tu constructor de reembolsos, todos conservan su trabajo. Escribe la posición del dial en tu registro, no el nombre del proveedor, porque "liquidamos en USDC" es la decisión y "MoonPay Commerce" es solo donde lo configuraste.

Aquí es donde la disciplina numérica que esta lección no para de predicar consigue su caso de prueba. La cifra pegada a este producto, tal como la reporta Solana en su resumen de ecosistema de abril de 2026, recuperada 2026-08-22, es que MoonPay Commerce reportó más de $40M en volumen de "single-payment", la etiqueta del propio proveedor para pagos de checkout de una sola vez como distintos de los recurrentes, desde su lanzamiento de octubre de 2025, con 88% de eso en Solana. Lleva la etiqueta entre comillas en tu registro, precisamente porque la definió el proveedor. Fíjate en todo lo que le acabo de hacer a ese número. Tiene una fuente, y la fuente es un reporte del reporte del propio MoonPay, así que la cadena tiene dos eslabones y deberías decirlo. Tiene una fecha de recuperación. Es una cifra elegida por el proveedor, o sea una halagadora, porque los proveedores eligen números halagadores. Y es una cifra de flujo de un producto joven, lo que quiere decir que va a estar obsoleta para cuando leas esto, posiblemente para cuando yo termine el párrafo. La participación de 88% sí es señal genuinamente útil sobre dónde vive la demanda de checkout cripto. Los $40M son una instantánea de un objeto en movimiento. Tu registro de decisión cita números así con fuente y fecha o no los cita en absoluto, porque una tabla de corredores llena de cifras de proveedor sin fechar no es investigación; es un folleto con tu nombre encima.

¿Qué cuesta el acuerdo que liquida en cripto? Le entregas al procesador la UX del checkout y el esquema de comisiones, y en el tramo card-to-crypto el comprador es por un momento cliente de MoonPay para la conversión, la misma costura que mapeaste en el onramp la lección pasada, antes de que los tokens resultantes paguen tu factura. A cambio consigues un alcance de corredores que no depende de la lista de países que tenga un adquirente, liquidación en un activo que ya sabes conciliar, y pay links que puedes soltar en un DM, lo que para una tienda de discos que hace drops de preventa calza genuinamente bien. Para que quede completo: Transak y Meso también rondan este territorio, y se quedan solo nombrados en este curso porque su cobertura de Solana quedó sin verificar cuando se comprobaron los datos de esta lección. Un distribuidor sin verificar no recibe una fila en la tabla. Dejarlo fuera es lo que hace que el resto de las filas valgan la confianza.

![Tres procesadores comparados lado a lado por activo de liquidación, rieles, cobertura y límites, con Transak y Meso mostrados como excluidos porque su cobertura de Solana quedó sin verificar.](assets/v04-comparison.webp)

### Sphere y el corredor PIX

Lo que nos trae al comprador que los dos distribuidores anteriores dejan parado en la caja. El riel de pago instantáneo de Brasil es PIX, y voy a gastar aquí mi único pedazo de credibilidad de local: en Superteam Brazil veo pagos aterrizar a diario, y en São Paulo puedo pasar meses sin ver una tarjeta física, porque el vendedor ambulante, el barbero y la boletería del local aceptan PIX desde un teléfono. Un comprador brasileño que llega a un checkout que solo ofrece campos de tarjeta no piensa "inconveniente." Piensa "extranjero," y una porción significativa de ellos se va. Un corredor no es un nice-to-have para esta geografía; es la diferencia entre tener clientes brasileños y tener visitantes brasileños.

Sphere es el ancla de Brasil de la investigación. Su propio sitio se presenta mucho más ancho que un solo país, liquidación en más de 160 mercados, y la razón por la que aun así entra en esta tabla como la elección para Brasil es el riel y no el conteo de mercados: PIX es lo que exige un comprador brasileño, y PIX es el riel que Sphere carga y que ninguna otra fila carga, junto a SEPA y ACH, con una cifra de liquidación cotizada en menos de 30 minutos. Fíjate en el verbo, cotizada. Es el número del proveedor, así que viaja con la misma disciplina que la cifra de volumen de MoonPay: féchala, dale fuente, y trátala como una afirmación que vuelves a verificar antes del capstone, no como una constante que heredaste. Pero incluso descontada, menos de media hora desde el toque de un comprador PIX hasta los fondos liquidados es otro deporte que la liquidación internacional con tarjeta de varios días a la que reemplaza, y SEPA en el mismo roster calladamente hace de Sphere un candidato también para tu corredor de la UE, no solo Brasil. Una celda que la página pública no te va a llenar: Sphere cotiza el *plazo* de liquidación, no el *activo* de liquidación. Tu registro de decisión igual necesita esa celda, así que anota lo que pide Wavelength (USDC, para mantener una sola tesorería entre EU y BR) y marca la celda confirm-with-vendor; una celda a la que no le puedes poner fuente es una celda que marcas, nunca una celda que adivinas.

La contrapartida es el espejo de la fortaleza. Sphere se gana la fila BR porque carga el riel que la aceptación cripto de Stripe no toca, y vale la pena decirlo como la trampa que es, porque he visto la suposición suelta en el mundo: Stripe pay-with-crypto no ofrece PIX. La aceptación cripto de Stripe está basada en redes de stablecoins; PIX es el riel de Sphere en este roster. Mientras tanto nada del roster de Sphere cambia la fila US, donde el comprador quiere un checkout de grado adquirente y cercano a la tarjeta, que era todo el territorio de Stripe. Ningún distribuidor cubre el mapa. Échale la culpa al mapa y no a los proveedores: esa asimetría es exactamente por qué el registro de decisión de corredores existe como tabla por geografía en vez de como una respuesta de una línea.

![Un diagrama de flujo de decisión que rutea a los compradores de EE. UU., la UE y Brasil hacia sus rieles, con cada resultado registrado junto al activo de liquidación, el merchant-of-record y una fecha de verificación.](assets/v05-flowchart.webp)

### La frontera que no es técnica

Una celda de la tabla que estás por firmar sigue marcada confirm-with-vendor: en qué te liquida Sphere en realidad. Antes de mandar ese correo, sé honesto sobre qué clase de pregunta tienes entre manos. Nada de tu stack la responde. Si un corredor siquiera puede liquidarte en USDC lo deciden textos publicados en un diario oficial, textos que cambian en fechas de entrada en vigor y no en ciclos de release, y la disciplina para manejarlos es la misma que acabas de aplicar a los números de proveedor, corrida con más cuidado porque el costo de equivocarse no es el margen. Así que aquí está esa disciplina como método, y después una jurisdicción corrida por ella de verdad.

Cinco preguntas, hechas en este orden, porque cada una acota la siguiente — aunque un caso vivo a menudo te hace responder la pregunta 3 primero, como hace el recorrido de Brasil de abajo:

1. **Nombra el tramo.** Nunca "aceptamos cripto" — ¿qué movimiento de valor, de quién, hacia quién, en qué activo? Una fila de corredor esconde varios tramos (comprador a procesador, procesador a ti, el reembolso de vuelta hacia afuera), y las reglas se pegan a los tramos, no a los productos.
2. **Nombra al actor, y su licencia.** Para cada tramo, ¿quién está moviendo el dinero de verdad, y como qué está autorizado, dónde? "El procesador se encarga" se vuelve una respuesta solo cuando puedes decir como qué está licenciado el procesador en esa geografía.
3. **Pregunta cómo clasifica el regulador tu activo de liquidación.** No cómo lo comercializa el proveedor — cómo lo define la ley. El mismo USDC puede ser un activo virtual en un reglamento y otra cosa en el siguiente, y cada obligación aguas abajo se ancla en esa clasificación.
4. **Pregunta la residencia de cada parte.** Los flujos transfronterizos y los domésticos se sientan bajo reglas distintas, y la dirección sorprendente es la que hay que comprobar: algunos perímetros alcanzan flujos que no tienen ninguna frontera adentro.
5. **Féchalo y agéndalo.** Los datos legales se degradan como los datos de proveedor, salvo que la degradación está programada: fechas de entrada en vigor, ventanas de presentación, períodos de transición. Cada respuesta recibe la fecha en que la comprobaste, y las fechas que van a cambiar la respuesta van en un calendario, no en una nota al pie.

Ahora Brasil, porque es mi terreno, porque es la fila BR de la tabla, y porque es el ejemplo vivo más afilado que conozco de una frontera que se movió mientras se escribía un curso. Consulté cada texto citado abajo en las fuentes primarias — el Diário Oficial da União para las resoluciones del banco central, Planalto para las leyes, el rastreador de la propia Câmara para el proyecto de ley — el 2026-09-02. Esa fecha importa más que mi lectura; vuelve a consultarlos antes de apoyarte en cualquiera de las dos.

**La pregunta 3 primero, porque todo cuelga de una sola palabra.** Lei 14.478/2022, art. 3º define un activo virtual y excluye explícitamente la moneda nacional y la extranjera de la definición. Una stablecoin referenciada al dólar es por lo tanto, en la ley brasileña, un activo virtual y no moneda extranjera, y esa clasificación decide en qué reglamento vive el resto de esta sección. También vas a ver citado el PL 4.308/2024 alrededor de este tema, así que ubícalo con precisión: cambiaría quién puede emitir y distribuir tokens referenciados a fiat — un perímetro para la emisión — y no toca la clasificación del art. 3º ni en el texto original ni en el sustitutivo aprobado en comisión. Además sigue siendo un proyecto de ley: al 2026-09-02 está en una comisión de la Câmara esperando el informe de un relator, todavía sin pasar por su cámara de origen. Un proyecto de ley es una entrada de agenda, no una regla.

**Preguntas 1, 2 y 4: la frontera viene en dos piezas, con dos fechas.** La primera pieza es la Resolução BCB 521, de 10 de noviembre de 2025, que escribió los servicios de activos virtuales dentro del mercado de cambio brasileño insertando los arts. 76-A y 76-B en la Resolução BCB 277. Esos dos artículos están en vigor desde el 2026-02-02; la maquinaria de reporte de la resolución entró en vigor después, el 2026-05-04, así que fecha el artículo que estás citando, no la resolución. El art. 76-A es la prueba de perímetro, y es más ancho que su marco de apariencia internacional. Cuatro grupos de actividad se sientan dentro del mercado de cambio cuando un proveedor de servicios de activos virtuales — los textos dicen PSAV — está en el flujo: pago o transferencia internacional con activos virtuales; transferencias atadas al uso internacional de tarjeta; transferencias hacia o desde una billetera autocustodiada, expresamente las que no tienen ningún pago internacional adentro; y la compra, venta o permuta por parte de un PSAV de activos virtuales referenciados a fiat, con "fiat" sin calificar, así que un token referenciado a BRL queda capturado igual que uno a USD.

Lee ese tercer grupo otra vez, porque es el caso más afilado de comprueba-no-supongas que te puedo entregar. Un comprador de São Paulo que paga desde una billetera de autocustodia, a través de un PSAV, a un comercio de São Paulo no cruza ninguna frontera en ningún lado — y aun así se sienta dentro del perímetro de cambio, porque 76-A III alcanza las transferencias de autocustodia cada vez que un PSAV es un tramo. Esa es la forma por defecto de un checkout nativo de Solana. Si tu instinto dijo doméstico-por-lo-tanto-afuera, acaba de fallar en el flujo exacto que entrega este curso, y solo leer el artículo lo atrapa. El instinto falla en la otra dirección también: el art. 76-B, que define el grupo internacional, cubre más que residente-paga-a-no-residente. Un cambio de titularidad entre dos no residentes cuenta, y también un movimiento del mismo dueño — un residente brasileño que manda su propia stablecoin a su propia billetera en el exterior es una transferencia internacional bajo 76-B II sin ninguna contraparte a la vista. La residencia, pregunta 4, tiene que preguntarse de cada parte, incluido tú mismo.

Dos cláusulas del 76-A hacen entonces el trabajo específico del comercio. Su §2º prohíbe que la compra o venta de activos virtuales por parte de un PSAV se pague o se reciba en moneda extranjera — fíjate en la forma operativa, una prohibición sobre un tramo fiat en moneda extranjera y no un mandato sobre lo que el tramo fiat tiene que ser, y la diferencia no es pedantería: una prohibición deja abierto lo que un mandato cerraría, y parafrasear lo uno como lo otro es cómo se tuercen los resúmenes de segunda mano. Y el §3º veda mover fondos de terceros a través de un servicio de activos virtuales dentro del alcance, con una sola excepción: un PSAV que sirve a una institución que está ella misma autorizada en el mercado de cambio y que actúa por sus clientes. Ahora camina la fila BR de Wavelength por esas cláusulas. Un proveedor que toma la stablecoin de tu comprador y te entrega valor está, en los propios términos definitorios de la resolución, comprando un activo virtual referenciado a fiat — grupo cuatro, dentro del perímetro — mientras mueve fondos en tu interés, que es justo lo que el §3º prohíbe salvo que el proveedor esté parado dentro de esa excepción. Así que el flujo intermediado de comercio que rutea la tabla de esta misma lección no está cómodamente dentro de la frontera. Está sobre ella, y de qué lado está parado tu proveedor es exactamente la pregunta 2, la pregunta de licencia que ahora sabes ponerle a Sphere en el mismo correo que la celda del activo de liquidación.

La segunda pieza es la Resolução BCB 561, de 30 de abril de 2026, en vigor 2026-10-01 en una entrada en vigor única e indivisa, apuntada a eFX — el régimen de Brasil para servicios de pago y transferencia internacionales. Su art. 50, I exige que la liquidación entre un proveedor eFX y su contraparte extranjera corra por una operación de cambio o una cuenta de no residente en reais, con el uso de activos virtuales expresamente vedado. Guarda la precisión: esto no es "eFX no puede tocar cripto" — la misma resolución crea un código de propósito para adquirir activos virtuales a través de eFX. Lo que prohíbe es que el tramo de liquidación de proveedor a contraparte extranjera esté denominado en activos virtuales. Una regla de tramo de liquidación, en otras palabras, apuntada exactamente a la columna de tu tabla que sigue marcada confirm.

**Pregunta 5, la agenda.** Dos fechas van en ella, cada una atribuida a su fuente real, porque argumentar desde el normativo equivocado es su propio modo de falla. **2026-10-30** es el plazo para que un PSAV ya establecido presente su solicitud de autorización, fijado por la Resolução BCB 520, art. 88, I — expresado ahí como 270 días desde la entrada en vigor de la resolución el 2026-02-02; un proveedor que lo pierde tiene que cerrar ordenadamente dentro de treinta días, y desde esa misma fecha una regla aparte en el art. 91 veda a las instituciones reguladas de Brasil tratar con PSAVs que no están ni autorizados ni en trámite de autorización. **2027-05-31** es el plazo para que los proveedores eFX fuera de la lista de instituciones enumeradas de la resolución pidan autorización como instituciones de pago, según la Resolução BCB 561, art. 56-B, con su propia regla de cesar-dentro-de-treinta-días detrás. Las dos fechas van al lado de la fila BR con una nota sobre qué volver a preguntar cuando pase cada una. Y cuando le mandes a Sphere el correo de confirmar-el-activo-de-liquidación, pregunta en el mismo mensaje bajo cuál de estos regímenes opera y qué ha presentado, con fecha. Un proveedor que responde con precisión te ha dicho algo. Un proveedor que responde "estamos plenamente en cumplimiento" también te ha dicho algo.

Para decirlo con todas las letras una vez más, con las mismas palabras que usó la lección uno de este módulo: esta es una lectura de ingeniería sobre dónde se sientan las costuras, no asesoría legal, y un producto real de servicios de dinero trae un abogado real. Lo que te compran las cinco preguntas es la capacidad de resumirle el caso a ese abogado en una página, con fechas, en vez de descubrir las preguntas en la reunión, a la tarifa por hora de la reunión.

### El precio de exhibición: el oráculo que no necesitas

Una pieza más de estructura de mercado antes de que firmes nada, porque parece un problema difícil y todo el punto es que, para esta tienda, no lo es. Wavelength les pone precio a los discos en USD. Los compradores pagan en USDC. ¿Cuál es el tipo de cambio?

No hay ninguno, y ese es el diseño. USDC es un peg a USD, así que el prensado de agosto de tu catálogo, con precio de $30, se cobra como exactamente 30 USDC, 1:1, sin cotización, sin spread, sin política de redondeo. **El precio de exhibición**, el precio de la etiqueta, y **el monto cobrado**, los tokens que se mueven, son el mismo número. Esto es precisamente por qué todo el curso ha corrido sobre rieles de stablecoin: el peg borra el problema de cambio en la capa del checkout. Así que cuando te descubras bosquejando una integración de price feed para una stablecoin con peg a USD cobrada 1:1, para. Estarías construyendo un oráculo para descubrir un número que el peg ya te prometió. Lo marco porque es una trampa real con víctimas reales: el cerebro que empareja patrones ve "pago cripto" y agarra "price feed" antes de comprobar si algo flota de verdad.

Donde el precio de exhibición sí se pone interesante es en el momento en que el activo cobrado flota contra la moneda de la etiqueta: ponerle precio a un disco en SOL, o aceptar un token volátil en el checkout. Ahí necesitas un precio vivo, una regla de obsolescencia, una política de spread y un oráculo que puedas defender, y esa maquinaria es una disciplina genuina con sus propios modos de falla. También es, deliberadamente, no la disciplina de este curso. El precio basado en oráculos — ventanas de obsolescencia, intervalos de confianza, política de spread — es territorio de DeFi and RWA Engineering, y tu registro de corredores va a anotar el traspaso en vez de colar una versión enseñada a medias. Un curso de pagos que te enseñara un cuarto de oráculo no te estaría haciendo ningún favor; el cuarto que te faltaría es el cuarto que pierde dinero.

![Precio de checkout con peg y flotante lado a lado, donde un disco con precio en USD cobra el mismo número de USDC mientras los activos volátiles necesitan un oráculo, reglas de obsolescencia y política de spread.](assets/v06-comparison.webp)

### La contrapartida que nadie esquiva

Aléjate y puntúa los tres acuerdos sobre un solo eje, porque cada elección de corredor es el mismo canje en proporciones distintas: cobertura contra costo y control. La aceptación que liquida en fiat te compra una tesorería simple y libros que tu contador ya entiende, y te cobra el tope, el calendario del procesador y su geografía. La aceptación que liquida en cripto te compra tu propia tesorería y un alcance agnóstico del riel, y te cobra la UX del checkout y el esquema de comisiones. El especialista de un solo corredor te compra una geografía entera, y te cobra todas las demás geografías. Corre la forma de juguete sobre el prensado de $30 vendido tres veces, una vez por corredor: la venta US liquida como fiat según el calendario de Stripe menos la comisión de Stripe; las ventas EU y BR aterrizan como 30 USDC cada una, menos la comisión propia de cada riel, en la tesorería del módulo 4. Tres ventas, tres esquemas de comisiones, dos activos de liquidación, y después de la liquidación tu dinero está sentado en dos tipos distintos de cuenta bajo tres conjuntos distintos de términos. No hay configuración del mercado de hoy en la que un solo procesador gane las tres filas por mérito. Cualquiera que te diga lo contrario está vendiendo una de las filas.

Por eso mismo el entregable honesto es una tabla y no una recomendación. Y por eso la tabla misma se degrada: las afirmaciones de cobertura, el plazo de liquidación cotizado y el volumen comercializado que lleva son todos instantáneas fechadas de proveedores en movimiento. Un registro de decisión sin fechas de verificación está mal; solo que todavía no te ha dicho cuándo.

Eso sí, no todo lo que lleva se degrada a la misma velocidad, y ordenar los datos por su reloj es lo primero que te pide el lab.

![Los datos de esta lección ordenados en dos relojes de degradación, cifras de proveedor que necesitan fuente y fecha contra datos estructurales de producto enunciados planos.](assets/v07-table.webp)

## Lab: firma los acuerdos de distribución

Hora de escribirlo. El artefacto es `corridor-decision`: un registro de decisión más un esqueleto de config. Deliberadamente no es código importable. Nada en el capstone va a hacer `import` de este archivo; el equipo del capstone, o sea tú dentro de tres módulos, lo va a leer y configurar en consecuencia. Consume `ramp-embed` en el sentido honesto de que sus filas tienen que concordar con las costuras de rampa que mapeaste la lección pasada.

**Paso 1: el scaffold.** Desde la raíz `wavelength`, la misma raíz de workspace en la que ha construido cada módulo:

```bash
mkdir -p corridor-decision
cd corridor-decision
touch DECISION.md corridors.config.ts
```

**Paso 2: vuelve a correr la sonda y féchala.** Corriste la comprobación de redirección de `hel.io` en la apertura. Pega el comando y su salida en `DECISION.md` bajo un encabezado llamado `Verified facts`, con la fecha de hoy al lado. Después agrega los otros números en movimiento que vas a citar, cada uno con fuente y fecha: la cifra de volumen de MoonPay Commerce (más de $40M single-payment desde el lanzamiento de octubre de 2025, 88% en Solana, tal como lo reporta Solana en su resumen de ecosistema de abril de 2026, recuperada 2026-08-22), y la liquidación de Sphere cotizada en menos de 30 minutos (spherepay.co: liquidación "en menos de 30 minutos en más de 160 mercados", recuperada 2026-08-22). Los datos estructurales congelados como la liquidación en fiat de Stripe, el límite de $10,000 por transacción y los reembolsos en stablecoin a la billetera de origen van en una lista aparte, porque cambian en ciclos de producto, no en ciclos de noticias. Que las dos listas se degraden a velocidades distintas es la razón por la que son dos listas.

**Paso 3: llena las tres filas.** En `DECISION.md`, escribe la tabla que pide la evaluación: US, EU, BR en el costado; riel, activo de liquidación, merchant-of-record arriba; una línea de justificación por fila. Discute con el diagrama de flujo de arriba, no desde él. Si ruteas la UE por el riel SEPA de Sphere en vez de por MoonPay Commerce, bien, defiéndelo en la línea de justificación. El registro es tuyo; el requisito es que cada celda sea una decisión que puedas defender en voz alta.

**Paso 4: codifica el esqueleto.** El archivo de config hace el registro legible para el tú del futuro sin pretender ser una librería. TypeScript ya es una devDependency del repo del curso desde el `npm install` del módulo 2, fijado ahí en `^5.6.0`; fuera del repo, `npm i -D typescript` te consigue el compilador, y una instalación pelada hoy aterriza en la línea 7.x (npm `latest` era 7.0.2 el 2026-08-22). Cualquiera de las dos sirve aquí, porque este archivo no importa nada y no usa sintaxis más nueva que 5.x. Esa deriva de versión es en sí misma la lección: como cada pin de este curso, el número en `package.json` envejece, y el tag llamado `latest` se mueve debajo de él.

```ts
// corridor-decision/corridors.config.ts
// Decision record skeleton. Read by humans configuring the capstone; imported by nothing.

export type Corridor = 'US' | 'EU' | 'BR';

export type Rail = 'stripe-pay-with-crypto' | 'moonpay-commerce' | 'sphere';

export type SettlementAsset = 'fiat-via-processor' | 'usdc';

export interface CorridorDecision {
  corridor: Corridor;
  rail: Rail;
  settlementAsset: SettlementAsset;
  /** Who the buyer's contract of sale is with. For every rail in this roster,
   *  Wavelength remains merchant-of-record for the record itself; conversion
   *  legs (e.g. card-to-crypto) briefly interpose the processor, per lesson 1. */
  merchantOfRecord: 'wavelength';
  /** Processor-imposed per-transaction ceiling in USD, or null if none stated. */
  perTxLimitUsd: number | null;
  /** One sentence you are prepared to defend to an accountant. */
  rationale: string;
  /** ISO date every moving fact in this row was last checked. Stale row, stale decision. */
  verifiedOn: string;
}

export const corridors: readonly CorridorDecision[] = [
  {
    corridor: 'US',
    rail: 'stripe-pay-with-crypto',
    settlementAsset: 'fiat-via-processor',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: 10_000,
    rationale:
      'US buyers get acquirer-grade checkout; fiat settlement keeps US books processor-side; refunds return as stablecoins to the originating wallet.',
    verifiedOn: '2026-08-22',
  },
  {
    corridor: 'EU',
    rail: 'moonpay-commerce',
    settlementAsset: 'usdc',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: null,
    rationale:
      'Stripe pay-with-crypto is US-GA; crypto settlement keeps EU sales in the module-4 treasury; Sphere SEPA is the recorded alternative.',
    verifiedOn: '2026-08-22',
  },
  {
    corridor: 'BR',
    rail: 'sphere',
    // Sphere's public page quotes settlement timing, not asset; 'usdc' here is
    // Wavelength's REQUESTED setting, flagged confirm-with-vendor before capstone.
    settlementAsset: 'usdc',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: null,
    rationale:
      'Brazilian buyers expect PIX; Sphere carries PIX alongside SEPA and ACH with settlement quoted under 30 minutes; one treasury asset across EU and BR.',
    verifiedOn: '2026-08-22',
  },
];
```

Esas tres filas son mis acuerdos, no la clave de respuestas. Las tuyas pueden diferir, y una fila que difiera con una línea de justificación más afilada le gana a coincidir con las mías cada vez.

**Paso 5: demuestra que compila.** Un esqueleto al que nadie le comprueba los tipos se pudre y se vuelve prosa:

```bash
npx tsc --noEmit corridors.config.ts
```

El silencio es el éxito. Si el compilador objeta, lee el error; la superficie de tipos es lo bastante pequeña como para que cada falla aquí sea una inconsistencia real en tu registro, que es toda la razón por la que el esqueleto está tipado en vez de ser un segundo archivo markdown.

**Paso 6: cierra el circuito con las rampas.** Agrega una sección final a `DECISION.md` titulada `Seams with ramp-embed`, y responde en dos o tres frases: ¿en qué corredores sigue importando el onramp de la lección pasada (un corredor que liquida en cripto igual necesita compradores que tengan USDC o un tramo card-to-crypto), y en qué corredor interactúa el offramp del payout de artista con tu elección de activo de liquidación? Si tu fila US liquida fiat vía el procesador, fíjate en lo que nunca haces para esas ventas: off-ramp. Escribir esa frase es la inoculación más barata contra la mala lectura de la liquidación en fiat con la que abrió esta lección.

![El registro de decisión de corredores se sienta entre el embed de la rampa que lo informa y el capstone que él informa, sosteniendo tres filas fechadas que los humanos leen pero que ningún código importa.](assets/v08-diagram.webp)

## Challenge

La barrera de esta lección es el registro mismo, sostenido a la forma que pide la evaluación. Produce el registro de decisión de corredores para los compradores de EE. UU., la UE y Brasil de Wavelength: una tabla de tres filas con las columnas riel, activo de liquidación y merchant-of-record, una línea de justificación por fila, más el esqueleto de config que compila. Barrera de aceptación: cada fila nombra las tres columnas explícitamente; cada cifra en movimiento que cites en cualquier parte del registro lleva una fuente y una fecha; la fila de Stripe declara el límite de $10,000 y el camino de reembolso en stablecoin; la fila BR puede decir en una frase por qué PIX es no negociable para ese corredor; `npx tsc --noEmit` se queda en silencio.

Después el estiramiento, que es donde se construye el músculo real de la lección: revisión adversarial de tu propia tabla. Para cada fila, escribe el argumento de una frase a favor del riel que no elegiste. Si no puedes escribir un argumento genuino para la alternativa, no tomaste una decisión, hiciste una adivinanza que dio la casualidad de caer en una casilla defendible. Y corre el ejercicio de obsolescencia: marca qué celdas de tu tabla apostarías que siguen valiendo en seis meses, y cuáles volverías a verificar antes de apostar el almuerzo. La cifra de volumen comercializado y el plazo de liquidación cotizado no deberían sobrevivir ese ordenamiento sin marca. Si lo hicieron, vuelve a leer la sección de MoonPay.

## Checkpoint, y el próximo cliente en la puerta

Ahora deberías poder mirar cualquier pitch de aceptación y ubicarlo en dos ejes en unos diez segundos: en qué liquida el comercio, y a qué corredores llega de verdad. Ese reflejo, más la disciplina de fechar cada número de proveedor, vale más que cualquier fila específica de la tabla de hoy, porque las filas van a derivar y los ejes no. Si la lección funcionó, la frase "damos soporte a pagos cripto" ahora te suena como un distribuidor diciendo "despachamos discos," y tu respuesta inmediata es: a qué territorios, liquidando en qué, sobre el papel de quién. Ese es el instinto de estructura de mercado que este módulo existe para instalar, y si escribir las líneas de justificación se sintió más difícil que leer las comparaciones, bien. Se supone que así sea. La lectura era la parte barata.

Dónde está parada la tienda: dinero adentro, dinero afuera, y ahora cada geografía de comprador humano mapeada a un riel con los términos por escrito. Lo que saca a la superficie lo extraño del próximo cliente en la puerta. No tiene geografía. No tiene tarjeta, ni banco, ni clave PIX, y nunca va a ver tu página de checkout, porque los próximos clientes no son humanos en absoluto: agentes y máquinas que pagan por llamada de API, miles de veces, en montos demasiado pequeños como para que a cualquier procesador de la tabla de hoy le importe. La API de Wavelength para el precio de los prensados está por recibir un paywall cuyos compradores son bots, y el riel para eso es el que la posición en cuatro frentes de Stripe no paraba de insinuar. Próximo módulo: x402.
