# O time no dia 0

Na lição passada você escreveu o brief da Colosseum: sete fatores citados da página ao vivo, quatro perguntas mapeadas sobre eles, o prazo convertido para o seu fuso horário. *Deixe ele aberto.* O bloco de desclassificação dele ganha **o nome de uma pessoa** ao lado antes de você terminar.

## Por que isso importa

O mês que perde tem um formato que já vi muitas vezes: o mês vai todo para o código. A apresentação é planejada perto do fim, sem mês sobrando para melhorá-la, e o projeto foi mostrado a pouca gente de fora do time, então o feedback que teria consertado o pitch chega tarde demais. *A solução não é um deck melhor na semana 4.* É uma frase no dia 0 e uma pessoa cujo trabalho é continuar reescrevendo essa frase.

A ideia única: o mês é **planejado de trás para frente a partir do pacote**, e o pitch é um artefato desde o dia 0. Abra um arquivo novo chamado **team-card.md** e coloque a data de hoje na primeira linha. Ele vence *antes de o relógio começar*, seja qual for a sua temporada.

## Faça isto

1. **Crie o repositório do toolkit**: faça uma pasta com o nome do seu projeto e inicialize o git dentro dela.

```bash
mkdir <your-project> && cd <your-project>
git init
```

**Salve o team-card.md na raiz e faça o commit** antes de encerrar o dia, *e cada artefato que o curso produzir daqui em diante vive neste repositório*.

2. **Instale o kit**. O curso roda no Claude Code com o plugin Solana AI Kit de solanabr/solana-ai-kit, e o comando de instalação é o que está no README do kit, lido no dia em que você o executa. Lido em 2026-09-07, a pasta plugin/skills do kit trazia **três skills**, hackathon, idea-sprint e pitch-deck, no commit 353e9a1 de 2026-08-20. *O kit muda entre edições, então o README vence esta lição.* Abra o Claude Code dentro do repositório e peça para ele **listar as skills do kit**. Se esses três nomes voltarem, o kit está instalado. Se não, o README é a próxima página.

3. **Coloque fundos em uma carteira na devnet**. Crie uma com qualquer app de carteira ou com a CLI da Solana, mude para a devnet e coloque SOL nela pelo faucet da devnet que o README do kit indica. O USDC de devnet vem da fonte que o README do kit lista, lida no dia em que você abastecer, em 2026-09-07. Leia **dois saldos** na devnet, SOL para as taxas e USDC de devnet para o fiado. Se qualquer um deles estiver em zero, resolva agora. *O mês não tem espaço para isso depois.*

4. **Coloque quatro papéis em nomes** no team-card.md, um nome ao lado de cada função. O builder entrega a fatia. O dono da concorrência descobre o que os rivais fazem certo e o que fazem errado, por escrito até o **fim da semana 1**. O dono das parcerias vende a ideia para projetos adjacentes antes do demo day, *para o time chegar já com alguns parceiros do seu lado*. O storyteller é dono da frase do pitch e do log de feedback e **reescreve a frase toda semana** até ela ser a primeira linha do deck.

Um time de duas pessoas ainda nomeia quatro funções, *porque as quatro perguntas do seu brief não encolhem quando o time encolhe*. Atribua primeiro os papéis que **só uma pessoa consegue fazer**, depois os dobrados, e escreva as semanas em que um papel dobrado vai ficar desassistido. Sozinho, você é dono dos quatro, e *a linha honesta é dizer quais dois recebem uma hora por semana*. Os papéis do Fiado, com o desassistido marcado:

![O cartão de duas pessoas do Fiado nomeia um dono para cada um dos quatro papéis, dobra dois por pessoa e marca parcerias como desassistido nas semanas 2 e 3.](assets/v01-table.webp)

Depois, os campos do portal. Na temporada World's Fair de 2026, a submissão da Colosseum pede cada integrante com sua formação e experiência anterior, e a localização do time. Cada integrante precisa **criar uma conta**. O líder do time adiciona os outros durante a submissão e precisa completar a submissão **antes do prazo**. *Participação solo é permitida.* Então o cartão carrega uma linha de formação por pessoa, a localização e **a palavra líder** ao lado de um nome, copiada hoje para o bloco de desclassificação do seu brief.

Um hackathon sazonal ou uma side track vai ter sua própria página com suas próprias entregas e seu próprio prazo. **Leia essa página no dia em que decidir entrar**, e reaproveite o pacote que você já tem. O cartão completo do Fiado:

```text
team-card.md, Fiado, dia 0, escrito em 2026-09-06
pitch v0: Uma dona de mercadinho que perde a caderneta perde quarenta dívidas pequenas,
          então o Fiado guarda o fiado on-chain e lembra cada freguês dela.

papel                  dono    dobrado com             nota
builder                Ana     dona das parcerias      entrega a fatia na devnet, semanas 2 e 3
dono da concorrência   Bruno   storyteller             mapa por escrito até 2026-09-20
dona das parcerias     Ana     builder                 DESASSISTIDO nas semanas 2 e 3; ligações na semana 1 e de 2026-10-05 a 10-07
storyteller            Bruno   dono da concorrência    dono do pitch v0 e do log de feedback, reescreve toda semana

líder:       Bruno (completa a submissão antes do prazo que está no brief)
localização: <cidade, país, como o portal pede>
formação:    Ana, desenvolvedora web, três anos entregando ferramentas para pequenos negócios, cresceu em um mercadinho
             Bruno, escreve profissionalmente, primeiro hackathon
```

5. **Planeje o mês de trás para frente**, embaixo do cartão. **Escreva a última data primeiro**: a data de encerramento da temporada que está no seu brief, com a hora relida na página ao vivo e convertida. Segundo a página em 2026-09-06, o hackathon World's Fair vai de **14 de setembro a 12 de outubro de 2026**, o que dá 28 dias, quatro semanas exatas. Do prazo, ande para trás uma semana de história, duas semanas de fatia, uma semana de evidência, e antes disso, agora. Derive os limites a partir da data de início *em vez de digitá-los*:

```bash
python3 -c "from datetime import date, timedelta; s = date(2026, 9, 14); print(*[s + timedelta(days=7*i) for i in range(5)], sep='\n')"
```

A primeira linha impressa é o dia em que o relógio começa, a última é o dia do prazo, e as três do meio são **os limites das fases**. Se a página ao vivo mover uma data, *mude a única data dentro do comando e todas as linhas abaixo se movem junto*. **Coloque as cinco datas** em cinco fases, nomeie os artefatos com que cada fase termina, e deixe a linha do prazo apontar para o brief para a hora. Na linha de antes do relógio, acrescente **três linhas datadas** para os passos 1 a 3. As do Fiado: repositório inicializado em 2026-09-06, kit instalado no mesmo dia, carteira de devnet da Ana com SOL e USDC de devnet, conferida em 2026-09-06. O plano dele é a linha do tempo abaixo, cinco linhas.

![O mês da World's Fair vai de antes do relógio, passando pela semana 1, semanas 2 e 3, semana 4 e submeter em 2026-10-12, cada fase datada e produzindo artefatos nomeados.](assets/v02-timeline.webp)

6. **Escreva o pitch v0** embaixo da data. Uma frase, o problema antes do produto, com menos de **25 palavras**: uma pessoa primeiro, depois a coisa que essa pessoa não consegue fazer hoje, depois o seu produto. A versão produto-primeiro do Fiado é "Fiado é uma conta de fiado on-chain para mercadinhos com liquidação em stablecoin e lembretes automáticos", dezesseis palavras, *todas verdadeiras, e sem ninguém dentro*. A v0 no topo do cartão tem 24 palavras, e as primeiras oito são uma dona de mercadinho e uma caderneta perdida, seguidas de quarenta dívidas pequenas.

**Conte as palavras**. Se a primeira palavra for o seu produto, guarde essa linha como registro do dia 0 e escreva uma segunda embaixo que comece com uma pessoa. *Vai ser uma frase ruim.* Ela existe, e isso é todo o requisito de uma v0.

7. **Defina a cadência de feedback**. Um log de feedback é um arquivo com uma data, um nome e o que aquela pessoa não entendeu. Mostre a frase para **três pessoas por semana** que não estão no cartão, e anote, nas palavras delas, a parte que não entenderam. *O que elas gostaram não vai a lugar nenhum.* Uma resposta de "legal" não é uma entrada do log, então pergunte do que a frase trata e anote o que elas respondem. Coloque os três nomes desta semana **hoje**, linhas vazias e tudo. Os três do Fiado estão a três distâncias do problema:

```text
feedback-log.md
data         nome, fora do time                                        o que não entendeu, nas palavras da pessoa
2026-09-07   Lucia, a dona do mercadinho, tia da Ana                   (frase mostrada esta semana)
2026-09-07   Jorge, um freguês que tem fiado                           (frase mostrada esta semana)
2026-09-08   Marcos, entrou em um hackathon uma vez, nunca teve fiado  (frase mostrada esta semana)
```

**Mande a sua frase** para os seus três hoje. As três reações do Fiado vão estar no log até **2026-09-13**, *e é por isso que a v1 vence no fim da semana 1*.

![Um arquivo de dia 0 é construído copiando o prazo e o líder do brief, derivando as datas das fases, nomeando quatro papéis, escrevendo o pitch v0 e registrando três reações de fora.](assets/v03-flowchart.webp)

## Está pronto quando

- **Quatro papéis** têm dono nomeado, e o nome do líder está no bloco de desclassificação do seu brief.
- Cada fase carrega **uma data que o comando imprimiu**, e a linha do prazo aponta para o brief para a hora.
- O pitch v0 tem **menos de 25 palavras** com a pessoa antes do produto.
- O log de feedback tem **três entradas datadas** nas palavras das próprias pessoas.

## Fique atento

- **Um plano sem datas** é uma lista de boas intenções, e o prazo é publicado em UTC, então a linha convertida da lição passada vai para o plano.
- Uma frase que **começa com o produto** pede para quem ouve se importar com um nome que nunca ouviu.
- Um time de duas pessoas em que o dono da concorrência também é o único builder, e **ninguém diz isso**, chega com um mapa raso na semana 1. Diga isso no cartão e date em volta.

O storyteller **custa um builder**, um quarto das horas de construção em um time de quatro, *e é a troca certa*, porque o código aparece em pouquíssimas das sete linhas do seu brief e o trabalho do storyteller aparece na maioria das outras.

## O que fica

O mês é planejado de trás para frente a partir do pacote, então **o prazo é a primeira data do plano** e cada fase é derivada dele. Quatro papéis têm dono nomeado mesmo em um time de dois, porque as quatro perguntas do seu brief não encolhem quando o time encolhe. *O pitch existe desde o dia 0 como uma frase que nomeia uma pessoa antes do produto, e três pessoas de fora do time já disseram o que não entenderam.*

## A seguir

A semana 1 começa na próxima lição, e a primeira coisa que você faz é **jogar fora a ideia** com que chegou, *ou provar que ela merece ficar*. Você coloca essa ideia ao lado de dois outros problemas que você, ou pessoas perto de você, realmente têm, e pontua os três até um sobreviver. A lição 4 abre o Claude Code dentro deste repositório, e a frase no seu arquivo é o primeiro candidato.
