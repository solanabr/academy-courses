# Ponha um paywall na API de preço de prensagem, depois construa o bot que paga por ela

## Resumo

A lição passada te entregou o fluxo x402 v2 no papel: os três headers que carregam ele, PAYMENT-REQUIRED na ida, PAYMENT-SIGNATURE na volta, PAYMENT-RESPONSE na ida de novo, o scheme exact-SVM, e a resposta exata para quem assina no /verify contra o /settle. O tempo de papel acabou. Hoje você constrói as duas pontas e liga os memos no livro-razão que você já opera.

Aqui está a situação que faz valer a pena construir. A Wavelength publica uma API de preço de prensagem: dê a ela um disco e um tamanho de tiragem, e ela cota quanto custa prensar o vinil. Ela é grátis, e um bot de colecionador está martelando ela dez mil vezes por dia sem pagar nada, empurrando a sua fatura de RPC para cima enquanto te paga zero. A correção não é um banimento. A correção é um preço. Um middleware na frente do endpoint, e o mesmo bot que ontem pegava carona de graça ou começa a pagar hoje, porque as cotações valem mais para o operador dele do que os centavos que agora custam, ou vai embora, e de todo jeito a carona de graça acaba.

Primeiro, suba o workspace. Ele fica ao lado do workspace `backoffice` do módulo 4, porque o caminho de import entre os dois é o ponto inteiro desta lição:

```bash
mkdir -p ~/wavelength/x402/src && cd ~/wavelength/x402
npm init -y && npm pkg set type="module"
npm install @x402/express@2.23.0 @x402/svm@2.23.0 @x402/core@2.23.0 \
  express@4.21.2 @solana/kit@6.10.0
npm install -D typescript tsx @types/node @types/express
```

Notas de pin, porque essa linha se mexe rápido: 2.23.0 é a linha `@x402/*` carimbada — publicada no npm em 2026-08-18, verificada como instalável em 2026-09-05 — e os nomes com escopo são os únicos a instalar; confira a tag de novo antes de fixar. `@x402/svm` tem `@solana/kit >=5.1.0` como peer, e este workspace fixa o kit em 6.10.0, a mesma linha v6 dos seus workspaces de checkout e de ops. O express está fixado dentro da faixa de peers `^4 || ^5` do `@x402/express` — e sim, 4.21.2 é um major de express diferente do 5.x que os seus workspaces de checkout, back office e capstone rodam, que é a mesma regra por workspace dos majors do kit: cada workspace fixa dentro das faixas de peers das próprias dependências, e as duas linhas de express nunca se encontram num mesmo `node_modules`.

Espere três linhas de `npm warn ERESOLVE overriding peer dependency` nessa instalação, e espere elas toda vez: `@x402/svm` depende de `@solana-program/token` 0.9, `@solana-program/token-2022` 0.6 e `@solana-program/compute-budget` 0.11, e os três ainda declaram `@solana/kit ^5.0` como peer. O npm resolve eles contra o kit 6.10.0 da sua raiz e avisa em vez de falhar (`npm ls @solana/kit` vai imprimir eles como `invalid: "^5.0"`). Avisos, não erros: a instalação completa e o lab roda. É também um retrato justo de quão nova é esta stack, e um motivo para reler esses avisos a cada bump em vez de treinar o olho para pular eles.

O que você leva embora, enquanto essa instalação roda:

- **A ponta do servidor**: o endpoint de preço de prensagem da Wavelength atrás do `@x402/express`, apontado para o facilitador da devnet, com um id de fatura `extra.memo` por chamada em todo desafio.
- **A ponta do cliente**: um agente pagador em cima do `@x402/svm` que engole o 402, assina parcialmente, refaz a chamada com o header de pagamento e guarda um recibo.
- **Conciliação por acréscimo**: os ids de fatura do memo caem no mesmo livro-razão do back office que você construiu no módulo 4. Uma venda de máquina e uma venda de humano correm pelo mesmo caminho de código.
- **Duas guardas**: o teto de spendControls do cliente (e a cilada da recusa silenciosa dele), e a história de por que o verificador do x402 embarca uma allowlist hardcoded para um programa de guarda de carteira.

A divisão de trabalho, em voz alta: eu percorro o encanamento do servidor e o hook de conciliação com você de ponta a ponta. A tabela de preços do middleware e o loop de pagar-e-repetir do agente são seus para preencher contra regras declaradas, modo Completion com as chamadas nomeadas. A guarda pre-flight `decidePayment` do Challenge é solo, sem walkthrough.

## Medindo uma chamada dos dois lados

### A tabela de preços que o middleware impõe

Comece pelo servidor, porque o servidor é onde mora a decisão sobre dinheiro. O `@x402/express` expõe `paymentMiddleware(routes, resourceServer)`: um objeto de rotas que diz o que custa o quê, e um resource server que sabe verificar e liquidar. Aqui está o formato das rotas, o de verdade vindo dos tipos da 2.23.0, percorrido campo a campo:

```ts
// The shape of one protected route (RoutesConfig entry, @x402/core 2.23.0)
const example = {
  'GET /price': {
    accepts: {
      scheme: 'exact',                                   // per-call, precise amount
      network: 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1', // CAIP-2 id: devnet
      payTo: 'YOUR_MERCHANT_ADDRESS',                    // the OWNER address; the scheme derives the ATA
      price: '$0.05',                                    // money form; resolved to devnet USDC
      extra: { memo: 'WVL-INV-0001' },                   // the invoice id, 256-byte ceiling
    },
    description: 'Wavelength pressing-price quote',
  },
};
```

Cada campo é uma decisão para a qual você já tem o vocabulário. `scheme: 'exact'` é o scheme de API medida da lição passada, uma chamada, uma liquidação. `network` é o id CAIP-2 da devnet; o id de mainnet `solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp` está a uma troca de string de distância, e todo o resto desta lição sobrevive a essa troca menos o facilitador, no qual vamos chegar. `payTo` recebe o endereço de dono do lojista, não uma token account: o scheme SVM deriva a associated token account sozinho, a mesma distinção entre dono e ATA que o seu verificador do módulo 4 impõe. `price` na forma de dinheiro `'$0.05'` é parseado pelo scheme contra a tabela de ativos embutida nele, que na devnet resolve para o mint de USDC de devnet `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`, seis casas decimais, o mesmo mint que o seu checkout usa desde o módulo 2. Você pode passar um par `{ asset, amount }` explícito no lugar quando precifica num token específico; a forma de dinheiro é o padrão porque uma tabela de preços em dólares é o que um lojista de fato mantém.

E `extra.memo` é o campo em torno do qual esta lição orbita: uma string de no máximo 256 bytes que o agente pagador tem que embutir na transação de pagamento como uma instrução de memo, verificada byte a byte pelo facilitador. É o seu id de fatura, pegando carona no próprio pagamento. Repare que o teto é medido em bytes UTF-8, não em caracteres; um id de fatura com caracteres multibyte gasta o orçamento mais rápido do que o comprimento dele sugere, que é por que o Challenge faz você medir isso direito.

![Mapeamento campo a campo da config de rota do lojista para o PaymentRequirements que o agente decodifica do header PAYMENT-REQUIRED, com o ativo em unidades base, um maxTimeoutSeconds que o SDK deixa em 300 por padrão, um fee payer fornecido pelo facilitador, e um rodapé marcando maxAmountRequired e amount como uma fronteira de v1 para v2.](assets/v01-comparison.png)

Uma coisa para absorver antes que ela te custe uma tarde, e absorva como uma fronteira de versão, não como uma deriva: o campo de valor se chama `maxAmountRequired` na v1 e `amount` na v2. Isso não é a spec discordando do SDK. Abra o `@x402/core` 2.23.0 e os dois schemas estão no mesmo build, `PaymentRequirementsV1Schema` com `maxAmountRequired` e `PaymentRequirementsV2Schema` com `amount`, porque o pacote fala os dois dialetos de propósito. Então o nome do campo é ele mesmo um sinal de versão: se você está olhando para `maxAmountRequired`, você está olhando para termos de v1, e todo o resto daquele desafio, inclusive o fato de ele ter chegado num corpo de resposta e não num header, decorre disso.

De onde vem o fee payer? Não de você. O resource server sincroniza com o facilitador na subida e aprende, por scheme e por network, o endereço de patrocinador com o qual o facilitador vai assinar. É por isso que a sequência de boot tem um passo `initialize()` explícito, e por isso que o seu servidor de lojista nunca segura uma chave de fee payer:

```ts
// x402/src/gateway.ts - one facilitator client, one resource server, boot-time work
import { x402ResourceServer } from '@x402/express';
import { HTTPFacilitatorClient } from '@x402/core/server';
import { registerExactSvmScheme } from '@x402/svm/exact/server';

const FACILITATOR_URL = 'https://x402.org/facilitator'; // devnet/testnet ONLY

const facilitator = new HTTPFacilitatorClient({ url: FACILITATOR_URL });
export const resourceServer = new x402ResourceServer(facilitator);
registerExactSvmScheme(resourceServer);
// the server calls resourceServer.initialize() once at boot
```

Essa URL merece uma frase própria em negrito nas suas notas de deploy. O facilitador x402.org é o deployment de referência e serve só devnet e testnet. Ele é perfeito para este lab e para CI, e é uma cilada se sobreviver até uma config de produção, porque ele não vai liquidar um pagamento de mainnet para você, nunca.

Este é também o momento em que o Challenge da lição passada é descontado. Você rascunhou um registro de decisão de cinco linhas sobre facilitador: uma restrição dominante, uma escolha primária, uma alternativa de compliance, uma configuração de CI e uma aceitação de confiança nomeada. Abra ele. A linha de CI daquele registro é o que o `FACILITATOR_URL` implementa hoje, e se o seu registro diz qualquer outra coisa que não o facilitador x402.org para CI, este lab é a sua chance de discutir com o seu eu do passado. A linha de escolha primária, Corbits, Dexter, PayAI ou Solvador, ou o CDP da Coinbase se liquidação com triagem for a sua restrição, é a troca de uma string que você faz ao entrar no ar. Entrar no ar é uma troca de URL mais a decisão de confiança que essa URL representa, e a metade da confiança é a metade difícil, que é por que você escreveu isso antes de ter código para se apegar. Mais uma cláusula pertence a essa decisão: a cautela de versão da lição passada continua valendo na troca, então antes de entrar no ar, confirme com o facilitador escolhido que ele liquida v2 na mainnet hoje, ou que ele negocia v1 para você, porque o mundo implantado ainda está com um pé em cada versão.

![Diagrama de sequência de uma chamada paga, do 402 carregando um memo de fatura, passando pela liquidação no facilitador, até a escrita no livro-razão que precede o recibo 200.](assets/v02-flowchart.png)

### Um memo por chamada, ou a conciliação desaba

Agora a parte que parece um detalhe e é de fato o problema de design da lição. O objeto de rotas acima é estático: construa ele uma vez, e todo 402 carrega o mesmo memo. Suba isso e a sua conciliação já nasce morta, porque três chamadas pagas caem no livro-razão como três pagamentos contra um id de fatura, e a sua fila de fulfillment reporta uma venda. Reusar um memo entre chamadas desaba a conciliação. O memo só é um id de fatura se for único por chamada.

Então de onde pode vir um id único por chamada? Pense no que o middleware de fato enxerga. O desafio 402 e o retry pago são duas requisições HTTP separadas, possivelmente com segundos de distância. Quando o retry chega, o middleware reconstrói os termos de pagamento a partir da config da rota e checa se os termos contra os quais o agente pagou batem com os termos que ele cotaria agora; só os campos que o scheme declara dinâmicos, as dicas de blockhash, ficam de fora dessa comparação. Um memo cunhado aleatoriamente no servidor na hora do desafio falha neste teste: o retry cunharia um diferente, a comparação erraria, e o agente encararia um loop infinito de 402s enquanto a carteira dele não drena nada. O id de fatura tem que ser algo que as duas requisições compartilham. E a única coisa que as duas requisições compartilham, byte a byte, é a URL.

Esse é o padrão: o agente cunha o id de fatura e carrega ele na query string, e o servidor dobra ele na config da rota daquela requisição. Determinístico nas duas pernas, único por chamada porque o agente faz assim:

```ts
// x402/src/routes.ts - the price sheet, one invoice id at a time
import type { RoutesConfig } from '@x402/core/server';

const SOLANA_DEVNET = 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1';
const MERCHANT = process.env.MERCHANT_ADDRESS ?? '';

// The price sheet for ONE invoice id. Rebuilt per request: every call carries
// its own extra.memo, and the 402 challenge and the paid retry agree on it
// because the invoice id rides the query string of both.
// The three TODO(config) marks are your completion rung: reason each value out
// from the rules above before reading the ones printed here, then compare.
export function routesFor(invoiceId: string): RoutesConfig {
  return {
    'GET /price': {
      accepts: {
        scheme: 'exact',
        network: SOLANA_DEVNET,     // TODO(config): the CAIP-2 network id
        payTo: MERCHANT,
        price: '$0.05',             // TODO(config): the per-call price
        extra: { memo: invoiceId }, // TODO(config): the per-call invoice id
      },
      description: 'Wavelength pressing-price quote',
    },
    'GET /price/rush': {
      accepts: {
        scheme: 'exact',
        network: SOLANA_DEVNET,
        payTo: MERCHANT,
        price: '$1.50', // deliberately over the client's default cap; see the guard section
        extra: { memo: invoiceId },
      },
      description: 'Rush quote: a human calls the pressing plant',
    },
  };
}
```

Antes de você objetar que um id de fatura cunhado pelo comprador só pode ser um buraco de segurança, jogue o ataque até o fim. O agente controla a string, então o pior que ele consegue fazer é reusar uma, e aí ele desabou os recibos dele, não os seus: o seu livro-razão registra todo pagamento liquidado sob o memo que ele carregou, cada um com a signature de transação dele, e um comprador que paga três vezes sob um id te pagou três vezes de todo jeito. O memo é uma chave de correlação, não uma autorização. A autorização mora na verificação que o facilitador faz de valor, ativo, destinatário e memo contra termos que o seu servidor produziu. Se a sua lógica de fulfillment algum dia tratar um memo como prova de qualquer coisa sozinho, você reconstruiu exatamente o bug de confiar em webhook que o módulo 4 passou uma lição inteira arrancando de você.

Você já viu essa ideia de conciliação antes com outro nome. Reference keys do Solana Pay e ids de fatura no memo do x402 são a mesma ideia: um marcador por pagamento que pega carona na transação para o lojista conseguir casar dinheiro com pedidos sem emitir um endereço de depósito único por venda. O módulo 3 carimbou a reference key nas suas transações de checkout; o x402 padroniza onde o marcador pega carona nos pagamentos entre máquinas. Dois trilhos, um padrão de conciliação.

![Diagrama contrastando um memo derivado da query string compartilhada, que sobrevive ao retry, contra um memo aleatório cunhado no servidor e um estático reusado.](assets/v03-diagram.png)

### Mesmo livro-razão, cliente novo

A conciliação é um hook só. O resource server dispara `onAfterSettle` depois que o facilitador confirma a liquidação, e o contexto entregue a esse hook carrega tudo que o seu livro-razão do módulo 4 precisa: os requirements que foram pagos (incluindo o memo e o valor) e o resultado do settle (incluindo a signature da transação on-chain). Então a venda de máquina cai exatamente onde a venda de humano cai:

```ts
// x402/src/reconcile.ts - the machine-sale path into the module 4 ledger
import type { x402ResourceServer } from '@x402/express';
import type { Ledger } from '../../backoffice/src/ledger.ts'; // the module 4 artifact, unchanged

export function wireReconciliation(resourceServer: x402ResourceServer, ledger: Ledger): void {
  resourceServer.onAfterSettle(async (ctx) => {
    if (!ctx.result.success) return;
    const memo = ctx.requirements.extra['memo'];
    if (typeof memo !== 'string' || memo.length === 0) return;
    ledger.record(
      {
        orderId: memo,                    // the invoice id IS the order id
        recipient: ctx.requirements.payTo,
        recipientAta: '',                 // derivable from (payTo, mint); record() never reads it
        mint: ctx.requirements.asset,
        amountBaseUnits: BigInt(ctx.requirements.amount),
      },
      ctx.result.transaction,             // the settled signature, same column as every webhook sale
    );
    console.log(`reconciled ${memo} -> ${ctx.result.transaction}`);
  });
}
```

Leia o formato que está sendo passado e repare que é o `ExpectedOrder` que você congelou no módulo 4, montado a partir do vocabulário do x402: memo vira `orderId`, `payTo` vira `recipient`, o mint do ativo e o valor em unidades base mapeiam direto. `recipientAta` fica vazio aqui porque o recibo de settle não carrega ele e `record()` nunca lê ele; dá para derivar de dono mais mint sempre que a ferramenta de ops quiser, exatamente como a lição do módulo 4 anotou quando adicionou o campo. Forçar o mapeamento pela mesma chamada de `record()` se paga no mês que vem, quando você conciliar uma semana de vendas: o checkout por QR do módulo 3, a transferência confirmada por webhook do módulo 4 e o bot que pagou por x402 hoje de manhã são linhas de um arquivo só com um formato só, e toda ferramenta de auditoria que você escrever funciona nas três.

Vou confessar onde a minha própria primeira versão deste hook deu errado: eu registrava a partir de `ctx.paymentPayload`, a coisa que o cliente mandou, em vez de `ctx.requirements`, a coisa que o servidor exigiu e o facilitador verificou. Mesmos dados no caminho feliz, direção de confiança errada. O hábito do módulo 4 transfere literalmente: o fulfillment registra o que foi verificado, nunca o que foi alegado.

![Dois caminhos de venda, uma reference key de checkout humano e um id de fatura no memo x402 de máquina, convergindo para um único formato de linha do livro-razão do back office.](assets/v04-diagram.png)

### O agente, o loop dele e o limite dele

Passe para o outro lado do balcão. O agente pagador precisa de três coisas: uma identidade que consegue assinar, um loop que responde a 402s, e um limite que impede ele de pagar qualquer coisa que ponham na frente dele.

Identidade primeiro, e repare que ela é deliberadamente diferente do tratamento de chave do módulo 2. Lá você carregou o arquivo `solana-keygen` existente do lojista, todos os 64 bytes dele, com `createKeyPairSignerFromBytes`. O agente é o comprador, não o lojista, então ele precisa de uma identidade própria, e não tem arquivo de keygen para carregar. O construtor irmão do kit cobre esse caso: `createKeyPairSignerFromPrivateKeyBytes` recebe uma seed de chave privada de 32 bytes em vez de um arquivo de keypair de 64 bytes. Gere a seed uma vez, persista ela, e recarregue ela numa chave Web Crypto não extraível a cada execução. Mesmo tipo de signer no fim, embrulhado para o x402:

```ts
// x402/src/signer.ts - a persistent agent identity
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { webcrypto } from 'node:crypto';
import { createKeyPairSignerFromPrivateKeyBytes } from '@solana/kit';
import { toClientSvmSigner, type ClientSvmSigner } from '@x402/svm';

const SEED_FILE = 'agent-seed.bin';

export async function loadSigner(): Promise<ClientSvmSigner> {
  if (!existsSync(SEED_FILE)) {
    const seed = new Uint8Array(32);
    webcrypto.getRandomValues(seed);
    writeFileSync(SEED_FILE, seed);
  }
  const seed = new Uint8Array(readFileSync(SEED_FILE));
  return toClientSvmSigner(await createKeyPairSignerFromPrivateKeyBytes(seed));
}
```

Repare no que este agente não precisa: SOL. O facilitador é o fee payer, então a carteira do agente segura USDC de devnet e mais nada. Essa assimetria é o design do exact-SVM fazendo o trabalho dele, clientes máquina não gerenciam gas.

O objeto de cliente registra o scheme SVM contra o id de network curinga, e o limite mora nesse cliente. Nenhuma chamada de `setSpendControls` aparece em lugar nenhum do código do agente, o que é uma decisão, não uma omissão: os padrões estão valendo, e os padrões têm dentes. Enuncie eles com precisão, porque são o fato congelado sobre o qual esta seção gira: de fábrica, os spendControls do cliente reconhecem só ativos atrelados ao dólar da tabela padrão do scheme, com teto de um dólar americano por pagamento, a menos que você sobrescreva. USDC de devnet a cinco centavos passa liso. A cotação expressa a um dólar e cinquenta não passa, e o jeito como ela não passa é a cilada: a checagem roda dentro da criação do pagamento, antes de qualquer coisa ser assinada, e ela lança. Se o código do seu agente não captura e loga esse throw, o agente simplesmente nunca paga, a sua fila de fulfillment fica vazia, e nada em lugar nenhum diz por quê. Um teto que você não consegue ver recusando é indistinguível de uma integração quebrada. Quando uma chamada vale legitimamente mais que um dólar, levante o teto de propósito com `setSpendControls({ maxAmountPerPayment: '$2' })` e escreva por quê; quando não vale, mantenha o padrão e faça a recusa gritar.

O loop em si é o seu degrau Completion, o TODO no meio do arquivo do agente. `wrapFetchWithPayment(fetch, client)` do `@x402/fetch` faz a dança inteira numa linha, e você vai usar isso em produção (ele está deliberadamente fora da linha de instalação deste workspace; adicione o pacote quando for pegar nele). Hoje você escreve a dança à mão uma vez, porque o engenheiro que construiu o loop consegue debugar o wrapper, e o engenheiro que só usou o wrapper não consegue:

```ts
// x402/src/agent.ts - the collector bot, taught to pay
import { x402Client } from '@x402/core/client';
import {
  decodePaymentRequiredHeader,
  decodePaymentResponseHeader,
  encodePaymentSignatureHeader,
} from '@x402/core/http';
import type { PaymentRequired } from '@x402/core/types';
import { ExactSvmScheme } from '@x402/svm/exact/client';
import { loadSigner } from './signer.ts';

const API = process.env.API_URL ?? 'http://localhost:4021';

const signer = await loadSigner();
console.log(`agent pays as ${signer.address}`);

const client = new x402Client().register('solana:*', new ExactSvmScheme(signer));
// No setSpendControls call: the DEFAULTS are in force. Pegged assets, $1 per payment.

// Your turn (completion rung). The four rules:
// 1. fetch(url); anything but HTTP 402 returns as-is, already paid or free.
// 2. Read the challenge out of the HEADER, never the body:
//    decodePaymentRequiredHeader(res.headers.get('PAYMENT-REQUIRED'))
//    returns the PaymentRequired object. The 402's body is the two bytes '{}'.
// 3. const payload = await client.createPaymentPayload(paymentRequired);
//    spendControls run INSIDE this call: an over-cap quote throws here,
//    before anything is signed. Let the throw escape to the caller.
// 4. Retry the SAME url with the header:
//    { 'PAYMENT-SIGNATURE': encodePaymentSignatureHeader(payload) }
async function payAndRetry(url: string, client: x402Client): Promise<Response> {
  throw new Error('Your turn: implement the four rules above.');
}

// The driver is worked: three paid calls, then one deliberate refusal.
for (let call = 1; call <= 3; call++) {
  const invoiceId = `WVL-INV-${Date.now()}-${call}`;
  const res = await payAndRetry(`${API}/price?run=500&invoice=${invoiceId}`, client);
  if (!res.ok) {
    // Failures are silent in the body and loud in the headers; the checkpoint
    // section at the end of this lesson reads them field by field.
    console.error(`call ${call} failed: HTTP ${res.status}`);
    continue;
  }
  const receiptHeader = res.headers.get('PAYMENT-RESPONSE');
  console.log(`call ${call}: paid ${invoiceId}`, await res.json());
  if (receiptHeader) {
    console.log(`  settled: ${decodePaymentResponseHeader(receiptHeader).transaction}`);
  }
}

// The over-cap call: $1.50 against the $1 default. Decline LOUDLY or not at all.
try {
  await payAndRetry(`${API}/price/rush?run=500&invoice=WVL-INV-RUSH-1`, client);
} catch (error) {
  const reason = error instanceof Error ? error.message : String(error);
  console.log(`rush call declined by spendControls: ${reason}`);
}
```

Segure o caminho de decisão inteiro numa imagem antes do lab, porque a posição da cancela dos spendControls, dentro da criação do pagamento e antes de qualquer signature, é o fato ao qual a seção de debugging vai te mandar de volta o tempo todo.

![Fluxograma do agente lidando com um 402, onde a checagem de spendControls dentro da criação do pagamento ou libera a chamada para ser assinada ou lança antes de qualquer signature.](assets/v05-flowchart.png)

### Verifique o que foi de fato assinado

Uma história de dentro do SDK antes do lab, porque ela vai recalibrar para sempre o jeito como você pensa sobre verificação: carteiras quebram verificação ingênua.

Suponha que você decidiu conferir os pagamentos você mesmo, do lado do servidor. O design óbvio: reconstruir a transação que você espera, a transferência, o memo, o compute budget, e comparar ela byte a byte com o que o agente submeteu. Rigoroso, né? Faça o deploy disso, e pagamentos legítimos começam a falhar. Não por maldade: por recursos de segurança. Phantom e Solflare injetam instruções de guarda do Lighthouse nas transações que assinam. Lighthouse é um programa de asserção; as carteiras acrescentam instruções para ele de modo que, se os efeitos da transação divergirem do que foi simulado para o usuário, a execução falha on-chain. Um airbag de proteção, e isso quer dizer que a transação assinada é um superconjunto da transação que você construiu.

O verificador SVM do x402 carrega a cicatriz no código dele: `mechanisms/svm/src/constants.ts` deixa hardcoded uma allowlist para o programa de guarda Lighthouse, `L2TExMFKdjpN9kozasaurPirfHy9P8sbXoAN1qA3S95`, adicionada depois de a issue #828 documentar exatamente essa falha. O verificador percorre a transação que foi de fato assinada, checa a transferência, o memo e o isolamento do fee payer, e tolera instruções daquele único programa de guarda na allowlist enquanto rejeita qualquer outra adição.

O princípio é maior que o incidente, então grave isso: verifique o que foi de fato assinado, nunca a transação idealizada que você teria construído. O seu verificador do módulo 4 já vive por essa regra sem você ter nomeado ela, ele lê a transação da blockchain e checa propriedades, dono, mint, delta, memo, em vez de exigir igualdade byte a byte com um template. Checagens de propriedade toleram adições benignas; comparações de bytes declaram guerra a todo recurso de segurança de carteira já lançado. Quando você escreve código de verificação em qualquer lugar da sua stack, você está escolhendo entre essas duas posturas, e este incidente é o argumento pela primeira.

![Comparação entre o casamento ingênuo byte a byte e a verificação por propriedades que o x402 faz da transação assinada, com a allowlist dele para instruções de guarda Lighthouse injetadas pela carteira.](assets/v06-comparison.png)

O mesmo pacote esconde uma segunda guarda que vale conhecer porque você construiu a prima dela no módulo 4. O lado do facilitador mantém um cache de liquidação: uma tabela em memória das transações que estão sendo liquidadas naquele momento, para que uma chamada /settle duplicada do mesmo pagamento seja rejeitada como `duplicate_settlement` em vez de correr contra a primeira submissão. As entradas são despejadas por um timer que o pacote amarra ao tempo de vida do blockhash, a documentação dele chama essa janela de mais ou menos 60 a 90 segundos e despeja em 120, cerca do dobro do tempo de vida, com o raciocínio de que, uma vez que o blockhash de um pagamento não consegue mais aterrissar, um settle reenviado dele não consegue mais dar certo, então lembrar dele é inútil. Se essa frase te deu déjà-vu, deveria mesmo: é a mesma aritmética de despejo do seu store de signatures processadas do módulo 4, que esquece uma signature assim que a transação dela não poderia de jeito nenhum ser confundida com uma nova. O seu store protege o fulfillment contra webhooks reenviados; o cache de liquidação protege a submissão contra settles reenviados. Mesmo formato, porta diferente. E o hábito do módulo 4 de derivar o relógio a partir do tempo de slot atual vale para os dois: no tempo de slot alvo de 300ms — engatilhado pelo SIMD-0525 na hora em que isto é escrito, com a epoch 1024 (2026-08-28) marcada para travar ele dias depois — a janela de 150 blocos dá uns 45 segundos, bem abaixo dos 60 redondos que as pessoas citam, e os cortes restantes engatilhados do SIMD-0525 vão encolher ela ainda mais, que é exatamente por que a margem do cache é generosa, e por que este curso não para de dizer derive, nunca decore.

![Linha do tempo de um pagamento, do partial signing até a expiração do blockhash e o despejo do cache de liquidação, posta ao lado do store de signatures processadas do módulo 4 no mesmo horizonte.](assets/v07-timeline.png)

### O cobrador de pedágio que você aluga

Nomeie a troca antes de subir isso, porque esta aqui é estrutural — e é a mesma troca que a lição passada já nomeou: o facilitador é fee payer e fronteira de confiança da liquidação numa coisa só, ele vê toda transação, ele pode recusar, e recusa em escala é censura seja lá como os termos de serviço chamem isso; os hospedados fazem triagem KYT por design, a mesma moeda virada para o lado do compliance. Nada disso é um bug que alguém vai corrigir; um patrocinador é uma contraparte. O que esta lição acrescenta é a disciplina de livro-razão: você fez exatamente esta troca com os ramps fiat no módulo 6 e escreveu isso num registro de decisão, a coluna do facilitador pertence à mesma tabela, e o facilitador de devnet x402.org especificamente pertence à linha de nunca-em-produção dessa tabela.

O segundo limite honesto é econômico, e é a versão comércio-de-máquinas de uma lição que toda pessoa de pagamentos acaba aprendendo: o custo de liquidação tem que caber dentro da coisa que está sendo vendida. exact liquida toda chamada on-chain, então toda chamada carrega custo real de blockchain, a taxa de transferência que o patrocinador engole mais o custo operacional das idas e voltas de verify e settle. A cinco centavos a cotação, esse overhead é erro de arredondamento e medição por chamada é exatamente o certo. A mil pings de telemetria de menos de um centavo por minuto, liquidação por chamada custa mais que o produto, e nenhuma quantidade de entusiasmo de engenharia muda a aritmética; esse tráfego quer o modelo de autorizar-um-teto do scheme upto ou liquidação em lote, os dois existindo exatamente porque exact não estica até lá. Case o scheme com as unit economics da chamada, e desconfie de qualquer plano de medição cuja margem dependa de o trilho de liquidação ser grátis. Aqui está a metade otimista, e é a metade que importa para a Wavelength: o bot de colecionador que hoje de manhã era puro centro de custo agora é um cliente com unit economics que fecham, num patamar de preço que nenhuma bandeira conseguiria compensar com lucro. Clientes máquina não são uma ameaça à tabela de preços: eles são o primeiro segmento de clientes da história que lê ela perfeitamente e nunca abandona um carrinho.

E o terceiro limite com o qual você conviveu a lição inteira: a linha `@x402/*` está fixada em 2.23.0 aqui (publicada em 2026-08-18), e já se mexeu duas vezes desde então — 2.24.0 em 2026-08-27, 2.25.0 em 2026-09-04, conferido em 2026-09-07 — o que é o ponto, e não uma vergonha: nada neste ecossistema sugere que ele vai ficar parado. Todo fato de wire desta lição foi lido daquele build exato e não de um documento: o transporte por header, `amount` em vez de `maxAmountRequired` no requirement v2, e o `maxTimeoutSeconds` que o resource server preenche para você. Esse pé em cada versão é o que torna essa disciplina não opcional, porque um pacote embarca os schemas dos dois dialetos lado a lado, então "que formato eu estou segurando" continua sendo uma pergunta viva a cada bump em vez de uma pergunta resolvida. Reverifique a cada toque, do jeito que esta lição fez, não do jeito que um bookmark faz.

![Cartão de três colunas da troca da medição: a fronteira de confiança do facilitador, a economia da liquidação por chamada, e o pin do pacote que se mexe rápido e precisa ser reverificado.](assets/v08-comparison.png)

## Lab: ponha a cancela, pague, concilie

O critério desta lição: uma chamada não paga responde 402, o seu agente liquida três chamadas pagas na devnet, três ids de fatura distintos se conciliam no livro-razão do módulo 4, e a chamada expressa acima do teto é recusada com um motivo logado. O encanamento está percorrido acima; você preenche a tabela de preços e o loop.

1. **Financie o agente.** Salve `src/signer.ts` e `src/agent.ts` da seção de teoria, depois rode o agente uma vez. Ele cunha a seed dele, imprime o endereço dele, e quebra na hora no loop não implementado, o que está certo: hoje a quebra é o seu marcador de TODO. Mande para o endereço impresso alguns dólares de USDC de devnet pelo faucet da Circle em faucet.circle.com (escolha Solana Devnet), o mesmo faucet que você usou no módulo 2. Nenhum airdrop de SOL é necessário: o facilitador paga as taxas, o que agora você consegue explicar em vez de só aproveitar.

```bash
cd ~/wavelength/x402
npx tsx src/agent.ts   # prints: agent pays as <address>, then throws 'Your turn'; fund that address
```

2. **Monte o servidor.** Três dos quatro arquivos dele vieram da seção de teoria: `src/gateway.ts` (cliente do facilitador mais resource server), `src/routes.ts` (a sua tabela de preços completada, os três campos TODO(config)) e `src/reconcile.ts` (o hook do livro-razão). O quarto é a ligação do Express abaixo, resolvida porque as duas decisões dela são sutis e nenhuma das duas é a lição:

```ts
// x402/src/server.ts - Wavelength's pressing-price API, gate in front
import express from 'express';
import { paymentMiddleware } from '@x402/express';
import { Ledger } from '../../backoffice/src/ledger.ts';
import { resourceServer } from './gateway.ts';
import { routesFor } from './routes.ts';
import { wireReconciliation } from './reconcile.ts';

const ledger = new Ledger(process.env.LEDGER_FILE ?? 'orders.jsonl');
wireReconciliation(resourceServer, ledger);

const app = express();

app.use((req, res, next) => {
  const invoiceId = typeof req.query.invoice === 'string' ? req.query.invoice : '';
  if (!invoiceId) {
    res.status(400).json({ error: 'invoice query param required, e.g. ?invoice=WVL-INV-0001' });
    return;
  }
  // Fresh middleware per request so the routes carry THIS call's memo.
  // syncFacilitatorOnStart=false: the shared resourceServer synced once at boot.
  void paymentMiddleware(routesFor(invoiceId), resourceServer, undefined, undefined, false)(
    req,
    res,
    next,
  );
});

function quote(runSize: number): { runSize: number; unitPriceUsd: number; totalUsd: number } {
  const unit = runSize >= 500 ? 6.1 : 7.4;
  return { runSize, unitPriceUsd: unit, totalUsd: Math.round(unit * runSize * 100) / 100 };
}

app.get('/price', (req, res) => {
  res.json(quote(Number(req.query.run ?? 100)));
});
app.get('/price/rush', (req, res) => {
  res.json({ ...quote(Number(req.query.run ?? 100)), rush: true });
});

await resourceServer.initialize(); // learn supported kinds + the fee payer from the facilitator
app.listen(4021, () => console.log('pressing-price API on :4021, gate armed'));
```

As duas decisões, para você ser dono delas: o middleware é construído por requisição puramente para o objeto de rotas conseguir carregar o memo de fatura desta chamada, com a sincronização cara com o facilitador feita uma vez no boot e desligada por requisição por meio daquele `false` final. E os handlers de negócio seguem completamente ignorantes de pagamentos; a função de cotação rodaria igualzinha com o middleware deletado, que é a propriedade que deixa a próxima lição acrescentar um segundo protocolo de pagamento sem tocar nela.

3. **Suba o servidor** com o seu endereço de lojista do módulo 2:

```bash
MERCHANT_ADDRESS=$(solana address) LEDGER_FILE=../backoffice/orders.jsonl npx tsx src/server.ts
```

Esperado: `pressing-price API on :4021, gate armed`. Se der throw em `initialize()`, o facilitador está inalcançável ou não suporta o par scheme/network; confira a URL e a sua rede antes de suspeitar do seu código.

4. **Prove a cancela a partir de um segundo terminal.** Sem header de pagamento, sem cotação:

```bash
curl -i "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST"
```

Esperado, e este é o formato que a lição passada prometeu (as linhas Date, ETag e keep-alive foram cortadas, e o base64 foi elidido nas reticências):

```text
HTTP/1.1 402 Payment Required
X-Powered-By: Express
Content-Type: application/json; charset=utf-8
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJQYXltZW50IHJlcXVpcmVkIiwicmVzb3VyY2Ui...
Cache-Control: no-store
Content-Length: 2

{}
```

A cancela está no ar, exatamente como anunciado. Decodifique o header:

```bash
curl -sD - -o /dev/null "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST" \
  | grep -i '^payment-required:' | sed 's/^[^:]*: *//' | tr -d '\r' \
  | base64 -d | node -p "JSON.stringify(JSON.parse(require('fs').readFileSync(0,'utf8')),null,2)"
```

```json
{
  "x402Version": 2,
  "error": "Payment required",
  "resource": {
    "url": "http://localhost:4021/price?run=500&invoice=WVL-INV-TEST",
    "description": "Wavelength pressing-price quote",
    "mimeType": ""
  },
  "accepts": [
    {
      "scheme": "exact",
      "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
      "amount": "50000",
      "asset": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
      "payTo": "<the MERCHANT_ADDRESS your server was started with>",
      "maxTimeoutSeconds": 300,
      "extra": {
        "memo": "WVL-INV-TEST",
        "feePayer": "CKPKJWNdJEqa81x7CkZ14BVPiY6y16Sxs7owznqtWYp5"
      }
    }
  ]
}
```

Leia isso contra o cartão de mapeamento da seção de teoria: o seu `'$0.05'` virou `"50000"` unidades base do mint de USDC de devnet, o seu memo passou byte a byte, e `resource.description` é a `description` que você escreveu na rota. Dois campos chegaram sem você nunca ter configurado. `extra.feePayer` veio da sincronização com o facilitador. E `maxTimeoutSeconds` é `300` porque a config da sua rota deixou ele de fora e o `@x402/core` preenche 300 segundos quando isso acontece; ponha `maxTimeoutSeconds: 90` ao lado de `price` no `accepts` daquela rota e o desafio diz `90` em vez disso. O `mimeType` está vazio pelo mesmo motivo ao contrário: nada declarou um, então nada foi inventado. Você consegue ver a mesma resposta sem rodar nada: `curl -s https://x402.org/facilitator/supported` lista os kinds que aquele facilitador liquida, e em 2026-08-22 a entrada `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1` dele anunciava o scheme `exact` com `extra.feePayer` valendo `CKPKJWNdJEqa81x7CkZ14BVPiY6y16Sxs7owznqtWYp5` (os patrocinadores rotacionam, então case o formato, não a string). Toda network daquela lista é uma testnet, que é o aviso de só-devnet reenunciado pelo próprio facilitador. Achar esse fee payer nos termos é a sua evidência de que a sequência de boot funcionou; um `feePayer` faltando quer dizer que `initialize()` nunca rodou ou que o facilitador não suporta o seu par de scheme e network.

5. **Complete o `payAndRetry`** em `src/agent.ts` contra as quatro regras da seção de teoria, depois rode o agente:

```bash
npx tsx src/agent.ts
```

Saída esperada, formatos e não strings exatas: três linhas `call N: paid WVL-INV-...` cada uma seguida de uma signature de transação liquidada, depois `rush call declined by spendControls:` com o teto nomeado no motivo.

As três signatures são transações de devnet de verdade, então gaste um minuto lendo uma delas em qualquer explorer, porque a lição inteira está sentada dentro dela. A instrução de transferência move `50000` unidades base de USDC de devnet da ATA do agente para a sua. A instrução de memo carrega a sua string de fatura, a história da conciliação visível num livro-razão público. As instruções de compute budget estão lá porque a spec exige elas numa transação de settle. E o fee payer da transação não é nem você nem o agente: é o endereço de patrocinador do facilitador vindo de `extra.feePayer`, a conta cuja signature foi a última a ser adicionada. Duas signatures, dois momentos de assinar, um deles aconteceu na sua máquina e o outro não. Isso é partial signing, não mais no papel.

6. **Audite o livro-razão.** Três linhas novas, três ids de fatura distintos, ao lado de quaisquer vendas do módulo 4 que o arquivo já segure:

```bash
tail -n 3 ../backoffice/orders.jsonl
```

Cada linha: o memo como `orderId`, a signature liquidada, `"50000"` unidades base, o mint de USDC de devnet. Um arquivo, um formato, dois tipos de cliente. Esse é o critério desta lição, checado na mão a partir do seu próprio terminal: cancela no ar (chamada não paga devolve 402), pelo menos uma chamada liquidada na devnet, e o memo dela conciliado no livro-razão.

## Challenge: a guarda pre-flight do decidePayment

Degrau Solo, sem walkthrough. O throw dos spendControls é a guarda do SDK; um agente de produção quer a decisão pre-flight dele mesmo, com motivos que os logs dele conseguem agregar, tomada antes de o SDK ser sequer consultado. Implemente `decidePayment` no widget de coding challenge decide-payment, que te entrega o starter e os testes dele. O widget chama ela posicionalmente, um argumento por campo: `decidePayment(scheme, network, asset, amount, feePayer, memo, maxUsd, allowedAssetsJson)`. Os seis primeiros são os termos de pagamento que um 402 cotou, desempacotados do objeto de requirements que você decodificou do header PAYMENT-REQUIRED; os dois últimos são os controles do próprio agente, e a allowlist chega serializada como uma string JSON mapeando mint para decimais, então `JSON.parse(allowedAssetsJson)` antes de checar qualquer coisa contra ela. Devolva um objeto de decisão que ou passa adiante o fee payer e o memo para pagamento ou recusa com um motivo preciso. O valor chega como `amount` porque estes são termos v2; uma guarda escrita contra uma contraparte v1 estaria lendo `maxAmountRequired` para o mesmo número, que é a fronteira de versão da seção de teoria aparecendo na sua primeira linha de código.

Rejeite com um motivo distinto para cada um, e rode as checagens nesta ordem fixa, para que os mesmos termos ruins sempre produzam o mesmo motivo, que é o que torna as recusas agregáveis:

```text
1. scheme     not "exact"                                    -> unsupported scheme
2. network    not a known Solana CAIP-2 id (mainnet/devnet)  -> unknown network
3. asset      absent from the agent's pegged allowlist       -> asset not allowed
4. memo       over 256 UTF-8 BYTES (not string length)       -> memo too large
5. fee payer  no feePayer named                              -> missing fee payer
6. amount     USD-converted value above the cap              -> exceeds spend cap
```

Seis linhas, e um campo do desafio deliberadamente não está entre elas. Todo requirement que você decodificou carrega `maxTimeoutSeconds`, `300` nos termos que este lab cota, e a guarda nunca olha para ele. As seis checagens todas respondem a uma pergunta, eu vou pagar estes termos, e cada uma é uma política sobre a qual o agente tem opinião. `maxTimeoutSeconds` não é uma política: é o prazo do lojista para completar o pagamento, definido na rota do lojista e entregue a você como um fato. Não há nada ali para o agente aprovar ou recusar, e um agente que paga na hora, como este paga, não consegue perder uma janela de cinco minutos de todo jeito. A versão desta guarda que checaria isso pertence a um agente que enfileira 402s e paga eles depois: compare a janela com a latência de pior caso da sua fila e descarte qualquer coisa que não dê tempo. Repare no que pular essa checagem custa a um agente que enfileira, porém, porque não é uma recusa. Os termos simplesmente param de ser pagáveis, a sua guarda não diz nada, e o que você aprender sobre isso, você aprende do `error` na resposta e não dos seus próprios logs.

A régua de aceitação, batendo com o critério da lição: uma chamada dentro da política devolve `willPay: true` com o fee payer e o memo passados adiante; uma chamada acima do teto recusa com um motivo de teto; uma chamada com ativo errado, um scheme não-exact e um memo grande demais recusam cada um com o motivo próprio; e o id CAIP-2 da devnet é aceito como conhecido. Os testes do widget rodam tudo; verde quer dizer pronto.

Se você terminar cedo, ligue isso de verdade: chame a sua guarda no topo do `payAndRetry` e compare os vereditos dela com os throws do SDK nas quatro chamadas do lab. Eles deveriam concordar em todas, e agora você tem duas opiniões independentes sobre todo pagamento que o seu agente faz, que é exatamente quanta paranoia um bot que segura carteira merece.

E um treino que cobra uma dívida do módulo 4, que vale dez minutos porque é o único lugar onde este curso consegue pagar ela honestamente. A sua lição de conciliação deixou `tryMatchByReferenceOrMemo` em `backoffice-refunds/src/sweep.ts` com a metade do memo em aberto, com a promessa de que o tráfego do módulo 7 ia precisar dela. Precisa mesmo — mas só no caminho infeliz, então faça um: rode uma chamada paga do agente com o seu hook de liquidação desligado (comente o `ledger.record` no `onAfterSettle`), para dinheiro de verdade aterrissar on-chain carregando o id de fatura em `extra.memo` e o seu livro-razão nunca ficar sabendo. Isso é um worker que caiu, reproduzido de propósito. Agora rode a varredura de tesouraria contra a ATA do seu lojista. Com a metade do memo preenchida, ela acha o crédito órfão, parseia o id de fatura de dentro do memo, e casa ele com o pedido aberto que o caminho da reference nunca teria achado, porque uma liquidação x402 não carrega reference key nenhuma. Aceite: uma linha recuperada, e a mesma varredura rodada duas vezes escreve ela uma vez só. Reative o hook depois.

## Checkpoint: três linhas e uma recusa

Onde isso costuma emperrar, na ordem em que você bateria:

1. **Um loop de 402 que nunca resolve** — o agente assina e repete para sempre enquanto nada liquida e nenhum USDC sai da carteira dele. O seu memo não é determinístico entre o desafio e o retry; releia a seção da query string, a URL é o único terreno compartilhado.
2. **Um agente que termina em silêncio com uma fila de fulfillment vazia** — você deixou o throw dos spendControls escapar sem logar, exatamente a cilada da recusa silenciosa contra a qual a seção de teoria avisou; capturar ele não é opcional em nada que você suba.
3. **Uma falha de verify numa chamada que os seus spendControls aprovaram numa boa** — não chute, e não leia o corpo, que é `{}` aqui como é em todo lugar. Decodifique o header PAYMENT-REQUIRED da resposta que falhou e leia o campo `error` dele, as palavras do próprio facilitador para o que deu errado. Uma ATA sem fundos aparece ali como `transaction_simulation_failed`, porque o verificador exact-SVM adiciona a signature do fee payer, simula o resultado, e vê a transferência falhar; essa string sozinha é a diferença entre checar o saldo de USDC de devnet do seu agente em dez segundos e suspeitar do seu próprio código por uma hora.
4. **Passou da verificação, morreu na liquidação** — o motivo pega carona no `PAYMENT-RESPONSE` como `errorReason`, e o corpo continua sendo `{}`. A guarda checa política, o facilitador checa realidade, e a realidade responde de volta por um header.
5. **Linhas caindo sob um único id de pedido** — você reusou um id de fatura, que o seu livro-razão vai registrar numa boa e a sua conciliação vai ler errado como uma venda só; o desabamento está depois da escrita, não nela.

Agora conte o que você tem na mão. Um padrão de API de produção onde o paywall é um middleware e a lógica de negócio nunca aprendeu que dinheiro existe. Um agente que paga por HTTP do jeito que os navegadores buscam, com um limite que ele impõe antes de assinar. Um caminho de conciliação onde vendas de máquina e vendas de humano são um livro-razão só, um formato de linha só, uma história de auditoria só, porque você roteou o memo do x402 pelo mesmo `record()` que uma venda por webhook percorre. E dois instintos de verificação afiados em incidentes de verdade: cheque propriedades do que foi assinado, nunca igualdade byte a byte com o que você construiu, e esqueça estado de replay só quando a própria blockchain torna o replay impossível.

O gancho para a frente já está sentado no seu código, nos handlers que nunca aprenderam que dinheiro existe. A mesma API está prestes a responder a um segundo protocolo de pagamento sem mudar uma linha de lógica de negócio: na próxima lição você põe a cancela da CLI pay na frente dela, para que bots que falam x402 e bots que falam MPP sejam atendidos por uma porta só. Você construiu a porta hoje. Na próxima lição ela aprende mais idiomas.
