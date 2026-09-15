# Ligue os instrumentos: perfilar, depurar e cobrir o swap

Na lição passada você se sentou na cadeira do framework e viu o que muda quando um mint chega como Token-2022 em vez de SPL simples. Você apontou um leitor pequeno do lado do cliente para um mint fornecido, leu o tamanho real dele e o transfer hook dormente dele direto do wire, e depois saiu caçando um mint cujo hook estivesse vivo. Nenhum código novo de programa foi escrito. O seu programa de swap, o R4 no barcade do Quarters, continua movendo tokens de fliperama para um lado e tickets para o outro. Funciona. E você ainda não mediu quanto custa uma troca só.

Faça a pergunta honesta: quantas unidades de compute uma troca queima? Toda resposta que você consegue dar hoje é um encolher de ombros disfarçado de estimativa. Este módulo inteiro é sobre trocar esse encolher de ombros por um número que você mesmo mediu, usando o ferramental first-party do próprio V2 em vez de um multiplicador de marketing do benchmark de outra pessoa.

Então, antes de qualquer teoria, faça a coisa. Se você ainda não colocou o V2 nesta máquina, faça o build do release candidate a partir do canal git documentado dele, a mesma instalação que você rodou no m01. Lembre daquela lição: o `avm install` não consegue buscar o RC, porque ele baixa binários pré-compilados das GitHub Releases e nenhum Release foi cortado para a tag v2. O caminho sancionado é um build a partir do código-fonte do lar atual do repositório, otter-sec/anchor (as URLs antigas coral-xyz e solana-foundation redirecionam para lá), fixado na tag `v2.0.0-rc.1`:

```bash
# The documented V2 RC install: build from source at the v2.0.0-rc.1 tag.
# Pin verified live on crates.io 2026-08-23: anchor-cli 2.0.0-rc.1 (published 2026-08-12).
# RC pins move fast: re-verify the tag and version before you pin a Dockerfile.
# The LTO prefix is needed on macOS when the link step dies, harmless on Linux
# (m01-l2 has the why). It is also why the earlier install blocks in this course
# print it: leave it on and one command works everywhere.
# Why git and not `cargo install anchor-cli@2.0.0-rc.1`? The crates.io publish is
# real but undocumented for CLI installs; the git build is the sanctioned channel
# for the BINARY. (The library is the opposite: your program crate takes
# anchor-lang from crates.io, because a published version cannot move.)
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
anchor --version
```

Agora, a partir do seu workspace do R4, rode o profiler contra os testes de swap que você já tem:

```bash
anchor test --profile
```

Quando terminar verde, olhe dentro de `target/anchor-v2-profile/`. Tem um SVG de flamegraph sentado ali, um por teste, que não existia cinco minutos atrás. Deixe ele aberto numa aba do navegador; o lab pega ele no passo 2 e lê ele direito. No fim você vai saber como ler ele, sobre o que ele mente, e o único número que vale anotar.

## Resumo

Você vai instrumentar o swap que já existe, não estender ele. Nenhuma lógica nova de programa é escrita. O R4 ganha uma camada de observabilidade construída a partir de quatro instrumentos. Três deles, o profiler, o debugger e a cobertura, rodam contra os testes de swap em LiteSVM que você já tem: nenhuma montagem nova, três leituras de uma rodada que já existe. O quarto, o Mollusk, é uma segunda montagem e vale dizer isso, porque ele te custa as próprias dev-dependencies dele, a própria fixture de conta dele e a própria invocação de `cargo test` dele. Esse é o preço de um número de CU preciso o bastante para virar asserção.

Três deles são do próprio Anchor: `anchor test --profile` para flamegraphs, `anchor debugger` para dar step numa instrução que falha, e `anchor coverage` para achar branches sem teste. O quarto é o `anza-xyz/mollusk`, um crate de terceiros sem nenhuma afiliação com o Anchor, que é de onde vem uma asserção precisa em unidades de compute sobre uma instrução só. Três instrumentos first-party mais um emprestado, e o emprestado é o único que trava. O lab percorre os quatro contra o swap como exemplo trabalhado. Depois você re-roda cada um contra o seu próprio swap sem ajuda e registra duas coisas: o custo de linha de base em unidades de compute de uma troca, e o nome do frame mais quente no flamegraph.

Essa divisão é o recuo da ajuda desta lição. Eu demonstro as quatro ferramentas no R4 com você olhando. Você re-roda elas no seu próprio programa com as rodinhas tiradas. E a interpretação, ler a largura do flamegraph e achar a lacuna de cobertura, é só sua no fim. Não tem código de programa para escrever. A única coisa que você escreve é uma constante só num teste, e o resto do fazer é medição.

Um termo de glossário, porque ele está em cada linha abaixo. Uma unidade de compute, ou CU, é a medição que a Solana faz do trabalho on-chain: cada instrução roda contra um orçamento de compute, e cada operação que o runtime executa debita alguma CU dele. "Quanto custa uma troca" quer dizer "quantas CU a instrução de troca consome." Mais barato quer dizer folga para mais trabalho na mesma transação e uma taxa menor no pouso.

## Os quatro instrumentos

Aqui está a forma do toolkit inteiro antes de a gente dirigir ele. Quatro ferramentas, quatro perguntas diferentes, uma fixture de teste compartilhada embaixo. Leia esta tabela uma vez e volte nela durante o lab.

![Um cartão de quatro linhas comparando o profiler, o debugger, a cobertura e o teste do Mollusk pela pergunta que cada um responde, pela saída dele, pelo tipo de build dele e por travar ou não a rodada.](assets/v01-comparison.png)

A coisa para internalizar é a última coluna. Três destes quatro reportam: eles te entregam um artefato e deixam você decidir o que ele quer dizer. Só o teste do Mollusk decide por você, porque um teste é um contrato de passa-ou-falha. Essa diferença é a razão de o número de linha de base com o qual você acaba se comprometendo morar no teste do Mollusk e em nenhum outro lugar.

Agora cada instrumento, na ordem em que você de fato vai pegar eles.

### O profiler: para onde a CU foi

O `anchor test --profile` roda a sua suíte normal de testes, mas ele compila o programa de um jeito específico e captura um artefato específico. Ele faz o build em DEBUG para o binário guardar os símbolos DWARF dele. DWARF é o formato de debug-info que mapeia instruções compiladas cruas de volta para os nomes das suas funções e as linhas de código. Sem ele, um profile é uma parede de endereços hex. Com ele, o profiler consegue rotular cada frame com a função à qual ele pertence.

O artefato capturado é um flamegraph. Um flamegraph é um gráfico de barras empilhadas de onde o tempo de execução, ou aqui o custo em compute, acumulou: cada caixa é uma função, a largura dela é o custo atribuído a ela, e as caixas empilham para mostrar quem chamou quem. Uma nota de orientação, porque ela decide para onde você olha: estes SVGs são desenhados em estilo icicle, com a raiz no topo e os chamados empilhando para baixo, então a caixa da sua instrução fica no topo e tudo o que ela chamou pendura *embaixo* dela. A caixa mais larga embaixo da raiz da sua instrução que seja código seu é, grosso modo, "para onde a CU foi." Um SVG é escrito por teste dentro de `target/anchor-v2-profile/`.

![Um fluxograma de cinco estágios desde a compilação em debug passando pela resolução de frames DWARF até um SVG por teste, avisando que a CU em debug mostra forma relativa em vez de custo em release.](assets/v02-flowchart.png)

Leia a figura. Aqui está o que um frame de flamegraph está te dizendo e o que ele não está.

![Um flamegraph estilizado onde os frames largos de matemática e de desserialização da instrução de swap são os hotspots de verdade enquanto um frame de montagem de teste igualmente largo aparece acinzentado como ruído de bancada de teste.](assets/v03-annotated-code.png)

Esse frame de bancada de teste é a primeira cilada e a mais comum. O seu teste em LiteSVM cunha e financia contas nas transações dele antes de chamar `swap_arcade_for_tickets`, e o profiler rastreia cada instrução da rodada, então essas instruções de montagem ganham as raízes delas no mesmo SVG, muitas vezes mais gordas que a troca. Elas são custo de verdade, mas não são o custo da sua instrução. Persiga uma delas e você vai otimizar a sua fixture de teste enquanto a troca continua exatamente tão cara quanto antes. Tudo com que você se importa pendura embaixo da raiz `swap_arcade_for_tickets` especificamente.

### O debugger: instrução por instrução

Às vezes o flamegraph não é a pergunta. Às vezes uma instrução falha e você precisa ver ela morrer. O `anchor debugger` é uma TUI no estilo foundry, uma interface de terminal em ratatui, que dá step no seu programa uma instrução sBPF por vez e te mostra os registradores conforme eles mudam. sBPF é o sabor Solana do bytecode eBPF no qual o seu Rust é compilado, a coisa de verdade que o runtime executa.

Para trabalho mais profundo ele se liga a um debugger de verdade. Passe `--gdb` e o Anchor expõe o gdb stub do solana-sbpf, um servidor pequeno que fala o protocolo remoto do gdb para você poder anexar o gdb e colocar breakpoints contra o programa rodando. No lab você vai apontar o debugger para uma troca quebrada de propósito e dar step até a instrução exata onde ela reverte.

### Cobertura: quais branches nunca rodaram

O `anchor coverage` responde uma pergunta que os outros três não conseguem: o que os seus testes nunca tocaram? Ele reconstrói cobertura de linha e de branch a partir de traces de registradores SBF e emite isso como LCOV, o formato padrão de relatório de cobertura de linha que editores e ferramentas de CI já sabem exibir. Aponte ele para o swap e ele vai te mostrar, por exemplo, que o branch da trava de slippage ou o seu retorno antecipado de valor zero nunca executou em nenhum teste.

Aqui está a cilada, dita sem enfeite para você não ficar esperando por ela: o `anchor coverage` reporta, ele não trava. Ele não vai falhar o seu build quando a cobertura cair. Ele te entrega um arquivo LCOV e vai embora. Se você quer um piso de cobertura imposto, isso é uma política de CI que você escreve em cima do relatório, não uma coisa que a ferramenta faz para você.

### Mollusk: exatamente quantas CU

As três primeiras ferramentas descrevem. O Mollusk faz assert. O Mollusk (`anza-xyz/mollusk`) é uma bancada de teste leve e in-process que roda uma instrução só numa SVM minificada e deixa você fazer checagens duras no resultado, incluindo uma checagem precisa em unidades de compute. Sem validador, sem localnet, sem async. Você monta uma instrução, entrega contas para ela, e faz assert tanto de que ela teve sucesso quanto de que ela consumiu um número específico de CU.

É aqui que o fio de testes do módulo escala. No m02 você escreveu um teste em LiteSVM: rápido, in-process, ótimo para comportamento. O LiteSVM responde "ele fez a coisa certa." O Mollusk responde "ele fez a coisa certa por exatamente esta quantidade de unidades de compute." A mesma velocidade in-process, um degrau mais afiado. Mais tarde, no capstone, o Surfpool entra para integração em localnet de salão inteiro contra estado real do cluster. Para uma asserção precisa em CU sobre uma instrução, hoje, o Mollusk é a ferramenta.

![Um diagrama de cubo e raios onde três instrumentos do Anchor leem a mesma rodada de swap em LiteSVM que já existe, com o teste de CU do Mollusk desenhado à parte como uma segunda fixture própria.](assets/v04-diagram.png)

### O trade-off, antes de você confiar em qualquer coisa disso

Instrumentação não é de graça e não é a verdade. Nomeie os custos agora para nenhum número te surpreender depois.

O `--profile` faz o build em DEBUG para pegar aqueles símbolos DWARF. Um build de debug não é o seu build de release, então os números de CU dele são inflados e têm forma diferente do que é entregue. Use o flamegraph para forma relativa, qual frame está gordo em relação aos outros, nunca como um custo absoluto que você cita para alguém. O debugger e a cobertura os dois adicionam tempo de build e de montagem que você paga em cada rodada, então você pega eles quando tem uma pergunta em vez de deixar eles ligados em cada rodada. E o limite mais profundo de todos: um flamegraph te diz ONDE o custo está, nunca POR QUÊ. As ferramentas medem. A próxima lição decide o que fazer com isso.

Essa honestidade não é só minha. A própria manchete de benchmark do V2 do Anchor ficou mais honesta com o tempo. No PR #4914, mergeado em 2026-08-13, os números de marketing foram revisados para baixo: a afirmação de "95% smaller bytecode" virou 94%, e a de "9.9x average CU reduction" virou 8.8x.

![Uma linha do tempo de dois pontos mostrando o PR #4914 em 2026-08-13 revisando a manchete do V2 do Anchor de 95 por cento para 94 por cento de bytecode e de 9.9x para 8.8x de CU média, motivando medir o seu próprio programa.](assets/v05-timeline.png)

Essa é a razão de este curso nunca te entregar um multiplicador para repetir. Uma manchete de benchmark é o programa de outra pessoa em cima da carga de trabalho de outra pessoa. A sua troca é sua. Meça ela.

## Lab: instrumente o swap

Exemplo trabalhado. A gente roda os quatro instrumentos contra o R4, o swap, e chega em um número de CU de linha de base para a troca mais o frame nomeado mais quente. Acompanhe no seu próprio checkout do R4. Cada comando abaixo é real.

### 1. Confirme o seu toolchain

Não fixe uma versão de memória. Essa é a última cilada e ela morde caladinho, porque um pin obsoleto compila bem e só mede a coisa errada.

```bash
anchor --version          # expect anchor-cli 2.0.0-rc.1 (the tag build; pin verified 2026-08-23)
solana --version          # expect 3.1.10, the course's local-CI pin from m01-l2.
                          # A different number is not an error, it is a note to yourself:
                          # your CU readings are against a different runtime than mine.
```

### 2. Gere o flamegraph

```bash
anchor test --profile
ls target/anchor-v2-profile/
```

Agora você deve ver um `.svg` por teste. Abra o do seu teste de swap num navegador. O gráfico é em estilo icicle, então as raízes ficam no topo: ache a caixa raiz `swap_arcade_for_tickets` lá em cima, ignore as raízes irmãs que são as transações de montagem do seu teste, e procure a caixa mais larga pendurada embaixo dela. No swap do Quarters do jeito que eu montei ele, esse frame de código próprio mais largo é `swap_out`, a matemática de produto constante, com a desserialização de conta logo atrás. Anote o que o seu de fato disser. Essa é metade da forma da sua resposta.

### 3. Dê step no caso que falha, no debugger

A gente quer algo que reverta, então quebre o swap temporariamente: no seu teste de slippage, coloque `min_out` um acima da saída cotada para a trava disparar. Depois aponte o debugger para aquele teste só pelo nome, a partir da raiz do workspace:

```bash
anchor build                                   # the debugger steps the built .so
anchor debugger --test slippage_reverts        # names the test whose transaction to step
```

Duas coisas para saber antes de a TUI abrir, porque um lançamento simples é confuso. O debugger não roda a sua suíte; ele faz o build do teste nomeado, repete a transação dele, e para na primeira instrução do seu programa, esperando por você. E ele dá step no sBPF *do seu programa*, não da bancada de teste, então a primeira instrução que você vê é o ponto de entrada, não `main`.

A TUI abre com o fluxo de instruções à esquerda e os registradores à direita. Dê step para frente e olhe eles. Você está procurando o momento em que o programa bate na sua trava `require!` e salta para o retorno de erro. Quando você achar, você localizou o branch que falha no nível da instrução, não adivinhou ele de um log.

Se você quer o gdb propriamente dito, relance com o stub. Ele escuta em `127.0.0.1:9001` e espera uma conexão antes de rodar qualquer coisa:

```bash
# gdb itself is not part of the Anchor toolchain. Install it once if you do not have it:
#   macOS: brew install gdb   Debian/Ubuntu: sudo apt install gdb
anchor debugger --test slippage_reverts --gdb
```

Depois, num segundo shell, anexe no stub e aponte o gdb para o binário sem stripping para ele conseguir resolver símbolos:

```bash
gdb target/deploy/token_ticket_swap.so
(gdb) target remote 127.0.0.1:9001
(gdb) break swap_arcade_for_tickets
(gdb) continue
```

Desfaça a sua quebra deliberada antes de seguir em frente. O trabalho do debugger aqui foi comprovar que você consegue caminhar uma instrução até a falha dela. Deixe o swap funcionando antes de você continuar.

### 4. Ache os branches sem teste

```bash
anchor coverage
ls target/anchor-v2-coverage/       # lcov.info lands here
```

Isso escreve `target/anchor-v2-coverage/lcov.info`, e LCOV cru é um formato de máquina: linhas `DA:` para acertos de linha, linhas `BRDA:` para acertos de branch, nenhum código à vista. Você não lê ele direto. Renderize ele:

```bash
# genhtml ships with lcov. macOS: brew install lcov   Debian/Ubuntu: sudo apt install lcov
genhtml target/anchor-v2-coverage/lcov.info -o target/anchor-v2-coverage/html
open target/anchor-v2-coverage/html/index.html    # xdg-open on Linux
```

Agora você tem uma view do código com cada linha colorida pela contagem de acertos. Clique dentro de `lib.rs` e leia quais branches do swap nunca rodaram. Muito provavelmente o seu caminho feliz está verde e um branch de borda, a reversão de slippage ou a trava de saída zero, aparece vermelho. Esse vermelho não é uma falha de build. Lembre: a cobertura reporta, ela não trava. Ela está te dizendo onde um teste futuro deve ir.

Se você preferir ficar no seu editor, a maioria das extensões de cobertura lê `lcov.info` direto; aponte uma delas para aquele caminho e pule o `genhtml`.

### 5. Fixe a linha de base com o Mollusk

Este é o que cola, e ele começa com um build, não com um teste. O Mollusk carrega um `.so` compilado do disco pelo nome, e nada naquele arquivo diz qual configuração fez o build dele — o `anchor test --profile` do passo 2 escreveu um build DEBUG ali, e passos posteriores escreveram em cima do diretório desde então. Meça o que por acaso estiver largado no disco e você não consegue nem dizer qual build você fixou como a sua linha de base. Então a regra, que vale toda vez que você tocar no Mollusk daqui em diante: **faça o build exatamente da configuração que você pretende medir, imediatamente antes de medir ela.**

```bash
cargo build-sbf              # release, defaults on: the configuration you actually ship
# Mollusk searches tests/fixtures, $SBF_OUT_DIR, and the current directory for the .so —
# NOT target/deploy. `anchor test` sets this var for you; a bare `cargo test` does not,
# and the miss reads `[MOLLUSK]: Program file not found`. Export it once per shell.
export SBF_OUT_DIR=$PWD/target/deploy
```

Depois adicione o Mollusk ao crate do seu programa como dev-dependency. Ele custa seis linhas de dev, mais uma checagem em um pin que o workspace inteiro já carrega:

```toml
# programs/token-ticket-swap/Cargo.toml
[dependencies]
# This row is the reason m02-l1 wrote the arcade pin as a ceiling instead of an equality.
# Mollusk's SVM stack reaches solana-address ^2.6.1, and `=2.6.0` refuses that resolve
# before anything compiles; 2.6.1 is still on wincode 0.5, so the real constraint — below
# 2.7 — still holds. Confirm it reads this way HERE AND IN EVERY SIBLING RUNG. Cargo
# resolves one solana-address for the whole workspace, so a single member still holding
# `=2.6.0` fails the entire workspace, not just its own crate.
solana-address = ">=2.6.1, <2.7"

[dev-dependencies]
# Pins verified live on crates.io 2026-09-01: mollusk-svm 0.15.1 (published 2026-08-29).
# A 0.15.0-agave-4.3.0-beta.0 also exists (2026-08-18); stay on the stable line unless
# you are tracking the agave 4.3 beta SVM. Mollusk 0.15 builds on the agave 4.2 SVM
# crates, so your solana dev-deps must be the 4.x line: a 2.x solana-sdk will not
# type-check against Mollusk's Pubkey/Account/Instruction types.
mollusk-svm = "0.15.1"
# Mollusk's minified SVM loads no program you don't register, and the trade CPIs
# into SPL Token — this companion crate (same 0.15 line, same publish batch)
# ships the real token-program ELF plus the two helpers the fixture and test
# use: token::add_program() and token::keyed_account().
mollusk-svm-programs-token = "0.15.1"
solana-sdk = "4"
# The two rows below hold Mollusk's own graph on the wincode 0.5 line. Without them the
# resolve succeeds and the BUILD dies, in solana-message and then in solana-transaction.
solana-short-vec = ">=3.2.2, <3.3"
solana-signature = ">=3.4.1, <3.5"
# The fixture builds real SPL mint and token-account state, and the test names
# spl_token::ID — the same pin m05-l1 installed.
spl-token = "9"
```

As duas linhas de range de `solana-*` são a classe de bug da issue #4937 de novo, uma camada mais para baixo, e vale entender elas em vez de colar. `solana-short-vec 3.3.0` e `solana-signature 3.5.0` os dois passaram para `wincode 0.6` ainda satisfazendo o que `solana-message 4.4.0` pede, então um resolve novo coloca dois majors de `wincode` no grafo e `solana-message` para de compilar contra qualquer um dos dois que o cargo escolher. Fixar os dois de volta abaixo desses majors mantém a linha solana 4.x inteira em `wincode 0.5`, que é a linha que o `anchor-lang 2.0.0-rc.1` já quer. Trate estes três pins como uma decisão só: quando o V2 cruzar para `wincode 0.6`, todos eles vão de uma vez.

Note a diferença de raio de impacto entre as linhas de dev e a linha `[dependencies]` acima delas, porque é essa a lição prática aqui. As duas linhas de range são dev-dependencies deste crate — elas moldam o lock do workspace, e nenhum irmão nunca precisa declarar elas. `solana-address` é o oposto, pela razão que o próprio comentário dele dá, e isso não é uma peculiaridade do Mollusk; é o que um workspace *é*. Um pin que é seu continua uma decisão por crate até o momento exato em que um irmão discorda dele.

Agora o teste preciso em CU. Ele monta a instrução `swap_arcade_for_tickets` usando os tipos que o Anchor gerou para o seu programa, entrega a fixture de conta para o Mollusk, e faz assert tanto no sucesso quanto nas unidades de compute. No exemplo trabalhado a bancada de teste e a montagem de contas são entregues para você. Aqui está a coisa inteira, com as duas linhas que você preenche durante o Challenge marcadas:

```rust
// programs/token-ticket-swap/tests/cu_baseline.rs
mod swap_fixture;

use anchor_lang::{InstructionData, ToAccountMetas};
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, instruction::Instruction, pubkey::Pubkey};

// The Anchor-generated instruction args + accounts for R4's swap handler.
// Anchor names both after the handler: `swap_arcade_for_tickets` -> `SwapArcadeForTickets`.
use token_ticket_swap::accounts::SwapArcadeForTickets as SwapAccounts;
use token_ticket_swap::instruction::SwapArcadeForTickets as SwapArgs;

/// Builds the swap fixture: the program, the accounts, and one swap instruction.
/// (Provided for you in the worked example. In your own swap you adapt the account list
/// to R4's actual `SwapArcadeForTickets` context.)
fn swap_fixture() -> (Mollusk, Instruction, Vec<(Pubkey, Account)>) {
    let program_id = token_ticket_swap::ID;
    // Mollusk loads the compiled .so by name, from wherever SBF_OUT_DIR points.
    let mut mollusk = Mollusk::new(&program_id, "token_ticket_swap");
    // A minified SVM runs only the programs you register, and the trade CPIs
    // into SPL Token. Put the real token program in the cache; the fixture
    // supplies the matching account row.
    mollusk_svm_programs_token::token::add_program(&mut mollusk);

    // `build_swap_accounts` builds the trader, the pool PDA, both mints, and the four
    // token accounts (two reserves, two trader-side), funds them, and returns them as
    // Mollusk's (Pubkey, Account) pairs. It is ordinary SPL fixture construction with
    // nothing V2-specific in it, so it ships beside this lesson at
    // `lessons/m06-l1/swap-fixture/`. Drop it in as `tests/swap_fixture.rs`
    // and `mod swap_fixture;` at the top of this file. Its full surface, which the
    // next lesson also leans on: keys(), build_swap_accounts(), build_swap_ix(),
    // build_init_accounts(), build_init_ix().
    let keys = swap_fixture::keys(&program_id);
    let accounts: Vec<(Pubkey, Account)> = swap_fixture::build_swap_accounts(&keys);

    let metas = SwapAccounts {
        trader: keys.trader,
        pool: keys.pool,
        mint_arcade: keys.mint_arcade,
        mint_ticket: keys.mint_ticket,
        reserve_arcade: keys.reserve_arcade,
        reserve_ticket: keys.reserve_ticket,
        trader_arcade: keys.trader_arcade,
        trader_ticket: keys.trader_ticket,
        token_program: spl_token::ID,
    }
    .to_account_metas(None);
    // The handler's own args: amount_in, and a min_out of 0 so the slippage guard
    // never decides the measurement for you.
    let data = SwapArgs { amount_in: 100, min_out: 0 }.data();
    let ix = Instruction { program_id, accounts: metas, data };

    (mollusk, ix, accounts)
}

#[test]
fn trade_cu_baseline() {
    let (mollusk, ix, accounts) = swap_fixture();

    // Read the raw number FIRST, so this test always prints what one trade costs
    // right now. `process_instruction` runs the trade and reports; it asserts nothing.
    let measured = mollusk.process_instruction(&ix, &accounts);
    println!("trade consumed {} CU", measured.compute_units_consumed);

    // YOU FILL THIS IN THE CHALLENGE: the CU bound you measured for one trade.
    // The assertion below is written for you; the number is the exercise.
    const TRADE_CU_BASELINE: u64 = /* your measured baseline */ 0;

    mollusk.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::success(), Check::compute_units(TRADE_CU_BASELINE)],
    );
}
```

O módulo de fixture em si é entregue ao lado desta lição como [swap-fixture/swap_fixture.rs](swap-fixture/swap_fixture.rs) — layouts reais de mint e de conta de token em SPL, o pool no PDA dele, nada que você já não tenha encontrado.

Rode ele agora, antes de você ter qualquer número para fixar. O `println!` dispara antes da asserção, então o limite placeholder `0` falha o teste e você mesmo assim sai com a sua medição:

```bash
cargo build-sbf && cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Resultado esperado: uma linha dizendo `trade consumed <N> CU`, seguida de uma falha em `Check::compute_units(0)`. Esse `N` é a sua linha de base. Coloque ele em `TRADE_CU_BASELINE` e rode o mesmo comando de novo; desta vez ele fica verde, e daqui em diante `Check::compute_units` falha o build no dia em que uma mudança tirar a troca daquele número. É esse o ponto de fixar isso num teste e não num flamegraph: o flamegraph é um instantâneo que você olha, a asserção do Mollusk é um alarme que vigia por você. Deixe a leitura-e-print no topo do teste, porque é assim que você vai tirar a leitura do "antes" na próxima lição.

Um número de referência para escala. A Helius publicou contagens de CU do V1 para um programa contador trivial, mais ou menos 5,095 para inicializar e 1,162 para incrementar: sem data, V1, um programa diferente. Use eles para uma coisa só, uma noção de ordem de magnitude. Uma instrução de verdade vive nos milhares de CU, não nas dezenas e não nos milhões. Se a sua leitura estiver muito fora dessa banda, desconfie da sua fixture antes de celebrar.

![Um gráfico de barras de um contador V1 sem data em 5095 e 1162 CU ao lado de uma barra fantasma para a própria troca do leitor, com legenda de só-escala em vez de meta.](assets/v06-chart.png)

## Challenge: meça a sua própria troca

Você rodou os comandos ao meu lado. Agora rode eles no frio, sem a página aberta, e produza dois fatos seus. A diferença não são as teclas, é que nada aqui te diz o que você está a ponto de ver.

Completion primeiro, a única linha que você escreve aqui. Preencha `TRADE_CU_BASELINE` em `trade_cu_baseline` com o número que você mediu, e veja o teste ir de vermelho para verde nessa única edição.

Depois a rodada solo:

1. `anchor test --profile` no seu swap. Abra o SVG, ache a raiz `swap_arcade_for_tickets`, e nomeie o frame de código próprio mais largo embaixo dela. O que ele disser é a sua resposta, inclusive se não for o frame que eu peguei: o meu swap e o seu divergiram por quatro lições de edições, e um frame quente diferente é um achado, não um erro. A única resposta errada é um frame que não está embaixo daquela raiz de jeito nenhum.
2. `anchor debugger` numa troca falhada de propósito. Dê step até a instrução que falha, depois desfaça a quebra.
3. `anchor coverage`. Abra o LCOV e nomeie um branch que os seus testes nunca exercitaram.
4. Leia a CU do Mollusk, fixe ela em `TRADE_CU_BASELINE`, e confirme que o teste fica verde.

A forma da sua resposta são exatamente duas coisas: um inteiro de CU para uma troca só, e o nome do frame mais quente do flamegraph. Anote eles em algum lugar onde você vai achar eles na próxima lição.

![Um cartão de registro com espaços em branco para a linha de base de CU da troca, o frame mais quente, as ferramentas e os tipos de build usados, o branch sem teste encontrado, e a data da medição.](assets/v07-table.png)

A barra para passar é simples e estrita. As quatro ferramentas rodam limpas. O número de linha de base existe e mora numa asserção do Mollusk que passa. E o frame que você nomeou é um que pendura embaixo da raiz da sua instrução, então ele é trabalho de instrução e não trabalho de fixture, qualquer que seja o nome dele no fim.

## Antes de seguir em frente

Se cheque contra três perguntas, porque estes são os lugares exatos onde esta lição dá errado na prática.

A sua CU de linha de base está vindo do Mollusk, não do flamegraph? Esse é o único lugar de onde ela tem permissão de vir, porque o flamegraph é um build de debug e os números dele são forma e não custo. O número com o qual você se compromete é o do Mollusk.

O seu frame mais quente está embaixo da raiz `swap_arcade_for_tickets`, e não embaixo de uma das raízes irmãs que as transações de montagem do seu teste produziram? Só frames embaixo da sua instrução são o custo da sua instrução.

Você notou que o `anchor coverage` nem uma vez ameaçou falhar o seu build? Bom. Ele reporta. Nada aqui trava, exceto o teste que você mesmo escreveu.

Você tem um número de verdade agora, medido por você, no seu programa, sem nenhum multiplicador emprestado de ninguém. Esse número é uma linha de partida, não uma chegada. O flamegraph está te mostrando um frame gordo sentado na sua instrução de troca. Você consegue ver o custo. Na próxima lição você faz ele menor, uma mudança medida por vez, com CU de antes-e-depois como a única prova que conta.

Vá pegar o seu número.
