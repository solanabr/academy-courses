# El equipo en el día 0

En la lección pasada escribiste el brief de Colosseum: siete factores citados de la página en vivo, cuatro preguntas mapeadas sobre ellos, la fecha límite convertida a tu zona horaria. *Déjalo abierto.* Su bloque de descalificación recibe **el nombre de una persona** al lado antes de que termines.

## Por qué importa

El mes que pierde tiene una forma que he visto muchas veces: el mes se va en código. La presentación se planea cerca del final, sin mes que quede para mejorarla, y el proyecto se mostró a muy poca gente fuera del equipo, así que el feedback que habría arreglado el pitch llega demasiado tarde. *El arreglo no es un deck mejor en la semana 4.* Es una oración en el día 0 y una persona cuyo trabajo es seguir reescribiéndola.

La única idea: el mes se **planea hacia atrás desde el paquete**, y el pitch es un artefacto desde el día 0. Abre un archivo nuevo llamado **team-card.md** y pon la fecha de hoy en la primera línea. Vence *antes de que arranque el reloj*, sea cual sea tu temporada.

## Haz esto

1. **Crea el repositorio del toolkit**: haz una carpeta con el nombre de tu proyecto e inicializa git adentro.

```bash
mkdir <your-project> && cd <your-project>
git init
```

**Guarda team-card.md en su raíz y haz commit** antes de cerrar el día, *y cada artefacto que el curso produzca de aquí en adelante vive en este repositorio*.

2. **Instala el kit**. El curso corre sobre Claude Code con el plugin Solana AI Kit de solanabr/solana-ai-kit, y el comando de instalación es el que está en el README del kit, leído el día que lo ejecutes. Leída el 2026-09-07, la carpeta plugin/skills del kit traía **tres skills**, hackathon, idea-sprint y pitch-deck, en el commit 353e9a1 del 2026-08-20. *El kit cambia entre ediciones, así que el README gana sobre esta lección.* Abre Claude Code dentro del repositorio y pídele que **liste los skills del kit**. Si vuelven esos tres nombres, el kit está adentro. Si no, el README es la siguiente página.

3. **Fondea una billetera de devnet**. Crea una con cualquier app de billetera o con la CLI de Solana, cámbiala a devnet y fondea su SOL desde el faucet de devnet al que apunta el README del kit. El USDC de devnet viene de la fuente que lista el README del kit, leída el día que la fondees, a la fecha del 2026-09-07. Lee **dos saldos** en devnet, SOL para las comisiones y USDC de devnet para el fiado. Si alguno marca cero, arréglalo ahora. *El mes no tiene espacio para eso después.*

4. **Pon cuatro roles sobre nombres** en team-card.md, un nombre junto a cada trabajo. El builder entrega la porción. El responsable de competidores averigua qué hacen bien y qué hacen mal los rivales, por escrito para el **final de la semana 1**. El responsable de alianzas le vende la idea a proyectos adyacentes antes del demo day, *para que el equipo llegue con algunos socios ya de su lado*. El storyteller es dueño de la oración del pitch y del registro de feedback y **reescribe la oración cada semana** hasta que sea la primera línea del deck.

Un equipo de dos igual nombra cuatro trabajos, *porque las cuatro preguntas de tu brief no se encogen cuando el equipo se encoge*. Asigna primero los roles que **solo una persona puede hacer**, luego los dobles, y escribe las semanas en que un rol doblado va a quedar desatendido. Solo, eres dueño de los cuatro, y *la línea honesta es cuáles dos reciben una hora por semana*. Los roles de Fiado, con el desatendido marcado:

![La tarjeta de dos personas de Fiado nombra un responsable para los cuatro roles, dobla dos por persona y marca alianzas como desatendido en las semanas 2 y 3.](assets/v01-table.webp)

Luego los campos del portal. A la fecha de la temporada World's Fair 2026, la entrega de Colosseum pide cada integrante con su trayectoria y experiencia previa, y la ubicación del equipo. Cada integrante tiene que **crear una cuenta**. El líder del equipo agrega a los demás durante la entrega y debe completarla **antes de la fecha límite**. *La participación individual está permitida.* Así que la tarjeta lleva una línea de trayectoria por persona, la ubicación y **la palabra líder** junto a un nombre, copiada hoy al bloque de descalificación de tu brief.

Un hackathon de temporada o un side track va a tener su propia página con sus propios entregables y su fecha límite. **Lee esa página el día que decidas entrar**, y reutiliza el paquete que ya tienes. La tarjeta completa de Fiado:

```text
team-card.md, Fiado, día 0, escrita 2026-09-06
pitch v0: Una dueña de tienda que pierde la libreta pierde cuarenta deudas pequeñas,
          así que Fiado guarda el fiado on-chain y recuerda a cada cliente.

rol                          quién   doblado con                   nota
builder                      Ana     responsable de alianzas       entrega la porción en devnet, semanas 2 y 3
responsable de competidores  Bruno   storyteller                   mapa por escrito para el 2026-09-20
responsable de alianzas      Ana     builder                       DESATENDIDO semanas 2 y 3; llamadas en la semana 1 y del 2026-10-05 al 10-07
storyteller                  Bruno   responsable de competidores   es dueño del pitch v0 y del registro de feedback, reescribe cada semana

líder:       Bruno (completa la entrega antes de la fecha límite del brief)
ubicación:   <ciudad, país, como lo pide el portal>
trayectoria: Ana, desarrolladora web, tres años entregando herramientas para pequeños negocios, creció en una tienda de barrio
             Bruno, escribe para vivir, primer hackathon
```

5. **Planea el mes hacia atrás**, debajo de la tarjeta. **Escribe la última fecha primero**: la fecha de cierre de la temporada en tu brief, con la hora releída en la página en vivo y convertida. A la fecha de la página del 2026-09-06, el hackathon World's Fair corre del **14 de septiembre al 12 de octubre de 2026**, que son 28 días, cuatro semanas exactas. Desde la fecha límite camina hacia atrás una semana de historia, dos semanas de porción, una semana de evidencia, y antes de eso, ahora. Deriva los límites desde la fecha de inicio *en vez de tipearlos*:

```bash
python3 -c "from datetime import date, timedelta; s = date(2026, 9, 14); print(*[s + timedelta(days=7*i) for i in range(5)], sep='\n')"
```

La primera línea impresa es el día en que arranca el reloj, la última es el día de la fecha límite, y las tres del medio son **los límites de las fases**. Si la página en vivo mueve una fecha, *cambia la única fecha dentro del comando y cada línea debajo se mueve también*. **Pon las cinco fechas** sobre cinco fases, nombra los artefactos con los que termina cada fase, y deja que la línea de la fecha límite apunte al brief para la hora. En la fila de antes del reloj, agrega **tres líneas con fecha** para los pasos 1 a 3. Las de Fiado: repositorio inicializado el 2026-09-06, kit instalado el mismo día, la billetera de devnet de Ana con SOL y USDC de devnet, verificada el 2026-09-06. Su plan es la línea de tiempo de abajo, cinco líneas.

![El mes de World's Fair corre desde antes del reloj hasta la semana 1, las semanas 2 y 3, la semana 4 y la entrega el 2026-10-12, cada fase con fecha y produciendo artefactos con nombre.](assets/v02-timeline.webp)

6. **Escribe el pitch v0** debajo de la fecha. Una oración, el problema antes del producto, de menos de **25 palabras**: una persona primero, luego lo que esa persona no puede hacer hoy, luego tu producto. La versión producto-primero de Fiado es "Fiado es una cuenta de fiado on-chain para tiendas de barrio con liquidación en stablecoin y recordatorios automáticos", dieciocho palabras, *todas verdaderas, y nadie adentro*. Su v0 al inicio de la tarjeta tiene 24 palabras, y las primeras doce son una dueña de tienda, una libreta perdida y cuarenta deudas pequeñas.

**Cuenta las palabras**. Si la primera palabra es tu producto, deja esa línea como registro del día 0 y escribe una segunda debajo que empiece con una persona. *Va a ser una mala oración.* Existe, y ese es todo el requisito para un v0.

7. **Fija la cadencia de feedback**. Un registro de feedback es un archivo con una fecha, un nombre y lo que esa persona no entendió. Muéstrale la oración a **tres personas por semana** que no estén en la tarjeta, y anota, con sus palabras, la parte que no entendieron. *Lo que les gustó no va a ningún lado.* Una respuesta de "qué bueno" no es una entrada del registro, así que pregúntales de qué trata la oración y anota lo que te respondan. Pon los tres nombres de esta semana **hoy**, aunque las filas queden vacías. Los tres de Fiado están a tres distancias del problema:

```text
feedback-log.md
fecha        nombre, fuera del equipo                                     lo que no entendió, con sus palabras
2026-09-07   Lucia, la dueña de la tienda, tía de Ana                     (oración mostrada esta semana)
2026-09-07   Jorge, un cliente habitual que tiene un fiado                (oración mostrada esta semana)
2026-09-08   Marcos, entró a un hackathon una vez, nunca manejó un fiado  (oración mostrada esta semana)
```

**Manda tu oración** a tus tres hoy. Las tres reacciones de Fiado van a estar en el registro para el **2026-09-13**, *y por eso el v1 vence al final de la semana 1*.

![Un archivo del día 0 se arma copiando la fecha límite y el líder del brief, derivando las fechas de las fases, nombrando cuatro roles, escribiendo el pitch v0 y registrando tres reacciones de afuera.](assets/v03-flowchart.webp)

## Está listo cuando

- **Cuatro roles** tienen responsable con nombre, y el nombre del líder está en el bloque de descalificación de tu brief.
- Cada fase lleva **una fecha que imprimió el comando**, y la línea de la fecha límite apunta al brief para la hora.
- El pitch v0 tiene **menos de 25 palabras** con la persona antes del producto.
- El registro de feedback tiene **tres entradas con fecha** con las palabras de la propia gente.

## Ojo con

- **Un plan sin fechas** es una lista de buenas intenciones, y la fecha límite se publica en UTC, así que la línea convertida de la lección pasada va en el plan.
- Una oración que **empieza con el producto** le pide a quien escucha que le importe un nombre que nunca oyó.
- Un equipo de dos donde el responsable de competidores es también el único builder, y **nadie lo dice**, termina con un mapa flaco en la semana 1. Dilo en la tarjeta y pon fechas alrededor.

El storyteller **te cuesta un builder**, un cuarto de las horas de construcción en un equipo de cuatro, *y es el intercambio correcto*, porque el código aparece en muy pocas de las siete líneas de tu brief y el trabajo del storyteller en la mayoría de las otras.

## Lo que queda

El mes se planea hacia atrás desde el paquete, así que **la fecha límite es la primera fecha del plan** y cada fase se deriva de ella. Cuatro roles tienen responsable con nombre incluso en un equipo de dos, porque las cuatro preguntas de tu brief no se encogen cuando el equipo se encoge. *El pitch existe desde el día 0 como una oración que nombra a una persona antes que al producto, y tres personas fuera del equipo ya te dijeron qué no entendieron.*

## Siguiente

La semana 1 arranca en la próxima lección, y lo primero que haces es **tirar a la basura la idea** con la que llegaste, *o probar que merece quedarse*. La pones junto a otros dos problemas que tú, o gente cerca de ti, de verdad tienen, y puntúas los tres hasta que uno sobreviva. La lección 4 abre Claude Code dentro de este repositorio, y la oración de tu archivo es la primera candidata.
