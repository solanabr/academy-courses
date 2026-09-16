# 1.x para 2.0: os deltas de reescrita

Na lição passada você mapeou o salto de 0.3x para 1.0: os renames, o único `#[error_code]` por programa, o passo de close do IDL on-chain, o `CpiContext::new` aprendendo a pegar um `Pubkey`. Cada uma dessas é sobrevivível com um find-and-replace e uma tarde. Você forçou a quebra num programa 0.32 e catalogou ela contra as seis mudanças; o port em si é subir um número no `Cargo.toml`, uma caçada pelos erros de compilação, e o mesmo programa saindo do outro lado. É essa a sensação de subir uma versão.

O salto de 1.x para 2.0 não é isso: é uma reescrita `no_std` do zero em cima do pinocchio, e a sintaxe que você já conhece para de passar na checagem de tipos. `Pubkey` não é mais um tipo. Os lifetimes `<'info>` que você escreveu em toda struct de conta desde o módulo 2 somem. O `has_one` compila mas se sublinha sozinho. O `dup` simples se recusa a compilar e te manda escrever `unsafe(dup)` no lugar. O `.reload()` sumiu, e não porque alguém renomeou ele. Então antes da gente raciocinar sobre qualquer coisa disso, vá ver a área de superfície que você está prestes a atravessar. Num programa v1 seu, rode isto:

```bash
rg -n 'Pubkey|\.key\(\)|has_one|zero_copy|<'"'"'info>|\.reload\(\)|realloc::payer|LazyAccount|AccountLoader|Migration<' src/
```

Toda linha que imprimir é uma linha que vai mudar. Algumas são renames de uma palavra. Umas poucas são erros de compilação com opinião. Uma delas, o `AccountLoader`, imprime como um hit comum que você vai ficar tentado a pular, porque o nome sobrevive até o V2 e o build nunca reclama dele. Essa é a mais perigosa do conjunto. Esse grep é a sua lista de trabalho para esta lição.

## Resumo

Este é o segundo delta de migração, e o mais difícil. No m10-l1 o modelo mental era "mesmo programa, grafia nova." Aqui o modelo honesto é "mesma intenção, programa novo," porque uma base de código 1.x não faz upgrade para 2.0, ela é reescrita linha por linha contra um release candidate que vive num branch separado e que não para de se mover.

O retorno pela dor é real, e ele é o fio condutor deste curso inteiro: o V2 mata classes inteiras de bug em tempo de compilação. A cilada de dado obsoleto que o `.reload()` remendava, o alias de duplicate-mutable, o cálculo de space feito na mão que conta a menos em silêncio. Cada uma dessas vira uma coisa que você não consegue escrever, não uma coisa que você tem que lembrar de checar. A gente vai merecer essa afirmação derivando cada mudança de sintaxe de volta para a decisão de design que forçou ela, a maioria das quais você já encontrou antes no curso. Depois você vai portar um programa v1 pequeno você mesmo, com o compilador como o seu par, e fechar num challenge de código que corrige o único bug de cálculo de space que sobrevive a um port descuidado.

O acompanhamento dá um passo atrás aqui de propósito. Os módulos do começo narravam cada tecla. A esta altura, o lab te entrega o código v1 e a tabela de delta e espera que você dirija, lendo os avisos do compilador como instruções em vez de esperar as minhas.

## Os deltas, e as decisões atrás deles

Comece pela pergunta que de fato importa, porque é ela que mantém o resto da lição honesto: por que 1.x para 2.0 quebra coisas que 0.3x para 1.0 não quebrou?

A resposta ingênua é "mais mudanças que quebram se acumularam." Tentadora, e errada. Se fosse só volume, a correção seria a mesma da última vez, só que mais longa: suba a versão, triture os erros, entregue. Essa abordagem falha no primeiro arquivo, e ela falha por uma razão específica. 0.3x para 1.0 era o mesmo framework com nomes novos. O V2 é um framework diferente que por acaso mantém a maioria dos nomes. É uma reescrita `no_std` construída em cima do pinocchio, a camada de runtime zero-copy e leve de dependências. O Anchor não editou o código velho dele para chegar aqui. Ele reconstruiu em cima de uma fundação nova.

Esse fato sozinho é o gerador. Quase todo delta à frente é consequência de uma de três decisões de design assadas dentro dessa reconstrução, e se você carregar as três decisões na cabeça você consegue prever os deltas em vez de decorar eles.

![Uma árvore mostrando três decisões de design na raiz (reescrita no_std, default zero-copy, CPI rastreada por borrow) cada uma se ramificando nas mudanças de sintaxe específicas que elas causam, mais um grupo transversal para os deltas narrados pelo compilador.](assets/v01-diagram.webp)

Antes das decisões, um pedaço de vocabulário e um mapa, porque o chão está se movendo enquanto você está de pé em cima dele. A linha v1 é o `anchor-lang` 1.1.2 — ou melhor, era: o v1.2.0 foi entregue em 2026-09-04, então a linha estável é 1.2.0 agora e ainda está recebendo commits. O V2 vive no branch `anchor-next` e é entregue como `2.0.0-rc.1`, publicado no crates.io em 2026-08-12 sob a tag git `v2.0.0-rc.1`. Essas são duas linhas paralelas, não um antes e um depois. Na escrita disto, 2026-08-22, a documentação de referência do V2 ainda carrega uma linguagem de instalação que antecede o publish no crates.io, te avisando para consumir os crates a partir do git. Não leia o texto de alpha ao lado dela como o mesmo tipo de atraso: o projeto rotula este release de `rc` e de `alpha` de propósito, e esse par é atual, não obsoleto. RCs se movem rápido. Re-cheque a versão no crates.io e a tag `anchor-next` antes de fixar qualquer coisa, e trate todo número de versão nesta lição como um retrato com uma data em cima.

![Uma linha do tempo de duas trilhas com a linha estável v1 e a linha release-candidate anchor-next do V2 correndo em paralelo ao longo de 2026, com o rc.1 chegando no crates.io em 2026-08-12.](assets/v02-timeline.webp)

### Decisão 1: a reescrita no_std renomeia as primitivas

`no_std` quer dizer que o framework não pode se apoiar na biblioteca padrão do Rust, então ele tira os tipos fundamentais dele de crates construídos para aquele mundo. Endereços agora vêm do `solana-address` através do pinocchio. É por isso que `Pubkey` vira `Address` e `.key()` vira `.address()`. Não é estética. O tipo velho vivia num grafo de dependências dentro do qual o V2 não fica mais.

Os lifetimes vão embora por uma razão aparentada. No v1 toda struct de conta carregava `<'info>` porque o framework passava um borrow do slice de contas da transação pelos seus tipos na mão, e você pagava por essa encanação em cada assinatura que você já escreveu. O modelo de conta do V2 rastreia esses borrows de outro jeito, então a anotação de lifetime para de ser uma coisa que você escreve. Handlers pegam `&mut Context<T>`, os wrappers perdem o `<'info>`, e uma coluna inteira de colchetes angulares desaparece do seu código. Comparado com o quê? Comparado com uma struct v1 onde `pub struct Initialize<'info>` e `Account<'info, Config>` repetiam o mesmo lifetime uma dúzia de vezes para dizer uma coisa que o compilador agora infere.

Se descer uma camada mais fundo do que "o framework lida com isso" é a coceira que você não para de coçar, é exatamente aí que o curso Low-Level Solana vive: inteiramente por baixo do framework, em cima do maquinário em que o V2 agora está sentado.

![Uma tabela de comparação emparelhando cada grafia do Anchor v1 com a substituição dela no V2 e a razão de uma linha, de Pubkey-para-Address até a remoção de reload().](assets/v03-comparison.webp)

### Decisão 2: zero-copy é o default, então a cerimônia em volta dele desaparece

No m02 você conheceu a cerimônia do v1 como história que você nunca teve que rodar: `#[account(zero_copy)]`, `AccountLoader`, `load()` e `load_mut()`, tudo para evitar desserializar uma conta grande para dentro da stack. O V2 faz do zero-copy o caminho comum, que é por que você só leu sobre o jeito velho. `Account<T>` é zero-copy por padrão, o que exige `T: Pod` com um layout sem preenchimento. **Pod** é plain old data, do módulo 2: uma struct de tamanho fixo e limpa de alinhamento em que todo padrão de bits é um valor válido, então o framework consegue deitar uma view tipada direto em cima dos bytes da conta em vez de decodificar eles. As consequências se espalham para fora, e é aqui que um port descuidado quebra em silêncio.

Primeiro, a fácil: o atributo `zero_copy` sumiu. Não tem nada para fazer opt-in porque você já está dentro. Delete ele.

Agora a complexidade que eu escondi, colocada de volta em voz alta. "Tudo é zero-copy" só é verdade para dados que são de fato Pod, e a barra para ser Pod é mais alta do que quem migra espera. Duas regras mordem no primeiro port. Primeira, todo campo tem que ser Pod ele mesmo, e um `bool` simples não é: só dois dos 256 padrões de bits dele são legais, então o V2 entrega `PodBool` para ele. Segunda, a struct tem que ter *preenchimento nenhum*. O `#[account]` emite `#[repr(C)]` mais uma asserção de tempo de compilação de que `size_of::<T>()` é igual à soma dos tamanhos dos campos, e quando não é você recebe isto, literalmente:

```text
account struct has padding bytes; reorder fields from largest to smallest
alignment to eliminate padding (e.g. u64 before u32 before u8)
```

Então uma struct v1 copiada para cá em geral não é Pod-legal na chegada. Você tem duas saídas honestas, e o port escolhe uma por conta. Redesenhe o layout da struct com campos de alinhamento 1 (`PodU64`, `PodBool`, arrays de bytes) para que a soma bata com o tamanho, ou roteie a conta através de `#[account(borsh)]` e do wrapper `BorshAccount<T>`, que é também onde vive qualquer coisa genuinamente de tamanho variável: um `Vec`, uma `String`. A segurança aqui não é uma convenção. Se um layout deixasse a leitura zero-copy insegura, o programa deixa de compilar em vez de ler bytes ambíguos em silêncio. O design inseguro é irrepresentável, que é a tese inteira do V2 numa frase só.

Aí a armadilha. O `AccountLoader` ainda existe no V2. O seu grep marca ele como mais uma linha entre muitas, o seu build não vai reclamar em nada, e é precisamente aí que está o perigo. Ele não quer dizer o que queria dizer no v1. O papel zero-copy do v1 que o `AccountLoader` costumava preencher se moveu para `Account<T>` (o novo default). O nome `AccountLoader` foi repropositado como um cursor sequencial de contas, uma coisa completamente diferente, e a documentação avisa explicitamente que ele "quer dizer outra coisa" agora. Este é o delta com mais chance de compilar e depois se comportar mal em vez de falhar alto. Tratar ele como "sumiu, delete" é errado. Tratar ele como "igual ao v1, mantém" é pior. Ele é um falso amigo: mesma cara, trabalho novo.

![Uma tabela mostrando o papel zero-copy do AccountLoader do v1 se movendo para Account-de-T, o nome AccountLoader repropositado como um cursor sequencial, e o LazyAccount ficando sem equivalente no V2.](assets/v04-comparison.webp)

Última consequência da Decisão 2, e a que você vai corrigir na mão no challenge: o cálculo de space. No 0.32 você escrevia na mão `space = 8 + 32 + 8`, onde o `8` da frente era o discriminator da conta que você mesmo acrescentava. A forma derivada chegou com o 1.0, como a mudança três do m10-l1, e o V2 mantém ela inalterada: `space = T::DISCRIMINATOR.len() + T::INIT_SPACE`. A razão de ela ainda morder quem migra no 2.0 é que a contagem na mão é aritmética de Rust legal, então um programa que pulou a edição do 1.0 compila no 1.1.2 com o `8` mágico intacto e chega aqui ainda carregando ele. O discriminator ainda tem 8 bytes (default sha256, inalterado e compatível com v1), então `DISCRIMINATOR.len()` é 8. A armadilha é que o `INIT_SPACE` é a soma dos tamanhos dos campos Pod e nada mais. Ele nunca inclui o discriminator. Um port pela metade que deleta o `8` mágico mas esquece que o `INIT_SPACE` exclui ele vai contar cada conta a menos por exatamente 8 bytes, dimensionar cada conta pequena demais, e estourar o buffer na primeira escrita.

Liste item por item, porque o número é o argumento inteiro. Pegue o `Config` do lab: um `Address` em 32 bytes, um `u64` em 8, um `bool` em 1. `INIT_SPACE` é `32 + 8 + 1 = 41`. O comprimento on-chain completo é `DISCRIMINATOR.len() + INIT_SPACE = 8 + 41 = 49`. O port descuidado computa `41` e aloca `41`, então a conta está exatamente um discriminator curta, e o primeiro byte do seu campo `authority` aterrissa onde o runtime esperava a conta acabar. O bug não vai dar crash em lugar nenhum que você consiga ler: é um off-by-8 que dimensiona certo na sua cabeça e errado na cadeia. Segure esse layout. Ele é o challenge.

![Uma faixa de layout de bytes para uma conta Config de 49 bytes: o discriminator de 8 bytes mais 41 bytes de INIT_SPACE, ao lado de uma alocação curta cujas escritas estouram em oito bytes.](assets/v05-diagram.webp)

### Decisão 3: a CPI é rastreada por borrow, então o .reload() não pode existir

Esta é a derivação carro-chefe da migração inteira, e ela é um retorno direto ao m04-l2, onde você já viu o acesso tipado durante um handle vivo virar um erro de compilação. Aplique essa mesma ideia à migração e o `.reload()` se explica sozinho.

Rebobine até por que o `.reload()` existia no v1. Você segurava uma cópia desserializada de uma conta. Você fez uma CPI que mutou aquela conta on-chain. A sua cópia em memória agora estava obsoleta, mostrando o saldo de antes da transferência. Se você tomasse uma decisão em cima daquela cópia obsoleta você tinha um bug de verdade, então o v1 te deu o `.reload()` para reler a conta depois da CPI e atualizar a sua cópia. Era um remendo para uma cilada que o sistema de tipos permitia.

O instinto de quem migra é: "o V2 removeu o `.reload()`, tudo bem, eu só chamo ele na mão depois de cada CPI como antes." Isso é ao mesmo tempo impossível e desnecessário, e a razão é um mecanismo só. No V2, contas de CPI são `CpiHandle`s rastreados por borrow. Um handle amarra a CPI a um borrow de Rust do wrapper tipado de onde ele veio, o que deixa o Anchor usar o caminho de CPI rápido e não checado do pinocchio enquanto o borrow checker proíbe aliasing tipado em tempo de compilação. Enquanto um handle está vivo, acesso tipado àquela conta é um erro de compilação. Então a situação de dado obsoleto não pode ser escrita. Não existe momento em que você segura uma cópia tipada obsoleta atravessando uma mutação, porque o borrow checker não vai deixar a cópia e o handle vivo coexistirem. Nada para dar reload. O método foi removido em vez de depreciado porque o bug que ele remendava não é mais exprimível.

Note qual classe de bug isso mata, porque é a pior classe. Um `.reload()` esquecido no v1 não falha no caso médio. A transferência ainda acontece, a CPI ainda tem sucesso, os testes que não dependem do saldo pós-CPI ainda passam. Ele falha só quando o seu handler lê o valor mutado e faz branch em cima dele: um withdrawal que checa um saldo que ele pensa que ainda é 100 quando a CPI acabou de mover ele para 0. Essa é a assinatura dos bugs mais caros na cadeia. Correto no caminho comum, errado exatamente quando tem dinheiro em jogo, invisível até o pior caso chegar. Depreciar o `.reload()` teria deixado aquela janela de pior caso aberta para qualquer um que esquecesse de chamar ele. Remover de vez a capacidade de segurar a cópia obsoleta fecha a janela para todo mundo, incluindo quem migra e nunca leu esta lição. Essa assimetria, caso-médio-tudo-bem contra pior-caso-catastrófico, é a razão exata de o V2 ter escolhido "irrepresentável" em vez de "documentado."

![Dois painéis de código: no v1 uma leitura pós-CPI fica obsoleta e o reload() remenda ela; no V2 acesso tipado durante um CpiHandle vivo é um erro de compilação.](assets/v06-annotated-code.webp)

Mais um delta pequeno desta decisão, porque é onde quem migra novo tropeça na sintaxe depois de entender o conceito: o `CpiContext::new` agora pega o programa como `&Address`. No m10-l1 você aprendeu a forma do 1.0, onde o `CpiContext::new` pegava um `Pubkey`. O V2 pega um `Address` emprestado. Mesma ideia, mais um rename de tipo seguindo a Decisão 1 rio abaixo para dentro da API de CPI.

### Os deltas transversais: deixe o compilador narrar

Duas mudanças não pertencem a uma decisão só. Elas pertencem a uma filosofia: fazer do caminho seguro o caminho comum, e quando você desviar, te obrigar a dizer isso no ponto exato. O compilador é o professor aqui, e ele foi engenheirado para ser.

Pegue o `has_one`. Porte `#[account(mut, has_one = authority)]` para o V2 e ele compila, mas emite um aviso de depreciação que sublinha especificamente a keyword `has_one`. Aquele sublinhado não é incidental. O parser do framework armazena o span da keyword `has_one` de propósito para que o codegen consiga apontar de volta para ele. O aviso está te dizendo a edição exata, nas palavras dele mesmo: "on the sibling field, use `#[account(address = owner.field)]` instead." O lado direito é qualquer expressão, na maioria das vezes o campo da conta pai. É uma depreciação com um mapa anexado. Não recorra a `#[allow(deprecated)]` para silenciar ele. O `has_one` está num caminho para remoção, e um RC posterior pode levar ele. O aviso está te fazendo um favor.

![Um painel de código mostrando um aviso de build do V2 que sublinha a keyword has_one através de um span de parser armazenado deliberadamente, com a grafia corrigida de address-igual-campo-do-pai embaixo dele, ainda num Signer.](assets/v07-annotated-code.webp)

Agora a mais afiada, a bifurcação linha a linha. O seu código v1 faz opt-out de uma conta mutável duplicada da checagem de duplicate-mutable com `#[account(mut, dup)]`. No V2 aquela linha não avisa. Ela falha em compilar. O `dup` simples é um erro de compilação, e o texto do erro nomeia a correção: escreva `unsafe(dup)`. Isso importa mais do que parece. O V2 ainda detecta mutáveis duplicadas, e a checagem ainda roda durante a validação de contas contra o bitvec percorrido. O que mudou é que a saída de emergência agora tem que ser grafada `unsafe`, no call site, toda vez que você usa ela. (O bitvec percorrido é a passada de tempo de execução do dispatcher lá do m01-l4: ele marca todo endereço que chega duas vezes, depois faz o AND disso contra a `MUT_MASK` de tempo de compilação da struct. O atributo é um evento de tempo de compilação; a colisão contra a qual ele protege é de tempo de execução.) A palavra está fazendo trabalho: ela te faz reconhecer, bem onde você desvia, que você assumiu a obrigação de escrever o handler de um jeito que ele nunca forme referências mutáveis conflitantes. A documentação diz que a saída se chama `unsafe(dup)` "de propósito." Um `dup` que compilava em silêncio era um risco que você podia esquecer. Um `unsafe(dup)` que você teve que digitar é um risco que você escolheu.

![Um fluxograma para portar uma conta duplicate-mutable: o dup simples falha em compilar, e duas perguntas sobre o alias te roteiam para remover o opt-out ou escrever unsafe(dup).](assets/v08-flowchart.webp)

### Renames mecânicos e as coisas que você tem que deixar para trás

Alguns deltas são pura grafia, nenhuma filosofia. `realloc::payer` vira `realloc_payer`, o namespace de dois-pontos-duplos colapsando num único token plano. Grep, substitui, segue. Essas são as edições com sabor de 0.3x-para-1.0 ainda vivendo dentro da migração mais difícil, e elas valem ser nomeadas precisamente porque elas te ninam: se as dez primeiras mudanças foram assim tão fáceis você vai supor que a décima primeira também é, e a décima primeira é o `AccountLoader`.

Aí as remoções. `LazyAccount` e `Migration<From, To>` são só do v1. Não existe grafia do V2 para portar eles. O `LazyAccount` era o wrapper somente-leitura, alocado no heap e de carregamento sob demanda do v1; num mundo zero-copy por padrão o nicho dele é em boa parte absorvido por `Account<T>`, então o wrapper não sobrevive. `Migration<From, To>` simplesmente não existe na linha do V2. Se o seu programa v1 se apoia em algum dos dois, isso não é um rename, é um redesign daquela peça.

Uma nota de segurança para o público do v1, já que alguns de vocês vão manter um programa na linha v1 por mais um tempo. O `LazyAccount` costumava pular a rechecagem de ownership dele no reload, então depois de uma CPI uma conta que você segurava de forma lazy podia ter o dono dela trocado embaixo de você sem o caminho lazy re-verificar isso. O PR #4784, "re-run ownership checks when reloading `LazyAccount`," corrigiu exatamente isso, e ele foi mergeado em 2026-07-16, que é *depois* de o 1.1.2 ser entregue em 2026-06-26. Leia as datas nessa ordem, porque elas são o ponto: se você está fixado no 1.1.2 você não tem a correção. Não é uma preocupação do V2, já que o `LazyAccount` não atravessa, mas é uma razão para auditar o seu uso de v1, e o seu pin de v1, antes de você decidir o que portar e o que reescrever.

### Você deveria portar, afinal, agora mesmo? Comparado com o quê?

Vale pausar na pergunta que um engenheiro cuidadoso faz antes de tocar num programa que funciona: você deveria migrar para o V2 hoje, ou esperar? O argumento por esperar é real, e eu vou fazer ele na forma mais forte antes de responder. O V2 é um release candidate. Ele é explicitamente não auditado, a documentação de referência ainda carrega ressalvas de alpha, e, como a #4937 mostrou, os próprios pins de crate podem quebrar um programa correto. Um programa segurando fundos de verdade na linha v1, estável no 1.1.2, tem toda razão para ficar lá até o 2.0 entregar um release estável e auditado. Isso é casar o risco do ferramental com o valor que ele protege, não timidez. Se o seu programa está em produção, o default honesto é: não porte fundos vivos para um RC.

Então comparado com o quê? Comparado com a alternativa de aprender os deltas depois de o 2.0 estar estável, sob prazo, numa base de código que você meio esqueceu. O port é uma reescrita, e uma reescrita que você faz com calma numa cópia de rascunho, enquanto o compilador te ensina cada delta, é uma tarefa completamente diferente da mesma reescrita feita na correria porque uma dependência finalmente derrubou o suporte a v1. Aprender os deltas agora é barato. Portar fundos de produção agora não é. Essas são duas decisões diferentes, e o erro é tratar elas como uma. Esta lição é a primeira: construa o mapa, porte algo descartável, domine o raciocínio. A segunda, quando mover um programa de verdade, é uma decisão que você toma depois com um release estável e uma auditoria na mão.

![Uma tabela de decisão separando a escolha barata, aprender os deltas e portar algo descartável agora, da cara de mover fundos de verdade para um RC não auditado.](assets/v09-comparison.webp)

### O tradeoff, e a disciplina que o RC força

Aqui está o livro-razão honesto. Do lado dos ganhos, o V2 remove classes de bug no nível dos tipos: a janela de dado obsoleto do `.reload()`, o alias de duplicate-mutable, o layout zero-copy inseguro, o discriminator contado na mão. Essas param de ser coisas que você checa e viram coisas que você não consegue expressar. Do lado do custo, uma base de código 1.x não faz upgrade, ela é portada, linha por linha, contra um release candidate num branch separado. Você troca uma subida mecânica de versão por uma reescrita de verdade, e você assume a rotatividade da era RC: o toolchain, os pins de crate, até a versão do serializador podem mudar embaixo de você entre uma escrita e a próxima.

Aquela última oração não é hipotética. A primeira história de guerra da era RC é a issue #4937, aberta em 2026-08-16 e fechada quatro dias depois, em 2026-08-20. O `anchor-lang` fixava o `wincode` (o serializador do V2) em `0.5` enquanto dependia do `solana-address` sem teto nenhum, e o `solana-address` subiu a própria exigência de `wincode` dele para `0.6`. O descompasso de trait bound entre os dois quebrou `#[account(borsh)]` com um simples erro `SchemaRead is not satisfied`. Ninguém escreveu código ruim. O grafo de dependências em si mordeu. Essa é a forma do risco num RC: código correto, pins incompatíveis. Então a disciplina não é opcional. Fixe versões exatas, não faixas de caret. Re-verifique a tag `anchor-next` e as versões dos crates antes de toda sessão de trabalho. Trate um build verde hoje como evidência só sobre hoje. A recompensa pela reescrita é um programa que falha em tempo de compilação em vez de na mainnet. O aluguel que você paga por isso, até o 2.0 estabilizar, é vigilância de versão.

Vigilância, concretizada, porque esse descompasso exato está vivo de novo enquanto você lê isto. A #4937 fechou quando os pins foram reconciliados, mas o `2.0.0-rc.1` publicado ainda fixa `wincode 0.5` enquanto deixa `solana-address` sem teto, e `solana-address 2.7.0` desde então se moveu para `wincode 0.6` — então uma resolução nova reconstrói exatamente o grafo que a issue descreveu, e o lab abaixo entra direto nele no `#[account(borsh)] Config`. Ele usa duas caras, uma causa só: `error[E0277]` no tipo da conta, `SchemaWrite`/`SchemaRead` "is not satisfied" com uma nota sobre múltiplas versões de `wincode` no grafo de dependências, e `error[E0433]: could not find wincode`, porque a expansão de `#[program]` nomeia `::wincode::` de forma absoluta e o seu crate não depende dele diretamente. A solução de contorno é o par de pins que todo `Cargo.toml` de programa neste curso carrega desde o m01-l2. O seu port é um crate autônomo, então ele toma a forma exata abaixo; o workspace do fliperama declara o mesmo teto `< 2.7` como uma faixa, porque cinco membros têm que concordar numa versão resolvida só — o m02-l1 faz esse argumento.

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"
wincode = { version = "0.5", features = ["derive"] }
solana-address = "=2.6.0"      # rc.1 pins wincode 0.5; solana-address 2.7.0 moved to 0.6
```

Os pins pertencem ao crate do programa seja qual for o canal em que o próprio `anchor-lang` anda, branch do git ou crates.io. Coloque eles no `Cargo.toml` do port antes do passo 1 do lab, ou encontre os dois erros no passo 6 e acrescente eles aí; de qualquer jeito, re-verifique eles toda sessão, porque este é exatamente o tipo de solução de contorno que um rc posterior deleta debaixo de você.

## Lab: porte um programa v1 para o V2, guiado pelo compilador

Você vai portar um programa v1 minúsculo mas completo: uma conta de config com uma authority, inicializada uma vez, depois atualizada por aquela authority. Ele exercita o rename do modelo de conta, o sumiço do lifetime, a depreciação do `has_one`, e o cálculo de space, que é a maior parte da superfície de delta em cinquenta linhas. Trabalhe num crate de rascunho, não num projeto de verdade.

Este lab faz a ajuda recuar. Eu te dou o código v1 e a tabela de delta acima. Você dirige o port e deixa o compilador te dizer o que sobrou.

1. **Instale e fixe o toolchain do V2.** Uma armadilha antes de você digitar qualquer coisa: o RC não passa pelo `avm`. Nenhum GitHub Release foi cortado para a tag v2, então o binário pré-compilado que o `avm install` baixa não está lá e o fetch dá 404. Os crates do rc.1 aterrissaram no crates.io em 2026-08-12, sim, mas a instalação documentada ainda é uma instalação a partir do git do branch `anchor-next`, fixada aqui na tag:

```bash
# no GitHub Release cut for the v2 tag -> no binary to download; use the git install
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version           # confirm 2.0.0-rc.1 before you touch code
```

Fixe essa versão exata no seu `Anchor.toml` e `Cargo.toml`. RCs se movem; re-cheque a tag no `anchor-next` primeiro (este lab foi escrito contra `2.0.0-rc.1`, 2026-08-22).

2. **Leia o código v1 que você está portando.** Ele compila na linha v1 (1.1.2), literal de space contado na mão e tudo, porque é um programa da era 0.32 que só recebeu as edições que o compilador forçou. Toda linha que o grep de antes marcaria é uma linha que você vai tocar:

<!-- verify: expect-fail the V1 'before' program in the migration module; it is not meant to build on V2 -->
```rust
use anchor_lang::prelude::*;

declare_id!("Cfg1111111111111111111111111111111111111111");

#[program]
pub mod config_v1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.seed = seed;
        config.active = true;
        Ok(())
    }

    pub fn set_active(ctx: Context<SetActive>, active: bool) -> Result<()> {
        ctx.accounts.config.active = active;
        Ok(())
    }
}

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub seed: u64,
    pub active: bool,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 8 + 1)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetActive<'info> {
    #[account(mut, has_one = authority)]
    pub config: Account<'info, Config>,
    pub authority: Signer<'info>,
}
```

3. **Renomeie as primitivas (Decisão 1).** `Pubkey` vira `Address`. `authority.key()` vira `authority.address()`. Uma pegadinha que o compilador vai te entregar: `.address()` devolve um `&Address`, não um `Address`, então a atribuição precisa de um deref. O scaffold do V2 escreve isso exatamente assim:

```rust
config.authority = *ctx.accounts.authority.address();
```

   Depois tire todo `<'info>` das definições de struct e de `Account<'info, Config>`.

4. **Corrija o modelo de conta e o space (Decisão 2).** É aqui que o port ingênuo morre. `Config` segura um `Address` (32), um `u64` (8) e um `bool` (1). O `bool` não é Pod de jeito nenhum, e mesmo se você trocasse ele por um `u8` o `u64` força alinhamento de 8 bytes, então o `repr(C)` estica a struct com preenchimento até 48 bytes enquanto os campos só somam 41: exatamente a asserção de preenchimento de antes. Pegue a segunda saída e roteie esta conta através do borsh, que é também para o que `#[derive(InitSpace)]` está documentado. Depois reescreva o space para a forma idiomática do V2, e note que ele não é `8 + 32 + 8 + 1` copiado adiante:

```rust
#[account(borsh)]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Address,
    pub seed: u64,
    pub active: bool,
}

#[account(
    init,
    payer = authority,
    space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
)]
pub config: BorshAccount<Config>,
```

5. **Corrija o constraint depreciado (o delta transversal).** `has_one = authority` compila mas se sublinha sozinho. Substitua ele pela checagem explícita de address que o aviso aponta. Note onde o constraint aterrissa: ele sai de `config` e vai para a conta `authority`, afirmando que o address da authority passada é igual ao campo guardado em `config`:

```rust
#[account(mut)]
pub config: BorshAccount<Config>,
#[account(address = config.authority)]
pub authority: Signer,
```

6. **Faça o build, e leia o compilador como a sua checklist.** Rode `anchor build`. Cada erro ou aviso é um delta que resta. Corrija o de cima, recompile, repita. Você deve chegar num build limpo sem `<'info>`, sem `Pubkey`, sem aviso de `has_one`, e com uma linha de space que lê `DISCRIMINATOR.len() + INIT_SPACE`.

**Checkpoint.** O `anchor build` tem sucesso e `rg 'Pubkey|<.info>|has_one' src/` não imprime nada. Se o build ainda reclamar de um lifetime, você deixou passar um `<'info>` numa struct. Se ele reclamar que um campo não é `Pod`, ou que a struct "has padding bytes," você deixou uma conta no caminho zero-copy default que não pode legalmente ficar ali: ou redesenhe o layout dela com campos de alinhamento 1 ou mova ela para `#[account(borsh)]` mais `BorshAccount<T>`, do jeito que o passo 4 fez com `Config`. Um programa V2 vem com um caminho de teste LiteSVM (`anchor_v2_testing::svm()`); um smoke test que inicializa o config e vira o `active` é a prova de que o port de fato roda, não só compila.

## Challenge: porte o cálculo de space sem perder o discriminator

Este é o único bug que sobrevive a um port de aparência cuidadosa, isolado para você matar ele limpo. Uma migração varreu um arquivo v1 atrás do `8` solto da forma idiomática `space = 8 + ...` — instinto certo, porque o V2 não tem número mágico — e levou todo oito aditivo que encontrou. O `* 8` que dimensiona um campo `u64` ficou intacto, corretamente: aquele é uma largura de campo, não um número mágico. Mas a varredura não conseguiu separar esses do `checked_add(8)` dentro de `with_discriminator`, o último elo da cadeia de dimensionamento, que era o passo que colocava o discriminator de volta, e o `INIT_SPACE` não coloca ele de volta para você. O elo sobrevive como uma função nomeada que agora passa a entrada dela direto adiante, e tudo que o helper dimensiona sai 8 bytes curto.

O seu trabalho é restaurar aquele elo para que `account_len` devolva o comprimento completo de dados on-chain: o discriminator de 8 bytes (default sha256, inalterado no V2) mais os tamanhos de campo somados, onde um `Address` tem 32 bytes, um `u64` tem 8, e um `bool` tem 1. Nomeie o nível, porque aquela soma só vale para um deles: estes são os tamanhos de `#[account(borsh)]`, campos escritos um atrás do outro sem preenchimento de alinhamento e sem prefixos de comprimento, que é o nível para o qual o passo 4 roteou o `Config`. Sob o default Pod os mesmos três campos nunca chegam a um comprimento — o `u64` força alinhamento de 8 bytes, a struct precisa de preenchimento de cauda, e o build para em `error[E0080]: account struct has padding bytes`. Note o nome também: a função não se chama `init_space`, porque `INIT_SPACE` é exatamente a metade que exclui o discriminator, e nomear ela assim é como o bug foi escrito em primeiro lugar.

A assinatura está congelada, porque os testes chamam ela por exatamente esta interface:

```rust
/// The chain's last link: from the INIT_SPACE half (field bytes only) to the
/// full on-chain data length. A `const fn`, so the compiler can prove it while
/// it builds (the m03-l3 device — what compile-only grading actually enforces).
const fn with_discriminator(init_space: u64) -> Option<u64> {
    // TODO: the 8 goes here -- checked, so an overflowing count stays a
    // refusal. Right now this link passes its input through unchanged.
    Some(init_space)
}

/// Returns the FULL on-chain data length for a `#[account(borsh)]` account:
/// T::DISCRIMINATOR.len() (8) + T::INIT_SPACE (the field bytes only).
fn account_len(address_fields: u64, u64_fields: u64, bool_fields: u64) -> u64 {
    // the field sums are all here; the last link is the gutted one above
    address_fields
        .checked_mul(32)
        .and_then(|bytes| bytes.checked_add(u64_fields.checked_mul(8)?))
        .and_then(|bytes| bytes.checked_add(bool_fields))
        .and_then(with_discriminator)
        .unwrap_or(0)
}
```

Deixe o resto da cadeia checada. Aquelas contagens chegam de quem chama, e o contrato do helper é que uma contagem de que ele não consegue fazer sentido volta como `0` — um comprimento que alocador nenhum vai aceitar — em vez de como um número que deu wrap e parece bem. Aceitação: `account_len` devolve `8 + 32*address_fields + 8*u64_fields + 1*bool_fields`; uma struct vazia `(0, 0, 0)` devolve `8`, não `0`; e a trava sobrevive à sua edição. Cinco testes: `(1,1,1)` dá `49`, `(2,3,0)` dá `96`, `(0,0,0)` dá `8`, `(1,0,2)` dá `42`, e `(u64::MAX,0,0)` dá `0`. Aquele último vetor não é uma conta — nenhuma struct tem dezoito quintilhões de campos — ele é a contagem-lixo fazendo as vezes do que quer que tenha dado errado rio acima, e ele está ali para cravar *onde* o seu 8 vai. Aparafuse ele depois da trava como `...unwrap_or(0) + 8` e a recusa volta como `8`, que lê exatamente como uma conta vazia legítima.

Três dicas, em ordem de quanto elas entregam. A forma idiomática do V2 é `T::DISCRIMINATOR.len() + T::INIT_SPACE`, e `DISCRIMINATOR.len()` é `8`. `INIT_SPACE` é só bytes de campo, então você soma o 8 de volta exatamente uma vez, nunca por campo. E o caso da struct vazia é a pista: se `(0,0,0)` devolve `0` você não somou nada; se devolve `16` você somou o discriminator duas vezes. Restaure o 8 como um `checked_add` dentro de `with_discriminator`, na mesma cadeia das somas de campo, não como um `+ 8` aparafusado depois do `unwrap_or` — e porque o elo é uma `const fn` com asserções embaixo dele, um elo esvaziado ou não checado nem chega a compilar.

## Antes de seguir em frente

A trava desta lição tem duas metades. Primeira, faça o challenge passar: starter vermelho, a sua correção verde, todos os cinco testes. Segunda, e esta é a que vale fazer longe do teclado, explique de memória por que o V2 removeu o `.reload()` em vez de meramente depreciar ele. Se a sua resposta apontar para o modelo de borrow do `CpiHandle`, que acesso tipado durante um handle vivo é um erro de compilação, então a janela de dado obsoleto é irrepresentável e não sobra nada para dar reload, você domina a derivação, não só o fato. Essa é a ideia do m04-l2 aplicada à migração, e é o exemplo mais claro de todos da tese do curso: a correção mais segura para uma cilada é tornar a cilada impossível de segurar.

Você tem os dois mapas agora, 0.3x para 1.0 e 1.x para 2.0. Um mapa não é uma migração, porém. A seguir você pega um programa 0.31 ou 1.0 de verdade e leva ele até o fim, até um build V2 que compila e passa no LiteSVM, usando estes deltas como a sua checklist e os avisos do compilador como o seu guia. Traga o grep. Bom port.
