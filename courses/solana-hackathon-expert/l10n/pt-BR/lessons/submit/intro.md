# O formulário, o prazo, as regras

Na lição passada você gravou os dois vídeos dentro dos limites, a apresentação entre dois e três minutos e a demo em três ou menos com a transação confirmando na tela, e registrou a rodada de feedback 4. **Deixe os dois links** onde você alcança. Este é o capstone: termina com um pacote submetido e uma captura de tela com data, *e nada é gravado hoje*.

## Por que isso importa

Toda temporada, times com um produto funcionando **perdem num formulário**. Um prazo convertido na direção errada. Uma segunda submissão de um integrante que quebra a regra de uma por pessoa. Um vídeo de demo que nunca chegou ao seu campo. Um repositório privado a que ninguém deu acesso ao revisor. Nada disso está nos critérios de julgamento. Tudo isso está nas regras, na mesma página dos sete fatores, *alguns parágrafos abaixo, onde menos gente lê*.

A pessoa que abre sua submissão primeiro não está pontuando Founder + Market Fit (encaixe entre fundador e mercado), está conferindo se o vídeo está lá, se o repositório abre, se o time tem nomes. **Completude é a única linha** nessa página inteira que um time controla por completo. Então hoje você **lê as regras** do jeito que leu os critérios de julgamento na lição 2, como a lista exata do que é conferido, e submete do jeito que faria um release, *contra um checklist escrito antes de o trabalho começar*.

## Faça isto

1. **Monte o pacote** antes de abrir qualquer portal. Crie um arquivo chamado submission-package.md na raiz do repositório do toolkit e coloque nele **uma linha por peça que você construiu**, com o caminho do artefato e a data, *de memória primeiro e depois corrigido contra o repositório*:

```text
submission-package.md, aberto em 2026-10-05
brief da Colosseum (os sete fatores, linha do prazo, desclassificadores)              caminho, data
cartão do time, plano do mês, pitch v0                                                caminho, data
memorando de ideia, mapa de concorrentes, pacote de evidências, memorando de decisão  caminho, data
design partners, log de tração                                                        caminho, data
cartão de escopo, roteiro da demo, fatia na devnet, log narrativo                     caminho, data
one-pager de GTM, lista de parceiros                                                  caminho, data
deck, as duas exportações                                                             caminho, data
vídeo de apresentação (2 a 3 min), vídeo de demo (3 min ou menos)                     link, duração cronometrada
respostas do portal                                                                   este arquivo, abaixo
```

Se uma linha não tem caminho, **essa peça está faltando**, e *hoje é o dia de terminar essa peça e não o dia da submissão*. Tudo abaixo assume que cada linha está preenchida.

2. **Responda ao portal da Colosseum**, todos os nove campos, a partir do arquivo e na ordem da página. O portal da Colosseum, na temporada World's Fair de 2026, pede nove coisas, e a página que as lista é colosseum.com/hackathon, a mesma página que o seu brief da Colosseum cita. **Copie-as para o arquivo do pacote** nas palavras da página antes de responder a qualquer uma:

```text
campos do portal, colosseum.com/hackathon, lido em 2026-09-06
1  product name and brief description (nome do produto e descrição breve)
2  blockchains and tools used (blockchains e ferramentas usadas)
3  all teammates with backgrounds and previous experience (todos os integrantes com formação e experiência anterior)
4  team location (localização do time)
5  a product logo or graphic (um logo ou imagem do produto)
6  GitHub repository link (link do repositório no GitHub; privado permitido se o acesso for concedido a hackathon@colosseum.com)
7  a two-to-three-minute presentation video (um vídeo de apresentação de dois a três minutos)
8  a product-demo video of no more than three minutes (um vídeo de demo do produto de no máximo três minutos)
9  go-to-market strategy, demand validation and distribution plans (estratégia de go-to-market, validação de demanda e planos de distribuição)
```

Se a página ao vivo mostrar uma lista diferente no dia em que você copiar, **a página vence**, e anote o que mudou. Cada campo é alimentado por uma peça que você já construiu. O nome do produto é a única palavra de produto no pitch final, a palavra que vem depois do problema. Blockchains e ferramentas vêm do log narrativo, nomeadas exatamente como o log as nomeia e nada que o build não usou, *porque a revisão do repositório lê o código e as duas listas devem concordar*. Os integrantes vêm do cartão do time, uma linha cada com a formação e a experiência anterior, e **nenhum lugar vazio**. A localização é a cidade do cartão do time.

O logo é a imagem de título do deck exportada como imagem, e *nada é desenhado hoje*. O link do repositório é o campo com **uma segunda pessoa dentro**: se o repositório é privado, um integrante concede acesso a hackathon@colosseum.com e outro confirma lendo a lista de acesso do repositório. O vídeo de apresentação é o arquivo de **2 a 3 minutos** da lição passada, o vídeo de demo é o de 3 minutos ou menos, e os dois entram como links com a duração cronometrada escrita ao lado.

O último campo é **três perguntas numa caixa só**. A estratégia de go-to-market é o one-pager. A validação de demanda é o pacote de evidências, as conversas da semana 1, e **a linha mais recente do log de tração** com a data. Os planos de distribuição são os parceiros nomeados na lista de parceiros, e *um parceiro que respondeu, mesmo com um não, é uma linha melhor do que uma categoria de parceiro*. Para o Fiado a linha de ferramentas diz Solana, devnet, a ferramenta de agente e as skills do kit como o log narrativo as nomeia, e a descrição e a resposta de GTM são **os dois campos deixados em branco** para você escrever no mesmo formato.

A descrição é **escrita num formato**, e um slide colado não lê bem. A skill de hackathon do kit carrega um: uma tagline, o problema, a coisa nova em negrito, uma parte intitulada 'What works today' (o que funciona hoje), e por que Solana, em **200 a 500 palavras**, pontuada contra o judging-criteria.md do próprio kit. Foi isso que a skill dizia em github.com/solanabr/solana-ai-kit em 2026-09-06. Verifique contra o README atual do kit antes de usar, *porque um arquivo de skill muda mais rápido do que uma lição*.

A mesma skill escolhe uma trilha pelo quão cheia cada uma está no portal ao vivo, pelo ext/colosseum do kit com um COLOSSEUM_COPILOT_PAT definido no ambiente, lido em 2026-09-06, e essa parte depende do README atual mais do que o formato, então **leia-o de novo no dia**.

```text
descrição, escrita no formato da skill de hackathon do kit (verifique contra o README atual)
tagline:             o one-liner do deck, cinco palavras ou menos, sem jargão
problema:            quem tem e o que custa para essa pessoa, do pacote de evidências
**a coisa nova**     uma frase em negrito, o insight do memorando de decisão
What works today:    a fatia, exatamente o que o vídeo da demo mostra, nada planejado
por que Solana:      o parágrafo do one-pager de GTM
tamanho: 200 a 500 palavras. pontuada contra o judging-criteria.md no kit.
```

Essa ordem é a ordem em que um revisor lê. 'What works today' é **a linha da honestidade**, a que a revisão do repositório confere contra o código, então ela lista exatamente o que o vídeo da demo mostra e nada que está planejado. Coloque a tecnologia ali, em palavras simples, *depois que o leitor tem um motivo para querê-la*.

![Cada um dos nove campos do portal da Colosseum é respondido por uma peça nomeada do curso, do pitch final para o nome até o one-pager de GTM e a lista de parceiros para a última caixa.](assets/v01-table.webp)

3. **Converta o prazo com uma ferramenta**, *nunca de cabeça*, e escreva os dois resultados no arquivo do pacote, um ao lado do outro. A string para praticar é inventada, mas o formato é o que toda página imprime: uma data, um T, uma hora, e um Z que significa UTC.

```bash
python3 -c "from datetime import datetime, timezone; d = datetime.fromisoformat('2026-09-08T02:59:59+00:00'); print(d.astimezone(timezone.utc)); print(d.astimezone())"
```

Num laptop configurado no horário de Brasília as duas linhas saem como 2026-09-08 02:59:59+00:00 e **2026-09-07 23:59:59-03:00**. A página diz dia oito. Seu calendário diz **dia sete**, um segundo antes da meia-noite. Um time que colocou "dia 8" no grupo e planejou submeter na manhã do dia 8 *já perdeu, com um produto funcionando e um vídeo bom*. O +00:00 no comando é o Z final escrito por extenso, e no **Python 3.11 ou mais novo** o Z funciona sozinho, então numa máquina mais antiga mantenha a forma longa.

Para a Colosseum a string vem do seu brief da Colosseum. A lição 2 copiou a data de fim da temporada da página ao vivo, e em 2026-09-06 a hora era a parte que a página não tinha impresso, *então o brief guarda um espaço para ela*. Na semana final, releia colosseum.com/hackathon, **cole a hora que a página imprime** no comando, e escreva o resultado local na linha do prazo ao lado do original em UTC.

Depois converta uma segunda vez com uma ferramenta diferente, o relógio mundial de um celular basta, e só quando as duas concordam a linha ganha a palavra "conferido" depois dela. **Escreva o nome do líder** nessa mesma linha, já que a página diz que o líder do time deve completar a submissão antes do prazo. Minha regra pessoal, *uma preferência e não uma regra da página*, é marcar o prazo do time **um dia inteiro antes** do impresso, para que o último dia seja para a captura de tela e não para o formulário.

![O prazo é copiado em UTC, convertido duas vezes, adiantado um dia para o time, conferido de novo na véspera, e então o líder submete e a captura de tela vai para o log.](assets/v02-timeline.webp)

4. **Leia os quatro desclassificadores** e assine-os. Da página, lida em 2026-09-06, nas palavras dela para o primeiro:

> 'Only one product submission is allowed per team, and therefore one per individual, during each hackathon' (só uma submissão de produto é permitida por time, e portanto uma por pessoa, em cada hackathon) (colosseum.com/hackathon, 2026-09-06).

**Um produto por time**, e portanto um por pessoa. O jeito como os times quebram isso sem querer é **a segunda conta**: um integrante se registra por conta própria para "garantir", ou submete um experimento paralelo no próprio nome, e agora uma pessoa tem duas submissões na temporada. Ninguém no time submete mais nada, e *o líder é a única pessoa que aperta o botão*.

Segundo, **o líder do time completa a submissão** antes do prazo, então o líder é nomeado no arquivo, na linha do prazo onde o nome já está. Terceiro, a página diz que **deturpar o histórico de desenvolvimento** pode desclassificar, então código anterior é declarado na submissão, em 'What works today' ou onde o formulário der espaço, copiado do log narrativo onde a versão honesta já mora, com data. Quarto, acesso. Um repositório privado sem acesso concedido a hackathon@colosseum.com é um repositório que o revisor não consegue abrir, e a minha leitura, *que a página não explicita*, é que um repositório que ninguém consegue abrir é **pontuado como nenhum repositório**.

```text
checklist de desclassificadores, colosseum.com/hackathon, lido em 2026-09-06
[ ] um produto deste time, e ninguém nele submete outro                     assinado: líder
[ ] código anterior e histórico de desenvolvimento declarados na submissão  assinado: líder
[ ] o líder completa a submissão antes do prazo                             assinado: líder
[ ] acesso ao repositório concedido a hackathon@colosseum.com, confirmado por um segundo integrante
```

O líder assina três e um segundo integrante assina o quarto, e o arquivo não é um checklist **até os nomes estarem nele**. Um checklist que ninguém assinou é *uma lista de coisas que alguém torceu para serem verdade*.

5. **Submeta**, numa ordem fixa. **Abra a página ao vivo** uma última vez e compare a lista de campos dela com a do seu arquivo, *porque um portal que adiciona um campo entre o dia em que você copiou e o dia da submissão não é algo que esta lição consiga descartar*, e um campo que você nunca viu é um campo deixado vazio.

Depois o líder entra no portal, cola cada resposta do arquivo, **relê cada campo** contra o arquivo antes de passar ao próximo, submete, e tira uma captura de tela da confirmação para o log narrativo com a data. Fora de temporada, quando nenhum portal está aberto, o formulário simulado é **este mesmo arquivo**: os campos na ordem da página, nada em branco, e o nome do líder e a data na última linha *no lugar da captura de tela*.

![No dia da submissão o líder compara a lista de campos ao vivo, cola e relê cada campo do arquivo, submete, registra a captura de tela, e só então abre um formulário de ensaio.](assets/v03-flowchart.webp)

6. **Passe o mesmo pacote** por um ensaio se houver um aberto, *depois ou junto com o formulário da Colosseum e nunca no lugar dele*. Dois tipos podem recebê-lo: um hackathon sazonal, na própria plataforma numa janela que fica antes de uma temporada da Colosseum, e uma side track (a trilha regional), um prêmio regional preso à temporada da Colosseum. Qualquer um dos dois vai ter sua própria página com suas próprias entregas e seu próprio prazo, então **leia essa página no dia em que decidir entrar** e reaproveite o pacote que você já tem. Edições e datas são lidas da página ao vivo toda vez.

Então o método é uma frase: abra a página dele, **copie as entregas e o prazo** para o arquivo do pacote com a data, responda às perguntas dele e cole o resto. **Não reconstrua o pacote** para ele e não escreva uma segunda descrição. Antes de qualquer segunda submissão, leia a regra na página do próprio hackathon e a regra de um-por-pessoa na página da Colosseum, no dia, e escreva no arquivo do pacote qual regra você leu e onde. Se as duas regras não podem ser cumpridas ao mesmo tempo, **pule o ensaio** e *escreva por quê*.

![Antes de uma segunda submissão o time lê a regra daquele hackathon e a regra de um-por-pessoa da Colosseum, submete só se as duas se sustentam, e caso contrário pula o ensaio.](assets/v04-flowchart.webp)

## Está pronto quando

- **Nenhum campo do portal está vazio**, e cada resposta no submission-package.md aponta para uma peça que você construiu.
- A **linha do prazo** mostra UTC e local, em acordo por duas ferramentas, marcada como "conferido", com o nome do líder.
- O **checklist de desclassificadores** está assinado, três caixas pelo líder e a quarta por um segundo integrante.
- Uma **confirmação**, captura de tela real ou linha do formulário simulado, está no log narrativo com a data.
- Se um formulário de ensaio estava aberto, suas **entregas, prazo e regra** estão no arquivo do pacote também.

## Fique atento

- **Converter o prazo uma vez** e confiar. Uma conversão feita um mês antes por um integrante que desde então mudou de fuso é a linha com mais chance de estar errada, *então confira de novo na véspera*.
- Uma descrição que **começa pela tecnologia**. "Um programa Solana usando PDAs para armazenar..." não diz nada para um revisor que ainda não conhece o problema.
- **Entrar num ensaio por reflexo**, o que acontece quando as regras dele moram num chat e não no arquivo, *então copie a página, não o resumo*.
- O tradeoff: um ensaio **custa um segundo formulário** e pode colidir com a regra de um-produto-por-pessoa se ele mesmo for uma trilha da Colosseum, *então pule um ensaio em vez de quebrar uma regra*.

## O que fica

As regras ficam na mesma página dos critérios de julgamento, alguns parágrafos abaixo, e são **a parte que um time controla por completo**: cada campo preenchido a partir de um arquivo que já existe, um prazo convertido por duas ferramentas e escrito nos dois fusos, um produto por pessoa, e o líder apertando o botão com um dia de folga. *O vídeo de apresentação é o campo que um jurado abre primeiro, e a regra de um-por-pessoa é a que desclassifica um time que ninguém avisou.*

## A seguir

O capstone está pronto quando a confirmação está no log, o último artefato que a promessa do curso pediu. A primeira ação da próxima lição é **um arquivo chamado next-season-plan.md** ao lado deste pacote, com duas frases copiadas da página ao vivo sobre o que acontece depois que as submissões fecham, a entrevista de 15 minutos para a qual um grupo menor é convidado e o anúncio mais ou menos um mês depois do prazo. A partir daí ela percorre *por que um pitch perdido não é um projeto perdido*.

Você termina o mês com **um pacote que um jurado consegue abrir**, um formulário que ninguém compôs no portal e uma captura de tela com data. Guarde a captura de tela, porque daqui a um mês, diga o que disser a página de vencedores, *ela é a prova de que o formulário não te venceu*.
