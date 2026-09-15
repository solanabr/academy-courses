# Transaction requests: o servidor constrói a transação

Na lição passada, o checkout colocou um QR numa página e um watcher atrás dele: a reference key que você embutiu deixou o seu servidor casar a transferência que chegou na devnet e validar ela. Progresso de verdade. Mas repare em quem fez a construção. A carteira do cliente montou aquela transação a partir de uma URL fixa, o que quer dizer que um disco a um preço é tudo que o seu checkout consegue expressar.

Agora imagine a loja de verdade. Um cliente tem três discos no carrinho, um código de cupom da newsletter do mês passado e um id de pedido que o seu livro-razão precisa carimbado on-chain. Onde é que isso mora? Não numa URL. Uma URL é uma string que o cliente segura, e cada caractere dela é editável antes de a carteira dele fazer qualquer coisa com ela. Um total numa URL é uma sugestão. Um cupom numa URL é um convite.

A bala de prata? Pare de deixar a carteira construir a transação. Esta lição inverte o protocolo: a carteira te traz a conta do cliente, e o seu servidor devolve uma transação completamente construída, com o carrinho precificado, o memo carimbado e a reference injetada, tudo por código que você controla. Monte o workspace agora, para a instalação rodar enquanto você lê. Rode o `mkdir` a partir da raiz `wavelength`, para a nova pasta ficar ao lado de `transfer-kit` e os imports relativos abaixo resolverem:

```bash
mkdir -p checkout-txreq/src checkout-txreq/public
cd checkout-txreq
npm init -y
npm pkg set type=module
npm install @solana/kit@6.10.0 @solana-program/token@0.14.0 express@5
npm install -D tsx@4 typescript @types/express @types/node
```

Notas de pin, conferidas em 2026-08-22: `@solana/kit` 6.10.0 é o último release da v6, e este workspace fica na v6 porque `@solana/pay` 1.0.26 (publicado em 2026-07-31, ainda `latest` no npm) tem kit ^6.9 como peer. `@solana-program/token` 0.14.0 é o último minor compatível com a kit v6; a leva 0.15.x tem kit ^7 como peer, então quebraria este workspace. `express` 5 é o major atual. `tsx` é o runner que você usou o curso inteiro. Uma ausência deliberada: NÃO adicione `@solana/pay` a este workspace. A faixa de peers dele quer `@solana-program/token` ^0.12, que briga com o pin 0.14.0, e este servidor nunca codifica uma URL de todo jeito; a biblioteca pay fica lá no projeto checkout, onde a página mora.

## Resumo

O índice de achados, cada linha acionável:

- Uma transaction request inverte o esquema da URL: `solana:<https-link>` em vez de `solana:<recipient>`. Uma diferença de formato, um protocolo completamente diferente embaixo.
- A carteira fala dois verbos no seu link. GET devolve `{ label, icon }` para a carteira poder mostrar quem está pedindo dinheiro. POST `{ account }` devolve `{ transaction }`, uma transação codificada em base64 construída para aquele pagador específico.
- Você entrega o **checkout-txreq**: um endpoint Express em `/txreq` (porta 3100, http puro localmente; as carteiras vão querer https, e a lição da maquininha põe um proxy na frente) mais o `buildOrderTransaction`, o núcleo de precificação e montagem que toda superfície posterior deste curso reusa.
- A precificação vai inteira para o lado do servidor. A URL carrega só um id de pedido opaco; o total, a matemática do cupom, o memo e a reference key são calculados e injetados pelo seu código. Nunca confie num preço mandado pelo cliente.
- A troca é uma nova fronteira de confiança: a carteira agora assina uma transação que o seu servidor escreveu. A regra da spec policia isso: uma requisição que exige uma signature de uma conta que o usuário não submeteu é maliciosa e precisa ser rejeitada. Você mesmo vai implementar essa rejeição.
- Uma transação construída embute um blockhash, válido por 150 blocos. No tempo de slot alvo atual de 300ms isso dá uns 45 segundos de relógio, e o SIMD-0525 já tem mais dois cortes de tempo de slot engatilhados; o número deriva do tempo de slot, então derive ele, nunca decore.

A sua parte do trabalho hoje: os handlers de GET e POST vêm trabalhados, cada linha dada. O cálculo do total no servidor e a injeção do memo e da reference chegam como TODOs no apoio, e a seção de teoria te ensina exatamente o que vai dentro deles. O caminho do cupom e a guarda contra requisição maliciosa são só seus no final.

## Quem constrói a transação agora

### Um caractere de diferença, um protocolo invertido

Ponha as duas formas de URL lado a lado. Uma transfer request é `solana:<recipient>?amount=...&spl-token=...&reference=...`: o destinatário é a URL, os parâmetros são a transação, e quem monta é a carteira. Uma transaction request é `solana:<https-link>`: o payload é um link para o seu servidor, e o trabalho da carteira encolhe para buscar, exibir e assinar. Mesmo prefixo `solana:`. Se o que vem depois faz o parse como uma URL https, a carteira trata aquilo como uma transaction request; se faz o parse como um endereço, uma transfer request. As carteiras despacham por esse formato, e o seu modelo mental deveria despachar também, porque todo o resto deste módulo depende de que lado dessa bifurcação você está.

![As duas formas de URL do Solana Pay lado a lado: a transfer request construída pela carteira expondo cada parâmetro, contra a transaction request construída pelo servidor carregando só um link e um id de pedido.](assets/v01-comparison.png)

Então por que a lição do QR veio primeiro? Porque a transfer request é trustless de um jeito que esta lição abre mão de propósito. Não tinha servidor nenhum para comprometer: a carteira construía a transferência sozinha, a partir de parâmetros que o cliente conseguia inspecionar. Simplíssima, e exatamente tão limitada quanto parece. No momento em que você quer lógica de carrinho, você precisa de código rodando em algum lugar que o cliente não consegue editar, e código rodando em algum lugar quer dizer confiança nesse algum lugar. Guarde esse pensamento; ele vira a seção de segurança.

Se você carrega cicatrizes de Stripe, essa inversão vai parecer familiar. O fluxo só-cliente, um preço cravado num widget de frontend, é a coisa de que todo guia de integração da Stripe te afasta já na primeira página. O fluxo adulto é um PaymentIntent: o seu backend decide o valor, anexa os metadados e entrega ao cliente algo já precificado. Uma transaction request é esse mesmo formato nos trilhos da Solana. A lógica de precificação volta para o seu backend, onde ela sempre pertenceu, e a carteira vira a superfície de confirmação em vez da calculadora.

### A ida e volta: GET, depois POST

A carteira faz duas chamadas ao seu link, numa ordem fixa, e a ordem é o ponto.

Primeiro ela faz um GET no seu endpoint. A sua resposta é pequena: uma string `label` e uma URL `icon`. Isso não é decoração. A carteira está prestes a pedir para um humano aprovar um pagamento construído por um servidor desconhecido, e o GET dá a ela algo honesto para exibir antes de qualquer coisa financeira acontecer: para quem você está pagando, com um rosto. Honesto, com uma ressalva que vale dizer em voz alta: o label é autodeclarado. Qualquer servidor consegue responder "Wavelength Records", e é por isso que carteiras bem construídas exibem o seu domínio ao lado do seu label, e por isso que a identidade que um cliente consegue de fato verificar é a origem https, não a string que você escolheu. O seu trabalho é manter essas duas apontando para o mesmo negócio. Só depois de renderizar esse contexto é que a carteira faz um POST de `{ "account": "<base58 pubkey>" }`, a chave pública do cliente, para a mesma URL. O seu servidor agora sabe a única coisa que ele não podia saber de antemão, quem está pagando, e consegue construir a transação para exatamente aquele pagador: a conta dele como fee payer, a conta de token dele como a fonte dos fundos.

![Fluxograma de um carrinho guardado passando pelo QR, o GET da carteira atrás de label e icon, o POST da conta, a precificação no servidor, a carteira assinando e o watcher da reference confirmando.](assets/v02-flowchart.png)

Repare no que essa divisão te compra. O GET é cacheável, não autenticado, seguro de bater cem vezes. O POST é por cliente e devolve uma transação que só vale por pouco tempo, porque o seu servidor carimba nela um blockhash recente. Um blockhash não pode ter mais de 150 blocos quando a transação aterrissa; no tempo de slot alvo atual de 300ms isso dá uns 45 segundos. Derive essa janela do tempo de slot toda vez que você citar ela, porque o tempo de slot é a coisa que se mexe: eram uns 60 segundos nos slots antigos de 400ms e uns 53 na etapa de 350ms do SIMD-0525, a etapa de 300ms entrou em vigor no epoch 1024, e mais dois cortes já estão engatilhados no código. Na prática: construa no POST, nunca no GET, e nunca cacheie uma transação construída. Um cliente que escaneia, sai andando e assina dois minutos depois pega um blockhash velho e um submit que falha, o que é chato mas seguro; a carteira dele simplesmente pede de novo.

A carteira não podia simplesmente te mandar o carrinho inteiro nesse POST e pular o id de pedido? Não, e a restrição é uma feature. A carteira fala a spec, e o corpo do POST da spec é `{account}`, nada mais. Qualquer outra coisa de que o seu build precise tem que viajar na URL que você cunhou, que é exatamente por que a URL carrega um id de pedido opaco apontando para estado que o seu servidor já guarda. A carteira continua um signatário burro e auditável; o seu servidor continua o único autor da lógica de negócio. No momento em que você se pegar desejando que a carteira te mandasse mais, você geralmente está tentando mover a precificação de volta para o cliente, e você sabe como essa história termina.

Uma consequência do construir-no-POST merece parágrafo próprio: o POST não é idempotente, e tudo bem. Bata no seu endpoint duas vezes para o mesmo pedido e você recebe duas transações diferentes, cada uma com a própria reference key nova, porque o `buildOrderTransaction` cunha uma por chamada. Uma carteira que pede de novo depois de um blockhash velho faz exatamente isso. Relaxe: no máximo uma dessas transações liquida, o cliente assina uma, e o seu watcher casa com a reference que de fato aterrissar on-chain. O que você NÃO pode fazer é tratar "eu construí uma transação" como "eu fiz uma venda". Um build é um orçamento. A liquidação é a venda, e o módulo de back office formaliza essa distinção com idempotência chaveada por signature do lado do livro-razão.

### O que o servidor monta

Hora de ser concreto sobre a coisa que o seu handler de POST devolve. É uma transação versão 0 com o cliente como fee payer e duas instruções dentro.

A última instrução é o pagamento em si: um `TransferChecked` movendo o total do carrinho em USDC de devnet (mint `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`, 6 decimais) da conta de token associada do cliente para a sua. Isso é território do transfer-kit do módulo 2, e você reusa ele direto: o `resolveAta(owner, mint, tokenProgram)` encontra as duas contas de token, passando o Token program clássico para o USDC de devnet, o `toBaseUnits` mantém a matemática de dinheiro em bigints exatos. Os totais são calculados a partir de strings decimais, somados como unidades base, nunca em float. O único truque novo é que o cliente é a `authority` mas ainda não assinou nada; o seu servidor constrói uma transação não assinada e a signature chega depois, na carteira dele. O kit fica confortável com isso: você monta a mensagem, compila ela e serializa ela com o slot de signature vazio.

![Diagrama da transação devolvida: um envelope v0, o cliente como fee payer não assinado, um memo com o id do pedido e uma instrução TransferChecked carregando uma conta de reference readonly.](assets/v03-diagram.png)

Antes de seguir, o modo de falha que vai de fato te morder numa demo: contas de token que não existem. O `TransferChecked` move fundos entre contas de token associadas, e uma ATA só existe depois que alguém pagou o rent dela — o mínimo isento de aluguel para uma conta de token de 165 bytes, 1,488,440 lamports lidos da devnet em 2026-09-07 e caindo conforme o SIMD-0437 desce a alíquota em degraus — a rubrica que você custeou no módulo 2 perguntando ao cluster em vez de confiar numa constante. A sua ATA de destino de lojista existe porque o transfer-kit criou ela quando você recebeu USDC pela primeira vez. A ATA de origem do cliente é a arriscada: uma carteira que nunca segurou USDC de devnet não tem conta de USDC, a sua transação construída referencia um endereço sem nada atrás dele, e a simulação que a carteira roda antes de assinar falha com um erro que o cliente vai ler em voz alta para você. Não tem correção do lado do servidor dentro desta transação sem assumir o rent de estranhos; o tratamento honesto é uma checagem na loja (o saldo dele é visível on-chain antes de você renderizar o QR) e uma mensagem clara. Checkouts de produção fazem exatamente isso, e o seu vai fazer no capstone.

Imediatamente antes da transferência fica um SPL Memo carregando `wavelength:<orderId>:<description>`. Na lição do QR o memo viajava junto como parâmetro de URL e a carteira incluía ele; agora o seu servidor escreve ele direto, o que quer dizer que ele pode carregar o seu id de pedido de verdade e um resumo do carrinho parseável por máquina, e o cliente não consegue editar nenhum dos dois. O seu módulo de back office vai se apoiar pesado nisso.

E a reference? Mesma disciplina que você aprendeu com o `findReference`: uma chave nova de uso único por pagamento, gerada antes de o pagamento existir, para você conseguir localizar a transação depois. O que muda é onde ela mora. A carteira não está mais montando nada, então o seu servidor injeta a reference ele mesmo, como uma conta extra acrescentada à instrução de transferência: readonly, não signatária, puro marcador. Toda conta listada numa transação é indexada pelos nós RPC, que é o truque inteiro por trás das reference keys e sempre foi. Aqui estão as duas construções, exatamente como elas entram nos TODOs de completion mais adiante:

![As duas construções de código para os TODOs de completion: acrescentar a reference como conta readonly não signatária, e construir uma instrução de memo cujos dados são a string utf8 do pedido.](assets/v04-annotated-code.png)

Eu quero apontar a decisão discreta desse trecho, porque ela é do tipo que você vai tomar toda semana como engenheiro de pagamentos. A reference podia ter ido na instrução de memo; a indexação on-chain acharia ela do mesmo jeito. Ela vai na transferência porque é ali que a própria lógica de validação do `@solana/pay` procura referências adjacentes à transferência, e casar com a convenção que as suas ferramentas esperam ganha de uma esperteza particular todas as vezes. Eu já me queimei com a escolha oposta antes, um layout "melhor" que fez toda biblioteca downstream brigar comigo. Convenção é uma feature.

### A fronteira de confiança que você acabou de criar

Aqui está o trade-off, dito sem enfeite. Transações construídas no servidor te dão controle total: precificação, memos, references, qualquer instrução que o seu código consiga montar. Em troca, a carteira agora recebe uma transação completamente construída de um endpoint https e é solicitada a assinar ela. A proteção do cliente não é mais "eu consigo ler os parâmetros da URL". É a inspeção que a carteira faz do que voltou. Você mudou a fronteira de confiança, e alguma coisa tem que policiar a nova linha.

A spec policia isso com uma regra brutal: uma requisição que exige uma signature de uma conta que o usuário não submeteu é maliciosa, e a carteira precisa rejeitar ela. Sente com o que isso pega. Um servidor hostil, ou o seu comprometido, podia devolver um pagamento com cara perfeitamente válida que também exige uma signature de alguma outra conta: uma chave de tesouraria que o usuário por acaso controla, um membro de multisig, qualquer coisa que o atacante torce para ser aprovada no automático. A carteira sabe que exatamente uma conta foi oferecida, a que ela mesma mandou no POST. Todo outro signatário exigido na transação devolvida é uma exigência com que ninguém concordou, com uma exceção legítima: se o próprio servidor assinou a transação parcialmente, essas signatures chegam já fornecidas, e a carteira verifica elas em vez de ser solicitada a produzir elas.

![Diagrama de confiança em três zonas: o servidor do lojista confiado só para construir, a carteira impondo que todo signatário exigido não assinado seja a conta submetida, e o cliente se apoiando nessa checagem.](assets/v05-diagram.png)

Como uma carteira checa isso de verdade? Decodificando os bytes do wire antes de assinar, e você vai fazer o mesmo no cliente smoke desta lição, porque o seu cliente smoke está fazendo o papel da carteira. O layout faz o trabalho por você: uma mensagem compilada declara `numSignerAccounts` no cabeçalho dela, e a lista estática de contas dela vem ordenada com os signatários na frente. Fatie os primeiros `numSignerAccounts` endereços e você está segurando a lista completa de signatários exigidos; qualquer coisa nessa fatia que não seja a sua conta submetida, e que não esteja já carregando uma signature de verdade, é motivo para rejeição. O kit traz os decoders (`getTransactionDecoder`, `getCompiledTransactionMessageDecoder`), então a checagem tem uma dúzia de linhas, e escrever ela uma vez vai fazer mais pela sua intuição do que qualquer diagrama, que é por que o degrau solo faz você escrever ela.

Mais uma nota honesta já que estamos aqui: essa regra protege signatures, não julgamento. Uma carteira que impõe ela perfeitamente ainda vai alegremente apresentar uma transação que paga o valor errado ao lojista errado, e a defesa do cliente ali é o resumo decodificado da carteira mais o seu label e o seu icon. A regra é o piso da segurança de transaction request; o teto está em algum lugar bem acima dela. Trate ela como o mínimo que você verifica, e deixe o seu back office (próximo módulo) verificar todo o resto depois da liquidação.

### Para onde foi a energia da spec

Uma batida curta de realidade antes do lab, porque o chão embaixo desta lição se mexeu recentemente e você devia saber para que lado. Vá olhar o repositório canônico do Solana Pay hoje e o README que te recebe não é uma biblioteca de checkout por QR. Desde 2026-08-21, o repo abre com um CLI de pagamentos agênticos construído em torno de x402 e MPP, pagamentos HTTP dirigidos por máquina, com a biblioteca clássica de checkout sobrevivendo como um subdiretório. A energia de pagamentos da Foundation visivelmente saiu do carteira-escaneia-um-QR na direção de fluxos de pagamento programáticos e dirigidos por servidor.

![Linha do tempo da spec do Solana Pay de 2022, passando pela biblioteca de checkout virando um subpacote, até o repo de 2026 abrindo com um CLI de pagamentos agênticos.](assets/v06-timeline.png)

Leia esse arco de onde você está sentado hoje, uma lição adentro de transações construídas no servidor. Um agente pagando por HTTP e uma carteira respondendo a uma transaction request são o mesmo movimento: um servidor que precifica, constrói e devolve algo para assinar. O QR sempre foi um mecanismo de entrega. O que você está construindo nesta lição, o par GET-POST em torno de um builder no servidor, é o formato em que o ecossistema está dobrando a aposta, que é por que este endpoint, e não a página do QR, é o artefato que o resto deste curso continua consumindo. O módulo 7 encara de frente a ponta agêntica desse arco.

## Lab: construa o checkout-txreq

O layout que você está prestes a preencher, e onde ele fica no workspace da Wavelength:

![Diagrama do workspace, transfer-kit e checkout alimentando o checkout-txreq com seu catálogo, builder, servidor e smoke test, que a barraca da maquininha (POS), o blink do drop e o capstone consomem em lições posteriores.](assets/v07-diagram.png)

1. **O catálogo e as regras de precificação.** Crie `src/catalog.ts`. As interfaces e os dados da loja são dados; o `priceOrder` chega como o seu primeiro TODO de completion, com as regras que ele precisa implementar sentadas logo em cima dele:

   ```typescript
   // checkout-txreq/src/catalog.ts
   // The store's source of truth. Prices are decimal strings, parsed exactly;
   // no float ever touches money in this course.
   import { toBaseUnits, fromBaseUnits } from '../../transfer-kit/src/index';

   export interface CatalogEntry {
     title: string;
     priceUsdc: string; // decimal string, e.g. "22.5"
   }

   export interface OrderLine {
     sku: string;
     quantity: number;
   }

   export interface PricedOrder {
     baseUnits: bigint;   // final total in USDC base units (6 decimals)
     totalUsdc: string;   // display form of the same number
     description: string; // "1x WVL-001, 2x WVL-002"
   }

   export const CATALOG: Record<string, CatalogEntry> = {
     'LP-041': { title: 'Slow Tides, Wavelength pressing 041', priceUsdc: '12.5' }, // last lesson's record, now priced server-side
     'WVL-001': { title: 'Wavelength LP, first press', priceUsdc: '18' },
     'WVL-002': { title: 'Late Static Night, 12-inch', priceUsdc: '22.5' },
     'WVL-045': { title: 'The August pressing, limited', priceUsdc: '30' },
   };

   export function priceOrder(lines: OrderLine[], coupon?: string): PricedOrder {
     // Rule 1: reject an empty cart, an unknown sku, and any quantity outside 1..20.
     // Rule 2: subtotal in base units: toBaseUnits(entry.priceUsdc, 6) * BigInt(quantity),
     //         summed as bigint. fromBaseUnits(total, 6) gives you totalUsdc back.
     // Rule 3: description joins the lines: "1x WVL-001, 2x WVL-002".
     // (coupon is unused for now; it is your solo rung.)
     throw new Error('Your turn: compute the total per the three rules above.');
   }
   ```

   O caminho do import presume que o transfer-kit está um diretório ao lado, como está desde o módulo 2; ajuste ele ao seu layout. Mantenha o `WVL-045` em 30 exatamente, uma lição posterior vende aquela prensagem por este mesmo catálogo.

2. **O builder.** Crie `src/build-order-transaction.ts`. Este arquivo é o núcleo de pagamento que o resto do curso importa, então os dois nomes que ele exporta importam tanto quanto o comportamento dele: `buildOrderTransaction` para quem chama, `finalizeTransaction` como a cauda compartilhada. Tudo é dado, menos as duas injeções que você já viu na seção de teoria:

   ```typescript
   // checkout-txreq/src/build-order-transaction.ts
   // The payment core: price an order server-side, stamp the memo and reference,
   // return a base64 transaction for the submitted account to sign.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstructions,
     compileTransaction,
     createSolanaRpc,
     createTransactionMessage,
     generateKeyPairSigner,
     getBase64EncodedWireTransaction,
     pipe,
     setTransactionMessageFeePayer,
     setTransactionMessageLifetimeUsingBlockhash,
     type Address,
     type Instruction,
   } from '@solana/kit';
   import { getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta } from '../../transfer-kit/src/index';
   import { priceOrder, type OrderLine } from './catalog';

   const USDC_MINT = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU'); // devnet USDC, 6 decimals
   const USDC_DECIMALS = 6;
   const MEMO_PROGRAM = address('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'); // SPL Memo v2
   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';

   const rpc = createSolanaRpc(RPC_URL);

   function merchantAddress(): Address {
     const configured = process.env.MERCHANT_ADDRESS;
     if (!configured) throw new Error('set MERCHANT_ADDRESS to the wallet checkout already pays');
     return address(configured);
   }

   export interface BuildOrderInput {
     account: string;      // base58 pubkey the wallet POSTed; the ONLY signer we may demand
     sku?: string;         // single-line form (a later lesson calls it this way)
     quantity?: number;
     lines?: OrderLine[];  // multi-line cart form
     coupon?: string;      // wired on the solo rung
     orderId?: string;
   }

   export interface BuiltOrder {
     transactionBase64: string;
     reference: Address;
     memo: string;
     totalUsdc: string;
   }

   export async function buildOrderTransaction(input: BuildOrderInput): Promise<BuiltOrder> {
     const lines =
       input.lines ?? (input.sku ? [{ sku: input.sku, quantity: input.quantity ?? 1 }] : []);
     const priced = priceOrder(lines, input.coupon);

     const payer = address(input.account);
     const reference = (await generateKeyPairSigner()).address;
     const orderId = input.orderId ?? `ord-${Date.now().toString(36)}`;
     const memo = `wavelength:${orderId}:${priced.description}`;

     // resolveAta takes the owning token program as its third seed since the
     // roster lesson. Devnet USDC is a classic Token mint, so the program is
     // static here; no per-request detection round trip needed.
     const sourceAta = await resolveAta(payer, USDC_MINT, TOKEN_PROGRAM_ADDRESS);
     const destinationAta = await resolveAta(merchantAddress(), USDC_MINT, TOKEN_PROGRAM_ADDRESS);

     const transferIx = getTransferCheckedInstruction({
       source: sourceAta,
       mint: USDC_MINT,
       destination: destinationAta,
       authority: payer,
       amount: priced.baseUnits,
       decimals: USDC_DECIMALS,
     });

     const transactionBase64 = await finalizeTransaction({
       feePayer: payer,
       transferIx,
       reference,
       memo,
     });

     return { transactionBase64, reference, memo, totalUsdc: priced.totalUsdc };
   }

   export interface FinalizeInput {
     feePayer: Address;
     transferIx: Instruction;
     reference: Address;
     memo: string;
   }

   // The shared tail every surface reuses: inject the reference, stamp the memo,
   // set the lifetime, serialize. Later lessons call this directly.
   export async function finalizeTransaction(input: FinalizeInput): Promise<string> {
     // TODO(completion) 1: rebuild the transfer instruction with ONE extra account
     // appended: { address: input.reference, role: AccountRole.READONLY }.
     // Spread the existing accounts; never mutate the instruction you were given.
     const transferWithReference: Instruction = input.transferIx; // replace me

     // TODO(completion) 2: the memo instruction: programAddress MEMO_PROGRAM,
     // no accounts, data = new TextEncoder().encode(input.memo).
     const memoIx: Instruction = { programAddress: MEMO_PROGRAM, data: new Uint8Array() }; // replace me

     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayer(input.feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions([memoIx, transferWithReference], m),
     );

     return getBase64EncodedWireTransaction(compileTransaction(message));
   }
   ```

   Leia o pipe uma vez, de cima a baixo, porque ele é o modelo de transação inteiro do kit em cinco linhas: uma mensagem versão 0 vazia, um fee payer, um lifetime, instruções, depois compilar e serializar. O `generateKeyPairSigner` cunha a reference; você guarda só o endereço dela, a chave privada nunca é usada e nunca é armazenada. Repare também no que o `buildOrderTransaction` se recusa a aceitar: nenhum campo de valor existe na entrada dele. Não tem jeito de quem chama, ou de um cliente, entregar um preço para esta função.

3. **O servidor.** Crie `src/server.ts`. Totalmente trabalhado; este é o par GET e POST da seção de teoria tornado literal:

   ```typescript
   // checkout-txreq/src/server.ts
   // The transaction-request endpoint: GET answers with display metadata,
   // POST {account} answers with a base64 transaction built server-side.
   import express from 'express';
   import type { NextFunction, Request, Response } from 'express';
   import { buildOrderTransaction } from './build-order-transaction';
   import type { OrderLine } from './catalog';

   const app = express();

   // Cross-origin access, before any other middleware. Next lesson the
   // point-of-sale page -- served from https://localhost:3001 -- POSTs to
   // this endpoint itself, and a browser preflights that cross-origin JSON
   // POST with an OPTIONS request. Without these headers the preflight
   // fails and the real POST never leaves the page. A wallet scanning a QR
   // is not a browser and never preflights; the POS page is, and does.
   app.use((req: Request, res: Response, next: NextFunction) => {
     res.setHeader('Access-Control-Allow-Origin', '*');
     res.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS');
     res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
     if (req.method === 'OPTIONS') {
       res.sendStatus(204);
       return;
     }
     next();
   });

   app.use(express.json());
   app.use(express.static('public'));

   const BASE_URL = process.env.BASE_URL ?? 'http://localhost:3100';
   const PORT = Number(process.env.PORT ?? 3100);

   interface Order {
     lines: OrderLine[];
     coupon?: string;
   }

   // The web-cart path: your storefront creates the order BEFORE showing the QR,
   // so the URL only ever carries an opaque order id. In production this map is
   // your database; the demo cart below is what smoke.ts buys.
   const ORDERS = new Map<string, Order>([
     ['demo-cart', { lines: [{ sku: 'WVL-001', quantity: 1 }, { sku: 'WVL-002', quantity: 2 }] }],
   ]);

   app.get('/txreq', (_req: Request, res: Response) => {
     res.json({
       label: 'Wavelength Records',
       icon: `${BASE_URL}/icon.png`,
     });
   });

   app.post('/txreq', async (req: Request, res: Response) => {
     const account: unknown = (req.body as { account?: unknown } | undefined)?.account;
     if (typeof account !== 'string' || account.length === 0) {
       res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
       return;
     }

     const orderId = typeof req.query.order === 'string' ? req.query.order : 'demo-cart';
     const order = ORDERS.get(orderId);
     if (!order) {
       res.status(404).json({ message: `unknown order: ${orderId}` });
       return;
     }

     try {
       const built = await buildOrderTransaction({
         account,
         lines: order.lines,
         coupon: order.coupon,
         orderId,
       });
       console.log(`[txreq] order ${orderId}: ${built.totalUsdc} USDC, ref ${built.reference}`);
       res.json({
         transaction: built.transactionBase64,
         message: `Wavelength Records: ${built.totalUsdc} USDC`,
       });
     } catch (err) {
       res.status(400).json({
         message: err instanceof Error ? err.message : 'could not build the order transaction',
       });
     }
   });

   app.listen(PORT, () => {
     console.log(`checkout-txreq listening on :${PORT}`);
   });
   ```

   Jogue qualquer PNG quadrado em `public/icon.png` (qualquer arte de placeholder serve; as carteiras só precisam que a URL resolva) para a URL de icon do GET funcionar. O endpoint loga a reference em todo build; mantenha esse hábito, ela é a chave de junção em que tanto o seu watcher quanto o seu back office vivem.

   O bloco de CORS no topo merece frase própria, porque a ausência dele seria invisível hoje e fatal na próxima lição. Todo consumidor que este servidor conheceu até agora — curl, smoke.ts, uma carteira resolvendo um QR — fala com ele sem navegador no caminho, então nada impõe CORS e o servidor pareceria funcionar sem aqueles headers. A lição da maquininha (POS) muda o cliente: a *página* da maquininha busca a transação ela mesma, cross-origin, e um navegador se recusa a mandar esse POST a menos que o OPTIONS de preflight volte carimbado com `Access-Control-Allow-Origin`. O middleware responde ao preflight com um 204 vazio e carimba toda resposta. Guarde o padrão; a lição de blinks torna essa mesma regra estrutural para um canal de distribuição inteiro.

4. **Rode.** Em um terminal, com a sua carteira de lojista do módulo 2:

   ```bash
   MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
   ```

   Checkpoint: `checkout-txreq listening on :3100`, e `curl http://localhost:3100/txreq` devolve o seu JSON de label e icon. Repare que isso roda em http puro, o que está bem para o cliente local de hoje; carteiras de verdade exigem https para links de transaction request, e a lição da maquininha põe este servidor exato atrás de um proxy SSL local em vez de ensinar certificados duas vezes.

5. **O cliente smoke.** Crie `smoke.ts` na raiz do projeto. Ele faz o papel da carteira: GET, POST, depois decodifica a resposta e verifica que o seu trabalho de fato aterrissou dentro dos bytes:

   ```typescript
   // checkout-txreq/smoke.ts
   // Plays the wallet: GET the metadata, POST {account}, then decode what came
   // back and prove the memo and reference made it into the transaction.
   import {
     generateKeyPairSigner,
     getBase64Encoder,
     getCompiledTransactionMessageDecoder,
     getTransactionDecoder,
   } from '@solana/kit';

   const TXREQ_URL = process.env.TXREQ_URL ?? 'http://localhost:3100/txreq';
   const MEMO_PROGRAM = 'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr';

   function fail(msg: string): never {
     console.error(`SMOKE FAIL: ${msg}`);
     process.exit(1);
   }

   async function main(): Promise<void> {
     const customer = (await generateKeyPairSigner()).address;

     // 1. GET: the display step
     const get = await fetch(TXREQ_URL);
     if (get.status !== 200) fail(`GET returned ${get.status}`);
     const meta = (await get.json()) as { label?: unknown; icon?: unknown };
     if (typeof meta.label !== 'string' || typeof meta.icon !== 'string') {
       fail('GET must return { label, icon }');
     }

     // 2. POST {account}: the build step
     const post = await fetch(TXREQ_URL, {
       method: 'POST',
       headers: { 'Content-Type': 'application/json' },
       body: JSON.stringify({ account: customer }),
     });
     if (post.status !== 200) fail(`POST returned ${post.status}: ${await post.text()}`);
     const body = (await post.json()) as { transaction?: unknown; message?: unknown };
     if (typeof body.transaction !== 'string' || body.transaction.length === 0) {
       fail('POST must return a base64 transaction');
     }

     // 3. Decode the wire bytes and check your work landed inside them
     const wireBytes = getBase64Encoder().encode(body.transaction);
     const tx = getTransactionDecoder().decode(wireBytes);
     const message = getCompiledTransactionMessageDecoder().decode(tx.messageBytes);
     if (message.version !== 0) {
       // Solana has two envelope shapes, legacy and version 0. Kit decodes
       // both, but our server builds version 0 and that is what this smoke
       // asserts came back.
       fail('server did not return a version 0 transaction envelope');
     }

     const memoPresent = message.instructions.some(
       (ix) => message.staticAccounts[ix.programAddressIndex] === MEMO_PROGRAM,
     );
     if (!memoPresent) fail('no memo instruction in the built transaction: the finalize TODOs are still open');

     const transfer = message.instructions.find(
       (ix) => message.staticAccounts[ix.programAddressIndex] !== MEMO_PROGRAM,
     );
     if (!transfer || (transfer.accountIndices ?? []).length !== 5) {
       fail('transfer instruction has no injected reference account (expected 5: source, mint, destination, authority, reference)');
     }

     const lastIx = message.instructions[message.instructions.length - 1];
     if (message.staticAccounts[lastIx.programAddressIndex] === MEMO_PROGRAM) {
       fail('memo is the last instruction: validateTransfer pops the last instruction and expects the transfer');
     }

     console.log(
       `GET returned label/icon; POST {account} returned a base64 transaction for the cart total (${String(body.message ?? '')})`,
     );
   }

   main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
   ```

   Rode `npx tsx smoke.ts` em um segundo terminal. Com os TODOs ainda abertos ele falha, primeiro no `priceOrder` vazio, depois na instrução de transferência sem a conta de reference injetada. (O `memoIx` de placeholder já nomeia o program address certo, então a checagem de memo só dispara se você apagar o placeholder de vez; a contagem de cinco contas é o que pega o TODO aberto.) Essa sequência de falhas é a lista de tarefas do seu degrau de completion na ordem certa. O bloco de decodificação vale uma leitura lenta mesmo antes de você consertar qualquer coisa: ele é um terço da guarda de segurança da carteira que você vai terminar solo, e `numSignerAccounts` mais a ordenação com signatários na frente é todo o conhecimento extra de que aquele passo precisa.

6. **Aponte para ele a página que você já tem.** A sua página de checkout da lição passada ainda codifica uma transfer request. Duas edições pequenas lá no projeto checkout (onde o `@solana/pay` 1.0.26 já mora) trocam ela para a forma de link. Primeiro, em `checkout/server.ts`, carimbe um id de pedido em vez de uma URL pronta: mude a div do QR para `<div id="qr" data-order="demo-cart"></div>`. Três coisas acima dela podem ir junto, e devem: a construção da `url`, a chamada `newReference()` e o bloco `awaitSale(...)` com o log `checkout open ref=` dele. As três pertenciam ao fluxo de transfer request. Sob uma transaction request a reference é cunhada no servidor pelo `buildOrderTransaction` no *outro* workspace, então uma reference que esta página cunhe não chega a carteira nenhuma nem a transação nenhuma, e um watcher armado nela nunca consegue resolver — uma subscription WebSocket viva por carregamento de página, esperando para sempre, mais uma linha de log nomeando uma reference que venda nenhuma vai carregar. A vigilância se muda junto com a reference: agora é trabalho do `checkout-txreq`. Depois substitua o corpo de `checkout/page.ts`:

   ```typescript
   // wavelength-checkout/checkout/page.ts: the transaction-request swap
   import { encodeURL, createQR } from '@solana/pay';

   // The server now stamps only an opaque order id; this file builds the
   // solana:<https-link> URL and turns it into pixels.
   const mount = document.getElementById('qr')!;
   const link = new URL(`http://localhost:3100/txreq?order=${mount.dataset.order}`);
   const url = encodeURL({ link });
   const qr = createQR(url.toString(), 360, 'white', 'black');
   qr.append(mount);
   ```

   Re-empacote (`npx esbuild checkout/page.ts --bundle --format=esm --outfile=checkout/public/page.js`), depois recarregue a página com os dois servidores no ar: a página de checkout na :3010, este endpoint na :3100. Mesmo `encodeURL`, campo diferente: passe `link` em vez de `recipient` e a biblioteca emite `solana:<https-link>` em vez de `solana:<recipient>`. Essa mudança de um campo é a migração inteira do lado do cliente, o que te diz para onde foi todo o trabalho de verdade. Checkpoint: a página renderiza um QR cujo texto decodificado começa com `solana:http`. Mas seja honesto consigo mesmo sobre o que um celular vai fazer com ele: carteiras exigem https para links de transaction request, então uma carteira estrita vai recusar este QR em http puro. Hoje o seu cliente smoke é a carteira; na próxima lição um proxy SSL entra na frente da porta 3100 e este mesmo QR fica escaneável numa barraca de feira. A página está ligada agora para que aquela lição só tenha que adicionar o proxy.

## Challenge

**Completion.** Preencha os dois pontos de TODO: o `priceOrder` em `src/catalog.ts` pelas três regras dele, e as duas construções em `finalizeTransaction` pelos formatos anotados na seção de teoria. Aceitação: `npx tsx smoke.ts` imprime `GET returned label/icon; POST {account} returned a base64 transaction for the cart total` com o total do carrinho demo na mensagem. Faça a aritmética você mesmo antes de confiar na saída: 1 a 18 mais 2 a 22.5 dá 63 USDC, e se o seu servidor disser qualquer outra coisa, a sua matemática de unidades base tem um float em algum lugar. A minha primeira rodada imprimiu exatamente `Wavelength Records: 63 USDC`, e a reference logada ao lado disso é a coisa que o watcher da lição passada casaria assim que esta venda liquidar.

**Solo, em três partes, sem passo a passo.** Primeiro, o caminho do cupom: adicione uma tabela `COUPONS` (`CRATEDIG10` com 10 por cento de desconto é o código demo), estenda o `priceOrder` para aplicar ele a um carrinho de vários itens em matemática de bigint (arredonde o desconto para baixo; deriva de arredondamento a favor da loja é um ticket de reembolso esperando para acontecer), e adicione a `ORDERS` um pedido que carregue o cupom. Segundo, a guarda: escreva `assertNoForeignSignatureDemand(transactionBase64, submittedAccount)` no seu cliente smoke usando os decoders que você já importou. Fatie as primeiras `numSignerAccounts` entradas de `staticAccounts`; todo endereço nessa fatia precisa ou ser a conta submetida ou já carregar uma signature diferente de zero em `tx.signatures`, e qualquer outra coisa lança. Depois prove que a guarda dispara: construa localmente uma fixture com formato malicioso chamando `finalizeTransaction` com uma instrução adulterada que liste uma segunda conta como `WRITABLE_SIGNER`, e afirme que a sua guarda rejeita ela. Terceiro, liquide uma: faça o POST como o seu cliente de mentira (`/tmp/customer.json`, a carteira que a lição passada abasteceu) e não como o lojista, assine com aquela chave (carregue ela com `createKeyPairSignerFromBytes`, assine com `signTransaction`, mande o base64 pelo `rpc.sendTransaction`), e veja ela aterrissar na devnet. Dois motivos para ter que ser o cliente e não você: a transferência que este endpoint constrói corre da conta de token da conta que fez o POST para a sua, então postar o seu próprio endereço constrói uma transferência de lojista para lojista que não prova nada, e a carteira de origem tem que de fato segurar o total do carrinho, que é 63 USDC no carrinho demo. Se a sua carteira de cliente estiver curta, ou complete ela a partir do lojista com o `pay` do transfer-kit (e peça mais no faucet se o lojista também estiver curto) ou liquide um pedido menor que você mesmo adicionou a `ORDERS` — a guarda e a matemática do cupom são o que este degrau está julgando, não o tamanho do ticket.

Aceitação, direto do gate desta lição: o POST devolve uma transação base64 que é submetida na devnet pelo total exato calculado no servidor, cupom aplicado, e a sua guarda rejeita a requisição com formato malicioso enquanto deixa passar a honesta. Se a guarda em algum momento deixar a fixture passar, cheque se você fatiou os signatários a partir da frente de `staticAccounts`; fatiar de qualquer outro lugar é o erro clássico, porque o layout põe os signatários primeiro e nada te avisa se você ignorar isso.

Quando as duas metades passarem, repare no que você está segurando: uma loja cujos preços não dá para adulterar a partir de uma URL, e um cliente que se recusa a assinar por contas que ninguém ofereceu. Esse par, controle do servidor mais ceticismo da carteira, é o modelo de confiança inteiro das transaction requests, e você construiu os dois lados dele. Vale uma pausa para o café.

Antes de fechar o terminal: lições posteriores presumem que este endpoint entrou limpo, então se algum passo brigou com você — a ordem dos TODOs, o fatiamento com signatários na frente, o arredondamento do cupom — sinalize para a comunidade do curso enquanto está fresco. E se a sua guarda pegou a fixture na primeira rodada, comemore em voz alta; você acabou de escrever a mesma checagem que os times de carteira entregam.

Só que o seu endpoint só foi dirigido de uma aba de navegador e de um script smoke até agora. Este endpoint está prestes a sair do navegador de vez e chegar numa mesa dobrável numa feira de discos de fim de semana. Na próxima lição você aponta uma maquininha de verdade para ele, e encontra a realidade de hardware móvel que é receber pagamentos na Solana presencialmente.
