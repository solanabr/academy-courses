# El camino de Rust: JSON-RPC crudo

## Resumen

m08-l2 llevó a producción las lecturas de TypeScript: el panel de Vercel renderiza el panel en vivo de Solana y el edge worker sirve un snapshot de la blockchain cacheado en KV, los dos re-entregados a sus URLs existentes con el presupuesto de backoff aplicado. Lo que deja exactamente una superficie ciega a la blockchain: el poller de Docker de M6, el de Rust. Hoy aprende a mirar slots y saldos, y lo hace sin un solo crate nuevo, porque la blockchain habla un protocolo para el que ya tienes todas las herramientas. El contrato de repliegue, en voz alta: esta es una lección guiada pero liderada por quien aprende. Yo trabajo un POST y un struct de respuesta para ti. Tú escribes la sonda getSlot, el enum de errores completo y el cableado de `/status` desde contratos, y la extensión del challenge es en solitario. Eso es un escalón hacia arriba en autonomía respecto de los esqueletos de completion de m06-l1, a propósito, porque cada línea de esta lección es una habilidad que ya ejercitaste.

## No hay magia debajo

Haz esto primero, antes de cualquier teoría. Abre `crates/pulse-pollerd/Cargo.toml` y haz una sola edición a una línea que está ahí sentada desde m06-l1:

```toml
reqwest = { version = "0.13", features = ["json"] }
```

Esa no es una dependencia nueva. Es una feature flag sobre un crate que ya entregaste dos veces, y después de m05-l2 puedes leerla: optar por la integración con serde de reqwest, los helpers `.json()` de request y response, que el poller nunca necesitó mientras solo le importaban los códigos de estado. Frescura de los pins: reqwest está en 0.13.4 en crates.io al 2026-09-02, thiserror 2.0.20, serde_json 1.0.151; tus dígitos de patch pueden ser más altos y eso está bien.

Ahora crea `crates/pulse-pollerd/examples/chain.rs`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "getBalance",
        "params": ["Vote111111111111111111111111111111111111111"]
    });
    let resp: serde_json::Value = reqwest::Client::new()
        .post("https://api.mainnet.solana.com")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    println!("{resp:#}");
    Ok(())
}
```

Córrelo: `cargo run -p pulse-pollerd --example chain`. Esto es lo que mainnet me devolvió cuando lo corrí mientras escribía esto, textualmente:

```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "context": {
      "apiVersion": "4.2.1",
      "slot": 443693778
    },
    "value": 1
  }
}
```

Mira lo que acaba de pasar. Leíste un saldo de Solana mainnet, desde Rust, y todo el cliente fue un POST que podrías haber tipeado de memoria. `serde_json::json!` construyó el request, un macro que usas desde la lección de config. `reqwest` lo transportó, el crate que viene sondeando tu flota desde m05-l3 en forma bloqueante y desde m06-l1 en forma async. serde parseó la respuesta. No hay ningún crate de Solana en tu árbol y nunca lo habrá en este curso, porque no hay magia debajo: la blockchain habla JSON simple sobre HTTP. Todo lo que aprendiste sobre parsear y sobre errores ES código de cliente de Solana. Esa oración es la lección; el resto de este archivo es hacerla de grado producción.

(El `value: 1` es real, de paso. Esa dirección es el programa de voto, y su cuenta de programa tiene exactamente 1 lamport. Vas a poner una dirección que de verdad te importe durante el lab.)

### El sobre, nombrado

La forma que acabas de imprimir es JSON-RPC 2.0, una convención de llamada a procedimiento remoto de 2010 que Solana adoptó por completo. El sobre del request tiene cuatro campos y escribiste todos: `jsonrpc` es el string literal `"2.0"`, `id` es cualquier valor elegido por el cliente que el servidor devuelve como eco para que puedas emparejar respuestas con requests, `method` nombra el procedimiento, y `params` es un array ordenado de argumentos, vacío cuando el método no toma ninguno. El sobre de la respuesta devuelve como eco `jsonrpc` e `id` y después carga exactamente uno de dos miembros: `result` cuando la llamada tuvo éxito, o `error` cuando falló. Agárrate de ese uno-o-el-otro. Está por importar más que nada más en esta lección.

![Un request de JSON-RPC carga jsonrpc, id, method y params, y la respuesta devuelve el id como eco junto con un miembro result o un miembro error.](assets/v01-diagram.webp)

¿Por qué existe `id`, si el servidor solo lo devuelve como eco? Porque JSON-RPC se diseñó para clientes que hacen pipelining: disparar cinco requests por una conexión, recibir cinco respuestas en el orden en que el servidor las termine, y emparejar cada respuesta con su pregunta por id. El protocolo incluso permite batching, un array de sobres de request respondido por un array de respuestas en un solo viaje de ida y vuelta. Nuestro poller manda un request a la vez y lo espera, así que un `id: 1` constante está perfectamente bien, y quiero que notes que esta es una decisión que acabas de poder tomar, consciente, porque eres dueño del sobre. Una librería cliente la habría tomado por ti, invisiblemente, junto con otras cincuenta. Ser dueño del sobre significa que toda la superficie del protocolo es tuya para usarla o ignorarla, y también significa que cuando con el tiempo quieras pipelining o batching, nadie te lo pre-construyó: ese es el trade-off, visible desde el primerísimo campo.

Un aparte sobre modales mientras andamos haciendo clientes HTTP a mano. Durante la investigación de este curso sondeé la API de crates.io por los pins de versión de arriba, y un `curl` pelado sin User-Agent volvió como un 403 con cuerpo vacío, no JSON. Lo reverifiqué hoy; sigue haciéndolo. El registro con el que habla tu propio toolchain rechaza clientes que no se identifican. Las APIs públicas hacen cumplir la etiqueta, y la semana en que empiezas a escribir clientes HTTP crudos contra ellas es la semana en que eso deja de ser trivia. El endpoint público de Solana tiene su propia versión de esto: los rate limits que aprendiste a respetar en el trabajo de backoff de TS aplican también a este poller.

### Structs tipadas: el pago de serde

`serde_json::Value` estuvo bien para el ejemplo, pero el poller no puede entregarse sobre eso. Meter la mano en un `Value` con claves de string es exactamente la pesca sin tipos que m02-l2 te enseñó a rechazar en TypeScript. Esta es la misma disciplina, tercera aparición: zod parsea JSON desconocido en la frontera y lo convierte en un tipo o falla a gritos, serde lo hizo para tu archivo de config en m05-l1, y ahora lo hace para una blockchain. "Parse, don't validate", ahora apuntado a mainnet.

Las structs para esa respuesta de getBalance:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RpcContext {
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
pub struct BalanceResult {
    pub context: RpcContext,
    pub value: u64,
}
```

Recorre los campos contra el JSON que imprimiste. `result` es un objeto con dos miembros, así que `BalanceResult` tiene dos campos. `context` te dice en qué slot respondió el nodo, metadata útil para un monitor, y `value` es el saldo en lamports. El string `apiVersion` adentro de context no tiene campo en el struct, y eso es deliberado: serde ignora campos desconocidos por default, así que tipas solo lo que consumes y la respuesta puede crecer sin romperte. Y fíjate en lo que `value: u64` está haciendo calladamente. En el panel de TypeScript de la lección pasada, los lamports forzaron la ceremonia de bigint porque los números de JavaScript pierden precisión pasado 2^53. El `u64` de Rust tiene el rango completo de forma nativa. El workaround era un problema de JavaScript; no lo importes donde el lenguaje no tiene la enfermedad.

La falla clásica de primer intento acá es deserializar directo a la forma del value, apuntar un `u64` pelado a `result` y ver fallar el parseo, porque `result` envuelve context y value y aplanarlo a mano no sobrevive el contacto con los bytes reales. Cuando un campo se renombra o falta, todo este enfoque hace fallar el parseo como un valor `Result` que ruteas. No una sorpresa en runtime tres funciones después. Esa propiedad está por volverse la columna vertebral del diseño de errores.

![Cada miembro del result de getBalance mapea a un campo del struct excepto apiVersion, que serde ignora por diseño.](assets/v02-annotated-code.webp)

### Cuatro formas de fallar, un enum

Acá va la trampa que separa a los lectores de blockchain de juguete de los de verdad, y te la puedo mostrar en vivo. Manda un nombre de método mal tipeado a mainnet y mira los dos canales:

```bash
curl -s -o /dev/null -w "%{http_code}\n" https://api.mainnet.solana.com \
  -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBalanceTypo","params":[]}'
# 200
```

HTTP dijo 200. El cuerpo dijo:

```json
{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}
```

Un error de JSON-RPC llega adentro de un éxito de HTTP. La capa de transporte hizo su trabajo perfectamente: entregó una respuesta bien formada que resulta que dice "error" en el vocabulario propio del sobre. Un poller que revisa `response.status()` y nada más va a marcar una lectura fallida como sana para siempre, y un monitor que reporta mal es peor que ningún monitor, porque la gente le cree. Así que el modelo de fallas para una sonda de blockchain tiene cuatro planos distintos, y merecen cuatro nombres distintos:

1. **Transporte.** El request nunca se completó: falla de DNS, conexión rechazada, timeout. reqwest te entrega su propio error.
2. **Estado HTTP.** Llegó una respuesta pero el estado no es 2xx: un 429 por rate limiting, un 502 de un proxy muriéndose.
3. **Error de RPC.** HTTP tuvo éxito y el miembro `error` del sobre está poblado. El plano trampa.
4. **Forma.** HTTP tuvo éxito, no hay miembro error, pero el JSON no parsea a tus tipos: un campo renombrado, un sobre equivocado, un cambio de API.

Cuatro planos, cuatro remediaciones. Un error de transporte podría significar tu red; reintenta con backoff. Un 429 significa bajá el ritmo. Un error de RPC significa que tu request está mal; ningún retry va a arreglar un método mal tipeado. Un error de forma significa que el mundo cambió y tus tipos necesitan un mantenedor. En m09-l2, cuando la estación reciba logging estructurado, cada plano se vuelve un nombre grepeable, y la diferencia entre "el RPC estaba caído" y "lo estábamos parseando mal" se vuelve un grep en vez de una tarde.

Este es el pago de thiserror que armaron las lecciones de errores de M5. Un enum, una variante por plano, y toda la taxonomía es un tipo que el compilador hace cumplir:

![Una respuesta de sonda pasa cuatro verificaciones ordenadas, y cada verificación que falla sale a su propia variante de error antes de un éxito tipado.](assets/v03-flowchart.webp)

Una trampa que desarmar antes del lab, porque vive en el plano uno. El cliente por default de reqwest no pone ningún timeout general de request. Ninguno. Un nodo de RPC colgado no da error; te mantiene la conexión abierta, y todo lo que la espera, indefinidamente. Tu loop de poll de m06-l1 ya aplica un timeout por request a las sondas de la flota, y la disciplina de m06-l1 aplica sin cambios acá: cada sonda de blockchain carga un timeout explícito, porque un poller empacado en una lectura es una estación con los ojos cerrados.

### La pregunta del crate, respondida con honestidad

Pregunta justa a esta altura: el ecosistema de Solana en Rust entrega crates cliente, entonces ¿por qué un curso de Solana te enseña a hacer JSON-RPC a mano en vez de agarrar `solana-client`?

Empieza por la asimetría que quizá ya estés sintiendo. La lección pasada las superficies de TypeScript recibieron una librería cliente, kit, y nadie hizo nada a mano. Esta lección la superficie de Rust va directo al cable. Eso no es inconsistencia; es la misma decisión de dimensionar bien aterrizando distinto en terreno distinto. Del lado de TS, kit es el cliente canónico del ecosistema, el curso hermano de frontend construye toda su práctica sobre él, y el trabajo de este curso era pasarte hablando ese dialecto. Del lado de Rust también hay un camino tipado, y lo vamos a nombrar como corresponde en un momento, pero alguien que lee fundamentos y hace dos lecturas en un temporizador recibe acá algo mejor que una librería: un lab donde la disciplina de serde de M5 y la taxonomía de errores del mismo módulo dejan de ser ejercicios y se vuelven un cliente de blockchain que funciona. El camino hecho a mano paga matrícula doble. Lee la blockchain Y demuestra que todo lo que aprendiste sobre parsear y sobre errores era código de cliente de Solana desde el principio.

Por lo que `solana-client` es. Es el cliente que trae de todo: además de las lecturas de RPC empaqueta un stack completo de transporte para envío de transacciones, clientes de QUIC y UDP, el cliente de TPU que habla directo con los líderes de bloque, cacheo de conexiones, la maquinaria del streamer. Ese es el equipamiento de un sistema que dispara transacciones a validadores. Nuestro poller hace dos preguntas en un temporizador. Alguien que lee fundamentos nunca llama a nada de ese transporte, y traerlo significa compilarlo, auditarlo y montarse a su churn de releases para nada.

Ahora la parte donde mantengo honesto el argumento, porque hay una versión más perezosa que alguien que aprende con cuidado va a cazar. La versión perezosa dice "usa `solana-rpc-client` en cambio, es el flaco", y agita conteos de dependencias. Cuenta tú mismo: al momento de la investigación de este curso, `solana-client` 4.2.2 tiene 29 dependencias directas más una dev-dependency, y `solana-rpc-client` 4.2.2 tiene 35. El crate "flaco" tiene MÁS deps directas por conteo crudo; arrastra una pila de crates de tipos livianos que suman. Lo que no arrastra es nada de la maquinaria de transporte de QUIC, TPU o streamer. Así que la afirmación defendible está acotada: flaco en alcance de transporte, no en conteo de deps. Argumenta desde el eje que de verdad diferencia. Si tu evidencia es un número, alguien va a revisar el número, y si el número está mal tu conclusión correcta se muere con él.

![JSON-RPC hecho a mano, solana-rpc-client y solana-client comparados en dependencias, alcance de transporte, tipado y encaje.](assets/v04-comparison.webp)

Así que el trade-off honesto, completo. JSON-RPC hecho a mano es el piso con transparencia total: cero churn de crates de Solana, y cada línea reusa una habilidad que este curso ya te enseñó. El costo es que TÚ eres dueño del sobre. Métodos nuevos significan structs nuevos. Niveles de commitment, encodings de respuesta, requests en batch: todo manual, todo tuyo. (Nivel de commitment, ya que lo acabo de nombrar: un parámetro opcional que elige qué tan final tiene que ser una respuesta antes de que el nodo te la dé. Aceptamos el default en este curso y dejamos el ajuste donde corresponde, en los docs de abajo.) Nadie actualiza tus tipos cuando el RPC evoluciona; tus pruebas de forma atrapan la rotura y después un humano, tú, la arregla. Para un poller que hace dos lecturas, ese trade-off es correcto. Un indexador real o un sistema de trading se gradúa a `solana-rpc-client` y a los crates `solana-*` granulares, que es exactamente por qué viven en el cuadro de abajo y no en esta lección.

**Profundiza (el 20%).** la referencia canónica para cada método de JSON-RPC que el patrón de esta lección puede alcanzar, params de request, formas de respuesta, niveles de commitment, y la especificación del sobre en sí, es la página de métodos HTTP de RPC de Solana: https://solana.com/docs/rpc/http (verificada en vivo el 2026-09-02). Déjala como bookmark; es la mitad que falta del patrón de hoy, y cuando tus lecturas le queden grandes a lo hecho a mano, el camino tipado es `solana-rpc-client` más los crates `solana-*` granulares, adoptados con la disciplina de lectura de dependencias de m05-l2. Nada en el lab de hoy depende de nada de eso.

## Lab: pollerd gana sondas de la blockchain

La construcción. Al final, `/status` responde con los targets de la flota que ya reporta más un bloque chain, y la imagen de GHCR se re-entrega sin que toques un solo archivo de ops.

1. **Manifest, treinta segundos.** Ya activaste la feature `json` de reqwest en el hazlo-primero. Agrega una suscripción a `crates/pulse-pollerd/Cargo.toml`:

   ```toml
   thiserror = { workspace = true }
   ```

   La versión vive en la raíz del workspace donde m05-l2 la declaró (2.0.20). Lee todo el bloque `[dependencies]` cuando termines y déjalo registrar: pulse-engine, serde, serde_json, tokio, axum, reqwest, thiserror. Todos preceden a esta lección. El workspace no gana nada nuevo hoy; ese es el punto, y sigue siendo cierto hasta el push final.

2. **El enum de errores, tuyo, desde un contrato.** Crea `crates/pulse-pollerd/src/chain.rs` y escribe `ProbeRpcError` tú mismo con `#[derive(Debug, Error)]`. El contrato, una variante por plano de la sección de teoría:

   - `Transport` envuelve `reqwest::Error`. Usa `#[from]`, el músculo de conversión de m05-l2, así `?` eleva las fallas de reqwest a tu tipo automáticamente.
   - `Status` carga el código ofensor como un `u16`.
   - `Rpc` es una variante de struct que carga `code: i64` y `message: String` levantados del objeto error del sobre.
   - `Shape` envuelve `serde_json::Error`, también con `#[from]`.

   Escribe un mensaje `#[error("...")]` para cada uno que te gustaría leer en un log a las 2am, y dale al enum un método chico: `pub fn plane(&self) -> &'static str`, que devuelva `"transport"`, `"http_status"`, `"rpc"` o `"shape"`. Cuatro strings estáticos. Ese método parece nada hoy; es el nombre grepeable sobre el que se va a apoyar el logging estructurado de m09-l2.

3. **La línea de transporte y la sonda trabajada.** Acá va mi mitad del contrato de repliegue: la llamada genérica, el parseo separado para poder testearlo, y el getBalance trabajado. Léelo, y después tipéalo en `chain.rs`:

   ```rust
   use std::time::Duration;

   use serde::Deserialize;

   const RPC_TIMEOUT: Duration = Duration::from_secs(5);

   #[derive(Debug, Deserialize)]
   struct RpcErrorObject {
       code: i64,
       message: String,
   }

   fn parse_rpc_response<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, ProbeRpcError> {
       let envelope: serde_json::Value = serde_json::from_str(text)?;
       if let Some(err) = envelope.get("error") {
           let err: RpcErrorObject = serde_json::from_value(err.clone())?;
           return Err(ProbeRpcError::Rpc { code: err.code, message: err.message });
       }
       let result = envelope.get("result").cloned().unwrap_or(serde_json::Value::Null);
       Ok(serde_json::from_value(result)?)
   }

   pub async fn rpc_call<T: serde::de::DeserializeOwned>(
       client: &reqwest::Client,
       url: &str,
       method: &str,
       params: serde_json::Value,
   ) -> Result<T, ProbeRpcError> {
       let body = serde_json::json!({
           "jsonrpc": "2.0",
           "id": 1,
           "method": method,
           "params": params,
       });
       let resp = client
           .post(url)
           .timeout(RPC_TIMEOUT)
           .json(&body)
           .send()
           .await?;
       let status = resp.status();
       if !status.is_success() {
           return Err(ProbeRpcError::Status(status.as_u16()));
       }
       let text = resp.text().await?;
       parse_rpc_response(&text)
   }

   pub async fn get_balance(
       client: &reqwest::Client,
       url: &str,
       address: &str,
   ) -> Result<BalanceResult, ProbeRpcError> {
       rpc_call(client, url, "getBalance", serde_json::json!([address])).await
   }
   ```

   Agrega los structs `RpcContext` y `BalanceResult` de la sección de teoría arriba de estos. Después recorre las costuras, porque dos decisiones acá son estructurales. Primero, `parse_rpc_response` toma un `&str`, no una respuesta de red, lo que hace toda la taxonomía de fallas testeable con fixtures de string y sin red; vas a explotar eso en el paso 4. Segundo, el parseo pasa por `serde_json::Value` antes que por tu struct tipado, así que el miembro error se revisa antes de que se interprete el miembro result, y una respuesta sin ninguno de los dos cae hasta un error `Shape` cuando `Null` se niega a convertirse en tu tipo. Los operadores `?` hacen el ruteo en silencio: una falla de send se eleva a `Transport`, una falla de from_str o from_value a `Shape`, las dos vía las conversiones `#[from]` que escribiste en el paso 2. La taxonomía no es un comentario. Es el sistema de tipos haciendo la clasificación.

   Honestamente, un `rpc_call<T>` genérico más un enum es la mayor parte de lo que es un SDK cliente. Todo lo demás son wrappers de conveniencia, y ahora te toca escribir dos.

4. **Tu sonda y tu demostración.** Escribe `get_slot` tú mismo. Su sobre de resultado no es un objeto para nada: `getSlot` devuelve un número pelado como `result`, así que toda la función es `rpc_call` con `T = u64`, método `"getSlot"`, y params `json!([])` vacíos. Una revelación para sentir: tu genérico maneja una forma de respuesta completamente distinta con cero cambios de transporte.

   Después las pruebas, en un `#[cfg(test)] mod tests` al fondo de `chain.rs`. La vara de aceptación es un camino feliz de deserialización y un fixture malformado por sonda. Dos fixtures trabajados de mi parte, capturados en vivo de mainnet el 2026-09-02, para que tus pruebas afirmen contra bytes reales:

   ```rust
   #[test]
   fn balance_happy_path() {
       let fixture = r#"{"jsonrpc":"2.0","result":{"context":{"apiVersion":"4.2.1","slot":443692897},"value":1},"id":1}"#;
       let parsed: BalanceResult = parse_rpc_response(fixture).expect("fixture parses");
       assert_eq!(parsed.value, 1);
       assert_eq!(parsed.context.slot, 443692897);
   }

   #[test]
   fn error_in_a_200_routes_to_rpc_variant() {
       let fixture = r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
       let parsed = parse_rpc_response::<u64>(fixture);
       assert!(matches!(parsed, Err(ProbeRpcError::Rpc { code: -32601, .. })));
   }
   ```

   El resto lo escribes tú: un camino feliz de slot (`result` es `443692896`, afirma que el número llega), un fixture de balance malformado (borra el miembro `context` y afirma `Err(ProbeRpcError::Shape(_))` con `matches!`), y un fixture sin result (un sobre sin ninguno de los dos miembros, misma afirmación). `cargo test -p pulse-pollerd`, en verde. Fíjate en lo que NO necesitaste: una red, un servidor mock, un runtime async en las pruebas. La costura del `&str` compró todo eso.

![Tres métodos de RPC devuelven tres results con formas distintas, y la misma llamada genérica maneja cada uno cambiando solo el parámetro de tipo.](assets/v05-comparison.webp)

5. **Cabléalo a `/status`, desde un contrato.** El poller ahora mismo sirve un solo mapa de targets de la flota. La forma objetivo después de este paso, el mismo JSON que curlea la barrera de verificación:

   ```json
   {
     "targets": { "…": "everything /status already reported" },
     "chain": {
       "slot": 443693778,
       "balance_lamports": 1,
       "watched_address": "Vote111111111111111111111111111111111111111",
       "last_error": null,
       "last_poll": 1788336000
     }
   }
   ```

   Los tipos del contrato, en `chain.rs`:

   ```rust
   use std::sync::{Arc, Mutex};

   use serde::Serialize;

   #[derive(Clone, Default, Serialize)]
   pub struct ChainStatus {
       pub slot: Option<u64>,
       pub balance_lamports: Option<u64>,
       pub watched_address: String,
       pub last_error: Option<String>,
       pub last_poll: u64,
   }

   pub type ChainState = Arc<Mutex<ChainStatus>>;
   ```

   El cableado es tuyo, y es la arquitectura de m06-l1 repetida en miniatura, así que constrúyelo por analogía, no desde cero. Escribe un `async fn chain_loop(chain: ChainState)` que haga su propio `reqwest::Client`, tictaquee un `tokio::time::interval` cada 30 segundos, espere `get_slot` y `get_balance` para tu dirección vigilada, y después, con los dos resultados ya en mano, tome el lock una vez y escriba los campos: los éxitos a `slot` y `balance_lamports`, cualquier falla a `last_error` como `format!("{}: {e}", e.plane())` para que el nombre del plano encabece el mensaje. La regla del lock de m06-l1 aplica textualmente y no me voy a disculpar por repetirla: los dos awaits terminan ANTES de que se tome el lock, nunca sostengas la guarda cruzando un await. En `main`, spawnea `chain_loop` al lado del spawn existente de `poll_loop`, después junta los dos estados en un struct chico `AppState { targets, chain }` (deriva `Clone`), cambia el `.with_state` del router a ese, y actualiza `status_handler` para que tome el lock de cada mapa brevemente, clone snapshots, y devuelva un `StatusResponse { targets, chain }`. Cambia `WATCHED_ADDRESS` por una dirección que te importe, o quédate con el programa de voto y su lamport solitario. `cargo run -p pulse-pollerd`, después `curl -s localhost:8080/status` y lee el primer bloque chain de tu estación.

![Un nuevo loop de chain se suma al loop de poll de la flota existente, y cada uno escribe su propio estado compartido que un solo endpoint de status reporta junto.](assets/v06-diagram.webp)

6. **Rómpelo a propósito.** La aceptación para la taxonomía no es que compile; es que la falla se degrade en vez de detonar. Cambia la URL del RPC por algo que no se pueda resolver, `https://rpc.invalid`, y corre el poller. El proceso tiene que quedarse arriba, `/status` tiene que seguir sirviendo, y en un arranque fresco el bloque chain debería leerse así:

   ```json
   {
     "targets": { "…": "still serving, unchanged" },
     "chain": {
       "slot": null,
       "balance_lamports": null,
       "watched_address": "Vote111111111111111111111111111111111111111",
       "last_error": "transport: error sending request",
       "last_poll": 1788336030
     }
   }
   ```

   El mensaje exacto después de `transport:` va a variar con tu OS y tu resolver, y eso está bien; el nombre del plano adelante es la parte que tu código garantiza. Si lo rompes contra un poller que ya está corriendo en cambio, `slot` y `balance_lamports` mantienen sus últimos valores honestos mientras `last_error` se llena, que es precisamente el comportamiento que quieres que tenga un monitor: desactualizado-pero-etiquetado le gana a vacío. Pon la URL real de vuelta y mira cómo el próximo tick lo sana: `last_error` vuelve a `null`, el número de slot retoma la subida. Que una URL equivocada te cueste un campo en una respuesta JSON en vez de un proceso es todo el argumento del paso 2, demostrado en noventa segundos.

7. **La re-entrega.** Commitea, empuja, y no hagas nada más. El pipeline de M6 levanta el commit, corre tus pruebas incluyendo los fixtures nuevos, construye el mismo Dockerfile multi-stage de cargo-chef, y empuja la imagen a GHCR, porque el workspace, el Dockerfile y el workflow están intactos desde m06-l4. Esa es la demostración hacia la que viene caminando la lección: el código que aterriza adentro de un sistema que ya se entrega hereda su entrega. Cuando la corrida esté en verde, demuéstralo de punta a punta desde afuera, como lo haría un desconocido:

   ```bash
   docker pull ghcr.io/<you>/pulse-pollerd:latest
   docker run --rm -p 8080:8080 ghcr.io/<you>/pulse-pollerd:latest
   # in another terminal:
   curl -s localhost:8080/status
   ```

   Targets de la flota, más un bloque chain, con un slot de mainnet en vivo adentro, servido por una imagen a la que cualquier máquina en la tierra le puede hacer pull. La tercera superficie de la estación acaba de recibir sus ojos.

![Un solo git push fluye por el pipeline de CI intacto hacia una imagen fresca del registro que quien aprende hace pull, corre y curlea.](assets/v07-flowchart.webp)

## Challenge

Solo. Agrega una sonda `get_health`. `getHealth` responde con el string `"ok"` como todo su `result`, una tercera forma que tus structs no conocen: no un objeto, no un número, un string pelado. Si lo ruteas por `rpc_call`, la función son dos líneas, y esa es la prueba de si entendiste el diseño: la taxonomía y el genérico tienen que absorber un método nuevo sin una sola edición a la línea de transporte. Sácalo a la superficie en el bloque chain como mejor te parezca, un campo `healthy: Option<bool>` es una respuesta limpia. Escribe las dos pruebas: un fixture de camino feliz que escribas tú mismo en el estilo capturado-en-vivo, y uno malformado. Y piensa la ruta de falla antes de correr nada: un nodo no sano reporta por el miembro error del sobre, lo que significa que la respuesta llega en el plano tres, ya clasificada por código que escribiste en el paso 2, y tu string de `plane()` dice `"rpc"` antes de que hayas leído una sola línea de log. Aceptación: `cargo test -p pulse-pollerd` en verde con tus dos fixtures nuevos, y `/status` mostrando el campo de health contra el endpoint real. Tu edge worker de TS viene sondeando este mismo método desde m07-l1 (el worker de Rust clasifica; no hace llamadas salientes); ahora sabes precisamente qué estaba haciendo esa sonda.

## Checkpoint

Lo que ahora puedes hacer, concretamente: leer estado de Solana mainnet en Rust con un POST de cinco líneas construido enteramente con crates que ya entregaste; tipar una respuesta de RPC para que un campo renombrado sea un `Result` ruteado, no una sorpresa en runtime; clasificar cada forma en que una lectura de blockchain puede fallar en cuatro planos y decir cuál de ellos se esconde adentro de un HTTP 200; y defender la decisión de no-usar-crate-de-Solana en el eje que sobrevive una auditoría, el alcance de transporte, concediendo el eje del conteo de deps que no sobrevive.

La recuperación de 30 segundos antes de cerrar la terminal: ¿por qué una lectura completamente fallida puede llegar como HTTP 200? (JSON-RPC reporta fallas adentro de su propio sobre; el transporte tuvo éxito entregando una respuesta que dice error, así que tienes que revisar el miembro `error`, no solo el estado.) ¿Y el argumento del crate en una oración? (`solana-client` empaqueta un transporte de envío por QUIC y TPU que un poller de dos lecturas nunca llama; ese alcance, no los conteos crudos de dependencias, es el argumento, y los conteos en realidad apuntan al otro lado.)

Un pedido de calibración. Esta lección te entregó el enum de errores como un contrato en prosa en vez de como un esqueleto de código, la primera vez que el curso hace eso. Si escribirlo desde los cuatro bullets se sintió como trabajo de verdad, esa es la dificultad buscada; si se sintió subespecificado y tuviste que adivinar formas, dilo en el feedback, porque m09 se apoya más fuerte en pasos guiados por contrato y quiero que la rampa sea honesta.

La estación ahora lee la blockchain desde cada superficie que tiene, en los dos lenguajes: el panel, el edge worker, y una imagen de Docker con lecturas tipadas de mainnet adentro. Lo que significa que cada afirmación que este curso hizo sobre el stack está ahora demostrada excepto una: que puede ESCRIBIR. La lección que viene es el pago del módulo. Una transferencia firmada en devnet, enmarcada como el chequeo de salud del camino de escritura de la estación, con una caja de honestidad sobre faucets y un validador local en tu bolsillo de atrás para el día en que el faucet esté seco. La sonda que muta.
