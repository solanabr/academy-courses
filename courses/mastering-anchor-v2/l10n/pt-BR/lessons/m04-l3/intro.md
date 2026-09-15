# Composição: construa o prize-escrow

Na lição passada o modelo de borrow fez uma coisa silenciosamente radical: ele matou a cilada do `.reload()` em tempo de compilação. Uma vez que um `CpiHandle` estava vivo, o compilador se recusava a deixar você ler a conta tipada de onde ele veio, então dado obsoleto pós-CPI deixou de ser uma disciplina que você tinha que lembrar e virou uma coisa que não compilava. Você se apoiou no compilador em vez da sua própria atenção. É essa a personalidade inteira do V2, e esta lição é onde ela se paga.

Aqui está a dor. Um vault que só assina por si mesmo é um cofrinho. Útil, claro, mas ele nunca precisa confiar em ninguém. No momento em que uma segunda parte aparece (um operador que financia um prêmio, um jogador que resgata ele só se ele mereceu) você não está mais escrevendo um programa. Você está ligando dois programas e apostando o seu dinheiro na emenda entre eles. Essa emenda se chama composição, e errar ela é onde os escrows vazam.

Então, antes de a gente teorizar, ligue a emenda. O R3 é um segundo programa no workspace do vault, então crie ele lá e aponte ele para o vault. A partir da raiz do seu workspace `quarter-vault`:

```bash
anchor new quarter-prize        # adds programs/quarter-prize and registers it in Anchor.toml
```

Depois ligue o R3 para consumir o R2 do jeito que programas V2 consomem uns aos outros: pela **interface** do vault, não pelo código-fonte dele. O `declare_program!` lê a IDL de um programa — a descrição JSON das instruções e contas dele que o toolchain deriva do código-fonte — e gera a superfície de CPI inteira a partir dela em tempo de compilação. Duas jogadas. Primeiro, extraia a IDL do vault para um diretório `idls/` na raiz do workspace (a macro sobe a partir do crate consumidor e usa o primeiro diretório `idls/` que encontra):

```bash
mkdir -p idls
( cd programs/quarter-vault && anchor idl build -o ../../idls/quarter_vault.json )
```

Segundo, abra o `programs/quarter-prize/Cargo.toml` — e repare no que *não* está nele. Nenhuma linha para o vault: a interface chega como JSON, não como crate.

```toml
[dependencies]
# crates.io, the same release your tag-pinned CLI was built from (see m01-l2)
anchor-lang = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row from m02-l1, verbatim. R3 lands in R2's workspace, and the
# two share one lock: the members have to agree on solana-address or nothing resolves.
solana-address = ">=2.6.1, <2.7"
# NO `quarter-vault = { path = "../quarter-vault", features = ["cpi"] }` row. The
# scaffold's feature table ships a `cpi = ["no-entrypoint"]` hook for source-level
# consumption, and this course deliberately never uses it — the IDL path below has
# none of its rc.1 sharp edges, and it is V2's own cross-program mechanism.
```

Então, no topo do `programs/quarter-prize/src/lib.rs`, uma linha traz o módulo gerado do vault para a existência:

```rust
declare_program!(quarter_vault);
```

Depois `anchor build`. Nada para resgatar ainda, e o build vai começar a falhar no momento em que você escrever código de verdade contra o R2, porque o R2 ainda é de autocustódia e não consegue receber uma segunda parte. Consertar isso é o Passo 1 do Lab e ele vem antes de tudo. O que você tem agora é o módulo `cpi` do programa chamado em escopo — gerado a partir da IDL, e rastreado por `include_bytes!`, então uma edição na IDL obriga o chamador a fazer build de novo e um desvio na forma do tipo é um erro de compilação — embora uma IDL obsoleta de um jeito que ainda passa na checagem de tipos compile verde e só falhe por incompatibilidade em tempo de execução, então gere ela de novo depois de cada mudança no programa chamado em vez de confiar no rastreamento — e um compilador que vai te dizer exatamente quais handles o vault espera. Esse loop de feedback é a lição.

> Freshness note: isto foi escrito contra o release candidate do Anchor V2 na linha 2.x (a árvore de docs publicada sob `v2`), 2026-08-22, tag mais nova `2.0.0-rc.1` (publicada no crates.io em 2026-08-12). O `avm install` não consegue buscar ela, como m01-l2 mostrou: nenhum GitHub Release foi cortado para a tag v2, então o binário pré-compilado que ele baixa dá 404s. Em vez disso, faça o build da CLI a partir do canal documentado, fixada na tag: `cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force`, e reconfira se apareceu um rc mais novo ou uma tag estável antes de fazer o build. O `anchor-cli 1.1.2` default da máquina é a linha V1 e não vai compilar as features `unsafe(dup)`, `CpiHandle` ou Pod-`Account` abaixo.

## Resumo

O R3, o prize-escrow. Duas instruções, duas partes, uma condição. O `reserve` recebe os lamports de um operador e estaciona eles dentro de uma instância real de quarter-vault através de uma chamada entre programas, e depois registra para quem é o prêmio e o que é preciso para ganhar. O `redeem` libera esses lamports, mas só quando a condição de vitória vale e só para o chamador exato que o escrow nomeou. Um resgate prematuro falha. Um resgate com o chamador errado falha. O dinheiro mora o tempo inteiro no vault que você construiu em m04-l1, que é o ponto: o R3 não reimplementa custódia, ele *compõe* sobre a do R2.

A ajuda recua de propósito. A CPI que deposita no vault está trabalhada por inteiro abaixo, com cada handle escrito por extenso, porque um depósito entre programas é o músculo novo e você deveria ver ele se mover uma vez. As duas linhas que fazem ela ser *segura*, a checagem da condição de vitória e o constraint de chamador, são suas para preencher: elas estão marcadas com `TODO(you)` e o checkpoint mostra a resposta. Depois o challenge te solta por completo: um segundo jeito, independente, de ganhar, comprovado no teste por você mesmo.

O roteiro da lição: primeiro como a composição funciona de fato no V2 (a borda do vault, a checagem de chamador, o default de duplicate-account, e a única regra de ordenação que separa um escrow de um exploit), depois o build, depois um checkpoint que comprova isso, depois você estende ele.

## Como um programa constrói sobre outro

Composição é um programa invocando uma instrução em outro e construindo sobre o estado desse programa. É essa a ideia inteira, e é também onde a sua superfície de confiança deixa de ser só sua. Quando o R3 chama o R2, a corretude do R3 agora *inclui* a corretude do R2. Você não está mais confiando só no seu próprio código.

Concretamente, o prêmio nunca fica no escrow. O escrow é um registro: ele diz "50,000,000 lamports para este jogador, liberados sob esta condição, custodiados lá". Os lamports em si moram numa instância de quarter-vault, e o R3 alcança esse vault só por CPI. Essa única seta, o R3 depositando em e depois liberando de um vault do R2, é a razão pela qual esta lição declara que o R3 consome o R2.

![O R3 faz CPI no R2 para depositar os lamports do operador no reserve e, depois de checar chamador e score, para liberar eles no redeem, sem ele mesmo segurar prêmio nenhum.](assets/v01-flowchart.png)

Comparado a quê, porém? O design mais simples e óbvio é deixar o escrow segurar os lamports direto: pular o R2 por completo, creditar a conta do escrow, debitar ela no redeem. Funciona, e para uma coisa de uma vez só é menos código. Mas veja do que você abre mão. Você estaria reimplementando custódia (as movimentações de lamport, a matemática do rent, as checagens de autoridade) dentro do R3, uma segunda cópia de lógica que já mora no R2 e que você já testou. Duas cópias divergem. No dia em que você corrigir um bug de custódia no vault, a cópia privada do escrow ainda vai ter ele. Compor sobre o R2 em vez disso quer dizer que o escrow é dono de exatamente uma coisa, a *decisão* de liberar, e delega o *guardar* ao programa construído para isso. É essa a troca que a lição inteira está defendendo: uma coisa menor sobre a qual você consegue raciocinar, parafusada numa coisa comprovada em que você já confia.

Para essa delegação funcionar, a instância do vault tem que responder ao escrow. O R2 deriva um par de vault a partir do dono dele, `[b"vault", owner]` para o livro-razão e `[b"sol", owner]` para os lamports, e a instância que este escrow usa é criada com o PDA do escrow como esse dono. Então existe exatamente um par de vault por escrow e só o escrow consegue mover os lamports dele. É por isso que o `redeem` consegue assinar o withdraw com as seeds do escrow e por que ninguém mais consegue. A relação de autoridade é o contrato entre os dois programas; erre ela e ou o deposit aterrissa em algum lugar que você não alcança ou o withdraw se recusa a assinar.

Essa delegação custa ao R2 exatamente três contas extras, e vale nomear isso em vez de esconder. O vault que você construiu em m04-l1 é um vault de *autocustódia*: o `deposit` puxa lamports da mesma `authority` que é dona do livro-razão, e o `withdraw` paga de volta para essa mesma authority. Um escrow precisa desses dois papéis separados, porque o operador financia um vault que é do escrow, e o jogador, não o escrow, recebe o pagamento. Então o `deposit` do R2 ganha uma conta `funder` que fornece os lamports, o `withdraw` ganha uma conta `destination` que recebe eles, e o `init_vault` ganha um `funder` que paga o rent, todos distintos da `authority` de que as seeds derivam. Três contas extras, nenhuma lógica nova de custódia, e o R3 constrói sobre o mesmo código em vez de copiar ele. É isso que "componível" de fato custa e de fato compra. O Passo 1 do Lab é essa edição, e é a primeira coisa que você faz, porque nada do R3 compila até o R2 conseguir receber um funder e pagar um destination.

### A checagem de chamador: address, não has_one

Lá em m03-l2, quando a gente percorreu o catálogo de constraints, uma das renomeações mecânicas foi a checagem de autoridade. O `has_one = maker` está depreciado no V2. Ele ainda faz parse, o compilador só sublinha ele e avisa, que é exatamente a migração guiada, de fazer uma vez, de que aquela lição inteira tratava. O substituto é `address = parent.field`:

```rust
// V1 idiom, deprecated in V2 (parses, warns):
#[account(has_one = maker)]
pub escrow: Account<Escrow>,

// V2 idiom: reads as what it does, this account's address must equal that stored field
#[account(mut, address = escrow.maker)]
pub maker: UncheckedAccount,
```

A virada é mais que cosmética. O `has_one` ficava preso a um campo cujo nome batia com o nome da conta. O `address` aceita qualquer expressão, então o `address = escrow.player` em quem faz o resgate diz, sem rodeios, "a conta passada aqui tem que ser a pubkey que este escrow registrou como o jogador". É essa a checagem de chamador para a liberação condicional inteira, e é uma linha.

![has_one = maker vira address = escrow.player; a mesma checagem de igualdade, agora escrita como uma expressão, e o compilador avisa na forma depreciada.](assets/v02-annotated-code.png)

### O default de duplicate-mutable: distintas está tudo bem, com alias não

Aqui está um default que derruba as pessoas na primeira vez e nunca mais. O V2 rejeita *contas mutáveis duplicadas*. Entregue a mesma conta sob dois nomes em uma instrução, bastando que um desses slots esteja marcado como `mut`, e a validação falha antes de o seu handler rodar, com `ConstraintDuplicateMutableAccount`.

Leia isso com atenção, porque a leitura errada comum é cara. Isso *não* quer dizer "duas contas mutáveis estão proibidas". O seu `redeem` recebe o escrow (mutável, ele é fechado) e o vault (mutável, os lamports dele se movem), os dois mutáveis, e o V2 fica perfeitamente feliz, porque são duas contas diferentes. O que o V2 recusa é o *aliasing*: a mesma conta entregue duas vezes sob dois nomes, onde uma escrita mutável atropela a outra em silêncio. Essa é uma classe de bug real, e agora ela é um erro de tempo de compilação e de carregamento em vez de um incidente às 2 da manhã.

Por que o default não te custa nada em tempo de execução no caso comum? Porque a checagem é dividida em dois lugares. A macro `#[derive(Accounts)]` computa, em tempo de compilação, uma máscara de 256 bits de quais campos são mutáveis: essa é a `MUT_MASK`, uma const associada embutida na sua struct de contas. Depois, à medida que o dispatcher carrega contas para a instrução, ele percorre essa máscara contra um bitvec de tempo de execução de endereços vistos até ali. O *formato* da checagem (quais campos são mutáveis) é decidido quando você compila. Os *valores* (dois quaisquer desses endereços são iguais) são decididos quando a transação roda. Const de tempo de compilação, despacho em tempo de execução. Essa divisão é a razão de ela ser barata e de ela não poder ser enganada.

![A macro de derive emite uma MUT_MASK de 256 bits de tempo de compilação dos campos mutáveis; o dispatcher checa os endereços desses campos contra um bitvec de tempo de execução à medida que carrega contas, falhando num alias.](assets/v03-diagram.png)

Quando você genuinamente quer passar uma conta duas vezes como mutável (uma instrução em batch tocando dois pools de prêmio que por acaso resolvem para o mesmo vault, digamos) você faz opt-out por campo, e o opt-out se escreve de um jeito que te faz sentir ele: `unsafe(dup)`. O `dup` puro sem o wrapper `unsafe` é um erro de compilação no V2, de propósito. A keyword é a luz do cinto de segurança: você está desligando uma checagem de alias, então agora o risco de aliasing é seu e você tem que escrever o handler para que ele nunca segure duas referências mutáveis conflitantes para aquela conta.

| O seu `redeem` recebe... | Veredito do V2 | O que você escreve |
|---|---|---|
| escrow (mut) + vault (mut), contas diferentes | aceito | nada extra, esta é composição normal |
| o mesmo vault duas vezes, qualquer um dos slots mut | rejeitado | `ConstraintDuplicateMutableAccount` no carregamento |
| o mesmo vault duas vezes, e você quis isso | aceito | `#[account(mut, unsafe(dup))]` nos dois, e assuma o aliasing |
| `dup` puro sem `unsafe` | erro de compilação | o compilador te diz para escrever `unsafe(dup)` |

### A única regra de ordenação: cheque antes de pagar

Agora a regra que separa um escrow de uma máquina de doação. Uma liberação condicional é só tão segura quanto o *quando* ela checa. A trava tem que rodar e passar antes de um único lamport se mover.

Vale descartar as alternativas ingênuas, porque cada uma delas parece boa até não parecer. A primeira jogada ingênua é "pague o jogador, depois cheque a condição, e se ela era falsa, reverta". Seja honesto com a mecânica primeiro: na Solana isso *reverte* mesmo — uma instrução que no fim das contas dá erro desfaz toda mudança de conta na transação, a CPI de pagamento incluída, exatamente como m04-l1 ensinou. Nada consegue se efetivar parcialmente antes da sua reversão. Então a objeção não é que pagar-depois-checar vaza lamports hoje. É que a segurança dela depende inteiramente de a checagem continuar fatal e continuar dentro desta instrução, para sempre — e refatorações corroem exatamente isso. Um `require!` é afrouxado e vira um branch logado. A condição escorrega para um auxiliar que retorna em vez de dar erro. A checagem começa a ler estado que a própria CPI de pagamento acabou de mudar, então ela mede a coisa errada. No dia em que qualquer uma dessas coisas acontecer, a transferência que você já compôs passa sob uma condição falsa, e nenhum compilador sinaliza isso. Trava-primeiro não tem invariante nenhuma desse tipo para manter: se a trava nunca passa, o código de pagamento nem chega a rodar. A segunda jogada ingênua é "confie na atomicidade, cheque em qualquer lugar da instrução". A mesma dependência frágil, mais um custo mais sutil: ordem é documentação. Um auditor lendo o `redeem` deveria ver a condição travar o pagamento por *posição*, não ter que comprovar que alguma reversão mais adiante te salva.

Então o formato real é forçado: verifique o chamador e a condição, e só então monte a CPI de liberação. No handler, as linhas de `require!` vêm primeiro e a chamada `quarter_vault::cpi::withdraw` vem por último. Nada se move até as travas terem passado.

![O redeem seguro checa o chamador e a condição antes da CPI de withdraw; pagar primeiro e checar depois, ou checar no meio da instrução e confiar na atomicidade, os dois falham.](assets/v04-flowchart.png)

### O trade-off que você está comprando

Composição não é de graça, e nomear a conta é a parte honesta. Três custos vêm anexados, e ou você comprova que eles são seguros, ou você faz opt-in e assume eles.

Primeiro, a sua superfície de confiança multiplicou. O R3 depende de o R2 estar correto; um bug no withdraw do vault agora é um bug no seu escrow. Segundo, a pilha de CPI é limitada, na altura de pilha de 5 em que m04-l1 já colocou um número e uma ressalva de SIMD pendente. O seu escrow chamando o vault fica na altura 2, nem perto dela, mas um protocolo que compõe cinco níveis de profundidade é um protocolo que um dia vai bater na parede, e esta é a lição em que você começa a gastar esse orçamento. Terceiro, o risco de aliasing: duas contas mutáveis em uma instrução são rejeitadas por padrão, e no dia em que você escrever `unsafe(dup)` você assinou pelas consequências por conta própria.

![A altura de pilha 1 é a instrução de topo e cada CPI soma um a um teto vivo de 5, então a chamada escrow-para-vault desta lição fica na altura 2.](assets/v05-table.png)

Existe uma tese embaixo de tudo isso. A issue manifesto do Anchor que deu início ao V2, número 4390, "Zero-copy account deserialization by default," defendeu exatamente uma ideia: torne o modelo de contas seguro por padrão e deixe o que não é sólido falhar na compilação. Composição com borrow rastreado é essa tese aplicada ao caso mais difícil, um programa construindo sobre o estado de outro. O `CpiHandle` que você enfrentou na lição passada e o default de duplicate-mutable que você acabou de conhecer são o mesmo princípio usando dois chapéus.

## Lab: construa o R3

Terminal novo. Esta é a máquina de prêmios de verdade do barcade: um operador carrega um prêmio de pelúcia atrás de um recorde, e o fliperama só paga quando um jogador de fato bate ele.

### Passo 1: separe os papéis do R2 para uma segunda parte poder usar ele

Faça isso primeiro, porque nada abaixo compila até isso estar feito. O vault que você construiu em m04-l1 é de autocustódia: o `deposit` puxa lamports da mesma `authority` de que as seeds derivam, e o `withdraw` paga de volta para essa mesma `authority`. Um escrow precisa desses papéis separados. Abra o R2 e acrescente uma conta em cada struct, deixando cada seed, bump e trava exatamente como estava:

```rust
// quarter-vault: Deposit gains a funder (who pays) alongside authority (who owns).
#[derive(Accounts)]
pub struct Deposit {
    /// CHECK: seeds derive from this; a top-up needs no permission from the owner
    pub authority: UncheckedAccount,   // was Signer: the owner no longer has to sign a deposit
    #[account(mut)]
    pub funder: Signer,                // NEW: sources the lamports, and signs for them
    #[account(mut, seeds = [b"vault", authority.address().as_ref()], bump = state.bump)]
    pub state: Account<Vault>,
    #[account(mut, seeds = [b"sol", authority.address().as_ref()], bump = state.sol_bump)]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}

// quarter-vault: Withdraw gains a destination (who receives) alongside authority (who owns).
#[derive(Accounts)]
pub struct Withdraw {
    #[account(address = state.owner @ VaultError::NotVaultOwner)]
    pub authority: Signer,
    #[account(mut)]
    pub destination: UncheckedAccount, // NEW: receives the payout
    #[account(mut, seeds = [b"vault", authority.address().as_ref()], bump = state.bump)]
    pub state: Account<Vault>,
    #[account(mut, seeds = [b"sol", authority.address().as_ref()], bump = state.sol_bump)]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}
```

O `InitVault` precisa do mesmo tratamento, e é esse que as pessoas deixam passar. Agora mesmo ele lê `authority: Signer` com `payer = authority`, o que diz que o dono do vault ao mesmo tempo autoriza a criação dele e financia o rent dele. Um PDA de escrow não consegue fazer nenhum dos dois: ele não tem chave para assinar e não tem lamports para pagar. Divida esses dois trabalhos do mesmo jeito:

```rust
// quarter-vault: InitVault gains a funder (who pays rent) alongside authority (who owns).
#[derive(Accounts)]
pub struct InitVault {
    /// CHECK: seeds derive from this; creating someone's vault needs no permission
    pub authority: UncheckedAccount,   // was Signer
    #[account(mut)]
    pub funder: Signer,                // NEW: pays rent for both accounts
    #[account(init, payer = funder, space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
              seeds = [b"vault", authority.address().as_ref()], bump)]
    pub state: Account<Vault>,
    #[account(init, payer = funder, space = 0, owner = System::id(),
              seeds = [b"sol", authority.address().as_ref()], bump)]
    /// CHECK: UncheckedAccount because SystemAccount has no init path in V2, and
    /// only UncheckedAccount may `init` with a foreign owner. `owner = System::id()`
    /// does that handoff — without it the account stays owned by THIS program and
    /// every later SystemAccount read fails at load with IllegalOwner.
    pub sol_vault: UncheckedAccount,
    pub system_program: Program<System>,
}
```

Criar um vault para um endereço é inofensivo: custa rent para o funder e dá ao dono uma conta vazia da qual só ele consegue fazer withdraw. É por isso que abrir mão da assinatura é seguro aqui e não seria no `Withdraw`.

Depois aponte as duas CPIs de `Transfer` para as contas novas: o `from` do `deposit` vira `funder`, e o `to` do `withdraw` vira `destination`. Nada mais muda, e esse é o ponto. Três contas extras em três structs, nenhuma lógica nova de custódia, e o R3 consegue construir sobre o mesmo código em vez de copiar ele.

![A autocustódia mantém cada papel nas mãos do jogador através de contas distintas, já que o default de duplicate-mutable rejeita uma chave com alias em vários slots, enquanto o vault que é do escrow separa os papéis entre PDA de escrow, operador e jogador com a lógica de custódia do R2 inalterada.](assets/v06-comparison.png)

Repare no único rebaixamento: a `authority` do `Deposit` deixa de ser um `Signer`. Recarregar o vault de alguém nunca precisou da permissão dessa pessoa, só do endereço dela para derivar as seeds, e o PDA de escrow não consegue assinar um deposit de que ele é meramente o dono. A `authority` do `Withdraw` continua um `Signer`, que é exatamente a conta que o PDA de escrow vai satisfazer através do `invoke_signed` no Passo 4.

Checkpoint: o `anchor build` no workspace do vault compila — se em vez disso ele falhar, você moveu uma seed ou um bump; ponha de volta. Depois rode de novo o teste de withdraw de m04-l1, e rode ele do jeito tentador primeiro: passe a chave do jogador para `authority`, `funder` e `destination`, do jeito que o teste antigo de autocustódia colapsava tudo numa carteira só. Ele falha na primeiríssima instrução com `Custom(2040)` — `ConstraintDuplicateMutableAccount`. Uma chave em dois slots é aliasing, e a trava rejeita uma conta com alias no momento em que *qualquer um* dos slots dela é `mut`; uma `authority` somente leitura não desculpa isso, porque no `InitVault` o `funder` mut dá alias nela e no `Withdraw` o `destination` mut faz o mesmo. Esse 2040 é o default de duplicate-mutable da visão geral fazendo o trabalho dele no seu próprio programa, uma lição mais cedo. A correção é a mesma divisão de papéis que você acabou de fazer nas structs, aplicada ao teste: dê a `funder` e a `destination` os keypairs próprios deles, do jeito que o escrow vai manter partes distintas no salão. Chaves distintas, rodada verde. Uma última jogada antes de você sair deste passo: rode de novo a extração da IDL da abertura. Você acabou de mudar a interface do R2, e o `declare_program!` compila contra o JSON, não contra o código-fonte — o arquivo obsoleto é exatamente o erro de compilação que o checkpoint abaixo nomeia.

### Passo 2: o registro do escrow

O escrow é pequeno e de tamanho fixo, então ele é um `Account` Pod: cada campo é um escalar simples, sem `Vec` nem `String`, que é o que deixa o V2 sustentar ele com o `Account<T>` zero-copy em vez do `BorshAccount<T>`. Crie o `src/state.rs`:

```rust
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Address,      // the operator who funded the prize
    pub player: Address,     // the only caller allowed to redeem
    pub vault: Address,      // the R2 vault ledger holding this prize
    pub amount: u64,         // prize size, in lamports
    pub winning_score: u64,  // the bar the player must clear
    pub bump: u8,            // canonical bump, stored so we never re-derive
    pub _pad: [u8; 7],   // explicit tail padding: V2 rejects implicit pad bytes
}
```

Uma coisa que você *não* escreve aqui, e a deleção vale uma batida. O `Program<T>` precisa de um tipo marcador com um impl de `Id`, e o `#[program]` não emite marcador nenhum — ele gera exatamente três módulos irmãos, `instruction`, `accounts` e `cpi` — então um consumidor em nível de código-fonte teria que escrever um na mão, `IDL_ADDRESS` e tudo, e um impl que pula o `IDL_ADDRESS` ainda compila enquanto a IDL perde em silêncio o endereço do programa chamado. O `declare_program!` gera o marcador em vez disso, em `quarter_vault::program::QuarterVault`, com o `IDL_ADDRESS` já preenchido a partir do JSON. Uma coisa menos para manter na mão, uma coisa menos para errar em silêncio.

Checkpoint: o `anchor build` compila. Se ele reclamar de preenchimento ou de um campo não-Pod, você acrescentou algo de tamanho variável; mantenha o registro escalar.

### Passo 3: reserve, com a CPI de deposit trabalhada por inteiro

O `reserve` cria o registro do escrow e, na mesma instrução, move os lamports do operador para uma instância de quarter-vault chamando o R2. Este é o exemplo trabalhado: leia cada handle.

Uma nota de posicionamento que vale para esta listagem e para a do `redeem` no passo 4, porque colar qualquer uma delas no nível de topo do `lib.rs` compila em código morto e depois falha no passo 5 com um `quarter_prize::instruction::Reserve` não resolvido. As linhas de `use` e a struct `#[derive(Accounts)]` são itens de nível de topo. O `pub fn` vai **dentro** do módulo `#[program] pub mod quarter_prize { use super::*; … }`, exatamente onde o `init_vault` de m03-l1 foi — esse módulo é o que gera os builders `instruction::` e `accounts::` que o teste busca.

```rust
use anchor_lang::prelude::*;
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault; // generated by declare_program!
use quarter_vault::Vault;                 // the vault's account type, also generated
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Reserve {
    #[account(
        init,
        payer = maker,
        space = Escrow::DISCRIMINATOR.len() + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.address().as_ref(), player.address().as_ref()],
        bump
    )]
    pub escrow: Account<Escrow>,

    #[account(mut)]
    pub maker: Signer,

    /// CHECK: recorded as the future claimant; never signs here
    pub player: UncheckedAccount,

    // R2's vault pair, both derived from the escrow PDA as owner. Neither exists
    // yet: `reserve` creates them by CPI, which is why vault_state is unchecked
    // here rather than a typed Account<Vault>.
    /// CHECK: created by the init_vault CPI below and validated by R2's own seeds
    #[account(mut)]
    pub vault_state: UncheckedAccount,
    #[account(mut)]
    pub vault_sol: SystemAccount,      // zero data, holds the actual lamports

    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}

pub fn reserve(ctx: &mut Context<Reserve>, amount: u64, winning_score: u64) -> Result<()> {
    // Copy scalars out BEFORE any handle borrows ctx.accounts (the borrow model,
    // again). The leading `*` is what makes each one a copy: .address() returns
    // &Address, and a bare binding would hold a borrow into ctx.accounts instead.
    let maker = *ctx.accounts.maker.address();
    let player = *ctx.accounts.player.address();
    let vault_ledger = *ctx.accounts.vault_state.address();

    // FIRST: bring the escrow's vault pair into existence. The escrow PDA owns it
    // (the seeds derive from the escrow), but the maker funds the rent, which is
    // exactly the split you just made to R2's InitVault. No PDA signature needed:
    // creating a vault for an address is not an authority action.
    let init_accounts = vault_cpi::accounts::InitVault {
        authority: ctx.accounts.escrow.cpi_handle(),
        funder: ctx.accounts.maker.cpi_handle_mut(),
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    vault_cpi::init_vault(CpiContext::new(
        ctx.accounts.quarter_vault_program.address(),
        init_accounts,
    ))?;

    // THEN: deposit the operator's lamports INTO that vault (the R3 -> R2 edge).
    let cpi_accounts = vault_cpi::accounts::Deposit {
        authority: ctx.accounts.escrow.cpi_handle(),        // owns the vault; signs nothing here
        funder: ctx.accounts.maker.cpi_handle_mut(),        // the operator's lamports
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.quarter_vault_program.address(), cpi_accounts);
    vault_cpi::deposit(cpi_ctx, amount)?;

    // Only after the money is custodied, and after the handles have dropped, do we write the record.
    let escrow = &mut ctx.accounts.escrow;
    escrow.maker = maker;
    escrow.player = player;
    escrow.vault = vault_ledger;
    escrow.amount = amount;
    escrow.winning_score = winning_score;
    escrow.bump = ctx.bumps.escrow;
    Ok(())
}
```

Três coisas para reparar, porque elas são a gramática de CPI do V2. O programa chamado expõe `vault_cpi::accounts::Deposit`, uma struct em que todo campo é um `CpiHandle`. Você preenche ela com `.cpi_handle_mut()` para as contas que o programa chamado vai escrever (o par de vault, o funder) e com `.cpi_handle()` para o resto. O `CpiContext::new` recebe o programa chamado como um `&Address`, que é exatamente o que o `.address()` da conta do programa devolve — então ele é passado direto, não embrulhado em um `AccountInfo`. E o wrapper gerado `vault_cpi::deposit(cpi_ctx, amount)` empacota os argumentos e invoca. É esse o mesmo handle com que você lutou na lição passada: uma vez que o `cpi_handle_mut()` faz borrow do vault, você não consegue também tocar nele como conta tipada até a chamada retornar, que é precisamente como o compilador te mantém honesto.

Checkpoint: `anchor build`. O compilador resolve `quarter_vault::cpi::*` porque o `declare_program!` gerou o módulo inteiro a partir de `idls/quarter_vault.json`. Se ele morrer com `` `idls` directory not found ``, o passo de extração da abertura nunca rodou; se ele não conseguir encontrar um *campo* como `funder`, o seu JSON é anterior à divisão de papéis — rode a extração de novo, porque a interface mudou e o arquivo tem que mudar com ela; e se o módulo em si não for resolvido a partir de um arquivo como `src/instructions/reserve.rs`, escreva o caminho a partir da raiz do crate — `use crate::quarter_vault::cpi as vault_cpi;` — porque a macro gera o módulo na raiz, e o `use quarter_vault::…` pelado de um submódulo não consegue ver ele.

### Passo 4: redeem, e as duas linhas que são suas

O `redeem` é a liberação condicional. As contas e o esqueleto estão aqui; dois trechos são `TODO(you)`. Preencha eles, depois confira contra a solução.

```rust
use anchor_lang::prelude::*;
use quarter_vault::cpi as vault_cpi;
use quarter_vault::program::QuarterVault; // generated by declare_program!
use quarter_vault::Vault;                 // the vault's account type, also generated
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Redeem {
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.player.as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<Escrow>,

    // Bound to the exact vault the escrow recorded: the record's vault, not any vault.
    #[account(mut, address = escrow.vault @ EscrowError::WrongVault)]
    pub vault_state: Account<Vault>,   // mutable AND distinct from escrow: no unsafe(dup) needed
    #[account(mut)]
    pub vault_sol: SystemAccount,      // the lamports actually leave from here

    // TODO(you): constrain this to the recorded player. One line.
    #[account(mut)]
    pub player: Signer,

    #[account(mut, address = escrow.maker)]
    pub maker: UncheckedAccount,       // close destination: rent returns to the operator

    pub quarter_vault_program: Program<QuarterVault>,
    pub system_program: Program<System>,
}

pub fn redeem(ctx: &mut Context<Redeem>, final_score: u64) -> Result<()> {
    // Copy scalars out BEFORE any handle borrows the escrow (the borrow model, again).
    let amount = ctx.accounts.escrow.amount;
    let winning = ctx.accounts.escrow.winning_score;
    let maker_key = ctx.accounts.escrow.maker;
    let player_key = ctx.accounts.escrow.player;
    let bump = ctx.accounts.escrow.bump;

    // TODO(you): check the win condition BEFORE the payout CPI. One line.

    let seeds: &[&[u8]] = &[b"escrow", maker_key.as_ref(), player_key.as_ref(), &[bump]];
    let signer: &[&[&[u8]]] = &[seeds];

    let cpi_accounts = vault_cpi::accounts::Withdraw {
        authority: ctx.accounts.escrow.cpi_handle(),   // escrow PDA owns the vault and signs for it
        state: ctx.accounts.vault_state.cpi_handle_mut(),
        sol_vault: ctx.accounts.vault_sol.cpi_handle_mut(),
        destination: ctx.accounts.player.cpi_handle_mut(),
        system_program: ctx.accounts.system_program.cpi_handle(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.quarter_vault_program.address(), cpi_accounts)
        .with_signer(signer);
    vault_cpi::withdraw(cpi_ctx, amount)?;
    Ok(())
}

#[error_code]
pub enum EscrowError {
    #[msg("The escrow does not custody this vault")]
    WrongVault,
    #[msg("The win condition has not been met")]
    ConditionNotMet,
}
```

As duas respostas. Na conta `player`, a checagem de chamador é uma linha: `#[account(mut, address = escrow.player)]`. Isso é o `has_one` aposentado em favor de uma expressão, exatamente a batida de migração de antes. E a condição, posta onde o `TODO` fica, para que ela trave a CPI por posição:

```rust
require!(final_score >= winning, EscrowError::ConditionNotMet);
```

Repare que o escrow (mutável, ele fecha), o livro-razão do vault e o SOL vault (mutável, os lamports dele se movem) ficam lado a lado, todos os três `mut`, e o V2 nunca pede `unsafe(dup)`, porque são contas distintas.

Repare também no que o R3 *não* checa. Só o `vault_state` carrega um constraint `address =`, amarrando ele ao vault que este escrow registrou. O `vault_sol` não tem constraint de seeds aqui, e isso é deliberado em vez de desleixado: a própria struct `Withdraw` do R2 já valida o par, derivando o SOL vault de `[b"sol", authority]` contra o `sol_bump` armazenado no livro-razão. Re-derivar ele no R3 pagaria duas vezes pela mesma checagem e, pior, recalcularia um bump que o R2 já armazenou. Compor quer dizer confiar que os constraints do programa chamado são trabalho do programa chamado. A checagem que você mantém é a única que só *você* consegue fazer: que este vault é o vault que este escrow nomeou. Todo o resto é o R2 reconquistando a confiança que você depositou nele ao depender dele.

Checkpoint: o `anchor build` está limpo, nenhum aviso de depreciação de `has_one` sobrando (você substituiu ele), e o `redeem` lê de cima para baixo como copia-escalares, trava, assina, paga.

Dê um passo atrás e olhe a vida inteira de um prêmio. Ele existe em exatamente dois estados, e o modelo de contas torna as transições totais: um escrow está ou aberto (financiado, esperando) ou liberado (condição atendida, lamports foram para o jogador, registro fechado). Todo resgate rejeitado deixa ele aberto, inalterado. Não existe um terceiro estado em que o dinheiro está meio movido, porque a liberação é uma instrução e as travas seguram ela.

![Um escrow está ou ABERTO e financiado no vault ou LIBERADO e fechado, e só o chamador certo com um score que passa faz essa transição.](assets/v07-diagram.png)

### Passo 5: comprove isso com um teste LiteSVM

A trava é um teste LiteSVM, que é o template de teste Rust default do V2. Acrescente ele com a mesma única dev-dependency que m04-l1 usou — `anchor-v2-testing` em `tag = "v2.0.0-rc.1"`, que carrega o litesvm 0.11.0 e re-exporta tudo que o arquivo de teste precisa, então você nunca nomeia o litesvm você mesmo — e levante os dois programas no mesmo processo. O teste carrega o R2 e o R3, financia um operador e um jogador, reserva um prêmio de 0.05 SOL atrás de um score de 5000, e depois tenta três redeems: um chamador errado, um score baixo prematuro, e a coisa de verdade.

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};
// The vault's generated module rides inside the escrow crate now, so the test
// borrows its ID from there instead of naming a quarter-vault dependency.
use quarter_prize::quarter_vault;

const PRIZE: u64 = 50_000_000; // 0.05 SOL
const WIN: u64 = 5_000;

fn escrow_pda(maker: &Address, player: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[b"escrow", maker.as_ref(), player.as_ref()],
        &quarter_prize::ID,
    )
}

// One place that signs an instruction into a sendable transaction.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn conditional_release() {
    let mut svm = anchor_v2_testing::svm();
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    let prize_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_prize.so");
    svm.add_program_from_file(quarter_vault::ID, vault_so).unwrap();
    svm.add_program_from_file(quarter_prize::ID, prize_so).unwrap();

    let maker = Keypair::new();
    let player = Keypair::new();
    let stranger = Keypair::new();
    for kp in [&maker, &player, &stranger] {
        svm.airdrop(&kp.pubkey(), 1_000_000_000).unwrap();
    }

    let (escrow, _b) = escrow_pda(&maker.pubkey(), &player.pubkey());
    // R2 derives its vault pair from the owner, which here is the escrow PDA.
    let (vault_state, _sb) = Address::find_program_address(
        &[b"vault", escrow.as_ref()], &quarter_vault::ID,
    );
    let (vault_sol, _lb) = Address::find_program_address(
        &[b"sol", escrow.as_ref()], &quarter_vault::ID,
    );

    // reserve: operator funds the prize into the vault via CPI
    let reserve_ix = Instruction {
        program_id: quarter_prize::ID,
        accounts: quarter_prize::accounts::Reserve {
            escrow,
            maker: maker.pubkey(),
            player: player.pubkey(),
            vault_state,
            vault_sol,
            quarter_vault_program: quarter_vault::ID,
            system_program: System::id(),
        }.to_account_metas(None),
        data: quarter_prize::instruction::Reserve { amount: PRIZE, winning_score: WIN }.data(),
    };
    let reserve_tx = tx(&svm, &maker, reserve_ix);
    svm.send_transaction(reserve_tx).unwrap();
    // the lamports live in the zero-data SOL vault, never in the escrow record
    assert!(svm.get_account(&vault_sol).unwrap().lamports >= PRIZE);
    assert!(svm.get_account(&escrow).unwrap().lamports < PRIZE);

    let redeem = |caller: &Keypair, score: u64| -> Instruction {
        Instruction {
            program_id: quarter_prize::ID,
            accounts: quarter_prize::accounts::Redeem {
                escrow,
                vault_state,
                vault_sol,
                player: caller.pubkey(),
                maker: maker.pubkey(),
                quarter_vault_program: quarter_vault::ID,
                system_program: System::id(),
            }.to_account_metas(None),
            data: quarter_prize::instruction::Redeem { final_score: score }.data(),
        }
    };

    // wrong caller: the address = escrow.player constraint rejects it
    let bad = tx(&svm, &stranger, redeem(&stranger, 9_999));
    assert!(svm.send_transaction(bad).is_err());

    // premature: right player, score below the bar -> ConditionNotMet
    let early = tx(&svm, &player, redeem(&player, 4_200));
    assert!(svm.send_transaction(early).is_err());

    // the real thing: right player, score clears the bar
    let before = svm.get_account(&player.pubkey()).unwrap().lamports;
    let good = tx(&svm, &player, redeem(&player, 5_200));
    svm.send_transaction(good).unwrap();
    let after = svm.get_account(&player.pubkey()).unwrap().lamports;
    assert!(after > before, "the prize should have landed with the player");
}
```

Rode ele. Esta suíte é o único teste LiteSVM no mesmo processo acima, e isso importa para o que o `anchor test` faz: para o template LiteSVM o RC pula levantar um validador local por completo, exatamente como m01-l2 e m01-l3 disseram que ele faria, então nada aqui precisa do Surfpool instalado. (O `anchor test` em um template sustentado por validador é onde o Surfpool entra, e m09-l3 instala ele lá para a rodada de integração do capstone.) Espere:

```text
running 1 test
test conditional_release ... ok

test result: ok. 1 passed; 0 failed
```

Checkpoint, e é este que importa: o chamador errado falha, o resgate prematuro falha, e só limpar a barra de verdade move os lamports. Se o redeem do estranho *tiver sucesso*, o seu constraint de chamador está faltando. Se o redeem de 4,200 tiver sucesso, o seu `require!` está faltando ou está depois da CPI. Essas duas falhas são a lição inteira, pegas pelo teste.

## Challenge: um segundo jeito de ganhar

Você comprovou um caminho de liberação. Agora se solte. Um prêmio de barcade de verdade não é sempre "bata o recorde". Às vezes é "o operador diz que você ganhou" (um override manual para uma final de torneio, julgada offline). Acrescente uma segunda condição de liberação, independente, ao R3 e comprove os dois caminhos no teste.

O formato é seu para escolher, mas a barra de aceitação é fixa: acrescente um campo ao escrow (uma flag `won` que o operador consegue definir é a óbvia), uma instrução pequena ou um argumento para o operador definir ela, e um caminho de redeem que libera por *qualquer um* dos dois, a barra de score ou a flag. Comprove tudo: o caminho da flag libera quando ela está definida e recusa quando não está, o caminho do score continua funcionando, e nenhum dos caminhos deixa o chamador errado passar. Quando as duas condições conseguem abrir a mesma custódia de forma independente e segura, você construiu composição de verdade, não uma demo.

Uma ressalva honesta antes de você rodar. O `final_score` nesta lição é auto-reportado pelo jogador, o que está bem para um teste e está errado para produção. Um fliperama de verdade teria o score *atestado*, assinado pelo programa contador que você construiu no módulo 2 (R1), então o jogador não consegue simplesmente passar 9,999. Essa atestação também é um problema de composição, e ela está deliberadamente fora de escopo aqui: esta lição é sobre a *mecânica* da liberação, não sobre o oráculo. Nomeie essa lacuna no seu próprio código com um comentário para que o próximo leitor saiba que é uma escolha, não um descuido.

## Deu certo?

Você deveria ter agora um programa `quarter-prize` que nunca toca nos lamports do prêmio ele mesmo. Ele reserva eles dentro de uma instância real de quarter-vault através de uma CPI, e libera eles só quando a condição vale e só para o chamador que ele nomeou, com um teste LiteSVM passando que comprova que tanto um resgate com o chamador errado quanto um resgate prematuro falham. A única linha que substituiu o `has_one` é `address = escrow.player`, e a trava que fica na frente do pagamento roda antes da CPI de withdraw, por posição.

Se você empacou, os dois culpados de sempre são os que o teste pega: um constraint `address` faltando (o estranho recebe) ou um `require!` no lugar errado (o score baixo recebe). Os dois são correções de uma linha, e os dois são exatamente por que a gente checou antes de pagar.

Na próxima lição, o chão se move embaixo de tudo isso. Tanto o escrow quanto o vault movem lamports hoje, SOL cru, custodiado na mão. As Quarters estão a ponto de virar tokens SPL de verdade, e aqui está a boa notícia que o modelo de borrow te comprou: cada linha de custódia que você escreveu (a CPI de deposit, a CPI de liberação, o saldo que você rastreia) muda em exatamente um lugar, porque você compôs sobre o vault em vez de copiar ele. Construa sobre estado, e você só paga uma vez por uma mudança.
