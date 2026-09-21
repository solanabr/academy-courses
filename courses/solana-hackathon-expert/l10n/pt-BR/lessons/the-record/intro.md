# O registro

## O arquivo que a maioria dos times nunca abriu

Em algum lugar de um arquivo JSON público existe uma linha dizendo que a Superteam Brasil colocou **dez mil dólares na mesa** para times brasileiros na última temporada global. Meu palpite é que a maioria dos times que poderiam ter disputado esse prêmio **nunca abriu o arquivo**. Esta lição mostra o que tem nele, *para você começar o mês sabendo o que os times que pularam essa etapa não sabiam*.

Aqui está a linha da Frontier, a última temporada global concluída, consultada em 2026-09-06:

```text
temporada     Frontier (a última temporada global concluída)
side tracks   54
total         US$439,410
uma linha     Side Track Superteam Brasil, US$10,000 USDG
```

Nada disso estava escondido. Ficou atrás de uma URL pública a temporada inteira: uma entrada por side track que a Superteam Earn rodou em paralelo à temporada Frontier da Colosseum, cada uma com um patrocinador, um valor e um pedido. Se um dia você quiser ver a versão ao vivo com seus próprios olhos, um único comando devolve a lista inteira, e *você não vai precisar dele mais do que uma ou duas vezes por temporada*:

```bash
curl -sL https://superteam.fun/api/hackathon/frontier | python3 -m json.tool
```

O que os times que leram o arquivo sabiam é simples: uma temporada da Colosseum paga de **mais de um bolso**, um prêmio principal e uma lista longa de prêmios regionais, cada um com seu próprio patrocinador e seu próprio pedido, *disputados com a mesma submissão*. Um mês de hackathon tem um registro, e **o registro é público**.

![Um único endpoint devolve as 54 side tracks da Frontier, somando US$439,410, filtradas até a linha de US$10,000 USDG da Superteam Brasil, enquanto links com o prefixo /earn/ terminam em 404.](assets/v01-flowchart.webp)

## O time que perdeu três vezes primeiro

Abra colosseum.com/hackathon no navegador e **encontre o card da Unruggable**. Ele diz que o time competiu em quatro hackathons antes de ganhar o grande prêmio: Renaissance, uma menção honrosa. Radar, outra menção honrosa. Breakout, primeiro lugar em uma track. Cypherpunk, o grande prêmio. Esse é o registro como o card da Colosseum o apresentava em 2026-09-08, e a mesma página, lida em 2026-09-06, dizia que a Colosseum organiza **dois hackathons globais por ano**, de abril a maio e de setembro a outubro. Um time que trata uma temporada como veredito *está jogando fora a segunda metade do ano*.

![A Unruggable levou menções honrosas na Renaissance e na Radar, primeiro lugar em uma track da Breakout e depois o grande prêmio da Cypherpunk, quatro temporadas a duas por ano.](assets/v02-timeline.webp)

## Duas temporadas, lado a lado

A temporada anterior à Frontier conta a outra metade da história. Troque uma palavra naquele comando, frontier por cypherpunk, e o endpoint responde pela temporada anterior. Consultada em 2026-09-06, a Cypherpunk trazia **41 side tracks** somando US$341,750, e a linha brasileira dizia Superteam Brasil x Tangem Wallet, com **US$11,200**. Entre as duas temporadas *a contagem subiu* e a linha brasileira passou de uma track co-patrocinada para uma de patrocinador único, com valor menor. Troque a palavra por worldsfair e, na mesma data, o endpoint devolveu **uma lista vazia**: nenhuma side track publicada ainda para uma temporada que não tinha aberto. *Uma resposta vazia também faz parte do registro*, e ela diz que a página merece uma segunda olhada assim que a temporada abrir.

O curso carrega um único projeto de exemplo por todas as lições, e é aqui que você o conhece. **Fiado** é a caderneta de fiado de um mercadinho de bairro brasileiro, o nome, o saldo e a data que a dona guarda ao lado do caixa, transformada em um fiado liquidado em stablecoin com lembrete de pagamento. *O Fiado é brasileiro, então a linha regional dele é a da Superteam Brasil*, e as duas temporadas do registro dele ficam assim:

![A Frontier mostra 54 tracks e US$439,410 com uma linha de US$10,000 USDG da Superteam Brasil, a Cypherpunk mostra 41 tracks e US$341,750 com uma linha de US$11,200, e a World's Fair ainda estava vazia.](assets/v03-table.webp)

Duas coisas nessa tabela valem a pena guardar. Primeira, **todo número carrega uma data**, porque o registro diz onde o dinheiro e a multidão estavam na temporada passada e nada de certo sobre onde vão estar nesta. Quando a sua temporada abrir, a página merece *uma olhada nova*, não o hábito de ficar puxando de novo. Segunda, a sua própria região tem uma linha nessa lista, ou uma linha ausente, e o mesmo vale para cada país vizinho, cada um com **seu próprio patrocinador e seu próprio pedido**. Um time brasileiro entrando na Frontier estava em duas disputas ao mesmo tempo com a mesma submissão, uma global e uma só contra o próprio país, e essa assimetria é, na minha opinião, *o fato mais subaproveitado dos hackathons de Solana*. A linha regional é o caminho mais curto até um prêmio, e *o que os jurados da Colosseum pontuam é o que faz um projeto merecer prêmio algum*, então um bom mês segura os dois.

## Quatro palavras, o mês, três regras

**Uma temporada global** é um dos dois hackathons da Colosseum por ano, e o relógio deste curso é o mês entre a abertura e o prazo de submissão. **Uma side track**, a trilha regional, é um prêmio regional atrelado a uma temporada global, um patrocinador, quase sempre uma Superteam regional, colocando um valor e um pedido. **Um prêmio regional** é a side track cujo patrocinador é a Superteam da sua região. **Um ensaio** é qualquer outra competição em que você entra com o mesmo pacote: uma temporada global da Colosseum é o alvo, e um hackathon sazonal da Superteam Brasil antes da próxima temporada da Colosseum, ou uma side track da Superteam Earn ao lado da temporada em que você entra, *é um ensaio*.

A ordem das lições **segue o calendário**. Antes do relógio, você lê a página do hackathon e seus critérios de julgamento, confirma um time, escreve um primeiro pitch e monta o repositório do toolkit no dia 0. A semana 1 leva uma ideia até evidência e uma **decisão escrita de matar, pivotar ou seguir**. As semanas 2 e 3 constroem a fatia demonstrável na devnet. A semana 4 *transforma a fatia em história*. Submeter monta o pacote e escreve um plano para a próxima temporada, e Ensaio, opcional, exporta tudo para os hackathons de ensaio.

No último dia **você tem seis coisas na mão**: uma fatia de demo na devnet, um deck, um vídeo de apresentação de dois a três minutos, um vídeo de demo de três minutos ou menos, um one-pager de GTM com parceiros nomeados e as respostas do portal rascunhadas com antecedência. Cada peça que você constrói antes *alimenta uma dessas seis*.

Três regras da casa:

1. Você vê a coisa **antes de ela ganhar nome**, e cada lição seguinte coloca um comando, uma consulta ou uma página nas primeiras centenas de palavras.
2. **O pitch existe desde o dia 0**, e a primeira versão pode estar errada, *o que ela não pode é não existir*.
3. **Confira cada peça contra os sete fatores de julgamento** e o pedido do patrocinador antes de seguir em frente, e *a próxima peça não começa enquanto a atual não passou com honestidade*.

![O curso percorre seis trechos, antes do relógio, semana 1, semanas 2 e 3, semana 4, submeter e o ensaio opcional, cada um produzindo artefatos nomeados com o pitch versionado a partir da v0.](assets/v04-timeline.webp)

## Próxima lição: os sete fatores

Você conhece a tabela de prêmios. Na próxima lição você **abre a página do hackathon da Colosseum** e lê os sete fatores de julgamento que ela publica, um de cada vez, do jeito que um jurado lê, e copia todos para um arquivo com a data na primeira linha. A primeira coisa que você vai notar é que *qualidade de código não é um deles*.
