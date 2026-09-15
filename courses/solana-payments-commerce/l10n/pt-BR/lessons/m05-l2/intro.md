# O programa oficial Subscriptions: planos, PDAs e a costura do kit 7

## Resumo

Na lição passada você construiu o club-crank cru em cima de uma única aprovação de delegado e bateu no muro dele: um slot de delegado por conta de token significa uma assinatura viva por usuário, e a aprovação de um segundo lojista te despeja. Em silêncio. Esta lição troca a aprovação crua pelo programa Subscriptions da Solana Foundation, cuja razão inteira de existir é esse despejo.

Antes de a gente nomear a costura de versão desta lição, prove ela para você mesmo. Você tem npm desde a lição de setup; rode isto de qualquer lugar:

```bash
npm view @solana/subscriptions@0.5.0 peerDependencies
```

Você deve ver `{ '@solana/kit': '^7.0.0' }`. Agora rode `npm view @solana/pay peerDependencies` e olhe a linha de kit dele: `^6.9.0`. Dois pacotes de que este curso depende, duas versões major do mesmo SDK, as duas corretas. Segure esse pensamento; a gente resolve isso direito na última seção de teoria, e custa a você uma pasta de workspace a mais. Os achados, logo de cara:

- O programa mora em `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` e o truque dele é um movimento só: um PDA de Subscription Authority por (usuário, mint) toma o slot de delegado único UMA VEZ com uma aprovação de u64::MAX, e daí PDAs de delegação por plano carregam os limites de cobrança reais e aplicados. Um slot, quantas assinaturas o usuário quiser.
- Você entrega o **club-billing**: cria o plano do disco do mês da Wavelength, inscreve um usuário de teste, puxa um período de cobrança e aterrissa esse pull como uma linha de fatura no exato livro-razão de pedidos do back office que você construiu na lição de webhook. Uma nota de honestidade que atravessa o lab: o pull oficial não carrega reference key nenhuma e nenhum memo, então a verdade de cobrança é escrita pelo crank, com chave na signature do pull, e não conciliada por reference do jeito que um checkout é; a lição de dunning anexa uma reference à fatura quando a liquidação precisa de uma.
- Dois bugs documentados de unidade e relógio mordem integradores no mundo real, e nenhum dos dois é um exploit nem uma cobrança em dobro — o programa recusa on-chain um pull fora de hora ou vencido, então os dois bugs queimam taxas do crank, não dinheiro do assinante. Planos medem o período deles em `periodHours` enquanto tudo contra o que você compara é segundo Unix, e contas de assinatura nunca se fecham sozinhas, então `expiresAtTs` — não a existência da conta — é o limite de tempo que a sua guarda precisa espelhar.
- O cliente é o `@solana/subscriptions` 0.5.0, e ele declara peer em `@solana/kit` ^7.0.0 enquanto os seus workspaces de checkout estão no kit ^6. Essa costura é real, é o estado atual do ecossistema, e a gente lida com ela com um pin de workspace separado, não com uma reescrita.

Mais uma coisa que vale dizer sem rodeios. A versão 0.5.0 deste programa teve deploy na mainnet em 2026-08-10, doze dias atrás enquanto eu escrevo isto. Essa data é o deploy da v0.5.0, não a estreia do programa na mainnet (versões anteriores já estavam no ar antes dela), mas ainda assim faz dele a coisa estrutural mais nova do curso. Você está aprendendo isso antes de a maioria dos guias de integração existir. Trate isso como o trabalho, e não como um aviso de risco: engenheiros de pagamentos são pagos para estar cedo e certos ao mesmo tempo.

## Um slot, muitas assinaturas

### A rampa de acesso ao PDA, em 30 segundos

Lá no módulo de stablecoin eu te disse que o endereço da conta de token associada é derivado, não escolhido, e prometi que a ideia completa ia te custar 30 segundos quando você finalmente precisasse dela. É este o momento.

Um endereço derivado de programa (PDA) é um endereço calculado a partir do endereço do próprio programa mais algumas seeds, construído de propósito para NÃO ter chave privada. Não é uma chave perdida, não é uma chave trancada: nenhuma chave existe, matematicamente. Ninguém nunca consegue assinar como aquele endereço. A única coisa que consegue agir como um PDA é o programa que deriva ele, de dentro do próprio código, sob as regras que esse código aplicar.

Leia isso de novo com o seu cérebro de Stripe ligado, porque é aqui que a ficha cai para a lição inteira. Quando o crank da Wavelength segurava o slot de delegado diretamente, na lição passada, um par de chaves que VOCÊ controlava tinha direitos de pull, e o assinante tinha que confiar no seu time de ops. Se um PDA segura o slot de delegado no lugar, não existe chave de lojista para vazar, não existe funcionário para virar a casaca, não existe invasão de servidor sua que renda direitos de pull. Um resíduo honesto: a autoridade de upgrade do programa ainda é uma chave que alguém segura, então a confiança se mudou do seu time de ops para o dono do programa em vez de sumir — uma superfície menor e auditável, e a ressalva que você oferece de bandeja numa revisão de segurança antes que outra pessoa ofereça. Passado esse resíduo, o único caminho até os fundos do assinante é a lógica do próprio programa, e a lógica do programa só move fundos dentro dos limites que o assinante assinou explicitamente. É isso que cobrança não custodial quer dizer, e ela se apoia em algo concreto: a ausência de uma chave privada do lado do lojista.

![Um par de chaves do lojista segurando o slot de delegado obriga a confiar em todo mundo que consegue assinar, enquanto um PDA de Subscription Authority não tem chave privada, então só a lógica do programa consegue puxar.](assets/v01-diagram.png)

### Uma aprovação ilimitada, muitas delegações limitadas

Aqui está a arquitetura, e ela soa invertida até você ver por que é o único formato que serve.

O assinante inicializa um PDA de **Subscription Authority** (SA), um por par (usuário, mint). Esse PDA toma o slot de delegado único da conta de token com uma aprovação de u64::MAX. Ilimitada. O número que seria aterrorizante num par de chaves de lojista está de bom tamanho aqui, porque o PDA de SA não é um gastador, é uma mesa telefônica. Ele nunca vai mover um token por iniciativa própria; ele não tem iniciativa, e não tem chave.

Se o número ilimitado ainda te incomoda, mapeie ele em cima de algo que você já entregou. Um cartão salvo num processador de pagamento é, mecanicamente, uma autoridade de cobrança ilimitada: nada na bandeira impede um lojista de cobrar o valor errado, e o limite de verdade é política, chargebacks e, no fim, advogados. Aqui a coisa ilimitada é sem chave e inerte, e a coisa limitada é código que recusa. Mesmo formato, ordem de confiança invertida. Eu sei por qual dos dois eu prefiro conduzir uma revisão de segurança.

Os limites de verdade moram uma camada abaixo, em **PDAs de delegação** criados sob aquela autoridade. Cada delegação é uma conta separada carregando os próprios termos aplicados, e o programa recusa qualquer pull que viole eles. Três modelos vêm na v0.5.0:

![Três modelos de delegação ficam lado a lado: fixed com um teto total e expiração opcional, recurring com um teto por período que reseta, em segundos, e plans publicados em horas.](assets/v02-table.png)

Um tour rápido por onde cada modelo ganha o pão, porque o lab usa só o terceiro e os outros dois vão aparecer nas suas conversas de produto dentro de uma semana. **Fixed** é uma comanda limitada: puxe até 50 USDC antes de sexta, e aí a delegação está gasta. Serve para autorizações pontuais com teto, um trial que não pode converter em silêncio, uma pré-venda que cobra quando a prensagem sai. **Recurring** é um limite que se renova: até 20 USDC por semana, indefinidamente ou até a expiração, com `amountPulledInPeriod` resetando a cada período. Serve para cobrança por uso em que o valor varia mas o teto não pode variar, uma API medida em dólares, uma carteira de crédito que se recarrega sozinha. O modelo **plan** é o que tem formato de lojista: termos publicados uma vez on-chain, todo assinante aceita exatamente aqueles termos, os pulls aterrissam uma vez por período de cobrança em destinos que o plano declarou de antemão. A Wavelength quer termos idênticos para todo membro e um catálogo público para apontar, então o clube é um plan. E se você algum dia se pegar cunhando centenas de planos quase idênticos para codificar termos sob medida por cliente, pare; é para isso que as delegações recurring existem.

Repare o que isso dissolve. O muro da lição passada era que aprovações sobrescrevem umas às outras. Agora o slot do assinante está ocupado exatamente uma vez, pelo PDA de autoridade dele mesmo, e entrar no plano de um segundo lojista só cria mais uma conta de delegação embaixo dele. Os direitos de pull da Wavelength sobrevivem ao assinante entrar em outros dez clubes. Ninguém despeja ninguém, porque ninguém encosta no slot de novo.

O modelo plan, que o lab usa, divide o estado em duas contas. O lojista cria um PDA `Plan` (com seeds no endereço do lojista mais um id de plano) segurando os termos: valor, `periodHours`, o mint, destinos de pull permitidos, uma whitelist de pullers. O aceite do assinante cria um PDA `SubscriptionDelegation` (com seeds no plano mais o assinante) rastreando o estado individual dele: `currentPeriodStartTs`, `amountPulledInPeriod`, `expiresAtTs`. Estado de lojista e estado de assinante nunca dividem uma conta, e é por isso que um plano escala para qualquer número de assinantes sem ninguém reescrever nada.

![Uma conta Plan do lojista publica os termos enquanto cada assinante ganha uma conta SubscriptionDelegation separada, então um plano escala para qualquer número de assinantes sem reescrever estado.](assets/v03-diagram.png)

E a aplicação dos limites não é um conselho. Tente puxar duas vezes num período e o programa recusa com um erro period-not-elapsed antes de um token se mover. Tente puxar para um endereço que o plano nunca declarou e você recebe uma recusa unauthorized-destination. Os limites que você vai implementar na guarda do crank desta lição são uma camada de cortesia que te poupa taxas e barulho de log; o programa é a camada que salva o assinante.

### A porta de saída, e o que o assinante trocou por ela

Cobrança não custodial só é honesta se sair for tão unilateral quanto entrar. E é. O assinante consegue cancelar a assinatura de um plano, o que fecha a conta de delegação dele, e consegue revogar a própria Subscription Authority, o que libera o slot de delegado e encerra toda delegação embaixo dela num movimento só. Nenhuma signature de lojista aparece em nenhum dos dois caminhos; a carteira que consentiu consegue retirar o consentimento sozinha, a qualquer momento. Uma precisão sobre o rent que essas contas seguram, já que é fácil supor que a saída reembolsa ele: cancelar a assinatura fecha a conta de delegação e devolve o rent dela para quem pagou por ela, mas revogar a Subscription Authority só libera o slot de delegado — ela não varre os PDAs. Recuperar esses é uma chamada `RevokeAbandoned` separada, rodada pelo lojista, cujo signatário é o **payer registrado**, que é a sua carteira se você patrocinou o subscribe. A próxima lição constrói essa fila; hoje o ponto é que sair é unilateral, não que seja autolimpante. Compare isso com o fluxo de retenção de vinte minutos que a sua última academia te obrigou a engolir.

O trade-off, nomeado, porque o design da lição passada tinha uma virtude que este aqui aposenta em silêncio. A aprovação crua de 60 USDC secava depois de quatro pulls, e esse esgotamento forçava uma conversa natural de reconsentimento a cada quatro meses. A aprovação de u64::MAX da SA nunca seca. A proteção do assinante não é mais um número que encolhe; são os limites por plano mais aquela saída unilateral. Mecanicamente isso é uma proteção estritamente melhor, e ainda assim merece este parágrafo, porque o limite minguante fazia um trabalho silencioso de UX no design cru que nada automático substitui aqui: ninguém é perguntado de novo por padrão. Exponha as assinaturas ativas na UI do seu produto e faça o cancelamento ser um toque só; a blockchain não vai ficar no pé do assinante em seu nome.

![Uma assinatura corre da inicialização da autoridade até os pulls periódicos; uma vencida persiste on-chain, parada só pela checagem de expiração, até o assinante cancelar ou revogar.](assets/v04-timeline.png)

### Quem construiu, e quem já aposta nisso

Procedência importa mais que o normal quando a coisa é nova assim. O programa foi escrito pela Moonsong Labs em Pinocchio, o framework Rust sem dependências, e auditado pela Cantina. Se a escolha por Pinocchio te deixa curioso sobre como um programa é construído nesse nível, essa curiosidade pertence ao curso Master Anchor V2, que é dono da camada de framework; aqui a gente consome o programa, não lê o código-fonte dele.

A batida de prova-de-produção é melhor que um selo de auditoria de qualquer jeito: a Helius roda a PRÓPRIA cobrança de assinaturas nesse mesmo programa Subscriptions da Foundation (blog de engenharia deles, acessado em 2026-08-21). Quando uma empresa de infraestrutura cujo produto é uptime cobra os clientes dela através de um programa, esse programa saiu do território de demo. Um programa auditado com um inquilino de produção de marca conhecida é uma aposta diferente de um deploy de uma semana aceito na fé.

### Token-2022, consumido, não ensinado

O mint do plano pode ser um mint Token-2022, e dois comportamentos importam para a cobrança. Primeiro, se o mint carrega um transfer hook, o programa encaminha as contas extras do hook para a CPI de TransferChecked dele, então um pull compõe com tokens travados por hook em vez de morrer neles; o cliente até entrega um helper `resolveTransferHookAccounts` para a resolução das contas. Segundo, se uma conta de destino tem MemoTransfer ligado (ele exige um memo em toda transferência que entra), o pull é rejeitado atomicamente: nenhum estado parcial, nenhum fundo preso, a transação simplesmente falha inteira. O programa também avalia o conjunto de extensões do mint quando a autoridade é inicializada e recusa combinações que não consegue cobrar com segurança, então você descobre na hora do setup, não na hora da cobrança.

É tudo o que a gente precisa SABER aqui. Como a interface de transfer hook em si funciona, de ponta a ponta, é território do curso Digital Assets, Tokenization and Token Extensions. A gente é consumidor de hook, e consumidor tem o direito de continuar felizmente magro.

### Os dois relógios que cobram errado das pessoas

Agora as ciladas, porque os dois bugs documentados de integração deste programa, os dois são bugs de tempo, os dois estragam uma rodada de cobrança do jeito deles, e os dois vão estar no starter do seu challenge de propósito.

**Bug um: horas não são segundos.** Um `Plan` armazena a cadência dele como `periodHours` (720 para o plano mensal da Wavelength). Uma `RecurringDelegation` armazena a cadência dela como `periodLengthS`, em segundos. Todo timestamp contra o qual você algum dia vai comparar, `currentPeriodStartTs`, `expiresAtTs`, o tempo da blockchain, é segundo Unix. Compare `periodHours` direto contra um delta em segundos e a sua janela encolhe por um fator de 3,600: o seu crank considera um plano de 24 horas devido de novo depois de 24 segundos. Repare quem salva o assinante aqui — o programa salva, exatamente como a seção de aplicação disse: o pull adiantado é recusado com o erro period-not-elapsed antes de um token se mover. O que o programa não consegue salvar é a sua carteira e os seus logs: todo pull recusado custa ao crank uma taxa base e uma linha de log, a cada tick, para sempre, até você notar. A regra é chata e absoluta: converta para segundos na fronteira, compare só segundos. 24 horas são 86,400 segundos, não 24.

**Bug dois: nada expira sozinho.** Contas de assinatura e de delegação persistem on-chain até uma instrução explícita de revoke fechar elas. Um plano cujo prazo terminou semana passada ainda tem uma conta de delegação viva ali parada, e se o seu crank só checa "a delegação existe?", ele vai continuar despachando pulls para aquele assinante vencido. Mesma divisão de trabalho do bug um: o programa aplica o limite de tempo — um pull contra uma delegação vencida é recusado com o erro subscription-cancelled dele antes de um token se mover, então ninguém é cobrado — e o que o crank que só olha existência compra para si mesmo é o mesmo imposto, uma taxa base e uma linha de log por tick recusado, para sempre. `expiresAtTs` contra o tempo da blockchain é a checagem que a sua guarda espelha a cada tick, então a recusa acontece no seu processo de graça em vez de on-chain por uma taxa. E o caso zero dele morde na direção oposta: `expiresAtTs` de 0 quer dizer "nunca expira", então uma guarda que compara ingenuamente `now >= expiresAtTs` trata toda assinatura sem expiração como expirada no epoch zero e se recusa a cobrar de qualquer um. Trate o zero primeiro, depois compare.

![Dois bugs documentados de cobrança ficam lado a lado: ler periodHours como segundos despacha pulls umas 3600 vezes mais do que deveria, e pular expiresAtTs continua despachando pulls para assinantes vencidos; o programa recusa os dois, a uma taxa base por recusa.](assets/v05-comparison.png)

### A costura do kit: checkout no v6, cobrança no v7

Hora de resolver a sonda que você rodou no primeiro minuto. O que ela trouxe à tona é o estado atual da indústria, então vamos tratar isso como profissionais: com fatos datados e um pin.

Os fatos, reverificados contra o npm em 2026-08-22: a dist-tag `latest` do kit aponta para 8.0.0 (publicada em 2026-08-21); a linha v7 terminou em 7.1.1; a linha v6 terminou em 6.10.0. A versão 7 é o padrão de peer do ecossistema agora: a leva de clientes `@solana-program/*` de julho de 2026 declara peer em `^7.0.0` (isso é o `@solana-program/token` 0.15.0), e o `@solana/subscriptions` 0.5.0 também. Repare na velocidade com que a frente do pelotão se mexe, porém: o `@solana-program/token` 0.16.0 saiu em 2026-08-21, no mesmo dia do kit 8, e já declara peer em `^8.0.0`. Os retardatários são igualmente reais e estruturais para a gente: o `@solana/pay` 1.0.26 declara peer no kit `^6.9.0`, que é exatamente por que os seus workspaces de checkout foram fixados no kit ^6.10 em primeiro lugar, e ele não está sozinho lá embaixo — o helius-sdk 3.1.0 declara peer na mesma faixa `^6.9.0`, então uma loja que tivesse mesmo pegado aquele SDK aterrissaria no pin idêntico. Três majors do kit, todas sendo publicadas, todas corretas para alguém. Instale subscriptions dentro daqueles workspaces e o resolvedor de peer do npm vai recusar, corretamente. Nunca fixe em `latest` em lugar nenhum; essas tags se mexeram duas vezes enquanto este curso estava sendo escrito.

O que destrava isso? Uma pasta que fica de propósito FORA da lista de workspaces. Workspaces registrados do npm não são isolamento — eles são o oposto: o npm iça todo pacote registrado para uma única resolução de raiz compartilhada, que é exatamente a árvore onde o kit 6 e o kit 7 se encontrariam e brigariam. Então o passo 1 abaixo cria `subscriptions/` como um pacote autônomo e nunca adiciona ele ao array `workspaces` da raiz — uma quebra deliberada do hábito de registrar tudo que o módulo 4 te ensinou. Só ela fixa o kit ^7 mais o `@solana/subscriptions` 0.5.0, roda o próprio `npm install`, resolve a partir do próprio `node_modules`, e as faixas de peer nunca se encontram. (Registre ela na raiz e o resolvedor do npm vai tentar reconciliar as duas majors do kit numa árvore só e recusar; o capstone entra nesse ERESOLVE exato de propósito e te mostra a saída de emergência.) E se aparecer um atrito de v7 que você não consiga resolver, o fallback documentado é uma edição de pin de duas linhas naquela pasta só: `@solana/subscriptions` 0.4.0 com kit ^6.4. Nenhuma mudança estrutural, nenhuma reescrita, o `package.json` de uma pasta.

![Os workspaces de checkout e de ops continuam fixados em pacotes do kit 6 enquanto a pasta subscriptions, deixada de propósito fora do array workspaces da raiz, fixa o kit 7, com o fallback documentado para subscriptions 0.4.0 no kit 6.4.](assets/v06-diagram.png)

Isso é irritante? Moderadamente. Isso é incomum? Nem um pouco: qualquer loja Node que sobreviveu à migração para ESM, ou a um major do React, já rodou exatamente esta jogada. Ecossistemas de SDK se mexem da frente para trás, os pacotes carro-chefe pulam primeiro, as integrações atrasam, e a fronteira mora nos seus lockfiles por um trimestre ou dois. Você não está contornando um erro; você está assistindo a um ecossistema no meio da passada, e o pin por workspace é a cara da competência enquanto isso aterrissa. E também não é uma regra que este curso inventou: o curso Rust & TypeScript Fundamentals crava ela na lição de faixas de peer e o Master Anchor V2 crava ela contra os peers declarados de um cliente gerado no módulo oito dele — três cursos, uma regra.

## Lab: cobre o clube do disco do mês

Como o trabalho se divide aqui: as chamadas de autoridade, de plano e de subscribe vêm com apoio e eu passo por elas inteiras; o passo de escrita no livro-razão é seu para ligar (completion); e a guarda de janela de período é o challenge solto depois do lab (solo). Lá no módulo de ops você vai estar construindo esta categoria de integração sem apoio nenhum, então repare no que o apoio faz enquanto você ainda tem ele.

**1. Crie o workspace e fixe a costura.** A partir da raiz do monorepo:

```bash
mkdir -p subscriptions/keys
cd subscriptions
npm init -y
npm pkg set type=module
npm install @solana/subscriptions@0.5.0 @solana/kit@7.1.1 @solana-program/token@0.15.0
npm install -D typescript tsx @types/node
```

Pins verificados em 2026-08-22 e recarimbados em 2026-09-05: o `@solana/subscriptions` 0.5.0 é a linha carimbada para este pacote — pré-1.0 e se mexendo, lançada em 2026-08-10, declara peer no kit ^7.0.0 — com a 0.4.0 mantida só como fallback documentado, nunca o pin de onde você parte; a 7.1.1 é o último release da linha v7 do kit; o `@solana-program/token` 0.15.0 é a leva de cliente de token que declara peer em ^7. Aquele último pin é o que as pessoas erram: o `latest` do cliente de token é 0.16.0, que declara peer no kit ^8 e não vai resolver aqui. Reconfira os três com `npm view <pkg> dist-tags peerDependencies` antes de instalar, porque este canto do npm se mexeu duas vezes neste trimestre. O `tsx` roda TypeScript direto e o `@types/node` te dá os tipos do Node; você usa os dois desde o módulo de checkout, mas este é um workspace novo, então eles são instalados do zero.

Você precisa de dois pares de chaves de devnet com saldo em `keys/`: `merchant.json` (Wavelength) e `subscriber.json` (o seu ouvinte de teste), com o assinante segurando um pouco do USDC de devnet em que você cobra desde o módulo 2. A lição do crank cunhou o `subscriber.json` (copie ele para `keys/`), mas nunca teve uma chave de lojista, então crie essa do zero: `solana-keygen new -o keys/merchant.json`, depois faça o airdrop e ponha saldo como sempre.

**2. Config compartilhada e um helper de envio.** Dois arquivos pequenos que você vai reconhecer de todo workspace até aqui. Primeiro o `config.ts`:

```typescript
import { readFileSync } from "node:fs";
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  sendAndConfirmTransactionFactory,
  type Address,
  type KeyPairSigner,
} from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";

export const rpc = createSolanaRpc("https://api.devnet.solana.com");
export const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com",
);
export const sendAndConfirm = sendAndConfirmTransactionFactory({
  rpc,
  rpcSubscriptions,
});

// The mint the club bills in: the same devnet USDC mint the crank lesson
// hardcoded (4zMM...ncDU), taken as an env var here so later lessons can
// re-point the club without code edits.
export const CLUB_MINT: Address = address(process.env.CLUB_MINT!);
export const CLUB_TOKEN_PROGRAM = TOKEN_PROGRAM_ADDRESS;

export async function loadSigner(path: string): Promise<KeyPairSigner> {
  const bytes = new Uint8Array(JSON.parse(readFileSync(path, "utf8")));
  return createKeyPairSignerFromBytes(bytes);
}
```

Depois o `send.ts`, o pipeline canônico de envio do kit. Mesmo formato que você já escreveu três vezes nos workspaces v6; a surpresa agradável da costura é que este código é idêntico no v7:

```typescript
import {
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  getSignatureFromTransaction,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import { rpc, sendAndConfirm } from "./config";

export async function sendIxs(
  payer: KeyPairSigner,
  ixs: Instruction[],
): Promise<string> {
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
  return getSignatureFromTransaction(signed);
}
```

**3. Inicialize a Subscription Authority do assinante.** Este é o passo uma-vez-por-(usuário, mint): o PDA de SA toma o slot de delegado com a aprovação de u64::MAX dele, e toda assinatura futura para este mint pendura nele. `01-init-authority.ts`:

```typescript
import { getInitSubscriptionAuthorityInstructionAsync } from "@solana/subscriptions";
import { findAssociatedTokenPda } from "@solana-program/token";
import { CLUB_MINT, CLUB_TOKEN_PROGRAM, loadSigner } from "./config";
import { sendIxs } from "./send";

async function main() {
  const subscriber = await loadSigner("keys/subscriber.json");

  const [userAta] = await findAssociatedTokenPda({
    owner: subscriber.address,
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const ix = await getInitSubscriptionAuthorityInstructionAsync({
    owner: subscriber,
    tokenMint: CLUB_MINT,
    userAta,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const sig = await sendIxs(subscriber, [ix]);
  console.log("subscription authority initialized:", sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Rode com `CLUB_MINT=<your devnet mint> npx tsx 01-init-authority.ts`. Repare quem assina: o ASSINANTE. Só o dono da conta consegue entregar o slot de delegado dele, que é o passo de consentimento da arquitetura inteira; a Wavelength nunca encosta nesta transação.

Uma conferência de realidade antes de você continuar, porque a versão mais nova do programa é fresca assim: o deployment de devnet pode atrasar em relação ao de mainnet. Sonde primeiro, `solana program show De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44 --url devnet` tem que imprimir um programa executável (imprimiu em 2026-08-22). A defasagem de versão só se revela na primeira instrução: se esta chamada ou qualquer outra depois falhar com o erro de programa customizado 133 ou 134 (o cliente chama eles de `DELEGATION_VERSION_MISMATCH` e `MIGRATION_REQUIRED`), o binário de devnet é mais velho que o seu cliente 0.5.0. Faça o remédio do lado do cliente, que é o lado que este curso fixa e verifica: derrube este workspace só para o pin de fallback documentado da seção da costura, `@solana/subscriptions@0.4.0` com kit `^6.4`, e continue contra o programa de devnet publicado, do jeito que ele está. Construir o programa você mesmo é um projeto diferente deste lab, e um cuja revisão de código-fonte nada aqui confere para você. Confira primeiro, esbraveje depois.

**4. Crie o plano.** A Wavelength publica o disco do mês pelo mesmo preço que o crank cru cobrava na lição passada: 15 USDC a cada 720 horas. `02-create-plan.ts`:

```typescript
import {
  findPlanPda,
  getCreatePlanInstruction,
} from "@solana/subscriptions";
import { address } from "@solana/kit";
import { CLUB_MINT, loadSigner } from "./config";
import { sendIxs } from "./send";

const PLAN_ID = 1n;

// The on-chain Plan stores destinations and pullers as fixed four-slot
// arrays; unused slots carry the system address as an explicit "empty".
const NONE = address("11111111111111111111111111111111");

async function main() {
  const merchant = await loadSigner("keys/merchant.json");

  const [planPda] = await findPlanPda({
    owner: merchant.address,
    planId: PLAN_ID,
  });

  const ix = getCreatePlanInstruction({
    merchant,
    planPda,
    tokenMint: CLUB_MINT,
    planData: {
      planId: PLAN_ID,
      mint: CLUB_MINT,
      terms: {
        amount: 15_000_000n, // 15 USDC at 6 decimals
        periodHours: 720n, // HOURS on the plan; you convert everywhere else
        // The program stamps its own clock over this field at execution;
        // 03-subscribe reads the stored value back rather than trusting ours.
        createdAt: BigInt(Math.floor(Date.now() / 1000)),
      },
      endTs: 0n, // 0 = no scheduled end, the same zero-means-never convention as expiry
      // Destinations are WALLET addresses, never token accounts: the program
      // whitelists the owner, and each pull presents that owner's ATA for the
      // plan's mint, derived at pull time. Declare an ATA here and every pull
      // is refused on-chain with the destination-not-in-whitelist error.
      destinations: [merchant.address, NONE, NONE, NONE],
      pullers: [merchant.address, NONE, NONE, NONE],
      metadataUri: "https://wavelength.example/plans/record-of-the-month.json",
    },
  });

  const sig = await sendIxs(merchant, [ix]);
  console.log("plan created:", planPda, sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Os arrays `destinations` e `pullers` são o controle de acesso do próprio plano, on-chain: um pull só consegue aterrissar na ATA de uma carteira de destino declarada, e só o dono do plano ou um puller na whitelist consegue iniciar um. Os dois arrays viajam como structs fixos de quatro slots (o cliente codifica exatamente quatro entradas, daí o preenchimento com `NONE`) e os dois armazenam endereços de carteira, verificado contra a devnet: planos reais armazenam donos, e um plano que armazenou uma ATA no lugar teve os pulls dele recusados com o erro destination-not-in-whitelist. O seu par de chaves de crank vai em `pullers` quando você produtizar; no lab a chave do lojista puxa direto.

**5. Inscreva o usuário de teste.** O passo de aceite, e o meu detalhe de design favorito do programa inteiro. `03-subscribe.ts`:

```typescript
import {
  fetchPlan,
  fetchSubscriptionAuthorityFromSeeds,
  findPlanPda,
  getSubscribeInstructionAsync,
} from "@solana/subscriptions";
import { address } from "@solana/kit";
import { CLUB_MINT, loadSigner, rpc } from "./config";
import { sendIxs } from "./send";

const PLAN_ID = 1n;
const MERCHANT = address(process.env.MERCHANT!);

async function main() {
  const subscriber = await loadSigner("keys/subscriber.json");

  const [planPda, planBump] = await findPlanPda({
    owner: MERCHANT,
    planId: PLAN_ID,
  });
  const plan = await fetchPlan(rpc, planPda);
  const authority = await fetchSubscriptionAuthorityFromSeeds(rpc, {
    user: subscriber.address,
    tokenMint: CLUB_MINT,
  });

  // The expected* fields pin the terms you read to the terms that execute.
  // If the plan changes between your read and your landing, the program
  // refuses with a terms-mismatch error instead of billing you.
  const ix = await getSubscribeInstructionAsync({
    subscriber,
    merchant: MERCHANT,
    planPda,
    subscriptionAuthorityPda: authority.address,
    subscribeData: {
      planId: PLAN_ID,
      planBump,
      // Double .data is not a typo: fetchPlan returns the account wrapper,
      // whose data field holds the program's versioned Plan struct. The
      // delegation and authority accounts decode flat, hence their single
      // .data everywhere else in this lab.
      expectedMint: plan.data.data.mint,
      expectedAmount: plan.data.data.terms.amount,
      expectedPeriodHours: plan.data.data.terms.periodHours,
      expectedCreatedAt: plan.data.data.terms.createdAt,
      expectedSubscriptionAuthorityInitId: authority.data.initId,
    },
  });

  const sig = await sendIxs(subscriber, [ix]);
  console.log("subscribed:", sig);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Aqueles campos `expected*` merecem a pausa. O assinante assina exatamente os termos que leu, e o programa compara eles com o plano na hora da execução. Um lojista que edita o preço entre o clique do assinante e a transação aterrissando recebe uma recusa, não uma bolada. Sistemas de assinatura Web2 aplicam isso com advogados e prints de tela; aqui é uma comparação de struct dentro da transação. Rode com `MERCHANT=<merchant pubkey> CLUB_MINT=<mint> npx tsx 03-subscribe.ts`.

![A cada tick de cobrança o crank lê o estado da assinatura, o decidePull filtra pulls canceled, expired e too-early antes de qualquer taxa ser gasta, e um pull due aterrissa uma vez no livro-razão.](assets/v07-flowchart.png)

**6. Puxe um período de cobrança e aterrisse ele no livro-razão.** Um import abaixo ainda não existe: `./decide-pull`. Salve agora o starter do Challenge do fim desta lição como `subscriptions/decide-pull.ts`, com bugs e tudo, para o lab rodar em ordem; consertar esses dois bugs é o trabalho solo que te espera lá. Este é o passo de acréscimo, e a razão de esta lição consumir dois artefatos anteriores em vez de um. O pull em si substitui o `TransferChecked` do crank cru; a escrita no livro-razão é o que transforma um movimento de token num evento de negócio. `04-pull.ts`:

```typescript
import {
  fetchPlan,
  fetchSubscriptionDelegation,
  findPlanPda,
  findSubscriptionAuthorityPda,
  findSubscriptionDelegationPda,
  getTransferSubscriptionInstructionAsync,
} from "@solana/subscriptions";
import { findAssociatedTokenPda } from "@solana-program/token";
import { address } from "@solana/kit";
import { CLUB_MINT, CLUB_TOKEN_PROGRAM, loadSigner, rpc } from "./config";
import { decidePull } from "./decide-pull";
import { recordInvoice } from "./ledger-bridge";
import { sendIxs } from "./send";

const PLAN_ID = 1n;
const SUBSCRIBER = address(process.env.SUBSCRIBER!);

async function main() {
  const merchant = await loadSigner("keys/merchant.json");

  const [planPda] = await findPlanPda({
    owner: merchant.address,
    planId: PLAN_ID,
  });
  const [subscriptionPda] = await findSubscriptionDelegationPda({
    planPda,
    subscriber: SUBSCRIBER,
  });
  const [authorityPda] = await findSubscriptionAuthorityPda({
    user: SUBSCRIBER,
    tokenMint: CLUB_MINT,
  });

  const plan = await fetchPlan(rpc, planPda);
  const sub = await fetchSubscriptionDelegation(rpc, subscriptionPda);

  // decidePull takes its five scalars positionally:
  // active, expiresAtTs, lastChargedTs, periodHours, now.
  const decision = decidePull(
    // Cancellation is not invisible on-chain: a canceled subscription's
    // delegation account persists, readable, until the subscriber revokes
    // it, and the program refuses pulls against it with its
    // subscription-cancelled error. This demo pulls the subscription you
    // created two steps ago, which cannot have been canceled yet, so it
    // passes active = true; the dunning lesson wires this argument to the
    // cancellation state it reads, so the guard's 'canceled' arm refuses
    // before a fee is spent instead of after a refusal.
    true,
    Number(sub.data.expiresAtTs),
    Number(sub.data.currentPeriodStartTs),
    Number(sub.data.terms.periodHours),
    Math.floor(Date.now() / 1000),
  );
  if (!decision.shouldPull) {
    console.log("refused:", decision.reason, "next eligible:", decision.nextEligibleTs);
    return;
  }

  const [delegatorAta] = await findAssociatedTokenPda({
    owner: SUBSCRIBER,
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });
  // destinations[0] is the treasury WALLET the plan declared; the receiving
  // token account is that wallet's ATA, derived here at pull time.
  const [receiverAta] = await findAssociatedTokenPda({
    owner: plan.data.data.destinations[0],
    mint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
  });

  const ix = await getTransferSubscriptionInstructionAsync({
    subscriptionPda,
    planPda,
    subscriptionAuthority: authorityPda,
    delegatorAta,
    receiverAta,
    caller: merchant,
    tokenMint: CLUB_MINT,
    tokenProgram: CLUB_TOKEN_PROGRAM,
    transferData: {
      amount: sub.data.terms.amount,
      delegator: SUBSCRIBER,
      mint: CLUB_MINT,
    },
  });

  const signature = await sendIxs(merchant, [ix]);

  const fresh = recordInvoice({
    kind: "subscription-pull",
    signature,
    invoiceId: `${planPda}-${sub.data.currentPeriodStartTs}`,
    plan: planPda,
    subscriber: SUBSCRIBER,
    amount: sub.data.terms.amount.toString(),
    mint: CLUB_MINT,
    pulledAt: Math.floor(Date.now() / 1000),
  });
  console.log(
    fresh
      ? `pull landed in backoffice ledger: ${signature}`
      : `duplicate pull skipped by ledger: ${signature}`,
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Rode, porque tudo depois deste passo supõe que um pull aterrissou:

```bash
CLUB_MINT=<your devnet mint> SUBSCRIBER=$(solana-keygen pubkey subscriber.json) \
  npx tsx 04-pull.ts
```

Checkpoint: `pull landed in backoffice ledger: <signature>`. Rode uma segunda vez logo em seguida e você deve receber `refused: too-early` antes de qualquer transação ser construída — essa recusa não é uma falha, é o seu crank se recusando a gastar uma taxa num pull que o programa rejeitaria de qualquer jeito. Recusas baratas são a razão inteira de a guarda existir do lado do cliente. (Com o starter do passo 6 ainda sem conserto você pode ver a primeira rodada aterrissar quando não deveria; essa é a costura que o Challenge fecha, e a trava no fim do lab é o que julga isso.)

Uma conversão ali dentro é deliberada e vale uma frase, porque o módulo 2 cravou o hábito oposto em você. Os casts de `Number(...)` são em timestamps e numa contagem de horas, nunca num valor: o `sub.data.terms.amount` continua um `bigint` até dentro da instrução, exatamente como a regra de unidades base exige. Segundos Unix e uma cadência de plano são inteiros pequenos que o JavaScript representa com exatidão por mais um quarto de milhão de anos; dinheiro não é. A guarda recebe numbers, a transferência recebe bigints, e a fronteira entre as duas é uma linha que você consegue apontar.

**7. Ligue a ponte do livro-razão (o seu passo de completion).** O `recordInvoice` ainda não existe; essa lacuna é sua. Ele acrescenta ao MESMO `backoffice/orders.jsonl` em que o receptor ao vivo da lição de webhook (`main.ts`) escreve as linhas de checkout, sob a mesma disciplina: uma linha JSON por registro, com chave na signature da transação, e uma signature já presente nunca é escrita duas vezes. Aqui está o meu, o `ledger-bridge.ts`; escreva o seu antes de espiar, depois compare:

```typescript
import { appendFileSync, existsSync, readFileSync } from "node:fs";

// Same file, same discipline as the backoffice orders ledger.
const LEDGER_PATH = "../backoffice/orders.jsonl"; // the file the live receiver (main.ts) writes

export interface InvoiceRow {
  kind: "subscription-pull";
  signature: string;
  invoiceId: string;
  plan: string;
  subscriber: string;
  amount: string; // base units, stringified bigint
  mint: string;
  pulledAt: number; // Unix seconds
}

export function recordInvoice(row: InvoiceRow): boolean {
  if (existsSync(LEDGER_PATH)) {
    const seen = readFileSync(LEDGER_PATH, "utf8")
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as { signature: string });
    if (seen.some((r) => r.signature === row.signature)) {
      return false; // exactly-once: the crank retried, the ledger did not
    }
  }
  appendFileSync(LEDGER_PATH, JSON.stringify(row) + "\n");
  return true;
}
```

Olhe o que acabou de acontecer com o seu back office. O livro-razão que registrava checkouts verificados por webhook agora registra pulls de cobrança, com a mesma garantia de exatamente-uma-vez com chave na signature, no mesmo arquivo que a sua conciliação lê. Uma cobrança de assinatura agora é registrada com a mesma disciplina de exatamente-uma-vez com chave na signature que um disco vendido no balcão. Uma diferença honesta: o pull oficial não carrega reference key nem memo, então o elo da blockchain até o livro-razão é a escrita do próprio crank, não um marcador on-chain que o seu conciliador pudesse redescobrir; se o crank morrer entre o envio e o registro, a varredura de tesouraria do módulo 4 é o que acha o órfão. Fora essa ressalva, esta é a lição em que construir o livro-razão antes da cobrança dá retorno.

Uma dobra que vale antecipar, porque a sua infraestrutura agora é boa o bastante para criar ela: o pull é uma transferência de token, então o webhook da Helius que você registrou na lição de back office TAMBÉM vai entregar ele ao seu receptor como um evento TRANSFER. O receptor vai tentar resolver ele para um pedido pelo memo, não vai achar nenhum, e vai rejeitar. Esse é o comportamento correto, não um bug para consertar. A verdade de checkout entra no livro-razão pelo pipeline de webhook, a verdade de cobrança entra pelo crank, e a chave de signature impede que os dois caminhos escrevam o mesmo pagamento duas vezes. Se os logs de rejeição te incomodam, filtre os pulls pela ATA da sua tesouraria; o que você não pode fazer é deixar o caminho do webhook escrever faturas. Um escritor por fluxo de receita.

![Eventos de checkout chegam por webhook verificado e pulls de assinatura chegam pelo crank de cobrança, convergindo como linhas de exatamente-uma-vez com chave na signature no único livro-razão de pedidos do back office.](assets/v08-diagram.png)

**8. Ponha o crank de volta no comando.** Tudo até aqui rodou como scripts avulsos, mas o artefato que esta lição entrega é o club-billing, e o que faz dele um sistema de cobrança em vez de uma demo é o loop de crank da lição passada dirigindo o caminho de pull num relógio. A refatoração leva dois minutos, e uma linha dela é estrutural de um jeito fácil de deixar passar. No `04-pull.ts`, suba o corpo do `main` para um `pullOnce(subscriber: Address)` exportado e tire a leitura da env `SUBSCRIBER` no escopo de módulo, já que o assinante é um parâmetro agora. Você ainda quer que o script avulso funcione, então mantenha o `main` que lê a variável de ambiente e chama o `pullOnce` — mas ele agora só pode rodar quando o arquivo for executado diretamente, porque o `05-crank.ts` está prestes a *importar* este arquivo, e um `main().catch(() => process.exit(1))` em nível de módulo dispararia no import e mataria o crank antes do primeiro tick dele:

```typescript
// 04-pull.ts, at the bottom. The guard is what lets one file be both
// a script and a library.
const runDirectly =
  process.argv[1] !== undefined &&
  import.meta.url === new URL(`file://${process.argv[1]}`).href;

if (runDirectly) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
```

Essa é a mesma trava de execução direta que o capstone coloca em todo arquivo de servidor, encontrada aqui primeiro. Depois o `05-crank.ts` é o formato de tick da lição passada apontado para as novas entranhas:

```typescript
// 05-crank.ts: the club-crank tick loop, now driving official pulls.
import { address } from "@solana/kit";
import { readFileSync } from "node:fs";
import { pullOnce } from "./04-pull";

const TICK_MS = 60_000;

async function tick(): Promise<void> {
  const subscribers = JSON.parse(
    readFileSync("subscribers.json", "utf8"),
  ) as string[];
  for (const s of subscribers) {
    try {
      await pullOnce(address(s));
    } catch (e) {
      console.error("pull failed for", s, e instanceof Error ? e.message : e);
    }
  }
}

tick();
setInterval(() => {
  void tick();
}, TICK_MS);
```

O `subscribers.json` é um array JSON simples de endereços de assinantes; para o lab ele contém o seu único ouvinte de teste. Em produção a lista vem de indexar as contas de delegação do programa, e indexação em escala é entregue ao curso Client-Side Mastery, o mesmo repasse que a lição de webhook fez. Repare no que o loop não faz mais: ele não lê o campo delegate da conta de token e compara ele com o próprio endereço, porque não existe par de chaves de crank com direitos de pull para comparar. A checagem de consentimento foi para on-chain, a checagem de agenda foi para dentro do `decidePull`, e o loop ficou mais burro, que é a direção correta de viagem para o componente que roda sem supervisão às 3 da manhã. A economia do pull vem da lição passada inalterada: quem chama paga a taxa base por pull, e o assinante não assina nada e não paga nada por ciclo. Checkpoint: `npx tsx 05-crank.ts` imprime uma linha `refused: too-early` por tick para o assinante que você acabou de cobrar, uma vez por minuto, e nunca submete uma transação. Assista a dois ticks, depois pare com ctrl-C, e não deixe rodando: o starter com bug que você salvou no passo 6 lê `periodHours` como segundos, então a janela de 720 "segundos" dele consideraria outro pull devido doze minutos depois do último — um pull que o programa recusa com o erro period-not-elapsed dele, uma taxa base gasta numa rejeição garantida, a cada doze minutos, até você notar. Esse loop que queima taxa é exatamente o que você conserta no Challenge.

**9. Verifique.** A trava da lição é o `subscriptions/pull.test.ts`. Ela exercita a matemática da guarda offline, depois lê o livro-razão do back office e prova que o seu pull aterrissou lá exatamente uma vez. Escreva ela agora; ela não precisa de nada além do `fs` do Node e da guarda:

```typescript
// subscriptions/pull.test.ts: the lesson's gate. Guard math first, then the ledger.
import { existsSync, readFileSync } from "node:fs";
import { decidePull } from "./decide-pull";

const LEDGER_PATH = "../backoffice/orders.jsonl"; // the file the live receiver (main.ts) writes
const HOUR = 3600;

function assert(condition: boolean, message: string): void {
  if (!condition) {
    console.error(`FAIL: ${message}`);
    process.exit(1);
  }
}

// 1. The two documented unit-and-clock bugs, as assertions.
// Args, in order: active, expiresAtTs, lastChargedTs, periodHours, now.
assert(decidePull(true, 0, 900_000, 24, 900_000 + 24 * HOUR).reason === "due", "boundary is inclusive");
assert(decidePull(true, 0, 900_000, 24, 900_000 + 23 * HOUR).reason === "too-early", "window is seconds");
assert(decidePull(true, 950_000, 900_000, 24, 960_000).reason === "expired", "expiry wins");
assert(decidePull(false, 0, 900_000, 24, 999_999).reason === "canceled", "canceled first");

// 2. The ledger: step 6's pull is in there, exactly once.
assert(existsSync(LEDGER_PATH), `no ledger at ${LEDGER_PATH}`);
const rows = readFileSync(LEDGER_PATH, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((line) => JSON.parse(line) as { kind?: string; signature: string });
const pulls = rows.filter((row) => row.kind === "subscription-pull");
assert(pulls.length > 0, "no subscription-pull row in the orders ledger");
assert(
  new Set(pulls.map((row) => row.signature)).size === pulls.length,
  "a pull signature was written twice",
);

console.log("period-window: due -> pull landed in backoffice ledger (invoice reconciled)");
```

Rode a partir da pasta `subscriptions/` — o diretório de trabalho que todo comando neste lab supõe desde o `cd subscriptions` do passo 1, e o mesmo contra o qual o caminho relativo de livro-razão da trava (`../backoffice/orders.jsonl`) está escrito:

```bash
npx tsx pull.test.ts
```

Saída esperada, literal:

```
period-window: due -> pull landed in backoffice ledger (invoice reconciled)
```

Espere ela vermelha na primeira rodada, e espere isso por um motivo nomeado: com o starter que você salvou no passo 6, a segunda asserção falha, porque uma guarda que trata `periodHours` como segundos considera devida uma assinatura de 23 horas de idade. Essa linha vermelha é a costura entre o lab e o Challenge, exatamente como o TODO da guarda do crank na lição passada; a trava fica verde quando você conserta os dois bugs abaixo.

## Challenge

A guarda de janela de período que você importou no lab é a peça solo, e o starter que eu te entrego contém, de propósito, exatamente os dois bugs documentados de unidade e relógio da teoria. A função recebe as cinco entradas dela como escalares posicionais simples, na ordem em que os campos importam, `active, expiresAtTs, lastChargedTs, periodHours, now`, que é também exatamente como o corretor (e o lab) vão chamar ela. Uma costura entre as duas superfícies: a cópia deste starter no widget de código carrega as declarações nuas, porque o corretor executa elas sem sintaxe de módulo, enquanto o arquivo que você salva mantém as palavras-chave `export` abaixo para os imports resolverem. Salve ele como `subscriptions/decide-pull.ts`, o módulo que tanto o `04-pull.ts` quanto a trava importam:

```typescript
export interface PullDecision {
  shouldPull: boolean;
  reason: string; // "due" when pulling, else why it was held
  nextEligibleTs: number; // earliest Unix second a pull may fire (0 if N/A)
}

export function decidePull(
  active: boolean, // false once CancelSubscription has run
  expiresAtTs: number, // Unix seconds; 0 = never expires
  lastChargedTs: number, // Unix seconds of the previous successful pull
  periodHours: number, // plan cadence, in HOURS (as published on the plan)
  now: number, // current chain time, Unix seconds
): PullDecision {
  if (!active) {
    return { shouldPull: false, reason: "canceled", nextEligibleTs: 0 };
  }

  // BUG: periodHours is treated as seconds, and expiry is never checked.
  const periodS = periodHours;
  const nextEligibleTs = lastChargedTs + periodS;

  if (now < nextEligibleTs) {
    return { shouldPull: false, reason: "too-early", nextEligibleTs };
  }

  return { shouldPull: true, reason: "due", nextEligibleTs };
}
```

Conserte para que tudo abaixo valha:

- Uma assinatura inativa retorna `shouldPull: false` com reason `canceled` e `nextEligibleTs: 0`, antes de qualquer outra checagem rodar.
- Uma assinatura em cima de um `expiresAtTs` diferente de zero, ou passada dele, retorna reason `expired` com `nextEligibleTs: 0`, mesmo quando a janela de período dela diz que um pull está devido.
- A janela é medida em segundos (`periodHours * 3600`), então um plano de 24 horas não cobra de novo dentro do dia. O `nextEligibleTs` reporta essa fronteira em segundos em toda decisão que tem uma, então um pull segurado diz ao painel de ops exatamente quando ele vai disparar.
- `expiresAtTs === 0` nunca é lido como "expirado no epoch"; zero quer dizer nenhuma expiração, ponto final.
- Em exatamente `lastChargedTs + period`, o pull é `due`. Fronteira inclusiva; um off-by-one aqui atrasa toda cobrança em um tick de crank para sempre.

A ordem importa: canceled primeiro, depois a expiração, depois a janela. Pergunte a si mesmo por quê antes de aceitar. A expiração de uma assinatura cancelada não tem sentido, e a janela de uma assinatura expirada não tem sentido; cada checagem só faz sentido nos sobreviventes da anterior. Erre a ordem e os seus MOTIVOS de recusa mentem mesmo quando as suas decisões de recusa estão certas, e na lição de dunning esses motivos viram entradas de máquina de estados, então mentira fica cara.

Uma nota de continuidade, porque a lição passada congelou duas strings de reason, `delegate-revoked` e `insufficient-allowance`, e prometeu que o resto deste módulo ia manter elas com sentido. Elas sobrevivem, uma camada abaixo. O `delegate-revoked` agora nomeia um evento mais raro e mais deliberado: o assinante revogou a Subscription Authority dele, o slot está vago, e todo plano embaixo dela morreu junto. O `insufficient-allowance` colapsa na única causa que sobra, uma ATA que não cobre o pull, porque a aprovação da própria SA nunca fica baixa. Os três reasons do `decidePull` entram nesse vocabulário em vez de substituir ele: a sua guarda fala antes de existir uma transação, a camada de transferência fala quando um pull falha mesmo assim, e a máquina de dunning da próxima lição consome os dois conjuntos como estados de entrada.

![O decidePull checa canceled primeiro, depois a expiração, onde expiresAtTs zero quer dizer nunca, depois uma janela de período convertida de horas para segundos, com um reason de recusa em cada saída.](assets/v09-flowchart.png)

Aceite, na devnet, a trava completa: plano criado, usuário inscrito, um pull conciliado como linha de fatura no livro-razão de pedidos, e a guarda recusando tanto um pull frequente demais (`too-early`) quanto uma assinatura expirada (`expired`). A sua evidência é um endereço de plano, uma signature de subscribe, uma signature de pull cujo id de fatura aparece no livro-razão, e os dois motivos de recusa impressos pelos seus testes.

## Checkpoint, e o que o clube já consegue fazer

Se o `pull.test.ts` está vermelho, a falha é quase com certeza uma de três, na minha experiência nesta ordem: a guarda comparou horas com segundos (o seu plano de 720 horas calcula uma janela de 720 segundos; o erro de matemática é enorme, o que ironicamente facilita achar ele num log), o caso de expiração zero curto-circuitando tudo para `expired`, ou o caminho do livro-razão apontando para um arquivo novo em vez do `orders.jsonl` do back office (o teste não acha fatura nenhuma porque você escreveu um segundo livro-razão; um clube, um livro-razão). E se as próprias chamadas on-chain estão recusando com erro 133 ou 134, revisite a checagem do binário de devnet do passo 3 do lab antes de debugar o seu próprio código; você não conserta uma defasagem de versão do lado do cliente.

Quando ela ficar verde, seja preciso sobre o que você tem agora, porque é mais do que o crank da lição passada com uma marca melhor. A Wavelength cobra qualquer número de assinantes a partir de UMA aprovação consentida cada, sobrevive aos assinantes dela entrando nos planos de outros lojistas, recusa cobranças frequentes demais e vencidas em dobro (uma vez na sua guarda, uma vez on-chain), e lança todo pull no mesmo livro-razão de exatamente-uma-vez que concilia os checkouts da loja. Receita recorrente não custodial com trilha de auditoria. Um monte de sistema em produção vai para o ar com menos.

O clube agora cobra muitos assinantes de forma não custodial e todo pull aterrissa no livro-razão. Mas um pull pode falhar por motivos que guarda nenhuma prevê: uma ATA vazia no dia da cobrança, um plano vencido, e você não consegue entrar à base de retry numa carteira que não controla. O que acontece depois de uma cobrança que falha é uma disciplina em si. Próxima lição: dunning como máquina de estados, e um olhar honesto sobre quem mais neste mercado de fato roda cobrança de assinaturas deste jeito. Traga os motivos de recusa; eles estão prestes a virar estados.
