# Roteamento de taxas e buyback/queima: para onde o dinheiro realmente flui

## Resumo

Na lição passada você entregou o airdrop de compost: a tabela de custo por destinatário que deixou honesta a aritmética de clássico contra comprimido, mais um claim destravado e um claim de vesting `claim_locked`, os dois provados contra o seu próprio port byte-fiel da árvore do distribuidor. Os tokens saíram pela porta. Hoje eles voltam.

O marketplace da Overgrowth fica com 1% de cada troca de SPROUT. A taxa dispara em toda transferência. Dá para ver isso nos valores que os compradores realmente recebem. E, uma semana depois do início do beta, o saldo da tesouraria continua exatamente zero. Ninguém roubou, nada está quebrado, e antes de você ler mais um parágrafo eu quero que você vá olhar o próprio mint. Premissas vigentes para esta abertura, já que um retorno a frio quebra as três: o seu surfnet dos módulos anteriores está de pé com o mint do SPROUT nele e algumas trocas cobrando taxa atrás dele (re-cunhe seguindo a abertura do m05-l1 e rode algumas transferências se você estiver voltando do zero), a raiz do workspace carrega os pins do kit e do token-2022 do m02, e você roda o comando a partir dessa raiz. Jogue isto em `labs/m09-l1/peek.ts`:

```ts
// peek.ts: how much of SPROUT's fee income has actually reached the mint?
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

async function main(): Promise<void> {
  const rpc = createSolanaRpc(process.env.RPC_HTTP ?? "http://127.0.0.1:8899");
  const mint = await fetchMint(rpc, address(process.env.SPROUT_MINT!));

  const exts = mint.data.extensions;
  const fee =
    exts.__option === "Some" ? exts.value.find((e) => e.__kind === "TransferFeeConfig") : undefined;
  if (fee?.__kind !== "TransferFeeConfig") throw new Error("no TransferFeeConfig on this mint");

  const bps = fee.newerTransferFee.transferFeeBasisPoints;
  console.log(`fee schedule: ${bps} bps, cap ${fee.newerTransferFee.maximumFee}`);
  console.log(`withheld ON THE MINT: ${fee.withheldAmount}`);
  console.log(`supply: ${mint.data.supply}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

Rode: `SPROUT_MINT=<your mint> npx tsx labs/m09-l1/peek.ts`, a partir da raiz do workspace. Você deve ver três linhas, e a terceira é o problema:

```text
fee schedule: 100 bps, cap 5000000
withheld ON THE MINT: 0
supply: 1000000000000000
```

Cem basis points, cobrados a semana inteira, e zero disso chegou ao mint.

Esse zero é a lição inteira. Hoje você transforma uma taxa que existe em dinheiro que se move, e depois em supply que desaparece. O percurso, em ordem: onde as taxas de fato ficam e o que o crank de harvest faz a respeito, depois os três modelos de taxa que as pessoas vivem confundindo e quanto custa confundi-los, depois o split que financia uma queima sem inventar tokens, depois o buyback em si, que é uma compra a um preço que alguém cobra de você, e por fim a queima, com a cilada da leitura desatualizada que anula a asserção de supply dela.

O recuo da ajuda nesta lição, em voz alta: a perna do harvest é trabalhada por inteiro, eu escrevo cada instrução e você acompanha. O split da taxa e o dimensionamento do buyback são um problema de completion, TODOs num arquivo cujo código ao redor já roda. O trilho completo, da taxa do marketplace ao harvest, à tesouraria, ao buyback e à queima, com uma asserção de supply no fim, é todo seu, solo.

## Onde o dinheiro realmente está

### Não existe um caixa atrás do pedágio

Imagine um mercado municipal onde cada banca paga 1% ao mercado. Você esperaria um caixa perto da porta. O Token-2022 não funciona assim. Quando uma transferência dispara, o programa tira a taxa do valor transferido e a estaciona num potinho trancado em cima da *própria mesa do destinatário*, dentro da conta de token dele, num slot chamado `TransferFeeAmount.withheldAmount`. O comprador não pode gastar. Você também não pode gastar, não de onde você está. Dez mil trocas significam dez mil potinhos espalhados por dez mil mesas, e nenhum deles é seu.

Alguém tem que percorrer o salão.

O que está em jogo para você é concreto e não é contabilidade abstrata: uma taxa da qual você nunca faz harvest é uma taxa que você nunca ganhou, receita que existe no papel e não financia nada. Todo buyback que você planeja, todo orçamento de ops, toda linha de "o protocolo se autofinancia" na sua documentação está a jusante de um cron job chato que ninguém é glamouroso o bastante para querer assumir.

Você construiu o mecanismo de percorrer o salão lá no módulo 2, na lição economics-extensions, e testou ele contra um único comprador. Hoje ele vira a primeira perna de um trilho com mais três pernas aparafusadas nele.

![Um fluxograma traça as taxas retidas desde as contas dos compradores, passando por um harvest sem permissão até o mint, um withdraw restrito à autoridade até o PDA da tesouraria, um buyback contra qualquer contraparte que você tenha, e uma queima que derruba o supply.](assets/v01-flowchart.png)

### Pernas um e dois: o crank de harvest (consolidar, depois arrecadar)

Duas instruções fazem o trabalho, e a divisão entre elas é um design de permissão, não um acidente.

`harvest_withheld_tokens_to_mint` recebe uma lista de contas de token de origem e varre os saldos retidos delas para o mint, para um campo `withheldAmount` que vive dentro do próprio `TransferFeeConfig` do mint. Esse é o campo que o `peek.ts` acabou de imprimir como zero. Ela também é sem permissão, o que significa que qualquer um pode chamá-la sobre as contas de qualquer um, e isso é seguro justamente porque consolidação não consegue roubar: os tokens só se movem de potes espalhados para um pote que uma única chave consegue abrir.

`withdraw_withheld_tokens_from_mint` então abre esse pote e manda a pilha para uma conta de token de destino. Essa é restrita pela `withdraw_withheld_authority` que você definiu na criação do mint. Existe também a rota direta, `withdraw_withheld_tokens_from_accounts`, que pula a escala no mint e puxa de uma lista nomeada de contas direto para o seu destino num único salto assinado pela autoridade. Duas pernas para arrecadação de rotina em escala, um salto para puxadas cirúrgicas.

```text
harvest_withheld_tokens_to_mint        accounts -> mint    PERMISSIONLESS
withdraw_withheld_tokens_from_mint     mint -> destination  withdraw_withheld_authority
withdraw_withheld_tokens_from_accounts accounts -> dest.    withdraw_withheld_authority
```

O seu destino é a tesouraria. Para a Overgrowth isso é um endereço derivado de programa, um PDA de tesouraria, cuja conta de token associada guarda SPROUT e cujo saldo em SOL financia o buyback. Um PDA em vez de uma chave quente porque a coisa que recebe receita de protocolo deveria ser um endereço sem chave privada, possuível por um programa e auditável por qualquer um com um explorer. Essa escolha não te custa nada hoje e te poupa da conversa em que um contratado de saída ainda tem a seed phrase da tesouraria. Uma nota de honestidade agora, para o código do lab não contradizer este parágrafo na sua cabeça: o lab carrega `treasury.json`, um keypair simples, e deixa ele assinar direto. Um PDA não pode morar num arquivo JSON e não pode assinar a não ser pelo `invoke_signed` do programa dele, e entregar o programa que seria dono do PDA de tesouraria de produção da Overgrowth está fora do escopo desta lição. Então o keypair `treasury` do lab é um dublê vestindo o papel do PDA: todo lugar em que ele assina é um lugar em que o programa do design de produção assinaria com seeds, e nada mais no trilho muda.

A fiação de autoridades ao redor disso merece trinta segundos, porque você a define uma vez na criação do mint e ela é quase permanente. `transfer_fee_config_authority` pode mudar a tabela de taxa. `withdraw_withheld_authority` pode arrecadar. São chaves separadas de propósito, e separá-las é a diferença entre um design de governança e um ponto único de falha: um multisig ou uma DAO pode segurar o poder de definir a alíquota enquanto uma chave de ops chata segura o poder de varrer, e rotacionar a chave de ops não toca na alíquota. Revogá-las também não é simétrico. Anule a autoridade de config e a sua tabela de taxa fica congelada para sempre, o que é um compromisso crível que holders conseguem verificar. Anule a autoridade de withdraw e toda taxa que o token um dia retiver, passada e futura, fica encalhada permanentemente: o harvest continua consolidando para o mint, e nada nunca mais consegue abrir aquele pote. Uma dessas revogações é uma promessa. A outra é uma lápide. No lab abaixo o signer da tesouraria segura a autoridade de withdraw, o que está ok para um fork e não é o que eu entregaria.

### O que custa rodar o crank

O crank tem um custo operacional e ele é seu para sempre. Alguém paga as taxas de transação, alguém percebe quando o cron morre, alguém decide se varrer 400 contas por semana é melhor do que varrer 40 contas por dia. Quase todo postmortem de token com taxa que eu li se resume a ninguém rodar o cron.

A boa notícia é que a metade da consolidação é barata. `harvest_withheld_tokens_to_mint` recebe um array `sources` inteiro, então uma instrução varre muitas contas, e na minha rodada de surfnet lá na lição de economia um harvest de origem única mediu por volta de 1,200 compute units. Consolidação ser quase de graça é exatamente o design certo para uma chamada que qualquer um pode fazer.

A restrição que de fato morde não é compute, é a transação. Toda conta de origem que você lista é mais uma chave de conta na mensagem, e a mensagem tem que caber em uma transação. Então o tamanho do seu lote é um problema de empacotamento: quantos endereços de conta cabem ao lado dos dados da instrução e das assinaturas. Descubra isso empiricamente para o seu próprio setup em vez de confiar num número de post de blog, porque address lookup tables, instruções extras e o seu fee payer todos mexem no teto. Depois quebre a saída da varredura em lotes desse tamanho e mande eles como transações separadas. Falha parcial é sobrevivível aqui de um jeito que raramente é: o harvest é idempotente no sentido que importa, já que uma conta com zero retido contribui com zero, então uma retentativa que reinclui uma conta já varrida é um no-op em vez de uma contagem dupla.

O que faz do próprio agrupamento em lotes umas seis linhas, e vale mantê-lo como uma função própria para que o tamanho do lote seja um número só que você pode ajustar depois de medir:

```ts
// chunk.ts: batch the scan's output so each harvest transaction fits the wire.
export function chunk<T>(items: T[], size: number): T[][] {
  if (size < 1) throw new Error("batch size must be at least 1");
  const out: T[][] = [];
  for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
  return out;
}
```

E achar as contas sujas também é problema seu. Uma conta de token coloca o mint dela no byte offset 0, então uma chamada de `getProgramAccounts` com um filtro memcmp te dá todo holder de SPROUT, e aí você lê o `TransferFeeAmount` de cada uma e fica com as diferentes de zero. Num fork com algumas dezenas de holders isso é uma varredura de dois segundos. Em contagens de holders do tamanho da Jupiter é um trabalho de indexação, `getProgramAccounts` sobre um programa grande é exatamente a query que os RPCs públicos mais estrangulam, e a resposta honesta é que você aluga isso, do mesmo jeito que a lição de leitura de assets te fez alugar um provedor de DAS em vez de rodar o seu próprio indexador.

![Uma tabela compara os quatro custos de rodar um crank de harvest de taxas retidas, do compute barato passando por limites de empacotamento e varreduras pesadas de contas até a propriedade operacional que causa a maioria das falhas.](assets/v02-comparison.png)

### Três modelos de taxa, e a semana que você perde confundindo eles

É aqui que eu vi gente competente queimar dias.

A pump.fun também tem taxas de criador. Elas não são taxas retidas do Token-2022, não usam `TransferFeeAmount`, e nenhuma quantidade de harvest vai encontrá-las. As taxas de criador da pump são do lado do programa: o programa as roteia para um PDA `creator_vault` derivado por criador, e o criador faz claim a partir daquele vault. Mecanismo diferente, conta diferente, caminho de claim diferente, as mesmas três palavras em inglês no pitch deck.

O resto da maquinaria de taxas da pump vale conhecer com precisão, porque é a coisa mais próxima de uma implementação de referência de política de taxas que o ecossistema tem. As taxas rodaram fixas em 100 basis points durante toda a era inicial. Então, em 2025-09-01 20:00 UTC, elas viraram uma tabela escalonada por market cap: o seu tier de taxa agora depende de onde a sua moeda está, o que significa que a taxa é uma política que se mexe embaixo de você em vez de uma constante que você configurou. As moedas de Cashback invertem a direção inteira, redirecionando a taxa de criador de volta para os traders através de PDAs acumuladores de volume. As taxas de protocolo rodam entre 8 destinatários de taxa para que as contas de arrecadação não virem uma única escrita quente e disputada. E o compartilhamento de taxa suporta até 10 acionistas, então o "criador" da taxa de criador pode ser um cap table.

Leia o que o dia da virada significa em vez de só arquivar a data. Antes dele, um criador lançando na pump sabia o número: 100 basis points, o mesmo para todo mundo, o mesmo no mês seguinte. Depois dele, a taxa que uma moeda paga é função de onde aquela moeda é negociada, o que é uma variável que o criador não define e não consegue congelar. Isso não é uma crítica à pump, cuja tabela é publicada e cujo raciocínio é defensável. É o formato geral de lançar no trilho de outra pessoa: você herda a política econômica dela, incluindo a versão dela que ela entrega depois que você lança. A sua própria taxa de Token-2022 é a troca oposta. Você é dono da alíquota, pode torná-la permanente anulando a autoridade de config, e em troca você é dono do harvest, da indexação, do cron e de toda integração que quebra porque valor enviado não é mais igual a valor recebido. Nenhum dos dois lados dessa troca é de graça. Escolha aquele por cujos custos você prefere ser responsável.

![Uma linha do tempo move as taxas da pump.fun de uma era fixa de 100 basis points para a tabela escalonada por market cap de 2025-09-01 e de lá para os redirecionamentos de Cashback, com a mecânica de vault permanente ao longo de todo o caminho.](assets/v03-timeline.png)

Agora o contraexemplo, que é o meu objeto favorito deste curso inteiro. Em maio de 2024, a PayPal e a Paxos entregaram o PYUSD como o mint Token-2022 emblemático em formato de compliance. Ele carrega uma config de taxa de transferência. Essa config está definida em 0 basis points, e ela nunca disparou. O token com capacidade de taxa mais institucionalmente sério da Solana não arrecada nada, de propósito, porque o que os emissores dele queriam era a *opção*, armada e dormente, disponível no dia em que um regulador ou um modelo de negócio pedir. Configurado não é a mesma coisa que ativo. Você já leu essa mesma distinção de um mint ao vivo com `decode-mint`, e este é o exemplo de maior aposta disso.

![Uma tabela comparativa separa as taxas de transferência retidas do Token-2022, as taxas de creator_vault do lado do programa da pump.fun e a config de taxa dormente de zero bps do PYUSD por ponto de acúmulo, quem move, alíquota e pegadinhas.](assets/v04-comparison.png)

A regra prática: antes de escrever uma única linha de código de arrecadação, leia as extensões do mint e descubra qual máquina você está olhando. Se `TransferFeeConfig` estiver presente com bps diferente de zero, harvest se aplica. Se as taxas forem do lado do programa, vá achar o vault do programa e a instrução de claim dele. Modelo errado, semana errada.

### O split é uma lei de conservação

O harvest deposita uma pilha na tesouraria. Agora você decide o que acontece com ela, e esta é a parte em que aritmética desleixada silenciosamente cunha ou destrói tokens na sua contabilidade.

A política da Overgrowth: uma fatia de todo harvest queima imediatamente, o resto financia as operações. Números redondos, caminhados um passo de cada vez. Você faz harvest de 1,000,000 unidades base de SPROUT. A fatia de queima é 20%, então 200,000 unidades queimam na chegada e 800,000 ficam na tesouraria. 200,000 mais 800,000 é 1,000,000. Isso não é uma coincidência pela qual você deveria ser grato, é o invariante que o seu código tem que sustentar em toda entrada, inclusive as feias: `burnedFromFees + toTreasury === harvested`. O split não cria nada e não destrói nada. Ele só rotula.

O buyback é uma perna completamente separada e é financiado por um ativo diferente. A tesouraria também guarda SOL, de taxas de listagem do marketplace, do lançamento, de onde quer que a sua receita realmente venha. Dimensionar o buyback é uma única divisão com piso: quantas unidades base de SPROUT aquele SOL compra ao preço atual do venue? 5,000,000,000 lamports a 1,000,000 lamports por unidade base compram 5,000 unidades. Aplique o piso em bigints, sempre, porque você não pode comprar uma fração de unidade base e o resto que você arredonda fora é exatamente o tipo de deriva silenciosa que aparece numa asserção de supply seis semanas depois, quando ninguém lembra de ter escrito a linha.

A cilada que eu quero que você nomeie em voz alta antes de escrever a função: a queima de taxa e a queima de buyback não precisam ser iguais, e nada está errado quando não são. São dois fluxos independentes para a mesma fornalha. Um é denominado em SPROUT que você já tinha, o outro em SOL que você converteu. Conservação vale dentro do split, não entre as duas pernas.

![Um diagrama divide um harvest de 1,000,000 unidades em uma queima de 200,000 e uma fatia de 800,000 para a tesouraria ao lado de um buyback separado financiado em SOL, com os dois fluxos convergindo para uma única queima.](assets/v05-diagram.png)

### Perna três: o buyback é um swap, e swaps custam dinheiro

Checkpoint rápido sobre como chegamos aqui, porque a próxima parte introduz um segundo ativo e um segundo cliente. Primeiro: as taxas existem mas ficam retidas nas contas dos destinatários, então saldo de tesouraria não é evidência de receita. Segundo: as pernas um e dois são um crank de duas instruções, sem permissão para consolidar e restrito à autoridade para arrecadar, e rodar isso é um trabalho operacional com um dono. Terceiro: o split de um harvest é uma lei de conservação, e é aritmética entre pernas, não uma perna própria; o buyback que ele dimensiona é financiado separadamente, a partir de SOL.

Agora a parte que é fácil de descrever e fácil de errar emocionalmente.

Um buyback com queima não é uma funcionalidade de protocolo que você habilita. É você, segurando SOL, entrando num mercado aberto, comprando o seu próprio token pelo que o mercado cobrar de você hoje, e depois destruindo o que você comprou. O venue que o m08-l2 escolheu para o SPROUT é o Meteora DBC graduando para uma pool DAMM v2, e essa escolha não é uma preferência, é o resíduo de toda decisão a montante: pump e LaunchLab fixam SPL clássico nas instruções de create deles e nunca conseguiriam segurar SPROUT, o DBC aceitou o mint Token-2022, e a migração do DBC cai na família DAMM, com a v2 sendo o lado que carrega um mint base Token-2022. O programa DAMM v2 é `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG`, e no seu fork de mainnet ele é a coisa real, estado forkado e tudo.

A fronteira, resolvida antes do lab em vez de descoberta dentro dele. **Este curso nunca lança SPROUT.** O m08-l1 derivou um limiar no papel, o m08-l2 escolheu um venue no papel, e nenhum dos dois levou uma config de DBC até `migration_quote_threshold` nem migrou coisa nenhuma, porque um lançamento é um evento de distribuição e este curso ensina a camada de token. Então, a menos que você tenha ido lá e lançado SPROUT você mesmo, o seu fork não carrega nenhuma pool de SPROUT, e a perna 3 não tem mercado contra o qual negociar. Isso não é uma lacuna que o lab disfarça; é uma bifurcação no caminho com duas portas honestas, e o passo 1c abaixo é onde você escolhe uma. Também significa que a flag de documentado-mas-não-verificado da lição de launchpad sobre o suporte do DBC a base Token-2022 continua aberta: nada nesta lição resolve isso, e um aluno que de fato lançar SPROUT no DBC e voltar com o relato é quem fecha isso.

A única matemática de AMM que esta lição usa é uma frase: o preço spot da pool é a razão entre as duas reservas de vault dela, então reserva de quote dividida por reserva de base te dá lamports por unidade base de SPROUT, e esse número é por quem você divide o SOL da sua tesouraria para dimensionar a compra. O DAMM v2 também cota esse mesmo preço nativamente como uma raiz quadrada em ponto fixo, e a distância entre as duas visões, uma vez que posições concentradas a abram, pertence ao curso de DeFi junto com o resto da matemática de pool. Esse é o conteúdo matemático inteiro, e são quatro linhas que você pode rodar agora mesmo:

```ts
// sizing, standalone: what does the treasury's SOL buy at the pool's current price?
const treasurySol = 5_000_000_000n; // lamports the treasury is willing to spend
const price = 1_000_000n; // lamports per SPROUT base unit = quoteReserve / baseReserve
const buyback = treasurySol / price;
console.log(`${buyback} SPROUT base units at spot, before slippage`); // 5000
```

Repare nas últimas três palavras daquela linha de log. Tudo daqui para frente na lição é sobre a distância entre "at spot" e o que de fato chega na sua conta.

Composição de pool, roteamento entre venues, matemática de tick e bin, estratégia de LP, e tudo mais que faz um swap ser eficiente em vez de meramente possível é material do curso planejado DeFi and RWA Engineering. Eu não vou te dar uma versão rasa de um assunto que tem uma casa própria.

O que você *de fato* precisa de mim é a lista honesta de custos, porque um buyback parece deflação de graça e não é.

Você paga slippage, porque a sua própria compra move o preço contra você, então o SPROUT que você recebe é menos do que a aritmética de preço spot prometeu, e numa pool rasa com uma ordem grande de tesouraria é bem menos. Você paga a taxa do venue por cima disso. Você está exposto a MEV: um buyback é uma ordem a mercado grande, previsível e anunciada publicamente, que é mais ou menos o formato ideal de um alvo de sanduíche, e as táticas de aterrissagem do lado do cliente que mitigam isso são território do curso planejado Client-Side Mastery, não desta lição. E existe uma pegadinha específica do Token-2022 que pega todo mundo na primeira vez: o SPROUT cobra uma taxa de transferência em *toda* transferência, inclusive aquela em que a sua contraparte manda SPROUT para a sua tesouraria. O seu buyback paga a sua própria taxa. O valor retido cai de volta na própria conta de token da tesouraria, esperando o próximo harvest. É circular e inofensivo e vai absolutamente fazer a sua aritmética discordar de si mesma se você calcular o que comprou em vez de medir.

Três desses quatro custos precisam de um mercado para existir. O quarto, a sua própria taxa de transferência, dispara em qualquer transferência que seja, e é por isso que ele é o único componente que o lab consegue colocar na sua frente independentemente da porta que você escolher.

Então meça. Leia o saldo da tesouraria antes do swap, leia depois, e queime a diferença. Toda outra abordagem é você afirmando o que a chain deveria ter feito.

O que também é por que o buyback é uma questão de política e não uma chave que você aciona. Quanto, com que frequência e com que previsibilidade são três botões, e mexer em qualquer um deles troca um custo por outro.

![Uma tabela de decisão pesa políticas de buyback mensal-grande, contínuo-pequeno e oportunista contra impacto no preço, custo do crank e previsibilidade para MEV.](assets/v06-table.png)

Existe uma pergunta anterior escondida aqui, e você já a respondeu. Um venue só aceita o seu token se o seu conjunto de extensões for um que ele tolera, que é o trabalho de roteabilidade que você fez na lição designing-a-routable-token. Um delegado permanente ou um transfer hook que a allowlist da pool rejeita significa que não há venue e portanto não há buyback. As decisões de extensão que você tomou no módulo 5 são o que torna o módulo 9 possível.

### Perna quatro: a queima, e a leitura desatualizada que anula a sua asserção

A última perna é `burn_checked` na conta de token da tesouraria, assinada pela autoridade da tesouraria, e é a única instrução deste trilho inteiro que reduz o supply. Harvest não queima. Withdraw não queima. Mandar tokens para um endereço morto também não queima, não importa o que o seu dashboard favorito alegue: aqueles tokens continuam existindo e continuam contando no `supply`.

Três coisas são chamadas de deflacionárias e só uma delas é. Uma queima destrói tokens e decrementa o campo `supply` do mint, que é um número on-chain verificável que qualquer um consegue ler da mesma conta de mint que você vem decodificando o curso inteiro. Um endereço de queima é uma carteira da qual ninguém tem a chave, o que remove tokens de circulação na prática e de nada nos dados: `supply` não se move, e qualquer número de supply circulante construído em cima disso é uma convenção e não um fato. Revogar a autoridade de mint limita a emissão futura e não destrói absolutamente nada. Diga qual das três você está fazendo, com essas palavras, em qualquer coisa que você publicar. Vale preferir a variante `burn_checked` ao `burn` simples pelo mesmo motivo que `transfer_checked` ganhou do `transfer`: ela te obriga a passar o mint e os decimals, e o programa recusa se eles discordarem da conta. Um erro de decimals numa queima é irrecuperável de um jeito que um erro de decimals numa transferência normalmente não é.

A cilada é a leitura, não a escrita. Se você buscar o mint, depois queimar, e depois reportar a partir do objeto que buscou antes, você vai reportar o supply antigo e a sua asserção vai passar ou falhar por motivos que não têm nada a ver com o seu código. Qualquer coisa que você decodificou antes de uma transação é uma fotografia, não um feed ao vivo. Busque o mint de novo depois que a queima confirmar. O equivalente disso em Anchor é chamar `.reload()` depois de uma CPI que tocou a sua conta, e o modo de falha é idêntico nos dois mundos.

![Seis linhas de código anotadas caminham de uma busca de supply pré-queima, passando por harvest, compra e queima, até uma rebusca obrigatória e a asserção de que o supply caiu exatamente o valor queimado.](assets/v07-annotated-code.png)

## Lab: ligue o trilho de taxas da Overgrowth

Você está construindo o `sprout-economy`, o degrau que transforma o SPROUT de um token com taxa em um token com economia. Ele consome duas coisas que você já tem: `sprout-mint` do módulo 2, que é onde a config de taxa mora, e `sprout-launch` das lições de lançamento, cuja decisão de venue é o motivo de o SPROUT graduar no Meteora DBC para uma pool DAMM v2 em vez de num launchpad de SPL clássico que não consegue segurar o mint dele. Quatro módulos de trabalho convergem em um script.

Rode contra o surfpool, forkado da mainnet, para que o programa DAMM v2 e as contas dele sejam reais. Se o surfpool ainda não estiver rodando dos labs anteriores, `surfpool start --no-tui --no-studio` em outro terminal é toda a cerimônia (instalação: `brew install txtx/taps/surfpool`, ou `cargo install surfpool-cli`; verificado no 1.2.1).

**1. Fixe a toolchain.** Duas linhas, e a segunda precisa de uma palavra de honestidade.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install @meteora-ag/cp-amm-sdk@1.4.6 @solana/web3.js@1.98.4 bn.js@5.2.2
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node @types/bn.js
```

Conferido contra o npm em 2026-09-05: a tag `latest` do kit é 8.2.0, publicada em 2026-08-29, mas a primeira linha fixa por faixa de peer, não por latest: `@solana-program/token-2022@0.15.0` é o minor atual que faz peer com o kit `^7.0.0` — o release 0.16.0 pulou para `^8` — então o kit fica no 7.1.1, o release mais novo dentro dessa faixa, e `@solana-program/system@0.13.0` combina com ele. Rode `npm view @solana-program/token-2022@0.15.0 peerDependencies` de novo quando você fizer o scaffold; essa matriz se mexe todo mês.

A segunda linha é a interessante, e ela só é necessária se você pegar a porta A abaixo. `@meteora-ag/cp-amm-sdk` é o cliente DAMM v2 de primeira parte da Meteora e já vem com tipos do web3.js v1, não do kit. Você vai rodar dois clientes em um workspace, e esse não é um erro que eu esteja escondendo de você: é como integrar com um SDK de primeira parte realmente se parece em 2026. O kit faz as pernas de Token-2022 porque é onde o kit é excelente. O web3.js v1 faz a perna do swap porque é o que o SDK do próprio venue fala. O pin 1.4.6 é uma leitura do npm de 2026-08-21, apropriado para um SDK cuja maquinaria mais profunda este curso repassa em vez de ensinar; rode `npm view @meteora-ag/cp-amm-sdk version` no dia em que você fizer o scaffold.

Nomeie a regra em que esse arranjo anda, porque o m05-l1 a enunciou e este lab é onde ela é testada: a Meteora não publica nenhuma superfície kit para o DAMM v2, então a dependência de v1 é inevitável, e ela fica em quarentena em **exatamente um arquivo**, `venue.ts`. Nada mais no lab importa web3.js, nem indiretamente — `venue.ts` constrói a própria `Connection`, assina com o próprio `Keypair`, e entrega ao resto do trilho valores simples. É a quarentena funcionando: as duas stacks se encontram na chain, não numa lista de imports compartilhada. Se você se pegar buscando uma `PublicKey` em `wire-economy.ts`, a emenda vazou e a correção é empurrar o vazamento de volta para dentro de `venue.ts`.

**1b. Crie `treasury.json`, a chave com que o trilho inteiro assina.** Nenhum módulo anterior criou este arquivo, e essa é uma lacuna para fechar agora em vez de descobrir no passo 6: os scripts do m02 estacionaram toda autoridade em signers descartáveis em memória, ótimo para mints descartáveis e inútil para um trilho cuja perna de withdraw tem que ser assinável semana que vem. Gere a chave uma vez e financie ela no fork:

```bash
solana-keygen new --no-bip39-passphrase -o labs/m09-l1/treasury.json
solana airdrop 100 "$(solana-keygen pubkey labs/m09-l1/treasury.json)" --url http://127.0.0.1:8899
```

Depois faça a chain concordar que essa chave segura os poderes que o trilho exerce. As autoridades de taxa são definidas na criação do mint, e as chaves descartáveis que as seguravam em qualquer SPROUT mais antigo morreram junto com o processo delas — então este é exatamente o caso do "re-cunhe seguindo a abertura do m05-l1" das premissas vigentes, com uma edição antes. Abra o builder do SPROUT composto (`labs/m02-l1/verify-economics.ts`, conforme re-apontado no passo 7 do m02-l4), carregue o signer da tesouraria no topo com as mesmas duas linhas de `createKeyPairSignerFromBytes` que o `wire-economy.ts` usa abaixo, e passe esse signer no lugar do descartável para exatamente dois papéis: a autoridade de mint e a `withdrawWithheldAuthority` da config de taxa. Re-cunhe, rode de novo as suas transferências de marketplace, e o fork agora carrega um SPROUT cujo pote de taxa este arquivo consegue abrir — e cujo supply a janela de conversão da próxima lição consegue cunhar, que é por que a autoridade de mint vai para a mesma chave. Um eco da seção de teoria, para o código não a contradizer na sua cabeça: produção quer um PDA neste papel, não um arquivo JSON; todo lugar em que essa chave assina é um lugar em que o seu programa faria `invoke_signed`.

Você não precisa abrir a conta de token de SPROUT da tesouraria na mão, e a assimetria com o `--fund-recipient` do passo 1c é deliberada e não um descuido: o `wire-economy.ts` a cria sozinho, de forma idempotente, como a primeira instrução da transação de harvest. Isso é de propósito. A conta do maker é montada uma vez por um humano antes de o trilho existir, enquanto a da tesouraria é uma pré-condição da primeira escrita do próprio trilho, então o trilho é dono dela e uma re-execução a frio não pode te deixar com um endereço derivado que nunca foi criado.

**1c. Escolha a porta da perna 3, e levante a contraparte dela.** O buyback precisa de alguém de quem comprar. Duas portas, e o trilho a jusante não consegue diferenciá-las, que é justamente o ponto.

*Porta A, o mercado.* Você lançou SPROUT você mesmo no Meteora DBC, empurrou a curva para além do seu `migration_quote_threshold`, e guardou o endereço da pool DAMM v2 que a migração imprimiu. Nada neste curso te conduz por isso, e eu não vou fingir o contrário. Se você tem essa pool, defina `SPROUT_POOL` e a perna 3 é um swap de verdade com slippage de verdade contra estado forkado de verdade.

*Porta B, o maker.* Você não tem, porque você seguiu o curso. O SPROUT não tem mercado, então você levanta um: uma única contraparte segurando SPROUT, disposta a vender a um preço que você define. Isto não é um mercado e o lab não vai chamar de mercado. O que ele preserva é toda propriedade do trilho que não depende de descoberta de preço — o harvest, a conservação do split, a taxa disparando no pagamento, medir em vez de calcular, e a asserção de supply no fim. O que ele deixa de fora é slippage e MEV, que precisam de um book contra o qual existir.

```bash
solana-keygen new --no-bip39-passphrase -o labs/m09-l1/maker.json
solana airdrop 10 "$(solana-keygen pubkey labs/m09-l1/maker.json)" --url http://127.0.0.1:8899
```

Depois dê ao maker alguma coisa para vender. A sua chave de tesouraria segura a autoridade de mint depois do passo 1b, então um `spl-token mint` coloca estoque nos livros do maker; dimensione bem acima do que quer que a tesouraria vá gastar, ou a perna 3 falha no saldo do vendedor em vez de falhar em qualquer coisa que você estivesse tentando aprender:

```bash
spl-token mint "$SPROUT_MINT" 1000 \
  --recipient-owner "$(solana-keygen pubkey labs/m09-l1/maker.json)" \
  --mint-authority labs/m09-l1/treasury.json \
  --fund-recipient --url http://127.0.0.1:8899
```

Diga a parte silenciosa antes de rodar o trilho: um preço de buyback que você mesmo define não é um preço de mercado. Num venue, o número é a razão entre as reservas e o mercado te entrega ele. Aqui você é os dois lados da troca, então o preço é uma decisão de governança que só parece uma constante, e toda conclusão que você tira de uma rodada pela porta B herda isso. O módulo venue do passo 3 existe para que, no dia em que o SPROUT de fato tiver uma pool, a única linha que muda seja qual porta você abriu.

**2. Ache a pilha.** Crie `find-withheld.ts`. Esta é a varredura de contas, e é a ferramenta sobre a qual o resto do trilho é construído.

```ts
// find-withheld.ts: which SPROUT accounts are sitting on withheld marketplace fees?
import type {
  Address,
  Base58EncodedBytes,
  GetMultipleAccountsApi,
  GetProgramAccountsApi,
  Rpc,
} from "@solana/kit";
import { TOKEN_2022_PROGRAM_ADDRESS, fetchAllMaybeToken } from "@solana-program/token-2022";

export type DirtyAccount = { account: Address; withheld: bigint };

/** Every token account of `mint` carrying a nonzero TransferFeeAmount.withheldAmount. */
export async function findWithheld(
  rpc: Rpc<GetProgramAccountsApi & GetMultipleAccountsApi>,
  mint: Address,
): Promise<DirtyAccount[]> {
  // Token account layout puts the mint at offset 0, so one memcmp finds every holder.
  const holders = await rpc
    .getProgramAccounts(TOKEN_2022_PROGRAM_ADDRESS, {
      encoding: "base64",
      withContext: false,
      dataSlice: { offset: 0, length: 0 },
      filters: [
        { memcmp: { offset: 0n, bytes: mint as string as Base58EncodedBytes, encoding: "base58" } },
      ],
    })
    .send();

  const decoded = await fetchAllMaybeToken(
    rpc,
    holders.map((h) => h.pubkey),
  );

  const dirty: DirtyAccount[] = [];
  for (const account of decoded) {
    if (!account.exists) continue;
    const extensions = account.data.extensions;
    if (extensions.__option !== "Some") continue;
    for (const ext of extensions.value) {
      if (ext.__kind === "TransferFeeAmount" && ext.withheldAmount > 0n) {
        dirty.push({ account: account.address, withheld: ext.withheldAmount });
      }
    }
  }
  return dirty;
}
```

O `dataSlice: { offset: 0, length: 0 }` importa mais do que parece. A varredura precisa de endereços, não de dados, então você pede ao RPC zero bytes por conta e depois busca e decodifica só o que encontrou. Num RPC público com um conjunto grande de holders, a versão que puxa os dados completos de conta para todo holder é a versão que te deixa com rate limit.

**3. O venue da porta A.** Crie `venue.ts`. Esta é a perna do swap, e ela é pequena porque o SDK está fazendo o trabalho. Ela também é toda a superfície de web3.js do lab: a `Connection`, o `Keypair`, as `PublicKey`s e o envio moram todos aqui dentro e nunca escapam.

```ts
// venue.ts: SPROUT's graduation venue, read and traded client-side.
// web3.js v1 here on purpose AND NOWHERE ELSE: the first-party Meteora SDK
// ships v1 types, so this file is the quarantine. It takes strings and bytes,
// it returns bigints and strings, and nothing v1-shaped crosses its boundary.
import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";
import { CpAmm, CP_AMM_PROGRAM_ID } from "@meteora-ag/cp-amm-sdk";

export const DAMM_V2_PROGRAM = CP_AMM_PROGRAM_ID.toBase58();

// The two token programs the pool straddles: SPROUT is Token-2022, wSOL is classic.
const TOKEN_2022 = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const TOKEN_CLASSIC = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

export type Venue = {
  /** lamports of quote per one base unit of SPROUT, floored */
  priceLamportsPerToken: bigint;
  /** Sends the buy and returns only once it has CONFIRMED. Measuring is the caller's job. */
  buy: (quoteLamports: bigint, slippagePct: number) => Promise<string>;
};

/**
 * Open a DAMM v2 pool on the fork and expose a client-side buy.
 * Throws if the pool is absent, because guessing which pool you meant is not
 * this module's job: SPROUT only has one if you launched and migrated it.
 */
export async function openVenue(
  rpcUrl: string,
  baseMint: string,
  pool: string,
  buyerSecret: Uint8Array,
): Promise<Venue> {
  const connection = new Connection(rpcUrl, "confirmed");
  const buyer = Keypair.fromSecretKey(buyerSecret);
  const poolKey = new PublicKey(pool);
  const baseKey = new PublicKey(baseMint);

  if ((await connection.getAccountInfo(poolKey)) === null) {
    throw new Error(`no DAMM v2 pool at ${pool} for ${baseMint}: this fork has no market for that mint`);
  }

  const cpAmm = new CpAmm(connection);
  const state = await cpAmm.fetchPoolState(poolKey);
  if (!state.tokenAMint.equals(baseKey)) {
    throw new Error(`pool ${pool} does not carry ${baseMint} as its base mint: wrong pool`);
  }

  // The only AMM math this course does: spot price is the vault-reserve ratio.
  const base = BigInt((await connection.getTokenAccountBalance(state.tokenAVault)).value.amount);
  const quote = BigInt((await connection.getTokenAccountBalance(state.tokenBVault)).value.amount);
  const priceLamportsPerToken = base === 0n ? 0n : quote / base;

  return {
    priceLamportsPerToken,
    buy: async (quoteLamports: bigint, slippagePct: number) => {
      // The SDK's slippage dial is minimumAmountOut: the spot-sized fill, shaved by your tolerance.
      const atSpot = priceLamportsPerToken === 0n ? 0n : quoteLamports / priceLamportsPerToken;
      const minOut = (atSpot * BigInt(100 - slippagePct)) / 100n;
      const tx = await cpAmm.swap({
        payer: buyer.publicKey,
        pool: poolKey,
        inputTokenMint: state.tokenBMint, // quote (wSOL) in; the builder handles the wrap
        outputTokenMint: state.tokenAMint, // SPROUT out
        amountIn: new BN(quoteLamports.toString()),
        minimumAmountOut: new BN(minOut.toString()),
        tokenAMint: state.tokenAMint,
        tokenBMint: state.tokenBMint,
        tokenAVault: state.tokenAVault,
        tokenBVault: state.tokenBVault,
        tokenAProgram: TOKEN_2022,
        tokenBProgram: TOKEN_CLASSIC,
        referralTokenAccount: null,
      });
      // v1's sendTransaction returns at SUBMISSION, not confirmation. Read the
      // treasury balance before this settles and you measure a pre-swap
      // photograph (the burn section's stale-read trap, client-side edition),
      // so the confirm belongs in here rather than in the caller's hopes.
      const swap = new Transaction().add(...tx.instructions);
      const signature = await connection.sendTransaction(swap, [buyer], { skipPreflight: false });
      const latest = await connection.getLatestBlockhash("confirmed");
      await connection.confirmTransaction({ signature, ...latest }, "confirmed");
      return signature;
    },
  };
}
```

Confira as duas asserções no topo de `openVenue`. Se a pool não estiver lá, a primeira lança em vez de adivinhar. Se a pool existir mas carregar algum outro mint base, a segunda lança antes de você negociar no mercado de outra pessoa. Uma ferramenta que falha alto na fronteira da própria responsabilidade vale dez que devolvem `undefined` e deixam a falha aparecer três funções depois. Leitores da porta B ainda deveriam provar que aquela fronteira é real em vez de acreditar na minha palavra, e isso custa um comando sem precisar de pool nenhuma:

```bash
SPROUT_MINT=<mint> npx tsx -e "import {openVenue} from './venue'; \
  await openVenue('http://127.0.0.1:8899', process.env.SPROUT_MINT!, '11111111111111111111111111111112', new Uint8Array(64));"
```

Aquele endereço é uma conta real e não uma pool DAMM v2, então você recebe o primeiro throw, com nome e tudo, em menos de um segundo. O módulo é inerte até o SPROUT ter um mercado; ele não está quebrado.

**3b. A contraparte da porta B.** Crie `maker.ts`. Mesma interface, sem SDK de fornecedor, sem web3.js — este é de primeira parte e portanto kit até o fim.

```ts
// maker.ts: leg 3's counterparty when the token has no market.
// The price is a POLICY input, not a reserve ratio. Everything else about the
// leg is identical to the venue's: SOL out, SPROUT in, the fee fires on the
// payout, and the caller measures what landed instead of computing it.
import type { Address, Instruction, TransactionSigner } from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";
import {
  findAssociatedTokenPda,
  getTransferCheckedInstruction,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";

export type Maker = {
  priceLamportsPerToken: bigint;
  buyIxs: (quoteLamports: bigint) => Instruction[];
};

export async function openMaker(
  mint: Address,
  decimals: number,
  maker: TransactionSigner,
  buyer: TransactionSigner,
  priceLamportsPerToken: bigint,
): Promise<Maker> {
  if (priceLamportsPerToken <= 0n) {
    throw new Error("MAKER_PRICE must be a positive lamport price per base unit");
  }
  const [makerAta] = await findAssociatedTokenPda({
    mint,
    owner: maker.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });
  const [buyerAta] = await findAssociatedTokenPda({
    mint,
    owner: buyer.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  return {
    priceLamportsPerToken,
    // Both legs of the trade in ONE transaction, so neither side can take the
    // money and walk. Two signers, one atomic settlement: this is the smallest
    // honest OTC trade you can write, and it is what an escrow program
    // automates when the counterparty is a stranger rather than your own key.
    buyIxs: (quoteLamports: bigint) => [
      getTransferSolInstruction({
        source: buyer,
        destination: maker.address,
        amount: quoteLamports,
      }),
      getTransferCheckedInstruction({
        source: makerAta,
        mint,
        destination: buyerAta,
        authority: maker,
        amount: quoteLamports / priceLamportsPerToken,
        decimals,
      }),
    ],
  };
}
```

A divisão com piso em `amount` é a mesma que `routeFees` faz, pelo mesmo motivo: você não pode comprar uma fração de unidade base. E repare no que este arquivo NÃO faz. Ele não calcula o que a tesouraria vai receber. Ele transfere `quoteLamports / price` unidades, a taxa de transferência é descontada disso em voo, e o que chega é menor. Ninguém aqui afirma o quanto.

**4. O split, e este é seu.** Crie `route-fees.ts` com a assinatura abaixo. O corpo tem dois TODOs e a seção de teoria já te deu as duas respostas em palavras simples.

```ts
// route-fees.ts: split the harvest, size the buyback. Pure arithmetic, no chain.
export function routeFees(
  harvested: bigint,
  burnBps: number,
  treasurySol: bigint,
  priceLamportsPerToken: bigint,
): { burnedFromFees: bigint; toTreasury: bigint; buyback: bigint } {
  const burnedFromFees = (harvested * BigInt(burnBps)) / 10000n;
  // TODO: what stays in the treasury, such that the two shares sum back to `harvested`
  const toTreasury = 0n;
  // TODO: how many SPROUT base units does `treasurySol` buy at this price, floored
  const buyback = 0n;
  return { burnedFromFees, toTreasury, buyback };
}
```

Mantenha tudo em bigints. No momento em que um `Number` toca uma contagem de lamports acima de 2^53 a sua aritmética começa a mentir baixinho, e contagens de lamports chegam lá mais rápido do que você pensa.

**5. O trilho.** Crie `wire-economy.ts`. Este é o artefato, e ele se lê de cima a baixo como as quatro pernas em ordem.

```ts
// wire-economy.ts: SPROUT's fee rail, end to end, against the mainnet fork.
// Kit only. The one web3.js dependency in this lab lives behind venue.ts.
import {
  address,
  appendTransactionMessageInstructions,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  fetchMint,
  fetchToken,
  getBurnCheckedInstruction,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getHarvestWithheldTokensToMintInstruction,
  getWithdrawWithheldTokensFromMintInstruction,
  findAssociatedTokenPda,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { findWithheld } from "./find-withheld";
import { openMaker } from "./maker";
import { routeFees } from "./route-fees";
import { openVenue } from "./venue";

const RPC_HTTP = process.env.RPC_HTTP ?? "http://127.0.0.1:8899";
const RPC_WS = process.env.RPC_WS ?? "ws://127.0.0.1:8900";
const SPROUT = address(process.env.SPROUT_MINT!);
const SPROUT_DECIMALS = 6;
const SPROUT_POOL = process.env.SPROUT_POOL; // door A only: a DAMM v2 pool that holds SPROUT
const MAKER_KEY = process.env.MAKER_KEY; // door B: the counterparty from step 1c
const MAKER_PRICE = BigInt(process.env.MAKER_PRICE ?? "1000000"); // door B: lamports per base unit
const BURN_BPS = 2000; // 20% of every harvest burns on arrival
const SLIPPAGE_PCT = 1;

const rpc = createSolanaRpc(RPC_HTTP);
const rpcSubscriptions = createSolanaRpcSubscriptions(RPC_WS);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

function loadSecret(path: string): Uint8Array {
  return new Uint8Array(JSON.parse(readFileSync(path, "utf8")));
}

async function send(payer: KeyPairSigner, ixs: Instruction[]): Promise<void> {
  const { value: blockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
    (m) => appendTransactionMessageInstructions(ixs, m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirm(signed, { commitment: "confirmed" });
}

async function main(): Promise<void> {
  const secret = loadSecret(process.env.TREASURY_KEY!);
  const treasury = await createKeyPairSignerFromBytes(secret);
  const [treasuryAta] = await findAssociatedTokenPda({
    mint: SPROUT,
    owner: treasury.address,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const supplyBefore = (await fetchMint(rpc, SPROUT)).data.supply;

  // LEGS 1 + 2: harvest (permissionless consolidate) then withdraw
  // (authority-gated collect), one transaction. Fees are withheld on
  // recipient accounts until someone moves them.
  const dirty = await findWithheld(rpc, SPROUT);
  const harvested = dirty.reduce((sum, d) => sum + d.withheld, 0n);
  console.log(`dirty accounts: ${dirty.length}, withheld total: ${harvested}`);
  if (harvested === 0n) throw new Error("nothing withheld: run marketplace trades first");

  // Open the treasury's SPROUT account before anything tries to pay into it.
  // `findAssociatedTokenPda` above DERIVED an address; deriving is not creating,
  // and every leg from here down writes to this account: leg 2 names it as
  // `feeReceiver`, leg 3 has the counterparty transfer into it, leg 4 burns out
  // of it. Idempotent, so re-running the rail is free.
  const openTreasuryAta = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer: treasury,
    owner: treasury.address,
    mint: SPROUT,
  });

  await send(treasury, [
    openTreasuryAta,
    getHarvestWithheldTokensToMintInstruction({
      mint: SPROUT,
      // Fork scale: a handful of dirty accounts fits one transaction, so the
      // whole list rides in one instruction. At fleet scale this is where the
      // packing problem bites: wrap the list in the chunk() helper from the
      // "what the crank costs to run" section and send one harvest per batch.
      sources: dirty.map((d) => d.account),
    }),
    getWithdrawWithheldTokensFromMintInstruction({
      mint: SPROUT,
      feeReceiver: treasuryAta,
      withdrawWithheldAuthority: treasury,
    }),
  ]);

  // BETWEEN LEGS: get a price, then run the split. The split is arithmetic,
  // not a leg of its own; its output sizes and predicts leg 3.
  //
  // Door A reads the price off a real pool's reserves. Door B is told the
  // price, because with no market there is nothing to read it from. The rest
  // of this function cannot tell which one ran, which is the whole design.
  const treasurySol = (await rpc.getBalance(treasury.address).send()).value / 2n;
  let priceLamportsPerToken: bigint;
  let buy: () => Promise<void>;

  if (SPROUT_POOL) {
    const venue = await openVenue(RPC_HTTP, SPROUT, SPROUT_POOL, secret);
    priceLamportsPerToken = venue.priceLamportsPerToken;
    buy = async () => {
      await venue.buy(treasurySol, SLIPPAGE_PCT);
    };
    console.log(`door A: DAMM v2 pool ${SPROUT_POOL} at ${priceLamportsPerToken} lamports/unit`);
  } else {
    if (!MAKER_KEY) throw new Error("set SPROUT_POOL (door A) or MAKER_KEY (door B); step 1c picks one");
    const maker = await createKeyPairSignerFromBytes(loadSecret(MAKER_KEY));
    const otc = await openMaker(SPROUT, SPROUT_DECIMALS, maker, treasury, MAKER_PRICE);
    priceLamportsPerToken = otc.priceLamportsPerToken;
    buy = async () => {
      await send(treasury, otc.buyIxs(treasurySol));
    };
    console.log(
      `door B: no SPROUT market on this fork; buying from maker ${maker.address} ` +
        `at ${priceLamportsPerToken} lamports/unit (a price you set, not a price you read)`,
    );
  }

  const plan = routeFees(harvested, BURN_BPS, treasurySol, priceLamportsPerToken);
  if (plan.toTreasury === 0n || plan.buyback === 0n) {
    throw new Error(
      "route-fees.ts TODOs look unfilled: a zero treasury share or zero-sized buyback plan means the split never ran. Fill them before running the rail.",
    );
  }
  console.log(
    `split: burn ${plan.burnedFromFees} + keep ${plan.toTreasury} = ${harvested}; ` +
      `buyback target ~${plan.buyback} SPROUT at ${priceLamportsPerToken} lamports/unit`,
  );

  // LEG 3: the buyback. Whichever door opened, it is sized in SOL and it costs
  // you something. plan.buyback is the floor-division PREDICTION of what that
  // SOL buys; the planned-vs-bought line below is leg 3's cost made visible.
  const balanceBefore = (await fetchToken(rpc, treasuryAta)).data.amount;
  await buy();
  // Re-read the account AFTER the buy confirms. Cached balances are how supply math goes wrong.
  const balanceAfter = (await fetchToken(rpc, treasuryAta)).data.amount;
  const bought = balanceAfter - balanceBefore;
  console.log(`bought ${bought} SPROUT (planned ${plan.buyback}, the gap is what leg 3 cost you)`);

  // LEG 4: burn exactly what the buyback bought, plus the fee-burn share.
  const toBurn = bought + plan.burnedFromFees;
  await send(treasury, [
    getBurnCheckedInstruction({
      account: treasuryAta,
      mint: SPROUT,
      authority: treasury,
      amount: toBurn,
      decimals: 6,
    }),
  ]);

  const supplyAfter = (await fetchMint(rpc, SPROUT)).data.supply;
  const dropped = supplyBefore - supplyAfter;
  console.log(`supply ${supplyBefore} -> ${supplyAfter} (down ${dropped}, burned ${toBurn})`);
  if (dropped !== toBurn) throw new Error(`supply drop ${dropped} != burn ${toBurn}`);
  console.log("rail closed: harvested, split, bought back, burned");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

**6. Rode, com os TODOs de `route-fees.ts` preenchidos primeiro.** O trilho agora se recusa a rodar contra o stub (um split zerado lança antes de qualquer lamport se mover), então este passo assume que o passo 4 está pronto. Todo comando deste passo roda de dentro de `labs/m09-l1/`, que é onde os passos 1b a 5 colocaram os arquivos; os caminhos relativos de chave abaixo assumem isso.

```bash
cd labs/m09-l1
# door B, the one the course guarantees
SPROUT_MINT=<mint> MAKER_KEY=./maker.json TREASURY_KEY=./treasury.json npx tsx wire-economy.ts
# door A, if you launched SPROUT yourself and have its pool
SPROUT_MINT=<mint> SPROUT_POOL=<pool> TREASURY_KEY=./treasury.json npx tsx wire-economy.ts
```

Uma rodada saudável pela porta B diz algo próximo disto. O total retido e o saldo da tesouraria são seus, então só o *formato* se transfere; a aritmética entre as linhas é o que você confere.

```text
dirty accounts: 7, withheld total: 2500000
door B: no SPROUT market on this fork; buying from maker 8k2...Uq at 1000000 lamports/unit (a price you set, not a price you read)
split: burn 500000 + keep 2000000 = 2500000; buyback target ~50000 SPROUT at 1000000 lamports/unit
bought 49500 SPROUT (planned 50000, the gap is what leg 3 cost you)
supply 1000000000000000 -> 999999999450500 (down 549500, burned 549500)
rail closed: harvested, split, bought back, burned
```

Leia essas cinco linhas umas contra as outras, porque elas só concordam se o trilho funcionou. A linha do split soma: 500,000 mais 2,000,000 é 2,500,000, exatamente o que a varredura encontrou. O alvo do buyback é a metade gastável da tesouraria, cerca de 50 SOL depois do airdrop do passo 1b, dividida pelo preço. E a quarta linha é a honesta: você planejou 50,000 e 49,500 chegaram, porque o SPROUT cobra a própria taxa de 100-bps no pagamento do maker para você e 500 unidades ficaram para trás como retidas — na própria conta da sua tesouraria, esperando o próximo harvest, que é a circularidade sobre a qual a seção de teoria avisou, tornada visível. Na porta A aquela mesma linha carregaria também o slippage e a taxa do venue, e o número seria menor ainda. De um jeito ou de outro você pagou alguma coisa para comprar o seu próprio token de volta, que é o que um buyback sempre foi depois que você tira do termo o marketing dele.

![Um gráfico de duas barras coloca um buyback planejado contra a quantidade menor efetivamente recebida, atribuindo a diferença a impacto no preço, taxa do venue e a própria taxa de transferência do token.](assets/v08-chart.png)

Se a rodada lançar `supply drop != burn`, você quase certamente calculou `bought` em vez de medir, ou reusou o objeto de mint pré-queima. Os dois são o mesmo erro.

## Challenge

**Completion.** Preencha os dois TODOs em `route-fees.ts` e prove eles com o coding challenge do módulo, `route-fees`, que te entrega uma versão quebrada e uma suíte de testes. Uma convenção difere da sua cópia do projeto: o arquivo avaliado é standalone, então ele declara uma `function routeFees(harvested, burnBps, treasurySol, priceLamportsPerToken)` simples, sem a palavra-chave `export`, porque o grader enxerta o seu código no runner dele e chama a função direto com aqueles quatro argumentos posicionais, bigints para os três valores e um number simples para os basis points. Mantenha o `export` na cópia do projeto que o `wire-economy.ts` importa; tire ele no editor do challenge. O starter quebra a conservação, porque `toTreasury` ignora a fatia queimada, e dimensiona mal o buyback, porque multiplica pelo preço em vez de dividir. A sua versão tem que sustentar `burnedFromFees + toTreasury === harvested` para toda entrada, aplicar o piso ao buyback em bigints, deixar o harvest inteiro na tesouraria com uma fatia de queima de 0 bps, e levar `toTreasury` a zero em 10000 bps.

**Solo.** Ligue o trilho inteiro você mesmo contra o fork e prove. Gere volume de marketplace primeiro, pelo menos uma dúzia de transferências entre vários compradores para a varredura achar trabalho de verdade, depois rode `wire-economy.ts` de ponta a ponta e produza quatro números: o valor com harvest feito, o delta da tesouraria, a quantidade de buyback efetivamente recebida, e o delta de supply pós-queima. O critério é a asserção que já está no script: o supply caiu exatamente o que você queimou, nem mais nem menos.

![Uma tabela de placar lista o valor com harvest feito, o delta da tesouraria, a quantidade de buyback e o delta de supply pós-queima, cada um com a sua fonte, a afirmação que ele prova e a falha característica dele.](assets/v09-table.png)

**A sondagem empírica, se você quiser a resposta de verdade para uma pergunta que esta lição só apontou de longe.** Rode o buyback duas vezes, uma com uma fatia pequena da tesouraria e uma com ela inteira, e registre a diferença entre entregue e planejado em cada vez. Depois olhe a própria conta de token da tesouraria e ache o SPROUT retido sentado nela, taxas que o seu próprio buyback pagou para você mesmo.

Na porta B o segundo número é o honesto e o primeiro é um controle: a diferença vai escalar exatamente com o tamanho da ordem, porque o único custo em jogo é uma taxa percentual com um teto, e ver ela *não* se comportar mal é como você aprende com o que o slippage teria se parecido se houvesse um book. Na porta A o primeiro número é o que ensina, porque a diferença se alarga de forma super-linear conforme a sua ordem come a pool, e essa curva é o que preenche a tabela de política da seção de teoria com os seus próprios valores. A escolha entre varrer tudo mensalmente e comprar ordens pequenas continuamente é uma que nenhuma lição pode fazer por você, porque depende da profundidade da sua pool e não das suas intenções — e a porta B, não tendo profundidade nenhuma, não consegue responder isso de jeito nenhum. Saber qual dessas duas rodadas você tem na mão é a mesma habilidade que o m05-l1 pediu quando separou um preditor de uma prova.

## Checkpoint

Você terminou quando uma única rodada do script imprime os quatro números e sai com zero: valor com harvest feito, delta da tesouraria, quantidade de buyback, delta de supply pós-queima, com a queda de supply igual à queima até a unidade base. Guarde essa saída. A lição de capstone te pede para compor uma economia a partir das primitivas que você construiu, e este trilho é a peça que torna a palavra economia honesta em vez de decorativa.

Mais uma coisa antes de você fechar a pasta, e é a parte de um design de trilho de taxas que nunca aparece na thread de lançamento. Nomeie os dois sinais que te diriam que esta política está errada, e nomeie eles agora enquanto você não tem posição emocional sobre a resposta. Sinal um: a diferença entre entregue e planejado nos seus buybacks. Se ela ficar pequena, o tamanho da sua ordem cabe na sua pool e comprar continuamente é barato. Se ela se alargar conforme a tesouraria cresce, você está pagando um imposto crescente para converter receita em queima, e em algum momento rotear aquele SOL para outra coisa que não um buyback é o melhor uso dele. Sinal dois: a razão entre o que custa rodar o crank e o que ele arrecada. Um harvest que varre menos valor do que as transações custam para enviar não é um trilho de taxas, é um hobby, e a resposta honesta é varrer com menos frequência em vez de fingir que o cronograma está funcionando. Anote os dois limiares com números de verdade das suas próprias rodadas. Uma política que ninguém consegue falsear é um slogan.

Três falhas que eu espero. A primeira é um harvest que reporta zero num mint que claramente cobra taxas, o que quase sempre significa que o filtro memcmp está batendo no offset errado ou no programa errado, já que um mint de SPL clássico e um mint de Token-2022 são donos diferentes e a varredura é escopada por programa. A segunda é um `withdraw_withheld_tokens_from_mint` que falha na autoridade, o que significa que você passou um endereço onde o builder queria um signer. A terceira é a compra da porta B falhando no saldo do maker, o que significa que o passo 1c cunhou menos estoque do que metade da sua tesouraria consegue comprar a `MAKER_PRICE`; cunhe mais ou aumente o preço, e repare que ter que escolher é em si o sinal de que você é o mercado em vez de estar negociando em um. Se os números ainda se recusarem a fechar depois de você ter conferido os três, leve a saída da sua rodada e a sua aritmética esperada para a discussão do curso, e na porta A poste as reservas da pool junto com elas, porque metade das vezes a discordância é slippage e não um bug e as reservas são o que prova isso.

O SPROUT agora ganha taxas para uma tesouraria que ele controla e queima supply num cronograma que você define. O que ele não faz é se importar com quem está segurando ele. A seguir: decidir quem entra, e transformar os pontos de compost que a Overgrowth vem rastreando off-chain em SPROUT de verdade que as pessoas conseguem de fato gastar.
