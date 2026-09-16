# El chequeo de salud del camino de escritura: una transferencia

## Resumen

m08-l3 le dio al poller de Rust sus sondas de la blockchain: lecturas tipadas con reqwest parseadas con serde_json, una taxonomía de fallas con thiserror para todo lo que el RPC puede lanzar, y un re-ship de la imagen de GHCR por el pipeline de M6 sin tocarle una línea. Cada superficie de la estación ahora lee la blockchain. Ninguna le escribe. Hoy eso cambia: entregas `tx-check`, un script que firma una transferencia de SOL real, la aterriza en devnet, imprime la firma, y sale distinto de cero si algún eslabón de esa cadena miente. Esa firma confirmada es SHIP #5, y es la señal de salud más fuerte que este curso te va a enseñar a emitir. La ayuda se repliega, en voz alta: este es el pico de dificultad del módulo, y te camino el flujo del pipe una vez en pantalla con cada paso explicado. Después armas el script tú mismo, incluida la lógica de airdrop-con-fallback y el switch de target. Verificar el delta de balance al final es totalmente en solitario. El próximo módulo te da checklists, no walkthroughs.

## La sonda que muta

Un comando primero. La transferencia de hoy aterriza en devnet, así que pregúntale a devnet directamente si está viva siquiera, con el mismo sobre que vienes POSTeando desde M7:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

`{"jsonrpc":"2.0","result":"ok","id":1}` significa que el target de todo lo que sigue está respondiendo. Ahora la teoría.

Tres lecciones de lecturas han demostrado que la estación puede preguntar. No demuestran nada sobre si este stack puede actuar. Una lectura puede tener éxito mientras el camino de escritura está completamente roto: bytes de clave equivocados, un mensaje malformado, un blockhash expirado, una red que acepta tu transacción y después se encoge de hombros. Las lecturas comparten exactamente dos superficies de falla con las escrituras, conectividad y salud del RPC, y esas ya las tienes cubiertas de seis formas. Todo lo que está por encima de esa línea, validez de la clave, construcción del mensaje, aceptación de la firma, inclusión, confirmación, es invisible para una lectura. Hay exactamente una forma de verlo: firmar algo real y mirarlo aterrizar.

![Un stack de siete capas donde las lecturas solo cubren alcanzabilidad y salud del RPC mientras una transferencia confirmada demuestra cada capa hasta la confirmación.](assets/v01-diagram.webp)

Ese es el marco del artefacto de hoy, y vale la pena decirlo sin adornos antes de cualquier código: `tx-check` es un chequeo de salud, no una demo. Las páginas de uptime del mundo entero muestran puntos verdes que quieren decir "el servidor respondió un ping", y vienes construyendo exactamente esa clase de sonda desde M2, a propósito, porque es el primer escalón correcto. Pero un cliente de blockchain que solo puede leer es una estación de monitoreo de un sistema que no puede tocar. La sonda que muta es una especie distinta: pone una afirmación firmada en el mundo y le pide a la red que se comprometa con ella. Cuando ese commitment vuelve, has demostrado tus claves, tu código de construcción de mensajes, tu firma, la disposición de la red a incluirte, y su maquinaria de confirmación, todo en una corrida de 30 segundos. Ninguna combinación de lecturas te da nada de eso.

Así que el plan es honesto y chico: consigue una clave desechable, consíguele algo de SOL de devnet sin valor, manda una fracción de vuelta para afuera, y exige una firma confirmada como el recibo. La transferencia misma es deliberadamente diminuta, 0.001 SOL de tu desechable a una segunda dirección fresca, porque el monto no es el punto. El recibo sí. Y el primer paso es pedirle plata a un faucet, que es por lo que la primera cosa honesta que hace esta lección es planear para el caso de que el faucet diga no.

### Mendiga antes de construir

Haz esto ahora, antes de cualquier teoría. En el repo de la estación, haz un workspace para el script e instala los dos paquetes que necesita:

```bash
mkdir tx-check && cd tx-check
npm init -y
npm i @solana/kit@^8 @solana-program/system@^0.14
npm i -D typescript tsx @types/node
```

Nota de versión: `@solana/kit` resuelve a 8.2.0 y `@solana-program/system` a 0.14.1 al 2026-09-02, y el rango de peer de system es `^8.0.0`, que es por lo que el dígito de kit es 8. Misma regla que m08-l2: fija lo que tus deps de `@solana-program/*` declaran peer, y vuelve a chequear los dígitos cuando instales, porque kit ya ha entregado dos majors en poco más de nueve semanas.

Ahora, a mendigar. La devnet de Solana es una red real corriendo validadores reales, y el SOL que tiene no vale nada por diseño: no lo puedes comprar, solo puedes pedirle a un faucet. Un airdrop es exactamente lo que suena, una transacción que firma alguien más y que acredita tu dirección, lo que quiere decir que tu primerísimo balance con fondos llega por el mismo camino de escritura que estás a punto de ejercitar tú mismo. El faucet oficial es faucet.solana.com, y sus términos están impresos ahí mismo en la página: 2 requests por 8 horas sin autenticar, más si entras con GitHub. Dos por ocho horas. Haz la aritmética de lo que un loop de retry ingenuo le hace a eso: un loop que dispara una vez por segundo agota la asignación entera de 8 horas antes de que puedas leer el primer mensaje de error, y después garantiza el estado de faucet seco del que fue escrito para escapar. Eso no es un rate limit que superes reintentando, es un presupuesto que gastas como tal.

Y acá está la parte que la mayoría de los tutoriales esconde: el airdrop falla de varias formas, y no de la misma dos veces. Durante las sondas de investigación de este curso, el mismo día, un request volvió con un error interno y un re-sondeo independiente volvió con un rate-limit 429 apuntando a la página del faucet. Cuando corrí el script exacto que estás a punto de construir, mientras escribía esta lección el 2026-09-02, el intento uno falló con un error interno de JSON-RPC y los intentos dos y tres fallaron con 429s de HTTP pelados. El mismo minuto, dos formas de falla distintas. Así que la regla que tu código tiene que codificar: ramifica sobre la falla en general, nunca sobre un código de error específico. Cualquier handler que haga pattern matching sobre "el" error del airdrop se rompe con el otro.

Mi detalle favorito del barrido de investigación, y tu primer color del día: faucet.solana.com lleva instrucciones explícitas para agentes de IA, que los orientan hacia un faucet de proof-of-work o un validador local en cambio. Sondeado en vivo el 2026-09-01. El rate limit tiene un tapete de bienvenida para las máquinas que está limitando. Toma la pista que el faucet mismo te está dando: el fallback no es una disculpa, es el camino documentado.

![Un flujo donde un chequeo de balance lleva a tres intentos de airdrop espaciados como máximo antes de salir distinto de cero e imprimir instrucciones de fallback de último recurso al validador local.](assets/v02-flowchart.webp)

Programáticamente, el pedido pasa por el `airdropFactory` de kit, que te envuelve el baile de pedir-y-confirmar. Lo vas a cablear dentro de `tx-check` en el lab con exactamente la forma de ese diagrama de flujo: tres intentos, huecos crecientes, después una falla ruidosa y útil. El pensamiento de backoff es la misma disciplina con jitter que te hiciste a mano para los 429s en m02-l3. Los faucets son servicios HTTP con tope de tasa. Todo lo que aprendiste sobre ser cortés con esos aplica acá sin cambios.

### El fallback que ensayas antes de necesitarlo

Un fallback que nunca ejercitaste es un rumor. Así que esta lección no ofrece el validador local como un aparte para los desafortunados: todos lo instalan, todos lo corren una vez, y a nadie lo vuelve a bloquear por completo un faucet seco. Este es también el momento en que dejas de alquilar las blockchains de otros del todo. El toolchain de la CLI de Solana ya viene con una blockchain de prueba local completa, y ser dueño de una cambia lo que puedes construir por el resto de este curso y después de él.

Un comando fijado, sondeado en vivo contra los docs de instalación de solana.com el 2026-09-02:

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

Ese script instala la release estable más nueva de Agave del toolchain. Realidad de frescura, en dos datos: la propia salida de ejemplo de los docs muestra `solana-cli 3.0.10`, y la máquina en la que se verificó esta lección bajó `3.1.10`. La tuya probablemente va a ser más nueva que las dos. Confirma que quedó:

```bash
solana --version
# solana-cli 3.1.10 (src:7bc9c805; feat:1620780344, client:Agave)
```

Después arranca tu propia blockchain:

```bash
solana-test-validator
```

Ese único comando arranca un cluster de Solana completo de un solo nodo en tu máquina: RPC en `http://127.0.0.1:8899`, websockets en `ws://127.0.0.1:8900`, un bloque de génesis acuñado hace segundos, y un faucet sin presupuesto porque el mint es tuyo. El nombre en esa cadena de versión, Agave, es el cliente validador en sí, el mismo software que corren los clusters públicos, que es por lo que tu blockchain local responde la interfaz JSON-RPC exacta que tus lecturas vienen golpeando desde m08-l1. Sigue corriendo en esa terminal hasta que le hagas Ctrl+C, con los slots haciendo tick en la esquina, y escribe su ledger a un directorio `test-ledger/` en la carpeta desde la que lo arrancaste. Borra ese directorio y el próximo arranque acuña un génesis fresco: una blockchain completamente nueva, cero historia, cada balance reseteado. Hay algo clarificador en eso. El intimidante objeto global que tu estación viene sondeando con cuidado por un módulo entero resulta ser software que puedes arrancar, parar y borrar como cualquier otro proceso. Honestamente, tener una blockchain entera en localhost es una bendición, y cuesta un comando.

Quédate un segundo con lo que ahora es tuyo, porque sobrevive a esta lección. Cada lectura que hace tu estación, cada script de este módulo, funciona contra esta blockchain cambiando un par de URLs. Cuando un experimento posterior necesite cincuenta cuentas con fondos, o mil transacciones en un loop apretado, o una prueba que tenga que empezar desde estado vacío, la devnet pública te pondría tope de tasa hasta la miseria y tu blockchain local no se va a enterar siquiera. Y para poner las expectativas con honestidad: ninguno de los cursos on-chain de la Academy te va a exigir este install, califican en entornos hospedados y en proceso a propósito. El validador local es tu herramienta de poder personal para experimentos demasiado ávidos para infraestructura compartida, no un prerrequisito que nada aguas abajo asuma.

El trade-off, porque siempre hay uno: tu blockchain local es una blockchain privada de uno. Los airdrops siempre aterrizan, las transacciones siempre confirman, nada está congestionado, y nada de eso demuestra nada sobre el camino de la red pública. El verde contra `solana-test-validator` demuestra tu código. El verde contra devnet demuestra tu código y la red entre tú y ella. Ten claro qué pregunta responde cada target, y haz que tu script pueda hacer las dos.

![Dos columnas que contrastan devnet como la prueba del camino de la red real contra el validador local como la prueba instantánea e ilimitada de corrección del código.](assets/v03-comparison.webp)

### Una clave que no tiene nada que perder

Toda escritura necesita un signer (un firmante), y este curso te da el mínimo honesto: una seed aleatoria fresca de 32 bytes, guardada en un archivo, cargada con el `createKeyPairSignerFromPrivateKeyBytes` de kit. Ese helper toma exactamente 32 bytes de material de clave privada y deriva la mitad pública él mismo, verificado contra kit 8.2.0. La clave vive en `.keys/devnet-throwaway.seed` adentro de la carpeta tx-check, y el nombre es la política de seguridad: es desechable, solo va a guardar SOL de devnet sin valor, y va al `.gitignore` antes de la primera corrida, no después.

Ese orden es la lección de verdad. Un archivo de par de claves al que le hiciste commit es una clave filtrada para siempre. La historia de git no olvida, los force-push no limpian de forma confiable, y el hábito de gitignorear material de clave antes de que exista vale más que cualquier clave suelta. Incluso una clave de devnet: el SOL no vale nada, pero el hábito se transfiere a claves que no lo son.

![Un hub que muestra una sola seed Ed25519 produciendo firmas idénticas ya sea que la clave viva en un archivo gitignoreado, una billetera de navegador, o un dispositivo de hardware.](assets/v04-diagram.webp)

Dos glosas antes de construir con ella, las dos estructurales. Primero, el signer. Los 32 bytes son una seed de clave privada Ed25519, y la matemática de la firma es idéntica ya sea que la clave viva en un archivo, un dispositivo de hardware, o una billetera de navegador. Dónde vive la clave cambia custodia y UX, nunca fuerza criptográfica, así que nada de este signer basado en archivo es un juguete: produce exactamente las firmas que verifican los validadores de mainnet. Segundo, los dos sustantivos en los que se apoya la próxima sección. Una instrucción es una unidad de trabajo dirigida a un programa: "system program, mueve N lamports de A a B". Un mensaje de transacción es el sobre: una o más instrucciones más los metadatos que la red necesita, quién paga la comisión y cuánto tiempo el sobre sigue siendo válido. Construyes el mensaje, firmas el mensaje, la red ejecuta las instrucciones que tiene adentro. Mantén esos dos niveles separados en tu cabeza y el pipe de abajo se lee solo.

Lo que deliberadamente no te toca acá es una billetera. Sin extensión de navegador, sin botón de conectar, sin popup de firma. Las billeteras manejan devnet bien, así que la exclusión no es técnica: es una frontera. Wallet-standard, la UX de conexión, y todo lo que involucra un prompt de firma pertenecen al curso de dominio del lado del cliente, en producción mientras escribo esto; espera la profundidad de billeteras y de aterrizaje de transacciones ahí. Un archivo lleno de bytes aleatorios es el signer que mantiene esta lección sobre la cosa que enseña: cómo un mensaje de transacción se construye, se firma y se confirma. El momento en que te descubres queriendo una billetera de verdad es el momento exacto en que empieza ese territorio, y hasta que el curso se entregue, wallet-standard es el término para buscar.

### El pipe, un paso honesto a la vez

Ahora el centro de la lección. Kit construye transacciones como un pipeline de transformaciones puras chicas sobre un valor de mensaje, y la forma canónica es una llamada a `pipe`. Acá está la cosa entera, exactamente como va a quedar en tu script, y después la desarmamos paso por paso. Cada identificador de acá está verificado contra los paquetes instalados: este código exacto compila bajo TypeScript estricto y aterrizó una transferencia confirmada mientras esta lección se escribía.

```typescript
const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const message = pipe(
  createTransactionMessage({ version: 0 }),
  (m) => setTransactionMessageFeePayerSigner(signer, m),
  (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
  (m) =>
    appendTransactionMessageInstruction(
      getTransferSolInstruction({
        source: signer,
        destination: recipient.address,
        amount: lamports(TRANSFER_LAMPORTS),
      }),
      m,
    ),
);

const signed = await signTransactionMessageWithSigners(message);
assertIsTransactionWithBlockhashLifetime(signed);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
await sendAndConfirm(signed, { commitment: "confirmed" });

const signature = getSignatureFromTransaction(signed);
```

**Paso 1: `createTransactionMessage({ version: 0 })`.** Un mensaje de transacción es la orden de trabajo sin firmar: quién paga, qué instrucciones corren, y por cuánto tiempo la orden es válida. Empieza vacío. El campo de versión elige el formato de mensaje moderno; la versión 0 es lo que produce el tooling actual, y eso es todo lo que necesitas saber acá.

**Paso 2: `setTransactionMessageFeePayerSigner(signer, m)`.** Toda transacción nombra una cuenta que paga la comisión, y esto pone la tuya. Un beat de atribución, porque te va a salvar de una falla clásica de copiar y pegar: kit trae dos setters de fee payer. El propio repositorio de kit, en su ejemplo de transferencia, usa la forma de dirección, `setTransactionMessageFeePayer`, que toma una dirección pelada. La documentación de kit en solanakit.com enseña la forma Signer que se usa arriba, que toma el objeto signer en sí para que el paso de firma posterior sepa exactamente quién tiene que firmar. Este curso enseña la forma Signer, con la autoridad de solanakit.com. Las dos formas son reales. Mezclar mitades de tutoriales que eligieron distinto es la forma clásica en que este flujo se rompe.

![Los setters de fee payer en forma de dirección y en forma Signer, lado a lado, con la mezcla de mitades de tutoriales señalada como el modo de falla.](assets/v05-comparison.webp)

**Paso 3: `setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m)`.** Este es el paso que tiene un reloj adentro. El mensaje queda estampado con un blockhash reciente, y la red solo acepta la transacción mientras ese blockhash siga siendo reciente, una ventana de 150 bloques, que al tiempo de slot de 300ms que mediste en m08-l1 es como 45 segundos. (Era un minuto allá cuando los slots apuntaban a 400ms, que es por lo que todavía te vas a topar con 'como un minuto' en escritos más viejos.) ¿Por qué una red te haría eso? Porque la alternativa es peor: sin una expiración, una transacción que no logró aterrizar podría quedarse en el vacío y ejecutarse horas después, después de que te rendiste y mandaste un reemplazo. El tiempo de vida es por lo que las transacciones sin aterrizar mueren limpio en vez de quedar rondándote. La regla práctica sale sola: trae el blockhash adentro del camino de envío, justo antes de construir el mensaje, nunca al arrancar el script. Un script que construye su mensaje al arranque, hace dos minutos de otro trabajo, después manda, va a fallar la confirmación todas y cada una de las veces, y ahora sabes por qué antes de que te pase.

![Una línea de tiempo que muestra una transacción firmada en segundos aterrizando a salvo mientras una mandada después de dos minutos llega pasada la expiración de blockhash de unos 45 segundos, 150 slots a 300ms cada uno, y muere.](assets/v06-timeline.webp)

**Paso 4: `appendTransactionMessageInstruction(getTransferSolInstruction({...}), m)`.** Una instrucción es una unidad de trabajo para un programa; un mensaje de transacción lleva una lista de ellas. El nuestro lleva exactamente una: una transferencia de SOL del system program, construida por `getTransferSolInstruction` de `@solana-program/system` con un signer de origen, una dirección de destino, y un monto. El helper `lamports()` marca el monto con el tipo correcto; un lamport, de m08-l1, es la unidad base, una milmillonésima de un SOL, y los montos son bigints porque u64 no cabe en un number de JavaScript.

**Paso 5: `signTransactionMessageWithSigners(message)`.** Como el fee payer entró como objeto signer, esta única llamada encuentra cada signer requerido pegado al mensaje y produce la transacción firmada. Cero malabares manuales con claves. La línea `assertIsTransactionWithBlockhashLifetime` que va después es una guarda a nivel de tipo que le dice al compilador lo que sabemos, que esta transacción lleva un tiempo de vida de blockhash, que es lo que exige el paso de confirmación.

**Paso 6: `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })`.** La factory toma tus conexiones de RPC una vez y devuelve una función de mandar-y-confirmar que puedes reusar. Fíjate que quiere las dos conexiones, y la segunda finalmente explica por qué este script abre un websocket siquiera: en vez de hacer polling de "¿ya aterrizó?" en un loop, la mitad de confirmación se suscribe a una notificación para tu firma y espera a que la red hable. Su contrato es el punto entero de esta lección: la función devuelta manda la transacción firmada y resuelve solo cuando la red la confirmó en el commitment que elegiste, o lanza. No "mandada". Confirmada. Pasamos `commitment: "confirmed"`, lo que quiere decir que un validador incluyó tu transacción en un bloque y una supermayoría del cluster votó sobre ese bloque. Hay niveles más superficiales y más profundos en esa perilla, y la historia completa del commitment, junto con qué te compra la finalidad de verdad, es territorio del lado del cliente más profundo de lo que un chequeo de salud necesita. Para un chequeo de salud, "el cluster votó sobre eso" es exactamente la vara correcta: lo bastante fuerte para significar algo, lo bastante rápido para correr a demanda.

Un beat más de honestidad sobre ese contrato, porque define tus códigos de salida. Resolver es demostración. Lanzar no siempre es la imagen espejo de la demostración: un lanzamiento puede querer decir que la transacción fue rechazada, o que expiró sin aterrizar, o simplemente que a tu websocket le dio un hipo mientras la red siguió adelante y la confirmó igual. Para tx-check esta asimetría está bien, un chequeo de salud debería ser paranoico, y una falsa alarma te cuesta una re-corrida. Para cualquier cosa que mueva valor real, "el envío lanzó, por lo tanto no pasó" es un bug con muertos encima, y la disciplina de retry-y-dedup que lo maneja como se debe es parte de la ciencia de aterrizaje que este curso traspasa. Ten presente que la costura existe; no la cruces hoy.

![Un envío resuelto demuestra la confirmación mientras un lanzamiento se abre en rechazo, expiración, o un mero hipo de websocket, que es por lo que un chequeo de salud puede levantar falsas alarmas.](assets/v07-diagram.webp)

El pipe son seis llamadas, y cada una carga un por qué. Ese es el patrón entero que este curso enseña para escrituras, y es deliberadamente el piso: signer crudo de par de claves, comisiones por defecto, cero sofisticación de retry. En un día congestionado una transacción con comisión por defecto simplemente puede no aterrizar, y esta lección acepta eso como un resultado enseñable en vez de contrabandear medio curso de aterrizaje.

Lo que nos lleva al cuadro del 20%, como prosa de traspaso en vez de links, porque estas costuras son del tamaño de un curso, no del tamaño de un párrafo. La ciencia de aterrizaje de transacciones, las comisiones de prioridad, la estrategia de retry-y-blockhash, y todo lo que tenga forma de billetera son un oficio propio del lado del cliente: el momento en que necesitas una garantía de aterrizaje o una billetera de navegador, esa es la costura que hay que ir a estudiar como se debe, como un curso de estudio, no un post de blog. Las transferencias de tokens, esta movió SOL nativo solamente, pertenecen al curso de activos digitales, que recorre los programas de token como se debe. Esta lección te da la escritura mínima honesta; el estudio más profundo te enseña a hacerla de grado de producción.

### Confirma, mira, listo

Un beat, como se prometió, porque el walkthrough del explorer se recortó para pagar tu validador local. La firma que devuelve `getSignatureFromTransaction` es una cadena base58 que nombra tu transacción de forma única para siempre. Pégala en cualquier explorer de Solana con el cluster puesto en devnet, o simplemente imprime el link que construye el script: `https://explorer.solana.com/tx/<signature>?cluster=devnet`. Vas a ver la transferencia, la comisión, el slot en el que aterrizó, y los dos balances cambiando. Esa página es el recibo público de un tercero de que tu stack puede actuar. Míralo una vez, disfrútalo, cierra la pestaña.

## Lab: tx-check

El marco, antes de los pasos: `tx-check` no es una demo, es una feature de la estación. Es el chequeo de salud del camino de escritura, que se corre a demanda, y que responde la única pregunta que ninguna lectura puede: ¿puede este stack firmar y aterrizar, no solo leer? Imprime una firma confirmada y sale 0, o falla a gritos y sale distinto de cero, lo que lo hace componible: el script de demo de m10-l1 lo va a llamar por exactamente ese contrato. Viste el pipe caminado una vez arriba. Ahora armas el script alrededor de él tú mismo.

1. **Gitignore primero.** En la raíz del repo de la estación, antes de que exista cualquier clave:

```bash
echo "tx-check/.keys/" >> .gitignore
echo "test-ledger/" >> .gitignore
git add .gitignore && git commit -m "chore: ignore devnet throwaway keys and local ledger"
```

(La línea del ledger está deliberadamente sin anclar: `solana-test-validator` escribe `test-ledger/` en cualquier directorio desde el que lo arranques, y el paso 7 solo dice "una segunda terminal", así que un patrón sin anclar cubre la raíz del repo, `tx-check/`, y cualquier otro lado desde donde lo lances. Un patrón fijado a un directorio dejaría la demostración con `git status` del Checkpoint llena de ruido la primera vez que arrancaras el validador una carpeta más allá.)

2. **Haz el scaffold del script.** Crea `tx-check/tx-check.ts` con las constantes y los imports. El switch de target son dos variables de entorno con los valores por defecto de devnet; esta es la interfaz, así que mantén los nombres exactos:

```typescript
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { randomBytes } from "node:crypto";
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromPrivateKeyBytes,
  generateKeyPairSigner,
  airdropFactory,
  lamports,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
  getSignatureFromTransaction,
  assertIsTransactionWithBlockhashLifetime,
  devnet,
  type KeyPairSigner,
} from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";

const RPC_URL = process.env.RPC_URL ?? "https://api.devnet.solana.com";
const WS_URL = process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com";
const SEED_PATH = ".keys/devnet-throwaway.seed";
const TRANSFER_LAMPORTS = 1_000_000n; // 0.001 SOL
const MIN_BALANCE = 5_000_000n; // transfer plus fees, with slack

const rpc = createSolanaRpc(devnet(RPC_URL));
const rpcSubscriptions = createSolanaRpcSubscriptions(devnet(WS_URL));
```

   Una arruga que vale su comentario: `devnet()` es un brand a nivel de tipo. No cuesta nada en tiempo de ejecución y le dice al compilador que este endpoint soporta airdrops, que es lo que exigen los tipos de `airdropFactory`. Tu validador local honra el mismo contrato de airdrop, así que el brand también se sostiene para `http://127.0.0.1:8899`.

3. **Escribe `loadOrCreateSigner`.** Contrato: si `SEED_PATH` existe, lee sus 32 bytes y devuelve `createKeyPairSignerFromPrivateKeyBytes(seed)`. Si no, hazle `mkdirSync` a la carpeta `.keys`, escribe `randomBytes(32)` al archivo, loguea que se creó una clave desechable nueva, después cárgala de la misma forma. La función devuelve `Promise<KeyPairSigner>`. La recarga determinista importa: el mismo archivo de seed tiene que producir la misma dirección en cada corrida, o tu balance de devnet queda varado en una dirección para la que ya no puedes firmar.

4. **Escribe `ensureBalance(signer)`.** Contrato: lee el balance con la misma línea `rpc.getBalance(signer.address).send()` que trae el solana-panel, y retorna temprano si supera `MIN_BALANCE`. Si no, construye `airdropFactory({ rpc, rpcSubscriptions })` e intenta un airdrop de `lamports(1_000_000_000n)`, un SOL de devnet, como máximo tres veces con demoras de 0, 2 y 8 segundos: la forma de backoff de m02-l3, dimensionada para un faucet cuyo presupuesto es 2 por 8 horas. Captura las fallas en general y loguea el mensaje; no hagas match contra ningún código específico, ya sabes por qué. Después de la tercera falla, lanza un error rico que le diga al operador exactamente qué hacer a continuación: arranca `solana-test-validator` y vuelve a correr con `RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900`.

5. **Arma `main`.** Loguea la URL del target, carga el signer, asegura el balance, después un `generateKeyPairSigner()` fresco como el destinatario para que la transferencia mueva valor visiblemente a una segunda dirección, después el pipe exactamente como se caminó, después imprime `confirmed: <signature>` y el link del explorer cuando el target es devnet. Cierra el archivo con el contrato de código de salida:

```typescript
main().catch((err) => {
  console.error("tx-check FAILED:", (err as Error).message);
  process.exit(1);
});
```

![Cuatro bloques anotados que muestran el cargador del signer, el airdrop presupuestado con fallback, el pipe de seis pasos en main, y el contrato de código de salida del que dependen las lecciones aguas abajo.](assets/v08-annotated-code.webp)

6. **Primera corrida, contra devnet.** `npx tsx tx-check.ts`. Dos cosas pueden pasar, y las dos son contenido de la lección. Si el faucet coopera te llega el pago de inmediato: la línea de confirmado, el link del explorer, salida 0. Si está seco te toca lo que me tocó a mí el 2026-09-02, pegado textualmente de la corrida en que se verificó este script exacto:

```text
target: https://api.devnet.solana.com
new throwaway seed written to .keys/devnet-throwaway.seed (gitignored)
balance: 0 lamports at 77Stnr644XdriXqnvnt6ZF1TeSBv2PneRS4QoTJ788vX
airdrop attempt 1 failed: JSON-RPC error: Internal JSON-RPC error (Internal error)
airdrop attempt 2 failed: HTTP error (429): Too Many Requests
airdrop attempt 3 failed: HTTP error (429): Too Many Requests
tx-check FAILED: airdrop failed after 3 attempts. Faucet may be dry (budget: 2/8h unauthenticated). Fallback: start solana-test-validator, then re-run with RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900
```

   Lee ese log como un operador. El intento uno y el intento dos fallaron distinto, adentro del mismo minuto, que es la afirmación de falla heterogénea que está en la caja de honestidad, observada en vivo. El script no se cayó, no hizo loop, no hizo match contra ninguno de los dos códigos: gastó sus tres intentos presupuestados, imprimió instrucciones que un humano a las 2am podría seguir, y salió 1. Ese no es un paso de lab fallido. Ese es tu script manejando una dependencia best-effort y presupuestada exactamente como se diseñó, y es el sistema de producción más chico que hayas escrito.

7. **La corrida de fallback, obligatoria para todos.** Incluso si devnet funcionó al primer intento. En una segunda terminal, `solana-test-validator`, espera unos segundos a que arranque, después:

```bash
RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900 npx tsx tx-check.ts
```

   Forma esperada, de la corrida en que se verificó este código exacto:

```text
target: http://127.0.0.1:8899
balance: 0 lamports at 77Stnr644XdriXqnvnt6ZF1TeSBv2PneRS4QoTJ788vX
airdrop landed
confirmed: 5Qe3VztdDPtXVcyzM7egpwMkgVL3itWfCkFo7Pt4AndgTDEfXAr9tM83nNzU6J4F3dsJiPFv5ZhuKuMLP8Do2mxN
```

   Tu dirección y tu firma van a ser distintas, la forma no. El airdrop aterriza al instante porque el faucet es tuyo. Y ahora el fallback no es un rumor: lo ejercitaste, la misma lógica que m09-l2 va a aplicar a las alarmas. Un faucet seco no puede volver a bloquearte por completo, y como efecto secundario ahora eres dueño de una blockchain local completa para todo lo que construyas después.

8. **El otro target.** De la red de la que todavía no conseguiste una confirmación, corre contra ella ahora, sin cambiar nada más que las dos URLs. Una de las dos todavía puede rechazarte, si el faucet se quedó seco adentro de su ventana de 8 horas; la barrera de abajo lo contempla con honestidad. El punto de este paso es el switch en sí: un script, dos blockchains, cero cambios de código.

![Una escalera de sondas de la estación desde chequeos de HTTP básicos hasta tx-check como la sonda más profunda, con el script de demo del módulo diez consumiendo su código de salida.](assets/v09-diagram.webp)

## Challenge

En solitario, y compone todo lo que este módulo construyó. Ahora mismo tx-check demuestra que una transacción se confirmó. Haz que demuestre que la transferencia quiso decir lo que dijo, semánticamente. Lee los dos balances antes de la transferencia y después de ella, emisor y destinatario, usando la misma lectura de `getBalance` que m08-l2 te enseñó. Después afirma dos cosas: que el balance del destinatario subió exactamente en `TRANSFER_LAMPORTS`, y que el balance del emisor bajó en `TRANSFER_LAMPORTS` más una comisión mayor que cero. Imprime la comisión que tu transacción pagó de verdad, en lamports, y formatea los montos de SOL con tu formateador de BigInt de m08-l2, nunca parseFloat. No hardcodees una comisión esperada: mídela, imprímela, y deja que la aserción solo exija que exista. Vas a notar que el emisor perdió un poco más de lo que mandó. Las lecturas eran gratis porque no mutan nada; la comisión es el otro lado de esa asimetría, el precio de la escritura, pagado por el fee payer que pusiste en el paso 2 del pipe. Por qué las comisiones son lo que son, y cómo pujar por aterrizar cuando importa, es oficio de aterrizaje de transacciones, y ese territorio le toca al curso de dominio del lado del cliente una vez que se entregue.

Dos notas de implementación, los dos lugares donde espero que los primeros intentos se tambaleen. El orden importa: toma los snapshots de antes después de que `ensureBalance` retorne, o el crédito del airdrop va a contaminar el delta de tu emisor, y toma los snapshots de después solo una vez que `sendAndConfirm` haya resuelto, ya que con commitment `confirmed` las dos lecturas van a reflejar la transferencia. Y la aritmética es bigint de punta a punta: los deltas, la comisión, la comparación contra `TRANSFER_LAMPORTS`, todo eso en matemática con sufijo `n`, exactamente la disciplina que el formateador de m08-l2 se construyó para proteger. Aceptación: el script sigue saliendo 0 en una transferencia confirmada y ahora verificada semánticamente, sale 1 si los deltas alguna vez no concuerdan con el monto, e imprime la comisión medida cruda y formateada.

## Checkpoint

Barrera sobre hacer, un pegado de terminal: la línea `confirmed: <signature>`, de devnet si el faucet te dejó entrar en esta ventana, de tu validador local si no, más el link del explorer si fue devnet. Después la evidencia de que el switch funciona: el mismo script pasando contra el otro target con solo las dos URLs cambiadas, admitiendo que un faucet seco puede dejar la corrida de devnet pendiente hasta que tu ventana de 8 horas se resetee. Y la demostración negativa: `git status` sin mostrar ningún archivo de clave cerca de staging, porque `.keys/` se ignoró antes de que la seed existiera. La corrida del validador local no es evidencia opcional; todos tienen uno a esta altura.

Una victoria de 30 segundos que tomó ocho módulos ganarse, así que tómate el beat. Cada módulo de este curso está parado adentro de esa firma: el TypeScript y el tooling de M1 a M3, la disciplina de backoff de M2 envuelta alrededor de un faucet, el hilo de ops que entregó las superficies que ahora leen la blockchain, y un mensaje firmado que una red real accedió a ejecutar. Sesenta y cuatro bytes de demostración de que tu stack puede actuar.

Si los modos de falla del faucet te sorprendieron de alguna cuarta forma nueva que esta lección no listó, esa es señal genuinamente útil: deja el texto exacto del error en el feedback del curso. La caja de honestidad del airdrop está construida a partir de fallas observadas, y crece de la misma forma en que creció tu ensayo del fallback, porque alguien se dio contra la pared primero y lo escribió.

La estación ahora lee desde cada superficie y ha demostrado que puede escribir. Eso la vuelve un sistema real, lo que quiere decir que puede fallar de verdad. El próximo módulo empiezas a correrla como alguien a quien ya le sonó el pager: auditando el árbol de dependencias sobre el que se para, después cableando logs y alarmas para que hasta la propia muerte del monitor sea ruidosa.
