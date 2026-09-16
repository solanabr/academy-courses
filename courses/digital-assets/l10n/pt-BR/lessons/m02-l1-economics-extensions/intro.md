# Extensões de economia: taxas, juros, UI escalada e para ONDE as taxas VÃO

## Resumo

Na lição passada você derivou a matriz de conflitos de extensões direto da fonte e construiu o check-combo, que aceita um conjunto legal de extensões e rejeita um ilegal. Agora você gasta essa matriz: você começa a construir o SPROUT de verdade.

Olha como foi a minha primeira passada nessa construção. Eu configurei uma taxa de transferência de 1% num mint de teste, rodei dez vendas e abri a conta de tesouraria para admirar a bolada. Zero. Não era erro de arredondamento, não era atraso: zero. As taxas eram reais, o programa tinha arrecadado cada uma delas, e elas estavam sentadas em algum lugar onde eu nunca tinha pensado em olhar. Esse algum lugar é toda a segunda metade desta lição, porque o Token-2022 retém taxas de transferência na conta de token do DESTINATÁRIO, e nada, nunca, move elas para uma tesouraria até você fazer isso você mesmo.

Hoje o SPROUT, a moeda de Overgrowth, o jogo fictício co-op de farming cuja economia este curso levanta, faz crescer a camada de economia dele: um mint Token-2022 carregando TransferFeeConfig mais uma das duas extensões que reescrevem a exibição, InterestBearingConfig ou ScaledUiAmount, construído a partir de instruções brutas com o kit. Você vai rodar transferências reais num surfnet local, ver as taxas se empilharem onde você menos espera e depois varrer elas para casa com a sequência de harvest que o trilho de roteamento de taxas do Módulo 9 depois chama pelo nome.

Uma coisa para fazer antes da teoria: levante o lab. Faça o scaffold ao lado do seu trabalho anterior e suba um surfnet local (surfpool 1.2.1 aqui, 2026-08-22; no macOS `brew install txtx/taps/surfpool`, em outras plataformas pegue ele na página de releases do surfpool):

```bash
mkdir -p labs/m02-l1 && cd labs/m02-l1
npm init -y && npm pkg set type=module
surfpool start --no-tui --no-studio
```

Deixe o surfnet rodando nesse terminal e abra um segundo. A gente vai enchendo a pasta conforme avança.

O recuo da ajuda nesta lição: a construção do mint é trabalhada por inteiro, eu caminho por cada instrução e você digita junto. As constantes de taxa e a sequência de harvest são exercícios de completion: o arquivo já vem com TODOs e a teoria te diz exatamente o que vai neles. E a aritmética da taxa em si é solo: o coding challenge do módulo te entrega um `transferFee` quebrado e uma suíte de testes, sem apoio.

## O ciclo de vida da taxa

### Uma extensão que move dinheiro, duas que só falam sobre ele

As três extensões deste conjunto parecem irmãs e não são. O TransferFeeConfig muda o que uma transferência FAZ: tokens de fato se movem de outro jeito, alguém de fato recebe menos. O InterestBearingConfig e o ScaledUiAmount mudam o que um saldo PARECE: eles reescrevem o número que uma carteira mostra enquanto o valor bruto on-chain fica intocado. Segure essa separação com firmeza, porque toda cilada daqui para frente vem de embaçar ela.

Comece pela que move dinheiro. O TransferFeeConfig é estado em nível de mint com duas autoridades e duas tabelas de taxa dentro dele. A taxa em si são dois números: `transfer_fee_basis_points`, a porcentagem em centésimos de por cento, e `maximum_fee`, um teto duro em unidades base. Em toda transferência o programa computa:

taxa = ceil(amount x basis_points / 10000), limitada a maximum_fee

A direção do arredondamento não é um detalhe. A taxa arredonda para CIMA, sempre, então o programa nunca cobra a menos de si mesmo: mande 1,001 unidades base a 100 bps e a matemática crua diz 10.01, o programa retém 11. E o teto é um limite sobre o quanto se leva no absoluto: a 100 bps com um teto de 5 tokens, uma venda de 250 tokens deve 2.5 e paga 2.5, enquanto uma venda de 750 tokens deve 7.5 e paga exatamente 5. Baleias ganham desconto, por design, porque o teto existe para impedir que transferências grandes e legítimas sangrem.

![A fórmula da taxa de transferência do Token-2022, divisão com arredondamento para cima limitada ao maximumFee, mostrada como TypeScript anotado com o exemplo de arredondamento 1,001-unidades-retém-11 destacado.](assets/v01-annotated-code.webp)

Mais duas formas dentro da extensão merecem uma olhada antes de a gente construir, porque a biblioteca cliente vai te forçar a reconhecer elas de qualquer jeito. Primeiro, as autoridades são separadas: a `transfer_fee_config_authority` pode mudar a taxa, e a `withdraw_withheld_authority` pode arrecadar ela. Duas chaves, dois trabalhos, separáveis de propósito: uma DAO pode segurar o poder de definir a taxa enquanto um bot de ops segura o poder de varrer, e revogar uma não revoga a outra. E revogação aqui não é simétrica nas consequências. Coloque a autoridade de config em none e a tabela de taxa fica congelada para sempre, o que é uma feature de compromisso crível: holders sabem que 1% nunca pode virar 10%. Coloque a autoridade de withdraw em none e toda taxa que o seu token já reteve, passada e futura, fica encalhada permanentemente, porque fazer harvest para o mint continua sem permissão mas nada nunca mais consegue puxar a pilha para fora. Uma dessas revogações é uma promessa. A outra é uma lápide. Decida qual autoridade vai para um multisig, qual para uma chave de ops e qual você algum dia zeraria, antes da inicialização, porque essa extensão já vem de nascença e a fiação das autoridades é parte do produto.

![Tabela dos quatro assentos de autoridade, os poderes deles e os resultados de revogação, de uma tabela de taxa congelada até taxas retidas permanentemente encalhadas.](assets/v02-table.webp)

Segundo, a config guarda DUAS tabelas de taxa completas, um mais velho e um mais novo. Uma mudança de taxa não entra em vigor quando você assina ela; a `set_transfer_fee` arma a tabela mais nova para uma epoch futura, e toda transferência pega a tabela que bate com a epoch em que ela executa. Ninguém leva rug no meio do voo por uma taxa que dobrou entre a prévia da carteira e a confirmação, e um integrador cotando taxas consegue ler as duas tabelas e saber exatamente qual se aplica quando. Quando você construir a forma da extensão no lab e os tipos exigirem `olderTransferFee` E `newerTransferFee`, isso não é boilerplate, isso é o mecanismo anti-rug olhando de volta para você.

![A autoridade assina set_transfer_fee no meio da epoch, a tabela de 200 bps arma numa fronteira de epoch futura, e as transferências continuam cobrando 100 bps até ela passar.](assets/v03-timeline.webp)

Então uma transferência dispara e uma taxa é computada. Para onde ela vai?

### A revelação: as taxas moram no destinatário

Nem perto da sua tesouraria. A taxa é retida NA conta de token do destinatário, dentro de um slot com que a conta já nasceu. Você conheceu o mecanismo na lição passada em `required_init_account_extensions`: TransferFeeConfig num mint força uma extensão TransferFeeAmount em toda conta de token daquele mint. Esse slot é onde as taxas se acumulam, por holder, invisivelmente, para sempre, até alguém varrer elas.

Mande 250 SPROUT a 100 bps e a conta do comprador recebe 247.5 SPROUT gastáveis mais 2.5 SPROUT de taxas retidas que ela não pode tocar. Rode dez vendas para dez compradores diferentes e a receita do seu protocolo agora está espalhada por dez contas como poeira retida. Nada roteia automaticamente. Não existe tesouraria nenhuma no quadro até você introduzir uma.

O slot forçado também tem um preço, e ele é pago por holder, não por você. Toda conta de token de um mint com taxa é algumas dezenas de bytes maior que uma simples, e bytes de conta são rent: cada novo holder paga um mínimo isento de aluguel um pouco mais alto na criação da conta, para sempre, tenha ele acumulado uma taxa retida algum dia ou não. Em uma conta é ruído. Num token com cem mil holders é um imposto permanente sobre toda a sua base de usuários, e foi você que inscreveu eles nele na inicialização do mint. Esta é a mesma lição que a matemática de bytes de m01-l2 ensinou pelo lado da leitura, agora pelo lado da emissão: extensões não são de graça para as pessoas que só têm o seu token.

![Diagrama de uma venda de 250 SPROUT a uma taxa de 1%: a conta do comprador segura 247.5 gastáveis mais 2.5 retidos no slot TransferFeeAmount dela, enquanto a tesouraria fica vazia e sem envolvimento.](assets/v04-diagram.webp)

Por que construir assim? Derive isso do que você sabe sobre o runtime em vez de aceitar como esquisitice. Suponha que as taxas roteassem inline para uma tesouraria. Então toda transferência do token precisaria da conta de tesouraria na lista de contas dela, gravável. Uma conta quente e gravável compartilhada por toda transferência significa que duas transferências do seu token nunca conseguem executar em paralelo: você teria serializado a sua economia de token inteira através de uma única trava. Pior, a forma de contas da instrução de transferência mudaria toda vez que a tesouraria se movesse. Reter no destinatário mantém cada transferência tocando só as contas que ela já estava tocando, então a execução paralela sobrevive, e isso converte o roteamento de taxas no que ele honestamente é: um job em lote assíncrono. O protocolo não faz o trabalho por você. Ele só torna o trabalho possível, e barato.

Esse job em lote tem um nome, três nomes na verdade, e eles são o vocabulário de trabalho desta lição. A `harvest_withheld_tokens_to_mint` varre taxas retidas de qualquer lista de contas de token para o próprio mint, para dentro de um campo `withheld_amount` dentro do TransferFeeConfig do próprio mint. Ela é sem permissão: qualquer um pode chamar ela, porque ela só consolida, ela não consegue roubar. A `withdraw_withheld_tokens_from_mint` então move a pilha consolidada do mint para qualquer conta de token de destino, e ESTA aqui passa pela cancela da `withdraw_withheld_authority`. Existe também uma rota direta, a `withdraw_withheld_tokens_from_accounts`, que puxa das contas de token direto para um destino em um único salto com cancela de autoridade, útil quando você quer as taxas para fora de contas específicas sem a escala no mint.

![Fluxograma do ciclo de vida da taxa: taxas retidas em contas de destinatário, harvest sem permissão para o mint, withdraw com cancela de autoridade para uma tesouraria, mais uma rota direta contas-para-destino com cancela de autoridade.](assets/v05-flowchart.webp)

Agora a consequência operacional, porque esta é a metade trade-off do acordo. Uma taxa de transferência torna um token autofinanciado, o que é uma capacidade real: o trilho de roteamento de taxas que você constrói no Módulo 9 transforma exatamente essa mecânica em buybacks automatizados. O custo disso: destinatários recebem menos do que o remetente mandou, o que quebra toda integração que assumiu que valor-de-entrada é igual a valor-de-saída; taxas se acumulam invisivelmente por milhares de contas; e fazer harvest é um job de ops recorrente, com custos de CU reais, que o seu protocolo agora carrega para a vida toda. Quem chama o harvest, com que frequência, e quem paga por essas transações é uma questão de equipe, não uma questão de código. A maioria dos postmortems de token com taxa não é exploit. É ninguém-rodou-o-cron.

A quebra de integração merece a própria cena concreta dela, porque você vai estar de um dos lados dela em algum momento. Uma exchange credita depósitos pelo valor que o remetente afirma ter enviado, envia 100 tokens de um mint com taxa para um saque de usuário, e o usuário recebe 99. Agora o livro-razão interno da exchange e a chain discordam em um token por saque, compondo, e os tickets de suporte fazem a contabilidade. Os padrões defensivos são chatos e obrigatórios: credite o que CHEGOU, nunca o que foi enviado (leia o delta de saldo do destino, não o amount da instrução); cote as taxas para os usuários antes de eles assinarem, usando a fórmula exata acima contra a tabela da epoch ATUAL; e para remetentes que prometem um valor exato recebido, faça gross-up do envio para que a chegada pós-taxa bata com a promessa, lembrando que o arredondamento para cima joga contra você. O gross-up é chato o suficiente para anotar uma vez e guardar:

```ts
// gross-up: smallest send amount whose post-fee arrival covers `target`.
function grossUp(target: bigint, basisPoints: number, maximumFee: bigint): bigint {
  if (basisPoints === 0 || target === 0n) return target;
  let amount = (target * 10_000n) / (10_000n - BigInt(basisPoints));
  while (amount - transferFee(amount, basisPoints, maximumFee) < target) amount += 1n;
  return amount;
}
// netting exactly 100 SPROUT at 100 bps means sending 101.010102
```

A forma fechada te deixa a menos de uma unidade e o loop absorve o viés do arredondamento para cima; prometer a um usuário "você vai receber exatamente X" sem isso é como filas de suporte nascem. Nada disso é difícil. Tudo isso tem que ser feito de propósito, e as integrações anteriores ao Token-2022 não fazem nada disso por padrão, que é boa parte do motivo pelo qual os venues colocam tokens com taxa atrás da cancela de allowlists.

O crank em si tem espaço de design que vale conhecer antes de o Módulo 9 automatizar ele. A `harvest_withheld_tokens_to_mint` recebe um array `sources` inteiro, então uma única instrução varre muitas contas sujas de uma vez, e o meu custo medido para um harvest de uma fonte só ficou em torno de 1,200 CU: consolidar é quase de graça, que é exatamente o que você quer para uma chamada sem permissão. Esse não pedir permissão a ninguém também é um pequeno presente para o ecossistema: um indexador, um bot, até um rival consegue arrumar as suas taxas na direção do mint, e nada se perde porque só a autoridade de withdraw consegue dar o salto final. A rota direta, a `withdraw_withheld_tokens_from_accounts`, abre mão dessa divisão de trabalho: uma única chamada com cancela de autoridade, mas a autoridade tem que assinar toda varredura e a lista de contas anda numa transação que ela paga. Duas pernas para arrecadação de rotina em escala, direto para pulls cirúrgicos. De qualquer jeito o problema de achar as contas sujas é seu, e é um problema de indexação: enumere as contas de token do mint, filtre por um valor retido diferente de zero, varra. A lição de roteamento de taxas do Módulo 9 é onde este curso faz essa enumeração direito, a caminho de automatizar o crank.

### As duas extensões de exibição, e por que elas não podem coexistir

Agora os faladores. O InterestBearingConfig guarda uma taxa em basis points (um i16, então ela pode ser negativa: sim, você pode configurar decaimento) mais timestamps, e instrui os clientes a exibir saldos como se eles tivessem capitalizado continuamente desde a inicialização. O saldo de UI de um holder deriva para cima dia após dia. O valor bruto na conta dele não se move. Nenhum token é cunhado, nenhum jamais será por esta extensão, e no momento em que qualquer código tratar esse número de exibição crescente como supply ele está contando duas vezes um valor que não existe. A extensão é uma convenção contábil com uma âncora on-chain: útil para títulos e wrappers que rendem juros, onde o valor de face do papel cresce, perigosa no instante em que alguém liga o `ui_amount` na matemática de liquidação. A chain até já vem com uma instrução `amount_to_ui_amount` que você pode simular para obter o valor exibido autoritativo, que é o jeito educado de dizer: a conversão é definida pelo programa, não pela sua planilha.

A taxa nem está travada: a `rate_authority` pode mudar ela, e os campos de estado de aparência esquisita da extensão existem exatamente para esse momento. Quando uma atualização de taxa aterrissa, o programa dobra tudo que foi acumulado até ali dentro de `pre_update_average_rate` e carimba `last_update_timestamp`, então a matemática de exibição fica por partes: a média antiga se aplica até o carimbo, o novo `current_rate` se aplica depois. A história não é reescrita quando a taxa é, que é a diferença entre um botão de rendimento e uma máquina do tempo. Quando a chamada `extension()` do lab te pedir todos os cinco campos, esse livro-razão por partes é o que você está inicializando.

Aqui está a contagem dupla na natureza, para ela parar de ser abstrata. Um protocolo de empréstimo lista um token que rende juros como colateral e, para economizar uma chamada, avalia posições pelo valor exibido na carteira. O número exibido capitaliza; os tokens brutos que dão lastro a ele não. Mês a mês os livros do protocolo criam colateral fantasma, exatamente a lacuna entre a matemática de exibição e a realidade, e a primeira cascata de liquidação marca isso a mercado de uma vez só. Nada foi hackeado. Alguém leu uma convenção de UI como se fosse um saldo. A regra que te mantém seguro é mecânica: valores brutos liquidam, valores de UI renderizam, e qualquer número que atravessa do segundo mundo para o primeiro tem que passar pela conversão do próprio programa, num timestamp que você escolheu de propósito.

![Uma linha plana de valor bruto diverge de uma curva crescente de valor exibido num mint de juros de 5%, com amount_to_ui_amount como a única ponte sancionada entre elas.](assets/v06-diagram.webp)

O ScaledUiAmount é o mesmo truque com outra forma: um único multiplicador f64 aplicado a todo saldo exibido, atualizável pela autoridade dele com um timestamp de vigência que você pode agendar com antecedência. Onde o InterestBearingConfig modela deriva contínua, o ScaledUiAmount modela saltos discretos: um desdobramento de 10 para 1, um rebase, uma redenominação, tudo sem tocar em uma única conta de holder. Uma instrução atualiza o multiplicador e toda carteira da Terra re-renderiza. O agendamento é a metade subestimada: um emissor pode anunciar na terça que o desdobramento entra em vigor na segunda 00:00 UTC, assinar a atualização na hora com aquele timestamp de vigência, e todo cliente vira no mesmo instante sem janela de migração, sem snapshot, sem fluxo de claim. Para emissores que de outro jeito migrariam milhares de contas para mudar uma denominação, esta é a saída barata. A mesma disciplina se aplica como com os juros: o multiplicador reescala a história, não o supply, e qualquer coisa que liquida tem que liquidar bruto.

E as duas não podem dividir um mint: você derivou isso você mesmo na lição passada como a regra 4 de `check_for_invalid_mint_extension_combinations`, a única exclusão mútua de verdade na matriz. As duas extensões reivindicam a posse do mesmo output, o valor exibido, e dois donos de um número só, sem ordem de composição definida, é uma ambiguidade que o programa se recusa a criar. O seu check-combo já rejeita o par; hoje ele se gradua de fixture de teste para cancela de pré-voo, porque o SPROUT ganha exatamente uma dessas e o validador é o que te impede de inicializar as duas sem prestar atenção.

![Comparação entre a deriva contínua de exibição do InterestBearingConfig e os saltos em degraus do ScaledUiAmount, com nenhuma das duas cunhando supply e a regra 4 permitindo que um mint carregue só uma.](assets/v07-comparison.webp)

Qual delas o SPROUT leva? A escolha é sua, de verdade: o lab constrói o InterestBearingConfig no caminho trabalhado porque um co-op de farming pagando rendimento sobre grão estocado é o encaixe mais natural, e a troca para o ScaledUiAmount é uma mudança de duas linhas que eu vou te mostrar no fim. Seja qual for a que você escolher, o validador de combo abençoa TransferFeeConfig mais a sua escolha, e teria te impedido de levar as duas.

Um compasso de realidade de mercado antes do lab, porque ele responde "alguma coisa vai sequer negociar isso?" A página de suporte a Token-2022 da Raydium fala a parte silenciosa em voz alta: a implementação de referência dela coloca na whitelist exatamente as extensões de taxa, de exibição e de contabilidade, TransferFeeConfig, MetadataPointer, TokenMetadata, InterestBearingConfig, ScaledUiAmount, e rejeita todo o resto, os poderes com forma de compliance incluídos (docs.raydium.io, 2026-08-21). Toda extensão que o SPROUT leva desta lição está nessa allowlist por design, e o raciocínio é exatamente a separação com que esta lição abriu: uma taxa ou um multiplicador de exibição não consegue varrer o vault de uma pool. As extensões que conseguem agir sobre saldos de outras pessoas são as que os venues recusam, e essa história, legal-mas-não-roteável, é o argumento de abertura do Módulo 5.

E se você está se perguntando por que o seu tutorial favorito nunca mencionou duas das três extensões de hoje: a educação oficial da Solana congelou no meio da trama. O repo solana-foundation/developer-content foi arquivado em 2025-01-24, então todo curso construído a partir daquele cânone é anterior a ScaledUiAmount, Pausable e ConfidentialMintBurn. As extensões que você está prestes a inicializar literalmente não existem na maior parte do material a partir do qual o ecossistema ainda ensina. Você está aprendendo com o programa porque, para esta camada, o programa é atualmente o único professor que acompanhou.

## Lab: construir o conjunto de economia do SPROUT

O artefato é o `sprout-mint-economics`: um mint Token-2022 com TransferFeeConfig mais InterestBearingConfig (ou ScaledUiAmount), uma rodada de transferências que espalha taxas retidas, e um harvest que varre elas para uma tesouraria e prova a aritmética. Ele consome as duas ferramentas que você já tem: o check-combo passa o conjunto pela cancela antes de qualquer lamport se mover, e o decode-mint inspeciona o resultado depois.

1. Instale a toolchain na pasta `labs/m02-l1` em que você fez o scaffold. Os pins precisam de um minuto de honestidade. O kit mais recente do npm é o 8.2.0 (publicado em 2026-08-29), mas "mais recente" não é a regra que decide um pin: você fixa o major do kit contra o qual os clientes `@solana-program/*` do seu workspace fazem peer, e os clientes deste workspace fazem peer com o kit ^7, verificado contra os ranges de peer reais do npm em 2026-09-05. O `@solana-program/token-2022@0.15.0` é o minor atual que faz peer com o kit ^7.0.0 (o 0.16.0 pulou para ^8), e o `@solana-program/system@0.13.0` é a contraparte dele (o 0.14.0 também pulou). Esse minor do token-2022 também faz peer com `@solana/sysvars` em ^7.0.0 e `@solana/zk-sdk` em ^0.5.1, que o npm resolve para você junto com o pin do kit, então nada mais precisa de um pin na mão. Reverifique com `npm view <pkg> peerDependencies` no dia em que você fizer o scaffold; esta matriz se move.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.20.5 typescript@5.9.3
```

2. Passe o conjunto de extensões pela cancela antes de construir qualquer coisa. Este é o primeiro dia do check-combo no trabalho para o qual ele foi construído. Crie o `gate.ts`:

```ts
// gate.ts: no SPROUT instruction is emitted until the set passes R2.
import { checkCombo } from "../m01-l4/check-combo";

const chosen = ["TransferFeeConfig", "InterestBearingConfig"];
const verdict = checkCombo(chosen);
if (!verdict.valid) {
  console.error(`illegal set: ${verdict.reason}`);
  process.exit(1);
}
console.log(`set [${chosen.join(", ")}] is legal to initialize`);

// The pairing the matrix forbids, proven rejected before we ever hit the chain:
const illegal = checkCombo(["TransferFeeConfig", "ScaledUiAmount", "InterestBearingConfig"]);
console.log(`both display extensions: ${illegal.valid ? "BUG in your R2" : illegal.reason}`);
```

Rode `npx tsx gate.ts`. O conjunto legal passa, o conjunto de exibição dupla é rejeitado com a razão da regra 4, e você acabou de usar uma coisa que você construiu para proteger uma coisa que você está prestes a construir. Esse loop é a escada de artefatos funcionando.

3. Agora a construção. Crie o `verify-economics.ts`. Ele é a lição inteira em um arquivo e eu vou te dar ele em três pedaços; digite eles no mesmo arquivo, em ordem. O pedaço um é setup: imports, constantes, a fórmula da taxa e um helper de envio que simula antes de enviar. Dois TODOs moram aqui e eles são seus: a seção de teoria já te disse que o SPROUT cobra 100 bps com um teto de 5 SPROUT.

```ts
// verify-economics.ts: SPROUT's economics layer, built from raw instructions.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  sendAndConfirmTransactionFactory,
  airdropFactory,
  lamports,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  getBase64EncodedWireTransaction,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  extension,
  getMintSize,
  getInitializeTransferFeeConfigInstruction,
  getInitializeInterestBearingMintInstruction,
  getInitializeMintInstruction,
  getCreateAssociatedTokenInstructionAsync,
  findAssociatedTokenPda,
  getMintToInstruction,
  getTransferCheckedInstruction,
  getHarvestWithheldTokensToMintInstruction,
  getWithdrawWithheldTokensFromMintInstruction,
  fetchToken,
  fetchMint,
} from "@solana-program/token-2022";

const rpc = createSolanaRpc("http://127.0.0.1:8899");
const rpcSubscriptions = createSolanaRpcSubscriptions("ws://127.0.0.1:8900");
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
const airdrop = airdropFactory({ rpc, rpcSubscriptions });

const DECIMALS = 6;
const FEE_BASIS_POINTS: number = 0; // TODO: SPROUT charges 1% on every transfer
const MAXIMUM_FEE: bigint = 0n; // TODO: capped at 5 SPROUT, expressed in base units

// Guard against the vacuous green run: with both constants at their shipped
// zeros every fee is 0n, expectedWithheld is 0, and all three headline
// assertions "pass" on a lab that never charged a fee. Fail loudly instead.
if (FEE_BASIS_POINTS === 0 || MAXIMUM_FEE === 0n) {
  throw new Error("fill in FEE_BASIS_POINTS and MAXIMUM_FEE first: a zero-fee run passes every assertion without proving anything");
}

// The on-chain formula, mirrored so the lab can assert against it.
export function transferFee(amount: bigint, basisPoints: number, maximumFee: bigint): bigint {
  if (basisPoints === 0 || amount === 0n) return 0n;
  const raw = (amount * BigInt(basisPoints) + 9_999n) / 10_000n;
  return raw < maximumFee ? raw : maximumFee;
}

async function send(feePayer: KeyPairSigner, instructions: Instruction[]) {
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(feePayer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(instructions, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  // No per-extension CU table exists anywhere. So we ask the cluster, every time.
  const sim = await rpc
    .simulateTransaction(getBase64EncodedWireTransaction(signed), { encoding: "base64" })
    .send();
  console.log(`  simulated CU: ${sim.value.unitsConsumed}`);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
}
```

Aquela chamada `simulateTransaction` dentro do helper de envio é a política de medição da lição feita executável. Congelar um número de CU por extensão tirado de um post de blog seria exatamente o erro do mapa copiado que m01-l4 gastou uma seção inteira derrubando; os números abaixo são da MINHA rodada, no MEU surfnet, e o helper existe para que toda rodada sua imprima a sua própria.

4. Pedaço dois, a construção e as vendas, anexado ao mesmo arquivo. Preste atenção na ORDEM das instruções dentro da transação do mint, porque ela é estrutural: os inicializadores de extensão rodam contra a conta alocada ANTES do `initialize_mint`, e o programa rejeita qualquer outro arranjo. A maioria das extensões do Token-2022 precisa ser habilitada na criação do mint e não pode ser adicionada depois da inicialização, então esta transação é a única e exclusiva chance do SPROUT de carregar este conjunto. Note também o que a forma do `extension()` força em você: o estado completo do TransferFeeConfig, as duas epochs de tabela de taxa incluídas, porque o `getMintSize` não consegue precificar uma conta sem o layout real.

```ts
async function main() {
  const payer = await generateKeyPairSigner();
  await airdrop({
    recipientAddress: payer.address,
    lamports: lamports(2_000_000_000n),
    commitment: "confirmed",
  });
  const mint = await generateKeyPairSigner();
  const buyer = await generateKeyPairSigner();

  const feeSchedule = {
    epoch: 0n,
    maximumFee: MAXIMUM_FEE,
    transferFeeBasisPoints: FEE_BASIS_POINTS,
  };
  const transferFeeExtension = extension("TransferFeeConfig", {
    transferFeeConfigAuthority: payer.address,
    withdrawWithheldAuthority: payer.address,
    withheldAmount: 0n,
    olderTransferFee: feeSchedule, // two schedules: the epoch-armed anti-rug
    newerTransferFee: feeSchedule,
  });
  const interestExtension = extension("InterestBearingConfig", {
    rateAuthority: payer.address,
    initializationTimestamp: 0n,
    preUpdateAverageRate: 500,
    lastUpdateTimestamp: 0n,
    currentRate: 500, // 5% APR. Display only. Supply never moves.
  });

  const space = BigInt(getMintSize([transferFeeExtension, interestExtension]));
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();
  console.log(`mint account: ${space} bytes, rent ${rent} lamports`);

  console.log("create + init mint:");
  await send(payer, [
    getCreateAccountInstruction({
      payer,
      newAccount: mint,
      lamports: rent,
      space,
      programAddress: TOKEN_2022_PROGRAM_ADDRESS,
    }),
    // Extension initializers BEFORE initialize_mint. The order is the protocol.
    getInitializeTransferFeeConfigInstruction({
      mint: mint.address,
      transferFeeConfigAuthority: payer.address,
      withdrawWithheldAuthority: payer.address,
      transferFeeBasisPoints: FEE_BASIS_POINTS,
      maximumFee: MAXIMUM_FEE,
    }),
    getInitializeInterestBearingMintInstruction({
      mint: mint.address,
      rateAuthority: payer.address,
      rate: 500,
    }),
    getInitializeMintInstruction({
      mint: mint.address,
      decimals: DECIMALS,
      mintAuthority: payer.address,
    }),
  ]);

  const [sellerAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: payer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  const [buyerAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: buyer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  console.log("create ATAs + mint supply:");
  await send(payer, [
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: payer.address }),
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: buyer.address }),
    getMintToInstruction({
      mint: mint.address,
      token: sellerAta,
      mintAuthority: payer,
      amount: 1_000_000_000n, // 1,000 SPROUT
    }),
  ]);

  // Ten sales of 25 SPROUT. Accumulate what the formula SAYS should be withheld.
  const SALE = 25_000_000n;
  let expectedWithheld = 0n;
  const sales: Instruction[] = [];
  for (let i = 0; i < 10; i++) {
    sales.push(
      getTransferCheckedInstruction({
        source: sellerAta,
        mint: mint.address,
        destination: buyerAta,
        authority: payer,
        amount: SALE,
        decimals: DECIMALS,
      }),
    );
    expectedWithheld += transferFee(SALE, FEE_BASIS_POINTS, MAXIMUM_FEE);
  }
  console.log("ten transfers:");
  await send(payer, sales);

  // The reveal, in data: the fees are on the BUYER's account.
  const buyerToken = await fetchToken(rpc, buyerAta);
  const ext =
    buyerToken.data.extensions.__option === "Some"
      ? buyerToken.data.extensions.value.find((e) => e.__kind === "TransferFeeAmount")
      : undefined;
  const withheld = ext?.__kind === "TransferFeeAmount" ? ext.withheldAmount : 0n;
  console.log(`withheld on buyer account: ${withheld} (expected ${expectedWithheld})`);
  if (withheld !== expectedWithheld) throw new Error("withheld mismatch: check your fee constants");
```

Uma dobra que vale nomear enquanto você digita: aquelas são instruções `transfer_checked` simples. Num mint com taxa o programa computa e retém a taxa por conta própria; você não faz opt-in por transferência. Existe também uma variante `transfer_checked_with_fee` que carrega a SUA taxa esperada e falha a transferência se o programa discordar, que é a jogada cinto-e-suspensórios para remetentes de produção que já cotaram uma taxa a um usuário. A gente faz o assert depois do fato em vez disso, porque o ponto deste lab é pegar o programa em flagrante.

Uma simplificação honesta: todas as dez vendas aqui aterrissam numa única ATA de comprador, então a pilha retida fica num lugar só e a asserção continua sendo uma linha. A cena de dez-contas-de-poeira-espalhada da teoria é real, mas você encontra ela no challenge, onde três compradores te forçam a enumerar e varrer várias contas sujas em um array `sources` só.

5. O pedaço três é o harvest, e esta parte é o exercício de completion. O apoio abaixo fecha a `main()`; os dois pontos de TODO são seus. Tudo de que você precisa está na teoria: a perna um é a varredura sem permissão das contas de token para o mint, a perna dois é o pull com cancela de autoridade do mint para a tesouraria. Os dois builders de instrução já estão na sua lista de imports, e as entradas deles são exatamente as contas que estão em escopo. Preencha eles.

```ts
  // Leg 1: sweep withheld fees from token accounts onto the mint. Permissionless.
  console.log("harvest accounts -> mint:");
  await send(payer, [
    // TODO: getHarvestWithheldTokensToMintInstruction. It wants the mint and a
    // `sources` array of token accounts to sweep. There is exactly one dirty
    // account in this lab so far. Which one holds the withheld fees?
  ]);

  const mintAfter = await fetchMint(rpc, mint.address);
  const mintExt =
    mintAfter.data.extensions.__option === "Some"
      ? mintAfter.data.extensions.value.find((e) => e.__kind === "TransferFeeConfig")
      : undefined;
  const onMint = mintExt?.__kind === "TransferFeeConfig" ? mintExt.withheldAmount : 0n;
  console.log(`withheld on mint after harvest: ${onMint}`);

  // Leg 2: pull the consolidated pile from the mint to the treasury. Gated.
  const treasury = await generateKeyPairSigner();
  const [treasuryAta] = await findAssociatedTokenPda({
    mint: mint.address,
    owner: treasury.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  console.log("withdraw mint -> treasury:");
  await send(payer, [
    await getCreateAssociatedTokenInstructionAsync({ payer, mint: mint.address, owner: treasury.address }),
    // TODO: getWithdrawWithheldTokensFromMintInstruction. It wants the mint, a
    // `feeReceiver` token account, and the withdrawWithheldAuthority as a SIGNER.
    // We set that authority during initialization. Who was it?
  ]);

  const treasuryToken = await fetchToken(rpc, treasuryAta);
  console.log(`treasury balance: ${treasuryToken.data.amount} (expected ${expectedWithheld})`);
  if (treasuryToken.data.amount !== expectedWithheld) {
    throw new Error("treasury balance does not equal summed fees");
  }
  console.log(`SPROUT economics mint: ${mint.address}`);
  console.log("economics lab: all assertions passed");
}

await main();
```

6. Preencha os dois TODOs de constante do passo 3 (a seção de teoria declarou os dois valores em palavras simples) e os dois TODOs de harvest, e então rode a cancela:

```bash
npx tsx verify-economics.ts
```

A minha rodada, para calibração (surfnet do surfpool 1.2.1 no solana-core 3.1.10, 2026-08-22; a sua vai derivar e esse é o ponto de medir). Dois desses números até derivam entre rodadas na MESMA máquina: os passos que criam conta aterrissaram em qualquer lugar entre 36,298 e 37,798 CU e o withdraw da tesouraria entre 18,731 e 21,731, dependendo do que já existia no surfnet. Os números de mint, transferência e harvest reproduziram até a unidade toda vez:

```
mint account: 334 bytes, rent 3215520 lamports
create + init mint:
  simulated CU: 4332
create ATAs + mint supply:
  simulated CU: 37798
ten transfers:
  simulated CU: 32470
withheld on buyer account: 2500000 (expected 2500000)
harvest accounts -> mint:
  simulated CU: 1207
withheld on mint after harvest: 2500000
withdraw mint -> treasury:
  simulated CU: 18731
treasury balance: 2500000 (expected 2500000)
SPROUT economics mint: HfVB99cPQEPGE1vPgfy3a2ynJVK552UW1SrEGt5fFsFf
economics lab: all assertions passed
```

Leia os recibos. Dez vendas de 25 SPROUT a 100 bps dá 250,000 unidades base retidas por venda, 2,500,000 no total, e lá está: primeiro encalhado na conta do comprador, depois consolidado no mint, depois aterrissado na tesouraria, até a unidade base. E a matemática de conta bate com m01-l2: um mint só com TransferFeeConfig tem 278 bytes, e o InterestBearingConfig adiciona o estado de 52 bytes dele mais um header TLV de 4 bytes, dando 334.

![Gráfico de barras das compute units medidas para as cinco transações do lab em ordem de ciclo de vida, da criação do mint passando pelas transferências com taxa e pelo harvest até o withdraw da tesouraria.](assets/v08-chart.webp)

Por que medir em vez de memorizar? Porque todo número naquele gráfico é função de coisas que se movem: a versão do programa deployada no seu cluster, o conjunto de features ativo lá, quantas extensões as suas contas carregam, se a ATA já existe. Uma transferência com taxa no SPROUT custa uns 3,200 CU no meu surfnet hoje; na mainnet no trimestre que vem, depois do próximo deploy do programa, vai custar outra coisa. O hábito que este curso fica martelando, desde que a lição de p-token derrubou uma transferência de 4,645 para 76 CU da noite para o dia, é que custos são fatos de cluster, não fatos de documentação. O seu helper de envio imprime a verdade de graça em toda rodada. Deixe ele imprimir.

7. Feche o loop com o decode-mint. O seu inspetor de m01-l2 lê o conjunto TLV de qualquer mint a partir de bytes brutos; aponte ele para o endereço do SPROUT que a sua rodada imprimiu (ajuste o caminho do import, o nome do export e o alvo de RPC para o seu próprio arquivo decode-mint, e mire ele no surfnet, `http://127.0.0.1:8899`, do jeito que a sua ferramenta receber um cluster):

```ts
// inspect-economics.ts: R1 reads what this lesson built.
import { decodeMint } from "../m01-l2/decode-mint";

const decoded = await decodeMint(process.argv[2]);
console.log(decoded.extensions.map((e) => e.name));
```

Esperado: `TransferFeeConfig` e `InterestBearingConfig` (ou `ScaledUiAmount` se você pegou o outro assento), e desta vez você consegue nomear cada byte dos dois. O inspetor que desmistificou o PYUSD no Módulo 1 agora audita um mint que você mesmo escreveu.

8. A troca para o ScaledUiAmount, se esse for o seu assento, é exatamente duas edições, mais uma cilada de nomenclatura que eu peguei para você não precisar. Para o cálculo de tamanho, a variante do `extension()` se chama `ScaledUiAmountConfig` (o cliente nomeia o ESTADO, enquanto o builder de instrução nomeia a operação, e passar `"ScaledUiAmount"` para `extension()` lança um erro de variante inválida antes de qualquer coisa chegar à chain):

```ts
const scaledExtension = extension("ScaledUiAmountConfig", {
  authority: payer.address,
  multiplier: 1,
  newMultiplierEffectiveTimestamp: 0n,
  newMultiplier: 1,
});
```

Depois troque o inicializador de juros por `getInitializeScaledUiAmountMintInstruction({ mint: mint.address, authority: payer.address, multiplier: 1 })`. Todo o resto, taxas incluídas, fica intocado. Rode a cancela primeiro: o check-combo abençoa TransferFeeConfig mais qualquer uma das extensões de exibição sozinha, e rejeita as duas juntas, que é precisamente por que o passo 2 existe.

## Challenge

O trabalho solo, sem apoio à vista.

**O coding challenge: `transferFee`, exatamente.** O challenge fee-calculator do módulo te entrega um starter que faz o floor da divisão e esquece o teto, mais uma suíte de testes de fluxos de transferência. Implemente a matemática de taxa do Token-2022: divisão com arredondamento para cima, teto de `maximumFee`, casos zero logo de cara, em bigints. O grader invoca a sua função direto como `transferFee(amount, basisPoints, maximumFee)`, posicional bigint, number, bigint, então mantenha ela exatamente como a `function transferFee(...)` simples de nível superior que já vem no starter, sem `export`, sem imports; o grader enxerta o seu arquivo no runtime dele, e sintaxe de módulo não vai parsear lá. A régua de aceitação é o comportamento on-chain até a unidade base: taxas fracionárias arredondam para CIMA, a taxa nunca ultrapassa o teto, zero bps ou valor zero não retém nada. Você já escreveu esta função uma vez dentro do lab com as respostas na frente de você; o challenge é provar que você consegue reconstruir ela só a partir da fórmula, porque o trilho de roteamento do Módulo 9 vai confiar na sua aritmética para prever o que o programa retém.

**A sondagem empírica.** O lab fez o assert do saldo retido de um comprador. Estenda a sua rodada: três compradores, um fluxo de transferências de tamanhos variados, incluindo pelo menos uma grande o suficiente para bater no teto do `maximumFee`. Compute o valor retido esperado por conta com o seu próprio `transferFee`, faça harvest das três contas em um array `sources` só, e faça o assert do total da tesouraria até a unidade base. Se a sua previsão e o programa discordarem, um dos dois está arredondando para baixo, e não é o programa.

**O confronto com a regra 4, on-chain desta vez.** Em m01-l4 você sondou um delta docs-vs-código empiricamente. Mesma disciplina, expectativa oposta: construa uma transação de mint carregando AMBOS `getInitializeInterestBearingMintInstruction` e `getInitializeScaledUiAmountMintInstruction`, dimensione a conta para os dois, e mande ela no seu surfnet. O seu check-combo prevê a resposta do programa antes de você apertar enter. Na minha rodada o programa entregou ela na instrução `initialize_mint`, custom program error 0x33 (decimal 51), o `InvalidExtensionCombination` do Token-2022: os dois inicializadores de extensão escrevem as entradas TLV deles alegremente, e é o `initialize_mint`, a última instrução, que roda as cinco regras sobre o conjunto montado e joga a transação inteira fora. Ver uma regra que você extraiu da fonte disparar de verdade, contra a sua própria transação, é o ponto inteiro de ter derivado ela.

## Checkpoint

O critério desta lição é o trio de asserções na sua rodada de verificação: retido-no-comprador é igual à sua soma computada, o harvest consolida isso no mint, e a tesouraria recebe o total exato. Ao lado da rodada que passa, escreva a resposta de uma frase que você daria a um colega de equipe que pergunta "então para onde vão as nossas taxas?" Se a sua frase contiver as palavras "até a gente fazer o harvest", você tem a mecânica; se ela contiver um cronograma de cron, você tem o negócio.

Os erros que eu espero: uma divergência de retido normalmente significa as constantes de taxa (100 bps é `100`, e 5 SPROUT com 6 decimals é `5_000_000n`, não `5n`); um withdraw que falha normalmente significa que a autoridade que você passou é um endereço onde o builder queria um signer. Se os números se recusarem a reconciliar depois disso, leve o seu fluxo de transferências e a sua matemática de taxa esperada para a discussão do curso e a gente vai achar o desacordo de arredondamento junto.

Agora você controla para onde o valor flui: o SPROUT cobra a parte dele, e você consegue varrer toda unidade retida para a tesouraria sob comando. Mas nada ainda controla QUEM tem permissão de mover, congelar ou tomar de volta. Próximo: as extensões de autoridade, incluindo a que sem alarde passa por cima das suas grades de segurança.
