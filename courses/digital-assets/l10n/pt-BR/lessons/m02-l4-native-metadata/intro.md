# Metadados nativos: MetadataPointer + TokenMetadata TLV

## Resumo

Você acabou de construir a camada de proteção: um mint de badge soulbound cujo par forçado NonTransferableAccount + ImmutableOwner você confirmou em toda conta de holder, e uma tesouraria que exige memo e recusa depósitos sem etiqueta. A mecânica está pronta, e ela está espalhada pelos artefatos do módulo de propósito: o mint de economia de m02-l1 cobra e faz harvest de taxas, os mints descartáveis de m02-l2 provam quem pode congelar e reaver, o badge e a tesouraria de m02-l3 provam o que uma conta pode recusar. E o SPROUT que uma carteira de fato renderiza continua uma string anônima de base58. Um token sem nome é inutilizável, não no sentido criptográfico mas no único sentido que importa para um usuário encarando `6NDNZ...vBSY` e se perguntando se acabou de tomar um rug.

Primeiro, olhe o padrão em produção. Com o surfnet de m02-l1 rodando (`surfpool start --no-tui --no-studio` num terminal separado), jogue isto em `labs/m02-l4/probe-pyusd.ts` e rode `npx tsx labs/m02-l4/probe-pyusd.ts`:

```typescript
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "http://127.0.0.1:8899");
const PYUSD = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
const mint = await fetchMint(rpc, PYUSD);
if (mint.data.extensions.__option === "Some") {
  for (const e of mint.data.extensions.value) {
    if (e.__kind === "MetadataPointer" && e.metadataAddress.__option === "Some")
      console.log("pointer ->", e.metadataAddress.value);
    if (e.__kind === "TokenMetadata")
      console.log("metadata:", e.name, "/", e.symbol);
  }
}
```

O ponteiro imprime `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`. O ponteiro de metadados do PYUSD aponta para o próprio mint do PYUSD. Guarde esse fato; a lição inteira é uma explicação do porquê.

Esta lição resolve isso do jeito nativo do Token-2022: o nome, o símbolo e a URI passam a ser armazenados NO PRÓPRIO MINT, dentro da mesma região TLV pela qual o seu inspetor vem caminhando desde o módulo 1, com um ponteiro que protege contra uma conta de metadados falsificada. É o último tijolo do R3, e como este é o primeiro aparecimento do degrau pelo nome: o R3 é o próprio mint SPROUT de produção, o terceiro artefato na escada do curso depois do R1 (decode-mint) e do R2 (check-combo). Depois de hoje o R3 está completo na forma que os módulos posteriores consomem: um mint compondo as extensões que podem legalmente dividi-lo (config de taxa, extensão de exibição, ponteiro autorreferencial, TokenMetadata), com os comportamentos de autoridade e de proteção do módulo provados nos próprios mints companheiros, já que um mint soulbound ou descartável não pode ser também a moeda negociável. Os módulos 5 e 9 consomem esse mint composto direto; o módulo 3 não consegue, porque o TransferHook é de tempo de criação, então ele cunha uma variante fresca com hook do lado dele.

O recuo da ajuda, dito em voz alta: a ligação do ponteiro e do TLV do TokenMetadata é trabalhada por inteiro, cada instrução explicada. A escrita do campo `additional_metadata` é um problema de completion, você constrói essa instrução sozinho antes de eu mostrá-la. E o challenge é totalmente solo: um mint fresco, uma asserção de ida e volta, sem apoio.

## Metadados que moram no mint

### De onde o nome de um token vem de verdade

Pense no que uma carteira faz quando ela renderiza o seu saldo. Ela tem um endereço de mint e nada mais. Em algum lugar ela precisa resolver esse endereço em "SPROUT, 6 decimals, este logo." Durante toda a era do SPL clássico, a resposta morava fora do programa de token: uma conta de metadados separada, de propriedade de um programa separado (o Token Metadata da Metaplex), num endereço derivado do mint. O programa de token não sabia nada sobre nomes. Dois programas, duas contas, uma identidade, colados por convenção.

![Uma carteira resolve um mint Token-2022 em uma leitura de conta, caminhando pelo TLV até o ponteiro autorreferencial e a entrada de metadados, diferente do caminho legado de duas contas da Metaplex.](assets/v01-flowchart.png)

O Token-2022 colapsa isso. Duas das 29 extensões de produção no enum ExtensionType existem exatamente para esse trabalho:

- **MetadataPointer** (tipo de extensão 18): um campo pequeno do lado do mint que responde uma pergunta, "onde moram os metadados canônicos deste mint?" Ele guarda uma autoridade opcional (quem pode re-apontar o ponteiro) e um endereço de metadados opcional.
- **TokenMetadata** (tipo de extensão 19): os metadados em si, uma entrada TLV de comprimento variável guardando o nome, o símbolo e a URI de verdade, definida por `spl_token_metadata_interface`.

O design é de duas peças em vez de uma de propósito. O ponteiro é a indireção: os metadados PODERIAM morar em alguma outra conta, mantida por algum outro programa que implementa a interface de metadados. Mas o padrão que este curso ensina, o padrão que o PYUSD entrega, é o caso degenerado: aponte o mint para ele mesmo e guarde o TLV inline. Uma conta, um programa, uma leitura.

![O modelo da Metaplex usa um PDA de metadados separado de propriedade de outro programa, enquanto o modelo nativo do Token-2022 guarda ponteiro e metadados dentro do próprio mint.](assets/v02-diagram.png)

Por que o ponteiro existe afinal, se a resposta é "aponte para você mesmo"? Porque a interface é maior que o caso inline. A `spl_token_metadata_interface` é uma especificação que qualquer programa pode implementar, e um mint criado antes de as extensões de metadados existirem ainda pode apontar o ponteiro dele para uma conta de metadados externa. O ponteiro é a resposta publicada, on-chain, para "qual conta é a canônica." O que nos leva ao ataque contra o qual ele foi projetado.

### O argumento antifalsificação, a partir dos primeiros princípios

Suponha que não houvesse ponteiro, só uma convenção: "os metadados do mint M moram em alguma conta que alega `mint = M`." Qualquer um pode criar uma conta. Eu posso criar uma conta amanhã que diz `mint = <PYUSD's address>`, `name = "PayPal USD"`, `symbol = "PYUSD"`, e uma URI apontando para o JSON que eu quiser. Nada na chain distingue a minha falsificação da coisa de verdade, porque a alegação mora na conta falsificável, não na coisa sobre a qual se alega. Um indexador que varre contas de metadados encontra dois candidatos para o PYUSD e não tem regra on-chain para escolher. Essa é a superfície de falsificação, e ela não é hipotética: é por isso que campanhas de golpe com NFT conseguiram pregar metadados de aparência oficial em mints lixo por anos.

O ponteiro inverte a direção da confiança. O mint diz qual conta fala por ele, e só o conjunto de autoridades do próprio mint poderia ter escrito aquele campo. Uma terceira conta ainda pode alegar o que ela quiser; nenhum leitor vai jamais seguir um ponteiro até ela. E a struct TokenMetadata fecha o loop do outro lado com o campo `mint` dela: os metadados nomeiam o mint deles, o mint nomeia os metadados dele. Quando os dois moram na mesma conta, como no SPROUT e no PYUSD, o loop tem uma conta de comprimento e não sobra nada para forjar. Para falsificar os metadados você precisaria de acesso de escrita ao próprio mint, e nesse ponto você é dono do token e não está falsificando nada.

Esta é também exatamente a cilada a evitar quando você liga isso: aponte o MetadataPointer para alguma conta arbitrária que você por acaso controla e você reintroduziu a indireção onde o ataque mora. A não ser que você esteja deliberadamente implementando um programa de metadados externo (você não está, e quase ninguém está), autorreferencial é o único valor que você deveria escrever.

![Sem um ponteiro qualquer conta pode alegar ser os metadados de um mint, enquanto um ponteiro autorreferencial significa que os leitores seguem só a referência de saída do mint, deixando as falsificações inalcançáveis.](assets/v03-diagram.png)

### O que está de fato no TLV, e o que não está

A entrada TokenMetadata é definida pela `spl_token_metadata_interface`, e o formato dela vale memorizar porque você vai reler ele pelo resto do curso:

```rust
// spl_token_metadata_interface state (the shape, as stored in the TLV)
pub struct TokenMetadata {
    pub update_authority: OptionalNonZeroPubkey, // who may edit fields later
    pub mint: Pubkey,                            // the mint this speaks for (anti-spoof, other direction)
    pub name: String,                            // ON-CHAIN
    pub symbol: String,                          // ON-CHAIN
    pub uri: String,                             // link to off-chain JSON
    pub additional_metadata: Vec<(String, String)>, // arbitrary on-chain key-value pairs
}
```

Dois equívocos para matar enquanto a struct está na sua frente. Primeiro: nome e símbolo são strings on-chain, armazenadas nos bytes do mint, legíveis com `getAccountInfo` e nada mais. A URI aponta para o JSON off-chain para os campos pesados (imagem, descrição, o que o seu produto precisar), mas os campos de identidade não moram nesse JSON. Uma carteira consegue renderizar "SPROUT" sem um único fetch HTTP. Segundo: esta não é a struct `Data` on-chain da Metaplex. Programa diferente, modelo de conta diferente, layout de campos diferente, e código escrito para um não vai desserializar o outro. A comparação completa, incluindo quando a stack da Metaplex ainda é a escolha certa, é a lição de abertura do módulo 6; por ora basta manter os dois mentalmente separados.

Isso deixa a outra ponta da URI sem explicação, e o Token-2022 não tem nada a dizer sobre ela. O programa armazena uma string e nunca faz fetch dela. Não existe schema on-chain, nem validador, nem checagem de conteúdo, nem imposição de nenhum tipo. O que existe no lugar é uma convenção: o formato de JSON off-chain que a Metaplex popularizou (`name`, `symbol`, `description`, `image`, e um array `attributes`), que as carteiras aprenderam a parsear anos antes de os metadados nativos existirem e que os tokens de metadados nativos herdaram por padrão. É o que uma carteira tenta primeiro quando faz fetch da sua URI. Duas consequências práticas decorrem daí. O seu `name` on-chain e o `name` do JSON podem divergir, e nada na chain vai impedir isso, então mantenha os dois em sincronia deliberadamente. E qualquer host que sirva essa URI agora é uma dependência da aparência do seu token, o que é um argumento concreto para manter a autoridade de atualização viva em vez de queimá-la no primeiro dia. O módulo 6 abre nesse padrão de JSON como se deve, incluindo quais campos os marketplaces de fato leem.

![O TLV do mint guarda os campos de identidade impostos enquanto a URI aponta para um JSON off-chain convencional que nada na chain valida nem mantém em sincronia.](assets/v04-diagram.png)

`additional_metadata` é a parte extensível: pares arbitrários de string chave-valor, on-chain, editáveis pela autoridade de atualização. O Overgrowth vai usar isso no lab para um campo `harvest_season`, e é o mecanismo atrás de todo esquema de "trait em um token fungível" que você vai encontrar na natureza.

Aquele campo `update_authority` é uma decisão de produto escondida numa struct, então tome ela consciente. É um `OptionalNonZeroPubkey`: segure ele e você pode editar todo campo depois pelas instruções de update da interface; defina como none e os metadados são imutáveis, para sempre, sem volta. Um emissor que talvez precise rotacionar um host de URI comprometido ou corrigir um typo mantém a autoridade viva, sob a mesma disciplina de ops da autoridade de mint que você configurou em m02-l2. Uma memecoin provando que nunca pode se renomear em silêncio queima a autoridade e diz isso. Nenhum dos dois está errado; entregar sem ter escolhido está. O SPROUT mantém a autoridade dele por ora, porque a passada de endurecimento de produção do módulo 9 revisita toda autoridade no mint de uma vez, e esta pertence àquela varredura.

### Os bytes, já que você consegue lê-los

Você construiu um caminhador de TLV em m01-l2, então nada sobre o armazenamento deveria continuar abstrato. Cada entrada é um tipo little-endian de 2 bytes, um comprimento de 2 bytes, e então o valor. O valor do ponteiro é fixo em 64 bytes: pubkey da autoridade, depois endereço dos metadados, 32 cada, zerados quando não definidos. O valor do TokenMetadata é uma serialização Borsh da struct acima: duas pubkeys cruas de 32 bytes (autoridade de atualização, mint), depois cada string como um prefixo de comprimento de 4 bytes mais bytes UTF-8, depois o vetor de pares como uma contagem de 4 bytes com strings prefixadas dentro. Comprimento variável, exatamente como a história do realloc exige.

Rode a aritmética uma vez para o SPROUT e o tamanho da conta para de ser mágico: 64 + (4 + 6) para `name = "SPROUT"`, (4 + 4) para `SPRT`, (4 + 38) para a URI, (4 + 18 + 10) para um par `harvest_season = "spring"`. São 156 bytes de valor, 160 com o header TLV dele. Segure esse 160, porque a próxima seção precifica a conta inteira com ele: um mint SPROUT só com ponteiro fica em 234 bytes, e 234 + 160 = 394 é o tamanho que o lab financia. Derivado aqui, com assert lá.

![Layout em nível de bytes da entrada TLV do TokenMetadata, duas pubkeys de 32 bytes mais strings com prefixo de comprimento Borsh totalizando 160 bytes, o que leva o mint de 234 bytes aos 394 bytes que o lab financia.](assets/v05-annotated-code.png)

Mais uma instrução completa a interface, e ela existe para o caso que o SPROUT nunca encontra: `Emit`. Um leitor que quer metadados sem saber para onde o ponteiro leva pode pedir ao programa dono dos metadados para serializar a struct em return data e ler isso de uma simulação. Para um mint autorreferencial ela é redundante, o `fetchMint` lê o TLV direto da conta com um `getAccountInfo`, nenhum indexador à vista. Mas quando o ponteiro aponta para um programa de metadados externo, o `Emit` é o caminho de leitura uniforme que mantém toda implementação da interface legível pelo mesmo código de cliente.

Mais um par de extensões pertence a este mapa mental, porque elas espelham este design exatamente: GroupPointer/TokenGroup (tipos 20 e 21) e GroupMemberPointer/TokenGroupMember (tipos 22 e 23) fazem por coleções o que o par de metadados faz pela identidade, um ponteiro autorreferencial mais estado TLV inline. Elas são como o Token-2022 expressa "este mint pertence àquele grupo" nativamente. A gente não vai ligar elas hoje, e este curso nunca liga; elas importam para ativos com formato de NFT, e quando o módulo 6 pegar coleções você vai ver o mesmo problema de pertencer-àquele-grupo resolvido do lado da Metaplex em vez disso.

### A emenda do tempo de criação, e a dança do realloc

Aqui está a parte que de fato morde as pessoas, e é uma regra que você já conhece de ter construído o SPROUT três vezes: extensões de mint são inicializadas ANTES do `InitializeMint`. Uma vez que um mint Token-2022 está inicializado, o conjunto de extensões dele é fixo. Você não pode parafusar um MetadataPointer no mint de ontem. É por isso que esta lição, como as três anteriores, recria o SPROUT em vez de atualizá-lo; o script de criação de mint do lab é o seu script de m02-l3 mais uma entrada na lista de extensões.

Mas o TokenMetadata quebra o padrão, e a assimetria é a decisão de design interessante desta lição. O ponteiro é de tamanho fixo e de tempo de criação. Os metadados são de comprimento variável: o seu nome hoje, uma URI mais longa amanhã, cinco pares `additional_metadata` a mais na próxima temporada. Dimensionar isso na alocação congelaria tudo. Então a interface faz dos metadados uma instrução pós-init: depois do `InitializeMint`, você chama o initialize da interface, o programa segue o próprio ponteiro do mint de volta até o mint, realoca a conta para caber a nova entrada TLV, e escreve os campos.

Realocar custa rent, e o programa não vai pagar por você. A dança, concretamente, com os números do build do lab:

- Um mint clássico simples tem 82 bytes. Com qualquer extensão presente, a base preenche até 165 bytes mais um byte de tipo de conta, e então as entradas TLV seguem.
- O SPROUT-só-com-ponteiro aloca em **234 bytes**: é isso que você passa para o `createAccount` como `space`.
- A entrada de metadados do SPROUT (nome `SPROUT`, símbolo `SPRT`, uma URI de 38 caracteres, um par chave-valor) vai realocar a conta para **394 bytes**: é esse o tamanho que você precisa FINANCIAR.

Então você aloca espaço para 234 e deposita lamports para 394. A biblioteca cliente deixa isso indolor: o `getMintSize` aceita uma cópia fantasma da extensão de metadados puramente para a aritmética, e os lamports resultantes ficam parados no mint até o realloc reivindicar eles. Financie de menos e a instrução de metadados falha com um erro de fundos-insuficientes-para-rent; nada corrompe, mas a sua transação de criar-e-nomear morre na instrução quatro de cinco.

E a dança não termina na criação, que é a parte que as pessoas descobrem em produção. Seis meses a partir de agora você troca a URI por uma string mais longa, ou adiciona um segundo par `additional_metadata`, e essa escrita realoca o mint de novo. A conta tem que estar isenta de aluguel no NOVO tamanho dela, e a instrução de update não vai conjurar a diferença do nada. Então uma atualização de metadados são de verdade duas operações: a chamada da interface, e uma transferência de lamports para o mint que cobre o crescimento. Encolher funciona no sentido contrário e simplesmente deixa o mint superfinanciado, já que ninguém te devolve a sobra. Faça orçamento para isso do jeito que você faria para uma migração de schema, porque debaixo do vocabulário é exatamente o que é.

![Fluxograma de cinco instruções, aloque 234 bytes financiados para 394, inicialize o ponteiro autorreferencial, inicialize o mint, e então a instrução de metadados pós-init realoca e escreve os campos.](assets/v06-flowchart.png)

Por que tolerar essa complexidade em vez de simplesmente fazer dos metadados uma extensão de tempo de criação também? Trade-off, nomeado sem rodeios. Metadados nativos mantêm a identidade no mint: nenhuma conta extra para criar, nenhum programa externo para confiar, nenhuma derivação de PDA para as carteiras saberem, e a superfície de falsificação fechada por construção. Os custos vêm em três sabores.

Rent, primeiro, e vamos precificar isso em vez de só apontar de longe. Pergunte ao mesmo método de RPC que o script do lab chama, no cluster em que o lab roda. No meu surfnet (2026-09-06) a isenção de aluguel para o mint clássico de 82 bytes é 1,461,600 lamports, o SPROUT só com ponteiro de 234 bytes é 2,519,520, e o SPROUT totalmente nomeado de 394 bytes é 3,633,120: 210, 362 e 522 bytes totais a 6,960 lamports cada. A mainnet cobra 6,333 desde 2026-09-03 (o primeiro passo do SIMD-0437) e retorna 1,329,930 / 2,292,546 / 3,305,826 para os mesmos três tamanhos, então um fork não está automaticamente te cotando o rent da mainnet mesmo quando está te servindo as contas da mainnet. De um jeito ou de outro o que importa é o formato: a identidade on-chain inteira do seu token custa cerca de 0.001 SOL acima do mint só com ponteiro, algumas dezenas de centavos de dólar aos preços recentes do SOL. Para um mint fungível isso é nada, que é exatamente por que o padrão cabe em tokens fungíveis: um mint, milhões de holders, os metadados pagos uma vez. Vire o formato para uma coleção de NFT, um mint POR item, e o rent de metadados por item começa a importar, uma de várias razões pelas quais o módulo 6 resolve esse trade-off do outro jeito.

Segundo, idade. O padrão nativo é anos mais novo que o da Metaplex, então o ferramental de carteiras e marketplaces construído em torno de "derive o PDA da Metaplex" precisa do caminho de seguir-o-ponteiro em vez disso, e o ferramental de cauda longa ainda ocasionalmente não tem isso. Os grandes leem ele bem, o PYUSD não renderizaria de outro jeito, mas se a vida do seu token depende de algum rastreador de portfólio de nicho, teste antes do lançamento em vez de supor.

E terceiro, o que você agora entende mecanicamente: dados de comprimento variável num mint significam a dança do realloc, dimensionando rent para bytes que você ainda não escreveu. Para um token fungível em 2026, metadados nativos são geralmente a escolha certa de todo jeito. Para coleções de NFT ricas, o caminho do Metaplex Core do módulo 6 é, e quando a gente chegar lá você vai ver o mesmo trade-off resolvido na direção oposta.

### O PYUSD roda exatamente este padrão

A sondagem que você rodou lá em cima não era um brinquedo. A PayPal e a Paxos lançaram o PYUSD na Solana em maio de 2024 como a implantação emblemática do Token-2022: uma stablecoin regulada e atestada pela KPMG (o levantamento de panorama de stablecoins da Helius colocou o supply circulante dela na Solana em $215.9M em 20.4k contas de holder em 2025-05-29; os números de hoje estão a uma leitura ao vivo de distância, e o módulo 9 faz essa leitura). O mint dele carrega oito extensões TLV. Leia elas na saída da sua própria sondagem, elas chegam nesta ordem: MintCloseAuthority, PermanentDelegate, TransferFeeConfig, ConfidentialTransferMint, ConfidentialTransferFee, TransferHook, MetadataPointer, TokenMetadata.

Você já configurou pessoalmente cinco daquelas oito em variantes do SPROUT, e a disciplina de m01 se aplica à lista inteira: presença não te diz nada, valores dizem. Na leitura de 2026-08-22, o transfer hook do PYUSD estava configurado com um programa nulo e a config de taxa dele ficava em 0 basis points com um máximo de 0, tanto na tabela de taxa mais velha quanto na mais nova. Chaves dormentes, instaladas para um futuro que o time de compliance deles pode ligar. Mas as duas que você está ligando hoje estão configuradas E vivas: o ponteiro resolve para o próprio mint, e o TLV relê `PayPal USD / PYUSD` com uma URI para `token-metadata.paxos.com`. Quando uma carteira mostra o logo da PayPal ao lado de um saldo, esta entrada TLV, lida direto do mint, é onde aquele render começa. O padrão nativo não é a opção experimental. É o que um emissor regulado de primeira linha entrega. (Se você quer o outro lado deste vidro, o curso Solana Payments & Commerce lê o mint do PYUSD ao vivo como um exercício de integração, checando o que um comerciante precisa lidar antes de aceitar ele. Aqui você é o emissor, escrevendo os bytes que aquele curso lê.)

![Comparação das oito extensões TLV do PYUSD, seis configuradas mas dormentes contra as duas extensões de metadados vivas que guardam o nome PayPal USD e o ponteiro autorreferencial.](assets/v07-comparison.png)

Mais um pedaço de contexto, rapidamente, já que você já encontrou o arquivo de 2025-01-24 do currículo oficial duas vezes: o material de metadados dele antecede o padrão nativo maduro e ainda ensina o mundo da conta separada como o default. Você está aprendendo este aqui pela interface e pelos bytes porque esse é atualmente o único lugar onde ele mora por completo.

## Lab: dê ao SPROUT o nome dele

O build: recrie o SPROUT com o ponteiro no conjunto de extensões dele, escreva o TLV, e então releia tudo e faça assert. No fim, o R3 está completo e o `verify-metadata.ts` existe na interface exata que os módulos posteriores chamam.

1. **Workspace e pins.** No seu workspace do curso (o mesmo de m02-l3), crie a pasta da lição e confirme o trio de dependências; se você está começando numa máquina nova, instale elas:

    ```bash
    mkdir -p labs/m02-l4
    npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
    ```

    Os mesmos pins, a mesma razão, como o parágrafo de pin de m02-l1 argumentou por inteiro: o workspace fixa o major do kit contra o qual os clientes `@solana-program/*` dele fazem peer, o que hoje significa kit 7.1.1 com cada cliente no minor kit-^7 atual dele (verificado contra o npm 2026-09-05; re-verifique quando você ler isto).

2. **Surfnet no ar.** O lab roda contra o surfnet local que você usa desde m02-l1 (`surfpool start --no-tui --no-studio`; ele forka a mainnet de forma preguiçosa, que é por que a sondagem do PYUSD funcionou, e honra airdrops, que é por que o próximo script se financia sozinho). A devnet serve como fallback: `RPC_URL=https://api.devnet.solana.com WS_URL=wss://api.devnet.solana.com npx tsx ...`, com o faucet substituindo a chamada de airdrop se ele te limitar por taxa.

3. **A ligação trabalhada.** Crie `labs/m02-l4/add-metadata.ts`. Esta é a ligação completa de ponteiro-e-TLV, trabalhada; leia os comentários contra a seção de teoria, especialmente a dança de dois tamanhos no meio:

    ```typescript
    import {
      airdropFactory,
      appendTransactionMessageInstructions,
      assertIsTransactionWithBlockhashLifetime,
      createSolanaRpc,
      createSolanaRpcSubscriptions,
      createTransactionMessage,
      generateKeyPairSigner,
      getSignatureFromTransaction,
      lamports,
      pipe,
      sendAndConfirmTransactionFactory,
      setTransactionMessageFeePayerSigner,
      setTransactionMessageLifetimeUsingBlockhash,
      signTransactionMessageWithSigners,
      some,
    } from "@solana/kit";
    import { getCreateAccountInstruction } from "@solana-program/system";
    import {
      TOKEN_2022_PROGRAM_ADDRESS,
      extension,
      getInitializeMetadataPointerInstruction,
      getInitializeMintInstruction,
      getInitializeTokenMetadataInstruction,
      getMintSize,
    } from "@solana-program/token-2022";
    import { writeFileSync } from "node:fs";

    const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";
    const WS_URL = process.env.WS_URL ?? "ws://127.0.0.1:8900";

    const NAME = "SPROUT";
    const SYMBOL = "SPRT";
    const URI = "https://overgrowth.example/sprout.json";

    async function main() {
      const rpc = createSolanaRpc(RPC_URL);
      const rpcSubscriptions = createSolanaRpcSubscriptions(WS_URL);
      const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

      // Payer doubles as mint authority and metadata update authority for the lab.
      const authority = await generateKeyPairSigner();
      const mint = await generateKeyPairSigner();

      await airdropFactory({ rpc, rpcSubscriptions })({
        commitment: "confirmed",
        recipientAddress: authority.address,
        lamports: lamports(2_000_000_000n),
      });

      // The pointer is a CREATE-TIME extension: it exists before InitializeMint,
      // so it belongs in the space calculation. Self-referential on purpose.
      const metadataPointer = extension("MetadataPointer", {
        authority: some(authority.address),
        metadataAddress: some(mint.address),
      });

      // The TokenMetadata TLV is POST-init: the program reallocs the mint to fit
      // it, so it never enters the allocated space. Its RENT does. This phantom
      // copy exists only to size the deposit.
      const tokenMetadata = extension("TokenMetadata", {
        updateAuthority: some(authority.address),
        mint: mint.address,
        name: NAME,
        symbol: SYMBOL,
        uri: URI,
        additionalMetadata: new Map([["harvest_season", "spring"]]),
      });

      const allocatedSpace = getMintSize([metadataPointer]);           // 234
      const fundedSpace = getMintSize([metadataPointer, tokenMetadata]); // 394
      const rent = await rpc
        .getMinimumBalanceForRentExemption(BigInt(fundedSpace))
        .send();

      const instructions = [
        // Allocate WITHOUT the metadata bytes, fund FOR them.
        getCreateAccountInstruction({
          payer: authority,
          newAccount: mint,
          space: allocatedSpace,
          lamports: rent,
          programAddress: TOKEN_2022_PROGRAM_ADDRESS,
        }),
        // Pointer before InitializeMint. Aim it at the mint itself.
        getInitializeMetadataPointerInstruction({
          mint: mint.address,
          authority: some(authority.address),
          metadataAddress: some(mint.address),
        }),
        getInitializeMintInstruction({
          mint: mint.address,
          decimals: 6,
          mintAuthority: authority.address,
          freezeAuthority: some(authority.address),
        }),
        // TLV after InitializeMint: the program follows the pointer back to the
        // mint, reallocs 234 -> 394, and writes the fields.
        getInitializeTokenMetadataInstruction({
          metadata: mint.address,
          updateAuthority: authority.address,
          mint: mint.address,
          mintAuthority: authority,
          name: NAME,
          symbol: SYMBOL,
          uri: URI,
        }),
        // COMPLETION TODO: one more instruction goes here in step 5.
      ];

      const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
      const transaction = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(authority, tx),
        (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
        (tx) => appendTransactionMessageInstructions(instructions, tx),
        (tx) => signTransactionMessageWithSigners(tx),
      );
      assertIsTransactionWithBlockhashLifetime(transaction);
      await sendAndConfirm(transaction, { commitment: "confirmed" });

      writeFileSync(
        new URL("./sprout-mint.json", import.meta.url),
        JSON.stringify({ mint: mint.address, name: NAME, symbol: SYMBOL, uri: URI }, null, 2),
      );
      console.log(`SPROUT mint with native metadata: ${mint.address}`);
      console.log(`tx: ${getSignatureFromTransaction(transaction)}`);
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    Uma nota de honestidade antes de você rodar: este script carrega só a preocupação de metadados, para o exemplo trabalhado continuar legível, e o mint que ele cria é um build de referência, não o R3. O mint composto, taxas mais exibição mais um nome, é o R3 de verdade que os módulos posteriores querem dizer quando dizem "o mint SPROUT," e o passo 7 torna essa composição obrigatória e a coloca atrás de um critério. (O badge e a tesouraria de m02-l3 ficam separados de propósito: um mint soulbound não pode ser a moeda negociável.) (Eu mantenho uma versão só-de-metadados por perto de todo jeito. Quando uma releitura se comporta mal meses depois, a reprodução mínima é a ferramenta de debugging que você vai desejar ter tido.)

4. **Rode.**

    ```bash
    npx tsx labs/m02-l4/add-metadata.ts
    ```

    Formato esperado da saída, os seus endereços vão ser diferentes:

    ```
    SPROUT mint with native metadata: 6NDNZ8kGXJbwg7JHyz8advCmivmoUEcEuRAVAUoWvBSY
    tx: wCgwQMAAc6azAp7jq9nbgMTpEYrAyPkxMgSi9X8vcayw8iBVp5t9ZRURn6FwT1EgE8oWpZN42YVtdh99SytaM3g
    ```

    O script também deixa `labs/m02-l4/sprout-mint.json`, o repasse de endereço que o script de verificação e os módulos posteriores leem. Trate esse arquivo como provisório por ora: o passo 7 o re-aponta para o mint composto, e é o endereço composto que os módulos posteriores precisam encontrar ali.

5. **Problema de completion: a escrita do campo.** O SPROUT precisa do campo `harvest_season` dele, e essa instrução é sua para construir. O que você sabe: o builder é o `getUpdateTokenMetadataFieldInstruction` do mesmo pacote, a entrada dele recebe `metadata` (o endereço do mint), `updateAuthority` (um signer, o nosso), um `field`, e um `value`. Chaves customizadas são expressas com o helper `tokenMetadataField`. Adicione o import, construa a instrução, coloque ela depois do initialize de metadados no array, e rode de novo. Escreva ela antes de seguir lendo.

    Pronto? Aqui está a checagem:

    ```typescript
    import { getUpdateTokenMetadataFieldInstruction, tokenMetadataField } from "@solana-program/token-2022";

    // ... appended after getInitializeTokenMetadataInstruction(...) in `instructions`:
    getUpdateTokenMetadataFieldInstruction({
      metadata: mint.address,
      updateAuthority: authority,
      field: tokenMetadataField("Key", ["harvest_season"]),
      value: "spring",
    }),
    ```

    Repare que a mesma instrução com `tokenMetadataField("Name")` edita o nome em si; inicializar-e-depois-atualizar é a API de escrita inteira, quatro instruções no total na interface (initialize, update field, remove key, update authority). Toda escrita pode crescer o TLV, que é por que o dimensionamento fantasma do passo 3 já incluía este par: o rent estava em depósito antes de o campo existir.

6. **Releia.** O critério para o R3 é uma releitura que faz assert, não um log de console que você olha. Crie `labs/m02-l4/verify-metadata.ts`; este arquivo é infraestrutura do curso, os módulos posteriores rodam ele exatamente por este caminho, então pegue ele completo:

    ```typescript
    import { address, createSolanaRpc } from "@solana/kit";
    import { fetchMint } from "@solana-program/token-2022";
    import { readFileSync } from "node:fs";
    import assert from "node:assert/strict";

    const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";

    async function main() {
      const saved = JSON.parse(
        readFileSync(new URL("./sprout-mint.json", import.meta.url), "utf8"),
      ) as { mint: string; name: string; symbol: string; uri: string };
      const mintAddress = address(process.argv[2] ?? saved.mint);

      const rpc = createSolanaRpc(RPC_URL);
      const mint = await fetchMint(rpc, mintAddress);

      assert(mint.data.extensions.__option === "Some", "mint carries no TLV extensions");
      const extensions = mint.data.extensions.value;

      const pointer = extensions.find((e) => e.__kind === "MetadataPointer");
      assert(pointer, "MetadataPointer extension missing");
      assert(
        pointer.metadataAddress.__option === "Some" &&
          pointer.metadataAddress.value === mintAddress,
        "MetadataPointer is not self-referential",
      );

      const metadata = extensions.find((e) => e.__kind === "TokenMetadata");
      assert(metadata, "TokenMetadata TLV missing");
      assert.equal(metadata.mint, mintAddress, "TLV mint field does not match this mint");
      assert.equal(metadata.name, saved.name);
      assert.equal(metadata.symbol, saved.symbol);
      assert.equal(metadata.uri, saved.uri);

      console.log(`OK: MetadataPointer -> ${pointer.metadataAddress.value} (the mint itself)`);
      console.log(
        `OK: TokenMetadata TLV reads back name=${metadata.name} symbol=${metadata.symbol} uri=${metadata.uri}`,
      );
      for (const [key, value] of metadata.additionalMetadata) {
        console.log(`OK: additional_metadata ${key}=${value}`);
      }
    }

    main().catch((err) => {
      console.error(err);
      process.exit(1);
    });
    ```

    ```bash
    npx tsx labs/m02-l4/verify-metadata.ts
    ```

    ```
    OK: MetadataPointer -> 6NDNZ8kGXJbwg7JHyz8advCmivmoUEcEuRAVAUoWvBSY (the mint itself)
    OK: TokenMetadata TLV reads back name=SPROUT symbol=SPRT uri=https://overgrowth.example/sprout.json
    OK: additional_metadata harvest_season=spring
    ```

    As duas asserções dos dois lados do loop antifalsificação acabaram de rodar: o ponteiro resolve para o mint, e o próprio campo `mint` do TLV aponta de volta. Repare que o script recebe um argumento de endereço opcional; `npx tsx labs/m02-l4/verify-metadata.ts <any mint>` agora é um verificador de metadados nativos de propósito geral. Aponte ele para o PYUSD.

7. **Componha no SPROUT de verdade, e re-aponte o repasse.** O mint só de metadados provou a ligação; o R3 é o mint composto, então faça a composição agora. Abra `labs/m02-l1/verify-economics.ts`, o script que constrói o SPROUT de taxa-e-exibição, carregue as constantes `NAME`/`SYMBOL`/`URI` para lá, e faça quatro mudanças, cada uma delas um empréstimo de `add-metadata.ts`:

    - Construa a mesma extensão `metadataPointer` (autorreferencial, exatamente como o passo 3 fez, com `payer.address` como autoridade dela) e a extensão `tokenMetadata` fantasma, e então dimensione a conta duas vezes: aloque em `getMintSize([transferFeeExtension, interestExtension, metadataPointer])` e financie em `getMintSize([transferFeeExtension, interestExtension, metadataPointer, tokenMetadata])`, passando o tamanho financiado para `getMinimumBalanceForRentExemption` e o tamanho alocado como `space`.
    - Encaixe o `getInitializeMetadataPointerInstruction` junto com os outros inicializadores de extensão, ANTES do `getInitializeMintInstruction`; a regra do tempo de criação não mudou.
    - Acrescente o `getInitializeTokenMetadataInstruction` e a sua escrita de campo do passo 5 DEPOIS do `getInitializeMintInstruction`.
    - No fim da `main()`, escreva o repasse apontado para a pasta desta lição, para os consumidores posteriores lerem o endereço composto:

    ```typescript
    writeFileSync(
      new URL("../m02-l4/sprout-mint.json", import.meta.url),
      JSON.stringify({ mint: mint.address, name: NAME, symbol: SYMBOL, uri: URI }, null, 2),
    );
    ```

    Rode de novo `npx tsx labs/m02-l1/verify-economics.ts` (todas as asserções de taxa dele precisam continuar verdes: a composição muda a identidade do mint, não a economia dele), e então rode o critério contra o mint composto: `npx tsx labs/m02-l4/verify-metadata.ts`. As três linhas OK precisam valer agora contra o endereço composto. Até valerem, o R3 não está completo.

8. **Feche o loop com o R1.** Aponte o seu inspetor `decode-mint` de m01-l2 para o mint composto. Duas linhas novas aparecem na caminhada de extensões dele: tipo 18 (MetadataPointer, 64 bytes de valor TLV) e tipo 19 (TokenMetadata, comprimento variável). As strings que você acabou de escrever estão sentadas dentro de bytes pelos quais o seu próprio decodificador consegue caminhar desde o módulo 1; o `fetchMint` é uma conveniência sobre exatamente essa caminhada, nada mais. E rode o `check-combo` no conjunto completo pelo bem do ritual: o MetadataPointer não conflita com nada na matriz.

![Fluxograma em hub das camadas de economia e de metadados do mint SPROUT completo, com provas em mints companheiros e consumidores a jusante nos módulos de hook, roteabilidade e roteamento de taxas.](assets/v08-flowchart.png)

## Challenge

Solo, sem apoio: o exercício de metadata-pointer. Crie um mint descartável novo, quaisquer decimals, cujo conjunto de extensões é exatamente um MetadataPointer autorreferencial, escreva um TLV TokenMetadata com um nome, um símbolo e uma URI da sua escolha mais pelo menos um par `additional_metadata`, e então escreva a asserção de ida e volta você mesmo: faça fetch do mint e faça assert de que o ponteiro resolve para o mint, e de que todo campo relê exatamente como foi escrito, caractere por caractere. Sem copiar `verify-metadata.ts`; o ponto é que o assert more nos seus dedos, porque um ida e volta que você consegue escrever do zero é o teste que você vai de fato buscar quando um mint de mainnet se comportar mal.

Aceito quando: um script, uma rodada, a asserção do ponteiro e toda asserção de campo passam, e matar qualquer um dos campos na escrita faz a asserção correspondente falhar (prove isso uma vez quebrando o símbolo de propósito).

## Checkpoint

O critério: `npx tsx labs/m02-l4/verify-metadata.ts` imprime as três linhas OK contra o mint COMPOSTO que o `sprout-mint.json` agora nomeia, o build do passo 7 carregando taxas, uma extensão de exibição, um ponteiro autorreferencial e o TokenMetadata em uma conta, e o ida e volta de mint novo do challenge passa com as suas próprias asserções. Com isso verde, o SPROUT (o R3) está completo para o resto do curso; uma rodada que passa contra o mint só de metadados do passo 3 sozinho não conta.

Os dois erros que eu espero, para você se autodiagnosticar rápido. Primeiro, ordenação: coloque o initialize do ponteiro depois do `InitializeMint`, ou tente adicionar metadados a um mint criado sem o ponteiro, e o programa te rejeita; extensões de MINT são de tempo de criação, o TLV de metadados é a exceção pós-init que você está usando (os TLVs de grupo do mapa mental dividem o mesmo caminho de ponteiro-e-depois-realloc, e extensões de conta como o MemoTransfer e o CpiGuard que você habilitou na lição passada têm o próprio caminho pós-criação, via a dança do reallocate), e só funciona porque o ponteiro estava lá primeiro. Segundo, financiamento: aloque E financie em 234 e a instrução quatro morre no meio da transação por rent; releia as linhas de dimensionamento fantasma do passo 3, o depósito tem que cobrir o tamanho pós-realloc. Se a sua falha não é nenhuma dessas, rode o script de verificação contra a minha ordem de falha: extensões presentes em geral, alvo do ponteiro, e então campos, e leve o primeiro assert que dispara para a discussão do curso com a sua lista de instruções.

Pegue o marco por um segundo, ele custou quatro lições: um mint que cobra e faz harvest de taxas, impõe as autoridades dele, deixa contas de holder recusarem o que deveriam recusar, e agora renderiza como SPROUT em qualquer coisa que leia o TLV. Esse é um ativo Token-2022 com formato de produção, o mesmo padrão que uma stablecoin atestada pela KPMG entrega, e você construiu cada byte dele a partir de instruções brutas.

No próximo módulo, a extensão que tudo até aqui vem te preparando para encontrar: a que roda O SEU código em toda transferência, uma por uma. É o único lugar neste curso onde você escreve um programa Rust seu, o hook de harvest. Uma nota honesta de logística para a emenda não te surpreender: o TransferHook é uma extensão de mint de tempo de criação, então o SPROUT terminado de hoje nunca pode crescer um. No próximo módulo você cunha uma variante com hook fresca, e ela é deliberadamente mínima, TransferHook e nada mais, porque o assunto daquele módulo é o hook, não a pilha de receitas; os dois mints vivem lado a lado do jeito que um token principal e a variante de teste dele com acesso restrito vivem em produção. Traga o kit de ferramentas. Vejo você no hook.
