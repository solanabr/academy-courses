# El deck para el jurado, no el deck para inversionistas

En la lección pasada escribiste el one-pager de GTM, argumentaste por qué Solana para el track y para la aceleradora, y registraste una respuesta de un socio. Deja el one-pager abierto, y abre al lado la bitácora narrativa y competitor-map-sizing.md, porque esos tres archivos **ya son el deck**, con la forma equivocada.

## Tres minutos y dos convicciones

Tu paquete es una pestaña entre muchas y un jurado le dedica **unos tres minutos**. En esos minutos el jurado tiene que entender un problema real y llegar a creer que este es el equipo que lo va a resolver. Los ganadores que vi tenían la presentación más convincente de la sala, y algunos tenían menos código que los equipos que quedaron debajo de ellos, porque *un problema que necesita solución, vendido de la manera correcta, le gana a las funcionalidades*.

Así que el deck se **construye desde el problema hacia afuera**, y lo primero que se construye es el orden. Abre un archivo nuevo llamado deck-judge.md, pon la fecha en la primera línea y escribe **siete encabezados** debajo, uno por línea. Todo lo demás de hoy va debajo de esas siete líneas.

## Haz esto

1. **Escribe los siete encabezados**. Para Fiado, empezado el 2026-10-05, el archivo dice:

```text
deck-judge.md, Fiado, temporada World's Fair de Colosseum, empezado 2026-10-05
1 portada         el problema en una oración, el nombre del producto en pequeño
2 problema        la libreta junto a la caja, y cuántas cajas hay
3 demo            el fiado en devnet, la transacción que un jurado puede abrir, la línea de tracción
4 qué es nuevo    lo que hace la porción que la libreta y la app del banco no hacen
5 por qué Solana  para el track y para la aceleradora
6 este equipo     quiénes somos, por qué este problema, y quién paga
7 próximos pasos  lo que todavía no está construido, y la petición
```

Una decisión ya está tomada: *el nombre del producto no aparece hasta que el problema se haya dicho*.

2. **Revisa la forma contra el kit** antes de llenarlo. Un deck para jurados tiene **de 5 a 7 diapositivas**, una de ellas una demo funcionando, una la novedad técnica, una el por qué Solana, con notas para el orador de 30 a 60 segundos en cada diapositiva. Así es como el skill de pitch-deck del kit enruta a la audiencia de jurados de hackathon, leído el 2026-09-06, y el mismo skill enruta a una audiencia de VC a 10 a 12 diapositivas y a una audiencia de grant o de aceleradora a 8 a 10.

![El kit enruta a los jurados de hackathon a 5 a 7 diapositivas con una demo, la novedad y el por qué Solana, mientras los decks para VC van de 10 a 12 y los de grant o aceleradora de 8 a 10.](assets/v01-comparison.webp)

3. **Corre la entrevista de 12 preguntas del kit** con los cuatro archivos abiertos: competitor-map-sizing.md, la bitácora narrativa, el one-pager con la lista de socios y el pitch v2. **Verifica la invocación** y la lista de preguntas contra el README actual del kit primero, *porque esta lección tiene fecha*. Cada respuesta es una copia de uno de esos archivos, y una pregunta que ninguno de ellos puede responder o pertenece a un deck para VC o es *un hueco en el paquete encontrado antes de que un jurado lo encontrara*. El skill escribe un esquema en markdown, luego las diapositivas con sus notas, luego una autoevaluación y una sesión de preguntas y objeciones. **Quédate con el esquema**, todavía no con sus diapositivas, y lee la autoevaluación como una primera pasada, no como una calificación.

![Cuatro artefactos existentes alimentan la entrevista del kit, que se convierte en un esquema, siete diapositivas, notas, una autoevaluación y objeciones, luego una ronda de feedback que fija el pitch, y al final el reacomodo para la temporada.](assets/v02-flowchart.webp)

4. **Llena la portada** y la diapositiva del problema. La portada es una oración, y **la oración es el problema**, con el nombre del producto en pequeño en una esquina. Para Fiado: una dueña de tienda lleva cuarenta fiados abiertos en una libreta de papel, y la libreta se pierde. Esa oración es la v2 hasta el paso 11.

Después **escribe el one-liner**, que es un objeto distinto de la oración de portada: la respuesta a *¿qué es esto?* en **cinco palabras o menos**, para el tagline del formulario de envío, la esquina de la portada junto al nombre y las primeras palabras de un post. Tres reglas, en orden. **Primero, sin ambigüedad**, porque diez personas que lo lean deberían imaginarse el mismo producto, y *si se imaginan diez productos la línea falló*. **Segundo, que emocione**, porque nadie se emociona con un producto que no entendió. **Tercero, suficientemente verdadero**, es decir que pinta la imagen correcta y los detalles vienen después.

Usa las palabras que usaría tu mamá tomando un café. **Nada de jerga**, y ninguna de las metáforas de lego, bloques de construcción, capas o pegamento. Dos formas funcionan: la analogía, *algo que todo el mundo conoce más tu giro*, y la descripción simple, un verbo, un objeto y un contexto. **Escribe cuatro candidatos y elige uno**, con la razón al lado. Los de Fiado:

```text
candidatos de one-liner, Fiado, 2026-10-05
préstamos para tiendas de barrio     mal: un fiado no es un préstamo, nadie presta nada
fiados que se pagan solos            ambiguo: diez lectores, diez productos
libreta de fiado en dos celulares    claro, la forma de analogía, seis palabras
fiado que ambos lados ven            elegido: cinco palabras, una imagen, verdadero hoy
```

La diapositiva del problema **se lleva la mayor parte del tiempo**: tres líneas cortas sobre lo que pasa, para Fiado la libreta junto a la caja, que se moja o se pierde o un cliente habitual se muda con un saldo abierto, y la dueña persigue cuarenta deudas pequeñas ella sola. Debajo de esas líneas, una línea de números: **los tres números de dimensionamiento** de competitor-map-sizing.md, cada uno con su fuente y la fecha en que se leyó. En el deck para el jurado *el mercado vive aquí y en ningún otro lugar*.

5. **Llena la demo** y la diapositiva de qué es nuevo. La demo es **una captura de pantalla** de la pantalla del fiado tomada de la bitácora narrativa, con la firma de la transacción en devnet debajo *para que un jurado pueda abrirla*. Debajo va **la línea de tracción**, copiada de traction-log.md con su fecha: para Fiado, cuatro tiendas con fiados reales, 34 fiados abiertos, tres tiendas que volvieron sin que se les pidiera, liquidación en devnet. *Si el registro está vacío, la diapositiva dice devnet y se detiene ahí*, y el hueco va a la lista de objeciones. Qué es nuevo es **la novedad técnica en tres líneas**, lo que hace la porción que la libreta y la app del banco no hacen: para Fiado, un fiado que ambos lados pueden leer, que se liquida en una stablecoin, con un recordatorio que la dueña no tiene que mandar ella misma. Aquí no va ningún diagrama de arquitectura.

![El deck de Fiado para el jurado va portada, problema, demo, novedad, por qué Solana, equipo y próximos pasos, y cada número apunta al archivo de dimensionamiento, la bitácora narrativa o el one-pager.](assets/v03-table.webp)

6. **Llena el por qué Solana** y la diapositiva del equipo. Por qué Solana es el párrafo de la lección pasada **cortado a tres líneas**, una para lo que hace la porción, una para lo que la cadena hace más fácil de eso, una para la aceleradora, sin números de benchmark.

La diapositiva del equipo responde a **Founder + Market Fit** (ajuste entre fundadores y mercado). En la página de Colosseum leída el 2026-09-06 ese factor pregunta si el equipo tiene las habilidades y la experiencia correctas para este mercado y por qué está motivado para resolver este problema, así que cada persona recibe una línea de lo que sabe hacer y una línea de por qué este problema. Para Fiado la segunda línea es *la persona que ha estado detrás de esa caja y ha visto la libreta mojarse*. Debajo del equipo, una línea: **quién paga**, copiada del one-pager, o la razón por la que todavía no hay precio. Esa es la respuesta de Viability (viabilidad).

7. **Llena los próximos pasos y la petición**. La diapositiva de próximos pasos dice **qué no está construido todavía**, con fecha, en el orden en que lo construirías, y es *el lugar honesto para la arquitectura que no construiste*. Para Fiado eso es el programa del fiado que pone el monto de apertura y la fecha de vencimiento on-chain junto a los pagos, los rieles de fiat, el onboarding de billeteras y después el despliegue con la cooperativa, las cosas que la tarjeta de alcance cortó en la semana 2. Un jurado lee una funcionalidad no construida aquí como un plan y la misma funcionalidad en la diapositiva de la demo como una mentira.

Después **la petición en una línea**: para Fiado la aceleradora, y una presentación con una segunda cooperativa, con nombre, con la respuesta del socio de la lección pasada como prueba de que hay una primera. **Lee las siete en voz alta** con un cronómetro y escribe el tiempo en la primera línea del archivo.

8. **Escribe las notas**, de 30 a 60 segundos por diapositiva, como oraciones habladas. Una nota que es una lista de viñetas se lee con la voz plana que la gente usa para las listas, y *la diapositiva del problema muere con esa voz*. La nota de la diapositiva del problema de Fiado dura **unos treinta segundos**: la libreta junto a la caja, unos cuarenta nombres con un total acumulado, el día que se mojó, y los tres números en la pantalla con las páginas de donde salieron. El borrador del kit suele ser más largo, así que cortar es la mayor parte del trabajo.

9. **Escribe las preguntas y objeciones**: las preguntas que un jurado va a hacer, cada una con una respuesta de una línea, escritas antes de que alguien pregunte. **Con cuatro alcanza** para una primera lista. Las de Fiado, el 2026-10-05, dicen:

```text
objections.md, Fiado, 2026-10-05
por qué no la app del propio banco       el banco no conoce al cliente habitual, la tienda sí, y el fiado está entre esos dos
qué pasa si la dueña pierde el celular   el fiado no está en el celular, un celular nuevo lee el mismo fiado
quién paga                               la línea de quién paga del one-pager, dicha de un tirón
por qué una cadena, para empezar         la línea de por qué Solana de la diapositiva 5, sin la palabra rápido
```

Escribe las tuyas antes de la ronda de feedback, *porque las tres personas van a preguntar dos de estas de todas formas*.

10. **Corre la ronda de feedback 3**: el deck leído en voz alta a tres personas que no lo han visto. Una tiene el problema, una dueña de tienda para Fiado y para ti quienes sean tus primeros 100 usuarios. Una es builder. Una no sabe nada de ninguna de las dos cosas, y *esa es la que te dice si la diapositiva 2 funciona*.

Después de la lectura, pídele a cada una que diga el problema en una oración y anota la oración que dijo, *no la que tú querías decir*. Luego pregunta qué es el producto **en cinco palabras** y anota eso también, *porque un one-liner que dos lectores oyen distinto es ambiguo, no ingenioso*. **Registra tres entradas**: quién, qué dijo, qué cambió. Si **dos de las tres** caen cerca de la portada, el pitch es final. Si no, reescribe la diapositiva del problema y léela otra vez el mismo día, y sigue siendo la ronda 3, con más de tres entradas.

11. **Fija el pitch final**. Es la oración de portada después de la ronda, y desde aquí no cambia, *porque los videos de la próxima lección se cortan a partir de ella y cada cambio posterior cuesta una regrabación*. Escribe **final y la fecha** al lado en el registro. El de Fiado, después de la ronda 3:

```text
pitch final, Fiado, semana 4, fijado 2026-10-06, final
Una dueña de tienda que pierde la libreta pierde cuarenta deudas pequeñas, así que Fiado
guarda el fiado en su teléfono, le muestra a cada cliente el mismo saldo, cobra parte
en USDC y manda el recordatorio por ella.

ronda 3, dicho de vuelta en una oración:
Lucia   "la tienda que pierde su libreta"                    cayó en la portada
Marcos  "pagar el fiado de una tienda desde tu celular"      cayó en la mitad del producto
Jorge   "la del dueño que deja de perseguirte"               cayó en la portada
cambió: nada; dos de tres cayeron; final
```

He visto un pitch peor que su producto perder, y perder por el pitch, con el mejor producto yéndose a casa, y por eso este curso **reescribe el pitch cuatro veces** antes de que un jurado lo vea.

12. **Reacomoda el mismo archivo** en la exportación para la temporada, si tienes un ensayo en el calendario. Un hackathon de temporada o un side track (el track regional) va a tener su propia página con sus propios entregables y su fecha límite, así que **lee esa página el día que decidas entrar** y reutiliza el paquete que ya tienes. No se escribe nada nuevo: **las siete diapositivas se reacomodan** en las secciones de esa página, y las notas conservan sus palabras. Si la página pide una diapositiva de mercado, constrúyela solo desde competitor-map-sizing.md, los tres números, cada uno con su fuente y la fecha en que se leyó. El argumento a favor de lo pequeño y con enlace sobre lo grande y sin fuente se hizo en la lección de investigación de mercado, y la diapositiva no lo reabre. Yo matizaría eso a la mayoría de los jurados, pero *el que califica un factor de tamaño de mercado está preguntando si el número es real*.

13. **Ahora el tuyo**. Al deck de Fiado, copiado en tu repositorio del toolkit, le faltan la diapositiva del problema y la de próximos pasos, y tú **escribes las dos desde la bitácora narrativa**: la diapositiva del problema desde la primera captura fechada de la bitácora y la entrada de decisión que nombró al usuario, la de próximos pasos desde las decisiones que cortaron alcance en la semana 2, cada una con su fecha. Después **tu propio deck-judge.md**, llenado desde la entrevista y los cuatro archivos, luego las notas, las objeciones, la lectura a tres personas, la oración de portada marcada como final, y la exportación para la temporada desde el mismo archivo si hay un ensayo en el calendario. Si tres personas son tres calendarios, *léeselo a una hoy y a dos mañana y conserva las entradas*. La ronda son las entradas, no el día.

![El pitch pasa de la v0 antes de que arranque el reloj a la v1 en la semana 1 y la v2 en las semanas 2 y 3 hasta la final en la semana 4, con las rondas de feedback 1, 2 y 3 unidas a las últimas tres versiones.](assets/v04-timeline.webp)

## Está listo cuando

- La exportación para el jurado tiene **menos de ocho diapositivas**, cada una con una nota.
- Si hay un ensayo en el calendario, su exportación es **el mismo archivo reacomodado** a los entregables de esa página, sección por sección.
- Cada número en cualquier diapositiva se rastrea hasta **el archivo de dimensionamiento** o el paquete de evidencias, con el nombre del archivo escrito al lado.
- Existen **tres entradas de feedback** para la ronda 3 con quién, qué dijo y qué cambió, y la oración de portada está marcada como final con una fecha.

## Ojo con

- Una diapositiva de título que **nombra el producto antes que el problema**: la primera diapositiva más común en un hackathon, y *gasta la única diapositiva que un jurado lee con ojos frescos*.
- Una diapositiva de mercado con un número que **no está en el archivo de dimensionamiento**: un jurado de tres minutos pasa por encima de una diapositiva de mercado, así que *mantenla honesta antes que grande* y pon esa hora en la diapositiva del problema.
- Una diapositiva de equipo con **tres fotos y tres cargos** y ninguna razón de por qué este equipo: pierde puntos en un factor que lleva Founder en el nombre.

## Lo que queda

El deck para el jurado es **un objeto distinto del deck para inversionistas**: siete diapositivas construidas desde el problema hacia afuera, una nota hablada debajo de cada una, y cada número apuntando a un archivo que ya escribiste. La oración de portada es la cuarta versión del pitch, y se le leyó a tres desconocidos antes de marcarla como final. *Si un desconocido que solo vio la diapositiva 2 puede decir tu problema en una oración, el deck funciona.*

## Próxima lección: los primeros veinte segundos

En la próxima lección el deck se convierte en dos videos, y la primera acción es **escribir los primeros veinte segundos** del guion de la presentación a partir de la diapositiva del problema que acabas de fijar, *porque esos veinte segundos deciden si un jurado ve el resto*. Trae las notas, son la forma larga del guion, y trae la lista de objeciones para el video de la demo, donde un jurado está pensando esas preguntas mientras la transacción se confirma.
