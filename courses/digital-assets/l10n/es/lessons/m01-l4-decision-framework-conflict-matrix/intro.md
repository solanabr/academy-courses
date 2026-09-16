# SPL clásico vs Token-2022: el framework de decisión + la matriz de conflictos

## Resumen

En m01-l3 aprendiste que SPL clásico es una interfaz congelada sobre el nuevo motor p-token, y que toda capacidad genuinamente nueva vive en las extensiones de Token-2022. Lo que levanta la pregunta en la que toda conversación de producto termina aterrizando: "¿puede este mint tener una comisión de transferencia Y saldos confidenciales?" Esperarías una tabla en algún lugar de la documentación que la responda. No hay tabla completa ni tabla que se haga cumplir. Algunas páginas de documentación afirman cosas sobre pares individuales (vas a poner una de esas afirmaciones a juicio al final de esta lección), pero nada autoritativo, nada entero. La respuesta honesta vive en una función de Rust llamada `check_for_invalid_mint_extension_combinations`, y esta lección vas y la lees.

Empieza ahora, antes de la teoría. Clona la fuente y fíjala al commit del que se deriva toda esta lección:

```bash
git clone https://github.com/solana-program/token-2022.git
cd token-2022
git checkout 426400f
```

Abre `interface/src/extension/mod.rs` y baja hasta la línea 1326. Esa función, unas cincuenta líneas de Rust simple, es todo el código legal para las combinaciones de mint de Token-2022. Déjala abierta en un panel dividido. Todo lo que sigue es un recorrido por lo que ya estás mirando.

Dos cosas se construyen encima de esa lectura. Primero, el framework de decisión: cuándo los requisitos de un producto apuntan a SPL clásico y cuándo fuerzan Token-2022, planteado como una elección de CONJUNTO de extensiones y no como una preferencia de programa. Segundo, el artefacto emblemático de este curso: `check-combo`, un validador de TypeScript que aplica las cinco reglas de la fuente a cualquier conjunto de extensiones y devuelve un veredicto con la regla que se disparó. Consume el inspector `decode-mint` que construiste en m01-l2, lo que quiere decir que para el final de esta lección puedes apuntarlo a cualquier mint vivo y preguntar "¿este conjunto siquiera es legal, y por qué?"

El repliegue de la ayuda en esta lección: la lectura de las reglas de la fuente se trabaja completa, recorro cada línea contigo. El validador es un ejercicio de completar: la regla 4 ya viene implementada, las reglas 1, 2, 3 y 5 te toca portarlas a ti. Y el único coding challenge del módulo, al final, es en solitario: el validador completo comprobado con pruebas, sin apoyo.

Esta lección es una piedra angular. Cada mint que construyamos del Módulo 2 en adelante pasa por la barrera del artefacto que escribes hoy.

## Derivando la matriz

### El framework: qué conjunto de extensiones, no qué programa

La decisión que la gente discute suele estar mal planteada. "¿Deberíamos usar SPL clásico o Token-2022?" suena a preferencia de marca, como elegir proveedor de base de datos. No lo es. Y el rendimiento es un mal diferenciador para argumentar: el cambio a p-token del que leíste la lección pasada sacó de la mesa el viejo motor de SPL clásico, y los costos de cómputo por extensión de Token-2022 no están publicados, que es por lo que una lección posterior los mide en un lab en vez de citar una tabla. Hasta que hayas medido, el argumento que de verdad decide es exactamente una pregunta:

¿Algún requisito de tu producto nombra un comportamiento que solo una extensión provee?

Si no, SPL clásico gana por defecto. Es la cuenta más barata de crear, cada billetera la renderiza, cada DEX la enruta, cada integración jamás escrita la asume. Una interfaz congelada no es una limitación cuando tus requisitos caben adentro. Es una garantía.

Y acá lo más barato es medible, no retórico. Tu inspector de m01-l2 ya imprimió los números por ti: un mint clásico pelado mide 82 bytes, y en el momento en que existe una sola extensión el mint cambia al layout extendido, donde el mint extendido más chico posible que midieron tus propias aserciones de `mintLen` es de 170 bytes. Más o menos el doble, por una extensión, antes de que hayas configurado nada. Sube desde ahí con cada entrada TLV. El rent se cobra por bytes, y de todas formas el mint es la mitad barata de esa historia, porque las extensiones de cuenta forzadas que vemos más adelante en esta lección ponen bytes en cada cuenta de tenedor que llegues a crear. Unas pocas docenas de bytes por cien mil tenedores ya no es una preferencia de diseño, es una línea en una hoja de cálculo de tesorería. Así que ir por defecto a SPL clásico es la respuesta correcta siempre que la lista de requisitos no obligue a otra cosa, y "tal vez queramos una comisión algún día" no es un requisito.

Si sí, estás en Token-2022, y ahí empieza la decisión de verdad. Comisiones en la transferencia, display de interés, saldos confidenciales, transfer hooks, no transferibilidad, metadatos nativos, pausabilidad: cada una de esas palabras en una especificación de producto es un nombre de extensión disfrazado. En el momento en que aparece una, la pregunta deja de ser "qué programa" y pasa a ser "qué conjunto de extensiones", porque los slots de extensión de un mint se reclaman en la inicialización. La regla precisa, que el módulo 2 va a demostrar sobre mints vivos: toda extensión de poder en el lado del mint es solo de init — una comisión, un hook, un delegado no se pueden atornillar después. Las excepciones son estrechas y todas de una misma forma. El TLV de TokenMetadata se escribe después de `initialize_mint`, pero solo detrás de un MetadataPointer que elegiste en la creación; los TLV de grupo y de miembro de grupo recorren ese mismo camino de puntero-y-después-realloc detrás de sus propios punteros; y las extensiones a nivel de cuenta llegan a las cuentas de token de los tenedores vía reallocate, nunca al mint. Fíjate en que cada una de esas todavía empieza con un puntero elegido al nacer, y en que ninguna de ellas es una extensión de poder. Así que para fines de diseño el conjunto de extensiones de un mint es un certificado de nacimiento, no una página de configuración.

![Diagrama de flujo con un solo criterio sobre el comportamiento exclusivo de extensiones, SPL clásico como la rama del no, y una rama del sí que dibuja un conjunto de extensiones, lo valida contra las cinco reglas de la fuente y después inicializa para siempre.](assets/v01-flowchart.webp)

De esa permanencia se desprenden dos consecuencias, y le dan forma a toda la lección. Una: el conjunto de extensiones tiene que diseñarse por adelantado, contra la hoja de ruta completa del producto, porque "agregamos transferencias confidenciales el próximo trimestre" no existe. El arreglo para una extensión que falta es un mint nuevo y una migración, que es una lección entera de dolor en el Módulo 9. Dos: el conjunto tiene que ser LEGAL, y la legalidad la decide el programa en la inicialización, no tú y no la documentación. Algunas combinaciones se rechazan de plano. Algunas combinaciones obligan a que otras extensiones vengan con ellas. El mapa completo de esas interacciones es lo que este curso llama la matriz de conflictos.

¿Y dónde está la matriz? Acá viene la parte que le gana a esta lección su lugar en el módulo.

### Por qué no hay matriz que copiar

No hay matriz de conflictos oficial ni actual para los 29 tipos de extensión de Token-2022. Yo fui a buscarla, la pasada de investigación de este curso fue a buscarla, y el resultado honesto es: las cinco reglas que deciden la legalidad viven en `interface/src/extension/mod.rs` de `token-2022`, líneas 1326-1374, y en ningún otro lado (solana-program/token-2022 @ 426400f, un commit fechado 2026-08-17 y leído para esta lección el 2026-08-22).

Los catálogos que sí existen son páginas explicativas por extensión, y ninguna publica las reglas. Además se mueven bajo tus pies, cosa que puedo demostrar con una vergüenza propia. Cuando la pasada de investigación de este curso barrió los sitios de documentación el 2026-08-21, solana.com/docs/tokens/extensions no tenía página para PermissionedBurn, la variante más nueva, mientras que solana-program.com ya cargaba una. Escribí ese hueco en un borrador anterior de esta lección como el ejemplo estelar de la deriva de la documentación. Al volver a revisar las dos páginas el 2026-08-22 antes de entregar: solana.com lista Permissioned Burn en su barra lateral de extensiones, justo después de Pausable Mint. El hueco se cerró en menos de un día, y mi observación "actual" sobre una página de documentación estaba vieja antes de que la lección en la que vivía estuviera terminada.

Quédate con eso, porque es la lección más útil y no es la que salí a enseñar. Una afirmación de catálogo copiada no se pudre en años; se pudrió en veinticuatro horas, y la única razón por la que no estás leyendo ahora mismo la versión equivocada es que alguien volvió a correr la revisión. El enum de la fuente es lo que no te hace esto: `ExtensionType` tiene 29 variantes de producción, PermissionedBurn entre ellas, más 3 variantes solo de prueba escondidas detrás de una bandera `cfg(test)` que nunca llegan a producción, y esa afirmación carga un hash de commit para que puedas decir exactamente qué mundo describe. Una página de documentación describe lo que describa hoy, sin ningún sello de versión en ninguna parte. El código es la verdad, y más importante todavía, el código es la verdad *en un momento que se puede nombrar*.

Tampoco me creas los 29 a mí. Tienes el repo abierto en el commit fijado, así que sube en ese mismo archivo hasta el enum `ExtensionType` y cuenta las variantes tú mismo. Resta las que están detrás de atributos `#[cfg(test)]`, y llegas a 29. Treinta segundos de contar, y ahora tienes el número en el que las dos páginas de documentación no se podían poner de acuerdo, con un hash de commit adjunto. Esa jugada, revisar el enum en vez de citar una página, es la versión más pequeña posible de todo lo que hace esta lección.

Los viejos cartógrafos tenían un nombre para este modo de falla, más o menos al revés: calles trampa. Los cartógrafos dibujaban una calle falsa en sus mapas para que, cuando el mapa de un rival mostrara la misma calle falsa, la copia quedara probada. La deriva de la documentación es el mismo mecanismo corriendo hacia adelante: una página omite una extensión, los tutoriales río abajo copian la página, y pronto a la mitad del modelo mental del ecosistema le falta una variante de producción real. Nadie plantó el error a propósito. La copia lo propagó igual. La única defensa es la que las víctimas de los cartógrafos nunca tuvieron: puedes ir a relevar el territorio tú mismo, porque el territorio es un repo de Git público.

![Una comparación que muestra la misma página de documentación omitiendo PermissionedBurn un día y listándolo al siguiente, al lado del enum de la fuente fijado a un commit, cuyas 29 variantes de producción quedan como fueron registradas.](assets/v02-comparison.webp)

Por eso la habilidad emblemática de este curso es la derivación y no la memorización. Una lista de conflictos copiada tiene una fecha de vencimiento impresa en tinta invisible. Una derivada carga su propia procedencia: esta matriz es verdadera en el commit 426400f, y acá está la función de la que salió, y acá está cómo volver a derivarla cuando el commit se mueva.

Con eso en mente, vamos a leer la función.

### Leyendo las cinco reglas en la fuente

Acá está el corazón del asunto, textual de `interface/src/extension/mod.rs` en 426400f. La función junta siete booleanos desde la lista de extensiones del mint propuesto, y después corre cinco guardas:

```rust
/// Check for invalid combination of mint extensions
pub fn check_for_invalid_mint_extension_combinations(
    mint_extension_types: &[Self],
) -> Result<(), TokenError> {
    let mut transfer_fee_config = false;
    let mut confidential_transfer_mint = false;
    let mut confidential_transfer_fee_config = false;
    let mut confidential_mint_burn = false;
    let mut interest_bearing = false;
    let mut scaled_ui_amount = false;
    let mut non_transferable = false;

    for extension_type in mint_extension_types {
        match extension_type {
            ExtensionType::TransferFeeConfig => transfer_fee_config = true,
            ExtensionType::ConfidentialTransferMint => confidential_transfer_mint = true,
            ExtensionType::ConfidentialTransferFeeConfig => {
                confidential_transfer_fee_config = true
            }
            ExtensionType::ConfidentialMintBurn => confidential_mint_burn = true,
            ExtensionType::InterestBearingConfig => interest_bearing = true,
            ExtensionType::ScaledUiAmount => scaled_ui_amount = true,
            ExtensionType::NonTransferable => non_transferable = true,
            _ => (),
        }
    }

    if confidential_transfer_fee_config && !(transfer_fee_config && confidential_transfer_mint)
    {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if transfer_fee_config && confidential_transfer_mint && !confidential_transfer_fee_config {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if confidential_mint_burn && !confidential_transfer_mint {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if scaled_ui_amount && interest_bearing {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if non_transferable && confidential_transfer_mint && !confidential_mint_burn {
        return Err(TokenError::InvalidExtensionCombination);
    }

    Ok(())
}
```

Fíjate en lo que NO está acá antes de numerar lo que sí. Solo siete de los 29 tipos de extensión siquiera aparecen en el match. MetadataPointer, PermanentDelegate, TransferHook, MintCloseAuthority, PermissionedBurn: ninguna de ellas participa en ninguna regla de conflicto. La abrumadora mayoría de los pares de extensiones son simplemente legales, que es en sí mismo un hallazgo que no podrías sacar de una documentación que nunca publicó la función. La matriz es casi toda verde, con cinco líneas rojas que cruzan una esquina.

Ahora las cinco guardas, en orden de la fuente, cada una con su porqué. La numeración es nuestra, la lógica es de ellos.

**Regla 1: ConfidentialTransferFeeConfig requiere AMBAS, TransferFeeConfig Y ConfidentialTransferMint.** La extensión de comisión confidencial es un puente. Existe para que la recaudación de comisiones funcione cuando los montos están cifrados, así que un mint que carga el puente sin sus dos extremos es incoherente: no habría comisión que recaudar, o no habría cifrado bajo el cual recaudarla. El programa se niega a inicializar un puente a ninguna parte.

**Regla 2: TransferFeeConfig más ConfidentialTransferMint juntas REQUIEREN ConfidentialTransferFeeConfig.** Este es el espejo de la regla 1, y es la dirección más interesante. Piensa en lo que pasaría sin ella. Las transferencias de un mint con comisión tienen que retener un porcentaje del monto. Las transferencias de un mint confidencial cifran el monto. Una transferencia que es las dos cosas no puede calcular la comisión retenida sobre un número que no tiene permitido ver, a menos que un mecanismo dedicado maneje las comisiones en el dominio cifrado. Ese mecanismo es exactamente ConfidentialTransferFeeConfig. Así que el par no está prohibido, solo incompleto, y el arreglo es aditivo: agrega la tercera extensión y el conjunto se vuelve legal. Quédate con esa distinción, porque "estas dos entran en conflicto" y "estas dos exigen una tercera" se ven idénticas en un mensaje de rechazo y significan cosas completamente distintas para tu producto.

**Regla 3: ConfidentialMintBurn requiere ConfidentialTransferMint.** ConfidentialMintBurn hace confidenciales los cambios de supply. Un supply confidencial en un mint cuyos saldos y transferencias son todos públicos no protegería nada: los deltas de mint y de burn serían reconstruibles a partir de los movimientos de cuenta visibles a su alrededor. La dependencia corre en una sola dirección. Las transferencias confidenciales sin supply confidencial son un producto perfectamente válido (saldos ocultos, emisión pública, la mayoría de las stablecoins confidenciales quieren exactamente esto). El supply confidencial sin transferencias confidenciales es una puerta mosquitera en un submarino.

**Regla 4: ScaledUiAmount e InterestBearingConfig son mutuamente excluyentes.** Son las únicas dos extensiones que reescriben cómo se MUESTRA un monto crudo, cada una aplicando su propia matemática de multiplicador. Dos multiplicadores de visualización en un mismo mint quiere decir que cada billetera y cada indexador tienen que responder "¿cuál gana, y en qué orden?", y cualquier respuesta sería arbitraria. El programa se niega a crear la ambigüedad. Esta es la única exclusión mutua de verdad en toda la matriz: no una pieza que falta, no una compañera forzada, solo dos extensiones que nunca pueden compartir un mint.

**Regla 5: NonTransferable más ConfidentialTransferMint es inválido A MENOS QUE ConfidentialMintBurn también esté presente.** La regla más extraña y mi favorita, porque puedes sentir la conversación de diseño que hay detrás. Un token no transferible (soulbound) con saldos confidenciales: ¿qué escondería siquiera? Las transferencias son la cosa que el cifrado protege, y no hay ninguna. Pero agrega ConfidentialMintBurn y el conjunto entra en foco: los montos de mint y de burn son ahora la superficie confidencial. Un emisor puede operar una credencial soulbound cuyos tamaños de emisión son privados. Así que la guarda es condicional: el par solo es un sinsentido, el trío es un producto.

![Tabla de las cinco reglas de conflicto con las extensiones involucradas, cada restricción y su clase (puente, aditiva, dependencia de una sola vía, exclusión mutua, condicional), citada a las líneas de la fuente fijada.](assets/v03-table.webp)

Una cosa más que la fuente te dice y que ninguna página de documentación te habría dicho. Mira lo que cada guarda devuelve en realidad: `Err(TokenError::InvalidExtensionCombination)`, el mismo error, cinco veces seguidas. El programa nunca te dice qué regla rompiste. Te dice que rompiste una. On-chain eso aparece como un número de error de programa personalizado dentro de una transacción fallida, sin número de regla, sin nombre de extensión y sin ninguna pista sobre si tu conjunto era contradictorio o apenas incompleto. Que es precisamente la brecha que `check-combo` existe para cerrar. El validador que estás por escribir devuelve un `reason` que nombra la guarda que se disparó, así que una revisión de diseño recibe una oración en vez de un código de error, y la recibe antes de que nadie gaste un lamport. Misma lógica, mejor diagnóstico. Quédate con eso cuando escribas las cadenas de reason en el lab: son todo el argumento para correr un validador local en vez de mandar la transacción y leer los escombros.

Lee las cinco como un conjunto y aparece un patrón: cuatro de las cinco orbitan alrededor de la suite de transferencias confidenciales. Eso no es un accidente. El cifrado es la única capacidad que cambia lo que las OTRAS extensiones pueden saber, así que es la única capacidad que genera ley entre extensiones. Las comisiones necesitan ver montos, las auditorías de supply necesitan ver mints y burns, lo soulbound necesita algo que valga la pena esconder. Si no recuerdas nada más de la teoría, recuerda: lo confidencial es el centro gravitacional de la matriz de conflictos, y la regla 4 es la excepción solitaria, una colisión de matemática de visualización sin nada de cifrado cerca.

### Tres briefs de producto, pasados por la máquina

El framework más las reglas son todo el loop de evaluación, así que córrelo en caliente tres veces antes de construir nada. Lee cada brief, traduce las frases de requisito a nombres de extensión, y después deja que las cinco reglas juzguen el conjunto borrador. Esta es exactamente la conversación que vas a tener en cada revisión de diseño del Módulo 2 en adelante.

Brief uno: SPROUT, la moneda del juego que nuestras construcciones de Overgrowth empiezan a acuñar el próximo módulo. La especificación dice que los tenedores pagan un pequeño impuesto en cada transferencia, que los saldos deberían mostrar el rendimiento que la cooperativa paga por el grano almacenado, y que el token lleva su propio nombre y símbolo sin ningún programa de metadatos externo. "Impuesto en cada transferencia" es TransferFeeConfig. "Muestra el rendimiento" es una de las dos extensiones de visualización, InterestBearingConfig o ScaledUiAmount, un solo asiento por la regla 4, y elegir entre ellas es la conversación de diseño del próximo módulo. "Su propio nombre y símbolo" es MetadataPointer más TokenMetadata, con el puntero apuntado al mint mismo. Nada más en la especificación nombra un comportamiento, así que el conjunto borrador son esas cuatro (con el asiento de visualización ocupado por exactamente un inquilino), y fíjate en que SPL clásico murió en la primera oración: la palabra "impuesto" sola forzó Token-2022. Corre las reglas: ninguna guarda se dispara contra el conjunto, así que es legal tal como está en el borrador. Este es el caso de todos los días, y su núcleo de comisión más metadatos es el `expectValid` al comienzo de tu archivo de pruebas.

El paso de traducción, escrito una vez para que veas su forma:

```text
SPROUT spec phrase              -> extension name
"tax on every transfer"         -> TransferFeeConfig
"displays yield"                -> InterestBearingConfig OR ScaledUiAmount (rule 4: one seat)
"own name and symbol"           -> MetadataPointer + TokenMetadata (pointer aimed at the mint)
verdict: no guard fires         -> legal as drafted
```

Brief dos: una stablecoin de nómina para un estudio que paga a sus colaboradores on-chain, donde los salarios no deben ser legibles para cada colega, y el emisor quiere una pequeña comisión de transferencia para financiar operaciones. "Salarios no legibles" es ConfidentialTransferMint. "Comisión de transferencia" es TransferFeeConfig. Conjunto borrador: esas dos. Las reglas dicen que no: se dispara la regla 2, porque una comisión no se puede calcular sobre un monto que el programa no tiene permitido ver sin un mecanismo para comisiones en el dominio cifrado. El arreglo es aditivo. Agrega ConfidentialTransferFeeConfig y el trío es legal. Nadie tuvo que descartar un requisito; el conjunto estaba incompleto, no contradictorio.

Brief tres: una credencial soulbound de harvest, no transferible por diseño, donde el emisor quiere que los tamaños de emisión queden privados. "No transferible" es NonTransferable. "Tamaños de emisión privados" arrastra a la suite confidencial, así que el borrador ingenuo es NonTransferable más ConfidentialTransferMint. La regla 5 lo rechaza, y ahora sabes por qué: sin transferencias que cifrar, el par no protege nada. Agrega ConfidentialMintBurn, que es donde vive de verdad la privacidad para este producto, y el trío pasa. Una vuelta de tuerca más ya que estamos: si esa credencial además quisiera mostrar un saldo con rebasing, le toca ScaledUiAmount O InterestBearingConfig, nunca las dos. La regla 4 no tiene arreglo aditivo. Alguien en la revisión de diseño tiene que elegir, y es mejor que ese alguien seas tú, hoy, y no un error `InvalidExtensionCombination` el día del lanzamiento.

![Tabla que pasa tres briefs de producto por el framework, mostrando requisitos traducidos a conjuntos de extensiones, la regla que juzga cada borrador y los arreglos aditivos que volvieron legales dos filas.](assets/v04-table.webp)

Tres briefs, dos rechazos, cero requisitos descartados. Esa proporción es el punto. La mayor parte de lo que las reglas hacen en la práctica no es prohibir productos, es decirte qué tercera extensión olvidó tu par, y un mensaje de rechazo que nombra su regla convierte un misterio del día del lanzamiento en un punto de la revisión de diseño.

### La capa de los pares forzados

Las cinco reglas responden "¿pueden coexistir estas extensiones de mint?" Una segunda función en el mismo archivo responde una pregunta distinta: "dado este mint, ¿qué tiene que llevar cada CUENTA de token?" Se llama `required_init_account_extensions`, está en la línea 1296, y es una búsqueda directa:

```rust
fn required_init_account_extensions(&self) -> &'static [Self] {
    match self {
        ExtensionType::TransferFeeConfig => &[ExtensionType::TransferFeeAmount],
        ExtensionType::NonTransferable => &[
            ExtensionType::NonTransferableAccount,
            ExtensionType::ImmutableOwner,
        ],
        ExtensionType::TransferHook => &[ExtensionType::TransferHookAccount],
        ExtensionType::Pausable => &[ExtensionType::PausableAccount],
        #[cfg(test)]
        ExtensionType::MintPaddingTest => &[ExtensionType::AccountPaddingTest],
        _ => &[],
    }
}
```

Cuatro pares de producción. TransferFeeConfig fuerza TransferFeeAmount en cada cuenta, porque las comisiones retenidas tienen que acumularse en algún lado por tenedor. NonTransferable fuerza dos: NonTransferableAccount, más ImmutableOwner para que un tenedor no pueda esquivar la condición soulbound reasignando la propiedad de la cuenta (transferir el contenedor en vez del token: una escapatoria que el par cierra al nacer). TransferHook fuerza TransferHookAccount, la bandera por cuenta que lee tu programa de hook. Pausable fuerza PausableAccount. Y ahí, en medio de código de producción, hay una rama `#[cfg(test)]`: una de las tres variantes solo de prueba que excluiste de la cuenta de 29, visible exactamente donde la documentación nunca te la mostraría.

Pon las dos funciones lado a lado y por fin puedes dimensionar el presupuesto de restricciones de todo el programa. Siete de las 29 variantes de producción aparecen en las guardas de conflicto. Cuatro aparecen en la búsqueda de pares forzados. Dos de esas, TransferFeeConfig y NonTransferable, aparecen en las dos, así que nueve extensiones distintas cargan alguna ley entre extensiones y las otras veinte se combinan libremente con todo. Ese es un diseño notablemente permisivo, y vale la pena decirlo en voz alta porque la frase "matriz de conflictos" hace que la gente imagine un campo minado. No es un campo minado. Es un campo casi todo abierto con nueve piedras marcadas, y ahora sabes dónde está cada una de ellas.

![Barra apilada que divide las 29 variantes de extensión de producción en 20 sin restricciones, 5 solo en las guardas de conflicto, 2 solo en la búsqueda de pares forzados y 2 en las dos.](assets/v05-chart.webp)

Fíjate en que las dos funciones responden preguntas en capas distintas, y en que la segunda muerde más tarde. Las cinco reglas corren en la inicialización del mint, así que un conjunto ilegal falla de inmediato y con ruido, una sola vez, delante de la persona que lo eligió. Los pares forzados corren en la inicialización de la cuenta de token, que pasa cada vez que aparece un tenedor nuevo, potencialmente meses después de que el mint se lanzó y mucho después de que esa persona ya siguió su camino. Un cliente que asigna espacio de cuenta a partir de una longitud hardcodeada en vez de consultar esta tabla se sale con la suya exactamente mientras el mint no cargue extensiones forzadas, y después empieza a fallar con usuarios reales el día que alguien habilita una comisión de transferencia en un mint nuevo y copia el código viejo del cliente a otro lado. Ese es un ticket de soporte sin autor evidente. Lee la longitud de la tabla, nunca de la memoria.

Esta capa importa para el costo y para la decodificación. Cada extensión de cuenta forzada son bytes en todas y cada una de las cuentas de tenedor, rent pagado por usuario, para siempre. Cuando el `decode-mint` de m01-l2 te muestre una cuenta de tenedor más gorda de lo que esperabas, esta tabla es el porqué. En el validador lo entregamos como datos y no como lógica: `check-combo` exporta la tabla para que lecciones posteriores puedan ponerle precio a la creación de cuentas antes de comprometerse con un diseño de mint.

### El trade-off, y lo que un check verde no quiere decir

Cada lección de este curso nombra su trade-off, y esta tiene dos, los dos filosos.

Primero: una matriz derivada de la fuente es exactamente correcta y exactamente perecedera. Lo que derivaste es correcto en el commit 426400f y puede moverse la próxima vez que el enum crezca o que cambie la función de reglas. Veintinueve variantes hoy era un número más chico no hace mucho, y PermissionedBurn es la prueba de que el enum todavía crece. Así que la habilidad que estás comprando en esta lección no es la matriz. La matriz es un subproducto. La habilidad es la derivación, repetible contra cualquier commit que esté fijado el día que lo necesites. Cuando una regla cambia upstream, los tests de tu validador fallan contra la fuente nueva, y esa falla ES la alerta que la documentación nunca te habría mandado.

Como la re-derivación es la habilidad durable, acá está todo el loop como procedimiento, para que dentro de seis meses te cueste una tarde y no una excavación arqueológica:

1. Elige el pin nuevo. En tu clon: `git fetch origin && git log --oneline -3 origin/main`, elige el commit, anota su hash y su fecha al lado del viejo.
2. Haz diff solo del archivo que importa: `git diff 426400f..<newpin> -- interface/src/extension/mod.rs`. El `--` es lo que le dice a git que lo que sigue es una ruta y no otra revisión, y dejarlo afuera es como te ganas un `unknown revision or path not in the working tree`. La mayor parte de la actividad upstream nunca toca este archivo, y un diff vacío quiere decir que tu matriz sigue vigente, listo en dos minutos.
3. Si el enum `ExtensionType` cambió: vuelve a contar las variantes de producción (resta las de `#[cfg(test)]`), y revisa si alguna variante nueva aparece en la función de guardas o en la búsqueda de pares forzados.
4. Si `check_for_invalid_mint_extension_combinations` cambió: vuelve a leer las guardas en orden de la fuente, porta el delta a `check-combo.ts`, y actualiza las cadenas `reason` para que las etiquetas sigan coincidiendo con las reglas.
5. Vuelve a correr `npx tsx test-check-combo.ts`, agrega un caso de prueba para lo que sea nuevo, y actualiza el hash fijado en el comentario de cabecera de tu archivo. El comentario de cabecera es la procedencia; un validador que no dice qué commit modela es una página de documentación esperando a suceder.

![Línea de tiempo que va de derivar y fijar la matriz en el commit 426400f, pasa por un evento de deriva upstream sin fecha, y llega al loop corto de diff-portar-reprobar que la vuelve a fijar, etiquetado como una tarde de trabajo.](assets/v06-timeline.webp)

Segundo, y este condiciona un módulo entero más adelante: las reglas del código son necesarias pero no completas. La documentación marca combinaciones que la función de init nunca rechaza. NonTransferable más TransferFeeConfig es el ejemplo canónico: las páginas de documentación llaman incompatible al par (una comisión sobre transferencias que no pueden pasar), pero las cinco reglas extraídas no dicen nada al respecto, y las reglas son lo único que el programa hace cumplir en el init. ¿Entonces qué afirmación gana? No te quedes con mi respuesta, y tampoco te quedes con la respuesta de la documentación. Esta es una pregunta con terminal, y tú tienes una. La vas a resolver tú mismo en el Challenge preguntándole al programa directo, porque "la documentación dice" y "el código hace cumplir" son clases distintas de afirmación y el punto entero es que ahora sabes cómo probar la brecha entre las dos.

Mantén separadas las dos clases de afirmación con nombres: init-inválido quiere decir que el programa rechaza el conjunto, punto final, que es lo que check-combo detecta. No-tiene-sentido quiere decir una advertencia a nivel de documentación que puede corresponder o no a algo que se haga cumplir. Y más allá de las dos está una tercera barrera que ningún validador puede ver: la aceptación de la plataforma. Un conjunto puede ser perfectamente legal de inicializar y aun así no ser enrutable, porque las allowlists de los DEX rechazan extensiones como PermanentDelegate por política, no por legalidad. La legalidad es un piso, no la enrutabilidad. Esa oración es el puente al Módulo 5, donde desarmamos como corresponde lo legal-pero-no-enrutable.

![Tres barreras se interponen entre un conjunto de extensiones y un token negociable: la legalidad de init que el código hace cumplir, que check-combo cubre, después las advertencias de la documentación que nadie hace cumplir, después la política de aceptación de cada plataforma.](assets/v07-diagram.webp)

Esa es toda la teoría. Un framework de decisión de un solo criterio, cinco reglas leídas de la fuente, cuatro pares forzados y un mapa honesto de lo que las reglas no cubren. Ahora lo hacemos ejecutable.

## Lab: porta las cinco reglas a check-combo

El artefacto es R2 en la escalera de este curso: una función pura, sin RPC, sin dependencias de blockchain, y eso es a propósito. Las reglas extraídas de la fuente deberían poder probarse en cuatro milisegundos sin red. La blockchain entra solo al final, cuando R2 consume R1.

1. Arma el lab al lado de tu trabajo de m01-l2 e instala el runner. Dos dependencias de desarrollo, cero dependencias de runtime, y esa es toda la huella (tsx 4.20.5, el mismo pin que introdujo m01-l1, más @types/node 26.2.0; los dos verificados el 2026-08-22, y los dos vale la pena volver a revisarlos cuando lo armes):

```bash
mkdir -p labs/m01-l4 && cd labs/m01-l4
npm init -y
npm pkg set type=module
npm install -D tsx@4.20.5 @types/node@26.2.0
```

tsx corre TypeScript directo sin paso de build; las definiciones de tipos de Node están ahí para que las llamadas a `process.exit` del script de pruebas pasen el chequeo de tipos en tu editor en vez de brillarte en rojo. Nada más se instala en este lab, y esa ausencia es a propósito: las reglas extraídas de la fuente deberían poder demostrarse sin un solo paquete que hable con una blockchain.

La línea `type=module` no es ceremonia. El script de pegamento del paso 7 usa top-level await, `npm init -y` deja el paquete en CommonJS por defecto, y tsx rechaza el top-level await bajo CommonJS con un error de transformación de esbuild que no nombra ninguno de esos dos hechos. Si alguna vez ves `Top-level await is currently not supported with the "cjs" output format` a mitad del curso, esta línea es el arreglo.

2. Crea `check-combo.ts` con la interfaz congelada y la tabla de pares forzados. La firma de abajo es un contrato: lecciones posteriores importan `checkCombo` con exactamente este nombre, y el mint de economía de m02-l1 pasa por su barrera. (`REQUIRED_ACCOUNT_EXTENSIONS` también se exporta, pero solo la lee la barrera de esta misma lección.)

```ts
// check-combo.ts: R2. The five rules of
// check_for_invalid_mint_extension_combinations, ported 1:1.
// Source of truth: solana-program/token-2022 @ 426400f,
// interface/src/extension/mod.rs (lines 1326-1374).

export interface ComboResult {
  valid: boolean;
  reason?: string;
}

// The forced-pair layer from required_init_account_extensions (same file,
// line 1296): initializing a token ACCOUNT for a mint with the left-hand
// extension forces the right-hand account extensions to exist.
export const REQUIRED_ACCOUNT_EXTENSIONS: Record<string, string[]> = {
  TransferFeeConfig: ["TransferFeeAmount"],
  NonTransferable: ["NonTransferableAccount", "ImmutableOwner"],
  TransferHook: ["TransferHookAccount"],
  Pausable: ["PausableAccount"],
};

export function checkCombo(extensions: string[]): ComboResult {
  const has = (name: string): boolean => extensions.includes(name);

  // TODO rule 1: ConfidentialTransferFeeConfig requires BOTH
  //   TransferFeeConfig AND ConfidentialTransferMint.
  // TODO rule 2: TransferFeeConfig + ConfidentialTransferMint together
  //   REQUIRE ConfidentialTransferFeeConfig.
  // TODO rule 3: ConfidentialMintBurn requires ConfidentialTransferMint.

  // Rule 4: ScaledUiAmount and InterestBearingConfig are mutually exclusive.
  if (has("ScaledUiAmount") && has("InterestBearingConfig")) {
    return {
      valid: false,
      reason:
        "rule 4: ScaledUiAmount and InterestBearingConfig are mutually exclusive",
    };
  }

  // TODO rule 5: NonTransferable + ConfidentialTransferMint is invalid
  //   UNLESS ConfidentialMintBurn is also present.

  return { valid: true };
}
```

3. Mira la forma de la regla 4 antes de escribir las otras, porque toda la función es esa forma repetida: cada regla es una cláusula de guarda que devuelve `{ valid: false, reason }` cuando se dispara su condición, y un conjunto que sobrevive a las cinco guardas cae hasta `{ valid: true }`. Una elección estructural que vale la pena nombrar: el original en Rust junta primero los booleanos y después corre las guardas, porque itera una sola vez un slice de variantes del enum por eficiencia dentro del programa. Nosotros no estamos dentro de un programa, así que el closure `has()` sobre `includes` se lee más cerca de los enunciados de las reglas mismas. Porta la LÓGICA, no el loop.

4. Ahora completa las reglas 1, 2, 3 y 5, trabajando directo desde el Rust que tienes abierto en el panel dividido. Escribe cada una donde ya está su TODO: los espacios están dispuestos en orden de la fuente, tres arriba de la guarda de la regla 4 y uno abajo, así que un archivo completo se lee de arriba abajo en la misma secuencia que el Rust. Este es el ejercicio de completar de la lección, así que no te voy a pasar los cuatro cuerpos, pero acá está la disciplina de mapeo que convierte a cada uno en un port de dos minutos. Toma la guarda de Rust de la regla 3: `if confidential_mint_burn && !confidential_transfer_mint`. Léela como una oración ("mint-burn presente y su dependencia ausente"), y después escribe esa misma oración con `has()`. La regla 1 quiere una conjunción negada, cuidado con los paréntesis: el estado inválido es el puente presente mientras NO lo están sus dos extremos. La regla 2 y la regla 5 son las dos guardas de tres términos, dos presencias y una ausencia. Empieza cada cadena `reason` con su etiqueta de regla, `"rule 1: ..."` hasta `"rule 5: ..."`, porque el archivo de pruebas hace su assert sobre la etiqueta: un validador que rechaza el conjunto correcto por la razón equivocada es un validador en el que no puedes confiar para explicar una decisión de producto.

![Lado a lado de la guarda de la regla 4 en Rust y su port a TypeScript, anotado para mostrar el mapeo de bandera a has() y que el validador agrega razones por regla donde el programa tiene un solo error colapsado.](assets/v08-annotated-code.webp)

5. Escribe el script de asserts, `test-check-combo.ts`. El mismo patrón del hilo de pruebas que `test-decode-mint.ts` de m01-l2: asserts pelados, exit 1 en el primer fallo, una línea de resumen al terminar bien. Los casos de abajo son el criterio de aceptación de la lección, incluido el giro de la regla 5 donde agregar una extensión vuelve legal un par ilegal:

```ts
// test-check-combo.ts: the assert-script for R2.
import { checkCombo, REQUIRED_ACCOUNT_EXTENSIONS } from "./check-combo";

let passed = 0;

function expectValid(extensions: string[]): void {
  const result = checkCombo(extensions);
  if (!result.valid) {
    console.error(`FAIL: expected [${extensions}] legal, got: ${result.reason}`);
    process.exit(1);
  }
  passed += 1;
}

function expectInvalid(extensions: string[], ruleTag: string): void {
  const result = checkCombo(extensions);
  if (result.valid) {
    console.error(`FAIL: expected [${extensions}] illegal (${ruleTag}), got valid`);
    process.exit(1);
  }
  if (!result.reason?.startsWith(ruleTag)) {
    console.error(
      `FAIL: [${extensions}] rejected, but by "${result.reason}" instead of ${ruleTag}`,
    );
    process.exit(1);
  }
  passed += 1;
}

// Legal set: a fee token with metadata. The everyday case.
expectValid(["TransferFeeConfig", "MetadataPointer"]);

// Rule 4: the two rebasing-display extensions are mutually exclusive.
expectInvalid(["ScaledUiAmount", "InterestBearingConfig"], "rule 4");

// Rule 3: confidential supply without confidential transfers is nonsense.
expectInvalid(["ConfidentialMintBurn", "MetadataPointer"], "rule 3");

// Rule 2: fees + confidential balances demand the confidential-fee bridge.
expectInvalid(["TransferFeeConfig", "ConfidentialTransferMint"], "rule 2");

// Rule 5: soulbound + confidential is illegal on its own...
expectInvalid(["NonTransferable", "ConfidentialTransferMint"], "rule 5");

// ...and becomes legal once ConfidentialMintBurn joins the set.
expectValid(["NonTransferable", "ConfidentialTransferMint", "ConfidentialMintBurn"]);

// Rule 1: the confidential-fee bridge cannot stand alone.
expectInvalid(["ConfidentialTransferFeeConfig"], "rule 1");

// The forced-pair table is data, not logic. Sanity-check its shape.
if (REQUIRED_ACCOUNT_EXTENSIONS["NonTransferable"].length !== 2) {
  console.error("FAIL: NonTransferable must force two account extensions");
  process.exit(1);
}
passed += 1;

console.log(`check-combo: all ${passed} assertions passed`);
```

6. Córrelo:

```bash
npx tsx test-check-combo.ts
```

Antes de que completes los TODO, la corrida falla en el caso de la regla 3, y ese starter que falla es el punto: demuestra que las pruebas pueden atrapar un validador incompleto. Con tus cuatro reglas ya en su lugar, la salida esperada:

```
check-combo: all 8 assertions passed
```

Si estás fallando por una cadena de reason y no por un veredicto, ese es el assert de la etiqueta haciendo su trabajo. Revisa qué regla dispara primero tu orden de guardas: un conjunto puede violar dos reglas a la vez (agrega ConfidentialMintBurn al par de la regla 4 y aplican tanto la regla 3 como la 4), y el orden de la fuente decide qué razón gana. Iguala el orden de la fuente y las etiquetas cuadran.

7. Ahora cierra el loop con R1. Todo hasta acá validó conjuntos hipotéticos; el contrato del artefacto dice que check-combo corre contra el conjunto REAL de un mint real, que es exactamente lo que emite tu inspector de m01-l2. Y hay una costura que cruzar primero, la que m01-l2 te advirtió en su nota sobre nombres: `checkCombo` es un port 1:1 de Rust, así que habla las grafías de Rust, mientras que `decode-mint` nombra las extensiones desde el enum `ExtensionType` del cliente JS fijado, y en tres entradas los dos no coinciden. El tipo 16 es `ConfidentialTransferFeeConfig` en la fuente y `ConfidentialTransferFee` en el cliente. El tipo 25 es `ScaledUiAmount` y `ScaledUiAmountConfig`. El tipo 26 es `Pausable` y `PausableConfig`. Mete las cadenas del cliente directo en las reglas de Rust y la regla 2 se dispara sobre PYUSD, el mint emblemático del propio curso, y eso no es en absoluto lo que debería producir un mint real que ya se inicializó.

Así que el pegamento tiene un solo trabajo antes de entregar la lista, y m01-l2 ya te dijo en qué campo confiar: el u16 es la identidad, los nombres son skins por toolchain encima de él. Normaliza sobre el NÚMERO. Escribe `check-live.ts` (ajusta la ruta de import y el nombre de export para que coincidan con tu propio archivo decode-mint; este snippet asume que el resultado decodificado lleva el array `extensions: {name, type, length}[]` que tu script de asserts de R1 ya revisa):

```ts
// check-live.ts: R1 feeds R2. Decode a live mint, judge its set.
import { decodeMint } from "../m01-l2/decode-mint";
import { checkCombo } from "./check-combo";

// The three type codes where the pinned client's enum and the Rust source
// disagree on spelling. Keyed on the u16 rather than on the client's string,
// because the number is the extension's identity and the string is a skin:
// a name-to-name table would go stale the next time a client renames one, and
// this table can only go stale if a type code changes meaning, which is a far
// louder event.
const RUST_NAME_BY_TYPE: Record<number, string> = {
  16: "ConfidentialTransferFeeConfig", // client: ConfidentialTransferFee
  25: "ScaledUiAmount", //               client: ScaledUiAmountConfig
  26: "Pausable", //                     client: PausableConfig
};

const mintAddress = process.argv[2];
if (!mintAddress) {
  console.error("usage: npx tsx check-live.ts <mint-address>");
  process.exit(1);
}

const decoded = await decodeMint(mintAddress);
const names = decoded.extensions.map((ext) => RUST_NAME_BY_TYPE[ext.type] ?? ext.name);
console.log(`extensions: [${names.join(", ")}]`);
console.log(checkCombo(names));
```

Apúntalo al mint fijado de m01-l2 y deberías ver su lista de extensiones, con el tipo 16 ahora impreso en la grafía de la fuente, seguida de `{ valid: true }`. Toma el desvío si eso te sorprende: comenta la normalización, vuelve a correrlo, y mira cómo tu validador recién nacido rechaza la stablecoin de PayPal con `rule 2: TransferFeeConfig + ConfidentialTransferMint require ConfidentialTransferFeeConfig` — un rechazo perfectamente correcto sobre la regla y perfectamente equivocado sobre el mint. Dos herramientas, dos vocabularios, un código de tipo, y una hora de debuggear reglas que nunca estuvieron rotas. Cada integración que llegues a escribir entre dos SDKs tiene una costura como esta en algún lado. Después cobra la nota que te guardaste del paso 10 de m01-l2: pasa por el mismo pipeline el mint más raro que encontraste. También va a volver válido, porque se inicializó, y esa es la parte interesante: lee su lista contra las nueve piedras marcadas y mira bajo qué ley entre extensiones vive, si es que vive bajo alguna. Claro que es válido: se inicializó, así que pasó estas mismas cinco reglas dentro del programa el día que nació. Que es el remate silencioso de todo el lab. Cada mint Token-2022 vivo en mainnet es un testigo que ya pasó la función que acabas de portar. Tu validador mueve ese juicio de después de los hechos a antes de la revisión de diseño.

![Diagrama de flujo del pipeline que va de la dirección del mint por decode-mint (R1) hasta checkCombo (R2), y se bifurca en seguir-si-es-válido o corregir-el-conjunto-si-hay-rechazo, haciendo de barrera para cada mint que se construya después en el curso.](assets/v09-flowchart.webp)

Ese es el lab. Una confesión antes del challenge: la primera pasada de este curso fijó una tabla de conflictos escrita a mano en un documento de planeación, tres reglas recordadas de un changelog. Escribir la pasada de extracción contra la función de verdad encontró las otras dos en menos de una hora, incluida la condicional de la regla 5 que la tabla había aplanado a un simple "incompatible". Una matriz copiada no solo se pone vieja. Empieza vieja. El validador que escribiste es cómo se mantiene fresca: las pruebas se rompen cuando la fuente se mueve, y rehacer la derivación toma una tarde.

## Challenge

Dos partes, una en solitario y una empírica.

**Solo: el coding challenge.** Este es el único coding challenge del módulo y es el artefacto de esta lección probado de punta a punta: implementa `checkCombo` para que las cinco reglas se disparen exactamente como las dispara la función de la fuente. El starter es un primo recortado del archivo que armaste en el paso 2 del lab: el mismo resultado `{ valid, reason? }`, la regla 4 ya implementada y las otras cuatro faltando. Una diferencia deliberada con el contrato del lab: el calificador llama a tu función con cada nombre de extensión como su propio argumento de cadena, así que la firma que viene es `checkCombo(ext1: string, ext2?: string, ext3?: string)`, la primera línea del starter junta esos argumentos en el mismo array `extensions` que recibe tu versión del lab, y de ahí para abajo la lógica de las reglas es idéntica. Es deliberadamente más flaco que la versión del lab, sin tabla de pares forzados y sin etiquetas de regla en su única cadena de reason, así que nada se acarrea salvo la lógica que derivaste. Falla la suite de pruebas tal como viene. Tu solución la pasa entera. Los criterios de aceptación, directo de la barrera:

- `checkCombo("TransferFeeConfig", "MetadataPointer").valid === true`
- `checkCombo("ScaledUiAmount", "InterestBearingConfig").valid === false`
- `checkCombo("ConfidentialMintBurn", "MetadataPointer").valid === false`
- `checkCombo("TransferFeeConfig", "ConfidentialTransferMint").valid === false`
- `checkCombo("NonTransferable", "ConfidentialTransferMint").valid === false`
- `checkCombo("NonTransferable", "ConfidentialTransferMint", "ConfidentialMintBurn").valid === true`

La barrera solo revisa veredictos. Exígete igual la vara más alta y haz que cada rechazo nombre la regla de la fuente que se disparó, como lo hace tu versión del lab, porque un validador que rechaza el conjunto correcto por la razón equivocada pasa las pruebas y aun así pierde una revisión de diseño. Si completaste el lab, ya hiciste el trabajo; el challenge es donde lo demuestras sin el apoyo a la vista.

**Empírico: sondea el delta entre documentación y código.** Esta mitad corre en tu propia máquina — necesita dos CLIs locales y un cluster con el que hablar; la barrera calificada de arriba no depende de eso, así que si estás trabajando solo desde el navegador, léelo, anota la dependencia y vuelve cuando los tengas. Ojo con qué quiere decir exactamente ese "los", porque el entorno de lab permanente del módulo 2 no es eso: el módulo 2 instala paquetes de npm y surfpool, y ninguno de los dos te da las dos herramientas de abajo. Necesitas la CLI de `solana` (m01-l3 te pasa su instalador de una línea como sondeo opcional) y `spl-token-cli`, que es un cargo install y por lo tanto quiere una toolchain de Rust — la que arma la lección del hook del módulo 3. Así que el punto honesto al que aplazar esto es el módulo 3, no el módulo 2, y la forma más barata de despejarlo temprano es correr el instalador de m01-l3 ahora y agregar `cargo install spl-token-cli` al lado. La sección de teoría dejó NonTransferable más TransferFeeConfig sin resolver a propósito: la documentación llama incompatible al par, las cinco reglas extraídas nunca lo mencionan. Tu validador, siguiendo fielmente el código, lo acepta. Así que pregúntale al programa directo. La CLI `spl-token` viene incluida en algunas instalaciones de la toolchain de Solana y en otras no, así que revisa primero con `spl-token --version`; si falta, `cargo install spl-token-cli` te consigue una (5.6.1 es la actual en crates.io al 2026-08-22, y la clave de respuestas de abajo se produjo en 5.5.0, así que cualquier versión por ese barrio está bien). Con la que termines, anota el número, porque importa para cómo lees el resultado. La CLI puede intentar exactamente este init contra devnet:

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 2   # devnet faucet; if rate-limited, use faucet.solana.com
spl-token create-token --program-2022 \
  --enable-non-transferable \
  --transfer-fee-basis-points 50 \
  --transfer-fee-maximum-fee 5000
```

Si el faucet de devnet te corta por límite de tasa (pasa, y seguido, y el airdrop simplemente falla), sirve cualquier validador local en su lugar: apunta `--url` a un mainnet-fork de `surfpool start --no-tui --no-studio` y haz el airdrop ahí. surfpool es un validador local que forkea el estado de mainnet a demanda (en macOS `brew install txtx/taps/surfpool`, en otras plataformas lo agarras desde la página de releases de surfpool); el próximo módulo se vuelve el entorno de lab permanente, así que instalarlo ahora no es esfuerzo desperdiciado. El fork corre el mismo binario de Token-2022 desplegado, así que su veredicto sobre esta pregunta es el mismo veredicto que daría mainnet.

Anota lo que pasa: creado, o rechazado, y por quién. Esos son tres resultados distintos, no dos. El programa puede rechazarlo. La CLI puede negarse del lado del cliente antes de que se mande nada, lo que te diría que el "conflicto" vive en una herramienta y no en el consenso. O puede pasar, en cuyo caso decodifica el mint resultante y confirma con tu propio inspector que las dos extensiones de verdad llegaron, porque "el comando salió con 0" no es la misma afirmación que "la cuenta lleva las dos". No te saltes ese último paso; es la diferencia entre creerle a una CLI y leer los bytes, que es el hábito entero que este módulo existe para construir.

![Un solo intento de init se abre en tres resultados, cada uno probando algo distinto: un rechazo del lado del cliente, un rechazo on-chain o un éxito que solo los bytes decodificados pueden confirmar.](assets/v10-flowchart.webp)

A propósito no estoy imprimiendo el resultado acá arriba. Córrelo primero, anota lo que te salió al lado de la fecha y de tu `spl-token --version`, y solo entonces lee la clave de respuestas al final del Checkpoint. Sea lo que sea que encuentres, fíjate en que check-combo queda sin cambios. Modela lo que el init hace cumplir, y agregar una regla que el programa no hace cumplir haría que tu validador no coincidiera con el programa que existe para predecir.

## Checkpoint

El criterio de esta lección: tu `check-combo` acepta un conjunto legal de extensiones y rechaza cada uno de los cuatro casos ilegales con la regla de la fuente correcta, coincidiendo con `check_for_invalid_mint_extension_combinations` en 426400f, y las pruebas del coding challenge pasan, el starter fallando, la solución en verde. Junto con la corrida que pasa, escribe la única oración que pide la evaluación: tu propia separación entre "legal para inicializar" y "aceptado por una plataforma o una billetera". Si tu oración menciona allowlists o PermanentDelegate, ya interiorizaste el argumento de apertura del Módulo 5.

### Clave de respuestas: qué devuelve el sondeo

Lee esto solo después de haberlo corrido. El 2026-08-22, con spl-token-cli 5.5.0 contra un mainnet-fork corriendo el binario de Token-2022 desplegado, el `create-token` de arriba **tuvo éxito**. Sí, 5.5.0: un minor atrás del 5.6.1 que crates.io listaba como actual ese mismo día, porque mi paquete de toolchain iba atrás del registro, que es exactamente por lo que el challenge te hizo anotar tu propia versión. Si tu sondeo corrió en 5.6.1 y algún comportamiento de abajo difiere, gana tu corrida; anota el delta con la versión adjunta. Imprimió una dirección y una firma como cualquier creación de mint común. Decodificar la cuenta nueva confirma que no fue una ilusión de la CLI. Apunta tu propio inspector de m01-l2 a la dirección que imprimió la CLI y la lectura vuelve con esta forma:

```text
npx tsx decode-mint.ts <THE_MINT_THE_CLI_PRINTED> <YOUR_FORK_RPC_URL>

owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
data length:   282 bytes -> extended-165+1
extensions (2):
  { name: NonTransferable, type: 9, length: 0 }
  { name: TransferFeeConfig, type: 1, length: 108 }
```

282 bytes, dueño `Tokenz...`, y dos entradas TLV, `NonTransferable` y `TransferFeeConfig` en la grafía del cliente fijado, sentadas en el mismo mint: exactamente los 166 + (4 + 0) + (4 + 108) en los que tu aritmética de `compute-len` valora el par. El programa lo aceptó.

Así que el delta se resuelve a favor del código, y tu validador hizo bien en aceptarlo. Las páginas de documentación que llaman "incompatible" a ese par están haciendo una afirmación *semántica*, no una afirmación de legalidad, y la afirmación semántica es justa: un mint NonTransferable rechaza toda transferencia, así que un esquema de comisiones encima nunca puede recaudar nada. Es configuración muerta por la que pagas rent. Eso es algo real de lo que advertirle a un diseñador, y genuinamente no es una regla que el programa haga cumplir en la inicialización. Las dos afirmaciones son verdaderas a la vez, que es exactamente por lo que necesitan nombres separados.

Dos hallazgos laterales de la misma corrida, los dos valen más que el titular. Primero, la CLI sí hace cumplir algunas reglas por su cuenta: pídele `--interest-rate` y `--ui-amount-multiplier` juntos y se niega antes de tocar la red, un conflicto del parser de argumentos y no un error del programa. La regla 4 está guardada dos veces, en dos lugares distintos, y solo uno de ellos es consenso. Segundo, pídele una comisión de transferencia más transferencias confidenciales, el par de la regla 2 que tu suite de pruebas espera que sea ilegal, y tiene éxito igual, porque la CLI agrega `ConfidentialTransferFeeConfig` por ti en silencio e inicializa el trío legal. Los dos hacen el mismo punto: una herramienta sentada entre tú y el programa puede agregar reglas que el programa no tiene, y satisfacer reglas que nunca le pediste que satisficiera. Ninguno de los dos se ve desde el código de salida. Solo la cuenta decodificada te dice qué construiste en realidad, y ese decodificador es tuyo desde m01-l2.

![Entre el conjunto de extensiones que pides y los bytes on-chain están la CLI, que puede negarse o agregar extensiones en silencio, y el programa haciendo cumplir solo sus cinco reglas.](assets/v11-diagram.webp)

Si tu corrida no coincidió con algo de esto, gana tu corrida y quiero enterarme, con la versión y la fecha adjuntas. Eso no es cortesía. Es el quinto paso del loop de re-derivación.

Las fallas por la razón equivocada y los resbalones de paréntesis en la regla 1 son los dos tropiezos que espero; si el assert de la etiqueta te sigue mordiendo después de revisar el orden de la fuente, lleva el caso que falla y tu orden de guardas a la discusión del curso y leemos tu port contra el Rust juntos. Ese tipo de diff, tu derivación contra la fuente, es precisamente el músculo de revisión que este curso está construyendo.

Ahora puedes decir qué conjuntos de extensiones son LEGALES de construir, y puedes demostrarlo contra cualquier mint vivo desde los bytes hacia arriba. Pero legal no es lo mismo que negociable, y antes de llegar a ese ajuste de cuentas, el próximo módulo construye cada extensión de verdad, empezando por el conjunto de economía: comisiones, interés, UI escalada. Tu mint SPROUT va a llevar TransferFeeConfig más una de las dos extensiones de visualización, tu validador va a ser lo que te frene de elegir las dos, y después perseguimos la pregunta que check-combo no puede responder: cuando una transferencia retiene una comisión, ¿dónde queda realmente el dinero, y quién tiene permiso de barrerlo? ¡Feliz derivación! 🌱
