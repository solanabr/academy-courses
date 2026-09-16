# O vault paga: assinar uma CPI como um PDA

Na lição passada você deu ao R2 uma regra própria. Você escreveu `quarters::min_balance` como um `AccountConstraint` de verdade, e agora `#[account(quarters::min_balance = 100)]` rejeita um vault subfinanciado em tempo de constraint, antes de o seu handler chegar a rodar. O vault consegue segurar credit e travar credit. O que ele nunca fez, nem uma vez neste módulo inteiro, é pagar alguém de volta.

É essa a lacuna que a gente fecha hoje. E tem um enigma escondido nela. O vault é um PDA. Um PDA não tem chave privada, por construção, então ele não consegue assinar nada do jeito normal. Então, quando um withdrawal move lamports para fora do vault, quem assina? O jogador não: não é a conta dele para gastar. Você também não, segurando algum segredo: não existe segredo para segurar. A resposta é o próprio programa, e fazer o programa assinar pelo PDA dele mesmo é a lição inteira.

Antes de qualquer coisa disso, faça o que torna o perigo concreto. O withdrawal debita um saldo, e um débito é subtração, e subtração num `u64` é a cilada mais sondada de todas num caminho de custódia. Jogue isto num arquivo de rascunho e rode:

```rust
// scratch.rs - the debit an attacker will aim at
use std::hint::black_box;

fn main() {
    // black_box hides the values from const-eval, exactly as a real instruction's
    // arguments would; without it rustc sees the overflow at compile time and
    // refuses to build, which is not the failure we are hunting.
    let balance: u64 = black_box(50);
    let amount: u64 = black_box(100); // someone asks to withdraw more than they have
    let new_balance = balance - amount; // the naive debit
    println!("new balance: {new_balance}");
}
```

```bash
rustc --version              # any stable rustc; 1.93.1 on my machine
rustc scratch.rs -o scratch && ./scratch
```

Um build `rustc` default dá panic: `attempt to subtract with overflow`. Isso parece o desfecho seguro, e é o *menos* ruim dos dois. Faça build do mesmo arquivo com `rustc -O` e o panic some: ele imprime `new balance: 18446744073709551566`, com a subtração tendo dado wrap em silêncio para um `u64` perto do teto dele. On-chain, um panic de debug aborta a sua instrução com um log confuso, e um wrap de release entrega ao chamador um vault que agora acredita ser dono de dezoito quintilhões de lamports. Segure essa linha que falha na cabeça. Toda trava no caminho do withdrawal existe para garantir que aquela subtração nunca seja alcançada com `amount > balance`.

## Resumo

- Um PDA está fora da curva ed25519 e não tem chave privada. O runtime deixa o **programa dono** assinar por ele apresentando as seeds exatas do PDA mais o bump canônico dele para o `invoke_signed`. É isso a assinatura de PDA: uma assinatura sintética que o programa autoriza, sem keypair nenhum em lugar nenhum.
- No Anchor V2 a forma da CPI mudou. `CpiContext::new(program: &Address, accounts)` recebe o programa alvo como um `&Address`, e as contas chegam como handles **`CpiHandle`** com borrow rastreado, vindos de `.cpi_handle()` / `.cpi_handle_mut()`. Você anexa o signatário PDA com `.with_signer(signer_seeds)`.
- Tem uma regra dura de propriedade embaixo de tudo isso: o System Program só consegue dar `transfer` em lamports para fora de uma conta da qual ele é *dono*. Um PDA que carrega dados (o seu `Account<Vault>`) pertence ao seu programa, então uma transferência de System a partir dele falha. Os lamports têm de morar num PDA separado, de propriedade do System, pelo qual o programa assina.
- O bump com que você assina é o bump canônico **armazenado**, lido do estado da conta, nunca recalculado. Recalcular ele a cada chamada é ao mesmo tempo um custo de CU e um risco de correção.
- O débito é `checked_sub`, e o withdrawal é travado *antes* de a CPI disparar: nenhum pedido zerado, nenhum over-withdraw, e nunca uma queda abaixo do piso isento de aluguel do vault.

O recuo da ajuda: no Lab eu escrevo com você a chamada inteira de `CpiContext::new` mais `invoke_signed`, linha por linha. No problema de completion você repõe só duas linhas de memória, o array de signer seeds e o débito checado. No problema solo você implementa `resolve_withdrawal`, a trava pré-CPI, a partir de uma especificação sem apoio.

## Assinar por uma conta que não tem chave

### Quem assina quando não existe chave privada

Comece do modelo que você já tem. Uma conta normal da Solana é um keypair: quem segura a chave privada consegue assinar uma transação que gasta a partir dela. É assim que a carteira do jogador funciona. Um PDA quebra esse modelo de propósito. Ele é um endereço derivado do ID do seu programa e de um conjunto de seeds, empurrado deliberadamente para *fora* da curva ed25519 para que nenhuma chave privada possa um dia corresponder a ele. Isso é a feature, não uma limitação: um vault sem chave é um vault cuja chave não pode ser roubada, nem obtida por phishing, nem vazada.

Mas um vault que consegue segurar fundos e nunca move eles é um cofrinho que você tem de quebrar. Então o runtime oferece uma troca. Quando o seu programa chama `invoke_signed` e entrega as seeds exatas mais o bump canônico usados para derivar uma das contas da chamada, o runtime re-deriva o endereço a partir dessas seeds e do ID do seu programa. Se bater, o runtime marca aquela conta como signatária pela duração da chamada. Nenhuma criptografia acontece. É uma permissão que o runtime concede porque só o programa dono daquelas seeds poderia ter apresentado elas. O PDA "assina" do jeito que um gerente assina pela conta de uma empresa: não com a identidade própria, mas provando que está autorizado a agir por ela.

![Uma carteira assina com a chave privada dela, enquanto um PDA é assinado pelo programa dono dele através do invoke_signed, e a tentativa forjada de outro programa falha na re-derivação.](assets/v01-diagram.webp)

A última linha daquele diagrama é o modelo de segurança inteiro numa frase só: uma CPI assinada por PDA só consegue assinar por seeds das quais *este* programa é dono. Você não pode assinar pelo PDA de outro programa, e ninguém pode assinar pelo seu. Guarde essa frase. Ela também é o limite exato do que o withdrawal pode e não pode fazer.

### A cilada que decide o design inteiro

É aqui que a maioria de quem já fez isso no v1 se dá mal, então a gente encara ela de frente antes de construir. O plano óbvio é: o PDA do vault segura lamports, o programa assina como o vault, e faz uma CPI para o System Program dar `transfer` naqueles lamports para o jogador. Limpo. Também não funciona, e vale entender a razão, porque ela dita a forma de tudo que vem abaixo.

O System Program só move lamports para fora de uma conta da qual o próprio System Program é *dono*. O seu `Account<Vault>` é uma conta que carrega dados: ela carrega a struct `#[account]` com o dono, o credit, o bump. No instante em que uma conta carrega os dados do seu programa, ela pertence ao seu programa, não ao System Program. Então um `transfer` de System com o seu vault de dados como origem falha em tempo de execução com um erro bem específico: `Transfer: from must not carry data`. É o erro de PDA mais comum de todos, e ele falha *depois* de você ter escrito e feito deploy da versão ingênua, que é a pior hora de aprender isso.

Existem dois jeitos corretos de mover lamports, e qual deles você usa depende inteiramente de quem é dono da conta de origem:

![Um PDA de dados de propriedade do programa não pode ser origem de uma transferência de System e tem de mover lamports diretamente, enquanto um PDA sem dados de propriedade do System pode, assinado via invoke_signed.](assets/v02-comparison.webp)

Leia a linha de conclusão, porque ela força a nossa arquitetura. Esta lição é sobre assinar uma CPI como um PDA. Esse mecanismo, `invoke_signed` para dentro do System Program, só existe para o caso de propriedade do System. Então o R2 não pode manter os lamports dele dentro do vault de dados. Ele precisa de um segundo PDA: um `SystemAccount` sem dados que segura o SOL de verdade, um pelo qual o programa consegue assinar uma transferência de System. O vault de dados continua sendo o livro-razão; o SOL vault novo segura o dinheiro. Essa divisão não é complexidade incidental, é a decisão de custódia para a qual o módulo vinha construindo, e é por isso que o vault não conseguia simplesmente "pagar" até agora.

### O R2 ganha um segundo PDA

Então o quarter-vault depois de hoje são duas contas embaixo de cada jogador, derivadas do mesmo dono mas com seeds diferentes:

![O R2 dá a cada jogador um PDA de state de propriedade do programa segurando o livro-razão e um PDA de SOL sem dados de propriedade do System segurando os lamports que o programa assina para mover.](assets/v03-diagram.webp)

Duas seeds, `b"vault"` e `b"sol"`, mantêm os endereços distintos para que um jogador tenha os dois. O PDA de state armazena dois bumps agora: o `bump` dele mesmo (inalterado desde as lições anteriores) e o `sol_bump`, o bump canônico do SOL vault, que a gente captura uma vez na inicialização e reusa para sempre. Por que armazenar o bump do SOL vault na conta de state e não recalcular ele no `withdraw`? É a próxima seção, e é onde os números de CU entram.

### A forma da CPI do V2, percorrida

A CPI do V2 é a forma que você vai digitar a lição inteira, então vamos nomear cada parte dela antes de ligar ela num handler. O Anchor V2 reconstruiu essa superfície. Duas coisas se mudaram da forma pré-1.0 e da linha 1.0 também: o argumento de programa agora é um `&Address`, não um `AccountInfo` e não um `Pubkey` por valor, e as contas são passadas como handles `CpiHandle` com borrow rastreado em vez de structs tipadas simples. Você obtém esses handles com `.cpi_handle()` para uma leitura e `.cpi_handle_mut()` para uma escrita. O signatário PDA vai junto através do `.with_signer(signer_seeds)`, que é o wrapper ergonômico em cima do syscall `invoke_signed` cru.

![Uma chamada de transfer do V2 anotada rotulando address(), as signer seeds com o bump armazenado, o argumento de programa &Address, os handles cpi_handle_mut e o with_signer.](assets/v04-annotated-code.webp)

Uma ressalva sobre aquele bloco, a mesma que as lições anteriores carregavam: isto é um release candidate, `2.0.0-rc.1` no crates.io na hora em que isto foi escrito. A ergonomia da CPI tem confiança alta mas ainda está assentando, então trate as formas exatas dos tokens como um alvo em movimento e releia elas contra o crate quando for fazer build. Os conceitos embaixo delas, um programa `&Address`, contas `CpiHandle` e `with_signer` para o PDA, são a parte estável.

### invoke ou invoke_signed: a bifurcação do deposit e do withdraw

Existem dois jeitos de fazer uma CPI, e a diferença é uma coisa só: de quem é a assinatura que o chamado precisa. O `invoke` é para uma CPI em que todo signatário exigido por ela já assinou a transação externa. O `invoke_signed` é para o caso em que o programa chamador tem de assinar em nome de um PDA do qual ele é dono. Por baixo do capô são o mesmo syscall, `sol_invoke_signed_rust`; o `invoke` é literalmente o `invoke_signed` com um array de seeds vazio. Então o modelo mental não é "duas funções", é "uma função, e se você entrega seeds para ela".

Os dois handlers de dinheiro do R2 caem em lados opostos dessa bifurcação, que é a razão de construir os dois numa lição só valer o handler a mais. Um `deposit` move lamports do jogador *para dentro* do SOL vault. O jogador é dono da origem e já assinou a transação, então a transferência de System não precisa de assinatura extra: `invoke` puro, um `CpiContext::new` sem nada anexado. Um `withdraw` move lamports *para fora* do SOL vault, cuja origem é um PDA sem chave que não assinou nada, então o programa tem de fornecer as seeds e deixar o runtime conceder ao PDA a assinatura sintética dele: `invoke_signed`, expresso como `.with_signer(signer_seeds)`.

![O deposit usa invoke porque o jogador já assinou, enquanto o withdraw usa invoke_signed porque a origem dele é um PDA sem chave pelo qual o programa assina com seeds.](assets/v05-comparison.webp)

Tem uma regra de privilégio que vale internalizar enquanto você está por aqui, porque ela é o guarda-corpo do runtime que torna a assinatura de PDA segura de expor. Uma CPI nunca consegue escalar privilégios: se o chamador não tinha uma conta como gravável ou como signatária, o chamado não pode inventar aquele privilégio. A única exceção sancionada é exatamente a assinatura de PDA: o runtime acrescenta um PDA ao conjunto de signatários, mas só quando o programa chamador apresenta seeds que re-derivam para aquele PDA sob o ID do próprio programa chamador. É essa a razão inteira de uma conta sem chave ser segura para receber poder de gasto. A autoridade não é um segredo que pode vazar, é uma derivação que só o programa dono consegue produzir.

### O bump é armazenado, não recalculado

As signer seeds carregam `ctx.accounts.state.sol_bump`, um byte que a gente lê do estado da conta. A gente não chama `find_program_address` dentro do `withdraw` para redescobrir ele. Vale ser preciso sobre isso, porque "funciona dos dois jeitos" é verdade e enganoso ao mesmo tempo.

Um bump canônico é encontrado pelo `find_program_address`, que começa no bump `255` e caminha para baixo, chamando `create_program_address` a cada passo até aterrissar num endereço que está fora da curva. Cada uma dessas tentativas de derivação é um syscall com um preço real em CU, e a caminhada pode levar uma tentativa ou muitas. Armazene o bump uma vez no init e o `withdraw` pula a busca inteira: ele apresenta o byte sabidamente bom e o runtime faz uma única re-derivação para checar ele. Recalcule o bump a cada chamada e você paga pela busca inteira, a cada chamada, para sempre, num caminho quente. A economia é real mas é uma *faixa*, não um número fixo, porque depende de quantos bumps a busca tem de tentar e dos custos por syscall atuais do runtime.

Esse último ponto não é enrolação, e é por isso que este curso cita economias de CU como faixas e nunca congela elas. Os benchmarks do Anchor V2 ficaram mais honestos com o tempo:

![Uma linha do tempo mostrando as alegações de manchete de 95 por cento e 9.9x do Anchor V2 revisadas para baixo até 94 por cento e 8.8x pelo PR 4914 em 2026-08-13.](assets/v06-timeline.webp)

A lição do PR #4914 não é que o Anchor ficou mais lento. É que um número de manchete que um mantenedor corrigiu uma vez vai ser corrigido de novo, então o jeito honesto de falar de um ganho de CU é como uma faixa que você re-mede no seu próprio programa, não um troféu que você cita. Armazenar o bump é inequivocamente mais barato que recalcular ele; o delta exato é seu para perfilar.

Tem uma borda de correção também, não só de custo. O `find_program_address` sempre devolve o bump *canônico* (o mais alto), mas uma derivação feita na mão com o bump errado pode aterrissar num endereço válido-mas-não-canônico, e um programa que às vezes assina por um endereço diferente do que ele armazena é um bug sutil e horroroso. Armazenar o único bump canônico no init e reusar ele elimina a classe inteira. Armazene o bump.

### O trade-off, dito sem rodeios

A assinatura de PDA torna o programa a autoridade sobre o dinheiro do vault. Isso é exatamente tão poderoso, e exatamente tão perigoso, quanto parece. Todo caminho de withdrawal que você expõe é um caminho que um atacante vai sondar: uma chamada de valor zero para ver o que acontece, um over-withdraw para caçar aquela subtração não checada, um pedido dimensionado para drenar o SOL vault abaixo do piso isento de aluguel dele e fechar a conta em silêncio por baixo do livro-razão. O programa é a única coisa entre "o vault paga a pessoa certa o valor certo" e "o vault paga quem pedir". Então as travas são o limite de segurança.

Uma propriedade está trabalhando a seu favor aqui, e vale nomear ela para você se apoiar nela do jeito certo. Uma instrução é atômica: se o handler devolve um erro em qualquer ponto, toda mudança que ele fez, incluindo uma CPI que já rodou, é desfeita. Então a ordem no `withdraw`, transferir primeiro e depois debitar o livro-razão, é segura mesmo parecendo arriscada. Se o `checked_sub` no livro-razão de alguma forma falhar depois de a transferência ter dado certo, a instrução inteira aborta e a transferência é desenrolada junto. Você nunca termina no meio-estado em que os lamports saíram mas o livro-razão não registrou. O que a atomicidade *não* faz é te salvar de um débito *não checado* que dá wrap em vez de dar erro: um wrap silencioso não é uma falha, então nada é desfeito, e o vault fica acreditando numa mentira. A atomicidade te protege de erros, não de bugs que nunca levantam um. É esse o argumento inteiro a favor do `checked_sub` em vez do `-` em três palavras: faça do bug um erro.

![O checked_sub levanta um erro para que a instrução inteira seja desfeita e o vault continue correto, enquanto a subtração simples, num build com overflow-checks desligado, dá wrap em silêncio e deixa a transferência commitada.](assets/v07-comparison.webp)

Dois limites duros vêm junto, e os dois são restrições que você aceita como dadas em vez de brigar com elas. Primeiro, o limite de seeds do diagrama de abertura: uma CPI assinada por PDA só consegue assinar por seeds das quais este programa é dono, então o `withdraw` consegue mover os lamports do SOL vault e nada mais. Segundo, a profundidade de CPI. A altura máxima da pilha de instruções é 5, o que quer dizer que um programa pode aninhar CPIs até 4 níveis de profundidade. O SIMD-0268 (status Accepted) sobe o limite de aninhamento de 4 para 8, uma altura de pilha de 9, mas o feature gate dele `6TkHkRmP7JZy1fdM6fg5uXn76wChQBWGokHBJzrLB3mj` ainda não tem conta na mainnet em 2026-08-22, então trate 5 como a lei e sonde o gate de novo na hora do build em vez de confiar num status que você cacheou. O nosso withdrawal tem uma CPI de profundidade, longe do teto, mas o número importa no instante em que o seu programa chama um programa que chama um programa.

Vale afastar o zoom por uma batida antes de a gente construir, porque esta instrução é a linha em que o módulo inteiro deixa de ser um brinquedo. Um programa que só consegue receber depósitos e ler saldos é um demo, e um demo é uma coisa que você mostra uma vez e nunca deixa cuidar de nada que importe; no instante em que um programa consegue mover valor para fora sob a autoridade dele mesmo, ele vira uma coisa em que um estranho pode se apoiar sem nunca te conhecer, e é essa a promessa inteira de colocar custódia numa chain em vez de dentro de uma empresa. Tudo que você construiu até aqui, o counter, o vault, os bumps armazenados, o constraint customizado, vinha montando em silêncio a única capacidade que faz qualquer coisa disso valer o deploy: a habilidade de segurar o dinheiro de alguém e devolver ele corretamente, de forma comprovável, sem um humano no meio que pudesse pegar ele ou perder ele. Essa capacidade é também precisamente a que um atacante mais quer quebrar, que é a razão de a trava sem glamour que você está prestes a escrever, três comparações e uma subtração checada, carregar mais do peso real do programa do que qualquer feature que você vá adicionar em cima dela.

## Lab: pague a partir do vault

Você está estendendo o R2, o programa `quarter_vault`, com uma instrução `withdraw` que assina uma transferência de System como o PDA do SOL vault e debita o livro-razão com matemática checada. Quando você terminar, o `anchor test` fica verde: um withdrawal assinado por PDA move lamports do SOL vault para o jogador e debita `credit`, e um over-withdraw devolve um erro em vez de dar panic. Aqui está a forma do handler que você está construindo, para os passos terem onde aterrissar:

![Um fluxograma de withdraw em que a trava rejeita pedidos zerados, over-withdraw e abaixo do piso de aluguel antes de a CPI de transferência assinada por PDA rodar e o livro-razão ser debitado.](assets/v08-flowchart.webp)

**1. Fixe o toolchain do V2.** O release candidate do V2 não desce pelo `avm install`: aquele comando baixa um binário pré-compilado da Release do GitHub daquela tag, nenhuma Release foi cortada para a tag v2, e o download dá 404, exatamente como a lição de toolchain (m01-l2) mostrou. O canal documentado é um install via git do cargo, fixado na tag `v2.0.0-rc.1` em vez da ponta do branch `anchor-next` em que ela está. Se você fez as lições anteriores deste módulo você já tem ele; se não, instale e confirme. Não faça build de conteúdo V2 num binário `anchor` V1:

```bash
# macOS, if the build trips on LTO: prefix with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
anchor --version   # must report 2.0.0-rc.1 (the RC as of 2026-08-12; re-check for a newer rc/stable), not a 1.x line
```

**2. Estenda o estado do vault com o bump do SOL vault, e renomeie dois campos já que você está lá dentro.** Abra o `lib.rs` do R2.

Duas renomeações mecânicas primeiro, porque o vault não é mais só de um jogador. Agora ele tem um dono que pode ser um jogador hoje e um programa de escrow amanhã, e agora ele tem duas contas em vez de uma, então "o vault" é ambíguo. Em toda struct de accounts do programa, renomeie o campo `player: Signer` para `authority`, e o campo `vault: Account<Vault>` para `state`. O *tipo* da conta continua `Vault`; só os nomes dos campos se mudam. Os builders do seu arquivo de teste nomeiam esses campos, então ele vai parar de compilar até você renomear lá também, e é o compilador fazendo a sua migração por você.

Depois o estado em si. O `Vault` das lições anteriores guardava o dono, um saldo de `credit`, o `bump` dele mesmo, e o preenchimento de cauda explícito que mantém ele Pod. Acrescente um campo, `sol_bump`, o bump canônico do PDA que segura o SOL, para o `withdraw` conseguir remontar as signer seeds sem uma busca. Ele sai do preenchimento, então o tamanho da conta não muda:

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

// Still the id anchor init generated for you.
declare_id!("<your generated program id>");

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address, // 32 bytes: whoever owns this vault
    pub credit: u64,    //  8 bytes: the withdrawable ledger balance
    pub bump: u8,       //  1 byte: this state PDA's canonical bump
    pub sol_bump: u8,   //  1 byte: the SOL vault PDA's canonical bump, stored at init
    pub _pad: [u8; 6],  //  6 bytes: explicit tail padding (was 7, sol_bump took one)
}
```

**2b. Ensine o `init_vault` sobre o segundo PDA.** Nada escreve `sol_bump` ainda, e um zero ali é o pior tipo de bug: todo `withdraw` remonta as signer seeds contra um bump que nunca foi canônico, o runtime se recusa a marcar o PDA como signatário, e o erro não diz nada sobre o porquê. O `init_vault` também tem de trazer o SOL vault à existência, porque um PDA `SystemAccount` que nenhuma instrução nunca criou é só um endereço vazio. Os dois trabalhos são uma edição só:

```rust
pub fn init_vault(ctx: &mut Context<InitVault>) -> Result<()> {
    let bump = ctx.bumps.state;
    let sol_bump = ctx.bumps.sol_vault;   // the second PDA's canonical bump
    let state = &mut ctx.accounts.state;
    state.owner = *ctx.accounts.authority.address();
    state.credit = 0;
    state.bump = bump;
    state.sol_bump = sol_bump;            // store it once, sign with it forever
    Ok(())
}

#[derive(Accounts)]
pub struct InitVault {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        init,
        payer = authority,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", authority.address().as_ref()],
        bump,
    )]
    pub state: Account<Vault>,
    // The money side: zero data, System-owned, created here so it exists to be
    // signed for later. `space = 0` plus the explicit `owner` are what keep it a
    // legal System transfer source.
    #[account(
        init,
        payer = authority,
        space = 0,
        owner = System::id(),
        seeds = [b"sol", authority.address().as_ref()],
        bump,
    )]
    /// CHECK: typed UncheckedAccount because SystemAccount has no init path in V2,
    /// and UncheckedAccount is the one wrapper `init` may hand to a foreign owner.
    /// The `owner = System::id()` line does that handoff; without it, `init`
    /// defaults the owner to THIS program, and every later SystemAccount read of
    /// this PDA fails at load with IllegalOwner.
    pub sol_vault: UncheckedAccount,
    pub system_program: Program<System>,
}
```

Repare de onde vem cada bump: `ctx.bumps.state` e `ctx.bumps.sol_vault`, um campo tipado por PDA na struct, os dois fornecidos pela macro porque os dois escreveram um `bump` pelado. É o único lugar deste programa em que um bump chega a ser buscado. Tudo rio abaixo lê o byte armazenado.

**3. Comprove a matemática do débito isolada.** Antes de tocar na CPI, acerte a versão *segura* do rascunho de abertura, porque o débito checado é uma das duas linhas que o problema de completion devolve para você. O `balance - amount` ingênuo deu panic ou deu wrap; o `checked_sub` devolve `None` no underflow para você poder transformar isso num erro limpo:

```rust
// scratch.rs - the safe debit
fn debit(balance: u64, amount: u64) -> Result<u64, &'static str> {
    balance.checked_sub(amount).ok_or("underflow: over-withdraw")
}

fn main() {
    assert_eq!(debit(100, 40), Ok(60)); // normal
    assert_eq!(debit(50, 100), Err("underflow: over-withdraw")); // rejected, no panic
    println!("checked debit ok");
}
```

```bash
rustc scratch.rs -o scratch && ./scratch   # prints: checked debit ok
```

É esse o débito travado. `None` no underflow, mapeado para um erro, nunca um panic e nunca um wrap. No handler de verdade o erro é um erro de programa, não um `&str`, mas a lógica é exatamente esta.

**4. Acrescente as variantes de erro.** Um programa ganha um espaço de erro com base em 6000, e um segundo enum `#[error_code]` compila verde enquanto numera as variantes dele em silêncio dentro dessa mesma faixa — uma colisão de offset, não um erro de compilação — então estenda o `VaultError` que você já tem em vez de adicionar um segundo enum. Mantenha toda variante das duas últimas lições, na ordem, e acrescente as novas: acrescentar importa, porque as variantes são numeradas a partir de 6000 por posição e reordenar elas renumera em silêncio erros nos quais os seus testes já fazem assert. O `withdraw` precisa de cinco novas razões para recusar:

```rust
#[error_code]
pub enum VaultError {
    // --- already yours, from m03-l2 and m03-l3. Do not reorder. ---
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
    #[msg("vault credit is below the quarters::min_balance floor")]
    BelowFloor,
    // --- new today ---
    #[msg("withdrawal amount must be greater than zero")]
    ZeroWithdrawal,
    #[msg("withdrawal exceeds the vault's lamport balance")]
    Overdraw,
    #[msg("withdrawal would drop the SOL vault below its rent-exempt floor")]
    WouldCloseVault,
    #[msg("arithmetic underflow while debiting the ledger")]
    Underflow,
    #[msg("caller is not the owner this vault recorded")]
    NotVaultOwner,
}
```

**5. Escreva o handler `withdraw`.** Este é o núcleo trabalhado. A trava roda primeiro e rejeita todo pedido inseguro antes de um único lamport se mover. Depois as signer seeds são montadas a partir das seeds do próprio SOL vault mais o `sol_bump` *armazenado*. Depois a CPI do V2 assina a transferência de System como o PDA. Depois o livro-razão é debitado com `checked_sub`. Repare que o handler recebe `&mut Context<T>`, a assinatura do V2:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
        // --- guard: refuse every unsafe request before touching lamports ---
        let vault_lamports = ctx.accounts.sol_vault.lamports();
        let rent_exempt_min = Rent::get()?.minimum_balance(0); // SOL vault carries zero data
        require!(amount > 0, VaultError::ZeroWithdrawal);
        require!(amount <= vault_lamports, VaultError::Overdraw);
        // checked_sub even here, where the line above already proved it cannot
        // underflow. The proof is one refactor away from being wrong, and this
        // lesson opened on what a bare `-` does in a release build.
        let remaining = vault_lamports
            .checked_sub(amount)
            .ok_or(VaultError::Overdraw)?;
        require!(remaining >= rent_exempt_min, VaultError::WouldCloseVault);

        // --- the PDA-signed CPI: sign the System transfer AS the SOL vault ---
        // Copy these out of ctx.accounts BEFORE any CPI handle is built. The `*`
        // matters: .address() returns &Address, and the deref makes `owner` an
        // owned copy instead of a live borrow of ctx.accounts.
        let owner = *ctx.accounts.authority.address();
        let sol_bump = ctx.accounts.state.sol_bump;
        let signer_seeds: &[&[&[u8]]] = &[&[b"sol", owner.as_ref(), &[sol_bump]]];
        let cpi = CpiContext::new(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.sol_vault.cpi_handle_mut(),
                to: ctx.accounts.authority.cpi_handle_mut(),
            },
        )
        .with_signer(signer_seeds);
        transfer(cpi, amount)?;

        // --- debit the ledger with checked math ---
        ctx.accounts.state.credit = ctx
            .accounts
            .state
            .credit
            .checked_sub(amount)
            .ok_or(VaultError::Underflow)?;

        Ok(())
    }

    /// Contrast handler: a deposit needs NO invoke_signed. The player owns the
    /// source, so the player signs the System transfer the ordinary way.
    pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.authority.cpi_handle_mut(),
                to: ctx.accounts.sol_vault.cpi_handle_mut(),
            },
        );
        transfer(cpi, amount)?;
        ctx.accounts.state.credit = ctx
            .accounts
            .state
            .credit
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }
}
```

Olhe os dois handlers lado a lado, porque o contraste é a espinha da lição. O `deposit` move lamports *para dentro* do SOL vault a partir do jogador, e ele não precisa de `with_signer`: o jogador é dono da origem e assina a transação externa, então um `invoke` comum (um `CpiContext::new` sem signatário anexado) basta. O `withdraw` move lamports *para fora* do SOL vault, cuja origem é um PDA sem chave, então ele tem de anexar `signer_seeds` e deixar o programa assinar. Mesma transferência de System, direção oposta, e a direção é o que decide quem assina.

Mais um detalhe que vale reparar agora: `owner` e `sol_bump` são copiados para fora de `ctx.accounts` para locais simples *antes* de qualquer `cpi_handle` ser construído. Essa ordem é estrutural no V2, não uma escolha de estilo, e a próxima lição é inteiramente sobre o porquê.

**6. Ligue as contas.** O `withdraw` precisa do livro-razão de state, do SOL vault de propriedade do System, do jogador e do System Program. O SOL vault é um `SystemAccount` (zero dados, então uma transferência de System pode ter origem nele), e o constraint `bump` dele reusa o `sol_bump` armazenado, não uma busca nova. Repare que nem o state nem o SOL vault são validados recalculando um bump: os dois usam `bump = ...` com o byte armazenado.

```rust
#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut, address = state.owner @ VaultError::NotVaultOwner)]
    pub authority: Signer,

    #[account(
        mut,
        seeds = [b"vault", authority.address().as_ref()],
        bump = state.bump,
    )]
    pub state: Account<Vault>,

    #[account(
        mut,
        seeds = [b"sol", authority.address().as_ref()],
        bump = state.sol_bump, // reuse the stored canonical bump, no runtime search
    )]
    pub sol_vault: SystemAccount,

    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct Deposit {
    #[account(mut)]
    pub authority: Signer,
    #[account(
        mut,
        seeds = [b"vault", authority.address().as_ref()],
        bump = state.bump,
    )]
    pub state: Account<Vault>,
    #[account(
        mut,
        seeds = [b"sol", authority.address().as_ref()],
        bump = state.sol_bump,
    )]
    pub sol_vault: SystemAccount,
    pub system_program: Program<System>,
}
```

O `address = state.owner` na authority prende o signatário à chave exata que o livro-razão armazenou no init, então um chamador não consegue apresentar o livro-razão de outra pessoa. Este é o substituto baseado em expressão do V2 para o `has_one` depreciado do v1, e é a mesma disciplina de "prender toda chave armazenada" das lições de constraints; num caminho de custódia ele não é opcional.

Repare em qual constraint *não* está em `Withdraw`, porque a lição passada fez questão disso. `quarters::min_balance = 100` continua no seu programa e continua em `require_funded`, e está deliberadamente ausente aqui. Um piso que bloqueasse withdrawals prenderia os últimos 100 credits de um jogador no vault para sempre, que é o oposto de uma garantia de custódia. É essa a metade honesta de "o constraint vem de graça em qualquer instrução que carrega o vault": ele vem junto nas instruções em que você põe ele, e decidir onde continua sendo uma questão de julgamento. O `require_funded` trava o *gasto*; o `withdraw` devolve o dinheiro do próprio jogador e trava só contra os lamports reais do vault.

Checkpoint dos passos 2 até o 6: rode `anchor build`. Ele compila limpo, sem aviso de depreciação de `has_one`, sem erro de campo faltando em `sol_bump`, e sem nomes de campo `player`/`vault` sobrando nas structs de derive nem nos builders de teste. Uma nota de runtime para depois: qualquer vault que você inicializou numa lição anterior vem de antes do `sol_bump` e de antes do PDA de SOL por completo, então o `sol_bump` armazenado dele é o que estivesse naqueles bytes de preenchimento e o SOL vault dele não existe. O tamanho e o conteúdo de uma conta são fixados no init, então aqueles vaults velhos não são atualizáveis aqui. Os testes abaixo criam novos, que é o único caminho que esta lição suporta.

**7. Escreva o teste do LiteSVM.** O LiteSVM roda o programa in-process sem validador, então o loop é rápido. Acrescente as dev-dependencies:

```toml
# The same one row as module 3, for the same reason: anchor-v2-testing owns the SVM
# version. At tag v2.0.0-rc.1 that is litesvm 0.11.0, and you never say so yourself.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
```

O teste financia o SOL vault com `deposit`, saca uma parte dele e comprova que os lamports se moveram e que o livro-razão caiu, depois comprova que um over-withdraw é rejeitado de forma limpa em vez de dar panic. O mover e o rejeitar são o artefato inteiro:

```rust
use anchor_lang::{
    prelude::Address,
    programs::System,
    solana_program::instruction::{AccountMeta, Instruction},
    Id, InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction};

fn ix(program_id: Address, accounts: Vec<AccountMeta>, data: Vec<u8>) -> Instruction {
    Instruction { program_id, accounts, data }
}

// One place that turns an instruction into a signed, sendable transaction.
fn tx(svm: &LiteSVM, payer: &Keypair, instruction: Instruction) -> VersionedTransaction {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap()
}

#[test]
fn withdraw_moves_lamports_and_rejects_overdraw() {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();
    let (state_pda, _) =
        Address::find_program_address(&[b"vault", authority.pubkey().as_ref()], &program_id);
    let (sol_pda, _) =
        Address::find_program_address(&[b"sol", authority.pubkey().as_ref()], &program_id);

    // init_vault (rewritten in step 2b) creates BOTH PDAs and stores both bumps.
    let init = tx(
        &svm,
        &authority,
        ix(
            program_id,
            quarter_vault::accounts::InitVault {
                authority: authority.pubkey(),
                state: state_pda,
                sol_vault: sol_pda,
                system_program: System::id(),
            }
            .to_account_metas(None),
            quarter_vault::instruction::InitVault {}.data(),
        ),
    );
    svm.send_transaction(init).unwrap();

    // deposit 2 SOL into the SOL vault (player signs; no PDA signature needed).
    let deposit = tx(
        &svm,
        &authority,
        ix(
            program_id,
            quarter_vault::accounts::Deposit {
                authority: authority.pubkey(),
                state: state_pda,
                sol_vault: sol_pda,
                system_program: System::id(),
            }
            .to_account_metas(None),
            quarter_vault::instruction::Deposit { amount: 2_000_000_000 }.data(),
        ),
    );
    svm.send_transaction(deposit).unwrap();
    let funded = svm.get_account(&sol_pda).unwrap().lamports;

    // withdraw 1 SOL: the PDA-signed CPI must move lamports OUT of the SOL vault.
    let withdraw = |svm: &LiteSVM, amount: u64| {
        tx(
            svm,
            &authority,
            ix(
                program_id,
                quarter_vault::accounts::Withdraw {
                    authority: authority.pubkey(),
                    state: state_pda,
                    sol_vault: sol_pda,
                    system_program: System::id(),
                }
                .to_account_metas(None),
                quarter_vault::instruction::Withdraw { amount }.data(),
            ),
        )
    };
    svm.send_transaction(withdraw(&svm, 1_000_000_000)).unwrap();
    let after = svm.get_account(&sol_pda).unwrap().lamports;
    assert_eq!(funded - after, 1_000_000_000, "1 SOL must leave the vault");

    // over-withdraw: asking for more than the vault holds must ERROR, not panic.
    let overdraw = svm.send_transaction(withdraw(&svm, 100_000_000_000));
    assert!(overdraw.is_err(), "an over-withdraw must be rejected cleanly");
}
```

**8. Faça build e rode.**

```bash
anchor test
```

Saída esperada, o único teste passando que limpa a barra:

```
running 1 test
test withdraw_moves_lamports_and_rejects_overdraw ... ok

test result: ok. 1 passed; 0 failed
```

A prova está nos dois asserts. O primeiro mostra que exatamente um SOL saiu do vault, o que só pode acontecer se o programa assinou a transferência como o PDA, porque o SOL vault não tem chave e o jogador não é dono dele. O segundo mostra que o over-withdraw voltou como `is_err()`, a trava rejeitando ele antes da CPI, não um panic no log. Se em vez disso o over-withdraw *dá panic* em vez de dar erro, a causa de sempre é a ordem da trava: você tem de comprovar `amount <= vault_lamports` antes do `vault_lamports - amount`, ou a subtração dá underflow dentro da própria trava. Rode de novo o rascunho do passo 3 para isolar a matemática da fiação.

## Challenge

Um degrau você repõe de memória, outro você constrói do zero.

**Completion.** Reabra o `withdraw` e apague as duas linhas que o Lab escreveu para você: o array `signer_seeds` e o débito de `checked_sub`. Deixe `let signer_seeds: &[&[&[u8]]] = /* TODO */;` e `ctx.accounts.state.credit = /* TODO */;`. Reponha as duas de memória. As seeds têm de ser as seeds do próprio SOL vault mais o bump *armazenado* dele copiado antes para um local, `&[&[b"sol", owner.as_ref(), &[sol_bump]]]`, e o débito tem de ser `checked_sub(amount).ok_or(VaultError::Underflow)?`. Se você apelar para o `find_program_address` para pegar o bump, pare: o ponto inteiro é o byte armazenado, e recalcular ele custa CU e arrisca um bump não canônico. A checagem de aceitação é o `anchor test` do passo 8, ainda verde.

**Solo.** Extraia a trava para uma função autônoma e testável, `resolve_withdrawal`, e comprove ela em Rust puro antes de ligar ela de volta no handler. Esta é a trava pré-CPI, destilada para poder ser testada em unidade sem framework nenhum:

![Um fluxograma de decisão devolvendo menos um para um pedido zerado, menos dois para um over-withdraw, menos três abaixo do piso de aluguel, e o valor pedido caso contrário.](assets/v09-flowchart.webp)

O starter, a solution e os vetores de teste moram em `lessons/m04-l1/resolve-withdrawal/`, ao lado dos outros challenges deste curso. O starter ignora toda trava e devolve `requested` sem condição nenhuma — e porque a função é uma `const fn` com asserções em tempo de compilação embaixo dela (o dispositivo da m03-l3, que é o que a avaliação só-de-compilação de fato impõe), a versão sem trava nem faz build: o primeiro erro nomeia o caso do pedido zerado. Uma verruga deliberada para reparar em vez de copiar: a assinatura devolve sentinelas `i64` porque uma função pura sem framework em escopo não tem `VaultError` para devolver, e os vetores ficam pequenos o bastante para o cast `as i64` ser exato. No handler ela vira um `Result` com os erros tipados do passo 4, e se você um dia se pegar entregando códigos sentinela para fora de código de programa de verdade, é esse o cheiro de que a seção de erros tipados do módulo 1 falava. Aceitação: os casos de check passam em ordem, o over-withdraw devolve `-2` em vez de dar underflow, o piso de aluguel é *inclusivo* então um withdrawal que deixa exatamente `rent_exempt_min` é permitido e um lamport a menos é `-3`, um over-withdraw que também romperia o piso continua sendo `-2` porque a checagem de saldo é alcançada primeiro, e — a parte que importa — a sua subtração é `checked_sub` em vez de um `-` pelado, exatamente como no handler — mesmo que a trava acima dela já tenha comprovado `requested <= balance`, porque aquela prova está a uma refatoração de distância de estar errada e um `-` pelado ou aborta a transação ou, num build com overflow-checks desligado, dá wrap em silêncio. Depois troque os três `require!`s do `withdraw` por uma chamada ao seu valor resolvido e confirme que o `anchor test` continua verde. Uma coisa que vale ficar de olho: o `resolve_withdrawal` trava o movimento de *lamports* contra o saldo do SOL vault, enquanto o `checked_sub` trava o *livro-razão*. São dois saldos diferentes fazendo dois trabalhos diferentes, e um bug de custódia de verdade é deixar os dois se descolarem.

Você fez o vault fazer a única coisa que ele não conseguia fazer antes: pagar alguém de volta, sob a autoridade do próprio programa, com uma assinatura que nenhum atacante consegue forjar porque não existe chave para roubar. Você construiu a trava, você assinou a CPI como o PDA com um bump armazenado, e você debitou o livro-razão com matemática que se recusa a dar underflow. É esse o loop de custódia, fechado.

O seu withdrawal funciona. Mas tem uma armadilha nele que o v1 armou para milhares de programas, e você só não bateu nela porque o Lab nunca releu o saldo do vault logo depois da CPI. No v1, ler os campos de uma conta desserializada *depois* de uma CPI ter mutado ela te dava dados velhos a não ser que você lembrasse de chamar `.reload()`, e esquecer era um jeito clássico de entregar um bug. No V2 o modelo de borrow do `CpiHandle` que você usou hoje é o que faz esse mesmo erro se recusar a compilar. Próxima lição: exatamente como, e por que o compilador agora é a coisa que te mantém seguro em vez de uma chamada `.reload()` que você tinha de lembrar.
