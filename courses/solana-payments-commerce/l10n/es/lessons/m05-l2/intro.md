# El programa oficial Subscriptions: planes, PDAs y la costura de kit-7

## Resumen

La lección pasada construiste el club-crank crudo sobre una sola aprobación de delegado y chocaste con su pared: un solo slot de delegado por cuenta de token quiere decir una sola suscripción viva por usuario, y la aprobación de un segundo comercio te desaloja. En silencio. Esta lección reemplaza la aprobación cruda por el programa Subscriptions de la Solana Foundation, cuya entera razón de existir es ese desalojo.

Antes de nombrar la costura de versiones de esta lección, demuéstratela a ti mismo. Tienes npm desde la lección de setup; corre esto desde cualquier lado:

```bash
npm view @solana/subscriptions@0.5.0 peerDependencies
```

Deberías ver `{ '@solana/kit': '^7.0.0' }`. Ahora corre `npm view @solana/pay peerDependencies` y mira su línea de kit: `^6.9.0`. Dos paquetes de los que depende este curso, dos versiones major del mismo SDK, las dos correctas. Guárdate esa idea; la resolvemos como se debe en la última sección de teoría, y te cuesta una carpeta de workspace extra. Los hallazgos, por delante:

- El programa vive en `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` y su truco es una sola jugada: una PDA de Subscription Authority por (usuario, mint) toma el único slot de delegado UNA VEZ con una aprobación de u64::MAX, y después las PDAs de delegación por plan cargan los límites de facturación reales y exigidos. Un slot, tantas suscripciones como el usuario quiera.
- Entregas **club-billing**: crea el plan del disco del mes de Wavelength, suscribe a un usuario de prueba, haz un pull de un período de facturación y aterriza ese pull como una fila de factura en el mismísimo libro mayor de pedidos del backoffice que construiste en la lección de webhooks. Una nota de honestidad que se arrastra por todo el lab: el pull oficial no lleva ninguna reference key ni ningún memo, así que la verdad de la facturación la escribe el crank, indexada sobre la firma del pull, no conciliada por reference como lo es un checkout; la lección de dunning le pega una reference a la factura cuando la liquidación necesita una.
- Dos bugs documentados de unidades y de reloj muerden a los integradores allá afuera, y ninguno es un exploit ni un cargo doble — el programa rechaza on-chain un pull mal cronometrado o caducado, así que los dos bugs queman comisiones del crank, no dinero del suscriptor. Los planes miden su período en `periodHours` mientras que todo contra lo que comparas son segundos Unix, y las cuentas de suscripción nunca se cierran solas, así que `expiresAtTs` — no la existencia de la cuenta — es el límite de tiempo que tu guarda tiene que replicar.
- El cliente es `@solana/subscriptions` 0.5.0, y hace peer con `@solana/kit` ^7.0.0 mientras tus workspaces de checkout se quedan en kit ^6. Esa costura es real, es el estado actual del ecosistema, y la manejamos con un pin de workspace aparte, no con una reescritura.

Una cosa más que vale decir sin vueltas. La versión 0.5.0 de este programa se desplegó a mainnet el 2026-08-10, hace doce días mientras escribo esto. Esa fecha es el deploy de la v0.5.0, no el debut del programa en mainnet (versiones anteriores estuvieron vivas antes), pero igual lo convierte en lo más nuevo y estructural de todo el curso. Lo estás aprendiendo antes de que exista la mayoría de las guías de integración. Tómalo como el trabajo y no como una advertencia de riesgo: a los ingenieros de pagos les pagan por ser tempranos y correctos al mismo tiempo.

## Un slot, muchas suscripciones

### La rampa de acceso a las PDA en 30 segundos

Allá en el módulo de stablecoins te dije que la dirección de la cuenta de token asociada es derivada, no elegida, y te prometí que la idea completa te iba a costar 30 segundos cuando por fin la necesitaras. Este es ese momento.

Una dirección derivada de programa (PDA) es una dirección calculada a partir de la dirección propia de un programa más algunos seeds, construida a propósito para que NO tenga clave privada. No es una clave perdida, no es una clave guardada bajo llave: no existe ninguna clave, matemáticamente. Nadie puede firmar nunca como esa dirección. Lo único que puede actuar como una PDA es el programa que la deriva, desde dentro de su propio código, bajo las reglas que ese código imponga.

Vuelve a leer eso con el cerebro de Stripe puesto, porque este es el clic sobre el que gira toda la lección. Cuando el crank de Wavelength tenía el slot de delegado directamente, la lección pasada, un keypair que TÚ controlabas tenía derechos de pull, y el suscriptor tenía que confiar en tus ops. Si en cambio una PDA tiene el slot de delegado, no hay clave de comercio que se filtre, no hay empleado que se descarríe, no hay brecha en tu servidor que entregue derechos de pull. Un residuo honesto: la upgrade authority del programa sigue siendo una clave que alguien tiene, así que la confianza se movió de tus ops al dueño del programa en vez de desaparecer — una superficie más chica y auditable, y la advertencia que te toca ofrecer en una revisión de seguridad antes de que la ofrezca alguien más. Pasado ese residuo, el único camino a los fondos del suscriptor es la lógica propia del programa, y la lógica del programa solo mueve fondos dentro de límites que el suscriptor firmó explícitamente. Eso es lo que quiere decir facturación no custodial, y se apoya en algo concreto: la ausencia de una clave privada del lado del comercio.

![Un keypair de comercio que tiene el slot de delegado obliga a confiar en todos los que puedan firmar, mientras que una PDA de Subscription Authority no tiene clave privada, así que solo la lógica del programa puede hacer un pull.](assets/v01-diagram.webp)

### Una aprobación ilimitada, muchas delegaciones acotadas

Acá está la arquitectura, y suena al revés hasta que ves por qué es la única forma que encaja.

El suscriptor inicializa una PDA de **Subscription Authority** (SA), una por cada par (usuario, mint). Esa PDA toma el único slot de delegado en la cuenta de token con una aprobación de u64::MAX. Ilimitada. El número que sería terrorífico en un keypair de comercio acá está bien, porque la PDA de SA no es un gastador, es un conmutador. Nunca va a mover un token por iniciativa propia; no tiene iniciativa, y no tiene clave.

Si el número ilimitado todavía te pica, mapéalo contra algo que ya entregaste. Una tarjeta guardada en un procesador de pagos es, mecánicamente, una autoridad de cobro ilimitada: nada en la red de tarjetas impide que un comercio cobre el monto equivocado, y el límite real es la política, los chargebacks y, en última instancia, los abogados. Acá la cosa sin límite es sin clave e inerte, y la cosa acotada es código que rechaza. Misma forma, orden de confianza opuesto. Sé cuál de las dos preferiría hacer pasar por una revisión de seguridad.

Los límites reales viven una capa más abajo, en **PDAs de delegación** creadas bajo esa autoridad. Cada delegación es una cuenta aparte que carga sus propios términos exigidos, y el programa rechaza cualquier pull que los viole. Tres modelos vienen en v0.5.0:

![Tres modelos de delegación puestos uno al lado del otro: fixed con un tope total y vencimiento opcional, recurring con un tope por período que se reinicia, en segundos, y plans publicados en horas.](assets/v02-table.webp)

Un recorrido rápido de cuándo se gana el sueldo cada modelo, porque el lab usa solo el tercero y los otros dos van a aparecer en tus conversaciones de producto dentro de una semana. **Fixed** es una cuenta con tope: haz pulls de hasta 50 USDC antes del viernes, y después la delegación está gastada. Encaja con autorizaciones puntuales con un techo, con un período de prueba que no debe convertirse en silencio, con una preventa que cobra cuando se despacha el prensado. **Recurring** es una asignación que se renueva: hasta 20 USDC por semana, indefinidamente o hasta el vencimiento, con `amountPulledInPeriod` reiniciándose en cada período. Encaja con la facturación por uso donde el monto varía pero el tope no debe, con una API medida en dólares, con una billetera de recarga que se rellena sola. El modelo **plan** es el que tiene forma de comercio: términos publicados una vez on-chain, cada suscriptor acepta esos términos exactos, los pulls aterrizan una vez por período de facturación en destinos que el plan declaró por adelantado. Wavelength quiere términos idénticos para cada miembro y un catálogo público al que pueda apuntar, así que el club es un plan. Y si alguna vez te encuentras acuñando cientos de planes casi idénticos para codificar términos a medida por cliente, detente; ese es el trabajo para el que existen las delegaciones recurring.

Fíjate en lo que esto disuelve. La pared de la lección pasada era que las aprobaciones se sobrescriben entre sí. Ahora el slot del suscriptor está ocupado exactamente una vez, por su propia PDA de autoridad, y sumarse al plan de un segundo comercio solo crea otra cuenta de delegación debajo. Los derechos de pull de Wavelength sobreviven a que el suscriptor se sume a otros diez clubes. Nadie desaloja a nadie, porque nadie vuelve a tocar el slot.

El modelo plan, el que usa el lab, parte el estado en dos cuentas. El comercio crea una PDA `Plan` (con seeds de la dirección del comercio más un id de plan) que tiene los términos: monto, `periodHours`, el mint, los destinos de pull permitidos, una whitelist de pullers. La aceptación del suscriptor crea una PDA `SubscriptionDelegation` (con seeds del plan más el suscriptor) que sigue su estado individual: `currentPeriodStartTs`, `amountPulledInPeriod`, `expiresAtTs`. El estado del comercio y el estado del suscriptor nunca comparten una cuenta, y por eso un plan escala a cualquier cantidad de suscriptores sin que nadie reescriba nada.

![Una sola cuenta Plan del comercio publica los términos mientras cada suscriptor recibe una cuenta SubscriptionDelegation aparte, así un plan escala a cualquier cantidad de suscriptores sin reescribir estado.](assets/v03-diagram.webp)

Y la exigencia no es un consejo. Intenta hacer dos pulls en un período y el programa lo rechaza con un error de período-no-transcurrido antes de que se mueva un token. Intenta hacer un pull hacia una dirección que el plan nunca declaró y te llevas un rechazo de destino-no-autorizado. Los límites que vas a implementar en la guarda del crank de esta lección son una capa de cortesía que te ahorra comisiones y ruido de logs; el programa es la capa que salva al suscriptor.

### La puerta de salida, y qué canjeó el suscriptor por ella

La facturación no custodial solo es honesta si irse es tan unilateral como entrar. Lo es. El suscriptor puede darse de baja de un plan, lo que cierra su cuenta de delegación, y puede revocar la Subscription Authority misma, lo que desocupa el slot de delegado y termina toda delegación debajo en un solo movimiento. Ninguna firma del comercio aparece en ninguno de los dos caminos; la billetera que consintió puede retirar el consentimiento sola, en cualquier momento. Una precisión sobre el rent que esas cuentas tienen, porque es fácil suponer que la salida lo reembolsa: darse de baja cierra la cuenta de delegación y devuelve su rent a quien lo pagó, pero revocar la Subscription Authority solo desocupa el slot de delegado — no barre las PDAs. Recuperar esas es una llamada `RevokeAbandoned` aparte, que corre el comercio, y cuyo firmante es el **pagador registrado**, que es tu billetera si patrocinaste el subscribe. La próxima lección construye esa fila de trabajo; hoy el punto es que irse es unilateral, no que se limpie solo. Compara eso con el flujo de retención de veinte minutos que te hizo comer tu última membresía de gimnasio.

La contrapartida, nombrada, porque el diseño de la lección pasada tenía una virtud que este retira en silencio. La aprobación cruda de 60 USDC se secaba después de cuatro pulls, y ese agotamiento forzaba una conversación natural de re-consentimiento cada cuatro meses. La aprobación de u64::MAX de la SA nunca se seca. La protección del suscriptor ya no es un número que se encoge; son los límites por plan más esa salida unilateral. Mecánicamente esa es una protección estrictamente mejor, y aun así merece este párrafo, porque la asignación que se iba encogiendo hacía un trabajo silencioso de UX en el diseño crudo que acá nada automático reemplaza: a nadie le vuelven a preguntar por defecto. Muestra las suscripciones activas en la UI de tu producto y haz que cancelar sea un toque; la blockchain no va a insistir de tu parte.

![Una suscripción corre desde la inicialización de la autoridad y a través de pulls periódicos; una caducada persiste on-chain, detenida solo por la comprobación de vencimiento, hasta que el suscriptor se da de baja o revoca.](assets/v04-timeline.webp)

### Quién lo construyó, y quién ya apuesta por él

La procedencia importa más que lo habitual cuando la cosa es así de nueva. El programa lo escribió Moonsong Labs en Pinocchio, el framework de Rust sin dependencias, y lo auditó Cantina. Si la elección de Pinocchio te da curiosidad sobre cómo se construye un programa a ese nivel, esa curiosidad le pertenece al curso Master Anchor V2, que es el dueño de la capa de framework; acá consumimos el programa, no leemos su código fuente.

El dato de proof-of-production le gana a una insignia de auditoría, de todos modos: Helius corre su PROPIA facturación de suscripciones sobre este mismo programa Subscriptions de la Foundation (su blog de ingeniería, traído el 2026-08-21). Cuando una empresa de infraestructura cuyo producto es el uptime les factura a sus clientes a través de un programa, ese programa dejó el territorio de las demos. Un programa auditado con un inquilino de producción de marca conocida es una apuesta distinta a un deploy de una semana tomado por fe.

### Token-2022, consumido, no enseñado

El mint del plan puede ser un mint de Token-2022, y dos comportamientos importan para la facturación. Primero, si el mint carga un transfer hook, el programa reenvía las cuentas extra del hook a su CPI de TransferChecked, así un pull compone con tokens condicionados por hook en vez de morir sobre ellos; el cliente incluso trae un helper `resolveTransferHookAccounts` para la resolución de cuentas. Segundo, si una cuenta de destino tiene MemoTransfer activado (exige un memo en cada transferencia entrante), el pull se rechaza atómicamente: nada de estado parcial, nada de fondos trabados, la transacción simplemente falla completa. El programa además examina el conjunto de extensiones del mint cuando se inicializa la autoridad y rechaza las combinaciones que no puede facturar con seguridad, así te enteras a la hora del setup, no a la hora de cobrar.

Eso es todo lo que necesitamos SABER acá. Cómo funciona la interfaz de transfer hook en sí, de punta a punta, es territorio del curso Digital Assets, Tokenization and Token Extensions. Somos consumidores de hooks, y a los consumidores les toca quedarse dichosamente delgados.

### Los dos relojes que le facturan mal a la gente

Ahora las trampas, porque los dos bugs de integración documentados de este programa son los dos bugs de tiempo, los dos arruinan una corrida de facturación a su manera, y los dos van a estar sentados en el starter de tu challenge a propósito.

**Bug uno: las horas no son segundos.** Un `Plan` guarda su cadencia como `periodHours` (720 para el plan mensual de Wavelength). Una `RecurringDelegation` guarda su cadencia como `periodLengthS`, en segundos. Cada timestamp contra el que vas a comparar alguna vez, `currentPeriodStartTs`, `expiresAtTs`, el tiempo on-chain, son segundos Unix. Compara `periodHours` directamente contra un delta en segundos y tu ventana se encoge por un factor de 3,600: tu crank declara que a un plan de 24 horas le toca de nuevo después de 24 segundos. Fíjate quién salva al suscriptor acá — el programa, exactamente como dijo la sección de la exigencia: el pull temprano se rechaza con el error de período-no-transcurrido antes de que se mueva un token. Lo que el programa no puede salvar es tu billetera y tus logs: cada pull rechazado le cuesta al crank una comisión base y una línea de log, en cada tick, para siempre, hasta que te des cuenta. La regla es aburrida y absoluta: convierte a segundos en la frontera, compara solo segundos. 24 horas son 86,400 segundos, no 24.

**Bug dos: nada vence por sí solo.** Las cuentas de suscripción y de delegación persisten on-chain hasta que una instrucción explícita de revoke las cierra. Un plan cuyo término terminó la semana pasada todavía tiene una cuenta de delegación viva ahí sentada, y si tu crank solo comprueba "¿existe la delegación?", va a seguir lanzando pulls para ese suscriptor caducado. Misma división del trabajo que el bug uno: el programa exige el límite de tiempo — un pull contra una delegación caducada se rechaza con su error de suscripción-cancelada antes de que se mueva un token, así que no le cobran a nadie — y lo que el crank de solo-existencia se compra es el mismo impuesto, una comisión base y una línea de log por cada tick rechazado, para siempre. `expiresAtTs` contra el tiempo on-chain es la comprobación que tu guarda replica en cada tick, así el rechazo pasa en tu proceso gratis en vez de on-chain por una comisión. Y su caso cero muerde en la dirección contraria: `expiresAtTs` en 0 quiere decir "nunca vence", así que una guarda que compara ingenuamente `now >= expiresAtTs` trata cada suscripción sin vencimiento como vencida en la epoch y se niega a facturarle a nadie. Maneja el cero primero, después compara.

![Dos bugs de facturación documentados puestos uno al lado del otro: leer periodHours como segundos lanza pulls alrededor de 3600 veces más seguido de lo debido, y saltarse expiresAtTs sigue lanzando pulls para suscriptores caducados; el programa rechaza los dos, a una comisión base por rechazo.](assets/v05-comparison.webp)

### La costura de kit: checkout en v6, facturación en v7

Hora de resolver el sondeo que corriste en el primer minuto. Lo que sacó a la superficie es el estado actual de la industria, así que manejémoslo como profesionales: con hechos fechados y un pin.

Los hechos, re-verificados contra npm el 2026-08-22: el dist-tag `latest` de kit apunta a 8.0.0 (publicado el 2026-08-21); la línea v7 terminó en 7.1.1; la línea v6 terminó en 6.10.0. La versión 7 es el estándar de peers del ecosistema ahora mismo: la ola de clientes `@solana-program/*` de julio de 2026 hace peer con `^7.0.0` (ese es `@solana-program/token` 0.15.0), y `@solana/subscriptions` 0.5.0 también. Mira qué rápido se mueve la punta del pelotón, igual: `@solana-program/token` 0.16.0 salió el 2026-08-21, el mismo día que kit 8, y ya hace peer con `^8.0.0`. Los rezagados son igual de reales y de estructurales para nosotros: `@solana/pay` 1.0.26 hace peer con kit `^6.9.0`, que es exactamente la razón por la que tus workspaces de checkout quedaron fijados a kit ^6.10 en primer lugar, y no está solo allá abajo — helius-sdk 3.1.0 hace peer con el mismo rango `^6.9.0`, así que una tienda que sí hubiera agarrado ese SDK aterrizaría en el pin idéntico. Tres majors de kit, todas en circulación, todas correctas para alguien. Instala subscriptions dentro de esos workspaces y el resolvedor de peers de npm va a rechazarlo, correctamente. Nunca fijes a `latest` en ningún lado; estos tags se movieron dos veces mientras se escribía este curso.

¿El desbloqueo? Una carpeta que deliberadamente se queda AFUERA del roster de workspaces. Los workspaces registrados de npm no son aislamiento — son lo contrario: npm hace hoisting de cada paquete registrado hacia una única resolución compartida en la raíz, que es exactamente el árbol donde kit 6 y kit 7 se encontrarían y pelearían. Así que el paso 1 de abajo crea `subscriptions/` como un paquete standalone y nunca lo agrega al array `workspaces` de la raíz — un quiebre deliberado con la costumbre de registrar-todo que te enseñó el módulo 4. Esa sola carpeta fija kit ^7 más `@solana/subscriptions` 0.5.0, corre su propio `npm install`, resuelve desde su propio `node_modules`, y los rangos de peers nunca se encuentran. (Regístrala en la raíz y el resolvedor de npm va a intentar reconciliar los dos majors de kit en un solo árbol y va a rechazarlo; el capstone se mete en ese ERESOLVE exacto a propósito y te muestra la vía de escape.) Y si aparece fricción de v7 que no puedes despejar, el fallback documentado es una edición de pin de dos líneas en esa única carpeta: `@solana/subscriptions` 0.4.0 con kit ^6.4. Ningún cambio estructural, ninguna reescritura, el `package.json` de una sola carpeta.

![Los workspaces de checkout y de ops se quedan fijados a paquetes de kit 6 mientras la carpeta subscriptions, deliberadamente dejada afuera del array workspaces de la raíz, fija kit 7, con el fallback documentado a subscriptions 0.4.0 sobre kit 6.4.](assets/v06-diagram.webp)

¿Esto es molesto? Un poco. ¿Es inusual? Ni un poco: cualquier taller de Node que sobrevivió la migración a ESM, o un major de React, ya corrió esta misma maniobra. Los ecosistemas de SDK se mueven de adelante hacia atrás, los paquetes emblemáticos saltan primero, las integraciones se retrasan, y la frontera vive en tus lockfiles por un trimestre o dos. No estás rodeando un error; estás mirando un ecosistema a mitad de zancada, y el pin por workspace es a lo que se parece la competencia mientras aterriza. Ni es una regla que este curso inventó: el curso Rust & TypeScript Fundamentals la machaca en su lección de rangos de peers y Master Anchor V2 la machaca contra los peers declarados de un cliente generado en su módulo ocho — tres cursos, una regla.

## Lab: factura al club del disco del mes

Cómo se parte el trabajo acá: las llamadas de authority, plan y subscribe vienen con scaffold y las camino completas; el paso de escritura al libro mayor te toca cablearlo a ti (completion); y la guarda de la ventana de período es el challenge independiente después del lab (solo). Para el módulo de ops vas a estar construyendo esta categoría de integración sin ningún scaffold, así que mira lo que hace el scaffold mientras todavía lo tienes.

**1. Crea el workspace y fija la costura.** Desde la raíz del monorepo:

```bash
mkdir -p subscriptions/keys
cd subscriptions
npm init -y
npm pkg set type=module
npm install @solana/subscriptions@0.5.0 @solana/kit@7.1.1 @solana-program/token@0.15.0
npm install -D typescript tsx @types/node
```

Pines verificados el 2026-08-22 y re-estampados el 2026-09-05: `@solana/subscriptions` 0.5.0 es la línea estampada para este paquete — pre-1.0 y en movimiento, liberada el 2026-08-10, hace peer con kit ^7.0.0 — con 0.4.0 guardada solo como fallback documentado, nunca el pin del que arrancas; 7.1.1 es el último release de la línea v7 de kit; `@solana-program/token` 0.15.0 es la ola del cliente de token que hace peer con ^7. Ese último pin es el que la gente equivoca: `latest` para el cliente de token es 0.16.0, que hace peer con kit ^8 y no va a resolver acá. Vuelve a comprobar los tres con `npm view <pkg> dist-tags peerDependencies` antes de instalar, porque este rincón de npm se ha movido dos veces este trimestre. `tsx` corre TypeScript directamente y `@types/node` te da los tipos de Node; has usado los dos desde el módulo de checkout, pero este es un workspace nuevo, así que se instalan de nuevo.

Necesitas dos keypairs de devnet fondeados en `keys/`: `merchant.json` (Wavelength) y `subscriber.json` (tu oyente de prueba), con el suscriptor teniendo algo de la USDC de devnet en la que has facturado desde el módulo 2. La lección del crank acuñó `subscriber.json` (cópialo dentro de `keys/`) pero nunca tuvo una clave de comercio, así que crea esa nueva: `solana-keygen new -o keys/merchant.json`, y después haz airdrop y fondea como siempre.

**2. Config compartida y un helper de envío.** Dos archivos chicos que vas a reconocer de cada workspace hasta ahora. Primero `config.ts`:

```typescript
import { readFileSync } from "node:fs";
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  sendAndConfirmTransactionFactory,
  type Address,
  type KeyPairSigner,
} from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

export const rpc = createSolanaRpc("https://api.devnet.solana.com");
export const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com",
);
export const sendAndConfirm = sendAndConfirmTransactionFactory({
  rpc,
  rpcSubscriptions,
});

// The mint the club bills in: the same devnet USDC mint the crank lesson
// hardcoded (4zMM...ncDU), taken as an env var here so later lessons can
// re-point the club without code edits.
export const CLUB_MINT: Address = address(process.env.CLUB_MINT!);
export const CLUB_TOKEN_PROGRAM = TOKEN_PROGRAM_ADDRESS;

export async function loadSigner(path: string): Promise<KeyPairSigner> {
  const bytes = new Uint8Array(JSON.parse(readFileSync(path, "utf8")));
  return createKeyPairSignerFromBytes(bytes);
}
```

Después `send.ts`, el pipeline canónico de envío de kit. La misma forma que has escrito tres veces en los workspaces de v6; la sorpresa agradable de la costura es que este código es idéntico en v7:

```typescript
import {
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  getSignatureFromTransaction,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import { rpc, sendAndConfirm } from "./config";

export async function sendIxs(
  payer: KeyPairSigner,
  ixs: Instruction[],
): Promise<string> {
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
  return getSignatureFromTransaction(signed);
}
```

**3. Inicializa la Subscription Authority del suscriptor.** Este es el paso de una-sola-vez-por-(usuario, mint): la PDA de SA toma el slot de delegado con su aprobación de u64::MAX, y toda suscripción futura para este mint cuelga de ella. `01-init-authority.ts`:

```typescript
import { getInitSubscriptionAuthorityInstructionAsync } from "@solana/subscriptions";
import { findAssociatedTokenPda } from "@solana-program/token";
import { CLUB_MINT, CLUB_TOKEN_PROGRAM, loadSigner } from "./config";
import { sendIxs } from "./send";

async function main() {
  const subscriber = await loadSigner("keys/subscriber.json");

  const [userAta] = await findAssociatedTokenPda({
    owner: subscriber.address,
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const ix = await getInitSubscriptionAuthorityInstructionAsync({
    owner: subscriber,
    tokenMint: CLUB_MINT,
    userAta,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const sig = await sendIxs(subscriber, [ix]);
  console.log("subscription authority initialized:", sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Córrelo con `CLUB_MINT=<your devnet mint> npx tsx 01-init-authority.ts`. Fíjate quién firma: el SUSCRIPTOR. Solo el dueño de la cuenta puede entregar su slot de delegado, que es el paso de consentimiento de toda la arquitectura; Wavelength nunca toca esta transacción.

Una comprobación con la realidad antes de que sigas, porque la versión más nueva del programa es así de fresca: el despliegue en devnet puede ir atrás del de mainnet. Sondéalo primero, `solana program show De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44 --url devnet` tiene que imprimir un programa ejecutable (lo hizo el 2026-08-22). El desfase de versiones solo se revela en la primera instrucción: si esta llamada o cualquiera posterior falla con el error de programa custom 133 o 134 (el cliente los nombra `DELEGATION_VERSION_MISMATCH` y `MIGRATION_REQUIRED`), el binario de devnet es anterior a tu cliente 0.5.0. Toma el remedio del lado del cliente, que es el lado que este curso fija y verifica: baja este único workspace al pin de fallback documentado en la sección de la costura, `@solana/subscriptions@0.4.0` con kit `^6.4`, y sigue contra el programa desplegado en devnet tal como está. Construir el programa tú mismo es un proyecto distinto de este lab, y uno cuya revisión de fuente nada de acá comprueba por ti. Comprueba primero, enójate después.

**4. Crea el plan.** Wavelength publica el disco del mes al mismo precio que facturó el crank crudo la lección pasada: 15 USDC cada 720 horas. `02-create-plan.ts`:

```typescript
import {
  findPlanPda,
  getCreatePlanInstruction,
} from "@solana/subscriptions";
import { address } from "@solana/kit";
import { CLUB_MINT, loadSigner } from "./config";
import { sendIxs } from "./send";

const PLAN_ID = 1n;

// The on-chain Plan stores destinations and pullers as fixed four-slot
// arrays; unused slots carry the system address as an explicit "empty".
const NONE = address("11111111111111111111111111111111");

async function main() {
  const merchant = await loadSigner("keys/merchant.json");

  const [planPda] = await findPlanPda({
    owner: merchant.address,
    planId: PLAN_ID,
  });

  const ix = getCreatePlanInstruction({
    merchant,
    planPda,
    tokenMint: CLUB_MINT,
    planData: {
      planId: PLAN_ID,
      mint: CLUB_MINT,
      terms: {
        amount: 15_000_000n, // 15 USDC at 6 decimals
        periodHours: 720n, // HOURS on the plan; you convert everywhere else
        // The program stamps its own clock over this field at execution;
        // 03-subscribe reads the stored value back rather than trusting ours.
        createdAt: BigInt(Math.floor(Date.now() / 1000)),
      },
      endTs: 0n, // 0 = no scheduled end, the same zero-means-never convention as expiry
      // Destinations are WALLET addresses, never token accounts: the program
      // whitelists the owner, and each pull presents that owner's ATA for the
      // plan's mint, derived at pull time. Declare an ATA here and every pull
      // is refused on-chain with the destination-not-in-whitelist error.
      destinations: [merchant.address, NONE, NONE, NONE],
      pullers: [merchant.address, NONE, NONE, NONE],
      metadataUri: "https://wavelength.example/plans/record-of-the-month.json",
    },
  });

  const sig = await sendIxs(merchant, [ix]);
  console.log("plan created:", planPda, sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Los arrays `destinations` y `pullers` son el control de acceso propio del plan, on-chain: un pull solo puede aterrizar en la ATA de una billetera de destino declarada, y solo el dueño del plan o un puller en la whitelist puede iniciar uno. Los dos arrays viajan como structs fijos de cuatro slots (el cliente codifica exactamente cuatro entradas, de ahí el relleno con `NONE`) y los dos tienen direcciones de billetera, verificado contra devnet: los planes reales guardan dueños, y a un plan que guardó una ATA en su lugar le rechazaron los pulls con el error de destino-no-en-whitelist. Tu keypair de crank va en `pullers` cuando lo lleves a producción; para el lab la clave del comercio hace los pulls directamente.

**5. Suscribe al usuario de prueba.** El paso de aceptación, y mi detalle de diseño favorito de todo el programa. `03-subscribe.ts`:

```typescript
import {
  fetchPlan,
  fetchSubscriptionAuthorityFromSeeds,
  findPlanPda,
  getSubscribeInstructionAsync,
} from "@solana/subscriptions";
import { address } from "@solana/kit";
import { CLUB_MINT, loadSigner, rpc } from "./config";
import { sendIxs } from "./send";

const PLAN_ID = 1n;
const MERCHANT = address(process.env.MERCHANT!);

async function main() {
  const subscriber = await loadSigner("keys/subscriber.json");

  const [planPda, planBump] = await findPlanPda({
    owner: MERCHANT,
    planId: PLAN_ID,
  });
  const plan = await fetchPlan(rpc, planPda);
  const authority = await fetchSubscriptionAuthorityFromSeeds(rpc, {
    user: subscriber.address,
    tokenMint: CLUB_MINT,
  });

  // The expected* fields pin the terms you read to the terms that execute.
  // If the plan changes between your read and your landing, the program
  // refuses with a terms-mismatch error instead of billing you.
  const ix = await getSubscribeInstructionAsync({
    subscriber,
    merchant: MERCHANT,
    planPda,
    subscriptionAuthorityPda: authority.address,
    subscribeData: {
      planId: PLAN_ID,
      planBump,
      // Double .data is not a typo: fetchPlan returns the account wrapper,
      // whose data field holds the program's versioned Plan struct. The
      // delegation and authority accounts decode flat, hence their single
      // .data everywhere else in this lab.
      expectedMint: plan.data.data.mint,
      expectedAmount: plan.data.data.terms.amount,
      expectedPeriodHours: plan.data.data.terms.periodHours,
      expectedCreatedAt: plan.data.data.terms.createdAt,
      expectedSubscriptionAuthorityInitId: authority.data.initId,
    },
  });

  const sig = await sendIxs(subscriber, [ix]);
  console.log("subscribed:", sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Esos campos `expected*` merecen la pausa. El suscriptor firma los términos exactos que leyó, y el programa los compara con el plan a la hora de ejecutar. Un comercio que edita el precio entre el clic del suscriptor y el aterrizaje de la transacción se lleva un rechazo, no una ganancia caída del cielo. Los sistemas de suscripción de la Web2 exigen esto con abogados y capturas de pantalla; acá es una comparación de structs dentro de la transacción. Córrelo con `MERCHANT=<merchant pubkey> CLUB_MINT=<mint> npx tsx 03-subscribe.ts`.

![En cada tick de facturación el crank lee el estado de la suscripción, decidePull filtra los pulls cancelados, vencidos y demasiado tempranos antes de gastar ninguna comisión, y un pull al que le toca aterriza una sola vez en el libro mayor.](assets/v07-flowchart.webp)

**6. Haz un pull de un período de facturación, y aterrízalo en el libro mayor.** Uno de los imports de abajo todavía no existe: `./decide-pull`. Guarda el starter del Challenge al final de esta lección como `subscriptions/decide-pull.ts` ahora, con bugs y todo, para que el lab corra en orden; reparar esos dos bugs es el trabajo solo que te espera allá. Este es el paso de acumulación, y la razón por la que esta lección consume dos artefactos anteriores en vez de uno. El pull en sí reemplaza el `TransferChecked` del crank crudo; la escritura al libro mayor es lo que convierte un movimiento de tokens en un evento de negocio. `04-pull.ts`:

```typescript
import {
  fetchPlan,
  fetchSubscriptionDelegation,
  findPlanPda,
  findSubscriptionAuthorityPda,
  findSubscriptionDelegationPda,
  getTransferSubscriptionInstructionAsync,
} from "@solana/subscriptions";
import { findAssociatedTokenPda } from "@solana-program/token";
import { address } from "@solana/kit";
import { CLUB_MINT, CLUB_TOKEN_PROGRAM, loadSigner, rpc } from "./config";
import { decidePull } from "./decide-pull";
import { recordInvoice } from "./ledger-bridge";
import { sendIxs } from "./send";

const PLAN_ID = 1n;
const SUBSCRIBER = address(process.env.SUBSCRIBER!);

async function main() {
  const merchant = await loadSigner("keys/merchant.json");

  const [planPda] = await findPlanPda({
    owner: merchant.address,
    planId: PLAN_ID,
  });
  const [subscriptionPda] = await findSubscriptionDelegationPda({
    planPda,
    subscriber: SUBSCRIBER,
  });
  const [authorityPda] = await findSubscriptionAuthorityPda({
    user: SUBSCRIBER,
    tokenMint: CLUB_MINT,
  });

  const plan = await fetchPlan(rpc, planPda);
  const sub = await fetchSubscriptionDelegation(rpc, subscriptionPda);

  // decidePull takes its five scalars positionally:
  // active, expiresAtTs, lastChargedTs, periodHours, now.
  const decision = decidePull(
    // Cancellation is not invisible on-chain: a canceled subscription's
    // delegation account persists, readable, until the subscriber revokes
    // it, and the program refuses pulls against it with its
    // subscription-cancelled error. This demo pulls the subscription you
    // created two steps ago, which cannot have been canceled yet, so it
    // passes active = true; the dunning lesson wires this argument to the
    // cancellation state it reads, so the guard's 'canceled' arm refuses
    // before a fee is spent instead of after a refusal.
    true,
    Number(sub.data.expiresAtTs),
    Number(sub.data.currentPeriodStartTs),
    Number(sub.data.terms.periodHours),
    Math.floor(Date.now() / 1000),
  );
  if (!decision.shouldPull) {
    console.log("refused:", decision.reason, "next eligible:", decision.nextEligibleTs);
    return;
  }

  const [delegatorAta] = await findAssociatedTokenPda({
    owner: SUBSCRIBER,
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });
  // destinations[0] is the treasury WALLET the plan declared; the receiving
  // token account is that wallet's ATA, derived here at pull time.
  const [receiverAta] = await findAssociatedTokenPda({
    owner: plan.data.data.destinations[0],
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const ix = await getTransferSubscriptionInstructionAsync({
    subscriptionPda,
    planPda,
    subscriptionAuthority: authorityPda,
    delegatorAta,
    receiverAta,
    caller: merchant,
    tokenMint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
    transferData: {
      amount: sub.data.terms.amount,
      delegator: SUBSCRIBER,
      mint: CLUB_MINT,
    },
  });

  const signature = await sendIxs(merchant, [ix]);

  const fresh = recordInvoice({
    kind: "subscription-pull",
    signature,
    invoiceId: `${planPda}-${sub.data.currentPeriodStartTs}`,
    plan: planPda,
    subscriber: SUBSCRIBER,
    amount: sub.data.terms.amount.toString(),
    mint: CLUB_MINT,
    pulledAt: Math.floor(Date.now() / 1000),
  });
  console.log(
    fresh
      ? `pull landed in backoffice ledger: ${signature}`
      : `duplicate pull skipped by ledger: ${signature}`,
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Córrelo, porque todo lo que viene después de este paso asume que un pull aterrizó:

```bash
CLUB_MINT=<your devnet mint> SUBSCRIBER=$(solana-keygen pubkey subscriber.json) \
  npx tsx 04-pull.ts
```

Checkpoint: `pull landed in backoffice ledger: <signature>`. Córrelo una segunda vez de inmediato y te debería dar `refused: too-early` antes de que se construya ninguna transacción — ese rechazo no es una falla, es tu crank negándose a gastar una comisión en un pull que el programa iba a rechazar de todos modos. Los rechazos baratos son toda la razón por la que la guarda existe del lado del cliente siquiera. (Con el starter del paso 6 todavía sin arreglar puedes ver que la primera corrida aterriza cuando no debería; esa es la costura que cierra el Challenge, y la barrera al final del lab es la que la juzga.)

Una conversión de ahí adentro es deliberada y vale una oración, porque el módulo 2 te machacó el hábito opuesto. Los casts de `Number(...)` están sobre timestamps y sobre una cuenta de horas, nunca sobre un monto: `sub.data.terms.amount` se queda `bigint` todo el camino hasta la instrucción, exactamente como lo exige la regla de unidades base. Los segundos Unix y una cadencia de plan son enteros chicos que JavaScript representa exactamente por el próximo cuarto de millón de años; el dinero no. La guarda toma numbers, la transferencia toma bigints, y la frontera entre las dos es una línea a la que puedes apuntar.

**7. Cablea el puente al libro mayor (tu paso de completion).** `recordInvoice` todavía no existe; ese hueco es tuyo. Agrega al MISMO `backoffice/orders.jsonl` en el que el receptor vivo de la lección de webhooks (`main.ts`) escribe filas de checkout, bajo la misma disciplina: una línea JSON por fila, indexada sobre la firma de la transacción, y una firma que ya está presente nunca se escribe dos veces. Acá está el mío, `ledger-bridge.ts`; escribe el tuyo antes de espiar, y después compara:

```typescript
import { appendFileSync, existsSync, readFileSync } from "node:fs";

// Same file, same discipline as the backoffice orders ledger.
const LEDGER_PATH = "../backoffice/orders.jsonl"; // the file the live receiver (main.ts) writes

export interface InvoiceRow {
  kind: "subscription-pull";
  signature: string;
  invoiceId: string;
  plan: string;
  subscriber: string;
  amount: string; // base units, stringified bigint
  mint: string;
  pulledAt: number; // Unix seconds
}

export function recordInvoice(row: InvoiceRow): boolean {
  if (existsSync(LEDGER_PATH)) {
    const seen = readFileSync(LEDGER_PATH, "utf8")
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as { signature: string });
    if (seen.some((r) => r.signature === row.signature)) {
      return false; // exactly-once: the crank retried, the ledger did not
    }
  }
  appendFileSync(LEDGER_PATH, JSON.stringify(row) + "\n");
  return true;
}
```

Mira lo que le acaba de pasar a tu back office. El libro mayor que registraba checkouts verificados por webhook ahora registra pulls de facturación, con la misma garantía de exactamente-una-vez indexada sobre la firma, en el mismo archivo que lee tu conciliación. Un cargo de suscripción ahora queda registrado con la misma disciplina de exactamente-una-vez indexada sobre la firma que un disco vendido sobre el mostrador. Una diferencia honesta: el pull oficial no lleva ninguna reference key ni memo, así que el vínculo de la blockchain al libro mayor es la escritura propia del crank, no un marcador on-chain que tu conciliador pudiera redescubrir; si el crank alguna vez se muere entre el envío y el registro, el barrido de tesorería del módulo 4 es lo que encuentra al huérfano. Salvo esa advertencia, esta es la lección donde construir el libro mayor antes de la facturación rinde.

Una arruga que vale la pena anticipar, porque tu infraestructura ya es lo bastante buena para crearla: el pull es una transferencia de tokens, así que el webhook de Helius que registraste en la lección del backoffice TAMBIÉN se lo va a entregar a tu receptor como un evento TRANSFER. El receptor va a intentar resolverlo a un pedido por memo, no va a encontrar ninguno, y lo va a rechazar. Ese es el comportamiento correcto, no un bug que arreglar. La verdad del checkout entra al libro mayor por el pipeline de webhooks, la verdad de la facturación entra por el crank, y la clave de la firma evita que los dos caminos escriban alguna vez el mismo pago dos veces. Si los logs de rechazo te molestan, filtra los pulls por la ATA de tu tesorería; lo que no debes hacer es dejar que el camino del webhook escriba facturas. Un solo escritor por flujo de ingresos.

![Los eventos de checkout llegan por webhook verificado y los pulls de suscripción llegan por el crank de facturación, convergiendo como filas indexadas sobre la firma, de exactamente una vez, en el único libro mayor de pedidos del backoffice.](assets/v08-diagram.webp)

**8. Vuelve a poner al crank a cargo.** Todo hasta ahora corrió como scripts puntuales, pero el artefacto que esta lección entrega es club-billing, y lo que lo hace un sistema de facturación y no una demo es el loop del crank de la lección pasada manejando el camino del pull sobre un reloj. La refactorización toma dos minutos, y una línea de ella es estructural de una manera fácil de pasar por alto. En `04-pull.ts`, levanta el cuerpo de `main` a un `pullOnce(subscriber: Address)` exportado y saca del scope de módulo la lectura de env de `SUBSCRIBER`, porque el suscriptor ahora es un parámetro. Todavía quieres que el script puntual funcione, así que quédate con el `main` que lee la variable de entorno y llama a `pullOnce` — pero ahora tiene que correr solo cuando el archivo se ejecuta directamente, porque `05-crank.ts` está a punto de *importar* este archivo, y un `main().catch(() => process.exit(1))` a nivel de módulo se dispararía en el import y mataría al crank antes de su primer tick:

```typescript
// 04-pull.ts, at the bottom. The guard is what lets one file be both
// a script and a library.
const runDirectly =
  process.argv[1] !== undefined &&
  import.meta.url === new URL(`file://${process.argv[1]}`).href;

if (runDirectly) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
```

Esa es la misma barrera de ejecución directa que el capstone le pone a cada archivo de servidor, encontrada acá primero. Después `05-crank.ts` es la forma de tick de la lección pasada apuntada a las nuevas internas:

```typescript
// 05-crank.ts: the club-crank tick loop, now driving official pulls.
import { address } from "@solana/kit";
import { readFileSync } from "node:fs";
import { pullOnce } from "./04-pull";

const TICK_MS = 60_000;

async function tick(): Promise<void> {
  const subscribers = JSON.parse(
    readFileSync("subscribers.json", "utf8"),
  ) as string[];
  for (const s of subscribers) {
    try {
      await pullOnce(address(s));
    } catch (e) {
      console.error("pull failed for", s, e instanceof Error ? e.message : e);
    }
  }
}

tick();
setInterval(() => {
  void tick();
}, TICK_MS);
```

`subscribers.json` es un array JSON plano de direcciones de suscriptores; para el lab tiene a tu único oyente de prueba. En producción la lista viene de indexar las cuentas de delegación del programa, y el indexado a escala se le pasa al curso Client-Side Mastery, el mismo relevo que hizo la lección de webhooks. Fíjate en lo que el loop ya no hace: no lee el campo delegate de la cuenta de token ni lo compara contra su propia dirección, porque no hay keypair de crank con derechos de pull que comparar. La comprobación de consentimiento se movió on-chain, la comprobación de agenda se movió dentro de `decidePull`, y el loop se volvió más tonto, que es el sentido de marcha correcto para el componente que corre sin supervisión a las 3 a.m. La economía del pull se traslada de la lección pasada sin cambios: el llamador paga la comisión base por pull, y el suscriptor no firma nada y no paga nada por ciclo. Checkpoint: `npx tsx 05-crank.ts` imprime una línea `refused: too-early` por tick para el suscriptor al que acabas de facturar, una vez por minuto, y nunca envía una transacción. Mira dos ticks, después páralo con ctrl-C, y no lo dejes corriendo: el starter con bugs que guardaste en el paso 6 lee `periodHours` como segundos, así que su ventana de 720 "segundos" haría que a otro pull le toque doce minutos después del último — un pull que el programa rechaza con su error de período-no-transcurrido, una comisión base gastada en un rechazo garantizado, cada doce minutos, hasta que te des cuenta. Ese loop que quema comisiones es exactamente lo que arreglas en el Challenge.

**9. Verifica.** La barrera de la lección es `subscriptions/pull.test.ts`. Ejercita la matemática de la guarda offline, después lee el libro mayor del backoffice y demuestra que tu pull aterrizó ahí exactamente una vez. Escríbelo ahora; no necesita nada más que el `fs` de Node y la guarda:

```typescript
// subscriptions/pull.test.ts: the lesson's gate. Guard math first, then the ledger.
import { existsSync, readFileSync } from "node:fs";
import { decidePull } from "./decide-pull";

const LEDGER_PATH = "../backoffice/orders.jsonl"; // the file the live receiver (main.ts) writes
const HOUR = 3600;

function assert(condition: boolean, message: string): void {
  if (!condition) {
    console.error(`FAIL: ${message}`);
    process.exit(1);
  }
}

// 1. The two documented unit-and-clock bugs, as assertions.
// Args, in order: active, expiresAtTs, lastChargedTs, periodHours, now.
assert(decidePull(true, 0, 900_000, 24, 900_000 + 24 * HOUR).reason === "due", "boundary is inclusive");
assert(decidePull(true, 0, 900_000, 24, 900_000 + 23 * HOUR).reason === "too-early", "window is seconds");
assert(decidePull(true, 950_000, 900_000, 24, 960_000).reason === "expired", "expiry wins");
assert(decidePull(false, 0, 900_000, 24, 999_999).reason === "canceled", "canceled first");

// 2. The ledger: step 6's pull is in there, exactly once.
assert(existsSync(LEDGER_PATH), `no ledger at ${LEDGER_PATH}`);
const rows = readFileSync(LEDGER_PATH, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line) as { kind?: string; signature: string });
const pulls = rows.filter((row) => row.kind === "subscription-pull");
assert(pulls.length > 0, "no subscription-pull row in the orders ledger");
assert(
  new Set(pulls.map((row) => row.signature)).size === pulls.length,
  "a pull signature was written twice",
);

console.log("period-window: due -> pull landed in backoffice ledger (invoice reconciled)");
```

Córrelo desde la carpeta `subscriptions/` — el directorio de trabajo que cada comando de este lab ha asumido desde el `cd subscriptions` del paso 1, y el que la ruta relativa al libro mayor de la barrera (`../backoffice/orders.jsonl`) toma como referencia:

```bash
npx tsx pull.test.ts
```

Salida esperada, textual:

```
period-window: due -> pull landed in backoffice ledger (invoice reconciled)
```

Espéralo en rojo en la primera corrida, y espéralo por una razón nombrada: con el starter que guardaste en el paso 6, la segunda aserción falla, porque una guarda que trata `periodHours` como segundos declara que a una suscripción de 23 horas ya le toca. Esa línea roja es la costura entre el lab y el Challenge, exactamente como el TODO de la guarda del crank la lección pasada; la barrera se pone verde cuando arreglas los dos bugs de abajo.

## Challenge

La guarda de la ventana de período que importaste en el lab es la pieza solo, y el starter que te entrego contiene, a propósito, exactamente los dos bugs documentados de unidades y de reloj de la teoría. La función toma sus cinco entradas como escalares posicionales simples, en el orden en el que los campos importan, `active, expiresAtTs, lastChargedTs, periodHours, now`, que es también exactamente cómo la va a llamar el calificador (y el lab). Una costura entre las dos superficies: la copia de este starter que tiene el widget de código lleva las declaraciones desnudas, porque el calificador las ejecuta sin sintaxis de módulo, mientras que el archivo que guardas se queda con las palabras clave `export` de abajo para que los imports resuelvan. Guárdalo como `subscriptions/decide-pull.ts`, el módulo que importan tanto `04-pull.ts` como la barrera:

```typescript
export interface PullDecision {
  shouldPull: boolean;
  reason: string; // "due" when pulling, else why it was held
  nextEligibleTs: number; // earliest Unix second a pull may fire (0 if N/A)
}

export function decidePull(
  active: boolean, // false once CancelSubscription has run
  expiresAtTs: number, // Unix seconds; 0 = never expires
  lastChargedTs: number, // Unix seconds of the previous successful pull
  periodHours: number, // plan cadence, in HOURS (as published on the plan)
  now: number, // current chain time, Unix seconds
): PullDecision {
  if (!active) {
    return { shouldPull: false, reason: "canceled", nextEligibleTs: 0 };
  }

  // BUG: periodHours is treated as seconds, and expiry is never checked.
  const periodS = periodHours;
  const nextEligibleTs = lastChargedTs + periodS;

  if (now < nextEligibleTs) {
    return { shouldPull: false, reason: "too-early", nextEligibleTs };
  }

  return { shouldPull: true, reason: "due", nextEligibleTs };
}
```

Arréglalo para que todo lo siguiente se cumpla:

- Una suscripción inactiva devuelve `shouldPull: false` con razón `canceled` y `nextEligibleTs: 0`, antes de que corra cualquier otra comprobación.
- Una suscripción que está en o pasada un `expiresAtTs` distinto de cero devuelve razón `expired` con `nextEligibleTs: 0`, incluso cuando su ventana de período dice que a un pull le toca.
- La ventana se mide en segundos (`periodHours * 3600`), así que un plan de 24 horas no vuelve a cobrar dentro del día. `nextEligibleTs` reporta esa frontera basada en segundos en cada decisión que tiene una, así que un pull retenido le dice a tu panel de ops exactamente cuándo se va a disparar.
- `expiresAtTs === 0` nunca se lee como "vencido en la epoch"; cero quiere decir sin vencimiento, punto.
- Justo en `lastChargedTs + period`, el pull está `due`. Frontera inclusiva; un off-by-one acá retrasa cada cargo un tick del crank para siempre.

El orden importa: primero la cancelación, después el vencimiento, después la ventana. Pregúntate por qué antes de aceptarlo. El vencimiento de una suscripción cancelada no tiene sentido, y la ventana de una suscripción vencida no tiene sentido; cada comprobación solo tiene sentido en los sobrevivientes de la anterior. Equivócate en el orden y tus RAZONES de rechazo mienten incluso cuando tus decisiones de rechazo están bien, y en la lección de dunning esas razones se vuelven entradas de una máquina de estados, así que las mentiras se ponen caras.

Una nota de continuidad, porque la lección pasada congeló dos cadenas de razón, `delegate-revoked` e `insufficient-allowance`, y prometió que el resto de este módulo las mantendría con sentido. Sobreviven, una capa más abajo. `delegate-revoked` ahora nombra un evento más raro y más deliberado: el suscriptor revocó su Subscription Authority, el slot quedó vacante, y todo plan debajo está muerto con ella. `insufficient-allowance` se colapsa en su única causa que queda, una ATA que no puede cubrir el pull, porque la aprobación de la SA misma nunca se queda corta. Las tres razones de `decidePull` se suman a ese vocabulario en vez de reemplazarlo: tu guarda habla antes de que exista una transacción, la capa de transferencia habla cuando un pull falla de todos modos, y la máquina de dunning de la próxima lección consume los dos conjuntos como estados de entrada.

![decidePull comprueba primero la cancelación, después el vencimiento donde expiresAtTs en cero quiere decir nunca, después una ventana de período convertida de horas a segundos, con una razón de rechazo en cada salida.](assets/v09-flowchart.webp)

Acepta, en devnet, la barrera completa: plan creado, usuario suscrito, un pull conciliado como una fila de factura en el libro mayor de pedidos, y la guarda rechazando tanto un pull demasiado frecuente (`too-early`) como una suscripción vencida (`expired`). Tu evidencia es una dirección de plan, una firma de subscribe, una firma de pull cuyo id de factura aparece en el libro mayor, y las dos razones de rechazo impresas por tus pruebas.

## Checkpoint, y lo que el club ahora puede hacer

Si `pull.test.ts` está en rojo, la falla es casi con seguridad una de tres, en mi experiencia en este orden: la guarda comparó horas con segundos (tu plan de 720 horas calcula una ventana de 720 segundos; el error de matemática es enorme, lo que irónicamente lo hace fácil de detectar en un log), el caso de vencimiento en cero cortocircuitando todo a `expired`, o la ruta del libro mayor apuntando a un archivo nuevo en vez del `orders.jsonl` del backoffice (la prueba no encuentra ninguna factura porque escribiste un segundo libro mayor; un club, un libro mayor). Y si las llamadas on-chain en sí están rechazando con el error 133 o 134, vuelve a la comprobación del binario de devnet en el paso 3 del lab antes de que debuguees tu propio código; no puedes arreglar un desfase de versiones del lado del cliente.

Cuando esté en verde, sé preciso sobre lo que ahora tienes, porque es más que el crank de la lección pasada con mejor branding. Wavelength le factura a cualquier cantidad de suscriptores desde UNA sola aprobación consentida cada uno, sobrevive a que sus suscriptores se sumen a los planes de otros comercios, rechaza los cargos demasiado frecuentes y los caducados dos veces (una en tu guarda, una on-chain), y asienta cada pull en el mismo libro mayor de exactamente una vez que concilia los checkouts de la tienda. Ingresos recurrentes no custodiales con un rastro de auditoría. Montones de sistemas en producción se entregan con menos.

El club ahora le factura a muchos suscriptores de forma no custodial y cada pull aterriza en el libro mayor. Pero un pull puede fallar por razones que ninguna guarda predice: una ATA vacía el día de la facturación, un plan caducado, y no puedes reintentar hasta entrar en una billetera que no controlas. Lo que pasa después de un cargo fallido es una disciplina propia. La próxima lección: dunning como máquina de estados, y una mirada honesta a quién más en este mercado corre de verdad la facturación de suscripciones de esta manera. Trae las razones de rechazo; están a punto de volverse estados.
