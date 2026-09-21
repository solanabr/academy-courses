# Problemas que você realmente tem

Na lição passada você escreveu o cartão do time, o plano datado do mês e um pitch v0 de uma frase, e três pessoas já reagiram a essa frase. **Deixe o log de feedback aberto**, *porque a frase está prestes a ser testada mais duro do que três amigos conseguiriam testar*.

## Por que isso importa

A semana 1 começa aqui, e a primeira coisa que você faz com a sua ideia é **tentar matá-la**. As boas ideias de hackathon que eu vi ganhar vieram de um problema que alguém do time, ou alguém próximo, tem ou teve, e as que começaram de "DeFi para X" na maioria das vezes não ficaram bem colocadas. Essa é a divisão entre **problema-primeiro**, uma pessoa e uma coisa que ela não consegue fazer hoje, e capacidade-primeiro, um recurso da chain procurando alguém em quem ser aplicado. Os sete fatores não pontuam tecnologia por si só, e *um jurado lendo as respostas escritas percebe quando a pessoa foi acrescentada na semana 4*.

Então hoje você gera três ideias problema-primeiro e pontua todas até uma sobreviver. **Abra o Claude Code** dentro do repositório do toolkit que você inicializou no dia 0, com o kit instalado do jeito que a lição 3 fez, e comece pelo passo 1.

## Rode o sprint no Fiado

Um lembrete antes do primeiro passo: **o Fiado é o projeto de exemplo do curso**, nada mais. É a caderneta de fiado do mercadinho da lição 1, transformada em um fiado liquidado em stablecoin, e todo laboratório deste curso roda nele primeiro *para você sempre ter um exemplo resolvido para copiar*. Depois você roda os mesmos passos no seu próprio problema.

1. **Peça a skill idea-sprint** pelo nome, com o problema do Fiado como entrada, nas suas palavras:

```text
Rode a skill idea-sprint. O problema: uma dona de mercadinho vende fiado para os fregueses
e guarda cada conta em uma caderneta de papel ao lado do caixa. Mês passado a caderneta molhou
e uns quarenta fiados em aberto ficaram ilegíveis. Me entreviste como essa dona de mercadinho.
```

*A invocação exata muda com as versões do kit*, então confira no **README atual do kit** antes de digitar. O kit está em github.com/solanabr/solana-ai-kit.

2. **Responda à entrevista** como a dona do mercadinho, sem rodeios, *e não conduza a conversa para a ideia do fiado*. A skill pergunta o que acontece hoje, o que quebra, **quem paga quando quebra** e o que a dona faz a respeito, e não aceita "seria legal se" como resposta. A dona do Fiado diz que os fregueses compram fiado, alguns pagam no dia cinco quando cai o salário e alguns não, a caderneta molhou, ela pagou por isso em dinheiro que nunca recebeu e em fregueses que pararam de vir, e agora fotografa cada página com o celular até a memória encher.

3. **Observe o portão de necessidade de cripto** fazer uma pergunta a cada candidato: construído com um banco de dados e um app web, o que quebraria? Se nada quebra, a ideia não precisa de chain, e *colocar uma por baixo é um custo que o jurado vai enxergar*. Se algo específico quebra, **anote essa frase**, porque ela é a sua linha de por-que-Solana para o mês e vai direto para a resposta de Insight (a sacada).

   A resposta do Fiado não é "pagamentos", já que um mercadinho pode aceitar cartão hoje. O que quebra é **a confiança entre duas pessoas** que não confiam em uma terceira: a dona segura o registro, e quando a caderneta some os dois ficam no chute. Um fiado que liquida em stablecoin em uma chain que nenhum dos dois controla é *um registro que os dois podem ler e nenhum pode editar depois*.

4. **Espere exatamente três candidatos**, e pegue um da porta ao lado. Ecossistemas adjacentes, as lojas, apps e hábitos logo ao lado do seu problema, estão cheios de coisas que funcionam no papel, e *uma coisa que funciona no papel muitas vezes está funcionando no papel por um motivo*.

   Os três do Fiado: o fiado que liquida em stablecoin com um lembrete na data que o freguês escolheu, o registro de pagamento a fornecedores para o dinheiro que a mesma dona entrega aos distribuidores no dia da entrega sem nenhum registro, e os selos de fidelidade em papel que uma padaria duas ruas adiante distribui, como um token que o freguês guarda em uma carteira. **O registro falha no portão** porque só a dona lê, *então não há de quem desconfiar*. **O cartão de selos falha** porque já é um registro que as duas partes enxergam.

![O idea-sprint roda uma entrevista sem rodeios, um portão de necessidade de cripto, exatamente três candidatos e uma pontuação de 0 a 15 que cai em go, condicional ou no-go com um pivô.](assets/v01-flowchart.webp)

5. **Leia a pontuação**. Cada candidato recebe um número de 0 a 15, e a linha que o kit traça, em 2026-09-06, está **no 8**: 8 ou mais é go, 6 a 7 é condicional, abaixo de 6 é no-go com um pivô anexado. Um condicional precisa nomear a única coisa que o tornaria um go, e se a skill não nomear, pergunte, já que *um condicional sem condição é um não educado*.

   Na rodada do Fiado o fiado fica em 11, um go, o registro de fornecedores em 6, condicional com o pivô "quem mais precisa ler esse registro?", e o token de fidelidade em 3, um no-go. A sua rodada não vai cair nesses números, e o que deve se manter é **a ordem e as decisões**. Se o token de fidelidade saiu acima do fiado, **releia a sua entrevista**, porque o motivo de sempre é que *você respondeu como um builder que gosta de tokens em vez de como uma dona de mercadinho que perdeu a caderneta*. Do que os 15 pontos são feitos muda entre versões, então **leia o detalhamento** na saída da própria skill no dia em que rodar.

```text
pontuação de 0 a 15, idea-sprint, github.com/solanabr/solana-ai-kit, lido em 2026-09-06
8 ou mais    go
6 a 7        condicional, com a condição nomeada
abaixo de 6  no-go, com um pivô anexado
```

![No exemplo resolvido o fiado marca 11 de 15 para um go, o registro de fornecedores 6 para um condicional e o token de fidelidade 3 para um no-go.](assets/v02-table.webp)

6. **Abra o arquivo que a skill escreveu**:

```bash
cat .claude/context/idea.md
```

Lido em 2026-09-06, o idea-sprint do kit escreve **exatamente três candidatos** ali, cada um com uma pontuação de 0 a 15 e um go, condicional ou no-go. Em 2026-09-06 a skill é adaptada de duas skills da coleção solana-new da sendaifun, find-next-crypto-idea e validate-idea, e o repositório diz isso. **Guarde esse crédito nas suas anotações**, *já que o upstream é aonde você vai quando o kit muda*.

## Pesquise o registro, depois rode o seu

7. **Encontre os projetos passados** mais próximos de cada candidato e anote o que eles não fizeram. Isso é análise de lacunas, e *a lacuna é onde o seu candidato vive*. O que eles fizeram mal é outra anotação para outra lição, e um candidato sem lacuna é **um clone do vencedor da temporada passada**, que os jurados já pontuaram uma vez.

   Na temporada World's Fair de 2026, o blog da Colosseum informa que a temporada Cypherpunk, com submissões até 30 de outubro de 2025 e vencedores anunciados em 13 de dezembro de 2025, teve 9,000+ participantes e **1,576 projetos finais**, nas tracks Infrastructure, Consumer, DeFi, Stablecoins, RWAs e Undefined. **Undefined é uma track**, então *as categorias não são uma lista de ideias para escolher*. Qual busca você roda depende de um token.

   Com um PAT: o Colosseum Copilot é uma skill que, em 2026-09-06, indexa **5,400+ projetos de hackathon** de dois anos de submissões mais 6,300+ produtos via The Grid, e roda no Claude Code, no Codex e no OpenClaw. O kit o entrega como o submódulo ext/colosseum, que aponta para github.com/ColosseumOrg/colosseum-copilot, versão 1.2.1 no dia em que isso foi lido, sob licença proprietária. **Pegue o personal access token** em colosseum.com/arena/copilot depois de fazer login, *porque sem ele a skill não responde*, e confira a versão e a página do token no README atual do kit. **Peça ao Copilot os projetos mais próximos** de cada um dos seus três candidatos e *leia os que foram finalizados, não só submetidos*.

   Sem um PAT: abra os **posts de vencedores da Cypherpunk e da Frontier** em blog.colosseum.com, procure em cada um as palavras do seu candidato e siga os nomes até os perfis de empresa em colosseum.com/companies. **Leia três perfis para cada candidato**. A própria página do hackathon lista só os vencedores do grande prêmio, e as páginas de projeto em arena.colosseum.org pedem login na Colosseum antes de mostrar qualquer coisa, lido em 2026-09-07, então *os posts do blog são a porta aberta*. Não custa nada, e *eu apostaria que a maioria dos times que ficam bem colocados fez isso do jeito lento pelo menos uma vez*.

![Com um PAT do Copilot a busca percorre 5,400+ projetos indexados, e sem ele a mesma nota de lacuna é escrita a partir dos posts de vencedores da temporada no blog e dos perfis públicos de empresa.](assets/v03-flowchart.webp)

8. **Escreva uma nota de lacuna por candidato**, incluindo o no-go, em um formato fixo: o projeto passado mais próximo, a temporada dele, o que ele construiu, o que não fez, e a data em que você pesquisou. A nota do Fiado nomeia a Yumi Finance, lida no post de vencedores da Cypherpunk em 2026-09-07, e *o formato é o que você copia*:

```text
nota de lacuna, candidato 1, o fiado, pesquisado em 2026-09-07
mais próximo:  Yumi Finance, Cypherpunk, primeiro prêmio da DeFi Track (post de vencedores em blog.colosseum.com)
construiu:     um produto onchain de compre agora, pague depois que faz a análise de crédito via originação de empréstimo
não fez:       segurar um fiado entre um mercadinho e um freguês que já se conhecem; sem análise de crédito, sem empréstimo, sem novo credor
segundo mais próximo: Corbits, Cypherpunk, segundo lugar da Infrastructure Track / não fez: ferramentas para lojistas em endpoints x402, sem crédito entre as duas partes
```

O projeto mais próximo do no-go costuma ser o que diz por que ele foi um no-go, então **a pontuação ganha o seu recibo**. Se o fiado é o único candidato com nota, *você não fez análise de lacunas, você procurou permissão*. Um candidato em 11 com um projeto mais próximo que já faz tudo que ele faz **não está mais em 11**, e o memorando diz isso, na linha do candidato, com a data.

9. Agora os seus três. **Anote três pessoas** com quem você conversou nas últimas duas semanas que reclamaram de alguma coisa: um pai ou mãe, um senhorio, a pessoa do balcão, o primo de um colega de time. *Quanto mais perto você está da pessoa, melhor a entrevista vai*, e se nenhum dos três for um problema que você pessoalmente pagaria para ver resolvido, encontre uma quarta pessoa antes de rodar qualquer coisa.

   **Rode o sprint**, uma vez por problema ou uma vez com os três, o que a skill atual suportar, e responda sem rodeios. Se você se pegar digitando a resposta que leva à ideia com que chegou, *essa é a ideia que o sprint existe para testar*, então digite a resposta verdadeira e deixe pontuar.

   Depois **rode a busca de lacunas** para os três e escreva as três notas no memorando de ideias, com uma linha marcada como escolhida. Se o candidato escolhido ficou abaixo de 8, aceite o pivô que a skill anexou, *já que rodar a entrevista de novo até aparecer o número que você queria é o mesmo que não pontuar*. A primeira linha da entrada escolhida é **uma pessoa e um problema**, nessa ordem, a mesma regra do pitch v0, porque as três próximas lições leem essa linha repetidas vezes.

![O memorando de ideias pronto tem três candidatos pontuados, um marcado como escolhido e uma nota de lacuna datada para cada um, incluindo o no-go.](assets/v04-table.webp)

## Está pronto quando

- O **memorando de ideias** tem três candidatos, cada um com pontuação de 0 a 15 e o portão respondido em uma linha.
- Um candidato está marcado como escolhido, a pontuação dele é **8 ou mais**, e a primeira linha dele é uma pessoa e um problema.
- Cada um dos três, o no-go incluído, tem uma **nota de lacuna** nomeando pelo menos um projeto passado, o que ele não fez e a data da pesquisa.
- A **linha de fonte** do memorando nomeia .claude/context/idea.md e a consulta ao Copilot ou as páginas públicas que você pesquisou.

## Fique atento

- Escolher a ideia que exibe **mais tecnologia**, porque os sete fatores não pontuam tecnologia por si só.
- Rodar a busca no registro **só depois de a ideia ser escolhida**, porque o sentido de ter três candidatos é que o registro pode reordená-los, e reordena.
- Tratar uma memecoin, ou **um clone do vencedor da temporada passada**, como lacuna.

Um memorando de ideias pontuado **mata ideias que você ama**, e esse é o trabalho dele, porque o time que pula a pontuação para proteger uma favorita não escapa da avaliação, entrega ela aos jurados na semana 3, *que avaliam em silêncio e nunca mandam o pivô*. Se a sua favorita ficou abaixo de 6, **discuta com a entrevista** em vez de com o número, e se as respostas foram honestas a ideia fica no memorando como um no-go datado que pode voltar com um 9 na próxima temporada.

## O que fica

Uma ideia conquista seu lugar sobrevivendo a uma entrevista sem rodeios, ao portão de necessidade de cripto e a uma pontuação ao lado de duas rivais, **não por ser a ideia com que você chegou**. A pergunta do portão, o que quebraria com um banco de dados e um app web, é a sua linha de por-que-Solana para o mês. *Um candidato escolhido é uma pessoa e um problema com uma nota de lacuna datada ao lado, e uma pontuação que você torcia para ninguém conferir é uma pontuação que os jurados vão conferir.*

## A seguir

O dono da concorrência ganha o papel na próxima lição: quem faz isso hoje, fora do registro dos hackathons, o que eles fazem certo, o que erram, e *o único número que um jurado vai pedir e que você ainda não tem*. A sua primeira ação lá é **copiar as três perguntas** que a Colosseum faz sob Potential Market Size (tamanho potencial do mercado) para um arquivo novo, antes de nomear um único concorrente, então deixe o brief e o memorando ao alcance da mão.
