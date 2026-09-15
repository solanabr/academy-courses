# Feche as anotações: três briefs a frio, e a porta que você atravessa em seguida

## Resumo

O R13, o capstone, foi a última construção que este curso te pede. Faltam dois movimentos, exatamente como o fechamento do capstone prometeu: um checkpoint cumulativo sobre três briefs a frio, pontuado contra a matriz de conflitos do módulo um e a tese de compatibilidade do módulo cinco, e então o encerramento. Um brief é inédito, um refaz o formato de badge em massa que o capstone precificou mas nunca te fez construir, e um fica de propósito ao lado do memo trabalhado do café, a âncora de calibragem. O recuo é total. Nenhuma resposta trabalhada chega antes da sua.

Então feche as anotações. Abas do navegador incluídas. Aqui estão os três briefs.

**Brief um, o café da esquina.** Uma rede de cafés com onze lojas quer uma moeda de fidelidade. Os clientes ganham ela a cada compra e podem vender ela entre si, e o dono é firme que ela precisa ser listável na Raydium para que o preço seja público. Nenhum regulador está envolvido, nenhum controle de compliance é desejado, e mais ou menos 40,000 clientes vão ter saldo.

**Brief dois, o badge de conclusão.** Um clube de corrida quer entregar a cada um dos seus um milhão de membros um badge por terminar uma maratona virtual. Tem que ser barato nessa contagem, e um badge que aparece à venda em um marketplace é um constrangimento que eles não vão aceitar.

**Brief três, a cota da co-op.** Uma co-op de alimentos tokeniza cotas de membros, uma emissão por membro, umas 900 delas, divisíveis porque a divisão anual de sobras cai em frações. O conselho precisa conseguir congelar uma cota quando um membro é expulso, e a cota nunca pode ser negociada em um venue público.

![Uma tabela de decisão de seis colunas em branco com uma linha por brief a frio e células vazias para família de primitiva, conjunto de extensões, veredicto de compatibilidade, trilho de economia e defesa.](assets/v01-table.png)

Agora preencha quatro células para cada brief, em um arquivo de texto, na mão: a família de primitiva, o conjunto de extensões ou plugins, o veredicto de compatibilidade, o único trilho de economia. Uma frase de defesa por linha. Sem pesquisar, sem rolar para trás. Dê quinze minutos e aceite o que sair, com brancos e tudo.

## O checkpoint é um espelho, não um teste

Vale nomear duas palavras antes de a gente seguir, porque elas são o método inteiro. Um **checkpoint cumulativo** é uma passada de recuperação, sem consulta, sobre um curso inteiro, produzida em vez de lida, e não tem nota e não barra nada. Uma **tabela de decisão** é a forma comprimida de tudo o que este curso ensinou: quatro células por brief, porque quatro células é o que uma decisão de produto de verdade realmente precisa antes de alguém abrir um editor.

Quero ser direto sobre por que a regra de produzir importa, porque eu já estive do lado errado dela. Ler um gabarito é exatamente igual a saber a resposta. O reconhecimento é instantâneo, o material parece familiar, e você fecha a aba se sentindo fluente em algo que não conseguiria ter gerado. Essa sensação é a ilusão mais cara que existe no aprendizado técnico. Ela não te custa nada hoje e te custa um re-mint depois, porque o conjunto de extensões de um mint é fixado na criação (as exceções são todas escritas de ponteiro-depois-realloc escolhidas no nascimento de qualquer jeito — metadados, as TLVs de grupo e as extensões no nível da conta — e nenhuma extensão de poder está entre elas), então um conjunto errado não é um patch, é um mint novo e uma migração.

A segunda regra é mais gentil. Uma célula em branco não é um fracasso, é um endereço. Cada ponto em que você empacou nos últimos quinze minutos aponta para exatamente um módulo, e no fim desta lição você vai ter uma pequena tabela mapeando cada tipo de branco para a lição que o preenche. Essa é a saída de verdade de um checkpoint. Não uma nota.

### Os dois instrumentos contra os quais você checa

Você tem duas regras para checar uma linha, e elas respondem perguntas diferentes, em uma ordem estrita.

A **matriz de conflitos** do módulo um responde "este mint vai sequer inicializar?" Você a portou da fonte da verdade, `check_for_invalid_mint_extension_combinations`, para o `checkCombo`, uma função pura sem RPC dentro. As regras dela são estruturais: `ScaledUiAmount` e `InterestBearingConfig` são mutuamente exclusivos porque dois multiplicadores de UI diferentes em um saldo é um absurdo. `ConfidentialMintBurn` exige `ConfidentialTransferMint`, porque o caminho de queima precisa do supply criptografado sobre o qual ele opera. E os pares forçados de `required_init_account_extensions` significam que escolher `TransferFeeConfig` no mint obriga silenciosamente um `TransferFeeAmount` em toda conta. Vinte e nove variantes de produção no enum `ExtensionType`, e a matriz é a diferença entre combinações que existem e combinações que inicializam.

A **tese de compatibilidade** do módulo cinco responde uma pergunta mais fria: "alguém vai conseguir negociar isso?" A allowlist do CP-Swap da Raydium é exatamente cinco extensões Token-2022, e você provou o formato da regra com o seu próprio preditor. É `every`, não `some`. Cinco extensões na allowlist mais uma extensão recusada continua recusado, porque a exposição da pool a um delegado permanente não encolhe quando uma config de taxa senta do lado. A Raydium escreveu o motivo em linguagem simples: um holder do delegado pode varrer qualquer conta de token, incluindo o vault da pool. A saída de emergência é uma `MINT_WHITELIST` hardcoded de quatro entradas, que não é uma coisa que você solicita numa terça-feira.

![Uma comparação em dois painéis da matriz de conflitos, que pergunta se um mint inicializa, contra a tese de compatibilidade, que pergunta se um venue precifica ele, checadas nessa ordem.](assets/v02-comparison.png)

### Linha um, checada

O brief do café te entrega duas restrições e uma ausência, e a ausência é a parte barulhenta. Ele precisa continuar negociável. Ele tem 40,000 holders. E ninguém pediu controles de compliance, o que significa que toda extensão de poder que você pudesse buscar é um custo sem comprador.

Então a linha defensável é um mint Token-2022 carregando extensões só de exibição, `MetadataPointer` mais `TokenMetadata`, e nada mais. As duas estão na allowlist, então o `isRoutable` volta true e a célula de veredicto diz roteável. O trilho é uma rota de taxa se a loja quiser uma fatia das trocas secundárias, o que significa que `TransferFeeConfig` precisa estar no conjunto declarado na criação, a regra de só-no-nascimento de novo, e essa também está na allowlist, também tudo bem. SPL clássico funcionaria também, e não está errado, só joga fora metadados nativos sem razão nenhuma em 2026.

A cilada nesta linha é um `TransferHook`, e é uma cilada tentadora. Logar toda transferência de fidelidade on-chain soa exatamente como o que um programa de fidelidade quer. Ele também invoca um programa customizado em toda transferência com consumo de compute arbitrário, que é precisamente por que a Raydium o recusa, e o brief disse que o preço tem que ser público. Se a sua linha tem um hook dentro, você não errou o mecanismo. Você errou a ordem: você satisfez a feature antes de satisfazer a restrição que foi declarada como inegociável.

### Linha dois, checada

O brief do badge é o que separa um modelo mental atual de um herdado, porque ele empilha duas restrições que costumavam ser respondidas por duas primitivas diferentes.

Barato em um milhão é o trabalho da compressão e de mais nada. O Metaplex Core é genuinamente barato por ativo, em torno de 0.003 SOL, publicado pelo fornecedor, que é um número maravilhoso até o momento em que você multiplica ele por um milhão e chega em mais ou menos 3,000 SOL. Uma árvore Bubblegum v2 dimensionada para um milhão de folhas tem 1,223,352 bytes, o que cai na casa de um dígito de SOL em qualquer taxa de rent que a rede cobrou este ano. Esse é um negócio melhor por mais de duas ordens de magnitude, o tipo de diferença em que a aritmética decide, não o gosto.

Não vendável é a metade que costumava quebrar isso. O folclore de 2024 diz que NFTs comprimidos não podem ser congelados nem tornados soulbound, o que era verdade então e é simplesmente falso agora. O Bubblegum v2 já vem com `set_non_transferable_v2`, e um NFT comprimido pode ser soulbound na cunhagem. Então a linha é um cNFT sob a coleção do clube, tornado não transferível, com uma cancela como trilho, e a célula de veredicto diz não aplicável porque um badge soulbound nunca ia para uma pool.

![Um gráfico de barras em escala logarítmica comparando o custo de um milhão de posições entre ativos Core, contas de token SPL clássicas e uma única árvore Bubblegum v2, que é mais barata por mais de duas ordens de magnitude.](assets/v03-chart.png)

### Linha três, checada

A cota da co-op inverte a linha um, e se você acertou as duas linhas você tem a tese, não só a regra.

Aqui o conselho quer autoridade de congelamento sobre o saldo de um holder, e o brief diz explicitamente que ela nunca é negociada publicamente. Então as extensões de poder que eram desqualificadoras para o café estão simplesmente disponíveis. `DefaultAccountState` setado como congelado dá à co-op um passo de aprovação antes que qualquer novo membro possa ter saldo, e um delegado permanente ou uma autoridade de congelamento dá ao conselho o caminho de expulsão dele. A célula de veredicto diz não roteável, e escrever isso é o ponto inteiro da linha: é um não-roteável escolhido, precificado e aceito, em vez de um não-roteável que você descobre no lançamento. O trilho é uma cancela, o mesmo padrão que deixou um cNFT Founding Farmer abrir o alpha do Overgrowth, apontado para associação em vez disso.

É aqui que a tese de compatibilidade se paga como ferramenta de design em vez de rótulo de aviso. Ela nunca disse que extensões de poder são ruins. Ela disse que elas custam venues, e um produto que não quer venues não paga nada.

### Para onde uma célula em branco aponta

Agora faça a coisa para a qual o checkpoint existe. Olhe o que você não conseguiu preencher e mapeie.

| A célula em que você empacou | O que isso significa que você pulou | Onde ela mora |
|---|---|---|
| família de primitiva, em qualquer linha | você tem as peças mas não o seletor | módulo um, o framework de decisão e a matriz de conflitos |
| conjunto de extensões, em uma linha fungível | o catálogo nunca foi comprimido em economia, autoridade e exibição | módulo dois, as quatro lições de extensões |
| a célula de veredicto | a tese de compatibilidade ficou um fato em vez de virar um hábito | módulo cinco, o artefato de allowlist e o design do token roteável |
| a linha do badge inteira | a compressão ainda está arquivada como uma opção exótica | módulo sete, Bubblegum v2 e a derivação de custo |
| a célula de trilho | você construiu os ativos e nunca ligou valor entre eles | módulo nove, roteamento de taxas, cancelas e migrações; m08-l3 se o branco foi o trilho do airdrop |
| como você provaria qualquer disso | ler ativos ainda é trabalho de outra pessoa | módulo sete, a lição de DAS e o leitor R9 |

Seis linhas, e uma reação madura a duas delas acenderem é ir reler essas duas lições, não se sentir mal. A tabela é o entregável.

## Lab: pontue a sua própria tabela

A ferramenta abaixo deliberadamente não é um gabarito. Ela conhece as restrições declaradas dos três briefs, que você consegue ler do texto acima, mais as duas regras ensinadas. Ela não faz ideia de qual é a primitiva certa. Essa distinção é o que a torna um espelho: ela consegue te dizer que uma linha se contradiz, e não consegue te dizer o que escrever.

**1.** Faça o scaffold de uma pasta ao lado do seu trabalho do capstone e instale o runner. Duas dev dependencies, nada que fale com uma chain, porque uma checagem de regras deveria rodar em milissegundos com o wifi desligado. O único import de fora desta pasta é o seu próprio `checkCombo` do lab de m01-l4, que é igualmente offline: uma função pura, sem RPC dentro.

```bash
mkdir -p checkpoint && cd checkpoint
npm init -y
npm pkg set type=module
npm install -D tsx@4.23.12 @types/node@24
```

Essas são as versões que eu rodei em 2026-08-22, e as duas se movem rápido o suficiente para que você deva refixar em vez de copiar elas daqui a um ano. `tsx` roda TypeScript sem passo de build; os tipos do Node impedem o `process.exit` de brilhar em vermelho no seu editor.

**2.** Crie o `score-table.ts`. Leia a metade de cima como uma reafirmação dos dois instrumentos, porque é isso que ela é: a matriz de conflitos importada direto do seu port de m01-l4, a allowlist do módulo cinco, as constantes de custo dos módulos seis, sete e oito, e as restrições dos briefs transcritas de prosa para campos.

```typescript
// score-table.ts: scores YOUR filled decision table against the two taught rules.
// There is no answer key in this file. It knows each brief's stated constraints
// and the rules from m01-l4 (the conflict matrix) and m05-l2 (the thesis).
import { checkCombo } from "../check-combo"; // instrument A, the m01-l4 port, verbatim

// Deliberately identical to memo.ts's union from the capstone, character for
// character, so the rows you transcribe from your memo keep their spellings.
// tsx strips types without checking them, so a drifted literal here would not
// error, it would silently misclassify: the worst kind of wrong.
export type PrimitiveFamily = "spl-token" | "token-2022" | "core-asset" | "cnft";

// Read straight off the brief text. Constraints, never answers.
// Deliberately NOT transcribed: brief three's "board can freeze a share".
// Every mint carries a base freeze authority whether or not any extension is
// present, so a boolean here would wrongly flag rows that lean on it. That
// constraint gets checked in your DEFENSE sentence, by you, and naming the
// omission beats pretending the transcription is complete.
export interface BriefConstraints {
  id: string;
  fungible: boolean;
  mustStayTradeable: boolean;
  soulbound: boolean;
  units: number;
}

// One row of your table.
export interface DecisionRow {
  brief: string;
  primitive: PrimitiveFamily;
  set: string[];
  verdict: "routable" | "not-routable" | "n/a";
  rail: string;
}

// m05-l2: Raydium CP-Swap's Token-2022 allowlist is exactly five extensions.
export const RAYDIUM_CP_SWAP_ALLOWLIST: readonly string[] = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "InterestBearingConfig",
  "ScaledUiAmount",
];

// Anything that makes a holder unable to move the asset, spelled the way each
// primitive's own lesson spelled it: the Token-2022 extension, the Core plugin,
// and the Bubblegum v2 instruction (a cNFT row names the instruction, because
// DAS's structural tags cannot express soulbound-ness; m10-l1's gate said so).
const SOULBINDING: readonly string[] = [
  "NonTransferable",
  "PermanentFreezeDelegate",
  "set_non_transferable_v2",
];

export function isRoutable(row: DecisionRow): boolean {
  if (row.primitive === "spl-token") return true;
  if (row.primitive !== "token-2022") return false;
  // Allowlisting is not additive: EVERY extension has to be on the list.
  return row.set.every((e) => RAYDIUM_CP_SWAP_ALLOWLIST.includes(e));
}

const SOL_PER_CORE_ASSET = 0.003; // Metaplex's published ~0.003, read 2026-09-07
// Module 7's depth-20 / buffer-256 / canopy-14 tree: 1,223,352 bytes.
// Scaling this linearly is a deliberate simplification. Tree rent is set by
// depth, buffer and canopy, not by leaf count, so a small tree costs MORE per
// leaf and this understates it. Good enough to trip the 100-SOL alarm, not a budget.
//
// The two rent figures below are BYTES x a per-byte rate, and the rate is a
// network parameter SIMD-0437 is stepping down. 6,333 is mainnet's, read
// 2026-09-06; devnet was already at 5,080. Re-read it (`solana rent 0`, divide
// by 128) before this alarm decides anything real.
const LAMPORTS_PER_BYTE = 6_333;
const SOL_PER_MILLION_CNFTS = ((128 + 1_223_352) * LAMPORTS_PER_BYTE) / 1e9;
const LAMPORTS_PER_CLASSIC_ATA = 293 * LAMPORTS_PER_BYTE;

export function mintCostSol(primitive: PrimitiveFamily, units: number): number {
  switch (primitive) {
    case "core-asset":
      return units * SOL_PER_CORE_ASSET;
    case "cnft":
      return (units / 1_000_000) * SOL_PER_MILLION_CNFTS;
    case "spl-token":
    case "token-2022":
      // Deliberate simplification, same class as the cNFT linearization above:
      // this prices every holder at the bare 165-byte ATA rent. A token-2022
      // mint with TransferFeeConfig forces a TransferFeeAmount slot onto every
      // holder account (the m02-l1 forced-pair rule), so real per-holder rent
      // runs a few dozen bytes higher. The model floors the cost; a row that
      // only squeaks under a rent alarm at the floor deserves a second look.
      return (units * LAMPORTS_PER_CLASSIC_ATA) / 1e9;
  }
}

export function checkRow(row: DecisionRow, brief: BriefConstraints): string[] {
  const problems: string[] = [];
  const isNftFamily = row.primitive === "core-asset" || row.primitive === "cnft";

  // Instrument A fires first, exactly as the theory section ordered it: a set
  // the program refuses at initialize_mint never reaches a venue's opinion.
  if (row.primitive === "token-2022") {
    const combo = checkCombo(row.set);
    if (!combo.valid) {
      problems.push(`conflict matrix: ${combo.reason ?? "invalid combination"}`);
    }
  }

  if (brief.fungible && isNftFamily) {
    problems.push(`${row.primitive} is not a divisible balance; this brief needs a fungible mint`);
  }
  if (!brief.fungible && !isNftFamily) {
    problems.push(`a fungible mint cannot carry per-item metadata; this brief needs an asset`);
  }
  if (brief.mustStayTradeable && !isRoutable(row)) {
    const offenders = row.set.filter((e) => !RAYDIUM_CP_SWAP_ALLOWLIST.includes(e));
    problems.push(
      `must stay tradeable, but pool creation reverts: ${offenders.join(", ") || row.primitive}`,
    );
  }
  if (brief.mustStayTradeable && row.verdict !== "routable") {
    problems.push(`verdict says "${row.verdict}" while the brief demands a routable mint`);
  }
  if (brief.soulbound && !row.set.some((e) => SOULBINDING.includes(e))) {
    problems.push("brief says soulbound, nothing in the set stops a transfer");
  }
  if (row.rail.trim().length === 0) {
    problems.push("no economy rail named");
  }

  const cost = mintCostSol(row.primitive, brief.units);
  if (cost > 100) {
    problems.push(`mint cost ~${cost.toFixed(1)} SOL for ${brief.units.toLocaleString("en-US")} holders`);
  }
  return problems;
}

const BRIEFS: BriefConstraints[] = [
  { id: "cafe", fungible: true, mustStayTradeable: true, soulbound: false, units: 40_000 },
  { id: "badge", fungible: false, mustStayTradeable: false, soulbound: true, units: 1_000_000 },
  { id: "co-op", fungible: true, mustStayTradeable: false, soulbound: false, units: 900 },
];

// REPLACE these three rows with the ones you wrote by hand.
const MY_TABLE: DecisionRow[] = [
  {
    brief: "cafe",
    primitive: "token-2022",
    set: ["MetadataPointer", "TokenMetadata", "TransferHook"],
    verdict: "routable",
    rail: "fee route to the shop treasury",
  },
  {
    brief: "badge",
    primitive: "core-asset",
    set: ["PermanentFreezeDelegate"],
    verdict: "n/a",
    rail: "gate the members channel",
  },
  {
    brief: "co-op",
    primitive: "token-2022",
    set: ["DefaultAccountState", "PermanentDelegate", "MetadataPointer"],
    verdict: "not-routable",
    rail: "",
  },
];

function main(): void {
  let failed = 0;
  for (const row of MY_TABLE) {
    const brief = BRIEFS.find((b) => b.id === row.brief);
    if (!brief) throw new Error(`no brief named "${row.brief}"`);
    const problems = checkRow(row, brief);
    console.log(`${row.brief.padEnd(6)} ${problems.length === 0 ? "PASS" : "FAIL"}  ${row.primitive}`);
    for (const p of problems) console.log(`         - ${p}`);
    if (problems.length > 0) failed += 1;
  }
  console.log(`\n${MY_TABLE.length - failed}/${MY_TABLE.length} rows survive the rules.`);
  process.exit(failed === 0 ? 0 : 1);
}

main();
```

**3.** Rode uma vez antes de tocar no `MY_TABLE`. O que está lá dentro é uma tabela errada plausível, do tipo que eu genuinamente já escrevi à 1 da manhã, e cada linha falha por um motivo diferente.

```bash
npx tsx score-table.ts
```

Você deve ver três linhas FAIL e `0/3 rows survive the rules` no exit 1. A linha do café falha no hook, a linha do badge falha nos 3,000 SOL, e a linha da co-op falha porque eu deixei a célula de trilho vazia. Leia essas três mensagens com atenção, porque elas são os três jeitos de uma tabela de decisão dar errado na prática: uma feature que vence uma restrição, uma primitiva cujo custo só aparece quando você multiplica, e um plano sem valor se movendo através dele.

**4.** Agora cole as suas próprias três linhas por cima do `MY_TABLE`, exatamente como você escreveu elas na mão, incluindo as partes sobre as quais você tem dúvida. Não conserte elas no caminho. O ponto é pontuar o que a sua memória produziu, não o que a sua memória produziu mais quinze segundos de dúvida em cima.

**5.** Rode de novo e leia cada linha de problema como um ponteiro em vez de um veredicto. Uma linha que falha em `must stay tradeable` te manda para o módulo cinco. Uma linha falhando em custo te manda para a lição de compressão. Uma linha sem trilho te manda para o módulo nove. Linhas que passam não são prova de que você está certo, são prova de que você não está se contradizendo, que é uma barra mais baixa e mais honesta do que parece.

![Um fluxograma das oito checagens em score-table.ts, da matriz de conflitos passando por fungibilidade e roteabilidade até o custo derivado, alimentando uma lista problems compartilhada que decide PASS ou FAIL.](assets/v04-flowchart.png)

## Challenge

Três coisas, todas de escrita, nenhuma delas código.

Primeiro, defenda cada linha sobrevivente em uma frase que nomeia o instrumento. Não "cNFT porque é barato" mas "Bubblegum v2 com `set_non_transferable_v2`, porque um milhão de ativos é um problema de compressão e soulbound não é mais motivo para deixar a compressão de lado." Uma defesa que não nomeia a regra que aplicou é só uma preferência.

Segundo, anote o módulo para o qual você vai voltar, e a coisa específica que você vai re-derivar quando chegar lá. Não "reler o módulo cinco" mas "rodar de novo a checagem aditiva e ver cinco extensões da allowlist mais um delegado voltarem rejeitadas." Intenções vagas de revisar são como as pessoas terminam um curso se sentindo bem e continuam exatamente tão capazes quanto eram.

Terceiro, pegue uma das suas três linhas e escreva a frase que a viraria. A minha, para o badge: se um marketplace algum dia suportar mover um cNFT soulbound por meio de uma autoridade delegada, a garantia de "não vendável" enfraquece e a linha volta para o Core com um congelamento permanente. Toda boa decisão tem uma condição de quebra nomeada; sem uma, o que você tem é só uma preferência.

## O encerramento

Aqui está tudo o que você tem em disco, em ordem, porque ver isso como uma lista só é o ponto.

Um inspetor que decodifica qualquer mint ao vivo até as extensões TLV dele. Um validador de matriz de conflitos portado da fonte. O SPROUT em si, um mint Token-2022 com uma taxa de transferência, um harvest de taxa retida funcionando, e metadados nativos, construído a partir de instruções brutas. Um programa Rust de transfer hook com o `ExtraAccountMetaList` dele, testado no LiteSVM. Uma variante confidencial, parada de lado como o galho de emissor especializado que ela é. Um relatório de roteabilidade que rodou o SPROUT e o gêmeo com hook dele contra a lógica de allowlist de verdade. A coleção Almanac no Metaplex Core, com royalties, uma impressão de Edition, e um badge Founding Farmer não transferível. Harvest crates como NFTs comprimidos Bubblegum v2, precificados antes de serem cunhados. Um script leitor que resolve os três formatos através do DAS e sinaliza quais extensões estão vivas versus configuradas e dormentes. Uma config de lançamento cujo limiar de graduação você derivou de constantes publicadas em vez de citar. Um airdrop de compressão com um caminho de claim com vesting. E a economia que amarra tudo junto: taxas com harvest feito para uma tesouraria, um buyback comprado e queimado com a queda de supply provada até a unidade base, um cNFT servindo de cancela para o alpha, e pontos de compost migrando para o SPROUT. Então o seu próprio produto, escolhido e entregue e provado.

Quatro ideias sustentam tudo isso, e são elas que você ainda deveria ter daqui a cinco anos quando todo número de versão neste curso tiver apodrecido.

**Interface versus implementação.** O fato mais 2026 deste curso é que o programa de token foi substituído debaixo dos pés de todo mundo e o cliente de ninguém mudou um byte. O SIMD-0266 foi mergeado em 2026-03-13, o p-token da Anza baseado em Pinocchio assumiu a implementação do SPL Token clássico no mesmo endereço, e um Transfer foi de 4,645 compute units para 76. O TransferChecked foi de 6,200 para 105. Não acredite em mim sobre o estado atual, gaste trinta segundos:

```bash
curl -s -X POST https://api.mainnet-beta.solana.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP",{"encoding":"base64"}]}' \
| node -e "
const chunks=[];process.stdin.on('data',c=>chunks.push(c)).on('end',()=>{
  const v=JSON.parse(Buffer.concat(chunks).toString()).result.value;
  if(!v){console.log('gate account not found: feature never activated');process.exit(0);}
  const b=Buffer.from(v.data[0],'base64');
  console.log(b[0]===1?'ACTIVE at slot '+b.readBigUInt64LE(1):'account exists, activation slot not set (queued)');
});"
```

Quando eu rodei isso em 2026-08-22 ele imprimiu `ACTIVE at slot 419472000`, o primeiro slot da epoch 971. O gate está ativo, o motor é novo, a interface nunca se moveu. Esse é o formato de todo bom limite de abstração que você vai desenhar na vida.

**A matriz de conflitos.** Combinações que existem e combinações que inicializam são conjuntos diferentes, e a diferença está escrita em código-fonte que você consegue ler.

**A tese de compatibilidade.** Poder custa venues. É um preço, não uma proibição, e agora você sabe como pagar ele deliberadamente.

**DAS como o unificador.** Uma interface de leitura sobre um mint fungível com extensões, um ativo Core, e uma folha comprimida. Quando alguém te entregar um endereço daqui a dois anos e perguntar o que ele é, você tem um script para isso.

![Um diagrama radial centrado em um endereço desconhecido, com quatro raios para as quatro ideias duráveis do curso, cada um rotulado com a sua pergunta e o seu artefato de prova.](assets/v05-diagram.png)

### O que este curso não sabe

O limite honesto, porque esse foi o acordo o tempo todo. Este curso te deixou fluente na camada de token e de ativo e na realidade de compatibilidade dela. Ele parou na borda da graduação de propósito. Nada de design de AMM, nada de profundidade de LP, nada de matemática de pool além de derivar um limiar a partir de constantes publicadas. Ele recusou o enquadramento legal e de emissor que transforma um conjunto de extensões com formato de compliance em um instrumento regulado de verdade. Ele usou Anchor para exatamente um programa de hook de 40 linhas e não te ensinou nada do framework. Ele nunca tocou em pouso de transação, taxas de prioridade, nem infraestrutura de indexação em escala, e ficou acima do runtime o caminho inteiro.

Cada um desses é o curso de alguém, e eu posso te dizer de quem — com uma ressalva que você merece de saída, porque uma passagem de bastão para uma porta que não abre é pior do que passagem de bastão nenhuma. Dos três abaixo, só o **Master Anchor V2** está publicado hoje. Os outros dois estão planejados e ainda não foram entregues, então trate eles como um mapa do território e de onde ficam as bordas dele, não como links que você pode clicar esta tarde. O catálogo é a fonte da verdade do que de fato existe.

O curso planejado **DeFi and RWA Engineering** é o que pega as primitivas que você acabou de aprender e transforma elas em um negócio de emissão: emissão específica de RWA, os trilhos de compliance em volta dela, como emissores ao vivo de fato estruturam os programas deles, e a profundidade de LP que este curso ficou passando adiante pelo nome. Tudo o que ele assume na camada de token é o que você acabou de construir, então você está entrando pela porta da frente em vez de subir por uma janela. Se a sua linha três, a cota da co-op, pareceu querer um advogado na sala, esse é o curso onde o advogado aparece.

O curso planejado **Client-Side Mastery** é dono de tudo o que acontece entre o seu script e a chain. Pouso de transação e taxas de prioridade, a camada de indexação embaixo de uma chamada DAS, Geyser e gRPC quando um índice alugado não basta. Toda vez que este curso disse "o seu script leitor assume um RPC que suporta DAS" e seguiu em frente, aquilo era a emenda. Esse curso está do outro lado dela.

O curso **Master Anchor V2** é o framework em si. Macros, constraints, mecânica de CPI, testes, migração. Você escreveu um programa de hook aqui e eu te disse o que digitar, deliberadamente, porque um transfer hook é um conceito de token e Anchor é um conceito de framework e misturar os dois teria piorado os dois. Se aquele programa foram as quarenta linhas mais interessantes do curso para você, essa é a sua próxima porta.

### Uma última coisa, e um favor

A educação oficial de Solana congelou no meio da trama. O repositório `solana-foundation/developer-content` virou somente leitura em 2025-01-24, e todo curso oficial sentado atrás daqueles links é anterior a `ScaledUiAmount`, `Pausable`, `ConfidentialMintBurn`, Bubblegum v2, Genesis e p-token. Isso não é uma reclamação sobre as pessoas que escreveram eles. É o melhor argumento que existe para o hábito que este checkpoint estava treinando: re-derivar, re-sondar, reler a fonte, porque o próprio texto canônico do ecossistema pode parar de atualizar e parou, enquanto a chain continuava se movendo.

![Uma linha do tempo de 2024 até agosto de 2026 marcando o lançamento do PYUSD, o arquivamento do conteúdo oficial para desenvolvedores, o desligamento do SimpleHash, o merge do SIMD-0266, e a ativação do gate do p-token.](assets/v06-timeline.png)

Agora o favor, e ele é de verdade. Todo número neste curso é datado e a maioria deles vai derivar: os compute units, o conteúdo da allowlist, os custos por ativo, as versões das ferramentas, as constantes de graduação. Se você rodar de novo uma sondagem de qualquer lição e a sua saída discordar da minha, poste a lição, o comando exato, e o que você obteve no canal de feedback do curso. Isso não é um report de bug por cortesia. É o mecanismo de manutenção de verdade do curso, e um aprendiz que pega um número velho é o hábito funcionando em voz alta na frente de todo mundo.

Não existe próxima lição. Essa é a parte estranha de terminar alguma coisa. Você é quem decide o que acontece com isso agora, e honestamente, o menor próximo passo útil não é outro curso de jeito nenhum: pegue um ativo do seu capstone, entregue o endereço dele para um amigo, e veja se o seu script leitor explica ele para essa pessoa. Essa é a habilidade, e não é tão difícil manter ela afiada depois que você tem.

Você consegue ler os bytes de qualquer mint ao vivo, escolher a primitiva certa, e provar que ela resolve. Qualquer porta que você escolher em seguida, RWA e DeFi, client-side e indexação, ou o próprio framework Anchor, a camada de token agora é sua para levar através dela. Vá re-derivar alguma coisa que ninguém checou ultimamente.
