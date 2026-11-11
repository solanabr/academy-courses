# A sua ideia: três caminhos para adaptar

O Stamp foi de três linhas até um deploy na devnet passando por treze dos quinze agentes. Esta lição mostra **a mesma rota em três outras ideias**, cada uma como uma cadeia de agentes com o seu primeiro prompt, e termina com um template para a sua. Leitura e um texto para escrever. Uns dez minutos.

## Por que isso importa

A ordem que o Stamp seguiu é o formato do kit para qualquer ideia: pesquisa, design, plano, build, teste, auditoria, as partes que as pessoas tocam, deploy, docs, revisão. O que muda de uma ideia para outra é **quais agentes entram e qual pack eles pedem**. Quando você consegue escrever essa cadeia para uma ideia em dez linhas, pode começar a construir na manhã seguinte, e os agentes vão avisar quando você escolheu o errado, porque todo arquivo carrega uma seção Handoffs.

## O método, em um bloco

```text
1. idea.md, três linhas: uma pessoa, o que ela não consegue fazer hoje, o que a sua coisa faz
2. solana-researcher: o que existe, citado, datado, rotulado
3. solana-architect (game-architect para um jogo): contas, instruções, invariantes, autoridade, riscos → design.md
4. /plan-feature: fases, riscos, testes, perguntas abertas → responda a elas no design.md
5. anchor-engineer: constrói a partir do design.md, fecha com /build-program, /test-rust, /audit-solana
6. solana-qa-engineer: barra de cobertura, testes negativos, passada de slop
7. as partes que as pessoas tocam: solana-frontend-engineer, mobile-engineer, rust-backend-engineer
8. os especialistas de que uma ideia precisa: token-engineer, defi-engineer, unity-engineer
9. /deploy devnet, com a parada de aprovação
10. devops-engineer para o CI, tech-docs-writer para o README, /product-review para uma leitura sem contexto
```

## Caminho um: Vaquinha, um pote coletivo para presente

Amigos juntam SOL para um presente. O pote paga o organizador quando a meta é atingida, ou devolve a todos no prazo final.

- **solana-researcher**: o que existe para potes coletivos e escrow na Solana, e como cada um trata o caminho do reembolso. Espere rótulos Low em metade.
- **solana-architect**: uma PDA de pote por presente guardando meta, prazo e organizador, uma PDA de contribuição por contribuinte, três instruções, contribute, release, refund. A lista de riscos começa com *quem pode chamar release e quando*, e a checagem que eu ficaria de olho é o **prazo**: leia do sysvar de clock dentro do programa, nunca de um argumento, porque um argumento é o que quem chama disser que é.
- **anchor-engineer, depois solana-qa-engineer**: os testes de fronteira são o prazo menos um segundo e a meta menos um lamport.
- **solana-frontend-engineer**: uma página por pote com barra de progresso e botão de contribuir, simular primeiro, confirmed e não finalized.
- **defi-engineer, como variante**: contribuições em qualquer token, convertidas para USDC por um swap da Jupiter com o output mínimo imposto on-chain.
- O que você tem no fim: um pote na devnet e uma página que um amigo pode abrir.

Primeiro prompt para colar:

```text
Use o agente solana-researcher. O que existe na Solana para potes de pagamento em grupo ou escrow de crowdfunding, como cada um trata reembolsos depois de um prazo, e quais fecharam? Cite, date e rotule cada achado.
```

## Caminho dois: Biblioteca de ferramentas, cauções em um galpão de bairro

Um galpão empresta furadeiras e escadas. Quem pega deixa uma caução travada, e ela volta quando a ferramenta volta.

- **solana-researcher**: padrões de caução e devolução, e quem segura a caução enquanto a ferramenta está fora.
- **solana-architect**: uma PDA de item por ferramenta, uma PDA de empréstimo por empréstimo ativo, instruções para emprestar, devolver e reivindicar a caução depois de um período de carência. A pergunta que ele vai fazer a você é *quem confirma a devolução*, e o design honesto faz o voluntário do galpão assinar.
- **token-engineer, como variante**: uma associação que não pode ser vendida, um mint Token-2022 com a extensão non-transferable, conferido antes de um empréstimo ser permitido.
- **mobile-engineer**: o voluntário escaneia um código na porta do galpão, então o app que importa está em um celular, Android primeiro porque é onde o Mobile Wallet Adapter roda.
- **rust-backend-engineer**: um indexador pequeno para o galpão ver quais ferramentas estão fora e desde quando, idempotente em reentrega.
- O que você tem no fim: emprestar e devolver funcionando na devnet, e uma lista de membros.

Primeiro prompt para colar:

```text
Use o agente solana-architect. Desenhe um programa de empréstimo de ferramentas: itens, empréstimos com caução travada, devolução confirmada por um voluntário e reivindicação da caução depois de um período de carência. Cada conta com owner, seeds, tamanho e pagador do rent, cada instrução com seus signers, as invariantes, o modelo de autoridade, os riscos abertos. Anchor, a menos que medido o contrário. Escreva o design.md.
```

## Caminho três: Lista da porta, ingressos que não podem ser cambiados

Um evento pequeno vende um número fixo de entradas que não podem ser revendidas, só devolvidas ao organizador, que as vende de novo pelo valor de face.

- **solana-researcher**: venda de ingressos na Solana, e quais projetos tentaram limitar a revenda e o que aconteceu com eles.
- **token-engineer**: esta ideia é o trabalho dele. Um ingresso é um token, e a regra vive em um **programa de transfer hook** que roda em toda transferência e vê origem, mint, destino e owner, então consegue permitir uma transferência de volta ao organizador e recusar todas as outras, e o exemplo de hook do próprio kit é uma allowlist. A escolha é feita antes de o mint existir porque extensões são fixadas na criação. O arquivo dele avisa que mints com hook são rejeitados por muitas plataformas, *o que para um ingresso que não deve ser negociado é uma vantagem*.
- **token-engineer, de novo**: o programa do hook em si também está na trilha dele, com o discriminador da interface de hook que o arquivo nomeia, e o anchor-engineer constrói o resto do programa em volta.
- **solana-frontend-engineer**: a página da porta, escaneando uma carteira e mostrando um ingresso válido, com a conexão da carteira adiada até o valor aparecer.
- **devops-engineer**: o único caminho em que uma data de mainnet é real, então valem o plano de upgrade authority e as regras de chave no CI da lição 6.
- O que você tem no fim: um mint com hook na devnet e uma página da porta que diz sim ou não.

Primeiro prompt para colar:

```text
Use o agente token-engineer. Ingressos para um evento de 200 pessoas como um mint Token-2022 que os portadores podem devolver ao organizador e transferir para nenhum outro lugar. Proponha o conjunto de extensões, a regra do transfer hook, quem detém cada autoridade e o que uma carteira mostra ao portador. O menor número de extensões que dá conta do trabalho, e diga quais plataformas vão rejeitar o mint.
```

![Três caminhos de ideia como cadeias de agentes: Vaquinha passa por researcher, architect, anchor, QA, frontend e uma variante DeFi, Biblioteca de ferramentas passa por researcher, architect, token, mobile e backend, Lista da porta passa por researcher, token-engineer com um transfer hook, anchor, frontend e devops, todos terminando em /deploy devnet.](assets/v01-three-threads.webp)

## Escreva o seu

Copie o bloco e preencha. Mantenha as três linhas da ideia honestas, e ponha um nome em cada passo, porque *um agente chamado pelo nome faz o trabalho que o arquivo dele descreve, e um pedido sem nome vai para onde a correspondência de descrição cair*.

```text
<nome>
<uma pessoa>, <o que ela não consegue fazer hoje>, <o que a sua coisa faz a respeito>

1. solana-researcher: <a única pergunta que você precisa responder primeiro>
2. <architect>: <as duas ou três contas que você espera, em palavras>
3. anchor-engineer, solana-qa-engineer: <o caso de fronteira que um teste precisa atingir>
4. <o agente voltado para pessoas>: <a única tela que precisa existir>
5. <o especialista, se houver>: <o pack que ele vai pedir>
6. /deploy devnet, depois tech-docs-writer e /product-review
```

Quando um backlog é real e você tem um dia para acompanhar, o kit também encadeia os passos para você: `/intake` lê issues para uma lista de trabalho, `/research`, `/decide` e `/plan-work` fecham o plano, `/implement`, `/review-round` e `/fix-wave` constroem e conferem, e **`/crew` roda esses sete e para nos seus gates**, sempre no plano e sempre antes de lançar. `/ship` é o único passo dessa cadeia que `/crew` nunca roda por você. Eu começaria com a cadeia acima na mão, e usaria `/crew` no segundo projeto.

## Está pronto quando

- O seu caminho tem três linhas de ideia, um agente nomeado por passo e um primeiro prompt colado.
- Você consegue dizer qual agente é dono de um ingresso sem revenda, de um reembolso em grupo e de uma devolução confirmada por voluntário.

## Fique atento

- **Um jogo começa com game-architect**, não com solana-architect. A descrição do arquiteto de sistema diz isso e faz o handoff.
- **A maioria dos especialistas pede um pack.** Reserve a instalação na primeira hora.
- **Prazos vêm do sysvar de clock.** Um design que recebe a hora como argumento confia isso a quem chama, e essa é uma regra minha, não uma linha do kit.

## O que fica

Uma rota, dez passos, **um agente nomeado em cada um**, e a única coisa que muda de uma ideia para outra é quais especialistas entram. *Escreva a cadeia antes de escrever o prompt.*

## A seguir

A última lição é sobre continuar bom nisso depois do workshop: o agente guia que ensina no seu nível, o comando que explica código que você não escreveu, e como o kit se atualiza. Ela termina com um quiz sobre o curso inteiro.
