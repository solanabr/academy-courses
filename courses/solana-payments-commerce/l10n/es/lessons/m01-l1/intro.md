# Mira cómo se liquida un dólar: tus primeros cinco minutos en los rieles de Solana

## Resumen

Esta es la primera lección, así que no hay nada que recapitular: todavía no se ha construido nada. Llegas con fluidez en rieles de tarjeta y en integración de PSP, el tipo de ingeniero que sabe lo que se siente una tormenta de reintentos de webhooks, y vamos a abrir leyendo dinero real moverse on-chain antes de cualquier teoría. En los próximos cinco minutos, sin billetera, sin claves y sin registro, vas a leer cómo se liquida en el libro mayor público de Solana un pago real denominado en dólares. Después, en una sentada más larga, vas a generar un código QR de pago que tu propio teléfono puede escanear. Nada de diapositivas primero.

## Los rieles que puedes leer

Copia esto en tu terminal y córrelo. Funciona en cualquier máquina que tenga `curl`, es decir, la tuya:

```bash
curl -s -X POST https://api.mainnet-beta.solana.com \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getTransaction","params":["3qE5iCo5uGGfGXZd4rwfgJryLse6EpTA2SQMtX3X3GbWsS6hDRw8JwcYb4wpj77Aqj5mQBJg52LawnaZyT688kXA",{"encoding":"jsonParsed","maxSupportedTransactionVersion":1}]}'
```

Ese muro de JSON que acabas de recibir es el pago liquidado de un desconocido. Uno real. Alguien, en algún lugar, movió USDC en mainnet de Solana, se liquidó, y acabas de extraer el registro completo de un endpoint público con una petición HTTP de una sola línea. Nadie preguntó quién eres. Nadie podría.

Quédate con eso un segundo, porque tu cerebro de Stripe debería estar picando. En el mundo de las tarjetas, un pago liquidado es una fila en la base de datos de un procesador. Ves TUS filas, a través de TU panel, después de que TU clave de API te autentique. La idea de que podrías traer el registro de liquidación de un pago entre dos desconocidos totales no es un bug de permisos que alguien olvidó cerrar. Aquí, es el diseño.

Un puñado de definiciones para que el JSON deje de ser ruido, y después nos ganamos las afirmaciones grandes. Estas nombran los campos en los que la lección de verdad se apoya; el resto de esa respuesta (`slot`, `computeUnitsConsumed`, `innerInstructions`, `logMessages` y compañía) se queda como ruido por ahora, a propósito, y el lab abre con un mapa de campos que muestra exactamente dónde vive cada valor impreso dentro del árbol, así que no los busques todavía.

- **mainnet**: la red Solana en vivo (la red principal), donde viven los saldos reales. También hay una red de pruebas gratuita llamada devnet, que es donde va a caer cada pago que envíes en este curso. Hoy solo estás leyendo mainnet, nunca escribiendo en ella.
- **endpoint RPC**: la URL que acabas de golpear es una puerta pública al libro mayor. Cualquiera puede llamar. Y algo crucial: la puerta solo se abre en un sentido para peticiones como esta: la demo lee, nunca escribe. No puedes mover, revertir ni volver a disparar los fondos de nadie por traer su transacción, como tampoco leer un comprobante gasta el dinero que figura en él.
- **USDC**: una stablecoin anclada al dólar emitida por una empresa regulada, Circle, que mantiene reservas contra cada token en circulación y los redime uno a uno. Esa es toda la razón por la que una cifra en dólares significa algo en estos rieles: 1 USDC es un derecho sobre 1 dólar estadounidense ante el emisor, y el mercado lo cotiza en consecuencia. Cuando este curso dice "se liquidó un dólar", quiere decir que se movió un token USDC. La paridad es una promesa del emisor, no una ley de la física, y ponerle precio a un negocio sobre ella es una decisión real que vamos a tomar explícitamente en la frontera fiat.
- **dirección del mint de USDC**: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` es el identificador on-chain de ese token en Solana, la cuenta que define la moneda en sí. Piénsalo como el código de moneda, solo que es una dirección globalmente única, no una cadena de tres letras. USDC en Solana lleva 6 decimales, así que el monto entero en crudo `36115` en ese JSON quiere decir 0.036115 USDC. Esas cadenas largas son **base58**, el alfabeto en el que Solana imprime direcciones y firmas: dígitos y letras menos las que los humanos confunden, así que no hay `0`, `O`, `I` ni `l`.
- **`spl-token` y `transferChecked`**: `spl-token` es el programa on-chain que es dueño de este saldo de USDC, y de la mayoría de los saldos de tokens en Solana, tal como un servicio de libro mayor es dueño de las filas de saldo. La mayoría, no todos: un segundo programa de tokens llamado Token-2022 es dueño de sus propios saldos (PYUSD vive ahí, por ejemplo), y el módulo 2 te enseña a comprobar a qué programa pertenece un mint antes de decodificar nada. `transferChecked` es la única instrucción de ese programa que este curso usa para mover tokens; la parte "checked" quiere decir que quien llama tiene que declarar además el mint y su cuenta de decimales, y el programa se niega si no coinciden. En el JSON, su campo `authority` es la cuenta que autorizó el movimiento, que para un pago ordinario es la propia billetera del remitente. Los saldos en sí viven en **cuentas de token**, una por dueño por moneda; el próximo módulo las construye como se debe.
- **`spl-memo` y el memo**: `spl-memo` es un segundo programa, diminuto, cuyo único trabajo es adjuntar una nota legible por humanos a una transacción. La nota en tu JSON dice `079cea64791142a59e12a3491a425f90`, la referencia interna de algún sistema. Más adelante en este curso, los memos se vuelven la forma en que un comercio empareja un pago con un pedido. Guárdate eso.

![Una transacción liquidada que contiene una instrucción transferChecked de spl-token moviendo 0.036115 USDC y una instrucción spl-memo que lleva la nota de referencia, empaquetadas bajo una sola firma.](assets/v01-diagram.webp)

Así que la transacción que trajiste se descompone en esto: el remitente `BhFRCUXHVm76PmXkSzus8T4LUGrD2MTW9Au6bocBox5U` movió 0.036115 USDC, con ese memo adjunto, y se liquidó el 2026-08-22. Eso son 3.6 centavos, y el tamaño es justamente el punto, no una vergüenza: en rieles de tarjeta una transferencia de 3.6 centavos no es un pago pequeño, es un pago imposible, porque el piso de comisiones supera el monto. Aquí alguien lo movió y la economía todavía funcionó. Le ponemos un número exacto a las comisiones en la próxima lección; por ahora el punto es que el registro es público, completo y tuyo para leer.

Fíjate, además, en lo que NO está en ese registro, porque las ausencias son tan instructivas como los campos. No hay número de tarjeta, así que no hay nada con forma de PCI que guardar en una bóveda. No hay CVV, ni vencimiento, ni dirección de facturación, ni un campo que diga "la ventana de chargeback cierra en 120 días". La dirección del remitente identifica una clave, no una persona, que es la razón por la que "quién me pagó de verdad" se vuelve una pregunta distinta en estos rieles de lo que era en las tarjetas, y emparejar pagos con clientes se va a apoyar en ese campo memo antes que en cualquier cosa parecida al nombre del titular de la tarjeta. Por ahora, solo registra la forma: todo lo que necesita la liquidación está presente, y no está ahí en absoluto nada de lo que necesita la responsabilidad por fraude con tarjetas.

![Una terminal envía una petición HTTPS sin autenticar a un endpoint RPC público, que lee una transacción liquidada del libro mayor compartido y devuelve el registro JSON; el camino es de solo lectura.](assets/v02-diagram.webp)

### La inversión que tu PSP nunca ofreció

Esta es la forma de lo que integras hoy. Un pago con tarjeta viaja a través de un adquirente, una red y un emisor, y el artefacto que recibes al final es un webhook más una fila que puedes consultar, acotada a tu cuenta de comercio. La verdad de la liquidación vive dentro del procesador. Cuando finanzas pregunta "¿el pedido 4412 de verdad se pagó?", la respuesta es lo que diga el panel, y el panel es infraestructura privada que alquilas.

Solana invierte eso. La verdad de la liquidación vive en un libro mayor público y compartido, y la cosa con forma de procesador que está en el medio es opcional. Cualquier parte de un pago, o cualquier tercero curioso, puede verificar la liquidación directamente, sobre cualquier endpoint RPC, para siempre. Tu futuro código de conciliación en este curso no le va a pedir a un proveedor "por favor dime si me pagaron". Va a leer el libro mayor mismo.

![Comparación lado a lado que muestra los registros de liquidación de tarjeta como filas privadas de un procesador detrás de una clave de API, frente a los registros de liquidación de Solana como entradas públicas de un libro mayor que cualquiera puede leer sobre cualquier RPC.](assets/v03-comparison.webp)

Lo público por defecto es lo que cambia el juego aquí, y quiero ser preciso sobre por qué. No es que lo público sea virtuoso. Es que lo público más lo legible por máquina colapsa categorías enteras de trabajo de integración que hoy haces: APIs de conciliación, exportaciones de reportes de liquidación, "contacta a soporte para rastrear este pago". El libro mayor es el reporte.

Hazlo concreto con el memo que acabas de leer. Más adelante en este curso, cuando Wavelength Records venda un prensado, el checkout estampa el id del pedido en el memo de la transferencia, exactamente como la referencia `079cea...` en tu JSON. Cuando finanzas pregunta "¿el pedido 4412 de verdad se pagó?", la respuesta no va a ser una llamada de API a un proveedor que a lo mejor está teniendo un incidente. Va a ser una consulta contra el mismo libro mayor público que golpeaste hace dos minutos, emparejando memo con pedido, y cualquiera que dude de la respuesta puede correr la misma consulta por su cuenta. Ese es todo el módulo de conciliación en una sola oración; vamos a gastar lecciones de verdad ganándonoslo como se debe.

### El repo que cambió de bando

Segunda demo, y esta viene con un giro de guion. La biblioteca oficial de pagos de Solana se publica en npm como `@solana/pay`. Instálala y algo inesperado aterriza al lado. El paso 5 del lab vuelve a correr esta instalación con dos paquetes más al lado, así que corre esto ahora para la demo y corre igual el comando más completo del paso 5 cuando llegues ahí:

```bash
npm i @solana/pay@1.0.26
npx pay --help
```

Un número de versión, dicho una vez y usado en todo este curso: el paquete de npm es `@solana/pay` **1.0.26** (el último en npm, comprobado el 2026-08-23; vuelve a comprobar con `npm view @solana/pay version`, porque este paquete se mueve). Junto con la biblioteca de TypeScript por la que presumiblemente viniste, el paquete deja un ejecutable `pay` en `node_modules/.bin`, que es lo que `npx pay` encuentra. Córrelo y la primera vez descarga un binario nativo para tu plataforma, y luego te saluda con un toolchain para, y cito su propio banner, "agentic payments": comandos para poner APIs detrás de una barrera de pago en stablecoins, envolver `curl` y agentes de código con IA para que puedan pagar por lo que traen, administrar cuentas, enviar stablecoins desde la línea de comandos.

Espera que el binario reporte un número distinto al del paquete. `npx pay --version` imprime la propia línea de release de la CLI, que era 0.26.0 en mi máquina al escribir (el `npx` no es opcional aquí: este curso instala localmente, así que el ejecutable vive en `node_modules/.bin` y un `pay` pelado es command-not-found hasta que el módulo 7 lo instala globalmente); en el repo, el tag de CLI más nuevo era pay-v0.28.0, cortado el 2026-08-26 (verificado contra la API de releases de GitHub el 2026-09-07); se mueve alrededor de una vez al mes, así que verifica en vez de suponer. Dos líneas de versión en un mismo paquete confunden la primera vez que te las topas y son completamente normales después: 1.0.26 es la biblioteca que instalas, y el número 0.2x es el ejecutable descargado que viene al lado. Ninguno está mal cuando no coinciden.

La historia detrás de ese binario vale sesenta segundos, porque es todo el ecosistema en miniatura. Por años, el repositorio canónico de Solana Pay vivió en solana-labs/solana-pay y su producto estrella era el checkout con código QR: el cliente escanea, la billetera paga, listo. Hoy esa URL de GitHub redirige a un repo de la Solana Foundation llamado simplemente "pay", y el producto de portada es la CLI de pagos agénticos que acabas de picar. La energía de pagos de la Foundation se movió visiblemente de "humanos escaneando códigos QR" hacia "agentes de software pagando sobre HTTP".

Lee el movimiento con precisión, porque importa: no se borró nada. El Solana Pay clásico, la biblioteca de checkout con código QR, sobrevive como un subpaquete dentro de ese mismo monorepo, mantenida y publicada, y este curso construye un checkout de verdad sobre ella en el módulo 3, el módulo de superficies de pago. Un cambio de portada no es un producto removido. Aprender a leer los movimientos del ecosistema a esa resolución, qué cambió de verdad frente a qué simplemente dejó de ser la cara visible, es una habilidad de supervivencia en rieles tan jóvenes, y vas a tener práctica de sobra.

![Línea de tiempo que muestra el repo canónico de Solana Pay redirigiendo al repo pay de la Foundation, cuyo producto de portada es una CLI de pagos agénticos, mientras la biblioteca clásica de checkout con código QR continúa como subpaquete de principio a fin.](assets/v04-timeline.webp)

### Esto es dinero real, no una economía de demo

Pregunta justa a esta altura: ¿algo de esto es producción, o es un sandbox con buen marketing? Te voy a dar dos datos y los dejo discutir por mí.

Primero, el actor establecido: el "Pay with crypto" de Stripe acepta USDC en Solana en el checkout y liquida al comercio en fiat. Lee eso otra vez desde tu silla de integrador. El PSP más masivo de tu mundo trata estos rieles como un método de pago de primera clase, y absorbe la parte cripto para que el comercio nunca sostenga un token. Cualesquiera sean tus prejuicios sobre los pagos cripto, el equipo de riesgo de Stripe aprobó este.

Segundo, la trayectoria. La oferta de stablecoins en Solana, es decir dólares tokenizados y sentados en estos rieles, pasó de alrededor de $1.5B en diciembre de 2023 a $11.7B para febrero de 2025 (según un artículo de investigación de Helius), y está en unos $15.87B al 2026-08-23 (DefiLlama, contando solo stablecoins ancladas al dólar; este número se mueve a diario, así que trata cualquier cifra que leas, incluida esta, como una foto con fecha). La propia documentación de pagos de Solana dice que la red procesó más de $1 billón en volumen de stablecoins en 2025. La oferta es el float; el volumen es el throughput; las dos curvas apuntan al mismo lado. El dinero se fue a donde la liquidación era barata y rápida, tal como el agua encuentra el desagüe. El float sentado en estos rieles creció más de diez veces en menos de tres años, y los actores establecidos lo siguieron hacia adentro en vez de esperar a que pasara.

![Gráfico de la oferta de stablecoins en Solana subiendo de 1.5 mil millones de dólares en diciembre de 2023 a 11.7 mil millones en febrero de 2025 y unos 15.87 mil millones en agosto de 2026.](assets/v05-chart.webp)

### Estos rieles se mueven debajo de ti

Ahora la contrapartida, porque siempre hay una, y este curso la va a nombrar cada vez. El mismo giro de repo que hace emocionante la demo te está diciendo que el piso no está quieto. Las versiones rotan. El tooling rota. Hasta las URLs canónicas rotan: el repo que habrías marcado hace dos años ahora redirige a otro lado. "Funciona hoy" es una afirmación con fecha en estos rieles, no una garantía, y un curso que pretendiera lo contrario te estaría mintiendo con amabilidad.

Así que estas son las reglas de la casa para todo lo que sigue, dichas una vez y aplicadas de principio a fin:

- **Cada pin de versión lleva una nota de vigencia.** Cuando digo `@solana/pay` 1.0.26, te digo cuándo eso era cierto y cómo volver a comprobarlo. Las versiones fijadas en los scripts son deliberadas, porque una demo sin fijar puede romperse en silencio cuando una dependencia transitiva publica un cambio incompatible.
- **Las afirmaciones con fecha llevan fecha.** Cifras de oferta, comportamiento de productos, distribución de repos: cada una llega con su fecha de corte y su fuente. Cuando te topes con un desajuste, y algún día te vas a topar, la fecha te dice si el curso está viejo o si lo está tu entorno.
- **La infraestructura en vivo lleva un fallback.** El RPC público de mainnet que usaste arriba te va a limitar la tasa si escaneas agresivamente, lo cual es justo, es gratis. Por eso exactamente esta lección fija una firma de transferencia real y re-verificada: la demo de los primeros cinco minutos decodifica esa transacción incluso cuando el escaneo en vivo queda estrangulado. Vas a ver el patrón en el script del lab.

### A dónde va este curso, y la tienda que estamos construyendo

Ya tocaste los dos extremos del arco: leíste un pago liquidado y conociste la herramienta que deja al software pagar por llamadas HTTP. El curso entre esos dos puntos abre en el módulo 2 construyendo el kit de transferencias en sí, las cuentas de token, los decimales y las primitivas de enviar-y-verificar que cada lección posterior importa, y después corre en seis movimientos. Módulo 3, superficies de pago: checkout con código QR, enlaces de pago, las rutas de la tienda que un cliente humano de verdad toca. Módulo 4, operaciones del comercio: política de confirmación, webhooks, conciliación contra el libro mayor público que acabas de leer, y reembolsos, que en estos rieles son un pago nuevo en la dirección contraria antes que una reversión. Módulo 5, ingresos recurrentes: suscripciones en rieles que no tienen tarjeta en archivo. Módulo 6, la frontera fiat: on-ramps, off-ramps, precios de exhibición y los puentes con forma de Stripe entre estos rieles y tu contabilidad. Módulo 7, pagos entre máquinas: el lado agéntico que vislumbraste en la CLI `pay`, donde el cliente es software. Módulo 8, endurecimiento para producción: checkout gasless, ventas offline, hacer que las transacciones aterricen cuando la red está ocupada. El módulo 9 es el capstone, donde cada pieza corre como una sola tienda.

Construimos todo eso para un solo comercio. Te presento a **Wavelength Records**, una tienda independiente de vinilos que existe solo en este curso: tienda en línea, una terminal POS de puesto de mercado, un club de suscripción al disco del mes y, con el tiempo, una API que cotiza precios de prensado a otras empresas. Cada lección le agrega una pieza real al stack de Wavelength, y para el final vas a haber construido una operación de comercio de punta a punta sobre rieles de Solana, no una pila de fragmentos desconectados. Cuando generes un código QR en el lab de hoy, va a estar denominado como un pedido de Wavelength, porque es uno.

![Mapa del curso en seis etapas, desde las superficies de pago pasando por operaciones, ingresos recurrentes, la frontera fiat, los pagos entre máquinas y el endurecimiento para producción, todo anclado a construir la tienda de Wavelength Records de punta a punta.](assets/v06-flowchart.webp)

Una pieza más de orientación antes del lab, sobre cómo las lecciones de este curso te pasan el trabajo. Cada lección atenúa la autonomía en tres pasos, en voz alta: la visión general que acabas de terminar se lee sin tocar un teclado; el lab lo hacemos juntos, paso a paso, con la salida esperada en cada Checkpoint; el Challenge del final lo haces solo, y es la parte que hace que la lección se quede. Hoy el lab es deliberadamente suave, y fíjate en lo que no hace: nada de billetera hasta el final. Ya leíste mainnet sin una, que es justamente el punto. La billetera llega solo cuando has construido algo que valga la pena escanear.

## Lab: a los rieles, de punta a punta

Los pasos 1 a 4 son los cinco minutos prometidos arriba: revisión del toolchain, carpeta, script, correr. Los pasos 5 a 8 son una sentada más larga, porque incluyen una instalación de npm, la descarga de un binario en la primera corrida, instalar una billetera desde una tienda de apps y escribir una frase de recuperación en papel. Presupuesta cuarenta minutos para todo y no dejes que la promesa de la apertura te apure a través del ritual de la billetera.

Vamos a decodificar una transferencia como se debe con un script, generar un código QR de pago de Wavelength y solo entonces configurar una billetera y escanear nuestro propio código QR con ella. Antes del script, un mapa: el JSON crudo que devolvió tu curl anida los campos interesantes unos niveles más abajo, y el decodificador que estás a punto de escribir no es más que una caminata hasta esos puntos. Aquí está dónde vive cada campo impreso.

![Un árbol JSON abreviado de getTransaction con flechas de llamada que mapean blockTime a settledAt, la instrucción transferChecked de spl-token a sender, amount y mint, y la instrucción spl-memo a la cadena del memo.](assets/v07-annotated-code.webp)

1. **Revisa tu toolchain.** Necesitas Node 24 o más nuevo, que es lo que cada lab de este curso asume de aquí en adelante. `node -v` debería imprimir v24.x o más alto; si imprime algo más viejo, actualiza ahora en vez de en la primera instalación que se niegue. Los scripts de abajo corren vía `tsx`, un runner de TypeScript sin configuración; lo invocamos a través de `npx` con una versión fijada (`tsx@4.23.12`, el último en npm al 2026-08-23), así que `tsx` mismo no necesita instalación.

2. **Crea una carpeta de trabajo y un manifiesto.** Este se vuelve el espacio de borrador de Wavelength, y cada comando de aquí al final de la lección corre dentro de él:

   ```bash
   mkdir wavelength-rails && cd wavelength-rails
   npm init -y
   ```

   Resultado esperado: un directorio que contiene exactamente un archivo, un `package.json` que npm generó con valores por defecto. Se ve escueto y eso es correcto; el paso 5 llena las dependencias. Lo creamos explícitamente en vez de dejar que `npm install` conjure uno después, para que nada de tu carpeta sea una sorpresa.

3. **Guarda el decodificador.** Crea `watch-a-dollar.ts` con exactamente este contenido:

   ```ts
   // watch-a-dollar.ts: read a settled USDC transfer straight off Solana mainnet.
   // No wallet, no keys, no signup. Run: npx -y tsx@4.23.12 watch-a-dollar.ts

   const RPC = "https://api.mainnet-beta.solana.com";
   const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC on Solana, 6 decimals

   // A real USDC transfer, pinned as a fallback in case the public RPC rate-limits
   // the live scan. Re-verified on mainnet 2026-08-23.
   const FALLBACK_SIG =
     "3qE5iCo5uGGfGXZd4rwfgJryLse6EpTA2SQMtX3X3GbWsS6hDRw8JwcYb4wpj77Aqj5mQBJg52LawnaZyT688kXA";

   async function rpc(method: string, params: unknown[]) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const json = await res.json();
     if (json.error) throw new Error(json.error.message);
     return json.result;
   }

   async function decode(signature: string) {
     const tx = await rpc("getTransaction", [
       signature,
       { encoding: "jsonParsed", maxSupportedTransactionVersion: 1 },
     ]);
     if (!tx) return null;
     const instructions = tx.transaction.message.instructions;
     // Deliberately narrow: USDC is an spl-token mint. A Token-2022 mint
     // (PYUSD, say) reports a different program here and this filter would
     // skip it -- module 2 builds the detector that handles both.
     const transfer = instructions.find(
       (ix: any) =>
         ix.program === "spl-token" &&
         ix.parsed?.type === "transferChecked" &&
         ix.parsed.info.mint === USDC_MINT,
     );
     if (!transfer) return null;
     const memoIx = instructions.find((ix: any) => ix.program === "spl-memo");
     return {
       signature,
       settledAt: new Date(tx.blockTime * 1000).toISOString(),
       sender: transfer.parsed.info.authority,
       amountUSDC: transfer.parsed.info.tokenAmount.uiAmountString,
       mint: transfer.parsed.info.mint,
       memo: memoIx ? memoIx.parsed : "(none attached)",
     };
   }

   async function main() {
     // Live scan: recent transactions that touched the USDC mint account.
     try {
       const sigs = await rpc("getSignaturesForAddress", [USDC_MINT, { limit: 20 }]);
       for (const s of sigs.filter((s: any) => s.err === null)) {
         const result = await decode(s.signature);
         if (result) {
           console.log("LIVE: a stranger's payment, settled moments ago.");
           console.log(result);
           return;
         }
       }
     } catch {
       console.log("Live scan rate-limited. Falling back to the pinned transfer.");
     }
     console.log("PINNED: a real settled transfer (verified 2026-08-23).");
     console.log(await decode(FALLBACK_SIG));
   }

   main();
   ```

   Resultado esperado: un archivo, `watch-a-dollar.ts`, sentado en `wavelength-rails`. Todavía no ha corrido nada.

4. **Córrelo.**

   ```bash
   npx -y tsx@4.23.12 watch-a-dollar.ts
   ```

   Salida esperada: o un bloque `LIVE` con una transferencia de USDC que se liquidó hace segundos, o el bloque `PINNED` mostrando el remitente `BhFRCUXHVm76PmXkSzus8T4LUGrD2MTW9Au6bocBox5U`, el monto `0.036115` USDC, el mint de USDC y el memo `079cea64791142a59e12a3491a425f90`. Los dos son igual de reales; el fijado solo tiene garantizado estar ahí. Cuando corrí el camino en vivo mientras escribía esto, atrapó una transferencia de 0.10 USDC que se había liquidado veinte segundos antes, lo cual nunca deja de ser un poco surrealista. **Checkpoint: tienes un objeto decodificado con signature, settledAt, sender, amountUSDC, mint y memo impresos en tu terminal.** Si ves `Live scan rate-limited` primero, eso es el RPC público gratuito haciendo exactamente lo que la sección de teoría advirtió, y el fallback es la lección funcionando según el diseño, no fallando.

5. **Instala la biblioteca de pagos y conoce al polizón.** Todavía en `wavelength-rails`:

   ```bash
   npm i @solana/pay@1.0.26 @solana/kit@6.10.0 qrcode-terminal@0.12.0
   npx pay --help
   ```

   `@solana/pay` 1.0.26 y `qrcode-terminal` 0.12.0 son los últimos en npm al 2026-08-23. `@solana/kit` está fijado en 6.10.0 a propósito, y la razón es tu primera probada en vivo de la rotación contra la que las reglas de la casa acaban de advertirte: el tag `latest` de kit estaba en la línea 8.x cuando revisé (8.2.0, publicado el 2026-08-29 — corre `npm view @solana/kit version` y espera un número distinto), mientras que `@solana/pay` 1.0.26 declara un rango de pares de `^6.9.0`, así que 6.10.0 es el release de kit más nuevo que acepta de verdad. La brecha es el hecho durable aquí; el dígito exacto al frente del paquete no lo es. Lo instalamos explícitamente en vez de apoyarnos en la auto-instalación de pares de npm porque `qr.ts` importa de él directamente, y todo lo que importas pertenece a tu propio `package.json`. El primer comando instala la biblioteca oficial de pagos más un pequeño renderizador de códigos QR para la terminal. También, como ya dijimos, deja el ejecutable `pay` en `node_modules/.bin`, que es lo que `npx pay --help` encuentra: en la primera corrida descarga la build de la CLI para tu plataforma y después te muestra el toolchain de pagos agénticos. Hojea el texto de ayuda, fíjate en `gate`, `curl`, `send` y `account`, y sigue adelante. Hoy no lo estamos usando; solo necesitabas ver que es real.

   Esta es una instalación local, y local es como cada workspace que construyas en este curso instala sus dependencias; ignora cualquier consejo de otro lado de agregar `-g` aquí. El módulo 7 es la única excepción deliberada, instalando la CLI `pay` globalmente para que un `pay` pelado quede en tu PATH para el trabajo con gate de esa lección, y lo dice cuando lo hace. **Checkpoint: `npm ls --depth=0` lista exactamente tres dependencias, y `npx pay --help` imprime un banner que nombra "agentic payments" seguido de una lista de comandos que incluye `gate`, `curl`, `send` y `account`.** Si en cambio recibes "command not found" o npx ofrece instalar un paquete llamado `pay` del registro, el ejecutable no aterrizó: corre `ls node_modules/.bin/pay` para confirmar, y si falta, vuelve a correr la instalación y lee su salida buscando una falla de descarga (el binario se trae por la red en la primera corrida, así que un proxy o una máquina sin conexión te van a detener aquí). Si `npx pay --version` imprime un número `0.2x` en vez de 1.0.26, nada está mal; esa es la propia línea de release del ejecutable, como advirtió la sección de teoría. Quédate con el `npx`: un `pay` pelado aquí falla con command-not-found por una razón aburrida (las instalaciones locales ponen los binarios en `node_modules/.bin`, no en tu PATH) que se ve exactamente igual que la falla que este párrafo te está diciendo cómo diagnosticar.

6. **Genera la primera solicitud de pago de Wavelength.** Crea `qr.ts`:

   ```ts
   // qr.ts: turn a Solana Pay request into a scannable QR, right in your terminal.
   // Run: npx -y tsx@4.23.12 qr.ts
   import { encodeURL } from "@solana/pay";
   import { address } from "@solana/kit";
   import qrcode from "qrcode-terminal";

   const url = encodeURL({
     recipient: address("4NDXfTUeUnCVvzTvGVUAEBAHzWkadwv2zubvBHEHgmVi"), // Wavelength's till; leave it alone, step 8 explains why
     amount: 24, // UI units: twenty-four whole USDC, NOT 24000000 base units
     splToken: address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC on Solana
     memo: "WAV-0001", // Wavelength order reference
   });

   console.log(url.toString());
   qrcode.generate(url.toString(), { small: true });
   ```

   Ese `amount: 24` merece una advertencia, porque contradice el reflejo que el párrafo de los decimales acaba de instalarte. Antes, leyendo la blockchain, el entero en crudo `36115` quería decir 0.036115 USDC, porque los montos on-chain siempre son unidades base y escalas por los decimales del mint. El campo `amount` de Solana Pay es la convención opuesta: es un **monto de UI**, el número que teclearía un humano, así que `24` quiere decir veinticuatro USDC enteros. Escribir `24000000` aquí le pediría a tu cliente veinticuatro millones de dólares. Las dos APIs difieren porque sirven a lectores distintos: la blockchain habla en enteros para que ningún redondeo de punto flotante pueda tocar jamás un saldo, mientras que una solicitud de pago es un documento de cara al humano y la especificación eligió unidades de cara al humano. La regla para llevarte es simplemente que cada monto que manejes tiene una unidad declarada, y la compruebas en cada límite en vez de suponerla.

   Córrelo con `npx -y tsx@4.23.12 qr.ts`. Salida esperada: una URL `solana:` que lleva el destinatario, `amount=24`, el mint de USDC como `spl-token` y `memo=WAV-0001`, seguida de un código QR escaneable dibujado en ASCII. Esa URL es toda la solicitud de pago: 24 USDC a una dirección específica, etiquetada con una referencia de pedido. `encodeURL` la construyó; qrcode-terminal solo la dibujó (crédito a quien corresponde, ese paquetito ha sido calladamente útil por una década). **Checkpoint: un código QR se renderiza en tu terminal y la URL arriba de él contiene `memo=WAV-0001`.**

7. **Ahora, y solo ahora, instala una billetera.** En tu teléfono, instala una billetera Solana estándar: Phantom y Solflare son las opciones comunes, las dos en la tienda de apps de tu plataforma. Crea una billetera nueva, y toma en serio el ritual de la frase de recuperación aunque esta billetera no vaya a sostener nada hoy: escribe la frase, en papel, nunca en una captura de pantalla. Dos minutos, y ya sostienes el tipo de clave que firmó la transferencia que decodificaste en el paso 4. **Checkpoint: tu app de billetera muestra un saldo en cero y una dirección que puedes copiar; esta es la billetera de teléfono que escanea y liquida la venta en persona cuando el módulo 3 pone un POS en tu mostrador.**

8. **Escanea la solicitud.** En tu billetera, copia tu dirección nueva (la billetera la llama tu dirección o tu clave pública; es una cadena base58 como las que has estado leyendo toda la lección). Ahora deja `qr.ts` apuntando al destinatario de relleno y simplemente vuelve a correrlo, y después apunta la billetera de tu teléfono al código QR en tu pantalla. La billetera parsea la URL y muestra una vista previa de pago: 24 USDC a ese destinatario, memo adjunto, y una negativa a proceder porque tu billetera recién creada no sostiene nada. No pagues nada; la vista previa es la línea de meta.

   Deja al destinatario como alguien que no seas tú para esta vista previa. Apuntar una solicitud a tu propia dirección es el único caso en el que las billeteras no se ponen de acuerdo: Phantom hoy renderiza una vista previa de auto-transferencia con un banner de advertencia, Solflare se niega en seco, y las dos se van a quejar por separado de que no tienes SOL para cubrir la comisión de red. Nada de eso es tu bug, son solo tres advertencias sin relación apiladas en una pantalla, y oscurece la cosa que viniste a ver. **Checkpoint: tu propia billetera, escaneando un código QR que generó tu propio código, muestra correctamente el destinatario, el monto `24 USDC` y el memo de pedido `WAV-0001`.** Ya te paraste en los dos lados del mostrador.

![Flujo desde el script qr.ts pasando por la URL codificada y el código QR de la terminal hasta una billetera de teléfono que la parsea y previsualiza un pago de 24 USDC sin pagar.](assets/v08-flowchart.webp)

## Challenge: la Rosetta de los rieles de tarjeta

Este lo haces solo; ese es el trato que hicimos en la visión general. Toma el objeto decodificado del paso 4 (en vivo o fijado, cualquiera está bien) y anota cada campo con su equivalente en rieles de tarjeta y una oración sobre dónde se sostiene la analogía y dónde tiene fugas. Trabaja desde lo que observaste, no desde un motor de búsqueda.

Forma de la respuesta, con una fila hecha para ti:

| Campo del libro mayor | Equivalente en rieles de tarjeta | Dónde tiene fugas la analogía |
|---|---|---|
| `signature` | id de transacción | Un id de txn de tarjeta está acotado al sistema de un solo procesador; esta firma es globalmente única y cualquiera puede buscarla en el libro mayor compartido. |
| `sender` | ? | ? |
| `amountUSDC` | ? | ? |
| `mint` | ? | ? |
| `memo` | ? | ? |
| `settledAt` | ? | ? |

Llena las cinco filas restantes. Puntos de anclaje, para que te puedas calificar: el monto mapea al monto autorizado, el mint a la moneda, el memo a una referencia de pedido. La columna interesante es la tercera; "sender" en particular debería hacerte pensar en qué te muestra una red de tarjetas sobre quien paga frente a lo que este libro mayor acaba de mostrarte sobre un desconocido total.

Una tabla completa más las demos corriendo del lab son la barrera de esta lección. Si la tercera columna de cada fila dice "ninguna diferencia", vuelve y mira con más cuidado; si encontraste una fuga en todas y cada una de las filas, ya entendiste más sobre estos rieles de lo que la mayoría de las guías de integración te va a contar jamás.

Llegaste al final de toda la primera lección, demos y todo, así que déjame nombrar su límite: lo que tienes hoy es observación, todavía no un modelo. Has visto dinero liquidarse y escaneaste un código QR que construyó tu propio código, y si la inversión del libro mayor público todavía se siente un poco ilegal, bien, ese instinto quiere decir que lo entendiste. La próxima lección, "Finality vs. el stack de tarjetas: el modelo mental de los pagos", aporta el por qué: el modelo que explica por qué estos rieles se comportan distinto, dónde tu vocabulario de PSP mapea limpiamente sobre `processed`, `confirmed` y `finalized`, y el único reflejo de tarjeta que te va a costar más si te lo quedas. Trae la transacción decodificada del lab de hoy; vamos a interrogar qué quería decir "liquidado" en realidad.
