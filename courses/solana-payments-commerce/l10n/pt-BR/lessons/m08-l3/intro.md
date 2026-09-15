# O gate de produção: um checklist de entrada no ar que fala sério

## Resumo

Você já consegue receber pagamentos sem SOL do comprador (lição retrasada) e sem conectividade (lição passada). Checkout, verificador, back office, assinaturas, ramps, x402, gasless, a fila offline: cada peça da stack da Wavelength existe e passou no próprio smoke test. Esta lição checa se a coisa toda está mesmo apta a abrir.

Aqui vem a parte incômoda. "Funciona" e "está pronto" são afirmações diferentes, e só uma delas chegou a ser testada. Cada degrau foi verificado isolado, no caminho feliz, por você, num dia bom. Ninguém perguntou ainda o que acontece quando o worker do webhook morre numa sexta à noite, ou quando um pedido de atacado de $9,000 liquida no mesmo commitment level que um disco de $12.

Então hoje você constrói o último artefato antes do capstone: o `prod-gate`, uma auditoria com nota que roda contra tudo que é seu. É um checklist em que você pode reprovar de verdade. Falhas não viram sentimento; viram fix-tasks com o seu nome nelas, e cada uma fecha antes do módulo 9.

Faça isto agora, antes de qualquer teoria:

```bash
cd ~/wavelength && mkdir -p gate && npx tsx --version
```

Se o `tsx` não estiver lá, instale ele no repositório que você vem construindo o curso inteiro: `npm i -D tsx` (não precisa de pin; é um runner, não uma superfície de API). Aquela pasta `gate/` vazia é onde o veredito sobre os seus últimos sete módulos vai morar no fim do lab.

Como o trabalho está dividido hoje, dito sem rodeios: a teoria abaixo vem servida por inteiro, o lab te entrega o runner do gate e quatro linhas prontas e te faz escrever o resto sozinho, e o challenge é puro julgamento sem guia, porque decidir o que "pronto" quer dizer para a sua própria stack é a habilidade que esta lição de fato ensina. Este é o módulo 8. Os apoios acabaram.

## O gate

### Por que um checklist e não um sentimento

Em 1935 o US Army Air Corps avaliou o Model 299 da Boeing, o avião que virou o B-17. Era a aeronave mais sofisticada já construída até ali, e no voo de avaliação ela caiu na decolagem, matando o piloto de teste, porque a tripulação esqueceu de soltar uma trava de comando. A resposta do Exército não foi "contrate pilotos melhores". Foi o checklist do piloto: uma lista curta, chata e pontuada de coisas que precisam ser verdade antes de as rodas saírem do chão. O avião era complexo demais para competência sozinha; competência mais um checklist voou com ele por trinta anos.

A sua stack de comércio acabou de cruzar a mesma linha de complexidade. Oito subsistemas, três protocolos de pagamento, dois arranjos de fee payer, uma fila offline segurando transações assinadas que funcionam quase como instrumento ao portador. O script de verify de nenhuma lição sozinha enxerga o todo. O gate é a checagem da trava de comando da loja.

Dois desses oito ficam de fora da auditoria do gate de propósito, então vamos dizer isso antes que o diagrama sugira outra coisa. Os caminhos de dinheiro do ramp embed rodam na infraestrutura da Coinbase, atrás das chaves da Coinbase, então a única pergunta em formato de gate que ele tem, higiene do segredo da CDP, se dobra na linha de separação de chaves em vez de ganhar artefato próprio. E a porta x402/MPP liquida no mesmo livro-razão do backoffice que as linhas de backoffice já auditam; o único item em aberto dela, o memo por chamada que falta no caminho do gate, já está na lista de fix-tasks que você levou daquela lição. Seis artefatos sob auditoria, oito subsistemas na stack, e o capstone liga todos eles.

E o que está em jogo não é hipotético. A Solana processou mais de um trilhão de dólares em volume de stablecoin em 2025, o número com que a documentação de pagamentos do solana.com abre (buscado em 2026-08-23, a mesma afirmação que a lição de abertura datou). É nesse caldo que a sua lojinha de discos está se plugando. Dinheiro nessa escala não está nem aí para o seu demo ter funcionado; ele acha o webhook que você nunca monitorou e o commitment level em que você nunca pensou, e acha os dois na pior hora possível. A realidade dura é: um checklist de entrada no ar só vale a pena escrever se ele puder reprovar você, e o seu vai reprovar hoje, pelo menos uma vez, na primeira rodada. É esse o ponto.

![Diagrama de seis artefatos do curso alimentando o auditor prod-gate, que emite um relatório com nota que controla a entrada no capstone do módulo 9.](assets/v01-diagram.png)

As linhas do gate se agrupam em quatro famílias: verdade da liquidação, falha silenciosa, manuseio do dinheiro, e o que acontece quando quebra mesmo assim. Percorra elas na ordem; cada família termina com a pergunta que as linhas dela vão fazer.

### Verdade da liquidação: commitment escalado ao valor

Todo caminho de pagamento que você construiu termina no mesmo momento: alguma linha de código decide que o dinheiro chegou e libera a mercadoria. A primeira família do gate audita o que aquela linha de fato checa.

A política vem direto da orientação oficial, e ela escala o commitment ao valor. Confirmed é o commitment da maioria dos pagamentos: uma confirmação otimista, alcançada em bem menos de um segundo, que na prática quase nunca reverte. Finalized fica reservado para pagamentos de alto valor ou sensíveis a compliance: custa vários segundos a mais de latência, e em troca a transação fica fora do alcance de rollback. E processed é só de UI e pode ser descartado durante forks, o que quer dizer que ele nunca, em circunstância nenhuma, é um gate de liquidação. Mostre um spinner no processed se quiser. Despache um disco em cima dele e você pode estar despachando um disco para uma transação que não existe mais.

Concretamente, para a Wavelength: a venda de disco de $12 libera no confirmed, porque fazer todo comprador esperar por finality para se salvar de uma reversão que essencialmente nunca acontece é latência gasta em nada. O pedido de atacado de $9,000 libera no finalized, porque nesse tamanho os segundos a mais saem mais baratos que a conversa com o seu contador. Você escolhe o limiar; o gate só exige que um limiar exista e que o código imponha ele.

![Comparação entre processed, confirmed e finalized: processed é só de UI e descartável em forks, confirmed é o gate de menos de um segundo para a maioria dos pagamentos, finalized o gate mais lento para pedidos de alto valor.](assets/v02-comparison.png)

Uma sutileza que atravessa a stack inteira e que esta família também pega: o drain do fair-queue da lição passada liquida transações que foram assinadas horas antes. O comprador foi embora da feira faz tempo. Se alguma dessas vendas cruzar o seu limiar de alto valor, o drain tem que segurar elas no finalized antes de marcar o pedido como atendido, porque não tem comprador nenhum na sua frente para passar o cartão de novo. Escreva a linha de um jeito que te obrigue a dar grep em todo argumento de commitment no código, não só nos que você lembra de ter escrito.

### Falha silenciosa: os canos que morrem sem fazer barulho

Uma stack de pagamentos tem dois tipos de falha. As barulhentas lançam erro, te acordam no plantão e são corrigidas. As quietas simplesmente param, e você descobre por um cliente. A segunda família caça as quietas.

O webhook é o clássico. O seu back office atende pedidos a partir das entregas de webhook da Helius, e um webhook da Helius que falha por tempo suficiente é desativado automaticamente: falhas de entrega sustentadas e a plataforma para de mandar, de propósito, para se proteger de martelar um endpoint morto. Comportamento razoável para infraestrutura. Catastrófico para uma loja sem alerta nenhum, porque de fora nada parece errado. Os checkouts completam, o dinheiro chega on-chain, o livro-razão enche de pagamentos verificados. O fulfillment só para, quieto, e o primeiro sinal é um e-mail irritado.

Vou admitir que esta aqui é pessoal: uma vez eu descobri que uma fila de fulfillment estava fora do ar por uma DM de cliente, não por dashboard nenhum, e o buraco vinha crescendo fazia dois dias. Sair disso na base do reembolso é exatamente tão divertido quanto parece. A correção é um alerta na taxa de falha do webhook, testado disparando ele de propósito, mais um poll de fallback (você construiu o verificador por polling no módulo 4; ele é a sua rede de segurança aqui) para que um webhook desativado degrade para fulfillment lento em vez de nenhum.

![Fluxograma mostrando falhas do worker de webhook se acumulando até a Helius desativar o webhook automaticamente, depois do que os pagamentos continuam dando certo mas o fulfillment para em silêncio, com duas mitigações marcadas, um alerta de taxa de falha e polling de fallback.](assets/v03-flowchart.png)

A mesma família cobre retries e idempotência, porque uma tempestade de retries é a gêmea da falha silenciosa: tudo parece bem enquanto você atende em dobro. Antes de escrever a linha, deixe uma distinção clara, porque ela decide o que "seguro para retry" quer dizer em todo caminho que é seu. Retransmitir a mesma transação assinada é inofensivo: a signature é a chave de deduplicação, e a rede não vai processar bytes idênticos duas vezes enquanto o blockhash viver. Montar e assinar uma transação nova para a mesma compra é uma autorização nova, e nada on-chain sabe que é "a mesma" venda. Esse segundo caso é para o que as suas defesas existem, e você já construiu elas. O livro-razão de pedidos chaveia a idempotência na signature da transação, então um evento de webhook reentregue cai numa linha que já existe e não faz nada. A reference em cada checkout quer dizer que um comprador tentando de novo um pagamento travado não consegue pagar duas vezes por um pedido, porque a segunda transação carrega a mesma reference e o verificador casa ela com um registro já liquidado. E o classificador do drain do fair-queue roteia todo nonce gasto para `unsafe`, nunca reenviando ele — não porque a rede aceitaria os bytes velhos (um nonce gasto é rejeitado de forma determinística, e é assim desde a correção de 2022), mas porque um nonce avançado quer dizer que aquela venda pode já ter liquidado uma vez, e o único retry que sobra é o do tipo signature nova: exatamente a cobrança em dobro por autorização nova que toda esta família de defesas existe para impedir.

As linhas do gate para esta família não perguntam se você construiu isso. Elas te pedem para provar, agora, reexecutando um evento real gravado e colando a única linha de pedido resultante no campo evidence. Uma defesa que você nunca disparou é uma hipótese.

![Comparação de dois casos: retransmitir os mesmos bytes assinados é deduplicado pela signature e seguro, enquanto assinar uma segunda transação para aquela venda é uma autorização nova que as defesas precisam pegar.](assets/v04-comparison.png)

### Manuseio do dinheiro: chaves e o orçamento de taxas

Terceira família, duas preocupações: quem pode mover o dinheiro, e o que mover ele te custa.

Chaves primeiro, e a postura é separação de poderes. O signatário do Kora que patrocina os checkouts gasless é delimitado pela allowlist dele e pelas regras de validação; a linha do gate te pede para demonstrar a negação, reexecutando a request fora da allowlist que você montou duas lições atrás e mostrando a recusa. A nonce authority da fila da feira é uma chave só dela, não a sua chave de tesouraria, então comprometer a barraca compromete uma fila, não a loja. E a carteira recebedora do lojista deveria ser exatamente isso: uma recebedora, com a chave guardada longe de qualquer lugar quente, varrida na cadência que te deixa dormir. Seja honesto sobre onde isso te deixa: se você construiu a fila da feira exatamente como a lição dela saiu, você reprova nesta linha hoje, porque aquele lab rodou de propósito a chave do lojista como fee payer e nonce authority ao mesmo tempo para manter um notebook só testável, e disse isso. A correção é mecânica, já que a authority de uma conta de nonce pode ser qualquer chave: gere um keypair dedicado de nonce authority, entregue as contas do pool para ele com a instrução AuthorizeNonceAccount do system program (ou recrie o pool embaixo dele), mantenha essa chave no notebook da barraca, e mova o recebedor do lojista para uma carteira fria. Se uma chave vazada consegue simultaneamente drenar o paymaster, avançar os nonces e esvaziar o caixa, o gate te reprova, e deve mesmo.

Agora as taxas, onde contabilidade honesta importa mais que otimização. O seu piso de custo por venda é a taxa base, 5000 lamports por signature: irrelevante para a margem em qualquer venda precificada em dólares. O número que de fato aparece nos livros é patrocínio. Todo checkout gasless em que o comprador precisa de uma conta de token nova te custa o mínimo isento de aluguel de 165 bytes — o número que `getMinimumBalanceForRentExemption(165)` devolve no cluster para onde você está fazendo deploy, que a lição de gasless te mandou ler em vez de decorar — e o comprador pode recuperar ele depois fechando a conta; você orçou isso como gasto, não como empréstimo, na lição um deste módulo, e a linha do gate simplesmente checa se o orçamento existe e se tem um teto diário.

Faça a conta uma vez para que o teto da linha seja um número e não um dar de ombros — é o guardanapo de cem compradores da lição que abre o módulo, na escala da feira: 200 vendas custam 0.001 SOL de taxa base no dia inteiro (um tiquinho a mais quando as vendas patrocinadas abaixo contarem a segunda signature delas), enquanto 50 estreantes gasless precisando de contas de token USDC novas custam uns 0.074 SOL de rent patrocinado — 50 × os 1,488,440 lamports que `getMinimumBalanceForRentExemption(165)` devolveu na devnet em 2026-09-07, uma alíquota que desce em degraus conforme o cronograma, então rode a consulta para pegar o seu próprio número em vez de confiar neste — umas setenta vezes a rubrica de taxa, cada lamport do rent recuperável pelos compradores e nenhum deles por você. A rubrica de taxa nos seus livros é na verdade uma rubrica de patrocínio, e o teto diário na linha do gate é o que impede uma leva scriptada de falsos compradores de primeira viagem de transformar o seu orçamento de crescimento na fazenda de rent deles.

Aí tem a taxa que você adiciona de propósito. Sob congestionamento, uma transação carregando só a taxa base pode ficar parada na fila e cair; uma pequena taxa de prioridade compra um lugar na fila para o seu pagamento. Taxas de prioridade são precificadas por unidade de computação, então a receita tem dois botões: um limite de unidades de computação perto do que a transação de fato usa (uma transação pode pedir até 1.4M CU; a sua transferência de token mais o memo usa uma lasquinha disso, e cada CU que você reserva sem precisar multiplica o preço que você paga), e um preço por CU lido do mercado recente nas contas que você toca. Aqui está a receita inteira, a única caixa que este curso entrega:

```typescript
// gate/fee-recipe.ts
import { address, createSolanaRpc } from '@solana/kit';
import {
  getSetComputeUnitLimitInstruction,
  getSetComputeUnitPriceInstruction,
} from '@solana-program/compute-budget';

const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');

// The one-box recipe: two instructions, one RPC read, three constants.
const CU_LIMIT = 60_000; // generous ceiling for a token transfer + memo
const FLOOR_MICROLAMPORTS = 1_000; // never send a bare base-fee tx
const CAP_MICROLAMPORTS = 1_000_000; // never let a spike eat the margin

export async function paymentFeeInstructions(writableAccounts: string[]) {
  const recent = await rpc
    .getRecentPrioritizationFees(writableAccounts.map(address))
    .send();

  const fees = recent
    .map((entry) => Number(entry.prioritizationFee))
    .sort((a, b) => a - b);

  const p75 = fees.length > 0 ? fees[Math.floor(fees.length * 0.75)] : 0;
  const microLamports = Math.min(Math.max(p75, FLOOR_MICROLAMPORTS), CAP_MICROLAMPORTS);

  return [
    getSetComputeUnitLimitInstruction({ units: CU_LIMIT }),
    getSetComputeUnitPriceInstruction({ microLamports }),
  ];
}
```

Nota de instalação para o único pacote novo neste workspace: `npm i @solana-program/compute-budget@0.16.0` no checkout-txreq, o workspace cujo builder monta toda transação de pagamento em que estas instruções pegam carona — o mesmo 0.16.0 que o workspace de gasless fixou mais cedo neste módulo, um número de compute-budget para o repositório inteiro. O pin está fazendo trabalho de verdade. Este workspace roda kit ^6.10, e 0.16.0 é o último minor de compute-budget cujo peer range aceita um kit v6 (verificado contra o npm em 2026-08-31: 0.16.0 tem peer de kit ^6.4.0, 0.17.0 pulou para ^7 e o 0.18.0 atual para ^8, então pegar o latest aqui quebra a instalação). Regra de frescor como sempre: recheque o peer range no dia em que você construir.

![Versão anotada do código da receita de taxa rotulando o limite de CU, o piso, o teto e a leitura de mercado de taxas recentes, com a saída sendo duas instruções de compute-budget por transação de pagamento.](assets/v05-annotated-code.png)

Uma nota para a frente sobre aquele campo `microLamports`, porque ele é a parte desta receita que não sobrevive à mudança de formato. Uma taxa de prioridade v0 é um *preço*: micro-lamports por unidade de computação, cobrados contra as unidades que você de fato consome. O formato de transação v1 troca isso por um *total*: lamports absolutos para a transação inteira, carregados na config da própria mensagem em vez de numa instrução de ComputeBudget. Essas são dimensões diferentes, então qualquer série de taxas que tire média entre as duas sem converter não quer dizer nada — multiplique um preço v0 pelo limite de CU e divida por 1,000,000 para pôr ele nas unidades do v1. Duas coisas viajam junto com essa mudança. No v1 as instruções de compute-budget acima viram no-ops, aceitas e ignoradas mas ainda custando um slot de instrução e uns 150 CU. E o v1 não entrega limites de recurso padrão, então uma transação que omite eles entra e depois falha, onde o v0 teria caído de volta para 200,000 CU. Nada disso faz a receita desta página estar errada: ela monta transações v0 e está correta para elas. Ela te diz o que você vai ter que rederivar em vez de copiar quando migrar.

Agora a fronteira, nomeada em voz alta porque fingir profundidade aqui seria pior que a superficialidade. Esta receita é deliberadamente rasa. Estratégia de verdade para landing de transação é uma disciplina: estimativa de taxa por conta, curvas de retry, bundles da Jito, ciência de compute-budget, e o curso Client-Side Mastery é dono disso tudo de ponta a ponta, do jeito que este curso é dono de comércio. A troca que você está aceitando é exata: superprovisione a taxa e você queima margem em toda venda; subprovisione e as transações caem sob congestionamento, e ajustar essa curva direito é ciência de landing, não comércio. A receita de uma caixa só faz as transações de pagamento entrarem com confiabilidade e paga um pouquinho a mais. Para uma loja, esse é o lado certo da troca. Quando o seu volume fizer o pagamento a mais doer, aquele curso é para onde você vai; o mesmo repasse vale para indexação em escala, que a sua montagem de webhook mais polling um dia vai superar.

### Quando quebra mesmo assim: o playbook e os livros

Última família. Tudo acima reduz as chances de um incidente; nada reduz elas a zero. A diferença entre uma hora ruim e um trimestre ruim é se a resposta foi escrita enquanto você estava calmo.

Um playbook de incidente para uma loja deste tamanho cabe em uma página, e a linha do gate checa que quatro coisas existem nele. Um interruptor de congelamento: um comando ou flag que para de aceitar pagamentos novos enquanto ainda registra os que já estão a caminho on-chain, porque os piores incidentes são aqueles em que você continua vendendo. Uma escada de severidade: o que você faz quando o fulfillment atrasa (cair para polling), versus quando o paymaster está drenando (congelar patrocínio, manter o checkout normal), versus quando você suspeita de vazamento de chave (congelar tudo, rotacionar, varrer). Uma linha de comunicação: a frase que você posta para os compradores, pré-escrita, porque você não vai escrever uma frase calma durante o incidente. E um dono: de quem o telefone toca. Se a resposta para qualquer uma dessas mora hoje na sua cabeça, ela não existe.

![Árvore de decisão do playbook de incidente: fulfillment atrasado leva a um fallback de polling, um paymaster drenando a um congelamento de patrocínio, uma suspeita de vazamento de chave a rotação e comunicação com compradores, cada uma terminando em postmortem.](assets/v06-flowchart.png)

E então a linha que eu não consigo deixar verde para você, porque o ecossistema não consegue. Quando a sua loja estiver no ar, alguém uma hora vai ter que fechar os livros: casar toda liquidação on-chain com um pedido, um reembolso, um tick de assinatura, e entregar a um contador algo que ele reconheça. Vá procurar um SaaS nativo de Solana para contabilidade e relatórios de lojista que faça isso e, nesta data de build, 2026-08-31, você não vai achar um feito para isso. Isso é uma lacuna de mercado observada, declarada com a data dela porque mercados jovens se mexem: ferramental genérico de imposto sobre cripto existe, dashboards de exchange existem, o dashboard da Stripe cobre as vendas que liquidaram nos trilhos da própria Stripe e mais nada. O seu livro-razão on-chain de Solana Pay e x402 é seu para conciliar.

A postura honesta do gate não é nem desespero nem negação. Você já tem a matéria-prima: toda venda no seu livro-razão carrega uma chave de reference ou um id de fatura no memo, casado com uma signature de transação pelo verificador. Então a linha exige a resposta provisória que você de fato consegue entregar, uma exportação de conciliação: um comando que emite um CSV datado de liquidações juntadas a pedidos, a coisa que você entregaria a um contador hoje e a coisa que você vai dar diff contra o SaaS de verdade no mês em que alguém finalmente construir ele. Uma lacuna declarada com uma data e uma solução de contorno é um plano. Uma lacuna que você supôs que um fornecedor tinha coberto é a emergência do ano que vem. (Se você é o leitor que ficou esperando uma ideia de startup este curso inteiro: esta linha é uma.)

## Lab: rode o gate

O lab constrói o `prod-gate` e roda ele contra a sua stack. Você recebe o runner e quatro linhas prontas; as linhas restantes são suas para escrever a partir das quatro famílias acima. Reserve a maior parte do seu tempo para o passo 5, que é a auditoria de verdade.

1. Defina o formato da linha e as quatro primeiras linhas em `gate/rows.ts`. Essas quatro são a semente; repare que cada campo `evidence` exige um artefato que você consegue colar, nunca uma impressão:

```typescript
// gate/rows.ts
export type Rung =
  | 'checkout'
  | 'verifier'
  | 'backoffice'
  | 'club-billing'
  | 'gasless-checkout'
  | 'fair-queue'
  | 'stack-wide';

export interface GateRow {
  id: string;
  rung: Rung;
  question: string;
  evidence: string;
}

export const rows: GateRow[] = [
  {
    id: 'commit-policy',
    rung: 'stack-wide',
    question:
      'Does every settlement path gate on confirmed, escalate to finalized above your high-value threshold, and never treat processed as settlement?',
    evidence:
      'grep every commitment argument in the verifier and the fair-queue drain; list each occurrence with its value and the payment path it guards',
  },
  {
    id: 'webhook-monitoring',
    rung: 'backoffice',
    question:
      'Do you alert on the webhook failure rate before sustained failures can auto-disable the webhook?',
    evidence:
      'show the alert rule and the last test alert it fired; a dashboard nobody watches is a fail',
  },
  {
    id: 'idempotency',
    rung: 'backoffice',
    question:
      'Does a replayed webhook event, or a client retry of the same checkout, fulfill exactly once?',
    evidence:
      'replay one recorded event twice against the orders ledger and paste the resulting single order row',
  },
  {
    id: 'fee-recipe',
    rung: 'checkout',
    question:
      'Does every payment transaction carry the one-box priority-fee pair (CU limit near real usage, floored and capped CU price)?',
    evidence: 'decode one live checkout tx and show both compute-budget instructions',
  },
];
```

Checkpoint: `npx tsx -e "import('./gate/rows.ts').then((m) => console.log(m.rows.length))"` imprime `4`.

2. Escreva as linhas restantes você mesmo, a partir das quatro famílias. Mire em dez a quatorze no total. No mínimo o conjunto precisa cobrir: orçamento de patrocínio com teto diário no degrau de gasless (o rent da ATA como gasto orçado, dimensionado a partir de uma leitura de rent ao vivo em vez de uma constante lembrada), a reexecução da negação fora da allowlist para o signatário do Kora, separação de chaves entre paymaster, nonce authority e recebedor, a exclusão `unsafe` de nonce gasto no drain do fair-queue, o playbook de incidente de quatro partes, a exportação de conciliação, um alerta de falha de tick de cobrança para o club-billing, e as regras de hospedagem do blink — `Access-Control-Allow-Origin: *` em toda rota de action E no `actions.json`, com o `actions.json` servido a partir da raiz do domínio. Essa última linha existe porque o blink é a única superfície cuja falha é invisível do seu terminal: curl não impõe CORS e um mount em sub-caminho responde perfeitamente bem ao curl, então os dois defeitos testam limpos localmente e não renderizam nada numa carteira. A evidência já é regenerável — os dumps de header do checklist pronto-para-registro da lição do blink — e a montagem é exatamente quando isso quebra, porque montar vários apps num servidor só é o momento em que um arquivo montado na raiz para de estar na raiz. Mantenha todo `question` respondível com sim ou não, e todo campo `evidence` um artefato colável.

3. Escreva o runner. Ele se recusa a pular linhas, dá nota no que lê, e transforma toda falha em uma fix-task:

```typescript
// gate/run.ts
import { readFileSync, writeFileSync } from 'node:fs';
import { rows } from './rows.ts';

type Verdict = 'pass' | 'fail';

interface VerdictEntry {
  verdict: Verdict;
  note: string;
}

const verdicts: Record<string, VerdictEntry> = JSON.parse(
  readFileSync(new URL('./verdicts.json', import.meta.url), 'utf8'),
);

let failures = 0;
const reportLines: string[] = [];
const fixTasks: string[] = [];

for (const row of rows) {
  const entry = verdicts[row.id];
  if (!entry) {
    throw new Error(`no verdict recorded for row "${row.id}": the gate does not skip rows`);
  }
  reportLines.push(`${entry.verdict.toUpperCase().padEnd(4)}  ${row.id} (${row.rung}): ${entry.note}`);
  if (entry.verdict === 'fail') {
    failures += 1;
    fixTasks.push(`- [ ] ${row.id}: ${entry.note}`);
  }
}

const report = [
  `# prod-gate report, ${new Date().toISOString().slice(0, 10)}`,
  '',
  ...reportLines,
  '',
  failures === 0
    ? 'GATE: GREEN. Proceed to the capstone.'
    : `GATE: RED. ${failures} failing row(s). Every fix-task below closes before m09.`,
  '',
  '## Fix tasks',
  ...(fixTasks.length > 0 ? fixTasks : ['(none)']),
  '',
].join('\n');

writeFileSync(new URL('./report.md', import.meta.url), report);
console.log(report);
```

Repare no que o runner não é: ele não é verificação automatizada. Ele não consegue dar grep no seu verificador nem disparar o seu alerta. O arquivo de vereditos é você, sob juramento, com evidência anexada. O trabalho do runner é escrituração e recusa: nenhuma linha sem resposta, nenhuma falha sem uma fix-task.

Checkpoint, e prove a metade da recusa antes de confiar na metade da escrituração: ponha um `{}` vazio em `gate/verdicts.json` e rode `npx tsx gate/run.ts`. Ele precisa lançar `no verdict recorded for row "commit-policy"` e não escrever relatório nenhum. Um runner que renderiza um relatório parcial não é um gate.

4. Ligue a receita de taxa antes de auditar a linha dela. Instale `@solana-program/compute-budget@0.16.0` (justificativa do pin na teoria acima), coloque o `gate/fee-recipe.ts` no lugar, e preponha a saída de `paymentFeeInstructions(...)` ao build de transação do servidor de checkout, passando as contas graváveis que o pagamento toca (a ATA do lojista é a disputada num dia de venda movimentado). Concretamente, a edição mora em um lugar só: dê a `finalizeTransaction` um array `extraInstructions` opcional que ele coloca na frente das instruções de pagamento, e faça o servidor de checkout passar as duas instruções de compute-budget ali. Toda superfície que reusa o builder, o blink e o checkout patrocinado inclusive, herda o botão; repare que a estimativa de taxa do Kora agora enxerga a taxa de prioridade também, que é exatamente o caso que o teto de patrocínio daquela lição existe para absorver. Rode de novo o smoke test de checkout do módulo 3 para confirmar que nada quebrou.

5. Agora audite. Para cada linha, em ordem, execute de fato a ação de evidência: rode o grep, reexecute o evento, monte e reexecute a request de patrocínio fora da allowlist, decodifique a transação ao vivo, abra o playbook. Registre vereditos honestos em `gate/verdicts.json`:

```json
{
  "commit-policy": { "verdict": "pass", "note": "verifier gates on confirmed; drain escalates to finalized over 500 USDC; no processed anywhere" },
  "webhook-monitoring": { "verdict": "fail", "note": "no alert on webhook failure rate yet" },
  "idempotency": { "verdict": "pass", "note": "replayed event produced one order row (signature-keyed)" },
  "fee-recipe": { "verdict": "fail", "note": "checkout txs still ship with no compute-budget instructions" }
}
```

Aquela amostra é da minha própria primeira rodada contra um build de referência desta stack, e sim, ela ficou dois de quatro. Um número nela merece etiqueta de aviso: o limiar de escalada para finalized de 500 USDC é a escolha daquele build, não uma recomendação do curso; o challenge te faz derivar o seu a partir do que um pagamento caído de fato te custa. Se a sua primeira rodada vier toda verde, a explicação mais provável é avaliação generosa, não uma stack impecável; releia os campos evidence e seja mais malvado.

6. Rode:

```bash
npx tsx gate/run.ts
```

Checkpoint: você deve ver o relatório com nota no terminal e em `gate/report.md`, terminando em `GATE: GREEN` ou em `GATE: RED` com uma lista de fix-tasks. Um gate RED aqui é um lab aprovado. O lab testa a auditoria, não a stack.

7. Feche o loop. Trabalhe as fix-tasks (as duas acima são uma tarde: uma regra de alerta, uma edição de servidor que você já ligou no passo 4), reaudite só as linhas reprovadas, e rode de novo até ficar GREEN. Guarde todo `report.md` datado; o capstone abre lendo o seu mais recente.

![Comparação dos estados de veredito do prod-gate: pass guarda evidência, fail força uma fix-task nomeada, uma linha pulada faz o runner lançar erro, e só um gate todo verde abre o capstone.](assets/v07-comparison.png)

## Challenge

Sem walkthrough para esta. Três julgamentos, escritos em `gate/DECISIONS.md`:

1. Defina o seu limiar de alto valor: o valor de pagamento acima do qual a Wavelength escala de confirmed para finalized. Defenda o número em três frases usando a sua lista de preços real (discos, assinaturas, atacado) e o que uma reversão em cada faixa te custaria em dinheiro e em confiança.
2. Escreva uma linha de gate que esta lição nunca mencionou, tirada de um modo de falha específico do seu build da stack (toda implementação deriva; a sua tem um ponto fraco que a minha não tem). Linha completa: id, rung, pergunta de sim ou não, evidência colável.
3. Argumente contra o seu próprio gate: nomeie a falha com mais chance de te machucar que nenhuma linha consegue pegar, e diga por que um checklist não consegue segurar ela. Se não vier nada à cabeça, a fila de nonce durável e as palavras "instrumento ao portador" são um lugar por onde começar a pensar.

Aceitação: o seu `report.md` final está GREEN com toda fix-task fechada, o `DECISIONS.md` guarda os três julgamentos, e a linha que você mesmo escreveu aparece com nota no relatório.

## Antes de virar a placa

Poste o seu relatório com nota e a linha que você mesmo escreveu no canal do curso, e leia as linhas de duas outras pessoas antes de comparar vereditos; as linhas que outros builders inventam para os pontos fracos deles são a melhor auditoria grátis que a sua stack vai receber. Se o seu gate pegou alguma coisa sobre a qual esta lição nunca te avisou, isso é o sistema funcionando, e eu quero saber.

O gate está verde e as fix-tasks estão fechadas. Isso faz desta a última lição que trata a stack como peças. O próximo módulo é o capstone: ligar todo degrau da escada numa loja só rodando e ver uma jornada de comprador completa, navegar, pagar, atender, conciliar, rodar de ponta a ponta. Você fez a inspeção. Agora abra as portas.
