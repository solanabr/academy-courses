# Problemas que você tem de verdade

Na lição passada você escreveu o cartão do time, o plano do mês com datas e um pitch v0 de uma frase, e três pessoas já reagiram a ela. **Deixe o log de feedback aberto**, *porque essa frase vai ser testada mais a fundo do que três amigos conseguiriam*.

## Por que isso importa

A semana 1 começa aqui, e a primeira coisa que você faz com a sua ideia é **tentar matar ela**. As boas ideias de hackathon que eu vi ganhar nasceram de um problema que alguém do time, ou alguém próximo, tem ou já teve, e as que começaram de "DeFi para X" quase nunca se classificaram. É essa a diferença entre **começar pelo problema**, uma pessoa e algo que ela não consegue fazer hoje, e começar pela tecnologia, um recurso da chain procurando alguém em quem ser aplicado. Os sete fatores não pontuam tecnologia por si só, e *um jurado que lê as respostas escritas percebe quando a pessoa foi encaixada na semana 4*.

Hoje você gera três ideias que começam pelo problema e pontua as três até sobrar uma. **Abra o Claude Code** dentro do repositório do toolkit que você criou no dia 0, com o kit instalado como na lição 3, e vá para o passo 1.

## Rode o sprint no Fiado

Um lembrete antes do primeiro passo: **o Fiado é o projeto de exemplo do curso**, só isso. É a caderneta de fiado do mercadinho da lição 1, transformada em um fiado que liquida em stablecoin, e todo exercício deste curso roda nele primeiro *para você sempre ter um exemplo resolvido para copiar*. Depois você roda os mesmos passos no seu problema.

1. **Chame a skill idea-sprint** pelo nome, passando o problema do Fiado nas suas palavras:

```text
Rode a skill idea-sprint. O problema: uma dona de mercadinho vende fiado para os fregueses
e guarda cada conta em uma caderneta de papel ao lado do caixa. Mês passado a caderneta molhou
e uns quarenta fiados em aberto ficaram ilegíveis. Me entreviste como essa dona de mercadinho.
```

*O comando exato muda de uma versão do kit para outra*, então confira o **README atual do kit** antes de digitar. O kit está em github.com/solanabr/solana-ai-kit.

2. **Responda a entrevista** como a dona do mercadinho, sem rodeios, *e não puxe a conversa para a ideia do fiado*. A skill pergunta o que acontece hoje, o que quebra, **quem paga quando quebra** e o que a dona faz a respeito, e não aceita "seria legal se" como resposta. A dona do Fiado conta que os fregueses compram fiado, que uns pagam no dia cinco quando cai o salário e outros não, que a caderneta molhou, que ela pagou por isso com dinheiro que nunca recebeu e com fregueses que pararam de vir, e que agora fotografa cada página com o celular até a memória encher.

3. **Veja o portão de necessidade de cripto** fazer a mesma pergunta a cada candidato: se isso fosse feito com um banco de dados e um app web, o que quebraria? Se nada quebra, a ideia não precisa de chain, e *enfiar uma por baixo é um custo que o jurado vai enxergar*. Se algo específico quebra, **anote essa frase**, porque ela é a sua linha de por-que-Solana para o mês inteiro e vai direto para a resposta de Insight (a sacada).

   A resposta do Fiado não é "pagamentos", porque mercadinho já aceita cartão hoje. O que quebra é **a confiança entre duas pessoas** que não confiam em uma terceira: a dona é quem guarda o registro, e quando a caderneta some, os dois ficam no chute. Um fiado que liquida em stablecoin numa chain que nenhum dos dois controla é *um registro que os dois leem e nenhum consegue editar depois*.

4. **Espere exatamente três candidatos**, e pegue um da porta ao lado. Os vizinhos do seu problema, as lojas, os apps e os hábitos logo ao lado dele, estão cheios de coisas que funcionam no papel, e *o que funciona no papel muitas vezes funciona no papel por um bom motivo*.

   Os três do Fiado: o fiado que liquida em stablecoin com lembrete na data que o freguês escolheu, o registro de pagamento a fornecedores, para o dinheiro que a mesma dona entrega ao distribuidor no dia da entrega sem anotar em lugar nenhum, e os selos de fidelidade de papel que uma padaria duas ruas abaixo distribui, virando um token que o freguês guarda na carteira. **O registro de fornecedores cai no portão** porque só a dona lê, *então não tem de quem desconfiar*. **O cartão de selos cai** porque já é um registro que os dois lados enxergam.

![O idea-sprint roda uma entrevista sem rodeios, um portão de necessidade de cripto, exatamente três candidatos e uma pontuação de 0 a 15 que cai em go, condicional ou no-go com um pivô.](assets/v01-flowchart.webp)

5. **Leia a pontuação**. Cada candidato recebe uma nota de 0 a 15, e a linha que o kit traça, em 2026-09-06, está **no 8**: 8 ou mais é go, 6 a 7 é condicional, abaixo de 6 é no-go com um pivô junto. Um condicional precisa dizer qual é a única coisa que o transformaria em go, e se a skill não disser, pergunte, porque *condicional sem condição é um não educado*.

   Na rodada do Fiado, o fiado fica em 11, go, o registro de fornecedores em 6, condicional com o pivô "quem mais precisa ler esse registro?", e o token de fidelidade em 3, no-go. A sua rodada não vai dar esses números, e o que precisa se repetir é **a ordem e as decisões**. Se o token de fidelidade ficou acima do fiado, **releia a sua entrevista**, porque o motivo quase sempre é que *você respondeu como um builder que gosta de tokens, e não como uma dona de mercadinho que perdeu a caderneta*. O que compõe os 15 pontos muda de versão para versão, então **leia o detalhamento** na saída da própria skill no dia em que rodar.

```text
pontuação de 0 a 15, idea-sprint, github.com/solanabr/solana-ai-kit, lido em 2026-09-06
8 ou mais    go
6 a 7        condicional, com a condição nomeada
abaixo de 6  no-go, com um pivô junto
```

![No exemplo resolvido o fiado tira 11 de 15 para um go, o registro de fornecedores 6 para um condicional e o token de fidelidade 3 para um no-go.](assets/v02-table.webp)

6. **Abra o arquivo que a skill escreveu**:

```bash
cat .claude/context/idea.md
```

Em 2026-09-06, o idea-sprint do kit escrevia ali **exatamente três candidatos**, cada um com nota de 0 a 15 e um go, condicional ou no-go. Na mesma data, a skill era uma adaptação de duas skills da coleção solana-new da sendaifun, find-next-crypto-idea e validate-idea, e o repositório diz isso. **Guarde esse crédito nas suas anotações**, *porque o upstream é para onde você vai quando o kit muda*.

## Pesquise o registro, depois rode o seu

7. **Encontre os projetos anteriores** mais parecidos com cada candidato e anote o que eles não fizeram. Isso é análise de lacunas, e *a lacuna é onde o seu candidato mora*. O que eles fizeram mal é outra anotação, para outra lição, e um candidato sem lacuna é **um clone do vencedor da temporada passada**, que os jurados já pontuaram uma vez.

   Na temporada World's Fair de 2026, o blog da Colosseum diz que a temporada Cypherpunk, com submissões até 30 de outubro de 2025 e vencedores anunciados em 13 de dezembro de 2025, teve 9,000+ participantes e **1,576 projetos finais**, nas tracks Infrastructure, Consumer, DeFi, Stablecoins, RWAs e Undefined. **Undefined é uma track**, então *as categorias não são um cardápio de ideias*. A busca que você roda depende de ter ou não um token.

   Com um PAT: o Colosseum Copilot é uma skill que, em 2026-09-06, indexava **5,400+ projetos de hackathon** de dois anos de submissões, mais 6,300+ produtos via The Grid, e roda no Claude Code, no Codex e no OpenClaw. O kit traz ele como o submódulo ext/colosseum, que aponta para github.com/ColosseumOrg/colosseum-copilot, versão 1.2.1 no dia da leitura, sob licença proprietária. **Pegue o personal access token** em colosseum.com/arena/copilot depois de fazer login, *porque sem ele a skill não responde*, e confira a versão e a página do token no README atual do kit. **Peça ao Copilot os projetos mais parecidos** com cada um dos seus três candidatos e *leia os que foram concluídos, não só submetidos*.

   Sem PAT: abra os **posts de vencedores da Cypherpunk e da Frontier** em blog.colosseum.com, procure em cada um as palavras do seu candidato e siga os nomes até os perfis de empresa em colosseum.com/companies. **Leia três perfis por candidato**. A página do hackathon só lista os vencedores do grande prêmio, e as páginas de projeto em arena.colosseum.org pedem login na Colosseum antes de mostrar qualquer coisa, lido em 2026-09-07, então *a porta aberta são os posts do blog*. Não custa nada, e *eu apostaria que a maioria dos times que se classificam fez isso do jeito lento pelo menos uma vez*.

![Com um PAT do Copilot a busca percorre 5,400+ projetos indexados, e sem ele a mesma nota de lacuna é escrita a partir dos posts de vencedores da temporada no blog e dos perfis públicos de empresa.](assets/v03-flowchart.webp)

8. **Escreva uma nota de lacuna por candidato**, o no-go incluído, sempre no mesmo formato: o projeto anterior mais parecido, a temporada dele, o que ele construiu, o que não fez, e a data em que você pesquisou. A nota do Fiado cita a Yumi Finance, encontrada no post de vencedores da Cypherpunk em 2026-09-07, e *o que você copia é o formato*:

```text
nota de lacuna, candidato 1, o fiado, pesquisado em 2026-09-07
mais parecido: Yumi Finance, Cypherpunk, primeiro prêmio da DeFi Track (post de vencedores em blog.colosseum.com)
construiu:     um produto onchain de compre agora, pague depois que faz a análise de crédito via originação de empréstimo
não fez:       segurar um fiado entre um mercadinho e um freguês que já se conhecem; sem análise de crédito, sem empréstimo, sem novo credor
segundo mais parecido: Corbits, Cypherpunk, segundo lugar da Infrastructure Track / não fez: ferramentas para lojistas em endpoints x402, sem crédito entre as duas partes
```

O projeto mais parecido com o no-go costuma ser justamente o que explica por que ele foi um no-go, então **a nota ganha o seu recibo**. Se o fiado for o único candidato com nota de lacuna, *você não fez análise de lacunas, você foi buscar permissão*. Um candidato em 11 cujo projeto mais parecido já faz tudo o que ele faz **não está mais em 11**, e o memorando diz isso, na linha do candidato, com a data.

9. Agora os seus três. **Anote três pessoas** com quem você conversou nas últimas duas semanas e que reclamaram de alguma coisa: pai ou mãe, o dono do imóvel que você aluga, a pessoa do balcão, o primo de um colega de time. *Quanto mais perto você está da pessoa, melhor a entrevista rende*, e se nenhum dos três problemas for um que você pagaria do próprio bolso para resolver, encontre uma quarta pessoa antes de rodar qualquer coisa.

   **Rode o sprint**, uma vez por problema ou uma vez com os três, conforme a skill atual permitir, e responda sem rodeios. Se perceber que está digitando a resposta que leva à ideia com que você chegou, *essa é exatamente a ideia que o sprint existe para testar*, então digite a resposta verdadeira e deixe a nota sair.

   Depois **rode a busca de lacunas** para os três e escreva as três notas no memorando de ideias, com uma linha marcada como escolhida. Se o candidato escolhido ficou abaixo de 8, aceite o pivô que a skill sugeriu, *porque rodar a entrevista de novo até sair o número que você queria dá no mesmo que não pontuar*. A primeira linha do candidato escolhido é **uma pessoa e um problema**, nessa ordem, a mesma regra do pitch v0, porque as três próximas lições voltam a essa linha o tempo todo.

![O memorando de ideias pronto tem três candidatos pontuados, um marcado como escolhido e uma nota de lacuna com data para cada um, incluindo o no-go.](assets/v04-table.webp)

## Está pronto quando

- O **memorando de ideias** tem três candidatos, cada um com nota de 0 a 15 e o portão respondido em uma linha.
- Um candidato está marcado como escolhido, a nota dele é **8 ou mais**, e a primeira linha dele é uma pessoa e um problema.
- Cada um dos três, o no-go incluído, tem uma **nota de lacuna** citando pelo menos um projeto anterior, o que ele não fez e a data da pesquisa.
- A **linha de fonte** do memorando cita .claude/context/idea.md e a consulta ao Copilot ou as páginas públicas que você pesquisou.

## Fique atento

- Escolher a ideia que mostra **mais tecnologia**, porque os sete fatores não pontuam tecnologia por si só.
- Rodar a busca no registro **só depois de escolher a ideia**, porque a graça de ter três candidatos é que o registro pode mudar a ordem deles, e muda.
- Tratar uma memecoin, ou **um clone do vencedor da temporada passada**, como lacuna.

Um memorando de ideias com nota **mata ideias que você ama**, e o trabalho dele é esse, porque o time que pula a nota para proteger a favorita não escapa da avaliação, só entrega ela para os jurados na semana 3, *que avaliam em silêncio e nunca mandam o pivô*. Se a sua favorita ficou abaixo de 6, **discuta com a entrevista**, não com o número, e se as respostas foram honestas a ideia fica no memorando como um no-go com data, que pode voltar com um 9 na próxima temporada.

## O que fica

Uma ideia ganha o lugar dela sobrevivendo a uma entrevista sem rodeios, ao portão de necessidade de cripto e a uma nota ao lado de duas rivais, **não por ser a ideia com que você chegou**. A pergunta do portão, o que quebraria com um banco de dados e um app web, é a sua linha de por-que-Solana para o mês. *Um candidato escolhido é uma pessoa e um problema com uma nota de lacuna com data ao lado, e a nota que você torcia para ninguém conferir é a que os jurados vão conferir.*

## A seguir

O responsável pela concorrência assume o papel na próxima lição: quem resolve isso hoje, fora do registro dos hackathons, o que eles fazem bem, o que fazem mal, e *o único número que um jurado vai pedir e que você ainda não tem*. A primeira coisa que você faz lá é **copiar as três perguntas** que a Colosseum faz em Potential Market Size (tamanho potencial do mercado) para um arquivo novo, antes de citar um concorrente sequer, então deixe o brief e o memorando à mão.
