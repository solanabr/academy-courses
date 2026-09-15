# Token Metadata como legado: pNFTs y la realidad de las regalías

## Resumen

En m06-l2 entregaste R7: la colección Core Almanac verificada, activos que llevan los plugins Royalties, Edition y PermanentFreeze, y un badge Founding-Farmer soulbound, todo sobre Metaplex Core, con regalías que configuraste tú mismo. Esta lección es la contraparte incómoda. No vas a entregar nada nuevo sobre Token Metadata, y al final vas a poder decir con precisión por qué nadie debería: vas a leer el estándar en el que todavía vive la mayoría de los NFTs existentes, decodificar el rule set emblemático que se suponía que imponía sus regalías, y descubrir con tu propio herramental que hoy bloquea cero programas. Las habilidades de aquí son habilidades de evaluación: leer el rule set de un pNFT en vivo, emitir un veredicto de impuesto-o-no con evidencia, y etiquetar `seller_fee_basis_points` por lo que es, un número que el runtime nunca toca. El repliegue: la decodificación emblemática se trabaja de punta a punta en el lab, salida esperada incluida; la excavación del historial de revisiones y el memo de veredicto por activo en el challenge son tuyos, en solitario, y son el trabajo de verdad.

Aquí está el planteo. Un marketplace lista un NFT legado, `seller_fee_basis_points = 500`, y el texto del anuncio promete una regalía de creador garantizada del 5%. Quinientos basis points, on-chain, en la cuenta de metadatos. Suena como algo que se puede imponer, ¿no? Pasaste la lección anterior configurando el plugin Royalties de Core, maquinaria que el propio programa del activo revisa, aunque la entregaste con `ruleSet("None")` y escuchaste exactamente qué deja eso sin imposición, así que la afirmación parece plausible por asociación. No lo es. Y en vez de creerme a mí, ve a tocar la evidencia, la cuenta de la que pende toda la historia de regalías del legado, el Metaplex Foundation Rule Set. Treinta segundos, nada que instalar, solo `curl` y el `python3` que ya tienes en tu sistema:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST -H "Content-Type: application/json" -d '
  {"jsonrpc":"2.0","id":1,"method":"getAccountInfo",
   "params":["eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9",{"encoding":"base64"}]}' \
  | python3 -c "import sys,json,base64; v=json.load(sys.stdin)['result']['value']; print('owner:', v['owner'], '| bytes:', len(base64.b64decode(v['data'][0])))"
```

Deberías ver `owner: auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg | bytes: 19001`. Ese owner es el programa Token Auth Rules, y esos 19,001 bytes guardan nueve revisiones del rule set que gobierna las transferencias de una porción enorme de los NFTs programables en mainnet. Desarmé esta cuenta esta mañana (2026-08-23), y la última revisión pone todas y cada una de las operaciones, las catorce, en `Pass`. Armada, pero inactiva. Para el final del lab lo vas a haber decodificado tú mismo, offsets de byte y todo.

## La máquina de regalías que no bloquea nada

### Legado

Primero, la declaración de estado, porque esta lección es donde el curso la hace y el resto del curso apunta aquí. Token Metadata es oficialmente legado; Metaplex Core es el estándar recomendado para trabajo nuevo con NFT (Bubblegum v2 para comprimidos). Los pNFTs imponen a través de Token Auth Rules, deprecado por Metaplex y aun así el camino de imposición vivo, y el rule set emblemático bloquea cero programas, armado pero inactivo; `seller_fee_basis_points` es puramente indicativo; las regalías de trabajo nuevo son plugins de Core o rulesets de Bubblegum v2. TM/pNFT se lee y se integra contra él, nunca se entrega nuevo.

Eso es mucho veredicto en un párrafo, así que ganémoslo pedazo por pedazo. La evidencia más limpia de que un estándar entró en modo de mantenimiento no es un anuncio, es el tren de releases. El cliente JavaScript de mpl-token-metadata se detuvo en v3.4.0, publicado el 2025-02-02, y npm todavía sirve 3.4.0 como `latest` hoy (revisé el registry esta mañana, 2026-08-23). Dieciocho meses, cero releases, en la biblioteca cliente del estándar que la mayoría de los NFTs de Solana realmente usa.

Sé preciso sobre la forma de eso, igual, porque "abandonado" es la palabra equivocada y la palabra equivocada te va a hacer decir algo falso en una reunión. El crate de Rust no está congelado: `mpl-token-metadata` cortó 5.1.1 el 2025-08-18 y tiene builds alpha desde entonces. Eso es exactamente cómo se ve el modo de mantenimiento desde afuera. El programa sigue compilando contra toolchains actuales, las correcciones entran cuando tienen que entrar, y nada nuevo se diseña. Compara al vecino: las tres líneas de release de `mpl-core`, las que leíste en la línea de tiempo del changelog de la lección anterior, seguían todas subiendo versiones minor hasta mediados de 2026, que es lo que hace un código base mientras su superficie de features todavía crece. Un código base recibe oxígeno. El otro recibe features. Observa dos bibliotecas cualesquiera durante dieciocho meses y la que está en modo de mantenimiento se identifica sola sin publicar nunca un post de blog al respecto.

![Línea de tiempo que compara la actividad de releases en la que el cliente JS de mpl-token-metadata se detuvo en v3.4.0 en febrero de 2025 mientras mpl-core siguió cortando releases hasta mediados de 2026.](assets/v01-timeline.png)

Dos notas prácticas antes de ir más profundo, y las dos te van a morder si te las saltas. Una: legado no significa raro. La mayoría de los NFTs ya acuñados en Solana viven en Token Metadata, así que un integrador se cruza con este estándar constantemente; por eso exactamente el curso lo enseña a profundidad de evaluación en vez de saltarlo. Dos: la documentación se mudó de casa. Los docs de Metaplex migraron de dominio, y el viejo subdominio developers ahora redirige permanentemente al nuevo hub de docs, así que los links en tutoriales más viejos y en respuestas de Stack Exchange pasan por un redirect o mueren de plano. Cuando verifiques cualquier cosa de abajo contra los docs, navega desde el hub actual en vez de confiar en un favorito de 2023.

### Qué es realmente un pNFT

El NFT programable existe por una pelea. Para entender la maquinaria, necesitas el mecanismo primero y la historia después, así que aquí está el mecanismo.

Un NFT Token Metadata común es una cuenta de token SPL más un PDA de metadatos. Nada impide que el dueño lo transfiera con una transferencia simple del token program, lo que significa que nada puede forzar a una transferencia a pasar por ninguna lógica de regalías. Un pNFT cierra ese agujero con un movimiento brutal: la cuenta de token está congelada en todo momento. No congelada como castigo, congelada como arquitectura. Una cuenta de token SPL congelada no puede moverse por la propia instrucción de transferencia del token program, punto. El único camino que funciona es la instrucción de transferencia de Token Metadata, que descongela, mueve y recongela dentro de un único flujo atómico, y que consulta un rule set antes de aceptar hacerlo.

Ese rule set vive en un programa aparte, Token Auth Rules, el owner `auth9Sig...` que viste en la apertura. Una cuenta de rule set mapea nombres de operación, `Transfer:Owner`, `Delegate:Sale`, y doce amigos, a reglas. Una regla puede ser un predicado real (una allow-list de programas, un compuesto de condiciones) o la regla trivial `Pass`, que aprueba todo. Cuando una transferencia de pNFT se ejecuta, Token Metadata resuelve el rule set configurado del activo, busca la operación que se está intentando, y evalúa la regla. Falla la regla, falla la transferencia. Esa es toda la pila de imposición: congela todo, encauza todo movimiento por una instrucción, deja que una cuenta de reglas decida.

![Diagrama de una transferencia de pNFT donde la cuenta de token congelada bloquea el camino simple, así que toda transferencia se encauza por la instrucción de Token Metadata y su evaluación de Token Auth Rules.](assets/v02-diagram.png)

Quédate un momento con lo que ese embudo te cuesta como integrador, porque este es el punto en el que un estándar abstracto se convierte en un bug en tu código. Una transferencia SPL simple quiere un origen, un destino, una autoridad y un mint. Una transferencia de pNFT quiere todo eso más la cuenta de metadatos, el master edition, un PDA de token record para el origen y otro para el destino, la cuenta del rule set, el propio programa Token Auth Rules, y el sysvar de instrucciones para que el motor de reglas pueda ver qué más viaja en la transacción. Olvídate de uno y no obtenés degradación elegante, obtenés una transacción fallida. Y si tu código nunca aprendió nada de esto, si solo llama al token program como lo hace para cualquier otro activo, muere en el congelamiento, que es la primera pared y la menos informativa. El error dice que la cuenta está congelada. Un dev que no sabe que los pNFTs existen va entonces a pasarse una tarde cazando a quién la congeló. Nadie la congeló. Nació así.

Ahora la paradoja que tienes que sostener sin titubear, porque hace tropezar a casi todo el mundo que lee los docs rápido: el propio hub de desarrolladores de Metaplex lista Token Auth Rules como deprecado, y Token Auth Rules sigue siendo el camino de imposición vivo para todos los pNFT que existen. Las dos afirmaciones son verdaderas a la vez. Deprecado significa "no construyas cosas nuevas sobre esto"; no significa que el programa se apagó. El programa está deployado, los rule sets resuelven, Token Metadata sigue llamándolo en cada transferencia de pNFT hoy. Si internalizás un hábito de este curso, que sea este: la deprecación es una recomendación sobre el futuro, no una afirmación sobre el presente. Lees la blockchain para aprender el presente.

### Por qué las regalías necesitaron toda esta maquinaria

Hora de derivar el diseño en vez de memorizarlo, porque la derivación es lo que te dice dónde se rompe. Arranca del statu quo, 2021: la cuenta de metadatos de un NFT lleva `seller_fee_basis_points`, un u16, donde 500 significa 5%. ¿Qué hace el runtime de Solana con ese número? Nada. No es un esquema de comisión, no es un parámetro de protocolo, no es nada que el camino de transferencia lea. Es una nota prendida al activo que dice "al creador le gustaría un 5%". Los marketplaces leyeron la nota y, por un tiempo, la honraron voluntariamente.

Así que la pregunta motivadora se escribe sola: si el campo es solo un pedido, ¿qué pasa cuando alguien rechaza el pedido? Exactamente lo que predecirías. Aparecieron marketplaces de regalía cero y de regalía opcional en 2022, rutearon los canjes alrededor de la comisión por construcción, y el volumen siguió al descuento. Los creadores vieron su línea de ingresos acercarse a cero en activos cuyos metadatos todavía prometían 5%, porque la promesa nunca fue estructural.

Recorre los intentos de arreglo por niveles, como el ecosistema los recorrió de verdad. Arreglo ingenuo uno: pedirles amablemente a los marketplaces, tal vez quitar colecciones de los agregadores que se saltan las regalías. La presión social funciona hasta que la economía la supera; no se sostuvo. Arreglo ingenuo dos: que el contrato del marketplace imponga la comisión. Pero el marketplace es la parte con el incentivo de saltearla, y un vendedor siempre puede usar otro contrato, o una transferencia simple de billetera a billetera disfrazada de venta. La imposición por parte de los dispuestos no es imposición. Lo que estrecha la pregunta a su forma real: las regalías solo se pueden imponer si el propio activo puede negarse a moverse excepto por programas que pagan. Y "el activo se niega a moverse" en Solana significa que la cuenta de token está congelada y algo con autoridad de descongelamiento media cada transferencia. Ese requisito estrechado fuerza esencialmente todo el diseño de pNFT que acabás de leer: congelamiento permanente, una instrucción de transferencia obligatoria, y un motor de reglas que decide qué llamadores son aceptables. El diseño no es barroco por diversión; es la forma mínima que satisface "el activo se niega".

![Diagrama de flujo que deriva el diseño del pNFT, donde los arreglos fallidos estrechan el problema hasta que el activo se niega a moverse, forzando la arquitectura de congelamiento más rule set y dejando abierto quién mantiene la lista.](assets/v03-flowchart.png)

Fíjate en la pregunta residual con la que termina el diagrama de flujo, porque es la bisagra de toda esta lección. El rule set es una lista de quién puede mover el activo. Alguien tiene que mantener esa lista, defenderla, actualizarla a medida que las plataformas aparecen y mueren, y absorber la política de excluir una plataforma. Para la mayoría de las colecciones de pNFT ese alguien es Metaplex, vía el Metaplex Foundation Rule Set compartido que sondeaste en la apertura. El mecanismo es sólido. La pregunta siempre fue si la lista se mantendría poblada. Ya sabes la respuesta, la viste en la cuenta, pero hagamos la lectura como se debe.

### Leyendo el rule set emblemático

Aquí está lo que esos 19,001 bytes son en realidad, recorridos campo por campo como los vas a decodificar en el lab. El byte 0 es un discriminador de cuenta, clave `1`, que significa RuleSet. Los bytes 1 a 8 son un u64 little-endian que guarda el offset en bytes del mapa de revisiones, que vive en la cola de la cuenta. Todo lo que está entre el header y el mapa es una pila de revisiones: un rule set es append-only, cada actualización empuja una revisión completa nueva, y el mapa del final registra dónde arranca cada revisión. Esta cuenta guarda nueve revisiones, y una nota de indexación antes de cualquier número: esta lección cuenta las revisiones desde cero, rev0 a rev8, así que la "revisión 8" ES la novena y última entrada, y una futura décima entrada sería la revisión 9. La última arranca con un único byte `lib_version` (aquí `1`, que significa que el cuerpo de la revisión está serializado en msgpack), seguido de una estructura de cuatro elementos: la versión otra vez, la pubkey del owner, el nombre legible por humanos, y el mapa de operaciones. El nombre en esta cuenta dice, literalmente, `Metaplex Foundation Rule Set`.

Y el mapa de operaciones, última revisión, decodificado en vivo al momento de escribir (2026-08-23):

![Tabla de las catorce operaciones de la última revisión del rule set emblemático, cinco operaciones Transfer y nueve operaciones Delegate, todas y cada una mapeadas a la regla Pass.](assets/v04-table.png)

Catorce operaciones. Catorce `Pass`. Una regla `Pass` aprueba a cualquier llamador sin condiciones, así que este rule set, evaluado en cada transferencia de cada pNFT que lo apunta, no bloquea nada. La maquinaria corre: la cuenta resuelve, el congelamiento aguanta, Token Metadata llama obedientemente a Token Auth Rules en cada transferencia, el motor de reglas evalúa, y la evaluación siempre tiene éxito. La imposición está armada pero por ahora inactiva. Esa frase es la que hay que llevarte de esta lección, porque las dos mitades importan para un integrador: armada significa que todavía tienes que manejar bien el camino de transferencia del pNFT o tus transferencias fallan de plano; inactiva significa que no le puedes decir a nadie que las regalías están garantizadas por ella.

### Armada, después inactiva: lo que las revisiones recuerdan

El diseño append-only tiene un regalo para nosotros: la cuenta recuerda su propia historia, y la historia es donde esto deja de ser abstracto. Decodifica las nueve revisiones (el challenge te hace exactamente esto) y aparece un arco limpio. Las primeras revisiones son árboles de reglas de verdad. Operaciones como `Transfer:SaleDelegate` llevan reglas compuestas, condiciones estructuradas con allow-lists de programas debajo en vez de una aprobación general, y los fallbacks de namespace rutean las operaciones no listadas a reglas base. Esta era la era de la imposición de regalías en carne y hueso: una lista mantenida de programas aprobados, con todo lo demás rechazado. Después la revisión 7 da vuelta casi todo el mapa a `Pass`, dejando una única operación con una regla real. La revisión 8 actual retira a ese último resistente. Catorce de catorce.

![Gráfico de las nueve revisiones del rule set, donde de cero a seis llevan quince o dieciséis reglas reales, la revisión siete baja a una, y la última lleva cero.](assets/v05-chart.png)

Un detalle que tu decodificador pasa por encima en silencio. El segundo elemento de cada revisión es el owner del rule set, la pubkey autorizada a empujar la revisión nueve. La desestructuración del lab lo tira con una coma vacía, que es un buen valor por defecto y un mal hábito. Decodificalo cuando estés auditando de verdad. Un rule set no es una constitución, es una cuenta con una autoridad de actualización, y saber quién tiene esa autoridad te dice exactamente qué tan durable es la postura de imposición que acabás de medir. Para el emblemático, esa autoridad es Metaplex. Para una colección que armó su propio rule set, podría ser un keypair en la laptop de alguien.

Agárrate de la honestidad de aquí, porque corta para los dos lados. Esto no es evidencia de que la imposición de regalías nunca funcionó; las primeras revisiones prueban que sí, estructuralmente, todo el tiempo que la lista se mantuvo. Es evidencia de que la imposición era una política expresada a través de una cuenta que una autoridad controla, y la política cambió. La propia documentación de Metaplex es refrescantemente directa sobre el estado actual: concede que este rule set hoy no niega ningún programa. Esa es toda la historia en una cláusula, porque un rule set que no le niega a nadie no puede hacer pagar a nadie. Todos citando basis points; un imponedor emblemático dejando pasar todo. Voy a admitir que esta me dolió en lo personal: escribí un script de acuñación en la era pNFT que logueaba `seller_fee_basis_points` bajo la etiqueta `royalty` como si la palabra fuera estructural, y nada en mi stack revisó nunca si algo la imponía. La mayoría del herramental del ecosistema todavía imprime ese campo igual que el mío.

Entonces, ¿qué es `seller_fee_basis_points`, dicho con exactitud? Un u16 en la cuenta de metadatos, denominado en basis points, que el programa Token Metadata almacena y sirve y nunca gasta. El runtime no lo descuenta. Token Metadata no lo descuenta. En la era de la imposición, el rule set tampoco lo descontaba; la imposición funcionaba negándoles la entrada a las plataformas que no pagaban regalías, no cobrando la comisión en sí, y pagar el monto siempre fue el código del marketplace honrando el campo. Hoy, con las reglas emblemáticas inactivas, el campo es exactamente lo que era en 2021: un pedido on-chain. Cuando ese marketplace hipotético promete un 5% garantizado porque el campo dice 500, la lectura correcta es: advertencia. Indicativo solamente, honrado a discreción de cada plataforma, impuesto por nada que puedas señalar. Si una contraparte quiere probar lo contrario, la carga está a una decodificación de distancia, y ahora la herramienta es tuya.

### El enum que creció más que su documentación

Un filo más, más chico pero muy la misma lección en miniatura. Cada cuenta de Token Metadata lleva un valor `TokenStandard` que les dice a los integradores qué tipo de activo tienen. La página de docs lista cinco variantes. El código fuente del programa define seis: `NonFungible`, `FungibleAsset`, `Fungible`, `NonFungibleEdition`, `ProgrammableNonFungible`, y la que la página de docs nunca menciona, `ProgrammableNonFungibleEdition`. Un pNFT puede tener ediciones, las ediciones de NFTs programables necesitan su propio valor de estándar, y el código hizo crecer la variante mientras la página de docs se quedó quieta.

![Comparación que muestra el enum TokenStandard, los docs listan cinco variantes mientras el código define seis, con ProgrammableNonFungibleEdition presente solo en el fuente.](assets/v06-comparison.png)

¿Contra cuál integrás? El código, sin dudar, y el razonamiento se generaliza a cada sistema legado que vayas a tocar. La documentación es un artefacto mantenido; modo de mantenimiento significa que deja de mantenerse al mismo reloj que el código, y el código es lo que se ejecuta contra tu transacción. Arma un match a partir de las cinco documentadas y la sexta variante llega a tu pipeline como un valor sin manejar en el peor momento posible, en producción, dentro de la transferencia de alguien. Esta es la misma epistemología que la paradoja deprecado-pero-vivo y la misma epistemología que el propio rule set: la blockchain y el fuente son el tiempo presente; la prosa sobre ellos es el pasado de cuando alguien la editó por última vez.

### La postura del integrador

Ahora arma el veredicto en una postura de trabajo, porque el trade-off de aquí es asimétrico y la asimetría es el punto. El Token Metadata legado sigue siendo el estándar de NFT más ampliamente integrado en Solana, así que tienes que poder leerlo e integrar contra él: a tu marketplace, billetera, índice o juego le van a entregar activos TM y pNFTs por años, y un pNFT mal manejado (digamos, intentar una transferencia simple de token contra una cuenta permanentemente congelada) falla fuerte. Pero entregar pNFTs nuevos significa adoptar una pila de imposición deprecada cuya regla emblemática hoy no impone nada, apostando tu producto a maquinaria de la que su propio proveedor se alejó. Y tratar `seller_fee_basis_points` como una comisión garantizada está simplemente mal, del modo que con el tiempo se convierte en un ticket de soporte, un creador enojado, o una cuestión legal. El costo de la compatibilidad más amplia es un estándar que Metaplex ya no recomienda. Así que la postura: leer, criticar, integrar, nunca entregar nuevo. Las regalías de tu trabajo nuevo viven donde las construiste la lección anterior, en el plugin Royalties de Core, y donde te lleva el próximo módulo para activos comprimidos, los rulesets de Bubblegum v2.

Y cuando un cliente te haga la pregunta directa, que la va a hacer, la respuesta honesta tiene tres partes y lleva alrededor de un minuto. La imposición on-chain de regalías para el Token Metadata legado no está pasando hoy, y puedes mostrarles la decodificación en vez de afirmarlo, lo que cambia toda la temperatura de la conversación. La imposición para trabajo nuevo está genuinamente disponible a través del plugin Royalties de Core, cuyas reglas de allow-list y deny-list son una revisión que corre el propio programa del activo, no una cuenta de reglas que alguien más tiene que seguir manteniendo. Y ningún estándar en ninguna cadena detiene a dos partes que quieren liquidar fuera de la plataforma. Lo que compra la imposición de regalías es fricción contra el camino casual, no una ley. Di eso en voz alta al principio y nadie vuelve enojado en seis meses.

![Árbol de decisión donde los activos Token Metadata y pNFT existentes se leen y se integran con un veredicto de imposición, mientras los drops nuevos llevan las regalías a plugins de Core o rulesets de Bubblegum v2.](assets/v07-flowchart.png)

## Lab: decodifica tú mismo el rule set emblemático

El `curl` de la apertura probó que la cuenta existe. Ahora construyes el decodificador que convierte esos bytes en un veredicto, la misma lectura que hice para la tabla de arriba. Esto es deliberadamente liviano en dependencias: una biblioteca msgpack, el `fetch` incorporado de Node, ningún SDK de Metaplex, porque el punto es que puedes auditar la historia de la imposición desde bytes crudos incluso si cada biblioteca cliente desaparece.

1. Arma un workspace. Necesitas Node (el piso del curso no cambió desde m01-l1: Node 20 o más nuevo, cualquiera con `fetch` incorporado) y exactamente un paquete:

   ```bash
   mkdir ruleset-audit && cd ruleset-audit
   npm init -y
   npm install @msgpack/msgpack
   ```

   Eso instala `@msgpack/msgpack` 3.1.3 al momento de escribir esto (2026-08-23); cualquier 3.x sirve, es una biblioteca de formato estable. Msgpack, si no la conocés, es una prima binaria compacta de JSON, y es con lo que serializa el formato de revisión V1 del rule set.

2. Escribe el decodificador. Crea `decode-ruleset.mjs`:

   ```javascript
   // decode-ruleset.mjs <rule-set-address> [rpc-url]
   // Reads a Token Auth Rules RuleSet account and prints the LATEST revision's
   // operation map, then renders an enforcement verdict.
   import { decode } from "@msgpack/msgpack";

   const address = process.argv[2] ?? "eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9";
   const rpc = process.argv[3] ?? "https://api.mainnet-beta.solana.com";

   const res = await fetch(rpc, {
     method: "POST",
     headers: { "Content-Type": "application/json" },
     body: JSON.stringify({
       jsonrpc: "2.0", id: 1, method: "getAccountInfo",
       params: [address, { encoding: "base64" }],
     }),
   });
   const { result } = await res.json();
   if (!result.value) throw new Error("account not found: " + address);

   const buf = Buffer.from(result.value.data[0], "base64");
   console.log("owner program:", result.value.owner);
   console.log("account size :", buf.length, "bytes");

   // Header: byte 0 is the account key (1 = RuleSet), bytes 1..9 are a u64
   // pointing at the revision map that lives at the END of the account.
   const revMapLoc = Number(buf.readBigUInt64LE(1));
   const revCount = buf.readUInt32LE(revMapLoc + 1);
   const offsets = [];
   for (let i = 0; i < revCount; i++) {
     offsets.push(Number(buf.readBigUInt64LE(revMapLoc + 5 + 8 * i)));
   }
   console.log("revisions    :", revCount, "(latest wins)");

   // Latest revision: 1 lib_version byte, then a msgpack-serialized RuleSetV1:
   // [lib_version, owner_pubkey_bytes, name, { operation -> rule }].
   const start = offsets[revCount - 1];
   const libVersion = buf[start];
   if (libVersion !== 1) throw new Error("not a msgpack (V1) revision: lib " + libVersion);
   const [, , name, operations] = decode(buf.subarray(start + 1, revMapLoc));

   console.log("rule set name:", name);
   const ruleKind = (rule) => (typeof rule === "string" ? rule : Object.keys(rule)[0]);
   let blocking = 0;
   for (const [op, rule] of Object.entries(operations)) {
     const kind = ruleKind(rule);
     if (kind !== "Pass") blocking++;
     console.log(`  ${op.padEnd(28)} ${kind}`);
   }
   console.log(
     blocking === 0
       ? `VERDICT: ${Object.keys(operations).length} operations, ALL Pass. Armed but idle: nothing is blocked, royalties are NOT enforced by this rule set.`
       : `VERDICT: ${blocking} operation(s) carry real rules. Enforcement is live for those paths.`
   );
   ```

   Lee la estrofa del medio antes de correrlo, porque los offsets son la verdadera clase de anatomía. El header apunta hacia adelante al mapa, el mapa apunta hacia atrás a cada revisión, y el decodificador no confía en nada más: ninguna IDL, ninguna biblioteca cliente, solo el layout. El helper `ruleKind` cubre las dos formas que toma una regla en el msgpack decodificado: las reglas triviales llegan como strings simples (`"Pass"`) y cada regla estructurada llega como un objeto de clave única cuya clave nombra el tipo de regla.

3. Córrelo contra el emblemático:

   ```bash
   node decode-ruleset.mjs
   ```

   Salida esperada, y esto es literalmente lo que la cuenta devolvió el 2026-08-23:

   ```text
   owner program: auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg
   account size : 19001 bytes
   revisions    : 9 (latest wins)
   rule set name: Metaplex Foundation Rule Set
     Transfer:WalletToWallet      Pass
     Transfer:Owner               Pass
     Transfer:MigrationDelegate   Pass
     Transfer:SaleDelegate        Pass
     Transfer:TransferDelegate    Pass
     Delegate:LockedTransfer      Pass
     Delegate:Update              Pass
     Delegate:Transfer            Pass
     Delegate:Utility             Pass
     Delegate:Staking             Pass
     Delegate:Authority           Pass
     Delegate:Collection          Pass
     Delegate:Use                 Pass
     Delegate:Sale                Pass
   VERDICT: 14 operations, ALL Pass. Armed but idle: nothing is blocked, royalties are NOT enforced by this rule set.
   ```

   Si tu conteo de revisiones o una regla difiere del mío, no asumas que rompiste algo: esta es una cuenta en vivo con una autoridad de actualización, y puede haber entrado una revisión nueva entre mi decodificación y la tuya. Esa posibilidad es la lección, no una nota al pie de ella. La postura de imposición de una porción enorme del ecosistema pNFT (nadie publica un conteo exacto, y este curso no va a inventar uno) está a una transacción de cambiar, en cualquier dirección, y la única respuesta actual es la que acabás de traer.

4. Ahora emite el veredicto de evaluación en tus propias palabras, en voz alta o en un archivo borrador, en la forma que este módulo califica: impuesto o no, con la evidencia del conjunto de operaciones, más la razón por la que no se puede confiar en el campo de basis points. El mío dice: "No impuesto. El rule set del activo mapea las catorce operaciones gobernadas a Pass, así que cada camino de transferencia se aprueba sin condiciones; `seller_fee_basis_points` es un pedido almacenado que ningún programa del camino de transferencia lee ni cobra, así que no puede funcionar como una comisión." Si tu versión nombra la evidencia y etiqueta el campo, tienes la habilidad que esta lección existe para instalar.

5. Checkpoint. Deberías tener ahora: un `decode-ruleset.mjs` que funciona y toma cualquier dirección de rule set, la decodificación del emblemático en tu propia terminal, y un veredicto escrito con evidencia. El decodificador es una herramienta de verdad, no una demo; guárdalo en el workspace de tu curso, porque el challenge lo apunta a la historia a continuación, y tú, en el futuro, lo vas a apuntar a cualquier colección de pNFT que un cliente te entregue.

## Challenge

El lab decodificó el presente. El challenge decodifica el pasado, y después hace la llamada hacia la que el brief viene construyendo. Primero, extiende `decode-ruleset.mjs` para recorrer las nueve revisiones en vez de solo la última: ya recolectás cada offset, así que corta cada revisión desde su offset hasta el siguiente (la revisión final termina donde arranca el mapa de revisiones), decodifica cada una, e imprime una línea por revisión con su conteo de operaciones que no son `Pass`. Revisa el byte `lib_version` por revisión antes de decodificar y salta, con una nota, cualquier revisión que no sea versión 1; esta cuenta es toda msgpack hoy, pero tu herramienta no debería asumir que cada rule set lo es. Tu salida debería reproducir el arco del gráfico: árboles de reglas reales hasta la revisión 6, una única regla viva en la revisión 7, cero en la revisión 8. Mientras estés ahí, expande la regla `Transfer:SaleDelegate` de una revisión temprana y mira cómo se veía la imposición estructuralmente de verdad, un árbol de reglas compuesto donde ahora se sienta un `Pass` general.

![Layout de bytes de una cuenta RuleSet donde el header apunta a un mapa de revisiones en la cola que apunta de vuelta a nueve revisiones apiladas, cada una arrancando con un byte lib_version.](assets/v08-diagram.png)

Después el memo de veredicto, tres oraciones, el entregable que un integrador realmente le pasaría a un equipo. Oración uno: si este rule set hoy impone regalías, con la evidencia de las operaciones. Oración dos: qué muestra el historial de revisiones que hacía antes, y qué implica eso sobre confiar en el estado actual de cualquier rule set. Oración tres: para un activo pNFT legado con el que tu producto podría cruzarse (elige cualquier colección de pNFT que conozcas, o razona nomás desde el emblemático, ya que una porción enorme apunta aquí), la llamada de estándares: leer-e-integrar, y bajo qué condiciones entregarías alguna vez algo nuevo sobre esta pila. Si tu tercera oración encontró una condición para entregar pNFTs nuevos, relee la lección; la respuesta honesta es que no hay ninguna, y decirlo con evidencia es el entregable.

Si tu decodificación del emblemático no coincide con los números impresos en esta lección, una décima revisión, una operación que ya no es `Pass`, un nombre de rule set que cambió, márcalo en el canal de feedback del curso con tu salida cruda y la fecha. Mis decodificaciones están estampadas el 2026-08-23 y esta es una cuenta en vivo bajo una autoridad activa; un aprendiz que la detecta derivando está haciendo exactamente el trabajo de leer-la-blockchain que esta lección enseña, y el curso se va a actualizar a partir de tu evidencia.

Ya puedes leer el estándar de NFT más ampliamente deployado en la cadena, juzgar su imposición con honestidad, y entregar un activo Core cuya maquinaria de regalías vive en un plugin que revisa el propio programa del activo, armado con `ruleSet("None")` por ahora, exactamente como admitió la lección anterior. ¿Pero qué pasa con un millón de crates Harvest? Un millón de activos Core es un millón de cuentas, y a la aritmética del rent no le importa tu roadmap. Próximo módulo: compresión de estado, Bubblegum v2, y leer activos de vuelta cuando casi nada está almacenado on-chain.
