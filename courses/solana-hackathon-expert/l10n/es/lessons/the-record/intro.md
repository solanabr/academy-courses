# El registro

## El archivo que la mayoría de los equipos nunca abrió

En algún archivo JSON público hay una línea que dice que Superteam Brasil puso **diez mil dólares sobre la mesa** para equipos brasileños en la última temporada global. Apuesto a que la mayoría de los equipos que podían reclamarlos **nunca abrió el archivo**. Esta lección te muestra lo que hay adentro, *para que empieces el mes sabiendo lo que esos equipos no sabían*.

Esta es la fila de Frontier, la última temporada global que terminó, consultada el 2026-09-06:

```text
temporada     Frontier (la última temporada global que terminó)
side tracks   54
total         US$439,410
una fila      Side Track Superteam Brasil, US$10,000 USDG
```

Nada de esto estaba escondido. Estuvo toda la temporada detrás de una URL pública: una entrada por cada side track que Superteam Earn corrió en paralelo a la temporada Frontier de Colosseum, cada una con su patrocinador, su monto y su pedido. Si algún día quieres ver la versión en vivo con tus propios ojos, un solo comando te devuelve la lista completa, y *no vas a necesitarlo más de una o dos veces por temporada*:

```bash
curl -sL https://superteam.fun/api/hackathon/frontier | python3 -m json.tool
```

Lo que sabían los equipos que sí lo leyeron es simple: una temporada de Colosseum paga de **más de una bolsa**, un premio principal y una lista larga de premios regionales, cada uno con su propio patrocinador y su propio pedido, *y todos se ganan con la misma entrega*. Un mes de hackathon tiene un registro, y **el registro es público**.

![Un solo endpoint devuelve los 54 side tracks de Frontier por US$439,410, filtrados hasta la fila de US$10,000 USDG de Superteam Brasil, mientras que los enlaces con prefijo /earn/ terminan en un 404.](assets/v01-flowchart.webp)

## El equipo que perdió tres veces antes

Abre colosseum.com/hackathon en el navegador y **busca la tarjeta de Unruggable**. Dice que el equipo compitió en cuatro hackathons antes de ganar el gran premio: Renaissance, una mención honorífica. Radar, otra mención honorífica. Breakout, primer lugar en un track. Cypherpunk, el gran premio. Ese es el registro tal como lo mostraba la tarjeta de Colosseum el 2026-09-08, y la misma página, leída el 2026-09-06, decía que Colosseum organiza **dos hackathons globales al año**, de abril a mayo y de septiembre a octubre. Un equipo que trata una temporada como un veredicto *está tirando a la basura la segunda mitad del año*.

![Unruggable se llevó menciones honoríficas en Renaissance y Radar, primer lugar en un track de Breakout y después el gran premio de Cypherpunk, cuatro temporadas a dos por año.](assets/v02-timeline.webp)

## Dos temporadas, lado a lado

La temporada anterior a Frontier cuenta la otra mitad de la historia. Cambia una palabra en ese comando, frontier por cypherpunk, y el endpoint te responde por la temporada anterior. Consultada el 2026-09-06, Cypherpunk tenía **41 side tracks** por un total de US$341,750, y la fila brasileña decía Superteam Brasil x Tangem Wallet por **US$11,200**. De una temporada a la otra *la cantidad subió*, y la fila brasileña pasó de un track con dos patrocinadores a uno con un solo patrocinador y un monto menor. Cambia la palabra por worldsfair y, en la misma fecha, el endpoint devolvía **una lista vacía**: todavía no había side tracks publicados para una temporada que no había abierto. *Una respuesta vacía también es parte del registro*, y te dice que vale la pena volver a mirar la página cuando la temporada abra.

El curso usa un mismo proyecto de ejemplo de principio a fin, y aquí lo conoces. **Fiado** es la libreta de fiado de una tienda de barrio brasileña, el nombre, el saldo acumulado y la fecha que la dueña anota junto a la caja, convertida en una cuenta que se liquida en stablecoin y manda un recordatorio de pago. *Fiado es brasileño, así que su fila regional es la de Superteam Brasil*, y sus dos temporadas en el registro se ven así:

![Frontier muestra 54 tracks y US$439,410 con una fila de US$10,000 USDG de Superteam Brasil, Cypherpunk muestra 41 tracks y US$341,750 con una fila de US$11,200, y World's Fair todavía estaba vacía.](assets/v03-table.webp)

De esa tabla vale la pena quedarse con dos cosas. Primero, **cada número lleva una fecha**, porque el registro te dice dónde estaban el dinero y la gente la temporada pasada, y nada seguro sobre dónde van a estar en esta. Cuando abra tu temporada, la página merece *una mirada fresca*, no el hábito de consultarla una y otra vez. Segundo, tu región tiene una fila en esa lista, o le falta una fila, y lo mismo cada país vecino, cada uno con **su propio patrocinador y su propio pedido**. Un equipo brasileño que entraba a Frontier competía en dos concursos a la vez con la misma entrega, uno global y otro solo contra su propio país, y esa asimetría es, creo yo, *el dato menos aprovechado de los hackathons de Solana*. La fila regional es el camino más corto a un premio, y *lo que los jurados de Colosseum puntúan es lo que hace que un proyecto merezca un premio, para empezar*, así que un buen mes sostiene las dos cosas.

## Cuatro palabras, el mes, tres reglas

**Una temporada global** es uno de los dos hackathons que Colosseum organiza al año, y el reloj de este curso es el mes entre su apertura y su fecha límite de entrega. **Un side track**, el track regional, es un premio regional atado a una temporada global: un patrocinador, casi siempre un Superteam regional, que pone un monto y un pedido. **Un premio regional** es el side track cuyo patrocinador es el Superteam de tu región. **Un ensayo** es cualquier otra competencia a la que entras con el mismo material de entrega: la temporada global de Colosseum es el objetivo, y un hackathon de temporada de Superteam Brasil antes de la siguiente temporada de Colosseum, o un side track de Superteam Earn en paralelo a la temporada a la que entras, *es un ensayo*.

El orden de las lecciones **sigue el calendario**. Antes de que arranque el reloj, lees la página del hackathon y sus criterios de evaluación, confirmas un equipo, escribes un primer pitch y dejas listo el repositorio del toolkit en el día 0. En la semana 1 conviertes una idea en evidencia y en una decisión escrita de **matar, pivotar o seguir**. En las semanas 2 y 3 construyes el recorte de la demo en devnet. La semana 4 *convierte ese recorte en la historia*. Entregar arma el material y escribe el plan para la próxima temporada, y Ensayo, que es opcional, lo reutiliza en los hackathons de ensayo.

El último día **tienes seis cosas en la mano**: un recorte de la demo funcionando en devnet, un deck, un video de presentación de dos a tres minutos, un video demo de tres minutos o menos, un one-pager de GTM con socios con nombre y las respuestas del portal redactadas de antemano. Cada entregable que construyes antes *alimenta una de esas seis*.

Tres reglas de la casa:

1. Primero lo ves y **después le ponemos nombre**: cada lección que sigue pone un comando, una consulta o una página dentro de sus primeros cientos de palabras.
2. **El pitch existe desde el día 0**. A la primera versión se le permite estar mal, *lo que no se le permite es no existir*.
3. **Revisa cada entregable contra los siete factores de evaluación** y el pedido del patrocinador antes de seguir, y *el siguiente no arranca hasta que el actual pasó con honestidad*.

![El curso recorre seis tramos, antes del reloj, semana 1, semanas 2 y 3, semana 4, entregar y el ensayo opcional, y cada uno produce artefactos con nombre, con el pitch versionado desde v0.](assets/v04-timeline.webp)

## Próxima lección: los siete factores

Ya conoces la tabla de premios. En la próxima lección **abres la página del hackathon de Colosseum** y lees los siete factores de evaluación que publica, uno por uno, como los lee un jurado, y los copias en un archivo con la fecha en la primera línea. Lo primero que vas a notar es que *la calidad del código no es uno de ellos*.
