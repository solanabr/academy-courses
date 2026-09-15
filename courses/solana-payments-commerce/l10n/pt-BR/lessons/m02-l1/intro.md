# Anatomia de um pagamento em USDC: mints, ATAs e o transfer-kit

O módulo 1 te deu o modelo dos trilhos do dinheiro. Você mapeou autorização, captura e liquidação de cartão em `processed`, `confirmed` e `finalized`, decodificou uma transferência de USDC ao vivo no livro-razão público e escaneou um QR code de pagamento gerado pelo seu próprio terminal. A única coisa que você produziu e que sobrevive à lição é um documento: `commitment-policy.md`, a política de confirmação que você escreveu e defendeu. Deixe ele aberto do seu lado; o passo 6 pede que você confira uma linha de código contra ele, e o módulo 4 é onde ele vira código rodando.

Um aviso sobre a pasta, porque esta lição começa uma nova. O `wavelength-rails` do módulo 1 era espaço de rascunho: um punhado de scripts de sondagem descartáveis e o arquivo de política. Ele já fez o trabalho dele. Copie o `commitment-policy.md` para o workspace `wavelength` que você está prestes a criar, no nível de cima, e depois deixe o `wavelength-rails` quieto ou apague; nada mais adiante no curso lê dele. Daqui em diante, `wavelength` é o único diretório em que este curso vive, e todo comando diz se roda a partir dessa raiz ou de dentro de uma pasta de workspace.

Antes de um único conceito, rode isto em qualquer terminal com Node instalado:

```bash
node -e "console.log(2.01 * 10 ** 6, Math.trunc(2.01 * 10 ** 6))"
```

Você recebe de volta:

```
2009999.9999999998 2009999
```

Leia isso duas vezes. Um cliente digitou $2.01, você multiplicou por um milhão para chegar à menor unidade do USDC, e o JavaScript te entregou um número que está uma unidade a menos. Trunque, do jeito que metade dos tutoriais da internet faz, e você acabou de pagar a menor. Nenhum erro lançado, nenhum aviso, nada nos seus logs. As duas armadilhas sobre as quais ninguém avisa um engenheiro de produto são exatamente o que você está prestes a deixar para trás: matemática de dinheiro em ponto flutuante que manda em silêncio o número errado de centavos, e o fato de que a carteira que você está pagando pode nem ter uma conta de USDC ainda, então o pagamento falha antes de começar. No fim desta lição as duas armadilhas estão mortas, mortas por um módulo que você vai reusar em todas as lições restantes deste curso.

## Resumo

Esta é a primeira lição de construção do curso. O artefato é o `transfer-kit`: um módulo TypeScript que exporta `toBaseUnits` e `fromBaseUnits` (matemática de dinheiro exata, com inteiros), `resolveAta` (descobrir onde uma carteira guarda um token, offline) e `sendStablecoin` (uma transferência checada carregando um memo e uma reference key). Você vai mandar um pagamento real em USDC na devnet, imprimir a signature e a reference key dele, e depois provar que o pagamento existe localizando ele só a partir da reference, com um smoke check de verificação. Todo degrau de checkout mais adiante importa este kit, então as interfaces que você escrever hoje são estruturais.

Como o trabalho está dividido hoje: esta lição é um exemplo guiado com apoio completo, então digite junto comigo até o fim. O desafio de código no final, interpretar valores digitados pelo cliente, é a sua primeira peça de trabalho solo. Adicionar um segundo token ao kit é o fecho sem guia. Na próxima lição você recebe menos apoio, de propósito: o apoio chega com um buraco nele.

## O formato de um pagamento com stablecoin

### Um mint é o token; a sua carteira nunca guarda ele

Primeira definição, na hora exata: um **mint** é a conta on-chain que É um token. Uma conta define o USDC na Solana: a oferta dele, as casas decimais dele, quem pode criar mais. Todo saldo de USDC em qualquer lugar da blockchain aponta de volta para ela. Na mainnet essa conta é `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`, e ela declara 6 decimais. O USDT é outro mint, `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`. Na devnet, onde a gente constrói, a Circle publica um mint de USDC de teste em `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`. Três endereços, um formato só. Seu kit vai tratar "qual stablecoin" como um parâmetro, e é por isso que adicionar USDT hoje mais tarde vai te tomar cinco minutos.

Aqui está a parte que quebra o modelo mental que você trouxe dos trilhos de cartão. O endereço de carteira do seu cliente não guarda USDC. Não tem como. Uma carteira guarda SOL nativamente, e é só isso que ela guarda. Saldos de token moram em contas separadas, uma por dono e mint, e a casa padrão de um saldo se chama **conta de token associada**, a ATA. (O dono e o mint são as duas seeds com que você vai se importar hoje; existe uma terceira, o token program que é dono do mint, e ela vira estrutural na próxima lição. O código abaixo passa ela explicitamente, então não se assuste ao ver três argumentos onde o texto fala de duas.) A carteira da Alice é um endereço; o USDC da Alice mora em um segundo endereço; o USDT da Alice moraria em um terceiro. Quando você paga a Alice, tokens se movem entre contas de token, e a carteira dela assina pela conta que ela possui.

![A carteira da Alice e o mint do USDC apontam cada um para uma terceira conta, a conta de token associada dela, que é derivada desse par e guarda o saldo.](assets/v01-diagram.png)

Como você encontra a conta de USDC da Alice se ela nunca te disse o endereço dela? Você calcula. Uma ATA é um endereço derivado de programa: determinístico, calculado a partir do dono, do mint e do token program do mint, com um programa como dono, sem chave privada, guarde esse pensamento para o módulo 5. Para hoje, a consequência prática é o ponto inteiro: dada qualquer carteira e qualquer mint, o seu código consegue derivar o endereço exato da conta de token sem perguntar para ninguém, sem consulta, sem ida e volta de RPC, sem mensagem para o cliente. O `resolveAta` vai ter quatro linhas.

### A conta que o seu cliente talvez não tenha

Agora a armadilha. Derivável não é a mesma coisa que existente. Se o Bob nunca teve USDC, o endereço onde o USDC dele moraria é calculável mas está vago: não existe conta nenhuma ali. Mande tokens para um endereço vago e a transferência falha. Nos trilhos de cartão o banco adquirente garante que o destino existe; aqui, ninguém garante. Todo cliente de primeira viagem chega sem pista de pouso, e o seu checkout é quem percebe.

A correção é que quem paga pode criar a conta na mesma transação, e criar custa dinheiro de verdade: uma ATA tem 165 bytes e precisa de lamports suficientes depositados para ficar isenta de aluguel. No módulo passado eu te prometi a rubrica única que os trilhos de cartão nunca te mostraram. É esta, o aluguel da maquininha destes trilhos: um custo fixo de montar o caixa, não um pedaço de cada venda. O programa de conta de token associada ainda traz uma versão idempotente da instrução de criação dele, criar-se-não-existir, que transforma "a conta existe?" de uma pergunta que você faz numa pergunta que você nunca precisa fazer. Seu kit vai colocar ela na frente de todo pagamento, sem condição nenhuma: se a conta existe é um no-op, se não existe você acabou de financiar a pista de pouso do seu cliente.

Pergunte à rede qual é esse depósito em vez de confiar em qualquer número que você leia, inclusive os meus:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getMinimumBalanceForRentExemption","params":[165]}'
```

Na devnet, em 2026-09-07, isso respondeu `1488440` lamports, cerca de 0.0015 SOL; a mesma chamada contra a mainnet respondeu `1855569`, cerca de 0.0019 SOL. Duas coisas para tirar do fato de esses números serem diferentes. Primeiro, a alíquota do rent é um parâmetro de runtime do cluster, não uma constante do token program, então um número copiado de um tutorial é um número do cluster de outra pessoa em outro dia. Segundo, ela está caindo de verdade: o SIMD-0437 vem baixando a alíquota por byte em degraus (o mínimo isento de aluguel é `(128 + bytes) × rate`, e 165 + 128 = 293, que é por que todo tamanho de conta escala pelo mesmo fator quando a alíquota se move), a mainnet deu o primeiro degrau em 2026-09-03, a devnet já está um degrau à frente dela, e há mais degraus agendados. Material mais antigo — e frases mais antigas na própria história deste curso — cita 2,039,280 lamports para esta conta, o que estava correto antes daquele primeiro degrau e agora está uns 37% alto no cluster contra o qual os seus labs rodam — o mesmo movimento dito ao contrário é uma redução de 27%, e os dois números são fáceis de trocar. Rode o curl.

Esse é o trade-off, e eu quero ele na mesa antes de a gente construir. Unidades base inteiras são exatas mas implacáveis: a contabilidade de decimais que o atalho do float escondia agora é sua. E pagar um cliente novinho em folha pode exigir criar e custear o rent da ATA dele primeiro, um custo real de mais ou menos um milésimo de SOL e um modo de falha a mais em todo comprador de primeira viagem. Nenhum dos dois custos está escondido agora. É esse o acordo nestes trilhos, repetidas vezes: a maquinaria está exposta, e é você quem está segurando ela.

![Um fluxograma mostrando a derivação, um create idempotente incondicional e a transferência agrupados como uma única transação atômica, com a checagem de existência riscada.](assets/v02-flowchart.png)

### Dinheiro como inteiros, ou o bug do $2.01 dissecado

O one-liner da abertura falhou porque a blockchain não guarda 2.01. Ela guarda uma contagem inteira da menor unidade. Com 6 decimais, um USDC é 1,000,000 de **unidades base**, então $2.01 é exatamente 2,010,000 delas. Inteiros são a única representação honesta de dinheiro, e o token program só fala inteiros.

Os números comuns do JavaScript são floats de 64 bits, e floats não conseguem representar a maioria das frações decimais de forma exata. `2.01` é guardado como algo um fio abaixo de 2.01, multiplique por um milhão e você recebe `2009999.9999999998`, trunque e você está uma unidade a menos. Você pode remendar com `Math.round` e os seus testes vão passar, e aqui está por que esse remendo é uma bomba-relógio e não uma correção: floats só são exatos para inteiros até 2 elevado a 53, que é 9,007,199,254,740,992. Passando disso, doubles fisicamente não conseguem representar todo inteiro, então o erro de arredondamento deixa de ser fracionário e vira uma deriva silenciosa de unidades inteiras que nenhuma chamada de arredondamento recupera. Com 6 decimais esse teto fica lá pelos nove bilhões de USDC, o que parece inalcançável até você lembrar que unidades base também são como você vai somar volume diário, guardar saldos de tesouraria e processar lotes de pagamento. E para tokens de 9 decimais, o teto cai para uns nove milhões, bem dentro do saldo de uma única baleia. A regra que sobrevive a toda escala: matemática de dinheiro nunca toca em float, nem uma vez, nem no meio. Interprete a string decimal do cliente direto para um `bigint`.

![Três caminhos convertendo 2.01 em unidades base: o truncamento em float chega uma unidade a menos, o arredondamento aguenta até dois elevado a cinquenta e três, e a interpretação exata da string continua correta.](assets/v03-annotated-code.png)

Uma assimetria completa o modelo de dinheiro. A conversão corre em duas direções, e só uma delas é perigosa. Indo para dentro, da string do cliente para unidades base, é onde os pagamentos são feitos ou corrompidos, então essa direção recebe o tratamento paranoico. Indo para fora, de unidades base para uma string de exibição, é formatação honesta: `12500000` com 6 decimais vira `12.5`, feito com fatiamento de string sobre o mesmo princípio de não usar float, e se uma UI depois escolher exibir `12.50` com um zero à direita, isso é uma escolha de apresentação que não toca em nada. O kit entrega as duas direções como um par, `toBaseUnits` e `fromBaseUnits`, porque um sistema que só sabe codificar dinheiro e nunca auditar de volta é meio sistema, e o passo de verificação no fim do lab se apoia na viagem de volta para relatar o que de fato se moveu.

### TransferChecked, o memo e a reference key

Mais três peças e a teoria acaba. Leia elas como respostas a três perguntas de produto: como eu não mando a coisa errada, como eu rotulo um pagamento, e como eu encontro ele de novo depois?

Não mandar a coisa errada: o token program tem duas instruções de transferência, e o seu kit usa a estrita. O `Transfer` simples recebe uma origem, um destino e um valor inteiro, e confia em você para todo o resto. O **TransferChecked** recebe também o endereço do mint e os decimais, e o programa verifica os dois contra as contas on-chain, rejeitando a transação em qualquer divergência. Coloque o mint errado no seu config e a transação é rejeitada. Deixe os decimais divergirem entre o seu código e a realidade e ela é rejeitada de novo, em vez de mandar um pagamento errado por ordens de grandeza. O `Transfer` simples também está deprecado nas ferramentas atuais, então a escolha se faz sozinha: uma instrução que carrega a própria checagem de sanidade é exatamente o que o dinheiro merece.

![Uma tabela de comparação mostrando que o TransferChecked verifica o mint e os decimais on-chain e rejeita divergências enquanto o Transfer simples confia na sua configuração, e que o Transfer está deprecado nas ferramentas atuais.](assets/v04-comparison.png)

Rotulagem: um **memo** é uma instrução minúscula do memo program que anexa uma string curta à transação, de forma permanente e pública. IDs de pedido, referências de fatura, "wavelength-order-0001". Público é a palavra operante: nunca coloque nomes ou e-mails de clientes em um, o mundo inteiro consegue ler. Pense nele como o campo de referência de uma transferência bancária, menos a privacidade.

Encontrar de novo: esta aqui é a estrela discreta do curso inteiro. Uma **reference key** é um endereço novo e único, de 32 bytes, que você anexa à instrução de transferência como uma conta extra. Ela não assina nada, não recebe nada, não faz nada. Mas toda conta que aparece em uma transação vira pesquisável, então uma busca de signature nesse endereço devolve exatamente um pagamento: o seu. Gere uma reference única por pagamento e você deu a todo checkout um código de rastreio que a rede indexa de graça. Quando o seu cliente disser "eu paguei", você não vasculha o livro-razão torcendo para casar valores; você consulta a reference. O motor de conciliação inteiro do módulo 4 se apoia nesse truque, e o QR code que você escaneou no módulo passado já carregava uma.

![Um endereço de reference novo viaja junto com a transferência como uma conta extra inerte, o livro-razão indexa ele, e uma consulta nesse endereço devolve exatamente um pagamento.](assets/v05-diagram.png)

Um pedaço de história, porque este trio exato é mais velho do que parece. Quando a Shopify anunciou suporte a Solana Pay em 2023-08-23, com MonkeDAO, Mad Lads e Helius entre os primeiros usuários, o discurso era eliminar taxas bancárias, chargebacks e prazos de retenção, e o pagamento por baixo do capô era precisamente este: uma transferência checada de stablecoin mais uma reference key para casar. O caminho ativo hoje passa pelo plugin do MoonPay Commerce, mas a primitiva nunca mudou. Você está prestes a construir o mesmo pagamento push que chegou aos lojistas da Shopify, pequeno o bastante para caber em um arquivo.

## Lab: construa o transfer-kit

Numerado e mão na massa daqui em diante. Os passos 1 e 2 são preparação, secos de propósito; as decisões interessantes ganham o porquê delas no meio do texto.

### 1. Ferramentas, chaves e SOL de devnet

Você precisa do Node 24 ou mais novo (`node --version` para conferir) e da CLI da Solana. Node 24 não é um piso arbitrário: os clientes `@solana-program/*` fixados no próximo passo declaram `node >= 24`, e um runtime mais velho ganha um aviso `EBADENGINE` em toda instalação daqui em diante. Instale a CLI com o instalador da Anza:

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Reinicie o seu shell, depois crie um par de chaves e aponte a CLI para a devnet:

```bash
solana-keygen new --no-bip39-passphrase
solana config set --url devnet
solana airdrop 2
```

O `solana-keygen new` escreve um arquivo de par de chaves em `~/.config/solana/id.json`; esse arquivo é a sua identidade de lojista pelo resto do curso, e o kit vai carregar ele direto. Uma regra de higiene, dita uma vez e cedo: esta é uma chave de desenvolvimento em texto plano para dinheiro de brincadeira da devnet, então nunca mande fundos reais de mainnet para ela, e quando este curso tocar a mainnet mais adiante a forma de assinar muda antes de qualquer outra coisa. O airdrop te dá SOL de devnet de graça para taxas e rent. Se ele reclamar de rate limit, espere um minuto e tente de novo, ou use o faucet web em faucet.solana.com. Confirme com `solana balance`; qualquer número diferente de zero quer dizer que você está pronto.

Mais uma conta antes do código, porque o módulo 4 assume que você já tem ela. Vá em dashboard.helius.dev, crie a sua conta (o plano gratuito cobre tudo o que este curso faz) e copie a API key do painel. Coloque ela no perfil do seu shell para que ela sobreviva a um terminal novo:

```bash
echo 'export HELIUS_API_KEY=<paste-your-key>' >> ~/.zshrc   # or ~/.bashrc
source ~/.zshrc
echo $HELIUS_API_KEY   # must print your key, not an empty line
```

Você não vai usar ela hoje. A lição de webhook do módulo 4 cria um webhook de devnet com ela, e esse é o único lugar deste curso que precisa de uma; nenhuma chamada de RPC que você escrever usa. Seu RPC vai para endpoints públicos o tempo todo: devnet para tudo o que você manda, e mainnet só para leitura nas duas lições que inspecionam contas ao vivo.

### 2. O workspace, com versões fixadas

Crie o projeto em que o curso inteiro vive:

```bash
cd ~ && mkdir wavelength && cd wavelength
npm init -y
npm pkg set type="module"
mkdir -p transfer-kit/src
npm init -y --workspace transfer-kit
npm pkg set type="module" --workspace transfer-kit
npm install --workspace transfer-kit @solana/kit@^6.10.0 @solana-program/token@0.14.0 @solana-program/memo@0.11.2
npm install --workspace transfer-kit --save-dev typescript@^5.6.0 tsx@^4.19.0 @types/node
```

Esses pins não são os números mais novos do npm, e eu me recuso a te entregar um mistério, então aqui está de onde cada um vem. Em 2026-08-22, verificado contra o registro enquanto eu escrevia isto: o `@solana/kit` mais recente do npm é a linha 8.x, e o padrão amplo de peer do ecossistema é a v7. Fixamos a linha v6, que terminou em 6.10.0, porque o `@solana/pay`, a biblioteca que os nossos degraus de checkout adotam no módulo 3, declara uma peer dependency em kit ^6.9, e toda lição posterior importa este kit. O `@solana-program/token@0.14.0` é o último release compatível com kit v6 (o 0.15.0 pula a faixa de peer dele para kit ^7, e instalar ele ao lado do kit 6 é um erro do npm, não um aviso), e o `@solana-program/memo@0.11.2` é a mesma história para o cliente de memo. Pins de versão envelhecem, então trate este parágrafo como datado: quando você construir fora deste curso, leia as faixas de peer no npm e fixe no que o seu grafo de dependências de fato exige. Adicione um `tsconfig.json` na raiz do workspace:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["transfer-kit/src"]
}
```

Checkpoint: `npm ls --workspace transfer-kit @solana/kit` resolve para `@solana/kit@6.10.x` e não imprime nenhum erro de peer dependency. Se em vez disso o npm reportar um conflito `ERESOLVE`, um dos três pins derivou; conserte aqui, porque todo arquivo abaixo é construído contra exatamente estes.

### 3. Matemática de dinheiro exata: `amounts.ts`

Primeiro arquivo, e é nele que o bug da abertura morre. Crie o `transfer-kit/src/amounts.ts`:

```typescript
/** Exact decimal-string -> integer base units. No floats anywhere. */
export function toBaseUnits(amount: string, decimals: number): bigint {
  const match = /^(\d+)(?:\.(\d+))?$/.exec(amount.trim());
  if (!match) {
    throw new Error(`unparseable amount: "${amount}"`);
  }
  const [, whole, fraction = ''] = match;
  if (fraction.length > decimals) {
    throw new Error(
      `"${amount}" has ${fraction.length} decimal places; this mint supports ${decimals}`,
    );
  }
  const padded = fraction.padEnd(decimals, '0');
  return BigInt(whole) * 10n ** BigInt(decimals) + BigInt(padded || '0');
}

/** Integer base units -> display string, exact, trailing zeros trimmed. */
export function fromBaseUnits(units: bigint, decimals: number): string {
  const digits = units.toString().padStart(decimals + 1, '0');
  const whole = digits.slice(0, digits.length - decimals);
  const fraction = decimals === 0 ? '' : digits.slice(digits.length - decimals);
  const trimmed = fraction.replace(/0+$/, '');
  return trimmed ? `${whole}.${trimmed}` : whole;
}
```

O design em um fôlego: a entrada do cliente continua sendo uma string até o último momento possível, a gente separa no ponto decimal, preenche a fração com exatamente `decimals` dígitos e monta o resultado com aritmética de `bigint` que não tem como arredondar. Entrada precisa demais, sete casas decimais contra as seis do USDC, é rejeitada em alto e bom som em vez de arredondada em silêncio, porque um checkout que inventa arredondamento de sub-centavo é um checkout que concilia errado para sempre. Repare no que está ausente: `parseFloat`, `Number`, qualquer float, em qualquer lugar.

Checkpoint, direto da raiz do workspace:

```bash
npx tsx -e "
import { toBaseUnits } from './transfer-kit/src/amounts.ts';
console.log(toBaseUnits('2.01', 6));
console.log(toBaseUnits('0.000001', 6));
console.log(toBaseUnits('90071992547.409934', 6));
"
```

Você deve ver `2010000n`, `1n` e `90071992547409934n`. Esse terceiro valor está bem além do teto do float e é exato até o último dígito; o caminho do float dá `90071992547409920` para a mesma entrada. (O `tsx`, que a gente instalou no passo 2, roda TypeScript direto; é também o que os scripts `pay` e `verify` usam.)

### 4. Os mints que você aceita: `mints.ts`

```typescript
import { address } from '@solana/kit';

export const USDC_MAINNET = address('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v');
export const USDT_MAINNET = address('Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB');
export const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');

export const USDC_DECIMALS = 6;
export const USDT_DECIMALS = 6;
```

Seco de propósito. O `address()` valida a string na construção e te dá um tipo marcado em que o resto do kit confia, então um typo em um endereço de mint morre aqui em vez de dentro de uma transação. Este arquivo também é a porta de entrada do seu desafio solo: aceitar uma stablecoin nova começa com duas constantes.

### 5. Onde o dinheiro do cliente mora: `ata.ts`

```typescript
import type { Address } from '@solana/kit';
import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';

/** Where `owner` holds `mint`. Pure derivation: no network call, no signer. */
export async function resolveAta(owner: Address, mint: Address): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({
    owner,
    mint,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });
  return ata;
}
```

Quatro linhas, como prometido, e o comentário é a lição: isto é derivação, não consulta. O `findAssociatedTokenPda` calcula o endereço determinístico localmente a partir de três seeds, o dono, o mint e o token program que é dono do mint; a rede nunca é consultada.

O `async` vai te incomodar, e deve, porque em todo o resto deste curso `await` quer dizer uma ida e volta. Aqui não quer. Derivar o endereço quer dizer fazer hash das seeds, e o kit recorre à API de digest do Web Crypto da plataforma, que é assíncrona por especificação, sendo o trabalho local ou não. Então `await findAssociatedTokenPda(...)` é uma promise resolvida pela sua própria CPU, não por um nó de RPC, e os dois `await` dentro do `Promise.all` em `send.ts` não te custam nada além de um tick de microtask. Se existe ou não uma conta no endereço derivado é outra pergunta, e o caminho de envio está prestes a tornar ela irrelevante.

### 6. O envio: `send.ts`

A peça central. Crie o `transfer-kit/src/send.ts`:

```typescript
import {
  AccountRole,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type KeyPairSigner,
  type Rpc,
  type RpcSubscriptions,
  type Signature,
  type SolanaRpcApi,
  type SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import {
  getCreateAssociatedTokenIdempotentInstruction,
  getTransferCheckedInstruction,
} from '@solana-program/token';
import { getAddMemoInstruction } from '@solana-program/memo';
import { toBaseUnits } from './amounts.js';
import { resolveAta } from './ata.js';

export interface SendStablecoinParams {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  payer: KeyPairSigner;
  /** The recipient's WALLET address. We derive their token account ourselves. */
  recipient: Address;
  mint: Address;
  decimals: number;
  /** Human-typed decimal string, e.g. "12.50". Never a float. */
  amount: string;
  memo: string;
}

export interface StablecoinReceipt {
  signature: Signature;
  reference: Address;
  destinationAta: Address;
  baseUnits: bigint;
}

export async function sendStablecoin(
  params: SendStablecoinParams,
): Promise<StablecoinReceipt> {
  const { rpc, rpcSubscriptions, payer, recipient, mint, decimals, amount, memo } = params;

  const baseUnits = toBaseUnits(amount, decimals);
  const reference = (await generateKeyPairSigner()).address;
  const [sourceAta, destinationAta] = await Promise.all([
    resolveAta(payer.address, mint),
    resolveAta(recipient, mint),
  ]);

  // First-time recipient: this creates and rent-funds their ATA (payer pays,
  // the rent-exempt minimum for 165 bytes). If it already exists, the
  // idempotent variant is a no-op, not an error.
  const createDestination = getCreateAssociatedTokenIdempotentInstruction({
    payer,
    ata: destinationAta,
    owner: recipient,
    mint,
  });

  const transfer = getTransferCheckedInstruction({
    source: sourceAta,
    mint,
    destination: destinationAta,
    authority: payer,
    amount: baseUnits,
    decimals,
  });

  // The reference key rides along as an extra readonly non-signer account:
  // it changes nothing on-chain, but the payment is now findable by this address.
  const transferWithReference = {
    ...transfer,
    accounts: [
      ...transfer.accounts,
      { address: reference, role: AccountRole.READONLY },
    ],
  };

  const memoInstruction = getAddMemoInstruction({ memo });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) =>
      appendTransactionMessageInstructions(
        [createDestination, memoInstruction, transferWithReference],
        m,
      ),
  );

  const transaction = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(transaction);

  // maxRetries: 0n - WE own the retry policy. A retry must resubmit these same
  // signed bytes, never rebuild with a fresh blockhash (that mints a second payment).
  const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  await sendAndConfirm(transaction, { commitment: 'confirmed', maxRetries: 0n });

  return {
    signature: getSignatureFromTransaction(transaction),
    reference,
    destinationAta,
    baseUnits,
  };
}
```

Arquivo longo, três decisões que merecem a sua atenção; o resto é o pipeline padrão de montar-assinar-enviar do kit e pode correr seco.

Decisão um, a ordem das instruções: criar-se-não-existir, depois o memo, depois a transferência checada. A transferência vai por **último** e o memo imediatamente antes dela, e isso não é estética: o `validateTransfer` do `@solana/pay` tira a última instrução e exige que ela seja a transferência de token, depois tira a anterior e exige que ela seja o memo. Todo validador rio abaixo neste curso, incluindo a trava de aceitação do módulo 3 e a maquininha em `m03-l3`, herda essa regra. As três ainda viajam em uma transação só, então continua atômico: ou o Bob ganha uma conta e o dinheiro e o rótulo, ou nada acontece. Não existe estado em que você custeou o rent de uma conta para um pagamento que falhou.

Decisão dois, onde colocar a reference. A gente gera um par de chaves descartável puramente para colher um endereço novo e único, depois acrescenta ele à lista de contas da instrução de transferência como readonly não signatário. O programa ignora; o livro-razão indexa. Dez caracteres de código, e todo pagamento que este kit mandar tem um código de rastreio.

Decisão três, a postura de retry, e esta semeia uma regra em que o curso inteiro se apoia. Ela precisa de uma definição antes, porque o `setTransactionMessageLifetimeUsingBlockhash` passou uma palavra estrutural por você sem que você percebesse.

Um **blockhash** é um hash de um bloco recente, e toda transação precisa carregar um. Ele não é um nonce e não é um timestamp; é um prazo de validade. A rede lembra dos últimos 150 blockhashes e rejeita qualquer transação cujo blockhash caiu do fim dessa lista, que é o que impede uma transação assinada de ficar pendurada para sempre e aterrissar meses depois. Então o blockhash que você busca quando monta é um tempo de vida: esta transação é válida até aquele hash envelhecer, e aí ela está morta, de forma permanente e segura.

Agora a regra de retry. A gente passa `maxRetries: 0n`, dizendo ao RPC para não reenviar por conta própria, porque reenvio cego pertence a quem guarda o recibo, e esse é você. Aqui está a propriedade que torna os retries seguros: a signature de uma transação assinada é uma função dos bytes exatos dela, então reenviar os mesmos bytes assinados sempre produz a mesma signature, e a rede vai aterrissar essa signature no máximo uma vez. Remontar o pagamento com um blockhash novo, em vez disso, cria uma signature nova, e se o primeiro envio de fato aterrissou enquanto você estava dando timeout, parabéns, você pagou duas vezes. Então a regra: no timeout, reenvie os bytes idênticos, e só remonte depois que o blockhash tiver expirado de verdade E a blockchain confirmar que o original nunca aterrissou. Duas condições, as duas checadas abaixo.

"Expirado de verdade" é uma coisa que você checa, não uma duração que você chuta. O RPC responde direto:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"isBlockhashValid","params":["<YOUR_BLOCKHASH>",{"commitment":"finalized"}]}'
```

Enquanto `result.value` for `true`, os seus bytes originais ainda podem aterrissar, então não remonte. No momento em que ele vira `false`, aquela transação em particular está morta: ela nunca mais pode aterrissar. Morta não é a mesma coisa que nunca-aterrissou, porém, e esta é a metade da regra que custa dinheiro às pessoas. O blockhash pode ter expirado *depois* de a sua transação ter liquidado em silêncio enquanto você dava timeout. Então a expiração é a primeira de duas checagens, não a checagem inteira — antes de remontar, pergunte à blockchain se o original já aterrissou:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSignatureStatuses","params":[["<YOUR_SIGNATURE>"],{"searchTransactionHistory":true}]}'
```

Um null em `result.value[0]` com a busca em histórico ligada quer dizer que ela nunca aterrissou, e só aí remontar é seguro. Um objeto de status quer dizer que você foi pago e que o retry que você estava prestes a mandar teria cobrado do seu cliente uma segunda vez. Expirado mais nunca-aterrissou: os dois, toda vez. O número de relógio de parede, cerca de 150 blocos e portanto uns 45 segundos no atual slot time alvo de 300ms, é útil para dimensionar um timeout mas não é a checagem; derive ele do slot time em vez de decorar os segundos, porque o slot time é o número que se move, e ele vem se movendo rápido: o SIMD-0525 já cortou ele duas vezes em uma semana, com mais dois cortes escalonados travados no código. A fila de pagamentos offline do módulo 8 desvia deste prazo por completo — uma barraca de feira não consegue rechecar um blockhash que ela não alcança, então ela assina contra um nonce durável em vez disso; hoje o kit só se recusa a ser sorrateiro sobre o prazo, e se você algum dia precisar da checagem, o curl acima é ela.

![Duas linhas do tempo sobre a janela de 150 blocos: reenviar os mesmos bytes mantém uma signature que aterrissa no máximo uma vez, enquanto remontar cria uma segunda signature e cobra do cliente duas vezes.](assets/v06-timeline.png)

Mais uma passada no encanamento, porque este é o seu primeiro contato com o pipeline do kit e ele vai se repetir em todo arquivo que toca a rede neste curso. A cadeia de `pipe` monta uma *message* de transação: uma descrição do que deve acontecer, de quem paga a taxa e de qual blockhash ancora o tempo de vida dela. Nada na message é final; você pode continuar transformando. O `signTransactionMessageWithSigners` é a porta de mão única: ele percorre a message, encontra toda conta que precisa assinar (o nosso payer cobre tanto a taxa quanto a transferência de token), assina e congela os bytes. Depois dessa porta, a transação É os bytes dela, que é exatamente por que a regra de reenvio funciona: a signature foi calculada sobre eles, então os bytes e a signature nunca conseguem se separar. O `sendAndConfirmTransactionFactory` então junta as duas conexões de RPC que você passou, HTTP para enviar e uma assinatura websocket para ouvir de volta, e bloqueia até a rede reportar o nível de commitment que você pediu. A gente pede `confirmed`. Agora abra o `commitment-policy.md` e confira isso contra o que você escreveu embaixo de `Everyday payments`: um pagamento de teste de $1.25 cai bem debaixo desse título, e se o seu arquivo dizia `finalized` ali, mude a string no código para bater com a sua política em vez de mudar a sua política para bater com o meu código. Este é um momento pequeno e é a única vez que esta lição toca o arquivo, mas é o hábito que o módulo 4 automatiza: o nível de commitment no código é consequência de uma política escrita, nunca um padrão que alguém digitou uma vez.

![Um pipeline de três fases: a message mutável ganha um fee payer, um tempo de vida por blockhash e instruções, depois o ato de assinar congela os bytes, e a transação imutável é enviada e confirmada.](assets/v07-flowchart.png)

Amarre os exports no `transfer-kit/src/index.ts`:

```typescript
export { toBaseUnits, fromBaseUnits } from './amounts.js';
export { resolveAta } from './ata.js';
export { sendStablecoin } from './send.js';
export type { SendStablecoinParams, StablecoinReceipt } from './send.js';
export * from './mints.js';
```

Checkpoint antes de você gastar qualquer coisa: rode `npx tsc --noEmit` a partir da raiz `wavelength`. O silêncio é a aprovação. Cinco arquivos, nenhuma saída, o kit compila.

### 7. Mande um pagamento de verdade: `pay.ts`

Duas coisas antes do script. Primeiro, USDC de devnet: a Circle mantém um faucet em faucet.circle.com, escolha Solana Devnet, cole o seu endereço vindo do `solana address`, e ele te manda uma pequena cota de teste para a ATA certa. Segundo, uma segunda carteira para pagar: `solana-keygen new --no-bip39-passphrase -o /tmp/customer.json`, e depois `solana-keygen pubkey /tmp/customer.json` imprime o endereço de carteira do seu cliente de mentira. Ela não tem conta de USDC, que é exatamente o que a gente quer: o seu primeiro pagamento exercita o caminho do comprador de primeira viagem de propósito.

Crie o `transfer-kit/src/pay.ts`:

```typescript
import { readFile, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
} from '@solana/kit';
import { sendStablecoin } from './send.js';
import { USDC_DEVNET, USDC_DECIMALS } from './mints.js';

const recipientArg = process.argv[2];
const amountArg = process.argv[3] ?? '1.25';
if (!recipientArg) {
  console.error('usage: npm run --workspace transfer-kit pay -- <recipient-wallet> [amount]');
  process.exit(1);
}

const keyfile = `${homedir()}/.config/solana/id.json`;
const bytes = new Uint8Array(JSON.parse(await readFile(keyfile, 'utf8')));
const payer = await createKeyPairSignerFromBytes(bytes);

const rpc = createSolanaRpc('https://api.devnet.solana.com');
const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

const receipt = await sendStablecoin({
  rpc,
  rpcSubscriptions,
  payer,
  recipient: address(recipientArg),
  mint: USDC_DEVNET,
  decimals: USDC_DECIMALS,
  amount: amountArg,
  memo: `wavelength-order-0001`,
});

console.log('signature :', receipt.signature);
console.log('reference :', receipt.reference);
console.log('base units:', receipt.baseUnits.toString());

await writeFile(
  new URL('../receipt.json', import.meta.url),
  JSON.stringify(
    {
      signature: receipt.signature,
      reference: receipt.reference,
      destinationAta: receipt.destinationAta,
      baseUnits: receipt.baseUnits.toString(),
    },
    null,
    2,
  ),
);
```

Ligue os scripts e rode:

```bash
npm pkg set scripts.pay="tsx src/pay.ts" scripts.verify="tsx src/verify.ts" --workspace transfer-kit
npm run --workspace transfer-kit pay -- $(solana-keygen pubkey /tmp/customer.json) 1.25
```

Saída esperada, com os seus próprios valores:

```
signature : 3Qx...long base58 string
reference : 7pK...a fresh address
base units: 1250000
```

Essa signature é um pagamento de devnet ao vivo que você pode colar em qualquer explorer, e você deveria, porque você já esteve do outro lado exato deste vidro. No módulo 1 você decodificou a transferência de USDC de outra pessoa; agora decodifique a sua. Ponha o explorer na devnet, abra a transação e leia a anatomia que você acabou de escrever: três instruções em ordem, o create aterrissando a conta do Bob, a string do memo sentada ali em público exatamente como avisado, e a transferência checada com um endereço extra pendurado nela sem fazer nada (acene para a sua reference key). O script também deixa cair um `receipt.json` ao lado do código, que é deliberadamente primitivo: ele faz as vezes da tabela de pedidos que o seu backend de verdade vai manter, e o passo de verificação está prestes a consumir ele. Aproveite o momento se quiser; o módulo 1 foi todo preparação para essas três linhas de saída.

### 8. Prove que aterrissou: `verify.ts`

Um pagamento que você não consegue verificar de forma independente é um pagamento que você não tem. O smoke check faz o papel do-você-de-amanhã-de-manhã: ele ignora tudo menos a reference key e o valor esperado, encontra o pagamento só a partir da reference e recalcula o que de fato chegou a partir dos próprios registros de saldo da blockchain. Crie o `transfer-kit/src/verify.ts`:

```typescript
import { readFile } from 'node:fs/promises';
import { address, createSolanaRpc, signature as asSignature } from '@solana/kit';
import { fromBaseUnits } from './amounts.js';
import { USDC_DECIMALS } from './mints.js';

const rpc = createSolanaRpc('https://api.devnet.solana.com');

const receipt = JSON.parse(
  await readFile(new URL('../receipt.json', import.meta.url), 'utf8'),
) as { signature: string; reference: string; destinationAta: string; baseUnits: string };

// 1. Locate the payment by its reference key, NOT by the signature we stored.
//    This is the whole point: reconciliation must work from the reference alone.
const reference = address(receipt.reference);
const found = await rpc.getSignaturesForAddress(reference, { limit: 10 }).send();
if (found.length === 0) {
  throw new Error(`no transaction found for reference ${receipt.reference}`);
}
const sig = found[0].signature;
if (sig !== receipt.signature) {
  throw new Error(`reference resolves to ${sig}, expected ${receipt.signature}`);
}

// 2. Fetch it and confirm it succeeded.
const tx = await rpc
  .getTransaction(asSignature(sig), { encoding: 'json', maxSupportedTransactionVersion: 1 })
  .send();
if (!tx || !tx.meta) throw new Error('transaction not found on devnet');
if (tx.meta.err) throw new Error(`transaction failed: ${JSON.stringify(tx.meta.err)}`);
const meta = tx.meta;

// 3. Recompute the received amount from token balances, in exact base units.
const keys = tx.transaction.message.accountKeys;
const destinationIndex = keys.findIndex((k) => k === receipt.destinationAta);
const balanceAt = (balances: typeof meta.preTokenBalances): bigint => {
  const entry = balances?.find((b) => b.accountIndex === destinationIndex);
  return entry ? BigInt(entry.uiTokenAmount.amount) : 0n;
};
const delta = balanceAt(meta.postTokenBalances) - balanceAt(meta.preTokenBalances);

if (delta !== BigInt(receipt.baseUnits)) {
  throw new Error(`recipient received ${delta}, expected ${receipt.baseUnits}`);
}

console.log('devnet transfer confirmed');
console.log(`located by reference ${receipt.reference}`);
console.log(`amount: ${delta} base units (${fromBaseUnits(delta, USDC_DECIMALS)} USDC)`);
```

Duas notas de design antes de rodar. A consulta pede até dez signatures, mas o esquema inteiro só funciona se ela sempre achar uma, e isso é disciplina, não esperança: o kit gera uma reference nova por pagamento e nunca reusa. Reuse uma reference entre pagamentos e o `getSignaturesForAddress` passa a devolver uma lista que você tem que desambiguar, o que é conciliação com passos a mais e um viveiro de bugs sutis; um pagamento, uma reference, para sempre. Segundo, a verificação deliberadamente não confia no valor do seu próprio recibo como fonte da verdade. Ela puxa o saldo da conta de destino antes e depois a partir dos metadados da transação e tira a diferença, em strings cruas de unidade base convertidas direto para `bigint`, com floats excluídos da trilha de auditoria tão completamente quanto do caminho do pagamento. Rode:

```bash
npm run --workspace transfer-kit verify
```

```
devnet transfer confirmed
located by reference 7pK...your reference
amount: 1250000 base units (1.25 USDC)
```

Esse é o checkpoint em que a lição inteira trava. Se a consulta não devolver nada, o suspeito de sempre é um RPC que não se atualizou; espere alguns segundos e rode de novo. Se a checagem de valor lançar erro, leia os dois números do erro, porque um deles veio do seu código e o outro veio da blockchain, e não é a blockchain que está errada.

![A verificação flui da reference key pela consulta de signature e pela busca da transação até um delta de saldo em unidades base, com saídas de falha para uma signature ausente, uma transação que falhou ou um valor divergente.](assets/v08-flowchart.png)

### 9. Quanto aquele pagamento custou de verdade

Feche o ciclo com o resumo de custos que o módulo 1 te prometeu. Seu pagamento carregou uma signature, então a taxa base foi de 5000 lamports, uma cobrança fixa, uma fração pequena de um centavo a qualquer preço de SOL que você queira colocar na conta. E porque o seu cliente de mentira nunca tinha tido USDC, a transação também depositou o mínimo isento de aluguel de 165 bytes que você consultou no começo desta lição — a rubrica do aluguel da maquininha, paga por você como fee payer. Não acredite em mim sobre quanto saiu da sua carteira: `solana balance` antes e depois de um pagamento para um endereço novo te mostra a taxa e o rent juntos, e subtrair a resposta do curl deixa a taxa. Rode o mesmo pagamento de novo para o mesmo cliente e veja a anatomia mudar: o create idempotente vira um no-op, sem rent, só a taxa de 5000 lamports. Compradores de primeira viagem te custam uma fração de centavo mais a pista de pouso; compradores que voltam custam só a fração.

Afaste a câmera por um parágrafo, porque o formato aqui importa mais do que os tamanhos. O custo recorrente de receber um pagamento nestes trilhos é fixo e minúsculo em qualquer tamanho de ticket, e o único custo relevante é uma despesa de capital única, por cliente, que compra um pedaço permanente de infraestrutura on-chain para aquela relação. Os trilhos de cartão te cobram uma porcentagem para sempre e não te devolvem nada durável. Aqui você paga centavos uma vez e todo pagamento seguinte daquele cliente viaja quase de graça. Para um negócio com clientes recorrentes, isso não é um desconto, é um modelo de custo diferente — leve isso com você para o módulo 6, onde modelos de custo como este são pesados quando a gente escolhe um trilho por corredor.

![Barras empilhadas comparam um cliente de primeira viagem pagando a taxa base de 5000 lamports mais o depósito único de rent da ATA, mais ou menos um milésimo de SOL, contra um cliente recorrente pagando só essa taxa fixa.](assets/v09-chart.png)

## Challenge

Duas partes, em ordem crescente de solidão.

**O desafio de código, a sua primeira repetição solo.** O `toBaseUnits` lida com um número que o seu código produziu. Agora lide com um número que um humano digitou. O degrau abaixo pede um segundo export para o `amounts.ts`, `parseAmount(input: string, decimals: number)`, que recebe a entrada crua do formulário de checkout e devolve ou as unidades base exatas ou o motivo da recusa: `{ ok: true, baseUnits }` ou `{ ok: false, reason }`, nunca um erro lançado. Esse tipo de retorno é a decisão de design inteira, então seja deliberado com ele. O `toBaseUnits` lança porque código do kit chama ele com uma string que o kit produziu, onde uma string ruim é um bug e parar o programa é a resposta correta. O `parseAmount` fica na fronteira com o cliente, onde entrada ruim é tráfego comum, e recusar é um desfecho normal e não uma emergência, então ele devolve algo que o checkout consegue renderizar ao lado do campo em vez de uma exceção que a UI tem que capturar para continuar viva. Os motivos são um conjunto fechado, `empty`, `malformed`, `negative`, `too-precise` e `too-large`, precisamente para que o formulário consiga mapear cada um para uma frase que um cliente entende. Isto é trabalho genuinamente novo, não uma redigitação do lab: o `toBaseUnits` pode presumir uma string decimal limpa, e este aqui recebe o que quer que o teclado de um cliente tenha produzido.

Algumas fronteiras a que ele te obriga. `" 12.5 "` vira `12500000n` com 6 decimais, porque espaço em branco antes e depois é artefato de colar, não erro, e `"0012.50"` vira a mesma coisa, já que zeros à esquerda são feios mas não errados. `"$12.50"` e `"12abc"` são `malformed`; `"."` também, que não tem dígito nenhum dos dois lados. Um campo intocado, ou um que só tem espaços, é `empty`, um motivo separado para que o formulário consiga dizer "digite um valor" em vez de "isso não é um número". `"-5"` é `negative`, porque um checkout nunca cobra abaixo de zero. `"7.071"` no mint de 2 decimais que você está prestes a criar é `too-precise`, e repare que essa mesma string é perfeitamente válida no USDC: precisão é uma propriedade do mint, não do parser. E existe um teto além de um piso, porque valores de token SPL são `u64`: `18446744073709.551615` com 6 decimais é o maior valor que qualquer transferência consegue carregar, e uma unidade base a mais é `too-large` antes mesmo de chegar a uma instrução.

A fronteira que a maioria das implementações erra é a de precisão demais, e vale desacelerar por ela. `"12.5000001"` em um mint de 6 decimais é `too-precise`, porque aquele sétimo dígito é dinheiro que o mint não consegue guardar e truncar ele inventa arredondamento de sub-centavo que concilia errado para sempre. Mas `"1.5000000"` **não** é preciso demais. Ela também tem sete dígitos de fração, e o sétimo é um zero, então o valor é exatamente 1500000 unidades base e exatamente representável. Contar os caracteres da fração é a checagem fácil e ela recusa um pagamento perfeitamente bom; perguntar se algum dos dígitos extras é diferente de zero é a checagem que é de fato verdadeira. (O `toBaseUnits` do lab conta caracteres, deliberadamente: ele é alimentado pelo seu próprio código, então um dígito a mais ali quer dizer bug e deve ser barulhento. A regra tolerante pertence à fronteira humana e só a ela.)

Depois o caso que o degrau não consegue testar e que você deve implementar mesmo assim: `"12,50"`. Essa vírgula é o que o seu teclado entrega por padrão e é o que os seus clientes vão digitar o dia inteiro. Ela é `malformed`, e o parser precisa recusar em vez de, prestativamente, arrancar a vírgula, virar `1250` e cobrar cem vezes o ticket. Ela está ausente dos casos abaixo por um motivo de encanamento, não pedagógico: o formato de caso que alimenta a sua função com os argumentos é ele mesmo separado por vírgula, então uma vírgula dentro de um valor entre aspas não sobrevive à viagem. A regra vale do mesmo jeito. E mais uma regra que nenhum caso consegue checar também, e que você impõe a si mesmo: nada de `parseFloat` e nada de `Number` em lugar nenhum do parse, porque uma versão que passa usando float passa por sorte.

**O fecho solo, sem guia.** Torne real o segundo token do kit. O `mints.ts` já carrega `USDT_MAINNET`, mas a Tether não publica mint oficial de devnet, então faça o papel dele do jeito que os times de integração fazem, e já que você está aí transforme isso num teste de verdade: crie o seu próprio mint de treino com **2 decimais**, não 6, para que o seu kit tenha mesmo que respeitar a precisão do próprio mint em vez de concordar em silêncio com a do USDC. Use a CLI do SPL token (`spl-token --version` para confirmar que você tem, `cargo install spl-token-cli` se a sua instalação não trouxe; depois `spl-token create-token --decimals 2 --url devnet`, `spl-token create-account` e `spl-token mint` um saldo para você mesmo), registre ele ao lado das constantes de verdade, e mande `7.07` para a sua carteira de cliente. A aceitação é a mesma barra em que a lição travou, agora para duas precisões distintas: valores corretos em unidades base exatas, e cada pagamento localizado pela própria reference através da checagem de verificação. Olhe o `verify.ts` em particular, porque ele formata a saída dele com um `USDC_DECIMALS` hardcoded, e um mint de 2 decimais vai fazer aquela linha mentir para você. Consertar isso é a refatoração de hoje à noite.

Quando as duas partes passarem, você passou da barra do módulo: dois tokens, valores exatos, todo pagamento encontrável pela reference.

Este kit é pequeno, e isso é a conquista, não uma limitação; o checkout inteiro da loja de discos vai se apoiar nestes quatro exports. Se o caminho de envio brigou com você, os dois culpados de sempre são um saldo de USDC de devnet vazio (o faucet resolve) e um timeout de RPC no meio da confirmação (reenvie os mesmos bytes, agora você sabe por quê). Leve qualquer coisa mais estranha para a comunidade do curso, de preferência com a sua reference key anexada, porque agora você fala em códigos de rastreio.

Então, o transfer-kit move USDC, e você tem recibos. Na próxima lição um cliente entra segurando PYUSD, e PYUSD não é um token clássico: ele vive no Token-2022 com oito extensões penduradas no mint, e o seu kit como está escrito mira o pagamento no programa completamente errado. A gente vai ler esse mint ao vivo e ensinar algumas boas maneiras ao kit.
