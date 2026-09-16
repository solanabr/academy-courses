# PDAs e bumps canônicos no V2

Na lição passada você fechou o modelo de estado: campos fixos e limitados com cast direto dos bytes como Pod, os genuinamente ilimitados isolados atrás de `BorshAccount<T>`. Você consegue descrever qualquer conta que este fliperama precise. O que você ainda não consegue é dizer como o runtime *encontra* uma delas, ou quem tem direito de tocar nela.

Você já usou a resposta sem que ninguém te dissesse o nome dela. O `Cabinet` que você construiu no começo do módulo 2 carregava `seeds = [b"cabinet", player.address().as_ref()]` e um `bump` pelado, e eu te disse na lata para copiar essas duas linhas e esperar. No fim daquele módulo o cabinet tinha ganhado uma `authority` e um `bump` armazenado, e as seeds tinham se mudado para `cabinet.authority`, que você também digitou na confiança. Tudo aquilo passou como encantamento: você digita, e a conta aparece num endereço que você nunca escolheu. Hoje o encantamento vira um mecanismo sobre o qual você consegue raciocinar — e você vai ver com precisão qual fatia dele o V2 de fato mudou, que é uma fatia mais fina do que os posts de blog do ecossistema afirmam.

O fliperama precisa da versão completa agora. Os jogadores estão a ponto de pré-pagar por créditos, e esses créditos ficam num vault por jogador. Você não pode entregar a cada jogador um keypair para o vault dele, porque aí o *jogador* controla o vault, não o programa. E você não pode manter uma tabela de "jogador -> endereço do vault" em algum lugar, porque essa tabela é mais uma coisa para corromper, migrar e pagar rent. O que você quer é um endereço que o programa re-deriva do zero, sozinho, a partir da chave do jogador, toda vez, para sempre.

É essa a razão inteira de os endereços derivados de programa existirem. Antes do porquê, confirme que o toolchain que vai gerar os seus bumps é o do V2. Você instalou isso lá em m01-l2, então esta é uma checagem de re-pin, não uma instalação nova:

```bash
anchor --version   # must report a 2.0.0 RC line, not 1.x
# If it reports 1.x or nothing, re-pin. avm (the Anchor Version Manager) cannot
# install the V2 RC: it downloads a prebuilt binary from a published GitHub
# release, and no release was cut for the v2 tag, so the fetch 404s;
# so the documented channel is a cargo git install, pinned to the RC's tag:
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
```

Uma freshness note antes de você se apoiar nesse pin: o Anchor V2 está em `2.0.0-rc.1` enquanto eu escrevo isto, publicado no crates.io em 2026-08-12. É um *release candidate*, o que quer dizer que a API ainda pode mudar entre RCs. Repare no que "pin" quer dizer aqui, porque as duas metades vêm de lugares diferentes de propósito. A CLI é uma instalação git fixada em `v2.0.0-rc.1`, uma tag, que é um ponto fixo e não uma ponta de branch que se move; m01-l2 instalou a partir do próprio branch `anchor-next` porque o canal era o assunto daquela lição, e tudo depois dela fixa a tag. A biblioteca no `Cargo.toml` que você escreve no passo 1 é a versão do crates.io, `anchor-lang = "2.0.0-rc.1"`, que é mais forte ainda, porque o registro proíbe republicar uma versão. Se você quiser nomear o toolchain com precisão ainda maior, a tag resolve para o commit `e4878b6d`, e o Dockerfile de verify de m08-l2 fixa exatamente esse. De todo jeito, reconfira a tag antes de começar uma sessão. O MSRV é Rust 1.89.0, então garanta que o seu `rustc` também atende a isso.

## Resumo

Aqui está a lição inteira como um índice de achados, cada linha uma conclusão sobre a qual você consegue agir:

- Um PDA é uma conta de propriedade do programa cujo endereço é derivado deterministicamente a partir das **seeds mais o program ID**, e que cai deliberadamente **fora** da curva Ed25519 para que nenhuma chave privada possa jamais assinar por ele.
- O **bump** é o byte extra que empurra para fora da curva um ponto que seria on-curve. O bump **canônico** é o primeiro (buscando de 255 para baixo) que cai fora da curva. Existe exatamente um, e é o único que você deveria usar, sempre.
- O bump canônico chega até você por meio de uma **struct `bumps` tipada que a macro gera em tempo de expansão**, lida como `ctx.bumps.vault` — e isso é *baseline*, não um delta do V2: o Anchor 0.29 substituiu o antigo mapa com chave de string `ctx.bumps.get("vault").unwrap()` dois majors atrás, e a linha 1.x lê `ctx.bumps.vault` exatamente como o V2 lê. A novidade real do V2 em bumps é mais estreita: quando toda seed é um literal de tempo de compilação, o bump canônico é pré-computado em tempo de macro como um const. Seeds que incluem um valor de runtime, como a chave do jogador deste vault, ainda derivam durante a validação em qualquer uma das linhas.
- Para um PDA de propriedade do programa, o V2 **pula a checagem de on-curve**, uma economia reportada pelo projeto de mais ou menos 1,000 CU por verify (changelog do Anchor V2). Re-meça isso contra o seu próprio build.
- Você **persiste o bump canônico** na conta no init, para que instruções posteriores re-derivem o mesmo endereço com `bump = vault.bump` e nunca paguem por uma busca em tempo de execução.
- O **esquema de seeds é a fronteira de segurança**. Uma seed que falta ou é controlada pelo usuário é um bug de colisão ou de spoofing, não uma frescura de estilo.

O recuo da ajuda de hoje: no Lab eu te entrego cada linha do quarter-vault. No problema de completion eu tiro de volta o array `seeds` e o binding de `bump`, e você reabastece os dois. No challenge solo você recebe uma spec e um formato de seed, e você escreve o handler, a struct do derive e o teste sem código nenhum na sua frente.

## Como um programa é dono de um endereço que ele nunca teve

Comece pela coisa contra a qual um PDA é definido, porque o contraste faz a maior parte do ensino. Uma conta normal da Solana tem um keypair: uma chave privada e a chave pública derivada dela. A chave pública é um ponto na curva Ed25519, e a chave privada é o que te deixa assinar. Dono da chave privada, dono da conta.

Um endereço derivado de programa é o oposto deliberado. Você pega algumas **seeds** (slices de bytes arbitrários que você escolhe) e o ID do próprio programa, faz o hash das duas coisas juntas, e checa se o resultado cai na curva. Se cai, aquele endereço *poderia* ter uma chave privada correspondente por aí, que é exatamente o que você não quer para uma conta que o seu programa precisa controlar unilateralmente. Então você rejeita ele e tenta de novo com um ajuste pequeno, até chegar num endereço que comprovadamente não tem chave privada. Esse é um endereço fora da curva, e é esse o truque inteiro: um programa pode receber autoridade para assinar por um endereço fora da curva justamente *porque* nenhum keypair pode.

As seeds são a entrada que você projeta. O program ID limita o escopo da derivação ao seu programa (outro programa com código diferente não consegue derivar para dentro do seu namespace). A saída é um endereço de 32 bytes que é uma função pura dessas entradas.

![Uma comparação de três linhas mostrando que keypairs por vault vazam, uma conta de registro acrescenta rent e custo de migração, e a derivação de PDA não guarda nada porque o mapeamento é a computação.](assets/v01-comparison.webp)

Contraste isso com as duas coisas que um programa poderia fazer no lugar, porque é a comparação que faz o PDA finalmente se encaixar. Ele poderia gerar um keypair por vault e enfiar o segredo em algum canto, o que quer dizer que o programa agora é o custodiante de milhares de segredos e um único vazamento drena todo mundo de uma vez. Ou ele poderia manter uma conta de registro mapeando cada jogador para o endereço do vault dele, o que quer dizer mais uma conta para alocar, pagar rent, manter consistente sob concorrência e migrar toda vez que o schema muda. O PDA colapsa esses dois problemas em aritmética. Não existe segredo para vazar porque não existe segredo, e não existe registro para corromper porque o mapeamento *é* a derivação. O programa re-computa o endereço de qualquer vault sob demanda a partir de entradas que ele já tem.

![Seeds, program ID e um byte de bump passam pelo hash; um resultado on-curve é rejeitado e o bump decrementa; um resultado fora da curva se torna o PDA, e nenhum keypair é gerado em momento algum.](assets/v02-diagram.webp)

### O bump, e por que só um deles conta

Aquele "ajuste pequeno" é o bump. É um único byte acrescentado às suas seeds antes do hash. A derivação começa no bump `255` e caminha para baixo: `255`, `254`, `253`, e assim por diante. Em cada valor ela faz o hash e checa a curva. Estatisticamente, mais ou menos metade de todos os pontos candidatos cai fora da curva, então você quase sempre acerta nas primeiras tentativas. O **primeiro** bump que produz um endereço fora da curva é o **bump canônico**, e por convenção é o único válido.

Ajuda visualizar o que "fora da curva" quer dizer de fato. Ed25519 é uma curva elíptica específica, e uma chave pública válida é um ponto que fica sobre ela. Mais ou menos metade de todos os valores de 32 bytes são pontos válidos da curva e metade não são. Uma chave normal de carteira é, por construção, um dos pontos *sobre* a curva, porque foi gerada a partir de um escalar privado que mora naquele ponto. Um PDA é o inverso: você segue fazendo hash até cair num dos valores que *não* é um ponto da curva, que é exatamente o conjunto de endereços que nenhuma chave privada consegue produzir. Pegue uma jogadora concreta, a Ana, cujo endereço é `An4...k2`. O runtime faz o hash de `[b"vault", An4...k2, 255, program_id]` e, digamos, cai na curva. Rejeita, cai para `254`, faz o hash de novo, ainda on-curve, rejeita, cai para `253`, e desta vez o ponto está fora da curva. Então `253` é o bump canônico da Ana, os 32 bytes resultantes são o vault da Ana, e os dois são uma função pura da chave dela e de nada mais. Rode a derivação idêntica na semana que vem, no ano que vem, de uma máquina fria que não guarda estado nenhum, e você recebe o mesmo endereço toda vez. Essa permanência é a feature inteira, não um efeito colateral dela.

Por que "só um conta" importa tanto? Porque outros bumps mais abaixo na lista podem *também* produzir endereços fora da curva. Esses são PDAs reais e deriváveis para as mesmas seeds. Se o seu programa aceita qualquer bump que o chamador entregar, um atacante consegue apresentar um PDA *diferente*, não canônico, para o mesmo vault lógico, semear ele com o estado próprio dele, e passar por baixo de uma checagem que só verificava "este é um PDA válido para estas seeds." Essa é a família de bugs de substituição de conta. A defesa é simples demais: use sempre o bump canônico, e nunca confie num bump que chegou como entrada não confiável.

![A derivação tenta o bump 255 para baixo; o primeiro bump que produz um endereço fora da curva é o canônico, e qualquer bump fora da curva mais baixo é um PDA não canônico que um atacante poderia substituir.](assets/v03-table.webp)

### A struct `bumps` tipada — uma recapitulação, e o delta que o V2 de fato acrescenta

Primeiro, crédito a quem merece, porque atribuir isso errado é o erro que metade dos posts de migração do ecossistema comete. No Anchor *antigo* — 0.28 e anteriores — a macro encontrava o bump canônico durante a validação das contas e jogava ele num mapa com chave de string. Você pescava ele de volta com `ctx.bumps.get("vault").unwrap()`. Duas coisas naquela linha envelheceram mal. Primeira, `"vault"` é uma string, então um typo compila numa boa e explode em tempo de execução. Segunda, `.get(...)` retorna um `Option`, então você dá `.unwrap()` num valor que o compilador não consegue comprovar que existe.

O Anchor 0.29 matou aquele mecanismo — dois majors antes de o V2 existir. De lá para cá, o `#[derive(Accounts)]` gera, em tempo de expansão, uma struct `bumps` tipada com um campo por conta PDA no seu derive, lida como um campo: `ctx.bumps.vault`. Sem string, sem `Option`, sem `.unwrap()`. Erre o nome do campo e o programa não compila. No baseline 1.x contra o qual este curso mede o V2, `ctx.bumps.get("vault")` não é "o jeito antigo" — ele não faz build de jeito nenhum. Então quando você lê `ctx.bumps.vault` no código abaixo, você está olhando para continuidade, não para mudança: o RC mantém a struct tipada exatamente como a 1.x tem.

O que o V2 *de fato* acrescenta fica embaixo, e vale ser preciso porque esta é a frase que as pessoas erram. Quando as seeds de um PDA são todas literais de byte de tempo de compilação, a macro do V2 pré-computa o bump canônico em tempo de expansão como um const, e a validação nunca busca nada. Essa otimização não consegue se aplicar a este vault: as suas seeds incluem `player.address()`, e nenhum compilador sabe qual jogador vai chamar, então o framework cai de volta em derivar durante a validação, exatamente como a 1.x faz. O que quer dizer que a história de CU aqui é inteiramente sobre qual constraint você escreve, em qualquer uma das linhas. O `bump` pelado roda a busca. O `bump = vault.bump` roda uma derivação contra um valor que você já armazenou. Essa diferença é o compute; a struct tipada é a segurança de tipos, e você tinha as duas antes do V2.

Se você está portando um programa 1.x, a linha do bump é a que *não* muda: `let bump = ctx.bumps.vault;` lê idêntico nos dois lados. A reescrita mecânica mora em outro lugar, e vale fazer à mão uma vez para que ela grude na memória muscular. Todo constraint de seed que lia `player.key().as_ref()` passa a ser `player.address().as_ref()`, já que `Pubkey` agora é `Address` e `.key()` agora é `.address()`. A assinatura do handler perde o lifetime `<'info>` e ganha um `&mut`, então `pub fn init(ctx: Context<Init>)` passa a ser `pub fn init(ctx: &mut Context<Init>)`. Os tipos de conta também largam os lifetimes deles, então `Account<'info, Vault>` colapsa para `Account<Vault>`. Nada disso é cosmético; cada edição entrega ao compilador uma garantia que ele não tinha antes. (Se o trecho que você herdou é de verdade antiquíssimo — da era 0.28, mapa de string e tudo — então `ctx.bumps.get("vault").unwrap()` de fato passa a ser `ctx.bumps.vault`, mas essa é uma migração de 0.29 que você está pagando atrasado, não uma de V2.)

![O Anchor pré-0.29 lia o bump por uma busca por string em tempo de execução, falível e capaz de dar panic; de 0.29 em diante, V2 incluído, ele lê um campo de struct tipado cuja fiação é resolvida em tempo de expansão de macro, então typos não compilam, enquanto uma derivação semeada em tempo de execução ainda roda durante a validação em toda linha.](assets/v04-comparison.webp)

Existe uma segunda economia escondida embaixo da mesma reescrita, e vale nomear com precisão porque é fácil exagerar nela. Para uma conta de que o *seu próprio programa é dono*, o V2 pula a checagem de on-curve que uma verificação de endereço genérica rodaria. O changelog do Anchor V2 reporta isso como mais ou menos 1,000 CU economizados por verify. Trate isso como um número reportado pelo projeto, não como uma lei da natureza: meça contra o seu próprio build, e reconfira quando você subir o RC, porque números de compute vão mudando entre release candidates. O raciocínio atrás desse pulo é limpo, que é por isso que ele é seguro. Se o endereço foi derivado pelo seu programa a partir de seeds e de um bump, e você está tratando ele como de propriedade do programa, então se ele por acaso fica sobre a curva é informação de que você não precisa. Você já sabe que ele é seu. Pagar compute para re-responder uma pergunta que você já respondeu é o tipo de desperdício que uma reescrita do zero existe para deletar.

![Uma barra de antes/depois mostra que pular a checagem de on-curve num PDA de propriedade do programa economiza mais ou menos 1,000 CU por verify, rotulada como um número reportado pelo projeto, para re-medir.](assets/v05-chart.webp)

De onde isso veio? Não é uma otimização isolada que alguém aparafusou. O Anchor V2 é uma reescrita `no_std` do zero construída sobre o pinocchio, o framework de contas mínimo e sem dependências, descrito exatamente assim no próprio crate lang-v2 em 2026-08. A mesma reescrita que tornou possíveis os bumps const-folded é a que está sinalizada no manifesto `#4390` "zero-copy by default" (otter-sec/anchor#4390), que reenquadra o antigo `Account<T>` desserializado com borsh como *o caminho lento* e empurra o zero-copy para o default. Bumps const e pulos de checagem de on-curve só estão na mesa porque alguém derrubou o framework até as vigas. Essa é a cor que vale levar para o Lab: a struct Pod que você construiu no módulo 2 não é mais um caso especial. No V2, ela é o veio da madeira.

### A decisão de custódia, dita em voz alta

Uma nota de escopo antes do código, porque isso vai te morder depois se ficar implícito. O quarter-vault que você está a ponto de construir guarda **SOL nativo**, como lamports, direto na conta. Não tokens. Créditos são fichas, fichas são lamports, por enquanto. Isso mantém esta lição sobre PDAs e nada mais. O upgrade para SPL, onde o vault é promovido a guardar um saldo de token de verdade, aterrissa no módulo 5. Se você se pegar indo atrás de uma conta de token hoje, pare: essa é uma lição posterior vazando para dentro desta.

E o trade-off, dito sem enfeite porque é a parte honesta. PDAs te dão endereçamento determinístico sem keypair para proteger e sem tabela de lookup para manter. O que você paga por isso é *responsabilidade de design de seeds, para sempre*. O esquema de seeds não é uma convenção de nomes, é o namespace e a fronteira de controle de acesso ao mesmo tempo. Acerte nele e cada jogador tem um vault isolado e re-derivável. Faça ele com preguiça, e você tem um bug de colisão que nenhuma quantidade de código posterior consegue encobrir.

![Com apenas a seed b"vault" cada jogador deriva um único PDA compartilhado, mas acrescentar o endereço do jogador dá a cada jogador um vault distinto e re-derivável.](assets/v06-diagram.webp)

Três jeitos de isso morder, nomeados agora para você reconhecer eles antes que custem algo a você. Primeiro, re-derivar o bump em tempo de execução. Se um handler posterior chama `find_program_address` para "pegar um bump fresco," você reintroduziu exatamente a busca que um bump armazenado foi feito para deletar, e você paga por ela em cada chamada. Ponha um número nisso: a referência de constantes da Solana precifica um syscall de derivação de PDA em **1,500 CU** (`create_program_address_units`). Validar contra um bump armazenado custa exatamente um desses. O `find_program_address` paga um *por bump que ele tenta* antes de cair fora da curva, caminhando de 255 para baixo, então a conta é 1,500 CU vezes quantos candidatos as seeds por acaso precisarem. Re-meça no seu próprio build, mas a direção não está em questão: cada tentativa evitada é outras 1,500 CU que você fica, que é por isso que você persiste o bump. Segundo, uma seed controlada pelo jogador ou subespecificada. Qualquer coisa que um chamador consiga influenciar no conjunto de seeds é uma alavanca que ele pode puxar para conduzir uma derivação até uma conta que não é dele, ou até uma conta compartilhada que devia ter sido isolada por jogador. A correção é amarrar identidade nas seeds, que é exatamente o que o endereço do jogador faz. Terceiro, e este é o sutil, tratar a economia do pulo de on-curve como um número fixo contra o qual você pode orçar. É um número reportado pelo projeto, vindo de um release candidate. Projete como se ele pudesse marcar 800 CU ou 1,200 no mês que vem, porque ele pode.

## Lab: construa o R2, o quarter-vault

Você está construindo o `quarter_vault`, um programa Anchor V2 com um só trabalho: criar um vault PDA por jogador, armazenar o owner, o bump canônico e um saldo de credit zerado, e expor um caminho de leitura. Um teste Rust em LiteSVM vai comprovar que o PDA é derivável e que o estado lê de volta. A barra de `verify` para este artefato é um teste passando: o vault deriva de `[b"vault", player]`, inicializa e lê de volta.

**1. Gere o scaffold e fixe o toolchain.** De um diretório vazio:

```bash
anchor init quarter-vault   # the default test template is LiteSVM (Rust tests)
cd quarter-vault
```

Abra o `Cargo.toml` do programa e confirme que a dependência anda no mesmo release de V2 que a CLI. O scaffold escreve uma linha git seguindo o branch `anchor-next`; troque ela pela versão do crates.io, a mesma edição que m01-l2 percorreu. A CLI é uma instalação git e a biblioteca é uma versão de registro, e isso não é descompasso — são grafos de dependência separados nomeando o mesmo release, e a versão do registro é a que não consegue se mover debaixo de você:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io; the branch tip wants solana-address 2.7.0
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row from m02-l1, verbatim: a ceiling, not an equality. One
# workspace resolves one solana-address, and module 6's Mollusk dev-dep needs ^2.6.1.
solana-address = ">=2.6.1, <2.7"
```

Esperado depois deste passo: o `anchor build` tem sucesso no template intocado e o `target/deploy/quarter_vault.so` existe. Se o build falhar aqui, é um problema de toolchain, não um problema de PDA, e consertar agora te poupa de depurar a camada errada no passo 5.

**2. Defina o estado do vault.** Esta é a struct Pod, o mesmo formato que você conhece do módulo 2, agora destinada a um endereço derivado de programa. Ponha ela em `programs/quarter-vault/src/lib.rs`. Repare nos tipos dos campos: no V2, `Pubkey` é `Address`.

```rust
use anchor_lang::prelude::*;

// Leave the id `anchor init` generated here. It matches the keypair in
// target/deploy/, and a hand-typed string gives you an id the deploy cannot
// sign for. `anchor keys sync` re-aligns them if they ever drift.
declare_id!("<your generated program id>");

#[account]
#[repr(C)]
#[derive(InitSpace)]
pub struct Vault {
    pub owner: Address,  // 32 bytes: the player who owns this vault
    pub credit: u64,     //  8 bytes: prepaid credit, native lamports for now
    pub bump: u8,        //  1 byte: the canonical bump, persisted for reuse
    pub _pad: [u8; 7],   //  7 bytes: explicit tail padding, zeroed, never read
}
```

Esperado depois deste passo: o `anchor build` continua tendo sucesso. A struct compila por conta própria, antes de qualquer constraint se referir a ele, que é o lugar mais barato possível para pegar um erro de tipo de campo.

Aquele campo `_pad` é a disciplina do módulo 2 chegando numa conta de verdade, então não delete ele. `credit` é um `u64` nativo, que carrega um alinhamento de 8 bytes, então o Rust arredonda a struct inteira para cima até um múltiplo de 8: 41 bytes de campos se tornam 48 bytes de layout, e esses sete bytes existem tenha você nomeado eles ou não. Sem nome, eles são preenchimento implícito e o bound Pod rejeita a struct. Nomeados e zerados, eles são um campo como qualquer outro e o cast é sólido. Escreva o preenchimento que você já está pagando.

O `#[derive(InitSpace)]` computa o tamanho de dados da conta para você, então você nunca conta à mão. Armazenar o `bump` na conta é a disciplina sobre a qual a lição inteira gira: você computa o bump canônico exatamente uma vez, no init, e reusa o valor armazenado em todo lugar depois.

Um detalhe do V2 vem de carona na linha `space` que você está a ponto de escrever. O V1 te ensinou a somar um `8` mágico para o discriminator da conta, a tag que o Anchor escreve na frente de toda conta para conseguir distinguir um `Vault` de um `Config` quando lê bytes crus. O V2 te impede de deixar isso hardcoded: você escreve `Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE`, e se o esquema de discriminator algum dia mudar debaixo de você, a sua matemática de space muda junto em vez de silenciosamente ficar errada. (O V2 até infere a linha `space` inteira a partir do `INIT_SPACE` do wrapper se você omitir ela; um `space =` explícito ainda é aceito, e escrever ele uma vez vale pela prática de ver de onde o número vem.) É o mesmo instinto do bump virando um const. Pare de carregar à mão números que o framework está disposto a te entregar.

![O constraint init pareia um array de seeds de b"vault" mais o endereço do jogador com um bump pelado, então a macro deriva e armazena o bump canônico.](assets/v07-annotated-code.webp)

**3. Escreva a struct do derive e o handler de init.** Handlers do V2 recebem `&mut Context<T>` e a struct de accounts não carrega lifetime `<'info>`. Os dois são consequências da reescrita em pinocchio. Acrescente isto ao `lib.rs`:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn init_vault(ctx: &mut Context<InitVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = *ctx.accounts.player.address();
        vault.bump = ctx.bumps.vault; // canonical bump, read as a field off the typed bumps struct
        vault.credit = 0;
        Ok(())
    }

    pub fn read_vault(ctx: &mut Context<ReadVault>) -> Result<()> {
        let vault = &ctx.accounts.vault;
        msg!("vault credit={} bump={}", vault.credit, vault.bump);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitVault {
    #[account(mut)]
    pub player: Signer,
    #[account(
        init,
        payer = player,
        space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
        seeds = [b"vault", player.address().as_ref()],
        bump,
    )]
    pub vault: Account<Vault>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct ReadVault {
    pub player: Signer,
    #[account(
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump, // reuse the STORED bump, no runtime search
    )]
    pub vault: Account<Vault>,
}
```

Esperado depois deste passo: o `anchor build` compila os dois handlers e as duas structs de derive. Seja preciso sobre qual typo este passo pega, porque só um dos dois é um erro de build. Erre o *campo* na struct bumps — `ctx.bumps.valut` — e o compilador rejeita ele, porque o derive gerou uma struct com um campo por conta PDA e não existe tal campo. Erre o *conteúdo da seed* — `b"valt"` em vez de `b"vault"` — e ela compila perfeitamente, porque uma byte string é tão válida quanto outra; essa aí falha em tempo de execução como uma violação do constraint seeds, que é exatamente a falha que o passo 5 te faz ler. A struct bumps tipada empurra o primeiro typo para a esquerda. Ela não tem nada a dizer sobre o segundo.

Olhe firme para a diferença entre as duas linhas de `bump`, porque é esse o ponto de todo o build. Em `InitVault` você escreve `bump` pelado, que diz à macro para encontrar o bump canônico e te entregar ele por meio de `ctx.bumps.vault`. Em `ReadVault` você escreve `bump = vault.bump`, que diz à macro para pular a busca inteira e validar contra o valor que você já armazenou. A primeira é compute que você paga uma vez. A segunda é compute que você nunca paga de novo.

![O cliente deriva o PDA e envia init_vault; a macro re-deriva ele, fornece o bump canônico, cria e financia a conta, e então o handler armazena owner, bump e credit.](assets/v08-flowchart.webp)

**4. Escreva o teste em LiteSVM.** O LiteSVM roda o programa no mesmo processo, sem validador, então o loop é rápido. Você chega nele por meio do `anchor-v2-testing`, a bancada de teste contra a qual o scaffold do V2 gera, e **não** por meio de uma dependência direta de `litesvm`:

```toml
# One row, not three. anchor-v2-testing wraps LiteSVM and re-exports the pieces a
# test file needs, so the SVM version is the harness's problem and not yours. At tag
# v2.0.0-rc.1 it carries litesvm 0.11.0; crates.io's latest is 0.16.0 as of
# 2026-09-07, and the anchor-next branch tip has moved it to 0.13.1. Name litesvm
# yourself and you are choosing one of those against a harness that expects
# another — two SVM versions in one graph, failing in a way that reads like your
# test is wrong.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
bytemuck = "1.25"     # to cast the account bytes back to the Pod state
```

Depois o teste em si, em `tests/quarter_vault.rs`. Ele deriva o PDA do mesmo jeito que o programa deriva, envia `init_vault`, e lê a conta crua de volta para afirmar o estado armazenado. Repare de onde vem cada import: o encanamento de instrução anda no `anchor_lang`, o encanamento de assinatura e de SVM anda no `anchor_v2_testing`, e nada alcança além desses dois:

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{Keypair, Message, Signer, VersionedMessage, VersionedTransaction};
use bytemuck::from_bytes;

#[test]
fn quarter_vault_pda_derives_and_reads_back() {
    // `svm()` is LiteSVM::new() plus the profiling hook `anchor test --profile` turns on.
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    // cargo runs a test binary with its working directory at the package root, so a bare
    // "target/deploy/..." would resolve inside programs/quarter-vault/ and miss. Anchor the
    // path on the crate instead: the artifact lives in the workspace target dir, two up.
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    // A player who will pay rent and own the vault.
    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();

    // Derive the PDA exactly as the program does: [b"vault", player].
    let (vault_pda, expected_bump) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // Build and send init_vault.
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::InitVault {
            player: player.pubkey(),
            vault: vault_pda,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::InitVault {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&player]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Read the raw account and cast the Pod state - no deserialize step, same as
    // module 2. `from_bytes` wants EXACTLY size_of::<Vault>() bytes, and it wants
    // Vault to be Pod, which is what the `_pad` field bought you. And yes, the
    // discriminator offset is spelled out rather than hardcoded as 8: same rule as
    // the `space =` line, for the same reason.
    let raw = svm.get_account(&vault_pda).unwrap();
    let start = quarter_vault::Vault::DISCRIMINATOR.len();
    let vault: &quarter_vault::Vault =
        from_bytes(&raw.data[start..start + core::mem::size_of::<quarter_vault::Vault>()]);

    assert_eq!(vault.owner, player.pubkey().into()); // owner is the player
    assert_eq!(vault.credit, 0); // credit starts at zero
    assert_eq!(vault.bump, expected_bump); // stored bump IS the canonical bump
}
```

Aquela última asserção é a que comprova a disciplina: o bump que você armazenou é igual ao bump canônico que o cliente derivou de forma independente. Sem re-derivação em tempo de execução, mesmo valor.

Agora exercite a outra metade. Acrescente uma segunda instrução ao mesmo teste, `read_vault`, construída do mesmo jeito a partir de `accounts::ReadVault { player, vault: vault_pda }` e `instruction::ReadVault {}`, e envie ela depois do init. Ela não tem asserção a fazer além de ter sucesso, e é esse o ponto: `read_vault` é o caminho que carrega o constraint `bump = vault.bump`, então um envio verde é prova de que o bump armazenado valida o mesmo endereço que a busca encontrou. Deixe ela de fora e a única linha que esta lição existe para ensinar nunca é executada.

Esperado depois deste passo: o arquivo de teste compila (`cargo test --no-run` já basta para checar) mesmo antes de você rodar ele. Um descasamento entre os nomes de conta aqui e os campos da sua struct de derive é um erro de compilação, então uma compilação limpa quer dizer que o cliente e o programa concordam sobre a lista de contas.

**5. Faça o build e rode.**

```bash
anchor test
```

Saída esperada, o único teste passando que limpa a barra:

```
running 1 test
test quarter_vault_pda_derives_and_reads_back ... ok

test result: ok. 1 passed; 0 failed
```

Se em vez disso você vê uma falha do constraint seeds no caminho `ReadVault`, a causa de sempre é derivar com uma ordem de seeds diferente ou com uma chave diferente no teste da que está no programa. As seeds precisam casar byte a byte nos dois lados. Isso não é um bug do Anchor, é o esquema de seeds sendo exatamente tão estrito quanto ele prometeu ser.

## Challenge

Dois degraus, e o apoio afina em cada um.

**Completion.** Abra a struct de derive que você acabou de escrever e apague duas coisas: troque o array `seeds = [...]` por `seeds = [/* TODO */]` e a linha `bump` por `bump, // TODO: canonical or stored?` tanto em `InitVault` quanto em `ReadVault`. Agora reabasteça as duas de memória. A checagem de aceitação: o `init_vault` usa `bump` pelado e deriva de `[b"vault", player.address().as_ref()]`, enquanto o `read_vault` usa `bump = vault.bump` e o *mesmo* array de seeds. Se você for atrás de `find_program_address` dentro de um handler, você pegou o caminho errado: a macro já fez esse trabalho.

**Solo.** Dê a um único jogador mais de um vault. Acrescente um argumento `slot: u8` a um novo handler `init_vault_slot` e costure ele nas seeds para que o PDA se torne `[b"vault", player.address().as_ref(), &[slot]]`. Você vai precisar de `#[instruction(slot: u8)]` na struct de derive para que o constraint consiga ver o argumento. Comprove duas coisas num teste LiteSVM: o slot `0` e o slot `1` para o mesmo jogador produzem dois endereços *distintos*, e chamar cada um uma segunda vez re-deriva o *mesmo* endereço que ele deu na primeira (então os dois são estáveis, re-deriváveis, e os bumps armazenados deles casam com o valor que o cliente deriva). Aceitação: os dois vaults inicializam, os dois re-derivam numa segunda chamada, e nenhum dos handlers re-deriva um bump em tempo de execução. Um crédito extra que vale perseguir: tente dar `init_vault` duas vezes para as mesmas seeds e veja a segunda chamada falhar. Essa falha é a trava de conta-já-existe fazendo o trabalho dela, e é a razão pela qual um vault não pode ser silenciosamente re-inicializado por baixo de um jogador.

![Uma linha do tempo desde os bumps com chave de string pré-0.29, passando pela struct bumps tipada do Anchor 0.29, pelo manifesto #4390 zero-copy-by-default e pela reescrita no_std em pinocchio do V2, até os consts de bump em tempo de macro para seeds literais de hoje e o pulo de on-curve para PDAs de propriedade do programa.](assets/v09-timeline.webp)

Quando os dois degraus passarem, sente com o que você de fato comprovou. Dois jogadores ganham dois vaults isolados, um jogador ganha quantos slots quiser, todo endereço re-deriva para os mesmos 32 bytes para sempre, e nenhum deles precisou de um keypair ou de uma tabela de lookup. Você escreveu o esquema de seeds, e o esquema de seeds *é* o modelo de custódia. Esse é o peso sobre o qual a parte honesta avisou, e você carregou ele corretamente.

Repare, também, em quanto o esquema de seeds já comprou para você. Porque o vault é derivado de `player.address()` e o `player` tem que assinar, ninguém consegue inicializar nem ler um vault que não é dele: a derivação *é* a checagem de acesso, de graça, sem nenhum constraint escrito. Isso vale saber com precisão, porque é a fronteira do que as seeds conseguem fazer por você.

E elas param exatamente aí. No momento em que este programa ganhar um *operador* de fliperama, alguém que consiga ajustar credit em todo vault, o esquema de seeds não tem nada a dizer sobre quem é essa pessoa, e o `Signer` comprova apenas que alguém assinou, nunca quem. Na próxima lição você maneja o catálogo de constraints do V2, `address`, `owner`, `constraint`, `close`, `realloc_payer`, e faz a macro de derive rejeitar as contas erradas antes do seu código de handler rodar. O vault ganha um leão de chácara.
