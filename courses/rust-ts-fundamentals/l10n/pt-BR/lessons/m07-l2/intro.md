# A recompensa: Rust no mesmo edge

## Resumo

A m07-l1 entregou o worker TS: a lógica pura do pulse-core rodando num cron em centenas de cidades, o último status conhecido estacionado no KV, o segredo tratado como gente grande, e o getHealth da Solana já sentado na lista de sondas. Esse era o motor um. A estação tem dois. Hoje o motor Rust recebe o mesmo tratamento: você compila o `pulse-engine` para WebAssembly, embrulha ele num projeto workers-rs, e aterrissa a segunda URL workers.dev ao vivo da estação com exatamente o mesmo comando de deploy que você rodou ontem. O contrato de recuo, em voz alta, precificado com honestidade: o template gerado chega funcionando, o handler de verdade chega completo na página, e os seus TODOs de mão na massa são a fiação em volta dele, a dependência do motor, o braço classify da CLI, e o par get e put do KV; a rota final da tabela de transições é a única construção totalmente solo da lição. Mais magra que alguns labs, de propósito, porque a tese é que o port em si é pequeno. Você fez este mesmo deploy uma lição atrás. O delta é a linguagem. O delta ser SÓ a linguagem é a lição inteira.

## A aposta paga

Lá no M4 eu fiz você manter o motor Rust puro: o classificador, a máquina de estados, os tipos serde, todos eles funções de valores para valores, nenhum socket ou file handle em lugar nenhum do crate. Na m05-l2 a gente separou essa pureza no próprio crate do workspace e eu listei o que um núcleo puro consegue alimentar, terminando com "até um worker WASM no tier de deploy". Já é mais tarde no tier de deploy. Dois comandos começam a coleção:

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-generate
```

O primeiro ensina o seu toolchain atual a emitir WebAssembly em vez de código de máquina nativo. Mesmo rustc, backend novo, um build da biblioteca padrão a mais. O segundo instala o `cargo-generate`, uma ferramenta de scaffolding que carimba projetos a partir de repos de template do Git, e ela existe nesta lição porque o caminho Rust oficial do Cloudflare começa com ela:

```bash
cargo generate cloudflare/workers-rs
```

Escolha o template `hello-world` quando ele perguntar, dê ao projeto o nome `pulse-edge-rs`, e rode na raiz do repo da sua estação para que ele aterrisse ao lado de `pulse-rs/` e do worker TS. O template também faz uma pergunta que este parágrafo deixaria você adivinhando, "Enable panic=unwind and abort recovery?": aceite o padrão, false. Não faça deploy ainda. Primeiro, dez minutos entendendo o target que você acabou de escolher, porque `wasm32-unknown-unknown` é o nome de target mais honesto do toolchain inteiro.

### Um OS chamado unknown

Um target de compilação do Rust nomeia três coisas: arquitetura, fornecedor, sistema operacional. `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`. Leia o do WASM do mesmo jeito: arquitetura WASM de 32 bits, fornecedor unknown, sistema operacional unknown. OS unknown não é um placeholder esperando um valor. É a especificação. O módulo compilado pode supor nenhuma thread, nenhum socket, nenhum sistema de arquivos, nenhum relógio que ele não tenha pedido, nenhuma variável de ambiente. É computação pura numa caixa lacrada, e qualquer coisa do mundo de fora tem que ser passada para dentro através de uma interface que o host escolhe expor.

Isso deve soar familiar, porque é exatamente o formato do seu crate do motor. O `classify_latency` recebe uma latência e devolve um `Verdict`. O `next_state` recebe um estado, um desfecho e a contagem de falhas consecutivas, e devolve um estado. Os tipos serde viram bytes em valores e de volta. Nada disso nunca abriu uma conexão; a CLI, o poller e o worker TS é que fizeram o I/O e alimentaram o motor com valores. No edge, o host é o workerd, o mesmo runtime da m07-l1, e o que ele passa para dentro da caixa é o contrato de plataforma que você já conhece: fetch, bindings de KV, triggers de cron, passados ao Rust através de uma camada de cola JS que as ferramentas geram para você.

![O crate puro do motor atravessa a costura para dentro do WebAssembly enquanto a CLI e o poller continuam nativos, com a plataforma fornecendo fetch, KV e cron do lado do WASM.](assets/v01-diagram.webp)

### O treino da recusa

Afirmações sobre o que "não dá para portar" são baratas, então vamos comprar o erro de verdade. De dentro do seu crate do motor, commite primeiro, depois sabote ele de propósito:

```bash
cargo add tokio --features full
cargo build --target wasm32-unknown-unknown
```

O build morre dentro do mio, a camada de event loop de OS do tokio, e o erro é incomumente educado sobre o porquê:

```text
error: This wasm target is unsupported by mio. If using Tokio, disable the net feature.
  --> mio-1.2.2/src/lib.rs:44:1
   |
44 | compile_error!("This wasm target is unsupported by mio. If using Tokio, disable the net feature.");
```

Eu rodei exatamente esta sabotagem enquanto escrevia a lição; todo bloco de erro nesta página está colado do meu terminal, não digitado de memória. E olhe para o que o erro está de fato dizendo: o trabalho do mio é embrulhar epoll e kqueue, as facilidades do OS para esperar em sockets. Num target cujo OS se chama unknown não existe nada para embrulhar, então o crate recusa em tempo de compilação. O reqwest falha de um jeito mais sorrateiro, e a sorrateirice vale conhecer: o crate em si compila no wasm32 porque ele carrega um backend de navegador, mas o módulo `blocking` que a sua CLI usa desde a m05-l3 é compilado para fora condicionalmente, então no momento em que o seu código toca nele você recebe `error[E0433]: could not find 'blocking' in 'reqwest'`. Mesma lei, mensageiro diferente: o arsenal async não é banido do WASM por política. Ele está ancorado ao nativo pelas chamadas de OS embaixo dele, e o compilador faz valer a âncora. Agora dê `git checkout` no Cargo.toml E no Cargo.lock, porque o `cargo add` reescreveu os dois e o build que falhou puxou os pins do tokio para o lockfile, e deixe o motor voltar a ser portável.

Esta é a costura núcleo-puro e casca-de-IO completando o arco dela. A m07-l1 provou isso em TypeScript, onde quem impôs foi o runtime do workerd recusando iniciar numa leitura de sistema de arquivos: o módulo com shim, o sistema operacional ausente. O Rust prova em tempo de compilação, antes de qualquer coisa ser entregue. Duas linguagens, uma lei: lógica que nunca tocou o OS vai para qualquer lugar; o I/O pertence à casca, e todo host ganha a própria casca.

![Uma linha do tempo das primeiras decisões de pureza nos módulos três e quatro até o port para WebAssembly de hoje, mostrando que o deploy foi planejado e não sorte.](assets/v02-timeline.webp)

### Lendo o template que você acabou de gerar

Abra `pulse-edge-rs/`. O cargo-generate deixou para você um projeto de verdade, pronto para deploy, e o hábito que este curso aplica a todo scaffold se aplica aqui: leia o artefato antes de rodar ele. `Cargo.toml` primeiro:

```toml
[package]
name = "pulse-edge-rs"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
worker = { version = "0.8" }
worker-macros = { version = "0.8" }
```

Três observações campo a campo. O `crate-type = ["cdylib"]` diz ao cargo para produzir uma biblioteca dinâmica no estilo C em vez de um binário, que é o formato que o wasm-bindgen sabe consumir. O crate `worker` é o SDK Rust para a plataforma Workers: os seus tipos para `Request`, `Response`, `Env`, e o binding de KV. E o pin diz `"0.8"`, não um latest pelado, porque o worker é pré-1.0. Diga a regra da m05-l2 em voz alta: sob semver, um minor 0.x tem permissão de te quebrar do jeito que um 2.0 quebraria em outro lugar, então o pin segura a linha 0.8 de propósito e você lê as release notes antes de escolher a 0.9. Enquanto escrevo, sondado no crates.io em 2026-09-02, a linha está em 0.8.5, publicada em 2026-06-12. Nota de freshness: reconfira esse dígito quando você fizer este lab; um SDK pré-1.0 é exatamente o tipo de dependência cujo minor atual importa. (O template também fixa a própria edition em 2021 enquanto o seu workspace roda 2024; isso é o levantamento da m05-l2 se desenrolando num manifest só, o ecossistema embarcando atrasado enquanto os seus próprios crates do lado do host andam na edition atual sob a ressalva datada daquela lição, e a regra dela cobre isto também: a edition do template é o contrato do template. Editions diferentes atravessando uma dependência de path estão de boa; editions são por crate, que é exatamente por isso que elas conseguem existir.)

O `src/lib.rs` do template tem oito linhas e você já consegue ler cada uma delas:

```rust
use worker::*;

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response> {
    Response::ok("Hello World!")
}
```

O `#[event(fetch)]` é a macro do crate worker marcando esta função como o handler de fetch, o mesmo slot de contrato que o seu worker TS preencheu com um método `fetch` exportado. Existe um gêmeo `#[event(scheduled)]` para cron, que recebe um `ScheduledEvent` em vez de um `Request`, e vale saber que este worker poderia rodar num agendamento com uma função e uma linha de wrangler.toml. A gente deliberadamente não faz isso: o worker TS já roda no cron, a topologia de hub do capstone quer este aqui respondendo sob demanda, e dar aos dois workers o mesmo trabalho te ensinaria copiar-e-colar, não arquitetura. Sim, a função é `async`, e isso merece uma pausa dado o treino da recusa que você acabou de rodar: Rust async funciona numa boa num worker. O que falta no target unknown é o I/O apoiado no OS do tokio, não o recurso async da linguagem. O workerd dirige esses futures ele mesmo, através do event loop JS de que ele já é dono.

Agora o `wrangler.toml`, onde o truque se esconde:

```toml
name = "pulse-edge-rs"
main = "build/index.js"
compatibility_date = "2026-09-02"

[build]
command = "cargo install -q \"worker-build@^0.8\" && worker-build --release"
```

O `main` aponta para um arquivo JavaScript que ainda não existe. O bloco `[build]` é o porquê: todo `wrangler deploy` roda primeiro o `worker-build`, que compila o seu crate para wasm32-unknown-unknown, roda o wasm-bindgen para gerar a cola JS que faz os tipos atravessarem a fronteira, roda o wasm-opt para encolher o módulo, e emite `build/index.js` como o shim de entrada. Três ferramentas empilhadas sob um comando. Estou nomeando elas para que a saída do build não pareça ruído, e isso é tudo que a gente faz com elas; worker-build, wasm-bindgen e wasm-opt são encanamento que este curso não ensina. Uma consequência prática vale guardar, mesmo assim: quando um erro de build mencionar cola do wasm-bindgen e parecer exótico, rode `cargo check` no seu workspace nativo primeiro. Erros de tipo de verdade aparecem lá com spans normais, e você debuga Rust em Rust em vez de através da cola.

![Um comando de deploy se abre em leque numa compilação Rust, geração de cola e otimização antes de o wrangler subir o resultado e imprimir uma URL ao vivo.](assets/v03-flowchart.webp)

### KV através de tipos Rust

Mais uma peça do crate worker antes da síntese, porque o lab se apoia nela: o binding de KV. Tudo o que você aprendeu sobre KV ontem continua valendo e não é reensinado aqui: é o armazenamento chave-valor persistente da plataforma, eventualmente consistente, com escrita limitada no tier gratuito, e a razão pela qual um worker sem estado consegue lembrar de qualquer coisa. O que muda em Rust é puramente a história dos tipos, e a história dos tipos é boa. O `env.kv("PULSE_STATUS")` procura o binding pelo nome e te passa um `KvStore`. As leituras voltam através de um builder que termina em `.json::<T>().await?`, o que quer dizer que o KV te passa um `Option<T>` do seu próprio tipo do motor, já desserializado, ou `None` quando a chave nunca foi escrita. As escritas vão no outro sentido: serialize com `serde_json::to_string`, depois `kv.put(key, value)?.execute().await?`, onde o `.execute()` é o builder de fato disparando (esquecer dele é o bug clássico de primeira semana, porque a linha sem ele compila numa boa e não faz nada). Serde de pé nas duas pontas do cano é o ponto a notar. As mesmas linhas de derive que alimentaram a saída JSON da CLI e o endpoint `/status` do poller agora definem o formato de wire do seu armazenamento no edge, num terceiro runtime, sem código novo. Quando as pessoas dizem que o ecossistema do Rust converge forte em serde, é isto que convergir compra.

### Um contrato, duas linguagens

Aqui está a síntese para a qual o módulo inteiro vem caminhando. Ontem você fez deploy de TypeScript com `npx wrangler deploy`. No lab abaixo você vai fazer deploy de Rust com `npx wrangler deploy`. Não um comando análogo. O mesmo comando, e a plataforma não consegue ver diferença, porque o que a plataforma define é um contrato: um formato de handler de fetch, um formato de handler de scheduled, bindings declarados na config, um verbo de deploy. Qualquer coisa que satisfaça o contrato é um worker. Faça a si mesmo a pergunta discriminante: o que, exatamente, o wrangler precisou saber sobre a sua linguagem para rodar o deploy de ontem? Nada. Ele rodou um comando de build a partir de um arquivo de config e subiu o que saiu, e vai fazer exatamente isso de novo hoje. Então o artefato que você está entregando ao Cloudflare nunca foi de verdade "um app TypeScript" ou "um app Rust"; é uma implementação de contrato, e a linguagem é um fornecedor por trás dela. Essa é a lição durável para levar deste módulo, porque você vai encontrar ela de novo em todo lugar onde plataformas vivem: o contrato de contêiner do M6 não se importava que a caixa tinha Rust dentro, só que um processo escutasse numa porta; o contrato JSON-RPC do próximo módulo não vai se importar com o que escreveu o request, só que os bytes façam parse. Quando uma plataforma define a costura, a linguagem para de ser uma decisão de arquitetura e vira uma escolha por componente, feita nos méritos de verdade de cada componente.

E dito isso, a tabela de custo honesta, porque a síntese corta dos dois lados. O caminho Rust empilha três ferramentas de build sob o deploy, anda num SDK pré-1.0 cujos minors podem quebrar entre uma sentada e outra, e abre mão de tokio e reqwest na costura. Para um worker que é quase todo cola em volta de fetch, o worker TS que você construiu ontem é o padrão de menor atrito, ponto final. Você paga o pedágio do WASM quando o valor é o próprio núcleo Rust compartilhado: um classificador, uma máquina de estados, um conjunto de tipos serde, já testados, já confiados pela CLI e pelo poller, agora respondendo do edge sem reescrever nada. Para o motor da estação essa troca vale a pena. A heurística para levar com você: conte as linhas que são suas. Se a substância do worker são chamadas de plataforma com um pouquinho de lógica polvilhada, escreva na linguagem natal da plataforma e siga em frente; se a substância é um núcleo que você mantém, testa e confia em Rust em outros lugares, porte o núcleo e mantenha uma implementação só da verdade. Saber de QUAL lado desse julgamento um serviço cai é exatamente a habilidade que este curso está vendendo.

![Uma comparação de seis linhas dos workers TypeScript e Rust compartilhando um comando de deploy enquanto diferem em ferramentas de build, maturidade de SDK e quando cada escolha ganha.](assets/v04-comparison.webp)

**Vá mais fundo (os 20%).** esta lição ensina o caminho workers-rs em profundidade de trabalho: o target, o template, as macros de handler, KV a partir de Rust, e o deploy. O catálogo completo de bindings, classes de Durable Object em Rust, wrappers de send-safety, e o resto da superfície do SDK moram nos docs de linguagem Rust do Cloudflare para Workers: [https://developers.cloudflare.com/workers/languages/rust/](https://developers.cloudflare.com/workers/languages/rust/) (URL checada em 2026-09-02). Salve como bookmark; o lab abaixo não precisa de nada do material de bookmark.

### A trava de tier: o que a gente pulou, e onde isso mora

O M7 fecha o tier de edge, então antes do lab, o mapa da família de plataforma em cima da qual a gente deliberadamente não construiu. Isto quer ser um catálogo; eu vou manter como um mapa, umas poucas linhas honestas por produto, porque saber o que você pulou e por quê é uma habilidade de verdade e fingir que a plataforma é só Workers mais KV seria uma mentira por omissão.

**R2** é armazenamento de objetos, no formato do S3 e compatível com a API do S3: arquivos, imagens, dumps de histórico de sondas, qualquer coisa de tamanho blob. O tier gratuito é genuinamente generoso, 10 GB-mês armazenados e egress de graça, e egress de graça é o pitch inteiro do R2 contra o S3. Mas várias threads da comunidade Cloudflare relatam que habilitar o R2 exige vincular um método de pagamento mesmo para uso no tier gratuito, e isso falha a regra de sem-cartão deste curso. Então o R2 é uma extensão opcional claramente rotulada para quem escolhe vincular um cartão, nunca um passo do caminho principal; o KV carrega o nosso estado. Se você um dia vincular um cartão, guardar o JSON cru de cada rodada de sonda no R2 é o primeiro uso natural, e o crate worker fala com ele com o mesmo padrão de binding que você está prestes a usar para KV.

**D1** é SQL no edge, SQLite por baixo do capô, 5,000,000 de linhas lidas por dia de graça. Ele responde a pergunta que o KV não consegue: consultas. No momento em que você quiser "todo alvo que ficou Degraded na última hora", buscas por chave param de ser o formato certo e o D1 é para onde a plataforma te manda, sem cartão exigido.

**Durable Objects** são coordenação com estado: um objeto de thread única com o próprio armazenamento, onde todo request para uma dada chave é roteado para a mesma instância, que é como a plataforma faz "existe exatamente um destes" sem você rodar um servidor. 100,000 requisições por dia de graça, e eles estão no plano gratuito agora, o que nem sempre foi verdade. Se a estação algum dia precisasse de um rate limiter por alvo ou de um contador ao vivo que não pode ter corrida, esta é a ferramenta.

**Queues** compram desacoplamento async entre workers, produtor de um lado e consumidor do outro, 10,000 operações por dia de graça. O trabalho que elas fazem para uma frota de edge é o mesmo trabalho que uma fila de mensagens faz em qualquer lugar: absorver uma rajada agora, processar com calma depois.

**Cloudflare Containers**, GA em 2026-04-13, é a plataforma admitindo em voz alta que algumas cargas de trabalho só querem uma caixa Linux: as suas imagens Docker do M6, gerenciadas, ao lado dos seus workers. Vai parecer a peça que falta na história deste curso, e arquiteturalmente quase é. Também é só no Workers Paid, um pré-requisito de $5/mo, então sob a regra de sem-cartão ele fica como uma placa aqui, não como um lab. Quando você tiver cinco dólares por mês e um motivo, as imagens do M6 que você já dá push para o GHCR são exatamente o que ele roda.

Nenhum curso irmão neste catálogo é dono destes produtos, então este mapa mais os docs oficiais são o repasse honesto: a linha em negrito é a descrição do cargo, e a documentação da própria plataforma é para onde você vai no dia em que a estação precisar de um.

![Cinco produtos da Cloudflare listados com os trabalhos e as cotas gratuitas deles, onde R2 e Containers carregam exigências de pagamento que mantêm eles fora do caminho principal deste curso.](assets/v05-table.webp)

## Lab: pulse-edge-rs

Este é o segundo ship de edge da estação, o gêmeo Rust do worker da m07-l1. O handler de fetch aceita fixtures de sonda ou as últimas amostras guardadas no KV, roda elas pelo MESMO classificador e máquina de estados que a CLI e o poller Docker usam, e devolve JSON classificado do edge.

1. **Faça o deploy do hello world primeiro.** De `pulse-edge-rs/`, antes de tocar em qualquer coisa:

   ```bash
   npx wrangler deploy
   ```

   A autenticação vem junto do login de navegador da m07-l1 (numa máquina nova, `npx wrangler login` primeiro). O wrangler em si não vem junto, porque o pin cuidadoso do v4 de ontem mora no `pulse-edge-ts/package.json` e este projeto do cargo-generate não tem package.json, então o `npx wrangler` pelado aqui busca o wrangler do zero; aceite o prompt do npx, ou mantenha o hábito de fixar com `npx wrangler@4 deploy`. De qualquer jeito a primeira execução instala o worker-build, compila o template, e imprime a sua segunda URL workers.dev ao vivo. Dê um curl nela, veja `Hello World!`, e aprecie o que acabou de acontecer: o seu primeiro pipeline Rust-para-WASM-para-edge funcionou antes de você entender ele, que é a ordem correta para pipelines. Agora a gente faz ele merecer a URL.

2. **Ligue o crate do motor.** Uma questão de fiação que vale resolver antes de qualquer código: dependência de path ou dependência de git? Path, e eu verifiquei a cadeia inteira antes de escrever esta página: um crate fora do workspace pode depender do `pulse-engine` por path, o cargo resolve as assinaturas de dependência `workspace = true` do motor contra o workspace do próprio motor, e o resultado compila limpo para wasm32-unknown-unknown. Adicione as dependências ao `pulse-edge-rs/Cargo.toml`:

   ```toml
   [dependencies]
   worker = { version = "0.8" }
   worker-macros = { version = "0.8" }
   serde = { version = "1.0.229", features = ["derive"] }
   serde_json = "1.0.151"
   pulse-engine = { path = "../pulse-rs/crates/pulse-engine" }
   ```

   Depois dê ao motor o único módulo novo que os dois consumidores vão compartilhar. As fixtures estão prestes a virar um formato de wire entre linguagens, então elas pertencem ao núcleo puro: crie `crates/pulse-engine/src/fixture.rs`:

   ```rust
   use crate::engine::{classify_latency, LatencyMs, Verdict};
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Serialize, Deserialize)]
   pub struct FixtureSample {
       pub name: String,
       pub latency_ms: u64,
   }

   #[derive(Debug, Serialize)]
   pub struct Classified {
       pub name: String,
       pub latency_ms: u64,
       pub verdict: Verdict,
   }

   pub fn classify_fixtures(samples: &[FixtureSample]) -> Vec<Classified> {
       samples
           .iter()
           .map(|s| Classified {
               name: s.name.clone(),
               latency_ms: s.latency_ms,
               verdict: classify_latency(LatencyMs(s.latency_ms)),
           })
           .collect()
   }
   ```

   Declare ele e re-exporte no `lib.rs` do motor (`pub mod fixture;` mais `pub use fixture::{Classified, FixtureSample, classify_fixtures};`), e faça a auditoria de derives ficar exaustiva enquanto você está lá, porque o código dado do passo 4 depende de cada item: o `Verdict` precisa de `Serialize` (e de um lugar nos re-exports da raiz do lib.rs, já que o worker vai nomear `pulse_engine::Verdict` direto), e o `ProbeState` precisa de `Deserialize` acrescentado ao lado do `Serialize` que ele pegou na m06-l1, porque o KV está prestes a fazer round-trip com ele, E ele ainda tem que carregar o cânone completo da m04-l3, `Debug, Clone, Copy, PartialEq, Eq`. O `Copy` em particular é estrutural: o `StoredStatus` do passo 4 deriva `Clone, Copy` segurando um `ProbeState`, o que só compila quando o enum de estado é `Copy` ele mesmo, então se a sua lista de derives descolou em algum momento, restaure ela agora em vez de encontrar o erro de derive dentro de código que esta página chamou de completo. Se os seus nomes de módulo ou de campo descolaram dos meus ao longo dos módulos, mantenha os seus e adapte; a interface que importa são os nomes de função exportados e o formato do JSON.

3. **Dê à CLI a outra metade da trava.** O teste de aceitação desta lição é a mesma fixture produzindo vereditos idênticos a partir de Rust nativo e de WASM no edge, então a CLI precisa de um subcomando `classify`. No enum de comandos do `pulse-cli`, uma variante nova e um braço novo:

   ```rust
   /// Classify a JSON fixture set from stdin and print the verdicts as JSON
   Classify,
   ```

   ```rust
   Command::Classify => {
       use pulse_engine::{FixtureSample, classify_fixtures};
       let raw = std::io::read_to_string(std::io::stdin())?;
       let samples: Vec<FixtureSample> = serde_json::from_str(&raw)?;
       println!("{}", serde_json::to_string(&classify_fixtures(&samples))?);
   }
   ```

   Uma linha de fiação antes desse braço compilar, porque a CLI nunca precisou de saída JSON até agora: dê ao `pulse-cli` a assinatura de serde_json do workspace nas dependências do `Cargo.toml` dele, `serde_json = { workspace = true }`. O `use` dentro do braço traz os dois nomes do motor para o escopo; nada mais muda.

   Escreva uma fixture compartilhada na raiz do repo da estação como `fixture.json`:

   ```json
   [{"name":"solana-rpc","latency_ms":180},{"name":"demo-api","latency_ms":740},{"name":"dead-host","latency_ms":4000}]
   ```

   E rode a partir do diretório de workspace `pulse-rs/`, dito porque as duas metades do comando dependem disso: o `-p` precisa do workspace do cargo como cwd dele, e `../fixture.json` alcança a raiz do repo de exatamente um nível abaixo:

   ```bash
   cd pulse-rs
   cargo run -p pulse-cli -- classify < ../fixture.json
   ```

   ```text
   [{"name":"solana-rpc","latency_ms":180,"verdict":"Up"},{"name":"demo-api","latency_ms":740,"verdict":"Degraded"},{"name":"dead-host","latency_ms":4000,"verdict":"Down"}]
   ```

   Aquela linha é a sua verdade nativa de referência. O edge tem que reproduzir ela exatamente.

![Três fixtures com latências fixas mapeiam para Up, Degraded e Down, e o edge worker tem que devolver a mesma linha serializada que o Rust nativo.](assets/v06-table.webp)

4. **Substitua a lógica de brinquedo.** Troque o `src/lib.rs` do template pelo handler de verdade. Dois TODOs ficam onde vai o par do KV; todo o resto está completo:

   ```rust
   use pulse_engine::{classify_fixtures, next_state, FixtureSample, ProbeState};
   use serde::{Deserialize, Serialize};
   use worker::*;

   #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
   struct StoredStatus {
       state: ProbeState,
       consecutive_failures: u32,
   }

   #[event(fetch)]
   async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
       let url = req.url()?;
       match url.path() {
           "/" => classify_handler(req, env).await,
           _ => Response::error("not found", 404),
       }
   }

   async fn classify_handler(mut req: Request, env: Env) -> Result<Response> {
       let kv = env.kv("PULSE_STATUS")?;
       let samples: Vec<FixtureSample> = if req.method() == Method::Post {
           req.json().await?
       } else {
           // TODO 1: read the "latest-samples" key from KV as Vec<FixtureSample>,
           // defaulting to an empty Vec when the key has never been written.
           Vec::new()
       };
       let classified = classify_fixtures(&samples);

       if req.method() == Method::Post {
           for c in &classified {
               let key = format!("status:{}", c.name);
               let prev: StoredStatus = kv
                   .get(&key)
                   .json()
                   .await?
                   .unwrap_or(StoredStatus { state: ProbeState::Pending, consecutive_failures: 0 });
               let ok = matches!(c.verdict, pulse_engine::Verdict::Up);
               let failures = if ok { 0 } else { prev.consecutive_failures + 1 };
               let next = next_state(prev.state, ok, failures);
               let stored = StoredStatus { state: next, consecutive_failures: failures };
               // TODO 2: write `stored` back to KV under `key`, serialized with serde_json.
           }
       }

       Response::from_json(&classified)
   }
   ```

   Leia o formato antes de preencher os buracos. POST quer dizer "aqui estão amostras frescas, classifique elas e avance o estado por alvo". GET quer dizer "classifique o que quer que o KV tenha visto por último", e é deliberadamente uma leitura pura: os vereditos são recalculados, mas o loop `for` que avança o estado fica atrás da trava do POST, então olhar para o worker não move nada e não escreve nada. Essa trava é estrutural duas vezes. A m04-l3 definiu `consecutive_failures` como falhas de SONDA consecutivas, e uma sonda acontece quando amostras chegam, não toda vez que alguém olha; um loop sem trava deixaria três GETs ociosos, ou um crawler passeando pela URL pública workers.dev, levar um alvo Degraded até Down com zero dado novo de sonda. E todo put no KV gasta o orçamento diário de escrita que a m07-l1 dimensionou com quase nenhuma folga; leituras não podem gastar ele. Os vereditos vêm do `classify_fixtures` e o caminho de POST avança através do `next_state`: as mesmas duas funções, a mesma máquina de quatro estados, os mesmos limiares que respondem na CLI desde o M4 e no poller desde o M6. O worker não autora lógica nenhuma. Ele é uma casca em volta do motor, que tem sido a definição deste curso de uma boa casca desde a m03-l1.

5. **Complete o par do KV.** Esta é a peça ensinada do lab, então aqui vão as duas linhas, com o raciocínio. TODO 1:

   ```rust
   kv.get("latest-samples")
       .json::<Vec<FixtureSample>>()
       .await?
       .unwrap_or_default()
   ```

   E o TODO 2, mais um put extra no fim do ramo POST para que o GET tenha algo para ler da próxima vez (coloque logo antes de `Response::from_json`, travado no método que você já casou):

   ```rust
   kv.put(&key, serde_json::to_string(&stored)?)?.execute().await?;
   ```

   ```rust
   if req.method() == Method::Post {
       kv.put("latest-samples", serde_json::to_string(&samples)?)?.execute().await?;
   }
   ```

   Repare no serde de pé nas duas pontas do cano: o `.json::<T>()` desserializa o que o KV guardou, o `serde_json::to_string` serializa o que você põe de volta, e os tipos que atravessam o cano são os do próprio motor. Este é o mesmo KV que o seu worker TS usou conceitualmente, mas um namespace separado na prática, então crie um e faça o bind dele:

   ```bash
   npx wrangler kv namespace create PULSE_STATUS
   ```

   Cole o bloco de id que o comando imprime no `wrangler.toml`:

   ```toml
   [[kv_namespaces]]
   binding = "PULSE_STATUS"
   id = "<the-id-wrangler-printed>"
   ```

   O nome do binding é o que o `env.kv("PULSE_STATUS")` procura em tempo de execução; o id é qual namespace de verdade responde. Dois workers, dois namespaces, zero estado compartilhado: conforme a topologia de hub do capstone, este worker sonda de forma independente e nunca consome o poller.

![Um request POST flui através da desserialização, do classificador puro e da máquina de estados, e de leituras e escritas no KV, enquanto o GET repassa as últimas amostras guardadas pelo mesmo caminho.](assets/v07-flowchart.webp)

6. **Rode local antes de entregar.** O loop de dev que você tinha para o worker TS existe para Rust também, mesmo comando:

   ```bash
   npx wrangler dev
   ```

   O wrangler roda o pipeline do worker-build e serve o seu worker em `localhost:8787`, com o binding de KV apontado para uma simulação local para que nada que você mande por POST aqui toque o namespace de verdade. Alimente ele com a fixture e olhe os vereditos:

   ```bash
   curl -s -X POST -H "content-type: application/json" --data @../fixture.json http://localhost:8787/
   ```

   O loop é mais lento que o de TS, porque toda mudança de código reexecuta uma compilação Rust antes do reload, e isso é um custo honesto do pedágio que você escolheu. Ainda é um loop de compilar-e-testar na sua própria máquina, o que ganha de debugar através de um deploy todas as vezes.

7. **Faça o deploy e rode a trava.** Mesmo verbo de ontem, depois as duas linhas que fecham o módulo:

   ```bash
   npx wrangler deploy
   ```

   ```bash
   cargo run -p pulse-cli -- classify < ../fixture.json
   curl -s -X POST -H "content-type: application/json" --data @../fixture.json https://pulse-edge-rs.<your-subdomain>.workers.dev/
   ```

   Os mesmos vereditos JSON. Se você quer o comprovante em nível de byte, cuide da quebra de linha final da CLI e deixe o diff não dizer nada:

   ```bash
   diff <(cargo run -q -p pulse-cli -- classify < ../fixture.json) \
        <(curl -s -X POST -H "content-type: application/json" --data @../fixture.json https://pulse-edge-rs.<your-subdomain>.workers.dev/; echo)
   ```

   Um binário rodou no seu laptop. O outro rodou como WebAssembly em qualquer que fosse a cidade da Cloudflare mais perto de você. O classificador não sabe e nem se importa. Depois prove que o round-trip tem memória: dê um GET pelado com `curl -s` na URL e veja as últimas amostras enviadas por POST voltarem classificadas, `npx wrangler deploy` de novo, GET de novo. O KV sobrevive ao redeploy porque ele nunca esteve dentro do worker; o worker é sem estado e substituível, o namespace persiste.

![Os dois motores, TypeScript e Rust, se abrem em leque para as superfícies de deploy deles, com os dois edge workers alinhados sob um comando de deploy e um contrato de plataforma compartilhados.](assets/v08-diagram.webp)

## Challenge

Totalmente solo, dados puros do motor, nenhum I/O novo: exponha `GET /transitions` devolvendo a tabela de transições legais da máquina ProbeState como JSON. Você tem tudo: as regras de transição da m04-l3 moram no `next_state`, o roteador é o `match` no `url.path()` que chegou escrito no passo 4 (estender ele é uma edição de um braço só), e o `Response::from_json` serializa qualquer coisa `Serialize`. Seja preciso sobre o formato da tabela antes de codar, porque o `next_state` recebe TRÊS entradas, e uma tabela chaveada pelo par `(from, probe_ok, to)` não consegue expressar a escada de Degraded de jeito nenhum: para `(Degraded, false)` a resposta depende da contagem de falhas. Então as linhas da tabela são `(from, probe_ok, consecutive_failures, to)`, e enumerar `consecutive_failures` de 0 a 3 cobre toda mudança de comportamento, já que a única trava da máquina fica em três. Derive as linhas chamando o `next_state` em três loops aninhados em vez de escrever elas à mão; uma tabela derivada nunca pode descolar do código. Aceitação: a rota responde na URL ao vivo, as linhas mostram as regras da m04-l3 (as linhas `Degraded, false` viram Down exatamente quando a contagem chega a três, e qualquer sucesso, mesmo a partir de Down, recupera direto para Up), e o `cargo check` no projeto do worker continua limpo. Se você quer a checagem-espelho: a tabela que a sua rota serve deve casar com os braços de match que você escreveu na m04-l3, braço por braço, com a trava visível como a contagem onde as linhas de Degraded mudam.

## Checkpoint

Trave no fazer, duas colagens: o lado a lado de duas linhas do passo 7 do lab mostrando vereditos idênticos do `pulse-cli classify` e do curl, e a saída do GET-depois-do-redeploy provando que o round-trip do KV sobreviveu a um deploy. (Se você topou o challenge, a URL da sua rota `/transitions` respondendo é a terceira de bônus; como todo challenge neste curso ela é evidência extra, não a trava.) Aquela primeira colagem é a tese do módulo comprimida em duas linhas de terminal, e é uma vitória de 30 segundos de verdade para mostrar para alguém.

O que você consegue fazer agora, concretamente: compilar um crate Rust puro para wasm32-unknown-unknown e explicar a partir do próprio nome do target por que tokio e reqwest não podem vir junto; fazer o scaffold, ligar e fazer deploy de um projeto workers-rs cuja lógica é um crate de workspace em que você já confiava; ler um pin pré-1.0 como uma decisão em vez de desatualização; rodar KV a partir de Rust com serde nas duas pontas do cano; e colocar R2, D1, Durable Objects, Queues e Containers num mapa com a realidade de tier gratuito deles anexada.

A pergunta de recuperação antes de você fechar o terminal: o seu colega de time adiciona reqwest ao crate do motor "só para um helper rápido de sonda" e o build WASM quebra. Qual é o comentário de review de uma frase? (O I/O pertence à casca; o motor continua puro para que todo host, nativo ou unknown, consiga carregar ele.)

Se o worker-build ou o target wasm brigou com a sua máquina, me conte o OS e o erro no feedback do curso. O toolchain de edge do Rust é a coisa mais nova que este curso entrega, aquele pin pré-1.0 é honesto sobre isso, e relatos de falha de verdade decidem se a caixa de triagem desta lição cresce.

A estação agora roda em duas linguagens em quatro plataformas, e cada uma dessas superfícies já está sondando um endpoint RPC da Solana, numa blockchain cuja meta de tempo de slot caiu de 400ms para 300ms, um quarto mais curta, exatamente na semana em que este curso foi pesquisado; o próximo módulo te passa os instrumentos para conferir esse número você mesmo. O módulo 8 para de tratar aquele endpoint como só mais uma URL: leituras de kit a partir de TypeScript, JSON-RPC cru a partir de Rust, e uma transferência assinada na devnet para provar que você consegue escrever, não só olhar. Os motores estão prontos. Hora de apontar eles para a blockchain pra valer.
