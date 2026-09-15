# Erros são valores: Result, `?` e o cânone de dois crates

## Resumo

A lição passada deixou o `pulse-rs` compilando: tipos limpos de ownership e um classificador puro rodando sobre latências de fixture. Mas qualquer entrada malformada ainda mata a execução inteira com um panic, e esse é o alvo de hoje. Não capturar o panic. APAGAR ele. No fim, todo desfecho de sonda flui pelo motor como um `Result` que o chamador é obrigado a olhar, as linhas malformadas voltam como valores `Err` impressos enquanto o relatório termina em volta delas, e a sua matemática de dinheiro se recusa a dar wrap. No caminho você vai escrever a sua primeira closure, dividir o projeto numa metade biblioteca e numa metade binário, e adotar exatamente o cânone de erros de dois crates que os maiores repos de Solana rodam em produção. A pegada do M4 continua: o lab e as reps de conserto são trabalhados comigo ou como conserto-de-código-dado, e as únicas linhas que você escreve sem guia dentro deles são a closure do `map_err` e as substituições por matemática checada. O challenge no fim então quebra a pegada do nunca-um-arquivo-em-branco uma vez, de propósito: uma reconstrução-e-conversão do zero, porque a conversão só prova que viaja quando a cena do crime é sua.

## Mate o relatório primeiro

Abra o `pulse-rs` da lição passada. Fixtures de verdade não chegam como literais arrumados `vec![LatencyMs(212)]`; elas chegam como texto, e texto mente. Simule isso em dois minutos. Adicione este helper acima de `main`:

```rust
fn parse_latency(line: &str) -> u64 {
    line.trim().parse().unwrap()
}
```

E substitua o loop de fixture em `main` por uma versão de texto cru:

```rust
    let raw_fixture = ["212", "487", "fast", "1204", "930"];

    println!("target: {} ({})", target.name, target.url);
    for line in raw_fixture {
        let latency = LatencyMs(parse_latency(line));
        println!("{line}ms -> {:?}", classify_latency(latency));
    }
    println!("probes classified: {}", raw_fixture.len());
```

`cargo run`:

```text
target: solana-rpc (https://api.mainnet.solana.com)
212ms -> Up
487ms -> Degraded

thread 'main' panicked at src/main.rs:58:25:
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
```

(A compilação também te recebe com três avisos de dead-code, `describe`, `classify_probe` e `ProbeResult` acabaram de perder os únicos chamadores deles quando você trocou o loop. Esperado, inofensivo e temporário: o passo 5 do lab põe os três de volta na folha de pagamento.)

Duas sondas classificadas, depois a morte. `"fast"` não é um número, o `parse` disse isso, e o `unwrap()` traduziu "disse isso" para "mate o processo". O 1204 e o 930 nunca foram olhados. Lembra do módulo 1 e da frota v0 sem tipos dele, aquela que mentia educadamente e registrava timeouts como strings de prosa? Este é o primo dela em Rust, só que mais alto: em vez de saída errada você não tem saída nenhuma. Nenhum dos dois é o que um operador encarando um relatório de frota precisa às 3 da manhã. Deixe o panic onde está. Estamos prestes a desmontar ele, e depois vamos apagar ele tão a fundo que um grep por `unwrap` no seu código de produção não retorna nada.

## Erros como valores de retorno, derivados

### O que aquele panic realmente era

Um panic é o Rust declarando o estado do programa não confiável e derrubando a thread: desenrola, imprime, morre. É a ferramenta certa para "isso nunca pode acontecer, e se aconteceu, a memória está suspeita". É catastroficamente a ferramenta errada para "um arquivo de texto tinha um typo", que não é uma emergência, é terça-feira. E `unwrap()` é a ponte de uma palavra entre as duas: quer dizer "se esta operação falhou, dá panic". Todo `unwrap` no seu código é uma pequena confissão assinada de que você escolheu não pensar no caso de falha.

Então o que o `parse` deveria fazer no lugar disso quando a entrada é lixo, já que Rust não tem exceções para lançar? Aqui está o design inteiro, e com honestidade ele mal precisa ser derivado: se uma função pode falhar, diga isso no tipo de retorno. É só isso. Uma exceção, vista a frio, é um segundo canal de retorno que nunca aparece na assinatura: o `JSON.parse` no seu TypeScript alega que retorna `any`, e o throw é uma porta lateral que você descobre em produção. Você lidou bem com isso no M2, com `try/catch` na fronteira e uma união discriminada carregando o desfecho para dentro, e esse instinto estava exatamente certo. Rust simplesmente remove a porta lateral por inteiro. A falha É o valor de retorno, de primeira classe, tipada, visível em toda assinatura por onde ela passa.

![Uma função falha por um canal lateral invisível enquanto outra retorna sucesso ou falha como um único valor visível sobre o qual o chamador precisa ramificar.](assets/v01-comparison.webp)

### Result e Option são enums que você já domina

Aqui está a parte que deve parecer reprise, porque é. `Result` é definido na biblioteca padrão mais ou menos assim:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Um enum com duas variantes, cada uma carregando dados. Você CONSTRUIU um desses ontem: `ProbeResult` com as variantes `Ok`, `Timeout` e `HttpError` dele era você fazendo erros-como-valores na mão, no seu próprio domínio, antes de a linguagem te contar que tinha uma versão de propósito geral. `Option<T>` é a mesma ideia para ausência: `Some(T)` ou `None`, a resposta da biblioteca padrão ao `null`, só que o compilador te obriga a olhar antes de tocar. E como são enums comuns, a ferramenta em que você já confia se aplica: dê `match` neles, exaustivamente, com o compilador se recusando a te deixar esquecer um braço.

```rust
match parse_latency("212") {
    Ok(ms) => println!("got {ms}"),
    Err(e) => println!("bad line: {e}"),
}
```

Os parâmetros genéricos `<T, E>` são os primeiros genéricos que você encontra em Rust, e hoje você só precisa da versão em nível de leitura: `Result<u64, ParseIntError>` quer dizer "Ok carrega um u64, Err carrega um ParseIntError". Esse é todo o entendimento exigido neste módulo.

Uma nota de campo sobre o quanto essa forma é universal: enquanto construía a ferramentaria deste curso eu bati na API do crates.io com um `curl` pelado e fui recusado, porque o crates.io rejeita chamadas sem um header User-Agent. A recusa chegou como dado, um corpo não-JSON que o meu script teve que parsear e reportar como um `Err`, não como uma exceção que alguém lá em cima esqueceu de capturar. Lá fora, no mundo real, falha é só mais um valor no fio. O sistema de tipos do Rust está concordando com a realidade, não inventando cerimônia.

### ? é retorno antecipado com bom gosto

Dar match em toda chamada falível cansa rápido. Veja o que acontece com uma função que faz três coisas falíveis com matches explícitos: ela vira uma escadaria de blocos `match` onde a lógica de verdade se esconde nos cantos. A resposta do Rust é um caractere. Escrever `?` depois de um `Result` quer dizer: se isto é `Ok(v)`, faça unwrap para `v` aqui mesmo e siga; se isto é `Err(e)`, retorne `Err(e)` da função envolvente imediatamente. Retorno antecipado, na pista de erro, com o caminho feliz deixado plano e legível.

![Uma chamada falível ou continua para baixo com o valor desembrulhado dela ou sai cedo por uma conversão From carregando o erro para fora da função.](assets/v02-flowchart.webp)

Tem mais uma coisa escondida naquela pista vermelha, e é o detalhe que faz o `?` compor entre bibliotecas: na saída, o erro passa pelo trait `From`. Se a sua função retorna `Result<T, ProbeError>` e a chamada interna falhou com um `ParseIntError`, o `?` vai converter o erro estrangeiro no seu, desde que exista uma conversão. Quando não existe, o compilador te para, e você vai bater exatamente nesse muro no lab, de propósito, porque a correção é a sua primeira closure. Nomeie o mecanismo agora, para que a mensagem de erro se leia como informação depois: ? propaga, e converte via From na saída.

Então a regra da casa, e repare que é bom gosto, não lei: nada de `unwrap()` no caminho de produção. Num teste, `unwrap` é aceitável, com honestidade é o certo: um panic num teste É o relatório de falha, entregue a você, na sua mesa. Num spike descartável, também tudo bem. A diferença é quem paga quando ele dispara. Você paga na sua mesa, ou um operador paga às 3 da manhã encarando um relatório de frota impresso pela metade. Escreva o unwrap onde é você quem está segurando a conta.

### O cânone de dois crates: thiserror no motor, anyhow no binário

Agora a camada de ecossistema, porque tratamento de erros em Rust de verdade são dois crates, e ONDE cada um vai importa mais do que qualquer um dos dois importa. A divisão decorre de uma pergunta: quem consome o erro?

Os erros de uma biblioteca são consumidos por código. O chamador do seu motor quer dar `match` no que deu errado, porque uma linha malformada e um valor fora da faixa merecem tratamentos diferentes. Código precisa de variantes, então uma biblioteca deve aos chamadores dela um TIPO de erro de verdade: um enum. Escrever o boilerplate desses enums (o impl de `Display`, a fiação do trait) é o que o `thiserror` apaga: você deriva, anota cada variante com uma string de mensagem `#[error("...")]`, e o crate escreve o resto. Trabalho poupado puro, e a coisa mais próxima de uma convenção universal que Rust tem.

Os erros de um binário são consumidos por um humano lendo um terminal. `main` não dá match em variantes; ele reporta, com o máximo de contexto possível, e sai com código diferente de zero. Isso é o `anyhow`: um único `anyhow::Result` flexível no qual qualquer erro se converte, mais o `.context("...")` para empilhar migalhas legíveis por humanos na subida.

O trade-off que prende cada um na sua pista: a flexibilidade do anyhow vem de APAGAR a informação de tipo que o thiserror preserva. Use anyhow numa biblioteca e ela compila bem, parece conveniente, e sem alarde rouba dos seus chamadores a capacidade de dar match no que deu errado. Essa inversão é o erro mais comum do mundo real com esses crates, que é exatamente por isso que o cânone é dois crates e não um.

E é o cânone, não a minha preferência. Eu sondei os arquivos Cargo.toml dos repos estruturais de Solana em 2026-09-01: agave (o cliente validador) e yellowstone-grpc estão no thiserror 2.x, enquanto photon e jito-relayer ainda rodam a linha 1.x, e o anyhow pega carona em todos eles. Dois majors coexistindo em um ecossistema já é uma pequena lição por si: antes de atualizar um crate fundacional, leia o que o seu ecossistema realmente fixa. Os repos de que você depende andam mais devagar que a tag latest do crates.io, e se igualar aos vizinhos vence correr atrás da ponta.

![Um enum de erro tipado no motor flui pelo operador de interrogação até um result do anyhow no binário, com os repos pesquisados listados embaixo.](assets/v03-diagram.webp)

### Matemática de dinheiro: o compilador não vai te salvar do +

Mais um modo de falha pertence aqui porque ele NÃO se anuncia. As suas latências e, mais adiante neste curso, os seus lamports são valores `u64`, e soma de `u64` pode dar overflow (estouro). Aqui está a parte feia: builds de debug dão panic no overflow, builds de release dão wrap em silêncio por padrão. A mesma linha de código, dois comportamentos. O seu colega de equipe diz "entregue o build de release, lá ele não dá panic", e o seu colega está tecnicamente certo do pior jeito possível, porque um u64 que deu wrap não é um crash, é um número errado de cara séria.

Trabalhe isso com valores de verdade. Digamos que um contador de saldo esteja perto do topo da faixa, em `u64::MAX - 1_000_000`, que é 18,446,744,073,708,551,615. Some um depósito de 2,000,000 lamports num build de release e a soma dá wrap passando do zero para 999,999. Dezoito quintilhões e meio de lamports viram mais ou menos um milésimo de um SOL, sem panic, sem linha de log, e todo cálculo rio abaixo consome o cadáver alegremente. Eu já entreguei o primo deste bug: um ticker de saldo que bateu 18,446,744,073,709,551,615 na tela porque um reembolso caiu duas vezes e a minha subtração deu wrap por baixo do zero. Ninguém percebeu por um dia, porque nada deu crash. "Não deu crash" não é "estava certo".

![Uma soma perto do topo da faixa de u64 sai correndo pela ponta de uma reta numérica e reaparece perto do zero como um total minúsculo que deu wrap.](assets/v04-diagram.webp)

A correção é a família de métodos que a biblioteca padrão criou exatamente para isso: `checked_add` e `checked_sub` retornam `Option<u64>`, `Some(sum)` no caso normal e `None` no overflow. E olhe o que isso retorna: um Option, que realimenta direto a maquinaria que você acabou de aprender. Overflow deixa de ser uma corrupção de estado silenciosa e vira mais um valor de erro subindo pelo mesmo pipeline de `?` que uma linha de fixture ruim. Uma história de tratamento de erros só para o programa inteiro.

**Vá mais fundo (os 20%).** esta lição ensinou o padrão de uso diário: Result e Option, `?`, a regra da casa de não usar unwrap, a divisão de dois crates, matemática checada. O tratamento completo de erros recuperáveis versus irrecuperáveis, quando um panic é genuinamente correto, e a superfície de API mais profunda do `Result` é o capítulo 9 do Rust Book: [https://doc.rust-lang.org/book/ch09-00-error-handling.html](https://doc.rust-lang.org/book/ch09-00-error-handling.html). E a closure que você está prestes a escrever tem um capítulo inteiro de profundidade atrás dela (modos de captura, os traits Fn, closures como argumentos e retornos) no capítulo 13: [https://doc.rust-lang.org/book/ch13-00-functional-features.html](https://doc.rust-lang.org/book/ch13-00-functional-features.html) (os dois links checados ao vivo em 2026-09-02). Salve os dois como bookmark, leia o ch09 esta semana. O lab não precisa de nenhum dos dois.

## Lab: o relatório que sobrevive às próprias fixtures

Estado alvo, para você saber quando terminou: `pulse-rs` dividido num módulo de motor (a metade biblioteca, erros tipados, sem impressão, sem saída) e num binário main (a metade anyhow, toda a impressão), lendo de ponta a ponta um arquivo de fixture deliberadamente sujo, com zero unwraps fora dos testes.

1. **Adicione os dois crates.** A partir da raiz do `pulse-rs`:

   ```bash
   cargo add thiserror anyhow
   ```

   `cargo add` é o npm-install do Cargo, e ele escreveu as duas dependências no `Cargo.toml` para você. Hoje (2026-09-02) isso puxa thiserror 2.0.20 e anyhow 1.0.104; os seus dígitos de patch podem estar mais novos, e para esses dois crates famosamente estáveis isso é aceitável.

2. **Divida o projeto em metades.** Crie `src/engine.rs` e mova para lá todo tipo e função de `main.rs` EXCETO o próprio `main`: `ProbeTarget`, `LatencyMs`, `ProbeResult`, `Verdict`, `classify_latency`, `classify_probe`, `describe` e o condenado `parse_latency`. Marque cada item movido como `pub` (public: visível fora do módulo; privacidade de módulo é o padrão do Rust e vamos passear por ela direito com o Cargo no M6), e seja preciso sobre o que "cada" quer dizer, porque privacidade é por item, não por arquivo: as duas structs precisam de `pub` nos CAMPOS delas também, já que `main.rs` as constrói e as lê. `ProbeTarget` vira `pub struct ProbeTarget { pub name: String, pub url: String }`, e o valor interno do newtype ganha isso também: `pub struct LatencyMs(pub u64);`. Os enums são mais fáceis: uma variante é exatamente tão pública quanto o enum dela, então `ProbeResult` e `Verdict` precisam só do único `pub` na frente de `enum`. Pule um campo e o `cargo check` te recebe com um erro de campo privado no ponto de construção em `main`. Então declare o módulo no topo do agora minúsculo `main.rs`:

   ```rust
   mod engine;
   ```

   Essa única linha diz ao Cargo que `src/engine.rs` existe e pertence a este programa. O seu binário agora tem uma metade biblioteca e uma metade binário, que é exatamente a costura que o cânone de dois crates quer.

3. **Derive o tipo de erro do motor.** No topo de `engine.rs`:

   ```rust
   use std::num::ParseIntError;
   use thiserror::Error;

   /// A latency past this is a corrupted fixture line, not a slow probe.
   pub const MAX_SANE_LATENCY_MS: u64 = 60_000;

   /// Worked-example constant for the checked-math path. The number is real;
   /// what an ATA actually is belongs to the Digital Assets course.
   pub const ATA_RENT_LAMPORTS: u64 = 2_039_280;

   #[derive(Debug, Error)]
   pub enum ProbeError {
       #[error("not a latency reading: {0}")]
       BadFixture(ParseIntError),
       #[error("{0}ms is past the {MAX_SANE_LATENCY_MS}ms sanity ceiling")]
       OutOfRange(u64),
       #[error("u64 arithmetic overflowed")]
       Overflow,
   }
   ```

   Leia isso pelo que é: um enum simples, como `Verdict`, exceto que cada variante é um jeito de o motor falhar, e cada uma carrega a evidência dela. `BadFixture` segura o `ParseIntError` de baixo para que nada se perca; `OutOfRange` segura o número ofensor. As strings `#[error("...")]` são a renderização legível por humanos, e o `{0}` interpola o campo de dados da variante. Esse derive mais essas strings substituem a dúzia de linhas de `impl Display` que você escreveria à mão por tipo de erro.

4. **Converta o parse e encontre o muro.** Apague a versão com `unwrap` de `parse_latency` e escreva o par honesto. Primeira tentativa, exatamente assim:

   ```rust
   pub fn parse_latency(line: &str) -> Result<u64, ParseIntError> {
       line.trim().parse()
   }

   pub fn parse_fixture_line(line: &str) -> Result<LatencyMs, ProbeError> {
       let ms = parse_latency(line)?;
       if ms > MAX_SANE_LATENCY_MS {
           return Err(ProbeError::OutOfRange(ms));
       }
       Ok(LatencyMs(ms))
   }
   ```

   `cargo check`:

   ```text
   error[E0277]: `?` couldn't convert the error to `ProbeError`
      |
      |     let ms = parse_latency(line)?;
      |                                 ^ the trait `From<ParseIntError>` is not
      |                                   implemented for `ProbeError`
      |
      = note: the question mark operation (`?`) implicitly performs a conversion
        on the error value using the `From` trait
   ```

   Lá está o mecanismo da seção de teoria, ao vivo: o `?` tentou converter `ParseIntError` em `ProbeError` via `From`, não achou conversão e parou. Existem duas correções. A que esta lição ensina é o map explícito na pista de erro, e escrever isso quer dizer escrever a sua primeira closure. Mude a linha para:

   ```rust
       let ms = parse_latency(line).map_err(|e| ProbeError::BadFixture(e))?;
   ```

   Agora pare e olhe para `|e| ProbeError::BadFixture(e)`, porque este é um grande pequeno momento: a sua primeira closure, chegando bem na hora, exatamente onde a linguagem a torna necessária. Uma closure é uma função anônima, escrita inline, que consegue capturar variáveis do escopo em volta dela. `|e|` declara o parâmetro dela, a expressão depois é o corpo dela, e a coisa toda é um VALOR que você entrega ao `map_err` como qualquer outro argumento. O `map_err` só a roda se o Result for um `Err`, transformando a pista de erro e deixando a pista do `Ok` intocada (o `map` é o gêmeo dele para a pista de valor; misturar os dois é um tropeço clássico de primeira semana, então diga em voz alta de qual pista você está falando). Esta aqui ainda não captura nada; closures que capturam, e as cadeias de iteradores onde as closures vivem os melhores dias delas, chegam no próximo módulo. A profundidade completa fica no bookmark do link do ch13 acima.

![Uma linha de código é dissecada com rótulos para a chamada falível, o map da pista de erro, o parâmetro da closure, o corpo que embrulha e o operador final de propagação.](assets/v05-annotated-code.webp)

   Rode `cargo clippy` antes de seguir e ele vai te cutucar sobre exatamente esta linha:

   ```text
   warning: redundant closure
       .map_err(|e| ProbeError::BadFixture(e))?;
                ^ help: replace the closure with the tuple variant itself:
                  `ProbeError::BadFixture`
   ```

   O clippy está certo, e vale a pena entender o motivo: um construtor de variante de tupla como `ProbeError::BadFixture` JÁ é uma função, então uma closure que só repassa para ele não acrescenta nada. `.map_err(ProbeError::BadFixture)` é idêntico e mais curto. A gente mantém a closure escrita por extenso neste módulo mesmo assim, para que a forma fique na sua frente enquanto é nova, e a gente diz isso ao clippy explicitamente. Ponha este atributo na função:

   ```rust
   #[allow(clippy::redundant_closure)]
   ```

   com um comentário dizendo por quê e quando ele morre (reduza a closure no M5, apague o allow). Um `#[allow]` com data de validade escrita é um empréstimo de ferramenta; um `#[allow]` sem data é como nasce dívida de lint. Para completar, a segunda correção para o muro E0277: o thiserror consegue derivar a conversão `From` sozinho com um atributo `#[from]` na variante, e aí o `?` pelado simplesmente funciona. A gente escolheu a closure explícita hoje porque você precisa VER a conversão uma vez antes de deixar um derive escondê-la.

5. **Ligue a metade anyhow.** Substitua `main.rs` abaixo da linha `mod engine;` pela metade binário inteira:

   ```rust
   use anyhow::{Context, Result};
   use engine::{LatencyMs, ProbeResult, ProbeTarget};

   fn main() -> Result<()> {
       let target = ProbeTarget {
           name: String::from("solana-rpc"),
           url: String::from("https://api.mainnet.solana.com"),
       };

       let raw = std::fs::read_to_string("fixture.txt")
           .context("could not read fixture.txt from the pulse-rs root")?;

       let mut clean: Vec<LatencyMs> = Vec::new();
       let mut rejected = 0u32;

       println!("target: {} ({})", target.name, target.url);
       for line in raw.lines() {
           match engine::parse_fixture_line(line) {
               Ok(latency) => {
                   println!("  {line}ms -> {:?}", engine::classify_latency(latency));
                   clean.push(latency);
               }
               Err(e) => {
                   println!("  {line:?} -> Err({e:?}): {e}");
                   rejected += 1;
               }
           }
       }

       // The structured path keeps a heartbeat on fixtures until the M6 daemon
       // feeds real network outcomes into the state machine.
       let structured = [
           ProbeResult::Ok {
               latency: LatencyMs(212),
           },
           ProbeResult::Timeout {
               budget: LatencyMs(3000),
           },
           ProbeResult::HttpError { status: 429 },
       ];
       for probe in &structured {
           println!(
               "  {} -> {:?}",
               engine::describe(probe),
               engine::classify_probe(probe)
           );
       }

       let total = engine::total_latency(&clean).context("summing the latency budget")?;
       let funding = engine::station_funding(1_000_000, engine::ATA_RENT_LAMPORTS, 50_000)
           .context("computing station funding")?;
       println!(
           "{} clean probes, {rejected} rejected, {total}ms total latency",
           clean.len()
       );
       println!("station funding needed: {funding} lamports");
       Ok(())
   }
   ```

   Percorra as costuras, porque cada uma é uma decisão. `main` retorna `anyhow::Result<()>`, que é o que deixa o `?` funcionar dentro dele: qualquer erro que escapa sai impresso com a cadeia de contexto dele e o processo sai com código diferente de zero, que é o trabalho inteiro de tratamento de erros de um binário. A leitura do arquivo veste um `.context("...")`, uma migalha para o humano. E o loop do relatório é a tese da lição em quatro linhas: dê `match` no Result de cada linha, imprima o veredito no `Ok`, imprima o erro no `Err`, e SIGA EM FRENTE. Um valor de erro não consegue matar um loop; só um panic consegue. As duas chamadas no bloco do resumo (`total_latency`, `station_funding`) ainda não existem; isso é o passo 6. Isto não vai compilar até que existam, e agora você consegue ler esse estado com calma em vez de com superstição.

![Uma linha de fixture se bifurca numa pista de erro e numa pista de valor que terminam as duas no mesmo relatório, que segue para a próxima linha de qualquer jeito.](assets/v06-flowchart.webp)

6. **Faça a matemática se recusar a mentir.** Em `engine.rs`, adicione o caminho de soma e o helper de funding, ambos construídos sobre aritmética checada alimentando o mesmo pipeline de erro:

   ```rust
   pub fn total_latency(latencies: &[LatencyMs]) -> Result<u64, ProbeError> {
       let mut total: u64 = 0;
       for latency in latencies {
           total = total.checked_add(latency.0).ok_or(ProbeError::Overflow)?;
       }
       Ok(total)
   }

   pub fn station_funding(base: u64, rent: u64, buffer: u64) -> Result<u64, ProbeError> {
       let subtotal = base.checked_add(rent).ok_or(ProbeError::Overflow)?;
       subtotal.checked_add(buffer).ok_or(ProbeError::Overflow)
   }
   ```

   `ok_or` é o método ponte: ele transforma `Option` em `Result` fornecendo o erro para o caso `None`, e dali o `?` assume como de costume. Então fixe o comportamento com testes no fim de `engine.rs`, incluindo um que documenta o que o modo release TERIA feito, para que o número do wrap da seção de teoria viva em forma executável:

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn overflow_is_an_error_not_a_wrap() {
           let nearly_full = LatencyMs(u64::MAX - 1_000_000);
           let rent_sized = LatencyMs(ATA_RENT_LAMPORTS);
           assert!(matches!(
               total_latency(&[nearly_full, rent_sized]),
               Err(ProbeError::Overflow)
           ));
       }

       #[test]
       fn what_release_mode_would_have_done() {
           let nearly_full: u64 = u64::MAX - 1_000_000;
           assert_eq!(nearly_full.wrapping_add(ATA_RENT_LAMPORTS), 1_039_279);
       }

       #[test]
       fn epoch_wall_clock_shrank_with_faster_slots() {
           let at_400ms = 432_000u64.checked_mul(400);
           let at_300ms = 432_000u64.checked_mul(300);
           assert_eq!(at_400ms, Some(172_800_000)); // 48 hours of milliseconds
           assert_eq!(at_300ms, Some(129_600_000)); // 36 hours of milliseconds
       }
   }
   ```

   O atributo `#[cfg(test)]` compila este módulo só para o `cargo test`, que é por isso que unwraps e panics são cidadãos legais lá dentro. Aquele último teste é um aquecimento de matemática checada com os próprios números da Solana: uma epoch é fixa em 432,000 slots, então quando a meta de tempo de slot da rede caiu de 400ms para 300ms neste agosto, as epochs encolheram de 48 horas para 36 em termos de relógio de parede. Mesma contagem de slots, batimento mais rápido, derivável em uma linha de matemática u64 honesta. Adicione mais dois ou três testes seus para o caminho do parse (uma linha de lixo é `Err(BadFixture(_))`, um dia em milissegundos é `Err(OutOfRange(_))`; `matches!` é o jeito de uma linha de afirmar a forma de um enum).

7. **Alimente ele com sujeira e verifique.** Crie `fixture.txt` na raiz do projeto, imundo de propósito:

   ```text
   212
   487
   fast
   1204
   86400000
   930
   ```

   Então a trava completa:

   ```bash
   cargo fmt
   cargo clippy
   cargo test
   cargo run
   ```

   fmt silencioso, clippy com zero avisos (o único lint que a gente ganhou está explicitamente permitido, com a nota de validade dele), testes verdes, e a execução imprime:

   ```text
   target: solana-rpc (https://api.mainnet.solana.com)
     212ms -> Up
     487ms -> Degraded
     "fast" -> Err(BadFixture(ParseIntError { kind: InvalidDigit })): not a latency reading: invalid digit found in string
     1204ms -> Down
     "86400000" -> Err(OutOfRange(86400000)): 86400000ms is past the 60000ms sanity ceiling
     930ms -> Degraded
     212ms -> Up
     no answer in 3000ms -> Down
     HTTP 429 -> Degraded
   4 clean probes, 2 rejected, 2833ms total latency
   station funding needed: 3089280 lamports
   ```

   Compare isso com a abertura. A mesma classe de lixo na entrada, e em vez de duas linhas e um cadáver você recebe o relatório inteiro: vereditos para as quatro sondas limpas, uma razão tipada e impressa para cada uma das duas rejeitadas (a forma de debug E a renderização do `#[error]` lado a lado), e totais honestos. O panic da abertura sumiu, e um grep prova o quanto sumiu:

   ```bash
   grep -n "unwrap" src/main.rs src/engine.rs
   ```

   O resultado esperado é silêncio, e para o código exatamente como dado, silêncio total: o caminho de produção tem zero unwraps, e o módulo de teste desta lição por acaso afirma com `matches!` e `assert_eq!` em vez de `unwrap`, então o grep não retorna nada mesmo. Se os testes extras que você escrever apelarem para `unwrap` (legal lá, e muitas vezes a escolha certa), a regra de auditoria é: toda ocorrência precisa ficar ABAIXO da linha `#[cfg(test)]` em `engine.rs`, e `main.rs` não pode mostrar nenhuma. Unwraps do lado do teste são panics funcionando como previsto, na sua mesa, como relatórios de falha.

### Reps de conserto: três correções que agora são suas

O loop de completion, mesma pegada da lição passada: código quebrado ou feio, um conserto com princípio para cada, numa bancada de rascunho (`cargo new error-reps && cd error-reps && cargo add thiserror`). Uma jogada de preparação antes da rep 1: copie o enum `ProbeError` do lab para o `main.rs` da bancada, junto com as linhas `use std::num::ParseIntError;` e `use thiserror::Error;` dele, porque a rep 1 retorna ele e o `HeaderError` da rep 2 é dado enquanto `ProbeError` é assumido. Preveja antes de conferir.

**Rep 1, escreva a closure sem guia.** Esta função não vai compilar. Corrija escrevendo você mesmo a closure do `map_err`, sem espiar o lab:

```rust
fn checked_line(line: &str) -> Result<u64, ProbeError> {
    let ms = line.trim().parse::<u64>()?;
    Ok(ms)
}
```

**Rep 2, três unwraps, três correções DIFERENTES.** Este helper funciona até o momento em que qualquer uma das três suposições dele quebra. Converta ele para retornar `Result<String, HeaderError>` (o enum abaixo é dado; as correções são suas). O porém: os três unwraps merecem três tratamentos diferentes, e saber qual é qual é a habilidade de verdade. Um deve propagar com `?` e um `map_err`, um deve virar um `Err` retornado via `ok_or`, e um deve ser engolido com `unwrap_or_default` mais um comentário defendendo a engolida:

```rust
#[derive(Debug, Error)]
enum HeaderError {
    #[error("the fixture is empty")]
    EmptyFixture,
    #[error("the first line is not a probe count: {0}")]
    BadCount(std::num::ParseIntError),
}

fn report_header(raw: &str) -> String {
    let first = raw.lines().next().unwrap();
    let count: u64 = first.trim().parse().unwrap();
    let label = std::env::var("STATION_NAME").unwrap();
    format!("{label}: expecting {count} probes")
}
```

Checagem de raciocínio, antes de digitar: a env var faltando é a engolida defensável (uma estação sem nome é irritante, um relatório morto é pior), a entrada vazia é um erro de verdade que o chamador precisa ouvir, e a contagem ruim sobe de carona no `?` como `BadCount`. Se você atribuiu de outro jeito, discuta comigo no feedback; existe espaço franco na env var.

![Três perguntas de sim ou não roteiam uma chamada falível para um de quatro tratamentos, de manter unwrap nos testes a propagar, usar default ou retornar um erro.](assets/v07-flowchart.webp)

**Rep 3, a substituição por matemática checada, sem guia.** De volta ao `pulse-rs`: esta versão de `station_funding` compila, passa num teste de caminho feliz e mente sob pressão. Substitua os dois operadores `+` por aritmética checada mapeando `None` em `ProbeError::Overflow`, depois fixe o conserto com um teste seu, porque o teste de overflow do passo 6 exercita `total_latency`, não esta função. Escreva `station_funding_overflow_is_an_error` na mesma forma (alimente `u64::MAX - 1_000_000` como `base` e `ATA_RENT_LAMPORTS` como `rent`, afirme `Err(ProbeError::Overflow)` com `matches!`) e faça ele passar:

```rust
pub fn station_funding(base: u64, rent: u64, buffer: u64) -> Result<u64, ProbeError> {
    Ok(base + rent + buffer)
}
```

Aceitação para as reps: as três compilam, `cargo test` verde, e para cada unwrap que você removeu você consegue dizer em uma frase quem teria pagado quando ele disparasse.

## Challenge

A rep sem guia é uma varredura, e você constrói a cena do crime você mesmo para que a conversão seja honesta. `cargo new no-unwrap-report && cd no-unwrap-report && cargo add thiserror anyhow`, depois reconstrua um pequeno binário de relatório de fixture no estado em que o seu `pulse-rs` estava hoje de manhã: o parse crivado de `unwrap` da abertura e o loop de fixture cru (ponha quatro ou cinco unwraps no caminho de produção já que está nisso, uma env var, uma leitura de arquivo, um `lines().next()`), um `+` pelado no total de lamports dele, e um `fixture.txt` sujo que mata ele na terceira linha. Depois converta ele de ponta a ponta: um enum de thiserror no módulo de motor dele, anyhow com context no `main` dele, todo unwrap substituído pelo tratamento que ele merece, matemática checada no total. O grader é a mesma trava que você acabou de rodar na mão: a execução precisa completar sobre a fixture suja imprimindo tanto linhas `Ok` quanto `Err`, `grep -rn 'unwrap()' src/` não pode imprimir nada fora dos módulos de teste (repare no `-r`; um `grep -c` pelado sobre um diretório se recusa a rodar), e o teste de overflow precisa passar. Atenção ao padrão exato: é `'unwrap()'` com os parênteses, não `unwrap` pelado, porque `unwrap_or_default` e os irmãos dele são tratamentos, não confissões, e a correção de env var da rep 2 tropeçaria num grep de `unwrap` pelado enquanto está exatamente certa. Tudo de que você precisa está acima; se você empacar, escale pela ordem do próprio lab, ache qual linha morre primeiro, depois a closure do `map_err`, depois a ponte do `ok_or`.

## Checkpoint

O que você já consegue fazer, concretamente: ler uma assinatura `Result` como um contrato em vez de cerimônia; propagar com `?` e explicar o pulo do `From` na pista vermelha; escrever uma closure no `map_err` e dizer o que a torna uma closure; dividir erros pelo cânone de dois crates e defender a divisão com o trade-off (anyhow apaga o que o thiserror preserva); e recusar aritmética que dá wrap em todo lugar onde moram números em formato de dinheiro. O seu `pulse-rs` sobrevive a um arquivo de fixture imundo e diz exatamente por que cada linha ruim foi rejeitada, num tipo em que um chamador conseguiria dar match.

A recuperação de 30 segundos antes de você fechar a aba, em voz alta: o que o `?` faz num `Err`? (Retorna antecipadamente o erro da função envolvente, convertendo via `From` na saída.) E qual perfil de build dá wrap no overflow? (Release. Debug dá panic. Nenhum dos dois substitui `checked_add`.)

Um pedido enquanto está fresco: na rep 2, a divisão em três correções diferentes pareceu ter princípio ou ser arbitrária, e qual unwrap você quase corrigiu errado? Me conte no feedback. Essa rep é a filosofia inteira do módulo em nove linhas, e se a engolida da env var pareceu trapaça eu quero ouvir o argumento, porque "quando um default é aceitável" é um debate que times de verdade têm toda semana.

Desfechos são valores agora. Mas a VIDA de uma sonda ao longo do tempo (pending, up, degraded, down) ainda é dado solto que o seu código meramente lembra de atualizar. Próxima lição: enums como máquinas de estado, o compilador segurando a caneta em toda transição legal, mais a trava de cargo test, clippy e fmt ligada ao seu workflow do Actions, que é a resposta honesta para "como você muda um código de que tem medo?". Até lá.
