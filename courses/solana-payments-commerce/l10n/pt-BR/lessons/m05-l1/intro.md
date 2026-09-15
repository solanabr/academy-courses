# A primitiva de delegado: aprove uma vez, puxe no cronograma

O módulo 4 fechou com um back office que verifica pagamentos no servidor, ingere webhooks de forma idempotente, concilia por reference key e emite reembolsos como pagamentos de push. A Wavelength consegue receber dinheiro e prestar contas dele, de ponta a ponta. O que ela não consegue é receber dinheiro DE NOVO no mês que vem. Todo pagamento até aqui começou com o cliente fazendo alguma coisa: escanear um QR, aprovar uma transação, assinar. Um clube do disco do mês precisa do contrário: o cliente assina uma vez em janeiro e a loja é paga em fevereiro, março e abril enquanto ele dorme.

A Stripe resolve isso guardando um cartão e cobrando dele num cronograma. Na Solana não existe cartão em arquivo nem custódia: então como é que um lojista puxa 15 USDC de um cliente no mês que vem sem segurar as chaves nem o dinheiro dele? A resposta é uma aprovação de delegado, e ela vem com uma restrição brutal que molda todo o resto deste módulo.

Comece o workspace agora para a instalação rodar enquanto você lê. Este é um irmão do transfer-kit, dentro da mesma raiz de workspace npm que você usa desde o módulo 2:

```bash
mkdir club-crank && cd club-crank
npm init -y
npm pkg set type=module
npm i @solana/kit@6.10.0 @solana-program/token@0.14.0 @solana-program/memo@0.11.2
npm i -D tsx typescript
```

Depois suba para a raiz `wavelength` e acrescente a pasta nova à lista de workspaces que você estendeu no módulo 4, que é o que deixa os scripts abaixo fazerem `import { resolveAta } from 'transfer-kit'` pelo nome:

```bash
cd ~/wavelength
npm pkg set --json workspaces='["transfer-kit","verifier","backoffice","backoffice-refunds","club-crank"]'
npm pkg set type="module"
npm install
```

Os pins, com a nota de frescor deles: o workspace fica na linha kit v6 em que o curso inteiro roda, e o `@solana-program/token` 0.14.0 é o último release desse cliente que faz peer com o kit v6 (a faixa de peer dele é `^6.5.0`, checada no npm em 2026-08-22; o 0.15.0 pulou para kit `^7` e o 0.16.0, que é o que o `latest` resolve hoje, já pulou de novo para `^8`, então instalar o npm-latest quebra este workspace de cara). Mesma história para o cliente de memo: o 0.11.2 é o mais novo que faz peer com kit `^6.4.0`, e a linha 0.12+ exige kit v7 ou mais novo. A próxima lição faz dessa divisão v6/v7 um tópico por direito próprio, porque o cliente oficial de Subscriptions fica do outro lado dela; hoje ela é uma linha de disciplina de pin. O `tsx` roda TypeScript direto, como em todo lugar neste curso.

Enquanto o npm trabalha, aqui vai o mapa de uma frase: esta lição é a permissão de gasto crua do SPL Token, a menor assinatura não custodial possível, e o único limite duro dela é exatamente o que o programa oficial de Subscriptions existe para consertar na próxima lição.

## Resumo

- `ApproveChecked` define um delegado numa conta de token: um endereço autorizado a mover até um valor aprovado de um mint específico, com os decimais declarados para que uma suposição errada falhe alto. O dono mantém a propriedade e pode dar `Revoke` a qualquer momento, unilateralmente, com uma instrução.
- Cada conta de token tem exatamente UM slot de delegado ativo. Aprovar um delegado novo revoga automaticamente o anterior. Nenhum livro-razão por delegado, nenhum estado de pausa, nenhuma fila. Essa regra única é a restrição de design que o módulo inteiro orbita.
- O valor aprovado é um saldo corrente, não um teto que zera. Toda transferência assinada pelo delegado decrementa ele, e a transferência que o deixa em exatamente zero também limpa o slot de delegado: uma aprovação de 60 USDC cobre quatro pulls de 15 USDC, e o quinto não acha delegado nenhum na conta.
- Um "crank" de backend segurando o par de chaves do delegado assina `TransferChecked` para puxar fundos no cronograma. O assinante não assina nada depois da aprovação inicial. O crank paga a taxa base de 5000 lamports por pull; o custo do assinante por ciclo é zero signatures e zero taxas.
- O delegado só consegue mover até-o-valor-aprovado do mint aprovado daquela única conta. Essa fronteira é imposta pelo Token program on-chain, não pela boa vontade do crank. Esta é a resposta honesta para "vocês conseguem drenar a minha carteira?": não, e não porque a gente promete.
- Antes de todo pull, releia a conta. O delegado pode não ser mais você, o limite pode estar mais baixo do que o seu banco de dados pensa, e o estado on-chain é o único livro-razão que importa.

O contrato de apoio, dito em voz alta: o caminho de pull do crank chega como um apoio trabalhado, completo e executável. A guarda que decide se um pull é permitido chega como três TODOs, e preenchê-los é o degrau de completion; as respostas estão derivadas, às claras, na teoria abaixo. Detectar o despejo do delegado concorrente é o degrau de solo, seu e de mais ninguém.

## Uma permissão de gasto, não um cartão guardado

### O que o dono de fato assina

Quando um cliente da Stripe salva um cartão, ele entrega uma credencial. Quem segura ela decide quanto cobrar e com que frequência; os limites moram no banco de dados da Stripe e na lei de disputas. Quando um assinante da Wavelength entra no clube de discos, ele assina uma instrução:

```ts
getApproveCheckedInstruction({
  source: subscriberAta,   // THEIR token account, which they keep owning
  mint: USDC_DEVNET,       // the only token this permission touches
  delegate: CRANK,         // the address allowed to pull
  owner: subscriber,       // the owner signs; nobody else can grant this
  amount: toBaseUnits('60', DECIMALS), // the hard ceiling, in base units
  decimals: DECIMALS,      // stated so a decimals mismatch fails the ix
})
```

Leia isso como uma frase: "este endereço pode mover no máximo 60 USDC desta minha única conta". Não "pode gerenciar a minha carteira". Não "pode cobrar da minha conta". Pode mover, no máximo, aquele valor, daquele mint, daquela conta. O sufixo `Checked` é a mesma disciplina que você usa desde o módulo 2: a instrução carrega o mint e os decimais, então se alguma coisa discordar sobre o que uma unidade base significa, a transação falha em vez de mover a magnitude errada.

Depois que isso aterrissa, a conta de token do assinante carrega três fatos que ela não carregava antes: um endereço `delegate`, um `delegatedAmount`, e mais nada. Sem nome de plano, sem cadência de cobrança, sem metadados. O Token program guarda um número e um endereço, e tudo o que uma "assinatura" significa além disso é problema seu, off-chain. Segure esse pensamento; a fatura disso vence no fim da lição.

![O ApproveChecked escreve apenas um endereço de delegado e um delegatedAmount na própria conta de token do assinante; a propriedade e o saldo ficam intocados, e o poder do crank é delimitado por esses dois campos.](assets/v01-diagram.png)

A saída é ainda menor. O `Revoke` recebe a conta de origem e a signature do dono, limpa os dois campos, e não precisa da permissão de ninguém:

```ts
getRevokeInstruction({ source: subscriberAta, owner: subscriber })
```

Uma instrução, só a taxa base. Compare isso com cancelar uma matrícula de academia algum dia.

### Um slot, e a regra de despejo

Agora a restrição. Cada conta de token tem exatamente um slot de delegado ativo. Não um por lojista, não uma lista. Um. Quando o dono assina um `ApproveChecked` novo, o programa sobrescreve o slot: delegado novo, valor novo, e o delegado anterior sumiu. Não pausado, não enfileirado atrás do novo. Sumiu, em silêncio, sem nenhuma notificação para o lojista que acabou de perdê-lo.

Rode a fita para a frente. O seu assinante ama o clube de discos. Em março ele também vira assinante de, digamos, um drop de café que roda o mesmo design de delegado cru na mesma conta de USDC. No momento em que a carteira dele assina o `ApproveChecked` da cafeteria, a aprovação do seu crank para de existir. O seu pull de abril falha. Ninguém fez nada errado: o assinante consentiu com os dois lojistas, os dois lojistas escreveram código correto, e a primitiva simplesmente não consegue segurar duas permissões vivas numa conta de token.

![Estados da conta antes e depois mostrando que um assinante aprovando um segundo lojista sobrescreve o delegado e o limite restante do primeiro lojista sem nenhuma notificação.](assets/v02-comparison.png)

É por isso que a lição fica dizendo "a primitiva crua". Uma assinatura viva por (usuário, mint) é um teto de produto real, e nenhuma quantidade de código esperto de backend levanta ele, porque o teto está no próprio layout da conta. O que o código de backend CONSEGUE fazer é detectar o despejo com honestidade em vez de dar erro às cegas, e esse é o seu desafio de solo hoje. Levantar o teto exige um programa que ocupa o slot uma vez e multiplexa arranjos de cobrança reais atrás dele, que é precisamente a próxima lição.

### O limite só desce

A segunda coisa que a intuição treinada pela Stripe erra: o valor aprovado não é um limite mensal e não zera no primeiro dia do mês. É um tanque de combustível, enchido uma vez pela signature do dono, drenado por toda transferência de delegado, reabastecível só por outra signature do dono.

O clube de discos cobra 15 USDC por ciclo. O assinante aprovou 60. Então:

![Um limite de 60 USDC desce por 45, 30 e 15 ao longo de quatro pulls bem-sucedidos; o quarto esvazia ele e o Token program limpa o delegado na mesma instrução, então o quinto pull acha um slot vazio, e só uma aprovação nova assinada pelo dono restaura os dois.](assets/v03-chart.png)

Quatro pulls e o tanque está seco, e o tanque leva a torneira junto. O Token program decrementa o `delegatedAmount` dentro da transferência assinada pelo delegado, e quando essa subtração cai em exatamente zero ele devolve o `delegate` da conta para nenhum na mesma instrução, que é o `null` que a sua guarda lê no ciclo seguinte: esgotar uma aprovação também limpa ela. Então a quinta transação não falha por limite vazio; ela falha porque a conta não tem mais delegado, o que faz da signature do crank só a signature de um estranho qualquer, e o Token program diz isso com `OwnerMismatch`, custom program error `0x4`. `InsufficientFunds`, custom program error `0x1`, é o caso vizinho: um limite pequeno demais para este pull mas ainda não zero, digamos 10 restantes contra uma cobrança de 15 USDC, onde o slot ainda é seu. De um jeito ou de outro a conta continua segurando bastante USDC; a permissão de mover ele é o que foi gasto. Não tem nada que o crank possa fazer a respeito a não ser pedir para o assinante assinar de novo. Isso parece um inconveniente e na verdade é uma feature: o assinante pré-consentiu com um total delimitado, e a fronteira está fazendo o trabalho dela. Uma aprovação de 60 USDC é quatro meses de clube, uma cadência natural de reconsentimento. Você podia pedir 600 adiantado e puxar por anos; alguns produtos vão pedir, e os usuários deles que cancelaram vão descobrir um limite vivo que esqueceram. Onde você põe o teto é uma decisão de produto que a blockchain não vai tomar por você. A blockchain só impõe o número que o dono assinou.

Uma consequência que vale precificar: depois da aprovação inicial, o custo do assinante por ciclo é zero. Nenhuma signature, nenhuma taxa, nada para lembrar. O crank paga a taxa base de 5000 lamports por pull, que a qualquer preço plausível do SOL é um erro de arredondamento contra uma assinatura de 15 USDC. Compare isso com os 2 a 3 por cento que uma bandeira tira de toda renovação, e você vê por que este formato vale o trabalho.

### O crank: mesma transferência, signatário diferente

"Crank" é gíria da Solana que vale adotar: um processo meio sem permissão que gira a manivela no cronograma, fazendo trabalho que a blockchain não vai fazer sozinha. A Solana não tem cron nativo; nada on-chain dispara no primeiro dia do mês. Alguma coisa off-chain precisa acordar, decidir que um pull está vencido, e submeter ele. O nosso é um script de backend num agendador.

Aqui está a parte que deveria parecer quase anticlimática. O pull é um `TransferChecked`, exatamente a instrução que o transfer-kit constrói desde o módulo 2, com um campo diferente:

```ts
getTransferCheckedInstruction({
  source: subscriberAta,      // the subscriber's account, as always
  mint: USDC_DEVNET,
  destination: merchantAta,
  authority: crank,           // the DELEGATE signs, not the owner
  amount: toBaseUnits('15', DECIMALS),
  decimals: DECIMALS,
})
```

O campo `authority` sempre significou "quem tem o direito de mover estes fundos". Até hoje isso era o dono. O Token program checa: o signatário é o dono? Não. O signatário é o delegado da conta, e o valor está dentro do `delegatedAmount`? Sim: transfere, depois decrementa o limite, atomicamente, na mesma instrução. Não existe um passo separado de escrituração para esquecer. O decremento É o efeito colateral da transferência.

Porque é o mesmo formato de instrução, tudo o que os módulos 3 e 4 ensinaram continua funcionando sem mudança. O crank anexa uma reference key nova para o back office conseguir conciliar o pull no livro-razão de pedidos, e um memo para a cobrança se nomear on-chain. O seu ingestor de webhooks do módulo 4 vai ver este pull como qualquer outro pagamento. Receita recorrente cai direto no pipeline que você já construiu, que é o retorno de ter construído ele nesta ordem.

O trabalho de verdade do crank, então, não é a transferência, é o parágrafo antes dela: decidir se puxar ainda é legítimo. Vou confessar o erro para você poder pular ele: o primeiro crank que eu montei cacheou o estado da aprovação no cadastro, porque por que é que ele mudaria? Uma carteira de teste reaprovou um delegado diferente no meio do ciclo, o meu crank submeteu mesmo assim, e eu passei uma noite encarando um custom program error 0x4 num log de transação antes de cair a ficha. O estado da conta é o livro-razão. O seu banco de dados é um cache com opiniões. Então o crank relê a conta de token todo santo ciclo, antes de todo pull, e responde três perguntas:

![Três checagens pré-pull mapeiam para desfechos: um delegado faltando recusa como delegate-revoked, seja porque o dono revogou ou porque um pull que esgotou limpou o slot; um delegado estranho recusa do mesmo jeito; um limite pequeno demais recusa como insufficient-allowance; e só um tudo-certo prossegue.](assets/v04-table.png)

O crank poderia pular a guarda e simplesmente submeter, deixando a blockchain rejeitar os pulls ruins? Mecanicamente sim, e os fundos ficariam exatamente tão seguros: o Token program impõe tudo o que a guarda checa. A guarda existe porque "transaction failed: custom program error 0x4" e "este assinante nos revogou, marque a assinatura como vencida" são fatos diferentes para um sistema de cobrança, e só um deles diz ao seu back office o que fazer em seguida. A blockchain te dá um não. A guarda te dá o motivo, antes de você gastar uma taxa para descobrir. Essas strings de motivo, `delegate-revoked` e `insufficient-allowance`, são o vocabulário da primitiva crua, e as próximas duas lições mantêm os dois nomes significativos uma camada abaixo: a guarda do programa oficial acrescenta os motivos dela por cima, e a nota de continuidade no Challenge da próxima lição percorre o mapeamento explicitamente.

### O que o delegado nunca consegue fazer

Rode o pior cenário do assinante com honestidade, porque um cliente vai perguntar, e "confie na gente" é uma resposta da Stripe, não uma resposta da Solana.

Suponha que a Wavelength vire do mal, ou mais realisticamente, que o par de chaves do crank vaze. O que quem segura ele consegue fazer? Assinar `TransferChecked` contra a conta de USDC do assinante, até o limite restante. Se três pulls já aconteceram, isso é no máximo 15 USDC. O que ele consegue fazer com o SOL do assinante? Nada; o delegado está em uma conta de token. Os outros saldos SPL dele e os NFTs dele moram em contas completamente diferentes, cada uma com o próprio slot de delegado intocado. Ele consegue aprovar para si mesmo um limite maior? Não: `ApproveChecked` exige a signature do dono. Ele consegue impedir o assinante de revogar? Não: `Revoke` exige só o dono. O raio de impacto de um crank totalmente comprometido é o limite não gasto exatamente nas contas que aprovaram ele, e cada um desses donos consegue zerar isso unilateralmente no momento em que o comprometimento for anunciado.

![Uma chave de crank vazada alcança apenas o limite restante na única conta de USDC aprovada; SOL, outros tokens, NFTs, autoaprovação e bloqueio de revogação ficam todos fora dessa fronteira.](assets/v05-diagram.png)

Essa é a promessa não custodial, dita sem romance: não que o lojista seja honesto, mas que a honestidade dele não é estrutural. A fronteira mora no Token program, o mesmo caminho de código auditado que liquidou toda transferência SPL que este curso fez. Você não fez deploy de um programa hoje, e é esse o ponto: não existe contrato novo para um assinante auditar. A permissão que ele concede é imposta por código em que ele já confia pelo simples fato de segurar o token.

O ecossistema reparou neste formato. Quando a Superteam rodou o tema de bounty Solana Native "Subscriptions and Allowances" em junho de 2026, dezenas de repos de demo convergiram exatamente para esta primitiva de delegado crua, aprovar-e-depois-crank, como a resposta padrão para receita recorrente não custodial. Você não está aprendendo uma curiosidade; você está aprendendo o padrão em torno do qual o ecossistema está se padronizando, uma lição antes de conhecer o programa que põe ele em produção.

## Lab: cobrar do clube do disco do mês

O clube: 15 USDC de devnet por ciclo, aprovado em 60, então o livro-razão conta a história inteira em quatro pulls e uma recusa. Você vai jogar dos dois lados, assinante e lojista, com dois pares de chaves.

![O assinante assina uma aprovação, o crank assina e paga a taxa de todo pull, e o lojista só recebe 15 USDC por ciclo.](assets/v06-diagram.png)

1. **Pares de chaves e fundos.** No workspace `club-crank`, cunhe duas identidades. A instalação do topo da lição já deve ter terminado a esta altura.

   ```bash
   solana-keygen new --no-bip39-passphrase -o crank.json
   solana-keygen new --no-bip39-passphrase -o subscriber.json
   solana airdrop 1 $(solana-keygen pubkey crank.json) --url devnet
   solana airdrop 1 $(solana-keygen pubkey subscriber.json) --url devnet
   ```

   O assinante também precisa de USDC de devnet para ser cobrado, e isso vale um parágrafo de aritmética em vez de um número, porque USDC de devnet é o único recurso que este curso não consegue conjurar. Mande da sua carteira de lojista com o script `pay` do transfer-kit, exatamente o fluxo do lab do módulo 2 lição 1 (que também cria a ATA do assinante, e te cobra uma vez o mínimo isento de aluguel de 165 bytes — o número que o módulo 2 fez você buscar com curl em vez de decorar):

   ```bash
   npm run --workspace transfer-kit pay -- $(solana-keygen pubkey subscriber.json) 60
   ```

   Sessenta não é decoração: o plano desta lição aprova 60 USDC e puxa 15 por ciclo, então 60 é exatamente quatro ciclos, e o quinto pull achando um slot de delegado vazio é o desfecho da lição. A sua carteira de lojista quase certamente não segura 60 USDC de devnet agora. O faucet da Circle pinga uma cota pequena por visita e aplica rate limit em visitas repetidas, então chegar a 60 quer dizer várias visitas espalhadas pelo dia. Dois caminhos honestos por aqui, escolha um antes de continuar:

   - **Pingue e espere.** Visite faucet.circle.com sempre que o cooldown deixar, até `spl-token balance 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU --url devnet` passar de 60, depois mande adiante. Fidelidade máxima aos números impressos abaixo, mais lento.
   - **Reduza a escala do plano.** Divida toda cifra em USDC desta lição por dez: aprove `6` em vez de `60`, ponha `PLAN = '1.5'` em vez de `'15'`, abasteça o assinante com 6. A aritmética do limite é idêntica — quatro pulls esgotam ele, o quinto não acha delegado nenhum — e todo checkpoint abaixo conta a mesma história em um décimo da escala. Este é o caminho que eu tomaria numa carteira de devnet nova, e o único custo é que as linhas de log imprimem números menores que os meus.

   Se os airdrops de SOL aplicarem rate limit, espere um minuto e tente de novo; você precisa de SOL nos dois pares de chaves porque o assinante paga a taxa da aprovação e o crank paga a taxa de todo pull.

   Checkpoint: `solana balance $(solana-keygen pubkey subscriber.json) --url devnet` imprime cerca de 1 SOL, o mesmo para o crank, e o saldo de USDC do assinante lê a cifra que você escolheu acima. Nada depois no lab funciona sem os três.

2. **A guarda, como um apoio de completion.** Crie `crank/guard.ts`. Isto é lógica pura, sem rede, que é exatamente o que faz dela testável offline antes de qualquer dinheiro de devnet se mover:

   ```ts
   // crank/guard.ts: the decision the crank makes before every pull.

   export type PullDecision =
     | { ok: true; pullBase: bigint; remainingAfter: bigint }
     | { ok: false; reason: 'delegate-revoked' | 'insufficient-allowance' };

   export function checkPull(input: {
     /** delegate currently set on the subscriber's token account, or null if none */
     delegate: string | null;
     /** remaining approved amount on the account, in base units */
     delegatedAmount: bigint;
     /** the crank's own address */
     crank: string;
     /** this cycle's pull, in base units */
     pullBase: bigint;
   }): PullDecision {
     // TODO 1: if no delegate is set, or the delegate is not our crank,
     //         refuse with reason 'delegate-revoked'.
     // TODO 2: if pullBase exceeds delegatedAmount, refuse with
     //         reason 'insufficient-allowance'.
     // TODO 3: otherwise return ok with pullBase and the decremented
     //         remainingAfter the pull will leave on-chain.
     throw new Error('TODO: implement the guard');
   }
   ```

   Toda entrada é um `bigint` em unidades base porque a regra do módulo 2 não venceu: valores são inteiros exatos, floats nunca tocam em dinheiro. Os três TODOs são as três linhas da tabela de decisão acima. Checkpoint: nada para rodar ainda, e esse é o resultado esperado. O arquivo existe, exporta uma função, e essa função lança até você preencher ela no Challenge.

3. **O teste que te julga.** Crie `crank/guard.test.ts`. Ele roda a aritmética trabalhada de 60/15 e os dois casos de recusa, inteiramente offline:

   ```ts
   // crank/guard.test.ts: offline proof the guard behaves before devnet money moves.
   import { checkPull } from './guard';

   const CRANK = 'CrankAddr1111111111111111111111111111111111';
   const OTHER = 'OtherAddr1111111111111111111111111111111111';
   const PULL = 15_000_000n; // 15 USDC at 6 decimals

   let failures = 0;
   function expectCase(name: string, pass: boolean) {
     if (!pass) {
       failures += 1;
       console.error(`FAIL: ${name}`);
     }
   }

   // 1. Fresh 60-USDC approval: four pulls succeed, 60 -> 45 -> 30 -> 15 -> 0.
   let allowance = 60_000_000n;
   for (let cycle = 1; cycle <= 4; cycle += 1) {
     const d = checkPull({ delegate: CRANK, delegatedAmount: allowance, crank: CRANK, pullBase: PULL });
     expectCase(`cycle ${cycle} pulls`, d.ok);
     if (d.ok) allowance = d.remainingAfter;
   }
   expectCase('allowance exhausted after four pulls', allowance === 0n);

   // 2. Two histories, one account state. The fourth pull zeroed the allowance and the
   //    Token program cleared the delegate in the same instruction; an owner running
   //    Revoke empties the same slot by hand. Cycle five reads null either way and is
   //    refused BEFORE submission. The account cannot tell you which happened, so
   //    "cancelled" versus "tank empty, ask for a renewal" is your ledger's call, not
   //    the chain's.
   const emptySlot = checkPull({ delegate: null, delegatedAmount: allowance, crank: CRANK, pullBase: PULL });
   expectCase('empty delegate slot rejected', !emptySlot.ok && emptySlot.reason === 'delegate-revoked');

   // 3. A partial allowance never over-pulls: 10 remaining cannot cover 15. Non-zero
   //    means the delegate is still set, so this is the other refusal reason.
   const partial = checkPull({ delegate: CRANK, delegatedAmount: 10_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('over-pull on partial allowance rejected', !partial.ok && partial.reason === 'insufficient-allowance');

   // 4. Owner approved a competing delegate: the slot holds someone else, and their
   //    60 USDC is not yours to spend.
   const evicted = checkPull({ delegate: OTHER, delegatedAmount: 60_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('evicted crank rejected', !evicted.ok && evicted.reason === 'delegate-revoked');

   // 5. Both refusals are true at once: evicted, and what the new delegate holds would
   //    not have covered this cycle anyway. Identity is checked before amount, so the
   //    reason must be delegate-revoked. Your back office routes on these strings, so
   //    the order the guard tests them in is part of the contract.
   const superseded = checkPull({ delegate: OTHER, delegatedAmount: 5_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('eviction outranks a short allowance', !superseded.ok && superseded.reason === 'delegate-revoked');

   if (failures > 0) {
     console.error(`guard: ${failures} case(s) failed`);
     process.exit(1);
   }
   console.log('guard: all crank cases passed (over-pull and revoked-delegate rejected)');
   ```

   Rode: `npx tsx crank/guard.test.ts`. Checkpoint: ele morre em `TODO: implement the guard`. Correto. Essa falha é a costura entre o apoio e o seu trabalho de completion; você vai fechá-la no Challenge, e a trava de verificação da lição é este teste imprimindo a linha final dele.

4. **Encanamento compartilhado.** Crie `crank/send.ts`, o pipeline de envio do kit do qual você escreveu variantes desde o módulo 2, mais um carregador de par de chaves para os arquivos gerados pela CLI:

   ```ts
   // crank/send.ts: load a CLI keypair, send a list of instructions on devnet.
   import { readFileSync } from 'node:fs';
   import {
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
     type KeyPairSigner,
   } from '@solana/kit';

   export const rpc = createSolanaRpc('https://api.devnet.solana.com');
   const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

   export async function loadSigner(path: string): Promise<KeyPairSigner> {
     const bytes = new Uint8Array(JSON.parse(readFileSync(path, 'utf8')));
     return createKeyPairSignerFromBytes(bytes);
   }

   export async function sendIxs(
     feePayer: KeyPairSigner,
     ixs: Instruction[],
   ): Promise<string> {
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const tx = await pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions(ixs, m),
       (m) => signTransactionMessageWithSigners(m),
     );
     assertIsTransactionWithBlockhashLifetime(tx);
     await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
       commitment: 'confirmed',
     });
     return getSignatureFromTransaction(tx);
   }
   ```

   Checkpoint: `npx tsx crank/send.ts` não imprime nada e sai limpo. Este módulo só exporta; se ele lançar no import, alguma instalação deu errado, então recheque os pins do topo da lição antes de escrever outra linha.

5. **A aprovação: o assinante se cadastra.** Crie `crank/approve.ts`. Esta é a única coisa que o assinante roda na vida:

   ```ts
   // crank/approve.ts: the SUBSCRIBER runs this once. It is the whole sign-up flow.
   import { address } from '@solana/kit';
   import { getApproveCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta, toBaseUnits } from 'transfer-kit';
   import { loadSigner, sendIxs } from './send';

   const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const DECIMALS = 6;
   const CRANK = address(process.env.CRANK_ADDRESS ?? '');

   async function main() {
     const subscriber = await loadSigner(process.env.SUBSCRIBER_KEYPAIR ?? 'subscriber.json');
     // Third seed, the owning token program, required since the roster lesson.
     // Devnet USDC is a classic Token mint, so it is static here.
     const subscriberAta = await resolveAta(subscriber.address, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);

     const approveIx = getApproveCheckedInstruction({
       source: subscriberAta,
       mint: USDC_DEVNET,
       delegate: CRANK,
       owner: subscriber,
       amount: toBaseUnits('60', DECIMALS), // four months of the 15-USDC plan
       decimals: DECIMALS,
     });

     const signature = await sendIxs(subscriber, [approveIx]);
     console.log(`approved: delegate=${CRANK} allowance=60 USDC sig=${signature}`);
   }

   main().catch((e) => {
     console.error(e);
     process.exit(1);
   });
   ```

   Rode com o endereço do crank no ambiente:

   ```bash
   CRANK_ADDRESS=$(solana-keygen pubkey crank.json) npx tsx crank/approve.ts
   ```

   Checkpoint: uma linha `approved:` com uma signature. Procure a transação num explorer na devnet e leia o `ApproveChecked` parseado: source, delegate, 60 USDC. Essa é a pegada contratual inteira do assinante.

6. **O pull: o ciclo agendado do lojista.** Crie `crank/pull.ts`, o apoio trabalhado que esta lição te entrega inteiro. Ele lê, guarda, puxa e relê:

   ```ts
   // crank/pull.ts: the MERCHANT backend runs this once per billing cycle.
   import {
     AccountRole,
     address,
     generateKeyPairSigner,
     unwrapOption,
     type Instruction,
   } from '@solana/kit';
   import {
     fetchToken,
     getTransferCheckedInstruction,
     TOKEN_PROGRAM_ADDRESS,
   } from '@solana-program/token';
   import { getAddMemoInstruction } from '@solana-program/memo';
   import { fromBaseUnits, resolveAta, toBaseUnits } from 'transfer-kit';
   import { checkPull } from './guard';
   import { loadSigner, rpc, sendIxs } from './send';

   const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const DECIMALS = 6;
   const PLAN = '15'; // USDC per cycle
   const SUBSCRIBER = address(process.env.SUBSCRIBER_ADDRESS ?? '');
   const MERCHANT = address(process.env.MERCHANT_ADDRESS ?? '');

   async function main() {
     const crank = await loadSigner(process.env.CRANK_KEYPAIR ?? 'crank.json');
     const subscriberAta = await resolveAta(SUBSCRIBER, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);
     const merchantAta = await resolveAta(MERCHANT, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);

     // 1. Read the account. Never pull on a cached view of the delegate slot.
     const tokenAccount = await fetchToken(rpc, subscriberAta);
     const delegate = unwrapOption(tokenAccount.data.delegate);
     const delegatedAmount = tokenAccount.data.delegatedAmount;

     // 2. Guard. The chain would reject a bad pull anyway; the guard names WHY first.
     const decision = checkPull({
       delegate,
       delegatedAmount,
       crank: crank.address,
       pullBase: toBaseUnits(PLAN, DECIMALS),
     });
     if (!decision.ok) {
       console.log(`refused: ${decision.reason}`);
       process.exit(1);
     }

     // 3. Build the pull: TransferChecked signed by the CRANK, not the owner,
     //    with a fresh reference key and a memo, the same shape transfer-kit taught.
     const reference = (await generateKeyPairSigner()).address;
     const transferIx = getTransferCheckedInstruction({
       source: subscriberAta,
       mint: USDC_DEVNET,
       destination: merchantAta,
       authority: crank, // the delegate signs; the subscriber signs nothing today
       amount: decision.pullBase,
       decimals: DECIMALS,
     });
     const transferWithReference: Instruction = {
       ...transferIx,
       accounts: [...transferIx.accounts, { address: reference, role: AccountRole.READONLY }],
     };
     const memoIx = getAddMemoInstruction({ memo: 'WVL-CLUB cycle pull' });

     const signature = await sendIxs(crank, [transferWithReference, memoIx]);

     // 4. Re-read: the on-chain allowance is the ledger, our math is a preview.
     const after = await fetchToken(rpc, subscriberAta);
     console.log(`pulled ${PLAN} USDC sig=${signature} ref=${reference}`);
     console.log(
       `allowance remaining: ${fromBaseUnits(after.data.delegatedAmount, DECIMALS)} USDC (expected ${fromBaseUnits(decision.remainingAfter, DECIMALS)})`,
     );
   }

   main().catch((e) => {
     console.error(e);
     process.exit(1);
   });
   ```

   Três detalhes ali ganham as linhas deles. O `fetchToken` decodifica a conta crua em campos tipados, e `delegate` volta como um option que você faz unwrap para um endereço ou null: null e "outra pessoa" são histórias de recusa diferentes, e a sua guarda distingue as duas. A reference key é um endereço recém-gerado anexado à lista de contas da transferência como um meta readonly, o mesmo truque de conciliação que o seu checkout usa desde o módulo 3, para o back office do m04 conseguir achar este pull por reference como qualquer venda. E o crank é o fee payer: receita recorrente custa ao lojista 5000 lamports por ciclo em taxas base.

   A decisão interessante é o que NÃO está aqui: o `sendStablecoin` do transfer-kit está ausente, de propósito, porque aquela função assina como dona da conta de origem, e hoje o ponto inteiro é que o dono está dormindo. O crank pega emprestados os helpers do kit (`resolveAta`, `toBaseUnits`, `fromBaseUnits`) e reconstrói o envio com `authority: crank`. Quando a diferença de uma linha força uma função nova, aquela linha é a lição.

   Você ainda não consegue rodar `pull.ts` com sucesso; a guarda dele ainda lança. Essa ordem é deliberada. Vá preencher os TODOs.

![Cada ciclo do crank lê a conta de novo, recusa com delegate-revoked ou insufficient-allowance, ou deixa passar um TransferChecked assinado pelo delegado, depois relê para confirmar o limite decrementado.](assets/v07-flowchart.png)

## Challenge

**Worked.** Feito acima: a aprovação aterrissou, o apoio do crank existe, e `npx tsx crank/guard.test.ts` falha no TODO nomeado. Se ele falhar em qualquer outra coisa, um caminho de import ou um typo está mentindo para você; conserte isso primeiro.

**Completion.** Preencha os três TODOs da guarda a partir da tabela de decisão: recuse `delegate-revoked` quando o slot estiver vazio ou segurar um endereço estranho, recuse `insufficient-allowance` quando o pull exceder o valor restante, senão devolva `ok` com o `remainingAfter` decrementado. São umas doze linhas. Aceitação, em dois estágios. Offline primeiro: `npx tsx crank/guard.test.ts` imprime exatamente `guard: all crank cases passed (over-pull and revoked-delegate rejected)`. Depois o livro-razão da devnet:

```bash
export SUBSCRIBER_ADDRESS=$(solana-keygen pubkey subscriber.json)
export MERCHANT_ADDRESS="<your module-2 merchant wallet>"
npx tsx crank/pull.ts   # allowance remaining: 45.000000
npx tsx crank/pull.ts   # allowance remaining: 30.000000
```

Dois pulls, duas signatures, e o limite restante impresso descendo 45 depois 30 com o valor esperado batendo. Agora force a terceira recusa sem esperar o tanque secar: ponha `PLAN` temporariamente em `'45'` e rode de novo. A guarda tem que imprimir `refused: insufficient-allowance` e, criticamente, nenhuma transação aparece na devnet; você recusou antes da submissão, nenhuma taxa gasta. Ponha `PLAN` de volta em `'15'`.

**Solo.** Detecte o despejo. Rode um segundo `ApproveChecked` do assinante aprovando um endereço DIFERENTE como delegado (gere um par de chaves descartável para fazer o papel da cafeteria). A próxima rodada do `pull.ts` do seu crank não pode dar erro às cegas e não pode simplesmente dizer "revogado": estenda o caminho de pull para que um delegado que está posto-mas-estranho (despejo) e um delegado que está ausente (revogação pura) sejam registrados de forma distinta: mantenha o tipo de dois motivos da guarda intocado, e num pequeno arquivo JSON de estado ao lado dos scripts escreva `{ "state": "delegate-revoked", "cause": "evicted" }` contra `"cause": "revoked"`, depois pule a conta nos ciclos seguintes até aparecer uma aprovação nova para o crank. Aceitação: dois pulls consecutivos dão certo e decrementam; um terceiro pull acima do teto é rejeitado antes da submissão; reaprovar um delegado diferente vira o estado guardado do crank para `delegate-revoked`; e um `ApproveChecked` novo de volta para o crank faz os pulls voltarem. Essa trilha de recibos, duas signatures que decrementam, uma recusa fundamentada, um despejo detectado, é a trava de maestria desta lição. O livro-razão prova; nenhum quiz consegue.

Se um pull falhar com um custom program error `0x4` do Token program, essa é a grafia on-chain de "o dono não bate", que para uma transferência assinada pelo delegado quer dizer que o slot de delegado da conta não segura o seu crank: ou ele foi revogado, despejado, ou esvaziado até zero por um pull anterior, ou a sua guarda leu uma conta e a sua transferência mirou outra, quase sempre uma env var `SUBSCRIBER_ADDRESS` apontando para o par de chaves errado.

Uma nota para o loop de feedback: este é o módulo em que o curso começa a confiar apoios a você em vez de arquivos prontos, e os TODOs da guarda estão calibrados pela tabela de decisão acima. Se eles te tomaram mais de vinte minutos, ou se a detecção de despejo pareceu subespecificada, diga isso no feedback do curso; quanto apoio a próxima revisão te entrega é ajustado exatamente por esses relatos. As partes que brigaram com você são as partes que a próxima revisão afia.

Dê um passo atrás e olhe o que você entregou: um lojista que cobra de um cliente dormindo, não consegue exceder um teto assinado pelo cliente, perde a permissão dele no instante em que o cliente muda de ideia, e sabe dizer por que um pull recusou. Isso é receita recorrente de verdade sem custódia em lugar nenhum, e dois pulls que decrementam na devnet provam que a primitiva funciona. Agora a fatura de mais cedo vence. On-chain, esta assinatura inteira é um número e um endereço: sem nome de plano, sem cadência, sem expiração, sem reset por período, sem registro de por que o delegado existe. Cada um desses você reconstruiu sozinho, num arquivo JSON largado ao lado dos scripts, o que está ótimo para um clube e é pouco sério a mil assinantes. A primitiva crua é auditada por construção, porque ela É o Token program, mas não modela nada; ou você fica reconstruindo semântica de cobrança off-chain para sempre, ou você migra para um programa que carrega ela on-chain. E existe um problema mais difícil que metadados. O seu assinante consegue segurar exatamente um delegado vivo, então no momento em que ele vira assinante de um segundo lojista o seu crank é silenciosamente despejado, e você acabou de ver isso acontecer no desafio de solo. Uma primitiva que pune o seu cliente por gostar de dois produtos ainda não é um sistema de cobrança. Próxima lição: o programa oficial de Subscriptions, que ocupa aquele slot único uma vez e faz uma conta de token carregar muitos arranjos de cobrança.
