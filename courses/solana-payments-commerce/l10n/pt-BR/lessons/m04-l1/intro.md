# Não confie em frontend nenhum: política de confirmação e verificação no servidor

## Resumo

O módulo 3 te deixou com três superfícies de checkout sobre um único núcleo de pagamento. Só os caminhos movidos a QR — o watcher de reference do checkout e o poll de findReference da maquininha — chegam a perguntar para a blockchain antes de chamar uma venda de real, e eles só respondem por pagamentos que já estavam vigiando. O resto despacha na palavra de um frontend. Esta é a lição em que isso para, para os três de uma vez.

O que você leva de hoje:

- Você entrega o **verifier**: um `verify(signature, expectedOrder)` no servidor que busca a própria transação, checa o programa de token, o mint, o delta de saldo na sua própria conta e o memo do pedido, e depois guarda um conjunto de signatures processadas para que um webhook reentregue nunca consiga cumprir duas vezes. Ele é o harness de aceitação do curso; todo degrau posterior roda contra ele.
- O commitment de confirmação vira uma decisão de política, não um default de config: `confirmed` para o disco de US$ 6, `finalized` para a fatura de atacado de US$ 6,000, `processed` para nada que encoste em cumprimento.
- A única testemunha que você chama é o `getTransaction` com encoding `jsonParsed`: o frontend te dá alegações, o livro-razão te dá fatos.
- A checagem ingênua ("existe uma signature, logo está pago") cumpre um pagamento com o token errado. Você assiste ela fazer isso e depois tira o emprego dela, checagem por checagem.

O cenário que faz esta lição existir: o navegador de um comprador vira para um check verde e faz POST de `paid: true` no seu servidor, então você despacha o disco. A transação por trás daquele check verde moveu 1.5 USDT, não USDC, para uma conta que não é sua. Nunca existiu um pagamento para o seu pedido, e não existe trilho de chargeback para trazer nada de volta. O frontend mentiu. Quem é a sua fonte da verdade?

Antes de responder em código, olhe a verdade crua uma vez com os seus próprios olhos. Pegue uma signature liquidada da devnet nos logs do watcher do seu checkout e pergunte direto para o livro-razão:

```bash
curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '{
  "jsonrpc": "2.0", "id": 1, "method": "getTransaction",
  "params": ["<YOUR_SIGNATURE>", {"encoding": "jsonParsed", "commitment": "confirmed", "maxSupportedTransactionVersion": 1}]
}' | head -c 2000
```

Role esse JSON por um segundo. Em algum lugar dele estão os arrays `preTokenBalances` e `postTokenBalances`, e dentro deles um `mint`, um `owner`, um `programId` e um `amount` exato em unidades base. Essa resposta é toda a matéria-prima desta lição. Tudo o que a gente constrói é um jeito disciplinado de ler ela.

## O verifier, checagem por checagem

### O check verde é uma alegação

Vamos ser precisos sobre o que você já verifica, porque o módulo 3 não era ingênuo. O watcher do checkout de QR rodava `validateTransfer` contra a blockchain: valor, destinatário, mint, para uma transfer request que ele já estava vigiando. Aquilo era verificação no servidor de verdade, escopada a um fluxo só. O que ele nunca checou: o programa de token por trás do mint. O que ele nunca teve: uma opinião sobre transações que ele já não estivesse vigiando, que é exatamente o que um webhook vai te entregar na próxima lição. E os fluxos de transaction request e de blink não têm nada parecido; os smoke tests deles provam que os seus endpoints respondem, não que dinheiro chegou. O verifier de hoje substitui todos aqueles smoke checks. Uma função, todas as superfícies, chamada com nada além de uma signature e o pedido que você acha que ela paga.

A regra de design vale ser dita como regra, porque ela é o módulo inteiro: **o frontend é UI para o comprador, nunca testemunha para você.** Uma flag `paid: true`, um redirect de sucesso, uma signature colada num formulário, tudo isso é entrada controlada pelo cliente. A única coisa que um cliente não consegue forjar é o que o livro-razão diz que uma transação confirmed fez. Então o servidor pergunta para o livro-razão, toda vez, e não cumpre com base em mais nada.

![O navegador manda alegações falsificáveis como paid true e uma signature, enquanto o servidor busca fatos no livro-razão via getTransaction, e só o canal de fatos alimenta a decisão de cumprimento.](assets/v01-diagram.webp)

Vou confessar de onde esta lição vem. Anos atrás, num projeto web2, eu liguei o cumprimento ao redirect de sucesso de um provedor de pagamento porque o exemplo da documentação fazia assim. Um testador com o devtools aberto reapresentou aquele redirect e conseguiu um pedido de graça em menos de uma hora, e a correção era a API de verificação no servidor do próprio provedor, que eu deveria ter lido antes. O pessoal do Stripe conhece isso como a regra de que você cumpre a partir do webhook mais um PaymentIntent recuperado, nunca a partir da URL de retorno do cliente. Mesma regra aqui, com dentes mais afiados: nos trilhos de cartão o meu erro dava para recuperar com um ticket de suporte. Aqui, a coisa que você despacha contra um pagamento falso simplesmente foi embora.

### Commitment é um preço, não uma configuração

Você conheceu `processed`, `confirmed` e `finalized` no módulo 1, mapeados contra o seu vocabulário de cartão. A orientação qualitativa da documentação oficial não mudou de forma: `confirmed` para a maioria dos pagamentos, `finalized` para os de valor alto ou sensíveis a compliance, `processed` só para UI, porque um bloco processed ainda pode ser descartado num fork. O que muda hoje é quem consome essa tabela. No módulo 1 ela era um modelo mental. Nesta lição ela é um parâmetro que o seu verifier recebe, e escolher ele por pagamento é trabalho seu.

Os números primeiro, com a procedência declarada com cuidado, porque este é um lugar onde as pessoas citam errado. A documentação te dá a tabela qualitativa e para por aí. Os números de latência são estimativas do ecossistema, observadas na rede, não impressas em documentação oficial nenhuma: `confirmed` aterrissa em mais ou menos 1 a 2 segundos; `finalized` quer dizer mais ou menos 31 ou mais blocos confirmed construídos em cima, o que dá mais ou menos 10 segundos de relógio (31 blocos no alvo de 300ms dá uns 9.3s; tempos de slot medidos ficam um pouco acima do alvo, e os blocos da cauda são "ou mais", daí o 10 redondo; derive isso de novo toda vez que o tempo de slot mudar, porque os cortes em etapas da SIMD-0525 vivem mudando ele). Cite eles assim. Uma lição, um runbook ou um memorando de compliance que atribui esses números à documentação está citando algo que a documentação nunca disse.

Então qual você compra? Precifique como seguro, porque é literalmente isso que é. O prêmio é latência no seu checkout; a indenização é proteção contra um bloco confirmed ser descartado num fork, o que é raro, e quanto mais valor um pagamento carrega, mais esse evento raro importa. Condicionar uma venda de um disco só de US$ 6 a `finalized` é teatro: você cobra de todo cliente 10 segundos encarando um spinner para se segurar contra um risco que, em US$ 6, arredonda para zero. Condicionar uma fatura de atacado de US$ 6,000 a `confirmed` é o erro oposto: um fork-drop real, ainda que raro, agora te custa quatro dígitos sem caminho de reversão, e você economizou oito segundos num pagamento que ninguém estava esperando de pé num balcão. O commitment escala com o que um pagamento descartado te custa. Escreva essa política em números, por faixa de produto, e deixe o verifier impor ela.

![Uma tabela de política de quatro linhas casando valores de pagamento com commitment levels: confirmed para as vendas de seis e de duzentos dólares, finalized para uma fatura de seis mil dólares, processed nunca.](assets/v02-comparison.webp)

Então o comprador fica encarando o quê enquanto o seu servidor espera? É aqui que o `processed` vale o que custa, porque ele é um nível de UI e nada mais. Mostre "pagamento visto" no instante em que a transação aparece em `processed`, abaixo de um segundo, e vire para "pago" só quando o commitment do seu verifier fechar. O comprador recebe retorno instantâneo, o cumprimento recebe a garantia dele, e nenhum dos dois pega emprestado o trabalho do outro. E quando você cumprir errado apesar de tudo, lembre do que o módulo 1 estabeleceu: não existe processo de disputa para encaminhar o erro. Um reembolso nestes trilhos é um push novinho de você para o comprador, construção original, e construir ele direito é uma lição inteira mais adiante neste módulo. O trabalho do verifier é fazer dos reembolsos um caso de atendimento ao cliente em vez de um mecanismo de sobrevivência.

Por que essa decisão pesa mais aqui do que jamais pesou em cartões? Por causa da assimetria em torno da qual este curso não para de girar. Quando a Shopify anunciou suporte a Solana Pay (2023-08-23), o discurso era que aquilo "elimina taxas bancárias, chargebacks e prazos de retenção". Cada palavra disso é uma vitória do lojista, e a linha do chargeback é a interessante, porque um chargeback nunca foi só fraude contra você. Ele era o botão de desfazer embutido do trilho, precificado em toda taxa de cartão como prêmio de seguro. Nestes trilhos o prêmio some e a apólice também: sem adquirente, sem processo de disputa, sem caminho institucional de reversão. O que está em jogo não é pouco. As stablecoins moveram cerca de US$ 27.6 trilhões em volume de transferência em 2024, superando Visa e Mastercard somadas, o número datado do guia de stablecoins da Helius que você conheceu no módulo 1. Dinheiro nessa escala está migrando para trilhos onde erros de cumprimento são definitivos. O verifier que você constrói hoje não é uma passada de endurecimento opcional. Ele é a última linha de defesa inteira, e a leitura honesta do discurso da Shopify é que estão te pagando o antigo prêmio de seguro em troca de você construir o seu próprio seguro. Boa troca, se você de fato construir.

Uma bandeira de roadmap, rotulada como tal, antes de a gente deixar a latência para trás. A reescrita de consenso chamada Alpenglow (SIMD-0326) foi aprovada pela governança com mais ou menos 99% do stake participante, está mergeada atrás de um feature gate que não está ativo na mainnet, e está mirada para o fim de 2026 via Agave 4.3 (status conferido na escrita deste texto, 2026-08-22). Se você for ler, repare que o cabeçalho de status do documento SIMD ainda diz "Review"; cabeçalhos ficam atrás da realidade, e a votação passou. Quando ela ativar, a matemática de finality embaixo desta tabela de política inteira comprime e você ganha o direito de afrouxar o lado da latência da troca. Por que finality funciona, votos, lockouts e o que o Alpenglow muda por baixo, é território do curso Low-Level Solana. Para este curso ela continua sendo o que é aqui: uma bandeira rotulada sobre uma política que você deriva de novo quando a rede muda debaixo de você.

### getTransaction, a única testemunha que vale a pena chamar

Agora a ferramenta. O `getTransaction` recebe uma signature e devolve o que aquela transação de fato fez, e com `encoding: "jsonParsed"` ele faz a decodificação de bytes para você: mudanças de saldo de token chegam como entradas estruturadas e instruções de programas conhecidos chegam já parseadas. Duas propriedades fazem dele a testemunha certa. Primeira, ele só responde em `confirmed` ou `finalized`; você literalmente não consegue perguntar para ele sobre uma transação `processed`, o que quer dizer que a sua política de commitment se encaixa direto no fetch e o nível fraco demais fica irrepresentável. Segunda, tudo na resposta foi computado pelo validador que executou a transação, não por nada que o dispositivo do comprador encostou.

Por que não o mais leve `getSignatureStatuses`, que é efetivamente no que o watcher do seu módulo 3 se apoiava para acompanhar progresso? Porque um status responde "aterrissou, e em qual commitment", e nada mais. Ele não consegue te dizer o que se moveu, em qual token, sob qual programa, para a conta de quem. Status servem para barra de progresso. Cumprimento precisa do conteúdo da transação, e o `getTransaction` é a chamada que devolve ele.

A resposta é um objeto grande. O seu verifier lê exatamente três partes dela:

![Mapa anotado de uma resposta jsonParsed do getTransaction marcando os saldos de token pre e post, a instrução spl-memo parseada carregando o id do pedido e o campo de erro em meta.](assets/v03-annotated-code.webp)

Três hábitos para fixar enquanto a anatomia está na sua frente. Case `preTokenBalances` com `postTokenBalances` por `accountIndex`, e trate uma entrada pre ausente como zero: uma conta de token criada dentro desta mesma transação (a ATA de um comprador de primeira viagem, ou a conta nova em folha de um atacante) tem saldo post e nenhum saldo pre. Faça a subtração em `bigint` sobre as strings de `amount`; a lição dos decimais já te ensinou por que float e dinheiro nunca se encontram, e `uiAmount` é float. E leia o campo `owner`, não só o endereço da conta: as entradas de saldo te dizem quem é dono de cada conta de token tocada, que é como o verifier acha créditos para você sem manter uma lista de toda conta de token que você já teve.

### Da checagem ingênua ao verifier, um ataque de cada vez

Aqui está a checagem que vai para produção em mais bases de código do que qualquer um admite, e é onde o exemplo trabalhado começa:

```ts
// The naive check. Every line of this lesson exists because this is not enough.
async function naiveVerify(signature: string): Promise<boolean> {
  const tx = await fetchTransaction(signature);
  return tx !== null && tx.meta?.err == null;
}
```

Existe uma signature, a transação deu certo, despache o disco. Dê para ela o cenário da abertura: 1.5 USDT movido entre duas contas, nenhuma delas sua, memo em branco. O `naiveVerify` devolve `true`. Ele responde "aconteceu alguma transação?" quando a pergunta é "este pedido foi pago?". Essas perguntas só soam parecidas. A gente fecha o vão um ataque de cada vez, e a ordem das checagens faz parte do design: cada checagem presume que as anteriores já passaram, e cada falha devolve o primeiro motivo da cadeia, exatamente um, para que o seu log de ops se leia como um diagnóstico em vez de um dar de ombros.

**Ataque 1: o mesmo pagamento, duas vezes.** Não é maldade, é infraestrutura. O webhook da próxima lição vai reentregar eventos, porque entrega at-least-once é como todo sistema de webhook sobrevive ao seu servidor cair por um instante. Uma signature é determinística para a transação dela, então o evento reentregue carrega a mesma signature, e um verifier sem memória cumpre o mesmo pedido duas vezes. A correção é o conjunto de signatures processadas: primeira checagem na entrada, última escrita no sucesso. Cheque `store.has(signature)` antes de qualquer outra coisa e devolva `duplicate`; chame `store.add(signature)` só depois que toda outra checagem passar, para que uma transação rejeitada possa ser retentada mas uma cumprida fique queimada. Essa ordenação torna o cumprimento exactly-once a partir do mesmo conjunto que o torna seguro.

**Ataque 2: o valor certo no programa errado.** Este é a pedra angular, e o que o `validateTransfer` nunca cobriu. Existem dois programas de token na Solana: o Token clássico em `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` e o Token-2022 em `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`. Qualquer um pode criar um mint Token-2022, dar o nome que quiser, mintar um bilhão de unidades para si mesmo e transferir 30.000000 delas para uma conta de token que pertence a você. On-chain aquilo é uma transação perfeitamente válida cujo `postTokenBalances` mostra o seu endereço creditado com exatamente o valor que você cobra. Um verifier que checa valor e owner mas não `programId` deixa passar, e o seu disco de US$ 30 acabou de ser vendido por confete. A checagem é uma linha, `credit.programId` tem que ser igual ao id do programa Token clássico, e o motivo de ela ter que vir antes da checagem de mint é sutil o bastante para ser dito em voz alta: endereços de mint só querem dizer o que o programa deles diz que eles querem dizer. Comparar strings de mint antes de ter estabelecido qual programa as define é conferir o rótulo de uma garrafa que outra pessoa imprimiu.

![Um atacante credita trinta unidades de um mint Token-2022 sem valor numa conta pertencente ao lojista, passando nas checagens de owner e de valor mas falhando na checagem de id de programa.](assets/v04-diagram.webp)

**Ataque 3: um token de verdade que não é o seu token.** Mesmo formato, menos esforço: te pagar 30 USDT quando o preço era 30 USDC. Os dois moram sob o programa Token clássico, então a checagem do ataque 2 passa. Agora a checagem de mint conquista o lugar dela: o `mint` do crédito tem que ser igual ao mint em que você precifica. Na mainnet, USDC quer dizer exatamente `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` e nada mais; na devnet o seu mint esperado é o que a config do seu transfer-kit fixou desde o módulo 2. Nomes, símbolos e logos são metadados que qualquer um copia. O endereço é a identidade.

Uma objeção justa antes do próximo ataque: a gente acabou de declarar o Token-2022 o vilão? Não, e a distinção importa, porque você já conheceu uma stablecoin que mora lá. O PYUSD, que você leu ao vivo on-chain lá no módulo 2, é um mint Token-2022, com todas as oito extensões dele. Se a Wavelength um dia precificar um item em PYUSD, você vai aceitar um pagamento Token-2022 de propósito. A checagem nunca diz Token clássico bom, Token-2022 ruim. Ela diz que o programa do crédito tem que ser o programa sob o qual o seu mint esperado de fato mora, como par. No lab o programa esperado é uma constante, porque a loja precifica em USDC do Token clássico; no dia em que você adicionar um ativo Token-2022, o programa esperado vira um campo por pedido emparelhado com o mint dele, e a checagem em si sobrevive sem mudança. A única versão imperdoável é a que aceita um mint sem nunca perguntar qual programa o define.

**Ataque 4: o número veio do comprador.** O formulário do pedido dizia 30, o POST do cliente dizia 30, e a transação moveu 0.30. Se o seu verifier lê o valor de qualquer lugar que não seja a blockchain, ele lê uma alegação. A checagem de verdade computa o delta na sua própria conta: case pre e post por `accountIndex`, fique com as entradas que pertencem a você, pegue o crédito e exija `postAmount - preAmount >= expected.amountBaseUnits` em unidades base. Repare no formato da comparação: a transação que nunca te tocou é só o caso degenerado em que o delta é zero, e é por isso que o verifier trata "nenhum crédito para o lojista em lugar nenhum desta transação" como um pagamento a menor de zero em vez de um caso especial. A transação de USDT-para-um-estranho da abertura morre exatamente aqui, antes mesmo de você considerar o mint dela, porque nenhuma das entradas de saldo dela pertence a você.

**Ataque 5: um pagamento de verdade para outro pedido.** O mais sutil dos cinco. A transação é genuína, programa certo, mint certo, valor certo, paga para você. Ela é só o pagamento do pedido `ord-0999`, e o comprador está reapresentando a signature dela contra o pedido `ord-1024`. Checagens de valor não pegam isso quando dois discos custam o mesmo. O id de pedido que o seu builder de transaction request carimba no spl-memo é a amarração: o verifier acha a instrução `spl-memo` parseada e exige que o id de pedido esperado apareça nela como campo inteiro, devolvendo `wrong-reference` caso contrário. Campo inteiro, nunca substring, e este é o único lugar onde as pessoas erram. O memo é `wavelength:<orderId>:<description>`, então uma checagem com `.includes()` casa o seu id em qualquer lugar daquela string, descrição inclusa; e o `buildOrderTransaction` aceita um `input.orderId` fornecido por quem chama quando recebe um, então nem o formato do id é seu para garantir. As fixtures do challenge de hoje à noite põem as duas direções de contenção na sua frente: um pagamento com memo `ORD-42710` não pode cumprir `ORD-4271`, e um com memo `ORD-427` também não. Quebrar o memo nos separadores dele e comparar um token por igualdade mantém o formato do memo como assunto do builder sem nunca aceitar um prefixo.

Cinco ataques, cinco checagens, uma ordem. Duplicate, depois programa de token, depois mint, depois valor, depois reference, e só então guarde a signature e diga `verified`:

![Fluxograma do pipeline do verifier rodando as checagens de duplicate, fetch, programa de token, mint, delta de saldo e memo até o cumprimento, cada falha saindo com um único motivo ordenado.](assets/v05-flowchart.webp)

Um motivo naquele diagrama não é como os outros. `not-found` é transitório, não um veredito: no commitment `confirmed` a transação pode simplesmente ainda não estar visível quando um webhook rápido dispara, então quem chamou espera e tenta de novo em vez de rejeitar o pedido. Uma transação que aterrissou mas falhou não precisa desse cuidado, e nem de caso especial: `meta.err` não nulo quer dizer que nada se moveu, os deltas dela são zero, e a checagem de underpaid dá conta dela.

### O conjunto que cresce para sempre

O conjunto de signatures processadas tem um custo que a versão resumida desta lição esconderia, então não vamos esconder. Todo pagamento cumprido adiciona uma entrada, para sempre, e um conjunto que só cresce é estado ilimitado: tranquilo no volume de uma loja de discos, uma fatura de verdade no de um processador de pagamentos. A saída é que as entradas param de valer o que custam. O blockhash de uma transação não pode ter mais de 150 blocos para aterrissar, o que no tempo-alvo de slot atual de 300ms é uma janela de mais ou menos 45 segundos (derive ela do tempo de slot, e derive de novo quando o tempo de slot mudar: os cortes em etapas da SIMD-0525 já moveram esta janela duas vezes, de ~60 segundos nos antigos slots de 400ms para ~53 na etapa de 350ms e para os ~45 de hoje, e mais dois cortes estão represados no código, então uma afirmação hardcoded de "mais ou menos um minuto" já está duas eras atrasada). Passada essa janela, a mesma transação assinada nunca mais consegue aterrissar na blockchain, então o replay on-chain acabou fisicamente. O que sobra é reentrega de webhook vinda da sua própria infraestrutura, que tem o próprio horizonte limitado de retry. Daí a regra de descarte: guarde uma signature pelo tempo de vida do blockhash mais uma margem generosa cobrindo a janela máxima de reentrega do seu provedor de webhook, e depois solte ela. O store em memória do lab varre entradas com mais de dez minutos, um limite deliberadamente preguiçoso que ainda está mais de uma ordem de grandeza além da janela on-chain.

![Linha do tempo mostrando uma signature guardada protegendo contra replay on-chain por mais ou menos quarenta e cinco segundos e contra reentrega de webhook por minutos, e depois sendo descartada aos dez minutos assim que as duas janelas fecham.](assets/v06-timeline.webp)

Persistência é outro eixo, e vale uma frase honesta: um conjunto em memória esquece no restart, então produção move a mesma interface de dois métodos para o seu banco de pedidos, onde uma linha de pedido cumprido com uma coluna de signature é o conjunto. A interface que você constrói hoje transforma essa troca num argumento de construtor.

## Lab: construa o harness de aceitação

A divisão de trabalho, dita em voz alta: eu percorro o workspace, os tipos, o store, o adaptador de RPC e o conjunto de fixtures com você, tudo trabalhado do começo ao fim. As duas checagens no coração do verifier, programa-de-token-e-mint e o delta de saldo, são TODOs com apoio que você mesmo preenche, com a seção de teoria acima como sua referência (completion). A rodada ao vivo na devnet e a lógica endurecida de motivo ordenado são só suas (solo, no Challenge). O gate é o `npm run verify:verifier` imprimindo a linha de aprovação completa.

**1. Monte o workspace.** O verifier mora ao lado dos seus workspaces de checkout e fica na mesma linha do kit que eles usam (`@solana/kit` 6.10.0, o último release da linha v6; o `latest` do npm é 8.0.0 em 2026-08-22 e a v7 é o padrão de peer atual do ecossistema, uma costura que os workspaces de cliente do curso revisitam depois, mas o lado de ops fica consistente com o código que ele verifica):

```bash
mkdir -p verifier/src verifier/fixtures
cd verifier
npm init -y
npm pkg set type=module
npm install @solana/kit@6.10.0
npm install -D tsx@4 typescript @types/node
npm pkg set scripts.verify:verifier="tsx verify-harness.ts"
```

As versões fixadas e o frescor delas: o `tsx` 4 é o runner que o curso inteiro usa (esta linha é a instalação dele se você estiver numa máquina nova); `type=module` importa porque o harness usa `import.meta.dirname` para achar as fixtures dele. O diretório `fixtures/` começa vazio de propósito: no passo 7 você mesmo semeia ele com cinco arquivos, um pagamento correto e quatro ataques semeados, cada um um JSON com o formato de `getTransaction` mais o pedido que ele diz pagar e o motivo que o verifier tem que devolver.

**2. Os tipos, que também são o contrato.** Salve como `verifier/src/types.ts`. Os formatos de `ExpectedOrder` e `VerifyResult` estão congelados daqui em diante: a lição do webhook, o dashboard de ops e o capstone todos chamam `verify(signature, expectedOrder)` exatamente como está tipado aqui.

```ts
// verifier/src/types.ts

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

export interface TokenBalanceEntry {
  accountIndex: number;
  mint: string;
  owner?: string;
  programId?: string;
  uiTokenAmount: { amount: string; decimals: number };
}

export interface ParsedInstructionLike {
  program?: string;
  programId: string;
  parsed?: unknown;
}

export interface VerifiableTransaction {
  meta: {
    err: unknown;
    preTokenBalances: readonly TokenBalanceEntry[];
    postTokenBalances: readonly TokenBalanceEntry[];
  } | null;
  instructions: readonly ParsedInstructionLike[];
}

export type FetchTransaction = (
  signature: string,
) => Promise<VerifiableTransaction | null>;

export interface ProcessedSignatureStore {
  has(signature: string): boolean;
  add(signature: string): void;
}
```

Duas escolhas deliberadas para reparar. `FetchTransaction` é uma função injetada, não uma chamada de RPC fixa no código, que é o que deixa o harness rodar os ataques semeados a partir das fixtures enquanto produção roda contra a devnet: mesmo verifier, fonte de testemunho diferente. E `recipientAta` pega carona no `ExpectedOrder` mesmo com as checagens se apoiando no `owner`: com owner, mint e programa todos validados, o endereço da ATA é determinístico, e o campo documenta em qual conta o delta do caminho feliz aterrissou, para os seus registros de conciliação.

**3. O store.** Salve como `verifier/src/store.ts`. Totalmente trabalhado; a varredura de descarte é o horizonte da seção de teoria virado concreto:

```ts
// verifier/src/store.ts
import type { ProcessedSignatureStore } from './types.ts';

export function createMemoryStore(
  evictionMs = 10 * 60_000,
): ProcessedSignatureStore {
  const seen = new Map<string, number>();

  function sweep(now: number): void {
    for (const [sig, storedAt] of seen) {
      if (now - storedAt > evictionMs) seen.delete(sig);
    }
  }

  return {
    has(signature) {
      return seen.has(signature);
    },
    add(signature) {
      const now = Date.now();
      sweep(now);
      seen.set(signature, now);
    },
  };
}
```

**4. O núcleo do verifier, o seu degrau de completion.** Salve como `verifier/src/verify.ts`. O esqueleto do pipeline, o colchete de dedup e a checagem de memo já vêm prontos. As duas regiões de TODO são as que a seção de teoria já derivou; escreva elas, não cole no escuro, porque os testes do challenge vão interrogar o seu entendimento das duas:

```ts
// verifier/src/verify.ts
import type {
  ExpectedOrder,
  FetchTransaction,
  ProcessedSignatureStore,
  VerifiableTransaction,
  VerifyResult,
} from './types.ts';

export const TOKEN_PROGRAM = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
export const TOKEN_2022_PROGRAM = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';

interface Credit {
  programId: string;
  mint: string;
  delta: bigint;
}

function creditToRecipient(
  tx: VerifiableTransaction,
  recipient: string,
): Credit | null {
  if (tx.meta === null) return null;

  // TODO(delta): pair preTokenBalances to postTokenBalances by accountIndex,
  // keeping only entries owned by `recipient`. A missing pre-entry is 0n.
  // Compute delta = post - pre as bigints from uiTokenAmount.amount.
  // Return the credit with the LARGEST positive delta as
  // { programId, mint, delta }, or null when nothing credited the recipient.
  throw new Error('TODO(delta): compute the recipient balance delta');
}

function memosOf(tx: VerifiableTransaction): string[] {
  const out: string[] = [];
  for (const ix of tx.instructions) {
    if (ix.program === 'spl-memo' && typeof ix.parsed === 'string') {
      out.push(ix.parsed);
    }
  }
  return out;
}

export function createVerifier(deps: {
  fetchTransaction: FetchTransaction;
  store: ProcessedSignatureStore;
}) {
  return async function verify(
    signature: string,
    expected: ExpectedOrder,
  ): Promise<VerifyResult> {
    if (deps.store.has(signature)) {
      return { ok: false, reason: 'duplicate', signature };
    }

    const tx = await deps.fetchTransaction(signature);
    if (tx === null) return { ok: false, reason: 'not-found', signature };

    // A transaction that never credited you is a zero credit in the right
    // program and mint: the delta check rejects it, no special case needed.
    const credit = creditToRecipient(tx, expected.recipient) ?? {
      programId: TOKEN_PROGRAM,
      mint: expected.mint,
      delta: 0n,
    };

    // TODO(program-and-mint): reject with 'wrong-token-program' when the
    // credit's programId is not the classic Token program, THEN reject with
    // 'wrong-mint' when its mint is not expected.mint. Order matters: a mint
    // string only means something once its program is established.

    if (credit.delta < expected.amountBaseUnits) {
      return { ok: false, reason: 'underpaid', signature };
    }

    // Whole-field equality, never substring: ids are variable-length, so a
    // payment memoed `ord-1024` would otherwise fulfill order `ord-102`.
    const memoMatches = memosOf(tx).some((m) =>
      m.split(/[\s:]+/).includes(expected.orderId),
    );
    if (!memoMatches) {
      return { ok: false, reason: 'wrong-reference', signature };
    }

    deps.store.add(signature);
    return { ok: true, reason: 'verified', signature };
  };
}
```

**5. O adaptador de RPC.** Salve como `verifier/src/rpc.ts`. Totalmente trabalhado. É aqui que a política de commitment se encaixa, e onde o formato cru da resposta é estreitado para os nossos tipos exatamente uma vez:

```ts
// verifier/src/rpc.ts
import { createSolanaRpc, signature as asSignature } from '@solana/kit';
import type {
  FetchTransaction,
  ParsedInstructionLike,
  TokenBalanceEntry,
  VerifiableTransaction,
} from './types.ts';

// The wire shape we rely on from getTransaction with encoding: 'jsonParsed'.
// Narrowed once, here, at the RPC boundary; everything downstream is our types.
interface RawGetTransactionResponse {
  meta: {
    err: unknown;
    preTokenBalances?: readonly TokenBalanceEntry[];
    postTokenBalances?: readonly TokenBalanceEntry[];
  } | null;
  transaction: {
    message: { instructions: readonly ParsedInstructionLike[] };
  };
}

export function createRpcFetchTransaction(opts: {
  url?: string;
  commitment?: 'confirmed' | 'finalized';
} = {}): FetchTransaction {
  const rpc = createSolanaRpc(
    opts.url ?? process.env.RPC_URL ?? 'https://api.devnet.solana.com',
  );
  const commitment = opts.commitment ?? 'confirmed';

  return async (sig: string): Promise<VerifiableTransaction | null> => {
    const response = await rpc
      .getTransaction(asSignature(sig), {
        commitment,
        encoding: 'jsonParsed',
        maxSupportedTransactionVersion: 1,
      })
      .send();

    if (response === null) return null;
    const raw = response as unknown as RawGetTransactionResponse;

    return {
      meta:
        raw.meta === null
          ? null
          : {
              err: raw.meta.err,
              preTokenBalances: raw.meta.preTokenBalances ?? [],
              postTokenBalances: raw.meta.postTokenBalances ?? [],
            },
      instructions: raw.transaction.message.instructions,
    };
  };
}
```

Olhe a opção `commitment` e veja a seção de política de novo: a faixa de US$ 6 constrói este adaptador com `'confirmed'`, a faixa de fatura com `'finalized'`, e `processed` não está no tipo. A política virou irrepresentável-se-errada, que é o tipo mais barato de imposição.

Olhe o `maxSupportedTransactionVersion` já que você está aí, porque é a outra opção daquele objeto que consegue quebrar o verifier de vez, e ela quebra barulhento em vez de quieto. Ele é um teto, não uma preferência: o RPC se recusa — erro `-32015` — a devolver qualquer transação cuja versão seja maior que o número que você passa. `0` quer dizer "eu entendo legacy e v0", o que descrevia toda transação da rede até o formato de transação v1 chegar, e não descreve mais. Passe `1` e você lê os dois. Deixe em `0` e, no dia em que um cliente te pagar com uma transação v1, o seu verifier não devolve a resposta errada, ele devolve um erro, e aquele pagamento fica sem conciliar até alguém reparar. Isto é um parâmetro de protocolo, não uma capacidade do SDK, então não te custa bump de versão nenhum: o kit 6.10.0, a versão em que este workspace está fixado, já tipa `TransactionVersion` como `'legacy' | 0 | 1`.

**6. O harness.** Salve como `verifier/verify-harness.ts`. Ele roda toda fixture pelo seu verifier com um fetch injetado apoiado em fixtures, afirma que cada uma devolve o motivo esperado, e depois opcionalmente verifica um pagamento ao vivo na devnet duas vezes para provar o dedup. Uma guarda que vale apontar antes de você salvar: um diretório `fixtures/` vazio falha barulhento em vez de passar. Um harness que não acha nada para testar e mesmo assim imprime a linha de aprovação é o check verde do frontend de novo, uma alegação sem testemunha por trás:

```ts
// verifier/verify-harness.ts
// Runs the verifier against the seeded-attack fixtures you author in step 7,
// then (when REAL_SIGNATURE is set) against a live devnet payment. This is the
// course's acceptance harness: later lessons re-run it against their own
// artifacts.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { createVerifier } from './src/verify.ts';
import { createMemoryStore } from './src/store.ts';
import { createRpcFetchTransaction } from './src/rpc.ts';
import type {
  ExpectedOrder,
  VerifiableTransaction,
  VerifyResult,
} from './src/types.ts';

interface Fixture {
  name: string;
  signature: string;
  expectedReason: VerifyResult['reason'];
  order: Omit<ExpectedOrder, 'amountBaseUnits'> & { amountBaseUnits: string };
  transaction: VerifiableTransaction;
}

function fail(msg: string): never {
  console.error(`VERIFY FAIL: ${msg}`);
  process.exit(1);
}

async function main() {
  const fixtureDir = join(import.meta.dirname, 'fixtures');
  const fixtures: Fixture[] = readdirSync(fixtureDir)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(join(fixtureDir, f), 'utf8')));

  if (fixtures.length === 0) {
    fail(`no fixtures in ${fixtureDir}: nothing was tested`);
  }

  const bySignature = new Map(fixtures.map((f) => [f.signature, f.transaction]));
  const verify = createVerifier({
    fetchTransaction: async (sig) => bySignature.get(sig) ?? null,
    store: createMemoryStore(),
  });

  for (const f of fixtures) {
    const result = await verify(f.signature, {
      ...f.order,
      amountBaseUnits: BigInt(f.order.amountBaseUnits),
    });
    if (result.reason !== f.expectedReason) {
      fail(`${f.name}: expected ${f.expectedReason}, got ${result.reason}`);
    }
    console.log(`  ${f.name}: ${result.reason}`);
  }

  const realSig = process.env.REAL_SIGNATURE;
  if (realSig) {
    const order: ExpectedOrder = {
      orderId: process.env.ORDER_ID ?? fail('set ORDER_ID for the live check'),
      recipient: process.env.MERCHANT ?? fail('set MERCHANT'),
      recipientAta: process.env.MERCHANT_ATA ?? fail('set MERCHANT_ATA'),
      mint: process.env.MINT ?? fail('set MINT'),
      amountBaseUnits: BigInt(process.env.AMOUNT_BASE_UNITS ?? '0'),
    };
    const liveVerify = createVerifier({
      fetchTransaction: createRpcFetchTransaction(),
      store: createMemoryStore(),
    });
    const first = await liveVerify(realSig, order);
    if (first.reason !== 'verified') fail(`live payment: ${first.reason}`);
    const second = await liveVerify(realSig, order);
    if (second.reason !== 'duplicate') {
      fail(`redelivery not deduped: ${second.reason}`);
    }
    console.log('  live devnet payment: verified once, duplicate on redelivery');
  }

  console.log(
    'verifier: correct payment fulfilled; wrong-token, wrong-mint, underpay, replayed-reference rejected; signature stored',
  );
}

main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
```

**7. Semeie os ataques.** O harness só é tão honesto quanto as transações que você dá para ele, então você mesmo escreve as testemunhas: cinco arquivos em `verifier/fixtures/`, cada um com o formato exato da interface `Fixture` no topo do harness que você acabou de salvar. Todo arquivo carrega um `transaction` com formato de `getTransaction`, o `order` que ele diz pagar e o único motivo que o seu verifier tem que devolver para ele. As fixtures fixam de propósito os endereços de mint reais de mainnet do USDC e do USDT, porque a regra de que o endereço é a identidade é mais fácil de internalizar com as identidades de verdade na página; a rodada ao vivo do passo 8 troca pelo seu mint de devnet através das variáveis de ambiente, e o verifier nunca percebe a diferença. Os prefixos numéricos só mantêm a saída do `readdirSync` na ordem de leitura.

![Os quatro campos de um arquivo de fixture anotados com o jeito que o harness consome eles, a signature chaveando o fetch falso e o motivo esperado guiando a asserção.](assets/v07-annotated-code.webp)

Primeiro, o pagamento que tem que passar. O lado do comprador da transferência pega carona nos arrays de saldo de propósito: o seu código de delta tem que achar a entrada que pertence ao lojista no meio de estranhos, que é o objetivo inteiro de chavear pelo `owner`. O crédito é exatamente 30 USDC, pre 1.000000 e post 31.000000. Salve como `verifier/fixtures/01-correct-payment.json`:

```json
{
  "name": "correct payment",
  "signature": "FixSigCorrectPayment11111111111111111111111",
  "expectedReason": "verified",
  "order": {
    "orderId": "ord-1024",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "BuyerWa11etAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "80000000", "decimals": 6 }
        },
        {
          "accountIndex": 2,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "BuyerWa11etAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "50000000", "decimals": 6 }
        },
        {
          "accountIndex": 2,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "31000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-token",
        "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
      },
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1024:clube-single"
      }
    ]
  }
}
```

Em seguida, o sósia do ataque 2. Tudo o que uma checagem parcial lê está certo: o valor exato, o memo copiado, e a conta que pertence ao lojista não tem entrada pre nenhuma, porque o atacante criou ela dentro desta mesma transação. A sua regra de pre-ausente-é-zero é o que faz o delta dar 30. Só o `programId` denuncia ele, e é por isso que esta fixture prova que as suas checagens rodam na ordem certa. Salve como `verifier/fixtures/02-wrong-token-program.json`:

```json
{
  "name": "token-2022 look-alike",
  "signature": "FixSigWrongProgram2222222222222222222222222",
  "expectedReason": "wrong-token-program",
  "order": {
    "orderId": "ord-1025",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "FakeUsdcTwentyTwo22222222222222222222222222",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
          "uiTokenAmount": { "amount": "30000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1025:clube-single"
      }
    ]
  }
}
```

Ataque 3: 30 USDT de verdade sob o programa certo, com um memo correto, aterrissando na sua conta de USDT. A checagem de owner ainda acha o crédito mesmo ele nunca tendo encostado no `recipientAta` do pedido, e a checagem de mint tem que ser a que mata ele. Salve como `verifier/fixtures/03-wrong-mint.json`:

```json
{
  "name": "usdt into your usdt account",
  "signature": "FixSigWrongMint3333333333333333333333333333",
  "expectedReason": "wrong-mint",
  "order": {
    "orderId": "ord-1026",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "5000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "35000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1026:clube-single"
      }
    ]
  }
}
```

Ataque 4: token certo, 1.50 a menos. O delta dá 28500000 contra um esperado de 30000000, e todo outro campo é honesto. Salve como `verifier/fixtures/04-underpaid.json`:

```json
{
  "name": "right token, 1.50 short",
  "signature": "FixSigUnderpaid4444444444444444444444444444",
  "expectedReason": "underpaid",
  "order": {
    "orderId": "ord-1027",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "29500000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-1027:clube-single"
      }
    ]
  }
}
```

Ataque 5: uma transação genuína, integralmente paga, do `ord-0999`, reapresentada contra o pedido `ord-1024`. Toda checagem até o memo passa, e o memo é a única coisa que sobrou de pé entre esta signature e um disco de graça. Salve como `verifier/fixtures/05-wrong-reference.json`:

```json
{
  "name": "ord-0999 payment replayed against ord-1024",
  "signature": "FixSigWrongReference55555555555555555555555",
  "expectedReason": "wrong-reference",
  "order": {
    "orderId": "ord-1024",
    "recipient": "WavRecordsMerchant111111111111111111111111",
    "recipientAta": "WavRecordsUsdcAta1111111111111111111111111",
    "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    "amountBaseUnits": "30000000"
  },
  "transaction": {
    "meta": {
      "err": null,
      "preTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "1000000", "decimals": 6 }
        }
      ],
      "postTokenBalances": [
        {
          "accountIndex": 1,
          "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
          "owner": "WavRecordsMerchant111111111111111111111111",
          "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          "uiTokenAmount": { "amount": "31000000", "decimals": 6 }
        }
      ]
    },
    "instructions": [
      {
        "program": "spl-memo",
        "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        "parsed": "wavelength:ord-0999:clube-single"
      }
    ]
  }
}
```

**8. Rode.** Fixtures primeiro:

```bash
npm run verify:verifier
```

Com os seus dois TODOs preenchidos corretamente, toda fixture imprime o motivo dela e a linha final é a frase de aprovação completa. Se o harness recusar com `no fixtures`, os seus cinco arquivos do passo 7 não estão onde o `import.meta.dirname` aponta; aquela recusa é deliberada, porque um harness que não testou nada não tem nada que imprimir uma aprovação. Se `wrong-token-program` voltar como `wrong-mint`, as suas checagens estão na ordem errada; se a fixture de underpaid verificar, o seu delta compara floats ou strings em vez de bigints. O conjunto de fixtures cobre exatamente o que a teoria derivou:

![Tabela de cinco fixtures, um pagamento correto e quatro ataques, cada uma emparelhada com o único motivo que o verifier tem que devolver e a propriedade que aquele motivo prova.](assets/v08-table.webp)

Depois a metade ao vivo. Faça um pagamento novo na devnet pelo seu checkout da lição do QR, ou mande um direto com o transfer-kit — de um jeito ou de outro ele tem que vir do cliente de mentira, não do seu keypair de lojista, porque a checagem de valor deste verifier é um delta de saldo na sua própria conta de token e um autopagamento move ele em zero:

```bash
PAYER_KEYFILE=/tmp/customer.json \
  npm run --workspace transfer-kit pay -- $(solana address) 12.5
```

Anote a signature que ele imprime e o id de pedido contra o qual você está testando, e então:

```bash
REAL_SIGNATURE=<sig> ORDER_ID=<id> MERCHANT=$(solana address) \
MERCHANT_ATA=<your usdc ata> MINT=<your devnet mint> AMOUNT_BASE_UNITS=<price> \
npm run verify:verifier
```

O checkpoint é concreto: o harness imprime `live devnet payment: verified once, duplicate on redelivery` seguido da linha de aprovação. Aquela segunda cláusula é a sua primeira sobrevivência a retry de webhook, provada antes de você ter recebido um webhook na vida.

## Challenge

**O challenge de código (harden-verify, no widget de challenge)** te entrega um starter que é o verifier ingênuo da seção de teoria mais a checagem de dedup que você já construiu; os cinco buracos que sobram são seus para fechar. Ele vem com um conjunto de transações contendo um pagamento correto e cada ataque semeado. O starter cumpre o pagamento com o token errado, de propósito; assista isso acontecer uma vez antes de consertar qualquer coisa, porque ver o falso positivo é a lição. O seu trabalho é o contrato de motivo ordenado: exatamente um motivo por chamada entre `duplicate`, `wrong-token-program`, `no-payment`, `wrong-mint`, `underpaid`, `wrong-reference` e `verified`, com o valor sempre computado a partir do delta de saldo on-chain. O widget chama o seu `verifyPayment` posicionalmente, do jeito que o corretor dele consegue — aqui está o contrato de sete argumentos como lista, porque você vai reler ele no meio do debug:

1. A transação parseada, como uma única string JSON.
2. `expectedMint`, direto.
3. `expectedTokenProgram`, direto.
4. `recipientAta`, direto.
5. `expectedAmount`, como bigint.
6. `orderRef`, direto.
7. O conjunto de signatures cumpridas, como outra string JSON.

O starter já faz `JSON.parse` das duas strings (argumentos 1 e 7) na entrada, então as suas checagens trabalham com valores de verdade; os valores de transferência chegam como strings decimais de unidades base, o formato em que o `getTransaction` reporta eles, então levante eles com `BigInt()` antes da matemática de dinheiro. `Number()` só é exato abaixo de 2**53 e uma das transações do widget liquida acima disso, que é a lição dos decimais cobrando a dívida dela. Duas pequenas diferenças de formato em relação ao lab, as duas declaradas no widget: ele te entrega uma lista de transferências já achatada em vez de arrays de saldo pre/post, então "nada aterrissou na sua ATA" é um motivo `no-payment` próprio em vez do pagamento a menor de delta zero do lab, e o memo dele é a ref de pedido nua, então a checagem de reference é igualdade sobre o memo inteiro em vez do casamento de campo inteiro dentro de um memo estruturado do lab. Os dois recusam um prefixo, que é a propriedade que importa. As três hints do widget são os três erros que todo mundo comete, em ordem de popularidade: confiar numa transferência antes de checar o programa dela, ler um valor de um campo do cliente, e devolver uma pilha de motivos em vez do primeiro.

**O degrau solo** é o gate de avaliação desta lição, e é ele que dá nome a este módulo. Rode o seu verifier completo contra um pagamento real na devnet e contra todos os ataques semeados; ele tem que cumprir só o correto e guardar a signature dele. Depois faça a parte que nenhum teste consegue checar por você, e faça no arquivo que está esperando por isso desde o módulo 1: abra o `commitment-policy.md`, o documento de quatro títulos que você rascunhou em m01-l2 e copiou para a raiz `wavelength` em m02-l1. O módulo 1 prometeu duas vezes que o módulo 4 transformaria aquele arquivo em código rodando, e este é o passo que paga a promessa. Afie os títulos dele numa tabela de três faixas de valor, cada uma com o commitment level dela e uma frase defendendo ele contra a assimetria do sem-chargeback — e depois conecte: a string de commitment que o seu verifier passa para o `getTransaction` tem que ser a que a sua tabela nomeia para aquela faixa, não um default que alguém digitou uma vez. Uma política de que o código discorda é um documento, não uma política. Não existe tabela universalmente certa. Existe uma tabela que o seu código obedece e que você consegue defender, e as duas metades são a habilidade.

Mais uma coisa antes de você fechar o editor, porque ela reenquadra tudo o que você acabou de construir. Este verifier sobrevive à lição de hoje:

![Diagrama mostrando a função verify construída hoje sendo consumida pela lição do webhook, pelos degraus de pagamento posteriores e pelo harness de aceitação do capstone, todos afunilando signatures pelas mesmas checagens.](assets/v09-diagram.webp)

O que quer que você construa para a Wavelength daqui em diante, a verdade sobre pagamentos passa por esta única função. Drift de interface aqui quebra toda lição posterior, que é exatamente por que os tipos congelaram no passo 2.

## Checkpoint: o que a aprovação prova

Se o harness empurrou de volta, a falha está numa lista curta. Motivos voltando na ordem errada quer dizer que a sua checagem de programa está depois da checagem de mint; releia a última frase do ataque 2. Um `verified` na fixture de underpaid quer dizer comparação de float ou de string; deltas são bigints ou são bugs. Um `not-found` no seu pagamento ao vivo normalmente quer dizer que o problema de velocidade do webhook chegou cedo: você verificou em `confirmed` antes de o RPC conseguir ver a transação, então espere um instante e rode de novo. E se a rodada ao vivo verificou mas a segunda chamada não imprimiu `duplicate`, o seu `store.add` está no lugar errado; ele pertence depois da checagem final, em nenhum lugar antes.

Quando a linha de aprovação imprimir, pare no que você de fato tem na mão. Quando este módulo abriu, duas das suas três superfícies despachavam na palavra de um frontend e a terceira só respondia por pagamentos que ela já estava vigiando. Agora nada é despachado até uma função que você escreveu ler o livro-razão e concordar, ela não pode ser cumprida duas vezes por um retry, não pode ser enganada por um programa falsificado nem por um mint sósia nem por um recibo reapresentado, e o nível de certeza que ela exige é uma política que você precificou deliberadamente, por pagamento. Isso é o back office valendo o que custa, e foi a sua construção mais difícil até aqui. Fique com a vitória!

Agora você tem na mão o verifier que é o harness de aceitação do curso. Mas repare no que ele ainda não consegue fazer: ele só inspeciona pagamentos sobre os quais alguém o avisa. Alguém tem que entregar uma signature para ele. Na próxima lição esse alguém chega, o webhook que vigia o seu endereço e chama o seu servidor a cada pagamento, e a gente fica preciso sobre exatamente como aquele webhook mente: reentregas, eventos fora de ordem, e por que a idempotência dele chaveada por signature aterrissa no store que você construiu hoje.
