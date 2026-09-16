# Ops de emisor: auditor, registro y supply confidencial

## Resumen

En m04-l1 construiste el modelo mental, los compromisos de sobre sellado, las tres pruebas, el auditor key opcional, pero no escribiste código. Aquí es donde lo configuras: lo primero confidencial que construyes de verdad. Vas a crear una variante confidencial de SPROUT que lleva ConfidentialTransferMint con una clave ElGamal de auditor real y una política de aprobación manual, todo desde instrucciones crudas de `@solana-program/token-2022`; vas a conocer el registro ElGamal que aprovisiona cuentas sin una firma del dueño por cuenta; vas a correr un depósito y una transferencia confidencial en devnet y vas a ver cómo se extiende por varias transacciones dependientes; y vas a ramificarte hacia ConfidentialMintBurn, la extensión que vuelve confidencial el supply mismo. El repliegue: la configuración del mint, la generación de claves y el flujo de depósito están trabajados de punta a punta; los parámetros de auditor y de auto-approve más el paso ConfigureAccountWithRegistry son problemas de completar que llenas tú (el del registro como práctica de forma que no puede ejecutarse sin una cuenta de registro del lado de Rust, un límite que el lab declara sin rodeos); la rama de supply confidencial y su demostración son tuyas en solitario, con un camino de degradación diseñado si el gate de pruebas de tu cluster está apagado. Ese camino de degradación es parte del plan y no una disculpa, y lo vas a oír dicho en voz alta antes de que lo necesites.

Antes de todo eso, demuestra que la maquinaria en la que te vas a apoyar está realmente desplegada. Las pruebas de la lección pasada no se verifican solas; lo hace un programa nativo dedicado. Treinta segundos, sin más setup que `curl`:

```bash
curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '
  {"jsonrpc":"2.0","id":1,"method":"getAccountInfo",
   "params":["ZkE1Gama1Proof11111111111111111111111111111",{"encoding":"base64"}]}' \
  | python3 -c "import sys,json; v=json.load(sys.stdin)['result']['value']; print('executable:', v['executable'], '| owner:', v['owner'])"
```

Deberías ver `executable: True | owner: NativeLoader1111111111111111111111111111111`. Corrí esto contra devnet y mainnet esta mañana (2026-08-22) y las dos respondieron lo mismo: el ZK ElGamal Proof Program está presente y es ejecutable en los dos clusters. La presencia no es toda la historia, y vamos a llegar a los feature gates que todavía pueden apagar la verificación, pero acabas de confirmar que el verificador existe donde estás por desplegar. Eso es más diligencia debida de la que jamás hace la mayoría de los tutoriales de transferencias confidenciales.

Tu CFO quiere que los salarios sean confidenciales. Los auditores igual necesitan conciliar cada centavo. El regulador quiere una ventana que nadie más tiene. "Confidencial" y "auditable" suenan a opuestos, y el modelo de la lección pasada ya te dijo que no lo son: una clave ElGamal opcional en el mint, y cada transferencia cifra calladamente una segunda copia de su monto solo para ese auditor. Hoy pones esa clave con tus propias manos.

## Qué configura de verdad un emisor

Todo en esta lección cuelga de un solo struct cuya forma ya viste. Cuando un mint de Token-2022 (programa `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`, la misma dirección que todos los mints que construiste) lleva la extensión ConfidentialTransferMint, su entrada TLV tiene exactamente tres campos: un `authority` que puede actualizar esta configuración y aprobar cuentas, un flag `auto_approve_new_accounts` y un `auditor_elgamal_pubkey` opcional. Tres campos. Toda la superficie de emisor de las transferencias confidenciales son esas tres decisiones, más una segunda extensión para el supply. Tomémoslos en el orden en que te van a morder.

### El auditor key: visibilidad dirigida, en todo el mint, para siempre

El auditor es una sola pubkey ElGamal global y opcional en el mint. Ponla, y cada transferencia confidencial de ese token tiene que cifrar además su monto bajo la clave del auditor. Recuerda la prueba del medio de la lección pasada, la prueba de validez de ciphertext agrupado: cubre tres handles, remitente, receptor y auditor, y el programa no va a aceptar una transferencia cuyo ciphertext de auditor falte o esté mal formado cuando hay un auditor configurado. No hay opt-out por transferencia. No hay un flag de "esta es sensible, sáltate el auditor". El mint decide, y el mint decide para todos, en cada transferencia, hasta que el authority cambie el ajuste.

![Una transferencia confidencial produce ciphertexts del monto bajo las claves del remitente, del receptor y del auditor, así que el auditor puede descifrar cada transferencia mientras el público solo lee ciphertext.](assets/v01-diagram.webp)

¿Por qué una sola clave global en vez de consentimiento por transferencia? Razónalo desde el modo de falla. Si la divulgación fuera por transferencia, la parte que intenta esconder algo simplemente no divulgaría, y un auditor que solo ve las transferencias que la gente eligió mostrarle no es un auditor, es una audiencia. La conciliación solo significa algo cuando la cobertura es total, así que el diseño pone la decisión donde la cobertura es total: en el mint, en tiempo de configuración, impuesta por la misma maquinaria de pruebas que impone todo lo demás. El costo es igual de estructural. Convertiste "nadie puede leer los montos" en "nadie puede leer los montos salvo quien tenga una clave secreta específica," y esa clave es ahora el secreto más valioso de tu stack de compliance. Rota el ajuste y los ciphertexts viejos no se vuelven a cifrar; quien tuvo la clave vieja puede descifrar la historia para siempre. Este es el canje en el corazón de la lección: la visibilidad del regulador se compra con un asterisco permanente, en todo el mint, sobre la promesa de privacidad, y la jugada honesta es escribir ese asterisco en tus docs en vez de esperar que nadie pregunte.

Una verificación con la realidad antes de que te encariñes con la historia del CFO. No hay usuarios de producción nombrados de transferencias confidenciales hoy. PYUSD, el despliegue más institucional de Token-2022 en la blockchain, ya viene con la extensión confidentialTransferMint en su mint justo ahora, y on-chain está configurada y dormida: el slot confidencial presente y sin usar, la stablecoin regulada emblemática manteniendo la puerta abierta sin pasar por ella. Así que enséñate esto como yo te lo estoy enseñando: con capacidad de emisor, no probado por emisores. Estás aprendiendo las ops porque los rieles están vivos y la puerta está abierta, no porque una docena de tesorerías ya haya pasado por ella.

### auto_approve_new_accounts: el flag de portero

El segundo campo es un booleano con un departamento de cumplimiento adentro. Cuando `auto_approve_new_accounts` es true, cualquier tenedor que configure su cuenta para transferencias confidenciales puede empezar a usarla de inmediato. Cuando es false, cada cuenta recién configurada queda sin aprobar, y todas las operaciones confidenciales sobre ella fallan hasta que el authority de transferencia confidencial del mint apruebe explícitamente esa cuenta. False es la forma KYC: nadie mueve montos ocultos hasta que el emisor haya dado el visto bueno a esa cuenta específica. Nuestra variante de SPROUT lo pone en false, en parte porque esa es la postura de emisor que enseña esta lección, y en parte porque te obliga a construir el paso de aprobación tú mismo, que resulta más interesante de lo que suena. La CLI que vas a instalar en el lab no tiene ningún comando de confidential-approve. La instrucción cruda existe, el tooling no se puso al día, y vas a salvar esa brecha con unas cuarenta líneas de TypeScript.

### El registro: firma una vez, aprovisiona para siempre

Ahora el problema de aprovisionamiento. Configurar una cuenta para transferencias confidenciales normalmente exige que el dueño de la cuenta firme, porque la configuración incluye la pubkey ElGamal del dueño y una prueba de que él la controla. Para un hacker que configura su propia billetera, bien. Para un emisor que aprovisiona diez mil cuentas de empleados, una firma del dueño por cuenta es una pesadilla de ops: cada lote de aprovisionamiento necesita a cada dueño en línea, firmando, en orden.

El programa de registro ElGamal existe para romper esa dependencia. Viene en el mismo repositorio que Token-2022, y el flujo es: un dueño registra su pubkey ElGamal en una cuenta de registro una vez, con una prueba de validez, firmando una vez. De ahí en adelante, cualquiera, un backend, un cron job, el servicio de aprovisionamiento del emisor, puede llamar a la instrucción `ConfigureAccountWithRegistry` en Token-2022, apuntando a la cuenta de registro en vez de juntar una firma nueva del dueño. La cuenta de registro es el consentimiento permanente; el programa de token lee la clave de ahí y configura la cuenta. Una firma amortizada sobre cada acción futura de aprovisionamiento.

![Diagrama de flujo que contrasta el camino de firma del dueño por cuenta con el camino del registro, donde un solo registro habilita el aprovisionamiento sin firma vía ConfigureAccountWithRegistry, y los dos convergen en la aprobación manual.](assets/v02-flowchart.webp)

La letra chica, porque decide qué puedes construir hoy: crear la entrada de registro en sí exige una prueba de validez de pubkey, y esa prueba la puedes generar en JavaScript puro — `@solana/zk-sdk`, un peer declarado del mismísimo cliente de token que fija esta lección, te va a entregar `new PubkeyValidityProofData(new ElGamalKeypair())` sin una línea de Rust. Lo que le falta al stack de JS en nuestros pines es la otra mitad: un builder de cliente para las instrucciones create/update del propio programa de registro. El lado Token-2022 del flujo, la instrucción `ConfigureAccountWithRegistry` que consume una cuenta de registro existente, tiene un builder de primera clase en el cliente de JS, y en el lab llenas esa llamada al builder para que la forma te quede en los dedos; te aviso desde ahora que ahí se queda en dique seco, porque sin una cuenta de registro creada no puede ejecutarse, y el lab lo dice en vez de disimular. El lado de creación del registro deberías conocerlo por nombre, `spl-elgamal-registry` en el repositorio de token; hasta que llegue un builder de JS para sus instrucciones, la ruta pragmática es su CLI de Rust — una brecha de tooling de cliente, no una criptográfica. Esta asimetría, donde la capa de instrucciones está completa y el tooling de cliente la cubre de forma desigual, es la textura recurrente de las transferencias confidenciales, y es exactamente por lo que esta lección se guarda un helper de Rust en el bolsillo de atrás.

### La realidad multi-transacción

Derivaste esta coreografía completa la lección pasada; aquí está en un solo aliento, porque hoy la vuelves a leer desde la silla del emisor, donde se convierte en líneas de presupuesto. Una transferencia confidencial hoy es una coreografía pequeña. Cada una de las tres pruebas de la lección pasada la verifica el ZK ElGamal Proof Program que sondeaste en la apertura, y las pruebas son demasiado grandes para viajar juntas en una sola transacción con la transferencia misma. Así que el flujo queda: crea una cuenta de contexto para una prueba, verifica la prueba dentro de ella, repite por prueba, después ejecuta la instrucción de transferencia que lee esos contextos verificados, después cierra las cuentas de contexto para recuperar su rent. Varias transacciones dependientes, ordenadas, cada una capaz de fallar de forma independiente.

![Línea de tiempo de una transferencia confidencial donde varias transacciones crean cuentas de contexto y verifican cada prueba antes de que la transferencia se ejecute y los contextos se cierren para recuperar rent.](assets/v03-timeline.webp)

Dos líneas de presupuesto caen directo de ese dibujo. Primero, las cuentas de contexto son rent que adelantas y recuperas, por prueba, por transferencia; a escala de flota ese vaivén es un ítem real y tu runbook de ops debería tratar el cierre de contextos como higiene obligatoria, no como limpieza. Segundo, la actualidad: el futuro de una sola transacción es real pero no está aquí, y "aquí" ahora depende de qué cluster quieras decir. Este es el mismo gate de `enable_tx_v1` del que la lección pasada te pasó la dirección, que vino en Agave 4.2 con un sobre lo bastante grande para llevar las pruebas adentro. El 2026-09-06 no tenía ninguna cuenta detrás en mainnet y estaba activo en devnet desde el slot 492,480,000. Construye para el flujo multi-transacción, porque mainnet es donde están tus usuarios, y vuelve a sondear el gate tú mismo la semana en que salgas a producción — un `solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url mainnet-beta` lo resuelve.

### Supply confidencial: la rama ConfidentialMintBurn

Todo hasta aquí esconde los montos de las transferencias. El supply sigue siendo público: cualquiera puede leer cuántos tokens existen, y cada acuñación y cada quema mueven ese número público. Para casi todos los tokens eso está bien y hasta es deseable. Para un emisor cuyos eventos de acuñación son sensibles en sí mismos, piensa en una tesorería que no quiere volúmenes de redención legibles en tiempo real, hay una segunda extensión: ConfidentialMintBurn.

Agrega cuatro campos al mint: un `supply_elgamal_pubkey` que cifra el supply, un supply descifrable que el emisor puede volver a leer con una clave AES, el supply confidencial cifrado en sí, y un acumulador de quema pendiente. Las quemas llegan en forma pendiente y una instrucción `apply_pending_burn` las pliega dentro del supply confidencial; la clave de supply se puede rotar con `rotate_supply_elgamal_pubkey` cuando la custodia de ese secreto pasa a otras manos. Y compone bajo reglas que ya son tuyas. Tu validador de check-combo de m01-l4 codifica la regla 3: ConfidentialMintBurn exige ConfidentialTransferMint en el mismo mint. También codifica la regla 5, la rara: NonTransferable más ConfidentialTransferMint es inválido a menos que ConfidentialMintBurn también esté presente. Un token confidencial soulbound solo tiene sentido si las operaciones de supply también son confidenciales, y el programa rechaza la solución a medias.

![Dos diagramas de mint que muestran el SPROUT confidencial principal con un auditor configurado y el mint de la rama en solitario agregando los cuatro campos de supply de ConfidentialMintBurn, anotados con las reglas de combo y la advertencia de PermanentDelegate.](assets/v04-diagram.webp)

Una trampa que desarmar antes de que diseñes nada en esta rama. En m02-l2 demostraste que PermanentDelegate se cuela por delante de CpiGuard y puede barrer cualquier cuenta de su mint. Los saldos confidenciales son donde se detiene ese poder: el delegado permanente no funciona sobre saldos confidenciales, punto. Un emisor que pensaba hacer clawback-and-burn vía delegado no tiene equivalente confidencial; el control del supply en el mundo confidencial corre por las instrucciones propias de ConfidentialMintBurn, firmadas por la autoridad de mint, o no ocurre. No eches mano del delegado como palanca de supply confidencial. No lo es.

### Dónde corre esto, y las cicatrices del programa

Plan de cluster, diseñado desde el principio y no postergado. Objetivo primario: devnet, donde el gate de pruebas está activo, así que tus depósitos y transferencias deberían verificar. Esa no es una afirmación de la documentación que te estoy pasando; leí las tres cuentas de feature de zk-ElGamal en los dos clusters el 2026-08-22 y cada una está activada, devnet y mainnet por igual. Vas a ver cómo en un momento. Secundario: un fork de surfpool, que usas desde m01 y que es ideal para la mitad de configuración del lab. Y la rama honesta: si el gate de pruebas de zk-ElGamal está apagado en cualquier cluster al que apuntes, tus pruebas no van a verificar, y el lab degrada a configurar-la-extensión más demostrar-su-estado-on-chain. Mismo mint, mismo auditor, mismo script de verificación; lo único que pierdes es la transferencia en vivo, y lo dices en vez de fingirlo. Esa es la jugada diseñada. Lo que nunca haces es forzar un gate encendido en un fork local y presentar el resultado como evidencia de grado mainnet.

¿Por qué tanta ceremonia alrededor de un solo feature gate? Porque este programa en particular tiene cicatrices, y a diferencia de la mayoría de las cicatrices, estas tienen timestamps que puedes leer tú mismo. El conjunto de features de Agave lleva tres gates cuyos nombres cuentan toda la historia: `zk_elgamal_proof_program_enabled`, `disable_zk_elgamal_proof_program`, `reenable_zk_elgamal_proof_program`. Cada gate es una cuenta; uno activado guarda el slot en el que se disparó, y `getBlockTime` convierte ese slot en una fecha. Los comandos, para que "sondear el gate" nunca sea vaguedad: `solana feature status --url mainnet-beta` sin argumento lista cada gate que la CLI conoce, dirección, estado y slot de activación incluidos, así que `solana feature status --url mainnet-beta | grep -i elgamal` imprime las tres filas en una línea de shell; toma un slot de esa salida y `solana block-time <slot>` lo fecha. Cambia a `--url devnet` para hacerle la misma pregunta al otro cluster. Ese par de comandos es todo el sondeo, y es lo que el camino de degradación del lab quiere decir con "probe the gate accounts". Hazlo en mainnet y el arco es crudo: habilitado 2025-01-23, pausado 2025-06-19 por un arreglo de seguridad, rehabilitado 2026-06-04. El verificador sobre el que se apoya toda esta lección estuvo apagado unos once meses. Este no es un subsistema de demo que nunca se tocó bajo fuego. Es un programa de producción con historial real de incidentes, y la maquinaria de pausa es lo bastante estructural como para haberse usado buena parte de un año. Respétalo en tu arquitectura: cualquier sistema que construyas sobre transferencias confidenciales debería tolerar que el programa de pruebas se pause otra vez, que es un argumento más para mantener tu camino de degradación ensayado en vez de teórico.

![Línea de tiempo fechada de los tres feature gates ZK ElGamal que muestra la activación de enero de 2025, la pausa de seguridad en junio de 2025 y la rehabilitación de junio de 2026, con un sondeo reciente que confirma el verificador en vivo.](assets/v05-timeline.webp)

Esa es la superficie de teoría: tres campos en una extensión, un flag de portero, un registro que amortiza el consentimiento, una transferencia que es una coreografía, una rama de supply con sus propias claves, y un verificador con historia. Hora de cablear todo.

## Lab: cablea el auditor, el registro y la rama de supply

El artefacto es `confidential-sprout`: una variante de SPROUT cuyo mint lleva ConfidentialTransferMint con una pubkey ElGamal de auditor configurada y `autoApproveNewAccounts: false`, un keypair de cifrado registrado, un depósito confidencial aplicado, y una transferencia confidencial completada en devnet, o la demostración de degradación ya declarada si el gate de tu cluster está apagado. Una segunda rama habilita ConfidentialMintBurn. El criterio del final es el mismo que usa siempre el curso: `npx tsx verify-confidential.ts` tiene que leer la extensión y el auditor de vuelta desde la blockchain.

1. Haz el scaffold de `labs/m04-l2` e instala el toolchain de JS. La misma lógica de pines que m02-l1, re-verificada hoy: fija el major de kit contra el que hacen peer tus clientes, y `@solana-program/token-2022@0.15.0` es el minor actual que hace peer con kit ^7.0.0 (0.16.0 saltó a ^8), con `@solana-program/system@0.13.0` haciéndole juego. Reinstalé y verifiqué los tipos de este trío exacto el 2026-09-05; corre `npm view @solana-program/token-2022 peerDependencies` tú mismo el día que hagas el scaffold, porque esta matriz se mueve mes a mes.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.23.12 typescript@5.9.3
```

2. Dos herramientas más, las dos del mundo Rust. La CLI de `spl-token` con la que sondeaste por primera vez en m01-l4; como advirtió esa lección, viene incluida con algunas instalaciones de Agave (el one-liner de m01-l3, `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`) y con otras no, y `cargo install spl-token-cli` te consigue el mismo binario por separado. Revisa la versión, porque la superficie confidencial cambió entre releases (el build de release de Agave instala la CLI sin fijar, así que tu copia incluida es lo que fuera actual el día que se cortó ese release):

```bash
spl-token --version   # spl-token-cli 5.6.1, the crates.io release as of 2026-08-22
```

   Ahora lee qué cubre 5.6.1 de verdad, porque recorrí su tabla de subcomandos línea por línea y el mapa de cobertura es lo más instructivo de este lab. La CLI maneja por completo el flujo del tenedor, cargado de pruebas: `configure-confidential-transfer-account`, `deposit-confidential-tokens`, `apply-pending-balance`, `withdraw-confidential-tokens` y `transfer --confidential` generan todas sus pruebas internamente. El flujo del emisor es otra historia. No hay comando de confidential-approve para la política manual (`spl-token approve` es el comando de delegación ordinario y no tiene nada que ver con esto). No hay comando de registro. No hay soporte de ConfidentialMintBurn en ningún lado, ni en `create-token` ni tampoco en la visualización de cuenta, donde el fuente todavía lleva una nota para agregarlo después. El conjunto de instrucciones crudas va por delante de la CLI emblemática, y esa brecha no es un inconveniente, es la razón por la que esta lección enseña instrucciones crudas. Un emisor que solo puede hacer lo que hace la CLI no puede correr hoy un mint de aprobación manual.

![Comparación que muestra que spl-token-cli 5.6.1 cubre el flujo del tenedor (configurar, depositar, aplicar, retirar, transferencia confidencial) mientras que approve, el aprovisionamiento del registro y todas las operaciones de ConfidentialMintBurn existen solo como instrucciones crudas.](assets/v06-comparison.webp)

   La segunda herramienta genera claves de cifrado. Crea un helper de Rust diminuto al lado de tu carpeta de lab (rustup instala cargo si nunca lo hiciste: `curl https://sh.rustup.rs -sSf | sh`):

```bash
cargo new ct-keygen && cd ct-keygen
cargo add solana-zk-sdk@7.0.1 bs58@0.5.1 base64@0.23.1
# Pins are what I ran on 2026-08-22. If cargo add refuses one (yanked, or the
# line moved), drop that crate's pin and take the current release; nothing in
# this helper depends on an exact version.
```

   Después `src/main.rs`. Este es el programa completo, y la contabilidad honesta de por qué existe: nada aquí exige Rust en sentido estricto — `@solana/zk-sdk` va a acuñar el mismo keypair ElGamal y el mismo cero cifrado con AES por `AeKey` en JS puro, y recodificar son dos líneas — pero los valores alimentan tanto a la CLI (cuyo texto de ayuda admite que hoy solo acepta base64, "más métodos en una versión futura") como al cliente de kit (que quiere base58), y un binario pequeño que imprime cada clave en cada codificación, construido sobre el mismo crate `solana-zk-sdk` en el que confía el programa on-chain, es la herramienta con forma de ops para eso. ¿Prefieres TypeScript? Pórtalo contra `@solana/zk-sdk` y conserva el formato de impresión:

```rust
// ct-keygen: derive the encryption keys issuer ops needs, print every encoding.
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use solana_zk_sdk::encryption::{auth_encryption::AeKey, elgamal::ElGamalKeypair};

fn main() {
    // 1. An ElGamal keypair (auditor key, or supply key for ConfidentialMintBurn).
    let elgamal = ElGamalKeypair::new_rand();
    let pubkey_bytes: [u8; 32] = (*elgamal.pubkey()).into();

    println!("elgamal pubkey (base64, for spl-token): {}", B64.encode(pubkey_bytes));
    println!("elgamal pubkey (base58, for kit code):  {}", bs58::encode(pubkey_bytes).into_string());

    // 2. An AES key + the encryption of zero (the initial decryptable supply).
    let aes = AeKey::new_rand();
    let zero_bytes = aes.encrypt(0).to_bytes();
    let listed: Vec<String> = zero_bytes.iter().map(|b| b.to_string()).collect();
    println!("decryptableSupply(0) bytes: [{}]", listed.join(", "));
}
```

   `cargo run` imprime tres líneas. Exporta la pubkey base58 como `AUDITOR_ELGAMAL_PUBKEY` para el paso siguiente, y guarda las otras dos líneas para la rama de supply. Una nota de honestidad sobre el manejo de claves: `new_rand` está bien para un lab. Un auditor de producción deriva su keypair ElGamal de forma determinista desde una firma de billetera para poder recuperarlo, y la clave secreta de aquí es la joya de la corona que discutimos; trata la impresión en consecuencia y tira estas claves de lab después.

3. Ahora el mint. Crea `create-confidential-sprout.ts`, y dale al problema de completar lo suyo: escribe el archivo con los tres valores de configuración en blanco, decídelos tú, y después contrasta con la referencia llena de abajo. Las tres líneas que son tuyas son exactamente los tres campos de la sección de teoría: el authority, la política de aprobación y el auditor. Todo lo demás es el mismo baile de crear-cuenta-y-después-inicializar que corres desde m02-l1, con la extensión inicializada antes de `InitializeMint`, como siempre.

```ts
// create-confidential-sprout.ts: a SPROUT variant that carries ConfidentialTransferMint.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  createKeyPairSignerFromBytes,
  sendAndConfirmTransactionFactory,
  address,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  getSignatureFromTransaction,
  type Instruction,
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  extension,
  getMintSize,
  getInitializeConfidentialTransferMintInstruction,
  getInitializeMintInstruction,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");
const rpcSubscriptions = createSolanaRpcSubscriptions(
  process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com"
);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

// The auditor's ElGamal pubkey, printed by ct-keygen in base58 form.
const AUDITOR_ELGAMAL_PUBKEY = address(process.env.AUDITOR_ELGAMAL_PUBKEY!);

async function main() {
  const payer = await createKeyPairSignerFromBytes(
    new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, "utf8")))
  );
  const mint = await generateKeyPairSigner();

  // Size the account for the extension BEFORE the base mint layout.
  const confidentialTransferMint = extension("ConfidentialTransferMint", {
    authority: payer.address,            // fill 1: who updates config + approves accounts
    autoApproveNewAccounts: false,       // fill 2: manual policy, the KYC shape
    auditorElgamalPubkey: AUDITOR_ELGAMAL_PUBKEY, // fill 3: the regulator's window
  });
  const space = BigInt(getMintSize([confidentialTransferMint]));
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

  const instructions: Instruction[] = [
    getCreateAccountInstruction({
      payer,
      newAccount: mint,
      lamports: rent,
      space,
      programAddress: TOKEN_2022_PROGRAM_ADDRESS,
    }),
    // Extension init runs BEFORE InitializeMint, same order as every mint you have built.
    getInitializeConfidentialTransferMintInstruction({
      mint: mint.address,
      authority: payer.address,
      autoApproveNewAccounts: false,
      auditorElgamalPubkey: AUDITOR_ELGAMAL_PUBKEY,
    }),
    getInitializeMintInstruction({
      mint: mint.address,
      decimals: 6,
      mintAuthority: payer.address,
      freezeAuthority: payer.address,
    }),
  ];

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const tx = await pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(instructions, m),
    (m) => signTransactionMessageWithSigners(m)
  );
  assertIsTransactionWithBlockhashLifetime(tx);
  await sendAndConfirm(tx, { commitment: "confirmed" });

  console.log(`confidential SPROUT mint: ${mint.address}`);
  console.log(`signature: ${getSignatureFromTransaction(tx)}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   `AUDITOR_ELGAMAL_PUBKEY=<base58 from ct-keygen> npx tsx create-confidential-sprout.ts` y guarda la dirección de mint impresa; cada paso posterior la toma como argumento. Acuérdate de la restricción que no puedes ver en este archivo: la configuración de transferencia confidencial es solo de creación. No hay instrucción de retrofit. Un SPROUT vivo con tenedores nunca puede ganar esta extensión, que es por lo que este es un mint variante y por lo que la decisión pertenece a tu checklist de lanzamiento, no a tu backlog.

4. Demuestra que quedó. Crea `verify-confidential.ts`, el criterio de esta lección, y la interfaz por la que una lección posterior va a llamar a este artefacto:

```ts
// verify-confidential.ts: prove the extension and the auditor read back from chain.
import { createSolanaRpc, address } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");

async function main() {
  const mintAddress = address(process.argv[2] ?? process.env.MINT!);
  const mint = await fetchMint(rpc, mintAddress);

  const extensions =
    mint.data.extensions.__option === "Some" ? mint.data.extensions.value : [];
  const ct = extensions.find((e) => e.__kind === "ConfidentialTransferMint");
  if (!ct || ct.__kind !== "ConfidentialTransferMint") {
    console.error("FAIL: ConfidentialTransferMint not present on this mint");
    process.exit(1);
  }

  const auditor =
    ct.auditorElgamalPubkey.__option === "Some"
      ? ct.auditorElgamalPubkey.value
      : "none";
  console.log("ConfidentialTransferMint present");
  console.log(`auditorElgamalPubkey=${auditor}`);
  console.log(`autoApproveNewAccounts=${ct.autoApproveNewAccounts}`);

  const mintBurn = extensions.find((e) => e.__kind === "ConfidentialMintBurn");
  if (mintBurn && mintBurn.__kind === "ConfidentialMintBurn") {
    console.log(`ConfidentialMintBurn present; supplyElgamalPubkey=${mintBurn.supplyElgamalPubkey}`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   Corre `npx tsx verify-confidential.ts <mint>`. La condición de aprobación es exacta: `ConfidentialTransferMint present`, la línea de auditor repitiendo la clave base58 que generaste, y `autoApproveNewAccounts=false`. Si el auditor imprime `none`, tu fill 3 no llegó a la instrucción; vuelve a crear el mint, porque no hay forma de editar para salir de una omisión en tiempo de creación sobre algo desechable, y notar ese reflejo importa más que el SOL que gastas en eso.

![Salida esperada anotada del script de verificación: extensión presente, pubkey de auditor coincidiendo con la clave generada, auto-approve en false, y una cuarta línea opcional para la rama ConfidentialMintBurn.](assets/v07-annotated-code.webp)

5. Aprovisiona y aprueba una cuenta. La mitad del registro primero, porque es el problema de completar: crea `configure-with-registry.ts` y escribe tú mismo la llamada al builder, desde la descripción de la sección de teoría, antes de seguir leyendo. La llamada llena son tres direcciones y un payer, y el punto es lo que no está en ella, ningún firmante del dueño en ninguna parte:

```ts
import { getConfigureConfidentialTransferAccountWithRegistryInstruction } from "@solana-program/token-2022";

const ix = getConfigureConfidentialTransferAccountWithRegistryInstruction({
  token: tokenAccount,          // the ATA to configure
  mint: mintAddress,            // your confidential SPROUT
  elgamalRegistry: registryAccount, // the standing consent, created once via spl-elgamal-registry
  payer,                        // funds the account reallocation; NOT the owner
});
```

   Cablea eso en el mismo esqueleto de pipe-and-send que cada transacción de este curso. Se ejecuta contra cualquier cuenta de registro que exista; crear una es la tarea del lado de Rust nombrada en la sección de teoría, así que para la cuenta de hoy tomamos el camino de firma del dueño que la CLI automatiza, y tu diseño de aprovisionamiento de flota se guarda el registro en el bolsillo. Configura tu propia ATA con la CLI (crea las claves ElGamal de la cuenta desde tu billetera y genera la prueba de validez internamente):

```bash
spl-token create-account <MINT> --url devnet
spl-token configure-confidential-transfer-account <MINT> --url devnet
```

   Configurado no es usable: pusiste la política en manual, así que esta cuenta ahora queda sin aprobar, y la CLI no tiene comando de confidential-approve. Salva la brecha tú mismo con `approve-account.ts`, el lado del emisor en el flag de portero:

```ts
// approve-account.ts: the manual-approval half of autoApproveNewAccounts=false.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromBytes,
  sendAndConfirmTransactionFactory,
  address,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
} from "@solana/kit";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getApproveConfidentialTransferAccountInstruction,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");
const rpcSubscriptions = createSolanaRpcSubscriptions(
  process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com"
);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

async function main() {
  const authority = await createKeyPairSignerFromBytes(
    new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, "utf8")))
  );
  const mint = address(process.argv[2]!);
  const owner = address(process.argv[3]!);
  const [token] = await findAssociatedTokenPda({
    mint,
    owner,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const ix = getApproveConfidentialTransferAccountInstruction({ token, mint, authority });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const tx = await pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(authority, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstruction(ix, m),
    (m) => signTransactionMessageWithSigners(m)
  );
  assertIsTransactionWithBlockhashLifetime(tx);
  await sendAndConfirm(tx, { commitment: "confirmed" });
  console.log(`approved ${token} for confidential transfers on ${mint}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   `npx tsx approve-account.ts <MINT> $(solana address)` y tu cuenta cruza de configurada a usable. Fíjate que la instrucción no necesita ninguna prueba, solo la firma ordinaria del authority; la aprobación es un acto de política, no uno criptográfico, que es exactamente por lo que nos salió barato construirlo y un poco vergonzoso que ningún tooling lo traiga.

6. Mueve dinero oculto. El flujo de depósito-y-transferencia es el script provisto, `confidential-flow.sh`; prepara un destinatario y lleva la cuenta del destinatario por configurar-y-aprobar, después corre el depósito, el apply del saldo pendiente y una transferencia confidencial. Tu propia cuenta de remitente no la toca: el paso 5 la configuró y la aprobó, y el script asume que ese trabajo está hecho, así que corre el paso 5 primero o la transferencia de abajo falla por una razón que no tiene nada que ver con el gate de pruebas. El camino de degradación está escrito adentro, en voz alta, en el único paso que puede toparse con el gate:

```bash
#!/usr/bin/env bash
# confidential-flow.sh <MINT>: deposit + one confidential transfer, degrade path included.
set -euo pipefail
MINT=$1
URL=${RPC_URL:-devnet}

# A recipient wallet, funded enough for rent.
solana-keygen new --no-bip39-passphrase --silent --outfile recipient.json
RECIPIENT=$(solana-keygen pubkey recipient.json)
solana transfer "$RECIPIENT" 0.1 --allow-unfunded-recipient --url "$URL"

# Recipient's account: create, configure, approve (our raw-instruction bridge).
spl-token create-account "$MINT" --owner "$RECIPIENT" \
  --fee-payer ~/.config/solana/id.json --url "$URL"
spl-token configure-confidential-transfer-account "$MINT" --owner recipient.json \
  --fee-payer ~/.config/solana/id.json --url "$URL"
npx tsx approve-account.ts "$MINT" "$RECIPIENT"

# Fund the public side, then move 40 SPROUT behind the curtain.
spl-token mint "$MINT" 100 --url "$URL"
spl-token deposit-confidential-tokens "$MINT" 40 --url "$URL"
spl-token apply-pending-balance "$MINT" --url "$URL"

# The confidential transfer: several dependent transactions under the hood.
# Prerequisite: YOUR sender account was configured + approved in step 5 and
# funded by the deposit above; the script does not repeat that work.
if spl-token transfer "$MINT" 15 "$RECIPIENT" --confidential --url "$URL"; then
  echo "confidential transfer complete: amount hidden, auditor copy included"
else
  echo "DEGRADE PATH: the confidential transfer failed."
  echo "First rule out your own state: sender configured + approved (step 5),"
  echo "deposit landed, apply-pending-balance run. Only if all of that holds is"
  echo "the likely cause the cluster's zk-ElGamal gate; probe the gate accounts"
  echo "before blaming the cluster:"
  echo "  solana feature status --url $URL | grep -i elgamal"
  echo "Falling back to the designed proof: extension + auditor state, on-chain."
  npx tsx verify-confidential.ts "$MINT"
fi
```

   Corre `./confidential-flow.sh <MINT>` (después de `chmod +x confidential-flow.sh`) y mira el paso de transferencia: la CLI está haciendo en silencio toda la coreografía de cuentas de contexto que viene de la sección de teoría, creándolas, verificando tres pruebas, transfiriendo, cerrando. En devnet esto debería completarse. Si en cambio caes en la rama de degradación, no perdiste nada de lo que esta lección te califica: la habilidad de emisor era la configuración, y el fallback lo demuestra on-chain, dicho abiertamente. El depósito antes de la transferencia no es opcional, por cierto, y tampoco lo es `apply-pending-balance`: los depósitos llegan a un saldo pendiente y solo el paso de apply los vuelve gastables, un diseño de dos fases que vas a reconocer del modelo de la lección pasada. Saltarse el apply es el ticket número uno de "por qué mi saldo disponible es cero" en este flujo.

7. Envuélvelo en un solo criterio. `npx tsx verify-confidential.ts <mint>` es la prueba de aceptación de la lección, y de aquí en adelante otras lecciones van a asumir que un `confidential-sprout` significa exactamente lo que afirma este script: extensión presente, tu auditor, política manual. Córrelo una última vez y lee tu propio auditor key volviendo desde infraestructura de grado mainnet. Hace dos lecciones el auditor era un diagrama. Ahora es un valor que generaste, configuraste y puedes demostrar.

## Challenge

Primero la pieza calificada: `validateConfidentialConfig`, el chequeo previo que un emisor corre antes siquiera de construir `initializeConfidentialTransferMint`, para que un conjunto de extensiones malo muera en revisión en vez de revertir on-chain. El paso 3 llenó a mano una config que resultó ser legal; esta función es lo que convierte esa suerte en política. El calificador llama a tu función posicionalmente, cuatro escalares en este orden, y el tipo de retorno es esta forma de veredicto exacta — `ConfidentialConfig` es el nombre que el starter le da al veredicto *exterior*, y la config construida de tres campos viaja adentro:

```ts
type ConfidentialConfig = {
  ok: boolean;
  reason: string; // "ok" when valid, else the rejection slug
  config: {
    authority: string;
    autoApproveNewAccounts: boolean;
    auditorElGamalPubkey: string | null;
  } | null;
};

function validateConfidentialConfig(
  extensionList: string, // the mint's full extension set, pipe-separated:
  //                        'NonTransferable|ConfidentialTransferMint'
  authorityKey: string, // confidential-transfer authority ("" if unset)
  autoApprove: boolean, // auto_approve_new_accounts
  auditorKey: string | null, // auditor ElGamal pubkey, or null (optional)
): ConfidentialConfig;
```

El contrato de falla es un veredicto devuelto, nunca un throw: reporta un rechazo devolviendo `{ ok: false, reason: "<slug>", config: null }` con el slug como el string entero de reason, nada antepuesto, nada agregado. El calificador lee `result.ok` y `result.reason`, así que "la primera razón que reportas" quiere decir la razón del primer rechazo que devuelve tu función. Divide `extensionList` por `'|'` antes de razonar sobre ella; ese único string es cómo viaja todo el conjunto de extensiones por el calificador. Después impón los tres rechazos que ya conoces de esta lección, en este orden exacto, porque más de uno puede sostenerse a la vez: un `authorityKey` vacío es `missing-authority` primero, después un conjunto sin ConfidentialTransferMint es `confidential-transfer-mint-not-enabled`, después la regla de combo 5, NonTransferable más ConfidentialTransferMint sin ConfidentialMintBurn, es `nontransferable-confidential-requires-mintburn`. La regla 3 no necesita código propio: un conjunto que lleva ConfidentialMintBurn sin ConfidentialTransferMint ya falla el segundo chequeo, y `confidential-transfer-mint-not-enabled` es su veredicto correcto. Un auditor null no es un error, es la política de sin-auditor, y una configuración válida devuelve `ok: true` con `reason: "ok"` y la config construida: `authority` desde `authorityKey`, `autoApproveNewAccounts` desde `autoApprove`, `auditorElGamalPubkey` desde `auditorKey` (el nombre de campo del calificador pone la G en mayúscula; el campo on-chain que lees en el lab no, una inconsistencia que el propio ecosistema trae y que te toca notar). El starter aprueba todo, que es precisamente el chequeo previo que deja pasar un mint que va a revertir.

![Diagrama de flujo de decisión que corre los tres chequeos de rechazo ordenados con sus slugs de error exactos antes de que una config válida caiga hasta el objeto devuelto.](assets/v08-flowchart.webp)

La rama en solitario es el supply confidencial. Toma las líneas de ct-keygen que guardaste, la pubkey ElGamal de supply y los bytes de `decryptableSupply(0)`, y construye `confidential-supply.ts`: una segunda variante de SPROUT cuyo mint inicializa ConfidentialTransferMint (sin auditor esta vez) y ConfidentialMintBurn en la misma transacción, dimensionado con `getMintSize` sobre las dos extensiones, inicializadas en ese orden, antes de `InitializeMint`. Cada builder que necesitas está en el mismo cliente: `getInitializeConfidentialMintBurnInstruction` toma la pubkey de supply y el cero cifrado, y la regla 3 la impone el programa, así que si inicializas MintBurn sin TransferMint vas a ver a la matriz de combo defenderse en producción.

Después demuestra algo. En un cluster donde las pruebas verifican, acuña de forma confidencial y muestra que el supply confidencial cambió: el supply descifrable del mint y el estado pendiente se mueven mientras el campo público `supply` se queda quieto, y `apply_pending_burn` y `rotate_supply_elgamal_pubkey` son las dos instrucciones de ops que tu runbook envolvería después. Si el gate de tu cluster está apagado, toma el camino de degradación y dilo: extiende la cuarta línea de `verify-confidential.ts` hasta una aserción completa, ConfidentialMintBurn presente, tu pubkey de supply repetida de vuelta, on-chain, con la mitad de la prueba de transferencia explícitamente fuera de alcance y nombrada como tal. La barra de aceptación es exacta: el script de verificación lee ConfidentialTransferMint más el auditor configurado y el flag de auto-approve de vuelta desde el mint on-chain, y o bien un depósito confidencial y una transferencia se completan en devnet o pasa la demostración de degradación. Lo que separa un ejercicio en solitario aprobado de uno con suerte es la frase que adjuntas en la redacción: qué camino tomaste, y por qué, en una línea honesta.

Si algún paso de aquí devolvió algo que el mío no, una prueba que falla en devnet donde afirmé que el gate estaba activo, un subcomando de la CLI que apareció o desapareció en un spl-token más nuevo, un corrimiento de rango de peers que rompió el pin de 0.15.0, márcalo en el canal de feedback del curso con el comando y el error exactos. El stack confidencial es la superficie que se mueve más rápido de este curso, mis sondeos están fechados 2026-08-22, y un aprendiz que detecta que devnet se desvía de la documentación está haciendo exactamente el trabajo de verificar-todo que este curso te sigue diciendo que le gana a cualquier tutorial, incluido este.

Ya entregaste la extensión más poderosa y menos enrutable del catálogo: un token al que ningún AMM le va a poner precio nunca, porque los montos cifrados no se pueden cotizar. Acuérdate de la allowlist de Raydium de m02-l1, y fíjate en lo que le acabas de hacer a las posibilidades de esta variante en ella. Esa colisión es la pregunta de apertura del próximo módulo: cuál conjunto de extensiones mantiene a SPROUT realmente negociable, decidido no por vibras sino por leer el código exacto que corren los DEX. El camino del emisor termina aquí, en su punto más especializado. El camino de la enrutabilidad empieza por averiguar cuánto cuesta.
