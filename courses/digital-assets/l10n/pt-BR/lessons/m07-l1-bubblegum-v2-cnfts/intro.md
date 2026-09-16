# Bubblegum v2: cunhe um milhão de NFTs pelo preço de um laptop

## Resumo

A lição passada fechou o arco dos padrões de NFT. O Token Metadata é legado, o Metaplex Core é o padrão recomendado para trabalho novo de NFT, e o rule set todo-Pass que você decodificou no emblemático legado acabou não impondo nada. Duas lições atrás, na m06-l2, você construiu o artefato em que esta lição se apoia: uma coleção Core Overgrowth Almanac verificada, um ativo cujo plugin Royalties de fato passa por uma checagem de programa, uma impressão de edição numerada, e um badge Founding-Farmer congelado na carteira dele para sempre. Quatro ativos. Cunhados à mão, um de cada vez.

Agora a Overgrowth precisa fazer um drop de um crate Harvest para cada jogador. Digamos um milhão deles.

No rent de conta clássica esse airdrop custa uma casa, e você não entregou ele porque ninguém tem como pagar. Então, antes de qualquer teoria, precifique você mesmo. Crie um diretório de trabalho, instale os SDKs, e deixe o dimensionador do account-compression responder à pergunta:

```bash
mkdir -p overgrowth && cd overgrowth
npm install @metaplex-foundation/mpl-bubblegum@5.1.0 \
  @metaplex-foundation/mpl-account-compression@0.0.1 \
  @metaplex-foundation/mpl-core@1.10.0 \
  @metaplex-foundation/umi@1.5.1 \
  @metaplex-foundation/umi-bundle-defaults@1.5.1 \
  @metaplex-foundation/digital-asset-standard-api@2.0.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Pins lidos do npm em 2026-08-22, a semana em que esta lição foi escrita. O `mpl-bubblegum` 5.1.0 foi publicado em 2026-08-17, então está fresco; o lado on-chain é uma linha Rust separada e anda no cronograma dele. Confira os dois de novo antes de fixar qualquer coisa de vida longa.

```typescript
// overgrowth/scratch-cost.ts
import { getMerkleTreeSize } from "@metaplex-foundation/mpl-account-compression";

const RPC = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";

// Rent is (128 bytes of account header + your data) x a per-byte rate, and
// that rate is a NETWORK PARAMETER, not a constant a course gets to bake in.
// SIMD-0437 is stepping it down and the clusters are on different steps, so
// ask the one you will actually pay on: a 0-byte account's rent-exempt
// minimum is exactly the header times the rate.
async function lamportsPerByte(): Promise<number> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMinimumBalanceForRentExemption", params: [0] }),
  });
  const { result } = (await res.json()) as { result: number };
  return result / 128;
}

async function main(): Promise<void> {
  const rate = await lamportsPerByte();
  const bytes = getMerkleTreeSize(20, 256, 14);
  const sol = ((128 + bytes) * rate) / 1_000_000_000;

  console.log(`${rate.toLocaleString()} lamports per byte on ${RPC}`);
  console.log(`${bytes.toLocaleString()} bytes of account`);
  console.log(`${sol.toFixed(3)} SOL of rent for ${(2 ** 20).toLocaleString()} leaves`);
  console.log(`${(sol / 2 ** 20).toFixed(8)} SOL per cNFT`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

`npx tsx scratch-cost.ts` imprime quatro linhas. As minhas, contra a devnet em 2026-09-06:

```
5,080 lamports per byte on https://api.devnet.solana.com
1,223,352 bytes of account
6.215 SOL of rent for 1,048,576 leaves
0.00000593 SOL per cNFT
```

Uma conta. Seis SOL e pouco. Um milhão de NFTs. Aquela terceira linha dá mais ou menos 0.000006 SOL cada. Precifique na âncora de $150/SOL que esta lição usa o tempo todo e a árvore inteira sai por menos de mil dólares: o preço de um laptop decente, para um airdrop que contas por ativo precificam nas centenas de milhares. (Se você quiser a âncora pequena em vez disso: um café de SOL cobre alguns milhares de crates em folhas.)

A sua primeira linha pode não dizer 5,080, e se não disser, nada aqui está quebrado. A Metaplex publica 8.5012 SOL para uma árvore de um milhão de folhas, e essa era a resposta certa a 6,960 lamports por byte, a taxa que todo cluster cobrava antes de o SIMD-0437 começar a baixá-la em degraus. A linha deles não é bem esta árvore, e o passo 3 deixa a diferença explícita: eles publicam profundidade 20 com buffer 1024 e canopy 13, esta lição constrói buffer 256 e canopy 14, e as duas caem a 0.17% de distância por coincidência, não por acordo. A mainnet passou para 6,333 em 2026-09-03; a devnet está um degrau adiante, em 5,080; um fork do surfpool ainda servia 6,960 quando eu conferi em 2026-09-06, porque um fork herda as contas da mainnet e não necessariamente a tabela de rent dela. Mesmos bytes, três custos diferentes. Este é o primeiro de muitos números neste curso que pertence ao mundo e não à página, e a razão de o script perguntar em vez de afirmar.

Hoje você constrói essa árvore de verdade. O recuo da ajuda, em voz alta: a teoria e a derivação do custo da árvore são trabalhadas por inteiro, a spec da árvore é sua para escolher e defender (o passo 5 faz você derivar a profundidade e justificar o buffer e o canopy antes de cunhar o seu primeiro crate), e o crate de conquista soulbound mais a prova de rejeição de transferência são inteiramente seus, solo, com só as duas assinaturas de função fixadas para você.

## Sem conta, só uma folha

### O número que força o design

Aqui está a dor em dólares, precificada do chão para cima. A conta por ativo mais barata na Solana é uma conta de token SPL simples de 165 bytes; pergunte a um cluster quanto custa uma (`solana rent 165`) e a mainnet respondeu 0.00185 SOL em 2026-09-06. Esse é o comparador mais generoso possível: qualquer formato real de NFT custa mais. Cunhe um milhão de qualquer coisa que precise até dessa conta mínima e você está segurando como refém da ordem de 1,900 SOL de rent; a $150 por SOL isso dá quase $300,000, travados, para entregar um crate. E os ativos Core que você entregou no módulo passado não te salvam: aos ~0.003 SOL por ativo publicados pelo fornecedor, um milhão de crates Core sai por uns 3,000 SOL. Barato por ativo, mais ou menos quarenta centavos de dólar cada nesse preço de SOL, ainda é uma casa na escala de uma frota. Qualquer coisa por ativo é o problema.

Esse número não é uma hipótese que alguém inventou para um curso. A Solana já tinha passado de 500 milhões de contas e estava adicionando mais ou menos um milhão por dia em novembro de 2024, que é exatamente a pressão que produziu a compressão de estado em primeiro lugar (a Helius escreveu sobre isso perto do keynote de compressão ZK deles, 2024-11-25; trate as contagens como o retrato que eles tiraram, não como uma leitura ao vivo). Cada uma dessas contas é RAM de um validador. A chain estava crescendo um problema de armazenamento mais rápido do que estava crescendo usuários.

Então, a reformulação. Você não precisa mesmo de um milhão de contas. Você precisa conseguir *provar*, para qualquer crate, que ele existe e quem é o dono dele. Esses são requisitos diferentes, e só o segundo é estrutural.

![Uma comparação em duas colunas entre um milhão de contas de token clássicas custando mais ou menos dois mil SOL de rent e uma árvore Bubblegum v2 da mesma capacidade custando SOL de um dígito, com leituras só por DAS e escritas que carregam prova.](assets/v01-comparison.webp)

### O que uma folha é de fato

A compressão de estado guarda um hash do seu ativo, não o seu ativo.

Concretamente: o Bubblegum pega os metadados do crate (nome, URI, taxa do vendedor, criadores, coleção, dono, e um nonce), faz o hash disso num único valor de 32 bytes, e escreve esse valor num slot de uma **árvore de Merkle concorrente** on-chain. Esse slot é a **folha**. Cada par de folhas é hasheado junto num pai, cada par de pais num avô, subindo até uma única **raiz** de 32 bytes guardada na conta da árvore. Nada mais sobre o seu crate está on-chain. Não existe conta de mint, nem conta de token, nem PDA de metadados, nem conta de ativo Core. Existe um hash, sentado num slot, numa conta grande que você pagou uma vez.

Se você já usou git, você já tem a intuição. Um hash de commit não contém o seu repositório. Ele se compromete com ele, com tanta precisão que mudar um byte em qualquer lugar muda o hash, e tão barato que você consegue nomear um milhão de arquivos com 32 bytes. Uma raiz de Merkle é esse mesmo truque com uma prova anexada: dada uma folha e o hash irmão em cada nível subindo a árvore, qualquer um consegue recalcular a raiz e conferir contra a que está on-chain. Vinte níveis para um milhão de folhas, porque é isso que o `log2` faz por você.

Essa prova é a barganha inteira. Você parou de pagar por armazenamento e começou a pagar por provas.

![Um caminho destacado sobe uma árvore de Merkle de vinte níveis, de uma folha até a raiz on-chain, com os vinte hashes irmãos sombreados formando uma prova de 640 bytes.](assets/v02-diagram.webp)

Vale ser preciso sobre o que essa troca custa de fato, porque a assimetria é a razão inteira de a compressão ser viável e não só esperta. Guardar um ativo como conta é O(1) para ler e O(n) em rent ao longo de n ativos, e o rent é o recurso caro porque é memória de validador segurada para sempre. Guardar um ativo como folha é O(1) em rent ao longo de n ativos, porque a conta da árvore tem tamanho fixo independente de quão cheia ela está, e O(log n) por escrita, porque uma prova é um hash irmão por nível. Dobrar o seu supply de um milhão para dois milhões não dobra o rent. Adiciona um nível, que adiciona um hash irmão a toda prova e trinta e dois bytes a toda escrita. Essa é a troca que o design faz: ele converte um custo linear de armazenamento num custo logarítmico de banda. Banda você consegue agrupar em lote, cachear e encurtar com um canopy. Rent você só consegue pagar.

O corolário é a coisa que as pessoas deixam passar. Compressão não é "NFTs, mais barato." É uma curva de custo diferente, e curvas se cruzam. Abaixo de alguns milhares de ativos, uma conta Core por ativo é genuinamente competitiva e muito mais simples de operar. Acima de cem mil não existe discussão a ter. As decisões interessantes moram todas no meio, e elas giram em torno da frequência de escrita, não da contagem.

Um nome no próximo visual precisa ser apresentado antes de você encontrar ele lá: o **programa Noop**. É um programa publicado que de propósito não faz nada quando é invocado; o valor inteiro dele é que os dados passados para ele vão parar nos logs da transação. Na hora do mint o Bubblegum faz CPI para o programa Noop com os metadados legíveis completos do crate, então os logs viram o único lugar onde esses dados são escritos, e indexadores reconstroem tudo o que servem para você reproduzindo esses logs. A chain em si guarda só o hash.

O **asset id** cai do mesmo design. Um cNFT não tem conta, então precisa de algum endereço canônico pelo qual ser referido, e o Bubblegum deriva ele: o asset id é `PDA(tree, leaf index)`. Determinístico, derivável offline, estável para sempre. Você vai derivar um no lab com `findLeafAssetIdPda` e depois ver um provedor de DAS te devolver a mesma string.

![Uma conta de árvore guarda a raiz, o canopy, o changelog buffer e os slots de folha ao lado de um índice DAS separado de metadados legíveis, com a conta por ativo inexistente riscada.](assets/v03-diagram.webp)

### O changelog buffer, e por que as provas ficam desatualizadas

Agora a parte que morde as pessoas em produção.

Uma prova de Merkle é um retrato. Ela diz: *dada esta raiz, a minha folha faz hash até ela.* No momento em que qualquer outra pessoa cunha, transfere ou queima qualquer coisa na mesma árvore, a raiz muda, e toda prova que alguém estava segurando agora é uma prova contra uma raiz que não existe mais. Numa árvore que faz uma escrita por dia, ninguém percebe. Numa árvore fazendo um drop de crates, todo mundo percebe de uma vez.

É para isso que serve o `max_buffer_size`. A conta da árvore mantém um **changelog buffer** das últimas N mudanças de raiz, então uma prova que era válida algumas escritas atrás ainda consegue ser reproduzida para a frente e aceita. Buffer 64 quer dizer que mais ou menos 64 escritas concorrentes conseguem cair num slot antes de as provas começarem a ricochetear. Buffer 256 te compra mais folga e te custa bytes. É por isso que a estrutura se chama árvore de Merkle *concorrente* e não só árvore de Merkle: sem o buffer, uma árvore serializaria para uma escrita por slot, e no alvo de slot de ~400ms da Solana o seu drop de um milhão de crates levaria uns quatro dias e meio.

A cilada dita sem rodeios: **busque a prova de novo imediatamente antes de toda escrita.** Não no topo do seu script. Não uma vez por lote. Imediatamente antes. O helper que você vai usar no challenge, `getAssetWithProof`, faz uma ida e volta nova ao DAS a cada chamada exatamente por essa razão, e se você cachear o resultado dele ao longo de um lote de transferências você vai receber uma enxurrada de erros de hashing-mismatch que parecem um bug no seu código e não são.

![Um fluxograma de três faixas contrastando uma prova que verifica direto, uma reproduzida para a frente a partir do changelog buffer depois de escritas concorrentes, e uma rejeitada porque a raiz envelheceu e saiu do buffer.](assets/v04-flowchart.webp)

### O canopy é o dinheiro

Vinte níveis de profundidade quer dizer que uma escrita do cliente precisa fornecer vinte hashes irmãos. Vinte chaves públicas, a 32 bytes cada, pegando carona na transação. Isso é 640 bytes de prova antes de você ter escrito uma única instrução, e o orçamento de transação da Solana não é generoso o bastante para você ignorar isso. Empurre os nós de prova para além do limite e a sua escrita não fica mais lenta, ela para de caber.

O **canopy** é a solução: guardar em cache os K níveis de cima de nós internos dentro da própria conta da árvore. Se a chain já conhece os 14 níveis de cima, o cliente só envia os `max_depth - canopy_depth` nós de baixo. Seis, nesse caso, em vez de vinte.

O que quer dizer que o canopy não é um botão de ajuste, é uma compra. Você compra provas de cliente mais curtas com rent. Ninguém neste curso mediu a fórmula exata, então em vez de te entregar uma tabela para confiar, derive ela. Esta é a parte em que o número de manchete do fornecedor para de ser folclore e começa a ser aritmética:

```typescript
// overgrowth/tree-size.ts
import { getMerkleTreeSize } from "@metaplex-foundation/mpl-account-compression";

export type TreeSpec = {
  maxDepth: number;
  maxBufferSize: number;
  canopyDepth: number;
};

/**
 * The (maxDepth, maxBufferSize) pairs the on-chain account layout is generated for.
 * Legality is per PAIR, not per field: depth 14 accepts buffer 64 but never 512.
 */
const DEPTH_BUFFER_PAIRS: ReadonlyArray<readonly [number, number]> = [
  [3, 8], [5, 8],
  [6, 16], [7, 16], [8, 16], [9, 16],
  [10, 32], [11, 32], [12, 32], [13, 32],
  [14, 64], [14, 256], [14, 1024], [14, 2048],
  [15, 64], [16, 64], [17, 64], [18, 64], [19, 64],
  [20, 64], [20, 256], [20, 1024], [20, 2048],
  [24, 64], [24, 256], [24, 512], [24, 1024], [24, 2048],
  [26, 512], [26, 1024], [26, 2048],
  [30, 512], [30, 1024], [30, 2048],
];

const DEPTHS = [...new Set(DEPTH_BUFFER_PAIRS.map(([d]) => d))].sort((a, b) => a - b);

export function depthForSupply(targetSupply: number): number {
  const depth = DEPTHS.find((d) => 2 ** d >= targetSupply);
  if (depth === undefined) throw new Error(`no supported depth holds ${targetSupply} leaves`);
  return depth;
}

export function treeBytes(spec: TreeSpec): number {
  const legal = DEPTH_BUFFER_PAIRS.some(
    ([d, b]) => d === spec.maxDepth && b === spec.maxBufferSize,
  );
  if (!legal) {
    throw new Error(
      `depth ${spec.maxDepth} / buffer ${spec.maxBufferSize} is not a supported pair`,
    );
  }
  return getMerkleTreeSize(spec.maxDepth, spec.maxBufferSize, spec.canopyDepth);
}

/** The account header the runtime charges rent on, on top of your data. */
export const ACCOUNT_HEADER_BYTES = 128;

/**
 * Read the per-byte rent rate off a live cluster instead of trusting a
 * literal. A 0-byte account's rent-exempt minimum is exactly the header times
 * the rate, so one RPC call gives you the whole schedule. SIMD-0437 is
 * stepping this number down in stages and the clusters are not in step with
 * each other, which is why no function here takes a default.
 */
export async function readLamportsPerByte(rpcUrl: string): Promise<number> {
  const res = await fetch(rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getMinimumBalanceForRentExemption", params: [0] }),
  });
  const { result } = (await res.json()) as { result: number };
  return result / ACCOUNT_HEADER_BYTES;
}

/** Rent-exempt minimum in SOL: (header + data) x the rate you just read. */
export function rentSol(bytes: number, lamportsPerByte: number): number {
  return ((ACCOUNT_HEADER_BYTES + bytes) * lamportsPerByte) / 1_000_000_000;
}

export function treeCostSol(spec: TreeSpec, lamportsPerByte: number): number {
  return rentSol(treeBytes(spec), lamportsPerByte);
}

/** Proof nodes a client must ship per write, once the canopy covers its top levels. */
export function proofNodesOnTheWire(spec: TreeSpec): number {
  return Math.max(spec.maxDepth - spec.canopyDepth, 0);
}
```

Varra o canopy contra uma profundidade e um buffer fixos e o trade-off se imprime sozinho:

```typescript
// overgrowth/tree-cost.ts
import {
  depthForSupply,
  proofNodesOnTheWire,
  readLamportsPerByte,
  treeBytes,
  treeCostSol,
} from "./tree-size";

const RPC = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
const targetSupply = Number(process.argv[2] ?? 1_000_000);
const maxBufferSize = Number(process.argv[3] ?? 256);
const maxDepth = depthForSupply(targetSupply);

// Metaplex's published million-leaf row, copied exactly: depth 20 with buffer
// 1024 and canopy 13, at 8.5012 SOL. Note the spec, because it is NOT the one
// this sweep runs. The rate is the pre-SIMD-0437 6,960 their page was written
// under, kept only so the comparison below can rescale their number to yours.
const VENDOR_SPEC = { maxDepth: 20, maxBufferSize: 1024, canopyDepth: 13 };
const RATE_BEHIND_THE_VENDOR_FIGURE = 6960;
const VENDOR_SOL_AT_THAT_RATE = 8.5012;

async function main(): Promise<void> {
  const rate = await readLamportsPerByte(RPC);
  console.log(`rent rate: ${rate.toLocaleString()} lamports/byte on ${RPC}`);
  console.log(`target supply ${targetSupply.toLocaleString()} -> maxDepth ${maxDepth} (${(2 ** maxDepth).toLocaleString()} leaves)`);
  console.log("canopy   bytes        SOL      proof nodes on the wire");
  for (const canopyDepth of [0, 6, 8, 10, 12, 13, 14]) {
    const spec = { maxDepth, maxBufferSize, canopyDepth };
    const line = [
      String(canopyDepth).padStart(4),
      String(treeBytes(spec)).padStart(10),
      treeCostSol(spec, rate).toFixed(3).padStart(9),
      String(proofNodesOnTheWire(spec)).padStart(10),
    ].join("  ");
    console.log(line);
  }

  // Check the derivation against the vendor by deriving THEIR spec, not ours.
  // Their number is quoted for exactly one configuration AND at one rent rate,
  // so reproduce the configuration and rescale the rate before comparing;
  // holding a 6,960-era number against a 5,080-era derivation would be a unit
  // error, not a finding. Doing it this way makes the check real: it agrees to
  // four decimal places, which is a verification. Pointing the same check at
  // OUR spec would also have printed MATCH, and that would have been luck.
  if (maxDepth === 20) {
    const vendorAtYourRate = VENDOR_SOL_AT_THAT_RATE * (rate / RATE_BEHIND_THE_VENDOR_FIGURE);
    const theirs = treeCostSol(VENDOR_SPEC, rate);
    const ours = treeCostSol({ maxDepth: 20, maxBufferSize, canopyDepth: 14 }, rate);
    console.log(`\nvendor row : ${VENDOR_SOL_AT_THAT_RATE} SOL, depth 20 / buffer 1024 / canopy 13, quoted at ${RATE_BEHIND_THE_VENDOR_FIGURE} lamports/byte`);
    console.log(`  rescaled to your rate: ${vendorAtYourRate.toFixed(4)} SOL`);
    console.log(`derived, their spec  : ${theirs.toFixed(4)} SOL`);
    console.log(
      Math.abs(theirs - vendorAtYourRate) < 0.005 * vendorAtYourRate
        ? "MATCH - the derivation reproduces the published row"
        : "MISMATCH - re-check the spec",
    );
    console.log(`derived, this sweep  : ${ours.toFixed(4)} SOL at depth 20 / buffer ${maxBufferSize} / canopy 14`);
    console.log(`  a DIFFERENT tree that lands ${(((ours - theirs) / theirs) * 100).toFixed(2)}% away - near, not the same`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

`npx tsx tree-cost.ts` te dá isto. A minha rodada, devnet, 2026-09-06 — a sua coluna de SOL se move com a taxa do seu cluster e a coluna de bytes não:

```
rent rate: 5,080 lamports/byte on https://api.devnet.solana.com
target supply 1,000,000 -> maxDepth 20 (1,048,576 leaves)
canopy   bytes        SOL      proof nodes on the wire
   0      174840      0.889          20
   6      178872      0.909          14
   8      191160      0.972          12
  10      240312      1.221          10
  12      436920      2.220           8
  13      699064      3.552           7
  14     1223352      6.215           6

vendor row : 8.5012 SOL, depth 20 / buffer 1024 / canopy 13, quoted at 6960 lamports/byte
  rescaled to your rate: 6.2049 SOL
derived, their spec  : 6.2049 SOL
MATCH - the derivation reproduces the published row
derived, this sweep  : 6.2153 SOL at depth 20 / buffer 256 / canopy 14
  a DIFFERENT tree that lands 0.17% away - near, not the same
```

Leia essa tabela duas vezes, porque é a coisa mais útil desta lição, e leia primeiro descendo pela coluna de BYTES, porque essa coluna é física e a coluna de SOL é política. Uma árvore de um milhão de folhas sem canopy nenhum tem 174,840 bytes; no canopy 14 ela tem 1,223,352. Sete vezes a conta, e portanto sete vezes o rent, a qualquer taxa que a rede um dia cobre. Seis sétimos daquela figura famosa, seja lá em que ela esteja denominada este mês, são canopy. O que você compra com isso é uma queda de vinte nós de prova para seis, que é a diferença entre "a minha instrução de transferência cabe" e "a minha instrução de transferência não cabe." E note o que a checagem acima acabou de te mostrar: a linha do próprio fornecedor compra quase exatamente a mesma conta por um caminho diferente, canopy 13 com um buffer quatro vezes mais fundo, que é o mesmo dinheiro gasto em concorrência de escrita em vez de em comprimento de prova. A figura de manchete deles codifica silenciosamente as duas escolhas, você agora sabe quais são, e pode escolher diferente de propósito.

O meu próprio viés, pelo que vale: eu já vi mais projetos se queimarem por um canopy subdimensionado do que por um superdimensionado, porque rent é um número que você vê no dia zero e um tamanho de transação estourado é um número que você vê no dia do drop. Se você estiver em dúvida, compre o canopy.

![Um gráfico de eixo duplo em que uma árvore de um milhão de folhas cresce sete vezes em bytes de conta conforme o canopy se aprofunda de 0 a 14, enquanto os nós de prova por escrita caem de 20 para 6.](assets/v05-chart.webp)

### Dimensionar não tem volta

`max_depth` é permanente. Uma árvore de profundidade 14 segura 16,384 folhas e vai segurar 16,384 folhas por todo o tempo em que existir. Não tem realloc, não tem migração, não tem "a gente aumenta ela depois." Quando a última folha é preenchida, essa árvore está encerrada, e a sua única jogada é criar outra e ensinar todo sistema a jusante que a sua coleção agora se espalha por duas árvores.

Isso não é fatal, e muitos drops de produção rodam multi-árvore de propósito. Mas é uma decisão que você quer tomar deliberadamente no dia zero em vez de descobrir no dia do drop, então rode os números contra a sua ambição real em vez do seu roadmap real.

| max_depth | folhas | bytes, sem canopy | bytes, canopy 8 | nós de prova no canopy 8 |
|---|---|---|---|---|
| 14 | 16,384 | 31,800 | 48,120 | 6 |
| 17 | 131,072 | 38,040 | 54,360 | 9 |
| 20 | 1,048,576 | 44,280 | 60,600 | 12 |
| 24 | 16,777,216 | 52,600 | 68,920 | 16 |

Essas são linhas de `maxBufferSize: 64`, produzidas com os helpers do `tree-size.ts` que você acabou de escrever, num loop de cinco linhas sobre profundidades com canopy fixo em 8 (o `tree-cost.ts` varre o canopy com profundidade fixa, então esta varredura é a irmã de um minuto dele). Elas estão em bytes e não em SOL de propósito: bytes são uma propriedade do layout da conta e vão ler igual para você e para mim, enquanto o SOL é a taxa do seu cluster vezes aquela coluna e se move debaixo de nós dois. Multiplique a coluna você mesmo e a parte surpreendente aparece. Profundidade quase não custa nada. Ir de dezesseis mil folhas para dezesseis *milhões* adiciona uns 20,800 bytes, mais ou menos 0.11 SOL na taxa da devnet no dia em que escrevi isto, porque a profundidade só afeta a largura de linha do changelog buffer, não o armazenamento das folhas. Nada guarda as suas folhas. Não tem nada para guardar.

Então a orientação quase se escreve sozinha: seja generoso com profundidade, seja deliberado com canopy, e seja honesto sobre buffer. Profundidade é quase de graça e permanente, então exagere nela. Canopy é caro e permanente, então precifique contra o comprimento de prova que as suas escritas conseguem de fato carregar. Buffer fica entre os dois e é igualmente permanente: mais barato que canopy, mas dinheiro de verdade na escala (o salto na profundidade 20 de buffer 64 para buffer 256 adiciona 130,560 bytes, uns 0.66 SOL na taxa da devnet), então case ele com a sua pior concorrência de escrita esperada em vez de deixá-lo alto por padrão.

Nem todo par de profundidade e buffer é legal, aliás, e essa é a razão de o `tree-size.ts` carregar aquela tabela de pares em vez de duas listas independentes de valores permitidos. O layout de conta on-chain é gerado para um conjunto fixo de combinações: profundidade 14 aceita buffer 64, 256, 1024 ou 2048 e nada mais, profundidade 26 começa em 512, e buffer 128 não é um tamanho legal em profundidade nenhuma. Checar os dois campos separadamente deixaria passar meia dúzia de pares que o programa vai recusar. Um par ruim também não falha com elegância em runtime, ele falha como um erro de tamanho de conta pouco útil depois de você já ter pago o rent, então deixe a trava lançar o erro antes de você gastar.

![Uma tabela de quatro linhas em bytes de conta em que subir a profundidade da árvore de 16 mil para 16 milhões de folhas adiciona só 20,800 bytes, localizando o custo real de uma árvore comprimida no canopy.](assets/v06-table.webp)

### O que mudou na v2

O Bubblegum foi entregue em 2023, e muito do que as pessoas ainda repetem com confiança sobre NFTs comprimidos descreve o programa V1. A versão 2 quebrou a maior parte disso.

A reversão que mais importa: **cNFTs soulbound existem agora.** Por dois anos a sabedoria recebida era que a compressão te comprava mints baratos e te custava toda primitiva de imposição, que um cNFT não podia ser congelado e nunca poderia ser tornado não transferível, e que se você queria soulbound você buscava um mint NonTransferable de Token-2022 em vez disso. O Bubblegum v2 já vem com `set_non_transferable_v2`, além de congelamento e descongelamento, e quando a coleção Core da árvore carrega um `PermanentFreezeDelegate` a coisa toda impõe no nível do programa (documentação do Bubblegum v2 da Metaplex; o conjunto de instruções está bem ali na própria interface do programa, que você vai chamar no challenge). O folclore está simplesmente desatualizado.

Vale ver o mecanismo, porque ele é um byte:

```typescript
// what the client library exposes for the v2 leaf's flags field
import { LeafSchemaV2Flags } from "@metaplex-foundation/mpl-bubblegum";

// LeafSchemaV2Flags.None                 === 0
// LeafSchemaV2Flags.FrozenByOwner        === 1
// LeafSchemaV2Flags.FrozenByPermDelegate === 2
// LeafSchemaV2Flags.NonTransferable      === 4

const soulboundAndFrozen =
  LeafSchemaV2Flags.NonTransferable | LeafSchemaV2Flags.FrozenByPermDelegate; // 6
```

Soulbound é um bit. O bit 2, num byte `flags` que as folhas V1 não têm. O que explica o próximo fato, o que vai te custar uma tarde se você deixar passar: **folhas V2 não são retrocompatíveis com folhas V1.** Esquema de folha diferente, hash diferente, superfície de programa diferente. Uma árvore V1 não tem como ser conduzida para dentro de um fluxo v2 e não existe upgrade no lugar. Se você herdar uma árvore V1, você lê ela com instruções V1 e cunha supply novo numa árvore v2 nova.

O resto do delta da v2, rapidinho:

| | Bubblegum V1 | Bubblegum v2 |
|---|---|---|
| Padrão de coleção | coleções do Token Metadata | coleções do MPL Core |
| Esquema de folha | `LeafSchema` V1, sem byte de flags | `LeafSchemaV2`, com um bitfield de flags |
| Congelar / descongelar | não disponível | `freeze_v2`, `thaw_v2`, variantes de delegado |
| Soulbound | não disponível | `set_non_transferable_v2` |
| Programa de compressão | account-compression da SPL | `mpl-account-compression` forkado |
| Verificação de coleção | instrução verify, passo separado | coleção é uma pubkey simples, sempre verificada |

Aquela linha do programa de compressão não é cosmética. O Bubblegum v2 roda no fork próprio de account compression da Metaplex, `mpl-account-compression`, cuja linha de crate 0.4.2 é de onde o programa forkado veio, no program id `mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW`. Esse é um endereço diferente do programa account-compression da SPL, e o próprio programa Bubblegum fica em `BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY`. Linhas de crate de programa andam mais rápido do que rascunhos de lição, então trate o número do crate como a procedência do fork e não como o estado do npm hoje; os pins de cliente no bloco de instalação acima são os que o seu lab de fato roda.

Uma coisa que a v2 manteve da V1, e que merece uma frase porque é a peça que as pessoas esquecem até precisarem dela: a árvore tem uma autoridade própria. O `create_tree_v2` escreve um PDA de config da árvore ao lado da conta de Merkle registrando quem criou a árvore, e os mints passam por um signer `treeCreatorOrDelegate`. Você pode delegar esse slot para um serviço de cunhagem sem entregar o seu keypair, e pode criar a árvore `public`, caso em que qualquer um pode anexar uma folha nela. Árvores públicas são como experiências de mint aberto funcionam e também como um estranho preenche o seu teto de supply para você, então o default de `public: false` é o default certo e você deve ter uma razão antes de virar ele. Tem duas autoridades separadas em jogo nesta lição, o que faz as pessoas tropeçarem: a autoridade da *árvore* assina mints para dentro da árvore, e a autoridade da *coleção* assina a filiação ao Almanac. A sua carteira de lab por acaso é as duas. Em produção elas normalmente não são a mesma chave, e a instrução de mint quer as duas assinaturas.

A linha da coleção é a que alcança de volta o trabalho da lição passada. O `MetadataArgsV2` carrega `collection` como um `Option<PublicKey>` simples, com o comentário da própria biblioteca cliente afirmando que na V2 ele "é só uma `Pubkey` e é sempre considerado verificado." Sem passo de verify, sem limbo de não verificado. A conta de coleção do Almanac que você criou na m06-l2 é o valor literal que você passa, e o mint falha se a autoridade da coleção não assinar. Coleção primeiro, depois os membros, um nível abaixo. Mesma regra que você aprendeu nos ativos Core.

![Uma linha do tempo de quatro paradas, do Bubblegum V1 em 2023 ao Bubblegum v2 em 2026, em que a alegação de que cNFTs não podem ser soulbound é finalmente riscada.](assets/v07-timeline.webp)

### O trade-off, nomeado

A compressão troca mints baratos por complexidade de leitura e acoplamento de escrita. As duas metades são propriedades permanentes do design, não arestas que alguém vai lixar.

**Leituras precisam de um RPC com suporte a DAS.** Não tem conta para buscar, então `getAccountInfo` num asset id de cNFT não retorna absolutamente nada. O seu crate é real, ele é comprovadamente seu, e um endpoint RPC público default não consegue te dizer uma única coisa sobre ele. Você agora depende do índice de um provedor, com qualquer atualidade e completude que esse provedor ofereça, para o ato básico de ler o seu próprio ativo. Essa é uma dependência real e você deveria senti-la no lab quando a sua primeira chamada `getAsset` voltar vazia por quatro segundos porque o indexador ainda não alcançou.

**Escritas precisam de uma prova nova.** Toda transferência, queima, congelamento e atualização de metadados carrega nós de prova, o que quer dizer que toda escrita está acoplada ao estado atual da árvore e disputa com toda outra escrita contra a mesma árvore.

**E o rent não é o custo todo.** O que quer que a árvore tenha te custado, comprou a árvore. Não compra o milhão de transações que a preenchem. Cada `mint_v2` é uma transação com a própria taxa base e, em qualquer dia que valha a pena fazer um drop, a própria taxa de prioridade, e nenhuma quantidade de dimensionamento esperto de árvore faz isso sumir. Agrupar vários mints numa transação ajuda muito e é a jogada padrão para um drop de verdade, mas o teto de quantos cabem é dado pelo tamanho da transação, que é dado pelo comprimento da prova, que é dado pelo seu canopy. A decisão de canopy alcança mais longe do que a linha de rent sugere. Fazer um lote grande cair de forma confiável é uma disciplina de lado cliente por si só, e o curso planejado Client-Side Mastery cuida desse território: taxas de prioridade, retries, e como impedir que um lote tenha sucesso pela metade em silêncio.

**E as decisões de dimensionamento não têm volta.** Subdimensione o `max_depth` e o seu teto de supply é permanente. Subdimensione o canopy e toda transação de cliente carrega provas mais longas para sempre, o que pode empurrar uma escrita para além do limite de tamanho de transação exatamente no momento em que você tem o maior número de escritas.

Vale a pena? Para um milhão de crates que jogador nenhum vai negociar individualmente, obviamente. Para quatro volumes do Almanac com royalties e números de edição, obviamente não, que é por isso que a lição passada usou contas Core e esta não usa. A regra honesta é chata: comprima quando a contagem de ativos é grande e a frequência de escrita por ativo é baixa. Nenhuma das metades sozinha basta.

## Lab: plante a árvore de crates

Sete passos. Os passos 1 a 4 são trabalhados por inteiro, o passo 5 faz você escolher e defender a spec da árvore antes de qualquer coisa gastar, e a cancela é o próprio script de mint rodando limpo de ponta a ponta (o Checkpoint reafirma isso). A árvore que a gente constrói aqui deliberadamente não é de um milhão de folhas; é de 16,384, que é o tamanho que o fornecedor precifica a uns 0.34 SOL (na taxa pré-SIMD-0437 sob a qual a página deles foi escrita) e que a sua própria derivação vai confirmar, no dinheiro do seu próprio cluster, antes de você gastar qualquer coisa.

Uma coisa muda no seu setup, e muda por uma razão com a qual você deveria sentar. **Esta lição roda contra a devnet, através de um RPC com suporte a DAS.** O surfnet que você vem usando desde a m02-l1 é um validador de verdade, ele vai criar a sua árvore e cunhar os seus crates numa boa, e depois vai ser completamente incapaz de te dizer o que você cunhou, porque um validador local não roda um indexador. Esse é o trade-off de complexidade de leitura, e você encontra ele no primeiro dia. Pegue um endpoint de devnet de um provedor de DAS (Helius, QuickNode, Triton e Shyft todos servem a interface; a lista de provedores e como escolher é assunto da próxima lição), então:

```bash
export DAS_RPC_URL="https://devnet.helius-rpc.com/?api-key=YOUR_KEY"
```

1. **O setup compartilhado.** Um helper é dono da conexão, da carteira e do catálogo de endereços. Note os três plugins Umi: `mplCore()` para a coleção, `mplBubblegum()` para a árvore, e `dasApi()` para as leituras. Sem esse terceiro, `umi.rpc.getAsset` não existe. (Uma nota de honestidade do sistema de tipos: com esses pins exatos a augmentação do das-api não alcança `umi.rpc` sob uma passagem estrita de resolução de módulos do `tsc`, então o seu editor pode sublinhar `getAsset` de vermelho; as rodadas documentadas de `npx tsx` do lab não são afetadas, e o sublinhado é a lacuna da stack fixada, não do seu código.)

    ```typescript
    // overgrowth/umi.ts
    import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
    import { keypairIdentity, sol, type Umi } from "@metaplex-foundation/umi";
    import { mplCore } from "@metaplex-foundation/mpl-core";
    import { mplBubblegum } from "@metaplex-foundation/mpl-bubblegum";
    import { dasApi } from "@metaplex-foundation/digital-asset-standard-api";
    import fs from "node:fs";

    export async function getUmi(): Promise<Umi> {
      const endpoint = process.env.DAS_RPC_URL;
      if (!endpoint) throw new Error("set DAS_RPC_URL to a devnet DAS endpoint");

      const umi = createUmi(endpoint).use(mplCore()).use(mplBubblegum()).use(dasApi());

      let secret: Uint8Array;
      if (fs.existsSync("wallet.json")) {
        secret = Uint8Array.from(JSON.parse(fs.readFileSync("wallet.json", "utf8")));
      } else {
        const fresh = umi.eddsa.generateKeypair();
        fs.writeFileSync("wallet.json", JSON.stringify(Array.from(fresh.secretKey)));
        secret = fresh.secretKey;
      }
      umi.use(keypairIdentity(umi.eddsa.createKeypairFromSecretKey(secret)));

      const balance = await umi.rpc.getBalance(umi.identity.publicKey);
      if (balance.basisPoints < sol(1).basisPoints) {
        throw new Error(`fund ${umi.identity.publicKey} with devnet SOL, then re-run`);
      }
      return umi;
    }

    export function book(): Record<string, string> {
      return fs.existsSync("crates.json") ? JSON.parse(fs.readFileSync("crates.json", "utf8")) : {};
    }

    export function remember(key: string, value: string): void {
      fs.writeFileSync("crates.json", JSON.stringify({ ...book(), [key]: value }, null, 2));
    }
    ```

    Rode tudo de dentro de `overgrowth/` para que `wallet.json` e o catálogo de endereços compartilhado `crates.json` caiam juntos. Financie o endereço impresso com um faucet de devnet na primeira rodada; o script te diz o endereço e para, em vez de construir metade de uma árvore com um pagador sem fundos.

2. **Ponha o Almanac neste cluster.** O seu Almanac da m06-l2 mora no surfnet, e a devnet nunca ouviu falar dele. Então o helper ou encontra a coleção registrada em `crates.json` ou cria ela, e desta vez ela carrega um `PermanentFreezeDelegate` desde o nascimento, porque `set_non_transferable_v2` exige que a autoridade que assina seja um delegado de congelamento permanente na coleção. Arme isso agora ou o challenge fica impossível de vencer depois.

    ```typescript
    // overgrowth/collection.ts
    import { generateSigner, publicKey, type PublicKey, type Umi } from "@metaplex-foundation/umi";
    import { createCollection, fetchCollection } from "@metaplex-foundation/mpl-core";
    import { book, remember } from "./umi";

    export async function getOrCreateAlmanac(umi: Umi): Promise<PublicKey> {
      const saved = book().collection;
      if (saved) {
        const existing = await fetchCollection(umi, publicKey(saved));
        return existing.publicKey;
      }

      const collection = generateSigner(umi);
      await createCollection(umi, {
        collection,
        name: "Overgrowth Almanac",
        uri: "https://overgrowth.example/almanac/collection.json",
        plugins: [
          {
            type: "PermanentFreezeDelegate",
            frozen: false,
            authority: { type: "Address", address: umi.identity.publicKey },
          },
        ],
      }).sendAndConfirm(umi);

      remember("collection", collection.publicKey);
      return collection.publicKey;
    }
    ```

    `frozen: false` é deliberado. Um congelamento no nível da coleção com `frozen: true` congelaria todo membro no momento em que ele entrasse, que não é o que um crate Harvest quer. O que você precisa é do *slot de delegado ocupado por uma autoridade que você controla*, para que um crate específico possa depois ser marcado como não transferível enquanto o resto do drop continua negociável.

3. **Escreva a matemática de dimensionamento e a checagem de custo da árvore.** Você já tem `tree-size.ts` da seção de teoria. Rode contra o seu alvo real antes de gastar:

    ```bash
    npx tsx tree-cost.ts 16384 64
    ```

    Isso imprime a varredura de canopy para uma árvore de 16,384 folhas. Esta aqui você consegue casar exatamente, diferente da linha de um milhão de folhas: a Metaplex publica profundidade 14 com buffer 64 e canopy 8 a 0.3358 SOL, que é precisamente a spec que este lab constrói. A página deles é anterior ao corte de rent, então case em BYTES e não em SOL: canopy 8 dá 48,120 bytes, que são 0.3358 SOL a 6,960 lamports/byte e imprimem como 0.245 SOL na devnet hoje. A varredura também vai te mostrar canopy 0 em 31,800 bytes, dois terços da conta e catorze nós de prova em toda escrita de cliente em vez de seis. Mesmo formato da tabela de um milhão de folhas, duas ordens de magnitude abaixo.

4. **Crie a árvore e cunhe um crate (trabalhado; o passo 5 te entrega as decisões de spec).** Aqui está o script principal. Leia ele inteiro antes de rodar.

    ```typescript
    // overgrowth/mint-harvest-crates.ts
    import { generateSigner, publicKey, type PublicKey, type Umi } from "@metaplex-foundation/umi";
    import {
      createTreeV2,
      findLeafAssetIdPda,
      mintV2,
      parseLeafFromMintV2Transaction,
    } from "@metaplex-foundation/mpl-bubblegum";
    import type { DasApiAsset } from "@metaplex-foundation/digital-asset-standard-api";
    import assert from "node:assert/strict";
    import { getOrCreateAlmanac } from "./collection";
    import { depthForSupply, readLamportsPerByte, treeCostSol } from "./tree-size";
    import { book, getUmi, remember } from "./umi";

    const CRATE_SUPPLY = 16_384;
    const MAX_BUFFER_SIZE = 64;
    const CANOPY_DEPTH = 8;

    async function createCrateTree(umi: Umi): Promise<PublicKey> {
      const saved = book().tree;
      if (saved) return publicKey(saved);

      const spec = {
        maxDepth: depthForSupply(CRATE_SUPPLY),
        maxBufferSize: MAX_BUFFER_SIZE,
        canopyDepth: CANOPY_DEPTH,
      };
      const rate = await readLamportsPerByte(process.env.DAS_RPC_URL!);
      console.log(`tree spec ${JSON.stringify(spec)} -> ${treeCostSol(spec, rate).toFixed(4)} SOL of rent at ${rate} lamports/byte`);

      const merkleTree = generateSigner(umi);
      const builder = await createTreeV2(umi, { merkleTree, ...spec });
      await builder.sendAndConfirm(umi);

      remember("tree", merkleTree.publicKey);
      return merkleTree.publicKey;
    }

    export async function mintCrate(
      umi: Umi,
      merkleTree: PublicKey,
      coreCollection: PublicKey,
      name: string,
    ): Promise<PublicKey> {
      const { signature } = await mintV2(umi, {
        merkleTree,
        coreCollection,
        leafOwner: umi.identity.publicKey,
        metadata: {
          name,
          uri: "https://overgrowth.example/crates/harvest.json",
          sellerFeeBasisPoints: 500,
          collection: coreCollection,
          creators: [{ address: umi.identity.publicKey, verified: true, share: 100 }],
        },
      }).sendAndConfirm(umi);

      const leaf = await parseLeafFromMintV2Transaction(umi, signature);
      const [assetId] = findLeafAssetIdPda(umi, { merkleTree, leafIndex: leaf.nonce });
      return assetId;
    }

    async function resolveThroughDas(umi: Umi, assetId: PublicKey): Promise<DasApiAsset> {
      for (let attempt = 0; attempt < 20; attempt += 1) {
        try {
          return await umi.rpc.getAsset(assetId);
        } catch {
          await new Promise((r) => setTimeout(r, 3000));
        }
      }
      throw new Error(`DAS never indexed ${assetId} - is DAS_RPC_URL a DAS provider?`);
    }

    async function main(): Promise<void> {
      const umi = await getUmi();
      const collection = await getOrCreateAlmanac(umi);
      const merkleTree = await createCrateTree(umi);
      console.log(`tree      ${merkleTree}`);
      console.log(`collection ${collection}`);

      const crate = await mintCrate(umi, merkleTree, collection, "Harvest Crate");
      remember("crate", crate);
      console.log(`crate     ${crate}`);

      const asset = await resolveThroughDas(umi, crate);
      assert.equal(asset.compression.compressed, true, "crate is not compressed");
      assert.equal(asset.compression.tree, merkleTree, "crate is in the wrong tree");
      assert.equal(asset.grouping[0]?.group_value, collection, "crate is not in the Almanac");
      console.log(`OK: getAsset resolved ${asset.content.metadata.name} under the Almanac collection`);

      const achievement = book().achievement;
      assert.ok(achievement, 'crates.json is missing "achievement" - the soulbound crate is yours to mint');
      const soulbound = await resolveThroughDas(umi, publicKey(achievement));
      assert.equal(soulbound.compression.compressed, true);
      console.log(`OK: soulbound crate ${achievement} still resolves through DAS`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Três detalhes ali merecem o porquê deles. `createTreeV2` é async e retorna um builder em vez de uma transação, porque ele tem que perguntar ao RPC quanto rent uma árvore daquele tamanho precisa antes de conseguir compor a criação da conta com a instrução de config da árvore. `parseLeafFromMintV2Transaction` extrai a folha dos próprios logs da transação de mint, que é como você descobre o índice da folha sem consultar nada. E `findLeafAssetIdPda(umi, { merkleTree, leafIndex })` é `PDA(tree, leaf index)` em código: derivação pura, sem chamada de rede, o asset id calculado a partir de dois valores que você já tem.

    O loop de retry em volta do `getAsset` não é preenchimento defensivo. Um provedor de DAS indexa a partir dos logs de transação, então existe uma lacuna real entre "o seu mint confirmou" e "o seu crate é consultável," normalmente alguns segundos na devnet. Sem o loop a sua primeira rodada falha e você gasta vinte minutos depurando uma árvore que estava boa.

5. **Escolha e defenda a spec.** Antes de rodar qualquer coisa: a spec da árvore é a parte que é sua. Descubra de que `maxDepth` um supply de 16,384 crates precisa e confira contra `DEPTHS` no `tree-size.ts`. Depois decida `maxBufferSize` e `canopyDepth` por conta própria a partir da varredura do passo 3 em vez de aceitar as constantes no topo do arquivo, e esteja pronto para dizer em voz alta por que escolheu elas. As constantes mostradas são uma resposta defensável, não a resposta.

    Então rode:

    ```bash
    npx tsx mint-harvest-crates.ts
    ```

    Você deve ver a linha de spec da árvore com a figura de SOL dela, o endereço da árvore, o endereço da coleção, o asset id do crate, e então, depois de uma pausa, `OK: getAsset resolved Harvest Crate under the Almanac collection`. Aí ele para, com `crates.json is missing "achievement"`. Isso está correto. A cancela está te dizendo o que falta.

6. **Prove que o ativo não tem conta.** Uma linha, e é a lição inteira numa única checagem. Com o id do crate de `crates.json`:

    ```bash
    solana account $(node -p "require('./crates.json').crate") --url devnet
    ```

    A CLI reporta que a conta não existe. O seu crate é real, ele está numa coleção verificada, o DAS acabou de descrever ele para você por inteiro, e não tem conta. Sente com isso por um segundo, porque a próxima lição é construída exatamente em cima dessa lacuna.

![Um fluxograma de seis passos traçando um crate Harvest da chamada mintV2 pelo hashing da folha, o log do Noop, o parsing do índice da folha, a derivação offline do asset id, e finalmente a indexação DAS onde getAsset resolve ele.](assets/v08-flowchart.webp)

7. **Leia o que o índice te deu.** Imprima a resposta bruta do `getAsset` uma vez, só para ver o formato:

    ```typescript
    // overgrowth/show-crate.ts
    import { publicKey } from "@metaplex-foundation/umi";
    import { book, getUmi } from "./umi";

    async function main(): Promise<void> {
      const umi = await getUmi();
      const asset = await umi.rpc.getAsset(publicKey(book().crate));
      console.log(JSON.stringify(asset, null, 2));
      console.log(`compressed: ${asset.compression.compressed}`);
      console.log(`tree:       ${asset.compression.tree}`);
      console.log(`collection: ${asset.grouping[0]?.group_value}`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Olhe três campos especificamente: `compression.compressed` é `true`, `compression.tree` é o endereço da sua árvore, e `grouping[0].group_value` é a coleção do Almanac. Esses três são o que a cancela faz assert, e são o eco, do lado do DAS, das três decisões que você tomou na hora do mint.

    Note o que a resposta *não* contém: qualquer pista de que isso é barato. O JSON parece um NFT. Carteiras renderizam ele como um NFT, marketplaces listam ele como um NFT, e o seu client de jogo lê `content.metadata.name` exatamente do jeito que lê o de um volume do Almanac. A compressão é invisível na camada de leitura, que é precisamente por que ela funciona como decisão de produto e precisamente por que é fácil subestimar o compromisso operacional embaixo. O resto daquele JSON é material da próxima lição.

## Challenge

Solo. Cunhe o crate de conquista Founding-Farmer e torne ele soulbound. Nota de namespace antes de você reclamar que já tem um desses: o BADGE Founding-Farmer da m06-l2 é um ativo Core no seu surfnet, e este CRATE de conquista é uma folha cNFT na árvore. Mesmo honorífico, dois padrões diferentes, os dois deliberadamente soulbound; a Overgrowth dá aos fundadores dela um de cada, e ter o par é uma boa lembrança das duas arquiteturas.

Escreva `overgrowth/soulbound.ts` exportando exatamente estas duas funções, porque o script da cancela e trabalhos posteriores chamam elas por esta interface:

```typescript
// overgrowth/soulbound.ts
import type { PublicKey, Umi } from "@metaplex-foundation/umi";

export async function markSoulbound(
  umi: Umi,
  assetId: PublicKey,
  coreCollection: PublicKey,
): Promise<void> {
  throw new Error(`implement me: ${umi.identity.publicKey} ${assetId} ${coreCollection}`);
}

export async function transferMustFail(
  umi: Umi,
  assetId: PublicKey,
  coreCollection: PublicKey,
): Promise<void> {
  throw new Error(`implement me: ${umi.identity.publicKey} ${assetId} ${coreCollection}`);
}
```

`markSoulbound` chama `set_non_transferable_v2`. A instrução quer uma raiz, um hash de dados, um hash de criador, um nonce, um índice e uma prova, que são seis coisas que você não quer montar à mão. `getAssetWithProof(umi, assetId, { truncateCanopy: true })` retorna um objeto cujos campos se espalham direto para dentro do input da instrução, e `truncateCanopy` manda ele descartar os nós de prova que a árvore já cacheia, então você envia seis em vez de catorze. A `authority` que assina tem que ser o delegado de congelamento permanente que você armou na coleção no passo 2 do lab.

`transferMustFail` busca uma prova **nova** (a escrita do `set_non_transferable_v2` mudou a raiz, então a que você acabou de usar está desatualizada), tenta um `transferV2` para um signer descartável, e faz assert de que o envio rejeita. Capture a rejeição e faça assert de que capturou. Um test que passa porque a sua transferência silenciosamente não fez nada não é um test.

Depois ligue as duas no `mint-harvest-crates.ts`: cunhe um segundo crate chamado `Founding Farmer` com `mintCrate`, faça `remember("achievement"...)` do asset id dele, marque ele como soulbound, e prove que a transferência falha. Para tornar a prova honesta, transfira o crate Harvest comum para o mesmo signer descartável na mesma rodada e faça assert de que essa *tem sucesso*. Se só o crate de conquista falhar, o congelamento é uma propriedade daquele crate. Se os dois falharem, você tem uma coleção mal configurada e estava prestes a entregá-la.

Aceito quando: `npx tsx mint-harvest-crates.ts` rodar limpo de ponta a ponta; `getAsset` resolver os dois crates sob a coleção do Almanac; a transferência do crate de conquista for rejeitada enquanto a transferência do crate Harvest cai na mesma rodada; e a sua saída do `tree-cost.ts` para a árvore que você de fato criou casar com a sua própria figura derivada do dimensionador dentro do arredondamento. Se você pegou a spec default de profundidade 14 / buffer 64 / canopy 8, essa figura também casa com o número publicado pelo fornecedor; se você exerceu a liberdade do passo 5 e escolheu uma spec legal diferente, não tem número de fornecedor para casar, e o seu dimensionador É a fonte, que é o ponto de ter construído ele.

## Checkpoint

A cancela: `npx tsx mint-harvest-crates.ts` imprime o endereço da árvore dele, o endereço da coleção dele, dois asset ids, e as duas linhas `OK`. Com isso verde, `harvest-crates` está completo: uma árvore Bubblegum v2 dimensionada pela sua própria matemática, um crate Harvest cunhado na coleção Core Almanac verificada, e um crate de conquista soulbound que o DAS ainda descreve numa boa e que ninguém consegue mover.

Três respostas que você deveria conseguir dar sem consultar nada. Onde os metadados de um crate moram on-chain? Não moram; um hash de 32 bytes deles fica numa folha, e a versão legível mora no índice de um provedor. O que uma árvore de SOL de um dígito compra? Mais ou menos um milhão de folhas, alguns milionésimos de um SOL cada, e seis sétimos daquela conta são canopy e não árvore. E um cNFT pode ser soulbound? Pode, desde a v2, via `set_non_transferable_v2` numa coleção que carrega um delegado de congelamento permanente, não importa o que um post de blog de 2024 te disse.

Se a sua rodada do challenge está vermelha agora, a correção é quase sempre uma de duas coisas. Hashing mismatch quer dizer prova desatualizada: busque de novo imediatamente antes da escrita, toda escrita, sem exceção. Um erro de autoridade no `set_non_transferable_v2` quer dizer que o signer não é um delegado de congelamento permanente na coleção Core, o que quer dizer que o passo 2 criou a sua coleção sem o plugin e você precisa de uma nova.

![Um diagrama de hub com harvest-crates no centro, alimentado pela coleção Core do Almanac e alimentando a lição do leitor de DAS, a lição da fronteira da compressão, e o capstone.](assets/v09-diagram.webp)

Você agora tem SPROUT, ativos Core do Almanac e cNFTs de crate Harvest espalhados por três formatos on-chain diferentes, e um deles você nem consegue buscar com `getAccountInfo`. Próxima lição, um script lê os três.
