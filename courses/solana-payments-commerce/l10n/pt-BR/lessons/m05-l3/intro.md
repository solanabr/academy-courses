# Dunning, ciclo de vida e quem mais faz isso

## Resumo

Na lição passada o clube começou a cobrar de verdade: muitos assinantes, de forma não custodial, no programa oficial Subscriptions, com todo pull bem-sucedido conciliado no livro-razão do back office como qualquer outro pagamento. Essa máquina funciona lindamente quando a ATA do assinante tem fundos. Hoje uma delas não tem. O pull mensal dispara, a transferência do delegado bate numa conta de token vazia, e a renovação falha. Nos trilhos de cartão é aqui que o dunning (a régua de cobrança) começa: a agenda de retentativas, o e-mail de atualize-seu-cartão, a escada de carência que toda empresa de SaaS roda. Nestes trilhos você estica a mão para esse manual e ela se fecha no ar. Não existe cartão para tentar de novo e não existe banco para cobrar. Você não consegue arrancar dinheiro de uma carteira que você não controla.

Então o que um sistema de cobrança não custodial adulto faz com uma renovação que falhou? A resposta que o operador mais visível do ecossistema usa nas próprias faturas: nada automático. As conclusões logo de cara:

- **Retry-never é a política, não uma lacuna.** A Helius cobra as próprias assinaturas neste mesmo programa, e a posição documentada dela é que uma renovação que falhou não é tentada de novo automaticamente contra a carteira. Ela vira uma fatura em aberto comum que o assinante pode liquidar depois.
- Você entrega o **dunning-loop**: uma máquina de estados no back office sobre o livro-razão de cobrança. Os desfechos do pull dirigem as transições: sucesso avança o ciclo, falha abre uma fatura, liquidação retoma, cancelamento enfileira a recuperação do rent.
- **RevokeAbandonedSubscription e RevokeAbandonedDelegation** fecham PDAs mortos de assinatura e de delegação e devolvem os lamports de rent deles para o payer registrado. Arranjos cancelados param de custar dinheiro para você.
- O cenário de provedores de 2026, negativas incluídas: os pay links da MoonPay Commerce suportam assinaturas, o Stripe Billing suporta assinaturas em stablecoin, e a Sphere não tem produto recorrente nenhum.

Ligue o cronômetro antes de ler, porque a rodada solo desta lição é a espera de relógio mais longa do curso e é uma espera que você não consegue comprimir. A rodada precisa de dois ciclos de cobrança completos, uma hora é o piso que a unidade `periodHours` do plano permite, então dois ciclos são duas horas de tempo real. Bote para rodar agora e leia a teoria enquanto ela corre. A partir da pasta `subscriptions/` da lição passada:

1. Crie o plano de id 2 com `periodHours: 1n` (o Challenge explica por que tem que ser um segundo plano e não uma edição: os termos do plano são imutáveis uma vez que alguém assina).
2. Faça o seu ouvinte de teste com fundos assinar ele, pelo mesmo fluxo de subscribe da lição passada, e garanta que o endereço dele é o que está em `subscribers.json`.
3. Aponte o `04-pull.ts` para o plano novo — o `PLAN_ID` dele está hardcoded como `1n` desde a lição passada — depois inicie o crank e deixe ele rodando: `CLUB_MINT=<your devnet mint> npx tsx 05-crank.ts`.

Quando você chegar no Challenge, o ciclo um já deve ter cobrado e o ciclo dois deve estar perto, e o esvaziamento entra entre o ciclo dois e o ciclo três. Se você preferir ler primeiro e rodar depois, tudo bem — só saiba que você está escolhendo ficar de fora por duas horas no fim em vez de durante.

Enquanto isso, prove a linha de base: a partir dessa mesma pasta `subscriptions/`, o diretório de trabalho de todo comando dela, rode de novo a trava dela:

```bash
npx tsx pull.test.ts
```

Você deve ver o pull aterrissar no livro-razão como uma fatura conciliada. Essa linha do livro-razão é a matéria-prima de hoje: dunning é o que acontece com as linhas que nunca chegam a ser escritas. Como o trabalho se divide hoje, em voz alta: a máquina de estados e o adaptador dela são guiados, digitados junto comigo contra as seis transições, a bancada de teste que serve de trava prova elas, e a rodada completa de dois-ciclos-mais-uma-falha no fim é solo, sem scaffold.

## A falha que o seu livro-razão assume

### Não há o que cobrar

Comece pelo porquê de o manual dos trilhos de cartão existir, para começo de conversa. Pagamentos de cartão são pagamentos pull contra um instrumento que a rede consegue tentar de novo: o lojista guarda um token do cartão do comprador, e quando uma cobrança é recusada, o processador consegue apresentar ela de novo amanhã, e na quinta, e semana que vem, escalando pelo que a indústria educadamente chama de estratégia de dunning. Retentativas funcionam ali porque a falha costuma ser transitória do lado do instrumento: um limite que reabriu, um salário que caiu, um cartão novo em arquivo. A maquinaria para alcançar dentro da conta do comprador existe, então tentar de novo é barato e muitas vezes dá certo.

Agora olhe o que o seu crank de fato tem na mão: um PDA de delegação que permite uma transferência limitada, se e somente se o limite e o saldo do assinante cobrirem ela. Quando a ATA está vazia, uma retentativa cinco minutos depois bate na mesma conta vazia. Uma retentativa às 3 da manhã bate na mesma conta vazia. Não existe emissor para reapresentar e nenhum instrumento permanente que possa ter se recarregado nos bastidores, só uma carteira cujo dono tem que agir. Toda retentativa automática é uma taxa de transação gasta para reaprender um fato que você já registrou. Meu primeiro instinto aqui ainda foi escrever a fila de retentativas. Eu estava com o arquivo aberto, `retry.ts`, a cadência do cron rascunhada num comentário, antes de admitir que o arquivo inteiro era um erro de categoria importado de outro trilho de pagamento.

A política honesta inverte o padrão, e é aqui que o dogfooding importa. A Helius roda a própria cobrança de assinaturas no programa Subscriptions da Foundation, e a política dela para uma renovação que falhou é que a cobrança não é tentada de novo automaticamente contra a carteira. O pull que falhou vira uma fatura em aberto, o assinante é avisado, e o dinheiro chega quando ele coloca saldo e liquida. Retry-never. Não retry-with-backoff, não retry-thrice-then-flag. A renovação se converte de um pull automatizado em um recebível comum, que é uma coisa com a qual o seu back office já sabe lidar, porque o módulo 4 ensinou ele a casar pagamentos que entram com pedidos em aberto.

![Os trilhos de cartão tentam de novo porque as falhas costumam ser transitórias e o processador consegue reapresentar, enquanto uma carteira vazia continua vazia até o dono agir, então a renovação vira uma fatura.](assets/v01-comparison.webp)

### A máquina, e onde ela mora

Se as retentativas sumiram, o que resta é estado. Uma assinatura, em qualquer momento, está em exatamente um de três lugares: **active**, cobrando normalmente; **open-invoice**, uma renovação falhou e existe um recebível em aberto (este é o seu estado de carência, o período em que a política de acesso é sua para escolher); ou **cancelled**, morta por escolha ou pela blockchain. O que move ela entre eles é uma lista curta de eventos: um desfecho de pull vindo do club-billing, uma liquidação, um cancel explícito.

Repare onde esta máquina roda. Não on-chain. A blockchain te dá a primitiva de pull e os limites duros dela, e vai continuar te dando exatamente isso; ela não tem opinião sobre períodos de carência, e-mails de dunning, ou se um assinante em atraso mantém acesso por três dias. O único lugar em que você controla estado é o seu próprio livro-razão, então falha e ciclo de vida moram lá, como dados. Esta é a tese silenciosa de toda a segunda metade deste curso: a blockchain é a camada de liquidação, e as decisões de produto são linhas que você escreve. Um pull ou tem sucesso, porque o limite delegado e o saldo cobriram ele, ou não tem, e uma falha irrecuperável não tem para onde tentar de novo automaticamente, então ela aterrissa no seu livro-razão como uma fatura e um estado.

As transições, exaustivamente, porque ser exaustiva é o ponto de uma máquina de estados:

- **active + pull ok** avança o ciclo: uma linha de fatura paga, o mesmo caminho de conciliação da lição passada.
- **active + pull insufficient-funds** move para open-invoice e escreve a linha da fatura. Nenhuma retentativa é agendada. Não existe efeito de retentativa para agendar.
- **active + pull cancelled** (a blockchain reporta a assinatura revogada ou o motivo canceled do guard) espelha a realidade: marca cancelled, enfileira a recuperação do rent.
- **open-invoice + pull** é recusado de cara. O crank não pode nem tentar; a máquina de estados lança. Isto é retry-never como invariante em vez de comentário.
- **open-invoice + settle** retoma: a fatura liquida, a assinatura volta para active, e o próximo ciclo cobra normalmente.
- **any + explicit cancel** marca cancelled e enfileira um RevokeAbandoned para o rent voltar para casa.

![Três estados, active, open-invoice e cancelled, com desfechos de pull, liquidação e cancel explícito dirigindo as transições; um pull contra open-invoice é recusado de cara e o cancelamento enfileira a recuperação de rent do RevokeAbandoned.](assets/v02-flowchart.webp)

### O que a carência de fato concede

O estado open-invoice tem um segundo nome nas transições acima, carência, e carência é uma decisão de produto e não uma decisão técnica. A blockchain não pausou nada quando o pull falhou. O seu livro-razão mudou uma linha. Se o assinante ainda recebe o disco deste mês é inteiramente uma questão do que o seu código de fulfillment faz quando lê aquela linha, o que quer dizer que você tem que decidir, por escrito, o que um assinante em atraso vive. A política do clube, e a que o lab assume: o acesso continua pela janela de carência. O período perdido é entregue, a fatura dele fica em aberto, e confia-se no assinante para liquidar. Isso soa generoso até você reparar que também é a opção barata: pausar o fulfillment no meio do ciclo quer dizer construir uma pausa, e tomar de volta o acesso a bens digitais que você já entregou quer dizer construir uma mentira. Um negócio com custo marginal real por período, digamos um aluguel de hardware, traçaria a linha de outro jeito e pausaria na primeira falha. Qualquer uma das duas políticas está ok. Uma política indecisa não está, porque indecisa na prática quer dizer o que quer que o seu código de fulfillment calhe de fazer, descoberto por um cliente.

A história da notificação é a outra metade da carência, e em trilhos retry-never ela deixa de ser uma cortesia. Nos trilhos de cartão um assinante pode ser cobrado de volta à saúde sem nunca ler um e-mail. Aqui o assinante é o único ator que consegue consertar a falha, então a mensagem é o mecanismo de recuperação. Três batidas bastam. Na falha: o que aconteceu, o que está devendo, e o link de pagamento carregando a reference key da fatura, porque uma mensagem sem um caminho de liquidação anexado é só má notícia. Um lembrete numa cadência declarada enquanto a fatura envelhece. Um aviso final quando o horizonte de abandono se aproxima, nomeando a data e o que acontece depois dela. Mande elas a partir dos efeitos do livro-razão, não do crank: o efeito open-invoice é o gatilho natural, o que mantém a lógica de notificação fora da máquina de estados e dentro dos consumidores, onde efeitos colaterais pertencem.

### Liquide e retome, com maquinaria que você já tem

É aqui que a escada de artefatos te paga de volta. Uma fatura em aberto precisa de um jeito de ser paga, e você construiu esse caminho inteiro no módulo 4. Dê à fatura uma reference key nova na criação, exatamente como um pedido de checkout. Diga ao assinante o que ele deve e onde, com essa reference anexada. Quando ele coloca saldo na carteira e paga, o pagamento carrega o ticket de resgate, o seu reconciliador acha ele com a mesma caminhada de busca-por-signature-mais-verificador de qualquer venda, e o resultado conciliado vira um evento settle na máquina de estados. Retomar é o `reconcileOrder` apontado para uma fatura em vez de um pedido, alimentando mais um tipo de evento num reducer.

Bom, uma ruga honesta. O pagamento de liquidação é um pagamento push comum do assinante, não um pull de delegado, então ele não toca em nada na janela de cobrança da assinatura. O seu próximo pull agendado dispara do mesmo jeito na cadência do próprio plano. Decida de propósito se uma liquidação no meio do ciclo cobre só o período perdido (a leitura simples, e a que o lab codifica) ou também desloca a âncora da cobrança futura; qualquer uma é defensável, mas o livro-razão tem que dizer qual delas você roda.

E nomeie o trade-off de frente, porque retry-never é honesto mas não é de graça. Uma renovação que falhou não se cura sozinha. Receita que os trilhos de cartão teriam recuperado em silêncio na retentativa de terça agora fica parada como recebível até um humano agir, então o seu caminho de liquidação e a sua história de notificação deixam de ser bom-ter e viram a diferença entre um estado de carência e uma máquina de churn silencioso. Você está trocando automação de recuperação de receita por custódia zero e cobranças-surpresa zero. Para um clube do disco cujos assinantes escolheram trilhos cripto de propósito, essa troca soa bem. Para um negócio cuja margem depende de recuperação passiva, é um custo real, e fingir o contrário é como esta política ganha má fama.

![Uma fatura em aberto carrega uma reference key nova; o pagamento de recarga é achado por busca de signature, verificado e conciliado num evento settle que reativa a assinatura.](assets/v03-diagram.webp)

### Recuperando o rent

Toda assinatura que um dia existiu deixou contas on-chain: o PDA da assinatura, PDAs de delegação, cada um segurando um depósito isento de aluguel medido em lamports. Pequeno individualmente, na ordem de uns dois milhões de lamports por conta, e você deveria ler o número exato direto da conta em vez de confiar no número redondo de alguém. Mas um clube que perde centenas de assinantes é silenciosamente um locador pagando aluguel de apartamentos vazios. A saída de ciclo de vida do programa existe exatamente para isso: a família de instruções RevokeAbandoned. O `RevokeAbandonedDelegation` funciona tanto para delegações fixas quanto recorrentes; o `RevokeAbandonedSubscription` fecha uma assinatura de plano abandonada. Cada um fecha a conta morta e devolve os lamports de rent dela.

Dois detalhes dos próprios docs do programa são estruturais. Primeiro, quem assina é o **payer registrado**, não o delegador, o delegatário nem o assinante. Se a sua carteira de lojista patrocinou a criação da conta lá no fluxo de subscribe, a sua carteira de lojista é quem recupera o rent, que é a razão inteira de isto pertencer à fila do seu back office e não a um botão virado para o usuário. Segundo, as duas instruções **falham se a Subscription Authority ainda estiver viva** para aquela conta. Um assinante que apenas ficou em atraso ainda tem uma SA viva; abandonado quer dizer que a autoridade em si sumiu ou está obsoleta. Enquanto a SA vive, o caminho de revoke comum é a saída correta no lugar. O seu consumidor de limpeza checa essa precondição antes de submeter, e trata o caso de falha como rotina, não como bug.

O que traz à tona o segundo trade-off da lição: recuperar rent é dinheiro de verdade de volta, mas só depois de você decidir que um arranjo está realmente morto, e essa decisão é irreversível de um jeito que os lamports não capturam. Uma vez revogada, a conta da assinatura é fechada e um assinante que volta tem que assinar de novo do zero: cerimônia de signature nova, conta nova, atrito de onboarding novo. (O programa entrega sim um `resumeSubscription` on-chain para um cancelamento que o assinante agendou e depois se arrependeu antes do revoke, guardado pela expiração que ele observou na hora de assinar, mas essa é uma porta diferente e mais estreita; ela não consegue ressuscitar uma conta fechada.) Recupere uma semana depois de um pull que falhou e você converteu um cliente em estado de carência num problema de reaquisição para economizar dois milhões de lamports. Trate o abandono como uma transição explícita e considerada: na política do clube, uma fatura em aberto que envelhece além de um horizonte declarado, ou um cancel explícito, e nada mais brando.

![Um pull que falhou passa por carência e lembretes até um horizonte declarado ou um cancel explícito, depois do qual o RevokeAbandoned devolve o rent; recuperar durante a carência força uma reassinatura completa.](assets/v04-timeline.webp)

### Quem mais faz isso

Afaste o zoom do livro-razão da Wavelength para o mercado, porque construir-ou-comprar é uma pergunta real e 2026 finalmente dá respostas reais para ela. Imagine o cenário de cobrança recorrente como três bancas numa feira de rua, cada uma vendendo uma coisa diferente, e uma delas, importante, não vendendo o que a placa dela pode te fazer supor.

Antes da caminhada, uma definição, porque a pergunta inteira de construir-ou-comprar gira em torno dela. Um merchant of record é a entidade que legalmente vende para o comprador: ela recebe o pagamento em nome próprio, assume as obrigações de reembolso e de disputa, cuida do imposto onde imposto se aplica, e te repassa depois. Quando você terceiriza a cobrança recorrente para um provedor hospedado, normalmente você está comprando alguma fatia desse arranjo, e o preço da fatia é ele ficar entre o seu cliente e o seu dinheiro. Cobrança não custodial é o canto oposto: você é o merchant of record, os fundos do assinante vão direto da carteira dele para a sua, e toda obrigação que o provedor teria absorvido, dunning bem incluído, é sua para construir, que é o que este módulo tem sido. Nenhum dos dois cantos é a escolha adulta em geral. O movimento adulto é saber qual dos dois você está rodando, porque os modos de falha diferem: um provedor pode segurar ou congelar os seus repasses, enquanto os seus próprios trilhos podem falhar uma renovação sem ninguém além de você em posição de perceber.

![Um merchant of record vende em nome próprio e assume dunning e reembolsos, aceitando retenções de repasse; a cobrança não custodial move fundos de carteira para carteira e deixa toda obrigação com você.](assets/v05-comparison.webp)

**MoonPay Commerce**, a plataforma antes conhecida como Helio, vende checkout hospedado: os pay links dela suportam assinaturas, então um lojista consegue levantar cobrança recorrente sem nenhuma integração com programa. A banca está movimentada; o apanhado de abril de 2026 do solana.com reporta a MoonPay Commerce com mais de quarenta milhões de dólares em volume de pagamento único desde o lançamento dela em outubro de 2025, 88 por cento disso na Solana. Repare no que esse número datado mede, pagamentos únicos, o que te diz que checkout hospedado é a metade provada e recorrente é a prateleira mais nova em cima dela. O **Stripe Billing** vende o pacote adjacente ao cartão: assinaturas em stablecoin dentro do mesmo produto de cobrança que roda metade das faturas de SaaS da internet, o que quer dizer lógica de dunning, pró-rata e tratamento de imposto que você não escreve, em troca de o Stripe ficar entre você e o trilho. E a **Sphere** vende infraestrutura de pagamentos, ramps, OTC e o corredor do PIX para liquidação instantânea em trilho bancário, e aqui está o resultado negativo que vale mais que a maioria dos positivos: a Sphere não tem produto recorrente nenhum. Nada de errado com a Sphere; muito de certo nela para fluxos avulsos. Mas supor que todo provedor de pagamentos oferece cobrança recorrente é precisamente como um time queima uma sprint de integração descobrindo que a feature que ele escopou não existe. Verifique o suporte a recorrente por fornecedor, por escrito, antes de arquitetar em torno dele.

A regra de decisão cai limpa. Construa em cima do programa da Foundation, como este curso fez, quando você quer não custodial e on-chain: os fundos do assinante nunca ficam com um intermediário, os limites são impostos pelo programa, e o ciclo de vida é seu, que é exatamente por que você teve que escrever a máquina de estados desta lição você mesmo. Recorra a um provedor quando você quer um produto hospedado e adjacente ao cartão e está contente em herdar a política de ciclo de vida dele junto com os e-mails de dunning dele. O que observar, se você pegar a estrada do provedor: se o produto recorrente do fornecedor é uma primitiva de primeira classe ou um loop de pay link, quem detém a custódia entre a cobrança e a liquidação, e qual é de fato a política dele para renovação que falha, porque agora você sabe que é uma política, não física.

![Quatro opções de cobrança recorrente comparadas: o programa da Foundation impõe recorrência não custodial on-chain, a MoonPay Commerce oferece pay links de assinatura, o Stripe Billing cobra assinaturas em stablecoin, e a Sphere não oferece nenhuma.](assets/v06-table.webp)

## Lab: construa o dunning-loop

Cinco passos. Este artefato é uma máquina de estados pura sobre dados do livro-razão, o que te compra uma coisa rara neste curso: zero dependências novas e zero drama de workspace. O workspace `dunning` não importa nada de `@solana/kit`, então ele fica inteiramente fora tanto do pin de kit ^6 quanto do de kit ^7 (esses pins, congelados em 2026-08, vivem onde as chamadas à blockchain vivem: checkout e ops no ^6, subscriptions no ^7). Ele roda com `tsx`, que está nas devDependencies do repo desde a lição do transfer-kit; se você por algum motivo estiver começando limpo, `npm install -D tsx typescript` põe ele de volta.

**Passo 1: scaffold.** Um workspace novo ao lado dos outros, e porque é TypeScript puro a instalação é minúscula. A partir da raiz `wavelength`:

```bash
mkdir dunning && cd dunning
npm init -y
npm pkg set type=module
npm install -D tsx typescript @types/node
```

Depois adicione `"dunning"` ao array `workspaces` da raiz do jeito que você fez para todo workspace desde a lição de conciliação. Três arquivos são criados nos passos abaixo, todos por você: `statemachine.ts` (a máquina), `outcome.ts` (a ponte do vocabulário do club-billing) e `statemachine.test.ts` (a trava).

**Passo 2: a máquina de estados.** Abra `dunning/statemachine.ts`. Os tipos vêm primeiro, porque neste arquivo os tipos são metade da política:

```ts
// dunning/statemachine.ts
export type SubscriptionState = 'active' | 'open-invoice' | 'cancelled';

export type PullOutcome =
  | { kind: 'ok'; signature: string; amountBaseUnits: bigint }
  | { kind: 'insufficient-funds'; amountBaseUnits: bigint }
  | { kind: 'cancelled' };

export type DunningEvent =
  | { type: 'pull'; period: number; outcome: PullOutcome }
  | { type: 'settle'; invoiceId: string; signature: string }
  | { type: 'cancel'; reason: string };

// Note what this vocabulary refuses to express: there is no
// 'schedule-retry' effect. Retry-never is unrepresentable-by-type,
// the same trick policy.ts pulled on fulfill-anyway last module.
export type LedgerEffect =
  | { effect: 'record-paid-invoice'; invoiceId: string; period: number; signature: string; amountBaseUnits: string }
  | { effect: 'open-invoice'; invoiceId: string; period: number; amountBaseUnits: string }
  | { effect: 'settle-invoice'; invoiceId: string; signature: string }
  | { effect: 'mark-cancelled'; reason: string }
  | { effect: 'enqueue-revoke-abandoned'; subscriptionPda: string; instruction: 'RevokeAbandonedSubscription' };

export type Subscription = {
  id: string;
  subscriptionPda: string;
  state: SubscriptionState;
  openInvoiceId?: string;
};

export type Transition = { next: SubscriptionState; effects: LedgerEffect[] };
```

Valores serializam como strings, a mesma regra de bigint-em-disco que o livro-razão impõe desde o módulo 4. Depois o reducer, guiado em vez de retido, porque as seis transições da seção de teoria já dizem a resposta inteira; digite ele braço por braço, conferindo cada um contra a linha da transição dele, e guarde o pensamento reservado para a rodada solo na devnet, onde nenhuma listagem vai te ajudar.

```ts
const invoiceId = (sub: Subscription, period: number) => `inv:${sub.id}:${period}`;

export function transition(sub: Subscription, event: DunningEvent): Transition {
  switch (sub.state) {
    case 'active': {
      if (event.type === 'pull') {
        const { outcome, period } = event;
        if (outcome.kind === 'ok') {
          return {
            next: 'active',
            effects: [{
              effect: 'record-paid-invoice',
              invoiceId: invoiceId(sub, period),
              period,
              signature: outcome.signature,
              amountBaseUnits: outcome.amountBaseUnits.toString(),
            }],
          };
        }
        if (outcome.kind === 'insufficient-funds') {
          return {
            next: 'open-invoice',
            effects: [{
              effect: 'open-invoice',
              invoiceId: invoiceId(sub, period),
              period,
              amountBaseUnits: outcome.amountBaseUnits.toString(),
            }],
          };
        }
        // outcome.kind === 'cancelled': the chain said no; mirror it.
        return cancelTransition(sub, 'revoked-on-chain');
      }
      if (event.type === 'cancel') return cancelTransition(sub, event.reason);
      throw new Error(`no ${event.type} transition from active`);
    }

    case 'open-invoice': {
      if (event.type === 'pull') {
        // THE load-bearing refusal: an open invoice is never retried
        // against the wallet. The crank must not even ask.
        throw new Error(
          `refusing pull for ${sub.id}: open invoice ${sub.openInvoiceId}; retry-never`,
        );
      }
      if (event.type === 'settle') {
        if (event.invoiceId !== sub.openInvoiceId) {
          throw new Error(`settle for unknown invoice ${event.invoiceId}`);
        }
        return {
          next: 'active', // resume
          effects: [{ effect: 'settle-invoice', invoiceId: event.invoiceId, signature: event.signature }],
        };
      }
      return cancelTransition(sub, event.reason);
    }

    case 'cancelled': {
      if (event.type === 'settle' && event.invoiceId === sub.openInvoiceId) {
        // A receivable collected after cancellation still settles;
        // the subscription itself stays dead.
        return {
          next: 'cancelled',
          effects: [{ effect: 'settle-invoice', invoiceId: event.invoiceId, signature: event.signature }],
        };
      }
      throw new Error(`no ${event.type} transition from cancelled`);
    }
  }
}

function cancelTransition(sub: Subscription, reason: string): Transition {
  return {
    next: 'cancelled',
    effects: [
      { effect: 'mark-cancelled', reason },
      {
        effect: 'enqueue-revoke-abandoned',
        subscriptionPda: sub.subscriptionPda,
        instruction: 'RevokeAbandonedSubscription',
      },
    ],
  };
}
```

Duas decisões aqui são deliberadas. O braço de pull do open-invoice lança em vez de devolver um no-op, porque um no-op silencioso convida um bug de crank em que retentativas acontecem e nada percebe; um invariante lançado transforma o mesmo bug num alerta de plantão. Uma consequência para o loop do crank da lição passada: o `catch` por assinante dele loga e segue em frente, o que engoliria em silêncio exatamente esse throw. Quando você rotear o crank pela máquina na rodada solo, ou cheque o estado do livro-razão antes de chamar o `pullOnce` (mais limpo), ou relance a partir do catch quando a mensagem começar com `refusing pull`; um invariante engolido é a falha das 3 da manhã que este braço existe para prevenir. E cancelamento a partir de qualquer estado vivo é roteado por um único `cancelTransition`, então existe exatamente um lugar na base de código em que a recuperação de rent é enfileirada, que é o lugar em que você vai depois adicionar a checagem do horizonte de abandono.

**Passo 3: o adaptador de desfecho.** A máquina de estados fala `PullOutcome`; o club-billing fala decisões de `decidePull` e resultados de envio. O `dunning/outcome.ts` é a ponte de dez linhas:

```ts
// dunning/outcome.ts
import type { PullOutcome } from './statemachine';

// The shape last lesson's period-window guard returns, verbatim.
export interface PullDecision {
  shouldPull: boolean;
  reason: string; // 'due' when pulling, else 'canceled' | 'expired' | 'too-early'
  nextEligibleTs: number;
}

export type PullAttempt =
  | { ok: true; signature: string }
  | { ok: false; error: 'insufficient-funds' | 'other'; detail?: string };

export function outcomeFromPull(
  decision: PullDecision,
  attempt: PullAttempt | null,
  amountBaseUnits: bigint,
): PullOutcome | null {
  if (!decision.shouldPull) {
    // Cancelled on-chain is a state-machine event; too-early and
    // expired are non-events for dunning (the crank just moves on).
    // NOTE the spelling seam, on purpose and documented: the guard's
    // reason string is 'canceled' (one L, the client's US spelling);
    // the state machine's own vocabulary is 'cancelled' (two Ls).
    // The comparison below is on the guard's side. Keep it one-L.
    return decision.reason === 'canceled' ? { kind: 'cancelled' } : null;
  }
  if (!attempt) throw new Error('decision said pull, but no attempt was made');
  if (attempt.ok) return { kind: 'ok', signature: attempt.signature, amountBaseUnits };
  if (attempt.error === 'insufficient-funds') {
    return { kind: 'insufficient-funds', amountBaseUnits };
  }
  // RPC hiccups and blockhash expiry are NOT dunning events: the pull
  // never legally failed, so the crank retries the SUBMISSION next
  // tick. Retry-never bans retrying the debt, not the plumbing.
  return null;
}
```

Esse último ramo é a linha mais sutil do lab, então que fique claro: retry-never governa a renovação, não a rede. Uma transação que expirou antes de aterrissar nunca cobrou ninguém e nunca falhou contra um saldo; reenviar ela é encanamento, não dunning. Só um pull que a blockchain de fato rejeitou por fundos insuficientes se converte em uma fatura em aberto.

**Passo 4: ligue o settle e a fila de revoke.** O lado do settle é uma ponte, não um sistema, e ele mora no workspace de ops do kit-v6 porque é virado para a blockchain (o reducer de dunning que ele alimenta é TypeScript puro e não importa de kit nenhum). Crie `backoffice-refunds/src/settle-invoices.ts`:

```ts
// backoffice-refunds/src/settle-invoices.ts: open invoices get paid through
// the exact machinery a checkout order uses. Two functions, one timer.
import { generateKeyPairSigner, address, type Address } from '@solana/kit';
import { reconcileOrder } from './reconcile';
import { recordOrder } from './ledger';

const MERCHANT = address(process.env.MERCHANT_ADDRESS!);
const MERCHANT_ATA = address(process.env.MERCHANT_ATA!);
const CLUB_MINT = address(process.env.CLUB_MINT!);

// Called from the open-invoice effect: mint the invoice a claim ticket and
// register it as an open order, exactly like a checkout does at page load.
export async function openInvoiceReference(
  invoiceId: string,
  amountBaseUnits: bigint,
): Promise<Address> {
  const reference = (await generateKeyPairSigner()).address;
  recordOrder(
    {
      orderId: invoiceId,
      recipient: MERCHANT,
      recipientAta: MERCHANT_ATA,
      mint: CLUB_MINT,
      amountBaseUnits,
    },
    reference,
  );
  return reference;
}

// Called on a timer for every open invoice: a paid reconcile becomes the
// settle event your dunning reducer consumes.
export async function settleTick(
  invoiceId: string,
  reference: Address,
): Promise<{ type: 'settle'; invoiceId: string; signature: string } | null> {
  const result = await reconcileOrder(reference);
  return result.status === 'paid'
    ? { type: 'settle', invoiceId, signature: result.signature }
    : null;
}
```

O loop do timer em si são três linhas de `setInterval` em volta do `settleTick`, e o evento devolvido vai direto para o `transition`. Nada aqui é maquinaria nova: `recordOrder` e `reconcileOrder` são os exports da lição de conciliação, fazendo por uma fatura exatamente o que fazem por um pedido. O lado do revoke drena os efeitos enfileirados. O consumidor monta a instrução com o client que você instalou na lição passada (`@solana/subscriptions`, pinado em 0.5.0 no workspace de kit ^7, pin congelado em 2026-08), exatamente como os docs do programa moldam:

```ts
import { getRevokeAbandonedSubscriptionInstruction } from '@solana/subscriptions';

const ix = getRevokeAbandonedSubscriptionInstruction({
  payer: payerSigner,             // the recorded payer signs, nobody else
  subscriptionAccount: subscriptionPda,
  subscriptionAuthority: subscriptionAuthorityPda,
  planPda,
});
```

Antes de submeter, o consumidor checa a precondição sobre a qual os docs avisam: se a Subscription Authority ainda estiver viva para aquela conta, a instrução falha por design, e o caminho de revoke comum é a saída correta no lugar. Logue a recusa e reenfileire com o motivo; uma fila de limpeza que derruba falhas em silêncio é como contas vazam para sempre.

O mesmo padrão de drenar efeitos carrega a história da notificação da metade teórica. Inscreva um segundo consumidor nos efeitos `open-invoice` e faça ele mandar a mensagem de falha com a reference key da fatura anexada; a cadência de lembretes e o horizonte de aviso final rodam a partir da idade da fatura, lida do livro-razão no mesmo timer que roda o reconciliador. Nenhuma arquitetura nova, só mais um leitor dos efeitos que você já emite, que é o argumento inteiro para fazer a máquina de estados devolver efeitos em vez de executar eles.

**Passo 5: escreva e rode a trava.** Crie `dunning/statemachine.test.ts`. Ele conduz dois ciclos bem-sucedidos, força um desfecho de fundos insuficientes, afirma que a recusa de pull-durante-open-invoice de fato lança, afirma que nenhum efeito de retentativa existe em lugar nenhum do rastro, liquida e cancela:

```ts
// dunning/statemachine.test.ts: the lesson's gate. Pure logic, no chain.
import { transition, type Subscription, type LedgerEffect } from './statemachine';

function fail(msg: string): never {
  console.error(`FAIL: ${msg}`);
  process.exit(1);
}

let sub: Subscription = { id: 'club-77', subscriptionPda: 'SubPda11111111111111111111111111111111111111', state: 'active' };
const trace: LedgerEffect[] = [];
function apply(t: { next: Subscription['state']; effects: LedgerEffect[] }, openInvoiceId?: string): void {
  sub = { ...sub, state: t.next, openInvoiceId: openInvoiceId ?? sub.openInvoiceId };
  trace.push(...t.effects);
}

// 1. Two successful cycles keep the subscription active.
for (const period of [1, 2]) {
  const t = transition(sub, {
    type: 'pull',
    period,
    outcome: { kind: 'ok', signature: `sig${period}`, amountBaseUnits: 15_000_000n },
  });
  if (t.next !== 'active') fail(`cycle ${period} should stay active`);
  apply(t);
}

// 2. Insufficient funds opens an invoice, schedules nothing.
const t3 = transition(sub, {
  type: 'pull',
  period: 3,
  outcome: { kind: 'insufficient-funds', amountBaseUnits: 15_000_000n },
});
if (t3.next !== 'open-invoice') fail('failed pull must open an invoice');
const opened = t3.effects.find((e) => e.effect === 'open-invoice');
if (!opened || opened.effect !== 'open-invoice') fail('open-invoice effect missing');
apply(t3, opened.invoiceId);

// 3. Retry-never is an invariant: a pull against open-invoice throws.
let threw = false;
try {
  transition(sub, {
    type: 'pull',
    period: 4,
    outcome: { kind: 'ok', signature: 'sigX', amountBaseUnits: 15_000_000n },
  });
} catch {
  threw = true;
}
if (!threw) fail('pull during open-invoice must throw (retry-never)');

// 4. No retry effect exists anywhere in the trace (or the vocabulary).
if (trace.some((e) => e.effect.includes('retry'))) fail('a retry effect leaked into the ledger');

// 5. Settlement resumes.
const t5 = transition(sub, { type: 'settle', invoiceId: sub.openInvoiceId!, signature: 'sigSettle' });
if (t5.next !== 'active') fail('settle must resume the subscription');
apply(t5);

// 6. Explicit cancel enqueues rent recovery.
const t6 = transition(sub, { type: 'cancel', reason: 'user-request' });
if (t6.next !== 'cancelled') fail('cancel must land in cancelled');
if (!t6.effects.some((e) => e.effect === 'enqueue-revoke-abandoned')) {
  fail('cancel must enqueue RevokeAbandoned');
}
apply(t6);

console.log('failed renewal -> open-invoice (no wallet retry); settle -> resume; cancel -> RevokeAbandoned enqueued');
```

Rode ele a partir da pasta `dunning`:

```bash
npx tsx statemachine.test.ts
```

Saída esperada:

```
failed renewal -> open-invoice (no wallet retry); settle -> resume; cancel -> RevokeAbandoned enqueued
```

Se a afirmação da recusa falhar, o seu braço de open-invoice amoleceu; se a afirmação de zero retentativas falhar, um efeito se infiltrou no seu vocabulário que deveria ser irrepresentável. As duas são a lição, imposta.

## Challenge

As transições foram trabalho de completion. A rodada é solo, na devnet, contra o clube de verdade da lição passada.

Os termos do plano são imutáveis uma vez que alguém assina (os campos `expected*` que você assinou fixaram os termos que você aceitou; a imutabilidade em si é obra do programa — o UpdatePlan nunca toca nos termos), então comprimir o período quer dizer um segundo plano, não uma edição: crie o plano de id 2 com `periodHours: 1n` (uma hora é o piso que a unidade do plano permite), faça o seu ouvinte de teste com fundos assinar ele, e deixe a delegação do plano antigo quieta; ela mantém o próprio relógio, e você pode dar unsubscribe nela depois. Seja honesto consigo mesmo sobre o relógio de parede que isso compra: dois ciclos completos num plano de hora em hora são duas horas, então inicie o crank cedo na sessão, deixe ele correr, e faça o esvaziamento entre o ciclo dois e o ciclo três. Rode esses dois ciclos e veja duas linhas de fatura paga aterrissarem no livro-razão do back office. Repare em quem escreve elas, porque este é o único lugar em que os dois escritores de livro-razão do curso são fáceis de confundir: o crank escreve as linhas de cobrança ele mesmo, chaveadas pela signature do pull, exatamente como a lição passada estabeleceu. O reconciliador do módulo 4 nunca vê elas — ele casa faturas ABERTAS por reference key, e um pull oficial não carrega nem reference nem memo. Ficar olhando o `reconcileOrder` aqui e concluir que o seu pipeline está quebrado é a curva errada previsível; olhe o `orders.jsonl` no lugar. Agora force a falha: esvazie a ATA do assinante de teste (mande o saldo de USDC-dev dela para outro lugar a partir da carteira do assinante), deixe o crank disparar o terceiro pull, e prove que a falha aterrissa como fatura em aberto, não como retentativa contra a carteira: o seu livro-razão tem que mostrar uma linha de fatura em aberto e zero tentativas de pull a mais contra aquela assinatura, o que a recusa lançada pela máquina de estados garante se o seu crank rotear por ela. Depois percorra as duas saídas. Saída um: recarregue a ATA, pague a fatura pela reference key dela, e confirme que o evento settle retoma a assinatura e que o próximo ciclo cobra normalmente. Saída dois: levante um segundo ouvinte de teste para ela — um keypair novo com um pouco de SOL de devnet e USDC-dev, pelo mesmo fluxo de subscribe da lição passada — e faça ele assinar o plano de hora em hora. Depois percorra o desmonte completo na ordem: cancele a assinatura dele de uma vez, revogue a Subscription Authority dele a partir da carteira daquele assinante (a porta de saída unilateral da lição passada; este é o passo que torna as contas *abandonadas* em vez de meramente em atraso, e repare que revogar a SA não devolve por si só o rent dos PDAs — ela só limpa a precondição que estava bloqueando o RevokeAbandoned), rode o consumidor da fila de revoke, e confirme que o RevokeAbandoned aterrissa, checando os lamports de rent chegando de volta no saldo do payer registrado — a sua carteira de lojista, se ela patrocinou o subscribe. Mantenha o seu primeiro ouvinte fora desta saída: a Subscription Authority dele tem que continuar viva para a cobrança retomada da Saída um, e um consumidor apontado para ele vai logar a recusa de SA-viva e reenfileirar — a precondição projetada, não um bug.

Aceite: um rastro do livro-razão mostrando dois ciclos pagos, uma fatura em aberto sem nenhuma retentativa automática contra a carteira, um settle-e-retoma, e um cancel cuja signature de recuperação de rent você consegue colar. Esse rastro, todas as cinco batidas dele, é o artefato.

![O rastro de aceite corre cinco batidas: dois ciclos pagos, uma falha de ATA esvaziada aterrissando como fatura em aberto com zero retentativas, uma liquidação chaveada por reference, e um cancelamento recuperando rent.](assets/v07-diagram.webp)

## Checkpoint, e o que o clube finalmente aguenta

Se a bancada de teste ou a rodada na devnet brigaram com você, os suspeitos prováveis na ordem: o braço de pull do open-invoice devolvendo um valor em vez de lançar (a afirmação pega isso, mas o custo real teria sido um crank alegremente tentando de novo para sempre), o adaptador de desfecho convertendo uma falha de RPC em fatura em aberto (encanamento confundido com dívida; o seu assinante leva cobrança por um timeout), ou o consumidor de revoke submetendo enquanto a Subscription Authority ainda está viva e lendo errado a falha projetada como bug. Cada um é uma correção de duas linhas depois de nomeado em voz alta.

Dê um passo atrás e olhe o formato do que você construiu ao longo deste módulo, porque agora é uma máquina só. O clube cobra muitos assinantes sem custódia, todo pull concilia no mesmo livro-razão de toda venda, uma renovação que falhou degrada para um recebível comum em vez de uma tempestade de retentativas, a liquidação retoma ela pelos exatos trilhos que um pagamento de checkout usa, e arranjos mortos devolvem o rent deles. Cobrando, falhando com elegância, limpando a própria bagunça. O clube do disco do mês é um negócio de assinatura de verdade em trilhos que não entregam nenhuma lógica de negócio de assinatura, e você sabe precisamente quais partes são física e quais partes são a sua política, porque você escreveu a política como tipos.

Todo comprador até aqui, porém, chegou pré-carregado: USDC na carteira, carteira na mão. O próximo módulo abre a porta pela qual o resto do mundo entra: a fronteira fiat. Botar dinheiro para dentro e para fora, e saber exatamente quem é merchant-of-record em cada costura.
