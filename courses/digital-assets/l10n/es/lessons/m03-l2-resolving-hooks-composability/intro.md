# Resolver hooks, y por qué rompen la componibilidad

## Resumen

La lección pasada escribiste el harvest-hook y lo probaste en un harness de LiteSVM: una transferencia de la allowlist pasó y una no permitida falló. Pero el harness le entregó al hook sus cuentas a mano, y esa es la ficción que esta lección rompe. Hoy te sientas en la silla del consumidor: armas una transferencia Token-2022 común de SPROUT con hook tal como la armaría una billetera y la ves revertir por una cuenta faltante, y después escribes el resolvedor del lado del cliente que lee el ExtraAccountMetaList desde el programa del hook, reconstruye la transferencia con las cuentas correctas y la hace entrar. Con esa evidencia sobre la mesa, derivamos el hecho de diseño del que cuelga todo el argumento de componibilidad: dentro de Execute, cada cuenta de la transferencia original llega de solo lectura y no firmante, así que un hook nunca puede gastar lo que inspecciona. Chequeo del repliegue: el resolvedor se trabaja contigo línea por línea, los scripts de transferencia son tuyos para cablear y correr, y el challenge es completamente en solitario. Nota de alcance: la interfaz del transfer hook se enseña de punta a punta aquí; los cursos hermanos que tocan transferencias con hook apuntan de vuelta en lugar de volver a derivarla.

Tu hook pasó sus pruebas. Quiero arruinarte esa sensación en los primeros cinco minutos, porque el harness te estaba mintiendo con educación: LiteSVM te dejaba poner cada cuenta en la transacción tú mismo, como un equipo de tramoya acomodando la utilería antes de que entre el actor. Las billeteras reales no saben que tu hook existe. Los DEX reales no saben que tu hook existe. En el momento en que cualquiera de los dos arma un TransferChecked estándar de cuatro cuentas para tu variante de SPROUT, el runtime se pone a buscar cuentas que no están en la transacción, y todo se muere en la rampa de lanzamiento.

Antes de cualquier teoría, corre esto. Calcula ocho bytes que conociste la lección pasada desde el lado del programa:

```bash
node -e 'const {createHash} = require("node:crypto");
console.log(createHash("sha256").update("spl-transfer-hook-interface:execute").digest().subarray(0, 8).toString("hex"))'
```

Deberías ver `692565c54bfb661a`. La lección pasada esos bytes eran el timbre que Token-2022 toca en tu programa. Esta lección son una deuda: todos y cada uno de los integradores que alguna vez toquen tu token tienen que encontrar las cuentas que ese discriminador exige, o su transferencia revierte. Mantén ese hex a la vista; lo vas a buscar con grep dentro de una cuenta real en unos minutos.

## El impuesto de resolución

### ¿Quién se supone que arma las cuentas?

Arranca desde la restricción que ya tienes del primer módulo de este curso: una transacción de Solana tiene que declarar, de entrada, cada cuenta que va a tocar. El runtime no descubre cuentas en pleno vuelo; agenda alrededor de la lista declarada. Ahora suma lo que construiste la lección pasada: cuando un mint lleva la extensión TransferHook, Token-2022 hace CPI al programa del hook en cada transferencia, y el Execute de tu harvest-hook necesita sus propias cuentas para hacer su trabajo, el estado de la allowlist que revisa y el log de tesorería que escribe.

Junta esos dos hechos y salta una pregunta con fuerza de verdad. La billetera que arma la transferencia nunca ha oído hablar de tu programa. Token-2022 conoce la dirección del hook por el mint, pero no qué cuentas podría querer un programa arbitrario. El hook conoce sus propias necesidades, pero los programas no pueden estirar la mano y traer cuentas en tiempo de ejecución. Alguien tiene que poner las cuentas del hook en la transacción antes de que se envíe, y nada en el stack se ofrece de voluntario.

Recorre las respuestas ingenuas, porque cada una falla de una forma instructiva. "El hook debería cargar lo que necesita" falla primero: en Solana no hay carga, una cuenta que no está en la transacción no existe en lo que respecta al programa. "Token-2022 debería tener hardcodeadas las cuentas extra" falla segundo: el punto entero del slot de hook es que el programa detrás de él es arbitrario, así que ninguna lista fija puede servirle a todos los hooks. "Que la billetera del remitente simplemente lo sepa" no es una respuesta, es el problema reformulado.

La respuesta real es la única pieza de la interfaz que todavía no has tocado: el hook publica un manifiesto legible por máquinas de las cuentas que necesita, on-chain, en una dirección conocida, y se espera que cada cliente lo lea. Ese manifiesto es el ExtraAccountMetaList que inicializaste la lección pasada, que vive en tu programa de hook en la PDA sembrada con el literal `extra-account-metas` y el mint. La convención para leerlo y actuar sobre él se llama TLV Account Resolution, y la implementación de referencia es el crate de Rust `spl-tlv-account-resolution`; este curso fija 0.11.1, la misma versión que fija el hook de la lección pasada, y el layout de bytes de abajo se leyó del código fuente de ese crate y no cambia de 0.11.0 a 0.11.3, la más nueva en crates.io al 2026-09-01 (los parches 0.11.2/0.11.3 rehacen el funcionamiento interno del resolvedor, no el formato serializado). El flujo que cada billetera, DEX y script tiene que ejecutar:

![Diagrama de flujo de una transferencia con hook: el cliente busca, decodifica y resuelve el ExtraAccountMetaList y agrega los extras; saltarse la resolución revierte con un error MissingAccount.](assets/v01-flowchart.webp)

Dos palabras de esa imagen merecen precisión, porque el código del lab depende de ellas.

Primero, la cuenta en sí. Los datos de la cuenta de validación son TLV, la misma disciplina tipo-longitud-valor que vienes decodificando desde las lecciones de mint, pero no le apuntes tu decodificador de mint: los anchos del header son distintos. El tipo de una entrada de mint es un entero u16 chico (tu inspector imprimió 18 y 19) con una longitud u16; el tipo de esta cuenta es un hash completo de 8 bytes (exactamente el `692565c54bfb661a` que calculaste arriba, el discriminador de execute, porque esta lista responde la pregunta "qué necesita execute") con una longitud u32, y después el valor. El valor es un array chico con prefijo de longitud: un conteo u32 y después esa cantidad de entradas de exactamente 35 bytes cada una.

Segundo, la entrada. Cada ExtraAccountMeta es un struct fijo de 35 bytes, y esos 35 bytes son una pequeña maravilla de compresión:

![Layout anotado de los 35 bytes de ExtraAccountMeta, con un discriminador de un byte que elige una de cuatro codificaciones, 32 bytes de dirección o de config empaquetada, y un byte para cada una de las flags de signer y writable.](assets/v02-annotated-code.webp)

Lee otra vez la fila del discriminador, porque esconde la trampa más afilada de esta lección. El discriminador 0 es fácil: los 32 bytes son la dirección, listo. El discriminador 1 dice: deriva una PDA en el programa del hook, con seeds desempaquetados de los bytes de config. El discriminador 2 dice: lee la pubkey de los datos de la instrucción o de la cuenta; el harvest-hook nunca lo emite, y el resolvedor de abajo lo rechaza ruidosamente en lugar de adivinar. Y 128 más i dice: deriva una PDA en el programa que ocupe el índice i en la lista de cuentas de la instrucción Execute. Ese último quiere decir que la resolución es posicional. Las cuentas de la instrucción Execute son, en orden: source en 0, mint en 1, destination en 2, owner en 3, la cuenta de validación en 4, y después cada extra en el orden de la lista. Un seed config que dice "clave de la cuenta 1" quiere decir el mint porque el mint está en el índice 1 de esa lista. Resuelve las metas fuera de orden, o sáltate una, y cada seed basado en índice después de ella deriva una dirección distinta. Nada te avisa. La PDA derivada simplemente está mal, la verificación on-chain falla, la transferencia revierte. El orden no es una convención aquí, es una entrada al hash.

![La lista ordenada de cuentas de la instrucción execute, índices 0 a 6, donde omitir o reordenar un extra corre todos los índices posteriores y deriva la PDA equivocada.](assets/v03-diagram.webp)

Hay una consecuencia más de almacenar el manifiesto en una cuenta mutable: puede cambiar. La autoridad del hook puede llamar a update-extra-account-metas y rehacer la lista, y un cliente que cacheó su resolución la semana pasada ahora está reenviando las cuentas de la semana pasada. Un conjunto viejo revierte exactamente igual que uno faltante. Resuelve en fresco, por transferencia, o acepta que tu integración se rompe el día que el emisor toca la lista. En la práctica eso pone la resolución en el mismo camino de código que buscar tu blockhash: las dos son lecturas de frescura contra el estado vivo de la blockchain, las dos quedan viejas de la misma manera, y una transferencia armada con cualquiera de esos valores cacheados es una transferencia armada para una blockchain que ya no existe. El crate ya viene con helpers de resolución del lado de Rust, y el cliente JS de referencia hace el mismo trabajo; en el lab vas a escribir el resolvedor tú mismo en TypeScript, porque después de haber decodificado esos 35 bytes una vez a mano, cada librería de helpers que llames alguna vez deja de ser magia.

### El hook que solo puede decir que no

Ahora la otra mitad de la pregunta, la que tu compañero de equipo hace en el momento en que propones entregar esto: acabamos de cablear código arbitrario en cada transferencia de nuestro token. ¿Qué impide que un hook malicioso, o nuestro propio hook después de una mala actualización, drene al remitente a mitad de la transferencia? Tiene la cuenta source. Tiene al owner. El owner firmó.

La respuesta no es una promesa, son cuatro líneas de construcción en el crate de la interfaz, y quiero que veas las líneas de verdad en lugar de confiar en mi paráfrasis. Así es como `spl-transfer-hook-interface` arma la instrucción Execute que Token-2022 le manda a tu hook:

```rust
// spl-transfer-hook-interface, instruction.rs: the execute() builder.
let accounts = vec![
    AccountMeta::new_readonly(*source_pubkey, false),
    AccountMeta::new_readonly(*mint_pubkey, false),
    AccountMeta::new_readonly(*destination_pubkey, false),
    AccountMeta::new_readonly(*authority_pubkey, false),
    // …the builder then appends the validation account and the resolved extras.
];
```

Cada cuenta que vino de la transferencia original, el source, el mint, el destination, el owner que firmó, se reconstruye como `AccountMeta::new_readonly(pubkey, false)`. Lee los dos argumentos como dos poderes revocados. Solo lectura: el hook no puede debitar el source, acreditarse a sí mismo ni mutar ningún estado que la transferencia toque, porque el runtime impone la capacidad de escritura por instrucción, y esta instrucción no otorga ninguna. El `false` es la flag de signer: aunque el owner firmó la transacción externa, esa firma no se extiende al frame del hook, así que el hook no puede darse vuelta y hacer CPI a Token-2022 fingiendo actuar por el owner. En el modelo de privilegios de CPI, los permisos solo se estrechan a medida que las llamadas descienden. La interfaz eligió el ajuste más estrecho para cada cuenta heredada, y esa elección se llama desescalada.

El estrechamiento corre en una sola dirección, y la alternativa muestra por qué tiene que ser así. Si el programa llamado pudiera escalar, entonces llamar a cualquier programa querría decir confiar en todos los programas que ese pueda llamar, transitivamente, para siempre. Así que el runtime de Solana deja que quien llama otorgue solo privilegios que ya tiene, y siempre le permite otorgar menos. La firma que le diste a tu billetera autoriza un frame; cada frame de abajo hereda a lo sumo lo que el frame de arriba eligió pasar hacia abajo, y Token-2022 eligió no pasar nada. El argumento entero de seguridad de los hooks descansa en esa elección, y acabas de leerlo en cuatro líneas del crate de la interfaz.

![En la transferencia externa el source y el destination son escribibles y el owner firma, pero dentro de Execute las cuatro cuentas llegan de solo lectura y no firmantes.](assets/v04-diagram.webp)

Ese es el inventario de poderes de la lección pasada, observar, registrar en su propio terreno y vetar, ahora leído de las líneas mismas de la interfaz en lugar de afirmado; la única adición que vale la pena hacer es que tu log de tesorería funciona porque la PDA del log es la cuenta del hook, declarada escribible en las metas, no heredada de la transferencia. Cuando tu compañero de equipo haga la pregunta del drenaje, la respuesta en una línea es que la interfaz le entrega al hook cada cuenta de la transferencia ya despojada a solo lectura y no firmante, así que no hay nada que pueda gastar ni ninguna firma que pueda reusar.

La honestidad pide el otro lado del inventario, eso sí, porque "no puede robar" no es "no puede hacer daño". Un veto es poder. Un hook que revierte sin condiciones congela a todos los tenedores del token, permanentemente si el hook es inmutable, arbitrariamente si su autoridad se vuelve hostil. Un hook puede quemar cómputo: Execute corre dentro del presupuesto de la transferencia, sin más tope propio que el límite de la transacción. Y un hook malicioso puede perfectamente mover fondos desde cuentas que su propio programa controla; la garantía cubre solo las cuentas de la transferencia. La desescalada hace al hook seguro para el saldo del remitente. No lo hace seguro para la liveness del token, y no hace nada sobre el costo. Sostén las dos mitades, porque el ecosistema sin duda lo hace.

### El reparto, con comprobantes

Así que ponle precio a la extensión con franqueza, desde las dos sillas. Desde la silla del emisor, un hook es control por transferencia: allowlists, logging, controles de compliance, la política que quepa en un programa. Desde cualquier otra silla de la mesa, ese mismo hook es una factura de impuestos. Cada integrador tiene que buscar, decodificar y reenviar las cuentas extra correctas en cada transferencia, para siempre, y una cuenta vieja o faltante revierte la transacción de un usuario en el peor momento posible. La sección de la desescalada te dijo que el hook no puede robarle al integrador. Nada en la interfaz le impide agotarlo.

Así le ponen precio los tres actores más instructivos en producción, y ninguno de ellos es hipotético:

![Cuatro posturas de producción frente a los transfer hooks: Raydium los rechaza, pump.fun estaciona un programa vacío en el slot, PYUSD deja el program id en null y Meteora DBC reenvía.](assets/v05-comparison.webp)

Cada fila premia una mirada más de cerca. Raydium es el directo: su referencia de Token-2022 rechaza TransferHook porque "invoca un programa personalizado en cada transferencia, con consumo arbitrario de CU", y rechaza PermanentDelegate en la misma frase porque "un tenedor del delegate puede barrer cualquier cuenta de token, incluido el vault del pool". Mira las cinco extensiones que SÍ entraron a su allowlist: una config de comisión, dos formas de metadatos, visualización de interés, visualización escalada. Todas y cada una de ellas son pasivas. El patrón es la tesis de compatibilidad del curso en miniatura: las extensiones que rehacen la visualización o acumulan comisiones entran a la whitelist, las extensiones que corren código o tienen poder sobre cuentas se rechazan, por nombre, en producción.

pump.fun es el gracioso, y después deja de ser gracioso. Su transfer hook de producción es literalmente `#[program] pub mod transfer_hook_authority {}`: seis líneas de anchor-lang 0.31.1, desplegadas en `333UA891CYPpAJAthphPT3hg1EkUBLhNFoP9HoWW3nug`, sin nada adentro. Ten cuidado con el POR QUÉ, porque la lectura obvia está mal y m05-l1 pone la corrección en la página: nadie más habría podido reclamar ese slot de todos modos. El slot de hook de un mint se fija en SU creación, por quien lo crea, así que no hay carrera que ganar ni nada que ocupar. Lo que hizo pump es más estrecho y más interesante: apuntaron el slot de hook de sus propios mints, el que solo se fija al nacer, a un programa que demostrablemente no hace nada, en vez de dejar el slot vacío para que una versión futura de ellos mismos lo llene con lógica viva. Es un mecanismo de compromiso apuntado a su propio futuro, y esa recámara vacía protege volumen real de transferencias hoy: el slot es lo bastante valioso para llenarlo y lo bastante peligroso para llenarlo con nada. Y fíjate en lo que costaría incluso un hook vacío si estuviera armado: en el momento en que un program id real entra en ese slot, cada integrador del planeta debe el baile de resolución que acabamos de enseñar, aunque Execute no haga nada en absoluto. Un hook no-op no es gratis. El impuesto se cobra sobre el slot, no sobre la lógica.

![Un espectro de cuatro estados para el slot de transfer hook, de ausente a null, a no-op y a armado, con el impuesto de resolución arrancando en el momento en que entra un program id.](assets/v06-diagram.webp)

PYUSD, el lanzamiento emblemático de Token-2022 en mayo de 2024, hace la misma jugada desde el lado de compliance sobre un mint de stablecoin regulada que carga cientos de millones de dólares en supply (774M PYUSD en una lectura en vivo, 2026-09-01): el mint lleva la extensión TransferHook con `transferHook.programId` puesto en null. El slot está configurado, el arma está cargada, la recámara está vacía. Los emisores recurren a este patrón porque preserva la opción de control por transferencia sin cobrarles el impuesto a los integradores de hoy; en el challenge vuelves a leer ese null con tu propio decodificador en lugar del script prestado que lo imprimió por primera vez en m01-l1. Y el DBC de Meteora es el contrapunto que demuestra que reenviar es posible a escala de protocolo: reenvía explícitamente las cuentas restantes del transfer hook en cada instrucción de transferencia. El impuesto se puede pagar. La mayoría de las plataformas simplemente decidió que tu hook no vale la factura.

Da un paso atrás una vez, porque la forma de este impuesto no es una rareza de Solana, es la factura de una decisión de diseño que ya conoces. Solana exige que cada cuenta se declare antes de ejecutar; eso es lo que hace posible el agendado en paralelo, y es la razón por la que la transferencia de p-token que mediste en el módulo uno corre en CU de dos dígitos. Una blockchain con despacho dinámico no cobra ningún impuesto de resolución: el ERC-777 de Ethereum dejaba que los contratos de token llamaran hooks del receptor sin nada predeclarado, y los integradores pagaron cero por adelantado. Pagaron después. Esos hooks podían reentrar al contrato que llamaba a mitad de la transferencia, y el pool de imBTC en Uniswap V1 fue drenado exactamente por esa puerta en abril de 2020. El transfer hook es la misma idea con los modos de falla movidos de lugar: el modelo de cuentas declaradas exporta la contabilidad al cliente, lo cual es molesto, y a cambio el hook llega en un frame donde no puede tocar nada, que es la garantía que ERC-777 nunca tuvo. Pagas el impuesto en código de integración en lugar de en exploits. Qué lado de ese canje se ve mejor depende de en qué silla te sientes, y la tabla de arriba es el mercado votando desde todas las sillas a la vez.

![El despacho dinámico de ERC-777 no les costó nada por adelantado a los integradores pero permitió el drenaje por reentrancia de imBTC, mientras que Token-2022 cobra un impuesto de resolución por un frame de hook de solo lectura y no firmante.](assets/v07-comparison.webp)

Ese es el trade-off, dicho tan sin rodeos como puedo: un hook le compra al emisor control por transferencia y exporta un impuesto de resolución a cada integrador río abajo, y la preferencia revelada del mercado es rechazar el impuesto. Si SPROUT tiene que negociarse en las plataformas principales, el hook es la extensión de la que más te vas a arrepentir. Si SPROUT es un instrumento con permisos donde tú controlas las plataformas, el hook es exactamente la herramienta correcta. El curso Payments & Commerce está del segundo lado de esa línea: su riel de suscripciones consume este mismo comportamiento de reenvío de hooks contra un flujo controlado por el comercio, y apunta de vuelta a esta lección en lugar de volver a derivarla.

## Lab: enséñale a una billetera a cargar las cuentas de tu hook

El plan: levantar un surfnet local, desplegar el harvest-hook de la lección pasada, acuñar una variante fresca de SPROUT con hook, y después mandar el mismo TransferChecked dos veces, una desnuda y otra resuelta. Todo el TypeScript de abajo pasó el chequeo estricto de tipos contra el toolchain fijado el 2026-08-22, y el camino de decodificación del resolvedor se probó además contra bytes de lista sintéticos hechos a mano; un puñado de los números de blockchain en vivo en esta lección se releyeron el 2026-09-01 durante la edición final, y donde una fecha importa se imprime al lado de su número. Las partes que necesitan tu hook desplegado son las partes que solo tú puedes correr.

1. **Arranca un surfnet local.** Surfpool te da un validador local que se comporta como mainnet y no necesita claves; es la misma herramienta que instalaste allá en m02-l1 (surfpool 1.2.1 en mi máquina, verificado el 2026-08-22; `curl -sL https://run.surfpool.run/ | bash` si esta máquina todavía no lo tiene):

   ```bash
   surfpool --version
   surfpool start --no-tui --no-studio
   ```

   Déjalo corriendo. El RPC queda en `http://127.0.0.1:8899`, los websockets en `ws://127.0.0.1:8900`. En una segunda terminal, apunta el CLI de `solana` (vino con la instalación del toolchain de Agave en la lección pasada) hacia él y asegúrate de que tu keypair por defecto tenga lamports: `solana config set --url http://127.0.0.1:8899`, y después `solana airdrop 100` si tu saldo está en cero.

2. **Despliega el harvest-hook.** Desde el repo del programa de la lección pasada (el build de anchor-lang 1.1.2; la capa del framework en sí es del curso Master Anchor V2, aquí solo entregamos el único programa). Primero un paso de reconciliación, porque esta es la trampa clásica del auto-deploy: el código de la lección pasada tiene hardcodeado `declare_id!("HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ")`, y en tu máquina no existe ningún keypair para esa dirección. `solana program deploy` despliega en la dirección del keypair autogenerado en `target/deploy/harvest_hook-keypair.json`, así que el id del runtime y el id declarado no coincidirían, y el entrypoint generado por Anchor rechaza cada llamada con `DeclaredProgramIdMismatch`. (Tu harness de LiteSVM nunca se topó con esto porque cargaba el `.so` en `harvest_hook::ID` directamente; un deploy real no tiene esa cortesía.) Sincroniza los dos antes de desplegar:

   ```bash
   solana address -k target/deploy/harvest_hook-keypair.json
   # paste that address into declare_id!(...) in src/lib.rs, then:
   cargo build-sbf
   solana program deploy target/deploy/harvest_hook.so
   ```

   Copia el program id impreso, que ahora coincide con tu `declare_id!`; es `$HOOK` para el resto del lab.

3. **Acuña la variante de SPROUT con hook.** La lección pasada probaste que TransferHook es solo de creación, así que acuñamos uno fresco en vez de adaptar el viejo. El CLI de `spl-token` de m01-l4 se encarga de toda la ceremonia (spl-token-cli 5.6.1 en crates.io al 2026-08-22):

   ```bash
   spl-token create-token --program-2022 --decimals 6 --transfer-hook $HOOK
   # copy the mint address -> $MINT
   spl-token create-account $MINT
   spl-token mint $MINT 1000
   solana-keygen new --no-bip39-passphrase -o stranger.json
   # copy the stranger's pubkey -> $DEST
   spl-token create-account $MINT --owner $DEST
   ```

   Checkpoint: `spl-token display $MINT` muestra la extensión TransferHook con tu program id adentro. Fíjate en lo que acaba de pasar sin avisar: las dos cuentas de token se crearon con la extensión TransferHookAccount, porque un mint con hook se la impone a cada tenedor.

4. **Rearma el estado on-chain del hook.** Dentro de LiteSVM inicializaste el ExtraAccountMetaList en el harness; este surfnet nunca lo ha visto. La llamada initialize es estándar de la interfaz, así que te la puedo entregar byte por byte. Arma primero un workspace de cliente (kit fijado en 7.1.1 exacto, el mismo pin de rango de pares que usa el workspace del curso; nota que la línea kit-^7 de `@solana-program/token-2022` es 0.15.0, que a propósito no necesitamos, todo lo de abajo es kit crudo):

   ```bash
   mkdir sprout-client && cd sprout-client
   npm init -y && npm pkg set type=module
   npm install @solana/kit@7.1.1
   npm install -D tsx@4.20.5
   export MINT=... HOOK=... DEST=...
   ```

   Guarda `init-metas.ts`:

   ```typescript
   // init-metas.ts: one call to the harvest-hook's own
   // initialize-extra-account-metas instruction, so the validation account
   // exists on the surfnet the way it existed inside last lesson's harness.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstruction,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';
   import { getExtraAccountMetaAddress } from './resolve.js';

   const SYSTEM_PROGRAM = address('11111111111111111111111111111111');
   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   const validationAccount = await getExtraAccountMetaAddress(MINT, HOOK);

   // Same hashed-string discriminator scheme as execute:
   // sha256("spl-transfer-hook-interface:initialize-extra-account-metas")[0..8].
   const initDiscriminator = new Uint8Array(
     createHash('sha256')
       .update('spl-transfer-hook-interface:initialize-extra-account-metas')
       .digest()
       .subarray(0, 8),
   );

   const ix: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: validationAccount, role: AccountRole.WRITABLE },
       { address: MINT, role: AccountRole.READONLY },
       { address: payer.address, role: AccountRole.WRITABLE_SIGNER },
       { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },
     ],
     data: initDiscriminator,
   };

   const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
   const tx = await signTransactionMessageWithSigners(
     pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstruction(ix, m),
     ),
   );
   assertIsTransactionWithBlockhashLifetime(tx);
   await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
     commitment: 'confirmed',
   });
   console.log('ExtraAccountMetaList initialized at', validationAccount);
   ```

   Uno de los imports, `getExtraAccountMetaAddress`, viene de `resolve.js`, que escribes en el próximo paso, así que todavía no corras nada. Otras dos llamadas de administración NO son estándar de la interfaz, y las dos tienen que correr antes de cualquier transferencia: el `initialize` propio de tu hook, y después su allowlist. Este surfnet no ha visto tus PDAs de hook-config ni de treasury-log, igual que no había visto la meta list; `allow_destination` no puede correr sin la config (su chequeo `has_one` la deserializa), y el Execute del acto 2 deserializa las dos PDAs con sus bumps guardados, así que saltarte `initialize` mata la transferencia resuelta con la misma seguridad que una meta list faltante. Las dos instrucciones pertenecen a la API propia de tu programa de la lección pasada, y como hasta ahora nada te ha mostrado cómo codificar a mano una instrucción de Anchor con un argumento, esta también viene trabajada. Dos hechos cargan todo el script: un discriminador de Anchor se deriva como `sha256("global:<method_name>")[0..8]`, el mismo truco de cadena hasheada que la interfaz bajo otro namespace, y un argumento `Pubkey` codificado en borsh son solo sus 32 bytes crudos agregados después del discriminador. Fíjate bien QUÉ clave pones en la allowlist: la comprobación que escribiste compara `ctx.accounts.destination.key()`, la CUENTA DE TOKEN de destino en el índice 2 de Execute, así que la entrada a agregar es la ATA del desconocido, no la pubkey de su billetera (`spl-token address --token $MINT --owner $DEST --verbose` la imprime; expórtala como `$DEST_ATA`). Guarda `init-hook.ts`:

   ```typescript
   // init-hook.ts: the two non-interface management calls, hand-encoded.
   // initialize (no args), then allow_destination(destination: Pubkey).
   import {
     AccountRole,
     address,
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getAddressEncoder,
     getProgramDerivedAddress,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';

   const SYSTEM_PROGRAM = address('11111111111111111111111111111111');
   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);
   const DEST_ATA = address(process.env.DEST_ATA!); // the stranger's TOKEN ACCOUNT

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const enc = getAddressEncoder();

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   // Anchor namespace, not the interface namespace: sha256("global:<name>")[0..8].
   const anchorDisc = (name: string): Uint8Array =>
     new Uint8Array(createHash('sha256').update(`global:${name}`).digest().subarray(0, 8));

   const pda = async (seed: string) => {
     const [addr] = await getProgramDerivedAddress({
       programAddress: HOOK,
       seeds: [seed, enc.encode(MINT)],
     });
     return addr;
   };
   const config = await pda('hook-config');
   const treasury = await pda('treasury');

   // initialize: no args; accounts in the Initialize struct's exact order.
   const initIx: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: payer.address, role: AccountRole.WRITABLE_SIGNER },
       { address: MINT, role: AccountRole.READONLY },
       { address: config, role: AccountRole.WRITABLE },
       { address: treasury, role: AccountRole.WRITABLE },
       { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },
     ],
     data: anchorDisc('initialize'),
   };

   // allow_destination(destination: Pubkey): 8-byte discriminator + 32 raw
   // borsh bytes of the pubkey. The Manage struct's order: authority, config.
   const allowData = new Uint8Array(40);
   allowData.set(anchorDisc('allow_destination'), 0);
   allowData.set(new Uint8Array(enc.encode(DEST_ATA)), 8);
   const allowIx: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: payer.address, role: AccountRole.READONLY_SIGNER },
       { address: config, role: AccountRole.WRITABLE },
     ],
     data: allowData,
   };

   const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
   const tx = await signTransactionMessageWithSigners(
     pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions([initIx, allowIx], m),
     ),
   );
   assertIsTransactionWithBlockhashLifetime(tx);
   await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
     commitment: 'confirmed',
   });
   console.log('hook config + treasury initialized; allowlisted', DEST_ATA);
   ```

   Orden de ejecución una vez que `resolve.ts` exista en el próximo paso: `npx tsx init-metas.ts`, y después `npx tsx init-hook.ts`. Si renombraste métodos, seeds o reordenaste los structs de cuentas en tu propio harvest-hook, ajusta este script a tu programa, no al revés. Haz todo eso antes del acto uno, porque el punto del próximo acto es que el reenvío, y solo el reenvío, es lo que se interpone entre el desconocido y sus tokens.

5. **Escribe el resolvedor.** Este es el corazón del lab, y es una transcripción directa a TypeScript de lo que `spl-tlv-account-resolution` 0.11.1 hace en Rust. Guarda `resolve.ts`:

   ```typescript
   // resolve.ts: client-side TLV Account Resolution for a Token-2022 transfer hook.
   // Mirrors what spl-tlv-account-resolution 0.11.1 does in Rust, byte for byte.
   import {
     AccountRole,
     getAddressDecoder,
     getAddressEncoder,
     getProgramDerivedAddress,
     type Address,
     type AccountMeta,
     type Rpc,
     type SolanaRpcApi,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';

   const enc = getAddressEncoder();
   const dec = getAddressDecoder();

   // The TLV entry we are looking for is typed by the execute instruction's
   // hashed-string discriminator: sha256("spl-transfer-hook-interface:execute")[0..8].
   export const EXECUTE_DISCRIMINATOR = new Uint8Array(
     createHash('sha256')
       .update('spl-transfer-hook-interface:execute')
       .digest()
       .subarray(0, 8),
   ); // 69 25 65 c5 4b fb 66 1a

   // One validation account per mint, on the HOOK program:
   // seeds = ["extra-account-metas", mint].
   export async function getExtraAccountMetaAddress(
     mint: Address,
     hookProgram: Address,
   ): Promise<Address> {
     const [pda] = await getProgramDerivedAddress({
       programAddress: hookProgram,
       seeds: ['extra-account-metas', enc.encode(mint)],
     });
     return pda;
   }

   // The fixed 35-byte entry: discriminator u8 | address_config [u8;32] |
   // is_signer u8 | is_writable u8.
   export interface ExtraAccountMeta {
     discriminator: number;
     addressConfig: Uint8Array;
     isSigner: boolean;
     isWritable: boolean;
   }

   // Account layout: [8-byte TLV type][u32 LE length][u32 LE count][count * 35 bytes].
   export function unpackExtraAccountMetaList(data: Uint8Array): ExtraAccountMeta[] {
     const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
     let offset = 0;
     while (offset + 12 <= data.length) {
       const tlvType = data.subarray(offset, offset + 8);
       const length = view.getUint32(offset + 8, true);
       if (equalBytes(tlvType, EXECUTE_DISCRIMINATOR)) {
         const count = view.getUint32(offset + 12, true);
         const entries = data.subarray(offset + 16, offset + 16 + count * 35);
         const metas: ExtraAccountMeta[] = [];
         for (let i = 0; i < count; i++) {
           const e = entries.subarray(i * 35, (i + 1) * 35);
           metas.push({
             discriminator: e[0],
             addressConfig: e.subarray(1, 33),
             isSigner: e[33] === 1,
             isWritable: e[34] === 1,
           });
         }
         return metas;
       }
       offset += 12 + length;
     }
     throw new Error('no TLV entry under the execute discriminator: is the list initialized?');
   }

   // Resolution is positional against the EXECUTE instruction, which the runtime
   // will build as: [0] source, [1] mint, [2] destination, [3] owner,
   // [4] validation account, then each extra in list order. Seeds that reference
   // "account at index N" mean N in THAT list, which is why order is not optional.
   export async function resolveTransferHookAccounts(
     rpc: Rpc<SolanaRpcApi>,
     mint: Address,
     hookProgram: Address,
     source: Address,
     destination: Address,
     owner: Address,
     amount: bigint,
   ): Promise<AccountMeta[]> {
     const validationAccount = await getExtraAccountMetaAddress(mint, hookProgram);
     const { value: account } = await rpc
       .getAccountInfo(validationAccount, { encoding: 'base64' })
       .send();
     if (!account) {
       throw new Error(`validation account ${validationAccount} does not exist on this cluster`);
     }
     const raw = Uint8Array.from(Buffer.from(account.data[0], 'base64'));
     const metas = unpackExtraAccountMetaList(raw);

     const executeKeys: Address[] = [source, mint, destination, owner, validationAccount];
     const executeData = buildExecuteData(amount);

     const resolved: AccountMeta[] = [];
     for (const meta of metas) {
       const resolvedAddress = await resolveMeta(meta, executeKeys, executeData, hookProgram);
       executeKeys.push(resolvedAddress);
       resolved.push({
         address: resolvedAddress,
         role: meta.isSigner
           ? meta.isWritable
             ? AccountRole.WRITABLE_SIGNER
             : AccountRole.READONLY_SIGNER
           : meta.isWritable
             ? AccountRole.WRITABLE
             : AccountRole.READONLY,
       });
     }

     // Canonical append order for the transferring instruction:
     // the resolved extras in list order, then the hook program, then the
     // validation account. This mirrors the reference client helper.
     return [
       ...resolved,
       { address: hookProgram, role: AccountRole.READONLY },
       { address: validationAccount, role: AccountRole.READONLY },
     ];
   }

   async function resolveMeta(
     meta: ExtraAccountMeta,
     executeKeys: Address[],
     executeData: Uint8Array,
     hookProgram: Address,
   ): Promise<Address> {
     // discriminator 0: address_config IS the pubkey.
     if (meta.discriminator === 0) {
       return dec.decode(meta.addressConfig);
     }
     // discriminator 1: PDA on the hook program.
     // discriminator 128 + i: PDA of the program whose address sits at
     // execute-instruction account index i.
     if (meta.discriminator === 1 || meta.discriminator >= 128) {
       const programAddress =
         meta.discriminator === 1 ? hookProgram : executeKeys[meta.discriminator - 128];
       if (!programAddress) {
         throw new Error(`seed program index ${meta.discriminator - 128} is out of range`);
       }
       const seeds = unpackSeeds(meta.addressConfig, executeKeys, executeData);
       const [pda] = await getProgramDerivedAddress({ programAddress, seeds });
       return pda;
     }
     // discriminator 2 (pubkey stored in account/instruction data) exists in the
     // crate but the harvest-hook never uses it; fail loudly instead of guessing.
     throw new Error(`ExtraAccountMeta discriminator ${meta.discriminator} not supported here`);
   }

   // address_config for a PDA holds packed seed configs, zero-padded to 32 bytes:
   // 1 = literal (len, bytes), 2 = instruction-data slice (index, length),
   // 3 = account key at index, 4 = account-data slice.
   function unpackSeeds(
     config: Uint8Array,
     executeKeys: Address[],
     executeData: Uint8Array,
   ): Uint8Array[] {
     const seeds: Uint8Array[] = [];
     let i = 0;
     while (i < 32) {
       const tag = config[i];
       if (tag === 0) break; // zero padding: no more seeds
       if (tag === 1) {
         const len = config[i + 1];
         seeds.push(config.subarray(i + 2, i + 2 + len));
         i += 2 + len;
       } else if (tag === 2) {
         const index = config[i + 1];
         const length = config[i + 2];
         seeds.push(executeData.subarray(index, index + length));
         i += 3;
       } else if (tag === 3) {
         const index = config[i + 1];
         const key = executeKeys[index];
         if (!key) {
           throw new Error(`seed wants account index ${index}, which is not resolved yet`);
         }
         seeds.push(new Uint8Array(enc.encode(key)));
         i += 2;
       } else {
         // tag 4 reads another account's data; the harvest-hook does not use it.
         throw new Error(`seed config tag ${tag} not implemented in this lab`);
       }
     }
     return seeds;
   }

   // The execute instruction's data, needed for instruction-data seeds:
   // [8-byte execute discriminator][u64 LE amount].
   function buildExecuteData(amount: bigint): Uint8Array {
     const data = new Uint8Array(16);
     data.set(EXECUTE_DISCRIMINATOR, 0);
     new DataView(data.buffer).setBigUint64(8, amount, true);
     return data;
   }

   function equalBytes(a: Uint8Array, b: Uint8Array): boolean {
     return a.length === b.length && a.every((byte, i) => byte === b[i]);
   }
   ```

   Tres detalles merecen una segunda lectura. El array `executeKeys` empieza como las cinco cuentas base y crece a medida que cada extra se resuelve, que es exactamente cómo los seeds basados en índice pueden referenciar legalmente a un extra anterior: el orden, otra vez, es una entrada. El mapeo de roles conserva las flags de signer y de writable que las metas declararon para las cuentas PROPIAS del hook, así es como tu log de tesorería sigue siendo escribible. Y `resolveMeta` rechaza el discriminador 2 ruidosamente en lugar de adivinar; el crate soporta una variante con la pubkey en los datos que el harvest-hook nunca usa, y un resolvedor que maneja mal una codificación en silencio es peor que uno que se detiene.

6. **Corre la historia de dos transferencias.** Ahora el script de la recompensa, y nota que la reversión on-chain te va a nombrar la falla. Si todavía no corriste `init-metas.ts` en el paso 4, córrelo ahora (`npx tsx init-metas.ts`; tsx corre TypeScript directamente, y el paso 4 puso el pin de 4.20.5 que introdujo m01-l1 en las devDependencies de este workspace en vez de confiar en lo que sea que `npx` encuentre). Correrlo una SEGUNDA vez no es inofensivo y la falla es opaca: el `init` de Anchor que lleva adentro crea la cuenta de validación, así que una segunda corrida vuelve como un `Custom program error: #0` pelado, que es `AccountAlreadyInUse` y quiere decir que la cuenta que querías ya existe. Después guarda `transfer.ts`:

   ```typescript
   // transfer.ts: the same TransferChecked, twice: once the way a naive wallet
   // builds it (simulated, watch it die), once with the resolved extras appended.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstruction,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getAddressEncoder,
     getBase64EncodedWireTransaction,
     getProgramDerivedAddress,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Address,
     type AccountMeta,
     type Instruction,
   } from '@solana/kit';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';
   import { resolveTransferHookAccounts } from './resolve.js';

   const TOKEN_2022 = address('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
   const ATA_PROGRAM = address('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);
   const DEST_OWNER = address(process.env.DEST!);
   const AMOUNT = 5_000_000n; // 5 SPROUT at 6 decimals
   const DECIMALS = 6;

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const enc = getAddressEncoder();

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   async function ata(owner: Address): Promise<Address> {
     const [addr] = await getProgramDerivedAddress({
       programAddress: ATA_PROGRAM,
       seeds: [enc.encode(owner), enc.encode(TOKEN_2022), enc.encode(MINT)],
     });
     return addr;
   }

   const source = await ata(payer.address);
   const destination = await ata(DEST_OWNER);

   // TransferChecked is Token-2022 instruction 12: [12][amount u64 LE][decimals u8].
   const data = new Uint8Array(10);
   data[0] = 12;
   new DataView(data.buffer).setBigUint64(1, AMOUNT, true);
   data[9] = DECIMALS;

   const baseAccounts: AccountMeta[] = [
     { address: source, role: AccountRole.WRITABLE },
     { address: MINT, role: AccountRole.READONLY },
     { address: destination, role: AccountRole.WRITABLE },
     { address: payer.address, role: AccountRole.READONLY_SIGNER },
   ];

   async function buildAndSign(accounts: AccountMeta[]) {
     const ix: Instruction = { programAddress: TOKEN_2022, accounts, data };
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstruction(ix, m),
     );
     return await signTransactionMessageWithSigners(message);
   }

   // Act 1: the naive transfer. Four accounts, exactly what a hookless wallet sends.
   const naive = await buildAndSign(baseAccounts);
   const sim = await rpc
     .simulateTransaction(getBase64EncodedWireTransaction(naive), { encoding: 'base64' })
     .send();
   // kit decodes numeric RPC fields as bigints unless the field is on its
   // allowed-numeric list, and the instruction index inside InstructionError
   // is not on it. JSON.stringify throws TypeError on any bigint it meets, so
   // without this replacer act 1 dies right here with a stack trace pointing
   // at your own file, and act 2 never runs. Number() is safe for an
   // instruction index and keeps it unquoted in the output.
   //
   // Worth knowing WHY this is a kit problem specifically: m01-l1 stringified
   // the same InstructionError shape with no replacer and no crash, because it
   // spoke to the RPC with a bare fetch and JSON.parse never produces bigints.
   // The hazard arrives with the typed client, not with the JSON.
   const bigintSafe = (_key: string, value: unknown): unknown =>
     typeof value === 'bigint' ? Number(value) : value;
   console.log('naive transfer err:', JSON.stringify(sim.value.err, bigintSafe));
   for (const line of sim.value.logs ?? []) console.log('  ', line);

   // Act 2: resolve the hook's extras off-chain and forward them.
   const extras = await resolveTransferHookAccounts(
     rpc, MINT, HOOK, source, destination, payer.address, AMOUNT,
   );
   console.log('\nforwarding', extras.length, 'extra accounts:');
   for (const meta of extras) console.log('  ', meta.address, 'role', meta.role);

   const resolved = await buildAndSign([...baseAccounts, ...extras]);
   assertIsTransactionWithBlockhashLifetime(resolved);
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   await sendAndConfirm(resolved, { commitment: 'confirmed' });
   console.log('\nresolved transfer landed:', getSignatureFromTransaction(resolved));
   ```

   Córrelo: `npx tsx transfer.ts`. Checkpoint, acto uno: el error de simulación debería leerse `{"InstructionError":[0,"MissingAccount"]}`, con una línea de log que nombra a tu hook como `Unknown program $HOOK` y una línea final que dice `An account required by the instruction is missing`. Lee cuál cuenta falta, porque no es la que casi todos adivinan. Token-2022 busca la cuenta de validación entre las cuentas que mandaste, no la encuentra y por eso nunca resuelve nada; después hace CPI a un programa de hook que tampoco está en la transacción, y el runtime no puede invocar un programa que nunca le entregaron. Esa es la falla con forma de billetera que abre esta lección, reproducida a pedido. Checkpoint, acto dos: una lista impresa de cuentas reenviadas (los extras de tu hook, después el programa del hook, después la cuenta de validación) seguida de `resolved transfer landed:` y una firma. Confírmalo con `spl-token balance $MINT --owner $DEST`: el desconocido tiene 5. Misma instrucción, mismo signer, mismo monto; la única diferencia entre morir y entrar fue la lista de cuentas.

![El TransferChecked ingenuo de cuatro cuentas revierte, mientras que la versión resuelta agrega los extras del hook, el programa del hook y la cuenta de validación con datos de instrucción idénticos.](assets/v08-comparison.webp)

7. **Mira a un cliente que resuelve hacerlo por ti.** Un último sondeo, ahora que sabes lo que cuesta la maquinaria:

   ```bash
   spl-token transfer $MINT 1 $DEST --allow-unfunded-recipient
   ```

   Esa flag no es sobre hooks y aquí no es opcional: el paso 3 le generó el keypair al desconocido sin fondearlo, y el CLI da un error duro del lado del cliente ante un destinatario sin SOL antes de armar nada. Quita la flag y obtienes un rechazo que no tiene nada que enseñarte sobre transfer hooks. Con ella, la transferencia entra sin ninguna flag específica de hooks, porque el CLI es un integrador bien portado que corre esta misma resolución en línea antes de mandar (su flag `--transfer-hook-account` existe para firmar sin conexión, donde no hay ningún RPC contra el cual resolver). Cada herramienta que "simplemente funciona" con tu token con hook está pagando en silencio el impuesto que acabas de desglosar.

## Challenge

Solo, tres partes, sin apoyo.

Primero, rompe la transferencia resuelta de tres maneras y diagnostica cada una por la capa que la mató. Manda otra vez la transferencia ingenua de cuatro cuentas del acto uno. Después manda una con todo reenviado EXCEPTO la última cuenta agregada, la PDA de validación. Después manda una totalmente reenviada a un destino que NO pusiste en la allowlist. Las tres mueren, y no hay dos que mueran en el mismo lugar: una ni siquiera llega a tu programa, otra llega y es rechazada antes de que corra tu lógica, y otra es tu propio veto disparándose. Escribe una oración por falla que nombre la capa y cite la línea de log que lo prueba. Dos advertencias, porque el caso del medio es la trampa: su mensaje de error contiene la frase "not enough account keys", que suena a que el runtime se queja pero no lo es, y la señal más segura del primer caso es una línea que está ausente en vez de presente. Si quieres una pista de por qué esta distinción importa en la operación, fíjate cuál de las tres le echarían tus usuarios a su billetera y cuál a tu token.

Segundo, el criterio de evaluación, en una línea: desde la desescalada de solo lectura, di por qué tu hook, o cualquier hook, no puede gastar los tokens del remitente. Si tu oración no menciona los dos poderes revocados, la capacidad de escritura y la firma, todavía no es la respuesta completa.

Tercero, llévalo a mainnet. Apunta tu inspector `decode-mint` de m01-l2 al mint de PYUSD, `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`, recorre el TLV hasta la entrada TransferHook y lee el program id: treinta y dos bytes en cero, el null que deja dormida a toda la extensión. El script prestado de m01-l1 te imprimió este null el día uno; hoy lo verificaste byte por byte con un decodificador que escribiste tú, sobre una stablecoin regulada de nueve cifras, y sabes exactamente qué cambiaría para cada integrador de PYUSD del planeta el día que ese campo deje de ser cero.

Si algún checkpoint de aquí imprimió algo que el mío no, o si el layout de cuentas de tu hook te obligó a adaptar un script, repórtalo en el canal de feedback del curso con el comando y la salida pegados. Los bugs de resolución son exactamente la clase de falla que solo aparece en máquinas reales, y la corrida rota de un lector le enseña más a este curso que una limpia.

El próximo módulo sube la apuesta sobre lo que una transferencia siquiera muestra. Acabas de probar que un hook no puede actuar sobre tokens que no controla; la próxima extensión va más lejos y esconde el monto mismo. Los saldos confidenciales cifran el número que se mueve dentro de un sobre sellado que solo el remitente, el receptor y un auditor opcional pueden abrir, y quedas advertido antes de que bosquejes un diseño que quiera las dos cosas: el emparejamiento es traicionero. Una transferencia confidencial igual invoca tu hook, pero le pasa un monto centinela (`u64::MAX`) en lugar del número real, así que cualquier hook cuya lógica esté condicionada a los montos se queda ciego justo cuando el monto está escondido. Ahí vamos después: cómo Token-2022 mueve valor que se niega a mostrar.
