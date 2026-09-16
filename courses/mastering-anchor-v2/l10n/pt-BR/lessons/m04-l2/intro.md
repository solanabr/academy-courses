# A cilada do reload acabou

Na lição passada o vault aprendeu a pagar. Você reconstruiu as signer seeds dele a partir das seeds mais o bump canônico armazenado, chamou `invoke_signed` através de `CpiContext::new(...).with_signer(...)`, e moveu lamports de verdade para fora de um PDA sem chave sob autoridade do programa. O vault assina por si mesmo agora. Lamports saem dele por ordem do programa, não pelo keypair de um humano.

Existe um bug que mora exatamente uma linha depois daquele withdrawal, e na linha mais antiga do Anchor ele era entregue o tempo todo. A forma é esta: você faz uma CPI, depois lê a conta que você acabou de mudar, e age sobre o valor antigo. Compilava. Rodava. Passava no teste do caminho feliz. E aí, em produção, tomava uma decisão sobre um número que já estava errado. Todo mundo que escreveu Anchor para viver bateu nisso pelo menos uma vez. No V2 esse bug acabou, e o jeito como ele acabou é mais estranho e melhor que o boato que você talvez tenha ouvido.

Então vamos procurar a parede. Abra o programa R2 da lição passada e ache o handler `withdraw`. Logo depois de você construir o valor `cpi`, antes de `transfer` consumir ele, acrescente uma linha:

```rust
let before = ctx.accounts.state.credit;
```

Agora rode `anchor build`. Se o que você ouviu sobre o modelo de borrow do V2 é "você não pode tocar `ctx.accounts` enquanto uma CPI está em voo", você espera um erro de compilação aqui. Você não vai ter um. O build volta verde, e ele *deveria* voltar: `state` não é uma das contas que esta CPI toma, então nada na transferência pode invalidar ele, e o compilador sabe disso. Onde a parede fica de verdade — uma conta adiante — é o ponto inteiro desta lição. Leia a próxima seção antes de tocar em qualquer outra coisa, porque o que o V2 permite aqui é tão deliberado quanto o que ele proíbe.

## Resumo

Um conceito, e ele é um porquê, não um como: ler os dados tipados de uma conta enquanto um `CpiHandle` vivo faz borrow mutável dessa mesma conta é um erro de compilação no Anchor V2, e essa regra — que cai exatamente sobre as contas que uma CPI poderia mudar e sobre nada mais — aposenta de vez a cilada do `.reload()`-depois-da-CPI do v1. O bug não é suavizado nem domado por lint; ele simplesmente não pode ser escrito.

A gente vai reconstruir por que os autores do framework escolheram impor isso com o borrow checker em vez de um aviso, percorrer as alternativas ingênuas e ver cada uma falhar, e cravar a borda exata da garantia, porque uma promessa de segurança que você julga errado é pior que nenhuma. O perigo do v1 era silencioso e rodava; o substituto do V2 é barulhento e para o build. Mover uma falha de tempo-de-execução-e-quieta para tempo-de-compilação-e-óbvia é a tese inteira do V2, e esta é essa tese aplicada ao momento em que um programa chama outro.

A trava que você está mirando é pequena e afiada. Dado um trecho que lê os dados tipados de uma conta enquanto o `CpiHandle` dessa própria conta está vivo e que por isso se recusa a compilar, você reordena a leitura para depois de o handle ser solto, para que o programa faça build, e você nomeia em uma frase a classe de bug que o V2 eliminou. É isso.

A ajuda recua do jeito de sempre. No Lab eu te mostro a forma que falha e a correção inteira, porque eu quero o erro do borrow checker nos seus olhos e a reordenação nos seus dedos. No Challenge você recebe um trecho quebrado diferente, sem apoio, e você conserta e nomeia a classe você mesmo. O degrau avaliado depois dele roda a mesma disciplina com o framework arrancado e um avaliador olhando: um recibo que tem que estar certo, não só compilar. Leia, quebre, conserte, nomeie.

## Por que o compilador agora te cobre

### A cilada do v1, com precisão

Comece por como isso funcionava antes, porque você não consegue valorizar a correção até ter sentido a ferida. Na linha 1.0, `CpiContext::new` tomava o programa como um `Pubkey` simples por valor (e na linha 0.x antes dela, um `AccountInfo`), e as contas que você entregava para ele eram structs desserializadas comuns. O Anchor lia os bytes da conta da cadeia uma vez, no topo da sua instrução, e te entregava uma cópia tipada para trabalhar.

Essa cópia é o problema. Quando você dispara uma CPI que muta uma conta, a mudança aterrissa nos bytes reais da conta on-chain. A sua cópia desserializada, a struct parada na memória da sua instrução, não se mexe. Ela continua segurando o que quer que segurasse quando o Anchor leu pela primeira vez. Então a sequência clássica parecia inocente e mentia:

```rust
// Anchor v1. This compiles, runs, and is wrong.
token::transfer(cpi_ctx, amount)?;              // the vault's on-chain amount drops
let remaining = ctx.accounts.vault.amount;      // reads the STALE pre-transfer copy
require!(remaining >= floor, VaultError::TooLow); // decides on a number that is already false
```

O saldo real do vault caiu. O seu `remaining` não. Aí você travou um pagamento, ou um mint, ou uma liquidação em cima de um valor que a cadeia já tinha invalidado. A correção que o v1 oferecia era um método que você tinha que lembrar de chamar: `.reload()`, que relia os bytes da conta e atualizava a sua cópia.

```rust
token::transfer(cpi_ctx, amount)?;
ctx.accounts.vault.reload()?;                   // re-read the live bytes; NOW the copy is fresh
let remaining = ctx.accounts.vault.amount;
```

Uma linha. Barata. E catastrófica de omitir, porque omitir ela não produzia erro nenhum, aviso nenhum, panic nenhum. Só um programa que lia o passado em silêncio.

![O Anchor desserializa o vault em 100, uma CPI de transfer derruba o saldo real para 40, e só o .reload() atualiza a cópia obsoleta antes de o código decidir.](assets/v01-flowchart.webp)

Fique um instante com por que isso era tão perigoso. Não era que a correção fosse difícil. Era que a falha era invisível. Um `checked_sub` faltando dá panic e você vê nos logs. Um `.reload()` faltando dá certo, e a única testemunha é um valor sutilmente errado num caminho de código que um teste raramente exercita. Silencioso-e-errado é estritamente pior que barulhento-e-quebrado, porque barulhento é corrigido na terça e silencioso é corrigido depois de um incidente.

Antes de a gente pesar a mão no v1, porém, conceda a ele a defesa honesta, porque o design não era burro, era uma troca razoável que envelheceu mal. Desserializar uma conta uma vez, no topo da instrução, e reusar essa cópia é genuinamente mais rápido que reler os bytes toda vez que você toca um campo. Para uma instrução que nunca dispara uma CPI, e muitas não disparam, a cópia está sempre correta e é sempre barata. O v1 otimizou para o caso comum e deixou uma ponta afiada no incomum. Essa é uma escolha defensável até exatamente o ponto em que o caso incomum é "mover dinheiro", e aí a ponta está exatamente onde você não pode pagar por ela.

E repare na forma da falha, porque ela é pior que "às vezes errado". Separe o caso médio do pior caso, do jeito que você deve fazer com qualquer perigo. No caso médio, a conta que você lê depois de uma CPI não foi de fato mudada por aquela CPI, então a cópia obsoleta por acaso é igual ao valor vivo e o seu código está acidentalmente correto. Essa é a armadilha: o bug testa limpo, demonstra limpo, e fica dormente por meses. O pior caso é aquele caminho específico em que a CPI de fato moveu o número que você então lê, e esse caminho costuma ser o de maior risco, um pagamento dimensionado contra um saldo, um mint travado num supply. Um mecanismo que está certo nos caminhos chatos e errado no único caminho que importa não é um mecanismo que você quer vigiando um vault. É uma mina com boas chances.

### A pergunta que força o design

Então aqui está a pergunta que os autores do V2 de fato tiveram que responder. Como você faz a leitura obsoleta pós-CPI não apenas desencorajada, não apenas documentada, mas *impossível de escrever*? Não "a documentação te manda chamar `.reload()`". Impossível. O programador não deveria conseguir expressar o bug nem se quisesse.

Essa é uma barra mais alta do que parece, e os jeitos óbvios de passar por ela falham todos. Veja eles falharem, porque descartar eles é o que faz a resposta de verdade parecer inevitável em vez de arbitrária.

![Quatro jeitos de pegar um bug de obsolescência pós-CPI, em que a documentação nunca pega, um lint é silenciável, o auto-refresh custa trabalho em tempo de execução, e a regra de borrow pega em tempo de compilação.](assets/v02-comparison.webp)

### As correções ingênuas, descartadas em níveis

A primeira correção ingênua é a que o v1 já tentou: escrever na documentação. "Lembre de chamar `.reload()` depois de uma CPI que muta uma conta que você lê." Isso falha no contato com a memória humana. Não é um mecanismo, é uma esperança, e é precisamente a esperança que falhou por uma década. Uma regra imposta pela lembrança é uma regra que é quebrada na semana em que você está cansado.

A segunda correção ingênua é um lint. Entregue uma regra do clippy que sinaliza uma leitura depois de uma CPI. Melhor, porque uma máquina checa isso. Mas um lint é consultivo por construção. Você pode dar `#[allow]` nele, você pode não rodar o clippy no CI, e pior, um lint que tenta rastrear "esta conta é a que a CPI mutou, através deste alias, atravessando esta função auxiliar" é exatamente o tipo de fluxo de dados de programa inteiro em que lints são ruins. Ele vai perder os casos interessantes e gritar lobo nos chatos. Uma garantia que você pode silenciar não é uma garantia.

A terceira correção ingênua é a tentadora: fazer o framework atualizar a conta para você. Depois de toda CPI, re-desserializar em silêncio toda conta que você segura, para que a sua cópia nunca fique obsoleta. Correto, e nunca esquece. Mas agora meça o custo contra a linha de base realista, porque "comparado a quê" é o único jeito honesto de julgar isso. A desserialização adiantada de `Account<T>` já era a maior despesa isolada de compute do framework. O manifesto do V2 que deu a largada nesse redesign inteiro, a issue #4390, "Zero-copy account deserialization by default," nomeou exatamente isso: `Account<T>` era o caminho lento, e a reclamação de performance número um dos desenvolvedores do Anchor. Atualizar automaticamente em toda CPI pegaria a coisa mais cara que o framework antigo fazia e faria ela *com mais frequência*, em toda conta, em toda chamada, quer você leia aquilo de novo ou não. Você compraria segurança com exatamente o imposto que o V2 foi construído para abolir. E ainda assim não cobriria leituras de bytes crus, então nem completo ele é.

Então o nível ingênuo desaba, e o requisito se afia em algo mais estreito e mais estranho. A gente não quer atualizar os dados, e não quer avisar sobre a leitura. A gente quer fazer com que você não consiga *segurar* uma leitura de uma conta enquanto uma CPI viva segura o direito de mudar aquela conta. Se essas duas coisas são mutuamente exclusivas, a linha de leitura obsoleta simplesmente não pode ser digitada. Isso não é uma checagem em tempo de execução e não é um lint. Isso é o borrow checker.

### O mecanismo: um CpiHandle é um borrow

A ideia sobre a qual o mecanismo inteiro se apoia: no V2 você não entrega mais um clone de `AccountInfo` para uma CPI. Você entrega um `CpiHandle`, que você pega com `.cpi_handle()` ou `.cpi_handle_mut()`. E um `CpiHandle` não é cópia de nada. É um borrow vivo do Rust sobre a conta, mantido por todo o tempo em que o handle está em escopo.

Essa única escolha de design faz todo o trabalho. Enquanto um `CpiHandle` está vivo, a conta para a qual ele aponta está em borrow, então o borrow checker não vai deixar você formar um segundo borrow conflitante para ler dados tipados. A leitura e o handle não podem coexistir.

![Um painel de código mostrando que dois handles mutáveis em dois campos disjuntos são legais, e que o CpiContext mantém vivos exatamente esses dois borrows por campo até a chamada da CPI consumir ele.](assets/v03-annotated-code.webp)

Seja exato sobre *o que* está em borrow, porque a exatidão é o design. `cpi_handle_mut()` faz borrow de um único campo: `ctx.accounts.sol_vault` e `ctx.accounts.authority` são caminhos disjuntos, e o Rust sempre permitiu dois borrows mutáveis de dois campos diferentes de uma struct. E não existe mágica de framework embaixo dessa frase — `Context` declara `accounts` como um campo simples e a sua struct de accounts é uma struct simples, então as regras comuns de borrow de campo são a história inteira. É por isso que o par dentro do literal `Transfer { from, to }` está de bom tamanho. E mover os dois handles para dentro de um `CpiContext` não alarga nada: o context agora carrega esses dois borrows — de `sol_vault` e de `authority`, e de nada mais — e mantém eles vivos até a chamada da CPI consumir ele. Do momento em que `cpi` existe até o momento em que `transfer(cpi, ...)` come ele, essas duas contas estão travadas e todo outro campo de `ctx.accounts` está tão legível quanto sempre esteve. É isso que o build verde no topo desta lição estava te dizendo: `state` nunca entrou na CPI, então ler `state.credit` não conflita com nada. Chame isso de exclusão do borrow checker, e enuncie com precisão: um handle vivo *exclui* acesso conflitante à conta da qual ele faz borrow, por exatamente o tempo em que ele vive — um handle mutável exclui as suas leituras, e qualquer handle exclui as suas escritas.

Se ajudar, pense em cada handle como alguém tirando um livro específico de uma biblioteca. Enquanto um livro está com a CPI, ninguém mais consegue ler aquele livro — mas o resto da biblioteca continua aberto — e no momento em que o livro volta, o que você pega é a edição atual, não uma fotocópia que você tirou semana passada. A analogia carrega a parte importante, que ficar com o livro é exclusivo e tem prazo, e ela quebra em um lugar que vale sinalizar: um livro de biblioteca é um objeto físico, enquanto o borrow aqui é imposto inteiramente em tempo de compilação, antes de uma única instrução rodar. Nada é travado em tempo de execução. O compilador simplesmente se recusa a emitir um programa em que os dois se sobrepõem, então o "conflito" nunca é uma corrida, é uma falha de build. Fique com a exclusividade da analogia e descarte a fisicalidade.

![Uma linha do tempo de uma instrução em que os dados tipados da conta que foi para a CPI são legíveis, depois excluídos pelo span em que o CpiHandle mutável dela está vivo, depois legíveis de novo assim que o handle é solto — enquanto contas fora da CPI continuam legíveis o tempo todo.](assets/v04-diagram.webp)

Agora olhe para a lição passada com olhos novos. Lembra da primeiríssima coisa que o handler de withdraw fazia? Ele copiava valores *para fora* de `ctx.accounts` antes de construir qualquer handle:

```rust
let owner = *ctx.accounts.authority.address();  // copied out FIRST — the `*` makes it a copy
let sol_bump = ctx.accounts.state.sol_bump;     // a plain u8 copy
```

Eu te disse na hora apenas que a ordenação era estrutural e não estilística, e prometi o motivo para esta lição. Aqui está ele, e é mais estreito e mais afiado que "`ctx.accounts` está fora de alcance". Olhe para a primeira linha. `.address()` não te entrega um valor possuído; ele retorna `&Address`, um borrow de `ctx.accounts.authority`. O `*` desreferencia isso para uma cópia simples na sua stack, e o borrow acaba no ponto e vírgula. Tire o `*` e `owner` continua uma referência para dentro de `ctx.accounts.authority` — que o seu array de signer seeds então carrega até a CPI. No momento em que `cpi_handle_mut()` pede o borrow mutável dele daquele mesmo campo `authority`, o compilador recusa: E0502, borrow imutável ainda vivo, borrow mutável pedido, mesma conta. A cópia para fora é estrutural porque `authority` é uma das contas que esta CPI toma. A segunda linha, em contraste, é convenção e não lei: `state` nunca entra nesta CPI, então o compilador aceitaria aquela leitura em qualquer lugar do handler. Manter ela no topo continua sendo o hábito certo — capturas primeiro, handles no meio, leituras pós-CPI depois — mas saiba qual linha o compilador impõe e qual é estilo.

E repare no que isso te compra na outra ponta. No v1 você lia depois da CPI e recebia uma cópia obsoleta a menos que chamasse `.reload()`. No V2 não existe `.reload()`, porque não existe nada para recarregar.

Essa afirmação se apoia no módulo 2, então junte os dois fatos em vez de aceitar na fé. A leitura obsoleta do v1 existia porque `Account<T>` desserializava uma *cópia* no topo da instrução: o seu handler segurava uma struct possuída na stack, e uma CPI mutando os bytes on-chain não tinha como alcançar ela. O `Account<T>` do V2 é uma view zero-copy sobre o buffer vivo da conta, então `ctx.accounts.state.credit` não é um campo de uma cópia, é uma leitura por offset nos bytes que a CPI acabou de escrever. Não existe segunda cópia em lugar nenhum para ficar obsoleta. É por isso que `.reload()` pôde ser apagado em vez de substituído: o modelo de borrow não atualiza os seus dados, ele trava *quando* você tem permissão de ler, e o modelo de conta zero-copy é o que faz a leitura que você então tem permissão de fazer já vir fresca. Duas metades da mesma tese, e é aqui que elas se encontram.

### A trava é exata, e exata é o ponto

Um leitor afiado levanta uma objeção aqui, e ela é boa, então vamos responder em vez de desviar. Se o ponto inteiro é segurança, por que a trava cai *só* sobre as contas entregues à CPI? Quando o handle do vault de SOL está vivo você ainda consegue ler a conta `state` — o build verde no topo desta lição comprovou isso. O framework não deveria travar `ctx.accounts` inteiro, só por garantia? Uma trava de struct inteira soa como a escolha paranoica, e o paranoico costuma ganhar num caminho de custódia.

Não — e descobrir por quê afia o modelo inteiro. Pergunte o que a trava mais larga compraria de fato. Uma CPI só consegue tocar contas que você entregou a ela: o runtime entrega a lista de contas da instrução para o callee e nada mais, então uma conta fora do `CpiContext` não pode ser mudada por aquela chamada, o que quer dizer que uma leitura dela no meio da CPI não pode ficar obsoleta *por aquela chamada*. Travar `state` enquanto o handle do vault está vivo não preveniria um único bug; só forçaria você a contorcer código correto. A exclusão que o V2 de fato impõe cai precisamente sobre as contas cuja cópia tipada teria ficado obsoleta no v1 — as que a CPI pode escrever — e sobre nada mais. Ela até divide mais fino que isso: uma conta que a CPI só *lê* entra por um `cpi_handle()` compartilhado, e borrows compartilhados toleram as suas leituras, então o mint que você passa para uma transferência de token continua legível enquanto o vault ao lado dele está travado. A exclusão cai exatamente sobre as contas que a CPI pode escrever, e sobre nada mais.

E repare no que o V2 *não* precisou construir para conseguir essa precisão. Um framework que inventasse a própria análise de "quais contas esta CPI consegue alcançar" teria que seguir toda conta através de todo alias, todo auxiliar, todo branch — e um alias perdido seria um vazamento, uma leitura obsoleta liberada com um check verde. O V2 contorna o problema inteiro não inventando análise nenhuma. `Context` declara `accounts` como um campo simples; a sua struct de accounts é uma struct simples; um `CpiHandle` é um borrow simples de um campo. A "análise" é o próprio borrow checking por lugar do Rust, as mesmas regras que governam toda struct em todo programa Rust, endurecidas por uma década do ecossistema inteiro se apoiando nelas. O framework ganha exatidão *e* ausência de vazamento numa jogada só, arrumando os tipos dele para que a linguagem faça a imposição.

![A trava de borrow cai só sobre as contas dentro do CpiContext: o vault sob um handle mutável rejeita leituras, enquanto a conta state disjunta e o mint de handle compartilhado continuam legíveis.](assets/v05-comparison.webp)

Esse é um instinto recorrente do V2, então vale nomear como uma regra que você pode carregar: não construa na mão uma garantia que o sistema de tipos vai te dar de graça. Uma trava sob medida — grosseira ou esperta — é código de framework que alguém tem que acertar e manter certo para sempre. Um borrow é uma regra da linguagem que o compilador já acerta em todo build. Quando você projeta as suas próprias APIs a mesma jogada está disponível: modele os seus tipos para que o invariante caia das regras comuns de borrow, e o compilador faz a imposição.

### A borda exata da garantia

É aqui que uma explicação preguiçosa te diria que o compilador agora cuida da freshness das contas e te mandaria seguir o seu caminho. Não acredite nisso, porque é falso de um jeito que vai te morder. Conhecer a borda precisa de uma garantia de segurança faz parte de usar ela bem, então vamos traçar a linha com exatidão.

O compilador protege *o acesso tipado a conta*, nas contas das quais um handle vivo faz borrow. É só isso. Se você contorna a camada tipada e lê lamports crus de `AccountInfo` direto, ou puxa bytes do buffer de dados de uma conta na mão, o modelo de borrow não te protege. Essas leituras são suas para raciocinar, exatamente como eram no v1. Um colega que te diz que o borrow checker quer dizer que você nunca mais pensa em freshness está errado nos dois pontos: ele não atualiza nada, e não cobre leituras cruas.

![Uma leitura tipada de uma conta entregue à CPI é excluída enquanto o handle daquela conta está vivo, uma leitura tipada de uma conta fora da CPI não tem nada contra o que se proteger, e lamports crus de AccountInfo ou leituras manuais de bytes continuam responsabilidade do programador exatamente como no v1.](assets/v06-comparison.webp)

E nomeie o trade-off com honestidade, porque existe um. O modelo de borrow compra segurança em tempo de compilação por um pouco de flexibilidade. Um punhado de padrões ergonômicos do v1 — os que leem uma conta no meio da montagem da própria CPI que a recebe — agora precisa de reestruturação: você solta o handle, depois você lê. O custo é uma refatoração mecânica, em geral mover uma linha algumas linhas para baixo. É essa a conta inteira. Você troca "posso escrever o código em qualquer ordem" por "a ordem em que tenho permissão de escrever não pode ser a errada". Num caminho que move o dinheiro dos outros, essa é uma troca que eu faço todas as vezes. Seguro barato contra um bug silencioso é o melhor tipo que existe.

A dúvida que costuma vir depois é justa: e se eu genuinamente precisar de um valor no meio da CPI, alguma coisa que eu tenha que ler enquanto a chamada está sendo montada? Quase sempre, você não precisa, você só acha que precisa por hábito do v1. Se o valor vem de uma conta que a CPI toma, capture ele *antes* de construir o handle, exatamente como a lição passada capturou `owner` no topo — e capture como uma cópia simples, que é para isso que serve o `*` na frente de `.address()`. Uma cópia na sua stack não faz borrow de nada, então nenhum handle pode jamais conflitar com ela; uma referência segurada para dentro da conta é um borrow, e ela vai colidir com o handle daquela conta no momento em que um for construído. E se o que você quer é o estado da conta *depois* da CPI, isso não é uma leitura no meio da CPI de jeito nenhum, é uma leitura pós-CPI, e o lugar dela é abaixo da linha que solta o handle, onde ela vai estar fresca de qualquer jeito. O padrão que não tem resposta limpa, ler os dados tipados vivos de uma conta no instante preciso em que ela é comprometida com uma CPI, é o padrão que também não tinha resposta correta no v1. O V2 só parou de fingir que tinha.

### A tese, e o framework se cobrando dela

Afaste o zoom por um segundo, porque isto é uma visão de mundo, não um truque esperto. O manifesto #4390 argumentou que o `Account<T>` ser o caminho lento não era uma nota de rodapé de performance, era a falha central: a coisa segura e a coisa rápida tinham se afastado, então as pessoas pagavam um imposto por segurança e algumas delas pararam de pagar. A resposta do V2 foi fazer do caminho seguro o caminho rápido e do caminho rápido o default. O modelo de borrow é essa mesma tese levada um nível acima, para dentro da composição. Em vez de tornar barata uma leitura pós-CPI segura, ele torna impossível uma insegura. Mesmo instinto, alavanca diferente: transformar uma classe inteira de bug em um erro de compilação em vez de um lint ou uma linha na documentação.

![Uma linha do tempo indo do manifesto da issue 4390 passando pelo Anchor 1.0.0 até o modelo de borrow que remove o .reload() e o fuzzing que achou quatro bugs do framework.](assets/v07-timeline.webp)

Essa última batida vale mais que uma nota de rodapé. Um framework que te entrega garantias de tempo de compilação deveria ganhar elas ele mesmo, e este fez o trabalho. O changelog do V2 credita ao fuzzing a descoberta de quatro bugs de correção no código do próprio framework, rastreados em #4431, e a suíte de testes entrega testemunhas do Miri e configs do Kani. O Miri pega comportamento indefinido em código unsafe; o Kani comprova que propriedades valem em todas as entradas, não só nas que um autor de teste pensou. O framework se submete à mesma disciplina de "comprove, não torça" que ele agora impõe ao seu programa. Quando uma ferramenta te diz para confiar no sistema de tipos, esse é o recibo que você quer ver por trás dela.

Um último alargamento, porque o modelo de borrow é uma única instância de um padrão que você vai ver por todo o V2, e enxergar o padrão vale mais que memorizar esta regra. O fio condutor é uma preferência por mover falhas para mais cedo e mais alto. Uma leitura obsoleta que antes aparecia em tempo de execução, em silêncio, em um caminho, em produção, agora aparece em tempo de compilação, alto, em todo caminho, na sua máquina. Essa é a mesma jogada que substituir um `require!` de tempo de execução por um tipo que não consegue segurar um estado inválido, ou uma checagem escrita à mão por um constraint que a macro impõe. Cada uma pega uma classe de "você tinha que lembrar" e transforma em "você não consegue esquecer". Um teste comprova que o seu código funciona nas entradas que você tentou; a prova estilo Kani e uma regra de borrow funcionam nas entradas que você não tentou. Quando você avaliar qualquer garantia de framework daqui em diante, esse é o eixo para avaliar: ela pega o bug quando é barato corrigir, ou quando é caro, e você consegue fazer opt-out sem querer.

## Lab: faça o borrow checker te parar

Você vai rodar dois experimentos. O primeiro, no vault da lição passada, comprova o que a trava *não* cobre, porque metade de usar bem uma garantia é saber onde ela não está. O segundo te leva até o erro de verdade, na forma de conta que rendeu ao v1 as cicatrizes dele, e você conserta reordenando. O único artefato novo é um crate de probe de rascunho, que te carrega através do Challenge e é apagado depois. Primeiro, confirme que você está no toolchain certo, porque nada disso vale na linha V1.

**Passo 1. Fixe o toolchain do V2.** Uma linha:

```bash
anchor --version   # must report the V2 line (2.0.0-rc.1 as of 2026-08-12), not 1.1.2
```

Se ele reportar 1.x, refixe com o bloco de instalação de m01-l2 — `--tag v2.0.0-rc.1`, `--locked` — e lembre por que o desvio existe: `avm install` busca um binário pré-compilado do GitHub Release da tag, nenhum Release foi cortado para a tag v2, então o download dá 404. Cheque de novo se apareceu um rc mais novo antes de fazer build, e fixe no seu `Anchor.toml` e no CI o que `anchor --version` reportar, para que um colega faça build do mesmo bytecode que você.

**Passo 2. Comprove que a trava é exata.** Abra o handler `withdraw` da lição passada. Aqui está a forma, com a leitura de probe do topo da lição parada no meio da montagem da CPI, enquanto os handles estão vivos:

```rust
pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
    // --- the guard block from last lesson stays EXACTLY as you wrote it ---
    // require!(amount > 0, ...), require!(amount <= vault_lamports, ...),
    // and the checked remainder against the rent-exempt floor. Those three
    // lines are the security boundary of this handler; nothing in this lesson
    // touches them, and deleting them to shorten the snippet is how a teaching
    // edit becomes a custody bug. Elided below only for length.

    let owner = *ctx.accounts.authority.address();
    let sol_bump = ctx.accounts.state.sol_bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"sol", owner.as_ref(), &[sol_bump]]];

    let cpi = CpiContext::new(
        ctx.accounts.system_program.address(),
        Transfer {
            from: ctx.accounts.sol_vault.cpi_handle_mut(),
            to:   ctx.accounts.authority.cpi_handle_mut(),
        },
    )
    .with_signer(signer_seeds);

    // The probe: typed account data, read while `cpi` (holding live handles) is alive.
    let before = ctx.accounts.state.credit;   // <-- compiles: state is not in this CPI

    transfer(cpi, amount)?;

    let state = &mut ctx.accounts.state;
    state.credit = state.credit.checked_sub(amount).ok_or(VaultError::Underflow)?;

    let _ = before;
    Ok(())
}
```

Rode `anchor build`. Verde. Fique um instante com isso, porque é a descoberta: a CPI toma `sol_vault` e `authority`, `state` não é nenhum dos dois, e o compilador libera a leitura de propósito. Depois olhe para *por que este programa não consegue te mostrar o erro da classe reload de jeito nenhum*: as duas contas que esta CPI de fato toma são um `SystemAccount` e um `Signer`, e nenhuma das duas carrega dados tipados para ler. A CPI do R2 não tem nada que você pudesse ler obsoleto, então o modelo de borrow não tem leitura obsoleta para proibir. Isso não é uma lacuna na lição; é a trava traçando o perigo com exatidão. Apague a linha do probe e deixe o R2 como você encontrou.

**Passo 3. Construa o probe onde a trava morde.** Para encontrar o erro você precisa de uma conta que tenha dados tipados *e* entre numa CPI de forma mutável — que é precisamente a forma para a qual o `.reload()` do v1 foi inventado: um vault de token cujo `amount` você checa em volta de uma transferência. Tokens chegam formalmente no módulo 5; você não está construindo infraestrutura de token hoje, só aproveitando um tipo de conta para um probe de dez minutos, para que quando o vault se formar em SPL em m05-l1 você já tenha encontrado a ponta mais afiada dele. Faça um crate de rascunho, e faça ele **fora** do seu workspace do fliperama — `cargo new` rodado numa raiz de workspace registra o pacote novo como membro, e um probe de dez minutos não tem nada que fazer no lock que os seus degraus compartilham:

```bash
cd ..   # out of the arcade workspace first
cargo new borrow_probe --lib && cd borrow_probe
```

Aponte o `Cargo.toml` dele para a linha V2, com os pins que todo crate de programa deste curso carrega. Sozinho, ele toma o pin exato de `solana-address` em vez do teto de workspace que os degraus usam:

```toml
[package]
name = "borrow_probe"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib"]

[dependencies]
anchor-lang = "2.0.0-rc.1"     # crates.io, not the branch: see m01-l2
anchor-spl  = "2.0.0-rc.1"     # anchor-lang and anchor-spl move together on the V2 line
# The pins from m01-l2 (issue #4937's class). The exact =2.6.0 suits this throwaway
# probe; the arcade workspace crates ride the ">=2.6.1, <2.7" ceiling from m02-l1
# instead, because a workspace resolves one solana-address for all of its members.
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

E faça `src/lib.rs` ser exatamente isto — uma conta de token do vault, um `transfer_checked` para fora dela, e a checagem de saldo onde a memória muscular do v1 põe:

```rust
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod borrow_probe {
    use super::*;

    pub fn payout(ctx: &mut Context<Payout>, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            ctx.accounts.token_program.address(),
            TransferChecked {
                from: ctx.accounts.vault_ta.cpi_handle_mut(),
                mint: ctx.accounts.mint.cpi_handle(),
                to: ctx.accounts.recipient_ta.cpi_handle_mut(),
                authority: ctx.accounts.authority.cpi_handle(),
            },
        );

        // v1 muscle memory: check the balance while the CPI is being assembled.
        let before = ctx.accounts.vault_ta.amount();

        token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;

        let _ = before;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Payout {
    pub authority: Signer,
    pub mint: InterfaceAccount<Mint>,
    #[account(mut)]
    pub vault_ta: InterfaceAccount<TokenAccount>,
    #[account(mut)]
    pub recipient_ta: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}
```

**Passo 4. Leia o erro.** Rode `cargo build`. Passada uma página de avisos de macro `unexpected cfg` do RC, você recebe uma rejeição do borrow checker, não um aviso de lógica. Esta é a saída de verdade do rustc:

```text
error[E0502]: cannot borrow `ctx.accounts.vault_ta` as immutable because it is also borrowed as mutable
  --> src/lib.rs:22:22
   |
14 |                 from: ctx.accounts.vault_ta.cpi_handle_mut(),
   |                       --------------------- mutable borrow occurs here
...
22 |         let before = ctx.accounts.vault_ta.amount();
   |                      ^^^^^^^^^^^^^^^^^^^^^ immutable borrow occurs here
23 |
24 |         token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;
   |                                           --- mutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

Leia o que ele está te dizendo, porque ele está te dizendo a verdade, e repare na primeira linha antes de qualquer outra coisa: o compilador nomeia `ctx.accounts.vault_ta` — a conta, não a struct. A linha 14 é onde `cpi_handle_mut()` tomou o borrow mutável do vault. A linha 24 é onde `transfer_checked` consome `cpi`, então o borrow tem que continuar vivo até lá. A sua leitura na linha 22 pede um borrow compartilhado da mesma conta no intervalo entre as duas. Dois borrows conflitantes de uma conta, um span, sem build. E veja do que ele *não* reclamou: a mesmíssima forma de expressão em `mint` — `ctx.accounts.mint.decimals()` na linha 24 — passa batido, porque o mint entrou na CPI por um `cpi_handle()` compartilhado e borrows compartilhados toleram leituras. O compilador não está chutando que a sua leitura pode estar obsoleta. Ele tornou a leitura-durante-mutação estruturalmente inexprimível, para exatamente a conta que está sendo mutada.

![A leitura que falha de vault_ta.amount() fica acima da chamada de transfer_checked, e mover ela para baixo, onde o handle é solto, compila e lê a conta viva.](assets/v08-annotated-code.webp)

**Passo 5. Conserte e comprove.** Mova a leitura para baixo da linha de `transfer_checked`, exatamente como o painel AFTER mostra. Faça build de novo:

```bash
cargo build
```

Verde. Checkpoint: o probe compila com a leitura depois da CPI, e o R2 — que você deixou intocado depois do Passo 2 — continua fazendo build sozinho. Você não acrescentou um `.reload()`. Você não clonou nada. Você moveu uma leitura para o outro lado do ponto em que o handle é solto, e o borrow checker deu o aval. Essa reordenação *é* a forma idiomática do V2. O handle existe para a CPI e só para a CPI; assim que ele é solto, os dados vivos são seus de novo. Mantenha o crate de probe por perto para o Challenge; ele está prestes a ganhar um segundo handler.

Uma coisa para internalizar antes do Challenge: o compilador não te parou porque o seu número estava obsoleto. Ele te parou porque você leu a única conta sobre a qual a CPI segurava o direito de mudar, no span em que ela segurava esse direito. No v1 a mesma leitura compilava e te entregava o passado. Rigoroso em tempo de compilação é uma conversa. Silencioso em tempo de execução é um incidente.

## Challenge: conserte e nomeie a classe de bug

Sem apoio desta vez, e dois degraus. O primeiro é um handler quebrado diferente para o programa de probe que você construiu no Lab: `refund` devolve tokens para fora do vault e quer emitir o saldo que o refund *deixa para trás*, mas ele lê esse número no lugar errado, então não vai compilar. Diferente do Lab, você está por conta própria na correção. O segundo degrau é avaliado, fica abaixo desta seção, e pede uma coisa que a correção não pede.

O handler precisa de três declarações que você ainda não tem, então tome elas como dadas e acrescente ao programa de probe. O evento é a forma simples de `#[event]` de m01-l4, e a struct de accounts é a `Payout` do Lab com a vaga do recipient renomeada:

```rust
#[event]
pub struct Refunded {
    pub authority: Address,
    pub remaining: u64,
}

#[derive(Accounts)]
pub struct Refund {
    pub authority: Signer,
    pub mint: InterfaceAccount<Mint>,
    #[account(mut)]
    pub vault_ta: InterfaceAccount<TokenAccount>,
    #[account(mut)]
    pub authority_ta: InterfaceAccount<TokenAccount>,
    pub token_program: Interface<'static, TokenInterface>,
}

#[error_code]
pub enum ProbeError {
    #[msg("refund exceeds the vault's token balance")]
    Overdraw,
}
```

(Se o seu crate de probe estiver sem as linhas `wincode` e `solana-address` do Passo 3, `#[event]` falha bem aqui com `Address: SchemaWrite<...> is not satisfied`, muito antes de você chegar a ver o erro de borrow.)

E aqui está o handler quebrado:

```rust
pub fn refund(ctx: &mut Context<Refund>, amount: u64) -> Result<()> {
    let owner = *ctx.accounts.authority.address();

    // Legal: no handle exists yet, so this typed read is free.
    require!(
        amount <= ctx.accounts.vault_ta.amount(),
        ProbeError::Overdraw
    );

    let cpi = CpiContext::new(
        ctx.accounts.token_program.address(),
        TransferChecked {
            from: ctx.accounts.vault_ta.cpi_handle_mut(),
            mint: ctx.accounts.mint.cpi_handle(),
            to: ctx.accounts.authority_ta.cpi_handle_mut(),
            authority: ctx.accounts.authority.cpi_handle(),
        },
    );

    // The event wants the post-refund balance. This read is in the wrong place.
    let remaining = ctx.accounts.vault_ta.amount();

    token_interface::transfer_checked(cpi, amount, ctx.accounts.mint.decimals())?;

    emit!(Refunded { authority: owner, remaining });
    Ok(())
}
```

Repare, antes de tocar nele, que este handler já lê `vault_ta.amount()` uma vez, legalmente: a trava de over-refund no topo roda antes de qualquer handle existir. A mesma leitura mais abaixo no handler, depois de o `CpiContext` ser construído, é a que o compilador rejeita — e mesmo que compilasse, seria o saldo *pré*-refund, o número errado para um evento que promete o saldo deixado para trás. O erro de borrow e o erro de lógica são a mesma linha, e uma jogada só conserta os dois.

Duas coisas são exigidas para passar.

1. Reordene o código para que ele compile e esteja correto: a leitura de `remaining` tem que cair depois de `transfer_checked` consumir `cpi`, para que os handles estejam mortos e `remaining` seja o verdadeiro saldo pós-refund — fresco por construção, porque `Account<T>` lê os bytes vivos. O `emit!` usa esse valor.
2. Em uma frase, nomeie a classe de bug que o V2 eliminou aqui, a que o v1 apenas compilava e entregava.

Critérios de aceitação que a revisão checa diretamente:

- o handler compila no toolchain do V2
- `remaining` é lido estritamente depois de `transfer_checked` consumir `cpi`, então ele reflete o saldo pós-refund e não o pré-refund
- a trava de over-refund mantém o lugar dela acima do `CpiContext`, onde a leitura é legal — uma linha depois de os handles ficarem vivos ela não seria
- nenhum `.reload()` aparece em lugar nenhum (ele não existe no V2, e buscar ele é a cilada da memória muscular)
- a sua resposta de uma frase nomeia a classe eliminada: uma leitura obsoleta pós-CPI, silenciosa, em que o programa lê a cópia pré-CPI de uma conta depois de uma chamada entre programas e age sobre um valor que a cadeia já mudou, a classe que o v1 exigia que você lembrasse do `.reload()` para evitar.

![Se um CpiHandle para a conta ainda está em escopo você não pode ler ela ainda, então solte ele consumindo o CpiContext, e depois leia o campo direto já que o .reload() acabou.](assets/v09-flowchart.webp)

Se você consegue enunciar a classe em uma frase e a sua reordenação faz build, você é dono do conceito, não só da correção. O que deixa o segundo degrau, o avaliado. Ele é um handler `settle` reduzido a Rust puro: substitutos feitos à mão para a conta, o handle e o `CpiContext`, pequenos o bastante para compilar sem framework nenhum na foto e fazendo borrow exatamente do jeito que os de verdade fazem. Ele deve um recibo — o saldo do vault *antes* da transferência e o saldo dele *depois*, os dois lidos da conta e não calculados a partir dos argumentos. O starter estaciona as duas leituras no span proibido, então ele não faz build, e o scaffold impõe a regra "lido, não derivado" do mesmo jeito: logo depois de as contas serem construídas ele sombreia `vault_start`, `recipient_start` e `amount` em valores unitários, então um recibo escrito a partir de aritmética nos argumentos também é um erro de tipo. A avaliação para Rust é compile-only, e o sombreamento faz a maior parte desse trabalho: uma leitura fora de lugar é um `E0502`, e um recibo derivado de `vault_start`, `recipient_start` ou `amount` é um erro de tipo. Seja honesto sobre a lacuna que isso deixa, porque uma cerca que você acha fechada é pior que uma que você sabe aberta — `transfer_amount` sobrevive ao sombreamento como um `u64` vivo, então `opening - transfer_amount` compila limpo e até retorna o par certo. Nada te impede de escrever isso a não ser saber por que a leitura é a coisa que está sendo ensinada. A avaliação compile-only cerca as formas que ela consegue ver, e esta é uma boa lição sobre quais formas são essas.

## Onde isso te deixa

Fique com a vitória. Você acabou de ver o borrow checker se recusar a compilar um bug que era entregue às centenas em programas Anchor de produção, e você consertou movendo uma linha. Essa é uma sensação estranha e boa: o compilador pegou uma coisa que antes exigia um code review, um revisor cuidadoso e um pouco de sorte. A classe leitura-obsoleta-depois-da-CPI não é mais uma coisa que você precisa lembrar de evitar, porque o compilador não deixa você escrever ela.

Mantenha a borda afiada na cabeça, porque é a parte que as pessoas erram, nas duas direções. A garantia cobre só o acesso tipado a conta — lamports crus de `AccountInfo` e leituras de bytes feitas à mão continuam suas para raciocinar — e ela cobre só as contas das quais um handle vivo de fato faz borrow: tudo que você não entregou à CPI continua legível o tempo todo, que é por que o probe do R2 fez build verde. O modelo de borrow não atualiza nada, ele trava quando você tem permissão de ler para que a leitura que você tem permissão de fazer seja fresca. Se a sua correção fez build, você passou na trava. Se não fez build, o culpado é quase sempre uma leitura que continua parada acima da linha que solta o handle: mova ela para baixo, passando a chamada que consome o `CpiContext`, e tente de novo.

Aqui está o conjunto de diagnóstico para levar desta lição, porque na próxima vez que um erro de borrow encarar você durante uma CPI, três perguntas resolvem sempre. Esta leitura é de um campo tipado de uma conta, ou de lamports ou bytes crus? Se é crua, o compilador não é quem está te parando e a freshness fica com você. Se é tipada, um `CpiHandle` para aquelas contas ainda está em escopo nesta linha? Se sim, esse é o erro inteiro, e a correção é encerrar o escopo do handle antes da leitura, não buscar nada novo. E eu quero o estado da conta antes da CPI ou depois dela? Antes quer dizer capturar uma cópia numa variável local lá no topo; depois quer dizer ler abaixo do ponto em que o handle é solto, onde ele já está vivo. Rode essas três e o borrow checker deixa de ser uma parede e vira um checklist.

Existe uma próxima pergunta natural escondida em tudo isso. Se um programa assinando por si mesmo é custódia, o que acontece quando um pagamento depende de duas partes e de uma condição, e o depósito precisa morar dentro de um vault que você já construiu? Isso é composição, e é onde o handle rastreado por borrow deixa de ser uma regra de segurança e começa a ser a coisa que deixa um programa construir com segurança sobre o estado de outro. Na próxima lição você constrói o R3, o prize-escrow: ele reserva um depósito dentro de uma instância real de quarter-vault do R2 através de uma CPI trabalhada, e libera o prêmio só quando a condição de vitória vale e o chamador confere. O escrow confia no vault, e o V2 faz dessa confiança uma coisa que o compilador te ajuda a manter.

Você quebrou e consertou, e sabe nomear a classe. Boa construção.
