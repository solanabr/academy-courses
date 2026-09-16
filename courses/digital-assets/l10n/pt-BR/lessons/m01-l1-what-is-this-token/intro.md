# O que é esse token, de verdade?

Esta é a lição um, então nada foi construído ainda. Você chega com os fundamentos da Solana na cabeça: contas, PDAs, transações, taxas, ATAs. Você também chega, muito provavelmente, com uma crença que eu segurei por tempo demais: a de que "um token" quer dizer o mint SPL clássico que você já usa. A gente vai quebrar essa crença nos próximos cinco minutos, usando um token que a PayPal entrega para milhões de pessoas. Se você chegou aqui vindo do curso Solana Payments and Commerce, você já conheceu este mint pelo lado de quem paga; agora você vai ler cada byte dele.

Nenhuma instalação de toolchain. Você precisa do Node 20 ou mais novo (`node --version` para conferir; eu estou no 23.9) e nada mais. Salve este arquivo como `read-pyusd.ts`:

```typescript
// read-pyusd.ts - read PayPal USD's live mint account and list what it carries.
// Zero npm dependencies. Run: npx tsx@4.20.5 read-pyusd.ts
// Read-only: one RPC call against mainnet, nothing signed, nothing sent.

const RPC = "https://api.mainnet-beta.solana.com";
const PYUSD_MINT = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
const CLASSIC_MINT_SIZE = 82; // a classic SPL mint is exactly this many bytes

async function main() {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      params: [PYUSD_MINT, { encoding: "jsonParsed" }],
    }),
  });
  const { result, error } = await res.json();
  if (error) throw new Error(JSON.stringify(error));

  const account = result.value;
  const info = account.data.parsed.info;

  console.log(`mint:          ${PYUSD_MINT}`);
  console.log(`owner program: ${account.owner}`);
  console.log(`account size:  ${account.space} bytes (a classic mint is ${CLASSIC_MINT_SIZE})`);
  console.log(`decimals:      ${info.decimals}`);
  console.log(`supply:        ${info.supply} base units`);
  console.log(`mintAuthority: ${info.mintAuthority}`);
  console.log(`freezeAuth:    ${info.freezeAuthority}`);

  const extensions = info.extensions ?? [];
  console.log(`\nextensions (${extensions.length}):`);
  for (const ext of extensions) {
    console.log(`  - ${ext.extension}`);
  }

  const hook = extensions.find((e: any) => e.extension === "transferHook");
  const fee = extensions.find((e: any) => e.extension === "transferFeeConfig");
  console.log(`\ntransferHook.programId: ${hook?.state.programId}`);
  console.log(
    `transferFee: ${fee?.state.newerTransferFee.transferFeeBasisPoints} bps, ` +
    `max ${fee?.state.newerTransferFee.maximumFee}`
  );
  if (hook && hook.state.programId === null) {
    console.log(`\n=> a transfer hook slot exists, but no hook program is set.`);
    console.log(`   configured, but dormant.`);
  }
}

main().catch((e) => { console.error(e); process.exit(1); });
```

Rode:

```bash
npx tsx@4.20.5 read-pyusd.ts
# tsx pinned at 4.20.5, verified working 2026-08-22; npx fetches it on first
# run, so there is genuinely nothing to install.
```

Quando eu rodei isso hoje, 2026-08-22, contra o RPC público gratuito, eu obtive:

```text
mint:          2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
account size:  866 bytes (a classic mint is 82)
decimals:      6
supply:        688176370728435 base units
mintAuthority: 8Jornc27vtAYPkwDzsZVgLQchAYyC8nD7aCNPCDV8Qk2
freezeAuth:    2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk

extensions (8):
  - mintCloseAuthority
  - permanentDelegate
  - transferFeeConfig
  - confidentialTransferMint
  - confidentialTransferFeeConfig
  - transferHook
  - metadataPointer
  - tokenMetadata

transferHook.programId: null
transferFee: 0 bps, max 0

=> a transfer hook slot exists, but no hook program is set.
   configured, but dormant.
```

Oito extensões, sentadas em um mint que milhões de usuários da PayPal movimentam sem nunca desconfiar de que ele tem um formato diferente de qualquer outro token de dólar. Um transfer hook que existe mas não aponta para programa nenhum. Uma taxa de transferência de zero basis points que mesmo assim está ali nos bytes, esperando. A sua linha de supply vai ser diferente da minha, porque esta é uma conta ao vivo e a PayPal cunha e queima contra ela todo dia. Todo o resto deve bater.

Espera, calma aí. Você "usa SPL tokens" todo dia. Então por que este aqui carrega um `permanentDelegate`, uma autoridade que pode mover o PYUSD de qualquer pessoa para fora da conta de qualquer pessoa? Por que o mint dele tem dez vezes o tamanho dos mints que você já fez? E por que existe um slot de hook configurado mas desligado?

Você ainda não consegue responder. Esse buraco é este curso.

## Resumo

Você acabou de decodificar um token que não fez, usando um decodificador que não escreveu. Esta lição é sobre o que você viu: o que uma conta de mint é de fato, o que as oito entradas de extensão no mint do PYUSD significam como formato (ainda não como mecanismo), e por que "SPL token" parou de nomear uma coisa só anos atrás. No lab você vai rodar um segundo script de zero setup que simula uma transferência clássica simples e lê o custo de computação dela direto do runtime: um segundo número que você ainda não consegue explicar. Os dois números são resolvidos ao longo deste módulo. A tese com que você sai hoje é curta: uma primitiva de ativo é uma decisão, e você não pode tomar uma decisão que não consegue ler.

Regras da casa, ditas uma vez e valendo para o curso inteiro:

- **Leia a teoria, não codifique junto com ela.** Suas mãos se movem no Lab numerado, e só lá. A abertura que você acabou de rodar é a única exceção que toda lição ganha: algo para fazer antes de qualquer coisa para acreditar.
- **Faça o Challenge sozinho.** Sem walkthrough, sem passos da solução. É ali que o aprendizado rende juros compostos.
- **Toda ferramenta mostra a instalação dela na primeira vez que aparece**, e todo pin de versão carrega uma data. Hoje foi `tsx@4.20.5`, verificado em 2026-08-22.
- **A ajuda recua conforme um cronograma.** Hoje eu te dou dois scripts completos e você roda os dois. Na próxima lição você constrói o decodificador: a fatia dos campos base é trabalhada junto com você, e a caminhada pelas extensões é sua para escrever. No fim do módulo você está escolhendo primitivas e defendendo a escolha. As rodinhas são tiradas deliberadamente, não de surpresa.

## Um token é uma decisão que você ainda não consegue tomar

Vamos nomear o que você leu de verdade, porque duas das palavras importam enormemente e desenvolvedores Solana embaralham as duas todo dia.

Uma **conta de mint** é a conta que define o próprio token: o supply dele, os decimals dele, quem pode criar mais dele, quem pode congelá-lo. Um mint por token, para sempre. Não é onde os saldos vivem. Os saldos vivem em **contas de token** (normalmente as associated token accounts, as ATAs, que você já conhece), uma por holder por token. Quando você "confere o seu saldo de PYUSD" você lê uma conta de token. Quando você pergunta o que o PYUSD *é*, você lê o mint. Hoje a gente lê o mint, e a gente vai continuar lendo mints o módulo inteiro, porque o mint é onde a identidade de um token e as regras dele estão escritas.

Agora o campo que deveria ter te parado: `owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`. Esse não é o programa de token que você conhece. O programa SPL Token clássico vive em `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`. O mint do PYUSD pertence a um programa diferente em um endereço diferente: Token-2022, também chamado de Token Extensions. Mesmo trabalho, regulamento diferente, e os dois não se misturam. Uma instrução de token clássico apontada para um mint Token-2022 falha, e vice-versa.

![Dois programas de token separados, o SPL Token clássico sendo dono do mint de 82 bytes do USDC e o Token-2022 sendo dono do mint de 866 bytes do PYUSD com oito extensões, com as instruções incapazes de cruzar entre eles.](assets/v01-diagram.webp)

Esta é a cilada número um deste curso inteiro, então eu vou dizer sem rodeios: **"SPL token" não é uma coisa só.** Passei um trecho constrangedor do começo do meu trabalho com Solana dizendo "SPL token" como se aquilo nomeasse um padrão único, e entreguei integrações em cima dessa suposição. Culpado, com folga. Me custou um fim de semana de debug na primeira vez que um mint Token-2022 bateu em código que tinha `Tokenkeg` hardcoded. Os dois programas coexistem na mainnet, para sempre, e toda carteira, DEX e indexador tem que lidar com os dois.

### Os 82 bytes e os 866

Tamanho é o jeito mais rápido de sentir a diferença. Um mint SPL clássico tem exatamente 82 bytes, sempre: autoridade de mint, supply, decimals, flag de inicializado, autoridade de congelamento. Layout fixo, nada opcional, nada além, e os mesmos cinco campos, seja o mint o USDC ou algo que você subiu numa terça à tarde para praticar. Seu script imprimiu o PYUSD em 866 bytes. Para onde foram os outros 784 bytes?

Eles foram para as **extensões**: pacotes opcionais de estado e comportamento extra que o Token-2022 deixa um emissor anexar a um mint (ou a contas de token individuais) no momento da criação. Cada extensão é depositada nos dados da conta usando um esquema chamado **TLV**, de tipo-comprimento-valor: uma tag pequena dizendo qual extensão é essa, um comprimento dizendo quantos bytes ela ocupa, e então os bytes de valor em si. Uma depois da outra, como caixas etiquetadas numa fileira. Um decodificador percorre a fileira: lê uma tag, lê um comprimento, pula à frente, repete. Essa caminhada é exatamente o que o parser do RPC fez por você hoje, e exatamente o que você vai implementar sozinho na próxima lição.

![Um mint clássico são cinco campos fixos totalizando 82 bytes, enquanto o mint do PYUSD tem os mesmos campos base seguidos por oito entradas de extensão tipo-comprimento-valor, totalizando 866 bytes.](assets/v02-diagram.webp)

Leia as oito entradas do PYUSD de novo, desta vez como uma história. O conjunto completo é mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer e tokenMetadata. Isso é um loadout com formato de compliance. Um delegado permanente significa que a Paxos, a emissora regulada, pode mover ou queimar PYUSD de qualquer conta: poderes de apreensão, a coisa que uma ordem judicial exige. O par de transferências confidenciais são trilhos de privacidade. O par de metadados coloca o nome e o símbolo do token on-chain no próprio mint em vez de na conta de um ecossistema separado. E duas das entradas estão carregadas mas não disparando: a taxa de transferência está configurada em 0 basis points com um máximo de 0, e o transfer hook, a extensão que deixaria um programa rodar em cada transferência, tem `programId: null`.

**Configurado, mas dormente.** Segure essa frase; ela faz trabalho de verdade o curso inteiro. Uma extensão estar presente nos bytes não é a mesma coisa que ela ter efeito ativo. O PYUSD carrega um slot de hook e uma tabela de taxas do jeito que um prédio carrega conduíte vazio: nada passa por ele hoje, mas ninguém precisa quebrar as paredes no martelete para mudar isso depois. Presente não é ativo, e essas são duas perguntas separadas que uma olhada única numa lista de extensões vai alegremente embaralhar. Quando você avaliar qualquer token de agora em diante, você pergunta primeiro se uma extensão existe nos bytes e depois, separadamente, se ela faz alguma coisa no momento.

Uma ressalva honesta antes da tese, porque é o trade-off em que esta lição inteira se apoia. Ler um mint te diz o que um token **é**. Não te diz o que o seu venue alvo vai **aceitar**. As mesmas oito extensões que deixam o PYUSD em compliance o bastante para a PayPal fariam um token ser recusado de cara por alguns programas de DEX que rejeitam extensões Token-2022 desconhecidas em vez de arriscar comportamento que não modelaram. Visibilidade é necessária e não suficiente. Você já consegue decodificar um token e ainda assim não consegue entregar um; a metade escolher-e-verificar dessa habilidade vem mais tarde no curso, e eu não vou fingir que o script de hoje te dá isso.

### O cardápio por trás da pergunta

Agora, o motivo de esta lição existir no dia um, antes de qualquer construção: tudo o que você vai fazer neste curso parte de uma decisão que a maioria das pessoas toma por padrão em vez de tomar de propósito: **qual primitiva de ativo você emite?**

Na Solana em 2026 esse cardápio tem quatro entradas sérias. Um mint SPL clássico: 82 bytes, nenhuma extensão, chato de propósito, suportado por literalmente tudo. Um mint Token-2022 com um conjunto de extensões escolhido: comportamento programável ao custo de perguntas de compatibilidade venue por venue. Um ativo Metaplex Core: o padrão recomendado atualmente para trabalho com NFT, uma família de programas completamente diferente. E um NFT comprimido: estado que vive numa árvore de Merkle em vez de na própria conta, uma redução de custo de mil vezes com as próprias consequências no caminho de leitura.

![Uma comparação de quatro vias entre mints SPL clássicos, mints Token-2022 com extensões, ativos Metaplex Core e NFTs comprimidos, cada um resumido por formato on-chain, reputação e cobertura no curso.](assets/v03-comparison.webp)

Aqui está a tese, e é a coisa mais próxima de filosofia que você ganha hoje. Cada um desses quatro não é um nível de produto; é uma resposta diferente para a pergunta "o que a chain deveria impor sobre este ativo?" Um mint clássico responde "quase nada além de supply e freeze". O conjunto de extensões do PYUSD responde "apreensão, taxas, privacidade e metadados, parte disso pré-instalada e dormente". Uma decisão dessas só é real se você consegue verificar o que foi de fato decidido, e o único lugar onde a decisão está escrita são os bytes que você leu hoje. Whitepapers descrevem intenções. Mints são a lei. Você não pode escolher uma primitiva de ativo que não consegue ler, e até esta manhã você não conseguia ler nenhuma. É por isso que a decodificação veio antes de tudo, incluindo o toolchain.

### Tokens mais novos que os tutoriais

Existe uma objeção justa rondando por aqui, e é a que eu teria levantado alguns anos atrás: talvez o Token-2022 seja principalmente uma especificação, impressionante no papel e pouco implantada na prática. O mint que você decodificou esta manhã é o contra-argumento, e é um contra-argumento pesado. PayPal e Paxos entregaram o PYUSD no Token-2022 em maio de 2024, o deployment emblemático do programa, e no fim de maio de 2025 a própria nota de desenvolvedor da Solana sobre o PYUSD contabilizou $215.9 milhões dele mantidos em apenas 20,400 contas de token. O número de supply que seu script imprimiu hoje é o que for hoje; o meu leu cerca de 688 milhões de dólares em base units. Número ao vivo, conta ao vivo, que é exatamente o motivo de o script ler isso em vez de eu afirmar.

Enquanto isso a educação oficial sobre tudo isso congelou no meio da trama. O repositório solana-foundation developer-content, a fonte por trás de uma geração inteira de cursos oficiais, foi arquivado em 24 de janeiro de 2025. Todo curso construído a partir dele é anterior a ScaledUiAmount, Pausable, ConfidentialMintBurn e à troca de motor do p-token. Pense no que isso significa por um instante: os tokens que você decodifica hoje são mais novos que os tutoriais que deveriam explicá-los.

![Uma linha do tempo que vai do lançamento do PYUSD em maio de 2024, passando pelo arquivamento do conteúdo oficial de desenvolvedor em janeiro de 2025, até as features de 2026 que nenhum tutorial cobre, terminando com o leitor decodificando o mint.](assets/v04-timeline.webp)

Essa data de arquivamento é o motivo de este curso ter uma disciplina de linha de testes que você vai encontrar muitas e muitas vezes: **meça, não memorize.** Números sobre um sistema vivo apodrecem. O que me traz ao segundo número que eu te prometi, o que você mesmo vai produzir no lab. Uma transferência simples no programa SPL Token clássico, do tipo chato de mint de 82 bytes, custa hoje 76 unidades de computação. **Unidades de computação**, CU, são o medidor da Solana para trabalho on-chain: toda instrução roda contra um orçamento (200,000 por padrão, e você vai ver esse número exato em uma linha de log daqui a pouco), e o que ela consome é reportado pelo próprio runtime. Durante anos essa mesma transferência custou 4,645 CU. Em 2026 a implementação por trás do programa de token clássico foi trocada por baixo da interface, e o preço despencou. Mesmo endereço de programa, mesmos bytes de instrução, uma queda de sessenta vezes. Como essa troca foi possível sem quebrar a carteira de ninguém é a lição três deste módulo, e é uma das melhores histórias de sistemas da Solana. Hoje você só mede as consequências, e se recusa a memorizá-las, porque um número que caiu sessenta vezes uma vez pode se mover de novo.

### O caminho daqui em diante: Overgrowth

Tudo neste curso constrói uma coisa só. **Overgrowth** é um jogo co-op de farming e crafting cuja economia on-chain inteira você vai levantar, primitiva por primitiva: SPROUT, a moeda dele, como um mint Token-2022 cujo conjunto de extensões você vai escolher e defender; NFTs Almanac, os livros de conhecimento colecionáveis que restringem o acesso às receitas de crafting; e Harvest crates, recompensas sazonais de drop cunhadas como NFTs comprimidos porque vão existir muitos deles demais para pagar rent por conta. No módulo final você terá emitido ativos em todos os formatos do cardápio acima, e o ponto de hoje é que você vai fazer isso como alguém que lê bytes antes de confiar em nomes.

Este módulo é a rampa de entrada, e ele roda de trás para frente de propósito: concreto primeiro, fundamentos depois. Hoje você pegou um decodificador emprestado e sentiu dois números que não consegue explicar. Na próxima lição você para de pegar emprestado: você constrói o decodificador sozinho, começando do mint pelado de 82 bytes e subindo pela caminhada do TLV, e esse inspetor vira a primeira ferramenta de verdade do kit Overgrowth, aquela que as lições seguintes chamam. A lição três explica o 76. A lição quatro transforma o catálogo de extensões em um framework de escolha e fecha o módulo com SPROUT como exemplo trabalhado; a decisão de fato sobre spec e tamanho do SPROUT abre a conversa de design do próximo módulo, com o framework nas suas mãos.

![Um mapa de módulo em quatro passos mostrando as duas medições sem explicação de hoje resolvidas pela construção do inspetor na lição dois, pela troca de motor na lição três e pelo framework de escolha na lição quatro.](assets/v05-flowchart.webp)

Chega de teoria. Vá medir o segundo número.

## Lab: dois números a partir do zero

Agora codifique junto; esta é a parte que você faz, não lê. Uns quinze minutos.

**1. Confirme seu runtime.** Você precisa do Node 20+ para o `fetch` nativo em que estes scripts se apoiam.

```bash
node --version
# v20.x or newer. Mine printed v23.9.0.
```

**2. Rode o leitor do PYUSD de novo, e desta vez leia como um auditor.** Você rodou ele na abertura; agora extraia afirmações dele. Rode `npx tsx@4.20.5 read-pyusd.ts` outra vez e confira quatro fatos contra a sua própria saída: o owner program começa com `Tokenz` (Token-2022, não clássico); a conta tem 866 bytes contra os 82 clássicos; a contagem de extensões é 8; e `transferHook.programId` é `null` enquanto a taxa lê 0 bps com max 0. Esses dois últimos são o par "configurado, mas dormente". Se a sua contagem de extensões for diferente de 8, não presuma que a lição está errada nem que você está. O conjunto de extensões de um mint é fixado quando o mint é criado, salvo uma exceção estreita de metadados (a lição quatro constrói um argumento inteiro em cima disso), então uma contagem diferente quase sempre significa que o parser do seu RPC expõe entradas com nomes diferentes, que o endpoint te serviu um parse velho ou parcial, ou que o endereço foi digitado errado. Leia a lista que a sua rodada imprimiu e compare entrada por entrada.

**3. Aponte o mesmo decodificador para um mint clássico.** Copie o arquivo para `read-usdc.ts` e mude uma constante, para você ver como um mint de programa clássico aparece pela mesma lente:

```typescript
// in read-usdc.ts, replace the PYUSD address with classic USDC's mint:
const PYUSD_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
```

```bash
npx tsx@4.20.5 read-usdc.ts
```

Minha rodada hoje:

```text
mint:          EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
owner program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
account size:  82 bytes (a classic mint is 82)
decimals:      6
supply:        7923463957481104 base units
mintAuthority: BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG
freezeAuth:    7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar

extensions (0):

transferHook.programId: undefined
transferFee: undefined bps, max undefined
```

O owner começa com `Tokenkeg`, o tamanho é exatamente 82, a contagem de extensões é zero. E olhe aquelas duas linhas `undefined`: aquilo é o nosso script fazendo perguntas de Token-2022 para um mint clássico. Não é dormente, não é zero. Não existe slot para estar dormente. Um mint clássico não consegue nem representar os conceitos, que é a prova mais limpa que você vai conseguir de que esses são dois programas diferentes, não um programa com opções.

**4. Agora o segundo gancho. Salve isto como `transfer-cu.ts`.** Ele simula uma transferência de token clássico real entre duas contas ao vivo da mainnet e pergunta ao runtime quanto custou. Nada é assinado, nenhuma taxa é paga, nada vai parar na blockchain; `simulateTransaction` com `sigVerify: false` é uma pergunta de graça, e é a mesma disciplina de medir primeiro que você vai usar o curso inteiro. O meio do arquivo monta uma transação crua na mão para a gente não precisar de biblioteca nenhuma; trate essa parte como uma caixa-preta lacrada hoje. Você é a pessoa operando o instrumento, ainda não a pessoa que o construiu.

```typescript
// transfer-cu.ts - simulate a classic SPL Token transfer and read its compute cost.
// Zero npm dependencies. Run: npx tsx@4.20.5 transfer-cu.ts
// Builds a real Transfer instruction against live mainnet accounts and asks the RPC
// to simulate it. No signatures, no fees paid, nothing lands on chain.

const RPC = "https://api.mainnet-beta.solana.com";
const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // classic SPL USDC
const TOKEN_PROGRAM = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // classic SPL Token
// Any wallet that owns two USDC accounts and some SOL works. This is a well-known,
// long-lived exchange hot wallet; we borrow it on paper, simulation-only.
const WALLET = "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9";

async function rpc(method: string, params: unknown[]): Promise<any> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const json = await res.json();
  if (json.error) throw new Error(`${method}: ${JSON.stringify(json.error)}`);
  return json.result;
}

// base58 -> bytes (Solana address alphabet)
const ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function b58decode(s: string): Uint8Array {
  let n = 0n;
  for (const c of s) {
    const i = ALPHABET.indexOf(c);
    if (i < 0) throw new Error(`bad base58 char ${c}`);
    n = n * 58n + BigInt(i);
  }
  const out: number[] = [];
  while (n > 0n) { out.unshift(Number(n & 0xffn)); n >>= 8n; }
  for (const c of s) { if (c === "1") out.unshift(0); else break; }
  return new Uint8Array(out);
}

// shortvec length prefix used by the legacy transaction format
function compactU16(n: number): number[] {
  const out: number[] = [];
  do { let b = n & 0x7f; n >>= 7; if (n > 0) b |= 0x80; out.push(b); } while (n > 0);
  return out;
}

async function main() {
  // 1. Find two of the wallet's live USDC token accounts, funded one first.
  const res = await rpc("getTokenAccountsByOwner", [
    WALLET, { mint: USDC_MINT }, { encoding: "jsonParsed" },
  ]);
  const accounts = res.value
    .sort((a: any, b: any) =>
      Number(BigInt(b.account.data.parsed.info.tokenAmount.amount) -
             BigInt(a.account.data.parsed.info.tokenAmount.amount)));
  if (accounts.length < 2) throw new Error("need a wallet with two token accounts");
  const source = accounts[0].pubkey;
  const dest = accounts[1].pubkey;

  // 2. Hand-assemble a legacy transaction: one classic Transfer of 1 base unit.
  //    Keys in required order: writable signer, writables, then read-only.
  const keys = [WALLET, source, dest, TOKEN_PROGRAM].map(b58decode);
  const ixData = new Uint8Array(9);
  ixData[0] = 3; // Transfer discriminator in the classic SPL Token interface
  ixData[1] = 1; // amount = 1 as u64 little-endian (0.000001 USDC)

  const msg: number[] = [
    1, 0, 1, // header: 1 signer, 0 read-only signers, 1 read-only non-signer
    ...compactU16(keys.length), ...keys.flatMap((k) => [...k]),
    ...new Uint8Array(32), // blockhash placeholder; the RPC replaces it
    ...compactU16(1), // one instruction
    3, // program id index -> TOKEN_PROGRAM
    ...compactU16(3), 1, 2, 0, // accounts: source, dest, authority
    ...compactU16(ixData.length), ...ixData,
  ];
  const tx = new Uint8Array([...compactU16(1), ...new Uint8Array(64), ...msg]);

  // 3. Simulate. sigVerify:false means our 64 zero bytes pass as a "signature".
  const sim = await rpc("simulateTransaction", [
    Buffer.from(tx).toString("base64"),
    { sigVerify: false, replaceRecentBlockhash: true, encoding: "base64" },
  ]);

  console.log(`simulated: Transfer of 1 base unit (0.000001 USDC)`);
  console.log(`  from ${source}`);
  console.log(`  to   ${dest}`);
  console.log(`err:           ${JSON.stringify(sim.value.err)}`);
  console.log(`unitsConsumed: ${sim.value.unitsConsumed}`);
  for (const line of sim.value.logs ?? []) console.log(`  ${line}`);
}

main().catch((e) => { console.error(e); process.exit(1); });
```

**5. Rode e leia o medidor.**

```bash
npx tsx@4.20.5 transfer-cu.ts
```

Minha rodada, mesmo dia:

```text
simulated: Transfer of 1 base unit (0.000001 USDC)
  from 7KJjY7rArbydeLBF7gQ5LdqXRKRYyPArT99NEctsHsgU
  to   FzbcyEZ9m8xjtergWgWDq7mfPoHEbboBF791B6cTpzbq
err:           null
unitsConsumed: 76
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [1]
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 76 of 200000 compute units
  Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
```

Lá está, da boca do próprio runtime: `consumed 76 of 200000 compute units`. Não é mais uma afirmação minha. É sua. `err: null` significa que a transferência ia dar certo de verdade se fosse assinada; os endereços específicos de conta na sua rodada podem ser diferentes dos meus, já que o script escolhe ao vivo as duas maiores contas USDC da carteira.

**6. Checkpoint.** Você termina o lab quando consegue apontar para quatro coisas na saída do seu próprio terminal: os dois owner programs (`Tokenz...` e `Tokenkeg...`), a diferença de tamanho 866 contra 82, o hook `null` em uma extensão que existe, e a linha onde o runtime reporta 76 CU. Se em vez disso algum dos scripts falhou, as causas esmagadoramente prováveis são Node abaixo do 20 (sem `fetch`) ou o RPC público te limitando por taxa; espere trinta segundos e rode de novo, ou troque por qualquer endpoint de RPC que você já use.

![Um gráfico de barras mostrando Transfer caindo de 4,645 para 76 CU e TransferChecked de 6,200 para 105 depois da troca de motor, com os números atuais medidos ao vivo e os números históricos citados.](assets/v06-chart.webp)

## Challenge

Solo, sem walkthrough. Escreva o relatório que um colega de time conseguiria usar para agir, quatro linhas, com suas próprias palavras:

1. Quantas extensões TLV o mint do PYUSD carrega, e quais duas estão configuradas mas dormentes? Diga o que "dormente" significou concretamente nos bytes que você leu.
2. Quanto uma Transfer do SPL clássico custa em CU, segundo a sua própria simulação, e por que este curso se recusa a deixar você memorizar esse número?
3. Escolha mais um token, qualquer endereço de mint da sua própria carteira ou de um explorer. Aponte o `read-pyusd.ts` para ele e classifique: qual programa é dono dele, e quantas extensões ele carrega?
4. Uma frase: por que você não pode escolher uma primitiva de ativo que não consegue ler?

Se a linha 4 sair algo como "porque as regras de verdade da primitiva vivem nos bytes do mint, não no nome dela nem nos docs dela", você pegou a lição. Se sair "porque ler é bom em geral", rode a comparação com USDC de novo e olhe com mais atenção para as duas linhas `undefined`.

## O que você decodificou, e o que não decodificou

Uma honestidade rápida sobre o dia. Você não aprendeu os mecanismos das extensões, você não escreveu um decodificador, e você ainda não consegue dizer por que um hook ou um delegado permanente deixaria uma DEX nervosa. O que você fez: você leu um mint de produção ao vivo que a maioria dos holders dele nunca vai olhar, você sacou uma distinção entre programas que ainda morde engenheiros em atividade, e você mediu um custo de runtime em vez de citar um. Você produziu dois números que não consegue explicar, de propósito, a partir do zero, em menos de uma hora. Isso é uma habilidade de verdade, e é aquela em que todo o resto aqui se apoia.

Na próxima lição, o empréstimo acaba. Você constrói o decodificador sozinho, do mint pelado de 82 bytes até a caminhada completa do TLV, e ele vira o R1, o inspetor de mint, a primeira ferramenta do kit Overgrowth e aquela que o resto do módulo fica chamando. Traga os dois scripts de hoje; a gente vai abrir a caixa-preta.

Se alguma coisa nesta lição soou errada contra o que o seu próprio terminal imprimiu, confie no terminal e me avise. Esse reflexo também é o curso.

Boa decodificação! 🌱
