# Hazlo real, y después di qué cambió

La lección pasada enviaste la porción de siete pasos a devnet con un equipo de agentes, y la última línea de tu bitácora es la firma de la transacción que un jurado puede abrir. Deja la porción corriendo en una segunda pantalla, porque **hoy no se construye nada nuevo**, con una pequeña excepción para las tiendas que empiezan a usarla de verdad. Hoy se la mira, tú y gente que no estuvo en la construcción, y *lo que ven se escribe*.

## Cuál de las dos es la tuya

Dos demos, la misma función. Una tiene un frontend que cualquier jurado que haya usado una herramienta de agentes reconoce en un segundo: el mismo layout, el mismo degradado, el mismo texto de relleno. La otra parece hecha por alguien a quien le importaba la persona del otro lado de la pantalla. Las demos que quedaron entre los premiados **no parecían hechas por IA**, y eran lo bastante funcionales para una demo, y *nada en esa oración dice bonita ni completa*. **Averigua cuál es la tuya** antes de seguir leyendo.

## Haz esto

1. **Lleva la bitácora narrativa**, empezando con una captura de antes. Abre la porción en la pantalla donde la demo pasa más tiempo, para Fiado la pantalla del fiado. Tómale una captura *exactamente como está*, pégala en la bitácora bajo la fecha de hoy, y escribe una línea al lado: antes de la pasada. Después lee en voz alta cada palabra de esa pantalla, cuenta las que nombran a una persona o una tienda, y **escribe la cuenta** junto a la captura.

```text
bitácora narrativa, Fiado, día 15
captura:     tab-screen-day15.png (antes de la pasada)
palabras en pantalla que nombran a una persona o una tienda: 0
lo que la pantalla dice en su lugar: Dashboard, Welcome back, Total Balance
```

La cuenta es **casi siempre cero**, porque eso es lo que entrega un subagente de frontend *cuando nadie le dijo quién es el usuario*. Una bitácora narrativa es un registro fechado de la construcción, **llevado mientras la construcción ocurre**: una captura cada vez que una pantalla cambió, con la vieja dejada arriba, la firma de cualquier transacción que la demo vaya a mostrar, con su enlace, una decisión cada vez que algo se cortó, en las palabras que el equipo usó en ese momento, y una vez por semana el guion del video de actualización. El deck y los dos videos de la semana 4 se **cortan de este archivo**.

La actualización semanal de Colosseum es opcional y muy recomendada, y su forma es **un video de un minuto**, según la página del hackathon leída el 2026-09-06. El guion son **cuatro oraciones**: qué cambió esta semana, *mostrado y no descrito*, por qué cambió, con la decisión de la bitácora, qué vería hoy un jurado si clonara el repo, y qué viene la próxima semana. **Menos de 60 segundos**, cámara del celular o grabación de pantalla, una toma. Una segunda toma está bien. Una tercera toma es la trampa del pulido. El guion de la semana 2 de Fiado, como entró en la bitácora el día 14:

```text
video de actualización, semana 2, Fiado, grabado el día 14, 52 segundos
0:00  Esta semana el fiado pasó de un mock a devnet. Esta es la transacción donde
      un cliente habitual paga 15 de un fiado de 40, y este es el mismo fiado marcando 25
      en el celular de la dueña.
0:18  Cortamos el registro de clientes. Dos dueñas nos dijeron en la semana 1 que el fiado
      vive en su celular, así que solo la dueña abre fiados y el cliente solo ve un
      saldo.
0:35  Si clonas el repo hoy, el quickstart corre los siete pasos y el primer
      pago parcial confirma en devnet.
0:46  La próxima semana la pantalla deja de decir Dashboard, y una página real de una
      libreta real va ahí.
```

La línea de 0:18, *una decisión con la evidencia de la semana 1 detrás*, es la oración que un jurado que pregunta **cómo prioriza el equipo** quiere oír.

![La bitácora de Fiado gana una entrada cada día que algo cambió entre el día 10 y el día 21, con las capturas de antes y después separadas por un día y un guion de video cada semana.](assets/v01-timeline.webp)

2. **Corre la pasada "¿parece real?"** sobre esa pantalla. Cuatro ítems, cada uno preguntando si una persona que tiene este problema *creería que la pantalla se hizo para ella*. **El texto nombra al usuario**: el nombre de la dueña en el encabezado, o al menos tu tienda, y el nombre de pila del cliente en cada fila, no Dashboard ni Welcome back. **La demo muestra datos reales**: una página real de una libreta real, nombres de pila y montos, con permiso de la dueña, porque desde el momento en que aparecen Customer 1 a Customer 4 con números redondos *el resto de la demo se puntúa como un mock*.

**Sin el look de componente por defecto**: la grilla de tarjetas, el encabezado con degradado y las tres fichas redondeadas de estadísticas salen, y el único número que le importa a la dueña va donde su ojo cae primero, una fuente, un color de acento, el total adeudado grande, unos 30 minutos. **El estado vacío está resuelto**: abre la app como una tienda nueva sin fiados, y si lo que aparece es una tabla con encabezados y nada debajo, *la demo tiene un hueco*, porque esa es la primera pantalla que va a ver un jurado que clone el repo. **No hay quinto ítem** sobre gusto.

![Cada ítem de la pasada nombra lo que entrega el frontend por defecto, lo que un jurado lee de eso, el arreglo, y un costo de minutos para tres ítems y una hora para los datos reales.](assets/v02-table.webp)

La pantalla del día 15 de Fiado era la normal, tres fichas de estadísticas todas en cero porque leían otra tabla, y si la tuya se ve así, *eso es lo que la herramienta entrega y no un fracaso*. El checklist para ella, con **tres ítems hechos**:

```text
pasada ¿parece real?, Fiado, pantalla del fiado, día 16
el texto nombra al usuario   cambió: encabezado "Dashboard" -> "Tienda de Lucia"; saludo
                             quitado; cada fila lleva el nombre de pila del cliente
datos reales en la demo      cambió: filas Customer 1 a 4 -> seis clientes habituales de la
                             página de la libreta de Lucia, nombres de pila y montos, con su permiso
sin look por defecto         cambió: tres fichas de estadísticas -> un número, el total adeudado,
                             grande arriba; grilla de tarjetas quitada; un color de acento
estado vacío resuelto        (te toca terminarlo)
```

Lucia es la dueña de la tienda de las conversaciones de la semana 1, la tía de Ana, ya en el registro de feedback desde el día 0 y la ronda 1, y la bitácora **registra su permiso con la fecha**, *porque un jurado puede preguntar*. **Ahora el cuarto ítem**: abre Fiado como una tienda sin fiados, como lo vería un jurado que acaba de correr el quickstart, toma la captura, y escribe el estado vacío en palabras de Lucia, algo como todavía no hay fiados, abre el primero desde la libreta, con el botón que lo hace justo debajo de la oración. **Marca el ítem con lo que cambió**, pega la captura de después bajo la de antes, y ponle fecha.

3. **Haz el roast de la porción**, primero el comando del kit y después una persona de afuera de la construcción. Un roast es una revisión por alguien de afuera de la construcción, con la instrucción de buscar *qué está mal y no qué está bien*. El kit trae un comando /product-review entre sus 30 comandos, según el repositorio solanabr/solana-ai-kit leído el 2026-09-06, y ese comando es **el primer roaster**. **Verifica el nombre del comando** contra el README actual del kit antes de correrlo, *porque el kit sigue su rama main y los nombres se mueven*. Apúntalo al repo y al guion de la demo.

**El segundo roaster** es una persona que no construyó la cosa, con la misma instrucción: qué está mal, qué confunde, qué no creerías. **Registra tres líneas por roaster**, *en las palabras del roaster*. Las de Fiado:

```text
notas del roast, Fiado, día 17
persona   el cliente no puede saber desde la pantalla qué pasa cuando el fiado se
          paga completo
persona   el total adeudado de arriba no tiene una fecha al lado
persona   la palabra devnet aparece en el pie donde una dueña de tienda la leería
          sin saber qué significa
comando   el quickstart asume una billetera con fondos y nunca lo dice
```

Dos de las cuatro se volvieron decisiones en la bitácora **esa misma tarde**. La tercera, la fecha junto al total, esperó a la ronda 2 de feedback para confirmar *que alguien más también la quería*.

4. **Corre la pasada de sanidad de seguridad**, en la forma que aplique, y la revisión a mano en cualquier caso. Donde la porción tiene un programa propio, el comando /audit-solana del kit lo lee, de la misma lista de 30 comandos con la misma nota de vigencia que el comando del roast. Donde no hay programa, y una porción recortada a siete pasos muchas veces no lo tiene, la pasada es **una revisión de firmas y manejo de llaves** que una persona hace a mano, *porque la demo firma algo y alguien tiene la llave que lo firma*.

**Cinco preguntas**, con las respuestas a la bitácora tal cual. Dónde vive la llave que firma la transacción de la demo, un archivo en el repo, una variable de entorno o una billetera en el navegador. Está esa llave, o cualquier otra, **en el historial de git**, *lo que una búsqueda de los formatos habituales de llave responde en un minuto*. Envía el bundle del frontend algo secreto. Firma la demo con la billetera de la dueña o con una llave del equipo que la reemplaza, y dice el guion cuál. Está la llave de devnet separada de cualquier llave que tenga valor en mainnet, en otra máquina o al menos en otro archivo que **nunca esté en la laptop de la demo**.

*El código construido por agentes puede verse correcto y aun así traer un hueco de seguridad*, y encontrarlo es un costo que paga el humano, no la herramienta, **cerca de una hora el día 17**. Las respuestas de Fiado: la demo firma con un keypair de devnet en una variable de entorno que estuvo, el día 13, brevemente en un archivo .env commiteado. Se quitó, **la llave se rotó**, y la bitácora lo dice con la fecha, *porque un jurado que lo encuentra en el historial sin una nota es peor que un jurado que encuentra la nota*.

![Una porción con un programa corre el comando de auditoría del kit, una porción sin programa responde a mano cinco preguntas de manejo de llaves, y ambas registran sus hallazgos el día 17.](assets/v03-flowchart.webp)

5. **Pon la porción en manos reales.** La porción pulida va a los design partners (clientes socios) de la semana 1 **esta semana**, no después de la fecha límite, *porque Traction (tracción) se cuenta en gente usando el producto y la cuenta necesita días para crecer*. El builder configura cada tienda socia a mano, porque el registro sigue siendo un no-objetivo. El fiado es real: la dueña abre fiados para sus clientes habituales reales, agrega compras reales, y los clientes abren su saldo en sus propios celulares.

   **La liquidación se queda en devnet**, así que un cliente real paga como siempre pagó, en efectivo en el mostrador, y la dueña lo marca como pagado. Ese botón es **lo único que se construye hoy**, porque una tienda no puede llevar un fiado que no puede bajar, y la transferencia en devnet sigue siendo el camino de la demo hasta que el producto pase a mainnet. Ahora hay nombres y teléfonos reales en tu backend, *así que la revisión de manejo de llaves del paso 4 cubre también esa base de datos*.

   Después **empieza traction-log.md** y llénalo cada día desde los propios registros de la app, nunca desde lo que un socio dijo por teléfono. Cinco columnas: tiendas invitadas, tiendas con al menos un fiado real, fiados abiertos, clientes que abrieron su enlace de saldo, y tiendas que volvieron otro día sin que se les pidiera. La última es **el uso recurrente**, el número en el que un jurado más confía, *porque cualquiera prueba una cosa una vez por cortesía*.

```text
traction-log.md, Fiado, semana 3 (contado desde los registros de la app)
día  tiendas invitadas  tiendas con fiado real  fiados abiertos  clientes abrieron enlace  volvieron sin pedirlo
16   4                  2                       9                4                         n/a
18   6                  3                       21               11                        1 de 2
21   7                  4                       34               19                        3 de 4
```

   Cuatro tiendas de siete lo usaron y **tres volvieron** otro día sin un recordatorio del equipo. Esa línea alimenta el pitch v2, la diapositiva de demo del deck y la respuesta de validación de demanda del portal, y el registro sigue creciendo durante la semana 4. La farmacia paró después de dos fiados, así que recibe una llamada y una fila en el registro de feedback, *porque la razón por la que un usuario real dejó de usarlo es la oración más útil que produce el mes*.

6. **Reescribe el pitch como v2**, desde lo que la porción hace. El pitch v1 se escribió el día 7 a partir de cinco conversaciones, antes de que existiera código, *así que solo podía decir lo que el equipo esperaba*. La regla para la v2 es que **cada verbo de la mitad del producto** nombra algo que la demo muestra en pantalla.

```text
pitch v1 (día 7):   Una dueña de tienda que pierde la libreta pierde cuarenta deudas
                    pequeñas, así que Fiado guarda el fiado en su teléfono, le muestra
                    a cada cliente el mismo saldo y lo liquida en una stablecoin.
pitch v2 (día 18):  Una dueña de tienda que pierde la libreta pierde cuarenta deudas
                    pequeñas, así que Fiado abre el fiado en su teléfono, le muestra
                    al cliente el mismo saldo, cobra parte en USDC y le manda el
                    recordatorio al cliente por ella.
cambió:             "guarda el fiado" -> "abre el fiado" (la primera pantalla que
                    muestra la demo; un formulario, no la transacción); "lo liquida
                    en una stablecoin" -> "cobra parte en USDC" (la única transacción
                    en devnet, lo que el cliente hace en pantalla); el recordatorio
                    vuelve, porque la porción dispara uno; "a cada cliente" -> "al
                    cliente", uno a la vez en pantalla
```

La mitad del problema **no se volvió a mover**, y después de dos rondas de evidencia y una construcción *probablemente ese es el problema correcto*. El recordatorio vuelve porque la porción lo dispara, y solo eso: si reduce los pagos atrasados es **todavía el supuesto sin probar** del memorando de decisión, así que la v2 afirma que el recordatorio se manda, que es lo que la demo muestra.

7. **Corre la ronda 2 de feedback** con las mismas tres personas de la ronda 1, *esta vez viendo la demo en lugar de leer una oración*. El storyteller la corre y el registro recibe **la forma del día 0**, qué no entendió cada persona y qué cambió. El de Fiado:

```text
registro de feedback, ronda 2, día 18
1  Marcos, entró a un         vio la demo; preguntó qué pasa cuando el cliente
   hackathon una vez          paga todo el fiado, si se cierra
                              cambió: nada en la oración; una decisión en la bitácora
                              de que el fiado se cierra en cero y se reabre con la
                              siguiente entrada; una línea para el estado vacío
2  Lucia, la dueña de la      vio su propia página en la pantalla; preguntó a quién
   tienda                     le llega el recordatorio, a ella o al cliente
                              cambió: la oración. el borrador decía "manda el
                              recordatorio"; la v2 dice que el cliente "recibe el
                              recordatorio"
3  Jorge, un cliente          vio la demo; preguntó cómo revisaría el pago él
   habitual con fiado         mismo, sin confiar en la app
                              cambió: nada en la oración; el enlace al explorer
                              va en la pantalla junto al total, así que la demo
                              muestra la firma sin salir de la app
```

Tres entradas, un cambio a nivel de palabra en la oración, **dos cambios en la construcción**. La tercera entrada es **la que hay que copiar**: la firma que registraste la lección pasada estaba en un archivo, y después de la ronda 2 está en la pantalla, *a un toque del número que la dueña mira*.

## Está listo cuando

El artefacto es **la bitácora narrativa** con seis cosas adentro, y está lista cuando:

- Cada ítem del checklist lleva qué cambió, en palabras, con **la captura de después** fechada bajo la de antes. Un ítem que ya estaba bien también queda escrito, *con el porqué*.
- Existe **un video de actualización de menos de 60 segundos**, su guion está en la bitácora, y una persona que no ha visto la porción *puede repetir qué cambió esta semana*.
- **Las notas del roast de dos roasters** y las notas de seguridad, revisión a mano incluida, están en la bitácora.
- **El pitch v2** nombra algo que la porción realmente hace: cada verbo de la mitad del producto apunta al paso de la demo que lo muestra, y un verbo que no apunta a nada sale *hasta que la porción lo sostenga*.
- La ronda 2 de feedback tiene **tres entradas en la forma del día 0**.
- **traction-log.md** tiene una fila por cada día desde que empezó el primer socio, contada desde los registros de la app, con el uso recurrente en su propia columna.

![La pasada de la semana 3 va desde una captura de antes con fecha, pasando por el checklist, un video de actualización, un roast, una pasada de seguridad y el pitch v2, hasta una ronda de feedback con tres personas.](assets/v04-flowchart.webp)

## Ojo con

- **Capturas tomadas solo al final**. Una bitácora escrita el último día *solo tiene el último día adentro*.
- **Un roast hecho por el propio equipo**. El equipo construyó la cosa y ya no puede verla, *como dejas de ver una errata en una página que leíste diez veces*.
- **Saltarse la revisión de manejo de llaves** porque no hay programa. Sin programa no significa sin llaves.
- **Contar un fiado que un socio prometió** en lugar de uno que la app registró. Una promesa no es uso.

**El tiempo de pulido es tiempo de construcción**, así que la pasada es un checklist y no un rediseño: un checklist tiene cuatro ítems y un costo fijo, unas dos horas en total, mientras que un rediseño encuentra un quinto ítem y un sexto, una navegación nueva y un sistema de color, y sigue abierto el día 21 cuando debería grabarse el video de actualización de la semana 3, así que ese equipo llega a la semana 4 con una pantalla más linda y sin video. **La misma disyuntiva** vive dentro del video: la primera toma suele estar bien, la segunda suele ser mejor, y yo diría que *la tercera es donde la mayoría de los equipos empieza a perder la semana*, llámalo una corazonada.

## Lo que queda

El pulido es un checklist con cuatro ítems y un costo fijo, no un rediseño, y la bitácora narrativa se escribe los días en que las cosas cambian. **La tracción empieza esta semana**, en tiendas reales con una liquidación en devnet, *y el registro de tracción cuenta uso, nunca promesas*. Un desconocido que vea tu actualización de 60 segundos debería poder *decir qué cambió esta semana*, y si en su lugar describiría el producto, el minuto fue un pitch, y **la diferencia está en la primera oración**.

## Próxima lección: cómo encuentra alguien esto

La semana 4 empieza la próxima lección con la pregunta que todo jurado hace y la mayoría de los equipos esquiva: cómo encuentra alguien esto, y por qué Solana. Lo primero que escribes son **los primeros 100 usuarios**, como un grupo con un nombre y un canal que los alcanza, *antes de cualquier discusión sobre la cadena*. **Trae la bitácora**, porque el párrafo de por qué Solana se escribe desde lo que la porción hace, y la porción está ahí con su firma en la pantalla.
