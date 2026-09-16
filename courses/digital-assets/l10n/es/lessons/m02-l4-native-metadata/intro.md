# Metadatos nativos: MetadataPointer + TokenMetadata TLV

## Resumen

Acabas de construir la capa de protección: un mint de insignia soulbound cuyo par forzado NonTransferableAccount + ImmutableOwner confirmaste en cada cuenta de tenedor, y una tesorería con memo obligatorio que rechaza los depósitos sin etiquetar. Las mecánicas están listas, y están repartidas entre los artefactos del módulo a propósito: el mint de economía de m02-l1 cobra comisiones y les hace harvest, los mints desechables de m02-l2 comprueban quién puede congelar y recuperar por la fuerza, la insignia y la tesorería de m02-l3 comprueban qué puede rechazar una cuenta. Y el SPROUT que una billetera renderiza de verdad sigue siendo una cadena anónima de base58. Un token sin nombre es inutilizable, no en el sentido criptográfico sino en el único sentido que le importa a un usuario que mira fijo `6NDNZ...vBSY` y se pregunta si le acaban de hacer un rug.

Primero, mira el patrón en producción. Con el surfnet de m02-l1 corriendo (`surfpool start --no-tui --no-studio` en una terminal aparte), guarda esto en `labs/m02-l4/probe-pyusd.ts` y corre `npx tsx labs/m02-l4/probe-pyusd.ts`:

```typescript
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "http://127.0.0.1:8899");
const PYUSD = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
const mint = await fetchMint(rpc, PYUSD);
if (mint.data.extensions.__option === "Some") {
  for (const e of mint.data.extensions.value) {
    if (e.__kind === "MetadataPointer" && e.metadataAddress.__option === "Some")
      console.log("pointer ->", e.metadataAddress.value);
    if (e.__kind === "TokenMetadata")
      console.log("metadata:", e.name, "/", e.symbol);
  }
}
```

El puntero imprime `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`. El puntero de metadatos de PYUSD apunta al propio mint de PYUSD. Quédate con ese dato; toda la lección es una explicación de por qué.

Esta lección arregla eso a la manera nativa de Token-2022: el nombre, el símbolo y la URI quedan guardados EN EL MINT MISMO, dentro de la misma región TLV que tu inspector viene recorriendo desde el módulo 1, con un puntero que protege contra una cuenta de metadatos suplantada. Es el último ladrillo de R3, y como esta es la primera aparición del peldaño por su nombre: R3 es el mint de producción de SPROUT en sí, el tercer artefacto de la escalera del curso después de R1 (decode-mint) y R2 (check-combo). Después de hoy R3 queda completo en la forma que consumen los módulos posteriores: un mint que compone las extensiones que legalmente pueden compartirlo (config de comisión, extensión de display, puntero autorreferencial, TokenMetadata), con los comportamientos de autoridad y de protección del módulo comprobados en sus propios mints acompañantes, ya que un mint soulbound o desechable no puede ser además la moneda negociable. Los módulos 5 y 9 consumen este mint compuesto directo; el módulo 3 no puede, porque TransferHook es de create-time, así que acuña una variante con hook fresca al lado.

El repliegue de la ayuda, dicho en voz alta: el cableado del puntero y del TLV de TokenMetadata está trabajado completo, cada instrucción explicada. La escritura del campo `additional_metadata` es un problema de completar, esa instrucción la construyes tú antes de que yo te la muestre. Y el challenge es totalmente en solitario: un mint fresco, una afirmación de ida y vuelta, sin apoyo.

## Metadatos que viven en el mint

### De dónde viene de verdad el nombre de un token

Piensa en lo que hace una billetera cuando renderiza tu saldo. Tiene una dirección de mint y nada más. En algún lado tiene que resolver esa dirección a "SPROUT, 6 decimals, este logo." Durante toda la era del SPL clásico, la respuesta vivía fuera del programa de token: una cuenta de metadatos aparte, propiedad de un programa aparte (el Token Metadata de Metaplex), en una dirección derivada del mint. El programa de token no sabía nada de nombres. Dos programas, dos cuentas, una identidad, pegadas por convención.

![Una billetera resuelve un mint de Token-2022 en una sola lectura de cuenta, recorriendo el TLV hasta el puntero autorreferencial y la entrada de metadatos, a diferencia del camino heredado de dos cuentas de Metaplex.](assets/v01-flowchart.webp)

Token-2022 colapsa eso. Dos de las 29 extensiones de producción del enum ExtensionType existen exactamente para este trabajo:

- **MetadataPointer** (tipo de extensión 18): un campo chico del lado del mint que responde una sola pregunta, "¿dónde viven los metadatos canónicos de este mint?" Guarda una autoridad opcional (quién puede reapuntar el puntero) y una dirección de metadatos opcional.
- **TokenMetadata** (tipo de extensión 19): los metadatos en sí, una entrada TLV de longitud variable que lleva el nombre, el símbolo y la URI de verdad, definida por `spl_token_metadata_interface`.

El diseño son dos piezas en vez de una a propósito. El puntero es la indirección: los metadatos PODRÍAN vivir en alguna otra cuenta, mantenida por algún otro programa que implemente la interfaz de metadatos. Pero el patrón que enseña este curso, el patrón que entrega PYUSD, es el caso degenerado: apunta el mint a sí mismo y guarda el TLV en línea. Una cuenta, un programa, una lectura.

![El modelo de Metaplex usa una PDA de metadatos aparte, propiedad de otro programa, mientras que el modelo nativo de Token-2022 guarda el puntero y los metadatos dentro del mint mismo.](assets/v02-diagram.webp)

¿Por qué existe el puntero siquiera, si la respuesta es "apunta a ti mismo"? Porque la interfaz es más grande que el caso en línea. `spl_token_metadata_interface` es una especificación que cualquier programa puede implementar, y un mint creado antes de que existieran las extensiones de metadatos todavía puede apuntar su puntero a una cuenta de metadatos externa. El puntero es la respuesta publicada, on-chain, a "qué cuenta es la canónica." Lo cual nos lleva al ataque contra el que fue diseñado.

### El argumento anti-suplantación, desde primeros principios

Supón que no hubiera puntero, solo una convención: "los metadatos del mint M viven en alguna cuenta que afirma `mint = M`." Cualquiera puede crear una cuenta. Yo puedo crear mañana una cuenta que diga `mint = <PYUSD's address>`, `name = "PayPal USD"`, `symbol = "PYUSD"`, y una URI que apunte al JSON que se me antoje. Nada en la blockchain distingue mi falsificación de la real, porque la afirmación vive en la cuenta falsificable, no en la cosa sobre la que se afirma. Un indexador que escanea buscando cuentas de metadatos encuentra dos candidatas para PYUSD y no tiene ninguna regla on-chain para elegir. Esa es la superficie de suplantación, y no es hipotética: por eso las campañas de estafa de NFT pudieron colgar metadatos con pinta oficial en mints basura durante años.

El puntero invierte la dirección de la confianza. El mint dice qué cuenta habla por él, y solo el conjunto de autoridades del mint mismo pudo haber escrito ese campo. Una tercera cuenta puede seguir afirmando lo que quiera; ningún lector va a seguir nunca un puntero hacia ella. Y el struct de TokenMetadata cierra el lazo desde el otro lado con su campo `mint`: los metadatos nombran su mint, el mint nombra sus metadatos. Cuando los dos viven en la misma cuenta, como en SPROUT y en PYUSD, el lazo mide una cuenta y no queda nada que falsificar. Para suplantar los metadatos necesitarías acceso de escritura al mint mismo, y en ese punto el token es tuyo y no estás suplantando nada.

Esta es también exactamente la trampa que hay que evitar cuando lo conectas: apunta el MetadataPointer a alguna cuenta arbitraria que casualmente controlas y ya reintrodujiste la indirección en la que vive el ataque. A menos que estés implementando a propósito un programa de metadatos externo (no lo estás, y casi nadie lo está), autorreferencial es el único valor que deberías escribir alguna vez.

![Sin un puntero cualquier cuenta puede afirmar ser los metadatos de un mint, mientras que un puntero autorreferencial significa que los lectores siguen solo la referencia de salida del mint, dejando las falsificaciones fuera de alcance.](assets/v03-diagram.webp)

### Qué hay de verdad en el TLV, y qué no

La entrada TokenMetadata la define `spl_token_metadata_interface`, y vale la pena memorizar su forma porque vas a leerla de vuelta durante el resto del curso:

```rust
// spl_token_metadata_interface state (the shape, as stored in the TLV)
pub struct TokenMetadata {
    pub update_authority: OptionalNonZeroPubkey, // who may edit fields later
    pub mint: Pubkey,                            // the mint this speaks for (anti-spoof, other direction)
    pub name: String,                            // ON-CHAIN
    pub symbol: String,                          // ON-CHAIN
    pub uri: String,                             // link to off-chain JSON
    pub additional_metadata: Vec<(String, String)>, // arbitrary on-chain key-value pairs
}
```

Dos ideas equivocadas que hay que matar mientras tienes el struct enfrente. Primero: el nombre y el símbolo son cadenas on-chain, guardadas en los bytes del mint, legibles con `getAccountInfo` y nada más. La URI apunta a JSON off-chain para los campos pesados (imagen, descripción, lo que tu producto necesite), pero los campos de identidad no viven en ese JSON. Una billetera puede renderizar "SPROUT" sin un solo fetch HTTP. Segundo: esto no es el struct `Data` on-chain de Metaplex. Programa distinto, modelo de cuentas distinto, layout de campos distinto, y el código escrito para uno no va a deserializar el otro. La comparación completa, incluido cuándo el stack de Metaplex sigue siendo la elección correcta, es la lección de apertura del módulo 6; por ahora basta con mantener los dos mentalmente separados.

Eso deja el otro extremo de la URI sin cubrir, y Token-2022 no tiene nada que decir al respecto. El programa guarda una cadena y nunca la busca. No hay esquema on-chain, ni validador, ni chequeo de contenido, ni imposición de ninguna clase. Lo que existe en cambio es una convención: la forma del JSON off-chain que popularizó Metaplex (`name`, `symbol`, `description`, `image`, y un array `attributes`), que las billeteras aprendieron a parsear años antes de que existieran los metadatos nativos y que los tokens de metadatos nativos heredaron por defecto. Es lo que una billetera prueba primero cuando busca tu URI. De ahí se siguen dos consecuencias prácticas. Tu `name` on-chain y el `name` del JSON pueden no coincidir, y nada en la blockchain los va a detener, así que mantenlos sincronizados a propósito. Y cualquier host que sirva esa URI es ahora una dependencia de la apariencia de tu token, lo cual es un argumento concreto para mantener viva la autoridad de actualización en vez de quemarla el primer día. El módulo 6 abre con ese estándar de JSON en serio, incluido qué campos leen de verdad los marketplaces.

![El TLV del mint lleva los campos de identidad que sí se hacen cumplir, mientras que la URI apunta a un JSON off-chain convencional que nada en la blockchain valida ni mantiene sincronizado.](assets/v04-diagram.webp)

`additional_metadata` es la parte extensible: pares arbitrarios de cadenas clave-valor, on-chain, editables por la autoridad de actualización. Overgrowth lo va a usar en el lab para un campo `harvest_season`, y es el mecanismo detrás de cada esquema de "trait sobre un token fungible" que te vas a encontrar en la vida real.

Ese campo `update_authority` es una decisión de producto escondida en un struct, así que tómala de forma consciente. Es un `OptionalNonZeroPubkey`: consérvala y puedes editar cada campo después con las instrucciones de update de la interfaz; ponla en none y los metadatos quedan inmutables, para siempre, sin vuelta atrás. Un emisor que quizá necesite rotar un host de URI comprometido o arreglar un typo mantiene la autoridad viva, detrás de la misma disciplina de ops que la autoridad de mint que configuraste en m02-l2. Una memecoin que demuestra que nunca puede renombrarse a sí misma en silencio quema la autoridad y lo dice. Ninguna de las dos está mal; entregar sin haber elegido, sí. SPROUT conserva su autoridad por ahora, porque la pasada de endurecimiento para producción del módulo 9 revisa todas las autoridades del mint de una vez, y esta pertenece a esa barrida.

### Los bytes, como ya puedes leerlos

Construiste un recorredor de TLV en m01-l2, así que nada del almacenamiento debería quedarse abstracto. Cada entrada es un tipo little-endian de 2 bytes, una longitud de 2 bytes, después el valor. El valor del puntero está fijo en 64 bytes: pubkey de autoridad, después dirección de metadatos, 32 cada una, en cero cuando no están seteadas. El valor de TokenMetadata es una serialización Borsh del struct de arriba: dos pubkeys crudas de 32 bytes (autoridad de actualización, mint), después cada cadena como un prefijo de longitud de 4 bytes más los bytes UTF-8, y después el vector de pares como un conteo de 4 bytes con las cadenas prefijadas adentro. Longitud variable, exactamente como lo exige la historia del realloc.

Corre la aritmética una vez para SPROUT y el tamaño de la cuenta deja de ser magia: 64 + (4 + 6) para `name = "SPROUT"`, (4 + 4) para `SPRT`, (4 + 38) para la URI, (4 + 18 + 10) para un par `harvest_season = "spring"`. Eso son 156 bytes de valor, 160 con su header TLV. Quédate con ese 160, porque la sección siguiente le pone precio a la cuenta entera con él: un mint SPROUT de solo puntero queda en 234 bytes, y 234 + 160 = 394 es el tamaño que el lab financia. Derivado acá, afirmado allá.

![Layout a nivel de bytes de la entrada TLV de TokenMetadata, dos pubkeys de 32 bytes más cadenas Borsh con prefijo de longitud que totalizan 160 bytes, lo que lleva el mint de 234 bytes a los 394 bytes que financia el lab.](assets/v05-annotated-code.webp)

Una instrucción más redondea la interfaz, y existe para el caso con el que SPROUT nunca se topa: `Emit`. Un lector que quiere metadatos sin saber a dónde lleva el puntero puede pedirle al programa dueño de los metadatos que serialice el struct hacia los datos de retorno y leerlo desde una simulación. Para un mint autorreferencial es redundante, `fetchMint` lee el TLV directo de la cuenta con un solo `getAccountInfo`, sin ningún indexador a la vista. Pero cuando el puntero apunta a un programa de metadatos externo, `Emit` es la ruta de lectura uniforme que mantiene cada implementación de la interfaz legible por el mismo código de cliente.

Un par más de extensiones pertenece a este mapa mental, porque espejan este diseño exactamente: GroupPointer/TokenGroup (tipos 20 y 21) y GroupMemberPointer/TokenGroupMember (tipos 22 y 23) hacen por las colecciones lo que el par de metadatos hace por la identidad, un puntero autorreferencial más estado TLV en línea. Son la forma en que Token-2022 expresa "este mint pertenece a ese grupo" de manera nativa. Hoy no las estamos conectando, y este curso nunca lo hace; importan para los activos con forma de NFT, y cuando el módulo 6 retome las colecciones vas a ver el mismo problema de pertenece-a-ese-grupo resuelto del lado de Metaplex en cambio.

### La costura del create-time, y el baile del realloc

Acá está la parte con la que la gente de verdad se tropieza, y es una regla que ya conoces de haber construido SPROUT tres veces: las extensiones de mint se inicializan ANTES de `InitializeMint`. Una vez que un mint de Token-2022 está inicializado, su conjunto de extensiones queda fijo. No puedes atornillarle un MetadataPointer al mint de ayer. Por eso esta lección, igual que las tres anteriores, vuelve a crear SPROUT en vez de actualizarlo; el script del lab que crea mints es tu script de m02-l3 más una entrada en la lista de extensiones.

Pero TokenMetadata rompe el patrón, y la asimetría es la decisión de diseño interesante de esta lección. El puntero es de tamaño fijo y de create-time. Los metadatos son de longitud variable: tu nombre hoy, una URI más larga mañana, cinco pares `additional_metadata` más la temporada que viene. Dimensionarlos en la asignación los congelaría. Así que la interfaz hace de los metadatos una instrucción post-init: después de `InitializeMint`, llamas al initialize de la interfaz, el programa sigue el propio puntero del mint de vuelta al mint, reasigna la cuenta para que quepa la nueva entrada TLV, y escribe los campos.

La reasignación cuesta rent, y el programa no lo va a pagar por ti. El baile, concretamente, con los números del build del lab:

- Un mint clásico pelado tiene 82 bytes. Con cualquier extensión presente, la base se rellena hasta 165 bytes más un byte de tipo de cuenta, y después siguen las entradas TLV.
- SPROUT-con-solo-puntero se asigna en **234 bytes**: eso es lo que le pasas a `createAccount` como `space`.
- La entrada de metadatos de SPROUT (nombre `SPROUT`, símbolo `SPRT`, una URI de 38 caracteres, un par clave-valor) va a hacer realloc de la cuenta hasta **394 bytes**: ese es el tamaño que tienes que FINANCIAR.

Así que asignas espacio para 234 y depositas lamports para 394. La librería cliente hace esto sin dolor: `getMintSize` acepta una copia fantasma de la extensión de metadatos puramente para la aritmética, y los lamports resultantes se quedan quietos en el mint hasta que el realloc los reclama. Financíalo de menos y la instrucción de metadatos falla con un error de fondos insuficientes para rent; nada se corrompe, pero tu transacción de crear-y-nombrar muere en la instrucción cuatro de cinco.

Y el baile no termina en la creación, que es la parte que la gente descubre en producción. De acá a seis meses cambias la URI por una cadena más larga, o agregas un segundo par `additional_metadata`, y esa escritura le vuelve a hacer realloc al mint. La cuenta tiene que quedar exenta de renta en su NUEVO tamaño, y la instrucción de update no va a sacar la diferencia de la nada. Así que una actualización de metadatos son en realidad dos operaciones: la llamada a la interfaz, y una transferencia de lamports al mint que cubra el crecimiento. Encoger va en el otro sentido y simplemente deja el mint con fondos de sobra, ya que nadie te devuelve la holgura. Presupuesta esto como presupuestarías una migración de esquema, porque debajo del vocabulario eso es exactamente lo que es.

![Diagrama de flujo de cinco instrucciones, asignar 234 bytes financiados para 394, inicializar el puntero autorreferencial, inicializar el mint, y después la instrucción de metadatos post-init hace realloc y escribe los campos.](assets/v06-flowchart.webp)

¿Por qué tolerar esta complejidad en vez de hacer de los metadatos una extensión de create-time también? Trade-off, nombrado sin rodeos. Los metadatos nativos mantienen la identidad en el mint: ninguna cuenta extra que crear, ningún programa externo en el que confiar, ninguna derivación de PDA que las billeteras tengan que conocer, y la superficie de suplantación cerrada por construcción. Los costos vienen en tres sabores.

El rent, primero, y vamos a ponerle precio en vez de mencionarlo de pasada. Pregúntale al mismo método RPC que llama el script del lab, en el cluster en el que corre el lab. En mi surfnet (2026-09-06) la exención de renta para el mint clásico de 82 bytes es de 1,461,600 lamports, el SPROUT de solo puntero de 234 bytes es 2,519,520, y el SPROUT completamente nombrado de 394 bytes es 3,633,120: 210, 362 y 522 bytes totales a 6,960 lamports cada uno. Mainnet cobra 6,333 desde el 2026-09-03 (el primer paso de SIMD-0437) y devuelve 1,329,930 / 2,292,546 / 3,305,826 para esos mismos tres tamaños, así que un fork no te está cotizando automáticamente el rent de mainnet aunque te esté sirviendo las cuentas de mainnet. De cualquier manera lo que importa es la forma: la identidad on-chain entera de tu token cuesta unos 0.001 SOL por encima del mint de solo puntero, unas cuantas decenas de centavos de dólar a los precios recientes de SOL. Para un solo mint fungible esto es nada, que es exactamente por lo que el patrón le queda bien a los tokens fungibles: un mint, millones de tenedores, los metadatos pagados una sola vez. Cambia la forma a una colección de NFT, un mint POR ítem, y el rent de metadatos por ítem empieza a importar, una de varias razones por las que el módulo 6 resuelve este trade-off del otro modo.

Segundo, la edad. El patrón nativo es años más joven que el de Metaplex, así que las herramientas de billeteras y marketplaces construidas alrededor de "deriva la PDA de Metaplex" necesitan en cambio el camino de seguir-el-puntero, y las herramientas de cola larga todavía a veces no lo tienen. Los grandes lo leen bien, PYUSD no renderizaría si no, pero si la vida de tu token depende de algún rastreador de portafolio de nicho, pruébalo antes del lanzamiento en vez de suponer.

Y tercero, el que ahora entiendes mecánicamente: datos de longitud variable en un mint quieren decir el baile del realloc, dimensionar rent para bytes que todavía no has escrito. Para un token fungible en 2026, los metadatos nativos suelen ser la decisión correcta de todos modos. Para colecciones de NFT ricas, el camino de Metaplex Core del módulo 6 lo es, y cuando lleguemos ahí vas a ver el mismo trade-off resuelto en la dirección opuesta.

### PYUSD corre exactamente este patrón

El sondeo que corriste arriba no era un juguete. PayPal y Paxos lanzaron PYUSD en Solana en mayo de 2024 como el despliegue emblemático de Token-2022: una stablecoin regulada y atestiguada por KPMG (el estudio stablecoin-landscape de Helius puso su supply circulante en Solana en $215.9M repartidos en 20.4k cuentas de tenedor el 2025-05-29; las cifras de hoy están a una lectura en vivo de distancia, y el módulo 9 hace esa lectura). Su mint lleva ocho extensiones TLV. Léelas de la salida de tu propio sondeo, llegan en este orden: MintCloseAuthority, PermanentDelegate, TransferFeeConfig, ConfidentialTransferMint, ConfidentialTransferFee, TransferHook, MetadataPointer, TokenMetadata.

Ya configuraste personalmente cinco de esas ocho sobre variantes de SPROUT, y la disciplina de m01 se aplica a la lista entera: la presencia no te dice nada, los valores sí. En la lectura del 2026-08-22, el transfer hook de PYUSD estaba configurado con un programa nulo y su config de comisión quedaba en 0 basis points con un máximo de 0, en los dos esquemas, el más viejo y el más nuevo. Interruptores dormidos, instalados para un futuro que su equipo de cumplimiento puede activar. Pero las dos que estás conectando hoy están configuradas Y vivas: el puntero resuelve al mint mismo, y el TLV se lee de vuelta como `PayPal USD / PYUSD` con una URI hacia `token-metadata.paxos.com`. Cuando una billetera muestra el logo de PayPal al lado de un saldo, esta entrada TLV, leída directo del mint, es donde arranca ese render. El patrón nativo no es la opción experimental. Es lo que entrega un emisor regulado de primer nivel. (Si quieres el otro lado de este vidrio, el curso Solana Payments & Commerce lee el mint de PYUSD en vivo como ejercicio de integración, comprobando qué tiene que manejar un comercio antes de aceptarlo. Acá tú eres el emisor, escribiendo los bytes que ese curso lee.)

![Comparación de las ocho extensiones TLV de PYUSD, seis configuradas pero dormidas frente a las dos extensiones de metadatos vivas que llevan el nombre PayPal USD y el puntero autorreferencial.](assets/v07-comparison.webp)

Una pieza más de contexto, breve, ya que te encontraste dos veces con el archivo del 2025-01-24 del currículum oficial: su material de metadatos es anterior al patrón nativo maduro y todavía enseña el mundo de cuenta-aparte como el valor por defecto. Estás aprendiendo este desde la interfaz y desde los bytes porque ese es por ahora el único lugar donde vive completo.

## Lab: dale su nombre a SPROUT

El build: vuelve a crear SPROUT con el puntero en su conjunto de extensiones, escribe el TLV, y después lee todo de vuelta y afírmalo. Al final, R3 queda completo y `verify-metadata.ts` existe en la interfaz exacta que llaman los módulos posteriores.

1. **Workspace y pins.** En tu workspace del curso (el mismo que en m02-l3), crea la carpeta de la lección y confirma el trío de dependencias; si estás arrancando en una máquina nueva, instálalas:

    ```bash
    mkdir -p labs/m02-l4
    npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
    ```

    Los mismos pins, por la misma razón, que el párrafo de pins de m02-l1 argumentó completo: el workspace fija el major de kit contra el que hacen peer sus clientes `@solana-program/*`, lo que hoy quiere decir kit 7.1.1 con cada cliente en su minor actual de kit-^7 (verificado contra npm el 2026-09-05; vuelve a verificar cuando leas esto).

2. **Surfnet arriba.** El lab corre contra el surfnet local que usas desde m02-l1 (`surfpool start --no-tui --no-studio`; forkea mainnet de forma perezosa, que es por lo que funcionó el sondeo de PYUSD, y honra los airdrops, que es por lo que el script siguiente se financia solo). Devnet sirve como plan B: `RPC_URL=https://api.devnet.solana.com WS_URL=wss://api.devnet.solana.com npx tsx ...`, con el faucet reemplazando la llamada de airdrop si te aplica límite de tasa.

3. **El cableado trabajado.** Crea `labs/m02-l4/add-metadata.ts`. Este es el cableado completo de puntero y TLV, trabajado; lee los comentarios contra la sección de teoría, especialmente el baile de dos tamaños del medio:

    ```typescript
    import {
      airdropFactory,
      appendTransactionMessageInstructions,
      assertIsTransactionWithBlockhashLifetime,
      createSolanaRpc,
      createSolanaRpcSubscriptions,
      createTransactionMessage,
      generateKeyPairSigner,
      getSignatureFromTransaction,
      lamports,
      pipe,
      sendAndConfirmTransactionFactory,
      setTransactionMessageFeePayerSigner,
      setTransactionMessageLifetimeUsingBlockhash,
      signTransactionMessageWithSigners,
      some,
    } from "@solana/kit";
    import { getCreateAccountInstruction } from "@solana-program/system";
    import {
      TOKEN_2022_PROGRAM_ADDRESS,
      extension,
      getInitializeMetadataPointerInstruction,
      getInitializeMintInstruction,
      getInitializeTokenMetadataInstruction,
      getMintSize,
    } from "@solana-program/token-2022";
    import { writeFileSync } from "node:fs";

    const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";
    const WS_URL = process.env.WS_URL ?? "ws://127.0.0.1:8900";

    const NAME = "SPROUT";
    const SYMBOL = "SPRT";
    const URI = "https://overgrowth.example/sprout.json";

    async function main() {
      const rpc = createSolanaRpc(RPC_URL);
      const rpcSubscriptions = createSolanaRpcSubscriptions(WS_URL);
      const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

      // Payer doubles as mint authority and metadata update authority for the lab.
      const authority = await generateKeyPairSigner();
      const mint = await generateKeyPairSigner();

      await airdropFactory({ rpc, rpcSubscriptions })({
        commitment: "confirmed",
        recipientAddress: authority.address,
        lamports: lamports(2_000_000_000n),
      });

      // The pointer is a CREATE-TIME extension: it exists before InitializeMint,
      // so it belongs in the space calculation. Self-referential on purpose.
      const metadataPointer = extension("MetadataPointer", {
        authority: some(authority.address),
        metadataAddress: some(mint.address),
      });

      // The TokenMetadata TLV is POST-init: the program reallocs the mint to fit
      // it, so it never enters the allocated space. Its RENT does. This phantom
      // copy exists only to size the deposit.
      const tokenMetadata = extension("TokenMetadata", {
        updateAuthority: some(authority.address),
        mint: mint.address,
        name: NAME,
        symbol: SYMBOL,
        uri: URI,
        additionalMetadata: new Map([["harvest_season", "spring"]]),
      });

      const allocatedSpace = getMintSize([metadataPointer]);           // 234
      const fundedSpace = getMintSize([metadataPointer, tokenMetadata]); // 394
      const rent = await rpc
        .getMinimumBalanceForRentExemption(BigInt(fundedSpace))
        .send();

      const instructions = [
        // Allocate WITHOUT the metadata bytes, fund FOR them.
        getCreateAccountInstruction({
          payer: authority,
          newAccount: mint,
          space: allocatedSpace,
          lamports: rent,
          programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        }),
        // Pointer before InitializeMint. Aim it at the mint itself.
        getInitializeMetadataPointerInstruction({
          mint: mint.address,
          authority: some(authority.address),
          metadataAddress: some(mint.address),
        }),
        getInitializeMintInstruction({
          mint: mint.address,
          decimals: 6,
          mintAuthority: authority.address,
          freezeAuthority: some(authority.address),
        }),
        // TLV after InitializeMint: the program follows the pointer back to the
        // mint, reallocs 234 -> 394, and writes the fields.
        getInitializeTokenMetadataInstruction({
          metadata: mint.address,
          updateAuthority: authority.address,
          mint: mint.address,
          mintAuthority: authority,
          name: NAME,
          symbol: SYMBOL,
          uri: URI,
        }),
        // COMPLETION TODO: one more instruction goes here in step 5.
      ];

      const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
      const transaction = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(authority, tx),
        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) => appendTransactionMessageInstructions(instructions, tx),
        (tx) => signTransactionMessageWithSigners(tx),
      );
      assertIsTransactionWithBlockhashLifetime(transaction);
      await sendAndConfirm(transaction, { commitment: "confirmed" });

      writeFileSync(
        new URL("./sprout-mint.json", import.meta.url),
        JSON.stringify({ mint: mint.address, name: NAME, symbol: SYMBOL, uri: URI }, null, 2),
      );
      console.log(`SPROUT mint with native metadata: ${mint.address}`);
      console.log(`tx: ${getSignatureFromTransaction(transaction)}`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Una nota de honestidad antes de que lo corras: este script carga solo con la preocupación de metadatos, para que el ejemplo trabajado siga siendo legible, y el mint que crea es un build de referencia, no R3. El mint compuesto, comisiones más display más un nombre, es el R3 de verdad al que se refieren los módulos posteriores cuando dicen "el mint de SPROUT," y el paso 7 vuelve obligatoria esa composición y le pone el criterio. (La insignia y la tesorería de m02-l3 quedan separadas a propósito: un mint soulbound no puede ser la moneda negociable.) (Yo igual me guardo una versión de solo metadatos. Cuando una lectura de vuelta se porta mal meses después, la reproducción mínima es la herramienta de debugging que vas a desear haber tenido.)

4. **Córrelo.**

    ```bash
    npx tsx labs/m02-l4/add-metadata.ts
    ```

    Forma esperada de la salida, tus direcciones van a ser distintas:

    ```
    SPROUT mint with native metadata: 6NDNZ8kGXJbwg7JHyz8advCmivmoUEcEuRAVAUoWvBSY
    tx: wCgwQMAAc6azAp7jq9nbgMTpEYrAyPkxMgSi9X8vcayw8iBVp5t9ZRURn6FwT1EgE8oWpZN42YVtdh99SytaM3g
    ```

    El script también deja `labs/m02-l4/sprout-mint.json`, el traspaso de la dirección que leen el script de verify y los módulos posteriores. Trata ese archivo como provisional por ahora: el paso 7 lo reapunta al mint compuesto, y es la dirección compuesta la que los módulos posteriores tienen que encontrar ahí.

5. **Problema de completar: la escritura del campo.** SPROUT necesita su campo `harvest_season`, y esa instrucción te toca construirla a ti. Lo que sabes: el builder es `getUpdateTokenMetadataFieldInstruction` del mismo paquete, su input toma `metadata` (la dirección del mint), `updateAuthority` (un firmante, el nuestro), un `field`, y un `value`. Las claves custom se expresan con el helper `tokenMetadataField`. Agrega el import, construye la instrucción, ponla después del initialize de metadatos en el array, y vuelve a correr. Escríbela antes de seguir leyendo.

    ¿Listo? Acá está el chequeo:

    ```typescript
    import { getUpdateTokenMetadataFieldInstruction, tokenMetadataField } from "@solana-program/token-2022";

    // ... appended after getInitializeTokenMetadataInstruction(...) in `instructions`:
    getUpdateTokenMetadataFieldInstruction({
      metadata: mint.address,
      updateAuthority: authority,
      field: tokenMetadataField("Key", ["harvest_season"]),
      value: "spring",
    }),
    ```

    Fíjate en que la misma instrucción con `tokenMetadataField("Name")` edita el nombre mismo; inicializar-y-después-actualizar es toda la API de escritura, cuatro instrucciones en total en la interfaz (initialize, update field, remove key, update authority). Cada escritura puede hacer crecer el TLV, y por eso el dimensionamiento fantasma del paso 3 ya incluía este par: el rent estaba en depósito antes de que el campo existiera.

6. **Léelo de vuelta.** El criterio de R3 es una lectura de vuelta que afirma, no un log de consola que revisas a ojo. Crea `labs/m02-l4/verify-metadata.ts`; este archivo es infraestructura del curso, los módulos posteriores lo corren por exactamente esta ruta, así que tómalo completo:

    ```typescript
    import { address, createSolanaRpc } from "@solana/kit";
    import { fetchMint } from "@solana-program/token-2022";
    import { readFileSync } from "node:fs";
    import assert from "node:assert/strict";

    const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";

    async function main() {
      const saved = JSON.parse(
        readFileSync(new URL("./sprout-mint.json", import.meta.url), "utf8"),
      ) as { mint: string; name: string; symbol: string; uri: string };
      const mintAddress = address(process.argv[2] ?? saved.mint);

      const rpc = createSolanaRpc(RPC_URL);
      const mint = await fetchMint(rpc, mintAddress);

      assert(mint.data.extensions.__option === "Some", "mint carries no TLV extensions");
      const extensions = mint.data.extensions.value;

      const pointer = extensions.find((e) => e.__kind === "MetadataPointer");
      assert(pointer, "MetadataPointer extension missing");
      assert(
        pointer.metadataAddress.__option === "Some" &&
          pointer.metadataAddress.value === mintAddress,
        "MetadataPointer is not self-referential",
      );

      const metadata = extensions.find((e) => e.__kind === "TokenMetadata");
      assert(metadata, "TokenMetadata TLV missing");
      assert.equal(metadata.mint, mintAddress, "TLV mint field does not match this mint");
      assert.equal(metadata.name, saved.name);
      assert.equal(metadata.symbol, saved.symbol);
      assert.equal(metadata.uri, saved.uri);

      console.log(`OK: MetadataPointer -> ${pointer.metadataAddress.value} (the mint itself)`);
      console.log(
        `OK: TokenMetadata TLV reads back name=${metadata.name} symbol=${metadata.symbol} uri=${metadata.uri}`,
      );
      for (const [key, value] of metadata.additionalMetadata) {
        console.log(`OK: additional_metadata ${key}=${value}`);
      }
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    ```bash
    npx tsx labs/m02-l4/verify-metadata.ts
    ```

    ```
    OK: MetadataPointer -> 6NDNZ8kGXJbwg7JHyz8advCmivmoUEcEuRAVAUoWvBSY (the mint itself)
    OK: TokenMetadata TLV reads back name=SPROUT symbol=SPRT uri=https://overgrowth.example/sprout.json
    OK: additional_metadata harvest_season=spring
    ```

    Las dos afirmaciones de los dos lados del lazo anti-suplantación acaban de correr: el puntero resuelve al mint, y el propio campo `mint` del TLV apunta de vuelta. Fíjate en que el script toma un argumento de dirección opcional; `npx tsx labs/m02-l4/verify-metadata.ts <any mint>` es ahora un verificador de metadatos nativos de propósito general. Apúntalo a PYUSD.

7. **Componlo dentro del SPROUT de verdad, y reapunta el traspaso.** El mint de solo metadatos comprobó el cableado; R3 es el mint compuesto, así que haz la composición ahora. Abre `labs/m02-l1/verify-economics.ts`, el script que construye el SPROUT de comisión-y-display, lleva para allá las constantes `NAME`/`SYMBOL`/`URI`, y haz cuatro cambios, cada uno de ellos sacado de `add-metadata.ts`:

    - Construye la misma extensión `metadataPointer` (autorreferencial, exactamente como en el paso 3, con `payer.address` como su autoridad) y la extensión `tokenMetadata` fantasma, y después dimensiona la cuenta dos veces: asigna en `getMintSize([transferFeeExtension, interestExtension, metadataPointer])` y financia en `getMintSize([transferFeeExtension, interestExtension, metadataPointer, tokenMetadata])`, pasando el tamaño financiado a `getMinimumBalanceForRentExemption` y el tamaño asignado como `space`.
    - Encaja `getInitializeMetadataPointerInstruction` junto con los otros inicializadores de extensiones, ANTES de `getInitializeMintInstruction`; la regla del create-time no ha cambiado.
    - Agrega `getInitializeTokenMetadataInstruction` y tu escritura de campo del paso 5 DESPUÉS de `getInitializeMintInstruction`.
    - Al final de `main()`, escribe el traspaso apuntando a la carpeta de esta lección, para que los consumidores posteriores lean la dirección compuesta:

    ```typescript
    writeFileSync(
      new URL("../m02-l4/sprout-mint.json", import.meta.url),
      JSON.stringify({ mint: mint.address, name: NAME, symbol: SYMBOL, uri: URI }, null, 2),
    );
    ```

    Vuelve a correr `npx tsx labs/m02-l1/verify-economics.ts` (todas sus afirmaciones de comisión tienen que seguir en verde: la composición cambia la identidad del mint, no su economía), y después corre el criterio contra el mint compuesto: `npx tsx labs/m02-l4/verify-metadata.ts`. Las tres líneas OK ahora tienen que sostenerse contra la dirección compuesta. Hasta que lo hagan, R3 no está completo.

8. **Cierra el lazo con R1.** Apunta tu inspector `decode-mint` de m01-l2 al mint compuesto. Aparecen dos filas nuevas en su recorrido de extensiones: tipo 18 (MetadataPointer, 64 bytes de valor TLV) y tipo 19 (TokenMetadata, longitud variable). Las cadenas que acabas de escribir están ahí adentro, en bytes que tu propio decodificador ha podido recorrer desde el módulo 1; `fetchMint` es una comodidad por encima de exactamente ese recorrido, nada más. Y corre `check-combo` sobre el conjunto completo por el bien del ritual: MetadataPointer no conflictúa con nada en la matriz.

![Diagrama de flujo tipo hub que muestra las capas de economía y de metadatos del mint SPROUT terminado, con demostraciones en mints acompañantes y consumidores más abajo en los módulos de hook, de enrutabilidad y de enrutamiento de comisiones.](assets/v08-flowchart.webp)

## Challenge

Solo, sin apoyo: el ejercicio del metadata-pointer. Crea un mint desechable fresco, los decimals que quieras, cuyo conjunto de extensiones sea exactamente un MetadataPointer autorreferencial, escribe un TLV de TokenMetadata con un nombre, un símbolo y una URI de tu elección más al menos un par `additional_metadata`, y después escribe tú mismo la afirmación de ida y vuelta: haz el fetch del mint y afirma que el puntero resuelve al mint, y que cada campo se lee de vuelta exactamente como se escribió, carácter por carácter. Sin copiar `verify-metadata.ts`; el punto es que el assert viva en tus dedos, porque una ida y vuelta que puedes escribir desde cero es la prueba a la que de verdad vas a recurrir cuando un mint de mainnet se porte mal.

Aceptado cuando: un script, una corrida, la afirmación del puntero y cada afirmación de campo pasan, y matar cualquiera de los campos en la escritura hace que falle la afirmación correspondiente (demuéstralo una vez rompiendo el símbolo a propósito).

## Checkpoint

El criterio: `npx tsx labs/m02-l4/verify-metadata.ts` imprime sus tres líneas OK contra el mint COMPUESTO que ahora nombra `sprout-mint.json`, el build del paso 7 que lleva comisiones, una extensión de display, un puntero autorreferencial y TokenMetadata en una sola cuenta, y la ida y vuelta del mint fresco del challenge pasa con tus propias afirmaciones. Con eso en verde, SPROUT (R3) queda completo para el resto del curso; una corrida que pasa contra el mint de solo metadatos del paso 3 por sí sola no cuenta.

Los dos errores que espero, para que puedas autodiagnosticarte rápido. Primero, el orden: pon el initialize del puntero después de `InitializeMint`, o intenta agregarle metadatos a un mint creado sin el puntero, y el programa te rechaza; las extensiones de MINT son de create-time, el TLV de metadatos es la excepción post-init que estás usando (los TLV de grupo del mapa mental recorren ese mismo camino de puntero-y-después-realloc, y las extensiones de cuenta como el MemoTransfer y el CpiGuard que habilitaste la lección pasada tienen su propio camino posterior a la creación vía el baile de reallocate), y solo funciona porque el puntero estaba ahí primero. Segundo, el financiamiento: asigna Y financia en 234 y la instrucción cuatro muere a mitad de transacción por el rent; vuelve a leer las líneas de dimensionamiento fantasma del paso 3, el depósito tiene que cubrir el tamaño posterior al realloc. Si tu falla no es ninguna de estas dos, corre el script de verify contra mi orden de fallas: extensiones presentes siquiera, objetivo del puntero, y después los campos, y lleva el primer assert que se dispara a la discusión del curso con tu lista de instrucciones.

Tómate un segundo para el hito, costó cuatro lecciones: un mint que cobra comisiones y les hace harvest, hace cumplir sus autoridades, deja que las cuentas de tenedor rechacen lo que deben rechazar, y ahora se renderiza como SPROUT en cualquier cosa que lea el TLV. Ese es un activo Token-2022 con forma de producción, el mismo patrón que entrega una stablecoin atestiguada por KPMG, y construiste cada byte de él desde instrucciones crudas.

El módulo que viene, la extensión para la que todo hasta acá te viene preparando: la que corre TU código en todas y cada una de las transferencias. Es el único lugar de este curso donde escribes un programa en Rust propio, el hook de harvest. Una nota honesta de logística para que la costura no te sorprenda: TransferHook es una extensión de mint de create-time, así que el SPROUT terminado de hoy nunca puede llegar a tener una. El módulo que viene acuñas una variante con hook desde cero, y es deliberadamente mínima, TransferHook y nada más, porque el tema de ese módulo es el hook, no la pila de recetas; los dos mints conviven lado a lado como lo hacen en producción un token principal y su variante de prueba de acceso restringido. Trae el toolkit. Nos vemos en el hook.
