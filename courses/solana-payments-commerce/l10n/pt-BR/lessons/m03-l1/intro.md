# Checkout com um QR: a spec do Solana Pay, ao vivo

No módulo passado você construiu o transfer-kit: ele detecta se um mint pertence ao Token clássico ou ao Token-2022 e consegue mover qualquer uma das principais stablecoins com um memo e uma reference anexados. Isso é uma capacidade de verdade, e tem um buraco que dá para sentir. Nada nele pede a um cliente que pague. Ele só envia. Você consegue pagar qualquer um a partir do código; um cliente não consegue te entregar dinheiro rodando o seu TypeScript.

O que um cliente consegue fazer é apontar um celular para um quadrado de pixels. Hoje você constrói esse quadrado, e o servidor que percebe quando alguém paga ele. No fim desta lição uma venda de devnet aterrissa na sua máquina: QR escaneado, transferência liquidada, pedido casado, valor confirmado.

Comece a instalação agora para ela terminar enquanto você lê. Este é um workspace novo ao lado do transfer-kit, não dentro dele, por um motivo que a seção de teoria vai tornar concreto. Rode o `mkdir` a partir da raiz `wavelength` para a pasta nova aterrissar do lado de `transfer-kit`:

```bash
mkdir wavelength-checkout && cd wavelength-checkout
npm init -y
npm pkg set type=module
npm i @solana/pay@1.0.26 @solana/kit@6.10.0
npm i -D tsx esbuild
```

Os pins, com a nota de frescor deles: `@solana/pay` 1.0.26 é o `latest` do npm (publicado em 2026-07-31, re-checado em 2026-08-22) e ele faz peer com `@solana/kit ^6.9.0`, o que te mantém na mesma linha kit v6 que o curso usa desde o módulo 2; 6.10.0 é o último release dessa linha. Um piso rígido para checar antes de você depurar qualquer outra coisa: o pay 1.0.26 declara `engines.node >= 20`, porque precisa do suporte a Ed25519 do Node no `crypto.subtle` — o Node 24+ que este curso assume desde o módulo 1 passa com folga. O `tsx` roda arquivos TypeScript direto, o `esbuild` empacota um arquivo para o navegador mais para frente. O npm 7+ instala automaticamente o resto das peer dependencies do pay para você.

Aquela linha `npm pkg set type=module` não é decoração. O `npm init -y` escreve um manifesto CommonJS, e todo arquivo no workspace é um módulo ES: o servidor e o smoke test usam os dois `import.meta.url`, que é um erro de sintaxe sob CommonJS, então sem essa única linha o lab morre na primeira execução com um erro que não fala nada sobre módulos. Todo workspace que este curso criar daqui para frente seta ela, e o bloco de scaffold de cada lição inclui a linha em vez de assumir que você lembra.

Enquanto isso roda, a versão em uma frase de onde você está na escada do módulo: esta lição é o checkout mais simples possível, uma URL que a carteira do cliente transforma em uma transação, e os limites dela são exatamente o que a próxima lição conserta.

## Resumo

O que esta lição estabelece, uma linha acionável para cada:

- Um transfer request do Solana Pay é uma URL: o esquema `solana:`, um endereço de destinatário e parâmetros de query para `amount`, `spl-token`, `reference`, `label`, `message` e `memo`. A carteira lê ela e constrói a transferência sozinha.
- `amount` é em unidades decimais do token. `12.5` quer dizer 12.5 USDC. Escrever unidades base (`12500000`) cobra 12.5 milhões de USDC do cliente, e este é o bug de checkout do Solana Pay mais comum que existe.
- A reference é uma chave base58 aleatória e nova de 32 bytes que você gera por checkout, antes de qualquer pagamento existir. Ela é a chave de junção entre o seu livro-razão de pedidos e a blockchain: você não consegue saber a signature de antemão, mas você escolheu a reference.
- `encodeURL` constrói a URL, `createQR` transforma ela em pixels (só no navegador, ele precisa de um DOM), `watchReference` abre uma inscrição de WebSocket que resolve quando uma transação mencionando a sua reference aterrissa, e `validateTransfer` então confirma que a transferência pagou o valor certo do mint certo ao destinatário certo.
- A biblioteca de checkout clássica mora em `typescript/packages/solana-pay/` dentro do repo pay. O destaque do repo migrou para um CLI de agentic payments; o subpacote é a coisa que você `npm i`.
- A página da spec está mais ou menos congelada em 2022. Ela ainda cita Phantom, FTX e Slope. A spec em si continua sendo o padrão vivo; a página é que é vintage. Message signing é uma extensão alpha, não parte do v1.
- A Shopify anunciou o Solana Pay em 2023-08-23; o caminho ativo da Shopify hoje é o plugin do MoonPay Commerce.

Como o trabalho está dividido, dito em voz alta: o lab é uma construção Worked com todo arquivo entregue, exceto por dois buracos deliberados. A geração da reference key e as expectativas do `validateTransfer` saem como TODOs, e preencher eles é o degrau de Completion do Challenge, com as respostas à vista na teoria abaixo. O degrau de Solo acrescenta um segundo disco e prova que as duas vendas são distinguíveis. Você está recebendo menos do que no módulo passado, de propósito.

## Uma URL que uma carteira consegue pagar

### O transfer request, campo a campo

Aqui está uma URL de transfer request completa, a string exata que a minha própria execução do código desta lição produziu:

```
solana:4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM?amount=12.5&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=CD9GnkB9qKo3XWX2UkDMNBc9YBTFe29ofLtASYbprvDh&label=Wavelength+Records&message=Slow+Tides%2C+Wavelength+pressing+041&memo=LP-041
```

Nenhum servidor participou de tornar aquilo pagável. O esquema `solana:` diz para a carteira que isto é uma URL de pagamento. O endereço logo depois dele é o destinatário, a sua carteira de lojista. Depois os parâmetros de query:

- `amount=12.5`: quanto, em unidades decimais do token.
- `spl-token=4zMM...ncDU`: qual token, por endereço de mint. Aquele ali é o USDC de devnet, o mint que o transfer-kit vem enviando desde o módulo 2. Omita este parâmetro e a carteira lê o valor como SOL nativo.
- `reference=CD9G...rvDh`: uma chave de conta extra que a carteira anexa à transferência, para você conseguir encontrar ela depois. Mais sobre isso daqui a pouco, é a ideia que sustenta a lição inteira.
- `label`, `message`: o que a carteira mostra ao cliente na hora da aprovação. O nome da sua loja, o item.
- `memo=LP-041`: escrito na própria transação via memo program, então o SKU viaja on-chain junto com o pagamento.

Um par dessa lista merece ser desembaraçado agora, porque os nomes deles convidam à confusão. `message` e `memo` soam como sinônimos e se comportam de formas completamente diferentes. `message` é só de exibição: a carteira mostra ele ao cliente na hora da aprovação, e ele evapora. Nunca toca na blockchain. `memo` é o oposto: a carteira escreve ele na transação através do memo program, então ele liquida on-chain, permanentemente, ao lado do pagamento, onde o seu back office (e qualquer outra pessoa, isto é um livro-razão público) consegue ler para sempre. A regra de bolso: `message` é para o humano que aprova, `memo` é para os sistemas que conciliam. Ponha o título do disco em `message` e o SKU em `memo`, nunca o contrário, e nunca ponha em `memo` nada que você não imprimiria num recibo público.

O cliente escaneia, a carteira dele interpreta esses parâmetros, constrói uma instrução `TransferChecked` a partir deles e pede uma digital. Você nunca vê uma chave privada, você nunca constrói uma transação. A URL é o protocolo inteiro entre a sua loja e a carteira dele.

Agora a cilada, porque ela merece o próprio parágrafo e um lado a lado. Você passou o módulo 2 inteiro convertendo valores decimais para unidades base com `toBaseUnits`, porque instruções de transferência on-chain falam unidades base. Uma URL do Solana Pay não fala. A spec define `amount` como uma quantidade de UI, e a carteira multiplica pelas casas decimais do mint para você. Os dois hábitos colidem de frente:

![Lado a lado de 12.5 USDC como valor decimal na URL versus 12500000 unidades base em uma instrução, avisando que unidades base em uma URL cobram milhões.](assets/v01-comparison.png)

As duas convenções estão corretas onde moram. A URL fala humano, a instrução fala unidades base, e a carteira é a tradutora. Mantenha o `toBaseUnits` completamente fora do seu código de URL.

### A reference key: escolhida antes de o pagamento existir

Aqui está o problema que a reference resolve. A signature de um pagamento é o identificador único óbvio para uma venda, e você não consegue usar ela, porque ela não existe até o cliente pagar. Você precisa de um identificador que você controla antes de a transação nascer, algo que você consiga escrever no seu livro-razão de pedidos na hora do checkout e depois usar para reconhecer o pagamento quando ele aterrissar.

Esse identificador é a reference: uma chave base58 aleatória e nova de 32 bytes que você gera por checkout. A carteira anexa ela à transferência como uma chave de conta extra. Ela não assina nada, não guarda nada, nunca é uma carteira. Ela existe para que a transação mencione ela, porque a camada de RPC da Solana consegue buscar transações por qualquer conta que elas mencionem. Você cunha uma chave que ninguém nunca viu, embute ela no QR, e a única transação no mundo que carrega ela é a sua venda.

Vale ser preciso sobre a mecânica, já que ela explica tanto por que isso funciona quanto por que não custa nada. A carteira acrescenta a sua reference à lista de contas da instrução de transferência como uma chave não signatária e não gravável. A conta por trás daquele endereço não existe e nunca vai existir; sem aluguel, sem criação, sem estado. É pichação pura na lista de contas da transação. Mas a Solana indexa transações por toda conta que elas mencionam, existente ou não, que é o que o `getSignaturesForAddress` consulta por baixo do capô. Pergunte ao RPC "quais transações mencionam `CD9G...rvDh`?" e a resposta é a sua venda e mais nada na história da blockchain. Um índice de transações gratuito, à prova de colisão e pré-atribuível, construído a partir de um endereço que ninguém financiou. Compare isso com a gambiarra tradicional, um endereço de depósito único por pedido com toda a gestão de chaves que isso arrasta junto, e a reference começa a parecer a ideia melhor que ela é.

![Fluxo de dados de uma reference key cunhada pelo servidor viajando pelo QR e pela carteira até a transação, e depois consultada de volta via getSignaturesForAddress para casar o pedido.](assets/v02-diagram.png)

Se você já integrou o Stripe, você já encontrou este formato antes: é a sua chave de idempotência e o seu id de pedido fundidos em um valor só, escolhido do lado do cliente antes da cobrança. Pense nela como uma ficha de guarda-volumes. A ficha é impressa antes de o casaco chegar, o número corresponde a exatamente um casaco, e segurar a ficha é como você reivindica ele depois. Mesma disciplina aqui: um checkout, uma reference, nunca reutilizada. Reutilize uma e duas vendas diferentes viram indistinguíveis, que é precisamente a falha que o desafio de Solo te faz provar que você evitou.

Gerar uma leva uma única linha com o kit, e sim, é um keypair completo do qual a gente joga a metade privada fora na hora. Só o endereço importa:

```typescript
import { generateKeyPairSigner } from '@solana/kit';

const reference = (await generateKeyPairSigner()).address;
```

Esta linha é um dos dois buracos TODO do lab. Você acabou de ver a resposta.

![A URL do transfer request dividida em partes rotuladas: esquema solana, destinatário, valor decimal, mint spl-token, reference key, label e message, e memo on-chain.](assets/v03-diagram.png)

### Onde a biblioteca de fato mora, e quão velha é a página da spec

Dois avisos honestos antes de você ler qualquer material oficial, os dois capazes de, do contrário, te custar uma noite.

Primeiro, o repo. O repositório canônico do Solana Pay é `solana-foundation/pay` (a URL antiga `solana-labs/solana-pay` redireciona para lá). Abra o README dele e você não vai encontrar a sua biblioteca de checkout. O produto de destaque agora é um CLI para agentic payments, fluxos HTTP máquina a máquina, e instalar `@solana/pay` globalmente até te entrega o binário desse CLI. A biblioteca de checkout clássica que você acabou de instalar mora em um subpacote: `typescript/packages/solana-pay/`. Ela não está descontinuada, não está congelada, e está bem publicada: a 1.0.26 saiu em 2026-07-31, reconstruída em cima do kit. A energia de pagamentos da Foundation se moveu para outra porta de entrada; a biblioteca continuou na casa. Salve nos favoritos o caminho do subpacote, não a raiz do repo.

![Árvore do repo solana-foundation/pay mostrando o README da raiz como destaque do CLI agentic e a biblioteca de checkout clássica morando em typescript/packages/solana-pay com os cinco exports principais dela.](assets/v04-diagram.png)

Segundo, a spec. A spec do Solana Pay em docs.solanapay.com continua sendo o padrão que toda carteira implementa, e a página em si está congelada em algum ponto da era 2022-2023: a linha de copyright dela diz 2023, o elenco dela é puro 2022. A linha de abertura dela, literalmente, é "Rough consensus on this spec has been reached, and implementations exist in Phantom, FTX, and Slope." Um desses três colapsou espetacularmente e outro sumiu. Leia a spec pelo protocolo, que envelheceu bem, e ignore o elenco, que não envelheceu. O repo carrega o mesmo texto em `typescript/packages/solana-pay/spec/SPEC.md`, ao lado de dois irmãos que vale conhecer pelo nome: `SPEC1.1.md` e `message-signing-spec.md`, que abrem os dois com a linha "This spec is currently alpha and subject to change." Message signing é essa extensão alpha, não parte do padrão v1 vivo de transfer/transaction request, e nenhum checkout deste curso se apoia nela.

Por que essa idade importa além da curiosidade? Porque em algum momento você vai colar de tutoriais velhos, todo mundo cola, e o ecossistema em volta desta spec tem três eras distintas: os originais de 2022 (tipos do web3.js 1.x, valores em `BigNumber`), o longo meio 0.2.x (só polling), e a linha 1.x que você instalou (tipos do kit, valores em `number` puro, um watcher baseado em push). Código da era errada vai te dar erro de tipo de formas confusas. Na dúvida, confie nos arquivos `.d.ts` do seu próprio `node_modules` mais do que em qualquer post de blog, inclusive o eu-futuro deste aqui.

### Da URL aos pixels: encodeURL e createQR

`encodeURL` é uma função pura: campos entram, `URL` sai. Sem rede, sem RPC, nada assíncrono:

```typescript
import { encodeURL } from '@solana/pay';
import { address } from '@solana/kit';

const url = encodeURL({
  recipient: address('4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM'),
  amount: 12.5,
  splToken: address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU'),
  reference,
  label: 'Wavelength Records',
  message: 'Slow Tides, Wavelength pressing 041',
  memo: 'LP-041',
});
```

Todo campo mapeia um para um nas partes da URL que você acabou de ler. `recipient`, `splToken` e `reference` são valores `Address` do kit, que é por isso que `address(...)` envolve as strings: ele valida o base58 no nível de tipo em vez de na hora do escaneamento.

`createQR(url, size, background, color)` transforma a URL em um objeto QR estilizado. Um fato estrutural decide a sua arquitetura: ele precisa de um DOM. Chame ele no Node e ele lança `document is not defined`; eu chamei, ele lança. Então o QR pertence a um arquivo de navegador, e a construção da URL pertence ao seu servidor, e essa divisão é a fronteira de confiança correta de qualquer jeito. O servidor decide o que está à venda e a que preço; o navegador só transforma uma URL pronta em pixels. Para testes do lado do servidor existe o `createQROptions`, que constrói a configuração do QR sem tocar no DOM. O smoke test do lab nem chega tão longe: já que o QR é desenhado no navegador, tudo que uma checagem do lado do Node consegue afirmar honestamente é que o bundle do navegador foi construído, que é exatamente o que ela afirma.

A biblioteca também entrega o espelho do `encodeURL`: o `parseURL`. Entregue uma URL a ele e ele te devolve os campos tipados, destinatário e valor e todo o resto, ou lança se a string estiver malformada. Isto é literalmente a metade do protocolo que cabe à carteira; quando um celular escaneia o seu QR, algo com exatamente o formato do `parseURL` roda do outro lado do vidro. O que faz dele uma ferramenta de teste discretamente perfeita para o seu lado: se a sua URL recém-codificada sobrevive a uma ida e volta pelo parser da própria biblioteca com o valor intacto, uma carteira compatível com a spec vai ler ela do jeito que você quis dizer. Sem celular nenhum. O smoke test se apoia nisso, e é um hábito que vale roubar para qualquer trabalho com protocolo: quando uma biblioteca entrega as duas direções de um codec, a ida e volta é a checagem de corretude mais barata que você vai escrever na vida.

### Ouvir o pagamento aterrissar: watchReference

A sua página agora mostra um QR. Em algum lugar por aí um cliente aprova o pagamento no celular dele. O seu servidor precisa ficar sabendo. Não a aba do navegador, o seu servidor: um callback de "pagamento bem-sucedido" no frontend é falsificável por qualquer um com o devtools aberto, e construir o reflexo de lojista de nunca confiar nele é metade do que o módulo 4 trata.

A biblioteca te dá duas ferramentas de detecção, um formato velho e um novo.

`findReference(rpc, reference)` pergunta ao RPC por HTTP: alguma transação mencionou esta reference key? Ele devolve a signature correspondente mais antiga e lança `FindReferenceError` quando nada aterrissou ainda. É uma pergunta de um tiro só, então usar ele sozinho quer dizer perguntar de novo e de novo num timer. Esse loop de polling era a história inteira da linha legada 0.2.x, e o app de ponto de venda oficial ainda funciona assim.

`watchReference(rpcSubscriptions, reference, options)` é o caminho 1.x e, honestamente, uma mão na roda para quem já escreveu a versão com polling. Ele abre uma inscrição de WebSocket (`logsNotifications`, filtrada para transações que mencionam a sua reference) e devolve uma promise que resolve no momento em que uma transação correspondente aterrissa, com a signature e o status de erro on-chain dela. Sem timer, sem aritmética de retry, sem requisições desperdiçadas. Você arma ele quando o checkout abre e dá await:

```typescript
import { watchReference } from '@solana/pay';
import { createSolanaRpcSubscriptions } from '@solana/kit';

const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

const { signature, err } = await watchReference(rpcSubscriptions, reference, {
  commitment: 'confirmed',
  abortSignal: controller.signal,
});
```

O `abortSignal` importa em código de verdade: um checkout do qual o cliente vai embora não deveria segurar uma inscrição para sempre. Aborte ele e a promise rejeita com `FindReferenceError`, que o seu código trata como "venda expirada", não como um crash. Na minha própria execução contra a devnet, armar o watcher em uma reference nova e abortar três segundos depois fez exatamente isso, de forma limpa.

A opção `commitment` é uma decisão de política para a qual você já tem o vocabulário. O módulo 1 construiu a escada: `confirmed` quer dizer que uma supermaioria do cluster votou no bloco da transação, `finalized` quer dizer que a blockchain construiu além dele o suficiente para rollback sair da mesa. Para um disco de 12.5 USDC, `confirmed` é a decisão certa de lojista, o mesmo julgamento em que a tabela de política do módulo 1 aterrissa para mercadoria de ticket baixo: o risco residual é minúsculo e o cliente está ali parado esperando. Vendendo algo que você não consegue reaver a um preço que doeria? Observe em `confirmed` para o recibo rápido, libere a mercadoria em `finalized`. O watcher aceita qualquer um dos dois; o ponto é que o parâmetro é uma decisão de negócio vestindo um nome técnico, e deixar ele no padrão sem decidir ainda é uma decisão.

Inscrição de WebSocket, em uma frase para quem só conhece polling de APIs REST: em vez de você perguntar repetidamente "já tem alguma coisa?", você mantém uma conexão longeva aberta e o nó RPC te empurra a resposta no momento em que ela existe. Mantenha o `findReference` na sua caixa de ferramentas mesmo assim. Um WebSocket que cai durante o pagamento perde a notificação, e um poll é como você varre atrás de qualquer coisa que uma inscrição tenha perdido. Os checkouts de produção do módulo 4 rodam os dois: inscreva-se pela velocidade, varra pela verdade.

![Comparação do findReference como um loop repetido de polling HTTP versus o watchReference como uma única inscrição de WebSocket que empurra a signature quando a transferência aterrissa.](assets/v05-comparison.png)

### A confiança chega por último: validateTransfer

O watcher resolver não quer dizer que você recebeu. Quer dizer que uma transação mencionando a sua reference aterrissou. Essas são afirmações diferentes. Qualquer um consegue mandar uma transação que menciona a sua reference key: um pagamento do valor errado, do token errado, para o destinatário errado, ou uma transferência que falhou on-chain mas ainda carrega a conta.

`validateTransfer` fecha a lacuna. Você entrega a ele a signature que o watcher pegou mais uma declaração do que esta venda deveria ser, e ele busca a transação liquidada e confere a história contra a blockchain:

```typescript
import { validateTransfer } from '@solana/pay';
import { createSolanaRpc } from '@solana/kit';

const rpc = createSolanaRpc('https://api.devnet.solana.com');

await validateTransfer(
  rpc,
  signature,
  {
    recipient: MERCHANT,      // the money went to you
    amount: 12.5,             // decimal units, same convention as the URL
    splToken: USDC_DEVNET,    // it was actually USDC, not a lookalike mint
    reference,                // and it is THIS sale, not another one
  },
  { commitment: 'confirmed' },
);
```

Se qualquer expectativa falhar, ele lança `ValidateTransferError` e você não fez uma venda, seja lá o que o navegador estiver mostrando. O objeto de expectativas é o segundo buraco TODO do lab, e agora você viu esta resposta também. O modelo mental para levar para o módulo 4: o watcher é a sua campainha, o `validateTransfer` é checar se o dinheiro é real antes de entregar o disco. Uma campainha não é um pagamento.

Juntando tudo, uma venda flui assim:

![Fluxo de venda em quatro faixas onde o servidor cunha uma reference e arma um watcher, o navegador renderiza o QR, a carteira submete a transferência, e o validateTransfer confirma ela.](assets/v06-flowchart.png)

### Quem já rodou este formato em escala

Este padrão de URL-mais-reference não é brinquedo de sala de aula. Em 2023-08-23, a Shopify anunciou o Solana Pay como opção de pagamento em toda a rede de lojistas dela, com MonkeDAO, Mad Lads e Helius entre os primeiros usuários. O discurso que o líder de integração da Shopify fez era economia de lojista pura, a mesma aritmética do módulo 1: sem taxas bancárias, sem chargebacks, sem prazos de retenção de vários dias em cima da sua própria receita. Uma venda no cartão é um empréstimo que a bandeira consegue reaver por meses; uma transferência de stablecoin liquidada é final em segundos, e para um lojista rodando com margens apertadas essa diferença é o argumento inteiro. O caminho de integração já mudou de mãos desde então, como encanamento de comércio tende a mudar: a rota ativa hoje para uma loja Shopify é o plugin do MoonPay Commerce. A spec por baixo é a que você está implementando agora mesmo.

![Linha do tempo da spec do Solana Pay de 2022, passando pelo anúncio da Shopify em 2023 com os três primeiros usuários nomeados, até a biblioteca baseada em kit de 2026 e o caminho do MoonPay Commerce.](assets/v07-timeline.png)

Agora o trade-off, porque o degrau desta lição tem um teto afiado e você deveria sentir ele antes de construir. Tudo que a carteira sabe sobre esta venda veio de uma URL que você imprimiu numa tela, e uma vez que ela está do lado do cliente do vidro você não controla nada disso. O valor, o mint, a reference: tudo isso é dado nas mãos do cliente antes de virar uma transação. `validateTransfer` quer dizer que adulteração não consegue te enganar, um pagamento adulterado simplesmente falha na validação. Mas também não consegue te expressar. Sem totais de carrinho calculados no servidor, sem lógica de cupom, sem memo dinâmico, sem estado de pedido nenhum além de uma reference key por carregamento de página. Um transfer request é simplíssimo e trustless, e é exatamente um disco a um preço. Esse teto é o problema de abertura da próxima lição.

## Lab: venda um disco na devnet

Construção Worked. Todo arquivo abaixo vai no workspace `wavelength-checkout` que você criou lá em cima. Dois arquivos saem com buracos TODO, marcados aos berros; a construção roda até uma falha específica e nomeada com eles no lugar, e o Challenge fecha eles.

1. **O disco.** Crie `checkout/record.ts`. Um disco, um preço, mais os dois endereços que todo o resto importa:

   ```typescript
   import { address } from '@solana/kit';

   // Your devnet merchant wallet. Paste the address you funded in module 2.
   export const MERCHANT = address('4vbaMR793oqgJHmJjwiFQsmcbd1ffQSeiTeKdvQTWgHM');

   // Devnet USDC, the same mint transfer-kit has been sending since module 2.
   export const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');

   export const RECORD = {
     sku: 'LP-041',
     title: 'Slow Tides, Wavelength pressing 041',
     priceUsdc: 12.5, // decimal token units. NOT base units. Never 12500000.
   };
   ```

   Substitua `MERCHANT` pelo seu próprio endereço de devnet ou a venda muito definitivamente não vai chegar em você.

2. **A URL de pagamento, com o buraco número um.** Crie `checkout/payment.ts`:

   ```typescript
   import { encodeURL } from '@solana/pay';
   import { generateKeyPairSigner, type Address } from '@solana/kit';
   import { MERCHANT, USDC_DEVNET, RECORD } from './record.ts';

   // One fresh reference per checkout: a random 32-byte base58 key chosen
   // BEFORE the payment exists. The join key between this sale and the chain.
   export async function newReference(): Promise<Address> {
     // TODO(completion): mint a fresh key and return its address.
     // One line. The theory section shows it, generateKeyPairSigner is
     // already imported, and the smoke test will fail here until you do.
     throw new Error('TODO: newReference');
   }

   export function buildPaymentURL(reference: Address): URL {
     return encodeURL({
       recipient: MERCHANT,
       amount: RECORD.priceUsdc,
       splToken: USDC_DEVNET,
       reference,
       label: 'Wavelength Records',
       message: RECORD.title,
       memo: RECORD.sku,
     });
   }
   ```

3. **O lado do navegador.** Crie `checkout/page.ts`. Este é o único arquivo que toca em `createQR`, pelo motivo do DOM que veio da teoria:

   ```typescript
   import { createQR } from '@solana/pay';

   // The server stamped the encoded URL onto the mount node; all this file
   // does is turn it into pixels. createQR needs a DOM, which is why it
   // lives here and not in server.ts.
   const mount = document.getElementById('qr')!;
   const qr = createQR(mount.dataset.url!, 512, 'white', 'black');
   qr.append(mount);
   ```

   Empacote para o navegador:

   ```bash
   npx esbuild checkout/page.ts --bundle --format=esm --outfile=checkout/public/page.js
   ```

   Checkpoint: o esbuild reporta o arquivo de saída. O meu saiu com 94.7kb, a biblioteca de estilização do QR é a maior parte disso.

4. **O watcher, com o buraco número dois.** Crie `checkout/watcher.ts`:

   ```typescript
   import { watchReference, validateTransfer } from '@solana/pay';
   import {
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     type Address,
     type Signature,
   } from '@solana/kit';
   import { MERCHANT, USDC_DEVNET, RECORD } from './record.ts';

   const rpc = createSolanaRpc('https://api.devnet.solana.com');
   const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

   // Resolves with the signature once a transfer carrying `reference` lands
   // on devnet AND survives validation against what this sale should be.
   export async function awaitSale(
     reference: Address,
     abortSignal?: AbortSignal,
   ): Promise<Signature> {
     const { signature, err } = await watchReference(rpcSubscriptions, reference, {
       commitment: 'confirmed',
       abortSignal,
     });
     if (err) {
       throw new Error(`transfer ${signature} landed but failed on-chain`);
     }

     const expected = {
       recipient: MERCHANT,
       amount: 0, // TODO(completion): the record's real price, decimal units
       // TODO(completion): add splToken and reference. Without the mint
       // check, 12.5 of any token passes. Without the reference, any sale
       // passes as this one.
     };
     await validateTransfer(rpc, signature, expected, { commitment: 'confirmed' });
     return signature;
   }
   ```

   Como ele sai, este watcher vai pegar um pagamento e depois rejeitar ele, porque nenhuma transferência real valida contra um valor esperado de zero. Isso é deliberado. A campainha funciona; a checagem do dinheiro é sua para escrever.

5. **O servidor.** Crie `checkout/server.ts`. `node:http` puro, nada mais; todo GET da página é um checkout, então cada visita cunha uma reference, codifica uma URL e arma um watcher antes de o HTML sequer sair do socket:

   ```typescript
   import { createServer } from 'node:http';
   import { readFileSync } from 'node:fs';
   import { newReference, buildPaymentURL } from './payment.ts';
   import { awaitSale } from './watcher.ts';
   import { RECORD } from './record.ts';

   const pageJs = readFileSync(new URL('./public/page.js', import.meta.url), 'utf8');

   const server = createServer(async (req, res) => {
     if (req.url === '/page.js') {
       res.writeHead(200, { 'content-type': 'text/javascript' });
       res.end(pageJs);
       return;
     }

     // Every GET / is one checkout: fresh reference, fresh QR, fresh watcher.
     const reference = await newReference();
     const url = buildPaymentURL(reference);

     awaitSale(reference)
       .then((signature) => {
         console.log(`SOLD ${RECORD.sku} ref=${reference} sig=${signature}`);
       })
       .catch((err) => {
         console.error(`checkout ${reference} did not validate:`, err.message);
       });
     console.log(`checkout open ref=${reference}`);

     res.writeHead(200, { 'content-type': 'text/html' });
     res.end(`<!doctype html>
   <title>Wavelength Records</title>
   <h1>${RECORD.title}</h1>
   <p>${RECORD.priceUsdc} USDC (devnet)</p>
   <div id="qr" data-url="${url.toString()}"></div>
   <script type="module" src="/page.js"></script>`);
   });

   server.listen(3010, () => {
     console.log('Wavelength checkout on http://localhost:3010');
   });
   ```

   Repare no que o servidor nunca faz: ele nunca diz ao navegador se o pagamento aconteceu. O recibo é uma linha de log do lado do servidor. Ligar o status da venda de volta na página é trabalho de verdade com decisões de confiança de verdade, e pertence ao módulo de back office.

   Repare também no que esta versão de brinquedo vaza, porque enxergar o vazamento agora te poupa uma sessão de depuração no módulo 4. Todo GET arma um watcher sem sinal de abort e sem expiração. Atualize a página cinco vezes e você tem cinco inscrições de WebSocket vivas, quatro delas órfãs que vão ficar sentadas na conexão RPC até o processo morrer. Tudo bem para um lab na devnet, um problema de verdade com qualquer tráfego. Um checkout de produção tem um ciclo de vida: ele abre, expira depois de alguns minutos, o watcher dele é abortado, e uma varredura periódica com `findReference` pega qualquer coisa que pagou depois de a inscrição fechar. Você já construiu a maquinaria de abort (`awaitSale` aceita um sinal, o smoke test exercita ele); este servidor só não usa ela ainda. O módulo de back office dá esse ciclo de vida aos checkouts direito, junto com a persistência que esta linha de log está substituindo.

![Handler de requisição anotado mostrando a sequência por checkout: cunhar uma reference nova, codificar a URL de pagamento, armar o watcher, e então servir a página.](assets/v08-annotated-code.png)

6. **O smoke test.** Crie `checkout/smoke.ts`, o verify padrão por lição do módulo:

   ```typescript
   import { parseURL } from '@solana/pay';
   import { newReference, buildPaymentURL } from './payment.ts';
   import { awaitSale } from './watcher.ts';
   import { RECORD } from './record.ts';
   import { existsSync } from 'node:fs';

   const main = async () => {
     // 1. The URL round-trips through the library's own parser.
     const reference = await newReference();
     const url = buildPaymentURL(reference);
     const parsed = parseURL(url);
     if (!('recipient' in parsed) || parsed.amount !== RECORD.priceUsdc) {
       throw new Error(`URL did not round-trip: ${url}`);
     }
     console.log('transfer-request URL valid');

     // 2. The browser bundle that draws the QR exists. We cannot
     //    render a QR in Node, so this is the honest check.
     if (!existsSync(new URL('./public/page.js', import.meta.url))) {
       throw new Error('checkout/public/page.js missing: run the esbuild step');
     }
     console.log('QR bundle built');

     // 3. The watcher arms against devnet and cancels cleanly.
     const controller = new AbortController();
     const armed = awaitSale(reference, controller.signal).catch(() => 'aborted');
     await new Promise((r) => setTimeout(r, 2000));
     controller.abort();
     if ((await armed) !== 'aborted') {
       throw new Error('watcher resolved without a payment?');
     }
     console.log('reference watcher armed');
     process.exit(0);
   };
   main();
   ```

7. **Rode até a falha projetada.**

   ```bash
   npx tsx checkout/smoke.ts
   ```

   Checkpoint: ele morre na hora com `TODO: newReference`. Esse erro exato é a linha de chegada deste lab. O scaffold está totalmente ligado, a página empacota, o código do servidor está completo, e os dois buracos entre você e uma loja funcionando são os dois conceitos que esta lição existe para ensinar. Fechar eles é o Challenge.

## Challenge

Três degraus, e o do meio é onde a loja ganha vida.

**Worked.** Feito: o lab acima era ele, terminando na falha TODO nomeada. Se o seu smoke test falhar com qualquer coisa que não seja `TODO: newReference`, conserte isso primeiro; os dois suspeitos de sempre são um `checkout/public/page.js` faltando (rode o passo do esbuild de novo) e um erro de digitação num literal de endereço, que `address(...)` rejeita na hora do import com um erro de base58.

**Completion.** Preencha os dois buracos. Em `payment.ts`, faça o `newReference` cunhar e devolver um endereço novo; em `watcher.ts`, substitua o objeto `expected` pela história verdadeira da venda: `amount` real, mais `splToken` e `reference`. As duas respostas aparecem literalmente na seção de teoria. Aceitação, em dois estágios. Primeiro, `npx tsx checkout/smoke.ts` imprime exatamente três linhas: `transfer-request URL valid`, `QR bundle built`, `reference watcher armed`.

Segundo, a venda em si, e ela precisa de um cliente que não seja você. O `validateTransfer` mede a *mudança* de saldo na conta de token do destinatário, então uma carteira pagando a si mesma não move nada e a venda é rejeitada com `amount not transferred` por mais correto que o seu código esteja. O seu keypair de lojista é o destinatário, então ele não pode ser também o comprador. Abasteça uma segunda carteira antes de subir o servidor, a partir da raiz `wavelength`:

```bash
CUSTOMER=$(solana-keygen pubkey /tmp/customer.json)   # the pretend customer from module 2 lesson 1
solana transfer "$CUSTOMER" 0.05 --allow-unfunded-recipient --url devnet
npm run --workspace transfer-kit pay -- "$CUSTOMER" 13
```

A primeira linha dá ao cliente SOL para pagar taxas de transação; a segunda move 13 USDC de devnet para fora do seu saldo de lojista, o suficiente para uma venda de 12.5 com troco. Financiar uma carteira na mesma moeda em que ela está prestes a te pagar de volta parece circular, e é: na devnet você é os dois lados do balcão e o dinheiro volta para casa no fim da execução. Se o seu saldo de lojista não cobrir 13, o faucet da Circle permite mais um pingo depois que o cooldown dele passar.

Agora rode `npx tsx checkout/server.ts`, abra `http://localhost:3010`, anote a linha `checkout open ref=...` que ele imprime, e pague aquele QR na devnet por uma de duas rotas.

*Celular.* Escaneie com uma carteira de celular trocada para devnet — a carteira que você criou no módulo 1, abastecida exatamente do mesmo jeito, substituindo `$CUSTOMER` nos dois comandos acima pelo endereço que o app da carteira te mostra. A maioria das carteiras esconde a troca para devnet atrás de um toggle de configurações de desenvolvedor; se a sua não trocar de rede de jeito nenhum, pule o celular, o caminho por CLI faz o mesmo gate.

*CLI.* Deixe o seu próprio ferramental fazer o papel do cliente. O script `pay` do transfer-kit ganhou exatamente os dois botões que isto precisa na lição passada: `PAYER_KEYFILE` escolhe a carteira que assina, e um quarto argumento aceita uma reference que outra pessoa cunhou. A partir da raiz `wavelength`:

```bash
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5 <ref-from-the-server-log>
```

`$(solana address)` é o endereço de lojista que você colou em `record.ts`, `12.5` é a string decimal que o kit converte para unidades base para você (a URL e o kit concordam em unidades decimais aqui; só a instrução por baixo fala unidades base), e a reference é o que deixa o watcher que o seu servidor já armou reconhecer esta transferência como a venda dele.

De qualquer jeito, a aceitação é uma linha de log do servidor: `SOLD LP-041 ref=<your reference> sig=<signature>`. Aquela signature é uma transação de devnet real; procure ela em qualquer explorer e encontre o seu memo `LP-041` sentado on-chain.

**Solo.** A Wavelength passa a estocar um segundo disco. Acrescente ele em `record.ts` a um preço diferente, sirva ele em `/lp-042` com o checkout próprio dele, e venda os dois. Complete a carteira do cliente primeiro com a mesma linha `pay` — o troco dela da primeira venda não vai cobrir uma segunda — e depois registre as duas. Aceitação: duas linhas de log `SOLD` com duas references diferentes, cada uma validada contra o próprio preço, e você consegue dizer em voz alta qual reference pertence a qual disco sem ler os valores. Se você reutilizou uma reference nas duas, você já sabe qual frase da teoria você pulou. As decisões de projeto são suas: funções de watcher por disco, um mapa de discos, o que quer que segure dois SKUs com honestidade.

Se a validação continuar rejeitando um pagamento que você tem certeza que enviou, leia a mensagem do `ValidateTransferError` antes de tocar em código, ela nomeia a expectativa que falhou. `amount not transferred` numa transferência que você consegue ver num explorer quer dizer que o pagador e o destinatário eram a mesma carteira: cheque se `PAYER_KEYFILE` está de fato no ambiente do comando que você rodou, porque sem ele o script assina como o lojista e o delta é zero. Uma *divergência* genuína de valor costuma ser o bug das unidades decimais aparecendo num lugar novo: cheque se alguma coisa no seu caminho de envio multiplicou por um milhão uma vez a mais.

Um pedido antes de você fechar o terminal: se algum passo brigou com você, anote qual. As lições restantes deste módulo assumem que este scaffold entrou limpo, e a lista de atritos dos leitores é como o curso fica mais afiado. Onde foi liso, aceite a vitória; você levantou um trilho de pagamento a partir de uma spec de URL e cinco funções, e a maioria das pessoas que integra checkouts de cartão nunca viu uma única vez o próprio dinheiro confirmar.

A sua loja agora vende um disco a um preço hardcoded, e todo fato sobre a venda ainda viaja por uma URL que a carteira do cliente controla. Um carrinho de verdade tem um total calculado a partir de itens de linha, um cupom que muda ele, um id de pedido do seu banco de dados, nada disso pertence a uma string que o cliente consegue editar. A próxima lição, "Transaction requests: o servidor constrói a transação", inverte a direção da confiança: a carteira para de construir a transferência a partir da sua URL e começa a pedir ao seu servidor uma transação que você construiu. O watcher que você acabou de escrever vem junto, sem mudanças.
