# Cancelas e migrações de pontos para token: ligando a economia inteira

## Resumo

Na lição passada você construiu o trilho de taxas do SPROUT. Taxas de marketplace retidas com harvest feito a partir das contas dos destinatários onde o Token-2022 as tinha estacionado discretamente, roteadas para a tesouraria, gastas em um buyback (um swap DAMM v2 se você tivesse lançado o SPROUT e tivesse a pool dele, senão uma compra OTC do maker que você levantou), e queimadas, com o supply do mint caindo exatamente pelo que você queimou: o buyback mais a parcela da queima de taxas. Dinheiro entra, dinheiro sai, comprovável nas duas pontas.

Duas coisas na Overgrowth ainda rodam no achismo.

O canal alpha pergunta "você é um Founding Farmer?" e hoje ele acredita em qualquer coisa que o cliente disser, o que significa que o canal tem cancela no mesmo sentido em que uma porta está trancada quando a chave está colada no batente. E algumas centenas de milhares de pontos de compost estão parados numa tabela que é sua, prometendo aos jogadores um SPROUT que ainda não existe, resgatável sob termos que você não escreveu em lugar nenhum. Um desses é um problema de segurança. O outro é um problema de supply. Esta lição fecha os dois, e fechá-los é o que finalmente faz as quatro coisas que você construiu se comportarem como uma economia só em vez de quatro scripts que por acaso dividem uma pasta.

Antes de qualquer teoria, ponha a evidência real na sua tela. Pegue uma carteira que tem um crate Harvest e uma carteira que não tem, e pergunte a um índice que não tem motivo nenhum para bajular nenhuma das duas. Três linhas de setup primeiro, porque este é o primeiro uso de `jq` do curso e o bloco de env completo do lab só chega no passo 1:

```bash
brew install jq          # macOS; apt install jq on Debian/Ubuntu, or drop the pipe and read raw JSON
export DAS_RPC_URL=<your DAS endpoint>
export WALLET=<the wallet to probe>   # you will run the query twice, once with each wallet here
```

Aí a pergunta em si:

```bash
curl -s "$DAS_RPC_URL" -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":"gate","method":"getAssetsByOwner",
       "params":{"ownerAddress":"'"$WALLET"'","page":1,"limit":50}}' \
  | jq '.result.items[] | {id, interface, owner: .ownership.owner,
                           collection: (.grouping[]? | select(.group_key=="collection") | .group_value)}'
```

Essa é a fronteira de confiança inteira em uma requisição só. A resposta é uma lista de ativos que um indexador público diz que este dono tem, e nada disso veio da pessoa pedindo acesso. Rode contra as duas carteiras. Uma imprime um crate na sua coleção Almanac, a outra imprime uma lista vazia ou o lixo de outra pessoa. Deixe os dois terminais abertos, porque a cancela que você está prestes a escrever transforma exatamente essa saída em um booleano.

No fim você vai ter o `gate-and-migrate.ts`: um script que passa um holder Founding-Farmer pela porta, nega um estranho, depois converte os pontos de compost desse holder em SPROUT de verdade por um claim de Merkle, e recusa o mesmo claim uma segunda vez. É a última peça do `sprout-economy`, e é o primeiro script deste curso que encosta em quatro artefatos anteriores de uma vez: o mint do módulo 2, os crates do módulo 7, o leitor do módulo 7, e o caminho de claim do airdrop do módulo 8.

O recuo, dito de saída para você saber onde as rodinhas são tiradas. A cancela é trabalhada por inteiro, código e raciocínio, porque a fronteira de confiança é a lição e eu não quero você adivinhando nela. A fiação do claim é um problema de completion: eu te entrego a árvore e a transação, você escreve as duas checagens que decidem se um claim é legítimo. O fluxo completo, os dois trilhos em uma execução só com um segundo claim rejeitado, é só seu.

## A porta e a janela

A sede da Overgrowth tem duas aberturas, e elas falham em direções opostas.

Existe uma **porta**, onde alguém alega ser membro e você decide se abre. E existe uma **janela**, onde alguém entrega um recibo e você devolve grão do silo. Uma porta que confia na evidência errada deixa entrar gente que deveria estar do lado de fora, o que é chato e recuperável, já que você sempre pode trocar a fechadura e reconferir todo mundo amanhã. Uma janela que honra o mesmo recibo duas vezes entrega grão que nunca esteve no silo, e nenhuma quantidade de reconferência traz ele de volta. O primeiro erro te custa exclusividade, o segundo custa a todo holder de SPROUT em diluição, e o segundo é o que você não consegue desfazer.

Aqui está a rota por esta lição. Primeiro a porta: em que evidência uma cancela tem permissão de acreditar, e o que te custa que a evidência seja um índice em vez da chain em si. Depois a janela: por que um monte de pontos vira um token por um claim em vez de uma cunhagem em massa, o que a folha realmente hasheia, e onde a trava contra um claim duplo realmente mora. Depois você liga as duas em uma execução só.

### Em que uma cancela tem permissão de acreditar

O cliente de um usuário consegue te dizer três coisas diferentes, e só uma delas é evidência.

Ele pode te mandar uma **mensagem assinada** dizendo "eu tenho um crate Founding-Farmer." A assinatura é real, a criptografia fecha, e ela prova exatamente uma coisa: quem mandou controla aquele keypair. Ela não diz nada sobre o que o keypair tem agora. Uma carteira que era dona de um crate na terça passada, vendeu na quarta, e assina a sua mensagem na quinta produz uma assinatura perfeitamente válida e uma alegação perfeitamente falsa. Assinaturas respondem "quem é você," nunca "o que você tem."

Em código, esse erro tem uma forma, e vale a pena conseguir reconhecer ela num pull request de bate-pronto:

```typescript
// overgrowth/anti-gate.ts - the shape to recognize and refuse. Do not ship this.
const FOUNDING_FARMER = 'founding-farmer';

interface AccessRequest {
  wallet: string;
  signature: string; // cryptographically valid, and beside the point
  claimsToHold: string; // written by the applicant
}

// A signature check answers "who signed this?", never "what do they hold?".
declare function signatureIsValid(wallet: string, signature: string): boolean;

export function badGate(req: AccessRequest): boolean {
  return signatureIsValid(req.wallet, req.signature) && req.claimsToHold === FOUNDING_FARMER;
}
```

O sinal é que o segundo operando saiu do corpo da requisição. Todo campo ali dentro foi escrito pela pessoa pedindo para ser deixada entrar, e nenhuma quantidade de validação de assinatura promove a autoria dela a evidência.

Ele pode te mandar a **lista de tokens em cache** dele, aquela que o wallet adapter mantém no navegador para a sua página de portfólio renderizar rápido. Esse cache é uma conveniência para o usuário e uma sugestão para você. Ele é desatualizado por design e editável por qualquer um com o devtools aberto. Ler ele não é verificação, é pedir para o candidato preencher a própria carta de recomendação.

Ou você pode **ler a posse você mesmo**, de uma fonte que o candidato não controla. Essa é a única que conta. Para um saldo fungível você consegue bater na chain direto. Para um crate Harvest você não consegue, porque um NFT comprimido não tem conta própria: leituras de cNFT exigem um RPC com suporte a DAS, que é a restrição que você encontrou quando construiu o leitor, e é o motivo de a cancela para um badge de cNFT ser uma chamada DAS e não um `getAccountInfo`.

![Três cartões de evidência comparam uma mensagem assinada, uma lista de tokens em cache no cliente, e uma leitura DAS, com só a leitura DAS provando posse atual, sob uma ressalva de atraso de indexador.](assets/v01-comparison.webp)

Então a regra é curta. Uma cancela decide com base em uma leitura na qual o candidato não consegue escrever. Todo o resto é um mimo de experiência de usuário que você pode mostrar na UI e nunca deve usar para bifurcar.

### O custo da atualidade

Agora a parte honesta, e é o motivo de esta seção existir em vez de uma linha só dizendo "é só usar DAS."

DAS é um índice. Ele observa a chain e anota o que vê, e anotar o que vê leva tempo. Um crate transferido cinco segundos atrás ainda pode resolver para o dono anterior, o que significa que a sua porta pode admitir alguém que genuinamente não tem mais o badge. Isso não é um bug na sua cancela e não é um bug no provedor. É o que um índice é.

Você tem três jeitos de pagar esse custo, e eles custam valores diferentes.

**Aceite o atraso.** Para um cargo no Discord ou um canal alpha, um ex-holder mantendo acesso por alguns segundos ou alguns minutos é um não-evento. Reconfira conforme um cronograma e a janela fecha sozinha. Essa é a resposta certa muito mais vezes do que os engenheiros gostariam que fosse.

**Confirme contra uma fonte mais atual.** Para uma cancela de fungível, siga a leitura DAS com um `getTokenAccountsByOwner` direto no commitment `confirmed` e bifurque nesse número em vez disso. Você perde a simplicidade de uma chamada só, você ganha uma leitura tão atual quanto o cluster. Para um cNFT não existe atalho equivalente, porque o ativo genuinamente não tem conta para ler, e a opção honesta é uma leitura de prova que custa uma ida e volta e ainda assim resolve pelo mesmo índice.

**Ponha a cancela em cima de algo que não pode se mover.** Essa é a minha favorita e quase ninguém recorre a ela. Se o badge é soulbound, o caso de transferência que torna a desatualização perigosa não existe. O Bubblegum v2 já vem com `set_non_transferable_v2`, então o crate Founding-Farmer pode ser cunhado sem conseguir sair da carteira para a qual foi concedido, o que contradiz frontalmente o folclore da era 2024 de que NFTs comprimidos não podem ser soulbound. Você já cunhou um assim no módulo 7. Um badge soulbound não torna o índice instantâneo, ele tira a transferência do modelo de ameaça, que é um tipo de correção diferente e melhor.

![Uma linha do tempo mostra uma transferência de cNFT caindo na blockchain, o índice DAS atrasando, e uma checagem de cancela dentro dessa lacuna passando erroneamente um ex-holder, com os remédios alinhados embaixo.](assets/v02-timeline.webp)

Existe uma quarta resposta para a qual as pessoas correm e eu quero nomear ela para você pular: fazer streaming do estado você mesmo para sempre ter a visão mais atual. Isso é uma técnica real e é um projeto real. Construir indexadores, plugins Geyser e pipelines gRPC é o material do curso planejado Client-Side Mastery, e se a sua cancela genuinamente precisa de atualidade abaixo de um segundo em ativos comprimidos, é para lá que se vai. Para uma porta de membros, é uma plataforma de dados que você agora possui para que um estranho não consiga ler o seu canal alpha por onze segundos.

### Pontos são uma promessa, SPROUT é a liquidação

Troque de abertura. A janela é onde a economia mora, e vale desacelerar, porque esta é a parte que os times erram em público.

Pontos de compost nunca foram um token. Eles são um número numa tabela que você controla, e eles valem exatamente o que o seu eu futuro decidir que eles valem. Isso é ok, é para isso que os pontos servem: eles deixam você recompensar comportamento antes de ter que se comprometer com supply. Todo programa de pontos neste ecossistema é a mesma troca, diga isso ou não. Você está rodando uma promessa, denominada em uma unidade que você pode redefinir, até o dia em que não pode.

A migração é o dia em que você não pode. É o momento em que o número privado vira um número público, e três decisões são tomadas, quer você as tome deliberadamente ou por acidente.

**Quem é elegível.** Um snapshot, tirado em um bloco declarado ou um timestamp declarado, publicado para as pessoas conseguirem conferir a própria linha. O snapshot é a parte que torna a coisa toda auditável, e tirar ele na surdina é como uma migração vira um escândalo.

**A que razão.** Quantas unidades base um ponto vira. Esta é a decisão de tokenomics inteira, e não existe mecanismo que decida ela por você.

**Em que cronograma.** Tudo de uma vez, ou uma parcela desbloqueada agora e o resto liberado ao longo do tempo.

Veja a razão fazer o trabalho dela com números redondos. Digamos que a Overgrowth tem 100,000 pontos de compost em aberto entre 4,000 jogadores, e você fecha em 10 unidades base de SPROUT por ponto. Isso são 1,000,000 de unidades base novas, cunhadas para dentro da existência no claim. Se o supply do SPROUT antes da migração era 9,000,000, você acabou de decidir que os holders de pontos ficam com 10% do token, e que todo mundo que já tem SPROUT possui uma fatia proporcionalmente mais fina do que tinha ontem. Ninguém foi roubado. Nada foi tirado de uma carteira. O custo foi pago em diluição pelas pessoas que já estavam lá, e a razão é a fatura.

Coloque a razão em 1 unidade base por ponto e os mesmos 100,000 pontos viram 100,000 unidades base, uns 1% do supply, e os seus grinders fiéis se sentem passados para trás. Coloque em 100 e os seus holders existentes são diluídos pela metade. O mecanismo que você está prestes a construir é idêntico nos três casos. O mecanismo é de graça. A razão não é.

![Um gráfico de barras agrupadas converte os mesmos 100,000 pontos de compost em três razões contra um supply existente fixo de 9,000,000, dando aos holders de pontos de cerca de um a cinquenta e três por cento.](assets/v03-chart.webp)

O que traz à tona o estudo de caso, e uma lacuna honesta nele. A Kamino rodou uma migração de pontos para token no KMNO, e é a coisa óbvia para apontar porque é uma das maiores que este ecossistema já fez. O que eu não consegui fazer foi verificar a tokenomics da conversão. Os números que te deixariam dizer "eles converteram a X por ponto" não estão publicados em lugar nenhum que eu pudesse confirmar, então eu não vou colocar na sua cabeça uma razão que eu não consigo referenciar. Pegue o mecanismo dele, uma migração por claim de Merkle de um ledger de pontos off-chain para um token on-chain, e pegue a razão da sua própria matemática de supply. Esse é o uso correto de um estudo de caso cujos números são privados, e é a mesma regra que este curso aplicou a toda figura disputada: meça ou cite, nunca divida a diferença.

### Por que um claim, e não uma cunhagem em massa

Você tem a lista de elegibilidade. Por que não simplesmente cunhar para todo mundo e acabar com isso?

Porque quem faz a cunhagem paga pelas contas. Você custeou exatamente isso quando construiu o airdrop de compost: uma conta de token clássica é 293 bytes de rent por destinatário, cerca de 1,855,569 lamports na taxa da mainnet em 2026-09-06, então empurrar tokens para 100,000 carteiras dá uns 186 SOL antes de você ter mandado uma única transação, e a rota comprimida trouxe isso para uns 1.03 SOL, bem mais de 99% economizado. Re-derive o lado clássico a partir da taxa do seu próprio cluster antes de citar ele; o lado comprimido não é rent e não se move.

Um claim muda quem está segurando a fatura. O distribuidor põe uma raiz na chain. Cada destinatário que quer os tokens dele manda a própria transação, paga o próprio rent de conta, e recebe os próprios tokens. E a cauda que nunca faz claim nunca te custa nada, o que importa mais do que parece: em todo drop grande, uma parcela significativa da alocação simplesmente nunca é arrecadada. Sob um modelo de push, você pagava rent para criar contas para pessoas que nunca iam voltar.

![Uma comparação de três colunas entre pushes de conta clássica, pushes de conta comprimida, e um claim de Merkle mostra que o claim transfere o custo para os destinatários e nunca cunha a cauda sem claim.](assets/v04-comparison.webp)

Existe um segundo motivo, e é aquele pelo qual o drop do JTO é famoso. Um distribuidor consegue segurar dois valores por destinatário: uma parcela que desbloqueia na hora, e uma parcela que é liberada ao longo do tempo. A Jito distribuiu o airdrop dela por um distribuidor de Merkle open-source com vesting linear que rodou até 2024-12-07, e o programa que fez isso, `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv`, ainda é a implementação de referência para esse padrão. A instrução que libera a parcela de vesting é `claim_locked`, e você já encontrou ela na lição de airdrop. A migração quer essa divisão mais do que um airdrop quer: um programa de pontos recompensa quem apareceu cedo, e entregar a cada um deles tokens totalmente líquidos no dia um é uma escolha de design com um gráfico muito previsível anexado.

### A folha, byte a byte

Aqui é onde um claim de Merkle deixa de ser uma abstração. Eu li o código-fonte do programa de referência em vez de descrever ele de memória, em 2026-08-22, e você deveria reler antes de apontar um distribuidor de verdade para dinheiro de verdade, porque um repositório que ficou quieto ainda pode mudar.

O distribuidor guarda uma raiz de 32 bytes. A entrada de um claimant é hasheada duas vezes. Primeiro o claim em si: SHA-256 sobre o endereço de 32 bytes do claimant, depois o valor desbloqueado dele como um u64 little-endian, depois o valor bloqueado dele como um u64 little-endian. Depois o resultado é hasheado de novo com um único byte `0` na frente, o prefixo de folha. Nós internos usam um byte `1` na frente em vez disso, sobre os dois filhos ordenados pelo valor em bytes, do menor primeiro.

Esses dois bytes de prefixo não são decoração. Sem eles, uma "folha" de 64 bytes poderia ser forjada para parecer um par de nós internos, e um claimant conseguiria provar a participação de uma folha que nunca esteve na árvore. Esse é o ataque de segunda pré-imagem, e a correção é um byte por hash. Ela aparece em quase toda implementação séria de Merkle exatamente por esse motivo, e o fato de a correção ser tão barata é o motivo de não haver desculpa para pular ela.

![Um detalhamento anotado da folha do distribuidor mostra o claimant e os valores hasheados em um nó, prefixos de byte zero e um em folhas e nós internos, explicados como proteção de segunda pré-imagem.](assets/v05-annotated-code.webp)

O retorno de saber isso com precisão é que você consegue computar a raiz localmente, em TypeScript, e obter os mesmos 32 bytes que o verificador on-chain vai computar. É assim que você confere uma distribuição antes de publicar ela, e como você debuga o único claim que falha enquanto os outros nove mil funcionam.

### Onde a trava de claim duplo realmente mora

Última peça de teoria, e é o problema da porta de novo, uma abertura ao lado.

Uma prova de Merkle prova que uma entrada está na árvore. Ela não prova que a entrada ainda não teve claim feito, e nunca vai poder, porque a prova é idêntica toda vez. Alguma coisa tem que lembrar. O distribuidor de referência lembra criando uma conta `ClaimStatus` por claimant, derivada das seeds `"ClaimStatus"`, o endereço do claimant, e o endereço do distribuidor, guardando o claimant, o valor bloqueado, o valor já sacado, e o valor desbloqueado. A conta é criada dentro da transação de claim. Tente fazer claim duas vezes e a segunda transação falha em criar uma conta que já existe.

Repare no que torna isso confiável: o seu cliente não consegue escrever nela. A trava não é uma flag no seu script, ela é um efeito colateral da mesma transação que move os tokens, o que significa que ela não consegue sair de sincronia com os tokens.

O que te diz quanto vale um ledger client-side. No lab de hoje você vai manter um arquivinho JSON de quem já fez claim, e esse arquivo vai corretamente impedir o seu script de pagar a mesma carteira duas vezes. É um ensaio, não uma fronteira. Se a autoridade de mint de verdade é uma chave no seu script e a única coisa entre uma carteira e uma segunda concessão é um arquivo no seu laptop, então uma segunda concessão está a um arquivo perdido de distância. Diga isso em voz alta quando escrever, porque a forma do código vai parecer tranquilizadoramente com a coisa real.

![Dois fluxos comparam um ledger JSON client-side, onde a trava fica fora da transação de mint, com o PDA ClaimStatus on-chain, onde trava e transferência acontecem em uma transação atômica só.](assets/v06-flowchart.webp)

### O trade-off, nomeado

Quatro custos, e nenhum deles some por você ser cuidadoso.

Uma cancela DAS é só tão atual quanto o indexador, então um ativo recém-transferido ainda pode resolver para o dono antigo e a sua porta pode admitir um ex-holder por um instante. Confiar numa posse reportada pelo cliente ou numa assinatura pura e simples em vez disso não é uma versão mais barata disso, é uma falha diferente e muito pior, porque a primeira é limitada por uma escrita de índice e a segunda não é limitada por nada.

Um claim de Merkle transfere o custo para os seus destinatários, o que é justo quando eles querem os tokens e hostil quando eles não sabem que existe um claim, e ele adiciona uma dependência de programa que você não controla. O repositório de referência está quieto faz um tempo, e quieto é um risco real para código que vai estar segurando uma distribuição muito depois de você fazer o deploy dele.

O mecanismo de migração é portátil, a tokenomics não é. Você consegue copiar o caminho de claim numa tarde e ficar confiante de que ele funciona, porque você consegue computar a raiz você mesmo e conferir toda prova antes de qualquer um fazer claim. Ninguém consegue te entregar a razão, e nenhuma quantidade de leitura da migração dos outros vai produzir ela, que é precisamente a lacuna que o estudo de caso da Kamino deixa aberta.

E uma trava de claim que mora no seu processo em vez de na transação não é uma trava, é um hábito que por acaso funciona até a primeira vez em que duas cópias do seu script rodam ao mesmo tempo.

![Um resumo de quatro linhas emparelha cada trade-off aceito com o que o limita, do atraso de indexador passando por claims pagos pelo destinatário até a janela de corrida do ledger client-side.](assets/v07-comparison.webp)

## Lab: gate-and-migrate.ts

Os dois trilhos, uma execução. A porta lê um índice público, então ela roda contra a devnet onde os seus crates foram de fato cunhados. A janela cunha SPROUT de verdade, então ela roda onde quer que o seu mint viva. Se os dois vivem na devnet, um arquivo de env cobre a coisa toda.

**1. Workspace e pins.**

Trabalhe na mesma pasta `overgrowth/` que segura o `das.ts` e o `classify.ts` da lição do leitor, porque você está prestes a importar os dois.

```bash
cd overgrowth
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Pins conferidos contra o npm em 2026-09-05. A tag `latest` do kit é a 8.2.0, publicada em 2026-08-29, mas latest não é a regra: um workspace fixa o major do kit contra o qual as próprias deps `@solana-program/*` dele fazem peer. Aqui esse client é o `@solana-program/token-2022@0.15.0`, cujo range de peer aceita kit `^7.0.0` — tudo da 0.16.0 em diante faz peer com `^8` — e isso decide o resto: kit 7.1.1, o release mais novo dentro do range. Esses clients entregam versões todo mês. Rode `npm view @solana-program/token-2022 peerDependencies` antes de confiar no par.

Aí o ambiente. Oito valores, nenhum segredo no repo — o último é um caminho, e o arquivo para o qual ele aponta é o `treasury.json` que você cunhou no passo 1b da lição passada, porque o `mintTo` da janela tem que ser assinado pela autoridade de mint do SPROUT e aquele setup colocou a autoridade exatamente nesta chave (se o seu SPROUT é anterior àquele passo, re-cunhe conforme ele primeiro; não tem como assinar o seu caminho ao redor de uma autoridade descartável morta):

```bash
export DAS_RPC_URL="https://<your-das-provider-endpoint>"
export RPC_URL="https://api.devnet.solana.com"
export WS_URL="wss://api.devnet.solana.com"
export SPROUT_MINT="<your Token-2022 mint from module 2>"
export ALMANAC_COLLECTION="<the Core collection your crates belong to>"
export HOLDER_WALLET="<a wallet holding a Founding-Farmer crate>"
export STRANGER_WALLET="<any wallet that does not>"
export KEYPAIR="<path to labs/m09-l1/treasury.json from m09-l1 step 1b>"
```

Se você preferir rodar a metade do mint localmente, o surfpool funciona aqui também (1.2.1 nesta máquina, 2026-08-22; no macOS `brew install txtx/taps/surfpool`, senão pegue um binário de release), iniciado com `surfpool start --no-tui --no-studio` e apontado para `http://127.0.0.1:8899` e `ws://127.0.0.1:8900`. A metade da porta ainda precisa de um endpoint DAS de verdade, porque um surfnet local não tem indexador nenhum observando ele.

**2. A porta.**

Esta é trabalhada por inteiro. Leia a forma primeiro: uma regra, uma leitura, um resultado que carrega o próprio timestamp.

```typescript
// overgrowth/gate.ts - decide access from an indexed on-chain read, never from a client claim.
import { createSolanaRpc, type Address } from '@solana/kit';
import { das } from './das';
import { classifyAsset, type DasAsset } from './classify';

export interface OwnedAsset extends DasAsset {
  id: string;
  ownership?: { owner?: string; frozen?: boolean; non_transferable?: boolean };
  grouping?: { group_key: string; group_value: string }[];
  token_info?: {
    balance?: number;
    decimals?: number;
    price_info?: { price_per_token?: number };
  };
}

export type GateRule =
  | { kind: 'collection-badge'; collection: string }
  | { kind: 'token-balance'; mint: string; minimum: bigint };

export interface GateResult {
  owner: string;
  allowed: boolean;
  reason: string;
  evidence: string | null;
  readAt: string;
}

interface OwnerPage {
  total: number;
  limit: number;
  page: number;
  items: OwnedAsset[];
}

export async function ownedAssets(owner: string): Promise<OwnedAsset[]> {
  const out: OwnedAsset[] = [];
  for (let page = 1; ; page += 1) {
    const res = await das<OwnerPage>('getAssetsByOwner', {
      ownerAddress: owner,
      page,
      limit: 1000,
      options: { showFungible: true },
    });
    out.push(...res.items);
    if (res.items.length < res.limit) return out;
  }
}

export async function checkGate(owner: string, rule: GateRule): Promise<GateResult> {
  const readAt = new Date().toISOString();
  const assets = await ownedAssets(owner);

  if (rule.kind === 'collection-badge') {
    for (const asset of assets) {
      if (asset.ownership?.owner !== owner) continue;
      const inCollection = (asset.grouping ?? []).some(
        (g) => g.group_key === 'collection' && g.group_value === rule.collection,
      );
      if (!inCollection) continue;
      const kind = classifyAsset(asset).category;
      if (kind !== 'nft' && kind !== 'compressed-nft') continue;
      return {
        owner,
        allowed: true,
        reason: `holds a ${kind} in collection ${rule.collection}`,
        evidence: asset.id,
        readAt,
      };
    }
    return {
      owner,
      allowed: false,
      reason: `no asset in collection ${rule.collection} resolves to this owner`,
      evidence: null,
      readAt,
    };
  }

  const held = assets.find((a) => a.id === rule.mint && classifyAsset(a).fungible);
  // token_info.balance arrives as a JSON number from most DAS providers. Do
  // not launder it through Math.floor: last lesson's rule stands, a Number
  // above 2^53 lies quietly (the damage is already done at JSON.parse), and
  // above ~1e21 String() turns it into exponent notation. The check below
  // makes that second case loud: a provider that ships a balance too big for
  // a JSON number is a provider you escalate, not round.
  const rawBalance = held?.token_info?.balance;
  const asString = rawBalance === undefined ? "0" : String(rawBalance);
  if (asString.includes("e") || asString.includes("E")) {
    throw new Error(`balance ${asString} exceeds JSON number range: escalate to the provider`);
  }
  const balance = BigInt(asString.split(".")[0] || "0");
  return {
    owner,
    allowed: balance >= rule.minimum,
    reason: `indexed balance ${balance} against minimum ${rule.minimum}`,
    evidence: balance > 0n ? rule.mint : null,
    readAt,
  };
}

export async function confirmBalanceOnChain(
  rpcUrl: string,
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<bigint> {
  const rpc = createSolanaRpc(rpcUrl);
  const { value } = await rpc
    .getTokenAccountsByOwner(owner, { mint }, { encoding: 'jsonParsed', commitment: 'confirmed' })
    .send();
  let total = 0n;
  for (const account of value) {
    if (account.account.owner !== tokenProgram) continue;
    total += BigInt(account.account.data.parsed.info.tokenAmount.amount);
  }
  return total;
}

export function describe(result: GateResult): string {
  const verdict = result.allowed ? 'PASS' : 'DENY';
  return `${verdict}  ${result.owner}  ${result.reason}  (read at ${result.readAt})`;
}
```

Quatro decisões aí dentro valem as palavras delas. A re-checagem de `ownership.owner` parece redundante contra uma query por dono e não é: em algum momento você vai passar para essa função uma lista de ativos que você pegou em outro lugar, e no dia em que fizer isso, aquela linha é a diferença entre uma cancela e uma sugestão. O `classifyAsset` está fazendo trabalho de verdade em vez de decoração, porque é ele que impede uma posição fungível na mesma coleção de satisfazer uma regra de badge. O `readAt` existe para que, quando alguém reclamar de ter sido negado, você consiga responder com um timestamp em vez de um dar de ombros. E o `confirmBalanceOnChain` é o remédio de atualidade, deliberadamente separado, deliberadamente não chamado por padrão. Ligue ele para a cancela que protege algo caro, deixe desligado para um cargo de chat.

![Um fluxograma traça o checkGate de um endereço de dono passando por uma leitura DAS paginada e três checagens sequenciais, saindo para uma aprovação com evidência ou uma negação, com todo resultado carimbado com timestamp.](assets/v08-flowchart.webp)

**3. Rode a porta.**

```bash
npx tsx -e "import {checkGate,describe} from './gate'; \
  const rule={kind:'collection-badge',collection:process.env.ALMANAC_COLLECTION!} as const; \
  for (const w of [process.env.HOLDER_WALLET!, process.env.STRANGER_WALLET!]) \
    console.log(describe(await checkGate(w, rule)));"
```

Você quer duas linhas, uma de cada veredicto:

```
PASS  7xK…9fQ  holds a compressed-nft in collection 4vT…2mL  (read at 2026-08-22T14:07:11.402Z)
DENY  3nB…kW1  no asset in collection 4vT…2mL resolves to this owner  (read at 2026-08-22T14:07:12.118Z)
```

Se as duas linhas disserem DENY, confira o valor da coleção antes de conferir qualquer outra coisa. A coleção de um cNFT mora em `grouping` com `group_key` igual a `collection`, e o valor é o endereço do ativo de coleção, não o nome dele.

**4. A árvore.**

Agora a janela. Este arquivo é dado a você completo, porque ele tem que ser compatível byte a byte com o que um verificador on-chain computa e não existe nota parcial para uma raiz que está quase certa. E um reconhecimento que você merece, porque você construiu exatamente esta árvore duas lições atrás no `compost-airdrop` com nomes diferentes: `leafHash` lá é `hashLeaf` aqui, `tree.proofFor` vira `getProof` sobre níveis explícitos, e os bytes hasheados são idênticos, prefixo de folha, prefixo intermediário, pares ordenados e tudo. Esta cópia é deliberadamente autocontida para que `overgrowth/` não carregue nenhum import entre pastas para quebrar.

Se você duvida que as duas concordam, dê um diff na coisa certa. Hashear uma entrada pelo `leafHash` e pelo `hashLeaf` prova quase nada: folhas são hasheadas de forma idêntica por construção, então essa checagem passa mesmo quando os dois builders discordam. O lugar onde dois ports de Merkle realmente divergem é a regra de combinação de níveis, e ela só aparece numa largura de nível ÍMPAR, que uma lista de teste de quatro ou oito entradas nunca produz. Então construa a mesma lista de TRÊS entradas pelos dois e dê um diff nas raízes. Raízes iguais em n=3 é evidência; hashes de folha iguais é uma tautologia.

```typescript
// overgrowth/merkle.ts - the distributor's tree, byte for byte.
// Ported from jito-foundation/distributor (merkle-tree/src/merkle_tree.rs,
// programs/merkle-distributor/src/instructions/new_claim.rs, verify/src/lib.rs),
// read on 2026-08-22. Re-read before you trust this against a live distributor.
import { createHash } from 'node:crypto';
import { getAddressEncoder, type Address } from '@solana/kit';

const LEAF_PREFIX = Uint8Array.from([0]);
const INTERMEDIATE_PREFIX = Uint8Array.from([1]);
const addressEncoder = getAddressEncoder();

export interface ClaimEntry {
  claimant: Address;
  unlocked: bigint;
  locked: bigint;
}

function sha256(...parts: Uint8Array[]): Buffer {
  const hash = createHash('sha256');
  for (const part of parts) hash.update(part);
  return hash.digest();
}

function u64le(value: bigint): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(value);
  return buf;
}

export function hashLeaf(entry: ClaimEntry): Buffer {
  const claimant = new Uint8Array(addressEncoder.encode(entry.claimant));
  const node = sha256(claimant, u64le(entry.unlocked), u64le(entry.locked));
  return sha256(LEAF_PREFIX, node);
}

function hashPair(left: Buffer, right: Buffer): Buffer {
  return Buffer.compare(left, right) <= 0
    ? sha256(INTERMEDIATE_PREFIX, left, right)
    : sha256(INTERMEDIATE_PREFIX, right, left);
}

export function buildTree(entries: ClaimEntry[]): Buffer[][] {
  if (entries.length === 0) throw new Error('empty distribution: nothing to migrate');
  const levels: Buffer[][] = [entries.map(hashLeaf)];
  while (levels[levels.length - 1].length > 1) {
    const below = levels[levels.length - 1];
    const above: Buffer[] = [];
    for (let i = 0; i < below.length; i += 2) {
      const left = below[i];
      const right = i + 1 < below.length ? below[i + 1] : below[i];
      above.push(hashPair(left, right));
    }
    levels.push(above);
  }
  return levels;
}

export function getRoot(levels: Buffer[][]): Buffer {
  return levels[levels.length - 1][0];
}

export function getProof(levels: Buffer[][], index: number): Buffer[] {
  if (index < 0 || index >= levels[0].length) throw new Error(`no leaf at index ${index}`);
  const proof: Buffer[] = [];
  let idx = index;
  for (let level = 0; level < levels.length - 1; level += 1) {
    const nodes = levels[level];
    const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
    proof.push(siblingIdx < nodes.length ? nodes[siblingIdx] : nodes[idx]);
    idx = Math.floor(idx / 2);
  }
  return proof;
}

export function verifyProof(leaf: Buffer, proof: Buffer[], root: Buffer): boolean {
  let computed = leaf;
  for (const sibling of proof) computed = hashPair(computed, sibling);
  return computed.equals(root);
}
```

Dois detalhes para reparar, porque os dois são lugares onde uma árvore feita à mão dá errado. Um nó ímpar em um nível é pareado com **ele mesmo**, não promovido para o nível de cima, que é o que o builder de referência faz e o que as provas dele assumem. E todo par é ordenado antes de hashear, que é o motivo de o `verifyProof` conseguir dobrar uma prova sem ser informado se cada irmão era um filho da esquerda ou da direita.

Antes de construir qualquer coisa em cima dela, prove ela para você mesmo. Três entradas, três provas, um valor adulterado:

```typescript
// overgrowth/tree-check.ts - trust the tree only after you have tried to break it.
// Run from inside overgrowth/: npx tsx tree-check.ts
import { address } from '@solana/kit';
import { buildTree, getProof, getRoot, hashLeaf, verifyProof, type ClaimEntry } from './merkle';

const entries: ClaimEntry[] = [
  { claimant: address('11111111111111111111111111111112'), unlocked: 100n, locked: 0n },
  { claimant: address('So11111111111111111111111111111111111111112'), unlocked: 250n, locked: 50n },
  { claimant: address('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'), unlocked: 7n, locked: 3n },
];

const levels = buildTree(entries);
const root = getRoot(levels);
console.log('root', root.toString('hex'));

entries.forEach((entry, i) => {
  const ok = verifyProof(hashLeaf(entry), getProof(levels, i), root);
  console.log(i, ok ? 'PROOF OK' : 'PROOF FAILED');
});

const greedy = { ...entries[0], unlocked: 1000n };
console.log('tampered accepted?', verifyProof(hashLeaf(greedy), getProof(levels, 0), root));
```

Três linhas `PROOF OK` e um `false`. A última linha é a que importa: mude o valor e a folha muda, então a mesma prova não dobra mais para a mesma raiz. Essa é a propriedade de segurança inteira de uma distribuição, demonstrada em quatro linhas.

**5. O claim, e as duas checagens que você escreve.**

Aqui está o problema de completion. O arquivo abaixo está completo tirando as duas decisões que decidem se um claim é legítimo. Cubra elas, escreva elas você mesmo a partir da descrição, depois compare.

A primeira: depois de reconstruir a árvore e pegar a prova, recuse continuar a não ser que a prova dobre para a raiz que você está prestes a publicar. A segunda: recuse continuar se este claimant já fez claim. As duas são três linhas. As duas são o trabalho inteiro.

```typescript
// overgrowth/claim.ts - migrate compost points into SPROUT through the merkle path.
import { readFile, writeFile } from 'node:fs/promises';
import {
  address,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getAddressEncoder,
  getProgramDerivedAddress,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type TransactionSigner,
} from '@solana/kit';
import {
  fetchMint,
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getMintToInstruction,
  TOKEN_2022_PROGRAM_ADDRESS,
} from '@solana-program/token-2022';
import { buildTree, getProof, getRoot, hashLeaf, verifyProof, type ClaimEntry } from './merkle';

/** The reference claim program: Jito's merkle distributor, the one the JTO drop used. */
export const MERKLE_DISTRIBUTOR_PROGRAM = address('mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv');

/** A leaf is claimable once. Where that fact is stored is the whole security question. */
export interface ClaimLedger {
  isClaimed(claimant: Address): Promise<boolean>;
  markClaimed(claimant: Address): Promise<void>;
}

/** Rehearsal ledger: good enough to stop YOUR script running twice, and nothing more. */
export class FileClaimLedger implements ClaimLedger {
  constructor(private readonly path: string) {}

  private async load(): Promise<string[]> {
    try {
      return JSON.parse(await readFile(this.path, 'utf8')) as string[];
    } catch {
      return [];
    }
  }

  async isClaimed(claimant: Address): Promise<boolean> {
    return (await this.load()).includes(claimant);
  }

  async markClaimed(claimant: Address): Promise<void> {
    const claimed = await this.load();
    if (!claimed.includes(claimant)) claimed.push(claimant);
    await writeFile(this.path, JSON.stringify(claimed, null, 2));
  }
}

/** The real boundary: the distributor's per-claimant ClaimStatus account. */
export async function claimStatusAddress(
  distributor: Address,
  claimant: Address,
): Promise<Address> {
  const encoder = getAddressEncoder();
  const [pda] = await getProgramDerivedAddress({
    programAddress: MERKLE_DISTRIBUTOR_PROGRAM,
    seeds: [
      new TextEncoder().encode('ClaimStatus'),
      encoder.encode(claimant),
      encoder.encode(distributor),
    ],
  });
  return pda;
}

export class OnChainClaimLedger implements ClaimLedger {
  constructor(
    private readonly rpc: ReturnType<typeof createSolanaRpc>,
    private readonly distributor: Address,
  ) {}

  async isClaimed(claimant: Address): Promise<boolean> {
    const pda = await claimStatusAddress(this.distributor, claimant);
    const { value } = await this.rpc.getAccountInfo(pda, { encoding: 'base64' }).send();
    return value !== null;
  }

  async markClaimed(): Promise<void> {
    // No-op on purpose: the distributor program creates ClaimStatus inside the claim
    // transaction. A client cannot mark this, which is exactly why it is trustworthy.
  }
}

export interface MigrationResult {
  claimant: Address;
  minted: bigint;
  stillLocked: bigint;
  root: string;
  signature: string;
  supplyBefore: bigint;
  supplyAfter: bigint;
}

export interface MigrationInput {
  rpcUrl: string;
  wsUrl: string;
  entries: ClaimEntry[];
  index: number;
  mint: Address;
  mintAuthority: TransactionSigner;
  payer: TransactionSigner;
  ledger: ClaimLedger;
}

export async function migrateClaim(input: MigrationInput): Promise<MigrationResult> {
  const { entries, index, mint, mintAuthority, payer, ledger } = input;
  const entry = entries[index];
  if (!entry) throw new Error(`no distribution entry at index ${index}`);

  const levels = buildTree(entries);
  const root = getRoot(levels);
  const proof = getProof(levels, index);

  // CHECK ONE: the proof must fold to this root, or the snapshot and the tree disagree.
  if (!verifyProof(hashLeaf(entry), proof, root)) {
    throw new Error('proof does not reproduce the root: your snapshot and your tree disagree');
  }

  // CHECK TWO: one leaf, one claim.
  if (await ledger.isClaimed(entry.claimant)) {
    throw new Error(`${entry.claimant} already claimed this distribution`);
  }

  const rpc = createSolanaRpc(input.rpcUrl);
  const rpcSubscriptions = createSolanaRpcSubscriptions(input.wsUrl);

  const before = await fetchMint(rpc, mint);
  const [ata] = await findAssociatedTokenPda({
    owner: entry.claimant,
    mint,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const createAta = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer,
    owner: entry.claimant,
    mint,
  });
  const mintTo = getMintToInstruction({
    mint,
    token: ata,
    mintAuthority,
    amount: entry.unlocked,
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions([createAta, mintTo], m),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(signed, {
    commitment: 'confirmed',
  });
  await ledger.markClaimed(entry.claimant);

  const after = await fetchMint(rpc, mint);
  return {
    claimant: entry.claimant,
    minted: entry.unlocked,
    stillLocked: entry.locked,
    root: root.toString('hex'),
    signature: getSignatureFromTransaction(signed),
    supplyBefore: before.data.supply,
    supplyAfter: after.data.supply,
  };
}

/** Points to base units. The MECHANISM is reusable; this ratio is a tokenomics decision. */
export function pointsToSprout(
  points: bigint,
  perPoint: bigint,
  lockedBps: number,
): { unlocked: bigint; locked: bigint } {
  const total = points * perPoint;
  const locked = (total * BigInt(lockedBps)) / 10000n;
  return { unlocked: total - locked, locked };
}
```

Duas notas sobre o que isto deliberadamente não faz. Ele cunha só a parcela desbloqueada e reporta o resto bloqueado, porque liberar a parcela bloqueada ao longo do tempo é o caminho `claim_locked` do distribuidor e eu não vou fingir um cronograma de vesting em um script cliente. E o `OnChainClaimLedger` deriva o endereço da trava sem alegar dirigir o programa: ele te mostra onde a resposta mora e como pedir por ela, que é a parte que você leva para uma migração de produção.

**6. O fluxo.**

O seu snapshot de pontos, `compost-points.json`, a coisa que você publicaria para os jogadores conseguirem conferir a própria linha:

```json
[
  { "wallet": "7xK...", "compostPoints": 4200 },
  { "wallet": "3nB...", "compostPoints": 150 }
]
```

Troque as duas carteiras de placeholder pelos seus endereços reais antes de rodar qualquer coisa: a primeira linha é `$HOLDER_WALLET`, a segunda é `$STRANGER_WALLET`. Deixadas literais, o `loadEntries()` joga `7xK...` direto no `address()` do kit, que lança um erro de parse críptico muito antes de a trava mais amigável de sem-pontos-para-migrar ter qualquer chance de disparar.

E o script de topo, que é o artefato:

```typescript
// overgrowth/gate-and-migrate.ts - the Overgrowth economy, end to end.
// Run from inside overgrowth/: npx tsx gate-and-migrate.ts
import { readFile } from 'node:fs/promises';
import { address, createKeyPairSignerFromBytes } from '@solana/kit';
import { checkGate, describe } from './gate';
import { FileClaimLedger, migrateClaim, pointsToSprout } from './claim';
import type { ClaimEntry } from './merkle';

interface PointsRow {
  wallet: string;
  compostPoints: number;
}

const RPC_URL = process.env.RPC_URL ?? 'https://api.devnet.solana.com';
const WS_URL = process.env.WS_URL ?? 'wss://api.devnet.solana.com';
const SPROUT_MINT = address(process.env.SPROUT_MINT ?? '');
const ALMANAC_COLLECTION = process.env.ALMANAC_COLLECTION ?? '';
const HOLDER = process.env.HOLDER_WALLET ?? '';
const STRANGER = process.env.STRANGER_WALLET ?? '';

/** 1 compost point becomes 10 SPROUT base units; a quarter of the grant vests. */
const SPROUT_PER_POINT = 10n;
const LOCKED_BPS = 2500;

async function loadEntries(path: string): Promise<ClaimEntry[]> {
  const rows = JSON.parse(await readFile(path, 'utf8')) as PointsRow[];
  return rows.map((row) => {
    const { unlocked, locked } = pointsToSprout(
      BigInt(row.compostPoints),
      SPROUT_PER_POINT,
      LOCKED_BPS,
    );
    return { claimant: address(row.wallet), unlocked, locked };
  });
}

async function main(): Promise<void> {
  const signerBytes = new Uint8Array(
    JSON.parse(await readFile(process.env.KEYPAIR ?? 'treasury.json', 'utf8')) as number[],
  );
  const authority = await createKeyPairSignerFromBytes(signerBytes);

  console.log('--- the door ---');
  const rule = { kind: 'collection-badge', collection: ALMANAC_COLLECTION } as const;
  for (const wallet of [HOLDER, STRANGER]) {
    console.log(describe(await checkGate(wallet, rule)));
  }

  console.log('\n--- the window ---');
  const entries = await loadEntries('compost-points.json');
  const index = entries.findIndex((e) => e.claimant === HOLDER);
  if (index < 0) throw new Error(`${HOLDER} has no compost points to migrate`);
  const ledger = new FileClaimLedger('claimed.json');

  const result = await migrateClaim({
    rpcUrl: RPC_URL,
    wsUrl: WS_URL,
    entries,
    index,
    mint: SPROUT_MINT,
    mintAuthority: authority,
    payer: authority,
    ledger,
  });
  console.log(`root      ${result.root}`);
  console.log(`minted    ${result.minted} base units to ${result.claimant}`);
  console.log(`locked    ${result.stillLocked} (claim_locked releases this linearly)`);
  console.log(`supply    ${result.supplyBefore} -> ${result.supplyAfter}`);
  console.log(`delta ok  ${result.supplyAfter - result.supplyBefore === result.minted}`);

  console.log('\n--- the same leaf, twice ---');
  try {
    await migrateClaim({
      rpcUrl: RPC_URL,
      wsUrl: WS_URL,
      entries,
      index,
      mint: SPROUT_MINT,
      mintAuthority: authority,
      payer: authority,
      ledger,
    });
    console.log('DOUBLE CLAIM LANDED - your guard is not a guard');
  } catch (err: unknown) {
    console.log(`rejected: ${err instanceof Error ? err.message : String(err)}`);
  }
}

main().catch((err: unknown) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
```

**7. Rode.**

```bash
npx tsx gate-and-migrate.ts
```

A forma de uma boa execução, com um holder em 4,200 pontos de compost a dez unidades base por ponto e um quarto bloqueado:

```
--- the door ---
PASS  7xK…9fQ  holds a compressed-nft in collection 4vT…2mL  (read at 2026-08-22T14:22:03.771Z)
DENY  3nB…kW1  no asset in collection 4vT…2mL resolves to this owner  (read at 2026-08-22T14:22:04.410Z)

--- the window ---
root      3d8e48de…ac736403   (yours will differ: the root is a function of your entry list)
minted    31500 base units to 7xK…9fQ
locked    10500 (claim_locked releases this linearly)
supply    9000000 -> 9031500   (illustrative: your mint's real before/after appear here)
delta ok  true

--- the same leaf, twice ---
rejected: 7xK…9fQ already claimed this distribution
```

Leia as últimas quatro linhas como um conjunto. O delta de supply é igual exatamente ao valor cunhado, então nada vazou. O resto bloqueado é declarado em vez de cunhado, então o seu gráfico de supply bate com a sua promessa. E o segundo claim foi recusado por uma checagem que rodou antes de qualquer transação ser construída, que é onde as recusas pertencem.

![Cinco artefatos anteriores convergem para o gate-and-migrate.ts, cujas duas pistas internas emitem veredictos de cancela, SPROUT cunhado com um delta de supply, e um segundo claim rejeitado.](assets/v09-diagram.webp)

## Challenge

Ligue a coisa toda você mesmo, em uma execução só, e faça ela produzir três artefatos de evidência.

Primeiro, um resultado de cancela por carteira, os dois veredictos, cada um de uma leitura DAS. Segundo, um valor migrado cujo delta de supply bate até a unidade base, pelo caminho de Merkle em vez de um mint direto. Seja preciso sobre o que essa frase exige, porque o seu `migrateClaim` É um `getMintToInstruction` e isso está ok: a exigência é que o mint só dispare depois que a sua verificação de prova e a sua checagem de ledger de claim passem as duas, para que uma folha adulterada ou repetida nunca chegue nele. Você está provando que a cancela na FRENTE do mint é estrutural, não que você dirigiu o distribuidor de verdade, o que esta lição explicitamente se recusou a fazer. Terceiro, um segundo claim rejeitado da mesma folha.

Depois pressione em cima, porque a parte interessante não é o caminho feliz.

Mude o valor de um destinatário no `compost-points.json` depois de construir a árvore e antes de fazer claim, e veja a prova parar de dobrar para a raiz. Isso é um claimant tentando se pagar mais, e é a falha que o hash de folha existe para produzir.

Derive o endereço da trava para o seu holder e vá olhar ele:

```bash
npx tsx -e "import {address} from '@solana/kit'; import {claimStatusAddress} from './claim'; \
  console.log(await claimStatusAddress(address(process.env.DISTRIBUTOR!), address(process.env.HOLDER_WALLET!)));"
```

Troque o `FileClaimLedger` pelo `OnChainClaimLedger`, aponte ele para qualquer endereço de distribuidor, e leia o que volta. Ele vai dizer "sem claim," porque aquela conta `ClaimStatus` não existe para um distribuidor que você nunca criou. Você não consegue fazer ele dizer "com claim" a partir de um cliente, e essa incapacidade é a propriedade que você está de fato comprando.

E decida a sua própria razão antes de olhar a minha. Pegue o seu supply real de SPROUT, pegue o total de pontos de compost em aberto, e escreva com que porcentagem do token os holders de pontos deveriam terminar. Depois trabalhe de trás para frente até o número por ponto. Se essa porcentagem te deixa desconfortável, você acabou de descobrir por que anúncios de migração são os posts mais tensos que esses times escrevem.

## Checkpoint

O critério desta lição: `npx tsx gate-and-migrate.ts` passa um holder Founding-Farmer, nega uma carteira sem um, os dois a partir de uma leitura DAS, cunha a quantidade certa de SPROUT pelo caminho de Merkle com um delta de supply que bate, e rejeita o segundo claim daquela folha.

As falhas que eu espero, mais ou menos na ordem em que aparecem. Toda chamada DAS falhando com um erro de método não encontrado significa que o seu `DAS_RPC_URL` é um RPC comum, e um ativo comprimido simplesmente não pode ser lido de um desses. Um holder que fica sendo negado geralmente significa que o valor de `grouping` contra o qual você comparou é o nome da coleção em vez do endereço dela. Um delta de supply que não bate com o valor cunhado significa que você leu o mint antes e depois em commitments diferentes, ou que você cunhou o total em vez da parcela desbloqueada. E um segundo claim que cai significa que o seu caminho de ledger nunca rodou, o que num ledger client-side é um bug de uma linha e em produção é uma conta faltando.

Uma coisa para escrever que não é código. No seu README, uma frase por trilho: o que a sua cancela lê, e qual é a razão da sua migração. "A cancela do alpha resolve um crate Founding-Farmer pelo DAS e aceita até N segundos de atraso de índice." "Um ponto de compost converte para X unidades base, um total de Y por cento do supply, com Z por cento em vesting." Daqui a seis meses a primeira frase diz a um engenheiro de plantão se um ticket de suporte é um bug ou física, e a segunda é a frase pela qual você vai ser citado. Eu já vi times escreverem o código com cuidado e deixarem as duas frases implícitas, e a versão implícita é a que é redescoberta em público durante um incidente.

Dê um passo atrás e olhe o que roda agora. O SPROUT existe com extensões que você escolheu deliberadamente. Os crates existem, um deles permanentemente preso à carteira que o ganhou. Um leitor lê tudo isso. Taxas fluem para uma tesouraria, fazem buyback, e queimam. E uma promessa que você guardava num banco de dados agora é um token na carteira de alguém, cunhado só quando ela pediu por ele, e cunhável exatamente uma vez. Isso é uma economia, e cada peça dela é algo para o qual você consegue apontar um script e conferir.

A próxima lição tira os trilhos e olha para tokens que de fato foram entregues. PYUSD e JTO, em produção, em escala. A coisa mais interessante sobre eles não é o que eles fazem. É o que eles armam e nunca disparam: extensões configuradas com autoridades definidas e parâmetros em zero, paradas ali dormentes, esperando por uma decisão que ninguém tomou ainda. Depois de você ter construído tudo isso você mesmo, essa contenção para de parecer indecisão e começa a parecer um design.
