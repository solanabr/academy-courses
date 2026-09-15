# Entrega el NFT: colecciones, plugins y ediciones de Metaplex Core

## Resumen

En m06-l1 mapeaste las capas de metadatos: el JSON off-chain, el struct Data on-chain de Metaplex, y el TLV nativo de Token-2022 que escribiste sobre SPROUT allá en m02-l4. Validaste el JSON de un activo real contra el estándar y ahora sabes exactamente dónde vive cada campo. Lo que no has hecho es acuñar un NFT. Hoy eso cambia, y nos saltamos el camino legado por completo.

Aquí está el argumento en dos números, ambos sacados de la propia tabla comparativa de Metaplex (metaplex.com/docs/smart-contracts/core, leída el 2026-09-07), así que trátalos como publicados por el proveedor, no como algo que este curso midió. Acuñar un NFT por Token Metadata cuesta alrededor de ~0.022 SOL y ~205,000 unidades de cómputo, porque el mint se desparrama por una cuenta de mint, una cuenta de token, una PDA de metadatos y una PDA de master edition. Acuñar el mismo activo por Metaplex Core cuesta ~0.003 SOL y ~17,000 CU. Una cuenta. Alrededor de 86% más barato, y corre la división sobre esas dos cifras de SOL tú mismo en vez de aceptar el porcentaje por fe. Cita su tilde también: ellos publican aproximaciones, y un curso que endureciera ~0.003 en un 0.0029 de apariencia precisa estaría inventando una precisión que el proveedor nunca afirmó. Y Core es el estándar que Metaplex ahora recomienda para trabajo nuevo, que es la razón por la que esta lección nunca te pide crear ningún NFT de Token Metadata.

Demostración antes que teoría. Con el surfnet levantado, el mismo que corres desde m02-l1 (`surfpool start --no-tui --no-studio` en una terminal aparte), instala el SDK y corre el mint de borrador:

```bash
mkdir -p labs/m06-l2 && cd labs/m06-l2
npm install @metaplex-foundation/mpl-core@1.10.0 @metaplex-foundation/umi@1.5.1 @metaplex-foundation/umi-bundle-defaults@1.5.1
```

Pins verificados contra npm y crates.io el 2026-08-23: el latest del SDK de JS es 1.10.0. Core además se versiona sobre dos números más que es fácil confundir entre sí. El programa on-chain está etiquetado `release/core@0.15.1` (2026-06-18), mientras que el crate cliente de Rust publicado en crates.io es `mpl-core` 0.12.1 (2026-06-16). Tres líneas de release, tres números, y el único que instalas aquí es el SDK de JS en 1.10.0; el tag del programa y el crate de Rust son contexto para leer changelogs, no dependencias de este lab. Vuelve a revisar los tres antes de fijarlos en algo de larga vida.

```typescript
// labs/m06-l2/first-mint.ts
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { mplCore, createCollection, create, fetchCollection } from "@metaplex-foundation/mpl-core";
import { generateSigner, keypairIdentity, sol } from "@metaplex-foundation/umi";

async function main() {
  const umi = createUmi(process.env.RPC_URL ?? "http://127.0.0.1:8899").use(mplCore());
  umi.use(keypairIdentity(umi.eddsa.generateKeypair()));
  await umi.rpc.airdrop(umi.identity.publicKey, sol(2));

  const collectionSigner = generateSigner(umi);
  await createCollection(umi, {
    collection: collectionSigner,
    name: "Scratch Collection",
    uri: "https://overgrowth.example/scratch.json",
  }).sendAndConfirm(umi);

  const collection = await fetchCollection(umi, collectionSigner.publicKey);
  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    collection,
    name: "Scratch Asset",
    uri: "https://overgrowth.example/scratch-asset.json",
  }).sendAndConfirm(umi);

  const raw = await umi.rpc.getAccount(assetSigner.publicKey);
  if (raw.exists) {
    console.log("asset account:", assetSigner.publicKey);
    console.log("bytes:", raw.data.length);
    console.log("rent:", Number(raw.lamports.basisPoints) / 1e9, "SOL");
    console.log("owner:", raw.owner);
  }
}

main().catch((err) => { console.error(err); process.exit(1); });
```

`npx tsx first-mint.ts` imprime una dirección, un largo de datos un poco arriba de 100 bytes (el tuyo cambia con los largos del name y del URI, ya que ambos se guardan inline), un depósito de rent en las milésimas bajas de un SOL, y un owner que arranca con `CoRE`. Ese es todo el NFT. Una colección, un activo acuñado dentro de ella, la membresía ya escrita, una cuenta que sostiene todo. Sin ATA, sin PDA de metadatos, sin PDA de edición.

Esta lección construye R7, `almanac-assets`: los NFT Almanac de Overgrowth como una colección Core verificada, activos con un plugin Royalties, una impresión de edición numerada, y un badge Founding-Farmer que nunca puede salir de su billetera. El repliegue de la ayuda, en voz alta: la creación de la colección y el primer mint de activo están trabajados completos; tú llenas la config del plugin Royalties y la assertion de membresía; y el badge soulbound, su demostración de que la transferencia falla, y el coding challenge del royalties-validator son totalmente en solitario.

## Una cuenta por activo

### Por qué el conteo de cuentas es toda la historia

Piensa de vuelta en tu trabajo con SPROUT. Cada capacidad que le agregaste a ese mint vivía en la misma cuenta, anexada como entradas TLV que tu propio decodificador podía recorrer. Token Metadata es la arquitectura opuesta: capacidad por acumulación de cuentas. El mint es un mint de SPL, el name vive en una PDA de metadatos que pertenece a otro programa, la condición de edición vive en otra PDA, y la imposición programable (el camino pNFT) arrastra todavía más. Cada cuenta cuesta rent, cada una cuesta CU para crearse, y cada una es una cosa más que todo lector aguas abajo tiene que derivar, traer y deserializar.

La respuesta de diseño de Core es la que ya conoces de Token-2022, aplicada a los NFT: una cuenta, con capacidades tipadas anexadas dentro de ella. El activo base guarda el owner, la autoridad de actualización, un name y un URI. Todo lo demás, regalías, comportamiento de congelamiento, numeración de ediciones, atributos, es un plugin serializado después de los datos base en esa misma cuenta. Versión desde primeros principios: el estado de un NFT es chico y sus capacidades son enumerables, así que pagar el overhead de creación de cuentas por capacidad es desperdicio puro; lo único que te compran las cuentas separadas es propiedad independiente, y de todas formas los plugins de un activo pertenecen todos al activo.

![Comparación de un mint de NFT, Token Metadata creando de cuatro a cinco cuentas a ~0.022 SOL y ~205,000 CU publicados por el proveedor mientras Core crea una cuenta a ~0.003 SOL y ~17,000 CU.](assets/v01-comparison.png)

Tu propia corrida de `first-mint.ts` te acaba de mostrar que el componente de rent varía con los largos de los strings, que es exactamente por qué la apertura etiquetó las cifras de titular como publicadas por el proveedor: cita al proveedor, muestra tu propia cuenta. Sigue siendo una gran historia.

### Qué vive dentro de un activo Core

El activo base es deliberadamente diminuto. Un discriminador de un byte (Core usa su propio enum de tipo de cuenta, no el hash de 8 bytes de Anchor), la pubkey del owner, un enum de autoridad de actualización, el name, el URI. El enum de autoridad de actualización es el campo estructural de esta lección: no siempre es una dirección. Tiene tres variantes, `None`, `Address` y `Collection`, y cuando un activo se acuña dentro de una colección la variante es `Collection` con la dirección de la cuenta de la colección adentro.

Lee eso otra vez, porque borra en silencio todo un ritual de Token Metadata. En Token Metadata, la membresía de colección es un campo en la PDA de metadatos más un booleano `verified` aparte que voltea una instrucción firmada por la autoridad de la colección; la membresía no verificada es un estado intermedio real (y peligroso): un activo puede AFIRMAR una colección antes de que alguna autoridad de colección lo haya atestiguado, la misma brecha entre afirmación y atestiguación que conociste en m06-l1 con la flag `verified` de creators, ahora a nivel de colección. En Core no hay booleano. La membresía ES la variante de autoridad de actualización, y solo se puede escribir cuando la autoridad de la colección firma el mint. La verificación no se volvió más fácil; se colapsó en una firma que tiene que estar ahí de todas formas.

![Layout de una sola cuenta de activo Core, los campos base incluyendo el enum updateAuthority de tres variantes, luego un registro de plugins que sostiene entradas Royalties, Edition, PermanentFreezeDelegate y Attributes en la misma cuenta.](assets/v02-diagram.png)

### Las Collections van primero

La consecuencia de ordenamiento se desprende directamente de ese diseño: la colección tiene que existir antes de que cualquier activo pueda acuñarse dentro de ella, porque la instrucción de mint necesita la cuenta de la colección para referenciarla y la firma de la autoridad de la colección para autorizar la escritura de membresía. Acuña activos primero y tienes huérfanos con autoridades de actualización `Address`. Existe un camino de update para mover un activo a una colección después, pero es una readaptación que necesita firmas de autoridad de los dos lados, y todo lo que está aguas abajo de ti, indexadores, marketplaces, el trabajo de compresión del módulo 7, está construido alrededor del flujo colección-primero.

Este ordenamiento no es una preferencia de estilo en este curso; es infraestructura estructural. Bubblegum v2, la apertura del módulo 7, acuña NFT comprimidos DENTRO de una colección Core. Sin colección Core, sin drop de cNFT. La colección Almanac que creas en el lab es el valor literal que toman las llamadas de mint de esa lección, reconstruida sobre el cluster contra el que corra. Adquiere el hábito ahora, mientras el modo de falla es barato.

![Diagrama de flujo que contrasta el flujo colección-primero, donde la membresía se escribe al momento del mint y alimenta a Bubblegum v2 en el módulo 7, con el flujo mint-primero que produce activos huérfanos y lecturas de membresía vacías.](assets/v03-flowchart.png)

### El catálogo de plugins

Los plugins son donde Core deja de ser un mint más barato y se vuelve un modelo de programación distinto. Cada plugin es un struct tipado con su propia autoridad, adjuntado al momento de crear o después, sobre un activo o sobre una colección (un plugin a nivel de colección aplica a cada miembro a menos que el miembro lo sobrescriba). El catálogo al que de verdad vas a echar mano:

| Plugin | Qué hace | El uso en el Almanac |
|---|---|---|
| Royalties | `basisPoints` + participaciones de `creators` + un `ruleSet` | 5% en cada venta de Almanac |
| TransferDelegate | un delegado puede transferir el activo | flujos de escrow y de marketplace |
| FreezeDelegate | un delegado puede congelar/descongelar (reversible) | bloqueos suaves estilo staking |
| BurnDelegate | un delegado puede quemar | consumo de ítems de juego |
| UpdateDelegate | un delegado puede actualizar metadatos | colecciones gestionadas |
| PermanentFreezeDelegate | congelamiento que se puede volver irrevocable | el badge Founding-Farmer soulbound |
| Attributes | pares clave/valor on-chain | traits que un programa puede leer sin traer un URI |
| Edition / MasterEdition | impresiones numeradas + tope de supply | la tirada de impresiones del Almanac |

### Cada plugin lleva su propia clave

Antes de que escribas cualquiera de estos, un hecho estructural que te ahorra una tarde de reversiones confusas: la autoridad de un plugin no es la autoridad del activo. Cada entrada de plugin en el registro lleva su propio campo de autoridad, y ese campo es un enum con cuatro variantes: `Owner`, `UpdateAuthority`, `Address` (cualquier pubkey que nombres) y `None`. Quien ocupe esa variante controla el plugin, independiente de quién es dueño del activo y de quién puede actualizar sus metadatos. Tres claves, tres trabajos. El dueño mueve el activo, la autoridad de actualización lo renombra, la autoridad del plugin opera el plugin.

El catálogo se divide a lo largo de esa línea. Los plugins gestionados por el dueño, los delegados de Transfer, Freeze y Burn, existen para que el DUEÑO preste una capacidad: delegas derechos de transferencia a un escrow, derechos de congelamiento a un programa de staking, y la delegación por defecto queda en tu propia clave hasta que la asignes a otro. Los plugins gestionados por la autoridad, Royalties, Attributes, UpdateDelegate, pertenecen al lado del creador y por defecto quedan en la autoridad de actualización, que para un miembro de colección se resuelve a través de la colección. Y la familia permanente juega con una regla más dura: `PermanentFreezeDelegate`, `PermanentTransferDelegate` y `PermanentBurnDelegate` solo se pueden adjuntar al momento del mint. No puedes colarle un congelamiento permanente a un activo que alguien ya tiene, que es exactamente la propiedad que hace seguro ser dueño de un activo Core, y exactamente por qué el badge Founding-Farmer tiene que nacer soulbound en vez de convertirse después.

![Diagrama que separa las tres claves de control de un activo Core, el owner, la autoridad de actualización, y el enum de autoridad de cada plugin con sus cuatro variantes, con las clases de plugin agrupadas como gestionados por el dueño, gestionados por la autoridad, y plugins permanentes solo-al-momento-del-mint.](assets/v04-diagram.png)

Quédate con la variante `None`. La mayor parte del tiempo asignas una autoridad de plugin para que alguien pueda actuar. Ponerla en `None` es la jugada inversa, y muerde: suelda el estado actual del plugin en su lugar, permanentemente, porque no existe ninguna clave que pudiera cambiarlo alguna vez. Un `PermanentFreezeDelegate` con `frozen: true` y autoridad `None` no es "congelado hasta que alguien importante diga lo contrario". Está congelado del modo en que un número es par.

Tres entradas del catálogo merecen una mirada más de cerca antes de que las escribas.

**Royalties** lleva tres campos y el programa impone su forma al momento del mint. `basisPoints` es un entero 0..10000 (500 significa 5%). `creators` es una lista de entradas de dirección más porcentaje cuyos porcentajes tienen que sumar exactamente 100, y una dirección de creador duplicada se rechaza. `ruleSet` decide quién puede mover el activo: `None` no pone restricciones de programa sobre las transferencias, `ProgramAllowList` permite que solo los programas listados estén involucrados, `ProgramDenyList` bloquea los programas listados. Fíjate en lo que `None` quiere decir para la palabra "regalía": el reparto queda registrado on-chain, legible por todos, impuesto por nadie en particular. Si alguien la paga de verdad es una decisión del marketplace, y esa oración incómoda es el tema entero de m06-l3. Ya me equivoqué al teclear un reparto de creadores, 60/50 entre dos billeteras porque edité un lado y no el otro, y el mint se revierte en el momento. Bien. Mejor una reversión al momento del mint que un marketplace repartiendo 110%.

![Config anotada del plugin Royalties que muestra basisPoints acotado de 0 a 10000, porcentajes de creador que tienen que sumar exactamente 100 sin direcciones duplicadas, y las tres variantes de ruleSet.](assets/v05-annotated-code.png)

**PermanentFreezeDelegate** es el hermano sin vuelta del FreezeDelegate reversible. Adjúntalo con `frozen: true` y una autoridad de `None` y tienes un activo que ninguna clave en la tierra puede descongelar ni mover. Eso no es un bug que haya que esquivar; es el mecanismo soulbound. Un badge de membresía, una credencial, una prueba de asistencia: las cosas que deberían no tener sentido a la venta son exactamente las cosas que congelas permanentemente. La contracara es la trampa de la que te avisa el nombre. Permanente significa permanente. No hay voto de gobernanza posterior, no hay ticket de soporte, no hay autoridad que pueda descongelar el badge Founding-Farmer una vez que lo acuñas así. Si un granjero pierde su billetera, necesita un badge nuevo, no una transferencia. Echa mano del FreezeDelegate reversible cada vez que puedas imaginar un movimiento futuro legítimo.

**Edition y MasterEdition** dividen un trabajo entre los dos tipos de cuenta. La colección lleva `MasterEdition` con un `maxSupply` y overrides opcionales de name/URI; cada activo impreso lleva `Edition` con su `number`. Las lecturas se componen exactamente como esperarías: trae la colección por el tope, trae cualquier impresión por su número.

![Diagrama de una tirada de impresiones donde la colección sostiene un plugin MasterEdition con maxSupply 100 y cada activo miembro lleva un plugin Edition con su propio número de impresión.](assets/v06-diagram.png)

### Delegados, y qué borra una transferencia

Los tres delegados gestionados por el dueño son a lo que echas mano cuando un activo tiene que participar en algo sin dejar de ser de su dueño. Adjunta `TransferDelegate` con la dirección de un programa de escrow y ese programa puede mover un Almanac fuera de una billetera cuando se liquida una venta, sin tenerlo nunca en custodia primero. Adjunta `FreezeDelegate` con `frozen: true` y un programa de staking como autoridad y el activo se bloquea en su lugar: sigue en la billetera del dueño, sigue visible en cada UI, simplemente inmovible hasta que el programa lo descongele. Así funciona el staking de Core, y es por eso que el staking de Core no necesita cuenta de vault. El activo nunca se va a ninguna parte; simplemente deja de poder hacerlo. `BurnDelegate` es el caso de crafteo. Un granjero mete dos volúmenes del Almanac en el composter de Overgrowth y el programa quema los dos bajo una delegación otorgada antes, sin prompt de firma al momento de quemar.

La API es `addPlugin` para adjuntar uno, y `approvePluginAuthority` para entregar la clave a la dirección de un programa después. El snippet de abajo es ilustrativo, no ejecutable tal como está pegado: las dos direcciones son PLACEHOLDERS que tienes que reemplazar (la primera es literalmente la dirección del System Program, que es en lo que decodifica un string base58 de todos unos; la segunda es una dirección válida arbitraria que hace de sustituto de tu programa de escrow), y el approve solo funciona sobre un plugin que existe, así que una llamada `addPlugin(umi, { asset, plugin: { type: "TransferDelegate" } })` lo precede en cualquier flujo real:

```typescript
// labs/m06-l2/delegate-transfer.ts (illustrative: replace BOTH placeholder
// addresses, and addPlugin the TransferDelegate first or this approve fails)
import { publicKey } from "@metaplex-foundation/umi";
import { approvePluginAuthority } from "@metaplex-foundation/mpl-core";
import { getUmi } from "./umi";

async function main() {
  const umi = await getUmi();
  await approvePluginAuthority(umi, {
    asset: publicKey("11111111111111111111111111111111"),      // PLACEHOLDER: your Almanac asset
    plugin: { type: "TransferDelegate" },
    newAuthority: {
      type: "Address",
      address: publicKey("8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR"),  // PLACEHOLDER: the escrow program
    },
  }).sendAndConfirm(umi);
}

main().catch((err) => { console.error(err); process.exit(1); });
```

Ahora la regla que decide tu arquitectura sin hacer ruido. Cuando un activo se transfiere, los plugins gestionados por el dueño tienen su autoridad revocada automáticamente de vuelta a `Owner`. Los plugins gestionados por la autoridad y la familia permanente sobreviven la transferencia intactos.

Esa sola oración tiene tres consecuencias que vale sostener por separado. Un comprador nunca hereda los derechos de escrow del vendedor ni los derechos de congelamiento que ese mismo vendedor cedió a un programa de staking, que es la propiedad que hace que comprar un activo Core cargado de plugins sea siquiera seguro: lo que delegó el dueño anterior se evapora en el momento en que el activo cambia de manos. Tu plugin Royalties, por ser gestionado por la autoridad, viaja con él para siempre, que es precisamente lo que una regalía tiene que hacer para significar algo a través de una reventa. Y la familia permanente también viaja con él, que es la razón más profunda por la que solo se puede adjuntar al momento del mint. Las propiedades irreversibles de un activo tienen que ser conocibles para un comprador antes de que compre, y la adjunción solo-al-momento-del-mint es lo que garantiza eso: nadie puede dejarte el activo soldado después de que ya es tuyo.

La versión desde primeros principios, si quieres que la regla sea memorable en vez de memorizada: una delegación es una declaración sobre la intención del dueño actual, así que no tiene que sobrevivir a ese dueño. Una regalía es una declaración sobre los términos del creador, así que sí. Core codifica la diferencia en la clase del plugin en vez de pedirle a cada integrador que recuerde cuál es cuál.

![Tabla de las clases de plugin en Core, los plugins gestionados por el dueño cuya autoridad se auto-revoca en la transferencia, los plugins gestionados por la autoridad como Royalties que persisten, y la familia permanente que se adjunta solo al momento del mint.](assets/v07-table.png)

### Attributes: traits que un programa on-chain puede leer de verdad

En m06-l1 aprendiste el array `attributes` del JSON off-chain, los traits que los marketplaces renderizan en una barra lateral. Un programa de Solana no puede leer ese array. Vive detrás de un URI HTTP, invisible para el runtime, así que cualquier lógica on-chain que quiera saber la temporada de un Almanac tiene que recibirlo de un firmante confiable. Eso es un oráculo, y ahora estás corriendo uno.

El plugin `Attributes` es la respuesta on-chain. Guarda una `attributeList` de pares string clave/valor dentro de la cuenta del activo misma, donde un programa lo deserializa de una cuenta que ya tiene cargada:

```typescript
// labs/m06-l2/tag-season.ts
import { publicKey } from "@metaplex-foundation/umi";
import { addPlugin, fetchAsset } from "@metaplex-foundation/mpl-core";
import { getUmi } from "./umi";

async function main() {
  const umi = await getUmi();
  const asset = publicKey("11111111111111111111111111111111");  // PLACEHOLDER: your Almanac asset address

  await addPlugin(umi, {
    asset,
    plugin: {
      type: "Attributes",
      attributeList: [
        { key: "season", value: "spring-2026" },
        { key: "yield", value: "3" },
      ],
    },
  }).sendAndConfirm(umi);

  const fetched = await fetchAsset(umi, asset);
  console.log(fetched.attributes?.attributeList);
}

main().catch((err) => { console.error(err); process.exit(1); });
```

Es gestionado por la autoridad, así que para un miembro de colección la autoridad de la colección firma las actualizaciones, no el tenedor. Ese es el valor por defecto correcto para el estado de juego: un granjero no debería poder reescribir el yield de su propio Almanac entre harvests. Dos restricciones para diseñar alrededor. Las claves y los valores son ambos strings, así que los números se convierten en string al entrar y se parsean al salir, y nada valida el parseo más que tú. Y cada atributo son bytes en una cuenta por la que pagas rent, así que este es un lugar para el puñado de traits sobre los que ramifican tus programas, no una base de datos. La lista completa de traits de cara al marketplace se queda en el JSON donde los marketplaces ya la buscan.

### Asset o colección: dónde debe vivir un plugin

Cada tipo de plugin se adjunta a cualquiera de los dos tipos de cuenta, y la regla de resolución es la parte donde está el dinero: un plugin en la colección aplica a cada miembro, y un plugin en el miembro tiene precedencia sobre la versión de ese mismo plugin en la colección. El SDK ya viene con `deriveAssetPlugins(asset, collection)` para que computes el conjunto efectivo en una sola llamada en vez de revisar las dos cuentas y reimplementar la precedencia tú mismo.

La economía se desprende sola. Un drop de 10,000 piezas que adjunta Royalties a cada activo paga por los bytes de ese plugin diez mil veces. Adjúntalo a la colección una vez y cada miembro lo hereda, y la única pieza con un reparto de creadores distinto lleva su propio plugin Royalties como override local. La misma lógica aplica a Attributes cuando el trait es de toda la colección en vez de por pieza.

El lab de hoy adjunta Royalties por activo de todas formas, y la razón es pedagógica más que arquitectónica: deberías escribir esa config a mano una vez, ver al programa rechazar un reparto malo, y después construir el validador que lo atrapa antes de que una transacción salga de tu máquina. Cuando el drop del capstone llegue a escala real, súbelo a la colección y deja que la herencia haga el trabajo.

### Mint a escala: Core Candy Machine

Todo en el lab acuña a mano porque estás acuñando cuatro activos. Un drop de 10,000 piezas quiere una máquina expendedora: precarga las configs, deja que los compradores acuñen ellos mismos, defiende el mint con reglas. Esa máquina es Core Candy Machine, programa `CMACYFENjoBMHzapRXyo1JZkVS6EtaDDzkjMrmQLvr4J`, y sus reglas son módulos de guarda: `solPayment`, `startDate`, `mintLimit`, `allowList` (con acceso restringido por prueba de Merkle), `botTax` (las verificaciones de guarda que fallan pagan un impuesto en vez de revertirse gratis), y más. ¿Cuántas hay en total? La página de docs anuncia "23+ guardas componibles" y nunca las enumera. El fuente sí: contar las declaraciones `mod` en el programa candy-guard, cruzado contra los campos de su struct `GuardSet`, da exactamente 31 (revisado 2026-08-23). Fíjate que los docs no están mal aquí; un piso rara vez lo está. Son vagos, y no puedes diseñar contra un piso. Cita el conteo del fuente, y vuelve a contarlo tú mismo el día en que el número tenga que cargar peso.

Las guardas se componen en grupos con nombre, que es cómo una sola máquina corre todo un calendario de drop. Un grupo es un conjunto de guardas etiquetado (las etiquetas topan en seis caracteres), los compradores pasan la etiqueta cuando acuñan, y cualquier guarda por defecto que pongas fuera de los grupos se hereda a menos que un grupo la sobrescriba. Así que un grupo `wl` lleva la raíz de Merkle de `allowList` y el `solPayment` con descuento, un grupo `public` lleva el precio completo y ninguna restricción de acceso, y `botTax` vive en los valores por defecto donde protege a los dos. Una vez que existen los grupos, acuñar solo con los valores por defecto no está permitido: siempre se requiere una etiqueta. Esa es la forma que el módulo 8 llena con números reales.

Dos cosas para llevarte y una para nunca hacer. Para llevarte: el par anti-snipe que acabas de conocer, `botTax` y `allowList`, es la columna vertebral de un mint justo, y el mismo problema de defender-el-lanzamiento vuelve en el módulo 8 alrededor de las bonding curves. Y Core Candy Machine acuña activos Core ÚNICAMENTE. El nunca: la línea legada Candy Machine V3 acuña NFT de Token Metadata y está deprecada junto con el estándar al que sirve; si un tutorial te pone V3 delante, estás leyendo historia.

![Pipeline de una transacción de comprador que pasa las guardas startDate, allowList, mintLimit y solPayment hacia un mint que coloca el activo en una colección Core, con las verificaciones que fallan enrutadas a botTax.](assets/v08-flowchart.png)

### El trade-off, nombrado

El modelo de una sola cuenta de Core es la razón por la que el mint es alrededor de 87% más barato y la razón por la que las regalías, el comportamiento soulbound y las ediciones son plugins tipados en vez de esparcimiento de PDA. A qué renuncias: superficie de madurez. Token Metadata tiene media década de integraciones, una ventaja corriendo desde el estándar de 2021 que viste en la línea de tiempo de m06-l1; cada billetera, marketplace y script de backend polvoriento lo entiende, mientras que el soporte de Core es amplio en 2026 pero más joven, y todavía vas a encontrar herramientas que leen TM y se encogen de hombros ante Core. Segundo, los dientes de un plugin Royalties son exactamente su `ruleSet`: entrega `None` y tu regalía on-chain es solo indicativa, una realidad que m06-l3 disecciona sin anestesia. Tercero, te comprometes con el ordenamiento: los activos se acuñan DENTRO de una colección verificada, y readaptar es una tarea de dos autoridades que deberías tratar como una falla de planificación, no como un flujo de trabajo.

Puedes leer el traspaso en los trenes de release solos, sin anuncio necesario. El paquete de JS `mpl-token-metadata` se detuvo en v3.4.0 en febrero de 2025 y no ha entregado una función desde entonces. Mientras tanto el programa Core cortó de 0.13.0 a 0.15.1 a lo largo de mayo y junio de 2026, su crate cliente de Rust llegó a 0.12.1 el 2026-06-16, y el SDK de JS de Core llegó a 1.10.0 en abril de 2026. Una línea se quedó callada; las otras tres mantuvieron una cadencia estable. Así se ve una migración de estándar desde el lado del changelog.

![Línea de tiempo que muestra la línea de JS de mpl-token-metadata deteniéndose en v3.4.0 en febrero de 2025 mientras Metaplex Core entregó JS 1.10.0 y las versiones de programa 0.13.0 a 0.15.1 a lo largo de 2026.](assets/v09-timeline.png)

## Lab: acuña el Almanac

El orden de construcción espeja la teoría: colección, luego miembro, luego impresión, luego demostración. Todo corre contra el surfnet, y cada script comparte una sola billetera para que las cuentas persistan entre corridas.

1. **El setup compartido.** Un helper es dueño de la persistencia y el fondeo de la billetera, así que las re-corridas no dejan huérfanas tus cuentas:

    ```typescript
    // labs/m06-l2/umi.ts
    import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
    import { mplCore } from "@metaplex-foundation/mpl-core";
    import { keypairIdentity, sol } from "@metaplex-foundation/umi";
    import fs from "node:fs";

    export async function getUmi() {
      const umi = createUmi(process.env.RPC_URL ?? "http://127.0.0.1:8899").use(mplCore());

      let secret: Uint8Array;
      if (fs.existsSync("wallet.json")) {
        secret = Uint8Array.from(JSON.parse(fs.readFileSync("wallet.json", "utf8")));
      } else {
        const fresh = umi.eddsa.generateKeypair();
        fs.writeFileSync("wallet.json", JSON.stringify(Array.from(fresh.secretKey)));
        secret = fresh.secretKey;
      }
      const keypair = umi.eddsa.createKeypairFromSecretKey(secret);
      umi.use(keypairIdentity(keypair));

      const balance = await umi.rpc.getBalance(keypair.publicKey);
      if (balance.basisPoints < sol(1).basisPoints) {
        await umi.rpc.airdrop(keypair.publicKey, sol(2));
      }
      return umi;
    }
    ```

    Corre todo desde `labs/m06-l2/` para que `wallet.json` y el directorio de direcciones que comparten los scripts (`almanac.json`) terminen en un solo lugar.

2. **Crea la colección Almanac (trabajado completo).** La colección lleva el plugin MasterEdition desde su nacimiento para que la tirada de impresiones del paso 5 tenga un tope que leer; una bandera de honestidad ahora, cobrada en el paso 5, es que el tope es dato registrado que el ecosistema lee, no algo que el programa imponga al momento del mint, así que llevarlo desde el nacimiento es la forma colección-primero más que una dependencia mecánica.

    ```typescript
    // labs/m06-l2/create-collection.ts
    import { generateSigner } from "@metaplex-foundation/umi";
    import { createCollection, fetchCollection } from "@metaplex-foundation/mpl-core";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const collectionSigner = generateSigner(umi);

      await createCollection(umi, {
        collection: collectionSigner,
        name: "Overgrowth Almanac",
        uri: "https://overgrowth.example/almanac/collection.json",
        plugins: [
          {
            type: "MasterEdition",
            maxSupply: 100,
            name: undefined,  // optional edition-line name; unset here, and step 5
            uri: undefined,   // passes each print its own name and uri anyway
          },
        ],
      }).sendAndConfirm(umi);

      const collection = await fetchCollection(umi, collectionSigner.publicKey);
      console.log("collection:", collection.publicKey);
      console.log("update authority:", collection.updateAuthority);
      console.log("master edition maxSupply:", collection.masterEdition?.maxSupply);

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ collection: collectionSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    `npx tsx create-collection.ts` imprime la dirección de la colección, tu billetera como su autoridad de actualización, y `maxSupply: 100`. Esa línea de autoridad importa: es la firma que autorizará cada escritura de membresía de aquí en adelante.

3. **Acuña el Almanac Vol. 1 (tú llenas dos huecos).** La llamada de mint se te entrega con el plugin Royalties y la assertion de membresía dejados abiertos. Llena los dos antes de espiar el paso 4. La forma de Royalties está en la sección de catálogo: 500 basis points, tu identidad como el único creador en 100, `ruleSet` de `None`. La assertion de membresía debería demostrar, solo desde el activo traído, que este activo es un miembro verificado de la colección que acabas de crear.

    ```typescript
    // labs/m06-l2/mint-almanac.ts
    import { generateSigner, publicKey } from "@metaplex-foundation/umi";
    import { create, fetchAsset, fetchCollection, ruleSet } from "@metaplex-foundation/mpl-core";
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      const collection = await fetchCollection(umi, publicKey(saved.collection));

      const assetSigner = generateSigner(umi);
      await create(umi, {
        asset: assetSigner,
        collection,
        name: "Almanac: Vol. 1",
        uri: "https://overgrowth.example/almanac/vol-1.json",
        plugins: [
          // TODO(you): the Royalties plugin.
          // 500 basis points, one creator (umi.identity.publicKey) at percentage 100,
          // ruleSet("None"). The exact shape is in the plugin catalog.
        ],
      }).sendAndConfirm(umi);

      const asset = await fetchAsset(umi, assetSigner.publicKey);
      console.log("asset:", asset.publicKey);
      console.log("owner:", asset.owner);

      // TODO(you): the membership assertion.
      // Prove asset.updateAuthority is the Collection arm, and that its address
      // is exactly saved.collection. Two asserts, no RPC calls beyond the fetch.

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ ...saved, asset: assetSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Mientras estás ahí dentro, prueba sabotear tu propia config de regalías una vez: pon el porcentaje del creador en 99 y córrelo. El programa rechaza el mint. Esa reversión es el comportamiento exacto que tu validador del coding challenge reproduce off-chain.

4. **La revelación.** El llenado de Royalties:

    ```typescript
    plugins: [
      {
        type: "Royalties",
        basisPoints: 500,
        creators: [{ address: umi.identity.publicKey, percentage: 100 }],
        ruleSet: ruleSet("None"),
      },
    ],
    ```

    Y la assertion de membresía:

    ```typescript
    assert.equal(asset.updateAuthority.type, "Collection");
    assert.equal(asset.updateAuthority.address, publicKey(saved.collection));
    ```

    Dos líneas, cero fetches extra. Las claves públicas de Umi son strings simples en runtime, así que la igualdad estricta sobre la dirección simplemente funciona. Si tu versión hizo assert contra una colección traída en vez de eso, no está mal, solo es más caro; el punto del diseño de Core es que la demostración de la membresía vive en los bytes del propio activo. `npx tsx mint-almanac.ts` debería imprimir ahora la dirección del activo y salir limpio a través de los dos asserts.

5. **Imprime la edición numerada (trabajado).** La misma llamada `create`, plugin distinto. El plugin Edition lleva el número de impresión; el MasterEdition de la colección lleva el tope que pusiste en el paso 2:

    ```typescript
    // labs/m06-l2/mint-print.ts
    import { generateSigner, publicKey } from "@metaplex-foundation/umi";
    import { create, fetchAsset, fetchCollection } from "@metaplex-foundation/mpl-core";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      const collection = await fetchCollection(umi, publicKey(saved.collection));

      const printSigner = generateSigner(umi);
      await create(umi, {
        asset: printSigner,
        collection,
        name: "Almanac 2026, print #1",
        uri: "https://overgrowth.example/almanac/print-1.json",
        plugins: [{ type: "Edition", number: 1 }],
      }).sendAndConfirm(umi);

      const print = await fetchAsset(umi, printSigner.publicKey);
      console.log("print:", print.publicKey);
      console.log("edition number:", print.edition?.number);

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ ...saved, print: printSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Una advertencia honesta antes de que escales esto, y cubre el tope igual que los números: cuando acuñas por SDK, la contabilidad te toca administrarla a ti. Nada impide que un script descuidado acuñe dos print #1, y nada on-chain impide el print #101 tampoco; el `maxSupply` del MasterEdition es dato registrado que el ecosistema lee, no un tope que el programa Core imponga al momento del mint. La integridad secuencial Y el tope de supply son promesas del lado del cliente aquí. A escala de drop, Core Candy Machine asigna los números e impone el tope por ti, que es una razón más para que exista. El brief del capstone para el drop de músicos construye directamente sobre este paso, así que asegúrate de que `edition number: 1` se imprima antes de seguir.

6. **El espejo de CLI (opcional, mostrado una vez).** Todo lo que scriptaste tiene un gemelo de línea de comandos en el CLI de Metaplex, útil para picar cuentas rápido sin abrir un editor:

    ```bash
    npm install -g @metaplex-foundation/cli
    mplx core asset fetch <your asset address> --rpc http://127.0.0.1:8899
    ```

    `mplx core asset create` y `mplx core collection create` también existen, junto con `mplx core plugins add`. El binario es `mplx`, el CLI va en su propia línea 0.x (0.4.3 al momento de escribir, así que vuelve a revisar el árbol de comandos con `mplx core --help` antes de scriptear contra él), y el orden de tópicos es sustantivo y después verbo: `core asset fetch`, no `core fetch asset`. El curso scriptea todo en TS porque los scripts se componen en barreras de verificación y las sesiones de CLI no, pero saber que el espejo existe te ahorra tiempo en lecturas de una sola vez.

7. **Corre la barrera.** El script de verificación de abajo es el contrato de R7: la demostración de que existen cuatro activos que llevan exactamente las propiedades que el resto de este curso supone que puedes construir. Se da completo porque las propiedades son el entregable, no el script. Eso sí, ten claro qué viaja de verdad. Las lecciones posteriores reconstruyen la colección Almanac sobre el cluster contra el que corran en vez de leer tu `almanac.json`, porque una cuenta de surfnet no significa nada en devnet. Lo que se lleva hacia adelante es la receta y el hábito colección-primero, no el archivo.

    ```typescript
    // labs/m06-l2/verify-almanac.ts
    import { publicKey } from "@metaplex-foundation/umi";
    import { fetchAsset, fetchCollection } from "@metaplex-foundation/mpl-core";
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      for (const key of ["collection", "asset", "print", "badge"]) {
        assert.ok(saved[key], `almanac.json is missing "${key}" - run the mint scripts first`);
      }

      // 1. The collection exists and carries its MasterEdition cap.
      const collection = await fetchCollection(umi, publicKey(saved.collection));
      assert.ok(collection.masterEdition, "collection has no MasterEdition plugin");
      console.log(`OK: collection ${collection.name} (${collection.publicKey})`);
      console.log(`OK: master edition maxSupply=${collection.masterEdition.maxSupply}`);

      // 2. The Almanac asset is a verified member and its Royalties plugin reads back.
      const asset = await fetchAsset(umi, publicKey(saved.asset));
      assert.equal(asset.updateAuthority.type, "Collection", "asset is not collection-owned");
      assert.equal(asset.updateAuthority.address, publicKey(saved.collection));
      assert.ok(asset.royalties, "asset has no Royalties plugin");
      const shares = asset.royalties.creators.reduce((sum, c) => sum + c.percentage, 0);
      assert.equal(shares, 100, "creator shares must sum to 100");
      console.log(`OK: ${asset.name} is a verified member of the Almanac collection`);
      console.log(`OK: royalties ${asset.royalties.basisPoints} bps, shares sum to ${shares}`);

      // 3. The print reads back its edition number.
      const print = await fetchAsset(umi, publicKey(saved.print));
      assert.equal(print.updateAuthority.type, "Collection");
      assert.ok(print.edition, "print has no Edition plugin");
      console.log(`OK: ${print.name} reads back edition number ${print.edition.number}`);

      // 4. The badge is permanently frozen: soulbound.
      const badge = await fetchAsset(umi, publicKey(saved.badge));
      assert.ok(badge.permanentFreezeDelegate, "badge has no PermanentFreezeDelegate plugin");
      assert.equal(badge.permanentFreezeDelegate.frozen, true, "badge is not frozen");
      console.log(`OK: ${badge.name} is non-transferable (PermanentFreeze)`);
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Corre `npx tsx verify-almanac.ts` ahora y se detiene en el primer assert: `almanac.json is missing "badge"`. Ese es el comportamiento correcto. La barrera te está diciendo qué falta, y lo que falta es el challenge. Fíjate en lo que el script nunca hace: nunca toca DAS, nunca le pregunta nada a un indexador. Cada demostración es una lectura directa de cuenta de bytes que acuñaste. Leer estos mismos activos a través de la interfaz DAS, a escala de colección, a través de un proveedor, es el trabajo de m07-l2, y se va a sentir lujoso después de esto.

## Challenge

Solo, sin apoyo. Acuña el badge Founding-Farmer: un activo Core en la colección Almanac que lleve `PermanentFreezeDelegate` con `frozen: true` y una autoridad de `None`, para que nada pueda descongelarlo nunca. Escribe su dirección en `almanac.json` bajo la clave `badge`. Después demuestra el congelamiento con tus propias assertions, en un script tuyo: un intento de `transfer` sobre el badge tiene que fallar (atrapa el rechazo y haz assert de que lo atrapaste), y un `transfer` de tu activo Almanac Vol. 1 a una billetera desechable tiene que funcionar, en la misma corrida, para que la demostración muestre que el congelamiento es propiedad del badge y no alguna mala configuración global. Piensa antes de acuñar: este es el único acto irreversible de la lección. Un typo en el name del badge es, quizá sorprendentemente, recuperable: el camino de update no es lo que PermanentFreeze bloquea, así que la autoridad de actualización todavía puede renombrarlo. Un badge acuñado a la billetera equivocada simplemente se fue: congelado donde quedó, intransferible, indescongelable, para siempre.

Después toma el coding challenge royalties-plugin-config: implementa `validateRoyalties(basisPoints, ruleSet, creatorSpec)`, el validador puro de pre-vuelo para la config exacta que escribiste a mano en el paso 3. El calificador pasa la config como tres argumentos posicionales, el número de basisPoints, el nombre del ruleSet, y el reparto de creadores achatado en un solo string de entradas `address=percentage` separadas por punto y coma (así que un reparto 70/30 llega como `'Farm1...=70;Farm2...=30'`); el starter ya parsea ese string de vuelta a objetos Creator por ti. El starter revisa solo la suma de las participaciones; tú agregas el resto, que es el rango de basisPoints, la variante de ruleSet, el rango de porcentaje propio de cada creador, y las direcciones de creador duplicadas. Viste la reversión on-chain en el paso 3; ahora construye la guarda que lo atrapa antes de que una transacción salga de tu máquina.

Aceptado cuando: `npx tsx verify-almanac.ts` pasa de punta a punta, las cuatro secciones en verde; tu demostración de la transferencia muestra el badge fallando y el Vol. 1 moviéndose en la misma corrida; y las pruebas del challenge pasan todos los casos, incluidos los que quizá no pensaste (un basisPoints de 20000 sentado encima de un reparto perfectamente válido, una dirección de creador duplicada cuyas participaciones igual suman 100, un ruleSet escrito `ProgramList`).

## Checkpoint

La barrera: `npx tsx verify-almanac.ts` imprime sus seis líneas OK. Collection con su tope de MasterEdition, el Vol. 1 un miembro verificado con las regalías leyéndose de vuelta en 500 bps, el print #1 llevando su número de edición, el badge congelado permanentemente. Con eso en verde, R7 está completo, y la respuesta de una sola oración que deberías poder dar sin mirar: la colección tiene que crearse antes de que se acuñe cualquier activo, porque la membresía se escribe dentro del activo al momento del mint bajo la firma de la autoridad de la colección, y atornillarla después es una readaptación de dos autoridades que deberías tratar como una falla de planificación, no como un flujo de trabajo.

Los fallos que espero. Primero, el ordenamiento: si tu assertion de membresía falla con `updateAuthority.type === "Address"`, acuñaste sin pasar la colección, y ninguna cantidad de re-fetching lo arregla; acuña otra vez, colección-primero. Segundo, la reversión de las regalías: un reparto que no suma 100 o un basisPoints fuera de 0..10000 falla al momento del mint con un error de Core, que es la spec de tu validador escrita como un stack trace. Tercero, si la transferencia del badge TIENE ÉXITO en la demostración de tu challenge, revisa cuál activo congelaste; más de un estudiante ha congelado permanentemente su Vol. 1 y dejado el badge líquido, y en un surfnet desechable eso es una lección gratis sobre exactamente por qué PermanentFreeze merece respeto en mainnet.

![Diagrama de hub del artefacto R7 completado, la colección Almanac con el activo con regalía, la impresión numerada y el badge congelado, consumido por Bubblegum v2, la lección de DAS, el módulo 8 y el capstone.](assets/v10-diagram.png)

Acuñaste una colección, tres tipos de miembro, y demostraste cada propiedad con lecturas directas. Costo total en tu surfnet: centavos, y el mismo flujo en mainnet se queda en las milésimas de un SOL por activo según los números del propio proveedor. Pero mira otra vez lo que realmente entregaste en ese plugin Royalties. Lo adjuntaste, pusiste 500 basis points, verificaste que se lee de vuelta. ¿Alguien la impone de verdad? Entregaste `ruleSet("None")`, y te dejé. La siguiente lección es la realidad de las regalías que nadie anuncia: qué imposición existe de verdad, qué hacen realmente los pNFT y Token Auth Rules, y por qué el estándar contra el que la mitad del ecosistema todavía integra es oficialmente legado. Trae estómago fuerte para la palabra "indicativo".
