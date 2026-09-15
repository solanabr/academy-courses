# Los estándares de metadatos que las billeteras realmente leen

## Resumen

En m05-l2 finalizaste el conjunto de extensiones de plataforma de lanzamiento del SPROUT y el informe de enrutabilidad R6; la mitad fungible de la economía de Overgrowth está decidida, es negociable y está en cumplimiento, con sus metadatos escritos como TLV nativo de Token-2022 en m02-l4. Esta lección abre la mitad NFT con un mapa, no con una acuñación: dónde viven realmente el nombre, la imagen y los traits de un activo. La respuesta son tres capas, numeradas una vez para que siempre puedas preguntar en qué capa vive un campo: la capa 1 es el documento JSON off-chain desde el que renderiza una billetera, la capa 2 es la pequeña struct on-chain cuyo puntero guarda la dirección de ese JSON, y la capa 3, solo en mints Token-2022, es el TLV nativo que ya conectaste a mano. Vas a traer un NFT famoso, imprimir su registro on-chain junto a su JSON off-chain, atrapar a los dos discrepando sobre la regalía, y terminar capaz de ubicar cualquier campo de cualquier activo de Solana en su capa. El repliegue de la ayuda: los dos scripts corren completos tal como se entregan, la tabla de ubicación de campos es tuya para llenar a mitad del lab, y el challenge es totalmente en solitario contra un activo que yo no elijo.

Abre tu billetera y mira cualquier NFT. Un nombre, una imagen, tal vez una lista de traits y una regalía en la página del marketplace. Se lee como un solo objeto. No lo es, y no es ni un solo lugar: parte de esa pantalla son bytes en una cuenta on-chain, la mayoría es un blob JSON en una URI a la que la cuenta apenas apunta, y en un mint Token-2022, parte de ella es TLV que ya sabes escribir. Antes de que te explique una sola capa, ve a mirar las costuras tú mismo.

Ninguna toolchain nueva hoy. En el workspace de tu curso, crea `fetch-asset.ts` a partir del lab de abajo y córrelo:

```bash
npx tsx fetch-asset.ts F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk
```

Esa dirección es el mint de Mad Lads #8420, uno de los NFTs más famosos de Solana. Esto es lo que volvió cuando lo corrí mientras escribía esto, el 2026-08-23:

```text
=== ON-CHAIN (the Data struct, decoded from the metadata PDA) ===
account:                DZAZ3mGuq7nCYGzUyw4MiA74ysr15EfqLpzCzX2cRVng (owner: metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s)
name:                   "Mad Lads #8420"
symbol:                 "MAD"
uri:                    https://madlads.s3.us-west-2.amazonaws.com/json/8420.json
seller_fee_basis_points: 420
creator:                5XvhfmRjwXkGp3jHGmaKpqeerNYjkuZZBYLVQYdeVcRv verified=true share=0
creator:                2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW verified=true share=100

=== OFF-CHAIN (the JSON document at that uri) ===
name:        "Mad Lads #8420"
symbol:      "MAD"
description: "Fock it."...
image:       https://madlads.s3.us-west-2.amazonaws.com/images/8420.png
attributes:  7 traits, e.g. {"trait_type":"Gender","value":"Male"}
properties.files: 2 file(s)
properties.category: image
seller_fee_basis_points in JSON: 500
top-level keys: name, description, symbol, image, external_url, seller_fee_basis_points, attributes, properties
```

Quédate un minuto con esa salida, porque la lección entera está ahí. La cuenta on-chain guarda cinco cosas y ninguna imagen. La imagen, los traits, la descripción, todo vive en un archivo JSON en la `uri`. La regalía aparece dos veces, y las dos copias discrepan: 420 basis points on-chain, 500 en el JSON. Y la URI de una de las colecciones más valiosas de Solana apunta a un bucket S3 de Amazon. Cada una de esas observaciones se convierte en una sección de esta lección.

## Dónde vive realmente un activo

Este es el colapso que desmitifica toda la pila: un registro de activo on-chain es solo una struct con un puntero, y el contenido rico es un documento JSON en ese puntero. Esa es toda la arquitectura. Todo lo que viene después de ese colapso es nombrar los campos de la struct, nombrar los campos del JSON, y hacer la única pregunta que el colapso fuerza: ¿qué pasa cuando el puntero sobrevive a la cosa a la que apunta?

¿Por qué construirlo así, para empezar? Lleva primero la alternativa ingenua hasta el final. Supón que guardaras la imagen on-chain. Un PNG de la calidad de Mad Lad ocupa unos cientos de kilobytes; los bytes on-chain cuestan rent por byte, una cuenta tiene un techo de 10 MiB, y cada byte de ella se replica a cada validador para siempre. Estarías pagando precios de almacenamiento de grado validador, en miles de máquinas, por una imagen que nunca cambia y que lee una billetera a la vez. Así que nadie hace eso. La cadena guarda lo que la cadena hace bien, hechos pequeños y autenticados: quién hizo esto, cómo se llama, dónde está el resto. El resto vive donde vive el contenido masivo, detrás de una URI. Acuñaciones baratas, contenido rico, y un modo de falla nuevo que vamos a nombrar con honestidad antes del lab.

![Mapa de tres capas de un activo de Solana numeradas de uno a tres, el JSON off-chain, la struct Data on-chain cuya uri apunta a él, y el TLV nativo de Token-2022.](assets/v01-diagram.png)

### Capa 1: el JSON off-chain desde el que renderiza la billetera

Empieza por la capa que llena la mayor parte de la pantalla. El documento off-chain sigue el estándar JSON de Token Metadata, y los campos que viste en la salida de Mad Lads son el núcleo del estándar. Recórrelos de a uno, porque una billetera también los recorre:

- **`name`** y **`symbol`**: strings de visualización. Nota que también existen on-chain; las copias del JSON son lo que la mayoría de las billeteras realmente renderiza, y mantener las dos en sincronía es una norma, no una regla.
- **`description`**: texto libre, renderizado en páginas de detalle. La de Mad Lads tiene dos palabras.
- **`image`**: la URI del arte. Este es el campo del que está hecha una grilla de billetera. Una `image` muerta es un cuadrado vacío.
- **`animation_url`**: URI opcional para video, audio, 3D, o un build interactivo. Las billeteras que lo soportan renderizan esto en vez de la imagen fija.
- **`external_url`**: un link hacia afuera, a un sitio web. Pura convención, seguido desactualizado, nunca estructural.
- **`attributes`**: la lista de traits, un array de pares `{ "trait_type": ..., "value": ... }`. Mad Lads #8420 lleva siete, empezando con `{"trait_type":"Gender","value":"Male"}`. Los marketplaces construyen sus herramientas de rareza enteramente a partir de ese array.
- **`properties.files`**: un array de entradas `{ uri, type }` (más una flag `cdn` opcional) que lista cada archivo que compone el activo, típicamente la imagen otra vez más alternativas. El `type` es un tipo MIME; las billeteras caen en adivinar por la extensión cuando falta.
- **`properties.category`**: una palabra que les dice a los renderizadores qué tipo de activo es este: `image`, `video`, `audio`, `vr`, o `html`.

El estándar fungible es el subconjunto mínimo del mismo documento: `name`, `symbol`, `description`, `image`. Un token como USDC necesita un logo y un nombre, no una lista de traits. Misma familia de schema, menos campos, que es por lo que una convención JSON sirve a las dos mitades del mundo de los activos.

Dos cosas que viste en el documento real merecen sospecha. Primero, `seller_fee_basis_points` aparece en el JSON de Mad Lads en 500. Ese es un campo legado: la regalía se fue on-chain hace años, y una copia en el JSON es lo que quien subió el archivo escribió por casualidad el día de la subida. El 420 on-chain es lo que leen los marketplaces; el 500 del JSON es un fósil. Cuando dos capas discrepan, la capa on-chain es la que tiene una autoridad de actualización y un timestamp, y la copia off-chain se podre en silencio. Segundo, ¿dónde está el schema en sí? La URL canónica del schema, https://schema.metaplex.com/nft1.0.json, es la que el ecosistema estandarizó. La sondeé mientras escribía esta lección, 2026-08-23: el host ya no resuelve, para nada. El registro de DNS se fue. El estándar no cambió, pero su dirección de referencia canónica está muerta, así que esta lección lleva el contrato de campos en prosa y en el script validador del lab en vez de meterte en un vacío.

Esa URL muerta no es un accidente aislado, y vale treinta segundos de historia porque explica por qué tanto de lo que medio recuerdas sobre metadatos de NFT va una generación atrás. La educación oficial de Solana se congeló a mitad de la trama: el repositorio solana-foundation/developer-content, la fuente detrás de los cursos oficiales, se archivó como solo lectura el 2025-01-24. Cada curso oficial es anterior a la pila de NFT actual. Los propios docs de Metaplex se mudaron de dominio, y el viejo developers.metaplex.com ahora hace 308-redirect a metaplex.com/docs, dejando años de links de tutorial a un redirect de su contenido. El estándar que estás aprendiendo hoy es estable; las URLs a su alrededor no. Reverifica cualquier cita de metadatos antes de confiar en ella, incluida, en cinco años, esta.

![Línea de tiempo desde el estándar Token Metadata de 2021, pasando por el archivado de la educación oficial de Solana el 2025-01-24, hasta 2026, cuando el host canónico del schema está muerto y Token Metadata mismo es legado.](assets/v02-timeline.png)

### Capa 2: la struct Data on-chain, cinco campos y un puntero

Ahora la capa chica, la que tu script decodificó a mano. Para un activo legado, el registro on-chain es una cuenta derivada del mint: el PDA de metadatos, sembrado con el string literal `"metadata"`, el id del programa Token Metadata, y la dirección del mint, propiedad del programa Token Metadata en `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`. Tu script lo derivó, lo leyó, y se salteó 65 bytes de header (una clave de cuenta de un byte, la autoridad de actualización de 32 bytes, el mint de 32 bytes) para llegar a la parte que el estándar llama la struct `Data`:

```text
Data {
  name:                    String   // Borsh: u32 length + bytes, stored at fixed capacity, null-padded
  symbol:                  String
  uri:                     String   // the pointer to layer 1
  seller_fee_basis_points: u16      // 420 for Mad Lads = 4.20%
  creators:                Option<Vec<Creator { address, verified, share }>>
}
```

Cinco campos. Esa es toda la identidad on-chain de un NFT legado, y ahora la salida que obtuviste al principio de la lección debería leerse distinto: a la cuenta nunca le "faltaba" la imagen. La imagen nunca debía estar ahí. Un compañero que trae la cuenta de un NFT, no encuentra traits ni imagen, y concluye que el activo está roto leyó mal la arquitectura; nada se guarda donde miró, por diseño, y el campo `uri` era la respuesta sentada en su propio dump.

Cada campo se gana una oración de respeto. `name` y `symbol` son las copias on-chain-autoritativas de los strings de visualización, guardadas con relleno nulo a capacidad fija, que es por lo que tu decodificador recorta los ceros del final. `uri` son los 200 bytes más estructurales del activo: es el único puente entre lo que la cadena autentica y lo que muestra la billetera. `creators` lleva hasta cinco direcciones con una flag `verified` cada una, y esa flag es superficie de seguridad real: solo puede ponerse en true si ese creator firma de verdad, que es como los marketplaces distinguen al verdadero creador de la colección de un copiador que pegó la misma dirección sin verificar. Tu salida de Mad Lads muestra a los dos creators verificados, el primero con share 0 (una dirección que firma por la colección) y el segundo con share 100 (a donde se supone que van las regalías).

Y `seller_fee_basis_points`, el campo que discrepó del JSON. On-chain dice 420, y on-chain gana. Pero ahora que confías en la copia correcta, aquí está la trampa más profunda: no leas ni la copia ganadora como una regalía garantizada. Es una preferencia declarada, solo indicativa, y nada en el token program impone una comisión al momento de la transferencia. Cómo se atornilló la imposición después, y con qué profundidad falló, es la historia de m06-l3, demostrada en vez de afirmada. Por hoy, calibra: este u16 es lo que los marketplaces eligen honrar, no lo que tienen que honrar.

![Comparación anotada lado a lado de la struct Data on-chain de Mad Lads #8420 y su JSON off-chain, donde image y traits existen solo off-chain y la regalía lee 420 on-chain pero 500 off-chain.](assets/v03-annotated-code.png)

### Capa 3: el camino nativo de Token-2022 que ya construiste

No acabas de aprender un tercer sistema de metadatos en m02-l4. Construiste uno. El nombre del SPROUT no vive en ninguna cuenta de Metaplex: vive en los bytes del propio mint, como una entrada TLV TokenMetadata (discriminador `[112,132,90,90,11,88,157,87]`) que lleva `name`, `symbol`, `uri`, y los pares clave-valor de `additional_metadata` donde escribiste `harvest_season = "spring"`. Al lado de ella está la extensión MetadataPointer, que apuntaste al mint mismo. Así que ubica toda esa construcción en el mapa de hoy: los metadatos nativos de Token-2022 son las capas 1 y 2 colapsadas dentro de la cuenta del mint, con la misma convención JSON todavía disponible al final de su campo `uri` para cualquier cosa rica.

La comparación contra el layout de Metaplex es donde el diseño se gana su lugar. En el modelo legado, la identidad vive en una cuenta separada que posee un programa distinto, y un lector tiene que derivar el PDA para encontrarla. En el modelo nativo no hay nada que derivar ni nada separado que traer: un `getAccountInfo` sobre el mint devuelve identidad, supply y cada extensión en una sola lectura. Y el argumento anti-spoofing de m02-l4 encaja limpio en el vocabulario de hoy: un MetadataPointer apuntado a cualquier lado que no sea el mint mismo reintroduce una indirección que un atacante puede apuntar a la cuenta de metadatos de otra persona, que es por lo que autorreferencial es el layout que conectaste y el único que deberías entregar. Los trade-offs corren para el otro lado también, y nombrarlos es el punto de un mapa. Los metadatos TLV nativos viven en el mint, así que cada campo que agregas hace crecer la cuenta y su rent, y todo el mecanismo existe solo en mints Token-2022. Los mints SPL clásicos, o sea la mayoría de los activos ya sueltos por ahí, no pueden llevarlo, que es por lo que las capas de Metaplex no se van a ningún lado y por lo que necesitas las tres columnas de la tabla que estás a punto de llenar.

![Comparación del modelo de cuenta de metadatos separada de Metaplex y el modelo TLV dentro del mint de Token-2022 en ubicación de la identidad, cantidad de lecturas, superficie de spoofing, crecimiento del rent, disponibilidad por programa, y el estándar JSON off-chain compartido.](assets/v04-comparison.png)

### El puntero es la junta débil: la realidad del almacenamiento

Cada capa de arriba termina en una `uri`, así que la durabilidad de todo el activo se reduce a una pregunta: ¿por cuánto tiempo sigue resolviendo esa URI? La cuenta on-chain no garantiza nada al respecto. El rent mantiene la struct viva para siempre; la struct va a apuntar felizmente a un 404 para siempre también. Un link muerto o un archivo cambiado en silencio significa que el "NFT" resuelve a nada, o peor, a otra cosa, mientras la cadena sigue atestiguando que el puntero está exactamente donde siempre estuvo.

Lo que nos trae de vuelta a la línea más silenciosamente alarmante de la salida de tu sondeo: `madlads.s3.us-west-2.amazonaws.com`. Mad Lads, una colección emblemática de Solana, sirve sus metadatos e imágenes desde un bucket S3 de Amazon. S3 es rápido, barato y mutable, y persiste precisamente mientras alguien siga pagando la cuenta y controle el bucket. Eso no es un escándalo; es una decisión de norma tomada en público, y los activos populares se archivan y se espejan en la práctica. Pero velo por lo que es: el contenido del activo está alquilado, y quien alquila es el equipo, no tú.

La alternativa que pone la permanencia primero es la familia Arweave. El modelo de Arweave es paga una vez, guarda para siempre, financiado por un mecanismo de dotación en vez de una suscripción, y las URIs de Arweave están difundidas por las colecciones de Solana exactamente por eso. Irys es el uploader que está adelante y que la CLI de Metaplex documenta como su opción por defecto, así que el camino de menor resistencia del herramental ya deja tu JSON en almacenamiento permanente. IPFS merece una salvedad honesta: las URIs direccionadas por contenido son una mejora de integridad real, ya que el hash en la URI es el contenido, pero la disponibilidad depende de que alguien siga pineando el archivo, y no verifiqué en esta ronda qué tan sano está el mercado comercial de pinning. Así que la regla honesta, la que hay que llevarse de esta lección: la URI es permanente solo si el almacenamiento lo es.

Hay una segunda decisión de norma escondida al lado del almacenamiento: la mutabilidad. El PDA de metadatos tiene una autoridad de actualización, y el TLV TokenMetadata tiene una también; cualquiera de las dos puede reescribir la `uri` o los campos mañana, a menos que esa autoridad se abandone. Los metadatos mutables son cómo un rug cambia el arte después de la acuñación, y son también cómo un juego legítimo evoluciona un ítem, arregla un typo, o migra de host. Inmutable-más-permanente es la postura de grado coleccionista; mutable-más-alquilado es la postura de servicio en vivo. Ninguna de las dos es un valor por defecto. Es una elección que vas a hacer explícitamente, por clase de activo, cuando Overgrowth acuñe el Almanac la lección que viene.

![Diagrama de flujo desde la dirección del mint pasando por el registro on-chain, uri, documento JSON e imagen, con puntos de ruptura en el host de la uri, el JSON mutable, y el link de la imagen que la cadena nunca detecta.](assets/v05-flowchart.png)

## Lab: ubica cada campo de un activo real

Corridas guiadas más un entregable que llenas tú mismo. Vas a correr el script de fetch como se debe, verlo fallar correctamente sobre SPROUT, validar un JSON real, y producir la tabla de ubicación de campos que es el criterio de esta lección. Unos veinticinco minutos.

1. **Fija el workspace.** El mismo workspace de curso de cada lab de TS. Si lo estás recreando desde cero:

   ```bash
   npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
   ```

   Nota de actualidad, verificada contra npm el 2026-09-05: la tag `latest` de kit ahora es 8.2.0, y el workspace del curso se queda fijado en 7.1.1 porque ese es el major de kit contra el que su cliente `@solana-program/token-2022@0.15.0` hace peer (^7; la línea 0.16 se movió a ^8). Los scripts de hoy también usan el `fetch` incorporado, así que Node 20 o más nuevo (el piso del curso desde m01-l1), y `npx tsx` para correr TypeScript directo (se instala solo en la primera llamada; el `package.json` del workspace lleva `"type": "module"` para que funcione el `await` de nivel superior).

2. **Crea `fetch-asset.ts`.** Este es el script provisto, completo. La única maquinaria nueva desde m01-l2 es la derivación del PDA arriba, así que es esa la que se lleva el presupuesto de comentarios:

   ```typescript
   import { createSolanaRpc, address, getProgramDerivedAddress, getAddressEncoder, getAddressDecoder } from '@solana/kit';

   const TOKEN_METADATA_PROGRAM = address('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
   const mint = address(process.argv[2] ?? 'F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk');
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.mainnet-beta.solana.com');

   // 1. Derive the metadata PDA: ["metadata", program id, mint], owned by Token Metadata.
   const addressEncoder = getAddressEncoder();
   const [metadataPda] = await getProgramDerivedAddress({
     programAddress: TOKEN_METADATA_PROGRAM,
     seeds: [
       new TextEncoder().encode('metadata'),
       addressEncoder.encode(TOKEN_METADATA_PROGRAM),
       addressEncoder.encode(mint),
     ],
   });

   // 2. Read the on-chain account and hand-decode the Data struct (Borsh).
   const { value: account } = await rpc.getAccountInfo(metadataPda, { encoding: 'base64' }).send();
   if (!account) throw new Error(`No metadata account at ${metadataPda}. Not a Token Metadata asset.`);

   const data = Buffer.from(account.data[0], 'base64');
   let offset = 1 + 32 + 32; // key (1) + update_authority (32) + mint (32)

   const readString = (): string => {
     const len = data.readUInt32LE(offset);
     offset += 4;
     const raw = data.subarray(offset, offset + len);
     offset += len;
     return raw.toString('utf8').replace(/\0+$/, ''); // strings are stored at fixed capacity, null-padded
   };

   const name = readString();
   const symbol = readString();
   const uri = readString();
   const sellerFeeBasisPoints = data.readUInt16LE(offset);
   offset += 2;

   const addressDecoder = getAddressDecoder();
   const creators: { address: string; verified: boolean; share: number }[] = [];
   if (data[offset] === 1) { // Option<Vec<Creator>> tag
     offset += 1;
     const count = data.readUInt32LE(offset);
     offset += 4;
     for (let i = 0; i < count; i++) {
       creators.push({
         address: addressDecoder.decode(data.subarray(offset, offset + 32)),
         verified: data[offset + 32] === 1,
         share: data[offset + 33],
       });
       offset += 34;
     }
   } else {
     offset += 1;
   }

   console.log('=== ON-CHAIN (the Data struct, decoded from the metadata PDA) ===');
   console.log(`account:                ${metadataPda} (owner: ${account.owner})`);
   console.log(`name:                   ${JSON.stringify(name)}`);
   console.log(`symbol:                 ${JSON.stringify(symbol)}`);
   console.log(`uri:                    ${uri}`);
   console.log(`seller_fee_basis_points: ${sellerFeeBasisPoints}`);
   for (const c of creators) console.log(`creator:                ${c.address} verified=${c.verified} share=${c.share}`);

   // 3. Follow the pointer: fetch the off-chain JSON the uri points at.
   const json = await (await fetch(uri)).json();
   console.log('\n=== OFF-CHAIN (the JSON document at that uri) ===');
   console.log(`name:        ${JSON.stringify(json.name)}`);
   console.log(`symbol:      ${JSON.stringify(json.symbol)}`);
   console.log(`description: ${JSON.stringify(json.description?.slice(0, 60))}...`);
   console.log(`image:       ${json.image}`);
   console.log(`attributes:  ${json.attributes?.length ?? 0} traits, e.g. ${JSON.stringify(json.attributes?.[0])}`);
   console.log(`properties.files: ${json.properties?.files?.length ?? 0} file(s)`);
   console.log(`properties.category: ${json.properties?.category}`);
   console.log(`seller_fee_basis_points in JSON: ${json.seller_fee_basis_points}`);
   console.log(`top-level keys: ${Object.keys(json).join(', ')}`);
   ```

   Córrelo sin argumento para pegarle a Mad Lads #8420. Checkpoint: tu bloque on-chain termina en dos creators con `verified=true` y `seller_fee_basis_points: 420`, y tu bloque off-chain reporta 7 traits y el 500 rancio del JSON. Si la lectura del RPC falla, el endpoint público por defecto limita la tasa de forma agresiva; apunta `RPC_URL` a cualquier endpoint que ya uses y vuelve a correr.

3. **Apúntalo a SPROUT y míralo fallar correctamente.** Corre el script otra vez con la dirección de tu mint SPROUT como argumento. Checkpoint: lanza `No metadata account at ...`. Ese error es la lección: SPROUT no tiene PDA de metadatos de Metaplex porque su identidad vive en la capa 3, dentro del mint. Pruébalo volviendo a correr tu script de lectura de m02-l4 (el del `fetchMint` que afirmó el puntero autorreferencial e imprimió `harvest_season = spring`). Un activo resuelto a través de una segunda cuenta derivada, uno a través de su propio TLV, la misma convención JSON esperando al final de los dos campos `uri`.

4. **Llena la tabla de ubicación de campos.** Este es el entregable. Cópiala a tus notas del curso y completa cada fila con sí/no por columna, usando tus propias dos salidas de sondeo más las secciones de teoría. Las tres primeras filas están hechas como calibración:

   | Campo | cuenta on-chain (struct Data + header) | JSON off-chain | TLV de Token-2022 |
   |---|---|---|---|
   | name | sí | sí (copia por convención) | sí |
   | image | no | sí | no (vía uri) |
   | seller_fee_basis_points | sí (indicativo) | fósil legado | no |
   | symbol | | | |
   | uri | | | |
   | description | | | |
   | attributes / traits | | | |
   | animation_url | | | |
   | external_url | | | |
   | properties.files + category | | | |
   | creators + flag verified | | | |
   | pares additional_metadata | | | |
   | autoridad de actualización | | | |

   Una fila explica el nombre ancho de la columna: la autoridad de actualización vive en los 65 bytes de header de la cuenta que tu decodificador se salteó deliberadamente, no dentro de la struct `Data` de cinco campos. Sigue siendo un hecho on-chain sobre la cuenta, que es por lo que la columna dice cuenta y no struct; responde esa fila para la cuenta como un todo.

5. **Valida un JSON contra el estándar.** Crea `validate-asset-json.ts`, también provisto completo. Como el host canónico del schema se fue, el script ES el schema, codificando el contrato de campos de la capa 1:

   ```typescript
   // validate-asset-json.ts: check an off-chain asset JSON against the Token Metadata
   // JSON standard's field contract. Usage: npx tsx validate-asset-json.ts <uri>
   const CATEGORIES = ['image', 'video', 'audio', 'vr', 'html'];

   const uri = process.argv[2];
   if (!uri) throw new Error('usage: npx tsx validate-asset-json.ts <uri>');

   const json = await (await fetch(uri)).json();
   const failures: string[] = [];
   const warnings: string[] = [];

   // Required by the standard: the fields a wallet cannot render without.
   if (typeof json.name !== 'string' || json.name.length === 0) failures.push('name: missing or empty');
   if (typeof json.description !== 'string') failures.push('description: missing');
   if (typeof json.image !== 'string' || !/^(https?|ipfs|ar):/.test(json.image))
     failures.push('image: missing or not a resolvable URI');

   // Optional but shape-checked when present.
   if (json.symbol !== undefined && typeof json.symbol !== 'string') failures.push('symbol: not a string');
   if (json.animation_url !== undefined && typeof json.animation_url !== 'string')
     failures.push('animation_url: not a string');
   if (json.attributes !== undefined) {
     if (!Array.isArray(json.attributes)) failures.push('attributes: not an array');
     else
       json.attributes.forEach((a: unknown, i: number) => {
         const attr = a as { trait_type?: unknown; value?: unknown };
         if (typeof attr.trait_type !== 'string' || attr.value === undefined)
           failures.push(`attributes[${i}]: needs trait_type (string) and value`);
       });
   }
   if (json.properties?.files !== undefined) {
     if (!Array.isArray(json.properties.files)) failures.push('properties.files: not an array');
     else
       json.properties.files.forEach((f: { uri?: unknown; type?: unknown }, i: number) => {
         if (typeof f.uri !== 'string') failures.push(`properties.files[${i}].uri: missing`);
         if (typeof f.type !== 'string') warnings.push(`properties.files[${i}].type: missing (wallets guess from extension)`);
       });
   }
   if (json.properties?.category !== undefined && !CATEGORIES.includes(json.properties.category))
     warnings.push(`properties.category: "${json.properties.category}" is not one of ${CATEGORIES.join('/')}`);

   // Legacy fields the standard moved on-chain: presence is a staleness signal, not an error.
   if (json.seller_fee_basis_points !== undefined)
     warnings.push(`seller_fee_basis_points: legacy JSON field (${json.seller_fee_basis_points}); the on-chain Data struct's value is what marketplaces read`);
   if (json.collection !== undefined)
     warnings.push('collection: legacy JSON field; collection membership is verified on-chain, never from JSON');

   console.log(`verdict: ${failures.length === 0 ? 'PASS' : 'FAIL'}`);
   for (const f of failures) console.log(`  FAIL  ${f}`);
   for (const w of warnings) console.log(`  warn  ${w}`);
   ```

   Córrelo contra la URI de Mad Lads que imprimió tu fetch. Checkpoint:

   ```text
   verdict: PASS
     warn  seller_fee_basis_points: legacy JSON field (500); the on-chain Data struct's value is what marketplaces read
   ```

   Un pass con una advertencia de fósil, que es exactamente cómo se ve una colección sana de nueve cifras con herramental de la era 2023.

![Espectro de opciones de almacenamiento de uri, desde hosting web alquilado y mutable, pasando por IPFS pineado, hasta Arweave financiado por dotación vía Irys, con la mutabilidad de los metadatos vía autoridad de actualización como una decisión ortogonal.](assets/v06-diagram.png)

## Challenge

Totalmente en solitario, y es el criterio de evaluación de esta lección. Elige un NFT de Solana que yo no elegí para ti: uno que tengas, o cualquier dirección de mint que saques de un listado de marketplace. Corre `fetch-asset.ts` contra él (si lanza el error de cuenta de metadatos ausente, encontraste un activo Core, comprimido, o Token-2022; anota cuál y elige uno legado para este ejercicio. Leer esos otros layouts es para lo que sirven las próximas dos lecciones, más la lección del módulo 7 sobre DAS, la read API del Digital Asset Standard). Después produce dos entregables en tus notas del curso. Primero, tu tabla de ubicación de campos completa del paso 4 del lab, con cada fila ubicada. Segundo, corre `validate-asset-json.ts` sobre la URI de tu activo y escribe un veredicto de tres líneas: pass o fail, la razón de cualquier falla o advertencia en tus propias palabras, y una oración sobre la durabilidad del host de la URI dado dónde se sienta en el espectro de almacenamiento. Si tu activo discrepa consigo mismo entre capas como lo hace Mad Lads, di qué copia gana y por qué. Cuando tu tabla sobreviva una revisión contra las secciones de teoría y tu veredicto nombre el almacenamiento detrás de la URI, tienes el mapa sobre el que construye este módulo.

Dos de tus sondeos de esta lección devolvieron algo que un tutorial no te habría mostrado: una colección emblemática en almacenamiento alquilado y un host canónico de schema muerto. Si tu propio activo del challenge sacó a la luz algo más extraño, o una de mis salidas de sondeo ya no coincide con la tuya, publícalo en el canal de feedback del curso con el comando y la salida pegados. Los metadatos son la capa donde la entropía del ecosistema se muestra primero, y los sondeos de los lectores son cómo esta lección se mantiene verdadera.

Mapeaste dónde viven realmente el nombre, la imagen y los traits de un activo, a través de las tres capas, y puedes ubicar cualquier campo en segundos. La lección que viene dejas de leer activos y empiezas a acuñarlos: Metaplex Core, el camino de NFT recomendado en 2026, una cuenta por activo, y la colección Almanac se crea, se verifica y se hace crecer hasta ser los primeros NFTs de Overgrowth.
