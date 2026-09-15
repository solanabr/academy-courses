# La trampa del reload ya no existe

La lección pasada el vault aprendió a pagar. Reconstruiste sus signer seeds a partir de los seeds más el bump canónico guardado, llamaste a `invoke_signed` a través de `CpiContext::new(...).with_signer(...)`, y moviste lamports reales fuera de un PDA sin clave bajo la autoridad del programa. Ahora el vault firma por sí mismo. Los lamports lo dejan por orden del programa, no de una persona con su par de claves.

Hay un bug que vive exactamente una línea más allá de ese retiro, y en la línea vieja de Anchor se entregaba constantemente. La forma es esta: haces una CPI, después lees la cuenta que acabas de cambiar, y actúas sobre el valor viejo. Compilaba. Corría. Pasaba la prueba del camino feliz. Y después, en producción, tomaba una decisión sobre un número que ya estaba equivocado. Todo el que escribió Anchor para ganarse la vida se topó con esto al menos una vez. En V2 ese bug ya no existe, y la forma en que desapareció es más extraña y mejor que el rumor que quizá escuchaste.

Así que vamos a buscar la pared. Abre el programa R2 de la lección pasada y encuentra el handler `withdraw`. Justo después de que construyes el valor `cpi`, antes de que `transfer` lo consuma, mete una línea:

```rust
let before = ctx.accounts.state.credit;
```

Ahora corre `anchor build`. Si lo que escuchaste sobre el modelo de borrow de V2 es "no puedes tocar `ctx.accounts` mientras una CPI está en vuelo," esperas un error de compilación acá. No vas a recibir ninguno. El build vuelve verde, y está *pensado* así: `state` no es una de las cuentas que toma esta CPI, así que nada de la transferencia puede invalidarla, y el compilador lo sabe. Dónde se levanta la pared en realidad — una cuenta más allá — es el punto entero de esta lección. Lee la sección siguiente antes de tocar cualquier otra cosa, porque lo que V2 permite acá es tan deliberado como lo que prohíbe.

## Resumen

Un concepto, y es un porqué, no un cómo: leer los datos tipados de una cuenta mientras un `CpiHandle` vivo toma esa misma cuenta prestada de forma mutable es un error de compilación en Anchor V2, y esa regla — que cae exactamente sobre las cuentas que una CPI podría cambiar y sobre ninguna otra — retira para siempre la trampa del `.reload()` post-CPI de v1. No suavizada. No relegada a un lint. Hecha imposible de escribir.

Vamos a reconstruir por qué los autores del framework eligieron imponer esto con el borrow checker en vez de con una advertencia, a recorrer las alternativas ingenuas y a ver fallar a cada una, y a fijar el borde exacto de la garantía, porque una promesa de seguridad que juzgas mal es peor que ninguna. El peligro de v1 era silencioso y corría; el reemplazo de V2 es ruidoso y detiene el build. Mover un fallo de tiempo-de-ejecución-y-callado a tiempo-de-compilación-y-obvio es toda la tesis de V2, y esta es esa tesis aplicada al momento en que un programa llama a otro.

El criterio al que apuntas es chico y filoso. Dado un snippet que lee los datos tipados de una cuenta mientras el propio `CpiHandle` de esa cuenta está vivo y por eso se niega a compilar, reordenas la lectura para que caiga después de que el handle se suelta, así el programa compila, y nombras en una oración la clase de bug que V2 eliminó. Eso es todo.

La ayuda se repliega como siempre. En el Lab te muestro la forma que falla y el arreglo completo, porque quiero el error del borrow checker en tus ojos y el reordenamiento en tus dedos. En el Challenge recibes otro snippet roto sin ningún apoyo, y lo arreglas y nombras la clase tú mismo. El peldaño calificado que viene después corre la misma disciplina con el framework arrancado y un calificador mirando: un recibo que tiene que estar bien, no solo compilar. Lee, rompe, arregla, nombra.

## Por qué ahora el compilador te cubre la espalda

### La trampa de v1, con precisión

Empieza por cómo funcionaba esto antes, porque no puedes apreciar el arreglo hasta que sentiste la herida. En la línea 1.0, `CpiContext::new` tomaba el programa como un `Pubkey` simple por valor (y en la línea 0.x anterior, un `AccountInfo`), y las cuentas que le entregabas eran structs deserializadas comunes. Anchor leía los bytes de la cuenta de la cadena una sola vez, al comienzo de tu instrucción, y te entregaba una copia tipada para trabajar.

Esa copia es el problema. Cuando disparas una CPI que muta una cuenta, el cambio aterriza en los bytes reales de la cuenta on-chain. Tu copia deserializada, la struct sentada en la memoria de tu instrucción, no se mueve. Sigue guardando lo que guardaba cuando Anchor la leyó por primera vez. Así que la secuencia clásica parecía inocente y mentía:

```rust
// Anchor v1. This compiles, runs, and is wrong.
token::transfer(cpi_ctx, amount)?;              // the vault's on-chain amount drops
let remaining = ctx.accounts.vault.amount;      // reads the STALE pre-transfer copy
require!(remaining >= floor, VaultError::TooLow); // decides on a number that is already false
```

El saldo real del vault bajó. Tu `remaining` no. Después condicionaste un pago, o una acuñación, o una liquidación a un valor que la cadena ya había invalidado. El arreglo que ofrecía v1 era un método que tenías que acordarte de llamar: `.reload()`, que releía los bytes de la cuenta y refrescaba tu copia.

```rust
token::transfer(cpi_ctx, amount)?;
ctx.accounts.vault.reload()?;                   // re-read the live bytes; NOW the copy is fresh
let remaining = ctx.accounts.vault.amount;
```

Una línea. Barata. Y catastrófica de omitir, porque omitirla no producía ningún error, ninguna advertencia, ningún panic. Nada más un programa que leía el pasado en silencio.

![Anchor deserializa el vault en 100, una CPI de transfer baja el saldo real a 40, y solo .reload() refresca la copia obsoleta antes de que el código decida.](assets/v01-flowchart.png)

Quédate un momento con por qué esto era tan peligroso. No era que el arreglo fuera difícil. Era que el fallo era invisible. Un `checked_sub` que falta entra en panic y lo ves en los logs. Un `.reload()` que falta tiene éxito, y el único testigo es un valor que está sutilmente equivocado en un camino de código que una prueba rara vez ejercita. Lo silencioso-y-equivocado es estrictamente peor que lo ruidoso-y-roto, porque lo ruidoso se arregla el martes y lo silencioso se arregla después de un incidente.

Antes de que le caigamos encima a v1, igual, concédele la defensa honesta, porque el diseño no era estúpido, era un canje razonable que envejeció mal. Deserializar una cuenta una vez, al comienzo de la instrucción, y reusar esa copia es genuinamente más rápido que releer los bytes cada vez que tocas un campo. Para una instrucción que nunca dispara una CPI, y muchas no lo hacen, la copia siempre es correcta y siempre es barata. v1 optimizó para el caso común y dejó un filo afilado en el poco común. Esa es una decisión defendible hasta el momento exacto en que el caso poco común es "mover dinero", y ahí el filo está exactamente donde no te lo puedes permitir.

Y fíjate en la forma del fallo, porque es peor que "a veces está mal". Separa el caso promedio del peor caso, como deberías hacer con cualquier peligro. En el caso promedio, la cuenta que leíste después de una CPI no fue cambiada en realidad por esa CPI, así que la copia obsoleta da la casualidad de que es igual al valor vivo y tu código es correcto por accidente. Esa es la trampa: el bug prueba limpio, demuestra limpio, y se queda dormido por meses. El peor caso es el único camino específico donde la CPI sí movió el número que después lees, y ese camino tiende a ser el de alto riesgo, un pago dimensionado contra un saldo, una acuñación condicionada a un supply. Un mecanismo que está bien en los caminos aburridos y mal en el único camino que importa no es un mecanismo que quieras cuidando un vault. Es una mina con buenas probabilidades.

### La pregunta que fuerza el diseño

Así que acá está la pregunta que los autores de V2 tuvieron que responder de verdad. ¿Cómo haces que la lectura obsoleta post-CPI no esté nada más desalentada, no esté documentada, sino que sea *imposible de escribir*? No "los docs te dicen que llames a `.reload()`". Imposible. El programador no tendría que poder expresar el bug ni aunque quiera.

Esa es una vara más alta de lo que suena, y las formas obvias de superarla fallan todas. Míralas fallar, porque descartarlas es lo que hace que la respuesta real se sienta inevitable en vez de arbitraria.

![Cuatro formas de atrapar un bug de obsolescencia post-CPI, donde los docs nunca lo atrapan, un lint se puede silenciar, el auto-refresh cuesta trabajo en tiempo de ejecución, y la regla de borrow lo atrapa en tiempo de compilación.](assets/v02-comparison.png)

### Los arreglos ingenuos, descartados por niveles

El primer arreglo ingenuo es el que v1 ya intentó: escribirlo en los docs. "Acuérdate de llamar a `.reload()` después de una CPI que muta una cuenta que lees." Esto falla al primer contacto con la memoria humana. No es un mecanismo, es una esperanza, y es precisamente la esperanza que falló durante una década. Una regla que se hace cumplir acordándose es una regla que se rompe la semana en que estás cansado.

El segundo arreglo ingenuo es un lint. Entrega una regla de clippy que marque una lectura después de una CPI. Mejor, porque lo verifica una máquina. Pero un lint es orientativo por construcción. Le puedes poner `#[allow]`, puedes no correr clippy en CI, y peor, un lint que intenta rastrear "¿es esta cuenta la que mutó la CPI, a través de este alias, cruzando esta función auxiliar?" es exactamente el análisis de flujo de datos sobre todo el programa en el que los lints son malos. Se va a perder los casos interesantes y va a dar falsas alarmas en los aburridos. Una garantía que puedes silenciar no es una garantía.

El tercer arreglo ingenuo es el tentador: que el framework refresque la cuenta por ti. Después de cada CPI, volver a deserializar en silencio cada cuenta que tengas, así tu copia nunca está obsoleta. Correcto, y nunca se olvida. Pero ahora mide el costo contra la línea base realista, porque "¿comparado con qué?" es la única forma honesta de juzgarlo. La deserialización eager de `Account<T>` ya era, ella sola, el mayor gasto de cómputo del framework. El manifiesto de V2 que arrancó todo este rediseño, el issue #4390, "Zero-copy account deserialization by default," nombró exactamente eso: el `Account<T>` era el camino lento, y la queja de rendimiento número uno entre los desarrolladores de Anchor. Auto-refrescar en cada CPI tomaría la cosa más cara que hacía el framework viejo y la haría *más seguido*, en cada cuenta, en cada llamada, sin importar si alguna vez vuelves a leer la cosa. Estarías comprando seguridad con el impuesto exacto que V2 se construyó para abolir. Y todavía no cubriría las lecturas de bytes crudos, así que ni siquiera está completo.

Así que el nivel ingenuo se derrumba, y el requisito se afila hasta volverse algo más angosto y más extraño. No queremos refrescar los datos, y no queremos advertir sobre la lectura. Queremos que no puedas *sostener* una lectura de una cuenta mientras una CPI viva sostiene el derecho de cambiar esa cuenta. Si esas dos cosas son mutuamente excluyentes, la línea de la lectura obsoleta simplemente no se puede tipear. Eso no es una verificación en tiempo de ejecución y no es un lint. Eso es el borrow checker.

### El mecanismo: un CpiHandle es un borrow

La idea sobre la que se apoya todo el mecanismo: en V2 ya no le entregas a una CPI un clon de `AccountInfo`. Le entregas un `CpiHandle`, que consigues con `.cpi_handle()` o `.cpi_handle_mut()`. Y un `CpiHandle` no es una copia de nada. Es un borrow vivo de Rust sobre la cuenta, sostenido todo el tiempo que el handle siga en scope.

Esa única decisión de diseño hace todo el trabajo. Mientras un `CpiHandle` está vivo, la cuenta a la que apunta está prestada, así que el borrow checker no te va a dejar formar un segundo borrow, conflictivo, para leer datos tipados. La lectura y el handle no pueden coexistir.

![Un panel de código que muestra que dos handles mutables sobre dos campos disjuntos son legales, y que el CpiContext mantiene vivos exactamente esos dos borrows por campo hasta que la llamada a la CPI lo consume.](assets/v03-annotated-code.png)

Sé exacto sobre *qué* se toma prestado, porque la exactitud es el diseño. `cpi_handle_mut()` toma prestado un solo campo: `ctx.accounts.sol_vault` y `ctx.accounts.authority` son rutas disjuntas, y Rust siempre permitió dos borrows mutables de dos campos distintos de una misma struct. Tampoco hay magia de framework debajo de esa oración — `Context` declara `accounts` como un campo común y tu struct de cuentas es una struct común, así que las reglas ordinarias de borrow por campo son toda la historia. Por eso el par que está dentro del literal `Transfer { from, to }` está bien. Y mover los dos handles a un `CpiContext` no ensancha nada: el contexto ahora carga esos dos borrows — de `sol_vault` y de `authority`, y de nada más — y los mantiene vivos hasta que la llamada a la CPI lo consume. Desde el momento en que `cpi` existe hasta el momento en que `transfer(cpi, ...)` se lo come, esas dos cuentas están bloqueadas y todos los demás campos de `ctx.accounts` son tan legibles como siempre fueron. Eso es lo que te estaba diciendo el build verde del comienzo de esta lección: `state` nunca entró a la CPI, así que leer `state.credit` no choca con nada. Llámalo exclusión por borrow checker, y dilo con precisión: un handle vivo *excluye* el acceso conflictivo a la cuenta que él toma prestada, exactamente mientras viva — un handle mutable excluye tus lecturas, y cualquier handle excluye tus escrituras.

Si te sirve, piensa en cada handle como sacar prestado un libro específico de una biblioteca. Mientras un libro está prestado a la CPI, nadie más puede leer ese libro — pero el resto de la biblioteca sigue abierta — y en el momento en que el libro vuelve, lo que sacas es la edición actual, no una fotocopia que hiciste la semana pasada. La analogía carga la parte importante, que el préstamo es exclusivo y acotado en el tiempo, y se rompe en un lugar que vale la pena señalar: un libro de biblioteca es un objeto físico, mientras que el borrow de acá se impone enteramente en tiempo de compilación, antes de que corra una sola instrucción. Nada queda bloqueado en tiempo de ejecución. El compilador simplemente se niega a emitir un programa en el que los dos se superpongan, así que el "conflicto" nunca es una carrera, es un fallo de build. Quédate con la exclusividad de la analogía y descarta lo físico.

![Una línea de tiempo de una sola instrucción donde los datos tipados de la cuenta que entró a la CPI son legibles, después quedan excluidos durante el tramo en que su CpiHandle mutable está vivo, y después vuelven a ser legibles en cuanto el handle se suelta — mientras las cuentas fuera de la CPI siguen legibles todo el tiempo.](assets/v04-diagram.png)

Ahora mira la lección pasada con ojos nuevos. ¿Te acuerdas de lo primerísimo que hizo el handler de withdraw? Copió valores *hacia afuera* de `ctx.accounts` antes de construir ningún handle:

```rust
let owner = *ctx.accounts.authority.address();  // copied out FIRST — the `*` makes it a copy
let sol_bump = ctx.accounts.state.sol_bump;     // a plain u8 copy
```

Entonces te dije nada más que el orden era estructural y no estilístico, y te prometí la razón para esta lección. Acá está, y es más angosta y más filosa que "`ctx.accounts` está prohibido." Mira la primera línea. `.address()` no te entrega un valor en propiedad; devuelve `&Address`, un borrow de `ctx.accounts.authority`. El `*` lo desreferencia y lo convierte en una copia simple en tu stack, y el borrow termina en el punto y coma. Quita el `*` y `owner` se queda como una referencia dentro de `ctx.accounts.authority` — que tu array de signer-seeds después lleva hasta la CPI misma. En el momento en que `cpi_handle_mut()` pide su borrow mutable de ese mismo campo `authority`, el compilador se niega: E0502, borrow inmutable todavía vivo, borrow mutable pedido, misma cuenta. La copia hacia afuera es estructural porque `authority` es una de las cuentas que toma esta CPI. La segunda línea, en cambio, es convención y no ley: `state` nunca entra a esta CPI, así que el compilador aceptaría esa lectura en cualquier parte del handler. Mantenerla arriba sigue siendo el hábito correcto — capturas primero, handles en el medio, lecturas post-CPI después — pero sabe cuál línea impone el compilador y cuál es estilo.

Y fíjate qué te compra esto en el otro extremo. En v1 leías después de la CPI y obtenías una copia obsoleta salvo que llamaras a `.reload()`. En V2 no hay `.reload()`, porque no hay nada que recargar.

Esa afirmación se apoya en el módulo 2, así que junta los dos hechos en vez de tomarlo por fe. La lectura obsoleta de v1 existía porque `Account<T>` deserializaba una *copia* al comienzo de la instrucción: tu handler sostenía una struct en propiedad en el stack, y una CPI que mutaba los bytes on-chain no tenía forma de llegar a ella. El `Account<T>` de V2 es una vista zero-copy sobre el buffer vivo de la cuenta, así que `ctx.accounts.state.credit` no es un campo de una copia, es una lectura por offset sobre los bytes que la CPI acaba de escribir. No hay ninguna segunda copia en ninguna parte que pueda quedar obsoleta. Por eso `.reload()` se pudo borrar en vez de reemplazar: el modelo de borrow no refresca tus datos, controla *cuándo* tienes permitido leer, y el modelo de cuentas zero-copy es lo que hace que la lectura que entonces tienes permitida ya esté fresca. Dos mitades de la misma tesis, y acá es donde se encuentran.

### El bloqueo es exacto, y lo exacto es el punto

Un lector filoso levanta una objeción acá, y es buena, así que vamos a responderla en vez de esquivarla. Si el punto entero es la seguridad, ¿por qué el bloqueo cae *solo* sobre las cuentas entregadas a la CPI? Cuando el handle del vault de SOL está vivo todavía puedes leer la cuenta `state` — el build verde del comienzo de esta lección lo demostró. ¿No debería el framework bloquear todo `ctx.accounts`, nada más por si acaso? Un bloqueo de toda la struct suena como la elección paranoica, y lo paranoico suele ganar en un camino de custodia.

No — y resolver por qué afila el modelo entero. Pregunta qué compraría de verdad el bloqueo más ancho. Una CPI solo puede tocar las cuentas que le entregaste: el runtime le pasa al callee la lista de cuentas de la instrucción y nada más, así que una cuenta que está fuera del `CpiContext` no puede ser cambiada por esa llamada, lo que quiere decir que una lectura de ella a mitad de la CPI no puede quedar obsoleta *por esa llamada*. Bloquear `state` mientras el handle del vault está vivo no prevendría ni un solo bug; solo te forzaría a contorsionar código correcto. La exclusión que V2 impone de verdad cae precisamente sobre las cuentas cuya copia tipada habría quedado obsoleta en v1 — las que la CPI puede escribir — y sobre ninguna otra. Incluso divide más fino que eso: una cuenta que la CPI solo *lee* entra por un `cpi_handle()` compartido, y los borrows compartidos toleran tus lecturas, así que el mint que le pasas a una transferencia de tokens sigue legible mientras el vault que está al lado queda bloqueado. La exclusión cae exactamente sobre las cuentas que la CPI puede escribir, y sobre ninguna otra.

Y fíjate qué *no* tuvo que construir V2 para conseguir esa precisión. Un framework que inventara su propio análisis de "a qué cuentas puede llegar esta CPI" tendría que seguir cada cuenta por cada alias, cada helper, cada rama — y un alias que se le escapara sería una fuga, una lectura obsoleta dejada pasar con un check verde. V2 esquiva el problema entero al no inventar ningún análisis. `Context` declara `accounts` como un campo común; tu struct de cuentas es una struct común; un `CpiHandle` es un borrow común de un campo. El "análisis" es el propio borrow checking por lugar de Rust, las mismas reglas que gobiernan cada struct en cada programa de Rust, endurecidas por una década del ecosistema entero apoyándose en ellas. El framework consigue exactitud *y* ausencia de fugas en una sola jugada, arreglando sus tipos para que el lenguaje haga la imposición.

![El bloqueo de borrow cae solo sobre las cuentas que están dentro del CpiContext: el vault bajo un handle mutable rechaza lecturas, mientras la cuenta state disjunta y el mint de handle compartido siguen legibles.](assets/v05-comparison.png)

Este es un instinto recurrente de V2, así que vale la pena nombrarlo como una regla que puedas llevarte: no construyas a mano una garantía que el sistema de tipos te va a dar gratis. Un bloqueo hecho a medida — grueso o ingenioso — es código de framework que alguien tiene que acertar y mantener acertado para siempre. Un borrow es una regla del lenguaje que el compilador ya acierta en cada build. Cuando diseñes tus propias APIs tienes disponible la misma jugada: da forma a tus tipos para que el invariante caiga solo de las reglas ordinarias de borrow, y el compilador hace la imposición.

### El borde exacto de la garantía

Acá es donde una explicación perezosa te diría que ahora el compilador se encarga de la frescura de las cuentas y te mandaría por tu camino. No te lo creas, porque es falso de una manera que te va a morder. Conocer el borde preciso de una garantía de seguridad es parte de usarla bien, así que vamos a trazar la línea exactamente.

El compilador protege el *acceso tipado a las cuentas*, sobre las cuentas que un handle vivo toma prestadas. Eso es todo. Si esquivas la capa tipada y lees directo los lamports crudos de `AccountInfo`, o sacas bytes a mano del buffer de datos que lleva una cuenta, el modelo de borrow no te protege. Esas lecturas son tuyas para razonarlas, exactamente como lo eran en v1. Un compañero que te dice que el borrow checker quiere decir que nunca más piensas en la frescura está equivocado en las dos cosas: no refresca nada, y no cubre las lecturas crudas.

![Una lectura tipada de una cuenta entregada a la CPI queda excluida mientras el handle de esa cuenta está vivo, una lectura tipada de una cuenta fuera de la CPI no tiene nada contra lo que protegerse, y los lamports crudos de AccountInfo o las lecturas manuales de bytes siguen siendo responsabilidad del programador exactamente como en v1.](assets/v06-comparison.png)

Y nombra el canje con honestidad, porque lo hay. El modelo de borrow compra seguridad en tiempo de compilación con un poco de flexibilidad. Un puñado de patrones ergonómicos de v1 — los que leen una cuenta a mitad del armado de la mismísima CPI que la toma — ahora necesitan reestructurarse: sueltas el handle, después lees. El costo es un refactor mecánico, normalmente mover una línea unas líneas hacia abajo. Esa es toda la cuenta. Canjeas "puedo escribir el código en cualquier orden" por "el orden en que tengo permitido escribir no puede ser el equivocado". En un camino que mueve el dinero de otra gente, ese es un canje que acepto todas y cada una de las veces. Un seguro barato contra un bug silencioso es el mejor tipo de seguro.

La duda que suele seguir es justa: ¿y si de verdad necesito un valor a mitad de la CPI, algo que tengo que leer mientras la llamada se está armando? Casi siempre, no lo necesitas, solo crees que sí por costumbre de v1. Si el valor viene de una cuenta que la CPI toma, captúralo *antes* de construir el handle, exactamente como la lección pasada capturó `owner` al comienzo — y captúralo como una copia simple, que es para lo que está el `*` delante de `.address()`. Una copia en tu stack no toma prestado nada, así que ningún handle puede chocar nunca con ella; una referencia sostenida dentro de la cuenta es un borrow, y va a chocar con el handle de esa cuenta en el momento en que se construya uno. Y si lo que quieres es el estado de la cuenta *después* de la CPI, eso no es una lectura a mitad de la CPI para nada, es una lectura post-CPI, y va debajo de la línea que suelta el handle, donde va a estar fresca de todas formas. El patrón que no tiene respuesta limpia, leer los datos tipados vivos de una cuenta en el instante preciso en que se compromete a una CPI, es el patrón que tampoco tenía respuesta correcta en v1. V2 nada más deja de pretender que la tenía.

### La tesis, y el framework exigiéndosela a sí mismo

Aléjate un segundo, porque esto no es un truco ingenioso, es una visión del mundo. El manifiesto #4390 argumentó que el `Account<T>` como camino lento no era una nota al pie de rendimiento, era la falla central: la cosa segura y la cosa rápida se habían separado, así que la gente pagaba un impuesto por la seguridad y algunos dejaron de pagarlo. La respuesta de V2 fue hacer que el camino seguro fuera el camino rápido y que el camino rápido fuera el default. El modelo de borrow es esa misma tesis llevada un nivel más arriba, a la composición. En vez de hacer barata una lectura post-CPI segura, hace imposible una insegura. Mismo instinto, palanca distinta: convertir una clase entera de bug en un error de compilación en vez de en un lint o en una línea en los docs.

![Una línea de tiempo que va desde el manifiesto del issue 4390, pasando por Anchor 1.0.0, hasta el modelo de borrow que quita .reload() y el fuzzing que encontró cuatro bugs del framework.](assets/v07-timeline.png)

Ese último tramo vale más que una nota al pie. Un framework que te entrega garantías de tiempo de compilación tendría que ganárselas él mismo, y este hizo el trabajo. El changelog de V2 le acredita al fuzzing haber encontrado cuatro bugs de corrección en el propio código del framework, rastreados en #4431, y la suite de pruebas entrega testigos de Miri y configs de Kani. Miri atrapa comportamiento indefinido en código unsafe; Kani demuestra que las propiedades valen para todas las entradas, no solo para las que a un autor de pruebas se le ocurrieron. El framework se somete a la misma disciplina de "demuéstralo, no lo esperes" que ahora le impone a tu programa. Cuando una herramienta te dice que confíes en el sistema de tipos, ese es el recibo que quieres ver detrás.

Un último ensanchamiento, porque el modelo de borrow es una sola instancia de un patrón que vas a ver por todo V2, y detectar el patrón vale más que memorizar esta regla. La línea que atraviesa todo es una preferencia por mover los fallos más temprano y más fuerte. Una lectura obsoleta que antes aparecía en tiempo de ejecución, en silencio, en un camino, en producción, ahora aparece en tiempo de compilación, a los gritos, en todos los caminos, en tu máquina. Esa es la misma jugada que reemplazar un `require!` de tiempo de ejecución con un tipo que no puede guardar un estado inválido, o una verificación escrita a mano con un constraint que la macro impone. Cada una toma una clase de "tenías que acordarte" y la convierte en "no te puedes olvidar". Una prueba demuestra que tu código funciona con las entradas que probaste; una demostración al estilo Kani y una regla de borrow funcionan con las entradas que no. Cuando evalúes cualquier garantía de framework de acá en adelante, ese es el eje sobre el que calificarla: si atrapa el bug cuando es barato arreglarlo o cuando es caro, y si puedes hacerle opt-out sin querer.

## Lab: haz que el borrow checker te detenga

Vas a correr dos experimentos. El primero, sobre el vault de la lección pasada, demuestra qué *no* cubre el bloqueo, porque la mitad de usar bien una garantía es saber dónde no está. El segundo te mete en el error de verdad, sobre la forma de cuenta que le ganó a v1 sus cicatrices, y lo arreglas reordenando. El único artefacto nuevo es un crate de sonda de borrador, que te lleva por el Challenge y se borra después. Primero, asegúrate de estar en el toolchain correcto, porque nada de esto vale en la línea V1.

**Paso 1. Fija el toolchain de V2.** Una línea:

```bash
anchor --version   # must report the V2 line (2.0.0-rc.1 as of 2026-08-12), not 1.1.2
```

Si reporta 1.x, vuelve a fijarlo con el bloque de instalación de m01-l2 — `--tag v2.0.0-rc.1`, `--locked` — y acuérdate de por qué existe el desvío: `avm install` baja un binario precompilado desde el GitHub Release del tag, no se cortó ningún Release para el tag v2, así que la descarga da 404. Vuelve a revisar si hay una rc más nueva antes de compilar, y fija lo que reporte `anchor --version` en tu `Anchor.toml` y en CI, así un compañero compila el mismo bytecode que compilaste tú.

**Paso 2. Demuestra que el bloqueo es exacto.** Abre el handler `withdraw` de la lección pasada. Acá está la forma, con la lectura de sonda que pusiste al comienzo de la lección, sentada en el medio del armado de la CPI, mientras los handles están vivos:

```rust
pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
    // --- the guard block from last lesson stays EXACTLY as you wrote it ---
    // require!(amount > 0, ...), require!(amount <= vault_lamports, ...),
    // and the checked remainder against the rent-exempt floor. Those three
    // lines are the security boundary of this handler; nothing in this lesson
    // touches them, and deleting them to shorten the snippet is how a teaching
    // edit becomes a custody bug. Elided below only for length.

    let owner = *ctx.accounts.authority.address();
    let sol_bump = ctx.accounts.state.sol_bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"sol", owner.as_ref(), &[sol_bump]]];

    let cpi = CpiContext::new(
        ctx.accounts.system_program.address(),
        Transfer {
            from: ctx.accounts.sol_vault.cpi_handle_mut(),
            to:   ctx.accounts.authority.cpi_handle_mut(),
        },
    )
    .with_signer(signer_seeds);

    // The probe: typed account data, read while `cpi` (holding live handles) is alive.
    let before = ctx.accounts.state.credit;   // <-- compiles: state is not in this CPI

    transfer(cpi, amount)?;

    let state = &mut ctx.accounts.state;
    state.credit = state.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;

    let _ = before;
    Ok(())
}
```

Corre `anchor build`. Verde. Quédate con eso, porque ese es el hallazgo: la CPI toma `sol_vault` y `authority`, `state` no es ninguna de las dos, y el compilador deja pasar la lectura a propósito. Después mira *por qué este programa no te puede mostrar el error de la clase reload en absoluto*: las dos cuentas que esta CPI sí toma son un `SystemAccount` y un `Signer`, y ninguna de las dos carga datos tipados que leer. La CPI de R2 no tiene nada que pudieras leer obsoleto, así que el modelo de borrow no tiene ninguna lectura obsoleta que prohibir. Eso no es un hueco en la lección; es el bloqueo trazando el peligro exactamente. Borra la línea de la sonda y deja R2 como la encontraste.

**Paso 3. Construye la sonda donde el bloqueo muerde.** Para encontrarte con el error necesitas una cuenta que tenga datos tipados *y* que entre a una CPI de forma mutable — que es precisamente la forma para la que se inventó el `.reload()` de v1: un vault de tokens cuyo `amount` revisas alrededor de una transferencia. Los tokens llegan formalmente en el módulo 5; hoy no estás construyendo infraestructura de tokens, nada más tomando prestado un tipo de cuenta para una sonda de diez minutos, así que cuando el vault se gradúe a SPL en m05-l1 ya te vas a haber encontrado con su filo más afilado. Haz un crate de borrador, y hazlo **fuera** de tu workspace del arcade — un `cargo new` corrido en la raíz de un workspace registra el paquete nuevo como miembro, y una sonda de diez minutos no tiene nada que hacer en el lock que comparten tus peldaños:

```bash
cd ..   # out of the arcade workspace first
cargo new borrow_probe --lib && cd borrow_probe
```

Apunta su `Cargo.toml` a la línea V2, con los pins que lleva cada crate de programa de este curso. Estando solo, toma el pin exacto de `solana-address` en vez del techo de workspace que usan los peldaños:

```toml
[package]
name = "borrow_probe"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib"]

[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io, not the branch: see m01-l2
anchor-spl  = "2.0.0-rc.1"     # anchor-lang and anchor-spl move together on the V2 line
# The pins from m01-l2 (issue #4937's class). The exact =2.6.0 suits this throwaway
# probe; the arcade workspace crates ride the ">=2.6.1, <2.7" ceiling from m02-l1
# instead, because a workspace resolves one solana-address for all of its members.
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

Y haz que `src/lib.rs` sea exactamente esto — una cuenta de token vault, un `transfer_checked` que sale de ella, y la verificación de saldo donde la memoria muscular de v1 la pone:

```rust
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod borrow_probe {
    use super::*;

    pub fn payout(ctx: &mut Context<Payout>, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            ctx.accounts.token_program.address(),
            TransferChecked {
                from: ctx.accounts.vault_ta.cpi_handle_mut(),
                mint: ctx.accounts.mint.cpi_handle(),
                to: ctx.accounts.recipient_ta.cpi_handle_mut(),
                authority: ctx.accounts.authority.cpi_handle(),
            },
        );

        // v1 muscle memory: check the balance while the CPI is being assembled.
        let before = ctx.accounts.vault_ta.amount();

        token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;

        let _ = before;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payout {
    pub authority: Signer,
    pub mint: InterfaceAccount<Mint>,
    #[account(mut)]
    pub vault_ta: InterfaceAccount<TokenAccount>,
    #[account(mut)]
    pub recipient_ta: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}
```

**Paso 4. Lee el error.** Corre `cargo build`. Pasada una página de advertencias de macro `unexpected cfg` que emite la RC, lo que obtienes es un rechazo del borrow checker, no una advertencia de lógica. Esta es la salida real de rustc:

```text
error[E0502]: cannot borrow `ctx.accounts.vault_ta` as immutable because it is also borrowed as mutable
  --> src/lib.rs:22:22
   |
14 |                 from: ctx.accounts.vault_ta.cpi_handle_mut(),
   |                       --------------------- mutable borrow occurs here
...
22 |         let before = ctx.accounts.vault_ta.amount();
   |                      ^^^^^^^^^^^^^^^^^^^^^ immutable borrow occurs here
23 |
24 |         token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;
   |                                           --- mutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

Lee lo que te está diciendo, porque te está diciendo la verdad, y fíjate en la primera línea antes que en cualquier otra cosa: el compilador nombra `ctx.accounts.vault_ta` — la cuenta, no la struct. La línea 14 es donde `cpi_handle_mut()` tomó el borrow mutable del vault. La línea 24 es donde `transfer_checked` consume `cpi`, así que el borrow tiene que seguir vivo hasta ahí. Tu lectura en la línea 22 pide un borrow compartido de la misma cuenta en el hueco del medio. Dos borrows conflictivos de una cuenta, un mismo tramo, no hay build. Y mira de qué *no* se quejó: la mismísima forma de expresión sobre `mint` — `ctx.accounts.mint.decimals()` en la línea 24 — pasa sin problema, porque el mint entró a la CPI por un `cpi_handle()` compartido y los borrows compartidos toleran lecturas. El compilador no está adivinando que tu lectura podría estar obsoleta. Volvió la lectura-durante-mutación estructuralmente inexpresable, para exactamente la cuenta que se está mutando.

![La lectura que falla de vault_ta.amount() está arriba de la llamada a transfer_checked, y moverla abajo, donde el handle se suelta, compila y lee la cuenta viva.](assets/v08-annotated-code.png)

**Paso 5. Arréglalo y demuéstralo.** Mueve la lectura debajo de la línea de `transfer_checked`, exactamente como muestra el panel AFTER. Recompila:

```bash
cargo build
```

Verde. Checkpoint: la sonda compila con la lectura después de la CPI, y R2 — que dejaste sin tocar después del Paso 2 — sigue compilando por sí solo. No agregaste ningún `.reload()`. No clonaste nada. Moviste una lectura al otro lado de donde el handle se suelta, y el borrow checker dio el visto bueno. Ese reordenamiento *es* la forma idiomática de V2. El handle existe para la CPI y solo para la CPI; en cuanto se suelta, los datos vivos son tuyos otra vez. Quédate con el crate de la sonda para el Challenge; está por crecerle un segundo handler.

Una cosa para internalizar antes del Challenge: el compilador no te detuvo porque tu número estuviera obsoleto. Te detuvo porque leíste la única cuenta sobre la que la CPI tenía el derecho de cambiar, en el tramo en que tenía ese derecho. En v1 la misma lectura compilaba y te entregaba el pasado. Lo estricto en tiempo de compilación es una conversación. Lo silencioso en tiempo de ejecución es un incidente.

## Challenge: arréglalo y nombra la clase de bug

Sin apoyo esta vez, y dos peldaños. El primero es otro handler roto para el programa de sonda que construiste en el Lab: `refund` paga tokens de vuelta desde el vault y quiere emitir el saldo que el reembolso *deja atrás*, pero lee ese número en el lugar equivocado, así que no va a compilar. A diferencia del Lab, el arreglo es todo tuyo. El segundo peldaño es calificado, se sienta debajo de esta sección, y pide una cosa que el arreglo no.

El handler necesita tres declaraciones que todavía no tienes, así que tómalas como dadas y agrégalas al programa de sonda. El evento es la forma `#[event]` simple de m01-l4, y el struct de cuentas es el `Payout` del Lab con el lugar del receptor renombrado:

```rust
#[event]
pub struct Refunded {
    pub authority: Address,
    pub remaining: u64,
}

#[derive(Accounts)]
pub struct Refund {
    pub authority: Signer,
    pub mint: InterfaceAccount<Mint>,
    #[account(mut)]
    pub vault_ta: InterfaceAccount<TokenAccount>,
    #[account(mut)]
    pub authority_ta: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[error_code]
pub enum ProbeError {
    #[msg("refund exceeds the vault's token balance")]
    Overdraw,
}
```

(Si a tu crate de sonda le faltan las filas `wincode` y `solana-address` del Paso 3, `#[event]` falla justo acá con `Address: SchemaWrite<...> is not satisfied`, mucho antes de que llegues a ver el error de borrow.)

Y acá está el handler roto:

```rust
pub fn refund(ctx: &mut Context<Refund>, amount: u64) -> Result<()> {
    let owner = *ctx.accounts.authority.address();

    // Legal: no handle exists yet, so this typed read is free.
    require!(
        amount <= ctx.accounts.vault_ta.amount(),
        ProbeError::Overdraw
    );

    let cpi = CpiContext::new(
        ctx.accounts.token_program.address(),
        TransferChecked {
            from: ctx.accounts.vault_ta.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.authority_ta.cpi_handle_mut(),
            authority: ctx.accounts.authority.cpi_handle(),
        },
    );

    // The event wants the post-refund balance. This read is in the wrong place.
    let remaining = ctx.accounts.vault_ta.amount();

    token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;

    emit!(Refunded { authority: owner, remaining });
    Ok(())
}
```

Fíjate, antes de tocarlo, que este handler ya lee `vault_ta.amount()` una vez, legalmente: la guarda de sobre-reembolso de arriba corre antes de que exista ningún handle. La misma lectura más abajo en el handler, después de que el `CpiContext` está construido, es la que el compilador rechaza — y aun si compilara, sería el saldo *pre*-reembolso, el número equivocado para un evento que promete el saldo que queda atrás. El error de borrow y el error de lógica son la misma línea, y una sola jugada arregla los dos.

Se requieren dos cosas para pasar.

1. Reordena el código para que compile y sea correcto: la lectura de `remaining` tiene que caer después de que `transfer_checked` consume `cpi`, así los handles están muertos y `remaining` es el verdadero saldo post-reembolso — fresco por construcción, porque `Account<T>` lee los bytes vivos. El `emit!` usa ese valor.
2. En una oración, nombra la clase de bug que V2 eliminó acá, la que v1 apenas compilaba y entregaba.

Criterios de aceptación que la revisión verifica directo:

- el handler compila en el toolchain de V2
- `remaining` se lee estrictamente después de que `transfer_checked` consume `cpi`, así refleja el saldo post-reembolso y no el pre-reembolso
- la guarda de sobre-reembolso mantiene su lugar arriba del `CpiContext`, donde la lectura es legal — una línea después de que los handles se activan no lo sería
- no aparece ningún `.reload()` en ninguna parte (no existe en V2, y echar mano de él es la trampa de la memoria muscular)
- tu respuesta de una oración nombra la clase eliminada: una lectura obsoleta post-CPI, silenciosa, donde el programa lee la copia pre-CPI de una cuenta después de una llamada entre programas y actúa sobre un valor que la cadena ya cambió, la clase que v1 te exigía acordarte de `.reload()` para evitar.

![Si un CpiHandle de la cuenta sigue en scope todavía no puedes leerla, así que suéltalo consumiendo el CpiContext, y después lee el campo directo porque .reload() ya no existe.](assets/v09-flowchart.png)

Si puedes enunciar la clase en una oración y tu reordenamiento compila, dominas el concepto, no nada más el arreglo. Lo que deja el segundo peldaño, el calificado. Es un handler `settle` recortado a Rust simple: sustitutos hechos a mano para la cuenta, el handle y el `CpiContext`, lo bastante chicos para compilar sin framework en la foto y tomando prestado exactamente como lo hacen los de verdad. Le debe un recibo — el saldo del vault *antes* de la transferencia y su saldo *después*, los dos leídos de la cuenta y no deducidos de los argumentos. El starter estaciona las dos lecturas en el tramo prohibido, así que no compila, y el apoyo impone la regla de "leído, no derivado" de la misma manera: justo después de que se construyen las cuentas, sombrea `vault_start`, `recipient_start` y `amount` en valores unitarios, así un recibo escrito a partir de aritmética sobre los argumentos también es un error de tipos. La calificación para Rust es solo-por-compilación, y el sombreado hace la mayor parte de ese trabajo: una lectura mal ubicada es un `E0502`, y un recibo derivado de `vault_start`, `recipient_start` o `amount` es un error de tipos. Sé honesto sobre el hueco que deja, porque una cerca que crees cerrada es peor que una que sabes abierta — `transfer_amount` sobrevive al sombreado como un `u64` vivo, así que `opening - transfer_amount` compila limpio y hasta devuelve el par correcto. Nada te impide escribirlo salvo saber por qué la lectura es la cosa que se está enseñando. La calificación solo-por-compilación cerca las formas que puede ver, y esta es una buena lección sobre cuáles son esas formas.

## Dónde te deja esto

Tómate la victoria. Acabas de ver al borrow checker negarse a compilar un bug que antes se entregaba por cientos en programas Anchor de producción, y lo arreglaste moviendo una línea. Es una sensación rara y buena: el compilador atrapó algo que antes requería una revisión de código, un revisor cuidadoso y un poco de suerte. La clase de lectura-obsoleta-después-de-CPI no es algo que ahora evitas. Es algo que ya no puedes escribir.

Mantén el borde bien afilado en la cabeza, porque es la parte que la gente se equivoca, en las dos direcciones. La garantía cubre solo el acceso tipado a las cuentas — los lamports crudos de `AccountInfo` y las lecturas de bytes hechas a mano siguen siendo tuyas para razonarlas — y cubre solo las cuentas que un handle vivo de verdad toma prestadas: todo lo que no le entregaste a la CPI sigue legible todo el tiempo, que es por qué la sonda de R2 compiló verde. El modelo de borrow no refresca nada, controla cuándo tienes permitido leer para que la lectura que tienes permitida sea fresca. Si tu arreglo compiló, cumpliste el criterio. Si no compiló, el culpable es casi siempre una lectura que sigue sentada arriba de la línea que suelta el handle: muévela hacia abajo, pasando la llamada que consume el `CpiContext`, y prueba otra vez.

Acá está el set de diagnóstico para llevarte de esta lección, porque la próxima vez que un error de borrow te devuelva la mirada durante una CPI, tres preguntas lo resuelven siempre. ¿Esta lectura es de un campo tipado de una cuenta, o de lamports o bytes crudos? Si es cruda, el compilador no es el que te está deteniendo y la frescura corre por tu lado. Si es tipada, ¿sigue habiendo un `CpiHandle` de esas cuentas en scope en esta línea? Si sí, ese es el error entero, y el arreglo es terminar el scope del handle antes de la lectura, no recurrir a nada nuevo. ¿Y quiero el estado de la cuenta antes de la CPI o después? Antes quiere decir capturar una copia en un local arriba; después quiere decir leer por debajo de donde se suelta, donde ya está viva. Corre esas tres y el borrow checker deja de ser una pared y se vuelve una lista de chequeo.

Hay una pregunta siguiente natural escondida en todo esto. Si un programa firmando por sí mismo es custodia, ¿qué pasa cuando un pago depende de dos partes y una condición, y el depósito necesita vivir dentro de un vault que ya construiste? Eso es composición, y es donde el handle con borrow rastreado deja de ser una regla de seguridad y empieza a ser la cosa que le deja a un programa construir con seguridad sobre el estado de otro. La lección que viene construyes R3, el prize-escrow: reserva un depósito dentro de una instancia real del quarter-vault de R2 a través de una CPI trabajada, y libera el premio solo cuando se cumple la condición de victoria y quien llama pasa el control. El escrow confía en el vault, y V2 hace de esa confianza algo que el compilador te ayuda a mantener.

Lo rompiste, lo arreglaste, lo nombraste. Feliz construcción.
