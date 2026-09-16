# Enums são máquinas de estados

## Resumo

m04-l2 transformou todo desfecho de sonda num Result: um motor tipado com thiserror, um binário anyhow, a sua primeira closure no map_err, e matemática de lamports checada. A execução sobrevive a fixtures sujas agora. O que ela não sobrevive é ao tempo. Um alvo não é só "ok neste instante"; ele tem uma história, e nada na estação acompanha isso: a frota TS classifica cada sonda isoladamente (os vereditos de m02-l1: up, degraded, down) e esquece. Hoje o motor Rust escreve a metade que falta do cânone: a vida de um alvo como quatro estados, Pending, Up, Degraded, Down, um enum cujo match É a tabela de transição, e transições ilegais deixam de ser bugs que você pega e viram programas que não existem. Depois o outro fio do módulo aterrissa: cargo test, clippy e fmt entram no pipeline do Actions como gate #3, e as duas linguagens acabam travadas num workflow só. A pergunta condutora da lição inteira: como você muda um código do qual você tem medo?

## Quebre a máquina primeiro

Comece levando uma recusa, de propósito, em menos de três minutos. Abra o `pulse-rs`, crie um arquivo de rascunho `src/bin/scratch.rs`, e cole exatamente isto, braço faltando e tudo:

```rust
#[derive(Debug, Clone, Copy)]
enum ProbeState {
    Pending,
    Up,
    Down,
}

fn next_state(state: ProbeState, probe_ok: bool) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Up, false) => Down,
        (Down, false) => Down,
    }
}

fn main() {
    println!("{:?}", next_state(ProbeState::Pending, true));
}
```

```bash
cargo check
```

O compilador responde com o caso que falta, por nome:

```text
error[E0004]: non-exhaustive patterns: `(ProbeState::Down, true)` not covered
  --> src/bin/scratch.rs:10:11
   |
   |     match (state, probe_ok) {
   |           ^^^^^^^^^^^^^^^^^ pattern `(ProbeState::Down, true)` not covered
```

Leia esse erro de novo, devagar, porque ele é a lição. Você não escreveu um teste para o caminho de recuperação. Você não lembrou do caminho de recuperação. Você esqueceu dele, do jeito que todo mundo esquece um caso, e o compilador imprimiu o caso esquecido por nome e se recusou a fazer o build até você decidir o que significa a sonda bem-sucedida de um alvo Down. Em m02-l1 você comprou essa garantia exata para a frota TS com o truque do `assertNever`: uma função esperta que você tinha que conhecer, ligar na fiação, e lembrar em todo switch. Aqui isso é o comportamento padrão do `match`. Ninguém opta por ele. A caneta está na mão do compilador.

Adicione o braço `(Down, true) => Up,` e o `cargo check` fica quieto. Apague o arquivo de rascunho quando terminar de quebrar as coisas; a máquina de verdade vai no motor.

## A caneta está na mão do compilador

### Estados como dados, transições como braços

A vida de um alvo precisa de quatro estados, então o motor ganha um enum de quatro variantes. Isto vai em `src/engine.rs`, do lado do trabalho de ProbeError da lição passada:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeState {
    Pending,
    Up,
    Degraded,
    Down,
}
```

O atributo `derive` é geração de código que você ganha de graça: `Debug` te dá impressão com `{:?}`, `Clone` e `Copy` deixam o tipo barato de passar por valor (ele tem um byte, copiar ele é mais barato do que pensar em pegar ele emprestado), e `PartialEq`/`Eq` deixam o `assert_eq!` comparar estados nos testes que você está prestes a escrever. Você vem usando derive desde m04-l1 sem cerimônia. Essa é a quantidade certa de cerimônia.

Agora a máquina em si. As regras são cânone novo, escrito hoje; elas estendem os vereditos por sonda da frota TS (que julgam uma resposta) para uma política sobre a vida de um alvo ao longo de várias respostas, e a metade TS vai adotar os mesmos estados quando as duas metades se encontrarem. Um primeiro sucesso leva um alvo Pending para Up, uma falha degrada um alvo Up em vez de matar ele, um alvo Degraded só morre depois de três falhas consecutivas, e qualquer sucesso recupera direto para Up. Escreva as regras como um match sobre o par:

```rust
pub fn next_state(state: ProbeState, probe_ok: bool, consecutive_failures: u32) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Up, false) => Degraded,
        (Degraded, true) => Up,
        (Degraded, false) if consecutive_failures >= 3 => Down,
        (Degraded, false) => Degraded,
        (Down, true) => Up,
        (Down, false) => Down,
    }
}
```

![Quatro estados de sonda conectados por arestas de ok e de falha, com a aresta que falta de Pending para Degraded riscada porque nenhum braço de match cria ela.](assets/v01-flowchart.webp)

Duas coisas nesse match são novas, e as duas se pagam. A tupla `(state, probe_ok)` deixa um match só cobrir a grade inteira de estado-vezes-desfecho, que é exatamente o formato de uma tabela de transição. E `if consecutive_failures >= 3` é uma trava de match: uma condição extra parafusada num braço só. Travas vêm com uma regra que vale dizer em voz alta, porque ela vai te morder na primeira vez que você se apoiar nelas: o compilador não consegue enxergar dentro do booleano de uma trava, então um braço com trava não conta para a exaustividade. É por isso que `(Degraded, false)` aparece duas vezes, uma com trava e uma pelado. Apague o pelado e o E0004 volta, te dizendo que o braço com trava sozinho não é uma promessa.

Aqui está a parte que levou um tempo vergonhoso para eu internalizar quando aprendi isso: repare no que NÃO está na função. Nenhuma checagem de `if state is valid`. Nenhum ramo de erro para transições ilegais. A regra "Pending nunca vai direto para Degraded" não mora em lugar nenhum, porque ela não precisa morar em lugar nenhum. Não existe braço que produza ela, então não existe caminho de código que execute ela, então o motor não consegue fazer isso, do mesmo jeito que a sua união de m02-l1 não conseguia representar uma sonda bem-sucedida sem uma latência. Irrepresentável ganha de validado. Validação que você nunca precisa lembrar ganha de validação que alguém uma hora vai esquecer.

### A união que você já entregou, vestindo uma bandeira

Ponha os dois artefatos lado a lado, porque você já desenhou esta máquina uma vez e eu me recuso a fingir o contrário. Do seu `pulse-core`, m02-l1:

```typescript
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };

function assertNever(value: never): never {
  throw new Error(`unhandled variant: ${JSON.stringify(value)}`);
}

switch (result.kind) {
  case 'ok': /* ... */ break;
  case 'timeout': /* ... */ break;
  case 'http-error': /* ... */ break;
  default:
    assertNever(result); // compile error here if a variant is unhandled
}
```

Um match sobre um enum de Rust É aquele switch exaustivo, com uma diferença que muda a experiência do dia a dia: o never-hack sumiu. Em TS, exaustividade era um padrão que você aplicava; pule o `assertNever` e o switch compila feliz com um caso faltando. Em Rust é a semântica do `match`; não existe versão não checada em que você caia por acidente. Mesma garantia, mas numa linguagem você carrega ela e na outra a linguagem carrega você. E variantes de Rust carregam dados direto (`Ok { latency: LatencyMs }` de m04-l1) onde TS soletra isso como formatos de objeto com um campo discriminante. Sintaxe diferente, mesmo tipo soma, e uma das costuras TS-para-Rust deste curso se fecha bem aqui: tudo que você aprendeu sobre modelar com uniões transfere um para um.

![Uma união discriminada de TypeScript com o switch exaustivo dela ao lado do enum e do match equivalentes em Rust, mostrando que a garantia é opt-in de um lado e padrão do outro.](assets/v02-annotated-code.webp)

### Structs, impl, derive: o trio que você já conhece pela metade

Você vem usando os três desde m04-l1, então isto é nomear, não ensinar. Uma `struct` é o seu tipo de registro. Um bloco `impl` pendura funções num tipo; um método que recebe `&self` lê, `&mut self` muta. Aqui está o que esta lição de fato precisa, uma conveniência que o código de renderização de status vai chamar:

```rust
impl ProbeState {
    pub fn is_alerting(&self) -> bool {
        matches!(self, ProbeState::Degraded | ProbeState::Down)
    }
}
```

`matches!` é uma macro de atalho: ela expande para um match que devolve true para os padrões listados e false para todo o resto. O que significa, e guarde isso para o lab, que ela contém um braço `_ => false` escondido. É um catch-all disfarçado. Isso vai importar daqui a uns vinte minutos.

### Um trait, porque uma fonte deveria ser trocável

O motor ainda lê latências de dados de fixture, e em m05-l3 ele ganha um braço de sonda HTTP de verdade. Essas são duas fontes para a mesma pergunta: qual é a próxima latência? Em TS você buscaria uma interface. A palavra do Rust é trait:

```rust
pub trait ProbeSource {
    fn next_latency(&mut self) -> Option<u64>;
}
```

Essa assinatura é um contrato congelado neste curso: pegue ela textualmente, porque o `drive` abaixo é escrito contra ela e nada mais adiante tem permissão de remodelar ela. `&mut self` porque uma fonte avança conforme você puxa dela; `Option<u64>` porque toda fonte uma hora seca, e você já sabe de m04-l2 que "talvez um valor" se escreve Option, não um sentinela como `-1`. Uma nota honesta sobre o que vem pela frente, para que este soquete nunca vire uma promessa que o curso larga caladinho: o braço HTTP de m05-l3 é uma chamada isolada que devolve uma medição, não uma implementação de `ProbeSource`, e aquela lição diz em voz alta por que mantém os dois separados. O trait é a costura que deixa o `drive` rodar sobre fixtures hoje e deixa uma segunda fonte se encaixar no dia em que uma ganhar o lugar dela; esse dia é m06-l1, quando uma fonte ao vivo apoiada em reqwest finalmente ocupa o soquete. A implementação de hoje é a apoiada em fixture:

```rust
pub struct FixtureSource {
    latencies: Vec<u64>,
    cursor: usize,
}

impl FixtureSource {
    pub fn new(latencies: Vec<u64>) -> Self {
        Self {
            latencies,
            cursor: 0,
        }
    }
}

impl ProbeSource for FixtureSource {
    fn next_latency(&mut self) -> Option<u64> {
        let latency = self.latencies.get(self.cursor).copied();
        self.cursor += 1;
        latency
    }
}
```

`impl Trait for Type` é toda a cerimônia: ele declara que FixtureSource cumpre o contrato de ProbeSource, e o compilador confere se a assinatura bate à risca. Código que consome uma fonte nomeia o trait, não a struct:

```rust
pub fn drive<S: ProbeSource>(source: &mut S, budget_ms: u64) -> ProbeState {
    let mut state = ProbeState::Pending;
    let mut consecutive_failures: u32 = 0;
    while let Some(latency) = source.next_latency() {
        let probe_ok = latency <= budget_ms;
        if probe_ok {
            consecutive_failures = 0;
        } else {
            consecutive_failures += 1;
        }
        state = next_state(state, probe_ok, consecutive_failures);
    }
    state
}
```

Esse `<S: ProbeSource>` é um bound genérico: "qualquer tipo S, desde que ele implemente ProbeSource". Você está lendo genéricos-em-assinaturas agora, e ler é tudo que este curso pede de você; escrever as suas próprias abstrações genéricas, trait objects e o vocabulário de bounds são exatamente a profundidade que a caixa abaixo deixa como bookmark. `while let` é o irmão caçula do match: faça loop enquanto o padrão casar, destruture o Some, pare no None.

![Uma função drive conectada a um soquete de fonte de sonda que aceita um plugue de fixture agora e um plugue HTTP num módulo mais adiante.](assets/v03-diagram.webp)

### A batida temperada: um conjunto de instruções é jogo em casa para um enum

Um desvio antes de o pareamento aterrissar, porque este padrão exato é por que o nicho web3 pertence ao Rust. Uma transação da Solana carrega instruções, e uma instrução é uma de um conjunto fechado de operações, cada uma com o payload dela: transfira tantos lamports para lá, delegue autoridade para aquela chave, feche esta conta. Conjunto fechado. Dados por variante. Dispatch exaustivo. Você vem encarando essa forma a lição inteira:

```rust
#[derive(Debug)]
pub enum StationInstruction {
    Transfer { lamports: u64, to: [u8; 32] },
    Delegate { authority: [u8; 32] },
    Close,
}

pub fn describe_instruction(ix: &StationInstruction) -> String {
    match ix {
        StationInstruction::Transfer { lamports, to } => {
            format!("move {lamports} lamports to the address ending {:02x}", to[31])
        }
        StationInstruction::Delegate { authority } => {
            format!("hand probe authority to the key starting {:02x}", authority[0])
        }
        StationInstruction::Close => "tear the account down".to_string(),
    }
}
```

Três variantes, três formatos de payload (campos nomeados, campos nomeados, nenhum campo), um match que precisa tratar toda operação ou falhar em compilar. (A função é `describe_instruction`, não `describe`, porque o motor já tem um `describe` para `ProbeResult`, e duas funções não podem dividir um nome num módulo.) Um projeto de uma struct por instrução mais uma string `kind` não te dá nada disso: o conjunto fechado vira aberto, o dispatch vira esperança stringly-typed. Isto é prática de modelagem, dito sem enfeite: nada faz deploy aqui, e os arrays `[u8; 32]` são arrays de bytes fazendo as vezes de endereços. O que os programas FAZEM com instruções é assunto do curso Master Anchor V2; por que transações carregam instruções afinal pertence ao curso do Bitcoin à Solana. A gente pega a forma emprestada porque ela é o melhor argumento do mundo real a favor de enums que carregam dados, e assim o enum grande de instruções de um programa Solana de verdade se lê como casa, não como algo impressionante.

### O pareamento: cargo test é o vitest vestindo uma bandeira diferente

Agora o segundo fio do módulo. Lá em m02-l4 você deu à frota TS uma suíte de testes e ligou ela no pipeline como gate #2. O motor Rust vem vivendo sem nada disso, e "como você muda um código do qual você tem medo?" tem uma resposta de duas partes neste curso: torne estados ilegais irrepresentáveis, depois trave todo o resto. Aqui está a história inteira de testes em Rust, e a versão honesta é que você já sabe ela:

| você paga por, em TS | você ganha de graça, em Rust | mesmo trabalho |
|---|---|---|
| vitest | `cargo test` | roda as afirmações, falha o build |
| prettier | `cargo fmt` | acaba com diffs de estilo para sempre |
| eslint | `cargo clippy` | sinaliza código que compila mas cheira mal |
| tsc --noEmit | `cargo check` | isto é sequer um programa |

Nenhuma linha de install nesta seção e esse é o ponto: as três ferramentas já vêm no toolchain stable que você instalou em m04-l1, junto com o rust-analyzer. Zero pacotes, zero arquivos de config, zero debates de runner. Os mesmos trabalhos que a frota TS paga quatro dev-dependencies para fazer, na caixa. Mesma ideia, encarregado mais rígido.

Os testes moram no mesmo arquivo que o código, dentro de um módulo que só existe para builds de teste:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ProbeState::*;

    #[test]
    fn one_failure_degrades_instead_of_killing() {
        assert_eq!(next_state(Up, false, 1), Degraded);
    }

    #[test]
    fn third_consecutive_failure_goes_down() {
        assert_eq!(next_state(Degraded, false, 3), Down);
    }
}
```

`#[cfg(test)]` compila o módulo só quando está testando, `use super::*` puxa os itens do arquivo que envolve ele, `#[test]` marca uma função como teste, `assert_eq!` é o seu `expect(x).toBe(y)`. Essa é a superfície de API inteira de que você precisa neste módulo. Testes dirigidos por tabela, fixtures, os padrões de m02-l4: todos eles se traduzem, e o lab escreve a suíte de transição completa. Uma diferença cultural que vale registrar pelo caminho: a sua frota TS guarda testes em arquivos `*.test.ts` irmãos, enquanto a convenção do Rust põe testes unitários no MESMO arquivo que o código que eles fixam, o que me pareceu errado por mais ou menos uma semana e depois virou a coisa de que eu sinto falta em todo lugar, porque o teste e o match que ele protege passam um pelo outro na mesma tela. O Rust também tem um segundo tier, testes de integração num diretório `tests/` de nível raiz que exercitam o seu crate por fora; o motor ganha esse tier quando o racha de workspace acontecer no próximo módulo, então deixe ele estacionado.

O clippy merece mais uma frase, porque "o compilador já checa tipos" é a objeção que todo time ouve quando o step de lint entra. O verificador de tipos prova que o seu código está bem formado. O clippy discute se ele é sábio: o cast `as` que perde informação, o Result ignorado, o clone desnecessário que você aprendeu a desconfiar em m04-l1. Ele é o gerador de comentário de review, a cadeira do eslint na tabela, e com `-D warnings` (deny: promove todo warning a erro duro) ele deixa de ser conselho e vira uma trava.

![Quatro ferramentas de TypeScript, cada uma pareada com o comando cargo que faz o mesmo trabalho, convergindo num único pipeline compartilhado.](assets/v04-comparison.webp)

Já que a gente está sendo honesto sobre o ecossistema: ensine o seu editor a rodar clippy, mas ensine ao seu Cargo.toml o ritmo real do ecossistema. O Rust atual é edition 2024, entregue no Rust 1.85.0 em 2025-02-20 (o `cargo new` vem escrevendo `edition = "2024"` nos seus manifests o módulo inteiro), e o rustc stable está em 1.98.0 desde 2026-08-20, checado em 2026-09-02, com um stable novo a cada seis semanas. Mas de cinco repos Rust adjacentes à Solana que este curso pesquisou, quatro ainda declaram edition 2021. Só o agave migrou. O trem de release sai no horário; o ecossistema embarca atrasado, e isso é normal, não negligência. Então quando um tutorial mostra `edition = "2021"`, ele não está errado, está mais velho, e quando você contribuir para um repo de verdade, acompanhe a edition DELE em vez de subir ela prestativamente num PR de passagem. (Uma nota de rodapé datada para ninguém se surpreender depois: os cursos de Solana on-chain da Academy escrevem 2021 por regra, porque o Rust avaliado deles constrói sobre um toolchain que ainda não aceita edition 2024. Tudo que é Rust NESTE curso compila na sua própria máquina, onde 2024 é simplesmente o atual; m05-l2 cuida desse racha por inteiro quando editions ganham o tratamento adequado delas.)

![Uma linha do tempo do release da edition 2024 até uma pesquisa de 2026 onde quatro de cinco repos Solana ainda declaram a edition mais velha.](assets/v05-timeline.webp)

Mais um pedaço de honestidade, o meu fato favorito deste módulo porque ele corta para os dois lados. Em 2025-11-19, enquanto o time de compilador do TypeScript portava o tsc para uma linguagem nativa por velocidade, a Prisma foi na direção oposta: o Prisma 7 apagou o motor de queries em Rust dele em favor de um compilador de queries em TypeScript, bundles uns 90% menores, um 3x em queries alegado pelo fornecedor. Leia as duas jogadas juntas e a guerra de linguagens se dissolve na única pergunta de verdade: a ferramenta certa para a camada. Um compilador quer o envelope de performance do Rust; uma camada de query dentro de um processo Node quer parar de pagar o imposto de fronteira. A sua estação roda as duas linguagens porque cada metade fica na camada em que ela é melhor, e esta lição travando as duas num pipeline só é a tese do curso em miniatura.

**Vá mais fundo (os 20%).** esta lição ensinou enums, match e um trait do jeito que a frota precisa deles no dia a dia. A profundidade está deixada como bookmark de propósito: o capítulo completo de enums, o zoológico de métodos do Option (`map`, `and_then`, `unwrap_or` e amigos), sintaxe de padrões além de tuplas e travas, moram no The Rust Book ch. 6 ([https://doc.rust-lang.org/book/ch06-00-enums.html](https://doc.rust-lang.org/book/ch06-00-enums.html)), e escrever os seus próprios genéricos, trait bounds e trait objects no ch. 10 ([https://doc.rust-lang.org/book/ch10-00-generics.html](https://doc.rust-lang.org/book/ch10-00-generics.html)), os dois verificados ao vivo em 2026-09-02. Quando este módulo acabar, o Rustlings é o pátio de treino: os conjuntos de exercícios de enums e de traits dele mapeiam um para um no material de hoje. O lab abaixo não depende de nada da profundidade deixada como bookmark.

### A parte honesta

Exaustividade é um contrato com um preço, e você vai sentir o preço antes de amar o contrato. Toda variante nova quebra todo match, em todo arquivo, até cada um decidir o que a variante significa. Magnífico para corretude, barulhento para velocidade, e esse barulho é por que catch-alls `_ =>` tentam: um braço curinga e os erros param. Mas leve o trade-off até o fim. Um catch-all compra silêncio de compilação hoje vendendo exatamente a garantia pela qual você modelou o enum; a próxima variante passa batido, classificada caladamente como o que quer que o curinga diga, e você está de volta ao registro forjado que m02-l1 te mostrou, na linguagem para a qual você veio atrás da garantia. A regra honesta: `_ =>` só em fronteiras de verdadeiro não-me-importa, e trate um numa máquina de estados como cheiro de código. Mesmo trade-off na trava de CI: `-D warnings` mantém o motor honesto e de vez em quando faz um merge inocente de refém por causa de um lint pedante. Esse atrito não é um defeito. Esse atrito É o code review.

## Lab: a máquina, o trait e o gate #3

Checagem de autonomia antes de começar, porque esta é a última resistência do loop de completion: os passos 1 e 2 são reparos de código dado, o passo 5 é trabalhado com você dirigindo o push, e o challenge é inteiramente seu. O M5 volta para a casca padrão de visão geral-lab-challenge; você se forma das rodinhas hoje.

### 1. Termine o match

Em `src/engine.rs` vai a máquina, exatamente como entregue aqui, ou seja: quebrada. O enum e os derives da seção de teoria, o `is_alerting`, e este next_state, dois braços a menos:

```rust
pub fn next_state(state: ProbeState, probe_ok: bool, consecutive_failures: u32) -> ProbeState {
    use ProbeState::*;
    match (state, probe_ok) {
        (Pending, true) => Up,
        (Pending, false) => Down,
        (Up, true) => Up,
        (Degraded, true) => Up,
        (Degraded, false) if consecutive_failures >= 3 => Down,
        (Degraded, false) => Degraded,
        (Down, false) => Down,
    }
}
```

Rode `cargo check` e use o erro como a planilha: o E0004 nomeia os dois padrões que faltam. Decida cada um pelas regras da frota (uma falha degrada, ela não mata; a recuperação é imediata), escreva os dois braços, e chegue no silêncio. Depois fixe a máquina com a suíte de transição. Uma regra de posicionamento antes de você colar: o `engine.rs` já termina num bloco `#[cfg(test)] mod tests`, o que m04-l2 construiu, e o Rust permite exatamente um módulo por nome, então colar este bloco textualmente embaixo dele é E0428, "the name `tests` is defined multiple times." Faça o merge em vez disso: adicione as cinco funções `#[test]` dentro do módulo existente, e adicione a linha `use ProbeState::*;` dele do lado do `use super::*;` que já está lá.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ProbeState::*;

    #[test]
    fn pending_first_success_goes_up() {
        assert_eq!(next_state(Pending, true, 0), Up);
    }

    #[test]
    fn one_failure_degrades_instead_of_killing() {
        assert_eq!(next_state(Up, false, 1), Degraded);
    }

    #[test]
    fn degraded_holds_below_three_failures() {
        assert_eq!(next_state(Degraded, false, 2), Degraded);
    }

    #[test]
    fn third_consecutive_failure_goes_down() {
        assert_eq!(next_state(Degraded, false, 3), Down);
    }

    #[test]
    fn down_recovers_straight_to_up() {
        assert_eq!(next_state(Down, true, 0), Up);
    }
}
```

```bash
cargo test
```

Checkpoint: os cinco testes de transição passam, junto com tudo que m04-l2 já tinha naquele módulo, então espere um total na faixa de dez-ou-mais dependendo de quantos testes de caminho de parse você adicionou na lição passada, com os cinco nomes acima verdes na lista. Se um teste de transição falhar, você preencheu um braço com o estado alvo errado; o nome do teste te diz qual regra reler.

### 2. Adicione a variante, siga os erros

O treino que responde a pergunta condutora. A estação vai precisar de um estado Maintenance algum dia: sondas suspensas de propósito, sem alertas. Adicione ele agora e veja o que o compilador faz com o seu medo. No enum:

```rust
    Down,
    Maintenance,
```

`cargo check`. E0004, no next_state, nomeando `(ProbeState::Maintenance, _)`. Todo match sem um catch-all agora exige uma decisão, e essa lista de erros é um inventário completo, escrito pelo compilador, de todo lugar no motor que precisa aprender o que Maintenance significa. Os seus instintos de TS dizem que você acabou de quebrar o projeto. Reenquadre: você fez uma pergunta ao projeto e recebeu de volta todo ponto relevante, por nome, com números de linha. Esta é a batida do refatorar-sem-medo, e é a resposta concreta para como você muda um código do qual você tem medo: você faz o compilador enumerar o raio da explosão, depois percorre a lista. Dê ao Maintenance os braços dele (sondas suspensas ignoram desfechos, então tanto `(Maintenance, true)` quanto `(Maintenance, false)` ficam parados em Maintenance).

![Um erro de compilação nomeando a nova variante Maintenance no match exato que precisa tratar ela, enquanto um ponto de curinga escondido fica em silêncio.](assets/v06-annotated-code.webp)

Agora a cilada que foi prometida a você. O `cargo check` está quieto, mas se pergunte: o `is_alerting` aprendeu sobre Maintenance? Não aprendeu, e nunca reclamou, porque o `matches!` esconde um braço `_ => false`. O curinga decidiu caladamente que Maintenance não está alertando, o que por acaso é o que a gente quer, por sorte, não por decisão. Isso é um catch-all fazendo exatamente o que a seção de trade-off avisou: absorver variantes novas sem te contar. Numa máquina de verdade esse silêncio tem dentes. Uma vez eu adicionei um estado atrás de um curinga num pipeline de status e levou dois dias de painéis errados para achar onde a decisão tinha sido tomada por mim, por um braço `_` escrito meses antes. Mantenha o `matches!` aqui se você aceitar o comportamento de false-por-padrão conscientemente, e agora você sentiu os dois lados do trade-off do curinga num treino só.

Termine o treino revertendo: o cânone da frota é quatro estados, e os testes do challenge são escritos contra exatamente esses quatro. `git restore src/engine.rs` a partir da raiz do `pulse-rs` se você commitou antes do treino (você commitou antes do treino, né?), ou apague a variante e os braços dela na mão e deixe um `cargo check` limpo confirmar a cirurgia. E já que esse commit é o primeiro do `pulse-rs`, uma jogada de higiene pertence à frente dele: o cargo pula escrever um `.gitignore` quando faz o scaffold dentro de um repo existente, e o arquivo de ignore da própria estação é só de node, então acrescente `target/` ao `.gitignore` do repo da estação antes de dar `git add`, ou você vai colocar a árvore de build inteira no stage, centenas de megabytes de saída de compilador que ninguém revisa.

### 3. Aterrisse o trait

Trabalhado, com menos apoio do que a seção de teoria te deu. Adicione `ProbeSource`, `FixtureSource` e `drive` da seção de teoria ao `src/engine.rs`, textualmente, depois ensine a máquina ao `main.rs`. O relatório de m04-l2 FICA: toda função do motor que ele exercita ficaria morta no momento em que você deletasse ele, e código morto é exatamente o que a trava de `-D warnings` do passo 5 se recusa a fazer ship. As linhas da máquina vão embaixo do relatório, ainda dentro do `main`, logo acima do `Ok(())` final. Duas colagens separadas em dois lugares separados, então mantenha elas apartadas. Primeiro, estenda os imports bem no topo do `main.rs`, do lado das linhas `use` de m04-l2:

```rust
use engine::{drive, parse_state, FixtureSource};
```

(O `StationInstruction` de propósito ainda não está importado; ele não existe até o passo 4, e importar ele agora é um erro de import não resolvido.) Segundo, acrescente dentro do `main`, embaixo do println de financiamento da estação:

```rust
    let boundary_input = "Pending";
    let Some(start) = parse_state(boundary_input) else {
        eprintln!("unknown state in fixture: {boundary_input}");
        return Ok(());
    };
    println!("starting from {start:?}");

    let mut source = FixtureSource::new(vec![212, 487, 1600, 1700, 1800, 90]);
    let state = drive(&mut source, 1500);
    println!("station state: {state:?} (alerting: {})", state.is_alerting());
```

Um pequeno sinal de que você está dentro do `main` da lição passada e não de um novo: a saída antecipada é `return Ok(());`, porque o `main` ainda devolve `anyhow::Result<()>` e um `return;` pelado se recusaria a compilar.

Uma função referenciada ali ainda não existe: `parse_state`. A regra do M2 é parse, don't validate, e ela veste a bandeira Rust dela aqui. Strings do mundo lá fora são parseadas para dentro do enum UMA VEZ na fronteira, e tudo terra adentro fala ProbeState:

```rust
pub fn parse_state(raw: &str) -> Option<ProbeState> {
    match raw {
        "Pending" => Some(ProbeState::Pending),
        "Up" => Some(ProbeState::Up),
        "Degraded" => Some(ProbeState::Degraded),
        "Down" => Some(ProbeState::Down),
        _ => None,
    }
}
```

E aí está o seu `_ =>` legítimo: uma fronteira de verdadeiro não-me-importa, onde toda string desconhecida significa exatamente uma coisa, não-é-um-estado. Esta é a casa honesta do curinga. Se estados stringly voltarem a se infiltrar terra adentro, se você pegar um estado `&str` fundo no motor, arraste o parse de volta para a borda.

Uma nota de honestidade sobre a fiação que você acabou de fazer, antes que um leitor atento pergunte: o `start` é parseado, impresso, e depois nunca alimentado ao `drive`, porque o `drive` fixa no código o próprio começo `ProbeState::Pending`. A rep de parse é real (uma string cruzou a fronteira e virou um estado tipado ou uma recusa em alto e bom som), mas o encanamento de propósito não está fechado: a assinatura do `drive` é parte do contrato congelado do m05 e, por isso, hoje o estado parseado para no println. Se a variável solta te incomoda, bom instinto; uma variante `drive_from(start...)` é um exercício de cinco linhas, só não renomeie o `drive` congelado para abrir espaço para ela.

Checkpoint: o `cargo run` imprime primeiro o relatório de fixture de m04-l2 inalterado, depois `starting from Pending`, depois `station state: Up (alerting: false)`. Antes de acreditar na impressão, percorra as seis fixtures na mão contra as regras de transição: duas sondas limpas seguram ele em Up, depois 1600 degrada ele, 1700 é a segunda falha consecutiva então Degraded segura, 1800 é a terceira então a trava dispara e o alvo vai para Down, e a sonda final de 90ms recupera ele direto para Up. Se a sua impressão disser Down em vez disso, o braço de recuperação é o suspeito: um braço `(Down, _) => Down`, ou qualquer curinga que engula `(Down, true)`, torna Down terminal, e aí o sucesso final de 90ms não consegue resgatar o alvo. Confira se `(Down, true) => Up` existe e é alcançável. (Esquecer de zerar `consecutive_failures` num sucesso também é um bug de verdade, mas esta fixture não consegue expor ele: o sucesso final recupera o alvo independentemente do que o contador diga, o que é em si uma lição sobre o que uma fixture consegue e não consegue provar.)

### 4. O desvio das instruções, entregue

Adicione `StationInstruction` e `describe_instruction` da seção de teoria ao motor, estenda a linha de import do passo 3 para `use engine::{drive, parse_state, FixtureSource, StationInstruction};`, depois despache uma fila no main, depois do relatório do drive:

```rust
    let queue = vec![
        StationInstruction::Transfer {
            lamports: 2_000_000,
            to: [7u8; 32],
        },
        StationInstruction::Delegate {
            authority: [9u8; 32],
        },
        StationInstruction::Close,
    ];
    for ix in &queue {
        println!("instruction: {}", engine::describe_instruction(ix));
    }
```

Esse número de lamports é o substituto de rent da lição passada dando mais um plantão como valor trabalhado, nada mais. Pequena nota de honestidade: se você adicionar o enum sem usar toda variante e todo campo, o `-D warnings` vai falhar o build em lints de código morto no passo 5. Isso é rigor nível clippy vindo do próprio rustc, e é por isso que a fila acima constrói as três variantes. Código morto num binário é um warning; atrás da flag de deny, warnings são a lei.

### 5. Gate #3: o pipeline aprende Rust

O re-ship. Mesmo repo, mesmo `pulse.yml` que você vem criando desde m01-l3; garanta que o `pulse-rs/` mora dentro do repo da estação (se você fez o scaffold dele em outro lugar em m04-l1, mova o diretório para dentro e commite antes de fazer a fiação, com `target/` no `.gitignore` da estação conforme o passo 2). Depois o diff do workflow, um job novo e uma linha mudada:

```yaml
  rust: # NEW: the engine's triple gate
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: pulse-rs
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check

  probe:
    needs: [typecheck, test, rust] # CHANGED: was [typecheck, test]
```

O step de toolchain ganha o porquê dele. A imagem ubuntu-latest do GitHub por acaso já vem com o Rust 1.98.0 pré-instalado no momento em que isto é escrito (sondado em 2026-09-02), mas a imagem atualiza no cronograma do GitHub, não no seu, e ela não carrega o clippy. `dtolnay/rust-toolchain@stable` (a action de toolchain padrão da comunidade, verificada ao vivo em 2026-09-02) instala o stable atual mais exatamente os componentes que você nomear, então o toolchain da trava é uma decisão sua em vez de um acidente de imagem. O `working-directory` aponta todo step `run` para a metade Rust do repo. E o `needs` ganha um terceiro nome, que é o ponto inteiro do segundo fio da lição: o cron que publica o status.json agora espera pelas duas linguagens.

Olhe o terceiro comando dentro do job antes de commitar, porque a flag dele carrega o design inteiro. O `cargo fmt` local reescreve arquivos no lugar; o `cargo fmt --check` não reescreve nada e falha se qualquer coisa FOSSE mudar. O CI recebe a forma check, sempre, pelo mesmo motivo que o workflow de m02-l4 rodava `npx vitest run` em vez de watch mode: o trabalho de uma trava é recusar, não consertar. É o step mais barato do pipeline e compra mais paz social por segundo do que qualquer trava que você vá adicionar: com a máquina resolvendo estilo, nenhum pull request gasta nunca mais três comentários de review em posicionamento de chave, e todo diff mostra só lógica. Quando ele fica vermelho, o conserto é um comando, sem juízo de valor: `cargo fmt`, commite, pronto. A cadeira do prettier do seu pipeline TS, mesmo raciocínio, flag diferente. Não amoleça ele para só-avisar; uma trava de estilo que só avisa apodrece até virar nenhuma trava em um mês.

![Triggers de push e de agendamento alimentam typecheck, test e uma nova trava rust cujas arestas de needs todas protegem o probe job que publica o status.](assets/v07-flowchart.webp)

Vermelho primeiro, sempre. Plante o lint do mundo de m04-l1 no `drive`, um clone desnecessário num tipo Copy:

```rust
        let probe_ok = latency.clone() <= budget_ms;
```

Commite, dê push, observe a execução: o job rust falha no step de clippy dele (`-D warnings` promovendo o lint a erro duro) com `using clone on type u64 which implements the Copy trait`, o probe job aparece como skipped, o status.json não recebe commit. O motor escreveu um comentário de review e bloqueou o próprio merge. Repare no que NÃO pegou isso: o `cargo test` passou, porque o clone é código correto, só que não é sábio. Essa é a cadeira do clippy fazendo um trabalho que a cadeira do teste não consegue. Reverta o clone, rode o trio local antes de dar push, porque o terceiro footgun desta lição é ligar `-D warnings` no CI sem nunca rodar clippy localmente, e aí descobrir lints um push de cada vez como um linter operado a moedas:

```bash
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

Dê push, e observe a sequência verde: três travas, depois a sonda.

![Uma execução de pipeline falha parada pelo step de lint acima de uma segunda execução totalmente verde que chega até a publicação do status.](assets/v08-diagram.webp)

Checkpoint, e ele é o do módulo: a execução do Actions do commit que você deu push mostra o job rust verde AO LADO do job vitest. Tire um screenshot. Duas linguagens, um pipeline, três travas, e nada faz o ship para o status.json que os dois compiladores e as duas suítes não tenham assinado.

## Challenge

A rep sem guia: o challenge probe-state-machine, no painel de coding-challenge desta lição. O starter compila e está errado das duas maneiras que você agora sabe consertar: Degraded está inteiramente ausente da máquina, e strings de estado desconhecidas vazam como Pending em vez de serem rejeitadas. Para ser preciso sobre o formato, porque `"Invalid"` NÃO é um quinto estado: o `next_state` que o grader enxerga recebe o estado atual como string e devolve o NOME do próximo estado como string. Então modele os quatro estados canônicos como um enum de verdade, faça o parse da string que chega uma vez na fronteira (o braço `_ =>` do parse é a casa honesta dele, exatamente como o `parse_state` no lab), devolva a string `"Invalid"` quando esse parse falhar, e despache todo estado parseado com sucesso por um match exaustivo sobre estado e desfecho com a trava de falhas consecutivas em três. Nenhum `_ =>` dentro desse match interno, e o enum fica com quatro variantes, e aqui honestidade importa sobre quem cobra isso: ninguém além de você. O grader roda pares de entrada e saída e nunca lê o seu código-fonte, então um atalho de curinga passa em todo teste enquanto desvia da lição inteira; dê um grep por `_ =>` na sua própria solução antes de dar ela por pronta e garanta que o único acerto é o parse de fronteira. Seis testes, incluindo o caminho de recuperação e a fronteira entre a segunda e a terceira falha. Tudo o que você precisa está acima; os hints no starter escalam da forma do enum até a sintaxe da trava, gaste eles em ordem.

## Onde isso deixa o motor

Diga a recuperação em voz alta antes de fechar o terminal, ganho de trinta segundos: vitest está para cargo test assim como prettier está para O QUÊ assim como eslint está para O QUÊ. Se as duas respostas vieram na hora, o pareamento aterrissou; é a dobradiça em que o quiz do módulo gira. Localmente, a sua trava tripla roda os mesmos três comandos que o workflow agora cobra, o que significa que "funciona na minha máquina" e "vai passar no CI" viraram a mesma frase, e essa identidade vale mais do que qualquer teste individual.

Um pedido enquanto o template do módulo está fresco: esta foi a última lição na pegada mais fina, reparos e reps trabalhadas até o fim, e o próximo módulo te dá arquivos em branco de novo. Se o treino de adicionar-uma-variante foi o momento em que a exaustividade fez sentido, ou se ele ainda parece cerimônia, diga isso no feedback; esse treino é a aposta da lição, e eu quero saber se ela se pagou.

O motor Rust agora bate com a spec da frota TS: tipado, honesto com erros, com máquina de estados, e travado no CI. Mas ele ainda lê latências de um vetor fixo no código, e a config dele mora no código-fonte. Próximo módulo: o serde parseia o MESMO arquivo de config que o seu schema zod parseia, um arquivo alimentando duas linguagens, o crate se divide num workspace de verdade, e a CLI ganha um braço de sonda HTTP de verdade, uma chamada reqwest isolada primeiro; plugar HTTP ao vivo atrás do próprio trait que você congelou hoje aterrissa um módulo depois, na lição do poller de longa duração, pelo lado da CLI, onde uma fonte bloqueante continua legal. Um arquivo de config está prestes a servir a dois senhores.
