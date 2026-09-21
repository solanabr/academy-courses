# El formulario, la fecha límite, las reglas

En la lección pasada grabaste los dos videos dentro de sus límites, la presentación entre dos y tres minutos y la demo en tres o menos con la transacción confirmándose en pantalla, y registraste la ronda de feedback 4. **Ten los dos enlaces** a mano. Este es el capstone: termina con el material de entrega enviado y una captura de pantalla con fecha, *y hoy no se graba nada*.

## Por qué importa

Cada temporada hay equipos con un producto que funciona que **pierden por un formulario**. Una fecha límite convertida al revés. Un segundo envío de un compañero que rompe la regla de uno por persona. Un video de demo que nunca llegó a su campo. Un repo privado al que nadie le dio acceso al revisor. Nada de eso está en los criterios de evaluación. Todo está en las reglas, en la misma página que los siete factores, *unos párrafos más abajo, donde ya casi nadie lee*.

La persona que abre tu envío primero no está calificando Founder + Market Fit (ajuste entre fundadores y mercado). Está revisando que el video esté, que el repo abra, que el equipo tenga nombres. **La completitud es la única línea** de toda esa página que un equipo controla por completo. Por eso hoy **lees las reglas** como leíste los criterios de evaluación en la lección 2, como la lista exacta de lo que se revisa, y envías como despacharías un release: *contra una lista de verificación escrita antes de empezar el trabajo*.

## Manos a la obra

1. **Arma el material de entrega** antes de abrir cualquier portal. Crea un archivo llamado submission-package.md en la raíz del repositorio del toolkit y pon **una línea por cada entregable que construiste**, con la ruta del archivo y su fecha, *primero de memoria y después corregido contra el repo*:

```text
submission-package.md, abierto 2026-10-05
brief de Colosseum (los siete factores, línea de fecha límite, descalificadores)  ruta, fecha
tarjeta del equipo, plan del mes, pitch v0                                        ruta, fecha
memorando de la idea, mapa de competidores, paquete de evidencias, memorando de decisión  ruta, fecha
design partners, registro de tracción                                             ruta, fecha
tarjeta de alcance, guion de demo, recorte en devnet, bitácora narrativa          ruta, fecha
one-pager de GTM, lista de socios                                                 ruta, fecha
deck, ambas exportaciones                                                         ruta, fecha
video de presentación (2 a 3 min), video de demo (3 min o menos)                  enlace, duración cronometrada
respuestas del portal                                                             este archivo, abajo
```

Si una línea no tiene ruta, **ese entregable falta**, y *hoy es el día de terminarlo, no el día de enviar*. Todo lo que sigue asume que cada línea está llena.

2. **Responde el portal de Colosseum**, los nueve campos, desde el archivo y en el orden de la página. Desde la temporada World's Fair 2026 el portal de Colosseum pide nueve cosas, y la página que las lista es colosseum.com/hackathon, la misma que cita tu brief de Colosseum. **Cópialas al archivo del material de entrega** con las palabras de la página antes de responder cualquiera:

```text
campos del portal, colosseum.com/hackathon, leído 2026-09-06
1  product name and brief description (nombre del producto y descripción breve)
2  blockchains and tools used (blockchains y herramientas usadas)
3  all teammates with backgrounds and previous experience (todos los integrantes con su trayectoria y experiencia previa)
4  team location (ubicación del equipo)
5  a product logo or graphic (un logo o gráfico del producto)
6  GitHub repository link, private allowed if access is granted to hackathon@colosseum.com (enlace al repositorio de GitHub, privado permitido si se da acceso a hackathon@colosseum.com)
7  a two-to-three-minute presentation video (un video de presentación de dos a tres minutos)
8  a product-demo video of no more than three minutes (un video de demo del producto de no más de tres minutos)
9  go-to-market strategy, demand validation and distribution plans (estrategia de go-to-market, validación de demanda y planes de distribución)
```

Si la página en vivo muestra otra lista el día que la copies, **gana la página**, y anota qué cambió. Cada campo sale de un entregable que ya construiste. El nombre del producto es la única palabra de producto del pitch final, la que viene después del problema. Las blockchains y herramientas salen de la bitácora narrativa, con los nombres exactos que usa la bitácora y nada que la construcción no haya usado, *porque la revisión del repo lee el código y las dos listas deberían coincidir*. Los integrantes salen de la tarjeta del equipo, una línea por persona con su trayectoria y su experiencia previa, y **ningún asiento vacío**. La ubicación es la ciudad de la tarjeta del equipo.

El logo es el gráfico de título del deck exportado como imagen, y *hoy no se diseña nada*. El enlace al repositorio es el campo que necesita **una segunda persona**: si el repo es privado, un integrante le da acceso a hackathon@colosseum.com y otro lo confirma leyendo la lista de acceso del repo. El video de presentación es el archivo de **2 a 3 minutos** de la lección pasada, el de demo es el de 3 minutos o menos, y los dos van como enlaces con la duración cronometrada escrita al lado.

El último campo son **tres preguntas en una sola caja**. La estrategia de go-to-market es el one-pager. La validación de demanda es el paquete de evidencias, las conversaciones de la semana 1 y **la última fila del registro de tracción** con su fecha. Los planes de distribución son los socios con nombre de la lista de socios, y *un socio que respondió, aunque haya dicho que no, es mejor línea que una categoría de socios*. Para Fiado, la línea de herramientas dice Solana, devnet, la herramienta de agentes y los skills del kit, tal como los nombra la bitácora narrativa, y la descripción y la respuesta de GTM son **los dos campos que quedan en blanco** para que los escribas con la misma forma.

La descripción **se escribe con un formato**, porque una diapositiva pegada no se lee de un vistazo. El skill de hackathon del kit trae uno: un tagline, el problema, lo novedoso en negrita, una parte titulada 'What works today' (lo que funciona hoy) y el por qué Solana, en **200 a 500 palabras**, calificado contra el judging-criteria.md del propio kit. Eso es lo que decía el skill en github.com/solanabr/solana-ai-kit el 2026-09-06. Verifícalo contra el README actual del kit antes de usarlo, *porque un archivo de skill cambia más rápido que una lección*.

El mismo skill elige un track según qué tan lleno está cada uno en el portal en vivo, a través del ext/colosseum del kit con un COLOSSEUM_COPILOT_PAT configurado en el entorno, leído el 2026-09-06, y esa parte depende del README actual más que el formato, así que **léelo fresco ese mismo día**.

```text
descripción, escrita con el formato del skill de hackathon del kit (verificar contra el README actual)
tagline:            el one-liner del deck, cinco palabras o menos, sin jerga
problema:           quién lo tiene y qué le cuesta, del paquete de evidencias
**lo novedoso**     una oración en negrita, el insight del memorando de decisión
What works today:   el recorte, exactamente lo que muestra el video de la demo, nada planeado
por qué Solana:     el párrafo del one-pager de GTM
extensión: 200 a 500 palabras. calificado contra judging-criteria.md en el kit.
```

Ese es el orden en que lee un revisor. 'What works today' es **la línea de la honestidad**, la que la revisión del repo compara contra el código, así que lista exactamente lo que muestra el video de la demo y nada que esté planeado. La tecnología va ahí, en palabras simples, *después de que el lector tenga una razón para quererla*.

![Cada uno de los nueve campos del portal de Colosseum se responde con un entregable del curso, con nombre, desde el pitch final para el nombre hasta el one-pager de GTM y la lista de socios para la última caja.](assets/v01-table.webp)

3. **Convierte la fecha límite con una herramienta**, *nunca de cabeza*, y escribe los dos resultados en el archivo del material de entrega, uno al lado del otro. La cadena para practicar es inventada, pero la forma es la que imprime cada página: una fecha, una T, una hora y una Z que significa UTC.

```bash
python3 -c "from datetime import datetime, timezone; d = datetime.fromisoformat('2026-09-08T02:59:59+00:00'); print(d.astimezone(timezone.utc)); print(d.astimezone())"
```

En una laptop con la hora de Brasília, las dos líneas imprimen 2026-09-08 02:59:59+00:00 y **2026-09-07 23:59:59-03:00**. La página dice el ocho. Tu calendario dice **el siete**, un segundo antes de la medianoche. Un equipo que puso "el 8" en el chat del grupo y planeó enviar la mañana del 8 *ya perdió, con un producto que funciona y un buen video*. El +00:00 del comando es la Z final escrita completa, y en **Python 3.11 o más nuevo** la Z funciona sola, así que en una máquina más vieja quédate con la forma larga.

Para Colosseum, la cadena sale de tu brief de Colosseum. En la lección 2 copiaste la fecha de cierre de la temporada de la página en vivo, y al 2026-09-06 la hora era lo que la página todavía no había impreso, *así que el brief tiene un espacio reservado para ella*. En la última semana, vuelve a leer colosseum.com/hackathon, **pega la hora que imprime la página** en el comando y escribe el resultado local en la línea de la fecha límite, junto al original en UTC.

Después conviértela una segunda vez con otra herramienta, con el reloj mundial de un celular alcanza, y solo cuando las dos coinciden la línea recibe la palabra "verificado" al final. **Escribe el nombre del líder** en esa misma línea, porque la página dice que el líder del equipo debe completar el envío antes de la fecha límite. Mi propia regla, *una preferencia mía y no una regla de la página*, es fijar la fecha límite del equipo **un día entero antes** de la impresa, para que el último día sea para la captura de pantalla y no para el formulario.

![La fecha límite se copia en UTC, se convierte dos veces, se adelanta un día para el equipo, se vuelve a verificar el día anterior, y después el líder envía y la captura de pantalla va a la bitácora.](assets/v02-timeline.webp)

4. **Lee los cuatro descalificadores** y fírmalos. De la página, leída el 2026-09-06, con sus propias palabras para el primero:

> 'Only one product submission is allowed per team, and therefore one per individual, during each hackathon' (solo se permite un envío de producto por equipo, y por lo tanto uno por individuo, durante cada hackathon) (colosseum.com/hackathon, 2026-09-06).

**Un producto por equipo**, y por lo tanto uno por persona. La forma en que los equipos rompen esto sin querer es **la segunda cuenta**: un compañero se registra por su cuenta "por si acaso", o envía un experimento paralelo con su propio nombre, y ahora una misma persona tiene dos envíos en la temporada. Nadie del equipo envía nada más, y *el líder es el único que aprieta el botón*.

Segundo, **el líder del equipo completa el envío** antes de la fecha límite, así que el líder queda nombrado en el archivo, en la línea de la fecha límite donde ya está su nombre. Tercero, la página dice que **tergiversar el historial de desarrollo** puede descalificar, así que el código previo se declara en el envío, en 'What works today' o donde el formulario te dé espacio, copiado de la bitácora narrativa, donde la versión honesta ya vive, con fecha. Cuarto, el acceso. Un repo privado sin acceso otorgado a hackathon@colosseum.com es un repo que el revisor no puede abrir, y mi lectura, *que la página no dice con todas las letras*, es que un repo que nadie puede abrir **se califica como si no hubiera repo**.

```text
lista de descalificadores, colosseum.com/hackathon, leído 2026-09-06
[ ] un producto de este equipo, y nadie en él envía otro                      firma: líder
[ ] código previo e historial de desarrollo declarados en el envío           firma: líder
[ ] el líder completa el envío antes de la fecha límite                      firma: líder
[ ] acceso al repo otorgado a hackathon@colosseum.com, confirmado por un segundo integrante
```

El líder firma tres y un segundo integrante firma la cuarta, y el archivo no es una lista de verificación **hasta que los nombres están puestos**. Una lista que nadie firmó es *una lista de cosas que alguien esperaba que fueran ciertas*.

5. **Envía**, en un orden fijo. **Abre la página en vivo** una última vez y compara su lista de campos con la de tu archivo, *porque esta lección no puede descartar que el portal agregue un campo entre el día que lo copiaste y el día del envío*, y un campo que nunca viste es un campo que queda vacío.

Después el líder entra al portal, pega cada respuesta desde el archivo, **relee cada campo** contra el archivo antes de pasar al siguiente, envía, y guarda la captura de la confirmación en la bitácora narrativa con la fecha. Fuera de temporada, cuando no hay portal abierto, el formulario simulado es **este mismo archivo**: los campos en el orden de la página, nada en blanco, y el nombre del líder y la fecha en la última línea *en lugar de la captura de pantalla*.

![El día del envío el líder compara la lista de campos en vivo, pega y relee cada campo desde el archivo, envía, registra la captura de pantalla, y solo entonces abre un formulario de ensayo.](assets/v03-flowchart.webp)

6. **Pasa el mismo material** por un ensayo si hay uno abierto, *después del formulario de Colosseum o junto con él, nunca en su lugar*. Dos tipos pueden recibirlo: un hackathon de temporada, en su propia plataforma y en una ventana que queda antes de una temporada de Colosseum, y un side track (el track regional), un premio regional unido a la temporada de Colosseum. Cualquiera de los dos va a tener su propia página, con sus propios entregables y su fecha límite, así que **lee esa página el día que decidas entrar** y reutiliza el material que ya tienes. Las ediciones y las fechas se leen de la página en vivo cada vez.

El método cabe en una oración: abre su página, **copia sus entregables y su fecha límite** al archivo del material de entrega con la fecha, responde sus preguntas y pega el resto. **No reconstruyas el material** para ese ensayo y no escribas una segunda descripción. Antes de cualquier segundo envío, lee la regla en la página de ese hackathon y la regla de uno por persona en la página de Colosseum, ese mismo día, y anota en el archivo qué regla leíste y dónde. Si las dos reglas no se pueden cumplir a la vez, **sáltate el ensayo** y *escribe por qué*.

![Antes de un segundo envío el equipo lee la regla de ese hackathon y la regla de uno por persona de Colosseum, envía solo si las dos se cumplen, y si no se salta el ensayo.](assets/v04-flowchart.webp)

## Está listo cuando

- **Ningún campo del portal está vacío**, y cada respuesta en submission-package.md apunta a un entregable que construiste.
- La **línea de la fecha límite** muestra UTC y hora local, coincidentes en dos herramientas, marcada "verificado" y con el nombre del líder.
- La **lista de descalificadores** está firmada, tres casillas por el líder y la cuarta por un segundo integrante.
- Una **confirmación**, captura de pantalla real o línea del formulario simulado, está en la bitácora narrativa con la fecha.
- Si había un formulario de ensayo abierto, sus **entregables, fecha límite y regla** también están en el archivo del material de entrega.

## Ojo con

- **Convertir la fecha límite una sola vez** y confiar en eso. Una conversión que hizo un compañero hace un mes, y que desde entonces cambió de zona horaria, es la línea con más probabilidad de estar mal, *así que vuelve a verificarla el día anterior*.
- Una descripción que **empieza por la tecnología**. "Un programa de Solana que usa PDAs para almacenar..." no le dice nada a un revisor que todavía no conoce el problema.
- **Entrar a un ensayo por reflejo**, que pasa cuando sus reglas viven en un chat y no en el archivo, *así que copia la página, no el resumen*.
- Lo que se sacrifica: un ensayo **cuesta un segundo formulario** y puede chocar con la regla de un producto por persona si es en sí mismo un track de Colosseum, *así que mejor saltarse un ensayo que romper una regla*.

## Lo que queda

Las reglas están en la misma página que los criterios de evaluación, unos párrafos más abajo, y son **la parte que un equipo controla por completo**: cada campo llenado desde un archivo que ya existe, una fecha límite convertida con dos herramientas y escrita en las dos zonas, un producto por persona, y el líder apretando el botón con un día de margen. *El video de presentación es el campo que un jurado abre primero, y la regla de uno por persona es la que descalifica a un equipo al que nadie avisó.*

## Siguiente

El capstone termina cuando la confirmación está en la bitácora, el último entregable que la promesa del curso pedía. La primera acción de la próxima lección es **un archivo llamado next-season-plan.md** junto a este material, con dos oraciones copiadas de la página en vivo sobre qué pasa cuando cierran los envíos, la entrevista de 15 minutos a la que invitan a un grupo más pequeño y el anuncio, más o menos un mes después de la fecha límite. Desde ahí recorre *por qué un pitch perdido no es un proyecto perdido*.

Terminas el mes con **un material de entrega que un jurado puede abrir**, un formulario que nadie improvisó en el portal y una captura de pantalla con fecha. Guarda la captura, porque dentro de un mes, diga lo que diga la página de ganadores, *es la prueba de que el formulario no te ganó*.
