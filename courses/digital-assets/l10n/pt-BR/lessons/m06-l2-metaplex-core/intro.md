# Entregue o NFT: coleções, plugins e edições do Metaplex Core

## Resumo

No m06-l1 você mapeou as camadas de metadados: o JSON off-chain, a struct Data on-chain da Metaplex e o TLV nativo do Token-2022 que você escreveu no SPROUT lá no m02-l4. Você validou o JSON de um ativo real contra o padrão e agora sabe exatamente onde cada campo vive. O que você não fez foi cunhar um NFT. Hoje isso muda, e a gente pula o caminho legado inteiro.

Aqui está o argumento em dois números, os dois tirados da própria tabela comparativa da Metaplex (metaplex.com/docs/smart-contracts/core, lida em 2026-09-07), então trate-os como publicados pelo fornecedor, não como algo que este curso mediu. Cunhar um NFT pelo Token Metadata custa por volta de ~0.022 SOL e ~205,000 compute units, porque o mint se espalha por uma conta de mint, uma conta de token, um PDA de metadados e um PDA de master edition. Cunhar o mesmo ativo pelo Metaplex Core custa ~0.003 SOL e ~17,000 CU. Uma conta. Uns 86% mais barato, e faça você mesmo a divisão desses dois números em SOL em vez de aceitar a porcentagem na fé. Cite o til deles também: eles publicam aproximações, e um curso que endurecesse ~0.003 num 0.0029 de aparência precisa estaria inventando uma precisão que o fornecedor nunca alegou. E o Core é o padrão que a Metaplex agora recomenda para trabalho novo, que é o motivo pelo qual esta lição nunca te pede para criar um NFT do Token Metadata.

Prova antes da teoria. Com o surfnet que você roda desde o m02-l1 no ar (`surfpool start --no-tui --no-studio` num terminal separado), instale o SDK e rode o mint de rascunho:

```bash
mkdir -p labs/m06-l2 && cd labs/m06-l2
npm install @metaplex-foundation/mpl-core@1.10.0 @metaplex-foundation/umi@1.5.1 @metaplex-foundation/umi-bundle-defaults@1.5.1
```

Pins conferidos contra npm e crates.io em 2026-08-23: a mais recente do SDK JS é a 1.10.0. O Core então versiona em dois números adicionais que é fácil confundir um com o outro. O programa on-chain está com a tag `release/core@0.15.1` (2026-06-18), enquanto o crate cliente em Rust publicado no crates.io é o `mpl-core` 0.12.1 (2026-06-16). Três linhas de release, três números, e o único que você instala aqui é o SDK JS na 1.10.0; a tag do programa e o crate Rust são contexto para ler changelogs, não dependências deste lab. Reconfira os três antes de fixar em qualquer coisa de vida longa.

```typescript
// labs/m06-l2/first-mint.ts
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { mplCore, createCollection, create, fetchCollection } from "@metaplex-foundation/mpl-core";
import { generateSigner, keypairIdentity, sol } from "@metaplex-foundation/umi";

async function main() {
  const umi = createUmi(process.env.RPC_URL ?? "http://127.0.0.1:8899").use(mplCore());
  umi.use(keypairIdentity(umi.eddsa.generateKeypair()));
  await umi.rpc.airdrop(umi.identity.publicKey, sol(2));

  const collectionSigner = generateSigner(umi);
  await createCollection(umi, {
    collection: collectionSigner,
    name: "Scratch Collection",
    uri: "https://overgrowth.example/scratch.json",
  }).sendAndConfirm(umi);

  const collection = await fetchCollection(umi, collectionSigner.publicKey);
  const assetSigner = generateSigner(umi);
  await create(umi, {
    asset: assetSigner,
    collection,
    name: "Scratch Asset",
    uri: "https://overgrowth.example/scratch-asset.json",
  }).sendAndConfirm(umi);

  const raw = await umi.rpc.getAccount(assetSigner.publicKey);
  if (raw.exists) {
    console.log("asset account:", assetSigner.publicKey);
    console.log("bytes:", raw.data.length);
    console.log("rent:", Number(raw.lamports.basisPoints) / 1e9, "SOL");
    console.log("owner:", raw.owner);
  }
}

main().catch((err) => { console.error(err); process.exit(1); });
```

`npx tsx first-mint.ts` imprime um endereço, um comprimento de dados um pouco acima de 100 bytes (o seu muda com os comprimentos do nome e da URI, já que os dois ficam guardados inline), um depósito de rent nos milésimos baixos de um SOL, e um owner começando com `CoRE`. É esse o NFT inteiro. Uma coleção, um ativo cunhado dentro dela, pertencimento já definido, uma conta guardando tudo. Sem ATA, sem PDA de metadados, sem PDA de edição.

Esta lição constrói o R7, `almanac-assets`: os NFTs do Almanac da Overgrowth como uma coleção Core verificada, ativos com um plugin Royalties, uma impressão de edição numerada, e um badge Founding-Farmer que nunca pode sair da carteira dele. O recuo da ajuda, em voz alta: a criação da coleção e o primeiro mint de ativo são trabalhados por inteiro; você mesmo preenche a config do plugin Royalties e a asserção de pertencimento; e o badge soulbound, a prova de que a transferência falha e o coding challenge do validador de royalties são totalmente solo.

## Uma conta por ativo

### Por que a contagem de contas é a história toda

Volte ao seu trabalho no SPROUT. Toda capacidade que você adicionou àquele mint vivia na mesma conta, anexada como entradas TLV que o seu próprio decodificador conseguia caminhar. O Token Metadata é a arquitetura oposta: capacidade por acréscimo de contas. O mint é um mint SPL, o nome vive num PDA de metadados que pertence a um programa diferente, a condição de edição vive em outro PDA, e a imposição programável (o caminho do pNFT) arrasta ainda mais. Cada conta custa rent, cada uma custa CU para criar, e cada uma é uma coisa a mais que todo leitor downstream tem que derivar, buscar e desserializar.

A resposta de design do Core é a que você já conhece do Token-2022, aplicada a NFTs: uma conta, com capacidades tipadas anexadas dentro dela. O ativo base guarda o dono, a autoridade de atualização, um nome e uma URI. Todo o resto, royalties, comportamento de congelamento, numeração de edição, atributos, é um plugin serializado depois dos dados base naquela mesma conta. Versão a partir dos primeiros princípios: o estado de um NFT é pequeno e as capacidades dele são enumeráveis, então pagar overhead de criação de conta por capacidade é desperdício puro; a única coisa que contas separadas te compram é propriedade independente, e os plugins de um ativo pertencem todos ao ativo de qualquer jeito.

![Comparação de uma cunhagem de NFT, o Token Metadata criando quatro a cinco contas a ~0.022 SOL e ~205,000 CU publicados pelo fornecedor, enquanto o Core cria uma conta a ~0.003 SOL e ~17,000 CU.](assets/v01-comparison.png)

A sua própria rodada do `first-mint.ts` acabou de te mostrar que o componente de rent varia com os comprimentos das strings, que é exatamente o motivo pelo qual a abertura marcou os números de manchete como publicados pelo fornecedor: cite o fornecedor, mostre a sua própria conta. Ainda é uma grande história.

### O que vive dentro de um ativo Core

O ativo base é deliberadamente minúsculo. Um discriminador de um byte (o Core usa o próprio enum de tipo de conta dele, não o hash de 8 bytes do Anchor), a pubkey do dono, um enum de autoridade de atualização, o nome, a URI. O enum de autoridade de atualização é o campo estrutural desta lição: ele não é sempre um endereço. Ele tem três braços, `None`, `Address` e `Collection`, e quando um ativo é cunhado dentro de uma coleção o braço é `Collection` com o endereço da conta da coleção dentro dele.

Leia isso de novo, porque isso silenciosamente apaga um ritual inteiro do Token Metadata. No Token Metadata, o pertencimento a uma coleção é um campo no PDA de metadados mais um booleano `verified` separado que uma instrução assinada pela autoridade da coleção vira; pertencimento não verificado é um estado intermediário real (e perigoso): um ativo pode ALEGAR uma coleção antes de qualquer autoridade de coleção ter atestado isso, a mesma lacuna entre alegação e atestação que você encontrou no m06-l1 no flag `verified` dos creators, agora no nível da coleção. No Core não existe booleano. O pertencimento É o braço de autoridade de atualização, e ele só pode ser escrito quando a autoridade da coleção assina o mint. A verificação não ficou mais fácil; ela foi colapsada numa assinatura que tem que estar ali de qualquer jeito.

![Layout de uma única conta de ativo Core, campos base incluindo o enum updateAuthority de três braços, depois um registro de plugins guardando entradas Royalties, Edition, PermanentFreezeDelegate e Attributes na mesma conta.](assets/v02-diagram.png)

### As coleções vêm primeiro

A consequência de ordenação cai direto desse design: a coleção tem que existir antes que qualquer ativo possa ser cunhado dentro dela, porque a instrução de mint precisa da conta da coleção para referenciar e da assinatura da autoridade da coleção para autorizar a escrita de pertencimento. Cunhe os ativos primeiro e você tem órfãos com autoridades de atualização `Address`. Existe um caminho de update para mover um ativo para dentro de uma coleção depois, mas é um retrofit que precisa de assinaturas de autoridade dos dois lados, e tudo que está downstream de você, indexadores, marketplaces, o trabalho de compressão do módulo 7, é construído em volta do fluxo coleção-primeiro.

Essa ordenação não é uma preferência de estilo neste curso; é infraestrutura estrutural. O Bubblegum v2, a abertura do módulo 7, cunha NFTs comprimidos DENTRO de uma coleção Core. Sem coleção Core, sem drop de cNFT. A coleção Almanac que você cria no lab é o valor literal que as chamadas de mint daquela lição recebem, reconstruída em qualquer cluster contra o qual ela rode. Pegue o hábito agora, enquanto o modo de falha é barato.

![Fluxograma contrastando o fluxo coleção-primeiro, onde o pertencimento é escrito no mint e alimenta o Bubblegum v2 no módulo 7, com o fluxo mint-primeiro, que produz ativos órfãos e leituras de pertencimento vazias.](assets/v03-flowchart.png)

### O catálogo de plugins

Os plugins são onde o Core para de ser um mint mais barato e se torna um modelo de programação diferente. Cada plugin é uma struct tipada com a própria autoridade dele, anexada na hora do create ou depois, num ativo ou numa coleção (um plugin no nível da coleção se aplica a todo membro, a não ser que o membro o sobrescreva). O catálogo que você vai realmente usar:

| Plugin | O que ele faz | O uso no Almanac |
|---|---|---|
| Royalties | `basisPoints` + as fatias de `creators` + um `ruleSet` | 5% em toda venda de Almanac |
| TransferDelegate | um delegado pode transferir o ativo | fluxos de escrow e marketplace |
| FreezeDelegate | um delegado pode congelar/descongelar (reversível) | imobilizações leves no estilo staking |
| BurnDelegate | um delegado pode queimar | consumo de item de jogo |
| UpdateDelegate | um delegado pode atualizar metadados | coleções gerenciadas |
| PermanentFreezeDelegate | congelamento que pode ser tornado irrevogável | o badge Founding-Farmer soulbound |
| Attributes | pares chave/valor on-chain | traits que um programa pode ler sem buscar a URI |
| Edition / MasterEdition | impressões numeradas + teto de supply | a tiragem do Almanac |

### Todo plugin carrega a própria chave

Antes de você digitar qualquer um desses, um fato estrutural que te economiza uma tarde de reverts confusos: a autoridade de um plugin não é a autoridade do ativo. Cada entrada de plugin no registro carrega o próprio campo de autoridade dela, e esse campo é um enum com quatro braços: `Owner`, `UpdateAuthority`, `Address` (qualquer pubkey que você nomear) e `None`. Quem estiver sentado nesse braço controla o plugin, independente de quem é dono do ativo e de quem pode atualizar os metadados dele. Três chaves, três trabalhos. O dono move o ativo, a autoridade de atualização o renomeia, a autoridade do plugin opera o plugin.

O catálogo se divide nessa linha. Os plugins gerenciados pelo dono, os delegados de Transfer, Freeze e Burn, existem para deixar o DONO emprestar uma capacidade: você delega direitos de transferência para um escrow, direitos de congelamento para um programa de staking, e a delegação fica por padrão na sua própria chave até você atribuí-la a outro. Os plugins gerenciados pela autoridade, Royalties, Attributes, UpdateDelegate, pertencem ao lado do criador e ficam por padrão na autoridade de atualização, que para um membro de coleção resolve através da coleção. E a família permanente joga com uma regra mais dura: `PermanentFreezeDelegate`, `PermanentTransferDelegate` e `PermanentBurnDelegate` só podem ser anexados na hora do mint. Você não consegue enfiar um congelamento permanente num ativo que alguém já possui, que é exatamente a propriedade que faz ser dono de um ativo Core seguro, e exatamente o motivo pelo qual o badge Founding-Farmer tem que nascer soulbound em vez de ser convertido depois.

![Diagrama separando as três chaves de controle de um ativo Core, o dono, a autoridade de atualização e o enum de autoridade de quatro braços de cada plugin, com as classes de plugin agrupadas como gerenciados pelo dono, gerenciados pela autoridade e plugins permanentes só na hora do mint.](assets/v04-diagram.png)

Segure o braço `None`. Na maior parte do tempo você atribui uma autoridade de plugin para que alguém possa agir. Definir ela como `None` é a jogada inversa, e ela morde: solda o estado atual do plugin no lugar, permanentemente, porque não existe chave que pudesse mudá-lo algum dia. Um `PermanentFreezeDelegate` com `frozen: true` e autoridade `None` não é "congelado até alguém importante dizer o contrário". É congelado do jeito que um número é par.

Três entradas do catálogo merecem um olhar mais de perto antes de você digitá-las.

**Royalties** carrega três campos e o programa impõe o formato deles no mint. `basisPoints` é um inteiro 0..10000 (500 significa 5%). `creators` é uma lista de entradas de endereço mais porcentagem cujas porcentagens têm que somar exatamente 100, e um endereço de creator duplicado é rejeitado. `ruleSet` decide quem pode mover o ativo: `None` não coloca nenhuma restrição de programa nas transferências, `ProgramAllowList` permite que só programas listados estejam envolvidos, `ProgramDenyList` bloqueia os programas listados. Note o que `None` significa para a palavra "royalty": o split fica registrado on-chain, legível por todo mundo, imposto por ninguém em particular. Se alguém de fato paga é uma decisão de marketplace, e essa frase incômoda é o assunto inteiro do m06-l3. Eu já errei o dedo num split de creator antes, 60/50 entre duas carteiras porque editei um lado e não o outro, e o mint reverte na hora. Bom. Melhor um revert no mint do que um marketplace dividindo 110%.

![Config anotada do plugin Royalties mostrando basisPoints limitado de 0 a 10000, porcentagens de creator que têm que somar exatamente 100 sem endereços duplicados, e as três variantes de ruleSet.](assets/v05-annotated-code.png)

**PermanentFreezeDelegate** é o irmão de mão única do FreezeDelegate reversível. Anexe-o com `frozen: true` e uma autoridade de `None` e você tem um ativo que nenhuma chave na terra consegue descongelar ou mover. Isso não é um bug para contornar; é o mecanismo soulbound. Um badge de associação, uma credencial, um comprovante de presença: coisas que deveriam ser sem sentido para vender são exatamente as coisas que você congela permanentemente. O outro lado da moeda é a cilada de que o nome está te avisando. Permanente significa permanente. Não existe voto de governança depois, nem ticket de suporte, nem autoridade que consiga descongelar o badge Founding-Farmer depois que você o cunha desse jeito. Se um farmer perde a carteira, ele precisa de um badge novo, não de uma transferência. Recorra ao FreezeDelegate reversível sempre que você conseguir imaginar uma movimentação futura legítima.

**Edition e MasterEdition** dividem um trabalho entre os dois tipos de conta. A coleção carrega o `MasterEdition` com um `maxSupply` e overrides opcionais de nome/URI; cada ativo impresso carrega o `Edition` com o `number` dele. As leituras compõem exatamente do jeito que você esperaria: busque a coleção para pegar o teto, busque qualquer impressão para pegar o número dela.

![Diagrama de uma tiragem onde a coleção guarda um plugin MasterEdition com maxSupply 100 e cada ativo membro carrega um plugin Edition com o próprio número de impressão dele.](assets/v06-diagram.png)

### Delegados, e o que uma transferência apaga

Os três delegados gerenciados pelo dono são o que você usa quando um ativo tem que participar de algo continuando sendo do dono dele. Anexe o `TransferDelegate` com o endereço de um programa de escrow e aquele programa pode mover um Almanac para fora de uma carteira quando uma venda liquida, sem nunca custodiá-lo primeiro. Anexe o `FreezeDelegate` com `frozen: true` e um programa de staking como autoridade e o ativo se imobiliza no lugar: ainda na carteira do dono, ainda visível em toda UI, simplesmente inamovível até o programa o descongelar. É assim que o staking no Core funciona, e é por isso que o staking no Core não precisa de conta de vault. O ativo nunca vai a lugar nenhum; ele só para de conseguir ir. O `BurnDelegate` é o caso de crafting. Um farmer alimenta o composter da Overgrowth com dois volumes do Almanac e o programa queima os dois sob uma delegação concedida antes, sem prompt de assinatura na hora da queima.

A API é `addPlugin` para anexar um, e `approvePluginAuthority` para entregar a chave a um endereço de programa depois. O trecho abaixo é ilustrativo, não roda como está colado: os dois endereços são PLACEHOLDERS que você tem que substituir (o primeiro é literalmente o endereço do System Program, que é no que uma string base58 só de uns decodifica; o segundo é um endereço válido arbitrário fazendo as vezes do seu programa de escrow), e o approve só funciona num plugin que existe, então uma chamada `addPlugin(umi, { asset, plugin: { type: "TransferDelegate" } })` o precede em qualquer fluxo real:

```typescript
// labs/m06-l2/delegate-transfer.ts (illustrative: replace BOTH placeholder
// addresses, and addPlugin the TransferDelegate first or this approve fails)
import { publicKey } from "@metaplex-foundation/umi";
import { approvePluginAuthority } from "@metaplex-foundation/mpl-core";
import { getUmi } from "./umi";

async function main() {
  const umi = await getUmi();
  await approvePluginAuthority(umi, {
    asset: publicKey("11111111111111111111111111111111"),      // PLACEHOLDER: your Almanac asset
    plugin: { type: "TransferDelegate" },
    newAuthority: {
      type: "Address",
      address: publicKey("8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR"),  // PLACEHOLDER: the escrow program
    },
  }).sendAndConfirm(umi);
}

main().catch((err) => { console.error(err); process.exit(1); });
```

Agora a regra que discretamente decide a sua arquitetura. Quando um ativo é transferido, os plugins gerenciados pelo dono têm a autoridade deles automaticamente revogada de volta para `Owner`. Os plugins gerenciados pela autoridade e a família permanente sobrevivem à transferência intocados.

Essa única frase tem três consequências que valem ser guardadas separadamente. Um comprador nunca herda os direitos de escrow do vendedor nem os direitos de congelamento do programa de staking do vendedor, que é a propriedade que faz comprar um ativo Core carregado de plugins ser seguro afinal: o que o dono anterior delegou evapora no momento em que o ativo muda de mãos. O seu plugin Royalties, sendo gerenciado pela autoridade, pega a carona para sempre, que é precisamente o que um royalty tem que fazer para significar algo através de uma revenda. E a família permanente pega a carona também, que é o motivo mais profundo de ela só poder ser anexada no mint. As propriedades irreversíveis de um ativo têm que ser conhecíveis por um comprador antes de ele comprar, e a anexação só na hora do mint é o que garante isso: ninguém consegue soldar o seu ativo fechado depois que você é dono dele.

A versão a partir dos primeiros princípios, se você quer que a regra seja memorável em vez de memorizada: uma delegação é uma declaração sobre a intenção do dono atual, então ela não deve sobreviver a esse dono. Um royalty é uma declaração sobre os termos do criador, então ele deve. O Core codifica a diferença na classe do plugin em vez de pedir que todo integrador lembre qual é qual.

![Tabela das classes de plugin do Core, plugins gerenciados pelo dono cuja autoridade se auto-revoga na transferência, plugins gerenciados pela autoridade como o Royalties que persistem, e a família permanente que se anexa só no mint.](assets/v07-table.png)

### Attributes: traits que um programa on-chain consegue de fato ler

No m06-l1 você aprendeu o array `attributes` do JSON off-chain, os traits que os marketplaces renderizam numa barra lateral. Um programa Solana não consegue ler esse array. Ele vive atrás de uma URI HTTP, invisível para o runtime, então qualquer lógica on-chain que queira saber a estação de um Almanac tem que ser informada por um signer confiável. Isso é um oráculo, e agora você está rodando um.

O plugin `Attributes` é a resposta on-chain. Ele guarda um `attributeList` de pares de strings chave/valor dentro da própria conta do ativo, onde um programa o desserializa de uma conta que ele já tem carregada:

```typescript
// labs/m06-l2/tag-season.ts
import { publicKey } from "@metaplex-foundation/umi";
import { addPlugin, fetchAsset } from "@metaplex-foundation/mpl-core";
import { getUmi } from "./umi";

async function main() {
  const umi = await getUmi();
  const asset = publicKey("11111111111111111111111111111111");  // PLACEHOLDER: your Almanac asset address

  await addPlugin(umi, {
    asset,
    plugin: {
      type: "Attributes",
      attributeList: [
        { key: "season", value: "spring-2026" },
        { key: "yield", value: "3" },
      ],
    },
  }).sendAndConfirm(umi);

  const fetched = await fetchAsset(umi, asset);
  console.log(fetched.attributes?.attributeList);
}

main().catch((err) => { console.error(err); process.exit(1); });
```

Ele é gerenciado pela autoridade, então para um membro de coleção é a autoridade da coleção que assina as atualizações, não o holder. Esse é o default correto para estado de jogo: um farmer não deveria conseguir reescrever o yield do próprio Almanac dele entre harvests. Duas restrições para projetar em volta. Chaves e valores são os dois strings, então números são convertidos em string na entrada e parseados na saída, e nada valida o parse além de você. E todo atributo é bytes numa conta pela qual você paga rent, então esse é um lugar para o punhado de traits sobre os quais os seus programas ramificam, não um banco de dados. A lista completa de traits voltada para marketplaces fica no JSON onde os marketplaces já procuram por ela.

### Asset ou coleção: onde um plugin deve morar

Todo tipo de plugin se anexa a qualquer um dos dois tipos de conta, e a regra de resolução é a parte com dinheiro dentro: um plugin na coleção se aplica a todo membro, e um plugin no membro tem precedência sobre a versão daquele mesmo plugin na coleção. O SDK já vem com `deriveAssetPlugins(asset, collection)` para você computar o conjunto efetivo numa chamada só em vez de checar as duas contas e reimplementar a precedência você mesmo.

A economia cai direto disso. Um drop de 10,000 peças que anexa o Royalties em todo ativo paga pelos bytes daquele plugin dez mil vezes. Anexe-o na coleção uma vez e todo membro o herda, e a única peça com um split de creator diferente carrega o próprio plugin Royalties dela como um override local. A mesma lógica se aplica ao Attributes sempre que o trait for de toda a coleção em vez de por peça.

O lab de hoje anexa o Royalties por ativo de qualquer jeito, e o motivo é pedagógico em vez de arquitetural: você deve digitar essa config à mão uma vez, ver o programa rejeitar um split ruim, e então construir o validador que pega isso antes de uma transação sair da sua máquina. Quando o drop do capstone alcançar escala real, mova-o para cima, para a coleção, e deixe a herança fazer o trabalho.

### Cunhagem em escala: Core Candy Machine

Tudo no lab cunha à mão porque você está cunhando quatro ativos. Um drop de 10,000 peças quer uma máquina de venda: pré-carregue as configs, deixe os compradores cunharem eles mesmos, defenda o mint com regras. Essa máquina é o Core Candy Machine, programa `CMACYFENjoBMHzapRXyo1JZkVS6EtaDDzkjMrmQLvr4J`, e as regras dele são módulos de trava: `solPayment`, `startDate`, `mintLimit`, `allowList` (com acesso restrito por prova de Merkle), `botTax` (checagens de trava que falham pagam uma taxa em vez de reverter de graça), e mais. Quantos existem no total? A página de docs anuncia "23+ composable guards" e nunca os enumera. A fonte enumera: contar as declarações `mod` no programa candy-guard, cruzado com os campos da struct `GuardSet` dele, dá exatamente 31 (conferido em 2026-08-23). Note que os docs não estão errados aqui; um piso raramente está. Eles são vagos, e você não consegue projetar contra um piso. Cite a contagem da fonte, e reconte você mesmo no dia em que o número tiver que carregar peso.

As travas se compõem em grupos nomeados, que é como uma máquina roda todo um cronograma de drop. Um grupo é um conjunto de travas rotulado (os labels são limitados a seis caracteres), os compradores passam o label quando cunham, e quaisquer travas default que você definir fora dos grupos são herdadas, a não ser que um grupo as sobrescreva. Então um grupo `wl` carrega a raiz de Merkle do `allowList` e o `solPayment` com desconto, um grupo `public` carrega o preço cheio e nenhuma cancela, e o `botTax` fica nos defaults, onde protege os dois. Uma vez que grupos existem, cunhar só com os defaults não é permitido: um label é sempre obrigatório. Essa é a forma que o módulo 8 preenche com números reais.

Duas coisas para levar adiante e uma para nunca fazer. Leve adiante: o par anti-snipe que você acabou de conhecer, `botTax` e `allowList`, é a espinha dorsal de um mint justo, e o mesmo problema de defender-o-lançamento volta no módulo 8 em volta de bonding curves. E o Core Candy Machine cunha SOMENTE ativos Core. O nunca: a linha legada do Candy Machine V3 cunha NFTs do Token Metadata e está deprecada junto com o padrão que ela serve; se um tutorial te entregar o V3, você está lendo história.

![Pipeline de uma transação de comprador passando pelas travas startDate, allowList, mintLimit e solPayment para dentro de um mint que deposita o ativo numa coleção Core, com as checagens que falham roteadas para o botTax.](assets/v08-flowchart.png)

### O trade-off, nomeado

O modelo de conta única do Core é o motivo pelo qual o mint é uns 87% mais barato e o motivo pelo qual royalties, comportamento soulbound e edições são plugins tipados em vez de espalhamento de PDA. O que você perde: superfície de maturidade. O Token Metadata tem meia década de integrações, uma vantagem rodando desde o padrão de 2021 que você viu na linha do tempo do m06-l1; toda carteira, marketplace e script de backend empoeirado o entende, enquanto o suporte ao Core é amplo em 2026 mas mais jovem, e você ainda vai encontrar ferramentas que leem o TM e dão de ombros para o Core. Segundo, os dentes de um plugin Royalties são exatamente o `ruleSet` dele: entregue `None` e o seu royalty on-chain é consultivo, uma realidade que o m06-l3 disseca sem anestesia. Terceiro, você se compromete com a ordenação: os ativos são cunhados DENTRO de uma coleção verificada, e fazer retrofit é uma tarefa de duas autoridades que você deveria tratar como uma falha de planejamento, não como um fluxo de trabalho.

Você consegue ler a passagem de bastão só nos trens de release, sem precisar de anúncio. O pacote JS `mpl-token-metadata` parou na v3.4.0 em fevereiro de 2025 e não entrega uma feature desde então. Enquanto isso o programa Core cortou da 0.13.0 até a 0.15.1 ao longo de maio e junho de 2026, o crate cliente em Rust dele chegou na 0.12.1 em 2026-06-16, e o SDK JS do Core chegou na 1.10.0 em abril de 2026. Uma linha ficou quieta; as outras três mantiveram uma cadência constante. É assim que uma migração de padrão se parece do lado do changelog.

![Linha do tempo mostrando a linha JS do mpl-token-metadata parando na v3.4.0 em fevereiro de 2025 enquanto o Metaplex Core entregou a JS 1.10.0 e as versões de programa 0.13.0 até 0.15.1 ao longo de 2026.](assets/v09-timeline.png)

## Lab: cunhe o Almanac

A ordem de construção espelha a teoria: coleção, depois membro, depois impressão, depois prova. Tudo roda contra o surfnet, e todo script compartilha uma carteira só para que as contas persistam entre rodadas.

1. **O setup compartilhado.** Um helper cuida da persistência e do financiamento da carteira, para que re-rodadas não deixem as suas contas órfãs:

    ```typescript
    // labs/m06-l2/umi.ts
    import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
    import { mplCore } from "@metaplex-foundation/mpl-core";
    import { keypairIdentity, sol } from "@metaplex-foundation/umi";
    import fs from "node:fs";

    export async function getUmi() {
      const umi = createUmi(process.env.RPC_URL ?? "http://127.0.0.1:8899").use(mplCore());

      let secret: Uint8Array;
      if (fs.existsSync("wallet.json")) {
        secret = Uint8Array.from(JSON.parse(fs.readFileSync("wallet.json", "utf8")));
      } else {
        const fresh = umi.eddsa.generateKeypair();
        fs.writeFileSync("wallet.json", JSON.stringify(Array.from(fresh.secretKey)));
        secret = fresh.secretKey;
      }
      const keypair = umi.eddsa.createKeypairFromSecretKey(secret);
      umi.use(keypairIdentity(keypair));

      const balance = await umi.rpc.getBalance(keypair.publicKey);
      if (balance.basisPoints < sol(1).basisPoints) {
        await umi.rpc.airdrop(keypair.publicKey, sol(2));
      }
      return umi;
    }
    ```

    Rode tudo de dentro de `labs/m06-l2/` para que `wallet.json` e o caderno de endereços que os scripts compartilham (`almanac.json`) caiam num lugar só.

2. **Crie a coleção Almanac (trabalhado por inteiro).** A coleção carrega o plugin MasterEdition desde o nascimento para que a tiragem do passo 5 tenha um teto para ler; uma bandeira de honestidade agora, descontada no passo 5, é que o teto é dado registrado que o ecossistema lê, não algo que o programa impõe no mint, então carregá-lo desde o nascimento é a forma coleção-primeiro em vez de uma dependência mecânica.

    ```typescript
    // labs/m06-l2/create-collection.ts
    import { generateSigner } from "@metaplex-foundation/umi";
    import { createCollection, fetchCollection } from "@metaplex-foundation/mpl-core";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const collectionSigner = generateSigner(umi);

      await createCollection(umi, {
        collection: collectionSigner,
        name: "Overgrowth Almanac",
        uri: "https://overgrowth.example/almanac/collection.json",
        plugins: [
          {
            type: "MasterEdition",
            maxSupply: 100,
            name: undefined,  // optional edition-line name; unset here, and step 5
            uri: undefined,   // passes each print its own name and uri anyway
          },
        ],
      }).sendAndConfirm(umi);

      const collection = await fetchCollection(umi, collectionSigner.publicKey);
      console.log("collection:", collection.publicKey);
      console.log("update authority:", collection.updateAuthority);
      console.log("master edition maxSupply:", collection.masterEdition?.maxSupply);

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ collection: collectionSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    `npx tsx create-collection.ts` imprime o endereço da coleção, a sua carteira como a autoridade de atualização dela, e `maxSupply: 100`. Essa linha de autoridade importa: é a assinatura que vai autorizar toda escrita de pertencimento daqui para frente.

3. **Cunhe o Almanac Vol. 1 (você preenche duas lacunas).** A chamada de mint é entregue a você com o plugin Royalties e a asserção de pertencimento deixados em aberto. Preencha os dois antes de espiar o passo 4. A forma do Royalties está na seção do catálogo: 500 basis points, a sua identidade como o único creator em 100, `ruleSet` de `None`. A asserção de pertencimento deve provar, só a partir do ativo buscado, que este ativo é um membro verificado da coleção que você acabou de criar.

    ```typescript
    // labs/m06-l2/mint-almanac.ts
    import { generateSigner, publicKey } from "@metaplex-foundation/umi";
    import { create, fetchAsset, fetchCollection, ruleSet } from "@metaplex-foundation/mpl-core";
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      const collection = await fetchCollection(umi, publicKey(saved.collection));

      const assetSigner = generateSigner(umi);
      await create(umi, {
        asset: assetSigner,
        collection,
        name: "Almanac: Vol. 1",
        uri: "https://overgrowth.example/almanac/vol-1.json",
        plugins: [
          // TODO(you): the Royalties plugin.
          // 500 basis points, one creator (umi.identity.publicKey) at percentage 100,
          // ruleSet("None"). The exact shape is in the plugin catalog.
        ],
      }).sendAndConfirm(umi);

      const asset = await fetchAsset(umi, assetSigner.publicKey);
      console.log("asset:", asset.publicKey);
      console.log("owner:", asset.owner);

      // TODO(you): the membership assertion.
      // Prove asset.updateAuthority is the Collection arm, and that its address
      // is exactly saved.collection. Two asserts, no RPC calls beyond the fetch.

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ ...saved, asset: assetSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Enquanto você está lá dentro, tente sabotar a sua própria config de royalties uma vez: coloque a porcentagem do creator em 99 e rode. O programa rejeita o mint. Esse revert é o comportamento exato que o validador do seu coding challenge reproduz off-chain.

4. **A revelação.** O preenchimento do Royalties:

    ```typescript
    plugins: [
      {
        type: "Royalties",
        basisPoints: 500,
        creators: [{ address: umi.identity.publicKey, percentage: 100 }],
        ruleSet: ruleSet("None"),
      },
    ],
    ```

    E a asserção de pertencimento:

    ```typescript
    assert.equal(asset.updateAuthority.type, "Collection");
    assert.equal(asset.updateAuthority.address, publicKey(saved.collection));
    ```

    Duas linhas, zero fetches extras. As chaves públicas do Umi são strings simples em runtime, então igualdade estrita no endereço simplesmente funciona. Se a sua versão fez assert contra uma coleção buscada em vez disso, não está errado, só é mais caro; o ponto do design do Core é que a prova de pertencimento vive nos próprios bytes do ativo. `npx tsx mint-almanac.ts` deve agora imprimir o endereço do ativo e sair limpo por ambos os asserts.

5. **Imprima a edição numerada (trabalhado).** Mesma chamada `create`, plugin diferente. O plugin Edition carrega o número de impressão; o MasterEdition da coleção carrega o teto que você definiu no passo 2:

    ```typescript
    // labs/m06-l2/mint-print.ts
    import { generateSigner, publicKey } from "@metaplex-foundation/umi";
    import { create, fetchAsset, fetchCollection } from "@metaplex-foundation/mpl-core";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      const collection = await fetchCollection(umi, publicKey(saved.collection));

      const printSigner = generateSigner(umi);
      await create(umi, {
        asset: printSigner,
        collection,
        name: "Almanac 2026, print #1",
        uri: "https://overgrowth.example/almanac/print-1.json",
        plugins: [{ type: "Edition", number: 1 }],
      }).sendAndConfirm(umi);

      const print = await fetchAsset(umi, printSigner.publicKey);
      console.log("print:", print.publicKey);
      console.log("edition number:", print.edition?.number);

      fs.writeFileSync(
        "almanac.json",
        JSON.stringify({ ...saved, print: printSigner.publicKey }, null, 2),
      );
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Uma ressalva honesta antes de você escalar isso, e ela cobre o teto tanto quanto os números: quando você cunha por SDK, a contabilidade é sua para administrar. Nada impede um script desleixado de cunhar dois print #1, e nada on-chain impede o print #101 tampouco; o `maxSupply` do MasterEdition é dado registrado que o ecossistema lê, não um teto que o programa Core impõe no mint. A integridade sequencial E o teto de supply são promessas do lado do cliente aqui. Em escala de drop, o Core Candy Machine atribui os números e impõe o teto para você, que é mais um motivo para ele existir. O brief do drop do músico do capstone se constrói diretamente sobre esse passo, então garanta que `edition number: 1` imprima antes de seguir adiante.

6. **O espelho na CLI (opcional, mostrado uma vez).** Tudo que você scriptou tem um gêmeo de linha de comando na CLI da Metaplex, útil para dar uma espiada rápida em contas sem abrir um editor:

    ```bash
    npm install -g @metaplex-foundation/cli
    mplx core asset fetch <your asset address> --rpc http://127.0.0.1:8899
    ```

    `mplx core asset create` e `mplx core collection create` também existem, junto com `mplx core plugins add`. O binário é `mplx`, a CLI está na própria linha 0.x dela (0.4.3 na hora em que escrevo, então reconfira a árvore de comandos com `mplx core --help` antes de scriptar contra ela), e a ordem dos tópicos é substantivo e depois verbo: `core asset fetch`, não `core fetch asset`. O curso scripta tudo em TS porque scripts se compõem em cancelas de verificação e sessões de CLI não, mas saber que o espelho existe te economiza tempo em leituras pontuais.

7. **Rode a cancela.** O script de verificação abaixo é o contrato do R7: a prova de que quatro ativos existem carregando exatamente as propriedades que o resto deste curso presume que você consegue construir. Ele é dado por inteiro porque as propriedades são o entregável, não o script. Mas seja claro sobre o que realmente viaja. As lições posteriores reconstroem a coleção Almanac em qualquer cluster contra o qual elas rodem em vez de ler o seu `almanac.json`, porque uma conta de surfnet não significa nada na devnet. O que é levado adiante é a receita e o hábito coleção-primeiro, não o arquivo.

    ```typescript
    // labs/m06-l2/verify-almanac.ts
    import { publicKey } from "@metaplex-foundation/umi";
    import { fetchAsset, fetchCollection } from "@metaplex-foundation/mpl-core";
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import { getUmi } from "./umi";

    async function main() {
      const umi = await getUmi();
      const saved = JSON.parse(fs.readFileSync("almanac.json", "utf8"));
      for (const key of ["collection", "asset", "print", "badge"]) {
        assert.ok(saved[key], `almanac.json is missing "${key}" - run the mint scripts first`);
      }

      // 1. The collection exists and carries its MasterEdition cap.
      const collection = await fetchCollection(umi, publicKey(saved.collection));
      assert.ok(collection.masterEdition, "collection has no MasterEdition plugin");
      console.log(`OK: collection ${collection.name} (${collection.publicKey})`);
      console.log(`OK: master edition maxSupply=${collection.masterEdition.maxSupply}`);

      // 2. The Almanac asset is a verified member and its Royalties plugin reads back.
      const asset = await fetchAsset(umi, publicKey(saved.asset));
      assert.equal(asset.updateAuthority.type, "Collection", "asset is not collection-owned");
      assert.equal(asset.updateAuthority.address, publicKey(saved.collection));
      assert.ok(asset.royalties, "asset has no Royalties plugin");
      const shares = asset.royalties.creators.reduce((sum, c) => sum + c.percentage, 0);
      assert.equal(shares, 100, "creator shares must sum to 100");
      console.log(`OK: ${asset.name} is a verified member of the Almanac collection`);
      console.log(`OK: royalties ${asset.royalties.basisPoints} bps, shares sum to ${shares}`);

      // 3. The print reads back its edition number.
      const print = await fetchAsset(umi, publicKey(saved.print));
      assert.equal(print.updateAuthority.type, "Collection");
      assert.ok(print.edition, "print has no Edition plugin");
      console.log(`OK: ${print.name} reads back edition number ${print.edition.number}`);

      // 4. The badge is permanently frozen: soulbound.
      const badge = await fetchAsset(umi, publicKey(saved.badge));
      assert.ok(badge.permanentFreezeDelegate, "badge has no PermanentFreezeDelegate plugin");
      assert.equal(badge.permanentFreezeDelegate.frozen, true, "badge is not frozen");
      console.log(`OK: ${badge.name} is non-transferable (PermanentFreeze)`);
    }

    main().catch((err) => { console.error(err); process.exit(1); });
    ```

    Rode `npx tsx verify-almanac.ts` agora e ele para no primeiro assert: `almanac.json is missing "badge"`. Esse é o comportamento correto. A cancela está te dizendo o que falta, e o que falta é o challenge. Note o que o script nunca faz: ele nunca toca o DAS, nunca pergunta nada a um indexador. Toda prova é uma leitura direta de conta dos bytes que você cunhou. Ler esses mesmos ativos pela interface DAS, em escala de coleção, através de um provedor, é o trabalho do m07-l2, e vai parecer luxuoso depois disso.

## Challenge

Solo, sem apoio. Cunhe o badge Founding-Farmer: um ativo Core na coleção Almanac carregando `PermanentFreezeDelegate` com `frozen: true` e uma autoridade de `None`, para que nada consiga descongelá-lo nunca. Escreva o endereço dele em `almanac.json` sob a chave `badge`. Então prove o congelamento com as suas próprias asserções, num script seu: uma tentativa de `transfer` no badge tem que falhar (capture a rejeição e faça assert de que você a capturou), e um `transfer` do seu ativo Almanac Vol. 1 para uma carteira descartável tem que ter sucesso, na mesma rodada, para que a prova mostre que o congelamento é uma propriedade do badge e não alguma má configuração global. Pense antes de cunhar: este é o único ato irreversível da lição. Um erro de digitação no nome do badge é, talvez surpreendentemente, recuperável: o caminho de update não é o que o PermanentFreeze bloqueia, então a autoridade de atualização ainda consegue renomeá-lo. Um badge cunhado para a carteira errada simplesmente se foi: congelado onde caiu, intransferível, indescongelável, para sempre.

Então encare o coding challenge de config do plugin de royalties: implemente `validateRoyalties(basisPoints, ruleSet, creatorSpec)`, o validador puro de pré-voo para a config exata que você escreveu à mão no passo 3. O grader passa a config como três argumentos posicionais, o número de basisPoints, o nome do ruleSet, e o split de creators achatado em uma string de entradas `address=percentage` separadas por ponto e vírgula (então um split 70/30 chega como `'Farm1...=70;Farm2...=30'`); o starter já parseia essa string de volta em objetos Creator para você. O starter checa só a soma das fatias; você adiciona o resto, que é o intervalo de basisPoints, a variante de ruleSet, o intervalo de porcentagem de cada creator, e endereços de creator duplicados. Você viu o revert on-chain no passo 3; agora construa a trava que pega isso antes de uma transação sair da sua máquina.

Aceito quando: `npx tsx verify-almanac.ts` passar de ponta a ponta, todas as quatro seções verdes; a sua prova de transferência mostrar o badge falhando e o Vol. 1 se movendo na mesma rodada; e os testes do challenge passarem em todos os casos, incluindo os que você talvez não tenha pensado (um basisPoints de 20000 sentado em cima de um split perfeitamente válido, um endereço de creator duplicado cujas fatias ainda somam 100, um ruleSet escrito `ProgramList`).

## Checkpoint

O critério: `npx tsx verify-almanac.ts` imprime as seis linhas OK dele. Collection com o teto de MasterEdition dela, o Vol. 1 um membro verificado com royalties lendo de volta em 500 bps, o print #1 carregando o número de edição dele, o badge permanentemente congelado. Com isso verde, o R7 está completo, e a resposta de uma frase que você deveria conseguir dar sem olhar: a coleção tem que ser criada antes que qualquer ativo seja cunhado, porque o pertencimento é escrito no ativo no mint sob a assinatura da autoridade da coleção, e pregá-lo depois é um retrofit de duas autoridades que você deveria tratar como uma falha de planejamento, não como um fluxo de trabalho.

Os erros que eu espero. Primeiro, ordenação: se a sua asserção de pertencimento falhar com `updateAuthority.type === "Address"`, você cunhou sem passar a coleção, e nenhuma quantidade de re-fetch resolve isso; cunhe de novo, coleção-primeiro. Segundo, o revert de royalties: um split que não soma 100 ou um basisPoints fora de 0..10000 falha na hora do mint com um erro do Core, que é a spec do seu validador escrita como stack trace. Terceiro, se a transferência do badge TIVER SUCESSO na sua prova do challenge, confira qual ativo você congelou; mais de um estudante já congelou permanentemente o Vol. 1 dele e deixou o badge líquido, e num surfnet descartável isso é uma lição de graça sobre exatamente por que o PermanentFreeze merece respeito na mainnet.

![Diagrama de hub do artefato R7 completo, a coleção Almanac com ativo de royalty, impressão numerada e badge congelado, consumido pelo Bubblegum v2, pela lição de DAS, pelo módulo 8 e pelo capstone.](assets/v10-diagram.png)

Você cunhou uma coleção, três tipos de membro, e provou toda propriedade com leituras diretas. Custo total no seu surfnet: trocados, e o mesmo fluxo na mainnet fica nos milésimos de um SOL por ativo, pelos números do próprio fornecedor. Mas olhe de volta para o que você de fato entregou naquele plugin Royalties. Você o anexou, você definiu 500 basis points, você verificou que ele lê de volta. Alguém de fato o impõe? Você entregou `ruleSet("None")`, e eu deixei. A próxima lição é a realidade dos royalties que ninguém anuncia: que imposição realmente existe, o que os pNFTs e o Token Auth Rules realmente fazem, e por que o padrão contra o qual metade do ecossistema ainda integra é oficialmente legado. Traga um estômago forte para a palavra "consultivo".
