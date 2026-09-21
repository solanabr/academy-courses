# Recorta el alcance a la demo de tres minutos

La lección pasada reconstruiste un brief de Colosseum desde la página archivada de Frontier contra reloj, escribiste los minutos al inicio de colosseum-brief-2.md y repuntuaste el paquete de evidencias de Fiado de 0 a 10 sobre factores que copiaste en lugar de recordar. Funciona **sacó un 3**, sobre un plan, porque no había nada construido. Hoy ese número empieza a moverse, *y lo primero que construyes sigue sin ser código*.

## Por qué la demo va antes que el código

Los proyectos que quedaron entre los premiados no estaban 100 por ciento completos. Estaban **lo bastante completos para hacer una demo**, y la demo se había decidido antes que el código. Escribes los tres minutos que un jurado va a ver, *y después construyes solo lo que esos tres minutos muestran*.

Tres minutos es el número de la página. Al momento de la temporada World's Fair de 2026, Colosseum pide un video de demo del producto de **no más de tres minutos** que explique cómo funciona el producto, y un video de presentación aparte de dos a tres minutos que llama 'one of the first resources judges review' (uno de los primeros recursos que revisan los jurados). Tres minutos, *con una transacción real adentro*.

## Haz esto

1. **Crea demo-script.md** junto al paquete de evidencias y ponle cuatro líneas, una por marca, con qué hay en pantalla en cada una. Nada más, y sin editor de código abierto:

```text
demo-script.md, v0, <fecha de hoy>
0:00  en pantalla:
0:30  en pantalla:
1:30  en pantalla:
2:45  en pantalla:
la única transacción:
```

Dale **diez minutos**. Si una marca se queda en blanco, déjala en blanco. *Una marca en blanco es información sobre tu alcance*, y es la razón por la que este archivo existe antes que el repo. La skill hackathon del kit pone la vara a la que apunta el archivo en su checklist: **una demo de menos de tres minutos** con una transacción real, un repo público con un quickstart que funcione, un enlace a devnet y un program ID, y un deck si el hackathon lo exige. Verifica esa lista contra el README actual del kit, *porque los checklists se mueven*.

2. **Elige el único trabajo** con dos preguntas: ¿puede un jurado verificarlo en pantalla en tres minutos, y qué supuesto de tu paquete de evidencias pone a prueba? La mayoría de las ideas empiezan demasiado amplias, y mi posición desde la hoja de trabajo del MVP es *elegir un trabajo para validar primero*. Un MVP es una porción de **4 a 6 semanas**, y un hackathon te da dos, así que tu porción es más chica que la que la hoja de trabajo tenía en mente.

Para Fiado el trabajo es que un cliente habitual **paga parte de su fiado** desde su propio celular, porque abrir un fiado es un formulario y *un jurado no puede distinguir un formulario que escribe en una cadena de uno que no escribe en nada*, y el recordatorio todavía carga la nota del memorando de decisión "necesita un fiado". El registro de proveedores y los sellos de lealtad del memorando de la idea no ponen a prueba nada de lo que el pitch afirma.

![De los cinco trabajos candidatos de Fiado solo pagar parte de un fiado se confirma en pantalla y a la vez pone a prueba una afirmación del paquete de evidencias, con el recordatorio agregado al final como instrumento para el siguiente supuesto.](assets/v01-diagram.webp)

3. **Escribe el camino feliz** como lo recorrería un usuario, con la tienda abierta y la dueña detrás del mostrador, y no edites mientras escribes. Después cuéntalo. Si el camino feliz pasa de **siete pasos** te estás pasando de alcance, y el arreglo es cortar pasos enteros a la lista de no-objetivos hasta que quepa, *nunca fundir tres pasos en uno para que la cuenta salga bien*.

El primer intento de Fiado salió en **once**. Instalar, cuenta, perfil de la tienda y conexión de billetera se fueron a los no-objetivos como pasos enteros, el cliente habitual instalando una billetera se fue con ellos, y la fecha de vencimiento y el recordatorio volvieron como pasos 6 y 7, el instrumento para la afirmación que nadie pudo probar en la semana 1. Eso da siete, *sin fundir nada*.

![El primer intento de once pasos de Fiado pierde cinco pasos enteros de configuración hacia la lista de no-objetivos y gana la fecha de vencimiento y el recordatorio, quedando en siete pasos sin fundir nada.](assets/v02-flowchart.webp)

4. **Escribe la tarjeta de alcance** en una página con cuatro campos: el único trabajo, el camino feliz, la única transacción y los no-objetivos. Nombra los no-objetivos en voz alta, *porque un no-objetivo que vive en la cabeza de alguien se construye en la semana 3*. La tarjeta de Fiado, como va en el repo junto al paquete de evidencias:

```text
scope-card.md, Fiado, semanas 2 y 3, <fecha>

único trabajo:   un cliente habitual paga parte de su fiado desde su propio celular

camino feliz:
  1  la dueña abre Fiado en su celular y abre un fiado para un cliente habitual:
     su nombre de pila y su número de teléfono
  2  ella agrega la compra de hoy, 40, y el fiado marca 40 adeudados; un enlace va a su teléfono
  3  el cliente abre el enlace y ve los mismos 40 en su celular, sin instalar nada, sin login
  4  él toca "pagar parte", escribe 15, y su billetera le pide aprobar una transferencia
     de USDC en devnet
  5  la transferencia confirma, los dos celulares marcan 25 adeudados, y un enlace bajo el saldo
     abre la firma en un explorer
  6  la dueña fija una fecha de vencimiento para los 25
  7  el recordatorio llega al celular del cliente con los 25 y el enlace

la única transacción:
  la transferencia de USDC del paso 4, desde la billetera del cliente a la billetera de la dueña,
  en devnet, con un memo que nombra el fiado; su firma está en pantalla en el paso 5;
  el saldo que muestran los dos celulares se deriva del historial de transferencias del fiado

no-objetivos (ahora no):
  cuentas, login, registro, un perfil de la tienda, una pantalla de ajustes
  incorporar al cliente a una billetera, o fondearla
  reales que entran o salen (la rampa de entrada y de salida a fiat)
  pagar un fiado completo, o cerrarlo
  tiendas que se registran solas, un segundo dueño por tienda
  el registro de proveedores y los sellos de lealtad del memorando de la idea
  un programa propio de Fiado para el fiado (los pagos ya son el registro on-chain,
  una transferencia con memo cada uno; el directorio de clientes, el monto de apertura,
  la fecha de vencimiento y el recordatorio se quedan en un backend simple; un programa
  para el fiado va en la diapositiva de próximos pasos)
```

Los montos son del guion, **40 en el fiado y 15 pagados**, para que dos celulares puedan mostrar el mismo número cambiando, y cualquier par chico funciona. Lee cada paso como lo leería una cámara. Si un paso no tiene pantalla no es un paso, es plomería, *y la plomería va debajo de un paso, nunca al lado de uno*. Debajo de la tarjeta escribe la nota de Lucia de la ronda 1 de feedback, que sus clientes nunca deberían toparse con **la palabra stablecoin**, como restricción para cada pantalla que ve el cliente: su pantalla dice pagar parte y un monto, su billetera dice USDC porque las billeteras lo dicen, y la narración dice Solana una vez, en 1:30, al jurado y no a él.

5. **Dale un destino a cada no-objetivo**. Cada línea dice este mes no, y las mejores líneas también dicen adónde va la cosa en su lugar, *porque un no-objetivo sin destino se vuelve a discutir en la semana 3* por alguien que no estaba en la sala cuando se cortó. La lista tiene **al menos tres líneas**, y la primera línea es la que el equipo más quería construir. Si la lista se siente corta y cómoda, vuelve a tu camino de once pasos, o el número que haya sido, y lee qué se quitó, *porque esas son tus primeras líneas*. La lista de Fiado se ordena en **tres destinos**, la diapositiva de próximos pasos, después de la temporada, y de vuelta al memorando de la idea.

![Cada uno de los siete no-objetivos de Fiado tiene un destino, la mayoría la diapositiva de próximos pasos, dos de ellos después de la temporada y uno de vuelta en el memorando de la idea, con el programa para el fiado marcado por la disyuntiva.](assets/v03-table.webp)

6. **Nombra la única transacción** con dos preguntas: ¿es el dinero moviéndose de verdad, en la dirección que dice el pitch, y puede el espectador verla confirmar? Para Fiado es una **transferencia de USDC en devnet** desde la billetera del cliente a la de la dueña, una transferencia de token con un memo que nombra el fiado y no una llamada a un programa, *porque es la integración más liviana que todavía prueba el valor de la oración*. El saldo que muestran los dos celulares se deriva del historial de transferencias de ese fiado, así que el número que lee el cliente es uno que cualquiera puede recalcular desde el explorer *sin confiar en la app*.

Escribe **tres notas** en la tarjeta para la semana de construcción. El USDC de devnet es un token de prueba que sale de un faucet, así que la billetera del cliente se **fondea antes de grabar**. La firma entra en la bitácora narrativa el día que existe, con el enlace al explorer, *porque es la única línea que un jurado puede clicar*. Y la confirmación queda en **1:30** y no al final, donde un espectador que dejó de prestar atención se la perdería. Si en tu proyecto no se mueve dinero, la única transacción es el cambio de estado que el pitch afirma y que un desconocido podría verificar: un registro escrito, un token minteado, una firma que prueba quién hizo qué y cuándo.

7. **Llena las cuatro marcas** desde los siete pasos. Una marca es una pantalla más una oración de narración y nada más, *y no se narra ninguna función que no esté en pantalla en ese momento*. El guion v0 de Fiado, con la marca de 1:30 dejada para ti:

```text
demo-script.md, Fiado, v0, <fecha>

0:00  en pantalla: el celular de la dueña, Fiado abierto en una lista de fiados vacía. Ella toca
      nuevo fiado y escribe el nombre de pila y el teléfono del cliente. Narración, una oración:
      qué es un fiado en una tienda de barrio. Para 0:25 ya agregó la compra de hoy y el fiado
      marca 40 adeudados. (pasos 1 y 2)

0:30  en pantalla: el segundo celular. El cliente abre el enlace desde sus mensajes y ve
      40 adeudados, el mismo número, sin instalar nada, sin login. Narración: la dueña y el
      cliente están leyendo un solo saldo, que es lo que la libreta nunca pudo hacer. (paso 3)

1:30  en pantalla: <escribe esta marca: pasos 4 y 5. Nombra la transacción, qué ve el espectador
      mientras confirma, y qué marcan los dos celulares después de que confirma>

2:15  en pantalla: el celular de la dueña. Ella fija una fecha de vencimiento para los 25. El
      celular del cliente se enciende con el recordatorio, los 25 y el enlace. Narración: el
      recordatorio es la afirmación que el equipo está probando con sus design partners
      (clientes socios). (pasos 6 y 7)

2:45  en pantalla: la lista de fiados de la dueña, una fila, 15 pagados con el enlace a la firma,
      25 por vencer en la fecha. Narración, una oración: qué no está en esta demo y dónde vive.
      Corte en 3:00 o antes.
```

**Escribe la marca de 1:30 de Fiado** desde los pasos 4 y 5 de la tarjeta, en la forma que usan las otras marcas: qué hay en pantalla, una oración de narración, los números de paso entre paréntesis. Tiene que nombrar la transacción, decir qué ve el espectador mientras confirma, y decir qué marcan los dos celulares después. Luego lee el guion completo en voz alta **con un cronómetro corriendo** y mira dónde cae realmente el 1:30. Si en tu lectura la transferencia confirma después de 2:00, *las marcas anteriores son demasiado largas, no la transferencia*.

Fíjate qué quita cada marca del repo: el paso 2 necesita **un campo de monto y ningún catálogo**, el paso 3 necesita un enlace que abra en un navegador y ninguna app de cliente, y el paso 7 necesita un mensaje en un celular, así que qué servicio lo envía es *una decisión de la semana de construcción*.

8. **Ahora el tuyo**, desde tu paquete de evidencias y tu pitch v1, solo: el único trabajo, el camino feliz contado y recortado, los no-objetivos con destinos, la única transacción y su dirección, y luego las cuatro marcas con la transacción en 1:30 o cerca. El mismo día **escribe la diapositiva de próximos pasos**, dos líneas, nombrando la arquitectura a la que la demo no llega y por qué todavía no está, para Fiado un programa para el fiado que ponga el monto de apertura y la fecha de vencimiento junto a los pagos.

Un jurado que puntúa Product + Execution (producto y ejecución) lee esa diapositiva como un equipo que eligió, las funciones priorizadas estratégicamente que pide la revisión del repo, *y un jurado que en cambio encuentra la brecha en la entrevista la lee como un equipo que la escondió*. **Ponles fecha a los dos archivos**, haz commit junto al paquete de evidencias, y espera una v1 cuando la construcción los cambie.

![La demo de Fiado pone el fiado en 0:00, el saldo compartido en 0:30, la transferencia en devnet en 1:30, el recordatorio en 2:15 y la lista de fiados de cierre en 2:45, por debajo del límite de tres minutos.](assets/v04-timeline.webp)

## Está listo cuando

- El camino feliz tiene **siete pasos o menos** y cada paso es algo que una cámara puede ver.
- La lista de no-objetivos tiene **al menos tres líneas** y la primera duele un poco.
- El guion nombra **la única transacción** y qué ve el espectador cuando confirma.

## Ojo con

- **Un camino feliz con un login**, un registro y una pantalla de ajustes adentro es el tiro en el pie que esta lección existe para nombrar.
- Un agente construye **una pantalla de ajustes en segundos** y un jurado le da cero puntos, así que elige el trabajo por lo que un jurado puede evaluar, *no por lo que el agente construye más rápido*.
- Si nada en la demo deja una firma, es **un video de un sitio web** y se puntúa como tal, porque un jurado no puede verificar un mock.

La disyuntiva: una porción que se demuestra bien en tres minutos muchas veces **no es la arquitectura que enviarías a producción**, y un mes de hackathon no puede construir las dos, así que *prefiero mostrar una transferencia que confirma y una diapositiva que dice qué sigue, antes que una arquitectura que nadie puede ver*.

## Lo que queda

La demo se decide antes que el código, y el código es solo lo que los tres minutos muestran. Una porción bien recortada cabe en un respiro, algo como *"ella abre un fiado, él paga parte, el recordatorio se dispara"*, y si toma dos respiros un paso está cargando un segundo trabajo, y ese paso es **tu próximo no-objetivo**. Cada no-objetivo recibe un destino, para que nadie lo vuelva a discutir en la semana 3.

## Siguiente

La próxima lección abre con **el kit clonado y contado**, después un calentamiento de una hora donde corres una transcripción dada de plan mode (modo de planificación) para los primeros tres pasos de Fiado y ves aterrizar la transacción, y solo entonces tus propios siete pasos entran a una sesión de plan mode y el equipo de agentes los construye en devnet. Te va a sorprender lo rápido que va, y después lo que hizo mal de forma plausible, *y el guion de hoy es cómo vas a saber cuál es cuál*.
