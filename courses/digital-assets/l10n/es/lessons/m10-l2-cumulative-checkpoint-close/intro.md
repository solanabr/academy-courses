# Cierra los apuntes: tres briefs en frío, y la puerta que cruzas después

## Resumen

R13, el capstone, fue la última construcción que este curso te pide. Quedan dos jugadas, exactamente como lo prometió el cierre del capstone: un checkpoint acumulativo sobre tres briefs en frío, puntuado contra la matriz de conflictos del módulo uno y la tesis de compatibilidad del módulo cinco, y después el cierre. Un brief es nuevo, otro vuelve a correr la forma de insignia masiva a la que el capstone le puso precio pero que nunca te hizo construir, y otro se sienta deliberadamente al lado del memo de la cafetería ya resuelto, el ancla de calibración. El repliegue es total. Ninguna respuesta resuelta llega antes que la tuya.

Así que cierra los apuntes. Pestañas del navegador incluidas. Aquí están los tres briefs.

**Brief uno, la cafetería de la esquina.** Una cadena de cafeterías con once locales quiere una moneda de lealtad. Los clientes la ganan por compra y pueden vendérsela entre ellos, y el dueño es tajante en que tiene que poder listarse en Raydium para que el precio sea público. No hay ningún regulador involucrado, no se quieren controles de compliance, y alrededor de 40,000 clientes van a tener saldo.

**Brief dos, la insignia de finisher.** Un club de running quiere entregarle a cada uno de su millón de miembros una insignia por terminar un maratón virtual. Tiene que salir barato a ese conteo, y una insignia que aparece a la venta en un marketplace es una vergüenza que no van a aceptar.

**Brief tres, la participación de la cooperativa.** Una cooperativa de alimentos tokeniza las participaciones de sus miembros, una emisión por miembro, unas 900, divisibles porque el reparto anual de excedentes cae en fracciones. El consejo tiene que poder congelar una participación cuando se expulsa a un miembro, y la participación nunca tiene que negociarse en una plataforma pública.

![Una tabla de decisión en blanco de seis columnas con una fila por brief en frío y celdas vacías para familia de primitivas, conjunto de extensiones, veredicto de compatibilidad, riel de economía y defensa.](assets/v01-table.webp)

Ahora llena cuatro celdas por cada brief, en un archivo de texto, a mano: la familia de primitivas, el conjunto de extensiones o de plugins, el veredicto de compatibilidad, el único riel de economía. Una oración de defensa por fila. Sin buscar, sin volver a desplazarte hacia arriba. Dale quince minutos y acepta lo que salga, con huecos y todo.

## El checkpoint es un espejo, no un examen

Vale la pena nombrar dos palabras antes de seguir, porque son el método entero. Un **checkpoint acumulativo** es una pasada de recuperación a libro cerrado sobre un curso entero, producida en vez de leída, y no lleva nota y no restringe el acceso a nada. Una **tabla de decisión** es la forma comprimida de todo lo que enseñó este curso: cuatro celdas por brief, porque cuatro celdas es lo que una decisión de producto real necesita de verdad antes de que alguien abra un editor.

Quiero ser directo sobre por qué importa la regla de producirlo, porque yo estuve del lado equivocado de ella. Leer una clave de respuestas se siente exactamente igual que saber la respuesta. El reconocimiento es instantáneo, el material se ve familiar, y cierras la pestaña sintiéndote fluido en algo que no habrías podido generar. Esa sensación es la ilusión más cara que existe en el aprendizaje técnico. No te cuesta nada hoy y te cuesta un re-mint más adelante, porque el conjunto de extensiones de un mint queda fijo en la creación (las excepciones son todas escrituras de puntero-y-después-realloc elegidas al nacer de todos modos — metadatos, los TLV de grupo y las extensiones a nivel de cuenta — y ninguna extensión de poder está entre ellas), así que un conjunto equivocado no es un parche, es un mint nuevo y una migración.

La segunda regla es más suave. Una celda en blanco no es una falla, es una dirección. Cada atasco que tuviste en los últimos quince minutos apunta a exactamente un módulo, y para el final de esta lección vas a tener una tablita que mapea cada tipo de hueco a la lección que lo llena. Ese es el resultado de verdad de un checkpoint. No un puntaje.

### Los dos instrumentos contra los que revisas

Tienes dos reglas para revisar una fila, y responden preguntas distintas, en un orden estricto.

La **matriz de conflictos** del módulo uno responde "¿este mint siquiera va a inicializar?" La portaste desde la fuente de verdad, `check_for_invalid_mint_extension_combinations`, hacia `checkCombo`, una función pura sin RPC adentro. Sus reglas son estructurales: `ScaledUiAmount` e `InterestBearingConfig` son mutuamente excluyentes porque dos multiplicadores de UI distintos sobre un mismo saldo es un sinsentido. `ConfidentialMintBurn` requiere `ConfidentialTransferMint`, porque la ruta de quema necesita el supply cifrado sobre el que opera. Y los pares forzados de `required_init_account_extensions` quieren decir que elegir `TransferFeeConfig` en el mint obliga en silencio a un `TransferFeeAmount` en cada cuenta. Veintinueve variantes de producción en el enum `ExtensionType`, y la matriz es la diferencia entre las combinaciones que existen y las combinaciones que inicializan.

La **tesis de compatibilidad** del módulo cinco responde una pregunta más fría: "¿alguien va a poder negociarlo?" La allowlist de CP-Swap de Raydium es de exactamente cinco extensiones de Token-2022, y demostraste la forma de la regla con tu propio predictor. Es `every`, no `some`. Cinco extensiones de la allowlist más una extensión rechazada siguen rechazadas, porque la exposición del pool a un delegado permanente no se encoge cuando una config de comisión se sienta al lado. Raydium escribió la razón en lenguaje llano: un tenedor del delegado puede barrer cualquier cuenta de token, incluido el vault del pool. La escotilla de escape es un `MINT_WHITELIST` hardcodeado de cuatro entradas, que no es algo que pidas un martes.

![Una comparación a dos paneles de la matriz de conflictos, que pregunta si un mint inicializa, contra la tesis de compatibilidad, que pregunta si una plataforma le pone precio, revisadas en ese orden.](assets/v02-comparison.webp)

### Fila uno, revisada

El brief de la cafetería te entrega dos restricciones y una ausencia, y la ausencia es la parte ruidosa. Tiene que seguir siendo negociable. Tiene 40,000 tenedores. Y nadie pidió controles de compliance, lo que quiere decir que cada extensión de poder a la que podrías recurrir es un costo sin comprador.

Así que la fila defendible es un mint Token-2022 que lleva extensiones de solo display, `MetadataPointer` más `TokenMetadata`, y nada más. Las dos están en la allowlist, así que `isRoutable` devuelve true y la celda de veredicto dice enrutable. El riel es una ruta de comisión si el local quiere una tajada de los canjes secundarios, lo que quiere decir que `TransferFeeConfig` tiene que estar en el conjunto declarado en la creación, la regla de solo-al-nacer otra vez, y eso también está en la allowlist, también está bien. SPL clásico funcionaría también, y no está mal, solo que tira a la basura los metadatos nativos sin ninguna razón en 2026.

La trampa de esta fila es un `TransferHook`, y es una tentadora. Registrar cada transferencia de lealtad on-chain suena exactamente a lo que quiere un programa de lealtad. También invoca un programa personalizado en cada transferencia con consumo de cómputo arbitrario, que es precisamente por qué Raydium lo rechaza, y el brief dijo que el precio tiene que ser público. Si tu fila tiene un hook adentro, no te equivocaste de mecanismo. Te equivocaste de orden: satisficiste la funcionalidad antes de satisfacer la restricción que estaba declarada como inamovible.

### Fila dos, revisada

El brief de la insignia es el que separa un modelo mental actual de uno heredado, porque apila dos restricciones que antes respondían dos primitivas distintas.

Salir barato a un millón es trabajo de la compresión y de nada más. Metaplex Core es genuinamente barato por activo, a unos 0.003 SOL, publicado por el proveedor, que es un número maravilloso justo hasta que lo multiplicas por un millón y te da alrededor de 3,000 SOL. Un árbol Bubblegum v2 dimensionado para un millón de hojas es de 1,223,352 bytes, que cae en un solo dígito de SOL a cualquier tasa de rent que la red haya cobrado este año. Ese es un mejor trato por más de dos órdenes de magnitud, el tipo de brecha donde la aritmética decide, no el gusto.

No vendible es la mitad que antes rompía esto. El folclore de 2024 dice que los NFT comprimidos no se pueden congelar ni volver soulbound, lo que era cierto entonces y es simplemente falso ahora. Bubblegum v2 ya viene con `set_non_transferable_v2`, y un NFT comprimido puede ser soulbound al acuñarse. Así que la fila es un cNFT bajo la colección del club, hecho no transferible, con un acceso restringido como riel, y la celda de veredicto dice no aplica porque una insignia soulbound nunca iba a ir a un pool.

![Un gráfico de barras en escala logarítmica que compara el costo de un millón de tenencias entre activos Core, cuentas de token de SPL clásico y un solo árbol Bubblegum v2, que es más barato por más de dos órdenes de magnitud.](assets/v03-chart.webp)

### Fila tres, revisada

La participación de la cooperativa invierte la fila uno, y si sacaste bien las dos filas tienes la tesis, no solo la regla.

Aquí el consejo quiere autoridad de congelamiento sobre el saldo de un tenedor, y el brief dice explícitamente que nunca se negocia en público. Así que las extensiones de poder que descalificaban a la cafetería están simplemente disponibles. `DefaultAccountState` puesto en congelado le da a la cooperativa un paso de aprobación antes de que cualquier miembro nuevo pueda tener saldo, y un delegado permanente o una autoridad de congelamiento le da al consejo su vía de expulsión. La celda de veredicto dice no enrutable, y escribir eso es el punto entero de la fila: es un no-enrutable elegido, con precio puesto y aceptado, en vez de un no-enrutable que descubres en el lanzamiento. El riel es un acceso restringido, el mismo patrón que dejó que un cNFT de Founding Farmer abriera el alpha de Overgrowth, apuntado a la membresía en su lugar.

Aquí es donde la tesis de compatibilidad se gana el sueldo como herramienta de diseño y no como etiqueta de advertencia. Nunca dijo que las extensiones de poder sean malas. Dijo que cuestan plataformas, y un producto que no quiere plataformas no paga nada.

### A dónde apunta una celda en blanco

Ahora haz la cosa para la que existe el checkpoint. Mira lo que no pudiste llenar y mapéalo.

| La celda en la que te atascaste | Lo que quiere decir que te saltaste | Dónde vive |
|---|---|---|
| familia de primitivas, en cualquier fila | tienes las piezas pero no el selector | módulo uno, el framework de decisión y la matriz de conflictos |
| conjunto de extensiones, en una fila fungible | el catálogo nunca se condensó en economía, autoridad y display | módulo dos, las cuatro lecciones de extensiones |
| la celda de veredicto | la tesis de compatibilidad se quedó en dato en vez de volverse hábito | módulo cinco, el artefacto de la allowlist y el diseño del token enrutable |
| la fila de la insignia entera | la compresión sigue archivada como opción exótica | módulo siete, Bubblegum v2 y la derivación del costo |
| la celda del riel | construiste los activos y nunca conectaste valor entre ellos | módulo nueve, enrutamiento de comisiones, restricción de acceso y migraciones; m08-l3 si el hueco era el riel de airdrop |
| cómo demostrarías cualquiera de esas cosas | leer activos sigue siendo trabajo de otro | módulo siete, la lección de DAS y el lector R9 |

Seis filas, y una reacción madura a que dos de ellas se enciendan es ir a releer esas dos lecciones, no sentirte mal. La tabla es el entregable.

## Lab: puntúa tu propia tabla

La herramienta de abajo deliberadamente no es una clave de respuestas. Conoce las restricciones declaradas de los tres briefs, que puedes leer del texto de arriba, más las dos reglas enseñadas. No tiene ni idea de cuál es la primitiva correcta. Esa distinción es lo que la vuelve un espejo: puede decirte que una fila se contradice a sí misma, y no puede decirte qué escribir.

**1.** Arma una carpeta al lado de tu trabajo del capstone e instala el runner. Dos dependencias de desarrollo, nada que hable con una blockchain, porque una revisión de reglas debería correr en milisegundos con el wifi apagado. El único import de fuera de esta carpeta es tu propio `checkCombo` del lab de m01-l4, que es igual de offline: una función pura, sin RPC adentro.

```bash
mkdir -p checkpoint && cd checkpoint
npm init -y
npm pkg set type=module
npm install -D tsx@4.23.12 @types/node@24
```

Esas son las versiones que corrí el 2026-08-22, y las dos cambian lo bastante rápido como para que vuelvas a fijarlas en vez de copiarlas dentro de un año. `tsx` corre TypeScript sin paso de build; los tipos de Node evitan que `process.exit` se ponga rojo en tu editor.

**2.** Crea `score-table.ts`. Lee la mitad de arriba como una reformulación de los dos instrumentos, porque eso es lo que es: la matriz de conflictos importada directo de tu port de m01-l4, la allowlist del módulo cinco, las constantes de costo de los módulos seis, siete y ocho, y las restricciones de los briefs transcritas de la prosa a campos.

```typescript
// score-table.ts: scores YOUR filled decision table against the two taught rules.
// There is no answer key in this file. It knows each brief's stated constraints
// and the rules from m01-l4 (the conflict matrix) and m05-l2 (the thesis).
import { checkCombo } from "../check-combo"; // instrument A, the m01-l4 port, verbatim

// Deliberately identical to memo.ts's union from the capstone, character for
// character, so the rows you transcribe from your memo keep their spellings.
// tsx strips types without checking them, so a drifted literal here would not
// error, it would silently misclassify: the worst kind of wrong.
export type PrimitiveFamily = "spl-token" | "token-2022" | "core-asset" | "cnft";

// Read straight off the brief text. Constraints, never answers.
// Deliberately NOT transcribed: brief three's "board can freeze a share".
// Every mint carries a base freeze authority whether or not any extension is
// present, so a boolean here would wrongly flag rows that lean on it. That
// constraint gets checked in your DEFENSE sentence, by you, and naming the
// omission beats pretending the transcription is complete.
export interface BriefConstraints {
  id: string;
  fungible: boolean;
  mustStayTradeable: boolean;
  soulbound: boolean;
  units: number;
}

// One row of your table.
export interface DecisionRow {
  brief: string;
  primitive: PrimitiveFamily;
  set: string[];
  verdict: "routable" | "not-routable" | "n/a";
  rail: string;
}

// m05-l2: Raydium CP-Swap's Token-2022 allowlist is exactly five extensions.
export const RAYDIUM_CP_SWAP_ALLOWLIST: readonly string[] = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "InterestBearingConfig",
  "ScaledUiAmount",
];

// Anything that makes a holder unable to move the asset, spelled the way each
// primitive's own lesson spelled it: the Token-2022 extension, the Core plugin,
// and the Bubblegum v2 instruction (a cNFT row names the instruction, because
// DAS's structural tags cannot express soulbound-ness; m10-l1's gate said so).
const SOULBINDING: readonly string[] = [
  "NonTransferable",
  "PermanentFreezeDelegate",
  "set_non_transferable_v2",
];

export function isRoutable(row: DecisionRow): boolean {
  if (row.primitive === "spl-token") return true;
  if (row.primitive !== "token-2022") return false;
  // Allowlisting is not additive: EVERY extension has to be on the list.
  return row.set.every((e) => RAYDIUM_CP_SWAP_ALLOWLIST.includes(e));
}

const SOL_PER_CORE_ASSET = 0.003; // Metaplex's published ~0.003, read 2026-09-07
// Module 7's depth-20 / buffer-256 / canopy-14 tree: 1,223,352 bytes.
// Scaling this linearly is a deliberate simplification. Tree rent is set by
// depth, buffer and canopy, not by leaf count, so a small tree costs MORE per
// leaf and this understates it. Good enough to trip the 100-SOL alarm, not a budget.
//
// The two rent figures below are BYTES x a per-byte rate, and the rate is a
// network parameter SIMD-0437 is stepping down. 6,333 is mainnet's, read
// 2026-09-06; devnet was already at 5,080. Re-read it (`solana rent 0`, divide
// by 128) before this alarm decides anything real.
const LAMPORTS_PER_BYTE = 6_333;
const SOL_PER_MILLION_CNFTS = ((128 + 1_223_352) * LAMPORTS_PER_BYTE) / 1e9;
const LAMPORTS_PER_CLASSIC_ATA = 293 * LAMPORTS_PER_BYTE;

export function mintCostSol(primitive: PrimitiveFamily, units: number): number {
  switch (primitive) {
    case "core-asset":
      return units * SOL_PER_CORE_ASSET;
    case "cnft":
      return (units / 1_000_000) * SOL_PER_MILLION_CNFTS;
    case "spl-token":
    case "token-2022":
      // Deliberate simplification, same class as the cNFT linearization above:
      // this prices every holder at the bare 165-byte ATA rent. A token-2022
      // mint with TransferFeeConfig forces a TransferFeeAmount slot onto every
      // holder account (the m02-l1 forced-pair rule), so real per-holder rent
      // runs a few dozen bytes higher. The model floors the cost; a row that
      // only squeaks under a rent alarm at the floor deserves a second look.
      return (units * LAMPORTS_PER_CLASSIC_ATA) / 1e9;
  }
}

export function checkRow(row: DecisionRow, brief: BriefConstraints): string[] {
  const problems: string[] = [];
  const isNftFamily = row.primitive === "core-asset" || row.primitive === "cnft";

  // Instrument A fires first, exactly as the theory section ordered it: a set
  // the program refuses at initialize_mint never reaches a venue's opinion.
  if (row.primitive === "token-2022") {
    const combo = checkCombo(row.set);
    if (!combo.valid) {
      problems.push(`conflict matrix: ${combo.reason ?? "invalid combination"}`);
    }
  }

  if (brief.fungible && isNftFamily) {
    problems.push(`${row.primitive} is not a divisible balance; this brief needs a fungible mint`);
  }
  if (!brief.fungible && !isNftFamily) {
    problems.push(`a fungible mint cannot carry per-item metadata; this brief needs an asset`);
  }
  if (brief.mustStayTradeable && !isRoutable(row)) {
    const offenders = row.set.filter((e) => !RAYDIUM_CP_SWAP_ALLOWLIST.includes(e));
    problems.push(
      `must stay tradeable, but pool creation reverts: ${offenders.join(", ") || row.primitive}`,
    );
  }
  if (brief.mustStayTradeable && row.verdict !== "routable") {
    problems.push(`verdict says "${row.verdict}" while the brief demands a routable mint`);
  }
  if (brief.soulbound && !row.set.some((e) => SOULBINDING.includes(e))) {
    problems.push("brief says soulbound, nothing in the set stops a transfer");
  }
  if (row.rail.trim().length === 0) {
    problems.push("no economy rail named");
  }

  const cost = mintCostSol(row.primitive, brief.units);
  if (cost > 100) {
    problems.push(`mint cost ~${cost.toFixed(1)} SOL for ${brief.units.toLocaleString("en-US")} holders`);
  }
  return problems;
}

const BRIEFS: BriefConstraints[] = [
  { id: "cafe", fungible: true, mustStayTradeable: true, soulbound: false, units: 40_000 },
  { id: "badge", fungible: false, mustStayTradeable: false, soulbound: true, units: 1_000_000 },
  { id: "co-op", fungible: true, mustStayTradeable: false, soulbound: false, units: 900 },
];

// REPLACE these three rows with the ones you wrote by hand.
const MY_TABLE: DecisionRow[] = [
  {
    brief: "cafe",
    primitive: "token-2022",
    set: ["MetadataPointer", "TokenMetadata", "TransferHook"],
    verdict: "routable",
    rail: "fee route to the shop treasury",
  },
  {
    brief: "badge",
    primitive: "core-asset",
    set: ["PermanentFreezeDelegate"],
    verdict: "n/a",
    rail: "gate the members channel",
  },
  {
    brief: "co-op",
    primitive: "token-2022",
    set: ["DefaultAccountState", "PermanentDelegate", "MetadataPointer"],
    verdict: "not-routable",
    rail: "",
  },
];

function main(): void {
  let failed = 0;
  for (const row of MY_TABLE) {
    const brief = BRIEFS.find((b) => b.id === row.brief);
    if (!brief) throw new Error(`no brief named "${row.brief}"`);
    const problems = checkRow(row, brief);
    console.log(`${row.brief.padEnd(6)} ${problems.length === 0 ? "PASS" : "FAIL"}  ${row.primitive}`);
    for (const p of problems) console.log(`         - ${p}`);
    if (problems.length > 0) failed += 1;
  }
  console.log(`\n${MY_TABLE.length - failed}/${MY_TABLE.length} rows survive the rules.`);
  process.exit(failed === 0 ? 0 : 1);
}

main();
```

**3.** Córrelo una vez antes de tocar `MY_TABLE`. Lo que hay adentro es una tabla equivocada plausible, del tipo que yo genuinamente he escrito a la 1am, y cada fila falla por una razón distinta.

```bash
npx tsx score-table.ts
```

Deberías ver tres líneas FAIL y `0/3 rows survive the rules` en exit 1. La fila de la cafetería falla por el hook, la fila de la insignia falla por 3,000 SOL, y la fila de la cooperativa falla porque dejé la celda del riel vacía. Lee esos tres mensajes con cuidado, porque son las tres maneras en que una tabla de decisión sale mal en la práctica: una funcionalidad que le gana a una restricción, una primitiva cuyo costo solo aparece cuando multiplicas, y un plan sin valor moviéndose por dentro.

**4.** Ahora pega tus propias tres filas encima de `MY_TABLE`, exactamente como las escribiste a mano, incluidas las partes de las que no estás seguro. No las arregles en el camino. El punto es puntuar lo que produjo tu memoria, no lo que produjo tu memoria más quince segundos de dudarlo.

**5.** Córrelo otra vez y lee cada línea de problema como un puntero y no como un veredicto. Una fila que falla en `must stay tradeable` te manda al módulo cinco. Una fila que falla por costo te manda a la lección de compresión. Una fila sin riel te manda al módulo nueve. Las filas que pasan no son demostración de que tengas razón, son demostración de que no te estás contradiciendo, que es una vara más baja y más honesta de lo que suena.

![Un diagrama de flujo con las ocho revisiones de score-table.ts, desde la matriz de conflictos pasando por fungibilidad y enrutabilidad hasta el costo derivado, que alimenta una lista de problemas compartida que decide PASS o FAIL.](assets/v04-flowchart.webp)

## Challenge

Tres cosas, todas de escritura, ninguna de ellas código.

Primero, defiende cada fila que sobrevivió en una oración que nombre el instrumento. No "cNFT porque es barato" sino "Bubblegum v2 con `set_non_transferable_v2`, porque un millón de activos es un problema de compresión y soulbound ya no es una razón para dejar la compresión." Una defensa que no nombra la regla que aplicó es solo una preferencia.

Segundo, escribe el módulo al que vas a volver, y la cosa específica que vas a re-derivar cuando llegues. No "releer el módulo cinco" sino "volver a correr la revisión aditiva y ver cómo cinco extensiones de la allowlist más un delegado vuelven rechazadas." Las intenciones vagas de repasar son la forma en que la gente termina un curso sintiéndose bien y sigue exactamente igual de capaz que antes.

Tercero, toma una de tus tres filas y escribe la oración que la revertiría. La mía, para la insignia: si algún marketplace llega a soportar mover un cNFT soulbound por medio de una autoridad delegada, la garantía de "no vendible" se debilita y la fila vuelve a Core con un congelamiento permanente. Toda buena decisión tiene una condición de ruptura nombrada; sin una, lo que tienes es solo una preferencia.

## El cierre

Esto es todo lo que tienes en disco, en orden, porque verlo como una sola lista es el punto.

Un inspector que decodifica cualquier mint vivo hasta sus extensiones TLV. Un validador de matriz de conflictos portado desde la fuente. SPROUT mismo, un mint Token-2022 con una comisión de transferencia, un harvest de comisiones retenidas que funciona, y metadatos nativos, construido desde instrucciones crudas. Un programa de transfer hook en Rust con su `ExtraAccountMetaList`, probado en LiteSVM. Una variante confidencial, parada a un costado como la rama de emisor especializada que es. Un informe de enrutabilidad que corrió SPROUT y su gemelo con hook contra lógica de allowlist real. La colección Almanac en Metaplex Core, con regalías, una impresión de Edition, y una insignia Founding Farmer no transferible. Harvest crates como NFT comprimidos de Bubblegum v2, con precio puesto antes de acuñarse. Un script lector que resuelve las tres formas a través de DAS y marca qué extensiones están vivas frente a configuradas y dormidas. Una config de lanzamiento cuyo umbral de graduación derivaste de constantes publicadas en vez de citarlo. Un airdrop de compresión con una ruta de reclamo con vesting. Y la economía que los une: el harvest de comisiones hacia una tesorería, un buyback comprado y quemado con la caída de supply demostrada hasta la unidad base, un cNFT restringiendo el acceso al alpha, y puntos de compost migrando hacia SPROUT. Después tu propio producto, elegido y entregado y demostrado.

Cuatro ideas sostienen todo eso, y son lo que todavía deberías tener dentro de cinco años, cuando cada número de versión de este curso se haya podrido.

**Interfaz versus implementación.** El hecho más 2026 que hay en este curso es que el programa de token fue reemplazado bajo los pies de todos y el cliente de nadie cambió un byte. SIMD-0266 se mergeó el 2026-03-13, el p-token de Anza basado en Pinocchio tomó el control de la implementación de SPL clásico en la misma dirección, y un Transfer pasó de 4,645 unidades de cómputo a 76. TransferChecked pasó de 6,200 a 105. No me creas a mí sobre el estado actual, toma treinta segundos:

```bash
curl -s -X POST https://api.mainnet-beta.solana.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP",{"encoding":"base64"}]}' \
| node -e "
const chunks=[];process.stdin.on('data',c=>chunks.push(c)).on('end',()=>{
  const v=JSON.parse(Buffer.concat(chunks).toString()).result.value;
  if(!v){console.log('gate account not found: feature never activated');process.exit(0);}
  const b=Buffer.from(v.data[0],'base64');
  console.log(b[0]===1?'ACTIVE at slot '+b.readBigUInt64LE(1):'account exists, activation slot not set (queued)');
});"
```

Cuando corrí eso el 2026-08-22 imprimió `ACTIVE at slot 419472000`, el primer slot del epoch 971. El gate está vivo, el motor es nuevo, la interfaz nunca se movió. Esa es la forma de todo buen límite de abstracción que llegues a diseñar.

**La matriz de conflictos.** Las combinaciones que existen y las combinaciones que inicializan son conjuntos distintos, y la diferencia está escrita en fuente que puedes leer.

**La tesis de compatibilidad.** El poder cuesta plataformas. Es un precio, no una prohibición, y ahora sabes cómo pagarlo a propósito.

**DAS como el unificador.** Una sola interfaz de lectura sobre un mint fungible con extensiones, un activo Core y una hoja comprimida. Cuando alguien te da una dirección dentro de dos años y te pregunta qué es, tienes un script para eso.

![Un diagrama radial centrado en una dirección desconocida, con cuatro radios para las cuatro ideas duraderas del curso, cada uno etiquetado con su pregunta y su artefacto que lo demuestra.](assets/v05-diagram.webp)

### Lo que este curso no sabe

El límite honesto, porque ese ha sido el trato desde el principio. Este curso te dejó fluido en la capa de tokens y de activos y en su realidad de compatibilidad. Se detuvo en el borde de la graduación a propósito. Sin diseño de AMM, sin profundidad de LP, sin matemática de pools más allá de derivar un umbral de constantes publicadas. Rechazó el encuadre legal y de emisor que convierte un conjunto de extensiones con forma de compliance en un instrumento regulado de verdad. Usó Anchor para exactamente un programa de hook de 40 líneas y no te enseñó nada del framework. Nunca tocó el aterrizaje de transacciones, las comisiones de prioridad, ni la infraestructura de indexación a escala, y se quedó por encima del runtime todo el camino.

Cada una de esas cosas es el curso de alguien, y puedo decirte de quién — con una advertencia que te mereces de entrada, porque un traspaso hacia una puerta que no abre es peor que ningún traspaso. De los tres de abajo, solo **Master Anchor V2** está publicado hoy. Los otros dos están planificados y todavía no se han entregado, así que trátalos como un mapa del territorio y de dónde están sus bordes, no como enlaces que puedas hacer clic esta tarde. El catálogo es la fuente de verdad de lo que existe en realidad.

El curso planificado **DeFi and RWA Engineering** es el que toma las primitivas que acabas de aprender y las convierte en un negocio de emisión: emisión específica de RWA, los rieles de compliance a su alrededor, cómo estructuran de verdad sus programas los emisores vivos, y la profundidad de LP que este curso siguió traspasando por nombre. Todo lo que asume en la capa de tokens es lo que acabas de construir, así que entras por la puerta principal en vez de trepar por una ventana. Si tu fila tres, la participación de la cooperativa, se sintió como que quería un abogado en la sala, ese es el curso donde aparece el abogado.

El curso planificado **Client-Side Mastery** es dueño de todo lo que pasa entre tu script y la blockchain. El aterrizaje de transacciones y las comisiones de prioridad, la capa de indexación debajo de una llamada a DAS, Geyser y gRPC cuando un índice rentado no alcanza. Cada vez que este curso dijo "tu script lector asume un RPC que soporta DAS" y siguió de largo, esa era la costura. Ese curso está del otro lado de ella.

El curso **Master Anchor V2** es el framework mismo. Macros, constraints, mecánica de CPI, pruebas, migración. Escribiste un programa de hook aquí y yo te dije qué teclear, a propósito, porque un transfer hook es un concepto de token y Anchor es un concepto de framework y mezclarlos habría empeorado los dos. Si ese programa fue para ti las cuarenta líneas más interesantes del curso, esa es tu próxima puerta.

### Una última cosa, y un favor

La educación oficial de Solana se congeló a mitad de trama. El repositorio `solana-foundation/developer-content` pasó a solo lectura el 2025-01-24, y cada curso oficial que hay detrás de esos enlaces es anterior a `ScaledUiAmount`, `Pausable`, `ConfidentialMintBurn`, Bubblegum v2, Genesis y p-token. Eso no es una queja sobre la gente que los escribió. Es el mejor argumento que existe para el hábito que este checkpoint venía entrenando: re-derivar, re-sondear, re-leer la fuente, porque el texto canónico propio del ecosistema pudo dejar de actualizarse y lo hizo, mientras la blockchain seguía avanzando.

![Una línea de tiempo desde 2024 hasta agosto de 2026 que marca el lanzamiento de PYUSD, el archivado del contenido oficial para desarrolladores, el cierre de SimpleHash, el merge de SIMD-0266 y la activación del gate de p-token.](assets/v06-timeline.webp)

Ahora el favor, y es uno de verdad. Cada número de este curso tiene fecha y la mayoría se va a desfasar: las unidades de cómputo, el contenido de la allowlist, los costos por activo, las versiones de las herramientas, las constantes de graduación. Si vuelves a correr un sondeo de cualquier lección y tu salida no coincide con la mía, publica la lección, el comando exacto y lo que te dio en el canal de feedback del curso. Eso no es un reporte de bug por cortesía. Es el mecanismo de mantenimiento real del curso, y un aprendiz que pesca un número desfasado es el hábito funcionando en voz alta delante de todos los demás.

No hay lección siguiente. Esa es la parte rara de terminar algo. Tú eres quien decide qué pasa con eso ahora y, con franqueza, el siguiente paso útil más pequeño no es otro curso en absoluto: toma un activo de tu capstone, dale su dirección a un amigo, y mira si tu script lector se lo explica. Esa es la habilidad, y no es tan difícil mantenerla afilada una vez que la tienes.

Puedes leer los bytes de cualquier mint vivo, elegir la primitiva correcta y demostrar que resuelve. Cualquiera sea la puerta que elijas después, RWA y DeFi, client-side e indexación, o el framework de Anchor mismo, la capa de tokens ya es tuya para llevarla a través de ella. Ve a re-derivar algo que nadie haya revisado últimamente.
