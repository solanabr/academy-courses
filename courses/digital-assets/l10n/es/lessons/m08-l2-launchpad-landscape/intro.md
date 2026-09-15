# El panorama de los launchpads, anti-snipe y lo que siembra la graduación

## Resumen

La lección pasada derivaste el umbral de graduación de SPROUT, de ~85 SOL, a partir de las constantes del propio pump y fijaste su destino de migración en PumpSwap. Esta lección amplía el encuadre de una sola plataforma al espacio de diseño: pump.fun, Raydium LaunchLab, Meteora Dynamic Bonding Curve y Metaplex Genesis, comparadas en los ejes que de verdad te cuestan dinero. Vas a sondear cuatro program ids en vivo, leer las perillas que cada plataforma le da al creador, sopesar las cuatro defensas anti-snipe que circulan y anotar exactamente qué siembra un launchpad en la graduación frente a lo que sembrarías a mano. Después tomas la decisión de SPROUT: una plataforma, una defensa, justificada contra el conjunto de extensiones enrutable que congelaste en R6. El repliegue es amplio aquí. Yo te doy la tabla de plataformas y la forma de la función de decisión; la regla de puntuación, la elección de la defensa y la justificación escrita son tuyas. Esta es la última lección donde el diseño del token te limita a ti en vez de al revés.

Aquí va algo que te va a costar dinero de verdad si nadie lo dice en voz alta. Todo launchpad del que hayas oído hablar publica una página de marketing sobre su curva, y casi ninguno publica lo que de verdad decide si tu token puede usarlos: en qué AMM aterriza el pool cuando la curva se completa, y qué va a aceptar ese AMM. Puedes ejecutar un lanzamiento impecable, alcanzar el umbral y ver cómo la transacción de migración revierte porque tu mint lleva una extensión que el pool de destino rechaza. La curva nunca fue el riesgo. La última instrucción sí.

Así que, antes de cualquier teoría, ve a averiguar cuántos programas hay en realidad detrás de las marcas. Vuelve a la carpeta `sprout-launch/` de la lección pasada, junto al `derive-graduation.ts` que escribiste ahí, y suelta esto:

```typescript
// probe-venues.ts: are these launchpads four programs, or fewer than they look?
const RPC = process.env.SOLANA_RPC_URL ?? "https://api.mainnet-beta.solana.com";

const VENUES: Record<string, string> = {
  "pump.fun": "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
  "Raydium LaunchLab": "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
  "LetsBonk": "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
  "Meteora DBC": "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN",
  "Metaplex Genesis": "GNS1S5J5AspKXgpjz6SvKL66kPaKWAhaGRhCqPRxii2B",
};

type AccountValue = { executable: boolean; owner: string } | null;

async function probe(address: string): Promise<AccountValue> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      // dataSlice keeps the response tiny: we want the flags, not the bytecode.
      params: [address, { encoding: "base64", dataSlice: { offset: 0, length: 0 } }],
    }),
  });
  const json = (await res.json()) as { result?: { value: AccountValue } };
  return json.result?.value ?? null;
}

async function main(): Promise<void> {
  const seen = new Map<string, string[]>();
  for (const [name, id] of Object.entries(VENUES)) {
    const acct = await probe(id);
    const state = acct ? (acct.executable ? "executable" : "NOT A PROGRAM") : "NOT FOUND";
    console.log(name.padEnd(20), id.padEnd(46), state);
    seen.set(id, [...(seen.get(id) ?? []), name]);
  }
  console.log("");
  for (const [id, names] of seen) {
    if (names.length > 1) console.log(`same program id: ${names.join(" + ")}  ->  ${id}`);
  }
  console.log(`distinct programs: ${seen.size} for ${Object.keys(VENUES).length} brands`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

```bash
npx tsx probe-venues.ts
```

Cinco marcas. Cuatro programas. Esa última línea es toda la lección en un solo número, y las próximas cuatro mil palabras tratan de por qué ocurre el colapso, qué perillas lo sobreviven y cómo eliges.

## La misma cocina detrás de cuatro carteles distintos

### Leer el sondeo que acabas de correr

Los cuatro program ids volvieron como ejecutables cuando corrí ese script el 2026-08-22, que es la única afirmación que estoy dispuesto a hacer sobre ellos sin que tú lo vuelvas a correr. Los program ids sí se reemplazan. Solo Raydium ya ha entregado varias generaciones de programas de pool, y un curso que congela una dirección es un curso que le miente a alguien en 2027. El script es el hecho; las direcciones que trae son una instantánea.

La fila interesante es LetsBonk. No es un fork de LaunchLab, y no es un competidor de LaunchLab en el sentido en que lo encuadró la cobertura de prensa en la semana del lanzamiento. Es LaunchLab con otro cartel. El programa de Raydium expone una **Platform PDA**, una cuenta de configuración por plataforma derivada bajo el programa de LaunchLab, y un tercero que crea una consigue su propio frontend de marca, su propia estructura de comisiones y su propia tajada de la recaudación, corriendo contra exactamente el mismo conjunto de instrucciones y graduándose hacia exactamente el mismo AMM. Eso es lo que atrapó tu sondeo. Los dos nombres resolvieron a una sola clave de 32 bytes, así que todo lo que sea cierto de la mecánica de LaunchLab es cierto de la mecánica de LetsBonk, por construcción.

Quiero ser honesto sobre lo que el sondeo demostró y lo que no, porque esta es la clase de afirmación que se repite hasta que deja de comprobarse. Lo que demostraste es que el id que etiqueté como "LetsBonk" es un programa ejecutable vivo y que es igual al de LaunchLab. Lo que no demostraste es que los lanzamientos de LetsBonk de verdad se enruten por ahí, porque eso exige leer una transacción de lanzamiento real, no una bandera de cuenta. Hazlo tú mismo antes de repetir la afirmación: abre cualquier token de LetsBonk en un explorador, busca su transacción de creación y lee el program id invocado en la instrucción. Si dice `LanMV9sA...`, la afirmación se sostiene en tus manos y no en las mías.

Piénsalo como una franquicia, porque la analogía aguanta hasta el fondo y la voy a seguir usando. Una cocina, una freidora, un contrato con el proveedor. El franquiciado elige el cartel sobre la puerta, los precios en la pizarra y quién se queda con la caja al cerrar. Lo que el franquiciado no puede cambiar es la freidora. Y ese es exactamente el canje que ofrece la Platform PDA: control total de marca y reparto de comisiones, cero control de mecánica.

![Un diagrama de centro y radios del programa Raydium LaunchLab con radios de Platform PDA para Raydium, LetsBonk y terceros, que comparten una sola curva y una sola regla de graduación mientras cada uno configura marca y comisiones.](assets/v01-diagram.png)

### Qué es en realidad un launchpad

Quítale la marca y un launchpad son cuatro decisiones empaquetadas juntas y vendidas como un solo producto. Toda plataforma que evalúes está respondiendo estas cuatro, lo digan o no sus docs.

**Uno, el mecanismo de precio.** Cómo se mueve el precio mientras el token sigue en la plataforma. Una curva de producto constante fija, una curva que parametrizas, una curva por tramos que moldeas segmento por segmento, o ninguna curva si la plataforma corre una subasta.

**Dos, el esquema de comisión.** Quién se lleva qué, de qué lado de la graduación, y si el esquema puede cambiar bajo tus pies después del lanzamiento. La comisión de bonding curve de pump ERA de 100 basis points planos sobre las operaciones contra la curva, hasta que el día de bandera del 2025-09-01 que fechaste la lección pasada la reemplazó por niveles por capitalización de mercado editables por el admin; presupuesta desde la tabla de niveles viva, nunca desde la tasa plana histórica. Metaplex Genesis publica 0.50% de protocolo más 0.60% de ingresos para el creador en su modo bonding curve (su modo estelar es la subasta que vas a conocer más abajo; el modo curva es el que tiene esta hoja de comisiones), y una hoja distinta después de la graduación: 0.40% de protocolo, 0.42% para los LPs, 0.04% para Raydium en el CPMM del launch pool. Esos números son los que estaban en sus docs el día que los leí, y las hojas de comisiones son la página que más rápido se pudre de todas las que publica un protocolo. Vuelve a leerlas antes de comprometerte.

**Tres, la defensa.** Qué se interpone, si acaso algo, entre tu lanzamiento y los bots. Este es el eje que la mayoría de los creadores descubre demasiado tarde, y es aquel en el que esta lección pasa más tiempo.

**Cuatro, el destino de graduación.** En qué AMM aterriza la liquidez cuando la curva se completa, y por lo tanto qué diseños de token son siquiera legales. Este es el eje que te come si te lo saltaste, y es la razón por la que el reporte R6 que escribiste el módulo pasado es un documento de lanzamiento y no un ejercicio de diseño.

Fíjate en lo que no está en esa lista: el estándar del token. Cada una de estas plataformas te va a acuñar con gusto un token SPL. Solo algunas van a llevar un mint de Token-2022 con extensiones de poder hasta el final de la migración, y ninguna te lo va a decir en el momento de acuñar. La falla aparece en la última instrucción.

![Una comparación de cuatro columnas de pump.fun, Raydium LaunchLab, Meteora DBC y Metaplex Genesis a lo largo de ocho ejes de lanzamiento, con Meteora DBC como la única plataforma documentada para soportar configuraciones de transfer hook.](assets/v02-comparison.png)

### pump.fun: la plataforma sin perillas

A esta ya la conoces por dentro, que es lo que la hace una línea base limpia. Cuatro constantes, ningún parámetro, una sola forma para cada token que se ha lanzado ahí. El supply es de mil millones con seis decimales, las reservas virtuales arrancan en 30 SOL y 1,073,000,000,000,000 unidades base, la reserva real del token es de 793,100,000,000,000, y drenar esa reserva es lo que pone `complete = true` y desbloquea un migrate sin permiso e idempotente que quema el LP. El umbral cae alrededor de 85 SOL, y lo derivaste en vez de buscarlo.

El día de bandera del 2025-09-01 es la parte que vale la pena traer a esta lección. De la noche a la mañana, el esquema bajo cada token de pump vivo cambió, y no se consultó a ningún creador, porque ningún creador tuvo nunca un voto. Eso es el trato, no un escándalo. Una plataforma sin perillas es una plataforma cuya política la fija otro, para siempre, incluidas las partes alrededor de las cuales pusiste el precio de tu lanzamiento.

La ventaja de no tener perillas es real y la subestima la gente a la que le gustan las perillas: no la puedes configurar mal. Cada lanzamiento es el mismo lanzamiento, así que la liquidez, los bots, los frontends y los dashboards conocen la forma de antemano, y la superficie de integración es enorme porque nunca cambia. Cero configuración es una feature cuando la alternativa eres tú, a las 3am, eligiendo una duración de cliff que no entiendes.

![Un gráfico de barras agrupadas con las tasas de comisión leídas el 2026-08-22, con el 1.00% de pump.fun marcado como histórico y reemplazado por niveles dinámicos, el modo curva de Genesis apilado en 1.10% y 0.86%, y dos plataformas dejadas como barras de marcador de posición marcadas como sin fuente.](assets/v03-chart.png)

### LaunchLab: perillas, un contrato de franquicia y un NFT que recauda renta

El LaunchLab de Raydium es la misma primitiva con el panel de ajustes desbloqueado, y ya viene con dos puertas. **JustSendit** es la puerta de los valores por defecto: sin configuración, y la curva se gradúa a un pool AMM de Raydium con 85 SOL recaudados. Sí, los mismos 85 que pump. Esa coincidencia merece una pausa, porque es exactamente la forma de suposición que quema a la gente: dos plataformas convergieron en el mismo número redondo por razones distintas, así que un lector que generalice "los launchpads se gradúan a 85 SOL" va a acertar dos veces y equivocarse en cada plataforma configurable del espacio. El umbral es por plataforma, y en las configurables es por lanzamiento.

**El modo LaunchLab** es la otra puerta: controles de supply, métricas de venta, parámetros de curva y vesting con un cliff y una duración de desbloqueo. Una propiedad de esas perillas de vesting merece su propia oración, porque es la diferencia entre un error y una catástrofe. Los períodos de cliff y de desbloqueo quedan fijados en el lanzamiento y no se pueden cambiar retroactivamente. No estás configurando un ajuste de dashboard. Estás escribiendo una cláusula en un contrato que va a seguir imponiéndose mucho después de que hayas olvidado qué número escribiste. (La lección de airdrops vuelve a estas mismas perillas de cliff y duración desde el lado del reclamo, donde el vesting es un reclamo merkle y no un parámetro de lanzamiento.)

Después está la pieza que replantea toda la economía, y es la razón por la que esta lección vive en el módulo de economía del token y no junto a la matemática de la curva. Habilita el reparto de comisiones posterior a la migración en la creación y, cuando el token se gradúa, LaunchLab acuña un **Fee Key NFT** a la billetera del creador. La billetera que tiene ese NFT puede reclamar el 10% de todas las comisiones de trading que el pool graduado le paga al LP. Lo que quiere decir que la quema del LP no es lo que probablemente supusiste: el 90% de los tokens LP se queman, y el 10% restante queda bloqueado en el Burn and Earn de Raydium, con la Fee Key como el ticket de reclamo.

Eso cambia lo que "el LP está quemado" quiere decir como señal de confianza. Un LP quemado es la promesa estándar de que nadie puede hacer un rug pull. Con el reparto de comisiones, la versión honesta es que el 90% del rug está clavado al piso y el otro 10% es una anualidad permanente y transferible que alguien posee. Los docs son tajantes sobre la consecuencia: quema o transfiere la Fee Key y el derecho a reclamar se pierde para siempre. Así que el activo más valioso a largo plazo de un lanzamiento se puede perder por una limpieza de billetera, y también se puede vender, que es un mercado que nadie planeó y que ahora todos tienen.

El trade-off, dicho sin rodeos. LaunchLab te compra configuración, un flujo de comisiones que sobrevive a la graduación y la opción de correr tu propio launchpad de marca sobre el programa auditado de otro. Lo que cuesta es que cada una de esas perillas es una decisión que ahora te pertenece para siempre, la historia de la quema del LP se vuelve más complicada de explicarles a los tenedores, y el AMM de graduación es Raydium CPMM, cuya allowlist de Token-2022 ya te sabes en frío desde R6. Tres extensiones adentro, cuatro afuera.

### Meteora DBC: la curva como estructura de datos

La Dynamic Bonding Curve de Meteora es la plataforma que trata la curva como configuración y no como producto. En vez de una sola curva de producto constante, la config guarda un array de pares `(sqrt_price, liquidity)`, y el pool interpola comportamiento de producto constante entre ellos. Por tramos, en otras palabras: dibujas un tramo plano y barato para quienes apoyan temprano, luego un tramo más empinado, luego la forma que tu distribución de verdad quiera.

Aquí es donde la lección consigue su segundo golpe de color, y es uno bueno para un hábito y no para un dato. Los docs describen una curva personalizable de 16 puntos. El array de config on-chain tiene 20 de ancho. Los dos números son reales, y los dos están en el código fuente. Saqué el archivo de constantes el 2026-08-22:

```rust
// programs/dynamic-bonding-curve/src/constants.rs (excerpt)
pub const MAX_CURVE_POINT: usize = 16;
pub const MAX_CURVE_POINT_CONFIG: usize = 20;
const_assert!(MAX_CURVE_POINT <= MAX_CURVE_POINT_CONFIG);

// programs/dynamic-bonding-curve/src/state/config.rs (excerpt, fields elided)
#[zero_copy]
pub struct LiquidityDistributionConfig {
    pub sqrt_price: u128,
    pub liquidity: u128,
}

#[account(zero_copy)]
pub struct PoolConfig {
    // ... quote_mint, fee_claimer, leftover_receiver, fee and vesting configs ...
    pub token_type: u8, // 0 = SplToken, 1 = Token2022
    // ... the rest of the u8 flag run ...
    pub padding_2: [u8; 7], // declared, not implied: a zero-copy struct may carry no implicit padding
    pub swap_base_amount: u64,
    pub migration_quote_threshold: u64,
    pub migration_base_threshold: u64,
    // ... migration_sqrt_price, locked vesting, supply and migrated-fee fields ...
    pub curve: [LiquidityDistributionConfig; MAX_CURVE_POINT_CONFIG],
}
```

Dos constantes, dos trabajos, y el `const_assert!` entre ellas es lo que lo delata: el propio código fuente declara una como techo por debajo de la otra. La cuenta reserva 20 slots para que el layout tenga margen; la curva validada tiene un tope más bajo. Si hubieras confiado solo en los docs, habrías creído que el array tenía 16 de ancho y habrías dimensionado mal un deserializador. Si hubieras confiado solo en el struct, habrías creído que podías pasar 20 puntos y una instrucción te habría rechazado. La regla que sobrevive a los dos errores es la que vienes corriendo desde la matriz de conflictos de extensiones: lee el código fuente, y lee lo suficiente para saber qué constante gobierna qué superficie. Después vuelve a verificar en tu propio momento de escritura, porque estoy citando un repositorio en una fecha y los repositorios se mueven.

Otros dos campos de ese struct deciden si SPROUT puede usar esta plataforma siquiera, y aquí la precisión sobre CUÁL lado del par cubre cada afirmación importa, porque son afirmaciones distintas con evidencia distinta. `token_type` es 0 para SPL clásico y 1 para Token-2022, y lo que codifica de forma demostrable, sentado en PoolConfig junto a `quote_mint`, es el programa de token del lado QUOTE: es la forma en que DBC acepta un quote mint de Token-2022.

El lado BASE, el lado donde viviría SPROUT, se apoya en una afirmación de documentación aparte: los docs de DBC describen tokens base de Token-2022 incluidas las configuraciones de transfer hook, lo que lo convierte en el contrapunto de Raydium en toda esta comparación, ya que un token con hook rechazado por CP-Swap tiene aquí un camino documentado. Pero documentado no es interrogado. La lección pasada leíste el IDL de pump línea por línea para demostrar su camino de create; esta lección no ha hecho eso con el create del lado base de DBC, así que trata el soporte de base-Token-2022-con-comisión-de-transferencia como documentado-pero-sin-verificar hasta que leas el camino de create de DBC o levantes un pool en devnet con un base mint que cobre comisión. El registro de plataforma del lab lleva esa bandera, y la decisión que alimenta hereda la salvedad.

`migration_quote_threshold` es el disparador de graduación, en unidades del token quote, y es un número que eliges en vez de un número que derivas.

El lado del costo, porque una plataforma así de flexible no es gratis. Cada segmento es una decisión de distribución que ahora tienes que defender, y una curva por tramos te da muchas más formas de equivocarte que una fija. La matemática del pool y la estrategia de LP que te dejarían moldear esos segmentos con inteligencia están de verdad fuera de alcance aquí, y no las voy a fingir: el curso planificado DeFi and RWA Engineering enseña provisión de liquidez a profundidad real, y ahí es donde moldear curvas deja de ser un menú y se vuelve una disciplina. Esta lección te lleva exactamente hasta elegir la plataforma y saber cuáles son sus perillas.

![Un extracto anotado del código fuente de la Dynamic Bonding Curve de Meteora que muestra MAX_CURVE_POINT en 16 y MAX_CURVE_POINT_CONFIG en 20, con llamadas que nombran la falla que causa cada número si se confía solo en él.](assets/v04-annotated-code.png)

### Genesis: cuando una curva es la forma equivocada por completo

Metaplex Genesis pertenece a esta comparación precisamente porque su modo estelar rechaza la premisa. (Una reconciliación con la sección de comisiones, para que las dos no se lean como una contradicción: Genesis también viene con un modo de lanzamiento por bonding curve, y la cifra de 1.10% citada antes sale de la hoja de comisiones de ESE modo. Esta sección cubre el diferenciador, la subasta, que es lo que modela el registro de plataforma del lab; el modo curva existe y no se modela aquí.) Una bonding curve es un mecanismo de precio con un sesgo específico horneado adentro: los compradores más tempranos pagan menos, mecánicamente, siempre. Ese sesgo es el punto cuando lanzas un token de comunidad y quieres que se recompense a quienes apoyan temprano. Es un bug cuando lanzas algo con demanda institucional real, porque convierte "llegar temprano" en "ser rápido", y ser rápido es un servicio que venden los bots.

Genesis ofrece en cambio una **subasta a precio uniforme**. Las pujas entran durante una ventana, se ordenan por precio, y cada postor ganador paga el mismo precio de corte, fijado en la puja ganadora más baja. A nadie se le recompensa por aterrizar una transacción 40 milisegundos antes que otro, porque el orden de llegada deja de ser una entrada del precio. Genesis lo encuadra como el modo para proyectos establecidos con interés institucional, y la arquitectura a su alrededor es un sistema de **buckets**: los buckets de entrada recaudan SOL de los participantes, los buckets de salida enrutan fondos a una tesorería o a un destino de vesting a través de comportamientos de cierre configurables.

Lo que te cuesta es aquello en lo que las curvas son de verdad buenas. Una subasta necesita que la demanda aparezca dentro de una ventana, y una ventana es un problema de coordinación: tienes que promocionarla, y si la ventana cierra floja, descubriste tu curva de demanda en público. Una bonding curve nunca tiene ese modo de falla, porque siempre está abierta y siempre está cotizando. Elige la subasta cuando el lanzamiento tenga suficiente peso para llenar una sala en una fecha fija. Elige una curva cuando no lo tenga.

![Un diagrama de flujo de datos que muestra a Metaplex Genesis enrutando el SOL de los participantes a través de buckets de entrada, un comportamiento de cierre y buckets de salida, con un carril paralelo que asigna tokens a los ganadores a un solo precio de corte.](assets/v05-diagram.png)

### Cuatro defensas contra la misma ventana disputada

Todo mecanismo anti-snipe en este espacio ataca la misma ventana. Desde el slot en que tu pool se vuelve negociable hasta el slot en que un ser humano puede reaccionar hay una brecha de unos cientos de milisegundos con los tiempos de slot actuales, y un bot con una conexión caliente y una comisión de prioridad se queda con toda ella. Las defensas difieren en qué palanca accionan.

**La ventana de depósito.** El Alpha Vault de Meteora se sienta delante del lanzamiento y acepta depósitos antes de que abra el trading, en modo por orden de llegada o a prorrateo, y después compra como un solo participante en la apertura. Todo depositante consigue el mismo precio de ejecución. La ventaja de velocidad del bot se evapora porque no hay nada que correr: la compra ya ocurrió, colectivamente, a un precio que nadie pudo saltarse. El costo es un calendario y un techo. Les estás pidiendo a quienes te apoyan que comprometan capital antes de un lanzamiento, que es un pedido mucho más grande que hacer clic en comprar, y en modo a prorrateo nadie sabe su ejecución exacta hasta que la ventana cierra.

**La comisión decreciente.** Empieza con la comisión de trading en modo castigo y déjala caer durante los primeros minutos o bloques. Un sniper que compra en el primer slot paga una tasa que se come el arbitraje; un comprador normal que llega cuatro minutos después paga casi lo normal. Esta es la defensa menos intrusiva, porque no cambia ningún flujo y no le pide nada a tu comunidad, y es la más débil, porque un salto de precio esperado lo bastante grande todavía justifica la comisión. Grava el sniping en vez de impedirlo.

**La primera compra reservada y sin comisión.** El creador, o un conjunto en la allowlist, consigue una compra al precio de apertura de la curva con las comisiones exoneradas antes de que el pool se abra a todos. Le garantiza al equipo o a la comunidad una posición en el piso. También concentra el supply exactamente de la forma que un público de fair launch está vigilando, así que te compra defensa al costo directo de la imagen pública por la que probablemente estabas lanzando.

**La subasta a precio uniforme.** Genesis, como arriba. La más fuerte de las cuatro, porque no grava ni retrasa la carrera, borra la carrera al sacar el tiempo de la función de precio. Y es la más cara, porque exige que el lanzamiento se comporte como un evento.

Hay una quinta opción que la gente olvida: ninguna defensa. Eso es lo que es un lanzamiento de pump a secas, y es una elección coherente si el token es pequeño, el lanzamiento es silencioso, y el costo de que un bot consiga una buena ejecución es de verdad menor que el costo de pedirle a tu comunidad que aprenda una ventana de depósito. Nombrarla como una elección es distinto de tropezarse con ella.

![Una línea de tiempo de lanzamiento con los primeros slots disputados por snipers tras la apertura del trading, y cuatro defensas posicionadas alrededor de esa zona: ventana de depósito, primera compra sin comisión, comisión decreciente y subasta a precio uniforme.](assets/v06-timeline.png)

### Qué siembra en realidad la graduación

Aquí está la parte que todo launchpad hace por ti en silencio, que es por lo que casi nadie puede enumerarla cuando se le pregunta. La graduación siembra exactamente tres cosas.

**El par.** Una cuenta de pool en el AMM de destino que sostiene tu token contra el quote mint. El par lo elige la plataforma: pump te da SPROUT/SOL en PumpSwap, LaunchLab te da SPROUT/SOL en Raydium CPMM, DBC te da SPROUT contra el quote mint que hayas configurado en DAMM. No te toca negociar el activo quote en la graduación. Lo elegiste cuando elegiste la plataforma.

**El precio inicial.** No un número que fijas tú. El precio implicado por donde se detuvo la curva. La razón final de reservas virtuales al completarse es la cotización de apertura en el AMM, que es por lo que el punto final de la curva y el primer tick del pool son el mismo hecho económico visto dos veces. En una plataforma con curva fija, ese precio queda determinado en el momento en que lanzas. En DBC, lo determina tu último segmento.

**La posición de LP y su destino final.** La migración acuña tokens LP contra la liquidez sembrada y después hace algo irreversible con ellos. pump los quema. LaunchLab quema el 90% y bloquea el 10% detrás de la Fee Key cuando el reparto de comisiones está activo. DBC quema o bloquea según tu config. Sea cual sea la regla, se ejecuta dentro de la transacción de migración, y después de eso la posición es exactamente tan inamovible como la regla dijo que sería.

Ahora el contrafáctico, que es la única forma de sentir lo que estás consiguiendo. Sin un launchpad crearías la cuenta de pool y pagarías su rent tú mismo, fondearías los dos lados desde una billetera que controlas, elegirías un precio de apertura por juicio propio en vez de por mecanismo, recibirías tokens LP en esa misma billetera, y después resolverías el problema de confianza a mano: quemarlos y demostrarlo, o bloquearlos en algún lado y demostrar eso. También serías dueño de todo el problema anti-snipe tú solo, porque un pool nuevo sin defensa es un pool al que le hacen snipe en su primer slot, por definición. Eso son cuatro trabajos y una demostración de confianza, a cambio del control sobre cada uno de ellos.

![Una tabla de tres columnas que enumera las cinco cosas que un launchpad siembra en la graduación, quién decide cada una, y el equivalente manual desde la creación del pool hasta la cobertura anti-snipe.](assets/v07-table.png)

### El conjunto de extensiones vota primero

Lo que devuelve toda la comparación a una decisión que ya tomaste. El conjunto enrutable de SPROUT de R6 es TransferFeeConfig, MetadataPointer y TokenMetadata, y la razón de que sean esos tres es que la allowlist de Token-2022 de Raydium CP-Swap contiene exactamente cinco extensiones y rechaza todo lo que le permita a un emisor correr código o mover tokens de otras personas.

Vuelve a leer la tabla de comparación con eso en la mano y el campo de plataformas se estrecha solo. Tres de los cuatro destinos de graduación rechazan un transfer hook. Si SPROUT hubiera conservado su hook, pump, LaunchLab y Genesis revertirían todos en la migración, y no en el lanzamiento, que es la parte cruel. Recaudarías el SOL, alcanzarías el umbral, y fallarías en la última instrucción con un token que nadie puede negociar y una curva que ya está completa.

Así que el orden es fijo, y es lo contrario de cómo se planifican la mayoría de los lanzamientos. El conjunto de extensiones decide qué AMMs de graduación son legales. Los AMMs legales deciden qué plataformas están disponibles. Las plataformas disponibles te ofrecen un menú de defensas. Tú eliges de ese menú. Quien elige primero la plataforma va a terminar cambiando su token para que le calce, que es un buen resultado mientras haya sido una decisión y no un descubrimiento.

![Un diagrama de flujo de cuatro etapas que va del conjunto de extensiones a los AMMs legales, a las plataformas disponibles, a la elección de defensa, con una flecha inversa que marca el plan al revés habitual y una llamada sobre la reversión en la migración.](assets/v08-flowchart.png)

## Lab: elige la plataforma de SPROUT y escribe la decisión

El artefacto es `sprout-launch/choose-venue.ts`, y es la mitad de decisión del paquete `sprout-launch` cuya mitad de curva construiste la lección pasada. Ingiere el conjunto de extensiones R6 de SPROUT y el umbral derivado, rechaza toda plataforma cuyo AMM de graduación rechazaría el mint, rechaza toda plataforma sin defensa si pediste una, e imprime la decisión con sus rechazos adjuntos. El criterio: `npx tsx sprout-launch/choose-venue.ts` debe imprimir una plataforma, un AMM de graduación, una defensa nombrada y una lista de rechazos no vacía, y salir con 0.

**1.** Trabaja en la carpeta `sprout-launch/` de la lección pasada, junto a `derive-graduation.ts`. Nada nuevo que instalar si hiciste ese lab. Si arrancas limpio, el runner es la misma única dependencia de desarrollo, revisada de nuevo hoy: `tsx@4.23.12` era el latest de npm el 2026-08-22, y ese número se pudre como cada pin de este curso, así que corre `npm view tsx version` tú mismo el día que hagas el scaffold.

```bash
npm init -y
npm install -D tsx@4.23.12
```

**2.** Pon el espacio de diseño en disco como datos antes de escribir nada de lógica. El punto de una tabla como esta es que cada campo es una afirmación en la que puedes equivocarte en un solo lugar en vez de cinco, y los valores de `programId` son los que tu script de sondeo ya comprobó.

```typescript
// venues.ts: the launchpad design space as data. Every field is a decision you inherit.
export type VenueId = "pump" | "launchlab" | "dbc" | "genesis";
export type Defense = "alpha-vault-window" | "decaying-fee" | "fee-free-first-buy" | "uniform-price-auction" | "none";

export interface Venue {
  id: VenueId;
  label: string;
  programId: string;
  priceMechanism: "fixed-curve" | "configurable-curve" | "piecewise-curve" | "auction";
  /** how many liquidity segments the creator controls; 0 = none */
  curveSegments: number;
  /** the AMM the pool lands on at graduation */
  graduationAmm: string;
  /**
   * Which token program the venue's CREATE path can mint the base token under.
   * This gate runs before any extension talk: last lesson you proved from pump's
   * own IDL that its create pins the classic SPL Token program and mints the
   * token itself, so a Token-2022 mint cannot exist there at all.
   */
  baseTokenProgram: "spl" | "token2022" | "both";
  /** Token-2022 extensions the graduation AMM is documented to accept */
  acceptsTransferHook: boolean;
  acceptsToken2022Quote: boolean;
  defenses: Defense[];
  threshold: { kind: "fixed" | "derived" | "configurable"; sol?: number; field?: string };
  /** fraction of LP tokens burned at migration; the rest is locked, not free */
  lpBurnedAtMigration: number;
  seededAtGraduation: string[];
}

export const VENUES: Venue[] = [
  {
    id: "pump",
    label: "pump.fun",
    programId: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
    priceMechanism: "fixed-curve",
    curveSegments: 0,
    graduationAmm: "PumpSwap (pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA)",
    baseTokenProgram: "spl", // proven from pump's IDL last lesson: create mints classic SPL itself
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["none"],
    // ~85 SOL is DERIVED from pump's constants (derive-graduation.ts), never a spec constant:
    // recording it as "fixed" would be the repeated-number mistake last lesson buried.
    threshold: { kind: "derived", sol: 85, field: "virtual reserves at completion" },
    lpBurnedAtMigration: 1,
    seededAtGraduation: ["SPROUT/SOL pair", "price implied by the curve endpoint", "LP burned"],
  },
  {
    id: "launchlab",
    label: "Raydium LaunchLab (JustSendit)",
    programId: "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
    priceMechanism: "fixed-curve", // this record is the JustSendit defaults door; LaunchLab's other door unlocks the configurable curve
    curveSegments: 1,
    graduationAmm: "Raydium CPMM",
    baseTokenProgram: "spl", // LaunchLab's create mints the base token as classic SPL; verify against its IDL the way you did pump's
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["none"],
    threshold: { kind: "fixed", sol: 85 },
    // 90% burned, 10% locked in Burn & Earn when creator fee share is on
    lpBurnedAtMigration: 0.9,
    seededAtGraduation: [
      "SPROUT/SOL pair on CPMM",
      "price implied by the curve endpoint",
      "90% LP burned, 10% locked behind the Fee Key NFT",
    ],
  },
  {
    id: "dbc",
    label: "Meteora Dynamic Bonding Curve",
    programId: "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN",
    priceMechanism: "piecewise-curve",
    curveSegments: 16,
    graduationAmm: "DAMM v1 or v2",
    // DOCUMENTED-UNVERIFIED on the base side: PoolConfig's token_type field
    // demonstrably covers the QUOTE mint; Token-2022 BASE support (incl.
    // transfer-hook configs) is a docs claim this course has not read at IDL
    // depth. Verify the create path before shipping a fee-bearing base mint.
    baseTokenProgram: "both",
    acceptsTransferHook: true,
    acceptsToken2022Quote: true,
    defenses: ["alpha-vault-window", "decaying-fee"],
    threshold: { kind: "configurable", field: "migration_quote_threshold" },
    // Per config, not absolute: DBC's LP can be burned OR locked at migration.
    // 1 models the burn configuration; adjust to the config you would ship.
    lpBurnedAtMigration: 1,
    seededAtGraduation: [
      "SPROUT/quote pair on DAMM",
      "price implied by the last curve segment",
      "LP burned or locked per config",
    ],
  },
  {
    id: "genesis",
    label: "Metaplex Genesis",
    programId: "GNS1S5J5AspKXgpjz6SvKL66kPaKWAhaGRhCqPRxii2B",
    priceMechanism: "auction",
    curveSegments: 0,
    graduationAmm: "Raydium CPMM launch pool",
    baseTokenProgram: "spl", // no documented Token-2022 base path; treat as classic-only until you verify
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["uniform-price-auction"],
    threshold: { kind: "configurable", field: "auction clearing price" },
    // UNSOURCED: no doc statement on Genesis LP disposal surfaced in this
    // course's reads; recorded as burn by analogy with the launch-pool
    // pattern. Verify against Genesis docs before this cell decides anything.
    lpBurnedAtMigration: 1,
    seededAtGraduation: [
      "SPROUT/SOL pair",
      "price set by the auction clearing price, not a curve",
      "LP disposal: unsourced, verify (recorded as burn by analogy)",
    ],
  },
];
```

**3.** Ahora la función de decisión, y esta es la interfaz que van a llamar lecciones posteriores, así que la forma importa más que la regla de puntuación que lleva adentro. `chooseVenue` toma el perfil, recorre las plataformas y junta rechazos a medida que avanza. Lee primero el camino de falla: cuando nada sobrevive, lanza un error con cada rechazo listado, porque una herramienta de decisión que devuelve un valor por defecto silencioso cuando la respuesta es "tu token no puede lanzarse en ningún lado" es peor que ninguna herramienta.

```typescript
// choose-venue.ts: SPROUT's launch decision, derived from R6 instead of vibes.
import { VENUES, Venue, VenueId, Defense } from "./venues";

/** The two prior artifacts, reduced to what a venue choice actually needs. */
export interface SproutProfile {
  /** which token program the mint lives under; SPROUT is Token-2022 */
  baseTokenProgram: "spl" | "token2022";
  /** the final routable extension set from routability-report.ts (R6) */
  extensions: string[];
  /** the threshold derived in derive-graduation.ts, in SOL; printed as the report's derived-reference line */
  graduationThresholdSol: number;
  /** true if the launch needs the community to buy before bots do */
  wantsAntiSnipe: boolean;
}

export interface Rejection {
  venue: VenueId;
  reason: string;
}

export interface LaunchDecision {
  venue: VenueId;
  programId: string;
  graduationAmm: string;
  defense: Defense;
  thresholdSol: number | null;
  seeds: string[];
  rejected: Rejection[];
}

const POWER_EXTENSIONS = ["TransferHook", "PermanentDelegate", "DefaultAccountState", "ConfidentialTransferMint"];

/** Can this venue hold the mint at all, and will its graduation AMM accept it? */
export function venueAcceptsMint(venue: Venue, profile: SproutProfile): Rejection | null {
  // Gate 0, before any extension talk: the venue's create path must be able to
  // mint under the base token program at all. This is the check last lesson's
  // IDL read made unavoidable: pump pins classic SPL and mints the token
  // itself, so a Token-2022 fee mint like SPROUT can never exist there.
  if (profile.baseTokenProgram === "token2022" && venue.baseTokenProgram === "spl") {
    return {
      venue: venue.id,
      reason: "create path mints classic SPL only; a Token-2022 mint (SPROUT carries TransferFeeConfig) cannot exist on this venue",
    };
  }
  const extensions = profile.extensions;
  if (extensions.includes("TransferHook") && !venue.acceptsTransferHook) {
    return {
      venue: venue.id,
      reason: `${venue.graduationAmm} rejects TransferHook; pool creation reverts at migration`,
    };
  }
  const otherPower = extensions.filter((e) => e !== "TransferHook" && POWER_EXTENSIONS.includes(e));
  if (otherPower.length > 0) {
    return {
      venue: venue.id,
      reason: `${venue.graduationAmm} has no documented acceptance for ${otherPower.join(", ")}`,
    };
  }
  return null;
}

export function chooseVenue(profile: SproutProfile): LaunchDecision {
  const rejected: Rejection[] = [];
  const eligible: Venue[] = [];

  for (const venue of VENUES) {
    const ammVerdict = venueAcceptsMint(venue, profile);
    if (ammVerdict) {
      rejected.push(ammVerdict);
      continue;
    }
    if (profile.wantsAntiSnipe && venue.defenses.every((d) => d === "none")) {
      rejected.push({
        venue: venue.id,
        reason: "no first-party anti-snipe defense; you would be building one yourself",
      });
      continue;
    }
    eligible.push(venue);
  }

  if (eligible.length === 0) {
    throw new Error(
      `No venue survives SPROUT's extension set [${profile.extensions.join(", ")}].\n` +
        rejected.map((r) => `  ${r.venue}: ${r.reason}`).join("\n") +
        "\nChange the token or change the requirement. There is no third option.",
    );
  }

  // TODO (yours): this tie-break prefers curve control. Justify it or replace it.
  const winner = eligible.reduce((best, v) => (v.curveSegments > best.curveSegments ? v : best), eligible[0]);
  const defense = winner.defenses.find((d) => d !== "none") ?? "none";

  return {
    venue: winner.id,
    programId: winner.programId,
    graduationAmm: winner.graduationAmm,
    defense,
    // A configurable threshold has NO number to inherit: pump's derived 85 is
    // pump's, and printing it under DBC would be the repeated-number mistake.
    thresholdSol: winner.threshold.kind === "configurable" ? null : (winner.threshold.sol ?? null),
    seeds: winner.seededAtGraduation,
    rejected,
  };
}
```

**4.** Después la mitad del reporte y el criterio. La lista `seeds` no es decoración: es la línea de "qué se siembra" que pide la evaluación, impresa desde el registro de plataforma para que no pueda desviarse de la plataforma que en verdad elegiste.

```typescript
// choose-venue.ts, continued.
export function renderDecision(profile: SproutProfile, decision: LaunchDecision): string {
  const lines = [
    "# SPROUT launch decision",
    "",
    `Extension set (R6): ${profile.extensions.join(", ")}`,
    `Venue:              ${decision.venue}  (${decision.programId})`,
    `Graduation AMM:     ${decision.graduationAmm}`,
    `Anti-snipe:         ${decision.defense}`,
    `Threshold:          ${decision.thresholdSol !== null ? `${decision.thresholdSol} SOL` : "configurable: you set migration_quote_threshold; there is no venue default to inherit"}`,
    `Derived reference:  ~${profile.graduationThresholdSol} SOL (last lesson's pump-constants derivation; on a configurable venue it is your starting anchor, never an inherited default)`,
    "",
    "Seeded at graduation:",
    ...decision.seeds.map((s) => `  - ${s}`),
    "",
    "Rejected:",
    ...decision.rejected.map((r) => `  - ${r.venue}: ${r.reason}`),
  ];
  return lines.join("\n");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const sprout: SproutProfile = {
    baseTokenProgram: "token2022",
    extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
    graduationThresholdSol: 85,
    wantsAntiSnipe: true,
  };
  const decision = chooseVenue(sprout);
  console.log(renderDecision(sprout, decision));

  if (decision.defense === "none") {
    console.error("\nGATE FAIL: wantsAntiSnipe is set but the chosen venue ships no defense.");
    process.exit(1);
  }
  console.log("\nAll gates pass.");
}
```

**5.** Córrelo y luego demuestra que las barreras son reales.

```bash
npx tsx sprout-launch/choose-venue.ts
```

Deberías ver `Venue: dbc`, `Anti-snipe: alpha-vault-window`, una línea de umbral que dice configurable en vez de tomar prestado un número, tres líneas de siembra, tres rechazos que nombran todos el camino de create de SPL clásico, y `All gates pass` con salida 0.

Fíjate en que esta respuesta no es la respuesta de la lección pasada, y la diferencia es el punto entero de ampliar el encuadre. `derive-graduation.ts` hacía una pregunta, "qué AMM puede sostener legalmente el mint de SPROUT," comparaba dos candidatos y aterrizaba en Raydium CP-Swap, que es donde se gradúa LaunchLab. Este script hace dos preguntas más encima: "si el propio camino de create de la plataforma puede acuñar siquiera un token de comisión de Token-2022," que el pin a SPL clásico de LaunchLab reprueba igual que el de pump, y "si la plataforma sobreviviente defiende los primeros slots." DBC es lo que sobrevive a las dos, lo que mueve el destino a DAMM. Ninguna de las dos corridas está equivocada. La primera respondió una pregunta más estrecha con una lista de candidatos más estrecha, y una decisión que cambia cuando agregas un eje es una decisión que sí estaba escuchando.

Ahora hazlo fallar de tres formas, porque un checkpoint que no puede fallar nunca fue un checkpoint. Agrega `"TransferHook"` a `extensions` y nota que la fila de DBC sobrevive mientras los otros tres rechazos siguen en pie. Pon `wantsAntiSnipe: false` y fíjate en que pump y LaunchLab NO vuelven al conjunto elegible; la barrera del programa base los rechazó antes de que la barrera de defensa llegara a correr, y ninguna bandera de preferencia puede conjurar un mint de Token-2022 sobre una plataforma de SPL clásico. Para ver la barrera de defensa morder de verdad, voltea el perfil entero a un token clásico simple (`baseTokenProgram: "spl"`, `extensions` vacío, `wantsAntiSnipe: true`) y mira cómo pump y LaunchLab quedan rechazados solo por la defensa. Por último pon `extensions` en `["PermanentDelegate"]` con el perfil de Token-2022 y mira el throw: ninguna plataforma sobrevive, cada rechazo impreso, ningún valor por defecto devuelto. Vuelve a poner los valores reales de SPROUT cuando termines.

![Un diagrama de flujo de choose-venue.ts que pasa cada plataforma por una barrera de extensiones y una barrera de preferencia, con los rechazos recogidos aparte, un desempate y una barrera de salida para las elecciones sin defensa.](assets/v09-flowchart.png)

## Challenge

La mitad en solitario es el juicio que la herramienta deliberadamente no hace por ti, y sale como escritura, no como código.

Primero, reemplaza el desempate. La línea marcada TODO prefiere la plataforma elegible que exponga más segmentos de curva, que es una regla defendible y no la única. Escribe la regla que de verdad crees, en código, y pon un comentario encima que diga para qué optimiza y qué sacrifica. Candidatas que tienes material para argumentar: preferir la defensa más fuerte, preferir la plataforma cuya quema de LP es total y no parcial, preferir la mayor superficie de integración, preferir la plataforma cuyo esquema de comisión no puede cambiar bajo tus pies. Cualquiera de esas le gana a "más segmentos" para algunos lanzamientos.

Segundo, escribe la decisión de lanzamiento de SPROUT como prosa, y haz que sobreviva a una lectura hostil. Cuatro cosas tienen que estar adentro, y el criterio de aceptación es el del brief. Nombra la plataforma y la única defensa anti-snipe que estás tomando. Justifica por qué el conjunto enrutable de R6 permite el AMM de graduación de esa plataforma, nombrando las extensiones y la allowlist en vez de señalarlas de lejos. Di en una línea qué siembra la graduación para ti: el par, el mecanismo de precio que fija la cotización de apertura, y la regla de destino final del LP incluyendo si la quema es total. Después di lo que habrías tenido que sembrar a mano en su lugar, en las mismas unidades, para que el servicio real del launchpad sea una cantidad y no una vibra.

Tercero, y esta es la que separa una decisión de una preferencia: escribe el párrafo que te haría cambiar de opinión. Nombra el hecho específico que, si resultara ser otro, da vuelta tu elección de plataforma. El mío sería el soporte de transfer hook que le atribuí a DBC. Todo mi orden se apoya en que eso sea un camino documentado y funcionando hoy, y no una frase de roadmap, y si lo verificas y encuentras que es una bandera de config que nadie ha llevado en producción a través de una migración, toda la comparación se rebaraja y el diseño de SPROUT se vuelve más simple a la fuerza. Encuentra el tuyo. Una decisión cuyo autor no puede nombrar su condición de ruptura es solo una preferencia.

Un pedido antes de que cierres la carpeta. Cada número de plataforma de esta lección está fechado el 2026-08-22 y leído de una página de docs o de un repositorio, no de un lanzamiento que yo haya corrido: el umbral JustSendit de 85 SOL, el reclamo del 10% de la Fee Key, la división 90/10 del LP, la hoja de comisiones de Genesis, las dos constantes de Meteora. La economía de los launchpads se pudre más rápido que casi cualquier otra cosa de este curso, porque la división de comisiones es el producto. Si tus propias lecturas no coinciden con las mías, publica la afirmación exacta y lo que encontraste en el canal de feedback del curso. El hábito que estás construyendo no es "conocer las plataformas". Es "volver a derivar la tabla de plataformas antes de cada lanzamiento", y un aprendiz que detecta un número viejo es ese hábito funcionando en voz alta.

Ya elegiste dónde se gradúa SPROUT y qué siembra eso. Ahora el otro lado de la distribución: meter el token en miles de manos sin una factura de rent tan grande como una casa.
