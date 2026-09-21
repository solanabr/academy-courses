# Constrúyelo con un equipo de agentes

La lección pasada escribiste el guion de la demo v0 con sus cuatro marcas de tiempo y recortaste la construcción a una tarjeta de alcance: un trabajo, siete pasos, siete no-objetivos y la única transacción, un cliente habitual que paga 15 de un fiado de 40 en USDC en devnet. **Deja los dos archivos abiertos**, *porque cada prompt y cada revisión de esta semana salen de ahí*.

## Por qué importa

Las preguntas frecuentes de Colosseum, al momento de la temporada World's Fair de 2026, lo dicen sin rodeos: 'We have backed non-technical founders in our Accelerator who built MVPs entirely with AI coding tools' (hemos respaldado en nuestro Accelerator a fundadores no técnicos que construyeron sus MVP enteramente con herramientas de programación con IA) (colosseum.com/hackathon, 2026-09-06). *A los jurados no les importa que el código lo haya escrito un agente.* Les importa que funcione, que el trabajo se haya hecho entre la fecha de inicio y la de fin de la temporada, y que puedan **correrlo desde tu README**.

La semana entera cabe en una idea: **el equipo de agentes construye los pasos**, y tú te quedas con el camino de la demo, la única transacción y el clon limpio. Las herramientas son Claude Code y el plugin Solana AI Kit desde el día 0, y *lo primero que lees es el árbol del propio kit*. Clónalo junto a tu repositorio del toolkit:

```bash
git clone https://github.com/solanabr/solana-ai-kit
cat solana-ai-kit/.gitmodules
ls -l solana-ai-kit/plugin/skills
```

## Manos a la obra

![La semana va de la tarjeta de alcance a una transacción que envías y lees a mano, pasando por una sesión de plan mode y tres subagentes, y termina con un quickstart desde un clon limpio, con la regla de la ventana de la temporada debajo.](assets/v01-flowchart.webp)

1. **Cuenta las entradas de submódulos** en la primera salida y los symlinks en la última. Leído el 2026-09-06, con un último commit del 2026-08-20, la primera salida tenía **18 entradas**, entre ellas colosseum, que apunta a ColosseumOrg/colosseum-copilot, solana-new, que apunta a sendaifun/solana-new, y helius, solana-dev, sendai, metaplex y jupiter. La última salida tenía tres, hackathon, idea-sprint y pitch-deck, cada una un symlink hacia .claude/skills, donde vive la skill wrapper.

2. Abre **una de las tres skills wrapper** y lee su encabezado. Cada una dice que fue adaptada de sendaifun/solana-new, MIT 2026 SendAI y Superteam, con la telemetría quitada, y *por eso la wrapper es la versión que corres siempre que existan las dos*.

Las wrappers son **la primera capa del kit**. La segunda son los comandos y agentes del propio kit, y los cuatro en los que se apoya esta semana son /plan-feature, /scaffold, /build-app y /diff-review.

La tercera es ext, que solo llega con la instalación completa que describe el README, y en ext/solana-new están las tres skills a las que una semana de construcción recurre cuando una wrapper no cubre algo: scaffold-project, build-with-claude y debug-program. Las skills upstream de ahí traen al inicio un preámbulo en bash que llama a casa antes de que la skill corra, y la página central del kit advierte que esos bloques de preámbulo nunca se ejecutan, así que abre primero el archivo de la skill, **sáltate el bloque de arriba**, y después corre la skill. ext/colosseum es Colosseum Copilot y sigue necesitando el PAT de la semana 1.

Cada nombre de comando, skill y modo en esta página se cotejó el 2026-09-06 con solanabr/solana-ai-kit main y con la documentación de Claude Code, y los dos cambian con cada versión, así que *si tu salida es distinta de estas, manda tu salida*, y **verifica cada nombre** en el README actual del kit antes de escribirlo.

3. **Crea un repo vacío** llamado fiado-warmup junto al toolkit. Colosseum, en la página leída el 2026-09-06, evalúa a los equipos solo por el trabajo completado entre las fechas de **inicio y fin** de la competencia, el código preexistente hay que declararlo, y mentir en cualquiera de las dos cosas puede descalificar a un equipo, vetarlo y quitarle un premio. *Un agente va a traer con gusto una plantilla que escribiste el año pasado*, así que el repo empieza vacío el primer día de la semana 2 y cualquier cosa más vieja se declara en el README.

4. **Abre Claude Code dentro del repo** y arranca la sesión de planificación con /plan-feature. Plan mode (modo de planificación) es el modo de Claude Code en el que el agente lee el repo, escribe un plan y **no edita nada hasta que lo apruebas**. Pega la transcripción de abajo tal cual. Es la tarjeta de alcance de Fiado de la lección pasada en la forma de prompt del kit, y *cuando vuelvas por tu propio recorte, tu tarjeta va en lugar de estas líneas*:

```text
/plan-feature
Construye el recorte de la demo de abajo. Solo planifica. No escribas archivos todavía.
Único trabajo: un cliente habitual paga parte de su fiado desde su propio celular.
Paso 1: la dueña abre Fiado en su celular y abre un fiado para un cliente habitual: su nombre de pila y su número de teléfono.
Paso 2: ella agrega la compra de hoy, 40, y el fiado marca 40 adeudados; un enlace va a su teléfono.
Paso 3: el cliente abre el enlace y ve los mismos 40 en su celular, sin instalar nada, sin login.
Paso 4: él toca "pagar parte", escribe 15, y su billetera le pide aprobar una transferencia de USDC en devnet, con un memo que nombra el fiado. Esta es la única transacción real.
Paso 5: la transferencia confirma, los dos celulares marcan 25 adeudados, y un enlace bajo el saldo abre la firma en un explorer.
Paso 6: la dueña fija una fecha de vencimiento para los 25.
Paso 7: el recordatorio llega al celular del cliente con los 25 y el enlace.
El saldo se deriva del historial de transferencias del fiado; la app guarda solo el directorio de clientes, el monto de apertura, la fecha de vencimiento y el recordatorio.
No-objetivos: cuentas, login, registro, un perfil de la tienda, una pantalla de ajustes; incorporar al cliente a una billetera, o fondearla; reales que entran o salen; pagar un fiado completo, o cerrarlo; tiendas que se registran solas, un segundo dueño por tienda; el registro de proveedores y los sellos de lealtad; un programa propio de Fiado para el fiado.
Ordena el plan para que el paso 4 se construya y corra antes de empezar los pasos 5 a 7.
```

5. **Lee el plan durante diez minutos** sin hacer nada más, contra tres cosas. Primero, los no-objetivos: casi seguro el plan va a proponer un login, porque la mayoría de las plantillas traen uno, y *la línea de no-objetivos está ahí para que lo cortes ahora y no en la semana 3*. Segundo, el orden: **el paso 4 antes** de los pasos 5 a 7, porque todo lo que viene después depende de una transacción que exista.

Tercero, la forma de la transacción: el plan debería decir USDC, devnet, y una transferencia real desde la billetera del cliente a la de la tienda, y si dice **mock, stub o simulate** en cualquier lugar cerca del paso 4, el plan te está avisando que piensa construir una demo que un jurado no puede verificar. La tarjeta de Fiado no lleva ningún programa propio, así que *si el plan propone uno, se pasó de la tarjeta*. Aprueba solo cuando las tres revisiones pasen, **guarda el plan como archivo** con la fecha en la primera línea, y haz commit, porque un jurado que lea el repo puede ver que se escribió dentro de la ventana.

```bash
git add plan.md
git commit -m "plan approved, 2026-09-22"
```

6. **Reparte el plan entre tres subagentes**, con un trabajo cada uno, y escribe cada trabajo como una especificación de pocas líneas sacada de la tarjeta, *nunca de memoria*. Un subagente es un segundo agente que Claude Code arranca para una sola tarea, y los subagentes **no comparten memoria**, así que lo único que comparten es el repo en disco.

El subagente de scaffold corre el comando /scaffold del kit contra el plan aprobado y se detiene cuando el repo tiene un README con una sección de quickstart vacía, un archivo de dependencias y **una carpeta por paso**. El subagente de frontend construye las pantallas con /build-app, y su especificación son las líneas de los pasos más las marcas 0:00 y 0:30 del guion de la demo. El subagente de tests escribe **un test por paso**, como lo que una persona ve cuando el paso está hecho, y el test que revisa el saldo después del paso 4 debería existir antes que el paso 4, *para que el paso tenga algo que reprobar*.

**Corre primero el scaffold**, después el frontend solo para el paso 1, después el paso 2, después el paso 3, un paso por corrida, y lee el repo entre corrida y corrida. *Creo que la aceleración no es pareja entre tipos de trabajo*: el scaffold está listo en minutos, el paso de la transacción toma una tarde, y esa tarde es **la parte que un jurado puede revisar**.

7. El paso 4 es tuyo. **Arranca el frontend**, abre el fiado del paso 1, toca pagar parte en el paso 3, escribe 15, y págalo desde la billetera de devnet que creaste y fondeaste con un faucet de devnet el día 0. Si no tiene USDC de devnet, primero recárgala desde un faucet de USDC de devnet (verifica cuál es el vigente en el README del kit), *porque una transacción que falla por saldo vacío no te enseña nada*. Cuando la billetera confirma, la app muestra **una firma**, la cadena larga en base58 que identifica una transacción en devnet.

**Cópiala, abre cualquier explorer**, cámbialo a devnet y pégala. **Lee primero la línea de estado**, porque una transacción puede enviarse y aun así fallar. Después el slot y la hora del bloque, *que fechan el trabajo dentro de la ventana de la temporada como ningún mensaje de commit puede hacerlo*. Después la comisión, pagada en SOL por la billetera que firmó. Después los cambios de saldo del token, USDC que sale de la cuenta del cliente y llega a la de la tienda, por los 15 que el paso 3 eligió. Después el memo, que nombra el fiado. Si el monto en el explorer no es el monto en la pantalla, **la pantalla está mintiendo** y el explorer no.

![La lectura del explorer va firma, estado, slot y hora, comisión, después el cambio de saldo de USDC comparado contra la pantalla de la app, y un resultado de no encontrado quiere decir que la transacción nunca llegó a devnet.](assets/v02-flowchart.webp)

8. **Empieza un archivo llamado narrative-log.md** en la raíz del repo y haz que esta sea la primera entrada:

```text
fecha       el día en que corrió el paso 4, escrito como 2026-09-22
evento      el paso 4 corrió en devnet
firma       la firma completa, pegada
explorer    el enlace al explorer, puesto en devnet
monto       el monto de USDC que eligió el paso 3
visto       estado, slot, comisión y el cambio de saldo del token, todo leído en el explorer
```

Ese archivo crece durante la pasada de pulido, y el deck y los dos videos se cortan de él, pero *esta primera entrada es la línea que un jurado puede clicar*.

9. **Prueba el quickstart** desde un clon limpio. Un quickstart es la parte del README con la que un desconocido, en unos pocos comandos, pasa de un clon limpio al recorte corriendo, y *el jurado que lee tu repo es ese desconocido*. Clona tu propio repo en una carpeta que nunca lo haya visto:

```bash
git clone <your-repo-url> fiado-clean
cd fiado-clean
```

Después haz exactamente lo que dice el README, y nada que no diga. Si se olvidó del archivo de entorno, de la configuración de la billetera o del USDC de devnet, te detienes, **escribes la línea que falta** en el README y vuelves a clonar en una carpeta nueva. Repite hasta que un clon que parte de cero llegue a la pantalla del paso 1, y **cronometra la última corrida**, porque ese número también va en el README.

El agente escribió un primer quickstart durante el scaffold y va a sonar bien, porque se escribió desde adentro de una sesión que tenía el archivo de entorno que el clon no tiene. *El trabajo escrito por IA puede parecer correcto y estar mal*, y el clon limpio es el instrumento más barato que tienes para atraparlo: cuesta una carpeta y diez minutos.

Después **escribe las dos líneas de declaración del README** debajo del quickstart: qué hay en el repo que sea anterior a la temporada, si hay algo, y de dónde salió, o la fecha en que se creó el repo si no hay nada. *Esas mismas dos líneas van en las respuestas del portal el día del envío.*

![El clon limpio atrapa el archivo de entorno que falta, la billetera sin fondos y la dependencia instalada a mano que la máquina del autor esconde, y el README también lleva la declaración de cualquier código más viejo que la temporada.](assets/v03-diagram.webp)

10. Los pasos 5 a 7 **van a los mismos subagentes** con el mismo tipo de especificación, un paso por corrida. En la tarjeta de Fiado, el subagente de frontend recibe el paso 5 y el paso 7, la pantalla que marca 25 adeudados con el enlace al explorer debajo y el recordatorio que llega al celular del cliente, y el subagente de tests recibe el paso 6, *porque una fecha de vencimiento es fácil de falsear y un test que dispara el recordatorio contra una fecha fija atrapa la trampa*. Lee el repo después de cada uno, y después **deja de construir**.

11. **Corre el /diff-review del kit** sobre todo lo que los subagentes escribieron desde que se aprobó el plan, toma su salida como una lista de lugares donde mirar y *nunca como un aprobado*, y después recorre el camino de la demo tú mismo, a mano, desde la marca 0:00 hasta la 2:45 del guion, con el explorer abierto en una segunda pantalla.

En Fiado la lectura a mano encuentra esto: el paso 5 muestra 25 adeudados, el test pasa, y el número son los 15 del formulario de pagar parte restados de los 40, sin que la transacción confirmada **se lea en ningún momento**, así que la pantalla mostraría el mismo número si la transacción hubiera fallado. Nada en el diff se ve mal, el test se escribió con el mismo supuesto, y *un jurado que en la entrevista paga otro monto ve a la app mentir*. El arreglo es chico, la app lee el monto de la transacción confirmada y muestra eso, y **encontrarlo tomó más tiempo que construir** el paso.

*Mi postura es que el código escrito por IA necesita una revisión distinta y más larga que el código que escribió una persona*, porque quien lo escribió no está en la sala para decirte qué supuso, y los equipos subestiman ese tiempo. **Pon la revisión en el calendario** como un bloque del mismo tamaño que el bloque de construcción, y si termina antes, recupera el tiempo.

12. **Graba la actualización semanal de Colosseum** el día que el paso 4 funcione. Es opcional y muy recomendada, en palabras de la propia página el 2026-09-06, y es **un minuto de video**, y lo que se ve en pantalla es la transacción confirmando, antes de que la pasada de pulido deje la pantalla más bonita, *porque a un jurado una pantalla simple con una firma real le vale más que una diseñada con un mock*.

![La semana 2 va desde un repo vacío, pasando por el plan, los subagentes y la transacción en devnet, y la semana 3 da los pasos 5 a 7 y un bloque de revisión del mismo tamaño que la construcción antes de la corrida final desde un clon limpio.](assets/v04-timeline.webp)

13. Ahora el tuyo: **tu propio recorte** por los siete pasos, sin ayuda, hasta devnet. Un repo vacío creado esta semana, tu tarjeta de alcance en el prompt de planificación con tus propios siete pasos y no-objetivos, las tres revisiones antes de aprobar, los mismos tres subagentes un paso por corrida, y *la misma regla sobre quién se queda con la transacción*. Si tu recorte lleva un programa adentro, **/build-program y /deploy** son los comandos del kit para eso, y la pasada de pulido corre la auditoría.

## Está listo cuando

- El quickstart funciona desde un clon limpio, siguiendo solo el README, y **la última corrida está cronometrada**.
- Hay una **firma de devnet** en narrative-log.md con su enlace al explorer, el enlace abre, y el monto en el explorer coincide con tu guion.
- La **marca de 1:30** del guion de la demo se alcanza en pantalla desde la pantalla del paso 1 sin tocar nada que el guion no muestre.

## Ojo con

- Un agente que se topa con un error de devnet a veces apunta la app a **un validador local o a un mock**, y la firma que muestra no abre nada. No encontrado quiere decir que nunca salió de localhost: arregla la ruta, envía de nuevo, lee de nuevo, y solo entonces escribe la entrada de la bitácora.
- Las skills upstream en ext/solana-new corren un **preámbulo de telemetría** al inicio del archivo, y el daño es silencioso. Lee la skill, sáltate el bloque, corre la skill.
- Un equipo de agentes entrega rápido y entrega **cosas plausibles pero incorrectas**, así que la velocidad se paga con tiempo de revisión, y el bloque de revisión va en el calendario con el mismo tamaño que el bloque de construcción.

## Lo que queda

El equipo de agentes construye los pasos, y tú te quedas con el camino de la demo, la única transacción y el clon limpio. Lo que la semana de construcción produce es **una firma que un jurado puede abrir y un README que un jurado puede correr**, y la revisión que atrapa el paso plausible pero incorrecto toma tanto como la construcción. *Si la semana te salió como una lista de funciones, vuelve a leer la primera entrada de narrative-log.md.*

## Próxima lección: que se vea real

La próxima lección haces que el recorte se vea real, que es un trabajo distinto de dejarlo más bonito. Lo primero es **abrir la pantalla del paso 1** que construyó el subagente de frontend, tomarle hoy una captura con fecha, y ponerla en narrative-log.md debajo de la firma, *porque un frontend que un jurado reconoce en un segundo como hecho por un agente cuesta más puntos que una función que falta*. Deja abierta la pestaña del explorer.
