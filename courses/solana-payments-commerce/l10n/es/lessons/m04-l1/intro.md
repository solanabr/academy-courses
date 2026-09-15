# No confíes en ningún frontend: política de confirmación y verificación del lado del servidor

## Resumen

El módulo 3 te dejó tres superficies de checkout sobre un solo núcleo de pagos. Solo los caminos guiados por QR —el watcher de reference del checkout y el polling de findReference del POS— le preguntan a la blockchain antes de dar una venta por real, y solo responden por pagos que ya estaban mirando. El resto despacha por la palabra de un frontend. Esta es la lección donde eso se termina, para los tres a la vez.

Lo que te llevas hoy:

- Entregas **verifier**: un `verify(signature, expectedOrder)` del lado del servidor que trae la transacción él mismo, revisa el programa de tokens, el mint, el delta de saldo en tu propia cuenta y el memo del pedido, y después guarda un conjunto de firmas procesadas para que un webhook reentregado nunca pueda despachar dos veces. Es el harness de aceptación del curso; cada peldaño posterior corre contra él.
- El commitment de confirmación pasa a ser una decisión de política, no un valor por defecto de config: `confirmed` para el disco de $6, `finalized` para la factura mayorista de $6,000, `processed` para nada que toque el despacho.
- El único testigo al que llamas es `getTransaction` con encoding `jsonParsed`: el frontend te da afirmaciones, el libro mayor te da hechos.
- La comprobación ingenua ("existe una firma, por lo tanto está pagado") despacha contra un pago con el token equivocado. La ves hacer eso, y después le vas quitando el trabajo comprobación por comprobación.

El escenario por el que existe esta lección: el navegador de un comprador salta a un check verde y le hace POST de `paid: true` a tu servidor, así que despachas el disco. La transacción detrás de ese check verde movió 1.5 USDT, no USDC, a una cuenta que no es tuya. Nunca hubo un pago para tu pedido, y no existe riel de chargeback que revierta nada. El frontend mintió. ¿Quién es tu fuente de verdad?

Antes de contestar en código, mira la verdad cruda una vez tú mismo. Agarra una firma liquidada de devnet desde los logs del watcher de tu checkout y pregúntale al libro mayor directamente:

```bash
curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0", "id": 1, "method": "getTransaction",
  "params": ["<YOUR_SIGNATURE>", {"encoding": "jsonParsed", "commitment": "confirmed", "maxSupportedTransactionVersion": 1}]
}' | head -c 2000
```

Recorre ese JSON un segundo. En algún lugar adentro están los arrays `preTokenBalances` y `postTokenBalances`, y dentro de ellos un `mint`, un `owner`, un `programId` y un `amount` exacto en unidades base. Esa respuesta es toda la materia prima de esta lección. Todo lo que construimos es una forma disciplinada de leerla.

## El verificador, comprobación por comprobación

### El check verde es una afirmación

Seamos precisos sobre lo que ya verificas, porque el módulo 3 no era ingenuo. El watcher del checkout QR corría `validateTransfer` contra la blockchain: monto, destinatario, mint, para un transfer request que ya estaba mirando. Eso era verificación del lado del servidor de verdad, acotada a un solo flujo. Lo que nunca revisó: el programa de tokens detrás del mint. Lo que nunca tuvo: una opinión sobre transacciones que no estaba mirando de antes, que es exactamente lo que un webhook te va a pasar la próxima lección. Y los flujos de transaction request y de blink no tienen nada parecido; sus pruebas de humo demuestran que tus endpoints responden, no que llegó dinero. El verificador de hoy reemplaza todas esas comprobaciones de humo. Una función, todas las superficies, llamada con nada más que una firma y el pedido que crees que paga.

La regla de diseño vale la pena decirla como regla, porque es el módulo entero: **el frontend es una UI para el comprador, nunca un testigo para ti.** Una bandera `paid: true`, un redirect de éxito, una firma pegada en un formulario, todo eso es input controlado por el cliente. Lo único que un cliente no puede falsificar es lo que el libro mayor dice que hizo una transacción confirmada. Así que el servidor le pregunta al libro mayor, cada vez, y no despacha por nada más.

![El navegador manda afirmaciones falsificables como paid true y una firma, mientras el servidor trae hechos del libro mayor vía getTransaction, y solo el canal de hechos alimenta la decisión de despacho.](assets/v01-diagram.png)

Voy a confesar de dónde sale esta lección. Hace años, en un proyecto web2, cableé el despacho al redirect de éxito de un proveedor de pagos porque así lo hacía el ejemplo de los docs. Un tester con las devtools abiertas repitió ese redirect y se sacó un pedido gratis en menos de una hora, y el arreglo fue la API del proveedor para verificar del lado del servidor, que debería haber leído primero. La gente de Stripe conoce esto como la regla de que despachas desde el webhook más un PaymentIntent recuperado, nunca desde la URL de retorno del cliente. Misma regla aquí, con dientes más filosos: en los rieles de tarjeta mi error se recuperaba con un ticket de soporte. Aquí, lo que despachas contra un pago falso simplemente se fue.

### El commitment es un precio, no una configuración

Conociste `processed`, `confirmed` y `finalized` en el módulo 1, mapeados contra tu vocabulario de tarjetas. La guía cualitativa de los docs oficiales no cambió de forma: `confirmed` para la mayoría de los pagos, `finalized` para los de alto valor o sensibles a compliance, `processed` solo para la UI porque un bloque processed todavía puede ser descartado en un fork. Lo que cambia hoy es quién consume esa tabla. En el módulo 1 era un modelo mental. En esta lección es un parámetro que toma tu verificador, y elegirlo pago por pago es tu trabajo.

Primero los números, con su procedencia enunciada con cuidado porque este es un lugar donde la gente cita mal. Los docs te dan la tabla cualitativa y se detienen. Las cifras de latencia son estimaciones del ecosistema, observadas desde la red, no impresas en ningún doc oficial: `confirmed` aterriza en alrededor de 1 a 2 segundos; `finalized` quiere decir alrededor de 31 o más bloques confirmados construidos encima, lo que da alrededor de 10 segundos de reloj de pared (31 bloques al objetivo de 300ms son unos 9.3s; los tiempos de slot medidos corren un poco por encima del objetivo, y los bloques finales son "o más", de ahí el 10 redondo; vuelve a derivar esto cada vez que cambie el tiempo de slot, porque los recortes escalonados de SIMD-0525 lo siguen cambiando). Cítalas así. Una lección, un runbook o un memo de compliance que le atribuya esos números a los docs está citando algo que los docs nunca dijeron.

¿Y cuál compras? Ponle precio como a un seguro, porque literalmente es eso. La prima es latencia en tu checkout; la indemnización es protección contra que un bloque confirmado sea descartado en un fork, lo cual es raro, y mientras más valor carga un pago, más importa ese evento raro. Condicionar una venta de un solo disco de $6 a `finalized` es teatro: le cobras a cada cliente 10 segundos de mirar un spinner para asegurarte contra un riesgo que, a $6, redondea a cero. Condicionar una factura mayorista de $6,000 a `confirmed` es el error opuesto: un descarte por fork real, aunque raro, ahora te cuesta cuatro cifras sin ruta de reversión, y ahorraste ocho segundos en un pago que nadie estaba esperando en un mostrador. El commitment escala con lo que te cuesta un pago descartado. Escribe esa política como números, por nivel de producto, y deja que el verificador la haga cumplir.

![Una tabla de política de cuatro filas que empareja valores de pago con niveles de commitment: confirmed para las ventas de seis y de doscientos dólares, finalized para una factura de seis mil dólares, processed nunca.](assets/v02-comparison.png)

¿Y qué mira el comprador mientras tu servidor espera? Aquí es donde `processed` se gana el sueldo, porque es un nivel de UI y nada más. Muestra "pago visto" en el momento en que la transacción aparece en `processed`, en menos de un segundo, y cambia a "pagado" solo cuando el commitment de tu verificador se cumple. El comprador recibe feedback instantáneo, el despacho se queda con su garantía, y ninguno de los dos toma prestado el trabajo del otro. Y cuando despaches mal a pesar de todo, acuérdate de lo que estableció el módulo 1: no hay proceso de disputa por el que enrutar el error. Un reembolso en estos rieles es un push completamente nuevo de ti al comprador, construcción original, y construirlo como se debe es su propia lección más adelante en este módulo. El trabajo del verificador es volver los reembolsos una historia de atención al cliente en vez de un mecanismo de supervivencia.

¿Por qué esta decisión pesa más aquí de lo que pesó nunca en tarjetas? Por la asimetría alrededor de la cual este curso no deja de girar. Cuando Shopify anunció el soporte de Solana Pay (2023-08-23), la propuesta era que "elimina comisiones bancarias, chargebacks y tiempos de retención." Cada palabra de eso es una victoria para el comercio, y la línea del chargeback es la interesante, porque un chargeback nunca fue solo fraude contra ti. Era el botón de deshacer que el riel traía incorporado, con su precio metido en cada comisión de tarjeta como una prima de seguro. En estos rieles la prima desaparece y la política también: sin adquirente, sin proceso de disputa, sin ruta de reversión institucional. Lo que está en juego no es poco. Las stablecoins movieron alrededor de $27.6 billones de volumen de transferencias en 2024, superando a Visa y Mastercard juntas, la cifra fechada que viene de la guía de stablecoins de Helius que conociste en el módulo 1. El dinero a esa escala se está moviendo hacia rieles donde los errores de despacho son definitivos. El verificador que construyes hoy no es una pasada de endurecimiento que estaría bueno tener. Es todo el respaldo, y la lectura honesta de la propuesta de Shopify es que te están pagando la vieja prima de seguro a cambio de que construyas tu propio seguro. Buen canje, si de verdad lo construyes.

Una marca de roadmap, etiquetada como tal, antes de dejar atrás la latencia. La reescritura de consenso llamada Alpenglow (SIMD-0326) fue aprobada por gobernanza con alrededor del 99% del stake participante, está fusionada detrás de un feature gate que no está activo en mainnet, y apunta a fines de 2026 vía Agave 4.3 (estado revisado al momento de escribir esto, 2026-08-22). Si te pones a leer, fíjate en que el encabezado de estado del documento SIMD todavía dice "Review"; los encabezados van atrás de la realidad, y la votación pasó. Cuando se active, la matemática de finality debajo de toda esta tabla de política se comprime y te toca aflojar el lado de latencia del canje. Por qué finality funciona siquiera, votos, lockouts, y qué cambia Alpenglow por debajo, es territorio del curso Low-Level Solana. Para este curso se queda en lo que es aquí: una marca etiquetada sobre una política que vuelves a derivar cuando la red cambia debajo de ti.

### getTransaction, el único testigo que vale la pena llamar

Ahora la herramienta. `getTransaction` toma una firma y devuelve lo que esa transacción de verdad hizo, y con `encoding: "jsonParsed"` hace el decodificado de bytes por ti: los cambios de saldo de tokens llegan como entradas estructuradas y las instrucciones de los programas conocidos llegan pre-parseadas. Dos propiedades lo hacen el testigo correcto. Primero, solo responde en `confirmed` o `finalized`; literalmente no puedes preguntarle por una transacción `processed`, lo que quiere decir que tu política de commitment se enchufa directo en el fetch y el nivel demasiado débil es irrepresentable. Segundo, todo lo que hay en la respuesta lo calculó el validador que ejecutó la transacción, no algo que tocó el dispositivo del comprador.

¿Por qué no el más liviano `getSignatureStatuses`, que es en la práctica en lo que se apoyaba tu watcher del módulo 3 para el progreso? Porque un estado contesta "¿aterrizó, y en qué commitment?", y nada más. No puede decirte qué se movió, en qué token, bajo qué programa, a la cuenta de quién. Los estados son para barras de progreso. El despacho necesita el contenido de la transacción, y `getTransaction` es la llamada que lo devuelve.

La respuesta es un objeto grande. Tu verificador lee exactamente tres partes de él:

![Mapa anotado de una respuesta jsonParsed de getTransaction que marca los saldos de tokens pre y post, la instrucción spl-memo parseada que lleva el id del pedido, y el campo de error de meta.](assets/v03-annotated-code.png)

Tres hábitos para fijar mientras tienes la anatomía enfrente. Empareja `preTokenBalances` con `postTokenBalances` por `accountIndex`, y trata una entrada pre faltante como cero: una cuenta de token creada dentro de esta mismísima transacción (la ATA de un comprador primerizo, o la cuenta recién hecha de un atacante) tiene saldo post y ningún saldo pre. Haz la resta en `bigint` sobre las cadenas de `amount`; la lección de decimales ya te enseñó por qué los floats y el dinero nunca se juntan, y `uiAmount` es un float. Y lee el campo `owner`, no solo la dirección de la cuenta: las entradas de saldo te dicen quién es dueño de cada cuenta de token tocada, que es como el verificador encuentra los créditos hacia ti sin mantener una lista de cada cuenta de token que hayas tenido alguna vez.

### De la comprobación ingenua al verificador, un ataque a la vez

Aquí está la comprobación que se entrega en más bases de código de las que cualquiera admite, y es donde arranca el ejemplo trabajado:

```ts
// The naive check. Every line of this lesson exists because this is not enough.
async function naiveVerify(signature: string): Promise<boolean> {
  const tx = await fetchTransaction(signature);
  return tx !== null && tx.meta?.err == null;
}
```

Existe una firma, la transacción tuvo éxito, despacha el disco. Dale el escenario de la apertura: 1.5 USDT movidos entre dos cuentas, ninguna de ellas tuya, memo en blanco. `naiveVerify` devuelve `true`. Contesta "¿pasó alguna transacción?" cuando la pregunta es "¿este pedido se pagó?" Esas preguntas solo suenan parecidas. Cerramos la brecha un ataque a la vez, y el orden de las comprobaciones es parte del diseño: cada comprobación asume que las anteriores ya pasaron, y cada fallo devuelve la primera razón de la cadena, exactamente una, para que tu log de ops se lea como un diagnóstico en vez de como un encogimiento de hombros.

**Ataque 1: el mismo pago, dos veces.** No es maldad, es infraestructura. El webhook de la próxima lección va a reentregar eventos, porque la entrega al-menos-una-vez es como todo sistema de webhooks sobrevive a que tu servidor se caiga un rato. Una firma es determinista para su transacción, así que el evento reentregado lleva la misma firma, y un verificador sin memoria despacha el mismo pedido dos veces. El arreglo es el conjunto de firmas procesadas: primera comprobación al entrar, última escritura al tener éxito. Revisa `store.has(signature)` antes de hacer cualquier otra cosa y devuelve `duplicate`; llama a `store.add(signature)` solo después de que pasen todas las demás comprobaciones, para que una transacción rechazada se pueda reintentar pero una despachada quede quemada. Ese orden hace que el despacho sea exactamente-una-vez desde el mismo conjunto que lo hace seguro.

**Ataque 2: el monto correcto en el programa equivocado.** Este ataque es la piedra angular, y el que `validateTransfer` nunca cubrió. Hay dos programas de tokens en Solana: el Token clásico en `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` y Token-2022 en `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`. Cualquiera puede crear un mint de Token-2022, ponerle el nombre que quiera, acuñarse mil millones de unidades y transferir 30.000000 de ellas a una cuenta de token de la que tú eres dueño. On-chain eso es una transacción perfectamente válida cuyo `postTokenBalances` muestra tu dirección acreditada con el monto exacto que cobras. Un verificador que revisa monto y dueño pero no `programId` la deja pasar, y tu disco de $30 se acaba de vender por confeti. La comprobación es una línea, `credit.programId` tiene que ser igual al id del programa Token clásico, y la razón por la que tiene que venir antes de la comprobación del mint es lo bastante sutil como para decirla en voz alta: las direcciones de mint solo quieren decir lo que su programa dice que quieren decir. Comparar cadenas de mint antes de haber establecido qué programa las define es revisar la etiqueta de una botella que imprimió otra persona.

![Un atacante acredita treinta unidades de un mint sin valor de Token-2022 a una cuenta propiedad del comercio, pasando las comprobaciones de dueño y de monto pero fallando la comprobación del id de programa.](assets/v04-diagram.png)

**Ataque 3: un token real que no es tu token.** Misma forma, menos esfuerzo: pagarte 30 USDT cuando el precio era 30 USDC. Los dos viven bajo el programa Token clásico, así que la comprobación del ataque 2 pasa. Ahora la comprobación del mint se gana su lugar: el `mint` del crédito tiene que ser igual al mint en el que pones el precio. En mainnet, USDC quiere decir exactamente `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` y nada más; en devnet tu mint esperado es el que tu config de transfer-kit tenga fijado desde el módulo 2. Los nombres, los símbolos y los logos son metadatos que cualquiera puede copiar. La dirección es la identidad.

Una objeción justa antes del próximo ataque: ¿acabamos de declarar villano a Token-2022? No, y la distinción importa, porque ya conociste una stablecoin que vive ahí. PYUSD, que leíste en vivo on-chain allá en el módulo 2, es un mint de Token-2022, con sus ocho extensiones y todo. Si Wavelength algún día le pone precio a un artículo en PYUSD, vas a aceptar un pago de Token-2022 a propósito. La comprobación nunca dice Token clásico bueno, Token-2022 malo. Dice que el programa del crédito tiene que ser el programa bajo el que de verdad vive tu mint esperado, como par. En el lab el programa esperado es una constante, porque la tienda pone precio en USDC de Token clásico; el día que agregues un activo de Token-2022, el programa esperado pasa a ser un campo por pedido emparejado con su mint, y la comprobación misma sobrevive sin cambios. La única versión imperdonable es la que acepta un mint sin preguntar nunca qué programa lo define.

**Ataque 4: el número vino del comprador.** El formulario del pedido decía 30, el POST del cliente decía 30, y la transacción movió 0.30. Si tu verificador lee el monto desde cualquier lado que no sea la blockchain, lee una afirmación. La comprobación de verdad calcula el delta en tu propia cuenta: empareja pre y post por `accountIndex`, quédate con las entradas de las que eres dueño, toma el crédito, y exige `postAmount - preAmount >= expected.amountBaseUnits` en unidades base. Fíjate en la forma de la comparación: la transacción que no te tocó en absoluto es apenas el caso degenerado donde el delta es cero, y por eso el verificador trata "ningún crédito al comercio en ninguna parte de esta transacción" como un pago insuficiente de cero en vez de como un caso especial. La transacción de USDT-a-un-desconocido de la apertura muere justo aquí, incluso antes de que consideres su mint, porque ninguna de sus entradas de saldo te pertenece.

**Ataque 5: un pago real para otro pedido.** El más sutil de los cinco. La transacción es genuina, programa correcto, mint correcto, monto correcto, pagada a ti. Solo que es el pago del pedido `ord-0999`, y el comprador está repitiendo su firma contra el pedido `ord-1024`. Las comprobaciones de monto no pueden atrapar esto cuando dos discos cuestan lo mismo. El id de pedido que tu constructor de transaction request estampa en el spl-memo es la unión: el verificador encuentra la instrucción `spl-memo` parseada y exige que el id de pedido esperado aparezca ahí como campo entero, devolviendo `wrong-reference` si no. Campo entero, nunca subcadena, y este es el único lugar donde la gente se equivoca. El memo es `wavelength:<orderId>:<description>`, así que una comprobación con `.includes()` hace match con tu id en cualquier parte de esa cadena, descripción incluida; y `buildOrderTransaction` acepta un `input.orderId` provisto por quien llama cuando se lo dan, así que la forma del id tampoco te toca garantizarla a ti. Los fixtures del Challenge de esta noche te ponen las dos direcciones de contención enfrente: un pago con memo `ORD-42710` no debe despachar `ORD-4271`, y tampoco uno con memo `ORD-427`. Partir el memo por sus separadores y comparar un token por igualdad mantiene el formato del memo como asunto del constructor sin aceptar nunca un prefijo.

Cinco ataques, cinco comprobaciones, un pedido. Duplicado, después programa de tokens, después mint, después monto, después reference, y entonces y solo entonces guarda la firma y di `verified`:

![Diagrama de flujo del pipeline del verificador corriendo las comprobaciones de duplicado, fetch, programa de tokens, mint, delta de saldo y memo hasta el despacho, y cada fallo saliendo con una razón ordenada.](assets/v05-flowchart.png)

Una razón de ese diagrama no es como las demás. `not-found` es transitoria, no un veredicto: en commitment `confirmed` la transacción puede simplemente no ser visible todavía cuando dispara un webhook rápido, así que quien llama espera y reintenta en vez de rechazar el pedido. Una transacción que aterrizó pero falló no necesita ese cuidado, ni un caso especial tampoco: `meta.err` no nulo quiere decir que no se movió nada, sus deltas son cero, y la comprobación de pago insuficiente se encarga de ella.

### El conjunto que crece para siempre

El conjunto de firmas procesadas tiene un costo que la versión resumida de esta lección escondería, así que no lo hagamos. Cada pago despachado agrega una entrada, para siempre, y un conjunto que solo crece es estado sin límite: está bien al volumen de una tienda de discos, una factura de verdad al de un procesador de pagos. La salida es que las entradas dejan de ganarse el sueldo. El blockhash de una transacción no puede tener más de 150 bloques de antigüedad para aterrizar, lo que al tiempo de slot objetivo actual de 300ms es una ventana de alrededor de 45 segundos (derívala del tiempo de slot, y vuelve a derivarla cuando el tiempo de slot cambie: los recortes escalonados de SIMD-0525 ya movieron esta ventana dos veces, de ~60 segundos en los viejos slots de 400ms a ~53 en la etapa de 350ms al ~45 de hoy, y hay dos recortes más detrás de un feature gate en el código, así que una afirmación hardcodeada de "como un minuto" ya está dos eras atrasada). Pasada esa ventana, la misma transacción firmada nunca puede volver a aterrizar on-chain, así que la repetición on-chain se terminó físicamente. Lo que queda es la reentrega de webhooks desde tu propia infraestructura, que tiene su propio horizonte acotado de reintentos. Así que la regla de desalojo: guarda una firma por el tiempo de vida del blockhash más un margen generoso que cubra la ventana máxima en la que tu proveedor de webhooks puede reentregar, y después suéltala. El store en memoria del lab barre las entradas de más de diez minutos, una cota deliberadamente floja que sigue estando más de un orden de magnitud más allá de la ventana on-chain.

![Línea de tiempo que muestra una firma guardada protegiendo contra la repetición on-chain durante unos cuarenta y cinco segundos y contra la reentrega de webhooks durante minutos, y después siendo desalojada a los diez minutos una vez que las dos ventanas se cerraron.](assets/v06-timeline.png)

La persistencia es otro eje, y vale una frase honesta: un conjunto en memoria se olvida al reiniciar, así que la producción mueve la misma interfaz de dos métodos a tu base de datos de pedidos, donde una fila de pedido despachado con una columna de firma es el conjunto. La interfaz que construyes hoy convierte ese cambio en un argumento del constructor.

## Lab: construye el harness de aceptación

Cómo se reparte el trabajo, dicho en voz alta: yo recorro contigo el workspace, los tipos, el store, el adaptador RPC y el conjunto de fixtures, todo trabajado. Las dos comprobaciones en el corazón del verificador, la de programa-de-tokens-y-mint y la del delta de saldo, son TODOs andamiados que llenas tú mismo, con la sección de teoría de arriba como referencia (completion). La corrida en vivo en devnet y la lógica endurecida de razón ordenada son solo tuyas (solo, en el Challenge). La barrera es que `npm run verify:verifier` imprima su línea de aprobación completa.

**1. Genera el scaffold del workspace.** El verificador vive al lado de tus workspaces de checkout y se queda en la misma línea de kit que usan ellos (`@solana/kit` 6.10.0, el último release de la línea v6; el `latest` de npm es 8.0.0 al 2026-08-22 y v7 es el estándar peer actual del ecosistema, una costura que los workspaces cliente del curso retoman más adelante, pero el lado de ops se queda consistente con el código que verifica):

```bash
mkdir -p verifier/src verifier/fixtures
cd verifier
npm init -y
npm pkg set type=module
npm install @solana/kit@6.10.0
npm install -D tsx@4 typescript @types/node
npm pkg set scripts.verify:verifier="tsx verify-harness.ts"
```

Los pins y su frescura: `tsx` 4 es el runner que usa todo el curso (esta línea es su instalación si estás en una máquina nueva); `type=module` importa porque el harness usa `import.meta.dirname` para encontrar sus fixtures. El directorio `fixtures/` empieza vacío a propósito: en el paso 7 lo siembras tú mismo con cinco archivos, un pago correcto y cuatro ataques sembrados, cada uno un JSON con forma de `getTransaction` más el pedido que afirma pagar y la razón que el verificador tiene que devolver.

**2. Los tipos, que también son el contrato.** Guárdalo como `verifier/src/types.ts`. Las formas de `ExpectedOrder` y `VerifyResult` quedan congeladas de aquí en adelante: la lección del webhook, el dashboard de ops y el capstone llaman todos a `verify(signature, expectedOrder)` exactamente como está tipado aquí.

```ts
// verifier/src/types.ts

export interface ExpectedOrder {
  orderId: string;         // the id your txreq builder stamps into the spl-memo
  recipient: string;       // the merchant owner address (base58)
  recipientAta: string;    // the merchant token account for the expected mint
  mint: string;            // the mint you price in (base58)
  amountBaseUnits: bigint; // the exact price, integer base units, never a float
}

export type RejectReason =
  | 'duplicate'
  | 'wrong-token-program'
  | 'wrong-mint'
  | 'underpaid'
  | 'wrong-reference';

export type VerifyResult =
  | { ok: true; reason: 'verified'; signature: string }
  | { ok: false; reason: RejectReason | 'not-found'; signature: string };

export interface TokenBalanceEntry {
  accountIndex: number;
  mint: string;
  owner?: string;
  programId?: string;
  uiTokenAmount: { amount: string; decimals: number };
}

export interface ParsedInstructionLike {
  program?: string;
  programId: string;
  parsed?: unknown;
}

export interface VerifiableTransaction {
  meta: {
    err: unknown;
    preTokenBalances: readonly TokenBalanceEntry[];
    postTokenBalances: readonly TokenBalanceEntry[];
  } | null;
  instructions: readonly ParsedInstructionLike[];
}

export type FetchTransaction = (
  signature: string,
) => Promise<VerifiableTransaction | null>;

export interface ProcessedSignatureStore {
  has(signature: string): boolean;
  add(signature: string): void;
}
```

Dos decisiones deliberadas para notar. `FetchTransaction` es una función inyectada, no una llamada RPC cableada, que es lo que deja al harness correr los ataques sembrados desde fixtures mientras producción corre contra devnet: mismo verificador, distinto suministro de testigos. Y `recipientAta` viaja adentro de `ExpectedOrder` aunque las comprobaciones se apoyen en `owner`: con dueño, mint y programa todos validados, la dirección de la ATA es determinista, y el campo documenta en qué cuenta aterrizó el delta del camino feliz para tus registros de conciliación.

**3. El store.** Guárdalo como `verifier/src/store.ts`. Todo trabajado; el barrido de desalojo es el horizonte de la sección de teoría hecho concreto:

```ts
// verifier/src/store.ts
import type { ProcessedSignatureStore } from './types.ts';

export function createMemoryStore(
  evictionMs = 10 * 60_000,
): ProcessedSignatureStore {
  const seen = new Map<string, number>();

  function sweep(now: number): void {
    for (const [sig, storedAt] of seen) {
      if (now - storedAt > evictionMs) seen.delete(sig);
    }
  }

  return {
    has(signature) {
      return seen.has(signature);
    },
    add(signature) {
      const now = Date.now();
      sweep(now);
      seen.set(signature, now);
    },
  };
}
```

**4. El núcleo del verificador, tu peldaño de completion.** Guárdalo como `verifier/src/verify.ts`. El esqueleto del pipeline, el corchete de dedup y la comprobación del memo vienen dados. Las dos regiones de TODO son las que la sección de teoría ya derivó; escríbelas, no las pegues a ciegas, porque las pruebas del Challenge van a interrogar tu entendimiento de las dos:

```ts
// verifier/src/verify.ts
import type {
  ExpectedOrder,
  FetchTransaction,
  ProcessedSignatureStore,
  VerifiableTransaction,
  VerifyResult,
} from './types.ts';

export const TOKEN_PROGRAM = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
export const TOKEN_2022_PROGRAM = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';

interface Credit {
  programId: string;
  mint: string;
  delta: bigint;
}

function creditToRecipient(
  tx: VerifiableTransaction,
  recipient: string,
): Credit | null {
  if (tx.meta === null) return null;

  // TODO(delta): pair preTokenBalances to postTokenBalances by accountIndex,
  // keeping only entries owned by `recipient`. A missing pre-entry is 0n.
  // Compute delta = post - pre as bigints from uiTokenAmount.amount.
  // Return the credit with the LARGEST positive delta as
  // { programId, mint, delta }, or null when nothing credited the recipient.
  throw new Error('TODO(delta): compute the recipient balance delta');
}

function memosOf(tx: VerifiableTransaction): string[] {
  const out: string[] = [];
  for (const ix of tx.instructions) {
    if (ix.program === 'spl-memo' && typeof ix.parsed === 'string') {
      out.push(ix.parsed);
    }
  }
  return out;
}

export function createVerifier(deps: {
  fetchTransaction: FetchTransaction;
  store: ProcessedSignatureStore;
}) {
  return async function verify(
    signature: string,
    expected: ExpectedOrder,
  ): Promise<VerifyResult> {
    if (deps.store.has(signature)) {
      return { ok: false, reason: 'duplicate', signature };
    }

    const tx = await deps.fetchTransaction(signature);
    if (tx === null) return { ok: false, reason: 'not-found', signature };

    // A transaction that never credited you is a zero credit in the right
    // program and mint: the delta check rejects it, no special case needed.
    const credit = creditToRecipient(tx, expected.recipient) ?? {
      programId: TOKEN_PROGRAM,
      mint: expected.mint,
      delta: 0n,
    };

    // TODO(program-and-mint): reject with 'wrong-token-program' when the
    // credit's programId is not the classic Token program, THEN reject with
    // 'wrong-mint' when its mint is not expected.mint. Order matters: a mint
    // string only means something once its program is established.

    if (credit.delta < expected.amountBaseUnits) {
      return { ok: false, reason: 'underpaid', signature };
    }

    // Whole-field equality, never substring: ids are variable-length, so a
    // payment memoed `ord-1024` would otherwise fulfill order `ord-102`.
    const memoMatches = memosOf(tx).some((m) =>
      m.split(/[\s:]+/).includes(expected.orderId),
    );
    if (!memoMatches) {
      return { ok: false, reason: 'wrong-reference', signature };
    }

    deps.store.add(signature);
    return { ok: true, reason: 'verified', signature };
  };
}
```

**5. El adaptador RPC.** Guárdalo como `verifier/src/rpc.ts`. Todo trabajado. Aquí es donde se enchufa la política de commitment, y donde la forma del cable se estrecha hacia nuestros tipos exactamente una vez:

```ts
// verifier/src/rpc.ts
import { createSolanaRpc, signature as asSignature } from '@solana/kit';
import type {
  FetchTransaction,
  ParsedInstructionLike,
  TokenBalanceEntry,
  VerifiableTransaction,
} from './types.ts';

// The wire shape we rely on from getTransaction with encoding: 'jsonParsed'.
// Narrowed once, here, at the RPC boundary; everything downstream is our types.
interface RawGetTransactionResponse {
  meta: {
    err: unknown;
    preTokenBalances?: readonly TokenBalanceEntry[];
    postTokenBalances?: readonly TokenBalanceEntry[];
  } | null;
  transaction: {
    message: { instructions: readonly ParsedInstructionLike[] };
  };
}

export function createRpcFetchTransaction(opts: {
  url?: string;
  commitment?: 'confirmed' | 'finalized';
} = {}): FetchTransaction {
  const rpc = createSolanaRpc(
    opts.url ?? process.env.RPC_URL ?? 'https://api.devnet.solana.com',
  );
  const commitment = opts.commitment ?? 'confirmed';

  return async (sig: string): Promise<VerifiableTransaction | null> => {
    const response = await rpc
      .getTransaction(asSignature(sig), {
        commitment,
        encoding: 'jsonParsed',
        maxSupportedTransactionVersion: 1,
      })
      .send();

    if (response === null) return null;
    const raw = response as unknown as RawGetTransactionResponse;

    return {
      meta:
        raw.meta === null
          ? null
          : {
              err: raw.meta.err,
              preTokenBalances: raw.meta.preTokenBalances ?? [],
              postTokenBalances: raw.meta.postTokenBalances ?? [],
            },
      instructions: raw.transaction.message.instructions,
    };
  };
}
```

Mira la opción `commitment` y vuelve a ver la sección de política: el nivel de $6 construye este adaptador con `'confirmed'`, el nivel de factura con `'finalized'`, y `processed` no está en el tipo. La política quedó irrepresentable-si-está-mal, que es la forma más barata de hacerla cumplir.

Mira `maxSupportedTransactionVersion` ya que estás, porque es la otra opción de ese objeto que puede hacer fallar al verificador por completo, y falla ruidosamente en vez de en silencio. Es un techo, no una preferencia: el RPC se niega —error `-32015`— a devolver cualquier transacción cuya versión sea más alta que el número que pasas. `0` quiere decir "entiendo legacy y v0", lo que describía cada transacción de la red hasta que se entregó el formato de transacción v1, y ya no. Pasa `1` y lees las dos. Déjalo en `0` y el día que un cliente te pague con una transacción v1 tu verificador no devuelve la respuesta equivocada, devuelve un error, y ese pago se queda sin conciliar hasta que alguien se dé cuenta. Este es un parámetro del cable y no una capacidad del SDK, así que no te cuesta ningún bump de versión: kit 6.10.0, el pin en el que está este workspace, ya tipa `TransactionVersion` como `'legacy' | 0 | 1`.

**6. El harness.** Guárdalo como `verifier/verify-harness.ts`. Corre cada fixture por tu verificador con un fetch inyectado respaldado por fixtures, verifica que cada uno devuelva su razón esperada, y después opcionalmente verifica dos veces un pago en vivo de devnet para demostrar el dedup. Una guarda que vale la pena señalar antes de guardarlo: un directorio `fixtures/` vacío falla ruidosamente en vez de pasar. Un harness que no encuentra nada que probar y aun así imprime la línea de aprobación es el check verde del frontend otra vez, una afirmación sin ningún testigo detrás:

```ts
// verifier/verify-harness.ts
// Runs the verifier against the seeded-attack fixtures you author in step 7,
// then (when REAL_SIGNATURE is set) against a live devnet payment. This is the
// course's acceptance harness: later lessons re-run it against their own
// artifacts.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { createVerifier } from './src/verify.ts';
import { createMemoryStore } from './src/store.ts';
import { createRpcFetchTransaction } from './src/rpc.ts';
import type {
  ExpectedOrder,
  VerifiableTransaction,
  VerifyResult,
} from './src/types.ts';

interface Fixture {
  name: string;
  signature: string;
  expectedReason: VerifyResult['reason'];
  order: Omit<ExpectedOrder, 'amountBaseUnits'> & { amountBaseUnits: string };
  transaction: VerifiableTransaction;
}

function fail(msg: string): never {
  console.error(`VERIFY FAIL: ${msg}`);
  process.exit(1);
}

async function main() {
  const fixtureDir = join(import.meta.dirname, 'fixtures');
  const fixtures: Fixture[] = readdirSync(fixtureDir)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(fixtureDir, f), 'utf8')));

  if (fixtures.length === 0) {
    fail(`no fixtures in ${fixtureDir}: nothing was tested`);
  }

  const bySignature = new Map(fixtures.map((f) => [f.signature, f.transaction]));
  const verify = createVerifier({
    fetchTransaction: async (sig) => bySignature.get(sig) ?? null,
    store: createMemoryStore(),
  });

  for (const f of fixtures) {
    const result = await verify(f.signature, {
      ...f.order,
      amountBaseUnits: BigInt(f.order.amountBaseUnits),
    });
    if (result.reason !== f.expectedReason) {
      fail(`${f.name}: expected ${f.expectedReason}, got ${result.reason}`);
    }
    console.log(`  ${f.name}: ${result.reason}`);
  }

  const realSig = process.env.REAL_SIGNATURE;
  if (realSig) {
    const order: ExpectedOrder = {
      orderId: process.env.ORDER_ID ?? fail('set ORDER_ID for the live check'),
      recipient: process.env.MERCHANT ?? fail('set MERCHANT'),
      recipientAta: process.env.MERCHANT_ATA ?? fail('set MERCHANT_ATA'),
      mint: process.env.MINT ?? fail('set MINT'),
      amountBaseUnits: BigInt(process.env.AMOUNT_BASE_UNITS ?? '0'),
    };
    const liveVerify = createVerifier({
      fetchTransaction: createRpcFetchTransaction(),
      store: createMemoryStore(),
    });
    const first = await liveVerify(realSig, order);
    if (first.reason !== 'verified') fail(`live payment: ${first.reason}`);
    const second = await liveVerify(realSig, order);
    if (second.reason !== 'duplicate') {
      fail(`redelivery not deduped: ${second.reason}`);
    }
    console.log('  live devnet payment: verified once, duplicate on redelivery');
  }

  console.log(
    'verifier: correct payment fulfilled; wrong-token, wrong-mint, underpay, replayed-reference rejected; signature stored',
  );
}

main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
```

**7. Siembra los ataques.** El harness es tan honesto como las transacciones que le des, así que los testigos los escribes tú: cinco archivos en `verifier/fixtures/`, cada uno con la forma exacta de la interfaz `Fixture` que está arriba del harness que acabas de guardar. Cada archivo lleva una `transaction` con forma de `getTransaction`, el `order` que afirma pagar, y la única razón que tu verificador tiene que devolver para él. Los fixtures fijan a propósito las direcciones reales de los mint de USDC y USDT en mainnet, porque la regla de la-dirección-es-la-identidad es más fácil de internalizar con las identidades reales en la página; la corrida en vivo del paso 8 mete tu mint de devnet por las variables de entorno, y el verificador nunca nota la diferencia. Los prefijos numéricos solo mantienen la salida de `readdirSync` en orden de lectura.

![Los cuatro campos de un archivo de fixture anotados con cómo los consume el harness, la firma como clave del fetch falso y la razón esperada manejando la aserción.](assets/v07-annotated-code.png)

Primero, el pago que tiene que pasar. El lado del comprador de la transferencia viaja adentro de los arrays de saldo a propósito: tu código de delta tiene que encontrar la entrada propiedad del comercio entre desconocidos, que es todo el punto de apoyarse en `owner`. El crédito es exactamente 30 USDC, pre 1.000000 y post 31.000000. Guárdalo como `verifier/fixtures/01-correct-payment.json`:

```json
{
  "name": "correct payment",
  "signature": "FixSigCorrectPayment11111111111111111111111",
  "expectedReason": "verified",
  "order": {
    "orderId": "ord-1024",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "BuyerWa11etAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "80000000", "decimals": 6 }
        },
        {
          "accountIndex": 2,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "BuyerWa11etAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "50000000", "decimals": 6 }
        },
        {
          "accountIndex": 2,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "31000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-token",
        "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
      },
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1024:clube-single"
      }
    ]
  }
}
```

Después, la imitación del ataque 2. Todo lo que lee una comprobación parcial está bien: el monto exacto, el memo copiado, y la cuenta propiedad del comercio no tiene ninguna entrada pre, porque el atacante la creó dentro de esta mismísima transacción. Tu regla de pre-faltante-es-cero es lo que hace que el delta salga en 30. Solo el `programId` la delata, y por eso este fixture demuestra que tus comprobaciones corren en el orden correcto. Guárdalo como `verifier/fixtures/02-wrong-token-program.json`:

```json
{
  "name": "token-2022 look-alike",
  "signature": "FixSigWrongProgram2222222222222222222222222",
  "expectedReason": "wrong-token-program",
  "order": {
    "orderId": "ord-1025",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "FakeUsdcTwentyTwo22222222222222222222222222",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": { "amount": "30000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1025:clube-single"
      }
    ]
  }
}
```

Ataque 3: 30 USDT reales bajo el programa correcto, con un memo correcto, aterrizando en tu cuenta de USDT. La comprobación de dueño igual encuentra el crédito aunque nunca tocó el `recipientAta` del pedido, y la comprobación del mint tiene que ser la que lo mata. Guárdalo como `verifier/fixtures/03-wrong-mint.json`:

```json
{
  "name": "usdt into your usdt account",
  "signature": "FixSigWrongMint3333333333333333333333333333",
  "expectedReason": "wrong-mint",
  "order": {
    "orderId": "ord-1026",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "5000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "35000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1026:clube-single"
      }
    ]
  }
}
```

Ataque 4: token correcto, 1.50 de menos. El delta sale en 28500000 contra un esperado de 30000000, y todos los demás campos son honestos. Guárdalo como `verifier/fixtures/04-underpaid.json`:

```json
{
  "name": "right token, 1.50 short",
  "signature": "FixSigUnderpaid4444444444444444444444444444",
  "expectedReason": "underpaid",
  "order": {
    "orderId": "ord-1027",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "29500000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1027:clube-single"
      }
    ]
  }
}
```

Ataque 5: una transacción genuina y completamente pagada para `ord-0999`, repetida contra el pedido `ord-1024`. Cada comprobación hasta el memo pasa, y el memo es lo único que queda en pie entre esta firma y un disco gratis. Guárdalo como `verifier/fixtures/05-wrong-reference.json`:

```json
{
  "name": "ord-0999 payment replayed against ord-1024",
  "signature": "FixSigWrongReference55555555555555555555555",
  "expectedReason": "wrong-reference",
  "order": {
    "orderId": "ord-1024",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "31000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-0999:clube-single"
      }
    ]
  }
}
```

**8. Córrelo.** Primero los fixtures:

```bash
npm run verify:verifier
```

Con tus dos TODOs llenados correctamente, cada fixture imprime su razón y la línea final es la frase de aprobación completa. Si el harness se niega con `no fixtures`, tus cinco archivos del paso 7 no están donde apunta `import.meta.dirname`; esa negativa es deliberada, porque un harness que no probó nada no tiene por qué imprimir una aprobación. Si `wrong-token-program` vuelve como `wrong-mint`, tus comprobaciones están en el orden equivocado; si el fixture de pago insuficiente verifica, tu delta compara floats o cadenas en vez de bigints. El conjunto de fixtures cubre exactamente lo que derivó la teoría:

![Tabla de cinco fixtures, un pago correcto y cuatro ataques, cada uno emparejado con la única razón que el verificador tiene que devolver y la propiedad que esa razón demuestra.](assets/v08-table.png)

Después la mitad en vivo. Haz un pago nuevo en devnet por tu checkout de la lección del QR, o manda uno directo con transfer-kit —de cualquiera de las dos formas tiene que venir del cliente de mentira, no de tu keypair de comercio, porque la comprobación de monto de este verificador es un delta de saldo en tu propia cuenta de token y un pago a ti mismo la mueve en cero:

```bash
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5
```

Anota la firma que imprime y el id del pedido contra el que estás probando, y después:

```bash
REAL_SIGNATURE=<sig> ORDER_ID=<id> MERCHANT=$(solana address) \
MERCHANT_ATA=<your usdc ata> MINT=<your devnet mint> AMOUNT_BASE_UNITS=<price> \
npm run verify:verifier
```

El checkpoint es concreto: el harness imprime `live devnet payment: verified once, duplicate on redelivery` seguido de la línea de aprobación. Esa segunda cláusula es tu primera supervivencia a un reintento de webhook, demostrada antes de recibir siquiera un webhook.

## Challenge

**El challenge de código (harden-verify, en el widget del challenge)** te entrega un starter que es el verificador ingenuo de la sección de teoría más la comprobación de dedup que ya construiste; los cinco huecos que quedan son tuyos para cerrar. Viene con un conjunto de transacciones que contiene un pago correcto y todos los ataques sembrados. El starter despacha contra el pago con el token equivocado, a propósito; míralo pasar una vez antes de arreglar nada, porque ver el falso positivo es la lección. Tu trabajo es el contrato de razón ordenada: exactamente una razón por llamada entre `duplicate`, `wrong-token-program`, `no-payment`, `wrong-mint`, `underpaid`, `wrong-reference` y `verified`, con el monto siempre calculado desde el delta de saldo on-chain. El widget llama a tu `verifyPayment` por posición, que es como puede su calificador — aquí está el contrato de siete argumentos como lista, porque lo vas a releer a mitad de un debug:

1. La transacción parseada, como una sola cadena JSON.
2. `expectedMint`, plano.
3. `expectedTokenProgram`, plano.
4. `recipientAta`, plano.
5. `expectedAmount`, como bigint.
6. `orderRef`, plano.
7. El conjunto de firmas despachadas, como otra cadena JSON.

El starter ya le hace `JSON.parse` a las dos cadenas (argumentos 1 y 7) al entrar, así que tus comprobaciones trabajan con valores reales; los montos de transferencia llegan como cadenas decimales de unidades base, la forma en la que `getTransaction` los reporta, así que levántalos con `BigInt()` antes de la matemática de dinero. `Number()` es exacto solo por debajo de 2**53 y una de las transacciones del widget liquida por encima, que es la lección de decimales cobrando su deuda. Dos diferencias chicas de forma respecto del lab, las dos enunciadas en el widget: te entrega una lista de transferencias ya aplanada en vez de arrays de saldo pre/post, así que "no aterrizó nada en tu ATA" es su propia razón `no-payment` en vez del pago insuficiente con delta cero del lab, y su memo es la reference de pedido pelada, así que la comprobación de reference es igualdad sobre el memo entero en vez del match de campo entero que el lab hace dentro de uno estructurado. Las dos rechazan un prefijo, que es la propiedad que importa. Los tres hints del widget son los tres errores que comete todo el mundo, en orden de popularidad: confiar en una transferencia antes de revisar su programa, leer un monto de un campo del cliente, y devolver una pila de razones en vez de la primera.

**El peldaño solo** es la barrera de evaluación de esta lección, y es aquello por lo que este módulo lleva su nombre. Corre tu verificador terminado contra un pago real de devnet y contra todos los ataques sembrados; tiene que despachar solo el correcto y guardar su firma. Después haz la parte que ninguna prueba puede revisar por ti, y hazla en el archivo que la está esperando desde el módulo 1: abre `commitment-policy.md`, el documento de cuatro encabezados que redactaste en m01-l2 y copiaste a la raíz de `wavelength` en m02-l1. El módulo 1 prometió dos veces que el módulo 4 convertiría ese archivo en código que corre, y este es el paso que paga la promesa. Afila sus encabezados en una tabla de tres niveles de valor, cada uno con su nivel de commitment y una frase que lo defienda contra la asimetría de no tener chargebacks — y después cabléala: la cadena de commitment que tu verificador le pasa a `getTransaction` tiene que ser la que tu tabla nombra para ese nivel, no un valor por defecto que alguien tipeó una vez. Una política con la que el código no está de acuerdo es un documento, no una política. No hay una tabla universalmente correcta. Hay una tabla que tu código obedece y que puedes defender, y las dos mitades son la habilidad.

Una cosa más antes de que cierres el editor, porque replantea todo lo que acabas de construir. Este verificador sobrevive a la lección de hoy:

![Diagrama que muestra la función verify construida hoy consumida por la lección del webhook, los peldaños de pago posteriores y el harness de aceptación del capstone, todos canalizando firmas por las mismas comprobaciones.](assets/v09-diagram.png)

Sea lo que sea que construyas para Wavelength de aquí en adelante, la verdad de los pagos fluye por esta única función. La deriva de interfaz aquí rompe cada lección posterior, y por eso exactamente los tipos se congelaron en el paso 2.

## Checkpoint: qué demuestra la aprobación

Si el harness te puso resistencia, el fallo es uno de una lista corta. Que las razones vuelvan en el orden equivocado quiere decir que tu comprobación de programa está después de la del mint; relee la última frase del ataque 2. Un `verified` en el fixture de pago insuficiente quiere decir comparación de floats o de cadenas; los deltas son bigints o son bugs. Un `not-found` en tu pago en vivo normalmente quiere decir que el problema de velocidad del webhook llegó temprano: verificaste en `confirmed` antes de que el RPC pudiera ver la transacción, así que espera un momento y vuelve a correr. Y si la corrida en vivo verificó pero la segunda llamada no imprimió `duplicate`, tu `store.add` está en el lugar equivocado; va después de la comprobación final, en ningún lado antes.

Cuando se imprima la línea de aprobación, detente en lo que de verdad tienes. Cuando se abrió este módulo, dos de tus tres superficies despachaban por la palabra de un frontend y la tercera solo respondía por pagos que ya estaba mirando. Ahora no se despacha nada hasta que una función que escribiste lee el libro mayor y está de acuerdo, no se le puede hacer doble despacho con un reintento, no se le puede engañar con un programa falsificado ni con un mint imitación ni con un comprobante repetido, y el nivel de certeza que exige es una política que pusiste a precio deliberadamente, pago por pago. Eso es el back office ganándose el sueldo, y fue tu construcción más difícil hasta ahora. ¡Tómate la victoria!

Ahora tienes el verificador que es el harness de aceptación del curso. Pero fíjate en lo que todavía no puede hacer: solo inspecciona los pagos de los que le avisan. Alguien tiene que pasarle una firma. La próxima lección llega ese alguien, el webhook que mira tu dirección y llama a tu servidor en cada pago, y nos ponemos precisos sobre exactamente cómo miente ese webhook: reentregas, eventos fuera de orden, y por qué su idempotencia con clave de firma aterriza en el store que construiste hoy.
