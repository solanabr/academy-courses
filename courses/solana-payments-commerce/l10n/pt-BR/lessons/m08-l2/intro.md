# Pagamentos diferidos: durable nonces na feira (e a queda que deixou cicatrizes neles)

## Resumo

Na lição passada você patrocinou taxas: um paymaster Kora co-assinou o checkout para que um comprador com USDC e zero SOL ainda saísse de lá com a prensagem. Esta lição remove a outra suposição que todo checkout que você construiu até aqui faz em silêncio, que é uma rede viva, no ar e alcançável no exato momento em que um comprador diz sim. A feira de discos é num porão. Não tem sinal. Você faz a venda mesmo assim.

Eis o que hoje estabelece, logo de cara:

- Uma transação normal carrega um prazo de relógio no qual você nunca precisou pensar: o blockhash dela vale por 150 blocos (`MAX_PROCESSING_AGE = 150`), o que no tempo de slot alvo atual de 300ms dá 45 segundos. Não "cerca de um minuto". Quarenta e cinco segundos, e encolhendo toda vez que o tempo de slot cai.
- Um durable nonce troca o blockhash por um valor guardado numa conta on-chain que não se mexe até você avançar ele. Assine no porão ao meio-dia, envie da calçada às seis, a transação continua válida.
- A primitiva tem duas cicatrizes, e as duas são estruturais: a documentação oficial avisa que ela pode ser depreciada, e um bug de processamento duplo de durable nonce parou a mainnet da Solana por cerca de 4.5 horas em 2022-06-01. Você vai construir a fila em torno das duas.
- O artefato é o `fair-queue`, um novo workspace irmão ao lado do pos-stall (a barraca em si não é editada hoje): vendas assinadas offline contra um pool de nonces, drenadas quando a conectividade volta, com um passo de drain que se recusa, estruturalmente, a retransmitir um nonce gasto.

Primeiro, ponha o workspace de pé. O `fair-queue` fica ao lado do `checkout-txreq` e do `pos-stall` no seu workspace da Wavelength:

```bash
cd ~/wavelength   # the workspace root; last lesson left you inside gasless-checkout
mkdir fair-queue && cd fair-queue
npm init -y && npm pkg set type=module
npm install @solana/kit@6.10.0 @solana-program/system@0.12.2
npm install -D tsx typescript @types/node
```

Notas de pin, conferidas em 2026-08-31: isto fica no workspace de kit v6 do curso, e o `@solana-program/system` 0.12.2 é o último release que faz peer com `@solana/kit ^6.x`. O release 0.13.0 (2026-07-15) pulou para kit ^7 e o 0.14.0 para kit ^8, então `npm install @solana-program/system` sem versão te entregaria um erro de peer-dependency contra este workspace. Fixe ele.

## Um cheque sem data

### O prazo com que você vem convivendo

Toda transação que este curso construiu até aqui, o checkout, a barraca, o crank das assinaturas, a compra patrocinada, carregava um `recentBlockhash`. Você tratou ele como um carimbo de frescor e seguiu em frente, e isso estava certo: para um checkout online o prazo nunca morde. Hoje ele morde, então vamos derivar ele de verdade.

A regra: o blockhash de uma transação não pode ter mais de 150 blocos, uma constante que o código do validador chama de `MAX_PROCESSING_AGE`. A profundidade da fila é fixa em blocos, não em segundos, então a janela de relógio é `150 x slot time`. Nos slots antigos de 400ms isso dava uns 60 segundos; na etapa de 350ms do SIMD-0525 (em vigor desde 2026-08-21) eram ~53. O alvo de hoje é 300ms, com a etapa seguinte tendo entrado em vigor na epoch 1024 em 2026-08-28, então a janela é 150 x 0.30 = 45 segundos. Mais dois cortes em etapas, 250ms e depois 200ms, já estão engatilhados no código e os dois já entraram em vigor na devnet, que está uma escada inteira à frente da mainnet: na mainnet as feature accounts de 250ms e 200ms ainda nem existem. Então a janela da mainnet encolhe de novo no dia em que cada uma virar lá, e 200ms é o fundo da escada do SIMD-0525. É por isso que este curso vive dizendo derive, nunca decore — e, como a probe logo abaixo está prestes a mostrar, por que você deriva do seu cluster e não da escada.

Não acredite na minha aritmética. Crie o `probe.ts` e veja um blockhash morrer em tempo real:

```typescript
import { createSolanaRpc } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

const {
  value: { blockhash, lastValidBlockHeight },
} = await rpc.getLatestBlockhash().send();
const start = Date.now();
console.log(`got ${blockhash}, valid until block height ${lastValidBlockHeight}`);

let height = await rpc.getBlockHeight().send();
while (height <= lastValidBlockHeight) {
  await new Promise((r) => setTimeout(r, 5000));
  height = await rpc.getBlockHeight().send();
}
console.log(`expired after ~${Math.round((Date.now() - start) / 1000)}s`);
```

Rode com `npx tsx probe.ts`. A minha imprimiu 28 segundos, com a margem do grão de 5 segundos do polling, e essa surpresa já é a lição: a probe roda contra a devnet, que corre à frente da mainnet no tempo de slot e portanto queima a janela de 150 blocos dela mais rápido — medido em 2026-09-07, a devnet estava produzindo um slot mais ou menos a cada 165ms contra os ~320ms da mainnet, no `solana-core` 4.3.0-beta.3. Leia 165ms duas vezes, porque está abaixo de 200ms, o fundo da escada acima, e isso não é typo. A devnet ativou todos os quatro gates, então o *alvo* dela é 200ms; ela estava rodando mais rápido que o próprio alvo. A mainnet, na etapa de 300ms, mediu ~320ms — mais devagar que o próprio alvo. Nenhum dos dois clusters fica no número declarado, e o próprio SIMD-0525 diz isso: a tabela fixa o trabalho por slot, hashes por tick e limites de bloco, e o SIMD anota que nenhum dos valores de tempo dele "are explicitly in protocol and may deviate from reality." A etapa é o que a rede mira, não um relógio ao qual ela é cobrada. O que é o argumento inteiro a favor da probe: `getRecentPerformanceSamples` devolve `numSlots` e `samplePeriodSecs`, e dividir um pelo outro é o tempo de slot atual do seu cluster, com escada ou sem escada. Não tire nenhum dos dois números desta página. Diga o que ela disser, a janela da devnet é mais apertada que os ~45 segundos da mainnet, e as duas continuam encolhendo. Imprima o que a sua probe imprimir, esse número é o enunciado inteiro do problema: no porão, o intervalo entre "o comprador assinou" e "você tem barrinhas de novo" se mede em horas, e a paciência da transação se mede em segundos. Nenhum loop de retry conserta isso, porque retransmitir uma transação expirada não estende a vida dela; o hash simplesmente é velho demais, e o RPC vai continuar te dizendo isso não importa o quão educadamente você pergunte de novo.

![Duas linhas do tempo horizontais: uma transação com blockhash morre por volta dos 45 segundos enquanto uma transação de durable nonce continua válida por horas até o nonce dela ser avançado.](assets/v01-timeline.png)

Se você veio de pagamentos com cartão, já viu esse problema resolvido antes. Maquininhas store-and-forward passam cartão em avião e em porão há décadas: a maquininha registra a autorização offline e manda o lote quando reconecta, e o adquirente resolve o risco depois. A razão de a Solana precisar de uma primitiva dedicada para o mesmo movimento é que não existe adquirente para absorver ambiguidade. A validação é global e mecânica, todo nó precisa concordar sobre se uma transação está fresca, e a regra de frescor é aquela janela de 150 blocos. Então a versão do store-and-forward nativa da blockchain não pode só segurar bytes e torcer; ela tem que mudar o que "fresco" significa para aquela transação.

A bala de prata? Uma transação cujo carimbo de frescor é você quem controla. É exatamente isso que um durable nonce é: em vez de apontar para um bloco recente, a transação aponta para um valor guardado numa conta, e esse valor só muda quando a autoridade dele manda. A documentação oficial põe isso em uma frase: transações de durable nonce trocam o blockhash recente por um valor de nonce guardado, removendo a janela de expiração de 150 slots, o que habilita assinar offline e enviar depois. Um cheque sem data nele.

### A conta de nonce, campo a campo

O valor guardado mora numa conta de nonce: uma conta de propriedade do System Program, 80 bytes de dados, que precisa ser isenta de aluguel (cerca de 0.00106 SOL nesse tamanho na devnet em 2026-09-07, e caindo; o seu código pergunta ao RPC em vez de hardcodar, que é a razão inteira de esse número ser seguro de imprimir aqui). Busque uma com o client gerado e você recebe o estado inteiro de volta tipado:

```typescript
import { fetchNonce } from '@solana-program/system';

const { data } = await fetchNonce(rpc, nonceAccountAddress);
// data.version               -> account format version
// data.state                 -> must be Initialized to back a transaction
// data.authority             -> the pubkey allowed to advance, withdraw, reauthorize
// data.blockhash             -> THE nonce value: a hash derived from a recent blockhash
// data.lamportsPerSignature  -> the fee rate captured when the nonce last advanced
```

Dois campos fazem o trabalho. `authority` é a autoridade do nonce, a conta que precisa assinar qualquer instrução que mexa nesse nonce; para a fila da feira essa é a chave do lojista, o mesmo signatário que roda o checkout-txreq. E `blockhash` é o valor do nonce em si. O nome não é acidente: o valor é um hash derivado de um blockhash de verdade no momento do último advance, e ele entra no campo `recentBlockhash` da transação como os mesmos 32 bytes, então o formato de wire nunca muda. O que muda é como o runtime valida ele. O campo restante, `lamportsPerSignature`, registra a alíquota de taxa capturada quando o nonce avançou pela última vez, um resquício do papel da conta na contabilidade de taxas que você vai ler e nunca tocar.

Custo, já que um lojista deveria sempre saber qual é. O depósito de rent para 80 bytes era de 1,056,640 lamports na devnet e 1,317,264 na mainnet quando eu li em 2026-09-07, e os dois continuam caindo conforme o SIMD-0437 vai baixando a alíquota por byte em degraus — então leia o seu de `getMinimumBalanceForRentExemption(80)` e não desta frase. É um depósito, não uma taxa: `WithdrawNonceAccount` devolve cada lamport para a autoridade no dia em que você aposentar um slot, então um pool de quatro slots prende cerca de quatro vezes esse número enquanto você tocar a barraca e não te custa nada desmontar. Por transação, o caminho de durable nonce é um pouco mais pesado que o de blockhash, já que toda venda carrega a instrução extra de advance e as contas dela. Para um fluxo de pagamentos essa troca é invisível; a matemática da taxa base que você fez no módulo 1 ainda domina.

![Uma conta de nonce de 80 bytes disposta campo a campo: version, state, a pubkey de authority de 32 bytes mapeada para a chave do lojista, o valor de nonce guardado de 32 bytes, e a alíquota de taxa.](assets/v02-diagram.png)

### Instrução 0 ou nada

Como um validador sabe que uma transação está usando um durable nonce e não um blockhash velho? Ele olha para exatamente um lugar: a instrução 0. Se, e somente se, a primeira instrução da transação for o `AdvanceNonceAccount` do System Program, com a conta de nonce como a primeira conta dessa instrução e gravável, o runtime troca de modo de validação. Ele carrega a conta de nonce, checa se ela parseia como `Initialized`, e checa se o valor guardado bate com o campo `recentBlockhash` da transação. Bateu, a transação segue; a instrução de advance então rola o valor guardado para um hash novo, que é o que faz esse nonce ser de uso único. Ponha o advance em qualquer lugar que não o primeiro e você não tem uma transação de durable nonce de jeito nenhum, só uma normal carregando um blockhash que a fila já vai ter deixado expirar faz tempo.

A instrução em si é minúscula: três contas (conta de nonce, o sysvar de recent-blockhashes, a autoridade como signatário) e quatro bytes de dados, `[4, 0, 0, 0]`, o discriminador do System Program para AdvanceNonceAccount. Esses quatro bytes valem a pena decorar porque o seu lab dá assert neles.

![A instrução AdvanceNonceAccount anotada: System Program, a conta de nonce gravável, o sysvar de recent-blockhashes, a autoridade como signatário, e os bytes de dados 4,0,0,0, válida só na posição de instrução zero.](assets/v03-annotated-code.png)

No kit você nunca monta essa instrução na mão para as suas próprias transações, porque o helper de lifetime faz isso por você. Onde o builder da lição passada chamava `setTransactionMessageLifetimeUsingBlockhash`, a fila da feira chama o irmão dele:

```typescript
import { setTransactionMessageLifetimeUsingDurableNonce, type Nonce } from '@solana/kit';

const message = setTransactionMessageLifetimeUsingDurableNonce(
  {
    nonce: nonceValue as Nonce,          // data.blockhash from fetchNonce, cached
    nonceAccountAddress,                  // the pool account backing this sale
    nonceAuthorityAddress,                // the merchant key
  },
  transactionMessage,
);
```

Uma chamada faz dois trabalhos: ela define a restrição de lifetime da mensagem como o valor do nonce, e ela prefixa a instrução AdvanceNonceAccount para que ela caia no índice 0. Crédito a quem merece, esse é o tipo de remoção de footgun que o kit faz bem; nos tempos do web3.js legado, esquecer de pôr o advance primeiro era uma falha silenciosa clássica. Um requisito continua sendo seu, porém: a autoridade do nonce precisa de fato assinar. No fair-queue o lojista é ao mesmo tempo fee payer e autoridade do nonce, então o único signatário embutido cobre os dois papéis e o `signTransactionMessageWithSigners` resolve tudo sem nenhuma rede por perto. Divida esses papéis em duas chaves e as duas precisam estar presentes na hora de assinar. Essa divisão é também a postura de produção: a lição do gate de entrada no ar vai pedir uma chave dedicada de autoridade de nonce, para que um notebook de barraca roubado comprometa uma fila e não o caixa, e o build de uma chave só deste lab é uma concessão de testabilidade que ela vai te fazer corrigir, não uma recomendação.

Repare no que assinar offline significa aqui. A transação assinada é um blob de bytes autocontido. Quem segurar esses bytes pode enviar eles, de qualquer máquina, a qualquer hora, até o nonce avançar. O enquadramento da própria documentação é seco: não existe restrição nenhuma sobre quão velho o nonce é. O seu arquivo de fila portanto não é um log; é uma gaveta cheia de cheques assinados e sem data. Qualquer um que segure um pode enviar ele — mas não redirecionar ele: o beneficiário está cozido dentro dos bytes assinados, então um ladrão não ganha nada além do poder de descontar o seu cheque no seu próprio caixa num momento que você não escolheu, e um cheque descontado pelas suas costas é o que transforma o seu re-assinar posterior numa cobrança dupla. Trate o `queue.json` com exatamente a seriedade com que você trataria aquela gaveta, porque daqui para frente o perigo nesta lição não é a expiração e sim o oposto dela.

### As duas cicatrizes

A primeira cicatriz está escrita direto na documentação oficial, e eu quero que você leia ela como está escrita e não a minha paráfrase: "Durable nonces may be deprecated in a future release," com um ponteiro para uma discussão de SIMD em aberto (docs do solana.com, re-buscados em 2026-08-31). Essa discussão agora tem uma proposta numerada por trás: o SIMD-0571, "Soft Deprecation of Durable Nonce Transactions", um pull request aberto e não mergeado no repo de SIMDs em 2026-08-31, o que quer dizer proposto, contestado e não aceito. Fique um tempo com o quanto isso é incomum, porque a página de docs que ensina a feature abre te avisando que a feature pode sumir. Construa a fila da feira, suba ela, mas arquitete de modo que o módulo da fila seja trocável: no dia em que a depreciação aterrissar, você quer trocar um diretório, não o seu checkout.

A segunda cicatriz é a razão de a primeira existir. Em 2022-06-01, um bug no tratamento de durable nonce deixou certas transações de nonce serem processadas duas vezes. Os validadores discordaram sobre o resultado, o consenso travou, e a mainnet parou por cerca de 4.5 horas. A resposta depois foi drástica: a feature foi desabilitada temporariamente na rede inteira enquanto a lógica do runtime era consertada. Leia isso como lojista, não como historiador de protocolo. Processar duas vezes uma transação de pagamento significa um comprador cobrado duas vezes, e a classe de falha não era exótica: transações que deveriam ser irrepetíveis foram honradas de novo. Repare no tempo verbal, porém — consertada. O reparo de 2022 separou os domínios de validação de durable nonce e de blockhash, então hoje um validador em conformidade rejeita deterministicamente bytes cujo nonce avançou: o valor guardado não bate mais com o `recentBlockhash` da transação, exatamente a checagem que você leu na seção da instrução 0. Então por que o seu drain ainda trata um nonce gasto como radioativo? Não porque a retransmissão possa cobrar duas vezes — o runtime fechou essa porta. Porque toda retransmissão de bytes mortos é um envio desperdiçado e um borrão no livro-razão, e porque a cobrança dupla que continua viva pertence inteiramente a você: RE-ASSINAR a mesma venda contra um nonce novo. A regra de nonce gasto do drain existe para que ninguém tome essa decisão às 6 da tarde com um arquivo de fila aberto e nenhum registro do que já aterrissou.

![Linha do tempo da queda da mainnet da Solana em 2022-06-01: um bug de processamento duplo de durable nonce para a rede por cerca de 4.5 horas, a origem da regra de nonce gasto do drain.](assets/v04-timeline.png)

Mais um pedaço de honestidade de lojista antes do livro-razão. Uma venda enfileirada não é uma venda liquidada. A signature na sua gaveta prova que o comprador autorizou o pagamento ao meio-dia; ela não prova que o pagamento vai aterrissar às seis, porque um envio na hora do drain ainda pode falhar como qualquer outra transação, o mais claramente quando o saldo do comprador foi gasto em outro lugar durante a tarde. Lojistas de cartão convivem exatamente com isso desde que o store-and-forward existe, e a postura é a mesma: entregue o disco na barraca se as suas margens toleram o risco, ou segure itens de alto valor para retirada depois que o drain confirmar. De um jeito ou de outro o seu livro-razão precisa de dois estados, enfileirada e aterrissada, e só o segundo é receita. O backoffice que você construiu no módulo 4 já tem a metade aterrissada; o arquivo de fila é a outra.

Então aqui está o livro-razão honesto sobre esta primitiva. Um durable nonce remove o prazo de relógio, e para uma fila offline isso é uma bênção. É também exatamente por isso que ele é perigoso: os bytes assinados são títulos ao portador até o nonce deles avançar, um drain que não sabe distinguir aterrissada de perdida convida a cobrança dupla do re-assinar, e a feature vem com o próprio aviso de depreciação. Para o checkout online que você construiu no módulo 3, nada dessa troca vale a pena; um blockhash novo é mais simples, mais seguro e auto-expirável, e isso continua sendo o default certo. Recorra a durable nonces só quando assinar tiver que acontecer longe da rede, que é precisamente, e apenas, o que a feira é. Uma nota de fronteira antes de a gente construir: quem envia também usa durable nonces online, como uma política deliberada de aterrissagem de transação para retries e signatários lentos. Esse é outro assento com outra matemática, e durable nonces como política de quem envia são território do curso de Client-Side Mastery. Aqui a gente fica na barraca.

## Lab: a fila da feira

A sua parte do trabalho, em voz alta: este é o módulo 8, território solo. Os passos trabalhados abaixo te entregam a infraestrutura, o pool de nonces e o snapshot da manhã, completos e rodáveis, porque encanamento de conta não é a lição desta lição. O signer offline e o drain são especificados como critérios de aceitação com os fragmentos estruturais mostrados, e você monta os arquivos sozinho. O classificador do drain no final é o desafio de código sem guia. No capstone, tudo isso é seu de qualquer jeito.

O dia tem um formato, e os scripts seguem ele:

![Ciclo de quatro estágios: crie o pool de nonces uma vez online, tire um snapshot dos valores de nonce toda manhã, assine vendas offline numa fila durante a feira, e drene a fila com segurança quando voltar a ficar online.](assets/v05-flowchart.png)

1. **Chaves e funding.** O `merchant.json` precisa ser *a* chave do lojista, a mesma para a qual o checkout-txreq paga, porque as vendas da fila creditam aquela carteira e o drain concilia contra o mesmo livro-razão. Essa chave é a identidade de CLI que o módulo 2 criou, então copie ela em vez de cunhar uma nova — `solana-keygen new -o merchant.json` te entregaria uma carteira diferente, e nada rio abaixo te avisaria: as vendas aterrissariam, na loja errada. O comprador de demonstração, em contraste, é genuinamente novo; ele faz as vezes da carteira de cliente que assinaria numa barraca de verdade:

   ```bash
   cp ~/.config/solana/id.json merchant.json    # THE merchant key, not a new one
   solana-keygen new -o buyer.json --no-bip39-passphrase
   solana airdrop 2 "$(solana-keygen pubkey merchant.json)" --url devnet
   solana airdrop 2 "$(solana-keygen pubkey buyer.json)" --url devnet
   ```

   Confirme a cópia antes de construir em cima dela: `solana-keygen pubkey merchant.json` precisa imprimir exatamente o que `solana address` imprime. Uma nota para frente, porque a próxima lição audita isso: hoje essa chave única veste três chapéus — recebedora do pagamento, fee payer e autoridade do nonce — o que está bem para um lab e é exatamente o que a linha de separação de chaves do prod-gate vai te pedir para dividir.

   Seja claro sobre o que o `buyer.json` é: um substituto de demonstração, e ele está escondendo o único problema genuinamente não resolvido deste design, então deixa eu nomear ele em vez de acenar para ele. Numa barraca de verdade a carteira do comprador precisa assinar no dispositivo dele, mas o fluxo de QR do módulo 3 não consegue entregar essa signature aqui: uma transaction request faz a carteira buscar a transação no seu servidor pela rede, e a premissa inteira desta lição é que não existe rede. Levar uma mensagem não assinada até o celular do comprador e trazer os bytes assinados de volta sem sinal exige um transporte local, um ida-e-volta de QR, NFC ou BLE, e construir um está fora do escopo deste curso, que é por que o keypair de demonstração é o único caminho de comprador suportado aqui. O que a fila em si precisa sobrevive intacto a essa limitação: ela não liga para quem produziu a signature do comprador, só para que a chave do lojista segure os dois papéis que importam, fee payer e autoridade do nonce, então quando existir uma integração de carteira por transporte local, o design da fila não muda.

   Checkpoint: os dois airdrops confirmam, e `solana balance "$(solana-keygen pubkey merchant.json)" --url devnet` e o mesmo comando contra o `buyer.json` imprimem 2 SOL cada.

2. **Crie o pool.** Eis o fato que dá forma ao artefato inteiro: um nonce sustenta uma transação. O advance que valida a venda A também invalida qualquer outra coisa assinada contra aquele mesmo valor, então uma fila de N vendas pendentes precisa de N contas de nonce. Isso é um pool de nonces, e o nosso tem quatro slots, cada um uma conta de 80 bytes criada e inicializada numa única transação de duas instruções. Quatro é uma decisão de dimensionamento, não um número mágico: o pool limita quantas vendas você consegue segurar entre drains, então dimensione ele para o trecho offline mais longo que você espera vezes a sua taxa de vendas, e lembre que cada slot extra custa só um depósito de rent reembolsável. Uma feira de porão com uma escadaria de sinal a cada uma ou duas horas faz de quatro bastante para uma demonstração; um fim de semana de festival ia querer mais. Crie o `create-pool.ts`:

   ```typescript
   import { readFileSync, writeFileSync } from 'node:fs';
   import {
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     generateKeyPairSigner,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
   } from '@solana/kit';
   import {
     getCreateAccountInstruction,
     getInitializeNonceAccountInstruction,
     getNonceSize,
     SYSTEM_PROGRAM_ADDRESS,
   } from '@solana-program/system';

   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
   const WS_URL = process.env.WS_URL ?? 'wss://api.devnet.solana.com';
   const POOL_SIZE = 4;

   const rpc = createSolanaRpc(RPC_URL);
   const rpcSubscriptions = createSolanaRpcSubscriptions(WS_URL);
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

   const merchant = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync('merchant.json', 'utf8'))),
   );

   const space = BigInt(getNonceSize());
   const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

   const pool: string[] = [];
   for (let i = 0; i < POOL_SIZE; i++) {
     const nonceAccount = await generateKeyPairSigner();
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (tx) => setTransactionMessageFeePayerSigner(merchant, tx),
       (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
       (tx) =>
         appendTransactionMessageInstructions(
           [
             getCreateAccountInstruction({
               payer: merchant,
               newAccount: nonceAccount,
               lamports: rent,
               space,
               programAddress: SYSTEM_PROGRAM_ADDRESS,
             }),
             getInitializeNonceAccountInstruction({
               nonceAccount: nonceAccount.address,
               nonceAuthority: merchant.address,
             }),
           ],
           tx,
         ),
     );
     const signed = await signTransactionMessageWithSigners(message);
     assertIsTransactionWithBlockhashLifetime(signed);
     await sendAndConfirm(signed, { commitment: 'confirmed' });
     console.log(`nonce ${i}: ${nonceAccount.address} (${getSignatureFromTransaction(signed)})`);
     pool.push(nonceAccount.address);
   }

   writeFileSync('nonce-pool.json', JSON.stringify(pool, null, 2));
   console.log(`pool of ${POOL_SIZE} written to nonce-pool.json`);
   ```

   Rode `npx tsx create-pool.ts`. Checkpoint: quatro endereços imprimem com signatures, e o `nonce-pool.json` existe. Repare no que esta transação em si usa: um lifetime de blockhash simples. A criação do pool acontece em casa, online, então ela não precisa de nonce; a graça do pool é o que acontece depois. Repare também em `nonceAuthority: merchant.address`, o padrão de autoridade separada: a conta de nonce tem o keypair descartável dela, mas o controle pertence à chave do lojista. O keypair da conta assina uma vez na criação e nunca mais é necessário.

3. **O snapshot da manhã.** Assinar offline exige saber cada valor de nonce antes de você perder o sinal, então o último ato online da manhã é cachear eles. Crie o `snapshot.ts`:

   ```typescript
   import { readFileSync, writeFileSync, existsSync } from 'node:fs';
   import { address, createSolanaRpc } from '@solana/kit';
   import { fetchNonce } from '@solana-program/system';

   type QueueSlot = { nonceAccount: string; nonceValue: string };

   const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
   const rpc = createSolanaRpc(RPC_URL);

   const pool = JSON.parse(readFileSync('nonce-pool.json', 'utf8')) as string[];
   const pending = existsSync('queue.json')
     ? (JSON.parse(readFileSync('queue.json', 'utf8')) as { pending: unknown[] }).pending
     : [];
   const busy = new Set(
     (pending as { nonceAccount: string }[]).map((e) => e.nonceAccount),
   );

   const free: QueueSlot[] = [];
   for (const nonceAccount of pool) {
     if (busy.has(nonceAccount)) continue; // still backing a queued sale
     const { data } = await fetchNonce(rpc, address(nonceAccount));
     free.push({ nonceAccount, nonceValue: data.blockhash });
   }

   writeFileSync('queue.json', JSON.stringify({ free, pending }, null, 2));
   console.log(`snapshot: ${free.length} free nonces cached, ${pending.length} sales still pending`);
   ```

   Checkpoint: `npx tsx snapshot.ts` imprime `4 free nonces cached, 0 sales still pending`, e o `queue.json` segura quatro pares `{ nonceAccount, nonceValue }`. Esses valores cacheados são o seu estoque do dia de cheques sem data, em branco e já assinados.

4. **O signer offline: a sua construção.** Andaimes abaixados. Escreva o `sign-sale.ts` de modo que `npx tsx sign-sale.ts <lamports> "<label>"` feche uma venda inteiramente offline. Critérios de aceitação:

   - Ele carrega o `merchant.json` e o `buyer.json` como signers, tira o próximo slot livre do `queue.json`, e lança erro com uma mensagem clara quando o pool acaba (quatro vendas pendentes e nenhum drain significa que a barraca para de aceitar pedidos; diga isso).
   - Ele monta a mensagem com o lojista como fee payer (a barraca cobre a taxa de rede; repare que essa é uma postura diferente da lição passada, onde uma chave dedicada de sponsor Kora pagava e a chave do lojista ficava fora do caminho quente) e o lifetime de durable nonce a partir do `nonceValue` cacheado do slot, usando a chamada `setTransactionMessageLifetimeUsingDurableNonce` da seção de teoria, com o lojista como autoridade do nonce.
   - Ele acrescenta um `getTransferSolInstruction({ source: buyer, destination: merchant.address, amount: lamports(saleLamports) })` como a venda em si. Uma transferência de SOL pelada, deliberadamente: o lifetime de nonce é ortogonal ao que a transação de fato faz, então o payload mais simples possível mantém a única ideia nova sem bagunça, e é por isso que o `buyer.json` levou um airdrop em vez do USDC que todo outro degrau usa. Troque pelo `TransferChecked` do módulo 2 e nem uma linha do snapshot, da guarda ou do drain muda.
   - Ele assina com `signTransactionMessageWithSigners`, e acrescenta ao array `pending` do `queue.json` uma entrada com exatamente este formato, porque o drain, o smoke test e mais tarde o harness de jornada do capstone leem todos ela:

   ```typescript
   type QueueEntry = {
     kind: 'durable-nonce';
     nonceAccount: string;   // the pool slot backing this sale
     nonceValue: string;     // the cached nonce it was signed against
     signature: string;      // getSignatureFromTransaction(signed)
     wire: string;           // getBase64EncodedWireTransaction(signed)
     signedAtSeconds: number;
     label: string;
   };
   ```

   E uma guarda que eu não vou deixar ao acaso, cole ela literalmente entre montar e assinar. Ela re-deriva a regra da seção de teoria contra a mensagem que você de fato montou:

   ```typescript
   // The runtime only treats this as a nonce transaction if
   // AdvanceNonceAccount sits at instruction 0. Prove it before signing.
   const [ix0] = message.instructions;
   const isAdvance =
     ix0 !== undefined &&
     ix0.programAddress === SYSTEM_PROGRAM_ADDRESS &&
     ix0.data !== undefined &&
     ix0.data.length === 4 &&
     ix0.data[0] === 4;
   if (!isAdvance) throw new Error('instruction 0 is not AdvanceNonceAccount: refusing to sign');
   ```

   Checkpoint, e esse é o divertido: desligue o seu wifi. Desligado de verdade. Então `npx tsx sign-sale.ts 1000000 "Kind of Blue, original pressing"`. Entra na fila. Nada no caminho de assinar toca a rede, o que você acabou de provar por construção. Assine uma segunda venda enquanto está lá embaixo; a fila segura as duas, cada uma contra o próprio slot do pool.

5. **A prova de 90 segundos e o drain.** Wifi ainda desligado, olhe para o relógio: qualquer transação com blockhash assinada quando você ficou sem sinal morreu na marca dos 45 segundos (antes disso na devnet, como a sua probe mostrou). Espere até terem passado pelo menos 90 segundos desde a sua primeira rodada de `sign-sale.ts`, que é o atraso que o harness de verificação também usa, confortavelmente além da janela. Então reconecte e escreva o `drain.ts` com estes critérios:

   - Para cada entrada pendente, ele chama `fetchNonce` no `nonceAccount` da entrada e compara o `data.blockhash` vivo com o `nonceValue` cacheado da entrada. Essa comparação é a história de segurança inteira, então ela acontece antes de qualquer outra coisa.
   - Os valores batem: o cheque ainda não foi descontado. Envie os bytes guardados exatamente como assinados e confirme com polling de status. Dois fragmentos carregam o peso de API aqui; eles se apoiam em `type Base64EncodedWireTransaction` e `signature` do `@solana/kit`, mais o import de `SYSTEM_PROGRAM_ADDRESS` que a guarda do passo 4 usou, então os três pertencem ao bloco de imports do drain.ts:

   ```typescript
   const sig = await rpc
     .sendTransaction(entry.wire as Base64EncodedWireTransaction, {
       encoding: 'base64',
       preflightCommitment: 'confirmed',
     })
     .send();
   ```

   ```typescript
   // A nonce tx has no blockhash deadline, so the blockhash-based confirm
   // helper does not apply. Poll the signature status instead.
   // searchTransactionHistory is load-bearing: without it the RPC consults
   // only its short-lived status cache (about a minute of recent slots), so
   // a sale that landed in an earlier drain reads back null here and would
   // be misrouted to UNSAFE -- whose remedy, re-selling against a fresh
   // nonce, is a double-charge. Same guard the refund builder uses.
   const { value: statuses } = await rpc
     .getSignatureStatuses([signature(entry.signature)], { searchTransactionHistory: true })
     .send();
   const s = statuses[0];
   const landed = s !== null && s !== undefined && s.err === null &&
     (s.confirmationStatus === 'confirmed' || s.confirmationStatus === 'finalized');
   ```

   - Os valores diferem: o nonce está GASTO, e este ramo não pode conter uma chamada de envio. Concilie em vez disso: se o `getSignatureStatuses` — perguntado com `searchTransactionHistory: true`, exatamente como no polling acima, já que a entrada pode ter aterrissado muitos minutos atrás — mostra a signature da própria entrada confirmada sem erro, um envio anterior já aterrissou ela, registre `RECONCILED` e libere o slot. Se o nonce mexeu mas a sua signature não está em lugar nenhum, registre `UNSAFE` e guarde a venda para uma decisão humana (re-vender contra um nonce novo); de todo jeito os bytes guardados estão mortos. Este ramo é a lição de 2022 como caminho de código.
   - Entradas aterrissadas e conciliadas saem de `pending`; um `snapshot.ts` seguinte re-busca os novos valores dos slots delas e devolve eles para `free`. Isso é reciclagem de nonce: o pool é quatro contas para sempre, não quatro contas por dia.

   Checkpoint: `npx tsx drain.ts` imprime `LANDED` com uma signature de devnet para cada venda, minutos depois de assinadas, e o explorer mostra a instrução 0 da sua transação como AdvanceNonceAccount com dados `[4, 0, 0, 0]`. Um blockhash jamais teria atravessado aquele intervalo.

6. **A tentativa de replay.** Rode `npx tsx drain.ts` uma segunda vez sem tirar snapshot. Toda entrada que ele acabou de aterrissar agora cai no ramo de nonce gasto, imprime `RECONCILED` e, essa é a asserção que importa, não envia nada. Então feche o ciclo com um harness que você escreve sozinho: `verify/fair-queue.smoke.ts`, quatro asserts de comprimento, dirigindo os seus próprios scripts de ponta a ponta. Ele assina uma venda, segura ela 90 segundos, drena ela, decodifica a instrução 0 para confirmar AdvanceNonceAccount (reuse a guarda do passo 4), e repete o drain esperando zero reenvios. Verde significa que o fair-queue é real.

![Fluxograma de decisão para drenar uma entrada: valores de nonce que batem enviam e confirmam, enquanto um nonce gasto é roteado para conciliação ou para um veredito unsafe que nunca reenvia.](assets/v06-flowchart.png)

## Challenge

O drain que você escreveu dá conta de uma fila numa tarde feliz. O desafio solo, `nonce-queue-drain`, é a versão geral, lógica pura, sem RPC: chega uma fila mista segurando entradas de lifetime de blockhash e de durable nonce (uma barraca realista assina online quando tem sinal e offline quando não tem), e você classifica cada entrada antes de qualquer coisa ser enviada.

```typescript
declare function drainFairQueue(
  nowSeconds: number,
  windowSeconds: number, // ~45 today; a parameter because slot time moves
  queueJson: string // the queue, flattened to one JSON string (see below)
): { submit: string[]; expired: string[]; unsafe: string[] };
```

Repare no formato antes de começar: três argumentos posicionais na entrada, e três listas de strings de `id` na saída, não três listas de entradas. O grader não consegue entregar para a sua função um objeto ou um literal de array, então a fila chega serializada:

- **Formato de wire.** `queueJson` é uma única string JSON segurando um array plano com quatro campos por entrada, em ordem de fila — `[id, signedAtSeconds, kind, nonceAdvanced]` repetido — onde `kind` é `'blockhash'` ou `'nonce'` (a entrada de durable nonce da sua fila, achatada) e `nonceAdvanced` é `0` ou `1`. O starter já dá `JSON.parse` nessa string e remonta itens tipados de verdade para você; o decode é encanamento, não a lição, e o seu trabalho começa no loop de classificação.
- **Regra um.** Entradas de blockhash expiram por relógio: um id cai em `submit` só quando `nowSeconds - signedAtSeconds <= windowSeconds`, senão ele cai em `expired`.
- **Regra dois.** Entradas de durable nonce nunca expiram por relógio, mas `nonceAdvanced === true` roteia para `unsafe`, nunca para `submit`, não importa quão nova a entrada seja.
- **Regra três.** A ordem é preservada dentro de cada bucket, porque a fila de uma barraca é também a ordem de atendimento dela.
- **A resposta errada clássica.** Aplicar a janela de idade a entradas de nonce, expirando em silêncio cheques que não têm data. Ramifique por `kind` antes de tocar em qualquer matemática de idade.

O starter e os testes estão no widget de desafio de código do nonce-queue-drain; a solução passa em todos os casos, e o starter falha em pelo menos um, então você sabe que os testes mordem.

![Comparação de três linhas dos buckets do classificador: submit para entradas frescas, expired para entradas de blockhash fora da janela, e unsafe para entradas de nonce gasto que nunca podem ser reenviadas.](assets/v07-comparison.png)

## Checkpoint, e a gaveta

Se o lab brigou com você, triagem nesta ordem. `fetchNonce` lançando account-not-found significa que a transação de criação do pool não confirmou; rode o `create-pool.ts` de novo e confie nas signatures impressas mais que nas suas suposições. Um `sign-sale.ts` que falha dentro do `signTransactionMessageWithSigners` enquanto offline é quase sempre um descasamento de signer: a autoridade do nonce que você nomeou na chamada de lifetime precisa ser um endereço cujo signer está embutido na mensagem, e nesta construção isso significa que o lojista é ao mesmo tempo fee payer e autoridade. E se o drain não aterrissa nada enquanto o explorer mostra as suas contas de nonce intocadas, decodifique você mesmo a instrução 0 de uma entrada enfileirada, do jeito que a guarda do passo 4 faz: se AdvanceNonceAccount não estiver sentado no índice 0, aquela entrada foi montada sem a chamada de lifetime de durable nonce, em geral porque algum caminho de código caiu em silêncio de volta para um lifetime de blockhash, e a guarda está se recusando a assinar exatamente como foi projetada.

Eu rodei a minha própria fila pelo ciclo completo enquanto escrevia isto: duas vendas assinadas com o wifi desligado, um café de atraso, as duas aterrissaram no primeiro drain, e o segundo drain conciliou as duas sem enviar um byte. A parte satisfatória não é a aterrissagem em nada; é ver o replay se recusar a enviar.

Olhe o que a barraca sobrevive agora. O módulo 3 deu a ela um QR code e precificação no servidor. A lição passada removeu a necessidade de o comprador ter SOL. Hoje removeu a necessidade de rede no momento da venda, e fez isso em cima de uma primitiva que você agora manuseia do jeito que a história dela exige: advance na instrução 0, um nonce por venda, uma gaveta tratada como papel ao portador, e um drain que prefere escalar para um humano a descontar um cheque gasto. A barraca consegue receber dinheiro sem SOL e sem sinal.

O que significa que a construção acabou e a dúvida começa. Antes de a Wavelength virar a plaquinha para aberto, falta um gate: provar que essa pilha inteira, do checkout à fila, está de fato pronta para dinheiro de verdade. A próxima lição é esse gate, um checklist de produção em que você pode ser reprovado, rodado contra tudo o que você construiu. Traga os artefatos; são eles que estão sendo examinados.
