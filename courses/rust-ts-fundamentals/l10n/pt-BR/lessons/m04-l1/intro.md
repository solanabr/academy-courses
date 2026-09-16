# Ownership é o ponto

## Resumo

O M3 fechou o tier de TypeScript: o `pulse-core` está publicado no npm, o painel está no ar no Vercel, e a trava do tier nomeou exatamente o que a gente pulou. A metade TS da estação está entregue e travada. Agora a segunda linguagem começa, e ela começa com uma briga. Em dez minutos você vai instalar o toolchain do Rust, fazer o scaffold do `pulse-rs`, colar cinco linhas inocentes, e levar uma recusa de um compilador por um código que o TypeScript rodaria sem piscar. A lição inteira é por que essa recusa é a feature que você veio buscar. Um aviso sobre como este módulo funciona: o M4 roda no grão mais fino do curso inteiro. Tudo aqui é ou trabalhado comigo ou conserta-um-snippet-dado. Você nunca autora a partir de um arquivo em branco; a única rep sem guia é o challenge do final.

## Primeiro, quebre alguma coisa

Instale o toolchain. O rustup é o instalador, a coisa inteira, um comando no macOS ou no Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

No Windows, baixe e rode o `rustup-init.exe` do mesmo site. Reinicie o seu shell, depois confirme:

```bash
rustc --version
cargo --version
```

Você deve ver rustc 1.98.0 ou mais novo (a 1.98.0 virou stable em 2026-08-20; verificado em 2026-09-02, e já que sai um stable novo a cada seis semanas, o seu dígito pode já estar mais alto, o que não é problema). Agora faça o scaffold da metade Rust da estação e rode ela. Faça isso a partir da raiz do repo da sua estação, aquele que guarda `packages/` e `pnpm-workspace.yaml`, porque o `pulse-rs` é a metade Rust da estação, não um projeto paralelo: ele mora dentro do repo, ao lado de `packages/`, e m04-l3 liga ele ao CI da própria estação exatamente nessa suposição.

```bash
cargo new pulse-rs
cd pulse-rs
cargo run
```

Hello, world. O Cargo é o npm, o tsc e o vitest em um binário só, e a gente vai passear por ele direito numa lição mais adiante. Hoje ele existe para compilar a sua primeira rejeição. Substitua tudo em `src/main.rs` por isto:

```rust
fn sum_latencies(latencies: Vec<u64>) -> u64 {
    latencies.iter().sum()
}

fn main() {
    let latencies = vec![212, 487, 1204];
    let total = sum_latencies(latencies);
    println!("total: {total} across {} probes", latencies.len());
}
```

Leia isso como o dev TypeScript que você agora é: faça um array, passe ele para um helper, imprima a soma e o tamanho. Em TS isso é terça-feira. Rode `cargo check` (compila sem produzir um binário, o seu loop rápido de feedback daqui em diante):

```text
error[E0382]: borrow of moved value: `latencies`
 --> src/main.rs:8:49
  |
6 |     let latencies = vec![212, 487, 1204];
  |         --------- move occurs because `latencies` has type `Vec<u64>`,
  |                   which does not implement the `Copy` trait
7 |     let total = sum_latencies(latencies);
  |                               --------- value moved here
8 |     println!("total: {total} across {} probes", latencies.len());
  |                                                 ^^^^^^^^^ value borrowed here after move
```

E0382. Use of moved value. O compilador recusou um programa que rodaria corretamente, hoje, na sua máquina, em qualquer linguagem com GC. Fique um segundo com o quanto isso parece absurdo, porque o resto desta lição é o argumento de que é a coisa mais razoável que um compilador já fez por você. Deixe o erro onde está. A gente conserta ele no lab, de propósito, com princípio.

## As regras, derivadas do problema

### Alguém tem que liberar isso

Comece pelo fato com que toda linguagem convive: quando o seu programa faz um `Vec` de latências, memória é alocada, e em algum momento essa memória tem que ser devolvida. Por alguém. Não existe uma quarta opção, só três respostas para "quem?"

Resposta um: um garbage collector. Node, Java, Go, Python. Um runtime observa os seus objetos, descobre para quais deles nada mais aponta, e libera eles. Você escreve código como se a memória fosse infinita, e paga pela ilusão em tempo de execução: o coletor consome CPU, e pausa o seu programa no cronograma dele, não no seu. Para o painel da frota esse custo é invisível. Para um cliente de validador ou um sistema de trading, uma pausa no milissegundo errado é dinheiro de verdade, que é uma razão honesta para tanto da stack da Solana ser Rust.

Resposta dois: você. C e C++. Você chama `free` quando terminou, e o compilador confia em você completamente. Os modos de falha dessa confiança têm nomes que você já ouviu mesmo se nunca escreveu C: use-after-free, double-free, dangling pointer. Décadas de boletins de segurança são o comprovante.

Resposta três, a resposta do Rust: fazer de "quem libera isso?" uma propriedade do próprio código, decidida em tempo de compilação. Todo valor tem exatamente um dono, a variável responsável por ele. Quando o dono sai de escopo, o valor é liberado, deterministicamente, sem coletor nenhum envolvido. Essa é a primeira regra de ownership (posse), e repare que você não decorou ela, você derivou ela: se a limpeza tem que acontecer exatamente uma vez sem runtime nenhum vigiando, exatamente um binding tem que responder por ela.

![Três colunas comparam garbage collection, liberação manual e ownership em tempo de compilação como respostas para quem libera a memória alocada.](assets/v01-comparison.webp)

### O move, e por que o seu snippet morreu

Siga a regra para dentro do snippet. `let latencies = vec![...]` faz de `latencies` o dono. Aí `sum_latencies(latencies)` passa o `Vec` por valor, e aqui o Rust faz uma coisa que o TS nunca te obrigou a pensar: o ownership se transfere. O parâmetro dentro do helper é o novo dono; ele vai liberar a memória quando a função terminar. Então o que é `latencies` dentro de `main` depois daquela linha? Se o Rust te deixasse continuar usando ele, dois bindings acreditariam, os dois, que são donos da mesma alocação. Dois donos significa ou limpeza dupla, liberar a mesma memória duas vezes, ou mutação ambígua, dois lugares com direito de mudar uma coisa só sem o outro saber. Então o acesso do binding antigo simplesmente acaba. Isso é um move, e o E0382 é o compilador te dizendo, com precisão: este valor tem um dono novo, o seu nome não está mais nele.

Repare no que o erro não está dizendo. Nada foi liberado cedo demais. Os dados estão vivos e bem dentro do helper. A rejeição não é sobre memória solta; é sobre ambiguidade, sobre a pergunta "quem é o dono disso?" ter momentaneamente duas respostas. A aposta inteira do Rust é que a ambiguidade, não a alocação, é onde os corpos estão enterrados.

![Um vetor continua vivo enquanto o ownership passa de um binding para um parâmetro de função, e o binding antigo é riscado.](assets/v02-diagram.webp)

### Empréstimo: use sem ser dono

O helper nunca quis ser dono das latências. Ele queria ler elas por um momento e devolver. O Rust tem uma palavra exatamente para isso: um borrow. Escreva `&latencies` e você entrega para a função uma referência, permissão para ler, ownership sem sair do lugar. A assinatura do helper declara que aceita dados emprestados ao receber `&[u64]`, um slice (uma fatia), que é "uma visão para dentro de uma sequência de u64s" e a forma padrão de aceitar dados de lista emprestados (um `&Vec<u64>` é coagido para ele automaticamente, então uma assinatura serve para todo mundo). Essa é a jogada de uso diário da linguagem inteira. Quando você observa programadores Rust em atividade, a esmagadora maioria dos parâmetros de função são borrows, porque a maioria das funções é convidada, não herdeira.

Aí tem o segundo tipo de borrow, e com ele a segunda regra, que você também consegue derivar em vez de decorar. Suponha que uma parte do seu código segura `let worst = &latencies[2];`, um leitor, enquanto outra chama `latencies.push(90)`, um escritor. Um push pode realocar o buffer do Vec, movendo cada elemento para um endereço novo, e nesse ponto `worst` aponta para memória liberada. Em C isso também é terça-feira, do tipo segfault. Então qual tem que ser a regra, se isso vai ser pego em tempo de compilação? Leitores e escritores não podem se sobrepor. Qualquer número de borrows compartilhados (`&`), OU exatamente um borrow exclusivo (`&mut`), nunca os dois de uma vez. Tente a sobreposição e você ganha o irmão da sua abertura:

```text
error[E0502]: cannot borrow `latencies` as mutable because it is also borrowed as immutable
```

Pense numa planilha compartilhada: qualquer número de pessoas pode visualizar ela ao mesmo tempo, mas no momento em que alguém segura o bloqueio de edição, quem está vendo vê um estado congelado e consistente ou ninguém edita. A analogia quebra em um lugar que vale nomear: a planilha impõe o bloqueio em tempo de execução, enquanto o Rust impõe ele antes de o programa existir, que é por que a mesma regra que salva `worst` de um buffer realocado é também, em código multithread, a regra que torna data races impossíveis de representar. Um escritor XOR muitos leitores é a regra de data race vestida de single-thread, e você acabou de encontrar ela num arquivo de cinco linhas.

![Dois estados permitidos mostram muitos leitores ou um escritor sobre um vetor, e um terceiro estado sobreposto é rejeitado pelo compilador.](assets/v03-diagram.webp)

Então as três regras, nenhuma delas recitada, todas elas forçadas pelo problema: todo valor tem um dono; o ownership se move quando você entrega o valor em si; borrows deixam você emprestar acesso, muitos leitores XOR um escritor. Aqui está a síntese, e ela é a alça deste módulo, então guarde: o borrow checker (o verificador de empréstimos) é o code review que você não pode pular. Toda rejeição nesta lição é um comentário que um sênior cuidadoso teria deixado no seu PR, "quem é o dono disso depois da linha 7?", "você está mutando uma lista que outra pessoa está lendo". O verificador não está te impedindo de fazer a coisa. Ele está te impedindo de fazer a coisa de forma ambígua. Você já entrega TypeScript que funciona; você não está sendo rebaixado, você está recebendo o review mais cedo, de um revisor que nunca cansa.

E esse revisor é um projeto vivo, não uma spec congelada. O Polonius-alpha, uma formulação de nova geração do borrow checker que aceita mais programas corretos, chegou ao nightly em 2026-08-04, e o trait solver de nova geração veio atrás em 2026-08-21, dezessete dias depois. Brigas que você perde hoje nas margens são brigas que o verificador está aprendendo a conceder onde você estava certo o tempo todo. O code review que você não pode pular está ficando mais inteligente.

### A caixa que o rustup te deu

Curto e honesto, porque você vai ouvir mitos. O toolchain stable que você acabou de instalar não é só um compilador. clippy (o linter) e rustfmt (o formatador) chegam instalados com o perfil default do rustup, e rust-analyzer (o motor de IDE com quem o seu editor conversa) também é um componente rustup do canal stable, embora esse esteja a um `rustup component add rust-analyzer` de distância em vez de vir pré-instalado, e as extensões Rust da maioria dos editores buscam a própria cópia de qualquer jeito. Ferramentas de primeira mão, não add-ons de terceiros. A única ferramenta que as pessoas esperam na caixa e não ganham é o miri, o interpretador que pega comportamento indefinido em código unsafe: o miri é só nightly. Você não vai precisar dele neste curso, e agora você não vai instalar um substituto de terceiros para ferramentas que você já tem. Dois hábitos começam hoje e nunca param: `cargo fmt` antes de commitar, `cargo clippy` antes de dar push. Eles são o Prettier e o ESLint desta linguagem, só que ninguém debate a config.

![Uma linha do tempo com datas mostra duas melhorias do borrow checker no nightly, o release stable atual, a data de verificação desta lição, e o próximo release esperado.](assets/v04-timeline.webp)

### O que isso te custa

O trade-off, dito sem enfeite: você paga em brigas. O borrow checker rejeita programas que uma linguagem com GC rodaria feliz e corretamente, e iniciantes perdem horas de verdade reestruturando código que "estava bom". Essas horas te compram zero pausas de GC, zero use-after-free, zero data races, e limpeza que acontece numa linha para a qual você consegue apontar. Se esse trade-off vale a pena depende do que você constrói; para a infraestrutura em que esta stack roda, a indústria já votou.

E saiba quando não brigar. `.clone()` faz uma cópia independente com dono próprio, e um clone em código frio, uma struct de config copiada uma vez no startup, é muitas vezes a decisão de engenharia correta, não uma derrota. O pecado é o clone reflexo: o E0382 aparece, você salpica `.clone()`, compila, e você não aprende nada. Eu clonei o meu caminho pelas minhas próprias primeiras semanas de Rust, e muito, e o hábito me custou duas vezes, uma em alocações e uma em nunca ouvir o que o verificador estava tentando me dizer. Neste módulo a saída de emergência do clone é nomeada e depois trancada: todo reparo abaixo tem que ser um conserto baseado em borrow, e a regra de não-clone do challenge é uma que você impõe a si mesmo (os testes dele avaliam entradas e saídas, então uma solução clonada passaria em todos eles, e não provaria nada).

**Vá mais fundo (os 20%).** esta lição te ensinou por que o ownership existe e as jogadas do dia a dia, o move, `&`, `&mut`, e o papel honesto do clone. A progressão completa, stack versus heap, como uma String é disposta na memória, a mecânica de slice, mora no capítulo de ownership do Book, e a versão para ler é o fork interativo da Brown University, com quizzes embutidos e visualizações do Aquascope que animam exatamente os moves e borrows que você acabou de conhecer: [https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html](https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html). Esse fork existe porque ownership é difícil o suficiente para ter gerado um programa de pesquisa acadêmica: o Cognitive Engineering Lab da Brown construiu ele em cima de pesquisa revisada por pares da OOPSLA 2023 e 2024 sobre como as pessoas de fato aprendem este modelo. Salve como bookmark, faça o ch04 com os quizzes esta semana. E depois de toda lição do M4 o Rustlings (`cargo install rustlings`, depois `rustlings init` e `rustlings`) é o pátio de treino: exercícios pequenos de conserte-o-código, o mesmo formato de loop de completion deste módulo, mantido pelo próprio projeto Rust. O lab abaixo não precisa de nada do material de bookmark.

## Lab: espelhe a frota em Rust

A reescrita começa. O `pulse-rs` vira o gêmeo Rust dos tipos do seu motor de sondas, e o TypeScript do M2 vai para a tela bem ao lado do Rust novo, porque você já desenhou esses tipos uma vez e eu me recuso a fingir o contrário.

1. **Conserte a abertura, com princípio.** A própria nota do compilador já te disse: o helper deveria pegar emprestado. Mude a assinatura para um slice e empreste no ponto de chamada:

   ```rust
   fn sum_latencies(latencies: &[u64]) -> u64 {
       latencies.iter().sum()
   }

   fn main() {
       let latencies = vec![212, 487, 1204];
       let total = sum_latencies(&latencies);
       println!("total: {total} across {} probes", latencies.len());
   }
   ```

   `cargo check` fica verde, `cargo run` imprime `total: 1903 across 3 probes`. Dois caracteres de pontuação, e `main` continua dono do Vec dele, empresta ele uma vez, e usa ele depois. Esse é o padrão de conserto que você vai aplicar o módulo inteiro: não "faça o erro sumir", mas "diga quem é dono e quem pega emprestado".

2. **Coloque a spec TS na tela.** Abra o seu `pulse-core` do M2 ao lado do `pulse-rs`. Este é o momento lado a lado; aqui está o TypeScript que você entregou, menos um braço. A união de verdade da sua frota carrega uma quarta variante, `{ kind: 'dns-error'; host: string }`, aquela que o treino de m02-l1 adicionou; o port de hoje espelha as três abaixo e conscientemente deixa dns-error de fora, o mesmo corte que qualquer port faz quando começa pelo core estrutural:

   ```ts
   type ProbeResult =
     | { kind: 'ok'; latencyMs: number }
     | { kind: 'timeout'; budgetMs: number }
     | { kind: 'http-error'; status: number };

   type Verdict = 'up' | 'degraded' | 'down';
   ```

3. **Modele a mesma verdade em Rust.** Substitua `src/main.rs` pelos tipos, variante por variante. Dois novatos: uma struct para o alvo de sonda (um tipo registro, campos e nada mais) e o padrão newtype, `LatencyMs(u64)`, uma struct de um campo só que não custa nada em tempo de execução mas te impede de algum dia passar um número de porta onde uma latência pertence. No M2 você comprou essa segurança com tipos literais; aqui ela é um wrapper de graça:

   ```rust
   #[derive(Debug)]
   struct ProbeTarget {
       name: String,
       url: String,
   }

   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   struct LatencyMs(u64);

   #[derive(Debug)]
   enum ProbeResult {
       Ok { latency: LatencyMs },
       Timeout { budget: LatencyMs },
       HttpError { status: u16 },
   }

   #[derive(Debug, PartialEq, Eq)]
   enum Verdict {
       Up,
       Degraded,
       Down,
   }
   ```

   Olhe `ProbeResult` ao lado do gêmeo TS dele. Um enum Rust É a sua união discriminada, só que o discriminante não é um campo `kind` que você mantém por convenção, é o próprio nome da variante, imposto por construção. As linhas `#[derive(...)]` pedem para o compilador escrever boilerplate para você; `Debug` é o que deixa `{:?}` imprimir um valor, e `Copy` em `LatencyMs` marca ele como barato o suficiente para copiar em vez de mover, que é por que um `u64` nunca te dá E0382 mas um `Vec` dá. A mensagem de erro da abertura disse exatamente isso, vá reler a segunda linha dela. Uma ruga honesta que você pode ter notado: o budget de `Timeout` também veste `LatencyMs`, embora um budget seja uma duração que você escolheu, não uma latência que você mediu. Essa reutilização é uma economia deliberada (um newtype de milissegundos para um arquivo de cinco tipos), não uma afirmação de categoria; no dia em que os dois papéis se encontrarem em uma assinatura só, um segundo newtype, `BudgetMs`, é o mesmo argumento porta-versus-latência aplicado a nós mesmos.

![Uma união discriminada de TypeScript e um enum de Rust ficam lado a lado com linhas pareando as três variantes que combinam.](assets/v05-annotated-code.webp)

4. **Porte o classificador, puro e alimentado por fixture.** Agora a função em volta da qual a estação inteira orbita. As mesmas faixas que você congelou no M2: abaixo de 400 é up, de 400 a 1000 inclusive é degraded, acima de 1000 é down, e um 429 quer dizer que o alvo está vivo mas cansado, então degraded. Adicione abaixo dos tipos:

   ```rust
   fn classify_latency(latency: LatencyMs) -> Verdict {
       let ms = latency.0;
       if ms < 400 {
           Verdict::Up
       } else if ms <= 1000 {
           Verdict::Degraded
       } else {
           Verdict::Down
       }
   }

   fn classify_probe(result: &ProbeResult) -> Verdict {
       match result {
           ProbeResult::Ok { latency } => classify_latency(*latency),
           ProbeResult::Timeout { .. } => Verdict::Down,
           ProbeResult::HttpError { status } => {
               if *status == 429 {
                   Verdict::Degraded
               } else {
                   Verdict::Down
               }
           }
       }
   }
   ```

   Três coisas ganham o seu porquê aqui. Nenhum `return` e nenhum ponto e vírgula nas linhas de cauda: a última expressão de um bloco é o valor dele, essa é só a forma do Rust. `match` é o seu switch exaustivo com `assertNever` embutido na linguagem: apague o braço `Timeout` e `cargo check` recusa o programa inteiro, a prova negativa que você extraiu à mão no M2 agora é o default. E `classify_probe` recebe `&ProbeResult`, um borrow, porque um classificador é convidado: ele lê, ele responde, ele não é dono de nada. Diga a pureza em voz alta também: este motor não chama nada, não busca nada, e isso não é um pedido de desculpas de placeholder. O braço HTTP de verdade dele chega no M5, e a pureza é exatamente o que deixa este mesmo motor compilar para WASM no M7 e rodar no edge. Alimentado por fixture hoje, de propósito.

5. **Alimente ele com uma fixture, e bata na terceira rejeição do módulo.** Escreva um `main` que espelha a ideia de `describe` do M2 e depois classifica uma fixture. Digite exatamente assim primeiro, com `for result in fixture`:

   ```rust
   fn describe(result: &ProbeResult) -> String {
       match result {
           ProbeResult::Ok { latency } => format!("{}ms", latency.0),
           ProbeResult::Timeout { budget } => format!("no answer in {}ms", budget.0),
           ProbeResult::HttpError { status } => format!("HTTP {status}"),
       }
   }

   fn main() {
       let target = ProbeTarget {
           name: String::from("solana-rpc"),
           url: String::from("https://api.mainnet.solana.com"),
       };

       let fixture = vec![
           ProbeResult::Ok { latency: LatencyMs(212) },
           ProbeResult::Ok { latency: LatencyMs(487) },
           ProbeResult::Timeout { budget: LatencyMs(3000) },
           ProbeResult::HttpError { status: 429 },
           ProbeResult::Ok { latency: LatencyMs(1204) },
       ];

       println!("target: {} ({})", target.name, target.url);
       for result in fixture {
           println!("{} -> {:?}", describe(&result), classify_probe(&result));
       }
       println!("probes classified: {}", fixture.len());
   }
   ```

   `cargo check`: E0382 de novo, e esse é mais sorrateiro que a abertura. `for result in fixture` consome o Vec, o loop toma o ownership e come ele elemento por elemento, então o `fixture.len()` depois é use-after-move. O conserto é a mesma ideia de sempre, emprestar em vez de entregar de vez: faça o loop sobre `&fixture`, e nesse ponto cada `result` já é uma referência e os dois argumentos `&result` simplificam:

   ```rust
       for result in &fixture {
           println!("{} -> {:?}", describe(result), classify_probe(result));
       }
       println!("probes classified: {}", fixture.len());
   ```

6. **Verifique.** A trava de aceitação para o artefato desta lição:

   ```bash
   cargo fmt
   cargo clippy
   cargo check
   cargo run
   ```

   `cargo check` tem que passar com zero erros e, como está escrito aqui, zero warnings (eu rodei este arquivo exato hoje, os dois checks limpos). `cargo run` imprime:

   ```text
   target: solana-rpc (https://api.mainnet.solana.com)
   212ms -> Up
   487ms -> Degraded
   no answer in 3000ms -> Down
   HTTP 429 -> Degraded
   1204ms -> Down
   probes classified: 5
   ```

   Segure essa saída ao lado do que o seu classificador TS diz para as mesmas cinco entradas. Os mesmos vereditos, variante por variante. O segundo corpo do motor está vivo.

![O core puro do classificador construído hoje flui sem mudanças para um serviço HTTP futuro e um build WASM no edge.](assets/v06-flowchart.webp)

### Reps de reparo: três borrows de maldade crescente

O loop de completion, no grão mais fino. Cada snippet abaixo falha ao compilar, cada um se conserta com UMA mudança com princípio, e o clone é a saída de emergência nomeada que desvia da lição, então ele está banido. Trabalhe num arquivo de rascunho (`cargo new borrow-reps` se você quiser uma bancada limpa). Preveja o erro antes de rodar `cargo check`, depois leia o que o compilador de fato diz; a lacuna da previsão é onde está o aprendizado.

**Rep 1, o move para dentro de um helper.** O padrão da abertura com roupa nova. Conserte mudando uma assinatura e um ponto de chamada:

```rust
fn max_latency(latencies: Vec<u64>) -> u64 {
    latencies.iter().copied().max().unwrap_or(0)
}

fn main() {
    let latencies = vec![212, 487, 1204];
    let max = max_latency(latencies);
    println!("max {max} out of {} probes", latencies.len());
}
```

**Rep 2, o choque entre leitor e escritor.** E0502 no mundo real. Nenhuma assinatura para mudar aqui; o conserto é uma reordenação, movendo uma linha para que o escritor termine antes de o leitor começar:

```rust
fn main() {
    let mut latencies = vec![212u64, 487, 1204];
    let worst = &latencies[2];
    latencies.push(90);
    println!("worst so far: {worst}ms");
}
```

**Rep 3, o loop que come a própria lista.** Você encontrou esse no lab; agora pegue ele você mesmo, e desestruture já que está aqui: iterar um slice emprestado te entrega referências, e `for &l in` desembrulha elas para que o corpo trabalhe com números simples:

```rust
fn main() {
    let latencies = vec![212u64, 487, 1204];
    let mut slow = 0;
    for l in latencies {
        if l > 1000 {
            slow += 1;
        }
    }
    println!("{slow} slow out of {}", latencies.len());
}
```

Aceitação para as reps: as três compilam, zero clones, e para cada uma você consegue dizer em uma frase QUAL regra disparou e POR QUE a sua mudança satisfaz ela, borrow, reordenação ou reestruturação. Se uma rep te custou três tentativas, ótimo, essa era a dose calibrada de maldade. Uma porta que eu deliberadamente não estou abrindo: em algum momento uma resposta da internet vai sugerir anotações de lifetime (tempo de vida) para problemas com esse formato. Lifetimes além de simplesmente ler eles estão fora do escopo deste curso por design, a trava do tier do M5 nomeia onde eles moram, e nada do M4 precisa deles.

## Challenge

A rep sem guia, e ela é ela mesma um reparo. O starter `fix-the-borrow` mora no painel interativo de coding-challenge na página desta lição (o editor no navegador com o próprio grader e os próprios hints, o mesmo painel que os challenges do lado TypeScript usaram; a maioria dos challenges avaliados neste curso mora lá, embora nem toda rep more, e o da próxima lição mesmo é um build local autoavaliado). Ele te entrega um `latency_report` que move o Vec dele para dentro de `max_latency`, e depois tenta entregar ele para `count_over`, que é o crime da abertura em escala de produção. Faça os dois helpers pegarem `&[u64]` emprestado, empreste `&latencies` nos pontos de chamada, e deixe `latency_report` dono dos dados dele o caminho inteiro, ainda imprimindo o `"max=930,over=1"` esperado da fixture do starter. Nenhum `.clone()` em lugar nenhum, e seja honesto sobre quem checa isso: os testes são pares de entrada e saída, então uma solução com `.clone()` passa em cada um deles enquanto desvia da lição inteira. Dê um grep por `clone` na sua própria solução antes de dar ela por pronta; essa autoauditoria é a rep. Cinco testes, incluindo a lista vazia (o max é 0, não um panic) e a fronteira estritamente maior, porque um off-by-one num limiar é o tipo de bug que classifica um RPC degradado como saudável. Tudo o que você precisa são as três reps que você acabou de fazer; os hints no starter escalam de "qual chamada consumiu ele" até a desestruturação do `for &l in`, gaste eles em ordem.

## Checkpoint

O que você já consegue fazer, concretamente: instalar e verificar um toolchain Rust e dizer o que está de fato na caixa stable (e que o miri não está); ler E0382 e E0502 como informação sobre ownership, não como obstrução; e escolher entre mover, pegar emprestado e pegar emprestado com exclusividade de propósito, com o clone rebaixado de reflexo a decisão. O seu `pulse-rs` passa no `cargo check` com os tipos do core da frota espelhados, três dos quatro braços da união com dns-error conscientemente adiado, e fez isso sem um único clone.

A recuperação de 30 segundos antes de você fechar a aba, em voz alta: quais são as duas coisas que um move previne? (Limpeza dupla e mutação ambígua, um dono significa uma resposta para as duas.) E a regra de borrow em cinco palavras? (Muitos leitores XOR um escritor.)

Um pedido enquanto está fresco: anote qual rep brigou mais com você e o que o compilador disse versus o que você previu. Essa lacuna é dado, para você e para mim, e se a rep 2 em particular pareceu arbitrária, me diga no feedback, porque o conserto por reordenação é aquele cujo porquê merece mais tempo de antena e eu quero saber se ele pegou. O grão deste módulo é um experimento de ensinar por reparo; os seus relatos de atrito são como ele é afinado.

Os seus tipos compilam e o seu classificador roda sobre fixtures. A fixture de hoje é construída no fonte, valores de enum tipados que não podem ser malformados. Na próxima lição a fixture vira texto cru parseado numa fronteira, e a pergunta fica ao vivo: no momento em que uma linha dela está malformada, o que o parser RETORNA? Não existe exceção para lançar; o Rust não tem elas. Na próxima lição, o erro É o tipo de retorno, e o enum que você construiu hoje acaba sendo exatamente a máquina que faz isso funcionar. Traga o enum.
