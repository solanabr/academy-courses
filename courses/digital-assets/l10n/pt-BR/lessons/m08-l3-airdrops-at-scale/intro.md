# Airdrops em escala: engenharia de custo e claims com vesting

## Resumo

Nas duas últimas lições você derivou o que é preciso para colocar o SPROUT numa curva, provou quais venues conseguem sequer segurar o mint dele, e escreveu a decisão que escolhe onde ele vai se graduar. O módulo 7 fez algo diferente: ensinou compressão como conceito, fez você precificá-la, e te deixou segurando uma promessa. Você raciocinou sobre tokens comprimidos por uma lição inteira sem nunca tocar em um.

Hoje você toca em um, nos primeiros dez minutos, e depois passa o resto da lição na aritmética que decide como um token chega a cem mil estranhos.

Comece pelo número, antes de qualquer explicação:

```bash
# 293 bytes (165 of data + the 128-byte account header) at your cluster's
# rent rate. Ask for the rate instead of pasting one; mine is mainnet's, read
# 2026-09-06, and SIMD-0437 is stepping it down.
RATE=$(curl -s https://api.mainnet-beta.solana.com -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getMinimumBalanceForRentExemption","params":[0]}' \
  | node -e "let d='';process.stdin.on('data',c=>d+=c).on('end',()=>console.log(JSON.parse(d).result/128))")
node -e "const classic=293*$RATE, compressed=10_300, n=100_000; console.log('rate', $RATE, 'lamports/byte | classic', (classic*n/1e9).toFixed(2), 'SOL / compressed', (compressed*n/1e9).toFixed(2), 'SOL /', ((1-compressed/classic)*100).toFixed(1)+'% saved')"
```

```
rate 6333 lamports/byte | classic 185.56 SOL / compressed 1.03 SOL / 99.4% saved
```

Quase duzentos SOL contra um. É a mesma distribuição, para as mesmas cem mil carteiras, precificada de dois jeitos. E a razão de esta lição existir é que nenhum dos dois números é o custo inteiro, os dois escondem uma decisão de escopo, e o método que você de fato entrega para o SPROUT é um terceiro que nenhum dos dois números descreve.

Você vai construir o compost-airdrop: uma tabela de custo que calcula os lamports por destinatário nos quatro métodos de distribuição e bate com os valores canônicos, mais um caminho de claim de Merkle que prova um claim desbloqueado e um claim `claim_locked` com vesting linear. O recuo, dito de saída: o modelo de custo é percorrido com você linha por linha, os valores comprimidos são um problema de completion que você preenche sozinho, e o segundo caminho de claim, o bloqueado, é inteiramente seu.

## O custo da distribuição

### Dez minutos de primeiro contato

Antes de qualquer teoria, coloque um token comprimido nas suas próprias mãos. Isso é uma tarefinha de devnet, não um build, e existe para que todo número mais adiante na lição se prenda a algo que você rodou.

O SDK da Light é uma stack web3.js v1. A faixa de peer publicada dele é `@solana/web3.js >=1.73.5`, e a Light não publica nenhuma superfície kit, então a viagem de v1 é inevitável em vez de escolhida: este lab é um workspace v1 em quarentena, de propósito, você escreve idiomas de v1 só onde o SDK do fornecedor obriga, e o resto do curso fica no kit. (O `latest` do kit no npm era 8.0.0, publicado em 2026-08-21, quando esta lição foi escrita. Não imprima `latest` num package.json, e reconfira cada pin abaixo antes de depender dele.)

```bash
mkdir -p labs/m08-l3 && cd labs/m08-l3
npm init -y
npm pkg set type=module
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
npm install @solana/web3.js@1.98.4 @solana/spl-token@0.4.15 \
  @lightprotocol/stateless.js@0.23.3 @lightprotocol/compressed-token@0.23.3
mkdir -p compost-airdrop
```

O `tsx` roda um arquivo TypeScript direto. O `@lightprotocol/stateless.js` conversa com o programa de sistema da Light e com um indexador Photon; o `@lightprotocol/compressed-token` é a camada de token em cima dele. Os dois estavam na 0.23.3 quando isto foi escrito. A linha `npm pkg set type=module` não é opcional: os dois pacotes da Light já vêm como módulos ES, e sem ela todo import do aquecimento falha no typecheck com uma reclamação de `require`.

Você precisa de um endpoint de devnet que sirva a API de compressão, porque uma conta comprimida não é legível com `getAccountInfo`. Qualquer provedor com suporte a ZK Compression funciona; a Helius publica um tier gratuito que fala isso, que foi o que eu usei.

```typescript
// compost-airdrop/warmup.ts
import { createRpc } from "@lightprotocol/stateless.js";
import { compress, createMint } from "@lightprotocol/compressed-token";
import {
  createAssociatedTokenAccount,
  mintTo as splMintTo,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const RPC_URL = process.env.DEVNET_RPC;
if (!RPC_URL) throw new Error("set DEVNET_RPC to a devnet endpoint with ZK Compression support");

const payer = Keypair.fromSecretKey(
  new Uint8Array(JSON.parse(readFileSync(process.env.KEYPAIR ?? "", "utf8"))),
);

// One URL, three roles: Solana RPC, Photon compression API, prover.
const rpc = createRpc(RPC_URL, RPC_URL, RPC_URL);

async function main(): Promise<void> {
  const { mint } = await createMint(rpc, payer, payer.publicKey, 9);
  console.log(`mint ${mint.toBase58()}`);

  const ata = await createAssociatedTokenAccount(rpc, payer, mint, payer.publicKey);
  await splMintTo(rpc, payer, mint, ata, payer, 1_000_000_000);
  console.log(`classic ATA ${ata.toBase58()} holds 1 token`);

  const signature = await compress(
    rpc,
    payer,
    mint,
    1_000_000_000,
    payer,
    ata,
    payer.publicKey,
  );
  console.log(`compressed in ${signature}`);

  // getAccountInfo would return nothing here. This read goes through Photon.
  const accounts = await rpc.getCompressedTokenAccountsByOwner(payer.publicKey, { mint });
  for (const account of accounts.items) {
    console.log(
      `compressed account: amount ${account.parsed.amount.toString()} ` +
        `owner ${account.parsed.owner.toBase58()} ` +
        `leafIndex ${account.compressedAccount.leafIndex}`,
    );
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
```

Rode com o seu par de chaves da devnet e um saldo financiado:

```bash
DEVNET_RPC="<your devnet rpc>" KEYPAIR="$HOME/.config/solana/id.json" npx tsx compost-airdrop/warmup.ts
```

Você deve ver um endereço de mint, uma ATA, uma assinatura de compress, e depois uma linha descrevendo uma **conta de token comprimida**: um saldo que vive como uma folha hasheada numa árvore de estado, com o livro-razão guardando o conteúdo dela e um indexador reconstruindo isso sob demanda. Não existe conta naquele endereço. O `leafIndex` na saída é o sinal honesto: o que você tem é uma posição numa árvore, não um slot no banco de dados de contas.

Repare no formato dessa última leitura. Você perguntou ao Photon, não ao validador. Esse é o imposto de leitura que o módulo 7 te cobrou nos cNFTs, agora cobrado sobre um saldo. E porque é uma leitura de índice, ela pode ficar atrás do livro-razão: se a lista de contas comprimidas voltar vazia numa rodada cuja assinatura de compress imprimiu certinho, nada falhou; o Photon só não alcançou ainda. Espere alguns segundos e rode a leitura de novo antes de suspeitar do compress.

Esses são os seus dez minutos. Agora, o dinheiro.

### Quanto custa um destinatário

Dois números fazem todo o trabalho aqui, e os dois são por destinatário.

Uma conta de token SPL clássica tem 165 bytes de dados mais o header de conta de 128 bytes que o runtime adiciona, ou seja, 293 bytes, e a isenção de aluguel precifica cada um desses bytes a uma taxa por byte. Os bytes são fixados pelo layout da conta. A taxa não é: ela é um parâmetro de rede, o SIMD-0437 está baixando ela em etapas, e os clusters não estão no mesmo passo. Então meça em vez de acreditar em mim, e meça no cluster em que você vai pagar:

```bash
solana rent 165 --url mainnet-beta
solana rent 165 --url devnet
```

```
Rent-exempt minimum: 0.001855569 SOL
Rent-exempt minimum: 0.00148844 SOL
```

Essas são as minhas leituras de 2026-09-06: 1,855,569 lamports na mainnet, que é exatamente 293 vezes 6,333, e 1,488,440 na devnet, exatamente 293 vezes 5,080. Antes de o primeiro degrau do corte aterrissar na mainnet em 2026-09-03 a taxa era 6,960 em todo lugar e este número era 2,039,280, que é o que você ainda vai encontrar na maioria dos posts de blog e, na hora em que isto foi escrito, num fork de surfpool, já que um fork herda as contas da mainnet e não necessariamente a tabela de rent dela. A aritmética de bytes não é uma história contada sobre o número, ela É o número, à taxa que o seu cluster cobrar hoje. Toda célula clássica na tabela abaixo é 293 vezes uma taxa, então a tabela é tão atual quanto essa taxa. Recalcule antes de orçar; a aritmética é uma multiplicação.

Um destinatário comprimido custa cerca de 10,300 lamports, e o modelo de custo da m07-l3 já te disse por quê: 5,000 lamports para criar a conta comprimida, mais cerca de 5,300 lamports de custo de estado pela única escrita que coloca tokens nela. Cria uma vez, escreve uma vez, pronto.

Nada mais no seu orçamento de airdrop importa tanto quanto essa razão, então coloque ela na página em quatro escalas:

| Destinatários | ATAs clássicas | Comprimido | Economizado |
|---|---|---|---|
| 1,000 | 1.86 SOL | 0.0103 SOL | 99.4% |
| 10,000 | 18.56 SOL | 0.103 SOL | 99.4% |
| 100,000 | 185.56 SOL | 1.03 SOL | 99.4% |
| 1,000,000 | 1,855.57 SOL | 10.30 SOL | 99.4% |

Coluna clássica aos 6,333 lamports/byte da mainnet, lidos em 2026-09-06. Multiplique pela sua própria taxa sobre 6,333 para chegar à sua; a coluna comprimida não se move, porque o custo de um destinatário comprimido não é rent.

A razão é plana porque os dois lados são lineares. O que muda com a escala é se o número é sobrevivível. Com mil destinatários ninguém liga. Com um milhão, a coluna clássica passa de 1,800 SOL, e a um preço de SOL de $150 isso é quase $300,000 de rent para distribuir um token. Um milhão de contas custa uma casa, e custava uma casa um pouco maior mês passado.

![Gráfico de barras agrupadas em eixo logarítmico comparando o custo de airdrop clássico e comprimido de 1k a 1M de destinatários, com a coluna clássica perto de dois mil SOL contra 10.3 SOL do comprimido.](assets/v01-chart.webp)

Esse quarto de milhão de dólares é a razão de a compressão ZK ter sido construída. A Solana passou de 500 milhões de contas e estava somando cerca de um milhão por dia por volta de novembro de 2024, que foi o enquadramento que a Helius usou no texto da keynote de compressão dela naquele mês. O crescimento de estado é o custo, e airdrops são o jeito mais rápido de inflá-lo.

A coluna clássica merece uma nota de honestidade, porque ela bajula a compressão se você pular isso. Rent é um depósito. Feche a conta e cada lamport dele volta. Os 10,300 comprimidos são gastos e nunca voltam. Então a frase correta não é "compressão é 200 vezes mais barata", é "compressão converte um depósito grande e reembolsável em um custo pequeno e permanente", e se isso é uma boa troca depende de alguém algum dia ir fechar essas contas. Num airdrop, quase ninguém fecha.

### Quatro jeitos de mover um token até um estranho

A tabela de custo só é útil se cobrir os métodos que você pode de fato escolher, e são quatro.

**Empurrar para ATAs clássicas.** Você paga rent por cada destinatário, eles não fazem nada, os tokens simplesmente estão lá. Simples, universalmente compatível, e a coluna que custa uma casa.

**Empurrar tokens comprimidos.** Você paga cerca de 10,300 lamports de estado por destinatário, eles seguram uma folha, e toda leitura posterior passa por um RPC de compressão. O Helius AirShip é a versão empacotada disso: instale globalmente, aponte para um mint e uma lista de destinatários, deixe ele fazer os lotes.

```bash
npm install -g helius-airship@0.9.4
helius-airship --help
```

**Claim de Merkle.** Você publica uma raiz de 32 bytes na chain e nunca toca numa conta de destinatário. Cada claimant prova a participação e paga pelo próprio claim. O seu custo por destinatário desaba para um setup fixo dividido por N, e o custo deles é uma taxa de transação mais o rent de uma pequena conta de status que os impede de fazer claim duas vezes. O próprio README do distribuidor coloca o custo líquido para um usuário em torno de 0.000010 SOL depois que ele fecha as contas de novo. Segure essas duas propriedades uma contra a outra por um segundo, porque as duas não podem ser incondicionalmente verdadeiras: a proteção contra replay é "o PDA de ClaimStatus já existe," e um PDA que um claimant pode fechar no meio da janela é um PDA que ele pode reabrir para um segundo claim. O programa precisa restringir o fechamento, pelo fim do vesting, pelo clawback ou pelo estado do distribuidor, e qual cancela ele de fato escolhe é um fato que você lê nas constraints de conta da instrução de fechamento, não numa frase de README. Acrescente isso à lista de leitura de última instrução que esta lição não para de aumentar.

**A primitiva Light Claim.** O equivalente em estado comprimido de um claim: uma conta comprimida que o destinatário materializa quando aparece, o que mantém o custo do remetente perto de zero e mantém o armazenamento do destinatário comprimido. É a mais nova das quatro, herda a exigência de leitura comprimida, e carrega a ressalva que o fechamento da m07-l3 já levantou sobre o trilho de token da Light: uma superfície emergente, devnet-first, sem constantes de custo estabelecidas e datadas até 2026-08. Essa ausência é por que o modelo de custo abaixo precifica os outros três métodos e adiciona uma linha de escopo `airship-tx-side` no quarto slot em vez de inventar um número de Light Claim; quando a Light publicar constantes de primeira mão, a tabela ganha uma linha honesta.

O eixo que ninguém coloca na página de marketing é quem paga.

![Tabela comparativa das quatro linhas do modelo de custo sobre custo do remetente, custo do claimant, reembolsabilidade e necessidades de RPC, notando que a linha do AirShip é uma fatia de escopo em cima do push comprimido e que o Light Claim está sem preço.](assets/v02-comparison.webp)

Leia essa tabela duas vezes. Um claim de Merkle não é barato, ele é *deslocado*. Os lamports não sumiram, eles passaram para a pessoa que recebe os tokens, e isso é uma decisão de produto tanto quanto uma decisão de custo: todo mundo que não faz claim não te custa nada, e todo mundo que faz claim paga cerca de 1.2 milhão de lamports para isso à taxa de rent atual da mainnet, a maior parte recuperável só se o caminho de fechamento do ClaimStatus deixar um claimant reaver o rent, uma cancela que esta lição sinaliza abaixo como algo que ela não rodou. Para um drop em que você espera que metade da lista te ignore, isso é uma dádiva. Para um drop a usuários que nunca tiveram SOL, é um muro.

### A lacuna do AirShip, medida e não discutida

Aqui está uma discrepância que você vai encontrar em menos de uma hora pesquisando isso por conta própria, e o jeito que você lida com ela importa mais que o número.

O material do próprio AirShip descreve um drop de 10,000 destinatários custando cerca de 0.01 SOL. A matemática de compressão por destinatário desta lição diz 10,000 vezes 10,300 lamports, que dá 0.103 SOL. Isso é um fator de dez, entre duas fontes que são as duas credíveis, sobre a mesma ferramenta.

Não tire a média delas. Não escolha a que você prefere. Abra a fonte e conte o que cada uma conta.

As constantes do AirShip ficam em `packages/core/src/config/constants.ts` no repositório helius-labs/airship, e lê-las (2026-08) resolve isso em cerca de um minuto:

```
maxAddressesPerTransaction = 15
baseFee                    = 5,000 lamports
compressionFee             = 1,500 * 3 = 4,500 lamports
computeUnitLimit           = 550,000
computeUnitPrice           = 10,000 micro-lamports
```

Uma taxa de prioridade de 550,000 unidades a 10,000 micro-lamports por unidade dá 5,500 lamports. Some a taxa base e a taxa de compressão e uma transação do AirShip custa 15,000 lamports para o remetente. Essa transação carrega 15 destinatários. Então o número por destinatário do AirShip é 1,000 lamports, e 10,000 destinatários vezes 1,000 lamports dá exatamente 0.01 SOL.

Os dois números estão certos. Eles contam coisas diferentes. O AirShip está citando o que sai da carteira do remetente em taxas, e o valor de 10,300 é o custo de estado das próprias contas comprimidas. Nenhum dos dois está mentindo; o escopo nunca foi declarado, então os dois números nunca foram comparáveis. O tudo-incluso por destinatário é a soma dos dois, cerca de 11,300 lamports, que é o número que você deveria colocar num orçamento.

Esse é o método inteiro. Quando um valor de fornecedor e uma derivação de primeira mão discordam por uma ordem de grandeza, a lacuna é quase sempre escopo, e a jogada honesta é medir em vez de citar. A sua tabela de custo ganha uma linha `airship-tx-side` explícita exatamente por isso: uma linha cujo label diz o que ela conta não pode ser citada fora de escopo depois.

Tem uma segunda coisa escondida naquele arquivo de constantes, e ela é a razão de o 15 existir. Uma transação tem teto de 1,232 bytes e cada conta que ela nomeia custa 32 deles, então juntar quinze destinatários mais as contas de sistema da Light, a pool de token e a árvore de estado numa transação só não cabe se você escrever cada endereço por extenso. O AirShip não escreve. Ele já vem com uma address lookup table, `9NYFyEqPkyXUhkerbGHXUXkvb4qpzeEdHuGpgbgpH1NJ` na mainnet e outra separada na devnet, guardando as contas estáticas de que toda transação de drop precisa, para que elas custem um byte de índice cada em vez de 32. O tamanho do lote não é uma preferência de ajuste, é o que o orçamento de bytes permite quando as contas fixas estão numa tabela.

![Duas barras de orçamento de bytes para uma transação de 1,232 bytes mostrando que trocar endereços de conta estáticos completos de 32 bytes por índices de lookup table de um byte deixa espaço para quinze destinatários em uma transação do AirShip.](assets/v03-diagram.webp)

### O caminho de claim, e por que o SPROUT tem que pegar ele

Agora a parte constrangedora, e ela é específica do token que você vem construindo o curso inteiro.

O conjunto de lançamento do SPROUT vindo da R6 são três extensões: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`. A tabela de extensões suportadas do AirShip, no mesmo arquivo de constantes, marca `transfer_fee_config` como não suportada, junto com `transfer_hook`, `permanent_delegate`, `mint_close_authority`, `default_account_state`, e o conjunto confidencial; `metadata_pointer`, `metadata`, `interest_bearing_config`, e os ponteiros de group são suportados.

Então duas das três extensões do SPROUT pegariam carona numa boa, e a terceira mata o drop inteiro, porque allowlisting também não é aditivo aqui, a mesma lição que a allowlist do CP-Swap te ensinou na R6, num trilho diferente. A cobertura de Token-2022 do programa de token comprimido segue mais ou menos a mesma linha, e não é arbitrário: uma taxa que precisa ser retida num slot por conta não tem para onde ir quando a conta é um hash numa árvore.

![Tabela separando as extensões Token-2022 por suporte a airdrop comprimido, com o par de metadados do SPROUT na coluna das suportadas e o TransferFeeConfig dele não suportado, desqualificando o caminho de compressão.](assets/v04-table.webp)

Então a coluna mais barata da sua tabela está indisponível para o seu próprio token. E uma reconciliação que você merece, porque a m07-l3 te avaliou pela conclusão oposta: o memo que aquela lição aceitou nomeou o compost drop como a única carga de trabalho da Overgrowth que genuinamente deveria comprimir, e nos eixos de custo que aquele memo argumentou, contagem de escritas e formato de acesso, ele estava certo. O que aquele memo assumiu em silêncio é que a cancela de extensões passa, e esta lição é onde a suposição finalmente é checada e falha especificamente para o SPROUT. O veredicto vira pela legalidade, não pela aritmética; o seu memo estava certo sobre a carga de trabalho e cego para o token, que é precisamente por que a ordenação desqualificadores-antes-da-aritmética que você codificou lá precisava de mais um desqualificador que ela ainda não conhecia. Leve isso como a lição em vez de uma derrota: o modelo de custo escolhe entre métodos que são legais para o token, e a legalidade vem do conjunto de extensões que você escolheu lá no módulo 2. Se você quer a coluna comprimida, você projeta para isso antes de cunhar.

O que deixa o caminho de claim, e a implementação de referência dele é um programa de verdade com uma história de verdade. A Jito construiu um distribuidor de Merkle para o airdrop do JTO: um programa Anchor de código aberto que carrega uma raiz de 32 bytes, entrega uma porção desbloqueada na hora, e libera uma segunda porção, bloqueada, linearmente até uma data final fixa. Para o JTO esse vesting foi até 7 de dezembro de 2024. O programa está deployado em `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv` e eu confirmei que ele ainda é executável na mainnet enquanto escrevia isto.

A ressalva honesta, porque ela é estrutural para as suas escolhas de dependência: o repositório está parado. O último push dele foi em 2025-04-30, uns dezesseis meses antes desta lição. Parado não é a mesma coisa que quebrado, e um distribuidor é um programa pequeno com um trabalho congelado, mas "a gente forka um repo que ninguém tocou em mais de um ano" é uma frase que o seu time deveria dizer em voz alta em vez de descobrir depois.

E uma segunda ressalva, a que o método deste módulo exige antes de você aceitar o meu roteamento: o distribuidor é um programa Anchor da era SPL, construído para o JTO, um mint SPL clássico. Se o vault dele e a CPI de transferência na hora do claim conseguem sequer segurar e pagar um mint Token-2022, quanto mais um cujo TransferFeeConfig retém uma fatia de todo claim de modo que os claimants recebem valores líquidos de taxa que as suas alocações de Merkle nunca modelaram, é uma pergunta de última instrução, e eu não rodei, então esta lição não responde. Antes de o SPROUT entregar este plano: leia a CPI de transferência do programa (ela invoca qualquer programa de token que seja dono do mint, ou um id clássico hardcoded?), suba um distribuidor de devnet financiado com um mint Token-2022 que cobra taxa, e faça claim contra ele. Se qualquer uma das duas checagens falhar, o plano recua para jogadas que este curso já ensinou: isentar o fluxo do distribuidor das taxas e acertar as contas com o harvest da taxa retida, forkar o caminho de claim para `transfer_checked`, ou se apoiar na divisão de dois mints da R6. Vetos de venue antes da matemática se aplicam ao caminho recomendado exatamente com a mesma força com que se aplicaram aos rejeitados.

O distribuidor também tem uma resposta para a parte da sua lista que nunca aparece, e vale projetar para isso antes de lançar em vez de depois. O estado dele carrega um `clawback_start_ts`, um `clawback_receiver`, e uma flag `clawed_back`. Antes daquele timestamp uma tentativa de clawback falha com `ClawbackBeforeStart`. Depois dele, qualquer um pode disparar a varredura, e é seguro deixar assim porque o destino está fixado: os tokens só podem aterrissar no `clawback_receiver` com que o distribuidor foi criado. Uma vez varrido, todo claim restante falha com `ClaimExpired`. Então a janela é uma política que você define no setup e não pode renegociar. Curta demais e você pune as pessoas que estavam de férias; longa demais e a sua tesouraria fica sentada em cima de tokens que ela não consegue planejar. Escolha deliberadamente, publique, e coloque a data no mesmo lugar em que você publica a árvore.

![Linha do tempo de um distribuidor de Merkle do setup, passando pela janela de vesting linear, até o ponto de clawback, depois do qual alocações sem claim são varridas e claims posteriores revertem.](assets/v05-timeline.webp)

### O que o claim_locked de fato faz

Um **distribuidor de Merkle** é uma conta que guarda uma raiz, um vault e contadores, mais uma pequena conta de status por claimant. Nada na chain conhece a lista de destinatários. O claimant traz a alocação dele e uma prova; o programa recalcula a folha e confere contra a raiz.

O layout da folha vale mostrar exatamente, porque um erro de ordem de bytes aqui produz uma prova que falha sem nenhum erro útil:

![Diagrama da pré-imagem da folha do distribuidor, fazendo o hash de uma pubkey de claimant de 32 bytes e de dois valores u64 little-endian em um nó, depois separando por domínio folhas e pais com bytes de prefixo.](assets/v06-annotated-code.webp)

Duas coisas decorrem dessa figura.

Primeiro, provas são pequenas mas não são de graça. Uma árvore de 100,000 folhas dá um caminho de prova de cerca de 17 hashes, 544 bytes, que cabe numa transação junto com tudo o mais de que um claim precisa. Compare com os dois outros formatos de prova que você encontrou neste curso: um claim de cNFT carrega um caminho completo por uma árvore de Merkle concorrente, que é por que essas árvores mantêm um **canopy**, uma faixa cacheada de nós de cima guardada na chain para que o cliente só precise mandar a parte de baixo do caminho; uma escrita de compressão ZK carrega uma **prova de validade** de 128 bytes, uma prova de conhecimento zero de tamanho constante de que a conta que ela está gastando existe na árvore. Três árvores, três estratégias de prova, três coisas diferentes acabando na sua transação.

Segundo, e esta é a cilada que vai de fato te morder: toda escrita numa árvore muda a raiz dela, e toda prova pendente contra a raiz antiga vira lixo. Para o distribuidor a raiz é fixada no setup, então isso te morde durante a preparação e não na hora do claim. Para tokens comprimidos morde o tempo todo, porque a árvore de estado recebe escritas de todo mundo. A regra é a mesma nos dois mundos. Busque a prova imediatamente antes de enviar, nunca de um cache, nunca de um arquivo que você gerou semana passada.

A metade do vesting é uma única função, e é curta o bastante para caber na sua cabeça. O distribuidor guarda `start_ts` e `end_ts`; a conta de status do claimant guarda `locked_amount` e `locked_amount_withdrawn`. O que o `claim_locked` pode pagar agora é a parcela já liberada pelo vesting menos o que já foi tirado:

```
vested      = 0                                    if now < start_ts
vested      = locked_amount                        if now >= end_ts
vested      = (now - start_ts) * locked_amount / (end_ts - start_ts)  otherwise
withdrawable = vested - locked_amount_withdrawn
```

Divisão inteira trunca, o que arredonda para baixo, o que favorece o vault em no máximo uma unidade base. Isso é de propósito e é o tipo de detalhe que vale copiar em vez de melhorar.

![Um gráfico de vesting onde uma linha reta acumula 900 milhões de unidades base ao longo de 90 dias enquanto uma escada de chamadas de claim_locked nos dias 30, 45 e 90 vai alcançando.](assets/v07-chart.webp)

Duas propriedades desse design merecem ser nomeadas. É um pull, então alocações sem claim ficam paradas no vault sem te custar nada. E é idempotente por claimant, porque a conta de status é um PDA derivado com seeds do claimant e do distribuidor: uma segunda tentativa de abrir ela falha na criação da conta, não numa checagem escrita à mão. É aí que os botões de vesting da lição de launchpad também pousam. O LaunchLab expressava bloqueios como um cliff mais uma duração pelo lado do launchpad; o distribuidor os expressa como `start_ts` e `end_ts` pelo lado da distribuição. Mesma ideia, cadeira diferente.

![Fluxograma de duas faixas com o operador publicando uma raiz de Merkle de 32 bytes uma vez enquanto cada claimant busca uma prova, chama new_claim, e depois chama claim_locked repetidamente conforme o vesting acumula.](assets/v08-flowchart.webp)

### O trade-off, nomeado

A compressão corta o rent de airdrop em bem mais de 99%, e aqui está a fatura disso.

Uma transferência de token comprimido custa uns 292,000 compute units, porque o programa verifica uma prova de validade e refaz os hashes do estado da árvore a cada escrita. O caminho clássico que você mediu no módulo 1, no motor p-token, é 76 CU para um `Transfer`. Não coloque esses dois números lado a lado como se fossem implementações concorrentes do mesmo produto. Um é um saldo numa conta, o outro é um saldo numa árvore mais uma prova; a diferença de compute é o que a economia de armazenamento custa, e ela é cobrada por escrita para sempre.

Saldos comprimidos também precisam de um RPC de compressão para serem lidos, geralmente querem ser descomprimidos antes de a maior parte do DeFi olhar para eles, e um token que carrega a extensão errada não pode usá-los de jeito nenhum.

O caminho de claim tem o próprio custo. Ele adiciona uma dependência de programa, neste caso uma cujo repositório está parado desde 2025-04-30. Exige uma segunda transação por destinatário para a porção com vesting, e uma terceira, e uma quarta, porque vesting linear quer dizer que um claimant volta quantas vezes ele quiser. Empurra cerca de 1.3 milhão de lamports de custo para cada claimant. E precisa de um artefato off-chain, a árvore e as provas dela, hospedado em algum lugar que os seus usuários alcancem.

Um limite antes do lab. Ler estado comprimido por um RPC de DAS ou Photon é consumo, e é aí que este curso para. Subir o indexador embaixo disso, plugins Geyser, streams gRPC, backfills, é território do curso planejado Client-Side Mastery, onde escolha de provedor e confiabilidade de índice são problemas de primeira classe em vez de uma linha num lab.

## Lab: construa o compost-airdrop

O artefato é o `compost-airdrop`, uma pasta de lição que fica ao lado do `sprout-launch/` das duas últimas lições (este módulo mantém os labs dele como pastas irmãs em vez de sob `labs/`; coloque onde o seu workspace guarda os outros, os caminhos nos comandos são todos relativos). Ele contém uma tabela de custo que calcula e faz assert dos valores canônicos, e um caminho de claim que prova tanto um claim desbloqueado quanto um com vesting. Ele consome o `sprout-mint`, o mint da R3 pelo nome dele na escada de artefatos, no sentido de que o conjunto de extensões do SPROUT é o que força a escolha de método que você acabou de ler.

Tudo depois do aquecimento é local e determinístico. Sem RPC, sem par de chaves, sem espera.

1. **O modelo de custo, percorrido.** Crie `compost-airdrop/cost-model.ts`. Toda constante ou é um valor congelado do curso ou é um valor lido de uma fonte pública, e o arquivo diz qual.

    ```typescript
    // compost-airdrop/cost-model.ts
    export const LAMPORTS_PER_SOL = 1_000_000_000;

    /** A classic SPL token account: 165 bytes of data plus the 128-byte account header. */
    export const CLASSIC_ATA_BYTES = 293;
    /**
     * Rent-exemption price of one byte for two years. THE ONLY NUMBER IN THIS
     * FILE THAT BELONGS TO THE NETWORK RATHER THAN TO THE LAYOUT, and the one
     * you must replace with your own read. mainnet-beta, 2026-09-06; devnet was
     * a step further down at 5,080 and a surfpool fork was still on the old
     * 6,960. Get yours in one line:
     *
     *   solana rent 0 --url <cluster>   # divide the lamports by 128
     *
     * SIMD-0437 is stepping this down over several releases, so a number you
     * copied from a lesson is a number you will re-derive.
     */
    export const LAMPORTS_PER_BYTE = 6_333;
    /**
     * Rent locked by one classic recipient account. DERIVED, not pasted: at
     * 6,333 this is 1,855,569, which is what `solana rent 165 --url
     * mainnet-beta` printed on 2026-09-06. Refundable if the account is closed.
     */
    export const CLASSIC_ATA_LAMPORTS = CLASSIC_ATA_BYTES * LAMPORTS_PER_BYTE;

    /** Creating one compressed token account (m07-l3's figure). */
    export const COMPRESSED_CREATE_LAMPORTS = 5_000;
    /** State cost of one compressed write on Light's V2 program line (m07-l3's figure). */
    export const COMPRESSED_WRITE_LAMPORTS = 5_300;
    /** One compressed recipient: created once, written once by the drop. */
    export const COMPRESSED_RECIPIENT_LAMPORTS =
      COMPRESSED_CREATE_LAMPORTS + COMPRESSED_WRITE_LAMPORTS;

    // AirShip's own transaction-side constants, read from
    // helius-labs/airship, packages/core/src/config/constants.ts (read 2026-08).
    export const AIRSHIP_BASE_FEE = 5_000;
    export const AIRSHIP_COMPRESSION_FEE = 1_500 * 3;
    export const AIRSHIP_CU_LIMIT = 550_000;
    export const AIRSHIP_CU_PRICE_MICRO_LAMPORTS = 10_000;
    export const AIRSHIP_RECIPIENTS_PER_TX = 15;

    export function airshipPriorityFeeLamports(): number {
      return Math.ceil(
        (AIRSHIP_CU_LIMIT * AIRSHIP_CU_PRICE_MICRO_LAMPORTS) / 1_000_000,
      );
    }

    export function airshipPerTransactionLamports(): number {
      return AIRSHIP_BASE_FEE + AIRSHIP_COMPRESSION_FEE + airshipPriorityFeeLamports();
    }

    /** AirShip's transaction-side cost per recipient. State cost is NOT in here. */
    export function airshipPerRecipientLamports(): number {
      return airshipPerTransactionLamports() / AIRSHIP_RECIPIENTS_PER_TX;
    }

    /** ClaimStatus: 8-byte discriminator + 32 + 8 + 8 + 8. */
    export const CLAIM_STATUS_BYTES = 64;
    /** Rent for the ClaimStatus PDA the claimant opens. Refundable on close. */
    export const CLAIM_STATUS_RENT_LAMPORTS =
      (CLAIM_STATUS_BYTES + 128) * LAMPORTS_PER_BYTE;
    export const CLAIM_TX_FEE_LAMPORTS = 5_000;

    export type MethodId = "classic" | "compressed" | "airship-tx-side" | "merkle-claim";

    export interface Method {
      id: MethodId;
      label: string;
      /** Lamports the drop operator pays per recipient. */
      senderLamports: number;
      /** Lamports the recipient pays to end up holding the tokens. */
      claimantLamports: number;
      /** Lamports on this row that come back if accounts are closed. */
      refundableLamports: number;
      note: string;
    }

    export function methods(): Method[] {
      return [
        {
          id: "classic",
          label: "classic SPL ATA, pushed",
          senderLamports: CLASSIC_ATA_LAMPORTS,
          claimantLamports: 0,
          refundableLamports: CLASSIC_ATA_LAMPORTS,
          note: `${CLASSIC_ATA_BYTES} bytes rent-exempt at ${LAMPORTS_PER_BYTE.toLocaleString(
            "en-US",
          )} lamports/byte; matches \`solana rent 165\``,
        },
        {
          id: "compressed",
          label: "compressed token, pushed",
          // TODO(you): a compressed recipient is created once and written once
          // by the drop. Both constants are already defined above.
          senderLamports: 0,
          claimantLamports: 0,
          refundableLamports: 0,
          note: "state cost only: create + one write, nothing refundable",
        },
        {
          id: "airship-tx-side",
          label: "compressed via AirShip (transaction side only)",
          senderLamports: airshipPerRecipientLamports(),
          claimantLamports: 0,
          refundableLamports: 0,
          note: `${airshipPerTransactionLamports()} lamports per tx / ${AIRSHIP_RECIPIENTS_PER_TX} recipients`,
        },
        {
          id: "merkle-claim",
          label: "merkle claim (claimant-paid)",
          senderLamports: 0,
          claimantLamports: CLAIM_TX_FEE_LAMPORTS + CLAIM_STATUS_RENT_LAMPORTS,
          // Booked as spent, deliberately: ClaimStatus rent is refundable only
          // if the program's close path lets the CLAIMANT reclaim it, and that
          // gate is flagged unverified in the prose. Flip this to
          // CLAIM_STATUS_RENT_LAMPORTS only after you have run the close.
          refundableLamports: 0,
          note: "sender pays a fixed setup; the claimant pays the claim (rent refund unverified)",
        },
      ];
    }

    export function sol(lamports: number): string {
      return `${(lamports / LAMPORTS_PER_SOL).toFixed(4)} SOL`;
    }
    ```

    O `TODO` é seu e é o problema de completion desta lição. Uma linha, duas constantes, e a linha de 100k começa a ler certo.

2. **A tabela, e os asserts que a tornam um teste.** Crie `compost-airdrop/cost-table.ts`. Uma tabela que ninguém confere é uma tabela que apodrece silenciosamente, então o arquivo termina fazendo assert dos valores canônicos.

    ```typescript
    // compost-airdrop/cost-table.ts
    import {
      CLASSIC_ATA_BYTES,
      CLASSIC_ATA_LAMPORTS,
      COMPRESSED_RECIPIENT_LAMPORTS,
      LAMPORTS_PER_BYTE,
      airshipPerRecipientLamports,
      methods,
      sol,
    } from "./cost-model";

    const SIZES = [1_000, 10_000, 100_000, 1_000_000];

    function pad(s: string, n: number): string {
      return s.length >= n ? s : s + " ".repeat(n - s.length);
    }

    console.log(
      `${pad("method", 48)}${pad("lamports/ea", 12)}` +
        SIZES.map((n) => pad(n.toLocaleString("en-US"), 14)).join(""),
    );

    for (const m of methods()) {
      const perRecipient = m.senderLamports + m.claimantLamports;
      const cells = SIZES.map((n) => pad(sol(perRecipient * n), 14)).join("");
      console.log(
        `${pad(m.label, 48)}${pad(perRecipient.toLocaleString("en-US"), 12)}${cells}`,
      );
    }

    const classic100k = CLASSIC_ATA_LAMPORTS * 100_000;
    const compressed100k = COMPRESSED_RECIPIENT_LAMPORTS * 100_000;
    const saved = 1 - compressed100k / classic100k;

    console.log("");
    console.log(
      `100k recipients: classic ${sol(classic100k)} / compressed ${sol(compressed100k)} ` +
        `(${(saved * 100).toFixed(1)}% saved)`,
    );
    console.log(
      `AirShip's transaction side adds ${airshipPerRecipientLamports().toLocaleString("en-US")} ` +
        `lamports/recipient on top of the state cost.`,
    );

    const expect = (label: string, got: number, want: number, tol: number) => {
      if (Math.abs(got - want) > tol) {
        throw new Error(`${label}: got ${got}, expected about ${want}`);
      }
    };

    // Two kinds of assertion, and the difference matters. The compressed
    // figures are absolutes, because a compressed recipient's cost is not rent
    // and does not move with the rent schedule. The classic figures assert
    // CONSISTENCY with whatever LAMPORTS_PER_BYTE you set, because pinning them
    // to an absolute would turn this gate into a tripwire that fires every time
    // the network cuts rent, which is the opposite of what a cost model is for.
    expect("classic per recipient", CLASSIC_ATA_LAMPORTS, CLASSIC_ATA_BYTES * LAMPORTS_PER_BYTE, 0);
    expect("compressed per recipient", COMPRESSED_RECIPIENT_LAMPORTS, 10_300, 0);
    expect("classic at 100k (SOL)", classic100k / 1e9, (CLASSIC_ATA_LAMPORTS * 100_000) / 1e9, 0.001);
    expect("compressed at 100k (SOL)", compressed100k / 1e9, 1.03, 0.01);
    expect("saving", saved, 1 - COMPRESSED_RECIPIENT_LAMPORTS / CLASSIC_ATA_LAMPORTS, 0.0001);

    // The assertion that actually reads your TODO: the compressed ROW in
    // methods() must carry the per-recipient state cost, not the shipped zero.
    const compressedRow = methods().find((m) => m.id === "compressed");
    expect(
      "compressed row senderLamports (the TODO in cost-model.ts)",
      compressedRow?.senderLamports ?? 0,
      COMPRESSED_RECIPIENT_LAMPORTS,
      0,
    );
    console.log("cost table OK");
    ```

    Rode. Antes de você preencher o `TODO`, a tabela imprime um `0` sem sentido em lamports/ea para a linha comprimida e o assert da linha comprimida lá embaixo lança, que é justamente o ponto: a cancela lê a linha em que o seu TODO vive, não só as constantes ao redor dela.

    ```bash
    npx tsx compost-airdrop/cost-table.ts
    ```

    Depois de preencher, as últimas linhas ficam assim:

    ```
    100k recipients: classic 185.5569 SOL / compressed 1.0300 SOL (99.4% saved)
    AirShip's transaction side adds 1,000 lamports/recipient on top of the state cost.
    cost table OK
    ```

    Esses valores em SOL são `LAMPORTS_PER_BYTE` vezes uma contagem fixa de bytes, então eles são seus para mover: ajuste a constante para a taxa do seu próprio cluster e a tabela inteira segue, asserts incluídos.

3. **A árvore.** Crie `compost-airdrop/merkle.ts`. Isto é o hashing do distribuidor, portado exatamente: sha256, um byte zero na frente das folhas, um byte um na frente dos pais, pares ordenados, e um nó ímpar pareado consigo mesmo. Nenhuma dependência. Essa última regra é a que exige atenção, porque ela é invisível na árvore de quatro destinatários deste lab (quatro é uma potência de dois, então nenhum nível é ímpar) e decide toda raiz que você calcular numa lista de verdade. O Challenge 1 adiciona um quinto destinatário, que é onde ela começa a importar.

    ```typescript
    // compost-airdrop/merkle.ts
    // Ported from jito-foundation/distributor (merkle-tree/src/merkle_tree.rs,
    // programs/merkle-distributor/src/instructions/new_claim.rs), read on
    // 2026-08-22. m09-l2 ports the same file again under different names; if
    // the two ever disagree, one of them is wrong about a deployed program.
    import { createHash } from "node:crypto";

    export type Hash32 = Uint8Array;

    export function sha256(...parts: Uint8Array[]): Hash32 {
      const h = createHash("sha256");
      for (const p of parts) h.update(p);
      return new Uint8Array(h.digest());
    }

    export function u64le(value: bigint): Uint8Array {
      const out = new Uint8Array(8);
      new DataView(out.buffer).setBigUint64(0, value, true);
      return out;
    }

    export function hex(bytes: Uint8Array): string {
      return Buffer.from(bytes).toString("hex");
    }

    export interface Allocation {
      /** 32-byte claimant pubkey. */
      claimant: Uint8Array;
      amountUnlocked: bigint;
      amountLocked: bigint;
    }

    /** The node the program hashes, then the leaf prefix that stops second-preimage tricks. */
    export function leafHash(a: Allocation): Hash32 {
      const node = sha256(a.claimant, u64le(a.amountUnlocked), u64le(a.amountLocked));
      return sha256(Uint8Array.of(0), node);
    }

    function compare(a: Uint8Array, b: Uint8Array): number {
      for (let i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return a[i] < b[i] ? -1 : 1;
      }
      return 0;
    }

    function hashPair(a: Hash32, b: Hash32): Hash32 {
      return compare(a, b) <= 0
        ? sha256(Uint8Array.of(1), a, b)
        : sha256(Uint8Array.of(1), b, a);
    }

    export interface Tree {
      root: Hash32;
      leaves: Hash32[];
      proofFor(index: number): Hash32[];
    }

    export function buildTree(allocations: Allocation[]): Tree {
      if (allocations.length === 0) throw new Error("empty tree");
      const leaves = allocations.map(leafHash);

      const levels: Hash32[][] = [leaves];
      while (levels[levels.length - 1].length > 1) {
        const below = levels[levels.length - 1];
        const above: Hash32[] = [];
        for (let i = 0; i < below.length; i += 2) {
          // An odd node is paired with ITSELF, not promoted to the level above.
          // This is the line that decides whether your root equals the deployed
          // program's: merkle_tree.rs duplicates the last entry when a level's
          // length is odd, and promoting instead gives a different root for
          // every tree whose width is not a power of two.
          const right = i + 1 < below.length ? below[i + 1] : below[i];
          above.push(hashPair(below[i], right));
        }
        levels.push(above);
      }

      return {
        root: levels[levels.length - 1][0],
        leaves,
        proofFor(index: number): Hash32[] {
          if (index < 0 || index >= leaves.length) throw new Error("no such leaf");
          const proof: Hash32[] = [];
          let i = index;
          for (let level = 0; level < levels.length - 1; level++) {
            const nodes = levels[level];
            const sibling = i % 2 === 0 ? i + 1 : i - 1;
            // Same duplication on the proof side: find_path uses level[index]
            // itself when the right sibling is off the end, so the proof folds
            // through the self-pair the builder created.
            proof.push(sibling < nodes.length ? nodes[sibling] : nodes[i]);
            i = Math.floor(i / 2);
          }
          return proof;
        },
      };
    }

    /** The on-chain check, in TypeScript. Same order, same prefixes, same sorting. */
    export function verifyProof(proof: Hash32[], root: Hash32, leaf: Hash32): boolean {
      let computed = leaf;
      for (const element of proof) computed = hashPair(computed, element);
      return compare(computed, root) === 0;
    }
    ```

4. **A matemática do vesting.** Crie `compost-airdrop/vesting.ts`. Mesmos ramos, mesma truncagem, mesmos nomes de campo do `ClaimStatus` do programa.

    ```typescript
    // compost-airdrop/vesting.ts
    export interface ClaimStatus {
      claimant: string;
      unlockedAmount: bigint;
      lockedAmount: bigint;
      lockedAmountWithdrawn: bigint;
    }

    /** How much of the locked allocation has vested at currTs. */
    export function unlockedAmount(
      cs: ClaimStatus,
      currTs: number,
      startTs: number,
      endTs: number,
    ): bigint {
      if (currTs < startTs) return 0n;
      if (currTs >= endTs) return cs.lockedAmount;
      const timeIntoUnlock = BigInt(currTs - startTs);
      const totalUnlockTime = BigInt(endTs - startTs);
      // Integer division truncates, which rounds down in the vault's favour.
      return (timeIntoUnlock * cs.lockedAmount) / totalUnlockTime;
    }

    /** What claim_locked would actually transfer right now. */
    export function amountWithdrawable(
      cs: ClaimStatus,
      currTs: number,
      startTs: number,
      endTs: number,
    ): bigint {
      const vested = unlockedAmount(cs, currTs, startTs, endTs);
      if (vested < cs.lockedAmountWithdrawn) {
        throw new Error("arithmetic error: withdrawn exceeds vested");
      }
      return vested - cs.lockedAmountWithdrawn;
    }
    ```

    Confira a ordem dos ramos contra o programa antes de seguir em frente. Tempo antes de `start_ts` rende nada; tempo em ou depois de `end_ts` rende o valor bloqueado inteiro; no meio é uma proporção. Um início depois do fim não é um caso especial, ele simplesmente nunca começa.

5. **A rodada de claim.** Crie `compost-airdrop/claim.ts`. Isto conduz um distribuidor de quatro destinatários pelos dois caminhos de claim e por três falhas. O claim desbloqueado vem resolvido para você; o loop bloqueado é a metade solo, e está marcado. Uma nota de stack: este arquivo é simulação local pura e não chama nada da Light, mas ele vive no mesmo workspace v1 em quarentena do aquecimento, então ele reutiliza o `PublicKey` daquele workspace para base58 e aritmética de bytes em vez de misturar um segundo SDK numa pasta só.

    ```typescript
    // compost-airdrop/claim.ts
    import { PublicKey } from "@solana/web3.js";
    import { buildTree, hex, leafHash, verifyProof, type Allocation } from "./merkle";
    import { amountWithdrawable, type ClaimStatus } from "./vesting";

    const DAY = 24 * 60 * 60;

    // A distributor with a 90-day linear unlock, the shape the JTO drop used.
    const START_TS = 1_760_000_000;
    const END_TS = START_TS + 90 * DAY;

    interface Recipient {
      name: string;
      address: PublicKey;
      unlocked: bigint;
      locked: bigint;
    }

    // Deterministic stand-in addresses so the run is reproducible.
    function addr(seed: string): PublicKey {
      const bytes = new Uint8Array(32);
      Buffer.from(seed).copy(bytes);
      return new PublicKey(bytes);
    }

    const RECIPIENTS: Recipient[] = [
      { name: "early-plot-holder", address: addr("overgrowth-plot-01"), unlocked: 400_000_000n, locked: 0n },
      { name: "seed-round-farmer", address: addr("overgrowth-farm-02"), unlocked: 100_000_000n, locked: 900_000_000n },
      { name: "almanac-author", address: addr("overgrowth-alma-03"), unlocked: 250_000_000n, locked: 0n },
      { name: "compost-donor", address: addr("overgrowth-comp-04"), unlocked: 50_000_000n, locked: 150_000_000n },
    ];

    const allocations: Allocation[] = RECIPIENTS.map((r) => ({
      claimant: r.address.toBytes(),
      amountUnlocked: r.unlocked,
      amountLocked: r.locked,
    }));

    const tree = buildTree(allocations);
    console.log(`root ${hex(tree.root)}`);
    console.log(`leaves ${tree.leaves.length}`);

    interface Distributor {
      root: Uint8Array;
      maxNumNodes: number;
      numNodesClaimed: number;
      clawedBack: boolean;
      vault: bigint;
    }

    const distributor: Distributor = {
      root: tree.root,
      maxNumNodes: RECIPIENTS.length,
      numNodesClaimed: 0,
      clawedBack: false,
      vault: RECIPIENTS.reduce((sum, r) => sum + r.unlocked + r.locked, 0n),
    };

    const claimStatuses = new Map<string, ClaimStatus>();

    function newClaim(index: number, proof: Uint8Array[]): ClaimStatus {
      const r = RECIPIENTS[index];
      const key = r.address.toBase58();
      if (distributor.clawedBack) throw new Error("ClaimExpired");
      if (claimStatuses.has(key)) throw new Error("already claimed: ClaimStatus PDA exists");

      // Verify BEFORE counting: a failed InvalidProof attempt must leave the
      // distributor untouched, or a stream of bad proofs (challenge 2 sends one
      // on purpose) inflates numNodesClaimed until legitimate claimants hit
      // MaxNodesExceeded for no visible reason. On chain the same property
      // falls out of transaction atomicity; a sim has to order it by hand.
      const leaf = leafHash({
        claimant: r.address.toBytes(),
        amountUnlocked: r.unlocked,
        amountLocked: r.locked,
      });
      if (!verifyProof(proof, distributor.root, leaf)) throw new Error("InvalidProof");

      if (distributor.numNodesClaimed + 1 > distributor.maxNumNodes) throw new Error("MaxNodesExceeded");
      distributor.numNodesClaimed += 1;

      const status: ClaimStatus = {
        claimant: key,
        unlockedAmount: r.unlocked,
        lockedAmount: r.locked,
        lockedAmountWithdrawn: 0n,
      };
      claimStatuses.set(key, status);
      distributor.vault -= r.unlocked;
      return status;
    }

    // TODO(you): claim_locked. Ask vesting.ts what is withdrawable at currTs,
    // reject a zero payout the way the program does (InsufficientUnlockedTokens),
    // add it to lockedAmountWithdrawn, refuse to exceed lockedAmount
    // (ExceededMaxClaim), and take it out of the vault. Return the amount.
    function claimLocked(status: ClaimStatus, currTs: number): bigint {
      throw new Error("not implemented");
    }

    // Claim 1: an allocation with no locked half at all.
    const plotIndex = 0;
    const plotProof = tree.proofFor(plotIndex);
    const plotStatus = newClaim(plotIndex, plotProof);
    console.log(
      `unlocked claim: ${RECIPIENTS[plotIndex].name} took ${plotStatus.unlockedAmount} base units ` +
        `with a ${plotProof.length}-hash proof`,
    );

    // Claim 2: unlocked now, then the locked half as it vests.
    const farmIndex = 1;
    const farmStatus = newClaim(farmIndex, tree.proofFor(farmIndex));
    console.log(`unlocked claim: ${RECIPIENTS[farmIndex].name} took ${farmStatus.unlockedAmount} base units`);

    for (const [label, ts] of [
      ["day 0", START_TS],
      ["day 30", START_TS + 30 * DAY],
      ["day 45", START_TS + 45 * DAY],
      ["day 90", END_TS],
    ] as const) {
      try {
        const amount = claimLocked(farmStatus, ts);
        console.log(
          `claim_locked at ${label}: released ${amount}, withdrawn so far ` +
            `${farmStatus.lockedAmountWithdrawn}/${farmStatus.lockedAmount}`,
        );
      } catch (err) {
        console.log(`claim_locked at ${label}: rejected (${(err as Error).message})`);
      }
    }

    // A double claim is refused by the ClaimStatus PDA, not by good manners.
    try {
      newClaim(plotIndex, plotProof);
    } catch (err) {
      console.log(`replay of the unlocked claim: rejected (${(err as Error).message})`);
    }

    // A tree write invalidates every outstanding proof.
    const grown = buildTree([
      ...allocations,
      { claimant: addr("overgrowth-late-05").toBytes(), amountUnlocked: 10n, amountLocked: 0n },
    ]);
    const stale = verifyProof(plotProof, grown.root, tree.leaves[plotIndex]);
    console.log(`stale proof against the grown tree verifies: ${stale}`);

    console.log(`vault left: ${distributor.vault} base units`);
    ```

6. **Rode e leia cada linha.**

    ```bash
    npx tsx compost-airdrop/claim.ts
    ```

    Com o `claimLocked` implementado, a saída é esta, e cada linha é uma afirmação que você consegue defender:

    ```
    root fb855944c186313d7cc04782398567bd468dc5be3812c2719f8089e079f4c1a3
    leaves 4
    unlocked claim: early-plot-holder took 400000000 base units with a 2-hash proof
    unlocked claim: seed-round-farmer took 100000000 base units
    claim_locked at day 0: rejected (InsufficientUnlockedTokens)
    claim_locked at day 30: released 300000000, withdrawn so far 300000000/900000000
    claim_locked at day 45: released 150000000, withdrawn so far 450000000/900000000
    claim_locked at day 90: released 450000000, withdrawn so far 900000000/900000000
    replay of the unlocked claim: rejected (already claimed: ClaimStatus PDA exists)
    stale proof against the grown tree verifies: false
    vault left: 450000000 base units
    ```

    O dia 30 de 90 libera exatamente um terço de 900,000,000. O dia 45 libera o sexto seguinte, porque um terço já tinha sido tirado. O dia 90 libera o restante de uma vez. A raiz é determinística, então se a sua difere, a sua pré-imagem de folha difere, e o diagrama de bytes acima é onde olhar.

7. **Faça o typecheck da coisa toda.** O lab é escrito em modo strict, e a rigidez é o que pega um `number` onde um `bigint` deveria estar.

    ```bash
    npx tsc --noEmit --strict --target es2022 --module esnext \
      --moduleResolution bundler --skipLibCheck compost-airdrop/*.ts
    ```

    Silêncio quer dizer limpo. Se você trocar `bundler` por `nodenext` aqui vão te mandar escrever `./merkle.js` nos imports, o que é correto para ESM feito à mão e uma fricção sem sentido para um lab que roda pelo `tsx`.

8. **Opcional, e honesto quanto ao custo dela.** Para rodar o claim contra o programa de verdade em vez de um port dele, clone o jito-foundation/distributor, faça o build do programa Anchor, faça o deploy numa instância de surfpool ou na devnet, e conduza com a CLI dele. Isso é uma toolchain de Rust e uma tarde. O port que você acabou de escrever verifica contra a mesma raiz com a mesma pré-imagem, então nada do que você aprendeu muda; o que você compra com a tarde é a confiança de que os seus bytes de folha batem com os de um programa deployado, o que vale ter antes de um drop na mainnet e não antes de uma lição.

## Challenge

Três extensões, em ordem crescente de quanto elas vão te ensinar.

**Um.** Adicione um quinto destinatário cuja alocação é inteiramente bloqueada, com zero desbloqueado. Faça claim dela. O `new_claim` transfere um valor desbloqueado de zero, o que dá certo e mesmo assim abre o PDA de ClaimStatus, e só então o `claim_locked` tem algo a pagar. Confirme que o primeiro `claim_locked` depois do `start_ts` é o que de fato move tokens para esse destinatário.

**Dois.** Prove a falha de prova obsoleta de ponta a ponta em vez de como um booleano, e tem duas ciladas embutidas nas quais uma leitura literal entra direto. Primeiro, o seu `newClaim` verifica contra `distributor.root`, que ainda guarda a raiz ANTIGA, então uma prova antiga verifica numa boa contra ela; para encenar a falha você precisa fazer o distribuidor carregar a raiz da árvore crescida, ou construindo um segundo distribuidor a partir da árvore reconstruída, ou setando explicitamente `distributor.root = grown.root` e dizendo isso num comentário. Segundo, use um destinatário que ainda NÃO fez claim, porque a checagem de ClaimStatus-existe dispara antes da verificação da prova e mascararia a falha que você está tentando ver. Com os dois resolvidos: gere uma prova, reconstrua a árvore com mais uma alocação, aponte o distribuidor para a raiz nova, tente `newClaim` com a prova antiga, e capture `InvalidProof`. Escreva uma frase num comentário explicando por que rebuscar a prova imediatamente antes de enviar é a única correção confiável.

![Um fluxo de cinco passos das checagens do newClaim onde o teste de ClaimStatus-existente no passo dois rejeita claimants repetidos antes de a prova no passo três chegar a ser verificada.](assets/v09-flowchart.webp)

**Três.** Estenda a tabela de custo com uma coluna `total_cost_of_ownership`: para cada método, o custo do remetente mais o custo do claimant menos o que for reembolsável, a 100,000 destinatários. Depois responda, no arquivo, qual método você entregaria para o SPROUT e por quê, dado que o SPROUT carrega uma taxa de transferência. A resposta não é a linha mais barata e o seu comentário deveria dizer isso.

Aceite o seu trabalho quando a linha de 100k da tabela mostrar cerca de 186 SOL no clássico à taxa de rent atual da mainnet (ou 293 vezes a taxa que você definir, vezes 100,000) contra cerca de 1.03 SOL comprimido, os dois claims aterrissarem, e os caminhos de replay e de prova obsoleta forem rejeitados pelas razões pelas quais o programa os rejeitaria.

## Checkpoint

Agora você consegue fazer uma coisa que soa trivial e que quase ninguém faz antes de se comprometer com um drop: precificá-lo de quatro jeitos, a partir de constantes para as quais você consegue apontar, e dizer quem paga.

Concretamente, você deveria conseguir responder isto sem consultar nada. Quanto custa um destinatário clássico, e ele é reembolsável? Quanto custa um destinatário comprimido, e ele é? Por que o número do AirShip e o valor de estado por destinatário diferem em dez vezes? Qual das três extensões do SPROUT desqualifica o drop comprimido, e quais duas teriam ficado de boa? E quanto o `claim_locked` paga no dia 30 de um desbloqueio de 90 dias numa alocação de 900,000,000 unidades base?

Se a última estiver nebulosa, vale rerodar o passo 6 com alguns timestamps extras em vez de ler a fórmula de novo. Ver o contador de retirado perseguir o valor já liberado é o que torna a subtração óbvia.

Uma coisa que eu vou sinalizar porque ela me pegou enquanto eu escrevia este lab: eu construí a árvore, guardei as provas em cache numa variável, e depois reconstruí a árvore com um destinatário extra dois passos adiante, exatamente do jeito que um operador de verdade adiciona uma alocação atrasada. Toda prova em cache estava sem valor, sem alarde. O código pega isso agora porque essa falha é impressa como uma linha de saída, que é a única razão de eu confiar nele.

A economia agora tem um token, um venue, e um caminho de distribuição. O módulo 9 liga o dinheiro: para onde as taxas de fato fluem depois que você tem holders, como uma tesouraria faz buyback e queima, e como um programa de pontos vira um token sem um segundo lançamento. Traga as suas anotações de retenção de taxa do módulo 2, você vai precisar delas na primeira página.

Bom composting.
