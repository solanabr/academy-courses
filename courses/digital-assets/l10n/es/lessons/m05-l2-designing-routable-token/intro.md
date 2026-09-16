# Diseñando un token enrutable y en cumplimiento

## Resumen

La lección pasada leíste la regla en el propio programa de Raydium que acepta un SPROUT de comisión+metadatos y rechaza su variante con transfer hook, construiste un predictor que la reproduce, y pusiste los dos mints frente a ella en un fork de mainnet. Esta lección convierte esa lectura en una decisión y en un entregable. Vas a enunciar la tesis de compatibilidad en una oración, defenderla desde el lado del pool en la mesa, elegir el conjunto final de extensiones de plataforma de lanzamiento del SPROUT contra la allowlist, y producir el informe de enrutabilidad: cada extensión incluida etiquetada como whitelisted o refused con su justificación, las plataformas en las que el SPROUT es verificado-enrutable, y una lista explícita de las afirmaciones que tienes que verificar en tu propio objetivo en vez de confiar en la matriz congelada de nadie, este curso incluido. El repliegue está casi completo: la tesis y el armado del conjunto se trabajan con apoyo liviano, la sección de honestidad del informe es tuya, en solitario, y no hay coding challenge porque el informe mismo es la revisión. Esta es una lección de criterio con la forma de una lección de build.

Pasaste cuatro módulos dándole capacidades al SPROUT. Comisiones que financian una tesorería. Metadatos nativos. Una variante con hook que loguea cada harvest. Una rama confidencial con una auditor key. Cada una de ellas fue un buen build. Y la lección pasada un DEX miró la suma de todas y dijo no. Entregar ahora es una decisión, no una consulta: cada extensión que te quedas es una capacidad que quieres y una plataforma que podrías perder, y tú eres quien firma el canje.

Así que empieza donde debería empezar una decisión, con la lista completa de candidatas y un veredicto por línea. En tu carpeta de lab de m05-l1, al lado de `predict-routability.ts`, suelta esto y córrelo:

```typescript
// audit-candidates.ts: every extension the SPROUT design has worn or weighed
// (the delegate and frozen-default rows were m02 catalog considerations, never
// minted onto SPROUT), one verdict each.
import { isRoutable } from "./predict-routability";

const CANDIDATES = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "TransferHook",
  "PermanentDelegate",
  "DefaultAccountState",
  "ConfidentialTransferMint",
];

for (const ext of CANDIDATES) {
  const ok = isRoutable({ tokenProgram: "token2022", extensions: [ext] });
  console.log(ext.padEnd(26), ok ? "whitelisted" : "refused");
}
```

```bash
npx tsx audit-candidates.ts
```

Tres whitelisted, cuatro refused. Esa salida es la lección entera; las próximas tres mil palabras son sobre por qué la línea cae exactamente ahí, y cómo se ve un informe honesto construido encima de ella.

## Quién tiene derecho a correr código en cada transferencia

### La tesis, una oración

Aquí está: un DEX pone en la whitelist las extensiones que solo cambian la visualización o raspan una comisión declarada, y rechaza las extensiones que dejan al emisor correr código arbitrario o mover tokens de otras personas. Comisión, visualización y contabilidad de un lado: TransferFeeConfig, MetadataPointer, TokenMetadata, InterestBearingConfig, ScaledUiAmount, exactamente las cinco de la allowlist de CP-Swap de Raydium. Poder y control del otro: TransferHook, PermanentDelegate, DefaultAccountState congelado, ConfidentialTransfer, todas refused. Dilo con esas palabras y no con las más cortas y tentadoras, porque "compliance entra en la whitelist" es falso de un modo que le va a costar un lanzamiento a alguien: PermanentDelegate y DefaultAccountState son las dos primitivas favoritas del equipo de cumplimiento y las dos caen del lado refused. El capstone entrena exactamente esa frase de una línea. Verificaste la mecánica de esto la lección pasada leyendo el fuente. La pregunta de hoy es otra. ¿Por qué la línea está AHÍ, y no en otro lugar?

### Derivando la línea desde el asiento del pool

Siéntate en la silla del pool un minuto. Un pool es una pila de dos tokens más una invariante, y su único trabajo no negociable es que la pila se quede donde la matemática dice que debería. Ahora recorre los diseños ingenuos y mira cómo falla cada uno.

¿Podría el DEX revisar tokens caso por caso? Ese es un diseño real, y Orca lo entrega. Un Token Badge es una aprobación por mint que el equipo de Orca concede después de mirar de verdad tu mint, que es cómo un token que lleva algo como PermanentDelegate puede negociarse ahí siquiera: no pasando una lista publicada, sino pasando una persona. Ponle precio a ese diseño con honestidad, porque no es estrictamente peor que una lista. Un revisor ve cosas que un match statement nunca va a ver: quién es el emisor, si la clave del delegate es una multisig o una sola laptop, si la autoridad de la comisión ya fue revocada. Lo que un revisor no puede hacer es escalar, responder en un tiempo acotado, o decirte el veredicto antes de que te hayas comprometido con un diseño y lo hayas acuñado. Y una fábrica de pools sin permiso no puede usar uno para nada. La creación de pool ahí es una transacción que cualquiera puede enviar en cualquier slot sin ningún humano en el camino, así que para CP-Swap la regla de aceptación tiene que ser código. El código no puede leer un whitepaper ni un dictamen legal. Solo puede leer el mint.

![Tabla de siete ejes que compara la revisión humana del Token Badge de Orca contra la allowlist compilada de Raydium, con la fila de la fábrica sin permiso marcada como la razón por la que la regla de CP-Swap tiene que ser código.](assets/v01-table.webp)

Bien, ¿entonces el pool podría aceptar todo y manejar las consecuencias? Recorre la lista refused y las consecuencias no son manejables. Un TransferHook significa que un programa que el DEX nunca auditó corre dentro de cada swap, con el compute que se le antoje; los docs de Raydium lo rechazan exactamente con esas palabras, un programa personalizado invocado en cada transferencia con consumo arbitrario de CU. Un PermanentDelegate significa que alguna clave por ahí puede mover tokens de cualquier cuenta, y el vault del pool es una cuenta; Raydium otra vez, literalmente: un tenedor del delegate puede barrer cualquier cuenta de token, incluido el vault del pool. DefaultAccountState congelado significa que el emisor decide si las propias cuentas del pool pueden transaccionar siquiera. ConfidentialTransfer significa que los montos están encriptados, y no puedes correr una invariante sobre ciphertext. Cada rechazo protege una suposición estructural distinta, pero todos son del mismo género: la extensión le da a alguien de afuera del pool autoridad sobre lo que pasa adentro.

### Por qué la respuesta tiene que ser binaria

Vale cerrar una escapatoria más, porque es la siguiente a la que va a recurrir un buen ingeniero. ¿Podría la regla ser graduada en vez de binaria? Deja entrar un mint riesgoso, pero cóbrale una comisión más alta, limita el tamaño de su pool, ponlo en cuarentena detrás de una advertencia. Graduar necesita que el pool le ponga un número al riesgo, y las extensiones refused no tienen números. ¿Cuál es el recargo correcto para un transfer hook cuya autoridad puede apuntarlo a un código completamente distinto mañana? No hay respuesta, porque la exposición no es una distribución sobre la que puedas integrar, es "lo que sea que ese programa decida hacer después". Una comisión declarada de 500 basis points es un costo. Un hook actualizable es una responsabilidad abierta, y las responsabilidades abiertas no tienen precios, tienen acepta-o-rechaza. Por eso la regla cae binaria, y por eso tiene que ser una función pura del conjunto de extensiones, nada más.

Ahora mira el lado aceptado con los mismos ojos. Una comisión de transferencia es poder del emisor también, técnicamente, pero está declarada, tiene techo y se lee en el mint, así que el pool puede ponerle precio, y la interfaz te entrega `calculate_fee`, el helper de matemática de comisión en el estado de TransferFeeConfig cuya fórmula espejaste en TypeScript como `transferFee` allá en m02-l1, para hacer exactamente eso. Los metadatos cambian lo que ven los humanos, nunca lo que hace el token. InterestBearingConfig y ScaledUiAmount son aritmética de visualización pura; los montos crudos que el pool contabiliza nunca se mueven. El patrón no es "las extensiones inofensivas pasan". El patrón es: cualquier cosa a la que el pool le pueda poner precio por completo desde datos on-chain pasa, cualquier cosa que le reserve discrecionalidad al emisor falla. Ese es un juicio de valor sobre quién tiene derecho a correr código en cada transferencia, codificado como un match statement de Rust, y genuinamente me parece más honesto que un formulario de listado. Al código no se le puede hacer lobby.

![Diagrama de dos columnas que separa las cinco extensiones de visualización y comisión de la whitelist de las cuatro extensiones de poder refused, divididas por si el pool puede computar su invariante sin confiar en el emisor.](assets/v02-diagram.webp)

Una vuelta más a la manivela, porque la tesis tiene un corolario filoso que conociste la lección pasada. Pídele a tu propio predictor que lo enuncie, en la misma carpeta que antes:

```typescript
// additive-check.ts: does a whitelisted extension rescue a refused one?
import { isRoutable } from "./predict-routability";

const CASES: [string, string[]][] = [
  ["fee alone", ["TransferFeeConfig"]],
  ["hook alone", ["TransferHook"]],
  ["fee + metadata + hook", ["TransferFeeConfig", "TokenMetadata", "TransferHook"]],
  ["all five allowlisted + delegate", [
    "TransferFeeConfig",
    "MetadataPointer",
    "TokenMetadata",
    "InterestBearingConfig",
    "ScaledUiAmount",
    "PermanentDelegate",
  ]],
];

for (const [label, extensions] of CASES) {
  const verdict = isRoutable({ tokenProgram: "token2022", extensions }) ? "ROUTABLE" : "REJECTED";
  console.log(label.padEnd(32), verdict);
}
```

```bash
npx tsx additive-check.ts
```

Solo el primer caso vuelve enrutable. Cinco extensiones de la allowlist más una refused sigue siendo rechazado, y ese es el corolario: estar en la allowlist no es aditivo. Una extensión fuera de la lista contamina el mint entero. No existe la defensa "pero también tiene TransferFeeConfig", porque la exposición del pool a un permanent delegate no se encoge cuando un fee config se sienta al lado. Tu predictor codifica esto como `every`, no `some`, y esa única palabra es la diferencia entre el modelo del folclore y el real.

### Lo que te cuesta el conjunto aburrido

Déjame decir el trade-off en voz alta, porque este curso prometió que siempre lo haría: el conjunto enrutable es el conjunto aburrido. Diseñar para negociabilidad máxima significa renunciar a todo lo interesante que construiste o pesaste desde el catálogo de autoridad del módulo dos en adelante. Nada de lógica en transferencia en el mint que la gente negocia. Nada de permanent delegate, nada de onboarding congelado por defecto, los dos poderes de m02 que el SPROUT consideró y nunca se puso. Nada de montos ocultos. Si tu producto genuinamente necesita una extensión de poder, esa necesidad es real y esta lección no te está diciendo que la abandones. Te está diciendo que le pongas precio: un hook significa territorio de Meteora DBC en vez de CP-Swap, o una revisión de Token Badge en Orca que puede salirte bien o no, o una arquitectura de dos mints donde la variante con poder nunca toca un pool. Elegir una superficie de plataformas más chica es un diseño legítimo. Descubrir una superficie de plataformas más chica en el lanzamiento es un incidente.

![Diagrama de flujo que recorre cada extensión candidata por necesidad, pertenencia a la allowlist, y ubicación emisor-versus-tenedor, terminando en descartar, quedarse, mover a una variante de emisor, o aceptar a sabiendas una superficie de plataformas más chica.](assets/v03-flowchart.webp)

### Armando el conjunto del SPROUT

Aplica el procedimiento a la auditoría que corriste arriba. TransferFeeConfig: la comisión financia la tesorería, ese es el motor económico del SPROUT desde m02-l1, y está en la whitelist. Se queda. MetadataPointer más TokenMetadata: los metadatos nativos que conectaste en m02-l4, en la whitelist, solo visualización. Se quedan los dos. Ese es el mint negociable entero. Tres extensiones, todas aburridas, todas con precio.

El hook es el adiós difícil. Lo escribiste tú mismo en m03, funciona, y es exactamente el código-arbitrario-en-cada-transferencia que un pool no puede cargar. Sale del mint negociable. Si loguear el harvest todavía le importa al producto, el hook vive en una variante de emisor separada y sin pool, el mismo patrón que la rama confidencial que archivaste en m04: mints con poder para flujos del emisor, un mint aburrido para el mercado. Y aquí hay un detalle que hace que la división en dos mints sea menos molesta de lo que suena: la matriz de combinaciones que construiste en m01-l4 habría pelado contra una fusión de todos modos. Mete ConfidentialTransferMint dentro del mint de comisión y la regla 2 de `check-combo` exige ConfidentialTransferFeeConfig encima, lo que arrastra todo el aparato confidencial de comisiones. El sistema de extensiones mismo sigue empujando el poder y el comercio a lados opuestos. Peleé contra ese empujón un rato en mis propios diseños antes de aceptar que era estructural.

![Comparación de tres columnas del SPROUT de comisión-más-metadatos que está lanzando (enrutable, verificado en fork) contra la variante con transfer hook (rechazada, solo para el emisor) y la rama confidencial archivada (imposible de poner en pool por construcción).](assets/v04-comparison.webp)

### Las dos extensiones de la allowlist que el SPROUT todavía no se lleva

Aquí está la objeción que yo levantaría releyendo esto: el conjunto del SPROUT tiene tres extensiones, y la allowlist tiene cinco. InterestBearingConfig y ScaledUiAmount están sentadas ahí, pre-aprobadas, costando cero de enrutabilidad. ¿Por qué no llevarlas? Un token de farming con sabor a yield podría plausiblemente querer acumulación de interés, y un multiplicador de visualización te da una historia de rebase gratis.

Porque la allowlist es un piso, no una lista de compras. Pasarla significa que la plataforma no te va a rechazar por esa extensión. Nunca significa que la extensión sea gratis. Las dos hacen la misma cosa astuta: cambian el número que muestra cada interfaz sin cambiar el número que el programa de verdad mueve. Ese hueco es toda la feature y es también todo el costo. En el momento en que el SPROUT lleve ScaledUiAmount, tu explorador, tu exportación de CSV, tu planilla de contabilidad, tus respuestas de soporte, el gráfico de precio del DEX y el saldo crudo en una respuesta de `getTokenAccountBalance` ya no son obviamente el mismo número, y cada uno de esos lectores ahora tiene derecho a una explicación que tienes que escribir y mantener verdadera. InterestBearingConfig acumula ese monto de UI continuamente; ScaledUiAmount lo multiplica por un factor que una autoridad puede cambiar después. La matemática de tesorería del SPROUT no necesita ninguno de los dos.

Así que la regla de diseño se generaliza más allá de la enrutabilidad, y esta es la versión que vale guardar: una extensión tiene dos costos, y la allowlist le pone precio solo a uno. El costo uno es el riesgo de rechazo, que la plataforma publica. El costo dos es cada lector downstream al que ahora le debes una explicación, que nadie publica y tú pagas para siempre. Yo entregué una extensión porque estaba en la lista de aprobados de alguien, y después gasté más horas explicándola en un canal de soporte que las que gasté usándola. Dos extensiones que no necesitas son dos párrafos en un runbook que vas a estar escribiendo a las 2 de la mañana.

### PYUSD hace la misma matemática a escala de miles de millones de dólares

No tienes que aceptar el patrón de dos carriles de un token de curso. Léelo del emblemático cuya historia cerró la lección pasada, y esta vez cuenta: el mint de PYUSD lleva ocho extensiones TLV, mintCloseAuthority, permanentDelegate, transferFeeConfig, el par confidentialTransfer, transferHook, metadataPointer, tokenMetadata. Configurado pero dormido: releí el mint desde mainnet mientras redactaba esto el 2026-08-23 y el `programId` del hook es null y la comisión está en 0 basis points, máximo 0 — cada opción de poder comprada, cada una apagada. Cómo se negocia en CP-Swap siquiera un mint con permanent delegate ya lo sabes: el bypass de la whitelist, y lo puedes resolver en cinco segundos porque la dirección del mint de PYUSD es uno de los cuatro strings en `MINT_WHITELIST` en `token.rs` L18-23, ahí mismo en la salida de `sed` que ya imprimiste. Entrega las extensiones con forma de compliance, mantén las de poder inactivas, y aun así la enrutabilidad vino de una puerta especial, no de la regla general.

![Diagrama de las ocho extensiones TLV de PYUSD leídas en vivo el 2026-08-23, con la comisión de transferencia en cero basis points y el program ID del transfer hook null, ilustrando extensiones de poder configuradas-pero-dormidas.](assets/v05-diagram.webp)

### La enrutabilidad es por plataforma, y la mayor parte del mapa está sin luz

Todo lo anterior es la ley de una plataforma. Sostén ese límite con firmeza, porque en el momento en que el SPROUT se enrute en CP-Swap tu cerebro va a querer escribir la oración "el SPROUT es negociable", y esa oración no es algo que sepas. El mapa de plataformas en sí es material de la lección pasada, así que sostén solo sus titulares: AMM v4 y Stable AMM bajo la misma marca Raydium aceptan solo SPL clásico; la tabla de soporte a nivel de docs de Orca más su Token Badge revisado por humanos hacen de esa respuesta una decisión por mint que puedes solicitar pero nunca predecir por completo desde tu conjunto de extensiones; Meteora DBC corre la contra-tesis y soporta configuraciones de transfer hook; y la política de enrutamiento por extensión de Jupiter simplemente no se encuentra en sus docs, en ningún barrido que haya corrido este curso. Lo que agrega hoy es la consecuencia de diseño: la única afirmación honesta sobre cualquier plataforma es verifica al momento de escribir, contra tu mint, en su API en vivo.

Las billeteras son más oscuras todavía. Si Phantom muestra una advertencia de comisión, si Backpack renderiza el metadata pointer, si Solflare marca un hook: sin verificar, todo eso, en cada pasada de investigación detrás de este curso. Yo podría pegar una matriz de compatibilidad plausible aquí y le creerías, y precisamente por eso no lo voy a hacer. Una matriz congelada de afirmaciones no medidas es peor que ninguna matriz, porque falla en silencio en el único lugar donde dejaste de revisar.

![Comparación de cinco plataformas, desde la allowlist en código de CP-Swap de Raydium y la regla de SPL clásico de AMM v4 hasta la revisión de Token Badge de Orca, la política ausente de Jupiter, y el soporte de hook de Meteora, marcando las filas de verifícalo-tú-mismo.](assets/v06-comparison.webp)

¿Por qué tanta oscuridad en un ecosistema que está madurando? En parte porque el terreno de verdad se mueve, y en parte porque la gente que solía mapearlo dejó de hacerlo. El repositorio developer-content de la Solana Foundation, la fuente detrás de años de material oficial de curso, se archivó el 2025-01-24. Cada curso oficial se congeló antes de que existieran las reglas de plataforma contra las que estás diseñando. No hay matriz canónica porque a nadie le pagan por mantener una verdadera, y los terceros que publican una están congelando los mismos hechos en movimiento que tú. Eso no es razón para la desesperación; es la restricción de diseño alrededor de la cual se construye tu informe. El entregable durable es una matriz más el método fechado para re-derivar cada celda.

![Línea de tiempo desde el lanzamiento de PYUSD en mayo de 2024, pasando por el archivado de la educación oficial de Solana en enero de 2025, hasta las lecturas fechadas de 2026 de esta lección, terminando en una flecha de reverificar-en-el-lanzamiento.](assets/v07-timeline.webp)

### Qué le debe al lector un ítem de verificación

"Verifica en tu objetivo" puede ser una disciplina profesional o un encogerse de hombros que movió el trabajo sin mover ningún conocimiento, y la diferencia es mecánica. Un ítem de verificación se gana su lugar cuando lleva tres partes. Un **objetivo**, nombrado con suficiente especificidad como para abrirlo: no "billeteras" sino Phantom en la versión que probaste. Una **afirmación**, enunciada con suficiente precisión como para que el resultado de un lector pueda contradecirla: no "la visualización puede variar" sino "si la deducción de la comisión de transferencia se muestra antes de firmar". Y un **procedimiento**, que es lo que corre un lector más lo que significa cada respuesta posible.

Quita cualquiera de las partes y mira al ítem podrirse de un modo predecible. Objetivo faltante, y escribiste un descargo, que te protege a ti y no ayuda a nadie. Afirmación faltante, y un lector que corre tu procedimiento no puede decir si lo que vio coincide contigo o te refuta, así que su resultado nunca viaja de vuelta. Procedimiento faltante, y pasaste una tarea en vez de un método, que es la versión cortés de adivinar.

Hay una regla que hace juego para las afirmaciones que sí verificaste, y es la más corta: féchalas. Una afirmación verificada necesita una fecha para poder podrirse a la vista; una afirmación sin verificar necesita un método para que alguien la pueda resolver. Cada afirmación fechada de esta lección sigue la primera regla, incluidas las que leí de mainnet esta mañana.

![Tabla que contrasta versiones decorativas y usables de un ítem de verificación en objetivo, afirmación y procedimiento, con la regla de que las afirmaciones verificadas llevan una fecha y las no verificadas un método.](assets/v08-table.webp)

Esa forma de tres partes no es una convención de escritura, es una estructura de datos, y en el lab que estás a punto de construir se convierte en una interfaz de TypeScript con exactamente tres campos. Que es lo lindo de codificar la honestidad en un programa: un encogerse de hombros no pasa la revisión de tipos.

## Lab: produce el informe de enrutabilidad

El artefacto es `routability-report.ts`, la forma terminada de R6. Para fijar el nombre una vez, ya que ahora apareció dos veces: R6 ES el informe de enrutabilidad; el predictor que construiste la lección pasada era su borrador, y viaja dentro de este archivo como el etiquetador. Consume tu predictor y tu revisor de combinaciones, etiqueta el conjunto final del SPROUT, enuncia las plataformas verificadas, y se niega a enunciar las no verificadas. El criterio es la forma usual del curso: `npx tsx routability-report.ts` tiene que emitir el conjunto final de extensiones con cada extensión etiquetada como whitelisted o refused según la allowlist, las plataformas verificadas-enrutables, y una lista no vacía de verifica-en-tu-objetivo que cubra Orca, Jupiter y la visualización en billetera.

1. Trabaja en la carpeta de lab de m05-l1, donde ya vive `predict-routability.ts`, y trae el revisor de combinaciones desde su propio lab para que los dos imports resuelvan desde un solo directorio: `cp ../m01-l4/check-combo.ts .`. Nada nuevo que instalar si hiciste ese lab; si estás empezando limpio, los pines del runner son las mismas dos herramientas de dev, revisadas de nuevo hoy (2026-08-23). `tsx@4.23.12` era el npm latest en esa lectura; `typescript@5.9.3` es una retención deliberada, ya que el npm latest se movió a la línea 7 y este curso fija la versión contra la que se verificaron sus labs. Los dos números se podren, así que corre `npm view tsx version` tú mismo el día que armes el scaffold:

```bash
npm install -D tsx@4.23.12 typescript@5.9.3
```

2. Antes de etiquetar nada, trae la verdad de campo de lo que tu mint negociable de verdad lleva. Tienes la CLI de `spl-token` del sondeo docs-versus-código de m01-l4 (el bundle de Agave de m01-l3 puede haberla incluido; si el tuyo no, `cargo install spl-token-cli` cierra el hueco). Apúntala a donde de verdad acuñaste el SPROUT, que en el camino por defecto es tu surfnet local; los forks son efímeros, así que si el tuyo se reinició desde que acuñaste, vuelve a acuñar primero con los dos comandos de la apertura de m05-l1 (y cambia a `--url devnet` si tomaste el fallback de devnet de m02-l4):

```bash
spl-token display <YOUR_SPROUT_MINT> --url http://127.0.0.1:8899
```

   Pasa de largo el supply y los decimals hasta el bloque de extensiones. Para el mint que quieres, lista un transfer fee config con tus basis points y máximo, un metadata pointer cuya dirección es el mint mismo, y el token metadata que guarda el nombre, el símbolo y la URI del SPROUT. Tres entradas, ninguna cuarta. Si una línea `Transfer Hook` sigue sentada ahí porque acuñaste la variante de m03 y nunca volviste a acuñar, detente aquí: ese mint es la variante con hook, no el mint de lanzamiento, y ninguna cantidad de etiquetado cuidadoso en el informe va a cambiar lo que lee la revisión del pool. Si la salida no coincide con el conjunto que tienes en la cabeza, gana el mint, y tu informe describe el mint en vez de tus intenciones. Esta revisión de treinta segundos es la diferencia entre un informe y un deseo, y es el único paso de este lab que me negaría a saltear.

3. Ahora el truco central del informe: deriva cada etiqueta del predictor en vez de copiar la allowlist a un segundo archivo. Dos copias de una lista se desvían; una función sondeada dos veces no puede. Empieza `routability-report.ts` con los imports y el etiquetador. El etiquetador es la única movida astuta del informe, dos líneas, así que léelo en vez de ojearlo:

```typescript
// routability-report.ts: SPROUT's launch-venue routability report (R6, final).
import { isRoutable, type MintProfile } from "./predict-routability";
import { checkCombo } from "./check-combo";

type Tag = "whitelisted" | "refused";

interface ExtensionRow {
  extension: string;
  tag: Tag;
  rationale: string;
}

interface VerifyItem {
  target: string;
  claim: string;
  howToVerify: string;
}

// Tag an extension by asking the PREDICTOR: a hypothetical Token-2022 mint
// carrying only this extension either passes CP-Swap's check or it does not.
function tagOf(extension: string): Tag {
  const probe: MintProfile = { tokenProgram: "token2022", extensions: [extension] };
  return isRoutable(probe) ? "whitelisted" : "refused";
}
```

4. Dale a cada fila una justificación, porque una etiqueta sin razón es folclore con mejor formato. Las entradas de la allowlist dicen qué puede seguir haciendo el pool; las entradas refused llevan la clase de razón de la plataforma, las que puedes defender desde el fuente. Después declara el conjunto de lanzamiento y el conjunto descartado, y cada descarte registra a dónde se fue la capacidad en vez de fingir que nunca existió:

```typescript
const RATIONALE: Record<string, string> = {
  TransferFeeConfig: "declared, capped fee the pool can read and price in",
  MetadataPointer: "display only; changes nothing about how a transfer executes",
  TokenMetadata: "display only; name/symbol/URI live on the mint itself",
  InterestBearingConfig: "UI-level accrual; raw token amounts are untouched",
  ScaledUiAmount: "UI multiplier; raw token amounts are untouched",
  TransferHook: "issuer code runs on every transfer, arbitrary CU; refused",
  PermanentDelegate: "issuer can move tokens from any account, pool vault included; refused",
  DefaultAccountState: "issuer decides whether new accounts can transact at all; refused",
  ConfidentialTransferMint: "encrypted amounts cannot be priced by an AMM; refused",
};

function buildRows(extensions: string[]): ExtensionRow[] {
  return extensions.map((extension) => ({
    extension,
    tag: tagOf(extension),
    rationale: RATIONALE[extension] ?? "no rationale recorded; justify before shipping",
  }));
}

// The tradeable mint: fees fund the treasury, metadata is native. All allowlisted.
const LAUNCH_SET = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"];

// Considered and dropped, with the decision recorded.
const DROPPED: ExtensionRow[] = [
  {
    extension: "TransferHook",
    tag: "refused",
    rationale:
      "dropped from the tradeable mint: costs CP-Swap outright; hook lives on the non-pooled issuer variant only",
  },
  {
    extension: "ConfidentialTransferMint",
    tag: "refused",
    rationale:
      "stays on the shelved issuer branch from m04; the confidential path cannot be pooled, and venues that admit such mints at all (Orca's table) do so for public transfers only",
  },
  {
    extension: "PermanentDelegate",
    tag: "refused",
    rationale:
      "an m02 catalog consideration, never adopted: refused by the allowlist, and SPROUT's product has no clawback requirement to justify the venue cost",
  },
  {
    extension: "DefaultAccountState",
    tag: "refused",
    rationale:
      "an m02 catalog consideration, never adopted: the classic just-in-case trap the decision flowchart warns about; gated onboarding is not a SPROUT requirement",
  },
];
```

5. Las dos secciones de plataforma son donde la honestidad se vuelve estructural. `VERIFIED_ROUTABLE` guarda solo plataformas donde la enrutabilidad se demostró, que para ti es exactamente una entrada, el pool-create en el fork de mainnet de la lección pasada. `VERIFY_YOURSELF` guarda cada afirmación que no tienes permitido afirmar, cada una con el procedimiento concreto que un lector corre en su propio objetivo. Este apoyo lleva los tres objetivos que exige el criterio; la redacción de cada `howToVerify` es tuya para afilar en el challenge:

```typescript
// Venues where routability was DEMONSTRATED, not inferred. If you took m05-l1's degrade
// path and never landed the pool-create, this entry is a claim you have not earned: say
// "predictor verdict ROUTABLE, matched against token.rs @ 244e124; pool-create unrun" instead.
const VERIFIED_ROUTABLE = [
  "Raydium CP-Swap: predictor verdict ROUTABLE, confirmed by a mainnet-fork pool-create (m05-l1 lab)",
];

// Claims we do NOT assert. Each names the target, the unverified claim, and
// how the reader verifies it in the venue they actually ship to.
const VERIFY_YOURSELF: VerifyItem[] = [
  {
    target: "Orca",
    claim: "whether SPROUT's extension set clears Orca's Token Badge review",
    howToVerify:
      "check the Token Badge requirements in Orca's current docs and submit the mint for review; the badge is a per-mint decision, not a published allowlist",
  },
  {
    target: "Jupiter",
    claim: "whether Jupiter routes this Token-2022 extension set",
    howToVerify:
      "no per-extension routing policy was found in Jupiter's docs at the time of writing. Jupiter's live API sees mainnet only and your SPROUT lives on a local fork, so the runnable version is two-step: today, quote a mainnet mint with the same extension shape (fee + metadata pointer + metadata) to learn the policy; at launch, quote your own mint the moment it exists on mainnet. A returned route is a yes; token-not-found or no-route is the no",
  },
  {
    target: "Wallets (Phantom, Backpack, Solflare)",
    claim: "how each wallet displays the transfer fee and whether it warns on any extension",
    howToVerify:
      "load the mint in the wallet you target and observe; per-extension display behavior is not standardized and was not measured by this course",
  },
];
```

6. Emite y autoevalúate. El informe imprime sus cuatro secciones, y después vuelve sus propias reglas sobre sí mismo: una extensión refused en el conjunto de lanzamiento, una violación de la matriz de combinaciones, un rechazo del predictor, o una lista de verifica-tú-mismo vacía, cada una sale con código distinto de cero. Ese último criterio importa más; un informe sin nada que quede por verificar no es minucioso, está congelado:

```typescript
function main(): void {
  const rows = buildRows(LAUNCH_SET);
  const combo = checkCombo(LAUNCH_SET);
  const verdict = isRoutable({ tokenProgram: "token2022", extensions: LAUNCH_SET })
    ? "ROUTABLE"
    : "REJECTED";

  console.log("# SPROUT routability report\n");
  console.log("## Final launch-venue extension set\n");
  for (const r of rows) {
    console.log(`- ${r.extension} [${r.tag}]: ${r.rationale}`);
  }
  console.log(`\nCombo matrix: ${combo.valid ? "valid" : `INVALID (${combo.reason})`}`);
  console.log(`CP-Swap predictor verdict: ${verdict}\n`);

  console.log("## Considered and dropped\n");
  for (const r of DROPPED) {
    console.log(`- ${r.extension} [${r.tag}]: ${r.rationale}`);
  }

  console.log("\n## Verified routable\n");
  for (const v of VERIFIED_ROUTABLE) console.log(`- ${v}`);

  console.log("\n## Verify in your target (not asserted by this report)\n");
  for (const v of VERIFY_YOURSELF) {
    console.log(`- ${v.target}: ${v.claim}`);
    console.log(`  how: ${v.howToVerify}`);
  }

  const refusedInSet = rows.filter((r) => r.tag === "refused");
  if (refusedInSet.length > 0) {
    console.error(`\nGATE FAIL: refused extension(s) in the launch set: ${refusedInSet.map((r) => r.extension).join(", ")}`);
    process.exit(1);
  }
  if (!combo.valid) {
    console.error(`\nGATE FAIL: combo matrix violation: ${combo.reason}`);
    process.exit(1);
  }
  if (verdict !== "ROUTABLE") {
    console.error("\nGATE FAIL: predictor rejects the launch set");
    process.exit(1);
  }
  if (VERIFY_YOURSELF.length === 0) {
    console.error("\nGATE FAIL: the verify-yourself section is empty; that is a frozen-matrix report");
    process.exit(1);
  }
  console.log("\nAll gates pass: allowlist-clean set, valid combo, non-empty verify list.");
}

main();
```

   Córrelo:

```bash
npx tsx routability-report.ts
```

   Deberías ver las cuatro secciones en orden, `Combo matrix: valid`, `CP-Swap predictor verdict: ROUTABLE`, tres entradas de verifica-tú-mismo, y la línea final `All gates pass` con código de salida 0. Después prueba que los gates son reales: agrega `"DefaultAccountState"` al `LAUNCH_SET`, corre otra vez, y mira al mismo script negarse a entregar su propio informe: una fila etiquetada refused, gate fail, exit 1. Un checkpoint que no puede fallar nunca fue un checkpoint. Saca la extensión de vuelta.

![Diagrama de flujo del script de informe consumiendo el predictor y el revisor de combinaciones, emitiendo cuatro secciones, y después fallando su propio build por una extensión refused, una combinación inválida, un conjunto rechazado, o una lista de verifica-tú-mismo vacía.](assets/v09-flowchart.webp)

## Challenge

La mitad en solitario es la sección de honestidad, y la escribes sin apoyo. Toma las tres entradas de `VERIFY_YOURSELF` y convierte cada `howToVerify` de mi redacción de relleno en un procedimiento que le pasarías a un compañero: para Orca, qué enviarías de verdad y por dónde vuelve la respuesta del Token Badge; para Jupiter, la petición de cotización exacta que harías una vez que el SPROUT exista en mainnet, más el mint de mainnet con la misma forma que sondearías hoy como su reemplazo, y cómo se ve una respuesta enrutada versus no enrutada; para las billeteras, qué pantallas abrirías y qué registrarías. Después agrega al menos un ítem de verificación que yo no te haya dado. Candidatos con los que ya te rozaste: si la allowlist de CP-Swap todavía coincide con el fuente en el commit que fijaste, ya que una allowlist impresa se podre como cualquier matriz, o si las entradas del bypass de la whitelist cambiaron. La barra de aceptación es la del brief, palabra por palabra: cada extensión incluida está justificada contra la allowlist, y ninguna afirmación sin verificar de billetera o agregador se enuncia como hecho. Lee tu informe terminado a la caza de una oración que afirme algo que nunca mediste. Si no encuentras ninguna, y los gates pasan, R6 está listo y el diseño del SPROUT está cerrado.

Un pedido antes de que cierres la carpeta. Si tus propias corridas de verificación contradicen algo fechado en esta lección, una página de política de Jupiter que ahora existe, un flujo de badge de Orca que se movió, una sexta extensión en la allowlist de CP-Swap, publica la afirmación exacta y lo que encontraste en el canal de feedback del curso. Mis lecturas están fechadas el 2026-08-23 y todo el argumento de esta lección es que las afirmaciones fechadas decaen; un aprendiz que pesca una decayendo es el sistema funcionando, y el formato de informe que acabas de construir es exactamente donde pertenece esa pesca.

El SPROUT está resuelto: un token de economía que puedes construir, enrutar y defender en papel, con un informe que dice dónde se negocia y admite lo que no sabe. Eso cierra la mitad fungible de este curso. El próximo módulo deja atrás los fungibles hacia la capa de coleccionables: la pila de NFT de 2026, los metadatos que las billeteras de verdad leen, y la realidad de las regalías que nadie publicita. Trae el mismo escepticismo; el lado de los coleccionables tiene todavía más folclore para quemar.
