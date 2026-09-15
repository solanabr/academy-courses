# Anatomia do programa: em que declare_id! e #[program] se expandem

Na lição passada você brigou com a instalação, fixou o toolchain do RC no seu arquivo central de pins e fez deploy do R0 na devnet. Faz build, faz deploy, abre uma conta. E ainda é uma caixa-preta: duas macros, `declare_id!` e `#[program]`, umas duas dúzias de linhas de Rust, e nenhuma ideia real do que elas viram.

Deixa eu consertar isso agora, antes de qualquer teoria. Do seu workspace do greeter, rode o teste que o scaffold já escreveu:

```bash
anchor test
```

Esse comando faz o build do programa e roda o teste Rust que o `anchor init` deixou do lado dele, o que inicializa o counter do scaffold e faz asserções sobre os campos dele. Olhe o fim da saída procurando `test result: ok. 1 passed`. Você acabou de invocar o seu próprio handler numa VM in-process, e o motivo de esse teste ser Rust e não TypeScript é a primeira coisa que vale entender hoje. Guarde esse pensamento. A gente vai ler exatamente o que essas duas macros geram, e depois fechar o loop de invocação na mão em um programa que você deixou no osso.

## Resumo

A rota é esta. Você já sabe que o greeter compila e faz deploy. O que você ainda não sabe é a *forma* em que ele compila: a assinatura do handler, os tipos de account, o tipo de endereço, o caminho de despacho dos bytes crus até o seu código. Então a gente trabalha de trás para frente, do código que você escreveu até a superfície que as macros geram, na ordem em que o framework realmente usa ela. Depois você leva essa superfície para um lab: reescrever o teste LiteSVM do scaffold em volta do seu handler no osso, preencher o único TODO dele para que ele acerte o `greet`, e confirmar pela CLI que a cópia que você fez deploy na lição passada continua viva no id do seu arquivo de pins.

Uma ressalva de saída, e é a honesta. Isto é o Anchor V2, um RC. A imensa maioria do código, dos tutoriais e das respostas de Stack Overflow que você vai encontrar ainda importa `@coral-xyz/anchor` e a forma do v1. Tudo abaixo é a expansão do V2, não a do v1. Onde eles divergem, eu mostro os dois, porque ler o ecossistema vai te custar um imposto de tradução e é melhor você já aprender a taxa de câmbio agora.

## Lendo a expansão

Comece com o greeter inteiro em uma tela. Este é o código-fonte, nada gerado ainda:

```rust
use anchor_lang::prelude::*;

declare_id!("3ynNB373Q3VAzKp7m4x238po36hjAGFXFJB4ybN2iTyg");

#[program]
pub mod greeter {
    use super::*;

    pub fn greet(_ctx: &mut Context<Greet>) -> Result<()> {
        msg!("gm, a player just tapped in");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Greet {
    pub player: Signer,
}
```

O seu scaffold da lição passada ainda é o counter gerado: um handler `initialize` que abre um `Account<Counter>`, mais o módulo `state` dele. Troque tanto o handler quanto a struct `Initialize` por essa forma do `greet` agora, e apague o módulo `state` e a linha `use state::Counter;` junto com eles. Você quer o menor programa que ainda comprova alguma coisa: um signatário, uma linha de log, nenhum estado. Troque o `declare_id!` pelo seu próprio program id também; o de cima é só um substituto.

No momento em que você apaga o `initialize`, o workspace para de compilar, e vale saber por quê antes de ver o erro. O teste do scaffold em `programs/greeter/tests/test_initialize.rs` nomeia `greeter::instruction::Initialize` e `greeter::accounts::Initialize`, os dois gerados *a partir* do handler e da struct que você acabou de remover. Esses símbolos não existem mais, então o teste falha em compilar. Esse teste não é uma coisa que você apara; é uma coisa que você reescreve, e o Lab abaixo reescreve ele. Faça a edição do programa e a edição do teste juntas, depois faça o build de novo, para o `.so` e os builders gerados combinarem com o que você lê a seguir.

Uma coisa que essa linha de log *não* faz é imprimir o endereço do player, e o motivo é uma restrição genuína do V2 que vale conhecer cedo em vez de como um erro de compilação misterioso. `Address` só implementa `Display` e `Debug` quando o crate `solana-address` tem a feature `decode` ligada, porque codificação base58 é exatamente o tipo de peso que um framework `no_std` se recusa a carregar por padrão, e o anchor-lang não habilita ela. Então `msg!("gm, {}", ctx.accounts.player.address())` não compila em um programa V2 de fábrica. Seja preciso sobre o que falta ali: `msg!` formata bem, e `msg!("lit at {} plays", plays)` em um `u64` é código V2 comum que você vai escrever na próxima lição. O buraco é `Display` em `Address` especificamente, então são endereços que você não pode interpolar, não valores em geral. Se você realmente precisa de um endereço base58 em um log, você liga essa feature de propósito (a feature `compat` do anchor-lang traz ela, junto com uma macro `debug!`) e paga por isso em tamanho de binário e compute units. O default é silêncio sobre endereços, e o default é o ponto.

![Uma tabela de quatro linhas mostrando que literais e valores u64 formatam bem em msg! enquanto um Address não, até que a feature compat seja habilitada deliberadamente, a um custo de tamanho e de compute.](assets/v01-table.png)

A lição passada nomeou as mudanças de superfície de passagem e prometeu que você ia abrir elas aqui. É isto. Três delas estão visíveis nas onze linhas acima: `&mut Context`, não `Context`. `Signer`, não `Signer<'info>`. E nenhum `Pubkey` em lugar nenhum. A quarta veio de carona na linha do scaffold que você está trocando, `ctx.accounts.counter.authority = *ctx.accounts.payer.address();`: `.address()`, não `.key()`. Nomear elas era o trabalho da lição passada. Derivar por que cada uma tem essa forma, e o que a macro gera em volta dela, é o desta. Pegue uma por vez.

### declare_id! ainda só declara o endereço

`declare_id!` é a calma. Ela pega o endereço base58 do programa e gera, mais ou menos, isto:

```rust
use anchor_lang::prelude::*;

// what declare_id! expands to, in spirit. The real macro decodes your base58
// string into these 32 bytes at compile time; zeros stand in for them here.
pub static ID: Address = Address::new_from_array([0u8; 32]);
pub fn id() -> Address { ID }
pub fn check_id(id: &Address) -> bool { id == &ID }
```

Uma constante `ID`, um acessor `id()` checado e um helper `check_id`, para o resto do programa e o ponto de entrada poderem comparar o endereço com que ele foi invocado contra o endereço para o qual ele foi compilado. Esse trabalho não mudou entre v1 e V2, e o nome da macro também não. A única coisa que mudou vem de carona sem fazer barulho nos tipos acima: a constante é um `Address`, não um `Pubkey`. Guarde isso, a gente volta nele duas seções abaixo.

O valor no seu é o que o `anchor keys sync` escreveu depois do seu deploy de m01-l2; o de cima é só um substituto. Coloque o program id do seu arquivo de pins quando for acompanhar, ou o `.so` construído vai carregar o endereço errado e toda invocação vai ser barrada na checagem de id.

Essa checagem de id não é decoração. É a primeiríssima coisa que o ponto de entrada gerado faz em toda chamada: dar `check_id` no program id declarado contra o program id de entrada, rejeitar se eles diferirem. Guarde isso, porque é o passo um do caminho de despacho que a gente alcança em breve. É também por que um `.so` construído com o `declare_id!` de outra pessoa nasce morto: o programa se recusa a rodar como um endereço para o qual não foi compilado.

### #[program]: o handler, e o &mut que te surpreende

`#[program]` marca o módulo que guarda os seus handlers de instrução. Toda função pública dentro dele se torna uma instrução para a qual o programa pode despachar. Isso tudo é v1 também. O que o V2 mudou é a assinatura do handler, e é a mudança em que você vai tropeçar primeiro.

No v1 um handler recebia o context dele *por valor*: `pub fn greet(ctx: Context<Greet>)`. No V2 ele recebe uma *referência mutável*: `pub fn greet(ctx: &mut Context<Greet>)`. Todo handler, uniformemente, até um como o `greet` que só lê. O framework constrói o `Context`, te entrega um `&mut` para ele, e reaproveita ele até a rotina de saída que persiste as mudanças nas suas contas. Você não constrói ele e você não devolve ele. Você faz um borrow dele, altera através dele, retorna `Ok(())`.

![A mesma instrução greet em v1 e V2, marcando quatro mudanças: uma referência mutável de Context, .address() substituindo .key(), Address substituindo Pubkey, e os lifetimes info removidos.](assets/v02-annotated-code.png)

Por que `&mut`, afinal, se o `greet` nunca escreve? A resposta franca é que o context por valor era um custo pequeno pago em toda instrução: o framework movia um context para dentro da sua função, você fazia o seu trabalho, ele movia o estado das contas de volta para fora. Uma referência remove o move e deixa o mesmo context atravessar a validação, o seu handler e a rotina de saída como uma coisa só, em borrow. É um ganho pequeno de ergonomia e de custo, e é da mesma peça que a tese inteira do V2 desde a lição um: pare de copiar o que você pode pegar por borrow ou fazer cast no lugar. Você não precisa amar a sintaxe. Você precisa reconhecer ela, porque um handler escrito `Context<T>` por valor não vai compilar contra o RC, e a mensagem de erro aponta para a assinatura, não para a causa.

### Pubkey agora é Address, e .key() agora é .address()

Volte na linha do scaffold que você acabou de apagar: `ctx.accounts.counter.authority = *ctx.accounts.payer.address();`. No v1 essa linha seria `ctx.accounts.payer.key()` e o tipo seria `Pubkey`. No V2 os dois nomes mudaram. O tipo é `Address` (do crate `solana-address` que o pinocchio traz), e o acessor é `.address()`, que te entrega uma referência, daí o `*` na frente quando você quer uma cópia própria para guardar.

Isso é uma renomeação direta de duas coisas que você usa sem parar, e é exatamente por isso que dói. Cada chave de conta que você lê, cada endereço que você compara, cada pubkey que você guardou em um campo de struct: `Pubkey` vira `Address`, `.key()` vira `.address()`. Não existe `.pubkey()` nem `.to_address()` na superfície do wrapper; o acessor é `.address()`, ponto. Seus dedos vão digitar `.key()` por pura memória muscular nas três primeiras vezes. Os meus digitaram. O compilador vai te parar todas as vezes, então é um hábito barato de quebrar, mas é um hábito.

A renomeação não é troca cosmética, e vale entender de onde ela vem para parar de parecer arbitrária. O V2 é uma reescrita no_std construída sobre o pinocchio, e o pinocchio traz o próprio tipo de endereço pelo crate `solana-address` em vez do mais antigo `solana-program::Pubkey`. Então, quando o Anchor V2 se assenta nessa fundação, o tipo que ele te entrega lá em cima é o que a fundação fala: `Address`. A mudança de `.key()` para `.address()` é o acessor seguindo o tipo. Leia assim e o padrão generaliza: a maior parte do que parece novo em uma assinatura V2 é a fundação pinocchio aparecendo através do framework em vez de ser encoberta. É a mesma tese da lição um, vista do lado dos tipos e não do lado do compute.

![Uma tabela de cinco linhas mapeando v1 para V2, cobrindo a assinatura do handler, Pubkey para Address, .key() para .address(), o lifetime removido do wrapper de account, e a leitura do bump que é idêntica nos dois lados porque o mapa de strings morreu no 0.29.](assets/v03-comparison.png)

### Os lifetimes <'info> saíram dos wrappers

Agora a struct de accounts. No v1 todo wrapper de account carregava um lifetime explícito, e a struct também: `pub struct Greet<'info>` com `pub player: Signer<'info>` dentro. Esse `<'info>` passava por tudo e era de longe a coisa que iniciantes mais erravam, porque o borrow checker tinha opiniões sobre isso e as mensagens de erro eram densas.

No V2 a superfície do wrapper não carrega `<'info>` nenhum. A sua struct é `pub struct Greet`, o seu campo é `pub player: Signer`. O lifetime não se mudou para o módulo `#[program]` nem foi se esconder em outro lugar. Ele saiu da superfície que você escreve. O framework ainda rastreia os lifetimes dos dados da conta subjacente por dentro, mas a contabilidade no nível de tipos que você escrevia na mão agora é problema da macro, não seu. Para quem vem do v1, é essa a mudança que faz uma struct de accounts do V2 parecer limpa até demais, como se faltasse alguma coisa. Não falta nada. Sempre foi trabalho do framework; o V2 finalmente parou de te fazer digitar isso.

Existe uma quinta renomeação que você não vai ver no greeter porque ele ainda não tem PDAs, mas ela pertence à mesma lista para não te dar uma emboscada depois: o Anchor mais antigo lia um bump armazenado com `ctx.bumps.get("vault")`, uma busca em mapa que retornava um `Option`. A linha 0.2x-to-0.3x já moveu isso para acesso de campo, e o V2 mantém: `ctx.bumps.vault`. É a única linha daquela tabela que não é uma mudança do V2, e está ali porque quem tem memória muscular antiga ainda vai buscar o mapa. Ela cai no momento em que você escreve a sua primeira conta com seeds, que é o módulo de PDAs, dois módulos daqui.

### Como bytes crus chegam de verdade no greet

Você leu as peças. Agora veja elas rodando, porque "a macro gera um ponto de entrada" não é uma explicação até você conseguir rastrear uma invocação por ela. Quando alguém invoca o seu programa, o código gerado faz quatro coisas em ordem.

Primeiro, o ponto de entrada checa se o program id declarado (do `declare_id!`) casa com o program id com que ele foi de fato invocado, e dá erro se não. Segundo, ele lê o começo dos dados de instrução e casa contra o discriminator de cada handler, a etiqueta pequena que diz "esta chamada é para o `greet`, não para outro handler." Terceiro, o wrapper do handler que casou desserializa as contas indicadas na transação para dentro da sua struct `Greet`, rodando cada constraint e cada checagem no caminho, e constrói o `Context`. Quarto, ele chama o código que você escreveu de verdade, o `greet`, com um `&mut` para esse context, e depois roda uma rotina de saída que persiste de volta qualquer mudança nas contas.

![Um fluxo de seis passos do ponto de entrada até a saída: checagem de program-id, casamento de discriminator, desserialização de contas para um Context, o corpo do handler greet, e depois a rotina de saída que persiste as mudanças.](assets/v04-flowchart.png)

Tudo menos o corpo do handler é gerado: a checagem de id, o despacho, a desserialização e a rotina de saída que persiste as mudanças. O único código que você escreveu é o corpo do `greet`. É essa a troca inteira de um framework: ele escreve o despacho, a desserialização, as checagens de constraint e a persistência, e em troca esconde uma fiação sobre a qual você ainda precisa conseguir raciocinar quando algo dá errado. Ler a expansão é como você fica com o raciocínio mesmo que a macro fique com a digitação.

Um detalhe do passo dois ganha um nome agora e tratamento completo na próxima lição. A etiqueta em que o dispatcher casa é um discriminator: os bytes da frente dos seus dados de instrução que dizem para qual handler é esta chamada. O Anchor computa ele a partir do nome do handler, então `greet` e `greet_high_score` recebem discriminators diferentes e nunca colidem, e o builder gerado `greeter::instruction::Greet` que você vai usar no lab estampa o certo nos bytes para você. É só isso que você precisa hoje: os bytes entram, o dispatcher lê o discriminator, o wrapper que casa roda. Como o discriminator é de fato derivado, por que um handler de instrução faz hash a partir de um namespace que surpreende as pessoas, e onde vivem os códigos de erro customizados, é a lição seguinte inteira sobre `#[derive(Accounts)]`. Um conceito por lição; esta é o lado do `#[program]`.

## Lab: fechar o loop de invocação

Hora de fazer o handler rodar. O veículo é o teste Rust LiteSVM que o `anchor init` já gerou no scaffold, reescrito em volta do programa que você acabou de deixar no osso. O passo a passo aqui é totalmente trabalhado; você acompanha exatamente. As duas coisas que você preenche sozinho vêm no Challenge logo depois.

Uma observação de escopo, para você não ficar esperando uma coisa que não vem. Invocar pela linha de comando o programa que você fez deploy não é algo que a CLI `solana` pura consiga fazer: ela não sabe computar um discriminator do Anchor nem empacotar account metas, então não existe encantamento `solana` que chame o `greet`. Construir um cliente que consiga é o trabalho inteiro do módulo 8, depois que você tiver um cliente gerado para construir a partir dele. Hoje o loop de invocação se fecha no LiteSVM, e o deploy na devnet é confirmado como deploy, não como chamada.

Um aparte rápido sobre por que o teste default é Rust e não TypeScript, já que você está encarando ele. O `anchor init` no V2 oferece cinco templates de teste: Mocha, Jest, Rust, Mollusk e Litesvm. Litesvm é o `#[default]`. Então o scaffold te entrega um teste de integração em Rust que carrega o seu `.so` compilado para uma VM in-process e invoca ele, sem nenhum TypeScript escrito e sem nenhum validador local iniciado.

![Uma tabela dos cinco templates de teste do anchor init, separando as opções de TypeScript-contra-validador das opções Rust in-process, com Litesvm marcado como o default.](assets/v05-comparison.png)

Por que esse default, e não um de TypeScript como em todo tutorial de v1 que você já leu? Porque no V2 a superfície Rust é o cliente primário, e ainda não existe pacote TypeScript oficial do V2 para pegar. Mas a escolha também é uma aposta, e Jacob Creech nomeou ela no memo de unificação dele: "I expect Anchor V2 to unify tools around using Litesvm, using the solana-verify standard, potentially surfpool, Gill." O loop de invocação default do LiteSVM que você está a ponto de rodar é esse memo sendo entregue. Ele rima com os incrementos datados do Anchor 1.0 que você narrou na lição passada, a mesma onda de decisões que renomeou o pacote TypeScript e trocou o validador default.

![Cinco incrementos do Anchor 1.0 mostrados como um conjunto: a renomeação do pacote, CpiContext recebendo um Pubkey, transfer_checked, LiteSVM como o template de teste default, e Surfpool como o validador default.](assets/v06-timeline.png)

É também por isso que este módulo inteiro fecha o loop de deploy e invocação em Rust, em vez de te entregar uma bancada de teste em TypeScript que ainda não existe.

**Passo 1. Reescreva o teste do scaffold.** O template LiteSVM escreve o teste dele do lado do crate do programa, não na raiz do workspace: abra `programs/greeter/tests/` e ache o `test_initialize.rs`. Renomeie o arquivo para `test_greet.rs` e troque o corpo dele pela versão abaixo. Isso é uma reescrita, não uma aparada: todo símbolo `Initialize` do arquivo antigo foi gerado a partir do handler que você acabou de apagar, então nada nele sobrevive à troca. Ele fica assim:

```rust
use {
    anchor_lang::{
        solana_program::instruction::Instruction, InstructionData, ToAccountMetas,
    },
    anchor_v2_testing::{Keypair, LiteSVM, Message, Signer, VersionedMessage, VersionedTransaction},
};

#[test]
fn test_greet() {
    let program_id = greeter::id();
    let payer = Keypair::new();
    let mut svm = anchor_v2_testing::svm();

    let bytes = include_bytes!("../../../target/deploy/greeter.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let ix = Instruction::new_with_bytes(
        program_id,
        &greeter::instruction::Greet {}.data(),
        greeter::accounts::Greet {
            // TODO: name the accounts greet needs
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "send_transaction failed: {:?}", res);
}
```

Uma coisa naquele bloco de imports merece uma nota antes de você ler o corpo, porque parece contradizer tudo acima. O teste busca `anchor_lang::solana_program::instruction::Instruction` e chama `payer.pubkey()`, exatamente os dois nomes que esta lição acabou de dizer que o V2 substituiu. Os dois estão corretos. As renomeações são uma mudança do *lado do programa*: dentro do seu programa, nos wrappers que o derive gera, o tipo é `Address` e o acessor é `.address()`. Código off-chain, que é o que um teste é, ainda fala os tipos de `solana-program` / `solana-sdk` que o runtime e o SDK sempre usaram, então um `Keypair` ainda te entrega um `Pubkey` pelo `.pubkey()`. Você está olhando a costura entre as duas superfícies, e ela fica ali pelo curso inteiro: on-chain é `Address`, off-chain é `Pubkey`.

Agora leia o que o teste faz contra o fluxo de despacho que você acabou de rastrear. `anchor_v2_testing::svm()` levanta a VM in-process. `add_program` insere o seu `.so` construído no program id dele. `greeter::instruction::Greet {}.data()` produz os bytes de instrução, discriminator incluído, em que o passo dois casa. `greeter::accounts::Greet { ... }.to_account_metas(None)` produz os account metas que o wrapper desserializa no passo três. Esses módulos `instruction::` e `accounts::` são gerados a partir do seu programa pelas mesmas macros; o teste não lê uma IDL em tempo de execução, ele usa os builders tipados diretamente.

![Um único greeter.so construído a partir do lib.rs alimenta duas provas, uma execução local in-process do handler no LiteSVM e uma conta na devnet que resolve como executável.](assets/v07-diagram.png)

**Passo 2. Faça o build e rode, e espere vermelho.** Da raiz do workspace:

```bash
anchor test
```

`anchor test` faz o build do programa, roda o teste LiteSVM e pula o início de um validador porque a VM in-process não precisa de um. Ele deve **falhar em tempo de compilação**, no arquivo de teste, com `error[E0063]: missing field 'player' in initializer of 'Greet'` — o builder `accounts::Greet` ainda é um TODO vazio, e um literal de struct em Rust precisa nomear todos os campos, então o compilador te para antes de qualquer coisa ser enviada. Esse vermelho é o estado correto para se estar agora. Fica vermelho até você preencher o TODO no Challenge, e deixar ele verde é a metade avaliada desta lição. O que você quer confirmar neste passo é mais estreito: o crate do programa em si compila depois da reescrita, e o único erro que sobrou nomeia exatamente o campo que você está a ponto de preencher — não um símbolo faltando por causa das renomeações do V2.

**Passo 3. Confirme que o deploy continua vivo.** O teste LiteSVM é onde o `greet` roda. Em separado, comprove que a cópia que você subiu em m01-l2 continua lá e continua sua. Aponte a CLI para o seu program id da devnet, do arquivo de pins:

```bash
solana program show <YOUR_GREETER_PROGRAM_ID> --url devnet
```

Isso imprime a conta do programa, o tamanho dos dados dela e a autoridade de upgrade dela. Três fatos que valem ser lidos em vez de passados de olho: a conta existe, ela é executável, e a autoridade de upgrade é a sua carteira, que é o que te deixa fazer deploy por cima dela depois. Note o que isso *não* faz. Isso não chama o `greet`, e nada nesta lição chama, na devnet. Ter deploy e ser invocável são duas afirmações diferentes, e hoje você comprova a primeira pela CLI e a segunda no LiteSVM.

## Challenge

Duas tarefas. A primeira é um completion, a segunda é com você. É aqui que as rodinhas saem, uma roda por vez.

**Completion, o TODO.** No `test_greet.rs`, preencha o builder `accounts::Greet { ... }` com as contas que o `greet` realmente precisa. Olhe o seu `#[derive(Accounts)] pub struct Greet` para a resposta: ele tem um campo, `player: Signer`, e o signatário é o `payer` do teste. Uma linha. Aí o `anchor test` deve virar a falha anterior em `1 passed`. Se você ainda estiver vermelho, releia o nome do campo na sua struct de accounts contra a chave que você colocou no builder; uma divergência ali é a falha inteira.

**Solo, a segunda variante.** Adicione uma segunda instrução de saudação ao módulo `#[program]`, reaproveitando os mesmos accounts de `Greet`, e comprove que ela despacha. A forma a buscar:

```rust
pub fn greet_high_score(_ctx: &mut Context<Greet>) -> Result<()> {
    msg!("someone is going for the high score");
    Ok(())
}
```

Faça o build de novo, depois escreva um segundo `#[test]` do lado do primeiro que envie `greet_high_score` em vez de `greet`. Nenhuma dica além do que o Passo 1 já te mostrou: as duas únicas linhas que mudam são o builder da instrução e o nome do teste, e descobrir em qual builder a nova instrução aparece é justamente o ponto. Se você consegue ler o fluxo de despacho, você sabe por que uma segunda função pública no módulo se torna uma segunda instrução despachável com o discriminator dela.

Aceitação: o `anchor test` reporta `test result: ok. 2 passed`.

## Checkpoint

Você terminou quando três coisas são verdade ao mesmo tempo: o workspace compila depois da reescrita, o `anchor test` reporta os dois testes de saudação passando, e o `solana program show` resolve o seu greeter na devnet como uma conta executável de que você detém a autoridade de upgrade. Se qualquer uma delas faltar, o loop não está fechado e a próxima lição vai parecer que pulou um passo.

Vale nomear o que mudou na sua cabeça, não só no seu terminal. O greeter tem uma dúzia de linhas mal contadas, é menor que o scaffold com que você começou a hora, e não é mais uma caixa-preta. Você consegue rastrear uma invocação dos bytes crus até a checagem de id, o casamento de discriminator, o wrapper, o seu handler e a rotina de saída, e consegue nomear toda renomeação do V2 que você encontrar lendo o programa de outra pessoa: `&mut Context<T>`, `Address` e `.address()`, nenhum `<'info>`, `ctx.bumps.name`. Um último lembrete, porque vai te poupar uma tarde: o ecossistema ainda é v1 de forma esmagadora. Quando um tutorial escrito no ano passado mostra `Context<T>` por valor e `.key()`, você agora sabe que não está errado, só está na outra linha. Até fontes cuidadosas atrasam aqui. A Helius, uma referência em qualidade de documentação, ainda entrega o seu principal texto de introdução ao Anchor com uma instalação 0.29.0 em 2026. Ler tutoriais de anatomia antigos engana justamente porque a anatomia mudou.

Agora você consegue ler o lado do `#[program]` de qualquer programa V2. Mas o lado dos accounts, `#[derive(Accounts)]`, é onde os constraints, o despacho, os discriminators e os erros de fato vivem, e ele esconde o detalhe mais afiado do V2 de todos: uma máscara em tempo de compilação imposta por uma checagem em tempo de execução. É o que vem a seguir, e é a coisa mais afiada deste módulo.
