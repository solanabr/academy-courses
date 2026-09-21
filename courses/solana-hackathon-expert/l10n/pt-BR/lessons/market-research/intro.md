# Tamanho de mercado em velocidade de hackathon

Na lição passada você pontuou três ideias e ficou com uma, e o memorando de ideias tem os projetos anteriores mais parecidos anotados ao lado da sobrevivente. Abra esse memorando. A **nota de lacuna** dele, a linha que diz o que esses projetos não fizeram, é *a primeira linha do arquivo que você monta hoje*.

## Por que isso importa

Imagine a entrevista. Um jurado pergunta qual o tamanho disso e o time responde bilhões. A pergunta seguinte é quantas pessoas poderiam usar isso no mês que vem, *e a sala fica em silêncio*. O primeiro número saiu de um slide que alguém achou em um relatório. O segundo, ninguém foi buscar. O segundo número é **o que pontua**, e até o fim de hoje você tem os dois, de fontes que um jurado consegue clicar.

Um número que o jurado consegue clicar é **uma contagem em uma página pública**, com a URL e a data ao lado, que ele pode abrir durante a avaliação. O mapa de concorrentes e os três números de tamanho de mercado são esse formato aplicado duas vezes. Comece agora: abra colosseum.com/hackathon, encontre Potential Market Size (tamanho potencial do mercado) na seção de julgamento e **crie competitor-map-sizing.md** ao lado do memorando de ideias, no repositório do toolkit.

## Mão na massa

1. **Copie as três perguntas do fator** para o arquivo, com a data na primeira linha. Lido em **2026-09-06**, na temporada World's Fair de 2026, o fator diz o seguinte:

```text
competitor-map-sizing.md, aberto em 2026-09-06
Potential Market Size, colosseum.com/hackathon, copiado em 2026-09-06
How big is the total addressable market for this project? (Qual é o tamanho do mercado total endereçável deste projeto?)
Is it already large, or small but growing rapidly? (Ele já é grande, ou pequeno mas crescendo rápido?)
What will be the impact of this project on the growth rate of their market? (Qual será o impacto deste projeto na taxa de crescimento desse mercado?)
```

Só a primeira pergunta quer um número grande. A segunda quer uma direção, e a terceira quer saber o que o seu projeto faria com essa direção. Um slide que diz bilhões *responde um terço do fator*.

2. **Defina o responsável** pelo mapa. Do jeito que eu toco um time de hackathon, uma pessoa responde pela pesquisa de concorrentes, e essa pessoa é quem recebeu o papel no cartão do time no dia 0. Se ninguém recebeu, escolha agora, *porque mapa sem responsável fica preenchido pela metade na última noite*.

Depois peça para o responsável treinar o formato em earn.superteam.fun. Em 2026-09-06 a página inicial mostrava **213,730+ usuários** e 2,680+ patrocinadores. Em 2026-09-07 ela mostrava 214,040 e 2,690. Anote as duas leituras no arquivo com a URL e a data. Isso é demanda por outro produto, mas tem o formato que todo número do seu arquivo precisa ter: **uma contagem, uma página, uma data**, e um sinal de mais que avisa que a página arredonda para baixo.

3. **Liste os cinco projetos** mais parecidos com o seu pelo problema do usuário, não pela stack, *de um jeito que uma caderneta de papel possa ser concorrente e um protocolo com a mesma arquitetura que a sua talvez não*. **Cinco linhas**, porque três escondem um concorrente e dez escondem o leitor. Se a sua ideia é on-chain, você vai querer listar cinco protocolos, e isso é a sua stack falando: volte à frase do pitch do dia 0, ache a pessoa que está nela e pergunte o que ela usa hoje. A resposta dela é a linha um. O cabeçalho do mapa, antes de qualquer linha, é uma linha só:

```text
linha  quem  acerta  erra  cobra  última atividade (fonte, data)
```

**Preencha quatro células** por linha. Preencha cobra mesmo quando a resposta for zero, *porque ela alimenta a linha de Viability (viabilidade) mais para frente*. Preencha última atividade com a página do explorer, a data da última atualização na loja de apps, a data do último commit no GitHub ou uma contagem pública que se mexeu, **nunca com um anúncio**, e se anúncio for tudo o que você achar, escreva "só anúncio" na célula, com data.

Para o Fiado, cujo usuário é o mercadinho que guarda o fiado numa caderneta ao lado do caixa, as cinco linhas vão da caderneta até um app de fintech que empresta no caixa. A linha um é **a caderneta de papel**: de graça, privada, funciona sem celular e sem sinal, se perde ou molha, não cobra nada, *em uso hoje em todo mercadinho da rua*. A linha dois é **a planilha ou a conversa de WhatsApp**: sobrevive à água e dá para pesquisar, a cobrança continua na mão, não cobra nada.

A linha três é **um app de compre agora, pague depois**, e o mais parecido no dia da leitura era a Pagaleve: a página para varejistas mostra faixas de faturamento e "A Pagaleve assume 100% dos riscos" sem taxa para o lojista, então a célula cobra diz não informado, cotação sob consulta, com data, e a home mostra +10.000 lojas. A página do Pix Parcelado da Koin, lida no mesmo dia, mostra "3x sem juros ou em ate 24x" e nenhuma porcentagem.

A linha quatro é **a função de crédito** dentro de uma maquininha ou sistema de PDV que o mercadinho já aluga. A linha cinco é **uma carteira ou ferramenta para lojistas em stablecoin** que já liquida em stablecoin e poderia ganhar um fiado. As linhas quatro e cinco precisam de nome, página de preços e leitura de última atividade, tudo de páginas que você abre hoje:

```text
mapa de concorrentes, Fiado, 2026-09-06
linha  quem                                acerta                          erra                           cobra                última atividade
1      a caderneta de papel                de graça, privada, sem celular  perde, molha, dona cobra       nada                 hoje, todo mercadinho
2      planilha / conversa de WhatsApp     sobrevive à água, pesquisável   cobrança ainda na mão          nada                 hoje
3      app BNPL: Pagaleve                  assume o risco de calote, +10.000 lojas (site, 2026-09-07)   <das avaliações, com data>   não informado, cotação sob consulta (pagaleve.com.br/varejistas, 2026-09-07)   <listagem na loja, com data>
4      crédito no PDV: <nome>              <do site dele, com data>        <das avaliações, com data>     <página de preços, com data>  <listagem na loja, com data>
5      ferramenta stablecoin: <nome>       <do site dele, com data>        <das avaliações, com data>     <página de preços, com data>  <página do programa no explorer, com data>
```

Os colchetes angulares estão aí para você tirar. Uma linha que ainda tiver um na sexta *é uma linha sobre a qual o jurado vai perguntar*.

![O mapa do Fiado tem as linhas da caderneta e da planilha preenchidas com o que o time sabe, uma linha de BNPL preenchida pela metade a partir das páginas da Pagaleve, com data, e as linhas de PDV e stablecoin deixadas como espaços com data.](assets/v01-table.webp)

4. **Leia a linha cinco direto da chain**, antes das outras, *porque ela não depende da permissão de ninguém*. Todo programa na Solana tem página em qualquer explorer, com contagem de transações e a hora da última, sem login. **Copie três coisas** para a linha: a contagem total, a contagem ou lista dos últimos trinta dias onde aparecer, e a hora da transação mais recente, depois a URL e a data.

Agora leia o que você copiou. Contagem total grande com a última transação semanas atrás é um produto que **teve usuários e perdeu**. Contagem pequena com uma transação de uma hora atrás é *um produto vivo e no começo*. Nenhum dos dois é o que o site do próprio projeto diz. Um comparável, ou seja, um projeto que faz um trabalho parecido para um usuário parecido, também pode publicar o **TVL**, o valor que as pessoas depositaram nele, e esse número com data diz quanto dinheiro as pessoas já confiam a esse tipo de produto.

Com a aba aberta, **faça a leitura da stablecoin** para o tamanho de mercado: a página do token mostra a contagem de holders e as transferências recentes. A contagem de holders entra como **um limite superior de pessoas**, *porque uma pessoa tem várias carteiras e um bot tem milhares*, e as transferências dos últimos trinta dias entram como leitura de atividade, as duas com data. Se o emissor publica um número regional, copie esse também e marque de que página veio, porque o número do emissor e o do explorer não vão bater.

Para as linhas três e quatro **a leitura é off-chain**: a listagem na loja de apps mostra a data da última atualização, um repositório público mostra o último commit, uma página de preços mostra o preço e a data em que você leu, e a data da avaliação mais recente é um sinal de última atividade. Uma side track ou bounty da Earn também serve, quando a listagem mostra **quantas pessoas se inscreveram**, *porque essa contagem diz quantos builders acharam que o problema do patrocinador valia uma semana deles*.

![A célula de última atividade on-chain é montada copiando de um explorer a contagem total de um programa, a contagem recente e a hora da última transação, cada uma com a data da leitura.](assets/v02-table.webp)

5. **Escreva os três números como uma conta** antes de qualquer valor entrar. TAM é o mercado total endereçável, todo mundo que tem o problema. SAM é a parte que você conseguiria atender de fato com este produto, neste lugar, neste trilho. SOM é a parte disso que você consegue alcançar nos primeiros meses, por um canal que você consegue nomear.

A Colosseum pontua Potential Market Size com as três perguntas que estão no topo do seu arquivo. Um hackathon sazonal ou uma side track tem página própria, com entregas e prazo próprios. **Leia essa página no dia em que decidir entrar**, e reaproveite o material de submissão que você já tem. O slide é escrito na semana 4. A conta é feita hoje, com uma regra: **nenhum número sem as entradas**, e nenhuma entrada sem URL e data. A conta do Fiado:

```text
tamanho de mercado, Fiado, 2026-09-06

TAM = A x B
  A = número de pequenos comércios de varejo no Brasil     424,120 lojas de varejo alimentar, todos os portes, um limite superior
                                                            (ABRAS Ranking 2025, abras.com.br/dados-ranking-2025, lido em 2026-09-07)
                                                            só mercadinhos: parcela estimada de A, faixa
  B = crédito anual concedido por mercadinho no fiado       <estimativa, URL da fonte, data>

SAM = TAM x C x D
  C = parcela desses mercadinhos que vendem fiado           <parcela, URL da fonte, data>
  D = parcela cujos fregueses já conseguem pagar em
      stablecoin ou por um trilho que o fiado pode usar     <contagem de adoção, URL da fonte, data>
      contexto: o Brasil recebeu US$318.8 bilhões em valor cripto, jul 2024 a jun 2025, todas as criptos
      (chainalysis.com/blog/latin-america-crypto-adoption-2025, 2025-10-02, lido em 2026-09-07); parcela de stablecoin: espaço

SOM = E, alcançável até o mês três
  E = mercadinhos alcançáveis por uma cooperativa ou
      associação de comerciantes que o time consiga nomear  <número de membros, página própria, data>
```

Cada letra é uma página. A é uma contagem pública e **um limite superior**, porque o número da ABRAS conta toda loja de varejo alimentar, não só mercadinhos, então a parte que é só mercadinho vai ao lado como estimativa, com uma faixa. B não tem página e vai ser estimado a partir das conversas da próxima lição, então escreva como faixa agora, com "estimado" ao lado. C e D são parcelas, e *parcela precisa de duas contagens, a de cima e a de baixo, as duas com data*.

E é **o menor número do arquivo** e o que o jurado lembra. Eu trato estimativa como *uma previsão que carrega incerteza, não um compromisso*, e é por isso que B e C entram no arquivo como um mínimo e um máximo, com o raciocínio ao lado, em vez de um número só, cheio de confiança.

![O TAM do Fiado é mercadinhos vezes crédito por mercadinho, o SAM afunila isso por duas parcelas com data, e o SOM são os mercadinhos que uma cooperativa com nome alcança até o mês três.](assets/v03-flowchart.webp)

6. **Dê nome ao canal do mês três** e copie a contagem dele. Ninguém consegue alcançar uma porcentagem do SAM. Uma cooperativa ou uma associação de comerciantes do bairro, sim, *uma pessoa com um telefone alcança*: ela tem uma página, a página tem **um número de membros ou uma lista**, e essa contagem com URL e data é o E. Escreva a linha do SOM embaixo da conta exatamente neste formato:

```text
SOM, Fiado: <E> mercadinhos, alcançáveis por <nome da associação, página dela, data>,
até o mês três, porque <a pessoa que vai nos apresentar, e o que ela combinou>
```

A última parte é *trabalho da semana que vem*. A contagem é de hoje.

Para a segunda pergunta, a direção, **copie a página de adoção de stablecoin** do D duas vezes, hoje e na data mais antiga que a página ou um arquivo mostrar, e escreva as duas leituras lado a lado. Se a página não tem leitura anterior, escreva "uma leitura, sem tendência" e **não invente uma curva**. A terceira pergunta, o mecanismo, quer uma frase, e a do Fiado diz: *cada mercadinho que abre um fiado traz os fregueses dele para o trilho*, então a contagem de adoção em D cresce pelo número de fregueses do mercadinho cada vez que E cresce em um.

![O arquivo é montado definindo o responsável pela concorrência, preenchendo cinco linhas a partir de páginas com data, escrevendo as fórmulas de tamanho de mercado com entradas em letras e dando nome ao canal do mês três.](assets/v04-flowchart.webp)

## Está pronto quando

- **competitor-map-sizing.md** está ao lado do memorando de ideias, com a data na linha um e as três perguntas da Colosseum embaixo.
- O mapa tem **cinco linhas**, as mais parecidas pelo problema do usuário, cada uma com acerta, erra, cobra e última atividade, e toda célula de última atividade aponta para uma página de explorer, uma listagem, um repositório ou uma contagem com data.
- **TAM, SAM e SOM** estão escritos como fórmulas com entradas em letras, e toda entrada aponta para uma URL e uma data ou está marcada como estimada, com um mínimo e um máximo.
- **SOM é uma contagem** alcançável nos três primeiros meses por um canal com nome, número de membros e página. Uma parcela do SAM nessa linha reprova, *por mais razoável que a parcela pareça*.

## Fique atento

- **Um número de tamanho de mercado sem a conta por trás** é questionado, toda vez, e "um relatório disse" é a resposta que encerra a conversa.
- **Uma carteira não é um usuário**, então uma contagem de endereços ou de holders entra como limite superior de pessoas, nunca como contagem de clientes.
- **Um press release** é o concorrente dizendo o que ele quer que você pense, e nunca preenche a célula de última atividade.

Um TAM grande sem caminho até os primeiros cem usuários pontua pior do que **um pequeno e alcançável**, porque Traction (tração) e Viability ficam ao lado de Potential Market Size na lista e um SOM de mês três alimenta os três, então *a honestidade custa o slide impressionante e compra a entrevista*.

## O que fica

Potential Market Size faz três perguntas, e um número grande responde só a primeira. Todo número do seu arquivo é **uma contagem em uma página pública, com URL e data**, a última atividade de um concorrente vem de um explorer ou de uma listagem, nunca de um anúncio, e contagem de carteiras é limite superior de pessoas, nunca contagem de clientes. *SOM é o menor número do arquivo, alcançável por um canal com nome, e é o que o jurado lembra.*

## A seguir

Agora você tem um mapa de concorrentes e três números, e os estimados do arquivo, B e C no caso do Fiado, são faixas esperando evidência. A próxima lição começa **escrevendo a única suposição** que, se estiver errada, torna o arquivo inteiro inútil, e o teste mais barato capaz de provar que ela está errada até sexta. *Você passa o resto da semana tentando matar a própria ideia*, e reescreve o pitch com o que sobreviver.
