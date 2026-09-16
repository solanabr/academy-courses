# Anatomia do mint & da conta a partir dos bytes brutos

Em m01-l1 você rodou um script pronto que decodificou as oito extensões do PYUSD e viu uma transferência clássica custar 76 CU. Você leu números que ainda não conseguia explicar. Pior, você aceitou a palavra do decodificador para tudo aquilo: o modo `jsonParsed` da RPC te entregou uma lista organizada de nomes de extensão, e você não tinha como conferir se ela estava dizendo a verdade. Isso é uma caixa-preta, e hoje a gente abre ela.

Comece olhando a matéria-prima com os próprios olhos. Nenhuma biblioteca ainda. Salve isto como `peek.ts` em qualquer lugar:

```typescript
// peek.ts - three bytes of PYUSD's mint, before any parser exists.
// Zero npm dependencies. Run: npx tsx@4.20.5 peek.ts
const RPC = "https://api.mainnet-beta.solana.com";
const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

async function peek() {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0", id: 1, method: "getAccountInfo",
      params: [PYUSD, { encoding: "base64" }],
    }),
  });
  const { result } = await res.json();
  const data = Buffer.from(result.value.data[0], "base64");

  console.log(`length:        ${data.length} bytes`);
  console.log(`byte 165:      ${data[165]}   (the account-type discriminator)`);
  console.log(`bytes 166-169: ${[...data.subarray(166, 170)].join(" ")}  (first TLV header)`);
}

// Wrapped in a function deliberately: a .ts file with no imports is a script,
// not a module, and top-level await is a module-only privilege.
peek();
```

```bash
npx tsx@4.20.5 peek.ts
# tsx pinned at 4.20.5, verified 2026-08-22; npx fetches it on first run.
```

Minha execução, hoje, 2026-08-22:

```text
length:        866 bytes
byte 165:      1   (the account-type discriminator)
bytes 166-169: 3 0 32 0  (first TLV header)
```

Os mesmos 866 bytes da lição passada, mas desta vez nada os parseou para você. Aquele `1` sentado no byte 165 e aquela sequencinha `3 0 32 0` são a anatomia inteira desta lição, e no fim dela você vai lê-los do jeito que você lê português. A diferença em relação à vez anterior é a diferença entre confiar num decodificador e ser dono da leitura: quando o seu inspetor imprime o conjunto exato de extensões de um mint que você nunca criou, calculado pela sua própria aritmética de cursor, ninguém mais consegue te enrolar.

## Resumo

Esta é uma lição de construção, e ela entrega a primeira ferramenta de verdade do kit Overgrowth: **R1, o inspetor `decode-mint`**. Dado um endereço de mint e uma RPC, ele imprime os campos base (autoridade de mint, supply como BigInt, decimals, autoridade de congelamento), informa se a conta é um mint simples de 82 bytes ou um mint estendido de 165 mais 1 byte, e enumera cada extensão TLV como `{name, type, length}`. Ele já vem com `test-decode-mint.ts`, um assert-script que decodifica um mint conhecido e fixado e falha ruidosamente em qualquer divergência. Esse script é o primeiro critério de aceitação do curso, e o padrão que ele estabelece, asserts simples contra um alvo fixado, é reaproveitado em cada degrau seguinte.

A teoria cobre toda a anatomia de bytes brutos sobre a qual o curso se apoia: o layout do mint simples de 82 bytes, por que mints estendidos preenchem até 165 bytes mais um byte discriminador, o formato da entrada TLV, a matemática de comprimento de conta por trás de `try_calculate_account_len`, e por que todo u64 neste curso é um BigInt. A ajuda recua conforme o cronograma: na lição passada você rodou scripts prontos; hoje eu te mostro a fatia base e a checagem de is-extended de ponta a ponta, mas o loop TLV é um TODO que você preenche sozinho. O challenge depois é totalmente solo. O próximo degrau é uma lição de conceito, guiada de ponta a ponta por design; o recuo é retomado em m01-l4, onde quatro das cinco regras do validador são suas para portar.

## A anatomia, a partir do offset zero

Os dados de uma conta são só um array de bytes. O programa que é dono da conta decide o que esses bytes significam, e para mints o significado está publicado no código-fonte do programa. Tudo abaixo é esse layout publicado, e nada nele é secreto ou esperto. É a planta baixa de um prédio em que você já mora há tempos.

### A base de 82 bytes

Um mint SPL clássico tem exatamente 82 bytes, e os dois programas de token usam os mesmos cinco campos na mesma ordem. Aqui está o mapa, offsets incluídos, porque você está prestes a escrever código contra eles:

![Um mapa de bytes horizontal do layout do mint de 82 bytes: uma autoridade de mint opcional de 36 bytes, um supply little-endian de 8 bytes, um byte de decimals, um byte de initialized e uma autoridade de congelamento opcional de 36 bytes.](assets/v01-diagram.webp)

Dois desses campos merecem um olhar mais de perto porque derrubam as pessoas.

**COption tem 36 bytes, não 33.** Uma pubkey opcional é serializada como uma tag u32 little-endian completa (0 para None, 1 para Some) seguida da chave de 32 bytes, que está sempre presente no buffer mesmo quando a tag diz None. Então a autoridade de mint ocupa os bytes 0 a 35, e a autoridade de congelamento ocupa 46 a 81. Se você internalizou o `Option` do Rust como um byte de tag, esse layout vai te pegar num off-by-three. O wire format gasta quatro bytes na tag.

**Supply é um u64, e u64 não cabe num number do JavaScript.** `Number.MAX_SAFE_INTEGER` é 2^53 menos 1, cerca de 9.0 quadrilhões; um u64 chega no teto perto de 18.4 quintilhões, três ordens de grandeza acima. Isso não é uma preocupação teórica que você pode adiar. Quando eu decodifiquei o mint do USDC clássico hoje, o supply dele marcou 7,923,463,957,481,104 unidades base, o que é cerca de 88 por cento do caminho até o maior inteiro que o JavaScript consegue representar com exatidão. Mais uma ordem de grandeza de crescimento de stablecoin, ou qualquer token de 9 decimals com um supply grande, e `Number(supply)` arredonda em silêncio. Em silêncio é a palavra que importa: nenhuma exceção, nenhum aviso, só um saldo errado em produção. Então a regra neste curso é absoluta e chata: **supply e amounts u64 são BigInt de ponta a ponta**, lidos com `DataView.getBigUint64`, impressos com o `n` ainda conceitualmente grudado, nunca passados por `Number`. Eu já entreguei a outra versão dessa decisão, anos atrás, num dashboard que exibia supplies de tokens. Funcionava em todo teste, porque supplies de teste são pequenos. É exatamente esse o tipo de bug que isto é.

![Uma comparação mostrando o supply do PYUSD em 7.6 por cento do limite de inteiro seguro do JavaScript, o do USDC em 88 por cento, e o máximo do u64 muito além dele, terminando com a regra de manter tudo em BigInt.](assets/v02-comparison.webp)

Essa é a base. Num mint clássico simples, essa também é a ponta final: o byte 82 é a borda da conta. O comprimento em si é o seu primeiro desvio de parser, e é uma resposta completa por conta própria. Uma conta que tem exatamente 82 bytes é um mint simples sem extensões, sem discriminador, sem região TLV, ponto final. Nada mais a checar.

### Por que mints estendidos recomeçam no byte 165

Agora a parte estranha. O mint do PYUSD tem 866 bytes, e a primeira extensão dele não começa no byte 82. Começa no byte 166. Entre a base e as extensões ficam 83 bytes de preenchimento com zeros e um byte misterioso. Por que um formato desperdiçaria 83 bytes por mint?

Por causa de uma decisão tomada anos antes de as extensões existirem. O programa de token clássico faz type-check das contas só pelo comprimento: um mint tem 82 bytes, uma conta de token tem 165, um multisig tem 355. Esse é o teste inteiro. Comprimento 82, trate os bytes como um mint; comprimento 165, trate como uma conta de token; 355, um multisig; qualquer outra coisa, rejeite. Barato, simples e completamente dependente de esses três números nunca colidirem.

As extensões quebram isso. Assim que as contas passam a ter caudas opcionais de comprimento variável, os comprimentos deixam de ser distintivos: algum mint estendido acabaria caindo em exatamente 165 bytes, e qualquer programa usando o teste de comprimento leria alegremente um mint como se fosse o saldo de token de alguém. Ler bytes errado numa fronteira de tipo é como fundos são roubados, então o Token-2022 fechou a porta estruturalmente. Toda conta estendida, mint ou conta de token, é preenchida para além da zona de colisão: os dados base primeiro, zeros até o offset 165, e então **um byte discriminador de tipo de conta no offset 165**. Valor 1 significa mint. Valor 2 significa conta de token. Seu `peek.ts` imprimiu exatamente esse `1`. Depois do discriminador, e só depois dele, a região de extensões começa no byte 166.

O custo desse design é honesto e visível: um mint estendido gasta 83 bytes em preenchimento, com rent pago em todos eles, puramente para que nenhum comprimento volte a ser ambíguo. A alternativa era um formato em que a confusão de tipos é possível e todo programa downstream carrega o fardo de nunca cometer o erro. Pagar 83 bytes uma vez, no nível do formato, para apagar uma classe inteira de bugs em todo lugar, é uma troca que os projetistas aceitaram sem hesitar, e, tendo passado um tempo com a literatura de exploits, eles estavam certos em aceitar.

Isso também resolve uma questão prática sobre as contas de token que você usa todo dia. Suas ATAs, as contas de token associadas que guardam seus saldos, são contas de token de 165 bytes no programa clássico. Uma conta de token do Token-2022 com qualquer extensão recebe o mesmo tratamento de um mint: preenchida (ela já está em 165), e aí o byte 165 carrega um 2 em vez de um 1. Mesmo slot de discriminador, valor diferente. Um byte, e mints e contas de token nunca mais podem ser confundidos, não importa o que as extensões façam com os comprimentos deles.

![Duas barras de bytes comparando um mint simples de 82 bytes com o mint de 866 bytes do PYUSD, cuja base idêntica é seguida de preenchimento, do discriminador de tipo de conta e de uma região TLV de 700 bytes.](assets/v03-diagram.webp)

### A caminhada TLV

Tudo do byte 166 até o fim da conta é uma sequência de **entradas TLV**: tipo, comprimento, valor, repetidos. Cada entrada é um código de tipo u16 little-endian dizendo qual extensão é essa, um comprimento u16 little-endian dizendo quantos bytes de valor vêm a seguir, e então exatamente essa quantidade de bytes de valor. Quatro bytes de header, e então o payload. A próxima entrada vem imediatamente depois. Sem separadores, sem campo de contagem na frente, sem índice. A lista é a caminhada.

A saída do seu `peek.ts` já continha um exemplo resolvido completo. Os quatro bytes em 166 eram `3 0 32 0`. Leia-os como dois u16 little-endian: tipo = 3, comprimento = 32. O tipo 3 é MintCloseAuthority, e o valor dele é uma pubkey de 32 bytes. Então os bytes 170 a 201 são essa autoridade, e o header da próxima entrada começa em 166 + 4 + 32 = 202. Em 202 você encontraria `12 0 32 0`: PermanentDelegate, outra pubkey de 32 bytes, próximo header em 238. E assim por diante, oito vezes, até a última entrada terminar exatamente no byte 866, a borda da conta. Quando seu cursor pousa precisamente no limite da conta sem sobrar nada, essa é a checagem de reconciliação que prova que sua caminhada leu cada byte.

![Uma tabela percorrendo as oito entradas TLV do PYUSD do byte 166 ao 866, mostrando o tipo, o nome e o comprimento de cada entrada, e a aritmética de cursor que pousa exatamente no limite da conta.](assets/v04-annotated-code.webp)

Olhe a coluna de tipo por um segundo, porque ela mata silenciosamente uma suposição tentadora. Os códigos vão 3, 12, 1, 4, 16, 14, 18, 19. Não estão ordenados. Entradas TLV aparecem na ordem em que o emissor as inicializou, não em ordem de tipo, então seu parser nunca pode fazer busca binária nem presumir posição. Você caminha, sempre.

Agora a cilada que garante a esta lição o lugar dela no curso, aquela que o brief e eu nos recusamos a deixar você aprender em produção. Quando você termina de ler uma entrada, você avança o cursor até o próximo header. O valor tem `length` bytes de comprimento, mas a entrada tem `4 + length` bytes, porque os próprios campos de tipo e de comprimento ocupam dois bytes cada. Avance só por `length` e o seu cursor pousa 4 bytes antes, no meio do valor que você acabou de ler. O próximo "tipo" que você lê são dois bytes da pubkey de alguma autoridade. O próximo "comprimento" são mais dois. Os dois parseiam sem problema, porque quaisquer dois bytes parseiam como um u16. Daí em diante, cada entrada que você decodifica é lixo que parece dado, e nada lança exceção. Uma constante errada, uma leitura silenciosamente corrompida de cada entrada seguinte. Esse é o imposto de fragilidade do parse à mão, e é por isso que o assert-script existe.

![Duas caminhadas sobre os mesmos bytes TLV, uma avançando por quatro mais comprimento até o próximo header, a outra pousando quatro bytes antes, de modo que toda leitura posterior fica silenciosamente errada.](assets/v05-diagram.webp)

Os próprios comprimentos já estão te contando o que mora dentro de cada valor, mesmo antes de a gente estudar os mecanismos. As duas entradas de 32 bytes, MintCloseAuthority e PermanentDelegate, são cada uma uma única pubkey: uma autoridade, nada mais. TransferHook e MetadataPointer marcam 64 as duas, e as duas são um par de pubkeys: uma autoridade com permissão para atualizar a entrada, mais o endereço para o qual ela aponta (um programa de hook num caso, uma conta de metadados no outro). O 65 esquisito do ConfidentialTransferMint são duas pubkeys mais um único byte de política de aprovação no meio. E o 174 do TokenMetadata é a única entrada de comprimento variável do conjunto: ela guarda strings de verdade (nome, símbolo, URI), então o comprimento dela varia por mint enquanto o comprimento de toda outra entrada é fixado pela struct dela. Você ainda não consegue decodificar os valores, e esta lição deliberadamente não faz isso: layouts de valor são conhecimento por extensão, e o módulo 2 pega um mecanismo de cada vez. Mas você já consegue fazer uma perícia surpreendentemente afiada só com `{name, type, length}`, que é exatamente a interface que seu inspetor exporta.

Uma nota de nomenclatura antes de você construir, porque três grafias de uma mesma extensão vão, senão, te custar meia hora de confusão. A saída `jsonParsed` da RPC na lição passada chamou a quinta extensão do PYUSD de `confidentialTransferFeeConfig`. O código-fonte em Rust chama de `ConfidentialTransferFeeConfig`. O enum do cliente JS fixado, que seu inspetor vai usar para os nomes, imprime `ConfidentialTransferFee`. Mesma extensão, código de tipo 16 nos três. O u16 é a identidade; nomes são só peles por toolchain em cima dele, que é exatamente por que seu inspetor imprime tanto o nome quanto o número.

E não é a única: no pin acima, o cliente também chama o tipo 25 de `ScaledUiAmountConfig` onde o código-fonte diz `ScaledUiAmount`, e o tipo 26 de `PausableConfig` onde o código-fonte diz `Pausable`. Três códigos, três discordâncias, uma regra — compare números, não strings, sempre que duas toolchains tiverem que concordar sobre a mesma extensão. m01-l4 faz você sacar isso: o validador dele é um port do Rust, então alimentá-lo com os nomes do lado cliente do seu inspetor é exatamente como você consegue fazer uma regra correta rejeitar um mint que é perfeitamente legal.

### A matemática de espaço: try_calculate_account_len

Agora você consegue ler qualquer mint existente. Criar um coloca o problema inverso: antes de a conta existir você precisa dizer ao system program quantos bytes alocar e quanto rent financiar, e você não consegue sair de uma resposta errada com um realloc depois, porque extensões de mint são definidas na criação. O modelo dessa computação mora no código-fonte Rust do Token-2022 como `ExtensionType::try_calculate_account_len`, e a gente lê em vez de escrever (nenhum Rust é escrito nesta lição). O que o código-fonte faz: se o conjunto de extensões está vazio, devolve o comprimento base simples. Caso contrário, parte de 165 mais 1, a base preenchida mais o discriminador, e soma `4 + value_length` para cada extensão pedida, exatamente a aritmética que sua caminhada TLV acabou de verificar ao contrário.

Os números resultantes são agradavelmente não-redondos, e cada um discrimina o próprio custo. Um mint só com NonTransferable, uma extensão marcadora cujo valor tem zero bytes, dá 165 + 1 + 4 + 0 = 170 bytes. Um mint só com TransferFeeConfig, cujo valor são 108 bytes de autoridades, quantias retidas e duas tabelas de taxa, dá 165 + 1 + 4 + 108 = 278. Esses dois valores saem direto das amostras token-2022 do program-examples da solana-developers, e agora eles também saem da sua própria aritmética. Toda extensão se paga em rent de conta, e só o comprimento já te diz o custo. O lab calcula esses números com o espelho do mesmo modelo no cliente JS, para que você nunca os decore, você os regenera.

![Um gráfico de barras de tamanhos de mint: 82 bytes simples, 170 com NonTransferable, 278 com uma config de taxa de transferência, e 866 para o PYUSD, contra o piso de base estendida de 166 bytes.](assets/v06-chart.webp)

Aqui está o trade-off honesto desta lição inteira, dito uma vez antes de você construir. Ler bytes brutos te dá uma verdade de base que nenhuma UI de carteira, nenhum parser de RPC e nenhum SDK consegue esconder de você. Também é frágil por natureza: offsets se deslocam conforme extensões são adicionadas, códigos de tipo precisam mapear corretamente, e você já viu como um avanço errado de cursor corrompe tudo depois dele silenciosamente. Fazer parse à mão é a jogada didática certa e a jogada errada em produção. Os clientes publicados existem justamente para que você raramente faça isso à mão, e o passo 6 do lab usa um deles para checar seu trabalho. Mas quando uma carteira mostra uma coisa e um explorer mostra outra, os bytes são o desempate, e depois de hoje você está qualificado a consultá-los. É esse o ponto: não substituir as ferramentas, mas parar de ser refém delas.

Uma nota de honestidade relacionada, e sua segunda cor do dia: agora você sabe o custo exato em bytes de cada extensão, mas ninguém publica o custo de compute de cada extensão. Números de CU por extensão não aparecem em lugar nenhum na documentação nem no código do Token-2022; eu chequei os dois (2026-08-21) e voltei de mãos vazias, e é por isso que uma lição posterior deste curso mede esses custos num lab em vez de citar uma tabela. Anatomia você consegue ler; custo de runtime você tem que medir.

## Lab: construir o R1, o inspetor de mints

Mãos no teclado, mais ou menos quarenta minutos. Este workspace é consumido por lições posteriores, então coloque ele onde o curso mora.

**1. Levante o workspace e fixe a toolchain.** A convenção de workspace do curso, valendo daqui até o capstone, é uma pasta por lição dentro de `labs/`. Lições posteriores importam entre essas pastas por caminho relativo, então os nomes são estruturais em vez de decorativos:

```bash
mkdir -p labs/m01-l2 && cd labs/m01-l2
npm init -y
npm pkg set type=module
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.20.5
```

O tsx entra no workspace como dependência de desenvolvimento no mesmo pin 4.20.5 que os one-liners de m01-l1 usaram, então os comandos `npx tsx` puros abaixo resolvem para essa cópia local fixada em vez do que quer que o npx fosse buscar hoje.

Os pins merecem um parágrafo, porque eu tive que tomar uma decisão de verdade aqui e você deveria ver isso. Em 2026-09-05, o `latest` do npm para `@solana/kit` é 8.2.0, mas "instale o latest" não é como você fixa um workspace Solana. A regra que de fato decide o número: fixe o major do kit contra o qual os clientes `@solana-program/*` do seu workspace fazem peer. O cliente deste workspace é `@solana-program/token-2022`, cuja linha atual é **0.15.0** e faz peer com kit ^7.0.0 — a 0.16.0 já pulou a faixa de peer dela para ^8, então instalar essa contra o kit 7 falha a checagem de peer na hora. O kit mais novo dentro da faixa ^7 é 7.1.1, e isso resolve a questão. Eu verifiquei essas faixas de peer contra o npm hoje em vez de confiar em qualquer doc, inclusive esta: elas vão derivar, e `npm view @solana-program/token-2022 peerDependencies` leva dez segundos. Então o par é kit 7.1.1 mais token-2022 0.15.0, versões exatas, nenhum caret na prática. Se a instalação acima terminou sem uma reclamação de `ERESOLVE`, seu workspace bate com o meu.

**2. Escreva o inspetor, a parte resolvida primeiro.** Antes do código, segure o fluxo de decisão inteiro na cabeça uma vez. Ele é curto, e todo desvio é algo que a teoria acabou de ensinar:

![Um fluxograma do inspetor de mints: buscar os bytes, parsear a base, reportar simples em exatamente 82 bytes, senão checar o byte 165 e caminhar pela região TLV a partir do byte 166.](assets/v07-flowchart.webp)

Agora crie `decode-mint.ts`. Tudo aqui é mostrado completo exceto uma região: a fatia base e a checagem de is-extended são suas para copiar, e o loop TLV é seu para escrever.

```typescript
// decode-mint.ts - R1, the Overgrowth mint inspector.
// Parses a mint account's raw bytes: 82-byte base, bare-vs-extended, TLV walk.
// Run: npx tsx decode-mint.ts <MINT_ADDRESS> [RPC_URL]
// Pins (verified 2026-09-05): @solana/kit 7.1.1, @solana-program/token-2022 0.15.0

import { pathToFileURL } from "node:url";
import { createSolanaRpc, address, getBase58Decoder } from "@solana/kit";
import { ExtensionType } from "@solana-program/token-2022";

const BARE_MINT_LEN = 82; // classic mint: full layout, nothing after
const EXTENDED_BASE_LEN = 165; // extended mint: base padded to token-account length
const ACCOUNT_TYPE_LEN = 1; // one discriminator byte at offset 165
const TLV_START = EXTENDED_BASE_LEN + ACCOUNT_TYPE_LEN; // 166

export interface TlvEntry {
  name: string;
  type: number;
  length: number;
}

export interface ParsedMint {
  kind: "bare-82" | "extended-165+1";
  mintAuthority: string | null;
  supply: bigint;
  decimals: number;
  freezeAuthority: string | null;
  accountType: number | null; // 1 = Mint; null on a bare mint
  extensions: TlvEntry[];
}

const b58 = getBase58Decoder();

/** Read a COption<Pubkey>: u32 LE tag (0 = None, 1 = Some) + 32-byte pubkey. */
function readCOptionPubkey(data: Uint8Array, offset: number): string | null {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const tag = view.getUint32(offset, true);
  if (tag === 0) return null;
  return b58.decode(data.subarray(offset + 4, offset + 36));
}

/** Parse the 82-byte base + (if present) the account-type byte and TLV region. */
export function parseMint(data: Uint8Array): ParsedMint {
  if (data.length < BARE_MINT_LEN) {
    throw new Error(`account is ${data.length} bytes; a mint is at least 82`);
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);

  // The 82-byte base, same in both programs:
  const mintAuthority = readCOptionPubkey(data, 0); //  0..36 COption<Pubkey>
  const supply = view.getBigUint64(36, true); // 36..44 u64 LE
  const decimals = data[44]; // 44     u8
  const isInitialized = data[45]; // 45     bool
  const freezeAuthority = readCOptionPubkey(data, 46); // 46..82 COption<Pubkey>
  if (isInitialized !== 1) throw new Error("mint is not initialized");

  // The length IS the first tell: exactly 82 bytes means bare, no TLV region.
  if (data.length === BARE_MINT_LEN) {
    return {
      kind: "bare-82",
      mintAuthority,
      supply,
      decimals,
      freezeAuthority,
      accountType: null,
      extensions: [],
    };
  }

  // Extended: base padded to 165 bytes, then one account-type byte.
  const accountType = data[EXTENDED_BASE_LEN];
  if (accountType !== 1) {
    throw new Error(`account-type byte is ${accountType}, expected 1 (Mint)`);
  }

  const extensions: TlvEntry[] = [];
  let cursor = TLV_START;
  // TODO(you): walk the TLV region.
  // While there is room for a 4-byte header (cursor + 4 <= data.length):
  //   - read type as u16 LE at cursor, and length as u16 LE at cursor + 2
  //   - a type of 0 (Uninitialized) is padding, not an entry: stop the walk
  //   - push { name: ExtensionType[type] ?? `unknown(${type})`, type, length }
  //   - advance the cursor past the header AND the value. Not by `length`.

  return {
    kind: "extended-165+1",
    mintAuthority,
    supply,
    decimals,
    freezeAuthority,
    accountType,
    extensions,
  };
}

/** The tool's interface: given a mint address and an RPC, return the parse. */
export async function decodeMint(
  mint: string,
  rpcUrl = "https://api.mainnet-beta.solana.com",
): Promise<ParsedMint & { owner: string; dataLength: number }> {
  const rpc = createSolanaRpc(rpcUrl);
  const { value: account } = await rpc
    .getAccountInfo(address(mint), { encoding: "base64" })
    .send();
  if (!account) throw new Error(`no account at ${mint}`);
  const data = Uint8Array.from(Buffer.from(account.data[0], "base64"));
  return { ...parseMint(data), owner: account.owner, dataLength: data.length };
}

async function main() {
  const [mint, rpcUrl] = process.argv.slice(2);
  if (!mint) {
    console.error("usage: npx tsx decode-mint.ts <MINT_ADDRESS> [RPC_URL]");
    process.exit(1);
  }
  const parsed = await decodeMint(mint, rpcUrl);

  console.log(`mint:          ${mint}`);
  console.log(`owner program: ${parsed.owner}`);
  console.log(`data length:   ${parsed.dataLength} bytes -> ${parsed.kind}`);
  console.log(`mintAuthority: ${parsed.mintAuthority}`);
  console.log(`supply:        ${parsed.supply} (bigint)`);
  console.log(`decimals:      ${parsed.decimals}`);
  console.log(`freezeAuth:    ${parsed.freezeAuthority}`);
  console.log(`extensions (${parsed.extensions.length}):`);
  for (const e of parsed.extensions) {
    console.log(`  { name: ${e.name}, type: ${e.type}, length: ${e.length} }`);
  }
}

// Run the CLI only when invoked directly, so tests can import parseMint.
// (String suffix checks are a trap here: "test-decode-mint.ts" also ends
// with "decode-mint.ts". Compare resolved URLs instead.)
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
```

Três escolhas de design que vale a pena nomear, e aí as partes rotineiras podem ficar rotineiras. `parseMint` é uma função pura de bytes para estrutura, sem rede por dentro, e é isso que a torna testável e é sobre isso que o challenge se apoia. `decodeMint` envolve ela com o fetch e é a interface que as lições posteriores importam; o validador `check-combo` de m01-l4 chama exatamente essa função, então o formato dela (`extensions` como `{name, type, length}[]`) é um contrato agora, não uma preferência de estilo. E o mapa de nomes é o próprio enum `ExtensionType` do cliente fixado em vez de uma tabela digitada à mão: o pacote que a gente instalou para o passo 6 já vem com o mapeamento de u16 para nome, gerado a partir do código-fonte do programa, e eu não vou manter uma cópia pior dele à mão. (Da última vez que copiei à mão um enum desses para dentro de um projeto, uma nova variante foi entregue upstream e minha tabela mentiu para mim por um mês. Reaproveite as tabelas do ecossistema.)

**3. Aponte para um mint simples primeiro.** O USDC clássico, o mesmo mint que você leu através do parser da RPC na lição passada:

```bash
npx tsx decode-mint.ts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
```

Minha execução hoje:

```text
mint:          EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
owner program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
data length:   82 bytes -> bare-82
mintAuthority: BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG
supply:        7923463957481104 (bigint)
decimals:      6
freezeAuth:    7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar
extensions (0):
```

O ramo de 82 bytes funciona de ponta a ponta com o código como foi dado: campos base decodificados dos bytes brutos pelos seus próprios offsets, `bare-82` reportado só a partir da checagem de comprimento, supply impresso como BigInt (aquele é o número de 88 por cento da teoria, ao vivo). Seu supply vai diferir do meu; é uma conta viva.

**4. Agora o PYUSD, e bata no TODO.** Rode contra o mint estendido:

```bash
npx tsx decode-mint.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

Com o TODO não preenchido você vai ver os campos base decodificarem corretamente, `866 bytes -> extended-165+1`, e então `extensions (0):`. O inspetor enxerga a região e não consegue lê-la. Este é o problema de completar: **preencha o loop TLV.** Você tem tudo o que precisa: o layout do header vindo da teoria, a caminhada resolvida sobre estes exatos 866 bytes na tabela acima, `view.getUint16(offset, true)` para leituras de u16 little-endian, e as duas condições de parada nos comentários do TODO. Tem menos de dez linhas. A única linha que importa é o avanço do cursor, e você sabe por quê.

**5. Rode de novo e reconcilie.** Quando seu loop estiver certo, o mesmo comando imprime:

```text
mint:          2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
data length:   866 bytes -> extended-165+1
mintAuthority: 8Jornc27vtAYPkwDzsZVgLQchAYyC8nD7aCNPCDV8Qk2
supply:        688176370728435 (bigint)
decimals:      6
freezeAuth:    2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk
extensions (8):
  { name: MintCloseAuthority, type: 3, length: 32 }
  { name: PermanentDelegate, type: 12, length: 32 }
  { name: TransferFeeConfig, type: 1, length: 108 }
  { name: ConfidentialTransferMint, type: 4, length: 65 }
  { name: ConfidentialTransferFee, type: 16, length: 129 }
  { name: TransferHook, type: 14, length: 64 }
  { name: MetadataPointer, type: 18, length: 64 }
  { name: TokenMetadata, type: 19, length: 174 }
```

Oito entradas, códigos de tipo fora de ordem, comprimentos batendo com a tabela da caminhada. Se em vez disso você obteve uma entrada correta seguida de nomes sem sentido como `unknown(53421)`, parabéns, você construiu o bug de cursor da seção de teoria; você avançou só por `length`. Conserte o avanço e veja o lixo se recompor em oito entradas limpas. Sinceramente, tropeçar nisso de propósito uma vez vale a pena, só para ver o quanto os destroços parecem plausíveis.

**6. Faça a verificação cruzada contra o cliente publicado.** Seu parser concorda com os bytes; agora confirme que ele concorda com o ecossistema. Crie `crosscheck.ts`:

```typescript
// crosscheck.ts - decode the same mint with the shipped client and compare.
// Run: npx tsx crosscheck.ts
import { createSolanaRpc, address } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

async function main() {
  const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");
  const mint = await fetchMint(rpc, address(PYUSD));
  console.log(`supply:   ${mint.data.supply} (${typeof mint.data.supply})`);
  console.log(`decimals: ${mint.data.decimals}`);
  const ext = mint.data.extensions;
  if (ext.__option === "Some") {
    console.log(`extensions (${ext.value.length}):`);
    for (const e of ext.value) console.log(`  - ${e.__kind}`);
  }
}
main().catch((e) => { console.error(e); process.exit(1); });
```

Rode `npx tsx crosscheck.ts`. O cliente reporta o mesmo supply (como bigint, repare: o kit fez a mesma escolha de u64 que a gente fez), os mesmos decimals, e os mesmos oito tipos de extensão na mesma ordem. Dois decodificadores independentes, um deles seu, concordando byte a byte. Essa é a relação saudável com a ferramenta publicada: use ela todo dia, e seja capaz de auditá-la quando duas fontes discordarem.

**7. Calcule comprimentos de conta em vez de memorizá-los.** O cliente JS espelha `try_calculate_account_len` como `getMintSize`. Crie `compute-len.ts`:

```typescript
// compute-len.ts - the account-length model, computed instead of memorized.
// Run: npx tsx compute-len.ts
import { address } from "@solana/kit";
import { getMintSize, type Extension } from "@solana-program/token-2022";

// A pubkey placeholder: sizes depend on the SET, never on the values.
const ANY = address("11111111111111111111111111111111");

// Bare classic mint: call it with no extension list at all.
console.log(`bare mint:              ${getMintSize()} bytes`);

// NonTransferable is a zero-length marker: 165 + 1 + (2 + 2 + 0) = 170.
const nonTransferable: Extension = { __kind: "NonTransferable" };
console.log(`+ NonTransferable:      ${getMintSize([nonTransferable])} bytes`);

// TransferFeeConfig carries 108 value bytes: 165 + 1 + (2 + 2 + 108) = 278.
const fee = { epoch: 0n, maximumFee: 0n, transferFeeBasisPoints: 0 };
const transferFee: Extension = {
  __kind: "TransferFeeConfig",
  transferFeeConfigAuthority: ANY,
  withdrawWithheldAuthority: ANY,
  withheldAmount: 0n,
  olderTransferFee: fee,
  newerTransferFee: fee,
};
console.log(`+ TransferFeeConfig:    ${getMintSize([transferFee])} bytes`);

// Both at once: one 165+1 base, then each entry pays its own 4-byte header.
console.log(`+ both:                 ${getMintSize([nonTransferable, transferFee])} bytes`);
```

```text
bare mint:              82 bytes
+ NonTransferable:      170 bytes
+ TransferFeeConfig:    278 bytes
+ both:                 282 bytes
```

O 170 e o 278 da teoria, regenerados sob demanda, mais o caso combinado: 166 + (4 + 0) + (4 + 108) = 282, uma base compartilhada, cada extensão pagando o próprio header. Quando você dimensionar o mint do SPROUT no trabalho de design do próximo módulo, esta é a ferramenta que precifica cada conjunto candidato de extensões antes de você comprometer rent com ele.

**8. O critério de aceitação.** Último arquivo: `test-decode-mint.ts`, o assert-script. Este é o padrão da linha de testes que o curso inteiro reaproveita: `node:assert` puro, um alvo fixado, exit 1 no primeiro erro.

```typescript
// test-decode-mint.ts - acceptance gate for R1, the mint inspector.
// Decodes a pinned known mint (PYUSD) and asserts the expected extension set.
// Run: npx tsx test-decode-mint.ts
import assert from "node:assert/strict";
import { createSolanaRpc, address } from "@solana/kit";
import { parseMint } from "./decode-mint.ts";

const RPC_URL = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
const PYUSD = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

// The pinned expectation: PYUSD's eight TLV entries, in on-chain order,
// with the exact value lengths read on 2026-08-22. The extension SET is fixed
// at mint creation, but TokenMetadata's value holds updatable strings (name,
// symbol, URI), so that one entry's length, and the account total, can
// legitimately change. If this ever fails on `length`, re-read the live
// account before blaming your parser.
const EXPECTED = [
  { name: "MintCloseAuthority", type: 3, length: 32 },
  { name: "PermanentDelegate", type: 12, length: 32 },
  { name: "TransferFeeConfig", type: 1, length: 108 },
  { name: "ConfidentialTransferMint", type: 4, length: 65 },
  { name: "ConfidentialTransferFee", type: 16, length: 129 },
  { name: "TransferHook", type: 14, length: 64 },
  { name: "MetadataPointer", type: 18, length: 64 },
  { name: "TokenMetadata", type: 19, length: 174 },
];

async function main() {
  const rpc = createSolanaRpc(RPC_URL);
  const { value: account } = await rpc
    .getAccountInfo(address(PYUSD), { encoding: "base64" })
    .send();
  assert.ok(account, "PYUSD mint account not found");

  const data = Uint8Array.from(Buffer.from(account.data[0], "base64"));
  const parsed = parseMint(data);

  // Base: correctly reported as extended, not bare.
  assert.equal(parsed.kind, "extended-165+1", "PYUSD must report extended");
  assert.equal(parsed.accountType, 1, "account-type byte must be 1 (Mint)");
  assert.equal(parsed.decimals, 6, "PYUSD has 6 decimals");

  // Supply is a bigint end to end: u64 does not fit a JS number safely.
  assert.equal(typeof parsed.supply, "bigint", "supply must be a bigint");
  assert.ok(parsed.supply > 0n, "live PYUSD supply is positive");

  // Every printed TLV entry matches the expected {name, type, length} set.
  assert.deepEqual(parsed.extensions, EXPECTED, "TLV extension set mismatch");

  // The lengths must reconcile with the account size: nothing skipped.
  const tlvBytes = parsed.extensions.reduce((n, e) => n + 4 + e.length, 0);
  assert.equal(166 + tlvBytes, data.length, "TLV walk must cover the account");

  console.log(`ok - ${parsed.extensions.length} extensions on ${PYUSD}`);
  console.log(`ok - base reported as ${parsed.kind}, supply is bigint`);
  console.log(`ok - 166 + ${tlvBytes} TLV bytes = ${data.length} bytes total`);
}

main().catch((e) => {
  console.error("FAIL:", e.message);
  process.exit(1);
});
```

**9. Checkpoint.** Rode o critério:

```bash
npx tsx test-decode-mint.ts
```

```text
ok - 8 extensions on 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
ok - base reported as extended-165+1, supply is bigint
ok - 166 + 700 TLV bytes = 866 bytes total
```

Se a execução morrer antes de qualquer assert com um erro de fetch ou um `429`, é a RPC pública gratuita te limitando por taxa, não o seu código: espere trinta segundos, ou aponte `RPC_URL` para qualquer endpoint que você já use (o script lê essa variável do ambiente exatamente para este momento). Se morrer no primeiro assert com `account not found`, confira a constante do mint caractere por caractere; endereços base58 não sobrevivem a um copiar-e-colar parcial, e o erro é indistinguível da conta genuinamente não existir.

Três linhas `ok` e um exit code zero, e o R1 é real: o inspetor enumera o conjunto TLV completo de um mint ao vivo, reporta simples versus estendido corretamente, e prova isso contra uma expectativa fixada. Repare no que o último assert te compra que olhar no olho nunca vai comprar: o reduce sobre `4 + e.length` recalcula o tamanho da conta a partir do seu próprio parse, então o bug de cursor exato do passo 5 não consegue passar por este critério nem se os nomes-lixo por acaso parecessem plausíveis. Se em vez disso a execução falhar num `length`, releia o comentário acima de `EXPECTED` antes de mexer no seu parser: testes fixados contra contas vivas carregam uma suposição datada, e a jogada honesta é reverificar o alvo, não enfraquecer o assert.

**10. Solte a coleira.** Antes do challenge, gaste cinco minutos apontando `decode-mint` para mints que ninguém te passou: algo da sua própria carteira, algo em alta num explorer, o mint Token-2022 mais esquisito que você conseguir achar.

```bash
# any mint, any RPC; the second argument is optional
npx tsx decode-mint.ts <MINT_ADDRESS_FROM_YOUR_WALLET>
```
 Toda execução é um de três resultados, e os três ensinam. Um `bare-82` com campos familiares: o mundo clássico, totalmente legível para você agora. Um mint estendido cuja lista de extensões explica o comportamento dele antes de você sequer ler a documentação. Ou um nome `unknown(N)`: um código de tipo mais novo que o enum do cliente fixado, o que não é um bug e sim um carimbo de tempo, prova de que o catálogo se move e de que seu parser degrada com elegância em vez de mentir. Guarde uma anotação do conjunto mais estranho que você encontrar; o validador de m01-l4 vai ter opiniões sobre ele.

## Challenge

Solo, sem walkthrough, sem espiar o código do lab. Esta é a versão coding-challenge do que você acabou de construir: as duas computações centrais como funções puras, provadas por testes, sem rede em lugar nenhum.

Escreva `tlv.ts` exportando duas funções:

1. `parseTlv(data: Uint8Array, start: number): TlvEntry[]`, um parser TLV puro: caminha pelos headers a partir de `start`, para na borda da conta ou num tipo 0, devolve entradas `{name, type, length}`. Ele precisa lançar exceção se um comprimento declarado passasse do fim do buffer (seu loop do lab nunca checou isso; parsers de produção precisam).
2. `mintLen(valueLengths: number[]): number`, o modelo de comprimento de conta como aritmética: array vazio dá 82, caso contrário 166 mais os headers e valores por entrada.

Depois `test-tlv.ts` provando as duas com arrays de bytes montados à mão, sem RPC: uma região TLV sintética de duas entradas que você mesmo constrói com tipos e comprimentos conhecidos; uma entrada marcadora de comprimento zero; um buffer truncado que precisa lançar exceção; e as asserções de que `mintLen([])` é 82, `mintLen([0])` é 170, e `mintLen([108])` é 278. Aceite quando os dois arquivos rodarem limpos sob `npx tsx test-tlv.ts` e o seu `parseTlv`, apontado para os bytes brutos da conta do PYUSD no offset 166 (busque-os em base64 do jeito que `peek.ts` fez; `decodeMint` devolve o parse, não os bytes), reproduzir exatamente as oito entradas que seu inspetor imprime.

Se seus testes sintéticos passarem mas a reprodução do PYUSD divergir, não confie em nenhum dos dois: dê um diff nas duas listas de entradas e descubra qual função está mentindo. Essa disciplina de diff, dois caminhos independentes para a mesma resposta, é o que o passo 6 ensinou, e é o hábito que sobrevive a qualquer toolchain.

## O que você domina agora

Momento de feedback, pontuado com honestidade. Você construiu a primeira ferramenta do curso e o primeiro critério de aceitação dele. Você consegue parsear a base de 82 bytes no escuro, você sabe por que o byte 165 existe e o que seus dois valores significam, você consegue caminhar por uma região TLV sem biblioteca, você conhece a única constante de cursor que separa um parser que funciona de um lixo plausível, e seus supplies são BigInts porque você viu um supply de mainnet ao vivo sentado em 88 por cento da linha de inteiro seguro do JavaScript. O que você ainda não consegue fazer: explicar o que qualquer uma daquelas oito extensões de fato faz na hora da transferência, ou escolher um conjunto para um mint seu. Bem dentro do cronograma; mecanismos são os módulos 2 e 3, e escolher está a duas lições de distância.

Agora você consegue ler os bytes de qualquer mint e ver a base clássica de 82 bytes sentada embaixo das extensões. Mas eis a questão sobre essa base: o programa que a serve não é o programa que a serviu por seis anos. Mesma interface, mesmos offsets que você acabou de memorizar, um motor completamente reescrito por baixo, e sua medição de 76 CU de m01-l1 é o recibo. Isso é o p-token, a troca de motor que aconteceu debaixo dos pés de todo mundo, e é a próxima lição.

Bom parsing! 🌱
