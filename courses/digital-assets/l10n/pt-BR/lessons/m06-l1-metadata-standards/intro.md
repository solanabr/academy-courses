# Os padrões de metadados que as carteiras de fato leem

## Resumo

No m05-l2 você finalizou o conjunto de extensões de launch-venue do SPROUT e o relatório de roteabilidade R6; a metade fungível da economia do Overgrowth está decidida, negociável e em conformidade, com seus metadados escritos como TLV nativo do Token-2022 no m02-l4. Esta lição abre a metade NFT com um mapa, não com uma cunhagem: onde o nome, a imagem e os traits de um ativo de fato vivem. A resposta são três camadas, numeradas uma vez para que você sempre possa perguntar em que camada um campo vive: a camada 1 é o documento JSON off-chain a partir do qual uma carteira renderiza, a camada 2 é a pequena struct on-chain cujo ponteiro guarda o endereço desse JSON, e a camada 3, só em mints Token-2022, é o TLV nativo que você já ligou na mão. Você vai buscar um NFT famoso, imprimir o registro on-chain dele ao lado do JSON off-chain, pegar os dois discordando sobre o royalty, e terminar capaz de colocar qualquer campo de qualquer ativo Solana na sua camada. O recuo da ajuda: os dois scripts rodam por completo como fornecidos, a tabela de localização de campos é sua para preencher no meio do lab, e o challenge é totalmente solo contra um ativo que eu não escolho.

Abra a sua carteira e olhe qualquer NFT. Um nome, uma figura, talvez uma lista de traits e um royalty na página do marketplace. Parece um objeto só. Não é, e não é nem um lugar só: parte dessa tela é bytes em uma conta on-chain, a maior parte é um blob JSON em uma URI para a qual a conta apenas aponta, e em um mint Token-2022, parte dela é TLV que você já sabe escrever. Antes de eu explicar uma única camada, vá olhar as emendas você mesmo.

Nenhuma toolchain nova hoje. No workspace do seu curso, crie `fetch-asset.ts` a partir do lab abaixo e rode:

```bash
npx tsx fetch-asset.ts F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk
```

Esse endereço é o mint do Mad Lads #8420, um dos NFTs mais famosos da Solana. Aqui está o que voltou quando eu rodei enquanto escrevia isto, em 2026-08-23:

```text
=== ON-CHAIN (the Data struct, decoded from the metadata PDA) ===
account:                DZAZ3mGuq7nCYGzUyw4MiA74ysr15EfqLpzCzX2cRVng (owner: metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s)
name:                   "Mad Lads #8420"
symbol:                 "MAD"
uri:                    https://madlads.s3.us-west-2.amazonaws.com/json/8420.json
seller_fee_basis_points: 420
creator:                5XvhfmRjwXkGp3jHGmaKpqeerNYjkuZZBYLVQYdeVcRv verified=true share=0
creator:                2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW verified=true share=100

=== OFF-CHAIN (the JSON document at that uri) ===
name:        "Mad Lads #8420"
symbol:      "MAD"
description: "Fock it."...
image:       https://madlads.s3.us-west-2.amazonaws.com/images/8420.png
attributes:  7 traits, e.g. {"trait_type":"Gender","value":"Male"}
properties.files: 2 file(s)
properties.category: image
seller_fee_basis_points in JSON: 500
top-level keys: name, description, symbol, image, external_url, seller_fee_basis_points, attributes, properties
```

Fique um minuto com essa saída, porque a lição inteira está nela. A conta on-chain guarda cinco coisas e nenhuma figura. A figura, os traits, a descrição, tudo vive em um arquivo JSON na `uri`. O royalty aparece duas vezes, e as duas cópias discordam: 420 basis points on-chain, 500 no JSON. E a URI de uma das coleções mais valiosas da Solana aponta para um bucket S3 da Amazon. Cada uma dessas observações se torna uma seção desta lição.

## Onde um ativo de fato vive

Aqui está o colapso que desmistifica a pilha inteira: um registro de ativo on-chain é só uma struct com um ponteiro, e o conteúdo rico é um documento JSON nesse ponteiro. Essa é a arquitetura inteira. Tudo depois desse colapso é nomear os campos da struct, nomear os campos do JSON, e fazer a única pergunta que o colapso força: o que acontece quando o ponteiro sobrevive à coisa para a qual ele aponta?

Por que construir assim, afinal? Leve a alternativa ingênua até o fim primeiro. Suponha que você guardasse a imagem on-chain. Um PNG da qualidade do Mad Lad ocupa algumas centenas de kilobytes; bytes on-chain custam rent por byte, uma conta tem teto de 10 MiB, e cada byte dela é replicado para todo validador para sempre. Você estaria pagando preços de armazenamento de nível validador, em milhares de máquinas, por uma figura que nunca muda e é lida por uma carteira de cada vez. Então ninguém faz isso. A chain guarda o que a chain faz bem, fatos pequenos e autenticados: quem fez isto, como se chama, onde está o resto. O resto vive onde conteúdo volumoso vive, atrás de uma URI. Cunhagens baratas, conteúdo rico, e um novo modo de falha que vamos nomear honestamente antes do lab.

![Mapa de três camadas de um ativo Solana numeradas de um a três, o JSON off-chain, a struct Data on-chain cuja uri aponta para ele, e o TLV nativo do Token-2022.](assets/v01-diagram.webp)

### Camada 1: o JSON off-chain a partir do qual a carteira renderiza

Comece pela camada que preenche a maior parte da tela. O documento off-chain segue o padrão JSON do Token Metadata, e os campos que você viu na saída do Mad Lads são o núcleo do padrão. Caminhe por eles um por um, porque uma carteira caminha por eles também:

- **`name`** e **`symbol`**: strings de exibição. Note que elas também existem on-chain; as cópias no JSON são o que a maioria das carteiras de fato renderiza, e manter as duas em sincronia é uma norma, não uma regra.
- **`description`**: texto livre, renderizado em páginas de detalhe. A do Mad Lads tem duas palavras.
- **`image`**: a URI da arte. Este é o campo de que uma grade de carteira é feita. Uma `image` morta é um quadrado vazio.
- **`animation_url`**: URI opcional para vídeo, áudio, 3D, ou um build interativo. Carteiras que a suportam renderizam isto em vez da imagem estática.
- **`external_url`**: um link para fora, para um site. Pura convenção, muitas vezes desatualizada, nunca estrutural.
- **`attributes`**: a lista de traits, um array de pares `{ "trait_type": ..., "value": ... }`. O Mad Lads #8420 carrega sete, começando com `{"trait_type":"Gender","value":"Male"}`. Os marketplaces constroem suas ferramentas de raridade inteiramente a partir desse array.
- **`properties.files`**: um array de entradas `{ uri, type }` (mais uma flag `cdn` opcional) listando todo arquivo que compõe o ativo, tipicamente a imagem de novo mais alternativas. O `type` é um tipo MIME; as carteiras caem no chute pela extensão quando ele falta.
- **`properties.category`**: uma palavra dizendo aos renderizadores que tipo de ativo é este: `image`, `video`, `audio`, `vr`, ou `html`.

O padrão fungível é o subconjunto mínimo do mesmo documento: `name`, `symbol`, `description`, `image`. Um token como o USDC precisa de um logo e de um nome, não de uma lista de traits. Mesma família de schema, menos campos, que é por que uma convenção JSON serve às duas metades do mundo dos ativos.

Duas coisas que você viu no documento real merecem suspeita. Primeiro, `seller_fee_basis_points` aparece no JSON do Mad Lads em 500. Esse é um campo legado: o royalty foi para on-chain anos atrás, e uma cópia no JSON é o que quem subiu o arquivo por acaso escreveu no dia do upload. O 420 on-chain é o que os marketplaces leem; o 500 do JSON é um fóssil. Quando duas camadas discordam, a camada on-chain é a que tem uma autoridade de atualização e um timestamp, e a cópia off-chain apodrece em silêncio. Segundo, onde está o schema em si? A URL canônica do schema, https://schema.metaplex.com/nft1.0.json, é aquela em que o ecossistema se padronizou. Eu a sondei enquanto escrevia esta lição, 2026-08-23: o host não resolve mais, de jeito nenhum. O registro de DNS se foi. O padrão não mudou, mas o endereço de referência canônico dele está morto, então esta lição carrega o contrato de campos em prosa e no script validador do lab em vez de te jogar para dentro de um vazio.

Essa URL morta não é um acidente isolado, e vale trinta segundos de história porque explica por que tanto do que você meio lembra sobre metadados de NFT está uma geração atrás. A educação oficial da Solana congelou no meio da trama: o repositório solana-foundation/developer-content, a fonte por trás dos cursos oficiais, foi arquivado como somente leitura em 2025-01-24. Todo curso oficial antecede a pilha de NFT atual. Os próprios docs da Metaplex mudaram de domínio, e o antigo developers.metaplex.com agora faz 308-redirect para metaplex.com/docs, deixando anos de links de tutorial a um redirect de distância do conteúdo deles. O padrão que você está aprendendo hoje é estável; as URLs em volta dele não são. Reverifique qualquer citação de metadados antes de confiar nela, inclusive, em cinco anos, nesta.

![Linha do tempo do padrão Token Metadata de 2021, passando pelo arquivamento da educação oficial da Solana em 2025-01-24, até 2026, quando o host canônico do schema está morto e o próprio Token Metadata é legado.](assets/v02-timeline.webp)

### Camada 2: a struct Data on-chain, cinco campos e um ponteiro

Agora a camada pequena, a que o seu script decodificou na mão. Para um ativo legado, o registro on-chain é uma conta derivada do mint: o PDA de metadados, semeado com a string literal `"metadata"`, o id do programa Token Metadata, e o endereço do mint, de propriedade do programa Token Metadata em `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`. O seu script o derivou, o leu, e pulou 65 bytes de header (uma chave de conta de um byte, a autoridade de atualização de 32 bytes, o mint de 32 bytes) para chegar à parte que o padrão chama de struct `Data`:

```text
Data {
  name:                    String   // Borsh: u32 length + bytes, stored at fixed capacity, null-padded
  symbol:                  String
  uri:                     String   // the pointer to layer 1
  seller_fee_basis_points: u16      // 420 for Mad Lads = 4.20%
  creators:                Option<Vec<Creator { address, verified, share }>>
}
```

Cinco campos. Essa é a identidade on-chain inteira de um NFT legado, e agora a saída que você recebeu no começo da lição deve ler diferente: a conta nunca estava "sem" a imagem. A imagem nunca deveria estar ali. Um colega que busca a conta de um NFT, não encontra traits nem figura, e concluiu que o ativo está quebrado leu a arquitetura errado; nada está guardado onde ele olhou, por design, e o campo `uri` era a resposta parada no próprio dump dele.

Cada campo merece uma frase de respeito. `name` e `symbol` são as cópias on-chain-autoritativas das strings de exibição, guardadas com preenchimento nulo em capacidade fixa, que é por que o seu decodificador corta os zeros do fim. `uri` são os 200 bytes mais estruturais do ativo: é a única ponte entre o que a chain autentica e o que a carteira mostra. `creators` carrega até cinco endereços com uma flag `verified` cada, e essa flag é superfície de segurança de verdade: ela só pode ser posta como true por aquele creator de fato assinando, que é como os marketplaces distinguem o verdadeiro criador da coleção de um copiador que colou o mesmo endereço sem verificação. A sua saída do Mad Lads mostra os dois creators verificados, o primeiro com share 0 (um endereço que assina pela coleção) e o segundo com share 100 (para onde os royalties deveriam ir).

E `seller_fee_basis_points`, o campo que discordou do JSON. On-chain diz 420, e on-chain ganha. Mas agora que você confia na cópia certa, aqui está a cilada mais profunda: não leia nem a cópia vencedora como um royalty garantido. É uma preferência declarada, indicativa apenas, e nada no token program impõe uma taxa na hora da transferência. Como a imposição foi enxertada depois, e com que profundidade ela falhou, é a história do m06-l3, provada em vez de afirmada. Por hoje, calibre: este u16 é o que os marketplaces escolhem honrar, não o que eles têm que honrar.

![Comparação anotada lado a lado da struct Data on-chain do Mad Lads #8420 e do JSON off-chain, em que image e traits existem só off-chain e o royalty lê 420 on-chain mas 500 off-chain.](assets/v03-annotated-code.webp)

### Camada 3: o caminho nativo do Token-2022 que você já construiu

Você não acabou de aprender um terceiro sistema de metadados no m02-l4. Você construiu um. O nome do SPROUT não vive em nenhuma conta da Metaplex: ele vive nos bytes do próprio mint, como uma entrada TLV TokenMetadata (discriminador `[112,132,90,90,11,88,157,87]`) carregando `name`, `symbol`, `uri`, e os pares chave-valor de `additional_metadata` em que você escreveu `harvest_season = "spring"`. Ao lado dela fica a extensão MetadataPointer, que você apontou para o próprio mint. Então coloque essa construção inteira no mapa de hoje: os metadados nativos do Token-2022 são as camadas 1 e 2 colapsadas dentro da conta do mint, com a mesma convenção JSON ainda disponível no fim do campo `uri` dele para qualquer coisa rica.

A comparação contra o layout da Metaplex é onde o design ganha o seu lugar. No modelo legado, a identidade vive em uma conta separada que um programa diferente possui, e um leitor precisa derivar o PDA para encontrá-la. No modelo nativo não há nada para derivar e nada separado para buscar: um `getAccountInfo` no mint devolve identidade, supply e toda extensão em uma leitura só. E o argumento anti-spoofing do m02-l4 encaixa no vocabulário de hoje de forma limpa: um MetadataPointer apontado para qualquer lugar que não seja o próprio mint reintroduz uma indireção que um atacante pode apontar para a conta de metadados de outra pessoa, que é por que autorreferencial é o layout que você ligou e o único que você deveria entregar. Os trade-offs correm para o outro lado também, e nomeá-los é o ponto de um mapa. Metadados TLV nativos vivem no mint, então cada campo que você adiciona faz a conta e o rent dela crescerem, e o mecanismo inteiro existe só em mints Token-2022. Mints SPL clássicos, ou seja a maioria dos ativos já soltos por aí, não podem carregá-lo, que é por que as camadas da Metaplex não vão a lugar nenhum e por que você precisa de todas as três colunas da tabela que você está a ponto de preencher.

![Comparação do modelo de conta de metadados separada da Metaplex e do modelo TLV dentro do mint do Token-2022 em localização da identidade, número de leituras, superfície de spoofing, crescimento do rent, disponibilidade por programa, e o padrão JSON off-chain compartilhado.](assets/v04-comparison.webp)

### O ponteiro é a junta fraca: a realidade do armazenamento

Toda camada acima termina em uma `uri`, então a durabilidade do ativo inteiro se reduz a uma pergunta: por quanto tempo essa URI continua resolvendo? A conta on-chain não garante nada sobre isso. O rent mantém a struct viva para sempre; a struct vai felizmente apontar para um 404 para sempre também. Um link morto ou um arquivo trocado em silêncio significa que o "NFT" resolve para nada, ou pior, para outra coisa, enquanto a chain segue atestando que o ponteiro está exatamente onde sempre esteve.

O que nos traz de volta à linha mais silenciosamente alarmante da sua saída de sondagem: `madlads.s3.us-west-2.amazonaws.com`. O Mad Lads, uma coleção emblemática da Solana, serve seus metadados e imagens de um bucket S3 da Amazon. O S3 é rápido, barato e mutável, e ele persiste precisamente pelo tempo em que alguém continuar pagando a conta e controlar o bucket. Isso não é um escândalo; é uma decisão de norma tomada em público, e ativos populares são arquivados e espelhados na prática. Mas veja pelo que é: o conteúdo do ativo é alugado, e quem aluga é o time, não você.

A alternativa que põe permanência em primeiro lugar é a família Arweave. O modelo do Arweave é pague uma vez, guarde para sempre, financiado por um mecanismo de dotação em vez de uma assinatura, e URIs do Arweave são difundidas pelas coleções da Solana exatamente por isso. O Irys é o uploader na frente dele que a CLI da Metaplex documenta como o seu default, então o caminho de menor resistência do ferramental já deixa o seu JSON em armazenamento permanente. O IPFS merece uma ressalva honesta: URIs endereçadas por conteúdo são um upgrade de integridade de verdade, já que o hash na URI é o conteúdo, mas a disponibilidade depende de alguém continuar pinando o arquivo, e eu não verifiquei nesta rodada quão saudável está o mercado comercial de pinning. Então a regra honesta, a que se deve carregar para fora desta lição: a URI é permanente só se o armazenamento for.

Há uma segunda decisão de norma escondida ao lado do armazenamento: a mutabilidade. O PDA de metadados tem uma autoridade de atualização, e o TLV TokenMetadata tem uma também; qualquer uma pode reescrever a `uri` ou os campos amanhã, a menos que essa autoridade seja abandonada. Metadados mutáveis são como um rug troca a arte depois da cunhagem, e são também como um jogo legítimo evolui um item, corrige um typo, ou migra de host. Imutável-mais-permanente é a postura de nível colecionador; mutável-mais-alugado é a postura de serviço ao vivo. Nenhuma das duas é um default. É uma escolha que você vai fazer explicitamente, por classe de ativo, quando o Overgrowth cunhar o Almanac na próxima lição.

![Fluxograma do endereço do mint passando pelo registro on-chain, uri, documento JSON e imagem, com pontos de ruptura no host da uri, no JSON mutável, e no link da imagem que a chain nunca detecta.](assets/v05-flowchart.webp)

## Lab: localize todo campo de um ativo real

Rodadas guiadas mais um entregável que você mesmo preenche. Você vai rodar o script de fetch direito, vê-lo falhar corretamente no SPROUT, validar um JSON real, e produzir a tabela de localização de campos que é o critério desta lição. Cerca de vinte e cinco minutos.

1. **Fixe o workspace.** O mesmo workspace de curso de todo lab TS. Se você estiver recriando do zero:

   ```bash
   npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
   ```

   Nota de atualidade, verificada contra o npm em 2026-09-05: a tag `latest` do kit agora é 8.2.0, e o workspace do curso fica fixado em 7.1.1 porque esse é o major do kit contra o qual o cliente `@solana-program/token-2022@0.15.0` dele faz peer (^7; a linha 0.16 passou para ^8). Os scripts de hoje também usam o `fetch` embutido, então Node 20 ou mais novo (o piso do curso desde o m01-l1), e `npx tsx` para rodar TypeScript direto (ele se instala na primeira chamada; o `package.json` do workspace carrega `"type": "module"` para que `await` de nível superior funcione).

2. **Crie `fetch-asset.ts`.** Este é o script fornecido, por inteiro. A única maquinaria nova desde o m01-l2 é a derivação de PDA no topo, então é ela que ganha o orçamento de comentários:

   ```typescript
   import { createSolanaRpc, address, getProgramDerivedAddress, getAddressEncoder, getAddressDecoder } from '@solana/kit';

   const TOKEN_METADATA_PROGRAM = address('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
   const mint = address(process.argv[2] ?? 'F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk');
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.mainnet-beta.solana.com');

   // 1. Derive the metadata PDA: ["metadata", program id, mint], owned by Token Metadata.
   const addressEncoder = getAddressEncoder();
   const [metadataPda] = await getProgramDerivedAddress({
     programAddress: TOKEN_METADATA_PROGRAM,
     seeds: [
       new TextEncoder().encode('metadata'),
       addressEncoder.encode(TOKEN_METADATA_PROGRAM),
       addressEncoder.encode(mint),
     ],
   });

   // 2. Read the on-chain account and hand-decode the Data struct (Borsh).
   const { value: account } = await rpc.getAccountInfo(metadataPda, { encoding: 'base64' }).send();
   if (!account) throw new Error(`No metadata account at ${metadataPda}. Not a Token Metadata asset.`);

   const data = Buffer.from(account.data[0], 'base64');
   let offset = 1 + 32 + 32; // key (1) + update_authority (32) + mint (32)

   const readString = (): string => {
     const len = data.readUInt32LE(offset);
     offset += 4;
     const raw = data.subarray(offset, offset + len);
     offset += len;
     return raw.toString('utf8').replace(/\0+$/, ''); // strings are stored at fixed capacity, null-padded
   };

   const name = readString();
   const symbol = readString();
   const uri = readString();
   const sellerFeeBasisPoints = data.readUInt16LE(offset);
   offset += 2;

   const addressDecoder = getAddressDecoder();
   const creators: { address: string; verified: boolean; share: number }[] = [];
   if (data[offset] === 1) { // Option<Vec<Creator>> tag
     offset += 1;
     const count = data.readUInt32LE(offset);
     offset += 4;
     for (let i = 0; i < count; i++) {
       creators.push({
         address: addressDecoder.decode(data.subarray(offset, offset + 32)),
         verified: data[offset + 32] === 1,
         share: data[offset + 33],
       });
       offset += 34;
     }
   } else {
     offset += 1;
   }

   console.log('=== ON-CHAIN (the Data struct, decoded from the metadata PDA) ===');
   console.log(`account:                ${metadataPda} (owner: ${account.owner})`);
   console.log(`name:                   ${JSON.stringify(name)}`);
   console.log(`symbol:                 ${JSON.stringify(symbol)}`);
   console.log(`uri:                    ${uri}`);
   console.log(`seller_fee_basis_points: ${sellerFeeBasisPoints}`);
   for (const c of creators) console.log(`creator:                ${c.address} verified=${c.verified} share=${c.share}`);

   // 3. Follow the pointer: fetch the off-chain JSON the uri points at.
   const json = await (await fetch(uri)).json();
   console.log('\n=== OFF-CHAIN (the JSON document at that uri) ===');
   console.log(`name:        ${JSON.stringify(json.name)}`);
   console.log(`symbol:      ${JSON.stringify(json.symbol)}`);
   console.log(`description: ${JSON.stringify(json.description?.slice(0, 60))}...`);
   console.log(`image:       ${json.image}`);
   console.log(`attributes:  ${json.attributes?.length ?? 0} traits, e.g. ${JSON.stringify(json.attributes?.[0])}`);
   console.log(`properties.files: ${json.properties?.files?.length ?? 0} file(s)`);
   console.log(`properties.category: ${json.properties?.category}`);
   console.log(`seller_fee_basis_points in JSON: ${json.seller_fee_basis_points}`);
   console.log(`top-level keys: ${Object.keys(json).join(', ')}`);
   ```

   Rode sem argumento para bater no Mad Lads #8420. Checkpoint: o seu bloco on-chain termina em dois creators com `verified=true` e `seller_fee_basis_points: 420`, e o seu bloco off-chain relata 7 traits e o 500 obsoleto do JSON. Se a leitura do RPC falhar, o endpoint público default limita taxa de forma agressiva; aponte `RPC_URL` para qualquer endpoint que você já use e rode de novo.

3. **Aponte para o SPROUT e veja falhar corretamente.** Rode o script de novo com o endereço do seu mint SPROUT como argumento. Checkpoint: ele lança `No metadata account at ...`. Esse erro é a lição: o SPROUT não tem PDA de metadados da Metaplex porque a identidade dele vive na camada 3, dentro do mint. Prove isso rodando de novo o seu script de leitura do m02-l4 (aquele do `fetchMint` que garantiu o ponteiro autorreferencial e imprimiu `harvest_season = spring`). Um ativo resolvido através de uma segunda conta derivada, um através do próprio TLV, mesma convenção JSON esperando no fim dos dois campos `uri`.

4. **Preencha a tabela de localização de campos.** Este é o entregável. Copie-a para as suas notas de curso e complete cada linha com sim/não por coluna, usando as suas duas saídas de sondagem mais as seções de teoria. As três primeiras linhas estão feitas como calibração:

   | Campo | conta on-chain (struct Data + header) | JSON off-chain | TLV do Token-2022 |
   |---|---|---|---|
   | name | sim | sim (cópia por convenção) | sim |
   | image | não | sim | não (via uri) |
   | seller_fee_basis_points | sim (indicativo) | fóssil legado | não |
   | symbol | | | |
   | uri | | | |
   | description | | | |
   | attributes / traits | | | |
   | animation_url | | | |
   | external_url | | | |
   | properties.files + category | | | |
   | creators + flag verified | | | |
   | pares additional_metadata | | | |
   | autoridade de atualização | | | |

   Uma linha explica o nome largo da coluna: a autoridade de atualização vive nos 65 bytes de header da conta que o seu decodificador deliberadamente pulou, não dentro da struct `Data` de cinco campos. Ainda é um fato on-chain sobre a conta, que é por que a coluna diz conta e não struct; responda essa linha para a conta como um todo.

5. **Valide um JSON contra o padrão.** Crie `validate-asset-json.ts`, também fornecido por inteiro. Já que o host canônico do schema se foi, o script É o schema, codificando o contrato de campos da camada 1:

   ```typescript
   // validate-asset-json.ts: check an off-chain asset JSON against the Token Metadata
   // JSON standard's field contract. Usage: npx tsx validate-asset-json.ts <uri>
   const CATEGORIES = ['image', 'video', 'audio', 'vr', 'html'];

   const uri = process.argv[2];
   if (!uri) throw new Error('usage: npx tsx validate-asset-json.ts <uri>');

   const json = await (await fetch(uri)).json();
   const failures: string[] = [];
   const warnings: string[] = [];

   // Required by the standard: the fields a wallet cannot render without.
   if (typeof json.name !== 'string' || json.name.length === 0) failures.push('name: missing or empty');
   if (typeof json.description !== 'string') failures.push('description: missing');
   if (typeof json.image !== 'string' || !/^(https?|ipfs|ar):/.test(json.image))
     failures.push('image: missing or not a resolvable URI');

   // Optional but shape-checked when present.
   if (json.symbol !== undefined && typeof json.symbol !== 'string') failures.push('symbol: not a string');
   if (json.animation_url !== undefined && typeof json.animation_url !== 'string')
     failures.push('animation_url: not a string');
   if (json.attributes !== undefined) {
     if (!Array.isArray(json.attributes)) failures.push('attributes: not an array');
     else
       json.attributes.forEach((a: unknown, i: number) => {
         const attr = a as { trait_type?: unknown; value?: unknown };
         if (typeof attr.trait_type !== 'string' || attr.value === undefined)
           failures.push(`attributes[${i}]: needs trait_type (string) and value`);
       });
   }
   if (json.properties?.files !== undefined) {
     if (!Array.isArray(json.properties.files)) failures.push('properties.files: not an array');
     else
       json.properties.files.forEach((f: { uri?: unknown; type?: unknown }, i: number) => {
         if (typeof f.uri !== 'string') failures.push(`properties.files[${i}].uri: missing`);
         if (typeof f.type !== 'string') warnings.push(`properties.files[${i}].type: missing (wallets guess from extension)`);
       });
   }
   if (json.properties?.category !== undefined && !CATEGORIES.includes(json.properties.category))
     warnings.push(`properties.category: "${json.properties.category}" is not one of ${CATEGORIES.join('/')}`);

   // Legacy fields the standard moved on-chain: presence is a staleness signal, not an error.
   if (json.seller_fee_basis_points !== undefined)
     warnings.push(`seller_fee_basis_points: legacy JSON field (${json.seller_fee_basis_points}); the on-chain Data struct's value is what marketplaces read`);
   if (json.collection !== undefined)
     warnings.push('collection: legacy JSON field; collection membership is verified on-chain, never from JSON');

   console.log(`verdict: ${failures.length === 0 ? 'PASS' : 'FAIL'}`);
   for (const f of failures) console.log(`  FAIL  ${f}`);
   for (const w of warnings) console.log(`  warn  ${w}`);
   ```

   Rode contra a URI do Mad Lads que o seu fetch imprimiu. Checkpoint:

   ```text
   verdict: PASS
     warn  seller_fee_basis_points: legacy JSON field (500); the on-chain Data struct's value is what marketplaces read
   ```

   Um pass com um aviso de fóssil, que é exatamente a aparência de uma coleção saudável de nove dígitos com ferramental da era de 2023.

![Espectro de opções de armazenamento de uri, de hospedagem web alugada e mutável, passando por IPFS pinado, até o Arweave financiado por dotação via Irys, com a mutabilidade dos metadados via autoridade de atualização como uma decisão ortogonal.](assets/v06-diagram.webp)

## Challenge

Totalmente solo, e é o critério de avaliação desta lição. Escolha um NFT Solana que eu não escolhi para você: um que você tenha, ou qualquer endereço de mint que você tire de um anúncio de marketplace. Rode o `fetch-asset.ts` contra ele (se ele lançar o erro de conta de metadados ausente, você encontrou um ativo Core, comprimido, ou Token-2022; anote qual e escolha um legado para este exercício. Ler esses outros layouts é para o que servem as duas próximas lições, mais a lição do módulo 7 sobre a DAS, a read API do Digital Asset Standard). Depois produza dois entregáveis nas suas notas de curso. Primeiro, a sua tabela de localização de campos completa do passo 4 do lab, com cada linha colocada. Segundo, rode o `validate-asset-json.ts` na URI do seu ativo e escreva um veredicto de três linhas: pass ou fail, a razão de qualquer falha ou aviso nas suas próprias palavras, e uma frase sobre a durabilidade do host da URI dado onde ele se senta no espectro de armazenamento. Se o seu ativo discordar de si mesmo entre camadas do jeito que o Mad Lads discorda, diga qual cópia ganha e por quê. Quando a sua tabela sobreviver a uma checagem contra as seções de teoria e o seu veredicto nomear o armazenamento atrás da URI, você tem o mapa sobre o qual este módulo constrói.

Duas das suas sondagens nesta lição devolveram algo que um tutorial não teria te mostrado: uma coleção emblemática em armazenamento alugado e um host canônico de schema morto. Se o seu próprio ativo do challenge revelou algo mais estranho, ou uma das minhas saídas de sondagem não bate mais com a sua, poste no canal de feedback do curso com o comando e a saída colados. Metadados são a camada em que a entropia do ecossistema aparece primeiro, e as sondagens dos leitores são como esta lição se mantém verdadeira.

Você mapeou onde o nome, a imagem e os traits de um ativo de fato vivem, através de todas as três camadas, e consegue colocar qualquer campo em segundos. Na próxima lição você para de ler ativos e começa a cunhá-los: Metaplex Core, o caminho de NFT recomendado em 2026, uma conta por ativo, e a coleção Almanac é criada, verificada e crescida para se tornar os primeiros NFTs do Overgrowth.
