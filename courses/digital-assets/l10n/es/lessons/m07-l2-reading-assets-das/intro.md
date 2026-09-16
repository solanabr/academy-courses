# Lee cualquier cosa: un solo script DAS para fungibles, activos Core y cNFTs

## Resumen

En la lección pasada acuñaste crates Harvest como NFTs comprimidos de Bubblegum v2 y aprendiste que el asset id de un crate es una PDA derivada de la dirección del árbol y del índice de la hoja. Después fuiste a traer uno y no obtuviste nada, porque un cNFT no tiene cuenta propia. También sigues teniendo SPROUT, el mint Token-2022 que vienes extendiendo desde el módulo 2, y los activos Core Almanac que acuñaste en m06-l2. Esos dos son cuentas comunes. Cualquier RPC los devuelve.

Así que ahora tienes tres activos en tres formas distintas, y una integración de billetera tiene que entender las tres sin importarle cuál es cuál. Demuéstrate la asimetría a ti mismo antes de que nombremos el arreglo. En tu workspace del curso:

```bash
mkdir -p overgrowth && cd overgrowth
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Pins, verificados contra npm el 2026-09-05. El tag `latest` de kit es 8.2.0, pero el número que decide el pin es el rango de peers, no el tag latest: `@solana-program/token-2022@0.15.0`, el minor actual del cliente de este curso, hace peer con kit ^7.0.0 — la línea 0.16 se movió a ^8 — así que kit queda en 7.1.1, el release más nuevo dentro de ese rango. Ese tren sale cada mes. Corre `npm view @solana-program/token-2022 peerDependencies` el día que hagas el scaffold.

```typescript
// overgrowth/shape-check.ts - does this id have an account at all?
// Run (from inside overgrowth/): npx tsx shape-check.ts <sprout> <almanac> <crate>
import { createSolanaRpc, address } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

async function main(): Promise<void> {
  for (const id of process.argv.slice(2)) {
    const { value } = await rpc.getAccountInfo(address(id), { encoding: 'base64' }).send();
    console.log(
      value === null
        ? `${id}  ->  NO ACCOUNT`
        : `${id}  ->  ${value.data[0].length} base64 chars, owner ${value.owner}`,
    );
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Una nota de cluster antes de correrlo: en este punto SPROUT y tus activos Almanac todavía viven en el surfnet (el lab migra los dos a devnet en unos quince minutos), así que apunta `RPC_URL` a tu surfnet para esta primera corrida. El veredicto del crate no depende del cluster: `NO ACCOUNT` se imprime para él contra tu surfnet, contra devnet donde de verdad vive, contra cualquier RPC que llegues a tener.

Pásale tus tres ids. SPROUT vuelve como propiedad del programa Token-2022, con un blob base64 que ya sabes decodificar. El activo Almanac vuelve como propiedad de un program id que empieza con `CoRE`. El crate Harvest imprime `NO ACCOUNT`, y el RPC no te está mintiendo. Ahí no hay nada. El crate existe como un hash dentro de un árbol de Merkle y como una fila en la base de datos de alguien.

Ese hueco es lo que cierra esta lección. El unificador es la Digital Asset Standard API, y para el final vas a tener `read-any-asset.ts`, un solo script que resuelve los tres por un mismo método, clasifica cada uno por su interface, imprime un precio en vivo para un fungible, y marca cuáles de las extensiones de un mint están haciendo algo de verdad y cuáles solo están configuradas. El repliegue de la ayuda, dicho sin rodeos: el transporte y el clasificador están resueltos por completo, el marcador de activo-versus-dormido es tuyo con dos de sus cinco casos mostrados, y la trampa de los creators vacíos es enteramente en solitario. El núcleo del clasificador es además el challenge calificado de esta lección, así que constrúyelo con cuidado.

## Una sola superficie de lectura para cada forma

### El problema, dicho una vez

Tres modelos de almacenamiento, una integración. Tómalos en orden de cuánto del activo vive on-chain.

Un mint fungible es una cuenta. Todo él. El supply, los decimals, las autoridades y cada extensión TLV que agregaste están en bytes que puedes traer y decodificar tú mismo, que es exactamente lo que hiciste en m01-l2 cuando escribiste tu propio decodificador y lo contrastaste con el cliente publicado.

Un activo de Metaplex Core también es una cuenta. Una sola cuenta, campos base más plugins, como viste cuando acuñaste el Almanac. Programa dueño distinto, misma historia de traer-y-decodificar.

Un NFT comprimido no es una cuenta. El programa Bubblegum hashea los datos del activo dentro de una hoja, y la raíz del árbol es lo que la blockchain guarda de verdad. Verificar un crate significa volver a reproducir una prueba de Merkle contra esa raíz. Leer un crate significa preguntarle a alguien que estaba mirando cuando aterrizó la transacción de mint y que anotó el contenido de la hoja. Ese alguien es un indexador.

![Tres carriles muestran a SPROUT y al activo Core devolviendo bytes de cuenta desde cualquier RPC mientras el NFT comprimido no devuelve ninguna cuenta, y los tres convergen en una sola llamada getAsset de DAS.](assets/v01-flowchart.webp)

### Qué es DAS en realidad

La Digital Asset Standard API, spec 1.1.0, es una extensión de JSON-RPC que un proveedor le atornilla a un endpoint RPC normal de Solana. La misma URL, la misma forma del body del POST, nombres de método distintos. Metaplex publica la spec y el indexador de referencia; los proveedores lo corren contra su propia infraestructura.

Los métodos que vas a usar hoy son cuatro:

- `getAsset`, un activo por id, que devuelve metadatos parseados, propiedad, regalía, estado de compresión y, para los fungibles, el token info.
- `getAssetsByOwner`, una lista paginada de todo lo que tiene una billetera, tipos mezclados incluidos.
- `getAssetProof`, la prueba de Merkle para un activo comprimido, que necesitas para escribir y no para leer.
- `searchAssets`, el mismo índice consultado por criterios arbitrarios (owner, collection, interface, frozen, burnt).

Hay más (`getAssetsByGroup`, `getAssetsByAuthority`, `getAssetsByCreator`, `getSignaturesForAsset`, `getNftEditions`, `getTokenAccounts`, y las variantes batch), y te vas a encontrar con dos de ellos en el lab. Pero cuatro cubren la superficie de lectura para toda una UI de billetera, que es el punto.

Aquí está el encuadre honesto, y importa más que la lista de métodos. DAS no es la blockchain. DAS es un **índice alquilado**: una base de datos que algún proveedor llenó mirando la blockchain, y tu lectura es tan fresca y tan completa como esa base de datos, nada más. Para SPROUT y el Almanac tienes opción, porque la cuenta está ahí mismo. Para el crate Harvest no tienes ninguna opción.

![Tres capas muestran la blockchain, una base de datos de indexador operada por un proveedor, y tu app llamando métodos de DAS, con un camino punteado de getAccountInfo que esquiva el índice solo para las cuentas.](assets/v02-diagram.webp)

### El enum de interface, recorrido

Cada respuesta de DAS arranca con un campo `interface`, y ese campo es el router de toda tu integración. El conjunto completo, verificado contra la documentación de los proveedores el 2026-08-22:

| Interface | Qué es |
|---|---|
| `V1_NFT` | no fungible de Token Metadata, el clásico |
| `V1_PRINT` | una impresión de edición sacada de una master edition |
| `LEGACY_NFT` | NFTs pre-estándar que el indexador todavía tiene que servir |
| `V2_NFT` | la forma no fungible más nueva de Token Metadata |
| `ProgrammableNFT` | pNFT, imposición a través de Token Auth Rules |
| `MplCoreAsset` | un activo de Metaplex Core, una cuenta, plugins adentro |
| `MplCoreCollection` | una colección Core |
| `MplCoreGroup` | un grupo Core bajo MIP-11, la Metaplex Improvement Proposal que agregó el agrupamiento (te los vas a encontrar en el mundo real, no en este curso) |
| `MplBubblegumV2` | un NFT comprimido de Bubblegum v2 |
| `FungibleAsset` | un fungible con metadatos adjuntos |
| `FungibleToken` | un fungible común |
| `Custom`, `Identity`, `Executable` | escotillas de escape que ruteas a un fallback |

Tu activo Almanac llega como `MplCoreAsset`. Tu crate Harvest llega como `MplBubblegumV2`. SPROUT llega como `FungibleToken` o `FungibleAsset` según las heurísticas de clasificación del proveedor; el corte se suele describir como ligado a los metadatos, pero en la práctica los proveedores devuelven `FungibleToken` para fungibles comunes incluso cuando un nombre resolvió bien (mi propia corrida imprimió `FungibleToken` y un nombre resuelto en la misma línea), así que trata al par como una sola categoría fungible y nunca ramifiques según cuál de los dos te tocó.

Dos cosas de este enum merecen una pausa, porque las dos le cuestan tiempo a la gente.

Primero, `MplCoreAsset` y `MplBubblegumV2` son agregados recientes. La documentación de DAS v2 de Alchemy los lista explícitamente como lo que v2 agrega sobre v1, lo que te dice que el enum creció después de que se escribiera mucho código de integración contra la forma vieja. Si heredas una base de código cuyo switch de activos tiene tres casos, por eso es.

Segundo, y esta es la trampa: **`is_agent` no es una variante de interface.** Es un campo booleano nullable que viaja junto a las filas `MplCoreAsset` cuando el activo lleva un plugin externo AgentIdentity, un plugin de Core más nuevo que marca un activo como la identidad de un agente on-chain, y los proveedores omiten el campo por completo cuando es false. Dos campos hermanos viajan con él, `asset_signer` (la dirección de firma del agente) y `agent_token` (su token asociado); este curso nunca acuña un activo de agente, pero tu lector se los va a encontrar en billeteras reales. Haz switch sobre `interface` para el tipo. Lee `is_agent` como un atributo. Tratarlo como un tipo es la clase de bug que funciona en cada prueba que escribas y se rompe con el primer activo de agente real que tenga un usuario.

![Cuatro tarjetas de categoría mapean los valores de interface de DAS a nft, compressed-nft, fungible y other, y solo compressed-nft requiere un RPC de DAS porque decide el flag de compresión.](assets/v03-comparison.webp)

Fíjate en lo que dice la tabla sobre `compressed-nft`. El nombre de la interface te acerca, pero el campo que de verdad decide es `compression.compressed`. Cada activo de DAS lleva un objeto `compression`, y en un NFT normal vuelve con `compressed: false` y cadenas de hash vacías. Ramifica sobre el booleano, no sobre el nombre, y tu clasificador sobrevive el próximo agregado al enum sin una edición. Ese es todo el instinto de diseño detrás del challenge al final de esta lección.

### Calladamente, DAS se volvió la API de precios

En algún punto del camino, a la API de activos le creció un segundo trabajo. Llama a `getAsset` con la opción de visualización `showFungible` puesta en true y la respuesta trae un objeto `token_info`: decimals, supply, el token program dueño, las autoridades de mint y de congelamiento y, cuando el mint califica, `price_info`.

`price_info.price_per_token` es un número en USD en vivo para un fungible, servido por el mismo endpoint que te acaba de contar de un cNFT. Sin segundo vendor, sin key de CoinGecko, sin tabla de mapeo de mint a coin-id. Funciona también para mints de Token-2022 con extensiones, lo cual no es obvio y vale la pena que lo verifiques tú mismo.

Dos límites van justo al lado de esa capacidad, y los dos están en la documentación de los proveedores y no en el blog post de nadie.

El precio está **cacheado hasta 600 segundos**. Eso está bien para una fila de portafolio y está mal para cualquier cosa que liquide valor. Si le estás poniendo precio a un swap, quieres un oráculo, y el curso planificado DeFi and RWA Engineering enseña a hacerlo como se debe.

El precio cubre más o menos los **diez mil tokens más grandes por volumen de 24 horas**. SPROUT es un token de curso en devnet. Nunca va a estar en ese conjunto, y tampoco la mayor parte de lo que tienen tus usuarios. Un `price_info` ausente no es un error y no es una mala configuración de tu lado. Ponlo en null por defecto, renderiza un guion, sigue adelante. Tu lector va a tratar esto como normal porque lo vas a escribir así en el lab.

### Pruebas, búsqueda y la paginación que de verdad vas a escribir

Dos de los cuatro métodos todavía no se han ganado su lugar, así que déjame gastarlos como se debe, porque los dos vienen con una idea equivocada pegada.

`getAssetProof` parece pertenecer a la lectura. No es así. Devuelve la prueba de Merkle para un activo comprimido: el id del árbol, la raíz actual, el hash de la hoja, el índice del nodo y un array de hashes hermanos cuya longitud es la profundidad del árbol. Necesitas todo eso para *escribir*, porque una transferencia o una quema de Bubblegum tiene que entregarle al programa suficientes hashes para recomputar la raíz y demostrar que tu hoja estaba adentro. Para mostrar un crate en una billetera, `getAsset` ya te dio todo, y traer una prueba que nunca usas es una ida y vuelta que pagas por nada.

```typescript
// overgrowth/proof-check.ts - a cNFT's proof is a write-time artifact.
// Run (from inside overgrowth/): npx tsx proof-check.ts <crate-asset-id>
import { das } from './das';

interface AssetProof {
  root: string;
  proof: string[];
  node_index: number;
  leaf: string;
  tree_id: string;
}

async function main(): Promise<void> {
  const id = process.argv[2];
  if (!id) throw new Error('pass a compressed asset id');

  const proof = await das<AssetProof>('getAssetProof', { id });
  console.log(`tree      ${proof.tree_id}`);
  console.log(`root      ${proof.root}`);
  console.log(`leaf      ${proof.leaf}`);
  console.log(`siblings  ${proof.proof.length} (tree depth)`);
  console.log(`node      ${proof.node_index}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Corre eso contra un crate Harvest y la cuenta de hermanos es la profundidad de tu árbol, la misma profundidad que elegiste cuando asignaste el árbol en la lección pasada. Y aquí está la parte que Metaplex pone en negrita en su propia documentación: una prueba queda vieja en el momento en que alguien más modifica el árbol. Tráela justo antes de la escritura, nunca al cargar la página, nunca desde un cache. Una prueba vieja no corrompe nada; simplemente falla, y falla de una manera que parece un error misterioso de transacción y no un problema de cache.

`searchAssets` es el otro, y es el método sobre el que se construye una vista de billetera de verdad. Consulta el mismo índice por criterios arbitrarios: owner, collection, creator, authority, interface, frozen, burnt, supply, con ordenamiento y paginación. Una sola llamada reemplaza las cuatro idas y vueltas separadas por-owner, por-collection, por-creator que si no coserías a mano.

```typescript
// overgrowth/search.ts - one query for a whole wallet view.
// Run (from inside overgrowth/): npx tsx search.ts <owner>
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';

interface Page<T> {
  total: number;
  limit: number;
  page: number;
  items: T[];
}

async function main(): Promise<void> {
  const owner = process.argv[2];
  if (!owner) throw new Error('pass an owner address');

  const counts = new Map<string, number>();
  for (let page = 1; ; page += 1) {
    const res = await das<Page<DasAsset>>('searchAssets', {
      ownerAddress: owner,
      burnt: false,
      page,
      limit: 1000,
      options: { showFungible: true },
    });
    for (const asset of res.items) {
      const key = classifyAsset(asset).category;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    if (res.items.length < res.limit) break;
  }
  for (const [category, n] of counts) console.log(`${category.padEnd(15)} ${n}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Ese loop es la forma de paginación que hay que internalizar. Las páginas de DAS empiezan en uno, el tamaño de página lo limita el proveedor (1000 es el techo común), y la condición de terminación es una página corta y no un total en el que confías. He visto a `total` quedarse atrás de los items en un índice ocupado, y un loop que le cree o para antes de tiempo o gira en falso. Una página corta es un hecho sobre la respuesta que tienes en la mano.

![Dos vías se ramifican desde el id de un activo comprimido: un camino de lectura corto por getAsset hasta el renderizado, y un camino de escritura por getAssetProof cuya prueba queda vieja con cualquier modificación del árbol.](assets/v04-flowchart.webp)

### Configurado no es lo mismo que activo

Aquí es donde un índice deja de alcanzar.

DAS te entrega la forma de un token. No te entrega el comportamiento del token. Esas son preguntas distintas, y confundirlas produce integraciones que muestran tonterías con toda confianza.

Trabájalo desde primeros principios. Una extensión tiene dos estados que se ven idénticos en cualquier lista de "qué extensiones tiene este mint": presente y haciendo algo, versus presente e inerte. Un `TransferFeeConfig` con una tasa de cero basis points y una comisión máxima de cero está estructuralmente ahí y económicamente ausente. Un `TransferHook` cuyo program id es la pubkey de todos ceros por defecto es un hook que no llama a nadie. Un `PausableConfig` que no está pausado es un arma cargada con el seguro puesto. En cada caso la presencia es un permiso que tiene el emisor, y el campo vivo es si lo han usado.

¿Por qué te importa esa distinción a ti en particular? Porque la presencia le dice a tu usuario qué podría pasar y el valor vivo le dice qué va a pasar en su próxima transferencia, y esas dos frases van en partes distintas de tu UI. "Este token puede ser pausado por su emisor" es una revelación de riesgo. "Este token está pausado ahora mismo" es un estado de error.

El ejemplo canónico resuelto es PYUSD, y ya leíste su mint una vez en este curso. Su mint de Token-2022 lleva ocho extensiones TLV: mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer y tokenMetadata. Configuradas pero dormidas, tal como las dejó el emisor: el program id del transfer hook es null y la comisión lee cero basis points con un máximo de cero. Volví a leer ese mint el 2026-08-22 mientras escribía esto y obtuve exactamente esas ocho, exactamente esos dos valores dormidos, que es la clase de afirmación que deberías volver a correr en vez de creerme.

El detalle mecánico con el que la gente tropieza: **DAS y la cuenta cruda no se ponen de acuerdo sobre cómo decir "sin definir."** Una respuesta de DAS anula el program id del transfer hook. El cliente generado de Token-2022, decodificando esos mismos bytes, te da la dirección de todos ceros del system program, porque eso es literalmente lo que hay en la cuenta. El mismo hecho, dos representaciones. Tu marcador tiene que saber cuál de las dos está mirando, y en el lab vas a leer el mint crudo exactamente por esta razón: el marcador es una lectura de la blockchain, no del índice.

![Paneles lado a lado comparan la vista del índice de DAS, donde el program id de un transfer hook dormido es null, con la vista de la cuenta decodificada, donde el mismo campo es la dirección de todos ceros.](assets/v05-annotated-code.webp)

### Elegir proveedor es parte de la lectura

Ahora la parte que nadie te cuenta hasta que te muerde: con los cNFTs, tu elección de RPC es una decisión de corrección, no de rendimiento.

Apunta `getAsset` a un RPC común y no obtienes una respuesta lenta ni una respuesta parcial. Obtienes `-32601 Method not found`, o peor, un proveedor que se lo traga y lo convierte en un resultado vacío y deja que tu UI renderice una billetera a la que le faltan tres de sus cinco activos. La propia documentación de Bubblegum de Metaplex lo dice sin rodeos: no todos los proveedores de RPC soportan la DAS API, revisa la página de proveedores. Tu lector debería fallar ruidosamente con ese código de error, que es la razón por la que el transporte que escribes en el paso 2 lo trata como caso especial.

Lo cual nos trae a un pedazo de historia reciente que le cambió la forma a toda esta decisión.

Durante años la respuesta por defecto a "cómo leo NFTs entre blockchains" fue SimpleHash. Era la API de NFT multi-chain más grande del negocio, y después Phantom la compró y cerró la API pública el **27 de marzo de 2025**. En Solana, la respuesta que absorbió el hueco fue DAS, lo que quiere decir que la pregunta dejó de ser "cuál API de NFT" y pasó a ser "cuál proveedor de DAS". Esa es una pregunta genuinamente distinta: estás eligiendo un operador de índice, y los operadores de índice se diferencian en frescura, en completitud, en cómo paginan, y en cómo le ponen nombre a las cosas.

La lista actual que vale la pena evaluar: **Helius**, **QuickNode**, **Alchemy** (cuyo DAS v2 es su propia cosa, ver abajo), **Triton** y **Shyft**. Para la compresión ZK, que es una historia de compresión distinta a la de Bubblegum, el índice es **Photon**.

No son intercambiables sin más, y el v2 de Alchemy es la ilustración más limpia. Migrar a él exige sufijar cada nombre de método con `_v2` (`getAsset` pasa a ser `getAsset_v2`), renombrar tres métodos de plano, renombrar de `displayOptions` a `options` el objeto de parámetros que le da forma a la respuesta, cambiar cómo lees la respuesta de proof-batch porque vuelve indexada por asset id en vez de ordenada, y manejar un campo `last_indexed_slot` nuevo en cada éxito. Nada de eso es irrazonable. Todo eso es trabajo que no descubres hasta que intentas cambiar. Escribe tu transporte de modo que el nombre del método y el endpoint sean lo único que toca un cambio, que es exactamente lo que hace `das.ts` en el lab.

![Una línea de tiempo va desde SimpleHash como la API de NFT por defecto, pasando por su cierre en marzo de 2025, hasta la lista actual de proveedores de DAS más Photon para la compresión ZK.](assets/v06-timeline.webp)

Ese hito del medio merece una frase propia. `solana-foundation/developer-content`, el repositorio detrás de los cursos oficiales de Solana, fue archivado el **2025-01-24**. Por lo tanto, cada curso oficial es anterior a Bubblegum v2 y anterior a los valores de interface sobre los que estás a punto de hacer switch. Si has estado contrastando este curso contra la documentación oficial y encontrando huecos, ese es el hueco, y es una fecha y no una conspiración.

¿Entonces cómo eliges uno de verdad? No por un blog post de benchmarks. Evalúa en los ejes que cambian tu código o tus reportes de incidentes, y evalúalos contra tus propios activos, en tu propio cluster, esta semana.

¿El proveedor soporta DAS en la red a la que despliegas, devnet incluido? Varios soportan solo mainnet, y descubrir eso a la hora de integrar en devnet es una mala tarde. ¿Cuál es el retraso de indexación para un activo comprimido recién acuñado, medido por ti, con un script de mint y un cronómetro? ¿Sirve `searchAssets` con los filtros que tu UI necesita, o solo el subconjunto por-owner y por-collection? ¿Cuál es el techo de página y se porta bien `total`? ¿La superficie de métodos es la spec canónica, o un dialecto como el sufijo `_v2` que te cuesta un shim? ¿Cómo cobra las llamadas a DAS contra las llamadas a un RPC común, dado que una vista de billetera son muchas lecturas chicas? Y la que la gente se salta: ¿qué pasa con un error, un error ruidoso de JSON-RPC o un array vacío y callado?

La última merece tu paranoia. Un índice que responde "no hay activos" cuando quiere decir "no implemento este método" va a pasar cada prueba que escribas y les va a mentir a tus usuarios en producción. Pruébalo a propósito: apunta tu lector a un RPC público común y confirma que tira error.

![Una tabla lista siete ejes de selección de proveedor con por qué cada uno cambia tu código y una auto-revisión para él, y al pie la lista de proveedores de DAS más Photon para la compresión ZK.](assets/v07-table.webp)

### El trade-off, nombrado

DAS te da una sola superficie de lectura sobre cuatro estándares de activos, más precios, al precio de confiar en la base de datos de alguien en vez de en el estado de la blockchain.

A qué renuncias, en concreto. La frescura es la del indexador, no la de la blockchain, así que un mint de hace cuatro segundos puede que todavía no esté ahí. La completitud también es la del indexador, y los reorgs y los huecos de backfill son reales. Los datos de precio están cacheados a diez minutos y cubren una rebanada superior de tokens. Y la cosa entera es una capa de lectura: no hace streaming, no hace backfill, y no es un pipeline de indexación.

¿Entonces cuándo no deberías usarlo? Tres casos, y todos son casos donde el índice es estrictamente peor que la cosa que copia. Cuando estás a punto de firmar una transacción cuya corrección depende del estado actual, lee la cuenta: un flag de congelado, un mint pausado, un delegado, un supply por el que estás a punto de dividir. Cuando necesitas un campo que DAS no modela, lee la cuenta: tus propias entradas TLV, estado de programa propio, cualquier cosa para la que el indexador no tenía esquema. Y cuando acabas de escribir y quieres confirmar, lee la cuenta, porque tu propia transacción está confirmada on-chain antes de estar en ninguna base de datos. La regla práctica que sobrevive: DAS responde "qué tiene este usuario", la blockchain responde "qué es verdad ahora mismo". Tu lector usó las dos hoy a propósito, DAS para los tres activos y un `fetchMint` directo para el estado de las extensiones, y esa división es el diseño, no un atajo.

![Un flujo de decisión rutea las lecturas críticas para firmar, las no modeladas y las recién escritas a la cuenta cruda mientras cualquier otra lectura se queda en DAS.](assets/v08-flowchart.webp)

Esa última cláusula sobre pipelines es un límite real, no modestia. Construir el pipeline (plugins de Geyser, gRPC de Yellowstone, ingestión por webhook, reproducir la historia hacia tu propio almacén) es una disciplina seria y le pertenece al curso planificado Client-Side Mastery, que trata DAS como un índice alquilado dentro de una disciplina de datos mucho más grande. Esta lección es consumo. Eres el cliente de un índice, y tu trabajo es ser uno bien portado: falla ruidosamente ante un método que falta, pon en null por defecto los precios que faltan, y nunca supongas que el índice sabe algo que la blockchain no ha confirmado.

## Lab: construye read-any-asset.ts

Un prerrequisito que, si no, te va a costar una hora. **Ningún indexador está mirando tu surfnet.** La lección pasada ya movió los crates a devnet detrás de un endpoint de DAS exactamente por esta razón; cada lab anterior a ese corría feliz contra un validador local, y ninguno de ellos puede servir esta lección, porque DAS es un índice y nadie está indexando un cluster que existe solo en tu laptop. Así que el crate ya está donde tiene que estar, y el Almanac también: el `getOrCreateAlmanac` de la lección pasada creó una colección Overgrowth Almanac en devnet, la registró en `crates.json`, y acuñó los crates adentro. Esa colección es lo que "el Almanac" quiere decir para el resto del curso. NO vuelvas a correr contra devnet el script de m06-l2 que crea la colección de surfnet; eso acuñaría un segundo Almanac, distinto, y dejaría tu crate afuera.

Lo que a devnet todavía le falta son dos cosas, presupuesta quince minutos. Primero, un ACTIVO Core Almanac para leer: acuña uno dentro de la colección de devnet que ya existe, con una variante chica del `mint-almanac.ts` de m06-l2 que cambie su conexión por el helper `getUmi()` de la lección pasada (el mismo `DAS_RPC_URL`, el mismo `wallet.json` fondeado) y lea la dirección de la colección desde `crates.json` en vez de `almanac.json`. Segundo, SPROUT: vuelve a correr `labs/m02-l4/add-metadata.ts` por su propio fallback de devnet documentado, `RPC_URL=https://api.devnet.solana.com WS_URL=wss://api.devnet.solana.com`, fondeando al payer desde el faucet cuando la llamada de airdrop te limite por rate. Ten claro qué recrea eso, para que la salida del marcador no te confunda después: el SPROUT de devnet que sale de este script es el build de referencia solo-metadatos, que lleva exactamente MetadataPointer y TokenMetadata, sin fee config, sin extensión de visualización. Tu marcador va a imprimir dos filas ACTIVE para él y nada más, y eso es correcto; las filas DORMANT de fee y de hook en los ejemplos de esta lección vienen de apuntar el bloque de extension-state a PYUSD, que es la auto-revisión del paso 5. Las direcciones de devnet van a ser distintas de las de tu surfnet; está bien, el paso 1 registra las nuevas.

Tres endpoints de aquí en adelante, y tenlos claros, porque no son intercambiables. `DAS_RPC_URL` es el endpoint de devnet con soporte DAS que exportaste en la lección pasada, el índice. `RPC_URL` es un RPC común de devnet, y es lo que usan las lecturas de cuenta cruda del paso 4. Y `MAINNET_DAS_RPC_URL` es el endpoint MAINNET del mismo proveedor de DAS (la mayoría de los proveedores sirven las dos redes con una sola key), usado exactamente una vez, para el sondeo de precio, porque el conjunto con precio es una rebanada de mainnet rankeada por volumen para la que ningún índice de devnet puede responder:

```bash
export DAS_RPC_URL="https://<your-das-devnet-endpoint>"
export RPC_URL="https://api.devnet.solana.com"
export MAINNET_DAS_RPC_URL="https://<your-das-mainnet-endpoint>"
```

Corre cada comando de este lab desde adentro de `overgrowth/`, la misma convención que la lección pasada, para que las rutas relativas como `assets.json` y `wallet.json` resuelvan.

**1. Anota lo que tienes.** Tu lector toma sus entradas de un archivo chico para que las lecciones posteriores y tus propios scripts puedan compartir una sola fuente de verdad. Una nota de continuidad antes de que lo llenes: si completaste el challenge de m07-l1 tal como está escrito, el crate Harvest común fue transferido a un signer descartable, así que tu billetera del lab ya no lo tiene. Usa el asset id del crate de logro soulbound para el campo `crate` (no puede haberse ido de tu billetera), o acuña un crate común nuevo; cualquiera de las dos mantiene los tres activos bajo un mismo `owner`, que es lo que necesita el paso de `getAssetsByOwner`. Crea `overgrowth/assets.json` con las cuatro direcciones:

```json
{
  "sprout": "<your SPROUT mint>",
  "almanac": "<an Almanac Core asset>",
  "crate": "<a Harvest crate asset id: the achievement crate, or a fresh mint (see note above)>",
  "owner": "<the wallet holding all three>"
}
```

**2. Un solo transporte, todos los métodos.** Cada llamada a DAS es el mismo POST con una cadena de método distinta, así que escríbelo una vez. El manejo de errores es la parte interesante y la razón por la que esto no es un one-liner:

```typescript
// overgrowth/das.ts - one JSON-RPC transport for every DAS method.

const DAS_RPC_URL = process.env.DAS_RPC_URL ?? '';

export interface DasError {
  code: number;
  message: string;
}

export async function das<T>(
  method: string,
  params: unknown,
  endpoint: string = DAS_RPC_URL,
): Promise<T> {
  if (!endpoint) {
    throw new Error('DAS endpoint is unset. Point DAS_RPC_URL at a DAS-supporting endpoint.');
  }
  const res = await fetch(endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 'overgrowth', method, params }),
  });
  if (!res.ok) {
    throw new Error(`${method}: HTTP ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as { result?: T; error?: DasError };
  if (body.error) {
    const hint =
      body.error.code === -32601
        ? ' (method not found: this endpoint does not implement DAS)'
        : '';
    throw new Error(`${method}: ${body.error.code} ${body.error.message}${hint}`);
  }
  if (body.result === undefined) {
    throw new Error(`${method}: empty result`);
  }
  return body.result;
}
```

La rama `-32601` es la que te salva la noche. El código estándar "method not found" de JSON-RPC es con lo que responde un endpoint que no es de DAS, y sin esa pista vas a pasar veinte minutos sospechando de tu asset id. El nombre del método es un parámetro y el endpoint es una variable de entorno, que es toda tu superficie de migración si alguna vez cambias de proveedor.

**3. El clasificador.** Este es el núcleo de ruteo, y también es el challenge calificado, así que léelo como una especificación sobre la que te van a evaluar y no como código para pegar:

```typescript
// overgrowth/classify.ts - route one DAS asset to a category, without a second RPC call.

export interface DasAsset {
  interface: string;
  compression?: { compressed?: boolean };
  token_info?: { price_info?: { price_per_token?: number } };
}

export interface AssetClassification {
  category: 'compressed-nft' | 'nft' | 'fungible' | 'other';
  compressed: boolean;
  fungible: boolean;
  pricePerToken: number | null;
  requiresDasRpc: boolean;
}

const NFT_INTERFACES = new Set<string>([
  'V1_NFT',
  'V1_PRINT',
  'V2_NFT',
  'LEGACY_NFT',
  'ProgrammableNFT',
  'MplCoreAsset',
  'MplBubblegumV2',
]);

const FUNGIBLE_INTERFACES = new Set<string>(['FungibleAsset', 'FungibleToken']);

export function classifyAsset(asset: DasAsset): AssetClassification {
  const compressed = asset.compression?.compressed === true;
  const fungible = FUNGIBLE_INTERFACES.has(asset.interface);
  const isNft = NFT_INTERFACES.has(asset.interface);
  const pricePerToken = asset.token_info?.price_info?.price_per_token ?? null;

  let category: AssetClassification['category'] = 'other';
  if (fungible) {
    category = 'fungible';
  } else if (isNft) {
    category = compressed ? 'compressed-nft' : 'nft';
  }

  return { category, compressed, fungible, pricePerToken, requiresDasRpc: compressed };
}
```

Tres decisiones en veinte líneas. `MplBubblegumV2` está en el conjunto de NFT en vez de tener su propia rama, porque la compresión es ortogonal a ser un NFT y el booleano ya la carga. Las interfaces desconocidas caen a `other` en vez de tirar error, porque un lector que se cae con un valor de enum agregado el martes pasado es un lector que entrega una caída de servicio por cada release de Metaplex. Y `requiresDasRpc` sigue a `compressed` y no a la interface, por la misma razón: es una afirmación sobre almacenamiento, no sobre estándar.

**4. Resuelve los tres.** Ahora el premio, un método para tres formas:

```typescript
// overgrowth/read-any-asset.ts - one reader for every Overgrowth asset.
// Run (from inside overgrowth/): npx tsx read-any-asset.ts
import { readFileSync } from 'node:fs';
import { createSolanaRpc, address } from '@solana/kit';
import { fetchMint } from '@solana-program/token-2022';
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';
import { readExtensionState } from './extension-state';

// PYUSD: a MAINNET Token-2022 mint inside the priced set, used as the price
// probe. The probe must ask a mainnet DAS endpoint: a devnet index has never
// heard of this mint, and no devnet token has the 24h volume the priced set ranks by.
const PRICE_PROBE = '2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo';
const MAINNET_DAS = process.env.MAINNET_DAS_RPC_URL ?? '';

interface AssetBook {
  sprout: string;
  almanac: string;
  crate: string;
  owner: string;
}

interface FullAsset extends DasAsset {
  id: string;
  content?: { metadata?: { name?: string } };
  ownership?: { owner?: string };
  compression?: { compressed?: boolean; tree?: string; leaf_id?: number };
  token_info?: {
    decimals?: number;
    token_program?: string;
    price_info?: { price_per_token?: number; currency?: string };
  };
}

function loadBook(): AssetBook {
  try {
    return JSON.parse(readFileSync('assets.json', 'utf8')) as AssetBook;
  } catch {
    throw new Error(
      'assets.json not found. Write it with sprout, almanac, crate and owner addresses (and run from inside overgrowth/).',
    );
  }
}

function getAsset(id: string, showFungible = false): Promise<FullAsset> {
  return das<FullAsset>('getAsset', { id, options: { showFungible } });
}

function line(label: string, asset: FullAsset): string {
  const c = classifyAsset(asset);
  const name = asset.content?.metadata?.name ?? '(unnamed)';
  const price = c.pricePerToken === null ? 'no price_info' : `$${c.pricePerToken}`;
  return [
    `${label.padEnd(9)} ${asset.interface.padEnd(16)} ${c.category.padEnd(15)}`,
    `das-rpc=${c.requiresDasRpc}`,
    `price=${price}`,
    `name=${name}`,
  ].join('  ');
}

async function main(): Promise<void> {
  const book = loadBook();

  const sprout = await getAsset(book.sprout, true);
  const almanac = await getAsset(book.almanac);
  const crate = await getAsset(book.crate);
  console.log(line('SPROUT', sprout));
  console.log(line('ALMANAC', almanac));
  console.log(line('CRATE', crate));

  if (classifyAsset(crate).compressed) {
    console.log(`  crate leaf: tree=${crate.compression?.tree} leaf_id=${crate.compression?.leaf_id}`);
  }

  if (!MAINNET_DAS) {
    throw new Error('MAINNET_DAS_RPC_URL is unset; the price probe needs a mainnet DAS endpoint.');
  }
  const probe = await das<FullAsset>(
    'getAsset',
    { id: PRICE_PROBE, options: { showFungible: true } },
    MAINNET_DAS,
  );
  const probePrice = classifyAsset(probe).pricePerToken;
  console.log(`PROBE     price_per_token=${probePrice ?? 'null'} (mainnet read, cached up to ~600s)`);

  const owned = await das<{ total: number; items: FullAsset[] }>('getAssetsByOwner', {
    ownerAddress: book.owner,
    page: 1,
    limit: 50,
  });
  const almanacSeen = owned.items.some((item) => item.id === book.almanac);
  console.log(`OWNER     ${owned.total} assets, almanac present=${almanacSeen}`);

  const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
  const mint = await fetchMint(rpc, address(book.sprout));
  const extensions =
    mint.data.extensions.__option === 'Some' ? mint.data.extensions.value : [];
  for (const state of readExtensionState(extensions)) {
    console.log(`  ${state.active ? 'ACTIVE ' : 'DORMANT'} ${state.kind.padEnd(22)} ${state.detail}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

No lo corras todavía: el import `./extension-state` apunta al archivo que escribes en el paso 5, así que la primera corrida exitosa le pertenece al paso 7.

Lee el objeto `options` en `getAsset` antes de seguir, porque es una costura de portabilidad. Helius documenta ese parámetro como `options`. El v1 de Alchemy lo llamaba `displayOptions` y lo renombró a `options` en v2. Los wrappers de SDK más viejos todavía emiten el nombre viejo. Si un proveedor ignora en silencio tu `showFungible` y no te vuelve ningún `token_info`, ese nombre es lo primero que hay que revisar.

La línea `PRICE_PROBE` necesita su propia palabra honesta, porque esconde otra trampa. SPROUT es un token de curso en devnet, así que va a resolver, va a clasificar como fungible, y va a volver **sin precio**, porque el conjunto con precio son más o menos los diez mil tokens más grandes por volumen de 24 horas. Ese es el caso normal, y tu línea renderiza `no price_info` en vez de caerse. La sonda contra PYUSD está ahí para que veas un `price_info` poblado al menos una vez con tus propios ojos, sobre un mint de Token-2022 con extensiones, desde el endpoint de mainnet de tu proveedor, porque el conjunto con precio es un fenómeno de mainnet y el mint mismo solo existe ahí. Cuando SPROUT eventualmente se negocie en algún lado con volumen real, la misma llamada se llena y no cambias nada.

**5. El marcador, casi todo tuyo.** Dos casos están resueltos, tres son tu relleno:

```typescript
// overgrowth/extension-state.ts - configured is not the same as active.
import { AccountState, type Extension } from '@solana-program/token-2022';

const NULL_ADDRESS = '11111111111111111111111111111111';

export interface ExtensionState {
  kind: string;
  active: boolean;
  detail: string;
}

export function readExtensionState(extensions: readonly Extension[]): ExtensionState[] {
  return extensions.map((ext): ExtensionState => {
    switch (ext.__kind) {
      case 'TransferFeeConfig': {
        const bps = ext.newerTransferFee.transferFeeBasisPoints;
        const max = ext.newerTransferFee.maximumFee;
        return { kind: ext.__kind, active: bps > 0 && max > 0n, detail: `${bps} bps, max ${max}` };
      }
      case 'TransferHook':
        return {
          kind: ext.__kind,
          active: ext.programId !== NULL_ADDRESS,
          detail: `programId ${ext.programId}`,
        };
      // YOUR FILL: PausableConfig (ext.paused), DefaultAccountState
      // (ext.state === AccountState.Frozen), ScaledUiAmountConfig (ext.multiplier !== 1).
      default:
        return { kind: ext.__kind, active: true, detail: 'presence is the behavior' };
    }
  });
}
```

La rama por defecto es una elección de diseño, no pereza. Para `PermanentDelegate`, `MintCloseAuthority`, `MetadataPointer` y compañía, la presencia es el comportamiento: no hay un segundo campo que los prenda, y reportarlos como activos es la respuesta honesta. Tus tres rellenos son los que tienen un interruptor vivo. Apunta el script terminado al mint de PYUSD en vez de a SPROUT por un minuto y deberías ver ocho extensiones con `TransferFeeConfig` y `TransferHook` marcados `DORMANT`. Esa es una buena auto-revisión, porque es el mismo resultado que obtuve el 2026-08-22. Una nota de honestidad sobre tus tres rellenos: ni tu SPROUT de devnet ni PYUSD llevan Pausable, DefaultAccountState ni ScaledUiAmountConfig, así que ningún mint vivo de este lab los ejercita. Demuéstralos por la vía barata: aliméntale al marcador objetos de extensión armados a mano en una prueba descartable, dale vuelta a `paused`, `state` y `multiplier`, y mira cómo cambian los veredictos. Un relleno que solo corriste contra extensiones que nunca ocurren es un relleno que no has demostrado.

**6. La trampa de los creators vacíos, completamente en solitario.** Echa mano de `getAssetsByCreator` sobre un mint de pump.fun y vas a recibir cero resultados para un token que visiblemente existe y se negocia todo el día. La tentación es culpar al índice, reintentar, o esperar a que se venza un cache. Todo mal. **pump.fun no llena el array de creators de Metaplex**, así que una consulta que usa creator como clave no tiene nada con qué hacer match. El arreglo de quien trabaja en esto es usar la autoridad de actualización como clave, o suscribirse al programa directamente.

Reprodúcelo, después arréglalo. Aquí está el diagnóstico; cablear el resultado dentro de tu lector es cosa tuya:

```typescript
// overgrowth/creators-probe.ts - why a creator query comes back empty.
// Run (from inside overgrowth/): npx tsx creators-probe.ts <mint>
import { das } from './das';

interface Page {
  total: number;
  items: { id: string; interface: string }[];
}

async function main(): Promise<void> {
  const mint = process.argv[2];
  if (!mint) throw new Error('pass a mint address');

  const asset = await das<{
    creators?: { address: string; verified: boolean }[];
    authorities?: { address: string; scopes: string[] }[];
  }>('getAsset', { id: mint });

  const creators = asset.creators ?? [];
  const authority = asset.authorities?.[0]?.address;
  console.log(`creators: ${creators.length}`);
  console.log(`authority: ${authority ?? 'none'}`);

  if (creators[0]) {
    const byCreator = await das<Page>('getAssetsByCreator', {
      creatorAddress: creators[0].address,
      page: 1,
      limit: 10,
    });
    console.log(`getAssetsByCreator -> ${byCreator.total}`);
  }
  if (authority) {
    const byAuthority = await das<Page>('getAssetsByAuthority', {
      authorityAddress: authority,
      page: 1,
      limit: 10,
    });
    console.log(`getAssetsByAuthority -> ${byAuthority.total}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Córrelo contra un mint de pump y `creators: 0` se imprime antes de que corra cualquier consulta, que es todo el diagnóstico en una línea. Córrelo contra tu activo Almanac y el array de creators está poblado, porque lo acuñaste a través de un estándar que llena el campo. Generaliza la lección y no el parche: antes de construir un feature sobre una clave de consulta de DAS, revisa que los activos que te importan de verdad llenen esa clave. Los creators, las collections y las authorities son todos opcionales en la práctica, diga lo que diga el esquema.

**7. Entrégalo.** `npx tsx read-any-asset.ts` ahora debería imprimir tres líneas de clasificación, una referencia a la hoja del crate, un precio poblado que viene de la sonda, un conteo del owner con tu activo Almanac presente, y el bloque de estado de las extensiones. Forma de una corrida que pasa, con tus propias direcciones y valores en lugar de los placeholders:

```text
SPROUT    FungibleToken    fungible         das-rpc=false  price=no price_info  name=SPROUT
ALMANAC   MplCoreAsset     nft              das-rpc=false  price=no price_info  name=Almanac Vol. 1
CRATE     MplBubblegumV2   compressed-nft   das-rpc=true   price=no price_info  name=Harvest Crate
  crate leaf: tree=<your tree> leaf_id=<n>
PROBE     price_per_token=<live number> (mainnet read, cached up to ~600s)
OWNER     <n> assets, almanac present=true
  ACTIVE  MetadataPointer        presence is the behavior
  ACTIVE  TokenMetadata          presence is the behavior
```

(Esas dos filas ACTIVE son el bloque entero para el SPROUT de devnet solo-metadatos. Las filas `DORMANT TransferFeeConfig 0 bps, max 0` y `DORMANT TransferHook` aparecen cuando apuntas el mismo bloque a PYUSD, la auto-revisión del paso 5.)

Lee ese bloque como un conjunto de assertions y no como decoración. La columna `das-rpc` es true exactamente una vez. La columna de categoría tiene tres valores distintos. La línea de la sonda tiene un número adentro. Si alguna de esas tres afirmaciones es falsa, el criterio no se cumple, sea cual sea el código con el que salga el script.

![La salida que pasa está anotada línea por línea: tres categorías, una columna das-rpc verdadera solo para el NFT comprimido, un precio en vivo de la sonda, y el estado de las extensiones desde el mint crudo.](assets/v09-annotated-code.webp)

Cablea `search.ts` dentro del mismo workspace ya que estás aquí. No es parte del criterio, pero un conteo por categoría sobre una billetera entera es la consulta con la que abre una integración de verdad, y correrlo contra tu propia dirección de owner es la forma más rápida de ver si la paginación de tu proveedor se porta como el loop supone.

![Un diagrama de componentes muestra tres artefactos previos alimentando al lector de activos, cuyos módulos emiten clasificaciones, un precio y un reporte de extensiones, con el streaming y el backfill marcados fuera del límite.](assets/v10-diagram.webp)

## Challenge

`classify-das-asset`. La lógica es exactamente el paso 3, pero la convención de llamada del calificador es más plana que tu módulo del workspace: invoca `classifyAsset(iface, detailsJson)`, donde `iface` es la cadena de `interface` de DAS y `detailsJson` es el resto de la respuesta de `getAsset` serializada como una cadena JSON. Hazle `JSON.parse` a esa cadena antes que nada, después rutea exactamente como arriba; las mismas tres decisiones, el mismo `AssetClassification` de salida. Haz que pasen los casos que fallan del starter.

Las pruebas pegan en las esquinas que importan en producción y no en el camino feliz. Un activo `MplBubblegumV2` con `compression.compressed` en true tiene que volver como `compressed-nft` con `requiresDasRpc` en true. Un `MplCoreAsset` tiene que volver como `nft` con `requiresDasRpc` en false, porque un activo Core es una cuenta común y no hace falta ningún índice para leerlo. Un `FungibleToken` que lleva `token_info.price_info.price_per_token` tiene que sacar ese número a la superficie, y un fungible sin price info tiene que devolver null en vez de cero, undefined, o un error tirado. Cero es un precio. Null es una ausencia. Renderizar una ausencia como un precio es cómo una UI de portafolio le dice a un usuario que lo que tiene no vale nada.

Las pistas del challenge te dan los conjuntos de interface, la revisión de compresión y la ruta del precio. Si escribiste el paso 3 tú mismo ya lo resolviste; si pegaste el paso 3, escríbelo de nuevo desde la firma de tipos y mira si las tres decisiones te vuelven.

## Checkpoint

El criterio: `npx tsx read-any-asset.ts` resuelve y clasifica correctamente los tres activos, e imprime un `price_per_token`. Tres líneas, tres categorías, un número. Di la respuesta en voz alta antes de seguir, porque es la cosa que esta lección existe para instalar: `FungibleToken` o `FungibleAsset` para SPROUT, `MplCoreAsset` para el Almanac, `MplBubblegumV2` para el crate Harvest, y solo el último necesitó un endpoint con soporte DAS para existir siquiera.

Las fallas que espero, en el orden en que pasan. Si cada llamada muere con `-32601 (method not found)`, tu `DAS_RPC_URL` es un RPC común y ninguna cantidad de reintentos va a cambiar eso; consigue un endpoint de DAS. Si el crate devuelve un asset id pero los campos están vacíos, el indexador no ha alcanzado a un mint que enviaste hace segundos, que es el trade-off de frescura llegando en persona: espera, después vuelve a correrlo. Si falta `token_info` por completo en SPROUT, te saltaste `showFungible`, o tu proveedor quiere ese flag bajo `displayOptions`. Y si `getAssetsByOwner` devuelve tu activo Almanac pero no tu crate, revisa el campo owner del crate y no la consulta, porque el owner de un NFT comprimido es un atributo de la hoja y transferir uno reescribe la hoja. La forma habitual en que los estudiantes llegan aquí es exactamente el challenge de m07-l1: el crate común fue transferido a un signer descartable ahí, que es la razón por la que el paso 1 te dijo que registraras el crate de logro o un mint nuevo en su lugar.

La primera de esas fallas se imprime exactamente así, directo desde el camino de error de `das.ts` que escribiste en el paso 2:

```text
getAsset: -32601 Method not found (method not found: this endpoint does not implement DAS)
```

Un entregable que no es código, y lo digo literalmente: anota qué proveedor usaste y qué eje lo decidió. Una frase en tu README alcanza. "Elegí X porque sirve DAS en devnet e indexó un crate nuevo en menos de N segundos, medido en esta fecha." Dentro de seis meses, cuando un incidente te haga reconsiderar, esa frase es la diferencia entre volver a correr una prueba y volver a correr la evaluación entera. También te obliga a haber medido algo de verdad en vez de haber elegido al proveedor cuya página de documentación cargó primero, que es, con franqueza, como se toman la mayoría de estas decisiones. Yo la tomé así. La medición toma veinte minutos y es la única parte de la elección de proveedor que es tuya y no del marketing.

Un hábito para llevarte de aquí, que vale más que el script: trata cada campo de DAS como opcional hasta que lo hayas visto poblarse para los activos que de verdad te importan. `price_info` en un token chico, `creators` en un mint de pump, `is_agent` en cualquier cosa que no sea un activo Core. El esquema es una promesa sobre la forma. Solo una lectura en vivo es una promesa sobre el contenido.

Ya puedes leer cada activo que construiste en este curso con una sola llamada, clasificarlo sin una segunda búsqueda, y decirle a un usuario la diferencia entre un token que puede ser pausado y un token que está pausado. Cada uno de esos activos, eso sí, es o una cuenta real o una hoja en el árbol de alguien. La próxima lección empuja sobre eso con una pregunta más filosa: ¿y si un token fungible no tuviera cuenta propia, qué significaría siquiera eso para los balances y las transferencias, y cuándo querrías eso de verdad? Trae el marcador. Vas a necesitar el hábito de preguntar qué hay realmente ahí.
