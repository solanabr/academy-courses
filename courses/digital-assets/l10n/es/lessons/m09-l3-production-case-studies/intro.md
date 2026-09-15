# Casos de producción y la frontera: lo que de verdad se entregó

## Resumen

La lección pasada cerraste la economía de Overgrowth. El acceso al alpha se restringe con la respuesta a "¿esta billetera tiene un cNFT Founding-Farmer?", tomada de una lectura de DAS y no de la promesa de un cliente, y los puntos de compost se volvieron SPROUT de verdad por la ruta de Merkle, con el reclamo marcado y el segundo reclamo rechazado. Mint, cNFT, airdrop, lector, control de acceso. Cada pieza que construiste ahora toca a todas las demás.

Así que aquí está la pregunta que cierra el arco: ¿alguien entrega de verdad estas cosas, o pasaste nueve módulos aprendiendo una especificación?

Respóndela tú mismo, ahora mismo, antes de leer otro párrafo. Esto lee una cuenta de mainnet y no necesita nada instalado:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
  | grep -o '"extension":"[a-zA-Z]*"'
```

Vuelven ocho líneas. `mintCloseAuthority`, `permanentDelegate`, `transferFeeConfig`, `confidentialTransferMint`, `confidentialTransferFeeConfig`, `transferHook`, `metadataPointer`, `tokenMetadata`. Ese es PayPal USD, lanzado en Solana en mayo de 2024 por PayPal y Paxos, con ocho extensiones TLV encima de un dólar regulado y vivo. Leíste esas mismas ocho en la primera lección de este curso, cuando eran formas que no podías explicar. Ahora puedes explicar las ocho.

Esto es lo que entonces no podías hacer y hoy vas a hacer. Cuatro de esas ocho están apagadas. No ausentes. Apagadas, con el interruptor todavía cableado y la clave de alguien todavía puesta encima. Esa brecha entre "la extensión está presente en los bytes" y "la extensión hace algo" es donde vive toda evaluación real de un token ya lanzado, y leerla mal en cualquiera de las dos direcciones es como la gente sale lastimada.

La ruta por esta lección: primero qué dice hoy el mint de PYUSD y cómo derivar activo-versus-dormido de los valores y no de la presencia, después la economía de un slot armado, que es la parte a la que nadie le pone precio. Después tres cosas ya lanzadas medidas contra lo que construiste: el drop de JTO de Jito contra tu ruta de Merkle, los rieles de stablecoins que pagan todo esto, y el trabajo de identidad de agentes que es genuinamente nuevo y genuinamente no probado. Te vas con `dormancy-report.ts`, una herramienta que apunta a cualquier mint y te dice cuáles de sus poderes están vivos.

El repliegue de la ayuda: el clasificador está trabajado completo para las extensiones que PYUSD lleva, tú escribes la regla para una que no lleva, y el memo del final es enteramente tuyo. Ese memo es la pieza en la que te calificas a ti mismo, y tiene la misma forma que el que tu capstone te pide la semana que viene.

## Armado, no disparado

### Qué dicen hoy ocho slots

La presencia es barata de leer. El comportamiento no, y los cuatro campos que cargan el comportamiento están repartidos en cuatro cuerpos de extensión distintos con cuatro nombres distintos. Empieza por los dos sobre los que gira todo el caso de estudio, leídos en vivo de mainnet el 2026-09-01:

`transferHook.programId` es `null`. Hay un slot de hook en el mint de PYUSD, y ningún programa dentro. Nada se invoca en la transferencia, no porque Token-2022 se niegue, sino porque el emisor no ha nombrado un programa que invocar. Tu propia lección de hooks construyó el otro lado de ese slot: un programa al que se le hace CPI en cada transferencia y que puede rechazar una. PYUSD tiene el tomacorriente y ninguna clavija.

`transferFeeConfig` lee 0 basis points, comisión máxima 0, en las dos entradas de comisión, la más vieja y la más nueva, las dos con el sello del epoch 605, que es la manera que tiene la extensión de decir que nunca se ha programado ningún cambio de tasa contra este mint y que cada transferencia de PYUSD que se ha liquidado alguna vez ha retenido exactamente nada. La extensión que le permitiría a un emisor regulado quedarse con una tajada de cada movimiento de un dólar está presente y puesta en cero.

Dos más que se leen inertes en cuanto miras más allá de la etiqueta. `confidentialTransferMint` tiene `autoApproveNewAccounts: false` y ninguna auditor key, lo que quiere decir que ninguna cuenta obtiene saldos confidenciales hasta que el emisor apruebe esa cuenta específica. Los rieles existen. El torniquete está trabado. Y `confidentialTransferFeeConfig` lleva un ciphertext retenido de puros ceros, que es exactamente lo que esperarías de un esquema de comisión que nunca ha cobrado nada.

![Una lectura JSON recortada del mint de PYUSD lleva llamadas de atención sobre el programId del hook, la entrada de comisión, los flags de transferencia confidencial y el delegado permanente, cuatro valores inertes y un poder vivo.](assets/v01-annotated-code.png)

Ahora el campo que no es nada inerte. `permanentDelegate.delegate` nombra una dirección, y también lo hacen `mintCloseAuthority.closeAuthority`, `metadataPointer.metadataAddress` y el cuerpo `tokenMetadata` que resuelve a "PayPal USD". Esos cuatro hacen algo hoy. Un delegado permanente puede mover o quemar PYUSD de cualquier cuenta sin que el dueño firme, que es la forma on-chain de una orden judicial, y está encendido ahora mismo.

Cuatro vivas, cuatro dormidas, un mint.

### Presente, activo y ejercitable son tres preguntas

El reflejo que tiene la mayoría es binario: la extensión está ahí, entonces el token hace esa cosa. Ese reflejo está mal en las dos direcciones y yo me he equivocado en las dos direcciones. Hace años miré la lista de extensiones de un mint Token-2022, vi `transferFeeConfig`, le dije a un compañero de equipo "hay comisiones, no lo enrutes," y nunca abrí el cuerpo. Cero bps. Nos hice perder un día con un token que se portaba exactamente como una transferencia simple. El error opuesto es peor y he visto a gente cometerlo: ver una comisión en 0 y concluir que el token es seguro para tratarlo como libre de comisión para siempre.

Tres preguntas, en orden, y necesitas las tres:

1. **¿La extensión está presente?** Lee la lista TLV. Barato, y es donde se detiene la mayoría.
2. **¿Su valor hace algo hoy?** Lee el campo específico. Program id del hook, basis points de comisión, dirección del delegado, flag de aprobación.
3. **¿Alguien puede cambiar ese valor?** Lee el campo de autoridad. Si existe una autoridad, la respuesta de hoy a la pregunta dos es una foto, no una propiedad.

La pregunta tres es la razón por la que existe esta lección. El procesador de transfer hook de Token-2022 te dice exactamente por qué: `process_update` carga la extensión, saca `Option::<Address>::from(extension.authority)` y devuelve `NoAuthorityExists` cuando esa opción es `None`. Un slot de hook con autoridad nula es un slot muerto, para siempre. Un slot de hook con autoridad viva es un interruptor, y el de PYUSD está vivo: las ocho extensiones listan la misma autoridad, `2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk`. Una sola clave tiene todas las opciones del mint.

Eso te da tres veredictos en vez de dos, y puedes derivar los tres de datos que ya trajiste.

![Un flujo de decisión de tres preguntas convierte presencia de la extensión, valor del campo y autoridad en uno de tres veredictos, y ordena las ocho extensiones de PYUSD en cuatro activas y cuatro dormidas.](assets/v02-flowchart.png)

Dormido no es sinónimo de inofensivo. Quiere decir armado, y la diferencia entre armado y disparando es una firma de una clave que puedes nombrar.

### Quién está corto en la opción

Aquí está el encuadre que me lo hizo encajar, y es la razón por la que esta lección está en el módulo de economía y no en el de mecánica.

Una extensión armada es una opción, en el sentido financiero aburrido. El emisor tiene el derecho, no la obligación, de encender un comportamiento. Cargarla le cuesta casi nada: unos bytes extra de rent en la creación, y una cuenta de mint un poco más grande. Le paga opcionalidad. Y alguien está del otro lado de esa opción, porque las opciones no tienen un solo lado.

Eres tú. Todo el que tiene el token está corto en ella.

Recorre un número limpio de punta a punta. Digamos que tu tesorería mueve 10,000 PYUSD a un socio, y digamos que la autoridad de comisión había programado 50 bps con un máximo de 100 tokens. Tus 10,000 salen de tu cuenta y llegan 9,950, y los otros 50 quedan retenidos en la cuenta de token del destinatario hasta que el emisor haga harvest. Nada de tu instrucción de transferencia cambió. Nada de tu integración cambió. Tu contabilidad queda desviada 50 tokens por cada 10,000, para siempre, y si tu producto le cotizó al destinatario un monto exacto, tu producto ahora está mal. Esa es la opción ejercida, y estabas corto en ella supieras o no que la posición existía.

Dos de los slots dormidos de PYUSD tienen tiempos de ejercicio muy distintos, y eso importa más que el número de la comisión en sí:

- **La opción del hook se liquida al instante.** `TransferHookInstruction::Update` escribe `extension.program_id` en una sola instrucción, con la firma de la autoridad y nada más. La siguiente transferencia después de ese bloque hace CPI a un programa que hace un minuto no existía en tu modelo.
- **La opción de la comisión se liquida dos epochs después.** El procesador de comisión de transferencia escribe a propósito una tasa nueva por delante del epoch actual, con un comentario en la fuente que lo dice sin rodeos: puesta dos epochs adelante para evitar rug pulls. El mint lleva `olderTransferFee` y `newerTransferFee` con sellos de epoch precisamente para que un cambio se vea antes de que muerda. A 432,000 slots por epoch y con el tiempo de slot objetivo actual de 300ms (la segunda etapa de SIMD-0525, vigente desde el epoch 1024; 400ms y 350ms ya son historia), eso son unos tres días de aviso, y el tiempo de slot medido corre un poquito más lento, así que trátalo como un piso.

Esa asimetría es una decisión de diseño que tomaron los autores de Token-2022 y deberías sentirla. La extensión que se lleva tu dinero te da tres días. La extensión que puede rechazar tu transferencia de plano no te da ninguno.

![Una sola clave de autoridad se conecta a ocho slots de extensión en el mint de PYUSD, cuatro de ellos vivos y cuatro armados, con el hook ejercitable de inmediato y la comisión solo después de dos epochs.](assets/v03-diagram.png)

### Por qué armar un slot que nunca piensas disparar

Objeción justa, y es la que yo levantaría: si PYUSD nunca cobra una comisión y nunca fija un hook, ¿para qué cargarlas? Peso muerto, bytes extra, escrutinio extra de cada integrador que lee la lista y entra en pánico.

Por una restricción que cargas desde el módulo uno. Las extensiones son solo de tiempo de creación. No hay instrucción que atornille `transferHook` a un mint que se lanzó sin él. La alternativa a armar un slot el día uno no es "agrégalo después." La alternativa es: acuñar un token nuevo, migrar a cada tenedor, volver a listarte en cada plataforma y actualizar cada integración que dejó tu dirección hardcodeada.

Ahora ponle precio a los dos caminos con honestidad. Armar ocho slots en la creación cuesta algo de rent extra en una cuenta, para siempre, y una carga permanente de explicación con los integradores. Migrar un dólar regulado con cientos de millones de supply cuesta coordinación con cada exchange, custodio y billetera que lo tocó, más la cola de valor varado en contratos que nadie actualiza. No son del mismo orden de magnitud, y no están ni cerca.

![Una comparación de dos caminos muestra que armar extensiones en la creación del mint cuesta bytes extra y escrutinio de los integradores, mientras que la alternativa es una migración completa del token más adelante.](assets/v04-comparison.png)

Así que un emisor con forma de compliance arma todo lo que un regulador podría plausiblemente exigir y no dispara nada. Eso no es indecisión. Es la manera más barata de cumplir una promesa que todavía no puedes describir: si llega una regla que exige una comisión, un hook de allowlist o saldos privados con una auditor key, la respuesta es una instrucción y no una migración. El brief de tu capstone te va a poner la misma decisión a una escala más chica, y la versión honesta de eso es una oración en un memo: este slot está armado, esta clave lo tiene, esto es lo que nos haría usarlo.

El trade-off corta para el otro lado, eso sí. Un slot armado también es una promesa a tus integradores, y ellos la leen como riesgo. Algunas plataformas rechazan mints Token-2022 cuyo conjunto de extensiones no han modelado, y rechazar por presencia y no por valor es algo perfectamente racional para un programa de DEX cuando el valor puede cambiar debajo de él. Armar un hook para estar a salvo con los reguladores puede costarte el listado que necesitabas. Nómbralo también en el memo.

### JTO: tu ruta de Merkle, a escala de airdrop

Caso distinto, misma jugada: contrasta lo ya lanzado con lo que construiste.

La distribución de JTO de Jito corrió sobre un merkle distributor con vesting lineal. El programa está en `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv`, hoy está vivo y es ejecutable en mainnet bajo el loader actualizable, y expone `claim_locked` junto al reclamo simple: los destinatarios reclamaron de inmediato una porción desbloqueada mientras el resto se liberaba linealmente, en ese caso hasta el 2024-12-07.

Lee el código de tu propia lección de airdrop al lado de eso. Construiste un árbol de Merkle de destinatarios, publicaste una raíz, dejaste que cada reclamante presentara una prueba, marcaste la hoja como reclamada para que el segundo intento falle, y cableaste `claim_locked` para la porción con vesting del drop de compost, que es el mismo mecanismo en la misma interfaz de la misma familia de programas que Jito usó para distribuir un token de gobernanza vivo con una cola de vesting. La diferencia entre tu drop de Overgrowth y una de las mayores distribuciones de token en Solana es el tamaño del árbol y el valor de las hojas.

Esta es la parte del caso de estudio que de verdad quiero que te lleves: las primitivas no vienen por niveles de escala. No hay un mecanismo de airdrop "real" al que te gradúas. Hay una raíz de Merkle, una prueba, un marcador de reclamo y un reloj de vesting opcional, y la razón por la que la gente todavía hace mal los airdrops nunca es el mecanismo. Es la lista de hojas, el marcador de doble reclamo y la tokenómica que nadie publicó.

![Una tabla mapea cinco primitivas del curso a sus contrapartes ya lanzadas, incluidos el conjunto de extensiones de PYUSD, su slot de hook dormido, el merkle distributor de JTO y el campo is_agent de DAS.](assets/v05-table.png)

### Los rieles sobre los que va todo lo demás

Aléjate un nivel, porque un token solo es interesante si algo se mueve a través de él.

Al 2026-09-01 hay alrededor de $16.05B de stablecoins ancladas al dólar circulando en Solana, según la serie de stablecoins de DefiLlama, solo las ancladas al dólar, que es una cifra viva que se mueve a diario y que deberías volver a consultar en vez de citarla desde un curso. La metodología aquí importa tanto como el número: distintos rastreadores cuentan distintos anclajes y distintos wrappers, así que dos fuentes que difieren por unos cientos de millones es normal, y una cifra sin su fuente y su fecha no es un dato, es una vibra.

La dirección es más fácil de defender que cualquier número suelto. Stripe compró Bridge por $1.1B, con cierre en febrero de 2025 y alrededor de $1.5B de volumen total de pagos mensual en ese momento según el estudio de Helius sobre el panorama de las stablecoins, y SpaceX viene agregando ingresos de Starlink en stablecoins. Cuando una empresa de pagos paga mil millones de dólares por infraestructura de stablecoins en vez de construirla, eso es un mercado diciéndote que los rieles ya están elegidos.

![Una línea de tiempo va desde el lanzamiento de PYUSD en mayo de 2024, pasando por el fin del vesting de JTO y el cierre de Bridge por Stripe, hasta la frontera de la identidad de agentes en 2026 y la lectura en vivo del mint de hoy.](assets/v06-timeline.png)

Esa es la razón honesta por la que vale la pena hacer tu capstone. No que los tokens sean emocionantes. Que la plomería que has estado construyendo es la plomería que un procesador de pagos acaba de pagar. El curso Solana Payments and Commerce lee este mismo mint de PYUSD desde el lado de la integración, y los rieles de compliance que están por encima de estas primitivas, Token ACL entre ellos, deliberadamente no se enseñan aquí; son territorio del curso planificado DeFi and RWA Engineering.

### La frontera, fechada y con reservas

Una más, y esta viene con una etiqueta de advertencia pegada antes del contenido.

Hay trabajo real de 2026 sobre darles a los agentes autónomos una identidad on-chain. Metaplex tiene un Agent Registry, Core tiene un plugin `AgentIdentity`, y DAS expone `is_agent` como booleano nullable a nivel del activo. La plomería se está tendiendo: un agente obtiene un activo, el activo lleva un plugin de identidad, y un indexador puede responder "¿esta cosa es un agente?" en la misma lectura que responde "¿quién es su dueño?"

Ahora la advertencia. Esto es un tema de frontera, no un estándar de producción. Nada de tu capstone debería depender de él. Un booleano nullable en una interfaz de lectura es exactamente lo que parece un campo temprano: puede ser null porque la mayoría de los activos no tiene nada que decir, y null te dice que el campo existe y no te dice que un ecosistema se haya puesto de acuerdo en lo que quiere decir. Si hoy condicionas el acceso a `is_agent`, lo estás condicionando a un campo cuya semántica todavía puede cambiar debajo de ti, y esa es una clase de riesgo distinta de condicionarlo a la propiedad.

La razón por la que pertenece aquí es que es el mismo músculo de evaluación, un peldaño más temprano en el ciclo de vida. Acabas de pasar una sección decidiendo si una extensión ya lanzada hace algo. Decidir si un campo de frontera quiere decir algo es la misma lectura: quién lo escribe, qué dice hoy, y qué le pasa a tu producto si esa respuesta cambia. Síguelo, haz un spike si los agentes son tu producto, no lo pongas en la ruta crítica.

### Tres fuentes, tres confiabilidades

Lo que me trae a la disciplina de lectura que toda esta lección ha estado enseñando de reojo, y a un ejemplo que es casi demasiado bueno.

Solana misma, en su página de soluciones de Token Extensions, todavía dice que las transferencias confidenciales están "expected EOY 2024." Página viva, afirmación muerta, y las transferencias confidenciales ya llevan un rato en mainnet. Esa página no es inútil. Lleva una lista de cinco firmas de auditoría que revisaron el programa, que de verdad vale la pena tener y no se pudre. Cítala para la lista de auditoría. No la cites para una fecha.

Y una segunda arruga en la otra dirección: Token-2022 sigue siendo un programa actualizable. El HEAD del repo puede llevar una extensión o un arreglo que el despliegue de mainnet todavía no tiene. Así que el código que lees en GitHub es un techo, no una descripción de lo que va a ejecutarse en el próximo bloque.

![Una comparación de tres vías ordena la cuenta de mint en vivo, las páginas oficiales de documentación y el repositorio del programa según para qué se puede confiar en cada una y dónde falla cada una.](assets/v07-comparison.png)

### El trade-off, nombrado

Leer la dormancia en vivo es más trabajo que leer una lista de funciones, y vence. Tu informe es verdadero para el bloque en el que lo corriste, y una autoridad puede invalidar su línea del hook en el bloque siguiente sin aviso y sin anuncio. Ese es el costo honesto de este método: te da una respuesta correcta con poca vida útil, y te tienta a tratar una foto como una propiedad.

La mitigación no es leer más fuerte. Es anotar quién tiene cada opción y cuál es la latencia de ejercicio, y después decidir una sola vez si puedes vivir con el peor caso. Tres días de aviso sobre una comisión es algo que un proceso de tesorería puede absorber. Cero aviso sobre un hook es algo que tu integración o sobrevive por diseño o no.

## Lab: el informe de dormancia

Vas a construir `dormancy-report.ts`: apúntalo a cualquier mint y imprime veredictos por extensión derivados de valores y autoridades. Es la herramienta que responde el paso de verificación de tu capstone, y es la herramienta que yo querría en cualquier llamada de evaluación.

1. **Prepara todo.** Haz una carpeta y revisa tu Node. Necesitas Node 20 o más nuevo para el `fetch` global que usa este script; yo estoy en 23.9.

   ```bash
   mkdir -p labs/m09-l3 && cd labs/m09-l3 && node --version
   ```

   Sin npm install. Este script tiene cero dependencias a propósito, y el propósito es una decisión que vale la pena enunciar: una herramienta de evaluación que necesita un workspace es una herramienta que no vas a correr cuando un compañero de equipo pega una dirección de mint en el chat. `npx tsx@4.23.12` trae el runner de TypeScript por demanda. Ese pin es del 2026-08-22 y ya lo había pasado 4.23.13 cuando se volvió a revisar el 2026-09-01; revisa `npm view tsx version` cuando leas esto, porque saca versiones seguido.

2. **Escribe el clasificador.** Guarda esto como `dormancy-report.ts`. Las dos extensiones sobre las que gira el caso de estudio están trabajadas completas, las demás siguen la misma forma.

   ```typescript
   // dormancy-report.ts - classify every extension on a live mint as ACTIVE, DORMANT, INERT or REVIEW.
   // Zero npm dependencies. Run: npx tsx@4.23.12 dormancy-report.ts <MINT_ADDRESS>
   // Read-only: two RPC calls, nothing signed, nothing sent.

   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
   const TOKEN_2022 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
   const SLOT_SECONDS = 0.3; // 300ms target since SIMD-0525 stage 2 (epoch 1024); measured wall clock runs a little slower

   type Verdict = "ACTIVE" | "DORMANT" | "INERT" | "REVIEW";
   type State = Record<string, unknown>;

   interface Extension {
     extension: string;
     state?: State;
   }
   interface MintInfo {
     decimals: number;
     supply: string;
     extensions?: Extension[];
   }
   interface AccountValue {
     owner: string;
     space: number;
     data: { parsed: { info: MintInfo; type: string } };
   }
   interface EpochInfo {
     epoch: number;
     slotsInEpoch: number;
   }

   async function rpc<T>(method: string, params: unknown[]): Promise<T> {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const body = (await res.json()) as { result?: T; error?: unknown };
     if (body.error) throw new Error(`${method}: ${JSON.stringify(body.error)}`);
     if (body.result === undefined) throw new Error(`${method}: empty result`);
     return body.result;
   }

   function str(state: State | undefined, key: string): string | null {
     const v = state?.[key];
     return typeof v === "string" ? v : null;
   }

   function fee(state: State | undefined, key: string): { epoch: number; bps: number; max: string } {
     const f = (state?.[key] ?? {}) as Record<string, unknown>;
     return {
       epoch: Number(f.epoch ?? 0),
       bps: Number(f.transferFeeBasisPoints ?? 0),
       max: String(f.maximumFee ?? 0),
     };
   }

   // The authority is the option holder: the key that can CHANGE the config.
   // Different extensions spell that field differently, hence the fallbacks.
   // One conflation to know about before you reuse this on arbitrary mints:
   // "delegate" is in this list as a convenience, and a permanent delegate is
   // the key that HOLDS a power fixed at creation, not one that can change it.
   // On PYUSD every key happens to be the same party, so the line reads fine;
   // on a mint with split keys, print the delegate under its own label.
   function authorityOf(ext: Extension): string | null {
     const s = ext.state;
     return (
       str(s, "authority") ??
       str(s, "closeAuthority") ??
       str(s, "delegate") ??
       str(s, "transferFeeConfigAuthority") ??
       str(s, "updateAuthority")
     );
   }

   function classify(ext: Extension, epoch: number): { verdict: Verdict; reason: string } {
     const s = ext.state;
     switch (ext.extension) {
       case "transferHook": {
         const programId = str(s, "programId");
         if (programId) return { verdict: "ACTIVE", reason: `every transfer CPIs into ${programId}` };
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: "programId null; the hook authority can set one at any time" }
           : { verdict: "INERT", reason: "programId null and no authority: the slot can never fire" };
       }
       case "transferFeeConfig": {
         const newer = fee(s, "newerTransferFee");
         const older = fee(s, "olderTransferFee");
         const live = epoch >= newer.epoch ? newer : older;
         if (live.bps > 0) {
           return { verdict: "ACTIVE", reason: `${live.bps} bps withheld per transfer, max ${live.max}` };
         }
         const scheduled = newer.epoch > epoch ? ` (a ${newer.bps} bps fee lands at epoch ${newer.epoch})` : "";
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: `0 bps at epoch ${epoch}; the fee authority can schedule one${scheduled}` }
           : { verdict: "INERT", reason: "0 bps and no fee authority: the rate can never move" };
       }
       case "permanentDelegate": {
         const delegate = str(s, "delegate");
         return delegate
           ? { verdict: "ACTIVE", reason: `${delegate} can move or burn from any account, no owner signature` }
           : { verdict: "INERT", reason: "no delegate set" };
       }
       case "mintCloseAuthority": {
         const closeAuthority = str(s, "closeAuthority");
         return closeAuthority
           ? { verdict: "ACTIVE", reason: `${closeAuthority} can close this mint once supply hits 0` }
           : { verdict: "INERT", reason: "no close authority set" };
       }
       case "confidentialTransferMint": {
         const auto = s?.autoApproveNewAccounts === true;
         const auditor = str(s, "auditorElgamalPubkey");
         if (auto) {
           return {
             verdict: "ACTIVE",
             reason: `any account can self-configure; auditor ${auditor ?? "none"}`,
           };
         }
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: "autoApproveNewAccounts false: every account needs issuer approval first" }
           : { verdict: "INERT", reason: "no auto-approval and no authority to grant it" };
       }
       case "confidentialTransferFeeConfig": {
         const withheld = str(s, "withheldAmount") ?? "";
         // Base64 of all-zero bytes is a run of "A" characters (every 6-bit
         // group of zeros encodes as "A"), possibly "="-padded; that is what
         // the regex matches. Deliberate simplification alongside it: this
         // branch judges only the withheld pile and never consults an
         // authority, so it can return DORMANT or ACTIVE but never INERT.
         // The three-question framework's Q3 is skipped here because the
         // extension's arming is decided by the confidential pair around it.
         const empty = /^A*=*$/.test(withheld);
         return empty
           ? { verdict: "DORMANT", reason: "withheld ciphertext is all zeros: nothing collected yet" }
           : { verdict: "ACTIVE", reason: "confidential fees are being withheld" };
       }
       case "metadataPointer": {
         const target = str(s, "metadataAddress");
         return target
           ? { verdict: "ACTIVE", reason: `metadata resolves at ${target}` }
           : { verdict: "INERT", reason: "pointer set to nothing" };
       }
       case "tokenMetadata": {
         const name = str(s, "name") ?? "?";
         const symbol = str(s, "symbol") ?? "?";
         return { verdict: "ACTIVE", reason: `on-mint metadata: ${name} (${symbol})` };
       }
       default:
         return { verdict: "REVIEW", reason: "no rule written for this extension yet: read the state by hand" };
     }
   }

   async function main() {
     const mint = process.argv[2];
     if (!mint) throw new Error("usage: npx tsx@4.23.12 dormancy-report.ts <MINT_ADDRESS>");

     const epochInfo = await rpc<EpochInfo>("getEpochInfo", []);
     const account = await rpc<{ value: AccountValue | null }>("getAccountInfo", [
       mint,
       { encoding: "jsonParsed" },
     ]);
     if (!account.value) throw new Error(`no account at ${mint}`);

     const { owner, space, data } = account.value;
     const info = data.parsed.info;
     const extensions = info.extensions ?? [];

     console.log(`mint:      ${mint}`);
     console.log(`program:   ${owner}${owner === TOKEN_2022 ? " (Token-2022)" : " (not Token-2022)"}`);
     console.log(`size:      ${space} bytes, ${info.decimals} decimals`);
     console.log(`epoch:     ${epochInfo.epoch}`);
     console.log(`extensions: ${extensions.length}\n`);

     const tally: Record<Verdict, number> = { ACTIVE: 0, DORMANT: 0, INERT: 0, REVIEW: 0 };
     for (const ext of extensions) {
       const { verdict, reason } = classify(ext, epochInfo.epoch);
       tally[verdict] += 1;
       const holder = authorityOf(ext) ?? "none";
       console.log(`${verdict.padEnd(8)} ${ext.extension}`);
       console.log(`         why: ${reason}`);
       console.log(`         authority: ${holder}`);
     }

     const days = (epochInfo.slotsInEpoch * 2 * SLOT_SECONDS) / 86_400;
     console.log(
       `\nverdict: ${tally.ACTIVE} active, ${tally.DORMANT} dormant, ${tally.INERT} inert, ${tally.REVIEW} unreviewed`,
     );
     console.log(
       `a hook flip lands immediately; a fee change lands two epochs out, about ${days.toFixed(1)} days at ${SLOT_SECONDS}s slots`,
     );
   }

   main().catch((e: unknown) => {
     console.error(e instanceof Error ? e.message : e);
     process.exit(1);
   });
   ```

3. **Córrelo contra PYUSD.**

   ```bash
   npx tsx@4.23.12 dormancy-report.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
   ```

   La cola de mi corrida del 2026-09-01:

   ```text
   verdict: 4 active, 4 dormant, 0 inert, 0 unreviewed
   a hook flip lands immediately; a fee change lands two epochs out, about 3.0 days at 0.3s slots
   ```

   Arriba de eso te salen ocho bloques, cada uno con su veredicto, el campo del que salió el veredicto y la autoridad que lo tiene. Cada línea de autoridad lee `2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk`. Una clave, ocho slots, cuatro vivos hoy y cuatro armados y esperando. (Armado es la palabra de esta lección para los cuatro DORMANT: configurados, quietos y a una firma de dispararse.)

   Si tu corrida muestra un conteo distinto, no asumas que la lección tiene razón y tu terminal está equivocada. Esta es una cuenta viva y los emisores cambian cosas. Lee la línea `why:` de la extensión que se movió. Ese reflejo es todo el curso, honestamente.

4. **Córrelo contra un mint que no tiene nada que decir.** El USDC clásico es el caso de control.

   ```bash
   npx tsx@4.23.12 dormancy-report.ts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
   ```

   Te debería salir `(not Token-2022)`, 82 bytes y cero extensiones. Un mint clásico no tiene slots que armar, lo cual es una propiedad real y a veces exactamente la que quieres. Nada que leer también es información.

5. **Apúntalo a tu propio mint de SPROUT.** El RPC es configurable, así que diríjelo al endpoint donde viva tu mint de Overgrowth:

   ```bash
   RPC_URL=https://api.devnet.solana.com npx tsx@4.23.12 dormancy-report.ts <YOUR_SPROUT_MINT>
   ```

   Tu comisión de transferencia debería volver como ACTIVE con los bps que fijaste, porque tú de verdad usas la tuya. Esa diferencia de una sola línea entre tu mint y el de PayPal es toda la distinción entre poner una comisión en producción y reservarla.

6. **Paso de completar: escribe tú mismo una regla del clasificador.** La rama `default` devuelve `REVIEW`, que es la respuesta honesta para una extensión en la que no has pensado. Elige una que hayas construido antes en el curso, `pausable` o `defaultAccountState` o `scaledUiAmount`, y agrégale un `case`. La regla tiene que responder las mismas tres preguntas: qué campo carga el comportamiento, qué valor lo vuelve inerte y qué autoridad puede moverlo. Si no puedes nombrar el campo de autoridad, todavía no tienes una regla, tienes una conjetura.

7. **Checkpoint.** Corre los tres mints seguidos. Deberías poder apuntar a cualquier línea de la salida de PYUSD y decir de qué campo de RPC salió, sin abrir el script.

## Challenge

Solo, y esta es la pieza en la que te exiges un estándar de aprobado/reprobado — nada la recoge, que es exactamente por qué escribirla con honestidad es el ejercicio. Escribe el memo de dormancia.

Corre `dormancy-report.ts` contra el mint en vivo de PYUSD y escribe cinco oraciones sobre las que un colega podría actuar, una por cada punto de abajo. Cuáles de sus ocho extensiones TLV están activas y cuáles están configuradas pero dormidas, con el program id del hook y los valores de comisión de transferencia que de verdad leíste. Quién tiene las opciones, por dirección. Cuál es la latencia de ejercicio del hook frente a la de la comisión, y por qué difieren. Una oración sobre qué monitorearías si tu producto liquidara en este token. Y una oración que nombre lo que tu informe no te puede decir.

Dalo por aceptado cuando el memo derive sus veredictos de valores y no de la presencia, cite números que imprimió tu propia corrida, se feche a sí mismo y nombre la dirección de la autoridad. Dalo por rechazado si dice que PYUSD tiene transferencias confidenciales, así que los saldos de PYUSD son privados. Los rieles están configurados, el torniquete está trabado, y la diferencia es la lección.

Segunda pasada opcional si los agentes están cerca de tu roadmap: agrega dos oraciones sobre `is_agent` y el Metaplex Agent Registry que sobrevivirían a un lector escéptico en 2027. Pista: contienen la palabra "todavía."

## Checkpoint

El criterio es un informe que corriste más un memo que enviarías. Si no puedes producir el memo sin volver a correr el script, está bien, para eso es el script.

La versión de una oración, terminal cerrada: que una extensión esté presente no te dice nada, los valores de sus campos te dicen qué pasa hoy, y su autoridad te dice quién puede cambiar eso, así que toda evaluación real tiene tres lecturas de profundidad y es verdadera solo para el bloque en el que la corriste.

Las fallas que espero, en el orden en que las espero. Primero, la trampa de la presencia, que es la que yo mismo pisé: leer `transferFeeConfig` como "este token cobra comisiones" sin abrir el cuerpo. Segundo, la trampa de la seguridad, que es peor: leer 0 bps como una propiedad del token y no como una foto con una clave nombrada detrás. Tercero, la trampa de la frontera: escribir sobre identidad de agentes como si el registry y el plugin y el campo `is_agent` sumaran un estándar. Suman una dirección. Di dirección.

Y con eso el arco de la economía queda cerrado. Construiste un mint con extensiones reales, un hook que corre en cada transferencia, una ruta de comisión y un buyback, una colección de cNFT, un lector, un control de acceso, un airdrop con vesting, y ahora el juicio para leer la versión que otro hizo de todo eso y decir qué hace en realidad.

Construiste todos los peldaños. El último módulo te entrega un brief de producto, cinco entre los que elegir, el quinto un espacio en blanco que puedes llenar con tu propio producto: elige la primitiva, entrégala, conecta un riel y demuestra que resuelve. Nadie te dice cuál primitiva esta vez, y el memo de dormancia que escribiste trabaja el mismo músculo que el memo de selección del capstone: juicio, dicho como oraciones fechadas sobre las que un colega puede actuar.
