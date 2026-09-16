# Conciliação, reembolsos e pagamentos parciais: o manual que falta

## Resumo

Na lição passada o back office ganhou ouvidos: ele ingere webhooks da Helius de forma idempotente, verifica todo evento on-chain antes de acreditar nele, e escreve um livro-razão de pedidos chaveado pela signature da transação, cumprindo exatamente uma vez. Hoje um comprador testa o que ele ainda não consegue fazer. Ele comprou uma prensagem de uma sessão ao vivo, o disco chegou empenado, e ele quer o dinheiro de volta. Você abre a documentação de pagamentos da Solana atrás do fluxo de reembolso e não acha nada, porque os chargebacks foram projetados para fora dos trilhos de propósito. A reversão é sua para construir.

Antes de qualquer coisa nova, dois minutos de arrumação que este módulo vinha adiando, e depois prove que a base ainda responde. As pastas de ops que você criou como irmãs soltas entram no workspace `wavelength` agora, para poderem importar umas às outras por nome (`verifier`, `transfer-kit`) em vez de por caminho relativo. Importar por nome exige duas coisas, e a escalação só fornece a primeira. Registrar as pastas como workspaces faz o npm criar um symlink de cada pacote dentro do `node_modules` da raiz sob o nome do pacote (o `npm init -y` nomeou cada um segundo a pasta dele). Mas quando um arquivo então diz `from 'verifier'`, o Node segue esse symlink até o `package.json` do pacote e abre o que quer que o `main` nomeie, e o `npm init -y` escreveu `"main": "index.js"`, um arquivo que nenhum dos dois pacotes tem. Aponte o `main` para a porta de entrada real de cada pacote, em vez disso. O `transfer-kit` tem um barrel em `src/index.ts` desde o módulo 2; o verificador nunca ganhou um, porque até hoje todo consumidor importava os arquivos dele por caminho relativo. Dê a ele a mesma porta de entrada:

```ts
// verifier/src/index.ts
export { createVerifier } from './verify.ts';
export { createMemoryStore } from './store.ts';
export { createRpcFetchTransaction } from './rpc.ts';
export type { ExpectedOrder, VerifyResult, RejectReason } from './types.ts';
```

Então, a partir da raiz `wavelength`, registre a escalação e mire os dois campos `main` (o tsx, que roda todo script deste curso, resolve uma entrada TypeScript diretamente):

```bash
cd ~/wavelength
npm pkg set --json workspaces='["transfer-kit","verifier","backoffice","backoffice-refunds"]'
npm pkg set type="module"
npm pkg set main="src/index.ts" --workspace transfer-kit --workspace verifier
npm install
npm run --workspace backoffice verify:backoffice
```

Você deve ver o webhook entregue em triplicata colapsar em exatamente uma linha do livro-razão e o evento forjado quicar no verificador. Esse livro-razão, chaveado pela signature, é o substrato sobre o qual tudo hoje se apoia. Se o smoke check falhar, conserte a lição passada primeiro; um conciliador em cima de um livro-razão quebrado só automatiza confusão.

Com a base respondendo, de saída, o que hoje estabelece:

- Você entrega o **backoffice-refunds**: um conciliador que casa pagamentos com pedidos pela reference key (a metade do memo sai como um TODO já ligado que o tráfego do módulo 7 completa), e um builder de reembolso que empurra stablecoins de volta para a carteira de origem através do transfer-kit, registrando todo reembolso no livro-razão ligado à signature de origem dele.
- A conciliação converge. Uma reference key do Solana Pay e um id de fatura no `extra.memo` do x402 são a mesma ideia, um id de pedido carregado num campo pesquisável, sem endereço de depósito único nenhum. Um conciliador serve os dois trilhos. Uma varredura de tesouraria pega o dinheiro que não casa com nada.
- Reembolsos são construção original, não citação de documentação. A gente segue o único precedente da indústria que existe: a Stripe devolve reembolsos de cripto como stablecoins para a carteira de origem, e a gente também vai.
- Pagar a mais, pagar a menos e pagamentos parciais recebem políticas nomeadas, nunca padrões silenciosos. Toda escolha aqui é um trade-off declarado.
- Uma regra está acima de tudo isso: um reembolso nunca sai da sua carteira enquanto o pagamento de origem não tiver finalizado. Um reembolso push é irreversível no instante em que aterrissa.

Como o trabalho se divide neste módulo: este é o trecho que puxa para o completion. O conciliador e a varredura chegam como esqueletos com TODOs marcados, do mesmo jeito que o registry de claims e a escrita no livro-razão da lição passada chegaram, e o builder de reembolso chega pronto, sob a teoria de que o único arquivo em que um erro gasta dinheiro de verdade é um arquivo que você deveria ler antes de escrever. A política de pagamento parcial no fim é solo, sem apoio, porque uma política que outra pessoa escreveu para você é exatamente o padrão silencioso que esta lição existe para matar.

## O conciliador e o pagamento reverso, de perto

### Um campo, dois trilhos

Comece pelo problema que a conciliação resolve. O dinheiro chega na ATA da sua tesouraria como um fluxo de transferências. Os pedidos vivem no seu livro-razão como linhas. Nada numa transferência crua diz qual pedido ela paga, e você tem uma conta de USDC, não uma por cliente. A correção clássica, a que as corretoras de cripto usam, é um endereço de depósito único por usuário: derive milhares de contas, observe todas elas, varra elas o tempo todo. Infraestrutura de verdade, custo de verdade, e para uma loja de discos, absurdo.

Os trilhos que você vem usando desde a lição do transfer-kit já carregam a resposta melhor. Todo pagamento que o seu checkout constrói inclui uma **reference key**: um valor base58 novo de 32 bytes anexado à transação como uma conta não signatária. Ela não faz nada on-chain. Ela existe para você conseguir achar a transação depois, porque busca de signature por conta é uma capacidade nativa do RPC. Gere uma por pedido, guarde ela na linha do pedido, e o pagamento carrega a própria ficha de retirada.

Aqui está a parte que vale pausar. No módulo 7 você vai conhecer o x402, o protocolo de pagamento nativo de HTTP, e as faturas dele carregam um campo `extra.memo` guardando um id de fatura. Trilho diferente, spec diferente, exatamente a mesma ideia: um identificador de pedido enfiado num slot pesquisável do próprio pagamento, para o lojista precisar de uma conta e uma consulta em vez de uma fábrica de endereços. A convergência não é coincidência. Qualquer trilho de push sem endereços de depósito tem que resolver o casamento, e um id de pedido pesquisável é a solução mínima. O que quer dizer que o conciliador que você escreve hoje é o padrão geral, e não encanamento do Solana Pay, e o módulo 7 vai plugar a metade do x402 nele sem reescrita. (Crédito a quem merece: a spec de reference do Solana Pay e o helper `findReference` dela entregaram este padrão primeiro; a gente constrói o nosso direto em cima do kit para um conciliador servir os dois trilhos.)

![As reference keys do Solana Pay e os ids de fatura no memo do x402 carregam, cada um, um id de pedido num campo pesquisável, então um conciliador casa qualquer um dos trilhos com o livro-razão de pedidos.](assets/v01-diagram.webp)

### Da reference ao pedido

Mecanicamente, a conciliação é uma caminhada de três passos. Pegue a reference key do pedido. Peça ao RPC as signatures que mencionam aquele endereço; o `getSignaturesForAddress` devolve elas das mais novas para as mais velhas, incluindo transações que falharam, então a caminhada filtra por `err === null`. Então, e este é o passo que te mantém honesto, rode cada candidata pelo verificador que você construiu no começo deste módulo. A reference key prova que alguém anexou a sua ficha de retirada a uma transação. Ela não prova que a transação te pagou o valor certo do token certo a partir do programa certo. Anexar uma conta arbitrária a uma transação é de graça; é isso que torna as references pesquisáveis, e também o que as torna falsificáveis como prova de pagamento. O verificador é o juiz, a reference é só o endereço do tribunal.

A saída do conciliador é deliberadamente mais rica do que pago-ou-não. O verificador computa o delta de saldo real na sua ATA em unidades base, e comparar esse delta com o preço do pedido divide o mundo em quatro estados honestos: **paid** (exato), **underpaid**, **overpaid** e **unmatched** (nada verificável encontrado). O livro-razão da lição passada só registrava o caminho feliz, porque o fulfillment travava na verificação exata. Hoje os outros três estados deixam de ser erros e viram entradas para política.

![A conciliação caminha da reference key de um pedido pela busca de signature e pelo verificador até uma comparação em unidades base que sai como paid, underpaid, overpaid ou unmatched.](assets/v02-flowchart.webp)

### O trilho do memo, e o dinheiro sem história

A reference key é a porta de entrada, mas o briefing deste artefato diz reference ou memo, e a metade do memo não é redundância. O seu checkout vem carimbando as duas em todo pagamento desde a lição de transaction request: uma reference key nova como a conta pesquisável, e o id do pedido dentro de um spl-memo como intenção legível por humanos. O verificador já parseia esse memo; é uma das cinco checagens dele. Para o tráfego do seu próprio checkout a reference sozinha basta. O caminho do memo existe para todo o resto, e todo o resto está vindo. Um pagamento x402 no módulo 7 vai chegar carregando o id de fatura dele no `extra.memo`, sem nenhuma reference do Solana Pay por perto, e um comprador que paga uma fatura compartilhada a partir de um saque de corretora pode estropiar o fluxo da reference inteirinho e ainda assim digitar o seu id de pedido num campo de memo. Um conciliador que consegue casar por qualquer um dos dois campos é o que faz dele um conciliador em vez de dois.

O que levanta a pergunta na direção contrária que todo lojista encontra dentro de um mês: e uma transferência que chega na ATA da sua tesouraria e não casa com nada? Nenhuma reference conhecida, nenhum memo parseável, só dinheiro. Você não vai ficar sabendo dela por um webhook que você não estava esperando, então a resposta honesta é uma varredura: percorra o histórico de signatures da sua própria ATA e confira toda transferência de entrada contra o livro-razão. A mesma chamada `getSignaturesForAddress` faz o serviço apontada para a própria conta de tesouraria, com duas mecânicas que vale conhecer. O RPC limita cada página a 1,000 signatures, e você pagina para trás passando a signature mais velha que recebeu como cursor `before` na chamada seguinte. E você não caminha até o começo dos tempos: persista a signature mais nova que cada varredura completa, e a varredura seguinte para quando chegar nela.

```ts
// backoffice-refunds/src/sweep.ts (core walk)
export async function sweepTreasury(treasuryAta: Address, stopAt?: Signature) {
  let before: Signature | undefined;
  while (true) {
    const page = await rpc
      .getSignaturesForAddress(treasuryAta, { limit: 1000, before })
      .send();
    if (page.length === 0) return;
    for (const entry of page) {
      if (entry.signature === stopAt) return; // reached last sweep's frontier
      if (entry.err !== null || isProcessed(entry.signature)) continue;
      const matched = await tryMatchByReferenceOrMemo(entry.signature);
      if (!matched) recordOrphan(entry.signature); // money with no story
    }
    before = page[page.length - 1].signature;
  }
}
```

Uma linha órfã não é um problema para resolver automaticamente; é um problema para trazer à tona. O padrão tentador, empurrar de volta direto para onde veio, é um auto-reembolso sem guarda, uma armadilha que a seção de política abaixo disseca, e queimaria taxas devolvendo poeira para bots. Órfãs vão para uma fila de revisão, e se alguma um dia for devolvida, ela passa pelo mesmo caminho de reembolso protegido que tudo o mais, checagem de finality e link de origem incluídos. O presente de verdade da varredura é uma propriedade, não uma feature: rode ela depois de qualquer queda e o livro-razão converge para o que a blockchain diz, porque a blockchain, e não a sua caixa de entrada de webhooks, sempre foi o relatório de liquidação.

### O reembolso que você tem que inventar

Agora a prensagem empenada. Nos trilhos de cartão com que você cresceu integrando, este momento é bem pavimentado. A Stripe tem uma API de reembolsos, o emissor tem um processo de chargeback por trás dela, e uma máquina de disputas inteira está por trás disso. A máquina existe porque pagamentos de cartão são pagamentos pull: o lojista enfiou a mão na conta do comprador, então o sistema entrega uma alavanca para enfiar a mão de volta. Pagamentos push invertem a geometria. O comprador te entregou tokens numa transferência final e atômica; não existe alavanca, não existe contraparte que consiga enfiar a mão na sua conta, e por isso ninguém escreveu uma página de reembolsos, porque não existe primitivo de reembolso para documentar. O pitch da Shopify que a lição de abertura deste módulo desempacotou — chargebacks eliminados por construção — estava vendendo exatamente este momento. Verdade! E é precisamente por isso que o que vem a seguir é construção original da nossa parte, montada com peças que você já tem, não algo que você possa consultar.

Então o que é um reembolso, estruturalmente? É um pagamento. Essa é a sacada inteira. Você tem a transação de origem, que nomeia a carteira do comprador como a fonte. Você tem o transfer-kit, que empurra stablecoins para qualquer endereço. Um reembolso é um pagamento push reverso: mesmo mint, valor menor ou igual ao que ele de fato pagou, destino lido da transação de origem, mais uma coisa que os trilhos de pagamento não vão te dar de graça, um link de auditoria. A linha do reembolso no livro-razão registra a signature de origem que ele reverte, e o memo on-chain dele carrega essa signature também, então qualquer um auditando qualquer um dos lados do livro-razão consegue caminhar da venda ao reembolso e de volta sem confiar no seu banco de dados.

Isso não é a gente improvisando no vácuo. A Stripe atravessou essa ponte primeiro para o fluxo de pagar-com-cripto dela, e o comportamento documentado dela é o precedente que a gente espelha: reembolsos são devolvidos como stablecoins para a carteira de origem. Não para um endereço que o comprador te manda por e-mail, não para quem pedir com jeitinho num ticket de suporte convincente. A carteira de origem, lida da blockchain. Essa regra sozinha apaga uma classe inteira de fraude em que "o comprador" que pede o reembolso não é a carteira que pagou. Vale reparar com que cuidado os grandes delimitaram este território: o mesmo trilho da Stripe limita os clientes a 10,000 USD por transação. Quando a empresa de pagamentos mais experiente do planeta põe guarda-corpos tão apertados em volta de dinheiro irreversível, aceite a dica sobre quanto respeito a direção contrária merece.

![Comparação entre trilhos de cartão e trilhos de push: cartões entregam uma máquina de chargeback documentada, enquanto trilhos de push não entregam primitivo de reversão nenhum, então o lojista constrói o reembolso como um pagamento novo.](assets/v03-comparison.webp)

A assimetria corta dos dois lados, e é aqui que ela morde você, e não o comprador. Nenhum chargeback protege o lojista também. No primeiro reembolso que eu enfileirei nestes trilhos, eu conferi o destino três vezes como se estivesse desarmando alguma coisa. Bom instinto, alvo errado: o endereço estava certo, o problema era que o pagamento de origem tinha segundos de vida. Pense no que isso quer dizer. Um pagamento no commitment `confirmed` pode, raramente, estar sentado num fork que acaba descartado. Se você reembolsa ele e o fork morre, o "pagamento" evapora enquanto o seu reembolso, uma transação totalmente independente, aterrissa e finaliza mesmo assim. Você acabou de pagar dinheiro de verdade para reverter um pagamento que nunca aconteceu, e não existe alavanca para enfiar a mão de volta, porque você construiu no trilho que não tem uma. A guarda é uma chamada de RPC: a signature de origem tem que reportar `finalized`, o commitment que a rede não vai desfazer, antes de o builder de reembolso assinar qualquer coisa. A finalização custa uns dez segundos estimados pelo ecossistema, os mesmos ~10s que a lição de abertura deste módulo derivou no alvo de slot de 300ms. Um reembolso nunca é tão urgente que não possa esperar dez segundos; a lição de abertura do módulo 4 já fez exatamente este argumento por valor para o fulfillment, e o caso do reembolso é mais forte, porque agora o pagador é você.

![Um pedido de reembolso passa por checagens no livro-razão e por uma trava de finality na signature de origem antes de stablecoins serem empurradas para a carteira de origem; uma origem não finalizada é recusada.](assets/v04-flowchart.webp)

### Política, não padrões

Pagamentos exatos eram os 95 por cento fáceis. Os outros três estados escondem, cada um, uma decisão, e a disciplina em que esta lição insiste é que você tome cada decisão de propósito, escreva ela, e codifique ela, porque todo padrão silencioso é um de dois modos de falha vestindo um sobretudo.

Percorra o caso de brinquedo. Uma prensagem custa 30 USDC. Um comprador manda 12. O que deveria acontecer? Fazer o fulfillment em silêncio é a primeira armadilha: intenção não é pagamento, você acabou de vender um disco de 30 dólares por 12, e corre a voz de que a Wavelength entrega com pagamento parcial. Reembolsar automaticamente na hora é a segunda armadilha, e ela é mais sutil. Imagine um adversário que não quer os seus discos, só quer te machucar. Ele escreve um loop: paga a menos por uma fração, deixa o seu auto-reembolsador empurrar o dinheiro de volta, repete. Cada ciclo custa quase nada para ele e custa a você uma taxa de transação, uma verificação via RPC, a construção de um reembolso e uma escrita no livro-razão, para sempre, na velocidade em que ele conseguir submeter transferências. Um caminho de auto-reembolso sem guarda é um convite para outra pessoa gastar o seu dinheiro em taxas. O caminho de reembolso precisa de uma guarda: um rate limit, um limite mínimo, uma fila de revisão manual, alguma coisa que faça o loop custar mais para o atacante do que custa a você.

Entre as armadilhas fica o espaço das políticas defensáveis, e defensável quer dizer que o trade-off está declarado. Segure o pedido como underpaid e dê ao comprador uma janela para completar o pagamento: a mais gentil para erros honestos, custa a você um disco preso e a burocracia de expiração. Reembolso menos uma taxa de processamento: o encerramento mais limpo, custa a boa vontade do comprador e exige a taxa publicada de antemão, e além disso ainda passa pelo caminho de reembolso protegido. O pagamento a maior espelha isso: guarde o excedente como crédito registrado na loja (simples, mas agora você está carregando um passivo), ou reembolse o excedente depois da finality pelo mesmo caminho protegido com `amountBaseUnits` ajustado para o excedente, nunca para o pagamento inteiro.

Um **pagamento parcial** é pagar a menos com um nome mais respeitável, e ele merece o próprio caso trabalhado porque pedidos com vários itens são onde política vaga vai morrer. Digamos que um carrinho da Wavelength tenha a prensagem a 30 USDC, uma ecobag a 10 e um slipmat a 5, e cheguem 40 dos 45. Existem duas leituras defensáveis. Pedido-atômico diz que o pedido é uma coisa só, segure ele inteiro pelos 5 que faltam ou reembolse os 40, o que mantém o fulfillment binário ao custo de atrasar dois itens que o comprador pagou por inteiro. Preenchimento por item de linha diz mande a prensagem e a ecobag, e depois encaminhe o slipmat com 5 de diferença pela sua bifurcação de pagamento a menor, o que é mais amigável e te custa lógica de alocação: quais itens os 40 cobriram? Os de maior valor primeiro? Na ordem do comprador? O seu memorando de política tem que nomear a regra de alocação, porque "obviamente a prensagem" deixa de ser óbvio no dia em que dois itens empatarem. Qualquer uma das leituras é defensável. Decidir às 2 da manhã por ticket não é.

O tl;dr é: nenhuma dessas respostas é correta, e esse é o ponto. O que é correto é que o seu livro-razão consiga mostrar, para todo pagamento não exato, qual política declarada encaminhou ele e quando. Essa auditabilidade é o que uma disputa parece em trilhos sem máquina de disputas.

![Tabela dos três estados de pagamento não exato, a armadilha de padrão silencioso de cada um, e duas políticas defensáveis por estado com o custo que cada política carrega.](assets/v05-table.webp)

## Lab: construa o backoffice-refunds

Seis passos. O conciliador e o builder de reembolso vêm com apoio e TODOs marcados; o módulo de política é seu. Tudo roda contra a devnet com as mesmas versões de workspace que o curso congelou no módulo 2: `@solana/kit` ^6.10.0 com `@solana-program/token` 0.14.0 (fixados em 2026-08; o padrão de peers do ecossistema do kit já se moveu para a linha v7 desde então, e estes workspaces ficam na v6 porque o `@solana/pay` faz peer com o kit ^6.9; o módulo de assinaturas percorre essa costura direito). Nenhuma dependência nova hoje, o que é uma pequena lição por si só: um sistema de reembolso é uma composição, não um pacote.

**Passo 1: scaffold.** Crie o workspace e instale o ferramental dele:

```bash
mkdir -p backoffice-refunds/src
cd backoffice-refunds && npm init -y && npm pkg set type=module
npm pkg set scripts.build="tsc --noEmit"
npm pkg set scripts.verify:refunds="tsx verify-refunds.ts"
npm install -D tsx typescript @types/node
cd .. && npm install
```

Crie estes sete arquivos dentro de `src/` conforme for atravessando os passos abaixo: `reconcile.ts`, `refund.ts`, `policy.ts`, `origin.ts`, `sweep.ts`, `ledger.ts` (uma extensão fina do livro-razão do backoffice) e `reconcile-demo.ts`, o driver de checkpoint impresso no passo 2. O harness de verificação que a linha de script acabou de ligar, `verify-refunds.ts`, mora na raiz do pacote exatamente como o do verificador morava, e o passo 6 imprime ele inteiro. O build passa com os apoios no lugar porque todo TODO lança em vez de dar erro de tipo; o harness é o que vai te dizer que eles estão inacabados.

**Passo 2: o conciliador.** Uma emenda ao verificador vem primeiro, porque a classificação abaixo lê um campo que o contrato da lição 1 não carrega. O congelamento do `VerifyResult` era sobre o formato da chamada, `verify(signature, expectedOrder)`, e sobre nunca mudar um campo existente; acrescentar um não é nenhum dos dois. Em `verifier/src/types.ts`, o resultado ganha o delta observado:

```ts
// verifier/src/types.ts: the one amendment to the frozen contract, additive.
export type VerifyResult =
  | { ok: true; reason: 'verified'; signature: string; paidBaseUnits: bigint }
  | {
      ok: false;
      reason: RejectReason | 'not-found';
      signature: string;
      paidBaseUnits?: bigint;
    };
```

Obrigatório no ramo de aprovação, opcional no ramo de recusa, porque recusas a montante da checagem de delta (`duplicate`, `not-found`) nunca souberam o número. Em `verifier/src/verify.ts`, os dois returns a jusante do cálculo do delta carregam ele:

```ts
// verifier/src/verify.ts: both returns that know the delta now report it.
    if (credit.delta < expected.amountBaseUnits) {
      return { ok: false, reason: 'underpaid', signature, paidBaseUnits: credit.delta };
    }
    // ...memo check unchanged...
    deps.store.add(signature);
    return { ok: true, reason: 'verified', signature, paidBaseUnits: credit.delta };
```

Todo chamador anterior continua passando na checagem de tipos, porque nenhum deles lia um campo que não existia; rode `npm run --workspace verifier verify:verifier` de novo para provar que a emenda não quebrou nada. Agora abra `backoffice-refunds/src/reconcile.ts`. A caminhada é a da seção de teoria: a busca de signature e o filtro de transações que falharam são dados, e o TODO é a classificação, o passo em que o delta observado do verificador vira um dos quatro estados:

```ts
// backoffice-refunds/src/reconcile.ts
import { createSolanaRpc } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';
import { createVerifier, createRpcFetchTransaction, createMemoryStore } from 'verifier';
import { getOrderByReference, recordPayment } from './ledger';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// Reconciliation gets its OWN verifier with a fresh, empty store, on purpose.
// The ingestion path's processed-signatures set exists to stop double
// fulfillment; reconciliation's whole job is to re-derive truth from chain
// state, including for payments the ledger already knows. Share the ingestion
// store and every already-fulfilled payment would come back 'duplicate' and
// read as unmatched. And "fresh per run" must mean per CALL, not per import:
// the verifier is built inside reconcileOrder below, because a module-scope
// store in a long-lived process would mark a re-reconciled payment
// 'duplicate' -> unmatched, the exact failure this isolation exists to avoid.

export type ReconcileResult =
  | { status: 'paid'; signature: Signature }
  | { status: 'underpaid'; signature: Signature; paidBaseUnits: bigint }
  | { status: 'overpaid'; signature: Signature; paidBaseUnits: bigint }
  | { status: 'unmatched' };

export async function reconcileOrder(reference: Address): Promise<ReconcileResult> {
  const verify = createVerifier({
    fetchTransaction: createRpcFetchTransaction(),
    store: createMemoryStore(), // fresh per call; see the note above
  });
  const order = getOrderByReference(reference);
  if (!order) return { status: 'unmatched' };

  // Given: candidate signatures for the reference key, newest first.
  const candidates = await rpc
    .getSignaturesForAddress(reference, { limit: 10 })
    .send();

  for (const entry of candidates) {
    if (entry.err !== null) continue; // failed txs appear in this list too

    // The lesson-1 checks, unchanged: program, mint, delta, memo. Dedup runs
    // against this call's fresh store, so history stays re-inspectable.
    const check = await verify(entry.signature, order);
    if (!check.ok && check.reason !== 'underpaid') continue;

    // TODO: classify by the observed delta, in base units, never floats.
    // The verifier hands you check.paidBaseUnits (always present on the
    // 'verified' and 'underpaid' branches; the optionality covers rejections
    // upstream of the delta check). Record the payment against order.id, then
    // return 'paid' when it equals order.amountBaseUnits, 'underpaid' when it
    // is short, 'overpaid' when it exceeds. Bigints only: one float
    // comparison here misroutes every borderline amount.
    throw new Error('TODO: classify the observed delta');
  }
  return { status: 'unmatched' };
}
```

Duas notas de design, porque são as decisões interessantes. Primeira, o verificador continua sendo o juiz único; a emenda que você acabou de fazer entrega ao conciliador o `paidBaseUnits` observado para ele conseguir classificar em vez de só aceitar ou recusar. Uma recusa `underpaid` não é mais um beco sem saída, é dado. Uma ressalva para levar ao módulo 7: na ordenação do verificador a checagem de valor roda antes da checagem de memo, então um veredito `underpaid` não carrega nenhuma ligação com o pedido por si só. No caminho da reference isso é seguro, a busca de signature já amarrou toda candidata à reference deste pedido, mas o caminho do memo tem que reconferir o memo antes de classificar um pagamento curto, ou a transferência de um estranho poderia ser encaminhada para a sua fila de política. Segunda, repare que o conciliador nunca confia no caminho do webhook, em momento nenhum. Webhooks te disseram que alguma coisa provavelmente aconteceu; a conciliação é o processo em lote que reconstruiria a verdade só a partir do estado da blockchain se todo webhook fosse perdido. Lojistas que já fecharam o mês contra um relatório de liquidação de PSP já conhecem este formato: o mesmo trabalho, exceto que o seu relatório de liquidação é a blockchain, consultável a qualquer hora.

O caminho do memo e a varredura de tesouraria da seção de teoria moram em `sweep.ts`, ligados mas com o TODO do `tryMatchByReferenceOrMemo` em aberto: consulta pela reference primeiro, parse do memo como fallback, linha órfã quando os dois erram. Preencha ele depois que a caminhada principal funcionar. O harness de hoje só exercita o caminho da reference, e seja claro sobre quando a metade do memo de fato ganha o pão dela, porque ela não é o caminho feliz em lugar nenhum: as liquidações x402 do módulo 7 conciliam pelo próprio hook `onAfterSettle` delas direto no livro-razão, id de memo e tudo, sem chegar perto desta varredura. Para o que a varredura serve é o dinheiro que chegou enquanto nada estava escutando — um worker que caiu, um webhook desabilitado, um settle que aterrissou depois de o processo morrer — e o desafio do módulo 7 te faz provar isso exatamente nesse caso. Escreva ela para o crash, não para o tráfego.

O driver de checkpoint é o `src/reconcile-demo.ts`, totalmente trabalhado. Ele semeia a linha de pedido em aberto que o conciliador vai consultar (em produção, o seu servidor de checkout faz essa chamada de `recordOrder` no instante em que cunha a reference; aqui a demo faz as vezes dele), depois roda a caminhada e imprime o veredito. Ele lê as mesmas variáveis de ambiente que o harness ao vivo do módulo 4 te ensinou, então nada novo para lembrar:

```ts
// backoffice-refunds/src/reconcile-demo.ts
// Usage: npx tsx backoffice-refunds/src/reconcile-demo.ts <reference>
// Seeds an open order against the reference, then reconciles it.
import { address } from '@solana/kit';
import { recordOrder } from './ledger';
import { reconcileOrder } from './reconcile';

function req(name: string): string {
  const v = process.env[name];
  if (!v) throw new Error(`set ${name}, same variable as the module 4 live harness`);
  return v;
}

if (!process.argv[2]) throw new Error('usage: reconcile-demo.ts <reference>');
const reference = address(process.argv[2]);

const order = {
  orderId: process.env.ORDER_ID ?? 'ord-0231',
  recipient: req('MERCHANT'),
  recipientAta: req('MERCHANT_ATA'),
  mint: process.env.MINT ?? '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
  amountBaseUnits: BigInt(process.env.AMOUNT_BASE_UNITS ?? '30000000'),
};
recordOrder(order, reference);

const result = await reconcileOrder(reference);
if (result.status === 'unmatched') {
  console.log(`reconcile: order ${order.orderId} unmatched`);
  process.exit(1);
}
const paid =
  'paidBaseUnits' in result ? result.paidBaseUnits : order.amountBaseUnits;
console.log(
  `reconcile: order ${order.orderId} ${result.status}, signature ${result.signature}, ${paid} base units`,
);
```

Ele importa `recordOrder` de `ledger.ts`, que o passo 3 preenche; a metade de pedidos em aberto são duas operações de Map, então se você está indo estritamente em ordem, pule para a frente, ligue essas duas funções e volte.

Checkpoint antes de seguir. Pague um dos seus próprios pedidos de devnet a partir de uma segunda carteira — uma segunda carteira especificamente, porque o verificador lê a mudança de saldo na sua conta de token de lojista e um lojista pagando a si mesmo não muda nada. A CLI do transfer-kit faz as duas metades, a partir da raiz `wavelength` (se `/tmp/customer.json` não tiver mais SOL, `solana transfer $(solana-keygen pubkey /tmp/customer.json) 0.05 --allow-unfunded-recipient --url devnet` reabastece ela):

```bash
npm run --workspace transfer-kit pay -- $(solana-keygen pubkey /tmp/customer.json) 13
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5
```

A segunda rodada imprime a `reference` que ela anexou. Alimente a demo com ela, com o preço que você de fato pagou:

```bash
MERCHANT=$(solana address) MERCHANT_ATA=<your usdc ata> AMOUNT_BASE_UNITS=12500000 \
  npx tsx backoffice-refunds/src/reconcile-demo.ts <reference-from-your-order>
```

```
reconcile: order ord-0231 paid, signature 5Kd...w2, 12500000 base units
```

Valores exatos, casamento exato, status `paid`. Se você receber `unmatched` num pagamento que consegue ver no explorer, o seu verificador recusou ele; rode a demo com `DEBUG=verify` e leia qual das cinco checagens disse não. Esse loop de falha, o conciliador diz unmatched, o verificador diz por quê, é o ritmo de debugging para o resto do curso.

**Passo 3: o livro-razão ganha um segundo tipo de linha.** Abra `ledger.ts`. Ele cresce em duas direções, e a primeira é um armazenamento de que este módulo precisava em silêncio desde sempre: uma **tabela de pedidos em aberto**. `recordOrder(order, reference)` guarda um `ExpectedOrder` contra a reference key que o seu checkout cunhou, e `getOrderByReference` é a consulta dela; o script reconcile-demo semeia uma linha para o pagamento que você está prestes a fazer, e o seu servidor de checkout é onde a chamada pertence em produção. Sem ela, um pedido não pago ou pago a menor seria invisível para o conciliador, porque o livro-razão da lição passada só registrava pagamentos cumpridos. Segunda, a linha de pagamento da lição passada estava congelada em cinco campos, então diga a extensão em voz alta antes de usar ela: linhas de pagamento ganham `paidBaseUnits`, o delta que o verificador de fato observou on-chain, que é um número diferente do `amountBaseUnits` que o pedido esperava no instante em que alguém paga a menos, mais um `refundSignature` anulável que fica vazio até uma reversão nomear ele. Os dois são aditivos, então todo leitor de `rows()` da lição passada continua funcionando. Diga o escritor em voz alta, porque é a metade que todo mundo esquece e a omissão é silenciosa: **o `recordRefund` escreve duas vezes** — ele acrescenta a linha de reembolso abaixo, e ele carimba `refundSignature` na linha de pagamento que ele reverte. Pule a segunda escrita e a guarda de `already refunded` no builder de reembolso lê um campo que nada nunca ajusta, então ela nunca dispara, e toda re-execução da sua trava empurra mais um reembolso de verdade na devnet — exatamente o envio duplicado que a guarda existe para prevenir. Reembolsos acrescentam uma linha ligada própria:

```ts
// backoffice-refunds/src/ledger.ts (additions)
export type RefundRow = {
  originSignature: string;  // the payment this reverses: the audit link
  refundSignature: string;  // the refund's own on-chain signature
  refundReference: string;  // fresh reference key, so refunds reconcile too
  amountBaseUnits: string;  // bigint serialized as string on disk
  reason: string;
  at: string;               // ISO timestamp
};
```

A signature de origem é o design inteiro. Uma linha de reembolso que não consegue nomear o pagamento que ela reverte é dinheiro saindo sem história, não auditável por você e indistinguível de roubo para qualquer outra pessoa lendo os seus livros. Uma verruga prática que merece o parêntese: `bigint` não sobrevive ao `JSON.stringify`, então unidades base vivem como strings no disco e revivem nas bordas. Você encontrou exatamente esta verruga na lição do transfer-kit pela outra direção; a mesma regra, string exata para dentro, bigint exato para fora.

![A linha de reembolso do livro-razão guarda a signature do pagamento de origem e a própria, e o memo da transação de reembolso repete a origem, então um auditor consegue caminhar pelo link nos dois sentidos.](assets/v06-diagram.webp)

**Passo 4: o builder de reembolso.** Este é o arquivo que não existia em nenhuma documentação que você pudesse ter copiado. Abra `refund.ts`:

```ts
// backoffice-refunds/src/refund.ts
import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { createSolanaRpc, createKeyPairSignerFromBytes, generateKeyPairSigner } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';
import { sendStablecoin } from 'transfer-kit';
import { getPayment, recordRefund } from './ledger';

const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
const RPC_WS_URL = process.env.RPC_WS_URL ?? 'wss://api.devnet.solana.com';
const rpc = createSolanaRpc(RPC_URL);

// The merchant wallet that received the sale is the wallet that funds the
// reversal: the same keypair file the module 2 lab created.
async function loadMerchantKeyBytes(): Promise<Uint8Array> {
  const keyfile = process.env.MERCHANT_KEYFILE ?? `${homedir()}/.config/solana/id.json`;
  return new Uint8Array(JSON.parse(await readFile(keyfile, 'utf8')));
}
const merchant = await createKeyPairSignerFromBytes(await loadMerchantKeyBytes());

export async function refundPayment(
  originSignature: Signature,
  opts: { to: Address; mint: Address; amountBaseUnits: bigint; reason: string },
) {
  // Guard 1: our own ledger must know this payment, once.
  const payment = getPayment(originSignature);
  if (!payment) throw new Error(`no ledger row for ${originSignature}; refusing to refund`);
  if (payment.refundSignature) throw new Error(`already refunded in ${payment.refundSignature}`);
  if (opts.amountBaseUnits > BigInt(payment.paidBaseUnits))
    throw new Error('refund exceeds the amount actually paid');

  // Guard 2, given because it is the one rule this lesson will not let you
  // get wrong: the origin payment must be FINALIZED before we push.
  const { value } = await rpc
    .getSignatureStatuses([originSignature], { searchTransactionHistory: true })
    .send();
  if (value[0]?.confirmationStatus !== 'finalized') {
    throw new Error('origin payment not finalized; a push refund is irreversible, wait');
  }

  // The refund is a plain push payment, built by the kit that sent the sale,
  // called with the exact signature transfer-kit has had since the roster lesson: the
  // CALLER mints the reference key, so we give the refund its own claim ticket.
  const refundReference = (await generateKeyPairSigner()).address;
  const { signature } = await sendStablecoin({
    rpcUrl: RPC_URL,
    rpcSubscriptionsUrl: RPC_WS_URL,
    payer: merchant,
    mint: opts.mint,
    recipient: opts.to,
    amount: opts.amountBaseUnits, // already base units, never a float
    memo: `refund:${originSignature}:${opts.reason}`,
    reference: refundReference,
  });

  // Writes twice: the refund row, AND refundSignature back onto the payment
  // row. That back-stamp is what arms the `already refunded` guard above.
  recordRefund({
    originSignature,
    refundSignature: signature,
    refundReference,
    amountBaseUnits: opts.amountBaseUnits.toString(),
    reason: opts.reason,
    at: new Date().toISOString(),
  });

  return { signature, reference: refundReference };
}
```

Leia o formato dele. Guardas primeiro, em ordem de barateza: três checagens no livro-razão que não custam nada, depois uma chamada de status ao RPC, e só então a transação. O destino merece o próprio tempo, porque é por onde um ataque de engenharia social entraria. O `opts.to` nunca é digitado por um humano e nunca é lido de um ticket de suporte; o chamador extrai ele da própria transação de origem. O apoio já entrega o helper, e ele é curto o bastante para ler inteiro:

```ts
// backoffice-refunds/src/origin.ts
import { createSolanaRpc } from '@solana/kit';
import type { Address, Signature } from '@solana/kit';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

export async function getOriginatingWallet(
  originSignature: Signature,
  mint: Address,
): Promise<Address> {
  const tx = await rpc
    .getTransaction(originSignature, {
      encoding: 'jsonParsed',
      maxSupportedTransactionVersion: 1,
    })
    .send();
  if (!tx?.meta) throw new Error('origin transaction not found');

  // The account whose token balance for our mint went DOWN is the payer's ATA;
  // its owner field is the wallet the refund returns to.
  for (const post of tx.meta.postTokenBalances ?? []) {
    if (post.mint !== mint) continue;
    const pre = tx.meta.preTokenBalances?.find(b => b.accountIndex === post.accountIndex);
    const preAmount = BigInt(pre?.uiTokenAmount.amount ?? '0');
    const postAmount = BigInt(post.uiTokenAmount.amount);
    if (postAmount < preAmount && post.owner) return post.owner as Address;
  }
  throw new Error('no debited token account for this mint in the origin transaction');
}
```

Os mesmos saldos de token pre e post que o verificador lê para o delta do seu lado, percorridos pela outra ponta: a conta que foi debitada nomeia o dono dela, e esse dono é o destino do reembolso. Isto é a regra da carteira de origem da Stripe feita estrutural em vez de procedimental; não existe caminho de código em que um e-mail persuasivo mude para onde o dinheiro vai.

![O destino do reembolso é lido da própria transação de origem: a conta de token debitada para o mint do pagamento nomeia o dono dela, e esse dono é a carteira de origem.](assets/v07-diagram.webp)

De volta ao builder em si. A checagem de `already refunded` importa mais do que parece: pedidos de reembolso vão chegar de ferramental de suporte, e ferramental de suporte tenta de novo, então idempotência-por-signature-de-origem aqui é a mesma disciplina que idempotência-por-signature no handler de webhook. E o memo torna a reversão legível on-chain, não só no seu banco de dados. O reembolso carrega a própria reference key nova, cunhada aqui e entregue ao transfer-kit exatamente do jeito que a reference de toda venda vem sendo entregue desde o módulo 2, o que quer dizer que um reembolso é localizável por uma busca de signature exatamente como um pagamento é. Dinheiro saindo anda nos mesmos trilhos pesquisáveis que dinheiro entrando, de graça, porque você compôs em vez de escrever um segundo sistema. Essa composição é o retorno silencioso de toda a escada de artefatos até aqui.

**Passo 5: codifique uma política.** O `policy.ts` é solo, mas a superfície de tipos é fixa para o harness conseguir encaminhar através dela:

```ts
// backoffice-refunds/src/policy.ts
import type { ExpectedOrder } from 'verifier';
import type { ReconcileResult } from './reconcile';

export type UnderpayPolicy =
  | { kind: 'hold-for-topup'; windowMinutes: number }
  | { kind: 'refund-minus-fee'; feeBaseUnits: bigint };

export type OverpayPolicy =
  | { kind: 'credit-to-order' }
  | { kind: 'refund-surplus-after-finality' };

// Yours to choose, and to defend in writing.
export const policy: { underpay: UnderpayPolicy; overpay: OverpayPolicy } = {
  underpay: { kind: 'hold-for-topup', windowMinutes: 60 },
  overpay: { kind: 'credit-to-order' },
};

// The ledger event a routed non-exact payment produces. No fulfillment
// member exists here on purpose.
export interface PolicyEvent {
  orderId: string;
  policy: UnderpayPolicy['kind'] | OverpayPolicy['kind'];
  paidBaseUnits: bigint;
  at: string;
}

// Yours to write: take an underpaid reconcile result and the order it
// shorts, and return the PolicyEvent your stated policy dictates.
export function routeUnderpaid(
  result: Extract<ReconcileResult, { status: 'underpaid' }>,
  order: ExpectedOrder,
): PolicyEvent {
  throw new Error('TODO: route by your stated policy');
}
```

Repare no que o sistema de tipos se recusa a expressar: não existe membro `fulfill-anyway` e não existe membro de auto-reembolso instantâneo sem guarda. As duas armadilhas são irrepresentáveis, o que é a guarda mais barata que você vai entregar na vida. O `routeUnderpaid` recebe um resultado de conciliação underpaid e devolve o evento de livro-razão que a sua política dita; o harness checa apenas que um pedido underpaid produza um evento carimbado com política em vez de um fulfillment.

**Passo 6: a trava.** Salve como `backoffice-refunds/verify-refunds.ts`, o arquivo para o qual a linha de script do passo 1 já aponta. Leia a ordem das operações antes de rodar: a checagem de underpaid vem primeiro porque a conciliação dela registra a linha de pagamento que a metade de reembolso depois reverte, e as duas sondas que devem falhar cercam a única chamada que gasta dinheiro.

```ts
// backoffice-refunds/verify-refunds.ts
// The lesson's acceptance gate, wired to `npm run verify:refunds`.
// Env: REFERENCE (the reference of the devnet payment you made in step 2's
// checkpoint), plus MERCHANT / MERCHANT_ATA / MINT / AMOUNT_BASE_UNITS from
// the module 4 live harness. Optional: FRESH_SIGNATURE, see below.
import { address, signature } from '@solana/kit';
import { getPayment, recordOrder } from './src/ledger';
import { reconcileOrder } from './src/reconcile';
import { policy, routeUnderpaid } from './src/policy';
import { refundPayment } from './src/refund';
import { getOriginatingWallet } from './src/origin';

function fail(msg: string): never {
  console.error(`REFUNDS FAIL: ${msg}`);
  process.exit(1);
}

function req(name: string): string {
  const v = process.env[name];
  return v ?? fail(`set ${name}, same variables as the module 4 live harness`);
}

const reference = address(req('REFERENCE'));
const priceBaseUnits = BigInt(process.env.AMOUNT_BASE_UNITS ?? '30000000');

// 1. Underpaid routing. An order for double the price turns the real payment
// into a short one, which must land in policy, never in fulfillment.
const order = {
  orderId: 'ord-underpaid-check',
  recipient: req('MERCHANT'),
  recipientAta: req('MERCHANT_ATA'),
  mint: process.env.MINT ?? '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
  amountBaseUnits: priceBaseUnits * 2n,
};
recordOrder(order, reference);
const short = await reconcileOrder(reference);
if (short.status !== 'underpaid') {
  fail(`expected underpaid, got ${short.status}: is your classification comparing base units?`);
}
const event = routeUnderpaid(short, order);
if (event.policy !== policy.underpay.kind) {
  fail(`routed to ${event.policy}, but your stated policy is ${policy.underpay.kind}`);
}
console.log(`  underpaid order ${order.orderId}: routed to ${event.policy}`);

// 2. Must fail: an origin the ledger has never seen.
const mintAddress = address(order.mint);
try {
  await refundPayment(signature('1'.repeat(64)), {
    to: address(order.recipient),
    mint: mintAddress,
    amountBaseUnits: 1n,
    reason: 'must-never-send',
  });
  fail('a refund against an origin the ledger has never seen went through');
} catch (err) {
  if (!(err instanceof Error && err.message.includes('refusing to refund'))) throw err;
  console.log('  unknown origin: refused, as it must be');
}

// 3. The real reversal: refund the payment part 1 just classified.
const originSignature = short.signature;
const payment = getPayment(originSignature);
if (!payment) fail('classification never recorded the payment row; re-read the step 2 TODO');
try {
  const to = await getOriginatingWallet(originSignature, mintAddress);
  const refund = await refundPayment(originSignature, {
    to,
    mint: mintAddress,
    amountBaseUnits: BigInt(payment.paidBaseUnits),
    reason: 'verify-harness',
  });
  console.log(`  refund ${refund.signature}: recorded, linked to origin`);
} catch (err) {
  const msg = err instanceof Error ? err.message : String(err);
  if (msg.includes('already refunded')) {
    console.log('  refund: idempotency guard held on re-run; linkage already in the ledger');
  } else if (msg.includes('not finalized')) {
    fail('origin payment not finalized yet; wait a few seconds and re-run');
  } else {
    throw err;
  }
}

// 4. Must fail, env-gated: a refund requested before the origin finalized.
const freshSig = process.env.FRESH_SIGNATURE;
if (freshSig) {
  const fresh = signature(freshSig);
  const freshRow = getPayment(fresh);
  if (!freshRow) fail('FRESH_SIGNATURE has no ledger row: reconcile it first');
  try {
    await refundPayment(fresh, {
      to: await getOriginatingWallet(fresh, mintAddress),
      mint: mintAddress,
      amountBaseUnits: BigInt(freshRow.paidBaseUnits),
      reason: 'must-never-send',
    });
    fail('refund went through: either the finality guard is missing, or the payment finalized before the gate ran; use a fresher signature');
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (!msg.includes('not finalized')) throw err;
    console.log('  finality guard: refused the pre-finality refund');
  }
} else {
  console.log('  finality guard: SKIP (set FRESH_SIGNATURE to exercise it)');
}

console.log(
  'refunds: refund recorded against origin signature; underpaid order routed to policy',
);
```

Rode a partir do pacote:

```bash
npm run --workspace backoffice-refunds verify:refunds
```

Saída esperada, primeira rodada:

```
  underpaid order ord-underpaid-check: routed to hold-for-topup
  unknown origin: refused, as it must be
  refund <signature>: recorded, linked to origin
  finality guard: SKIP (set FRESH_SIGNATURE to exercise it)
refunds: refund recorded against origin signature; underpaid order routed to policy
```

As duas sondas que devem falhar são os dentes. A recusa de origem desconhecida roda toda vez: `signature('1'.repeat(64))` é a signature toda zerada, e se empurrar dinheiro contra ela fizer qualquer coisa além de lançar, a sua guarda de livro-razão é ornamental — volte para o passo 4. A recusa por finality precisa de um pagamento que ainda não finalizou, coisa que harness nenhum consegue conjurar sob demanda, então ela é travada por variável de ambiente: pague um pedido, rode o `reconcile-demo` nele na hora, exporte a signature como `FRESH_SIGNATURE`, e rode a trava de novo dentro da janela de finalização de ~10 segundos que a lição de abertura deste módulo derivou (31 blocos ou mais no alvo de 300ms, cerca de 9.3s, arredondado para cima porque a cauda é "ou mais"). Reembolsar ele tem que ser recusado. Repare também no que uma re-execução prova de graça: a parte 3 bate na guarda de `already refunded` e reporta isso como aprovação, porque idempotência sobreviver a uma segunda rodada é a propriedade, não um inconveniente.

## Challenge

O trabalho de completion está para trás se o harness passa: conciliação pela reference com classificação julgada pelo verificador, e o builder de reembolso protegido. O degrau solo tem duas partes, e a segunda é a que vai parecer estranha.

Primeiro, a metade operacional. Emita um reembolso de verdade na devnet: pague um dos seus próprios pedidos a partir de uma segunda carteira, espere a finality, reembolse ele pelo `refundPayment`, e então prove que o reembolso é localizável pela própria ficha de retirada: peça `getSignaturesForAddress(refundReference)` e busque a transação que ele devolve. Você deve ver exatamente uma signature, a do reembolso, carregando o memo `refund:<origin>:<reason>`, porque dinheiro saindo anda nos mesmos trilhos de reference pesquisável que dinheiro entrando. (Não empurre ele pelo `reconcileOrder`: essa função classifica créditos para a sua tesouraria contra um pedido em aberto, e um reembolso é um débito sem linha de pedido, então `unmatched` é a resposta correta ali, não um bug.) Depois quebre ele de propósito: peça um segundo reembolso da mesma origem e confirme que a guarda de idempotência lança.

Segundo, o memorando de política. Escolha a sua política de pagamento a menor e de pagamento parcial, codifique ela em `policy.ts`, e escreva ela no repo como um documento curto que um humano do suporte conseguisse aplicar: o que acontece a 12 de 30 USDC, o que acontece quando dois de três itens de linha estão cobertos, o que é dito ao comprador, e qual é o trade-off declarado, querendo dizer o que esta política te custa e por que você aceita esse custo. Depois encaminhe o pedido underpaid semeado pelo harness através dela. Se o seu memorando não consegue justificar a política para um comprador cético e um contador cético ao mesmo tempo, ele não está pronto. Não existe resposta de referência; a barra de aceitação é a ligação e o estar declarado, não concordar com a minha.

Aceite: um reembolso aparece no livro-razão amarrado à signature do pagamento original, carrega a origem no memo on-chain dele, e resolve a partir da própria reference key numa busca de signature; a tentativa de reembolso duplo lança; um pedido underpaid é encaminhado para a sua política declarada, não para fulfillment silencioso; o memorando de política existe e nomeia o trade-off dele.

![Linha do tempo de um pedido de 30 USDC pago a 12: o conciliador marca ele como underpaid, uma janela de 60 minutos para completar o pagamento se abre, e o pedido ou se completa ou é encaminhado para um reembolso protegido.](assets/v08-timeline.webp)

## Checkpoint: o back office, completo

Se o harness se recusa a ficar verde, os suspeitos de sempre, em ordem: o conciliador classificando com matemática em float em vez de unidades base (a comparação encaminha errado, em silêncio, os valores de fronteira; você conhece este bug do módulo 2), a guarda de finality checando `confirmed` em vez de `finalized` (o caso de pagamento fresco do harness pega exatamente isso), ou o memo do reembolso sem a signature de origem, de modo que a asserção de ligação falha mesmo com o dinheiro tendo se movido. As três são correções de cinco minutos uma vez nomeadas. Quando ele passa, pause no que você de fato construiu, porque é mais raro do que parece: um fluxo de reembolso para trilhos que não entregam nenhum, com uma trilha de papel mais forte do que a que os trilhos de cartão te dão. Toda reversão nomeia a causa dela on-chain. Pouquíssimos lojistas em produção nestes trilhos conseguem dizer isso hoje. Você consegue.

E com isso, o back office está completo: verificar, ingerir, conciliar, reembolsar. Dinheiro entrando está provado, dinheiro saindo está protegido e ligado, e todo pagamento não exato aterrissa numa política que você escolheu em voz alta. O que a loja ainda não consegue fazer é voltar no mês que vem. Vendas avulsas são o negócio inteiro até aqui, e a melhor ideia da Wavelength é um clube do disco do mês, o que quer dizer receita recorrente, o que em trilhos de push levanta uma pergunta genuinamente picante: como você cobra de alguém todo mês sem nunca tomar custódia dos fundos dele? Esse é o próximo módulo. Até lá.
