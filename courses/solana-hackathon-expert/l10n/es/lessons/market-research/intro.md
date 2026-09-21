# Dimensionar a velocidad de hackathon

En la lección pasada puntuaste tres ideas y te quedaste con una, y el memorando de ideas tiene los proyectos pasados más cercanos escritos junto a la sobreviviente. Abre ese memorando. Su **nota de brecha**, la línea que dice qué no hicieron esos proyectos pasados, es *la primera fila del archivo que construyes hoy*.

## Por qué importa

Imagina la entrevista. Un jurado pregunta qué tan grande es esto y el equipo dice miles de millones. La siguiente pregunta es cuánta gente podría usarlo el mes que viene, *y la sala se queda en silencio*. El primer número salió de una diapositiva que alguien encontró en un reporte. El segundo número nadie lo buscó. El segundo número es **el que puntúa**, y al final de hoy tienes los dos, de fuentes que un jurado puede clickear.

Un número que un jurado puede clickear es **un conteo en una página pública**, con la URL y la fecha al lado, que el jurado puede abrir durante la revisión. El mapa de competidores y los tres números de dimensionamiento son esa forma aplicada dos veces. Empieza ahora: abre colosseum.com/hackathon, busca Potential Market Size (tamaño potencial del mercado) en la sección de evaluación y **crea competitor-map-sizing.md** junto al memorando de ideas en el repositorio del toolkit.

## Haz esto

1. **Copia las tres preguntas del factor** en el archivo, con la fecha en la primera línea. Leído el **2026-09-06**, a la fecha de la temporada World's Fair 2026, el factor dice esto:

```text
competitor-map-sizing.md, abierto 2026-09-06
Potential Market Size, colosseum.com/hackathon, copiado 2026-09-06
¿Qué tan grande es el mercado total direccionable para este proyecto?
¿Ya es grande, o pequeño pero creciendo rápido?
¿Cuál será el impacto de este proyecto en la tasa de crecimiento de su mercado?
```

Solo la primera pregunta quiere un número grande. La segunda quiere una dirección, y la tercera quiere lo que tu proyecto le haría a esa dirección. Una diapositiva que dice miles de millones *responde un tercio del factor*.

2. **Nombra al responsable** del mapa. Como yo manejo un equipo de hackathon, una persona es dueña de la investigación de competidores, y esa es quien recibió el rol en la tarjeta de equipo el día 0. Si nadie lo recibió, elige ahora, *porque un mapa sin dueño se llena a medias la última noche*.

Luego haz que el responsable practique la forma en earn.superteam.fun. El 2026-09-06 la página de inicio mostraba **213,730+ usuarios** y 2,680+ patrocinadores. El 2026-09-07 imprimía 214,040 y 2,690. Escribe las dos lecturas en el archivo con la URL y la fecha. Eso es demanda por un producto distinto, pero tiene la forma que necesita cada número de tu archivo: **un conteo, una página, una fecha**, y un signo de más que te dice que la página redondea hacia abajo.

3. **Lista los cinco proyectos** más cercanos al tuyo por el problema del usuario, no por tu stack, *para que una libreta de papel pueda ser un competidor y un protocolo con tu arquitectura quizás no*. **Cinco filas**, porque tres esconden un competidor y diez esconden al lector. Si tu idea es on-chain vas a querer cinco protocolos, y ese es tu stack hablando: vuelve a la oración del pitch del día 0, encuentra a la persona que está adentro y pregunta qué usa hoy. Su respuesta es la fila uno. El encabezado del mapa, antes de cualquier fila, es una línea:

```text
fila  quién  bien  mal  cobra  última actividad (fuente, fecha)
```

**Llena cuatro celdas** por fila. Llena cobra aunque la respuesta sea cero, *porque alimenta la línea de Viability (viabilidad) más adelante*. Llena última actividad desde la página del explorer, la última actualización de la ficha en la tienda de apps, la fecha del commit en GitHub o un conteo público que se movió, **nunca desde un anuncio**, y si el anuncio es todo lo que encuentras, escribe "solo anuncio" en la celda, con fecha.

Para Fiado, cuyo usuario es la tienda de barrio que lleva su crédito en una libreta junto a la caja, las cinco filas van desde la libreta hasta una app fintech que presta en la caja. La fila uno es **la libreta de papel**: gratis, privada, funciona sin teléfono y sin señal, se pierde o se moja, no cobra nada, *activa hoy en cada tienda de la calle*. La fila dos es **la hoja de cálculo o el hilo de WhatsApp**: sobrevive al agua y se puede buscar, todavía se cobra a mano, no cobra nada.

La fila tres es **una app de compre ahora, pague después**, y la más cercana el día en que se leyó esto era Pagaleve: su página para comercios imprime niveles de facturación y "A Pagaleve assume 100% dos riscos" sin comisión al comercio, así que la celda de cobra dice no impreso, cotización a pedido, con fecha, y su inicio imprime +10.000 lojas. La página de Pix Parcelado de Koin, leída el mismo día, imprime "3x sem juros ou em ate 24x" y ningún porcentaje.

La fila cuatro es **la función de crédito** dentro de una terminal de tarjeta o un sistema de punto de venta que la tienda ya alquila. La fila cinco es **una billetera de stablecoin o herramienta para comercios** que ya liquida en una stablecoin y podría agregar un fiado. Las filas cuatro y cinco necesitan un nombre, una página de precios y una lectura de última actividad de páginas que abres hoy:

```text
mapa de competidores, Fiado, 2026-09-06
fila  quién                                bien                              mal                                  cobra                última actividad
1     la libreta de papel                  gratis, privada, sin teléfono     se pierde, se moja, la dueña cobra   nada                 hoy, cada tienda
2     hoja de cálculo / hilo de WhatsApp   sobrevive al agua, buscable       todavía se cobra a mano              nada                 hoy
3     app BNPL: Pagaleve                   asume el riesgo de impago, +10.000 lojas (sitio, 2026-09-07)   <de reseñas, con fecha>   no impreso, cotización a pedido (pagaleve.com.br/varejistas, 2026-09-07)   <ficha en la tienda, con fecha>
4     función de crédito en POS: <nombre>  <de su sitio, con fecha>          <de reseñas, con fecha>              <página de precios, con fecha>  <ficha en la tienda, con fecha>
5     herramienta de comercio en stablecoin: <nombre> <de su sitio, con fecha>   <de reseñas, con fecha>          <página de precios, con fecha>  <página del programa en el explorer, con fecha>
```

Los corchetes angulares son tuyos para quitar. Una fila que todavía tiene uno el viernes *es una fila por la que el jurado va a preguntar*.

![El mapa de Fiado tiene las filas de la libreta y la hoja de cálculo llenas con lo que el equipo sabe, una fila de BNPL llena a medias desde las páginas con fecha de Pagaleve, y las filas de POS y stablecoin dejadas como espacios con fecha.](assets/v01-table.webp)

4. **Lee la fila cinco desde la cadena** primero, *porque no necesita el permiso de nadie*. Un programa en Solana tiene una página en cualquier explorer, con un conteo de transacciones y la hora de la última transacción, sin iniciar sesión. **Copia tres cosas** en la fila: el conteo histórico, el conteo o la lista de los últimos treinta días donde se muestre, y la hora de la transacción más reciente, luego la URL y la fecha.

Ahora lee lo que copiaste. Un conteo histórico grande con una última transacción de hace semanas es un producto que **tuvo usuarios y los perdió**. Un conteo pequeño con una transacción de hace una hora es *un producto que está vivo y es temprano*. Ninguno es lo que dice el sitio del propio proyecto. Un comparable, es decir, un proyecto que hace un trabajo parecido para un usuario parecido, también puede publicar su **TVL**, el valor que la gente ha depositado en él, y ese número con su fecha dice cuánto dinero la gente ya le confía a ese tipo de producto.

Con la pestaña abierta, **haz la lectura de la stablecoin** para el dimensionamiento: la página del token muestra un conteo de holders y las transferencias recientes. El conteo de holders entra como **un límite superior de personas**, *porque una persona tiene varias billeteras y un bot tiene miles*, y las transferencias de los últimos treinta días entran como la lectura de actividad, ambas con la fecha. Si el emisor publica un número regional, cópialo también y marca de qué página salió, porque el número del emisor y el número del explorer no van a coincidir.

Para las filas tres y cuatro **la lectura es off-chain**: una ficha en la tienda de apps muestra la fecha de la última actualización, un repositorio público muestra el último commit, una página de precios muestra el precio y la fecha en que la leíste, y la fecha de la reseña más reciente es una señal de última actividad. Un side track o premio de Earn también sirve, donde la publicación muestra **cuánta gente aplicó**, *porque ese conteo dice cuántos builders creyeron que el problema del patrocinador valía su semana*.

![La celda de última actividad on-chain se arma copiando el conteo histórico de un programa, su conteo reciente y la hora de su última transacción desde un explorer, cada uno con la fecha de lectura.](assets/v02-table.webp)

5. **Escribe los tres números como aritmética** antes de que entre cualquier valor. TAM es el mercado total direccionable, todos los que tienen el problema. SAM es la parte que de verdad podrías atender con este producto, en este lugar, sobre este riel. SOM es la parte de eso que puedes alcanzar en los primeros meses, por un canal que puedes nombrar.

Colosseum puntúa Potential Market Size con las tres preguntas al inicio de tu archivo. Un hackathon de temporada o un side track va a tener su propia página con sus propios entregables y su fecha límite. **Lee esa página el día que decidas entrar**, y reutiliza el paquete que ya tienes. La diapositiva se escribe en la semana 4. La aritmética se escribe hoy, bajo una regla: **ningún número sin sus entradas**, y ninguna entrada sin una URL y una fecha. La aritmética de Fiado:

```text
dimensionamiento, Fiado, 2026-09-06

TAM = A x B
  A = número de pequeñas tiendas minoristas en Brasil      424,120 tiendas de alimentos, todos los tamaños, un límite superior
                                                            (ABRAS Ranking 2025, abras.com.br/dados-ranking-2025, leído 2026-09-07)
                                                            solo tiendas de barrio: participación estimada de A, rango
  B = crédito anual extendido por tienda en el fiado        <estimación, URL de la fuente, fecha>

SAM = TAM x C x D
  C = participación de esas tiendas que llevan un fiado     <participación, URL de la fuente, fecha>
  D = participación cuyos clientes ya pueden pagar en una
      stablecoin o por un riel que el fiado pueda usar       <conteo de adopción, URL de la fuente, fecha>
      contexto: Brasil recibió US$318.8 mil millones en valor cripto, jul 2024 a jun 2025, todo cripto
      (chainalysis.com/blog/latin-america-crypto-adoption-2025, 2025-10-02, leído 2026-09-07); participación de stablecoins: espacio

SOM = E, alcanzable para el mes tres
  E = tiendas alcanzables a través de una cooperativa o
      asociación de comerciantes que el equipo pueda nombrar  <conteo de miembros, su propia página, fecha>
```

Cada letra es una página. A es un conteo público y **un límite superior**, porque la cifra de ABRAS cuenta cada tienda de alimentos y no solo las tiendas de barrio, así que el recorte a tiendas de barrio va al lado como una estimación con un rango. B no tiene página y se estima a partir de conversaciones en la próxima lección, así que escríbelo ahora como un rango con "estimado" al lado. C y D son participaciones, y *una participación necesita dos conteos, el de arriba y el de abajo, ambos con fecha*.

E es **el número más pequeño del archivo** y el que el jurado recuerda. Yo trato una estimación como *una predicción que carga incertidumbre, no un compromiso*, y por eso B y C entran al archivo como un mínimo y un máximo con el razonamiento al lado, y no como una sola cifra confiada.

![El TAM de Fiado es tiendas por crédito por tienda, el SAM lo recorta con dos participaciones con fecha, y el SOM son las tiendas que una cooperativa nombrada alcanza para el mes tres.](assets/v03-flowchart.webp)

6. **Nombra el canal del mes tres** y copia su conteo. Nadie puede alcanzar un porcentaje del SAM. Una cooperativa o una asociación de comerciantes del barrio sí se puede alcanzar, *por una persona, con un teléfono*: tiene una página, la página tiene **un conteo de miembros o una lista**, y ese conteo con su URL y su fecha es E. Escribe la línea del SOM debajo de la aritmética exactamente con esta forma:

```text
SOM, Fiado: <E> tiendas, alcanzables a través de <nombre de la asociación, su página, fecha>,
para el mes tres, porque <la persona que nos va a presentar, y a qué se comprometió>
```

La última cláusula es *el trabajo de la próxima semana*. El conteo es de hoy.

Para la segunda pregunta, la dirección, **copia la página de adopción de stablecoins** en D dos veces, hoy y en la fecha anterior más temprana que la página o un archivo muestre, y escribe las dos lecturas lado a lado. Si la página no tiene una lectura anterior, escribe "una lectura, sin tendencia" y **no inventes una pendiente**. La tercera pregunta, el mecanismo, quiere una oración, y para Fiado dice: *cada tienda que abre un fiado trae a sus clientes habituales al riel*, así que el conteo de adopción en D crece por el número de clientes de la tienda cada vez que E crece en uno.

![El archivo se arma nombrando al responsable de competidores, llenando cinco filas desde páginas con fecha, escribiendo las fórmulas de dimensionamiento con entradas por letra y nombrando el canal del mes tres.](assets/v04-flowchart.webp)

## Está listo cuando

- **competitor-map-sizing.md** está junto al memorando de ideas, con la fecha en la línea uno y las tres preguntas de Colosseum debajo.
- El mapa tiene **cinco filas**, las más cercanas por el problema del usuario, cada una con bien, mal, cobra y última actividad, y cada celda de última actividad apunta a una página de explorer, una ficha, un repositorio o un conteo con fecha.
- **TAM, SAM y SOM** están escritos como fórmulas con entradas por letra, y cada entrada apunta a una URL y una fecha o está marcada como estimada con un mínimo y un máximo.
- **El SOM es un conteo** alcanzable en los primeros tres meses a través de un canal con nombre, un conteo de miembros y una página. Una participación del SAM en esa línea reprueba, *por más razonable que se vea la participación*.

## Ojo con

- **Un número de tamaño de mercado sin su aritmética** genera una pregunta, cada vez, y "un reporte decía" es la respuesta que termina la conversación.
- **Una billetera no es un usuario**, así que un conteo de direcciones o de holders entra como límite superior de personas, nunca como conteo de clientes.
- **Un comunicado de prensa** es el competidor diciéndote lo que quiere que pienses, y nunca llena la celda de última actividad.

Un TAM grande sin camino a los primeros cien usuarios puntúa peor que **uno pequeño y alcanzable**, porque Traction (tracción) y Viability están junto a Potential Market Size en la lista y un SOM del mes tres alimenta a los tres, así que *la honestidad te cuesta la diapositiva dramática y te compra la entrevista*.

## Lo que queda

Potential Market Size hace tres preguntas, y un número grande responde solo la primera. Cada número de tu archivo es **un conteo en una página pública con una URL y una fecha**, la última actividad de un competidor se lee de un explorer o una ficha y nunca de un anuncio, y un conteo de billeteras es un límite superior de personas, nunca un conteo de clientes. *El SOM es el número más pequeño del archivo, alcanzable a través de un canal con nombre, y es el que el jurado recuerda.*

## Siguiente

Ahora tienes un mapa de competidores y tres números, y los estimados del archivo, B y C para Fiado, son rangos esperando evidencia. La próxima lección abre **escribiendo la única suposición** que, si está mal, deja todo el archivo sin valor, y la prueba más barata que podría demostrar que está mal para el viernes. *Pasas el resto de la semana tratando de matar tu propia idea*, y reescribes el pitch con lo que sobreviva.
