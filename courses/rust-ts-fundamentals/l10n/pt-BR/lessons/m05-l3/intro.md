# clap, um braço de sonda de verdade, e um binário que qualquer um pode baixar

## Resumo

O m05-l2 moldou o código como o ecossistema: um workspace de dois crates com as deps declaradas uma vez, editions e MSRV escolhidas de propósito, e três pins do mundo real lidos como um dev que trabalha lê. O motor agora é uma biblioteca. E nada chama ele além dos testes. Hoje isso acaba três vezes de uma vez: a CLI ganha uma interface de verdade com clap, o caminho de sonda da estação finalmente toca uma URL viva por HTTP (pelo lado da CLI; o motor continua livre de I/O, exatamente como foi projetado), e o CI começa a publicar um binário que um estranho consegue baixar e rodar sem nunca ver o seu código-fonte. Este é o último build do tier de Rust, mais um artefato entregue na prateleira da estação, e os apoios são os mais finos do módulo: o derive do clap é trabalhado comigo, o fetch é guiado, as medições e o segundo subcomando são seus, e a extensão do workflow no fim é totalmente sem guia. Esse é o recuo terminando o que o M4 começou.

## Dez minutos até uma tela de help que você nunca escreveu

Faça isso antes de ler qualquer outra coisa. Na raiz do seu workspace `pulse-rs`, abra `crates/pulse-cli/Cargo.toml` e adicione uma dependência:

```toml
clap = { version = "4.6", features = ["derive"] }
```

(`cargo add clap --features derive` de dentro de `crates/pulse-cli` faz a mesma edição e escreve o dígito exato de hoje, 4.6.6. Atual no crates.io em 2026-09-02; a linha 4.x está estável desde 2022-09-28, então o pin é calmo. E sim, local do membro com uma versão inline, uma lição depois do sermão sobre hoisting: deliberado. A regra do m05-l2 vale o preço dela para deps COMPARTILHADAS, e o clap, como o reqwest que você adiciona mais tarde, tem exatamente um consumidor; faça o hoist dos dois para `[workspace.dependencies]` no dia em que um segundo crate quiser qualquer um deles.)

Agora substitua `crates/pulse-cli/src/main.rs` por este esqueleto. Repare no que a substituição estaciona, de propósito, não por esquecimento: o corpo do m05-l2 que lia `pulse.config.json` através do `parse_config` sai de cena, porque os subcomandos da CLI pegam o alvo deles do argv hoje. A fiação de config volta quando o poller do m06 rodar frotas inteiras num agendamento, e a assinatura congelada do `parse_config` é exatamente o que ele vai chamar; nada na superfície do motor muda nesse meio-tempo.

```rust
use clap::{Parser, Subcommand};

/// Pulse Station's Rust probe arm.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Probe one URL and print its status and latency
    Probe {
        /// The target to hit, scheme included
        url: String,
    },
    /// Run the latency-stats pass over a synthetic fixture set
    Report,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Probe { url } => println!("would probe {url}"),
        Command::Report => println!("would report"),
    }
    Ok(())
}
```

(Mantenha a assinatura `anyhow::Result<()>` do `main` vinda do m05-l2 mesmo que nada falhe ainda; o fetch que você conecta mais tarde usa `?`, que só compila dentro de uma função que retorna `Result`, e a cauda `Ok(())` é o custo inteiro.)

Rode (o hífen duplo sozinho é o separador do cargo: tudo depois dele vai para o SEU binário, não para o cargo):

```bash
cargo run -- --help
```

E olhe o que volta:

```text
Pulse Station's Rust probe arm

Usage: pulse-cli <COMMAND>

Commands:
  probe   Probe one URL and print its status and latency
  report  Run the latency-stats pass over a synthetic fixture set
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Uma tela de help formatada e versionada, com linhas de uso, resumos de subcomandos e um `-V` funcionando. Você não escreveu nada disso. Lá no M1 o tratamento de argumentos da CLI `pulse` em TS era o seu código e os seus bugs: o fatiamento do `process.argv`, a checagem de "será que passaram uma URL", a string de uso que você vivia esquecendo de atualizar. Aqui a interface é derivada, o mesmo truque que o serde puxou no módulo passado: o tipo é a spec, e a macro escreve a maquinaria. Para superfícies de CLI, é o melhor trade-off da caixa de ferramentas.

## De derivado a baixável

### O tipo é a interface

Olhe no que cada peça daquela struct se transformou. O doc comment na struct `Cli` virou a linha do about. O doc comment em cada variante do enum virou o resumo de uma linha dela na lista de comandos. O campo `url: String` virou um argumento posicional obrigatório, e porque ele é tipado, o clap fica com as reclamações:

```bash
cargo run -- probe
```

```text
error: the following required arguments were not provided:
  <URL>

Usage: pulse-cli probe <URL>

For more information, try '--help'.
```

Adicione uma flag tipada e o parse de valores vem de graça também. Dê um timeout ao `Probe`:

```rust
    Probe {
        /// The target to hit, scheme included
        url: String,
        /// Request timeout in seconds
        #[arg(long, default_value_t = 10)]
        timeout: u64,
    },
```

(O compilador vai apontar na hora que o braço do seu `match` agora precisa do campo novo: `Command::Probe { url, timeout }`. Deixe ele te guiar; isso é exaustividade trabalhando a seu favor, não contra você.) Agora `--timeout 5` faz o parse para um `u64`, sem flag quer dizer 10, e lixo é recusado com uma mensagem que nomeia a flag, o valor e a razão: `error: invalid value 'abc' for '--timeout <TIMEOUT>': invalid digit found in string`. Cada um desses comportamentos é código que você não escreveu e testes que você não mantém.

![Cada linha de uma struct derivada do clap se mapeia por seta ao texto de help, à flag ou à validação que ela gera.](assets/v01-annotated-code.webp)

O mecanismo importa porque você já conheceu o inverso dele. Rust não tem reflexão em tempo de execução: nada consegue inspecionar a sua struct enquanto o programa roda. Então `#[derive(Parser)]` faz o trabalho dele em tempo de compilação, lendo a definição da struct e gerando o código de parsing, validação e help ali mesmo, exatamente como `#[derive(Deserialize)]` fez pelo seu arquivo de config em m05-l1. Mesma jogada, alvo diferente: o serde deriva a fronteira de dados, o clap deriva a fronteira humana.

E aqui o seu músculo de leitura de pin da lição passada ganha um treino. O clap que você acabou de adicionar é o 4.6.6. O agave, a principal base de código de validador de Solana, fixa `clap = "2.33.1"`, um major de mais ou menos uma década atrás, e entrega ele para produção todo dia. Depois do m05-l2 você consegue ler esse pin em vez de ficar confuso com ele: um major de uma década de idade num repo mantido ativamente é uma decisão, sustentada por alguma coisa, provavelmente a pura superfície de migrar cada flag de CLI que um validador expõe. O seu projeto, o seu 4.x. O repo deles, o 2.x deles. Os dois corretos, e agora você sabe com o que um PR contra cada um deveria parecer.

Uma fronteira antes de a gente seguir: tudo acima é a API de derive do clap, structs entram, interface sai. O clap também tem uma API de builder onde você constrói o parser à mão em tempo de execução, e camadas de completion dinâmico e de help customizado abaixo disso. O uso diário é o derive. O resto fica como bookmark no fim desta seção.

### O braço de sonda, enfim

Dois módulos de Rust, e toda latência que o seu motor já classificou era um fixture. Esse foi o acordo que a gente fez em m05-l1, dito em voz alta: latências continuam fixtures, o braço de HTTP chega em m05-l3. É m05-l3. Adicione a segunda dependência em `crates/pulse-cli/Cargo.toml`:

```toml
reqwest = { version = "0.13", features = ["blocking"] }
```

O reqwest 0.13.4 é o atual em 2026-09-02, e você tecnicamente já conheceu este crate: ele foi a estrela do treino de leitura de pin da lição passada, onde o agave senta em 0.12.28, um major atrás. Agora você segura ele você mesmo. A feature `blocking` não é decoração opcional. A superfície padrão do reqwest é async, e sem a feature o módulo `reqwest::blocking` simplesmente não existe. Esqueça dela e o erro do compilador aponta para um módulo faltando, não para o seu Cargo.toml, o que faz dela um primeiro footgun genuinamente maldoso: o conserto mora num arquivo diferente do erro.

Por que blocking, quando todo tutorial de HTTP em Rust da internet corre para async? Por causa do que este programa é. Uma CLI sonda um alvo, imprime uma linha, e sai. Não há concorrência a explorar, então um runtime async seria puro overhead e um conceito novo gasto sem nenhum benefício. Blocking é a decisão certa de engenharia para este formato de programa, escolhida de propósito, não um atalho. O custo honesto: no momento em que você quiser cinquenta sondas em voo num agendamento, blocking vira o gargalo. Essa pressão exata é o problema de abertura do próximo módulo, e o delta de async cresce a partir deste mesmo crate. Uma mina para sinalizar agora para que ela nunca detone depois: o cliente blocking dá panic se for chamado de dentro de um runtime async. Tudo bem hoje, numa CLI simples sem runtime nenhum em lugar nenhum. Lembre disso em m06-l1.

Aqui está o fetch trabalhado, o formato da coisa toda:

```rust
use std::time::Instant;

fn probe(url: &str, timeout: u64) -> Result<(), ProbeError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout))
        .build()
        .map_err(|e| ProbeError::Unreachable {
            reason: e.to_string(),
        })?;

    let started = Instant::now();
    let response = client
        .get(url)
        .send()
        .map_err(|e| ProbeError::Unreachable {
            reason: e.to_string(),
        })?;
    let elapsed = started.elapsed();

    println!(
        "{url} -> {} in {} ms",
        response.status(),
        elapsed.as_millis()
    );
    Ok(())
}
```

(As quebras de linha dentro das closures de `map_err` e do `println!` são do rustfmt, não gosto pessoal: este é o formato em que o `cargo fmt` assenta, impresso já assentado para que o `fmt --check` da trava de CI não tenha nada de que reclamar. Digite mais apertado e a trava vai pedir exatamente isto.)

Vinte e poucas linhas, e metade delas é o caminho de erro. Aqui está a jornada inteira numa figura só antes de a gente dissecar essa metade.

![Uma sonda flui do parsing de argumentos pelo cliente blocking até uma linha de status impressa ou um erro mapeado.](assets/v02-flowchart.webp)

Leia o caminho de erro de perto, porque é o seu músculo do m04-l2 disparando numa academia nova. `send()` retorna um `Result` cujo tipo de erro é `reqwest::Error`, um estranho para a taxonomia `ProbeError` do seu motor. A closure `map_err` é a ponte, a mesma jogada que você usou para puxar o erro do serde para dentro do `BadConfig` na lição passada, e o erro de parse do `std` para dentro do `BadFixture` lá em m04-l2. A variante em que ele aterrissa é nova:

```rust
#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("not a latency reading: {0}")]
    BadFixture(ParseIntError),
    #[error("{0}ms is past the {MAX_SANE_LATENCY_MS}ms sanity ceiling")]
    OutOfRange(u64),
    #[error("u64 arithmetic overflowed")]
    Overflow,
    #[error("config rejected: {0}")]
    BadConfig(serde_json::Error),
    #[error("probe could not reach the target: {reason}")]
    Unreachable { reason: String },
}
```

(As quatro primeiras estão reimpressas exatamente como o m04-l2 e o m05-l1 deixaram elas, mensagens incluídas, para você poder dar diff disso contra o seu próprio arquivo: só a última variante é nova. Se as suas strings de `#[error]` estiverem diferentes porque você escreveu as suas, fique com as suas, as strings são suas para frasear; os FORMATOS das variantes é que são aquilo em que o resto do curso se apoia, e o `BadFixture(ParseIntError)` em particular é contra o que a síntese `.map_err(ProbeError::BadFixture)?` de m05-l1 compila.)

Repare no que o `Unreachable` carrega: uma `String`, não um `reqwest::Error`. Isso é deliberado, e é a decisão arquitetural da lição. O enum mora em `pulse-engine`, e se a variante segurasse o tipo de erro do reqwest, o motor ganharia uma dependência de reqwest, e o núcleo puro que você vem protegendo desde o M4 não seria mais puro. Por que essa pureza importa o bastante para achatar um erro numa string? Porque o motor tem mais futuros do que esta CLI: o m06 quer ele dentro de um poller de longa duração, e o m07-l2 quer compilar ele para WASM para um Worker do Cloudflare, um ambiente onde uma stack HTTP nativa não consegue seguir. Um motor livre de I/O porta; um motor com um socket dentro não. Então o braço de HTTP mora em `pulse-cli`, o motor continua uma calculadora, e a costura entre os dois é uma `String` atravessando uma fronteira de crate.

Uma promessa mantida honesta enquanto estamos aqui: este `probe` é uma chamada isolada, e ele NÃO implementa a trait `ProbeSource` que você congelou em m04-l3. O `drive` nunca vê essas latências hoje; a linha impressa é o produto inteiro. Nem o poller do m06 pluga ele mais tarde: esse daemon chama `next_state` direto com o resultado de cada sonda, que é o caminho mais curto quando existe exatamente um tipo de fonte e ela já roda sob um runtime async. Então a trait continua sendo o que o m04-l3 construiu ela para ser, a costura que deixa o `drive` rodar contra latências enlatadas em vez de um socket, que é precisamente o que o `tests/engine_contract.rs` do m05-l2 faz a partir do tier de integração, sentado pronto para o dia em que uma segunda fonte aparecer, e esse dia é a próxima lição: o m06-l1 fecha o loop com uma fonte de reqwest blocking plugada atrás da trait, bem aqui em `pulse-cli`, onde o cliente blocking continua legal. Se os seus dedos coçarem para escrever `impl ProbeSource for` uma fonte apoiada em reqwest agora mesmo, essa é uma coceira saudável e precisamente a jogada do m06-l1, então segure ela por uma lição; nada nesta lição, no lab ou no challenge espera isso.

![Um crate de motor puro sem I/O fica ao lado de um crate de CLI segurando clap e reqwest, com futuros consumidores poller e WASM acoplados ao motor.](assets/v03-diagram.webp)

Conecte o `probe` no braço do match, rode ele contra uma URL viva, e a era dos fixtures acaba:

```bash
cargo run -- probe https://www.rust-lang.org
```

```text
https://www.rust-lang.org -> 200 OK in 557 ms
```

Esse número é real, e ele não vai se repetir. A minha própria primeira execução imprimiu 557 ms, a seguinte 208 ms, reuso de conexão e cache de DNS sendo o que são. Latência é clima, não arquitetura. O que monta a pergunta que a próxima seção responde com um cronômetro: se os números balançam, como você faz uma afirmação de performance afinal?

### Dois binários, duas personalidades

Tudo que você construiu até agora rodou através do `cargo build`, o que quer dizer o perfil dev: compilações rápidas, código lento, falhas barulhentas. `cargo build --release` produz um binário diferente a partir do mesmo código-fonte, e as diferenças não são cosméticas. Otimizações vão de essencialmente nenhuma a completas. Asserções de debug desligam. E overflow de inteiro para de dar panic e começa a dar wrap, a mudança exata de personalidade que você conheceu em m04-l2 como regra; hoje ela ganha um número de lab anexado.

Quão diferente? Eu cronometrei a passagem de latency-stats do motor, a caminhada de soma com checked_add do m04-l2, sobre cinco milhões de amostras sintéticas de latência, vinte passagens, os dois perfis, na minha própria máquina:

| perfil | 20 passagens sobre 5M amostras | tamanho do binário |
|---|---|---|
| dev (`cargo build`) | 983 ms | 18M |
| release (`cargo build --release`) | 67 ms | 6.4M |

Quinze vezes mais rápido, e o binário encolheu para mais ou menos um terço (informação de debug é pesada). Os seus números vão ser diferentes, e isso é em parte o ponto: o lab faz você produzir o seu próprio par, porque a razão é a lição durável, não os meus dígitos.

![Barras mostram um build de release rodando a mesma carga de trabalho mais ou menos quinze vezes mais rápido que o build de dev enquanto produz um binário menor.](assets/v04-chart.webp)

Duas consequências, as duas obrigatórias. Primeira: qualquer afirmação de performance feita a partir de um binário de debug é uma mentira. Não exagerada, uma mentira, errada por uma ordem de grandeza. Meça `--release`, sempre; os números de performance deste curso seguem essa regra. Segunda, mais sutil: os perfis discordam sobre aritmética. Builds de dev dão panic no overflow, builds de release dão wrap em silêncio, então uma soma de `u64` que quebra com honestidade no seu laptop pode entregar lixo a partir do CI. É exatamente por isso que o m04-l2 fez você escrever `checked_add` em vez de `+`: a sua aritmética retorna `Err(Overflow)` nos dois perfis, e a mudança de personalidade não consegue te tocar. A disciplina nunca foi sobre estilo. Era sobre fazer os dois binários dizerem a verdade.

![Colunas lado a lado contrastam os perfis de dev e de release enquanto um rodapé nota que aritmética checada se comporta igual nos dois.](assets/v05-comparison.webp)

O ritmo a internalizar: desenvolva em dev, meça e entregue em release. Os botões por trás das personalidades (`opt-level`, `debug-assertions`, `overflow-checks`) moram em seções `[profile]` do Cargo.toml e podem ser sobrescritos por projeto; saber que eles existem já basta para este curso.

### Entregue para um estranho

O último movimento é curto porque a plataforma não é nova. O seu `.github/workflows/pulse.yml` vem acumulando jobs desde o M1: a sonda TS, a trava de typecheck, o job `rust` do m04-l3 rodando test, clippy e fmt. Hoje ele aprende a distribuir binários. Nenhuma plataforma nova, um job novo.

Três conceitos, glosados uma vez. Um **GitHub Release** é um objeto de primeira classe anexado a uma tag do git: um título, notas e arquivos baixáveis. Um **release asset** é um desses arquivos, e ele é servido para qualquer um que tenha a URL, sem conta, sem toolchain, sem clone. E um **tag push** (`git tag v0.1.0 && git push origin v0.1.0`) é o evento que vai disparar o nosso, que é o contrato convencional: merges para a main rodam travas, tags cortam releases. As peças que você já tem cobrem o resto: o workflow precisa de `permissions: contents: write` pela mesma razão que o seu job de commit-back precisou em m01-l3, e a imagem do runner já traz tanto um toolchain estável de Rust (o job do m04-l3 já se apoia nele) quanto a CLI `gh`, que consegue criar um release e anexar arquivos num comando só.

![Uma tag pushada flui pela trava de teste e pelo build de release até um asset baixável que um estranho roda numa máquina sem o código-fonte.](assets/v06-flowchart.webp)

Honestidade sobre o que isto entrega: um binário de alvo único, x86_64 Linux, construído no runner. Distribuição de verdade cresce matrizes de alvo, assinatura para macOS e Windows, e checksums; esses são nomeados aqui e ensinados em lugar nenhum deste curso, porque um alvo só já basta para descontar a afirmação que importa. E a afirmação vale ser dita com franqueza: o cargo transforma "funciona na minha máquina" num arquivo que você pode passar para um estranho. O tier do interpretador nunca conseguiu dizer isso. A sua frota TS é entregue como código-fonte mais um lockfile mais uma versão do Node mais um passo de install; este aqui é entregue como um arquivo só que já É o programa. Você vai construir o job você mesmo no challenge. Não antes.

**Vá mais fundo (os 20%).** esta lição ensinou clap em nível de derive, um fetch blocking e um pipeline de release de alvo único, o que cobre trabalho diário de CLI. As camadas abaixo ficam como bookmark: o tutorial de derive do próprio clap percorre cada atributo que o derive suporta, incluindo a API de builder embaixo dele, em [https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html), e a documentação do módulo blocking do reqwest cobre as opções de cliente que a gente pulou, timeouts, headers, política de redirect, em [https://docs.rs/reqwest/latest/reqwest/blocking/index.html](https://docs.rs/reqwest/latest/reqwest/blocking/index.html). As duas URLs checadas ao vivo em 2026-09-02. Middleware, tuning de pool de conexão e cross-compilation também ficam como bookmark. O lab não precisa de nenhum deles.

## Lab: a CLI ganha o nome dela

Trabalhe a partir da raiz do workspace `pulse-rs`. Estimados 60 a 75 minutos.

1. **A interface (trabalhada, quase pronta).** Se você fez a abertura, tem o clap conectado com os subcomandos `probe` e `report` e a flag `--timeout`. Faça Checkpoint dos três comportamentos gratuitos, um comando cada:

   ```bash
   cargo run -- --help                 # prints the derived help
   cargo run -- probe                  # refuses with the missing-URL error
   cargo run -- probe x --timeout abc  # rejects the value by name
   ```

   Três comportamentos, zero linhas de código de tratamento seu.

2. **A taxonomia ganha uma variante (guiado).** No `pulse-engine`, adicione `Unreachable { reason: String }` ao `ProbeError` com uma mensagem `#[error(...)]` como a minha acima. O compilador não vai te forçar a atualizar matches antigos a não ser que você faça match exaustivamente em algum lugar; confira qualquer `match` sobre `ProbeError` que você escreveu no M4 e estenda ele de propósito, não por wildcard.

3. **O fetch (guiado, você escreve a ponte).** Adicione a dependência do reqwest, depois escreva o `probe` a partir da minha versão trabalhada mas deixe as duas closures de `map_err` de fora e tente compilar. Leia o erro resultante por inteiro: ele nomeia `reqwest::Error` e o tipo de erro do seu `Result`, e ler ele de ponta a ponta é a mesma disciplina que o E0382 treinou em você em m04-l1, informação, não obstrução. Agora escreva as duas pontes você mesmo. Checkpoint:

   ```bash
   cargo run -- probe https://www.rust-lang.org
   ```

   Isso imprime um status real e uma latência real. Depois sonde uma URL que não consegue resolver e confirme que você recebe a sua própria mensagem de `Unreachable`, não um panic.

4. **O segundo subcomando (seu).** Implemente o `report` para que a CLI de Rust espelhe o formato da CLI `pulse` em TS: ela deve rodar a passagem de latency-stats do motor (a caminhada de checked_add do m04-l2; se a sua carregar um nome diferente, fique com ele) sobre um conjunto de fixtures gerado e imprimir média, máximo e contagem de amostras. Os nomes que você precisa, `total_latency` e `LatencyMs`, vêm direto do `pulse_engine::`, os dois na lista de re-export que o m05-l2 curou exatamente para este consumidor. A minha gera cinco milhões de amostras com um embaralhador de multiplica-e-soma com seed para que as execuções sejam comparáveis, e imprime uma linha: `mean=479 ms, max=929 ms over 5000000 samples; 20 passes took 983 ms`. Vinte passagens existem puramente para deixar o próximo passo mensurável. Sem apoio neste aqui, e para ser honesto sobre o que "tudo de que ela precisa" quer dizer: a passagem de stats já está no seu motor, e o gerador de fixtures é escolha sua, porque nada no curso ensinou PRNGs e nada aqui exige um. Qualquer fonte determinística passa: um embaralhador com seed se você curte o flex, ou simplesmente uma fatia pequena de latências reciclada até cinco milhões (`[212u64, 487, 930, 479].iter().cycle().take(5_000_000)` já basta). A sua média e o seu máximo impressos vão refletir o seu gerador, não o meu; o entregável que o passo 5 avalia é a razão entre dev e release, que qualquer uma dessas produz.

5. **Meça as duas personalidades (seu).** Construa os dois perfis e rode o mesmo report:

   ```bash
   cargo build --workspace
   cargo build --release --workspace
   ./target/debug/pulse-cli report
   ./target/release/pulse-cli report
   ls -lh target/debug/pulse-cli target/release/pulse-cli
   ```

   Anote todos os quatro números: os dois tempos, os dois tamanhos. Esse par escrito é um entregável do lab, e a barra de aceitação é honesta: a sua razão não vai ser o meu 15x, mas um build de dev que não é várias vezes mais lento que o de release quer dizer que o seu report ainda não está fazendo trabalho de verdade.

6. **Veja a virada de personalidade do overflow (2 minutos).** Largue este arquivo como `crates/pulse-cli/src/bin/overflow_demo.rs` (qualquer arquivo em `src/bin/` vira o próprio binário, uma convenção do cargo que vale conhecer):

   ```rust
   fn main() {
       let start: u8 = std::env::args()
           .nth(1)
           .and_then(|s| s.parse().ok())
           .unwrap_or(250);
       let mut v = start;
       for _ in 0..10 {
           v += 1;
       }
       println!("{v}");
   }
   ```

   `cargo run --bin overflow_demo` dá panic com `attempt to add with overflow`. `cargo run --release --bin overflow_demo` imprime `4`, em silêncio, código de saída zero. Mesmo código-fonte, 250 mais 10 deram wrap num `u8`. Delete o demo depois que ele tiver te perturbado direito, e note que o seu subcomando report é imune: as somas dele passam por `checked_add`.

7. **Verde antes de entregar.** Rode `cargo fmt` primeiro, depois o triplo local (`cargo test && cargo clippy -- -D warnings && cargo fmt --check`); um código digitado à mão do tamanho de uma lição quase sempre carrega uma quebra ou duas que o rustfmt quer de volta, e descobrir isso localmente custa segundos onde descobrir no CI custa um push. Depois dê push no branch e confirme que o job rust do m04-l3 passa no código novo: test, clippy, fmt, tudo verde. O job de release que você está prestes a escrever vai sentar downstream desta trava, que é o ponto de ter construído a trava primeiro.

## Challenge

Totalmente sem guia, e é a checagem interina deste degrau. Estenda o `pulse.yml` para que um tag push publique o seu binário:

- O workflow hoje dispara em `push` para a `main` e no agendamento. Faça ele disparar também em tags no formato `v*`, e mantenha o job de commit-back da sonda fora das execuções de tag (um checkout de tag não é um branch; um push a partir dele falha. A trava é uma condição `if:` no nível do job, nova bem aqui; a documentação de expressões do GitHub tem a sintaxe, e o job de referência abaixo mostra ela, depois da sua tentativa, não antes).
- Adicione um job `release` que roda só em refs de tag, dá `needs` no job rust, constrói `--release`, e usa `gh release create` para publicar um Release com o binário anexado como `pulse-cli-linux-x86_64`. Tudo que é exigido está nomeado na seção de entrega: a permissão, a variável de ambiente do token (`GH_TOKEN: ${{ github.token }}`), o `gh` pré-instalado, e o working directory do seu workspace.
- Marque com a tag `v0.1.0`, dê push na tag, assista ao pipeline, e então faça o teste de aceitação de verdade: baixe o asset numa máquina que nunca viu o seu código-fonte e rode `./pulse-cli-linux-x86_64 probe https://www.rust-lang.org`. A caixa Linux de um amigo ou qualquer VM Linux ou WSL serve. Se você está no macOS e o Docker por acaso já estiver instalado, esta linha única dentro de um contêiner funciona com zero conhecimento de Docker (a gente desmistifica tudo isso no próximo módulo): `docker run --rm -it ubuntu:24.04 bash`, depois lá dentro, `apt-get update && apt-get install -y curl ca-certificates`, baixe a URL do asset com curl, `chmod +x`, rode. Apple Silicon precisa de `--platform linux/amd64` no comando run. O asset é só para Linux e isso é dito com honestidade, não pedindo desculpas: alvo único, limites ensinados.

Aceitação: o comando de verify verde, quer dizer

```bash
cargo run --release -- probe https://www.rust-lang.org
```

imprime um status e uma latência a partir do seu binário; as duas medições de perfil anotadas do lab; e o Release asset baixado imprimindo o mesmo formato de linha de sonda numa máquina limpa. Quando o seu funcionar, compare contra o meu job abaixo. Depois, não antes; o review do diff é onde está o aprendizado, e uma rep sem guia que você espiou é uma rep guiada com passos extras.

```yaml
on:
  push:
    branches: [main]
    tags: ["v*"]

# ...schedule and existing jobs unchanged; probe job gains:
#   if: github.ref == 'refs/heads/main' || github.event_name == 'schedule'

  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: rust
    runs-on: ubuntu-latest
    permissions:
      contents: write
    defaults:
      run:
        working-directory: pulse-rs
    steps:
      - uses: actions/checkout@v7
      - run: cargo build --release --workspace
      - name: Attach the binary to a GitHub Release
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          cp target/release/pulse-cli pulse-cli-linux-x86_64
          gh release create "$GITHUB_REF_NAME" pulse-cli-linux-x86_64 --title "$GITHUB_REF_NAME" --generate-notes
```

## As portas que a gente deixou fechadas

Isto fecha o tier de Rust, então deixa eu te passar o mapa do que a gente deliberadamente não ensinou, porque saber onde uma porta está é melhor do que fingir que ela não existe. Três portas: lifetimes além da leitura, `unsafe`, e autoria de macros (mais os internals de async, que o m06 vai nomear de novo). As duas primeiras moram atrás do Rustonomicon, o livro oficial de Rust das artes das trevas, cujo próprio aviso de abertura eu vou simplesmente citar, verificado na página em 2026-09-01: "Se você deseja uma carreira longa e feliz escrevendo programas em Rust, deveria voltar agora e esquecer que um dia viu este livro. Ele não é necessário." A documentação oficial te dizendo para não ler é o argumento 80/20 inteiro deste tier feito pelos próprios mantenedores da linguagem. Ele fica em [https://doc.rust-lang.org/nomicon/](https://doc.rust-lang.org/nomicon/) para o dia em que você genuinamente precisar dele, e esse dia não é pré-requisito para nada que você queira fazer em seguida. A terceira porta, autoria de macros, mora em outro lugar, e ela surpreende quem assume que todo Rust avançado se esconde num livro só: o capítulo de macros do The Rust Programming Language cobre `macro_rules!` e um primeiro olhar em macros procedurais, e o The Little Book of Rust Macros é onde moram os padrões profundos. O Rustonomicon não tem capítulo de macros nenhum; você já usou macros pelo lado do consumidor toda vez que digitou `println!`, e consumir é o único lado de que este curso precisa.

![Um mapa mostra as habilidades que o tier ensinou no centro, recursos de treino gratuitos ao lado, e portas fechadas rotuladas com onde moram os tópicos mais profundos de Rust.](assets/v07-diagram.webp)

O que você deveria de fato fazer em seguida é treinar, não descer. O Rustlings (`cargo install rustlings`, depois `rustlings init` e `rustlings`) continua sendo o pátio de treino, reps pequenas de conserta-o-código mantidas pelo próprio projeto Rust. E o Comprehensive Rust, em [https://google.github.io/comprehensive-rust/](https://google.github.io/comprehensive-rust/), é o curso que o time de Android do Google usa para fazer o onboarding de engenheiros que trabalham para dentro do Rust, gratuito, mantido ativamente (o repo dele recebeu commits no dia exato em que a pesquisa deste curso rodou, 2026-09-01). O curso que uma empresa de um trilhão de dólares usa para retreinar engenheiros de C++ não te custa nada; essa é a barra de recurso gratuito em que os bookmarks deste tier vêm se apoiando esse tempo todo. Uma opção paga merece uma menção porque ela nomeou o público exato deste curso três anos antes da gente: "Rust for TypeScript Developers" do ThePrimeagen, 5h19m, publicado em 2023-04-25, pago com um preview gratuito. Divulgado uma vez, aqui, ao lado do cânone gratuito.

E para onde o Rust vai on-chain? Por uma porta diferente do Nomicon, e isto importa: escrever programas Solana não exige turismo de unsafe nem bruxaria de lifetime. Leia as placas com honestidade, porém. O curso Mastering Anchor neste catálogo é a parte funda, não a rampa de entrada: a barra dele é que você já entrega programas Anchor e quer o delta do V2. O que o seu nível de saída aqui te compra é o nível de leitura para o código daquele mundo, structs, enums, traits, `Result`, pensamento com formato de serde; a distância que resta é experiência de entrega, não vocabulário. E os conceitos de conta e de transação por baixo da autoria de programas, o que um programa de fato recebe e por quê, pertencem ao curso de evolução do Bitcoin à Solana. Nível de leitura daqui, conceitos de lá, reps por sua conta: essa é a rota honesta on-chain.

## Checkpoint

O que você consegue fazer agora, concretamente: derivar uma CLI tipada cujo help, erros de parse e defaults são gerados a partir do tipo; fazer requisições HTTP de verdade a partir do Rust e fazer a ponte de erros estrangeiros para dentro da sua própria taxonomia sem poluir um crate puro; medir a diferença entre dev e release com os seus próprios números e dizer com precisão o que mudou entre os perfis; e publicar um binário através do CI que roda numa máquina que nunca viu o seu código-fonte. A metade Rust da estação não é mais uma biblioteca que só os testes conseguem amar. Ela é uma ferramenta.

A recuperação de 30 segundos antes de você fechar a aba, de memória: nomeie as três portas de Rust profundo que este tier deixou fechadas e onde cada uma mora. (Lifetimes além da leitura e unsafe, as duas atrás do Rustonomicon, cujo próprio aviso te mandou voltar; autoria de macros, atrás do capítulo de macros do TRPL e do The Little Book of Rust Macros. E Rust on-chain não está atrás de nenhuma delas: essa é a porta do curso Anchor.)

Um pedido enquanto o tier está fresco: de tudo no M4 e no M5, me diga no feedback qual conceito único te custou mais tempo de relógio, e se o retorno chegou até hoje ou ainda é uma promissória. A aposta inteira do tier é que os 20% que a gente ensinou cobrem os seus 80% do dia a dia, e o seu relatório de atrito é o único instrumento que mede se a aposta deu certo.

O seu binário sonda de verdade, e então ele sai. Uma estação precisa de um batimento que não saia. No próximo módulo o mesmo crate de motor vai para dentro de um poller tokio de longa duração com um endpoint `/status`, o fetch blocking que você escreveu hoje ganha o delta async dele sob pressão de concorrência de verdade, e a coisa toda vai para dentro de uma caixa. Da próxima vez que você iniciar o processo, planeje deixar ele rodando.
