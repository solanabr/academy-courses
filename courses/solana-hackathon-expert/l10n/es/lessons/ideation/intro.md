# Problemas que de verdad tienes

En la lección pasada escribiste la tarjeta de equipo, el plan del mes con fechas y un pitch v0 de una oración, y tres personas ya reaccionaron a esa oración. **Deja abierto el registro de feedback**, *porque la oración está a punto de pasar una prueba más dura que la que tres amigos le pueden hacer*.

## Por qué importa

La semana 1 arranca aquí, y lo primero que haces con tu idea es **tratar de matarla**. Las buenas ideas de hackathon que vi ganar salieron de un problema que alguien del equipo, o alguien cercano, tiene o tuvo, y las que arrancaron de "DeFi para X" en su mayoría no se colocaron. Esa es la diferencia entre **problema-primero**, una persona y algo que no puede hacer hoy, y capacidad-primero, una función de la cadena buscando a quién aplicársela. Los siete factores no puntúan la tecnología por sí misma, y *un jurado que lee las respuestas escritas se da cuenta cuando la persona se agregó en la semana 4*.

Hoy generas tres ideas problema-primero y las puntúas hasta que una sobreviva. **Abre Claude Code** dentro del repositorio del toolkit que inicializaste en el día 0, con el kit instalado como lo hizo la lección 3, y arranca con el paso 1.

## Corre el sprint con Fiado

Un recordatorio antes del primer paso: **Fiado es el proyecto de ejemplo del curso**, nada más. Es la libreta de fiado de la tienda de barrio de la lección 1, convertida en una cuenta que se liquida en stablecoin, y cada laboratorio de este curso corre primero sobre ella *para que siempre tengas un ejemplo resuelto que copiar*. Después corres los mismos pasos con tu propio problema.

1. **Pide el skill idea-sprint** por su nombre, con el problema de Fiado como entrada, en tus propias palabras:

```text
Corre el skill idea-sprint. El problema: la dueña de una tienda de barrio les fía a sus
clientes habituales y lleva cada fiado en una libreta de papel junto a la caja. El mes
pasado la libreta se mojó y unos cuarenta fiados abiertos quedaron ilegibles.
Entrevístame como esa dueña de tienda.
```

*La invocación exacta cambia con cada versión del kit*, así que verifícala contra el **README actual del kit** antes de escribirla. El kit está en github.com/solanabr/solana-ai-kit.

2. **Responde la entrevista** como la dueña de la tienda, sin rodeos, *y no la empujes hacia la idea del fiado*. El skill pregunta qué pasa hoy, qué se rompe, **quién paga cuando se rompe** y qué hace la dueña al respecto, y no acepta "estaría bueno que" como respuesta. La dueña de Fiado dice que los clientes habituales compran fiado, que algunos pagan el cinco cuando cobran y otros no, que la libreta se mojó, que ella lo pagó con dinero que nunca cobró y con clientes habituales que dejaron de venir, y que ahora le saca foto a cada página con el teléfono hasta que el teléfono se llena.

3. **Mira cómo el filtro de necesidad de cripto** le hace la misma pregunta a cada candidata: si la construyes con una base de datos y una app web, ¿qué se rompe? Si no se rompe nada, la idea no necesita una cadena, y *ponerle una debajo es un costo que el jurado va a ver*. Si se rompe algo concreto, **escribe esa oración**, porque es tu línea de por qué Solana para todo el mes y va directo a la respuesta de Insight (la idea clave).

   La respuesta de Fiado no es "pagos", porque una tienda ya puede aceptar tarjeta hoy. Lo que se rompe es **la confianza entre dos personas** que no confían en una tercera: la dueña tiene el registro, y cuando la libreta desaparece los dos quedan adivinando. Un fiado que se liquida en una stablecoin sobre una cadena que ninguno de los dos opera es *un registro que los dos pueden leer y ninguno puede editar después*.

4. **Espera exactamente tres candidatas**, y toma una de al lado. Los ecosistemas vecinos, las tiendas, apps y hábitos que están justo al lado de tu problema, están llenos de cosas que funcionan en papel, y *algo que funciona en papel muchas veces funciona en papel por una razón*.

   Las tres de Fiado: el fiado que se liquida en una stablecoin con un recordatorio en la fecha que eligió el cliente, el registro de pagos a proveedores por el efectivo que la misma dueña les entrega a los distribuidores el día de la entrega sin ningún comprobante, y las tarjetas de sellos de papel que reparte una panadería a dos calles, como un token que un cliente habitual guarda en una billetera. **El registro reprueba el filtro** porque solo la dueña lo lee, *así que no hay nadie de quien desconfiar*. **La tarjeta de sellos reprueba** porque ya es un registro que las dos partes pueden ver.

![El idea-sprint corre una entrevista sin rodeos, un filtro de necesidad de cripto, exactamente tres candidatas y una puntuación sobre 15 que cae en go, condicional o no-go con un pivote.](assets/v01-flowchart.webp)

5. **Lee la puntuación**. Cada candidata recibe un número sobre 15, y la línea que traza el kit, a la fecha del 2026-09-06, está **en 8**: 8 o más es un go, 6 a 7 es un condicional, menos de 6 es un no-go con un pivote adjunto. Un condicional tiene que nombrar lo único que lo convertiría en un go, y si el skill no lo nombra, pregúntale, porque *un condicional sin condición es un no educado*.

   En el sprint de Fiado el fiado queda en 11 para un go, el registro de proveedores en 6 para un condicional con el pivote "¿quién más necesita leer este registro?", y el token de lealtad en 3 para un no-go. Tus números no van a ser esos, y lo que sí debería mantenerse es **el orden y los veredictos**. Si el token de lealtad quedó por encima del fiado, **relee tu entrevista**, porque la razón más común es que *respondiste como un builder al que le gustan los tokens y no como una dueña de tienda que perdió una libreta*. De qué se componen los 15 puntos cambia entre versiones, así que **lee el desglose** en la salida del propio skill el día que lo corras.

```text
puntuación sobre 15, idea-sprint, github.com/solanabr/solana-ai-kit, leído 2026-09-06
8 o más     go
6 a 7       condicional, con la condición nombrada
menos de 6  no-go, con un pivote adjunto
```

![En el ejemplo resuelto el fiado puntúa 11 de 15 para un go, el registro de proveedores 6 para un condicional, y el token de lealtad 3 para un no-go.](assets/v02-table.webp)

6. **Abre el archivo que escribió el skill**:

```bash
cat .claude/context/idea.md
```

Leído el 2026-09-06, el idea-sprint del kit escribe ahí **exactamente tres candidatas**, cada una con su puntuación sobre 15 y su go, condicional o no-go. A la fecha del 2026-09-06 el skill está adaptado de dos skills de la colección solana-new de sendaifun, find-next-crypto-idea y validate-idea, y el repositorio lo dice. **Guarda ese crédito en tus notas**, *porque el upstream es adonde vas cuando el kit cambia*.

## Busca en el registro, después corre el tuyo

7. **Encuentra los proyectos pasados** más cercanos a cada candidata y escribe qué no hicieron. Eso es análisis de brechas, y *tu candidata vive en la brecha*. Lo que hicieron mal es otra nota para otra lección, y una candidata sin brecha es **un clon del ganador de la temporada pasada**, que los jurados ya puntuaron una vez.

   A la fecha de la temporada World's Fair 2026, el blog de Colosseum reporta que la temporada Cypherpunk, con entregas hasta el 30 de octubre de 2025 y ganadores anunciados el 13 de diciembre de 2025, tuvo más de 9,000 participantes y **1,576 proyectos finales**, repartidos en los tracks Infrastructure, Consumer, DeFi, Stablecoins, RWAs y Undefined. **Undefined es un track**, así que *las categorías no son una lista de ideas de dónde elegir*. Qué búsqueda haces depende de un token.

   Con un PAT: Colosseum Copilot es un skill que, a la fecha del 2026-09-06, indexa **más de 5,400 proyectos de hackathon** de dos años de entregas, más 6,300+ productos a través de The Grid, y corre en Claude Code, Codex y OpenClaw. El kit lo trae como el submódulo ext/colosseum, que apunta a github.com/ColosseumOrg/colosseum-copilot, versión 1.2.1 el día en que se leyó, bajo una licencia propietaria. **Consigue el personal access token** en colosseum.com/arena/copilot después de iniciar sesión, *porque sin él el skill no responde*, y verifica la versión y la página del token contra el README actual del kit. **Pídele a Copilot los proyectos más cercanos** a cada una de tus tres candidatas y *lee los que se terminaron, no solo los que se entregaron*.

   Sin un PAT: abre los **posts de ganadores de Cypherpunk y Frontier** en blog.colosseum.com, busca en cada uno las palabras de tu candidata, y sigue los nombres hasta los perfiles de empresa en colosseum.com/companies. **Lee tres perfiles por candidata**. La página del hackathon lista solo a los ganadores del gran premio, y las páginas de proyecto en arena.colosseum.org piden iniciar sesión en Colosseum antes de mostrar algo, leído el 2026-09-07, así que *los posts del blog son la puerta abierta*. No cuesta nada, y *apostaría a que la mayoría de los equipos que se colocan lo hicieron por el camino lento al menos una vez*.

![Con un PAT de Copilot la búsqueda corre sobre más de 5,400 proyectos indexados, y sin uno la misma nota de brecha se escribe desde los posts de ganadores de temporada del blog y los perfiles públicos de empresa.](assets/v03-flowchart.webp)

8. **Escribe una nota de brecha por candidata**, incluida la no-go, con una forma fija: el proyecto pasado más cercano, su temporada, qué construyó, qué no hizo y la fecha en que buscaste. La nota de Fiado nombra a Yumi Finance, leída en el post de ganadores de Cypherpunk el 2026-09-07, y *lo que copias es la forma*:

```text
nota de brecha, candidata 1, el fiado, buscado 2026-09-07
más cercano:  Yumi Finance, Cypherpunk, primer premio del track DeFi (post de ganadores en blog.colosseum.com)
construyó:    un producto onchain de compre ahora, pague después que maneja la evaluación de riesgo mediante originación de préstamos
no hizo:      sostener un fiado entre una tienda y un cliente habitual que ya se conocen; sin evaluación de riesgo, sin préstamo, sin prestamista nuevo
segundo más cercano: Corbits, Cypherpunk, segundo del track Infrastructure / no hizo: herramientas para comercios con endpoints x402, sin crédito entre las dos partes
```

El proyecto más cercano de la no-go suele ser el que te dice por qué fue una no-go, así que **la puntuación queda con su comprobante**. Si el fiado es la única candidata con nota, *no hiciste un análisis de brechas, saliste a buscar permiso*. Una candidata en 11 con un proyecto más cercano que ya hace todo lo que ella hace **ya no está en 11**, y el memorando lo dice, en la fila de la candidata, con la fecha.

9. Ahora tus tres. **Escribe tres personas** con las que hablaste en las últimas dos semanas y que se quejaron de algo: tu papá o tu mamá, el casero, la persona del mostrador, el primo de un compañero. *Cuanto más cerca estés de la persona, mejor sale la entrevista*, y si ninguno de los tres es un problema que tú pagarías por resolver, busca una cuarta persona antes de correr nada.

   **Corre el sprint**, una vez por problema o una vez con los tres, lo que soporte la versión actual del skill, y responde sin rodeos. Si te descubres escribiendo la respuesta que lleva a la idea con la que llegaste, *esa es justo la idea que el sprint existe para probar*, así que escribe la respuesta verdadera y deja que puntúe.

   Después **corre la búsqueda de brechas** para las tres y escribe las tres notas en el memorando de ideas, con una fila marcada como elegida. Si la candidata elegida puntuó menos de 8, toma el pivote que adjuntó el skill, *porque repetir la entrevista hasta que salga el número que querías es lo mismo que no puntuar*. La primera línea de la fila elegida es **una persona y un problema**, en ese orden, la misma regla que el pitch v0, porque las próximas tres lecciones leen esa línea una y otra vez.

![El memorando de ideas terminado tiene tres filas de candidatas puntuadas con una marcada como elegida y una nota de brecha con fecha para cada una, incluida la no-go.](assets/v04-table.webp)

## Está listo cuando

- El **memorando de ideas** tiene tres candidatas, cada una puntuada sobre 15, cada una con el filtro respondido en una línea.
- Una candidata está marcada como elegida, su puntuación es **8 o más**, y su primera línea es una persona y un problema.
- Cada una de las tres, la no-go incluida, tiene una **nota de brecha** que nombra al menos un proyecto pasado, qué no hizo y la fecha de la búsqueda.
- La **línea de fuente** del memorando nombra .claude/context/idea.md y la consulta a Copilot o las páginas públicas donde buscaste.

## Ojo con

- Elegir la idea que **luce más tecnología**, porque los siete factores no puntúan la tecnología por sí misma.
- Buscar en el registro **recién después de elegir la idea**, porque tener tres candidatas sirve justamente para que el registro pueda reordenarlas, y lo hace.
- Tratar una memecoin, o **un clon del ganador de la temporada pasada**, como una brecha.

Un memorando de ideas con puntuación **mata ideas que amas**, y ese es su trabajo, porque el equipo que se salta la puntuación para proteger a su favorita no se libra de la evaluación, se la deja a los jurados en la semana 3, *que la hacen en silencio y nunca mandan el pivote*. Si tu favorita puntuó menos de 6, **discute con la entrevista** y no con el número, y si las respuestas fueron honestas la idea se queda en el memorando como una no-go con fecha que puede volver con un 9 la próxima temporada.

## Lo que queda

Una idea se gana su lugar sobreviviendo una entrevista sin rodeos, el filtro de necesidad de cripto y una puntuación junto a dos rivales, **no por ser la que traías**. La pregunta del filtro, qué se rompería con una base de datos y una app web, es tu línea de por qué Solana para todo el mes. *Una candidata elegida es una persona y un problema con una nota de brecha con fecha al lado, y una puntuación que esperabas que nadie revisara es una puntuación que los jurados van a revisar.*

## Siguiente

El responsable de la competencia se gana el rol en la próxima lección: quién hace esto hoy, fuera del registro del hackathon, qué hacen bien, qué hacen mal y *el único número que un jurado te va a pedir y que todavía no tienes*. Lo primero que haces ahí es **copiar las tres preguntas** que Colosseum hace bajo Potential Market Size (tamaño potencial del mercado) en un archivo nuevo, antes de nombrar un solo competidor, así que deja el brief y el memorando a la mano.
