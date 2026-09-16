# Abrindo o `#[derive(Accounts)]`: despacho, discriminators e erros

Na lição passada você leu no que `declare_id!` e `#[program]` se expandem, e você fechou o loop de deploy-e-invoke no greeter. Você viu o lado do programa. Mas um programa V2 tem duas metades, e a metade que de fato decide se uma transação maliciosa consegue rodar é a que você ainda não abriu: `#[derive(Accounts)]`.

Esse derive é quieto. Você escreve quatro campos e um par de atributos `#[account(...)]`, e ele gera o carregamento de contas, as checagens de constraint, uma trava de duplicate-mutable, e a fiação que o dispatcher chama antes de o seu handler chegar a executar. Esta lição abre ele. No fim você vai conseguir traçar a ordem gerada, derivar na mão os preimages de discriminator do Anchor, e ler o layout de erros do framework versus os customizados bem o bastante para prever um código de erro no wire antes de rodar o programa.

Aqui está o jeito mais rápido de deixar uma dessas ideias concreta agora mesmo. Um discriminator (discriminador) é só os primeiros 8 bytes de um hash sha256 sobre uma string com namespace. A instrução `greet` do seu greeter tem um, e você pode computar ele no seu terminal:

```bash
# shasum ships with macOS and most Linux distros. If yours lacks it,
# swap in the coreutils equivalent: sha256sum.
echo -n "global:greet" | shasum -a 256 | cut -c1-16
```

Essa string hex é exatamente a tag de 8 bytes que um cliente coloca na frente dos dados da instrução para que o dispatcher saiba qual handler chamar.

## Resumo

A rota segue a ordem do próprio framework. Primeiro você lê o que o derive gera no greeter que você já tem: carregamento, depois constraints, depois despacho. Depois a trava de duplicate-mutable, onde uma máscara de tempo de compilação encontra uma checagem em tempo de execução e a maioria das pessoas chuta a errada. Depois os discriminators e o layout dos códigos de erro, que são as duas superfícies que os seus clientes de fato veem no wire. O lab estende o R0 com uma segunda instrução e uma struct de accounts pequena, então cada uma dessas superfícies vira algo que você pode ver disparar.

O recuo da ajuda desta lição: o passo a passo da expansão do derive é totalmente trabalhado, feito para você etapa por etapa. O challenge discriminator-preimage é um problema de completion, um starter que falha até você corrigir. A previsão de erro customizado e o raciocínio de duplicate-mutable são solo, sem apoio.

## Abrindo o derive

### O que o derive gera, em ordem

Comece pelo status quo e pelo limite dele. Um programa Solana cru recebe um slice plano de contas e um slice plano de bytes. Toda propriedade de segurança que te interessa, que esta conta é um signatário, que esta aqui é de propriedade do seu programa, que esta pubkey é de verdade o PDA que você pensa que é, tem que ser checada na mão, na ordem certa, sem ajuda nenhuma do compilador. Deixe passar uma checagem e você tem uma vulnerabilidade. A razão inteira de `#[derive(Accounts)]` existir é mover essa checklist de algo que você lembra para algo que a macro gera.

Então a pergunta natural é: o que exatamente ele gera, e em que ordem? Porque a ordem decide se um constraint te protege ou roda tarde demais para importar.

O derive gera três fases, e elas sempre rodam nesta sequência:

1. **Carregamento.** Cada campo é desserializado do slice de contas cru para o wrapper tipado dele. `Account<Marquee>` checa o owner e o discriminator de 8 bytes e te entrega uma view tipada. `Signer` checa o bit de assinatura. `Program<System>` checa o endereço e a flag de executável. Se um campo não consegue carregar como o tipo declarado dele, a validação para aqui, antes de qualquer constraint seu rodar.
2. **Constraints.** Os atributos `#[account(...)]` disparam como hooks: `mut`, `init`, `seeds` e `bump`, `has_one` (que o V2 deprecia em favor de `address = ...`; ele ainda parseia, com um aviso), `constraint = ...`. Esses rodam depois do carregamento porque a maioria deles precisa dos dados carregados para checar qualquer coisa. Um `has_one = authority` não consegue comparar contra um campo que ele ainda não desserializou.
3. **Despacho.** Só depois que o carregamento e os constraints passam é que o dispatcher entrega o `ctx.accounts` validado ao seu handler. O corpo do seu handler é a última coisa a rodar, não a primeira.

![Um fluxo de cima para baixo de três fases, carregamento, depois constraints, depois despacho, em que cada fase só roda se a anterior passou e o corpo do handler roda por último.](assets/v01-flowchart.webp)

Essa ordenação é o modelo mental que você carrega pelo resto do curso. Todo constraint que você escrever mora na fase dois, o que quer dizer que ele pode presumir que a conta já carregou como o tipo dela, e ele roda antes da sua lógica, o que quer dizer que um constraint que falha te custa a taxa da transação mas nunca deixa uma conta ruim chegar ao seu handler.

### A expansão, mostrada de verdade

Fases abstratas são boas, mas você não precisa aceitar elas por fé. O derive gera uma implementação de trait de verdade, e você pode ver ela. Se você tem `cargo-expand` instalado, aponte ele para o seu programa e leia a saída:

```bash
cargo install cargo-expand
cargo expand --package greeter
```

Segure esse comando até o passo 1 do Lab. Agora o seu programa tem uma struct de accounts, `Greet`, com um único `Signer` dentro dela, e expandir isso te mostra uma função de duas linhas que não prova nada. A struct que vale ler a expansão é a `LightMarquee`, que você digita no Lab: três campos, um `payer: Signer` com `mut`, um `marquee: Account<Marquee>` com `init`, e um `system_program: Program<System>`. Então leia o esboço abaixo agora, digite a struct no Lab, depois rode `cargo expand` e confira a saída real contra ele.

Para essa struct, o derive gera uma implementação do trait `TryAccounts` cuja função `try_accounts` é, em essência, as três fases escritas como código em linha reta. Ela roda na ordem de declaração dos campos, que é por isso que a ordem em que você escreve os seus campos é a ordem em que eles carregam:

![Um esboço do dispatcher testando o bitvec de duplicatas percorrido contra a MUT_MASK de LightMarquee, e depois try_accounts carregando e aplicando constraints em cada campo na ordem de declaração antes de o handler rodar.](assets/v02-annotated-code.webp)

Esse esboço é simplificado de propósito, mas a estrutura é fiel. Três coisas valem ser puxadas dele, porque elas respondem perguntas que a versão abstrata deixa em aberto.

Primeiro, ordem de campo é ordem de carregamento. A macro percorre a sua struct de cima para baixo. Se o constraint de um campo posterior depende de um campo anterior, por exemplo um `address = config.authority` que compara contra uma conta `config` declarada acima dele, o campo anterior tem garantia de ter carregado primeiro. Reordene os seus campos e você pode genuinamente mudar qual checagem roda contra dados carregados versus não carregados.

Segundo, a trava de duplicate-mutable não está embutida no carregamento de nenhum campo isolado, e ela nem mora em `try_accounts`. O dispatcher primeiro percorre as views de conta que chegam, anota num bitvec qualquer endereço que aparece duas vezes, e faz o AND desse bitvec contra a `MUT_MASK` de tempo de compilação da struct num único teste de quatro palavras, antes de o carregamento tipado começar. Compostos são tratados em tempo de compilação em vez de em tempo de execução: um campo `Nested<Inner>` embute a `MUT_MASK` da própria struct interna, deslocada pelo offset daquele campo, na máscara externa. Então um teste cobre a árvore de contas inteira, e ele pega uma colisão mesmo quando a mesma conta é passada para um campo direto e para um campo enterrado dentro de um composto.

Terceiro, o handler não aparece em `try_accounts` de jeito nenhum. Carregamento e validação são uma função gerada; o seu handler é uma função separada que o dispatcher chama só depois de `try_accounts` retornar `Ok`. Essa separação é a razão estrutural pela qual um constraint nunca pode rodar "tarde demais": ele fisicamente não consegue, porque mora numa função que termina antes de o seu código começar.

### A trava de duplicate-mutable: máscara de tempo de compilação, checagem em tempo de execução

Agora o detalhe mais afiado do V2, e o que vale desacelerar por ele, porque mora exatamente na emenda entre o que o compilador sabe e o que só o runtime pode saber.

Considere uma instrução que recebe duas contas do mesmo tipo, as duas mutáveis:

```rust
#[derive(Accounts)]
pub struct TallyTwo {
    #[account(mut)]
    pub first: Account<Marquee>,
    #[account(mut)]
    pub second: Account<Marquee>,
}
```

Quem chama controla quais pubkeys aterrissam em `first` e `second`. Nada impede que passem a *mesma* conta para os dois. E isso é genuinamente perigoso. Tanto `first` quanto `second` desserializariam os mesmos bytes subjacentes em duas cópias mutáveis separadas. O seu handler muta `first`, depois muta `second`, e na saída o Anchor serializa os dois de volta. A segunda escrita atropela a primeira. Você somou um a um contador e ele subiu um em vez de dois, silenciosamente, sem erro nenhum. Esse é um bug clássico de aliasing de conta, e o V2 proíbe isso por padrão.

Aqui está a pergunta que importa: *onde* essa colisão é pega? A resposta ingênua é "em tempo de compilação, o compilador sabe que existem dois campos mutáveis." Isso está meio certo e é a metade que derruba as pessoas. O compilador de fato sabe o *formato*: quais campos são mutáveis e serializam na saída. O derive codifica isso como uma const associada de 256 bits, `MUT_MASK`, um bit por slot de conta, ligado para cada campo mutável que serializa. Essa máscara é fixada em tempo de compilação e não custa nada em tempo de execução.

Mas o compilador não pode saber os *valores*. Se `first` e `second` guardam o mesmo endereço depende inteiramente do que quem chama manda, e isso só é sabível quando a transação chega. Então o runtime faz a outra metade: o dispatcher percorre as views de conta que chegam, liga um bit para cada slot cujo endereço ele já viu, e faz o AND desse bitvec contra a `MUT_MASK`. Se algum bit sobrevive, dois slots mutáveis carregam o mesmo endereço, e a chamada retorna `ConstraintDuplicateMutableAccount` a partir do dispatcher, em tempo de execução, antes de o seu handler rodar.

![Um diagrama em duas partes da MUT_MASK como uma bitmask fixa de tempo de compilação dos campos mutáveis, testada contra um bitvec de tempo de execução de endereços repetidos, com qualquer bit sobrevivente levantando um erro.](assets/v03-diagram.webp)

Existe uma segunda coisa, separada, que as pessoas confundem com essa, e fixar ela é o ponto inteiro do checkpoint mais adiante. Se você de fato *quer* passar a mesma conta mutável duas vezes, porque o seu handler está escrito para nunca manter referências mutáveis conflitantes, você faz opt-out por campo:

```rust
#[derive(Accounts)]
pub struct TouchTwice {
    #[account(mut, unsafe(dup))]
    pub first: Account<Marquee>,
    #[account(mut, unsafe(dup))]
    pub second: Account<Marquee>,
}
```

O opt-out se escreve `unsafe(dup)`. O `unsafe` é deliberado: dar aliasing em dados de conta mutáveis é uma cilada, e o V2 te obriga a nomear isso. Agora a distinção. Escrever `dup` puro — a grafia v1 desse atributo — sem `unsafe` é um *erro de compilação*, e o compilador te diz para escrever `unsafe(dup)` em vez disso. Esse é um evento de tempo de compilação sobre o *atributo que você digitou*. Não tem nada a ver com duas contas de fato colidirem. A colisão em si, a mesma pubkey chegando nos dois slots, é pega em *tempo de execução*, no dispatcher, contra aquele bitvec percorrido. Dois eventos diferentes, dois tempos diferentes. Não deixe a palavra compartilhada "dup" embaçar os dois.

Uma nota sobre escopo, porque ela te poupa uma tarde confusa. A trava se apoia no atributo `mut`, não no tipo do wrapper. Todo campo marcado com `mut` liga o bit dele, `Account<T>` e `Signer` igualmente; um campo sem `mut` não liga nenhum, então passar a mesma conta somente leitura para dois slots é sempre tranquilo. Duas exceções encolhem a máscara: um campo com `unsafe(dup)` é excluído por design, e um campo `Option<_>` é excluído da máscara de tempo de compilação (um slot `None` é codificado como o id do programa, o que de outro jeito leria como uma colisão) e ganha uma checagem por campo mais estreita em vez disso. É por isso que o nosso `TallyTwo` marca os dois campos com `mut`: sem esse atributo não haveria nada para colidir.

Só que nada disso é você aceitar a palavra do Anchor. O dispatcher que você está lendo passa por fuzzing ele mesmo. O changelog do V2 credita ao fuzzing a descoberta de 4 bugs de correção no framework, rastreados na issue #4431, e a suíte de testes do Anchor carrega testemunhas do Miri e configurações do Kani. A cola gerada que percorre as suas contas é checada contra comportamento indefinido do mesmo jeito que você checaria um programa atrás do qual você estava a ponto de colocar dinheiro. Isso é uma coisa razoável de confiar, precisamente porque é verificado em vez de afirmado.

### Os discriminators ganham a casa com nome

Você já computou um no começo da lição. Agora vamos dar à ideia o formato completo dela, porque existem três namespaces e exatamente um deles surpreende as pessoas.

Um discriminator é uma tag de 8 bytes que o Anchor prefixa para que o runtime consiga distinguir uma coisa de outra sem parsear a carga inteira. Contas ganham um para que uma desserialização possa rejeitar o tipo de conta errado de cara. Instruções ganham um para que o dispatcher consiga rotear para o handler certo. Eventos ganham um para que um indexador consiga dizer qual evento ele decodificou. Por padrão cada um deles é os primeiros 8 bytes de `sha256` sobre uma *string de preimage com namespace*, e o namespace é a parte que carrega a armadilha:

- uma struct de conta faz hash a partir de `account:<Name>`, por exemplo `sha256("account:Marquee")[..8]`
- uma struct de evento faz hash a partir de `event:<Name>`, por exemplo `sha256("event:MarqueeLit")[..8]`
- um handler de instrução faz hash a partir de `global:<Name>`, por exemplo `sha256("global:greet")[..8]`

![Uma comparação de três linhas dos namespaces de discriminator mostrando que structs de conta usam account, eventos usam event e handlers de instrução usam global, com o namespace global sinalizado como a armadilha comum.](assets/v04-comparison.webp)

Por que `global:`? História. O Anchor antigo colocava handlers de instrução sob um namespace de estado `global`, um design que foi embora quase todo mas deixou a convenção de preimage atrás. Não existe namespace `instruction:` e nunca existiu. Se você algum dia montar na mão uma tag de instrução a partir de `instruction:<Name>`, os seus bytes não vão bater com os que o programa gerou, e o dispatcher vai rejeitar a chamada como uma instrução desconhecida.

Alguns fatos a mais amarram a superfície, e eles importam no momento em que você se preocupa com compatibilidade de wire. Os discriminators do V2 não mudaram por padrão: mesmo esquema sha256 de 8 bytes, então o formato de wire de um programa V2 continua compatível com um cliente v1 que já sabe montar essas tags. O V2 aperta uma regra: um discriminator de conta todo zerado é rejeitado, porque todos-zeros é o que uma conta não inicializada lê como, então permitir isso deixaria uma conta vazia se passar por uma de verdade. Essa rejeição é especificada para discriminators de conta especificamente, a confusão que ela previne só existe para contas.

E existe uma compactação de opt-in. Se 8 bytes na frente de toda instrução parece pesado, o V2 te deixa anotar um handler com `#[discrim = N]`, o que troca o prefixo sha256 de 8 bytes dele por uma tag de inteiro pequeno. É engenharia honesta, mas leia a troca antes de recorrer a ela, porque não é o override por item que você poderia esperar.

![Uma tabela de comparação dos discriminators sha256 de 8 bytes default contra discriminators de instrução compactos, mostrando que o compacto economiza bytes mas é tudo-ou-nada por programa, acrescenta validação de ambiguidade de prefixo, e abre mão da compatibilidade de wire v1 default.](assets/v05-table.webp)

Repare no formato dessa troca. O default te custa 8 bytes no wire mais as compute units para compará-los, e te compra legibilidade e compatibilidade v1 de graça. A opção compacta economiza os bytes e a computação, mas é tudo-ou-nada por programa, ela tem que ser validada contra ambiguidade de prefixo para que duas tags não possam dar alias, e ela abre mão da compatibilidade v1 default. Compatibilidade e legibilidade versus eficiência bruta. É essa a decisão inteira, e para a maioria dos programas o default ganha, que é exatamente por que o caminho compacto continua uma aresta.

Jacob Creech, descrevendo o redesign do discriminator, colocou a realidade de uso sem enfeite: discriminators customizados estão "being considered ... almost have 0 usage today." É esse o contexto de por que `#[discrim = N]` é uma aresta de opt-in e não o default. O default é default porque é isso que essencialmente todo mundo entrega de fato.

### O layout de erros: framework nos 2000s, customizado em 6000

A última peça da superfície é o que um cliente vê quando algo falha. Os códigos de erro do Anchor são particionados em faixas, e a partição não é arbitrária. Ela existe para que um erro do framework e o seu erro nunca possam colidir no wire.

Todo constraint que o framework impõe retorna um código na banda dele. A que você vai encontrar constantemente é a banda de constraint, que começa em 2000. `ConstraintHasOne`, o erro que um `has_one` violado lança, é 2001. `ConstraintDuplicateMutableAccount`, a trava que a gente acabou de traçar, mora nesse mesmo território do framework. Os seus próprios erros, os que você declara com `#[error_code]`, começam em 6000 e contam para cima a partir daí, indexados a partir de zero por variante.

![Um gráfico em bandas das faixas de códigos de erro do Anchor, indo de erros de instrução em 100 passando por erros de constraint em 2000 até erros customizados começando em 6000.](assets/v06-comparison.webp)

Isso te dá uma ferramenta preditiva genuinamente útil. Pegue um programa cujo enum `#[error_code]` customizado tem, digamos, três variantes, e nenhum override de `offset`. A primeira variante é 6000, a segunda 6001, a terceira 6002. Se você sabe a posição base zero de uma variante você sabe o número dela no wire sem rodar nada. O erro clássico é ver os 2000s num erro decodificado e presumir que é ali que os erros customizados moram. Não é. Os 2000s são a banda de constraint do framework. Os seus erros começam em 6000. Quando você quer mover essa base, por exemplo para deixar espaço ou para casar com uma convenção externa, `#[error_code(offset = N)]` desloca ela.

É essa a superfície de erros inteira: o framework é dono de tudo abaixo de 6000, você é dono de 6000 para cima, e a fronteira é fixa para que os dois nunca possam ser confundidos.

### Onde os eventos encaixam, e onde o dispatcher começa

O terceiro namespace, `event:`, merece o momento dele, porque o greeter que você estende no Lab abaixo emite um e você deveria saber exatamente o que acontece no wire quando ele emite. O handler `light_marquee` que você está a ponto de escrever chama `emit!(MarqueeLit { plays })`. Essa macro serializa a struct de evento com o **wincode**, o serializador do V2 (você conheceu o nome em m01-l2 como a crate no centro da quebra de dependência #4937; isto é o que ele de fato faz), rodando ele sob uma config de wire cuja saída é byte-idêntica à do borsh. Depois ela prefixa o discriminator `event:MarqueeLit` de 8 bytes e escreve a coisa toda para fora pelo syscall `sol_log_data`. Não fica guardado numa conta e não é devolvido a quem chamou. Aterrissa nos dados de log da transação, onde um indexador off-chain inscrito no seu programa pode ler, casar os 8 bytes iniciais contra `sha256("event:MarqueeLit")[..8]`, e decodificar o resto como um `MarqueeLit`. O discriminator é o que deixa o indexador distinguir o seu `MarqueeLit` de todo outro evento que qualquer programa emitiu no mesmo bloco.

Existe uma variante de evento mais rápida, `#[event(bytemuck)]`, que pula o serializador inteiro e dispõe os campos como uma struct Pod de tamanho fixo, então decodificar é um cast em vez de um parse. A gente deixa ela para o próximo módulo de propósito, porque ela só faz sentido depois que você conheceu o Pod e o layout zero-copy. Recorra a ela agora e você estaria ligando uma ferramenta cuja fundação você não concretou. Para esta lição, o `emit!` puro está exatamente certo: ele mostra o namespace `event:` fazendo o trabalho dele sem arrastar máquinas que não te ensinaram.

O que fecha o loop de volta para onde o capítulo inteiro começou, o dispatcher. Trace uma chamada completa e cada namespace aparece no lugar dele. Um cliente monta uma transação, prefixa a tag `global:light_marquee` de 8 bytes aos dados da instrução, e manda. O dispatcher lê aqueles primeiros 8 bytes, casa eles contra o discriminator de instrução de cada handler, e roteia para `light_marquee`. Depois `try_accounts` roda: ele carrega cada `Account<Marquee>` checando o discriminator `account:` dele, roda os constraints, percorre a trava de duplicate-mutable. Só então o corpo do seu handler roda, e quando ele chama `emit!`, sai o discriminator `event:` no log. Três namespaces, uma invocação, cada um fazendo o único trabalho para o qual ele foi hasheado.

![Uma linha do tempo de seis paradas de uma invocação, com o namespace global roteando a instrução, account: validando a conta carregada, e event: etiquetando o log emitido.](assets/v07-timeline.webp)

## Lab: estenda o R0 e observe a superfície

Você continua trabalhando no R0, o greeter. Este lab estende ele com uma segunda e uma terceira instrução e uma struct de conta pequena, puramente para você observar o carregamento gerado, os constraints, o despacho, os namespaces de discriminator, e o layout de erros em código que você escreveu. Estado real e testado chega no próximo módulo. Por agora, o ponto é ver a superfície.

Primeiro, o toolchain. Você já fez o build do V2 RC em m01-l2, e você aprendeu lá por que `avm install` não consegue buscar ele para você: nenhum GitHub Release foi cortado para a tag v2, então o binário pré-compilado que ele baixa dá 404s. O RC vive no canal git (`cargo install --git ... --tag v2.0.0-rc.1 anchor-cli --locked --force` — m01-l2 construiu a partir do próprio branch `anchor-next` porque o canal era o assunto dele; daqui pra frente o curso fixa a tag), e o binário já está na sua máquina. Confirme que é ele que está respondendo antes de tocar em código:

```bash
anchor --version   # the real check: expect anchor-cli 2.0.0-rc.1, not 1.1.2
which anchor       # ~/.cargo/bin/anchor either way — the avm shim lives at that same
                   # path, so this only proves something is on PATH, never which build
```

Se isso imprimir `1.1.2`, o seu PATH te entregou a linha estável de novo; releia a tabela das quatro paredes em m01-l2 e corrija a ordenação antes de qualquer outra coisa. Enquanto você está lá, re-carimbe a data `verified` na linha do anchor-cli de `PINS.md`. O pin é `2.0.0-rc.1` na hora em que isto é escrito (agosto de 2026), RCs se mexem, e uma data nova num valor inalterado é o registro de que um humano olhou.

**Passo 1: abra o greeter e acrescente a struct de conta.** No `lib.rs` do seu programa, ao lado do handler `greet` da lição passada, acrescente um tipo de conta minúsculo e duas instruções novas. Este é o programa estendido inteiro:

```rust
use anchor_lang::prelude::*;

declare_id!("3ynNB373Q3VAzKp7m4x238po36hjAGFXFJB4ybN2iTyg");

#[program]
pub mod greeter {
    use super::*;

    // From m01-l3: the greeter. Logs and returns.
    pub fn greet(_ctx: &mut Context<Greet>) -> Result<()> {
        msg!("gm, a player just tapped in");
        Ok(())
    }

    // New: open a marquee account and set its play count.
    pub fn light_marquee(ctx: &mut Context<LightMarquee>, plays: u64) -> Result<()> {
        require!(plays > 0, BarcadeError::DeadMachine);
        ctx.accounts.marquee.plays = plays;
        emit!(MarqueeLit { plays });
        msg!("marquee lit at {} plays", plays);
        Ok(())
    }

    // New: bump two marquee accounts. Two mut Account<Marquee>, no unsafe(dup):
    // the duplicate-mutable guard is armed.
    pub fn tally_two(ctx: &mut Context<TallyTwo>) -> Result<()> {
        ctx.accounts.first.plays = ctx
            .accounts
            .first
            .plays
            .checked_add(1)
            .ok_or(BarcadeError::Overflow)?;
        ctx.accounts.second.plays = ctx
            .accounts
            .second
            .plays
            .checked_add(1)
            .ok_or(BarcadeError::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Greet {
    pub player: Signer,
}

#[derive(Accounts)]
pub struct LightMarquee {
    #[account(mut)]
    pub payer: Signer,
    #[account(init, payer = payer)]
    pub marquee: Account<Marquee>,
    pub system_program: Program<System>,
}

#[derive(Accounts)]
pub struct TallyTwo {
    #[account(mut)]
    pub first: Account<Marquee>,
    #[account(mut)]
    pub second: Account<Marquee>,
}

#[account]
pub struct Marquee {
    pub plays: u64,
}

#[event]
pub struct MarqueeLit {
    pub plays: u64,
}

#[error_code]
pub enum BarcadeError {
    #[msg("a marquee cannot open on zero plays")]
    DeadMachine,
    #[msg("play counter overflowed")]
    Overflow,
}
```

Duas grafias de erro aparecem nesse programa e vale saber por que as duas compilam, porque a segunda parece errada na primeira vez que você encontra ela. `require!(cond, BarcadeError::DeadMachine)` recebe a variante nua; a macro monta o erro para você. `.ok_or(BarcadeError::Overflow)?` entrega a `ok_or` uma variante nua também, produzindo um `Result<_, BarcadeError>`, e depois o `?` converte ele, porque `#[error_code]` gera o impl `From<BarcadeError>` para o tipo de erro do Anchor. Duas grafias, um enum, e este é o par que você vai ver em todo programa deste curso. A única vez em que você recorre a outra coisa é quando você quer a localização de origem estampada no erro, que é o que `error!(BarcadeError::Overflow)` acrescenta.

Note a superfície do V2 que você está olhando direto. Nenhum tempo de vida `<'info>` nas structs de accounts. Handlers recebem `&mut Context<T>`. `LightMarquee` usa `init` sem `space` explícito, porque o V2 infere o tamanho a partir do tipo de conta. Todo campo em todo derive é um carregamento de fase um seguido de constraints de fase dois, exatamente a ordem do diagrama.

![Uma struct de accounts LightMarquee anotada rotulando cada campo com o trabalho de carregamento e de constraint que ele gera, e o corpo do handler como a fase de despacho que roda por último.](assets/v08-annotated-code.webp)

**Passo 2: veja um discriminator com os seus próprios olhos.** Você não precisa chutar qual tag `init` escreve na frente de uma conta `Marquee`. Compute ela:

```bash
echo -n "account:Marquee" | shasum -a 256 | cut -c1-16
```

Esses são exatamente os 8 bytes que `Account<Marquee>` checa em cada carregamento. Faça o mesmo para `global:light_marquee` e `event:MarqueeLit` e você derivou na mão todo discriminator do seu próprio programa. É esse o músculo que o challenge pede.

**Passo 3: arme e observe a trava de duplicate-mutable.** A instrução `tally_two` recebe dois campos `Account<Marquee>` mutáveis sem nenhum `unsafe(dup)`. Isso quer dizer que a trava está ativa. Aqui está um teste que abre um marquee, depois chama `tally_two` passando aquela única conta para *os dois*, `first` e `second`, e afirma que a chamada é rejeitada. É o mesmo formato LiteSVM em Rust do teste que você reescreveu na lição passada, com duas instruções em vez de uma. Acrescente ele ao lado daquele teste:

```rust
use {
    anchor_lang::{
        programs::System, solana_program::instruction::Instruction, Id, InstructionData,
        ToAccountMetas,
    },
    anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction},
};

#[test]
fn rejects_same_mut_account_twice_without_unsafe_dup() {
    let program_id = greeter::id();
    let payer = Keypair::new();
    let marquee = Keypair::new();
    let mut svm = anchor_v2_testing::svm();

    let bytes = include_bytes!("../../../target/deploy/greeter.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Open the marquee: light_marquee inits it with plays = 1.
    let init_ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::LightMarquee { plays: 1 }.data(),
        greeter::accounts::LightMarquee {
            payer: payer.pubkey(),
            marquee: marquee.pubkey(),
            system_program: System::id(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[init_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer, &marquee]).unwrap();
    svm.send_transaction(tx).unwrap();

    // Now pass the SAME account into both mutable slots. No unsafe(dup): the guard is armed.
    let dup_ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::TallyTwo {}.data(),
        greeter::accounts::TallyTwo {
            first: marquee.pubkey(),
            second: marquee.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[dup_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_err(), "expected the duplicate-mutable guard to fire");
}
```

Rode ele:

```bash
anchor test
```

A chamada a `tally_two` nunca chega ao seu handler. O dispatcher percorre as views de conta, sinaliza o endereço repetido, faz o AND desse bitvec contra a `MUT_MASK`, vê um bit sobrevivente, e retorna `ConstraintDuplicateMutableAccount` em tempo de execução, antes de os dois campos sequer carregarem. A sua lógica de `checked_add` é irrelevante aqui, porque a trava dispara antes do corpo. É esse o checkpoint: uma conta mutável duplicada, passada sem `unsafe(dup)`, é uma rejeição em tempo de execução vinda do dispatcher. Você deve ver o teste passar porque o erro foi lançado, que é a trava fazendo o trabalho dela.

![Um fluxo de invocação da esquerda para a direita em que o dispatcher acha o mesmo endereço duas vezes e rejeita com ConstraintDuplicateMutableAccount, então nem o carregamento de contas nem o corpo do handler rodam.](assets/v09-flowchart.webp)

Se você quer comprovar o opt-out para você mesmo, acrescente `unsafe(dup)` aos dois campos de `TallyTwo` e rode de novo. Agora a mesma chamada é aceita, as duas escritas apontam para a mesma conta, e o segundo `checked_add` vê o valor que o primeiro escreveu. É esse o aliasing do qual o V2 te protege por padrão, tornado visível sob demanda.

## Challenge: monte o preimage do discriminator

Este é um problema de completion, e é o challenge de código do módulo. Você recebe uma função que deveria devolver exatamente a string de preimage com namespace que o Anchor faz hash para cada tipo de item. O starter cola: o mapeamento `namespace` dele só ecoa o tipo de item, então o preimage sai como `instruction:increment` onde o Anchor de fato quer `global:increment`, e o build falha nesse caso exato.

```rust
// Anchor derives every 8-byte discriminator by hashing a NAMESPACED preimage:
// sha256("<namespace>:<Name>")[..8]. The bytes come later. The preimage STRING is
// the part you have to get right, and one of the three namespaces is a classic trap.
//
// `namespace` maps an item kind to the namespace Anchor actually hashes from:
//   - an account struct       -> "account"
//   - an instruction handler  -> "global"     <-- NOT "instruction"
//   - an event struct         -> "event"
const fn namespace<'a>(item_kind: &'a str) -> &'a str {
    // TODO: map each item_kind to its real Anchor namespace. Only one of the
    // three differs from its item kind -- that one is the whole exercise.
    item_kind
}

fn discriminator_preimage(item_kind: &str, name: &str) -> String {
    format!("{}:{name}", namespace(item_kind))
}
```

O seu trabalho é mapear os três tipos de item para os namespaces reais deles dentro de `namespace`. Dois dos três mapeiam para si mesmos. Um não, e os critérios de aceitação abaixo te dizem qual. O mapeamento é um `const fn` de propósito — um recurso de tempo de compilação que você vai encontrar de novo no constraint challenge de m03-l3 — então o bloco de verificação entregue debaixo do starter roda no próprio compilador: um mapeamento não corrigido não faz build, e a mensagem de erro nomeia o namespace que ele errou. (Uma consequência de `const fn`: `match` não consegue comparar `&str` direto ali, porque igualdade de string é uma chamada de trait e chamadas de trait não são `const` no Rust estável — faça match em `item_kind.as_bytes()` com padrões de byte-string como `b"instruction"` em vez disso.)

Os critérios de aceitação são exatos:

- `discriminator_preimage("account", "CabinetCounter")` devolve `account:CabinetCounter`
- `discriminator_preimage("instruction", "increment")` devolve `global:increment`
- `discriminator_preimage("event", "HighScore")` devolve `event:HighScore`
- `discriminator_preimage("account", "HighScore")` devolve `account:HighScore` — mesmo nome do caso acima dele, namespace diferente, porque o prefixo é uma função do *tipo*
- `discriminator_preimage("instruction", "initialize")` devolve `global:initialize`

Os cinco casos acima são os vetores empacotados com o challenge, e eles documentam o contrato — mas a correção de produção para Rust checa se o seu código *compila*, e as asserções de tempo de compilação debaixo do starter são o que impõe isso: o build em si falha até o mapeamento `instruction -> global` estar certo. Os dois últimos vetores estão ali de propósito: eles fazem uma busca chaveada no *nome* falhar, que é o atalho que de outro jeito passa nos três primeiros — um atalho que a separação de `namespace` também fecha estruturalmente, já que o mapeamento nunca vê o nome. É uma função simples sem framework no caminho, então se você preferir trabalhar localmente, jogue ela em qualquer crate de rascunho e dirija ela a partir de um `#[test]`:

```bash
cargo test
```

O mapeamento não corrigido do starter falha o build no caso de instruction; a sua solução faz build limpo e passa em todos os cinco vetores.

**Solo, sem apoio, duas partes.** Primeiro, acrescente uma terceira variante ao enum `BarcadeError` do seu programa e, antes de rodar qualquer coisa, escreva o número exato no wire que você espera que um cliente veja quando ela disparar. A seção do layout de erros tem tudo de que você precisa para derivar ele. Depois dispare ela e se confira contra o wire. Segundo, raciocine em uma ou duas frases suas: por que escrever `dup` puro é um erro de compilação, enquanto passar a mesma conta mutável duas vezes é uma rejeição em tempo de execução? Dois eventos diferentes em dois tempos diferentes, e nomear o que cada um sabe é o exercício inteiro. Sem resposta aqui; se a sua frase se sustentar quando você reler a seção da `MUT_MASK`, ela se sustenta.

## Onde isso te deixa

A trava desta lição é pequena e concreta. Complete o challenge discriminator-preimage, starter falhando, solução passando, e diga em uma frase como a `MUT_MASK` de tempo de compilação difere da checagem do dispatcher em tempo de execução. Se o 6002 que você previu bateu com o wire e a sua frase se sustenta, você terminou aqui.

Você agora consegue ler a superfície completa de qualquer programa V2, tanto o lado do programa que você abriu na lição passada quanto o lado das contas que você abriu hoje: a ordem gerada carregamento-constraints-despacho, a trava de duplicate-mutable na emenda dela entre máscara-de-tempo-de-compilação e checagem-em-tempo-de-execução, os três namespaces de discriminator com `global:` como o que morde, e o layout de erros com o framework abaixo de 6000 e o seu código nele e acima dele. Essa é a fiação de segurança inteira de um programa V2, e nada disso é mágica para você mais.

No próximo módulo você dá estado real às ideias do greeter. O `Account<Marquee>` que você escreveu hoje já era zero-copy, porque no V2 é simplesmente isso que `Account<T>` é; um único `u64` acaba sendo Pod-legal sem você pensar nisso, que é exatamente por que a disciplina ficou invisível. No próximo módulo ela para de ser invisível. Você conhece as regras de layout que `Account<T>` vem impondo em silêncio esse tempo todo, descobre o que acontece na primeira vez em que você tenta colocar algo num campo que não é bytes planos, e vê por que `#[event(bytemuck)]` só faz sentido depois que você consegue ler uma struct como bytes. Depois você constrói o R1, o cabinet-counter: o primeiro degrau com dados que valem testar. Você leu a superfície. Agora você faz ela segurar alguma coisa.
