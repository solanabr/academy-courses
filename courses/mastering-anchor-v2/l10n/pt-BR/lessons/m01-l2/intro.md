# Duas linhas paralelas, e a instalação que briga com você

Na lição passada você conferiu duas transações que outra pessoa já havia colocado na devnet, uma contra o gêmeo v1 e outra contra o gêmeo v2, e leu o delta de compute units direto dos logs. Você observou a diferença. Você não fez build nem deploy de nada seu.

Isso muda agora. Mas antes de conquistar o seu primeiro deploy, abra um terminal e rode isto:

```bash
anchor --version
which anchor
```

O que aparecer de volta é quase certamente o Anchor **1.1.2**, a linha estável atual, instalado pelo `avm` e morando no seu PATH. Esse binário é a ferramenta errada para este curso, e ele não vai te avisar. Ele vai fazer build de um lab V2 contra a semântica do V1 numa boa e te entregar erros que não fazem sentido. Então a primeira coisa que você aprende sobre o Anchor V2 não é uma macro. É que a versão que você já tem é uma armadilha, e a versão que você quer briga de volta quando você tenta instalar.

É essa a lição. Não um desvio em volta do atrito, o atrito em si. Instalar um release candidate (candidata a versão final) a partir de um branch, quando o instalador oficial não tem binário nenhum para te entregar, é a sensação de verdade de viver na fronteira. Quero que você sinta isso uma vez, comigo narrando cada parede para você saber que é a ferramenta e não você.

## Resumo

O Anchor é entregue em duas linhas paralelas agora mesmo: a estável **1.1.2**, e a **2.0.0-rc.1** montada num branch não mergeado chamado `anchor-next`. Este curso vive na segunda linha. Você instala esse RC num toolchain isolado a partir do canal git documentado dele, aprende por que o `avm install 2.0.0-rc.1` não consegue baixá-lo para você, registra tudo isso num arquivo central de pins com datas de freshness, e então gera o scaffold do greeter (R0), faz o build e o deploy dele na devnet como o seu primeiro deploy independente. O R0 é o programa de rascunho: ele fica abaixo do primeiro degrau da escada Quarters, e você segue estendendo ele pelo resto deste módulo antes dos degraus de verdade começarem.

O recuo da ajuda aqui é deliberado e raso. Esta é uma lição de toolchain, então a instalação e o scaffold são **totalmente guiados**: eu mostro cada comando, você segue exatamente, nada de solo ainda. O único passo que é só seu é o deploy final. Você roda o `anchor deploy` contra a devnet, você lê de volta um program id, e você cola isso no arquivo de pins. É essa a graduação inteira.

Uma nota honesta logo de saída. Todo número de versão nesta página é um instantâneo com uma data grudada, e o RC vai se mover. Isso não é desleixo, é o custo de estar semanas na frente. A disciplina de re-verificar que você constrói aqui é a habilidade de verdade.

## Por que o RC mora na casa dele

Comece pela coisa que você já consegue ver. Existem duas linhas do Anchor, e elas não são uma escadinha de beta para estável. Elas são paralelas.

A linha estável é a **1.1.2**. É o que o `avm` instala, o que o crates.io serve como `anchor-lang`, e contra o que a maior parte do ecossistema faz build hoje. A linha da fronteira é a **2.0.0-rc.1**. Ela não mora num release publicado e abençoado como a 1.1.2 mora. Ela mora num branch de desenvolvimento chamado `anchor-next`, e o único jeito documentado de tirar dele uma CLI que funciona é fazer você mesmo o build desse branch com o cargo.

![Um lado a lado do Anchor estável 1.1.2 (avm/crates, já instalado) contra a fronteira 2.0.0-rc.1 (construída a partir do branch git anchor-next, e rotulada tanto como "rc" quanto como "alpha").](assets/v01-comparison.png)

Aqui está o porquê debaixo do quê, porque vale derivar isso uma vez. Um release candidate num branch não mergeado não é uma promessa, é um trabalho em andamento que por acaso tem um número de versão. Se você deixar ele sobrescrever a 1.1.2 do seu PATH, agora você tem exatamente um Anchor, e é o que está fervendo. No momento em que o `anchor-next` quebrar (e RCs quebram, é o trabalho deles), todo projeto na sua máquina quebra junto. Isolamento não é cautela por si mesma. É manter uma ferramenta estável para o seu trabalho estável e uma ferramenta de fronteira para o seu trabalho de fronteira, lado a lado, cada uma honesta sobre o que é.

A boa notícia é que "isolado" aqui não quer dizer um contêiner nem uma máquina virtual. É mais simples e mais físico que isso. O `cargo install` que você está a ponto de rodar deixa um único binário em `~/.cargo/bin/anchor`. O `avm`, enquanto isso, gerencia a sua 1.1.2 por um shim próprio. Os dois querem responder quando você digita `anchor`, e quem ganha é decidido por nada mais exótico que a ordem do PATH. É esse o modelo de isolamento inteiro: dois binários no disco, um nome, e o seu shell pegando a primeira correspondência. É também por isso que a confusão mais comum desta instalação toda é um build que se comporta como V1 quando você tinha certeza de que instalou o V2. O RC está lá. Seu PATH só te entregou o outro. Você vai confirmar qual binário responde no lab, e vale internalizar agora que, na fronteira, `which anchor` é um comando de depuração, não uma formalidade.

### Por que o `avm install` não consegue fazer isso para você

Seu instinto, corretamente, é pegar o `avm`. O **Anchor Version Manager** é a ferramenta que instala e troca entre versões da CLI do Anchor, do jeito que o `rustup` faz para o Rust. Você quase certamente usou ele para pegar a sua 1.1.2. Se você não o instalou, o caminho documentado é um cargo install a partir do repositório do Anchor:

```bash
# avm - the Anchor Version Manager. The install docs still publish the
# solana-foundation URL; it 301-redirects to otter-sec/anchor, which is where
# the repo actually lives now (custody went coral-xyz -> solana-foundation ->
# otter-sec during 2026). Either URL resolves. Re-check before you run this.
# (freshness 2026-08-22)
cargo install --git https://github.com/otter-sec/anchor avm --locked --force
avm install 1.1.2
avm use 1.1.2
```

Então você tenta a coisa óbvia:

```bash
avm install 2.0.0-rc.1
```

E ele recusa. Leia a falha com cuidado, porque o motivo é mais chato e mais útil do que parece:

```
Failed to download the binary for version `2.0.0-rc.1` (status code: 404 Not Found)
```

Isso não é uma rejeição de política. O `avm` faz o parse de `2.0.0-rc.1` perfeitamente bem; ele tem um canal de prerelease inteiro (`avm install latest-pre-release`, `avm list --pre-release`). A falha é mecânica. Para qualquer versão em 0.31 ou acima, o `avm install` não faz build de nada: ele baixa um **binário pré-compilado** dos assets da Release daquela tag no GitHub, em `releases/download/v2.0.0-rc.1/anchor-2.0.0-rc.1-<your-target>`. Esses assets só existem se alguém cortou um **objeto Release** para a tag, e ninguém cortou. Vá procurar `releases/tags/v2.0.0-rc.1` e você leva um 404 também. Sem Release, sem asset, sem download.

Note o que *não* está acontecendo aqui, porque é uma história que você vai ouvir contada errado. O `avm` não verifica nenhuma atestação criptográfica do release, e nunca verificou: não existe código de atestação dentro dele. A parede é um arquivo que falta, não uma checagem de assinatura que falhou. Vale saber isso com precisão, porque uma checagem de segurança que você não consegue satisfazer e um artefato de build que ninguém subiu pedem respostas completamente diferentes.

![O avm install baixa um binário pré-compilado dos assets de release da tag v2, leva um 404 porque nenhuma Release foi cortada, e aborta, então o cargo git install documentado assume.](assets/v02-flowchart.png)

Para completar: o `avm install` carrega sim uma flag `--from-source`, que pula o download e passa o trabalho para `cargo install --git https://github.com/otter-sec/anchor --tag v2.0.0-rc.1` — o mesmo build que você está a ponto de rodar na mão, com o canal escolhido para você.

Existe uma armadilha de nome que vale sinalizar antes que ela te morda. Se você for procurar no crates.io pelo `avm` em si, você vai achar um, e ele **não é esta ferramenta**. O crate `avm` no crates.io é um pacote sem relação nenhuma, de 2016 (schultyy/avm). O gerenciador de versões do Anchor não é distribuído com esse nome de crate, ele instala a partir do repo do Anchor. Instale o `avm` errado e você vai passar uma hora confuso sobre por que nenhum dos comandos existe.

### O canal de instalação, e a publicação que é uma pista falsa

O canal documentado é um build via git. Aqui está o comando exato, e você vai rodar ele de verdade no lab:

```bash
cargo install --git https://github.com/otter-sec/anchor.git \
  --branch anchor-next anchor-cli --locked --force
```

Leia da esquerda para a direita, porque cada flag é estrutural. O `--git` mais a URL do fork diz "faça o build a partir do código neste repositório, não do crates.io." O `--branch anchor-next` fixa a fonte no branch da fronteira especificamente. O `anchor-cli` é o crate dentro desse repo que você de fato quer como binário. O `--locked` diz "respeite o Cargo.lock commitado, não resolva dependências mais novas em silêncio," o que num RC é a diferença entre um build reproduzível e um mistério. O `--force` sobrescreve qualquer `anchor-cli` que o cargo já tenha colocado em `~/.cargo/bin`.

O `--branch anchor-next` é a flag que esta lição escolhe de propósito e que toda lição posterior sobrescreve, então resolva o delta aqui. A ponta de um branch se move; uma tag não. Em 2026-08-22 a ponta do `anchor-next` está à frente da `v2.0.0-rc.1` e já moveu os dois crates que você está a ponto de fixar na mão: o `anchor-lang` da ponta pede `wincode 0.6` e `solana-address 2.7.0`, enquanto a tag — e o crate `2.0.0-rc.1` publicado a partir dela — pede `wincode 0.5` e a linha `solana-address 2.x` abaixo de 2.7. A linha de falha da issue #4937 passa exatamente aí, e é por isso que os pins no lab abaixo carregam uma versão *e* uma data.

Você fica em cima do branch hoje porque o canal *é* o assunto desta lição: é o que os docs do V2 publicam, e um canal que se move não é algo sobre o que você consegue raciocinar de fora. Toda lição depois desta reimprime o comando com **`--tag v2.0.0-rc.1`** no lugar de `--branch anchor-next`, e a m08-l2, cujo trabalho inteiro é um build reproduzível byte a byte, aperta uma vez mais para `--rev e4878b6d`, o commit para onde essa tag aponta. Três grafias, duas coisas distintas: fique em cima do branch aqui para ver o canal, em cima da tag em todo lugar depois, para que os pins que você escreve continuem significando o que significavam quando você os escreveu.

Agora, uma coisa que vai te tentar. O RC **foi** publicado no crates.io em 2026-08-12 como `2.0.0-rc.1`. Então você poderia razoavelmente pensar que consegue pular a dança do git e só rodar `cargo install anchor-cli --version 2.0.0-rc.1`. Não confie nisso como o seu caminho de instalação. A publicação no crates.io existe, mas é não documentada e não testada para instalação de CLI. O canal sancionado e reproduzível para o **binário** é o build via git. Quando os docs e o registro discordam sobre o que é seguro instalar, os docs atrasam em relação à realidade constantemente, mas um crate publicado-mas-não-testado é uma aposta pior que o build documentado. Confiar no registro em cima do processo documentado é a cilada número três, retomando a contagem da lição passada.

Leia isso de forma tão estreita quanto está escrito, porque o lab abaixo faz o *oposto* para a biblioteca. O `anchor-cli` é um binário que o projeto constrói e testa pelo canal git dele; o `anchor-lang` é uma biblioteca que o projeto publica no crates.io de propósito, e o próprio scaffold dele te manda depender da versão publicada assim que existir uma. Um artefato por canal, cada um no canal que os mantenedores dele de fato sustentam.

### A tensão rc-versus-alpha, ensinada em voz alta

Aqui está uma coisinha que te diz muito sobre onde este release está de verdade. O crates.io marca ele como `rc`. A própria página de benchmarks do projeto chama o mesmo build de `alpha`. Mesmo código, dois rótulos de maturidade, vindos do próprio projeto.

Eu não vou escolher um para você e fingir que o conflito não existe. Isso lavaria exatamente o sinal de que você precisa. Um `rc` deveria significar "a gente acha que isso está quase entregável." Um `alpha` significa "isso é cedo, espere quebra." Quando o projeto usa as duas palavras para um build, a leitura honesta é: está em algum lugar no meio, e você deveria fixar a versão exata que instalou e re-verificar em um cronograma em vez de confiar no rótulo. O conflito de rótulos não é ruído para resolver, é o sinal de maturidade, e a resposta correta a ele é um arquivo de pins, que é justo o que a gente está a ponto de construir.

A primeira história de guerra de verdade do RC deixa o ponto concreto. A issue #4937, aberta em 2026-08-16 e fechada em 2026-08-20, era um descasamento de dependência: o `anchor-lang` fixava o `wincode` em 0.5 enquanto o `solana-address` 2.7.0 subia a própria exigência de `wincode` para 0.6, e o descasamento de trait bound quebrou `#[account(borsh)]`. (A versão que se moveu é a do `wincode`; o `solana-address` só entregou uma linha 2.x na vida, como o bloco de pins no fim desta lição deixa claro.) Isso é disciplina de fixar dependências da era RC pega ao vivo. É exatamente por isso que o `--locked` está no seu comando de instalação e exatamente por isso que cada pin que você escreve ganha uma data do lado.

### Os incrementos datados do 1.0, para o V2 ter um "antes"

Mais um pedaço de contexto, e este é para todo leitor, não importa qual Anchor você já tocou antes. Para entender por que o V2 mudou coisas, você precisa do mapa do que o Anchor **1.0** já mudou. Estes são os incrementos que chegaram com o Anchor 1.0.0 em **2026-04-02**, e lições posteriores vão voltar a esta lista toda vez que a gente disser "o V2 manteve isso" ou "o V2 foi além."

![Uma linha do tempo marcando o Anchor 1.0.0 em 2026-04-02 com os cinco incrementos dele (a renomeação do pacote, o CpiContext recebendo um Pubkey, o transfer_checked como default, o LiteSVM, o Surfpool) e a publicação da 2.0.0-rc.1 no crates.io em 2026-08-12.](assets/v03-timeline.png)

Percorra todos uma vez, devagar, porque cada um é um retorno esperando para acontecer.

A **renomeação do pacote** é a primeira. O cliente TypeScript que morava em `@coral-xyz/anchor` agora é publicado como `@anchor-lang/core`. Isso não é uma mudança cosmética. Cada linha de import em cada cliente que você escrever contra um programa Anchor aponta para o nome novo, e no dia em que você ligar um cliente na m08, você vai pesar o `@anchor-lang/core` de propósito — e deixar ele de lado de propósito, porque ele ainda anda em cima do web3.js v1, então em vez dele você gera um cliente kit-native — enquanto a maioria dos tutoriais online ainda mostra o nome antigo.

Segundo, **o `CpiContext` mudou de forma**. Quando um programa chama outro (uma invocação entre programas), você monta um `CpiContext`, e no 1.0 o construtor dele recebe o programa alvo como um `Pubkey` (via `program.key()`), não como um `AccountInfo` do jeito que o Anchor mais antigo fazia. Esse tem uma armadilha embutida: a página de documentação do anchor-lang.com sobre CPIs ainda mostra a forma antiga com `AccountInfo`. Quando a m04 te colocar em chamadas entre programas de verdade, o compilador é a autoridade, não aquela página. Passe o tipo errado e ele vai te dizer `expected Pubkey, found AccountInfo`.

Terceiro, **o `transfer_checked` é a movimentação de token default**. O `transfer` simples em que o código SPL token mais antigo se apoiava está deprecado, e a CPI que você pega agora é o `transfer_checked`, que recebe também o mint e os decimais dele para o runtime pegar um descasamento de decimais antes de mover valor. Quando a m05 ligar os fluxos de token, você vai digitar `transfer_checked` sem pensar, e o motivo de não ser o `transfer` simples começa em 2026-04-02.

Quarto, **o LiteSVM é o template de teste default**. Os testes gerados pelo Anchor não assumem mais que você sobe um validador inteiro para rodar uma única asserção. O LiteSVM roda o seu programa in-process, que é por que a linha de testes que abre na m02 é rápida o bastante para rodar a cada save.

Quinto, **o Surfpool é o validador local default**. Quando você roda `anchor test` ou `anchor localnet`, o validador embaixo é o Surfpool, não o antigo `solana-test-validator`. Você não roda um teste hoje, mas o scaffold já escreveu um para você em `programs/greeter/tests/test_initialize.rs`, e você roda ele na próxima lição. Como o template default é o LiteSVM, esse teste roda in-process e nunca precisa do Surfpool; no momento em que você pegar um template que conversa mesmo com um validador, esta é a máquina do outro lado.

Essa renomeação tem uma sobrevida marcante. Uns oito meses depois de `@coral-xyz/anchor` virar `@anchor-lang/core`, o pacote antigo ainda é baixado mais que o novo numa proporção de cerca de **40 para 1** (601,707 contra 14,745 numa única semana). "Atual" e "comumente usado" divergiram forte. Essa diferença é o seu lembrete de que o ecossistema se move mais devagar que os números de versão, e de que quando você escrever um cliente mais para frente neste curso, você escolhe o nome que está correto, não o nome que é popular.

Nenhuma dessas cinco mudanças do 1.0 é coisa que você toca no lab abaixo. O programa que o scaffold gera para você abre uma conta e zera um contador, nada mais. Mas o ponto de narrar isso agora é que, quando a m05 te mostrar `transfer_checked` e perguntar "por que o `transfer` simples sumiu," a resposta começa aqui, em 2026-04-02, e não no V2.

### As quatro paredes, e como não dar de cara nelas

Antes de abrir um terminal, mantenha os modos de falha à vista. A fronteira tem exatamente quatro paredes que pegam quase todo mundo, e cada uma delas é um caso de uma ferramenta sendo honesta enquanto você esperava uma ferramenta diferente. Nenhuma delas é o seu código.

![Uma tabela de runbook emparelhando cada uma das quatro paredes de instalação da fronteira, mais a armadilha de nome do avm, com a única linha corretiva que resolve cada uma.](assets/v04-table.png)

Mantenha essa tabela por perto durante o lab. Quando algo quebrar, e na fronteira algo normalmente quebra, case o sintoma com uma linha antes de assumir que você fez algo errado.

## Lab: instale o RC, gere o scaffold do greeter, faça o deploy do R0

Você vai construir um arquivo central de pins, instalar o RC isolado, gerar o scaffold de um greeter, fazer o build dele e o deploy na devnet. Os passos 1 a 6 são totalmente guiados. O passo 7, o deploy, é seu.

**1. Confirme as duas ferramentas embaixo do Anchor.** O Anchor fica em cima do Rust e da CLI do Solana (Agave), então fixe essas primeiro. Se você não tem Rust, instale e fixe o MSRV que o RC exige, que é **1.89.0** (freshness 2026-08-22):

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install 1.89.0
rustc +1.89.0 --version   # expect: rustc 1.89.0
```

Instale, não faça dele o default da sua máquina. O mesmo argumento de isolamento da seção de teoria se aplica uma camada abaixo: fixar o seu Rust global no MSRV do RC arrasta todo outro projeto que você tem para aquele toolchain. Em vez disso você dá escopo a ele. O scaffold que você gera no passo 4 escreve um `rust-toolchain.toml` nomeando 1.89.0, que o rustup honra automaticamente dentro daquele diretório; se você algum dia precisar forçar na mão, `rustup override set 1.89.0` a partir da raiz do workspace faz o mesmo trabalho só para aquele diretório.

O **MSRV**, versão mínima suportada do Rust, é o Rust mais antigo em que o crate promete compilar. Para um RC isso não é uma sugestão. Faça build com algo mais antigo e você ganha erros que parecem dizer que o seu código está errado quando é o toolchain que está.

Depois a CLI do Solana, que você instala pelo instalador da Anza:

```bash
# Agave (Solana) CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
solana --version
```

Uma palavra precisa sobre versões aqui, porque isso importa para o curso inteiro. A imagem de integração contínua deste curso fixa a CLI do Solana em **3.1.10**. Esse número é um **pin de toolchain de CI local**, a CLI exata contra a qual o verificador do lab roda, e nada mais. Não é uma afirmação sobre o que é "o Solana atual". A linha estável atual do Agave é a **v4.2.1** no momento em que escrevo isto (agosto de 2026; re-verifique, isso se move). Se você algum dia vir 3.1.10 e pensar "então o Solana está no 3.x," essa é a cilada número quatro. É um build fixado para labs reproduzíveis, ponto final.

**2. Crie o arquivo central de pins.** Isto é infraestrutura do curso, não algo descartável. Crie um arquivo na raiz de onde você vai guardar o trabalho deste curso, chamado `PINS.md`, e semeie ele:

```markdown
# Course toolchain pins (re-verify on a schedule; the RC moves)

| pin                         | value                        | channel                          | verified   |
|-----------------------------|------------------------------|----------------------------------|------------|
| anchor-cli (this lesson)    | 2.0.0-rc.1                   | git anchor-next (otter-sec fork) | 2026-08-22 |
| anchor-cli (rest of course) | 2.0.0-rc.1                   | git tag v2.0.0-rc.1 = e4878b6d   | 2026-08-22 |
| anchor-lang (library)       | 2.0.0-rc.1                   | crates.io (immutable)            | 2026-08-22 |
| wincode                     | 0.5 (features = ["derive"])  | crates.io                        | 2026-08-22 |
| solana-address              | =2.6.0                       | crates.io                        | 2026-08-22 |
| Rust (MSRV)                 | 1.89.0                       | rustup                           | 2026-08-22 |
| macOS build workaround      | CARGO_PROFILE_RELEASE_LTO=off| env var (release profile)        | 2026-08-22 |
| Solana CLI (CI pin)         | 3.1.10                       | agave-install (LOCAL-CI ONLY)    | 2026-08-22 |
| R0 greeter program id       | <fill after deploy>          | devnet                           | <fill>     |

Note: 3.1.10 is the local-CI pin, NOT "current Solana" (current stable Agave: v4.2.1, Aug 2026).
Note: the two anchor-cli rows report the SAME version string. Only the ref distinguishes them.
Note: solana-address is =2.6.0 for this greeter, a workspace of one. From m02-l1 the arcade
programs share a workspace, and every member of it reads ">=2.6.1, <2.7" instead. Same ceiling.
Label tension: crates.io says "rc", the benchmarks page says "alpha". Pinned + re-verified on purpose.
```

Uma **freshness note** é só aquela coluna de data `verified`. Um pin sem data é uma mentira esperando para acontecer, porque a coisa para onde ele aponta pode se mover no dia depois de você escrever. Na fronteira a data é metade do pin.

**3. Instale o RC isolado.** Este é o comando da seção de teoria, rodado de verdade. No macOS, prefixe ele com a solução de contorno de LTO (vou explicar a solução de contorno logo depois):

```bash
# macOS: the RC build dies during link-time optimization without this
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --branch anchor-next anchor-cli --locked --force
```

No Linux o prefixo `CARGO_PROFILE_RELEASE_LTO=off` é inofensivo, então deixar ele aí mantém um comando que funciona em todo lugar. No macOS ele é obrigatório: sem ele o build do RC morre com confiabilidade durante o **LTO** (link-time optimization, o passe final de otimização entre crates), e a falha parece um crash do linker, não um problema do Anchor. Definir a variável de ambiente do profile de release do cargo desliga esse passe e o build completa. Essa única linha pertence ao seu `PINS.md`, que é exatamente por que ela já está na tabela acima. Note o nome: é `CARGO_PROFILE_RELEASE_LTO`, uma variável padrão de profile do cargo, não alguma invenção `ANCHOR_LTO`.

![O comando de instalação do RC dividido nas partes dele, com cada flag explicada: a variável de ambiente de LTO, --git e --branch anchor-next, --locked, e --force.](assets/v05-annotated-code.png)

Quando terminar, verifique que você pegou o RC e não o seu binário antigo:

```bash
anchor --version   # expect: anchor-cli 2.0.0-rc.1
```

Se isso ainda imprimir 1.1.2, o seu shell resolveu o binário antigo primeiro. Confira o `which anchor` e garanta que `~/.cargo/bin` está no começo do seu PATH. Isto é o isolamento funcionando: o cargo colocou o RC em `~/.cargo/bin/anchor`, e o shim do `avm`, se ganhar a corrida do PATH, vai continuar te servindo 1.1.2. O que quer que imprima, registre o verdadeiro no `PINS.md`.

**4. Gere o scaffold do greeter.** Agora faça o seu primeiro programa V2. O `anchor init` gera um workspace completo e pronto para build:

```bash
anchor init greeter
cd greeter
```

Esse único comando escreve um projeto inteiro. Aqui está o que chega, para a árvore não ser uma caixa preta:

![A árvore do workspace greeter gerado, com programs/greeter/src/lib.rs destacado como o programa de verdade, um teste Rust LiteSVM gerado ao lado dele, e app/ e migrations/ marcados como scaffold ainda não usado.](assets/v06-diagram.png)

Abra `programs/greeter/src/lib.rs`. Aqui está o que o template do V2 escreve de fato, literalmente, tirando o program id gerado para você:

```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod greeter {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        ctx.accounts.counter.count = 0;
        ctx.accounts.counter.authority = *ctx.accounts.payer.address();
        msg!("Counter initialized");
        Ok(())
    }
}

pub mod state {
    use super::*;

    #[account]
    pub struct Counter {
        pub count: u64,
        pub authority: Address,
    }
}

use state::Counter;

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer)]
    pub counter: Account<Counter>,
    pub system_program: Program<System>,
}
```

Isto não é o "hello world" vazio que os templates 0.x escreviam. O V2 gera pelo scaffold um contador pequeno: uma instrução que cria uma conta e a zera. O `declare_id!` declara o endereço on-chain do programa. O `#[program]` marca o módulo de handlers de instrução. O `initialize` abre um `Counter`, coloca a contagem dele em zero, e estampa o payer como a autoridade dele. É esse o R0: não porque ele faz algo interessante, mas porque é a menor coisa completa que o seu toolchain consegue fazer build, deploy e comprovar.

Quatro detalhes aí dentro vão parecer errados se você carrega memória muscular de 0.x ou 1.0, e cada um é uma mudança real do V2: o handler recebe `&mut Context<T>` em vez de um context por valor; a struct de accounts e os wrappers dela não carregam lifetime `<'info>`; o tipo de endereço é `Address`, não `Pubkey`, e você lê ele com `.address()`; e o `init` não nomeia nenhum `space`, porque o V2 dimensiona a conta a partir do tipo dela. É uma caixa preta de propósito hoje: na próxima lição você abre essas macros e lê exatamente o que elas geram.

Agora arrume como o crate do programa pega o RC. Abra `programs/greeter/Cargo.toml`. O template **não** fixa uma versão do crates.io; ele aponta para o mesmo branch de onde você instalou a CLI, e deixa uma nota para si mesmo sobre isso (freshness 2026-08-22):

```toml
[dependencies]
# Once anchor-lang is published to crates.io, swap to: anchor-lang = "2.0.0-rc.1"
anchor-lang = { git = "https://github.com/otter-sec/anchor.git", branch = "anchor-next" }
solana-program-log = { version = "1.1", features = ["macro"] }
```

Esse comentário gerado é uma pequena cápsula do tempo que vale ler, e também é a sua instrução. O template diz "once anchor-lang is published to crates.io," e ele *foi*, em 2026-08-12. Faça exatamente o que o comentário diz. Mude essa linha para:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io; a published version is immutable
solana-program-log = { version = "1.1", features = ["macro"] }
```

Duas linhas que o template **ainda não** escreve, e sem as quais nada neste curso compila. Adicione elas na mesma tabela `[dependencies]` antes do seu primeiro build:

```toml
# #[program] expands absolute ::wincode:: paths, so the serializer must be a direct dep
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

Essas três linhas são o conjunto de pins que todo crate de programa deste curso carrega, e são o motivo de a linha do `anchor-lang` ter de sair do branch. O greeter é um workspace de um só, então o `=2.6.0` exato enuncia bem a restrição aqui; da m02-l1 em diante, onde os programas dividem um workspace e o cargo tem de resolver um só `solana-address` para todos eles, essa linha é escrita como a faixa `">=2.6.1, <2.7"` em vez disso. Mesmo teto, enunciado de um jeito que mais de um crate consiga concordar com ele — a m02-l1 faz o argumento. Tente do outro jeito e o cargo nem vai chegar a compilar:

```
error: failed to select a version for `solana-address`.
    ... required by package `anchor-lang v2.0.0-rc.1 (https://github.com/otter-sec/anchor.git?branch=anchor-next#a6510ad7)`
versions that meet the requirements `^2.7.0` are: 2.7.0
all possible versions conflict with previously selected packages.
  previously selected package `solana-address v2.6.0`
```

Essa é a classe de bug da #4937 de novo, ao vivo num resolve novo no momento em que escrevo. O `anchor-lang` da ponta do branch agora exige `solana-address 2.7.0`, que puxa `wincode 0.6`; o crate `2.0.0-rc.1` no registro — construído a partir da tag `v2.0.0-rc.1` — não exige nenhum dos dois, então `wincode 0.5` e `solana-address 2.6.0` se sustentam. Dois majors de wincode num grafo é uma parede de erros `SchemaRead`/`SchemaWrite` "is not satisfied" quando resolve, e pular a dep direta de `wincode` quer dizer que a expansão de `#[program]` não consegue nem nomear o serializador dela (`error[E0433]: could not find wincode in the list of imported crates`).

Então a divisão é deliberada, e vale enunciar isso como regra e não como solução de contorno. **A CLI vem do git; a biblioteca vem do crates.io.** Eles são grafos de dependência separados — o binário `anchor` em `~/.cargo/bin` foi linkado uma vez e nunca participa do resolve do seu programa — então fixá-los em refs diferentes do mesmo release não é descompasso, é precisão. E o pin do registro é o mais forte dos dois: o crates.io proíbe republicar uma versão, então `2.0.0-rc.1` são bytes que não podem mudar, enquanto um branch é um nome que aponta para onde quer que alguém tenha dado o último push. E o `anchor --version` imprime `2.0.0-rc.1` a partir da ponta do branch *e* a partir da tag, então a string de versão nunca vai te dizer em qual você está. A ref é o pin. A versão é só um rótulo.

Carregue as três linhas em todo crate de programa que você escrever neste curso, e re-verifique elas do jeito que você re-verifica todo pin de RC: elas param de ser necessárias no dia em que o RC reconciliar o grafo dele.

**5. Faça o build.** A partir da raiz do workspace:

```bash
anchor build
```

No macOS, se o build do programa em si bater na mesma parede de LTO, prefixe do mesmo jeito: `CARGO_PROFILE_RELEASE_LTO=off anchor build`. Um build limpo escreve o programa compilado em `target/deploy/greeter.so` e um keypair em `target/deploy/greeter-keypair.json`. Dois artefatos, dois trabalhos. O `.so` é o seu programa compilado para o formato de bytecode on-chain, a coisa de verdade que vai rodar dentro do runtime; fazer deploy não é nada mais que subir esses bytes para uma conta e marcar ela como executável. O keypair é a identidade on-chain do seu programa: a chave pública dele é o endereço que outras transações vão chamar, e a chave secreta dele é a autoridade que te deixa dar upgrade nos bytes deployados depois. Proteja o keypair. Perca ele e você nunca mais consegue dar upgrade neste programa, só fazer o deploy de um novo em um endereço novo.

O primeiro `anchor build` no RC é também o passo mais lento da configuração, porque o cargo está compilando o framework Anchor inteiro a partir do código, não baixando um crate pré-compilado. É esse o custo do canal git. Os builds seguintes são rápidos; só o primeiro paga o preço cheio.

**6. Aponte o Anchor para a devnet e financie uma carteira.** Coloque a CLI na devnet, garanta que você tem um keypair, e faça um airdrop de algum SOL de devnet para você mesmo pagar o deploy:

```bash
solana config set --url devnet
solana address                 # your deployer wallet; solana-keygen new if you have none
solana airdrop 2               # devnet SOL; retry if the faucet is rate-limited
solana balance
```

Depois sincronize o program id para que `declare_id!` e `Anchor.toml` batam com o keypair que você acabou de construir:

```bash
anchor keys sync
```

O `anchor keys sync` lê `target/deploy/greeter-keypair.json`, deriva a chave pública dele, e reescreve tanto o `declare_id!` no `lib.rs` quanto o endereço sob `[programs.devnet]` no `Anchor.toml` para baterem. Se você pular isso, o `declare_id!` ainda guarda o id placeholder do template e o deploy não vai se alinhar. Rode `anchor build` mais uma vez depois de sincronizar para o binário compilado carregar o id corrigido.

**7. Faça o deploy do R0 na devnet. Este passo é seu.** Tudo acima eu te levei pela mão. Este você roda e lê por conta própria:

```bash
anchor deploy --provider.cluster devnet
```

![O build emite o .so, o keys sync alinha os program ids, o deploy imprime um Program Id, e um explorador de devnet confirma que ele resolve como executável.](assets/v07-flowchart.png)

Sucesso é a cara das palavras **Deploy success** e de uma linha dizendo `Program Id:` seguida de uma string base58. Essa string é o endereço do seu greeter na devnet. Copie ela para a linha `R0 greeter program id` do `PINS.md`, com a data de hoje na coluna verified. Depois cole ela em qualquer explorador de devnet e confirme que a conta resolve como um programa executável. Essa resolução é o seu checkpoint. Se o explorador mostrar um programa executável no seu id, o R0 está no ar e o seu toolchain de RC isolado funciona de ponta a ponta.

Se o deploy falhar por falta de fundos, o airdrop não chegou ou foi pequeno demais, então rode de novo `solana airdrop 2` e confira `solana balance` antes de tentar outra vez. Se falhar em um program id descasado, você pulou `anchor keys sync` ou não refez o build depois. Arrume essa única coisa e faça o deploy de novo. Todo o resto que poderia dar errado neste ponto volta para a questão do PATH do passo 3: o `anchor` errado é que está fazendo o deploy.

## Challenge

A sua trava é simples de enunciar, e ou ela passa ou não passa.

**Completion, feito comigo:** o `PINS.md` existe e as quatro linhas que a caminhada da instalação acabou de verificar — a CLI do anchor, o Rust, a solução de contorno do macOS, e a CLI do Solana — estão preenchidas a partir dos valores que você de fato instalou, não desta página. Isso quer dizer que `anchor --version` imprimiu de verdade `2.0.0-rc.1`, o seu Rust é de verdade 1.89.0, a linha da solução de contorno do macOS está registrada se você está num Mac, e a linha da CLI do Solana está rotulada como pin de CI. O greeter sai do scaffold e o `anchor build` produz um `greeter.so`.

**Solo, só seu:** faça o deploy do R0 na devnet, cole o program id dele de volta na última linha do `PINS.md`, e ponha a data de freshness em cada pin.

**Aceitação:** o `anchor --version` imprime o RC, o greeter faz deploy, e o program id resolve como um programa executável em um explorador de devnet. Três fatos, todos conferíveis. Se os três valerem, você conquistou o seu primeiro deploy de V2 num toolchain que quase ninguém no ecossistema está rodando ainda.

Uma coisa para você sentar com ela enquanto compila. Agora você está seguindo duas linhas do Anchor ao mesmo tempo, a 1.1.2 e a `anchor-next`, e a da fronteira vai derivar para fora de debaixo dos seus pins. Isso não é um bug na sua configuração. É o acordo. A conveniência que você abriu mão, um `avm install` abençoado que só funciona, você trocou por estar semanas na frente no V2. O preço dessa troca é a coluna `verified`, e você paga ele re-rodando `anchor --version` e relendo seus pins em um cronograma em vez de confiar neles para sempre.

![Dois gatilhos alimentam um loop de observar-e-estampar que reescreve a data verified no PINS.md toda vez que um humano re-confere o RC que se move.](assets/v08-flowchart.png)

Faça esse cronograma existir, porque uma intenção vaga de "conferir de vez em quando" é como um arquivo de pins apodrece. Uma cadência que funciona num RC: re-rodar `anchor --version` no começo de qualquer sessão em que um build de repente se comporta diferente de ontem, e refazer o build do RC a partir do `anchor-next` quando as notas de release do projeto ou um build quebrado te disserem que o branch se moveu. Quando você re-verifica, você não confia na data que já está no arquivo. Você re-observa o valor e estampa a data de hoje, mesmo que o valor não tenha mudado, porque uma data fresca em um valor inalterado é informação por si só: ela diz que alguém olhou. A issue #4937, o descasamento entre `wincode` e `solana-address` que quebrou `#[account(borsh)]` e fechou em 2026-08-20, é o argumento inteiro em um bug. Uma dependência dois níveis abaixo se moveu, e a única defesa foi `--locked` mais um humano que re-conferiu. No estável você pode ser preguiçoso com isso. Na fronteira a re-conferência é o trabalho.

A razão mais profunda para fixar exatamente, em vez de seguir um "latest" flutuante, é de cadeia de suprimentos e vale nomear, já que um artefato de release que faltava é o que começou este desvio inteiro. Cada vez que você instala de um alvo móvel você está confiando no que aquele alvo por acaso é naquele instante. Uma versão fixada com um canal registrado e uma data é uma afirmação que você consegue auditar depois: este build exato, deste branch exato, verificado neste dia. Um release cortado ao menos te daria um artefato imutável para apontar; um branch não te dá nada além do commit que você por acaso buscou. Então você mantém a versão humana da mesma garantia: escreva, date, re-verifique.

## Onde isso chega

Você instalou um release candidate que o instalador oficial se recusa a tocar, você escreveu cada parede com uma data do lado, e você fez o deploy de um programa que abre uma conta na devnet. A instalação brigou com você e você ganhou, que é o único jeito que essa briga termina uma vez que você sabe que o RC mora na casa dele.

O greeter é uma caixa preta agora. Ele faz build e faz deploy, e você não tem ideia, mecanicamente, de como `declare_id!` e `#[program]` transformaram uma dúzia de linhas de Rust em uma conta executável na devnet. Isso é o próximo. Na próxima lição mesmo você abre essas duas macros e lê o que elas geram, forma por forma, e o greeter para de ser mágica. Você fez a parte difícil. O toolchain é real, o deploy é real, e o id no seu arquivo de pins é seu.

Até a próxima lição, id na mão. Entregue isso primeiro.
