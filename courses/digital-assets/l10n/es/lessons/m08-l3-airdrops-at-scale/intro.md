# Airdrops a escala: ingeniería de costos y reclamos con vesting

## Resumen

En las últimas dos lecciones derivaste lo que hace falta para poner SPROUT en una curva, probaste qué plataformas pueden siquiera sostener su mint, y escribiste la decisión que elige dónde se va a graduar. El módulo 7 hizo algo distinto: enseñó la compresión como concepto, te hizo ponerle precio, y te dejó con una promesa en la mano. Razonaste sobre tokens comprimidos durante una lección entera sin tocar ninguno.

Hoy tocas uno, en los primeros diez minutos, y después pasas el resto de la lección en la aritmética que decide cómo un token llega a cien mil desconocidos.

Empieza por el número, antes de cualquier explicación:

```bash
# 293 bytes (165 of data + the 128-byte account header) at your cluster's
# rent rate. Ask for the rate instead of pasting one; mine is mainnet's, read
# 2026-09-06, and SIMD-0437 is stepping it down.
RATE=$(curl -s https://api.mainnet-beta.solana.com -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getMinimumBalanceForRentExemption","params":[0]}' \
  | node -e "let d='';process.stdin.on('data',c=>d+=c).on('end',()=>console.log(JSON.parse(d).result/128))")
node -e "const classic=293*$RATE, compressed=10_300, n=100_000; console.log('rate', $RATE, 'lamports/byte | classic', (classic*n/1e9).toFixed(2), 'SOL / compressed', (compressed*n/1e9).toFixed(2), 'SOL /', ((1-compressed/classic)*100).toFixed(1)+'% saved')"
```

```
rate 6333 lamports/byte | classic 185.56 SOL / compressed 1.03 SOL / 99.4% saved
```

Casi doscientos SOL contra uno. Es la misma distribución, a las mismas cien mil billeteras, con precio puesto de dos maneras. Y la razón por la que esta lección existe es que ninguno de los dos números es la factura completa, los dos esconden una decisión de alcance, y el método que de verdad entregas para SPROUT es un tercero que ninguno de los dos describe.

Vas a construir el compost-airdrop: una tabla de costos que calcula los lamports por destinatario en cuatro métodos de distribución y coincide con las cifras canónicas, más una ruta de reclamo merkle que prueba un reclamo desbloqueado y un reclamo `claim_locked` con vesting lineal. El repliegue, dicho de entrada: el modelo de costos se recorre contigo línea por línea, las cifras comprimidas son un problema de completion que rellenas tú mismo, y la segunda ruta de reclamo, la bloqueada, es enteramente tuya.

## La factura de la distribución

### Diez minutos de primer contacto

Antes de cualquier teoría, consigue un token comprimido en tus propias manos. Esto es un mandado en devnet, no una construcción, y existe para que cada número posterior de la lección se ate a algo que corriste.

El SDK de Light es un stack de web3.js v1. Su rango de peers publicado es `@solana/web3.js >=1.73.5`, y Light no publica ninguna superficie de kit, así que el viaje en v1 es inevitable y no una elección: este lab es un workspace v1 en cuarentena, a propósito, escribes modismos de v1 solo donde el SDK del proveedor te obliga, y el resto del curso se queda en kit. (El `latest` de kit en npm era 8.0.0, publicado el 2026-08-21, cuando se escribió esta lección. No pongas `latest` en un package.json, y vuelve a revisar cada pin de abajo antes de confiar en él.)

```bash
mkdir -p labs/m08-l3 && cd labs/m08-l3
npm init -y
npm pkg set type=module
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
npm install @solana/web3.js@1.98.4 @solana/spl-token@0.4.15 \
  @lightprotocol/stateless.js@0.23.3 @lightprotocol/compressed-token@0.23.3
mkdir -p compost-airdrop
```

`tsx` corre un archivo TypeScript directamente. `@lightprotocol/stateless.js` habla con el programa de sistema de Light y con un indexador Photon; `@lightprotocol/compressed-token` es la capa de token encima. Los dos estaban en 0.23.3 al momento de escribir esto. La línea `npm pkg set type=module` no es opcional: los dos paquetes de Light ya vienen como módulos ES, y sin ella cada import del calentamiento falla en el typecheck con una queja de `require`.

Necesitas un endpoint de devnet que sirva la API de compresión, porque una cuenta comprimida no se puede leer con `getAccountInfo`. Sirve cualquier proveedor con soporte de ZK Compression; Helius publica un tier gratis que la habla, que es el que usé yo.

```typescript
// compost-airdrop/warmup.ts
import { createRpc } from "@lightprotocol/stateless.js";
import { compress, createMint } from "@lightprotocol/compressed-token";
import {
  createAssociatedTokenAccount,
  mintTo as splMintTo,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const RPC_URL = process.env.DEVNET_RPC;
if (!RPC_URL) throw new Error("set DEVNET_RPC to a devnet endpoint with ZK Compression support");

const payer = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(readFileSync(process.env.KEYPAIR ?? "", "utf8"))),
);

// One URL, three roles: Solana RPC, Photon compression API, prover.
const rpc = createRpc(RPC_URL, RPC_URL, RPC_URL);

async function main(): Promise<void> {
  const { mint } = await createMint(rpc, payer, payer.publicKey, 9);
  console.log(`mint ${mint.toBase58()}`);

  const ata = await createAssociatedTokenAccount(rpc, payer, mint, payer.publicKey);
  await splMintTo(rpc, payer, mint, ata, payer, 1_000_000_000);
  console.log(`classic ATA ${ata.toBase58()} holds 1 token`);

  const signature = await compress(
    rpc,
    payer,
    mint,
    1_000_000_000,
    payer,
    ata,
    payer.publicKey,
  );
  console.log(`compressed in ${signature}`);

  // getAccountInfo would return nothing here. This read goes through Photon.
  const accounts = await rpc.getCompressedTokenAccountsByOwner(payer.publicKey, { mint });
  for (const account of accounts.items) {
    console.log(
      `compressed account: amount ${account.parsed.amount.toString()} ` +
        `owner ${account.parsed.owner.toBase58()} ` +
        `leafIndex ${account.compressedAccount.leafIndex}`,
    );
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
```

Córrelo con tu keypair de devnet y un saldo fondeado:

```bash
DEVNET_RPC="<your devnet rpc>" KEYPAIR="$HOME/.config/solana/id.json" npx tsx compost-airdrop/warmup.ts
```

Deberías ver una dirección de mint, una ATA, una firma de compresión, y después una línea que describe una **cuenta de token comprimida**: un saldo que vive como una hoja hasheada en un árbol de estado, con el ledger guardando su contenido y un indexador reconstruyéndolo a pedido. No hay cuenta en esa dirección. El `leafIndex` de la salida es la señal honesta: lo que posees es una posición en un árbol, no un slot en la base de datos de cuentas.

Fíjate en la forma de esa última lectura. Le preguntaste a Photon, no al validador. Ese es el impuesto de lectura que el módulo 7 te cobró sobre los cNFT, ahora cobrado sobre un saldo. Y como es una lectura de índice, puede ir atrasada respecto al ledger: si la lista de cuentas comprimidas vuelve vacía en una corrida cuya firma de compresión se imprimió bien, no falló nada; Photon todavía no se puso al día. Espera unos segundos y vuelve a correr la lectura antes de sospechar de la compresión.

Esos son tus diez minutos. Ahora el dinero.

### Cuánto cuesta un destinatario

Dos números hacen todo el trabajo aquí, y los dos son por destinatario.

Una cuenta de token SPL clásica son 165 bytes de datos más el header de cuenta de 128 bytes que agrega el runtime, o sea 293 bytes, y la exención de renta le pone precio a cada uno de esos bytes a una tasa por byte. Los bytes están fijos por el layout de la cuenta. La tasa no: es un parámetro de red, el SIMD-0437 la está bajando por etapas, y los clusters no van al mismo paso. Así que mídela en vez de confiar en mí, y mídela en el cluster en el que vas a pagar:

```bash
solana rent 165 --url mainnet-beta
solana rent 165 --url devnet
```

```
Rent-exempt minimum: 0.001855569 SOL
Rent-exempt minimum: 0.00148844 SOL
```

Esas son mis lecturas del 2026-09-06: 1,855,569 lamports en mainnet, que es exactamente 293 por 6,333, y 1,488,440 en devnet, exactamente 293 por 5,080. Antes de que el primer escalón del recorte cayera en mainnet el 2026-09-03 la tasa en todas partes era 6,960 y este número era 2,039,280, que es lo que todavía vas a encontrar en la mayoría de los posts de blog y, al momento de escribir esto, en un fork de surfpool, ya que un fork hereda las cuentas de mainnet y no necesariamente su esquema de renta. La aritmética de bytes no es un cuento que se cuenta sobre el número, ES el número, a la tasa que tu cluster cobre hoy. Cada celda clásica de la tabla de abajo es 293 por una tasa, así que la tabla es tan actual como esa tasa. Recalcula antes de presupuestar; la aritmética es una multiplicación.

Un destinatario comprimido cuesta unos 10,300 lamports, y el modelo de costos de m07-l3 ya te dijo por qué: 5,000 lamports para crear la cuenta comprimida, más unos 5,300 lamports de costo de estado por la única escritura que le pone tokens. Crear una vez, escribir una vez, listo.

Nada más en tu presupuesto de airdrop importa tanto como esa proporción, así que ponla en la página a cuatro escalas:

| Destinatarios | ATA clásicas | Comprimido | Ahorrado |
|---|---|---|---|
| 1,000 | 1.86 SOL | 0.0103 SOL | 99.4% |
| 10,000 | 18.56 SOL | 0.103 SOL | 99.4% |
| 100,000 | 185.56 SOL | 1.03 SOL | 99.4% |
| 1,000,000 | 1,855.57 SOL | 10.30 SOL | 99.4% |

Columna clásica a los 6,333 lamports/byte de mainnet, leídos el 2026-09-06. Multiplica por tu propia tasa sobre 6,333 para sacar la tuya; la columna comprimida no se mueve, porque el costo de un destinatario comprimido no es rent.

La proporción es plana porque los dos lados son lineales. Lo que cambia con la escala es si el número es sobrevivible. Con mil destinatarios a nadie le importa. Con un millón, la columna clásica pasa de 1,800 SOL, y a un precio de SOL de $150 eso es casi $300,000 de rent para repartir un token. Un millón de cuentas cuesta una casa, y el mes pasado costó una casa un poco más grande.

![Gráfico de barras agrupadas en un eje logarítmico que compara el costo de un airdrop clásico y uno comprimido de 1k a 1M de destinatarios, con la columna clásica cerca de dos mil SOL contra 10.3 SOL comprimido.](assets/v01-chart.png)

Ese cuarto de millón de dólares es la razón por la que se construyó la compresión ZK. Solana pasó los 500 millones de cuentas y estaba sumando alrededor de un millón al día cerca de noviembre de 2024, que fue el encuadre que usó Helius ese mes en su reseña de la keynote de compresión. El crecimiento del estado es la factura, y los airdrops son la forma más rápida de inflarla.

La columna clásica merece una nota de honestidad, porque halaga a la compresión si te la saltas. El rent es un depósito. Cierra la cuenta y cada lamport vuelve. Los 10,300 comprimidos se gastan y no vuelven nunca. Así que la oración correcta no es "la compresión es 200 veces más barata", es "la compresión convierte un depósito grande y reembolsable en un costo pequeño y permanente", y si eso es un buen canje depende de si alguien iba a cerrar esas cuentas alguna vez. En un airdrop, casi nadie lo hace.

### Cuatro maneras de mover un token hacia un desconocido

La tabla de costos solo sirve si cubre los métodos que podrías elegir de verdad, y son cuatro.

**Empujar hacia ATA clásicas.** Pagas rent por cada destinatario, ellos no hacen nada, los tokens simplemente están ahí. Simple, universalmente compatible, y la columna que cuesta una casa.

**Empujar tokens comprimidos.** Pagas unos 10,300 lamports de estado por destinatario, ellos tienen una hoja, y cada lectura posterior pasa por un RPC de compresión. Helius AirShip es la versión empaquetada de esto: instálalo globalmente, apúntalo a un mint y a una lista de destinatarios, déjalo agrupar en lotes.

```bash
npm install -g helius-airship@0.9.4
helius-airship --help
```

**Reclamo merkle.** Publicas una raíz de 32 bytes en la blockchain y nunca tocas la cuenta de un destinatario. Cada reclamante prueba su pertenencia y paga su propio reclamo. Tu costo por destinatario colapsa a un setup fijo dividido entre N, y el costo de ellos es una comisión de transacción más el rent de una cuenta de estado pequeña que les impide reclamar dos veces. El propio README del distributor pone el costo neto para un usuario alrededor de 0.000010 SOL una vez que vuelven a cerrar las cuentas. Sostén esas dos propiedades una contra otra por un segundo, porque no pueden ser las dos incondicionalmente ciertas: la protección contra replay es "la PDA ClaimStatus ya existe," y una PDA que un reclamante puede cerrar a mitad de la ventana es una PDA que puede volver a abrir para un segundo reclamo. El programa tiene que condicionar el cierre, al final del vesting, al clawback o al estado del distributor, y cuál de esas barreras elige en realidad es un hecho que lees en las restricciones de cuentas que lleva la instrucción de cierre, no en una frase de un README. Agrégalo a la lista de lectura de última instrucción que esta lección sigue haciendo crecer.

**La primitiva Light Claim.** El equivalente en estado comprimido de un reclamo: una cuenta comprimida que el destinatario materializa cuando aparece, lo que mantiene el costo del remitente cerca de cero y mantiene comprimido el almacenamiento del destinatario. Es la más nueva de las cuatro, hereda el requisito de lectura comprimida, y carga con la advertencia que el cierre de m07-l3 ya levantó sobre el riel de tokens de Light: una superficie emergente, primero en devnet, sin constantes de costo asentadas y fechadas al 2026-08. Esa ausencia es la razón por la que el modelo de costos de abajo le pone precio a los otros tres métodos y agrega una fila de alcance `airship-tx-side` en el cuarto slot en vez de inventar un número de Light Claim; cuando Light publique constantes de primera mano, la tabla gana una fila honesta.

El eje que nadie pone en la página de marketing es quién paga.

![Tabla comparativa que pone las cuatro filas del modelo de costos según costo del remitente, costo del reclamante, reembolsabilidad y necesidades de RPC, señalando que la fila de AirShip es una rebanada de alcance encima del empuje comprimido y que Light Claim no tiene precio.](assets/v02-comparison.png)

Lee esa tabla dos veces. Un reclamo merkle no es barato, está *desplazado*. Los lamports no desaparecieron, se movieron hacia la persona que recibe los tokens, y eso es una decisión de producto tanto como una decisión de costo: todo el que no reclama no te cuesta nada, y todo el que sí reclama paga unos 1.2 millones de lamports por hacerlo a la tasa de rent actual de mainnet, recuperables en su mayoría solo si la ruta de cierre de ClaimStatus le deja a un reclamante recuperar el rent, una barrera que esta lección marca más abajo como algo que no corrió. Para un drop donde esperas que la mitad de la lista te ignore, eso es una bendición. Para un drop a usuarios que nunca tuvieron SOL, es un muro.

### La brecha de AirShip, medida y no argumentada

Aquí hay una discrepancia con la que vas a chocar dentro de la primera hora de investigar esto por tu cuenta, y la forma en que la manejas importa más que el número.

El propio material de AirShip describe un drop de 10,000 destinatarios que cuesta unos 0.01 SOL. La matemática de compresión por destinatario de esta lección dice 10,000 por 10,300 lamports, que son 0.103 SOL. Eso es un factor de diez, entre dos fuentes que son las dos creíbles, sobre la misma herramienta.

No los promedies. No elijas el que te gusta. Abre la fuente y cuenta lo que cuenta cada uno.

Las constantes de AirShip viven en `packages/core/src/config/constants.ts` en el repositorio helius-labs/airship, y leerlas (2026-08) lo zanja en cosa de un minuto:

```
maxAddressesPerTransaction = 15
baseFee                    = 5,000 lamports
compressionFee             = 1,500 * 3 = 4,500 lamports
computeUnitLimit           = 550,000
computeUnitPrice           = 10,000 micro-lamports
```

Una comisión de prioridad de 550,000 unidades a 10,000 micro-lamports por unidad son 5,500 lamports. Suma la comisión base y la comisión de compresión y una transacción de AirShip le cuesta al remitente 15,000 lamports. Esa transacción lleva 15 destinatarios. Así que el número por destinatario de AirShip es 1,000 lamports, y 10,000 destinatarios por 1,000 lamports son exactamente 0.01 SOL.

Los dos números son correctos. Cuentan cosas distintas. AirShip está citando lo que sale de la billetera del remitente en comisiones, y la cifra de 10,300 es el costo de estado de las cuentas comprimidas mismas. Ninguno miente; el alcance nunca se declaró, así que los dos números nunca fueron comparables. El total por destinatario es la suma de los dos, unos 11,300 lamports, que es el número que deberías poner en un presupuesto.

Ese es el método entero. Cuando una cifra de un proveedor y una derivación de primera mano difieren en un orden de magnitud, la brecha es casi siempre de alcance, y la jugada honesta es medir en vez de citar. Tu tabla de costos recibe una fila explícita `airship-tx-side` exactamente por esta razón: una fila cuya etiqueta dice lo que cuenta no se puede citar fuera de alcance después.

Hay una segunda cosa escondida en ese archivo de constantes, y es la razón por la que el 15 existe. Una transacción tiene un tope de 1,232 bytes y cada cuenta que nombra cuesta 32 de ellos, así que agrupar quince destinatarios más las cuentas de sistema de Light, el pool de tokens y el árbol de estado en una sola transacción no entra si deletreas cada dirección. AirShip no las deletrea. Ya viene con una address lookup table, `9NYFyEqPkyXUhkerbGHXUXkvb4qpzeEdHuGpgbgpH1NJ` en mainnet y otra aparte en devnet, que guarda las cuentas estáticas que cada transacción de drop necesita para que cuesten un byte de índice cada una en vez de 32. El tamaño del lote no es una preferencia de ajuste, es lo que el presupuesto de bytes permite una vez que las cuentas fijas están en una tabla.

![Dos barras de presupuesto de bytes para una transacción de 1,232 bytes que muestran que reemplazar las direcciones estáticas completas de 32 bytes por índices de lookup table de un byte deja espacio para quince destinatarios en una sola transacción de AirShip.](assets/v03-diagram.png)

### La ruta de reclamo, y por qué SPROUT tiene que tomarla

Ahora la parte incómoda, y es específica del token que llevas construyendo todo el curso.

El conjunto de lanzamiento que SPROUT trae desde R6 son tres extensiones: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`. La tabla de extensiones soportadas de AirShip, en el mismo archivo de constantes, marca `transfer_fee_config` como no soportada, junto con `transfer_hook`, `permanent_delegate`, `mint_close_authority`, `default_account_state` y el conjunto confidencial; `metadata_pointer`, `metadata`, `interest_bearing_config` y los punteros de grupo están soportados.

Así que dos de las tres extensiones de SPROUT viajarían sin problema, y la tercera mata el drop entero, porque la allowlist tampoco es aditiva aquí, la misma lección que la allowlist de CP-Swap te enseñó en R6, sobre un riel distinto. La cobertura de Token-2022 que tiene el programa de tokens comprimidos sigue más o menos la misma línea, y no es arbitraria: una comisión que debe retenerse en un slot por cuenta no tiene adónde ir cuando la cuenta es un hash en un árbol.

![Tabla que divide las extensiones de Token-2022 según el soporte para airdrops comprimidos, con el par de metadatos de SPROUT en la columna soportada y su TransferFeeConfig sin soporte, lo que descalifica la ruta de compresión.](assets/v04-table.png)

Así que la columna más barata de tu tabla no está disponible para tu propio token. Y una reconciliación que se te debe, porque m07-l3 te calificó con la conclusión opuesta: el memo que esa lección aceptó nombró el drop de compost como la única carga de trabajo de Overgrowth que de verdad debería comprimirse, y en los ejes de costo que ese memo argumentó, conteo de escrituras y forma de acceso, tenía razón. Lo que ese memo asumió en silencio es que se pasa la barrera de extensiones, y esta lección es donde por fin se comprueba la suposición y falla para SPROUT en concreto. El veredicto se invierte por la legalidad, no por la aritmética; tu memo tenía razón sobre la carga de trabajo y estaba ciego al token, que es precisamente por qué el orden de descalificadores-antes-que-aritmética que codificaste ahí necesitaba un descalificador más que todavía no conocía. Toma eso como la lección y no como una derrota: el modelo de costos elige entre métodos que son legales para el token, y la legalidad viene del conjunto de extensiones que elegiste allá en el módulo 2. Si quieres la columna comprimida, la diseñas antes de acuñar.

Lo que deja la ruta de reclamo, y la implementación de referencia para ella es un programa real con una historia real. Jito construyó un merkle distributor para el airdrop de JTO: un programa de Anchor de código abierto que carga una raíz de 32 bytes, reparte una porción desbloqueada de inmediato, y libera una segunda porción, bloqueada, linealmente hasta una fecha de fin fija. Para JTO ese vesting corrió hasta el 7 de diciembre de 2024. El programa está desplegado en `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv` y confirmé que sigue siendo ejecutable en mainnet mientras escribía esto.

La advertencia honesta, porque es estructural para tus decisiones de dependencias: el repositorio está quieto. Su último push fue el 2025-04-30, unos dieciséis meses antes de esta lección. Quieto no es lo mismo que roto, y un distributor es un programa pequeño con un trabajo congelado, pero "forkeamos un repo que nadie ha tocado en más de un año" es una oración que tu equipo debería decir en voz alta en vez de descubrirla después.

Y una segunda advertencia, la que el método propio de este módulo exige antes de que aceptes mi ruteo: el distributor es un programa de Anchor de la era SPL, construido para JTO, un mint SPL clásico. Si su vault y su CPI de transferencia en el momento del reclamo pueden siquiera sostener y pagar un mint Token-2022, y mucho menos uno cuyo TransferFeeConfig retiene una tajada de cada reclamo de modo que los reclamantes reciben montos netos de comisión que tus asignaciones merkle nunca modelaron, es una pregunta de última instrucción, y yo no la corrí, así que esta lección no la responde. Antes de que SPROUT entregue este plan: lee la CPI de transferencia del programa (¿invoca el programa de tokens que sea dueño del mint, o un id clásico hardcodeado?), levanta un distributor en devnet fondeado con un mint Token-2022 con comisión, y reclama contra él. Si cualquiera de las dos comprobaciones falla, el plan recae en jugadas que este curso ya enseñó: exime del pago de comisiones al flujo del distributor y cuadra los libros con el harvest de comisiones retenidas, forkea la ruta de reclamo hacia `transfer_checked`, o apóyate en la división de dos mints de R6. Que la plataforma vete antes que la matemática se aplica a la ruta recomendada exactamente con la misma dureza con la que se aplicó a las rechazadas.

El distributor también tiene una respuesta para la parte de tu lista que nunca aparece, y vale la pena diseñarla antes de lanzar y no después. Su estado carga un `clawback_start_ts`, un `clawback_receiver` y una bandera `clawed_back`. Antes de esa marca de tiempo un intento de clawback falla con `ClawbackBeforeStart`. Después de ella, cualquiera puede disparar el barrido, y es seguro dejarlo así porque el destino está fijado: los tokens solo pueden ir a parar al `clawback_receiver` con el que se creó el distributor. Una vez barridos, cada reclamo restante falla con `ClaimExpired`. Así que la ventana es una política que fijas en el setup y no puedes renegociar. Demasiado corta y castigas a la gente que estaba de vacaciones; demasiado larga y tu tesorería se queda sentada sobre tokens con los que no puede planificar. Elígela a propósito, publícala, y pon la fecha en el mismo lugar donde publicas el árbol.

![Línea de tiempo de un merkle distributor desde el setup, pasando por la ventana de vesting lineal, hasta el punto de clawback, después del cual las asignaciones no reclamadas se barren y los reclamos posteriores revierten.](assets/v05-timeline.png)

### Qué hace claim_locked en realidad

Un **merkle distributor** es una cuenta que tiene una raíz, un vault y contadores, más una cuenta de estado pequeña por reclamante. Nada en la blockchain conoce la lista de destinatarios. El reclamante trae su asignación y una prueba; el programa recalcula la hoja y la comprueba contra la raíz.

Vale la pena mostrar el layout de la hoja exactamente, porque un error de orden de bytes aquí produce una prueba que falla sin ningún error útil:

![Diagrama de la preimagen de la hoja en el distributor, que hashea una pubkey de reclamante de 32 bytes y dos montos u64 little-endian en un nodo, y después separa por dominio las hojas y los padres con bytes de prefijo.](assets/v06-annotated-code.png)

Dos cosas se siguen de ese dibujo.

Primero, las pruebas son pequeñas pero no gratis. Un árbol de 100,000 hojas da una ruta de prueba de unos 17 hashes, 544 bytes, que cabe en una transacción junto con todo lo demás que un reclamo necesita. Compárala con las otras dos formas de prueba que conociste en este curso: un reclamo de cNFT lleva una ruta completa a través de un árbol merkle concurrente, que es la razón por la que esos árboles guardan un **canopy**, una banda cacheada de nodos superiores almacenada en la blockchain para que el cliente solo tenga que mandar la parte baja de la ruta; una escritura de compresión ZK lleva una **prueba de validez** de 128 bytes, una prueba de conocimiento cero de tamaño constante de que la cuenta que está gastando existe en el árbol. Tres árboles, tres estrategias de prueba, tres cosas distintas que terminan en tu transacción.

Segundo, y esta es la trampa que de verdad te va a morder: cada escritura a un árbol cambia su raíz, y cada prueba pendiente contra la raíz vieja se vuelve basura. Para el distributor la raíz se fija en el setup, así que esto te muerde durante la preparación y no en el momento del reclamo. Para los tokens comprimidos muerde todo el tiempo, porque el árbol de estado recibe escrituras de todo el mundo. La regla es la misma en los dos mundos. Trae la prueba inmediatamente antes de enviarla, nunca de un caché, nunca de un archivo que generaste la semana pasada.

La mitad del vesting es una sola función, y es lo bastante corta como para tenerla en la cabeza. El distributor guarda `start_ts` y `end_ts`; la cuenta de estado del reclamante guarda `locked_amount` y `locked_amount_withdrawn`. Lo que `claim_locked` puede pagar ahora mismo es la parte consolidada menos lo que ya se retiró:

```
vested      = 0                                    if now < start_ts
vested      = locked_amount                        if now >= end_ts
vested      = (now - start_ts) * locked_amount / (end_ts - start_ts)  otherwise
withdrawable = vested - locked_amount_withdrawn
```

La división entera trunca, lo que redondea hacia abajo, lo que favorece al vault en como mucho una unidad base. Eso es a propósito y es el tipo de detalle que vale la pena copiar en vez de mejorar.

![Un gráfico de vesting donde una línea recta acumula 900 millones de unidades base a lo largo de 90 días mientras una escalera de llamadas a claim_locked en los días 30, 45 y 90 la alcanza.](assets/v07-chart.png)

Dos propiedades de ese diseño merecen que se las nombre. Es un pull, así que las asignaciones no reclamadas se quedan en el vault sin costarte nada. Y es idempotente por reclamante, porque la cuenta de estado es una PDA sembrada sobre el reclamante y el distributor: un segundo intento de abrirla falla en la creación de la cuenta, no en una comprobación escrita a mano. Ahí es también donde encajan las perillas de vesting que vienen de la lección del launchpad. LaunchLab expresaba los bloqueos como un cliff más una duración del lado del launchpad; el distributor los expresa como `start_ts` y `end_ts` del lado de la distribución. Misma idea, asiento distinto.

![Diagrama de flujo de dos carriles donde el operador publica una raíz merkle de 32 bytes una sola vez mientras cada reclamante trae una prueba, llama a new_claim, y después llama repetidamente a claim_locked a medida que el vesting se acumula.](assets/v08-flowchart.png)

### El trade-off, nombrado

La compresión recorta el rent del airdrop en bastante más del 99%, y aquí está la factura de eso.

Una transferencia de token comprimido cuesta alrededor de 292,000 unidades de cómputo, porque el programa verifica una prueba de validez y vuelve a hashear el estado del árbol en cada escritura. La ruta clásica que mediste en el módulo 1, sobre el motor p-token, es de 76 CU para un `Transfer`. No pongas esos dos números uno al lado del otro como si fueran implementaciones competidoras del mismo producto. Uno es un saldo en una cuenta, otro es un saldo en un árbol más una prueba; la diferencia de cómputo es lo que cuesta el ahorro de almacenamiento, y se cobra por escritura para siempre.

Los saldos comprimidos también necesitan un RPC de compresión para leerse, normalmente quieren que se los descomprima antes de que la mayoría del DeFi los mire, y un token que carga la extensión equivocada no puede usarlos en absoluto.

La ruta de reclamo tiene su propia factura. Agrega una dependencia de programa, en este caso una cuyo repositorio está quieto desde el 2025-04-30. Requiere una segunda transacción por destinatario para la porción con vesting, y una tercera, y una cuarta, porque vesting lineal quiere decir que un reclamante vuelve tantas veces como le dé la gana. Empuja unos 1.3 millones de lamports de costo sobre cada reclamante. Y necesita un artefacto off-chain, el árbol y sus pruebas, alojado en algún lugar al que tus usuarios puedan llegar.

Un límite antes del lab. Leer estado comprimido a través de un RPC de DAS o de Photon es consumo, y ahí es donde este curso se detiene. Levantar el indexador que va debajo, plugins de Geyser, streams de gRPC, backfills, es el territorio del curso planificado Client-Side Mastery, donde la elección de proveedor y la confiabilidad del índice son problemas de primera clase y no una línea en un lab.

## Lab: construye el compost-airdrop

El artefacto es `compost-airdrop`, una carpeta de lección que vive al lado del `sprout-launch/` de las últimas dos lecciones (este módulo mantiene sus labs como carpetas hermanas y no bajo `labs/`; ponlas donde tu workspace guarde las otras, las rutas de los comandos son todas relativas). Tiene una tabla de costos que calcula y hace assert de las cifras canónicas, y una ruta de reclamo que prueba tanto un reclamo desbloqueado como uno con vesting. Consume `sprout-mint`, el mint de R3 por su nombre en la escalera de artefactos, en el sentido de que el conjunto de extensiones de SPROUT es lo que fuerza la elección de método que acabas de leer.

Todo lo que viene después del calentamiento es local y determinista. Sin RPC, sin keypair, sin esperas.

1. **El modelo de costos, recorrido.** Crea `compost-airdrop/cost-model.ts`. Cada constante es o una cifra congelada del curso o un valor leído de una fuente pública, y el archivo dice cuál.

    ```typescript
    // compost-airdrop/cost-model.ts
    export const LAMPORTS_PER_SOL = 1_000_000_000;

    /** A classic SPL token account: 165 bytes of data plus the 128-byte account header. */
    export const CLASSIC_ATA_BYTES = 293;
    /**
     * Rent-exemption price of one byte for two years. THE ONLY NUMBER IN THIS
     * FILE THAT BELONGS TO THE NETWORK RATHER THAN TO THE LAYOUT, and the one
     * you must replace with your own read. mainnet-beta, 2026-09-06; devnet was
     * a step further down at 5,080 and a surfpool fork was still on the old
     * 6,960. Get yours in one line:
     *
     *   solana rent 0 --url <cluster>   # divide the lamports by 128
     *
     * SIMD-0437 is stepping this down over several releases, so a number you
     * copied from a lesson is a number you will re-derive.
     */
    export const LAMPORTS_PER_BYTE = 6_333;
    /**
     * Rent locked by one classic recipient account. DERIVED, not pasted: at
     * 6,333 this is 1,855,569, which is what `solana rent 165 --url
     * mainnet-beta` printed on 2026-09-06. Refundable if the account is closed.
     */
    export const CLASSIC_ATA_LAMPORTS = CLASSIC_ATA_BYTES * LAMPORTS_PER_BYTE;

    /** Creating one compressed token account (m07-l3's figure). */
    export const COMPRESSED_CREATE_LAMPORTS = 5_000;
    /** State cost of one compressed write on Light's V2 program line (m07-l3's figure). */
    export const COMPRESSED_WRITE_LAMPORTS = 5_300;
    /** One compressed recipient: created once, written once by the drop. */
    export const COMPRESSED_RECIPIENT_LAMPORTS =
      COMPRESSED_CREATE_LAMPORTS + COMPRESSED_WRITE_LAMPORTS;

    // AirShip's own transaction-side constants, read from
    // helius-labs/airship, packages/core/src/config/constants.ts (read 2026-08).
    export const AIRSHIP_BASE_FEE = 5_000;
    export const AIRSHIP_COMPRESSION_FEE = 1_500 * 3;
    export const AIRSHIP_CU_LIMIT = 550_000;
    export const AIRSHIP_CU_PRICE_MICRO_LAMPORTS = 10_000;
    export const AIRSHIP_RECIPIENTS_PER_TX = 15;

    export function airshipPriorityFeeLamports(): number {
      return Math.ceil(
        (AIRSHIP_CU_LIMIT * AIRSHIP_CU_PRICE_MICRO_LAMPORTS) / 1_000_000,
      );
    }

    export function airshipPerTransactionLamports(): number {
      return AIRSHIP_BASE_FEE + AIRSHIP_COMPRESSION_FEE + airshipPriorityFeeLamports();
    }

    /** AirShip's transaction-side cost per recipient. State cost is NOT in here. */
    export function airshipPerRecipientLamports(): number {
      return airshipPerTransactionLamports() / AIRSHIP_RECIPIENTS_PER_TX;
    }

    /** ClaimStatus: 8-byte discriminator + 32 + 8 + 8 + 8. */
    export const CLAIM_STATUS_BYTES = 64;
    /** Rent for the ClaimStatus PDA the claimant opens. Refundable on close. */
    export const CLAIM_STATUS_RENT_LAMPORTS =
      (CLAIM_STATUS_BYTES + 128) * LAMPORTS_PER_BYTE;
    export const CLAIM_TX_FEE_LAMPORTS = 5_000;

    export type MethodId = "classic" | "compressed" | "airship-tx-side" | "merkle-claim";

    export interface Method {
      id: MethodId;
      label: string;
      /** Lamports the drop operator pays per recipient. */
      senderLamports: number;
      /** Lamports the recipient pays to end up holding the tokens. */
      claimantLamports: number;
      /** Lamports on this row that come back if accounts are closed. */
      refundableLamports: number;
      note: string;
    }

    export function methods(): Method[] {
      return [
        {
          id: "classic",
          label: "classic SPL ATA, pushed",
          senderLamports: CLASSIC_ATA_LAMPORTS,
          claimantLamports: 0,
          refundableLamports: CLASSIC_ATA_LAMPORTS,
          note: `${CLASSIC_ATA_BYTES} bytes rent-exempt at ${LAMPORTS_PER_BYTE.toLocaleString(
            "en-US",
          )} lamports/byte; matches \`solana rent 165\``,
        },
        {
          id: "compressed",
          label: "compressed token, pushed",
          // TODO(you): a compressed recipient is created once and written once
          // by the drop. Both constants are already defined above.
          senderLamports: 0,
          claimantLamports: 0,
          refundableLamports: 0,
          note: "state cost only: create + one write, nothing refundable",
        },
        {
          id: "airship-tx-side",
          label: "compressed via AirShip (transaction side only)",
          senderLamports: airshipPerRecipientLamports(),
          claimantLamports: 0,
          refundableLamports: 0,
          note: `${airshipPerTransactionLamports()} lamports per tx / ${AIRSHIP_RECIPIENTS_PER_TX} recipients`,
        },
        {
          id: "merkle-claim",
          label: "merkle claim (claimant-paid)",
          senderLamports: 0,
          claimantLamports: CLAIM_TX_FEE_LAMPORTS + CLAIM_STATUS_RENT_LAMPORTS,
          // Booked as spent, deliberately: ClaimStatus rent is refundable only
          // if the program's close path lets the CLAIMANT reclaim it, and that
          // gate is flagged unverified in the prose. Flip this to
          // CLAIM_STATUS_RENT_LAMPORTS only after you have run the close.
          refundableLamports: 0,
          note: "sender pays a fixed setup; the claimant pays the claim (rent refund unverified)",
        },
      ];
    }

    export function sol(lamports: number): string {
      return `${(lamports / LAMPORTS_PER_SOL).toFixed(4)} SOL`;
    }
    ```

    El `TODO` es tuyo y es el problema de completion de esta lección. Una línea, dos constantes, y la fila de 100k empieza a leerse correctamente.

2. **La tabla, y los asserts que la convierten en una prueba.** Crea `compost-airdrop/cost-table.ts`. Una tabla que nadie revisa es una tabla que se pudre en silencio, así que el archivo termina haciendo assert de las cifras canónicas.

    ```typescript
    // compost-airdrop/cost-table.ts
    import {
      CLASSIC_ATA_BYTES,
      CLASSIC_ATA_LAMPORTS,
      COMPRESSED_RECIPIENT_LAMPORTS,
      LAMPORTS_PER_BYTE,
      airshipPerRecipientLamports,
      methods,
      sol,
    } from "./cost-model";

    const SIZES = [1_000, 10_000, 100_000, 1_000_000];

    function pad(s: string, n: number): string {
      return s.length >= n ? s : s + " ".repeat(n - s.length);
    }

    console.log(
      `${pad("method", 48)}${pad("lamports/ea", 12)}` +
        SIZES.map((n) => pad(n.toLocaleString("en-US"), 14)).join(""),
    );

    for (const m of methods()) {
      const perRecipient = m.senderLamports + m.claimantLamports;
      const cells = SIZES.map((n) => pad(sol(perRecipient * n), 14)).join("");
      console.log(
        `${pad(m.label, 48)}${pad(perRecipient.toLocaleString("en-US"), 12)}${cells}`,
      );
    }

    const classic100k = CLASSIC_ATA_LAMPORTS * 100_000;
    const compressed100k = COMPRESSED_RECIPIENT_LAMPORTS * 100_000;
    const saved = 1 - compressed100k / classic100k;

    console.log("");
    console.log(
      `100k recipients: classic ${sol(classic100k)} / compressed ${sol(compressed100k)} ` +
        `(${(saved * 100).toFixed(1)}% saved)`,
    );
    console.log(
      `AirShip's transaction side adds ${airshipPerRecipientLamports().toLocaleString("en-US")} ` +
        `lamports/recipient on top of the state cost.`,
    );

    const expect = (label: string, got: number, want: number, tol: number) => {
      if (Math.abs(got - want) > tol) {
        throw new Error(`${label}: got ${got}, expected about ${want}`);
      }
    };

    // Two kinds of assertion, and the difference matters. The compressed
    // figures are absolutes, because a compressed recipient's cost is not rent
    // and does not move with the rent schedule. The classic figures assert
    // CONSISTENCY with whatever LAMPORTS_PER_BYTE you set, because pinning them
    // to an absolute would turn this gate into a tripwire that fires every time
    // the network cuts rent, which is the opposite of what a cost model is for.
    expect("classic per recipient", CLASSIC_ATA_LAMPORTS, CLASSIC_ATA_BYTES * LAMPORTS_PER_BYTE, 0);
    expect("compressed per recipient", COMPRESSED_RECIPIENT_LAMPORTS, 10_300, 0);
    expect("classic at 100k (SOL)", classic100k / 1e9, (CLASSIC_ATA_LAMPORTS * 100_000) / 1e9, 0.001);
    expect("compressed at 100k (SOL)", compressed100k / 1e9, 1.03, 0.01);
    expect("saving", saved, 1 - COMPRESSED_RECIPIENT_LAMPORTS / CLASSIC_ATA_LAMPORTS, 0.0001);

    // The assertion that actually reads your TODO: the compressed ROW in
    // methods() must carry the per-recipient state cost, not the shipped zero.
    const compressedRow = methods().find((m) => m.id === "compressed");
    expect(
      "compressed row senderLamports (the TODO in cost-model.ts)",
      compressedRow?.senderLamports ?? 0,
      COMPRESSED_RECIPIENT_LAMPORTS,
      0,
    );
    console.log("cost table OK");
    ```

    Córrelo. Antes de que rellenes el `TODO`, la tabla imprime un `0` lamports/ea sin sentido para la fila comprimida y el assert de la fila comprimida del final lanza, que es justamente el punto: la barrera lee la fila donde vive tu TODO, no solo las constantes de alrededor.

    ```bash
    npx tsx compost-airdrop/cost-table.ts
    ```

    Después de que lo rellenes, las últimas líneas dicen:

    ```
    100k recipients: classic 185.5569 SOL / compressed 1.0300 SOL (99.4% saved)
    AirShip's transaction side adds 1,000 lamports/recipient on top of the state cost.
    cost table OK
    ```

    Esas cifras de SOL son `LAMPORTS_PER_BYTE` por un conteo fijo de bytes, así que son tuyas para moverlas: pon la constante a la tasa de tu propio cluster y toda la tabla la sigue, asserts incluidos.

3. **El árbol.** Crea `compost-airdrop/merkle.ts`. Este es el hasheo del distributor, portado exactamente: sha256, un byte cero delante de las hojas, un byte uno delante de los padres, pares ordenados, y un nodo impar emparejado consigo mismo. Ninguna dependencia en absoluto. Esa última regla es la que hay que vigilar, porque es invisible en el árbol de cuatro destinatarios de este lab (cuatro es una potencia de dos, así que ningún nivel es nunca impar) y decide cada raíz que computes sobre una lista real. El challenge 1 agrega un quinto destinatario, que es donde empieza a importar.

    ```typescript
    // compost-airdrop/merkle.ts
    // Ported from jito-foundation/distributor (merkle-tree/src/merkle_tree.rs,
    // programs/merkle-distributor/src/instructions/new_claim.rs), read on
    // 2026-08-22. m09-l2 ports the same file again under different names; if
    // the two ever disagree, one of them is wrong about a deployed program.
    import { createHash } from "node:crypto";

    export type Hash32 = Uint8Array;

    export function sha256(...parts: Uint8Array[]): Hash32 {
      const h = createHash("sha256");
      for (const p of parts) h.update(p);
      return new Uint8Array(h.digest());
    }

    export function u64le(value: bigint): Uint8Array {
      const out = new Uint8Array(8);
      new DataView(out.buffer).setBigUint64(0, value, true);
      return out;
    }

    export function hex(bytes: Uint8Array): string {
      return Buffer.from(bytes).toString("hex");
    }

    export interface Allocation {
      /** 32-byte claimant pubkey. */
      claimant: Uint8Array;
      amountUnlocked: bigint;
      amountLocked: bigint;
    }

    /** The node the program hashes, then the leaf prefix that stops second-preimage tricks. */
    export function leafHash(a: Allocation): Hash32 {
      const node = sha256(a.claimant, u64le(a.amountUnlocked), u64le(a.amountLocked));
      return sha256(Uint8Array.of(0), node);
    }

    function compare(a: Uint8Array, b: Uint8Array): number {
      for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return a[i] < b[i] ? -1 : 1;
      }
      return 0;
    }

    function hashPair(a: Hash32, b: Hash32): Hash32 {
      return compare(a, b) <= 0
        ? sha256(Uint8Array.of(1), a, b)
        : sha256(Uint8Array.of(1), b, a);
    }

    export interface Tree {
      root: Hash32;
      leaves: Hash32[];
      proofFor(index: number): Hash32[];
    }

    export function buildTree(allocations: Allocation[]): Tree {
      if (allocations.length === 0) throw new Error("empty tree");
      const leaves = allocations.map(leafHash);

      const levels: Hash32[][] = [leaves];
      while (levels[levels.length - 1].length > 1) {
        const below = levels[levels.length - 1];
        const above: Hash32[] = [];
        for (let i = 0; i < below.length; i += 2) {
          // An odd node is paired with ITSELF, not promoted to the level above.
          // This is the line that decides whether your root equals the deployed
          // program's: merkle_tree.rs duplicates the last entry when a level's
          // length is odd, and promoting instead gives a different root for
          // every tree whose width is not a power of two.
          const right = i + 1 < below.length ? below[i + 1] : below[i];
          above.push(hashPair(below[i], right));
        }
        levels.push(above);
      }

      return {
        root: levels[levels.length - 1][0],
        leaves,
        proofFor(index: number): Hash32[] {
          if (index < 0 || index >= leaves.length) throw new Error("no such leaf");
          const proof: Hash32[] = [];
          let i = index;
          for (let level = 0; level < levels.length - 1; level++) {
            const nodes = levels[level];
            const sibling = i % 2 === 0 ? i + 1 : i - 1;
            // Same duplication on the proof side: find_path uses level[index]
            // itself when the right sibling is off the end, so the proof folds
            // through the self-pair the builder created.
            proof.push(sibling < nodes.length ? nodes[sibling] : nodes[i]);
            i = Math.floor(i / 2);
          }
          return proof;
        },
      };
    }

    /** The on-chain check, in TypeScript. Same order, same prefixes, same sorting. */
    export function verifyProof(proof: Hash32[], root: Hash32, leaf: Hash32): boolean {
      let computed = leaf;
      for (const element of proof) computed = hashPair(computed, element);
      return compare(computed, root) === 0;
    }
    ```

4. **La matemática del vesting.** Crea `compost-airdrop/vesting.ts`. Las mismas ramas, el mismo truncamiento, los mismos nombres de campo que el `ClaimStatus` del programa.

    ```typescript
    // compost-airdrop/vesting.ts
    export interface ClaimStatus {
      claimant: string;
      unlockedAmount: bigint;
      lockedAmount: bigint;
      lockedAmountWithdrawn: bigint;
    }

    /** How much of the locked allocation has vested at currTs. */
    export function unlockedAmount(
      cs: ClaimStatus,
      currTs: number,
      startTs: number,
      endTs: number,
    ): bigint {
      if (currTs < startTs) return 0n;
      if (currTs >= endTs) return cs.lockedAmount;
      const timeIntoUnlock = BigInt(currTs - startTs);
      const totalUnlockTime = BigInt(endTs - startTs);
      // Integer division truncates, which rounds down in the vault's favour.
      return (timeIntoUnlock * cs.lockedAmount) / totalUnlockTime;
    }

    /** What claim_locked would actually transfer right now. */
    export function amountWithdrawable(
      cs: ClaimStatus,
      currTs: number,
      startTs: number,
      endTs: number,
    ): bigint {
      const vested = unlockedAmount(cs, currTs, startTs, endTs);
      if (vested < cs.lockedAmountWithdrawn) {
        throw new Error("arithmetic error: withdrawn exceeds vested");
      }
      return vested - cs.lockedAmountWithdrawn;
    }
    ```

    Comprueba el orden de las ramas contra el programa antes de seguir. Un tiempo anterior a `start_ts` no da nada; un tiempo en o después de `end_ts` da el monto bloqueado entero; en medio es una proporción. Un inicio posterior al fin no es un caso especial, simplemente nunca empieza.

5. **La corrida de reclamos.** Crea `compost-airdrop/claim.ts`. Esto lleva un distributor de cuatro destinatarios por las dos rutas de reclamo y por tres fallas. El reclamo desbloqueado está trabajado para ti; el loop bloqueado es la mitad en solitario, y está marcado. Una nota sobre el stack: este archivo es pura simulación local y no llama nada de Light, pero vive en el mismo workspace v1 en cuarentena que el calentamiento, así que reusa el `PublicKey` de ese workspace para base58 y aritmética de bytes en vez de mezclar un segundo SDK en una sola carpeta.

    ```typescript
    // compost-airdrop/claim.ts
    import { PublicKey } from "@solana/web3.js";
    import { buildTree, hex, leafHash, verifyProof, type Allocation } from "./merkle";
    import { amountWithdrawable, type ClaimStatus } from "./vesting";

    const DAY = 24 * 60 * 60;

    // A distributor with a 90-day linear unlock, the shape the JTO drop used.
    const START_TS = 1_760_000_000;
    const END_TS = START_TS + 90 * DAY;

    interface Recipient {
      name: string;
      address: PublicKey;
      unlocked: bigint;
      locked: bigint;
    }

    // Deterministic stand-in addresses so the run is reproducible.
    function addr(seed: string): PublicKey {
      const bytes = new Uint8Array(32);
      Buffer.from(seed).copy(bytes);
      return new PublicKey(bytes);
    }

    const RECIPIENTS: Recipient[] = [
      { name: "early-plot-holder", address: addr("overgrowth-plot-01"), unlocked: 400_000_000n, locked: 0n },
      { name: "seed-round-farmer", address: addr("overgrowth-farm-02"), unlocked: 100_000_000n, locked: 900_000_000n },
      { name: "almanac-author", address: addr("overgrowth-alma-03"), unlocked: 250_000_000n, locked: 0n },
      { name: "compost-donor", address: addr("overgrowth-comp-04"), unlocked: 50_000_000n, locked: 150_000_000n },
    ];

    const allocations: Allocation[] = RECIPIENTS.map((r) => ({
      claimant: r.address.toBytes(),
      amountUnlocked: r.unlocked,
      amountLocked: r.locked,
    }));

    const tree = buildTree(allocations);
    console.log(`root ${hex(tree.root)}`);
    console.log(`leaves ${tree.leaves.length}`);

    interface Distributor {
      root: Uint8Array;
      maxNumNodes: number;
      numNodesClaimed: number;
      clawedBack: boolean;
      vault: bigint;
    }

    const distributor: Distributor = {
      root: tree.root,
      maxNumNodes: RECIPIENTS.length,
      numNodesClaimed: 0,
      clawedBack: false,
      vault: RECIPIENTS.reduce((sum, r) => sum + r.unlocked + r.locked, 0n),
    };

    const claimStatuses = new Map<string, ClaimStatus>();

    function newClaim(index: number, proof: Uint8Array[]): ClaimStatus {
      const r = RECIPIENTS[index];
      const key = r.address.toBase58();
      if (distributor.clawedBack) throw new Error("ClaimExpired");
      if (claimStatuses.has(key)) throw new Error("already claimed: ClaimStatus PDA exists");

      // Verify BEFORE counting: a failed InvalidProof attempt must leave the
      // distributor untouched, or a stream of bad proofs (challenge 2 sends one
      // on purpose) inflates numNodesClaimed until legitimate claimants hit
      // MaxNodesExceeded for no visible reason. On chain the same property
      // falls out of transaction atomicity; a sim has to order it by hand.
      const leaf = leafHash({
        claimant: r.address.toBytes(),
        amountUnlocked: r.unlocked,
        amountLocked: r.locked,
      });
      if (!verifyProof(proof, distributor.root, leaf)) throw new Error("InvalidProof");

      if (distributor.numNodesClaimed + 1 > distributor.maxNumNodes) throw new Error("MaxNodesExceeded");
      distributor.numNodesClaimed += 1;

      const status: ClaimStatus = {
        claimant: key,
        unlockedAmount: r.unlocked,
        lockedAmount: r.locked,
        lockedAmountWithdrawn: 0n,
      };
      claimStatuses.set(key, status);
      distributor.vault -= r.unlocked;
      return status;
    }

    // TODO(you): claim_locked. Ask vesting.ts what is withdrawable at currTs,
    // reject a zero payout the way the program does (InsufficientUnlockedTokens),
    // add it to lockedAmountWithdrawn, refuse to exceed lockedAmount
    // (ExceededMaxClaim), and take it out of the vault. Return the amount.
    function claimLocked(status: ClaimStatus, currTs: number): bigint {
      throw new Error("not implemented");
    }

    // Claim 1: an allocation with no locked half at all.
    const plotIndex = 0;
    const plotProof = tree.proofFor(plotIndex);
    const plotStatus = newClaim(plotIndex, plotProof);
    console.log(
      `unlocked claim: ${RECIPIENTS[plotIndex].name} took ${plotStatus.unlockedAmount} base units ` +
        `with a ${plotProof.length}-hash proof`,
    );

    // Claim 2: unlocked now, then the locked half as it vests.
    const farmIndex = 1;
    const farmStatus = newClaim(farmIndex, tree.proofFor(farmIndex));
    console.log(`unlocked claim: ${RECIPIENTS[farmIndex].name} took ${farmStatus.unlockedAmount} base units`);

    for (const [label, ts] of [
      ["day 0", START_TS],
      ["day 30", START_TS + 30 * DAY],
      ["day 45", START_TS + 45 * DAY],
      ["day 90", END_TS],
    ] as const) {
      try {
        const amount = claimLocked(farmStatus, ts);
        console.log(
          `claim_locked at ${label}: released ${amount}, withdrawn so far ` +
            `${farmStatus.lockedAmountWithdrawn}/${farmStatus.lockedAmount}`,
        );
      } catch (err) {
        console.log(`claim_locked at ${label}: rejected (${(err as Error).message})`);
      }
    }

    // A double claim is refused by the ClaimStatus PDA, not by good manners.
    try {
      newClaim(plotIndex, plotProof);
    } catch (err) {
      console.log(`replay of the unlocked claim: rejected (${(err as Error).message})`);
    }

    // A tree write invalidates every outstanding proof.
    const grown = buildTree([
      ...allocations,
      { claimant: addr("overgrowth-late-05").toBytes(), amountUnlocked: 10n, amountLocked: 0n },
    ]);
    const stale = verifyProof(plotProof, grown.root, tree.leaves[plotIndex]);
    console.log(`stale proof against the grown tree verifies: ${stale}`);

    console.log(`vault left: ${distributor.vault} base units`);
    ```

6. **Córrelo y lee cada línea.**

    ```bash
    npx tsx compost-airdrop/claim.ts
    ```

    Con `claimLocked` implementado, la salida es esta, y cada línea es una afirmación que puedes defender:

    ```
    root fb855944c186313d7cc04782398567bd468dc5be3812c2719f8089e079f4c1a3
    leaves 4
    unlocked claim: early-plot-holder took 400000000 base units with a 2-hash proof
    unlocked claim: seed-round-farmer took 100000000 base units
    claim_locked at day 0: rejected (InsufficientUnlockedTokens)
    claim_locked at day 30: released 300000000, withdrawn so far 300000000/900000000
    claim_locked at day 45: released 150000000, withdrawn so far 450000000/900000000
    claim_locked at day 90: released 450000000, withdrawn so far 900000000/900000000
    replay of the unlocked claim: rejected (already claimed: ClaimStatus PDA exists)
    stale proof against the grown tree verifies: false
    vault left: 450000000 base units
    ```

    El día 30 de 90 libera exactamente un tercio de 900,000,000. El día 45 libera el siguiente sexto, porque ya se había tomado un tercio. El día 90 libera el resto de una sola vez. La raíz es determinista, así que si la tuya difiere, tu preimagen de hoja difiere, y el diagrama de bytes de arriba es dónde mirar.

7. **Haz typecheck de todo.** El lab está escrito en strict, y la estrictez es lo que atrapa un `number` donde va un `bigint`.

    ```bash
    npx tsc --noEmit --strict --target es2022 --module esnext \
      --moduleResolution bundler --skipLibCheck compost-airdrop/*.ts
    ```

    Silencio quiere decir limpio. Si cambias `bundler` por `nodenext` aquí te van a decir que escribas `./merkle.js` en los imports, lo que es correcto para ESM hecho a mano y fricción sin sentido para un lab que corre a través de `tsx`.

8. **Opcional, y honesto sobre su costo.** Para correr el reclamo contra el programa real en vez de contra un port de él, clona jito-foundation/distributor, construye el programa de Anchor, despliégalo en una instancia de surfpool o en devnet, y manéjalo con su propia CLI. Eso es una toolchain de Rust y una tarde. El port que acabas de escribir verifica contra la misma raíz con la misma preimagen, así que nada de lo que aprendiste cambia; lo que compras con la tarde es la confianza de que tus bytes de hoja coinciden con los de un programa desplegado, que vale la pena tener antes de un drop en mainnet y no antes de una lección.

## Challenge

Tres extensiones, en orden creciente de cuánto te van a enseñar.

**Uno.** Agrega un quinto destinatario cuya asignación esté enteramente bloqueada, con cero desbloqueado. Reclámalo. `new_claim` transfiere un monto desbloqueado de cero, lo que tiene éxito e igual abre la PDA ClaimStatus, y solo entonces `claim_locked` tiene algo que pagar. Confirma que el primer `claim_locked` después de `start_ts` es lo que de verdad mueve tokens para ese destinatario.

**Dos.** Demuestra la falla de prueba vieja de punta a punta y no como un booleano, y hay dos trampas horneadas dentro en las que una lectura literal cae de frente. Primero, tu `newClaim` verifica contra `distributor.root`, que todavía tiene la raíz VIEJA, así que una prueba vieja verifica perfectamente contra ella; para montar la falla tienes que hacer que el distributor cargue la raíz del árbol crecido, ya sea construyendo un segundo distributor a partir del árbol reconstruido o poniendo explícitamente `distributor.root = grown.root` y diciéndolo en un comentario. Segundo, usa un destinatario que NO haya reclamado ya, porque la comprobación de ClaimStatus-existe se dispara antes de la verificación de la prueba y enmascararía la falla que intentas ver. Con las dos cosas manejadas: genera una prueba, reconstruye el árbol con una asignación más, apunta el distributor a la raíz nueva, intenta `newClaim` con la prueba vieja, y atrapa `InvalidProof`. Escribe una oración en un comentario explicando por qué volver a traer la prueba inmediatamente antes de enviarla es el único arreglo confiable.

![Un flujo de cinco pasos por las comprobaciones de newClaim donde el test de ClaimStatus ya existente del paso dos rechaza a los reclamantes repetidos antes de que siquiera se verifique la prueba del paso tres.](assets/v09-flowchart.png)

**Tres.** Extiende la tabla de costos con una columna `total_cost_of_ownership`: para cada método, el costo del remitente más el costo del reclamante menos lo que sea reembolsable, con 100,000 destinatarios. Después responde, en el archivo, qué método entregarías para SPROUT y por qué, dado que SPROUT carga una comisión de transferencia. La respuesta no es la fila más barata y tu comentario debería decirlo.

Acepta tu trabajo cuando la fila de 100k de la tabla diga unos 186 SOL en clásico a la tasa de rent actual de mainnet (o 293 por la tasa que pongas, por 100,000) contra unos 1.03 SOL comprimido, los dos reclamos aterricen, y las rutas de replay y de prueba vieja sean rechazadas por las razones por las que el programa las rechazaría.

## Checkpoint

Ahora puedes hacer algo que suena trivial y que casi nadie hace antes de comprometerse con un drop: ponerle precio de cuatro maneras, a partir de constantes que puedes señalar, y decir quién paga.

En concreto, deberías poder responder estas sin buscar nada. ¿Cuánto cuesta un destinatario clásico, y es reembolsable? ¿Cuánto cuesta un destinatario comprimido, y lo es? ¿Por qué el número de AirShip y la cifra de estado por destinatario difieren en diez veces? ¿Cuál de las tres extensiones de SPROUT descalifica el drop comprimido, y cuáles dos habrían estado bien? ¿Y cuánto paga `claim_locked` en el día 30 de un desbloqueo de 90 días sobre una asignación de 900,000,000 unidades base?

Si la última te queda borrosa, vale la pena volver a correr el paso 6 con un par de marcas de tiempo extra en vez de leer la fórmula otra vez. Ver cómo el contador de retirado persigue el monto consolidado es lo que hace obvia la resta.

Una cosa que marco porque me atrapó a mí mientras escribía este lab: construí el árbol, cacheé las pruebas en una variable, y después reconstruí el árbol con un destinatario extra dos pasos más tarde, exactamente como un operador real agrega una asignación tardía. Cada prueba cacheada era silenciosamente inútil. El código lo atrapa ahora porque esa falla se imprime como una línea de salida, que es la única razón por la que confío en él.

La economía ahora tiene un token, una plataforma y una ruta de distribución. El módulo 9 conecta el dinero: adónde fluyen realmente las comisiones una vez que tienes tenedores, cómo una tesorería recompra y quema, y cómo un programa de puntos se convierte en token sin un segundo lanzamiento. Trae tus notas sobre retención de comisiones del módulo 2, las vas a necesitar en la primera página.

¡Feliz composting!
