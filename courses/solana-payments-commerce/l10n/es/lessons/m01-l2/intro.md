# Finality vs. el stack de tarjetas: el modelo mental de los pagos

## Resumen

La lección pasada decodificaste una transferencia de USDC en vivo y escaneaste un código QR de pago que generó tu propia terminal. Esta es la lección de modelo mental del curso: la última que es sobre todo pensar, aunque todavía te pone dos scripts y un archivo de política en la carpeta. Tres jugadas. Primero ponemos tu vocabulario de tarjetas al lado de los niveles de commitment de Solana, uno a uno, y sacamos la regla de decisión que vas a usar en cada pago: qué nivel esperas, dado cuánto vale el pago. Segundo desarmamos lo que cuesta un pago aquí, porque el costo se diferencia de las comisiones de tarjeta en forma y no solo en tamaño, y la forma es lo que da vuelta la economía de las ventas de ticket pequeño y del float de crédito que cada comercio brasileño financia sin decirlo. Tercero miramos de frente la asimetría: las transferencias liquidadas no se pueden revertir, por nadie, nunca, lo que borra el fraude por chargeback (contracargo) y la red de seguridad de tu cliente de un solo golpe. Cerramos con la pregunta que todo modelo necesita: ¿cuándo son estos rieles la elección equivocada?

Cómo se reparte el trabajo hoy: lee el modelo con las manos en el regazo, escribe conmigo en el lab, y el Challenge del final lo haces solo. Cada lección te entrega un poco más del trabajo que la anterior, y desde el próximo módulo estás construyendo.

## El modelo mental de los rieles del dinero

La lección pasada viste moverse el dinero. Lo que todavía no tienes es el modelo que explica por qué se mueve como se mueve, y ese modelo es la diferencia entre un checkout que funciona sin ruido y uno que despacha $2,000 de inventario contra un pago que nunca existió.

Así que antes de cualquier teoría, corre esto. El mismo RPC público de la lección pasada, sin billetera, sin claves:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[{"commitment":"processed"}]}'
```

Te devuelve un número de **slot** de nueve dígitos, y esa palabra necesita su definición antes de que el número signifique algo. Un slot es una ventana de tiempo fija, actualmente de 300 milisegundos, en la que un validador designado alcanza a producir un bloque. Los slots avanzan para siempre, numerados desde el primer día de la red, así que un número de slot es una lectura de reloj y la diferencia entre dos de ellos es una duración que puedes multiplicar. Ese es todo el truco que estás a punto de hacer.

Ahora corre el comando de nuevo con `"commitment":"finalized"` en lugar de `"processed"` y resta los dos resultados. La brecha suele andar alrededor de treinta slots, que a 300ms por slot son unos nueve segundos. Guarda ese número. Para el final de esta lección vas a saber qué te compra esa brecha, por qué tu PSP nunca te mostró nada parecido, y por qué uno de tus mejores instintos de tarjeta, "el banco siempre puede revertirlo", está a punto de volverse el supuesto más caro de tu código.

Tu PSP te enseñó tres palabras: autorización, captura, liquidación. Estos rieles también tienen tres palabras: `processed`, `confirmed`, `finalized`. El mapeo entre los dos vocabularios es real y te va a llevar bastante lejos. El lugar donde se rompe es donde está el dinero.

### Tu vocabulario de PSP, traducido

Empieza por lo que ya sabes, porque es un apoyo genuinamente bueno. En los rieles de tarjeta un pago es una promesa por etapas. La autorización dice que los fondos existen y están apartados. La captura dice que piensas tomarlos. La liquidación, días después, dice que el dinero se movió de verdad entre bancos. Tres etapas, confianza en aumento, y tu código de integración se ancla en la etapa en la que estás.

Solana también tiene una escalera de confianza en aumento. Solo que se mide en otra unidad: qué tan segura está la red de que el bloque que contiene tu pago va a sobrevivir.

Tres palabras cargan esa escalera, así que tómalas ahora. Un **validador** es una de las máquinas que corren la red. El **stake** es el SOL que ese validador tiene bloqueado como lo que se juega, y la influencia de un validador es proporcional a él, así que "el stake de la red" es la forma correcta de contar cabezas aquí en lugar de contar máquinas. Un **voto** es una transacción que un validador publica diciendo "he visto este bloque y estoy construyendo sobre él", y los votos son lo que convierte la opinión de un nodo en la de la red.

- **`processed`**: algún nodo ejecutó la transacción y la puso en un bloque. Ese bloque todavía puede ser descartado si la red discrepa un momento sobre la punta de la blockchain, lo que se llama un fork. Piénsalo como ver la tarjeta entrar en la terminal. Algo pasó. Nada está prometido.
- **`confirmed`**: validadores que tienen una supermayoría del stake de la red votaron sobre el bloque. En la práctica este es el nivel que hace el trabajo pesado, y la reversión en este punto deja de ser un evento realista.
- **`finalized`**: se construyeron y se votaron suficientes bloques más encima del tuyo como para que ningún fork pueda desalojarlo. Eso es lo que llena la brecha de nueve segundos que midiste: no esperar, sino apilar. Voto tras voto cae en slots posteriores, y cada uno eleva el costo de deshacer el tuyo hasta que queda fuera del alcance de cualquier coalición. No hay nivel más profundo. Esto es liquidación, solo que llega en segundos en lugar de días.

![Una tabla de traducción que empareja autorización con processed, captura con confirmed y liquidación con finalized, con la nota al pie de que la liquidación de tarjeta es reversible por chargeback mientras que finalized no lo es.](assets/v01-comparison.png)

Entonces, ¿contra qué nivel construyes? La guía oficial es cualitativa, y vale citarla en su forma: usa `confirmed` para la mayoría de los pagos, espera `finalized` cuando el pago es de alto valor o sensible a compliance, y trata `processed` como solo para la UI, porque una transacción processed puede ser descartada en un fork. Esa sola oración es tu motor de políticas. Todo lo demás es ajuste.

Ahora los números, con cuidado. ¿Cuánto tarda cada peldaño? La experiencia del ecosistema pone `confirmed` en alrededor de uno a dos segundos, y las cifras que solía citar para `finalized`, alrededor de diez a trece segundos, se recogieron con tiempos de slot más viejos. Quiero ser preciso sobre qué son esas cifras: estimaciones de gente que mira la red, no números impresos en la documentación oficial. La documentación te da la tabla cualitativa y se detiene. Y cualquier cifra que derives de la aritmética de slots tiene que usar el tiempo de slot objetivo actual de 300ms, no los 400ms o 350ms que vas a encontrar en artículos más viejos, porque ese número se está recortando por etapas: 400ms cayó a 350ms el 2026-08-21, 350ms cayó a 300ms en la época 1024 el 2026-08-28, y dos recortes más, a 250ms y luego a 200ms, ya están detrás de un gate en el código del validador y ya viven en devnet. Un tutorial que derive de 400ms ahora sobreestima cada cifra de reloj de pared en un tercio, y uno que hardcodea los 300ms de hoy quedará obsoleto el día que se active el próximo gate. Objetivo, ojo, no medición: el tiempo de slot real en reloj de pared corre un poco por encima del objetivo cuando se mide, así que cada número que derives de 300 es un piso y no una promesa. Tu experimento de curl de la apertura ya te dio la medición: alrededor de treinta slots entre `processed` y `finalized`, y treinta slots al objetivo de 300ms son unos nueve segundos, un poco más a tiempos de slot medidos. Ese es todo el método, y los recortes por etapas son exactamente por qué el método vale más que cualquier cifra de este párrafo: derívalo, nunca lo memorices, porque el número del mes que viene sale de los mismos dos comandos de shell y una multiplicación. Vale la pena notar lo que no tuviste que hacer: en los rieles de tarjeta, el momento de la liquidación es algo que tu adquirente te cuenta en un PDF, y aquí lo derivaste tú.

![Un pago pasa de enviado a processed en un slot de 300ms, a confirmed en uno a dos segundos, y luego a finalized irreversible en unos nueve segundos.](assets/v02-flowchart.png)

Aquí está la derivación que hace que la regla de política sea tuya y no mía. ¿Por qué no esperar siempre `finalized` y estar a salvo? Porque nueve segundos y algo son una eternidad en una terminal de punto de venta, y para un café de $3 la cosa contra la que te estás asegurando, un fork descartando un bloque confirmed, no es un riesgo que valga nueve segundos extra del tiempo de una fila. ¿Por qué no usar siempre `confirmed` y ser rápido? Porque "no es un evento realista" e "imposible" son afirmaciones de ingeniería distintas, y cuando el pago es de $2,000 compras la imposible. La escalera existe para que puedas ponerle precio a la espera frente al ticket. Tu PSP tomó esta decisión por ti y te cobró por el privilegio. Aquí, la decisión está expuesta, y es tuya. Esa es la forma que se repite en todo este curso: los rieles te entregan el dial en bruto y tú construyes la política.

Un reflejo para guardar, uno para tirar. Quédate con el instinto de confianza por etapas; mapea hermosamente. Tira el instinto que dice que las etapas son problema de alguien más. No hay ningún adquirente aguas abajo de ti volviendo a revisar nada.

### Lo que cuesta un pago aquí

Hora de la anatomía del costo, y aquí es donde el modelo deja de ser un ejercicio de traducción y empieza a ser un caso de negocio.

En los rieles de tarjeta presupuestas un porcentaje más un monto fijo, algo como 2.9% más 30 centavos en un procesador típico. El porcentaje es la parte estructural: la red se lleva una tajada del valor movido, así que una venta más grande cuesta más de mover, y una venta diminuta apenas sobrevive a sus propias comisiones. Cada decisión de precios que hayas tomado aguas abajo de eso, pedidos mínimos, recargos, los carteles de "mínimo $10 con tarjeta" en el mostrador, existe porque la comisión es un porcentaje.

La comisión base aquí es de 5000 lamports planos por firma, y eso necesita desempacarse antes de poder compararse con nada. **SOL** es el token nativo propio de Solana, el que la red cobra sus comisiones en; no es una stablecoin, su precio flota, y es una cosa completamente separada del USDC en el que te pagan tus clientes. Un **lamport** es la unidad más pequeña de SOL, una milmillonésima parte de uno. Así que 5000 lamports son 0.000005 SOL, y convertir eso en dinero es una multiplicación por lo que sea que valga el SOL cuando lo corras.

Haz la multiplicación tú mismo en lugar de confiar en una cifra incrustada en un curso: a $150 por SOL la comisión es de $0.00075, a $300 es de $0.0015. En todos los precios a los que SOL ha operado, una firma cuesta una fracción pequeña de un centavo. Esa banda es la forma defendible de la afirmación, y es la forma que deberías citar en una reunión, porque la cifra en dólares se mueve con un mercado y la cifra en lamports no.

Pero lo barato no es la propiedad interesante. Lo plano es la propiedad interesante. La red te está cobrando por un slot de trabajo, no llevándose un porcentaje del valor movido, así que una transferencia de $2,000 y una de $0.50 le cuestan a la red la misma fracción de centavo para liquidarse.

Corre un ejemplo de números redondos, del tipo que vas a rehacer en la cabeza para cada decisión de producto de este curso. Un cliente compra un disco de $12 en Wavelength Records, la tienda que estamos construyendo. Rieles de tarjeta: 2.9% de $12 son unos 35 centavos, más el monto fijo de 30 centavos, digamos 65 centavos, o 5.4% de la venta que se fue. Estos rieles: una fracción de centavo, que a ese tamaño de ticket es un error de redondeo sobre un error de redondeo, unos tres órdenes de magnitud menos. Ahora encoge el ticket. Una venta de $0.50 en rieles de tarjeta cuesta 31 centavos procesarla, así que la comisión se come el 62 por ciento de la venta, y por debajo de unos 31 centavos la comisión excede el ticket completo y la venta deja de ser posible del todo. Por eso nadie vende cosas de 50 centavos una por una en internet. Aquí a la comisión no le importa que el ticket se haya encogido. Categorías enteras de negocio, micro-pagos, cobro por artículo, APIs de pago por llamada, dejan de ser chistes y empiezan a ser partidas.

![Un gráfico de barras en escala logarítmica de las comisiones de tarjeta subiendo de 31 centavos a 58 dólares en cuatro tamaños de ticket, mientras la comisión base de Solana se queda plana y por debajo del centavo.](assets/v03-chart.png)

Dos notas al pie antes de alejar la cámara, porque un modelo de costos con partidas escondidas es peor que ningún modelo. Primero, 5000 lamports son la comisión base; los momentos de congestión pueden agregar una pequeña comisión de prioridad encima, y vas a conocer ese dial más adelante en el curso. Segundo, hay un costo único que la comisión por pago esconde: las cuentas de token cuestan rent para crearse. Piensa en el rent como el alquiler de la terminal en tu analogía de tarjetas, un costo fijo de montar la caja y no una tajada de cada venta. Qué es en realidad una cuenta de token, y la partida exacta del rent, llega el próximo módulo, donde vas a crear una con tus propias manos. Para hoy basta con que el modelo tenga un espacio para eso: costo por pago cerca de cero, costo único de configuración de cuenta pequeño pero real.

Ahora aleja la cámara, porque la comisión plana es el motor detrás de un número que ya conociste. La lección pasada te dio el stock: alrededor de $15.87B de stablecoins sentadas en Solana. Esta lección te puede dar el flujo. En 2024, el volumen de transferencias de stablecoins entre blockchains se reportó en $27.6 billones, una cifra que la guía de pagos con stablecoins de Helius dice que superó a Visa y Mastercard juntas (su afirmación fechada, consultada el 2026-08-21). En Solana específicamente, el $1 billón de volumen de 2025 que viene de la lección pasada es una cifra de año completo, no un contador en vivo. Divide ese flujo por ese stock, y sí, esto es aritmética gruesa que mezcla un flujo de 2025 con una foto de 2026, pero cada dólar en circulación rotó del orden de sesenta veces en un año. Esto no es dinero estacionado. Es dinero haciendo lo que hace el dinero cuando moverlo no cuesta nada: se mueve.

Y la comisión plana decide quién puede participar en ese movimiento. La versión más fuerte de ese argumento está al lado de la caja de cada tienda brasileña que hayas integrado. Primero la línea visible: promediada en todo el mercado, la tasa de descuento del comercio corre en 2.13% en crédito y 1.08% en débito (mi propio cálculo a partir de la serie de la API DESCONTODA del BCB, el Banco Central do Brasil, Q1 2026). El segundo eje es el que esta lección no ha puesto en precio: el tiempo. Una venta de crédito à vista —pago de contado— se liquida al comercio en un calendario D+30, y una venta parcelado 12x gotea a lo largo de doce cuotas mensuales. El cliente se fue con la mercancía; tus ingresos están en un plan de pagos.

Los adquirentes le ponen precio a esa espera en público, así que no tienes que adivinar cuánto vale. La tabla de InfinitePay para su tramo de mayor volumen, comercios por encima de R$80,000 al mes (consultada el 2026-09-01; la página misma no lleva fecha), ofrece crédito à vista a 1.62% en el calendario de liquidación y 2.69% a D+1. La línea de 12x dice 2.25% por cuota frente a 8.99% por la venta completa a D+1 — y lee "por cuota" como lo entiende la tabla de tarifas: 2.25% tomado de cada cuota a medida que se liquida, lo que totaliza 2.25% de la venta, no 2.25% apilado doce veces. El 8.99% compra el dinero de esa misma venta a D+1 en lugar de que gotee a lo largo de un año — dinero que de otro modo te llegaría a seis meses y medio en promedio. Los dos spreads se resuelven en alrededor de 1.05% al mes, la cifra que una tabla de tarifas brasileña escribe como 1.05% a.m. (ao mês). Eso no es una comisión de procesamiento. Es interés, y el capital son tus propios ingresos: antecipação es la industria vendiéndote tu propio dinero de vuelta, antes, a una tasa mensual que corre.

![Un gráfico de barras agrupadas con las tarifas publicadas de InfinitePay para su tramo superior: crédito à vista a 1.62 por ciento en el calendario estándar frente a 2.69 por ciento a D+1, una venta 12x a 2.25 por ciento por cuota frente a 8.99 por ciento a D+1, y una banda de conclusión que le pone precio a los dos spreads en alrededor de 1.05 por ciento al mes.](assets/v04-chart.png)

Y cuanto más pequeño eres, peor el trato. Esos números ordenados pertenecen al tramo de R$80,000 al mes; baja por la tabla y la misma venta 12x cuesta 20.39% por debajo de R$3,000 al mes de volumen y 11.51% por encima de R$30,000 en las tarifas publicadas de Ton, o 22.59% contra 13.69% en las de Mercado Pago. Una comisión plana de 5000 lamports no sabe tu volumen mensual, y esa indiferencia es justamente el punto.

Y la espera con precio puesto tampoco es un producto de nicho para tiendas apretadas de efectivo. Núclea, la cámara de compensación donde se registran las cuentas por cobrar de tarjeta brasileñas, reporta R$614.9 mil millones de volumen de antecipação —el adelanto de cuentas por cobrar— contra R$428.6 mil millones en su lectura anterior, con 33.4% más establecimientos haciendo antecipação (cifras verificadas para este curso el 2026-09-01). Financiarte con tus propias cuentas por cobrar es la norma operativa, y está creciendo en los dos ejes. Pon los rieles de esta lección al lado de esa maquinaria: liquidado quiere decir gastable, en segundos, por una fracción plana de centavo, en cualquier tramo de volumen, porque la cuenta por cobrar que toda esa industria monetiza nunca existe. Aquí el float se borra en lugar de descontarse.

Antes de llevar ese argumento a cualquier lugar cerca de un líder de finanzas brasileño, di la próxima oración tú mismo, antes de que te la digan a ti. PIX no le cuesta nada al cliente y a un comercio le cuesta o nada o una fracción de un por ciento — el BCB exige la gratuidad solo para pessoas físicas, o sea personas físicas y no empresas (Resolução BCB 19/2020), así que los PSP pueden cobrarles a los CNPJs (el registro fiscal de una empresa brasileña) y de hecho lo hacen: Stone publicó 0%, Mercado Pago 0.49% para un vendedor nuevo, y Ton 0.99% después de su ventana de prueba cuando estos se leyeron de las propias páginas de precios de los proveedores (consultado el 2026-09-01; los precios de los PSP son promocionales y se mueven, así que vuelve a leerlos antes de citar cualquiera de ellos) — es instantáneo, y para una venta doméstica simple en BRL le gana a los rieles de tarjeta y le gana a lo que construye este curso. Esa es la base honesta de los pagos brasileños, y este curso no va a pretender otra cosa. Si la venta que tienes delante es una a la que PIX sirve, un cliente pagando ahora en reais, usa PIX, y no dejes que nadie te venda una blockchain para eso. Un curso de pagos que no puede decir esa oración en voz alta es publicidad.

Entonces, ¿por qué sobrevive el argumento del float a esa concesión? Mira a qué riel le ganó PIX de verdad. ABECS, la asociación que representa a la industria de pagos con tarjeta de Brasil, puso el volumen de año completo de 2025 en R$3.1 billones en crédito frente a R$1 billón en débito, con las cuotas sin interés (parcelado sem juros, o PSJ) representando 42.6% de ese total de crédito (el comunicado Balanço 2025 de ABECS, publicado el 2026-02-11). PIX ganó la pelea del débito; la venta instantánea de pago inmediato ya le pertenece. Lo que no reemplazó es el crédito, porque una venta parcelado es el cliente comprando la demora misma, y PIX no vende demora. Tampoco nada de lo que construye este curso, así que sé preciso sobre el residuo. Da vuelta ese 42.6%: el resto de la cartera de crédito se reparte entre à vista y cuotas con interés, y es la tajada à vista —ventas de un solo cargo sentadas en el riel de crédito por costumbre, por puntos o por un valor por defecto del checkout, mientras el comercio financia un float D+30 que nadie pidió— la que un riel de push puede pelear sin pretender vender cuotas. Así que la comparación no es PIX, cuya pelea ya terminó. Lo que queda es el stack de crédito, y ahora sabes cómo leer su precio completo: el porcentaje visible, más el float, en términos que empeoran a medida que te haces más pequeño. Los rieles baratos no solo hacen más baratos los pagos existentes. Cambian cuáles de los costos de un comercio son leyes de la naturaleza y cuáles son partidas que puedes rechazar.

### La asimetría: nadie puede recuperarlo

Aquí está la parte del modelo contra la que tus instintos de tarjeta van a pelear más fuerte, así que derivémosla en lugar de afirmarla.

En los rieles de tarjeta un pago es un pull. El cliente te entrega credenciales y tú, a través de tu PSP, extraes los fondos de su cuenta. Como el sistema está construido sobre el pull, necesita una marcha atrás: el cliente tiene que poder disputar un pull que no autorizó, así que la red corre una ruta institucional de reversión, del emisor a la red al adquirente a ti, y un chargeback puede viajar de vuelta por ella semanas después de la liquidación. Ya sentiste el costo de esa maquinaria: retenciones por fraude, reservas rotativas, la comisión de chargeback de $15 en una venta de $12, fondos que "tienes" y no puedes tocar por días. La ruta de reversión nunca fue un agregado; es el precio que paga todo el stack por el dinero basado en el pull.

En estos rieles un pago es un push. El cliente firma una transacción que mueve sus fondos hacia ti; nadie, ni tú, llega nunca a meter la mano en su cuenta. Y una vez que ese push está finalized, es definitivo en el sentido de la física del libro mayor: no hay ruta institucional de reversión, ningún operador de red con un botón de deshacer, ningún banco al que llamar. Toda transferencia liquidada es permanente, punto.

Ahora deriva las consecuencias en lugar de detenerte en el eslogan, porque cortan para los dos lados.

Del lado del comercio, las ganancias son reales: el fraude por chargeback no se reduce, es estructuralmente imposible; no hay reservas rotativas porque no hay nada contra lo que reservar; liquidado quiere decir gastable, en segundos. La industria entera de gestión de disputas por la que tu PSP te cobra no tiene nada que gestionar.

![Los rieles de tarjeta extraen fondos con una flecha punteada de chargeback corriendo hacia atrás por semanas; Solana empuja fondos sin flecha hacia atrás, así que un reembolso es un pago nuevo y separado.](assets/v05-diagram.png)

Del lado del cliente, la misma propiedad se lee muy distinto. La marcha atrás en la que tu cliente fue entrenado a confiar toda su vida adulta se fue. Si los estafan, no hay proceso de disputa esperando. Si teclean mal una dirección, la red no los va a ayudar; un envío equivocado no tiene banco al que llamar. Y tu realidad operativa cambia con eso: los reembolsos todavía tienen que existir, los clientes los van a exigir con razón, pero la red no te da nada. Un reembolso se vuelve un pago push voluntario en la otra dirección, uno que tú diseñas, financias, autorizas y construyes. Ve a buscar la página de reembolsos en la documentación oficial de pagos en solana.com/docs/payments. No existe: la barra lateral va desde transfers hasta Solana Pay y se detiene, y ninguna página debajo de ella describe revertir un pago liquidado. Esa ausencia es la asimetría enunciada como documentación: la lógica de reversión aquí es un feature de la aplicación y no un feature de los rieles, y en el módulo 4 la vas a construir, emparejamiento de pedidos, reembolsos parciales, todo el paquete.

Voy a confesar el reflejo al que esta lección de verdad apunta, porque yo he sido culpable de él: tratar finality como un detalle que se anota y no como una arquitectura para diseñar a su alrededor, archivando mentalmente "no hay chargebacks" como puro beneficio durante semanas antes de caer en cuenta de que también había borrado en silencio la red de seguridad entera de mis clientes y me había puesto la reconstrucción en mi propio roadmap. La asimetría de no tener chargebacks no es un descuento. Es una transferencia de responsabilidad, desde la maquinaria de disputas de la red de tarjetas hacia tu código. Ponle precio como se debe y es un canje que muchos negocios deberían aceptar. Ponle precio de almuerzo gratis y se convierte en una fila de tickets de soporte con tu nombre encima.

### Cuándo son estos rieles la elección equivocada

Todo modelo mental que te entrega este curso viene con la misma sección final, y es la que te mantiene creíble en una revisión de diseño.

Si el negocio que estás integrando depende de derechos de disputa respaldados por el banco, de reversiones iniciadas por el comprador o de la protección contra chargebacks de la red de tarjetas, entonces la finality sin chargebacks quita exactamente aquello de lo que se depende. Un vertical de consumo con mucho fraude donde los compradores esperan reversión a pedido es un mal encaje, no porque la tecnología falle, sino porque la expectativa central del cliente es justo la propiedad que estos rieles borran por diseño. La misma irreversibilidad que baja tu costo de fraude a casi cero también quita la marcha atrás institucional que tu comprador puede creer, razonablemente, que se le debe. Puedes reconstruir maquinaria de confianza en la capa de aplicación, y los módulos posteriores lo hacen, pero deberías entrar sabiendo que estás reconstruyendo algo que los rieles de tarjeta te daban gratis.

¿Dónde encajan mejor estos rieles? Flujos de comisión baja y liquidación instantánea: las ventas donde el intercambio basado en porcentaje se come el ticket, donde un calendario de cuentas por cobrar se interpone entre los ingresos y la nómina, y, más abajo en la lista, ventas que cruzan una frontera, donde la banca corresponsal todavía se come días. ¿Dónde encajan peor? Donde sea que la reversibilidad sea el producto. Si lo que tu cliente está comprando en realidad es la capacidad de cambiar de opinión después de que el dinero se movió, véndele rieles de tarjeta. Saber cuándo no usar tu herramienta nueva es el modelo funcionando.

![Una matriz de dos columnas: buen encaje para ventas de ticket pequeño, flujos que necesitan escapar del float de liquidación y, último, pagos transfronterizos; mal encaje para ventas con mucho fraude, negocios que venden reversibilidad y regímenes de compliance que exigen una marcha atrás institucional.](assets/v06-comparison.png)

Ese es el modelo completo: una escalera en la que eliges un peldaño, una comisión cuya forma da vuelta tu economía unitaria y una asimetría que cambia costo de fraude por responsabilidad. Hora de hacer que produzca decisiones.

## Lab: sondea la escalera y ponle precio a un pago

El lab formaliza tu experimento de apertura en una herramienta pequeña, lee una comisión real de mainnet y termina contigo escribiendo el primer borrador de una política de confirmación real. Escribe conmigo. Necesitas el Node 24 que configuraste la lección pasada; los dos scripts de aquí usan el `fetch` incorporado de Node, así que no hay nada que instalar.

1. Trabaja en la carpeta `wavelength-rails` que hiciste la lección pasada, y crea un archivo llamado `commitment-ladder.mjs`:

   ```js
   // commitment-ladder.mjs
   // Probes mainnet's three commitment levels and prints how far each trails
   // the tip, in slots and in wall-clock time. Node 24 (built-in fetch), no deps.
   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";

   // Mainnet target slot time in ms. 300ms since epoch 1024 (2026-08-28),
   // the second SIMD-0525 staged cut after 350ms took force on 2026-08-21.
   // Two more cuts, 250ms and then 200ms, are gated in the validator code
   // and live on devnet, so re-check this constant before trusting it.
   const SLOT_TIME_MS = 300;

   async function getSlot(commitment) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({
         jsonrpc: "2.0",
         id: 1,
         method: "getSlot",
         params: [{ commitment }],
       }),
     });
     const json = await res.json();
     if (json.error) throw new Error(`RPC error: ${json.error.message}`);
     return json.result;
   }

   const levels = ["processed", "confirmed", "finalized"];
   const slots = {};
   for (const level of levels) {
     slots[level] = await getSlot(level);
   }

   console.log("commitment   slot          behind tip   approx wall-clock");
   for (const level of levels) {
     const behind = slots.processed - slots[level];
     const secs = ((behind * SLOT_TIME_MS) / 1000).toFixed(1);
     console.log(
       `${level.padEnd(12)} ${String(slots[level]).padEnd(13)} ${String(behind).padEnd(12)} ~${secs}s`
     );
   }
   ```

   Resultado esperado: un archivo guardado al lado del `watch-a-dollar.ts` de la lección pasada. Nada se ha ejecutado todavía.

2. Córrelo:

   ```bash
   node commitment-ladder.mjs
   ```

   Checkpoint: tres filas, con `processed` en cero atrás, `confirmed` en cualquier punto entre cero y un puñado de slots atrás, y `finalized` alrededor de 25 a 40 slots atrás, que a 300ms por slot son alrededor de ocho a doce segundos, tu vecindario de nueve segundos. Córrelo tres o cuatro veces. Los números de slot marchan hacia adelante; las brechas se quedan más o menos en su lugar. Estás viendo respirar la escalera. (Las tres llamadas le compiten a la punta de la blockchain entre solicitudes, así que una brecha puede tambalearse unos pocos slots de corrida en corrida, y `confirmed` puede incluso imprimir un número negativo pequeño cuando la punta avanzó entre tu primera y tu segunda llamada. Ese jitter es la medición, no un bug.)

3. Ahora lee una comisión real de la blockchain. Crea `fee-anatomy.mjs`. Una advertencia sobre lo que agarra: toma la transacción limpia más reciente que tocó la cuenta del mint de USDC, y no todo lo que toca un mint es un pago. Puedes caer en una transferencia, o en la contabilidad de algún protocolo. La anatomía de la comisión es idéntica de cualquier forma, que es el punto, pero no le narres la salida a nadie como "un cliente pagó esto".

   ```js
   // fee-anatomy.mjs
   // Finds a recent finalized transaction touching the USDC mint and prints
   // its fee, split into base fee and priority tip. Node 24, no deps.
   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
   const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
   const BASE_FEE_PER_SIG = 5000; // lamports, flat, per signature

   async function rpc(method, params) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const json = await res.json();
     if (json.error) throw new Error(`RPC error: ${json.error.message}`);
     return json.result;
   }

   const sigs = await rpc("getSignaturesForAddress", [
     USDC_MINT,
     { limit: 10, commitment: "finalized" },
   ]);
   const sig = sigs.find((s) => s.err === null)?.signature;
   if (!sig) throw new Error("No clean signature in the last 10; run it again.");

   const tx = await rpc("getTransaction", [
     sig,
     { maxSupportedTransactionVersion: 1, commitment: "finalized", encoding: "json" },
   ]);

   const fee = tx.meta.fee;
   const sigCount = tx.transaction.signatures.length;
   const base = BASE_FEE_PER_SIG * sigCount;
   console.log(`signature:    ${sig}`);
   console.log(`signatures:   ${sigCount}`);
   console.log(`total fee:    ${fee} lamports`);
   console.log(`base fee:     ${base} lamports (5000 per signature)`);
   console.log(`priority tip: ${fee - base} lamports (anything above base)`);
   ```

   Resultado esperado: un segundo archivo guardado. Dos scripts ahora, uno para la escalera y uno para la comisión.

4. Corre `node fee-anatomy.mjs`. Checkpoint: una comisión total en lamports, descompuesta. Con una firma la línea base dice 5000; lo que quede por encima es un priority tip que quien envía eligió agregar. **Un priority tip de 0 es un resultado correcto y extremadamente común**, no un script roto: la mayoría de los remitentes no agrega nada cuando la red está tranquila, así que `priority tip: 0 lamports` quiere decir que el remitente pagó el piso y nada más. Convierte el total usando la aritmética de la sección de costos: divide los lamports entre mil millones para pasar a SOL, luego multiplica por el precio del SOL de hoy. Incluso un priority tip generoso deja el total en una fracción de centavo. Aplica la misma advertencia de RPC público de la lección pasada: el endpoint gratuito de mainnet te va a limitar la tasa si lo saturas, así que si ves un HTTP 429 o un error de RPC, espera unos segundos y vuelve a correrlo.

5. Extiende la tabla Rosetta que construiste en el Challenge de la lección pasada. Ábrela: ya mapeaste signature, sender, amount, mint, memo y settledAt a sus equivalentes de riel de tarjeta, con una columna de fugas para cada uno. Esta lección introdujo tres cosas más que merecen un nombre de riel de tarjeta, así que agrega tres filas y llena la columna de fugas tú mismo:

   | Cosa on-chain | Análogo del riel de tarjeta | Dónde tiene fugas la analogía |
   |---|---|---|
   | comisión base (5000 lamports, plana) | intercambio más comisión del procesador | ? |
   | rent de cuenta de token (próximo módulo) | alquiler de la terminal, un costo único de instalación | ? |
   | `finalized` | fondos liquidados | ? |

   Resultado esperado: tu tabla de la lección pasada, ahora con nueve filas, con las tres celdas de fugas nuevas escritas por ti. Si la columna de fugas de la fila `finalized` no dice algo sobre las reversiones, todavía tienes por delante el punto de toda esta lección.

6. Empieza el artefacto que esta lección de verdad produce: crea `commitment-policy.md` con cuatro encabezados, `High-value orders`, `Everyday payments`, `Micro-payments` y `UI display`, y bajo cada uno escribe una oración que nombre el nivel de commitment que vas a esperar y una oración que lo defienda. Déjalo en bruto. El Challenge llena dos de estos encabezados como se debe y, en el módulo 4, las lecciones de operaciones de pago convierten el archivo en código que corre.

   Resultado esperado: un archivo markdown de cuatro encabezados con ocho oraciones en bruto. Esto es lo único que te llevas de esta lección, así que guárdalo donde lo vayas a encontrar.

![Líneas de tiempo paralelas de una venta de 2,000 dólares: Solana llega a finality irreversible en unos nueve segundos, mientras los rieles de tarjeta liquidan en días y dejan la ventana de chargeback abierta por semanas.](assets/v07-timeline.png)

## Challenge

Aquí no escribes conmigo; esta es tuya. Llegan dos pagos a Wavelength Records.

Primero: un pedido de $2,000 bajo `High-value orders`, un coleccionista comprando una pared de primeras prensadas, con envío hoy. Segundo: un cargo de $0.40 bajo `Micro-payments`, una llamada a la API de precios de prensado que Wavelength les cotiza a otros sellos, cobrada por consulta. Esos dos encabezados son los que llenas; deja `Everyday payments` y `UI display` como las oraciones en bruto que ya escribiste. Para cada uno de los dos, escribe: el nivel de commitment con el que liberas la mercancía, el riesgo concreto que aceptas por no esperar más, y el costo concreto que te niegas a pagar por no esperar, en ese orden. Usa la regla cualitativa de esta lección más tus propios números medidos de la escalera, y recuerda qué son los tiempos: estimaciones que verificaste, no evangelio que heredaste.

Después cierra el archivo con dos oraciones que no tengan nada que ver con los niveles de commitment. Nombra el único reflejo de tarjeta de tus integraciones existentes que no se porta a estos rieles, y nombra lo que su ausencia te obliga a construir más adelante en este curso. Si tus dos oraciones mencionan reversiones y reembolsos, tienes el modelo.

Esa es la decisión producida que esta lección toma como criterio de aceptación: una política que puedes defender en voz alta, no una página que asentiste sin más. Si tu defensa del caso de $2,000 no menciona forks, vuelve a leer la sección de la escalera; si tu defensa de la llamada de $0.40 a la API no menciona la forma de la comisión, vuelve a leer la anatomía del costo.

Esta se alargó para una lección sin build, pero el modelo había que ganárselo, no afirmarlo. Si algún peldaño todavía se siente tambaleante, o tu archivo de política salió distinto de donde esperabas, llévalo a la comunidad del curso y discútelo; un desacuerdo defendido enseña mejor que una tabla aceptada con la cabeza.

Ya puedes defender una política de confirmación, y sabes que los reembolsos son tu trabajo, no el de la red. El próximo módulo dejamos de hablar y construimos: el transfer-kit que toda lección posterior importa, empezando por la única primitiva que esta lección siguió postergando. La cuenta de token, y el rent que cuesta crearla, van primero. Guarda bien el archivo de política: el verificador del lado del servidor que llega en el módulo 4 es donde esos tres encabezados dejan de ser prosa y se vuelven el argumento de commitment que tu código de verdad pasa.
