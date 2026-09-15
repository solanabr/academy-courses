# Um blink para o drop: actions que executam em qualquer lugar (que as renderize)

Na lição passada, o `pos-stall` levou o mesmo endpoint de transaction request para o outro lado de uma mesa com um QR. O endpoint já vendeu um disco por uma página e por uma barraca. Hoje ele vende por um link que você pode colar em qualquer lugar, e a gente fica honesto sobre o que "qualquer lugar" quer dizer.

Antes de a gente construir qualquer coisa nova, prove que o core ainda responde. Suba o seu servidor checkout-txreq da lição de transaction request, com a mesma linha `MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts` que você rodou lá, e bata direto no POST dele (ele escuta na 3100 em `/txreq`; troque pela sua própria porta e rota se você mudou elas):

```bash
curl -s -X POST http://localhost:3100/txreq \
  -H 'Content-Type: application/json' \
  -d '{"account":"'$(solana address)'"}'
```

Você deve receber de volta um JSON com uma transação em base64 dentro. Olhe essa resposta por um segundo. Uma carteira dá POST em `{account}`, o seu servidor precifica o pedido e devolve uma transação pronta para assinar. É esse o truque inteiro desta lição: a spec Actions é esse mesmo par requisição-resposta, mais uma camada de metadados para qualquer superfície conseguir renderizar um botão em volta dele. Você já construiu a parte difícil.

## Resumo

Esta lição transforma o seu core de pagamento em um link. Não um link para uma loja: um link que É a loja. As descobertas logo de cara, porque algumas delas não são o que o marketing de 2024 prometeu:

- Você entrega o **drop-blink**: um `actions.json` na raiz do seu domínio, um endpoint GET devolvendo os metadados da action e um endpoint POST devolvendo um `ActionPostResponse` em conformidade com a spec. A transação dentro dele vem exatamente do builder que você escreveu na lição de transaction request. Zero código de pagamento novo.
- Três regras de hospedagem decidem se alguma carteira vai um dia carregar a sua action: o `actions.json` fica na raiz do domínio, toda rota de action manda `Access-Control-Allow-Origin: *`, e o `actions.json` também. Esqueça qualquer uma delas e você ganha a falha clássica de "funciona no curl, morto numa carteira".
- O ferramental está congelado: `@dialectlabs/blinks` 0.22.5 (publicado em 2025-04-04) e `@solana/actions` 1.6.6 (publicado em 2024-11-05) ainda são as versões mais novas que existem em 2026-08-22. Este curso não instala nenhum dos dois; você constrói contra o contrato de wire via `@solana/actions-spec` 2.4.2, e se um dia você adotar os SDKs, fixe exatamente essas versões com uma nota de obsolescência.
- Onde os blinks de fato renderizam em 2026 é incerto. A renderização no X é mediada por extensão do Chrome, não nativa. Então o gate do lab é conformidade com a spec mais um client local, e as suas afirmações de alcance pertencem a uma caixa de "verificado na escrita", não a um pitch deck.

![O único builder de transaction request da lição de checkout alimenta três superfícies, a página de checkout, a barraca da maquininha (POS) e agora o blink do drop.](assets/v01-diagram.png)

## O protocolo de actions, de perto

### Do link de pagamento ao protocolo

Se você já publicou alguma coisa com Stripe, você já fez um Payment Link: uma URL que codifica "venda esta coisa", que os servidores da Stripe transformam numa página de checkout hospedada. Uma Solana Action é essa ideia com a renderização destacada. O seu servidor descreve o checkout (metadados) e constrói a transação (o POST que você já tem). Qualquer superfície que exiba o link, uma carteira, um feed, um client de chat, um site intersticial, fica livre para renderizar o próprio botão de compra a partir dos seus metadados e executar a compra ali mesmo. Um **blink** (blockchain link) é a forma renderizada: a URL mais o client que desdobrar ela em UI.

O protocolo são dois verbos em uma URL, mais um arquivo de descoberta:

1. **GET** na URL da action: devolve metadados. Ícone, título, descrição, label e, opcionalmente, uma lista de subactions parametrizadas. Isto é tudo que um client precisa para desenhar o botão.
2. **POST** de `{account}` na mesma URL: devolve um `ActionPostResponse` carregando uma transação codificada em base64 para aquele usuário específico assinar. O mesmo contrato do seu transaction request, e isso não é coincidência: a spec Actions generaliza o fluxo de transaction request do Solana Pay que você já implementou.
3. **`actions.json`** na raiz do seu domínio: diz aos clients quais caminhos do seu domínio são actions, para que um link puro para o seu site possa ser mapeado até o endpoint de action dele.

![Fluxo desde colar um blink, passando pela descoberta via actions.json, os metadados do GET, o botão renderizado, o POST com account, a transação assinada e o passo de agradecimento encadeado por links.next, com CORS e actions.json como gates de falha.](assets/v02-flowchart.png)

Por que isso importa para uma loja de discos? Distribuição. Todo checkout até aqui exigia que o cliente viesse até você: a sua página, a sua barraca. Um blink inverte isso. A loja viaja para onde a conversa já está. Para uma prensagem limitada de 200 cópias, a diferença entre "clique para o nosso site" e "compre bem aqui" é conversão que dá para sentir. Esse era o pitch de 2024, e o pitch era bom. Segure esse pensamento, porque a seção da realidade de 2026 lá embaixo é onde a gente precifica isso com honestidade.

### GET: os metadados que viram uma UI

Este é o formato que um GET precisa devolver, espelhado de `@solana/actions-spec` 2.4.2 (o pacote de tipos; ele é a fonte da verdade da spec e você pode importar estes daqui em vez de escrever eles):

```ts
// drop-blink/src/types.ts
// Mirrored from @solana/actions-spec 2.4.2, trimmed to the fields this lesson
// exercises; the spec package carries more (optional `type` on the top-level
// action, an `error` field, parameter pattern/min/max, a wider LinkedActionType
// union). Diff against node_modules and you will find those extras on the spec
// side — deliberate omissions, not drift. The package is frozen alongside the
// rest of the tooling; these shapes are the live contract blink clients check.

export interface ActionParameter {
  name: string;      // the template variable this fills, e.g. {qty}
  label?: string;    // placeholder text the client shows
  required?: boolean;
  type?:
    | 'text' | 'number' | 'email' | 'url' | 'date'
    | 'datetime-local' | 'textarea' | 'checkbox' | 'radio' | 'select';
  options?: Array<{ label: string; value: string; selected?: boolean }>;
}

export interface LinkedAction {
  type: 'transaction'; // this action's POST returns a transaction to sign
  href: string;        // relative or absolute; may carry {param} templates
  label: string;       // the button text
  parameters?: ActionParameter[];
}

export interface ActionGetResponse {
  type: 'action';
  icon: string;        // absolute URL, not a path; clients will not resolve relatives
  title: string;
  description: string;
  label: string;       // fallback button text when links.actions is absent
  disabled?: boolean;
  links?: { actions: LinkedAction[] };
}
```

Percorra campo a campo, porque todo campo é UI estrutural. `icon` precisa ser uma URL **absoluta**; um caminho relativo renderiza como imagem quebrada em todo client, e é de longe o bug cosmético mais comum em actions publicadas. `label` é o fallback de um botão só; `links.actions` substitui ele por vários botões quando está presente. E `parameters` é como um botão ganha um input: um `href` igual a `/api/actions/drop?qty={qty}` mais um parâmetro chamado `qty` diz ao client para renderizar um campo numérico e substituir o valor na URL antes de dar POST. O `type` de um parâmetro é uma dica de renderização (`select` e `radio` carregam `options`); clients que não reconhecem um type caem de volta para texto. Validação de entrada fica no seu servidor. Sempre. Os types de parâmetro estilizam um formulário, eles não protegem você do que chega na query string.

Aí tem o `disabled`, o campo que um blink de comércio de fato exercita. Os metadados são buscados ao vivo a cada renderização, o que dá a um blink uma propriedade que nenhuma página de loja estática tem: o link se atualiza sozinho em todo lugar onde ele já foi postado. Quando a cópia 200 da prensagem vender, o seu GET começa a devolver `disabled: true` com uma descrição reescrita dizendo esgotado, e todo card já parado em todo post antigo deixa o botão dele cinza na próxima renderização. Ninguém edita um tweet, ninguém sai atrás de um link velho. Esgotado vira um estado que o seu endpoint reporta, não um 404 que você torce para as pessoas notarem. Para um drop, esse booleano sozinho é metade do argumento do protocolo inteiro: marketing de escassez funciona exatamente quando o artefato que diz "acabou" é o mesmo artefato que dizia "compre".

### POST: {account} entra, transação sai

O lado do POST você conhece. O body é `{account}`, a chave pública em base58 do cliente. O POST de transaction request devolve uma transação codificada em base64, e a spec Actions embrulha esse mesmo payload em um envelope nomeado:

![Três formatos JSON de ActionPostResponse, o mínimo com apenas type e transaction, um acrescentando message, um acrescentando links.next, anotados com a regra de que chaves opcionais são omitidas por completo quando ausentes.](assets/v03-annotated-code.png)

Duas adições em cima do endpoint que você já tem. `message` é uma string humana opcional que a carteira pode mostrar depois de assinar, a sua confirmação de pedido em miniatura. `links.next` é **encadeamento de actions**: um objeto `{ type: 'post', href }` dizendo ao client "depois que esta transação confirmar, dê POST aqui para o próximo passo". O POST encadeado inclui a signature confirmada, o que faz dele o lugar natural para um cartão de agradecimento, um passo de resgate ou a próxima action em um fluxo de vários passos. A gente vai usar ele para uma tela de agradecimento que ecoa o recibo.

Mais uma feature da spec para conhecer pelo nome: **action identity**. Um provedor de action pode anexar uma instrução SPL Memo no formato `solana-action:<identity>:<reference>:<signature>`, onde a identity é um keypair que assina a reference. Ela existe para que indexadores e registros consigam atribuir transações on-chain de volta à action que produziu elas. Você não precisa dela para o lab, e o seu builder já carimba o memo do pedido e a reference key que a sua própria conciliação usa. Arquive isso em "que memo esquisito é esse quando você vê um num explorer".

De onde vem a transação em si? De `buildOrderTransaction`, sem mudança. Este é o acúmulo que o módulo inteiro vinha escalando, então deixa eu dizer sem rodeio: o blink acrescenta uma camada de metadados e um envelope de resposta. A precificação, o memo, a reference key, a transferência segura nos decimais, tudo isso é o caminho de código da lição de transaction request, importado. Se você se pegar reescrevendo a montagem da transação dentro de um handler de blink, pare; você está bifurcando a sua lógica de pagamento em uma segunda cópia que vai derivar.

### Descoberta e CORS: as duas regras que matam blinks

Agora a parte que gera mais threads de suporte. Os seus endpoints podem estar perfeitos e nenhuma carteira vai carregar eles nunca, porque blinks são carregados **cross-origin**. O client que renderiza o seu blink vive no domínio de outra pessoa, então o navegador impõe CORS em toda requisição para o seu, e a descoberta acontece através de um arquivo que você talvez tenha esquecido de servir.

Regra um: o `actions.json` mora na raiz do domínio. `https://shop.example/actions.json`, não `/api/actions.json`, não atrás de um redirect para um caminho. Ele mapeia padrões de URL no seu domínio para caminhos de API de action:

```json
{
  "rules": [{ "pathPattern": "/api/actions/**", "apiPath": "/api/actions/**" }]
}
```

Os padrões são globs: `*` casa dentro de um segmento de caminho, `**` casa em qualquer profundidade. E esse mapeamento se paga. Uma regra pode emparelhar uma página humana com a action atrás dela, digamos `pathPattern: "/drop/**"` em `apiPath: "/api/actions/drop/**"`, para que um cliente que cole a URL comum da página de produto da prensagem ainda receba um botão de compra renderizado, porque o client resolveu a página até a action dela. Se URLs de action só fossem coladas diretamente, a descoberta poderia morar na própria URL; o `actions.json` existe para que os seus links normais também virem blinks.

Regra dois: toda resposta de action manda `Access-Control-Allow-Origin: *`. Inclusive, e essa é a que todo mundo esquece, no próprio `actions.json`. O fetch de descoberta também é cross-origin. O curl não impõe CORS, navegadores impõem, que é exatamente por que a cara da falha é um endpoint que testa limpo no seu terminal e não mostra nada numa carteira.

![Tabela comparativa de quatro falhas de hospedagem, actions.json faltando na raiz, CORS faltando nas rotas, CORS faltando no actions.json, URL de ícone relativa, cada uma funcionando no curl e quebrada em um client de verdade.](assets/v04-comparison.png)

Preflight também importa: clients mandam OPTIONS antes do POST, então o seu middleware de CORS responde OPTIONS com os mesmos headers e um 204 vazio. A spec também define dois headers informativos de resposta, `X-Action-Version` (a versão da spec que você implementa) e `X-Blockchain-Ids` (um chain id CAIP-2; CAIP-2 é o padrão de nomenclatura cross-chain de namespace mais reference, aqui `solana:` mais o hash de genesis truncado em 32 caracteres, então a devnet é `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1`). Clients em conformidade leem eles para decidir compatibilidade; mandar eles custa duas linhas.

### O teste de realidade: onde isto de fato renderiza?

Hora de precificar o trade-off, porque eu te vendi o sonho duas seções atrás e você merece a fatura.

Blinks foram lançados em meados de 2024 com uma demo que grudou na cabeça de todo mundo: um link se desdobrando em um botão de compra num feed do X. Eu repeti esse pitch para uma sala de lojistas na época, com convicção total. A convicção sobreviveu aos fatos. O que de fato aconteceu é que a renderização no X era, e continua sendo, mediada por uma extensão do navegador Chrome: quem vê sem a extensão vê um link puro, não um botão. A demo do feed era real, a implicação de "para todo mundo que vê" não era.

E o histórico do ferramental conta a própria história. Esta é a linha do tempo de releases, que você pode verificar no npm em trinta segundos:

![Linha do tempo do lançamento dos blinks em 2024 até os últimos releases dos SDKs, o pivô da Dialect para uma biblioteca hospedada e a data de escrita em 2026 sem nenhuma versão mais nova publicada.](assets/v05-timeline.png)

O `@solana/actions` não publica desde 2024-11-05. O `@dialectlabs/blinks` não publica desde 2025-04-04. Dezesseis meses de silêncio do SDK de client não são uma lacuna de manutenção que você contorna, são um sinal sobre para onde a atenção do fornecedor foi: a Dialect pivotou para uma Standard Blinks Library hospedada, um serviço gerenciado, em vez do SDK aberto. Então este curso constrói contra o contrato de wire em vez dos SDKs congelados; se algum projeto seu de fato adotar eles, fixe exatamente as duas versões congeladas que o Resumo nomeia, escreva a nota de obsolescência no comentário do seu package.json ou no README, e trate "esperar o próximo release" como não sendo um plano.

A Standard Blinks Library hospedada pode parecer a saída do problema do SDK congelado: deixe a Dialect rodar o blink por você. Duas razões para construir o seu próprio endpoint mesmo assim. Primeira, o seu drop é o seu estoque, a sua precificação, a sua conciliação de memo-e-reference; um serviço hospedado é a casa errada para a lógica de pagamento em que o seu back office inteiro se apoia, e este curso vem construindo essa lógica em um único caminho de código próprio há três lições. Segunda, e essa é a razão durável: o artefato desta lição é conformidade com a spec, e nenhum host consegue ser dono disso por você. A spec Actions é um contrato de wire, metadados no GET, POST de `{account}` na saída; um SDK congelado não muda o contrato que os seus endpoints falam, e qualquer renderizador construído ano que vem contra a mesma spec executa a sua loja sem você publicar uma linha. Você está construindo contra o protocolo, não contra o roadmap da Dialect. Essa é a dependência correta de assumir com um fornecedor cujo SDK está em silêncio há dezesseis meses.

Aí tem o registro. A Dialect opera um registro de blinks onde as actions carregam um status, e a documentação define os três de uma vez só: **trusted** é "registrada pelo desenvolvedor e aceita pelo comitê de registro" e renderiza por completo nos clients participantes, **none** quer dizer que "a action não foi registrada" e normalmente renderiza com avisos ou UI degradada, e **blocked** foi "sinalizada como maliciosa pela comunidade de registro" e não renderiza. Se registrar não é uma chamada de API que você faz: a rota documentada da Dialect é um envio por e-mail, e a documentação diz sem rodeio que "atualmente a revisão de registro é um processo manual". Ler o registro programaticamente é outra história, e em parte com gate por chave: a lista pública em `registry.dial.to/v1/list` responde para qualquer um, enquanto o endpoint de consulta por URL devolve 403 sem uma chave de API da Dialect (os dois sondados em 2026-08-22). Você também vai ler afirmações sobre quando a imposição do registro começou ou começa; essa data não tem fonte, então este curso não cita nenhuma, e você também não deveria. O que produz um fato operacional duro: você não pode pôr o gate de um lançamento, nem desta lição, na fila manual de um terceiro.

![Diagrama do fluxo do registro da Dialect: um envio por e-mail entra em revisão manual, e os status trusted, none e blocked mapeiam para renderização completa, degradada e recusada.](assets/v06-diagram.png)

Então com quais superfícies você pode de fato contar? Esta é a caixa honesta.

> **Verificado na escrita, 2026-08-22.** A renderização no X é mediada por extensão do Chrome, não nativa. Os SDKs centrais estão congelados em `@dialectlabs/blinks` 0.22.5 e `@solana/actions` 1.6.6. A revisão do registro da Dialect é um processo manual alcançado por e-mail, e a API de consulta por URL dela tem gate por chave. Além disso, a lista de 2026 de carteiras e superfícies que renderizam blinks nativamente é **não verificada**: as páginas de ecossistema que nomeiam carteiras específicas datam da onda de lançamento de 2024, e a gente não conseguiu confirmar o comportamento atual de nenhuma carteira específica no momento da escrita. Trate toda afirmação de "renderiza na carteira X" que você ler, incluindo versões antigas de afirmações como estas, como velha até você testar naquela carteira, naquela semana.

O trade-off, dito uma vez e carregado para tudo que você construir hoje: um blink transforma qualquer superfície em um checkout, mas ele só executa onde alguma coisa renderiza ele. O ferramental congelou em 2025, o X precisa de uma extensão, a admissão no registro é uma revisão manual que você não consegue agendar. "Renderiza em todo lugar" é falso. O que é verdade, e segue genuinamente valioso, é mais estreito: um blink é um endpoint de checkout autodescritivo e em conformidade com a spec. Qualquer superfície atual ou futura que fale a spec consegue executar a sua loja. Você está comprando uma opção sobre distribuição, barato, porque o custo marginal em cima do endpoint que você já tem é uma tarde. É um bom negócio desde que você precifique como uma opção e não como um feed prometido. E tem uma segunda razão para o padrão ter pernas: o próprio repositório canônico do Solana Pay agora abre com um CLI de agentic payments cujo README chama x402 e MPP de "dois padrões de pagamento vivos na Solana" (os dois são padrões para pagar por HTTP puro, construídos para compradores máquina; o módulo 7 ensina eles direito), enquanto a biblioteca clássica de checkout segue viva em um subdiretório (README do repo e redirects, rechecados em 2026-08-22). A aposta do ecossistema é que coisas que executam pagamentos de onde quer que estejam postadas, para humanos ou para agentes, são a direção da viagem. Blinks são a ponta voltada para humanos dessa mesma mudança, e a sua ponta voltada para máquinas chega no módulo 7.

## Lab: entregue o blink do drop

A divisão de hoje: eu percorro o apoio, os endpoints e as regras de hospedagem com você (worked). Você monta o builder do `ActionPostResponse` sozinho contra três regras declaradas, com o desafio de código como o seu verificador (completion). Aí a compra pelo client local e o checklist pronto para o registro são só seus (solo). No fim, `npx tsx drop-blink/smoke.ts` passa e um client local de blinks completa uma compra na devnet da prensagem em destaque.

**1. Gere o scaffold do workspace.** O blink do drop mora ao lado do seu projeto checkout-txreq para poder importar o builder. Mesma linha do kit do workspace checkout-txreq (`@solana/kit` 6.10.0, o último release da v6; este workspace fica na v6 porque o builder que ele importa fica):

```bash
mkdir -p drop-blink/src drop-blink/public
cd drop-blink
npm init -y
npm pkg set type=module
npm install express@5.2.1 @solana/kit@6.10.0 @solana/actions-spec@2.4.2
npm install -D tsx@4 typescript @types/express @types/node
```

Os pins, com as notas de frescor: `express` 5.2.1 é o `latest` do npm na linha 5.x no momento da escrita (rechecado em 2026-08-22; qualquer 5.x funciona). `@solana/actions-spec` 2.4.2 é o mais novo e, como o resto do ferramental de blink, está congelado; a gente instala ele para você poder dar diff nos tipos espelhados à mão contra a fonte da verdade. O `tsx` é o runner de TypeScript que você usa o curso inteiro; se esta for uma máquina nova, a linha de dev-install acima é a instalação dele.

Checkpoint: `npm ls @solana/actions-spec express` imprime 2.4.2 e um express 5.x, sem avisos de peer não atendida. Uma reclamação de peer sobre `@solana/kit` aqui quer dizer que você derivou da linha v6 em que vive o builder que você está prestes a importar.

**2. Crie os tipos.** Salve o arquivo `ActionParameter` / `LinkedAction` / `ActionGetResponse` da seção de teoria como `drop-blink/src/types.ts`, e acrescente os formatos do lado do POST:

```ts
// drop-blink/src/types.ts (continued)

export interface ActionPostRequest {
  account: string; // base58 public key of the user who will sign
}

export interface NextActionLink {
  type: 'post';
  href: string; // the client POSTs here after the transaction confirms
}

export interface ActionPostResponse {
  type: 'transaction';
  transaction: string; // base64-encoded transaction
  message?: string;    // optional post-sign confirmation text
  links?: { next: NextActionLink };
}

export interface ActionsJson {
  rules: Array<{ pathPattern: string; apiPath: string }>;
}
```

**3. O builder da resposta, o seu degrau de completion.** Esta é a função que o handler do POST vai chamar, e a única coisa neste lab que você escreve sem mim. O contrato é exatamente o do desafio de código, então você pode conferir o seu trabalho lá antes de plugar ele aqui:

```ts
// drop-blink/src/action-post-response.ts
import type { ActionPostResponse } from './types';

export function buildActionPostResponse(
  transactionBase64: string,
  message?: string | null,
  nextActionHref?: string | null,
): ActionPostResponse {
  // Rule 1: always set type: 'transaction' and transaction from transactionBase64.
  // Rule 2: add message ONLY when message is provided (not null, not undefined);
  //         the minimal response has no message key at all, not a message key
  //         set to undefined.
  // Rule 3: add links.next as { type: 'post', href } ONLY when nextActionHref
  //         is provided; otherwise the response has no links key.
  throw new Error('Your turn: assemble the response per the three rules above.');
}
```

Os argumentos são posicionais, e os dois opcionais podem chegar como `null` ou serem omitidos, então trate `null` e `undefined` igualmente como "ausente". É exatamente assim que o verificador do desafio chama ela: `buildActionPostResponse('B64', null, 'https://.../thanks')` é uma resposta com uma action encadeada e sem message.

Por que tanto rigor com ausente-versus-undefined? Porque clients validam o formato da resposta, e uma chave `links` segurando lixo falha na validação onde nenhuma chave `links` passa. Construa o objeto condicionalmente; não construa ele no máximo e apague depois.

Checkpoint: como vem, este arquivo lança `Your turn: assemble the response per the three rules above.` em toda chamada. Esse throw é o degrau de completion esperando por você, e a rodada de smoke do passo 8 é onde ele aparece.

**4. CORS e o esqueleto do servidor.** Um middleware só, aplicado antes de toda rota, respondendo o preflight:

```ts
// drop-blink/src/server.ts
import express from 'express';
import type { Request, Response, NextFunction } from 'express';
import { buildOrderTransaction } from '../../checkout-txreq/src/build-order-transaction';
import { buildActionPostResponse } from './action-post-response';
import type { ActionGetResponse, ActionPostRequest, ActionsJson } from './types';

const app = express();
app.use(express.json());

const ACTION_HEADERS: Record<string, string> = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET,POST,PUT,OPTIONS',
  'Access-Control-Allow-Headers':
    'Content-Type, Authorization, Content-Encoding, Accept-Encoding',
  'Access-Control-Expose-Headers': 'X-Action-Version, X-Blockchain-Ids',
  'X-Action-Version': '2.4.2',
  'X-Blockchain-Ids': 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1', // devnet CAIP-2
};

app.use((req: Request, res: Response, next: NextFunction) => {
  res.set(ACTION_HEADERS);
  if (req.method === 'OPTIONS') {
    res.status(204).end();
    return;
  }
  next();
});

// Static AFTER the CORS middleware, never before it: the icon is fetched
// cross-origin too, and a static route that matches first would ship it
// without the headers. This ordering is the works-in-curl table's row one.
app.use(express.static('public'));
```

O caminho de import de `buildOrderTransaction` presume o layout de módulos da lição de transaction request sentado um diretório ao lado. Se você deixou a montagem da transação inline no handler do POST daquela lição em vez de extrair ela para uma função, tire cinco minutos agora e extraia. O blink é exatamente o motivo: um builder, muitas superfícies. O contrato dele aqui é o mesmo em que o seu capstone também vai se apoiar: ele recebe o account do comprador mais o que ele está comprando, precifica no servidor, carimba o memo e a reference, e resolve para a transação em base64.

**5. Descoberta e o ícone.** O arquivo da raiz e uma imagem estática (jogue qualquer PNG quadrado em `public/drop-icon.png`; qualquer arte de placeholder serve):

```ts
// drop-blink/src/server.ts (continued)

const BASE_URL = process.env.BASE_URL ?? 'http://localhost:3000';
const DROP_SKU = 'WVL-045';
const DROP_PRICE_USDC = 30;

app.get('/actions.json', (_req: Request, res: Response) => {
  const payload: ActionsJson = {
    rules: [{ pathPattern: '/api/actions/**', apiPath: '/api/actions/**' }],
  };
  res.json(payload);
});
```

Servido pelo Express na raiz do app, que precisa SER a raiz do seu domínio no deploy. Se o seu site roda atrás de um prefixo de caminho ou de um proxy, o arquivo ainda tem que responder em `https://yourdomain/actions.json`; essa é uma regra de reverse proxy, não uma rota da aplicação, e é o detalhe de deploy mais frequentemente perdido entre "funcionou local" e produção.

**6. O GET: metadados para a prensagem em destaque.**

```ts
// drop-blink/src/server.ts (continued)

app.get('/api/actions/drop', (_req: Request, res: Response) => {
  const payload: ActionGetResponse = {
    type: 'action',
    icon: `${BASE_URL}/drop-icon.png`,
    title: 'Wavelength Records: the August pressing',
    description: `Limited pressing ${DROP_SKU}. ${DROP_PRICE_USDC} USDC on devnet, 200 copies, gone when they are gone.`,
    label: `Buy for ${DROP_PRICE_USDC} USDC`,
    links: {
      actions: [
        {
          type: 'transaction',
          label: `Buy 1 for ${DROP_PRICE_USDC} USDC`,
          href: '/api/actions/drop',
        },
        {
          type: 'transaction',
          label: 'Buy more than one',
          href: '/api/actions/drop?qty={qty}',
          parameters: [
            { name: 'qty', label: 'How many copies (max 5)', required: true, type: 'number' },
          ],
        },
      ],
    },
  };
  res.json(payload);
});
```

Dois botões de um endpoint só: uma compra fixa de um clique, e uma compra por quantidade parametrizada cujo template `{qty}` o client substitui na query string. URL de ícone absoluta. Repare no que NÃO está aqui: nenhuma matemática de preço, nenhuma lógica de estoque. Os metadados descrevem; o POST decide.

**7. O POST e o passo de agradecimento encadeado.** O handler valida a entrada, limita a quantidade no servidor (lembre: types de parâmetro estilizam um formulário, eles não validam), reusa o builder e monta a resposta através da sua função do degrau de completion:

```ts
// drop-blink/src/server.ts (continued)

app.post('/api/actions/drop', async (req: Request, res: Response) => {
  const body = req.body as ActionPostRequest;
  if (!body?.account || typeof body.account !== 'string') {
    res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
    return;
  }
  const qty = Math.min(Math.max(Number(req.query.qty ?? 1) || 1, 1), 5);

  try {
    const { transactionBase64 } = await buildOrderTransaction({
      account: body.account,
      sku: DROP_SKU,
      quantity: qty,
    });
    res.json(
      buildActionPostResponse(
        transactionBase64,
        `Order placed: ${qty}x ${DROP_SKU}. Sign to complete the purchase.`,
        '/api/actions/drop/thanks',
      ),
    );
  } catch (err) {
    res.status(400).json({
      message: err instanceof Error ? err.message : 'Could not build the order transaction',
    });
  }
});

app.post('/api/actions/drop/thanks', (req: Request, res: Response) => {
  const signature =
    typeof req.body?.signature === 'string' ? req.body.signature : undefined;
  res.json({
    type: 'completed',
    icon: `${BASE_URL}/drop-icon.png`,
    title: 'You got the pressing',
    description: signature
      ? `Payment landed. Signature ${signature.slice(0, 8)}... is your receipt.`
      : 'Payment landed. Your order is in.',
    label: 'Done',
  });
});

app.listen(3000, () => {
  console.log('drop-blink listening on :3000');
});
```

O caminho de erro devolve `{ message }` com um status diferente de 200, que é o formato `ActionError` da spec; clients exibem essa message, então escreva ela para o cliente e não para os seus logs. O handler de agradecimento é o alvo do `links.next`: depois que a carteira confirma a transação, o client dá POST aqui com a signature, e um payload `type: 'completed'` fecha o fluxo com um cartão de recibo. O encadeamento vai mais fundo do que a gente leva (um próximo passo pode ser uma action inteira com a própria transação), mas um salto é suficiente para dominar o padrão.

Checkpoint: `MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts` imprime `drop-blink listening on :3000`. A variável de ambiente não é opcional: o builder que você importou continua lendo `MERCHANT_ADDRESS` do ambiente, exatamente como fazia no workspace de origem dele, e sem ela todo POST dá 400 com a própria mensagem de set-MERCHANT_ADDRESS do builder antes de o seu throw de placeholder ser alcançado. Num segundo terminal, `curl -s http://localhost:3000/actions.json` devolve o seu objeto de uma regra só e `curl -s http://localhost:3000/api/actions/drop` devolve os metadados com os dois labels de botão dentro. Espera-se que o POST falhe por enquanto, no throw de placeholder do passo 3.

**8. Rode o smoke.** O harness de verificação deste artefato, e o seu checkpoint:

```ts
// drop-blink/smoke.ts
import { generateKeyPairSigner } from '@solana/kit';

const BASE = process.env.BLINK_URL ?? 'http://localhost:3000';

function fail(msg: string): never {
  console.error(`SMOKE FAIL: ${msg}`);
  process.exit(1);
}

async function main() {
  const testAccount =
    process.env.TEST_ACCOUNT ?? (await generateKeyPairSigner()).address;

  const aj = await fetch(`${BASE}/actions.json`);
  if (aj.headers.get('access-control-allow-origin') !== '*') {
    fail('actions.json is missing Access-Control-Allow-Origin: *');
  }
  const discovery = (await aj.json()) as { rules?: unknown[] };
  if (!Array.isArray(discovery.rules) || discovery.rules.length === 0) {
    fail('actions.json has no rules');
  }

  const get = await fetch(`${BASE}/api/actions/drop`);
  if (get.headers.get('access-control-allow-origin') !== '*') {
    fail('GET metadata is missing CORS');
  }
  const meta = (await get.json()) as Record<string, unknown>;
  for (const field of ['icon', 'title', 'description', 'label']) {
    if (typeof meta[field] !== 'string') fail(`GET metadata is missing ${field}`);
  }
  if (!String(meta.icon).startsWith('http')) fail('icon must be an absolute URL');

  const post = await fetch(`${BASE}/api/actions/drop`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ account: testAccount }),
  });
  if (post.status !== 200) fail(`POST returned ${post.status}`);
  const body = (await post.json()) as Record<string, unknown>;
  if (body.type !== 'transaction') fail("POST response type must be 'transaction'");
  if (typeof body.transaction !== 'string' || body.transaction.length === 0) {
    fail('POST response has no base64 transaction');
  }

  console.log('SMOKE PASS: actions.json + GET metadata + POST response all conformant');
}

main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
```

Rode o servidor em um terminal:

```bash
MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
```

Espere a linha de listening dele, depois rode a checagem de smoke em um segundo terminal:

```bash
npx tsx smoke.ts
```

Dois terminais em vez de um comando em background, de propósito: o `smoke.ts` abre com um `fetch` que não tenta de novo, e o `tsx` leva um segundo ou dois para compilar e fazer o bind. Encadeado atrás de um `&`, o fetch normalmente perde essa corrida e te entrega um `ECONNREFUSED` que não tem nada a ver com o seu código.

Você deve ver `SMOKE PASS`. Se em vez disso ele falhar no POST, o seu `buildActionPostResponse` ainda lança o placeholder dele, que é o lab te dizendo que o degrau de completion é genuinamente seu. Termine ele, ou trabalhe ele primeiro pelo desafio de código e cole a sua implementação que passa de volta aqui.

## Challenge

**O desafio de código** (no widget de desafio desta lição) é o `buildActionPostResponse` isolado, chamado posicionalmente como `buildActionPostResponse(transactionBase64, message?, nextActionHref?)`: a transação em base64 sozinha produz exatamente `type` mais `transaction` e nada mais, `message` aparece só quando um segundo argumento não nulo é fornecido, `links.next` aparece só como `{ type: 'post', href }` quando um terceiro argumento `nextActionHref` não nulo é fornecido. Passe nele, depois traga o código para casa no passo 3.

**O degrau solo, em duas partes.** Sem passo a passo desta vez; você tem tudo que precisa.

Primeiro, complete uma compra de verdade sendo você mesmo o client que renderiza, porque é só isso que um client de blink é: GET, renderizar, POST, assinar, enviar, seguir o `links.next`. Escreva `drop-blink/client.ts` no workspace. Ele busca os metadados do GET e imprime o title e os dois labels de botão (esse é o seu passo de renderização), dá POST em `{account}` com o endereço da sua carteira de devnet com saldo, decodifica a transação em base64 devolvida, assina ela e envia exatamente do jeito que o solo de transaction request mandou você fazer (carregue a chave com `createKeyPairSignerFromBytes`, assine com `signTransaction`, envie o base64 pelo `rpc.sendTransaction`), depois dá POST da signature confirmada no href do `links.next` e imprime o title do card de completed. O sucesso é concreto: uma transação na devnet liquida carregando o memo do seu pedido e a reference key, e o seu terminal termina no title do card de agradecimento. Uma nota de honestidade enquanto você constrói isso: um client Node não impõe CORS, que é exatamente por que o smoke test checa os headers explicitamente; a tabela de quatro linhas do funciona-no-curl é o que um client rodando no navegador encontraria, e os seus dumps de header abaixo são a prova de que você sobreviveria a um.

Segundo, produza um **checklist pronto para o registro** do blink do drop: um arquivo markdown curto no repo afirmando, com evidência, (1) conformidade com a spec, a sua saída do smoke; (2) CORS em toda rota de action e no `actions.json`, com dumps de header; (3) `actions.json` alcançável na raiz do domínio do seu alvo de deploy; (4) prontidão para envio, a URL pública da action e o contato que você mandaria para a revisão manual da Dialect, mais qual variável de ambiente guardaria uma chave de API da Dialect se você um dia consumir o endpoint de consulta com gate por chave. Repare no que o checklist deliberadamente não afirma: nenhuma renderização em superfície externa, nem nenhum prazo de registro. Essa contenção é o ponto. O checklist é o artefato que um você do futuro, ou um cliente, consegue entregar para a revisão da Dialect sem uma única promessa que você não consegue cumprir.

A evidência de header leva um comando por endpoint; `-D -` despeja os headers de resposta no stdout e `-o /dev/null` descarta o body:

```bash
curl -s -D - -o /dev/null http://localhost:3000/actions.json | grep -i access-control
curl -s -D - -o /dev/null http://localhost:3000/api/actions/drop | grep -i access-control
```

Cole as duas saídas no checklist na íntegra. Evidência que você consegue regenerar em dez segundos bate garantias em prosa toda vez, e quando você fizer o redeploy atrás de um proxy diferente no capstone, rodar duas linhas de curl comprova a afirmação de novo.

![Tabela das quatro afirmações de pronto para o registro (conformidade com a spec, CORS em todo lugar, actions.json na raiz, prontidão para envio), cada uma com evidência regenerável e um selo de gate ou de preparação.](assets/v07-table.png)

Aceite: o GET valida, o POST devolve uma resposta em conformidade com a spec reusando o builder de transaction request, o seu script de client completa uma compra na devnet, e o checklist existe com todos os quatro pontos de evidência.

## Checkpoint, e onde o módulo aterrissa

Se o smoke test brigou com você, a falha é quase certamente uma de quatro: headers de CORS aplicados depois de uma rota casar (mova o middleware para cima de toda rota, static incluído), `actions.json` montado sob `/api` em vez da raiz, o caminho de import do builder não batendo com o seu layout do checkout-txreq, ou `MERCHANT_ADDRESS` faltando no ambiente do servidor (o builder importado exige ela e dá 400 sem ela). Dez minutos, na minha experiência, na maioria das vezes a terceira. E se alguma coisa mais sutil quebrou, dê diff no seu arquivo de tipos contra `@solana/actions-spec` em `node_modules`; o pacote da spec está congelado, então qualquer campo que a gente espelha e que discorda da fonte está do nosso lado por definição (campos que só existem do lado da spec são os cortes deliberados que o cabeçalho do arquivo de tipos nomeia). Quando você conseguir o pass, leve a vitória a sério: você entregou uma vitrine-num-link em conformidade com o protocolo, com afirmações de alcance que você consegue defender linha por linha, e essa combinação é mais rara no mundo real do que o endpoint em si.

Três superfícies, um core de pagamento: o checkout por QR, a barraca da feira, e agora um blink que executa onde quer que alguma coisa renderize ele. A Wavelength consegue vender um disco por uma página, do outro lado de uma mesa e dentro de um link. O que quer dizer que a frente da loja está pronta, e a pergunta honesta se move para dentro: dinheiro está chegando de três superfícies e você ainda está confiando em frontends para te contar sobre isso. O próximo módulo deixa a vitrine para o back office, onde você não confia em nenhum frontend e verifica todo pagamento no servidor.
