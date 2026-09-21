# Dimensionar o mercado em ritmo de hackathon

Na lição passada você pontuou três ideias e ficou com uma, e o memorando de ideias tem os projetos passados mais próximos escritos ao lado da sobrevivente. Abra esse memorando. A **nota de lacuna** dele, a linha que diz o que esses projetos passados não fizeram, é *a primeira linha do arquivo que você constrói hoje*.

## Por que isso importa

Imagine a entrevista. Um jurado pergunta quão grande é isso e o time responde bilhões. A pergunta seguinte é quantas pessoas poderiam usar isso no mês que vem, *e a sala fica em silêncio*. O primeiro número saiu de um slide que alguém achou em um relatório. O segundo número ninguém foi buscar. O segundo número é **o que pontua**, e até o fim de hoje você tem os dois, de fontes que um jurado consegue clicar.

Um número que um jurado consegue clicar é **uma contagem em uma página pública**, com a URL e a data ao lado, que o jurado pode abrir durante a avaliação. O mapa de concorrentes e os três números de dimensionamento são esse formato aplicado duas vezes. Comece agora: abra colosseum.com/hackathon, encontre Potential Market Size (tamanho potencial do mercado) na seção de julgamento e **crie competitor-map-sizing.md** ao lado do memorando de ideias no repositório do toolkit.

## Faça isto

1. **Copie as três perguntas do fator** para o arquivo, com a data na primeira linha. Lido em **2026-09-06**, na temporada World's Fair de 2026, o fator diz isto:

```text
competitor-map-sizing.md, aberto em 2026-09-06
Potential Market Size, colosseum.com/hackathon, copiado em 2026-09-06
How big is the total addressable market for this project? (Quão grande é o mercado total endereçável deste projeto?)
Is it already large, or small but growing rapidly? (Ele já é grande, ou pequeno mas crescendo rápido?)
What will be the impact of this project on the growth rate of their market? (Qual será o impacto deste projeto na taxa de crescimento desse mercado?)
```

Só a primeira pergunta quer um número grande. A segunda quer uma direção, e a terceira quer o que o seu projeto faria com essa direção. Um slide que diz bilhões *responde um terço do fator*.

2. **Nomeie o dono** do mapa. Do jeito que eu toco um time de hackathon, uma pessoa é dona da pesquisa de concorrentes, e essa pessoa é quem recebeu o papel no cartão do time no dia 0. Se ninguém recebeu, escolha agora, *porque um mapa sem dono fica preenchido pela metade na última noite*.

Depois faça o dono praticar o formato em earn.superteam.fun. Em 2026-09-06 a página inicial mostrava **213,730+ usuários** e 2,680+ patrocinadores. Em 2026-09-07 ela imprimia 214,040 e 2,690. Escreva as duas leituras no arquivo com a URL e a data. Isso é demanda por um produto diferente, mas tem o formato de que todo número no seu arquivo precisa: **uma contagem, uma página, uma data**, e um sinal de mais que diz que a página arredonda para baixo.

3. **Liste os cinco projetos** mais próximos do seu pelo problema do usuário, não pela sua stack, *para que uma caderneta de papel possa ser concorrente e um protocolo com a sua arquitetura talvez não*. **Cinco linhas**, porque três escondem um concorrente e dez escondem o leitor. Se a sua ideia é on-chain você vai querer cinco protocolos, e isso é a sua stack falando: volte à frase do pitch do dia 0, encontre a pessoa nela e pergunte o que ela usa hoje. A resposta dela é a linha um. O cabeçalho do mapa, antes de qualquer linha, é uma linha só:

```text
linha  quem  acerta  erra  cobra  última atividade (fonte, data)
```

**Preencha quatro células** por linha. Preencha cobra mesmo quando a resposta é zero, *já que ela alimenta a linha de Viability (viabilidade) depois*. Preencha última atividade a partir da página do explorer, da última atualização na listagem da loja de apps, da data do commit no GitHub ou de uma contagem pública que se moveu, **nunca a partir de um anúncio**, e se o anúncio for tudo que você encontrar, escreva "só anúncio" na célula, com data.

Para o Fiado, cujo usuário é o mercadinho que guarda o fiado em uma caderneta ao lado do caixa, as cinco linhas vão da caderneta até um app de fintech que empresta no caixa. A linha um é **a caderneta de papel**: grátis, privada, funciona sem celular e sem sinal, se perde ou molha, não cobra nada, *ativa hoje em todo mercadinho da rua*. A linha dois é **a planilha ou a conversa de WhatsApp**: sobrevive à água e dá para pesquisar, ainda cobrada na mão, não cobra nada.

A linha três é **um app de compre agora, pague depois**, e o mais próximo no dia em que isso foi lido era a Pagaleve: a página para varejistas imprime faixas de faturamento e "A Pagaleve assume 100% dos riscos" sem taxa para o lojista, então a célula cobra diz não impresso, cotação sob consulta, com data, e a home imprime +10.000 lojas. A página do Pix Parcelado da Koin, lida no mesmo dia, imprime "3x sem juros ou em ate 24x" e nenhuma porcentagem.

A linha quatro é **o recurso de crédito** dentro de uma maquininha ou sistema de ponto de venda que o mercadinho já aluga. A linha cinco é **uma carteira ou ferramenta para lojistas em stablecoin** que já liquida em stablecoin e poderia acrescentar um fiado. As linhas quatro e cinco precisam de um nome, uma página de preços e uma leitura de última atividade de páginas que você abre hoje:

```text
mapa de concorrentes, Fiado, 2026-09-06
linha  quem                                acerta                          erra                           cobra                última atividade
1      a caderneta de papel                grátis, privada, sem celular    perde, molha, dona cobra       nada                 hoje, todo mercadinho
2      planilha / conversa de WhatsApp     sobrevive à água, pesquisável   ainda cobrada na mão           nada                 hoje
3      app BNPL: Pagaleve                  assume o risco de calote, +10.000 lojas (site, 2026-09-07)   <das avaliações, com data>   não impresso, cotação sob consulta (pagaleve.com.br/varejistas, 2026-09-07)   <listagem na loja, com data>
4      crédito no POS: <nome>              <do site dele, com data>        <das avaliações, com data>     <página de preços, com data>  <listagem na loja, com data>
5      ferramenta stablecoin: <nome>       <do site dele, com data>        <das avaliações, com data>     <página de preços, com data>  <página do programa no explorer, com data>
```

Os colchetes angulares são seus para remover. Uma linha que ainda tiver um na sexta-feira *é uma linha sobre a qual o jurado vai perguntar*.

![O mapa do Fiado tem as linhas da caderneta e da planilha preenchidas com o que o time sabe, uma linha de BNPL preenchida pela metade a partir das páginas datadas da Pagaleve, e as linhas de POS e stablecoin deixadas como espaços datados.](assets/v01-table.webp)

4. **Leia a linha cinco direto da chain** primeiro, *já que ela não precisa da permissão de ninguém*. Um programa na Solana tem uma página em qualquer explorer, com uma contagem de transações e a hora da última transação, sem login. **Copie três coisas** para a linha: a contagem total, a contagem ou lista dos últimos trinta dias onde aparecer, e a hora da transação mais recente, depois a URL e a data.

Agora leia o que você copiou. Uma contagem total grande com a última transação semanas atrás é um produto que **teve usuários e os perdeu**. Uma contagem pequena com uma transação uma hora atrás é *um produto que está vivo e no começo*. Nenhum dos dois é o que o site do próprio projeto diz. Um comparável, ou seja, um projeto que faz um trabalho parecido para um usuário parecido, também pode publicar o **TVL**, o valor que as pessoas depositaram nele, e esse número com a data diz quanto dinheiro as pessoas já confiam a esse tipo de produto.

Com a aba aberta, **faça a leitura da stablecoin** para o dimensionamento: a página do token mostra uma contagem de holders e as transferências recentes. A contagem de holders entra como **um limite superior de pessoas**, *porque uma pessoa tem várias carteiras e um bot tem milhares*, e as transferências dos últimos trinta dias entram como a leitura de atividade, as duas com a data. Se o emissor publica um número regional, copie esse também e marque de que página veio, porque o número do emissor e o do explorer não vão bater.

Para as linhas três e quatro **a leitura é off-chain**: uma listagem na loja mostra a data da última atualização, um repositório público mostra o último commit, uma página de preços mostra o preço e a data em que você leu, e a data da avaliação mais recente é um sinal de última atividade. Uma side track ou bounty da Earn também serve, quando a listagem mostra **quantas pessoas se candidataram**, *já que essa contagem diz quantos builders acreditaram que o problema do patrocinador valia a semana deles*.

![A célula de última atividade on-chain é construída copiando de um explorer a contagem total de um programa, a contagem recente e a hora da última transação, cada uma com a data da leitura.](assets/v02-table.webp)

5. **Escreva os três números como aritmética** antes de qualquer valor entrar. TAM é o mercado total endereçável, todo mundo que tem o problema. SAM é a parte que você realmente conseguiria atender com este produto, neste lugar, neste trilho. SOM é a parte disso que você consegue alcançar nos primeiros meses, por um canal que você consegue nomear.

A Colosseum pontua Potential Market Size com as três perguntas no topo do seu arquivo. Um hackathon sazonal ou uma side track vai ter sua própria página com suas próprias entregas e seu próprio prazo. **Leia essa página no dia em que decidir entrar**, e reaproveite o pacote que você já tem. O slide é escrito na semana 4. A aritmética é escrita hoje, sob uma regra: **nenhum número sem as entradas**, e nenhuma entrada sem URL e data. A aritmética do Fiado:

```text
dimensionamento, Fiado, 2026-09-06

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

Toda letra é uma página. A é uma contagem pública e **um limite superior**, já que o número da ABRAS conta toda loja de varejo alimentar e não só mercadinhos, então o recorte para mercadinhos vai ao lado como estimativa com uma faixa. B não tem página e é estimado a partir de conversas na próxima lição, então escreva como faixa agora com "estimado" ao lado. C e D são parcelas, e *uma parcela precisa de duas contagens, a de cima e a de baixo, as duas datadas*.

E é **o menor número do arquivo** e o que o jurado lembra. Eu trato uma estimativa como *uma previsão que carrega incerteza, não um compromisso*, e é por isso que B e C entram no arquivo como um mínimo e um máximo com o raciocínio ao lado, em vez de um único número confiante.

![O TAM do Fiado é mercadinhos vezes crédito por mercadinho, o SAM estreita isso por duas parcelas datadas, e o SOM é os mercadinhos que uma cooperativa nomeada alcança até o mês três.](assets/v03-flowchart.webp)

6. **Nomeie o canal do mês três** e copie a contagem dele. Ninguém consegue alcançar uma porcentagem do SAM. Uma cooperativa ou uma associação de comerciantes do bairro pode ser alcançada, *por uma pessoa, com um telefone*: ela tem uma página, a página tem **um número de membros ou uma lista**, e essa contagem com a URL e a data é E. Escreva a linha do SOM embaixo da aritmética exatamente neste formato:

```text
SOM, Fiado: <E> mercadinhos, alcançáveis por <nome da associação, página dela, data>,
até o mês três, porque <a pessoa que vai nos apresentar, e o que ela combinou>
```

A última cláusula é *o trabalho da semana que vem*. A contagem é de hoje.

Para a segunda pergunta, direção, **copie a página de adoção de stablecoin** em D duas vezes, hoje e na data anterior mais antiga que a página ou um arquivo mostrar, e escreva as duas leituras lado a lado. Se a página não tem leitura anterior, escreva "uma leitura, sem tendência" e **não invente uma inclinação**. A terceira pergunta, mecanismo, quer uma frase, e para o Fiado ela diz: *cada mercadinho que abre um fiado traz os fregueses para o trilho*, então a contagem de adoção em D cresce pelo número de fregueses do mercadinho cada vez que E cresce em um.

![O arquivo é construído nomeando o dono da concorrência, preenchendo cinco linhas a partir de páginas datadas, escrevendo as fórmulas de dimensionamento com entradas em letras e nomeando o canal do mês três.](assets/v04-flowchart.webp)

## Está pronto quando

- **competitor-map-sizing.md** está ao lado do memorando de ideias, com a data na linha um e as três perguntas da Colosseum embaixo.
- O mapa tem **cinco linhas**, as mais próximas pelo problema do usuário, cada uma com acerta, erra, cobra e última atividade, e toda célula de última atividade aponta para uma página de explorer, uma listagem, um repositório ou uma contagem datada.
- **TAM, SAM e SOM** estão escritos como fórmulas com entradas em letras, e toda entrada aponta para uma URL e uma data ou está marcada como estimada com um mínimo e um máximo.
- **SOM é uma contagem** alcançável nos três primeiros meses por um canal com nome, número de membros e página. Uma parcela do SAM nessa linha reprova, *por mais razoável que a parcela pareça*.

## Fique atento

- **Um número de tamanho de mercado sem a aritmética** é questionado, toda vez, e "um relatório disse" é a resposta que encerra a conversa.
- **Uma carteira não é um usuário**, então uma contagem de endereços ou de holders entra como limite superior de pessoas, nunca como contagem de clientes.
- **Um press release** é o concorrente dizendo o que ele quer que você pense, e nunca preenche a célula de última atividade.

Um TAM grande sem caminho até os primeiros cem usuários pontua pior do que **um pequeno e alcançável**, já que Traction (tração) e Viability ficam ao lado de Potential Market Size na lista e um SOM de mês três alimenta os três, então *a honestidade custa o slide dramático e compra a entrevista*.

## O que fica

Potential Market Size faz três perguntas, e um número grande responde só a primeira. Todo número no seu arquivo é **uma contagem em uma página pública com URL e data**, a última atividade de um concorrente é lida em um explorer ou em uma listagem e nunca em um anúncio, e uma contagem de carteiras é um limite superior de pessoas, nunca uma contagem de clientes. *SOM é o menor número do arquivo, alcançável por um canal com nome, e é o que o jurado lembra.*

## A seguir

Você agora tem um mapa de concorrentes e três números, e os estimados no arquivo, B e C para o Fiado, são faixas esperando por evidência. A próxima lição abre **escrevendo a única suposição** que, se estiver errada, torna o arquivo inteiro inútil, e o teste mais barato que poderia provar que ela está errada até sexta-feira. *Você passa o resto da semana tentando matar a própria ideia*, e reescreve o pitch a partir do que sobreviver.
