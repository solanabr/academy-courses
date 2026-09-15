# Extensiones de autoridad: quién puede tocar tus tokens

## Resumen

En la lección pasada construiste la economía de SPROUT desde instrucciones crudas y le hiciste harvest a sus comisiones retenidas hacia una tesorería, así que el valor ahora fluye exactamente hacia donde tú lo enrutas. Esta lección hace la pregunta más difícil: quién tiene permitido mover, congelar o recuperar por la fuerza ese valor en primer lugar. Vas a configurar cinco extensiones con forma de autoridad sobre mints desechables (PermanentDelegate, Pausable, DefaultAccountState, PermissionedBurn, MintCloseAuthority), y para cada una menos MintCloseAuthority vas a afirmar que la operación bloqueada de verdad se bloquea; la autoridad de cierre se decodifica en vez de ejercerse, porque cerrar necesita supply cero. Un asterisco: PermissionedBurn es más nueva que el build del programa que trae el surfnet, así que su prueba corre como una simulación en mainnet más un desvío opcional por devnet; el paso 5 explica la costura. Después, la demostración emblemática: un mint que lleva a la vez un PermanentDelegate y una cuenta con CpiGuard activado, donde compruebas qué bloquea la guarda y qué cuela el delegado. El repliegue: cada extensión recibe una config trabajada, la prueba de guarda contra delegado se trabaja línea por línea, y el challenge de cierre es totalmente en solitario: reconstruye esa prueba como dos transacciones, sin apoyo.

Ahora mira fallar una suposición de protección de valor. Activas CpiGuard en una cuenta de token, el riel a nivel de cuenta que impide que un programa mueva tus fondos a tus espaldas mediante una CPI. Estás, razonablemente, convencido de que está sellada. Entonces una sola instrucción la vacía de todos modos, invocada a través del PermanentDelegate del mint, que CpiGuard no tiene poder para detener. Algunas autoridades están *por encima* del tenedor de la cuenta.

Antes de cualquier teoría, ve a mirar una de estas autoridades en un token que casi con certeza has tenido. PYUSD, la stablecoin de PayPal y Paxos, lleva un delegado permanente en su mint ahora mismo. Tienes `solana` de la instalación anterior; si te la saltaste, instala las herramientas de Agave (solana-cli 3.1.10, verificado el 2026-08-22):

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Después lee el mint de PYUSD directo de mainnet:

```bash
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta
```

Lo que vuelve es un volcado hex, no un listado amigable, y la señal está en la línea del header: `Length: 866 (0x362) bytes` (verificado el 2026-08-22). Un mint SPL clásico es de 82 bytes. Todo lo que pasa de ahí es TLV, y dentro viven una entrada `permanentDelegate` real y un `mintCloseAuthority` real. Si los quieres nombrados en vez de contados, apunta tu propio inspector `decode-mint` a la misma dirección; nombrar esas entradas es el trabajo para el que lo construiste. Las autoridades que estás a punto de configurar en mints desechables son exactamente las que un emisor regulado usa en un token que guarda cientos de millones de dólares. Tenlo presente. Esta es la superficie de compliance del token más institucional que hay en la blockchain, no un set de funciones de juguete.

## Las autoridades que están por encima del tenedor de la cuenta

Empieza por el modelo mental, porque es lo que la mayoría de los tutoriales nunca dibuja. Una cuenta de token tiene un dueño, y tu instinto dice que el dueño es soberano sobre esa cuenta. Para el SPL simple, ese instinto es más o menos correcto. Token-2022 lo rompe a propósito. Varias extensiones instalan autoridades a nivel de *mint*, y una autoridad a nivel de mint actúa sobre cuentas que el tenedor nunca consintió en entregar. El dueño de la cuenta no firmó la configuración del mint. Optó por tener el token, y tener el token significó heredar los poderes que el emisor grabó en el mint. Esa asimetría es el tema entero de esta lección.

![Un diagrama de dos niveles con las autoridades a nivel de mint arriba, bajando hacia las cuentas de los tenedores, que muestra que las guardas a nivel de cuenta defienden solo la capa de la cuenta y no pueden anular un delegado permanente a nivel de mint.](assets/v01-diagram.png)

Un hecho operativo antes del catálogo, porque le da forma a cada decisión de emisión que vas a tomar con estas herramientas. Las cinco extensiones son todas extensiones de pre-inicialización: su instrucción de config tiene que correr después de que la cuenta de mint se crea y antes de `initializeMint`, sobre una cuenta ya dimensionada para la entrada TLV. No puedes atornillarle una autoridad a un mint que ya está vivo. No existe el "agrega un delegado permanente después, cuando cumplimiento pida uno." El set de autoridades se decide el día en que nace el mint, bajo incertidumbre, y vives con él toda la vida del token. Los emisores que se saltan este análisis no tienen una segunda pasada.

### PermanentDelegate, y la guarda a la que siempre vence

Un **delegado permanente** es una sola dirección, fijada una vez en el mint, que puede firmar `Transfer` y `Burn` por *cualquier* cuenta de ese mint. No una cuenta que el tenedor le haya delegado. Cualquier cuenta. Es la primitiva de recuperación forzosa: un emisor que necesita congelar e incautar por una orden judicial, revertir una acuñación equivocada o drenar una cuenta comprometida echa mano de exactamente esto. PYUSD lleva uno. Y es la autoridad más afilada de todo el catálogo de extensiones, porque vuelve trivial barrer cualquier cuenta de tenedor para quien tenga la llave del delegado.

No lo confundas con el delegado que ya conoces del SPL clásico. Un delegado normal lo concede el tenedor: el dueño firma un `approve` sobre su propia cuenta, lo topa en un monto, y puede hacerle `revoke` cuando quiera. Su alcance es una cuenta, su presupuesto es finito, y existe solo mientras el dueño lo tolere. El delegado permanente invierte todas y cada una de esas propiedades. El emisor lo fija al crear el mint, ningún tenedor firma nunca nada, no hay tope de monto, no hay revoke disponible para el tenedor, y su alcance es cada cuenta del mint que vaya a existir jamás. Misma palabra, especie distinta. Uno es un permiso que el tenedor concede. El otro es un poder que el tenedor hereda por elegir tener el token.

![Contraste a dos columnas del delegado por approve, concedido por el dueño, topado y revocable, contra el delegado permanente fijado por el emisor, que abarca todo el mint, no tiene tope, es irrevocable y siempre esquiva CpiGuard.](assets/v02-comparison.png)

Ahora la revelación, y este es el momento que vale la pena frenar. Pensarías que CpiGuard detiene esto. CpiGuard es la extensión a nivel de cuenta que un tenedor enciende para decir "ningún programa puede sacar fondos de mi cuenta mediante una invocación entre programas sin mi firma directa de nivel superior." Existe precisamente para matar los trucos de delegar-y-cerrar que hacen los programas maliciosos. Así que un tenedor cuidadoso activa CpiGuard y supone que un barrido del delegado permanente ahora está bloqueado como cualquier otro movimiento CPI furtivo.

No lo está. Y aquí vale la pena derivar la pregunta en vez de afirmarla, así que lee lo que la guarda promete de verdad. La especificación de la propia extensión no dice "no salen fondos durante una CPI." Enuncia una regla sobre *quién tiene que estar firmando*:

![Panel de regla que muestra que CpiGuard bloquea una transferencia CPI firmada por el dueño con CpiGuardTransferBlocked mientras que el delegado permanente siempre puede transferir o quemar, esquivando la guarda.](assets/v03-annotated-code.png)

Lee la regla y la respuesta sale sola. La guarda no pregunta "¿es este un movimiento CPI que no me gusta?". Pregunta "¿el firmante es un delegado?". Eso invierte la lectura ingenua: la única autoridad que la guarda rechaza durante una CPI es la del *dueño*, porque una firma de dueño es exactamente lo que un programa malicioso recolecta cuando te hace firmar una instrucción opaca. La delegación es visible y acotada, así que la guarda insiste en ella. La autoridad general del propio dueño es lo que la ingeniería social persigue, así que la guarda se la revoca dentro de una CPI.

Ahora pon un delegado permanente contra esa regla y el resultado queda sobredeterminado desde las dos direcciones. Estructuralmente, el delegado permanente no es el dueño, así que la cláusula que bloquea al dueño nunca se le aplica. Y la guía de extensiones de Token-2022 cierra la puerta de forma explícita en vez de dejarlo a la inferencia: si un mint lleva la extensión de delegado permanente, ese delegado siempre puede quemar o transferir tokens, esquivando CPI Guard. La guarda defiende la capa de la cuenta con honestidad y por completo. El delegado permanente opera una capa más arriba, donde la guarda no tiene jurisdicción. Ese es el límite de lo que una defensa a nivel de cuenta puede prometer, no un bug en CpiGuard.

Por esto importa la formulación y por esto "PermanentDelegate siempre esquiva CpiGuard" no es un eslogan sino un hecho documentado y estructural. No hay configuración de CpiGuard que lo cambie, porque la regla de la guarda es sobre delegación y el delegado permanente está exceptuado por nombre.

Raydium es tajante al respecto. Su política de soporte de Token-2022 rechaza PermanentDelegate, y la razón declarada es: quien tenga el delegado puede barrer cualquier cuenta de token, incluido el vault del pool. (Lee el programa del pool, no solo la página de docs, y el rechazo tiene una puerta: la verificación se salta para los mints SPL clásicos, para una `MINT_WHITELIST` corta y hardcodeada, y para los mints con una cuenta de asociación de mint inicializada. La política real es "ninguno salvo los que revisamos a mano".) Esa sola frase es toda la razón por la que un puñado de stablecoins de compliance entran por nombre en esa lista revisada a mano mientras que todos los demás que llevan la misma extensión son rechazados. Fíjate en qué sustantivo hace el trabajo: el TOKEN entra en la whitelist, una dirección a la vez. La EXTENSIÓN con forma de compliance se rechaza con la misma dureza que cualquier otro poder, que es una distinción a la que vuelven tanto m05-l2 como el capstone. Si tu token puede tener su liquidez barrida por una sola llave, un creador de mercado automatizado que custodia liquidez en una cuenta vault no puede listarlo con seguridad. Elegir PermanentDelegate es elegir qué plataformas van a tocar tu token alguna vez.

Y aun así PYUSD ya viene con uno, lo que te dice que el canje es deliberado, no descuidado. PayPal y Paxos eligieron a propósito un set de extensiones con forma de compliance, y para el 2025-05-29 el token tenía $215.9M repartidos en apenas 20.4k cuentas de token (Helius, "Solana's stablecoin landscape", 2025-05-29; supply circulante en Solana, no el total multi-chain; el campo `supply` de ese mint leía 688,176,370,728,435 unidades base a 6 decimales el 2026-08-22). Esa proporción es la señal delatora de un instrumento institucional: valor enorme, pocos tenedores, un emisor que necesita la capacidad legal de congelar e incautar cuando haga falta. Un delegado permanente es un pasivo para un DEX y un activo para un emisor regulado que le responde a un departamento de cumplimiento, y las dos lecturas son correctas a la vez. La extensión es una perilla que apunta tu token hacia un tipo de hogar y lo aleja de otro. Cuando la fijas, no estás agregando una función, estás eligiendo a qué lado del ecosistema quieres resultarle legible.

### Pausable: un solo interruptor detiene el mint entero

**Pausable** instala un alto global. Cuando la autoridad de pausa lo activa, cada transferencia, acuñación y quema de ese mint revierte de golpe, en toda la blockchain, hasta que alguien reanude. La trampa aquí es un error de categoría: los desarrolladores echan mano de Pausable esperando un congelamiento por cuenta, una forma de poner en cuarentena a un solo tenedor problemático. No es eso. Es un interruptor de apagado para el token entero. Lo activas y has congelado a todos los tenedores a la vez, incluida tu propia liquidez, tu propia tesorería, cada usuario honesto a mitad de una transacción. En el processor, un mint pausado hace que las rutas de quema y transferencia devuelvan `MintPaused` incondicionalmente. No existe un argumento "pausa la cuenta X", porque la pausa vive en el mint, no en la cuenta.

![Activar Pausable en el mint detiene cada transferencia, acuñación y quema de todos los tenedores a la vez hasta que la misma autoridad reanude, a diferencia de un congelamiento, que apunta a una sola cuenta.](assets/v04-diagram.png)

Úsala para lo que es: un freno de emergencia para el token entero, una herramienta de respuesta a incidentes, una forma de parar la hemorragia durante un exploit. Nunca como medida dirigida. Si necesitas detener una cuenta, eso es un congelamiento, que es el estado a nivel de cuenta que gobierna DefaultAccountState. Echa mano de la herramienta de cuenta para un problema de cuenta.

El interruptor es simétrico, lo cual es una carga operativa en sí misma. La misma autoridad de pausa reanuda el mint con una instrucción resume correspondiente, y hasta que lo haga, nada se mueve para nadie. Eso vuelve la custodia de la llave de pausa una cuestión de respuesta a incidentes, no una comodidad. Una autoridad de pausa filtrada es una llave de denegación de servicio contra tu token entero, y una autoridad de pausa que vive en la laptop de un ingeniero es un único punto de falla para la liquidez de cada tenedor. Si entregas Pausable, pon la llave detrás de un multisig, y ensaya la ruta de reanudación antes del día en que necesites la pausa. Un freno de emergencia que nadie puede soltar es peor que no tener freno.

### DefaultAccountState: onboarding congelado por defecto

**DefaultAccountState** puesta en `Frozen` es la primitiva más limpia para una forma de compliance concreta: cada cuenta nueva abre congelada y sigue congelada hasta que una autoridad de congelamiento la descongela. Este es el patrón "ningún tenedor transacciona hasta que pase el KYC", y es genuinamente elegante, porque invierte el defecto. Normalmente una cuenta es usable en el instante en que existe y tienes que atrapar a los malos actores después del hecho. Con DefaultAccountState(Frozen), la cuenta es inerte al nacer y un tenedor pasa a estar activo solo mediante un descongelamiento deliberado. El onboarding es opt-in del emisor, no opt-out.

![La cuenta de un tenedor abre congelada, las transferencias revierten con AccountFrozen hasta que la autoridad de congelamiento descongela esa cuenta específica después de las verificaciones, tras lo cual las transferencias normales funcionan.](assets/v05-flowchart.png)

El contraste con Pausable es lo que hay que fijar, porque un quiz seguro va a intentar cambiártelas. Pausable es un alto global que activas para todo el mint. DefaultAccountState(Frozen) es una barrera por cuenta que despejas un tenedor a la vez con un descongelamiento. Uno es un interruptor de apagado. El otro es un torniquete. Se sienten adyacentes y son herramientas completamente distintas.

Un grado de libertad más: el defecto no es para siempre. La autoridad de congelamiento puede actualizar el estado por defecto del mint después, así que un emisor puede lanzar con el acceso restringido y relajarse hacia un onboarding abierto una vez que el panorama de compliance se aclare, sin tocar una sola cuenta existente. Las cuentas conservan el estado que ya tengan; solo las cuentas creadas después de la actualización heredan el defecto nuevo. Lanza estricto, afloja a propósito. Esa es la ruta de migración que la mayoría de los equipos de compliance quiere de verdad, y es la única autoridad de esta lección cuya postura puede ablandarse durante la vida del token en vez de quedar congelada al nacer.

### PermissionedBurn: la extensión que los docs olvidaron

**PermissionedBurn** hace que cada quema exija la co-firma de la autoridad de quema. Una quema estándar, donde el tenedor le prende fuego a sus propios tokens, deja de funcionar en el momento en que esta extensión está presente. El processor rechaza una quema estándar contra un mint que lleva PermissionedBurn con `InvalidInstruction`, y te empuja por la ruta con permiso, donde un firmante extra, la autoridad de quema, tiene que firmar al lado. Los emisores la usan cuando la destrucción de supply tiene que autorizarse de forma central: piensa en flujos de redención donde solo el emisor puede retirar tokens.

La forma que esto atiende es la contabilidad de redención. Un tenedor sale a fiat mandando tokens a la cuenta de custodia del emisor, el fiat sale por un riel bancario, y entonces el emisor, y solo el emisor, retira el supply con una quema con permiso desde custodia. El supply on-chain se mantiene como un espejo honesto de los pasivos off-chain porque nadie más puede encogerlo: ningún tercero quema de forma unilateral, y ningún tenedor puede desinflar el circulante sin hacer ruido prendiéndole fuego a tokens que los libros del emisor todavía cuentan como en circulación. Para un token cuyo número de supply es una afirmación auditada, esa co-firma es la diferencia entre un libro mayor y una sugerencia.

Aquí está la trampa, y es una trampa de documentación, no una trampa de código. En m01-l4 ya conociste el hecho de que el catálogo de extensiones de solana.com omite PermissionedBurn por completo, mientras que el enum `ExtensionType` del código fuente la lista como una de las 29 variantes de producción. Si vas a buscar esta extensión en los docs oficiales y concluyes que no existe, le has creído a una página por encima del código. El enum es la verdad. Los docs son la instantánea que alguien tomó de la verdad, envejeciendo en silencio. Cada vez que construyes contra Token-2022, el enum del código fuente fijado zanja qué es real, y una entrada faltante en los docs no zanja nada. Esta es la segunda vez que este curso atrapa al catálogo oficial quedándose atrás del código, y no será la última.

### MintCloseAuthority: recuperar el rent, y la trampa de la resurrección

**MintCloseAuthority** le deja a una autoridad designada cerrar un mint una vez que su supply es cero, recuperando los lamports de rent que estaban inmovilizados para mantener viva la cuenta. PYUSD lleva una. Es mantenimiento aburrido la mayor parte del tiempo: levantaste un mint, cumplió su propósito, lo cierras y recuperas el rent.

La trampa es sutil y muerde en producción. Cuando cierras una cuenta, sus lamports se drenan y sus datos quedan en cero, pero la *dirección* no desaparece. Cualquiera puede mandarle lamports de vuelta a esa dirección y recrear una cuenta ahí. Una cuenta cerrada y luego resucitada puede confundirse con estado nuevo y confiable por código que supone "esta dirección existía antes, así que es legítima." La defensa es la higiene de marcar-como-cerrado: escribe un byte centinela en la cuenta antes de cerrarla para que una cuenta resucitada se reconozca como un cadáver, no como un recién nacido. Si tu sistema lee la mera existencia de una cuenta como prueba de procedencia, un ataque de resurrección convierte esa suposición en un agujero.

![Después de que un mint cierra, su dirección puede volver a fondearse como una cuenta nueva en la que el código ingenuo confía, así que un byte centinela escrito antes del cierre marca las resurrecciones como reutilizadas.](assets/v06-flowchart.png)

### El trade-off, nombrado con honestidad

Todas y cada una de estas autoridades te compran la misma moneda: control. Congelar, pausar, recuperar por la fuerza, restringir el onboarding, autorizar la destrucción. Y todas y cada una gastan la misma moneda a cambio: descentralización, que un exchange, un auditor o un creador de mercado va a leer como riesgo de contraparte. No hay autoridad gratis. Un ingeniero de integración de un DEX que escanea las entradas TLV de tu mint está leyendo un perfil de riesgo, y cada extensión de autoridad es una línea en él. PermanentDelegate es la línea más roja, que es exactamente por lo que Raydium la rechaza y por lo que las stablecoins de compliance que la llevan entran en la whitelist de plataformas específicas en vez de listarse en todas partes.

Si quieres el procedimiento de decisión en vez de la intuición, ya lo construiste: estas cinco encajan directo en la matriz de conflictos de m01-l4. El set de preguntas es corto. ¿Quién tiene que poder actuar contra un tenedor, bajo qué disparador legal, y en qué plataformas necesita vivir el token? Una stablecoin de nómina que le responde a un regulador acaba en PermanentDelegate más DefaultAccountState(Frozen) y se come las restricciones de listado, porque sus tenedores son contrapartes antes que usuarios. Un token de comunidad que necesita liquidez en Raydium no puede llevar un delegado permanente en absoluto, prefieran lo que prefieran los abogados. Escribe primero la lista de plataformas. Después elige solo las autoridades que esa lista permite, y documenta las que dejaste fuera a propósito, porque "podríamos haber tomado este poder y elegimos no hacerlo" es en sí una señal de confianza que los auditores leen.

![Una tabla comparativa de las cinco extensiones de autoridad que lista qué controla cada una, su trampa nombrada, y cómo un DEX o un auditor la lee como riesgo.](assets/v07-comparison.png)

Ese es el lente de diseño. No estás eligiendo funciones, estás eligiendo una postura de confianza y, con ella, el conjunto de lugares donde tu token puede vivir. Ahora constrúyelas.

## Lab: configura las autoridades, después rompe la guarda

Siete pasos, corriendo contra un surfnet local para que tengas el programa Token-2022 vivo sin gastar lamports reales. Los pasos 1 al 5 son configs trabajadas que corres tal cual. El paso 6, el emblemático, está trabajado por completo, y el challenge te hace reconstruirlo sin el apoyo. El paso 7 conecta todo con el criterio que las lecciones posteriores suponen que corre en verde. Presupuesta unos cuarenta y cinco minutos, la mayoría en el paso 6.

Surfpool te da un simnet que jala cuentas de mainnet de forma perezosa a medida que las tocas, con los programas SPL cargados y listos. Una advertencia que vale la pena conocer antes de que te muerda en el paso 5: los programas SPL que sirve un surfnet son los builds que surfpool trae consigo, no copias byte por byte de lo que está desplegado en mainnet, así que una instrucción muy nueva puede estar viva en mainnet y ausente de tu simnet. Instálalo si no lo has hecho (yo estoy en surfpool 1.2.1, verificado el 2026-08-22):

```bash
brew install txtx/taps/surfpool
```

Arranca un surfnet en una terminal y déjalo corriendo (las mismas banderas `--no-tui --no-studio` que la lección pasada, para que la TUI no se apodere de la terminal que estás dejando abierta):

```bash
surfpool start --no-tui --no-studio
```

Una nota de workspace antes de la instalación, porque el layout cambia aquí y se queda cambiado por el resto del curso. Las dependencias compartidas ahora viven en la RAÍZ del workspace, la carpeta que contiene `labs/`. Corre las instalaciones de abajo desde esa raíz (si la raíz todavía no tiene `package.json`: primero `npm init -y && npm pkg set type=module`). El código de las lecciones sigue viviendo en carpetas por lección como `labs/m02-l2/`, y cada comando de ejecución de aquí en adelante se da desde la raíz. El paquete autocontenido `labs/m02-l1` de la lección pasada se queda exactamente como está: los imports relativos como `../m01-l2/decode-mint` se resuelven por ubicación de archivo, no por dónde corres, así que ahí no se rompe nada.

Los pins son el mismo trío que en m02-l1, por las razones argumentadas a fondo allí (kit 7.1.1 porque ese es el major contra el que hacen peer los clientes de este workspace; `@solana-program/token-2022@0.15.0` y `@solana-program/system@0.13.0` son los minors actuales que hacen peer con kit ^7, y los minors siguientes saltan a ^8 y fallan duro con `ERESOLVE`). El cliente de token 0.15.0 ya viene con cada builder que esta lección necesita (`getInitializePermanentDelegateInstruction`, `getInitializePausableConfigInstruction`, `getInitializePermissionedBurnInstruction`, y el resto). Si npm se queja de un peer irresoluble en `@solana/kit`, esa es exactamente esta costura: fija los tres exactamente en vez de pelearte con ello.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.23.12 typescript@5.9.3
```

1. **Arma un helper que cree e inicialice un mint de Token-2022 con un conjunto elegido de extensiones.** Crea `mkdir -p labs/m02-l2` y arranca el archivo de la lección exactamente en `labs/m02-l2/verify-authorities.ts`; cada fragmento de los pasos 1 al 6 se agrega a este único archivo, y el paso 7 lo convierte en el criterio que una lección posterior vuelve a correr por esa ruta exacta. Construiste el andamiaje de creación de mints en el lab de economía de m02-l1 (asignar la cuenta al tamaño correcto, correr las instrucciones de extensión pre-init, después `initializeMint`). Reutilízalo. Lo único nuevo por autoridad es qué instrucción pre-init antepones. Aquí está la forma, con la plomería que tu lab de economía ya estableció plegada dentro de `createExtendedMint`, más los dos firmantes en los que se apoya cada paso posterior: `payer`, fondeado por un airdrop en el momento en que el archivo arranca, y `mintAuthority`, que nunca necesita lamports porque `payer` paga las comisiones de todo:

   ```typescript
   import {
     airdropFactory,
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     generateKeyPairSigner,
     lamports,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
     type KeyPairSigner,
   } from '@solana/kit';
   import { readFileSync } from 'node:fs';
   import {
     getInitializeMintInstruction,
     getMintSize,
     TOKEN_2022_PROGRAM_ADDRESS,
   } from '@solana-program/token-2022';
   import { getCreateAccountInstruction } from '@solana-program/system';

   // Cluster-agnostic on purpose: step 5 will want to re-run this whole file
   // against devnet, so the endpoints yield to env vars.
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions(process.env.RPC_WS_URL ?? 'ws://127.0.0.1:8900');
   const send = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   const airdrop = airdropFactory({ rpc, rpcSubscriptions });

   // The two signers the whole lab leans on. `payer` fee-pays and funds
   // everything; `mintAuthority` only ever signs, so it needs no lamports.
   //
   // The payer is env-overridable for one concrete reason. A throwaway signer
   // is perfect against a surfnet, which airdrops on demand, and useless
   // against devnet's faucet, which rate-limits: a funded address dies with
   // the process, so "top it up and retry" would fund a key the next run has
   // never heard of. Point PAYER_KEY at a file (`solana-keygen new -o
   // payer.json`), fund THAT address once, and step 5's devnet re-run works.
   const payer = process.env.PAYER_KEY
     ? await createKeyPairSignerFromBytes(
         new Uint8Array(JSON.parse(readFileSync(process.env.PAYER_KEY, 'utf8'))),
       )
     : await generateKeyPairSigner();
   const mintAuthority = await generateKeyPairSigner();
   console.log(`payer: ${payer.address}`);
   if (!process.env.PAYER_KEY) {
     await airdrop({
       recipientAddress: payer.address,
       lamports: lamports(5_000_000_000n),
       commitment: 'confirmed',
     });
   }

   async function submit(payer: KeyPairSigner, instructions: Instruction[]): Promise<void> {
     const { value: blockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
       (m) => appendTransactionMessageInstructions(instructions, m),
     );
     const signed = await signTransactionMessageWithSigners(message);
     // kit needs this narrowing: the signed tx's lifetime is a union until you assert it.
     assertIsTransactionWithBlockhashLifetime(signed);
     await send(signed, { commitment: 'confirmed' });
   }

   // preInit: extension instructions that must run BEFORE initializeMint.
   async function createExtendedMint(
     payer: KeyPairSigner,
     mintAuthority: KeyPairSigner,
     decimals: number,
     sizeExtensions: Parameters<typeof getMintSize>[0],
     preInit: (mint: KeyPairSigner) => Instruction[],
   ): Promise<KeyPairSigner> {
     const mint = await generateKeyPairSigner();
     const space = BigInt(getMintSize(sizeExtensions));
     const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

     const create = getCreateAccountInstruction({
       payer,
       newAccount: mint,
       lamports: rent,
       space,
       programAddress: TOKEN_2022_PROGRAM_ADDRESS,
     });
     const initMint = getInitializeMintInstruction({
       mint: mint.address,
       decimals,
       mintAuthority: mintAuthority.address,
       freezeAuthority: mintAuthority.address,
     });

     await submit(payer, [create, ...preInit(mint), initMint]);
     return mint;
   }
   ```

   Checkpoint: `createExtendedMint` devuelve un firmante cuya `.address` es un mint vivo en tu surfnet. Todavía no se afirma nada; los pasos siguientes le dan instrucciones de extensión reales.

2. **PermanentDelegate.** Antepón una instrucción que nombre al delegado:

   ```typescript
   import { getInitializePermanentDelegateInstruction } from '@solana-program/token-2022';
   import { extension } from '@solana-program/token-2022';

   const delegate = await generateKeyPairSigner();
   const pdMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('PermanentDelegate', { delegate: delegate.address })],
     (mint) => [
       getInitializePermanentDelegateInstruction({ mint: mint.address, delegate: delegate.address }),
     ],
   );
   ```

   Checkpoint: `decode-mint pdMint.address` (tu inspector de m01-l2) lista un TLV `PermanentDelegate` cuyo `delegate` es igual a `delegate.address`.

3. **Pausable.** El mismo patrón, más el alto en sí para que puedas ver revertir una transferencia:

   ```typescript
   import {
     getInitializePausableConfigInstruction,
     getPauseInstruction,
   } from '@solana-program/token-2022';

   const pauseAuthority = mintAuthority;
   const pausableMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('PausableConfig', { authority: pauseAuthority.address, paused: false })],
     (mint) => [
       getInitializePausableConfigInstruction({ mint: mint.address, authority: pauseAuthority.address }),
     ],
   );

   // Flip the global halt.
   await submit(payer, [getPauseInstruction({ mint: pausableMint.address, authority: pauseAuthority })]);
   ```

   Checkpoint: después de la pausa, cualquier `transferChecked` contra `pausableMint` revierte con `MintPaused`, error de programa personalizado 0x43 (decimal 67). Afirma la reversión, no un éxito, y fíjate en que se dispara para *cada* cuenta del mint, no una.

4. **DefaultAccountState(Frozen).** El argumento de estado es el enum `AccountState`:

   ```typescript
   import {
     getInitializeDefaultAccountStateInstruction,
     AccountState,
   } from '@solana-program/token-2022';

   const frozenDefaultMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('DefaultAccountState', { state: AccountState.Frozen })],
     (mint) => [
       getInitializeDefaultAccountStateInstruction({ mint: mint.address, state: AccountState.Frozen }),
     ],
   );
   ```

   Checkpoint: crea una cuenta de token nueva para este mint e intenta enviar desde ella. Revierte con `AccountFrozen`. Descongela esa única cuenta con `getThawAccountInstruction` firmada por la autoridad de congelamiento, reintenta, y la transferencia funciona. Acabas de hacerle onboarding a un tenedor sin tocar ningún otro.

5. **PermissionedBurn y MintCloseAuthority.** Configura las dos — en mints separados, y con la más nueva detrás de un sondeo, porque este es el paso donde muerde la advertencia del inicio del lab:

   ```typescript
   import {
     getInitializePermissionedBurnInstruction,
     getInitializeMintCloseAuthorityInstruction,
     getBurnCheckedInstruction,
     getPermissionedBurnCheckedInstruction,
   } from '@solana-program/token-2022';

   const burnAuthority = await generateKeyPairSigner();

   // MintCloseAuthority on its own mint, unconditionally: every cluster's
   // Token-2022 build knows this extension, so step 7's TLV assert always
   // has a live mint to decode.
   const closableMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('MintCloseAuthority', { closeAuthority: mintAuthority.address })],
     (mint) => [
       getInitializeMintCloseAuthorityInstruction({
         mint: mint.address,
         closeAuthority: mintAuthority.address,
       }),
     ],
   );

   // PermissionedBurn behind a probe: the newest extension in the catalog,
   // and this cluster's bundled build may predate it. On a build that does,
   // the extension initializer itself throws before the mint exists — catch
   // it, say so, and leave the mint null so step 7 can branch on it.
   let permissionedMint: KeyPairSigner | null = null;
   try {
     permissionedMint = await createExtendedMint(
       payer,
       mintAuthority,
       6,
       [extension('PermissionedBurn', { authority: burnAuthority.address })],
       (mint) => [
         getInitializePermissionedBurnInstruction({
           mint: mint.address,
           authority: burnAuthority.address,
         }),
       ],
     );
   } catch {
     console.log(
       'SKIPPED: PermissionedBurn (simnet build predates the extension; prove it on devnet with this step\'s re-run)',
     );
   }
   ```

   Checkpoint, en dos mitades. En cualquier cluster: `closableMint` está vivo y `decode-mint closableMint.address` lista su TLV `MintCloseAuthority`. En un cluster cuyo build de Token-2022 conoce PermissionedBurn: `permissionedMint` también está vivo, un `burnChecked` estándar contra él revierte con `Error: Invalid instruction`, error de programa personalizado 0xc (decimal 12), y la quema con permiso, `getPermissionedBurnCheckedInstruction` con `burnAuthority` co-firmando, funciona. La quema estándar está muerta en el momento en que PermissionedBurn está presente.

   Aquí está por qué el sondeo vive en el código trabajado en vez de quedar en tus manos. PermissionedBurn es la extensión más nueva del catálogo, y en surfpool 1.2.1 el build de Token-2022 que viene incluido todavía no la conoce: `getInitializePermissionedBurnInstruction` regresa `Error: Invalid instruction`, 0xc, desde el inicializador de la extensión mismo, antes de que el mint llegue a existir. Ese es tu simnet, no tu código — y como este lab es un archivo de awaits de nivel superior corridos en orden, un throw sin atrapar aquí mataría los pasos 6 y 7 en cada corrida de surfnet. El programa desplegado en mainnet sí la soporta, y puedes comprobarlo sin gastar un lamport, porque una simulación se ejecuta contra el programa real: arma la misma lista de instrucciones y mándala a `simulateTransaction` en mainnet con `sigVerify: false` y `replaceRecentBlockhash: true`, y los logs regresan `Instruction: PermissionedBurnExtension` / `PermissionedBurnInstruction::Initialize` / success. Para ver correr el checkpoint completo de verdad, la quema estándar muerta y la quema co-firmada viva, las dos, vuelve a correr el archivo entero contra devnet, el cluster donde puedes escribir con el programa real — los endpoints y el payer quedaron sobrescribibles por env en el paso 1 exactamente para este momento. Haz primero un payer que sobreviva a un reintento, porque el faucet de devnet limita la tasa y el firmante desechable del paso 1 dejaría varado cada lamport que le dieras:

   ```bash
   solana-keygen new --no-bip39-passphrase -o labs/m02-l2/payer.json
   solana airdrop 2 "$(solana-keygen pubkey labs/m02-l2/payer.json)" --url devnet
   # rate-limited? paste that same address into faucet.solana.com, then retry
   PAYER_KEY=labs/m02-l2/payer.json \
     RPC_URL=https://api.devnet.solana.com RPC_WS_URL=wss://api.devnet.solana.com \
     npx tsx labs/m02-l2/verify-authorities.ts
   ``` Todo lo demás en este lab corre en el surfnet tal como está escrito; Pausable, verificada en el mismo build, está bien.

6. **Lo emblemático: guarda contra delegado.** Esta es la prueba emblemática. Tienes un mint que lleva un PermanentDelegate y una cuenta de tenedor con CpiGuard activado. CpiGuard solo actúa *dentro de una CPI*, así que los dos movimientos se enrutan por el programa `spl-instruction-padding` (`iXpADd6AW1k5FaaXum5qHbSqyd7TtoN6AD7suVa83MF`), que envuelve una instrucción interna y la vuelve a invocar vía CPI.

   Primero, levanta las cuentas sobre las que actúa la prueba, porque ninguno de los pasos anteriores las creó: un dueño, su cuenta de token con algo de `pdMint`, y un destino. `payer` fondea y paga las comisiones de todo, así que ni `owner` ni `delegate` necesitan jamás un lamport propio, y una falla del fee payer nunca puede hacerse pasar por un veredicto de la guarda:

   ```typescript
   import {
     findAssociatedTokenPda,
     getCreateAssociatedTokenIdempotentInstruction,
     getMintToCheckedInstruction,
   } from '@solana-program/token-2022';

   const owner = await generateKeyPairSigner();
   const destinationOwner = await generateKeyPairSigner();

   const [ownerAccount] = await findAssociatedTokenPda({
     mint: pdMint.address,
     owner: owner.address,
     tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
   });
   const [destination] = await findAssociatedTokenPda({
     mint: pdMint.address,
     owner: destinationOwner.address,
     tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
   });

   await submit(payer, [
     getCreateAssociatedTokenIdempotentInstruction({
       payer,
       ata: ownerAccount,
       mint: pdMint.address,
       owner: owner.address,
     }),
     getCreateAssociatedTokenIdempotentInstruction({
       payer,
       ata: destination,
       mint: pdMint.address,
       owner: destinationOwner.address,
     }),
     getMintToCheckedInstruction({
       mint: pdMint.address,
       token: ownerAccount,
       mintAuthority,
       amount: 100n,
       decimals: 6,
     }),
   ]);
   ```

   Ahora la prueba en sí:

   ```typescript
   import {
     ExtensionType,
     getEnableCpiGuardInstruction,
     getReallocateInstruction,
     getTransferCheckedInstruction,
   } from '@solana-program/token-2022';
   // `Instruction` is already imported at the top of this file (step 1); this
   // is one file, so re-importing the type is a TS2300 duplicate-identifier
   // error even though tsx strips it and runs fine.
   import { type Address, AccountRole } from '@solana/kit';

   const PADDING_PROGRAM =
     'iXpADd6AW1k5FaaXum5qHbSqyd7TtoN6AD7suVa83MF' as Address;

   // Wrap an inner token instruction so the padding program re-invokes it via CPI.
   // Wire format (PadInstruction::Wrap): [1][num_accounts u32 LE][data_len u32 LE][inner data].
   // Accounts: the inner accounts, then the inner program id as a readonly account.
   function wrapForCpi(inner: Instruction): Instruction {
     const innerAccounts = inner.accounts ?? [];
     const innerData = inner.data ?? new Uint8Array();
     const header = new Uint8Array(9);
     header[0] = 1; // Wrap
     new DataView(header.buffer).setUint32(1, innerAccounts.length, true);
     new DataView(header.buffer).setUint32(5, innerData.length, true);
     const data = new Uint8Array(header.length + innerData.length);
     data.set(header, 0);
     data.set(innerData, header.length);
     return {
       programAddress: PADDING_PROGRAM,
       accounts: [
         ...innerAccounts,
         { address: inner.programAddress, role: AccountRole.READONLY },
       ],
       data,
     };
   }

   // ownerAccount holds tokens, owned by `owner`, CpiGuard enabled.
   // pdMint carries the permanent delegate `delegate`.
   // `payer` fee-pays every transaction; kit collects the other signers
   // (owner, delegate) straight off the instructions they are embedded in.
   async function proveGuardVsDelegate(
     payer: KeyPairSigner,
     owner: KeyPairSigner,
     delegate: KeyPairSigner,
     ownerAccount: Address,
     destination: Address,
     pdMint: Address,
   ): Promise<{ ownerBlocked: boolean; delegatePassed: boolean }> {
     // The ATA was created at its minimal size, and an account extension needs
     // its bytes to exist before it can be enabled. So: grow the account with
     // Reallocate naming the extension, THEN flip the guard on. Skip the grow
     // and the enable fails on account size.
     await submit(payer, [
       getReallocateInstruction({
         token: ownerAccount,
         payer,
         owner,
         newExtensionTypes: [ExtensionType.CpiGuard],
       }),
       getEnableCpiGuardInstruction({ token: ownerAccount, owner }),
     ]);

     // Proof leg 1: the owner-signed transfer, wrapped for CPI. Expect it to
     // REVERT with CpiGuardTransferBlocked; ownerBlocked is true only if it did.
     const ownerMove = getTransferCheckedInstruction({
       source: ownerAccount,
       mint: pdMint,
       destination,
       authority: owner, // the owner is NOT a delegate -> the guard's must-be-a-delegate rule blocks this inside CPI
       amount: 1n,
       decimals: 6,
     });
     let ownerBlocked = false;
     try {
       await submit(payer, [wrapForCpi(ownerMove)]);
     } catch {
       ownerBlocked = true;
     }

     // Proof leg 2: the SAME transfer authorized by the permanent delegate,
     // wrapped identically. Expect it to SUCCEED; delegatePassed is true only if it confirmed.
     const delegateMove = getTransferCheckedInstruction({
       source: ownerAccount,
       mint: pdMint,
       destination,
       authority: delegate, // the permanent delegate is carved out by name -> always passes the guard
       amount: 1n,
       decimals: 6,
     });
     let delegatePassed = false;
     try {
       await submit(payer, [wrapForCpi(delegateMove)]);
       delegatePassed = true;
     } catch {
       delegatePassed = false;
     }

     return { ownerBlocked, delegatePassed };
   }
   ```

   Checkpoint, y este es el criterio: `ownerBlocked === true` y `delegatePassed === true`. La transferencia CPI del propio dueño la detiene la guarda que él mismo activó, y la transferencia idéntica del delegado permanente pasa de largo por esa misma guarda. En mi corrida la bloqueada regresó con error de programa personalizado 0x2a (decimal 42), `CpiGuardTransferBlocked`, y el log de programa `CPI Guard is enabled, and a program attempted to transfer user funds via CPI without using a delegate`, la misma cadena que cita el panel de regla de arriba; el movimiento del delegado se confirmó. Vale la pena correr también el control: manda la transferencia del dueño SIN el envoltorio de padding y funciona, porque la guarda es inerte fuera de una CPI.

![La misma cuenta protegida por CpiGuard bloquea la transferencia CPI envuelta de la propia Alice pero permite la transferencia envuelta idéntica del delegado permanente, porque la guarda exige un firmante delegado y el delegado permanente está exceptuado por nombre.](assets/v08-diagram.png)

7. **Conéctalo al criterio.** Las cinco demostraciones ya viven en un archivo, `labs/m02-l2/verify-authorities.ts`; ahora termínalo hasta volverlo un criterio: crea cada mint desechable, decodifícalo con tu inspector de m01-l2 para afirmar que su TLV de extensión de verdad está presente, después corre las pruebas de comportamiento: la transferencia de Pausable revirtiendo, el congelar-y-después-descongelar de DefaultAccountState, y el par guarda contra delegado. La prueba de PermissionedBurn es condicional, por la advertencia de simnet del paso 5 — y el sondeo ya existe: el código trabajado del paso 5 deja `permissionedMint` en null en un build que precede a la extensión e imprime por ti la línea `SKIPPED: PermissionedBurn (simnet build predates the extension; prove it on devnet with this step's re-run)`. Bifurca según eso: cuando `permissionedMint` no es null, corre contra él las afirmaciones de quema-estándar-muerta y quema-co-firmada-viva; cuando es null, el skip impreso ya dijo la verdad. Un skip que nombra su razón mantiene honesto el criterio en cada cluster contra el que corre este curso. Este archivo es el artefacto que la lección le agrega al toolkit de SPROUT, `sprout-mint-authorities`, y consume las dos cosas que ya entregaste: la plomería para crear mints del lab de economía y el inspector `decode-mint`. La cola de afirmaciones para lo emblemático se ve así:

   ```typescript
   const { ownerBlocked, delegatePassed } = await proveGuardVsDelegate(
     payer, owner, delegate, ownerAccount, destination, pdMint.address,
   );
   if (!ownerBlocked) throw new Error('CpiGuard failed to block the owner-signed CPI move');
   if (!delegatePassed) throw new Error('permanent delegate did not bypass CpiGuard');
   console.log('authority gate: all assertions hold');
   ```

   Córrelo:

   ```bash
   npx tsx labs/m02-l2/verify-authorities.ts
   ```

   Checkpoint, y este es el criterio de la lección reformulado como script: cada extensión de autoridad está presente en su mint, el movimiento de PermanentDelegate funciona a través de CpiGuard mientras que el movimiento CPI directo del dueño queda bloqueado, y el alto de Pausable hace revertir una transferencia. Cuando esa salida está en verde, el artefacto está en el estante y el lab está terminado. Deja el archivo exactamente en esa ruta con exactamente esas afirmaciones; una lección posterior lo llama por nombre.

## Challenge

Sin apoyo nuevo. Dado un mint que lleva un PermanentDelegate y una cuenta con CpiGuard activado, escribe las dos transacciones, desde cero, que comprueben qué operación bloquea la guarda y cuál esquiva el delegado. Esta es la prueba de delegado contra guarda, en solitario.

Tu barra de aceptación es exactamente el criterio del lab, pero construyes todo tú mismo: la guarda tiene que bloquear el movimiento CPI firmado por el dueño y sin delegado, el movimiento del delegado permanente tiene que funcionar contra esa misma cuenta protegida, y tus afirmaciones tienen que sostenerse en ambos sentidos. Dos cosas separan un intento en solitario que aprueba de uno con suerte. Primero, los dos movimientos tienen que ir envueltos para que se ejecuten *dentro de una CPI*, porque CpiGuard es inerte en una instrucción de nivel superior; si mandas una transferencia pelada del dueño y funciona, no has puesto a prueba la guarda, has esquivado la condición. Segundo, la única diferencia entre tus dos transacciones es la autoridad que firma. Misma cuenta origen, mismo destino, mismo monto, mismos decimales, mismo envoltorio. Si difiere cualquier otra cosa, no has aislado la variable, y tu prueba no prueba nada. Cuando el movimiento CPI firmado por el dueño revierte con `CpiGuardTransferBlocked` y el movimiento firmado por el delegado se confirma, y puedes señalar la razón de una línea en la condición del processor, esto es tuyo.

Después ponte a prueba a ti mismo contra la trampa que esta lección está construida para desarmar: si te sorprendes pensando "pero yo podría apretar CpiGuard para que también bloquee al delegado", vuelve a leer la regla. No existe ese ajuste. La única palanca de CpiGuard es `lockCpi`, encendida o apagada, y lo que impone cuando está encendida es que la autoridad que firma durante una CPI tiene que ser un delegado. La guía de extensiones exceptúa entonces al delegado permanente por nombre. Apretar no está en el menú.

¿Algo aquí devolvió un resultado que el mío no, o tu programa forkeado rechazó un init? Márcalo en el canal de feedback del curso con el nombre de la extensión y el error exacto, idealmente con la instrucción que falla pegada. Un lector que atrape otro inicializador de extensión que el build de simnet todavía no conoce, como se comporta PermissionedBurn en el paso 5, está haciendo reconocimiento real del que se beneficia el resto de la cohorte, y es exactamente el hábito de "verifica contra el programa vivo, no le creas al tutorial" que este curso no deja de entrenar.

Ya manejaste el lado MINT de la autoridad: los poderes que un emisor graba en el token mismo, por encima de cada tenedor. Eso deja la otra mitad de la historia. La lección siguiente baja al lado de la cuenta y hace la pregunta invertida: ¿qué puede *rechazar* la propia cuenta de un tenedor, incluso cuando el mint dice que sí? CpiGuard fue un adelanto de esa capa. Vas a conocer el resto de las extensiones que protegen al tenedor, las guardas que un tenedor activa en su propia cuenta, más una excepción deliberada que pertenece con ellas de todos modos, NonTransferable, una extensión del lado del mint cuyo beneficiario es la integridad de la credencial y no el control del emisor, y vas a ver dónde termina el veto del tenedor y dónde empieza la autoridad del mint. Cuando los emisores componen estas primitivas en rieles de compliance reales, restricción de acceso basada en congelamiento apilada sobre delegados permanentes, ese es el territorio del curso planificado DeFi and RWA Engineering; aquí construiste las primitivas que compone.
