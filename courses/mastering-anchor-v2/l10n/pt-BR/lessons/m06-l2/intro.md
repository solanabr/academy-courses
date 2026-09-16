# O loop medido: recompre CU uma mudança por vez

Na lição passada você apontou quatro instrumentos para o swap: uma asserção de CU do Mollusk, um flamegraph de `--profile`, o anchor debugger, e um relatório de cobertura. Você saiu de lá com um número de CU de linha de base para a instrução `swap_arcade_for_tickets` e o nome do frame mais quente no flamegraph. Esse número é a coisa que você agora ataca.

Aqui está a tentação, e eu quero nomear ela antes de você sentir ela. Você tem um número e um frame gordo. A jogada óbvia é mudar cinco coisas de uma vez, re-rodar o teste, ver o número cair, e comemorar. Faça isso e você não aprendeu nada. Você não vai saber qual das cinco mudanças ajudou, qual atrapalhou, qual cancelou outra, e você vai entregar todas as cinco no escuro. Um número menor que você não consegue explicar não é uma otimização. É uma coincidência à qual você se apegou.

Então antes de qualquer teoria, faça a única coisa em que esta lição inteira se apoia: releia a sua linha de base. Não da memória, não da nota que você escreveu na lição passada. Leia ela fresca da máquina, agora mesmo, porque ela é o "antes" de toda medição que vem depois.

```bash
# Build FIRST, then read. Mollusk measures whatever .so SBF_OUT_DIR points it at,
# so a build with different flags is a different measurement wearing the same name.
cargo build-sbf
export SBF_OUT_DIR=$PWD/target/deploy   # a fresh shell needs it again; see m06-l1
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Anote o inteiro que ele imprime. Esse é o único número desta lição em que você tem permissão de confiar sem re-medir, e até ele você acabou de re-medir. Tudo daqui para frente é: mude exatamente uma coisa, rode isto de novo, e deixe a diferença entre os dois números ser o argumento inteiro.

## Resumo

Você não está escrevendo lógica nova de programa nesta lição. O swap mantém o comportamento dele. O que você ataca é o custo dele, com a mesma medição da lição passada rodada duas vezes, uma antes e uma depois de uma única edição — e a primeira alavanca que você puxar vai se recusar a mover o número, o que acaba ensinando mais do que uma vitória teria ensinado.

Aqui está a forma disso:

- **O método é um loop, não um saco de truques.** Linha de base (você tem ela) depois mude exatamente uma coisa depois re-meça a mesma instrução depois mantenha ou reverta com base no delta. A disciplina é a lição. As três alavancas abaixo, duas feature flags e uma refatoração, são só coisas para rodar através dele.
- **Alavanca um: `guardrails` desligado — e o zero que ensina.** O `guardrails` é um conjunto default-on de redes de proteção em runtime no Anchor V2, e virar ele para desligado *deveria* devolver CU e cerca de 300 bytes de binário. No R4 ele devolve exatamente nada, e o trabalho do lab é pegar esse zero e ler a razão direto do grafo de dependências: a unificação de features do cargo, com o `anchor-spl` como a aresta que vira a flag de volta. Uma mudança que nunca chegou no binário é o outro modo de falha do loop, e ela é tão mensurável quanto uma vitória.
- **Alavanca dois: `const-rent` ligado.** O `const-rent` dobra a constante de rent em tempo de compilação, economizando aproximadamente 85 a 90 CU por CPI de criação de conta. As letras miúdas dele, escritas no próprio Cargo.toml do Anchor, são que a constante dobrada fica obsoleta se a fórmula de rent mudar. Esse comentário cita a SIMD-0194, que é exatamente uma proposta para mudar a fórmula de rent.
- **O artefato é um teste de regressão.** Você leva o swap por um ciclo completo no lab, pega o zero, atribui ele, e codifica o orçamento medido num teste chamado `cu_swap_regression` para que no dia em que algo empurrar a troca para além dele, o build falhe.

O recuo desta lição: eu rodo um ciclo completo de ponta a ponta no lab, guardrails desligado, meço, e atribuo um delta de zero à aresta de dependência exata que engoliu a flag. Você depois roda o loop para uma segunda alavanca, o `const-rent`, com a bancada de teste entregue a você, e reporta o delta atribuído — um de verdade desta vez. O challenge de código é o degrau solo: você escreve a função de cotação otimizada do zero e faz ela passar.

## O loop medido

Uma pausa rápida para desmistificar primeiro, porque duas palavras desta lição soam mais pesadas do que são. Uma feature flag do Cargo é só compilação condicional. O `#[cfg(feature = "guardrails")]` fica na frente de um bloco de código, e se aquele bloco está no seu binário depende de a feature estar ligada quando você compilou. O `guardrails` e o `const-rent` são duas flags dessas que o Anchor V2 entrega. Ligar ou desligar uma não muda o seu código-fonte. Muda quais linhas o compilador mantém. É esse o mecanismo inteiro.

Agora o método, que é o conteúdo de verdade.

Pense em como você estabeleceria que uma única mudança no design de uma ponte deixou ela mais leve. Você não trocaria o aço, o tabuleiro, e os cabos tudo de uma vez e depois pesaria ela. Você mudaria um elemento, pesaria, e mudaria de volta se ficasse mais pesado. A razão é que um delta só tem significado causal quando exatamente uma variável se moveu. Mude duas e o número que você recebe é uma soma que você não consegue decompor. Esta é a ideia mais antiga do método experimental, e ela é exatamente tão verdadeira para unidades de compute quanto é para qualquer coisa que você consiga pesar.

Então o loop tem quatro passos, e o passo dois é estrutural:

1. **Meça** a instrução. Você tem isto: o `trade_cu_baseline` imprimiu um número.
2. **Mude exatamente uma coisa.** Uma feature flag, ou uma refatoração. Não duas.
3. **Re-meça** a mesma instrução, do mesmo jeito, com a mesma fixture.
4. **Mantenha ou reverta** com base no delta, e anote qual mudança única causou ele.

![Um loop cíclico de medir, mude exatamente uma coisa, re-medir, e manter-ou-reverter, com a regra de variável única sinalizada como o passo estrutural.](assets/v01-flowchart.webp)

Por que isto vale uma lição inteira em vez de uma frase? Porque o modo de falha é sedutor. Três mudanças simultâneas e uma queda total de CU parecem progresso. Mas esse total poderia facilmente esconder uma regressão: uma mudança economizou 400 CU, outra custou 200, uma terceira não fez nada, e você entregou a regressão de 200 CU porque a soma ainda caiu. Você carregaria ela para sempre, invisível, porque você nunca isolou ela. O loop é a coisa que faz uma vitória ser real. O delta é a evidência, e evidência exige um experimento controlado.

Com o método na mão, aqui estão três alavancas concretas para rodar através dele. As duas primeiras são feature flags, deliberadamente diferentes em caráter: uma tira algo para fora, uma dobra algo para dentro, e os trade-offs delas apontam em direções opostas. A terceira não é uma flag de jeito nenhum, o que é o ponto de incluir ela.

### Alavanca um: desligar o guardrails

O `guardrails` é uma feature default-on. Esse "default-on" importa: quer dizer que a sua linha de base da lição passada já foi medida com as redes de proteção no lugar. Elas são checagens de runtime que o framework insere por você, e elas custam CU em cada rodada porque elas executam em cada rodada.

Compile com o `guardrails` desligado, num crate onde a virada de fato aterrissa, e duas coisas acontecem. O binário encolhe (a figura do próprio Anchor é aproximadamente 300 bytes, medida no programa de benchmark dele), e a instrução fica mais barata, porque aquelas checagens não estão mais rodando dentro dela. Segure essa expectativa com cuidado, porque o lab está a ponto de violar ela: no R4 a virada aterrissa em nada, o medidor não pisca, e a razão é uma aresta de dependência que você vai ler com os seus próprios olhos.

Aqui está a parte que eu não vou deixar você pular, porque as leituras erradas dela são as tentadoras. O `guardrails` não é um lint, um mimo de tempo de compilação, nem uma configuração do Mollusk. Ele muda o programa compilado — *quando ele de fato desliga*. O que você estaria trocando fora são as checagens em si, redes de proteção de runtime de verdade, com a CU voltando porque o trabalho saiu. E a precondição que o lab existe para gravar a ferro: uma feature do cargo só está desligada quando *nada em lugar nenhum do seu grafo* liga ela de volta.

Então o que são as redes, concretamente? Isto importa, porque você não consegue argumentar um invariante que você não consegue nomear. A família guardrails é a classe de checagens defensivas que o framework insere em volta do seu handler para que uma chamada malformada falhe de forma limpa em vez de fazer algo pior. Pense nas garantias em que você vem se apoiando sem escrever: que uma conta entregue a você é de fato de propriedade do programa que você pensa que é dono dela, que um discriminator casa com o tipo de conta com o qual você desserializou ela, que um caminho aritmético que poderia dar wrap é pego em vez de truncar em silêncio, que um limite que você assumiu que vale de fato valeu. Em cada chamada única, aquelas checagens rodam, e cada uma delas debita um pouco de CU. É essa a forma da vitória quando você desliga elas, e é também a forma exata do risco. Você não tornou a chamada malformada impossível. Você fez o framework parar de checar por ela.

Então o veredito honesto sobre guardrails-off é: ele é defensável só para código cujos invariantes você consegue argumentar você mesmo. Se você consegue olhar o handler `swap_arcade_for_tickets` e dizer, em voz alta e corretamente, "as reservas são sempre contas distintas, a escala de taxa é fixa, a saída é limitada por `reserve_out`, e nada aqui pode dar underflow porque a cotação retorna 0 num pool vazio", então você fez o argumento que o framework estava fazendo por você, e você pode tirar as redes. Se você não consegue fazer esse argumento, deixe elas ligadas. Entregar guardrails-off sem nenhum argumento de invariante escrito é uma aposta que você não sabia que estava fazendo.

![Uma comparação de guardrails-off, que tira as checagens do caminho quente em troca de CU e 300 bytes mas faz você ser dono dos invariantes, contra const-rent-on, que dobra a constante de rent e pode ficar obsoleta.](assets/v02-comparison.webp)

### Alavanca dois: dobrar para dentro o const-rent

O `const-rent` vai para o outro lado. Em vez de remover uma checagem, ele remove uma computação, dobrando uma constante.

Cada vez que um programa cria uma conta através de uma CPI, ele tem que financiar aquela conta até o mínimo isento de aluguel, ou a conta não consegue sobreviver. Esse mínimo é uma função do tamanho em bytes da conta, e derivar ele quer dizer rodar a fórmula de rent: um custo por byte mais um overhead fixo, multiplicado para o tamanho que você está alocando. O runtime pode computar isso em cada criação de conta, ou, se o tamanho é conhecido em tempo de compilação, a resposta pode ser embutida como um literal. Esse literal embutido é o que o `const-rent` dobra para dentro, então o runtime pula a computação. A economia é aproximadamente 85 a 90 CU por CPI de criação de conta.

Note que eu escrevi uma faixa, não um número único, e eu vou ser teimoso quanto a isso. Esta é a mesma disciplina que fez este curso se recusar a congelar o próprio multiplicador de benchmark do Anchor na lição passada. A economia aparece como cerca de 85 no próprio comentário da feature no `Cargo.toml` do `anchor-lang` e cerca de 90 na entrada de changelog do V2 que introduziu ela, e além disso ela pode divergir, então congelar um dígito seria falsa precisão, não rigor. Vá ler as duas antes de citar qualquer uma; elas estão a quatro linhas de distância num repositório que você já tem clonado. A forma honesta é a faixa mais uma nota dizendo re-verifique. Um número de CU é determinístico para um dado programa, entrada, e toolchain, então esta faixa não é ruído de uma rodada para outra, e sim discordância de fontes mais risco de divergência, o que é uma coisa diferente e mais interessante.

E a divergência é a lição de verdade aqui. O `const-rent` dobra a constante de rent, o que só é correto enquanto a fórmula de rent que produziu ela ficar parada. O comentário no próprio Cargo.toml do Anchor diz exatamente isto, e ele cita a SIMD-0194 para dizer. A SIMD-0194 tem o título "Deprecate Rent Exemption Threshold." É uma proposta Core, Accepted, arquivada lá em novembro de 2024, que mudaria como o mínimo isento de aluguel é derivado. Se ela ativar, a constante que você dobrou para dentro fica errada, em silêncio, e a sua criação de conta agora está computando rent contra um número obsoleto.

![Uma linha do tempo a partir do arquivamento da SIMD-0194 em novembro de 2024, mostrando a constante dobrada do const-rent ficando válida só até a fórmula de rent mudar, o que faz da flag um item de re-verificação.](assets/v03-timeline.webp)

Fique um instante com o quão estranho isso é. Você buscou uma feature flag para raspar menos de cem unidades de compute de uma criação de conta, e as letras miúdas te devolveram uma questão viva de governança de protocolo. Uma edição de uma linha no Cargo colocou no seu build uma dependência do status de ativação de uma SIMD. Essa é genuinamente a coisa mais interessante sobre o `const-rent`, e é por isso que a faixa importa: você não está só citando uma economia, você está citando uma economia com uma data de validade que você não controla.

![Uma barra de faixa cobrindo aproximadamente 85 a 90 CU ao longo de duas citações de fonte, com uma extensão tracejada de divergência mostrando por que um dígito congelado seria falsa precisão.](assets/v04-chart.webp)

### Alavanca três: uma refatoração, não uma flag

As duas primeiras alavancas eram feature flags, uma edição numa linha de build. O loop não se importa de onde a mudança vem. Uma refatoração no nível do código-fonte roda através dele exatamente do mesmo jeito, e o swap tem uma sentada à vista de todos: a própria função de cotação.

Aqui é onde o outro entregável da lição passada vence. Você anotou o frame de código próprio mais quente embaixo da instrução de troca, e no swap de Quarters aquele frame é o `swap_out`, a cotação. Um flamegraph não te diz o que fazer sobre um frame gordo; ele te diz para onde apontar o loop. Então aponte ele para lá.

A cotação que você entregou, o `swap_out`, já promove para `u128` antes de multiplicar, porque o módulo 5 fez disso a barra de aceitação. O que ela não faz é expor a taxa, e tem uma questão de design de verdade escondida em como uma versão geral acaba escrita. Você busca o `checked_mul` no `u64` e faz branch no overflow, o que é seguro mas coloca um branch no caminho quente? Ou você promove para `u128` primeiro, onde a multiplicação não pode dar overflow por construção, então nenhuma checagem é necessária? Esses dois não são igualmente baratos, e eles não são igualmente seguros, e o único jeito de saber qual é a troca de verdade para as suas reservas é rodar os dois através do loop. ("Por construção" carrega um qualificador que você derivou no m05-l2 — dois fatores `u64`. O challenge no fim desta lição adiciona um terceiro.)

É isso que o challenge de código no fim desta lição é: uma cotação generalizada, o `get_amount_out`, com a taxa erguida para um parâmetro, escrita do zero e depois medida. É uma função separada do `swap_out` que você entregou, não uma edição nele, então você pode segurar as duas e comparar. Escreva ela, troque ela para dentro do handler atrás de uma mudança de chamada de uma linha, e re-meça a troca. Se o delta for uma vitória e a função ainda casar com as saídas de referência dela, mantenha ela. Se a sua versão de `checked_mul`-mais-branch chegou mais barata nas suas reservas, é para isso que o loop serve, e o número decide, não a sua intuição sobre qual lê mais rápido.

![Uma comparação lado a lado de uma multiplicação checada com um branch contra promover para u128, mostrando que as duas são seguras e só uma medição em reservas reais decide qual custa menos.](assets/v05-comparison.webp)

O ponto que generaliza além desta função só: uma alavanca é qualquer coisa que você consegue virar e re-medir em isolamento. Uma feature flag é o tipo mais limpo porque ela não move nada no seu código-fonte. Uma refatoração também é uma alavanca, desde que você faça uma e só uma, e depois meça. O método não muda. Só a coisa que você está mudando muda.

### O trade-off, dito sem enfeite

Cada CU que você compra tem um preço, e as alavancas precificam ele de formas diferentes.

Guardrails-off, onde ele aterrissa, devolve CU e cerca de 300 bytes ao preço das redes de proteção de runtime — nunca entregue ele sem um argumento de invariante que você consiga defender, porque o risco que você assume é corretude: você agora é a checagem. No R4 ele não aterrissa de jeito nenhum, e o risco se inverte em algo mais quieto: uma linha de build que *diz* nets-off enquanto o binário ainda carrega elas documenta uma otimização que nunca aconteceu, e ela fica ali esperando o dia em que o grafo mudar embaixo dela.

Const-rent troca uma computação de runtime por uma constante de tempo de compilação que pode divergir se a fórmula de rent mudar. O risco que você assumiu é obsolescência. Ele precisa de uma nota de re-verificação amarrada à SIMD-0194, não de um commit atire-e-esqueça.

E tem um terceiro trade-off que não tem nada a ver com nenhuma das duas flags: para onde você aponta o loop. Otimizar uma instrução fria é esforço desperdiçado. Umas 50 CU medidas raspadas de um caminho que ninguém acessa são ruído, não uma vitória, e pior, são ruído que te custou tempo real e muitas vezes comprou um risco real. Isto é diretamente relevante para as alavancas, porque elas miram em instruções diferentes. Guardrails-off mira no caminho quente da troca, o que roda em cada swap, milhares de vezes — onde ele chega a aterrissar. Const-rent ajuda CPIs de criação de conta, o que neste programa quer dizer o setup do pool, uma instrução que roda uma vez quando você levanta o pool e nunca mais durante a negociação.

As duas miras são legítimas, mas o peso não é o mesmo. Uma CU economizada no caminho da troca é economizada em cada troca para sempre, então ela compõe com o volume. Uma CU economizada no init de uma vez só é economizada exatamente uma vez. Isso não torna a otimização do init inútil, levantar uma conta mais barato continua sendo mais barato, mas isso quer dizer sim que você não deveria gastar uma tarde raspando o caminho frio enquanto o quente ainda tem um frame gordo que você não tocou. Ordene as alavancas por frequência vezes delta, não por delta sozinho. O instrumento que te diz a frequência não é o flamegraph, é o seu próprio conhecimento de como o programa é de fato chamado.

O que vale dizer sem enfeite contra o que esta lição depois pede que você faça, porque parece uma contradição. O completion abaixo roda o loop no `const-rent`, uma alavanca de caminho frio, enquanto o frame quente continua intocado até o challenge. Essa ordenação é pedagógica, não uma recomendação: o `const-rent` é a segunda volta mais limpa em torno do loop porque o trade-off dele é divergência em vez de corretude, então você consegue praticar o método sem também ter que defender um argumento de invariante — e, depois do zero do lab, é a primeira volta em que o número de fato se move. No seu próprio programa você faria o caminho quente primeiro. Aqui você está aprendendo o loop, e o loop é mais barato de aprender na alavanca que não consegue te machucar.

![Uma comparação de duas colunas pesando o caminho quente da troca contra o caminho frio do pool-init, mostrando que o mesmo delta de CU deveria ser ordenado por frequência vezes delta.](assets/v06-comparison.webp)

A outra metade de apontar o loop corretamente é isolamento, e é onde o método inteiro vive ou morre.

![Um diagrama de dois painéis contrastando uma mudança atribuível contra três mudanças simultâneas cujo total líquido esconde uma regressão que é entregue invisivelmente porque a soma ainda caiu.](assets/v07-diagram.webp)

## Lab: rode um ciclo completo no swap

Exemplo trabalhado, rodinhas no lugar. Eu rodo uma passada completa do loop contra a instrução `swap_arcade_for_tickets` usando o `guardrails` como alavanca, e a gente chega num orçamento medido codificado num teste de regressão. Aviso honesto sobre a forma do final: a resposta do loop aqui é um zero, e o zero — pego, atribuído, e nomeado — é o achado. Rode cada comando no seu próprio checkout enquanto você lê; os números que você recebe não vão ser os meus, e é esse o ponto.

O recuo da ajuda é explícito. Eu rodo o ciclo de guardrails de ponta a ponta aqui, delta e decisão ditos em voz alta. Você depois roda o loop de novo para o `const-rent` na instrução de pool-init, bancada de teste fornecida, na seção de completion abaixo. E o challenge de código depois dela, a cotação otimizada escrita do zero, é só seu.

### 1. Confirme que a linha de base ainda é a linha de base

Não confie no número nas suas notas. Releia ele, porque uma linha de base obsoleta envenena cada delta depois dela.

```bash
# The "before". Build the defaults configuration, then measure THAT build.
cargo build-sbf
ls -l target/deploy/token_ticket_swap.so     # write the byte size down too
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Anote o inteiro como `BEFORE`, e o tamanho em bytes do `.so` do lado dele. Os dois são com o `guardrails` ligado, porque esse é o default. A linha de build importa mais do que parece: o `cargo test` não refaz o build do artefato on-chain, ele só roda a bancada de teste contra qualquer `.so` que já esteja no disco, então cada medição desta lição é precedida pelo build cujo custo ela está reportando. Pule um build e você vai medir a configuração anterior e atribuir o delta à mudança errada.

### 2. Mude exatamente uma coisa: guardrails desligado

O jeito limpo de virar uma feature de framework é encaminhar ela através do `Cargo.toml` do seu próprio crate de programa, então o interruptor é uma flag no comando de build e nada no seu código-fonte se move. Ligue o repasse da feature uma vez:

```toml
# programs/token-ticket-swap/Cargo.toml
# RC tags move fast: check the crate's Cargo.toml for the exact feature names on the
# branch you pin, then re-verify. anchor-lang's DEFAULT features are `alloc` +
# `guardrails`; const-rent is opt-in.
[features]
default = ["anchor-lang/guardrails"]      # safety nets ship ON by default
const-rent = ["anchor-lang/const-rent"]   # opt in to the compile-time rent fold

[dependencies]
# Program deps come from crates.io at 2.0.0-rc.1 (published 2026-08-12, re-verified
# 2026-08-23), exactly as m02-l1 pinned them: a published version is immutable, and
# it is the same release the tag-pinned CLI was built from.
# default-features = false makes guardrails toggleable, but it also drops `alloc`,
# the OTHER v2 default. Re-enable alloc here explicitly: otherwise your
# --no-default-features build moves TWO variables (and your allocator), not one.
anchor-lang = { version = "2.0.0-rc.1", default-features = false, features = ["alloc"] }
# The SPL surface R4 has carried since module 5 — and, as this lab is about to
# measure, the row that quietly decides the whole guardrails story.
anchor-spl  = "2.0.0-rc.1"
# The pins from m01-l2 — every program crate in this course carries them (issue #4937's class).
wincode = { version = "0.5", features = ["derive"] }
# The arcade-workspace row, unchanged since m02-l1 and confirmed in m06-l1 when Mollusk
# arrived: this is that same crate. The ceiling below 2.7 is the constraint that matters;
# 2.6.1 is still on wincode 0.5. Every member of the workspace carries this same row.
solana-address = ">=2.6.1, <2.7"

[dev-dependencies]
# Unchanged from m06-l1, and listed here so the whole crate is on one page: Mollusk,
# its token-program companion, plus the two rows that hold its graph on wincode 0.5.
mollusk-svm = "0.15.1"
mollusk-svm-programs-token = "0.15.1"
solana-sdk = "4"
solana-short-vec = ">=3.2.2, <3.3"
solana-signature = ">=3.4.1, <3.5"
spl-token = "9"   # the fixture's state types and spl_token::ID (m05-l1's pin)
```

Agora a mudança única. Compile o programa com as default features desligadas, o que tira o `guardrails`:

```bash
# ONE change: build with guardrails off. Nothing else moves.
cargo build-sbf --no-default-features
```

Resultado esperado: um build limpo — e um tamanho em bytes que **não se moveu**. Rode o `ls -l` de novo e compare contra o passo 1: idêntico, byte por byte. Na maioria dos crates um tamanho que não se moveu significaria que o repasse da feature não está ligado e você está a ponto de medir nada. O seu repasse está ligado exatamente certo, e o tamanho que não se moveu ainda está te dizendo que a flag nunca chegou no binário — essa contradição é o achado de verdade deste lab, e o passo 3 roda a medição de todo jeito antes de explicar ela, porque a regra do loop é medir primeiro, explicar depois.

É isso. Você mudou uma coisa. Resista a adicionar `--features const-rent` na mesma linha, porque então você teria movido duas variáveis e o delta seria uma soma.

### 3. Re-meça a mesma instrução, do mesmo jeito

```bash
# The "after" number. Identical command, identical fixture.
cargo test -p token-ticket-swap trade_cu_baseline -- --nocapture
```

Esse comando imprimiu o número — e o teste continuou **verde**. Leia a linha: `trade consumed <N> CU`, o mesmo inteiro que o `BEFORE`, e o `Check::compute_units(BEFORE)`, uma igualdade exata, ainda passa. O alarme da lição passada está funcionando exatamente como foi projetado: ele fixa o número, o número não se moveu, então nada dispara. Anote o `AFTER` de todo jeito, porque o loop exige, e compute o delta: `AFTER - BEFORE = 0`. Você mudou uma coisa e o medidor não piscou.

Um delta de zero tem exatamente duas leituras honestas, e o loop te força a escolher a certa. Ou a mudança genuinamente não custa nada — ou a mudança nunca chegou no binário. O tamanho em bytes que não se moveu do passo 2 já votou pela segunda. Agora leia o culpado direto, com o instrumento construído para exatamente esta pergunta:

```bash
cargo tree -e features -i anchor-lang --no-default-features
```

```text
anchor-lang v2.0.0-rc.1
├── anchor-lang feature "alloc"
│   └── token-ticket-swap v0.1.0 (…/programs/token-ticket-swap)
│   └── anchor-lang feature "default"
│       └── anchor-spl v2.0.0-rc.1
│           ├── anchor-spl feature "default"
│           │   └── token-ticket-swap v0.1.0 (…/programs/token-ticket-swap)
│           └── anchor-spl feature "guardrails"
│               └── anchor-spl feature "default" (*)
├── anchor-lang feature "default" (*)
└── anchor-lang feature "guardrails"
    └── anchor-lang feature "default" (*)
```

Leia a última estrofe de baixo para cima: o `anchor-lang feature "guardrails"` é ligado pelo `anchor-lang feature "default"`, e a flecha embaixo *daquele* aponta direto para o `anchor-spl v2.0.0-rc.1`. A sua linha `default-features = false` fez o trabalho dela — a sua própria aresta parou de pedir os defaults — mas o `anchor-spl` declara um `anchor-lang = "=2.0.0-rc.1"` simples, defaults e tudo, e o cargo **unifica features em cada aresta do grafo**: um crate é compilado para todo mundo, então qualquer aresta única que peça uma feature liga ela para todos eles. Features são aditivas por design; o seu `false` não consegue subtrair o que uma aresta irmã adiciona. Você virou a flag, e o grafo virou ela de volta antes de o compilador sequer ver.

Então a frase de atribuição para este ciclo, dita em voz alta e verdadeira: "desligar o guardrails não mudou nada, porque a aresta de dependência do anchor-spl mantém a feature ligada". Essa frase é o ponto inteiro do loop — um zero que você consegue explicar bate uma vitória que você não consegue.

### 4. Mantenha ou reverta, com o argumento dito

Aqui está a decisão, e o zero toma ela por você: **reverta**. Tire o `--no-default-features` de volta da linha de build. Não porque as redes tenham que ficar — porque a flag não faz nada aqui, e uma linha de build que alega nets-off enquanto o binário ainda carrega elas é pior do que qualquer um dos dois estados honestos. Ela documenta uma otimização que nunca aconteceu, e ela fica armada: no dia em que a aresta do anchor-spl mudar, a sua flag "no-op" silenciosamente se torna um build nets-off de verdade que ninguém nunca argumentou.

A disciplina de invariante que a troca nets-off exige não é desperdiçada, porém — ponha ela na prateleira no ponto onde o zero deixou ela. Num crate que não fica embaixo do `anchor-spl` — um programa de lógica pura com só o `anchor-lang` no grafo dele — essa virada exata aterrissa, o binário encolhe, e a regra se aplica por inteiro: nunca entregue nets-off sem um argumento escrito que defenda cada invariante que as checagens estavam cobrindo. Para este swap o parágrafo seria até escrevível: as duas reservas são contas de token distintas por construção, então não tem aliasing para pegar; a cotação retorna 0 num pool vazio ou de entrada zero e o handler reverte numa saída 0; a escala de taxa é uma constante fixa; e a saída é limitada por `reserve_out`, um `u64`, então o cast final não pode truncar. Escreva ele no dia em que a virada puder aterrissar. Hoje o grafo vetou a troca antes de você poder fazer ela.

![Uma tabela de decisão travando um build guardrails-off em um delta real, um argumento de invariante escrito por inteiro para o handler, e o argumento de fato commitado, caso contrário reverta.](assets/v08-comparison.webp)

### 5. Codifique o número medido como um teste de regressão

O ciclo do lab terminou numa reversão, e ele ainda deixa um artefato — este. Um número que você mediu uma vez é uma história; um número codificado num teste é um alarme, e o custo real, nets-on, da troca merece um. O passo de verify desta lição é um teste chamado `cu_swap_regression`, e o trabalho dele é falhar o build no dia em que algo empurrar a troca de volta para além do orçamento dela.

Ele também substitui o `trade_cu_baseline`, deliberadamente, e não porque alguma coisa está vermelha — nada está. Um pin de igualdade exata era a ferramenta certa para estabelecer uma linha de base uma vez; como teste permanente ele fica vermelho em cada melhoria além de cada regressão, o que treina as pessoas a ignorar ele. Delete o `tests/cu_baseline.rs` quando o teste novo estiver verde, e mantenha o módulo de fixture que ele usava, porque este também precisa dele.

```rust
// programs/token-ticket-swap/tests/cu_swap_regression.rs
use mollusk_svm::{result::Check, Mollusk};
use solana_sdk::{account::Account, instruction::Instruction, pubkey::Pubkey};

mod swap_fixture;   // the same module cu_baseline.rs used last lesson

// The same swap fixture: program, accounts, one swap ix.
fn fixture() -> (Mollusk, Instruction, Vec<(Pubkey, Account)>) {
    let program_id = token_ticket_swap::ID;
    let mut mollusk = Mollusk::new(&program_id, "token_ticket_swap");
    // The trade's CPI target, registered exactly as in m06-l1.
    mollusk_svm_programs_token::token::add_program(&mut mollusk);
    let keys = swap_fixture::keys(&program_id);
    let accounts = swap_fixture::build_swap_accounts(&keys);
    let ix = swap_fixture::build_swap_ix(&program_id, &keys);
    (mollusk, ix, accounts)
}

// The budget from this lab's measurement: your BEFORE (which is also your AFTER —
// that zero was the finding), plus a little headroom. This is not a wish. It is the
// number you measured, rounded up so a future toolchain bump that shifts the number
// a little does not flap the build.
// Set this before you run the test; the println below tells you what to set it to.
const TRADE_CU_BUDGET: u64 = 0; // <- set to your measured AFTER + a small headroom

#[test]
fn cu_swap_regression() {
    let (mollusk, ix, accounts) = fixture();

    // Assert the trade still succeeds, then assert it costs AT OR BELOW the budget.
    // Check::compute_units asserts EQUALITY, which is why the bound is a comparison
    // here instead: a change that gets cheaper should pass, not fail.
    let result = mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    // Print before you assert, same as the baseline test did, so a red run still
    // hands you the number you need to set the budget to.
    println!("trade consumed {} CU (budget {})", result.compute_units_consumed, TRADE_CU_BUDGET);

    assert!(
        result.compute_units_consumed <= TRADE_CU_BUDGET,
        "trade regressed: {} CU consumed, budget is {} CU",
        result.compute_units_consumed,
        TRADE_CU_BUDGET
    );
}
```

![Um painel anotado explicando as quatro linhas estruturais do teste de regressão: o orçamento conquistado mais a folga, a checagem de sucesso primeiro, uma comparação de igual-ou-abaixo, e os dois números impressos na falha.](assets/v09-annotated-code.webp)

Rode ele:

```bash
cargo test cu_swap_regression -- --nocapture
```

Com o `TRADE_CU_BUDGET` ainda em `0` ele falha e imprime o número para definir. Defina ele, re-rode, verde. Daí em diante, no dia em que uma refatoração empurrar a troca de volta para além daquele orçamento, este teste fica vermelho e te diz em qual direção ele se moveu. Um flamegraph é algo que você vai e olha. Este é algo que olha por você.

## Completion: rode o loop para o const-rent

Rodinhas meio tiradas. Mais uma volta em torno do loop antes do degrau solo, esta vez com o `const-rent` como alavanca e a bancada de teste entregue a você. Tem uma pegadinha que é ela mesma uma lição: o `const-rent` ajuda CPIs de criação de conta, e o `swap_arcade_for_tickets` não cria contas. Então não meça a troca para esta. Meça a instrução que levanta o pool, o init, porque essa é a instrução que a alavanca de fato toca. Medir a troca aqui te mostraria um delta de zero e te ensinaria a coisa errada.

O ciclo é idêntico em forma:

1. **Meça** a CU da instrução de pool-init. Essa é a instrução que você escreveu no lab de swap do módulo 5, a que dá `init` na conta `Pool` e cria as duas contas de token de reserva embaixo do PDA do pool; cada um daqueles `init`s é uma CPI de create-account do System Program, que é precisamente a chamada para a qual o `const-rent` dobra a constante.

   Não tem bancada de teste fornecida para ela, e escrever ela é o ponto deste degrau: você tem o padrão duas vezes agora. Copie o `cu_swap_regression.rs` para `tests/init_cu_baseline.rs` e mude quatro coisas. Renomeie a própria fn `#[test]` de `cu_swap_regression` para `init_cu_baseline` — o filtro posicional do cargo casa nomes de *função*, nunca nomes de arquivo, então sem essa renomeação os dois comandos abaixo filtram até `running 0 tests` e não imprimem número nenhum. Faça o build da instrução de init em vez da instrução de swap (o `swap_fixture::build_init_ix`, que pertence ao mesmo módulo de fixture que você colocou lá na lição passada). Passe a lista de contas do init, que a fixture expõe como `swap_fixture::build_init_accounts(&keys)`, e que difere da do swap porque o pool e as duas reservas têm que chegar *não inicializados*. E tire a asserção de orçamento por completo; mantenha só o `Check::success()` e o `println!`, porque para este degrau você quer o número impresso duas vezes, não fixado.

   ```bash
   cargo build-sbf                                                 # build first, always
   cargo test -p token-ticket-swap init_cu_baseline -- --nocapture
   ```
2. **Mude exatamente uma coisa** em relação ao seu build do passo 1: ligue o `const-rent` e não mova nada mais. O lab terminou com os defaults de volta ligados — a flag de guardrails era um no-op e você tirou ela da linha — então isto é uma flag em cima de um build default em todo o resto:

```bash
cargo build-sbf --features const-rent
```

Uma assimetria que vale notar enquanto você digita: a unificação vetou a *remoção* do guardrails, mas ela não consegue vetar esta *adição*. Features são aditivas, então LIGAR uma precisa só que a sua própria aresta peça por ela — o que é exatamente por que esta alavanca consegue aterrissar onde a última não conseguiu. A única variável que se move entre o passo 1 e o passo 3 é o `const-rent`.

3. **Re-meça** a mesma instrução de init, mesma fixture, mesmo comando. A linha de build acima já refez o build com a flag nova; rode o `cargo test -p token-ticket-swap init_cu_baseline -- --nocapture` de novo e leia o segundo número.
4. **Reporte** a CU do antes, a CU do depois, e a atribuição de uma linha: "ligar o const-rent economizou N CU na criação de conta do pool-init".

O seu delta deveria aterrissar na vizinhança da figura de 85-a-90-CU-por-CPI-de-criação-de-conta, escalada por quantas contas o init cria. Se ele voltar perto de zero, cheque se você mediu a instrução de init e não a troca. Essa é a forma mais comum de este completion dar errado, e é a mesma cilada de perseguir um caminho frio: você mediu a instrução que a alavanca não toca.

## Challenge: a cotação otimizada de produto constante

Rodinhas totalmente tiradas. Este é o degrau solo, e é o único em que você tanto escreve código quanto roda o loop nele.

Lá no módulo 5 você construiu o `swap_out`, a função de cotação com uma taxa de 0.3% hardcoded, e na lição passada você viu ela sentada ali como o frame de código próprio mais quente embaixo da instrução de swap. Agora generalize ela. O `get_amount_out` é a mesma curva com a taxa erguida para fora em um parâmetro, o `fee_bps`, em basis points. Aqui está o starter dela, e ele está errado de propósito:

```rust
/// Quote a constant-product swap output. THIS STARTER IS BROKEN:
/// it ignores fee_bps, multiplies in u64, and checks nothing, so its
/// quotes are wrong whenever fee_bps > 0, it can overflow on large
/// reserves, and it quotes the whole pool on an empty reserve_in.
const fn get_amount_out(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u64) -> u64 {
    let _ = fee_bps; // the fee is being ignored
    let numerator = reserve_out * amount_in;
    let denominator = reserve_in + amount_in;
    numerator / denominator
}
```

Uma palavra naquela assinatura é nova desde que o `swap_out` do módulo 5 foi entregue, e ela está fazendo trabalho de verdade: `const fn`. O arquivo avaliado carrega asserções de tempo de compilação embaixo da função — o dispositivo do m03-l3, e o que a avaliação só-de-compilação de fato impõe — então o starter quebrado não faz build: as linhas cegas para a taxa falham em asserções cujas mensagens nomeiam o caso, e a linha de pool profundo falha na própria avaliação de const com `attempt to multiply with overflow`, o rustc apontando para a multiplicação `u64` exata que você está aqui para corrigir.

Duas coisas estão erradas nele, e elas são as mesmas duas que o challenge do módulo 5 te fez corrigir no `swap_out`. Ele nunca aplica a taxa, então cada cotação com `fee_bps > 0` paga a mais para o trader. E ele multiplica dois valores `u64`, então um `reserve_out * amount_in` grande pode dar overflow. No scaffold em que você está de fato em pé isso é um panic de qualquer jeito — o workspace gerado pelo Anchor entrega `overflow-checks = true` no profile de release, e o `cargo build-sbf` faz build de release, um fato em que o m07-l2 se apoia — então o modo de falha aqui é um abort no pool mais profundo que você tem, que é a troca na qual você menos quer falhar. Tire aquela linha de profile, ou herde um crate que nunca teve ela, e a mesma multiplicação dá wrap em silêncio em vez disso, o que é pior. Você corrigiu as duas uma vez já numa curva sem taxa. Esta é a mesma reparação na geral — e no dia em que você estiver num crate onde um build nets-off genuinamente aterrissa, uma multiplicação que deu wrap não tem nada por trás dela, então a versão checada é a que você quer nos seus dedos agora.

Uma terceira coisa está errada nele que o módulo 5 já te ensinou e que este starter tira em silêncio: ele não checa as entradas dele. Ponha o `reserve_in` em zero e o denominador colapsa para `amount_in`, o `amount_in` cancela, e a função cota todo o `reserve_out` — o pool inteiro — para quem pedir primeiro. Essa é a mesma trava com que o `swap_out` abre, e a versão geral também precisa dela.

O seu trabalho: reescreva o `get_amount_out` para ele aplicar a taxa, recusar as entradas em que a curva não está definida, e rodar cada produto intermediário através de um único `u128` antes de uma divisão final de volta para `u64`. Mantenha a assinatura exatamente como congelada acima, porque os casos de referência abaixo chamam ela por aquela interface.

Três empurrõezinhos, a mesma forma que você já viu antes, generalizada para basis points:

- `amount_in_with_fee = amount_in * (10_000 - fee_bps)`
- `out = (reserve_out * amount_in_with_fee) / (reserve_in * 10_000 + amount_in_with_fee)`
- faça cast para `u128` antes das multiplicações, faça cast de volta para `u64` depois da divisão única

As saídas de referência com as quais a sua solução tem que casar:

```
get_amount_out(1_000_000, 1_000_000,  10_000, 30) == 9_871    // 0.3% fee, balanced pool
get_amount_out(5_000_000, 2_000_000, 100_000, 30) == 39_100   // 0.3% fee, uneven reserves
get_amount_out(1_000_000, 1_000_000,   1_000,  0) == 999      // zero fee = plain constant product
get_amount_out(1_000_000, 1_000_000, 500_000, 30) == 332_665  // large trade, missing fee is obvious

// ...and the inputs the curve is not defined on, which must degrade
// rather than panic or over-quote:
get_amount_out(0, 1_000_000, 10_000, 30)             == 0  // empty reserve_in: unguarded, quotes 1_000_000
get_amount_out(0, 0, 0, 30)                          == 0  // drained pool, zero quote: unguarded, 0 / 0
get_amount_out(1_000_000, 1_000_000, 10_000, 10_001) == 0  // fee past the scale: 10_000 - fee_bps underflows
get_amount_out(1_000_000, 1_000_000, 10_000, 10_000) == 0  // 100% fee: here 0 is the arithmetic answer
get_amount_out(u64::MAX, u64::MAX, u64::MAX, 30)     == 0  // numerator exceeds u128, the check degrades it

// and one that is not degenerate at all: it fits u128 with room to spare
// but overflows a u64 multiply, so it is the case that forces the promotion
get_amount_out(1_000_000_000_000_000_000, 1_000_000_000_000_000_000, 1_000_000_000_000, 30) == 996_999_005_991
```

Esse último par é o que pede um instante, porque é onde a comparação acima paga pelo qualificador dela. Você já fez esta aritmética no m05-l2, na escala de taxa `× 997`: dois fatores `u64` cabem em `u128` com menos de um bit de sobra, e um terceiro fator gasta a lasca. Mesma forma aqui com `10_000 - fee_bps` como o terceiro — reservas `u64::MAX` colocam o numerador em torno de 3.4e42 contra um teto perto de 3.4e38. A linha de 1e18 é o contrapeso, com pico perto de 1.0e34 e limpando aquele teto por quatro ordens de magnitude e meia. Espaço enorme, então, e ainda não uma garantia, que é por que os produtos ficam checados e a checagem degrada para 0 em vez de dar panic no meio da instrução.

Agora o limite que a assinatura congelada não consegue expressar: o `10_000 - fee_bps` dá underflow para qualquer `fee_bps` acima de 10,000, e o tipo de retorno é um `u64` simples sem lugar nenhum para colocar um erro. Trave ele de todo jeito e retorne 0, do jeito que você trava a reserva vazia — um 0 errado-mas-quieto bate uma instrução que dá panic, e num crate cujo profile de release de fato tem overflow-checks desligado — não o deste scaffold, que fixa eles ligados — o underflow dá wrap no intermediário `u128` para um número enorme em vez disso, o que cota um pagamento que drena o pool. Mas seja claro com você mesmo sobre o que aquele 0 quer dizer. Para `fee_bps == 10_000` ele é aritmética: uma taxa de 100% come a entrada inteira. Para todo o resto ele é um sentinela fazendo as vezes de um erro que a assinatura não consegue retornar, e AMMs de verdade não degradam assim — o `getAmountOut` do Uniswap V2 reverte com INSUFFICIENT_LIQUIDITY numa reserva vazia. Então quem chama continua sendo dono do limite. No handler, o `fee_bps` é uma constante de tempo de compilação que você controla; no momento em que ele se torna um parâmetro que um usuário pode definir, ele precisa de um `require!` antes de chegar nesta função, e o handler de qualquer jeito se recusa a concluir uma troca que cota 0. Escreva esse raciocínio como um comentário acima da fn para que o próximo leitor saiba que foi uma decisão e não um acidente.

Depois meça ela, porque uma cotação que você não rodou através do loop é só uma reescrita. Troque o `get_amount_out(reserve_in, reserve_out, amount_in, 30)` para dentro do handler no lugar do `swap_out`, refaça o build, e re-rode o `cu_swap_regression`. Reporte o delta do mesmo jeito que você reportou os outros dois: um número, uma mudança nomeada, uma frase.

A barra de aceitação: a taxa é aplicada antes de cotar, as entradas são travadas antes de qualquer coisa ser computada, cada produto intermediário é computado num `u128` checado para que reservas grandes não consigam dar overflow, existe exatamente uma divisão no caminho quente, cada saída de referência casa — as degeneradas incluídas — e você consegue declarar o delta de CU medido contra o `swap_out`. O starter já cumpre exatamente um item daquela lista — ele faz uma divisão única — e falha em todos os outros; a sua solução passa em todos. Compute o primeiro caso à mão, chegue em 9,871, depois faça o código concordar com a sua aritmética.

## Antes de seguir em frente

Quatro formas de esta lição dar errado na prática. Se cheque contra cada uma.

Você mudou exatamente uma coisa entre cada par de medições? Se você virou duas flags numa linha de build, o seu delta é uma soma que você não consegue atribuir, e a resposta honesta é voltar e isolar elas.

Você atribuiu o zero em vez de dar de ombros para ele? Um delta de zero tem duas leituras, e só uma delas é "esta mudança é grátis": a sua era "a mudança nunca chegou no binário", e o `cargo tree -e features` nomeou a aresta do anchor-spl que engoliu ela. Carregue o corolário também: no dia em que você estiver num crate onde a virada aterrissa, o argumento de invariante vence por inteiro — um build nets-off é só tão seguro quanto o parágrafo que você escreveu do lado dele, e nenhum parágrafo quer dizer reverter.

Você citou a economia do const-rent como uma faixa com uma nota de re-verificação, e não um dígito único congelado? A economia é aproximadamente 85 a 90 CU, ela abrange duas citações de fonte, e ela pode divergir se a fórmula de rent mudar. A SIMD-0194 é a razão de o número carregar uma data de validade que você não controla.

E você mediu cada alavanca na instrução que ela de fato toca? Guardrails-off estava mirado no caminho quente da troca — e o grafo vetou ele antes de ele chegar; const-rent aterrissa no init de criação de conta. Uma vitória de 50 CU num caminho que ninguém acessa é ruído. Pese a vitória por quão frequentemente a instrução roda.

Você rodou o loop de verdade e você consegue comprovar cada resultado que ele produziu: um zero atribuído a uma aresta de dependência nomeada, uma economia no init atribuída a uma flag, uma cotação reescrita medida pelos próprios méritos dela, e um teste de regressão segurando a linha da troca por você. O próximo módulo faz uma pergunta mais difícil do que "é rápido?" Ele pergunta "é seguro?" Você vai escrever os exploits clássicos do Anchor contra o seu próprio programa, as substituições de conta e as checagens de signer faltando, e ver quais o V2 simplesmente se recusa a compilar e quais sobrevivem a todo framework e continuam sendo suas para defender. O loop medido te ensinou a comprar CU uma mudança por vez. O módulo de segurança te ensina o que você nunca deve trocar por isso.

Vejo você no módulo de segurança.
