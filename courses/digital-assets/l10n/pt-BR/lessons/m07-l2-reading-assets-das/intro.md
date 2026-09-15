# Leia qualquer coisa: um script DAS para fungíveis, ativos Core e cNFTs

## Resumo

Na lição passada você cunhou crates Harvest como NFTs comprimidos Bubblegum v2 e aprendeu que o id de ativo de um crate é um PDA derivado do endereço da árvore e do índice da folha. Aí você foi buscar um e não veio nada, porque um cNFT não tem conta própria. Você também ainda tem o SPROUT, o mint Token-2022 que você vem estendendo desde o módulo 2, e os ativos Core do Almanac que você cunhou no m06-l2. Esses dois são contas comuns. Qualquer RPC devolve os dois.

Então agora você está com três ativos em três formatos diferentes, e uma integração de carteira tem que entender os três sem se importar com qual é qual. Prove a assimetria para você mesmo antes de a gente nomear a correção. No seu workspace do curso:

```bash
mkdir -p overgrowth && cd overgrowth
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Pins, conferidos contra o npm em 2026-09-05. A tag `latest` do kit é 8.2.0, mas o número que decide o pin é o peer range, não a tag latest: `@solana-program/token-2022@0.15.0`, o minor atual do cliente deste curso, tem peer no kit ^7.0.0 — a linha 0.16 foi para ^8 — então o kit é 7.1.1, o release mais novo dentro dessa faixa. Esse trem entrega mensalmente. Rode `npm view @solana-program/token-2022 peerDependencies` no dia em que você fizer o scaffold.

```typescript
// overgrowth/shape-check.ts - does this id have an account at all?
// Run (from inside overgrowth/): npx tsx shape-check.ts <sprout> <almanac> <crate>
import { createSolanaRpc, address } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

async function main(): Promise<void> {
  for (const id of process.argv.slice(2)) {
    const { value } = await rpc.getAccountInfo(address(id), { encoding: 'base64' }).send();
    console.log(
      value === null
        ? `${id}  ->  NO ACCOUNT`
        : `${id}  ->  ${value.data[0].length} base64 chars, owner ${value.owner}`,
    );
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Uma nota de cluster antes de você rodar: neste ponto o SPROUT e os seus ativos do Almanac ainda vivem no surfnet (o lab migra os dois para a devnet em uns quinze minutos), então aponte `RPC_URL` para o seu surfnet nesta primeira rodada. O veredicto do crate não depende do cluster: `NO ACCOUNT` é o que imprime para ele contra o seu surfnet, contra a devnet onde ele de fato vive, contra qualquer RPC que você venha a ter.

Passe os seus três ids. O SPROUT volta com dono o programa Token-2022 e um blob base64 que você já sabe decodificar. O ativo do Almanac volta com dono um program id que começa com `CoRE`. O crate Harvest imprime `NO ACCOUNT`, e o RPC não está mentindo para você. Não tem nada lá. O crate existe como um hash dentro de uma árvore de Merkle e como uma linha no banco de dados de alguém.

É essa lacuna que esta lição fecha. O unificador é a Digital Asset Standard API, e no fim você vai ter o `read-any-asset.ts`, um script que resolve os três por um único método, classifica cada um pela interface dele, imprime um preço ao vivo para um fungível, e sinaliza quais extensões de um mint estão de fato fazendo alguma coisa em vez de apenas configuradas. O recuo da ajuda, dito sem rodeios: o transporte e o classificador são trabalhados por inteiro, o sinalizador de ativo-versus-dormente é seu com dois dos cinco casos mostrados, e a pegadinha dos creators vazios é inteiramente solo. O núcleo do classificador também é o challenge avaliado desta lição, então construa ele com cuidado.

## Uma superfície de leitura para cada formato

### O problema, enunciado uma vez

Três modelos de armazenamento, uma integração. Pegue eles na ordem de quanto do ativo vive na chain.

Um mint fungível é uma conta. Ele inteiro. Supply, decimals, autoridades, e toda extensão TLV que você anexou ficam em bytes que você mesmo pode buscar e decodificar, que é exatamente o que você fez no m01-l2 quando escreveu o seu próprio decodificador e conferiu ele contra o cliente publicado.

Um ativo Metaplex Core também é uma conta. Uma conta, campos base mais plugins, como você viu quando cunhou o Almanac. Programa dono diferente, mesma história de buscar-e-decodificar.

Um NFT comprimido não é uma conta. O programa Bubblegum faz o hash dos dados do ativo em uma folha, e a raiz da árvore é o que a chain de fato guarda. Verificar um crate significa reproduzir uma prova de Merkle contra essa raiz. Ler um crate significa perguntar a alguém que estava olhando quando a transação de mint aterrissou e que anotou o conteúdo da folha. Esse alguém é um indexador.

![Três pistas mostram o SPROUT e o ativo Core devolvendo bytes de conta de qualquer RPC enquanto o NFT comprimido não devolve conta nenhuma, e as três convergindo em uma única chamada getAsset do DAS.](assets/v01-flowchart.png)

### O que o DAS é de fato

A Digital Asset Standard API, spec 1.1.0, é uma extensão JSON-RPC que um provedor parafusa em um endpoint RPC normal da Solana. Mesma URL, mesmo formato de corpo do POST, nomes de método diferentes. A Metaplex publica a spec e o indexador de referência; os provedores rodam ele contra a própria infraestrutura.

Os métodos que você vai usar hoje são quatro:

- `getAsset`, um ativo por id, devolvendo metadados parseados, propriedade, royalty, status de compressão, e para fungíveis a token info.
- `getAssetsByOwner`, uma lista paginada de tudo que uma carteira tem, tipos misturados incluídos.
- `getAssetProof`, a prova de Merkle de um ativo comprimido, que você precisa para escritas e não para leituras.
- `searchAssets`, o mesmo índice consultado por critérios arbitrários (owner, collection, interface, frozen, burnt).

Tem mais (`getAssetsByGroup`, `getAssetsByAuthority`, `getAssetsByCreator`, `getSignaturesForAsset`, `getNftEditions`, `getTokenAccounts`, e as variantes em lote), e você vai encontrar dois deles no lab. Mas quatro cobrem a superfície de leitura de uma UI de carteira inteira, que é o ponto.

Aqui está o enquadramento honesto, e ele importa mais que a lista de métodos. O DAS não é a chain. O DAS é um **índice alugado**: um banco de dados que algum provedor preencheu olhando a chain, e a sua leitura é tão atual e tão completa quanto esse banco. Para o SPROUT e o Almanac você tem escolha, porque a conta está bem ali. Para o crate Harvest você não tem escolha nenhuma.

![Três camadas mostram a chain, um banco de dados de indexador rodado por um provedor, e o seu app chamando métodos DAS, com um caminho tracejado de getAccountInfo que contorna o índice apenas para contas.](assets/v02-diagram.png)

### O enum de interface, caminhado

Toda resposta do DAS começa com um campo `interface`, e esse campo é o roteador da sua integração inteira. O conjunto completo, verificado contra a documentação dos provedores em 2026-08-22:

| Interface | O que é |
|---|---|
| `V1_NFT` | não fungível do Token Metadata, o clássico |
| `V1_PRINT` | uma impressão de edição tirada de uma master edition |
| `LEGACY_NFT` | NFTs pré-padrão que o indexador ainda tem que servir |
| `V2_NFT` | o formato não fungível mais novo do Token Metadata |
| `ProgrammableNFT` | pNFT, imposição via Token Auth Rules |
| `MplCoreAsset` | um ativo Metaplex Core, uma conta, plugins dentro |
| `MplCoreCollection` | uma coleção Core |
| `MplCoreGroup` | um grupo Core sob o MIP-11, a Metaplex Improvement Proposal que adicionou agrupamento (você vai encontrar esses no mundo real, não neste curso) |
| `MplBubblegumV2` | um NFT comprimido Bubblegum v2 |
| `FungibleAsset` | um fungível com metadados anexados |
| `FungibleToken` | um fungível simples |
| `Custom`, `Identity`, `Executable` | válvulas de escape que você roteia para um fallback |

O seu ativo do Almanac chega como `MplCoreAsset`. O seu crate Harvest chega como `MplBubblegumV2`. O SPROUT chega como `FungibleToken` ou `FungibleAsset` dependendo das heurísticas de classificação do provedor; a divisão é muitas vezes descrita como ligada a metadados, mas na prática os provedores devolvem `FungibleToken` para fungíveis comuns mesmo quando um nome resolveu direitinho (a minha própria rodada imprimiu `FungibleToken` e um nome resolvido na mesma linha), então trate o par como uma única categoria fungível e nunca faça branch em qual dos dois você recebeu.

Duas coisas sobre esse enum valem uma pausa, porque as duas custam tempo das pessoas.

Primeiro, `MplCoreAsset` e `MplBubblegumV2` são adições recentes. A documentação do DAS v2 da Alchemy lista os dois explicitamente como o que a v2 adiciona sobre a v1, o que te diz que o enum cresceu depois que muito código de integração já tinha sido escrito contra o formato antigo. Se você herdar uma base de código cujo switch de ativo tem três casos, é por isso.

Segundo, e esta é a cilada: **`is_agent` não é uma variante da interface.** É um campo booleano anulável que pega carona nas linhas de `MplCoreAsset` quando o ativo carrega um plugin externo AgentIdentity, um plugin Core mais novo que marca um ativo como a identidade de um agente on-chain, e os provedores omitem o campo inteiro quando ele é falso. Dois campos irmãos viajam junto com ele, `asset_signer` (o endereço de assinatura do agente) e `agent_token` (o token associado dele); este curso nunca cunha um ativo de agente, mas o seu leitor vai encontrar eles em carteiras reais. Faça switch em `interface` para tipo. Leia `is_agent` como um atributo. Tratar isso como um tipo é o tipo de bug que funciona em todo teste que você escreve e quebra no primeiro ativo de agente de verdade que um usuário tiver.

![Quatro cards de categoria mapeiam valores de interface do DAS em nft, compressed-nft, fungible e other, com apenas compressed-nft exigindo um RPC DAS porque a flag de compressão decide.](assets/v03-comparison.png)

Repare no que a tabela diz sobre `compressed-nft`. O nome da interface te deixa perto, mas o campo que de fato decide é `compression.compressed`. Todo ativo DAS carrega um objeto `compression`, e em um NFT comum ele volta com `compressed: false` e strings de hash vazias. Faça o branch pelo booleano, não pelo nome, e o seu classificador sobrevive à próxima adição de enum sem uma edição. É esse o instinto de design inteiro por trás do challenge no fim desta lição.

### O DAS virou a API de preço sem alarde

Em algum ponto do caminho a API de ativos ganhou um segundo emprego. Chame `getAsset` com a opção de exibição `showFungible` em true e a resposta carrega um objeto `token_info`: decimals, supply, o programa de token dono, autoridades de mint e de congelamento, e, quando o mint se qualifica, `price_info`.

`price_info.price_per_token` é um número em USD ao vivo para um fungível, servido pelo mesmo endpoint que acabou de te falar de um cNFT. Sem segundo fornecedor, sem chave da CoinGecko, sem tabela de mapeamento de mint para coin id. Funciona para mints Token-2022 com extensões também, o que não é óbvio e vale você mesmo conferir.

Dois limites ficam bem ao lado dessa capacidade, e os dois estão na documentação dos provedores em vez de no blog post de alguém.

O preço fica **em cache por até 600 segundos**. Isso serve para uma linha de portfólio e está errado para qualquer coisa que liquide valor. Se você está precificando um swap, você quer um oráculo, e o curso planejado DeFi and RWA Engineering ensina a fazer isso direito.

O preço cobre mais ou menos os **dez mil tokens do topo por volume de 24 horas**. O SPROUT é um token de curso na devnet. Ele nunca vai estar nesse conjunto, e a maior parte do que os seus usuários têm também não. Um `price_info` faltando não é um erro e não é uma configuração errada do seu lado. Coloque null como default, renderize um traço, siga em frente. O seu leitor vai tratar isso como normal porque é assim que você vai escrever no lab.

### Provas, busca, e a paginação que você vai de fato escrever

Dois dos quatro métodos ainda não pagaram o próprio sustento, então deixe eu gastar eles direito, porque os dois vêm com um equívoco anexado.

`getAssetProof` parece pertencer à leitura. Não pertence. Ele devolve a prova de Merkle de um ativo comprimido: o id da árvore, a raiz atual, o hash da folha, o índice do nó, e um array de hashes irmãos cujo comprimento é a profundidade da árvore. Você precisa de tudo isso para *escrever*, porque uma transferência ou queima do Bubblegum tem que entregar ao programa hashes suficientes para recalcular a raiz e provar que a sua folha estava nela. Para exibir um crate em uma carteira, `getAsset` já te deu tudo, e buscar uma prova que você nunca usa é um ida e volta que você paga por nada.

```typescript
// overgrowth/proof-check.ts - a cNFT's proof is a write-time artifact.
// Run (from inside overgrowth/): npx tsx proof-check.ts <crate-asset-id>
import { das } from './das';

interface AssetProof {
  root: string;
  proof: string[];
  node_index: number;
  leaf: string;
  tree_id: string;
}

async function main(): Promise<void> {
  const id = process.argv[2];
  if (!id) throw new Error('pass a compressed asset id');

  const proof = await das<AssetProof>('getAssetProof', { id });
  console.log(`tree      ${proof.tree_id}`);
  console.log(`root      ${proof.root}`);
  console.log(`leaf      ${proof.leaf}`);
  console.log(`siblings  ${proof.proof.length} (tree depth)`);
  console.log(`node      ${proof.node_index}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Rode isso contra um crate Harvest e a contagem de irmãos é a profundidade da sua árvore, a mesma profundidade que você escolheu quando alocou a árvore na lição passada. E aqui está a parte que a Metaplex põe em negrito na própria documentação: uma prova fica obsoleta no instante em que qualquer outra pessoa modifica a árvore. Busque ela imediatamente antes da escrita, nunca no carregamento da página, nunca de um cache. Uma prova obsoleta não corrompe nada; ela só falha, e falha de um jeito que parece um erro misterioso de transação em vez de um problema de cache.

`searchAssets` é o outro, e é o método sobre o qual uma view de carteira de verdade é construída. Ele consulta o mesmo índice por critérios arbitrários: owner, collection, creator, authority, interface, frozen, burnt, supply, com ordenação e paginação. Uma chamada substitui os quatro idas e voltas separados por dono, por coleção e por criador que você senão costuraria junto.

```typescript
// overgrowth/search.ts - one query for a whole wallet view.
// Run (from inside overgrowth/): npx tsx search.ts <owner>
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';

interface Page<T> {
  total: number;
  limit: number;
  page: number;
  items: T[];
}

async function main(): Promise<void> {
  const owner = process.argv[2];
  if (!owner) throw new Error('pass an owner address');

  const counts = new Map<string, number>();
  for (let page = 1; ; page += 1) {
    const res = await das<Page<DasAsset>>('searchAssets', {
      ownerAddress: owner,
      burnt: false,
      page,
      limit: 1000,
      options: { showFungible: true },
    });
    for (const asset of res.items) {
      const key = classifyAsset(asset).category;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    if (res.items.length < res.limit) break;
  }
  for (const [category, n] of counts) console.log(`${category.padEnd(15)} ${n}`);
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Esse loop é o formato de paginação para internalizar. As páginas do DAS são indexadas a partir de um, o tamanho de página é limitado pelo provedor (1000 é o teto comum), e a condição de parada é uma página curta em vez de um total em que você confia. Eu já vi `total` ficar atrás dos itens em um índice movimentado, e um loop que confia nele ou para cedo ou fica girando. Uma página curta é um fato sobre a resposta que está na sua mão.

![Duas trilhas se ramificam a partir de um id de ativo comprimido: um caminho curto de leitura por getAsset até a renderização, e um caminho de escrita por getAssetProof cuja prova fica obsoleta a qualquer modificação da árvore.](assets/v04-flowchart.png)

### Configurado não é o mesmo que ativo

É aqui que um índice para de ser suficiente.

O DAS te entrega o formato de um token. Ele não te entrega o comportamento do token. Essas são perguntas diferentes, e confundir as duas produz integrações que exibem bobagem com confiança.

Trabalhe isso a partir dos primeiros princípios. Uma extensão tem dois estados que parecem idênticos em qualquer lista de "que extensões esse mint tem": presente e fazendo alguma coisa, versus presente e inerte. Um `TransferFeeConfig` com uma taxa de zero basis points e uma taxa máxima de zero está estruturalmente ali e economicamente ausente. Um `TransferHook` cujo program id é a pubkey default toda de zeros é um hook que não chama ninguém. Um `PausableConfig` que não está pausado é uma arma carregada com a trava de segurança acionada. Em cada caso a presença é uma permissão que o emissor tem, e o campo ao vivo é se ele já usou ela.

Por que essa distinção importa para você especificamente? Porque a presença diz ao seu usuário o que poderia acontecer e o valor ao vivo diz a ele o que vai acontecer na próxima transferência dele, e essas duas frases pertencem a partes diferentes da sua UI. "Este token pode ser pausado pelo emissor dele" é uma divulgação de risco. "Este token está pausado agora" é um estado de erro.

O exemplo canônico trabalhado é o PYUSD, e você já leu o mint dele uma vez neste curso. O mint Token-2022 dele carrega oito extensões TLV: mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer, e tokenMetadata. Configuradas mas dormentes, como o emissor as deixou: o program id do transfer hook é null e a taxa lê zero basis points com um máximo de zero. Eu reli esse mint em 2026-08-22 enquanto escrevia isto e achei exatamente essas oito, exatamente esses dois valores dormentes, que é o tipo de afirmação que você deveria rodar de novo em vez de aceitar de mim.

O detalhe mecânico que derruba as pessoas: **o DAS e a conta bruta discordam sobre como dizer "não definido."** Uma resposta do DAS anula o program id do transfer hook. O cliente Token-2022 gerado, decodificando os mesmos bytes, te dá o endereço do system program todo de zeros, porque é literalmente o que está na conta. Mesmo fato, duas representações. O seu sinalizador tem que saber para qual dos dois está olhando, e no lab você vai ler o mint bruto exatamente por essa razão: o sinalizador é uma leitura de chain, não uma leitura de índice.

![Painéis lado a lado comparam a visão do índice DAS, onde o program id de um transfer hook dormente é null, com a visão da conta decodificada, onde o mesmo campo é o endereço todo de zeros.](assets/v05-annotated-code.png)

### Escolher um provedor faz parte da leitura

Agora a parte que ninguém te conta até ela morder: com cNFTs, a sua escolha de RPC é uma decisão de correção, não de desempenho.

Aponte `getAsset` para um RPC comum e você não recebe uma resposta lenta nem uma resposta parcial. Você recebe `-32601 Method not found`, ou pior, um provedor que engole isso em um resultado vazio e deixa a sua UI renderizar uma carteira com três dos cinco ativos dela faltando. A própria documentação do Bubblegum da Metaplex diz de forma seca: nem todos os provedores de RPC suportam a DAS API, confira a página de provedores. O seu leitor deveria falhar alto nesse código de erro, e é por isso que o transporte que você escreve no passo 2 trata ele como caso especial.

O que nos traz a um pedaço de história recente que remodelou essa decisão inteira.

Durante anos a resposta default para "como eu leio NFTs entre chains" era SimpleHash. Era a maior API de NFT multi-chain do ramo, e aí a Phantom comprou ela e desligou a API pública em **27 de março de 2025**. Na Solana, a resposta que absorveu a lacuna foi o DAS, o que significa que a pergunta deixou de ser "qual API de NFT" e virou "qual provedor de DAS". Essa é uma pergunta genuinamente diferente: você está escolhendo um operador de índice, e operadores de índice diferem em atualidade, em completude, em como paginam, e em como nomeiam as coisas.

A lista atual que vale avaliar: **Helius**, **QuickNode**, **Alchemy** (cujo DAS v2 é uma coisa à parte, veja abaixo), **Triton**, e **Shyft**. Para a compressão ZK, que é uma história de compressão diferente da do Bubblegum, o índice é o **Photon**.

Eles não são intercambiáveis de encaixe direto, e a v2 da Alchemy é a ilustração mais limpa. Migrar para ela exige sufixar todo nome de método com `_v2` (`getAsset` vira `getAsset_v2`), renomear três métodos por completo, renomear o objeto de parâmetros que molda a resposta de `displayOptions` para `options`, mudar como você lê a resposta de lote de provas porque ela volta chaveada por id de ativo em vez de ordenada, e lidar com um campo novo `last_indexed_slot` em todo sucesso. Nada disso é absurdo. Tudo isso é trabalho que você não descobre até tentar trocar. Escreva o seu transporte de forma que o nome do método e o endpoint sejam as únicas coisas que uma troca toca, que é exatamente o que `das.ts` faz no lab.

![Uma linha do tempo vai do SimpleHash como a API de NFT default, passando pelo desligamento dele em março de 2025, até a lista de provedores de DAS de hoje mais o Photon para compressão ZK.](assets/v06-timeline.png)

Aquele marcador do meio merece uma frase própria. `solana-foundation/developer-content`, o repositório por trás dos cursos oficiais da Solana, foi arquivado em **2025-01-24**. Todo curso oficial, portanto, é anterior ao Bubblegum v2 e anterior aos valores de interface em que você está prestes a fazer switch. Se você vem conferindo este curso contra a documentação oficial e encontrando lacunas, é essa a lacuna, e ela é uma data em vez de uma conspiração.

Então como você escolhe um de verdade? Não por blog post de benchmark. Avalie pelos eixos que mudam o seu código ou os seus relatórios de incidente, e avalie eles contra os seus próprios ativos, no seu próprio cluster, nesta semana.

O provedor suporta DAS na rede em que você faz deploy, devnet incluída? Vários suportam só a mainnet, e descobrir isso na hora da integração com a devnet é uma tarde ruim. Qual é o atraso de indexação para um ativo comprimido recém-cunhado, medido por você, com um script de mint e um cronômetro? Ele serve `searchAssets` com os filtros que a sua UI precisa, ou só o subconjunto por dono e por coleção? Qual é o teto de página e o `total` se comporta? A superfície de métodos é a spec canônica, ou um dialeto como o sufixo `_v2` que te custa um shim? Como ele precifica chamadas DAS contra chamadas RPC comuns, dado que uma view de carteira é muitas leituras pequenas? E a que as pessoas pulam: o que acontece em um erro, um erro JSON-RPC alto ou um array vazio silencioso?

A última merece a sua paranoia. Um índice que responde "nenhum ativo" quando quer dizer "eu não implemento este método" vai passar em todo teste que você escrever e mentir para os seus usuários em produção. Teste isso de propósito: aponte o seu leitor para um RPC público comum e confirme que ele lança erro.

![Uma tabela lista sete eixos de seleção de provedor com o porquê de cada um mudar o seu código e um autoteste para ele, com rodapé da lista de provedores de DAS mais o Photon para compressão ZK.](assets/v07-table.png)

### O trade-off, nomeado

O DAS te dá uma superfície de leitura única entre quatro padrões de ativo, mais preços, ao preço de confiar no banco de dados de alguém em vez do estado da chain.

O que você abre mão, concretamente. A atualidade é do indexador, não da chain, então um mint de quatro segundos atrás pode ainda não estar lá. A completude também é do indexador, e reorgs e buracos de backfill são reais. Os dados de preço ficam em cache por dez minutos e cobrem uma fatia de topo dos tokens. E a coisa toda é uma camada de leitura: ela não faz stream, não faz backfill, e não é um pipeline de indexação.

Então quando você não deveria usar? Três casos, e são todos casos em que o índice é estritamente pior que a coisa que ele copia. Quando você está prestes a assinar uma transação cuja correção depende do estado atual, leia a conta: uma flag de congelado, um mint pausado, um delegado, um supply pelo qual você está prestes a dividir. Quando você precisa de um campo que o DAS não modela, leia a conta: as suas próprias entradas TLV, estado de programa customizado, qualquer coisa para a qual o indexador não tinha schema. E quando você acabou de escrever e quer confirmar, leia a conta, porque a sua própria transação está confirmada na chain antes de estar em qualquer banco de dados. A regra de bolso que sobrevive: o DAS responde "o que este usuário tem", a chain responde "o que é verdade agora". O seu leitor usou os dois hoje de propósito, DAS para os três ativos e um `fetchMint` direto para o estado das extensões, e essa divisão é o design, não um atalho.

![Um fluxo de decisão roteia leituras críticas para assinatura, não modeladas, e recém-escritas para a conta bruta enquanto toda outra leitura fica no DAS.](assets/v08-flowchart.png)

Aquela última cláusula sobre pipelines é uma fronteira real, não modéstia. Construir o pipeline (plugins do Geyser, gRPC do Yellowstone, ingestão por webhook, reproduzir histórico para dentro do seu próprio armazenamento) é uma disciplina séria e pertence ao curso planejado Client-Side Mastery, que trata o DAS como um índice alugado dentro de uma disciplina de dados muito maior. Esta lição é consumo. Você é o cliente de um índice, e o seu trabalho é ser um cliente bem-comportado: falhar alto num método ausente, usar null como default para preços ausentes, e nunca supor que o índice sabe algo que a chain não confirmou.

## Lab: construa o read-any-asset.ts

Um pré-requisito que senão vai te custar uma hora. **Nenhum indexador está olhando o seu surfnet.** A lição passada já moveu os crates para a devnet atrás de um endpoint DAS exatamente por essa razão; todo lab antes daquele rodou feliz contra um validador local, e nenhum deles consegue servir esta lição, porque o DAS é um índice e ninguém está indexando um cluster que existe só no seu laptop. Então o crate já está onde precisa estar, e o Almanac também: o `getOrCreateAlmanac` da lição passada criou uma coleção Overgrowth Almanac na devnet, registrou ela em `crates.json`, e cunhou os crates dentro dela. Essa coleção é o que "o Almanac" significa para o resto do curso. NÃO rode de novo o script de coleção do surfnet do m06-l2 contra a devnet; isso cunharia um segundo Almanac, diferente, e deixaria o seu crate fora dele.

O que a devnet ainda não tem são duas coisas, reserve quinze minutos. Primeiro, um ATIVO Core do Almanac para ler: cunhe um dentro da coleção devnet existente com uma pequena variante do `mint-almanac.ts` do m06-l2 que troca a conexão dele pelo helper `getUmi()` da lição passada (mesma `DAS_RPC_URL`, mesmo `wallet.json` com fundos) e lê o endereço da coleção de `crates.json` em vez de `almanac.json`. Segundo, o SPROUT: rode de novo `labs/m02-l4/add-metadata.ts` pelo próprio fallback de devnet documentado dele, `RPC_URL=https://api.devnet.solana.com WS_URL=wss://api.devnet.solana.com`, financiando o payer pelo faucet quando a chamada de airdrop bater no rate limit. Seja claro sobre o que isso recria, para que a saída do sinalizador não te confunda depois: o SPROUT de devnet desse script é o build de referência só de metadados, carregando exatamente MetadataPointer e TokenMetadata, sem config de taxa, sem extensão de exibição. O seu sinalizador vai imprimir duas linhas ACTIVE para ele e mais nada, e isso está correto; as linhas DORMANT de taxa e de hook nos exemplos desta lição vêm de apontar o bloco de estado de extensões para o PYUSD, que é o autoteste do passo 5. Os endereços de devnet vão ser diferentes dos do seu surfnet; tudo bem, o passo 1 registra os novos.

Três endpoints daqui em diante, e mantenha eles separados, porque não são intercambiáveis. `DAS_RPC_URL` é o endpoint de devnet com suporte a DAS que você exportou na lição passada, o índice. `RPC_URL` é um RPC de devnet comum, e é o que as leituras de conta bruta do passo 4 usam. E `MAINNET_DAS_RPC_URL` é o endpoint de MAINNET do mesmo provedor de DAS (a maioria dos provedores serve as duas redes com uma chave), usado exatamente uma vez, para a sondagem de preço, porque o conjunto precificado é uma fatia de mainnet, ranqueada por volume, que nenhum índice de devnet consegue responder:

```bash
export DAS_RPC_URL="https://<your-das-devnet-endpoint>"
export RPC_URL="https://api.devnet.solana.com"
export MAINNET_DAS_RPC_URL="https://<your-das-mainnet-endpoint>"
```

Rode todo comando deste lab de dentro de `overgrowth/`, a mesma convenção da lição passada, para que caminhos relativos como `assets.json` e `wallet.json` resolvam.

**1. Anote o que você tem.** O seu leitor pega as entradas dele de um arquivo pequeno para que lições posteriores e os seus próprios scripts possam compartilhar uma fonte da verdade. Uma nota de continuidade antes de você preencher: se você completou o challenge do m07-l1 como escrito, o crate Harvest comum foi transferido para um signer descartável, então a sua carteira do lab não tem mais ele. Use o id de ativo do crate de conquista soulbound no campo `crate` (ele não pode ter saído da sua carteira), ou cunhe um crate comum novo; qualquer um dos dois mantém os três ativos sob um só `owner`, que é o que o passo do `getAssetsByOwner` precisa. Crie `overgrowth/assets.json` com os quatro endereços:

```json
{
  "sprout": "<your SPROUT mint>",
  "almanac": "<an Almanac Core asset>",
  "crate": "<a Harvest crate asset id: the achievement crate, or a fresh mint (see note above)>",
  "owner": "<the wallet holding all three>"
}
```

**2. Um transporte, todo método.** Toda chamada DAS é o mesmo POST com uma string de método diferente, então escreva isso uma vez. O tratamento de erro é a parte interessante e a razão de isso não ser um one-liner:

```typescript
// overgrowth/das.ts - one JSON-RPC transport for every DAS method.

const DAS_RPC_URL = process.env.DAS_RPC_URL ?? '';

export interface DasError {
  code: number;
  message: string;
}

export async function das<T>(
  method: string,
  params: unknown,
  endpoint: string = DAS_RPC_URL,
): Promise<T> {
  if (!endpoint) {
    throw new Error('DAS endpoint is unset. Point DAS_RPC_URL at a DAS-supporting endpoint.');
  }
  const res = await fetch(endpoint, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: 'overgrowth', method, params }),
  });
  if (!res.ok) {
    throw new Error(`${method}: HTTP ${res.status} ${res.statusText}`);
  }
  const body = (await res.json()) as { result?: T; error?: DasError };
  if (body.error) {
    const hint =
      body.error.code === -32601
        ? ' (method not found: this endpoint does not implement DAS)'
        : '';
    throw new Error(`${method}: ${body.error.code} ${body.error.message}${hint}`);
  }
  if (body.result === undefined) {
    throw new Error(`${method}: empty result`);
  }
  return body.result;
}
```

O branch do `-32601` é o que salva a sua noite. O código padrão "method not found" do JSON-RPC é com o que um endpoint não-DAS responde, e sem essa dica você vai gastar vinte minutos desconfiando do seu id de ativo. O nome do método é um parâmetro e o endpoint é uma variável de ambiente, que é a sua superfície de migração inteira se você algum dia trocar de provedor.

**3. O classificador.** Este é o núcleo de roteamento, e também é o challenge avaliado, então leia isso como uma especificação sobre a qual você está prestes a ser testado em vez de código para colar:

```typescript
// overgrowth/classify.ts - route one DAS asset to a category, without a second RPC call.

export interface DasAsset {
  interface: string;
  compression?: { compressed?: boolean };
  token_info?: { price_info?: { price_per_token?: number } };
}

export interface AssetClassification {
  category: 'compressed-nft' | 'nft' | 'fungible' | 'other';
  compressed: boolean;
  fungible: boolean;
  pricePerToken: number | null;
  requiresDasRpc: boolean;
}

const NFT_INTERFACES = new Set<string>([
  'V1_NFT',
  'V1_PRINT',
  'V2_NFT',
  'LEGACY_NFT',
  'ProgrammableNFT',
  'MplCoreAsset',
  'MplBubblegumV2',
]);

const FUNGIBLE_INTERFACES = new Set<string>(['FungibleAsset', 'FungibleToken']);

export function classifyAsset(asset: DasAsset): AssetClassification {
  const compressed = asset.compression?.compressed === true;
  const fungible = FUNGIBLE_INTERFACES.has(asset.interface);
  const isNft = NFT_INTERFACES.has(asset.interface);
  const pricePerToken = asset.token_info?.price_info?.price_per_token ?? null;

  let category: AssetClassification['category'] = 'other';
  if (fungible) {
    category = 'fungible';
  } else if (isNft) {
    category = compressed ? 'compressed-nft' : 'nft';
  }

  return { category, compressed, fungible, pricePerToken, requiresDasRpc: compressed };
}
```

Três decisões em vinte linhas. `MplBubblegumV2` fica no conjunto de NFT em vez de ganhar o próprio branch, porque compressão é ortogonal a ser um NFT e o booleano já carrega isso. Interfaces desconhecidas caem em `other` em vez de lançar erro, porque um leitor que quebra em um valor de enum adicionado na terça passada é um leitor que entrega uma interrupção por release da Metaplex. E `requiresDasRpc` acompanha `compressed` em vez da interface, pela mesma razão: é uma afirmação sobre armazenamento, não sobre padrão.

**4. Resolva os três.** Agora o retorno, um método para três formatos:

```typescript
// overgrowth/read-any-asset.ts - one reader for every Overgrowth asset.
// Run (from inside overgrowth/): npx tsx read-any-asset.ts
import { readFileSync } from 'node:fs';
import { createSolanaRpc, address } from '@solana/kit';
import { fetchMint } from '@solana-program/token-2022';
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';
import { readExtensionState } from './extension-state';

// PYUSD: a MAINNET Token-2022 mint inside the priced set, used as the price
// probe. The probe must ask a mainnet DAS endpoint: a devnet index has never
// heard of this mint, and no devnet token has the 24h volume the priced set ranks by.
const PRICE_PROBE = '2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo';
const MAINNET_DAS = process.env.MAINNET_DAS_RPC_URL ?? '';

interface AssetBook {
  sprout: string;
  almanac: string;
  crate: string;
  owner: string;
}

interface FullAsset extends DasAsset {
  id: string;
  content?: { metadata?: { name?: string } };
  ownership?: { owner?: string };
  compression?: { compressed?: boolean; tree?: string; leaf_id?: number };
  token_info?: {
    decimals?: number;
    token_program?: string;
    price_info?: { price_per_token?: number; currency?: string };
  };
}

function loadBook(): AssetBook {
  try {
    return JSON.parse(readFileSync('assets.json', 'utf8')) as AssetBook;
  } catch {
    throw new Error(
      'assets.json not found. Write it with sprout, almanac, crate and owner addresses (and run from inside overgrowth/).',
    );
  }
}

function getAsset(id: string, showFungible = false): Promise<FullAsset> {
  return das<FullAsset>('getAsset', { id, options: { showFungible } });
}

function line(label: string, asset: FullAsset): string {
  const c = classifyAsset(asset);
  const name = asset.content?.metadata?.name ?? '(unnamed)';
  const price = c.pricePerToken === null ? 'no price_info' : `$${c.pricePerToken}`;
  return [
    `${label.padEnd(9)} ${asset.interface.padEnd(16)} ${c.category.padEnd(15)}`,
    `das-rpc=${c.requiresDasRpc}`,
    `price=${price}`,
    `name=${name}`,
  ].join('  ');
}

async function main(): Promise<void> {
  const book = loadBook();

  const sprout = await getAsset(book.sprout, true);
  const almanac = await getAsset(book.almanac);
  const crate = await getAsset(book.crate);
  console.log(line('SPROUT', sprout));
  console.log(line('ALMANAC', almanac));
  console.log(line('CRATE', crate));

  if (classifyAsset(crate).compressed) {
    console.log(`  crate leaf: tree=${crate.compression?.tree} leaf_id=${crate.compression?.leaf_id}`);
  }

  if (!MAINNET_DAS) {
    throw new Error('MAINNET_DAS_RPC_URL is unset; the price probe needs a mainnet DAS endpoint.');
  }
  const probe = await das<FullAsset>(
    'getAsset',
    { id: PRICE_PROBE, options: { showFungible: true } },
    MAINNET_DAS,
  );
  const probePrice = classifyAsset(probe).pricePerToken;
  console.log(`PROBE     price_per_token=${probePrice ?? 'null'} (mainnet read, cached up to ~600s)`);

  const owned = await das<{ total: number; items: FullAsset[] }>('getAssetsByOwner', {
    ownerAddress: book.owner,
    page: 1,
    limit: 50,
  });
  const almanacSeen = owned.items.some((item) => item.id === book.almanac);
  console.log(`OWNER     ${owned.total} assets, almanac present=${almanacSeen}`);

  const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
  const mint = await fetchMint(rpc, address(book.sprout));
  const extensions =
    mint.data.extensions.__option === 'Some' ? mint.data.extensions.value : [];
  for (const state of readExtensionState(extensions)) {
    console.log(`  ${state.active ? 'ACTIVE ' : 'DORMANT'} ${state.kind.padEnd(22)} ${state.detail}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Não rode ainda: o import `./extension-state` aponta para o arquivo que você escreve no passo 5, então a primeira rodada bem-sucedida pertence ao passo 7.

Leia o objeto `options` em `getAsset` antes de seguir, porque ele é uma emenda de portabilidade. A Helius documenta esse parâmetro como `options`. A v1 da Alchemy chamava ele de `displayOptions` e renomeou para `options` na v2. Wrappers de SDK mais antigos ainda emitem o nome antigo. Se um provedor ignora silenciosamente o seu `showFungible` e você não recebe `token_info` de volta, esse nome é a primeira coisa a conferir.

A linha do `PRICE_PROBE` precisa da própria palavra honesta, porque ela esconde outra cilada. O SPROUT é um token de curso na devnet, então ele vai resolver, classificar como fungível, e voltar **sem preço**, porque o conjunto precificado é mais ou menos os dez mil tokens do topo por volume de 24 horas. Esse é o caso normal, e a sua linha renderiza `no price_info` em vez de quebrar. A sonda apontada para o PYUSD está ali para você ver um `price_info` preenchido pelo menos uma vez com os seus próprios olhos, em um mint Token-2022 com extensões, a partir do endpoint de mainnet do seu provedor, porque o conjunto precificado é um fenômeno de mainnet e o mint em si só existe lá. Quando o SPROUT enfim for negociado em algum lugar com volume de verdade, a mesma chamada preenche e você não muda nada.

**5. O sinalizador, na maior parte seu.** Dois casos são trabalhados, três são o seu preenchimento:

```typescript
// overgrowth/extension-state.ts - configured is not the same as active.
import { AccountState, type Extension } from '@solana-program/token-2022';

const NULL_ADDRESS = '11111111111111111111111111111111';

export interface ExtensionState {
  kind: string;
  active: boolean;
  detail: string;
}

export function readExtensionState(extensions: readonly Extension[]): ExtensionState[] {
  return extensions.map((ext): ExtensionState => {
    switch (ext.__kind) {
      case 'TransferFeeConfig': {
        const bps = ext.newerTransferFee.transferFeeBasisPoints;
        const max = ext.newerTransferFee.maximumFee;
        return { kind: ext.__kind, active: bps > 0 && max > 0n, detail: `${bps} bps, max ${max}` };
      }
      case 'TransferHook':
        return {
          kind: ext.__kind,
          active: ext.programId !== NULL_ADDRESS,
          detail: `programId ${ext.programId}`,
        };
      // YOUR FILL: PausableConfig (ext.paused), DefaultAccountState
      // (ext.state === AccountState.Frozen), ScaledUiAmountConfig (ext.multiplier !== 1).
      default:
        return { kind: ext.__kind, active: true, detail: 'presence is the behavior' };
    }
  });
}
```

O branch default é uma escolha de design, não preguiça. Para `PermanentDelegate`, `MintCloseAuthority`, `MetadataPointer`, e companhia, a presença é o comportamento: não existe um segundo campo que os ligue, e reportar eles como ativos é a resposta honesta. Os seus três preenchimentos são os que têm um interruptor ao vivo. Aponte o script pronto para o mint do PYUSD em vez do SPROUT por um minuto e você deve ver oito extensões com `TransferFeeConfig` e `TransferHook` marcados `DORMANT`. Esse é um bom autoteste, porque é o mesmo resultado que eu obtive em 2026-08-22. Uma nota de honestidade sobre os seus três preenchimentos: nem o seu SPROUT de devnet nem o PYUSD carrega Pausable, DefaultAccountState ou ScaledUiAmountConfig, então nenhum mint ao vivo neste lab exercita eles. Prove eles do jeito barato: alimente o sinalizador com objetos de extensão construídos à mão em um teste de rascunho, vire `paused`, `state`, e `multiplier`, e veja os veredictos mudarem. Um preenchimento que só foi rodado contra extensões que nunca ocorrem é um preenchimento que você não provou.

**6. A pegadinha dos creators vazios, totalmente solo.** Vá de `getAssetsByCreator` em um mint da pump.fun e você vai receber zero resultados para um token que visivelmente existe e é negociado o dia inteiro. A tentação é culpar o índice, tentar de novo, ou esperar um cache passar. Tudo errado. **a pump.fun não preenche o array creators da Metaplex**, então uma consulta chaveada por criador não tem nada para casar. A correção do praticante é chavear pela autoridade de atualização em vez disso, ou assinar o programa diretamente.

Reproduza, depois conserte. Aqui está o diagnóstico; ligar o resultado ao seu leitor é com você:

```typescript
// overgrowth/creators-probe.ts - why a creator query comes back empty.
// Run (from inside overgrowth/): npx tsx creators-probe.ts <mint>
import { das } from './das';

interface Page {
  total: number;
  items: { id: string; interface: string }[];
}

async function main(): Promise<void> {
  const mint = process.argv[2];
  if (!mint) throw new Error('pass a mint address');

  const asset = await das<{
    creators?: { address: string; verified: boolean }[];
    authorities?: { address: string; scopes: string[] }[];
  }>('getAsset', { id: mint });

  const creators = asset.creators ?? [];
  const authority = asset.authorities?.[0]?.address;
  console.log(`creators: ${creators.length}`);
  console.log(`authority: ${authority ?? 'none'}`);

  if (creators[0]) {
    const byCreator = await das<Page>('getAssetsByCreator', {
      creatorAddress: creators[0].address,
      page: 1,
      limit: 10,
    });
    console.log(`getAssetsByCreator -> ${byCreator.total}`);
  }
  if (authority) {
    const byAuthority = await das<Page>('getAssetsByAuthority', {
      authorityAddress: authority,
      page: 1,
      limit: 10,
    });
    console.log(`getAssetsByAuthority -> ${byAuthority.total}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

Rode contra um mint da pump e `creators: 0` imprime antes de qualquer consulta rodar, que é o diagnóstico inteiro em uma linha. Rode contra o seu ativo do Almanac e o array creators está preenchido, porque você cunhou ele por um padrão que preenche o campo. Generalize a lição em vez do workaround: antes de construir uma feature sobre uma chave de consulta do DAS, confira que os ativos com que você se importa de fato preenchem essa chave. Creators, collections e authorities são todos opcionais na prática, o que quer que o schema sugira.

**7. Entregue.** `npx tsx read-any-asset.ts` deve agora imprimir três linhas de classificação, uma referência à folha do crate, um preço preenchido vindo da sonda, uma contagem de dono com o seu ativo do Almanac presente, e o bloco de estado das extensões. Formato de uma rodada que passa, com os seus próprios endereços e valores no lugar dos placeholders:

```text
SPROUT    FungibleToken    fungible         das-rpc=false  price=no price_info  name=SPROUT
ALMANAC   MplCoreAsset     nft              das-rpc=false  price=no price_info  name=Almanac Vol. 1
CRATE     MplBubblegumV2   compressed-nft   das-rpc=true   price=no price_info  name=Harvest Crate
  crate leaf: tree=<your tree> leaf_id=<n>
PROBE     price_per_token=<live number> (mainnet read, cached up to ~600s)
OWNER     <n> assets, almanac present=true
  ACTIVE  MetadataPointer        presence is the behavior
  ACTIVE  TokenMetadata          presence is the behavior
```

(Essas duas linhas ACTIVE são o bloco inteiro para o SPROUT de devnet só de metadados. As linhas `DORMANT TransferFeeConfig 0 bps, max 0` e `DORMANT TransferHook` aparecem quando você mira o mesmo bloco no PYUSD, o autoteste do passo 5.)

Leia esse bloco como um conjunto de afirmações em vez de decoração. A coluna `das-rpc` é true exatamente uma vez. A coluna de categoria tem três valores diferentes. A linha da sonda tem um número nela. Se qualquer uma dessas três afirmações for falsa, o critério não foi atendido, seja lá com o que o script sair.

![A saída que passa é anotada linha a linha: três categorias, uma coluna das-rpc verdadeira só para o NFT comprimido, um preço ao vivo da sonda, e o estado das extensões a partir do mint bruto.](assets/v09-annotated-code.png)

Ligue `search.ts` no mesmo workspace enquanto você está aqui. Ele não faz parte do critério, mas uma contagem por categoria sobre uma carteira inteira é a consulta com que uma integração de verdade abre, e rodar ela contra o seu próprio endereço de dono é o jeito mais rápido de ver se a paginação do seu provedor se comporta do jeito que o loop assume.

![Um diagrama de componentes mostra três artefatos anteriores alimentando o leitor de ativos, cujos módulos emitem classificações, um preço, e um relatório de extensões, com streaming e backfill marcados fora da fronteira.](assets/v10-diagram.png)

## Challenge

`classify-das-asset`. A lógica é exatamente o passo 3, mas a convenção de chamada do grader é mais achatada que o seu módulo de workspace: ele invoca `classifyAsset(iface, detailsJson)`, onde `iface` é a string `interface` do DAS e `detailsJson` é o resto da resposta do `getAsset` serializado como uma string JSON. Dê `JSON.parse` nessa string logo de cara, depois roteie exatamente como acima; as mesmas três decisões, o mesmo `AssetClassification` de saída. Faça os casos que falham do starter passarem.

Os testes batem nos cantos que importam em produção em vez do caminho feliz. Um ativo `MplBubblegumV2` com `compression.compressed` true tem que voltar como `compressed-nft` com `requiresDasRpc` true. Um `MplCoreAsset` tem que voltar como `nft` com `requiresDasRpc` false, porque um ativo Core é uma conta comum e nenhum índice é exigido para ler ele. Um `FungibleToken` carregando `token_info.price_info.price_per_token` tem que trazer esse número à tona, e um fungível sem price info tem que devolver null em vez de zero, undefined, ou um erro lançado. Zero é um preço. Null é uma ausência. Renderizar uma ausência como preço é como uma UI de portfólio diz a um usuário que o que ele tem não vale nada.

As dicas do challenge te dão os conjuntos de interfaces, a checagem de compressão, e o caminho do preço. Se você escreveu o passo 3 você mesmo, já resolveu; se você colou o passo 3, escreva de novo a partir da assinatura de tipo e veja se as três decisões voltam para você.

## Checkpoint

O critério: `npx tsx read-any-asset.ts` resolve e classifica corretamente os três ativos, e imprime um `price_per_token`. Três linhas, três categorias, um número. Diga a resposta em voz alta antes de seguir, porque é a coisa que esta lição existe para instalar: `FungibleToken` ou `FungibleAsset` para o SPROUT, `MplCoreAsset` para o Almanac, `MplBubblegumV2` para o crate Harvest, e só o último precisou de um endpoint com suporte a DAS para sequer existir.

Os erros que eu espero, na ordem em que acontecem. Se toda chamada morre com `-32601 (method not found)`, o seu `DAS_RPC_URL` é um RPC comum e nenhuma quantidade de retentativas vai mudar isso; consiga um endpoint DAS. Se o crate devolve um id de ativo mas os campos estão vazios, o indexador não alcançou um mint que você mandou segundos atrás, que é o trade-off de atualidade chegando em pessoa: espere, depois rode de novo. Se `token_info` está faltando por inteiro no SPROUT, você deixou cair o `showFungible`, ou o seu provedor quer essa flag sob `displayOptions`. E se `getAssetsByOwner` devolve o seu ativo do Almanac mas não o seu crate, confira o campo de dono no crate em vez da consulta, porque o dono de um NFT comprimido é um atributo da folha e transferir um reescreve a folha. O jeito usual de os alunos aterrissarem aqui é exatamente o challenge do m07-l1: o crate comum foi transferido para um signer descartável lá, que é por que o passo 1 te disse para registrar o crate de conquista ou um mint novo em vez dele.

O primeiro desses erros imprime exatamente assim, direto do caminho de erro do `das.ts` que você escreveu no passo 2:

```text
getAsset: -32601 Method not found (method not found: this endpoint does not implement DAS)
```

Um entregável que não é código, e eu digo isso literalmente: anote qual provedor você usou e qual eixo decidiu. Uma frase no seu README basta. "Escolhi X porque ele serve DAS na devnet e indexou um crate novo em menos de N segundos, medido nesta data." Daqui a seis meses, quando um incidente te fizer reconsiderar, essa frase é a diferença entre rodar de novo um teste e rodar de novo a avaliação inteira. Ela também te força a ter de fato medido alguma coisa em vez de escolher o provedor cuja página de documentação carregou primeiro, que é, honestamente, como a maior parte dessas decisões é tomada. Eu já tomei assim. A medição leva vinte minutos e é a única parte da escolha de provedor que é sua em vez do marketing.

Um hábito para levar daqui, que vale mais que o script: trate todo campo do DAS como opcional até você ter visto ele se preencher para os ativos com que você realmente se importa. `price_info` em um token pequeno, `creators` em um mint da pump, `is_agent` em qualquer coisa que não seja um ativo Core. O schema é uma promessa sobre formato. Só uma leitura ao vivo é uma promessa sobre conteúdo.

Agora você consegue ler todo ativo que construiu neste curso por uma única chamada, classificar ele sem uma segunda consulta, e dizer a um usuário a diferença entre um token que pode ser pausado e um token que está pausado. Cada um desses ativos, porém, é ou uma conta real ou uma folha na árvore de alguém. A próxima lição pressiona esse ponto com uma pergunta mais afiada: e se um token fungível em si não tivesse conta nenhuma, o que isso significaria para saldos e transferências, e quando você de fato ia querer isso? Traga o sinalizador. Você vai precisar do hábito de perguntar o que está realmente lá.
