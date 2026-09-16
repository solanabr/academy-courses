# Bonding curves y graduación: pump.fun a partir de cuatro constantes

## Resumen

El módulo 7 cerró con la compresión como concepto, una prueba de validez de 128 bytes y una cuenta de 5,000 lamports, y prometió que pronto pondrías a trabajar un token comprimido. Antes de eso, una decisión que no puedes postergar: cómo entra SPROUT al mundo. Tienes el mint de R3 y tienes el informe de enrutabilidad de R6, que te dice exactamente qué extensiones mantienen a SPROUT negociable y cuáles hacen que lo rechacen en la puerta. Esta lección toma el número más repetido de la cultura de lanzamientos en Solana, los 85 SOL que "gradúan" una moneda de pump.fun, y se niega a repetirlo. En cambio lo vas a derivar, a partir de las constantes publicadas de pump y un invariante (tres de las cuatro constantes hacen la derivación; la cuarta, total supply, solo le pone precio a un sobrante en una nota aparte), y después vas a construir la config de lanzamiento que lo calcula en vivo y elige la plataforma de graduación de SPROUT preguntando si esa plataforma puede siquiera sostener el token que construiste. El repliegue es pronunciado aquí: la derivación se trabaja completa en la sección de teoría, el lab te hace escribir la línea del invariante en papel antes de mostrarte el listado, y el challenge te entrega `graduationSol` con nada más que las constantes y un archivo de test. Al final vas a tener una herramienta que recalcula un umbral para cualquier curva, lo cual importa más que el número, porque el número le pertenece al programa de otra persona y puede cambiar un martes.

Empieza por la respuesta, en una línea. Abre una terminal en cualquier lugar donde haya Node:

```bash
node -e 'const vs=30,vt=1073e6,rt=793.1e6;console.log((vs*vt/(vt-rt)-vs).toFixed(3),"SOL")'
```

```
85.005 SOL
```

Tres números adentro, la constante del folclore afuera. `vs` es la reserva virtual de SOL de la curva, `vt` su reserva virtual de token, `rt` los tokens reales que te va a vender. Nada en esa línea lee una blockchain, y nada en ella contiene un 85. El resto de esta lección trata de por qué esas tres entradas y esa única expresión son toda la historia, qué está haciendo en realidad cada una de ellas, y qué quiere decir para tu token que la historia le pertenezca a un programa que no controlas.

## De dónde salen en realidad los 85 SOL

### El número que nadie guarda

Cada hilo de lanzamiento, cada video explicativo, cada hilo de "cómo funciona pump" repite la misma forma: tu moneda se negocia sobre una curva hasta que entran unos 85 SOL de presión compradora, y después se gradúa a un AMM de verdad. Dicho así, 85 suena como un umbral que vive en algún campo, revisado por el programa en cada compra.

Así que revísalo. La teoría más ingenua es que la graduación es un parámetro guardado, y refutarla toma unos treinta segundos desde el propio IDL de pump en vez de desde el post de blog de cualquiera:

```bash
npm pack @pump-fun/pump-sdk@1.36.0
tar xzf pump-fun-pump-sdk-1.36.0.tgz
node -e '
const idl = require("./package/src/idl/pump.json");
console.log("program:", idl.address);
const curve = idl.types.find(t => t.name === "BondingCurve");
console.log("BondingCurve fields:", curve.type.fields.map(f => f.name).join(", "));
'
```

```
program: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
BondingCurve fields: virtual_token_reserves, virtual_quote_reserves, real_token_reserves, real_quote_reserves, token_total_supply, complete, creator, is_mayhem_mode, is_cashback_coin, quote_mint
```

Diez campos. Cuatro reservas, un supply, un booleano, un creador, dos flags de modo más nuevas y un quote mint, y ningún precio de graduación en ninguna parte de ahí, ningún umbral, ninguna capitalización de mercado objetivo, lo que quiere decir que la cuenta que gobierna toda la vida pre-AMM de tu moneda no tiene idea de que 85 sea un número que le importe a nadie. Ese pin de versión merece una nota, porque lo leí el 2026-08-22 cuando el latest de npm era 1.36.0, y el SDK se mueve lo bastante rápido como para que la lista de campos que imprimas sea más larga que la mía. Si lo es, el argumento sobrevive: lo que estás buscando es un umbral guardado, y su ausencia es justo el punto.

De esa lectura salieron otras dos cosas que van a importar después. `virtual_quote_reserves` antes era `virtual_sol_reserves`, renombrado cuando pump empezó a soportar quote mints distintos de SOL, así que los artículos viejos y el IDL actual no coinciden en el nombre mientras se refieren al mismo slot. Y el program id es `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`, que es lo que hay que grepear cuando quieres saber si una transacción que estás mirando tocó pump siquiera.

Si el umbral no está guardado, tiene que estar implícito. ¿En qué?

### Las reservas virtuales son un esquema de precios disfrazado de saldo

Cuatro constantes definen una curva de pump al nacer, y las cuatro viven en la cuenta `Global` del programa, no en su fuente: `initial_virtual_token_reserves`, `initial_virtual_sol_reserves`, `initial_real_token_reserves` y `token_total_supply`. Para la configuración de referencia esas son 1,073,000,000,000,000 unidades base de token virtual, 30,000,000,000 lamports de SOL virtual, 793,100,000,000,000 unidades base de token real, y un total supply de 1,000,000,000 tokens a 6 decimales.

Convierte a tokens enteros y los números se vuelven más amables: 1.073 mil millones de tokens virtuales, 30 SOL virtuales, 793.1 millones de tokens reales, 1 mil millones de supply.

Clava la vista en la primera y en la última un segundo. La curva declara 1.073 mil millones de tokens en reserva. El mint solo llega a crear 1 mil millones. Una reserva que sostiene más que todo el supply no es un saldo, y esa es la señal: las reservas virtuales no son custodia, son los dos números que una fórmula de precios necesita. Una reserva real es lo que el programa de verdad te va a entregar, mientras que una reserva virtual es solo el lugar donde el programa finge pararse en la curva de precios, y la brecha entre las dos es una decisión de diseño más que un accidente, la decisión que fija tu precio de apertura y por lo tanto toda la forma del viaje que sigue.

![La reserva virtual de token de 1.073 mil millones se extiende más allá de la línea de supply de 1 mil millones mientras que la reserva real de 793.1 millones queda dentro de ella, marcando las reservas virtuales como coordenadas de precio.](assets/v01-diagram.webp)

La regla de precios es la más vieja de los mercados on-chain. El producto de las dos reservas virtuales se mantiene constante en cada canje:

```
k = virtualSol x virtualToken
```

Compra tokens y la reserva virtual de token baja mientras la reserva virtual de SOL sube, en exactamente la proporción que mantiene `k` donde estaba. Ese es el invariante de producto constante, y vale la pena nombrarlo con precisión porque todo lo que sigue es una consecuencia de él. El precio spot en cualquier momento es apenas la razón entre las dos reservas, SOL por token. Al nacer eso es 30 dividido por 1.073 mil millones, o unos 2.796e-8 SOL por token. Barato a propósito. Se supone que el primer comprador se sienta temprano.

Si quieres escribir un swap de producto constante tú mismo en vez de leer uno, el curso Master Anchor V2 construye uno de juguete como patrón de framework. Aquí solo necesitamos el invariante como hecho contable, no como un programa para escribir.

### El modelo de precios ingenuo, y exactamente qué tan equivocado está

Esta es la estimación que casi todo el mundo escribe primero, yo incluido la primera vez que intenté revisar la afirmación de los 85. Conoces el precio de apertura. Conoces cuántos tokens reales va a vender la curva. Multiplica:

```
793,100,000 tokens x 2.7959e-8 SOL/token = 22.174 SOL
```

Eso son 22 SOL, no 85. La brecha no es un artefacto de redondeo ni una comisión que falta, es un factor de 3.8, y la tentación en ese momento es salir a cazar los 63 SOL que faltan en algún lugar del esquema de comisión de pump. No existe tal comisión. 100 basis points planos sobre 85 SOL son menos de un solo SOL.

La estimación está mal por una razón estructural: le pone a cada token el precio del primero. La curva se empina a medida que se drena. Cada token que compras encarece el siguiente, porque la reserva de token bajó y la reserva de SOL subió y `k` se niega a moverse. Ponerle precio a 793 millones de tokens a la tarifa de apertura es como ponerle precio a todo un tramo de escaleras a la altura del primer escalón.

El arreglo apenas menos ingenuo es promediar el primer y el último precio, que al menos admite que la curva se mueve. Eso también falla, de forma más sutil: una curva de producto constante no es lineal, así que la media aritmética de los extremos no es el precio medio pagado. Estarías integrando una hipérbola con un trapecio, y en una curva cuyo precio sube casi quince veces de punta a punta, el error es lo bastante grande como para importarle a cualquiera que esté dimensionando un lanzamiento.

Hay un tercer arreglo ingenuo que también merece morir, porque es el que buscan las personas con experiencia: busca la capitalización de mercado a la que se observa que las monedas se gradúan, y saca el SOL de ahí. Da más o menos la respuesta correcta, que es lo que lo vuelve peligroso. Es una medición de una población, no una propiedad del mecanismo, así que absorbe en silencio cualquier era de comisiones, quote mint y configuración de curva bajo la que las monedas muestreadas hayan salido a lanzarse. Cambia cualquiera de esas y tu número queda viejo sin ningún mensaje de error. La observación no te puede decir por qué, y solo el por qué sobrevive a un cambio de config.

Así que la pregunta de verdad es más angosta que "cuál es el costo total." Es esta: ¿en qué estado está la curva en el momento exacto en que deja de vender, y qué reserva de SOL exige el invariante para ese estado? Responde eso y el total sale solo, sin ninguna integración.

### Drenar la reserva, y la única línea que hace la integración por ti

La graduación pasa cuando la reserva real de token llega a cero, que es decir que cada uno de esos 793.1 millones de tokens reales se vendió y no queda nada que la curva le pueda entregar a nadie, así que da vuelta un booleano y se detiene.

Ahora traduce "la reserva real está vacía" al idioma de las reservas virtuales, porque ese es el idioma que habla el invariante. Cada token real que sale de la curva también sale de la reserva virtual de token, ya que se mueven juntos en cada compra. Drena toda la reserva real y la reserva virtual de token cayó exactamente en `realTokenReserves`:

```
finalVirtualToken = virtualToken - realToken
                  = 1,073,000,000 - 793,100,000
                  = 279,900,000
```

El invariante fue verdadero todo el tiempo y sigue siendo verdadero en ese instante, así que la reserva virtual final de SOL queda forzada:

```
finalVirtualSol = k / finalVirtualToken
                = (30 x 1,073,000,000) / 279,900,000
                = 32,190,000,000 / 279,900,000
                = 115.005 SOL
```

La curva arrancó con 30 SOL virtuales y termina sosteniendo 115.005. La diferencia es el SOL que tuvo que entrar de los compradores:

```
graduationSol = 115.005 - 30 = 85.005 SOL
```

Ahí está. El número que la gente repite como una ley del universo es la aritmética de `30 x 1073 / (1073 - 793.1) - 30`, y no está guardado en ninguna parte porque no necesita estarlo. Está implícito en tres de las cuatro constantes publicadas, SOL virtual, token virtual y token real, del mismo modo que la cuota de una hipoteca está implícita en una tasa y un plazo; la cuarta constante, total supply, nunca entra en esta aritmética y solo importa para la nota sobre el sobrante de más abajo.

![Una curva de precios de producto constante que sube 14.7 veces de la apertura a la graduación, con el área verdadera debajo de ella marcada 85.005 SOL contra un rectángulo de precio plano mucho más chico marcado 22.174 SOL.](assets/v02-chart.webp)

Vale la pena llevarse dos consecuencias de esta sección, porque son lo que vuelve la derivación una herramienta y no un truco.

La primera: el umbral se mueve cuando se mueven las constantes. Toma una curva con 30 SOL virtuales, 1 mil millones de tokens virtuales y 800 millones de tokens reales. Entonces la reserva virtual final de token es 200 millones, la reserva virtual final de SOL es 30 mil millones sobre 200 millones, o 150, y el umbral es 120 SOL. Misma fórmula, curva distinta, una vara de graduación 41% más alta, y sin necesidad de folclore. Una reserva real más grande en relación con la virtual quiere decir que estás drenando más arriba en la parte que se empina de la curva, y el SOL requerido sube en consecuencia.

La segunda: el precio final también es una derivación. En la graduación el precio spot es 115.005 dividido por 279.9 millones, unos 4.109e-7 SOL por token, que es 14.7 veces el precio de apertura. Ese múltiplo no es una cifra de marketing, es `(1073 / 279.9)` al cuadrado, y te dice la forma de todo el viaje pre-AMM en un solo número. Quien compre en la cima de la curva paga más o menos quince veces lo que pagó el primer comprador, antes de que cualquier cosa se negocie en un AMM siquiera.

Y una pieza de aritmética que sorprende a la gente: 1 mil millones de supply menos 793.1 millones de reserva real deja 206.9 millones de tokens, alrededor del 20.7% del supply, que la curva nunca le ofrece a nadie. Ese sobrante es lo que se lleva al pool en la migración junto con el SOL recaudado. Yo verificaría eso contra una transacción de migración real antes de repetirlo en un pitch, y este curso preferiría que lo revises antes que confiar en él, pero la aritmética es la aritmética y explica de dónde sale la profundidad inicial del pool de una moneda graduada.

### La flag complete, y quién tiene permitido apretar el botón

El programa pone `complete = true` cuando `real_token_reserves` llega a cero, y desde ese instante las compras y las ventas contra la curva fallan en vez de operar a algún precio final. La propia tabla de errores de pump nombra los dos lados de la cerca, y leer códigos de error es una forma rápida de aprender la máquina de estados de un programa:

```
6005 BondingCurveComplete     "The bonding curve has completed and liquidity migrated to raydium."
6006 BondingCurveNotComplete  "The bonding curve has not completed."
```

Ese primer mensaje es un fósil, por cierto. Todavía dice raydium, de la era anterior a que pump corriera su propio AMM, mientras que la instrucción `migrate` en el mismo IDL le entrega la liquidez a `pump_amm`. Las cadenas de error envejecen mal en toda base de código; trátalas como historia, no como documentación.

Ahora la pregunta interesante. ¿Quién llama a `migrate`? Mira las cuentas y la respuesta es cualquiera:

```
migrate accounts (25): global, withdraw_authority (w), mint, bonding_curve (w),
  associated_bonding_curve (w), user (signer), system_program, token_program,
  pump_amm, pool (w), pool_authority (w), pool_authority_mint_account (w),
  pool_authority_wsol_account (w), amm_global_config, wsol_mint, lp_mint (w),
  user_pool_token_account (w), pool_base_token_account (w),
  pool_quote_token_account (w), token_2022_program, associated_token_program,
  pump_amm_event_authority, event_authority, program, rent
signers: user
args: []
```

Veinticinco cuentas, y exactamente una de ellas firma. El único signer es `user`, y no hay ninguna restricción que ate `user` al creador. Ningún argumento. La migración es sin permiso, y es idempotente: la liquidez se mueve al AMM de PumpSwap en `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`, los tokens LP se queman, y un segundo llamador que le corra una carrera al primero no puede migrar dos veces ni drenar nada. La migración en sí lleva un `pool_migration_fee` de 15,000,001 lamports, un número extrañamente preciso que es él mismo un campo `Global` en vez de una constante, lo que quiere decir que está a una transacción de autoridad de ser otra cosa.

Sin-permiso-más-idempotente es el diseño correcto aquí y vale la pena entenderlo como un patrón, no solo como trivia. Un paso que cualquiera podría correr a disparar, en un momento impredecible, no puede depender de que una parte específica esté despierta. Los bots vigilan la flag y disparan migraciones gratis. Si el paso estuviera restringido a una autoridad en cambio, una moneda graduada cuyo creador se desconectó quedaría con liquidez muerta hasta que el creador volviera. Si fuera sin permiso pero no idempotente, la carrera misma sería el exploit. Quieres las dos propiedades o ninguna.

![Un flujo vertical de cinco etapas desde la creación de la curva pasando por el umbral derivado de 85.005 SOL hasta la flag complete y un migrate sin permiso e idempotente que quema el LP en PumpSwap.](assets/v03-flowchart.webp)

### La curva es una política, y la política cambió debajo de todos

Todo hasta aquí trata las cuatro constantes como física. Son política, fijada por una autoridad, y la demostración más clara de eso es lo que les pasó a las comisiones de pump.

Durante la mayor parte de la vida de pump la comisión de la bonding curve fue 100 basis points planos. Uno por ciento, igual para una moneda que vale cuatrocientos dólares y una moneda que vale cuatro millones. Después, el 2025-09-01 a las 20:00 UTC, eso dejó de ser verdad. Las comisiones se volvieron dinámicas, escaladas por la capitalización de mercado de la moneda, y la forma del cambio se ve en los propios tipos del SDK:

```
FeeConfig { bump: u8, admin: pubkey, flat_fees: Fees,
            fee_tiers: Vec<FeeTier>, stable_fee_tiers: Vec<FeeTier> }
FeeTier   { market_cap_lamports_threshold: u128, fees: Fees }
Fees      { lp_fee_bps: u64, protocol_fee_bps: u64, creator_fee_bps: u64 }
```

Lee esa estructura con cuidado, porque dice más de lo que dijo el anuncio. La comisión ya no es un número, son tres: una parte para el proveedor de liquidez, una parte para el protocolo y una parte para el creador. Y el tier que aplica se selecciona comparando la capitalización de mercado de la curva contra una lista de umbrales guardada en una cuenta `FeeConfig` en la blockchain. Dos listas de umbrales, de hecho, ya que las monedas cotizadas en un mint estable tienen su propio esquema de `stable_fee_tiers`. Las dos son `Vec`s, que es el detalle en el que vale la pena detenerse: no un array fijo de largo conocido, una lista sin límite cuyo largo es él mismo dato. El SDK ya viene con la función de selección. No viene con los números. Lo que quiere decir que cualquier tabla de tiers específica que leas en un post de blog, esta lección incluida, es un snapshot de una cuenta que un admin puede reescribir, y la única respuesta actual es la que sacas tú mismo.

Esto es lo que está en juego para ti, y no es abstracto. Si estás modelando un lanzamiento, la comisión que pagas a 10 SOL de capitalización de mercado y la comisión que pagas a 300 SOL pueden caer en tiers distintos, y una planilla construida sobre la era de los 100 bps planos va a poner mal los dos precios. Peor aún, los va a poner mal en una dirección que no puedes predecir desde afuera, porque los límites de los tiers son dato.

![Un diagrama de flujo que rastrea la comisión de un canje desde la derivación de la capitalización de mercado pasando por la selección de tier en el FeeConfig editable hasta una división en tres partes enrutada a ocho destinatarios rotativos.](assets/v04-flowchart.webp)

Ese mismo día del cambio trajo las monedas Cashback, donde las comisiones del creador se enrutan de vuelta a los traders en vez de al creador, contabilizadas a través de PDAs acumuladoras de volumen por usuario que la instrucción de compra toca en cada canje. Ese es un objeto económico genuinamente distinto detrás de la misma interfaz: matemática de curva idéntica, incentivo opuesto para quien la esté negociando. Y la recaudación de comisiones misma rota entre ocho direcciones de destinatario, un `fee_recipient` más un array `fee_recipients` de siete entradas, lo que es un detalle operativo hasta el día en que estés indexando flujos de comisiones y te preguntes por qué se dispersan.

![Una línea de tiempo que marca 2025-09-01 20:00 UTC, con una comisión plana de 100 basis point antes y comisiones por tier de capitalización de mercado después, sobre una banda que señala que el invariante no cambió.](assets/v05-timeline.webp)

Así que nombra el canje con honestidad, porque esta es la parte de la que de verdad depende una decisión de lanzamiento. Una bonding curve te compra descubrimiento de precio instantáneo y sin permiso, sin contraparte con la que negociar, y un pool garantizado al final con el LP quemado para que nadie pueda sacarlo. Lo que pagas es la pérdida total de control sobre la política económica. La forma de la curva la fijan constantes que no pones tú, el esquema de comisión es una cuenta que otra persona puede editar, la plataforma de graduación la elige el programa, y la abrumadora mayoría de las monedas lanzadas así nunca llega al umbral. He visto que se citan tasas de graduación de un solo dígito, seguido alrededor del uno o el dos por ciento, y yo no armaría un plan sobre ninguna cifra que no hubiera medido yo mismo en una ventana que elegí yo, porque ese número se mueve con cada ciclo de mercado. La dirección no está en duda, eso sí: la mayoría de las curvas se estanca, y las que se estancan no son un bug en el mecanismo. Son el mecanismo funcionando, ordenando la demanda.

Ese es el trato. Es un buen trato para una moneda cuya tesis entera es "que el mercado decida, ya, sin ningún guardián," y un trato terrible para un token que tiene opiniones sobre cómo debería comportarse, que son la mayoría de los tokens que existen por una razón distinta de negociar.

Antes de la pregunta de la plataforma, un checkpoint, porque la derivación tuvo varias partes móviles y la sección siguiente las gasta todas de golpe. A dónde llegamos: 1) la curva pone precio con `k = virtualSol x virtualToken`, mantenido constante en cada canje; 2) las reservas virtuales son coordenadas de precio, las reservas reales son inventario, y solo la real puede llegar a cero; 3) drenar la reserva real baja la reserva virtual de token exactamente en ese monto, lo que fuerza la reserva virtual final de SOL a `k / (virtualToken - realToken)`; 4) el SOL que tuvo que llegar es esa reserva final menos la inicial, 85.005 para las constantes de referencia; 5) ninguno de esos cuatro números es una ley, los cuatro son campos `Global`, y el esquema de comisión que se apoya encima de ellos es una cuenta aparte con su propio admin.

Eso te da una herramienta portátil, así que vuélvela portátil en voz alta. Cuando te encuentres con cualquier curva, en cualquier launchpad, hazle tres preguntas. ¿Cuáles son sus cuatro constantes, y dónde viven, en código o en una cuenta que alguien puede editar? ¿Qué condición termina la curva, y esa condición está sobre un saldo real o sobre uno implícito? ¿Y quién tiene permitido disparar la transición, con qué comisión pegada? Responde esas tres y puedes ponerle precio a cualquier bonding curve que te encuentres en una tarde, incluidas las que todavía no se construyeron. No las hagas y estás de vuelta repitiendo un número que leíste en alguna parte, que es donde empezó esta lección.

![Una tabla de cuatro filas que ordena los números de protocolo en derivados, guardados, fijados en código y meramente repetidos, con los 85 SOL del folclore archivados bajo repetidos.](assets/v06-table.webp)

Lo que nos trae a SPROUT.

### La plataforma veta antes de que la matemática llegue a importar

SPROUT tiene opiniones. De R6 conoces su conjunto final de plataforma de lanzamiento: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`. Tres extensiones, todas en la allowlist de Raydium CP-Swap, elegidas precisamente para que el token siga negociable, y es un mint Token-2022, que no es un detalle que puedas canjear después porque la comisión de transferencia que financia la tesorería solo existe en Token-2022 para empezar.

Ahora pregúntale al IDL de pump si puede tomar ese mint:

```bash
node -e '
const idl = require("./package/src/idl/pump.json");
const create = idl.instructions.find(i => i.name === "create");
console.log("create.token_program:", create.accounts.find(a => a.name === "token_program").address);
'
```

```
create.token_program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
```

Ese es el programa SPL Token clásico, fijado como una dirección fija en la cuenta, lo que quiere decir que a la instrucción `create` no se le puede entregar un mint Token-2022 de ninguna manera. Y va más allá de un desajuste de program id: pump crea el mint él mismo. No le llevas un token a pump, le pides a pump que haga uno, y el conjunto de extensiones de la cosa que hace es la elección de pump y no la tuya. No hay asiento en esa mesa para un token con una comisión de transferencia que configuraste tú.

Ese es un veto de plataforma, y llega antes que cualquier parte de la matemática. Puedes derivar el umbral de pump a la perfección y seguir sin poder usar pump, que es exactamente el tipo de cosa que es obvia en retrospectiva y cara por adelantado. He visto a un equipo elegir un launchpad desde una landing page, construir tres semanas de tokenomics sobre su división de comisiones, y descubrir el pin del token program durante la integración. La derivación se transfiere a cualquier curva que te encuentres. La plataforma no.

Así que la config de lanzamiento que estás por construir tiene dos trabajos, y el segundo es el que te salva la semana: derivar el umbral a partir de las constantes que publique cualquier plataforma, y rechazar cualquier plataforma cuyo camino de lanzamiento no pueda representar el token que ya construiste.

![Una comparación de dos columnas que muestra a pump.fun rechazando SPROUT porque su instrucción create fija el programa SPL Token clásico, contra Raydium CP-Swap aceptando las tres extensiones de SPROUT desde su allowlist de cinco entradas.](assets/v07-comparison.webp)

## Lab: deriva el umbral de SPROUT y fija su plataforma

El artefacto es `sprout-launch/derive-graduation.ts`, y su criterio es la forma de siempre del curso: `npx tsx sprout-launch/derive-graduation.ts` imprime el umbral derivado en unos 85.005 SOL, las reservas virtuales finales, el objetivo de migrate, y un veredicto de plataforma por candidato, saliendo con código distinto de cero si ningún candidato puede sostener SPROUT o si las constantes de referencia dejan de derivar a 85.

1. Haz la carpeta y fija el runner. Una sola herramienta de dev hace todas las corridas aquí, `tsx`, el mismo pin que el lab de R6 (ese lab también sostenía `typescript@5.9.3` para el typechecking del editor; nada en ESTE lab lo invoca, así que instálalo solo si quieres el soporte del editor). Los pasos 2 al 5 trabajan dentro de `sprout-launch/`; el paso 6 corre desde su padre, y el paso lo dice cuando llegues ahí:

```bash
mkdir -p sprout-launch && cd sprout-launch
npx --yes tsx@4.23.12 --version
```

   El pin lleva la misma nota de actualidad que llevó el informe, re-revisada el 2026-08-22: `tsx@4.23.12` era el latest de npm en esa lectura. Corre `npm view tsx version` tú mismo el día que hagas el scaffold. Un pin que copiaste de una lección sin revisar es un pin que vas a debuggear después.

2. Verifica los dos hechos de los que depende la config, desde el IDL de primera mano y no desde esta página. Corriste los dos comandos en la sección de teoría; córrelos otra vez aquí para que las salidas queden en la carpeta en la que estás por construir:

```bash
npm pack @pump-fun/pump-sdk@1.36.0 && tar xzf pump-fun-pump-sdk-1.36.0.tgz
node -e '
const idl = require("./package/src/idl/pump.json");
console.log("program:", idl.address);
console.log("create.token_program:",
  idl.instructions.find(i => i.name === "create")
     .accounts.find(a => a.name === "token_program").address);
console.log("BondingCurve fields:",
  idl.types.find(t => t.name === "BondingCurve").type.fields.map(f => f.name).join(", "));
'
```

   Quieres tres cosas en pantalla antes de escribir código: el program id, el programa de token SPL-clásico fijado en `create`, y una lista de campos de `BondingCurve` sin ningún umbral adentro. Si tu versión del SDK imprime una lista de campos más larga que la mía, anota la versión que leíste y sigue. Si imprime un campo de umbral de graduación, detente y avísale al curso, porque eso querría decir que el programa cambió de forma y la afirmación central de esta lección necesita una actualización.

3. Arranca el módulo con las constantes y el invariante. `finalReserves` es donde vive toda la lección, y su única expresión estructural es algo que derivaste dos secciones atrás. Así que antes de bajar al listado, escribe esa expresión en papel: el SOL virtual final al completarse, en términos de k y la reserva de token encogida. Después lee el listado y revísate contra su última línea:

```typescript
// derive-graduation.ts: SPROUT's launch curve (R10).
// Derives a bonding curve's graduation threshold from the constant-product
// invariant, then checks which graduation venue SPROUT's R6 extension set allows.

/** A bonding curve's published constants. Token amounts in WHOLE tokens. */
export interface CurveConstants {
  /** SOL the curve pretends to hold at t=0. Never a real balance. */
  virtualSolReserves: number;
  /** Tokens the curve pretends to hold at t=0. Never a real balance. */
  virtualTokenReserves: number;
  /** Tokens actually available to buyers before the curve completes. */
  realTokenReserves: number;
}

/** pump.fun's Global-account constants, converted from base units at 6 decimals. */
export const PUMP_REFERENCE_CURVE: CurveConstants = {
  virtualSolReserves: 30, // 30_000_000_000 lamports
  virtualTokenReserves: 1_073_000_000, // 1_073_000_000_000_000 base units
  realTokenReserves: 793_100_000, // 793_100_000_000_000 base units
};

export interface FinalReserves {
  k: number;
  finalVirtualToken: number;
  finalVirtualSol: number;
}

/**
 * The curve at the moment realTokenReserves hits zero: k is unchanged, the
 * virtual token reserve has dropped by every real token sold.
 */
export function finalReserves(c: CurveConstants): FinalReserves {
  const k = c.virtualSolReserves * c.virtualTokenReserves;
  const finalVirtualToken = c.virtualTokenReserves - c.realTokenReserves;
  if (finalVirtualToken <= 0) {
    throw new Error(
      `realTokenReserves (${c.realTokenReserves}) must be smaller than virtualTokenReserves (${c.virtualTokenReserves})`,
    );
  }
  return { k, finalVirtualToken, finalVirtualSol: k / finalVirtualToken };
}
```

   La guarda no es decoración. Una curva configurada con una reserva real más grande que su reserva virtual de token no tiene ningún estado de graduación, y sin esa revisión devolverías en silencio un umbral negativo y lo imprimirías sin pestañear.

4. Agrega las dos cantidades derivadas. Las dos son de una sola línea encima de `finalReserves`, y mantenerlas separadas es lo que deja que el challenge y la lección siguiente las llamen de forma independiente:

```typescript
/** SOL that must enter the curve to drain the real token reserve. */
export function graduationSol(c: CurveConstants): number {
  return finalReserves(c).finalVirtualSol - c.virtualSolReserves;
}

/** Spot price in SOL per token at the current reserve ratio. */
export function spotPrice(virtualSol: number, virtualToken: number): number {
  return virtualSol / virtualToken;
}
```

   Fíjate en lo que hace `graduationSol` con una curva cuya reserva real es cero: la reserva virtual final de token es igual a la inicial, la reserva virtual final de SOL es igual a la inicial, y la respuesta es 0 SOL. Ningún caso especial, ninguna rama. Una curva que no tiene nada que vender ya se graduó, y la fórmula lo sabe.

5. Ahora los registros de plataforma, que son la mitad honesta de este artefacto. Cada uno lleva de dónde vinieron sus hechos, porque una tabla de plataformas sin procedencia es exactamente el folclore contra el que argumenta esta lección:

```typescript
export interface GraduationVenue {
  name: string;
  /** Where liquidity lands after migration, when we have a first-party id for it. */
  ammProgramId?: string;
  /** The token program the venue's launch path can represent. */
  baseTokenProgram: "spl-token" | "token-2022";
  /** Extensions the venue's pool program accepts on a Token-2022 mint. */
  extensionAllowlist: string[];
  /** Where each field above was read, and when. */
  source: string;
}

export interface VenueVerdict {
  venue: string;
  accepted: boolean;
  reasons: string[];
}

/** SPROUT's final launch-venue extension set, transcribed from the R6 report. */
export const SPROUT_ROUTABLE_SET = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
];

export const VENUES: GraduationVenue[] = [
  {
    name: "pump.fun -> PumpSwap",
    ammProgramId: "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA",
    baseTokenProgram: "spl-token",
    extensionAllowlist: [],
    source:
      "@pump-fun/pump-sdk IDL: the create instruction pins token_program to TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (read 2026-08-22)",
  },
  {
    name: "Raydium CP-Swap",
    baseTokenProgram: "token-2022",
    extensionAllowlist: [
      "TransferFeeConfig",
      "MetadataPointer",
      "TokenMetadata",
      "InterestBearingConfig",
      "ScaledUiAmount",
    ],
    source: "CP-Swap extension allowlist, verified on a mainnet fork in m05-l1",
  },
];

/** A venue accepts SPROUT only if it can hold the mint AND every extension on it. */
export function checkGraduationVenue(
  venue: GraduationVenue,
  routableSet: string[],
): VenueVerdict {
  const reasons: string[] = [];
  if (venue.baseTokenProgram !== "token-2022") {
    reasons.push(
      `venue mints/accepts ${venue.baseTokenProgram} only; SPROUT is a Token-2022 mint`,
    );
  }
  const refused = routableSet.filter(
    (e) => !venue.extensionAllowlist.includes(e),
  );
  if (refused.length > 0) {
    reasons.push(`extensions not on the venue allowlist: ${refused.join(", ")}`);
  }
  return { venue: venue.name, accepted: reasons.length === 0, reasons };
}
```

   La entrada de CP-Swap a propósito no tiene `ammProgramId`. No te voy a entregar una cadena base58 para pegar desde una página de curso cuando el propio SDK de la plataforma la exporta, y el campo es opcional exactamente por esa razón: el id de PumpSwap es un hecho de primera mano congelado que este curso verificó, el de CP-Swap lo lees de `CREATE_CPMM_POOL_PROGRAM` en el SDK de Raydium que ya clonaste en m05-l1.

6. Emite y autoevalúate. La salida es el entregable, así que imprime la derivación con sus valores intermedios en vez de solo la respuesta, que es lo que la vuelve revisable por alguien que no estuvo en esta lección:

```typescript
function fmt(n: number, places = 3): string {
  return n.toLocaleString("en-US", {
    minimumFractionDigits: places,
    maximumFractionDigits: places,
  });
}

function main(): void {
  const c = PUMP_REFERENCE_CURVE;
  const f = finalReserves(c);
  const grad = graduationSol(c);

  console.log("# SPROUT launch curve (R10)\n");
  console.log("## Derived graduation threshold\n");
  console.log(`k (held constant):        ${fmt(f.k, 0)} SOL*tokens`);
  console.log(`final virtual token:      ${fmt(f.finalVirtualToken, 0)} tokens`);
  console.log(`final virtual SOL:        ${fmt(f.finalVirtualSol)} SOL`);
  console.log(`SOL added to graduate:    ${fmt(grad)} SOL`);

  const open = spotPrice(c.virtualSolReserves, c.virtualTokenReserves);
  const close = spotPrice(f.finalVirtualSol, f.finalVirtualToken);
  console.log(`opening spot price:       ${open.toExponential(4)} SOL/token`);
  console.log(`graduation spot price:    ${close.toExponential(4)} SOL/token`);
  console.log(`price multiple:           ${fmt(close / open, 2)}x`);
  console.log(
    `flat-price estimate:      ${fmt(c.realTokenReserves * open)} SOL (wrong by construction)\n`,
  );

  console.log("## Migrate target\n");
  const target = VENUES[0];
  const targetId = target.ammProgramId ?? "read the id from the venue's own SDK";
  console.log(`${target.name.split(" -> ")[1]} (${targetId})\n`);

  console.log("## Graduation venue check vs SPROUT's R6 set\n");
  console.log(`SPROUT routable set: ${SPROUT_ROUTABLE_SET.join(", ")}\n`);
  const verdicts = VENUES.map((v) => checkGraduationVenue(v, SPROUT_ROUTABLE_SET));
  for (const v of verdicts) {
    console.log(`- ${v.venue}: ${v.accepted ? "ACCEPTED" : "REFUSED"}`);
    for (const r of v.reasons) console.log(`    reason: ${r}`);
  }

  const chosen = verdicts.find((v) => v.accepted);
  if (chosen === undefined) {
    console.error("\nGATE FAIL: no candidate venue accepts SPROUT's R6 set");
    process.exit(1);
  }
  console.log(`\nSelected graduation venue: ${chosen.venue}`);

  if (Math.abs(grad - 85.005) > 0.01) {
    console.error(
      `\nGATE FAIL: reference constants should derive to ~85.005 SOL, got ${fmt(grad)}`,
    );
    process.exit(1);
  }
  console.log("All gates pass: threshold derived, venue selected.");
}

main();
```

   Córrelo desde la carpeta padre (`cd ..` para salir de `sprout-launch/` primero, el cambio de directorio de trabajo del que avisó el paso 1) para que la ruta coincida con el comando de verificación del curso:

```bash
npx tsx sprout-launch/derive-graduation.ts
```

```
# SPROUT launch curve (R10)

## Derived graduation threshold

k (held constant):        32,190,000,000 SOL*tokens
final virtual token:      279,900,000 tokens
final virtual SOL:        115.005 SOL
SOL added to graduate:    85.005 SOL
opening spot price:       2.7959e-8 SOL/token
graduation spot price:    4.1088e-7 SOL/token
price multiple:           14.70x
flat-price estimate:      22.174 SOL (wrong by construction)

## Migrate target

PumpSwap (pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA)

## Graduation venue check vs SPROUT's R6 set

SPROUT routable set: TransferFeeConfig, MetadataPointer, TokenMetadata

- pump.fun -> PumpSwap: REFUSED
    reason: venue mints/accepts spl-token only; SPROUT is a Token-2022 mint
    reason: extensions not on the venue allowlist: TransferFeeConfig, MetadataPointer, TokenMetadata
- Raydium CP-Swap: ACCEPTED

Selected graduation venue: Raydium CP-Swap
All gates pass: threshold derived, venue selected.
```

7. Checkpoint, y después rómpelo a propósito, porque un criterio que nunca viste fallar es una decoración. Primero demuestra que la derivación está viva: cambia `PUMP_REFERENCE_CURVE` a `{ virtualSolReserves: 30, virtualTokenReserves: 1_000_000_000, realTokenReserves: 800_000_000 }` y corre otra vez. Deberías ver `SOL added to graduate: 120.000` y después `GATE FAIL`, porque la assertion de los 85 SOL está revisando las constantes de referencia específicamente. Esa falla es la demostración: el número recalculado, por su cuenta, a partir de constantes que editaste. Vuelve a poner la curva de referencia.

   Después demuestra que la revisión de plataforma es real: quita `"TransferFeeConfig"` de la `extensionAllowlist` de CP-Swap y corre otra vez. Ahora las dos plataformas rechazan, ningún candidato queda seleccionado, y el script sale con código distinto de cero en vez de entregar un plan de lanzamiento para un token que nadie va a poner en un pool. Vuelve a ponerlo.

![La derivación de cuatro líneas anotada línea por línea, llevando una k de 32.19 mil millones a través de una reserva final de token de 279.9 millones hasta el umbral de graduación de 85.005 SOL.](assets/v08-annotated-code.webp)

## Challenge

El lab te hizo derivar la línea clave de `finalReserves` en papel antes de entregarte el listado. El challenge te saca el apoyo.

Abre el coding challenge de esta lección y vas a encontrar un starter que modela la graduación como la modela primero la mayoría de la gente: toma el precio spot de apertura y lo multiplica por la reserva real de token. Es la respuesta de 22 SOL, disfrazada de TypeScript. Tu trabajo es reemplazar ese modelo por una derivación de producto constante, implementando `graduationSol` para que el umbral salga de las reservas en vez de estar afirmado. Dos notas de interfaz antes de que empieces. Primero, el grader llama a tu función con tres números posicionales en un orden fijo, `graduationSol(virtualSolReserves, virtualTokenReserves, realTokenReserves)`, y no con el objeto de config que el lab pasaba de mano en mano. Segundo, el challenge guarda sus reservas de token en *millones* de tokens y no en los tokens enteros que usó el lab, así que las constantes de pump llegan como `graduationSol(30, 1073, 793.1)`. Al invariante no le importa el escalado, que es en sí mismo el punto: escala las dos reservas de token por el mismo factor y la respuesta en SOL no cambia.

Cuatro pruebas, y la tercera es la que hay que pensar. Las constantes de referencia de pump tienen que devolver unos 85.005 SOL. Una curva alterada en 30 / 1000 / 800 tiene que devolver 120. Una curva que arranca con una reserva de SOL más profunda, 85 / 1073 / 793.1, tiene que devolver unos 240.848, mismas reservas de token, misma forma, y el costo escala exactamente por el factor por el que escaló la reserva de SOL, 85/30, porque `graduationSol` es lineal en la reserva de SOL inicial. Esa es la prueba que un modelo de precio plano falla por el margen más ancho. Y una curva con reserva real cero tiene que devolver 0, que el modelo ingenuo también pasa, así que no demuestra nada por su cuenta y está ahí como ancla de cordura. Si te encuentras escribiendo un loop que recorre la curva en pasos chicos y acumula, detente: eso va a pasar las cuatro pruebas y quiere decir que estás integrando numéricamente algo que el invariante ya resolvió en forma cerrada.

![Una tabla con los cuatro casos de prueba del challenge que empareja cada umbral de graduación esperado con la respuesta equivocada de precio plano, desde la curva de referencia de 85.005 hasta el ancla de cordura de reserva cero.](assets/v09-table.webp)

Después una pieza de juicio que ninguna prueba puede calificar, y es el entregable que este módulo de verdad quiere. Escribe tres oraciones sobre el lanzamiento de SPROUT. Oración uno: el umbral de graduación que modelarías para SPROUT, y las constantes de las que se deriva, dado que SPROUT no se está lanzando en pump. Oración dos: por qué pump no está disponible para SPROUT, nombrando el mecanismo específico y no la vibra. Oración tres: a qué tendrías que renunciar de SPROUT para que pump esté disponible, y si lo harías. Si tu tercera oración concluye que quitar la comisión de transferencia para caber en la plataforma está bien, vuelve a tu informe de R6 y lee qué está financiando la comisión antes de comprometerte. Esa es una decisión de tesorería, y la restricción de tooling es solo lo que la destapó.

Una cosa más que vale la pena hacer mientras la derivación está fresca. Toma los tipos `FeeConfig` y `FeeTier` de más atrás en esta lección y ve a leer la tabla de tiers real directo de la blockchain. A propósito no imprimí los umbrales aquí, porque son datos de cuenta con un admin, y un curso que los congela es un curso que le miente a quien lo lea en seis meses. Sácalos tú mismo, anota la fecha al lado de lo que encuentres, y habrás hecho la cosa que esta lección entera de verdad está enseñando, que es distinguir entre un número que se deriva, un número que se guarda y un número que se repite.

Si tu umbral derivado no coincide con los 85.005 impresos aquí, revisa las constantes primero, porque la cuenta `Global` de pump está viva y una autoridad puede cambiar cualquiera de las cuatro. Si las constantes coinciden y el número sigue difiriendo, repórtalo en el canal de feedback del curso con tu salida y la fecha en que leíste el IDL. Mis lecturas están estampadas 2026-08-22 contra `@pump-fun/pump-sdk` en el latest de npm 1.36.0, y un aprendiz que detecte esto yéndose a la deriva está haciendo precisamente el trabajo que la lección existe para instalar.

Ahora puedes derivar dónde termina una curva, a partir de constantes en vez de folclore, y tienes una config que rechaza una plataforma en la que tu token no puede aterrizar legalmente. Pero la curva que derivaste es una curva, y la plataforma que te rechazó es una plataforma. Cada launchpad publica sus propias constantes, su propia división de comisiones, y su propia opinión sobre qué programas de token merecen un asiento. Lo que sigue: el panorama de los launchpads, y por qué LetsBonk es un skin de Raydium.
