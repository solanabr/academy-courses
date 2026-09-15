# Uma lista que mora nos bytes da própria conta

Em m02-l1 você construiu o R1, o cabinet-counter: um `Account<T>` Pod com `play_count` e `high_score`, e você subiu a bancada de teste LiteSVM que agora trava cada degrau. Você viu o estado aterrissar sem nenhuma chamada de desserialização em nenhum ponto do caminho. O cast de bytes era o truque inteiro.

Aqui está a coisa sobre a qual aquele contador mente em silêncio. Um `high_score` sozinho não é o que um fliperama guarda. Um fliperama guarda uma *tabela*: os dez melhores, iniciais e tudo, em ordem, para que a pessoa que acabou de perder por 400 pontos consiga ver exatamente qual nome ela tem que bater. Um número só é um placar para um jogo que ninguém joga sozinho.

Então a gente vai dar ao R1 um leaderboard de verdade. E antes de qualquer teoria, faça a única edição que começa isso. Abra o projeto R1 e ache a struct de accounts atrás do `increment`. Esta lição renomeia ela para `PostScore`, porque postar um score num board é o que ela está a ponto de fazer. Olhe para a linha que declara a conta do cabinet:

```rust
pub cabinet: Account<Cabinet>,
```

Mude ela para isto:

```rust
pub cabinet: Slab<Cabinet, Score>,
```

Essa é a jogada estrutural inteira desta lição numa linha só. `Account<Cabinet>` sempre foi `Slab<Cabinet, HeaderOnly>` por baixo do capô: um header, depois uma cauda de nada. Você acabou de trocar a cauda vazia por uma sequência de itens `Score`. A conta agora carrega uma lista limitada nos próprios bytes, e ela continua nunca desserializando. O compilador vai reclamar que `Score` ainda não existe e que ninguém dimensiona a cauda. Ótimo. Essas duas reclamações são a lição.

![No v1 todo toque desserializa e re-serializa o Vec<Score> inteiro; no V2 o Slab é uma view de bytes mutada no lugar sem nada para re-serializar.](assets/v01-comparison.png)

## A versão curta

Você está transformando o contador do R1 numa tabela de recordes limitada. Três coisas carregam isso. Primeiro, o toolkit de campos Pod: `PodU64`, `PodVec<T, MAX>`, `#[derive(bytemuck::Pod)]` e `Nested<T>`, os wrappers que mantêm uma struct capaz de aceitar cast direto dos bytes. Segundo, `Slab<Header, TailItem>`, a primitiva de lista-dentro-de-conta, em que um header fixo é seguido por uma sequência limitada de itens. Terceiro, a recompensa que m01-l4 te prometeu: `#[event(bytemuck)]`, um evento zero-copy que você emite e lê de volta dos logs da transação.

A pegadinha honesta atravessa tudo isso, então ouça ela uma vez logo de cara: a capacidade de um Slab é fixa em tempo de compilação. Você dimensiona para o pior caso, você paga rent pelos slots vazios, e uma escrita além de `MAX` é um erro duro, nunca um resize automático. Esse limite fixo não é uma verruga. É o preço exato de uma lista que você nunca precisa serializar, e escolher `MAX` é uma decisão de design de verdade.

Sobre autonomia: o Lab te entrega o layout do Slab pronto e a struct do evento. Você escreve sozinho a lógica de admitir e despejar e a asserção de ordenação. O challenge solo do fim, o cutoff do leaderboard, você faz sem apoio nenhum. Esta lição é onde as rodinhas do layout de dados saem e ficam fora.

## Construindo um leaderboard dentro de uma conta só

### O toolkit de campos Pod

Relembre a regra de m02-l1: `Account<T>` exige `T: Pod`, plain-old-data, um layout fixo sem surpresas de preenchimento, para que o framework consiga fazer cast dos bytes da conta direto para `&T` com zero cópia. Essa regra não amolece porque os seus dados ficaram mais interessantes. Um `Score` tem que ser Pod. Uma tabela de `Score` tem que ser Pod. Então a primeira pergunta é mecânica: como você constrói uma struct Pod a partir dos campos que você de fato quer?

Um `u64` puro estava de bom tamanho no header do R1, porque o header fica num offset com alinhamento de 8 bytes garantido. Um item de cauda não ganha essa promessa: ele cai onde o header e o campo de comprimento deixarem, num passo de `size_of::<Score>()`, e nenhum dos dois é coisa sobre a qual você queira raciocinar campo a campo. Então num item de cauda você recorre ao wrapper de campo em vez disso. `PodU64` é um `u64` guardado como um array de bytes com métodos de acesso, então ele tem alinhamento 1 e lê corretamente de qualquer offset. Mesma história para `PodI128`, `PodBool` e o resto da família. Eles não são tipos novos nos quais você tem que pensar. São os tipos velhos com o imposto de alinhamento pré-pago.

O custo que você paga por isso é um tiquinho de cerimônia no ponto de uso: você passa por acessadores em vez de tocar o valor direto.

```rust
let mut plays = cabinet.play_count.get();      // read: byte array -> u64
plays += 1;
cabinet.play_count = PodU64::from(plays);      // write: u64 -> byte array
// A Slab Derefs to its header, so header fields are plain field access.
// .get() reads; PodU64::from(v) / v.into() writes. There is no .set().
```

Esse `.get()` na saída e uma conversão `From` na entrada é o imposto inteiro. Em troca, o campo lê corretamente não importa em que offset ele caia dentro da conta, que é o que faz o registro inteiro aceitar cast.

```rust
use anchor_lang::prelude::*;

// Deriving bytemuck::Pod makes a fixed-size struct castable straight from bytes:
// every field is itself Pod, so the whole struct has a defined layout and no
// padding. This is one leaderboard entry. (#[pod_wrapper] is for ENUMS — structs
// take the derive directly, and V2 rejects the attribute on a struct outright.)
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Score {
    pub player: Address,   // 32 bytes: who set it
    pub points: PodU64,    //  8 bytes: the score
    pub slot: PodU64,      //  8 bytes: when, for tie-breaking display
}                          // 48 bytes, fixed, no padding
```

`#[derive(bytemuck::Pod)]` (pareado com `bytemuck::Zeroable`) é o que certifica a sua própria struct de tamanho fixo como Pod: ele checa que todo campo é Pod e dá ao tipo a bênção do cast de bytes. Recorra a `#[pod_wrapper]` só num *enum* — o V2 rejeita de saída o atributo numa struct, com uma mensagem te mandando usar o derive em vez disso. Quando um campo Pod é ele mesmo uma struct que você quer aninhar, você embrulha ela em `Nested<T>` para que o alinhamento dela siga definido dentro do pai em vez de abrir um buraco no layout. E quando você quer uma lista limitada como *campo* em vez de como a cauda inteira da conta, isso é `PodVec<T, MAX>`: um vetor com capacidade de tempo de compilação, o comprimento dele guardado inline, sem heap em lugar nenhum.

![Uma tabela mapeando cada wrapper Pod (PodU64, o derive bytemuck::Pod, Nested, PodVec, Slab) para o tipo puro que ele substitui e para quando usar ele.](assets/v02-table.png)

Por que o framework te fez passar por isso em vez de simplesmente te deixar escrever `Vec<Score>`? Porque não existe almoço grátis no cast de bytes. Um `Vec` é um ponteiro, um comprimento e uma capacidade apontando para memória de heap que não existe dentro de uma conta. Para fazer uma lista aceitar cast você tem que deitar ela plana e fixa, no lugar, e os wrappers são como você faz isso sem escrever aritmética de offset na mão. O que nos leva à primitiva que segura a tabela inteira.

### Alinhamento é o imposto do Pod

Vale ver a falha exata que os wrappers previnem, porque é a que aparece como um aviso que você fica tentado a silenciar do jeito errado. Suponha que você pule o toolkit e escreva a entrada na mão como uma struct pura com campos nativos:

```rust
// DON'T: bare multi-byte fields inside a byte-cast struct.
#[repr(C)]
pub struct Score {
    pub player: Address,  // 32 bytes, fine
    pub points: u64,      // native u64: alignment 8
    pub slot: u64,        // native u64: alignment 8
}
```

Um `u64` nativo quer sentar numa fronteira de 8 bytes. Um `Score` na cauda mora no offset para o qual o header e o campo de comprimento empurram ele, então esse alinhamento não é garantido, e o cast tem que desistir em vez de te entregar uma referência desalinhada, que é comportamento indefinido em Rust. O instinto é recorrer a `#[repr(packed)]` para tirar o preenchimento que o aviso parece culpar. Isso é exatamente ao contrário. `repr(packed)` é o que *cria* o risco de referência desalinhada: tomar uma referência a um campo empacotado é a cilada, não a correção. A jogada certa é o toolkit. `PodU64` guarda o valor como um `[u8; 8]` lido por `.get()` e escrito por uma conversão `From`, então o alinhamento dele é 1 e ele lê corretamente de *qualquer* offset, e `#[derive(bytemuck::Pod)]` (ou `Nested<T>` para um campo de struct aninhada) mantém o alinhamento do registro inteiro definido para que o cast direto de bytes siga sólido.

![Um campo u64 puro causa uma referência desalinhada no cast; repr(packed) piora isso garantindo o desalinhamento; campos PodU64 mais o derive bytemuck::Pod dão alinhamento 1 e um cast sólido.](assets/v03-annotated-code.png)

### Slab: uma lista que mora na conta

Aqui está a definição, na hora certa. `Slab<Header, TailItem>` é um layout de conta que é uma struct `Header` fixa seguida por uma sequência limitada de registros `TailItem` no próprio lugar. É isso. O header são os seus campos fixos, aqueles dos quais todo cabinet tem exatamente um. A cauda é a lista. E a identidade que você já conheceu faz a ficha cair: `Account<T>` é literalmente `Slab<T, HeaderOnly>`, um header com uma cauda de um marcador de tamanho zero. O contador do R1 era um Slab esse tempo todo. Ele só não tinha nada na cauda.

Então o `Cabinet` de m02-l1 vira o header. Ele mantém os dois contadores dele e ganha dois campos novos que o degrau do leaderboard precisa de qualquer jeito: um `authority` para que as seeds do PDA não dependam mais de um único jogador, e o `bump` guardado a partir do qual você re-deriva a validação:

```rust
// The HEADER: the fixed part every cabinet has exactly one of.
// R1's Cabinet, plus two new fields: authority (for the seeds) and bump.
#[account]
#[repr(C)]
pub struct Cabinet {
    pub authority: Address,
    pub play_count: PodU64,
    pub high_score: PodU64,   // still here: the single top score, for quick reads
    pub bump: u8,
}
```

E a conta no seu handler é o Slab que pareia esse header com uma cauda de `Score`. O Slab rastreia o próprio comprimento de cauda e deriva a capacidade dele de quanto espaço a conta recebeu no init. Você lê a cauda como um slice e muta ela no lugar:

```rust
#[derive(Accounts)]
pub struct PostScore {
    #[account(mut)]
    pub player: Signer,

    #[account(
        mut,
        seeds = [b"cabinet", cabinet.authority.as_ref()],
        bump = cabinet.bump,
    )]
    pub cabinet: Slab<Cabinet, Score>,
}

// The Slab surface you work against:
//   *cabinet                    -> Deref/DerefMut to Cabinet, so the header's
//                                  fields are plain field access:
//                                  cabinet.play_count.get()
//                                  cabinet.play_count = PodU64::from(n)
//   cabinet.as_slice()          -> &[Score]      (the live list, a byte view)
//   cabinet.as_mut_slice()      -> &mut [Score]
//   cabinet.len() / .capacity() -> usize         (live items / MAX at allocation)
//   cabinet.is_full()           -> bool
//   cabinet.get(i) / .get_mut(i) / .first() / .last() / .iter()
//   cabinet.try_push(Score)     -> Result<(), ProgramError>
//                                  (Err past capacity; there is no infallible push
//                                   and no implicit growth)
//   cabinet.address()           -> &Address       (the account's OWN address; this
//                                  one lives on the Slab, not on the Cabinet header)
//   cabinet.resize_to_capacity(n) -> Result<()>   (the ONLY growth path: reallocs
//                                  the account and settles the rent difference)
//   Slab::<Cabinet, Score>::space_for(MAX) -> usize   (const, for `space =`)
```

Repare que `cabinet.as_mut_slice()` te entrega um `&mut [Score]` puro. Uma vez que você tem esse slice, ordenar e comparar são Rust comum sobre memória comum, só que a memória é a conta e toda escrita que você faz no slice já está persistida. Não existe passo de serialize no fim do handler porque não tem nada para serializar de volta. O slice *são* os bytes da conta.

Como o Slab sabe quantos itens `Score` estão vivos contra quantos slots estão alocados-mas-vazios? Ele guarda o próprio comprimento como um `u32` little-endian na conta, bem entre o header e os itens, do mesmo jeito que um `Vec` rastreia comprimento separado da capacidade, só que os dois moram dentro da conta e nenhum deles pode apontar para um heap. `capacity()` é derivado do comprimento de dados da conta: total de bytes, menos o discriminator, menos o header, menos aquele campo de comprimento, dividido por `size_of::<Score>()`. É por isso que o espaço que você aloca no init é o teto até você mudar ele de propósito: `try_push` além da capacidade devolve um erro em vez de crescer, e o único jeito de a conta ficar maior é uma chamada explícita de `resize_to_capacity(n)` que realoca o buffer e acerta a diferença de rent. Nada cresce nas suas costas. Existe uma primitiva irmã que vale nomear aqui para você recorrer à certa: `PodVec<T, MAX>` é a lista limitada *no nível do campo*, a que você coloca dentro de um header quando uma struct precisa de uma listinha inline própria, enquanto `Slab<Header, TailItem>` é a *no nível da conta*, em que a lista é a cauda inteira da conta. Regra de bolso: uma lista limitada que é o ponto da conta é uma cauda de Slab, uma lista limitada pequena pendurada num registro maior é um campo `PodVec`.

![A conta é um discriminator, depois o header Cabinet fixo, depois um campo de comprimento vivo de 4 bytes, depois dez slots Score fixos de 48 bytes; slots não preenchidos continuam alocados e pagam rent.](assets/v04-diagram.png)

Esse diagrama é também o tradeoff te encarando de volta. Dez slots a 48 bytes dão 480 bytes de cauda, alocados e com rent pago no instante em que você faz o init da conta, tenha o cabinet um score nele ou dez. Que é a batida honesta em que este design inteiro se apoia.

### Por que MAX é fixo, e por que isso é o ponto inteiro

Vale desacelerar aqui, porque "faça ela dinâmica e pronto" é o instinto óbvio e vale ver exatamente por que o Slab se recusa.

Comece do que você de fato quer: acrescentar um score, manter a lista ordenada, nunca perder as entradas do topo. A resposta ingênua é um `Vec<Score>` que cresce no heap e realoca quando enche. Isso falha dentro de uma conta por um motivo seco: uma conta é uma região fixa de bytes com um dono e um saldo de rent, não um heap que você pode crescer. Não existe lugar para o `Vec` crescer *para dentro* sem uma instrução `realloc` separada e explícita que move rent e muda o tamanho da conta. O crescimento nunca é de graça e nunca é automático.

A próxima resposta ingênua mantém o `Vec` mas paga o imposto de serialização: desserializa a lista inteira na leitura, re-serializa na escrita, deixa o borsh lidar com o comprimento variável. Esse é precisamente o modelo do v1, e é exatamente disso que m02-l1 te ensinou que o cast Pod existe para escapar. Você estaria comprando flexibilidade de volta re-introduzindo o custo que o framework inteiro foi construído para remover. Para uma lista que você toca a cada partida, essa troca está de cabeça para baixo.

Então o requisito fica mais afiado: você quer mutação no lugar com zero serialização, numa lista cujo comprimento muda. O único jeito de ter uma lista que aceita cast de bytes é deitar ela plana e fixa, o que quer dizer que a capacidade tem que ser conhecida antes de você escrever um byte que seja. Isso é `MAX`. O Slab tem sim uma válvula de escape, `resize_to_capacity(n)`, que realoca a conta e acerta a diferença de rent, mas repare que é uma instrução que você roda de propósito, não algo que um `try_push` faz nas suas costas. Crescimento *implícito* e um cast de bytes sem serialização são as duas coisas que você não pode ter ao mesmo tempo, e o Slab escolhe o cast.

O que deixa a frase para guardar: um `MAX` fixo é o preço de uma lista sem serialização, porque uma lista da qual você consegue fazer cast direto dos bytes precisa ter o tamanho conhecido antes do primeiro byte ser escrito, e mudar ele depois é um realloc explícito, não um efeito colateral de inserir. Você paga rent em slots vazios e faz o despejo você mesmo, e em troca todo toque custa o item que você tocou em vez do comprimento da lista. Escolher `MAX` agora é trabalho seu, e é um de verdade. Dez é um leaderboard. Dez mil é uma conta de rent que você vai lamentar.

Percorra isso uma vez de forma concreta, num board minúsculo com `MAX = 3`, e a disciplina de despejo para de ser abstrata. Comece vazio. Um score de `50` entra: o board está abaixo da capacidade, então ele aterrissa, e o cutoff, o menor score vivo, é `50`. Depois `90`: ainda abaixo da capacidade, então admita ele sem comparação nenhuma, e o cutoff continua `50`, porque `50` ainda é o menor de `[90, 50]`. Depois `70`: o board enche para `[90, 70, 50]`, cutoff `50`. Agora o board está cheio e um `60` chega. Ele é estritamente maior que o cutoff `50`, então o `50` é sobrescrito no lugar e o board vira `[90, 70, 60]`, cutoff novo `60`. Em seguida um `60` chega de novo: ele *empata* com o cutoff, então é rejeitado, o board fica inalterado. Por fim um `40`: abaixo do cutoff, rejeitado. Essa sequência, admitir-abaixo-do-cap, despejar-só-se-estritamente-maior, empate-perde, é exatamente a lógica que você escreve na cauda do Slab no Lab e de novo do zero no challenge. Mesmas regras, uma vez que você vê elas se mexerem.

![Vec-com-realloc paga serialização e crescimento manual; Vec borsh re-introduz o imposto de serialização; o Slab de MAX fixo faz cast no lugar mas te faz pagar rent em slots vazios e despejar você mesmo.](assets/v05-comparison.png)

Isto não é uma preferência abstrata que o framework inventou no vácuo. Quando o design do V2 estava sendo discutido em público, a nota mais alta da comunidade era exatamente essa fricção. O ChewingGlass disse isso sem rodeios na discussão #3742, a thread "What do you want to see in Anchor V2?": "the default serialization should probably behave more like zero-copy but with better UX (IE not having to try to have perfect byte alignment, etc). Not sure if that's possible. But borsh is kind of terrible." A issue de design #4390 cita esse comentário de volta, nas próprias palavras dela: "As #3742 discussion feedback put it: *the default serialization should probably behave more like zero-copy but with better UX*," e lista a #3742 nas referências dela. Em outro ponto da mesma discussão o mesmo comentarista crava a outra metade da reclamação, sobre ergonomia do lado do cliente: "Boilerplate kills new devs because they don't know the sacred incantations." O Slab e o toolkit Pod são a resposta entregue para a primeira metade: zero-copy por padrão, com wrappers que pagam o imposto de alinhamento de bytes por você em vez de fazer disso problema seu. Voz da comunidade virada estrutura de dados.

![O comentário zero-copy-com-UX-melhor do ChewingGlass na discussão #3742 é citado de volta pela issue de design #4390, que alimentou o empurrão de benchmarks #4355 e foi entregue como o toolkit Pod.](assets/v06-timeline.png)

### O dividendo do Pod: eventos zero-copy

Agora a recompensa que m01-l4 referenciou lá na frente. Você tem um leaderboard que muta no lugar. Quando um score novo entra no board, você quer contar para o mundo lá fora: um indexador, um frontend, um bot de Discord que posta os novos dez melhores. Na Solana você faz isso escrevendo nos logs da transação, e o `emit!` do Anchor desce para `sol_log_data`, a syscall que larga um blob prefixado por comprimento nos logs para quem estiver lendo a transação pegar.

O blob não é payload cru sozinho. O `emit!` prefixa ele com o discriminator de 8 bytes do evento, o mesmo mecanismo de tag que você conheceu em m01-l4, para que um leitor consiga distinguir um tipo de evento de outro antes de tentar decodificar. Quando você puxa os logs de uma transação confirmada, cada evento emitido aparece como uma linha `Program data:` carregando esse blob, codificado em base64. Um consumidor casa os primeiros 8 bytes contra o discriminator com que ele se importa, depois decodifica o resto. Esse passo de decode é onde as duas variantes se separam.

O `#[event]` default serializa esse blob com **wincode**. Seja preciso sobre o que o wincode é, porque o nome é usado de forma solta e isso importa na próxima lição: é uma *implementação* diferente, não um *formato* diferente. O `BORSH_CONFIG` dele produz bytes idênticos byte a byte ao borsh (duas exceções documentadas, ordenação de `HashMap`/`HashSet` e NaN, que você encontra em m02-l3), então qualquer coisa no wire segue compatível com borsh enquanto o código que faz o trabalho é do próprio V2. O que quer dizer que a economia de CU é real e a codificação continua sendo uma codificação: para uma struct que você já teve o trabalho de tornar Pod, isso é o imposto de serialização voltando pela porta dos fundos. Então o V2 te dá `#[event(bytemuck)]`: o mesmo caminho do `sol_log_data`, mas o payload é um `memcpy` zero-copy dos bytes da struct em vez de um serialize campo a campo. Sem borsh, mesmos logs.

```rust
// A zero-copy event: emitted via sol_log_data as a raw memcpy of the struct.
// Same rules as any Pod type - fixed layout, Pod fields, defined alignment.
#[event(bytemuck)]
#[repr(C)]
pub struct HighScorePosted {
    pub cabinet: Address,
    pub player: Address,
    pub points: PodU64,
    pub cutoff: PodU64,   // the lowest score still on the board after this post
}
```

Quanto mais barato é tudo isso? O único número que o projeto publica mora no `event.rs` dele, e compara contra o v1: eventos wincode, o `#[event]` simples do V2, rodam de 3 a 10 vezes menos unidades de computação que os eventos borsh do v1, dependendo do payload. Os mesmos bytes no wire, de 3 a 10 vezes menos trabalho para produzir eles; é isso que uma implementação mais rápida de um formato idêntico te compra. Eu estou citando isso como a alegação do projeto, não lavando ela para virar um fato medido meu, e repare no que ele *não* diz: ele não põe número nenhum em `#[event(bytemuck)]` contra o `#[event]` simples. Essa lacuna você mede você mesmo com `anchor test --profile` quando chegar no módulo de instrumentação, e você deveria. O mecanismo, porém, não está em dúvida: um `memcpy` de uma struct fixa é estritamente menos trabalho que percorrer os campos dela por qualquer serializador, wincode incluído.

Tem uma cilada que vem de brinde com a velocidade, e é do tipo que falha em silêncio em produção. `#[event]` e `#[event(bytemuck)]` escrevem *bytes diferentes* no log. Um é wincode no wire, o outro é um memcpy cru. Os dois saem por `sol_log_data`, então o evento está genuinamente nos logs de um jeito ou de outro. Mas um leitor construído para decodificar a variante borsh vai receber lixo dos bytes bytemuck e vice-versa. Todo consumidor lá na frente tem que decodificar com a mesma variante que o programa emite. Escolha uma, anote, e garanta que o seu indexador recebeu o recado.

![As duas variantes de evento emitem por sol_log_data, mas uma carrega bytes borsh e a outra um memcpy, então um leitor tem que decodificar com a variante que o programa usou.](assets/v07-diagram.png)

Esse é o toolkit. Um tipo de entrada Pod, um Slab para segurar uma sequência limitada deles, um `MAX` fixo que você escolheu de propósito, e um evento zero-copy para anunciar mudanças. Hora de ligar isso no R1 e ver rodar.

## Lab: parafuse uma tabela de recordes no R1

Você vai estender o handler `PostScore` do R1 para que todo score seja admitido num board limitado e ordenado, e um `#[event(bytemuck)]` dispare quando o board mudar. O layout do Slab e a struct do evento acima são dados. A lógica de admitir e despejar, e a asserção de teste de que o board segue ordenado e limitado, são suas para escrever. Essa divisão é deliberada: a forma é mostrada, a disciplina é praticada.

Primeiro, o toolchain. Você instalou o RC lá em m01-l2 e confirmou ele de novo no topo de m02-l1; se ele estiver faltando, aqui está o mesmo build via git documentado, porque o Anchor V2 é um release candidate instalado a partir do git, não do canal de binários pré-construídos do `avm`:

```bash
# Anchor V2 2.0.0-rc.1 - git otter-sec/anchor, tag v2.0.0-rc.1 (commit e4878b6d).
# RC/alpha, and no release binary is published for it. The tag is a fixed point;
# the anchor-next branch tip it sits on is not. The Docker verify gate in m08-l2
# names the same commit outright. Re-verify the tag before you rely on it.
# macOS, if the build trips on LTO: prefix that line with CARGO_PROFILE_RELEASE_LTO=off
cargo install --git https://github.com/otter-sec/anchor.git \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
```

A bancada de teste é a mesma que você subiu em m02-l1: `anchor-v2-testing`, que embrulha o LiteSVM e reexporta as peças que o arquivo de teste precisa. Você continua não dependendo do `litesvm` pelo nome. A única dev-dependency nova que esta lição acrescenta é `base64`, para o decode de evento abaixo:

```toml
# programs/cabinet-counter/Cargo.toml
# anchor-v2-testing at tag v2.0.0-rc.1 pins litesvm 0.11.0 internally (crates.io
# latest is 0.15.2 as of 2026-08-22; do not float ahead of the harness by pulling
# litesvm in yourself). This is why the tag and not the branch: the anchor-next tip
# has already moved that pin to =0.13.1, and a harness that changes SVM majors
# under a green test suite is the exact failure the pin exists to prevent.
# base64 0.22 is pinned on purpose (0.23.1 is current as of 2026-08-22):
# re-verify the Engine API before bumping it.
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor.git", tag = "v2.0.0-rc.1" }
base64 = "0.22"
```

Agora os passos.

1. **Troque o tipo da conta, e renomeie o que o degrau superou.** Você já fez a troca de tipo na abertura: `pub cabinet: Account<Cabinet>` vira `pub cabinet: Slab<Cabinet, Score>`. Agora termine a renomeação que vem junto, porque o R1 não está mais contando, está postando: `Increment` vira `PostScore` e `increment` vira `post_score`; `Init` vira `InitCabinet` e `init` vira `init_cabinet`. Acrescente a struct `Score` com `#[derive(bytemuck::Pod, bytemuck::Zeroable)]` e o evento `HighScorePosted` com `#[event(bytemuck)]`, os dois exatamente como mostrado na teoria acima, e dê ao header `Cabinet` os dois campos novos dele, `authority` e `bump`.

   `init_cabinet` é mais que uma renomeação, porque esses dois campos novos do header têm que ser escritos por alguém e nada mais vai fazer isso. Acrescente as duas linhas ao corpo do handler:

   ```rust
   pub fn init_cabinet(ctx: &mut Context<InitCabinet>) -> Result<()> {
       let bump = ctx.bumps.cabinet;               // the canonical bump the macro found
       let cabinet = &mut ctx.accounts.cabinet;
       cabinet.authority = *ctx.accounts.authority.address();
       cabinet.bump = bump;
       cabinet.play_count = PodU64::from(0);
       cabinet.high_score = PodU64::from(0);
       Ok(())
   }
   ```

   Pule essas duas atribuições e tudo continua compilando, que é a parte perigosa: `authority` fica todo zero e `bump` fica `0`, então todo `post_score` posterior falha no constraint de `seeds` e `bump` contra um endereço que nunca foi derivável, e o erro não te diz nada sobre o porquê.

   Resultado esperado depois deste passo: ainda não faz build, e o teste `cabinet_round_trips` de m02-l1 também não. Isso é correto e vale nomear em vez de descobrir. O teste chama `instruction::Init` e `accounts::Init`, que não existem mais com esses nomes, e ele afirma `raw.len() == 24`, que era verdade de uma conta só de header e é falso no instante em que existe uma cauda. Você vai substituir essa asserção no passo 4. Reaponte os builders do teste para `InitCabinet`/`PostScore` agora para que a única coisa que sobre vermelha seja a lógica que você está a ponto de escrever. Se você trouxe o teste de reset do challenge de m02-l1 para este workspace, ele fica vermelho pelo mesmo motivo — mesmas renomeações, mesmo reapontamento.

2. **Dimensione a cauda no init.** Nos constraints de conta do seu handler `init_cabinet`, o `space` do cabinet agora tem que cobrir o header mais os slots de score. Defina `MAX_SCORES = 10` como um `const` e deixe o Slab fazer a aritmética: o discriminator, mais o header `Cabinet`, mais o campo de comprimento vivo de 4 bytes, mais `MAX_SCORES * size_of::<Score>()`. É exatamente isso que o `space_for` computa, que é por que você nunca escreve isso na mão:

   ```rust
   pub const MAX_SCORES: usize = 10;

   #[derive(Accounts)]
   pub struct InitCabinet {
       #[account(mut)]
       pub authority: Signer,
       #[account(
           init,
           payer = authority,
           // discriminator + Cabinet header + len field + MAX_SCORES slots.
           // space_for is a const fn on the Slab, so you never hand-roll the math.
           space = Slab::<Cabinet, Score>::space_for(MAX_SCORES),
           seeds = [b"cabinet", authority.address().as_ref()],
           bump,
       )]
       pub cabinet: Slab<Cabinet, Score>,
       pub system_program: Program<System>,
   }
   ```

   Isso dá 480 bytes de cauda (`10 * 48`), mais o header e o campo de comprimento de 4 bytes dele, alocados no instante em que o cabinet existe, cheios ou não. É aqui que você compromete rent com os slots vazios, então vale ler o número direto da conta e sentir ele: um cabinet vazio e um cheio custam a mesma coisa. Esse é o tradeoff, feito concreto. Se você tivesse escolhido `MAX_SCORES = 1000`, você estaria pagando rent em 48,000 bytes de cauda para um leaderboard que quase nenhum cabinet vai encher algum dia.

3. **Escreva a lógica de admitir e despejar.** Esta é a parte que você implementa. Em `post_score`, depois de subir o `play_count`, insira o `Score` novo na cauda e mantenha o board limitado e ordenado do maior para o menor. As regras:
   - Enquanto `cabinet.len() < cabinet.capacity()`, sempre dê `try_push` no score.
   - Quando o board estiver cheio, ache o cutoff atual (o menor score vivo). Admita só se o score novo for *estritamente* maior que o cutoff, e quando for, sobrescreva o slot do cutoff no lugar por `cabinet.as_mut_slice()`. Um empate não despeja.
   - Ordene a cauda viva em ordem decrescente para que o slot 0 seja sempre o score do topo, e mantenha o `high_score` do header em sincronia com o slot 0.

   ```rust
   pub fn post_score(ctx: &mut Context<PostScore>, points: u64) -> Result<()> {
       let cabinet = &mut ctx.accounts.cabinet;
       let player = *ctx.accounts.player.address();
       let cabinet_address = *cabinet.address();

       // routine: one more play recorded (Slab derefs to the Cabinet header)
       let plays = cabinet.play_count.get().checked_add(1)
           .ok_or(CabinetError::Overflow)?;
       cabinet.play_count = PodU64::from(plays);

       let entry = Score {
           player,
           points: PodU64::from(points),
           slot: PodU64::from(Clock::get()?.slot),
       };

       // TODO(you): admit `entry` under the fixed capacity.
       //   - under capacity  -> cabinet.try_push(entry)?;
       //   - full + strictly beats cutoff -> overwrite the cutoff slot in place
       //   - full + ties or loses -> the board is unchanged. Do NOT return early:
       //     every call emits, so a rejected post reports the unchanged cutoff and
       //     an indexer can still see that the attempt happened.
       //   - then sort cabinet.as_mut_slice() descending and sync high_score
       // The cutoff you compute here is the value you emit below.
       let cutoff = todo!("return the lowest live score after admitting");

       emit!(HighScorePosted {
           cabinet: cabinet_address,
           player,
           points: PodU64::from(points),
           cutoff: PodU64::from(cutoff),
       });
       Ok(())
   }
   ```

   A lógica aqui é a mesma disciplina do challenge solo abaixo, só que operando numa cauda de Slab em vez de num `Vec`. Faça funcionar aqui onde você consegue ver a conta, depois faça isso do zero no challenge.

4. **Escreva o teste LiteSVM.** Insira *mais que* `MAX` scores num cabinet só, transbordando o board de propósito, depois afirme duas coisas. Primeiro, que a cauda segura exatamente `MAX` entradas e que elas estão em ordem decrescente com só os scores do topo retidos. Segundo, leia o evento `HighScorePosted` de volta dos logs da transação e afirme que o `cutoff` dele bate com o menor score vivo do board. O decode do evento é a única peça de encanamento de teste que você não viu, então aqui está ela, completa:

   ```rust
   // Pull a #[event(bytemuck)] payload back out of the transaction logs.
   // The program wrote it via sol_log_data as: [8-byte discriminator][raw struct].
   // We match the discriminator, then bytemuck-cast the rest. Decoding with the
   // SAME variant the program emitted is the rule from the theory - here it is bytemuck.
   use base64::Engine as _; // decode() is a trait method on Engine

   fn decode_highscore(logs: &[String]) -> HighScorePosted {
       for line in logs {
           let Some(b64) = line.strip_prefix("Program data: ") else { continue };
           let bytes = base64::engine::general_purpose::STANDARD
               .decode(b64).expect("valid base64 program data");
           if bytes.len() >= 8 && &bytes[..8] == HighScorePosted::DISCRIMINATOR {
               return *bytemuck::from_bytes::<HighScorePosted>(&bytes[8..]);
           }
       }
       panic!("no HighScorePosted event in logs");
   }
   ```

   ```rust
   // TODO(you): the overflow-and-order assertion.
   //   - post MAX_SCORES + 3 scores with distinct values through post_score
   //   - fetch the cabinet, read its Slab tail as &[Score] via as_slice()
   //   - assert tail.len() == MAX_SCORES
   //   - assert the tail is sorted descending (non-increasing: equal scores are
   //     legal side by side, because a tie only loses against a FULL board's cutoff)
   //   - assert the smallest retained score equals the last event's cutoff
   ```

5. **Rode.** Com o toolchain instalado e o teste escrito:

   ```bash
   anchor test
   ```

   Uma rodada verde comprova a forma que você construiu: o board admitiu mais scores que `MAX`, guardou só os `MAX` do topo em ordem decrescente, despejou o resto, e o evento zero-copy fez o caminho de ida e volta pelos logs com um `cutoff` que concorda com o board. Se a asserção de cauda falhar com entradas fora de ordem, o seu sort rodou antes do insert ou você ordenou de forma crescente. Se `decode_highscore` der panic, ou você emitiu o `#[event]` borsh por engano ou a sua leitura de indexador está procurando o discriminator errado. Os dois são a cilada de "decodifique a variante que você emitiu" aparecendo exatamente onde a teoria disse que apareceria.

![Se o board está abaixo de MAX, faça push; se cheio, admita só acima do cutoff, sobrescrevendo aquele slot; empates são rejeitados, scores admitidos ordenam de forma decrescente, e todo caminho ainda emite o evento.](assets/v08-flowchart.png)

## Challenge: o cutoff do leaderboard

Agora o degrau solo, sem Slab e sem conta, só a disciplina destilada numa função pura sobre a qual você consegue raciocinar isoladamente. Esta é a lógica de inserção limitada que um Slab Pod te obriga a ter on-chain, levantada para fora onde nada mais está no caminho.

Implemente `admit`:

```rust
/// Admit a new score to a fixed-capacity cabinet high-score board and return
/// the LOWEST score still on the board afterwards - the leaderboard "cutoff".
///
/// Rules:
///   * while the board has fewer than `cap` entries, always admit the score;
///   * once the board is full, the board retains the `cap` highest scores, so a
///     score that only ties the current cutoff cannot raise it;
///   * keep the board bounded and highest-first, including when the board you
///     were handed already exceeds `cap`.
///
/// Return the cutoff (the minimum retained score), or 0 for an empty board.
fn admit(board: Vec<u64>, score: u64, cap: usize) -> u64 {
    todo!()
}
```

A barra de aceitação, todos os sete casos:

- Um score admitido abaixo da capacidade aparece no board e o cutoff reflete ele.
- Num board cheio, só um score estritamente maior que o cutoff despeja o mínimo.
- Um score que só empata com o cutoff deixa o cutoff exatamente onde ele estava.
- O board nunca excede `cap` — e dois casos te entregam um board que *começa* acima da capacidade, então o aparo tem que acontecer quer o score novo seja admitido ou não.
- O valor devolvido é o menor score retido, e `0` quando o board está vazio.

Três dicas, na ordem em que você vai querer elas:

1. Enquanto `board.len() < cap`, todo score é admitido sem comparação nenhuma.
2. Quando o board está cheio, a decisão de admissão é `beats_cutoff`, a pequena `const fn` acima do `admit`: estritamente maior que o mínimo atual entra, um empate não. Implemente ela ali e roteie o ramo de board cheio por ela.
3. Ordene do maior para o menor, trunque em `cap`, e devolva o último, o menor score retido. O truncate é o passo que os casos acima da capacidade existem para pegar — devolva o mínimo do board *retido*, não do board que te entregaram.

O starter e os testes estão em `lessons/m02-l2/high-score-cutoff/`. A regra de admissão está fatorada em `beats_cutoff` para que o compilador consiga comprovar ela enquanto faz o build — um dispositivo de asserção em tempo de compilação que você vai reencontrar no challenge de constraint de m03-l3, e o que a avaliação compile-only de produção de fato exige: uma regra não corrigida simplesmente não faz build. Rode os sete vetores também, até os sete passarem. A função é pequena. O ponto não é o volume de código, é internalizar que on-chain você não consegue crescer no heap para escapar disso. O limite é o jogo inteiro, então o insert tem que respeitar ele todas as vezes.

## Antes do próximo degrau

Responda isto em uma frase antes de seguir em frente, porque é o conceito que tem que grudar: por que um `MAX` fixo de tempo de compilação é o preço de uma lista sem serialização? Se a sua frase aterrissar em "porque uma lista da qual você faz cast direto dos bytes precisa saber o tamanho dela antes do primeiro byte ser escrito, então você troca crescimento dinâmico por zero serialização e faz o despejo você mesmo," você tem. Se não aterrissar, releia a seção do tradeoff, porque toda decisão em forma de lista que você tomar neste framework daqui para frente passa por ela.

Você ganhou um checkpoint de verdade aqui. O R1 saiu de um número só para um leaderboard limitado, ordenado e no lugar que nunca desserializa, e ele anuncia as mudanças dele com um evento zero-copy que você consegue ler dos logs. Essa é uma estrutura de dados on-chain genuinamente não trivial, e você construiu a metade difícil dela sozinho. Existe um motivo para o V2 ter tido que comprovar que essas primitivas valiam o compute que consomem: a issue #4355 enquadrou os benchmarks do V2 contra o Quasar e o Pinocchio antes da conferência Accelerate no começo de maio de 2026, como tarefa de caminho crítico, precisamente porque listas e eventos sem serialização têm que merecer os números de CU deles ou a tese inteira do zero-copy é só conversa. Você acabou de rodar a tese.

Listas Slab são rápidas porque tudo nelas tem tamanho fixo. Mas alguns dados são genuinamente de comprimento variável: um rótulo de cabinet de forma livre, um conjunto ilimitado de algo que você não consegue limitar em tempo de compilação. Quando a rigidez do Pod para de se encaixar na forma dos seus dados, o V2 te entrega uma saída de emergência, e ela te custa o cast de bytes para ter flexibilidade de volta. A seguir, em m02-l3, essa saída: borsh, `BorshAccount<T>`, e exatamente quando recorrer a ela é a escolha certa em vez de uma falha de coragem.
