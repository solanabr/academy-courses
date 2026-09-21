# O registro

## O arquivo que a maioria dos times nunca abriu

Em algum lugar de um arquivo JSON público existe uma linha dizendo que a Superteam Brasil colocou **dez mil dólares na mesa** para times brasileiros na última temporada global. Meu palpite é que a maioria dos times que poderia ter disputado esse prêmio **nunca abriu o arquivo**. Esta lição mostra o que tem nele, *para você começar o mês sabendo o que os times que pularam essa etapa não souberam*.

Esta é a linha da Frontier, a última temporada global concluída, consultada em 2026-09-06:

```text
temporada     Frontier (a última temporada global concluída)
side tracks   54
total         US$439,410
uma linha     Side Track Superteam Brasil, US$10,000 USDG
```

Nada disso estava escondido. Passou a temporada inteira em uma URL pública: uma entrada por side track que a Superteam Earn rodou junto com a temporada Frontier da Colosseum, cada uma com um patrocinador, um valor e um pedido. Se quiser ver a versão ao vivo com os próprios olhos, um comando só devolve a lista inteira, e *você não vai precisar dele mais de uma ou duas vezes por temporada*:

```bash
curl -sL https://superteam.fun/api/hackathon/frontier | python3 -m json.tool
```

O que os times que leram o arquivo sabiam é simples: o dinheiro de uma temporada da Colosseum sai de **mais de um bolso**, um prêmio principal e uma lista longa de prêmios regionais, cada um com seu patrocinador e seu pedido, *disputados com a mesma submissão*. Um mês de hackathon tem um registro, e **o registro é público**.

![Um único endpoint devolve as 54 side tracks da Frontier, somando US$439,410, filtradas até sobrar a linha de US$10,000 USDG da Superteam Brasil, enquanto links com o prefixo /earn/ terminam em 404.](assets/v01-flowchart.webp)

## O time que perdeu três vezes antes de ganhar

Abra colosseum.com/hackathon no navegador e **encontre o card da Unruggable**. Ele diz que o time competiu em quatro hackathons antes de ganhar o grande prêmio: Renaissance, uma menção honrosa. Radar, outra menção honrosa. Breakout, primeiro lugar em uma track. Cypherpunk, o grande prêmio. É esse o registro que o card da Colosseum mostrava em 2026-09-08, e a mesma página, lida em 2026-09-06, dizia que a Colosseum organiza **dois hackathons globais por ano**, de abril a maio e de setembro a outubro. Um time que trata uma temporada como veredito *está jogando fora a segunda metade do ano*.

![A Unruggable levou menções honrosas na Renaissance e na Radar, primeiro lugar em uma track da Breakout e depois o grande prêmio da Cypherpunk, quatro temporadas a duas por ano.](assets/v02-timeline.webp)

## Duas temporadas, lado a lado

A temporada anterior à Frontier conta a outra metade da história. Troque uma palavra naquele comando, frontier por cypherpunk, e o endpoint responde com a temporada anterior. Consultada em 2026-09-06, a Cypherpunk tinha **41 side tracks** somando US$341,750, e a linha brasileira dizia Superteam Brasil x Tangem Wallet, com **US$11,200**. De uma temporada para a outra *o número de tracks subiu*, e a linha brasileira passou de uma track com dois patrocinadores para uma de patrocinador único, com valor menor. Troque a palavra por worldsfair e, na mesma data, o endpoint devolvia **uma lista vazia**: nenhuma side track publicada ainda para uma temporada que não tinha aberto. *Uma resposta vazia também faz parte do registro*, e é ela que avisa que vale voltar à página assim que a temporada abrir.

O curso usa um único projeto de exemplo do começo ao fim, e é aqui que ele aparece pela primeira vez. **Fiado** é a caderneta de fiado de um mercadinho de bairro, aquela em que a dona anota nome, saldo e data ao lado do caixa, transformada em um fiado que liquida em stablecoin e lembra o freguês de pagar. *O Fiado é brasileiro, então a linha regional dele é a da Superteam Brasil*, e as duas temporadas do registro dele ficam assim:

![A Frontier mostra 54 tracks e US$439,410 com uma linha de US$10,000 USDG da Superteam Brasil, a Cypherpunk mostra 41 tracks e US$341,750 com uma linha de US$11,200, e a World's Fair ainda estava vazia.](assets/v03-table.webp)

Duas coisas nessa tabela merecem ficar na cabeça. Primeira: **todo número vem com data**, porque o registro diz onde o dinheiro e a galera estavam na temporada passada, e não garante nada sobre onde vão estar nesta. Quando a sua temporada abrir, vale *uma olhada nova* na página, não o hábito de ficar consultando toda hora. Segunda: a sua região tem uma linha nessa lista, ou uma linha que falta, e o mesmo vale para cada país vizinho, cada um com **seu próprio patrocinador e seu próprio pedido**. Um time brasileiro que entrou na Frontier disputava duas coisas ao mesmo tempo com a mesma submissão, uma global e outra só contra times do próprio país, e essa assimetria é, na minha opinião, *o fato mais mal aproveitado dos hackathons de Solana*. A linha regional é o caminho mais curto até um prêmio, e *o que os jurados da Colosseum pontuam é o que faz um projeto merecer qualquer prêmio*, então um mês bem usado cuida dos dois.

## Quatro palavras, o mês, três regras

**Uma temporada global** é um dos dois hackathons da Colosseum por ano, e o relógio deste curso é o mês entre a abertura e o prazo de submissão. **Uma side track**, a trilha regional, é um prêmio regional atrelado a uma temporada global, com um patrocinador, quase sempre uma Superteam regional, colocando um valor e um pedido. **Um prêmio regional** é a side track cujo patrocinador é a Superteam da sua região. **Um ensaio** é qualquer outra competição em que você entra com o mesmo material de submissão: o alvo é uma temporada global da Colosseum, e um hackathon sazonal da Superteam Brasil antes da próxima temporada, ou uma side track da Superteam Earn rodando junto com a temporada em que você entra, *é um ensaio*.

A ordem das lições **segue o calendário**. Antes do relógio, você lê a página do hackathon e os critérios de julgamento, confirma um time, escreve um primeiro pitch e monta o repositório do toolkit no dia 0. A semana 1 transforma uma ideia em evidência e em uma **decisão escrita de matar, pivotar ou seguir**. Nas semanas 2 e 3 você constrói o recorte da demo na devnet. A semana 4 *transforma esse recorte em história*. Submeter monta o material de submissão e escreve o plano da próxima temporada, e Ensaio, opcional, leva tudo isso para os hackathons de ensaio.

No último dia **você tem seis coisas na mão**: o recorte da demo rodando na devnet, um deck, um vídeo de apresentação de dois a três minutos, um vídeo de demo de três minutos ou menos, um one-pager de GTM com parceiros nomeados e as respostas do portal rascunhadas com antecedência. Cada entrega que você faz antes disso *alimenta uma dessas seis*.

Três regras da casa:

1. Primeiro você vê, **depois vem o nome**, e cada lição daqui em diante põe um comando, uma consulta ou uma página nas primeiras centenas de palavras.
2. **O pitch existe desde o dia 0**, e a primeira versão pode estar errada, *o que ela não pode é não existir*.
3. **Confira cada entrega contra os sete fatores de julgamento** e o pedido do patrocinador antes de seguir, e *a próxima entrega não começa enquanto a atual não passar de verdade*.

![O curso percorre seis trechos, antes do relógio, semana 1, semanas 2 e 3, semana 4, submeter e o ensaio opcional, cada um produzindo entregas com nome e o pitch versionado a partir da v0.](assets/v04-timeline.webp)

## Próxima lição: os sete fatores

Você já conhece a tabela de prêmios. Na próxima lição você **abre a página do hackathon da Colosseum** e lê os sete fatores de julgamento que ela publica, um de cada vez, do jeito que um jurado lê, e copia todos para um arquivo com a data na primeira linha. A primeira coisa que você vai notar é que *qualidade de código não é um deles*.
