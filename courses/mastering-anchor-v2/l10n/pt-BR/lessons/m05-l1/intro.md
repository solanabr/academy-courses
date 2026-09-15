# token_interface and transfer_checked: mova o vault e o escrow para tokens de verdade

Na lição passada você construiu o prize-escrow, o R3. Ele nunca segura o prêmio em si. Ele estaciona o prêmio numa instância de quarter-vault (R2) por uma chamada entre programas, registra quem pode fazer claim e o score que precisa bater, e só solta quando a condição vale e o jogador certo pede. Os dois programas funcionam de ponta a ponta. E os dois fazem a custódia de lamports nativos.

O que é justamente o problema, porque ninguém num fliperama paga em SOL cru. Os jogadores têm tokens de fliperama, mints SPL de verdade, e a economia inteira que você está ligando roda em cima deles. Então aqui está a pergunta que se sente na pele, a que decide se as duas últimas lições foram um aquecimento ou a coisa de verdade: sair de lamports para tokens de verdade significa reescrever tudo, ou muda um conjunto pequeno e contável de linhas enquanto o esqueleto de PDA e CPI fica exatamente como você construiu?

Antes de ler a minha resposta, vá buscar a sua. Abra o vault do R2 e conte as linhas que de fato pertencem à custódia:

```bash
# In your R2 quarter-vault program, list every line that truly touches custody:
grep -n 'b"sol"\|sol_vault\|sol_bump\|lamports\|system_program' programs/quarter-vault/src/lib.rs
```

Tudo que voltar pertence a uma camada: o PDA de custódia `[b"sol", owner]`, o `sol_bump` armazenado que assina por ele, e a transferência do System Program que move os lamports. O que o grep *não* imprime é a metade mais interessante: as seeds `[b"vault", authority]` do próprio PDA de state, o `bump` armazenado dele, o pin de authority `address =`, o `checked_sub` no livro-razão. Esses são agnósticos de custódia. As linhas que voltaram são a lição inteira, e a gente vai trocar elas.

## Resumo

Você vai dar upgrade em dois programas lamport que funcionam para a custódia em token SPL: o quarter-vault (R2) e o prize-escrow (R3). As formas das instruções continuam as mesmas. `deposit`, `withdraw`, `release` no vault; `reserve` e `redeem` no escrow. O que muda é a camada de custódia: o PDA separado que segurava os lamports se aposenta, a transferência do System Program vira um `transfer_checked` assinado por PDA, e as accounts ganham um mint mais constraints de conta de token. O PDA de state e o bump armazenado dele, a checagem `address = vault.owner`, o `checked_sub` no livro-razão e a trava de condição do escrow não se movem.

A ajuda recua do jeito que recua em toda lição de build. Eu conduzo o upgrade do vault todo trabalhado, `withdraw` incluído, porque essa forma é a que eu quero nos seus dedos. Você termina sozinho a release do `redeem` do escrow: a fiação de mint-e-decimais para dentro da CPI do vault é a única lacuna que eu deixo. Depois você re-roda as duas checagens intermediárias do zero em SPL, solo. Você termina quando um `transfer_checked` assinado por PDA move um saldo de token sob a assinatura do vault e o escrow libera só na condição certa, os dois verdes.

Uma fronteira logo de saída, porque é fácil atravessar ela sem perceber. Esta lição é tokens *da cadeira do framework*: como o seu programa move eles. O que o Token-2022 de fato muda num mint, as taxas e os hooks e os saldos confidenciais, é material do curso de Digital Assets, e este curso revisita só a borda disso vista da cadeira do programa, na m05-l3. Aqui a gente se importa com exatamente uma coisa: o mesmo caminho de código servindo o SPL Token clássico e o Token-2022 sem você deixar nenhum dos dois hardcoded.

## O que a custódia de fato custa quando ela vira SPL

A forma da resposta primeiro; os artefatos vêm depois.

Custódia é uma camada, não o programa. A identidade do seu vault, o PDA derivado de `[b"vault", authority]` com o bump armazenado dele, é quem o vault *é*. Custódia é o que o vault *segura*. Na versão lamport, segurar significava que os lamports ficavam num segundo PDA, de propriedade do system, e se moviam por uma transferência do System Program assinada pelas seeds daquele PDA. Na versão SPL, segurar significa uma conta de token cuja authority é o próprio PDA de state, e mover significa uma CPI de `transfer_checked` para o token program, assinada pelas seeds do PDA de state. Identidade intocada. A conta de custódia e o verbo são o que muda.

É essa a espinha do upgrade inteiro, e vale ver ela como um diff antes de a gente tocar numa única linha.

![Duas colunas: as seeds do PDA, o bump armazenado, a checagem de authority, o débito e as formas das instruções ficam idênticos, enquanto só a camada de custódia troca para uma conta de token e transfer_checked.](assets/v01-comparison.png)

Leia aquela coluna da direita de novo. Quatro itens. É isso que uma migração de custódia custa. Tudo à esquerda é o trabalho que você já fez no R2 e no R3, e ele sobrevive à mudança intacto. É exatamente por isso que o caminho de upgrade ensina tokens melhor do que um build novo ensinaria. Um build novo esconde a emenda, porque tudo é novo de uma vez. O upgrade isola a emenda, e a emenda é a lição.

### A cadeira de tokens: Interface, InterfaceAccount e um caminho de código

O Anchor V2 dá ao seu programa uma cadeira específica para tokens, e ela mora em `token_interface`. Dois tipos carregam ela.

A conta de programa é `Interface<'static, TokenInterface>`. Compare isso com `Program<Token>`, que é o que você pegaria por reflexo. O `Program<Token>` fixa a sua instrução em exatamente um programa, o SPL Token program clássico no endereço único dele. O `Interface<'static, TokenInterface>` aceita ou o Token program clássico ou o Token-2022 program. O mesmo lugar, dois inquilinos válidos.

Os mints e as contas de token são `InterfaceAccount<Mint>` e `InterfaceAccount<TokenAccount>`. E aqui está a coisa que derruba todo mundo que aprendeu Anchor um ano atrás: o `InterfaceAccount<T>` no V2 é literalmente um alias de `Account<T>`. Não é um primo, não é um wrapper. É o mesmo tipo.

O que quer dizer que o wrapper não é onde o comportamento mora, e esta é a frase para guardar: a diferença entre uma conta que aceita os dois token programs e uma que não aceita é *de qual módulo o `T` veio*. O `anchor_spl::token::TokenAccount` carrega o id do Token program clássico como owner esperado dele. O `anchor_spl::token_interface::TokenAccount` aceita qualquer um dos dois. Escreva `Account<TokenAccount>` com um import de `token_interface` e você ganha o comportamento de interface; escreva `InterfaceAccount<TokenAccount>` com um import de `token` e você ganha o comportamento só-clássico, alias ou não. A convenção abaixo é emparelhar `InterfaceAccount` com imports de `token_interface`, porque ler o wrapper é mais rápido que rastrear um import, mas é uma convenção, não o mecanismo.

Por que um *alias*, e não o tipo separado e mais pesado que ele era antes? Essa vale uma pausa, porque a resposta explica a direção inteira do V2.

![Uma linha do tempo de três paradas: o Account<T> é chamado de caminho lento do Anchor, a issue #4390 defende zero-copy por padrão, e o V2 faz do InterfaceAccount<T> um alias do Account<T> agora rápido.](assets/v02-timeline.png)

Então quando você escreve `InterfaceAccount<TokenAccount>` no V2, você ganha o `Account<TokenAccount>` zero-copy de hoje mais a propriedade de que ele vai validar um mint ou uma conta de token de propriedade de *qualquer um* dos dois token programs. Você não paga nada extra pela capacidade de interface. É o design se pagando: o caminho rápido e o caminho compatível agora são o mesmo caminho. É a mesma alavanca por trás da melhora média de 8.8x em CU que o Anchor reporta na própria bancada de benchmark dele, número que o PR #4914 revisou para baixo dos 9.9x em 2026-08-13. Datado e atribuído, não medido aqui; o módulo 6 é onde você mede o seu.

O que levanta a pergunta óbvia de quem migra: se `InterfaceAccount<T>` é só `Account<T>`, por que não continuar escrevendo `Account<TokenAccount>`? Porque no caminho do reflexo o `Account<TokenAccount>` vem com um `use anchor_spl::token::TokenAccount`, e esse `T` deixa o Token program clássico hardcoded como owner. No momento em que um mint Token-2022 aparece, essa conta falha no carregamento. Emparelhar o wrapper `InterfaceAccount` com o módulo `token_interface` é o que fica agnóstico de owner nos dois programas. As combinações não são intercambiáveis, e as que você vai encontrar de verdade valem ser vistas lado a lado.

![Duas tipagens de conta, uma só-clássica e uma que aceita os dois token programs, mais a escolha correspondente de conta de programa entre Program<Token> e Interface<'static, TokenInterface>.](assets/v03-comparison.png)

Existe uma cilada de verdade escondida naquele primeiro card, sutil o bastante para queimar uma tarde. Se você tipa uma conta como `Account<T>` e espera um erro de owner *customizado*, você não vai ganhar um. As checagens de owner e de discriminator rodam dentro do passo de carregamento da conta, que acontece antes de qualquer um dos seus hooks de constraint. Então um descasamento de owner aparece como um `IllegalOwner` genérico, e a sua mensagem customizada bem escrita nunca dispara. Se você genuinamente precisa de um erro de owner customizado, você desce para `UncheckedAccount` e afirma o owner você mesmo no handler. Guarde essa no bolso.

### Os constraints que colocam a conta de token do vault

Duas famílias de constraint aparecem nas contas de token do lab, e vale glosar elas uma vez para que leiam como intenção em vez de encantamento quando você encontrar elas.

A família `associated_token::` diz "esta conta é a Associated Token Account deste mint e deste owner". Uma ATA é a única conta de token canônica que um dado owner tem para um dado mint, num endereço determinístico derivado do owner, do mint e do token program. Quando você escreve `associated_token::mint = mint`, `associated_token::authority = vault` e `associated_token::token_program = token_program` num campo, o Anchor deriva esse endereço canônico e confere se a conta que te entregaram fica nele. Emparelhe os três com `init` e o Anchor *cria* a ATA se ela ainda não existir, pagando o rent a partir do `payer`; emparelhe eles com `mut` sozinho e o Anchor só valida uma ATA que ele espera já estar ali. É essa a diferença inteira entre o `initialize` do vault (que inicializa a ATA do vault) e o `deposit` dele (que valida uma que já existe).

A família `mint::`, por outro lado, restringe o mint em si: `mint::decimals`, `mint::authority`, `mint::freeze_authority`. Você não vai precisar deles neste upgrade, porque você está fazendo a custódia de um mint de token de fliperama *existente*, não criando um. Mas eles têm a mesma forma, e saber que a família existe te impede de reinventar uma checagem de decimais na mão mais tarde. O motivo pelo qual a linha `associated_token::token_program = token_program` importa é a história de um-caminho-de-código de antes: porque `token_program` é um `Interface`, o endereço de ATA derivado é computado contra o token program que de fato é dono do mint, então o mesmo constraint resolve certo para um mint clássico e para um mint Token-2022. Deixe o programa clássico hardcoded ali e você derivaria em silêncio o endereço errado no dia em que um mint Token-2022 chegar.

![Duas famílias de constraint lado a lado: associated_token coloca e deriva uma conta de token para um mint e um owner, enquanto mint restringe os decimais e as authorities de um mint existente.](assets/v04-comparison.png)

### transfer_checked: a primitiva que carrega o mint

A versão lamport movia valor com uma transferência do System Program. A versão SPL move valor com `transfer_checked`, e o nome está fazendo trabalho de verdade. Um `transfer` simples recebe um valor e confia nele. O `transfer_checked` recebe o valor *mais a conta de mint mais os decimais do mint*, e ele se recusa a rodar se os decimais que você afirma discordarem dos decimais que estão no mint. É o token program conferindo duas vezes que você e ele concordam sobre o que uma "unidade" significa antes de mover qualquer coisa.

Esta é a única linha que a maioria de quem migra erra primeiro, porque a memória muscular pega o `transfer` simples que ela usava dois anos atrás, e no V2 esse `transfer` simples está depreciado e não vai compilar do jeito que ela lembra. O `transfer_checked` é a primitiva agora. Diga uma vez, em voz alta: o mint e os decimais dele viajam com toda transferência.

Vale perguntar por que o token program se incomoda, já que o valor já é um inteiro cru de unidades base e a transferência moveria exatamente essa quantidade de qualquer forma. A resposta é a classe de bug que a checagem fecha. Um valor de token não tem significado sem os decimais dele: `1_000_000` é um token inteiro a seis decimais e um milésimo de token a nove. Um cliente que computa um valor contra os decimais errados, ou um programa que deixa um valor de decimais hardcoded que depois se afasta do mint, move a quantidade errada de valor enquanto o inteiro cru parece perfeitamente razoável. O `transfer` simples não consegue pegar isso, porque ele nunca vê o mint. O `transfer_checked` vê os dois, e ele aborta antes de mover qualquer coisa se os decimais que você afirma discordarem dos decimais registrados na conta de mint. É uma checagem de consistência barata parada exatamente onde um erro silencioso e caro morava.

Agora o trade-off, porque é a parte honesta desta cadeira. O `InterfaceAccount<T>` te compra um caminho de código único nos dois token programs, e isso é genuinamente valioso. Mas você paga por ele em duas moedas. Primeira, o `transfer_checked` força o mint e os decimais para dentro de toda transferência, então o mint tem de estar *presente* em instruções que antes não precisavam dele. Segunda, essa conta de mint extra é uma conta a mais por instrução, uma coisa a mais para passar do cliente, uma linha a mais na struct de accounts. Nenhum dos dois custos é grande. Os dois são reais. O jeito de sentir o tamanho deles é fazer o upgrade e contar, que é exatamente o que o lab faz.

## Lab: dê upgrade no quarter-vault para SPL

Passos numerados. Os interessantes carregam o porquê deles; os rotineiros vão secos. Os checkpoints te dizem com o que o sucesso se parece para você nunca ter de adivinhar se funcionou.

### 1. Fixe o toolchain do V2 RC

A sua máquina vem com o anchor-cli na linha 1.x. Este curso é V2, uma superfície de compilador diferente, e a CLI do V2 é consumida do git, não do `avm`. Você montou isso lá na m01-l2, então este é um re-pin, não uma instalação nova: o `avm install` não consegue buscar a tag do V2 porque nenhum Release do GitHub foi cortado para ela e o binário pré-compilado que ele baixa dá 404, então o canal documentado é um build de fonte fixado na tag `v2.0.0-rc.1`.

```bash
# The documented V2 channel. `avm install` 404s on the RC (see m01-l2); build from git.
# macOS, if the build trips on LTO: prefix with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version   # confirm you are on the V2 line, not your old 1.1.2
```

Freshness note (checada em 2026-08-22): o V2 é entregue como `2.0.0-rc.1` a partir do branch `anchor-next`, e o `avm list` não carrega nada acima de `1.1.2`, então o build de git é o único canal. Assim que o `anchor --version` imprimir a string exata do RC, fixe *essa* string no `Anchor.toml` sob `[toolchain]` e fixe o commit no Dockerfile de CI, porque APIs do V2 podem se mover entre commits. No macOS, prefixe a instalação com `CARGO_PROFILE_RELEASE_LTO=off` se o passo de link ficar sem memória. O Rust mínimo suportado pelo V2 é 1.89.0; se o `rustc --version` for mais antigo, rode `rustup update` antes de fazer o build.

Checkpoint: o `anchor --version` imprime a string do V2 RC, não `1.1.2`. Se ele ainda imprime a versão antiga, o seu shell está resolvendo um binário mais antigo antes no `PATH`; conserte isso antes de ir mais longe, porque todo erro de compilação depois deste ponto seria uma mentira.

### 2. Acrescente a dependência de token

O vault precisa da superfície SPL. Em `programs/quarter-vault/Cargo.toml`, acrescente `anchor-spl` do lado de `anchor-lang`, os dois na mesma versão do V2. Nada a configurar além disso: `token`, `token_interface`, `associated_token` e `token_2022` são todos módulos incondicionais do crate do V2.

```toml
[dependencies]
# The V2 crates are named plainly: the repo's lang-v2/ directory publishes `anchor-lang` and
# spl-v2/ publishes `anchor-spl` (its V1 directories are the ones suffixed -v1). There is no
# token-2022 feature to opt into — Token-2022 support IS token_interface.
anchor-lang = "2.0.0-rc.1"
anchor-spl  = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
# They are already in this file from m03-l1; keep them when you add the SPL rows above.
wincode = { version = "0.5", features = ["derive"] }
solana-address = ">=2.6.1, <2.7"   # the arcade-workspace row from m02-l1, unchanged
```

Freshness note: `anchor-lang` e `anchor-spl` se movem juntos na linha do V2, então fixe os dois na *mesma* string de versão e mude eles juntos. A armadilha aqui é a versão, não a fonte: os nomes dos crates são idênticos nas duas linhas, então `anchor-spl = "1"` — ou um `anchor-spl = "*"` simples que resolve para a linha 1.x — te entrega o crate V1 sob o nome V2, os tipos de handle de conta não vão bater, e você ganha erros de tipo exatamente na CPI. O `2.0.0-rc.1` explícito é o que te mantém na linha do V2, e porque o registry proíbe republicar uma versão, ela não pode se afastar do jeito que a ponta do branch `anchor-next` se afasta.

Checkpoint: o `cargo check` resolve os dois crates e o build falha só no seu próprio código, não no grafo de dependências. Se ele reclamar que `token_interface` não existe, você resolveu o `anchor-spl` 1.x — confira a string de versão, não uma lista de features.

### 3. Troque as accounts: o PDA lamport ganha uma conta de token

Este é o primeiro lugar em que o diff aparece. No R2, o valor morava num *segundo* PDA: o `sol_vault` de propriedade do System, semeado em `[b"sol", owner]`, segurando os lamports. Agora o valor mora numa conta de token cuja *authority* é o próprio PDA de state. O PDA de state `Vault` fica exatamente onde estava, seeds e bump inalterados; o `sol_vault` se aposenta, e um novo `vault_token_account` (uma ATA cuja authority é o PDA do vault) assume o trabalho de segurar.

![Um diff de struct de accounts: a conta do vault mantém as seeds e o bump dela, o PDA sol-vault separado é removido, e linhas de mint, de conta de token e de token program são acrescentadas.](assets/v05-annotated-code.png)

O **caminho de custódia** inteiro, do `initialize` ao `release`. Leia `withdraw` com atenção: é essa a checagem intermediária que você re-roda.

Uma tabela de destino antes de você colar, porque "o programa do vault" agora significa mais do que estes quatro handlers, e um colar-por-cima deletaria em silêncio o resto do módulo 3 sem ninguém dizer nada. A tese desta lição é que só a camada de custódia muda; aqui está essa tese explicitada artefato por artefato.

| artefato | de | o que acontece com ele |
|---|---|---|
| o PDA `Config` e a trava `address = config.authority` no `admin_set_credit` | m03-l2 | **Mantenha, inalterado.** Ela trava quem pode escrever no livro-razão, e o livro-razão não mudou de forma. |
| `close_vault` | m03-l2 | **Mantenha, e note o que ele agora implica.** O valor do vault não mora mais na conta que você está fechando. Feche o PDA de state enquanto a ATA dele ainda segura tokens e nada no programa vai mover eles de novo, porque todo caminho que assina pelo vault carrega o state que você acabou de fechar. Trave isso — recuse fechar um vault cujo `credit` não seja zero — ou drene primeiro. |
| `quarters::min_balance` e `require_funded` | m03-l3 | **Mantenha, inalterado.** O constraint lê `vault.credit`, que continua ali e continua um `u64`. |
| `set_credit` | m03-l3 | **Delete ele aqui.** A m03-l3 chamou ele de "exatamente o handler que você deletaria antes de entregar", e esta é a lição em que o vault começa a segurar valor de verdade. Um setter de crédito sem trava sentado ao lado de custódia de verdade contradiz o caminho de deposit que você está a ponto de escrever; o `deposit` é dono do livro-razão de agora em diante. |
| o PDA `sol_vault` e o campo `sol_bump` | m04-l1 | **Sumiu.** Esta é a única deleção genuína que a troca de custódia força: o valor se mudou para uma conta de token, então o PDA de lamports de propriedade do System e o bump que você armazenou para ele não têm mais trabalho nenhum. |

Só as duas últimas linhas são deleções, e só uma delas é obra da troca de custódia. É a tese se sustentando.

```rust
use anchor_lang::prelude::*;
use anchor_spl::{
    // `token` and `associated_token` are imported as MODULES, not just for their types: a
    // `token::mint = ...` or `associated_token::authority = ...` constraint expands to code that
    // names the module by path, so it has to be in scope or the derive fails to resolve.
    associated_token::{self, AssociatedToken},
    token,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
// InterfaceAccount comes from anchor_lang::prelude — the alias inside
// anchor_spl::token_interface is private and cannot be imported.

declare_id!("3pX5NKLru1UBDVckynWQxsgnJeUN3N1viy36Gk9TSn8d");

#[program]
pub mod quarter_vault {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        let authority = *ctx.accounts.authority.address();
        let vault = &mut ctx.accounts.vault;
        vault.owner = authority;
        vault.credit = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
        // Deposit is signed by the depositor, a real keypair. No PDA signing here.
        let decimals = ctx.accounts.mint.decimals();
        let accounts = TransferChecked {
            from: ctx.accounts.depositor_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.vault_token_account.cpi_handle_mut(),
            authority: ctx.accounts.depositor.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_add(amount).ok_or(VaultError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
        // Withdraw is signed by the vault PDA, with the SAME seeds and stored bump
        // the lamport version used. Only the verb changed.
        let decimals = ctx.accounts.mint.decimals();
        let authority_key = ctx.accounts.vault.owner;
        let bump = [ctx.accounts.vault.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", authority_key.as_ref(), &bump]];

        let accounts = TransferChecked {
            from: ctx.accounts.vault_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.authority_token_account.cpi_handle_mut(),
            authority: ctx.accounts.vault.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts)
            .with_signer(signer_seeds);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        // The books debit is byte-for-byte the lamport version: checked_sub, never `-`.
        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;
        Ok(())
    }

    pub fn release(ctx: &mut Context<Release>, amount: u64) -> Result<()> {
        // Release is the composition path: R3 (the escrow) is this vault's recorded
        // authority and signs the CPI, then the vault PDA signs the token move.
        let decimals = ctx.accounts.mint.decimals();
        let authority_key = ctx.accounts.vault.owner;
        let bump = [ctx.accounts.vault.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", authority_key.as_ref(), &bump]];

        let accounts = TransferChecked {
            from: ctx.accounts.vault_token_account.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.recipient_token_account.cpi_handle_mut(),
            authority: ctx.accounts.vault.cpi_handle(),
        };
        let cpi = CpiContext::new(ctx.accounts.token_program.address(), accounts)
            .with_signer(signer_seeds);
        token_interface::transfer_checked(cpi, amount, decimals)?;

        let vault = &mut ctx.accounts.vault;
        vault.credit = vault.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address,  // 32  the only key allowed to authorize a move
    pub credit: u64,     //  8  the books, still debited with checked_sub
    pub bump: u8,        //  1  stored canonical bump, never re-derived
    pub _pad: [u8; 7],   //  7  explicit Pod padding (32+8+1 -> 48)
}

#[error_code]
pub enum VaultError {
    #[msg("Arithmetic overflow on the vault books")]
    Overflow,
    #[msg("Withdraw exceeds the vault balance")]
    Underflow,
}

#[derive(Accounts)]
pub struct Initialize {
    // Carried over from m04-l3's role split: the vault's owner does not have to sign
    // or pay for its creation, because an escrow PDA can do neither. The seeds derive
    // from `authority`; the rent comes from `funder`.
    /// CHECK: seeds derive from this; creating a vault for an address is not an authority action
    pub authority: UncheckedAccount,
    #[account(mut)]
    pub funder: Signer,
    #[account(
        init,
        payer = funder,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", authority.address().as_ref()],
        bump
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        init,
        payer = funder,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
    pub associated_token_program: Program<AssociatedToken>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub depositor: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    pub depositor_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[derive(Accounts)]
pub struct Withdraw {
    // has_one is deprecated in V2. The replacement is address = parent.field:
    // the signer's address must equal the authority the vault stored at init.
    #[account(mut, address = vault.owner)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
    )]
    pub authority_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[derive(Accounts)]
pub struct Release {
    // The recorded authority must sign to authorize a release. In composition this
    // is the escrow PDA, which signs via its own seeds from R3.
    #[account(address = vault.owner)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    // NOT an ATA constraint: the recipient's owner is whoever the caller names, so
    // there is no owner to derive an address from. The token:: family constrains an
    // existing token account's fields instead of placing it: token::mint pins which
    // mint it holds, token::token_program pins which token program owns it. Reach
    // for token:: whenever you accept a token account you did not derive.
    #[account(
        mut,
        token::mint = mint,
        token::token_program = token_program,
    )]
    pub recipient_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}
```

Três linhas merecem um comentário. O cálculo de space é `Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE`, com o `_pad` explícito mantendo a struct num múltiplo limpo de 8 para que o `INIT_SPACE` derivado e o layout Pod concordem exatamente. O campo `owner` é `Address`, não `Pubkey`; o V2 renomeou o tipo de chave, e `.address()` é como você lê uma chave de um signer ou de uma account (ele substitui `.key()`). E a linha de authority do `withdraw`, `address = vault.owner`, é a substituição congelada de `has_one`: a mesma garantia, uma grafia nova.

Checkpoint: o `anchor build` compila o programa com as accounts novas. Ele ainda não vai fazer nada útil, mas deve estar verde.

### 4. Troque a transferência: a conta de custódia muda, a jogada de assinatura não

Este é o coração da coisa, e eu quero que você note com precisão o que se moveu e o que não. Na versão lamport você assinava com as seeds da própria conta de *custódia*, `[b"sol", owner, &[state.sol_bump]]`, porque os lamports ficavam naquele segundo PDA. Sob SPL não existe segundo PDA. Os tokens ficam numa ATA cuja authority é o PDA de state, então você assina com as seeds do PDA de state, `[b"vault", authority_key.as_ref(), &bump]`, e o `sol_bump` se aposenta junto com a conta que ele descrevia.

O array de seeds mudou. A *jogada* não, e é essa a parte transferível: reconstrua as seeds a partir de um bump que você armazenou no init, nunca re-derive ele, anexe `.with_signer`, e o runtime concede o privilégio de signer do PDA seja a chamada interna uma transferência do System Program ou um `transfer_checked` de token. As seeds são a assinatura, e esse mecanismo é agnóstico de custódia.

![Um antes-e-depois da CPI de withdraw: a jogada de assinatura fica inalterada, enquanto a transferência de lamports do System Program vira um transfer_checked carregando o mint e um argumento de decimais no final.](assets/v06-annotated-code.png)

Quatro coisas naquele bloco `AFTER` são específicas do V2 e valem ser nomeadas, porque a memória muscular de 1.x (a versão ainda na sua máquina) vai brigar com você em cada uma.

- `cpi_handle()` e `cpi_handle_mut()` substituem `.to_account_info()` quando você monta as accounts da CPI. O V2 roteia CPIs de token por um caminho de handle mais barato e com borrow rastreado; use `_mut` para as accounts cujos saldos mudam (`from`, `to`) e o simples para as só-leitura (`mint`, `authority`).
- O `CpiContext::new` recebe o programa como um address, `token_program.address()`, não como um `AccountInfo`. É a mesma mudança do V2 que você já encontrou na CPI do System Program.
- O handler recebe `&mut Context<T>`, não `Context<T>`. As assinaturas de handler do V2 são de contexto mutável por padrão.
- `.with_signer(signer_seeds)` é o que transforma uma CPI simples numa CPI assinada por PDA. Tire ele e esta chamada exata vira uma transferência sem assinatura que o runtime rejeita, porque o PDA do vault nunca autorizou ela.

![O programa reconstrói as signer seeds dele a partir do bump armazenado e chama transfer_checked; o token program verifica que essas seeds reproduzem o PDA do vault antes de mover o saldo.](assets/v07-diagram.png)

Mais uma coisa para ter na cabeça antes de rodar: a ordem em que o runtime faz tudo isso, porque saber a sequência é como você localiza uma falha na linha certa em vez de adivinhar.

![Um fluxograma vertical de oito passos do withdraw SPL, do carregamento de conta e do constraint address, passando pelo transfer_checked assinado por PDA, até o débito checked_sub, com avisos de falha por passo.](assets/v08-flowchart.png)

Checkpoint: o `anchor build` está verde, e você consegue ler `withdraw` de cima a baixo e nomear cada linha como identidade (inalterada) ou custódia (mudada), e apontar em qual dos oito passos ela mora.

### 5. Re-rode a checagem intermediária do R2 em SPL

A checagem do R2 sempre fez a mesma afirmação: o valor sai do vault *só* sob a assinatura do PDA do vault. Na versão lamport você afirmava sobre saldos de lamports. Agora você afirma sobre o saldo de token SPL. O mesmo teste, unidade nova. O template de teste default do V2 é LiteSVM em Rust, então a gente fica nele.

```bash
# The test surface. anchor-v2-testing is the V2 harness the scaffold generates
# against; it wraps LiteSVM and pins its own LiteSVM version, so add it rather
# than a bare litesvm that could resolve to a different one. Pin the TAG, not the
# branch: the tag carries litesvm 0.11.0 and the anchor-next tip has already moved
# to 0.13.1, and two SVM crates in one graph is the skew this pin exists to avoid.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1
# The fixture's two SPL crates, pinned like everything else here: these are the
# versions that resolve against the rc.1 pin set (verified 2026-09-01).
cargo add spl-token@9 spl-associated-token-account@8 --dev
```

O setup é mais do que um comentário, então aqui está ele inteiro. Ponha em `programs/quarter-vault/tests/spl_setup.rs`, do lado do crate do programa, que é o que faz o `.so` construído ficar alcançável:

```rust
// tests/spl_setup.rs - the fixture. Creates a 6-decimal mint, initializes the
// vault and its ATA, mints into the authority's ATA, and deposits 1.0 token.
// Imports ride anchor_lang and anchor_v2_testing and reach past neither: the
// instruction plumbing is anchor_lang's, the signing and SVM plumbing is the
// harness's, and nothing here names a solana crate the Cargo.toml never declared.
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction};

// Cargo compiles every tests/*.rs as its own crate root, so a sibling `mod spl_helpers`
// in the test file is not in scope here. Pull the helpers in by path instead; the test
// file then declares only `mod spl_setup;`.
#[path = "spl_helpers.rs"]
mod spl_helpers;

pub const DECIMALS: u8 = 6;
pub const ONE_TOKEN: u64 = 1_000_000;

pub struct Ctx {
    pub authority: Keypair,
    pub mint: Address,
    pub vault: Address,
    pub vault_ata: Address,
    pub authority_ata: Address,
}

// LiteSVM's own failure type is not among anchor_v2_testing's re-exports, and adding
// litesvm by name is exactly the skew this lesson's pin avoids, so the error is
// flattened to a String at the boundary. Callers only ever ask "did it fail, and why".
pub fn send(svm: &mut LiteSVM, payer: &Keypair, signers: &[&Keypair], ixs: &[Instruction])
    -> Result<(), String>
{
    let bh = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &bh);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx).map(|_| ()).map_err(|e| format!("{:?}", e.err))
}

pub fn token_balance(svm: &LiteSVM, ata: &Address) -> u64 {
    let acct = svm.get_account(ata).expect("token account exists");
    // SPL token account layout: amount is a little-endian u64 at offset 64.
    u64::from_le_bytes(acct.data[64..72].try_into().unwrap())
}

pub fn setup(svm: &mut LiteSVM) -> Ctx {
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();

    // The rent gets its own payer. Initialize takes authority AND funder — the
    // m04-l3 role split — and funder is the mut one, so handing one keypair to
    // both slots is exactly the aliasing the duplicate-mutable default rejects
    // at runtime (ConstraintDuplicateMutableAccount). Distinct keys, no friction.
    let funder = Keypair::new();
    svm.airdrop(&funder.pubkey(), 5_000_000_000).unwrap();

    // 1. the mint, created and minted with the spl-token helpers
    let mint = spl_helpers::create_mint(svm, &authority, DECIMALS);
    let authority_ata = spl_helpers::create_ata(svm, &authority, &mint, &authority.pubkey());
    spl_helpers::mint_to(svm, &authority, &mint, &authority_ata, ONE_TOKEN);

    // 2. the vault PDA and its ATA, created by the program's own initialize
    let (vault, _b) = Address::find_program_address(
        &[b"vault", authority.pubkey().as_ref()], &program_id);
    let vault_ata = spl_helpers::ata_address(&mint, &vault);
    let init = Instruction {
        program_id,
        accounts: quarter_vault::accounts::Initialize {
            authority: authority.pubkey(),
            funder: funder.pubkey(),
            vault,
            mint,
            vault_token_account: vault_ata,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::Initialize {}.data(),
    };
    send(svm, &funder, &[&funder], &[init]).unwrap();

    // 3. deposit 1.0 token into the vault ATA
    let dep = Instruction {
        program_id,
        accounts: quarter_vault::accounts::Deposit {
            depositor: authority.pubkey(),
            vault,
            mint,
            depositor_token_account: authority_ata,
            vault_token_account: vault_ata,
            token_program: spl_token::ID,
        }.to_account_metas(None),
        data: quarter_vault::instruction::Deposit { amount: ONE_TOKEN }.data(),
    };
    send(svm, &authority, &[&authority], &[dep]).unwrap();

    Ctx { authority, mint, vault, vault_ata, authority_ata }
}

pub fn send_withdraw(svm: &mut LiteSVM, ctx: &Ctx, amount: u64) -> Result<(), String> {
    let ix = Instruction {
        program_id: quarter_vault::ID,
        accounts: quarter_vault::accounts::Withdraw {
            authority: ctx.authority.pubkey(),
            vault: ctx.vault,
            mint: ctx.mint,
            authority_token_account: ctx.authority_ata,
            vault_token_account: ctx.vault_ata,
            token_program: spl_token::ID,
        }.to_account_metas(None),
        data: quarter_vault::instruction::Withdraw { amount }.data(),
    };
    send(svm, &ctx.authority, &[&ctx.authority], &[ix])
}
```

O `spl_helpers` ali é o wrapper fino sobre `spl_token` e `spl_associated_token_account` que monta as quatro instruções de setup (`create_mint`, `create_ata`, `mint_to`, `ata_address`). É código de cliente SPL comum, sem nada específico do V2 dentro, então ele é entregue junto com esta lição como [spl-helpers/spl_helpers.rs](spl-helpers/spl_helpers.rs); jogue ele dentro como `tests/spl_helpers.rs`.

Agora a checagem em si, em `programs/quarter-vault/tests/vault_spl_withdraw.rs`:

```rust
mod spl_setup;   // it pulls spl_helpers in itself, by #[path]

use anchor_v2_testing::svm;

#[test]
fn withdraw_moves_the_spl_balance_under_the_vault_signature() {
    let mut svm = svm();
    let ctx = spl_setup::setup(&mut svm); // 1.0 token (6 decimals) now sits in the vault ATA

    let before = spl_setup::token_balance(&svm, &ctx.vault_ata);
    assert_eq!(before, spl_setup::ONE_TOKEN, "vault holds 1.0 token before withdraw");

    // the vault PDA signs the move out; the human authority only authorizes it
    spl_setup::send_withdraw(&mut svm, &ctx, spl_setup::ONE_TOKEN)
        .expect("PDA-signed withdraw should succeed");

    let after = spl_setup::token_balance(&svm, &ctx.vault_ata);
    assert_eq!(after, 0, "the vault PDA signed the tokens out");

    // an over-withdraw is rejected, not panicked. Be precise about who refuses:
    // the token program fails the transfer CPI on insufficient funds before your
    // checked_sub ever runs; the ledger guard exists for drift the token balance
    // cannot see, and this assert only proves the refusal path is an Err.
    assert!(
        spl_setup::send_withdraw(&mut svm, &ctx, 1_000_000_000).is_err(),
        "over-withdraw must return an error, never a wrap or a panic"
    );
}
```

Rode:

```bash
anchor build && cargo test --test vault_spl_withdraw
```

Checkpoint: verde. O saldo de token saiu da ATA do vault, e a única coisa que autorizou a movimentação foram as seeds do PDA do vault. Se você comentar `.with_signer(signer_seeds)` no handler e re-rodar, este teste deve falhar com um erro de assinatura faltando. Essa falha é a prova de que o PDA é o que autoriza a movimentação. Descomente e siga em frente.

## Challenge: costure a release do escrow e depois re-rode as duas em SPL

Aqui o recuo entra. O vault está trabalhado. O escrow é seu, e é uma mudança menor do que você pode temer, por um motivo que é o retorno inteiro do design da lição passada.

O prize-escrow (R3), o programa `quarter-prize` que você construiu na lição passada, nunca fez a custódia do prêmio em si. Ele delegou a custódia a uma instância de vault do R2 e alcançou ela por uma CPI. Então quando a custódia do R2 virou SPL, a *política* do escrow não mudou nada: o `reserve` continua registrando o maker, o player, o vault, o valor e o score vencedor; o `redeem` continua conferindo `final_score >= escrow.winning_score` antes de qualquer coisa se mover; o chamador continua preso com `address = escrow.player`; o escrow continua fechando para o `maker` registrado para que o rent volte para o operador que pagou ele; o escrow continua assinando como o PDA dele mesmo com `[b"escrow", maker, player, bump]`, seeds e ordem inalteradas. A única coisa que mudou é o que o escrow *passa para baixo* para o vault: o mint e as contas de token agora viajam de carona nas CPIs de `deposit` e `release`.

É essa a frase para deixar assentar. Porque você construiu sobre um vault em vez de embutir a custódia, a migração para SPL nunca sai da borda de CPI do escrow: as accounts que ele costura na CPI do vault mudam, e o `reserve` segue uma única renomeação mecânica, a instrução de init do vault indo de `init_vault` para `initialize`. Política, condição, pin do chamador, assinatura de PDA: intocados.

![Os campos de política do escrow, a checagem de condição, o pin do chamador e a assinatura de PDA ficam inalterados; só a borda de CPI muda, agora carregando o mint e as contas de token.](assets/v09-diagram.png)

Aqui está o `redeem`, com a trava de condição e a assinatura do escrow já no lugar, e a fiação de accounts da CPI de release deixada como a sua lacuna. A resposta está impressa dentro do bloco marcado em vez de escondida, porque seis nomes de campo sem um compilador na sua frente é jogo de adivinhação, não exercício. Cubra com a mão, escreva a struct a partir do que o vault te ensinou, depois descubra e faça o diff. A lacuna que é genuinamente sua é a seção solo abaixo dele.

```rust
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::state::Escrow;
use crate::error::EscrowError;
// The vault's module is the one declare_program! generated back in m04-l3 —
// marker included. Re-harvest idls/quarter_vault.json after this lesson's SPL
// upgrade lands, because R2's interface just changed shape. And if this handler
// lives in an instructions submodule rather than lib.rs, spell the path from the
// crate root — `use crate::quarter_vault::…` — the macro generates the module at
// the root, and a submodule's bare `use quarter_vault::…` cannot see it.
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault;

pub fn handler(ctx: &mut Context<Redeem>, final_score: u64) -> Result<()> {
    // GUARD (unchanged from the lamport escrow): condition before payout.
    require!(
        final_score >= ctx.accounts.escrow.winning_score,
        EscrowError::ConditionNotMet
    );

    // Read escrow state out before any CpiHandle goes live (the borrow model).
    let amount = ctx.accounts.escrow.amount;
    let maker = ctx.accounts.escrow.maker;
    let player = ctx.accounts.escrow.player;
    let bump = ctx.accounts.escrow.bump;

    // The escrow signs as ITS OWN PDA to satisfy the vault's address = vault.owner.
    // Seeds unchanged from the lamport version, order included.
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        maker.as_ref(),
        player.as_ref(),
        &[bump],
    ]];

    // === YOUR LINE ===
    // Build vault_cpi::accounts::Release. In the lamport version it was
    //   { authority, vault, recipient, system_program }.
    // Now the vault moves TOKENS, so thread the SPL custody accounts through:
    // the mint, the vault's token account (from), the winner's token account (to),
    // and the token program. That mint account is how decimals reach transfer_checked.
    let accounts = vault_cpi::accounts::Release {
        authority: ctx.accounts.escrow.cpi_handle(),
        vault: ctx.accounts.vault.cpi_handle_mut(),
        mint: ctx.accounts.mint.cpi_handle(),
        vault_token_account: ctx.accounts.vault_token_account.cpi_handle_mut(),
        recipient_token_account: ctx.accounts.winner_token_account.cpi_handle_mut(),
        token_program: ctx.accounts.token_program.cpi_handle(),
    };
    // === END YOUR LINE ===

    let cpi = CpiContext::new(ctx.accounts.quarter_vault_program.address(), accounts)
        .with_signer(signer_seeds);
    vault_cpi::release(cpi, amount)?;
    Ok(())
}
```

Note o que *não* está ali: nenhum `transfer_checked`, nenhum argumento de decimais, nenhuma consulta de decimais do mint. Isso tudo mora no `release` do R2, que você trabalhou no lab. O trabalho do escrow é só entregar ao vault as accounts certas e assinar como a authority. Os decimais viajam dentro da conta `mint` que você costurou ali. É esse o dividendo da composição: a primitiva mora num lugar só, e o chamador só aponta para os tokens certos.

A struct de accounts `Redeem` ganha as mesmas quatro accounts de custódia, e mantém cada linha de política:

```rust
#[derive(Accounts)]
pub struct Redeem {
    // caller guard, unchanged: only the recorded player may redeem
    #[account(address = escrow.player)]
    pub player: Signer,
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<Escrow>,
    /// CHECK: close destination only, pinned to the maker the escrow recorded.
    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount,
    /// CHECK: address fixed by seeds; the quarter_vault program validates and signs it.
    #[account(
        mut,
        seeds = [b"vault", escrow.address().as_ref()],
        bump,
        seeds::program = quarter_vault_program.address(),
    )]
    pub vault: UncheckedAccount,
    pub mint: InterfaceAccount<Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = player,
        associated_token::token_program = token_program,
    )]
    pub winner_token_account: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}
```

Duas linhas ali vêm da versão lamport e são fáceis de perder numa reescrita, então nomeie elas: `close = maker` no escrow, e a vaga de `maker` que ele precisa como destino de close, presa com `address = escrow.maker`. Tire qualquer uma das duas e um escrow resgatado fica aberto com o rent dele encalhado — a linha de política sobrevive à troca de custódia exatamente como o resto.

Um constraint ali é novo, então pegue a versão de trinta segundos agora em vez de adivinhar. O `seeds::program = quarter_vault_program.address()` diz ao Anchor para derivar aquele PDA contra o id de *outro* programa em vez do seu. Você precisa dele porque o PDA do vault pertence a `quarter_vault`, não ao escrow: sem essa linha o Anchor derivaria `[b"vault"...]` sob o id de programa do escrow, ganharia um endereço diferente e rejeitaria a conta que você de fato queria. Toda vez que você restringe um PDA de que um programa que você está chamando é dono, `seeds::program` é a linha que aponta a derivação para o owner certo.

Depois a parte solo: as duas checagens intermediárias em SPL, do zero, sem nenhum handler trabalhado na sua frente.

1. **R2 em SPL, o withdraw.** Você rodou isso no lab. Faça de novo sem olhar: crie o mint, inicialize o vault e a ATA dele, faça deposit, faça withdraw, e afirme que o saldo de token do vault foi a zero *e* que tirar `.with_signer` faz ele falhar. Essa falha é o ponto da checagem.
2. **R3 em SPL, a release condicional.** Faça o operador dar `reserve` num prêmio dentro da instância de vault do escrow. Uma coisa para acertar antes de começar, porque é o passo que trava as pessoas: o vault do escrow ainda não existe, e o PDA do escrow não pode trazer ele à existência por conta própria. Não porque ele está quebrado — um PDA isento de aluguel segura lamports por definição, e o próprio vault lamport deste curso era um PDA cujo trabalho inteiro era segurar eles — mas por causa de quem pode gastar eles: o pagador do rent num `init` é debitado pelo System Program, e o System Program só debita contas de que ele é dono. O PDA do escrow carrega dados e pertence ao programa do escrow, então ele nunca pode ocupar a vaga de payer. É exatamente por isso que o `Initialize` ganhou um `funder` no passo 3 do lab. Então o `reserve` chama por CPI o `initialize` do vault primeiro — atenção ao nome: o upgrade para SPL renomeou a instrução de init do vault, do `init_vault` da versão lamport para `initialize`, então o módulo que a sua IDL re-colhida gera expõe `initialize`, e a fiação `vault_cpi::accounts::InitVault` da era m04-l3 pega o nome novo junto — com o **maker** como `funder` e o **PDA do escrow** como `authority`, depois chama por CPI o `deposit`, a mesma forma de duas chamadas da versão lamport. Depois chame `redeem` com um `final_score` abaixo de `winning_score` e afirme que ele dá erro com `ConditionNotMet` (nada se move). Depois dê `redeem` com um score vencedor e afirme que o prêmio aterrissou na conta de token do vencedor. Verde significa que a trava de condição se sustenta e que o PDA do escrow é a única coisa que consegue assinar a saída do prêmio.

Aceitação: o `withdraw` move o saldo de token só sob a assinatura do PDA do vault; o escrow libera só na condição certa, só para o player registrado; os dois verdes. E você consegue dizer, em uma frase, quais linhas mudaram em relação à versão lamport. Se a sua frase ficar mais longa que "a camada de custódia do vault, o mint e as contas de token que o escrow costura na CPI de release, e a renomeação de `init_vault` para `initialize` que o `reserve` segue", você mudou mais do que precisava.

## Onde isso te deixa

Aqui está o loop de feedback, sem enfeite. Se o `redeem` do seu escrow compilou na primeira tentativa, o vault te ensinou o padrão e você transferiu ele. Bom. Se não compilou, a falha foi quase com certeza uma de três, na ordem em que elas costumam morder: você pegou um `transfer` simples em vez de `transfer_checked` em algum lugar do vault, você esqueceu a conta de mint (então os decimais nunca chegaram ao `transfer_checked`), ou você deixou cair um `.with_signer` e o runtime rejeitou uma movimentação de PDA sem assinatura. Cada um desses é um erro de camada de custódia, não um erro de identidade, que é a lição aterrissando: o esqueleto que você construiu no R2 e no R3 estava correto, e trocar o que ele segura não quebrou quem ele é.

![Uma tabela de diagnóstico de três linhas mapeando um transfer simples, uma conta de mint faltando e uma signer seed que ficou de fora, cada um para a sua correção, os três sendo erros de custódia em vez de erros de identidade.](assets/v10-table.png)

É um marco de verdade, então nomeie ele pelo que custou. Você acabou de comprovar que o design de um programa que funciona sobrevive à custódia dele. O protótipo lamport não era descartável. Ele era o esqueleto, e o esqueleto se sustentou. E o escrow comprovou a segunda afirmação, mais afiada: porque ele delegou a custódia ao vault em vez de embutir ela, a migração para tokens nunca saiu da borda de CPI dele. Isso não é sorte. É isso que construir sobre um vault te compra, e é a mesma razão pela qual um protocolo de verdade separa política de custódia.

Dois programas SPL agora ficam no seu workspace: um vault que não cota nada e um escrow que não cota nada. Eles só movem tokens que já seguram. Só que os jogadores têm tokens de fliperama e querem tickets, e nenhum destes dois programas tem opinião sobre preço. Na lição que vem você constrói um terceiro programa, um pool que cota a própria taxa de câmbio a partir das reservas dele e troca tokens por tickets, segurando cada lado numa conta de token própria. Nem o vault nem o escrow entram nesse build, e isso é de propósito: a authority de uma reserva tem de ser o pool, então ela não pode ser também um vault. O que carrega para a frente é a forma, não o artefato. O `transfer_checked` assinado por PDA que você acabou de escrever é o payout do swap, e a release condicional que você escreveu no R3 é exatamente o fluxo de controle de que a trava de slippage do swap precisa. A mesma cadeira de PDA-e-CPI que você acabou de aprender, um trabalho novo: precificação.
