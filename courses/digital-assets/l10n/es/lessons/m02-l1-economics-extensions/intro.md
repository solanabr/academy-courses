# Extensiones de economía: comisiones, interés, UI escalada y adónde VAN las comisiones

## Resumen

La lección pasada derivaste la matriz de conflictos de extensiones directo de la fuente y construiste check-combo, que acepta un conjunto legal de extensiones y rechaza uno ilegal. Ahora gastas esa matriz: empiezas a construir SPROUT de verdad.

Así me fue en mi primera pasada por esta construcción. Puse una comisión de transferencia del 1% en un mint de prueba, corrí diez ventas y abrí la cuenta de tesorería para admirar la recaudación. Cero. No un error de redondeo, no una demora: cero. Las comisiones eran reales, el programa había recaudado cada una de ellas, y estaban paradas en un lugar en el que nunca se me ocurrió mirar. Ese lugar es toda la segunda mitad de esta lección, porque Token-2022 retiene las comisiones de transferencia en la cuenta de token del DESTINATARIO, y nada, nunca, las mueve a una tesorería hasta que lo hagas tú mismo.

Hoy SPROUT, la moneda de Overgrowth, el juego ficticio de cultivo cooperativo cuya economía levanta este curso, hace crecer su capa de economía: un mint Token-2022 que lleva TransferFeeConfig más una de las dos extensiones que reescriben el display, InterestBearingConfig o ScaledUiAmount, construido desde instrucciones crudas con kit. Vas a correr transferencias reales en un surfnet local, mirar cómo las comisiones se apilan donde menos las esperas, y después barrerlas a casa con la secuencia de harvest que, en el Módulo 9, el riel de enrutamiento de comisiones después llama por su nombre.

Algo que hacer antes de la teoría: levanta el lab. Ármalo al lado de tu trabajo anterior y arranca un surfnet local (surfpool 1.2.1 acá, 2026-08-22; en macOS `brew install txtx/taps/surfpool`, en otras plataformas agárralo de la página de releases de surfpool):

```bash
mkdir -p labs/m02-l1 && cd labs/m02-l1
npm init -y && npm pkg set type=module
surfpool start --no-tui --no-studio
```

Deja el surfnet corriendo en esa terminal y abre una segunda. Vamos a ir llenando la carpeta sobre la marcha.

El repliegue de la ayuda en esta lección: la construcción del mint se trabaja completa, recorro cada instrucción y tú escribes a la par. Las constantes de comisión y la secuencia de harvest son ejercicios de completar: el archivo ya viene con TODOs y la teoría te dice exactamente qué va en ellos. Y la aritmética de comisiones en sí es en solitario: el coding challenge del módulo te entrega un `transferFee` roto y un conjunto de pruebas, sin apoyo.

## El ciclo de vida de la comisión

### Una extensión que mueve dinero, dos que solo hablan de él

Las tres extensiones de este conjunto parecen hermanas y no lo son. TransferFeeConfig cambia lo que una transferencia HACE: los tokens de verdad se mueven distinto, alguien de verdad recibe menos. InterestBearingConfig y ScaledUiAmount cambian cómo se VE un saldo: reescriben el número que muestra una billetera mientras el monto crudo on-chain sigue intacto. Sostén esa división con firmeza, porque cada trampa que viene más adelante nace de difuminarla.

Empieza por la que mueve dinero. TransferFeeConfig es estado a nivel de mint con dos autoridades y dos esquemas de comisión adentro. La comisión en sí son dos números: `transfer_fee_basis_points`, el porcentaje en centésimas de punto porcentual, y `maximum_fee`, un tope duro en unidades base. En cada transferencia el programa calcula:

fee = ceil(amount x basis_points / 10000), con tope en maximum_fee

La dirección del redondeo no es un detalle. La comisión redondea hacia ARRIBA, siempre, así que el programa nunca se cobra de menos a sí mismo: envía 1,001 unidades base a 100 bps y la matemática cruda dice 10.01, el programa retiene 11. Y el tope es un techo sobre la recaudación absoluta: a 100 bps con un tope de 5 tokens, una venta de 250 tokens debe 2.5 y paga 2.5, mientras que una venta de 750 tokens debe 7.5 y paga exactamente 5. Las ballenas reciben un descuento, por diseño, porque el tope existe para que las transferencias grandes y legítimas no se desangren.

![La fórmula de Token-2022 para la comisión de transferencia, división por techo con tope en maximumFee, mostrada como TypeScript anotado y con el ejemplo de redondeo de 1,001-unidades-retiene-11 destacado.](assets/v01-annotated-code.png)

Dos formas más dentro de la extensión merecen una mirada antes de construir, porque la librería cliente te va a obligar a reconocerlas de todos modos. Primero, las autoridades están divididas: `transfer_fee_config_authority` puede cambiar la comisión, y `withdraw_withheld_authority` puede recaudarla. Dos claves, dos trabajos, separables a propósito: una DAO puede tener el poder de fijar la comisión mientras un bot de ops tiene el poder de barrer, y revocar una no revoca la otra. Y la revocación acá no es simétrica en sus consecuencias. Pon la autoridad de config en none y el esquema de comisión queda congelado para siempre, que es una característica de compromiso creíble: los tenedores saben que el 1% nunca puede volverse 10%. Pon la autoridad de withdraw en none y cada comisión que tu token retenga alguna vez, pasada y futura, queda varada permanentemente, porque hacer harvest hacia el mint sigue siendo sin permiso pero ya nada puede volver a sacar la pila. Una de esas revocaciones es una promesa. La otra es una lápida. Decide qué autoridad va a un multisig, cuál a una clave de ops, y cuál anularías alguna vez, antes de la inicialización, porque esta extensión ya viene de nacimiento y el cableado de autoridades es parte del producto.

![Tabla de los cuatro asientos de autoridad, sus poderes y los resultados de revocarlos, desde un esquema de comisión congelado hasta comisiones retenidas varadas permanentemente.](assets/v02-table.png)

Segundo, la config guarda DOS esquemas de comisión completos, uno más viejo y uno más nuevo. Un cambio de comisión no toma efecto cuando lo firmas; `set_transfer_fee` arma el esquema más nuevo para un epoch futuro, y cada transferencia elige el esquema que corresponde al epoch en el que se ejecuta. A nadie le hacen un rug a mitad de vuelo por una comisión que se duplicó entre la vista previa de la billetera y la confirmación, y un integrador que cotiza comisiones puede leer los dos esquemas y saber exactamente cuál aplica y cuándo. Cuando construyas la forma de la extensión en el lab y los tipos te exijan `olderTransferFee` Y `newerTransferFee`, eso no es boilerplate, eso es el mecanismo anti-rug devolviéndote la mirada.

![La autoridad firma set_transfer_fee a mitad de epoch, el esquema de 200 bps se arma en un límite de epoch futuro, y las transferencias siguen cobrando 100 bps hasta que ese límite pasa.](assets/v03-timeline.png)

Así que una transferencia se dispara y una comisión se calcula. ¿Adónde va?

### La revelación: las comisiones viven en el destinatario

Ni cerca de tu tesorería. La comisión se retiene EN la cuenta de token del destinatario, dentro de un slot con el que la cuenta nació. Conociste el mecanismo la lección pasada en `required_init_account_extensions`: TransferFeeConfig en un mint fuerza una extensión TransferFeeAmount en cada cuenta de token de ese mint. Ese slot es donde se acumulan las comisiones, por tenedor, de forma invisible, para siempre, hasta que alguien las barre.

Envía 250 SPROUT a 100 bps y la cuenta del comprador recibe 247.5 SPROUT gastables más 2.5 SPROUT de comisiones retenidas que no puede tocar. Corre diez ventas a diez compradores distintos y los ingresos de tu protocolo quedan desparramados en diez cuentas como polvo retenido. Nada se enruta automáticamente. No hay ninguna tesorería en el cuadro hasta que tú metas una.

El slot forzado también tiene su etiqueta de precio, y la paga cada tenedor, no tú. Cada cuenta de token de un mint con comisión pesa unas cuantas docenas de bytes más que una común, y los bytes de cuenta son rent: cada tenedor nuevo paga un mínimo exento de renta un poco más alto al crear la cuenta, para siempre, acumule o no alguna vez una comisión retenida. En una sola cuenta es ruido. En un token con cien mil tenedores es un impuesto permanente sobre toda tu base de usuarios que tú les firmaste en la inicialización del mint. Esta es la misma lección que la matemática de bytes de m01-l2 enseñó desde el lado de la lectura, ahora desde el lado de la emisión: las extensiones no son gratis para la gente que apenas tiene tu token.

![Diagrama de una venta de 250 SPROUT con una comisión del 1%: la cuenta del comprador tiene 247.5 gastables más 2.5 retenidos en su slot TransferFeeAmount, mientras la tesorería queda vacía y sin participar.](assets/v04-diagram.png)

¿Por qué construirlo así? Derívalo de lo que ya sabes del runtime en vez de tomarlo como una rareza. Supón que las comisiones se enrutaran en línea hacia una tesorería. Entonces cada transferencia del token necesitaría la cuenta de tesorería en su lista de cuentas, escribible. Una sola cuenta caliente y escribible compartida por cada transferencia significa que dos transferencias de tu token no pueden ejecutarse en paralelo, nunca: habrías serializado toda tu economía de token a través de un solo candado. Peor aún, la forma de cuentas que pide la instrucción de transferencia cambiaría cada vez que la tesorería se mudara. Retener en el destinatario mantiene cada transferencia tocando solo las cuentas que ya tocaba, así que la ejecución en paralelo sobrevive, y convierte el enrutamiento de comisiones en lo que honestamente es: un trabajo por lotes asíncrono. El protocolo no hace el trabajo por ti. Solo hace que el trabajo sea posible, y barato.

Ese trabajo por lotes tiene un nombre, tres nombres en realidad, y son el vocabulario de trabajo de esta lección. `harvest_withheld_tokens_to_mint` barre las comisiones retenidas desde cualquier lista de cuentas de token hacia el mint mismo, a un campo `withheld_amount` dentro del propio TransferFeeConfig del mint. Es sin permiso: cualquiera puede llamarla, porque solo consolida, no puede robar. `withdraw_withheld_tokens_from_mint` después mueve la pila consolidada desde el mint hacia cualquier cuenta de token de destino, y ESTA está restringida al `withdraw_withheld_authority`. También hay una ruta directa, `withdraw_withheld_tokens_from_accounts`, que extrae desde cuentas de token directo a un destino en un solo salto restringido por autoridad, útil cuando quieres las comisiones fuera de cuentas específicas sin la escala en el mint.

![Diagrama de flujo del ciclo de vida de la comisión: comisiones retenidas en las cuentas de los destinatarios, harvest sin permiso hacia el mint, withdraw restringido por autoridad hacia una tesorería, más una ruta directa de cuentas a destino restringida por autoridad.](assets/v05-flowchart.png)

Ahora la consecuencia operativa, porque esta es la mitad trade-off del trato. Una comisión de transferencia hace que un token se autofinancie, que es una capacidad real: el riel de enrutamiento de comisiones que construyes en el Módulo 9 convierte exactamente esta mecánica en buybacks automatizados. La factura por eso: los destinatarios reciben menos de lo que envió el remitente, lo que rompe cada integración que asumió que monto-de-entrada es igual a monto-de-salida; las comisiones se acumulan de forma invisible a lo largo de miles de cuentas; y hacer harvest es un trabajo de ops recurrente, con costos de CU reales, que tu protocolo ahora tiene de por vida. Quién llama a harvest, con qué frecuencia, y quién paga esas transacciones es una pregunta de personal, no una pregunta de código. La mayoría de los postmortems de tokens con comisión no son exploits. Son nadie-corrió-el-cron.

La ruptura de integración merece su propia escena concreta, porque tarde o temprano vas a estar de un lado de ella. Un exchange acredita depósitos por el monto que el remitente afirma haber enviado, envía 100 tokens de un mint con comisión a un retiro de usuario, y el usuario recibe 99. Ahora el libro contable interno del exchange y la blockchain difieren en un token por retiro, acumulándose, y los tickets de soporte hacen la contabilidad. Los patrones defensivos son aburridos y obligatorios: acredita lo que LLEGÓ, nunca lo que se envió (lee el delta de saldo del destino, no el monto de la instrucción); cotiza las comisiones a los usuarios antes de que firmen, usando la fórmula exacta de arriba contra el esquema del epoch ACTUAL; y para los remitentes que prometen un monto recibido exacto, hazle gross-up al envío para que la llegada después de la comisión coincida con la promesa, recordando que el techo redondea en tu contra. El gross-up es lo bastante delicado como para escribirlo una vez y guardarlo:

```ts
// gross-up: smallest send amount whose post-fee arrival covers `target`.
function grossUp(target: bigint, basisPoints: number, maximumFee: bigint): bigint {
  if (basisPoints === 0 || target === 0n) return target;
  let amount = (target * 10_000n) / (10_000n - BigInt(basisPoints));
  while (amount - transferFee(amount, basisPoints, maximumFee) < target) amount += 1n;
  return amount;
}
// netting exactly 100 SPROUT at 100 bps means sending 101.010102
```

La forma cerrada te deja a menos de una unidad y el loop absorbe el sesgo del techo; prometerle a un usuario "vas a recibir exactamente X" sin esto es la forma en que nacen las colas de soporte. Nada de esto es difícil. Todo tiene que hacerse adrede, y las integraciones anteriores a Token-2022 no hacen nada de eso por defecto, que es gran parte de por qué las plataformas siquiera restringen el acceso a los tokens con comisión detrás de allowlists.

El crank mismo, la rutina periódica que alguien tiene que correr, tiene margen de diseño que conviene conocer antes de que el Módulo 9 lo automatice. `harvest_withheld_tokens_to_mint` toma un array `sources` entero, así que una sola instrucción barre muchas cuentas sucias de una vez, y mi costo medido para un harvest de una sola fuente anduvo cerca de 1,200 CU: consolidar es casi gratis, que es exactamente lo que quieres para una llamada sin permiso. Ese carácter sin permiso también es un pequeño regalo al ecosistema: un indexador, un bot, hasta un rival puede acomodar tus comisiones hacia el mint, y no se pierde nada porque solo la autoridad de withdraw puede dar el salto final. La ruta directa, `withdraw_withheld_tokens_from_accounts`, renuncia a esa división del trabajo: una sola llamada restringida por la autoridad, pero la autoridad tiene que firmar cada barrida y la lista de cuentas viaja en una transacción que ella misma paga. Dos tramos para la recaudación de rutina a escala, directo para extracciones quirúrgicas. En cualquier caso el problema de encontrar-las-cuentas-sucias es tuyo, y es un problema de indexación: enumera las cuentas de token del mint, filtra por un monto retenido distinto de cero, barre. En el Módulo 9, la lección de enrutamiento de comisiones es donde este curso hace esa enumeración como se debe, camino a automatizar el crank.

### Las dos extensiones de display, y por qué no pueden coexistir

Ahora las habladoras. InterestBearingConfig guarda una tasa en basis points (un i16, así que puede ser negativa: sí, puedes configurar decaimiento) más timestamps, e instruye a los clientes a mostrar los saldos como si se hubieran compuesto continuamente desde la inicialización. El saldo de UI de un tenedor deriva hacia arriba día tras día. El monto crudo en su cuenta no se mueve. No se acuña ningún token, esta extensión nunca va a acuñar ninguno, y en el momento en que cualquier código trate ese número de display creciente como supply está contando dos veces un valor que no existe. La extensión es una convención contable con un ancla on-chain: útil para bonos y wrappers que devengan rendimiento donde el valor nominal del pagaré crece, peligrosa en el instante en que alguien conecta `ui_amount` a la matemática de liquidación. La blockchain incluso ya trae una instrucción `amount_to_ui_amount` que puedes simular para sacar el valor mostrado que manda, que es la manera educada de decir: la conversión la define el programa, no tu planilla.

La tasa ni siquiera está bloqueada: la `rate_authority` puede cambiarla, y los campos de estado de aspecto raro de la extensión existen exactamente para ese momento. Cuando llega una actualización de tasa, el programa pliega todo lo acumulado hasta ahí dentro de `pre_update_average_rate` y estampa `last_update_timestamp`, así que la matemática de display se vuelve por tramos: el promedio viejo aplica hasta la estampa, el `current_rate` nuevo aplica después. La historia no se reescribe cuando la tasa sí, que es la diferencia entre una perilla de rendimiento y una máquina del tiempo. Cuando la llamada `extension()` del lab te pide los cinco campos, ese registro por tramos es lo que estás inicializando.

Acá está el doble conteo en la vida real, para que deje de ser abstracto. Un protocolo de préstamos lista un token que devenga interés como colateral y, para ahorrarse una llamada, valúa posiciones por el monto que muestra la billetera. El número mostrado se compone; los tokens crudos que lo respaldan no. Mes a mes los libros del protocolo hacen crecer colateral fantasma, precisamente la brecha entre la matemática de display y la realidad, y la primera cascada de liquidaciones lo marca a mercado de golpe. No hackearon nada. Alguien leyó una convención de UI como un saldo. La regla que te mantiene a salvo es mecánica: los montos crudos liquidan, los montos de UI renderizan, y cualquier número que cruce del segundo mundo al primero tiene que pasar por la conversión del propio programa, en un timestamp que elegiste a conciencia.

![Una línea plana de monto crudo diverge de una curva creciente de monto mostrado en un mint que devenga 5% de interés, con amount_to_ui_amount como el único puente autorizado entre las dos.](assets/v06-diagram.png)

ScaledUiAmount es el mismo truco con otra forma: un único multiplicador f64 aplicado a cada saldo mostrado, actualizable por su autoridad con un timestamp efectivo que puedes programar por adelantado. Donde InterestBearingConfig modela deriva continua, ScaledUiAmount modela saltos discretos: un split de 10 por 1, un rebase, una redenominación, todo sin tocar una sola cuenta de tenedor. Una instrucción actualiza el multiplicador y cada billetera del planeta vuelve a renderizar. La programación temporal es la mitad subestimada: un emisor puede anunciar un martes que el split toma efecto el lunes a las 00:00 UTC, firmar la actualización de inmediato con ese timestamp efectivo, y cada cliente da el vuelco en el mismo instante sin ventana de migración, sin snapshot, sin flujo de reclamo. Para emisores que de otro modo tendrían que migrar miles de cuentas para cambiar una denominación, esta es la salida barata. Aplica la misma disciplina que con el interés: el multiplicador reescala la historia, no el supply, y todo lo que liquide tiene que liquidar en crudo.

Y las dos no pueden compartir un mint: eso lo derivaste tú mismo la lección pasada como regla 4 de `check_for_invalid_mint_extension_combinations`, la única exclusión mutua de verdad en la matriz. Las dos extensiones reclaman la propiedad de la misma salida, el monto mostrado, y dos dueños de un número sin orden de composición definido es una ambigüedad que el programa se niega a crear. Tu check-combo ya rechaza el par; hoy se gradúa de fixture de prueba a barrera previa al vuelo, porque SPROUT se lleva exactamente una de estas y el validador es lo que te frena de inicializar las dos sin darte cuenta.

![Comparación entre la deriva continua de display de InterestBearingConfig y los saltos escalonados de ScaledUiAmount, sin que ninguna acuñe supply y con la regla 4 permitiendo que un mint lleve solo una.](assets/v07-comparison.png)

¿Cuál se lleva SPROUT? Tu decisión, en serio: el lab construye InterestBearingConfig en el camino trabajado porque una cooperativa de cultivo que paga rendimiento sobre el grano guardado es el calce más natural, y el cambio a ScaledUiAmount son dos líneas que te voy a mostrar al final. Elijas la que elijas, el validador de combos bendice TransferFeeConfig más tu elección, y te habría frenado de llevarte las dos.

Un momento de realidad de mercado antes del lab, porque responde "¿alguien va siquiera a negociar esto?". La página de Raydium sobre el soporte de Token-2022 dice en voz alta la parte que se calla: su implementación de referencia pone en la whitelist exactamente las extensiones de comisión, de display y de contabilidad, TransferFeeConfig, MetadataPointer, TokenMetadata, InterestBearingConfig, ScaledUiAmount, y rechaza todo lo demás, incluidos los poderes con forma de compliance (docs.raydium.io, 2026-08-21). Cada extensión que SPROUT se lleva de esta lección está en esa allowlist por diseño, y el razonamiento es exactamente la división con la que abrió esta lección: una comisión o un multiplicador de display no pueden barrer el vault de un pool. Las extensiones que pueden actuar sobre los saldos de otra gente son las que las plataformas rechazan, y esa historia, legal-pero-no-enrutable, es el argumento de apertura del Módulo 5.

Y si te estás preguntando por qué tu tutorial favorito nunca mencionó dos de las tres extensiones de hoy: la educación oficial de Solana se congeló a mitad de la trama. El repo solana-foundation/developer-content fue archivado el 2025-01-24, así que cada curso construido desde ese canon es anterior a ScaledUiAmount, Pausable y ConfidentialMintBurn. Las extensiones que estás por inicializar literalmente no existen en la mayor parte del material desde el que el ecosistema todavía enseña. Estás aprendiendo del programa porque, para esta capa, el programa es hoy por hoy el único maestro que se mantuvo al día.

## Lab: construye el conjunto de economía de SPROUT

El artefacto es `sprout-mint-economics`: un mint Token-2022 con TransferFeeConfig más InterestBearingConfig (o ScaledUiAmount), una corrida de transferencias que desparrama comisiones retenidas, y un harvest que las barre hacia una tesorería y demuestra la aritmética. Consume las dos herramientas que ya tienes: check-combo le pone la barrera al conjunto antes de que se mueva un solo lamport, y decode-mint inspecciona el resultado después.

1. Instala la toolchain en la carpeta `labs/m02-l1` que armaste. Los pins necesitan un minuto de honestidad. El kit latest de npm es 8.2.0 (publicado el 2026-08-29), pero "latest" no es la regla que decide un pin: fijas el major de kit contra el que hacen peer los clientes `@solana-program/*` de tu workspace, y los clientes de este workspace hacen peer con kit ^7, verificado contra los rangos de peer reales de npm el 2026-09-05. `@solana-program/token-2022@0.15.0` es el minor actual que hace peer con kit ^7.0.0 (0.16.0 saltó a ^8), y `@solana-program/system@0.13.0` es su contraparte (0.14.0 también saltó). Ese minor de token-2022 también hace peer con `@solana/sysvars` en ^7.0.0 y con `@solana/zk-sdk` en ^0.5.1, que npm te resuelve junto con el pin de kit, así que nada más necesita un pin a mano. Vuelve a verificar con `npm view <pkg> peerDependencies` el día que lo armes; esta matriz se mueve.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.20.5 typescript@5.9.3
```

2. Ponle la barrera al conjunto de extensiones antes de construir nada. Este es el primer día de check-combo en el trabajo para el que fue construido. Crea `gate.ts`:

```ts
// gate.ts: no SPROUT instruction is emitted until the set passes R2.
import { checkCombo } from "../m01-l4/check-combo";

const chosen = ["TransferFeeConfig", "InterestBearingConfig"];
const verdict = checkCombo(chosen);
if (!verdict.valid) {
  console.error(`illegal set: ${verdict.reason}`);
  process.exit(1);
}
console.log(`set [${chosen.join(", ")}] is legal to initialize`);

// The pairing the matrix forbids, proven rejected before we ever hit the chain:
const illegal = checkCombo(["TransferFeeConfig", "ScaledUiAmount", "InterestBearingConfig"]);
console.log(`both display extensions: ${illegal.valid ? "BUG in your R2" : illegal.reason}`);
```

Corre `npx tsx gate.ts`. El conjunto legal pasa, el conjunto de doble display es rechazado con la razón de la regla 4, y acabas de usar una cosa que construiste para proteger una cosa que estás por construir. Ese loop es la escalera de artefactos funcionando.

3. Ahora la construcción. Crea `verify-economics.ts`. Es la lección entera en un solo archivo y te la voy a dar en tres trozos; escríbelos en el mismo archivo, en orden. El trozo uno es el setup: imports, constantes, la fórmula de comisión, y un helper de send que simula antes de enviar. Acá viven dos TODOs y son tuyos: la sección de teoría ya te dijo que SPROUT cobra 100 bps con un tope de 5 SPROUT.

```ts
// verify-economics.ts: SPROUT's economics layer, built from raw instructions.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  sendAndConfirmTransactionFactory,
  airdropFactory,
  lamports,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  getBase64EncodedWireTransaction,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  extension,
  getMintSize,
  getInitializeTransferFeeConfigInstruction,
  getInitializeInterestBearingMintInstruction,
  getInitializeMintInstruction,
  getCreateAssociatedTokenInstructionAsync,
  findAssociatedTokenPda,
  getMintToInstruction,
  getTransferCheckedInstruction,
  getHarvestWithheldTokensToMintInstruction,
  getWithdrawWithheldTokensFromMintInstruction,
  fetchToken,
  fetchMint,
} from "@solana-program/token-2022";

const rpc = createSolanaRpc("http://127.0.0.1:8899");
const rpcSubscriptions = createSolanaRpcSubscriptions("ws://127.0.0.1:8900");
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
const airdrop = airdropFactory({ rpc, rpcSubscriptions });

const DECIMALS = 6;
const FEE_BASIS_POINTS: number = 0; // TODO: SPROUT charges 1% on every transfer
const MAXIMUM_FEE: bigint = 0n; // TODO: capped at 5 SPROUT, expressed in base units

// Guard against the vacuous green run: with both constants at their shipped
// zeros every fee is 0n, expectedWithheld is 0, and all three headline
// assertions "pass" on a lab that never charged a fee. Fail loudly instead.
if (FEE_BASIS_POINTS === 0 || MAXIMUM_FEE === 0n) {
  throw new Error("fill in FEE_BASIS_POINTS and MAXIMUM_FEE first: a zero-fee run passes every assertion without proving anything");
}

// The on-chain formula, mirrored so the lab can assert against it.
export function transferFee(amount: bigint, basisPoints: number, maximumFee: bigint): bigint {
  if (basisPoints === 0 || amount === 0n) return 0n;
  const raw = (amount * BigInt(basisPoints) + 9_999n) / 10_000n;
  return raw < maximumFee ? raw : maximumFee;
}

async function send(feePayer: KeyPairSigner, instructions: Instruction[]) {
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(feePayer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(instructions, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  // No per-extension CU table exists anywhere. So we ask the cluster, every time.
  const sim = await rpc
    .simulateTransaction(getBase64EncodedWireTransaction(signed), { encoding: "base64" })
    .send();
  console.log(`  simulated CU: ${sim.value.unitsConsumed}`);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
}
```

Esa llamada a `simulateTransaction` dentro del helper de send es la política de medición de la lección hecha ejecutable. Congelar un número de CU por extensión sacado de un blog sería exactamente el error del mapa copiado que m01-l4 se pasó una sección entera derribando; los números de abajo son de MI corrida, en MI surfnet, y el helper existe para que cada corrida tuya imprima los tuyos.

4. El trozo dos, la construcción y las ventas, agregado al mismo archivo. Mira el ORDEN de las instrucciones dentro de la transacción del mint, porque es estructural: los inicializadores de extensión corren contra la cuenta ya asignada ANTES de `initialize_mint`, y el programa rechaza cualquier otra disposición. La mayoría de las extensiones de Token-2022 tienen que habilitarse en la creación del mint y no pueden agregarse después de la inicialización, así que esta transacción es la única y exclusiva oportunidad de SPROUT para llevar este conjunto. Fíjate también en lo que la forma de `extension()` te obliga a poner: el estado completo de TransferFeeConfig, incluidos los dos epochs de esquema de comisión, porque `getMintSize` no puede ponerle precio a una cuenta sin el layout real.

```ts
async function main() {
  const payer = await generateKeyPairSigner();
  await airdrop({
    recipientAddress: payer.address,
    lamports: lamports(2_000_000_000n),
    commitment: "confirmed",
  });
  const mint = await generateKeyPairSigner();
  const buyer = await generateKeyPairSigner();

  const feeSchedule = {
    epoch: 0n,
    maximumFee: MAXIMUM_FEE,
    transferFeeBasisPoints: FEE_BASIS_POINTS,
  };
  const transferFeeExtension = extension("TransferFeeConfig", {
    transferFeeConfigAuthority: payer.address,
    withdrawWithheldAuthority: payer.address,
    withheldAmount: 0n,
    olderTransferFee: feeSchedule, // two schedules: the epoch-armed anti-rug
    newerTransferFee: feeSchedule,
  });
  const interestExtension = extension("InterestBearingConfig", {
    rateAuthority: payer.address,
    initializationTimestamp: 0n,
    preUpdateAverageRate: 500,
    lastUpdateTimestamp: 0n,
    currentRate: 500, // 5% APR. Display only. Supply never moves.
  });

  const space = BigInt(getMintSize([transferFeeExtension, interestExtension]));
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();
  console.log(`mint account: ${space} bytes, rent ${rent} lamports`);

  console.log("create + init mint:");
  await send(payer, [
    getCreateAccountInstruction({
      payer,
      newAccount: mint,
      lamports: rent,
      space,
      programAddress: TOKEN_2022_PROGRAM_ADDRESS,
    }),
    // Extension initializers BEFORE initialize_mint. The order is the protocol.
    getInitializeTransferFeeConfigInstruction({
      mint: mint.address,
      transferFeeConfigAuthority: payer.address,
      withdrawWithheldAuthority: payer.address,
      transferFeeBasisPoints: FEE_BASIS_POINTS,
      maximumFee: MAXIMUM_FEE,
    }),
    getInitializeInterestBearingMintInstruction({
      mint: mint.address,
      rateAuthority: payer.address,
      rate: 500,
    }),
    getInitializeMintInstruction({
      mint: mint.address,
      decimals: DECIMALS,
      mintAuthority: payer.address,
    }),
  ]);

  const [sellerAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: payer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  const [buyerAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: buyer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  console.log("create ATAs + mint supply:");
  await send(payer, [
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: payer.address }),
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: buyer.address }),
    getMintToInstruction({
      mint: mint.address,
      token: sellerAta,
      mintAuthority: payer,
      amount: 1_000_000_000n, // 1,000 SPROUT
    }),
  ]);

  // Ten sales of 25 SPROUT. Accumulate what the formula SAYS should be withheld.
  const SALE = 25_000_000n;
  let expectedWithheld = 0n;
  const sales: Instruction[] = [];
  for (let i = 0; i < 10; i++) {
    sales.push(
      getTransferCheckedInstruction({
        source: sellerAta,
        mint: mint.address,
        destination: buyerAta,
        authority: payer,
        amount: SALE,
        decimals: DECIMALS,
      }),
    );
    expectedWithheld += transferFee(SALE, FEE_BASIS_POINTS, MAXIMUM_FEE);
  }
  console.log("ten transfers:");
  await send(payer, sales);

  // The reveal, in data: the fees are on the BUYER's account.
  const buyerToken = await fetchToken(rpc, buyerAta);
  const ext =
    buyerToken.data.extensions.__option === "Some"
      ? buyerToken.data.extensions.value.find((e) => e.__kind === "TransferFeeAmount")
      : undefined;
  const withheld = ext?.__kind === "TransferFeeAmount" ? ext.withheldAmount : 0n;
  console.log(`withheld on buyer account: ${withheld} (expected ${expectedWithheld})`);
  if (withheld !== expectedWithheld) throw new Error("withheld mismatch: check your fee constants");
```

Una arruga que vale la pena nombrar mientras escribes: esas son instrucciones `transfer_checked` comunes. En un mint con comisión el programa calcula y retiene la comisión por su cuenta; no optas por ella transferencia por transferencia. También hay una variante `transfer_checked_with_fee` que lleva TU comisión esperada y hace fallar la transferencia si el programa no está de acuerdo, que es la jugada de cinturón y tirantes para los emisores de producción que ya le cotizaron una comisión a un usuario. Nosotros en cambio hacemos assert después del hecho, porque el punto de este lab es atrapar al programa con las manos en la masa.

Una simplificación sincera: las diez ventas de acá llegan a una sola ATA de comprador, así que la pila retenida queda en un solo lugar y el assert se mantiene en una línea. La escena de la teoría de las diez-cuentas-de-polvo-desparramado es real, pero te la topas en el challenge, donde tres compradores te obligan a enumerar y barrer varias cuentas sucias en un solo array `sources`.

5. El trozo tres es el harvest, y esta parte es el ejercicio de completar. El apoyo de abajo cierra `main()`; los dos sitios TODO son tuyos. Todo lo que necesitas está en la teoría: el tramo uno es la barrida sin permiso desde las cuentas de token hacia el mint, el tramo dos es la extracción restringida por la autoridad desde el mint hacia la tesorería. Los dos builders de instrucciones ya están en tu lista de imports, y sus entradas son exactamente las cuentas que están en el scope. Llénalos.

```ts
  // Leg 1: sweep withheld fees from token accounts onto the mint. Permissionless.
  console.log("harvest accounts -> mint:");
  await send(payer, [
    // TODO: getHarvestWithheldTokensToMintInstruction. It wants the mint and a
    // `sources` array of token accounts to sweep. There is exactly one dirty
    // account in this lab so far. Which one holds the withheld fees?
  ]);

  const mintAfter = await fetchMint(rpc, mint.address);
  const mintExt =
    mintAfter.data.extensions.__option === "Some"
      ? mintAfter.data.extensions.value.find((e) => e.__kind === "TransferFeeConfig")
      : undefined;
  const onMint = mintExt?.__kind === "TransferFeeConfig" ? mintExt.withheldAmount : 0n;
  console.log(`withheld on mint after harvest: ${onMint}`);

  // Leg 2: pull the consolidated pile from the mint to the treasury. Gated.
  const treasury = await generateKeyPairSigner();
  const [treasuryAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: treasury.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  console.log("withdraw mint -> treasury:");
  await send(payer, [
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: treasury.address }),
    // TODO: getWithdrawWithheldTokensFromMintInstruction. It wants the mint, a
    // `feeReceiver` token account, and the withdrawWithheldAuthority as a SIGNER.
    // We set that authority during initialization. Who was it?
  ]);

  const treasuryToken = await fetchToken(rpc, treasuryAta);
  console.log(`treasury balance: ${treasuryToken.data.amount} (expected ${expectedWithheld})`);
  if (treasuryToken.data.amount !== expectedWithheld) {
    throw new Error("treasury balance does not equal summed fees");
  }
  console.log(`SPROUT economics mint: ${mint.address}`);
  console.log("economics lab: all assertions passed");
}

await main();
```

6. Llena los dos TODOs de constantes del paso 3 (la sección de teoría dijo los dos valores en palabras llanas) y los dos TODOs de harvest, y después corre la barrera:

```bash
npx tsx verify-economics.ts
```

Mi corrida, para calibrar (surfnet de surfpool 1.2.1 sobre solana-core 3.1.10, 2026-08-22; la tuya va a derivar y ese es el punto de medir). Dos de estos números incluso derivan entre corridas en la MISMA máquina: los pasos que crean cuentas cayeron en cualquier punto entre 36,298 y 37,798 CU y el withdraw de tesorería entre 18,731 y 21,731, dependiendo de lo que ya existiera en el surfnet. Los números del mint, de la transferencia y del harvest se reprodujeron a la unidad todas las veces:

```
mint account: 334 bytes, rent 3215520 lamports
create + init mint:
  simulated CU: 4332
create ATAs + mint supply:
  simulated CU: 37798
ten transfers:
  simulated CU: 32470
withheld on buyer account: 2500000 (expected 2500000)
harvest accounts -> mint:
  simulated CU: 1207
withheld on mint after harvest: 2500000
withdraw mint -> treasury:
  simulated CU: 18731
treasury balance: 2500000 (expected 2500000)
SPROUT economics mint: HfVB99cPQEPGE1vPgfy3a2ynJVK552UW1SrEGt5fFsFf
economics lab: all assertions passed
```

Lee los recibos. Diez ventas de 25 SPROUT a 100 bps son 250,000 unidades base retenidas por venta, 2,500,000 en total, y ahí está: primero varado en la cuenta del comprador, después consolidado en el mint, después recibido en la tesorería, hasta la unidad base. Y la matemática de la cuenta cuadra con m01-l2: un mint con solo TransferFeeConfig es de 278 bytes, e InterestBearingConfig agrega su estado de 52 bytes más un header TLV de 4 bytes para dar 334.

![Gráfico de barras de las unidades de cómputo medidas para las cinco transacciones del lab en orden de ciclo de vida, desde la creación del mint pasando por las transferencias con comisión y el harvest hasta el withdraw de tesorería.](assets/v08-chart.png)

¿Por qué medir en vez de memorizar? Porque cada número de ese gráfico es función de cosas que se mueven: la versión del programa desplegada en tu cluster, el conjunto de features activo ahí, cuántas extensiones llevan tus cuentas, si la ATA ya existe. Una transferencia con comisión en SPROUT cuesta alrededor de 3,200 CU en mi surfnet hoy; en mainnet el trimestre que viene, después del próximo deploy del programa, va a costar otra cosa. El hábito que este curso no para de machacar, desde que la lección de p-token bajó una transferencia de 4,645 a 76 CU de un día para otro, es que los costos son hechos del cluster, no hechos de la documentación. Tu helper de send imprime la verdad gratis en cada corrida. Déjalo.

7. Cierra el loop con decode-mint. Tu inspector de m01-l2 lee el conjunto TLV de cualquier mint desde bytes crudos; apúntalo a la dirección de SPROUT que imprimió tu corrida (ajusta la ruta de import, el nombre de export y el objetivo de RPC a tu propio archivo decode-mint, y apúntalo al surfnet, `http://127.0.0.1:8899`, como sea que tu herramienta reciba un cluster):

```ts
// inspect-economics.ts: R1 reads what this lesson built.
import { decodeMint } from "../m01-l2/decode-mint";

const decoded = await decodeMint(process.argv[2]);
console.log(decoded.extensions.map((e) => e.name));
```

Esperado: `TransferFeeConfig` e `InterestBearingConfig` (o `ScaledUiAmount` si te llevaste el otro asiento), y esta vez puedes nombrar cada byte de los dos. El inspector que desmitificó PYUSD en el Módulo 1 ahora audita un mint que escribiste tú.

8. El cambio a ScaledUiAmount, si ese es tu asiento, son exactamente dos ediciones, más una trampa de nombres con la que me topé para que tú no tengas que hacerlo. Para el cálculo de tamaño, la variante de `extension()` se llama `ScaledUiAmountConfig` (el cliente nombra el ESTADO, mientras que el builder de instrucciones nombra la operación, y pasarle `"ScaledUiAmount"` a `extension()` lanza un error de variante inválida antes de que nada llegue a la blockchain):

```ts
const scaledExtension = extension("ScaledUiAmountConfig", {
  authority: payer.address,
  multiplier: 1,
  newMultiplierEffectiveTimestamp: 0n,
  newMultiplier: 1,
});
```

Después reemplaza el inicializador de interés por `getInitializeScaledUiAmountMintInstruction({ mint: mint.address, authority: payer.address, multiplier: 1 })`. Todo lo demás, comisiones incluidas, queda intacto. Corre la barrera primero: check-combo bendice TransferFeeConfig más cualquiera de las dos extensiones de display por separado, y rechaza las dos juntas, que es precisamente por lo que existe el paso 2.

## Challenge

El trabajo en solitario, sin apoyo a la vista.

**El coding challenge: `transferFee`, exacto.** El challenge fee-calculator del módulo te entrega un starter que hace floor de la división y se olvida del tope, más un conjunto de pruebas sobre flujos de transferencias. Implementa la matemática de comisión de Token-2022: división por techo, tope de `maximumFee`, casos cero por delante, sobre bigints. El calificador invoca tu función directamente como `transferFee(amount, basisPoints, maximumFee)`, posicional bigint, number, bigint, así que déjala exactamente como la `function transferFee(...)` plana de nivel superior que ya trae el starter, sin `export`, sin imports; el calificador injerta tu archivo en su propio runtime, y la sintaxis de módulos no va a parsear ahí. La vara de aceptación es el comportamiento on-chain hasta la unidad base: las comisiones fraccionarias redondean hacia ARRIBA, la comisión nunca excede el tope, cero bps o monto cero no retiene nada. Ya escribiste esta función una vez dentro del lab con las respuestas enfrente; el challenge es demostrar que puedes reconstruirla desde la fórmula sola, porque el riel de enrutamiento del Módulo 9 va a confiar en tu aritmética para predecir lo que el programa retiene.

**El sondeo empírico.** El lab hizo assert del saldo retenido de un solo comprador. Extiende tu corrida: tres compradores, un flujo de transferencias de tamaños variados, incluyendo al menos una lo bastante grande como para tocar el tope de `maximumFee`. Calcula el monto retenido esperado por cuenta con tu propio `transferFee`, haz harvest de las tres cuentas en un solo array `sources`, y haz assert del total de la tesorería hasta la unidad base. Si tu predicción y el programa no coinciden, uno de los dos está redondeando hacia abajo, y no es el programa.

**La confrontación con la regla 4, esta vez on-chain.** En m01-l4 sondeaste empíricamente un delta entre documentación y código. La misma disciplina, la expectativa opuesta: construye una transacción de mint que lleve LOS DOS, `getInitializeInterestBearingMintInstruction` y `getInitializeScaledUiAmountMintInstruction`, dimensiona la cuenta para los dos, y mándala a tu surfnet. Tu check-combo predice la respuesta del programa antes de que aprietes enter. En mi corrida el programa la entregó en la instrucción `initialize_mint`, error de programa personalizado 0x33 (decimal 51), el `InvalidExtensionCombination` de Token-2022: los dos inicializadores de extensión escriben sus entradas TLV tan contentos, y es `initialize_mint`, la última instrucción, la que corre las cinco reglas sobre el conjunto ensamblado y tira la transacción entera a la basura. Ver una regla que extrajiste de la fuente dispararse de verdad, contra tu propia transacción, es el punto entero de haberla derivado.

## Checkpoint

El criterio de esta lección es el trío de asserts en tu corrida de verificación: retenido-en-el-comprador es igual a tu suma calculada, el harvest lo consolida en el mint, y la tesorería recibe el total exacto. Junto con la corrida que pasa, escribe la respuesta de una sola oración que le darías a un compañero de equipo que pregunta "¿y adónde van nuestras comisiones?". Si tu oración contiene las palabras "hasta que hagamos harvest", tienes la mecánica; si contiene un calendario de cron, tienes el negocio.

Los errores que espero: un desajuste de retenido normalmente quiere decir las constantes de comisión (100 bps es `100`, y 5 SPROUT con 6 decimales es `5_000_000n`, no `5n`); un withdraw fallido normalmente quiere decir que la autoridad que pasaste es una dirección donde el builder quería un signer. Si los números se niegan a reconciliar después de eso, lleva tu flujo de transferencias y tu matemática de comisión esperada a la discusión del curso y encontramos juntos el desacuerdo de redondeo.

Ahora controlas por dónde fluye el valor: SPROUT cobra su parte, y puedes barrer cada unidad retenida hacia la tesorería cuando lo ordenes. Pero nada controla todavía QUIÉN tiene permiso para moverlo, congelarlo o recuperarlo a la fuerza. Lo que sigue: las extensiones de autoridad, incluida la que sin hacer ruido esquiva tus barandas de seguridad.
