# Restricción de acceso y migraciones de puntos a token: conectar toda la economía

## Resumen

La lección pasada armaste el riel de comisiones de SPROUT. Comisiones de marketplace retenidas a las que les hiciste harvest en las cuentas de los destinatarios donde Token-2022 las había dejado sin hacer ruido, enrutadas a la tesorería, gastadas en un buyback (un swap de DAMM v2 si habías lanzado SPROUT y tenías su pool, si no una compra OTC al maker que levantaste), y quemadas, con el supply del mint cayendo exactamente por lo que quemaste: el buyback más la parte de la quema de comisiones. Dinero que entra, dinero que sale, demostrable en los dos extremos.

Dos cosas en Overgrowth todavía funcionan a pura intuición.

El canal alfa pregunta "¿eres un Founding Farmer?" y hoy le cree a lo que sea que le diga el cliente, lo que quiere decir que el acceso al canal está restringido en el mismo sentido en que una puerta está cerrada con llave cuando la llave está pegada al marco. Y unos cientos de miles de puntos de compost están en una tabla tuya, prometiéndoles a los jugadores un SPROUT que todavía no existe, canjeable bajo términos que no has escrito. Uno de esos es un problema de seguridad. El otro es un problema de supply. Esta lección cierra los dos, y cerrarlos es lo que por fin hace que las cuatro cosas que construiste se comporten como una sola economía y no como cuatro scripts que casualmente comparten una carpeta.

Antes de cualquier teoría, pon la evidencia real en tu pantalla. Toma una billetera que tenga un Harvest crate y una que no, y pregúntale a un índice que no tiene motivo para adular a ninguna de las dos. Primero tres líneas de preparación, porque este es el primer uso de `jq` del curso y el bloque completo de env del lab recién llega en el paso 1:

```bash
brew install jq          # macOS; apt install jq on Debian/Ubuntu, or drop the pipe and read raw JSON
export DAS_RPC_URL=<your DAS endpoint>
export WALLET=<the wallet to probe>   # you will run the query twice, once with each wallet here
```

Después la pregunta en sí:

```bash
curl -s "$DAS_RPC_URL" -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":"gate","method":"getAssetsByOwner",
       "params":{"ownerAddress":"'"$WALLET"'","page":1,"limit":50}}' \
  | jq '.result.items[] | {id, interface, owner: .ownership.owner,
                           collection: (.grouping[]? | select(.group_key=="collection") | .group_value)}'
```

Esa es toda la frontera de confianza en una sola petición. La respuesta es una lista de activos que un indexador público dice que este dueño tiene, y nada de eso vino de la persona que pide acceso. Córrelo contra las dos billeteras. Una imprime un crate bajo tu colección Almanac, la otra imprime una lista vacía o basura de otra persona. Deja las dos terminales abiertas, porque la barrera que vas a escribir convierte exactamente esta salida en un booleano.

Al final vas a tener `gate-and-migrate.ts`: un script que deja pasar por la puerta a un tenedor de Founding-Farmer, le niega la entrada a un desconocido, después convierte los puntos de compost de ese tenedor en SPROUT real mediante un reclamo por merkle, y rechaza el mismo reclamo una segunda vez. Es la última pieza de `sprout-economy`, y es el primer script de este curso que toca cuatro artefactos anteriores a la vez: el mint del módulo 2, los crates del módulo 7, el lector del módulo 7, y la ruta de reclamo del airdrop del módulo 8.

El repliegue, dicho de entrada para que sepas dónde se quitan las rueditas. La barrera está trabajada completa, código y razonamiento, porque la frontera de confianza es la lección y no quiero que la adivines. El cableado del reclamo es un problema de completion: yo te doy el árbol y la transacción, tú escribes las dos comprobaciones que deciden si un reclamo es legítimo. El flujo completo, los dos rieles en una corrida con un segundo reclamo rechazado, corre por tu cuenta.

## La puerta y la ventanilla

La casa club de Overgrowth tiene dos aberturas, y fallan en direcciones opuestas.

Hay una **puerta**, donde alguien afirma ser miembro y tú decides si la abres. Y hay una **ventanilla**, donde alguien entrega un recibo y tú devuelves grano del silo. Una puerta que confía en la evidencia equivocada deja entrar a gente que debería quedarse afuera, lo cual es molesto y recuperable, porque siempre puedes cambiar la cerradura y volver a revisar a todos mañana. Una ventanilla que honra el mismo recibo dos veces entrega grano que nunca estuvo en el silo, y ninguna cantidad de revisiones lo trae de vuelta. El primer error te cuesta exclusividad, el segundo le cuesta a cada tenedor de SPROUT en dilución, y el segundo es el que no puedes deshacer.

Aquí está la ruta por esta lección. Primero la puerta: qué evidencia tiene permitido creer una barrera, y qué te cuesta que esa evidencia sea un índice y no la blockchain misma. Después la ventanilla: por qué un montón de puntos se vuelve un token mediante un reclamo y no mediante una acuñación masiva, qué hashea en realidad la hoja, y dónde vive de verdad la guarda contra un doble reclamo. Después conectas las dos en una sola corrida.

### Qué tiene permitido creer una barrera

El cliente de un usuario puede decirte tres cosas distintas, y solo una de ellas es evidencia.

Puede mandarte un **mensaje firmado** que dice "tengo un Founding-Farmer crate." La firma es real, la criptografía cuadra, y demuestra exactamente una cosa: quien la mandó controla ese keypair. No dice nada sobre lo que el keypair tiene ahora mismo. Una billetera que fue dueña de un crate el martes pasado, lo vendió el miércoles y firma tu mensaje el jueves produce una firma perfectamente válida y una afirmación perfectamente falsa. Las firmas responden "quién eres," nunca "qué tienes."

En código, ese error tiene una forma, y vale la pena poder reconocerla de un vistazo en un pull request:

```typescript
// overgrowth/anti-gate.ts - the shape to recognize and refuse. Do not ship this.
const FOUNDING_FARMER = 'founding-farmer';

interface AccessRequest {
  wallet: string;
  signature: string; // cryptographically valid, and beside the point
  claimsToHold: string; // written by the applicant
}

// A signature check answers "who signed this?", never "what do they hold?".
declare function signatureIsValid(wallet: string, signature: string): boolean;

export function badGate(req: AccessRequest): boolean {
  return signatureIsValid(req.wallet, req.signature) && req.claimsToHold === FOUNDING_FARMER;
}
```

La señal es que el segundo operando salió del cuerpo de la petición. Cada campo de ahí lo escribió la persona que pide que la dejen entrar, y ninguna cantidad de validación de firma convierte esa autoría en evidencia.

Puede mandarte su **lista de tokens en caché**, la que el wallet adapter guarda en el navegador para que tu página de portafolio renderice rápido. Ese caché es una comodidad para el usuario y una sugerencia para ti. Está desactualizado por diseño y cualquiera con devtools abierto lo puede editar. Leerlo no es verificación, es pedirle al solicitante que llene su propia carta de recomendación.

O puedes **leer la tenencia tú mismo**, desde una fuente que el solicitante no controla. Esa es la única que cuenta. Para un saldo fungible puedes ir directo a la blockchain. Para un Harvest crate no puedes, porque un NFT comprimido no tiene cuenta propia: las lecturas de cNFT requieren un RPC con soporte de DAS, que es la restricción con la que te topaste cuando construiste el lector, y es la razón por la que la barrera para un badge de cNFT es una llamada a DAS y no un `getAccountInfo`.

![Tres tarjetas de evidencia comparan un mensaje firmado, una lista de tokens en caché del cliente y una lectura de DAS, y solo la lectura de DAS demuestra la tenencia actual, bajo la advertencia del retraso del indexador.](assets/v01-comparison.png)

Así que la regla es corta. Una barrera decide con una lectura en la que el solicitante no puede escribir. Todo lo demás es un detalle de experiencia de usuario que puedes mostrar en la UI y sobre el que nunca debes ramificar.

### La factura de la frescura

Ahora la parte honesta, y es la razón por la que esta sección existe en vez de una sola línea que diga "usa DAS y ya."

DAS es un índice. Mira la blockchain y anota lo que ve, y anotar lo que ve toma tiempo. Un crate transferido hace cinco segundos todavía puede resolver a su dueño anterior, lo que quiere decir que tu puerta puede admitir a alguien que genuinamente ya no tiene el badge. Eso no es un bug de tu barrera y no es un bug del proveedor. Es lo que un índice es.

Tienes tres formas de pagar esa factura, y cuestan cantidades distintas.

**Acepta el retraso.** Para un rol de Discord o un canal alfa, que un antiguo tenedor conserve el acceso unos segundos o unos minutos no es nada. Vuelve a revisar según un calendario y la ventana se cierra sola. Esta es la respuesta correcta mucho más seguido de lo que los ingenieros quisieran.

**Confirma contra una fuente más fresca.** Para una barrera sobre un saldo fungible, sigue la lectura de DAS con un `getTokenAccountsByOwner` directo en commitment `confirmed` y ramifica sobre ese número en su lugar. Pierdes la simplicidad de una sola llamada, ganas una lectura tan fresca como el cluster. Para un cNFT no hay atajo equivalente, porque el activo genuinamente no tiene cuenta que leer, y la opción honesta es una lectura de prueba que cuesta un viaje de ida y vuelta y aun así resuelve a través del mismo índice.

**Condiciona el acceso a algo que no se pueda mover.** Esta es mi favorita y casi nadie la elige. Si el badge es soulbound, el caso de transferencia que hace peligroso el desfase no existe. Bubblegum v2 ya viene con `set_non_transferable_v2`, así que el Founding-Farmer crate se puede acuñar sin poder salir de la billetera a la que se otorgó, lo cual contradice de plano el folclore de la era 2024 de que los NFT comprimidos no pueden ser soulbound. Ya acuñaste uno así en el módulo 7. Un badge soulbound no hace instantáneo el índice, saca la transferencia del modelo de amenazas, que es un tipo de arreglo distinto y mejor.

![Una línea de tiempo muestra una transferencia de cNFT aterrizando on-chain, el índice de DAS quedándose atrás, y una comprobación de la barrera dentro de ese hueco dejando pasar por error a un antiguo tenedor, con los remedios alineados debajo.](assets/v02-timeline.png)

Hay una cuarta respuesta que la gente elige y quiero nombrarla para que la saltes: hacer streaming del estado tú mismo para tener siempre la vista más fresca. Esa es una técnica real y es un proyecto real. Construir indexadores, plugins de Geyser y pipelines de gRPC es el material del curso planificado Client-Side Mastery, y si tu barrera de verdad necesita frescura por debajo del segundo sobre activos comprimidos, ahí es a donde ir. Para una puerta de miembros, es una plataforma de datos de la que ahora eres dueño para que un desconocido no pueda leer tu canal alfa durante once segundos.

### Los puntos son una promesa, SPROUT es la liquidación

Cambia de abertura. La ventanilla es donde vive la economía, y vale la pena ir despacio, porque esta es la parte que los equipos hacen mal en público.

Los puntos de compost nunca fueron un token. Son un número en una tabla que tú controlas, y valen exactamente lo que tu yo futuro decida que valen. Eso está bien, para eso son los puntos: te dejan premiar comportamiento antes de tener que comprometerte con un supply. Todo programa de puntos de este ecosistema es el mismo canje, lo diga o no. Estás corriendo una promesa, denominada en una unidad que puedes redefinir, hasta el día en que no puedas.

La migración es el día en que no puedes. Es el momento en que el número privado se vuelve público, y se toman tres decisiones, las tomes a propósito o por accidente.

**Quién es elegible.** Un snapshot, tomado en un bloque declarado o un timestamp declarado, publicado para que la gente pueda revisar su propia fila. El snapshot es la parte que hace auditable todo el asunto, y tomarlo en silencio es como una migración se vuelve un escándalo.

**A qué ratio.** En cuántas unidades base se convierte un punto. Esta es toda la decisión de tokenomics, y no hay mecanismo que la decida por ti.

**Con qué calendario.** Todo de una vez, o una porción desbloqueada ahora y el resto liberado con el tiempo.

Mira cómo el ratio hace su trabajo con números redondos. Digamos que Overgrowth tiene 100,000 puntos de compost pendientes entre 4,000 jugadores, y te decides por 10 unidades base de SPROUT por punto. Eso son 1,000,000 de unidades base nuevas, acuñadas al reclamar. Si el supply de SPROUT antes de la migración era 9,000,000, acabas de decidir que los tenedores de puntos se llevan el 10% del token, y que todos los que ya tienen SPROUT son dueños de una porción proporcionalmente más delgada que la de ayer. Nadie fue robado. Nada salió de una billetera. Quienes ya estaban ahí pagaron en dilución, y el ratio es la factura.

Pon el ratio en 1 unidad base por punto y los mismos 100,000 puntos se vuelven 100,000 unidades base, alrededor del 1% del supply, y tus grinders leales se sienten estafados. Ponlo en 100 y tus tenedores actuales se diluyen a la mitad. El mecanismo que estás por construir es idéntico en los tres casos. El mecanismo es gratis. El ratio no.

![Un gráfico de barras agrupadas convierte los mismos 100,000 puntos de compost a tres ratios contra un supply existente fijo de 9,000,000, y les da a los tenedores de puntos entre alrededor de uno y cincuenta y tres por ciento.](assets/v03-chart.png)

Lo cual trae el caso de estudio, y un hueco honesto dentro de él. Kamino corrió una migración de puntos a token hacia KMNO, y es lo obvio a lo que apuntar porque es una de las más grandes que ha hecho este ecosistema. Lo que no pude hacer es verificar la tokenomics de la conversión. Los números que te dejarían decir "convirtieron a X por punto" no están publicados en ningún lado que yo pudiera confirmar, así que no voy a meterte en la cabeza un ratio que no puedo citar. Toma de ahí el mecanismo, una migración con reclamo por merkle desde un libro mayor de puntos off-chain hacia un token on-chain, y toma el ratio de tu propia matemática de supply. Ese es el uso correcto de un caso de estudio cuyos números son privados, y es la misma regla que este curso ha aplicado a cada cifra en disputa: mídela o cítala, nunca partas la diferencia.

### Por qué un reclamo y no una acuñación masiva

Tienes la lista de elegibles. ¿Por qué no acuñarle a todo el mundo y terminar con esto?

Porque quien acuña paga las cuentas. Costeaste exactamente esto cuando construiste el airdrop de compost: una cuenta de token clásica son 293 bytes de rent por destinatario, alrededor de 1,855,569 lamports a la tasa de mainnet del 2026-09-06, así que empujar tokens a 100,000 billeteras son alrededor de 186 SOL antes de haber mandado una sola transacción, y la ruta comprimida bajó eso a alrededor de 1.03 SOL, bastante más del 99% ahorrado. Vuelve a derivar el lado clásico desde la tasa de tu propio cluster antes de citarlo; el lado comprimido no es rent y no se mueve.

Un reclamo cambia quién tiene la factura en la mano. El distributor pone una sola raíz on-chain. Cada destinatario que quiere sus tokens manda su propia transacción, paga el rent de su propia cuenta, y recibe sus propios tokens. Y la cola que nunca reclama nunca te cuesta nada, lo cual importa más de lo que suena: en todo drop grande, una parte significativa de la asignación simplemente nunca se recauda. Bajo un modelo de push pagabas rent para crear cuentas de gente que nunca iba a volver.

![Una comparación de tres columnas entre pushes a cuentas clásicas, pushes a cuentas comprimidas y un reclamo por merkle muestra que el reclamo traslada el costo a los destinatarios y nunca acuña la cola no reclamada.](assets/v04-comparison.png)

Hay una segunda razón, y es por la que el drop de JTO es famoso. Un distributor puede tener dos montos por destinatario: una porción que se desbloquea de inmediato, y una porción que se libera con el tiempo. Jito distribuyó su airdrop mediante un distributor merkle de código abierto con vesting lineal que corrió hasta el 2024-12-07, y el programa que lo hizo, `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv`, sigue siendo la implementación de referencia de este patrón. La instrucción que libera la porción de vesting es `claim_locked`, y ya te topaste con ella en la lección del airdrop. La migración quiere esa división más que un airdrop: un programa de puntos premia a la gente que apareció temprano, y entregarles a todos ellos tokens totalmente líquidos el día uno es una decisión de diseño con un gráfico muy predecible pegado.

### La hoja, byte por byte

Aquí es donde un reclamo por merkle deja de ser una abstracción. Leí el código fuente del programa de referencia en vez de describirlo de memoria, el 2026-08-22, y deberías releerlo antes de apuntar un distributor real a dinero real, porque un repositorio que se quedó callado todavía puede cambiar.

El distributor guarda una sola raíz de 32 bytes. La entrada de un reclamante se hashea dos veces. Primero el reclamo en sí: SHA-256 sobre la dirección de 32 bytes del reclamante, después su monto desbloqueado como u64 little-endian, después su monto bloqueado como u64 little-endian. Después el resultado se vuelve a hashear con un solo byte `0` al frente, el prefijo de hoja. Los nodos internos usan en su lugar un byte `1` al frente, sobre los dos hijos ordenados por su valor de byte, el más bajo primero.

Esos dos bytes de prefijo no son decoración. Sin ellos, una "hoja" de 64 bytes podría fabricarse para parecer un par de nodos internos, y un reclamante podría demostrar la pertenencia de una hoja que nunca estuvo en el árbol. Ese es el ataque de segunda preimagen, y el arreglo es un byte por hash. Aparece en casi toda implementación seria de merkle exactamente por esta razón, y que el arreglo sea así de barato es por lo que no hay excusa para saltárselo.

![Un desglose anotado de la hoja del distributor muestra al reclamante y los montos hasheados en un nodo, prefijos de byte cero y uno en hojas y nodos internos, explicados como protección de segunda preimagen.](assets/v05-annotated-code.png)

La recompensa de saber esto con precisión es que puedes calcular la raíz localmente, en TypeScript, y sacar los mismos 32 bytes que calculará el verificador on-chain. Así es como revisas una distribución antes de publicarla, y como depuras el único reclamo que falla mientras los otros nueve mil funcionan.

### Dónde vive en realidad la guarda contra el doble reclamo

Última pieza de teoría, y es otra vez el problema de la puerta, una abertura más allá.

Una prueba de Merkle demuestra que una entrada está en el árbol. No demuestra que esa entrada no se haya reclamado ya, y nunca podrá, porque la prueba es idéntica cada vez. Algo tiene que acordarse. El distributor de referencia se acuerda creando una cuenta `ClaimStatus` por reclamante, derivada de los seeds `"ClaimStatus"`, la dirección del reclamante y la dirección del distributor, que contiene al reclamante, el monto bloqueado, el monto ya retirado y el monto desbloqueado. La cuenta se crea dentro de la transacción de reclamo. Intenta reclamar dos veces y la segunda transacción falla al crear una cuenta que ya existe.

Fíjate en qué la hace confiable: tu cliente no puede escribirla. La guarda no es un flag en tu script, es un efecto secundario de la misma transacción que mueve los tokens, lo que quiere decir que no puede desincronizarse de los tokens.

Lo cual te dice cuánto vale un libro mayor del lado del cliente. En el lab de hoy vas a mantener un archivo JSON chico de quién ha reclamado, y ese archivo va a impedir correctamente que tu script le pague dos veces a la misma billetera. Es un ensayo, no una frontera. Si la autoridad de mint de verdad es una clave dentro de tu script y lo único entre una billetera y una segunda entrega es un archivo en tu laptop, entonces una segunda entrega está a un archivo perdido de distancia. Dilo en voz alta cuando lo escribas, porque la forma del código se va a parecer tranquilizadoramente a la cosa real.

![Dos flujos comparan un libro mayor JSON del lado del cliente, donde la guarda queda fuera de la transacción de acuñación, con la PDA ClaimStatus on-chain, donde guarda y transferencia ocurren en una sola transacción atómica.](assets/v06-flowchart.png)

### El trade-off, nombrado

Cuatro costos, y ninguno se va por tener cuidado.

Una barrera de DAS solo es tan fresca como el indexador, así que un activo recién transferido todavía puede resolver al dueño viejo y tu puerta puede admitir a un antiguo tenedor por un instante. Confiar en su lugar en una tenencia reportada por el cliente o en una firma pelada no es una versión más barata de esto, es una falla distinta y mucho peor, porque la primera está acotada por una escritura de índice y la segunda no está acotada por nada.

Un reclamo por merkle traslada el costo a tus destinatarios, lo cual es justo cuando quieren los tokens y hostil cuando no saben que existe un reclamo, y agrega una dependencia de programa que no controlas. El repositorio de referencia lleva un rato callado, y el silencio es un riesgo real para código que va a estar sosteniendo una distribución mucho después de que lo despliegues.

El mecanismo de migración es portable, la tokenomics no. Puedes copiar la ruta de reclamo en una tarde y estar seguro de que funciona, porque puedes calcular la raíz tú mismo y revisar cada prueba antes de que alguien reclame. Nadie puede darte el ratio, y ninguna cantidad de leer la migración de otro lo va a producir, que es exactamente el hueco que deja abierto el caso de estudio de Kamino.

Y una guarda de reclamo que vive en tu proceso en vez de en la transacción no es una guarda, es una costumbre que funciona de casualidad hasta la primera vez que dos copias de tu script corren a la vez.

![Un resumen de cuatro filas empareja cada trade-off aceptado con lo que lo acota, desde el retraso del indexador pasando por los reclamos pagados por el destinatario hasta la ventana de carrera en el libro mayor del lado del cliente.](assets/v07-comparison.png)

## Lab: gate-and-migrate.ts

Los dos rieles, una sola corrida. La puerta lee un índice público, así que corre contra devnet, donde de verdad se acuñaron tus crates. La ventanilla acuña SPROUT real, así que corre donde sea que viva tu mint. Si los dos viven en devnet, un solo archivo env cubre todo.

**1. Workspace y versiones fijadas.**

Trabaja en la misma carpeta `overgrowth/` que tiene `das.ts` y `classify.ts` de la lección del lector, porque estás por importar los dos.

```bash
cd overgrowth
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Versiones fijadas verificadas contra npm el 2026-09-05. El tag `latest` de kit es 8.2.0, publicado el 2026-08-29, pero latest no es la regla: un workspace fija la major de kit contra la que hacen peer sus propias deps `@solana-program/*`. Aquí ese cliente es `@solana-program/token-2022@0.15.0`, cuyo rango de peers acepta kit `^7.0.0` — todo de 0.16.0 en adelante hace peer con `^8` — y eso decide el resto: kit 7.1.1, el release más nuevo dentro del rango. Estos clientes entregan cada mes. Corre `npm view @solana-program/token-2022 peerDependencies` antes de confiar en el par.

Después el entorno. Ocho valores, ningún secreto en el repo — el último es una ruta, y el archivo al que apunta es el `treasury.json` que acuñaste en el paso 1b de la lección pasada, porque el `mintTo` de la ventanilla tiene que estar firmado por la autoridad de mint de SPROUT y esa preparación puso la autoridad exactamente en esta clave (si tu SPROUT es anterior a ese paso, vuelve a acuñar según él primero; no hay forma de firmar que esquive una autoridad desechable muerta):

```bash
export DAS_RPC_URL="https://<your-das-provider-endpoint>"
export RPC_URL="https://api.devnet.solana.com"
export WS_URL="wss://api.devnet.solana.com"
export SPROUT_MINT="<your Token-2022 mint from module 2>"
export ALMANAC_COLLECTION="<the Core collection your crates belong to>"
export HOLDER_WALLET="<a wallet holding a Founding-Farmer crate>"
export STRANGER_WALLET="<any wallet that does not>"
export KEYPAIR="<path to labs/m09-l1/treasury.json from m09-l1 step 1b>"
```

Si prefieres correr localmente la mitad de acuñación, surfpool también funciona aquí (1.2.1 en esta máquina, 2026-08-22; en macOS `brew install txtx/taps/surfpool`, si no agarra un binario de release), arrancado con `surfpool start --no-tui --no-studio` y apuntado a `http://127.0.0.1:8899` y `ws://127.0.0.1:8900`. La mitad de la puerta sigue necesitando un endpoint de DAS real, porque un surfnet local no tiene ningún indexador mirándolo.

**2. La puerta.**

Esto está trabajado completo. Lee primero la forma: una regla, una lectura, un resultado que lleva su propio timestamp.

```typescript
// overgrowth/gate.ts - decide access from an indexed on-chain read, never from a client claim.
import { createSolanaRpc, type Address } from '@solana/kit';
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';

export interface OwnedAsset extends DasAsset {
  id: string;
  ownership?: { owner?: string; frozen?: boolean; non_transferable?: boolean };
  grouping?: { group_key: string; group_value: string }[];
  token_info?: {
    balance?: number;
    decimals?: number;
    price_info?: { price_per_token?: number };
  };
}

export type GateRule =
  | { kind: 'collection-badge'; collection: string }
  | { kind: 'token-balance'; mint: string; minimum: bigint };

export interface GateResult {
  owner: string;
  allowed: boolean;
  reason: string;
  evidence: string | null;
  readAt: string;
}

interface OwnerPage {
  total: number;
  limit: number;
  page: number;
  items: OwnedAsset[];
}

export async function ownedAssets(owner: string): Promise<OwnedAsset[]> {
  const out: OwnedAsset[] = [];
  for (let page = 1; ; page += 1) {
    const res = await das<OwnerPage>('getAssetsByOwner', {
      ownerAddress: owner,
      page,
      limit: 1000,
      options: { showFungible: true },
    });
    out.push(...res.items);
    if (res.items.length < res.limit) return out;
  }
}

export async function checkGate(owner: string, rule: GateRule): Promise<GateResult> {
  const readAt = new Date().toISOString();
  const assets = await ownedAssets(owner);

  if (rule.kind === 'collection-badge') {
    for (const asset of assets) {
      if (asset.ownership?.owner !== owner) continue;
      const inCollection = (asset.grouping ?? []).some(
        (g) => g.group_key === 'collection' && g.group_value === rule.collection,
      );
      if (!inCollection) continue;
      const kind = classifyAsset(asset).category;
      if (kind !== 'nft' && kind !== 'compressed-nft') continue;
      return {
        owner,
        allowed: true,
        reason: `holds a ${kind} in collection ${rule.collection}`,
        evidence: asset.id,
        readAt,
      };
    }
    return {
      owner,
      allowed: false,
      reason: `no asset in collection ${rule.collection} resolves to this owner`,
      evidence: null,
      readAt,
    };
  }

  const held = assets.find((a) => a.id === rule.mint && classifyAsset(a).fungible);
  // token_info.balance arrives as a JSON number from most DAS providers. Do
  // not launder it through Math.floor: last lesson's rule stands, a Number
  // above 2^53 lies quietly (the damage is already done at JSON.parse), and
  // above ~1e21 String() turns it into exponent notation. The check below
  // makes that second case loud: a provider that ships a balance too big for
  // a JSON number is a provider you escalate, not round.
  const rawBalance = held?.token_info?.balance;
  const asString = rawBalance === undefined ? "0" : String(rawBalance);
  if (asString.includes("e") || asString.includes("E")) {
    throw new Error(`balance ${asString} exceeds JSON number range: escalate to the provider`);
  }
  const balance = BigInt(asString.split(".")[0] || "0");
  return {
    owner,
    allowed: balance >= rule.minimum,
    reason: `indexed balance ${balance} against minimum ${rule.minimum}`,
    evidence: balance > 0n ? rule.mint : null,
    readAt,
  };
}

export async function confirmBalanceOnChain(
  rpcUrl: string,
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<bigint> {
  const rpc = createSolanaRpc(rpcUrl);
  const { value } = await rpc
    .getTokenAccountsByOwner(owner, { mint }, { encoding: 'jsonParsed', commitment: 'confirmed' })
    .send();
  let total = 0n;
  for (const account of value) {
    if (account.account.owner !== tokenProgram) continue;
    total += BigInt(account.account.data.parsed.info.tokenAmount.amount);
  }
  return total;
}

export function describe(result: GateResult): string {
  const verdict = result.allowed ? 'PASS' : 'DENY';
  return `${verdict}  ${result.owner}  ${result.reason}  (read at ${result.readAt})`;
}
```

Cuatro decisiones de ahí valen sus palabras. La reverificación de `ownership.owner` parece redundante contra una consulta por dueño y no lo es: tarde o temprano le vas a pasar a esta función una lista de activos que sacaste de otro lado, y el día que lo hagas, esa línea es la diferencia entre una barrera y una sugerencia. `classifyAsset` está haciendo trabajo real y no decoración, porque es lo que impide que una posición fungible en la misma colección satisfaga una regla de badge. `readAt` existe para que cuando alguien se queje de que le negaron la entrada, puedas responder con un timestamp en vez de un encogimiento de hombros. Y `confirmBalanceOnChain` es el remedio de frescura, deliberadamente separado, deliberadamente no llamado por defecto. Préndelo para la barrera que protege algo caro, déjalo apagado para un rol de chat.

![Un diagrama de flujo traza checkGate desde una dirección de dueño, pasando por una lectura paginada de DAS y tres comprobaciones secuenciales, hasta salir a una aprobación con evidencia o a una negación, con cada resultado marcado con timestamp.](assets/v08-flowchart.png)

**3. Corre la puerta.**

```bash
npx tsx -e "import {checkGate,describe} from './gate'; \
  const rule={kind:'collection-badge',collection:process.env.ALMANAC_COLLECTION!} as const; \
  for (const w of [process.env.HOLDER_WALLET!, process.env.STRANGER_WALLET!]) \
    console.log(describe(await checkGate(w, rule)));"
```

Quieres dos líneas, una de cada veredicto:

```
PASS  7xK…9fQ  holds a compressed-nft in collection 4vT…2mL  (read at 2026-08-22T14:07:11.402Z)
DENY  3nB…kW1  no asset in collection 4vT…2mL resolves to this owner  (read at 2026-08-22T14:07:12.118Z)
```

Si las dos líneas dicen DENY, revisa el valor de la colección antes que cualquier otra cosa. La colección de un cNFT vive en `grouping` con un `group_key` de `collection`, y el valor es la dirección del activo de colección, no su nombre.

**4. El árbol.**

Ahora la ventanilla. Este archivo se te da completo, porque tiene que ser compatible byte a byte con lo que calcula un verificador on-chain y no hay crédito parcial por una raíz que casi está bien. Y un reconocimiento que se te debe, porque construiste exactamente este árbol hace dos lecciones en `compost-airdrop` con otros nombres: `leafHash` allá es `hashLeaf` aquí, `tree.proofFor` se vuelve `getProof` sobre niveles explícitos, y los bytes que se hashean son idénticos, prefijo de hoja, prefijo intermedio, pares ordenados y todo. Esta copia es deliberadamente autocontenida para que `overgrowth/` no cargue ningún import entre carpetas que se pueda romper.

Si dudas de que los dos coincidan, haz diff de lo correcto. Hashear una entrada por `leafHash` y por `hashLeaf` no demuestra casi nada: las hojas se hashean idénticamente por construcción, así que esa comprobación pasa incluso cuando los dos constructores no coinciden. El lugar donde dos ports de merkle de verdad divergen es la regla de combinación de niveles, y solo aparece con un ancho de nivel IMPAR, que una lista de prueba de cuatro u ocho entradas nunca produce. Así que construye la misma lista de TRES entradas por los dos y haz diff de las raíces. Que las raíces coincidan con n=3 es evidencia; que los hashes de hoja coincidan es una tautología.

```typescript
// overgrowth/merkle.ts - the distributor's tree, byte for byte.
// Ported from jito-foundation/distributor (merkle-tree/src/merkle_tree.rs,
// programs/merkle-distributor/src/instructions/new_claim.rs, verify/src/lib.rs),
// read on 2026-08-22. Re-read before you trust this against a live distributor.
import { createHash } from 'node:crypto';
import { getAddressEncoder, type Address } from '@solana/kit';

const LEAF_PREFIX = Uint8Array.from([0]);
const INTERMEDIATE_PREFIX = Uint8Array.from([1]);
const addressEncoder = getAddressEncoder();

export interface ClaimEntry {
  claimant: Address;
  unlocked: bigint;
  locked: bigint;
}

function sha256(...parts: Uint8Array[]): Buffer {
  const hash = createHash('sha256');
  for (const part of parts) hash.update(part);
  return hash.digest();
}

function u64le(value: bigint): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(value);
  return buf;
}

export function hashLeaf(entry: ClaimEntry): Buffer {
  const claimant = new Uint8Array(addressEncoder.encode(entry.claimant));
  const node = sha256(claimant, u64le(entry.unlocked), u64le(entry.locked));
  return sha256(LEAF_PREFIX, node);
}

function hashPair(left: Buffer, right: Buffer): Buffer {
  return Buffer.compare(left, right) <= 0
    ? sha256(INTERMEDIATE_PREFIX, left, right)
    : sha256(INTERMEDIATE_PREFIX, right, left);
}

export function buildTree(entries: ClaimEntry[]): Buffer[][] {
  if (entries.length === 0) throw new Error('empty distribution: nothing to migrate');
  const levels: Buffer[][] = [entries.map(hashLeaf)];
  while (levels[levels.length - 1].length > 1) {
    const below = levels[levels.length - 1];
    const above: Buffer[] = [];
    for (let i = 0; i < below.length; i += 2) {
      const left = below[i];
      const right = i + 1 < below.length ? below[i + 1] : below[i];
      above.push(hashPair(left, right));
    }
    levels.push(above);
  }
  return levels;
}

export function getRoot(levels: Buffer[][]): Buffer {
  return levels[levels.length - 1][0];
}

export function getProof(levels: Buffer[][], index: number): Buffer[] {
  if (index < 0 || index >= levels[0].length) throw new Error(`no leaf at index ${index}`);
  const proof: Buffer[] = [];
  let idx = index;
  for (let level = 0; level < levels.length - 1; level += 1) {
    const nodes = levels[level];
    const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
    proof.push(siblingIdx < nodes.length ? nodes[siblingIdx] : nodes[idx]);
    idx = Math.floor(idx / 2);
  }
  return proof;
}

export function verifyProof(leaf: Buffer, proof: Buffer[], root: Buffer): boolean {
  let computed = leaf;
  for (const sibling of proof) computed = hashPair(computed, sibling);
  return computed.equals(root);
}
```

Dos detalles para notar, porque los dos son lugares donde un árbol hecho a mano sale mal. Un nodo impar en un nivel se empareja **consigo mismo**, no se promueve al nivel de arriba, que es lo que hace el constructor de referencia y lo que asumen sus pruebas. Y cada par se ordena antes de hashear, que es por lo que `verifyProof` puede plegar una prueba sin que le digan si cada hermano era hijo izquierdo o derecho.

Antes de construir nada encima, demuéstratelo a ti mismo. Tres entradas, tres pruebas, un monto adulterado:

```typescript
// overgrowth/tree-check.ts - trust the tree only after you have tried to break it.
// Run from inside overgrowth/: npx tsx tree-check.ts
import { address } from '@solana/kit';
import { buildTree, getProof, getRoot, hashLeaf, verifyProof, type ClaimEntry } from './merkle';

const entries: ClaimEntry[] = [
  { claimant: address('11111111111111111111111111111112'), unlocked: 100n, locked: 0n },
  { claimant: address('So11111111111111111111111111111111111111112'), unlocked: 250n, locked: 50n },
  { claimant: address('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'), unlocked: 7n, locked: 3n },
];

const levels = buildTree(entries);
const root = getRoot(levels);
console.log('root', root.toString('hex'));

entries.forEach((entry, i) => {
  const ok = verifyProof(hashLeaf(entry), getProof(levels, i), root);
  console.log(i, ok ? 'PROOF OK' : 'PROOF FAILED');
});

const greedy = { ...entries[0], unlocked: 1000n };
console.log('tampered accepted?', verifyProof(hashLeaf(greedy), getProof(levels, 0), root));
```

Tres líneas `PROOF OK` y un `false`. La última línea es la que importa: cambia el monto y la hoja cambia, así que la misma prueba ya no se pliega a la misma raíz. Esa es toda la propiedad de seguridad de una distribución, demostrada en cuatro líneas.

**5. El reclamo, y las dos comprobaciones que escribes tú.**

Aquí está el problema de completion. El archivo de abajo está completo salvo por las dos decisiones que deciden si un reclamo es legítimo. Tápalas, escríbelas tú mismo a partir de la descripción, después compara.

La primera: después de reconstruir el árbol y tomar la prueba, niégate a continuar a menos que la prueba se pliegue a la raíz que estás por publicar. La segunda: niégate a continuar si este reclamante ya reclamó. Las dos son tres líneas. Las dos son todo el trabajo.

```typescript
// overgrowth/claim.ts - migrate compost points into SPROUT through the merkle path.
import { readFile, writeFile } from 'node:fs/promises';
import {
  address,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getAddressEncoder,
  getProgramDerivedAddress,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type TransactionSigner,
} from '@solana/kit';
import {
  fetchMint,
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getMintToInstruction,
  TOKEN_2022_PROGRAM_ADDRESS,
} from '@solana-program/token-2022';
import { buildTree, getProof, getRoot, hashLeaf, verifyProof, type ClaimEntry } from './merkle';

/** The reference claim program: Jito's merkle distributor, the one the JTO drop used. */
export const MERKLE_DISTRIBUTOR_PROGRAM = address('mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv');

/** A leaf is claimable once. Where that fact is stored is the whole security question. */
export interface ClaimLedger {
  isClaimed(claimant: Address): Promise<boolean>;
  markClaimed(claimant: Address): Promise<void>;
}

/** Rehearsal ledger: good enough to stop YOUR script running twice, and nothing more. */
export class FileClaimLedger implements ClaimLedger {
  constructor(private readonly path: string) {}

  private async load(): Promise<string[]> {
    try {
      return JSON.parse(await readFile(this.path, 'utf8')) as string[];
    } catch {
      return [];
    }
  }

  async isClaimed(claimant: Address): Promise<boolean> {
    return (await this.load()).includes(claimant);
  }

  async markClaimed(claimant: Address): Promise<void> {
    const claimed = await this.load();
    if (!claimed.includes(claimant)) claimed.push(claimant);
    await writeFile(this.path, JSON.stringify(claimed, null, 2));
  }
}

/** The real boundary: the distributor's per-claimant ClaimStatus account. */
export async function claimStatusAddress(
  distributor: Address,
  claimant: Address,
): Promise<Address> {
  const encoder = getAddressEncoder();
  const [pda] = await getProgramDerivedAddress({
    programAddress: MERKLE_DISTRIBUTOR_PROGRAM,
    seeds: [
      new TextEncoder().encode('ClaimStatus'),
      encoder.encode(claimant),
      encoder.encode(distributor),
    ],
  });
  return pda;
}

export class OnChainClaimLedger implements ClaimLedger {
  constructor(
    private readonly rpc: ReturnType<typeof createSolanaRpc>,
    private readonly distributor: Address,
  ) {}

  async isClaimed(claimant: Address): Promise<boolean> {
    const pda = await claimStatusAddress(this.distributor, claimant);
    const { value } = await this.rpc.getAccountInfo(pda, { encoding: 'base64' }).send();
    return value !== null;
  }

  async markClaimed(): Promise<void> {
    // No-op on purpose: the distributor program creates ClaimStatus inside the claim
    // transaction. A client cannot mark this, which is exactly why it is trustworthy.
  }
}

export interface MigrationResult {
  claimant: Address;
  minted: bigint;
  stillLocked: bigint;
  root: string;
  signature: string;
  supplyBefore: bigint;
  supplyAfter: bigint;
}

export interface MigrationInput {
  rpcUrl: string;
  wsUrl: string;
  entries: ClaimEntry[];
  index: number;
  mint: Address;
  mintAuthority: TransactionSigner;
  payer: TransactionSigner;
  ledger: ClaimLedger;
}

export async function migrateClaim(input: MigrationInput): Promise<MigrationResult> {
  const { entries, index, mint, mintAuthority, payer, ledger } = input;
  const entry = entries[index];
  if (!entry) throw new Error(`no distribution entry at index ${index}`);

  const levels = buildTree(entries);
  const root = getRoot(levels);
  const proof = getProof(levels, index);

  // CHECK ONE: the proof must fold to this root, or the snapshot and the tree disagree.
  if (!verifyProof(hashLeaf(entry), proof, root)) {
    throw new Error('proof does not reproduce the root: your snapshot and your tree disagree');
  }

  // CHECK TWO: one leaf, one claim.
  if (await ledger.isClaimed(entry.claimant)) {
    throw new Error(`${entry.claimant} already claimed this distribution`);
  }

  const rpc = createSolanaRpc(input.rpcUrl);
  const rpcSubscriptions = createSolanaRpcSubscriptions(input.wsUrl);

  const before = await fetchMint(rpc, mint);
  const [ata] = await findAssociatedTokenPda({
    owner: entry.claimant,
    mint,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const createAta = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer,
    owner: entry.claimant,
    mint,
  });
  const mintTo = getMintToInstruction({
    mint,
    token: ata,
    mintAuthority,
    amount: entry.unlocked,
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions([createAta, mintTo], m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(signed, {
    commitment: 'confirmed',
  });
  await ledger.markClaimed(entry.claimant);

  const after = await fetchMint(rpc, mint);
  return {
    claimant: entry.claimant,
    minted: entry.unlocked,
    stillLocked: entry.locked,
    root: root.toString('hex'),
    signature: getSignatureFromTransaction(signed),
    supplyBefore: before.data.supply,
    supplyAfter: after.data.supply,
  };
}

/** Points to base units. The MECHANISM is reusable; this ratio is a tokenomics decision. */
export function pointsToSprout(
  points: bigint,
  perPoint: bigint,
  lockedBps: number,
): { unlocked: bigint; locked: bigint } {
  const total = points * perPoint;
  const locked = (total * BigInt(lockedBps)) / 10000n;
  return { unlocked: total - locked, locked };
}
```

Dos notas sobre lo que esto deliberadamente no hace. Acuña solo la porción desbloqueada y reporta el resto bloqueado, porque liberar la porción bloqueada con el tiempo es la ruta `claim_locked` del distributor y no voy a fingir un calendario de vesting en un script de cliente. Y `OnChainClaimLedger` deriva la dirección de la guarda sin afirmar que maneja el programa: te muestra dónde vive la respuesta y cómo pedirla, que es la parte que te llevas a una migración de producción.

**6. El flujo.**

Tu snapshot de puntos, `compost-points.json`, lo que publicarías para que los jugadores puedan revisar su propia fila:

```json
[
  { "wallet": "7xK...", "compostPoints": 4200 },
  { "wallet": "3nB...", "compostPoints": 150 }
]
```

Reemplaza las dos billeteras de relleno por tus direcciones reales antes de correr nada: la primera fila es `$HOLDER_WALLET`, la segunda `$STRANGER_WALLET`. Si las dejas tal cual, `loadEntries()` le mete `7xK...` directo al `address()` de kit, que lanza un error de parseo críptico mucho antes de que la guarda más amable de no-hay-puntos-que-migrar tenga oportunidad de dispararse.

Y el script de nivel superior, que es el artefacto:

```typescript
// overgrowth/gate-and-migrate.ts - the Overgrowth economy, end to end.
// Run from inside overgrowth/: npx tsx gate-and-migrate.ts
import { readFile } from 'node:fs/promises';
import { address, createKeyPairSignerFromBytes } from '@solana/kit';
import { checkGate, describe } from './gate';
import { FileClaimLedger, migrateClaim, pointsToSprout } from './claim';
import type { ClaimEntry } from './merkle';

interface PointsRow {
  wallet: string;
  compostPoints: number;
}

const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
const WS_URL = process.env.WS_URL ?? 'wss://api.devnet.solana.com';
const SPROUT_MINT = address(process.env.SPROUT_MINT ?? '');
const ALMANAC_COLLECTION = process.env.ALMANAC_COLLECTION ?? '';
const HOLDER = process.env.HOLDER_WALLET ?? '';
const STRANGER = process.env.STRANGER_WALLET ?? '';

/** 1 compost point becomes 10 SPROUT base units; a quarter of the grant vests. */
const SPROUT_PER_POINT = 10n;
const LOCKED_BPS = 2500;

async function loadEntries(path: string): Promise<ClaimEntry[]> {
  const rows = JSON.parse(await readFile(path, 'utf8')) as PointsRow[];
  return rows.map((row) => {
    const { unlocked, locked } = pointsToSprout(
      BigInt(row.compostPoints),
      SPROUT_PER_POINT,
      LOCKED_BPS,
    );
    return { claimant: address(row.wallet), unlocked, locked };
  });
}

async function main(): Promise<void> {
  const signerBytes = new Uint8Array(
    JSON.parse(await readFile(process.env.KEYPAIR ?? 'treasury.json', 'utf8')) as number[],
  );
  const authority = await createKeyPairSignerFromBytes(signerBytes);

  console.log('--- the door ---');
  const rule = { kind: 'collection-badge', collection: ALMANAC_COLLECTION } as const;
  for (const wallet of [HOLDER, STRANGER]) {
    console.log(describe(await checkGate(wallet, rule)));
  }

  console.log('\n--- the window ---');
  const entries = await loadEntries('compost-points.json');
  const index = entries.findIndex((e) => e.claimant === HOLDER);
  if (index < 0) throw new Error(`${HOLDER} has no compost points to migrate`);
  const ledger = new FileClaimLedger('claimed.json');

  const result = await migrateClaim({
    rpcUrl: RPC_URL,
    wsUrl: WS_URL,
    entries,
    index,
    mint: SPROUT_MINT,
    mintAuthority: authority,
    payer: authority,
    ledger,
  });
  console.log(`root      ${result.root}`);
  console.log(`minted    ${result.minted} base units to ${result.claimant}`);
  console.log(`locked    ${result.stillLocked} (claim_locked releases this linearly)`);
  console.log(`supply    ${result.supplyBefore} -> ${result.supplyAfter}`);
  console.log(`delta ok  ${result.supplyAfter - result.supplyBefore === result.minted}`);

  console.log('\n--- the same leaf, twice ---');
  try {
    await migrateClaim({
      rpcUrl: RPC_URL,
      wsUrl: WS_URL,
      entries,
      index,
      mint: SPROUT_MINT,
      mintAuthority: authority,
      payer: authority,
      ledger,
    });
    console.log('DOUBLE CLAIM LANDED - your guard is not a guard');
  } catch (err: unknown) {
    console.log(`rejected: ${err instanceof Error ? err.message : String(err)}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

**7. Córrelo.**

```bash
npx tsx gate-and-migrate.ts
```

La forma de una buena corrida, con un tenedor con 4,200 puntos de compost a diez unidades base por punto y un cuarto bloqueado:

```
--- the door ---
PASS  7xK…9fQ  holds a compressed-nft in collection 4vT…2mL  (read at 2026-08-22T14:22:03.771Z)
DENY  3nB…kW1  no asset in collection 4vT…2mL resolves to this owner  (read at 2026-08-22T14:22:04.410Z)

--- the window ---
root      3d8e48de…ac736403   (yours will differ: the root is a function of your entry list)
minted    31500 base units to 7xK…9fQ
locked    10500 (claim_locked releases this linearly)
supply    9000000 -> 9031500   (illustrative: your mint's real before/after appear here)
delta ok  true

--- the same leaf, twice ---
rejected: 7xK…9fQ already claimed this distribution
```

Lee las últimas cuatro líneas como un conjunto. El delta de supply iguala exactamente el monto acuñado, así que no se fugó nada. El resto bloqueado se declara en vez de acuñarse, así que tu gráfico de supply coincide con tu promesa. Y el segundo reclamo lo rechazó una comprobación que corrió antes de que se construyera ninguna transacción, que es donde corresponden los rechazos.

![Cinco artefactos previos convergen en gate-and-migrate.ts, cuyos dos carriles internos emiten veredictos de la barrera, SPROUT acuñado con un delta de supply, y un segundo reclamo rechazado.](assets/v09-diagram.png)

## Challenge

Conecta todo tú mismo, en una sola corrida, y haz que produzca tres artefactos de evidencia.

Primero, un resultado de barrera por billetera, los dos veredictos, cada uno desde una lectura de DAS. Segundo, un monto migrado cuyo delta de supply coincide hasta la unidad base, mediante la ruta de merkle y no una acuñación pelada. Sé preciso sobre lo que exige esa frase, porque tu `migrateClaim` ES un `getMintToInstruction` y eso está bien: el requisito es que la acuñación se dispare solo después de que pasen tu verificación de prueba y tu comprobación del libro mayor de reclamos, para que una hoja adulterada o repetida nunca llegue a ella. Estás demostrando que la barrera que está DELANTE de la acuñación es estructural, no que manejaste el distributor real, que es lo que esta lección explícitamente se negó a hacer. Tercero, un segundo reclamo rechazado de la misma hoja.

Después empújalo, porque la parte interesante no es el camino feliz.

Cambia el monto de un destinatario en `compost-points.json` después de construir el árbol y antes de reclamar, y mira cómo la prueba deja de plegarse a la raíz. Ese es un reclamante intentando pagarse más a sí mismo, y es la falla que el hash de hoja existe para producir.

Deriva la dirección de la guarda para tu tenedor y ve a mirarla:

```bash
npx tsx -e "import {address} from '@solana/kit'; import {claimStatusAddress} from './claim'; \
  console.log(await claimStatusAddress(address(process.env.DISTRIBUTOR!), address(process.env.HOLDER_WALLET!)));"
```

Cambia `FileClaimLedger` por `OnChainClaimLedger`, apúntalo a cualquier dirección de distributor, y lee lo que vuelve. Va a decir "no reclamado," porque esa cuenta `ClaimStatus` no existe para un distributor que nunca creaste. No puedes hacer que diga "reclamado" desde un cliente, y esa incapacidad es la propiedad que en realidad estás comprando.

Y decide tu propio ratio antes de mirar el mío. Toma tu supply real de SPROUT, toma el total de puntos de compost pendientes, y escribe con qué porcentaje del token deberían terminar los tenedores de puntos. Después trabaja hacia atrás hasta el número por punto. Si ese porcentaje te incomoda, acabas de descubrir por qué los anuncios de migración son los posts más tensos que escriben estos equipos.

## Checkpoint

El criterio de esta lección: `npx tsx gate-and-migrate.ts` deja pasar a un tenedor de Founding-Farmer, le niega la entrada a una billetera sin uno, los dos desde una lectura de DAS, acuña la cantidad correcta de SPROUT mediante la ruta de merkle con un delta de supply que coincide, y rechaza el segundo reclamo de esa hoja.

Los errores que espero, más o menos en el orden en que aparecen. Que toda llamada a DAS falle con un error de método no encontrado quiere decir que tu `DAS_RPC_URL` es un RPC común, y un activo comprimido simplemente no se puede leer desde uno. Un tenedor al que le siguen negando la entrada suele significar que el valor de `grouping` contra el que comparaste es el nombre de la colección y no su dirección. Un delta de supply que no coincide con el monto acuñado quiere decir que leíste el mint antes y después en commitments distintos, o que acuñaste el total en vez de la porción desbloqueada. Y un segundo reclamo que aterriza quiere decir que tu ruta del libro mayor nunca corrió, que en un libro mayor del lado del cliente es un bug de una línea y en producción es una cuenta faltante.

Una cosa que escribir que no es código. En tu README, una frase por riel: qué lee tu barrera, y cuál es el ratio de tu migración. "La barrera alfa resuelve un Founding-Farmer crate mediante DAS y acepta hasta N segundos de retraso del índice." "Un punto de compost se convierte en X unidades base, un total de Y por ciento del supply, con Z por ciento en vesting." Dentro de seis meses la primera frase le dice a un ingeniero de guardia si un ticket de soporte es un bug o es física, y la segunda es la frase por la que te van a citar. He visto equipos escribir el código con cuidado y dejar esas dos frases implícitas, y la versión implícita es la que se redescubre en público durante un incidente.

Da un paso atrás y mira lo que corre ahora. SPROUT existe con extensiones que elegiste a propósito. Los crates existen, uno de ellos atado permanentemente a la billetera que se lo ganó. Un solo lector lee todo. Las comisiones fluyen a una tesorería, recompran y queman. Y una promesa que tenías guardada en una base de datos ahora es un token en la billetera de alguien, acuñado solo cuando lo pidió, y acuñable exactamente una vez. Eso es una economía, y cada pieza de ella es algo a lo que puedes apuntarle un script y revisarla.

La próxima lección quita los rieles y mira tokens que de verdad se entregaron. PYUSD y JTO, en producción, a escala. Lo más interesante de ellos no es lo que hacen. Es lo que arman y nunca disparan: extensiones configuradas con autoridades puestas y parámetros en cero, ahí dormidas, esperando una decisión que nadie ha tomado todavía. Una vez que has construido tú mismo todo esto, esa contención deja de parecer indecisión y empieza a parecer un diseño.
