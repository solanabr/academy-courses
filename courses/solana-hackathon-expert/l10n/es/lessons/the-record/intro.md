# El registro

## El archivo que la mayoría de los equipos nunca abrió

En algún lugar de un archivo JSON público hay una línea que dice que Superteam Brasil puso **diez mil dólares sobre la mesa** para equipos brasileños en la última temporada global. Mi apuesta es que la mayoría de los equipos que podrían haberlos reclamado **nunca abrió el archivo**. Esta lección te muestra lo que hay adentro, *para que empieces el mes sabiendo lo que los equipos que se lo saltaron no sabían*.

Esta es la fila de Frontier, la última temporada global completada, consultada el 2026-09-06:

```text
temporada     Frontier (la última temporada global completada)
side tracks   54
total         US$439,410
una fila      Side Track Superteam Brasil, US$10,000 USDG
```

Nada de esto estaba escondido. Estuvo detrás de una URL pública durante toda la temporada: una entrada por cada side track que Superteam Earn corrió en paralelo a la temporada Frontier de Colosseum, cada una con un patrocinador, un monto y un pedido. Si alguna vez quieres ver la versión en vivo por tu cuenta, un solo comando devuelve la lista completa, y *no lo vas a necesitar más de una o dos veces por temporada*:

```bash
curl -sL https://superteam.fun/api/hackathon/frontier | python3 -m json.tool
```

Lo que sabían los equipos que sí lo leyeron es simple: una temporada de Colosseum paga desde **más de una bolsa**, un premio principal y una lista larga de premios regionales, cada uno con su propio patrocinador y su propio pedido, *ganados con la misma entrega*. Un mes de hackathon tiene un registro, y **el registro es público**.

![Un solo endpoint devuelve los 54 side tracks de Frontier por US$439,410, filtrados hasta la fila de US$10,000 USDG de Superteam Brasil, mientras que los enlaces con prefijo /earn/ terminan en un 404.](assets/v01-flowchart.webp)

## El equipo que perdió tres veces primero

Abre colosseum.com/hackathon en un navegador y **busca la tarjeta de Unruggable**. Dice que el equipo compitió en cuatro hackathons antes de ganar el gran premio: Renaissance, una mención honorífica. Radar, otra mención honorífica. Breakout, primer lugar en un track. Cypherpunk, el gran premio. Ese es el registro tal como lo puso la tarjeta de Colosseum el 2026-09-08, y la misma página, leída el 2026-09-06, decía que Colosseum corre **dos hackathons globales al año**, de abril a mayo y de septiembre a octubre. Un equipo que trata una temporada como un veredicto *está tirando a la basura la segunda mitad del año*.

![Unruggable se llevó menciones honoríficas en Renaissance y Radar, primer lugar en un track de Breakout y luego el gran premio de Cypherpunk, cuatro temporadas a dos por año.](assets/v02-timeline.webp)

## Dos temporadas, lado a lado

La temporada anterior a Frontier cuenta la otra mitad de la historia. Cambia una palabra en ese comando, frontier por cypherpunk, y el endpoint responde por la temporada anterior. Consultada el 2026-09-06, Cypherpunk traía **41 side tracks** por un total de US$341,750, y la fila brasileña decía Superteam Brasil x Tangem Wallet por **US$11,200**. Entre las dos temporadas *la cantidad subió* y la fila brasileña pasó de un track copatrocinado a una de un solo patrocinador con un monto menor. Cambia la palabra por worldsfair y, en la misma fecha, el endpoint devolvió **una lista vacía**: ningún side track publicado todavía para una temporada que no había abierto. *Una respuesta vacía también es parte del registro*, y te dice que vale la pena mirar la página otra vez cuando la temporada abra.

El curso lleva un proyecto de ejemplo por todas las lecciones, y aquí lo conoces. **Fiado** es la libreta de fiado de una tienda de barrio brasileña, el nombre, el saldo acumulado y la fecha que la dueña guarda junto a la caja, convertida en una cuenta liquidada en stablecoin con un recordatorio de pago. *Fiado es brasileño, así que su fila regional es la de Superteam Brasil*, y sus dos temporadas del registro se ven así:

![Frontier muestra 54 tracks y US$439,410 con una fila de US$10,000 USDG de Superteam Brasil, Cypherpunk muestra 41 tracks y US$341,750 con una fila de US$11,200, y World's Fair todavía estaba vacía.](assets/v03-table.webp)

Vale la pena quedarse con dos cosas de esa tabla. Primero, **cada número lleva una fecha**, porque el registro te dice dónde estaban el dinero y la gente la temporada pasada y nada seguro sobre dónde van a estar en esta. Cuando tu temporada abra, la página merece *una mirada fresca*, no el hábito de consultarla una y otra vez. Segundo, tu propia región tiene una fila en esa lista, o una fila ausente, y lo mismo cada país vecino, cada uno con **su propio patrocinador y su propio pedido**. Un equipo brasileño que entraba a Frontier estaba en dos concursos a la vez con la misma entrega, uno global y uno solo contra su propio país, y esa asimetría es, creo yo, *el dato menos aprovechado en los hackathons de Solana*. La fila regional es el camino más corto a un premio, y *lo que los jurados de Colosseum puntúan es lo que hace que un proyecto merezca un premio en primer lugar*, así que un buen mes sostiene las dos cosas.

## Cuatro palabras, el mes, tres reglas

**Una temporada global** es uno de los dos hackathons al año de Colosseum, y el reloj de este curso es el mes entre su apertura y su fecha límite de entrega. **Un side track**, el track regional, es un premio regional atado a una temporada global: un patrocinador, casi siempre un Superteam regional, que pone un monto y un pedido. **Un premio regional** es el side track cuyo patrocinador es el Superteam de tu región. **Un ensayo** es cualquier otra competencia a la que entras con el mismo paquete: una temporada global de Colosseum es el objetivo, y un hackathon de temporada de Superteam Brasil antes de la siguiente temporada de Colosseum, o un side track de Superteam Earn en paralelo a la temporada a la que entras, *es un ensayo*.

El orden de las lecciones **sigue el calendario**. Antes del reloj, lees la página del hackathon y sus criterios de evaluación, confirmas un equipo, escribes un primer pitch y preparas el repositorio del toolkit en el día 0. La semana 1 lleva una idea hasta la evidencia y una decisión escrita de **matar, pivotar o seguir**. Las semanas 2 y 3 construyen la porción demostrable en devnet. La semana 4 *convierte la porción en la historia*. Entregar arma el paquete y escribe un plan para la próxima temporada, y Ensayo, opcional, lo exporta a los hackathons de ensayo.

El último día **tienes seis cosas en la mano**: una porción demo en devnet, un deck, un video de presentación de dos a tres minutos, un video demo de tres minutos o menos, un one-pager de GTM con socios nombrados y las respuestas del portal redactadas con anticipación. Cada pieza anterior que construyes *alimenta una de esas seis*.

Tres reglas de la casa:

1. Ves la cosa **antes de que tenga nombre**, y cada lección posterior pone un comando, una consulta o una página dentro de sus primeros cientos de palabras.
2. **El pitch existe desde el día 0**, y a la primera versión se le permite estar mal, *lo que no se le permite es no existir*.
3. **Revisa cada pieza contra los siete factores de evaluación** y el pedido del patrocinador antes de seguir, y *la siguiente pieza no arranca hasta que la actual pasó con honestidad*.

![El curso recorre seis tramos, antes del reloj, semana 1, semanas 2 y 3, semana 4, entregar y el ensayo opcional, cada uno produciendo artefactos con nombre y el pitch versionado desde v0.](assets/v04-timeline.webp)

## Próxima lección: los siete factores

Ya conoces la tabla de premios. En la próxima lección **abres la página del hackathon de Colosseum** y lees los siete factores de evaluación que publica, uno por uno, como los lee un jurado, y los copias en un archivo con la fecha en la primera línea. Lo primero que vas a notar es que *la calidad del código no es uno de ellos*.
