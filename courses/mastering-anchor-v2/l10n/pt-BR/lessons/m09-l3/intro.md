# Capstone: rode o salão de fliperama inteiro

Você acabou de diffar o seu vault nativo contra a expansão de macro do V2, linha por linha, e deu a olhada de um compasso no asm-v2. Você agora consegue prever o que o derive escreve e por quê, que quer dizer que o framework parou de ser uma caixa preta na última vez em que você rodou `cargo expand`. Bom. Segure isso, porque esta lição gasta isso.

Aqui está a dor, dita sem enfeite. Todo degrau que você construiu fica sozinho. O cabinet-counter conta. O quarter-vault guarda. O prize-escrow acerta. O token-ticket swap cota. Quatro programas, quatro suítes de teste verdes, um deles — o swap — já vivo na devnet, e nenhum deles sabe que os outros existem. Um fliperama não é quatro máquinas em quatro salas. É um salão: uma partida incrementa um counter, o counter alimenta um crédito para dentro de um vault, uma vitória libera um prêmio de um escrow, e uma pilha de tickets troca por alguma coisa no balcão. Ninguém ligou o salão ainda. É esse o capstone, e ele é quase inteiramente seu.

Uma tarefa de arrumação primeiro, porque tudo abaixo assume ela. O R1 vem vivendo por conta própria desde m02-l1: o `anchor init cabinet-counter` fez dele um workspace de um só, enquanto o R2, o R3 e o R4 cresceram todos dentro do workspace `quarter-vault` que você começou em m03-l1. O salão faz build, testa e faz deploy como um workspace só — um `Anchor.toml` só, um `target/deploy` só de onde toda bancada de teste carrega, um diretório `idls/` só — então copie o R1 para dentro antes de gerar o scaffold de qualquer coisa. O workspace original fica onde está — isto é uma cópia, não uma movimentação, e você pode apagar a árvore antiga depois que o registry fizer build. A partir da raiz daquele workspace do salão:

```bash
cp -R ../cabinet-counter/programs/cabinet-counter programs/cabinet-counter
cp ../cabinet-counter/target/deploy/cabinet_counter-keypair.json target/deploy/
```

Depois registre ele do jeito que o `anchor new` teria feito: acrescente ele ao `members` do `Cargo.toml` do workspace se essa lista nomear os programas um por um em vez de dar glob em `programs/*`, e acrescente a linha dele embaixo de `[programs.localnet]` no `Anchor.toml`. Mantenha o `declare_id!` com que o crate chega — esse id casa com o keypair com que você fez o deploy do R1 em m02-l1, e editar ele na mão órfã o deploy. O segundo `cp` acima levou o keypair dele junto, então o `anchor keys sync` vai confirmar que o par ainda casa. Uma última checagem enquanto você está por lá: a linha `solana-address` dele tem que ler o range de m02-l1, porque o R1 é membro deste workspace agora e um único membro segurando uma igualdade reprova o resolve inteiro.

Agora vamos fazer o salão existir antes da teoria. Gere o scaffold do último programa e aponte ele para os quatro que você já entregou. Nenhuma instalação nova para esta parte:

```bash
anchor new floor-registry   # adds programs/floor-registry to the workspace
```

Depois ligue o registry aos quatro degraus do mesmo jeito que o escrow alcançou o vault em m04-l3 — pelas interfaces deles, quatro vezes seguidas. Primeiro, extraia a IDL de cada degrau para o diretório `idls/` na raiz do workspace. Os degraus precisam ser nomeados *antes* de o registry sequer compilar, porque o `declare_program!` lê o JSON em tempo de expansão de macro:

```bash
mkdir -p idls
for rung in cabinet-counter quarter-vault quarter-prize token-ticket-swap; do
  ( cd programs/$rung && anchor idl build -o ../../idls/$(echo $rung | tr '-' '_').json )
done
```

(O seu `idls/quarter_vault.json` já existe da última re-extração do módulo 5; o loop simplesmente atualiza os quatro para o que os degraus disserem hoje, que é o único estado de IDL contra o qual vale fazer build.)

O `Cargo.toml` do registry então carrega **zero linhas de degrau** — este é o diff contra todo workspace multi-crate que você já viu antes, e ele é uma deleção:

```toml
[dependencies]
# The rc.1 crates landed on crates.io (2026-08-12), and that is where the LIBRARY comes
# from course-wide: a published version is immutable. The CLI is the git build (see m01-l2).
anchor-lang = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row, identical to the one every rung has carried since m02-l1.
# Step 4 puts Mollusk in this crate, and Mollusk's SVM stack reaches solana-address
# ^2.6.1; the ceiling is what the pin was always for — 2.6.1 is still wincode 0.5, and
# 2.7.0 is the version that moved. The registry and the four rungs share one
# workspace and one lock, so all five members must read this row, not just this one.
solana-address = ">=2.6.1, <2.7"
# No `{ path = "../<rung>", features = ["cpi"] }` rows — the rungs arrive as IDLs.
```

e o topo de `programs/floor-registry/src/lib.rs` nomeia os quatro degraus (o R3 é o crate `quarter-prize`, o scaffold de m04-l3 — "prize-escrow" é o papel que ele faz no salão, não o nome pelo qual o cargo conhece ele):

```rust
use anchor_lang::prelude::*;

declare_program!(cabinet_counter);
declare_program!(quarter_vault);
declare_program!(quarter_prize);
declare_program!(token_ticket_swap);
```

Por que não quatro linhas de dep por path com o hook `features = ["cpi"]` do scaffold, do jeito que metade dos tutoriais de Anchor que você leu faz? Porque no rc.1 essa estrada acaba fisicamente em um degrau. O `cpi` liga o `no-entrypoint`, e sob `no-entrypoint` cada programa consumido exporta a função de despacho dele como um símbolo sem mangling — então no momento em que um programa referencia os módulos `cpi` de *dois* desses degraus, o link SBF morre com `duplicate symbol: __anchor_dispatch`, e um registry que compõe quatro degraus nunca linka de jeito nenhum. O mesmo hook tem uma segunda falha, mais silenciosa: um `cargo build-sbf` na raiz do workspace unifica a feature `no-entrypoint` no próprio degrau consumido e emite um `.so` *sem símbolo de ponto de entrada* — ele faz build limpo, fica em `target/deploy/` com cara de que dá para fazer deploy, e o loader rejeita ele. O `declare_program!` não é uma solução de contorno adotada a contragosto; ele é o mecanismo próprio do V2 entre programas, gera a superfície de CPI em modo de interface, onde não existe símbolo de despacho para colidir, e o curso aposentou as linhas de feature `cpi` no dia em que o salão de quatro degraus virou a meta.

Rode o `anchor build`. Ele vai compilar um registry que ainda não faz nada, mas os quatro módulos gerados estão em escopo agora, e o compilador vai começar a te dizer exatamente quais handles cada degrau espera. Esse loop de feedback é o lab inteiro.

**Resumo.** O floor-registry é um programa Anchor V2 só que compõe os quatro degraus por CPI: ele incrementa o counter de um fliperama (R1), roteia créditos pelo quarter-vault (R2), acerta prêmios pelo prize-escrow (R3), e cota pelo swap (R4). Você vai ligar a borda do R1 como um passo trabalhado, depois construir o resto solo, depois levar a coisa inteira pelo ciclo de vida de produção completo que este curso vem ensinando: uma suíte de LiteSVM mais Mollusk, um caso de fuzz verde, uma passada no checklist de segurança, um perfil de CU com uma otimização medida, uma rodada de localnet no Surfpool do salão de cinco programas junto, um deploy na devnet, e um verify-from-repo local que comprova que o seu build reproduz o bytecode on-chain. Quando o `anchor test` imprimir `floor-registry ... passing` contra o salão em localnet e o seu verify casar, você terminou.

O recuo da ajuda, dito em voz alta para você saber o que é seu. A CPI do counter do R1 é trabalhada para você por inteiro, porque acreção neste curso é sempre demonstrada, nunca entregue pronta. A gramática de CPI que você reusa para os outros três degraus está na página. Tudo depois disso (as instruções de vault, escrow e swap, depois cada passo do ciclo de vida) é o capstone. É solo. Eu vou te mostrar a forma de cada jogada e o comando que comprova ela, e você vai rodar isso contra o seu próprio código.

## O salão como um programa só

Comece pela figura, porque o registry é mais fácil de segurar como um hub. Ele é dono de quase nenhum estado próprio. Do que ele é dono é das decisões sobre *quando* chamar cada degrau e *em que ordem*, e ele delega toda mudança de estado de verdade ao programa construído para ela. É esse o argumento inteiro a favor da composição: o registry é uma coisa pequena sobre a qual você consegue raciocinar, parafusada em quatro coisas comprovadas em que você já confia.

![O floor-registry fica no centro com uma seta de CPI para cada um dos quatro degraus, mais uma segunda seta mostrando o acerto do prêmio como uma chamada de dois saltos.](assets/v01-diagram.webp)

### Por que um programa e quatro CPIs, e não um programa grande

Pause na escolha de design antes da mecânica, porque é essa a escolha que o capstone inteiro está defendendo. Você poderia escrever um programa monolítico só que conta partidas, guarda créditos, acerta prêmios e cota swaps, tudo num crate só. Ele faria deploy como um `.so` só, não precisaria de CPI nenhuma, e rodaria em uma profundidade de invocação só. Para um projeto de fim de semana é menos código. Então por que o registry é um hub de quatro chamadas em vez disso?

A resposta é a divergência, e é a mesma razão pela qual o escrow não reimplementou custódia. Cada degrau é uma coisa limitada que você já testou, endureceu, fuzzou e perfilou. Dobre a lógica dele para dentro de um monolito e agora você é dono de uma segunda cópia dessa lógica, uma que não compartilha nada com o degrau entregue e que começa a divergir no dia em que você corrige um bug em um e esquece o outro. Duas cópias de matemática de custódia são dois bugs de custódia esperando para sair de sincronia. Compor sobre os degraus quer dizer que o registry é dono de exatamente uma responsabilidade, a decisão sobre o que chamar e quando, e toda mudança de estado de verdade fica atrás da interface do programa construído para ela. Quando você corrige o vault, o salão ganha a correção de graça, porque o salão nunca teve um vault próprio. Essa é a diferença entre uma base de código que fica mais segura conforme cresce e uma que acumula cópias do mesmo erro.

Tem uma segunda razão que só aparece na emenda: uma interface para dentro da qual você faz CPI é um contrato que você consegue verificar de forma independente. Os testes do vault comprovam o vault. Os testes do registry comprovam que o registry chama o vault direito. Nenhum dos dois tem que recomprovar o outro, e um auditor consegue ler cada um isolado. Um monolito colapsa os dois num blob só em que a lógica de contagem e a lógica de custódia conseguem alcançar o estado uma da outra em silêncio, e agora nada é comprovável sozinho. Coisas pequenas parafusadas em coisas comprovadas, cada uma checável por si, é como você mantém auditável um programa que cresce. A CPI é o parafuso.

### Composição é uma CPI, e a gramática é a que você já escreveu

Não tem nada de novo para aprender sobre como um programa chama outro. Você fez isso no escrow. O registry faz a mesma coisa quatro vezes. A gramática de CPI do V2 tem três partes e você já usou as três. Primeiro, o programa chamado expõe uma struct de accounts gerada, um `CpiHandle` por conta, que você preenche com `.cpi_handle_mut()` para as contas que o programa chamado vai escrever e `.cpi_handle()` para o resto. Segundo, o `CpiContext::new` recebe o program id do programa chamado através do `.address()` na conta `Program` dele, que no V2 entrega um `&Address`, não um clone de `AccountInfo`. Terceiro, o wrapper gerado empacota os seus argumentos e invoca.

Vale pausar no contraste, porque é a diferença entre a linha que você deletou e a linha que você manteve.

![Uma tabela de três colunas comparando cada peça de uma chamada de CPI entre as linhas do Anchor, enfatizando que leituras obsoletas-depois-de-CPI viraram erro de compilação no V2.](assets/v02-comparison.webp)

O único hábito que vai direto para dentro do capstone: leia qualquer estado de que você precise *antes* de abrir um handle. Uma vez que o `.cpi_handle_mut()` faz borrow de uma conta, você não consegue tocar naquela conta pela view tipada dela até a chamada consumir o `CpiContext` e o handle ser solto. Isso não é mais uma regra que você segue. É uma regra que o compilador segue por você, e é exatamente por isso que o swap do vault leu as reservas dele de antemão, uma vez, antes de cotar. Continue fazendo isso e o borrow checker vai continuar pegando as suas leituras obsoletas antes de um validador ver elas.

### O custo de confiança, e o teto de profundidade

Composição é poderosa, mas nomeie a troca com honestidade, porque é esse o ponto da lição. Todo degrau para dentro do qual você faz CPI é um degrau em que você confia. O registry confia que o counter atualiza o fliperama certo, que o vault move os lamports certos, que o escrow libera só numa condição verdadeira. Essa confiança é código de verdade que você consegue ler e testes que você já rodou, o que é muito melhor do que confiar no programa de um estranho. Continua sendo confiança, e continua custando.

Dois custos são concretos. O primeiro é a profundidade de invocação. O runtime da Solana limita o quão fundo uma cadeia de CPIs consegue aninhar: a altura máxima da pilha de invocação é **5**, que é a sua instrução de nível de topo mais quatro CPIs aninhadas. Uma subida está especificada, o SIMD-0268, "Raise CPI Nesting Limit", status Accepted, que levaria o aninhamento de 4 para 8, mas o feature gate dele, `6TkHkRmP7JZy1fdM6fg5uXn76wChQBWGokHBJzrLB3mj`, não tinha conta na mainnet quando esta lição foi escrita (sondado em 2026-08-22), então 5 é o número em vigor. Sonde o gate de novo na hora do build em vez de confiar nesta frase para sempre; um gate pendente é exatamente o tipo de fato que vira entre um curso ser escrito e um curso ser lido. Isso importa aqui porque o `settle_prize` é uma chamada de três saltos: o registry faz CPI no escrow, o escrow faz CPI no vault, e o vault faz CPI no token program. Conte: a sua instrução de nível de topo mais três chamadas aninhadas é altura de pilha 4, então você tem exatamente uma chamada aninhada de folga sobrando. É esse o tipo de número que você não conseguia calcular de jeito nenhum quando cada programa morava sozinho.

![Uma pilha vertical mostrando o settle_prize aninhando pelo floor-registry, prize-escrow, quarter-vault e o token program, chegando à altura de pilha 4 do máximo de 5.](assets/v03-diagram.webp)

O segundo custo é a própria disciplina de borrow, mas esse é um presente disfarçado de custo: o compilador te fazendo sequenciar as suas leituras é a razão pela qual um acerto de dois saltos não paga em silêncio contra um saldo obsoleto. Você paga em um pouco de rigidez de antemão e isso te compra uma classe de incidente das 2 da manhã que você nunca vai ter.

Uma propriedade trabalha a seu favor em todo degrau, e vale nomear ela porque muda como você raciocina sobre falha. Uma transação é atômica. Se qualquer CPI na cadeia devolve um erro, a transação inteira reverte, e toda mudança de estado acima dela é desfeita junto. Então o `settle_prize` não consegue acertar pela metade: se a checagem de condição do escrow falha, a CPI de redeem dá erro, e o depósito ou o incremento do counter mais cedo na mesma transação também é desenrolado. Isso é uma rede de proteção de verdade, e também é uma armadilha se você se apoiar nela como a sua única trava. A atomicidade te salva quando uma chamada *dá erro*. Ela não faz nada quando uma chamada *tem sucesso sob uma premissa falsa*, que é exatamente por que o escrow checa a condição antes de montar a CPI de liberação em vez de pagar primeiro e confiar na reversão. A ordem continua sendo a sua trava. A atomicidade é o último recurso, não o plano.

## Lab: ligue o salão e rode ele de ponta a ponta

Este é o lab do capstone. A borda do R1 é trabalhada. O resto é seu. Eu vou manter os passos do ciclo de vida secos, um comando e a saída que comprova ele, porque a esta altura você já rodou cada uma dessas ferramentas pelo menos uma vez e o capstone é sobre montar elas, não sobre reensinar elas.

> Freshness note: isto foi escrito contra o release candidate do Anchor V2 na linha 2.x (a árvore de docs publicada sob `v2`), `2.0.0-rc.1` em 2026-08-22. Instale o toolchain a partir do canal git documentado (passo 0, o `avm` não consegue buscar o RC). O `anchor-cli 1.1.2` default da máquina é a linha V1 e não vai compilar a gramática de `CpiHandle` ou `&Address` abaixo. Os pins de versão neste lab carregam a data em que foram checados; reconfira antes de fazer o build.

**Passo 0. Fixe o toolchain.** Uma linha, e a armadilha atrás dela é uma que você já sabe de cor — o `avm install` ainda dá 404 no RC, nenhum GitHub Release foi cortado para a tag — então se a checagem falhar, refaça a instalação git de m01-l2 (`--tag v2.0.0-rc.1`, `--locked`):

```bash
anchor --version           # expect: anchor-cli 2.0.0-rc.1
```

Uma nota de linha de versão para ninguém tropeçar: este curso fixa a CLI da Solana em `3.1.10` como o toolchain local de build e de CI, que é o que o contêiner de build verificável usa. Esse pin é uma escolha de reprodutibilidade, não uma afirmação sobre a rede atual. O release estável atual do Agave é uma coisa separada e que se move mais rápido (v4.2.1 em 2026-08-22; confira `agave-install info` ou `solana --version` na hora do build). Nunca leia o pin `3.1.10` como "a versão atual da Solana".

**Passo 1. As quatro interfaces já estão dentro.** Você extraiu as IDLs e escreveu as quatro linhas de `declare_program!` na abertura. Confirme que o `anchor build` ainda compila o registry vazio com todos os quatro módulos gerados resolvidos. `` error: `idls` directory not found `` quer dizer que a extração nunca rodou — os degraus precisam ser nomeados antes de o registry compilar. Um *item* faltando dentro de um módulo (uma função ou campo que o compilador não consegue encontrar) quer dizer um JSON obsoleto: rode de novo o loop de extração, porque o `declare_program!` compila contra o arquivo, não contra o código-fonte.

**Passo 2. Ligue o R1, o incremento do counter (trabalhado para você).** Uma partida num fliperama é uma CPI só: o registry chama o `post_score` do counter — `increment` como m02-l1 escreveu ele primeiro, renomeado em m02-l2 quando o fliperama ganhou um board. Aqui está ele por inteiro. Leia cada linha, porque este é o template que você vai copiar três vezes.

```rust
use anchor_lang::prelude::*;

declare_program!(cabinet_counter);
declare_program!(quarter_vault);
declare_program!(quarter_prize);
declare_program!(token_ticket_swap);

// Everything a caller needs comes out of the generated module: the `cpi`
// builders, the rung's account types (`Cabinet`, straight from the IDL), and —
// unlike source-level consumption, where the caller hand-writes it — the
// Program<T> marker, at cabinet_counter::program::CabinetCounter, with its
// IDL_ADDRESS already filled from the JSON.
use cabinet_counter::cpi as counter_cpi;
use cabinet_counter::program::CabinetCounter;
use cabinet_counter::Cabinet;

declare_id!("F1oorReg1stry111111111111111111111111111111");

#[program]
pub mod floor_registry {
    use super::*;

    // A play bumps the cabinet's counter by CPI-ing into R1.
    // This is the accretion edge, wired as a worked step, not handed to you finished.
    pub fn record_play(ctx: &mut Context<RecordPlay>, score: u64) -> Result<()> {
        // Build the callee's accounts struct from HANDLES, not AccountInfos.
        // cpi_handle_mut() takes a live borrow of `cabinet` for the callee; while it is
        // held you cannot also touch `cabinet` through its typed view. That borrow IS the
        // reload discipline, enforced by the compiler instead of your memory.
        // Named after the INSTRUCTION, and m02-l2 renamed it: `increment` became
        // `post_score` when R1 grew a leaderboard, so the generated struct is
        // `accounts::PostScore`, not `accounts::Increment`.
        let cpi_accounts = counter_cpi::accounts::PostScore {
            cabinet: ctx.accounts.cabinet.cpi_handle_mut(),
            player: ctx.accounts.player.cpi_handle(),
        };

        // .address() hands the callee's program id as &Address (V2), not an AccountInfo.
        let cpi_ctx = CpiContext::new(
            ctx.accounts.cabinet_counter_program.address(),
            cpi_accounts,
        );

        // The generated wrapper packs the score and invokes R1.post_score, whose
        // argument m02-l2 named `points`.
        counter_cpi::post_score(cpi_ctx, score)?;
        Ok(())
    }
}

// V2 wrappers carry no <'info> lifetime, and handlers take &mut Context<T>.
// If you catch yourself typing Account<'info, Cabinet>, you are on the V1 line.
#[derive(Accounts)]
pub struct RecordPlay {
    // Owner-checked to the cabinet-counter program; R1's own increment context
    // re-validates the [b"cabinet", player] seeds when the CPI lands.
    #[account(mut)]
    pub cabinet: Account<Cabinet>,
    pub player: Signer,
    pub cabinet_counter_program: Program<CabinetCounter>,
}
```

O jogador assina a transação de fora, e esse privilégio de signatário se estende para baixo pela CPI, então o counter vê um `player` assinado sem o registry assinar nada ele mesmo. Nada aqui é novo. É o depósito `reserve` do escrow com nomes diferentes.

Resultado esperado: o `anchor build` compila o registry com uma instrução e sem avisos sobre caminhos `counter_cpi` não resolvidos. Um erro "no method named `cpi_handle_mut`" quer dizer que você está na CLI V1 default da máquina, não no RC do passo 0; um `cabinet_counter::cpi` não resolvido quer dizer que a linha `declare_program!` ou o `idls/cabinet_counter.json` dela está faltando. Uma regra de nomenclatura para guardar no bolso para as bordas solo: as structs de accounts de CPI geradas são nomeadas a partir da **instrução** (`accounts::PostScore` para `post_score`), não a partir do nome que o programa chamado deu ao próprio tipo de contexto dele — a IDL carrega nomes de instrução, e nos degraus os dois por acaso coincidem. Que é também por que um rename que você fez dois módulos atrás te alcança aqui: um `unresolved import counter_cpi::accounts::Increment` é um *nome* obsoleto, não uma extração obsoleta, e rodar a extração de novo não vai consertar isso.

![Um cartão de código anotado isolando as três partes reusáveis de uma CPI do V2, com a regra de ler estado tipado antes de abrir um handle.](assets/v04-annotated-code.webp)

**Passo 3. Ligue o R2, o R4 e o R3 (solo).** Estes são o capstone. Cada um é a mesma gramática de três partes apontada para um degrau diferente. Construa eles um de cada vez e deixe o `anchor build` te dizer quais handles estão faltando.

- O `route_credit` chama `quarter_vault::cpi::deposit(cpi_ctx, amount)`. Esse é o R2 como ele está depois do módulo 5: o vault com upgrade para SPL, cujo `deposit` move tokens com `transfer_checked`, não a versão de lamport do módulo 4. Então as contas que você preenche são o estado do vault, o depositante, o mint e as duas contas de token. O jogador é o depositante e assina, então isto é um `CpiContext::new` simples, sem signer seeds. Mesma forma do `reserve` do escrow, um degrau para fora.
- O `quote_swap` chama `token_ticket_swap::cpi::swap_arcade_for_tickets(cpi_ctx, amount_in, min_out)`. Leia as reservas de que você precisa antes de abrir qualquer handle, depois passe as contas do swap. A trava de slippage já mora dentro do R4; o registry só roteia.
- O `settle_prize` chama `quarter_prize::cpi::redeem(cpi_ctx, final_score)` — o R3, o prize-escrow, cujo crate o cargo conhece como `quarter-prize`. Esta é a chamada mais funda do salão, então cuidado com a profundidade: o próprio escrow vai fazer CPI no vault para liberar. O registry não assina pelo PDA do escrow. O escrow assina por si mesmo, como sempre fez.

Duas dessas têm um porém que vale sinalizar antes de você esbarrar nele. O `quote_swap` lê as reservas do pool para dimensionar a troca, e essa leitura tem que acontecer antes de você abrir qualquer handle a partir dessas mesmas contas de reserva, ou o borrow checker te para seco. Esta é a própria disciplina de ler-antes-do-handle do swap, agora uma camada para fora: o registry lê, depois roteia. E o `settle_prize` é o caminho mais fundo do salão, então mantenha o diagrama de profundidade na cabeça. Conte com precisão, porque o número é o ponto: a sua instrução de nível de topo é altura 1, a chamada do registry para dentro do escrow é 2, a chamada do escrow para dentro do vault é 3, e o `transfer_checked` do vault para dentro do token program é 4. Quatro dos cinco que o runtime permite. Uma chamada aninhada de folga sobrando. Acrescente um degrau entre o registry e o escrow e você gastou ela.

Se uma chamada se recusa a fazer build com um erro de borrow, quase sempre é uma leitura tipada sentada acima da linha que solta um handle. Mova a leitura para cima, antes de o handle abrir, e tente de novo. Esse erro é o compilador fazendo a sua disciplina de reload por você.

**Passo 4. A suíte de unidade: LiteSVM mais Mollusk.** Cada degrau já tem testes. O registry precisa dos dele, exercitando cada borda isolada contra um runtime in-process. O LiteSVM roda o seu programa compilado inteiro num validador leve em memória; o Mollusk dirige uma instrução só e reporta a CU que ela queimou. Acrescente os dois como dev-dependencies, e repare que os dois chegam por rotas diferentes:

```bash
# LiteSVM comes through the V2 harness, never by name. anchor-v2-testing owns the
# litesvm version (0.11.0 at tag v2.0.0-rc.1; the anchor-next head has already moved
# it to 0.13.1), so pinning the tag pins the SVM. A bare `cargo add --dev litesvm`
# resolves the crates.io latest against your rc.1 program: two SVM majors, one graph.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1

# Mollusk is a separate stack and carries its own solana pins, exactly as in m06-l1:
# 0.15 builds on the agave 4.x SVM crates, so the measurement tests need solana-sdk 4
# for their Pubkey/Account/Instruction types. Those rows are Mollusk's, not LiteSVM's.
cargo add mollusk-svm@0.15.1 --dev
# SPL Token's cache entry + account row for Mollusk, exactly as m06-l1 used it.
cargo add mollusk-svm-programs-token@0.15.1 --dev
cargo add solana-sdk@4 --dev
# And the two rows that keep Mollusk's graph on wincode 0.5, straight out of m06-l1.
# Quote them: the shell would read < and > as redirects.
cargo add 'solana-short-vec@>=3.2.2, <3.3' --dev
cargo add 'solana-signature@>=3.4.1, <3.5' --dev

cargo build-sbf                      # both harnesses load the .so; build before you measure
export SBF_OUT_DIR=$PWD/target/deploy   # Mollusk reads this, not target/deploy (m06-l1)
cargo test -p floor-registry         # runs BOTH suites
```

> Nota de pin, e é a razão pela qual a linha `solana-address` no topo desta lição — e em todos os quatro degraus que ela puxa — lê `">=2.6.1, <2.7"` em vez de um `=2.6.0` exato. O escopo é o workspace, não este crate. O `cargo` resolve um `solana-address` só para todo membro de uma vez, então um único degrau ainda segurando `=2.6.0` reprova o resolve inteiro com `all possible versions conflict`, e o registry nem chega a compilar. A pilha de SVM do Mollusk alcança `solana-address ^2.6.1`; um pin exato em qualquer lugar do workspace recusa isso. As duas linhas de range abaixo são o mesmo perigo um nível mais para baixo, e elas se comportam de um jeito diferente: elas são dev-dependencies deste crate, então elas moldam o lock sem todo irmão ter que declarar elas — `solana-short-vec 3.3.0` e `solana-signature 3.5.0` foram para `wincode 0.6` e continuam satisfazendo `solana-message`, então sem elas o resolve dá certo e o *build* morre. As três linhas dizem uma coisa só: segure este grafo no `wincode 0.5`, a linha que o rc.1 quer. Nenhuma delas sobrevive ao V2 cruzar para o 0.6, e nenhuma vai antes disso.

Escreva dois tipos de teste nesse crate, porque as duas ferramentas respondem perguntas diferentes e o passo 7 precisa da segunda. Os testes de LiteSVM são a suíte de comportamento: um por borda, `record_play`, `route_credit`, `settle_prize`, `quote_swap`, cada um afirmando que a CPI aterrissou e que o estado do programa chamado se moveu; os imports deles pegam carona em `anchor_lang` e `anchor_v2_testing` e não alcançam nada além, a mesma forma que todo teste de LiteSVM neste curso usou. Os testes de Mollusk são a suíte de medição, a mesma forma que você construiu no módulo 6: uma instrução, uma fixture, `process_instruction`, e um `println!` de `compute_units_consumed`, importando `Account`, `Instruction` e `Pubkey` de `solana_sdk`. Um porém específico do capstone que a forma do módulo 6 não tinha: uma SVM minificada roda só os programas que você registra, e o `settle_prize` invoca uma cadeia inteira deles. Registre cada degrau local na bancada de teste — `mollusk.add_program(&quarter_prize::ID, "quarter_prize")`, e o mesmo para o vault — o que carrega cada `.so` por nome a partir do `SBF_OUT_DIR` que você exportou acima; os ids saem dos próprios módulos gerados do registry (`use floor_registry::quarter_prize;` no teste — não existe um extern crate `quarter_prize` de onde importar). Registre o SPL Token pelo crate companheiro dele, `mollusk_svm_programs_token::token::add_program(&mut mollusk)`. Depois dê a cada programa que recebe CPI a linha de conta dele também: o `mollusk_svm::program::create_program_account_loader_v3(&quarter_prize::ID)` constrói a conta executável de propriedade do loader que o runtime exige, e a linha do token program é `token::keyed_account()`. As duas metades falham de duas formas diferentes, as duas valendo reconhecer de cara: uma *entrada de cache* faltando deixa a sua instrução de fora rodar e depois mata a CPI com `Unsupported program id`, enquanto uma *linha de conta* faltando nem chega ao seu programa — a própria bancada de teste dá panic com `[MOLLUSK]: An account required by the instruction was not provided`. Mantenha as duas suítes em arquivos de teste separados: elas falam duas pilhas de SVM diferentes, e um arquivo que mistura os tipos delas não compila — arquivos separados são o requisito inteiro, e as duas suítes então ficam num crate só, felizes. Você precisa de pelo menos um teste de Mollusk para o `settle_prize`, porque esse inteiro impresso é o número "antes" que o passo 7 pede para você registrar.

Verde aqui quer dizer que cada borda funciona sozinha. Isso é necessário e não suficiente, que é a razão inteira de o passo 8 existir.

**Passo 5. Um caso de fuzz verde.** O Anchor V2 traz uma bancada de fuzzing junto (o Crucible). Aponte ela para o registry e deixe ela jogar entradas geradas em uma instrução até você ter um caso que sobreviva:

```bash
anchor fuzz init floor-registry              # scaffold a Crucible target for the registry
anchor fuzz run floor-registry --release     # run it; --stateful for sequences of instructions
```

Você não está atrás de cobertura completa num capstone, só comprovando que a bancada de teste roda contra a sua composição e que um alvo volta verde.

**Passo 6. O checklist de segurança.** Percorra o checklist por instrução que este curso vem construindo: toda conta validada para owner, signer e PDA; aritmética checada em todo lugar; nenhum `unwrap()` em código de programa; alvos de CPI fixados no `Program<T>` certo; e a específica de composição, a classe de substituição de conta que sobrevive a todo framework. O módulo 7 te mostrou quais classes de vulnerabilidade o V2 mata em tempo de compilação; substituição de conta através de uma CPI é a classe que não morre sozinha, então confirme que cada conta de degrau é a que você queria, por tipo e por seed.

**Passo 7. Perfil de CU mais uma otimização.** Perfile a borda mais pesada, o `settle_prize`, porque três saltos queimam o máximo. Leia as unidades de compute do teste de Mollusk que você escreveu no passo 4 e registre o seu número — para escala, o próprio aparato de verificação do curso mede a cadeia registry-para-escrow-para-vault dele com handlers de stub na casa dos milhares de CU, e os seus handlers de verdade aterrissam mais alto; o número é seu, a forma milhares-e-não-dezenas é a checagem de sanidade. Depois refaça o build, faça uma mudança medida, e registre de novo. Uma mudança concreta que compensa: se o seu handler lê uma conta antes e depois de uma CPI, e a segunda leitura só precisa de um valor de lamport ou de byte em vez da view tipada, tire a leitura tipada redundante. Não fabrique o ganho; meça ele. A regra é a mesma que este curso segura desde o módulo 1: reporte o número que você viu, não o número que você esperava.

![Um pipeline de oito estágios indo do anchor build até a suíte de unidade, o fuzz, o endurecimento, o perfil de CU, o localnet do Surfpool, o deploy na devnet, e o verify-from-repo local.](assets/v05-flowchart.webp)

**Passo 8. A rodada de integração em localnet no Surfpool.** Este é o passo que pega o que todo teste de unidade acima não consegue. Os seus testes de LiteSVM comprovam que cada degrau funciona sozinho. Eles nunca levantam o salão inteiro junto, então uma CPI que passa a conta errada, ou uma seed que deriva um vault no teste e outro no salão, passa batido pelos testes de unidade e falha só quando os programas de fato compõem. O `anchor test` no V2 sobe um localnet do Surfpool por padrão, faz deploy do workspace inteiro, e roda os seus testes contra o salão de cinco programas inteiro rodando junto, antes de um único byte tocar a devnet. O Surfpool é um binário separado que o `anchor test` dirige; se você fez o Digital Assets, este é o mesmo Surfpool que você dirige desde o módulo 2 dele — lá ele forkava estado de mainnet embaixo dos seus testes, aqui ele levanta o seu salão de cinco programas em localnet. Instale ele uma vez para que o validador default esteja no seu PATH:

```bash
# Surfpool's documented installer. The repo moved from txtx to the Solana Foundation
# (the old URL redirects); latest release v1.5.0, checked 2026-08-22. `anchor test`
# needs surfpool >= 1.1.2.
curl -sL https://run.surfpool.run/ | bash
surfpool --version
anchor test                         # V2 default validator is surfpool; runs the floor together
# expect the registry suite line:
#   floor-registry ... passing
```

Torne o modo de falha concreto, porque é ele que morde. Digamos que o `settle_prize` do registry deriva o vault do escrow de `[b"vault", escrow.key()]` mas o seu helper de teste criou o vault do escrow de `[b"vault", operator.key()]`. Todo teste de unidade passa: o teste do registry monta as contas próprias dele e nunca cruza a emenda, o teste do escrow monta as dele também. Aí o salão roda junto, o registry entrega ao escrow um endereço de vault que o escrow não reconhece como sendo dele, e a CPI de liberação falha numa conta pela qual ela não consegue assinar. Esse bug não tem casa em teste nenhum de um programa só. Ele mora inteiramente na emenda, e a rodada em localnet é o único passo antes da devnet que levanta os dois programas nas mesmas contas ao mesmo tempo. Se falhas entre programas existem, elas aparecem aqui, na sua máquina, de graça. É esse o ponto de uma rodada de integração em localnet e é por isso que pular ela para "só fazer deploy e ver" é o atalho mais caro desta lista. Vale saber que este pipeline não é três ferramentas que alguém parafusou juntas. O memorando de unificação do Anchor do Jacob Creech (discussão #3742) nomeou ele de antemão: "I expect Anchor V2 to unify tools around using Litesvm, using the solana-verify standard, potentially surfpool." LiteSVM para a suíte de unidade, Surfpool para a rodada de integração, `solana-verify` para a prova. O seu capstone é essa frase, executada.

**Passo 9. Faça deploy na devnet.** Aponte o `Anchor.toml` para a devnet, financie a carteira, e faça deploy do registry junto com os degraus que ele chama:

```bash
solana config set --url devnet
solana airdrop 2                    # devnet SOL for the deploy
# One callback before you deploy: m08-l2 handed the swap's upgrade authority to
# /tmp/new-authority.json as a rehearsal. `anchor deploy` upgrades the whole
# workspace signed by your workspace wallet, which is no longer the swap's
# authority, so take it back first (and if a reboot already wiped /tmp, that
# program is frozen at its current bytes and you deploy the rest without it).
# Both sides are keypair FILES, which is m08-l2's own rule: a bare pubkey needs
# --skip-new-upgrade-authority-signer-check, and that is not a flag to rehearse.
solana program set-upgrade-authority <SWAP_PROGRAM_ID> -u devnet \
  --upgrade-authority /tmp/new-authority.json \
  --new-upgrade-authority ~/.config/solana/id.json
anchor deploy                       # deploys the workspace to devnet
# expect, per program:
#   Deploy success
#   Program Id: <FLOOR_REGISTRY_PROGRAM_ID>
```

Anote esse program id. O passo 10 precisa dele duas vezes, e o livro-razão de conclusão pede ele como evidência. Se o deploy falhar por falta de fundos, faça airdrop de novo; a devnet limita um airdrop único bem abaixo do que cinco programas custam para fazer deploy numa passada só.

**Passo 10. Verifique a partir do repo, localmente, contra o programa na devnet.** Esta é a trava do capstone. O `solana-verify` refaz o build do seu programa a partir do código-fonte dentro de uma imagem Docker fixada para que o bytecode seja determinístico, depois compara esse hash com o programa que recebeu deploy on-chain. Instale ele, faça build, faça deploy do artefato verificável, e verifique contra o seu programa na devnet:

```bash
cargo install solana-verify --locked   # v0.5.1 (solana-foundation/solana-verifiable-build; the old Ellipsis-Labs URL redirects); re-check the latest release
solana-verify build --library-name floor_registry
solana-verify get-executable-hash target/deploy/floor_registry.so
# solana-verify build just overwrote target/deploy with the deterministic artifact.
# Step 9's anchor deploy shipped a non-deterministic build, so redeploy NOW — skip
# this and the two hashes below will not match:
solana program deploy target/deploy/floor_registry.so \
  --program-id target/deploy/floor_registry-keypair.json
# then compare against the on-chain program:
solana-verify get-program-hash -u devnet <FLOOR_REGISTRY_PROGRAM_ID>
solana-verify verify-from-repo -u devnet \
  --program-id <FLOOR_REGISTRY_PROGRAM_ID> \
  --mount-path programs/floor-registry \
  --library-name floor_registry \
  https://github.com/<you>/quarter-vault
```

Quando os dois hashes casam, você comprovou que o seu código-fonte público reproduz exatamente o bytecode rodando na devnet. Isso é uma prova de verdade, e vale ser preciso sobre o que ela é e o que ela não é.

![Uma linha do tempo de duas trilhas separando a prova local de reprodutibilidade contra a devnet dos passos de distribuição e de autoridade só de mainnet, que são demonstrados mas nunca rodados aqui.](assets/v06-timeline.webp)

O job remoto da OtterSec (a flag `--remote`) submete o seu build para um registro público, e verificação remota só roda contra a mainnet. O Squads v4 executando um upgrade sob um multisig é o fluxo de autoridade para um lançamento de verdade. Os dois são demonstrados neste curso e rotulados como só de mainnet, porque estão além do cluster deste curso. Nenhum dos dois é a *prova* de verificação. A prova é o rebuild local casando com o hash on-chain, e você acabou de rodar ela contra a devnet. Reprodutibilidade é reprodutibilidade em qualquer cluster para o qual você apontar ela.

Seja preciso sobre o que um hash que casa compra e não compra para você, porque é aqui que as pessoas leem demais no check verde. Um build verificado comprova uma coisa exatamente: o bytecode rodando on-chain foi produzido pelo código-fonte naquele commit, byte a byte, então ninguém enfiou um programa diferente atrás do endereço que você auditou. É essa a propriedade que faz uma auditoria on-chain significar alguma coisa, e ela não é pequena. Ela é também estritamente uma afirmação sobre *procedência*, não sobre *corretude*. Um build verificado de um programa com bug é um bug fielmente reproduzido. Verificação diz para os seus usuários "o código que você consegue ler é o código que roda". Ela não diz para eles que o código está certo; é para isso que servem os seus testes, o seu caso de fuzz, o seu checklist de segurança, e uma auditoria de verdade. Entregue a escada inteira, não só o último degrau, e o hash verde quer dizer o que as pessoas acham que ele quer dizer.

## Challenge: produza o salão e leve ele pelo ciclo de vida

Nenhum scaffold novo. A trava é a coisa inteira, montada por você.

Construa as três instruções restantes do registry (`route_credit`, `settle_prize`, `quote_swap`) usando a gramática do passo 2, depois rode o salão por cada estágio. Aceite como pronto quando tudo o que segue valer:

![Uma tabela de livro-razão de conclusão em duas colunas listando cada estágio do capstone e o sinal concreto de aprovação dele, da suíte de unidade verde até a rodada em localnet no Surfpool, o deploy na devnet, e o verify-from-repo que casa.](assets/v07-comparison.webp)

Você vai saber que conseguiu quando três coisas forem verdade ao mesmo tempo: o `anchor test` imprime `floor-registry ... passing`, o programa está vivo num endereço de devnet que você consegue procurar, e o `solana-verify verify-from-repo` contra esse endereço casa com o seu build local. Se a rodada em localnet falhar mas todo teste de unidade tiver passado, não corra para a devnet. A falha é um bug de composição, que é exatamente o que o passo 8 existe para pegar, e é mais barato consertar na sua máquina do que depurar através de um cluster. Se o verify não casar, o seu artefato entregue e o seu código-fonte se descolaram; refaça o build com o `solana-verify build`, faça deploy de novo desse `.so` exato, e verifique de novo.

É essa a escada, de cima a baixo. Você construiu um counter e sentiu o imposto de desserialização sumir. Você deu custódia a ele, depois uma condição, depois um preço. Você endureceu ele, fuzzou ele, perfilou ele, e entregou ele. E agora você compôs tudo isso num programa só que roda o salão inteiro e comprovou, a partir do seu próprio código-fonte, que a coisa na devnet é a coisa que você escreveu. Vale sentar com isso por um segundo. Cinco programas, quatro dos quais você escreveu a partir de um arquivo em branco, compostos por um quinto, comprovados byte a byte contra o código-fonte que você pode publicar. Isso é a forma de um deploy de verdade, não um artefato de tutorial.

O salão roda, verificado, na devnet. Resta uma pergunta, e é a que decide se qualquer coisa disso importa para o código que você já tem: você deveria mover uma base de código de verdade para o V2 hoje? O módulo final mapeia os dois deltas de versão a partir de fontes primárias e porta um programa 0.31/1.0 de verdade até um V2 que compila e é testado. Você comprovou que consegue construir V2 do zero. Em seguida você comprova que consegue trazer o mundo antigo com você.
