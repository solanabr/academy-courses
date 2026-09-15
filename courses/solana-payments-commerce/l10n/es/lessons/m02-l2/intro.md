# El roster de stablecoins de 2026: PYUSD, Token-2022 y cómo viaja USDC

La lección pasada construiste transfer-kit y enviaste USDC real en devnet: unidades base seguras en decimales, un memo, una clave de reference, una firma que podías volver a encontrar. El kit funciona. También tiene una mina adentro, y hoy un cliente la pisa: paga en PYUSD, el kit construye y firma una transacción que se ve perfectamente bien formada, y la red la rechaza en la puerta: el RPC le hace un dry-run a cada transacción antes de transmitirla (un paso llamado simulación de preflight), la simulación le entrega al programa Token clásico un mint que no le pertenece, y el envío vuelve como una promesa rechazada. Ninguna transacción fallida en un explorador, ninguna firma de la que la blockchain haya oído hablar jamás, y ningún dinero movido.

PYUSD es una stablecoin de dólar. Seis decimales, igual que USDC. La misma palabra en la etiqueta. ¿Entonces por qué el código exacto que mueve USDC sin problemas ve su transferencia de PYUSD expulsada en la puerta? Corre esto antes de cualquier teoría. Dos curls, no hace falta billetera:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",{"encoding":"jsonParsed"}]}' \
  | grep -o '"owner":"[^"]*"'
```

Ese es el mint de USDC. Te devuelve `"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"`. Ahora cambia por el mint de PYUSD, `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`, y córrelo de nuevo:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
  | grep -o '"owner":"[^"]*"'
```

`"owner":"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"`. Dirección distinta. Programa distinto. La misma palabra en la etiqueta, una máquina distinta debajo, y tu kit hardcodeó la primera máquina. Ese único campo, `owner`, es todo el bug, y arreglarlo como se debe es esta lección.

## Resumen

Hoy transfer-kit aprende a leer antes de firmar. Primero la teoría: qué quiere decir que un mint sea propiedad de un programa de tokens, qué es Token-2022, y cómo el mint de PYUSD carga ocho extensiones que le dan a su emisor poderes que un comercio necesita conocer, incluido uno que puede sacar PYUSD de cualquier billetera. Después un recorrido rápido por el resto del roster de 2026 que de verdad te van a pedir aceptar: EURC, USDG, USDT y el grupo de los que rinden intereses, que le pasamos a otro curso a propósito. Después CCTP, porque el USDC que te paga muchas veces no nació en Solana y vale la pena saber cómo llegó hasta acá. El lab extiende el kit con `detectTokenProgram`, un reporte `readMint` y un `sendStablecoin` consciente del programa, y entrega una verificación de humo `verify:roster` que demuestra que los dos programas de tokens funcionan de punta a punta.

Cómo se reparte el trabajo hoy: la teoría y casi todo el lab van trabajados, yo escribo primero y tú sigues. El único hueco que dejo en el apoyo, el switch de programa dueño en sí, lo llenas tú como el desafío de completion. La enumeración en vivo de PYUSD y el envío de dos mints al final son solo tuyos, sin guía.

## Mismo ticker, máquina distinta

### El programa dueño: un campo decide cómo se mueve el dinero

Cada cuenta en Solana tiene un campo `owner` que nombra al programa que puede mutarla. Te cruzaste con esta idea de reojo la lección pasada, cuando derivamos ATAs. Ahora se vuelve estructural: un mint es propiedad de exactamente un programa de tokens, y cada instrucción que toca ese mint tiene que ir dirigida a ese programa. No a "el programa de tokens" en abstracto. Al que está en el campo `owner`.

Durante años hubo en la práctica una sola respuesta, el programa Token clásico en `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`, así que toda una generación de código de pagos lo hardcodeó y se salió con la suya. Después llegó Token-2022 en `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`: un segundo programa de tokens, separado, con la misma interfaz central, más un sistema de extensiones que el programa clásico nunca tuvo. No reemplazó al programa clásico. Los dos corren lado a lado, y un mint vive en uno o en el otro, para siempre. USDC es clásico. PYUSD es Token-2022. Los dos son dólares; el dólar no es la máquina.

![Dos tarjetas de mint, USDC y PYUSD, cada una con una flecha de owner hacia un programa de tokens distinto, encima de una regla que dice que las transferencias tienen que apuntar al dueño del mint.](assets/v01-diagram.png)

Así que el arreglo para el kit no es "agregar soporte para PYUSD" sino algo más simple: deja de suponer, empieza a leer. Pregúntale a la blockchain quién es dueño del mint, y después construye la transferencia contra esa respuesta. Una lectura de mint por pago, cacheada si quieres, y desaparece toda la clase de fallas por programa equivocado. El lab convierte esto en una función de cinco líneas, y las cinco líneas importan menos que el hábito: en estos rieles, el mint es el archivo de configuración, y es público. Léelo.

Sé preciso sobre dónde cae esa falla, porque cambia cómo la depuras. Nada lanza mientras construyes. Tu `TransferChecked` de la lección pasada hardcodea el id del programa clásico y deriva cuentas de token debajo de él, y los dos son bytes que se ven válidos, así que el mensaje se construye y se firma sin problemas. El rechazo le pertenece al programa Token clásico: cuando le entregan una cuenta de mint que no le pertenece, revisa el campo owner y da error. Pero con el tramo final de envío del kit nunca llega a decirlo en la blockchain. Antes de transmitir, el RPC pasa tu transacción firmada por un dry-run contra el estado actual —la **simulación de preflight**— y la comprobación de owner se dispara ahí. `sendAndConfirmTransactionFactory` lo expone como una promesa rechazada que lleva `Transaction simulation failed` más el error del programa, y la transacción nunca aterriza: ninguna entrada en el explorador, y `getSignatureStatuses` sobre tu firma devuelve null incluso con la búsqueda en el historial activada. Solo si desactivaras ese dry-run (`skipPreflight`) el mismo rechazo pasaría en la blockchain, te costaría la comisión y dejaría atrás una transacción fallida. El paso 6 del lab te hace pisar la misma mina desde la CLI, que la atrapa en una tercera costura, todavía más temprana.

![Una transferencia al programa equivocado se construye y se firma sin problemas, y después muere en la simulación de preflight del RPC cuando el programa Token clásico encuentra un owner que no coincide — rechazada antes de transmitirse, sin rastro en la blockchain.](assets/v02-flowchart.png)

Molesto en el peor momento, claro, pero este es el buen modo de falla: el programa rechazó en vez de mover dinero mal. Guarda ese instinto mientras avanzamos, porque los rechazos ruidosos son un feature que vas a construir dentro de tu propio kit hoy, esta vez localmente, antes de que se gaste una sola firma.

La división de propiedad alcanza un lugar más que no adivinarías: las direcciones. La línea de seeds de la lección pasada decía que un ATA se calcula a partir del owner y del mint. La verdad completa es que el programa de tokens que es dueño también es parte de la derivación, así que la cuenta de PYUSD de tu cliente y su cuenta de USDC se diferencian no solo por el seed del mint sino por el seed del programa. Deriva el ATA de un mint Token-2022 con el programa clásico en los seeds y obtienes una dirección que se ve perfectamente válida y que ninguna billetera va a fondear nunca. Por eso `resolveAta` gana un tercer parámetro en el lab, y por eso el programa detectado tiene que enhebrarse por cada paso del envío, no solo por la instrucción de transferencia. Una detección, usada en todas partes.

### Extensiones TLV: lo que PYUSD realmente carga

Ahora la mitad interesante. ¿Por qué existe siquiera Token-2022? Porque los emisores seguían necesitando poderes que el programa clásico no podía expresar: comisiones en la transferencia, metadatos en el propio mint, montos confidenciales, controles de cumplimiento. La respuesta de Token-2022 son las extensiones: registros tipados opcionales que se agregan a un mint o a una cuenta de token, codificados como TLV, tipo-longitud-valor. Cada registro dice qué es, qué largo tiene, y después su payload. Un mint opta por un conjunto de extensiones al crearse, y cualquiera puede leer ese conjunto directo de la cuenta.

Una sola decisión de diseño hace posible todo tu lab: las extensiones se agregan después del layout clásico del mint, que se mantiene intacto byte por byte al frente. Por eso un solo decodificador puede leer los dos tipos de mint, y por eso el tooling viejo que solo entiende el prefijo clásico sigue leyendo correctamente el supply y los decimales de un mint Token-2022. Los poderes nuevos viven estrictamente en el apéndice.

PYUSD es el ejemplo trabajado al que apunta todo el ecosistema. Su mint carga ocho extensiones TLV: mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer, tokenMetadata. Ocho es el número que te sale contando entradas TLV; a veces vas a oír siete, contando el par confidencial como una sola suite. Leemos todo esto en vivo en el lab, pero tres de las ocho merecen toda la atención de un comercio ahora mismo.

![El mint de PYUSD dibujado como una tarjeta con el layout clásico arriba y ocho filas de extensiones TLV agregadas, tres de ellas marcadas para la atención del comercio.](assets/v03-diagram.png)

**permanentDelegate** es la que hay que mirar con calma. Nombra una autoridad permanente, aquí el emisor, que puede sacar PYUSD de la cuenta de token de cualquier tenedor. Cualquier billetera, cualquier saldo, ninguna firma del tenedor. Eso es capacidad de incautación, y no es un bug ni un riesgo de hackeo: es política del emisor, la expresión en la blockchain de la obligación que tiene una empresa regulada de congelar y recuperar fondos bajo una orden judicial. Tu código de transferencia no la agrega, no puede quitarla y nunca la dispara. Pero cuando le pones precio a una venta en PYUSD, aceptas un activo cuyo emisor conserva ese poder, y deberías saberlo igual que sabes que tu adquirente de tarjetas puede revertir una liquidación.

**transferFeeConfig** quiere decir que el mint puede cobrar una comisión, en centésimos de punto porcentual, retenida de cada transferencia. Aquí está la trampa de integración: la presencia de la extensión no te dice nada sobre la tasa. La comisión es un número en la cuenta del mint, la autoridad de la comisión puede cambiarlo, y un cambio entra en vigor en un límite de epoch — pero no en el siguiente. Token-2022 estampa la tasa entrante con `newer_fee_start_epoch = current epoch + 2`, así que se activa dos límites de epoch más allá, y la fuente del programa dice por qué con todas sus letras: "set two epochs ahead to avoid rug pulls at the end of an epoch." Una demora de una sola epoch declarada en los últimos minutos de una epoch no sería ningún aviso. Una **epoch** es la unidad de planificación de Solana, un bloque de exactamente 432,000 slots (`getEpochSchedule` te lo va a decir), que al tiempo de slot objetivo de 300ms que este curso usa desde el módulo 1 da alrededor de 36 horas, un día y medio — derívalo en vez de memorizarlo, porque el tiempo de slot es el número que no para de moverse y el material más viejo que cita "unos dos días" está citando la era de 400ms. Después duplícalo, porque dos epochs es lo que realmente te toca: unos tres días de aviso al tiempo de slot de hoy. Esa es la manera que tiene la red de decir "no de inmediato, sino en el próximo relevo acordado", y es la razón por la que un cambio de comisión se anuncia en vez de aplicarse. La comisión de PYUSD está configurada hoy en cero centésimos de punto porcentual. Te lo digo como un hecho sobre hoy, verificado en vivo mientras escribía esto, y el lab hace que tu kit lea el valor actual desde el mint en runtime, porque "actualmente cero" es exactamente el tipo de hecho que nunca hardcodeas. Una comisión que hoy es cero puede ser distinta de cero después, en silencio, sin ningún cambio de código de tu lado.

Y entiende dónde mordería una comisión distinta de cero: se retiene del monto transferido. Envía 100 tokens en un mint con una comisión de 50 centésimos de punto porcentual y a la cuenta del destinatario se le acreditan 99.50; la media unidad retenida se acumula para que la autoridad de la comisión la recaude. Para una tienda eso quiere decir que el precio que muestra tu checkout y el monto que recibe tu libro mayor dejan de coincidir en el momento en que se enciende una comisión, y cada reporte de conciliación aguas abajo hereda esa diferencia. Esa es la razón concreta por la que `readMint` expone los centésimos de punto porcentual como un campo de primera clase: un checkout que conoce la tasa en vivo puede repreciar, advertir o rechazar. Un checkout que supuso cero simplemente pierde margen en silencio.

**transferHook** deja que un mint adjunte un programa que corre en cada transferencia, y que puede agregar cuentas requeridas extra a la instrucción. En PYUSD está configurado pero dormido: la extensión está presente y el id del programa del hook es null, así que las transferencias de hoy no necesitan nada extra. Tu kit va a revisar esto y va a rechazar ruidosamente si alguna vez se encuentra con un mint con un hook activo, porque una transferencia construida sin las cuentas del hook falla de maneras confusas. Construir la interfaz del hook de punta a punta explícitamente no es nuestro trabajo: esa profundidad del lado de la autoría — la interfaz de transfer-hook y el resto de las tripas de las extensiones — es territorio del curso Digital Assets, Tokenization and Token Extensions. Este curso lee y enruta, nada más.

![Una tabla de tres filas con los poderes del emisor: un delegado permanente que puede incautar tokens, una comisión de transferencia cambiable que hoy está en cero centésimos de punto porcentual, y un transfer hook dormido.](assets/v04-comparison.png)

Las cinco restantes, rápido: mintCloseAuthority deja que el emisor cierre la propia cuenta del mint; el par confidencial habilita transferencias con montos cifrados (opt-in, y no es algo que un checkout necesite); metadataPointer y tokenMetadata ponen el nombre y el símbolo del token en la cuenta del mint en vez de en un registro externo. Poderes ordinarios, que vale la pena nombrar, nada sobre lo que una integración de pagos tenga que actuar.

Esta es la disyuntiva honesta sobre la que está construida esta lección. Un kit que habla los dos programas de tokens es más ramificación, más dependencias, más código que el que tenías ayer. Y los mints de Token-2022 pueden cargar poderes que un comercio tiene que aceptar a sabiendas: un delegado permanente quiere decir que el emisor puede recuperar fondos, y una comisión de transferencia se puede encender después. La seguridad no está en evitar Token-2022, que querría decir rechazar PYUSD y la mitad del roster de abajo. La seguridad está en leer el mint en runtime, todas las veces, y en nunca confiar en el ticker. El ticker dice dólar. El mint dice de qué tipo.

Vale preguntarse por qué PayPal se tomó la molestia de toda esta maquinaria. La respuesta es que funcionó: PYUSD llegó a alrededor de $332M de capitalización de mercado dentro de los cuatro meses de su lanzamiento en Solana, con PayPal declarando en Breakpoint 2024 por qué eligieron estos rieles, y el conjunto de extensiones del mint (poder de incautación, hook dormido, capacidad confidencial) es exactamente lo que un emisor regulado necesita para satisfacer a sus reguladores mientras liquida en segundos. Las extensiones son el departamento de cumplimiento, compilado.

![Una línea de tiempo de cuatro puntos desde el lanzamiento de PYUSD en Solana, pasando por alrededor de 332 millones de dólares en capitalización de mercado y PayPal en Breakpoint 2024, hasta la lectura en vivo del mint en 2026.](assets/v05-timeline.png)

### El resto del roster de 2026

A tu checkout le van a pedir más que USDC y PYUSD. Este es el resto del roster, un párrafo honesto cada uno, y el hábito de arriba aplica a cada fila: lee el mint, cree en la lectura.

**EURC** es la stablecoin en euros de Circle, y importa porque poner precios en euros sin una pata de FX es un feature real para una tienda con clientes europeos. En Solana es un mint Token clásico en `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` (fijado aquí desde una lectura en vivo, 2026-08-22), seis decimales. Tu kit, después de hoy, lo maneja con la rama clásica, sin casos especiales. La única novedad está en tus libros, no en la blockchain: es una moneda distinta, no un dólar distinto.

**USDG** es el Global Dollar, emitido por Paxos. Y aquí va un detalle que de verdad disfruté encontrar mientras escribía esto: lee el mint de USDG en `2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH` y te sale Token-2022, seis decimales, y las mismas ocho extensiones que PYUSD, entrada por entrada (verificado en vivo, 2026-08-22). Eso no es una coincidencia, es una huella digital: Paxos es también el emisor detrás de PYUSD, y así se ve la plantilla estándar de un emisor regulado. Dos nombres de marca distintos, una sola forma de máquina. A tu kit no le importa, que es todo el punto de hoy.

**USDT** es el mayor de la tabla, en `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`, un mint Token clásico que ya agregaste al kit la lección pasada. Sigue estando en todas partes, sobre todo en flujos que empezaron off-shore o fuera de Solana. Acéptalo con el mismo camino de código que USDC y sigue adelante; sus complicaciones son preguntas de negocio y de jurisdicción, no preguntas de integración.

Después está el grupo de los que rinden intereses: stablecoins cuyo saldo o valor de rescate crece porque la reserva genera intereses. Se ven como un mint más, y tratarlos como USDC común es un error, porque sus mecánicas (saldos que hacen rebasing, precios por participación que se acumulan, restricciones de transferencia) se meten exactamente en la contabilidad que hace tu tienda. No los estamos cubriendo, a propósito; sus mecánicas son territorio de DeFi and RWA Engineering. Si un socio te pide aceptar uno, esa profundidad es el prerrequisito, no este párrafo.

![Una tabla de roster con USDC, PYUSD, EURC, USDG y USDT con el programa dueño y los decimales de cada mint, más una fila de traspaso para los tokens que rinden intereses.](assets/v06-comparison.png)

### Cómo viaja USDC: CCTP en una sección

Una pieza más de la historia del roster, porque el USDC que te paga muchas veces empezó su vida en otro lado. Un comprador tiene USDC en Ethereum o Base; tu checkout está en Solana. ¿Cómo se convierte su dólar en un dólar acá?

La vieja respuesta de los puentes era lock-and-wrap: estacionar el token real en un pool en la blockchain de origen, acuñar un IOU en la de destino. Funciona hasta que un exploit vacía el pool, y el IOU wrapped vale solo lo que valga el puente que tiene detrás. La respuesta de Circle para USDC es CCTP, el Cross-Chain Transfer Protocol, y el mecanismo es distinto en especie: burn-and-mint. El USDC se quema en la blockchain de origen, Circle da fe de la quema, y se acuña USDC nativo en la de destino. Ningún pool de colateral bloqueado, ningún sustituto wrapped, y lo que llega es el mismo mint de USDC nativo que tu kit ya envía, indistinguible de cualquier otro USDC en Solana.

Dos hechos específicos de Solana para guardarte. En el esquema de direccionamiento de CCTP cada blockchain es un dominio numerado, y Solana es el dominio 5 (Ethereum es el dominio 0); vas a ver ese número en los mensajes y logs de CCTP cuando depures una llegada cross-chain. Y la velocidad, con la dirección dicha con cuidado, porque este es el detalle que cada resumen de CCTP entiende al revés. Tanto la ruta estándar como la rápida esperan en la blockchain de **origen** — los propios docs de Circle limitan la disponibilidad de Fast Transfer a las blockchains de origen, ya que lo que se está esperando es que la quema se vuelva irreversible donde ocurrió. Así que los tan citados "unos 8 segundos" son la cifra para Solana *como origen*, no para las llegadas hacia Solana. Un comprador que cruza por puente desde Ethereum espera a la finality de Ethereum, y su ruta rápida es correspondientemente más lenta que 8 segundos, aunque sigue siendo una gran mejora sobre la ruta estándar. La tabla de Circle hace concreta la brecha, y es más ancha de lo que admite la mayoría de los artículos. Con Solana como origen, la estándar son 32 confirmaciones, unos 25 segundos. Con Ethereum como origen, la estándar son alrededor de 65 confirmaciones, unos 15 a 19 minutos. Así que "la ruta estándar tarda alrededor de un cuarto de hora" es un hecho sobre Ethereum citado como un hecho sobre CCTP; no hay una sola duración de ruta estándar que citar. Lee la tabla por blockchain de Circle para el origen desde el que de verdad pagan tus clientes, y cita esa fila en vez de la de Solana. El punto de producto sobrevive igual: los compradores cross-chain dejan de ser un ticket de soporte y se vuelven un pago normal que llega un poco tarde.

![Un diagrama de flujo que contrasta CCTP, que quema USDC y lo acuña de forma nativa en el dominio 5 de Solana después de esperar la finality de la blockchain de origen, contra los puentes lock-and-wrap que guardan tokens en un pool.](assets/v07-flowchart.png)

Para tu integración el remate es casi anticlimático, y anticlimático es la meta. No integras CCTP en este curso; las billeteras y los on-ramps lo manejan. Tú solo recibes USDC. Todo lo que construiste la lección pasada, y todo lo que construyes hoy, ya maneja la llegada.

## Lab: enséñale al kit a leer antes de firmar

El kit gana tres habilidades, en orden: detectar el programa dueño de un mint, producir un reporte completo del mint con datos de extensiones en vivo, y enrutar un envío por el programa correcto. Después una comprobación de humo demuestra todo el roster. Trabaja dentro del workspace `transfer-kit` de la lección pasada: corre la línea `npm install --workspace` y los checkpoints de `tsc` desde la raíz `wavelength`, y haz `cd transfer-kit` para todo lo demás, porque cada ruta de archivo de abajo es relativa a ese workspace.

1. Instala el cliente de Token-2022. El workspace está fijado a kit ^6.10.0 (ese pin viene de la lección pasada: los peldaños de checkout aguas abajo importan este kit, y @solana/pay hace peer con kit ^6.9), así que el cliente de Token-2022 tiene que ser un minor de kit v6:

```bash
npm install --workspace transfer-kit @solana-program/token-2022@0.12.0
```

   Nota de frescura sobre ese pin: 0.12.0 es el último minor de @solana-program/token-2022 cuyo rango de peers acepta kit ^6 (hace peer con ^6.4.0); desde 0.13.0 el paquete hace peer con kit ^7 y npm va a rechazar la instalación dentro de este workspace. Verificado contra el registro de npm el 2026-08-22; vuelve a revisar el rango de peers si estás leyendo esto mucho después, porque la ola de kit v7 del ecosistema es donde aterrizan los minors nuevos. Checkpoint: la instalación sale limpia. Un error de peer `ERESOLVE` quiere decir que el pin se corrió más allá de kit v6, y ningún paso posterior va a funcionar hasta que esté bien.

2. Crea `src/detect.ts`. Este es el desafío de completion: la lectura de la cuenta está escrita para ti, la clasificación no. El archivo compila tal como está, y cada paso aguas abajo va a lanzar hasta que lo termines:

```ts
// src/detect.ts: which token program owns this mint?
import type { Address, Rpc, GetAccountInfoApi } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";

export { TOKEN_PROGRAM_ADDRESS, TOKEN_2022_PROGRAM_ADDRESS };

/**
 * Reads the mint account and returns the token program that owns it.
 * Every transfer this kit builds MUST target this program, never a
 * hardcoded one.
 */
export async function detectTokenProgram(
  rpc: Rpc<GetAccountInfoApi>,
  mint: Address,
): Promise<Address> {
  const { value } = await rpc
    .getAccountInfo(mint, { encoding: "base64" })
    .send();
  if (!value) {
    throw new Error(`Mint ${mint} does not exist on this cluster`);
  }
  const owner = value.owner;

  // COMPLETION CHALLENGE: classify `owner`.
  // Return it when it matches one of the two exported program
  // addresses; throw for anything else, because an account owned by
  // neither token program is not a mint and the kit must refuse to
  // build a transfer against it. Two comparisons and one throw.
  throw new Error(`TODO: classify owner program ${owner}`);
}
```

   Llénalo ahora, antes de seguir. Las dos direcciones contra las que comparar ya están importadas y re-exportadas al principio del archivo; el mensaje de rechazo debería nombrar al owner inesperado, porque el tú-del-futuro depurando un mint raro lo va a querer. No te saltes la rama de rechazo. Devolver un programa por defecto para un owner desconocido es la manera en que un kit firma algo que no entiende.

3. Crea `src/read-mint.ts`, el generador de reportes. Una llamada, una imagen honesta de cualquier mint:

```ts
// src/read-mint.ts: one live report per mint: program, decimals,
// extensions, and the CURRENT transfer fee. Never trust the ticker.
import { address, type Address, type Rpc, type GetAccountInfoApi } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";
import { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";

// An unset hook program decodes as the all-zero pubkey, which prints
// as the same base58 string as the system program address.
const UNSET = address("11111111111111111111111111111111");

export interface MintReport {
  mint: Address;
  programAddress: Address;
  /** Extension names as found on the mint, empty for classic Token. */
  extensions: string[];
  decimals: number;
  /** Live transfer-fee basis points, null when no transferFeeConfig. */
  transferFeeBps: number | null;
  /** Issuer seizure power: the permanent delegate, when configured. */
  permanentDelegate: Address | null;
  /** Transfer-hook program, when one is actually wired. Can be null
   *  even when the extension is present: configured but dormant. */
  transferHookProgram: Address | null;
}

export async function readMint(
  rpc: Rpc<GetAccountInfoApi>,
  mint: Address,
): Promise<MintReport> {
  const programAddress = await detectTokenProgram(rpc, mint);

  // The token-2022 client's mint codec also decodes classic mints:
  // same base layout, just an empty extension list.
  // Second read of the same account, and yes, it could be one. The
  // codec wants a decoded account and detectTokenProgram wants the raw
  // owner field, so collapsing them means hand-rolling the fetch. For a
  // read that a real checkout caches per mint anyway, clarity wins.
  const account = await fetchMint(rpc, mint);
  const data = account.data;

  const report: MintReport = {
    mint,
    programAddress,
    decimals: data.decimals,
    extensions: [],
    transferFeeBps: null,
    permanentDelegate: null,
    transferHookProgram: null,
  };

  if (programAddress !== TOKEN_2022_PROGRAM_ADDRESS) return report;
  if (data.extensions.__option === "None") return report;

  for (const ext of data.extensions.value) {
    report.extensions.push(ext.__kind);
    if (ext.__kind === "TransferFeeConfig") {
      // Two fee schedules exist. The older stays in force until the
      // epoch stamped on the newer one arrives. We surface the newer:
      // the rate this mint is heading for.
      report.transferFeeBps = ext.newerTransferFee.transferFeeBasisPoints;
    }
    if (ext.__kind === "PermanentDelegate") {
      report.permanentDelegate = ext.delegate;
    }
    if (ext.__kind === "TransferHook") {
      report.transferHookProgram =
        ext.programId === UNSET ? null : ext.programId;
    }
  }
  return report;
}
```

   Fíjate en lo que la lógica de la comisión no hace: nunca supone una tasa. Un cambio de comisión se agenda contra una epoch, así que el mint carga dos calendarios, el más viejo y el más nuevo, y el más viejo sigue en vigor hasta que llega la epoch estampada en el más nuevo. Exponemos `newerTransferFee` porque es la tasa hacia la que va el mint, y en PYUSD hoy los dos calendarios leen cero, así que la distinción es gratis.

   Di la limitación en voz alta, porque es tu kit y deberías saber dónde es aproximado: **`readMint` reporta la tasa agendada, no necesariamente la tasa en vigor hoy.** Si te encuentras con un mint cuyo calendario más nuevo tiene una estampa de epoch en el futuro, la comisión que de verdad se retiene ahora mismo es la del más viejo, y un checkout que cotice desde `transferFeeBps` estaría cotizando el número de mañana. Hacerlo exacto son dos agregados y ningún concepto nuevo: guarda `ext.olderTransferFee` junto al más nuevo en `MintReport`, pídele a la blockchain `await rpc.getEpochInfo().send()` y lee su campo `epoch`, y después elige el calendario más viejo siempre que `epoch < ext.newerTransferFee.epoch`. Lo dejamos fuera del lab porque cada mint que este curso toca cobra cero en los dos calendarios, así que la rama nunca se ejecutaría y nunca la verías fallar. Ponlo antes de aceptar un mint que cobra comisión por dinero real.

4. Lee PYUSD en vivo. Haz una carpeta `scripts/` al lado de `src/` con `mkdir -p scripts`, y después agrega un runner chiquito, `scripts/read.mts`:

```ts
// scripts/read.mts: usage: npx tsx scripts/read.mts <MINT_ADDRESS>
import { createSolanaRpc, address } from "@solana/kit";
import { readMint } from "../src/read-mint.js";

const rpc = createSolanaRpc(
  process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com",
);
const report = await readMint(rpc, address(process.argv[2]));
console.log(
  JSON.stringify(report, (_k, v) => (typeof v === "bigint" ? v.toString() : v), 2),
);
```

   Córrelo contra los dos mints de la apertura:

```bash
npx tsx scripts/read.mts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
npx tsx scripts/read.mts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

   Checkpoint: USDC reporta el programa clásico y una lista de extensiones vacía. PYUSD reporta Token-2022 y ocho extensiones, `transferFeeBps: 0`, una dirección real bajo `permanentDelegate`, y `transferHookProgram: null`. Si tu switch de `detect.ts` está mal, aquí es donde se nota: que PYUSD vuelva como "clásico" con cero extensiones quiere decir que tu clasificación cayó a un valor por defecto. Una salvedad de nombres para que la salida no te asuste: el cliente imprime nombres de códec como `ConfidentialTransferFee`, mientras que la vista jsonParsed del RPC de la misma entrada dice `confidentialTransferFeeConfig`. Las mismas ocho entradas TLV, dos grafías; cuenta entradas, no grafías. (Me tomó un minuto de entrecerrar los ojos la primera vez.)

Antes del código de envío, ten toda la ruta en la cabeza. El nuevo camino de decisión del kit por pago:

![Un diagrama de flujo del camino de envío: detectar el programa dueño del mint, rechazar dueños desconocidos y transfer hooks activos, y después enrutar cada transferencia hacia un tramo final compartido de firmar-y-confirmar.](assets/v08-flowchart.png)

5. Reescribe `src/send.ts` para que la ruta de arriba sea real. Esto reemplaza la versión hardcodeada de la lección pasada; el pipe de abajo queda intacto, que es el punto:

```ts
// src/send.ts: program-aware sendStablecoin. Four diffs from last
// lesson, all deliberate:
//   1. resolveAta moves here from ata.ts and takes a third seed, the
//      mint's owner program, which is read once and threaded through.
//   2. The idempotent-create helper becomes the ...Async variant, which
//      derives the ATA itself, so there is no explicit `ata:` argument.
//   3. `signature` is returned as a plain string, not kit's branded
//      Signature type, so callers can serialize it without ceremony.
//   4. The RPC clients are constructed inside from URLs. That is a
//      deliberate simplification for a course kit and it costs you the
//      ability to inject a fake RPC in a test; if you later want that
//      back, take rpc/rpcSubscriptions as optional overrides.
import {
  AccountRole,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getTransferCheckedInstruction as getClassicTransferChecked,
} from "@solana-program/token";
import { getTransferCheckedInstruction as get2022TransferChecked } from "@solana-program/token-2022";
import { getAddMemoInstruction } from "@solana-program/memo";
import { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";
import { readMint } from "./read-mint.js";

/** ATA derivation now takes the owner PROGRAM as a seed: the same
 *  wallet has a different USDC address and PYUSD address partly
 *  because the token program is part of the derivation. */
export async function resolveAta(
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({
    owner,
    mint,
    tokenProgram,
  });
  return ata;
}

export interface SendResult {
  signature: string;
  reference: Address;
  tokenProgram: Address;
}

export async function sendStablecoin(opts: {
  rpcUrl: string;
  rpcSubscriptionsUrl: string;
  payer: KeyPairSigner;
  mint: Address;
  recipient: Address;
  /** exact base units from toBaseUnits, never a float */
  amount: bigint;
  memo: string;
  reference: Address;
}): Promise<SendResult> {
  const rpc = createSolanaRpc(opts.rpcUrl);
  const rpcSubscriptions = createSolanaRpcSubscriptions(
    opts.rpcSubscriptionsUrl,
  );

  // 1. The switch this whole lesson exists for. ONE read: readMint
  // already detects the owner program internally and hands it back on
  // the report, so calling detectTokenProgram here too would be a
  // second round trip for an answer we are already holding.
  const report = await readMint(rpc, opts.mint);
  const tokenProgram = report.programAddress;

  // Refuse surprises instead of eating them: a live transfer hook
  // means extra required accounts this kit does not resolve.
  if (report.transferHookProgram !== null) {
    throw new Error(
      `Mint ${opts.mint} has an active transfer hook ` +
        `(${report.transferHookProgram}); this kit does not resolve ` +
        `hook accounts. See the Digital Assets course for the interface.`,
    );
  }

  // 2. Both ATA derivations carry the detected program.
  const sourceAta = await resolveAta(
    opts.payer.address,
    opts.mint,
    tokenProgram,
  );
  const destinationAta = await resolveAta(
    opts.recipient,
    opts.mint,
    tokenProgram,
  );

  // 3. Idempotent ATA creation for first-time holders, same rung as
  // last lesson, now told which program will own the account.
  const createAtaIx = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer: opts.payer,
    owner: opts.recipient,
    mint: opts.mint,
    tokenProgram,
  });

  // 4. Route the transfer to the matching client. Same instruction
  // layout on both programs; different program id on the wire.
  const transferInput = {
    source: sourceAta,
    mint: opts.mint,
    destination: destinationAta,
    authority: opts.payer,
    amount: opts.amount,
    decimals: report.decimals,
  };
  const baseTransferIx =
    tokenProgram === TOKEN_2022_PROGRAM_ADDRESS
      ? get2022TransferChecked(transferInput)
      : getClassicTransferChecked(transferInput);

  // 5. Reference key: the read-only non-signer marker reconciliation
  // will search for, exactly as in last lesson.
  const transferIx: Instruction = {
    ...baseTransferIx,
    accounts: [
      ...baseTransferIx.accounts,
      { address: opts.reference, role: AccountRole.READONLY },
    ],
  };

  const memoIx = getAddMemoInstruction({ memo: opts.memo });

  // 6. The send pipe is untouched from last lesson.
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(opts.payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) =>
      appendTransactionMessageInstructions(
        [createAtaIx, memoIx, transferIx],
        m,
      ),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(signed, {
    commitment: "confirmed",
    maxRetries: 0n,
  });

  return {
    signature: getSignatureFromTransaction(signed),
    reference: opts.reference,
    tokenProgram,
  };
}
```

   Dos de estas decisiones importan más allá de este archivo. Los decimales ahora vienen del reporte del mint en vez de una constante, así que un futuro token de 8 decimales no puede quedar mal escalado en silencio por un kit que supuso seis. Y el valor de retorno ganó un campo `tokenProgram`: el back office de tu tienda va a registrar qué máquina movió cada pago, y cuando llegue una pregunta de soporte meses después ese único campo registrado la responde antes de que abras un explorador.

   Dos tareas que crea esta reescritura, y son el precio de cambiar una interfaz compartida. Dije la lección pasada que las interfaces del kit son estructurales, y lo son, que es precisamente la razón por la que un cambio en ellas es una migración caminada y no un ejercicio que se te deja a ti.

   **Tarea uno: `src/index.ts`.** Todavía exporta `resolveAta` desde `./ata.js` y dos nombres de tipo que el nuevo `send.ts` no define, así que `tsc` falla en la línea de export antes de llegar siquiera a tu lógica. Retira `src/ata.ts` (su `resolveAta` de dos argumentos no puede derivar una dirección de Token-2022) y reemplaza el archivo barrel con exactamente esto:

```ts
// src/index.ts
export { toBaseUnits, fromBaseUnits } from "./amounts.js";
export { resolveAta, sendStablecoin } from "./send.js";
export type { SendResult } from "./send.js";
export { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";
export { readMint } from "./read-mint.js";
export type { MintReport } from "./read-mint.js";
export * from "./mints.js";
```

   **Tarea dos: `src/pay.ts`.** `sendStablecoin` ahora toma URLs de RPC en vez de clientes vivos, unidades base exactas en vez de un string decimal, y un `reference` que generas en el sitio de la llamada, y devuelve `tokenProgram` donde la forma vieja devolvía `destinationAta` y `baseUnits`. Quien llama absorbe todo eso, y `receipt.json` conserva la forma exacta que `verify` ya lee. Reemplaza el archivo:

```ts
// src/pay.ts
import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import {
  address,
  createKeyPairSignerFromBytes,
  generateKeyPairSigner,
} from "@solana/kit";
import { sendStablecoin, resolveAta } from "./send.js";
import { toBaseUnits } from "./amounts.js";
import { USDC_DEVNET, USDC_DECIMALS } from "./mints.js";

const recipientArg = process.argv[2];
const amountArg = process.argv[3] ?? "1.25";
const referenceArg = process.argv[4];
if (!recipientArg) {
  console.error(
    "usage: [PAYER_KEYFILE=<path>] npm run --workspace transfer-kit pay -- <recipient-wallet> [amount] [reference]",
  );
  process.exit(1);
}

// PAYER_KEYFILE lets this script send AS somebody else, which is the whole
// point of the pretend customer you made in lesson 1. Unset, it is your
// merchant identity, exactly as before.
const keyfile = process.env.PAYER_KEYFILE ?? `${homedir()}/.config/solana/id.json`;
const bytes = new Uint8Array(JSON.parse(await readFile(keyfile, "utf8")));
const payer = await createKeyPairSignerFromBytes(bytes);

// The caller owns these two now, on purpose: the checkout that
// generates a reference is the thing that must remember it. Pass a
// reference in when you are paying a checkout that already minted one;
// omit it and this script mints a throwaway of its own.
const baseUnits = toBaseUnits(amountArg, USDC_DECIMALS);
const reference = referenceArg
  ? address(referenceArg)
  : (await generateKeyPairSigner()).address;

const result = await sendStablecoin({
  rpcUrl: "https://api.devnet.solana.com",
  rpcSubscriptionsUrl: "wss://api.devnet.solana.com",
  payer,
  recipient: address(recipientArg),
  mint: USDC_DEVNET,
  amount: baseUnits,
  memo: "wavelength-order-0001",
  reference,
});

// Rebuild the two fields the new return shape dropped, so receipt.json
// stays byte-identical to what verify.ts already parses.
const destinationAta = await resolveAta(
  address(recipientArg),
  USDC_DEVNET,
  result.tokenProgram,
);

console.log("signature :", result.signature);
console.log("reference :", result.reference);
console.log("program   :", result.tokenProgram);
console.log("base units:", baseUnits.toString());

await writeFile(
  new URL("../receipt.json", import.meta.url),
  JSON.stringify(
    {
      signature: result.signature,
      reference: result.reference,
      destinationAta,
      baseUnits: baseUnits.toString(),
    },
    null,
    2,
  ),
);
```

   Entraron dos perillas que el archivo viejo no tenía, y el módulo 3 cobra las dos. `PAYER_KEYFILE` elige qué keypair firma: déjalo sin definir y eres el comercio, ponlo en `/tmp/customer.json` y este script se vuelve tu cliente de mentira que te paga a *ti*. Un cuarto argumento acepta una reference que acuñó alguien más, que es como un script hace de cliente para un checkout que ya generó su propio número de seguimiento. Ninguna perilla cambia la corrida de hoy; las dos existen porque un comercio pagándose a sí mismo no es un pago, y el checkout del módulo 3 necesita una segunda billetera real del otro lado del mostrador.

   Checkpoint: `npx tsc --noEmit` desde la raíz `wavelength` vuelve a quedarse en silencio, y `npm run --workspace transfer-kit verify` sigue pasando contra el comprobante de la lección pasada.

6. Necesitas un mint Token-2022 que de verdad puedas gastar en devnet, y PYUSD no reparte saldos de devnet, así que haz tu propio mint de prueba. La CLI `spl-token` viene en el mismo release de Agave que instalaste para `solana` la lección pasada; revisa con `spl-token --version` (si de algún modo falta, `cargo install spl-token-cli` la restaura):

```bash
spl-token create-token \
  --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb \
  --decimals 6 --url devnet
```

   Copia la dirección de mint impresa, y después crea tu propia cuenta de token para él y acúñate un saldo:

```bash
spl-token create-account <YOUR_T22_MINT> --url devnet
spl-token mint <YOUR_T22_MINT> 100 --url devnet
export T22_MINT=<YOUR_T22_MINT>
```

   Checkpoint: `spl-token balance $T22_MINT --url devnet` imprime 100. Corre `npx tsx scripts/read.mts $T22_MINT` con `RPC_URL=https://api.devnet.solana.com` y tu propio generador de reportes te dice lo que acabas de hacer: Token-2022, seis decimales, ninguna extensión. Un mint Token-2022 pelado es uno perfectamente legal; las extensiones son opt-in al crearse, y el curso Digital Assets es donde aprenderías a optar por ellas.

   Ahora pisa la mina a propósito, porque una falla que has visto vale por diez de las que te han advertido. Apunta el camino solo-clásico de la lección pasada a este mint Token-2022:

```bash
spl-token transfer $T22_MINT 1 <ANY_WALLET> \
  --program-id TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  --url devnet
```

   Ese `--program-id` es el programa Token clásico, que es precisamente la constante que tu viejo `send.ts` hardcodeó. Resultado esperado: la CLI rechaza antes de que se construya o se envíe nada — código de salida 1 y un error de owner que no coincide nombrando tu mint, con la forma `Account <YOUR_T22_MINT> is owned by TokenzQd..., not configured program id Tokenkeg...`. Fíjate en lo que **no** obtuviste: una firma, una entrada en el explorador, o una comisión gastada. La CLI lee el owner del mint y lo compara contra el programa pedido antes de construir nada — del lado del cliente, exactamente el hábito que tu kit acaba de aprender. Tu viejo `send.ts` no tenía esa comprobación, que es la razón por la que su versión de esta falla aflora una costura después, en la simulación de preflight del RPC. El mismo rechazo, tres costuras posibles — comprobación del cliente, preflight, en la blockchain — y mientras más temprano lo atrapes, más barato es. Esa brecha entre "bien formado" y "va a ejecutar" es todo el motivo por el que el kit ahora lee el mint antes de construir nada.

7. Entrega la verificación de humo. Crea `scripts/verify-roster.mts`:

```ts
// scripts/verify-roster.mts: the per-lesson smoke check.
// 1. Mainnet read: PYUSD reports eight extensions + live fee bps.
// 2. Devnet sends: one classic-Token mint, one Token-2022 mint,
//    each routed to the correct owner program.
import { readFile } from "node:fs/promises";
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  generateKeyPairSigner,
} from "@solana/kit";
import { readMint } from "../src/read-mint.js";
import { sendStablecoin } from "../src/send.js";
import {
  TOKEN_PROGRAM_ADDRESS,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "../src/detect.js";

const PYUSD = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
const DEVNET_USDC = address("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
if (!process.env.T22_MINT) {
  throw new Error("set T22_MINT to the devnet Token-2022 mint you made in step 6");
}
const T22_MINT = address(process.env.T22_MINT);

const mainnet = createSolanaRpc("https://api.mainnet-beta.solana.com");
const pyusd = await readMint(mainnet, PYUSD);
// Do NOT gate on the count. Two reasons, both from this lesson: the
// live mint's extension set is PayPal's to change, and the client and
// the RPC's jsonParsed view spell some entries differently, so a count
// is a spelling artifact. Print it, then assert on the properties that
// actually decide whether we can take the payment.
console.log(`PYUSD extensions (${pyusd.extensions.length}):`, pyusd.extensions);
if (pyusd.transferHookProgram !== null) {
  throw new Error(`PYUSD: transfer hook is live; this kit cannot route it`);
}
if (typeof pyusd.transferFeeBps !== "number") {
  throw new Error(`PYUSD: transfer fee did not read as a number`);
}
console.log(`PYUSD: ${pyusd.extensions.length} extensions, transfer fee ${pyusd.transferFeeBps} bps (live)`);

const keyBytes = new Uint8Array(
  JSON.parse(await readFile(`${process.env.HOME}/.config/solana/id.json`, "utf8")),
);
const payer = await createKeyPairSignerFromBytes(keyBytes);
const recipient = (await generateKeyPairSigner()).address;

for (const [label, mint, expected] of [
  ["classic", DEVNET_USDC, TOKEN_PROGRAM_ADDRESS],
  ["token-2022", T22_MINT, TOKEN_2022_PROGRAM_ADDRESS],
] as const) {
  const result = await sendStablecoin({
    rpcUrl: "https://api.devnet.solana.com",
    rpcSubscriptionsUrl: "wss://api.devnet.solana.com",
    payer,
    mint,
    recipient,
    amount: 250_000n, // 0.25 at 6 decimals, exact base units
    memo: `verify:roster ${label}`,
    reference: (await generateKeyPairSigner()).address,
  });
  if (result.tokenProgram !== expected) {
    throw new Error(`${label}: routed to ${result.tokenProgram}`);
  }
  console.log(`${label}: confirmed ${result.signature} via ${result.tokenProgram}`);
}
console.log("verify:roster PASS");
```

   Conéctalo en los scripts de `package.json` del workspace, al lado del `verify` de la lección pasada:

```json
{
  "scripts": {
    "pay": "tsx src/pay.ts",
    "verify": "tsx src/verify.ts",
    "verify:roster": "tsx scripts/verify-roster.mts"
  }
}
```

   El envío por la rama clásica gasta el USDC de devnet que todavía tienes de cuando corriste verify la lección pasada; si el saldo se secó, acuña más desde el flujo del faucet de devnet que usaste allá. No corras todavía la verificación completa. Correrla es la mitad de atrás del Challenge.

## Challenge

La mitad de completion ya la conociste: el switch de `detectTokenProgram` en el paso 2. Si lo dejaste para después, ciérralo ahora, y sé estricto contigo mismo con la tercera rama. El rechazo para un owner desconocido es la diferencia entre un kit y un peligro.

La mitad solo, sin guía: corre el roster. Enumera las extensiones de PYUSD desde una lectura en vivo del mint e imprime los centésimos de punto porcentual actuales de la comisión de transferencia, usando tu propio `scripts/read.mts`. Después envía el mismo monto por dos mints en devnet, el mint de USDC de Token clásico y tu propio mint de prueba Token-2022, y termina con:

```bash
npm run --workspace transfer-kit verify:roster
```

Acepta cuando se cumplan las tres: las dos transferencias aterrizan en devnet, el kit reporta el programa dueño correcto para cada mint, y tu lectura imprime la lista de extensiones en vivo y la comisión en centésimos de punto porcentual desde el propio mint en vez de desde la memoria, y la comprobación del hook pasa. Si el envío de Token-2022 falla mientras el clásico pasa, tu derivación de ATA casi con certeza no tiene el seed del programa: vuelve a correr `scripts/read.mts` sobre tu mint de prueba, y después quédate mirando `resolveAta`.

Una pregunta de reflexión para cerrar el loop, sin código: tu tienda quiere aceptar USDG el próximo trimestre. ¿Qué, concretamente, ya sabe hacer tu kit, y qué único hecho verificarías todavía antes de encenderlo? Si tu respuesta incluye leer el mint en vivo y revisar la comisión y el delegado, la lección aterrizó. Si tu respuesta es "nada, la tabla dijo que está bien", vuelve a leer la disyuntiva.

Esta cubrió mucho roster para una sola lección, y si los poderes de las extensiones todavía se sienten abstractos, eso es lo esperado: hoy los leíste, no los escribiste, y la profundidad de autoría vive en el curso Digital Assets por diseño. El catálogo completo de extensiones, los transfer hooks, y las transferencias confidenciales detrás de este mismísimo mint son los módulos dos al cuatro de ese curso; si quieres el lado del emisor de lo que acabas de leer, esa es la puerta. Lleva la salida de tu `verify:roster` a la comunidad del curso si algo se enrutó mal; una firma que falla con una dirección de mint es un diagnóstico de cinco minutos cuando otros builders pueden verla.

Tu kit ahora mueve cualquier stablecoin del roster, clásico o Token-2022, y rechaza los que no puede firmar con seguridad. El próximo módulo deja de ser un script y se vuelve una tienda: un solo núcleo de pagos detrás de un checkout con código QR, un puesto de mercado y un link de drop compartible. Las claves de reference que has venido adjuntando diligentemente están a punto de ganarse el sueldo.
