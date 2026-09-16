# La allowlist del DEX como artefacto didáctico

## Resumen

La lección pasada construiste una variante confidencial de SPROUT: auditor key, registro ElGamal, una transferencia cifrada de varias transacciones, y después la archivaste como una rama de emisor especializada. Ahora de vuelta al SPROUT principal, el que de verdad quieres que la gente negocie.

Este es el momento por el que existe esta lección. Pasaste cuatro módulos convirtiendo SPROUT en exactamente el token que querías: comisiones que financian una tesorería, metadatos nativos que la billetera lee sin indexador, y al lado la variante con hook cuyo programa registra cada harvest. Después vas a sembrar un pool de Raydium con la variante con hook, para que el SPROUT restringido pueda por fin negociarse, y la transacción de creación del pool revierte. Ningún stack trace apuntando a tu código, porque tu código está bien. El programa de Raydium leyó la lista de extensiones de ese mint y lo rechazó a propósito, y los docs de Raydium dicen en voz alta la parte que nadie dice sobre exactamente por qué. (Mantén las dos variantes separadas toda la lección: el SPROUT principal lleva comisión más metadatos y va a salir bien; la variante con hook es la que se queda en la puerta.)

Antes de la autopsia, levanta el lab. El mismo surfnet que en el módulo 1 (surfpool 1.2.1 aquí, verificado el 2026-08-22; `brew install txtx/taps/surfpool` en macOS, en otras plataformas se compila desde la página de releases). Surfpool forkea mainnet, y eso importa hoy: el programa Token-2022 real y el despliegue real de CP-Swap están los dos cargados.

```bash
mkdir -p labs/m05-l1 && cd labs/m05-l1
npm init -y && npm pkg set type=module
surfpool start --no-tui --no-studio
```

Un dato de continuidad antes de que te cueste una hora: un fork es efímero. Las cuentas de mainnet las trae de forma perezosa y bajo demanda, pero los mints que creaste TÚ existen solo en un surfnet que siguió levantado desde que los hiciste. Si el tuyo se reinició (o este comando acaba de arrancar uno nuevo), vuelve a acuñar los dos locales que necesita el paso 5 antes de llegar ahí. Los dos comandos corren desde la RAÍZ del workspace, así que primero haz `cd` de vuelta hacia arriba saliendo de `labs/m05-l1`: `npx tsx labs/m02-l4/add-metadata.ts` recrea el SPROUT con nombre y reescribe `sprout-mint.json`, y la ceremonia de m03-l2 (`spl-token create-token --program-2022 --decimals 6 --transfer-hook $HOOK` después de volver a desplegar el hook) recrea la variante con hook. Cinco minutos, y las direcciones cambian, lo cual está bien; todo aquí toma direcciones como argumentos.

Ya te has rozado con esta allowlist dos veces: m02-l1 la citó cuando el conjunto de economía resultó quedar entero dentro de ella, y el cierre del módulo confidencial te apuntó de vuelta hacia ella. Hoy dejamos de citar y leemos el código que la impone. La negociabilidad no es una sensación y no es un ticket de soporte: es una allowlist concreta que vive en el código que el DEX corre de verdad, y puedes leerla en unos noventa segundos. Vas a leer desde el código fuente la allowlist de cinco extensiones de Raydium CP-Swap, aprender los tres bypasses documentados que la hacen más sutil que "no se permiten extensiones", y construir el primer borrador de R6, un predictor de enrutabilidad que reproduce la decisión de aceptar o rechazar del programa a partir del perfil de un mint. Después lo corres contra tu propio SPROUT y su variante con hook en un fork de mainnet y ves si tu predicción sobrevive al contacto.

El repliegue de la ayuda, dicho en voz alta: la lectura del código fuente y el primer bypass son una guía trabajada, escribo junto contigo. El predictor es un problema de completar, recibes el archivo con dos huecos y la teoría te dice qué los llena. El coding challenge es en solitario, sin apoyo, la regla completa de aceptar y rechazar a partir de un perfil de mint arbitrario.

Deja ese surfnet corriendo y abre una segunda terminal, porque lo primero que hacemos es leer el Rust de otra persona.

## Leer la regla que decide si tu token se negocia

### Las cinco que pasan

El AMM de producto constante de Raydium vive en el repositorio `raydium-cp-swap`, y toda la política de Token-2022 es una función en un archivo. Clónalo y mira:

```bash
git clone https://github.com/raydium-io/raydium-cp-swap.git
cd raydium-cp-swap
git rev-parse --short HEAD
sed -n '18,23p' programs/cp-swap/src/utils/token.rs
sed -n '225,236p' programs/cp-swap/src/utils/token.rs
```

En el commit `244e124` (pusheado el 2026-08-19, que es lo que leí el 2026-08-22) esos dos rangos son la parte que todo el mundo cita. El segundo recorre la lista de extensiones del mint y devuelve `Ok(false)` en el momento en que se topa con una extensión fuera de un conjunto fijo de cinco. El primero es un array hardcodeado de cuatro direcciones de mint que se saltan el recorrido por completo. Imprime tu propia salida de `git rev-parse` y anótala, porque un número de línea en una lección es una promesa con fecha de caducidad corta.

Las cinco que pasan, en el orden del propio programa: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`, `InterestBearingConfig`, `ScaledUiAmount`. Esa es la lista. Cada otra extensión del catálogo que pasaste tres módulos construyendo, cada una de ellas, revierte la creación del pool en esta plataforma.

Mira qué tienen en común esas cinco antes de mirar qué falta. Una comisión de transferencia mueve valor, pero lo mueve por una regla declarada en el TLV del propio mint, a una tasa que un pool puede leer y con la que puede fijar precio. La referencia de Token-2022 de Raydium es explícita sobre cómo lo maneja: la matemática del pool resta la comisión de entrada, y el programa Token-2022 se encarga de la de salida. Interest-bearing es todavía más manso, porque el pool contabiliza en montos de principal y el multiplicador de UI es solo decorativo. Scaled UI es visualización y nada más. Metadata pointer y token metadata son cadenas y una dirección. Ni una de las cinco puede correr código, tener una clave sobre el saldo de otra persona, o volver ilegible un número.

![Comparación de las cinco extensiones de Token-2022 que Raydium CP-Swap acepta frente a las seis que rechaza, cada rechazo con la justificación publicada por Raydium.](assets/v01-comparison.webp)

Ahora contrasta eso con el modelo ingenuo que carga la mayoría de la gente, el que cargué yo por más tiempo del que me gustaría admitir: "los tokens Token-2022 no se negocian". Ese modelo está equivocado en las dos direcciones a la vez. Cinco extensiones se enrutan sin problema, así que un mint de Token-2022 con comisión, con metadatos y que acumula intereses es un activo de pool perfectamente ordinario. Y un mint con una sola extensión fuera de la lista no se negocia un poco peor, no crea el pool en absoluto. La falla es binaria y pasa en la creación, no en el momento del swap.

Tampoco tomes mi lista por fe. Desde dentro del clon, esto imprime los nombres con los que el archivo hace match hoy de verdad:

```bash
sed -n '225,236p' programs/cp-swap/src/utils/token.rs | grep -o 'ExtensionType::[A-Za-z]*'
```

Si la forma se movió desde que la leí, ese comando te lo dice en una línea, que es exactamente el hábito que esta lección intenta instalar.

Un detalle de implementación carga peso diagnóstico real, así que fíjate en él mientras el archivo está abierto. El chequeo de soporte no lanza cuando te rechaza. Devuelve un booleano, `Ok(false)`, y quien lo llama convierte eso en la falla de creación del pool. Lo que llega a tu terminal es el genérico de la plataforma, "este mint no está soportado", no "tu entrada de TransferHook es el problema, todo lo demás estaba bien". Ya te topaste con este hueco antes: en m01-l4 encontraste que las cinco reglas de combinación de Token-2022 devolvían el mismo error idéntico, así que el programa podía decirte que rompiste una regla pero nunca cuál. La misma arquitectura, el mismo silencio, una capa más arriba. Y es el argumento completo para construir un predictor local en vez de aprender por revert. A las 2am, la distancia entre "mint no soportado" y "quítale el hook o llévate esto a Meteora" es la distancia entre un ticket de soporte y un arreglo.

### Por qué la línea cae exactamente ahí

La pregunta interesante no es qué contiene la lista. Es por qué un equipo de ingenieros de AMM trazó el límite en ese lugar en particular, porque una vez que puedes derivar su razonamiento puedes predecir la lista de la próxima plataforma antes de leerla.

Parte de lo que es un pool, mecánicamente. Un pool de CP-Swap es un programa que custodia dos cuentas de token, los vaults, y les fija precio a los swaps contra sus saldos. (Esa sola frase es todo el AMM que necesitamos. La matemática de pools, la mecánica de ticks y bins y la estrategia de LP pertenecen al curso planificado DeFi and RWA Engineering; lo que esta lección toma prestado es solo el asiento del pool en la mesa.) Todo lo que puede admitir con seguridad se desprende de dos requisitos: tiene que poder computar un precio a partir de un saldo, y tiene que poder confiar en que un saldo que tiene se queda donde está.

Corre las reglas candidatas ingenuas contra eso y míralas fallar. Regla uno: rechazar cualquier cosa que cambie los montos. Equivocada, porque TransferFeeConfig cambia montos y pasa; el pool puede computar alrededor de una tasa declarada. Regla dos: rechazar cualquier cosa que toque los números que muestra una UI. También equivocada, porque interest-bearing y scaled UI reescriben los dos el número visualizado y pasan los dos; el pool lee los montos crudos por debajo y trata el multiplicador como decoración. Regla tres: rechazar cualquier cosa no auditada. Esa está más cerca, y es literalmente la razón declarada para los punteros de grupo y de miembro ("sin revisar"), pero no explica por qué un programa de hook bien auditado sigue rechazado.

![Tres reglas candidatas de allowlist, cada una tachada por la extensión que la refuta, que bajan hacia la regla sobreviviente basada en capacidades a pleno contraste.](assets/v02-comparison.webp)

Lo que sobrevive es más estrecho, y es la frase hacia la que este curso entero ha estado caminando. Un DEX admite extensiones que solo reconfiguran la visualización o se llevan una comisión declarada, y rechaza extensiones que dejan a alguien correr código arbitrario dentro de la transferencia o mover tokens que el pool tiene en custodia.

Lee los tres rechazos emblemáticos con esa regla en la mano. `PermanentDelegate` se rechaza porque, en palabras de Raydium, "un tenedor del delegado puede barrer cualquier cuenta de token, incluido el vault del pool". El vault es el inventario del pool. Una extensión cuyo propósito entero es una clave que puede mover el saldo de cualquiera es, desde el asiento del pool, un riesgo de inventario que no se puede cubrir. `ConfidentialTransfer` se rechaza porque "los montos cifrados impiden fijar precio", que es el mismo argumento que derivaste desde el otro lado en el módulo confidencial: un AMM que no puede leer un monto no puede cotizar un precio. Y `TransferHook` se rechaza porque "invoca un programa custom en cada transferencia, con consumo de CU arbitrario".

Ese último merece un momento, porque es el que la gente entiende al revés. El hook no puede robarle al pool. Esto lo sabes del módulo 3: cada cuenta de la transferencia original queda des-escalada a solo lectura dentro del hook, así que el programa del hook no puede mover fondos, y la propia guía para desarrolladores de Solana lo dice. El rechazo no es sobre robo. Es sobre costo y sobre tuberías. Cada programa que mueve un token con hook tiene que resolver la lista de cuentas extra del mint y reenviar esas cuentas en cada instrucción que transfiere, y el hook entonces quema una cantidad no acotada de unidades de cómputo dentro del presupuesto del swap. Un pool que admite un mint con hook se ofreció voluntario a cargar la resolución de cuentas de un desconocido y la factura de cómputo de un desconocido en cada swap, para siempre, sin pin de versión y sin límite superior. Rechazar es un presupuesto de cómputo con un nombre puesto.

![Diagrama que mapea seis extensiones de Token-2022 sobre tres invariantes del pool, mostrando qué invariante rompe cada extensión rechazada y por qué las admitidas no.](assets/v03-diagram.webp)

Vale la pena responder dos objeciones aquí, porque todo ingeniero que haya entregado un AMM plantea las dos.

La primera: ¿por qué no ponerle simplemente un techo al cómputo del hook y admitirlo? Porque el techo está en el lugar equivocado. El presupuesto de cómputo de una transacción le pertenece a la transacción, y el hook gasta del mismo sobre del que gasta el swap, así que un pool que quiere estar seguro tiene que reservar holgura para el programa de un desconocido en cada cotización. Esa holgura no es gratis, es o una peor cotización o un swap que muere en el límite cuando el hook decide hacer más trabajo del que hizo ayer. Una plataforma que admite diez mints con hook tiene diez presupuestos desconocidos distintos contra los que reservar.

La segunda, más filosa: ¿por qué no simular una transferencia y ver cuánto cuesta el hook de verdad? Porque la simulación responde una pregunta sobre el pasado. Te dice qué hizo ese programa una vez, contra el bytecode desplegado en ese slot, con la lista de cuentas extra tal como estaba. Los programas de hook son actualizables por quien tenga la upgrade authority, y la lista de cuentas extra es estado de cuenta que su autoridad puede reescribir. Así que una allowlist indexada al comportamiento observado es una allowlist que una transacción de upgrade puede invalidar en silencio, en un momento en que nadie está mirando. Indexarla a la capacidad en cambio es incómodo, tosco y estable, y la estabilidad es para lo que optimiza un programa que tiene la liquidez de desconocidos.

Vale la pena nombrar lo que acaba de pasar, porque es la parte reutilizable. La allowlist es un juicio de valor escrito como un match statement. Alguien decidió qué capacidades puede conservar un emisor y seguir teniendo permitida la entrada a una plataforma que tiene el dinero de otras personas, y después compiló esa decisión. Cuando eliges las extensiones de SPROUT no estás eligiendo funcionalidades, estás pujando por la admisión, y la lista de precios es pública.

### Los tres bypasses, y la stablecoin que no debería enrutarse pero lo hace

Aquí es donde una allowlist impresa empieza a mentirte.

Si la regla fuera solo "cada extensión debe estar en la lista", entonces una stablecoin regulada que lleva `PermanentDelegate` para el congelamiento y el clawback de compliance sería no negociable en CP-Swap. Varias de ellas se negocian sin problema. Pasé una tarde vergonzosa convencido de que los docs estaban equivocados antes de volver a `token.rs` y leer las líneas arriba del recorrido de extensiones.

El chequeo tiene tres escotillas de escape documentadas, y ninguna de ellas es una entrada de la allowlist.

La primera es a nivel de programa. El chequeo de extensiones solo existe porque CP-Swap sabe de Token-2022; un mint cuyo dueño es el programa SPL Token clásico no tiene TLV que recorrer y se salta la rama entera. El SPL clásico no está "en la allowlist", está fuera del alcance de la pregunta.

La segunda es el `MINT_WHITELIST` hardcodeado arriba del mismo archivo, de cuatro direcciones de largo en el commit que leí. Cuatro. Una lista de excepciones por nombre, en producción, en el código fuente, sin ninguna ceremonia de gobernanza alrededor. Si tu mint es uno de los cuatro, el recorrido de extensiones nunca corre.

La tercera es una cuenta de asociación de mint, y como es estructural en el diagrama de flujo y en el challenge, esto es lo que es de verdad: la contabilidad propia de CP-Swap, no una extensión de Token-2022. Es una cuenta pequeña por mint del programa CP-Swap, inicializada por la ruta de admin de Raydium para un mint específico, así que funciona como la hermana mayor de la whitelist: aprobación por mint registrada como una cuenta en vez de un array hardcodeado, es decir que una aprobación nueva toma una transacción de admin en vez de un redespliegue del programa. Si existe una cuenta de asociación inicializada para tu mint, el recorrido de extensiones nunca corre. El mismo efecto que el array, una puerta distinta, y una que puedes verificar desde afuera chequeando si la cuenta existe.

Así que el enunciado honesto de la regla es una cosa de dos ramas, y esto es exactamente lo que tu predictor tiene que codificar. Primero pregunta si aplica algún bypass. Solo si ninguno aplica, pregunta si cada extensión está en la lista de cinco. Equivoca ese orden y vas a predecir con confianza el rechazo de un token que se está negociando delante de ti.

![Diagrama de flujo del chequeo que Raydium CP-Swap corre al crear un pool, que muestra tres ramas de bypass para SPL clásico, mints en la whitelist y mints con cuenta de asociación, antes de la prueba contra la allowlist de cinco extensiones y de la ruta de rechazo.](assets/v04-flowchart.webp)

El valor didáctico de esa whitelist no son las cuatro direcciones, es lo que su existencia te dice sobre cómo funciona de verdad la admisión a una plataforma. Algunos tokens entran porque su conjunto de extensiones es aburrido. Otros entran porque alguien en la plataforma tomó una decisión sobre ellos por nombre. Si tu plan de producto es "vamos a llevar un delegado permanente para compliance y a entrar en la whitelist como lo hicieron las stablecoins", ese es un plan de desarrollo de negocio más que uno de ingeniería, y deberías costearlo como tal.

### Lo que el chequeo no puede ver

Antes de que vayas a construir un predictor que reproduce esta regla, ten claro lo estrecha que es la regla, porque dos de sus puntos ciegos van a moldear decisiones que tomes la próxima lección.

Corre una vez. El recorrido de extensiones pasa en la creación del pool, y después de eso el pool existe. Nada vuelve a correr la allowlist sobre un pool vivo cuando el mint cambia por debajo, y el mint puede cambiar: tu `transfer_fee_config_authority` puede armar un esquema de comisión nuevo para un epoch futuro en cualquier momento que quiera, que es exactamente el mecanismo que construiste y viste entrar en vigor en m02-l1. Así que "enrutable" es un enunciado sobre la admisión, no una promesa sobre el comportamiento para siempre. La admisión se le concedió a un tipo de extensión, y la configuración dentro de ese tipo siguió siendo tuya.

Y lee tipos, no configuraciones. El recorrido hace match con variantes de extensión: pregunta si hay una entrada `TransferFeeConfig` presente, no si la comisión es cero o cinco por ciento. Sigue eso hasta el final y te sale un resultado que a la gente le sorprende la primera vez. Un mint que lleva una entrada `TransferHook` cuyo program id es null, un slot de hook que no llama a nada en absoluto, igual falla el recorrido, porque la entrada de TLV está ahí y la entrada es lo que se matchea. Dormido no es ausente. Esa es la imagen espejo del diseño de PYUSD al que llego en un momento, y es por eso que "configuramos la extensión pero la dejamos apagada" te compra buena voluntad con un auditor y exactamente nada con un programa.

![Línea de tiempo que muestra que el chequeo de extensiones de Raydium corre solo en la creación del pool, mientras que los cambios posteriores del esquema de comisión, las acciones de autoridad y un upgrade de hook contrafactual no disparan ningún re-chequeo.](assets/v05-timeline.webp)

### Por venue, nunca por DEX

Una corrección más antes del lab, y es la que te va a salvar de una caída real.

Todo lo de arriba es cierto de CP-Swap y de CLMM de Raydium. No es cierto de Raydium. La misma marca corre plataformas con reglas distintas, y las más viejas son más estrictas: AMM v4 y el Stable AMM toman solo SPL clásico, y la razón que da Raydium es que el programa es anterior a Token-2022. Así que tu SPROUT con comisión, que pasa sin fricción por la allowlist de CP-Swap, no se puede poner en un pool de AMM v4 en absoluto, y quitar una extensión no va a ayudar. Mientras tanto los mints de recompensa de Farm v6 sí pueden ser Token-2022, y LaunchLab crea sus mints con un metadata pointer y una comisión de transferencia opcional con techo de cinco por ciento. Cuatro respuestas distintas, una marca.

Sal de Raydium y la forma cambia otra vez.

Orca publica una tabla de soporte por extensión más un proceso de revisión de Token Badge, que es un mecanismo completamente distinto: no una lista hardcodeada en un programa, sino una aprobación por mint que otorga una persona. Según mi lectura del 2026-08-21 la tabla muestra transfer fee, memo transfer, metadata pointer, token metadata e interest-bearing como soportados, confidential transfer como soportado solo para transferencias no confidenciales, y permanent delegate como algo que requiere un Token Badge. El resto de las filas, incluida la que más quieres, la fila de transfer hook, no volvió en mi consulta, y no voy a adivinarla. Trata eso como un ítem abierto por verificar, no como un hueco que llenar con sensaciones.

Meteora es el contrapunto que mantiene esto honesto. Su Dynamic Bonding Curve soporta explícitamente configs de token con transfer hook; su config tiene opciones de update authority que su README describe como válidas solo para configs y pools con transfer hook. Una plataforma construida después, con el reenvío de hooks diseñado desde el principio, tomó la decisión opuesta a la de Raydium. La propia guía de integración de Meteora también les dice a los builders que rechacen las extensiones de Token-2022 no soportadas antes de mostrar un flujo de lanzamiento y que reenvíen las cuentas del hook en cada instrucción que transfiere. Soporte defensivo, pero soporte real.

Y Jupiter, por donde de verdad se enruta la mayor parte del flujo retail: no pude encontrar una política de enrutamiento de Token-2022 publicada en sus docs para desarrolladores el 2026-08-21. Que no haya página de política no es lo mismo que no tener política. Es una incógnita, y va a tu lista de verificación con su fecha adjunta. La agregación como disciplina de cliente pertenece al curso planificado Client-Side Mastery; lo que te pertenece aquí es saber que la pregunta existe y que nadie la respondió por ti por escrito.

![Tabla que compara seis plataformas de negociación en soporte de Token-2022 y aceptación de mints con hook, con dos celdas marcadas explícitamente como sin resolver o desconocidas y cada fila con su fuente y su fecha de lectura.](assets/v06-table.webp)

Lo que me trae al trade-off que te debo, y va en contra de la lección que estás leyendo. Leer la allowlist de un DEX te dice la verdad para esa única plataforma en ese único commit. No es una especificación portable. La revisión de badge de Orca, la política de enrutamiento de Jupiter y el comportamiento de visualización de cada billetera son reglas separadas que tienes que chequear tú, y congelar las cinco de Raydium como "la regla del ecosistema" es precisamente el error que esta lección existe para matar. La lista también se mueve. Por eso el predictor que estás por construir lleva su commit de origen en un comentario de header, y por eso volver a leer `token.rs` en tu commit fijado es el paso cero de cada lanzamiento, no una tarea de una sola vez.

Dos historias de producción hacen el mismo punto desde extremos opuestos, y después construimos.

La que todavía me hace reír: el programa de transfer hook de pump.fun, desplegado en `333UA891CYPpAJAthphPT3hg1EkUBLhNFoP9HoWW3nug`, tiene seis líneas de largo. `#[program] pub mod transfer_hook_authority {}`, y eso es todo. Apuntaron el slot de hook de solo nacimiento a un no-op y lo dejaron ahí, así que el mint lleva un hook que tiene garantizado no hacer nada en vez de un slot que después se podría apuntar a lógica viva. Un programa vacío que custodia miles de millones es la ilustración más limpia posible de por qué existen las allowlists: desde el asiento de un pool no hay manera de distinguir ese programa de uno que quema 200k unidades de cómputo y llama a otros tres programas, salvo leer y fijar su bytecode.

La seria: PYUSD, el despliegue emblemático de Token-2022, entregado por PayPal y Paxos en mayo de 2024 con un conjunto de extensiones con forma de compliance que incluye un delegado permanente y un transfer hook. La encuesta de Helius sobre el panorama de stablecoins lo puso en $215.9M sostenidos a través de solo 20.4k cuentas de token al 2025-05-29; la supply se mueve a diario, y el único número actual es el que devuelve tu propio `getAccountInfo`. Cada una de esas extensiones de poder está configurada y dormida, el program id del hook null, la comisión cero basis points. Pero contrasta esto con lo que acabas de aprender: dormido no es ausente, así que esas entradas de TLV igual fallan el recorrido de extensiones, y en CP-Swap es el `MINT_WHITELIST` hardcodeado, no la dormancia, lo que deja que PYUSD se enrute. Ese es un token diseñado por gente que entendió el precio de la admisión exactamente: quédate con el interruptor, déjalo apagado, cómprale la buena voluntad al auditor con dormancia, y compra la admisión plataforma por plataforma, por nombre.

## Lab: predice el veredicto, después deja que el fork te chequee

Vas a construir `predict-routability.ts`, el primer borrador de R6, conectarlo a bytes on-chain reales, y poner las dos variantes de SPROUT delante de él. La interfaz de abajo es un contrato: la próxima lección importa `isRoutable` de este archivo exacto con este nombre exacto.

1. **Instala los pins.** En `labs/m05-l1`, con tu surfnet todavía corriendo. La lógica de los pins no cambió desde el módulo 2 y la volví a verificar contra el registry el 2026-09-05: el `latest` de npm para kit es 8.2.0, pero un workspace fija el major de kit contra el que hacen peer sus propias deps `@solana-program/*`, y el `@solana-program/token-2022@0.15.0` de este hace peer con `@solana/kit@^7.0.0`, con `@solana-program/system@0.13.0` como su contraparte. Corre `npm view @solana-program/token-2022@0.15.0 peerDependencies` tú mismo antes de confiar en esa frase; este tren sale cada mes.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx typescript
```

2. **Lee la regla, después córrela.** Clonaste el repo cuando leíste el código fuente. Ahora compila la regla en aislamiento para poder picarla. Guarda esto como `allowlist.rs` al lado de tu lab (es una transcripción de la forma, no una copia del programa: la función real recorre un `StateWithExtensions<Mint>` y hace match con variantes de `ExtensionType`, esta toma los nombres ya decodificados así que se compila con `rustc` a secas y sin árbol de dependencias):

```rust
// allowlist.rs: a standalone transcription of the RULE in raydium-cp-swap,
// programs/cp-swap/src/utils/token.rs L225-236 @ 244e124 (read 2026-08-22).
// The real function walks a StateWithExtensions<Mint> and matches ExtensionType
// variants; this one takes the already-decoded names so it runs with no deps:
//   rustc allowlist.rs && ./allowlist
const MINT_WHITELIST: &[&str] = &[/* the four addresses at token.rs L18-23 */];

fn is_supported_mint(mint: &str, extensions: &[&str]) -> bool {
    if MINT_WHITELIST.contains(&mint) {
        return true;
    }
    extensions.iter().all(|ext| {
        matches!(
            *ext,
            "TransferFeeConfig"
                | "MetadataPointer"
                | "TokenMetadata"
                | "InterestBearingConfig"
                | "ScaledUiAmount"
        )
    })
}

fn main() {
    let sprout = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"];
    let hooked = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"];
    println!("SPROUT       {}", is_supported_mint("SPROUT_MINT", &sprout));
    println!("SPROUT+hook  {}", is_supported_mint("HOOKED_MINT", &hooked));
}
```

```bash
rustc allowlist.rs -o allowlist && ./allowlist
```

Deberías ver `SPROUT true` y `SPROUT+hook false`. Llena las cuatro direcciones de la whitelist desde tu propia salida de `sed` si quieres que la rama de bypass haga algo; a propósito no las imprimo aquí, por la misma razón por la que no te imprimí las reglas de la matriz de conflictos en m01-l4. El territorio es el archivo fuente, no esta página.

Mientras los dos archivos están delante de ti, haz la comparación que hace que esto se quede: pon tu salida de `sed` al lado de la transcripción y marca lo que mi versión dejó fuera. La función real recibe un mint decodificado e itera variantes reales de `ExtensionType`, así que también carga el desempaque, las tuberías de error, y quien la llama y convierte un `false` en una instrucción fallida. Lo que sobrevive a la reducción es la decisión misma, y la decisión tiene cuatro líneas de largo.

![Guía anotada del chequeo de soporte que Raydium CP-Swap corre al crear un pool, que mapea su bypass de whitelist, su match de cinco extensiones y su retorno temprano de false sobre las tres partes del predictor de TypeScript construido en esta lección.](assets/v07-annotated-code.webp)

3. **Escribe el predictor, con dos huecos.** Crea `predict-routability.ts`. Este es el problema de completar: el tipo y la forma de la función están dados, el contenido de la allowlist y las ramas de bypass son tuyos.

```typescript
// predict-routability.ts (skeleton). Two holes to fill from the source you just read.
export interface MintProfile {
  tokenProgram: "spl" | "token2022";
  extensions: string[];
  whitelisted?: boolean;
}

// TODO 1: the five extension names CP-Swap accepts, exactly as token.rs lists them.
export const CP_SWAP_ALLOWLIST: ReadonlySet<string> = new Set([]);

export function isRoutable(mint: MintProfile): boolean {
  // TODO 2: the two bypass branches that run BEFORE the extension check.
  //   (The source has three doors; your profile has two branches, because the
  //   whitelist and the mint-association account both arrive collapsed into
  //   the single `whitelisted` flag. The classic-SPL door is the other branch.)
  return mint.extensions.every((e) => CP_SWAP_ALLOWLIST.has(e));
}
```

Dos cosas que hay que hacer bien, y las dos son preguntas de orden más que de tipeo. Los bypasses corren primero, o un mint con delegado permanente que está en la whitelist recibe un veredicto equivocado. Y la prueba de extensiones es `every`, no `some`: una extensión fuera de la lista contamina el mint entero, porque el programa devuelve false en la primera entrada mala y nunca se recupera.

4. **Llénalo y dale una boca.** Aquí está el archivo terminado. Corre por sí solo y exporta limpio, y por eso la ejecución propia está detrás de una guarda: la próxima lección importa de este módulo y no quiere tu salida de consola.

```typescript
// predict-routability.ts: R6 draft. Reproduces Raydium CP-Swap's pool-creation
// verdict from a mint's token program plus its extension set.
// Modeled on raydium-io/raydium-cp-swap, programs/cp-swap/src/utils/token.rs
// L225-236 (allowlist) and L18-23 (MINT_WHITELIST), commit 244e124, read 2026-08-22.
// Re-read that file at YOUR pinned commit before trusting this file.
import { fileURLToPath } from "node:url";

export interface MintProfile {
  tokenProgram: "spl" | "token2022";
  extensions: string[];
  whitelisted?: boolean;
}

export const CP_SWAP_ALLOWLIST: ReadonlySet<string> = new Set([
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "InterestBearingConfig",
  "ScaledUiAmount",
]);

export function isRoutable(mint: MintProfile): boolean {
  if (mint.tokenProgram === "spl") return true;
  if (mint.whitelisted) return true;
  return mint.extensions.every((e) => CP_SWAP_ALLOWLIST.has(e));
}

export function explain(mint: MintProfile): string {
  if (mint.tokenProgram === "spl") return "classic SPL: extension check skipped";
  if (mint.whitelisted) return "bypass: MINT_WHITELIST or mint-association account";
  const offList = mint.extensions.filter((e) => !CP_SWAP_ALLOWLIST.has(e));
  return offList.length === 0
    ? `all ${mint.extensions.length} extensions on the allowlist`
    : `off-list: ${offList.join(", ")}`;
}

export const SPROUT: MintProfile = {
  tokenProgram: "token2022",
  extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
};

// The DESIGNED hook variant: the full base set plus TransferHook. The variant
// you actually minted in m03 carries TransferHook alone (kept minimal there on
// purpose); the verdict is identical either way, one off-list entry taints it.
export const SPROUT_HOOKED: MintProfile = {
  tokenProgram: "token2022",
  extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"],
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const cases: Array<[string, MintProfile]> = [
    ["SPROUT", SPROUT],
    ["SPROUT+hook", SPROUT_HOOKED],
  ];
  for (const [name, profile] of cases) {
    const verdict = isRoutable(profile) ? "ROUTABLE" : "REJECTED";
    console.log(`${name.padEnd(12)} ${verdict.padEnd(9)} ${explain(profile)}`);
  }
}
```

```bash
npx tsx predict-routability.ts
```

```
SPROUT       ROUTABLE  all 3 extensions on the allowlist
SPROUT+hook  REJECTED  off-list: TransferHook
```

Esa cadena de `explain` no es decoración, y es el mismo argumento que hice para el campo `reason` de `check-combo` allá en m01-l4. Un booleano le dice a un compañero de equipo que no puede lanzar. Una razón le dice qué extensión quitar.

5. **Dale bytes reales.** Hasta ahora juzgaste perfiles tipeados a mano, lo que no prueba nada sobre tus mints reales. Conecta el predictor a la blockchain. `profile-from-mint.ts` lee un mint vivo a través de tu surfnet y construye el perfil a partir del TLV que tu inspector `decode-mint` viene recorriendo desde m01-l2:

```typescript
// profile-from-mint.ts: turn a live mint into a MintProfile the predictor can judge.
// Reads the same TLV bytes your decode-mint inspector has walked since m01-l2.
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint, TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";
import { explain, isRoutable, type MintProfile } from "./predict-routability";

// The four addresses live in token.rs L18-23 at your pinned commit. Read them
// yourself and paste them here; a printed whitelist in a lesson goes stale.
const MINT_WHITELIST: string[] = [];

const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";

export async function profileFromMint(mintAddress: string): Promise<MintProfile> {
  const rpc = createSolanaRpc(RPC_URL);
  const mint = await fetchMint(rpc, address(mintAddress));
  const extensions =
    mint.data.extensions.__option === "Some"
      ? mint.data.extensions.value.map((e) => e.__kind)
      : [];
  return {
    tokenProgram: mint.programAddress === TOKEN_2022_PROGRAM_ADDRESS ? "token2022" : "spl",
    extensions,
    whitelisted: MINT_WHITELIST.includes(mintAddress),
  };
}

const target = process.argv[2];
if (target) {
  const profile = await profileFromMint(target);
  console.log(profile);
  console.log(isRoutable(profile) ? "ROUTABLE" : "REJECTED", "-", explain(profile));
}
```

Córrelo contra tres cosas: el mint de SPROUT que construiste en m02-l4, la variante con hook que acuñaste en el surfnet en m03-l2 (las dos se pueden volver a acuñar según el arranque si tu fork se reinició), y un mint de mainnet que tu fork ya conoce, siendo PYUSD el obvio ya que lo leíste en el módulo 1. Fíjate en lo que el tercer run le hace a tu confianza en el predictor, y guarda ese pensamiento para el paso 7.

```bash
npx tsx profile-from-mint.ts <YOUR_SPROUT_MINT>
npx tsx profile-from-mint.ts <YOUR_HOOKED_SPROUT_MINT>
npx tsx profile-from-mint.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

Esperado para las dos primeras: los mismos veredictos que en el paso 4, SPROUT ROUTABLE y la variante con hook REJECTED, solo que ahora juzgados desde bytes de TLV vivos en vez de un perfil tipeado a mano. Un desajuste es esperado e inofensivo: la variante con hook viva imprime una lista de extensiones de una sola entrada, `TransferHook` sola, donde el perfil `SPROUT_HOOKED` del paso 4 modelaba la variante diseñada de cuatro extensiones. m03 acuñó la variante mínima a propósito, y al veredicto no le importa, porque una sola entrada fuera de la lista contamina el mint sin importar el conjunto que la rodee. El tercer veredicto es el que estás guardando para el paso 7.

![Diagrama de flujo del pipeline desde una dirección de mint, pasando por el predictor de enrutabilidad, hasta un veredicto, con tres artefactos anteriores que alimentan la entrada y un intento de crear un pool en un fork de mainnet que aporta la verdad de campo.](assets/v08-flowchart.webp)

6. **Ahora la parte que puede demostrar que estás equivocado.** Todo hasta aquí es tu modelo del programa. La verdad de campo es el programa. En tu fork de surfnet, el despliegue de CP-Swap y sus cuentas de config son los reales de mainnet, así que un intento de crear un pool es una prueba genuina. Raydium trae un repositorio de demo cuya sección de CPMM construye exactamente esta llamada con `raydium.cpmm.createPool({ programId: CREATE_CPMM_POOL_PROGRAM, poolFeeAccount: CREATE_CPMM_POOL_FEE_ACC, mintA, mintB... })`. Clónalo en una carpeta separada: el SDK de Raydium es código de proveedor que va montado en web3.js v1 y no trae ninguna superficie de kit, así que la dependencia de v1 es inevitable aquí. La regla que sigue este curso, enunciada con suficiente precisión para que puedas chequear un lab posterior contra ella: **pon en cuarentena un SDK de proveedor en la unidad más pequeña que todavía compile, y deja que los dos stacks se encuentren en la blockchain y no en un import compartido.** A veces esa unidad es un workspace entero, como aquí y en el lab de Light de m08-l3, donde cada archivo de la carpeta habla v1 porque el proveedor lo hace y mezclar un segundo SDK en una sola carpeta sería peor. A veces es un solo archivo, como en m09-l1, donde `venue.ts` tiene el único import de web3.js del lab y le pasa valores simples al código de kit de los dos lados. Lo que la regla nunca permite es un archivo propio que importe los dos clientes para ahorrarse una conversión:

```bash
cd .. && git clone https://github.com/raydium-io/raydium-sdk-V2-demo.git
cd raydium-sdk-V2-demo && npm install
```

   Después tres ediciones en los propios archivos de la demo antes de correr nada, porque esta configuración ES el paso que produce tu verdad de campo:

   - `src/config.ts` conecta el cluster y el signer: apunta `connection` a tu surfnet (`http://127.0.0.1:8899`) y carga `owner` desde `~/.config/solana/id.json`.
   - En `src/cpmm/createCpmmPool.ts`, cambia el par `DEVNET_PROGRAM_ID` con el que viene el archivo por las constantes de mainnet `CREATE_CPMM_POOL_PROGRAM` / `CREATE_CPMM_POOL_FEE_ACC` que ya importa; tu fork lleva el despliegue de mainnet, no el de devnet.
   - `mintA` es tu variante de SPROUT; para `mintB` usa WSOL (`So11111111111111111111111111111111111111112`), que el fork ya conoce y que tu payer fondea con `spl-token wrap 1`.

   Después córrelo dos veces, una por cada variante de SPROUT.

Esperado: SPROUT crea un pool, la variante con hook falla dentro del programa. Registra la falla exacta, porque la forma de la falla es el hallazgo: lo que cuenta es un error de programa custom atribuido al program id de CP-Swap en los logs de la transacción, la forma en que quien llama expresa que el chequeo devolvió `Ok(false)`, no una excepción del SDK lanzada antes de que se enviara nada. Y lee esto con honestidad. Hay dos maneras en que este paso se puede torcer y quieren decir cosas distintas. Si la búsqueda de tokens del SDK no puede resolver tu mint local a través de la API alojada de Raydium, eso es una falla del lado del cliente y no el veredicto del programa; aprendiste algo sobre el SDK, nada sobre la allowlist. Solo un error lanzado por el programa cuenta como el programa respondiendo. Si no logras que el camino completo corra hoy, dilo en tus notas en vez de promover la opinión del predictor a evidencia, y toma el camino degradado del paso 7.

7. **Pon la barrera.** Cualquiera sea el camino que te tocó, cierra el loop con un script de asserts, el mismo patrón que `test-check-combo.ts`. Esta es la prueba de aceptación de la lección y codifica la regla entera, bypasses incluidos:

```typescript
// verify-routability.ts: this lesson's gate. Same assert-script pattern as
// test-check-combo.ts from m01-l4: plain asserts, exit 1 on the first miss.
import assert from "node:assert/strict";
import { isRoutable, type MintProfile } from "./predict-routability";

const t22 = (extensions: string[], whitelisted = false): MintProfile => ({
  tokenProgram: "token2022",
  extensions,
  whitelisted,
});

const cases: Array<[string, MintProfile, boolean]> = [
  ["classic SPL, no extensions", { tokenProgram: "spl", extensions: [] }, true],
  ["SPROUT: fee + metadata pair", t22(["TransferFeeConfig", "MetadataPointer", "TokenMetadata"]), true],
  ["SPROUT + harvest hook", t22(["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"]), false],
  ["permanent delegate, unlisted", t22(["PermanentDelegate"]), false],
  ["permanent delegate, whitelisted", t22(["PermanentDelegate", "MetadataPointer"], true), true],
  ["confidential SPROUT branch", t22(["ConfidentialTransferMint"]), false],
  ["scaled UI display only", t22(["ScaledUiAmount"]), true],
];

let passed = 0;
for (const [label, profile, expected] of cases) {
  assert.equal(isRoutable(profile), expected, `${label}: expected ${expected}`);
  passed += 1;
}
console.log(`routability predictor: all ${passed} assertions passed`);
```

```bash
npx tsx verify-routability.ts
```

```
routability predictor: all 7 assertions passed
```

Fíjate en el sexto caso. Tu SPROUT confidencial de la lección pasada también está ahí, y falla, que es la aritmética de la elección que ya tomaste: la variante más privada que construiste es la que ningún AMM va a cotizar nunca. Nada que arreglar. Esa es la forma del espacio de diseño. Y cierra el pensamiento que guardaste del paso 5: PYUSD volvió REJECTED de tu predictor mientras se negocia en esta misma plataforma en mainnet, porque tu constante `MINT_WHITELIST` sigue vacía y la real no. PYUSD se enruta a través de la whitelist hardcodeada, no a través de un recorrido de extensiones que sus extensiones de poder fallarían. El predictor es solo tan actual como las cuatro direcciones que pegas en él, que es la rama de bypass dos ganándose el sueldo.

## Challenge

En solitario, sin apoyo, y es el artefacto de esta lección probado sobre entradas que no elegí yo por ti. Implementa `isRoutable(tokenProgram, extensionList, whitelisted)` para que reproduzca la decisión que CP-Swap toma al crear un pool, para un perfil de mint arbitrario. Una diferencia con tu `predict-routability.ts` local: el calificador pasa el perfil aplanado a tres escalares posicionales en vez de un objeto. `tokenProgram` es `'spl'` o `'token2022'`; `extensionList` son los nombres de tipo de extensión separados por espacios en una sola cadena, `''` cuando el mint no lleva ninguna, divídela tú antes de juzgarla; `whitelisted` es el flag de bypass, true para una entrada de MINT_WHITELIST o una cuenta de asociación de mint inicializada. La misma regla, los mismos cinco nombres, tuberías distintas. El starter que recibes codifica el modelo del folclore, "SPL clásico o ninguna extensión, todo lo demás se rechaza", y falla la suite exactamente donde viven los casos interesantes.

Los criterios de aceptación, directo de la barrera:

- un mint de SPL clásico se enruta sin importar lo que diga su campo de extensiones
- un mint en el que cada extensión está en la allowlist de cinco entradas se enruta
- un mint con cualquier extensión fuera de la lista (`TransferHook`, `PermanentDelegate`, `ConfidentialTransferMint`) se rechaza
- un mint con `PermanentDelegate` que está en la whitelist se enruta por el bypass
- un mint que mezcla una extensión de la allowlist y una fuera de la lista se rechaza

Tres pistas, en el orden en que las vas a necesitar. La allowlist es exactamente cinco nombres. Las ramas de bypass hacen short-circuit antes del chequeo de extensiones. Y el último criterio es el que separa una solución que pasa de una plausible: piensa `every`, no `some`.

Después una extensión del challenge que ninguna prueba puede calificar, y es la que importa en el momento del lanzamiento. Elige cualquier mint de Token-2022 vivo que NO sea tuyo, léelo con `profile-from-mint.ts`, y anota su veredicto más la única frase que vuelve el veredicto accionable para su emisor. Si tu frase nombra una extensión específica y una plataforma específica, estás haciendo el trabajo. Si dice "el soporte de Token-2022 es complicado", estás citando un ticket de soporte.

![Comparación de tres barreras que un token debe pasar, la legalidad de inicialización impuesta por Token-2022, la admisión a la plataforma impuesta por cada DEX, y la visualización en la billetera que no impone nadie, cada una con su modo de falla.](assets/v09-comparison.webp)

## Checkpoint

El criterio de esta lección: `npx tsx verify-routability.ts` en verde en las siete assertions, y las dos variantes de SPROUT pasadas por `profile-from-mint.ts` contra tu fork con los veredictos coincidiendo con lo que predijiste, SPROUT enrutable y la variante con hook rechazada. Si lograste que el camino completo de creación de pool corriera en el paso 6, la respuesta del fork y la respuesta de tu predictor coinciden y tienes evidencia. Si tomaste el camino degradado, tienes un modelo validado contra el código fuente y no contra la ejecución, y la frase honesta para el reporte es "el predictor coincide con token.rs en 244e124; el intento de creación de pool está sin correr". Las dos son aprobaciones. Solo una de las dos es la demostración, y saber cuál tienes en la mano es la habilidad de verdad.

Los errores que espero, en el orden en que suelen pasar. Un caso de delegado permanente en la whitelist que devuelve `false` quiere decir que tus ramas de bypass están debajo del chequeo de extensiones en vez de arriba. Un mint que lleva una extensión fuera de la lista y pasa mientras un mint pelado sin extensiones falla quiere decir que se colaron `some` donde va `every`. Y un run de `profile-from-mint` que reporta cero extensiones en un mint que sabes que lleva tres suele querer decir que lo apuntaste a un clon de SPL clásico de tu mint, o que tu surfnet se reinició y perdió el mint local que habías creado antes. Vuelve a acuñar con los dos comandos que nombra el arranque y vuelve a correr; el fork es barato.

Si tu lectura de `token.rs` no coincide con la mía, los números de línea se movieron, las cinco pasaron a ser cuatro o seis, o la whitelist creció más allá de cuatro entradas, eso no es un bug en tu trabajo, eso es el blanco en movimiento sobre el que esta lección no deja de advertir. Publica el hash del commit y el diff en la discusión del curso. Prefiero que esta página sea corregida por un estudiante que creída por uno.

Ahora puedes leer si cualquier extensión individual mantiene a SPROUT negociable en Raydium, desde el código fuente que lo decide. La próxima lección convierte esa lectura en una decisión de diseño: eliges el conjunto final de extensiones de SPROUT para la plataforma de lanzamiento, lo defiendes desde el lado del pool de la mesa, y escribes el informe de enrutabilidad que dice honestamente dónde se negocia y qué te queda por verificar tú. Feliz verificación.
