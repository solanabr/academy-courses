# Enrutamiento de comisiones y buyback/quema: por dónde fluye realmente el dinero

## Resumen

La lección pasada entregaste el compost airdrop: la tabla de costos por destinatario que volvió honesta la aritmética de clásico contra comprimido, más un reclamo desbloqueado y un reclamo con vesting `claim_locked`, ambos demostrados contra tu propio port fiel a los bytes del árbol del distributor. Los tokens salieron por la puerta. Hoy vuelven.

El marketplace de Overgrowth se lleva el 1% de cada canje de SPROUT. La comisión se dispara en cada transferencia. Puedes verla en los montos que los compradores reciben de verdad. Y a una semana del beta el saldo de la tesorería sigue siendo exactamente cero. Nadie lo robó, nada está roto, y antes de que leas otro párrafo quiero que vayas a mirar el mint mismo. Supuestos de partida para esta apertura, porque retomar en frío rompe los tres: tu surfnet de los módulos anteriores está levantado con el mint de SPROUT encima y algunos canjes que cobraron comisión detrás (vuelve a acuñar según la apertura de m05-l1 y corre unas cuantas transferencias si estás retomando desde cero), la raíz del workspace lleva las versiones fijadas de kit y token-2022 de m02, y corres el comando desde esa raíz. Deja esto en `labs/m09-l1/peek.ts`:

```ts
// peek.ts: how much of SPROUT's fee income has actually reached the mint?
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

async function main(): Promise<void> {
  const rpc = createSolanaRpc(process.env.RPC_HTTP ?? "http://127.0.0.1:8899");
  const mint = await fetchMint(rpc, address(process.env.SPROUT_MINT!));

  const exts = mint.data.extensions;
  const fee =
    exts.__option === "Some" ? exts.value.find((e) => e.__kind === "TransferFeeConfig") : undefined;
  if (fee?.__kind !== "TransferFeeConfig") throw new Error("no TransferFeeConfig on this mint");

  const bps = fee.newerTransferFee.transferFeeBasisPoints;
  console.log(`fee schedule: ${bps} bps, cap ${fee.newerTransferFee.maximumFee}`);
  console.log(`withheld ON THE MINT: ${fee.withheldAmount}`);
  console.log(`supply: ${mint.data.supply}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Córrelo: `SPROUT_MINT=<your mint> npx tsx labs/m09-l1/peek.ts`, desde la raíz del workspace. Deberías ver tres líneas, y la tercera es el problema:

```text
fee schedule: 100 bps, cap 5000000
withheld ON THE MINT: 0
supply: 1000000000000000
```

Cien basis points, cobrados toda la semana, y cero ha llegado al mint.

Ese cero es toda la lección. Hoy conviertes una comisión que existe en dinero que se mueve, y después en supply que desaparece. La ruta, en orden: dónde están de verdad las comisiones y qué hace el crank de harvest al respecto, después los tres modelos de comisión que la gente confunde sin parar y lo que cuesta confundirlos, después el reparto que financia una quema sin inventar tokens, después el buyback mismo, que es una compra a un precio que alguien te cobra, y por último la quema, con la trampa de lectura vieja que se come su assertion de supply.

El repliegue de la ayuda en esta lección, en voz alta: el tramo de harvest está trabajado por completo, yo escribo cada instrucción y tú sigues el hilo. El reparto de comisiones y el dimensionamiento del buyback son un problema de completion, TODOs en un archivo cuyo código circundante ya corre. El riel completo, de comisión del marketplace a harvest a tesorería a buyback a quema con una assertion de supply al final, es tuyo en solitario.

## Dónde está el dinero de verdad

### No hay caja detrás de la caseta de peaje

Imagina un mercado techado donde cada puesto le paga al mercado un 1% de tajada. Esperarías una caja junto a la puerta. Token-2022 no funciona así. Cuando se dispara una transferencia, el programa saca la comisión del monto transferido y la deja en un frasquito cerrado con llave que está sobre la *mesa del propio destinatario*, dentro de su cuenta de token, en un slot llamado `TransferFeeAmount.withheldAmount`. El comprador no puede gastarla. Tú tampoco puedes gastarla, no desde donde estás parado. Diez mil canjes son diez mil frasquitos repartidos por diez mil mesas, y ninguno es tuyo.

Alguien tiene que recorrer el salón.

Lo que está en juego para ti es concreto y no es contabilidad abstracta: una comisión a la que nunca le haces harvest es una comisión que nunca ganaste, ingresos que existen en el papel y no financian nada. Cada buyback que planeas, cada presupuesto de ops, cada línea de "el protocolo se autofinancia" en tus docs está río abajo de un aburrido cron que nadie es lo bastante glamoroso como para querer tener a su cargo.

Construiste el mecanismo para recorrer el salón allá en el módulo 2, en la lección de economics-extensions, y lo probaste contra un solo comprador. Hoy se convierte en el primer tramo de un riel con otros tres tramos atornillados encima.

![Un diagrama de flujo traza las comisiones retenidas desde las cuentas de los compradores, pasando por un harvest sin permiso hacia el mint, un withdraw condicionado a la autoridad hacia la PDA de tesorería, un buyback contra la contraparte que tengas, y una quema que baja el supply.](assets/v01-flowchart.png)

### Tramos uno y dos: el crank de harvest (consolidar, después recaudar)

Dos instrucciones hacen el trabajo, y la división entre ellas es un diseño de permisos, no un accidente.

`harvest_withheld_tokens_to_mint` toma una lista con las cuentas de token de origen y barre sus saldos retenidos hacia el mint, hacia un campo `withheldAmount` que vive dentro del propio `TransferFeeConfig` del mint. Ese es el campo que `peek.ts` acaba de imprimir como cero. También es sin permiso, lo que significa que cualquiera puede llamarlo sobre las cuentas de cualquiera, y eso es seguro precisamente porque consolidar no puede robar: los tokens solo se mueven de frascos dispersos a un solo frasco que una sola llave puede abrir.

`withdraw_withheld_tokens_from_mint` abre después ese frasco y manda el montón a una cuenta de token de destino. Esta está condicionada al `withdraw_withheld_authority` que fijas al crear el mint. También existe la ruta directa, `withdraw_withheld_tokens_from_accounts`, que se salta la escala en el mint y extrae de una lista nombrada de cuentas directo a tu destino en un solo salto firmado por la autoridad. Dos tramos para recaudación de rutina a escala, un salto para extracciones quirúrgicas.

```text
harvest_withheld_tokens_to_mint        accounts -> mint    PERMISSIONLESS
withdraw_withheld_tokens_from_mint     mint -> destination  withdraw_withheld_authority
withdraw_withheld_tokens_from_accounts accounts -> dest.    withdraw_withheld_authority
```

Tu destino es la tesorería. Para Overgrowth eso es una dirección derivada de programa, una PDA de tesorería, cuya cuenta de token asociada guarda SPROUT y cuyo saldo en SOL financia el buyback. Una PDA en vez de una clave caliente porque lo que recibe los ingresos del protocolo debería ser una dirección sin clave privada, que un programa pueda poseer y que cualquiera con un explorador pueda auditar. Esa elección no te cuesta nada hoy y te ahorra la conversación donde un contratista que se va todavía tiene la frase semilla de la tesorería. Una nota de honestidad ahora, para que el código del lab no pueda contradecir este párrafo en tu cabeza: el lab carga `treasury.json`, un keypair común, y lo deja firmar directo. Una PDA no puede vivir en un archivo JSON y no puede firmar salvo a través del `invoke_signed` de su programa, y entregar el programa que sería dueño de la PDA de tesorería que Overgrowth usaría en producción queda fuera del alcance de esta lección. Así que el keypair `treasury` del lab es un suplente que hace el papel de la PDA: cada lugar donde firma es un lugar donde el programa del diseño de producción firmaría con seeds, y nada más del riel cambia.

El cableado de autoridades alrededor merece treinta segundos, porque lo fijas una sola vez al crear el mint y es casi permanente. `transfer_fee_config_authority` puede cambiar el esquema de comisión. `withdraw_withheld_authority` puede recaudar. Son claves separadas a propósito, y separarlas es la diferencia entre un diseño de gobernanza y un único punto de falla: un multisig o una DAO pueden tener el poder de fijar la tasa mientras una aburrida clave de ops tiene el poder de barrer, y rotar la clave de ops no toca la tasa. Revocarlas tampoco es simétrico. Anula la config authority y tu esquema de comisión queda congelado para siempre, lo cual es un compromiso creíble que los tenedores pueden verificar. Anula la withdraw authority y cada comisión que el token retenga alguna vez, pasada y futura, queda varada de forma permanente: el harvest sigue consolidando hacia el mint, y nada podrá volver a abrir ese frasco. Una de esas revocaciones es una promesa. La otra es una lápida. En el lab de abajo el firmante de la tesorería tiene la withdraw authority, lo cual está bien para un fork y no es lo que yo entregaría.

### Lo que cuesta correr el crank

El crank tiene un costo operativo y es tuyo para siempre. Alguien paga las comisiones de transacción, alguien nota cuando el cron muere, alguien decide si barrer 400 cuentas por semana le gana a barrer 40 cuentas por día. Casi todo postmortem de token con comisión que he leído se reduce a que nadie corrió el cron.

La buena noticia es que la mitad de consolidación es barata. `harvest_withheld_tokens_to_mint` toma un array `sources` entero, así que una instrucción barre muchas cuentas, y en mi corrida sobre surfnet allá en la lección de economía un harvest de una sola fuente midió alrededor de 1,200 unidades de cómputo. Que consolidar salga casi gratis es exactamente el diseño correcto para una llamada que cualquiera tiene permitido hacer.

La restricción que de verdad muerde no es el cómputo, es la transacción. Cada cuenta de origen que listas es otra clave de cuenta en el mensaje, y el mensaje tiene que caber en una transacción. Así que el tamaño de tu lote es un problema de empaquetado: cuántas direcciones de cuenta caben junto a los datos de la instrucción y las firmas. Resuélvelo empíricamente para tu propio montaje en vez de confiar en un número de un post de blog, porque las address lookup tables, las instrucciones extra y tu fee payer mueven el techo. Después corta la salida del escaneo en lotes de ese tamaño y mándalos como transacciones separadas. La falla parcial se sobrevive aquí de un modo en que rara vez se puede: el harvest es idempotente en el sentido que importa, ya que una cuenta con cero retenido aporta cero, así que un reintento que vuelva a incluir una cuenta ya barrida es un no-op y no un doble conteo.

Lo que hace que el loteo en sí sean unas seis líneas, y que valga la pena mantenerlo como su propia función para que el tamaño del lote sea un número que puedas ajustar después de medir:

```ts
// chunk.ts: batch the scan's output so each harvest transaction fits the wire.
export function chunk<T>(items: T[], size: number): T[][] {
  if (size < 1) throw new Error("batch size must be at least 1");
  const out: T[][] = [];
  for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
  return out;
}
```

Y encontrar las cuentas sucias también es problema tuyo. Una cuenta de token pone su mint en el offset de byte 0, así que una sola llamada a `getProgramAccounts` con un filtro memcmp te da cada tenedor de SPROUT, y después lees el `TransferFeeAmount` de cada una y te quedas con las distintas de cero. En un fork con unas pocas docenas de tenedores eso es un escaneo de dos segundos. Con el conteo de tenedores que maneja Jupiter es un trabajo de indexación, `getProgramAccounts` sobre un programa grande es exactamente la consulta que los RPC públicos estrangulan más fuerte, y la respuesta sincera es que lo alquilas, igual que la lección de lectura de activos te hacía alquilar un proveedor de DAS en vez de correr tu propio indexador.

![Una tabla compara los cuatro costos de correr un crank de harvest sobre comisiones retenidas, desde cómputo barato pasando por límites de empaquetado y escaneos pesados de cuentas hasta la propiedad operativa que causa la mayoría de las fallas.](assets/v02-comparison.png)

### Tres modelos de comisión, y la semana que pierdes por confundirlos

Aquí es donde he visto a gente competente quemar días.

pump.fun también tiene comisiones de creador. No son comisiones retenidas de Token-2022, no usan `TransferFeeAmount`, y ningún harvest las va a encontrar. Las comisiones de creador de pump son del lado del programa: el programa las enruta a una PDA `creator_vault` derivada por creador, y el creador reclama desde ese vault. Mecanismo distinto, cuenta distinta, ruta de reclamo distinta, las mismas tres palabras en inglés en el pitch deck.

Vale la pena conocer con precisión el resto de la maquinaria de comisiones que usa pump, porque es lo más parecido a una implementación de referencia para políticas de comisión que tiene el ecosistema. Las comisiones corrieron planas en 100 basis points durante toda la era temprana. Después, el 2025-09-01 20:00 UTC, pasaron a ser un esquema escalado por capitalización de mercado: tu nivel de comisión ahora depende de dónde se ubique tu moneda, lo que quiere decir que la comisión es una política que se mueve debajo de ti y no una constante que configuraste. Las monedas Cashback invierten la dirección por completo, redirigiendo la comisión de creador de vuelta a los traders a través de PDAs acumuladoras de volumen. Las comisiones de protocolo rotan entre 8 destinatarios de comisión para que las cuentas de recaudación no se vuelvan una única escritura caliente y disputada. Y compartir comisiones admite hasta 10 accionistas, así que el "creador" de la comisión de creador puede ser una tabla de capitalización.

Lee lo que quiere decir ese día del cambio en vez de solo archivar la fecha. Antes, un creador que lanzaba en pump conocía el número: 100 basis points, el mismo para todos, el mismo el mes siguiente. Después, la comisión que paga una moneda es función de dónde se negocia esa moneda, que es una variable que el creador no fija ni puede congelar. Eso no es una crítica a pump, cuyo esquema está publicado y cuyo razonamiento es defendible. Es la forma general de lanzar sobre el riel de otro: heredas su política económica, incluida la versión de ella que entregan después de que tú lances. Tu propia comisión Token-2022 es el canje opuesto. Tú eres dueño de la tasa, puedes hacerla permanente anulando la config authority, y a cambio eres dueño del harvest, de la indexación, del cron, y de cada integración que se rompe porque el monto enviado ya no es igual al monto recibido. Ningún lado de ese canje es gratis. Elige aquel por cuyos costos preferirías ser responsable.

![Una línea de tiempo mueve las comisiones de pump.fun desde una era plana de 100 basis points hasta el esquema escalado por capitalización de mercado del 2025-09-01 y de ahí a los redireccionamientos Cashback, con mecánicas de vault constantes a lo largo de todo.](assets/v03-timeline.png)

Ahora el contraejemplo, que es mi objeto favorito de todo este curso. En mayo de 2024, PayPal y Paxos entregaron PYUSD como el mint Token-2022 emblemático con forma de compliance. Lleva una transfer fee config. Esa config está en 0 basis points, y nunca se ha disparado. De todos los tokens de Solana capaces de cobrar comisión, el más serio institucionalmente no recauda nada, a propósito, porque lo que sus emisores querían era la *opción*, armada y dormida, disponible el día que un regulador o un modelo de negocio la pidan. Configurado no es lo mismo que activo. Ya leíste esa misma distinción de un mint en vivo con `decode-mint`, y este es el ejemplo con más en juego.

![Una tabla comparativa separa las comisiones de transferencia retenidas de Token-2022, las comisiones creator_vault del lado del programa en pump.fun, y la config de comisión dormida a cero bps en PYUSD, según punto de acumulación, quién la mueve, tasa y giros.](assets/v04-comparison.png)

La regla práctica: antes de escribir una sola línea de código de recaudación, lee las extensiones del mint y averigua qué máquina tienes enfrente. Si `TransferFeeConfig` está presente con bps distinto de cero, aplica el harvest. Si las comisiones son del lado del programa, ve a buscar el vault del programa y su instrucción de reclamo. Modelo equivocado, semana equivocada.

### El reparto es una ley de conservación

El harvest deja un montón en la tesorería. Ahora decides qué pasa con él, y esta es la parte donde la aritmética descuidada acuña o destruye tokens sin hacer ruido en tu contabilidad.

La política de Overgrowth: una parte de cada harvest se quema de inmediato, el resto financia operaciones. Números redondos, recorridos de a un paso. Haces harvest de 1,000,000 de unidades base de SPROUT. La parte quemada es del 20%, así que 200,000 unidades se queman al llegar y 800,000 se quedan en la tesorería. 200,000 más 800,000 es 1,000,000. Eso no es una coincidencia por la que debas estar agradecido, es el invariante que tu código tiene que sostener en cada entrada, incluidas las feas: `burnedFromFees + toTreasury === harvested`. El reparto no crea nada y no destruye nada. Solo etiqueta.

El buyback es un tramo completamente aparte y lo financia un activo distinto. La tesorería también guarda SOL, de comisiones de listado en el marketplace, del lanzamiento, de donde sea que vengan de verdad tus ingresos. Dimensionar el buyback es una sola división entera hacia abajo: ¿cuántas unidades base de SPROUT compra ese SOL al precio actual de la plataforma? 5,000,000,000 lamports a 1,000,000 lamports por unidad base compran 5,000 unidades. Trúncalo hacia abajo sobre bigints, siempre, porque no puedes comprar una fracción de unidad base y el resto que redondeas hacia afuera es exactamente el tipo de deriva silenciosa que aparece en una assertion de supply seis semanas más tarde, cuando nadie recuerda haber escrito la línea.

La trampa que quiero que nombres en voz alta antes de escribir la función: la quema de comisiones y la quema del buyback no tienen por qué ser iguales, y nada está mal cuando no lo son. Son dos flujos independientes hacia el mismo horno. Uno está denominado en SPROUT que ya tenías, el otro en SOL que convertiste. La conservación aplica dentro del reparto, no entre los dos tramos.

![Un diagrama divide un harvest de 1,000,000 de unidades en una quema de 200,000 y una parte de 800,000 para la tesorería, junto a un buyback aparte financiado con SOL, con ambos flujos convergiendo en una sola quema.](assets/v05-diagram.png)

### Tramo tres: el buyback es un swap, y los swaps cuestan dinero

Checkpoint rápido sobre cómo llegamos aquí, porque la próxima parte introduce un segundo activo y un segundo cliente. Primero: las comisiones existen pero quedan retenidas en las cuentas de los destinatarios, así que el saldo de la tesorería no es evidencia de ingresos. Segundo: los tramos uno y dos son un crank de dos instrucciones, sin permiso para consolidar y condicionado a la autoridad para recaudar, y correrlo es un trabajo operativo con un dueño. Tercero: el reparto de un harvest es una ley de conservación, y es aritmética entre tramos, no un tramo propio; el buyback que dimensiona se financia aparte, con SOL.

Ahora la parte que es fácil de describir y fácil de equivocar emocionalmente.

Un buyback-and-burn no es una función del protocolo que activas. Eres tú, con SOL en la mano, entrando a un mercado abierto, comprando tu propio token a lo que sea que el mercado te cobre hoy, y destruyendo después lo que compraste. La plataforma que m08-l2 eligió para SPROUT es Meteora DBC graduando a un pool DAMM v2, y esa elección no es una preferencia, es el residuo de cada decisión río arriba: pump y LaunchLab fijan SPL clásico en sus instrucciones de create y nunca podrían sostener SPROUT, DBC aceptó el mint Token-2022, y la migración de DBC aterriza en la familia DAMM, con v2 como el lado que lleva un mint base Token-2022. El programa DAMM v2 es `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`, y en tu fork de mainnet es el de verdad, con estado forkeado y todo.

El límite, zanjado antes del lab en vez de descubierto dentro de él. **Este curso nunca lanza SPROUT.** m08-l1 derivó un umbral en el papel, m08-l2 eligió una plataforma en el papel, y ninguno llevó una config de DBC hasta `migration_quote_threshold` ni migró nada, porque un lanzamiento es un evento de distribución y este curso enseña la capa de token. Así que a menos que hayas ido a lanzar SPROUT tú mismo, tu fork no lleva ningún pool de SPROUT, y el tramo 3 no tiene mercado contra el cual operar. Eso no es un hueco que el lab tape; es una bifurcación en el camino con dos puertas honestas, y el paso 1c de abajo es donde eliges una. También quiere decir que la marca de documentado-pero-no-verificado que la lección de launchpad puso sobre el soporte de mint base Token-2022 en DBC sigue abierta: nada en esta lección la zanja, y quien sí lance SPROUT en DBC y lo reporte es quien la cierra.

La única matemática de AMM que usa esta lección es una frase: el precio spot del pool es la razón entre sus dos reservas de vault, así que la reserva quote dividida por la reserva base te da lamports por unidad base de SPROUT, y ese número es por el que divides el SOL de tu tesorería para dimensionar la compra. DAMM v2 también cotiza ese mismo precio de forma nativa como una raíz cuadrada en punto fijo, y la brecha entre las dos vistas, una vez que las posiciones concentradas la abren, le pertenece al curso de DeFi junto con el resto de la matemática de pools. Ese es todo el contenido matemático, y son cuatro líneas que puedes correr ahora mismo:

```ts
// sizing, standalone: what does the treasury's SOL buy at the pool's current price?
const treasurySol = 5_000_000_000n; // lamports the treasury is willing to spend
const price = 1_000_000n; // lamports per SPROUT base unit = quoteReserve / baseReserve
const buyback = treasurySol / price;
console.log(`${buyback} SPROUT base units at spot, before slippage`); // 5000
```

Fíjate en las últimas tres palabras de esa línea de log. Todo lo que viene después de este punto en la lección trata sobre la brecha entre "at spot" y lo que de verdad llega a tu cuenta.

La composición de pools, el ruteo entre plataformas, la matemática de ticks y bins, la estrategia de LP, y todo lo demás que hace que un swap sea eficiente en vez de apenas posible es material del curso planificado DeFi and RWA Engineering. No te voy a dar una versión superficial de un tema que tiene un hogar propio.

Lo que *sí* necesitas de mí es la lista honesta de costos, porque un buyback se lee como deflación gratis y no lo es.

Pagas slippage, porque tu propia compra mueve el precio en tu contra, así que el SPROUT que recibes es menos de lo que prometía la aritmética del precio spot, y en un pool delgado con una orden grande de tesorería es bastante menos. Pagas la comisión de la plataforma encima de eso. Estás expuesto a MEV: un buyback es una orden de mercado grande, predecible y anunciada en público, que es más o menos la forma ideal de un objetivo de sándwich, y las tácticas de aterrizaje en el lado del cliente que mitigan eso son territorio del curso planificado Client-Side Mastery, no de esta lección. Y hay un giro específico de Token-2022 que agarra a todos la primera vez: SPROUT cobra una comisión de transferencia en *cada* transferencia, incluida aquella en la que tu contraparte manda SPROUT a tu tesorería. Tu buyback paga tu propia comisión. El monto retenido va a parar de vuelta a la cuenta de token de la propia tesorería, esperando el próximo harvest. Es circular e inofensivo y va a hacer que tu aritmética no concuerde consigo misma, sin ninguna duda, si calculas lo que compraste en vez de medirlo.

Tres de esos cuatro costos necesitan un mercado para existir. El cuarto, tu propia comisión de transferencia, se dispara en cualquier transferencia, la que sea, y por eso es el único componente que el lab puede ponerte enfrente sin importar qué puerta tomes.

Así que mídelo. Lee el saldo de la tesorería antes del swap, léelo después, y quema la diferencia. Todo otro enfoque es que tú afirmes lo que la blockchain debería haber hecho.

Lo cual es también la razón por la que el buyback es una pregunta de política y no un interruptor que accionas. Cuánto, cada cuánto y con cuánta previsibilidad son tres perillas, y mover cualquiera de ellas cambia un costo por otro.

![Una tabla de decisión pesa políticas de buyback mensuales-grandes, continuas-pequeñas y oportunistas frente al impacto en el precio, el costo del crank y la previsibilidad ante el MEV.](assets/v06-table.png)

Hay una pregunta previa escondida aquí, y ya la respondiste. Una plataforma solo acepta tu token si tu conjunto de extensiones es uno que tolera, que es el trabajo de enrutabilidad que hiciste en la lección de designing-a-routable-token. Un delegado permanente o un transfer hook que la allowlist del pool rechaza quiere decir que no hay plataforma y por lo tanto no hay buyback. Las decisiones de extensiones que tomaste en el módulo 5 son las que hacen posible el módulo 9.

### Tramo cuatro: la quema, y la lectura vieja que se come tu assertion

El último tramo es `burn_checked` sobre la cuenta de token de la tesorería, firmado por la autoridad de la tesorería, y es la única instrucción de todo este riel que reduce el supply. Hacer harvest no quema. Hacer withdraw no quema. Mandar tokens a una dirección muerta tampoco quema, diga lo que diga tu dashboard favorito: esos tokens siguen existiendo y siguen contando en `supply`.

A tres cosas se les llama deflacionarias y solo una lo es. Una quema destruye tokens y decrementa el campo `supply` del mint, que es un número on-chain verificable que cualquiera puede leer de la misma cuenta de mint que llevas todo el curso decodificando. Una dirección de quema es una billetera de la que nadie tiene la llave, lo que saca tokens de circulación en la práctica y de nada en los datos: `supply` no se mueve, y cualquier cifra de supply circulante construida sobre eso es una convención y no un hecho. Revocar la autoridad de mint limita la emisión futura y no destruye absolutamente nada. Di cuál de las tres estás haciendo, con esas palabras, en lo que sea que publiques. Vale la pena preferir la variante `burn_checked` sobre el `burn` a secas por la misma razón por la que `transfer_checked` le ganó a `transfer`: te obliga a pasar el mint y los decimals, y el programa se niega si no concuerdan con la cuenta. Un error de decimals en una quema es irrecuperable de un modo en el que un error de decimals en una transferencia normalmente no lo es.

La trampa está en la lectura, no en la escritura. Si haces fetch del mint, después quemas, y después reportas desde el objeto que trajiste antes, vas a reportar el supply viejo y tu assertion va a pasar o fallar por razones que no tienen nada que ver con tu código. Cualquier cosa que decodificaste antes de una transacción es una fotografía, no un feed en vivo. Vuelve a hacer fetch del mint después de que la quema confirme. El equivalente en Anchor de esto es llamar a `.reload()` después de una CPI que tocó tu cuenta, y el modo de falla es idéntico en los dos mundos.

![Seis líneas de código anotadas recorren desde un fetch de supply previo a la quema, pasando por harvest, compra y quema, hasta un re-fetch obligatorio y la assertion de que el supply bajó en el monto quemado.](assets/v07-annotated-code.png)

## Lab: arma el riel de comisiones de Overgrowth

Estás construyendo `sprout-economy`, el peldaño que convierte a SPROUT de un token con comisión en un token con economía. Consume dos cosas que ya tienes: `sprout-mint` del módulo 2, que es donde vive la config de comisión, y `sprout-launch` de las lecciones de lanzamiento, cuya decisión de plataforma es la razón por la que SPROUT se graduaría en Meteora DBC hacia un pool DAMM v2 y no en un launchpad de SPL clásico que no puede sostener su mint. Cuatro módulos de trabajo convergen en un solo script.

Córrelo contra surfpool, forkeado desde mainnet, para que el programa DAMM v2 y sus cuentas sean reales. Si surfpool no está ya corriendo de los labs anteriores, `surfpool start --no-tui --no-studio` en otra terminal es toda la ceremonia (instalación: `brew install txtx/taps/surfpool`, o `cargo install surfpool-cli`; verificado en 1.2.1).

**1. Fija la toolchain.** Dos líneas, y la segunda necesita una palabra de honestidad.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install @meteora-ag/cp-amm-sdk@1.4.6 @solana/web3.js@1.98.4 bn.js@5.2.2
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node @types/bn.js
```

Verificado contra npm el 2026-09-05: la etiqueta `latest` de kit es 8.2.0, publicada el 2026-08-29, pero la primera línea fija por rango de peers, no por latest: `@solana-program/token-2022@0.15.0` es la minor actual que peerea con kit `^7.0.0` — el release 0.16.0 saltó a `^8` — así que kit se queda en 7.1.1, el release más nuevo dentro de ese rango, y `@solana-program/system@0.13.0` coincide. Vuelve a correr `npm view @solana-program/token-2022@0.15.0 peerDependencies` cuando hagas el scaffold; esta matriz se mueve cada mes.

La segunda línea es la interesante, y solo hace falta si tomas la puerta A de abajo. `@meteora-ag/cp-amm-sdk` es el cliente DAMM v2 propio de Meteora y ya viene con tipos de web3.js v1, no de kit. Vas a correr dos clientes en un mismo workspace, y eso no es un error que te esté escondiendo: es cómo se ve de verdad integrar con un SDK propio del proveedor en 2026. Kit hace los tramos de Token-2022 porque ahí es donde kit es excelente. Web3.js v1 hace el tramo del swap porque eso es lo que habla el SDK de la propia plataforma. El pin de 1.4.6 es una lectura de npm del 2026-08-21, apropiada para un SDK cuya maquinaria más profunda este curso deriva en vez de enseñar; corre `npm view @meteora-ag/cp-amm-sdk version` el día que hagas el scaffold.

Nombra la regla sobre la que cabalga este arreglo, porque m05-l1 la enunció y este lab es donde se pone a prueba: Meteora no publica ninguna superficie kit para DAMM v2, así que la dependencia v1 es inevitable, y se queda en cuarentena en **exactamente un archivo**, `venue.ts`. Nada más en el lab importa web3.js, ni siquiera de forma indirecta — `venue.ts` construye su propia `Connection`, firma con su propio `Keypair`, y le entrega al resto del riel valores planos. Esa es la cuarentena funcionando: los dos stacks se encuentran en la blockchain, no en una lista de imports compartida. Si te descubres buscando un `PublicKey` en `wire-economy.ts`, la costura tiene una fuga y el arreglo es empujar la fuga de vuelta adentro de `venue.ts`.

**1b. Crea `treasury.json`, la clave con la que firma todo el riel.** Ningún módulo anterior creó este archivo, y ese es un hueco para cerrar ahora en vez de descubrirlo en el paso 6: los scripts de m02 dejaron cada autoridad en firmantes desechables en memoria, algo que está bien para mints desechables e inútil para un riel cuyo tramo de withdraw tiene que poder firmarse la semana que viene. Genera la clave una vez y fondéala en el fork:

```bash
solana-keygen new --no-bip39-passphrase -o labs/m09-l1/treasury.json
solana airdrop 100 "$(solana-keygen pubkey labs/m09-l1/treasury.json)" --url http://127.0.0.1:8899
```

Después haz que la blockchain esté de acuerdo en que esta clave tiene los poderes que el riel ejerce. Las autoridades de comisión se fijan al crear el mint, y las claves desechables que las tenían en cualquier SPROUT anterior murieron con su proceso — así que este es exactamente el caso de "vuelve a acuñar según la apertura de m05-l1" de los supuestos de partida, con una edición primero. Abre el builder de SPROUT compuesto (`labs/m02-l1/verify-economics.ts`, tal como se re-apuntó en el paso 7 de m02-l4), carga el firmante de la tesorería arriba del todo con las mismas dos líneas de `createKeyPairSignerFromBytes` que `wire-economy.ts` usa más abajo, y pasa ese firmante en lugar del desechable para exactamente dos roles: la autoridad de mint y el `withdrawWithheldAuthority` de la config de comisión. Vuelve a acuñar, vuelve a correr tus transferencias del marketplace, y el fork ahora lleva un SPROUT cuyo frasco de comisiones este archivo puede abrir — y cuyo supply puede acuñar la ventana de conversión de la próxima lección, que es la razón por la que la autoridad de mint pasa a la misma clave. Un eco de la sección de teoría, para que el código no pueda contradecirlo en tu cabeza: producción quiere una PDA en este rol, no un archivo JSON; cada lugar donde firma esta clave es un lugar donde tu programa haría `invoke_signed`.

No necesitas abrir a mano la cuenta de token SPROUT de la tesorería, y la asimetría con el `--fund-recipient` del paso 1c es deliberada y no un descuido: `wire-economy.ts` la crea por sí mismo, de forma idempotente, como la primera instrucción de la transacción de harvest. Eso es a propósito. La cuenta del maker la monta un humano una sola vez antes de que el riel exista, mientras que la de la tesorería es una precondición de la primera escritura del propio riel, así que el riel es su dueño y una re-corrida en frío no puede dejarte una dirección derivada que nunca se creó.

**1c. Elige la puerta del tramo 3, y levanta su contraparte.** El buyback necesita a alguien a quien comprarle. Dos puertas, y el riel río abajo no puede distinguirlas, que es justo el punto.

*Puerta A, el mercado.* Lanzaste SPROUT tú mismo en Meteora DBC, llevaste la curva más allá de tu `migration_quote_threshold`, y guardaste la dirección del pool DAMM v2 que imprimió la migración. Nada en este curso te lleva de la mano por eso, y no voy a fingir lo contrario. Si tienes ese pool, fija `SPROUT_POOL` y el tramo 3 es un swap real con slippage real contra estado forkeado real.

*Puerta B, el maker.* No lo hiciste, porque seguiste el curso. SPROUT no tiene mercado, así que levantas uno: una sola contraparte que tiene SPROUT y está dispuesta a vender a un precio que tú fijas. Esto no es un mercado y el lab no lo va a llamar así. Lo que conserva es cada propiedad del riel que no depende del descubrimiento de precio — el harvest, la conservación del reparto, la comisión disparándose en el pago, medir en vez de calcular, y la assertion de supply al final. Lo que deja fuera son el slippage y el MEV, que necesitan un libro de órdenes contra el cual existir.

```bash
solana-keygen new --no-bip39-passphrase -o labs/m09-l1/maker.json
solana airdrop 10 "$(solana-keygen pubkey labs/m09-l1/maker.json)" --url http://127.0.0.1:8899
```

Después dale al maker algo que vender. Tu clave de tesorería tiene la autoridad de mint después del paso 1b, así que un solo `spl-token mint` pone inventario en los libros del maker; dimensiónalo bien por encima de lo que sea que la tesorería vaya a gastar, o el tramo 3 falla por el saldo del vendedor y no por nada de lo que estabas tratando de aprender:

```bash
spl-token mint "$SPROUT_MINT" 1000 \
  --recipient-owner "$(solana-keygen pubkey labs/m09-l1/maker.json)" \
  --mint-authority labs/m09-l1/treasury.json \
  --fund-recipient --url http://127.0.0.1:8899
```

Di la parte que nadie dice antes de correr el riel: un precio de buyback que fijas tú mismo no es un precio de mercado. En una plataforma, el número es la razón entre reservas y el mercado te lo entrega. Aquí eres los dos lados del canje, así que el precio es una decisión de gobernanza que solo parece una constante, y cada conclusión que saques de una corrida por la puerta B hereda eso. El módulo venue del paso 3 existe para que el día que SPROUT sí tenga un pool, la única línea que cambia sea cuál puerta abriste.

**2. Encuentra el montón.** Crea `find-withheld.ts`. Este es el escaneo de cuentas, y es la herramienta sobre la que se construye el resto del riel.

```ts
// find-withheld.ts: which SPROUT accounts are sitting on withheld marketplace fees?
import type {
  Address,
  Base58EncodedBytes,
  GetMultipleAccountsApi,
  GetProgramAccountsApi,
  Rpc,
} from "@solana/kit";
import { TOKEN_2022_PROGRAM_ADDRESS, fetchAllMaybeToken } from "@solana-program/token-2022";

export type DirtyAccount = { account: Address; withheld: bigint };

/** Every token account of `mint` carrying a nonzero TransferFeeAmount.withheldAmount. */
export async function findWithheld(
  rpc: Rpc<GetProgramAccountsApi & GetMultipleAccountsApi>,
  mint: Address,
): Promise<DirtyAccount[]> {
  // Token account layout puts the mint at offset 0, so one memcmp finds every holder.
  const holders = await rpc
    .getProgramAccounts(TOKEN_2022_PROGRAM_ADDRESS, {
      encoding: "base64",
      withContext: false,
      dataSlice: { offset: 0, length: 0 },
      filters: [
        { memcmp: { offset: 0n, bytes: mint as string as Base58EncodedBytes, encoding: "base58" } },
      ],
    })
    .send();

  const decoded = await fetchAllMaybeToken(
    rpc,
    holders.map((h) => h.pubkey),
  );

  const dirty: DirtyAccount[] = [];
  for (const account of decoded) {
    if (!account.exists) continue;
    const extensions = account.data.extensions;
    if (extensions.__option !== "Some") continue;
    for (const ext of extensions.value) {
      if (ext.__kind === "TransferFeeAmount" && ext.withheldAmount > 0n) {
        dirty.push({ account: account.address, withheld: ext.withheldAmount });
      }
    }
  }
  return dirty;
}
```

El `dataSlice: { offset: 0, length: 0 }` importa más de lo que parece. El escaneo necesita direcciones, no datos, así que le pides al RPC cero bytes por cuenta y después traes y decodificas solo lo que encontraste. En un RPC público con un conjunto grande de tenedores, la versión que trae los datos completos de cada tenedor es la versión que hace que te limiten la tasa.

**3. La plataforma de la puerta A.** Crea `venue.ts`. Este es el tramo del swap, y es pequeño porque el SDK hace el trabajo. También es toda la superficie de web3.js del lab: la `Connection`, el `Keypair`, los `PublicKey`s y el envío viven aquí adentro y nunca se escapan.

```ts
// venue.ts: SPROUT's graduation venue, read and traded client-side.
// web3.js v1 here on purpose AND NOWHERE ELSE: the first-party Meteora SDK
// ships v1 types, so this file is the quarantine. It takes strings and bytes,
// it returns bigints and strings, and nothing v1-shaped crosses its boundary.
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";
import { CpAmm, CP_AMM_PROGRAM_ID } from "@meteora-ag/cp-amm-sdk";

export const DAMM_V2_PROGRAM = CP_AMM_PROGRAM_ID.toBase58();

// The two token programs the pool straddles: SPROUT is Token-2022, wSOL is classic.
const TOKEN_2022 = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const TOKEN_CLASSIC = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

export type Venue = {
  /** lamports of quote per one base unit of SPROUT, floored */
  priceLamportsPerToken: bigint;
  /** Sends the buy and returns only once it has CONFIRMED. Measuring is the caller's job. */
  buy: (quoteLamports: bigint, slippagePct: number) => Promise<string>;
};

/**
 * Open a DAMM v2 pool on the fork and expose a client-side buy.
 * Throws if the pool is absent, because guessing which pool you meant is not
 * this module's job: SPROUT only has one if you launched and migrated it.
 */
export async function openVenue(
  rpcUrl: string,
  baseMint: string,
  pool: string,
  buyerSecret: Uint8Array,
): Promise<Venue> {
  const connection = new Connection(rpcUrl, "confirmed");
  const buyer = Keypair.fromSecretKey(buyerSecret);
  const poolKey = new PublicKey(pool);
  const baseKey = new PublicKey(baseMint);

  if ((await connection.getAccountInfo(poolKey)) === null) {
    throw new Error(`no DAMM v2 pool at ${pool} for ${baseMint}: this fork has no market for that mint`);
  }

  const cpAmm = new CpAmm(connection);
  const state = await cpAmm.fetchPoolState(poolKey);
  if (!state.tokenAMint.equals(baseKey)) {
    throw new Error(`pool ${pool} does not carry ${baseMint} as its base mint: wrong pool`);
  }

  // The only AMM math this course does: spot price is the vault-reserve ratio.
  const base = BigInt((await connection.getTokenAccountBalance(state.tokenAVault)).value.amount);
  const quote = BigInt((await connection.getTokenAccountBalance(state.tokenBVault)).value.amount);
  const priceLamportsPerToken = base === 0n ? 0n : quote / base;

  return {
    priceLamportsPerToken,
    buy: async (quoteLamports: bigint, slippagePct: number) => {
      // The SDK's slippage dial is minimumAmountOut: the spot-sized fill, shaved by your tolerance.
      const atSpot = priceLamportsPerToken === 0n ? 0n : quoteLamports / priceLamportsPerToken;
      const minOut = (atSpot * BigInt(100 - slippagePct)) / 100n;
      const tx = await cpAmm.swap({
        payer: buyer.publicKey,
        pool: poolKey,
        inputTokenMint: state.tokenBMint, // quote (wSOL) in; the builder handles the wrap
        outputTokenMint: state.tokenAMint, // SPROUT out
        amountIn: new BN(quoteLamports.toString()),
        minimumAmountOut: new BN(minOut.toString()),
        tokenAMint: state.tokenAMint,
        tokenBMint: state.tokenBMint,
        tokenAVault: state.tokenAVault,
        tokenBVault: state.tokenBVault,
        tokenAProgram: TOKEN_2022,
        tokenBProgram: TOKEN_CLASSIC,
        referralTokenAccount: null,
      });
      // v1's sendTransaction returns at SUBMISSION, not confirmation. Read the
      // treasury balance before this settles and you measure a pre-swap
      // photograph (the burn section's stale-read trap, client-side edition),
      // so the confirm belongs in here rather than in the caller's hopes.
      const swap = new Transaction().add(...tx.instructions);
      const signature = await connection.sendTransaction(swap, [buyer], { skipPreflight: false });
      const latest = await connection.getLatestBlockhash("confirmed");
      await connection.confirmTransaction({ signature, ...latest }, "confirmed");
      return signature;
    },
  };
}
```

Revisa las dos assertions al principio de `openVenue`. Si el pool no está ahí, la primera lanza en vez de adivinar. Si el pool existe pero lleva algún otro mint base, la segunda lanza antes de que operes en el mercado de otro. Una herramienta que falla ruidosamente en el límite de su propia responsabilidad vale por diez que devuelven `undefined` y dejan que la falla aparezca tres funciones más tarde. Los lectores de la puerta B deberían igual demostrar que ese límite es real en vez de creerme, y cuesta un solo comando sin necesidad de pool:

```bash
SPROUT_MINT=<mint> npx tsx -e "import {openVenue} from './venue'; \
  await openVenue('http://127.0.0.1:8899', process.env.SPROUT_MINT!, '11111111111111111111111111111112', new Uint8Array(64));"
```

Esa dirección es una cuenta real y no un pool DAMM v2, así que te sale el primer throw, por nombre, en menos de un segundo. El módulo está inerte hasta que SPROUT tenga mercado; no está roto.

**3b. La contraparte de la puerta B.** Crea `maker.ts`. Misma interfaz, sin SDK de terceros, sin web3.js — este es propio y por lo tanto es kit hasta el fondo.

```ts
// maker.ts: leg 3's counterparty when the token has no market.
// The price is a POLICY input, not a reserve ratio. Everything else about the
// leg is identical to the venue's: SOL out, SPROUT in, the fee fires on the
// payout, and the caller measures what landed instead of computing it.
import type { Address, Instruction, TransactionSigner } from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";
import {
  findAssociatedTokenPda,
  getTransferCheckedInstruction,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";

export type Maker = {
  priceLamportsPerToken: bigint;
  buyIxs: (quoteLamports: bigint) => Instruction[];
};

export async function openMaker(
  mint: Address,
  decimals: number,
  maker: TransactionSigner,
  buyer: TransactionSigner,
  priceLamportsPerToken: bigint,
): Promise<Maker> {
  if (priceLamportsPerToken <= 0n) {
    throw new Error("MAKER_PRICE must be a positive lamport price per base unit");
  }
  const [makerAta] = await findAssociatedTokenPda({
    mint,
    owner: maker.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  const [buyerAta] = await findAssociatedTokenPda({
    mint,
    owner: buyer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  return {
    priceLamportsPerToken,
    // Both legs of the trade in ONE transaction, so neither side can take the
    // money and walk. Two signers, one atomic settlement: this is the smallest
    // honest OTC trade you can write, and it is what an escrow program
    // automates when the counterparty is a stranger rather than your own key.
    buyIxs: (quoteLamports: bigint) => [
      getTransferSolInstruction({
        source: buyer,
        destination: maker.address,
        amount: quoteLamports,
      }),
      getTransferCheckedInstruction({
        source: makerAta,
        mint,
        destination: buyerAta,
        authority: maker,
        amount: quoteLamports / priceLamportsPerToken,
        decimals,
      }),
    ],
  };
}
```

La división entera de `amount` es la misma que hace `routeFees`, por la misma razón: no puedes comprar una fracción de unidad base. Y fíjate en lo que este archivo NO hace. No calcula lo que la tesorería va a recibir. Transfiere `quoteLamports / price` unidades, la comisión de transferencia se descuenta de eso en vuelo, y lo que llega es menor. Nadie aquí afirma cuánto.

**4. El reparto, y este te toca a ti.** Crea `route-fees.ts` con la firma de abajo. El cuerpo tiene dos TODOs y la sección de teoría ya te dio las dos respuestas con palabras simples.

```ts
// route-fees.ts: split the harvest, size the buyback. Pure arithmetic, no chain.
export function routeFees(
  harvested: bigint,
  burnBps: number,
  treasurySol: bigint,
  priceLamportsPerToken: bigint,
): { burnedFromFees: bigint; toTreasury: bigint; buyback: bigint } {
  const burnedFromFees = (harvested * BigInt(burnBps)) / 10000n;
  // TODO: what stays in the treasury, such that the two shares sum back to `harvested`
  const toTreasury = 0n;
  // TODO: how many SPROUT base units does `treasurySol` buy at this price, floored
  const buyback = 0n;
  return { burnedFromFees, toTreasury, buyback };
}
```

Mantén todo en bigints. En el momento en que un `Number` toca un conteo de lamports por encima de 2^53 tu aritmética empieza a mentir en voz baja, y los conteos de lamports llegan ahí más rápido de lo que crees.

**5. El riel.** Crea `wire-economy.ts`. Este es el artefacto, y se lee de arriba abajo como los cuatro tramos en orden.

```ts
// wire-economy.ts: SPROUT's fee rail, end to end, against the mainnet fork.
// Kit only. The one web3.js dependency in this lab lives behind venue.ts.
import {
  address,
  appendTransactionMessageInstructions,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  fetchMint,
  fetchToken,
  getBurnCheckedInstruction,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getHarvestWithheldTokensToMintInstruction,
  getWithdrawWithheldTokensFromMintInstruction,
  findAssociatedTokenPda,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { findWithheld } from "./find-withheld";
import { openMaker } from "./maker";
import { routeFees } from "./route-fees";
import { openVenue } from "./venue";

const RPC_HTTP = process.env.RPC_HTTP ?? "http://127.0.0.1:8899";
const RPC_WS = process.env.RPC_WS ?? "ws://127.0.0.1:8900";
const SPROUT = address(process.env.SPROUT_MINT!);
const SPROUT_DECIMALS = 6;
const SPROUT_POOL = process.env.SPROUT_POOL; // door A only: a DAMM v2 pool that holds SPROUT
const MAKER_KEY = process.env.MAKER_KEY; // door B: the counterparty from step 1c
const MAKER_PRICE = BigInt(process.env.MAKER_PRICE ?? "1000000"); // door B: lamports per base unit
const BURN_BPS = 2000; // 20% of every harvest burns on arrival
const SLIPPAGE_PCT = 1;

const rpc = createSolanaRpc(RPC_HTTP);
const rpcSubscriptions = createSolanaRpcSubscriptions(RPC_WS);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

function loadSecret(path: string): Uint8Array {
  return new Uint8Array(JSON.parse(readFileSync(path, "utf8")));
}

async function send(payer: KeyPairSigner, ixs: Instruction[]): Promise<void> {
  const { value: blockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
}

async function main(): Promise<void> {
  const secret = loadSecret(process.env.TREASURY_KEY!);
  const treasury = await createKeyPairSignerFromBytes(secret);
  const [treasuryAta] = await findAssociatedTokenPda({
    mint: SPROUT,
    owner: treasury.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const supplyBefore = (await fetchMint(rpc, SPROUT)).data.supply;

  // LEGS 1 + 2: harvest (permissionless consolidate) then withdraw
  // (authority-gated collect), one transaction. Fees are withheld on
  // recipient accounts until someone moves them.
  const dirty = await findWithheld(rpc, SPROUT);
  const harvested = dirty.reduce((sum, d) => sum + d.withheld, 0n);
  console.log(`dirty accounts: ${dirty.length}, withheld total: ${harvested}`);
  if (harvested === 0n) throw new Error("nothing withheld: run marketplace trades first");

  // Open the treasury's SPROUT account before anything tries to pay into it.
  // `findAssociatedTokenPda` above DERIVED an address; deriving is not creating,
  // and every leg from here down writes to this account: leg 2 names it as
  // `feeReceiver`, leg 3 has the counterparty transfer into it, leg 4 burns out
  // of it. Idempotent, so re-running the rail is free.
  const openTreasuryAta = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer: treasury,
    owner: treasury.address,
    mint: SPROUT,
  });

  await send(treasury, [
    openTreasuryAta,
    getHarvestWithheldTokensToMintInstruction({
      mint: SPROUT,
      // Fork scale: a handful of dirty accounts fits one transaction, so the
      // whole list rides in one instruction. At fleet scale this is where the
      // packing problem bites: wrap the list in the chunk() helper from the
      // "what the crank costs to run" section and send one harvest per batch.
      sources: dirty.map((d) => d.account),
    }),
    getWithdrawWithheldTokensFromMintInstruction({
      mint: SPROUT,
      feeReceiver: treasuryAta,
      withdrawWithheldAuthority: treasury,
    }),
  ]);

  // BETWEEN LEGS: get a price, then run the split. The split is arithmetic,
  // not a leg of its own; its output sizes and predicts leg 3.
  //
  // Door A reads the price off a real pool's reserves. Door B is told the
  // price, because with no market there is nothing to read it from. The rest
  // of this function cannot tell which one ran, which is the whole design.
  const treasurySol = (await rpc.getBalance(treasury.address).send()).value / 2n;
  let priceLamportsPerToken: bigint;
  let buy: () => Promise<void>;

  if (SPROUT_POOL) {
    const venue = await openVenue(RPC_HTTP, SPROUT, SPROUT_POOL, secret);
    priceLamportsPerToken = venue.priceLamportsPerToken;
    buy = async () => {
      await venue.buy(treasurySol, SLIPPAGE_PCT);
    };
    console.log(`door A: DAMM v2 pool ${SPROUT_POOL} at ${priceLamportsPerToken} lamports/unit`);
  } else {
    if (!MAKER_KEY) throw new Error("set SPROUT_POOL (door A) or MAKER_KEY (door B); step 1c picks one");
    const maker = await createKeyPairSignerFromBytes(loadSecret(MAKER_KEY));
    const otc = await openMaker(SPROUT, SPROUT_DECIMALS, maker, treasury, MAKER_PRICE);
    priceLamportsPerToken = otc.priceLamportsPerToken;
    buy = async () => {
      await send(treasury, otc.buyIxs(treasurySol));
    };
    console.log(
      `door B: no SPROUT market on this fork; buying from maker ${maker.address} ` +
        `at ${priceLamportsPerToken} lamports/unit (a price you set, not a price you read)`,
    );
  }

  const plan = routeFees(harvested, BURN_BPS, treasurySol, priceLamportsPerToken);
  if (plan.toTreasury === 0n || plan.buyback === 0n) {
    throw new Error(
      "route-fees.ts TODOs look unfilled: a zero treasury share or zero-sized buyback plan means the split never ran. Fill them before running the rail.",
    );
  }
  console.log(
    `split: burn ${plan.burnedFromFees} + keep ${plan.toTreasury} = ${harvested}; ` +
      `buyback target ~${plan.buyback} SPROUT at ${priceLamportsPerToken} lamports/unit`,
  );

  // LEG 3: the buyback. Whichever door opened, it is sized in SOL and it costs
  // you something. plan.buyback is the floor-division PREDICTION of what that
  // SOL buys; the planned-vs-bought line below is leg 3's cost made visible.
  const balanceBefore = (await fetchToken(rpc, treasuryAta)).data.amount;
  await buy();
  // Re-read the account AFTER the buy confirms. Cached balances are how supply math goes wrong.
  const balanceAfter = (await fetchToken(rpc, treasuryAta)).data.amount;
  const bought = balanceAfter - balanceBefore;
  console.log(`bought ${bought} SPROUT (planned ${plan.buyback}, the gap is what leg 3 cost you)`);

  // LEG 4: burn exactly what the buyback bought, plus the fee-burn share.
  const toBurn = bought + plan.burnedFromFees;
  await send(treasury, [
    getBurnCheckedInstruction({
      account: treasuryAta,
      mint: SPROUT,
      authority: treasury,
      amount: toBurn,
      decimals: 6,
    }),
  ]);

  const supplyAfter = (await fetchMint(rpc, SPROUT)).data.supply;
  const dropped = supplyBefore - supplyAfter;
  console.log(`supply ${supplyBefore} -> ${supplyAfter} (down ${dropped}, burned ${toBurn})`);
  if (dropped !== toBurn) throw new Error(`supply drop ${dropped} != burn ${toBurn}`);
  console.log("rail closed: harvested, split, bought back, burned");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

**6. Córrelo, con los TODOs de `route-fees.ts` llenados primero.** El riel ahora se niega a correr contra el stub (un reparto en cero lanza antes de que se mueva un solo lamport), así que este paso asume que el paso 4 está hecho. Cada comando de este paso corre desde dentro de `labs/m09-l1/`, que es donde los pasos 1b a 5 pusieron los archivos; las rutas relativas de claves de abajo lo asumen.

```bash
cd labs/m09-l1
# door B, the one the course guarantees
SPROUT_MINT=<mint> MAKER_KEY=./maker.json TREASURY_KEY=./treasury.json npx tsx wire-economy.ts
# door A, if you launched SPROUT yourself and have its pool
SPROUT_MINT=<mint> SPROUT_POOL=<pool> TREASURY_KEY=./treasury.json npx tsx wire-economy.ts
```

Una corrida sana por la puerta B dice algo cercano a esto. El total retenido y el saldo de la tesorería son tuyos, así que solo se traslada la *forma*; la aritmética entre las líneas es lo que revisas.

```text
dirty accounts: 7, withheld total: 2500000
door B: no SPROUT market on this fork; buying from maker 8k2...Uq at 1000000 lamports/unit (a price you set, not a price you read)
split: burn 500000 + keep 2000000 = 2500000; buyback target ~50000 SPROUT at 1000000 lamports/unit
bought 49500 SPROUT (planned 50000, the gap is what leg 3 cost you)
supply 1000000000000000 -> 999999999450500 (down 549500, burned 549500)
rail closed: harvested, split, bought back, burned
```

Lee esas cinco líneas unas contra otras, porque solo concuerdan si el riel funcionó. La línea del reparto suma: 500,000 más 2,000,000 es 2,500,000, exactamente lo que encontró el escaneo. El objetivo del buyback es la mitad gastable de la tesorería, alrededor de 50 SOL después del airdrop del paso 1b, dividida por el precio. Y la cuarta línea es la sincera: planeaste 50,000 y llegaron 49,500, porque SPROUT cobra su propia comisión de 100 bps sobre el pago que el maker te hace y 500 unidades se quedaron atrás como retenidas — en la cuenta de tu propia tesorería, esperando el próximo harvest, que es la circularidad de la que advirtió la sección de teoría hecha visible. En la puerta A esa misma línea también llevaría slippage y la comisión de la plataforma, y el número sería todavía más chico. De cualquier manera pagaste algo por recomprar tu propio token, que es lo que un buyback siempre ha sido una vez que le quitas al término su marketing.

![Un gráfico de dos barras enfrenta un buyback planeado con la cantidad menor realmente recibida, atribuyendo la brecha al impacto en el precio, la comisión de la plataforma y la propia comisión de transferencia del token.](assets/v08-chart.png)

Si la corrida lanza `supply drop != burn`, casi seguro calculaste `bought` en vez de medirlo, o reusaste el objeto del mint previo a la quema. Los dos son el mismo error.

## Challenge

**Completion.** Llena los dos TODOs de `route-fees.ts` y demuéstralos con el coding challenge del módulo, `route-fees`, que te entrega una versión rota y una suite de pruebas. Una convención difiere de tu copia del proyecto: el archivo calificado es autónomo, así que declara una `function routeFees(harvested, burnBps, treasurySol, priceLamportsPerToken)` a secas sin la palabra clave `export`, porque el grader inserta tu código en su propio runner y llama a la función directo con esos cuatro argumentos posicionales, bigints para los tres montos y un number simple para los basis points. Deja el `export` en la copia del proyecto que `wire-economy.ts` importa; quítalo en el editor del challenge. El starter rompe la conservación, porque `toTreasury` ignora la parte quemada, y dimensiona mal el buyback, porque multiplica por el precio en vez de dividir. Tu versión tiene que sostener `burnedFromFees + toTreasury === harvested` en cada entrada, truncar el buyback hacia abajo sobre bigints, dejar todo el harvest en la tesorería con una parte quemada de 0 bps, y llevar `toTreasury` a cero con 10000 bps.

**Solo.** Arma el riel completo tú mismo contra el fork y demuéstralo. Genera volumen en el marketplace primero, al menos una docena de transferencias entre varios compradores para que el escaneo encuentre trabajo real que hacer, después corre `wire-economy.ts` de punta a punta y produce cuatro números: el monto del harvest, el delta de la tesorería, la cantidad de buyback realmente recibida, y el delta de supply después de la quema. El criterio es la assertion que ya está en el script: el supply bajó exactamente lo que quemaste, ni más ni menos.

![Una tabla de puntuación lista el monto del harvest, el delta de la tesorería, la cantidad de buyback y el delta de supply después de la quema, cada uno con su fuente, la afirmación que demuestra y su falla característica.](assets/v09-table.png)

**El sondeo empírico, si quieres la respuesta real a una pregunta que esta lección solo insinuó.** Corre el buyback dos veces, una con una tajada pequeña de la tesorería y otra con todo, y registra la brecha entre lo entregado y lo planeado cada vez. Después mira la cuenta de token de la propia tesorería y encuentra el SPROUT retenido que está sentado en ella, comisiones que tu propio buyback se pagó a sí mismo.

En la puerta B el segundo número es el franco y el primero es un control: la brecha va a escalar exactamente con el tamaño de la orden, porque el único costo en juego es una comisión porcentual con un tope, y verla *no* portarse mal es cómo aprendes cómo se habría visto el slippage si hubiera habido un libro de órdenes. En la puerta A el primer número es el que enseña, porque la brecha se ensancha de forma superlineal a medida que tu orden se come el pool, y esa curva es la que llena con tus propios valores la tabla de políticas que está en la sección de teoría. La elección entre barrer todo cada mes y comprar órdenes pequeñas de forma continua es una que ninguna lección puede tomar por ti, porque depende de la profundidad de tu pool y no de tus intenciones — y la puerta B, al no tener profundidad, no puede responderla en absoluto. Saber cuál de esas dos corridas tienes en la mano es la misma habilidad que m05-l1 pidió cuando separó un predictor de una demostración.

## Checkpoint

Terminaste cuando una sola corrida del script imprime los cuatro números y sale con cero: monto del harvest, delta de la tesorería, cantidad de buyback, delta de supply después de la quema, con la caída de supply igual a la quema hasta la unidad base. Guarda esa salida. La lección capstone te pide componer una economía a partir de las primitivas que construiste, y este riel es la pieza que hace que la palabra economía sea honesta en vez de decorativa.

Una cosa más antes de que cierres la carpeta, y es la parte que nunca aparece en el hilo de lanzamiento cuando diseñas un riel de comisiones. Nombra las dos señales que te dirían que esta política está mal, y nómbralas ahora mientras no tienes posición emocional sobre la respuesta. Señal uno: la brecha entre lo entregado y lo planeado en tus buybacks. Si se mantiene chica, el tamaño de tu orden le queda bien a tu pool y comprar de forma continua es barato. Si se ensancha a medida que crece la tesorería, estás pagando un impuesto creciente por convertir ingresos en quema, y en algún punto enrutar ese SOL hacia algo que no sea un buyback es el mejor uso que le puedes dar. Señal dos: la razón entre lo que cuesta correr el crank y lo que recauda. Un harvest que barre menos valor de lo que cuesta mandar las transacciones no es un riel de comisiones, es un pasatiempo, y la respuesta sincera es barrer menos seguido en vez de fingir que el calendario está funcionando. Anota los dos umbrales con números reales de tus propias corridas. Una política que nadie puede refutar es un eslogan.

Tres fallas que espero. La primera es un harvest que reporta cero en un mint que claramente cobra comisiones, lo que casi siempre quiere decir que el filtro memcmp está calzando con el offset equivocado o el programa equivocado, ya que un mint de SPL clásico y un mint de Token-2022 tienen dueños distintos y el escaneo está acotado por programa. La segunda es un `withdraw_withheld_tokens_from_mint` que falla por autoridad, lo que quiere decir que pasaste una dirección donde el builder quería un firmante. La tercera es que la compra de la puerta B falle por el saldo del maker, lo que quiere decir que el paso 1c acuñó menos inventario del que puede comprar la mitad de tu tesorería a `MAKER_PRICE`; acuña más o sube el precio, y nota que tener que elegir es en sí mismo la señal de que tú eres el mercado en vez de estar operando en uno. Si los números todavía se niegan a cuadrar después de que hayas revisado las tres, lleva la salida de tu corrida y tu aritmética esperada a la discusión del curso, y en la puerta A publica las reservas del pool junto a ellas, porque la mitad de las veces el desacuerdo es slippage y no un bug y las reservas son lo que lo demuestra.

SPROUT ahora gana comisiones hacia una tesorería que controla y quema supply según un calendario que tú fijas. Lo que no hace es importarle quién lo tiene. Siguiente: decidir quién entra, y convertir los puntos de compost que Overgrowth ha estado rastreando off-chain en SPROUT real que la gente pueda gastar de verdad.
