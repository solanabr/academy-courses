# Tokens, DeFi e jogos: mais quatro especialistas

Na lição passada três engenheiros construíram a página do cartão, planejaram o app de celular e desenharam o indexador da cafeteria. Esta lição faz três perguntas que uma cafeteria de verdade faria em seguida: **os carimbos podem ser um token na carteira do cliente, o cliente pode pagar em qualquer moeda, e o cartão pode ser um jogo?** Cada pergunta tem um agente, e o arquivo de cada agente carrega a regra que impede o erro óbvio. Prompts e leitura, sem toolchain. Uns quinze minutos.

## Por que isso importa

Esses quatro agentes cobrem as partes do ecossistema que mudam com frequência, e onde a memória de um modelo costuma estar desatualizada: extensões de token que eram novidade no ano passado, uma API de swap que mudou de versão, um SDK de jogos com limites de plataforma. **token-engineer** constrói mints, extensões Token-2022, programas de transfer hook, coleções de NFT e planos de lançamento. **defi-engineer** integra Jupiter, Kamino, Marginfi, Raydium, Orca, Meteora e os oráculos, por SDK, API ou CPI. **game-architect** e **unity-engineer** desenham e constroem jogos na Solana. Os quatro leem skills de protocolo que o kit fixa em versão e instala sob demanda, e o arquivo do engenheiro de DeFi diz com todas as letras: *construa a partir da skill do protocolo, não da memória*.

## Mão na massa

1. **Transforme os carimbos em um token.** Digite dentro do Claude Code:

```text
Use o agente token-engineer. Os carimbos do Stamp devem aparecer na carteira do cliente como um token que ele não pode vender. Proponha o mint para a devnet: programa de token, o conjunto de extensões, quem detém cada autoridade e o que uma carteira vai mostrar. O menor número de extensões que dá conta do trabalho.
```

A expressão *menor número de extensões* é a primeira regra do próprio agente, citada de volta. O arquivo dele explica o motivo: **as extensões do mint são fixadas na criação, e cada extensão reduz onde o token pode ser negociado**, porque muitas pools, carteiras e agregadores rejeitam hooks, permanent delegates e contas congeladas por padrão. Para os carimbos, espere um mint Token-2022 com a extensão **non-transferable** e a **extensão metadata**, a cafeteria como mint authority, e uma nota de que um café grátis é um burn. Espere também a armadilha: a associated token account deriva do program id do Token-2022, e o id clássico dá o endereço errado.

Se um dia você lançar um token que as pessoas possam negociar, o mesmo agente carrega as **checagens pré-lançamento**: alocações documentadas, tokens do time com vesting on-chain, toda autoridade restante em um multisig da Squads ou revogada, e *revogar é permanente, então ele pergunta antes*. `/inspect-mint` decodifica as extensões e as autoridades atuais de qualquer mint a partir dos bytes brutos, e é assim que você confere o token de outra pessoa antes de integrá-lo.

2. **Deixe o cliente pagar em qualquer token.** Digite:

```text
Use o agente defi-engineer. Um cliente quer pagar o café com qualquer token que tiver, e a cafeteria quer USDC. Desenhe a etapa de swap com a Jupiter: qual API, como a cafeteria fica protegida de uma cotação ruim, e como você testaria. Só o design.
```

O arquivo dele nomeia a API atual, **swap v2 com chave de API**, e três checagens que uma integração apressada pula: **impor um output mínimo on-chain**, depois recarregar a conta de destino após o swap e conferir a diferença de saldo em vez da cotação. **Preço vindo de um oráculo** com feed id e idade máxima, nunca das reservas spot de uma pool, que um flash loan consegue mover. E **testar em um fork da mainnet no Surfpool**, para que as contas reais do protocolo carreguem. Ele vai pedir o pack `jupiter`, e o `sendai` se lending ou oráculos entrarem.

![A faixa de token, DeFi e jogos: token-engineer constrói o mint de carimbos com o menor número de extensões, defi-engineer adiciona um swap da Jupiter com output mínimo on-chain e preço de oráculo testado em um fork no Surfpool, e game-architect escreve concept.md e plan.md para o unity-engineer, com pagamentos verificados on-chain.](assets/v01-token-defi-game-lane.webp)

3. **Transforme o cartão em um jogo.** Digite:

```text
Use o agente game-architect. O Stamp quer um jogo pequeno: depois de cada café o cliente joga uma rodada de dez segundos no app da cafeteria e uma boa pontuação rende um carimbo bônus. Decida o que vive on-chain e o que não vive, escolha a plataforma e escreva concept.md e plan.md. Antes, me pergunte o que você precisa saber.
```

O arquivo do agente faz ele perguntar sobre jogabilidade, plataforma e o que precisa estar on-chain antes de escrever qualquer coisa, e depois dá a divisão: **on-chain o que tem valor, é negociável ou depende da confiança de outros jogadores**, off-chain a jogabilidade por frame e o estado da sessão, escritas em checkpoints em vez de por ação. A frase para procurar na resposta é esta: **pontuações reportadas pelo cliente estão sob controle do atacante**, então o carimbo bônus só é pago contra uma checagem que o cliente não consegue forjar. Ele escreve `concept.md` com números em vez de adjetivos e `plan.md` com passos numerados, e espera a sua aprovação. **unity-engineer** então constrói a partir desses dois arquivos em passos pequenos, editar, buildar, testar, e roda `/build-unity` e `/test-dotnet` antes de devolver, e o arquivo dele carrega as armadilhas do Unity: objetos que só vivem na thread principal, nenhuma thread C# no WebGL, texturas que nunca são coletadas.

4. **Escolha a sua variante.** O token, o swap e o jogo são três versões do Stamp que você poderia construir em seguida. Escreva a que você construiria no `design.md`, com o agente dono dela.

## Está pronto quando

- Você tem um conjunto de extensões proposto para o token de carimbos, com um responsável para cada autoridade.
- Você consegue dizer quais três checagens protegem a cafeteria no design do swap.
- `concept.md` e `plan.md` existem para o minijogo, com a divisão on-chain escrita.

## Fique atento

- **Cada um desses agentes pede um pack.** `metaplex`, `jupiter`, `sendai`, `solana-game`. O hub imprime o comando de instalação, `/add-skill <id>` roda, e `/update` mantém.
- **Ações que movem valor param.** Os arquivos de token e de DeFi dizem que o plugin de segurança intercepta ações de mainnet, de movimentação de valor e de autoridade, e que um ask ou deny é a sua política, reportada em vez de tentada por outro caminho.
- **Taxas de transferência reduzem o que o destinatário recebe.** Credite a diferença de saldo, não o argumento de valor. O arquivo de token diz isso, e o de DeFi diz o mesmo sobre o output de um swap: confira a diferença de saldo, nunca a cotação.
- **Mobile e PSG1 só quando você pedir.** O arquiteto de jogos começa por desktop e depois WebGL.

## O que fica

Quatro agentes para quatro partes do ecossistema que mudam rápido, e cada arquivo carrega **a regra que impede o erro óbvio**: menor número de extensões, output mínimo on-chain, oráculo em vez de spot, verificar o que paga. *Eles constroem a partir de skills de protocolo com versão fixada e pedem o pack de que precisam.*

## A seguir

Na próxima lição o Stamp vai ao ar: um deploy na devnet com a parada de aprovação, um pipeline de CI, docs que declaram o program id e o layout de cada conta, e uma revisão de produto por alguém que encontra o app pela primeira vez. O firewall aparece onde importa, no deploy.
