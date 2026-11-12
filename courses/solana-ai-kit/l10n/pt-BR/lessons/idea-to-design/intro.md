# Da ideia ao design

Na lição passada o pesquisador, chamado pelo nome, respondeu uma pergunta. Nesta lição dois agentes levam o Stamp de **três linhas de ideia a um design a partir do qual um engenheiro consegue construir**, e você aprende o que cada um deles deve a você quando termina. Uns vinte minutos, e nada aqui precisa de toolchain.

## Por que isso importa

A maioria dos projetos de hackathon que vi falhar perdeu o mês na primeira semana, com o modelo de contas errado e uma ideia copiada. O kit coloca dois especialistas nesse ponto. **solana-researcher** responde perguntas sobre o ecossistema com fontes citadas e datadas e um rótulo de confiança em cada achado, porque o ecossistema anda mais rápido que a memória de qualquer modelo. **solana-architect** fecha contas, PDAs, fronteiras de programa, signatários e o modelo de confiança *antes* de existir código, para que os engenheiros nunca precisem reabri-los. Jogos têm um arquiteto próprio, **game-architect**, que escreve um arquivo de conceito e um arquivo de plano e decide o que vive on-chain. A saída desta lição é um arquivo que o resto do curso lê.

## Mão na massa

1. **Escreva a ideia em três linhas.** Crie `idea.md` no projeto com uma pessoa, a coisa que ela não consegue fazer hoje e o que o Stamp faz a respeito. A nossa:

```text
Stamp
Rita tem uma cafeteria e dá um café grátis a cada décima compra, controlado em cartões de papel que os clientes perdem.
O Stamp guarda o cartão na Solana: a cafeteria carimba a carteira do cliente no caixa, dez carimbos resgatam um café, e o cliente vê o cartão em qualquer app de carteira.
```

2. **Pergunte ao pesquisador o que existe.** Digite dentro do Claude Code:

```text
Use o agente solana-researcher. O que existe hoje na Solana para cartões fidelidade ou cartões de carimbo de pequenos negócios, incluindo projetos que fecharam e por quê? Cite uma fonte para cada achado com data e rótulo de confiança, e termine com o que você não conseguiu verificar.
```

O arquivo dele fixa o formato da resposta: **primeiro uma resposta de duas ou três frases**, depois achados, cada um com link e rótulo, **High** quando várias fontes com autoridade concordam, **Medium** para uma só, **Low** para fontes só da comunidade ou fontes que se contradizem, **Speculative** para uma inferência, depois uma recomendação e uma lista do que não conseguiu verificar. Leia os itens Low como pistas, não como fatos. *O agente pode dizer que não encontrou nada sólido, e essa resposta vale ter antes de você construir.*

### Se você quiser ir mais fundo na pesquisa

Os achados do pesquisador param no que é público na web. O kit também traz o **Colosseum Copilot** como um pack de skills principal, que responde a partir do acervo de hackathons e dos históricos de projetos da Colosseum: o que submissões passadas afirmaram, o que entregaram depois e o que continua desconhecido. Ele precisa de uma conta na Colosseum e de um login uma vez por máquina, em um terminal, com Node 20 ou mais novo:

```bash
npx @colosseum-org/copilot-connect login
```

O login abre o navegador. Adicione `--device` em uma máquina onde nenhum navegador consegue abrir, e confira a conexão com `status`, que também é o que o `/doctor` roda na quinta verificação. Depois pergunte dentro do Claude Code:

```text
Use o agente solana-researcher com a skill do Colosseum copilot. Quais submissões de hackathons passados construíram cartões fidelidade ou de carimbo para pequenos negócios? Para cada uma, separe o que ela afirmou na submissão do que aconteceu depois, com datas, e termine com o que você não conseguiu encontrar.
```

Leia a entrada do pack no registro do kit antes de fazer login. Ela marca o pack como *caution*: o helper é um pacote npm sem versão fixada, a skill consulta a API da Colosseum como você, e as suas perguntas e as respostas do agente vão para a Colosseum quando você marca a caixa opcional de compartilhamento no login, que vem desmarcada por padrão. *Para um workshop, os achados públicos bastam. Para uma inscrição em hackathon, este é o acervo que você quer conferido.*

3. **Entregue a ideia ao arquiteto.** Digite:

```text
Use o agente solana-architect para projetar o Stamp a partir de idea.md. Quero cada conta com o owner, as seeds, o tamanho, quem paga o rent e quem pode alterar ou fechar a conta, cada instrução com os signatários, as invariantes, o modelo de autoridade e os riscos em aberto. Use Anchor por padrão, a menos que você consiga mostrar uma razão medida para Pinocchio. Escreva o resultado em design.md.
```

Essa lista é a própria seção **Output** do arquiteto, citada de volta para ele, um truque que vale guardar: *um agente a quem se pede exatamente o que o arquivo dele promete raramente deixa algo de fora.* Espere algo assim para o Stamp:

- Uma **conta da cafeteria**, uma PDA a partir de um prefixo fixo e da carteira da cafeteria, guardando o número de carimbos por recompensa e a autoridade da cafeteria.
- Uma **conta de cartão por cliente**, uma PDA a partir do prefixo, da cafeteria e do cliente, guardando a contagem de carimbos e a contagem de resgates. Uma conta por cliente, porque o arquivo do arquiteto diz que contas graváveis compartilhadas serializam toda transação que as toca, e dois clientes no caixa ao mesmo tempo não deveriam esperar um pelo outro.
- Três instruções: **create shop** (a cafeteria assina), **stamp** (a cafeteria assina, o cartão do cliente é escrito), **redeem** (o cliente assina, dez carimbos saem do cartão). O arquiteto, ou o `/plan-feature` depois dele, vai perguntar quem paga o rent de um cartão novo, e *a resposta honesta para uma cafeteria é a cafeteria.*
- Invariantes como carimbos nunca ficam negativos e um resgate sempre custa exatamente o número configurado, um modelo de autoridade para a chave da cafeteria, e uma lista de riscos que começa com a chave da cafeteria estar em um celular.

![A faixa de design: três linhas de ideia vão para o solana-researcher, que devolve achados citados com rótulos de confiança, depois para o solana-architect, que devolve contas, instruções, invariantes e riscos em design.md, depois para o /plan-feature, que devolve um plano em fases, com o game-architect assumindo a etapa de design para jogos.](assets/v01-design-lane.webp)

4. **Transforme o design em fases.** Digite:

```text
/plan-feature Stamp, a partir de design.md
```

O comando não escreve código. Ele lê o design, pergunta sobre decisões que o alteram e devolve um plano com nove seções, resumo, contas, instruções, integrações, clientes, fases, riscos, testes e rollout, descartando as que não se aplicam e **terminando em perguntas em aberto para você**. Responda a elas em `design.md`, para que os próximos agentes leiam um só arquivo.

5. **Se você vai a um hackathon, adicione o pack de startup.** O `/add-skill startup-builder` instala o pack inteiro de dez skills, idea-sprint, positioning, pricing, pitch-deck, a skill hackathon, build-status, fundraising, launch, incident-comms e ecosystem-bd, porque elas apontam para os arquivos umas das outras. A idea-sprint dele escreve `.claude/context/idea.md`, e o `/plan-feature` parte desse arquivo quando ele existe. O Colosseum Copilot, do aparte de pesquisa acima, é o que a skill hackathon dele lê para conferir o que já foi feito antes.

## Três sementes de ideia para tentar em vez do Stamp

Os mesmos dois prompts funcionam com qualquer ideia. Troque as três linhas e mantenha o resto:

- **Vaquinha**: amigos juntam SOL para um presente e o pote paga quando a meta é atingida, ou devolve para todo mundo quando o prazo vence. Primeira pergunta de pesquisa: o que existe para potes em grupo e escrow, e como eles tratam o caminho do reembolso?
- **Biblioteca de ferramentas**: um galpão do bairro empresta furadeiras e escadas mediante uma caução que volta na devolução. Primeira pergunta de pesquisa: padrões de caução e devolução, e quem guarda a caução.
- **Lista da porta**: um evento pequeno vende um número fixo de entradas que não podem ser revendidas de jeito nenhum, só devolvidas ao organizador. Primeira pergunta de pesquisa: venda de ingressos na Solana, e quais projetos tentaram impedir a revenda.

## Está pronto quando

- `idea.md` tem três linhas e `design.md` tem contas, instruções, invariantes, autoridade e riscos.
- Cada achado na resposta da pesquisa carrega um link e um rótulo.
- As perguntas em aberto do plano estão respondidas em `design.md`.

## Fique atento

- **O arquiteto projeta e não codifica.** Se ele começar a escrever Rust, interrompa e peça o arquivo de design.
- **Pinocchio precisa de uma razão medida.** O arquivo do arquiteto diz isso: Anchor por padrão, e Pinocchio quando uma instrução continua acima do orçamento de compute depois das alavancas do próprio Anchor, quando as taxas de compute em volume ditam o custo do produto, ou quando o tamanho do binário ou o rent do deploy é uma restrição dura. Uma instrução quente é otimizada no lugar primeiro. Um programa em que a maioria das instruções é quente é escrito em Pinocchio desde o início.
- **Um rótulo Low é uma pista.** Não coloque um número Low em um deck.
- **Jogos vão para o game-architect.** Ele pergunta sobre gameplay, plataforma e o que precisa estar on-chain, depois escreve `concept.md` e `plan.md` e espera a sua aprovação antes de alguém implementar.

## O que fica

A pesquisa volta **citada e rotulada**, e um design volta como **contas, instruções, invariantes, autoridade e riscos** em um arquivo. *Peça a cada agente a saída que o próprio arquivo dele promete, e guarde o arquivo, porque os próximos agentes o leem.*

## A seguir

Na próxima lição o anchor-engineer constrói as três instruções do Stamp a partir de `design.md`, o engenheiro de QA as testa, e você vê os três comandos que fecham toda tarefa de programa. Rust, Anchor e a Solana CLI entram aí, então instale antes da sessão se quiser acompanhar no teclado. O `/doctor` já imprimiu o comando da Solana CLI, e o passo de scaffold do engenheiro traz a toolchain do Anchor à vista.
