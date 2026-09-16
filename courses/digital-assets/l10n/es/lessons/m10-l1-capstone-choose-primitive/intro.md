# Capstone: elige tu primitiva y conecta la economía

## Resumen

Dos lecciones atrás conectaste la migración de compost-points y la restricción de acceso del cNFT Founding-Farmer, que terminó la economía de Overgrowth: un mint, un cNFT, un airdrop y un lector DAS disparando todos en un solo flujo. La lección pasada leíste tokens de producción en vez de escribir uno, y viste que el PYUSD de PayPal lleva ocho extensiones configuradas, cuatro de ellas dormidas según el conteo de tu propio clasificador, con las dos que una plataforma juzgaría primero, la comisión y el hook, deliberadamente dejadas apagadas.

Cada peldaño hasta ahora te entregó la elección. Construye SPROUT con comisión más harvest más metadatos. Acuña la colección Almanac con un plugin Royalties. Dimensiona un árbol a profundidad 14. Esta lección te quita las rueditas. Recibes un brief de producto y nadie te dice a qué primitiva recurrir, porque ese es el trabajo real, y porque la decisión es cara de deshacer: el conjunto de extensiones de un mint queda fijado en la creación (las excepciones estrechas de m02, la escritura de metadatos posterior al init y los reallocs del lado de la cuenta, nunca cubren las extensiones de poder), un mint que la allowlist de Raydium rechaza entra a esa plataforma solo por la excepción de whitelist revisada a mano que compran los emisores regulados, y un cNFT que no puedes leer es un hash que nadie puede resolver.

El repliegue es total. Sin build resuelto. Recibes un scaffold de `verify.ts` fijado al mismo toolchain que usaron los labs, una plantilla de memo y las recetas que ya escribiste. Todo lo demás es tuyo.

Empieza por reclamar un brief. Crea la carpeta y escribe una sola oración dentro antes de leer otro párrafo:

```bash
mkdir -p capstone
printf '# Selection memo\n\nBrief: <the one sentence of product you are building>\n' > capstone/memo.md
```

Hay cinco briefs en el menú, y el quinto es una puerta:

1. **Moneda de lealtad de una cafetería.** Una cadena de seis locales, puntos que se ganan por compra, canjeables en tienda, y el dueño quiere una tajada de cada transferencia entre pares.
2. **Un drop de edición de un músico con regalías.** Impresiones numeradas, una regalía de 5% registrada on-chain, una colección que los compradores pueden verificar.
3. **Un token de pago regulado.** Divisible, tiene que seguir siendo negociable en una plataforma real, y el equipo de cumplimiento necesita una forma de congelar a un mal actor.
4. **Un badge de dispositivo DePIN.** Un badge por dispositivo desplegado, potencialmente un millón de ellos, no transferible, legible por cualquiera.
5. **Tu propio producto.** Las mismas reglas, la misma demostración.

Elige ahora. El resto de esta lección asume que tienes uno enfrente.

## Elegir la primitiva y después defenderla

### Las cuatro restricciones escondidas en todo brief

Los briefs de producto no dicen "usa Token-2022 con un delegado permanente." Dicen cosas como "el dueño quiere una tajada" y "el equipo de cumplimiento necesita una forma de congelar." Tu primer movimiento es una traducción, y solo hay cuatro preguntas que vale la pena hacer, porque entre ellas eligen la familia.

**¿Es divisible?** Si un tenedor puede tener 0.4 de esto, necesitas un mint fungible, y solo dos familias te dan uno: SPL clásico y Token-2022. Los activos Core y los cNFT son NFT. Esa sola pregunta mata más diseños candidatos que las otras tres juntas, y los mata gratis, antes de que hayas escrito una línea.

**¿Tiene que seguir siendo negociable?** No "estaría bien," sino que tiene que. Un punto de lealtad canjeado en un mostrador no tiene ningún requisito de plataforma. Un token de pago que no puede entrar a un pool es un token de pago que nadie puede cotizar. En el momento en que la respuesta es sí, la tesis de compatibilidad que viene de la lección del token enrutable se apodera de tu lista de extensiones, y ya no estás eligiendo libremente.

**¿Cuántos van a existir?** Uno, mil o un millón. Entre mil y un millón está la línea donde la compresión deja de ser ingeniosa y pasa a ser la única opción, y el costo de acuñar es todo el argumento.

**¿Quién tiene que poder hacerle qué después de que se entregue?** Congelar, confiscar, impedir transferencias, actualizar metadatos, quedarse con una tajada. Cada una de esas cosas se mapea a una extensión o un plugin con nombre, y cada una es un poder que cuesta compatibilidad. Esta es la pregunta que convierte un brief en un conjunto.

Responde esas cuatro por escrito, en tu memo, antes de tocar una receta. He visto gente (yo incluido, en un fin de semana de hackathon que preferiría no volver a litigar) acuñar primero y responder después, y el resultado siempre es el mismo: un mint con el conjunto de extensiones equivocado y una reacuñación el domingo por la mañana.

![Un diagrama de flujo de decisión donde la divisibilidad elige fungible versus NFT, la negociabilidad restringe el conjunto de extensiones, la cantidad elige Core versus NFT comprimidos, y una cuarta pregunta sobre los poderes posteriores a la entrega abarca todos los resultados.](assets/v01-flowchart.webp)

### La matriz te rechaza antes que el mercado

Dos barreras se interponen entre un conjunto y un mint publicado, y se disparan en este orden.

La primera es la matriz de conflictos que portaste en el módulo 1. Cinco reglas, sacadas directo de `check_for_invalid_mint_extension_combinations`, y no son consejos. Un conjunto que viola una de ellas falla en `initialize_mint`, on-chain, con un error `InvalidExtensionCombination` — y como el flujo enseñado agrupa create-account, init-extensions e `initialize_mint` en una sola transacción, todo revierte atómicamente: los lamports de rent rebotan de vuelta a tu pagador y solo pierdes la comisión de transacción, más el rediseño. Tu función `checkCombo` ya codifica las cinco, así que el costo de preguntar es un import y una llamada. Pregunta.

La segunda barrera es la allowlist de la plataforma, y es más estricta que la matriz en el sentido que más importa: la matriz te dice qué rechaza el token program, y la allowlist te dice qué rechaza el *mercado*. Son fallas distintas. La primera pasa en un segundo, en devnet, gratis. La segunda pasa semanas después, cuando alguien intenta crear un pool y descubre que tu mint no puede tener uno.

Y debajo de las dos está la trampa que hace que esta lección exista, la regla de solo-al-nacer del párrafo inicial: las extensiones se habilitan en la creación del mint, con las dos rutas estrechas posteriores al init de m02 como únicas excepciones. Toda extensión de poder en el mint es solo-al-nacer. No hay migración. No hay parche. Si te equivocaste con el conjunto, acuñas un token nuevo y mueves a todos a él, lo cual es un evento de producto, no un deploy.

Piénsalo como fundir una campana. Todo lo del tono se decide en el molde, en una sola colada, y una vez que el metal está frío tu única herramienta restante es una amoladora. Puedes afinar una campana después de fundirla. No puedes convertirla en otra campana.

### La tesis de compatibilidad, de boca de Raydium

Esta es la oración que debería vivir en tu memo. La referencia de Token-2022 de Raydium (leída el 2026-08-21) no se esconde detrás de lenguaje de políticas. Rechaza `PermanentDelegate` porque un tenedor del delegado puede barrer cualquier cuenta de token, *incluido el vault del pool*, y rechaza `TransferHook` porque el hook invoca un programa personalizado en cada transferencia, con consumo de cómputo arbitrario.

Lee esas dos razones otra vez, porque generalizan más allá de Raydium. Las dos son la misma queja desde el asiento de un integrador: tu extensión pasó una decisión que antes le pertenecía al pool a las manos de alguien que el pool no puede auditar. Una plataforma que lista tu token está tomando custodia de él dentro de un vault. Cualquier cosa que te deje meter la mano en ese vault, o hacer que una transferencia cueste una cantidad ilimitada de cómputo, es un riesgo que no aceptó correr.

Por eso la allowlist de CP-Swap contiene exactamente cinco extensiones, y por eso son las aburridas: comisiones, puntero de metadatos, metadatos de token, config de interés acumulable, monto de UI escalado. La visualización y la contabilidad entran. El poder queda rechazado.

Así que la tesis, en una línea que le puedes entregar a un product manager: las extensiones de comisión, de visualización y de contabilidad entran en la allowlist, las extensiones de poder quedan rechazadas. Fíjate en la palabra que esa línea NUNCA debe contener: compliance. Las extensiones con forma de compliance, PermanentDelegate y DefaultAccountState, son exactamente los poderes que la allowlist rechaza, y un PM que se va diciendo "compliance entra" entrega la regla equivocada. Cualquier otra cosa que quieras, o la impones fuera de la plataforma, o te compras la entrada a una whitelist estática como hacen los emisores regulados.

![Una tabla comparativa que puntúa SPL clásico, Token-2022, Metaplex Core y los cNFT de Bubblegum v2 en divisibilidad, costo por unidad, poderes posteriores a la entrega disponibles, enrutabilidad en DEX, y qué necesita un lector para resolverlos.](assets/v02-comparison.webp)

### El eje de costo es donde se decide el brief de acuñación masiva

Los números que zanjan el brief del badge DePIN salieron de tus propios labs y no de una página de marketing.

Un mint de Metaplex Core cuesta ~0.003 SOL según la cifra que publicó el proveedor, una cuenta, sin PDA de metadatos, sin PDA de master edition. Esa es la opción barata sin comprimir, y la cifra la publicó Metaplex, no este curso — tilde incluida, porque su tabla da una aproximación y endurecerla hasta un cuarto dígito sería este curso inventando precisión en su nombre.

El árbol que dimensionaste en el módulo 7 a profundidad 14, buffer 64, canopy 8 pesa 48,120 bytes y aguanta 16,384 hojas: 0.245 SOL a la tasa de rent de devnet el 2026-09-06, o sea unos 0.000015 SOL por badge, unas 200 veces más barato por unidad que Core, pagado por adelantado, por una capacidad fija a la que te comprometes en la creación. Ese múltiplo depende de la tasa de una forma en que los de cNFT-a-cNFT no, porque solo un lado de él es rent: en los 6,960 previos al SIMD-0437 marcaba unas 150 veces.

No extrapoles ese número por hoja, y esta es la parte donde la gente tropieza. El rent de un árbol no es lineal en la cantidad de hojas. Una cuenta de árbol de Merkle concurrente se dimensiona por profundidad, buffer y canopy, no por cuántas hojas piensas llenar, así que comprar capacidad es casi gratis mientras que comprar canopy no lo es. El árbol de un millón de hojas del módulo 7, profundidad 20 con buffer 256 y canopy 14, pesa 1,223,352 bytes para 1,048,576 hojas, que dieron 6.215 SOL a la tasa de rent de devnet el 2026-09-06 y habrían sido 8.515 SOL antes de que el SIMD-0437 empezara a recortar esa tasa. Ponle precio en tu propio cluster; los bytes son la mitad durable. De cualquier forma son unas pocas millonésimas de SOL por badge, cientos de veces más barato por unidad que Core. El árbol más grande es el más barato por hoja, que es al revés de toda otra intuición de rent que tengas, y es por eso que al brief de acuñación masiva se le pone precio contra el árbol que construirías de verdad y no contra el que construiste para practicar.

Un millón de dispositivos a precios de Core son unos 3,000 SOL. Un millón de dispositivos en un solo árbol son SOL de un solo dígito. No hay argumento de diseño que sobreviva a esa proporción, que es lo útil de las brechas de orden de magnitud: terminan debates en vez de empezarlos.

La factura llega del lado de la lectura, y es la cuarta trampa del curso. La huella on-chain de un cNFT es un hash de hoja. El activo en sí lo reconstruyen los indexadores DAS a partir de almacenes de datos que administra el RPC. Apunta un script de verificación a un RPC sin soporte DAS y `getAsset` no devuelve nada, para un activo que se acuñó perfecto, y vas a pasar veinte minutos sospechando del id de tu activo. Ese costo no lo puedes refactorizar después. Es una dependencia que aceptas en tiempo de diseño, y va en el memo justo al lado de la línea de rent del árbol.

![Un gráfico de barras en escala logarítmica que compara el costo por activo de un mint de Metaplex Core contra hojas de NFT comprimidos en dos tamaños de árbol, mostrando brechas de dos y casi tres órdenes de magnitud.](assets/v03-chart.webp)

### Los briefs de NFT tienen exactamente una ruta de entrega

Si tu brief cayó del lado NFT de esa primera pregunta, la familia está decidida y el estándar, en su mayor parte, también.

Token Metadata es oficialmente legacy. Metaplex Core es el estándar recomendado para trabajo nuevo de NFT, Bubblegum v2 para comprimidos, y la lección de legacy del módulo 6 es donde vive el argumento completo, así que no lo voy a volver a litigar aquí. Lo que importa para un memo de selección es la única oración a la que un revisor te va a objetar: los pNFT imponen regalías a través de Token Auth Rules, que Metaplex marca como deprecado y cuyo rule set emblemático bloquea cero programas. Armado y ocioso. Si eliges ese camino para la imposición, entregaste un stack deprecado para conseguir una garantía que hoy no está garantizando nada, y `seller_fee_basis_points` en un NFT clásico de Token Metadata es puramente indicativo de todas formas.

Así que el drop del músico se resuelve en una colección Core con el plugin Royalties llevando `basisPoints` y una lista de creators que suma exactamente 100, más el plugin Edition para impresiones numeradas, que es justo el paso de edición resuelto que ya construiste en el Almanac. Escribe la realidad de la imposición en el memo con palabras llanas: el reparto queda registrado on-chain, legible por todos, y lo honra la política del marketplace y no el token program. Un comprador merece saber eso antes de comprar, y un memo que finge lo contrario es la clase de documento que envejece mal.

El badge de dispositivo se resuelve para el otro lado, en un árbol de Bubblegum v2 con hojas soulbound, y su sección del memo trata de capacidad de árbol y dependencia de lectura, no de regalías en absoluto.

![Un diagrama de flujo que resuelve los briefs de NFT hacia una colección Core con los plugins Royalties y Edition o hacia un árbol soulbound de Bubblegum v2, con la ruta deprecada del pNFT como callejón sin salida.](assets/v04-flowchart.webp)

### PYUSD es el brief regulado, ya resuelto

Si tomaste el brief tres, alguien ya entregó tu token, y lo puedes leer.

PayPal y Paxos lanzaron PYUSD en Solana en mayo de 2024 como un mint de Token-2022 en `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`. Lleva ocho extensiones TLV: autoridad de cierre del mint, delegado permanente, config de comisión de transferencia, el par de transferencia confidencial, transfer hook, puntero de metadatos y metadatos de token. Volví a leer el mint el 2026-08-22 y obtuve las mismas ocho, seis decimales, sin cambios.

Ahora la parte que lo vuelve una lección y no un caso de estudio. `transferHook.programId` es null. No hay ningún programa de hook configurado. La comisión es 0 basis points con un máximo de 0. Y tu clasificador de m09-l3 contó el mint en 4 activas, 4 dormidas: el par confidencial está apagado de la misma forma, aprobación manual encendida y nadie aprobó. Esta lección separa la comisión y el hook de esas cuatro dormidas porque son el par que un recorrido de la allowlist pesaría de verdad; el par confidencial es maquinaria dormida que un pool nunca inspecciona. De cualquier forma, los poderes están configurados y dormidos, lo que quiere decir que el emisor pagó el costo de tamaño de cuenta y el costo de compatibilidad para tener opciones que no ha ejercido.

Esa es una estrategia deliberada y deberías nombrarla como tal si la copias. Configurar una extensión es una decisión permanente. Activarla es una reversible. Por eso un emisor regulado carga por adelantado todo poder que pudiera llegar a necesitar en el mint, y después deja los interruptores apagados, porque la alternativa es descubrir en el año dos que su obligación de cumplimiento exige una extensión que no puede agregar.

El costo es exactamente lo que predice la tesis: un mint que lleva un delegado permanente queda rechazado por un programa de pool sin permiso por dormido que esté ese delegado, porque la allowlist lee el tipo de extensión, no tus intenciones. PYUSD se negocia igual. Llegó ahí por el camino que un token de curso no tiene, que es lo honesto para escribir en tu memo si tomas este brief: tu conjunto de compliance es defendible, y tu ruta a una plataforma es una conversación de negocios, no una transacción.

![Una lectura anotada del mint de PYUSD que muestra ocho extensiones configuradas, cuatro de ellas dormidas, bajo la regla de que configurar es permanente mientras que activar es reversible.](assets/v05-annotated-code.webp)

### La sección de plataforma deriva su propio número

Tu memo necesita una sección de plataforma de lanzamiento, y consume la config de lanzamiento que construiste en el módulo 8 en vez de repetir un número sacado de un blog.

El número en cuestión es 85 SOL, el umbral de graduación que todo el mundo cita para una moneda de pump.fun. Tu `sprout-launch/derive-graduation.ts` no lo guarda. Lo calcula, a partir de tres constantes publicadas y la invariante de producto constante: 30 SOL virtuales, 1,073,000,000 tokens virtuales, 793,100,000 tokens reales. Drena la reserva real y el lado de tokens virtuales queda en 279,900,000, así que el lado final de SOL virtuales es 30 por 1,073 sobre 279.9, unos 115.005, y el SOL que tuvo que entrar es 115.005 menos 30. Eso es 85.005.

Que es todo el punto. El umbral es consecuencia de una curva que configuró otra persona, no una constante de la naturaleza, y la misma función devuelve 120 SOL para la curva alternativa que probaste en el módulo 8: 30 SOL virtuales, 1,000,000,000 tokens virtuales, 800,000,000 tokens reales. Cambia cualquiera de las dos constantes del lado del token y el número se mueve. Tu memo no afirma 85. Corre la función e imprime lo que producen las constantes de hoy, para que cuando las constantes cambien tu memo esté equivocado de forma ruidosa y no de forma callada.

![Una tabla de derivación que lleva las tres constantes de curva publicadas por pump por la invariante de producto constante hasta una reserva final de SOL virtuales de 115.005 y un umbral de graduación de 85.005 SOL.](assets/v06-table.webp)

La sección de plataforma también hace un trabajo que no tiene nada que ver con curvas: pregunta si la plataforma puede sostener siquiera el token que elegiste. `checkGraduationVenue` rechaza el camino de pump para cualquier mint de Token-2022, porque la instrucción `create` fija el token program clásico, y acepta CP-Swap solo cuando cada extensión de tu conjunto está en la allowlist de cinco ítems. Córrela contra tu propio conjunto declarado y pega la salida. Un veredicto de plataforma calculado a partir de tu conjunto vale más que tres párrafos de prosa sobre enrutabilidad, y toma un solo comando.

La composición de pools, el enrutamiento y la estrategia de LP para lo que sea que listes son otra disciplina, y el curso planificado DeFi and RWA Engineering las enseña como corresponde. Tu memo se detiene en "esta plataforma puede sostener este token, y este es el umbral con las constantes de hoy."

### Exactamente un riel, y por qué exactamente uno

Hay cuatro rieles en el menú enseñado: tres salieron del módulo de economía, y el cuarto, el airdrop, del compost drop de m08-l3. Conectas exactamente uno de los cuatro. El enrutamiento de comisiones hace harvest de los montos retenidos que una comisión de transferencia acumula hacia una tesorería. Un acceso restringido revisa si una billetera tiene cierta tenencia y le da o le niega el acceso. Un airdrop distribuye contra una raíz de Merkle con una ruta de reclamo. Un buyback gasta SOL de la tesorería en un swap del lado del cliente y quema lo que compró.

La mayoría de los briefs tiene un encaje obvio. Que el dueño de la cafetería quiera una tajada de las transferencias entre pares es una ruta de comisión, porque la comisión ya se está acumulando en las cuentas de los destinatarios y el riel es el harvest. El drop del músico es un acceso restringido si los tenedores reciben algo, o un airdrop si el drop es la distribución. El badge de dispositivo es un acceso restringido casi por definición, ya que el badge existe para autorizar un dispositivo. El token regulado es una ruta de comisión o nada, porque el riel de compliance es off-chain por construcción — y si lo tomas, ten en cuenta que el conjunto de compliance avalado lleva su comisión en 0 bps, que no retiene nada: pon una comisión simbólica distinta de cero (1 bps alcanza) para el riel conectado, o tu impresión de antes-y-después no puede diferir.

La razón de que sea uno y no tres no es la carga de trabajo. Es la demostración. Un riel cuenta solo cuando un script imprime un antes y un después, y tres rieles conectados a medias producen cero de esos mientras que un riel terminado produce un recibo. Elige el riel cuyo antes-y-después de verdad puedas hacer visible en una terminal, y conecta ese como se debe.

![Una tabla comparativa de los cuatro rieles de economía enseñados, que lista qué mueve cada uno, la demostración que un script debe imprimir, y el brief de producto al que mejor le calza cada uno.](assets/v07-comparison.webp)

### Nombrar el canje que aceptaste es el memo

Todo lo de arriba colapsa en un solo meta-tradeoff, y escribirlo es el entregable.

Cada elección de primitiva cambia poder por compatibilidad. Un mint de Token-2022 con un delegado permanente te compra control de compliance y queda rechazado por un pool sin permiso. Un activo Core es barato y rico en plugins y no es un token fungible, así que ninguna cantidad de ingenio con plugins lo vuelve divisible. Un cNFT es casi gratis a un millón de unidades y necesita un RPC con DAS para poder leerse siquiera, y no lleva estado de cuenta arbitrario del que puedas colgar un programa.

No hay posición gratis en esa curva. El memo no es donde afirmas que encontraste una. Es donde escribes, en un párrafo, a qué poder renunciaste y qué conseguiste a cambio, para que la persona que herede tu token en dieciocho meses pueda distinguir entre una restricción y un accidente.

![Un diagrama radial con el capstone en el centro, alimentado por nueve artefactos etiquetados de lecciones anteriores del curso, y marcado como terminal, sin consumidor aguas abajo.](assets/v08-diagram.webp)

## Lab: memo, activo, riel, demostración

El criterio de esta lección es que `npx tsx capstone/verify.ts $ASSET_ADDRESS` salga con 0. Todo lo que viene antes es tuyo para enrutar.

Una regla permanente, y es calificada: reutiliza solo habilidades que este curso enseñó. Ninguna dependencia que no hayas instalado ya en un lab anterior. Si tu diseño necesita un paquete que este curso nunca presentó, el diseño queda fuera del alcance del capstone, y esa restricción te está haciendo un favor al mantener la superficie lo bastante chica como para terminarla de verdad.

**1. Prepara todo, con las mismas versiones fijadas que usaron los labs.** Instala en el workspace del curso, no en uno nuevo, porque `verify.ts` importa el lector que ya escribiste:

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Nota de actualidad, ya que estas versiones fijadas son las primeras que vas a volver a revisar cuando algo se rompa dentro de un año. El 2026-09-05 el `latest` de npm para `@solana/kit` era 8.2.0, y este curso fija 7.1.1 de todos modos: `@solana-program/token-2022@0.15.0` declara un rango de peers de `^7.0.0`, así que el par de arriba es la combinación válida en peers para el código que ya escribiste. Nunca instales "latest" aquí. Revisa el rango de peers y fija el par exacto.

**2. Escribe primero la mitad legible por máquina del memo.** Los memos en prosa se apartan de la realidad; un memo JSON se compara contra la blockchain. Este es el archivo que lee `verify.ts`:

```typescript
// capstone/memo.ts: the memo's machine-readable half (R13).
// The prose memo is for humans. This file is the part verify.ts reads.
import { readFileSync } from 'node:fs';

export type PrimitiveFamily = 'spl-token' | 'token-2022' | 'core-asset' | 'cnft';

export interface SelectionMemo {
  /** One line of product, in the brief's own words. */
  product: string;
  primitiveFamily: PrimitiveFamily;
  /**
   * Token families: extension kinds, spelled the way the Token-2022 client
   * spells them. core-asset: plugin names. cnft: the structural tags DAS
   * can actually return.
   */
  declaredSet: string[];
  /** Exactly one, from the rails this course taught. */
  rail: 'fee-route' | 'gate' | 'airdrop' | 'buyback';
  /** The power you gave up, or the compatibility you gave up. In writing. */
  tradeoff: string;
  /** The address you shipped. verify.ts cross-checks its argv against this. */
  assetAddress: string;
}

// The full cNFT tag menu verify.ts understands. Leave this constant alone:
// your declaration happens in memo.json, and it must list only the tags YOUR
// mint will actually produce. 'compressed' always appears; 'collection'
// appears only if you mint the leaf INTO a collection. A badge minted
// collectionless comes back as ['compressed'] alone, so declare just that, or
// mint under a collection and declare both. Neither is wrong for the badge
// brief; verify.ts holds you to whichever you declared.
export const CNFT_TAGS = ['compressed', 'collection'];

export function loadMemo(path = 'capstone/memo.json'): SelectionMemo {
  let raw: string;
  try {
    raw = readFileSync(path, 'utf8');
  } catch {
    throw new Error(`${path} not found. Fill it from the template in this lesson.`);
  }
  const memo = JSON.parse(raw) as SelectionMemo;
  if (memo.declaredSet.length === 0 && memo.primitiveFamily !== 'spl-token') {
    throw new Error(
      `${path}: declaredSet is empty for ${memo.primitiveFamily}. An empty set is a claim too, so make it on purpose.`,
    );
  }
  return memo;
}
```

Esa única guarda existe por un modo de falla con el que quiero que te topes aquí y no después. Un `declaredSet` vacío en un mint de Token-2022 normalmente quiere decir que alguien se saltó el paso de diseño, así que lanza un error a menos que la familia sea SPL clásico, donde vacío es la respuesta correcta. Fíjate en lo que `loadMemo` deliberadamente *no* revisa: `assetAddress`. El memo está legítimamente sin dirección hasta el paso 5, porque el trabajo de diseño de los pasos 3 y 4 ocurre antes de que se acuñe nada. El requisito de la dirección le pertenece a `verify.ts`, el único consumidor para el cual un memo que verifica contra nada es un documento, no una demostración, y lo vas a ver imponerlo por su cuenta.

**3. Llena `capstone/memo.json` antes de acuñar.** El orden importa. Este archivo es una predicción, y acuñar es el experimento:

```json
{
  "product": "Cafe loyalty currency for a six-store chain",
  "primitiveFamily": "token-2022",
  "declaredSet": ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
  "rail": "fee-route",
  "tradeoff": "no PermanentDelegate, so no clawback if a wallet is compromised; in exchange the mint stays poolable on CP-Swap with no whitelist request",
  "assetAddress": ""
}
```

Después pasa el conjunto por la matriz antes de que llegue siquiera a una transacción. Este se lleva su propio archivo, ya que saltárselo es el descuido que te cuesta un mint: guárdalo como `capstone/precheck.ts` y corre `npx tsx capstone/precheck.ts`. Un import, una llamada, y atrapa los errores de combinación que si no te costarían un mint:

```typescript
// capstone/precheck.ts - run before any lamport moves: npx tsx capstone/precheck.ts
import { checkCombo } from '../check-combo';
import { loadMemo } from './memo';

const memo = loadMemo();
const combo = checkCombo(memo.declaredSet);
if (!combo.valid) {
  throw new Error(`declared set is invalid on chain: ${combo.reason}`);
}
```

**4. Calcula la sección de plataforma.** Esta es la parte del memo que consume la config de lanzamiento por su nombre, y es corta porque el trabajo se hizo hace dos módulos:

```typescript
// capstone/venue.ts: the memo's launch-venue section, computed not asserted.
// Run: npx tsx capstone/venue.ts
// Consumes R10 (sprout-launch/derive-graduation.ts) by name.
import {
  PUMP_REFERENCE_CURVE,
  VENUES,
  checkGraduationVenue,
  finalReserves,
  graduationSol,
  spotPrice,
} from '../sprout-launch/derive-graduation';
import { loadMemo } from './memo';

function fmt(n: number, places = 3): string {
  return n.toLocaleString('en-US', {
    minimumFractionDigits: places,
    maximumFractionDigits: places,
  });
}

function main(): void {
  const memo = loadMemo();
  const c = PUMP_REFERENCE_CURVE;
  const f = finalReserves(c);
  const grad = graduationSol(c);
  const open = spotPrice(c.virtualSolReserves, c.virtualTokenReserves);
  const close = spotPrice(f.finalVirtualSol, f.finalVirtualToken);

  console.log('## Launch venue\n');
  console.log(`Product:            ${memo.product}`);
  console.log(`Primitive family:   ${memo.primitiveFamily}`);
  console.log(`Declared set:       ${memo.declaredSet.join(', ') || '(none)'}\n`);

  console.log('Reference curve, derived live from the published constants:');
  console.log(`  virtual SOL / virtual tokens / real tokens: ${c.virtualSolReserves} / ${fmt(c.virtualTokenReserves, 0)} / ${fmt(c.realTokenReserves, 0)}`);
  console.log(`  final virtual SOL:      ${fmt(f.finalVirtualSol)} SOL`);
  console.log(`  SOL added to graduate:  ${fmt(grad)} SOL`);
  console.log(`  price multiple:         ${fmt(close / open, 2)}x\n`);

  if (memo.primitiveFamily === 'core-asset' || memo.primitiveFamily === 'cnft') {
    console.log(
      `A ${memo.primitiveFamily} is not a fungible mint, so no curve venue applies. Say that in the memo and name where it trades instead.`,
    );
    return;
  }

  if (memo.primitiveFamily === 'spl-token') {
    console.log(
      'Declared family is classic SPL. checkGraduationVenue was written against SPROUT, a Token-2022 mint, and it hardcodes that assumption in its reason strings, so running it here prints a refusal that is about SPROUT rather than about you. A classic mint carries no extensions for a venue to refuse, so both taught venues accept it by construction. Write that sentence in the memo instead of a verdict table.',
    );
    return;
  }

  console.log('Venue verdicts for the declared set:\n');
  let anyAccepted = false;
  for (const venue of VENUES) {
    const verdict = checkGraduationVenue(venue, memo.declaredSet);
    anyAccepted = anyAccepted || verdict.accepted;
    console.log(`  ${verdict.accepted ? 'ACCEPTS ' : 'REFUSES '} ${verdict.venue}`);
    for (const reason of verdict.reasons) {
      console.log(`            ${reason}`);
    }
    console.log(`            source: ${venue.source}`);
  }

  if (!anyAccepted) {
    console.log(
      '\nNo taught venue accepts this set. That is a finding, not a bug: write it in the memo, or change the set.',
    );
  }
}

main();
```

`npx tsx capstone/venue.ts` imprime 85.005 SOL para la curva de referencia y una línea de veredicto por plataforma. Si tu brief es fungible, pega esa salida directo en `memo.md`. Si tomaste el drop del músico o el badge de dispositivo, pega solo la línea de que no aplica ninguna plataforma de curva: la derivación de la curva de referencia que está arriba se imprime para calibración en cada corrida, y la matemática de graduación en un memo de badge es justo la clase de residuo fuera de spec que un revisor marca.

Dos de las ramas de ese archivo existen porque un artefacto reutilizado fuera de su supuesto original te va a mentir en vez de dar error. Si tomaste el drop del músico o el badge de dispositivo, no aplica ninguna plataforma de curva, y decirlo vale más que inventar una. Y si tomaste SPL clásico, fíjate en que `checkGraduationVenue` no es un oráculo general: se escribió para SPROUT, así que su primerísima prueba pregunta si la plataforma habla Token-2022 y su cadena de razón nombra a SPROUT en voz alta. Apúntalo a un mint clásico y rechazaría pump.fun, justo la plataforma en la que un mint clásico es bienvenido. Por eso la rama de spl-token en el código de arriba se salta la llamada por completo: tu corrida imprimió la oración honesta en vez de un rechazo fuera de spec. Eso no es un bug en la función, es una función usada fuera de su spec, y atraparlo es la clase de cosa para la que existe un capstone.

**5. Entrega el activo, a partir de una receta que ya escribiste.** Aquí no hay mecanismo nuevo, que es el punto de un capstone. Mint de Token-2022: tu receta del módulo 2 con tu conjunto declarado. Colección Core con Royalties e impresiones numeradas: tu receta del módulo 6, y el plugin Edition es la pieza a la que el brief del músico apuntaba desde el principio. Badge cNFT: tu árbol del módulo 7, dimensionado con tu propia matemática, con hojas soulbound si el brief dice no transferible.

Escribe la dirección resultante en `memo.json` en cuanto aparezca, y después deja el memo en paz. Editar el memo después de ver la blockchain es cómo una demostración se vuelve un trámite.

Devnet es el cluster correcto para esto y no deberías sentirte mal por eso. La verificación es idéntica, el estado de extensiones es idéntico, y lo único que mainnet agregaría es una factura. La única advertencia honesta es el campo de precio: un mint de devnet va a resolver, va a clasificar como fungible, y va a volver sin `price_info`, porque el conjunto con precio son alrededor de los diez mil tokens con más volumen en 24 horas. Ese es el caso normal y tu script no depende de él.

**6. Conecta exactamente un riel.** Uno. No dos porque te sobra tiempo. Ruta de comisión: haz harvest de los montos retenidos hacia una tesorería y muestra el movimiento del saldo. Acceso restringido: revisa si una billetera tiene la tenencia y niégale el acceso. Airdrop: la ruta de reclamo de Merkle con un reclamo real. Buyback: la llamada de swap del lado del cliente contra la plataforma, y después la quema. Demuéstralo en un script que imprima un antes y un después, porque un riel que nadie puede ver ejecutarse es un diagrama.

Nombra el script según el riel, `capstone/rail-fee-route.ts` o `capstone/rail-gate.ts`, y haz que salga con un código distinto de cero cuando el después no difiere del antes. Esa última parte importa más de lo que parece. Un script de riel que loguea y siempre sale con 0 va a reportar éxito alegremente contra una transacción que nunca llegó, y no lo vas a notar hasta que otra persona lo corra.

**7. Escribe `capstone/verify.ts`.** El scaffold está abajo, completo y ejecutable. Lee la bifurcación de tres vías en `liveState` antes de correrlo, porque esa bifurcación es toda la lección comprimida en treinta líneas: tres familias de primitivas, tres definiciones completamente distintas de qué quiere decir siquiera "estado en vivo".

```typescript
// capstone/verify.ts: R13's integration proof.
// Run: npx tsx capstone/verify.ts <ASSET_ADDRESS>
// Exit 0 means: DAS resolved the asset, its interface matches the family the
// memo claims, and its live on-chain state equals the memo's declared set.
import { address, createSolanaRpc } from '@solana/kit';
import { fetchMint } from '@solana-program/token-2022';
import { das } from '../overgrowth/das';
import { classifyAsset, type DasAsset } from '../overgrowth/classify';
import { readExtensionState } from '../overgrowth/extension-state';
import { CNFT_TAGS, loadMemo, type SelectionMemo } from './memo';

const TOKEN_2022 = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const TOKEN_CLASSIC = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

interface CapstoneAsset extends DasAsset {
  id: string;
  content?: { metadata?: { name?: string } };
  ownership?: { owner?: string };
  grouping?: { group_key?: string; group_value?: string }[];
  compression?: { compressed?: boolean; tree?: string; leaf_id?: number };
  token_info?: {
    decimals?: number;
    token_program?: string;
    price_info?: { price_per_token?: number };
  };
  plugins?: Record<string, unknown>;
}

interface LiveState {
  set: string[];
  notes: string[];
}

function expectedInterfaceFamily(memo: SelectionMemo): (asset: CapstoneAsset) => string | null {
  return (asset) => {
    const c = classifyAsset(asset);
    switch (memo.primitiveFamily) {
      case 'core-asset':
        return asset.interface === 'MplCoreAsset'
          ? null
          : `memo says core-asset, DAS says interface=${asset.interface}`;
      case 'cnft':
        return c.compressed
          ? null
          : `memo says cnft, DAS says compressed=false (interface=${asset.interface})`;
      case 'token-2022':
      case 'spl-token': {
        if (c.category !== 'fungible') {
          return `memo says ${memo.primitiveFamily}, DAS classifies this as ${c.category}`;
        }
        const want = memo.primitiveFamily === 'token-2022' ? TOKEN_2022 : TOKEN_CLASSIC;
        const got = asset.token_info?.token_program;
        if (!got) {
          // Same philosophy as the Core branch below: a silent pass on an
          // unread field is worse than no verification at all.
          return `DAS returned no token_info.token_program to check the declared family against; use an endpoint that returns it`;
        }
        if (got !== want) {
          return `memo says ${memo.primitiveFamily}, token_program is ${got}`;
        }
        return null;
      }
    }
  };
}

async function liveState(memo: SelectionMemo, asset: CapstoneAsset): Promise<LiveState> {
  if (memo.primitiveFamily === 'core-asset') {
    if (!asset.plugins) {
      throw new Error(
        'this endpoint returned no plugins object for a Core asset, so the memo cannot be checked against it. Try another DAS provider.',
      );
    }
    return { set: Object.keys(asset.plugins), notes: ['plugin names read from the DAS payload'] };
  }

  if (memo.primitiveFamily === 'cnft') {
    const set: string[] = [];
    if (asset.compression?.compressed === true) set.push('compressed');
    if ((asset.grouping ?? []).some((g) => g.group_key === 'collection')) set.push('collection');
    return {
      set,
      notes: [
        `a cNFT leaf has no extension or plugin account, so the live set is the structural tags DAS returns: ${CNFT_TAGS.join(', ')}`,
        `tree=${asset.compression?.tree ?? '(none)'} leaf_id=${asset.compression?.leaf_id ?? '(none)'}`,
        'BLIND SPOT, on record: these tags cannot see soulbound-ness. If the brief hinges on non-transferable (brief 4), this gate cannot check it; prove it with a transfer-must-fail script the way m07-l1 taught, and say so in the memo.',
      ],
    };
  }

  const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
  // Off-spec reuse, named, since venue.ts just got a lecture about exactly
  // this: for the 'spl-token' family this calls the token-2022 client's
  // fetchMint against a classic mint. That works because the 82-byte base
  // layout is shared between the two programs and a classic mint simply has
  // no TLV region, so extensions decode as None; the family check above has
  // already pinned the owning program, which is what makes the reuse safe.
  const mint = await fetchMint(rpc, address(asset.id));
  const extensions = mint.data.extensions.__option === 'Some' ? mint.data.extensions.value : [];
  const states = readExtensionState(extensions);
  return {
    set: states.map((s) => s.kind),
    notes: states.map((s) => `${s.active ? 'ACTIVE ' : 'DORMANT'} ${s.kind}: ${s.detail}`),
  };
}

function compare(declared: string[], live: string[]): { missing: string[]; extra: string[] } {
  const declaredSet = new Set(declared);
  const liveSet = new Set(live);
  return {
    missing: declared.filter((x) => !liveSet.has(x)),
    extra: live.filter((x) => !declaredSet.has(x)),
  };
}

async function main(): Promise<void> {
  const argAddress = process.argv[2];
  if (!argAddress) {
    throw new Error('usage: npx tsx capstone/verify.ts <ASSET_ADDRESS>');
  }
  const memo = loadMemo();
  if (!memo.assetAddress) {
    throw new Error('memo.json: assetAddress is empty. Ship the asset (step 5) before you verify it.');
  }
  if (memo.assetAddress !== argAddress) {
    throw new Error(
      `memo.json declares ${memo.assetAddress}, you passed ${argAddress}. Verify the thing you wrote down.`,
    );
  }

  const showFungible = memo.primitiveFamily !== 'core-asset' && memo.primitiveFamily !== 'cnft';
  const asset = await das<CapstoneAsset>('getAsset', {
    id: argAddress,
    options: { showFungible },
  });

  const c = classifyAsset(asset);
  console.log(`asset      ${asset.id}`);
  console.log(`name       ${asset.content?.metadata?.name ?? '(unnamed)'}`);
  console.log(`interface  ${asset.interface}  category=${c.category}  das-rpc-required=${c.requiresDasRpc}`);
  console.log(`owner      ${asset.ownership?.owner ?? '(n/a for a fungible mint)'}`);
  console.log(`rail       ${memo.rail}`);

  const familyError = expectedInterfaceFamily(memo)(asset);
  if (familyError) {
    throw new Error(familyError);
  }

  const state = await liveState(memo, asset);
  console.log('\nlive state');
  for (const note of state.notes) console.log(`  ${note}`);

  const { missing, extra } = compare(memo.declaredSet, state.set);
  console.log('\nmemo vs chain');
  console.log(`  declared  ${memo.declaredSet.join(', ') || '(empty)'}`);
  console.log(`  live      ${state.set.join(', ') || '(empty)'}`);
  if (missing.length > 0) console.log(`  MISSING   ${missing.join(', ')}`);
  if (extra.length > 0) console.log(`  EXTRA     ${extra.join(', ')}`);

  if (missing.length > 0 || extra.length > 0) {
    throw new Error(
      'live state does not equal the memo. Either the memo is wrong or the asset is, and only one of those is cheap to fix.',
    );
  }

  console.log(`\nOK  memo matches chain. Tradeoff on record: ${memo.tradeoff}`);
}

main().catch((err: unknown) => {
  console.error(`FAIL  ${err instanceof Error ? err.message : String(err)}`);
  process.exit(1);
});
```

Tres decisiones de ese archivo se ganan su porqué, y el resto es plomería.

La revisión de `EXTRA` no es decoración simétrica. Una extensión faltante quiere decir que tu mint no consiguió lo que pediste. Una de más quiere decir que conseguiste algo que nunca declaraste, normalmente porque una receta puso un campo por defecto, y esa es la dirección más peligrosa: un poder no declarado en un mint es exactamente lo que hace que te rechacen en una plataforma con la que asumías que eras compatible.

La comparación de `token_program` es un seguro barato contra el error más vergonzoso posible, que es entregar un mint de SPL clásico mientras tu memo dice Token-2022. DAS te entrega el programa dueño, así que pregunta, y niégate a seguir cuando el campo vuelve ausente, por la misma razón por la que la rama de Core rechaza un objeto plugins faltante.

Y la rama de Core lanza un error en vez de pasar cuando el endpoint no devuelve ningún objeto `plugins`. Una verificación que no puede ver lo que está verificando tiene que fallar ruidosamente. Un pase silencioso sobre un campo no leído es peor que no verificar nada, porque le vas a creer.

![Un diagrama de flujo vertical de cuatro barreras para el script de verificación, desde la revisión de la dirección pasando por la resolución en DAS y la coincidencia de interfaz hasta la comparación de conjuntos, con ramas por familia en las dos últimas barreras.](assets/v09-flowchart.webp)

**8. Corre la barrera.** Apunta `DAS_RPC_URL` a un endpoint con soporte DAS (`RPC_URL` puede quedarse en devnet para la lectura del mint) y córrelo:

```bash
export DAS_RPC_URL="https://<your-das-endpoint>"
npx tsx capstone/verify.ts <YOUR_ASSET_ADDRESS>
```

Una corrida que pasa imprime la línea del activo, la interfaz y la categoría, el estado en vivo, la comparación de memo contra chain, y una línea `OK` que termina en el tradeoff que escribiste. Código de salida 0. Si sale con 1, lee cuál de las cuatro barreras se disparó, porque cada una nombra un error distinto y solo una de ellas es un bug de código.

## Challenge

Rompe tu propia demostración, a propósito, tres veces. Este es un ejercicio de cinco minutos y es la diferencia entre una verificación en la que confías y una verificación que hiciste.

**Uno.** Agrega un nombre de extensión falso a `declaredSet` en `memo.json` y vuelve a correr. Deberías verlo bajo `MISSING` y salir con 1. Si pasa, tu comparación no está comparando.

**Dos.** Quita una extensión real de `declaredSet` y vuelve a correr. Debería aparecer bajo `EXTRA` y salir con 1. La primera versión de este script que escribe la mayoría revisa una sola dirección, y siempre pasa, y siempre es inútil.

**Tres.** Apunta `DAS_RPC_URL` a un RPC común sin soporte DAS y vuelve a correr. Si tu activo es un cNFT vas a recibir la ruta de método-no-encontrado de JSON-RPC desde tu propio transporte, que es el modo de falla del que te advirtió el módulo 7. Si tu activo es un mint de Token-2022, anota qué pasa y escríbelo: algunos endpoints comunes responden `getAsset` para fungibles y otros no, y saber cuál hace el tuyo es un dato de portabilidad sobre tu stack.

Después restaura el memo y vuelve a una corrida verde. Opcionalmente, y esta es la versión que vale la pena poner en un portafolio: escribe el memo de un segundo brief sin acuñar nada, corre `capstone/venue.ts` contra él, y pon los dos memos lado a lado. Dos elecciones defendibles para dos productos distintos, desde el mismo toolkit, es un artefacto más fuerte que un token publicado.

## Checkpoint

Terminaste cuando cuatro cosas existen juntas.

Un memo, prosa más JSON, que nombre la familia de primitivas, el conjunto de extensiones o de plugins, el tradeoff que aceptaste en un párrafo, y una sección de plataforma cuyos números salieron de `capstone/venue.ts` y no de un post de blog. El activo, publicado solo a partir de recetas enseñadas. Un riel, conectado y probado en un script que imprime un antes y un después. Y un `npx tsx capstone/verify.ts $ASSET_ADDRESS` verde, salida 0, con el conjunto en vivo igual al conjunto declarado, más, si tomaste el brief 4, la demostración de que la transferencia debe fallar al lado, porque el conjunto de tags de cNFT no puede ver la condición soulbound y la barrera imprime ese punto ciego por su cuenta.

Di la respuesta a una sola pregunta en voz alta antes de cerrar la terminal, porque es aquello hacia lo que venía construyendo todo este curso: ¿a qué poder renunciaste, y qué conseguiste a cambio? Si puedes responder eso en una oración sin mirar tus notas, no solo entregaste un token. Tomaste una decisión de arquitectura y dejaste el recibo.

Una nota sobre cómo se siente eso, ya que el repliegue fue total aquí y los repliegues totales son incómodos. La incomodidad es el currículum. Nueve módulos de builds resueltos existen para que esta lección pudiera quitarlos, y si entregaste algo que resuelve vía DAS con un memo que le calza, hiciste lo que el trabajo de verdad pide.

Entregaste un producto de tu propia elección y demostraste que resuelve. Queda una cosa: cerrar el ciclo. La próxima lección vuelves a derivar la decisión en frío, sin notas, contra tres briefs en frío: uno que nunca viste, uno que vuelve a correr la forma de badge masivo a la que esta lección le puso precio pero no construyó por ti, y uno deliberadamente al lado del memo resuelto de la cafetería de arriba, porque la práctica de recuperación necesita un ancla contra la cual calibrar. Después mapeas adónde te lleva este toolkit.
