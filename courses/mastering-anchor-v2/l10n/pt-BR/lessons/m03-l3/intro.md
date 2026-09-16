# Escreva o seu próprio constraint: o trait AccountConstraint

Na lição passada você endureceu o vault com os constraints que o framework entrega: `address` como a trava de autoridade, a cilada do `owner` e a solução de contorno com `UncheckedAccount`, `close` para a devolução do rent, e o caminho de reuso do `init_if_needed` com o risco de reinit que sobrevive a ele. Cada um deles era uma keyword que outra pessoa escolheu. O que está tudo bem, exatamente até o ponto em que a regra de que você de fato precisa não está na lista.

Aqui está a regra que eu quero no R2, o quarter-vault que você construiu neste módulo. Um jogador não deveria conseguir rodar certas operações contra um vault cujo `credit` armazenado está abaixo de um piso. Chame ela de `quarters::min_balance = 100`: abaixo de 100, rejeita. Repare *do que* a regra trata, porque senão o nome vai te enganar. Ela lê o contador `credit` na conta, não o saldo de lamports da conta, e até a lição passada esse contador ainda não é lastreado por lamport nenhum. O módulo 4 é onde os dois viram o mesmo número. O piso de hoje é um piso nos livros. Antes do V2 essa regra tinha duas casas e nenhuma delas era boa. Na maior parte das vezes era código de handler: um `if` no topo da instrução, mais a esperança de que toda outra instrução que tocasse o vault escrevesse o mesmo `if`, mais a reza de que ninguém acrescentasse um call site seis meses depois e esquecesse. Às vezes era um `constraint = <expr>` inline, que pelo menos rodava na macro mas era um booleano anônimo que você não conseguia nomear, reusar, nem ler de uma IDL. De qualquer jeito o invariante morava na sua disciplina em vez de no tipo.

Antes de eu argumentar sobre por que esse é um lugar ruim para manter um invariante, faça a coisa que torna o resto concreto. Jogue isto num arquivo de rascunho e rode:

```bash
rustc --version   # any stable rustc at or above Anchor V2's MSRV, Rust 1.89.0
```

```rust
// scratch.rs - the check hook, distilled to plain Rust.
// Deliberately NOT named AccountConstraint / MinBalanceConstraint: the real trait has a
// different shape (static methods, an associated Value, a program error), and
// this file exists only to get the comparison right before the wiring.
pub trait BalanceGate {
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalanceRule {
    pub min: u64,
}

impl MinBalanceRule {
    // The condition, split out as a plain `const fn` this file owns. Not a hook:
    // the trait has one method, `check`. It sits here so the compiler can
    // evaluate it at build time, which the graded version leans on.
    const fn meets_floor(&self, _balance: u64) -> bool {
        // This is the starter: it never rejects anything. That is the bug.
        true
    }
}

impl BalanceGate for MinBalanceRule {
    fn check(&self, balance: u64) -> Result<(), String> {
        if self.meets_floor(balance) {
            Ok(())
        } else {
            Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ))
        }
    }
}

fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalanceRule { min }.check(balance).is_ok()
}

fn main() {
    // A 50-lamport vault against a 100 floor should be REJECTED.
    println!("50 vs 100 -> {}", run_constraint(50, 100)); // prints true. wrong.
    println!("100 vs 100 -> {}", run_constraint(100, 100)); // prints true. correct, by accident.
}
```

```bash
rustc scratch.rs -o scratch && ./scratch
```

Você vai ver `50 vs 100 -> true`. Um vault segurando metade do piso passa batido, porque a condição para a qual o hook encaminha não faz nada. Segure essa linha que falha na cabeça. No fim do Lab este exato `check` é um constraint de verdade, visível na IDL, no R2, disparando antes de o seu handler rodar. O problema de completion te entrega de volta este corpo de hook para preencher; o problema solo pede um segundo constraint sem apoio nenhum. No Lab em si eu percorro cada linha.

## Resumo

- O Anchor V2 despacha qualquer constraint com namespace e não-token (`ns::key = value`) por um trait público, `AccountConstraint<A>`. Você implementa ele para um tipo marcador e ganha uma keyword `#[account(...)]` nova, sem fork nenhum e sem mudança nenhuma na macro de derive.
- O trait expõe quatro hooks de ciclo de vida: `init`, `check`, `update`, `exit`. O codegen chama cada um numa fase específica. Um piso de saldo é uma trava de tempo de leitura na conta carregada, então ele pertence ao `check`, não ao `init` (só na criação) e não ao `exit` (pós-handler).
- Por que um trait e não mais keywords embutidas? Porque nenhum autor de framework consegue enumerar de antemão os invariantes de todo programa. Um trait aberto move essa decisão para você, permanentemente.
- O custo é real e vale nomear de cara: um hook `check` roda em toda instrução que casa, dentro do orçamento de compute daquela instrução. Um hook errado ou um hook caro é CU que você paga num caminho quente para sempre, e ele consegue esconder um bug de lógica atrás de um "passa nos constraints" verde.

O recuo da ajuda de hoje, e é a terceira volta do módulo neste loop, então ele vai um ponto mais longe do que o da lição passada: eu escrevo o impl do trait com você no Lab, mas o vault, as seeds dele, o bump armazenado dele e a bancada de teste em LiteSVM agora são seus para reproduzir sem narração. No problema de completion você reabastece o corpo do `check` de memória. No problema solo você recebe uma spec, nenhum apoio, e você projeta um segundo namespace de constraints você mesmo.

## Por que a superfície de constraints é um trait que você consegue implementar

### Uma porta de entrada de 30 segundos: traits, tipos marcadores, itens associados

Você não precisa ser fluente em Rust para ler o que vem a seguir, só reconhecer três formas.

Um **trait** é um conjunto nomeado de métodos que um tipo promete fornecer. Se um tipo "implementa `AccountConstraint`", ele fornece corpos para os métodos desse trait, e qualquer código escrito contra o trait agora consegue chamar eles. É essa a ideia inteira: o código depende da promessa, não do tipo concreto.

Um **tipo marcador** é uma struct sem campos, `pub struct MinBalanceConstraint;`, que existe só para ter um trait implementado nela. Ela não carrega dado nenhum. É um nome no qual você consegue pendurar comportamento. Quando você vê `impl AccountConstraint<Account<Vault>> for MinBalanceConstraint`, leia assim: "aqui está o que a regra de min-balance faz quando aplicada a um `Vault` carregado".

Um **item associado** é um tipo ou uma constante que pertence a uma implementação de trait em vez de ser passado para dentro. `AccountConstraint` tem um tipo associado `Value`, o tipo do valor à direita do `=`. Para `quarters::min_balance = 100`, `Value` é `u64` e o `100` é esse valor. É genuinamente toda a maquinaria de Rust na qual esta lição se apoia. Trait, tipo marcador, tipo associado. Todo o resto é o argumento de por que eles estão aqui.

![Uma referência de três cartões mapeando o trait AccountConstraint, o tipo marcador sem campos MinBalanceConstraint e o tipo associado Value para a única linha de impl que amarra os três juntos.](assets/v01-diagram.webp)

### O status quo, e o lugar exato em que ele quebra

Comece por como os constraints funcionavam antes do V2, porque o limite é a motivação inteira. Um constraint como `has_one` ou `address` é uma keyword que o autor do framework embutiu na macro de derive. A macro parseia o seu atributo `#[account(...)]`, casa a keyword com uma lista fixa que ela conhece, e emite a checagem correspondente. A lista é fechada. É um dicionário que o autor do framework terminou de imprimir antes de algum dia encontrar o seu programa.

Para os constraints que estão nessa lista, isso é ótimo. `has_one`, `owner`, `seeds`: esses são quase universais, e ter eles como keywords de primeira classe, visíveis na IDL, quer dizer que um cliente lendo a sua IDL consegue ver eles, e o compilador impõe eles do mesmo jeito em toda instrução. O problema é sempre e apenas o constraint que *não* está na lista. E sempre existe um, porque o seu programa tem invariantes que nenhum autor de framework poderia ter previsto: um piso de vault, uma regra de owner-é-um-papel-específico, uma regra de "este contador nunca passa de um teto". Regras de domínio. No momento em que você precisa de uma, a lista fechada não tem nada para você, e você cai de volta em código de handler.

Aqui está a pergunta afiada que força o design. Se `has_one` pode ser um constraint declarativo, de toda instrução, visível na IDL, por que o *seu* invariante deveria ser um `if` escrito à mão no topo de um handler que todo outro call site tem que lembrar de copiar?

### Descartando as respostas fáceis primeiro

As correções óbvias falham, uma por uma, e ver elas falharem é o que constrói a necessidade da correção de verdade.

A primeira resposta fácil: "só escreva a checagem no handler". É aqui que todo mundo começa, e ela tem três modos de falha específicos, não um. Ela não é visível na IDL, então um cliente não tem como saber que a regra existe. Ela não é imposta pelo tipo, então uma segunda instrução que toca o mesmo vault não herda ela. E ela roda *depois* do carregamento das contas e muitas vezes depois de outra lógica, então um bug de ordenação consegue deixar uma conta ruim passar por uma checagem parcial. A regra é real mas ela mora na sua memória, e memória não sobrevive a um colega novo acrescentando um call site.

A segunda resposta fácil: "então acrescente a minha keyword ao framework". Faça um fork do Anchor, acrescente `min_balance` ao parser e ao codegen, pronto. Só que agora você é dono de um fork de um framework, para sempre, e refaz o merge dele a cada release. Pior, a sua regra é específica do *seu* vault; ela não tem nada que fazer no Anchor de todo mundo. Um framework que aceitasse o invariante privado de todo programa como uma keyword embutida desabaria sob uma lista de keywords que ninguém conseguiria ler. Escale a ideia e ela é absurda: uma keyword por programa não é uma linguagem, é um lixão.

A terceira, mais sutil: "faça dela um `constraint = <expr>` genérico". O V2 entrega sim `constraint = <expr>` exatamente para o caso pontual, e para uma condição descartável é a ferramenta certa. Mas é um booleano inline, não uma coisa nomeada, reusável, legível numa IDL. Você não consegue aplicar `constraint = vault.credit >= 100` a uma segunda conta e fazer um leitor ver "ah, essa é a regra de min-balance". Ela não compõe, ela não se nomeia, e ela não aparece como um constraint distinto na interface. É um remendo, não uma primitiva.

Então a pergunta de verdade se estreita para esta: como você deixa um programa acrescentar uma keyword de constraint *nomeada, reusável, visível na IDL* sem tocar no framework e sem fazer um fork dele? Uma vez que a pergunta é tão precisa assim, o mecanismo é quase forçado.

![Uma matriz comparando ifs de handler, fazer um fork do Anchor, expressões de constraint inline e implementar AccountConstraint; só o trait é visível na IDL, imposto pelo tipo em todo call site, livre de fork e reusável de uma vez só.](assets/v02-comparison.webp)

### O mecanismo: despacho por um trait

A resposta do V2 é parar de tratar a lista de constraints como uma tabela fechada de keywords e começar a tratar ela como um trait aberto. Qualquer constraint com namespace, qualquer coisa no formato `ns::key = value` em que `ns` não é um embutido como `token`, despacha por `AccountConstraint<A>`. Um crate downstream, querendo dizer o seu programa ou qualquer biblioteca da qual você depende, implementa esse trait para um tipo marcador, e a macro de derive roteia a keyword nova para ele. Nada na macro central muda. Nada disto é uma escotilha especial aparafusada para um caso só: é a mesma filosofia que faz de `AnchorAccount`, `Id` e `Discriminator` traits públicos no V2: o framework é deliberadamente aberto nas costuras, para que crates downstream consigam entregar wrappers de conta novos, IDs de programa bem conhecidos novos e esquemas de discriminator novos sem um fork (superfície de extensibilidade do lang-v2 e do docs-v2, verificada em 2.0.0-rc.1). Uma fronteira para manter clara: o namespace `token::*` é a exceção embutida, o único namespace que o derive central trata sozinho, para o bem do `anchor-spl`. Todo *outro* namespace, o seu incluído, despacha pelo trait, e esse despacho aberto é a porta pela qual você está a ponto de passar.

Aqui está o trait, verificado contra o fonte do docs-v2 no release candidate 2.0.0-rc.1 (crates.io, publicado em 2026-08-12). Trate o formato exato como um alvo móvel: isto é um release candidate, a superfície de extensibilidade é de alta confiança mas ainda está assentando, então releia ele contra o crate quando você fizer o build:

```rust
pub trait AccountConstraint<A> {
    type Value;
    fn init(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn check(_account: &A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn update(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
    fn exit(_account: &mut A, _value: &Self::Value) -> Result<()> { Ok(()) }
}
```

`A` é o tipo de conta *carregado* ao qual o constraint se aplica — o wrapper, `Account<Vault>` no nosso caso, não o `Vault` pelado, porque o que o codegen está segurando quando um hook dispara é o wrapper carregado, e o wrapper faz deref para a sua struct, então o corpo lê igual de qualquer jeito. `Value` é o tipo do lado direito, `u64` para um piso de lamports. Repare que todo hook entrega um corpo default de `Ok(())`: um impl sobrescreve só as fases com as quais ele se importa, e as que você deixa sem escrever são no-ops por construção. E os quatro métodos são as quatro fases da vida de um constraint. Essa última parte é a peça que você tem que acertar, então ela merece uma seção própria.

### Os quatro hooks, e em qual deles um piso mora

O codegen não chama todos os quatro métodos toda vez. Ele chama o que casa com como o constraint foi escrito. Esta é a tabela de roteamento, e ela é o fato estrutural da lição:

![Um mapa de roteamento mostrando que ns::key pelado roteia para check, o prefixado com init roteia para init, init_if_needed bifurca para init e depois check, update(...) roteia para update, e exit dispara em qualquer caminho bem-sucedido.](assets/v03-diagram.webp)

Agora raciocine sobre onde um piso de min-balance pertence, em voz alta, porque o raciocínio é o ponto e é isso que a avaliação pede que você defenda.

O `init` roda uma vez, depois que a conta é criada. Se você põe o piso ali, você impõe ele exatamente na criação e nunca mais. Um vault criado acima do piso poderia cair abaixo dele na instrução seguinte e nada pegaria isso. Um piso que só vale no nascimento não é um piso. Descarte ele.

O `exit` roda no fim, durante a serialização da conta, só para contas que foram mutadas e escrevem de volta. Pôr uma validação ali é tentador porque "checar o estado final" soa seguro. Mas o `exit` roda *depois* da lógica do seu handler, então ele é uma asserção post-hoc, não uma trava. Ele também não roda de jeito nenhum para uma conta somente-leitura que nunca serializa, que é precisamente a conta com a qual um piso de tempo de leitura se importa. Se o seu objetivo é rejeitar uma conta ruim logo de cara, antes de qualquer trabalho acontecer, o `exit` é a fase errada. Descarte ele também.

O `update` só dispara dentro de uma cláusula `update(...)` explícita. Ele é para o caso em que o próprio constraint muta a conta durante um fluxo de update. Um piso é uma validação, não uma mutação. Este também não.

Sobra o `check`, e o `check` está exatamente certo, não por eliminação, mas por encaixe. O `check` roda na conta carregada, em toda instrução que casa, antes do handler. É essa a definição de uma trava de tempo de leitura: o vault tem que já satisfazer o invariante para a instrução prosseguir, e isso é re-verificado a cada chamada, sem cache, sem ser uma vez só. Um piso de saldo é uma trava de tempo de leitura. Então ele mora no `check`. Quando a avaliação pergunta qual hook e por quê, esta é a resposta inteira: `check`, porque um piso é um invariante na conta carregada que tem que valer antes de o handler rodar e em toda chamada.

![Uma árvore de decisão roteando uma regra para update quando ela muta, check quando ela tem que valer em todo carregamento, init só na criação, ou exit para asserções pós-handler.](assets/v04-flowchart.webp)

### O trade-off, dito sem enfeite

Um trait de constraint aberto compra uma coisa real para você. O seu invariante agora mora na macro, é visível na IDL, e não pode ser esquecido num call site, porque ele está preso ao próprio campo de conta, não às linhas de abertura de um handler. Acrescente uma instrução nova que carrega o vault com o mesmo constraint, e o piso vem junto de graça. É esse o reuso-sem-disciplina que os `if`s de handler nunca conseguem te dar.

Mas você agora é o autor de código que roda dentro do orçamento de compute de toda instrução que casa. Isto não é de graça, e fingir o contrário é como você entrega um programa lento. Comparado a um `if` de handler que você escreveu uma vez, um hook `check` roda a mesma comparação, então o custo de uma checagem *barata* dá na mesma. O perigo é uma checagem que não é barata. Se um colega escreve um `check` que relê e refaz o hash de um slab grande de dados a cada chamada, isso é compute que você agora paga em toda instrução que casa, num caminho quente, para sempre. E existe um segundo custo, mais silencioso: porque a conta então lê como "válida", uma checagem pesada ou sutilmente errada consegue esconder um bug de lógica atrás de um "passa nos constraints" verde. A regra de bolso é curta. Mantenha os hooks baratos, mantenha eles na fase certa, e nunca deixe um constraint fazer trabalho que pertence ao handler. A extensibilidade te entrega código adjacente ao framework para você ser dono; seja dono dele com cuidado.

![Uma comparação linha a linha mostrando que o hook check ganha em reuso e em visibilidade na IDL, empata no custo de uma comparação barata, e perde feio quando o hook é caro ou sutilmente errado.](assets/v05-comparison.webp)

Existe um pedaço de linhagem que vale carregar para dentro do Lab, porque ele explica por que esta porta existe afinal. A pressão não foi acadêmica. Ela veio de construtores. Na discussão #3742, o ChewingGlass colocou o problema de ergonomia do framework sem rodeios, "Boilerplate kills new devs because they don't know the sacred incantations," e, numa sub-thread de Codama daquela mesma discussão, "But borsh is kind of terrible." A issue de design #4390 carrega a mesma pressão nas palavras dela mesma, que "the default serialization should probably behave more like zero-copy but with better UX." Esse é o argumento da comunidade, comprimido, que empurrou o V2 na direção de uma superfície de constraints que você consegue estender em vez de uma lista de keywords que você só consegue aceitar. O trait aberto é como "menos boilerplate" fica quando ele para de ser uma reclamação e vira uma API.

![Uma linha do tempo desde a lista fechada de keywords do v1, passando pela pressão da comunidade para cortar boilerplate, até a reescrita no_std do V2 que fez os constraints despacharem por traits públicos e implementáveis.](assets/v06-timeline.webp)

## Lab: entregue `quarters::min_balance` no R2

Você está estendendo o R2, o programa `quarter_vault`, com um constraint customizado. Quando você terminar, `#[account(quarters::min_balance = 100)]` é uma keyword de verdade no campo do vault, o hook `check` dele rejeita um vault subfinanciado em tempo de constraint, e um teste em LiteSVM comprova tanto a rejeição quanto a passagem. A barra de `verify` para este artefato é uma coisa só: o `anchor test` está verde, um vault abaixo do piso é rejeitado pela camada de constraints, e um vault no piso ou acima dele passa.

**1. Confirme o toolchain do V2.** A mesma checagem da abertura de m03-l1, uma linha. Se a versão está errada, o comando de re-pin está abaixo dela; não construa conteúdo de V2 num binário `anchor` de V1:

```bash
anchor --version         # must report a 2.0.0 RC line, not 1.x; rc.1 as of 2026-08-12
# Only if it does not, re-pin (still no release binary for the v2 tag, so build from
# the tag: it is a fixed point, unlike the anchor-next branch tip it sits on):
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
```

**2. Comprove a lógica isolada primeiro.** Antes de tocar na macro, deixe a lógica do `check` correta como Rust puro, exatamente o arquivo de rascunho do topo da lição. Esta é a mesma destilação que o challenge de código avalia, e vale passar nela antes de ligar ela no framework, porque um constraint que falha e que você não consegue isolar é miserável de depurar. Preencha `meets_floor` para que ele rejeite abaixo do piso e aceite no piso ou acima dele. O piso é inclusivo: um saldo igual ao mínimo passa. O degrau avaliado entrega este mesmo arquivo com a mesma divisão e acrescenta um bloco de asserções `const` embaixo dele — já que `meets_floor` é um `const fn`, essas rodam no compilador, então uma regra errada faz o build falhar com uma mensagem nomeando o caso que ela errou.

```rust
// scratch.rs - now with the condition filled in
pub trait BalanceGate {
    fn check(&self, balance: u64) -> Result<(), String>;
}

pub struct MinBalanceRule {
    pub min: u64,
}

impl MinBalanceRule {
    const fn meets_floor(&self, balance: u64) -> bool {
        balance >= self.min
    }
}

impl BalanceGate for MinBalanceRule {
    fn check(&self, balance: u64) -> Result<(), String> {
        if self.meets_floor(balance) {
            Ok(())
        } else {
            Err(format!(
                "quarters::min_balance violated: {} < {}",
                balance, self.min
            ))
        }
    }
}

fn run_constraint(balance: u64, min: u64) -> bool {
    MinBalanceRule { min }.check(balance).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_is_inclusive_and_rejects_below() {
        assert!(run_constraint(500, 100)); // above passes
        assert!(run_constraint(100, 100)); // exactly at the floor passes
        assert!(!run_constraint(50, 100)); // below is rejected
        assert!(!run_constraint(0, 1)); // an empty vault fails a 1-lamport floor
    }
}
```

```bash
rustc --test scratch.rs -o scratch_test && ./scratch_test
```

Esperado: `test result: ok. 1 passed; 0 failed`. A lógica está fechada. Tudo daqui em diante é ligar ela no V2 para que ela dispare a partir da macro em vez de a partir de uma função que você lembrou de chamar.

**3. Escreva o tipo marcador e o impl de `AccountConstraint` dele.** Abra o `lib.rs` do R2. Relembre o vault de mais cedo no módulo: uma conta Pod, porque o `Account<T>` do V2 exige `T: Pod`, segurando o owner, um saldo de `credit` em lamports nativos por enquanto, e o bump canônico.

```rust
use anchor_lang::prelude::*;

// Already in your file from earlier in the module, reproduced here for context;
// do not re-paste them. The id stays the one anchor init generated for you, and
// `_pad` is the explicit tail padding that keeps Vault Pod.
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

Agora o constraint, e primeiro a única regra mecânica que faz a coisa toda funcionar, porque nada mais nesta lição vai te contar e você não consegue escrever o problema solo sem ela. A macro resolve `ns::key = value` por caminho, não por registro. Ela pega o namespace `ns` literalmente como um caminho de módulo que tem que estar em escopo, e converte o `key` em snake_case num nome de tipo em PascalCase dentro dele, e então acrescenta um sufixo `Constraint` obrigatório. Então `quarters::min_balance = 100` compila para uma chamada em `quarters::MinBalanceConstraint`, e `quarters::max_balance` resolveria para `quarters::MaxBalanceConstraint`. Três consequências que vale segurar: o módulo tem que ser alcançável de onde a struct do derive está escrita, o nome do tipo marcador não é um rótulo que você escolhe livremente (é o `key` em PascalCase mais o sufixo `Constraint`, ou nada), e o `100` da direita tem que passar na checagem de tipos como o `Self::Value` daquele impl, que é por que o tipo associado é `u64` e não uma coisa que você pode inferir.

Com essa regra enunciada: o tipo marcador `MinBalanceConstraint` mora num módulo `quarters`, e implementar `AccountConstraint<Account<Vault>>` para ele é o que dá a `quarters::min_balance` um corpo para chamar. O impl mira o wrapper carregado, `Account<Vault>`, não o `Vault` pelado, porque o wrapper é o que o codegen tem em mãos quando o hook dispara — e já que o wrapper faz deref para a sua struct, `vault.credit` lê exatamente como leria no tipo pelado. O corpo do `check` é a lógica que você acabou de comprovar, traduzida para o trait de verdade: o `check` recebe `&Account<Vault>` e `&Self::Value`, e rejeita com um erro de programa em vez de com uma `String`. Os outros três hooks ficam sem escrita para um piso puramente de tempo de leitura: o trait deixa todo hook em `Ok(())` por padrão, então deixar `init`, `update` e `exit` de fora *é* a declaração de que esta regra não faz nada nessas fases:

```rust
pub mod quarters {
    use super::*;

    /// Marker type. Implementing AccountConstraint<Account<Vault>> for it makes
    /// `#[account(quarters::min_balance = N)]` a real, IDL-visible constraint.
    pub struct MinBalanceConstraint;

    impl AccountConstraint<Account<Vault>> for MinBalanceConstraint {
        type Value = u64;

        fn check(vault: &Account<Vault>, floor: &u64) -> Result<()> {
            // The read-time gate: the loaded vault must already hold the floor.
            require_gte!(vault.credit, *floor, VaultError::BelowFloor);
            Ok(())
        }
    }
}

// You already have this enum from last lesson. Anchor allows exactly one
// #[error_code] per program, so do not add a second: just add the BelowFloor
// variant to the one that is already there.
#[error_code]
pub enum VaultError {
    #[msg("caller is not the configured arcade authority")]
    Unauthorized,
    #[msg("credit addition overflowed")]
    Overflow,
    #[msg("account is owned by the wrong program")]
    WrongOwner,
    #[msg("vault credit is below the quarters::min_balance floor")]
    BelowFloor,
}
```

`require_gte!(a, b, err)` é a macro do V2 para "a tem que ser maior ou igual a b, senão devolva err". É a forma nativa do framework de escrever o piso inclusivo; ir atrás de um `if` cru com um `return Err(...)` manual também compilaria, mas a macro é o estilo da casa e mantém o caminho de erro uniforme.

![A função check lê o vault carregado somente para leitura, faz deref do piso u64, e usa require_gte para rejeitar qualquer credit abaixo desse piso inclusivo.](assets/v07-annotated-code.webp)

Esperado depois deste passo: o `anchor build` compila o impl mesmo que nada use o constraint ainda. Um erro de compilação nomeando `AccountConstraint` aqui quer dizer que o formato do trait se moveu debaixo do RC, então releia ele contra o crate antes de ir adiante.

**4. Aplique o constraint a uma instrução travada.** Acrescente dois handlers. O `set_credit` é uma fixture que põe o `credit` armazenado do vault direto, fazendo as vezes do caminho de depósito de verdade que chega no módulo 4, quando o vault começa a mover lamports de verdade. Seja honesto com você mesmo sobre o que ele é: um setter sem trava, uma lição depois de uma lição inteira sobre travar escritas. Ele é entregue sem um `address = config.authority` de propósito, para que o teste consiga levar o vault a qualquer credit numa linha só, e ele é exatamente o handler que você deletaria antes de entregar. O constraint é o artefato; a fixture é um gabarito, e a tabela de destinação de m05-l1 é onde você de fato deleta ele. O `require_funded` é a operação travada: o corpo dele é vazio de propósito, para que qualquer passagem ou falha só possa vir da camada de constraints, nunca da lógica de handler. A trava é a única linha nova, `quarters::min_balance = 100`, no campo do vault:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    /// Fixture: set stored credit directly. Real lamport movement is module 4.
    pub fn set_credit(ctx: &mut Context<SetCredit>, amount: u64) -> Result<()> {
        ctx.accounts.vault.credit = amount;
        Ok(())
    }

    /// Guarded no-op: reaching this body at all proves the floor was satisfied.
    pub fn require_funded(_ctx: &mut Context<RequireFunded>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetCredit {
    pub player: Signer,
    #[account(
        mut,
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump, // reuse the stored canonical bump, no runtime search
    )]
    pub vault: Account<Vault>,
}

#[derive(Accounts)]
pub struct RequireFunded {
    pub player: Signer,
    #[account(
        seeds = [b"vault", player.address().as_ref()],
        bump = vault.bump,
        quarters::min_balance = 100, // the custom constraint: check hook fires here
    )]
    pub vault: Account<Vault>,
}
```

Olhe para onde o constraint fica. Ele está no campo do vault em `RequireFunded`, bem ao lado de `seeds` e `bump`, que são keywords embutidas, e ele lê exatamente como elas. É esse o retorno: `quarters::min_balance` agora é um constraint de primeira classe, indistinguível no uso dos que o framework entregou. O vault aqui nem é `mut`, que é o sinal honesto de que isto é uma trava somente-leitura e portanto o `exit` nunca roda para ele, só o `check`.

Esperado depois deste passo: o `anchor build` está limpo e a IDL de `require_funded` carrega o constraint na conta `vault` dela. Essa linha de IDL é a diferença entre uma regra que o framework impõe e uma regra que você lembrou de escrever.

**5. Escreva o teste em LiteSVM.** O LiteSVM roda o programa no mesmo processo, sem validador, então o loop é rápido. Acrescente as dev-dependencies:

```toml
# Same one row as last lesson, and for the same reason: the harness owns the SVM
# version so you cannot drift off it. At tag v2.0.0-rc.1 anchor-v2-testing carries
# litesvm 0.11.0. Naming litesvm yourself is how you end up with two of them.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
```

O teste, em `tests/min_balance.rs`, faz quatro coisas: inicializa o vault, põe o credit dele abaixo do piso e comprova que o `require_funded` é rejeitado, depois põe ele no piso e comprova que o `require_funded` passa. A rejeição e a passagem são o artefato inteiro.

```rust
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction,
};

fn require_funded_tx(
    svm: &LiteSVM,
    player: &Keypair,
    program_id: Address,
    vault_pda: Address,
) -> VersionedTransaction {
    let ix = Instruction {
        program_id,
        accounts: quarter_vault::accounts::RequireFunded {
            player: player.pubkey(),
            vault: vault_pda,
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::RequireFunded {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
    VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[player]).unwrap()
}

#[test]
fn min_balance_rejects_below_and_passes_at_floor() {
    let mut svm = anchor_v2_testing::svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let player = Keypair::new();
    svm.airdrop(&player.pubkey(), 1_000_000_000).unwrap();
    let (vault_pda, _bump) =
        Address::find_program_address(&[b"vault", player.pubkey().as_ref()], &program_id);

    // Init the vault (init_vault was built in the earlier lesson; credit starts at 0).
    let init_ix = Instruction {
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
    let init_msg = Message::new_with_blockhash(&[init_ix], Some(&player.pubkey()), &blockhash);
    let init_tx =
        VersionedTransaction::try_new(VersionedMessage::Legacy(init_msg), &[&player]).unwrap();
    svm.send_transaction(init_tx).unwrap();

    // Helper to set stored credit.
    let set_credit = |svm: &LiteSVM, amount: u64| {
        let ix = Instruction {
            program_id,
            accounts: quarter_vault::accounts::SetCredit {
                player: player.pubkey(),
                vault: vault_pda,
            }
            .to_account_metas(None),
            data: quarter_vault::instruction::SetCredit { amount }.data(),
        };
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&player.pubkey()), &blockhash);
        VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&player]).unwrap()
    };

    // Below the floor: the constraint layer must REJECT require_funded.
    svm.send_transaction(set_credit(&svm, 50)).unwrap();
    let below = svm.send_transaction(require_funded_tx(&svm, &player, program_id, vault_pda));
    assert!(below.is_err(), "under-floor vault must be rejected by the constraint");

    // At the floor: the constraint layer must PASS require_funded.
    svm.send_transaction(set_credit(&svm, 100)).unwrap();
    // LiteSVM never advances its blockhash on its own, so a second require_funded
    // transaction built right now would be byte-identical to the rejected one above --
    // same signature -- and the SVM would refuse it as a duplicate (AlreadyProcessed)
    // instead of re-running it. Expire the blockhash so the retry is genuinely new.
    svm.expire_blockhash();
    let at_floor = svm.send_transaction(require_funded_tx(&svm, &player, program_id, vault_pda));
    assert!(at_floor.is_ok(), "at-or-above-floor vault must pass the constraint");
}
```

**6. Faça o build e rode.**

```bash
anchor test
```

Saída esperada, o único teste passando que limpa a barra:

```
running 1 test
test min_balance_rejects_below_and_passes_at_floor ... ok

test result: ok. 1 passed; 0 failed
```

A prova está no handler vazio. O `require_funded` não faz nada, então o `is_err()` no vault de 50 de credit e o `is_ok()` no vault de 100 de credit só podem ser o hook `check` falando. O invariante disparou em tempo de constraint, antes de o seu código rodar, que é exatamente onde você argumentou que ele deveria. Se em vez disso você vê a chamada abaixo do piso *passar*, a causa de sempre é o constraint lendo o campo errado ou um `>` onde você queria `>=` voltando escondido; rode de novo o teste de rascunho do passo 2 para isolar a lógica da fiação.

## Challenge

Dois degraus. O primeiro te entrega o trait e tira de volta o corpo; o segundo não te entrega nada.

**Completion.** Reabra o hook `check` no seu impl de `MinBalanceConstraint` e apague o corpo, deixando `fn check(vault: &Account<Vault>, floor: &u64) -> Result<()> { /* TODO */ }`. Reabasteça ele de memória para que o piso seja inclusivo: um vault cujo credit é igual ao piso passa, um abaixo dele devolve `VaultError::BelowFloor`. Duas rodadas aceitam ele: o degrau avaliado, que é o arquivo de rascunho do passo 2 com as asserções de tempo de compilação penduradas embaixo dele, e o `anchor test` do passo 6. Se você for atrás do `init` ou do `exit` para fazer isso, pare: uma trava de tempo de leitura mora no `check`, e pôr ela em qualquer outro lugar ou perde instruções posteriores ou dispara na fase errada.

**Solo.** Acrescente um *segundo* constraint com namespace e comprove que ele compõe com o primeiro numa conta só. Escolha um: `quarters::max_balance = N`, que rejeita um vault cujo credit está *acima* de um teto, ou `quarters::owner_is = <expr>`, que rejeita um vault cujo owner armazenado não é um endereço dado. Implemente ele como o próprio tipo marcador dele com o próprio impl de `AccountConstraint<Account<Vault>>` dele, escolhendo o hook correto (os dois são travas de tempo de leitura, então os dois são `check`, e raciocinar o porquê é metade do exercício). Depois aplique os *dois* num único campo, `#[account(quarters::min_balance = 100, quarters::max_balance = 10_000)]`, e escreva um teste em LiteSVM comprovando que um vault dentro da faixa passa enquanto um de qualquer um dos lados é rejeitado. Aceitação: o segundo constraint dispara a partir da macro de derive sem mudança nenhuma no framework, os dois constraints compõem numa conta só, e o challenge de código em Rust puro continua passando. Uma coisa que vale observar: um vault que falha no primeiro constraint nunca deveria alcançar o segundo, porque hooks `check` fazem curto-circuito no primeiro erro, igual a qualquer resultado propagado por `?`.

Agora você tem a história inteira dos constraints: você consegue derivar PDAs, manejar o catálogo embutido inteiro e, a partir de hoje, estender esse catálogo com keywords que os autores do framework nunca escreveram. O vault está validado tão apertado quanto você consegue descrever. O vault consegue segurar credit e travar credit. O que ele nunca fez foi pagar ninguém de volta. A seguir, o módulo 4 põe ele para trabalhar: assinando um withdrawal de lamports de verdade *como* o PDA, através do modelo de borrow do `CpiHandle` do V2, onde o compilador, e não uma chamada `.reload()` que você lembrou de fazer, é o que te mantém seguro. Os livros param de ser livros.
