# O catálogo de constraints (e init_if_needed, in situ)

Na lição passada você derivou o PDA do quarter-vault e armazenou o bump canônico dele. Ele cria e re-deriva, o que é progresso de verdade. Mas olhe o que ele ainda faz: `init_vault` confia em toda conta que o chamador entrega. Nada checa que a conta que você *acha* que é o config é o config, que o signatário que paga por uma ação de admin é de fato o operador do fliperama, ou que uma conta qualquer passada onde deveria ir uma conta de token é de propriedade do programa que você supõe ser o dono dela. A macro deriva o endereço e aí o seu handler dá de ombros e confia no resto.

No v1, esse dar de ombros era o jeito mais comum de programas serem drenados. Alguém esquece uma checagem, um atacante passa a conta dele onde a sua era esperada, e o handler opera nela alegremente. Então, antes de qualquer teoria, sinta o buraco você mesmo. Abra o `programs/quarter-vault/src/lib.rs` da lição passada e acrescente este handler de admin ingênuo, do tipo que parece bom na revisão:

```rust
// Looks fine. Is exploitable. There is no check that `authority` is anyone in particular.
pub fn admin_set_credit(ctx: &mut Context<AdminSetCredit>, new_credit: u64) -> Result<()> {
    ctx.accounts.vault.credit = new_credit;
    Ok(())
}

#[derive(Accounts)]
pub struct AdminSetCredit {
    pub authority: Signer,          // ANY signer. That is the bug.
    #[account(mut, seeds = [b"vault", vault.owner.as_ref()], bump = vault.bump)]
    pub vault: Account<Vault>,
}
```

Rode `anchor build`. Compila limpo. E também deixa qualquer carteira na Solana pôr o credit de qualquer jogador em qualquer coisa, porque `Signer` comprova que alguém assinou, não *quem*. Segure esse pensamento. É nesta lição que você faz a macro `#[derive(Accounts)]` fazer o policiamento, para que uma conta ruim seja rejeitada antes de o seu código de handler rodar. E é nela que uma keyword, `init_if_needed`, reabre em silêncio uma porta que o framework passou anos trancando.

## Resumo

Aqui está a lição inteira como um índice de achados, cada linha uma conclusão sobre a qual você consegue agir:

- Constraints moram na **macro de derive**, então a validação roda *antes* do seu handler e aparece na IDL. O que a macro rejeita, o seu código nunca vê.
- Duas fases importam e a ordem delas é o truque inteiro: o **`load`** (checagens de owner + discriminator) roda *primeiro*, depois os **hooks de constraint** (`seeds`, `address`, `owner`, `constraint`, `close`) rodam *depois*. Uma checagem que você põe na fase errada nunca dispara, em silêncio.
- `address = parent.field` é o substituto do V2 para o agora depreciado **`has_one`**. Ele aceita *qualquer expressão*, não só uma chave armazenada, que é por que a keyword mais velha perdeu o emprego.
- A **cilada do erro de owner**: `#[account(owner = X @ MyErr)]` em `Account<T>` *não* vai fazer aparecer `MyErr`. Owner roda no `load`, antes do seu hook, então você recebe o `IllegalOwner` do framework no lugar. Para um erro de owner customizado, desça para `UncheckedAccount` e afirme a propriedade com um `constraint` explícito.
- `close = destination` devolve a **reserva isenta de aluguel** da conta ao destino e invalida a conta, atomicamente, dentro da macro. É o jeito correto de devolver o rent de um jogador.
- As sub-keywords de realloc achataram: o `realloc::payer` / `realloc::zero` do v1 agora são **`realloc_payer` / `realloc_zero`**. Mesmo comportamento, grafia nova. A forma antiga não compila.
- `init_if_needed` chega no V2 **sem feature gate** e com validação de reuso de verdade, mas o **ataque de reinicialização** sobrevive a ela. A validação de reuso recheca estrutura, nunca o seu estado de negócio. Essa trava é sua para escrever.

O recuo vai um ponto mais longe do que na lição passada: no Lab eu ainda te entrego o programa inteiro travado por constraints e dois testes passando, mas o vault em si, as seeds dele e o bump armazenado dele, agora chega sem re-explicação. No problema de completion eu tiro de volta os constraints `address` e `close` como TODOs e você reabastece eles. No solo você constrói o caminho do `init_if_needed` sozinho e escreve o teste que comprova que ele não pode ser abusado.

## O catálogo de constraints, keyword por keyword

Comece pela máquina, não pelas keywords, porque a máquina explica por que uma das keywords é uma armadilha. Quando uma transação chega no seu programa, o Anchor não pula direto para dentro do seu handler. Ele primeiro *constrói* a struct de accounts, e essa construção acontece em duas fases ordenadas.

A primeira fase é o **`load`**. Para cada campo `Account<T>`, o framework lê a conta crua, checa que o **owner** dela é o seu programa, e checa que os primeiros bytes casam com o **discriminator** de `T` (a tag que diz "isto é um `Vault`, não um `Config`"). Se qualquer uma das duas checagens falha, o `load` aborta na hora com um erro embutido. A segunda fase são os **hooks de constraint**: `seeds`, `bump`, `address`, `owner`, `constraint`, `has_one`, `close` e companhia. Esses rodam *depois* que todo campo carregou. Um "hook de constraint" é só o código gerado que a macro emite para impor uma cláusula `#[account(...)]`, e ele roda na segunda fase, nunca na primeira.

Glossário, porque ele se paga em trinta segundos: o **hook de constraint** é o passo de imposição de um único constraint, rodado depois do `load`. Mantenha essa ordenação na cabeça. Quase toda ponta afiada deste catálogo é consequência dela.

![A struct do derive valida em duas fases: o load checa owner e discriminator e pode sair com erros embutidos, depois os hooks de constraint rodam antes do handler, que é por que um constraint de owner carregando um erro customizado só dispara em uma conta unchecked.](assets/v01-flowchart.webp)

### address = parent.field: a trava de autoridade, e por que has_one perdeu o emprego

O seu `admin_set_credit` ingênuo precisa de exatamente uma coisa: prova de que o signatário é o operador do fliperama, não um passante qualquer. O Anchor oferece duas grafias para isso há muito tempo. A que todo tutorial de v1 alcança é `has_one`: armazene a chave do operador numa conta `Config` e escreva `has_one = authority` nesse config, querendo dizer "a conta chamada `authority` nesta struct tem de ser igual a `config.authority`". Funcionava, mas era rígido — `has_one` só consegue comparar contra um *campo de chave armazenado* com um *nome de campo casando*. O geral é `address = <expression>`: você põe ele na conta que você quer restringir e dá a ele qualquer expressão que avalie para um `Address`, e a macro checa que a chave da conta é igual a essa expressão, na fase de hooks de constraint, com um `@ CustomError` opcional. Seja preciso sobre a linhagem, porque é fácil errar: `address = <expr>` *não* é uma invenção do V2 — a linha v1 já entrega ele, expressão e erro customizado incluídos (a 0.30 estendeu ele para expressões de campo; a 0.31 corrigiu a regressão que quebrou brevemente as não-const).

O que o V2 de fato muda é o veredito entre as duas: ele aposenta `has_one`. A keyword estreita que só sabia fazer chave-armazenada-igual-a-conta-de-mesmo-nome está depreciada em favor da checagem geral por expressão que faz isso e todo o resto, então o caso especial vira peso morto. A trava do operador, na forma que o V2 padroniza, é uma linha no signatário:

```rust
#[account(address = config.authority @ VaultError::Unauthorized)]
pub authority: Signer,
```

É essa a correção inteira. Se o endereço do signatário não é `config.authority`, a macro levanta `VaultError::Unauthorized` antes de `admin_set_credit` rodar. Nenhum branch no handler, nenhum `require!`, nada para esquecer. E aqui está o fio da trajetória da lição passada aparecendo de novo, em espírito: o V2 segue deletando casos especiais. Na lição passada foi o bump de seed literal dobrando para um const em tempo de macro; aqui é a keyword estreita cedendo lugar à checagem geral que engloba ela. `has_one` só fazia chave-armazenada-igual-a-conta-de-mesmo-nome. `address = expr` faz isso *e todo o resto*, então a estreita vira peso morto.

Ela não sumiu, porém, e o jeito como ela sobrevive é uma bela peça de arqueologia de framework. `has_one` ainda parseia no V2. Ele só emite um aviso de depreciação. O parser armazena o span de origem da keyword especificamente para que o codegen consiga sublinhar ela para você (lang-v2 `derive/src/parse.rs`, em 2026-08). Alguém deliberadamente manteve a localização por perto só para desenhar um risquinho ondulado embaixo dela. Então as migrações não quebram, o código antigo compila, e o compilador te cutuca na direção de `address =` um aviso por vez.

![has_one compara um campo de chave armazenado contra uma conta de mesmo nome e está depreciado, enquanto address = expr compara a chave da conta contra qualquer expressão e é a forma que o V2 padroniza.](assets/v02-comparison.webp)

### owner: a cilada que se esconde à vista de todos

Agora o constraint que o catálogo mais quer que você use errado. Digamos que você aceite uma conta de que um programa *diferente* é dono, algum registry que o seu vault lê mas não é dono, e você quer um erro amigável quando alguém passa a coisa errada. O código óbvio se escreve sozinho:

```rust
// Reads correctly. Does NOT do what you think.
#[account(owner = REGISTRY_PROGRAM_ID @ VaultError::WrongOwner)]
pub registry: Account<Registry>,
```

Você testa com uma conta ruim, esperando `WrongOwner`, e o teste reporta um `IllegalOwner` genérico no lugar. O seu erro customizado nunca dispara. Por quê?

Percorra as explicações ingênuas primeiro, porque descartar elas é o que faz a razão de verdade grudar. Será que a sintaxe de erro `@` só funciona em `constraint =`? Não, `@` amarra erros em vários constraints, `address` e `owner` incluídos. Será que a constante é uma expressão que a macro não consegue avaliar? Não, ela avalia numa boa. A razão é o modelo de fases de duas seções atrás, e nada mais. Em `Account<T>`, a checagem de owner roda no **`load`**, a primeira fase, e o owner contra o qual ela checa é *o seu programa*, hardcoded, porque é isso que um wrapper tipado quer dizer. O `load` reprova a conta antes de existir qualquer hook de constraint para levantar o seu erro, e você recebe `IllegalOwner`.

Leia isso um passo além do que a mensagem de erro lê, porque é o fato mais importante. `owner = <some other program>` num `Account<T>` não está meramente com o erro sombreado, ele é inalcançável: o wrapper tipado já fixou o owner no seu próprio programa, então uma conta de que qualquer outro seja dono nunca consegue carregar como `Account<Registry>`, para começo de conversa. O constraint `owner =` só tem algo a dizer sobre um wrapper que *não* fixou o owner durante o carregamento.

Então a correção não é "mova o erro", é "use o wrapper que deixa a pergunta em aberto". Desça para uma conta crua e afirme a propriedade você mesmo na fase de hooks:

```rust
/// CHECK: ownership is asserted explicitly below so a custom error can fire.
#[account(
    constraint = registry.owner() == &REGISTRY_PROGRAM_ID @ VaultError::WrongOwner
)]
pub registry: UncheckedAccount,
```

`UncheckedAccount` pula o `load` tipado, então nada fixou um owner e não existe checagem adiantada para passar na sua frente. O `constraint =` roda na fase de hooks, avalia o seu booleano, e levanta `WrongOwner` em caso de falha. Você trocou a checagem automática de owner do framework por uma manual, de propósito, para comprar tanto um erro customizado quanto a capacidade de nomear um owner estrangeiro, para começo de conversa. (O registry aqui é ilustrativo; nada no R2 lê uma conta estrangeira ainda, e o caso de owner estrangeiro de verdade chega com tokens no módulo 5. A cilada é a lição.)

O trade-off honesto: `Account<T>` fixar o owner no seu programa durante o `load` é uma *feature* noventa e nove vezes em cada cem. Quer dizer que você quase nunca escreve checagens de owner na mão, e aquela que você escreveu e esqueceu não é um bug porque o framework fez ela por você. A cilada é só o caso extremo em que você quer uma *mensagem customizada* naquela checagem automática. Não saia trocando todo `Account<T>` por `UncheckedAccount` para ter erros bonitos. Você estaria desligando o cinto de segurança para mudar a cor dele.

![Um constraint de owner com um erro customizado em Account<T> roda durante o load e resulta em IllegalOwner, enquanto a mesma asserção em UncheckedAccount roda na fase de hooks e faz aparecer WrongOwner.](assets/v03-comparison.webp)

### close = destination: devolvendo o rent

Um jogador que não quer mais um vault deveria receber o dinheiro dele de volta. Quando você criou o vault na lição passada, o jogador bancou a **reserva isenta de aluguel** dele, o saldo de lamports que toda conta tem de segurar para não ser expurgada pelo runtime. Essa reserva é um depósito e não uma taxa, que é a razão inteira de ela poder voltar: ela fica na conta enquanto a conta viver, e deveria voltar para o jogador no momento em que a conta morre.

`close = destination` faz exatamente isso, e faz como um constraint de macro para que você nunca faça o movimento de lamports na mão:

```rust
#[account(
    mut,
    close = player,          // rent goes to `player`; the account is invalidated
    seeds = [b"vault", player.address().as_ref()],
    bump = vault.bump,
)]
pub vault: Account<Vault>,
```

Três coisas acontecem atomicamente quando esta instrução tem sucesso. O saldo inteiro de lamports da conta, reserva isenta de aluguel incluída, se move para `player`. Os dados da conta são zerados e o discriminator dela é apagado para que ela nunca possa ser silenciosamente revivida e confundida com um vault vivo. E tudo isso fica visível na IDL, então um indexador ou um cliente sabe que esta instrução fecha uma conta sem ler o corpo do seu handler. Na verdade o corpo do handler pode ser vazio, porque o constraint carrega a operação inteira sozinho. Compare isso com a versão na mão, em que você debitaria lamports manualmente, zeraria os dados, e torceria para não ter deixado um caminho de volta à vida, e a diferença de verdade não é concisão: a forma com constraint não consegue esquecer um passo e a na mão consegue.

![Antes do close o vault segura a reserva de rent dele; o constraint close move todo lamport para o jogador, zera os dados, e apaga o discriminator para que a conta não possa ser revivida.](assets/v04-diagram.webp)

### realloc_payer e realloc_zero: a mesma ideia, uma grafia nova

Se o vault algum dia precisar crescer, digamos que você acrescente um slab de recordes recentes, você realoca o espaço dele. O V2 manteve realloc mas achatou a sintaxe das sub-keywords, então o `realloc::payer` e o `realloc::zero` aninhados por dois-pontos do v1 sumiram e as formas do V2 são os identificadores planos e únicos `realloc_payer` e `realloc_zero`. Escreva a forma antiga com dois-pontos e ela não compila:

```rust
#[account(
    mut,
    realloc = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE + EXTRA_SLAB,
    realloc_payer = player,   // v1 was realloc::payer
    realloc_zero = false,     // v1 was realloc::zero; false = keep existing bytes on grow
)]
pub vault: Account<Vault>,
```

`realloc_payer` nomeia quem banca o rent extra quando a conta cresce (crescer precisa de mais rent; encolher devolve rent). `realloc_zero` decide se o buffer redimensionado é apagado: `true` quando você está encolhendo e quer os dados antigos do final fora, `false` quando você está crescendo e quer os bytes existentes preservados. É essa a migração inteira. Se você está portando um programa v1 e o compilador rejeita `realloc::payer`, esta renomeação é o porquê, e a correção é mecânica.

### init_if_needed, in situ: a porta que reabriu

Agora a keyword que o resumo desta lição sinalizou. `init_if_needed` deixa uma instrução dizer "crie esta conta se ela não existe, senão use a que existe". É genuinamente conveniente para um fluxo de "recarregar ou abrir": o jogador roda uma instrução quer ele já tenha um vault, quer não.

No v1, esta keyword ficava atrás de um feature gate e vinha embrulhada em avisos, porque ela é a casa clássica do **ataque de reinicialização**: um atacante força o seu caminho de criar-ou-reusar para o branch de *reuso* numa conta que já segura estado vivo, e o seu handler, achando que acabou de criar uma conta nova, reseta esse estado. Saldo vivo, sumiu. No V2 a keyword chega **sem feature gate nenhum** e com um arquivo de teste de validação de reuso de 604 linhas sustentando ela (testes do lang-v2, 2026-08). Isso é um abrandamento de verdade, tanto da postura travada do v1 quanto da regra da casa que dizia "nunca". Alguém escreveu um monte de testes para tornar isso defensável por padrão.

Aqui está a parte que você não pode ler errado. A validação de reuso do V2 recheca a **estrutura** da conta quando ela já existe: o space está certo, o owner é o seu programa, o discriminator casa com `Vault`. Isso vale ter. O que ela *não* faz, o que ela *não consegue* fazer, é conhecer os seus invariantes. Ela não faz ideia de que `credit` é um saldo vivo que um jogador bancou. Então o ataque de reinicialização sobrevive, em exatamente uma forma estreitada: a validação de reuso trava a *forma*, e o *estado* é seu para travar.

![A validação de reuso cobre space, owner e discriminator numa conta init_if_needed existente mas não o estado de negócio vivo; a trava é ramificar se o vault é novo antes de resetar qualquer campo.](assets/v05-diagram.webp)

A trava é um único branch. Numa conta recém-criada todo byte é zero, então `owner == Address::default()` te diz que ela é nova. Inicialize só nesse caso, e só *adicione* ao saldo, nunca atribua:

```rust
pub fn top_up_or_open(ctx: &mut Context<TopUpOrOpen>, amount: u64) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    if vault.owner == Address::default() {
        // Fresh: reuse-validation confirmed shape, but the fields are still zeroed.
        vault.owner = *ctx.accounts.player.address();
        vault.bump = ctx.bumps.vault;
        vault.credit = 0;
    }
    // Existing OR fresh: only ADD. Never `vault.credit = amount`, that is the clobber.
    vault.credit = vault.credit.checked_add(amount).ok_or(VaultError::Overflow)?;
    Ok(())
}
```

Uma nota honesta de contabilidade, porque a lição passada foi enfática de que créditos são fichas e fichas são lamports. Nem este handler nem `admin_set_credit` movem um único lamport. `credit` é um contador flutuante e solto pela lição inteira, de propósito: mover valor quer dizer uma transferência assinada por PDA, e esse é o assunto inteiro do módulo 4. Então o vault de hoje tem livros honestos e nenhum dinheiro neles. O módulo 4 é onde o número começa a ser lastreado pelo saldo da conta, e onde um `credit` sem lastro vira um bug em vez de uma simplificação.

Esse `if` é a defesa inteira, e é a disciplina para a qual o framework nunca vai te entregar uma keyword. O trade-off do catálogo inteiro aterrissa bem aqui. Constraints movem a validação para dentro da macro, onde ela não pode ser esquecida e onde ela aparece na IDL, o que é um ganho de verdade e a maior parte desta lição. Mas duas pontas continuam afiadas: a cilada do erro de owner quer dizer que uma mensagem de owner customizada precisa de `UncheckedAccount`, e `init_if_needed` troca um branch de conveniência por uma superfície de reinicialização que agora é sua. Ergonomia de um lado, uma disciplina de checagem explícita que você não pode terceirizar para keywords do outro. As keywords fazem muita coisa. Elas não fazem esse `if`.

## Lab: dê um leão de chácara ao vault

Você está estendendo o `quarter_vault` da lição passada para um programa travado por constraints. Ele ganha três coisas neste Lab: uma conta `Config` segurando a chave do operador do fliperama, um `admin_set_credit` travado por autoridade, e um `close_vault` que devolve rent. O caminho do `top_up_or_open` da seção de teoria deliberadamente não está aqui; ele é o challenge solo, e construir ele a frio é o ponto. Dois testes em LiteSVM limpam a barra: uma chamada de admin com autoridade errada é rejeitada *pelo constraint*, e `close_vault` devolve rent e invalida a conta.

**1. Acrescente o estado Config e o init dele.** A chave do operador precisa de um lar. Ponha um PDA `Config` numa seed fixa de instância única. Em `programs/quarter-vault/src/lib.rs`:

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Address, // 32: the arcade operator
    pub bump: u8,           // 1: canonical bump, stored per last lesson's discipline
}

#[derive(Accounts)]
pub struct InitConfig {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        init,
        payer = authority,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<Config>,
    pub system_program: Program<System>,
}
```

E o handler, que é o mesmo padrão de armazenar-o-bump que você já conhece:

```rust
pub fn init_config(ctx: &mut Context<InitConfig>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = *ctx.accounts.authority.address();
    config.bump = ctx.bumps.config;
    Ok(())
}
```

Esperado depois deste passo: o `anchor build` está limpo e o programa agora expõe dois caminhos de init, o `init_vault` da lição passada e o `init_config`. Nada está travado ainda, que é o ponto do próximo passo.

**2. Endureça o caminho de admin.** Troque a struct `AdminSetCredit` ingênua da abertura pela travada. A mudança é uma única linha de constraint, e é o ponto inteiro da lição:

```rust
#[derive(Accounts)]
pub struct AdminSetCredit {
    // The gate: this signer MUST equal config.authority, enforced in the hook phase.
    #[account(address = config.authority @ VaultError::Unauthorized)]
    pub authority: Signer,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<Config>,

    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
}
```

O corpo do handler não muda em relação ao ingênuo. É essa a mensagem em que vale pausar: a segurança se mudou *para fora* do handler e *para dentro* da struct do derive, onde ela não pode ser esquecida e onde a IDL anuncia ela.

![A struct AdminSetCredit trava o signatário authority com address = config.authority, re-deriva o config somente-leitura a partir do bump armazenado dele, e re-deriva o vault alvo mutável do mesmo jeito.](assets/v06-annotated-code.webp)

Esperado depois deste passo: o build *não* compila ainda, e vale ler o erro em vez de temer ele. O constraint nomeia `VaultError::Unauthorized`, um enum que não existe até o passo 4, então o `anchor build` para com um E0433 `failed to resolve` em `VaultError`. Deixe vermelho até o passo 3; o passo 4 paga isso. Assim que o enum aterrissar, a IDL gerada para `admin_set_credit` vai listar uma conta `config` que ela não listava um minuto atrás. Essa nova conta na interface *é* a trava, visível para qualquer um que leia a IDL sem ler o seu Rust.

**3. Acrescente close_vault.** O handler é vazio. O constraint carrega o trabalho:

```rust
pub fn close_vault(_ctx: &mut Context<CloseVault>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct CloseVault {
    #[account(mut)]
    pub player: Signer,
    #[account(
        mut,
        close = player,   // rent-exempt reserve returns to the player; account invalidated
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<Vault>,
}
```

A linha `seeds` está fazendo controle de acesso em silêncio: só o jogador cuja chave deriva exatamente este vault consegue passar na checagem de seed, então ninguém consegue fechar um vault que não é dele. Você não precisou de um constraint de propriedade separado, o esquema de seeds já é um.

**4. Acrescente o enum de erro.** Mantenha um único enum `#[error_code]` por programa — um segundo compila verde, mas os dois numeram suas variantes a partir da mesma base 6000 e colidem em silêncio — então tudo mora aqui:

```rust
#[error_code]
pub enum VaultError {
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
}
```

Esperado depois deste passo: o `anchor build` está limpo. Mas fique de olho num segundo enum `#[error_code]` sobrando: ele compila verde, e os dois enums numeram as variantes deles a partir da mesma base 6000, colidindo silenciosamente em tempo de execução. Toda variante que o programa algum dia vai levantar tem de aterrissar neste.

**5. Comprove a trava com um teste de autoridade errada.** Esta é a trava de avaliação: a rejeição tem de vir do *constraint*, não de um branch no handler. Acrescente isto ao `tests/quarter_vault.rs` que você escreveu na lição passada, mantendo o teste existente daquele arquivo e descartando as linhas `use` duplicadas em vez de colar elas duas vezes:

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

fn setup() -> (LiteSVM, Address) {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();
    (svm, program_id)
}

// New here, because this file now sends three transactions instead of one: fold the
// blockhash-fetch-and-sign into one helper rather than repeating it at every send.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn wrong_authority_is_rejected_at_the_constraint() {
    let (mut svm, program_id) = setup();

    let operator = Keypair::new();   // the real authority
    let attacker = Keypair::new();   // a random signer
    let player = Keypair::new();
    for kp in [&operator, &attacker, &player] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    // init_config with the real operator
    let (config_pda, _) = Address::find_program_address(&[b"config"], &program_id);
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitConfig {
            authority: operator.pubkey(),
            config: config_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitConfig {}.data(),
    };
    let init_config = tx(&svm, &operator, ix);
    svm.send_transaction(init_config).unwrap();

    // init a player vault (uses last lesson's init_vault)
    let (vault_pda, _) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let init_vault = tx(&svm, &player, ix);
    svm.send_transaction(init_vault).unwrap();

    // ATTACK: attacker signs admin_set_credit, presenting themselves as authority.
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::AdminSetCredit {
            authority: attacker.pubkey(),
            config: config_pda,
            vault: vault_pda,
        }.to_account_metas(None),
        data: quarter_vault::instruction::AdminSetCredit { new_credit: 9_999 }.data(),
    };
    let attack = tx(&svm, &attacker, ix);

    // The macro rejects it in the constraint-hook phase, before the handler runs.
    assert!(svm.send_transaction(attack).is_err(),
        "attacker must be rejected by address = config.authority");
}
```

**6. Comprove que close devolve rent.** O segundo teste mostra a reserva voltando para casa e a conta indo embora:

```rust
#[test]
fn close_returns_rent_and_invalidates() {
    let (mut svm, program_id) = setup();
    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();

    let (vault_pda, _) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // init_vault first
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let init = tx(&svm, &player, ix);
    svm.send_transaction(init).unwrap();

    let before = svm.get_balance(&player.pubkey()).unwrap();

    // close_vault
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::CloseVault {
            player: player.pubkey(),
            vault: vault_pda,
        }.to_account_metas(None),
        data: quarter_vault::instruction::CloseVault {}.data(),
    };
    let close = tx(&svm, &player, ix);
    svm.send_transaction(close).unwrap();

    let after = svm.get_balance(&player.pubkey()).unwrap();
    assert!(after > before, "rent-exempt reserve should return to the player");
    assert!(svm.get_account(&vault_pda).is_none(), "closed account is invalidated");
}
```

**7. Rode.**

```bash
anchor test
```

Saída esperada, as duas travas verdes:

```
running 2 tests
test wrong_authority_is_rejected_at_the_constraint ... ok
test close_returns_rent_and_invalidates ... ok

test result: ok. 2 passed; 0 failed
```

Se o teste de autoridade errada *falha* (a transação tem sucesso quando não deveria), a causa de sempre é que você deixou a struct `Signer` ingênua no lugar e nunca acrescentou `address = config.authority`. O constraint é a trava inteira. Sem ele, `Signer` comprova que alguém assinou, nunca quem.

## Challenge

Dois degraus de novo, e desta vez o segundo não tem código nenhum na página.

**Completion.** Abra as structs `AdminSetCredit` e `CloseVault` que você acabou de escrever e apague dois constraints: troque a linha `address = ...` por `// TODO: gate the signer` e a linha `close = ...` por `// TODO: refund + invalidate`. Agora reabasteça as duas de memória. Aceitação: o teste de autoridade errada rejeita no constraint (não no handler), e `close_returns_rent_and_invalidates` passa. Se você se pegar acrescentando uma checagem `if ctx.accounts.authority.address() != ...` *dentro* do handler, pare. Esse é o hábito de v1 que o catálogo existe para deletar. A checagem pertence à struct do derive.

**Solo.** Construa o caminho do `top_up_or_open` com `init_if_needed`, usando o handler travado da seção de teoria, e depois escreva o teste que comprova que o seu branch de reuso não consegue atropelar um saldo vivo. O formato: inicialize um vault, recarregue ele até um credit diferente de zero, depois chame `top_up_or_open` *de novo* com um segundo valor e afirme que o credit final é a *soma*, não o segundo valor sozinho. Essa única asserção é a prova de que a sua trava `if vault.owner == Address::default()` segurou e de que o risco de reinicialização não mordeu. Aceitação: a chamada de reuso preserva e adiciona ao saldo existente, uma chamada nova inicializa limpo, e nenhum dos caminhos reseta `credit` incondicionalmente. Se o seu teste vê o saldo igual só à última recarga, a sua trava está faltando ou invertida, e você escreveu exatamente a vulnerabilidade contra a qual a seção avisou, o que é uma coisa genuinamente útil de ter visto falhar uma vez, de propósito, num teste.

![Uma linha do tempo do init_if_needed atrás de feature gate no v1, passando pela reescrita que projetou a validação de reuso, até o V2 entregar ele sem gate enquanto os seus invariantes de estado de negócio continuam sendo a sua própria trava.](assets/v07-timeline.webp)

Quando os dois degraus passarem, fique um instante com o que o vault virou. Um signatário errado quica na macro de derive antes de o seu código rodar. Um jogador recebe o rent dele de volta com um handler vazio e nenhum caminho de volta à vida. Uma instrução de criar-ou-reusar existe e *não* deixa ninguém sobrescrever um saldo com fundos, porque você escreveu o único `if` que nenhuma keyword vai escrever por você. Cada uma dessas garantias agora está visível na IDL, o que quer dizer que a próxima pessoa a ler o seu programa vê as regras sem ler a lógica. É essa a troca que o catálogo ofereceu, e você ficou com o lado bom dela: validação que você não pode esquecer, menos duas pontas afiadas que você agora conhece pelo nome.

O catálogo que você acabou de percorrer é o conjunto que o *framework* entrega. Mas você vai esbarrar numa regra para a qual o framework não tem keyword nenhuma, algo como "o credit deste vault nunca pode cair abaixo de um piso", e você não vai querer espalhar aquele `require!` por nove handlers. Na próxima lição você escreve o seu próprio namespace de constraints, um `quarters::min_balance` que a macro impõe exatamente como `address` ou `close`, implementando o trait `AccountConstraint` que o framework deixa aberto precisamente para isso. O leão de chácara aprende uma regra da casa que você inventou. Mantenha ele rigoroso.
