# O time no dia 0

Na lição passada você escreveu o brief da Colosseum: sete fatores copiados da página ao vivo, as quatro perguntas distribuídas entre eles, o prazo convertido para o seu fuso. *Deixe ele aberto.* Antes de esta lição acabar, o bloco de desclassificação dele ganha **o nome de uma pessoa** ao lado.

## Por que isso importa

O mês que perde tem um formato que já vi muitas vezes: o mês inteiro vai para o código. A apresentação só é pensada perto do fim, sem tempo sobrando para melhorar, e o projeto foi mostrado para pouca gente de fora do time, então o feedback que teria consertado o pitch chega tarde demais. *A solução não é um deck melhor na semana 4.* É uma frase no dia 0 e uma pessoa cujo trabalho é reescrever essa frase sem parar.

A ideia única desta lição: o mês é **planejado de trás para frente, a partir do material de submissão**, e o pitch é uma entrega desde o dia 0. Abra um arquivo novo chamado **team-card.md** e coloque a data de hoje na primeira linha. O prazo dele é *antes de o relógio começar*, seja qual for a sua temporada.

## Mão na massa

1. **Crie o repositório do toolkit**: crie uma pasta com o nome do projeto e inicialize o git dentro dela.

```bash
mkdir <your-project> && cd <your-project>
git init
```

**Salve o team-card.md na raiz e faça o commit** antes de fechar o dia, *e tudo que o curso produzir daqui em diante mora neste repositório*.

2. **Instale o kit**. O curso roda no Claude Code com o plugin Solana AI Kit, de solanabr/solana-ai-kit, e o comando de instalação é o que estiver no README do kit no dia em que você rodar. Em 2026-09-07, a pasta plugin/skills do kit tinha **três skills**, hackathon, idea-sprint e pitch-deck, no commit 353e9a1 de 2026-08-20. *O kit muda de uma edição para outra, então o README vale mais do que esta lição.* Abra o Claude Code dentro do repositório e peça para ele **listar as skills do kit**. Se voltarem esses três nomes, o kit está instalado. Se não, a próxima página é o README.

3. **Abasteça uma carteira na devnet**. Crie uma em qualquer app de carteira ou pela CLI da Solana, mude para a devnet e pegue SOL no faucet de devnet que o README do kit indica. O USDC de devnet vem da fonte que o README lista, conferida no dia em que você abastecer, como estava em 2026-09-07. Confira **dois saldos** na devnet, SOL para as taxas e USDC de devnet para o fiado. Se algum estiver zerado, resolva agora. *Depois o mês não tem espaço para isso.*

4. **Dê nome aos quatro papéis** no team-card.md, um nome ao lado de cada função. O builder entrega o recorte da demo. O responsável pela concorrência descobre o que os concorrentes fazem bem e o que fazem mal, por escrito até o **fim da semana 1**. O responsável por parcerias vende a ideia para projetos vizinhos antes do demo day, *para o time chegar lá já com alguns parceiros do seu lado*. O storyteller cuida da frase do pitch e do log de feedback, e **reescreve a frase toda semana** até ela virar a primeira linha do deck.

Um time de duas pessoas continua nomeando quatro papéis, *porque as quatro perguntas do brief não encolhem quando o time encolhe*. Distribua primeiro os papéis que **só uma pessoa consegue fazer**, depois os que vão ser acumulados, e anote em quais semanas um papel acumulado vai ficar descoberto. Sozinho, os quatro são seus, e *a linha honesta diz quais dois vão receber só uma hora por semana*. Os papéis do Fiado, com o descoberto marcado:

![O cartão do Fiado, de duas pessoas, dá um responsável a cada um dos quatro papéis, acumula dois por pessoa e marca parcerias como descoberto nas semanas 2 e 3.](assets/v01-table.webp)

Agora os campos do portal. Na temporada World's Fair de 2026, a submissão da Colosseum pede a formação e a experiência anterior de cada integrante, e a localização do time. Cada integrante precisa **criar uma conta**. O líder do time adiciona os outros na hora de submeter e precisa concluir a submissão **antes do prazo**. *Dá para participar sozinho.* Por isso o cartão tem uma linha de formação por pessoa, a localização e **a palavra líder** ao lado de um nome, copiada hoje para o bloco de desclassificação do brief.

Um hackathon sazonal ou uma side track tem página própria, com entregas e prazo próprios. **Leia essa página no dia em que decidir entrar**, e reaproveite o material de submissão que você já tem. O cartão completo do Fiado:

```text
team-card.md, Fiado, dia 0, escrito em 2026-09-06
pitch v0: Uma dona de mercadinho que perde a caderneta perde quarenta dívidas pequenas,
          então o Fiado guarda o fiado on-chain e lembra cada freguês dela.

papel                          responsável   acumula com     nota
builder                        Ana           parcerias       entrega o recorte da demo na devnet, semanas 2 e 3
responsável pela concorrência  Bruno         storyteller     mapa por escrito até 2026-09-20
responsável por parcerias      Ana           builder         DESCOBERTO nas semanas 2 e 3; ligações na semana 1 e de 2026-10-05 a 10-07
storyteller                    Bruno         concorrência    cuida do pitch v0 e do log de feedback, reescreve toda semana

líder:       Bruno (conclui a submissão antes do prazo que está no brief)
localização: <cidade, país, como o portal pede>
formação:    Ana, desenvolvedora web, três anos entregando ferramentas para pequenos negócios, cresceu em um mercadinho
             Bruno, escreve profissionalmente, primeiro hackathon
```

5. **Planeje o mês de trás para frente**, embaixo do cartão. **A primeira data que você escreve é a última**: a data de encerramento da temporada que está no brief, com a hora relida na página ao vivo e convertida. Pela página em 2026-09-06, o hackathon World's Fair vai de **14 de setembro a 12 de outubro de 2026**, 28 dias, quatro semanas cravadas. A partir do prazo, volte uma semana de história, duas semanas de recorte, uma semana de evidência, e antes disso, o agora. Calcule os limites a partir da data de início *em vez de digitar cada um*:

```bash
python3 -c "from datetime import date, timedelta; s = date(2026, 9, 14); print(*[s + timedelta(days=7*i) for i in range(5)], sep='\n')"
```

A primeira linha impressa é o dia em que o relógio começa, a última é o dia do prazo, e as três do meio são **os limites das fases**. Se a página ao vivo mudar uma data, *mude só a data dentro do comando e todas as linhas mudam junto*. **Distribua as cinco datas** em cinco fases, diga com que entregas cada fase termina, e deixe a linha do prazo apontar para o brief para a hora. Na linha de antes do relógio, acrescente **três linhas com data** para os passos 1 a 3. As do Fiado: repositório criado em 2026-09-06, kit instalado no mesmo dia, carteira de devnet da Ana com SOL e USDC de devnet, conferida em 2026-09-06. O plano dele é a linha do tempo abaixo, cinco linhas.

![O mês da World's Fair vai de antes do relógio, passando pela semana 1, semanas 2 e 3, semana 4 e submeter em 2026-10-12, cada fase com data e produzindo entregas com nome.](assets/v02-timeline.webp)

6. **Escreva o pitch v0** embaixo da data. Uma frase, o problema antes do produto, com menos de **25 palavras**: primeiro uma pessoa, depois o que essa pessoa não consegue fazer hoje, depois o seu produto. A versão do Fiado que começa pelo produto é "Fiado é uma conta de fiado on-chain para mercadinhos com liquidação em stablecoin e lembretes automáticos", dezesseis palavras, *todas verdadeiras, e sem ninguém dentro*. A v0 no topo do cartão tem 24 palavras, e as primeiras oito são uma dona de mercadinho e uma caderneta perdida, seguidas de quarenta dívidas pequenas.

**Conte as palavras**. Se a primeira palavra for o nome do produto, guarde essa linha como registro do dia 0 e escreva outra embaixo, começando por uma pessoa. *Vai ser uma frase ruim.* Ela existe, e é só isso que uma v0 precisa.

7. **Defina o ritmo do feedback**. Um log de feedback é um arquivo com data, nome e o que aquela pessoa não entendeu. Mostre a frase para **três pessoas por semana** que não estão no cartão, e anote, nas palavras delas, a parte que não entenderam. *O que elas gostaram não entra.* Um "legal" não é entrada de log, então pergunte do que a frase fala e anote a resposta. Coloque os três nomes desta semana **hoje**, mesmo com as linhas vazias. Os três do Fiado estão a três distâncias diferentes do problema:

```text
feedback-log.md
data         nome, fora do time                                        o que não entendeu, nas palavras da pessoa
2026-09-07   Lucia, a dona do mercadinho, tia da Ana                   (frase mostrada esta semana)
2026-09-07   Jorge, um freguês que tem fiado                           (frase mostrada esta semana)
2026-09-08   Marcos, entrou em um hackathon uma vez, nunca teve fiado  (frase mostrada esta semana)
```

**Mande a sua frase** para os seus três hoje. As três reações do Fiado estarão no log até **2026-09-13**, *e é por isso que a v1 tem prazo no fim da semana 1*.

![Um arquivo de dia 0 é montado copiando o prazo e o líder do brief, calculando as datas das fases, nomeando quatro papéis, escrevendo o pitch v0 e registrando três reações de fora.](assets/v03-flowchart.webp)

## Está pronto quando

- **Os quatro papéis** têm um responsável com nome, e o nome do líder está no bloco de desclassificação do brief.
- Cada fase tem **uma data que saiu do comando**, e a linha do prazo aponta para o brief para a hora.
- O pitch v0 tem **menos de 25 palavras**, com a pessoa antes do produto.
- O log de feedback tem **três entradas com data**, nas palavras das próprias pessoas.

## Fique atento

- **Um plano sem datas** é uma lista de boas intenções, e o prazo é publicado em UTC, então a linha convertida da lição passada vai para o plano.
- Uma frase que **começa pelo produto** pede que quem ouve se importe com um nome que nunca ouviu.
- Um time de duas pessoas em que o responsável pela concorrência também é o único builder, e **ninguém fala isso**, chega na semana 1 com um mapa raso. Escreva isso no cartão e marque as datas em volta.

O storyteller **custa um builder**, um quarto das horas de construção em um time de quatro, *e é a troca certa*, porque o código aparece em pouquíssimas das sete linhas do brief e o trabalho do storyteller aparece na maioria das outras.

## O que fica

O mês é planejado de trás para frente, a partir do material de submissão, então **o prazo é a primeira data do plano** e cada fase sai dele. Os quatro papéis têm responsável com nome mesmo em um time de dois, porque as quatro perguntas do brief não encolhem quando o time encolhe. *O pitch existe desde o dia 0 como uma frase que nomeia uma pessoa antes do produto, e três pessoas de fora do time já disseram o que não entenderam.*

## A seguir

A semana 1 começa na próxima lição, e a primeira coisa que você faz é **jogar fora a ideia** com que chegou, *ou provar que ela merece ficar*. Você coloca essa ideia ao lado de dois outros problemas que você, ou gente perto de você, tem de verdade, e pontua os três até sobrar um. A lição 4 abre o Claude Code dentro deste repositório, e a frase do seu arquivo é o primeiro candidato.
