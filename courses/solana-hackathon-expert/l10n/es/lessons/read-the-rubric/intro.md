# Qué significa ganar en Colosseum

En la lección pasada leíste el registro: dos temporadas de side tracks y los cuatro hackathons a los que Unruggable entró antes de ganar uno, según su tarjeta en colosseum.com/hackathon. En esta lección escribes el archivo que dice *cuánto vale ese registro para ti*.

## Por qué importa

Colosseum revisa tu repositorio, y su página del hackathon, a la fecha de la temporada World's Fair 2026, dice con sus propias palabras que la revisión no mira tu lenguaje, tu framework, tus patrones ni la calidad de tu código. *Léelo dos veces.* La misma página dice que los hackathons de Colosseum son competencias de startups. Esa es la única idea: **los criterios de evaluación te dicen exactamente qué se puntúa**, y lo que se puntúa es una empresa, no el código.

Siempre pongo el ejemplo de un hackathon presencial que gané en Dubái, con una idea que me parecía demasiado simple para colocarse y sin demo planeada. Encajaba exactamente con lo que buscaban los organizadores y el pitch la vendió, y *nada de eso tenía que ver con el código*. La suerte no es un plan que le puedas pasar a un compañero. El brief sí.

## Manos a la obra

1. Abre colosseum.com/hackathon, busca la sección de evaluación y **copia los nombres de los siete factores** en un archivo nuevo, colosseum-brief.md, tal como están escritos, con la fecha en la primera línea. Leído el **2026-09-06**:

```text
factores de evaluación de Colosseum, colosseum.com/hackathon, leído 2026-09-06
Founder + Market Fit
Insight
Product + Execution
Potential Market Size
Founder Communication
Viability
Traction
```

Si la página en vivo muestra otros nombres, **gana la página**, y anota qué cambió. Fíjate bien en Viability (viabilidad): *si tus notas traen otra palabra en ese lugar, son de otra temporada*. La lista también cambia según la página: la página de Eternal de Colosseum lista **seis factores sin Traction** (tracción), leída el mismo día. Copia la lista de la página del hackathon al que vas a entrar.

2. Debajo de cada nombre, **escribe la pregunta que hace la propia página**, en pocas palabras tuyas.

Founder + Market Fit (ajuste entre fundadores y mercado) pregunta si el equipo tiene las habilidades y la experiencia correctas, y por qué está motivado. Insight (la idea clave) pide una intuición única, o una tecnología o tendencia nueva. Product + Execution (producto y ejecución) pregunta qué tan bien funciona el producto, cómo se compara con la competencia y **qué tan rápido entrega el equipo**. Potential Market Size (tamaño potencial del mercado) pregunta qué tan grande es el TAM, el mercado total que el producto podría atender, y si ya es grande *o es pequeño pero crece rápido*. Founder Communication (comunicación de los fundadores) pregunta si los fundadores comunican la visión con claridad. Viability pregunta si esto puede convertirse en un negocio escalable y sostenible. Traction pregunta si el producto ya tiene demanda o ingresos, y qué tan duraderos son.

Cuenta cuántas podría responder un jurado abriendo tu código: **una**, y solo una parte de esa.

3. **Copia la revisión del repositorio** debajo de los factores, las dos mitades:

```text
revisión del repositorio, colosseum.com/hackathon, leído 2026-09-06
busca:          trabajo significativo durante la ventana del hackathon
                trabajo hecho por el equipo, no por un tercero
                funcionalidades priorizadas estratégicamente
no mira:        lenguaje, framework, patrones, calidad del código
```

Priorizadas estratégicamente te dice exactamente qué tiene que mostrar tu recorte: quieren ver que **dejaste cosas afuera a propósito**, y *un historial de commits lo muestra con más honestidad que cualquier deck*.

4. Anota **cómo se produce la puntuación**. La página, leída el 2026-09-06, dice que una entrega pasa por varias rondas internas de evaluación, después una lista corta llega al panel de jurados, un grupo más chico recibe una invitación a una entrevista de 15 minutos por Zoom, y los ganadores se anuncian **más o menos un mes** después de la fecha límite de entrega. También dice qué artefacto se abre primero: el video de presentación de dos a tres minutos es, en sus palabras, 'one of the first resources judges review' (uno de los primeros recursos que revisan los jurados). Escribe las dos cosas en el brief, y *pon el nombre de quien se queda disponible durante el mes después de la fecha límite*.

![Una entrega a Colosseum conoce a los jurados primero por su video, pasa rondas internas y una lista corta sin el equipo, después una entrevista de 15 minutos, con los ganadores nombrados un mes después.](assets/v01-flowchart.webp)

5. **Ordena los siete** en las cuatro preguntas que un jurado tiene que decidir, y escríbelas debajo de los factores en cuatro líneas:

```text
un problema real:      Insight, Potential Market Size, Viability (preguntada con dinero de por medio)
funciona:              Product + Execution (la competencia y la velocidad de entrega son la misma pregunta con el tiempo)
la historia convence:  Founder Communication (respuestas escritas, video, entrevista de 15 minutos)
este equipo:           Founder + Market Fit, la revisión del repositorio (trabajo del equipo), la entrevista
```

Traction no es una pregunta nueva. Son las dos primeras respondidas con **evidencia en vez de argumentos**: la gente ya lo usa, así que el problema es real, y aguanta, así que funciona. Viability y Traction son las dos líneas que agrega una competencia de startups. Junto a cada pregunta, anota lo que tu proyecto ya puede decir y **escribe "abierto" donde no pueda**. La línea de Viability de Fiado dice "quién paga por el fiado: todavía abierto", y su línea de Traction es una meta, "cinco tiendas con fiados reales para la fecha límite, y cuántas vuelven", nunca una afirmación, *porque un número inventado en el día 0 es peor que un espacio en blanco*.

**Traction es el factor que más subestiman los equipos**, y es el que un mes todavía puede mover. Un jurado no puede valorar la tecnología si nadie usa el producto construido encima. Traction es **gente usando el producto**, contada, y no espera a mainnet: tiendas con fiados reales mientras los pagos se liquidan en devnet es tracción, *y un deploy en mainnet que nadie abre no lo es*. El curso construye hacia ese conteo desde la semana 1.

![Los siete factores de Colosseum se ordenan en cuatro preguntas, con Traction alimentando dos de ellas como evidencia y Viability y Traction marcadas como las adiciones de la competencia de startups.](assets/v02-flowchart.webp)

6. **Copia la fecha límite y conviértela** con una herramienta. La página, leída el 2026-09-06, dice que el hackathon World's Fair va del 14 de septiembre al **12 de octubre de 2026**, y que la fecha límite de entrega es la fecha de cierre de la temporada. Imprime la apertura como timestamp y el cierre como fecha, así que el brief lleva la fecha, *una nota para releer la hora en la página en vivo en la última semana*, y la línea convertida a tu hora local cuando exista. Practica con la cadena que la página sí imprime, el momento de apertura:

```bash
python3 -c "from datetime import datetime; print(datetime.fromisoformat('2026-09-14T11:00Z').astimezone())"
```

Eso imprime el momento en la zona horaria de tu máquina, con el desfase incluido. **Python 3.11 o más nuevo** entiende la Z final por su cuenta, en una versión más vieja reemplaza la Z por +00:00, y verifica el comportamiento contra la documentación de datetime de tu versión. Cuando la hora de la fecha límite aparezca en la página, pégala en lugar de esa cadena y guarda el resultado con el original en UTC al lado, *para que un compañero en otra zona horaria pueda rehacer la cuenta*.

![La temporada World's Fair abre el 2026-09-14T11:00Z y corre hasta el 12 de octubre de 2026, cuya hora todavía hay que releer y convertir, con los ganadores nombrados más o menos un mes después.](assets/v03-timeline.webp)

7. **Cita las líneas que descalifican**, tomadas de la página:

```text
descalifica, colosseum.com/hackathon, leído 2026-09-06
una entrega de producto por equipo, y por lo tanto una por persona: nadie entra con un segundo proyecto por su cuenta
el líder del equipo completa la entrega antes de la fecha límite: el brief nombra quién tiene ese rol
tergiversar el historial de desarrollo, o no declarar código preexistente: descalifica, expulsa, revoca un premio
```

La página agrega que una violación del Código de Conducta también puede descalificar, una cuarta línea que copias sin comentarios. Si construyes con un equipo de agentes y traes código viejo tuyo, **la tercera línea es para ti**: escribe qué existía antes de que abriera la ventana y dilo en la entrega, *porque un historial de commits que arranca el día antes de la temporada parece justo lo que la página prohíbe*.

8. **Dale quince minutos a la página de otro hackathon** y escribe tres líneas: qué pide, bajo cuál de las cuatro preguntas cae, y qué descalifica. Un hackathon de temporada o un side track va a tener su propia página, con sus propios entregables y su fecha límite. Lee esa página el día que decidas entrar, y reutiliza el material de entrega que ya tienes. Sea lo que sea que esa página nombre, **etiquétalo con una de las cuatro preguntas**, *porque una página más corta nunca hace una quinta*. La forma, donde los corchetes angulares son lo único que reemplazas:

```text
<hackathon>, <url>, leído <fecha>
pide:          <los criterios tal como los nombra la página>, cada uno etiquetado con una de las cuatro preguntas
descalifica:   <las líneas de la propia página>
```

**No es una tabla**: con tres líneas alcanza para decidir si entrar y qué adaptar.

## Está listo cuando

- Cada factor en **colosseum-brief.md** es una cita de la página en vivo y lleva fecha.
- Cada una de **las cuatro preguntas** apunta a uno o más factores, con una nota o un "abierto" debajo.
- La fecha límite está en tu zona horaria **con el desfase escrito**, y una segunda herramienta (la app de reloj de tu teléfono cuenta) da la misma respuesta.
- Lo que los jurados abren primero, el paso de la entrevista y **las líneas que descalifican** están en el archivo, con fecha.
- La lectura de quince minutos de otra página produjo **tres líneas, no una tabla**.

## Ojo con

- Colosseum está abierto a builders de todos los ecosistemas blockchain, con tracks de premios dedicados por ecosistema, y el Accelerator exige alguna forma de integración con Solana, así que **Solana es un track al que entras**, y la razón por la que tu proyecto está en Solana va en tu línea de Insight.
- Las fechas, los premios y los tracks cambian cada temporada, y los factores solo se han mantenido hasta ahora, así que relee la página en la última semana: *mi apuesta, de ver equipos y no de ningún conteo*, es que una fecha límite **convertida una vez y nunca releída** pierde más hackathons que una mala demo.
- Los siete de Colosseum cubren todo lo que pide una página más corta, pero Viability y Traction le cuestan al builder solitario de Fiado más o menos una semana del mes hablando con una dueña de tienda, una semana que una temporada de Colosseum puntúa dos veces, bajo Traction y otra vez bajo Founder Communication cuando la historia abre el video, y una página de ensayo sin esa línea **no la puntúa en ningún lado**, *así que elige a propósito y escribe por qué*.

## Lo que queda

Colosseum puntúa una empresa, no el código: los siete factores se ordenan en cuatro preguntas, **un problema real, funciona, la historia convence, este equipo**, y Viability y Traction hacen dos de ellas con dinero y evidencia de por medio. El código vive en una línea de las siete, y un jurado conoce tu proyecto primero por un video de dos a tres minutos, sin ti en la sala. *Cada entregable que construyas de aquí en adelante se revisa contra esas cuatro preguntas antes de seguir.*

## Próxima lección: una oración antes del reloj

En la próxima lección el reloj todavía no arrancó y ya tienes un pitch. **Una oración, escrita en el día 0**, antes de que exista una tarjeta de equipo o un plan del mes. *La vas a odiar, y esa es la idea*, porque las cuatro preguntas que acabas de ordenar son las que esa oración tiene que sobrevivir.
