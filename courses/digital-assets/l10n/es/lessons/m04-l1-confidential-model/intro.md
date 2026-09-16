# El modelo confidencial: saldos cifrados en una blockchain pública

## Resumen

En la lección pasada resolviste las cuentas extra de un transfer hook desde el lado del cliente y viste exactamente por qué la mitad del ecosistema DEX rechaza el código arbitrario en la transferencia. Eso era código que podías leer. Esta extensión esconde los números mismos.

Porque ahora mismo puedes leer cada byte de una transferencia de SPROUT: remitente, destinatario, monto, todo a la vista. Un token de nómina no puede entregarse así. La empresa entera vería cada salario. Así que la pregunta que responde esta lección es: ¿cómo pones en un libro mayor público un monto que los validadores puedan verificar pero que nadie pueda leer?

Antes de cualquier término técnico, aquí va la intuición de 30 segundos. Puedes sumar dos sobres sellados de efectivo y saber que el total está bien sin abrir ninguno de los dos. Quédate con esa imagen. Es todo el truco, y todo lo que sigue es maquinaria construida a su alrededor.

Primero, algo para correr. La maquinaria tiene un verificador on-chain, y está vivo en mainnet ahora mismo. Sondéalo (curl ya viene con macOS y con todo Linux de uso común; en Debian, `apt install curl`):

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["ZkE1Gama1Proof11111111111111111111111111111",{"encoding":"base64"}]}'
```

Deberías recibir `"executable": true`, owner `NativeLoader1111111111111111111111111111111` y 24 bytes de datos. Esa cuenta es el ZK ElGamal Proof Program, el programa nativo que verifica cada prueba de conocimiento cero de esta lección. Esos 24 bytes los decodificamos en el lab.

El repliegue de la ayuda: esta es una lección de concepto, así que el ejemplo resuelto es una derivación, no un programa. Recorro el modelo de punta a punta contigo, incluida una transferencia confidencial de SPROUT desarmada campo por campo. En el lab produces el artefacto tú mismo: una tabla de campos público-contra-cifrado más las tres pruebas, cada una derivada del engaño que cierra. El challenge es en solitario: atacas el modelo con una prueba de menos y predices exactamente qué se rompe. Hoy no se entrega código nuevo. La próxima lección configura todo esto de verdad.

## Derivando el modelo confidencial

### Sobres sellados, y luego el término técnico

Imagina un notario que salda deudas entre personas que se niegan a revelar sus salarios. Cada persona entrega un sobre sellado con efectivo adentro. El notario no puede abrir ningún sobre. Pero son sobres especiales: apila dos y la pila se comporta como un solo sobre que contiene la suma. El notario puede verificar que el sobre A más el sobre B pesa exactamente lo que pesa el sobre C, sin ver jamás un solo billete.

Esa propiedad tiene nombre: un commitment homomórfico. "Commitment" porque el sobre te ata a un valor que después no puedes cambiar. "Homomórfico" porque las operaciones sobre los sobres sellados se corresponden con operaciones sobre los valores ocultos: suma los ciphertexts y ya sumaste los montos de adentro.

![Un notario apila dos sobres sellados y verifica que el sobre combinado contiene la suma sin abrir ninguno, mapeando sobres a ciphertexts y el apilado a la suma de ciphertexts.](assets/v01-diagram.webp)

La extensión de transferencia confidencial de Token-2022 es este notario, industrializado. Todo saldo confidencial en la blockchain es un sobre sellado. Toda transferencia confidencial es el validador apilando sobres: resta este ciphertext del saldo del remitente, suma aquel otro a la pila pendiente del destinatario. La blockchain hace aritmética sobre números que nunca ve.

La construcción del sobre se llama Twisted ElGamal. No necesitas su álgebra para usarla bien, pero sí necesitas tres de sus propiedades, porque de ellas sale cada decisión de diseño que viene después:

1. El cifrado se hace contra una clave pública, así que cualquiera puede sellar un sobre dirigido a ti. Un remitente cifra el monto de la transferencia bajo tu clave sin que tú participes.
2. Los ciphertexts se suman. La propiedad homomórfica de la intuición de arriba es real: el programa literalmente llama a la suma y la resta de ciphertexts sobre los saldos (`ciphertext_arithmetic::add` y `subtract_from` en el procesador).
3. Descifrar es caro. Abrir tu propio sobre es resolver un logaritmo discreto, y eso solo es tratable cuando el número oculto es pequeño. Esta única propiedad, que descifrar sea el camino lento, moldea más el diseño de la extensión que ninguna otra. Tenla cargada.

### Dos claves por cuenta: una para la blockchain, otra para ti

Aquí está la primera consecuencia. Si descifrar un ciphertext ElGamal es lento, ¿cómo te muestra una billetera tu propio saldo sin machacar una búsqueda de logaritmo discreto cada vez que abres la app?

La respuesta de la extensión: cada cuenta confidencial lleva dos cifrados del mismo saldo, bajo dos claves distintas, al servicio de dos amos distintos. Mira el estado real de la cuenta, de `spl_token_2022_interface` (recortado a los campos que importan hoy):

```rust
pub struct ConfidentialTransferAccount {
    /// `true` if this account has been approved for use.
    pub approved: Bool,
    /// The public key associated with ElGamal encryption
    pub elgamal_pubkey: PodElGamalPubkey,
    /// The low 16 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_lo: EncryptedBalance,
    /// The high 48 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_hi: EncryptedBalance,
    /// The available balance (encrypted by `elgamal_pubkey`)
    pub available_balance: EncryptedBalance,
    /// The decryptable available balance
    pub decryptable_available_balance: DecryptableBalance,
    /// If `false`, the account rejects incoming confidential transfers
    pub allow_confidential_credits: Bool,
    /// If `false`, the base account rejects any incoming transfers
    pub allow_non_confidential_credits: Bool,
    /// Number of Deposit and Transfer instructions that have credited pending
    pub pending_balance_credit_counter: U64,
    /// Max credits allowed before ApplyPendingBalance must run
    pub maximum_pending_balance_credit_counter: U64,
    // ...expected/actual credit counter bookkeeping trimmed
}
```

El `available_balance` es la copia del mint: un ciphertext Twisted ElGamal sobre el que la blockchain puede hacer aritmética, y contra el que se verifica cada prueba. El `decryptable_available_balance` es tuyo: el mismo número sellado con una clave AES que solo tú tienes (`AeCiphertext` en el código fuente). Descifrar AES es instantáneo. Tu billetera lee ese campo, descifra en un microsegundo y te muestra tu salario. La blockchain nunca lo toca; el programa solo guarda el nuevo ciphertext descifrable que le pases cada vez que tu saldo cambia, porque solo tú puedes producirlo.

Un saldo, dos sobres, dos audiencias. El ciphertext ElGamal es la verdad que impone la blockchain. El ciphertext AES es un caché de conveniencia para su dueño. Si alguna vez se separan, gana la copia de la blockchain, y la billetera tiene que recurrir a abrir el sobre ElGamal por las malas. La propiedad 3 debería hacerte dar un respingo ante esa frase, y con razón: una búsqueda de logaritmo discreto sobre todo el rango de 64 bits no es práctica. La vía de escape es que la búsqueda está acotada por lo que el saldo puede plausiblemente ser, trabaja en los mismos chunks pequeños que impone el resto de la extensión, y puede precomputarse y reanudarse offline, así que recuperarlo es lento-pero-finito para saldos realistas en vez de instantáneo. Trata el caché AES como estructural, no decorativo, y trata perder la clave AES como un incidente.

![Diagrama que divide un saldo oculto en dos ciphertexts almacenados, una copia ElGamal sobre la que la blockchain calcula pero que los dueños descifran lentamente, y una copia AES que los dueños leen al instante.](assets/v02-diagram.webp)

### Pendiente contra disponible: por qué el dinero entrante se queda en una sala de espera

Ahora la segunda consecuencia, y explica los campos más raros de ese struct: ¿por qué existe siquiera un `pending_balance`, partido en mitades `lo` y `hi`?

Recórrelo. Alguien te manda una transferencia confidencial. El monto llega cifrado bajo tu clave ElGamal, y la blockchain lo suma homomórficamente a tu saldo. Bien. Pero tu `decryptable_available_balance`, el caché AES, ya quedó desactualizado, y el remitente no puede arreglarlo: producir un ciphertext AES nuevo requiere tu clave AES, que el remitente no tiene y nunca debe tener.

Peor: si las transferencias entrantes cayeran directo en `available_balance`, entrarían en carrera con tus propios gastos. Generas una prueba contra el saldo X, alguien te acredita en pleno vuelo, tu saldo ahora es X más algo que no puedes ver, y tu prueba ya no coincide con el ciphertext en la blockchain. Cada pago entrante invalidaría cada pago saliente que tuvieras en curso.

Así que la extensión le da a cada cuenta una sala de espera. Los créditos confidenciales entrantes caen en `pending_balance`, y solo el dueño los mueve a `available_balance` firmando una instrucción `ApplyPendingBalance`, que además le entrega al programa un caché AES recién re-cifrado. El `pending_balance_credit_counter` cuenta los depósitos desde la última aplicación, y `maximum_pending_balance_credit_counter` limita cuántos pueden amontonarse (65,536 por defecto) antes de que la cuenta deje de aceptar créditos hasta que el dueño la vacíe. La partición en un ciphertext `lo` y uno `hi` existe por la razón que ya tienes: descifrar es una búsqueda de logaritmo discreto, así que cada chunk cifrado tiene que quedarse lo bastante pequeño como para que su dueño lo abra. Sé preciso sobre cuál partición es cuál, porque difieren. El monto de una transferencia entrante llega como un chunk bajo de 16 bits más un chunk alto de 32 bits (esa forma de 16 + 32 es exactamente de donde sale el tope de transferencia por debajo de 2^48 que viene más adelante en esta lección), y cada chunk se suma a su propio bucket pendiente. Los buckets mismos cargan peso posicional, `lo` para los 16 bits bajos del saldo y `hi` para los 48 bits de arriba, y son deliberadamente más holgados que cualquier transferencia individual para que hasta 65,536 créditos puedan acumularse entre una aplicación y la siguiente mientras los dos buckets se quedan dentro de un rango de descifrado buscable.

![Diagrama de flujo de un crédito confidencial entrante que cae en el saldo pendiente y espera a que el ApplyPendingBalance del dueño lo pliegue dentro del saldo disponible y refresque el caché AES.](assets/v03-flowchart.webp)

Si alguna vez usaste un banco que muestra los depósitos "en proceso" aparte de tu saldo gastable, ya tienes la forma de esto. La diferencia es el motivo: el banco está corriendo verificaciones de fraude, mientras que esta cuenta espera a la única persona viva que puede volver a sellar el sobre legible.

### Las tres pruebas, derivadas de los tres engaños

Aquí es donde el modelo se gana el sueldo, y donde te quiero derivando en vez de memorizando. La blockchain es un notario que hace aritmética sobre sobres que no puede abrir. Así que haz la pregunta adversarial: si nadie puede ver los montos, ¿qué me impide mentir?

Prueba primero los arreglos ingenuos. "Que los validadores descifren y verifiquen" se refuta solo; el punto entero es que no pueden. "Confía en la aritmética del remitente" muere en un bloque; alguien se transfiere a sí mismo un sobre que dice menos un millón y el supply se infla en silencio. La respuesta real es la que le faltaba a la historia del sobre sellado: junto a los sobres, el remitente tiene que adjuntar pruebas de conocimiento cero, enunciados que convencen al verificador de que una afirmación sobre los valores ocultos es verdadera sin revelar nada más sobre ellos.

Bien. Pero ¿por qué TRES pruebas? ¿Por qué no una sola prueba que diga "esta transferencia es honesta"? Porque "honesta" no es una sola afirmación. Siéntate a intentar engañar de verdad a este sistema y vas a encontrar exactamente tres mentiras distintas al alcance de un remitente, y cada una necesita su propia refutación. Esta es la derivación por la que existe la lección, así que tómatelo con calma.

Engaño uno: probar en rango un remanente fabricado. Aquí está la sutileza que hace que este engaño sea siquiera posible. La resta homomórfica de la blockchain produce tu saldo nuevo como un ciphertext, pero una prueba de rango (la refutación del engaño tres) no corre sobre ese ciphertext directamente: prueba enunciados sobre commitments que aporta el REMITENTE, incluido uno para el saldo que el remitente dice que le queda después del débito. La blockchain no puede abrir su propio ciphertext posterior a la resta para verificar la afirmación, así que hasta aquí nada ata el remanente declarado a la realidad. Yo podría tener 3 SPROUT, mandarte 5, y entregarle al verificador un "saldo restante" de 10 bellamente bien formado y cómodamente dentro de rango que inventé para la ocasión, mientras mi saldo verdadero daba la vuelta a negativo por debajo. La refutación es una prueba de igualdad, `CiphertextCommitmentEqualityProof` en el código fuente: certifica que tu ciphertext de saldo disponible nuevo, el que produce la resta on-chain, se compromete con el mismo valor que el commitment del remanente sobre el que testifica el resto del paquete de pruebas. El remanente declarado ES el remanente real, así que cada garantía que dan las otras pruebas se adhiere a los libros de verdad, no a un cuento sobre ellos.

![La prueba de igualdad suelda el commitment del remanente declarado por el remitente al ciphertext de saldo posterior al débito de la blockchain, y cierra el engaño del remanente fabricado.](assets/v04-diagram.webp)

Engaño dos: mandar basura. Los ciphertexts ElGamal son solo puntos de curva; nada en los bytes los obliga a ser un cifrado bien formado de nada bajo la clave de nadie. Yo podría entregarte un "ciphertext" que descifra a un sinsentido bajo tu clave o, peor, cifrar el monto real para ti pero adjuntar bytes destrozados para el auditor, así que el cumplimiento ve ruido mientras la transferencia pasa sin problema. La refutación es una prueba de validez de ciphertext agrupado, `BatchedGroupedCiphertext3HandlesValidityProof`: el monto está correctamente cifrado, como un único ciphertext agrupado con tres handles, bajo la clave del remitente, la clave del destinatario Y el auditor key opcional del mint. El mismo número, tres lectores, demostrablemente. Esta es la prueba que le da sentido al asiento de auditor en `ConfidentialTransferMint`; ese asiento lo configuramos en la próxima lección.

Engaño tres: ponerse en negativo. La aritmética de ciphertexts es aritmética módulo un orden de grupo, y la aritmética modular no sabe qué es un número negativo. Restar un "monto" que da la vuelta es la versión confidencial de un underflow de entero, y ya sabes lo que un underflow le compra a un atacante en texto plano: resta uno de un saldo cero y aterrizas en casi 2^64. La refutación es una prueba de rango, `BatchedRangeProofU128`: todo monto oculto de la transferencia es no negativo y está dentro de los límites. El U128 del nombre es contabilidad honesta. Una sola prueba en lote cubre tu saldo restante (64 bits, el commitment del remanente que aporta el remitente y que la prueba de igualdad del engaño uno suelda a los libros de verdad) más el chunk bajo del monto de la transferencia (16 bits) y el chunk alto (32 bits), rellenado con 16 hasta una potencia de dos: 128 bits de valores comprometidos, probados en rango juntos.

Tres mentiras, tres pruebas, y la asignación es exacta. Quita cualquiera y su engaño se reabre; lo vas a demostrar tú mismo en el challenge.

![Diagrama de mapeo que empareja cada uno de los tres engaños del remitente con la prueba de conocimiento cero que lo cierra y la garantía que da cada una, todas verificadas por el ZK ElGamal Proof Program.](assets/v05-diagram.webp)

La verificación, cosa notable, no la hace el propio Token-2022. El SIMD-0153 le dio a la red un programa nativo dedicado para esto, el ZK ElGamal Proof Program que sondeaste en el resumen, vivo en `ZkE1Gama1Proof11111111111111111111111111111`. Token-2022 confirma que cada prueba fue verificada por ese programa y después hace la aritmética de sobres. División del trabajo: un programa que sabe de criptografía, un programa que sabe de tokens.

### Por qué una transferencia son varias transacciones, y por qué los montos se detienen en 2^48

Así que una transferencia confidencial son ciphertexts más tres pruebas. Ahora el hecho operativo feo: esas pruebas son grandes. Una sola prueba de rango llega a cientos de bytes, y el trío junto se pasa por mucho de lo que cabe al lado de una instrucción de transferencia dentro de la transacción de 1,232 bytes de Solana. Las pruebas son demasiado grandes para viajar junto, así que una transferencia lógica se vuelve hoy varias transacciones dependientes.

El mecanismo que hace esto viable es la cuenta de estado de contexto: una cuenta de vida corta, propiedad del programa de pruebas, que registra "la prueba X fue verificada" para que una transacción posterior pueda apuntar a ella en vez de cargar la prueba. El baile, en orden:

1. Crear y verificar: por cada prueba, una transacción le entrega la prueba al ZK ElGamal Proof Program, que la verifica y escribe una cuenta de contexto (la prueba de rango suele necesitar una transacción para ella sola).
2. Transferir: se ejecuta la instrucción `Transfer` real de Token-2022, referenciando las tres cuentas de contexto en vez de pruebas en línea.
3. Cerrar: las cuentas de contexto se cierran y su rent se recupera.

![Diagrama de flujo de una transferencia confidencial partida en transacciones dependientes: primero las pruebas verificadas dentro de cuentas de contexto, después la transferencia que las referencia, después la limpieza de las cuentas de contexto, todo restringido por el límite de 1,232 bytes por transacción.](assets/v06-flowchart.webp)

Esto no es para siempre, y ya se está moviendo. El formato de transacción v1 (la línea del SIMD-0296, ahora retomada por el SIMD-0385) amplía el margen precisamente para que flujos como este puedan colapsar en una sola transacción, y ya se entregó en Agave. Pero entregado no es activado, y activado es por cluster. El feature set de Agave nombra el gate `enable_tx_v1` y declara su dirección en `feature-set/src/lib.rs`:

```bash
solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url mainnet-beta
solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url devnet
```

El 2026-09-06 mainnet respondió `Error: AccountNotFound` — sin cuenta, así que no activado y ni siquiera preparado — mientras que devnet devolvió una cuenta propiedad de `Feature111111111111111111111111111111111111` cuyos nueve bytes de datos decodifican como una etiqueta `1` seguida del slot de activación 492,480,000. Devnet lo tiene. Mainnet no. Así que 1,232 bytes sigue siendo la ley donde están tus usuarios, el baile de varias transacciones sigue siendo la realidad para la que diseñas, y devnet es ahora un cluster donde esta restricción en particular calladamente no se reproduce — lo cual es su propia trampa si solo pruebas ahí.

Dos notas sobre el sondeo mismo, porque el obvio no funciona. `solana feature status` imprime solo los gates compilados dentro de la CLI que tienes en la mano: 76 filas en solana-cli 3.1.10, y este gate no está entre ellos, así que tanto `| grep -i tx_v1` como `solana feature status <that address>` vuelven vacíos o con `Unknown feature`. Leer la cuenta directamente es el sondeo que no se puede volver obsoleto, porque le pregunta a la blockchain y no al binario. Y vuelve a revisarlo antes de citarle este párrafo a nadie: cambió en un cluster entre la redacción de esta lección y su última revisión.

La última restricción es el tope del monto, y a estas alturas puedes derivarlo tú mismo. Los montos se cifran en chunks lo bastante pequeños como para descifrarlos (un chunk bajo de 16 bits, un chunk alto de 32 bits), así que un solo depósito o transferencia queda topado por debajo de 2^48. El código fuente lo declara como una constante:

```rust
/// Maximum bit length of any deposit or transfer amount
///
/// Any deposit or transfer amount must be less than 2^48
pub const MAXIMUM_DEPOSIT_TRANSFER_AMOUNT: u64 =
    (u16::MAX as u64) + (1 << 16) * (u32::MAX as u64);
```

Eso evalúa a 281,474,976,710,655 unidades base. Para SPROUT con 6 decimales, una transferencia confidencial llega como máximo a unos 281 millones de tokens enteros, de sobra para una nómina. Pero no es un u64 de rango completo, y un mint con 9 decimales pierde tres órdenes de magnitud. La aritmética de techos va en tu revisión de diseño, no en las notas de incidentes de producción.

### La derivación resuelta: una transferencia confidencial de SPROUT, campo por campo

Ahora arma el modelo entero diseccionando una transferencia. Digamos que te mando 5 SPROUT de forma confidencial. Aquí está todo lo que llega al libro mayor, ordenado por quién puede leerlo. Esta tabla es el patrón para el artefacto que produces en el lab, así que léela como una clave de respuestas resuelta.

| Campo en tránsito | Público o cifrado | Quién puede leerlo, y qué prueba lo condiciona |
| --- | --- | --- |
| Cuenta de token del remitente (y pubkey del dueño) | Público | Todos. No hay prueba involucrada; las firmas autorizan como siempre |
| Cuenta de token del destinatario (y pubkey del dueño) | Público | Todos. Cifrado no es anónimo |
| Dirección del mint, programa, el hecho de que hubo una transferencia | Público | Todos. El análisis de tráfico ve la arista, no el peso |
| Monto de la transferencia, ciphertext agrupado (lo + hi) | Cifrado | Solo las claves del remitente, del destinatario y del auditor; condicionado por la prueba de validez (bien formado bajo los tres handles) |
| Ciphertext del nuevo saldo disponible del remitente | Cifrado | Solo el remitente; condicionado por la prueba de igualdad (se compromete con el valor verdadero posterior al débito) |
| Nuevo saldo descifrable del remitente (AES) | Cifrado | Solo el remitente; sin prueba, la blockchain lo guarda a ciegas |
| No negatividad y límites de todo valor oculto | Probado, no revelado | Condicionado por la prueba de rango sobre 64 + 16 + 32 bits comprometidos |

Fíjate a qué suma la columna pública: las dos identidades, el token, el momento, la comisión de la transacción pagada en SOL visible. La confidencialidad aquí es exactamente una propiedad, montos ocultos, y nada más. Un analista todavía puede dibujar tu grafo de pagos entero; simplemente no puede ponderar las aristas. Si tu modelo de amenazas necesita participantes ocultos, esta extensión no lo da, punto, y pretender lo contrario es así como los equipos de cumplimiento se llevan sorpresas desagradables.

![Comparación lado a lado de una transferencia de SPROUT normal y una confidencial, donde los campos de identidad y de mint quedan públicos y solo los campos de monto y de saldo pasan a forma cifrada con tres pruebas.](assets/v07-comparison.webp)

### Lo que cuesta la confidencialidad

Cada extensión de este curso ha traído su etiqueta de precio, y la de esta es cara: la confidencialidad se compra con componibilidad y con simplicidad.

La componibilidad primero. Un AMM le fija precio a un canje leyendo los saldos y los montos del pool. Montos cifrados quieren decir que no hay nada que leer, así que ningún AMM puede ponerle precio al token. Esto no es cautela hipotética: ya te topaste dos veces con el filtro de mints de Raydium, primero como la allowlist de cinco extensiones de m02-l1 y otra vez en la lección del transfer hook, y su política hacia esta extensión es el rechazo categórico, sobre la base declarada de que los montos cifrados impiden fijar precios. Sé preciso sobre el alcance de eso, porque el módulo 5 te lo va a exigir: lo que ningún AMM puede fijar es el precio de un monto cifrado, no necesariamente el de un mint que meramente lleva la extensión. La allowlist de Raydium rechaza la extensión misma, mientras que la tabla publicada de Orca soporta esos mints "solo para transferencias no confidenciales". De cualquier modo el camino confidencial es el no enrutable. Para una nómina eso es irrelevante. Para cualquier cosa que necesite un mercado líquido, es descalificante, y ninguna cantidad de ingeniería de tu lado lo cambia.

La simplicidad, segunda. Una transferencia lógica son varias transacciones dependientes con generación de pruebas en medio, lo cual quiere decir flujos de cliente, reintentos y estados de falla que no tienes con una transferencia normal. Los montos topan por debajo de 2^48. Los dos lados de una transferencia necesitan cuentas confidenciales configuradas, con claves ElGamal y AES derivadas y gestionadas. Y encima apila cuatro trampas, cada una, una restricción de diseño que heredas en el momento en que echas mano de esta extensión:

- La transferencia confidencial tiene que habilitarse en la creación del mint. No puedes agregársela después a un mint existente; no existe el retrofit, solo un mint nuevo y una migración.
- Cifrado no es anónimo. Las direcciones del remitente y del destinatario quedan completamente públicas; solo el monto está oculto.
- Un transfer hook no puede ver ni actuar sobre montos confidenciales. Las dos extensiones sí componen mecánicamente — una transferencia confidencial igual invoca el hook, pasándole el monto centinela `u64::MAX`, y el propio mint de PYUSD lleva las dos — pero tu hook se queda ciego a los montos en el camino confidencial. Todo hook cuya lógica se condicione a los montos tiene que diseñarse para ese centinela, o el emparejamiento es una trampa.
- El tope por debajo de 2^48 quiere decir que un saldo confidencial no es un u64 de rango completo, y tu validación de montos tiene que decirlo.

Cuándo NO echar mano de esto, entonces, se reduce a una sola prueba: todo token que tenga que negociarse en un DEX o liquidarse en una sola transacción queda fuera. Lo que queda es el caso de la nómina, la liquidación B2B, las operaciones de tesorería: flujos entre contrapartes que ya se conocen y simplemente no quieren los montos en una valla publicitaria.

Una nota de honestidad antes del lab, porque este curso no exagera la adopción. Al momento de escribir esto no hay ningún emisor de producción con nombre corriendo transferencias confidenciales a escala al que pueda apuntarte. El modelo es real y está vivo; el despliegue emblemático no. PYUSD ya viene con la suite confidencial entre sus ocho extensiones TLV, configurada pero dormida, y vas a leer su config dormida tú mismo en unos dos minutos. El rastro documental del ecosistema es raro de la misma manera: solana.com/solutions/token-extensions, una página oficial viva, todavía dice que las transferencias confidenciales llegarán en cuanto Agave 2.0 "sea adoptado por la red, lo que se espera que ocurra para fin de 2024". Pregúntale a la red qué corre de verdad; un sondeo lo resuelve:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}'
```

El 2026-08-22 eso devolvió `"solana-core": "4.2.0"`, la línea Agave. Dos versiones mayores después de la promesa, la promesa sigue en la página. La misma página sigue siendo útil como cita de las cinco firmas de auditoría que revisaron el programa de la extensión (Halborn, Zellic, Trail of Bits, NCC Group, OtterSec). Y la educación oficial de Solana se congeló a mitad de la trama: el repo solana-foundation/developer-content se archivó en modo de solo lectura el 2025-01-24, así que todo curso oficial es anterior a la forma actual de esta suite. Que es más o menos por lo que existe esta lección. Estás aprendiendo material cuyo rastro de documentación dejó de moverse antes que la maquinaria.

![Tarjeta de trade-off de dos paneles que lista lo que dan las transferencias confidenciales, montos ocultos verificables, frente a costos como la componibilidad DEX perdida, la liquidación en varias transacciones y el tope de monto, y termina en una regla de decisión.](assets/v08-comparison.webp)

## Lab: sondea la maquinaria, y después deriva el modelo en papel

El artefacto producido para esta lección es una tabla de campos llena más las tres pruebas con sus garantías, de tu puño y letra. Los pasos 1 y 2 son lecturas en vivo contra mainnet; los pasos 3 a 5 son la derivación. Necesitas `curl` (ya probado funcionando por la apertura) y `python3` (ya viene con macOS; en Debian, `apt install python3`). Las respuestas RPC de abajo se capturaron el 2026-08-22 contra un nodo que reportaba Agave 4.2.0; las cuentas vivas se mueven, así que espera que tus bytes coincidan y tus números de slot no.

1. Decodifica el verificador que sondeaste en el resumen. Esos 24 bytes de datos de cuenta son base64; ábrelos:

   ```bash
   echo "emtfZWxnYW1hbF9wcm9vZl9wcm9ncmFt" | base64 -d
   ```

   Salida esperada: `zk_elgamal_proof_program`. Los programas nativos llevan su nombre como datos de cuenta, así que acabas de leer la placa con el nombre del verificador on-chain. Fíjate en lo que implica su existencia: la verificación de pruebas es una primitiva a nivel de red (SIMD-0153), no algo que cada programa de tokens reimplementa.

2. Lee una config confidencial dormida en el mundo real. El mint de PYUSD lleva el par confidencial entre sus ocho extensiones TLV. Trae el mint parseado y fíltralo:

   ```bash
   curl -s https://api.mainnet-beta.solana.com -X POST \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
   | python3 -c "import json,sys; exts=json.load(sys.stdin)['result']['value']['data']['parsed']['info']['extensions']; print(json.dumps([e for e in exts if e['extension']=='confidentialTransferMint'], indent=2))"
   ```

   Esperado: una entrada `confidentialTransferMint` con `auditorElgamalPubkey: null`, un `authority` y `autoApproveNewAccounts: false`. Lee eso contra la lección: no hay auditor key puesto, y con el flag en false, los tenedores todavía pueden configurar cuentas confidenciales libremente, pero cada cuenta configurada queda inservible hasta que el emisor la apruebe explícitamente. Configurado pero dormido, verificado por tu propia lectura; la próxima lección recorre ese flujo de configurar-y-después-aprobar de punta a punta.

3. Arma la tabla de campos. Toma la transferencia "mandas 5 SPROUT de forma confidencial a un compañero de equipo" y escribe una tabla de dos columnas: cada campo que llega al libro mayor en la columna uno, `public` o `encrypted` en la columna dos. Trabaja desde la sección de disección pero escríbela primero a libro cerrado; estás verificando si el modelo está en tu cabeza o todavía en la página. Filas mínimas: cuenta del remitente, cuenta del destinatario, mint, el hecho de la transferencia, el monto de la transferencia, el ciphertext del nuevo saldo del remitente, el caché AES.

4. Deriva las tres pruebas. Debajo de la tabla, escribe los tres engaños que un remitente podría intentar, con tus propias palabras. Para cada engaño, nombra la prueba que lo cierra (nombres de tipo exactos: `CiphertextCommitmentEqualityProof`, `BatchedGroupedCiphertext3HandlesValidityProof`, `BatchedRangeProofU128`) y enuncia la única garantía que da, una línea cada una. Si alguna garantía te lleva más de una línea, estás describiendo el mecanismo, no la garantía; comprime hasta que sea una afirmación.

5. Cierra el círculo con los contadores. Agrega una línea final a tu artefacto que responda: ¿por qué el remitente no puede actualizar tu saldo descifrable, y qué instrucción lo arregla? Si tu respuesta nombra la clave AES y `ApplyPendingBalance`, la maquinaria del saldo pendiente ya aterrizó.

Checkpoint: tu tabla marca exactamente una familia de campos como cifrada (los ciphertexts de monto y de saldo) y todo lo que tenga forma de identidad como público; tus tres líneas de prueba emparejan cada una un engaño con un nombre de tipo y una garantía. Ese artefacto es el criterio de evaluación de esta lección, y la próxima lección asume que puedes reproducirlo de memoria.

## Challenge

Solo, sin apoyo: rompe el modelo tres veces, en papel.

Para cada una de las tres pruebas, asume que el verificador se saltó esa prueba y solo esa, y escribe el ataque concreto que corre un remitente malicioso: qué envía, qué acepta la blockchain y cuál es el daño (supply inflado, saldo corrupto, auditor ciego). Tres ataques, un párrafo corto cada uno. El ejercicio fuerza el punto que la lección derivó: las pruebas no son defensa en profundidad, son tres cerraduras en tres puertas distintas, y cualquier puerta abierta es fatal.

Stretch, para quienes bucean en el código fuente: el procesador de transferencias de `token-2022` acepta dos offsets de prueba opcionales adicionales, `fee_sigma_proof` y una prueba de validez de ciphertext de comisión, usados cuando el mint también lleva comisiones de transferencia confidenciales. Antes de leer más a fondo la extensión de comisiones, predice desde primeros principios qué dos engaños nuevos introduce una comisión oculta. Tienes todas las herramientas que necesitas: una comisión es solo un monto oculto más sobre el que alguien podría mentir.

## Checkpoint y lo que viene después

Ahora puedes enunciar el modelo confidencial sin vaguedades: los saldos son sobres Twisted ElGamal que la blockchain suma sin abrir, los dueños guardan un caché AES de lectura rápida, los créditos entrantes esperan en un bucket pendiente hasta que se apliquen, y cada transferencia arrastra tres pruebas de conocimiento cero hasta el ZK ElGamal Proof Program, una por cada engaño disponible. También conoces la factura: ningún DEX le va a fijar precio, una transferencia son varias transacciones hoy, los montos se detienen por debajo de 2^48, y nada de esto se puede atornillar después de la creación del mint. Ese es un modelo mental completo, producido por ti, en papel, y con franqueza eso te pone por delante de la mayor parte del material escrito del ecosistema sobre esta extensión.

Ahora tienes el modelo: commitments, tres pruebas, un asiento de auditor opcional todavía vacío. La próxima lección lo configuras de verdad: un auditor key, el registro ElGamal, el supply confidencial y el muro de varias transacciones con el que de verdad vas a chocar cuando SPROUT se apague. Ten las tres pruebas a mano; todo el poder del auditor en la próxima lección depende de la del medio.
