# Hacia dónde van los rieles desde aquí

## Resumen

La lección pasada cableaste cada peldaño en un solo workspace y viste pasar en devnet un journey de comprador completo de siete tramos, con el verificador afirmando cada tramo a medida que aterrizaba. La tienda está abierta al público. Lo que quiere decir que a esta lección no le queda nada por construir, y no voy a pretender lo contrario. Ninguna API nueva, ningún peldaño nuevo. Lo que te toca en cambio es lo que todo curso te debe al final y casi ninguno entrega: un mapa honesto de dónde estás parado, fechado, con las partes móviles marcadas como móviles. Vamos a recorrer el mapa de conceptos una última vez, trazar una línea dura en la tabla de versiones entre lo que está congelado y lo que sigue en movimiento, nombrar los cursos hermanos que son dueños de la profundidad que este postergó a propósito, y cerrar el marco que abrimos en la primerísima lección. Todavía hay una cosa por hacer con tu terminal, y va primero.

## El mapa con una fecha encima

Corre esto. Es el último comando que este curso te va a pedir de entrada:

```bash
curl -sI https://github.com/solana-labs/solana-pay | grep -i '^location'
# location: https://github.com/solana-foundation/pay
```

Ese 301 es por donde entraste. El módulo uno abrió contigo decodificando el dólar de 3.6 centavos de un desconocido en mainnet, y unos minutos después conociste el repo detrás de `@solana/pay` y aprendiste su historia: el repositorio canónico de Solana Pay, alguna vez la casa del checkout con QR para humanos, ahora redirige a un repo de la Foundation llamado simplemente `pay`, cuyo producto estelar es una CLI de pagos agéntica. Corre `npm i @solana/pay` en una carpeta de prueba hoy —local, como siempre; la prohibición de `-g` del módulo uno sigue en pie— y te instalas ese binario de CLI al lado de la librería de checkout (redirección y README verificados el 2026-08-21, y la redirección de arriba se acaba de re-verificar sola en tu máquina). El marco con el que este curso abrió, el repo que cambió de bando, es el marco con el que cierra. Esa redirección es toda la tesis de esta lección final comprimida en un solo header HTTP: los rieles sobre los que acabas de construir están vivos, y vivo quiere decir en movimiento.

![Línea de tiempo desde el primer curl a mainnet del lector en el módulo uno, pasando por el repo de Solana Pay redirigiendo al repo pay de la Foundation, hasta la misma redirección vuelta a sondear en la lección final.](assets/v01-timeline.png)

Así que antes de hablar de lo que se mueve, mira lo que cruzaste. Nueve módulos, y nunca fueron un surtido al azar; fueron un solo argumento, cada módulo arreglando el límite que expuso el anterior. Aprendiste los rieles y lo que el no-chargeback le hace al dinero. Construiste el kit de transferencias que importó cada peldaño posterior. Pusiste el checkout frente a humanos de tres maneras: página con QR, puesto de feria, blink. Construiste el back office que no le cree a ningún frontend y verifica cada pago del lado del servidor. Facturaste en un cronograma sin custodia. Cruzaste la frontera fiat en ambas direcciones y aprendiste a preguntar quién es merchant-of-record en cada costura. Mediste el consumo de una API para compradores máquina sobre dos protocolos. Patrocinaste comisiones, pusiste ventas en fila offline y pasaste una barrera de producción. Después cableaste todo eso en Wavelength y viste un journey de comprador correr a lo largo de él.

![Nueve módulos dibujados como una sola cadena de izquierda a derecha, desde el modelo de rieles pasando por el kit de transferencias, las superficies de checkout, el back office, las suscripciones, la frontera fiat, los pagos entre máquinas y el endurecimiento de producción, terminando en el capstone de Wavelength.](assets/v02-diagram.png)

Cada uno de esos movimientos se entregó contra versiones fijadas, y aquí viene la parte que importa ahora: esos pins no envejecen a la misma velocidad. Algunos ya terminaron de moverse. Algunos tenían días de vida cuando se escribió este curso. Tratar esas dos categorías igual es el error más caro que te puedes llevar de aquí.

### El estante congelado y el estante móvil

Toma la tabla de versiones del curso y trázale una sola línea.

Del lado congelado están la spec v1 de Solana Pay y todo el stack de blinks. La página de la spec es visiblemente de cosecha 2023; todavía menciona a FTX y a Slope en sus ejemplos de billeteras, lo que se lee alarmante hasta que entiendes lo que quiere decir. El formato de URL de transfer-request no ha necesitado cambiar, así que no ha cambiado. Tu checkout con QR del módulo tres corre sobre un formato de cable que lleva años estable. La misma historia un estante más allá: `@dialectlabs/blinks` entregó por última vez 0.22.5 en abril de 2025, `@solana/actions` entregó por última vez 1.6.6 en noviembre de 2024, y la spec de actions está en 2.4.2 (los tres vueltos a comprobar contra npm el 2026-08-23, sin cambios). Más de dieciséis meses de silencio.

Ahora la trampa, y es exactamente la que el quiz adjunto a esta lección te va a tomar: congelado no quiere decir muerto. Congelado quiere decir estable. Una spec que dejó de cambiar porque funciona es la dependencia más segura que tienes; los nombres de campo que sirve tu endpoint de blink van a parsear en tres años. El instinto que traes de la cultura npm, donde un paquete sin tocar por un año huele a abandonado, apunta exactamente al revés para las superficies de protocolo. El formato de URL de Solana Pay y la spec de actions son las dos cosas de tu stack con MENOS probabilidad de romperse debajo de ti. No reescribas código de checkout que funciona porque su página de spec nombra a un exchange muerto.

El estante móvil es la historia opuesta, y deberías sentir la diferencia de temperatura. `@solana/subscriptions` estaba en 0.5.0, publicado el 2026-08-10, y el build v0.5.0 correspondiente del programa Delegation se desplegó a mainnet el mismo día, como te dijo el módulo cinco en su momento: esa fecha es el deploy de v0.5.0, no el debut del programa en mainnet, y el cliente por el que factura tu club del disco del mes todavía tenía días de vida cuando lo aprendiste. Los paquetes de x402 estaban en 2.23.0, publicados el 2026-08-18, cinco días antes de la fecha de escritura de este curso. La CLI de `pay` con la que le pusiste barrera a la API de precios de prensado etiqueta releases más rápido de lo que la mayoría lee changelogs. Y MPP no es ni siquiera un estándar publicado, en ninguna de sus dos capas: el esquema base es un Internet-Draft de la IETF, draft-httpauth-payment-00, que formalmente vence el 2026-12-21, y la mitad de Solana contra la que pusiste la barrera, draft-solana-charge-00, es una spec de método de pago que no está registrada en la IETF para nada y se mueve cada vez que se mueve su repo. Todo lo que está en este estante se habrá movido para cuando le pases este stack a un colega. La única pregunta es qué tan lejos.

Después está `@solana/kit`, la única pieza móvil que ya aprendiste a manejar. El tag latest de npm dice 8.0.0 (comprobado el 2026-08-23), mientras que tus workspaces sostienen a propósito la línea 6.10 donde `@solana/pay` lo exige y la línea 7.1 donde lo hace el cliente de suscripciones, un pin por workspace en cada package.json (npm los escribió como rangos con caret, que sostienen el major — y el major es lo que le importa a la matemática de peers). Esa fue la lección de costuras del módulo cinco, y fíjate en lo que te enseñó sin decirlo: una dependencia móvil no es una amenaza cuando fijas por workspace, registras por qué, y nunca escribes la palabra latest en nada. Llevas todo este tiempo practicando para el estante móvil.

![Tarjeta de dos columnas que divide la tabla de dependencias del curso en un estante congelado estable por un año o más y un estante móvil de solo días o semanas de vida.](assets/v03-comparison.png)

Lo que nos lleva a la segunda trampa, la que este curso te viene metiendo en la cabeza calladamente desde el módulo uno: nunca cites de memoria un número que se mueve. Cada cifra macro de este curso llegó con una fuente y una fecha pegadas. La oferta de stablecoins fue una captura de DefiLlama con fecha, no un hecho. El panorama de versiones de kit fue una lectura de dist-tag con fecha. Ese hábito era la lección. Cuando cites la versión del cliente de suscripciones del próximo trimestre, o el estado de MPP, o cualquier cosa del estante móvil, la vuelves a consultar en el momento de usarla o dices que no lo hiciste. No hay una tercera opción que te mantenga honesto.

### La pregunta que sobrevive a cada pin

Una cosa de tu mapa no está congelada ni se mueve, porque no es software. Cada módulo que tocó dinero real se topó con la misma pregunta en una forma distinta: ¿quién es merchant-of-record? En la frontera fiat decidió quién carga con el KYC. En el procesamiento de aceptación decidió quién se come el cumplimiento. En la apuesta de x402 contra MPP fue la fila que de verdad separó a los estándares. Aquí está la versión fría de por qué sobrevive a cada pin de tu tabla: el regulador no lee tu package.json. Cuando el dinero se mueve y algo sale mal, la pregunta que se hace en toda jurisdicción del planeta es quién era el comercio, y alguna entidad con nombre va a ser la respuesta, ya sea que la hayas elegido a propósito o que hayas caído en ella por defecto. Los protocolos van a cambiar sin parar debajo de ti el resto de tu carrera. Esa pregunta va a estar sentada en la misma silla, sin cambios, todas y cada una de las veces. Es la única pieza de este curso que te puedo prometer que no va a necesitar re-verificación en 2030.

Y se empareja con el hecho estructural que le dio forma a todo lo demás: aquí no hay chargebacks por construcción. Esa sola asimetría es la razón de que tus reembolsos sean pagos push que tú originas, de que la verificación sea del lado del servidor y definitiva, de que tu postura ante disputas no se parezca en nada a la de una integración con tarjeta. Las comisiones y la velocidad son características. El chargeback ausente es la física.

### Dónde vive la profundidad ahora

Este curso hizo cortes deliberados, y los hizo en voz alta. Cada corte tiene un dueño, y como no hay una lección siguiente a la que postergarlos, aquí es donde te entrego las puertas de verdad.

El aterrizaje de transacciones bajo carga, la ciencia de las priority fees, los bundles de Jito, el ajuste del compute budget y la indexación a escala más allá del webhook de un solo comercio: este curso te dio la receta de una sola caja y se detuvo. El curso Master Solana Frontend and Client-Side Development, el que cada traspaso de estos nueve módulos llamó Client-Side Mastery para abreviar, es dueño de todo ese territorio; las políticas de envío con nonce durable que tu fila de la feria solo pidió prestadas están a esa profundidad. Los internals de Token-2022, la maquinaria detrás de las ocho extensiones que leíste en el mint de PYUSD en el módulo dos, transfer hooks incluidos: ese catálogo es territorio del curso Digital Assets, Tokenization and Token Extensions. Las stablecoins que rinden como USDY, y profundidad de oráculos de verdad más allá del precio de display: el curso DeFi and RWA Engineering. Por qué funciona la finality, qué son en realidad confirmed y finalized debajo de tu política de confirmación, y la reescritura del consenso que este curso marcó como roadmap allá en el módulo cuatro: Low-Level Solana. Y si ver el programa Subscriptions te dio ganas de escribir programas en vez de solo llamarlos, Master Anchor V2 es el curso de autoría de programas que este nunca fue; acuérdate de que Wavelength se entregó sin una sola línea de Rust.

![Mapa de centro y radios con el stack de comercio del lector en el centro y cinco flechas llevando capacidades postergadas hacia los cinco cursos hermanos que son sus dueños.](assets/v04-diagram.png)

Más allá de los cursos, mantén una lista de vigilancia corta de superficies, porque el estante móvil se va a mover y estas son las que lo anuncian. Los tags de release del repo pay de la Foundation, para la CLI y la librería de checkout por igual. Las páginas de npm de los paquetes de suscripciones y de x402, donde un salto de versión es tu señal para volver a leer un changelog antes de confiarte del código del curso. El sitio del ecosistema x402, donde los logos de los adquirentes te dicen hacia dónde se está inclinando la adopción de los comercios; el día en que el logo de un PSP grande aparece o desaparece de ahí vale más que un trimestre de noticias de protocolo. El registro de Dialect, para saber si la historia del renderizado de blinks se vuelve a despertar. Y MPP se lleva dos entradas en vez de una, porque sus capas se mueven de forma independiente: la página del datatracker de la IETF para el esquema base, `draft-httpauth-payment`, donde un `-01` o aterriza o llega primero el vencimiento, y el log de commits de `tempoxyz/mpp-specs`, que es el único lugar donde la spec del método `solana/charge` anuncia un cambio siquiera. Seis superficies, siguen siendo veinte minutos al mes, y aquí un recordatorio en el calendario le gana a las buenas intenciones; la deriva no se anuncia a la gente que no está mirando. Ese es todo el costo de mantenimiento de estar al día con un stack que ahora entiendes de punta a punta.

Aquí está la contrapartida honesta de todo este curso, dicha tan claro como puedo. Todo lo que construiste es una captura de un stack que se mueve rápido, y partes de la captura se tomaron a mitad de un sprint. Los pins se van a pudrir; algunos ya empezaron. Lo que no se pudre es lo que los pins estaban enseñando. El patrón de integración, un servidor que pone precio y construye mientras un cliente solo firma, sobrevivió intacto desde el checkout con QR del módulo tres hasta el agente pagando un 402 del módulo siete, y cualquier estándar que gane la guerra de los pagos entre máquinas va a ser una implementación más de esa misma forma. Y la disciplina de verificar del lado del servidor, no confiarle nunca a un frontend, a un webhook o a la palabra de una billetera por encima de la del libro mayor, es el hábito que tu verificador impuso en cada peldaño hasta que dejó de sentirse como disciplina y empezó a sentirse como sentido común. Esas dos se transfieren a stacks que todavía no existen. Lee esta conclusión como un mapa con una fecha encima, no como un índice permanente. Los caminos del mapa duran más que sus pueblos.

## Lab: el ritual de re-verificación

Cada lab anterior te llevó de la mano por algo. Este no, y ese es justo el punto: el traspaso que este curso viene corriendo desde el módulo uno, cada lección dejándote un poco más del trabajo a ti, termina aquí en autonomía total. El ritual de abajo es el que vas a correr solo, meses a partir de ahora, antes de reusar cualquier parte de este stack. Casi todo comandos, dos momentos cortos de escritura al final. El criterio es tuyo.

![Diagrama de flujo del ritual de re-verificación: sondea cada pin móvil, compara con la tabla fechada, lee el changelog ante cualquier cambio y registra una nota fechada nueva.](assets/v05-flowchart.png)

1. Sondea el marco mismo:

   ```bash
   curl -sI https://github.com/solana-labs/solana-pay | grep -i '^location'
   ```

   Espera el mismo header `location:` que corriste al principio de esta lección. Un destino distinto, o ningún header, es la señal más fuerte posible de que el suelo se movió.

2. Vuelve a consultar cada pin móvil (no hacen falta instalaciones, `npm view` es de solo lectura):

   ```bash
   npm view @solana/pay version
   npm view @solana/subscriptions version time.modified
   npm view @x402/core version
   npm view @solana/kit dist-tags
   ```

   Una nota de lectura de la lección uno que sigue aplicando: `@solana/pay` responde con la versión del wrapper de npm (1.0.26 al momento de escribir el curso), mientras que la tarjeta del estante sigue el tag de release propio de la CLI (`pay-v0.27.0`), el binario que ese wrapper descarga. Dos esquemas de versión, una herramienta, ninguno equivocado; compara cada uno contra su propia línea.

   Espera cuatro respuestas que no puedes predecir desde aquí; esa imprevisibilidad es la definición del estante móvil. Lleva las cuatro al paso 4 en vez de juzgar alguna de ellas por separado.

3. Demuestra que el estante congelado sigue congelado:

   ```bash
   npm view @dialectlabs/blinks version time.modified
   npm view @solana/actions version time.modified
   ```

   Espera 0.22.5 y 1.6.6, los mismos dos números que citó esta lección. Si alguno de los dos se movió, el estante congelado se acaba de descongelar, y ese es el resultado más interesante que este ritual puede producir: ve a leer qué se despertó.

4. Compara cada resultado contra la tarjeta de estantes congelado-y-móvil de arriba; esa tarjeta es la tabla de versiones del curso, no hay un archivo aparte que buscar. Para cualquier cosa que se haya movido, encuentra y lee su changelog antes de correr cualquier workspace del curso contra la versión nueva. No actualices por reflejo; decide.

5. Escribe tu propia tabla de frescura fechada: paquete, versión que observaste, fecha, y una palabra, `frozen` o `moving`. Haz commit de ella en tu repo de Wavelength al lado de tus reportes de gate; de ahora en adelante ella, no esta lección, es la tabla en la que confías.

Checkpoint: tu repo contiene una tabla de versiones fechada hoy, de tu propia mano, con cada pin móvil vuelto a observar y cada deriva anotada. Yo corrí este ritual exacto el 2026-08-23 mientras escribía este cierre: el estante congelado se sostuvo exactamente, y el estante móvil mostró precisamente la deriva que esta lección ya narra, el latest de kit en npm sentado dos majors más allá de nuestros pins, que es un estante móvil haciendo lo que hacen los estantes móviles. La tuya es la próxima observación.

## Challenge

Arma tu plan de qué sigue, y hazlo lo bastante concreto para actuar sobre él. Cinco filas, tres columnas: la capacidad que este curso postergó, el curso hermano que es su dueño, y la única superficie del ecosistema que vas a seguir personalmente para ella. Las cinco capacidades están fijas: aterrizaje de transacciones, internals de Token-2022, stables que rinden, internals de finality y autoría de programas. Los nombres de los cursos están en esta lección. Las superficies son tuyas para elegir, y elegirlas es el ejercicio; una superficie que no vas a comprobar de verdad es una respuesta equivocada aunque sea técnicamente relevante. Después agrega una sexta fila para lo que personalmente más quieras profundizar, sea o no dueño de eso algún curso. Acepta cuando: cinco filas correctas, cinco superficies para las que puedas nombrar una cadencia de comprobación, y la sexta fila te asuste un poco.

## El último Checkpoint

Si el ritual sacó a la superficie una deriva, eso no es un defecto del curso; es el curso funcionando. Un pin movido más tu nota fechada más una lectura del changelog es precisamente la postura que este stack exige, y es una postura que la mayoría de los integradores en ejercicio nunca desarrolla. Si algo de la deriva rompe un workspace del curso y el changelog no lo explica, tienes un verificador que afirma cada tramo de un journey de comprador; apúntalo a la rotura y deja que te diga qué peldaño se movió. Ese harness siempre fue el entregable de verdad.

Hace nueve módulos sacaste de mainnet el pago de 3.6 centavos de un desconocido con una línea de curl y sin permiso. Hoy una tienda que construiste cobra dinero de siete maneras distintas, verifica cada centavo del lado del servidor, factura sin custodia, le vende a máquinas y sobrevive a la feria sin señal. Entre esos dos puntos, la puerta de entrada del propio ecosistema cambió de bando, y lo viste pasar con una sonda de redirección en vez de con un rumor. Esa es toda la habilidad, honestamente. No los pins. El hábito de comprobar, fechar y construir de todos modos sobre rieles que se niegan a quedarse quietos.

No hay una lección siguiente. Hay un mapa en tu repo con la fecha de hoy encima, cinco puertas con nombres de cursos, y una tienda que está abierta. Hacia dónde van los rieles desde aquí es en parte una pregunta sobre protocolos, y esta lección te dio la respuesta honesta: algunos están congelados, algunos se mueven, vuelve a consultar antes de confiar. Pero sobre todo es una pregunta sobre ti, y esa la acabas de responder en la sexta fila de una tabla. Ve a construir la cosa que te asustó un poco. Felices ventas.
