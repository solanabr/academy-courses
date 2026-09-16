# domínio do cargo: editions, workspaces, features, MSRV

## Resumo

O m05-l1 deu serde ao motor: o `pulse-rs` faz o parse do mesmo arquivo de config que a frota TS parseia com zod, os tipos de sonda moram num enum com tag, e o seu primeiro pipeline de iterador filtra e mapeia a lista de alvos. Tudo isso ainda mora num crate só. Esta lição conserta o formato, não o código: você divide o `pulse-rs` num workspace cargo de dois crates nos primeiros dez minutos, depois passa o resto da sessão aprendendo a ler o arquivo que essa divisão produziu, porque o Cargo.toml é onde editions, MSRV, features e os pins de versão dos outros todos moram. Como as reps rodam desta vez: a divisão em si é trabalhada comigo, o hoisting das dependências é guiado pelos erros do próprio compilador, declarar o seu MSRV é com você, e o treino final de leitura de pin é totalmente sem guia. Esse último é o músculo que esta lição existe para construir.

## A negociação

O Cargo.toml parece configuração. Ele é na verdade uma negociação com toda máquina que um dia vai construir o seu código: o seu laptop, o CI, o toolchain de três anos de idade de um contribuidor, o resolvedor escolhendo versões numa máquina que você nunca vai ver. Toda linha dentro dele é um termo desse contrato. A gente vai escrever um que valha a pena assinar, começando agora, divisão primeiro, teoria depois.

A partir da raiz do `pulse-rs`:

```bash
cargo new crates/pulse-engine --lib
cargo new crates/pulse-cli
```

Dois crates novos, um biblioteca, um binário. Agora substitua o `Cargo.toml` raiz inteiro (aquele que carrega o projeto todo desde o m04-l1) por três linhas:

```toml
[workspace]
resolver = "3"
members = ["crates/pulse-engine", "crates/pulse-cli"]
```

Aquela linha do meio ganha a explicação dela mais adiante nesta lição. Mova o código: todo módulo exceto o `main.rs` vai para o motor, o `main.rs` vai para a CLI. Uma cilada primeiro: o `cargo new` já largou um `main.rs` hello-world dentro de `crates/pulse-cli/src/`, e o `git mv` se recusa a sobrescrever um destino existente (`fatal: destination exists`), então apague o stub antes de mover:

```bash
git mv src/config.rs src/engine.rs crates/pulse-engine/src/
rm crates/pulse-cli/src/main.rs
git mv src/main.rs crates/pulse-cli/src/main.rs
rm -rf src
```

Os seus nomes de arquivo podem diferir dos meus dependendo de como você recortou o m05-l1; a regra não difere: tudo que é puro vai para o motor, o ponto de entrada vai para a CLI. Dê ao motor as dependências dele colando o bloco `[dependencies]` antigo dentro de `crates/pulse-engine/Cargo.toml`, menos uma linha: o `anyhow` fica fora do motor, porque é maquinaria do lado do binário pelo cânone do m04-l2 (thiserror na biblioteca, anyhow no binário) e em vez disso ele se move para o manifest da CLI abaixo:

```toml
[package]
name = "pulse-engine"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
thiserror = "2.0.20"
```

(Esses são os dígitos atuais no crates.io enquanto eu escrevo, sondados em 2026-09-02; os seus são o que quer que o m05-l1 tenha deixado com você, e está tudo bem por ora, o lab revisita eles.) O `src/lib.rs` do motor declara os módulos e re-exporta os nomes que o mundo de fora pode usar:

```rust
pub mod config;
pub mod engine;

pub use config::{Config, ProbeKind, ProbeTarget, Target, parse_config};
pub use engine::{
    FixtureSource, LatencyMs, ProbeError, ProbeState, drive, next_state, total_latency,
};
```

Cure essa lista contra os consumidores dela, não contra o `main.rs` de hoje sozinho: `next_state`, `LatencyMs` e `total_latency` estão nela porque o braço de report do m05-l3 e o poller do m06 chamam os três por estes nomes `pulse_engine::` pelados. Uma lista de re-export que só cobre o chamador atual força todo consumidor futuro a fuçar caminhos de módulo `pulse_engine::engine::`, o que anula o sentido de nomear uma superfície pública.

A CLI depende do motor por path e mantém só o que um binário fino precisa:

```toml
[package]
name = "pulse-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
pulse-engine = { path = "../pulse-engine" }
anyhow = "1.0.104"
```

E o `main.rs` encolhe para um consumidor, chamando a assinatura exata de `parse_config` que o Checkpoint do m05-l1 te mandou não mexer:

```rust
use pulse_engine::{Config, parse_config};

fn main() -> anyhow::Result<()> {
    let raw = std::fs::read_to_string("pulse.config.json")?;
    let config: Config = parse_config(&raw)?;
    for target in &config.targets {
        println!("{} -> {:?}", target.name, target.kind);
    }
    Ok(())
}
```

Conserte os caminhos de `use` nos módulos movidos (`crate::engine::ProbeError` continua funcionando dentro do motor; qualquer coisa que o `main.rs` usava agora vem por `pulse_engine::`), depois:

```bash
cargo check --workspace
```

Verde. Dez minutos depois, e o seu Rust agora tem o formato de todo repo Rust sério que você um dia vai abrir. O agave, o cliente validador em que este ecossistema inteiro roda, é exatamente esta estrutura em escala maior: um manifest raiz, uma lista de membros no estilo `crates`, bibliotecas no meio, binários na borda.

![Um crate segurando três arquivos vira um workspace onde uma CLI fina e consumidores futuros apontam todos para uma biblioteca de motor pura.](assets/v01-diagram.webp)

Por que este formato, e por que agora? Porque o M4 fez você pagar por pureza: o classificador, os tipos de config, o enum de erro todos recebem valores e devolvem valores, sem I/O em lugar nenhum perto deles. Esse investimento começa a render aluguel hoje. Um núcleo puro compila para uma biblioteca que qualquer coisa consegue consumir: a CLI que você acabou de fazer, a suíte de testes, o daemon que este curso constrói mais tarde, até um Worker WASM no tier de deploy. A borda impura fica fina e trocável. Se essa música soa familiar, deveria mesmo: é o m03-l1 nota por nota, onde você extraiu o `pulse-core` para dentro de um workspace pnpm para o painel e o cron poderem compartilhar ele. Monorepo-lite é uma ideia usando dois toolchains, e você agora construiu ela nos dois.

### Uma declaração, muitos assinantes

Agora mesmo cada membro declara as próprias versões de dependência no próprio manifest: o motor carrega dígitos para serde, serde_json e thiserror, e a CLI carrega dígitos para anyhow mais a própria grafia em `path` da dependência do motor. Nada está duplicado ainda, e essa é exatamente a cilada: o primeiro crate futuro que precisar de um destes, e este workspace vai crescer, pega os dígitos dele por copiar e colar de qualquer manifest que você por acaso abrir, e daquele momento em diante dois crates podem sentar em linhas diferentes sem nenhum diff sozinho dizer isso. Dois manifests, dois lugares para as versões descolarem uma da outra, e hoje, enquanto a contagem é pequena, é o momento barato de fechar a porta. O agave tem centenas de crates internos, e nessa escala a deriva é um incidente de cadeia de suprimentos esperando um diff de lockfile que ninguém lê. A resposta deles, e o padrão que você adota no lab, é `[workspace.dependencies]`: declare cada dependência compartilhada uma vez na raiz, com a versão e as features dela, e deixe os crates membros assinarem.

Aqui está o manifest raiz que você vai ter no fim do lab:

```toml
[workspace]
resolver = "3"
members = ["crates/pulse-engine", "crates/pulse-cli"]

[workspace.dependencies]
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
thiserror = "2.0.20"
anyhow = "1.0.104"
pulse-engine = { path = "crates/pulse-engine" }
```

E o bloco de dependências de cada membro se resume a assinaturas:

```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

Repare no que o membro NÃO carrega: dígitos de versão. A raiz declara; o membro opta por entrar com `workspace = true`. Membros que nunca mencionam serde nunca ganham ele, que é o ponto, injeção automática incharia todo crate com toda dependência. Um bump de versão vira um diff de uma linha na raiz, e dois crates do workspace não podem sentar em silêncio em linhas de serde diferentes. Este é o formato observado do agave, não uma invenção do curso: o manifest raiz deles declara cada dependência compartilhada exatamente uma vez, features e tudo, sondado ao vivo em 2026-09-02.

![O manifest raiz declara serde uma vez, o motor assina e recebe ele, a CLI não assina e não ganha nada, um lockfile só abrange os dois.](assets/v02-flowchart.webp)

Antes da segunda metade da higiene, deixe assentado o que uma feature de fato é, porque a gente vem usando uma desde o m05-l1 sem nomear ela. Uma feature é uma flag nomeada que um autor de crate expõe e que trava código opcional e dependências opcionais atrás de compilação condicional. `serde = { version = "1.0.229", features = ["derive"] }` é você virando a flag `derive` do serde, o que puxa o crate de proc-macro `serde_derive` e a maquinaria por trás de `#[derive(Deserialize)]`; deixe a flag desligada e aquela subárvore inteira nunca compila. Features são aditivas por convenção: ligar uma adiciona capacidade, nunca remove, que é o que deixa o cargo fazer unificação de features, construindo cada crate uma vez com a união de toda feature que qualquer crate do grafo pediu. E todo crate já vem com uma feature especial chamada `default`, o conjunto que o autor liga para você a menos que você diga o contrário.

É aí que entra a segunda metade da higiene do agave: `default-features = false`. Todo autor de crate escolhe um conjunto de features default para o consumidor médio, e um workspace não é médio. Desligar os defaults e nomear só as features que você usa quer dizer que toda capacidade do seu build foi escolhida de propósito: binários menores, builds mais rápidos e um manifest que documenta para o que cada dependência de fato serve. O agave aplica isso com julgamento em vez de dogma, o que também vale copiar. O serde deles mantém os defaults ligados com `derive` adicionado; o reqwest e o clap deles rodam defaults-off porque esses crates carregam maquinaria opcional pesada (stacks de TLS, helpers de terminal) que um validador não quer por acidente.

A pegadinha, e você vai bater nela de propósito no lab: quando você desliga uma feature default de que alguma coisa precisava, o erro não diz isso. Ele aparece como um item faltando dentro do código da própria DEPENDÊNCIA, uma trait não implementada, um módulo que parece ter sumido de um crate que definitivamente tem um. O compilador está dizendo a verdade, o item foi compilado para fora, mas ele aponta para o fonte deles, não para a sua linha de manifest. O debugger disso é o `cargo tree -e features`, que imprime o grafo de dependências resolvido com toda aresta de feature visível. Para arqueologia de manifest não existe nada melhor, e ele já vem dentro do cargo, nada para instalar:

```bash
cargo tree -e features -p pulse-engine
```

Rode ele agora contra o workspace que você acabou de dividir e leia as arestas do serde antes de o lab fazer você precisar delas.

O trade-off, nomeado antes de você se apegar: workspaces custam acoplamento. Todo membro resolve contra o mesmo grafo de dependências, um lockfile, uma negociação. Se um dia a CLI quiser um major novo e brilhante de algum crate e o motor depender de algo que fixa o antigo, a CLI espera. O apetite de upgrade de um crate pode ficar refém da restrição de outro, e daqui a alguns minutos você vai ver exatamente até onde isso pode ir dentro do agave. Versões compartilhadas e deriva zero, compradas com destino compartilhado. Para um projeto como o nosso, e para a maioria, o trade-off vale a pena ser aceito de propósito.

### Editions, com honestidade

Os dois crates novos seus dizem `edition = "2024"`, porque o `cargo new` escreveu isso. Hora de saber o que você assinou. Uma edition é o mecanismo do Rust para fazer mudanças que quebram a linguagem sem quebrar ninguém: o seu crate declara contra qual conjunto de regras ele foi escrito, e o compilador cobra de todo crate a declaração dele próprio, para sempre. Crates em editions diferentes linkam juntos sem problema. É por isso que um crate de edition 2015 ainda compila hoje e por que editions são contratos por crate, não eventos do ecossistema.

A edition 2024 é a atual. Ela foi entregue no Rust 1.85.0 em 2025-02-20, e no nosso nível mudou três coisas que você de fato vai notar: o resolvedor de dependências passa a ter v3 como default (próxima seção), blocos `extern` precisam ser marcados como `unsafe`, e pegar referências a `static mut` é negado. O mesmo release também entregou async closures, e aqui está uma distinção que a maioria dos posts de blog erra: async closures são uma feature de linguagem do Rust 1.85.0, disponível em toda edition, não travada atrás da 2024. "O Rust 1.85 entregou a edition 2024 mais async closures" é a frase precisa, e ela importa porque te diz o que uma edition não é: features novas chegam com releases do compilador; editions só abrigam as mudanças de regra incompatíveis.

Agora a parte honesta. Eu pesquisei os manifests de cinco repos Rust adjacentes à Solana enquanto pesquisava este curso (agave, yellowstone-grpc, photon, jito-relayer, carbon, sondados em 2026-09-01), e quatro dos cinco ainda declaram `edition = "2021"`. Só o agave migrou. O trem sai no horário; o ecossistema embarca atrasado. Então a gente ensina 2024 e a gente escreve 2024, mas ninguém deveria fingir que o ecossistema migrou, e a regra do contribuidor cai direto disso: quando você abre PR num repo 2021, você escreve 2021. Editions são o contrato do repo, não a sua preferência pessoal, e copiar `edition = "2024"` de um tutorial para dentro do crate de outra pessoa é exatamente o tipo de contribuição de passagem que ninguém mergeia. Para os seus PRÓPRIOS crates, quando a próxima edition enfim chegar, a taxa de embarque é pequena: o `cargo fix --edition` reescreve os padrões incompatíveis mecanicamente, você vira o dígito, você revisa o diff. O guia de editions documenta essa dança de ponta a ponta, que é parte do porquê de o trem conseguir continuar saindo no horário.

E uma divulgação com data nela, 2026-09, porque o catálogo em que este curso mora tem uma regra que parece contradizer esta lição, e você deveria ouvir isso de mim. Os cursos de Solana on-chain da Academy escrevem `edition = "2021"` por regra, e para eles 2024 não está fora de moda, está rejeitada: o Rust avaliado deles compila num servidor de build cujo toolchain ainda não aceita a edition 2024, então escrever 2024 lá produz um build quebrado, não uma discordância de estilo. Este curso é um recorte deliberado dessa regra, e ele é seguro por exatamente duas razões: toda linha de Rust aqui compila na sua própria máquina com o seu próprio rustc atual, e os challenges avaliados são exercícios de arquivo único que compilariam idêntico sob qualquer uma das duas editions. Então a gente ensina a edition atual de propósito, e quando você depois se matricular num curso on-chain, você já tem a regra que resolve a tensão: o servidor de build é o repo. Acompanhe a edition DELE.

![Quatro editions do Rust são entregues em intervalos de três anos enquanto uma pesquisa de cinco repos mostra a maior parte do ecossistema ainda na edition anterior.](assets/v03-timeline.webp)

Mais uma coisa que o seu manifest raiz de três linhas já resolveu, e agora eu consigo explicar. A raiz de um workspace como o nosso é um manifest virtual: ele não tem `[package]`, então não tem `edition`, então o cargo não consegue inferir qual resolvedor você quis dizer. Deixe `resolver = "3"` de fora e o cargo te avisa disso em todo build:

```text
warning: virtual workspace defaulting to `resolver = "1"` despite one or more
workspace members being on edition 2024 which implies `resolver = "3"`
```

Esse warning é o cargo te pedindo para declarar um termo do contrato explicitamente. Você já declarou.

### MSRV é um contrato

`rust-version` é o campo de manifest que ninguém ensina e que todo repo sério carrega. Ele declara a sua Minimum Supported Rust Version: o toolchain mais antigo com permissão de construir o seu crate. É cobrado, não decorativo. Aponte um cargo velho demais para um crate que declara `rust-version = "1.98.0"` e o build morre na hora com:

```text
error: rustc 1.93.1 is not supported by the following package:
  pulse-engine@0.1.0 requires rustc 1.98.0
```

Leia esse erro uma vez e você entende o campo: ele é um leão de chácara, checado antes de qualquer código compilar. E o `cargo add` respeita ele na outra direção, escolhendo versões de dependência cujo MSRV próprio cabe no seu em vez de te passar algo que o seu piso não consegue parsear.

O que nos traz de volta ao `resolver = "3"`. O resolvedor v3 é o default da edition 2024 (ele precisa de Rust 1.84 ou mais novo só para rodar), e o único trabalho dele é deixar a resolução de versões ciente do MSRV. Sob o v2, o cargo pega a versão semver-compatível mais nova de toda dependência e deixa um toolchain velho descobrir o problema em tempo de compilação. Sob o v3, o cargo checa o `rust-version` declarado de cada candidato contra o seu e dá fallback por cima de releases que exigem um compilador mais novo do que você suporta. Mecanicamente ele vira uma chave de config, `resolver.incompatible-rust-versions`, de `allow` para `fallback`.

Grafe essa chave com cuidado: ela é plural, `incompatible-rust-versions`. Eu sinalizo isso porque o próprio guia de editions imprime um typo no singular na página de resolver dele, enquanto a referência do cargo imprime a chave real, quatro vezes. Copie a grafia da página oficial errada e você entrega uma chave de config que o cargo ignora em silêncio. Duas fontes oficiais, uma delas errada: este é o hábito de verifique-suas-fontes do curso em miniatura, e ele vale para rust-lang.org exatamente tanto quanto para um blog qualquer. Quando dois docs discordam, a referência mais próxima da implementação ganha.

![O resolvedor v2 escolhe a dependência mais nova e quebra um toolchain velho enquanto o resolvedor v3 compara MSRVs e dá fallback para uma versão que cabe.](assets/v04-flowchart.webp)

Então que número você escreve? Não existe resposta certa, só um contrato que você escolhe de propósito, e o ecossistema te mostra os dois polos. O agave declara `rust-version = "1.98.0"`, que é exatamente o stable atual (o 1.98.0 foi entregue em 2026-08-20; os dois fatos sondados em 2026-09-02, e com um release stable a cada seis semanas o dígito vai se mover de novo logo). O carbon, um framework de biblioteca para construir indexadores, declara `rust-version = "1.82"`, dezesseis releases atrás. Os dois estão certos. Uma aplicação como um validador controla o próprio ambiente de build, então ela acompanha o mais novo e pega toda API nova da std no dia em que ela aterrissa. Uma biblioteca roda nos toolchains dos outros, então ela fica para trás, recebendo usuários que nunca conheceu ao custo de proibir a si mesma APIs mais novas. Declarar 1.82 recebe usuários mais velhos E amarra as suas mãos; declarar 1.98 solta as suas mãos E exclui gente. Apps acompanham, bibliotecas ficam para trás, e o único pecado de verdade é não escolher.

![Uma aplicação de validador fixa o piso de Rust dela no stable atual enquanto uma biblioteca de indexador fica dezesseis releases atrás, cada postura trocando liberdade por alcance.](assets/v05-comparison.webp)

Para o `pulse-rs` você vai declarar `rust-version = "1.85"` nos dois crates no lab. O raciocínio é a postura de biblioteca: nada no nosso código precisa de nada mais novo que o próprio piso da edition 2024, e o motor é uma biblioteca por construção, então ele fica para trás de propósito. Um footgun antes de você escrever: `rust-version` é um piso, não um pin. Ele impede um toolchain velho de construir você; ele não impede VOCÊ, num toolchain novo, de escrever um idioma que o seu piso declarado não consegue parsear. A cobrança honesta é um job de CI que constrói no próprio toolchain do MSRV. A gente não vai adicionar um hoje; bibliotecas sérias adicionam. Sem ele, o campo é uma promessa sem teste.

### Lendo pins como um dev que trabalha

Tudo até aqui foi escrever o seu próprio manifest. A habilidade de ciclo de vida de dev escondida aqui é ler os dos outros, porque toda linha de dependência num repo de verdade é uma decisão que alguém tomou, e os pins são onde as decisões aparecem. Três artefatos ao vivo, todos sondados a partir dos manifests dos repos deles em 2026-09-02. Para cada um, a pergunta de trabalho: o que este pin está te dizendo?

Primeiro. O manifest raiz do agave:

```toml
reqwest = { version = "0.12.28", default-features = false }
```

O reqwest mais recente do crates.io é o 0.13.4, e o 0.13.0 está fora desde o fim de 2025. O repo mantido mais ativamente do ecossistema Solana está um major inteiro atrás no cliente HTTP dele, e não é acidente, ninguém esquece uma dependência numa base de código tão auditada. Um major antigo fixado num repo vivo quer dizer que alguém avaliou o upgrade e disse ainda não: custo de migração, risco de comportamento, superfície de review, alguma coisa. O pin é uma decisão que você está lendo, não uma tarefa que ninguém fez.

Segundo, mesmo manifest, mais abaixo:

```toml
clap = { version = "2.33.1", default-features = false, features = ["suggestions"] }
```

O clap hoje é 4.6.6, e a linha 4.x está estável desde 2022. Este pin é um major de uma DÉCADA de idade, ainda sendo entregue em produção, ainda parseando os argumentos do software que roda uma rede monetária. Eu lembro da primeira vez que um pin assim me parou seco no repo de outra pessoa: o meu instinto disse mal mantido, e o meu instinto estava errado. A leitura correta é mais fria. O que quer que o clap 2 faça para aqueles binários, ele faz, e o custo de mexer numa coisa que funciona, vezes cada binário do workspace, perdeu a briga de custo-benefício todo ano por dez anos. A pior resposta a esta linha é o clássico primeiro PR rejeitado: um upgrade de passagem para o clap 4 de alguém que leu o número de versão mas não o repo. Repare também que este pin é o trade-off do workspace lá de cima em escala plena: porque o agave centraliza toda versão em `[workspace.dependencies]`, o clap de um crate é o clap de todo crate, então um upgrade não é uma migração, é todas elas de uma vez, e é precisamente por isso que o pin se sustenta.

Terceiro, do photon, o indexador da Helius para contas comprimidas:

```toml
sqlx = { version = "0.6.2", features = [
    "macros",
    "runtime-tokio-rustls",
    # ...more features elided
] }
# time pinned because of https://github.com/launchbadge/sqlx/issues/3189
```

O sqlx hoje é 0.9.0, três majors à frente. Mas olhe o que fica embaixo do bloco: um comentário linkando a issue upstream exata a que o pin remete. Este é o padrão-ouro, o pin que explica a si mesmo. Um contribuidor chegando neste manifest não precisa fazer engenharia reversa da intenção a partir do git blame; a razão está a um clique, e quando a issue upstream fechar, quem ver sabe precisamente o que retestar. Quando você fixar alguma coisa no `pulse-rs` por uma razão que não é óbvia, este comentário é o formato a copiar.

![Três barras medem quantos majors cada pin de produção fica atrás do release mais recente dele, de um para o reqwest a três para o sqlx.](assets/v06-chart.webp)

Se o padrão parece específico do Rust, não é. Você viu a metade TypeScript desta mesma stack fazer isso mais rápido e mais alto: o @solana/kit entregou dois majors em pouco mais de nove semanas, entre 2026-06-16 e 2026-08-21, logo atrás de um minor, e as lições do M3 deste curso te ensinaram a fixar aquilo em que as suas dependências de fato dão peer, em vez do que o npm chama de latest. Mesma regra, os dois toolchains: leia o que o seu ecossistema fixa antes de dar upgrade em qualquer coisa. É um hábito de sobrevivência, não pedantismo.

O entregável profissional de uma leitura de manifest é uma linha por pin: o que ele implica para um contribuidor, o que acompanhar se você abrir PR ali. Você vai escrever três dessas linhas no challenge, e o hábito volta com coisa de verdade em jogo no lab de auditoria de dependências do M9, onde a árvore que você lê é a sua.

**Vá mais fundo (os 20%).** esta lição ensinou os padrões de workspace, edition, MSRV e feature que você vai usar toda semana, mais a habilidade de leitura de pin. O que ela deliberadamente pulou é a referência completa dos campos do manifest, profiles e customização de build, e como a resolução funciona por dentro. Os capítulos canônicos, os dois sondados ao vivo hoje: a referência do cargo sobre o resolvedor em [https://doc.rust-lang.org/cargo/reference/resolver.html](https://doc.rust-lang.org/cargo/reference/resolver.html), e sobre rust-version em [https://doc.rust-lang.org/cargo/reference/rust-version.html](https://doc.rust-lang.org/cargo/reference/rust-version.html). Aquela página do resolver também é o lugar certo de onde copiar a chave `incompatible-rust-versions`. Nada no lab depende de nenhuma das duas páginas; deixe elas como bookmark para o dia em que uma resolução te surpreender.

## Lab: faça o hoist, quebre, declare, verifique

A divisão já aconteceu na abertura, então o lab começa de um `cargo check --workspace` verde e faz o workspace se pagar. Os passos 1 a 3 são guiados; o passo 4, a declaração de MSRV, é o que é seu; o passo 5 te passa o arquivo de teste de integração completo, porque o que se paga ali é ler a segunda linha de runner e saber por que o tier existe, não digitar oito linhas; o passo 6 prova a coisa toda para o CI. A rep sem guia em que esta lição está de fato apostando é o challenge, três vereditos de pin escritos a frio.

1. Commite a divisão como ela está, para todo diff seguinte ser legível:

   ```bash
   git add -A && git commit -m "split pulse-rs into engine + cli workspace"
   ```

2. Faça o hoist das dependências. Adicione a tabela `[workspace.dependencies]` da seção de teoria ao manifest raiz (serde com `derive`, serde_json, thiserror, anyhow e a entrada de path do `pulse-engine`), depois reescreva os dois blocos `[dependencies]` de membro como assinaturas: `serde = { workspace = true }` e companhia no motor, `pulse-engine = { workspace = true }` e `anyhow = { workspace = true }` na CLI. Rode `cargo check --workspace` depois de cada manifest que você tocar, não no fim; um erro de manifest encontrado na hora nomeia a própria causa. E o deslize clássico aqui falha alto e de forma prestativa: assine algo que você esqueceu de declarar na raiz e o cargo diz exatamente o que está faltando:

   ```text
   error inheriting `thiserror` from workspace root manifest's
   `workspace.dependencies.thiserror`

   Caused by:
     `dependency.thiserror` was not found in `workspace.dependencies`
   ```

   Segure o contraste: erros de manifest como este nomeiam a causa deles em texto plano, enquanto o erro de feature que você está prestes a conhecer no próximo passo faz qualquer coisa menos isso.

3. Agora quebre de propósito, porque a rep guiada aqui é ler a quebra. Na declaração da raiz, vire o serde para defaults-off:

   ```toml
   serde = { version = "1.0.229", default-features = false, features = ["derive"] }
   ```

   `cargo check --workspace` de novo e leia o que você recebe. Não um recado amigável sobre features. Isto (no cargo 1.98.1; toolchains mais velhos nomeiam itens auxiliares diferentes do mesmo módulo, `Content` em vez de `TaggedContentVisitor`, e o formato é idêntico):

   ```text
   error[E0433]: failed to resolve: cannot find `TaggedContentVisitor` in `de`
     --> crates/pulse-engine/src/config.rs
   note: found an item that was configured out
     --> .../serde-1.0.229/src/private/de.rs
   ```

   O erro aponta para dentro do fonte do próprio serde, na sua linha de derive, sobre um item que foi "configured out," e se repete para um punhado de nomes irmãos (`ContentDeserializer`, `Content`, `ContentVisitor`). Nada em lugar nenhum diz que você desligou o `std`. Este é o footgun da seção de teoria ao vivo na sua tela, e o debugger é:

   ```bash
   cargo tree -e features -p pulse-engine
   ```

   Na saída, ache o serde e leia quais arestas de feature existem. Com os defaults desligados você vai ver a aresta `derive` mas nenhuma aresta `default`, e essa ausência é o bug inteiro. Agora faça a escolha deliberada: o nosso motor faz o parse de arquivos com `String`s e `Vec`s em todo lugar, ele precisa de `std`, e o próprio agave mantém os defaults do serde ligados. Reverta a virada. Defaults-off é uma ferramenta para crates pesados com maquinaria opcional que você não quer; aplicado ao serde aqui é fazer cargo cult da higiene sem o julgamento. Saber quando NÃO aplicar o padrão é o padrão.

![Cada linha do manifest de workspace terminado carrega uma nota de margem explicando a promessa que ela faz a quem constrói e ao resolvedor.](assets/v07-annotated-code.webp)

4. Declare o contrato de MSRV. Este é conduzido pelo aprendiz: adicione `rust-version = "1.85"` às tabelas `[package]` dos dois crates, e consiga dizer por que 1.85 e não 1.98 numa frase antes de seguir em frente (a divisão app-versus-biblioteca da seção de teoria é a frase). Prove a si mesmo que o campo é cobrado lendo, não rodando: o texto de erro na seção de MSRV acima é o que um toolchain 1.84 imprimiria para os seus usuários. Depois `cargo test --workspace`. Os dois crates constroem, os testes de m04 do motor passam a partir da raiz, o mesmo verde de antes da divisão, formato novo.

5. Colete o tier de integração. O m04-l3 nomeou o segundo tier de testes do Rust, testes de integração num diretório `tests/` de nível superior, e te mandou estacionar isso até a divisão do workspace acontecer. Acabou de acontecer, então colete. Crie `crates/pulse-engine/tests/engine_contract.rs`:

   ```rust
   // The integration tier: this file compiles as its own tiny crate, linked
   // against pulse-engine, so it sees exactly what any outside consumer sees.
   // Private items are unreachable from here; pub ones are, either by the
   // short name lib.rs re-exported or by their full module path.
   use pulse_engine::{FixtureSource, ProbeState, drive};

   #[test]
   fn the_fixture_story_survives_the_public_surface() {
       // m04-l3's six-fixture walk: two clean probes, three failures past the
       // budget, one recovery. Ends Up, exactly as you walked it by hand.
       let mut source = FixtureSource::new(vec![212, 487, 1600, 1700, 1800, 90]);
       assert_eq!(drive(&mut source, 1500), ProbeState::Up);
   }
   ```

   Rode `cargo test --workspace` e leia a saída com olhos novos: abaixo do binário de teste unitário você ganha uma segunda linha de runner, `Running tests/engine_contract.rs`, porque o cargo compilou aquele arquivo como o próprio crate dele e linkou ele contra a sua biblioteca. Essa é a costura entre os dois tiers, e ela é visibilidade, não geografia: um bloco `#[cfg(test)] mod tests` mora dentro do módulo e enxerga itens privados, enquanto um arquivo em `tests/` consome o crate exatamente como o `pulse-cli` consome, pela superfície pública que você curou dez minutos atrás. O que deixa este teste único fazer um trabalho que os cinco testes unitários não conseguem: tire um nome da lista de re-export do `lib.rs` e a suíte unitária continua verde enquanto este arquivo para de compilar, o primeiro consumidor a notar que a porta da frente mudou. Um teste segura o tier aberto hoje; a suíte de transição do m04-l3 fica onde está, dentro do módulo ao lado do match que ela fixa, porque cutucar os casos de borda do `next_state` é trabalho de tier unitário e a colocação do arquivo deveria dizer isso.

6. Dê push, e veja a trava de CI do m04-l3 rodar sem mudança. Nenhuma edição de workflow: os comandos de cargo da trava rodam contra o que quer que o manifest raiz descreva, e o manifest raiz agora descreve um workspace, então `cargo test` cobre os dois membros e os dois tiers, incluindo o novo crate `tests/`. CI agnóstico de layout é um dos ganhos silenciosos de o cargo ser uma ferramenta em vez de cinco. Quando a execução estiver verde, a barra de aceitação da metade de build desta lição está cumprida: deps declaradas uma vez na raiz, os dois crates na edition 2024 com `rust-version` declarado, `cargo test --workspace` verde localmente e no CI.

## Challenge

A rep sem guia, no papel, sem compilador para se apoiar. Três trechos de manifest da seção de teoria: o reqwest 0.12.28 do agave, o clap 2.33.1 do agave, o sqlx 0.6.2 do photon com o comentário de link para a issue dele. Para cada um, escreva UMA linha declarando o que o pin diz a um contribuidor que está prestes a abrir um PR contra aquele repo: o que ele implica sobre a base de código, e o que você acompanharia ou evitaria tocar. Sem apoio, sem hints, e resista à vontade de espiar de volta as minhas leituras; o treino é produzir o veredito você mesmo, a frio. Aceitação: três linhas escritas, cada uma nomeando uma implicação concreta (quais idiomas de API o seu patch precisa usar, o que você não deve relitigar dentro de um PR não relacionado, ou o que a issue linkada quer dizer para retestar). Não guarde as três linhas num arquivo de rascunho. Commite elas, na raiz do repo da estação, como `docs/pin-reads.md`, um cabeçalho por pin e o seu veredito embaixo, datado. Duas razões, e a segunda é a de verdade. Um veredito que nada consegue achar de novo é um veredito que você vai revisar em silêncio; um commitado é um em que você pode estar errado na frente dos outros. E o lab de auditoria de dependências do m09-l1, que pede exatamente esta habilidade contra a sua própria árvore e avalia ela, abre este arquivo e faz você ler os seus vereditos frios contra uma checklist que você não vai ter até lá. Escrito hoje, avaliado daqui a quatro lições.

## Checkpoint

Os músculos novos, concretamente: dividir um projeto Rust no formato de workspace que o ecossistema de fato usa, com dependências declaradas uma vez e membros assinando; dizer o que a edition 2024 mudou e o que ela não mudou (async closures são uma feature de linguagem do 1.85, todas as editions); declarar um MSRV como um contrato escolhido e explicar o contrato de quem ele espelha, o do agave ou o do carbon; e ler o pin de versão de um estranho como informação em vez de ruído.

A recuperação de 30 segundos antes de você fechar a aba: o que o resolvedor v3 faz que o v2 não fazia, numa frase? (Ele considera o rust-version declarado de cada dependência na hora de escolher versões, dando fallback por cima de releases que o seu MSRV não consegue construir, em vez de sempre pegar a mais nova compatível.) Se essa frase te custou mais de uma tentativa, releia o fluxograma da seção de MSRV, é a peça desta lição que aparece em entrevistas.

Um pedido enquanto está fresco: a quebra de defaults-off no passo 3 do lab é deliberadamente desorientadora, e eu quero saber o quanto. Note se o erro "configured out" fez sentido antes ou só depois do `cargo tree -e features`, e me diga no feedback. Se a maioria de vocês só entendeu depois, a próxima revisão ensina a árvore primeiro e quebra depois.

O workspace agora tem o formato do ecossistema: um crate de motor puro que qualquer coisa consegue consumir, uma CLI fina em cima dele que não faz nada além de consumir. O que quer dizer que a CLI hoje é uma casca oca com um cérebro emprestado, e na próxima lição ela ganha o nome: o clap dá a ela uma interface de linha de comando de verdade, o reqwest blocking finalmente dá à estação um braço de sonda DE VERDADE em vez de latências de fixture, e o CI começa a passar a estranhos um binário que eles conseguem baixar e rodar. O Cargo.toml era a negociação; a próxima lição entrega algo que vale a pena negociar.
