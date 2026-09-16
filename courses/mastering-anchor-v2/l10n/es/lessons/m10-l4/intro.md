# Conclusión: qué construiste, y ¿deberías moverte a V2 hoy?

La lección pasada tomaste un vault 0.31/1.0 de verdad y lo llevaste hasta un build V2 que compila y pasa en LiteSVM, siguiendo las advertencias del compilador como una lista de pendientes hasta que el port quedó verde. Eso era lo último que este curso tenía para enseñarte. Ya construiste Anchor V2 desde cero y migraste una base de código vieja hacia él. El salón de arcade está terminado: un cabinet-counter, un quarter-vault, un prize-escrow, un swap de token-a-ticket, y un floor-registry que les hace CPI a todos juntos. Probado, fuzzeado, perfilado, verificado.

Queda una cosa, y no es un lab. Elige un proyecto que de verdad te importe, ahora mismo. No uno de juguete. Algo con tu nombre puesto, o el de tu equipo. Escribe una línea sobre él en un post-it: ¿está sosteniendo valor real hoy, es un build nuevo sin nada en juego, o es una base de código grande que ya corre en la línea vieja? Deja ese post-it a tu lado. Al final de esta lección vas a rutearlo por un árbol de decisión, en voz alta, y vas a poder defender la respuesta desde los hechos de verdad. Ese es el trabajo entero de una conclusión que respeta tu tiempo. No una vuelta olímpica. Una sola decisión honesta que ahora puedes tomar solo.

## Qué hace esta lección, y qué entrega

Dos jugadas, y una negativa deliberada.

![Un mapa de eje y radios del plan de cierre: encajar los cinco programas en un solo modelo, correr el árbol de decisión de adopción, entregar las capas adyacentes a los cursos hermanos.](assets/v01-diagram.webp)

Primero, encaja todo lo que construiste en un solo modelo conectado, para que los cinco programas dejen de ser cinco ejercicios y se vuelvan una sola imagen de cómo piensa V2. Segundo, recorre el árbol de decisión de quien migra contra las tensiones reales del proyecto hoy, porque "V2 es mejor" y "deberías poner tu dinero de mainnet en V2 esta tarde" son dos preguntas distintas y solo una de ellas es fácil. La negativa: este curso es dueño de la capa de framework y de nada más, así que el cierre te apunta hacia afuera, hacia los cursos hermanos que son dueños de las capas sobre las que se apoya V2.

Acá está el repliegue de la ayuda, dicho sin adornos. Cada lección anterior te recorrió el constraint en el momento en que le pegaste. Esta no. Yo no voy a tomar la decisión de adopción por ti. Voy a entregarte el árbol y las entradas honestas, y después dar un paso atrás y dejar que sea tuya. Así se supone que se siente terminar un curso: el apoyo se baja y la cosa sigue de pie.

## El mapa entero, y después la sala que tienes que leer

No aprendiste cinco trucos sin relación. Aprendiste cuatro ideas, cuatro veces cada una, con nombres distintos. Alinéalas.

`Account<T>` es Pod zero-copy por defecto. Esa es la tesis de la issue #4390 de Anchor, la que reencuadró la reescritura entera: deja de deserializar la cuenta entera al heap en cada instrucción, y empieza a leer campos en el lugar desde un layout fijo. Tu cabinet-counter fue la prueba más chica posible de eso. Es también de donde vienen el ahorro de bytes y el ahorro de compute, porque el trabajo que antes pagabas en cada llamada simplemente ya no está ahí.

Las CPI son `CpiHandle`s con borrow rastreado. La trampa del reload — en V1, cualquier CPI que mutara una cuenta que todavía tenías tipada forzaba un `.reload()` o una lectura obsoleta silenciosa — se fue por construcción. El borrow checker ahora sabe que una cuenta cambió debajo de una CPI, así que el sistema de tipos carga la frescura que el código viejo cargaba en tu memoria. Tu floor-registry, llamando al vault y al swap, es donde eso dejó de ser una regla que tenías que recordar y se volvió una cosa que el compilador recuerda por ti.

Los bumps de PDA te llegan por una struct tipada que la macro construye en tiempo de expansión. Guardabas bumps canónicos en V1 para ahorrar las ~1500 unidades de cómputo que cuesta una re-derivación, y los sigues guardando en V2, porque una búsqueda sobre la clave de quien llama solo puede correr en la validación. Di el delta con precisión, porque este es el que la gente atribuye mal: el campo tipado *no* es eso. `ctx.bumps.vault` reemplazó al viejo `ctx.bumps.get("vault").unwrap()` con clave de string en Anchor **0.29**, dos majors antes de V2, y la línea 1.x contra la que mediste lo lee exactamente como lo lee V2. La noticia de bump de verdad de V2 es más angosta — cuando cada seed es un literal de byte en tiempo de compilación, el bump canónico se precomputa en tiempo de macro como un const, que es un caso que tu quarter-vault no toca. Así que lo que cargaste por el port acá fue continuidad, y la disciplina de CU siguió siendo tuya en los dos lados. Tu quarter-vault firmando por sí mismo es esa idea en carne y hueso.

Y tres clases nombradas de vulnerabilidad ahora son errores de compilación en vez de exploits de runtime. El módulo 7 puso números sobre exactamente cuáles: el type cosplay, el alias de duplicate-mutable, y la lectura obsoleta después de una CPI. Explotaste las tres sobre formas de v1 y después viste a los defaults de V2 negarse a construirlas. Mantén el alcance honesto, porque es el alcance que auditaste tú mismo: tres de las once clases de la taxonomía de la Foundation, no todas, y la sustitución de cuentas a través de una CPI sigue siendo tuya para validar a mano. Tu prize-escrow y tu swap son donde sentiste a esa barandilla empujar de vuelta, y donde encontraste su borde.

![Un mapa conceptual con Anchor V2 en el centro y cuatro radios: cuentas Pod-por-defecto, CPI con borrow rastreado, un const de bump para seeds todas literales encima de la struct tipada que 0.29 ya te dio, y seguridad en tiempo de compilación, cada uno cableado a un programa que construiste.](assets/v02-diagram.webp)

Fíjate en lo que el mapa te está diciendo de verdad. Estas no son cuatro features atornilladas, sino una sola decisión, aplicada de forma consistente: mover el trabajo que el desarrollador hacía en runtime, y en el que se equivocaba, hacia arriba, al sistema de tipos y a la generación de código. Ese es el hilo conductor de la reescritura entera.

### El framework que narra su propio por qué

Haz un zoom hacia afuera un clic más y puedes ver la trayectoria que produjo esto. El Anchor temprano que tal vez recuerdes era pesado en macros y hambriento de compute, comprando ergonomía de desarrollador con costo de runtime. La línea 1.0 estabilizó ese trato y lo puso bajo una tutela de verdad. La línea 2.0, el trabajo de anchor-next, volvió y pagó el costo que había asumido, empujando la ergonomía hacia el compilador y los bytes hacia un runtime más flaco.

![Una línea de tiempo de tres paradas: el Anchor temprano pesado en macros, la línea 1.1.2 estabilizada y todavía mantenida, y la línea 2.0 de candidata a release más flaca.](assets/v03-timeline.webp)

Un framework que revisa sus propios trade-offs en voz alta es raro, y es exactamente el tipo de cosa que quieres debajo de tus programas. Pero esa misma honestidad es la que vuelve difícil la pregunta de adopción, porque la honestidad se extiende a las partes que todavía no están terminadas.

### La tensión, dicha sin titubear

Acá es donde muchos textos se ponen alegres y dejan de ser útiles. Déjame no hacer eso.

El mismo release se llama a sí mismo dos cosas contradictorias. La versión es `2.0.0-rc.1`. "rc" quiere decir release candidate, que se lee como "casi listo, solo sacudiendo bugs". Pero el proyecto etiqueta ese mismísimo release como "alpha" en otro lado, que se lee como "temprano, espera movimiento". Las dos etiquetas, un artefacto. Eso son los mantenedores diciéndote, en dos palabras, que la cosa está genuinamente en el medio, y no un typo que tengas permitido redondear.

La documentación dice, con sus propias palabras, que V2 no está auditado. No "audit pending," no "audit in progress that we will link." No auditado. Para un framework cuyo pitch entero incluye matar clases de vulnerabilidad, esa es la frase más importante de la página.

No hay fecha comprometida para que la candidata a release se vuelva estable. No "Q4", no "el próximo trimestre". Al momento de escribir esto no hay ninguna fecha así publicada en ningún lado. La ausencia de una fecha no es una fecha corta. Es la ausencia de una promesa, y deberías leerla como exactamente eso y nada más suave.

Y la línea vieja no está quieta. Anchor 1.1.2 es el estable actual, y su línea sigue mantenida y entregando. Esta es la realidad de las dos líneas paralelas: un v1 en movimiento, auditado por el tiempo, probado en producción, al lado de un v2 más rápido, no auditado, sin fecha. No estás eligiendo entre una opción viva y una muerta; estás eligiendo entre dos opciones vivas con perfiles de riesgo opuestos.

Esa distinción importa más de lo que parece, porque cambia lo que te cuesta "esperar". Esperar en 1.1.2 no es lo mismo que estancarse en él. La línea sigue recibiendo arreglos, y sigue siguiéndole el paso al runtime a medida que la red misma cambia debajo de tu programa. Estás estacionado en un camino que todavía se está pavimentando, no varado en uno abandonado, y es exactamente por eso que la rama paciente del árbol es una opción de verdad y no un eufemismo para quedarse atrás.

![Un lado a lado de Anchor 1.1.2, estable y endurecido en producción, contra 2.0.0-rc.1, etiquetado tanto rc como alpha y explícitamente no auditado pero mucho más flaco en bytecode y CU.](assets/v04-comparison.webp)

### El número que se volvió más honesto

Ahora, sobre esos ahorros, porque son reales y deberías cargarlos correctamente.

Los propios benchmarks de V2 reportan alrededor de 94% menos bytecode desplegado y como una reducción promedio de 8.8x en unidades de cómputo. Esos son números grandes. También no son los números que el proyecto publicó primero. Un pull request, el #4914, aterrizó el 2026-08-13 y revisó las cifras del titular a la baja: 95% pasó a 94% en bytecode, y 9.9x pasó a 8.8x en compute. El proyecto hizo su propio alardeo más chico.

Quédate con eso un segundo, porque es la cosa más tranquilizadora de esta lección entera. Un proyecto que revisa sus números de marketing a la baja, a propósito, es un proyecto en el que puedes confiar más, no menos. Es "no confíes, verifica" aplicado por los mantenedores a sí mismos. La trampa es que también te dice que los números todavía pueden moverse, porque la página de benchmark lo dice igual: los valores se corren a medida que cambian la generación de código y el runtime pinocchio subyacente. Así que la forma correcta de cargarlos es como evidencia con reservas, direccional, re-verificable. V2 es dramáticamente más flaco. Esa es la afirmación. Un multiplicador congelado no lo es.

![Dos figuras emparejadas que muestran los benchmarks del titular revisados a la baja, el bytecode de 95% a 94% y el compute de 9.9x a 8.8x, según el PR #4914.](assets/v05-chart.webp)

Hay un hecho más que vale pesar antes de rutear cualquier cosa, y se apoya en la superficie de confianza y no en el código. Un solo guardián ahora custodia toda la cadena de suministro. OtterSec sostiene el framework, publica sus crates, corre el registry de builds verificados contra el que Anchor chequea los releases, y firmó con GPG el tag v2. Un guardián único, competente y enfocado en seguridad a través de los crates, del registry y de las firmas es un punto de verdad a favor de V2, y la última parte la puedes chequear tú mismo en vez de creerme. Lo vamos a hacer en el lab.

Un registry de builds verificados vale entenderlo, no solo anotarlo, porque te compra algo específico. Deja que cualquiera confirme que el bytecode que corre on-chain se produjo desde el código fuente que de verdad puedes leer, en vez de confiar en la palabra de un mantenedor de que los dos coinciden. Junta eso con tags de release firmados y crates publicados bajo un solo guardián, y la cadena de suministro que estás heredando es auditable de punta a punta. Mantén esto separado en tu cabeza de la auditoría que todavía se debe: puedes verificar la procedencia hoy, hasta la firma, mientras la revisión de seguridad de la lógica del framework todavía no pasó. Dos tipos distintos de confianza, y V2 se ganó exactamente uno de ellos hasta ahora.

## La única decisión que queda: el árbol de decisión de quien migra

Todo lo de arriba alimenta un solo diagrama de flujo. Las entradas son las mismas para cada proyecto: etiquetado de rc-y-alpha, no auditado, ninguna fecha estable comprometida, una línea v1 que todavía se mueve. Lo que cambia es tu proyecto, y eso cambia el ruteo.

![Un árbol de decisión que rutea un proyecto greenfield a sí, un protocolo vivo que sostiene valor a no-hoy, y una base de código v1 grande a mapea-ahora-y-porta-cuando-esté-estable-y-auditado.](assets/v06-flowchart.webp)

Lee cada rama como una frase que le podrías decir a un compañero escéptico.

Greenfield, aprendizaje, o un experimento sin nada en riesgo: construye sobre V2 hoy. Lo único expuesto es tu tiempo, las ganancias son inmediatas, y sales fluido en el framework en el que va a estar todo el mundo una vez que aterrice el estable. Esta es la rama que este curso entero te estaba preparando calladamente para tomar sin miedo.

Producción, sosteniendo valor real de usuario ahora mismo: quédate en 1.1.2. El ahorro de compute es genuino y todavía no compensa poner fondos de usuario sobre código no auditado, sin fecha, auto-etiquetado de alpha. Vigila dos señales específicas, una fecha estable comprometida y una auditoría publicada, y no trates ni la ganancia de CU ni un feature flag como sustituto de ellas. Correr la RC en producción detrás de un flag no resuelve el riesgo de auditoría. Lo esconde.

Base de código v1 grande ya existente: divide la decisión en dos. Mapea ahora, porta después. Haz el análisis de migración hoy, exactamente como el que corriste la lección pasada, para que conozcas tu superficie real de port y no te sorprenda. Después aprieta el gatillo cuando aterrice la fecha estable y exista una auditoría. El ensayo difícil ya lo hiciste; esta rama solo dice que no confundas el ensayo con la noche de estreno.

### Cuatro formas de leer mal el árbol

El árbol es tan bueno como la lectura, así que acá están las cuatro lecturas equivocadas que te van a rutear mal. Cada una es una trampa en la que he visto a gente cuidadosa caminar derecho.

La primera es leer "V2 es más rápido" como "V2 está listo para producción hoy". Esas son afirmaciones sin relación sentadas en ejes distintos. Los benchmarks tienen reservas de alpha y el código no está auditado, así que la velocidad es un argumento para construir tu próximo proyecto de aprendizaje sobre V2, y no es un argumento para mover fondos de usuario hacia él este trimestre. El árbol mantiene los dos ejes aparte a propósito, porque confundirlos es la forma más común en que un equipo de verdad se convence a sí mismo de un deploy malo.

La segunda es tratar la ausencia de una fecha estable como "pronto". Una fecha que falta no carga ninguna información sobre plazos. No es una cuenta regresiva que por casualidad no puedes ver. Planifica contra lo que está de verdad comprometido, que hoy es nada, y deja que una fecha publicada de verdad cambie tu plan cuando aparezca, en vez de dejar que tu esperanza lo cambie antes.

La tercera es oír "no existe ningún otro curso de V2" como una ley probada del universo. Es un resultado de relevamiento de una búsqueda exhausta, no un teorema. Material nuevo podría aterrizar la semana que viene y calladamente volver falsa la afirmación. Enúncialo como querrías que se enunciara una afirmación sobre tu propio trabajo: como lo que encontró una mirada cuidadosa y fechada, y como algo honestamente abierto a estar equivocado.

La cuarta, y la que calladamente cuesta más, es asumir que un tema que este curso se salteó es un tema que no importa. Cada omisión acá fue una entrega, no un veredicto. La interfaz de transfer hook, el aterrizaje de transacciones, el runtime debajo del loader: este curso rechazó cada uno de ellos porque un hermano nombrado es su dueño y lo enseña mejor de lo que jamás podría un capítulo atornillado. Salteado no es lo mismo que sin importancia, y confundir los dos es cómo terminas reconstruyendo, mal, una cosa que alguien ya enseñó bien.

![Una tabla que empareja cada una de las cuatro formas de leer mal el árbol de adopción con por qué está equivocada y la lectura corregida.](assets/v07-table.webp)

## Lab: rutea tres perfiles, y después verifica una firma

El lab es un lab de razonamiento, no un lab de código, con un comando de verdad al final. Trabájalo en orden. El repliegue de la ayuda quiere decir que yo te doy los perfiles y los checkpoints; las justificaciones son tuyas para escribir.

1. Toma tres perfiles de proyecto: un build de aprendizaje de fin de semana sin nada en juego, un protocolo vivo que sostiene fondos de usuario hoy, y una base de código v1 de 40,000 líneas en producción. Para cada uno, nombra la rama a la que rutea.

2. Para cada ruteo, escribe una frase de justificación que cite las tensiones de verdad por su nombre, no "se siente más seguro". Una buena justificación para el protocolo vivo se lee así: "no hoy, porque V2 está etiquetado tanto rc como alpha, la documentación dice que no está auditado, y ninguna fecha estable está comprometida, así que el valor en riesgo se queda en la línea 1.1.2 mantenida." Checkpoint: si tu justificación no nombra al menos dos de las cuatro tensiones congeladas, es una sensación, no una decisión. Reescríbela.

3. Ahora haz el de verdad. Toma el proyecto de tu post-it del comienzo de la lección y rutéalo. Escribe su justificación de la misma forma. Esta es la decisión que de verdad viniste a tomar acá. Checkpoint: el post-it ahora carga una rama (sí, no hoy, o mapea-ahora-porta-después) y una frase que nombra las tensiones que la fuerzan. Si la frase no sobreviviría a un compañero escéptico preguntando "¿por qué no el mes que viene en cambio?", no está terminada.

4. Confirma el toolchain en el que van a aterrizar tus ramas de "sí". Instalaste la RC allá en m10-l3; esto es una verificación, no una cuarta instalación:

```bash
anchor --version   # expect 2.0.0-rc.1 (the pin as of 2026-08-22; no stable date is committed)
```

Checkpoint: `anchor --version` imprime la string V2 que fijaste. Si imprime una versión 1.x, tu PATH está resolviendo el binario viejo primero; arregla el PATH, o reinstala con el comando git fijado de m10-l3, y vuelve a verificar antes de seguir.

5. Haz la jugada de "no confíes, verifica" sobre la superficie de confianza. OtterSec firmó con GPG el tag v2. Cada instalación de este curso pasó por `cargo install --git`, que no te deja ningún repositorio para inspeccionar, así que clona uno y chequea la firma en vez de asumirla:

```bash
git clone https://github.com/otter-sec/anchor.git anchor-src
cd anchor-src
git verify-tag v2.0.0-rc.1
```

Checkpoint: el comando reporta una firma buena una vez que la clave pública de OtterSec está en tu keyring. Si da error con "no public key", eso es esperable hasta que importes su clave. El punto no es que pase en el primer intento. El punto es que la firma sea chequeable, punto, que es la propiedad que quieres del guardián de tu cadena de suministro.

## Challenge: toma la decisión de memoria

Cierra las notas. Acá está la barrera.

Te entregan tres proyectos: un build de aprendizaje de fin de semana, un protocolo vivo que sostiene fondos de usuario, y una base de código v1 de 40,000 líneas. Ubica cada uno en el árbol de decisión de adopción de V2 y justifica el ruteo, de memoria, usando el etiquetado de RC-y-alpha, el estado no auditado, la fecha estable que falta, y la línea v1 todavía mantenida. Produce tres triples de proyecto-a-decisión-a-justificación.

Pasas cuando los tres ruteos son correctos y cada justificación nombra las tensiones específicas que la fuerzan, no una incomodidad general. Si puedes hacer eso sin mirar hacia atrás, puedes hacerlo en una reunión de planificación de verdad, que es el único lugar donde importa.

## Feedback, y dónde entrega la capa de framework

Un momento honesto antes de la puerta. Si el ruteo del protocolo vivo se sintió anticlimático, "la cosa nueva y rápida, y la respuesta es esperar", quédate con por qué no se sintió así al escribirlo. Recomendar paciencia para los fondos de usuario de otra persona, sobre alpha no auditado, es la cosa más optimista-de-builder de este curso, no la menos. Optimista en la tecnología, honesto en el riesgo. Esa es la postura entera.

Acá hay una cosa que vale saber, enmarcada como resultado de relevamiento y no como un absoluto. La ruta de aprendizaje oficial en solana.com/developers/courses ahora hace un redirect 308 hacia un repositorio de contenido para desarrolladores que se archivó y congeló el 2025-01-24. Al 2026-08 fuimos a mirar y no encontramos ningún otro curso de Anchor V2 en ningún lado. Esa es la razón por la que esta conclusión es una rampa de entrada y no un competidor: eres, tan lejos como una búsqueda exhausta puede decir, el que tiene el mapa actual de un lugar que casi nadie escribió todavía.

Este es el final del curso, así que el gancho hacia adelante apunta hacia afuera en vez de a una próxima lección. La capa de framework es tuya ahora, y es deliberadamente solo la capa de framework. Las cosas que este curso rechazó, las rechazó porque un hermano es su dueño y las enseña como se debe.

![Una tabla de entrega que rutea cada próximo tema al curso hermano que es su dueño: Digital Assets, Client-Side, Low-Level Solana, DeFi y RWA, y Payments y Commerce.](assets/v08-table.webp)

Cada uno de esos cursos construye exactamente sobre lo que acabas de aprender — la capa de framework es tuya ahora, y ellos se paran sobre ella. Dos de ellos ya están publicados, Digital Assets y Payments and Commerce; Client-Side, Low-Level Solana y DeFi and RWA Engineering son hermanos planeados todavía en producción, así que lee sus filas como un mapa de dónde vive cada tema, no como puertas para hacer clic ya. El curso de Digital Assets es dueño de los estándares de token que este curso tocó solo desde el asiento del programa, la interfaz de transfer hook incluida. El curso de Client-Side es dueño de hacer que una transacción de verdad aterrice y de leer datos de la blockchain de vuelta hacia afuera — la mitad de cliente entera que este curso nunca abrió ni una vez. Low-Level Solana va debajo del loader sobre el que estuviste parado todo este tiempo, hacia el sBPF y las syscalls, sin framework alguno. DeFi y RWA Engineering es dueño de lo que el diseño de protocolo de verdad y los venues de verdad exigen más allá del swap de juguete que escribiste como patrón de Anchor. Y Payments y Commerce convierte la pila entera en rieles sobre los que un negocio puede correr dinero. Cinco capas, cinco dueños, y uno de esos dueños, el de la capa de framework, ahora eres tú.

Terminaste el mapa. Construiste cada programa que hay en él, migraste una base de código de verdad hacia él, y ahora puedes mirar cualquier proyecto y decir, desde los hechos, si debería moverse hoy. Esa última habilidad es la que va a seguir siendo verdad después de que cambien los números de versión. Anda a tomar la decisión sobre tu post-it. Te ganaste la confianza para tomarla, y ya no me necesitas en la sala para revisar tu trabajo.
