# Gasless con Kora: nadie trae SOL a una feria de discos

La lección pasada le pusiste gate a la API de precios de prensado con `pay`, un endpoint contestando tanto x402 como MPP, y viste a tu agente pagador liquidar derecho a través de él. Cargando esa liquidación, como lo hace desde que construiste el agente, estaba el facilitador: un servicio que liquida el pago de un agente y, calladamente, paga la comisión de la transacción mientras lo hace. El modo pull de MPP, el otro protocolo detrás de ese gate, sienta a un co-firmante de servidor en exactamente el mismo lugar, que es el hilo que la lección pasada te dijo que sostuvieras. Ese asiento de fee payer está a punto de importar mucho más de lo que importaba.

Imagínate el puesto de Wavelength en una feria de discos de fin de semana. Se acerca un coleccionista, quiere el prensado limitado de agosto, y tiene 30 USDC sentados en su billetera. También tiene exactamente cero SOL, porque compró stablecoins en un exchange y nadie le dijo que existiera un token de gas. En el checkout que construiste hasta ahora, su transacción no puede ni pagar su propia comisión de 5000 lamports. Se va. Ves morir una venta por medio centavo de un token que el comprador no tenía ninguna razón para tener.

El arreglo no es "haz que el comprador consiga SOL". El fee payer no tiene que ser el comprador. Nunca tuvo que serlo. Monta el scaffold del workspace ahora para que la instalación corra mientras lees, y acuña a la estrella de esta lección de paso: una billetera de comprador que nunca va a tener un solo lamport.

```bash
cd ~/wavelength   # the workspace root; gasless-checkout must sit beside transfer-kit and checkout-txreq or the ../../ imports below cannot resolve
mkdir -p gasless-checkout/src gasless-checkout/verify
cd gasless-checkout
npm init -y
npm pkg set type=module   # the smoke script uses top-level await and import.meta
npm install @solana/kit@6.10.0 @solana-program/token@0.14.0 @solana/kora@0.2.1 express@5 \
  @solana/kit-plugin-instruction-plan@^0.6.0 @solana/kit-plugin-payer@^0.6.0 \
  @solana/kit-plugin-rpc@^0.6.0 @solana-program/compute-budget@0.16.0 --legacy-peer-deps
npm install -D tsx@4 typescript @types/express @types/node --legacy-peer-deps
solana-keygen new --no-bip39-passphrase -o buyer.json
```

Las dos líneas de install llevan `--legacy-peer-deps`, y la segunda no es un descuido de copiar y pegar. npm revalida el árbol de dependencias entero en cada install, no solo los paquetes que nombraste, así que el conflicto de abajo se reevalúa cuando agregas cuatro herramientas de dev que no tienen nada que ver con él. Saca la bandera de la segunda línea y sale con `ERESOLVE` antes de instalar nada.

Notas de pins, comprobadas el 2026-08-31. `@solana/kora` 0.2.1 es el `latest` de npm, publicado el 2026-03-27 (las betas más nuevas de 0.3.0 viven solo en el tag `beta`). Peerea `@solana/kit` ^6.1.0, que nuestro 6.10.0 satisface limpiamente, y este workspace se queda en la línea v6 del kit como cada peldaño de checkout antes de él, así que no vayas a buscar aquí el cliente de subscriptions del kit ^7. La arruga es que dos de los otros rangos de peers del SDK son más viejos que lo que este curso fija: pide `@solana-program/token` ^0.12.0 contra nuestro 0.14.0, y `@solana-program/compute-budget` ^0.13.0 contra nuestro 0.16.0. El mensaje de error de npm nombra al que golpea primero, normalmente compute-budget, así que no te sorprendas cuando el texto difiera de este párrafo. Los nombres que Kora importa de verdad de los dos paquetes existen sin cambios en nuestros pins, que es por lo que `--legacy-peer-deps` es seguro en este workspace específico y no un hábito para conservar. `express` 5 y `tsx` 4 son los mismos majors que el servidor de transaction-request ya corre. `@solana-program/compute-budget` está fijado en 0.16.0, el último minor cuyo rango de peers acepta un kit v6 (0.17.0 saltó su peer a ^7) — el mismo dígito que la lección de fee-recipe fija más adelante en este módulo, así que el repo dice un solo número de compute-budget en todas partes. `--legacy-peer-deps` también apaga la instalación automática de peers de npm, que es por lo que los cuatro paquetes de plugin están nombrados explícitamente.

Un archivo más antes de la teoría, porque un checkpoint más adelante en el lab se apoya en él. Este workspace alcanza hacia atrás a `transfer-kit` y `checkout-txreq` por caminos relativos, y el `tsconfig.json` de la raíz del módulo 2 solo incluyó siempre `transfer-kit/src`, así que un `tsc` corrido aquí no haría la comprobación de tipos de ninguno de los archivos que estás a punto de escribir. Dale al workspace su propia config:

```bash
cat > tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "types": ["node"]
  },
  "include": ["src", "verify", "../transfer-kit/src", "../checkout-txreq/src"]
}
JSON
```

`moduleResolution: "bundler"` es el ajuste honesto aquí en lugar del `NodeNext` de la raíz: cada import entre paquetes de este curso está escrito sin extensión de archivo, que es lo que `tsx` resuelve y lo que `NodeNext` rechaza. La lista `include` es el punto — nombra los dos paquetes hermanos que este workspace importa, así que un camino relativo obsoleto o una firma de función a la deriva de allá falla aquí en vez de esconderse hasta el runtime.

## Resumen

El índice de hallazgos, cada línea accionable:

- El fee payer es un asiento, no una identidad. El módulo 1 preguntó quién paga la comisión y la respuesta era el comprador. La lección pasada un facilitador la pagó por un agente. Hoy un paymaster la paga por un humano, y la billetera del comprador necesita cero SOL, nunca.
- Octane, la respuesta vieja a este problema, fue archivado el 2026-04-20 y su README ahora apunta a su sucesor. Kora es ese sucesor: un nodo paymaster JSON-RPC 2.0 de la Solana Foundation, que lleva una auditoría de Runtime Verification (reporte 20251119), con el pago de comisiones en tokens SPL incorporado.
- Entregas **gasless-checkout**: un nodo de Kora corrido localmente con una config bien cerrada, un cliente de patrocinador con un tope de comisión, y un constructor patrocinado que reusa `finalizeTransaction` de checkout-txreq con un input cambiado: el fee payer ahora es el firmante de Kora, no el comprador.
- La transacción lleva dos firmas: Kora firma como fee payer, el comprador firma solo la transferencia. La comisión base es de 5000 lamports por firma, así que tu patrocinador paga 10,000 lamports por checkout, más cualquier priority fee que agregues.
- El costo más grande se esconde en otra parte: si el patrocinio crea una cuenta de token, ese es el mínimo exento de rent de 165 bytes — pregúntale a `getMinimumBalanceForRentExemption(165)`, que contestó 1,488,440 lamports en devnet el 2026-09-07 y está cayendo mientras SIMD-0437 baja la tasa por escalones — y el comprador puede reclamarlo después cerrando la cuenta. Presupuéstalo como gasto, nunca como un préstamo.
- Un paymaster sin reglas de validación es un drenaje abierto. Las allowlists de Kora y su política de fee payer, que por defecto niega todo cuando se omite, son la historia de seguridad entera: vas a configurar las allowlists y vas a dejar a propósito la política de fee payer en su negar-por-defecto.
- Verifica: `npx tsx verify/gasless.smoke.ts` imprime `sponsored=true`, un fee payer igual al firmante de Kora y no al comprador, un delta de lamports del comprador de exactamente 0, y una firma del comprador.

La división del trabajo, dicha sin vueltas: este es el módulo 8, territorio en solitario. La config del nodo y el constructor patrocinado están trabajados porque el cableado de Kora es la última superficie de integración nueva de este curso. El manejo de la cotización de comisión y el ensamblaje de doble firma son TODOs de completion contra criterios de aceptación, no recorridos. La regla de validación que demuestra que tu paymaster rechaza transacciones ajenas es enteramente tuya en el Challenge.

![Desglose de costos apilado de un checkout patrocinado: una comisión base de 10,000 lamports por dos firmas, una priority fee opcional, y un depósito de rent de cuenta de token condicional mucho más grande.](assets/v01-chart.png)

## El asiento de fee payer

### Tercer inquilino, mismo asiento

Cada transacción de Solana nombra una cuenta como su fee payer: la primera cuenta del mensaje, la que tiene una firma que hace también de id de la transacción, la que el runtime debita por la comisión base. Nada en el protocolo dice que esa cuenta tenga que beneficiarse de la transacción. Solo tiene que firmar y tener lamports.

El curso ha estado rondando este asiento desde el módulo 1. En ese entonces la pregunta "quién paga la comisión" tenía una respuesta aburrida: el comprador, porque la billetera del comprador construyó la transacción y se puso primera. La lección pasada la respuesta se volvió interesante: el facilitador de x402 liquidó el pago del agente y se sentó él mismo en el asiento de fee payer, que es precisamente por lo que tu agente necesitaba un saldo de USDC y ningún SOL. Hoy el patrón recibe un nombre y una herramienta de producción. Un paymaster (algunos ecosistemas dicen relayer) es un servicio cuyo trabajo entero es ocupar el asiento de fee payer en las transacciones de otras personas, bajo reglas que su operador controla. El mismo asiento las tres veces. Lo único que cambió es quién se sienta.

Una advertencia desde la costura entre la lección pasada y esta, ya que las dos herramientas son vecinas naturales: un facilitador de x402 puede correr él mismo sobre Kora, y la guía oficial de ese emparejamiento todavía importa nombres de paquete estilo v1, `x402` y `x402-express`. El SDK vivo es la línea `@x402/*` v2 con scope contra la que construiste la lección pasada. Construye v2, y lee los nombres pelados de la guía como deriva de docs, no como una instrucción.

![Un asiento de fee payer con tres inquilinos por turno: el comprador en el módulo 1, el facilitador de x402 en el módulo 7, y el paymaster de Kora del comercio aquí.](assets/v02-diagram.png)

### Octane murió y nombró a su sucesor

Por años la respuesta estándar a gasless en Solana fue Octane, un relayer comunitario que co-firmaba transacciones a cambio de tokens SPL. Si buscas "gasless Solana" hoy todavía vas a encontrar tutoriales construidos sobre él. No los sigas. Octane fue archivado el 2026-04-20, y su README ahora dice "Check out Kora". Un paymaster que murió y apuntó a su propio sucesor es casi la señal de deprecación más limpia que este ecosistema produce; capta la indirecta al pie de la letra.

Kora es ese sucesor: un nodo paymaster de la Solana Foundation, escrito en Rust, que habla JSON-RPC 2.0 sobre HTTP pelado. Lo corres tú (o alquilas uno hosteado), le das un firmante, y le expone un conjunto chico de métodos a tu backend: `getConfig`, `getPayerSigner`, `getSupportedTokens`, `estimateTransactionFee`, `signTransaction`, `signAndSendTransaction`, y unos cuantos parientes. Dos detalles importan para un comercio que decide si ponerlo o no en el camino del dinero. Primero, puede ponerle precio a las comisiones en tokens SPL, así que un comprador que solo tiene USDC puede pagar su propia comisión en USDC si configuras ese modo; hoy corremos el modo más simple, donde el comercio simplemente se come la comisión. Segundo, lleva una auditoría de Runtime Verification (reporte 20251119), que es el dato de credibilidad que separa "un firmante detrás de un puerto abierto" de infraestructura que puedes defender poner en el asiento de fee payer de cada venta que haces.

El lado de TypeScript es un solo paquete, `@solana/kora`, que ya instalaste: un cliente tipado y delgado donde cada método es una sola llamada JSON-RPC. Sin magia, y vas a leer las respuestas tú mismo en el lab.

![Línea de tiempo desde la era de Octane pasando por la auditoría de Kora de 2025 y los releases de principios de 2026 de kora-cli 2.0.5 y el cliente @solana/kora 0.2.1, terminando en el archivado de Octane el 2026-04-20.](assets/v03-timeline.png)

### El viaje de ida y vuelta de doble firma

Aquí está el corazón mecánico de la lección, y es más chico de lo que suena. Una transacción de Solana es un mensaje más una firma por cada firmante requerido, y cada firmante firma los mismos bytes. Así que un checkout patrocinado es apenas una transacción con dos firmantes requeridos en vez de uno: el paymaster, listado primero como fee payer, y el comprador, requerido porque es la autoridad sobre la transferencia de USDC. Ninguna de las dos firmas es especial más allá de la posición. El ensamblaje es una carrera de relevos:

1. Tu servidor construye la transacción exactamente como checkout-txreq siempre lo hizo, con un input cambiado: `feePayer` es la dirección del firmante de Kora (le preguntas al nodo por `getPayerSigner`), no el comprador.
2. El servidor manda la transacción base64 sin firmar al `signTransaction` de Kora. Kora la simula, la corre contra sus reglas de validación y, si pasa, devuelve la misma transacción con la firma del fee payer rellenada. El slot del comprador sigue vacío. Por eso el método toma `sig_verify: false` por defecto: tiene que poder firmar una transacción que todavía no está firmada por completo.
3. La billetera recibe esa transacción parcialmente firmada, le muestra al comprador qué hace, junta la firma del comprador, y la manda. Esto no es un truco de protocolo atornillado al costado: la spec de transaction-request que implementaste en el módulo 3 permite explícitamente que el servidor devuelva una transacción parcialmente firmada, y este es el caso para el que existe esa cláusula.

¿Por qué `signTransaction` y no `signAndSendTransaction`, cuando el nodo ofrece los dos? Por quién falta. En el momento en que Kora ve la transacción, el comprador todavía no firmó, así que el nodo no puede mandarla; solo puede aportar su firma y devolver los bytes. Sign-and-send existe para el flujo inverso, donde el cliente ya tiene una transacción firmada por completo por el comprador y quiere que el paymaster co-firme y la retransmita en un solo salto. Para un checkout, solo-firmar es también el valor por defecto más seguro: el envío se queda con la billetera, lo que quiere decir que el rechazo del comprador es un veto de verdad, no una carrera contra un relay. La config del lab desactiva la variante de send de plano, un verbo menos que un atacante puede sondear.

El orden importa en una sola dirección: Kora firma antes que el comprador porque la firma de Kora se computa sobre el mensaje, y el mensaje tiene que estar final (fee payer, instrucciones, blockhash) antes de que firme nadie. Después de eso, las firmas se pueden agregar en cualquier orden; no se cubren entre ellas. Y el blockhash de adentro pone tu reloj en hora: alrededor de 150 bloques de validez, unos 45 segundos al slot time objetivo actual de 300ms (la etapa de 300ms de SIMD-0525 entró en vigor en la epoch 1024, el 2026-08-28), que alcanza y sobra para construir, co-firmar y un toque en un teléfono, y es exactamente por lo que construyes la transacción por petición en vez de pre-firmar una pila de ellas.

![Flujo de checkout patrocinado: el servidor construye una transacción sin firmar con Kora como fee payer, Kora firma primero, después la billetera del comprador firma y la manda, y al patrocinador se le debita.](assets/v04-flowchart.png)

### Cuando el comprador paga la comisión en USDC

Antes de la aritmética del comercio-se-lo-come, conoce el modo que esta lección a propósito no construye, porque te lo vas a encontrar en el mundo real y el capstone puede quererlo. Kora puede cobrarle al comprador la comisión en un token SPL en vez de absorberla. El flujo agrega una instrucción: tu servidor llama a `estimateTransactionFee` con un `fee_token`, recibe de vuelta tanto `fee_in_lamports` como `fee_in_token` (el mismo costo, denominado en las unidades base del mint mismo), después le pide a `getPaymentInstruction` una transferencia chica de ese token del comprador a la dirección de pago del nodo, la agrega a la transacción, y procede exactamente como antes. El comprador sigue teniendo cero SOL; nada más paga unos centavos de USDC por el viaje, y tu float de patrocinador se vuelve una cuenta de capital de trabajo que recicla en vez de un subsidio que se drena.

Dos perillas de config gobiernan el precio de ese viaje. `price_source` nombra el oráculo que convierte lamports a tokens, y el modelo de `[validation.price]` fija el margen: `free` (el modo de hoy, el comprador no paga nada), `margin` (costo más un porcentaje), o `fixed` (un monto plano por transacción, en un token que nombras tú). El lab fija `price_source = "Mock"` por una razón que vale la pena recordar: los mints de devnet no tienen mercado vivo, así que una fuente de oráculo de verdad cotizaría un disparate; producción lo cambia a precios de Jupiter y por lo demás la misma config sigue en pie. Cobrarle-al-comprador es el modo que convierte un paymaster de un costo de marketing en una característica de pagos, y cada línea del build de hoy es reusable bajo ese modo: solo cambian el modelo de precio y una instrucción agregada.

### Lo que cuesta el patrocinio, honestamente

Ahora la parte por la que va a preguntar tu contador, porque gasless no es gratis, es prepagado por ti.

La línea visible es chica. La comisión base es de 5000 lamports por firma, y el checkout de doble firma lleva dos, así que cada venta patrocinada le cuesta a tu float 10,000 lamports más cualquier priority fee que agregues. Una precisión que te va a ahorrar un diagnóstico equivocado más adelante: esa comisión base de 5000 lamports es fija y no se mueve con la congestión. Cuando la red está ocupada, lo que sube es la priority fee opcional que eliges agregar, nunca la base. Si la billetera de tu patrocinador se drena más rápido de lo que predice tu aritmética de comisiones, la comisión base no es la sospechosa. A esta tasa un airdrop de 2 SOL financia decenas de miles de checkouts, y si la historia terminara ahí, el patrocinio sería un error de redondeo.

No termina ahí. El evento caro es la cuenta de token asociada. Si tu flujo patrocinado alguna vez crea una ATA para el comprador (su primera cuenta de USDC, un mint nuevo, un token de lealtad), el depósito de rent es el mínimo exento de rent para la cuenta de 165 bytes, y lo lees de `getMinimumBalanceForRentExemption(165)` y no de ninguna página, incluida esta — en devnet el 2026-09-07 eso era 1,488,440 lamports, unas 150 veces la comisión de la transacción de doble firma entera. Vuelve a leerlo antes de presupuestar, porque SIMD-0437 está bajando la tasa por byte por escalones y los dos clusters ya se movieron; la proporción es lo durable aquí, no los lamports. Y aquí está la advertencia que el brief de cada despliegue de paymaster debería llevar en negrita: ese rent no se fue, está sentado en una cuenta que el comprador tiene. El comprador puede cerrar esa cuenta de token cuando le guste y quedarse con el rent reclamado. No hay mecanismo para devolvértelo. Así que trata el rent patrocinado como gasto, con su precio puesto en la venta como las comisiones de procesamiento de tarjeta, y nunca lo anotes en los libros como un préstamo recuperable. Corre la servilleta tú mismo para una feria de cien compradores: cien checkouts de doble firma son 0.001 SOL de comisiones, y cien ATAs de comprador primerizo son cien veces lo que sea que ese curl te acaba de decir — dos órdenes de magnitud de diferencia a cualquier tasa que la red haya cobrado. La línea del rent es el presupuesto; la línea de la comisión es ruido.

![Gráfico de barras en escala logarítmica que compara una comisión base de doble firma de 10,000 lamports contra alrededor de 1.5 millones de lamports de rent de creación de ATA, unas 150 veces más grande y reclamable solo por el comprador.](assets/v05-chart.png)

La otra línea honesta: cuando el comprador ya tiene SOL, el patrocinio es costo puro. Pagas 10,000 lamports para ahorrarle a alguien medio centavo que podría haber pagado él mismo, y de paso ensanchas tu superficie de ataque. El despliegue maduro patrocina selectivamente (primera compra, flujos de onboarding, billeteras con cero SOL) y no por reflejo. La comparación de abajo es la decisión de un vistazo, y es la contrapartida de esta lección entera: gasless le quita al comprador el requisito de SOL, y pagas por eso dos veces, una en rent que deberías dar por perdido y una en una carga de validación que ahora es obligatoria.

![Tabla de comparación entre el checkout que paga el comprador y el patrocinado a través de requisitos de SOL, cantidad de firmas, comisiones, rent de la ATA, conversión, superficie de ataque, y cuándo gana cada modo.](assets/v06-comparison.png)

### La validación es el producto

¿Por qué tanta ceremonia con las reglas? Porque un paymaster es una máquina que firma las transacciones de otras personas con una llave fondeada. Quítale la validación y construiste un faucet. Cualquiera que pueda alcanzar el puerto puede mandar cualquier transacción que le guste con tu firmante en el asiento de fee payer: drenaje de comisiones como mínimo, y mucho peor si una instrucción puede gastar desde cuentas que tu firmante controla. La auditoría de arriba cubre el código de Kora. Nada audita tu config más que tú.

La config de Kora te da controles por capas, y el lab pone cada uno de ellos a propósito. `allowed_programs` es la pared exterior: Kora se niega a patrocinar cualquier transacción que invoque un programa fuera de la lista, y tu checkout necesita exactamente tres (el Token program para el TransferChecked, Memo para el sello del pedido, compute budget porque las billeteras lo agregan). `allowed_tokens` acota en qué mints va a cotizar comisiones. `max_signatures` y `max_allowed_lamports` le ponen tope al radio de impacto de cualquier transacción individual. Y después está `fee_payer_policy`, el sutil: interruptores de grano fino para si el fee payer mismo puede ser el origen de una transferencia de System, el dueño en una transferencia de token, una autoridad de nonce, y una docena de roles similares. Esa política es lo que detiene el drenaje más agudo de todos, una transacción cuya instrucción interna calladamente mueve lamports fuera de la cuenta del patrocinador que la está firmando.

La postura por defecto es la correcta: en el código fuente actual, cada interruptor de `fee_payer_policy` por defecto niega cuando el bloque se omite de la config. La config de ejemplo del repo escribe el bloque explícitamente — sus interruptores hoy dan la casualidad de estar en false, pero una copia del ejemplo está a una edición de upstream de meterte en poderes que nunca elegiste. En el lab omitimos el bloque a propósito y dejamos que el negar-por-defecto haga su trabajo.

![kora.toml del lab anotado: allowlists de programas y de tokens, topes de firmas y de lamports, transacciones durables apagadas, precios free, y un bloque de política de fee payer omitido para que cada poder por defecto niegue.](assets/v07-annotated-code.png)

Esa es la superficie de confianza, dicha sin alarmismo: estás corriendo (o alquilando) un servicio que tiene una llave fondeada y firma lo que le manden desconocidos, y las reglas de validación son la diferencia entera entre un paymaster y una donación. Sobrio, no aterrador. Configúralo como si lo dijeras en serio y los modos de falla de arriba se quedan teóricos.

La decisión de correr-o-alquilar en sí es aritmética ordinaria de infraestructura, y vale treinta segundos ahora porque le da forma a lo que despliegas después de esta lección. Correr tu propio nodo, el camino de hoy, quiere decir que tienes la llave del patrocinador, dimensionas el float, miras su saldo (Kora trae un endpoint de métricas exactamente para esto), y pones un `api_key` o un secreto HMAC en `[kora.auth]` antes de que el puerto le dé la cara a internet, porque un paymaster sin autenticar es un faucet público con pasos extra. Alquilar un paymaster hosteado mueve la carga de ops y la custodia de la llave a otra persona, y mueve tu confianza ahí junto con ella: su config de validación, no la tuya, decide qué se patrocina en tu nombre, así que léela como un contrato. De cualquier manera el modelo mental es el mismo que construiste para el facilitador la lección pasada: un tercero en el camino del dinero cuyas reglas tienes que poder recitar. La diferencia es que este firma con una billetera que fondeas tú.

## Lab: construye gasless-checkout

Lo que estás ensamblando, y dónde se sienta en el workspace de Wavelength:

![Diagrama del workspace que muestra gasless-checkout reusando transfer-kit y checkout-txreq, hablando con un nodo de Kora local en el puerto 8080, y exportando buildSponsoredOrder para el capstone.](assets/v08-diagram.png)

1. **Instala el paymaster y acuña su firmante.** El nodo es un binario de Rust; el SDK de cliente que ya instalaste le habla. Después crea la billetera del patrocinador, la única billetera del lab que tiene SOL, y fondéala en devnet:

   ```bash
   cargo install kora-cli
   solana-keygen new --no-bip39-passphrase -o sponsor.json
   solana airdrop 2 $(solana-keygen pubkey sponsor.json) --url devnet
   ```

   Nota de pin: `cargo install kora-cli` resuelve a 2.0.5 al 2026-08-31 (publicado el 2026-03-11; las betas 2.2.0 son pre-release, y `cargo install` las ignora); corre `kora --version` y espera la línea 2.x. Este es el único paso de todo el curso que necesita un toolchain de Rust: si `cargo` no está en tu máquina, `rustup` (rustup.rs) lo instala en un comando, un desvío de cinco minutos — y si nunca tocaste Rust y preferirías conocer el toolchain como se debe en vez de instalarlo a ciegas, el curso Rust & TypeScript Fundamentals lo levanta desde cero al principio de su módulo cuatro, que abre prometiendo "En diez minutos vas a instalar el toolchain de Rust" y un scaffold además. Mantén `sponsor.json` fuera de git y lejos de tu billetera de comercio: el patrocinador es una cuenta de float que recargas, dimensionada para que perderla duela en vez de arruinarte. Checkpoint: el airdrop confirma y `solana balance $(solana-keygen pubkey sponsor.json) --url devnet` imprime 2 SOL.

2. **Configura el nodo.** Dos archivos en la raíz del proyecto. Primero `kora.toml`, que es la sección de validación de la teoría hecha literal (el esqueleto es una versión reducida de la propia config de ejemplo del repo; cada desvío de ese ejemplo está comentado):

   ```toml
   # gasless-checkout/kora.toml
   [kora]
   rate_limit = 100

   [kora.auth]
   # open for local dev; set api_key or hmac_secret before this port faces the internet

   [kora.enabled_methods]
   liveness = true
   estimate_transaction_fee = true
   get_supported_tokens = true
   sign_transaction = true
   sign_and_send_transaction = false  # the wallet submits; this node only ever co-signs
   transfer_transaction = false       # not a checkout verb; off
   get_blockhash = true
   get_config = true
   get_payer_signer = true
   get_version = true

   [validation]
   max_allowed_lamports = 1000000
   max_signatures = 2                 # sponsor + buyer, nothing else
   price_source = "Mock"              # devnet mints have no live oracle price; Jupiter in prod
   allow_durable_transactions = false # stays off: next lesson's offline queue signs durable-nonce txs, and it deliberately does NOT route them through Kora (the merchant pays those fees directly)
   allowed_programs = [
       "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", # Token program: the TransferChecked
       "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr", # Memo v2: the order stamp
       "ComputeBudget111111111111111111111111111111", # compute budget: wallets attach it
   ]
   allowed_tokens = [
       "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", # devnet USDC
   ]
   allowed_spl_paid_tokens = []
   disallowed_accounts = []

   [validation.price]
   type = "free"                      # merchant eats the fee; buyers pay zero

   # No [validation.fee_payer_policy] block on purpose: omitted means every
   # fee-payer power defaults to DENY. The repo's sample sets them all true.
   ```

   Después `signers.toml`, que le dice al nodo dónde vive su llave:

   ```toml
   # gasless-checkout/signers.toml
   [signer_pool]
   strategy = "round_robin"

   [[signers]]
   name = "wavelength_sponsor"
   type = "memory"
   private_key_env = "KORA_PRIVATE_KEY"
   weight = 1
   ```

   Un firmante de memoria lee su llave de la variable de entorno nombrada, y el valor puede ser base58, un array de bytes `[0, 1...]`, o un camino a un archivo JSON de keypair. La forma de camino quiere decir que la salida de tu `solana-keygen` funciona tal cual. Arranca el nodo en su propia terminal:

   ```bash
   KORA_PRIVATE_KEY=./sponsor.json kora --rpc-url https://api.devnet.solana.com \
     --config kora.toml rpc start --signers-config signers.toml
   ```

   Kora escucha en :8080 por defecto. Checkpoint, desde una segunda terminal:

   ```bash
   curl -s http://localhost:8080 -H 'content-type: application/json' \
     -d '{"jsonrpc":"2.0","id":1,"method":"getPayerSigner","params":[]}'
   ```

   El `signer_address` de la respuesta tiene que ser igual a `solana-keygen pubkey sponsor.json`. Esa dirección está a punto de volverse el fee payer de cada venta en la feria.

   Segundo checkpoint, y agarra el hábito: pregúntale al nodo qué cree. Cambia el método por `getConfig` en el mismo curl y lee el JSON de vuelta. Deberías ver tus tres programas permitidos, el único token permitido, `max_signatures` en 2, y un `fee_payer_policy` cuyos interruptores están todos en false, la postura de negar-por-defecto haciendo su trabajo sin que escribas una sola regla de negación, que honestamente es una bendición el día en que alguien edita este archivo apurado. Si la respuesta muestra en cambio la política todo-en-true de la config de ejemplo, el nodo cargó un `kora.toml` distinto del que acabas de escribir; arregla el camino antes de seguir, porque cada afirmación de seguridad de esta lección depende de cuál archivo leyó realmente ese proceso.

3. **El cliente de patrocinador.** Crea `src/sponsor.ts`. La construcción del cliente viene dada; el manejo de la cotización de comisión es tu primer TODO de completion, con sus reglas de aceptación sentadas justo encima:

   ```typescript
   // gasless-checkout/src/sponsor.ts
   // One KoraClient for the whole package, plus the quote-and-cap gate every
   // sponsored order passes through before we ask the node to sign anything.
   import { KoraClient } from '@solana/kora';

   const KORA_URL = process.env.KORA_URL ?? 'http://localhost:8080';

   export const USDC_DEVNET = '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU';

   // The most SOL we will ever sponsor for one checkout. Two signatures at the
   // 5000-lamport base fee is 10_000; the cap leaves headroom for a priority fee
   // without letting one weird transaction bite the float.
   export const MAX_SPONSOR_LAMPORTS = 20_000;

   export const kora = new KoraClient({ rpcUrl: KORA_URL });

   export interface SponsorQuote {
     feeToken: string;
     feeInLamports: number;
     signerAddress: string;
   }

   export async function quoteSponsorship(transactionBase64: string): Promise<SponsorQuote> {
     // TODO(completion) 1: three calls, three rules.
     // a) const { tokens } = await kora.getSupportedTokens();
     //    This reports [validation].allowed_tokens from kora.toml (the mints the
     //    node will validate inside transactions), NOT allowed_spl_paid_tokens
     //    (the mints buyers may pay fees in, empty in our free mode), so under
     //    the config you just wrote the list is exactly one entry: devnet USDC.
     //    Pick USDC_DEVNET out of it; if it is absent, throw. An empty or
     //    surprising list means the node config drifted, and you want that loud.
     // b) const est = await kora.estimateTransactionFee({
     //      transaction: transactionBase64, fee_token: <the mint from a> });
     // c) if (est.fee_in_lamports > MAX_SPONSOR_LAMPORTS) throw with both numbers
     //    in the message; otherwise return { feeToken, feeInLamports:
     //    est.fee_in_lamports, signerAddress: est.signer_pubkey }. (Yes, the
     //    node spells the same key `signer_address` on getPayerSigner and
     //    `signer_pubkey` on the estimate and sign calls; two spellings, one
     //    sponsor key, and under type = "free" the estimate's fee_in_token is
     //    informational only, since the buyer is never charged.)
     throw new Error('Your turn: quote the fee and enforce the cap per a, b, c above.');
   }
   ```

   ¿Para qué ponerle tope a algo que la config ya cotiza como free? Porque "free" es el precio del comprador, no el tuyo, y `estimateTransactionFee` reporta lo que al patrocinador le van a debitar de verdad. El tope es tu cortacircuitos para el día en que una billetera le agrega una priority fee absurda a una transacción que estás a punto de co-firmar.

   Listo se ve así, y el paso 7 es donde te enteras: `quoteSponsorship` devuelve un `SponsorQuote` cuyo `signerAddress` coincide con la dirección de `getPayerSigner` del paso 2, y lanza con los dos números en el mensaje en el momento en que una estimación excede `MAX_SPONSOR_LAMPORTS`.

4. **El constructor patrocinado.** Crea `src/build-sponsored-order.ts`. Completamente trabajado, y vale la pena leerlo con atención por lo poco que hay de nuevo: el precio es `priceOrder` del módulo 3, el ensamblaje es `finalizeTransaction` del módulo 3, y el único cambio estructural es de quién es la dirección que aterriza en el slot de `feePayer`:

   ```typescript
   // gasless-checkout/src/build-sponsored-order.ts
   // Same pricing, same assembly tail as checkout-txreq. One changed input:
   // the fee payer is the Kora signer, and Kora co-signs before the buyer sees it.
   import { address, generateKeyPairSigner, type Address } from '@solana/kit';
   import { getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta } from '../../transfer-kit/src/index';
   import { priceOrder, type OrderLine } from '../../checkout-txreq/src/catalog';
   import { finalizeTransaction } from '../../checkout-txreq/src/build-order-transaction';
   import { kora, quoteSponsorship } from './sponsor';

   const USDC_MINT = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const USDC_DECIMALS = 6;

   function merchantAddress(): Address {
     const configured = process.env.MERCHANT_ADDRESS;
     if (!configured) throw new Error('set MERCHANT_ADDRESS to the wallet checkout already pays');
     return address(configured);
   }

   export interface SponsoredOrderInput {
     account: string;      // the buyer's base58 pubkey, straight from the wallet POST
     lines: OrderLine[];
     orderId?: string;
   }

   export interface SponsoredOrder {
     transactionBase64: string; // already carries the sponsor's signature
     feePayer: string;          // the Kora signer; verify prints this
     reference: Address;
     memo: string;
     totalUsdc: string;
   }

   export async function buildSponsoredOrder(input: SponsoredOrderInput): Promise<SponsoredOrder> {
     const priced = priceOrder(input.lines);
     const buyer = address(input.account);
     const reference = (await generateKeyPairSigner()).address;
     const orderId = input.orderId ?? `fair-${Date.now().toString(36)}`;
     const memo = `wavelength:${orderId}:${priced.description}`;

     const { signer_address } = await kora.getPayerSigner();

     // Third seed since the roster lesson: the owning token program. Devnet
     // USDC is classic Token, so it is static here, exactly as in m03's builder.
     const sourceAta = await resolveAta(buyer, USDC_MINT, TOKEN_PROGRAM_ADDRESS);
     const destinationAta = await resolveAta(merchantAddress(), USDC_MINT, TOKEN_PROGRAM_ADDRESS);

     const transferIx = getTransferCheckedInstruction({
       source: sourceAta,
       mint: USDC_MINT,
       destination: destinationAta,
       authority: buyer,          // the buyer stays the transfer authority
       amount: priced.baseUnits,
       decimals: USDC_DECIMALS,
     });

     // The m03 tail, unchanged: reference injected, memo stamped, blockhash set.
     // The fee payer was always an input; today is the day that pays off.
     const unsignedBase64 = await finalizeTransaction({
       feePayer: address(signer_address),
       transferIx,
       reference,
       memo,
     });

     const quote = await quoteSponsorship(unsignedBase64);
     console.log(`[gasless] quote: ${quote.feeInLamports} lamports, cap ok`);

     const signed = await kora.signTransaction({ transaction: unsignedBase64 });

     return {
       transactionBase64: signed.signed_transaction,
       feePayer: signed.signer_pubkey,
       reference,
       memo,
       totalUsdc: priced.totalUsdc,
     };
   }
   ```

   El capstone importa `buildSponsoredOrder` por este nombre, así que el export es estructural igual que `finalizeTransaction` lo era en el módulo 3. También monta el servidor que estás a punto de escribir dentro del stack ensamblado, así que mantén las rutas de abajo en `/gasless` — esta es una superficie por derecho propio, no una rama de la app de transaction-request que da la casualidad de reusar su parte final. Fíjate también en lo que no cambió: ningún campo de monto en el input, nunca. Un checkout patrocinado sigue siendo un checkout, y el servidor sigue siendo el dueño del precio.

   Checkpoint: `npx tsc --noEmit -p tsconfig.json` desde `gasless-checkout` pasa limpio la comprobación de tipos. Apúntalo a la propia config de este workspace, no a la de la raíz — el `-p` es lo que hace real la comprobación, porque la config de la raíz del módulo 2 incluye solo `transfer-kit/src` y saldría con 0 en un archivo que nunca abrió. Con la config correcta es la forma más barata de agarrar un camino relativo obsoleto en los tres imports entre paquetes (`transfer-kit`, el catálogo de m03, el constructor de m03), o un helper de allá cuya firma se movió desde la última vez que lo llamaste, antes de que el servidor esconda cualquiera de los dos detrás de un 400.

5. **El servidor de la feria.** Crea `src/server.ts`. Completamente trabajado; es el par de transaction-request que ya conoces, en su propio puerto, devolviendo una transacción que ya lleva una de sus dos firmas:

   ```typescript
   // gasless-checkout/src/server.ts
   // The gasless endpoint: GET for display metadata, POST {account} returns a
   // PARTIALLY SIGNED base64 transaction. Kora signed as fee payer; the buyer's
   // wallet adds the second signature and submits.
   import express from 'express';
   import type { Request, Response } from 'express';
   import { buildSponsoredOrder } from './build-sponsored-order';
   import type { OrderLine } from '../../checkout-txreq/src/catalog';

   const app = express();
   app.use(express.json());

   // :3200 is also where module 6's ramp-embed session route listens, and that
   // port is registered in your CDP allowlist, so do not renumber it there.
   // Stop the ramp server before starting this one, or run this on another
   // port with PORT=3210 -- nothing here hardcodes 3200 but this default.
   const PORT = Number(process.env.PORT ?? 3200);

   const ORDERS = new Map<string, { lines: OrderLine[] }>([
     // The limited August pressing, priced at 30 in the m03 catalog and kept
     // at 30 exactly ever since. This is the sale from the lesson's opener.
     ['fair-045', { lines: [{ sku: 'WVL-045', quantity: 1 }] }],
   ]);

   app.get('/gasless', (_req: Request, res: Response) => {
     res.json({ label: 'Wavelength Records (fees on us)', icon: 'http://localhost:3100/icon.png' });
   });

   app.post('/gasless', async (req: Request, res: Response) => {
     const account: unknown = (req.body as { account?: unknown } | undefined)?.account;
     if (typeof account !== 'string' || account.length === 0) {
       res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
       return;
     }
     const orderId = typeof req.query.order === 'string' ? req.query.order : 'fair-045';
     const order = ORDERS.get(orderId);
     if (!order) {
       res.status(404).json({ message: `unknown order: ${orderId}` });
       return;
     }
     try {
       const built = await buildSponsoredOrder({ account, lines: order.lines, orderId });
       console.log(
         `[gasless] order ${orderId}: ${built.totalUsdc} USDC, fee payer ${built.feePayer}, ref ${built.reference}`,
       );
       res.json({
         transaction: built.transactionBase64,
         message: `Wavelength Records: ${built.totalUsdc} USDC, network fee on us`,
         feePayer: built.feePayer,
       });
     } catch (err) {
       const message = err instanceof Error ? err.message : 'could not build the sponsored order';
       console.log(`[gasless] REFUSED order ${orderId}: ${message}`);
       res.status(400).json({ message });
     }
   });

   app.listen(PORT, () => {
     console.log(`gasless-checkout listening on :${PORT}`);
   });
   ```

   Córrelo en una tercera terminal, con la misma billetera de comercio a la que el checkout le ha pagado todo el curso:

   ```bash
   MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
   ```

   Si eso sale con `EADDRINUSE`, la ruta de sesión de ramp-embed del módulo 6 todavía tiene el puerto: párala (ctrl-C en su terminal), o arranca esta con `PORT=3210` y pon `SERVER_URL` a juego en la comprobación de humo de abajo. La ruta de ramp se queda con :3200 porque ese origen exacto está registrado en tu allowlist de CDP y cambiarlo quiere decir editar un dashboard; este servidor no tiene esa atadura, así que es el que se mueve. Checkpoint: `gasless-checkout listening on :3200`, y la terminal de Kora se queda callada hasta que llega un POST. La línea de log `REFUSED` del bloque catch es la evidencia de negación que el Challenge y la barrera de esta lección te piden producir.

6. **Fondea al comprador con USDC y nada más.** La billetera de comprador que acuñaste en el scaffold no tiene SOL, y se queda así. Dale el dinero del disco usando tu propio kit del módulo 2, desde la raíz del workspace:

   ```bash
   npm run --workspace transfer-kit pay -- $(solana-keygen pubkey gasless-checkout/buyer.json) 31
   ```

   El dólar extra por encima del precio de 30 USDC del prensado es deliberado: después de la venta el comprador debería terminar en 1 USDC, no en 0, así que un débito de precio exacto se lee como éxito y un saldo drenado a cero se lee como un bug.

   Aquí pasan dos cosas que le hacen eco a la teoría. Tu billetera de comercio paga la comisión de transferencia, y también fondea la ATA de USDC del comprador, que es el evento de rent de la sección de costos, nada más pagado en el tramo de fondeo en vez del tramo de checkout. La misma economía, el mismo dueño: ese rent ahora vive en una cuenta que el comprador controla. Checkpoint: `solana balance $(solana-keygen pubkey gasless-checkout/buyer.json) --url devnet` imprime exactamente 0 SOL, y el comprobante del script de pay muestra 31 USDC entregados. Una billetera con dinero y sin gas: el coleccionista del principio, reproducido.

7. **La comprobación de humo.** Crea `verify/gasless.smoke.ts`. Hace de billetera del comprador: pide la transacción patrocinada, verifica quién es el fee payer, agrega la firma del comprador, la manda, y demuestra que el comprador pagó cero lamports. El ensamblaje de doble firma es tu segundo TODO de completion:

   ```typescript
   // gasless-checkout/verify/gasless.smoke.ts
   // Plays the wallet for a SOL-less buyer. Proves: fee payer is the Kora signer,
   // the buyer contributed exactly one signature, and the buyer paid 0 lamports.
   import { readFile } from 'node:fs/promises';
   import {
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     getBase64Encoder,
     getBase64EncodedWireTransaction,
     getCompiledTransactionMessageDecoder,
     getTransactionDecoder,
     partiallySignTransaction,
   } from '@solana/kit';
   import { KoraClient } from '@solana/kora';

   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
   const kora = new KoraClient({ rpcUrl: process.env.KORA_URL ?? 'http://localhost:8080' });
   const SERVER = process.env.SERVER_URL ?? 'http://localhost:3200';

   const bytes = new Uint8Array(JSON.parse(await readFile(new URL('../buyer.json', import.meta.url), 'utf8')));
   const buyer = await createKeyPairSignerFromBytes(bytes);

   const before = (await rpc.getBalance(buyer.address).send()).value;

   const res = await fetch(`${SERVER}/gasless?order=fair-045`, {
     method: 'POST',
     headers: { 'content-type': 'application/json' },
     body: JSON.stringify({ account: buyer.address }),
   });
   if (!res.ok) throw new Error(`server said ${res.status}: ${await res.text()}`);
   const { transaction } = (await res.json()) as { transaction: string };

   const sponsoredTx = getTransactionDecoder().decode(getBase64Encoder().encode(transaction));

   // Who is the fee payer? First static account of the message, by layout.
   const message = getCompiledTransactionMessageDecoder().decode(sponsoredTx.messageBytes);
   const feePayer = message.staticAccounts[0];
   const { signer_address } = await kora.getPayerSigner();
   if (feePayer !== signer_address) throw new Error(`fee payer ${feePayer} is not the Kora signer ${signer_address}`);
   if (feePayer === buyer.address) throw new Error('buyer is paying its own fee; sponsorship failed');

   // TODO(completion) 2: the dual-signature assembly.
   // a) const dualSigned = await partiallySignTransaction([buyer.keyPair], sponsoredTx);
   //    partiallySignTransaction ADDS the buyer's signature and preserves Kora's.
   // b) assert dualSigned.signatures[buyer.address] is non-null (the buyer signed),
   //    and that ALL entries in dualSigned.signatures are non-null (2 of 2 present).
   const dualSigned = sponsoredTx; // replace me

   const buyerSigned = dualSigned.signatures[buyer.address] != null;
   const totalSigs = Object.values(dualSigned.signatures).filter((s) => s != null).length;

   const signature = await rpc
     .sendTransaction(getBase64EncodedWireTransaction(dualSigned), { encoding: 'base64' })
     .send();

   for (let i = 0; i < 30; i++) {
     const status = (await rpc.getSignatureStatuses([signature]).send()).value[0];
     if (status?.confirmationStatus === 'confirmed' || status?.confirmationStatus === 'finalized') break;
     await new Promise((resolve) => setTimeout(resolve, 2000));
   }

   const after = (await rpc.getBalance(buyer.address).send()).value;

   console.log('sponsored=true');
   console.log(`feePayer=${feePayer} (kora signer, not the buyer)`);
   console.log(`buyerLamportDelta=${(after - before).toString()}`);
   console.log(`buyerSignatures=${buyerSigned ? 1 : 0} of ${totalSigs} total`);
   console.log(`signature=${signature}`);
   ```

   Con los dos TODOs llenos y las tres terminales corriendo (Kora en :8080, el servidor en :3200, este script), corre la barrera:

   ```bash
   npx tsx verify/gasless.smoke.ts
   ```

   Salida esperada, con tus propias direcciones:

   ```
   sponsored=true
   feePayer=Ay5u...your sponsor pubkey (kora signer, not the buyer)
   buyerLamportDelta=0
   buyerSignatures=1 of 2 total
   signature=4dJx...a devnet signature
   ```

   Ese cero es toda la lección. Una billetera que nunca tuvo un lamport acaba de comprar un prensado limitado, y la venta liquidó on-chain como cualquier otra. Si en cambio ves una falla de verificación de firma, tu TODO 2 mandó la transacción antes de que el comprador firmara; si la terminal de Kora muestra un rechazo, lee su razón, porque esa es tu config de validación hablando, y es exactamente la voz que quieres fuerte.

## Challenge

**Completion.** Llena los dos sitios TODO: `quoteSponsorship` en `src/sponsor.ts` según sus tres reglas, y el ensamblaje de doble firma en `verify/gasless.smoke.ts` según sus dos. La aceptación es la barrera de arriba, textual: un comprador de devnet sin SOL completa la compra, el fee payer impreso es igual al firmante de Kora y no al comprador, `buyerLamportDelta=0`, una firma del comprador presente. Guarda la firma de devnet y la pubkey decodificada del fee payer; son los artefactos de respuesta de esta lección.

**Solo, sin recorrido.** Tu paymaster hoy le tiene confianza a tu servidor. Demuestra que rechaza a todos los demás. Arma a mano una transacción que tu checkout nunca emitiría, una transferencia del System program es la clásica (fíjate que la allowlist de la config nunca incluyó el System program, a propósito: las ATAs existentes quieren decir que el checkout nunca lo toca). Constrúyela con las mismas llamadas del kit que usaste en el módulo 3, pon el fee payer en la dirección del firmante de Kora, y mándala derecho al `signTransaction` del nodo. Después apriétale los tornillos una vez más: agrega una entrada de `disallowed_accounts` o baja `max_signatures` a 1, reinicia el nodo, y mira fallar también tu propio checkout honesto, y después restáuralo. Aceptación: la petición fuera de la allowlist se rechaza con un error de Kora que nombra la violación, la línea de log `REFUSED` de tu servidor captura una negación de punta a punta, y el flujo honesto todavía pasa después. Guarda la línea de log de la petición negada al lado de la firma de devnet; la barrera pide las dos.

Si la transacción armada resulta patrocinada en vez de rechazada, comprueba cuál archivo de config cargó realmente el nodo que está corriendo antes de dudar de las reglas; un camino de `kora.toml` obsoleto es la falsa alarma clásica aquí, y el `getConfig` del SDK te va a mostrar exactamente qué cree el nodo.

Un pedido antes de que apagues las terminales. Esta lección levantó más partes móviles que cualquier peldaño hasta ahora: un nodo de Rust, dos archivos de config, tres procesos. Si alguna de ellas te peleó, anota cuál (el cargo install, el env del firmante, la arruga de peer-deps), porque el capstone asume que este stack arranca limpio y las notas de fricción son cómo llega ahí. Y si tu drain armado resultó rechazado en el primer intento, dilo en voz alta en algún lado; acabas de ver a una config de validación ganarse el sueldo, que es una cosa que la mayoría de la gente solo aprende del postmortem del incidente.

Tu puesto ahora puede vender a un comprador sin SOL. La próxima lección te quita algo más grande: la red. Una feria de discos en un sótano sin señal, ventas que todavía necesitan firmarse, y una fila que se drena cuando vuelves a estar en línea. El fee payer mantuvo la venta viva hoy; los nonces durables la mantienen viva offline.
