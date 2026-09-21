# O deck de hackathon, não o deck de investidor

Na lição passada você escreveu o one-pager de GTM, defendeu o por-que-Solana para a trilha e para a aceleradora e registrou a resposta de um parceiro. Deixe o one-pager aberto e abra do lado dele o log narrativo e o competitor-map-sizing.md, porque esses três arquivos **já são o deck**, só que no formato errado.

## Três minutos e duas crenças

Seu material de submissão é uma aba entre muitas, e o jurado dá **uns três minutos** para ele. Nesses três minutos o jurado precisa entender um problema real e sair acreditando que este é o time que vai resolver. Os vencedores que eu vi tinham a apresentação mais convincente da sala, e alguns tinham menos código do que times que ficaram abaixo deles, porque *um problema que precisa ser resolvido, vendido do jeito certo, ganha de funcionalidade*.

Por isso o deck é **construído do problema para fora**, e a primeira coisa a construir é a ordem. Abra um arquivo novo chamado deck-judge.md, coloque a data na primeira linha e escreva **sete títulos** logo abaixo, um por linha. Todo o resto do dia de hoje vai debaixo dessas sete linhas.

## Mão na massa

1. **Escreva os sete títulos**. Para o Fiado, começado em 2026-10-05, o arquivo fica assim:

```text
deck-judge.md, Fiado, temporada World's Fair da Colosseum, começado em 2026-10-05
1 capa             o problema em uma frase, o nome do produto pequeno
2 problema         a caderneta ao lado do caixa, e quantos caixas existem
3 demo             o fiado na devnet, a transação que um jurado pode abrir, a linha de tração
4 o que é novo     o que o recorte faz que a caderneta e o app do banco não fazem
5 por que Solana   para a trilha e para a aceleradora
6 este time        quem somos, por que este problema, e quem paga
7 próximos passos  o que ainda não foi construído, e o pedido
```

Uma decisão já está tomada aqui: *o nome do produto só aparece depois que o problema foi dito*.

2. **Confira o formato com o kit** antes de preencher. Um deck para jurados tem **de 5 a 7 slides**: um deles é a demo funcionando, um é a novidade técnica, um é o por-que-Solana, e cada slide tem uma nota de apresentação de 30 a 60 segundos. É assim que a skill de pitch-deck do kit trata o público de jurados de hackathon, lido em 2026-09-06. A mesma skill manda um público de VC para 10 a 12 slides e um público de grant ou aceleradora para 8 a 10.

![O kit manda jurados de hackathon para 5 a 7 slides com demo, novidade e por-que-Solana, enquanto decks de VC têm de 10 a 12 e decks de grant ou aceleradora de 8 a 10.](assets/v01-comparison.webp)

3. **Rode a entrevista de 12 perguntas do kit** com os quatro arquivos abertos: competitor-map-sizing.md, o log narrativo, o one-pager com a lista de parceiros e o pitch v2. Antes, **confira a invocação** e a lista de perguntas no README atual do kit, *porque esta lição tem data*. Cada resposta é copiada de um desses arquivos. Uma pergunta que nenhum deles responde ou pertence a um deck de VC ou é *um buraco no material que você achou antes do jurado*. A skill escreve um esboço em markdown, depois os slides com as notas, depois uma autoavaliação e um Q&A de objeções. **Guarde o esboço**, ainda não os slides, e leia a autoavaliação como uma primeira passada, não como nota.

![Quatro arquivos que já existem alimentam a entrevista do kit, que vira um esboço, sete slides, notas, uma autoavaliação e objeções, depois uma rodada de feedback que trava o pitch e, por fim, a redistribuição para a temporada.](assets/v02-flowchart.webp)

4. **Preencha a capa** e o slide do problema. A capa é uma frase, e **a frase é o problema**, com o nome do produto pequeno num canto. Para o Fiado: uma dona de mercadinho mantém quarenta fiados abertos numa caderneta de papel, e a caderneta se perde. Essa frase é a v2 até o passo 11.

Depois **escreva o one-liner**, que é outra coisa, diferente da frase da capa: é a resposta para *o que é isso?* em **cinco palavras ou menos**, para a tagline do formulário de submissão, para o canto da capa ao lado do nome e para as primeiras palavras de um post. Três regras, nesta ordem. **Primeiro, sem ambiguidade**: dez pessoas lendo têm que imaginar o mesmo produto, e *se imaginam dez produtos, a frase falhou*. **Segundo, empolgante**, porque ninguém se empolga com um produto que não entendeu. **Terceiro, verdadeiro o bastante**, ou seja, pinta a imagem certa e os detalhes vêm depois.

Use as palavras que a sua mãe usaria num café. **Sem jargão**, e nada de metáfora de lego, bloco de montar, camada ou cola. Dois formatos funcionam: o análogo, *algo que todo mundo conhece mais o seu toque*, e a descrição simples, um verbo, um objeto e um contexto. **Escreva quatro candidatos e escolha um**, com o motivo ao lado. Os do Fiado:

```text
candidatos de one-liner, Fiado, 2026-10-05
empréstimo para mercadinhos              errado: fiado não é empréstimo, ninguém empresta nada
fiado que se paga sozinho                ambíguo: dez leitores, dez produtos
caderneta de fiado em dois celulares     claro, o formato análogo, seis palavras
fiado visível dos dois lados             escolhido: cinco palavras, uma imagem, verdade hoje
```

O slide do problema **é o que recebe mais tempo**: três linhas curtas sobre o que acontece. Para o Fiado, a caderneta ao lado do caixa, que molha, some ou fica com o saldo aberto de um freguês que se mudou, e a dona correndo atrás de quarenta dívidas pequenas sozinha. Abaixo dessas linhas, uma linha de números: **os três números de dimensionamento** do competitor-map-sizing.md, cada um com a fonte e a data em que foi lido. No deck para jurados *o mercado mora aqui e em nenhum outro lugar*.

5. **Preencha a demo** e o slide do que-é-novo. A demo é **uma captura de tela** da tela do fiado, tirada do log narrativo, com a assinatura da transação na devnet embaixo, *para o jurado poder abrir*. Abaixo vai **a linha de tração**, copiada do traction-log.md com a data. Para o Fiado: quatro mercadinhos rodando fiados reais, 34 fiados abertos, três mercadinhos que voltaram sem ninguém pedir, liquidação na devnet. *Se o log está vazio, o slide diz devnet e para por aí*, e a lacuna vai para a lista de objeções. O que-é-novo é **a novidade técnica em três linhas**, o que o recorte faz que a caderneta e o app do banco não fazem. Para o Fiado: um fiado que os dois lados leem, que liquida em stablecoin, com um lembrete que a dona não precisa mandar. Nada de diagrama de arquitetura aqui.

![O deck para jurados do Fiado vai de capa, problema, demo, novidade, por que Solana, time e próximos passos, e cada número dele aponta para o arquivo de dimensionamento, o log narrativo ou o one-pager.](assets/v03-table.webp)

6. **Preencha o por-que-Solana** e o slide do time. O por-que-Solana é o parágrafo da lição passada **cortado para três linhas**: uma para o que o recorte faz, uma para o que a chain facilita nisso, uma para a aceleradora. Sem número de benchmark.

O slide do time responde ao **Founder + Market Fit** (encaixe entre fundador e mercado). Na página da Colosseum lida em 2026-09-06, esse fator pergunta se o time tem as habilidades e a experiência certas para este mercado e por que está motivado a resolver este problema. Cada pessoa ganha uma linha com o que sabe fazer e uma linha com o porquê deste problema. Para o Fiado a segunda linha é *quem já ficou atrás daquele caixa e viu a caderneta molhar*. Abaixo do time, uma linha: **quem paga**, copiada do one-pager, ou o motivo de ainda não ter preço. Essa é a resposta de Viability (viabilidade).

7. **Preencha os próximos passos e o pedido**. O slide de próximos passos diz **o que ainda não foi construído**, com data, na ordem em que você construiria, e é *o lugar honesto para a arquitetura que você não construiu*. Para o Fiado são o programa de fiado que coloca o valor de abertura e a data de vencimento on-chain ao lado dos pagamentos, os trilhos de fiat, o onboarding de carteira e, depois, a expansão pela cooperativa, tudo o que o cartão de escopo cortou na semana 2. Uma funcionalidade não construída aqui o jurado lê como plano. A mesma funcionalidade no slide da demo ele lê como mentira.

Depois, **o pedido em uma linha**. Para o Fiado: a aceleradora e uma apresentação para uma segunda cooperativa, com nome, usando a resposta do parceiro da lição passada como prova de que a primeira existe. **Leia os sete em voz alta** com um cronômetro e escreva o tempo na primeira linha do arquivo.

8. **Escreva as notas**, de 30 a 60 segundos por slide, como frase falada. Nota em forma de lista de tópicos é lida naquela voz chapada que as pessoas usam para listas, e *o slide do problema morre nessa voz*. A nota do slide do problema do Fiado tem **uns trinta segundos**: a caderneta ao lado do caixa, uns quarenta nomes com o total correndo, o dia em que ela molhou, e os três números na tela com as páginas de onde vieram. O rascunho do kit costuma vir mais longo, então cortar é a maior parte do trabalho.

9. **Escreva o Q&A de objeções**: as perguntas que o jurado vai fazer, cada uma com uma resposta de uma linha, escrita antes de alguém perguntar. **Quatro bastam** para uma primeira lista. As do Fiado, em 2026-10-05:

```text
objections.md, Fiado, 2026-10-05
por que não o app do próprio banco     o banco não conhece o freguês, o mercadinho conhece, e o fiado é entre esses dois
e se a dona perder o celular           o fiado não está no celular, um celular novo lê o mesmo fiado
quem paga                              a linha de quem-paga do one-pager, dita de uma vez só
por que uma chain afinal               a linha de por-que-Solana do slide 5, sem a palavra rápido
```

Escreva as suas antes da rodada de feedback, *porque as três pessoas vão fazer duas dessas perguntas de qualquer jeito*.

10. **Faça a rodada de feedback 3**: o deck lido em voz alta para três pessoas que ainda não viram. Uma tem o problema: uma dona de mercadinho no caso do Fiado e, no seu caso, quem quer que sejam seus primeiros 100 usuários. Uma é builder. Uma não entende nada nem de um nem de outro, e *é essa que te diz se o slide 2 funciona*.

Depois da leitura, peça para cada uma dizer o problema em uma frase e anote a frase que ela disse, *não a que você quis dizer*. Depois pergunte o que é o produto **em cinco palavras** e anote também, *porque um one-liner que dois leitores entendem de jeitos diferentes é ambíguo, não engenhoso*. **Registre três entradas**: quem, o que disse, o que mudou. Se **duas das três** chegam perto da capa, o pitch é final. Se não, reescreva o slide do problema e leia de novo no mesmo dia. Ainda é a rodada 3, só que com mais de três entradas.

11. **Trave o pitch final**. Ele é a frase da capa depois da rodada, e daqui em diante não muda mais, *porque os vídeos da próxima lição são cortados a partir dele e cada mudança depois disso custa uma regravação*. Escreva **final e a data** ao lado dele no log. O do Fiado, depois da rodada 3:

```text
pitch final, Fiado, semana 4, travado em 2026-10-06, final
Uma dona de mercadinho que perde a caderneta perde quarenta dívidas pequenas, então o
Fiado guarda o fiado no celular dela, mostra a cada freguês o mesmo saldo,
recebe parte dele em USDC e manda o lembrete por ela.

rodada 3, dito de volta em uma frase:
Lucia   "o mercadinho que perde a caderneta"                 bateu com a capa
Marcos  "pagar o fiado do mercadinho pelo celular"           bateu com o lado do produto
Jorge   "aquele em que a dona para de correr atrás de você"  bateu com a capa
mudou: nada; dois de três bateram; final
```

Eu já vi pitch pior que o produto perder, e perder no pitch, com o produto melhor indo para casa. É por isso que este curso **reescreve o pitch quatro vezes** antes de um jurado ver.

12. **Redistribua o mesmo arquivo** na exportação para a temporada, se tem um ensaio no seu calendário. Um hackathon sazonal ou uma side track (a trilha regional) vai ter a própria página, com as próprias entregas e o próprio prazo, então **leia essa página no dia em que decidir entrar** e reaproveite o material que você já tem. Nada novo é escrito: **os sete slides se redistribuem** nas seções daquela página, e as notas mantêm as mesmas palavras. Se a página pede um slide de mercado, monte só a partir do competitor-map-sizing.md: os três números, cada um com a fonte e a data em que foi lido. O argumento a favor de pequeno e clicável em vez de grande e sem fonte foi feito na lição de pesquisa de mercado, e o slide não reabre isso. Isso vale para a maioria dos jurados, não para todos, mas *quem pontua um fator de tamanho de mercado está perguntando se o número é real*.

13. **Agora o seu**. O deck do Fiado, copiado para o seu repositório do toolkit, está sem o slide do problema e sem o slide de próximos passos, e você **escreve os dois a partir do log narrativo**: o slide do problema a partir da primeira captura de tela com data do log e da entrada de decisão que nomeou o usuário, o slide de próximos passos a partir das decisões que cortaram escopo na semana 2, cada uma com a data. Depois **o seu próprio deck-judge.md**, preenchido a partir da entrevista e dos quatro arquivos, depois as notas, as objeções, a leitura para três pessoas, a frase da capa marcada como final e, se tem um ensaio no calendário, a exportação para a temporada a partir do mesmo arquivo. Se três pessoas são três agendas, *leia para uma hoje e para duas amanhã e guarde as entradas*. A rodada é as entradas, não o dia.

![O pitch vai da v0, antes de o relógio começar, passando pela v1 na semana 1 e pela v2 nas semanas 2 e 3, até a final na semana 4, com as rodadas de feedback 1, 2 e 3 presas às três últimas versões.](assets/v04-timeline.webp)

## Está pronto quando

- A exportação para jurados tem **menos de oito slides**, cada um com uma nota.
- Se tem um ensaio no calendário, a exportação dele é **o mesmo arquivo redistribuído** nas entregas daquela página, seção por seção.
- Cada número em qualquer slide aponta para **o arquivo de dimensionamento** ou para o pacote de evidências, com o nome do arquivo escrito ao lado.
- Existem **três entradas de feedback** da rodada 3 com quem, o que disse e o que mudou, e a frase da capa está marcada como final, com data.

## Fique atento

- Um slide de título que **nomeia o produto antes do problema**: é o primeiro slide mais comum em hackathon, e *ele gasta o único slide que o jurado lê com olhos frescos*.
- Um slide de mercado com um número que **não está no arquivo de dimensionamento**: um jurado de três minutos passa o olho no slide de mercado, então *ele precisa ser honesto, não grande*, e a hora que você economiza vai para o slide do problema.
- Um slide de time com **três fotos e três cargos** e nenhum motivo para ser este time: perde ponto num fator que tem Founder no nome.

## O que fica

O deck para jurados é **um objeto diferente do deck de investidor**: sete slides construídos do problema para fora, uma nota falada embaixo de cada um, e cada número apontando para um arquivo que você já escreveu. A frase da capa é a quarta versão do pitch, e foi lida para três desconhecidos antes de ser marcada como final. *Se um desconhecido que só viu o slide 2 consegue dizer o seu problema em uma frase, o deck funciona.*

## Próxima lição: os primeiros vinte segundos

Na próxima lição o deck vira dois vídeos, e a primeira ação é **escrever os primeiros vinte segundos** do roteiro da apresentação a partir do slide do problema que você acabou de travar, *porque esses vinte segundos decidem se o jurado assiste ao resto*. Traga as notas, elas são a forma longa do roteiro, e traga a lista de objeções para o vídeo da demo, onde o jurado está pensando nessas perguntas enquanto a transação confirma.
