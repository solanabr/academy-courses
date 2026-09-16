# Webhooks que mentem: ingestão e idempotência

## Resumo

Na lição passada você construiu o verificador: um `verify(signature, expectedOrder)` do lado do servidor que busca a transação, checa o programa de token, o mint, o delta de saldo e o memo, e mantém um conjunto de signatures já processadas. É a bancada de aceitação do curso. O que ele não consegue fazer é perceber um pagamento sozinho. Alguém tem que contar para ele que uma signature existe. Hoje a gente constrói quem conta, sabendo que quem conta mente.

As conclusões logo de cara:

- Você entrega o **backoffice**: um receptor Express para eventos TRANSFER Enhanced da Helius que deduplica pela signature da transação, passa todo evento pelo verificador da lição passada antes de atender qualquer coisa, e escreve um livro-razão de pedidos onde uma linha quer dizer um pagamento real.
- Entregas duplicadas não são um bug que você trata; são o contrato que você assinou. A própria documentação da Helius diz que ela pode tentar de novo as entregas de webhook se o seu servidor não responder com sucesso, e que você pode receber eventos duplicados (checado em 2026-08-22). Responder devagar conta como não responder.
- A chave de dedup é a signature da transação. Essa escolha é uma inferência nossa, sólida mas nossa: a Helius documenta os retries, não a chave. A gente vai defender ela direito mais abaixo.
- Um webhook é uma notificação, não uma prova. Todo evento passa por verificação on-chain antes de o livro-razão registrar uma venda; os números do payload nunca são entradas para o fulfillment.
- Webhooks têm um teto, e ele é aplicado: um webhook falhando a 95 por cento ou mais ao longo de 7 dias é desativado automaticamente nos planos pagos (uma janela de 24 horas no plano gratuito). Passando do volume de operações de lojista, a ingestão migra para indexação com Yellowstone gRPC, que pertence ao curso Client-Side Mastery.

Sinta a falha primeiro. Salve isto como `naive.ts` em qualquer lugar (assume que o `express` está instalado; se você estiver numa pasta nova, `npm install express@5.1.0` antes):

```ts
// naive.ts - the receiver you must never ship
import express from 'express';

const app = express();
app.use(express.json());

let fulfilled = 0;
app.post('/webhooks/helius', (req, res) => {
  for (const event of req.body) {
    fulfilled++;
    console.log(`shipped order ${fulfilled} for signature ${event.signature.slice(0, 8)}...`);
  }
  res.status(200).end();
});

app.listen(4000, () => console.log('naive receiver on :4000'));
```

Rode com `npx tsx naive.ts`, depois banque a Helius por um momento. Um pagamento real, entregue três vezes, exatamente como uma tempestade de retries entregaria:

```bash
for i in 1 2 3; do
  curl -s -X POST localhost:4000/webhooks/helius \
    -H 'Content-Type: application/json' \
    -d '[{"signature":"5KtPn1abcDEF","type":"TRANSFER"}]'
done
```

Três pedidos despachados. Um pagamento. Se este receptor tocasse o clube do disco do mês da Wavelength, você acabou de mandar pelo correio três cópias de uma tiragem de 200 prensagens para o mesmo cliente e engoliu o custo de duas. O pipeline que você está prestes a construir existe para fazer aquele loop imprimir `shipped order 1` e depois ficar quieto.

![Pipeline mostrando uma entrega de webhook passando pela auth, por um filtro de formato e por um ack 200 imediato, depois um claim de signature, a resolução do pedido e a verificação on-chain antes de uma linha do livro-razão.](assets/v01-diagram.webp)

## Notificações, não prova

### Higiene da Stripe, chaves novas

Se você já integrou a Stripe, já fez esta lição uma vez. A Stripe reenvia webhooks que o seu endpoint não confirma. Integradores da Stripe deduplicam por uma chave de idempotência para que um `checkout.session.completed` reentregue não despache duas vezes. A Stripe te diz, em negrito, para verificar o evento contra a API dela em vez de confiar no corpo do POST, porque qualquer um pode mandar um POST de JSON no seu endpoint. Cada uma dessas frases sobrevive à viagem para a Solana com uma substituição: a chave de idempotência vira a signature da transação, e "verifique contra a API da Stripe" vira "verifique contra a blockchain".

Esse mapeamento vale ser levado a sério em vez de virar slogan, porque a Stripe não é mais uma espectadora nesta história. Em 2026, a Stripe ocupa uma posição em quatro frentes nos pagamentos em cripto: o logo dela está no mural de quem confia do x402.org, ela é coautora do Agentic Commerce Protocol com a OpenAI, é coautora do esquema de autenticação HTTP "Payment" sobre o qual o Machine Payments Protocol é construído, e opera como adquirente de USDC na Solana que liquida lojistas em moeda fiduciária (x402.org, agenticcommerce.dev, o datatracker da IETF e a própria documentação da Stripe, checados em 2026-08-21). A empresa que escreveu a cartilha de higiene de webhook agora processa exatamente o trilho sobre o qual você está construindo. Quando a sua disciplina de webhook aqui bate com a deles, isso é evolução convergente sob o mesmo predador: o evento duplicado.

![Tabela mapeando cinco hábitos de webhook da Stripe para os equivalentes na Solana, com a chave de idempotência virando a signature da transação e a verificação por API virando uma verificação on-chain do pagamento.](assets/v02-table.webp)

Uma assimetria não se transfere, e ela aumenta o risco em vez de diminuir. Quando um integrador da Stripe atende em dobro, existe uma API de reembolso e, atrás dela, uma bandeira que consegue estornar o dinheiro. Aqui, o módulo 1 já te ensinou a verdade seca do trilho: sem chargebacks. Um disco enviado duas vezes não é um ticket de suporte constrangedor, é estoque que se foi. A higiene é a mesma da Stripe; o preço de pular ela é mais alto. O que, honestamente, é a versão de boa notícia de quem constrói. A disciplina que você já conhece é suficiente. Você só precisa, de fato, fazer.

### O evento, Enhanced ou Raw

Webhooks da Helius vêm em dois sabores principais, escolhidos na hora da criação pelo `webhookType`. O **Enhanced** entrega eventos parseados e legíveis por humanos: a Helius roda a transação crua pelo parser dela e te entrega um objeto tipado com um campo `type` como `TRANSFER`, uma `signature`, uma `description` e `tokenTransfers` estruturados. O **Raw** entrega a transação mais perto do formato bruto de transmissão, com menos parsing e latência de entrega mais baixa. Existem também variantes de Discord e gêmeos de devnet de cada um (`enhancedDevnet`, `rawDevnet`), que é o que o lab usa para os seus pagamentos de teste ficarem na devnet.

Para operações de lojista, o Enhanced é o padrão certo. A diferença de latência importa quando você está correndo contra os blocos; uma loja de discos que atende dentro do minuto não liga, e o formato já parseado faz o seu código de filtro ler como prosa. A troca: você está consumindo a interpretação da Helius sobre a transação, mais um motivo para o payload continuar sendo uma pista e não uma fonte de verdade.

Criar um é um único POST autenticado, autenticado com a `HELIUS_API_KEY` que você exportou na preparação do módulo 2. (As suas chamadas de RPC não usam ela; este curso roda o RPC dele contra endpoints públicos, e a chave existe só para webhooks.) Se `echo $HELIUS_API_KEY` não imprimir nada, volte para aquele passo antes de continuar:

```bash
curl -s -X POST "https://mainnet.helius-rpc.com/v0/webhooks?api-key=$HELIUS_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
    "webhookURL": "https://backoffice.wavelength.example/webhooks/helius",
    "webhookType": "enhancedDevnet",
    "transactionTypes": ["TRANSFER"],
    "accountAddresses": ["<YOUR_MERCHANT_USDC_ATA>"],
    "authHeader": "wavelength-webhook-secret"
  }'
```

Uma coisa sobre essa URL antes dos campos: `mainnet.helius-rpc.com` aqui é o host da própria API de webhooks, não uma declaração sobre qual cluster você está observando. O cluster é escolhido pelo `webhookType`, que é por isso que um webhook de devnet é criado contra o mesmo host.

Campo por campo, porque cada um é uma decisão. A `webhookURL` tem que ser uma URL que a Helius consiga alcançar, então localhost precisa de um túnel durante o desenvolvimento (qualquer túnel HTTPS serve; o lab anota uma opção). `transactionTypes: ["TRANSFER"]` estreita a entrega para o tipo parseado que interessa para a gente. `accountAddresses` é a lista de observação: a sua ATA de lojista, a conta em que todo checkout do módulo 3 deposita. E `authHeader` é um valor de sua escolha que a Helius devolve no header Authorization de cada entrega, para o seu receptor poder descartar tráfego que nunca veio da Helius. O painel consegue criar o mesmo webhook por formulário se você preferir clicar. Não existe SDK nesse caminho de um jeito ou de outro: o curl acima é o cadastro inteiro, e o `helius-sdk` nunca é instalado neste curso (se você mesmo assim for atrás dele, repare que o 3.1.0 declara peer `@solana/kit` ^6.9, então ele te prenderia na mesma linha kit v6 em que os workspaces de ops já estão).

O que chega no seu endpoint é um **array** de eventos de transação enhanced, mesmo para uma única transação. Cada elemento carrega `signature`, `type`, `slot`, `timestamp`, `feePayer`, um array `tokenTransfers` com mints e valores, e mais. Aqui está a parte que deve parecer estranha até cair a ficha: de todo esse objeto rico, o nosso receptor vai ler exatamente dois campos, `signature` e `type`. Todo o resto é cenário. Não porque o dado geralmente esteja errado, mas porque "geralmente" não é política de fulfillment, e a gente tem um verificador cujo trabalho inteiro é estabelecer esses mesmos fatos a partir da própria blockchain.

![Comparação entre webhooks Enhanced e Raw em formato de payload, latência, filtragem, postura de confiança e adequação, com o Enhanced marcado como o padrão para operações de lojista e os dois custando um crédito por evento.](assets/v03-comparison.webp)

### Duplicatas são o contrato

Agora o coração da coisa. A promessa de entrega da Helius é deliberadamente modesta: se o seu servidor não responder com sucesso, ela pode tentar de novo, e você pode receber eventos duplicados. "Responder com sucesso" é mais ou menos "responder com um 2xx, a tempo". Leia isso como projetista de sistema e duas consequências caem no colo.

Primeiro: a velocidade do seu endpoint faz parte da corretude dele. Um handler que verifica on-chain antes de responder pode levar segundos sob carga; segundos é tempo suficiente para parecer uma falha, e uma falha quer dizer reentrega. Então o receptor dá o ack primeiro e trabalha depois. Aceite o POST, cheque as coisas baratas (header de auth, formato), responda 200, depois processe cada evento de forma assíncrona. Isso inverte o instinto que a maioria da gente tem, que é responder só quando o trabalho acabou. Aqui, responder É um serviço separado do trabalho, e misturar os dois fabrica exatamente as duplicatas às quais você depois tem que sobreviver.

Segundo: já que duplicatas são esperadas, o fulfillment exatamente-uma-vez não pode morar na camada de entrega de jeito nenhum. Ele tem que morar no seu estado, chaveado em algo que é idêntico em toda duplicata do mesmo pagamento e diferente em todo pagamento distinto. Olhe o evento e pergunte o que se qualifica. `timestamp`? Duplicatas idênticas poderiam discordar se fossem re-parseadas, e dois pagamentos diferentes podem compartilhar um. Os valores de `tokenTransfers`? Dois clientes comprando o mesmo disco de 28 USDC produzem valores idênticos. O payload inteiro em hash? Um re-parse com um campo a mais e a sua chave muda enquanto o pagamento não muda. A signature da transação? Única por transação por construção, imutável assim que a transação aterrissa, presente em toda entrega daquela transação e, melhor de tudo, é exatamente a entrada que o seu verificador já consome.

Então: chaveie a idempotência na signature. Deixa eu rotular isso do jeito que as regras de honestidade deste curso exigem. A Helius documenta os retries; ela não documenta "deduplique pela signature". Esse passo é uma inferência nossa. É uma inferência sólida, que se apoia no que uma signature é on-chain e não em qualquer comportamento de fornecedor, e é a mesma inferência que todo integrador sério faz. Mas se a Helius algum dia mudasse o que uma entrega contém, a inferência é o que você iria rechecar, o que é mais um argumento para o hábito que esta lição treina: nunca deixe a palavra do webhook chegar no livro-razão sem a blockchain confirmar ela.

O mecanismo é um registry de claims com três estados. Antes de qualquer trabalho acontecer sobre um evento, o receptor tenta dar claim na signature dele. Um claim fresh marca a signature como `processing` e segue em frente. Uma segunda entrega que chega no meio da verificação encontra o claim e para seco: é por isso que o claim tem que ser escrito antes de a verificação começar, e não depois de ela dar certo. Quando o trabalho termina, o claim faz settle para `fulfilled` ou `rejected`, os dois terminais. E quando o trabalho falha por um motivo que não é culpa do pagamento, um timeout de RPC, um crash no nosso próprio código, o claim é liberado por completo, então a signature volta a ser lida como nunca-vista.

O registry inteiro é pequeno o bastante para ler num fôlego só:

```ts
// claims.ts - three-state claim registry keyed on the transaction signature
type ClaimState = 'processing' | 'fulfilled' | 'rejected';

const claims = new Map<string, ClaimState>();

export function claim(signature: string): 'fresh' | 'seen' {
  if (claims.has(signature)) return 'seen';
  claims.set(signature, 'processing');
  return 'fresh';
}

export function settle(signature: string, outcome: 'fulfilled' | 'rejected'): void {
  claims.set(signature, outcome);
}

export function release(signature: string): void {
  claims.delete(signature); // transient failure: the next retry claims fresh again
}
```

Por que release em vez de reject? Por causa do que os retries viram então. Se o seu processo morre no meio da verificação de um pagamento real, a Helius vai bater de novo, o claim liberado responde `fresh`, e o pagamento é atendido na segunda passada com zero linhas de código escritas por você. O comportamento de retry contra o qual você passou esta seção inteira se defendendo acaba sendo a sua recuperação de crash, de graça, assim que você para de brigar com ele. O modo de falha que isso mata é real: dê reject num erro transitório e o pagamento daquele cliente fica permanentemente encalhado num estado terminal enquanto o dinheiro dele está parado na sua ATA. Você acharia isso na conciliação da próxima lição, mas "o livro-razão se cura sozinho" ganha de "o livro-razão é auditado".

Antes de sair desta seção, pague pelo padrão ack-depois-trabalho com honestidade, porque ele não é de graça e fingir que é seria exatamente o tipo de esperteza da qual este curso vive jurando abrir mão. No momento em que você responde 200 antes de o trabalho estar feito, você disse à Helius que a entrega deu certo, o que quer dizer que a Helius nunca vai reenviar ela, o que quer dizer que qualquer evento que morra entre o seu ack e o seu settle simplesmente sumiu do ponto de vista do sistema de entrega. Um crash de processo nessa janela, um deploy que reinicia o servidor no meio do lote, uma rejeição não tratada no `processEvent`: o pagamento aterrissou on-chain, a notificação foi entregue e confirmada, e o seu livro-razão não sabe de nada. A válvula do `release` não te salva aqui, já que release só ajuda quando um retry está vindo, e você abriu mão do retry com o seu 200. Então o que de fato segura essa brecha? Duas coisas. Dentro de um tempo de vida de processo, o registry de claims mais o release dão conta de tudo que falha em voz alta. Entre mortes de processo, nada nesta lição dá, de propósito: a rede de segurança para trabalho perdido em silêncio é a varredura de conciliação que você constrói na próxima lição, que percorre a própria história da blockchain contra o livro-razão e faz aparecer todo pagamento que nunca ganhou uma linha. Sistemas de produção estreitam mais a janela empurrando os eventos aceitos para uma fila durável antes do ack, para a fila sobreviver ao crash mesmo depois de a troca HTTP ter acabado. Para volume de operações de lojista, ack-depois-trabalho mais conciliação é a troca honesta e proporcional: você aceita uma pequena janela de perda silenciosa que uma varredura noturna repara, em troca de um endpoint rápido o bastante para a tempestade de retries nunca começar. Só saiba que você fez essa troca, porque a falha que ela permite é invisível até você ir procurar.

![Fluxograma mostrando uma signature com claim feito antes de qualquer trabalho, duplicatas caindo no claim, eventos verificados fazendo settle para fulfilled ou rejected, e erros transitórios dando release no claim para o próximo retry.](assets/v04-flowchart.webp)

### O livro-razão que fala sério

O fulfillment precisa de um registro, e o registro é um artefato próprio: o **livro-razão de pedidos**. O nosso é um arquivo JSONL append-only, uma linha por pedido atendido, carregando o id do pedido, a signature que pagou por ele, o valor e o mint que o pedido esperava, e um timestamp. Isso é deliberadamente sem graça. A parte interessante é a invariante que o pipeline concede a ele: uma linha só é escrita depois de um claim fresh e de uma verificação on-chain aprovada, então uma linha é igual a um pagamento real, sempre. O livro-razão nunca registra tentativas, notificações ou esperanças. Ele registra dinheiro.

Sem graça também é estrutural para o que vem a seguir. Este arquivo é o artefato que a conciliação da próxima lição lê, linha por linha, contra a história on-chain. Mantenha o formato estável, porque uma lição futura chama `rows()` nele e espera exatamente esses campos.

Em produção esse par, registry mais livro-razão, é um banco de dados com duas tabelas, e o claim é um insert numa tabela com índice único na signature. A garantia de unicidade do banco substitui o nosso Map em processo, e o padrão de claim antes do trabalho sobrevive inalterado: insert primeiro, trabalho depois, delete da linha na falha transitória. O nosso registry em memória perde os claims `processing` no restart, e isso é por design e não por preguiça. Linhas fulfilled persistem no arquivo; claims em voo morrem com o processo; os retries reentregam o que morreu em voo. Você também pode reconstruir as entradas `fulfilled` do registry a partir do livro-razão no boot, um loop sobre `ledger.rows()` chamando `settle(row.signature, 'fulfilled')`; vale acrescentar no dia em que você ligar o servidor de verdade, e ficou de fora do lab porque o smoke parte de um arquivo vazio de todo jeito.

Existe também a pergunta silenciosa de quão grande esse estado fica, e a lição passada já te deu o vocabulário para ela. O conjunto de signatures processadas do verificador precisava de um horizonte de expurgo porque guardar toda signature para sempre é crescimento sem limite, e o registry herda a mesma aritmética: todo pagamento que a Wavelength receber na vida deixa uma entrada `fulfilled` para trás. A lógica de expurgo transfere quase inalterada. Uma signature só pode ser reentregue enquanto a transação dela ainda puder ser confundida com uma nova, e assim que um pagamento fica velho o bastante para o tempo de vida do blockhash dele mais uma margem confortável ter passado, e a linha dele estiver em segurança no livro-razão, a entrada do registry cumpriu o trabalho dela e pode ir embora. O livro-razão em si, por contraste, nunca é expurgado; ele é o registro do negócio, cresce uma linha pequena por venda, e um ano de uma loja de discos movimentada cabe em alguns megabytes. Mantenha a distinção nítida na cabeça: o registry é memória operacional com um horizonte, o livro-razão é história sem nenhum.

Mais um hábito da lição passada segue adiante: o conjunto de signatures processadas dentro do verificador continua existindo e continua rodando. O registry do receptor é o portão rápido na porta da frente; a dedup do verificador é defesa em profundidade atrás dele. Duas camadas chaveadas na mesma signature custam quase nada, e o dia em que uma delas tiver um bug é o dia em que você aprende a amar a outra.

![Uma linha JSON anotada do livro-razão com id do pedido, signature, valor em unidades base como string, mint e timestamp, escrita só depois de um claim fresh e de uma verificação aprovada.](assets/v05-annotated-code.webp)

### Spoofs morrem no verificador

Hora de pensar como o atacante, porque o seu endpoint é uma URL pública e JSON é de graça. Duas classes de spoof importam.

Classe um: ficção pura. Um POST com um payload Enhanced bem formado e uma signature que não existe on-chain, ou existe mas é uma transação alheia sem relação nenhuma. O header de auth para a versão preguiçosa disso, que é por que a gente checa ele, mas um segredo compartilhado vaza, fica num dump de configuração ou é adivinhado, então ele é um segurança de porta, não uma prova. A parede de verdade é que o fulfillment exige que o `verify()` passe, e o `verify()` começa por um getTransaction contra a blockchain. Uma signature fictícia não resolve para nada. Uma signature real sem relação falha nas checagens de programa de token, mint, delta ou memo. De um jeito ou de outro o claim faz settle para `rejected` e o livro-razão nunca fica sabendo.

Classe dois, a sutil: um pagamento real, descrito errado. O atacante manda uma transação genuína, talvez 0.01 USDC para a sua ATA de lojista de verdade, depois te manda um POST com um evento no formato Enhanced para aquela signature onde `tokenTransfers` diz 28 USDC e a descrição nomeia a sua prensagem mais cara. Todo campo bate menos os que importam. Se o seu receptor lesse valores do payload, este ataque despacha discos por um centavo. O nosso não lê nada além da signature; o verificador busca a transação real e calcula o delta real, 0.01 USDC, contra os 28 USDC esperados do pedido, e recusa por pagamento a menos. A lição comprime para uma frase que você já deveria conseguir recitar: o payload roteia, a blockchain decide.

É também por isso que o `resolveOrder` no nosso pipeline funciona do jeito que funciona. Mapear uma signature para um pedido pela descrição do payload entregaria rotear E decidir ao atacante. Em vez disso o resolver faz o que o verificador faz: busca a transação e lê o nosso próprio id de pedido a partir do memo on-chain que o seu checkout carimbou no módulo 3. Entrada não confiável pode indicar uma signature para inspeção. É tudo que ela pode fazer.

![Três pistas passando pelos mesmos portões: um spoof fictício morre na verificação, um pagamento real descrito errado morre na checagem de valor, e só o pagamento honesto chega no livro-razão.](assets/v06-diagram.webp)

### O teto, e o que existe depois dele

Webhooks falham de um jeito que a plataforma percebe. Toda entrega que o seu endpoint erra, dá timeout ou responde com 500 conta contra você, e o guarda-corpo é publicado e automático: sustente uma taxa de falha de 95 por cento ou mais ao longo de 7 dias num plano pago e a Helius desativa o webhook (planos gratuitos são julgados numa janela de 24 horas). Isso não é punição por um deploy ruim; 95 por cento ao longo de uma semana é um endpoint que está efetivamente morto. O auto-disable é a plataforma se recusando a fazer DDoS no seu cadáver. As suas defesas são as que você já construiu: dê ack rápido para lentidão não ser lida como falha, mantenha o handler magro, e monitore as estatísticas de entrega do webhook no painel do mesmo jeito que você observaria a taxa de erro de um endpoint da Stripe.

Faça a economia enquanto a gente está aqui, porque ela decide a arquitetura com mais honestidade do que o gosto decide. A entrega custa 1 crédito por evento. Uma loja de discos fazendo até mil vendas por dia gasta mil créditos em ingestão, erro de arredondamento contra o seu uso de RPC, e o webhook só dispara quando uma conta observada de fato se move. Este é o regime para o qual webhooks foram projetados: eventos de baixa frequência e alto valor, onde preço por evento é desprezível e um minuto de latência é invisível. Agora inverta. Indexar toda transferência que toca um programa popular, dezenas de milhões de eventos, exigências de frescor abaixo do segundo: preço de entrega por evento e overhead de um HTTP por evento os dois param de fazer sentido, e nenhuma dose de higiene de retry conserta um descasamento de arquitetura. A resposta errada clássica é um loop de polling martelando faixas de getTransaction, que queima créditos para rebuscar estado quase sempre inalterado e ainda atrasa. A resposta certa é uma inscrição de streaming direto do firehose do validador: Yellowstone gRPC e os parentes dele. Esse mundo, ingestão por gRPC, encanamento de Geyser, backfill, a cadeira inteira de infraestrutura de dados, é território do curso Client-Side Mastery, e a lição de webhook dele pega exatamente onde esta para. A Wavelength não tem esse problema. Um back office de lojista é precisamente a cadeira de operações de lojista, e para ela, o humilde webhook mais a disciplina que você agora tem é a engenharia correta, não a versão iniciante de algo mais sofisticado.

![Linha do tempo de um handler de webhook lento acumulando entregas falhas e retries ao longo de uma semana até a taxa de falha de 95 por cento em sete dias disparar a desativação automática do endpoint.](assets/v07-timeline.webp)

## Lab: construa o backoffice

Como o trabalho de hoje está dividido: eu percorro o servidor e o pipeline com você de ponta a ponta (worked). Você implementa os dois movimentos que esta lição existe para ensinar, o claim da signature e a escrita no livro-razão, contra regras declaradas e com o smoke test como seu juiz (completion). Depois o replay de entrega tripla e um spoof feito à mão são só seus (solo). Feito quer dizer `npx tsx smoke.ts` imprimindo `SMOKE PASS`.

**1. Monte o workspace.** O `backoffice` mora ao lado do seu projeto `verifier` da lição passada para ele conseguir importar o `verify`. Mesma stack de servidor do resto do curso:

```bash
mkdir -p backoffice/src
cd backoffice
npm init -y
npm pkg set type=module
npm install express@5.1.0
npm install -D tsx@4 typescript @types/express @types/node
```

Os pins e as notas de frescor deles: o `express` está fixado em 5.1.0 aqui, mas qualquer 5.x serve e nada neste receptor depende da diferença (a lição de blink instalou a 5.2.1, que é a 5.x atual do npm em 2026-08-22). O `tsx` 4 é o runner que o curso inteiro usa; esta linha de instalação é a instalação dele se a máquina estiver nova. Nenhum SDK da Helius aparece nesta linha de instalação, e este curso nunca instala um: o webhook é criado com um curl e o receptor lê JSON puro de um POST HTTP, então a `HELIUS_API_KEY` da preparação do módulo 2 é a única dependência de Helius que você tem. Esse é justamente o ponto — ingestão é só HTTP, e um receptor de webhook que precisa do SDK de um fornecedor para parsear um corpo de requisição assumiu uma dependência à toa.

**2. Tipos: o contrato que a gente consome e os dois campos que a gente lê.**

```ts
// backoffice/src/types.ts

// The verifier's contract, frozen in the last lesson and restated here
// verbatim so backoffice compiles standalone. The real wiring imports the
// verifier package directly; these shapes must keep matching it exactly.
export interface ExpectedOrder {
  orderId: string;         // the id your txreq builder stamps into the spl-memo
  recipient: string;       // the merchant owner address (base58)
  recipientAta: string;    // the merchant token account for the expected mint
  mint: string;            // the mint you price in (base58)
  amountBaseUnits: bigint; // the exact price, integer base units, never a float
}

export type RejectReason =
  | 'duplicate'
  | 'wrong-token-program'
  | 'wrong-mint'
  | 'underpaid'
  | 'wrong-reference';

export type VerifyResult =
  | { ok: true; reason: 'verified'; signature: string }
  | { ok: false; reason: RejectReason | 'not-found'; signature: string };

export type VerifyFn = (
  signature: string,
  expectedOrder: ExpectedOrder,
) => Promise<VerifyResult>;

// Maps a signature to the open order it claims to pay, by reading OUR memo
// out of the transaction on-chain. Returns undefined when the transaction is
// visible but matches no open order. If the transaction is not visible yet,
// THROW instead: the catch in processEvent releases the claim, and the next
// Helius retry resolves it cleanly.
export type ResolveOrderFn = (signature: string) => Promise<ExpectedOrder | undefined>;

// The minimum we read from a Helius Enhanced event: the signature and the type.
// Everything else in the payload is a hint, never an input to fulfillment.
export interface EnhancedEvent {
  signature: string;
  type: string;
}
```

**3. O registry, o seu primeiro degrau de completion.** As regras estão nos comentários; a implementação é sua. A lição inteira depende de o `claim` fazer o set antes do trabalho, então conquiste isso:

```ts
// backoffice/src/registry.ts

export type SigState = 'processing' | 'fulfilled' | 'rejected';

export class SignatureRegistry {
  private states = new Map<string, SigState>();

  // The idempotency gate. Called BEFORE any verification work.
  // Rule 1: if the signature is already tracked (any state), return 'seen'.
  // Rule 2: otherwise record it as 'processing' and return 'fresh'.
  // The set-before-work order is the whole trick: a duplicate delivery that
  // arrives while the first is still verifying must land on 'seen'.
  claim(signature: string): 'fresh' | 'seen' {
    throw new Error('Your turn: implement the claim per the two rules above.');
  }

  // Terminal states. A settled signature is never processed again.
  settle(signature: string, state: 'fulfilled' | 'rejected'): void {
    this.states.set(signature, state);
  }

  // Transient-failure escape hatch: forget the claim so the NEXT redelivery
  // gets a clean 'fresh'. This is what turns Helius retries from a nuisance
  // into your crash recovery.
  release(signature: string): void {
    this.states.delete(signature);
  }

  stateOf(signature: string): SigState | undefined {
    return this.states.get(signature);
  }
}
```

**4. O livro-razão, o seu segundo degrau de completion.** JSONL append-only; o `record` é seu, o `rows` vem dado porque a conciliação da próxima lição depende do comportamento exato dele:

```ts
// backoffice/src/ledger.ts
import { appendFileSync, existsSync, readFileSync } from 'node:fs';
import type { ExpectedOrder } from './types';

export interface LedgerRow {
  orderId: string;
  signature: string;
  amountBaseUnits: string; // the bigint price serialized; JSON has no bigint
  mint: string;
  fulfilledAt: string; // ISO timestamp
}

// Append-only JSONL. One row = one fulfillment = one real payment.
// Exactly-once is enforced UPSTREAM by the registry claim; the ledger's own
// invariant is that record() is only ever reached through a fresh claim.
export class Ledger {
  constructor(private file: string) {}

  record(order: ExpectedOrder, signature: string): LedgerRow {
    // Rule 1: build a LedgerRow from the ORDER's fields (orderId, mint, and
    //         amountBaseUnits via .toString()) plus the signature. Never from
    //         any webhook payload.
    // Rule 2: fulfilledAt is new Date().toISOString().
    // Rule 3: append the row to this.file as one JSON line ending in '\n',
    //         then return the row.
    throw new Error('Your turn: write the row per the three rules above.');
  }

  rows(): LedgerRow[] {
    if (!existsSync(this.file)) return [];
    return readFileSync(this.file, 'utf8')
      .split('\n')
      .filter((line) => line.length > 0)
      .map((line) => JSON.parse(line) as LedgerRow);
  }
}
```

**5. O receptor, trabalhado.** Leia a colocação do ack e o bloco catch duas vezes; eles são as duas decisões nas quais a seção de teoria gastou mais palavras:

```ts
// backoffice/src/server.ts
import express from 'express';
import type { EnhancedEvent, ResolveOrderFn, VerifyFn } from './types';
import type { SignatureRegistry } from './registry';
import type { Ledger } from './ledger';

export interface BackofficeDeps {
  authSecret: string; // the authHeader value you set at webhook creation
  verify: VerifyFn; // the lesson-1 verifier
  resolveOrder: ResolveOrderFn; // signature -> open order, via the on-chain memo
  registry: SignatureRegistry;
  ledger: Ledger;
  onSettled?: (signature: string) => void; // test hook; unused in production
}

function isEnhancedEvent(value: unknown): value is EnhancedEvent {
  if (typeof value !== 'object' || value === null) return false;
  const v = value as Record<string, unknown>;
  return typeof v.signature === 'string' && typeof v.type === 'string';
}

export function createApp(deps: BackofficeDeps): express.Express {
  const app = express();
  app.use(express.json({ limit: '1mb' }));

  app.post('/webhooks/helius', (req, res) => {
    // Gate 1: the shared secret. A bouncer, not proof.
    if (req.get('authorization') !== deps.authSecret) {
      res.status(401).json({ error: 'bad auth header' });
      return;
    }

    // Helius posts an ARRAY of events. Anything that isn't one is malformed.
    if (!Array.isArray(req.body)) {
      res.status(400).json({ error: 'expected an array of events' });
      return;
    }

    const events = req.body.filter(isEnhancedEvent).filter((e) => e.type === 'TRANSFER');

    // Ack FIRST, work after. A slow answer counts as a failed delivery,
    // and enough failed deliveries kill the webhook.
    res.status(200).json({ received: events.length });

    for (const event of events) {
      void processEvent(deps, event.signature);
    }
  });

  return app;
}

async function processEvent(deps: BackofficeDeps, signature: string): Promise<void> {
  try {
    // The idempotency gate: claim before any work. (Inside the try so a
    // throw here, including the lab's placeholder, fails loudly in the catch
    // instead of tearing the process down as an unhandled rejection.)
    if (deps.registry.claim(signature) === 'seen') return;

    // The payload told us a signature. The CHAIN tells us which order it pays.
    const order = await deps.resolveOrder(signature);
    if (!order) {
      deps.registry.settle(signature, 'rejected');
      deps.onSettled?.(signature);
      return;
    }

    // A webhook is a notification, not proof. The verifier is the proof.
    const result = await deps.verify(signature, order);
    if (result.ok) {
      deps.ledger.record(order, signature);
      deps.registry.settle(signature, 'fulfilled');
    } else if (result.reason === 'not-found') {
      // The last lesson's contract: not-found is transient, not a verdict.
      // At confirmed commitment the transaction can lag a fast webhook.
      // Release, and the next retry re-verifies against a caught-up chain.
      deps.registry.release(signature);
      return;
    } else {
      deps.registry.settle(signature, 'rejected');
    }
    deps.onSettled?.(signature);
  } catch (err) {
    // Transient failure (RPC hiccup, our own bug): log it with its reason so
    // the ops log reads like a diagnosis, then release the claim so the next
    // redelivery retries cleanly. Crashing here without releasing would
    // strand the signature in 'processing' forever.
    console.error(
      `[backoffice] transient failure for ${signature.slice(0, 8)}...:`,
      err instanceof Error ? err.message : err,
    );
    deps.registry.release(signature);
  }
}
```

Percorra a parte trabalhada comigo. O corpo do handler antes do ack faz só trabalho de tempo constante: comparação de header, checagem de array, filtro de formato. Tudo que pode ser lento ou pode falhar mora depois do `res.status(200)`, dentro do `processEvent`, disparado com `void` porque a resposta HTTP não deve nada a ele. O `processEvent` é a seção de teoria virada código: claim, resolve, verify, settle, com `release` no catch como a válvula que cura retry. Um ramo merece uma segunda olhada: o `not-found` do verificador faz release em vez de reject, honrando o contrato da lição passada de que not-found é transitório, já que no commitment `confirmed` um webhook rápido pode ultrapassar a visibilidade do `getTransaction`. A próxima reentrega reverifica contra uma blockchain que já se atualizou. E repare no que está ausente: nenhum campo do evento além de `signature` e `type` é lido em momento algum. O spoof de descrição falsa não tem com o que conversar.

**6. O smoke test.** Esta é a trava de aceitação do lab e o mesmo replay de entrega tripla que você rodou contra o receptor ingênuo, agora com um spoof de carona. Ele stuba o verificador e o resolver para rodar offline; os stubs honram os contratos reais com exatidão:

```ts
// backoffice/smoke.ts
// Triple-delivers a real event and one spoof against the receiver.
// Pass = exactly one ledger row, spoof rejected. Run: npx tsx smoke.ts
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createApp } from './src/server';
import { SignatureRegistry } from './src/registry';
import { Ledger } from './src/ledger';
import type { ExpectedOrder, VerifyResult } from './src/types';

const REAL_SIG = 'RealSig1111111111111111111111111111111111111111111111111111111111111111111111111111111111';
const SPOOF_SIG = 'SpoofSig111111111111111111111111111111111111111111111111111111111111111111111111111111111';

const order: ExpectedOrder = {
  orderId: 'ord-0088',
  recipient: 'WVLmerchantOwner1111111111111111111111111111', // stub base58
  recipientAta: 'WVLmerchantUsdcAta11111111111111111111111111', // stub base58
  mint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  amountBaseUnits: 28_000_000n, // 28 USDC
};

// Stub the chain so the smoke runs offline. The real wiring imports the
// lesson-1 verifier and the on-chain memo resolver instead of these.
const resolveOrder = async (_signature: string): Promise<ExpectedOrder | undefined> =>
  order; // both real event and spoof resolve to this order; the spoof dies at verify, not here
const verify = async (signature: string, _expected: ExpectedOrder): Promise<VerifyResult> =>
  signature === REAL_SIG
    ? { ok: true, reason: 'verified', signature }
    : { ok: false, reason: 'wrong-reference', signature };

const registry = new SignatureRegistry();
const ledger = new Ledger(join(mkdtempSync(join(tmpdir(), 'backoffice-')), 'ledger.jsonl'));

const settled = new Set<string>();
const app = createApp({
  authSecret: 'wavelength-webhook-secret',
  verify,
  resolveOrder,
  registry,
  ledger,
  onSettled: (sig) => settled.add(sig),
});

const server = app.listen(0, async () => {
  const address = server.address();
  if (address === null || typeof address === 'string') throw new Error('no port');
  const url = `http://127.0.0.1:${address.port}/webhooks/helius`;

  const post = (signature: string, auth = 'wavelength-webhook-secret') =>
    fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json', authorization: auth },
      body: JSON.stringify([{ signature, type: 'TRANSFER' }]),
    });

  // 1. Triple delivery of the same real event: Helius retry behavior, replayed.
  for (let i = 0; i < 3; i++) {
    const res = await post(REAL_SIG);
    if (res.status !== 200) throw new Error(`delivery ${i + 1}: expected 200, got ${res.status}`);
  }
  // 2. A spoofed event whose signature fails on-chain verification.
  await post(SPOOF_SIG);
  // 3. A delivery with the wrong auth header never even reaches processing.
  const unauth = await post(REAL_SIG, 'wrong-secret');
  if (unauth.status !== 401) throw new Error(`expected 401 for bad auth, got ${unauth.status}`);

  // Wait for the async processing to settle both signatures.
  for (let i = 0; i < 100 && settled.size < 2; i++) {
    await new Promise((r) => setTimeout(r, 10));
  }

  const rows = ledger.rows();
  if (rows.length !== 1) throw new Error(`expected exactly 1 ledger row, found ${rows.length}`);
  if (rows[0].signature !== REAL_SIG) throw new Error('ledger row carries the wrong signature');
  if (registry.stateOf(SPOOF_SIG) !== 'rejected') throw new Error('spoof was not rejected');
  if (registry.stateOf(REAL_SIG) !== 'fulfilled') throw new Error('real payment not fulfilled');

  console.log('backoffice: triple-delivered webhook -> exactly-once ledger row; spoofed event rejected');
  console.log('SMOKE PASS');
  server.close();
});
```

Rode:

```bash
npx tsx smoke.ts
```

Com os dois throws de placeholder ainda no lugar, o smoke falha na contagem de linhas do livro-razão (cada throw é capturado, logado com a mensagem `Your turn` dele pelo bloco catch do receptor, e o claim liberado, então nenhuma linha chega a ser escrita), que é o lab te dizendo que os degraus de completion são genuinamente seus. Quando o seu `claim` e o seu `record` estiverem certos, ele imprime a linha de aprovação. Ligue `npm run verify:backoffice` a este script no `package.json` (`"verify:backoffice": "tsx smoke.ts"`), para o verify de cada degrau continuar rodável pelo nome — o hábito que se paga quando a bancada de jornada do capstone sacode a stack montada e você precisa rechecar um degrau isolado.

![Comparação entre o verificador, o resolver e o livro-razão temporário stubados do smoke test e a ligação ao vivo na devnet, com o registry de signatures idêntico dos dois lados.](assets/v08-comparison.webp)

**7. Aponte um webhook de verdade para ele.** O smoke stubou a blockchain; a rodada ao vivo precisa da ligação real, e o `createApp` só monta o app, então dê um ponto de entrada a ele. Crie `backoffice/src/main.ts`:

```ts
// backoffice/src/main.ts - the live wiring for step 7. Before you pay,
// register the order your checkout is about to mint in OPEN_ORDERS.
import { createSolanaRpc, signature as asSignature } from '@solana/kit';
import { createVerifier } from '../../verifier/src/verify.ts';
import { createMemoryStore } from '../../verifier/src/store.ts';
import { createRpcFetchTransaction } from '../../verifier/src/rpc.ts';
import { createApp } from './server';
import { SignatureRegistry } from './registry';
import { Ledger } from './ledger';
import type { ExpectedOrder } from './types';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// The one open order this live run expects, keyed by orderId. A toy on
// purpose: next lesson replaces it with a real open-orders store.
const OPEN_ORDERS = new Map<string, ExpectedOrder>();

// The real resolver: fetch the transaction, read OUR order id out of the
// on-chain memo (wavelength:<orderId>:<description>), map it to an open
// order. Throws when the tx is not visible yet, so processEvent's catch
// releases the claim and the next Helius retry resolves it cleanly.
async function resolveOrder(sig: string): Promise<ExpectedOrder | undefined> {
  const tx = await rpc
    .getTransaction(asSignature(sig), {
      encoding: 'jsonParsed',
      maxSupportedTransactionVersion: 1,
      // Match the verifier's commitment: the default here is `finalized`,
      // which would make every fresh payment "not visible yet" for ~10 extra
      // seconds and lean on the retry loop to paper over the lag.
      commitment: 'confirmed',
    })
    .send();
  if (!tx) throw new Error('transaction not visible yet');
  for (const ix of tx.transaction.message.instructions) {
    const p = ix as { program?: string; parsed?: unknown };
    if (p.program === 'spl-memo' && typeof p.parsed === 'string') {
      const order = OPEN_ORDERS.get(p.parsed.split(':')[1] ?? '');
      if (order) return order;
    }
  }
  return undefined; // visible, but matches no open order
}

const app = createApp({
  authSecret: process.env.WEBHOOK_SECRET ?? 'wavelength-webhook-secret',
  verify: createVerifier({
    fetchTransaction: createRpcFetchTransaction(),
    store: createMemoryStore(),
  }),
  resolveOrder,
  registry: new SignatureRegistry(),
  ledger: new Ledger('orders.jsonl'), // lands at backoffice/orders.jsonl; later modules read this exact path
});
app.listen(4000, () => console.log('backoffice listening on :4000'));
```

Acrescente o seu pedido ao `OPEN_ORDERS` (o id de pedido que o seu checkout vai carimbar, o seu endereço e a sua ATA de lojista, o mint, o preço exato em unidades base), suba com `npx tsx src/main.ts`, e exponha a porta 4000 por um túnel HTTPS (qualquer túnel; se você não tiver nenhum instalado, `npx localtunnel --port 4000` é uma opção sem configuração). Depois rode o curl de criação da seção de teoria com `webhookType: "enhancedDevnet"`, a URL do seu túnel e a sua ATA de lojista na devnet em `accountAddresses`. Depois registre uma venda no checkout do módulo 3, pagando a partir de `/tmp/customer.json` do jeito que o degrau de completion daquela lição faz (`PAYER_KEYFILE=/tmp/customer.json npm run --workspace transfer-kit pay -- $(solana address) 12.5 <ref>`) — o lojista é quem está sendo pago, então o lojista não pode ser quem paga — e veja um evento Enhanced de verdade chegar, dar claim, resolver, verificar contra a devnet e aterrissar uma linha no livro-razão. A página de webhooks do painel mostra a entrega de um jeito ou de outro, que é a sua janela de depuração quando o túnel cair.

## Challenge

**O degrau de completion** está para trás se o smoke passar: o seu `claim` e o seu `record`, julgados por entrega tripla.

**O degrau solo, duas partes, sem passo a passo.**

Primeiro, replay hostil. Estenda o `smoke.ts` (ou escreva `attack.ts` ao lado dele) para cobrir os dois casos que o smoke básico não cobre: as três duplicatas do evento real dentro de UM array de entrega, o que exercita claim antes do trabalho dentro de um lote só, e um treino de recuperação de crash onde o seu stub de `verify` lança na primeira chamada e passa na segunda, provando que uma reentrega depois do `release` atende exatamente uma vez. Aceite: uma linha no livro-razão nos dois casos, e o estado do registry no treino terminando em `fulfilled`.

Segundo, um spoof de verdade contra a ligação de verdade. Na devnet, mande uma transferência genuína para a sua ATA de lojista com um valor de token bem abaixo de qualquer pedido aberto, fabrique à mão um evento no formato Enhanced para aquela signature real alegando um pagamento de preço cheio, e mande ele por POST no seu receptor com o header de auth correto. Aceite: o verificador recusa pelo delta on-chain, o registry lê `rejected`, e o livro-razão não ganhou nada. Se o seu spoof de algum jeito for atendido, não conserte o spoof. Conserte o receptor, porque aquele buraco era real.

## Checkpoint, e o que o livro-razão não consegue te contar

Se o smoke falhar na contagem de linhas, o seu `claim` está checando depois do trabalho em vez de antes, ou o `record` está escrevendo mais do que mandaram; os dois ficam visíveis em menos de um minuto lendo o seu próprio diff contra as regras. Se o passo do webhook real não entregar nada, é quase sempre a URL do túnel ou a ATA em `accountAddresses`, nessa ordem, e o log de entregas do painel decide qual. Quando passar, repare no que você agora tem nas mãos, porque é mais do que um handler de webhook: retries, crashes, duplicatas e duas classes inteiras de spoof colapsam todas numa invariante silenciosa, uma linha de livro-razão por pagamento real. Essa invariante é a diferença entre uma demo e um back office.

Mas sente um pouco com o livro-razão e os pontos cegos dele encaram você de volta. Ele sabe o que foi pago e verificado, e nada mais. Um cliente que pagou um dólar a mais: uma linha, o pagamento a mais invisível. Um pagamento que aterrissou on-chain enquanto o seu webhook estava desativado: nenhuma linha, e o dinheiro continua sendo seu, sem registro. E em lugar nenhum de nada que você construiu existe um jeito de mandar dinheiro de volta. O livro-razão registra a verdade; ele não consegue perceber as verdades que está deixando de fora, e não consegue desfazer nenhuma. A próxima lição é conciliação, varrendo a blockchain contra este arquivo para achar cada descasamento, e depois o fluxo de reembolso que nenhuma documentação oficial vai te ensinar. O trilho não tem chargebacks, então a gente constrói o devolver por conta própria. Vejo você lá.
