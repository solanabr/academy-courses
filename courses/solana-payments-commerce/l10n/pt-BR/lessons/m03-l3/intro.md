# A feira de discos: maquininha (POS) e a realidade do hardware mobile

Na lição passada você moveu a construção da transação para o seu backend: o checkout-txreq monta a transação no servidor e carimba ele mesmo o memo e a reference, então o preço mora no seu código em vez de numa URL controlada pelo cliente. Esse endpoint só foi dirigido de uma aba de navegador e de um script de smoke na sua própria máquina. Hoje ele encara um cliente.

É sábado. Você tem uma mesa dobrável numa feira de discos, um caixote de prensagens da Wavelength, nenhuma maquininha de cartão e uma fila se formando. O seu checkout é uma página web. Ele consegue receber dinheiro do outro lado de uma mesa?

Consegue, e você não vai escrever um frontend novo para isso. O repo do Solana Pay traz uma maquininha de primeira parte, o `examples/point-of-sale`, um app Next.js com teclado numérico, tela de QR e fluxo de confirmação. Um toggle lá dentro aponta a coisa inteira para o endpoint de transaction request que você já construiu. Comece o clone agora para a instalação rodar enquanto você lê:

```bash
# the old solana-labs/solana-pay URL redirects here; cloned fresh 2026-08-22
git clone https://github.com/solana-foundation/pay.git
cd pay && git checkout 94b3627   # the POS example this lesson was written against; main moved to kit v8 on 2026-08-31

# the example consumes the repo's core package by path, and that package ships unbuilt.
# build it from the repo's own pnpm workspace first, or the app 500s on a missing import.
cd typescript && pnpm install && pnpm --filter @solana/pay build

cd packages/solana-pay/examples/point-of-sale
npm install   # Node 24+, the course floor, more than covers this repo's own pins
```

Esse bloco presume que o `pnpm` está no seu path; se não estiver, `corepack enable pnpm` liga o shim que o Node já traz.

Sobre a versão do Node, porque senão ela te morde antes de a instalação terminar: o `package.json` do próprio exemplo fixa `engines.node >=18`, mas ele consome o pacote core do repo por caminho (`"@solana/pay": "file:../../core"`), e esse pacote fixa `engines.node >= 20` por causa do Ed25519 em `crypto.subtle`. Então o piso que este repo impõe é 20 — e o Node 24+ que você roda desde o módulo 1 passa com folga. Confirme mesmo assim, em vez de descobrir no primeiro import que falha:

```bash
node --version
```

Checkpoint: `v24.x` ou mais novo, o piso do curso desde o módulo 1. No 18 a instalação pode até funcionar e aí a primeira checagem de signature explode dentro do `crypto.subtle`, o que é uma falha confusa de depurar só pela mensagem de erro.

Mais uma correção de pré-voo, e ela é do upstream, não sua. O `@solana/connector`, a camada de carteira que a maquininha usa, lista o `@solana/web3.js` como peer *opcional* e vai atrás dele com `await import('@solana/web3.js')` dentro de um ramo de transação legada que este app nunca toma. Peers opcionais não são instalados, e o exemplo fica fora do workspace pnpm do repo, então o npm resolve ele sem lock e esse import não tem para onde apontar. O webpack não está nem aí para o ramo estar morto: ele resolve `import()` em tempo de build e quebra o build. Você recebe um 500 no primeiro carregamento da página dizendo `Module not found: Can't resolve '@solana/web3.js'`. Ainda em aberto na `main` em 2026-09-01, então o pin não causou isso.

A correção é uma linha de config do bundler, e é a honesta: você está contando a verdade para o webpack, que uma dependência opcional está ausente. Adicione um hook `webpack` ao objeto de config no `next.config.js`, ao lado do `reactStrictMode`:

```js
webpack(config) {
    // @solana/connector's optional peer, reached only by a legacy code path this app
    // never takes. It is not installed; resolve it to nothing instead of failing the build.
    config.resolve.fallback = { ...config.resolve.fallback, '@solana/web3.js': false };
    return config;
},
```

Não "conserte" isso instalando o `@solana/web3.js`. Nada nesta barraca roda em cima dele, e puxar o SDK deprecado para dentro de uma árvore kit v6 só para satisfazer um import morto é o hábito errado de aprender. Checkpoint com as duas correções no lugar: `npm run dev`, e em outro terminal `curl -o /dev/null -w '%{http_code}\n' 'http://localhost:3000/new?recipient=<any address>&label=Test'` imprime `200`.

Enquanto isso instala, uma promessa sobre o resto desta lição: metade dela é a construção, e a outra metade é uma varredura honesta do que de hardware Solana presencial de fato existe em 2026. A segunda metade importa tanto quanto a primeira, porque o jeito mais rápido de perder a confiança de um lojista é prometer pagamento por aproximação numa blockchain que não tem isso.

## Resumo

Aqui está o que hoje estabelece, linha por linha:

- O repo do pay traz uma maquininha de primeira parte em `examples/point-of-sale` (Next.js, teclado numérico, QR, fluxo de confirmação). Você configura ela com uma URL: `/new?recipient=<address>&label=<name>`.
- Uma linha comentada no `App.tsx` troca a maquininha de transfer requests para transaction requests. Aponte esse `link` para o seu endpoint checkout-txreq e a barraca reusa o preço no servidor que você construiu na lição passada.
- No modo link, a maquininha anexa os parâmetros da venda (`recipient`, `amount`, uma `reference` nova, `label`) à URL do seu endpoint antes de codificar o QR. O preço do teclado é entrada do lojista a partir do aparelho do próprio lojista, o que é um modelo de confiança diferente de uma URL editável pelo cliente. O seu endpoint ainda valida ele, e nunca tira o recebedor da query.
- A tela de confirmed é polling, não push: o `findReference` percorre o `getSignaturesForAddress` na reference key da venda até aparecer uma signature, e aí o pagamento é validado. Conhecer esse loop é como você depura um "pending" travado.
- Não existe primitivo de NFC nem de pagamento por aproximação na Solana. Nenhuma spec define um protocolo de leitor e nenhuma documentação oficial descreve um. A spec diz, sim, que uma URL de Solana Pay "pode ser codificada em QR codes ou tags NFC", mas isso é transporte, uma tag carregando a mesma string, não um fluxo de aproximação. QR é a superfície de checkout padrão. Não prometa aproximação.
- Seeker, Seed Vault e a dApp Store são produtos reais, mas os números na homepage deles (uma linha de "150,000+ usuários" com referente ambíguo, uma linha de "0% de taxas de plataforma") são afirmações de homepage. Os números de unidades vendidas não são divulgados. Não cite nenhum deles como fato verificado.
- A Decaf, um dia o nome de maquininha de festival na Solana, hoje vende links de pagamento globais que aceitam cartão, transferência bancária ou cripto, com retirada em dinheiro e repasses transfronteiriços em mais de 180 países, e nenhuma maquininha Solana sobrou no site dela (decaf.so, reconferido em 2026-08-22). Não existe fornecedor de terminal para comprar, então você constrói em cima do exemplo de primeira parte.
- O Commerce Kit é beta ("as APIs podem mudar"). Ele ganha uma menção aqui e nenhum papel estrutural.

Como o trabalho está dividido hoje: o lab guiado te entrega todo comando para clonar, religar e rodar a barraca. O degrau de completion deixa o recebedor do lojista, o link do endpoint e o carrinho como três TODOs no config do `pos-stall`. O degrau solo é a coisa de verdade: uma venda presencial completa, do scan à signature liquidada, com recibo registrado. A checagem desta lição é esse deploy, não um quiz.

## Um toggle, de página web a maquininha

### O que você acabou de clonar

O exemplo point-of-sale é um app pequeno com formato de produção: um frontend Next.js, uma camada de API, rate limiting e um `.env.example` que aponta o `CLUSTER_ENDPOINT` para a devnet por padrão. O `package.json` dele, no commit fixado, roda em `@solana/kit ^6.9.0` e consome o `@solana/pay` do pacote core do próprio repo, cujo release no npm é 1.0.26 (publicado em 2026-07-31, ainda `latest` numa conferência de 2026-08-22, com peer em kit ^6.9). Isso mantém a barraca dentro do mesmo workspace kit v6 que este curso usa desde o módulo 2. Na `main` já não manteria: um commit em 2026-08-31 moveu o exemplo para o kit v8, que é exatamente por que o clone acima faz checkout de `94b3627` em vez de pegar carona na `main`. No pin, nenhuma linha nova de SDK, nenhum precipício de versão.

De fábrica, o app inteiro é configurado por uma URL, sem nenhum arquivo de config envolvido:

```
/new?recipient=<your merchant address>&label=<your stall name>
```

Abra isso e aparece um teclado numérico; você digita um valor e ele renderiza um QR codificando um transfer request, o formato `solana:<recipient>` de duas lições atrás. O cliente escaneia, a carteira dele monta a transferência, e o app faz polling do `findReference` até o pagamento aterrissar, e aí vira uma tela de confirmed. Útil, mas tem exatamente a limitação que te empurrou para os transaction requests: a carteira monta a transação, então um recebedor fixo com um valor digitado é tudo que ela consegue expressar. Sem memo, sem order id, sem lógica de carrinho.

O destravamento está nove linhas dentro do componente do app. Abra `src/client/components/pages/App.tsx` e ache isto:

```tsx
// If you're testing without a mobile wallet, set this to true to allow a browser wallet to be used.
const connectWallet = false;

// Toggle comments on these lines to use transaction requests instead of transfer requests.
const link = undefined;
// const link = useMemo(() => new URL(`${baseURL}/api/`), [baseURL]);
```

Os mantenedores deixaram a costura ali de propósito. Quando `link` é uma URL, a maquininha para de codificar transfer requests e começa a codificar transaction requests, `solana:<https-link>`, mirando onde quer que `link` aponte. A linha comentada mira ele na API empacotada do próprio app. Você vai mirar no checkout-txreq em vez disso, porque você já construiu o endpoint melhor: ele precifica no servidor, ele carimba o memo e a reference, e o smoke test da lição passada prova que ele devolve uma transação em base64 para um `{account}` postado.

![Trecho anotado do App.tsx mostrando a flag connectWallet e o toggle do link: link undefined quer dizer que a carteira monta uma transferência, link definido quer dizer que o seu servidor monta.](assets/v01-annotated-code.png)

### Como a venda de fato flui

Trace uma venda pela barraca religada, porque dois dos saltos são novos e um deles muda o trabalho do seu endpoint.

Você digita 0.15 no teclado numérico e aperta gerar. A maquininha pega a sua URL de `link` e anexa a venda nela como parâmetros de query antes de codificar qualquer coisa: o `recipient` configurado, o `amount` digitado, o `label` da sua barraca, e `reference`, uma chave pública nova de uso único que ela cunha para esta venda com `generateKeyPairSigner` (a mesma disciplina de reference de 32 bytes em base58 que você usa desde a primeira lição de QR). Depois ela codifica a coisa toda com `encodeURL({ link })` e pinta o QR. O cliente escaneia. A carteira dele faz o passo duplo que você implementou na lição passada: GET no seu endpoint atrás de um label e um ícone, depois POST de `{account}` na mesma URL, query string e tudo. O seu servidor monta a transação, a carteira assina e submete, e a maquininha faz polling do `findReference` na reference que ela cunhou até a signature aparecer, e aí valida e vira a tela de confirmed.

![Fluxograma de uma venda em três raias (maquininha, carteira do cliente, servidor), de digitar um valor e cunhar uma reference até assinar, submeter e a maquininha confirmar via findReference.](assets/v02-flowchart.png)

Aqui está o salto que muda o trabalho do seu endpoint: o valor chegou na URL. Duas lições deste curso martelaram "nunca confie num preço mandado pelo cliente", e agora o preço volta a viajar numa query string. Então qual é?

Olhe quem cunhou a URL. No módulo 3, lição 1, o cliente segurava um link que ele podia editar antes de a carteira dele sequer usar, então um preço naquela URL era entrada do cliente. Na barraca, a maquininha roda no seu aparelho, atrás da sua mesa. O valor do teclado é entrada do lojista: você digitou, o seu aparelho cunhou o QR, e o cliente só escaneia o que você mostrou. Esse é o mesmo modelo de confiança de digitar um preço numa maquininha de cartão. A regra não mudou, o autor da URL mudou. O seu endpoint ainda deve validar o formato com rigor (finito, positivo, com limites sensatos), porque um endpoint https é alcançável por qualquer um, não só pela sua maquininha, e uma chamada malformada ou hostil precisa falhar alto em vez de montar uma transação. O que o endpoint não precisa mais fazer nas vendas de barraca é buscar um carrinho por order id, porque o total foi composto na mesa:

```typescript
// checkout-txreq: the stall branch of your POST handler's pricing step.
// Keypad sales arrive with amount + reference minted by YOUR pos device;
// validate the shape, then price from them instead of a stored cart.
import { toBaseUnits } from '../../transfer-kit/src/index';

export function priceFromStallLink(query: URLSearchParams): {
  baseUnits: bigint;
  reference: string;
} {
  const amount = query.get('amount');
  const reference = query.get('reference');
  if (!amount || !reference) {
    throw new Error('stall sales carry amount + reference minted by the POS');
  }
  // Number() here is a RANGE check on the string, never the money math.
  const bound = Number(amount);
  if (!Number.isFinite(bound) || bound <= 0 || bound > 100) {
    throw new Error(`rejected amount: ${amount}`);
  }
  // Exact base units straight from the decimal string, using the same helper
  // transfer-kit has used since module 2: devnet USDC at 6 decimals, the same
  // mint and decimals the web-cart path prices in. No float touches the amount.
  return { baseUnits: toBaseUnits(amount, 6), reference };
}
```

Ligue isso no POST handler que você construiu na lição passada como um ramo: se a query carrega `amount` e `reference`, é uma venda de barraca, precifique a partir do link; senão, é o caminho do carrinho web que você já tem. Repare qual função o ramo da barraca chama, porque não é `buildOrderTransaction`: aquela função continua se recusando a aceitar um valor de qualquer chamador, exatamente como projetada na lição passada, e o projeto se sustenta. O ramo da barraca monta o próprio `TransferChecked` a partir do valor validado do teclado e entrega ele direto para `finalizeTransaction`, a cauda compartilhada que a lição passada exportou exatamente para este tipo de chamador, e passa adiante a `reference` cunhada pela maquininha em vez de cunhar uma nova. Essa última parte é estrutural: a maquininha faz polling do `findReference` na chave que ela cunhou, então um servidor que troca por uma reference própria deixa a tela de confirmed em pending para sempre. Tudo rio abaixo, o carimbo do memo, a injeção da reference, a resposta em base64, continua sendo o código que você já escreveu. É esse o acúmulo que este módulo não para de prometer: o pos-stall não substitui o checkout-txreq, ele consome ele.

Um parâmetro merece uma regra mais dura que validação. A maquininha também anexa `recipient` na query, e o seu endpoint deve ignorar ele completamente. O recebedor é configuração no seu servidor, definida uma vez, não um valor que chega a cada requisição; um endpoint que paga qualquer recipient que a query nomear é um open redirect para dinheiro, porque qualquer um que alcance a URL https consegue botar o próprio endereço nela. Mesma história para `memo` se ele aparecer: o seu endpoint carimba o próprio memo com o próprio order id, e isso continua valendo na barraca. A query tem permissão de te dizer quanto custa esta venda. Ela nunca tem permissão de te dizer quem recebe ou o que dizem os livros.

![Uma decomposição rotulada da URL do payload do QR: o esquema solana, o link https que marca ela como transaction request, o caminho /txreq e os parâmetros por venda que a maquininha anexa.](assets/v03-diagram.png)

### O que a tela de confirmed de fato sabe

O último salto merece um olhar mais de perto, porque é nele que você vai ficar encarando quando uma venda travar. Como é que uma página web no seu notebook sabe que uma transação que ela nunca viu, assinada num celular em que ela nunca encostou, acabou de aterrissar na devnet?

Ela faz polling. Não existe canal de push neste fluxo: a maquininha chama `findReference` na reference key que ela cunhou para a venda, e por baixo dos panos isso é `getSignaturesForAddress` contra o seu RPC, repetido num intervalo. A reference viaja na transação como uma chave não signatária na instrução de transferência (o seu endpoint injeta ela, esse foi o trabalho da lição passada), então no momento em que a transação aterrissa, a reference key tem um histórico de signatures de exatamente uma entrada. Até lá, o `findReference` lança `FindReferenceError`, a maquininha captura, espera e pergunta de novo. Pending não é um estado que a blockchain reporta; pending é o loop ainda não ter achado nada.

Duas consequências caem desse desenho, e as duas vão te poupar tempo de depuração no sábado. Primeira, uma tela travada em pending tem exatamente três suspeitos: o cliente nunca aprovou (olhe o celular dele), a transação falhou on-chain (a carteira mostra o erro), ou o seu RPC ainda não indexou a signature (espere, ou confira a reference key num explorer você mesmo). A maquininha não consegue distinguir os três, mas você consegue, em uns dez segundos, checando nessa ordem. Segunda, a confirmação que a maquininha te mostra roda num nível de commitment; a biblioteca nem aceita `processed` para esta consulta, o que é a API impondo em silêncio a política que este curso repete desde o módulo 1: nunca entregue mercadoria num status que ainda pode ser revertido. Se `confirmed` basta para entregar um disco, ou se você espera `finalized` fazendo conversa fiada, é uma decisão de política de verdade com números de latência de verdade presos nela, e a lição de liquidação do próximo módulo deixa isso rigoroso. Para uma barraca de sábado na devnet, o padrão está de bom tamanho.

Depois que a signature aparece, a maquininha valida a transação encontrada antes de virar a tela, checando que o que aterrissou bate com a venda que ela codificou. Guarde essa ordem na cabeça: encontrada, depois validada, depois confirmed na tela. Uma signature existir não é a mesma coisa que o pagamento certo existir.

![Um fluxograma do loop de confirmação da maquininha, o findReference fazendo polling até aparecer uma signature e então validando antes de mostrar confirmed, com um checklist ordenado de três suspeitos para diagnosticar uma tela travada em pending.](assets/v04-flowchart.png)

### A realidade do hardware: o que existe, o que não existe

Agora a segunda metade da lição, e eu quero ser direto com você, porque é aqui que muito conteúdo de comércio na Solana promete demais em silêncio. Você está prestes a rodar um ponto de venda em cima de um notebook e um QR code, e um vendedor da mesa do lado vai fazer a pergunta óbvia: "dá para só aproximar o celular?"

Não. Não existe primitivo de NFC nem de pagamento por aproximação na Solana: nada na spec do Solana Pay, na documentação oficial ou na stack da Solana Mobile define um, e nenhuma fonte primária descreve um fluxo de aproximação que você pudesse entregar.

Seja preciso sobre uma linha que é lida errado como promessa, porque um lojista que der grep vai achar ela. A seção de motivação da spec diz que URLs de Solana Pay "podem ser codificadas em QR codes ou tags NFC, ou enviadas entre usuários e aplicações para solicitar pagamento e compor transações". Essa frase é sobre *transporte*: uma tag NFC é mais um jeito de entregar a mesma URL `solana:` para alguém, exatamente como imprimir ela num adesivo. Ela não define protocolo de leitor nenhum, handshake nenhum, secure element nenhum, nada do que acontece quando um cliente encosta um celular perto da sua mesa. Pagamento por aproximação como você conhece de uma maquininha de cartão é uma capacidade do mundo EMV, construída em cima de secure elements, redes de adquirentes e leitores certificados, e nada desse encanamento tem equivalente na Solana hoje. Uma URL numa tag NFC é um QR code que você não consegue ver; não é pagamento por aproximação. QR é a superfície de checkout padrão, ponto final. Quando você promete "checkout cripto" para um lojista, o formato honesto dessa promessa é uma câmera apontada para uma tela.

Antes de ler isso como um rebaixamento, olhe para onde os pagamentos presenciais de fato foram nesta década. Os maiores sistemas de pagar escaneando do planeta são sistemas de QR: Pix, UPI, Alipay e WeChat Pay todos pararam na câmera como superfície de checkout, exatamente neste tipo de mesa, precisamente porque um QR code precisa de zero hardware especial do lado de quem vende. Um vendedor de rua imprime um código uma vez e está em operação. A aproximação exige um secure element certificado dentro de um leitor certificado dentro de uma relação certificada com um adquirente; escanear exige uma tela, ou papel. Então o jeito honesto de colocar isso para o seu vizinho de barraca não é "a Solana ainda não faz aproximação", é "o checkout da Solana funciona do jeito que funcionam os trilhos de pagamento mais novos do mundo". A lacuna que é real, e sobre a qual vale ser direto, é o acabamento: esses sistemas nacionais têm uma década de UX de carteira atrás deles, e o primeiro scan de Solana Pay de um cliente vai parecer menos ensaiado do que o centésimo scan de Pix dele. Essa é uma lacuna de software, não de hardware, e lacunas de software fecham.

E do lado do celular? A Solana Mobile entrega o Seeker, um celular com Seed Vault, que é custódia de chave em hardware, não uma funcionalidade de pagamento, e ele roda uma dApp Store. Produtos reais, e a proposta de taxa da dApp Store é genuinamente interessante para a economia de apps. Mas cuidado com os números da homepage: o número de "150,000+ usuários" tem um referente ambíguo (usuários de quê, exatamente, não está dito), a linha de "0% de taxas de plataforma" é uma afirmação de preço, e os números de unidades vendidas não são divulgados. Trate tudo isso como afirmações de homepage, não apresente nada disso como dado de adoção verificado, e repare no que está ausente: nada na stack do Seeker dá pagamento por aproximação para a sua barraca também. Um cliente de Seeker na sua mesa ainda escaneia o mesmo QR que um cliente de iPhone.

![Uma matriz de capacidades marcando transfer requests e transaction requests por QR e o exemplo de maquininha de primeira parte como reais, o Commerce Kit como beta e terminais de pagamento por aproximação como inexistentes na Solana.](assets/v05-comparison.png)

A evidência mais afiada de quão fino é o nicho de loja física vem da empresa que era dona dele. A Decaf era a queridinha de maquininha de festival deste ecossistema: o nome que você ouvia sempre que alguém pagava comida com USDC num evento Solana. Entre em decaf.so hoje (eu reconferi em 2026-08-22) e a maquininha Solana sumiu. O produto é um link de pagamento que você cria em dois minutos, pagável com cartão, transferência bancária ou cripto, com retirada em dinheiro e repasses em mais de 180 países, mirado exatamente no remetente que a Stripe e o PayPal não vão atender. Mesma empresa, mesmos trilhos por baixo, cliente completamente diferente.

Leia esse pivô como dado de mercado, porque é isso que ele é. A demanda é um ator aqui, e ela votou: o lojista de pé atrás de um terminal acabou sendo um cliente bem menor do que o trabalhador mandando dinheiro para casa ou o negócio faturando do outro lado de uma fronteira. A demanda por maquininha cripto em loja física era mais fina do que a demanda por repasse, então o capital e o produto seguiram os repasses. O mesmo formato aparece nas comunidades de builders da América Latina: o pagamento cripto que acontece todo santo dia é o repasse transfronteiriço para um colaborador, não o café comprado com uma carteira. Nada disso quer dizer que a sua barraca é uma má ideia. Quer dizer que ninguém vai te vender um terminal para ela, não existe catálogo de fornecedores em que se apoiar, e o exemplo de primeira parte que você clonou é a base sancionada precisamente porque a camada comercial acima dele se esvaziou. Construa de acordo, e saiba que o mesmo endpoint que move a sua mesa é a peça que se transfere para onde a demanda de fato mora.

![Linha do tempo da Decaf saindo de ponto de venda de festival na Solana, passando por demanda fina em loja física, até links de pagamento globais e repasses transfronteiriços em mais de 180 países.](assets/v06-timeline.png)

Tem mais uma peça de honestidade sobre hardware, e é a que ninguém põe num slide: a rede na feira. Percorra o fluxo da venda de novo e conte as conexões que ele precisa. O seu notebook precisa ser alcançável pelo celular do cliente (o GET e o POST no seu endpoint) e precisa alcançar o RPC da devnet (o polling de confirmação). O celular do cliente precisa ter dados, porque a carteira dele submete a transação assinada para a própria blockchain. São três dependências de rede para uma venda, e uma feira de discos num salão paroquial com paredes de concreto e duzentos celulares num único ponto de acesso vai testar cada uma delas. Isso não é uma fraqueza específica de cripto, maquininhas de cartão também morrem com conectividade ruim, mas um fornecedor de maquininha de cartão passou vinte anos fazendo engenharia de store-and-forward em volta disso, e você não. Ainda. Mais adiante neste curso você constrói exatamente isso: uma fila offline que assina vendas na mesa e escoa elas quando a rede volta, em cima de um primitivo chamado durable nonce. Por enquanto, as mitigações práticas são chatas e eficazes: o seu próprio hotspot para o notebook, um QR de fallback impresso para um item de preço fixo, e saber qual falha se parece com qual na tela de pending.

![Um checklist de pré-feira cobrindo alcance na LAN, acesso ao RPC, dados no celular do cliente, um hotspot, aceitação de certificado e um QR de fallback impresso, além de como cada falha de rede se apresenta na barraca.](assets/v07-table.png)

Um último aparte antes do lab. Existe um Commerce Kit no ecossistema, e ele é beta, com o aviso "as APIs podem mudar" da própria documentação grudado nele. Você já sabe o bastante para decodificar o que isso quer dizer para uma barraca da qual você depende: um sábado de vendas não é lugar para uma superfície de API que se reserva o direito de mexer debaixo de você. Saiba que ele existe, acompanhe ele amadurecer, e construa a mesa de hoje em cima do exemplo de primeira parte e do seu próprio endpoint. É essa a menção inteira.

## Lab: monte a barraca

Worked rung: todo comando abaixo é dado. Você clona (já feito acima), religa e registra uma venda localmente.

1. **Rode a maquininha de fábrica, uma vez.** A partir de `pay/typescript/packages/solana-pay/examples/point-of-sale`, suba o servidor de dev e, num segundo terminal, o proxy SSL (ele vem como dependência de dev, o seu `npm install` já baixou):

   ```bash
   npm run dev     # Next.js on http://localhost:3000
   npm run proxy   # local-ssl-proxy: https://localhost:3001 -> 3000
   ```

   Abra `https://localhost:3001/new?recipient=<YOUR_MERCHANT_ADDRESS>&label=Wavelength%20Records`, aceite o certificado assinado localmente, e checkpoint: você deve ver o teclado numérico com o nome da sua barraca no topo. Digite um valor e gere um código para ver o fluxo de transfer request de fábrica uma vez. Conhecer o comportamento de fábrica é o que torna a mudança do passo 3 visível.

2. **Ponha o checkout-txreq atrás de https.** Carteiras exigem https para links de transaction request, e o seu endpoint da lição passada roda em http puro localmente (o meu escuta na 3100; troque pela sua porta). Faça proxy dele do mesmo jeito que a maquininha faz proxy de si mesma:

   ```bash
   npx local-ssl-proxy --source 3443 --target 3100
   ```

   Checkpoint: `curl -k https://localhost:3443/txreq` devolve a resposta de GET do seu endpoint, o JSON de label e ícone do smoke test da lição passada. A flag `-k` pula a checagem de confiança do certificado, e o navegador não vai pular: abra `https://localhost:3443/txreq` no navegador também e aceite o certificado autoassinado agora, ou o fetch da página da maquininha morre depois com ERR_CERT_AUTHORITY_INVALID antes de o fluxo de venda sequer começar.

3. **Vire o toggle.** Em `src/client/components/pages/App.tsx`, faça as duas edições da seção de teoria:

   ```tsx
   const connectWallet = true;  // browser-wallet dev loop for this lab

   // Toggle comments on these lines to use transaction requests instead of transfer requests.
   // const link = undefined;
   const link = useMemo(() => new URL('https://localhost:3443/txreq'), []);
   ```

   Depois uma terceira edição, que a seção de teoria não cobriu porque é sobre o token, não sobre o link: o app de fábrica vem configurado para SOL nativo. Lembre da ordem da seção da tela de confirmed, encontrada, depois validada. A maquininha vai encontrar a sua signature, depois validar a transação aterrissada contra a config dela mesma, e um `validateTransfer` configurado para SOL checando o `TransferChecked` de USDC que o seu endpoint monta rejeita ela e pinta **Invalid**. No mesmo arquivo, adicione `USDCIcon` aos imports (o componente já vem no exemplo, do lado do `SOLIcon`):

   ```tsx
   import { USDCIcon } from '../images/USDCIcon';
   ```

   e no `<ConfigProvider>` mais abaixo, adicione `splToken` e troque as quatro props com sabor de SOL pelos valores de USDC (`address` já está importado no topo do arquivo):

   ```tsx
   splToken={address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU')}
   symbol="USDC"
   icon={<USDCIcon />}
   decimals={6}
   minDecimals={2}
   ```

   Adicione o ramo da barraca da seção de teoria (`priceFromStallLink`) ao POST handler do seu checkout-txreq, se ainda não adicionou. Checkpoint: recarregue a página da maquininha, digite um valor, gere, e o QR agora codifica `solana:https://localhost:3443/txreq?amount=...&reference=...`. A tela de pending aparece e o log do seu endpoint mostra o GET chegando.

4. **Registre uma venda na devnet.** Com `connectWallet = true`, pague por uma carteira de navegador na mesma máquina (abastecida com SOL de devnet para as taxas e USDC de devnet do faucet da Circle, como no módulo 2; a transação que o seu endpoint monta é o mesmo `TransferChecked` de USDC do carrinho web). Aprove a transação que a maquininha te entrega. Checkpoint: a maquininha vira de pending para confirmed e o anel de progresso completa; a signature em si mora atrás de "Recent Transactions", não na tela de confirmed. O log do seu endpoint mostra o POST `{account}` e a transação em base64 que ele devolveu. Aquela signature liquidou uma venda que o seu servidor precificou a partir do teclado. A página web acabou de receber dinheiro do outro lado de uma mesa. Se em vez disso o anel disser **Invalid**, o `validateTransfer` rejeitou a transferência: confira se `splToken` está definido no App.tsx e se o seu memo vem antes da transferência em `finalizeTransaction`. E se o anel nunca sair de pending enquanto o log do seu endpoint mostra GETs mas nenhum POST, o navegador bloqueou o fetch cross-origin: o middleware de CORS do servidor da lição passada precisa estar no processo que roda atrás do proxy 3443 — o `local-ssl-proxy` encaminha headers, ele não adiciona headers.

5. **Faça o scaffold do artefato.** O artefato do curso nesta lição é o `pos-stall`, um diretório fininho que fixa a configuração da sua barraca e prova ela com um smoke test, sentado do lado do `checkout-txreq` no seu workspace Wavelength:

   ```bash
   mkdir pos-stall && cd pos-stall
   npm init -y
   npm pkg set type=module
   npm install @solana/pay@1.0.26 @solana/kit@^6.10.0 qrcode
   npm install -D tsx typescript @types/node @types/qrcode
   ```

   A linha `type=module` importa: o `@solana/pay` publica os tipos de TypeScript dele sob a entrada ESM, e um pacote ESM é o que deixa o `tsc --strict` resolver eles direito (eu mesmo bati no erro de declaração faltando antes de adicionar ela, então considere esses dez minutos doados). Notas de pin, conferidas em 2026-08-22: `@solana/pay` 1.0.26 é o `latest` do npm (publicado em 2026-07-31) e tem peer em kit ^6.9, então `@solana/kit@^6.10.0` mantém isto no workspace v6 do curso; o `qrcode` (1.5.x na hora da conferência) renderiza QR codes num terminal Node puro, coisa que a biblioteca de estilização presa ao navegador que a maquininha usa não consegue.

6. **Escreva o config, com os TODOs de completion deixados em aberto.** Crie `pos-stall/config.ts`:

   ```typescript
   export interface CartLine {
     sku: string;
     title: string;
     priceUsdc: number; // UI units with cents precision, e.g. 12.5
   }

   export const STALL: {
     label: string;
     recipient: string;
     txreqLink: string;
     cart: CartLine[];
   } = {
     label: 'Wavelength Records',
     // TODO(completion): your merchant wallet, the same recipient checkout-txreq pays
     recipient: '11111111111111111111111111111111',
     // TODO(completion): where your transaction-request endpoint lives (https, /txreq path)
     txreqLink: 'https://localhost:3443/txreq',
     // TODO(completion): the crate you are selling today, priced in devnet USDC
     cart: [],
   };
   ```

7. **Escreva e rode o smoke test.** Crie `pos-stall/smoke.ts`. Ele faz exatamente o que a maquininha faz por venda, cunhar uma reference, anexar `amount` e `reference` ao link, codificar, então uma rodada que passa prova que o seu config produziria um QR de barraca escaneável:

   ```typescript
   import { encodeURL } from '@solana/pay';
   import { generateKeyPairSigner } from '@solana/kit';
   import QRCode from 'qrcode';
   import { STALL } from './config.js'; // .js extension: ESM resolution rule, even from .ts

   async function main(): Promise<void> {
     const link = new URL(STALL.txreqLink);
     if (!link.pathname.endsWith('/txreq')) {
       throw new Error(`POS must point at the transaction-request endpoint, got ${link.pathname}`);
     }
     if (link.protocol !== 'https:') {
       throw new Error(`transaction requests require https, got ${link.protocol}`);
     }

     // Summed in integer cents so no float drift ever reaches the QR amount;
     // the URL speaks UI units (decimal USDC), same convention as every
     // Solana Pay amount this module has written.
     const totalCents = STALL.cart.reduce(
       (sum, line) => sum + Math.round(line.priceUsdc * 100),
       0,
     );
     if (totalCents <= 0) {
       throw new Error('cart is empty; fill the completion TODOs in config.ts first');
     }
     const totalUsdc = (totalCents / 100).toFixed(2);

     // one fresh reference per sale, exactly as the POS mints one per payment
     const referenceSigner = await generateKeyPairSigner();
     link.searchParams.append('amount', totalUsdc);
     link.searchParams.append('reference', referenceSigner.address);

     const url = encodeURL({ link });
     console.log(await QRCode.toString(url.toString(), { type: 'terminal', small: true }));
     console.log(`POS points at /txreq; QR generated for the cart total (${totalUsdc} USDC, ${STALL.cart.length} items)`);
   }

   main().catch((error) => {
     console.error(error instanceof Error ? error.message : error);
     process.exit(1);
   });
   ```

   Rode `npx tsx smoke.ts`. Com os TODOs ainda em aberto ele falha com `cart is empty; fill the completion TODOs in config.ts first`, o que está correto: o smoke test falhando é a lista de tarefas do seu degrau de completion. (Este arquivo exato, nestes pins exatos, passa no type-check sob `npx tsc --strict --noEmit smoke.ts config.ts` e roda; se não passar para você, a extensão do import e a linha `type=module` do passo 5 são os dois suspeitos de sempre.)

![Diagrama de deploy da barraca: um notebook rodando a maquininha e o endpoint atrás de proxies SSL locais, um celular de cliente alcançando eles pela LAN, e a devnet liquidando a transação.](assets/v08-diagram.png)

## Challenge

Dois degraus, e o segundo é a checagem de verdade da lição.

**Completion.** Preencha os três TODOs em `pos-stall/config.ts`: o recebedor do seu lojista, o link https do seu endpoint e um carrinho de verdade (três ou quatro prensagens com preços em USDC já basta). Aceitação: `npx tsx smoke.ts` imprime um QR no terminal e a linha `POS points at /txreq; QR generated for the cart total`, e o valor codificado é igual à soma das linhas do seu carrinho. Se o smoke test rejeitar o seu link, leia o erro dele antes de mexer em código: os dois modos de falha que ele checa (caminho errado, http puro) são os dois que matam vendas em silêncio numa mesa de verdade.

**Solo.** Rode uma venda presencial completa de ponta a ponta e registre o recibo. Estilo presencial quer dizer que o QR atravessa o ar: uma carteira de celular escaneando a tela do seu notebook. Troque `localhost` pelo endereço de LAN do seu notebook tanto no link do App.tsx quanto na sua configuração de proxy para que o celular alcance o endpoint, e espere atrito do certificado autoassinado, carteiras de celular são mais rígidas com certificados do que o navegador do seu desktop (este é o custo honesto de um lab https local; um endpoint deployado com um certificado de verdade faz isso sumir — um deploy que este curso deixa por sua conta, já que até o capstone fica de propósito numa máquina só). Se a sua carteira de celular recusar o certificado de cara, o caminho da carteira de navegador do passo 4 do lab continua sendo o seu fallback para a venda liquidada; diga isso no seu recibo. Aceitação: um QR escaneado liquida um carrinho na devnet através da maquininha, e o seu recibo registra o valor digitado, a reference que a maquininha cunhou e a signature liquidada. Esse artefato de venda concluída, não um quiz, é o gate desta lição.

Antes de desmontar a mesa, repare no que você não construiu hoje: um frontend. A superfície inteira veio do exemplo de primeira parte do repo do pay, e toda linha que você de fato escreveu ou configurou ele ou estendeu o endpoint que você já tinha. Essa é a proporção certa para trabalho de comércio, e vale sentir isso pelo menos uma vez: infraestrutura que outra pessoa mantém, lógica de preço que é sua. Se a venda liquidou, você está na frente da maioria dos pitches de "maquininha cripto" que cruzaram uma mesa de demo este ano, e se alguma coisa no fluxo brigou com você, anote onde enquanto a memória está fresca.

O seu endpoint já vendeu um disco por uma página web e do outro lado de uma mesa dobrável, duas superfícies, um núcleo de pagamento. Mas uma tela de barraca ainda faz o cliente vir até você. O mesmo endpoint pode ser mais que isso: ele pode ser um link que você solta no social e que É a loja, executando a compra onde quer que seja postado. Na próxima lição você transforma ele num blink, e a gente encara, com a mesma honestidade que esta lição te devia sobre hardware, onde blinks de fato renderizam.
