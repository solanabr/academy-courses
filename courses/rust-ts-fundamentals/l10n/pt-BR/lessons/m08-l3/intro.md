# O caminho do Rust: JSON-RPC cru

## Resumo

m08-l2 colocou as leituras em TypeScript em produção: o painel no Vercel renderiza o painel ao vivo da Solana e o edge worker serve um snapshot da blockchain em cache no KV, os dois com re-ship nas URLs que já tinham e o orçamento de backoff aplicado. O que deixa exatamente uma superfície cega para a blockchain: o poller em Docker do M6, o de Rust. Hoje ele aprende a observar slots e saldos, e faz isso sem um único crate novo, porque a blockchain fala um protocolo para o qual você já tem todas as ferramentas. O contrato de recuo, em voz alta: esta é uma lição guiada-mas-conduzida-pelo-aprendiz. Eu trabalho um POST e uma struct de resposta para você. Você escreve a sonda getSlot, o enum de erro completo e a fiação do `/status` a partir de contratos, e a extensão do challenge é solo. Isso é um degrau de autonomia acima dos esqueletos de completion do m06-l1, de propósito, porque toda linha desta lição é uma habilidade que você já treinou.

## Nenhuma mágica por baixo

Faça isto primeiro, antes de qualquer teoria. Abra `crates/pulse-pollerd/Cargo.toml` e faça uma única edição numa linha que está sentada ali desde m06-l1:

```toml
reqwest = { version = "0.13", features = ["json"] }
```

Essa não é uma dependência nova. É uma flag de feature num crate do qual você já fez ship duas vezes, e depois do m05-l2 você sabe ler ela: optar pela integração do reqwest com o serde, os helpers `.json()` de request e de response, de que o poller nunca precisou enquanto só se importava com códigos de status. Freshness de pin: o reqwest está em 0.13.4 no crates.io em 2026-09-02, o thiserror em 2.0.20, o serde_json em 1.0.151; os seus dígitos de patch podem estar mais altos e isso é normal.

Agora crie `crates/pulse-pollerd/examples/chain.rs`:

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

Rode: `cargo run -p pulse-pollerd --example chain`. Aqui está o que a mainnet devolveu quando eu rodei isso enquanto escrevia, textualmente:

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

Olhe o que acabou de acontecer. Você leu um saldo da mainnet da Solana, de dentro do Rust, e o cliente inteiro foi um POST que você poderia ter digitado de memória. `serde_json::json!` montou o request, uma macro que você usa desde a lição de config. `reqwest` levou ele, o crate que sonda a sua frota desde m05-l3 na forma blocking e desde m06-l1 na forma async. O serde parseou a resposta. Não existe nenhum crate da Solana na sua árvore e nunca vai existir neste curso, porque não existe mágica por baixo: a blockchain fala JSON puro sobre HTTP. Tudo o que você aprendeu sobre parsing e erros É código de cliente da Solana. Essa frase é a lição; o resto deste arquivo é deixar ela em nível de produção.

(O `value: 1` é real, aliás. Aquele endereço é o vote program, e a conta de programa dele guarda exatamente 1 lamport. Você vai trocar por um endereço com que você realmente se importa durante o lab.)

### O envelope, nomeado

O formato que você acabou de imprimir é JSON-RPC 2.0, uma convenção de chamada remota de procedimento de 2010 que a Solana adotou por inteiro. O envelope de request tem quatro campos e você escreveu todos: `jsonrpc` é a string literal `"2.0"`, `id` é qualquer valor escolhido pelo cliente que o servidor devolve no eco para você casar respostas com requests, `method` nomeia o procedimento, e `params` é um array ordenado de argumentos, vazio quando o método não recebe nenhum. O envelope de resposta ecoa `jsonrpc` e `id` e depois carrega exatamente um de dois membros: `result` quando a chamada teve sucesso, ou `error` quando ela falhou. Segure esse um-ou-outro. Ele está a ponto de importar mais do que qualquer outra coisa nesta lição.

![Um request JSON-RPC carrega jsonrpc, id, method e params, e a resposta ecoa o id com um membro result ou um membro error.](assets/v01-diagram.webp)

Por que `id` existe, se o servidor só ecoa ele? Porque o JSON-RPC foi projetado para clientes que fazem pipelining: dispare cinco requests por uma conexão, receba cinco respostas na ordem em que o servidor terminar elas, e case cada resposta de volta com a pergunta dela pelo id. O protocolo até permite batching, um array de envelopes de request respondido por um array de respostas em uma ida e volta. O nosso poller manda um request por vez e dá await nele, então um `id: 1` constante está completamente de boa, e eu quero que você note que essa é uma decisão que você acabou de poder tomar, consciente, porque o envelope é seu. Uma biblioteca de cliente teria tomado ela por você, invisivelmente, junto com outras cinquenta. Ser dono do envelope quer dizer que toda a superfície do protocolo é sua para usar ou ignorar, e também quer dizer que quando você um dia quiser pipelining ou batching, ninguém pré-construiu isso: esse é o trade-off, visível desde o primeiríssimo campo.

Um aparte sobre boas maneiras enquanto a gente escreve clientes HTTP na mão. Durante a pesquisa deste curso eu sondei a API do crates.io para os pins de versão acima, e um `curl` pelado sem User-Agent voltou como um 403 com corpo vazio, não JSON. Eu reverifiquei isso hoje; continua voltando. O registro com que o seu próprio toolchain conversa recusa clientes que não se identificam. APIs públicas cobram etiqueta, e a semana em que você começa a escrever clientes HTTP crus contra elas é a semana em que isso deixa de ser curiosidade. O endpoint público da Solana tem a própria versão disso: os rate limits que você aprendeu a respeitar no trabalho de backoff em TS valem para este poller também.

### Structs tipadas: a recompensa do serde

`serde_json::Value` serviu para o exemplo, mas o poller não pode entregar em cima dele. Enfiar a mão num `Value` com chaves de string é exatamente a pescaria sem tipos que m02-l2 te ensinou a recusar em TypeScript. Esta é a mesma disciplina, terceira aparição: o zod parseia JSON desconhecido na fronteira para um tipo ou falha alto, o serde fez isso para o seu arquivo de config em m05-l1, e agora faz isso para uma blockchain. Parse, don't validate, agora apontado para a mainnet.

As structs para aquela resposta de getBalance:

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

Percorra os campos contra o JSON que você imprimiu. `result` é um objeto com dois membros, então `BalanceResult` tem dois campos. `context` te diz em qual slot o nó respondeu, metadado útil para um monitor, e `value` é o saldo em lamports. A string `apiVersion` dentro de context não tem campo na struct, e isso é deliberado: o serde ignora campos desconhecidos por padrão, então você tipa só o que consome e a resposta pode crescer sem te quebrar. E note o que `value: u64` está fazendo silenciosamente. No painel em TypeScript da lição passada, os lamports forçaram a cerimônia do bigint porque números de JavaScript perdem precisão passando de 2^53. O `u64` do Rust segura a faixa inteira nativamente. A gambiarra era um problema de JavaScript; não importe ela para onde a linguagem não tem a doença.

A falha clássica de primeira tentativa aqui é desserializar direto para o formato do value, apontando um `u64` pelado para `result` e ver o parse falhar, porque `result` embrulha context e value e achatar isso na mão não sobrevive ao contato com os bytes de verdade. Quando um campo é renomeado ou falta, essa abordagem inteira faz o parse falhar como um valor `Result` que você roteia. Não uma surpresa em tempo de execução três funções depois. Essa propriedade está a ponto de virar a espinha dorsal do design de erros.

![Cada membro do result de getBalance mapeia para um campo de struct, exceto apiVersion, que o serde ignora por design.](assets/v02-annotated-code.webp)

### Quatro jeitos de falhar, um enum

Aqui está a cilada que separa leitores de blockchain de brinquedo dos de verdade, e eu consigo te mostrar ela ao vivo. Mande um nome de método digitado errado para a mainnet e observe os dois canais:

```bash
curl -s -o /dev/null -w "%{http_code}\n" https://api.mainnet.solana.com \
  -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBalanceTypo","params":[]}'
# 200
```

O HTTP disse 200. O corpo disse:

```json
{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}
```

Um erro de JSON-RPC chega dentro de um sucesso de HTTP. A camada de transporte fez o trabalho dela perfeitamente: entregou uma resposta bem formada que por acaso diz "error" no vocabulário do próprio envelope. Um poller que verifica `response.status()` e nada mais vai marcar uma leitura que falha como saudável para sempre, e um monitor que reporta errado é pior que nenhum monitor, porque as pessoas acreditam nele. Então o modelo de falha de uma sonda de blockchain tem quatro planos distintos, e eles merecem quatro nomes distintos:

1. **Transport.** O request nunca completou: falha de DNS, conexão recusada, timeout. O reqwest te passa o erro próprio dele.
2. **HTTP status.** Uma resposta chegou mas o status não é 2xx: um 429 de rate limiting, um 502 de um proxy morrendo.
3. **RPC error.** O HTTP teve sucesso e o membro `error` do envelope está preenchido. O plano cilada.
4. **Shape.** O HTTP teve sucesso, nenhum membro error, mas o JSON não parseia para os seus tipos: um campo renomeado, um envelope errado, uma mudança de API.

Quatro planos, quatro remediações. Um erro de transport pode significar a sua rede; tente de novo com backoff. Um 429 quer dizer vá mais devagar. Um erro de RPC quer dizer que o seu request está errado; nenhum retry vai consertar um método com typo. Um erro de shape quer dizer que o mundo mudou e os seus tipos precisam de um mantenedor. Em m09-l2, quando a estação ganhar logging estruturado, cada plano vira um nome grepável, e a diferença entre "o RPC estava fora" e "a gente estava parseando errado" vira um grep em vez de uma tarde.

Esta é a recompensa do thiserror que as lições de erro do M5 prepararam. Um enum, uma variante por plano, e a taxonomia inteira é um tipo que o compilador impõe:

![Uma resposta de sonda passa por quatro verificações ordenadas, e cada verificação que falha sai para a própria variante de erro dela antes de um sucesso tipado.](assets/v03-flowchart.webp)

Um footgun para desarmar antes do lab, porque ele vive no plano um. O cliente padrão do reqwest não define nenhum timeout geral de request. Nenhum. Um nó de RPC preso não dá erro; ele mantém a sua conexão aberta, e tudo que está dando await nela, indefinidamente. O seu loop de poll do m06-l1 já aplica um timeout por request nas sondas da frota, e a disciplina do m06-l1 vale sem mudança aqui: toda sonda de blockchain carrega um timeout explícito, porque um poller empacado numa leitura é uma estação com os olhos fechados.

### A questão do crate, respondida com honestidade

Pergunta justa neste ponto: o ecossistema Rust da Solana já traz crates de cliente, então por que um curso de Solana está te ensinando a escrever JSON-RPC na mão em vez de pegar o `solana-client`?

Comece pela assimetria que você talvez já esteja sentindo. Na lição passada as superfícies em TypeScript ganharam uma biblioteca de cliente, o kit, e ninguém escreveu nada na mão. Nesta lição a superfície em Rust vai direto no fio. Isso não é inconsistência; é a mesma decisão de dimensionamento aterrissando diferente em terreno diferente. No lado do TS, o kit é o cliente canônico do ecossistema, o curso irmão de frontend constrói toda a prática dele em cima dele, e o trabalho deste curso era te passar falando esse dialeto. No lado do Rust existe um caminho tipado também, e a gente vai nomear ele direito em um instante, mas um leitor de fundamentos fazendo duas leituras num timer ganha aqui algo melhor que uma biblioteca: um lab onde a disciplina de serde do M5 e a taxonomia de erros do mesmo módulo param de ser exercícios e viram um cliente de blockchain funcionando. O caminho escrito na mão paga matrícula dobrada. Ele lê a blockchain E prova que tudo o que você aprendeu sobre parsing e erros era código de cliente da Solana desde o começo.

Por causa do que o `solana-client` é. Ele é o cliente pau-para-toda-obra: ao lado das leituras de RPC ele empacota uma stack de transporte de submissão de transações completa, clientes QUIC e UDP, o cliente TPU que fala direto com os líderes de bloco, cache de conexão, a maquinaria do streamer. Esse é o equipamento de um sistema que atira transações em validadores. O nosso poller faz duas perguntas num timer. Um leitor de fundamentos nunca chama nada desse transporte, e trazer ele para dentro quer dizer compilar ele, auditar ele e acompanhar o retrabalho de release dele à toa.

Agora a parte em que eu mantenho o argumento honesto, porque existe uma versão mais preguiçosa dele que um aprendiz atento vai pegar. A versão preguiçosa diz "use o `solana-rpc-client` em vez dele, esse é o enxuto", e acena para contagens de dependência. Conte você mesmo: na data da pesquisa deste curso, o `solana-client` 4.2.2 tem 29 dependências diretas mais uma dev-dependency, e o `solana-rpc-client` 4.2.2 tem 35. O crate "enxuto" tem MAIS deps diretas na contagem crua; ele puxa uma pilha de type-crates leves que se somam. O que ele não puxa é nada da maquinaria de transporte de QUIC, TPU ou streamer. Então a afirmação defensável é escopada: enxuto em escopo de transporte, não em contagem de deps. Argumente a partir do eixo que de fato diferencia. Se a sua evidência é um número, alguém vai conferir o número, e se o número está errado a sua conclusão correta morre com ele.

![JSON-RPC escrito na mão, solana-rpc-client e solana-client comparados em dependências, escopo de transporte, tipagem e adequação.](assets/v04-comparison.webp)

Então o trade-off honesto, por inteiro. JSON-RPC escrito na mão é o piso com transparência total: zero retrabalho de crate da Solana, e toda linha reusa uma habilidade que este curso já te ensinou. O custo é que VOCÊ é o dono do envelope. Métodos novos querem dizer structs novas. Níveis de commitment, encodings de resposta, requests em batch: tudo manual, tudo seu. (Nível de commitment, já que acabei de nomear: um parâmetro opcional que escolhe quão final uma resposta tem que ser antes de o nó te dar ela. A gente aceita o default neste curso e deixa o ajuste onde ele pertence, na documentação abaixo.) Ninguém atualiza os seus tipos quando o RPC evolui; os seus testes de shape pegam a quebra e depois um humano, você, conserta. Para um poller fazendo duas leituras, esse trade-off é correto. Um indexer de verdade ou um sistema de trading se forma para o `solana-rpc-client` e os crates `solana-*` granulares, que é exatamente por que eles moram na caixa abaixo e não nesta lição.

**Vá mais fundo (os 20%).** a referência canônica para todo método JSON-RPC que o padrão desta lição consegue alcançar, params de request, formatos de resposta, níveis de commitment, e a própria spec do envelope, é a página de RPC HTTP methods da Solana: https://solana.com/docs/rpc/http (verificada ao vivo em 2026-09-02). Salve como bookmark; ela é a metade que falta do padrão de hoje, e quando as suas leituras crescerem além do escrito na mão, o caminho tipado é o `solana-rpc-client` mais os crates `solana-*` granulares, adotados com a disciplina de leitura de dependências do m05-l2. Nada no lab de hoje depende de nada disso.

## Lab: o pollerd ganha sondas de blockchain

A construção. No fim, o `/status` responde com os alvos da frota que ele já reporta mais um bloco chain, e a imagem do GHCR faz re-ship sem você tocar em um único arquivo de ops.

1. **Manifest, trinta segundos.** Você já virou a feature `json` do reqwest no faça-primeiro. Acrescente uma assinatura ao `crates/pulse-pollerd/Cargo.toml`:

   ```toml
   thiserror = { workspace = true }
   ```

   A versão mora na raiz do workspace onde m05-l2 declarou ela (2.0.20). Leia o bloco `[dependencies]` inteiro quando terminar e deixe isso registrar: pulse-engine, serde, serde_json, tokio, axum, reqwest, thiserror. Todos eles são anteriores a esta lição. O workspace não ganha nada novo hoje; esse é o ponto, e isso continua verdade até o push final.

2. **O enum de erro, seu, a partir de um contrato.** Crie `crates/pulse-pollerd/src/chain.rs` e autore `ProbeRpcError` você mesmo com `#[derive(Debug, Error)]`. O contrato, uma variante por plano da seção de teoria:

   - `Transport` embrulha `reqwest::Error`. Use `#[from]`, o músculo de conversão do m05-l2, para que `?` eleve as falhas do reqwest para o seu tipo automaticamente.
   - `Status` carrega o código ofensor como um `u16`.
   - `Rpc` é uma variante struct carregando `code: i64` e `message: String` retirados do objeto error do envelope.
   - `Shape` embrulha `serde_json::Error`, também `#[from]`.

   Escreva uma mensagem `#[error("...")]` para cada uma que você gostaria de ler num log às 2 da manhã, e dê ao enum um método pequeno: `pub fn plane(&self) -> &'static str`, retornando `"transport"`, `"http_status"`, `"rpc"` ou `"shape"`. Quatro strings estáticas. Esse método não parece nada hoje; ele é o nome grepável em que o logging estruturado do m09-l2 vai se apoiar.

3. **A linha de transporte e a sonda trabalhada.** Aqui está a minha metade do contrato de recuo: a chamada genérica, o parse separado para testabilidade, e o getBalance trabalhado. Leia, depois digite no `chain.rs`:

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

   Acrescente as structs `RpcContext` e `BalanceResult` da seção de teoria acima destas. Depois percorra as costuras, porque duas decisões aqui dentro são estruturais. Primeiro, `parse_rpc_response` recebe um `&str`, não uma resposta de rede, o que torna a taxonomia de falhas inteira testável com fixtures de string e sem rede; você vai explorar isso no passo 4. Segundo, o parse passa por `serde_json::Value` antes da sua struct tipada, então o membro error é verificado antes de o membro result ser interpretado, e uma resposta sem nenhum dos dois membros cai para um erro de `Shape` quando `Null` se recusa a virar o seu tipo. Os operadores `?` fazem o roteamento silenciosamente: uma falha de send eleva para `Transport`, uma falha de from_str ou from_value para `Shape`, as duas via as conversões `#[from]` que você escreveu no passo 2. A taxonomia não é um comentário. É o sistema de tipos fazendo a classificação.

   Com honestidade, um `rpc_call<T>` genérico mais um enum é a maior parte do que um SDK de cliente é. Todo o resto são wrappers de conveniência, e agora você pode escrever dois deles.

4. **Sua sonda e sua prova.** Escreva `get_slot` você mesmo. O envelope de result dele não é objeto nenhum: `getSlot` retorna um número pelado como `result`, então a função inteira é `rpc_call` com `T = u64`, método `"getSlot"` e params `json!([])` vazios. Uma revelação para sentir: o seu genérico lida com um formato de resposta completamente diferente com zero mudanças no transporte.

   Depois os testes, num `#[cfg(test)] mod tests` no fim do `chain.rs`. A barra de aceitação é um caminho-feliz de desserialização e uma fixture malformada por sonda. Duas fixtures trabalhadas de mim, capturadas ao vivo da mainnet em 2026-09-02, para que os seus testes afirmem contra bytes de verdade:

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

   Você escreve o resto: um caminho feliz de slot (`result` é `443692896`, afirme que o número chega), uma fixture de balance malformada (delete o membro `context` e afirme `Err(ProbeRpcError::Shape(_))` com `matches!`), e uma fixture sem result (um envelope sem nenhum dos dois membros, mesma afirmação). `cargo test -p pulse-pollerd`, verde. Note o que você NÃO precisou: de rede, de um mock server, de um runtime async nos testes. A costura de `&str` comprou tudo isso.

![Três métodos de RPC retornam três results com formatos diferentes, e a mesma chamada genérica lida com cada um mudando só o parâmetro de tipo.](assets/v05-comparison.webp)

5. **Faça a fiação dele no `/status`, a partir de um contrato.** O poller hoje serve um mapa só de alvos da frota. O formato pretendido depois deste passo, o mesmo JSON que a trava de verify dá curl:

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

   Os tipos do contrato, em `chain.rs`:

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

   A fiação é sua, e ela é a arquitetura do m06-l1 reencenada em miniatura, então construa por analogia, não do zero. Escreva uma `async fn chain_loop(chain: ChainState)` que cria o próprio `reqwest::Client` dela, tiqueteia um `tokio::time::interval` a cada 30 segundos, dá await em `get_slot` e `get_balance` para o endereço que você observa, e depois, com os dois resultados já em mãos, pega o lock uma vez e escreve os campos: sucessos em `slot` e `balance_lamports`, qualquer falha em `last_error` como `format!("{}: {e}", e.plane())` para que o nome do plano lidere a mensagem. A regra de lock do m06-l1 vale textualmente e eu não vou pedir desculpa por repetir ela: os dois awaits terminam ANTES de o lock ser pego, nunca segure a guard atravessando um await. No `main`, dê spawn em `chain_loop` ao lado do spawn de `poll_loop` que já existe, depois empacote os dois estados numa struct pequena `AppState { targets, chain }` (derive `Clone`), troque o `.with_state` do router para ela, e atualize `status_handler` para dar lock em cada mapa brevemente, clonar snapshots e retornar um `StatusResponse { targets, chain }`. Troque `WATCHED_ADDRESS` por um endereço com que você se importa, ou mantenha o vote program e o lamport solitário dele. `cargo run -p pulse-pollerd`, depois `curl -s localhost:8080/status` e leia o primeiro bloco chain da sua estação.

![Um novo loop chain se junta ao loop poll da frota que já existe, cada um escrevendo o próprio estado compartilhado dele que um endpoint de status reporta junto.](assets/v06-diagram.webp)

6. **Quebre de propósito.** A aceitação da taxonomia não é que ela compila; é que a falha degrada em vez de detonar. Mude a URL de RPC para algo irresolvível, `https://rpc.invalid`, e rode o poller. O processo tem que continuar em pé, o `/status` tem que continuar servindo, e num boot novo o bloco chain deve ler assim:

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

   A mensagem exata depois de `transport:` vai variar com o seu SO e resolvedor, e isso é normal; o nome do plano na frente dela é a parte que o seu código garante. Se em vez disso você quebrar contra um poller rodando, `slot` e `balance_lamports` seguram os últimos valores honestos deles enquanto `last_error` se preenche, que é precisamente o comportamento que você quer que um monitor tenha: desatualizado-mas-etiquetado ganha de vazio. Coloque a URL de verdade de volta e veja o próximo tique curar isso: `last_error` volta para `null`, o número do slot volta a subir. Uma URL errada te custando um campo numa resposta JSON em vez de um processo é o argumento inteiro do passo 2, demonstrado em noventa segundos.

7. **O re-ship.** Faça commit, push, e nada mais. O pipeline do M6 pega o commit, roda os seus testes incluindo as fixtures novas, constrói o mesmo Dockerfile multi-stage com cargo-chef, e empurra a imagem para o GHCR, porque o workspace, o Dockerfile e o workflow estão intocados desde m06-l4. Essa é a demonstração para a qual a lição vinha caminhando: código que aterrissa dentro de um sistema que já faz ship herda o ship dele. Quando a execução estiver verde, prove de ponta a ponta de fora, do jeito que um estranho faria:

   ```bash
   docker pull ghcr.io/<you>/pulse-pollerd:latest
   docker run --rm -p 8080:8080 ghcr.io/<you>/pulse-pollerd:latest
   # in another terminal:
   curl -s localhost:8080/status
   ```

   Alvos da frota, mais um bloco chain, com um slot ao vivo da mainnet dentro, servidos por uma imagem que qualquer máquina na Terra pode dar pull. A terceira superfície da estação acabou de ganhar os olhos dela.

![Um único git push flui pelo pipeline de CI inalterado até uma imagem nova no registro que o aprendiz dá pull, roda e dá curl.](assets/v07-flowchart.webp)

## Challenge

Solo. Adicione uma sonda `get_health`. `getHealth` responde com a string `"ok"` como todo o `result` dele, um terceiro formato que as suas structs não conheceram: não é objeto, não é número, é uma string pelada. Se você rotear ele por `rpc_call`, a função tem duas linhas, e esse é o teste de se você entendeu o design: a taxonomia e o genérico têm que absorver um método novo sem uma única edição na linha de transporte. Exponha isso no bloco chain como você achar melhor, um campo `healthy: Option<bool>` é uma resposta limpa. Escreva os dois testes: uma fixture de caminho feliz que você mesmo autora no estilo capturado ao vivo, e uma malformada. E pense na rota de falha antes de rodar qualquer coisa: um nó não saudável reporta pelo membro error do envelope, o que quer dizer que a resposta chega no plano três, já classificada por código que você escreveu no passo 2, e a sua string de `plane()` diz `"rpc"` antes de você ter lido uma única linha de log. Aceitação: `cargo test -p pulse-pollerd` verde com as suas duas fixtures novas, e o `/status` mostrando o campo de health contra o endpoint de verdade. O seu edge worker em TS vem sondando esse mesmo método desde m07-l1 (o worker em Rust classifica; ele não faz chamadas de saída); agora você sabe exatamente o que aquela sonda estava fazendo.

## Checkpoint

O que você consegue fazer agora, concretamente: ler estado da mainnet da Solana em Rust com um POST de cinco linhas construído inteiramente a partir de crates dos quais você já fez ship; tipar uma resposta de RPC de forma que um campo renomeado seja um `Result` roteado, não uma surpresa em tempo de execução; classificar toda maneira em que uma leitura da blockchain pode falhar em quatro planos e dizer qual deles se esconde dentro de um HTTP 200; e defender a decisão de nenhum-crate-da-Solana no eixo que sobrevive a uma auditoria, escopo de transporte, concedendo o eixo de contagem de deps que não sobrevive.

A recuperação de 30 segundos antes de você fechar o terminal: por que uma leitura completamente falha pode chegar como HTTP 200? (O JSON-RPC reporta falhas dentro do próprio envelope dele; o transporte teve sucesso em entregar uma resposta que diz error, então você precisa verificar o membro `error`, não só o status.) E o argumento do crate em uma frase? (O `solana-client` empacota um transporte de submissão QUIC e TPU que um poller de duas leituras nunca chama; esse escopo, não contagens cruas de dependência, é o argumento, e as contagens de fato apontam para o outro lado.)

Um pedido de calibração. Esta lição te passou o enum de erro como um contrato em prosa em vez de um esqueleto de código, a primeira vez que o curso fez isso. Se autorar ele a partir dos quatro tópicos pareceu trabalho de verdade, essa é a dificuldade pretendida; se pareceu subespecificado e você teve que adivinhar formatos, diga isso no feedback, porque o m09 se apoia mais em passos guiados por contrato e eu quero a rampa honesta.

A estação agora lê a blockchain de toda superfície que ela tem, nas duas linguagens: o painel, o edge worker, e uma imagem Docker com leituras tipadas da mainnet dentro. O que quer dizer que toda afirmação que este curso fez sobre a stack está provada agora, exceto uma: que ela consegue ESCREVER. A próxima lição é a recompensa do módulo. Uma transferência assinada na devnet, enquadrada como a checagem de saúde do caminho de escrita da estação, com uma caixa de honestidade sobre faucets e um validador local no seu bolso de trás para o dia em que o faucet estiver seco. A sonda que muta.
