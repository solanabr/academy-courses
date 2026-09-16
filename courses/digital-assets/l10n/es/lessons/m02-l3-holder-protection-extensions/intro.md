# Extensiones de protección del tenedor: lo que las cuentas pueden rechazar

## Resumen

En la lección pasada configuraste las autoridades del lado del mint, PermanentDelegate, Pausable, DefaultAccountState, PermissionedBurn, MintCloseAuthority, y viste al delegado permanente pasar de largo por CpiGuard como si la guarda no estuviera ahí. Esos eran poderes que el mint tiene sobre el tenedor. Todos y cada uno decían: no importa lo que quiera tu cuenta, el mint decide.

Ahora la imagen espejo. Algunas extensiones no son poderes sobre ti. Son el veto propio de tu cuenta.

Antes de hablar de cualquiera de ellas, deja tu banco de trabajo en el estado en que lo dejamos. Arranca tu simnet local y vuelve a correr el criterio de m02-l2, porque esta lección se apila directo sobre ese artefacto:

```bash
# surfpool 1.2.1, installed in m02-l1 (brew install txtx/taps/surfpool if you
# skipped it; other OSes: github.com/txtx/surfpool). Checked 2026-08-22.
surfpool start --no-tui --no-studio

# in a second terminal, from your course workspace:
npx tsx labs/m02-l2/verify-authorities.ts
```

Si eso todavía imprime verde (en un simnet, la única línea nombrada `SKIPPED: PermissionedBurn` del sondeo de cluster del criterio cuenta como verde), tu capa de autoridad está intacta y podemos construir encima. Si no, arréglalo primero; nada de esta lección tiene sentido sobre un piso roto.

Mientras eso corre, mantén el giro en la cabeza, porque es la lección entera. Una insignia SPROUT soulbound que físicamente no se puede transferir, por nadie, dueño incluido. Una cuenta de tesorería que rechaza cualquier depósito que llegue sin un memo adjunto. Una cuenta cuyo campo de dueño quedó soldado para siempre. La tesorería con memo es una guarda del lado de la cuenta que el tenedor activa. El dueño soldado también es del lado de la cuenta, pero nadie lo activa: cada ATA que hayas creado alguna vez ya lleva ImmutableOwner de forma automática, y la lección te lo comprueba. La primera es la excepción deliberada en la taxonomía de esta lección: NonTransferable es una extensión del lado del mint que fija el emisor, archivada aquí de todos modos porque la parte que protege es la integridad de la credencial, no el control del emisor, y es al tenedor, no al emisor, a quien se le rechaza la transferencia. Todas son rechazos, y hoy vas a configurar cuatro de ellas y después comprobar, con transacciones que revierten, que cada una de verdad rechaza.

Esta lección desarrolla las protecciones del lado de la cuenta de Token-2022: NonTransferable, MemoTransfer, CpiGuard (revisitada desde el otro lado del escudo) e ImmutableOwner. Vas a aprender por qué activar NonTransferable en un mint fuerza un PAR específico de extensiones sobre cada cuenta de tenedor, por qué MemoTransfer le cobra en silencio un impuesto a todo el que alguna vez te transfiera, y por qué cada ATA que hayas creado alguna vez ya lleva ImmutableOwner sin que lo pidieras. En el lab extiendes el toolkit de SPROUT con un mint de insignia soulbound y una tesorería con memo obligatorio, y el criterio no es "funciona": el criterio es que las operaciones prohibidas reviertan, a propósito, con los códigos de error exactos que están en la página.

Dónde estás en la rampa de autonomía, dicho en voz alta: en m02-l1 llenaste unos cuantos huecos TODO dentro de una construcción por lo demás trabajada, y en m02-l2 te entregué código de configuración completo y tú lo corriste. Hoy la config sigue trabajada para ti, pero las dos afirmaciones estructurales son problemas de completar: yo te digo qué comprobar y tú escribes la línea que lo comprueba antes de ver la mía. El Challenge del final es totalmente en solitario, sin guía, y esa es la forma de cada lección de aquí en adelante. Las rueditas se van quitando un perno a la vez, según el calendario.

## El veto de la cuenta

### Dos caras del mismo TLV

Esta es la forma más limpia que conozco de tener el catálogo de Token-2022 en la cabeza: cada extensión responde una de dos preguntas. ¿Quién puede actuar sobre este token incluso contra la voluntad del tenedor? Ese es el lado del mint, la lección pasada. Y: ¿qué puede rechazar esta cuenta, incluso cuando todos los demás dicen que sí? Ese es el lado de la cuenta, esta lección.

La distinción es física, no retórica. Viste en m01-l2 que un mint y una cuenta de token son los dos una base de 165 bytes más un byte de tipo más un recorrido TLV. Las extensiones del lado del mint viven en el TLV del mint; las protecciones del lado de la cuenta viven en el TLV de la cuenta del tenedor. Cuando tu inspector `decode-mint` recorre PYUSD imprime entradas de mint. Cuando lo apuntas a tu propia ATA más adelante en el lab, vas a ver entradas de cuenta. Mismos bytes, mismo recorrido, políticas opuestas.

![Dos cajas contrastan las extensiones de poder del lado del mint con las extensiones de rechazo del lado de la cuenta, con una flecha que muestra NonTransferable en el mint forzando NonTransferableAccount e ImmutableOwner dentro de las cuentas de los tenedores.](assets/v01-diagram.webp)

Una nota al pie, y corta, ya que m02-l1 contó la historia del archivo: las protecciones del lado de la cuenta que estás conectando hoy son mecánicas de la era 2026 que los cursos canónicos congelados nunca alcanzaron. Los docs que sí existen describen cada extensión por separado; lo que no enseñan es la parte que muerde, los emparejamientos forzados y los impuestos de integración. Así que ahí es donde vamos a gastar nuestro tiempo.

### NonTransferable: soulbound por construcción

Primero el problema. Digamos que Overgrowth, la cooperativa agrícola cuya economía on-chain este curso viene construyendo desde que se especificó SPROUT, quiere otorgar insignias SPROUT por completar una temporada de harvest: un token fungible, decimals 0, una unidad por logro. El punto entero de una insignia es que TÚ te la ganaste. En el momento en que las insignias son transferibles hay un mercado, y en el momento en que hay un mercado, la insignia deja de significar "esta billetera hizo la cosa" y pasa a significar "esta billetera pagó por la prueba de que otra billetera hizo la cosa." Para las credenciales, la transferibilidad no es una función que se está quitando. Es el ataque.

El desbloqueo es una sola extensión a nivel de mint: NonTransferable, tipo de extensión 9 en el catálogo que tu inspector ya mapea. Activada al crear el mint (como la mayoría de las extensiones, no se puede agregar después de la inicialización), hace que cada transferencia de este token revierta a nivel de programa. No "revierte salvo para el admin", no "revierte salvo que enrutes con astucia". El programa de token mismo rechaza. Acuñar y quemar siguen funcionando, que es exactamente el ciclo de vida que quiere una insignia: el emisor te la acuña, nadie la mueve, tú o el emisor pueden quemarla para limpiar.

Ahora deriva la parte que los docs enuncian pero nunca explican. Supón que el programa solo bloqueara `transfer` y se detuviera ahí. Tienes una insignia en una cuenta de token. Las cuentas de token tienen una autoridad de dueño, y el programa de token base siempre te ha dejado reasignarla con SetAuthority. Así que "vendes tu insignia" vendiendo la cuenta entera: firmas un SetAuthority que le pasa la cuenta al comprador. Nunca corrió ninguna instrucción de transferencia, el saldo nunca se movió entre cuentas, y la garantía soulbound está muerta. El token no se movió; el alma sí.

Por eso el emparejamiento es forzado. Cuando inicializas una cuenta de token para un mint NonTransferable, Token-2022 se niega a crearla salvo que la cuenta lleve ImmutableOwner, y le estampa a la cuenta una extensión marcadora, NonTransferableAccount (tipo 13), que deja registrado que esta cuenta guarda tokens soulbound. El par [NonTransferableAccount, ImmutableOwner] aparece en cada cuenta de tenedor, siempre, o la cuenta no puede existir. Cierra la puerta de la transferencia y tienes que soldar también la puerta de la propiedad, o la primera puerta era decoración. Eso no es una convención que sigas. El programa lo hace cumplir, y en el lab vas a leer las dos entradas sacándolas del TLV de tu propia cuenta.

![Diagrama de flujo que muestra un mint no transferible bloqueando transferencias con el error 0x25 y, mediante el par forzado de NonTransferableAccount más ImmutableOwner, bloqueando también la reasignación de dueño con el error 0x22, y cerrando así el resquicio de la venta de la cuenta.](assets/v02-flowchart.webp)

La forma tiene un nombre que vale la pena cargar: soulbound-fungible. No un NFT con supply 1 y un estándar de metadatos atornillado encima, que es adonde va el módulo 6. Un mint fungible ordinario, decimals 0, el supply que quieras, cuyas unidades están soldadas a quien sea que las recibió. Una insignia aquí es solo un número que no se puede mover.

Y la aplicación corre antes de lo que supondrías. La inicialización normal de una cuenta agrega ImmutableOwner por ti, así que en el camino feliz la verificación nunca se dispara de forma visible. Token-2022 resguarda el lado del mint de todos modos: un `MintTo` hacia una cuenta que carece de propiedad inmutable falla con el error de programa personalizado 0x26 (decimal 38), y el mensaje es la decisión de diseño puesta por escrito, "Non-transferable tokens can't be minted to an account without immutable ownership". La propia suite de pruebas del programa tiene que esforzarse para llegar a ese error, recreando un mint en la misma dirección con un set de extensiones distinto. La ruta del mint confidencial lleva la guarda idéntica, con el ataque deletreado en un comentario del código fuente: sin ella, alguien podría acuñar hacia una cuenta de dueño mutable y después abrirse camino con SetAuthority hasta el control de los tokens. Dos rutas de código independientes rechazando el mismo agujero es una señal decente de que el agujero es real.

Una pregunta de diseño que deberías estarte haciendo, ya que construiste la alternativa la lección pasada: ¿por qué no simplemente congelar? DefaultAccountState(Frozen) con una autoridad de congelamiento que nunca descongela también produce tokens que nadie puede mover. La guía de extensiones de Token-2022 traza la comparación ella misma y nombra la diferencia: NonTransferable "es muy parecida a emitir un token y después congelar la cuenta, pero le deja al dueño quemar y cerrar la cuenta si así lo quiere." Una cuenta congelada es inerte para todos, su tenedor incluido, así que tu aprendiz queda atrapado pagando rent por una insignia que ni siquiera puede limpiar. Una cuenta soulbound sigue siendo suya para quemar y cerrar. Y congelar exige una autoridad viva que tienes que conservar, podrías abusar y podrías perder. NonTransferable no exige a nadie. Para una credencial el veredicto es fácil: pon la garantía en los bytes del mint, no en tu buena conducta continuada.

Una mina de compatibilidad antes de seguir, directo de la matriz de conflictos que construiste en m01-l4: NonTransferable con ConfidentialTransferMint es inválida salvo que ConfidentialMintBurn también esté presente. Tiene sentido en cuanto lo dices en voz alta (un token que no puede transferirse no tiene uso para transferencias confidenciales, a menos que la maquinaria confidencial esté ahí para los montos de acuñación y quema). Tu validador check-combo de esa lección ya marca esto como la regla 5 de la matriz; confía en él cuando compongas.

### ImmutableOwner: el defecto que nunca notaste

ImmutableOwner merece su propio momento, porque llevas años usándola sin haber consentido en ella, y eso es algo bueno.

La extensión hace un solo trabajo: bloquea la autoridad de dueño de la cuenta para que SetAuthority nunca pueda reasignarla. ¿Por qué importaría eso fuera de las insignias soulbound? Por cómo funcionan las ATA. La dirección de una cuenta de token asociada se deriva de forma determinista de (dueño, programa de token, mint). Todo el mundo, billeteras, DEXes, scripts de airdrop, computa la dirección de tu ATA y manda fondos ahí sin preguntarte. Ahora imagina que la propiedad fuera reasignable: le reasignas la propiedad de tu ATA a alguien más, la dirección se sigue derivando de TU pubkey, y cada remitente futuro que compute "la ATA de la billetera X" ahora está fondeando una cuenta controlada por alguien que no es X. Toda una clase de trucos de toma de cuentas y de fondos mal dirigidos vive en ese hueco.

Así que el programa de ATA lo cerró: cada cuenta de token asociada ya viene con ImmutableOwner por defecto. En Token-2022 es una entrada TLV real que hace cumplir de verdad. Y aquí hay un detalle que de verdad me encanta: el programa SPL Token clásico no puede almacenar extensiones en absoluto, así que cuando el programa de ATA le manda InitializeImmutableOwner, el token clásico acepta la instrucción como un no-op y registra "Please upgrade to SPL Token 2022 for immutable owner support". Un encogimiento de hombros cortés, preservado en cada creación de ATA clásica que hayas simulado alguna vez. La invariante de dirección derivada importa tanto que un programa la hace cumplir y el otro al menos hace el gesto. Entre los defectos silenciosos, este es una bendición.

![Muchos remitentes computan la misma dirección de ATA derivada, así que reasignar su dueño redirigiría los depósitos futuros, y ImmutableOwner hace que esa reasignación revierta con el error 0x22.](assets/v03-diagram.webp)

El rechazo que te compra es concreto, y lo vas a disparar en el lab: un SetAuthority con tipo de autoridad AccountOwner contra una cuenta con ImmutableOwner revierte con el error de programa personalizado 0x22 (decimal 34). En una cuenta así, la reasignación de dueño desapareció en vez de quedar meramente restringida.

### MemoTransfer: la cuenta que exige un comprobante

MemoTransfer invierte la dirección del control de una forma que ninguna otra extensión logra. Todo lo demás que hemos tocado configura lo que un mint o una cuenta puede hacer. MemoTransfer configura lo que todos los DEMÁS tienen que hacer para llegar a ti.

La mecánica: MemoTransfer (tipo 8) es una extensión de cuenta, activada por el dueño de la cuenta, sobre la cuenta, después de su creación. Una vez activada, cualquier transferencia entrante tiene que ir inmediatamente precedida en la transacción por una instrucción de memo, el programa SPL Memo escribiendo una cadena en el log de la transacción. Sin memo, no hay depósito: la transferencia revierte con el error de programa personalizado 0x24 (decimal 36), y el log del programa lo deletrea en inglés llano: "Error: No memo in previous instruction required for recipient to receive a transfer". (La cadena Display del propio tipo de error lleva un punto y coma después de "instruction"; la cadena que el programa registra de verdad, no. Iguala lo que imprime el log cuando lo busques con grep.) El caso de uso de Overgrowth se escribe solo: una tesorería de cooperativa donde cada pago entrante tiene que cargar una referencia de liquidación, hecha cumplir por el runtime en vez de por una hoja de cálculo y la esperanza. Los exchanges corren el mismo patrón para etiquetar depósitos, y a los equipos de cumplimiento les encanta porque el rastro de auditoría está en el libro mayor mismo.

Pero mira quién paga. Tú no: activaste una instrucción y te quedaste con contabilidad impuesta por el runtime. El costo cae sobre cada remitente, para siempre. Un socio que integra tu tesorería escribe una transferencia normal y correcta, la prueba contra cuentas normales, la entrega, y revierte en producción contra la tuya. Nada en la API de transferencia les avisó; el requisito vive en el TLV de TU cuenta, y su código nunca miró. Esto no es hipotético. La checklist de integración de Token-2022 de Meteora les dice a los integradores, textualmente, que "ensure destinations accept memo-required", que es un DEX documentando la configuración de tu cuenta como un peligro que sus socios tienen que sortear en código. Cuando la checklist de una plataforma viva nombra tu extensión, créele a la checklist.

![Dos carriles de transacción muestran una transferencia sin memo revirtiendo con el error 0x24 en la barrera de memo del destino, mientras que una transferencia idéntica precedida por una instrucción de memo sí llega.](assets/v04-flowchart.webp)

Anticipando la pregunta que deberías estarte haciendo: ¿puede el dueño apagarlo? Sí. MemoTransfer es simétrica, activar y desactivar existen las dos, las dos firmadas por el dueño. Es el veto del tenedor en el sentido más puro: entras, sales, y mientras está encendida, el runtime te hace cumplir el papeleo por ti.

### CpiGuard, revisitada desde el otro lado

Conociste CpiGuard (tipo 11) la lección pasada como la cosa que PermanentDelegate deja en ridículo. Déjame darle una audiencia más justa ahora que estamos del lado de la cuenta de la mesa, porque dentro de su jurisdicción real es una protección seria.

La amenaza a la que apunta: firmas una transacción para algún programa, un juego, un marketplace, un botón de reclamar de aspecto inocente, y enterrada en la ejecución de ese programa hay una CPI que llama al programa de token con autoridades que técnicamente firmaste. Aprobar un delegado, cambiar el destino de cierre, transferir con tu firma de dueño. Autorizaste UNA cosa al nivel superior; el programa gastó tu autoridad en otras. CpiGuard, activada y firmada por el dueño en la cuenta (como MemoTransfer, conmutable en los dos sentidos), bloquea las operaciones peligrosas con forma de autoridad cuando llegan vía CPI en vez de desde una instrucción de nivel superior que firmaste de forma visible. Las acciones que cubre la guarda tienen que pasar donde puedas verlas, o no pasar en absoluto.

La advertencia honesta, y sigue siendo estructural desde l2: CpiGuard defiende la superficie de autoridad de la cuenta misma. Un PermanentDelegate en el mint no es la autoridad de la cuenta. Es un poder a nivel de mint que la cuenta nunca consintió, y pasa de largo por la guarda todas las veces, cosa que comprobaste tú mismo con tus propias dos transacciones la lección pasada. Así que ubica CpiGuard correctamente en tu modelo mental: protección real contra programas que hacen mal uso de autoridades que tú delegaste, cero protección contra poderes que el mint se reservó por encima de ti. Una guarda en tu puerta de entrada, en una casa donde el casero se quedó una llave maestra. Si la lista de trampas dice "suponer que CpiGuard es una defensa completa", el arreglo es sostener los dos hechos a la vez, y nunca dejar que una afirmación de seguridad de billetera se apoye solo en la guarda.

![Diagrama de CpiGuard como un escudo que bloquea las operaciones de autoridad invocadas por CPI mientras que las acciones de nivel superior firmadas por el dueño pasan por una barrera y un movimiento de PermanentDelegate a nivel de mint pasa por encima del escudo sin ser tocado.](assets/v05-diagram.webp)

### Lo que cuestan estas protecciones

Cada lección de este módulo nombra su trade-off, y esta tiene el más claro del curso hasta ahora: las protecciones del tenedor vuelven un token más seguro y más auditable al estrechar quién puede transaccionar con él, y el costo siempre se empuja hacia afuera, sobre alguien que no eres tú.

NonTransferable mata los mercados secundarios por diseño; para una insignia ese es el punto, para cualquier cosa pensada para negociarse es fatal, y ningún DEX la va a enrutar jamás. MemoTransfer te vuelve incompatible con todo remitente que no adjunte memos, un impuesto de integración recaudado de socios que nunca han leído el TLV de tu cuenta, que es por lo que existe la checklist de Meteora. CpiGuard estrecha qué programas pueden componer de forma útil con tu cuenta, y la protección es real pero parcial. ImmutableOwner es la más barata de las cuatro, de costo casi cero precisamente porque las ATA la volvieron universal antes de que nadie pudiera construir sobre el comportamiento inseguro.

Así que aquí está la regla de decisión, tan sin rodeos como la puedo poner. Echa mano de NonTransferable solo cuando la negociabilidad sea la amenaza y no la función, porque no puedes deshacerla después de `initializeMint`. Echa mano de MemoTransfer solo cuando controlas los dos extremos del cable, o cuando las contrapartes son tan pocas que puedes avisarle a cada una a mano. CpiGuard es casi gratis en cuentas que controlas y una mala cosa de suponer en cuentas que no. ImmutableOwner ya la tienes y no la elegiste. Si no puedes nombrar la operación exacta que quieres que se rechace y la persona exacta a la que el rechazo va a incomodar, no estás eligiendo una protección. Estás decorando un mint.

![Tabla comparativa de NonTransferable, ImmutableOwner, MemoTransfer y CpiGuard que muestra dónde vive cada una, qué rechaza, su código de error observado, y quién carga con el costo.](assets/v06-comparison.webp)

Fíjate en el tema: las cuatro vuelven cosas imposibles en vez de posibles, de forma selectiva, y la disciplina de ingeniería que exigen es comprobar la imposibilidad en vez de afirmarla. Que es precisamente lo que hace el lab.

## Lab: convierte el rechazo en una prueba

El artefacto que esta lección le agrega al toolkit de Overgrowth es `sprout-mint-protections`: un mint de insignia soulbound con su par de cuenta forzado, y una tesorería con memo obligatorio que rechaza los depósitos sin etiquetar, todo comprobado por un script de criterio donde las afirmaciones son reversiones. Se construye al lado de tus mints de autoridad de m02-l2; estás apilando una segunda capa, no reemplazando la primera. Corrí este mismo criterio cuatro veces mientras escribía esta lección, sobre un simnet fresco cada vez: los mismos tres rechazos, los mismos códigos de error, en cada corrida. La tuya debería ser igual de aburrida.

![Pipeline de los ocho pasos del lab, desde el fondeo pasando por la creación del mint soulbound, la afirmación del par forzado, tres reversiones esperadas, el depósito con memo y CpiGuard, terminando en un criterio en verde.](assets/v07-flowchart.webp)

1. **Workspace y pins.** Trabaja en la raíz del workspace, el layout que estableció m02-l2 (deps compartidas en el `package.json` raíz, código de lección bajo `labs/`), con el simnet del inicio todavía corriendo. Los pins son el set de m02-l1 más un recién llegado, memo, y la misma regla del párrafo de pins de esa lección decide cada versión aquí: el minor actual que hace peer con kit ^7, vuelve a verificar cuando leas esto.

   ```bash
   npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 \
               @solana-program/memo@0.12.0 @solana-program/system@0.13.0
   npm install -D tsx@4.23.12 typescript@5.9.3   # already there if you did the m02-l2 root install
   # memo 0.12.0 and system 0.13.0: the kit-^7-peer versions of each,
   # verified against npm 2026-09-05. Newer minors peer kit ^8.
   ```

2. **Arma el criterio.** Crea `labs/m02-l3/verify-protections.ts`. Imports y tres helpers: un enviador de transacciones (el mismo pipe de kit que vienes construyendo desde m01-l3, ahora factorizado aparte porque vamos a mandar nueve transacciones), un `expectRevert` que FALLA si la operación tiene éxito, y un creador de ATA. Lee `expectRevert` dos veces; es la postura de ingeniería de la lección en ocho líneas. Que la op prohibida pase es la condición de error.

   ```ts
   // labs/m02-l3/verify-protections.ts
   // Gate for m02-l3: every holder-protection extension must refuse the op it exists to refuse.
   // Run against a local surfpool simnet: `surfpool start --no-tui --no-studio` in another terminal, then
   // `npx tsx labs/m02-l3/verify-protections.ts`.
   import {
     airdropFactory,
     assertIsTransactionWithBlockhashLifetime,
     appendTransactionMessageInstructions,
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
   import { getCreateAccountInstruction } from '@solana-program/system';
   import { getAddMemoInstruction } from '@solana-program/memo';
   import {
     AuthorityType,
     ExtensionType,
     TOKEN_2022_PROGRAM_ADDRESS,
     extension,
     fetchToken,
     findAssociatedTokenPda,
     getCreateAssociatedTokenIdempotentInstructionAsync,
     getEnableCpiGuardInstruction,
     getEnableMemoTransfersInstruction,
     getInitializeMintInstruction,
     getInitializeNonTransferableMintInstruction,
     getMintSize,
     getMintToInstruction,
     getReallocateInstruction,
     getSetAuthorityInstruction,
     getTransferCheckedInstruction,
   } from '@solana-program/token-2022';

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   const airdrop = airdropFactory({ rpc, rpcSubscriptions });

   async function sendTx(feePayer: KeyPairSigner, instructions: Instruction[]) {
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const tx = await pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions(instructions, m),
       (m) => signTransactionMessageWithSigners(m),
     );
     assertIsTransactionWithBlockhashLifetime(tx);
     await sendAndConfirm(tx, { commitment: 'confirmed' });
   }

   async function expectRevert(label: string, run: () => Promise<void>) {
     try {
       await run();
     } catch {
       console.log(`PASS  ${label}: reverted as required`);
       return;
     }
     throw new Error(`FAIL  ${label}: the disallowed op went through`);
   }

   async function createAta(payer: KeyPairSigner, mint: KeyPairSigner, owner: KeyPairSigner) {
     const [ata] = await findAssociatedTokenPda({
       owner: owner.address,
       mint: mint.address,
       tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
     });
     await sendTx(payer, [
       await getCreateAssociatedTokenIdempotentInstructionAsync({
         payer,
         owner: owner.address,
         mint: mint.address,
         tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
       }),
     ]);
     return ata;
   }
   ```

3. **El mint de insignia soulbound.** Abre `main()`, fondea dos actores y crea el mint. El orden dentro de la transacción de creación es la misma regla que aprendiste en m02-l1 y sigue mordiendo: los inicializadores de extensión corren ANTES que `initializeMint`, porque una vez que el mint está inicializado su set de extensiones queda sellado. `getMintSize` con la lista de extensiones computa el espacio exacto que incluye el TLV, la misma matemática que tu inspector le sacó por ingeniería inversa a `try_calculate_account_len` en m01-l2.

   ```ts
   async function main() {
     const payer = await generateKeyPairSigner();
     const alice = await generateKeyPairSigner();
     const bob = await generateKeyPairSigner();
     await airdrop({
       commitment: 'confirmed',
       recipientAddress: payer.address,
       lamports: lamports(5_000_000_000n),
     });
     await airdrop({
       commitment: 'confirmed',
       recipientAddress: alice.address,
       lamports: lamports(1_000_000_000n),
     });

     // ---- 1. Soulbound badge mint: NonTransferable forces the account-extension pair ----
     const badgeMint = await generateKeyPairSigner();
     const badgeSpace = BigInt(getMintSize([extension('NonTransferable', {})]));
     const badgeRent = await rpc.getMinimumBalanceForRentExemption(badgeSpace).send();
     await sendTx(payer, [
       getCreateAccountInstruction({
         payer,
         newAccount: badgeMint,
         space: badgeSpace,
         lamports: badgeRent,
         programAddress: TOKEN_2022_PROGRAM_ADDRESS,
       }),
       getInitializeNonTransferableMintInstruction({ mint: badgeMint.address }),
       getInitializeMintInstruction({
         mint: badgeMint.address,
         decimals: 0,
         mintAuthority: payer.address,
       }),
     ]);
   ```

4. **Cuentas de tenedor, y el par forzado: primero tu afirmación.** Crea ATA para alice y bob y acúñale a alice su única insignia. Después párate, porque este es el primer problema de completar. Sabes por la teoría que la cuenta de alice ahora tiene que llevar NonTransferableAccount e ImmutableOwner las dos, o la teoría está mal. `fetchToken` devuelve la cuenta decodificada con `data.extensions` como un Option sobre un array de entradas `{ __kind: ... }`. Escribe la afirmación tú mismo antes de seguir scrolleando: haz el fetch, desenvuelve el option, junta los kinds, y lanza si falta cualquiera de los dos kinds requeridos. Después compárala con la mía:

   ```ts
     const aliceBadge = await createAta(payer, badgeMint, alice);
     const bobBadge = await createAta(payer, badgeMint, bob);
     await sendTx(payer, [
       getMintToInstruction({
         mint: badgeMint.address,
         token: aliceBadge,
         mintAuthority: payer,
         amount: 1n,
       }),
     ]);

     // The forced pair: NonTransferableAccount + ImmutableOwner on the holder account.
     const aliceBadgeAccount = await fetchToken(rpc, aliceBadge);
     const exts = aliceBadgeAccount.data.extensions;
     const kinds = exts.__option === 'Some' ? exts.value.map((e) => e.__kind) : [];
     for (const required of ['NonTransferableAccount', 'ImmutableOwner'] as const) {
       if (!kinds.includes(required)) {
         throw new Error(`FAIL  forced pair: holder account is missing ${required}`);
       }
     }
     console.log(`PASS  forced pair: holder account carries [${kinds.join(', ')}]`);
   ```

   En mi corrida la línea de PASS imprimió el par en orden de creación:

   ```text
   PASS  forced pair: holder account carries [ImmutableOwner, NonTransferableAccount]
   ```

   Nunca pediste ninguna de las dos extensiones. Inicializaste un mint NonTransferable y una ATA ordinaria, y el programa puso las dos entradas ahí porque la cuenta no podía existir legalmente sin ellas. Para una segunda opinión directo de los bytes, apunta tu propio inspector a la cuenta (`npx tsx decode-mint.ts <aliceBadge address> http://127.0.0.1:8899`): el recorrido TLV que escribiste en m01-l2 lee las cuentas de token exactamente igual que los mints, y va a imprimir el tipo 7 y el tipo 13 junto a los nombres.

![Salida anotada del inspector para la cuenta del tenedor de la insignia, que muestra una base de 165 bytes, el byte de tipo de cuenta 2, y dos entradas TLV forzadas de longitud cero, ImmutableOwner tipo 7 y NonTransferableAccount tipo 13.](assets/v08-annotated-code.webp)

5. **Dos rechazos, comprobados.** Ahora vuelve falsable la teoría. Alice, la dueña legítima, firma una transferencia de su propia insignia hacia bob: tiene que revertir. Después intenta pasarle la cuenta misma a bob vía SetAuthority: tiene que revertir. Las dos pasan por `expectRevert`, así que si cualquiera tiene éxito, el criterio muere ruidosamente.

   ```ts
     // A soulbound badge cannot move, even with the owner signing.
     await expectRevert('NonTransferable transfer', () =>
       sendTx(payer, [
         getTransferCheckedInstruction({
           source: aliceBadge,
           mint: badgeMint.address,
           destination: bobBadge,
           authority: alice,
           amount: 1n,
           decimals: 0,
         }),
       ]),
     );

     // ImmutableOwner refuses owner reassignment on the same account.
     await expectRevert('ImmutableOwner reassignment', () =>
       sendTx(payer, [
         getSetAuthorityInstruction({
           owned: aliceBadge,
           owner: alice,
           authorityType: AuthorityType.AccountOwner,
           newAuthority: bob.address,
         }),
       ]),
     );
   ```

   Si quieres ver los rechazos crudos en vez del catch tragándoselos, simula cualquiera de las dos transacciones y lee los logs. La transferencia muere con `custom program error: 0x25` (decimal 37, el rechazo de no transferible de Token-2022) y la reasignación con `custom program error: 0x22` (decimal 34, el rechazo de dueño inmutable). Saqué los dos códigos de los logs de simulación de mi propia corrida de simnet del 2026-08-22; vale la pena reconocerlos de vista, porque en producción llegan sin lección adjunta.

6. **La tesorería con memo obligatorio.** Segunda mitad del artefacto. Crea un mint transferible simple que hace de SPROUT (decimals 6, sin extensiones de mint: la protección que estamos probando vive en la CUENTA), una cuenta de remitente fondeada para el payer, y una ATA de tesorería cuya dueña es alice. Después la aceptación del lado del dueño, y fíjate en el baile de dos pasos: la ATA se creó a su tamaño mínimo, así que alice primero la agranda con `Reallocate` nombrando los tipos de extensión para los que quiere espacio, después activa `EnableMemoTransfers`. Las dos instrucciones son suyas para firmar, de nadie más. Eso es lo que quiere decir "protección del tenedor" en los bytes.

   ```ts
     // ---- 2. Memo-required treasury: MemoTransfer rejects memo-less deposits ----
     const sproutMint = await generateKeyPairSigner();
     const sproutSpace = BigInt(getMintSize());
     const sproutRent = await rpc.getMinimumBalanceForRentExemption(sproutSpace).send();
     await sendTx(payer, [
       getCreateAccountInstruction({
         payer,
         newAccount: sproutMint,
         space: sproutSpace,
         lamports: sproutRent,
         programAddress: TOKEN_2022_PROGRAM_ADDRESS,
       }),
       getInitializeMintInstruction({
         mint: sproutMint.address,
         decimals: 6,
         mintAuthority: payer.address,
       }),
     ]);

     const senderSprout = await createAta(payer, sproutMint, payer);
     const treasury = await createAta(payer, sproutMint, alice);
     await sendTx(payer, [
       getMintToInstruction({
         mint: sproutMint.address,
         token: senderSprout,
         mintAuthority: payer,
         amount: 1_000_000_000n,
       }),
     ]);

     // Holder-side opt-in: grow the account, then flip the requirement on.
     await sendTx(alice, [
       getReallocateInstruction({
         token: treasury,
         payer: alice,
         owner: alice,
         newExtensionTypes: [ExtensionType.MemoTransfer],
       }),
       getEnableMemoTransfersInstruction({ token: treasury, owner: alice }),
     ]);
   ```

7. **El remitente ingenuo falla; el remitente informado paga el impuesto.** Segundo problema de completar, y este es sobre ser el remitente. Primera transacción: un `TransferChecked` perfectamente normal de 25 SPROUT hacia la tesorería, envuelto en `expectRevert`, porque estás haciendo el papel del socio que nunca leyó el TLV. Segunda transacción: la misma transferencia, arreglada. Antes de mirar mi versión, responde desde la teoría: ¿qué exige exactamente el arreglo, y de qué lado del cable vive? Escribe la transacción arreglada, después compara:

   ```ts
     await expectRevert('memo-less deposit', () =>
       sendTx(payer, [
         getTransferCheckedInstruction({
           source: senderSprout,
           mint: sproutMint.address,
           destination: treasury,
           authority: payer,
           amount: 25_000_000n,
           decimals: 6,
         }),
       ]),
     );

     // Same transfer, memo attached first: the sender pays the integration tax.
     await sendTx(payer, [
       getAddMemoInstruction({ memo: 'harvest-settlement:2026-08-22' }),
       getTransferCheckedInstruction({
         source: senderSprout,
         mint: sproutMint.address,
         destination: treasury,
         authority: payer,
         amount: 25_000_000n,
         decimals: 6,
       }),
     ]);
     console.log('PASS  memo-carrying deposit landed');
   ```

   El arreglo entero es un `getAddMemoInstruction` puesto antes de la transferencia, en la transacción del remitente. La tesorería no cambió nada entre el depósito que falla y el que llega. Simula la versión que falla y el log del programa te entrega la historia completa: `Error: No memo in previous instruction required for recipient to receive a transfer`, error de programa personalizado 0x24. Esa línea de log es de lo que la checklist de Meteora está defendiendo a sus integradores.

8. **CpiGuard, activada y verificada, y corre el criterio.** Última capa: el payer endurece la cuenta del remitente con CpiGuard, el mismo baile de Reallocate-y-después-activar, y afirmamos que la extensión de verdad quedó en el TLV. Lo que no hacemos es demostrar el bloqueo mismo, y quiero ser franco sobre por qué: CpiGuard rechaza operaciones que llegan vía CPI, así que dispararla con honestidad necesita un programa desplegado haciendo la llamada, y tú comprobaste tanto el bloqueo como el bypass de PermanentDelegate con exactamente ese montaje en m02-l2. Esta afirmación es de presencia; la de m02-l2 era de comportamiento; juntas son el cuadro completo.

   ```ts
     // ---- 3. CpiGuard: enabled and present (the bypass demo lives in m02-l2) ----
     await sendTx(payer, [
       getReallocateInstruction({
         token: senderSprout,
         payer,
         owner: payer,
         newExtensionTypes: [ExtensionType.CpiGuard],
       }),
       getEnableCpiGuardInstruction({ token: senderSprout, owner: payer }),
     ]);
     const guarded = await fetchToken(rpc, senderSprout);
     const guardedKinds =
       guarded.data.extensions.__option === 'Some'
         ? guarded.data.extensions.value.map((e) => e.__kind)
         : [];
     if (!guardedKinds.includes('CpiGuard')) {
       throw new Error('FAIL  CpiGuard: extension not present after enable');
     }
     console.log('PASS  CpiGuard enabled on the sender account');

     console.log('\nAll holder-protection assertions hold. m02-l3 gate: green.');
   }

   main().catch((err) => {
     console.error(err);
     process.exit(1);
   });
   ```

   Córrelo:

   ```bash
   npx tsx labs/m02-l3/verify-protections.ts
   ```

   La salida esperada, textual de mi corrida del 2026-08-22:

   ```text
   PASS  forced pair: holder account carries [ImmutableOwner, NonTransferableAccount]
   PASS  NonTransferable transfer: reverted as required
   PASS  ImmutableOwner reassignment: reverted as required
   PASS  memo-less deposit: reverted as required
   PASS  memo-carrying deposit landed
   PASS  CpiGuard enabled on the sender account

   All holder-protection assertions hold. m02-l3 gate: green.
   ```

   Seis PASS, tres de los cuales son fallas comportándose correctamente. Ese es el criterio.

## Challenge

Solo, sin guía, y este es el ejercicio de config de protección del tenedor hacia el que el módulo viene construyendo: elige una extensión de control que hayamos cubierto en este módulo, la que sea, del lado del mint o del lado de la cuenta, y escribe `labs/m02-l3/challenge-control.ts` que la configure sobre un mint o una cuenta desechable y compruebe con una afirmación al estilo `expectRevert` que bloquea una operación prohibida. DefaultAccountState(Frozen) rechazando una transferencia hacia una cuenta nunca descongelada es una elección limpia; también lo es una variante fresca de MemoTransfer con la ruta de desactivación también afirmada. Tu barra de aceptación, la misma que la del lab: la op prohibida tiene que revertir, la afirmación tiene que FALLAR ruidosamente si alguna vez deja de revertir, y un comentario de una línea tiene que nombrar el código de error que observaste y cómo lo llama el programa. Si tu script pasa al primer intento, sospecha; borra la instrucción que la activa y confirma que el criterio se pone en rojo por la razón correcta antes de confiar en el verde.

## Lo que SPROUT rechaza ahora

Haz balance de la escalera de artefactos, porque se acumula sin hacer ruido. R1 lee cualquier mint o cuenta hasta el TLV. La capa de economía enruta el valor. La capa de autoridad de m02-l2 dice quién puede mover, congelar y recuperar por la fuerza. Y desde hoy, `sprout-mint-protections` agrega la otra voz de la conversación: una insignia que no puede dejar a quien se la ganó, una tesorería que rechaza dinero indocumentado, cuentas cuya propiedad no se puede reasignar, y una guarda cuyos límites puedes enunciar con precisión porque los mediste desde los dos lados. Eso no lo leíste en una matriz. Hiciste que cada rechazo ocurriera y lo atrapaste en una prueba.

Si alguna reversión no se disparó en tu máquina, o se disparó con un código distinto de los que están en esta página, ese es exactamente el tipo de reporte del que quiero enterarme, con tus logs de simnet adjuntos; el tren de la toolchain se mueve cada mes y el criterio existe para atraparlo moviéndose.

SPROUT ahora hace cumplir quién puede moverlo y qué rechazan sus cuentas. Pero búscalo en cualquier billetera y sigue siendo una pubkey con un saldo: sin nombre, sin símbolo, sin imagen, nada que un humano pueda renderizar. La lección siguiente arreglamos eso donde Token-2022 quiere que se arregle, metadatos nativos guardados en el mint mismo, y tu inspector llega a leer un token que por fin se presenta.
