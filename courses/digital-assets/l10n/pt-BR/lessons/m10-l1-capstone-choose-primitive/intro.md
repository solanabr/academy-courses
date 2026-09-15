# Capstone: escolha a sua primitiva e ligue a economia

## Resumo

Duas lições atrás você ligou a migração de compost-points e a cancela de cNFT Founding-Farmer, que fechou a economia do Overgrowth: um mint, um cNFT, um airdrop e um leitor DAS disparando todos em um fluxo só. Na lição passada você leu tokens de produção em vez de escrever um, e viu o PYUSD da PayPal carregar oito extensões configuradas, quatro delas dormentes pela contagem do seu próprio classificador, com as duas que um venue julgaria primeiro, a taxa e o hook, deliberadamente deixadas desligadas.

Cada degrau até aqui te entregou a escolha. Construa o SPROUT com taxa mais harvest mais metadados. Cunhe a coleção do Almanac com um plugin Royalties. Dimensione uma árvore na profundidade 14. Esta lição tira as rodinhas. Você recebe um brief de produto e ninguém te diz qual primitiva buscar, porque esse é o trabalho de verdade, e porque a decisão é cara de desfazer: o conjunto de extensões de um mint é fixado na criação (as exceções estreitas do m02, a escrita de metadados pós-init e os reallocs do lado da conta, nunca cobrem as extensões de poder), um mint que a allowlist da Raydium recusa só entra naquele venue pela exceção de whitelist com avaliação manual que emissores regulados compram, e um cNFT que você não consegue ler é um hash que ninguém consegue resolver.

O recuo é total. Nenhuma construção trabalhada. Você recebe um scaffold `verify.ts` fixado na mesma toolchain que os labs usaram, um template de memo, e as receitas que você já escreveu. Todo o resto é seu.

Comece reivindicando um brief. Crie a pasta e escreva uma frase dentro dela antes de ler outro parágrafo:

```bash
mkdir -p capstone
printf '# Selection memo\n\nBrief: <the one sentence of product you are building>\n' > capstone/memo.md
```

Cinco briefs estão no cardápio, e o quinto é uma porta:

1. **Moeda de fidelidade de café.** Uma rede de seis lojas, pontos ganhos por compra, resgatáveis na loja, e o dono quer uma fatia de toda transferência entre pares.
2. **Um drop de edição de músico com royalties.** Impressões numeradas, um royalty de 5% registrado on-chain, uma coleção que os compradores conseguem verificar.
3. **Um token de pagamento regulado.** Divisível, tem que continuar negociável em um venue de verdade, e compliance precisa de um jeito de congelar um ator malicioso.
4. **Um badge de dispositivo DePIN.** Um badge por dispositivo implantado, potencialmente um milhão deles, não transferível, legível por qualquer um.
5. **O seu próprio produto.** Mesmas regras, mesma prova.

Escolha agora. O resto desta lição assume que você tem um na sua frente.

## Escolhendo a primitiva, e depois defendendo ela

### As quatro restrições escondidas em todo brief

Briefs de produto não dizem "use Token-2022 com um delegado permanente." Eles dizem coisas como "o dono quer uma fatia" e "compliance precisa de um jeito de congelar." O seu primeiro movimento é uma tradução, e só existem quatro perguntas que valem a pena fazer, porque juntas elas escolhem a família.

**É divisível?** Se um holder pode ter 0.4 dele, você precisa de um mint fungível, e só duas famílias te dão um: SPL clássico e Token-2022. Ativos Core e cNFTs são NFTs. Essa única pergunta mata mais designs candidatos que as outras três juntas, e mata de graça, antes de você ter escrito uma linha.

**Ele tem que continuar negociável?** Não "seria legal," mas tem que. Um ponto de fidelidade resgatado no balcão não tem requisito nenhum de venue. Um token de pagamento que não consegue entrar numa pool é um token de pagamento que ninguém consegue precificar. No momento em que a resposta é sim, a tese de compatibilidade da lição do token roteável toma conta da sua lista de extensões, e você não está mais escolhendo livremente.

**Quantos vão existir?** Um, mil, ou um milhão. Entre mil e um milhão está a linha onde a compressão para de ser esperta e passa a ser a única opção, e o custo de mint é o argumento inteiro.

**Quem tem que conseguir fazer o quê com ele depois que ele for entregue?** Congelar, reaver, impedir transferência, atualizar metadados, tirar uma fatia. Cada uma dessas mapeia para uma extensão ou plugin com nome, e cada uma delas é um poder que custa compatibilidade. Esta é a pergunta que transforma um brief em um conjunto.

Responda essas quatro por escrito, no seu memo, antes de tocar em uma receita. Eu já vi gente (eu incluído, num fim de semana de hackathon que eu prefiro não re-litigar) cunhar primeiro e responder depois, e o resultado é sempre o mesmo: um mint com o conjunto de extensões errado e um re-mint na manhã de domingo.

![Um fluxograma de decisão onde a divisibilidade escolhe fungível versus NFT, a negociabilidade restringe o conjunto de extensões, a quantidade escolhe Core versus NFTs comprimidos, e uma quarta pergunta sobre poderes pós-entrega atravessa todos os resultados.](assets/v01-flowchart.png)

### A matriz te recusa antes do mercado

Duas cancelas separam um conjunto de um mint publicado, e elas disparam nesta ordem.

A primeira é a matriz de conflitos que você portou no módulo 1. Cinco regras, tiradas direto de `check_for_invalid_mint_extension_combinations`, e elas não são conselho. Um conjunto que viola uma delas falha no `initialize_mint`, on-chain, com um erro `InvalidExtensionCombination` — e porque o fluxo ensinado junta create-account, init-extensions e `initialize_mint` em uma transação só, a coisa toda reverte atomicamente: os lamports de rent voltam para o seu payer e você fica no prejuízo só da taxa de transação, mais o redesenho. A sua função `checkCombo` já codifica as cinco, então o custo de perguntar é um import e uma chamada. Pergunte.

A segunda cancela é a allowlist do venue, e ela é mais estrita que a matriz do jeito que mais importa: a matriz te diz o que o token program recusa, e a allowlist te diz o que o *mercado* recusa. Essas são falhas diferentes. A primeira acontece em um segundo, na devnet, de graça. A segunda acontece semanas depois, quando alguém tenta criar uma pool e descobre que o seu mint não pode ter uma.

E debaixo das duas está a cilada que faz esta lição existir, a regra de só-no-nascimento do parágrafo de abertura: extensões são habilitadas na criação do mint, com apenas os dois caminhos pós-init estreitos do m02 como exceções. Toda extensão de poder no mint é só-no-nascimento. Não existe migração. Não existe patch. Se você errou o conjunto, você cunha um token novo e move todo mundo para ele, o que é um evento de produto, não um deploy.

Pense nisso como fundir um sino. Tudo sobre o timbre é decidido no molde, em um único despejo de metal, e uma vez que o metal esfria a sua única ferramenta restante é um esmeril. Você consegue afinar um sino depois de fundido. Você não consegue transformá-lo em outro sino.

### A tese de compatibilidade, da própria boca da Raydium

Aqui está a frase que deveria estar no seu memo. A referência de Token-2022 da Raydium (lida em 2026-08-21) não se esconde atrás de linguagem de política. Ela rejeita `PermanentDelegate` porque quem segura o delegado consegue varrer qualquer conta de token, *incluindo o vault da pool*, e rejeita `TransferHook` porque o hook invoca um programa customizado em toda transferência, com consumo de compute arbitrário.

Leia essas duas razões de novo, porque elas generalizam para além da Raydium. As duas são a mesma reclamação do assento de um integrador: a sua extensão moveu uma decisão que costumava pertencer à pool para as mãos de alguém que a pool não consegue auditar. Um venue que lista o seu token está tomando custódia dele dentro de um vault. Qualquer coisa que te deixe enfiar a mão naquele vault, ou fazer uma transferência custar uma quantidade ilimitada de compute, é um risco que ele não assinou embaixo.

Que é por isso que a allowlist do CP-Swap contém exatamente cinco extensões, e por que elas são as chatas: taxas, metadata pointer, token metadata, interest-bearing config, scaled UI amount. Exibição e contabilidade entram. Poder é recusado.

Então a tese, em uma linha que você pode entregar para um gerente de produto: extensões de taxa, de exibição e de contabilidade entram na whitelist, extensões de poder são recusadas. Repare na palavra que essa frase NUNCA pode conter: compliance. As extensões com forma de compliance, PermanentDelegate e DefaultAccountState, são exatamente os poderes que a allowlist recusa, e um PM que sai dizendo "compliance entra" entrega a regra errada. Qualquer outra coisa que você queira, ou você impõe fora do venue, ou você compra o seu lugar em uma whitelist estática do jeito que emissores regulados fazem.

![Uma tabela de comparação pontuando SPL clássico, Token-2022, Metaplex Core e cNFTs Bubblegum v2 em divisibilidade, custo por unidade, poderes pós-entrega disponíveis, roteabilidade em DEX, e o que um leitor precisa para resolvê-los.](assets/v02-comparison.png)

### O eixo de custo é onde o brief de mint em massa é decidido

Os números que resolvem o brief do badge DePIN saíram dos seus próprios labs, não de uma página de marketing.

Um mint Metaplex Core custa ~0.003 SOL publicados pelo fornecedor, uma conta, nenhum PDA de metadados, nenhum PDA de master edition. Essa é a opção não comprimida barata, e quem publicou o número foi a Metaplex, não este curso — til incluído, porque a tabela deles dá uma aproximação e endurecê-la em um quarto dígito seria este curso inventando precisão em nome deles.

A árvore que você dimensionou no módulo 7 na profundidade 14, buffer 64, canopy 8 tem 48,120 bytes e segura 16,384 folhas: 0.245 SOL na taxa de rent da devnet em 2026-09-06, ou seja, mais ou menos 0.000015 SOL por badge, cerca de 200 vezes mais barato por unidade que o Core, pago adiantado, por uma capacidade fixa com a qual você se compromete na criação. Esse múltiplo é dependente da taxa de um jeito que os de cNFT-para-cNFT não são, porque só um lado dele é rent: nos 6,960 pré-SIMD-0437 ele dava cerca de 150 vezes.

Não extrapole esse número por folha, e é essa a parte que pega as pessoas. O rent de árvore não é linear na contagem de folhas. Uma conta de árvore de Merkle concorrente é dimensionada por profundidade, buffer e canopy, não por quantas folhas você pretende preencher, então comprar capacidade é quase de graça enquanto comprar canopy não é. A árvore de um milhão de folhas do módulo 7, profundidade 20 com buffer 256 e canopy 14, tem 1,223,352 bytes para 1,048,576 folhas, o que deu 6.215 SOL na taxa de rent da devnet em 2026-09-06 e teria sido 8.515 SOL antes de o SIMD-0437 começar a cortar essa taxa. Precifique no seu próprio cluster; os bytes são a metade durável. De qualquer jeito são alguns milionésimos de um SOL por badge, centenas de vezes mais barato por unidade que o Core. A árvore maior é a mais barata por folha, o que é o contrário de toda outra intuição de rent que você tem, e é por isso que o brief de mint em massa é precificado contra a árvore que você realmente construiria em vez da que você construiu para praticar.

Um milhão de dispositivos a preços de Core dá mais ou menos 3,000 SOL. Um milhão de dispositivos em uma árvore só dá SOL de um dígito. Não existe argumento de design que sobreviva a essa razão, que é a coisa útil sobre lacunas de ordem de grandeza: elas encerram debates em vez de começá-los.

O custo chega do lado da leitura, e ele é a quarta cilada do curso. A pegada on-chain de um cNFT é um hash de folha. O ativo em si é reconstruído por indexadores DAS a partir de data stores que o RPC administra. Aponte um script de verificação para um RPC sem suporte a DAS e o `getAsset` não devolve nada, para um ativo que foi cunhado perfeitamente, e você vai passar vinte minutos desconfiando do seu asset id. Você não consegue refatorar esse custo para longe depois. É uma dependência que você aceita no momento do design, e ela pertence ao memo, bem ao lado da linha de rent da árvore.

![Um gráfico de barras em escala logarítmica comparando o custo por ativo de um mint Metaplex Core contra folhas de NFT comprimido em dois tamanhos de árvore, mostrando lacunas de duas e quase três ordens de grandeza.](assets/v03-chart.png)

### Os briefs de NFT têm exatamente um caminho de entrega

Se o seu brief caiu do lado NFT daquela primeira pergunta, a família está decidida e, na maior parte, o padrão também.

O Token Metadata é oficialmente legado. O Metaplex Core é o padrão recomendado para trabalho novo com NFT, o Bubblegum v2 para comprimido, e a lição de legado no módulo 6 é onde mora o argumento completo, então eu não vou re-litigá-lo aqui. O que importa para um memo de seleção é a única frase em que um revisor vai bater: pNFTs impõem royalties através do Token Auth Rules, que a Metaplex marca como deprecado e cujo rule set emblemático bloqueia zero programas. Armada e inativa. Se você escolhe esse caminho para imposição, você entregou uma stack deprecada para conseguir uma garantia que no momento não está garantindo nada, e `seller_fee_basis_points` em um NFT clássico de Token Metadata é puramente indicativo de qualquer jeito.

Então o drop do músico se resolve em uma coleção Core com o plugin Royalties carregando `basisPoints` e uma lista de criadores que soma exatamente 100, mais o plugin Edition para impressões numeradas, que é precisamente o passo de edição trabalhado que você já construiu dentro do Almanac. Escreva a realidade da imposição no memo em palavras simples: o split fica registrado on-chain, legível por todo mundo, e honrado pela política de marketplace em vez de pelo token program. Um comprador merece saber disso antes de comprar, e um memo que finge o contrário é o tipo de documento que envelhece mal.

O badge de dispositivo se resolve do outro jeito, em uma árvore Bubblegum v2 com folhas soulbound, e a seção de memo dele é sobre capacidade de árvore e dependência de leitura, não sobre royalties.

![Um fluxograma resolvendo briefs de NFT em uma coleção Core com plugins Royalties e Edition ou em uma árvore Bubblegum v2 soulbound, com o caminho deprecado do pNFT como beco sem saída.](assets/v04-flowchart.png)

### O PYUSD é o brief regulado, já resolvido

Se você pegou o brief três, alguém já entregou o seu token, e você consegue lê-lo.

PayPal e Paxos lançaram o PYUSD na Solana em maio de 2024 como um mint Token-2022 em `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`. Ele carrega oito extensões TLV: mint close authority, delegado permanente, transfer fee config, o par de transferência confidencial, transfer hook, metadata pointer, e token metadata. Eu reli o mint em 2026-08-22 e obtive as mesmas oito, seis decimais, inalteradas.

Agora a parte que faz disso uma lição em vez de um estudo de caso. `transferHook.programId` é null. Nenhum programa de hook está setado. A taxa é 0 basis points com um máximo de 0. E o seu classificador do m09-l3 contabilizou o mint em 4 ativas, 4 dormentes: o par confidencial está desligado do mesmo jeito, aprovação manual ligada e ninguém aprovado. Esta lição separa a taxa e o hook daquelas quatro dormentes porque elas são o par que uma caminhada pela allowlist realmente pesaria; o par confidencial é maquinário dormente que uma pool nunca inspeciona. De qualquer jeito, os poderes estão configurados e dormentes, o que significa que o emissor pagou o custo de tamanho de conta e o custo de compatibilidade para segurar opções que ele não exerceu.

Essa é uma estratégia deliberada e você deveria nomeá-la como tal se copiá-la. Configurar uma extensão é uma decisão permanente. Ativá-la é uma reversível. Um emissor regulado portanto adianta todo poder de que ele possa vir a precisar na cunhagem, e depois deixa as chaves desligadas, porque a alternativa é descobrir no ano dois que a obrigação de compliance dele exige uma extensão que ele não consegue adicionar.

O custo é exatamente o que a tese prevê: um mint carregando um delegado permanente é recusado por um programa de pool sem permissão por mais dormente que esse delegado esteja, porque a allowlist lê o tipo de extensão, não as suas intenções. O PYUSD é negociado assim mesmo. Ele chegou lá pelo caminho que um token de curso não tem, que é a coisa honesta de escrever no seu memo se você pegar este brief: o seu conjunto de compliance é defensável, e a sua rota até um venue é uma conversa de negócios, não uma transação.

![Uma leitura anotada do mint do PYUSD mostrando oito extensões configuradas, quatro delas dormentes, sob a regra de que configurar é permanente enquanto ativar é reversível.](assets/v05-annotated-code.png)

### A seção de venue deriva o próprio número

O seu memo precisa de uma seção de launch-venue, e ela consome a config de lançamento que você construiu no módulo 8 em vez de repetir um número de um blog.

O número em questão é 85 SOL, o limiar de graduação que todo mundo cita para uma moeda pump.fun. O seu `sprout-launch/derive-graduation.ts` não o armazena. Ele o calcula, a partir de três constantes publicadas e da invariante de produto constante: 30 SOL virtuais, 1,073,000,000 tokens virtuais, 793,100,000 tokens reais. Drene a reserva real e o lado virtual de token para em 279,900,000, então o lado virtual final de SOL é 30 vezes 1,073 sobre 279.9, mais ou menos 115.005, e o SOL que teve que entrar é 115.005 menos 30. Isso dá 85.005.

Que é o ponto inteiro. O limiar é consequência de uma curva que outra pessoa configurou, não uma constante da natureza, e a mesma função devolve 120 SOL para a curva alternativa que você tentou no módulo 8: 30 SOL virtuais, 1,000,000,000 tokens virtuais, 800,000,000 tokens reais. Mude qualquer uma das constantes do lado de token e o número se move. O seu memo não afirma 85. Ele roda a função e imprime o que as constantes de hoje produzem, para que quando as constantes mudarem o seu memo esteja errado alto em vez de errado quieto.

![Uma tabela de derivação levando as três constantes de curva publicadas da pump pela invariante de produto constante até uma reserva virtual final de SOL de 115.005 e um limiar de graduação de 85.005 SOL.](assets/v06-table.png)

A seção de venue também faz um trabalho que não tem nada a ver com curvas: ela pergunta se o venue consegue segurar o token que você escolheu. O `checkGraduationVenue` recusa o caminho da pump para qualquer mint Token-2022, porque a instrução `create` prende o token program clássico, e ele aceita o CP-Swap só quando toda extensão no seu conjunto está na allowlist de cinco itens. Rode-o contra o seu próprio conjunto declarado e cole a saída. Um veredicto de venue calculado a partir do seu conjunto vale mais que três parágrafos de prosa sobre roteabilidade, e leva um comando.

Composição de pool, roteamento e estratégia de LP para o que quer que você liste são uma disciplina diferente, e o curso planejado DeFi and RWA Engineering ensina isso direito. O seu memo para em "este venue consegue segurar este token, e aqui está o limiar nas constantes de hoje."

### Exatamente um trilho, e por que exatamente um

Quatro trilhos estão no cardápio ensinado: três saíram do módulo de economia, e o quarto, o airdrop, do compost drop do m08-l3. Você liga exatamente um dos quatro. O roteamento de taxa faz harvest dos valores retidos que uma taxa de transferência acumula para dentro de uma tesouraria. Uma cancela checa uma carteira em busca de uma posse e concede ou recusa acesso. Um airdrop distribui contra uma raiz de Merkle com um caminho de claim. Um buyback gasta SOL da tesouraria em um swap do lado do cliente e queima o que comprou.

A maioria dos briefs tem um encaixe óbvio. O dono do café querendo uma fatia das transferências entre pares é uma rota de taxa, porque a taxa já está acumulando nas contas dos destinatários e o trilho é o harvest. O drop do músico é uma cancela se os holders ganham alguma coisa, ou um airdrop se o drop é a distribuição. O badge de dispositivo é uma cancela quase por definição, já que o badge existe para autorizar um dispositivo. O token regulado é uma rota de taxa ou nada, porque o trilho de compliance é off-chain por construção — e se você pegar ele, repare que o conjunto de compliance endossado carrega a taxa dele em 0 bps, que não retém nada: ponha uma taxa simbólica diferente de zero (1 bps já basta) para o trilho ligado, ou a sua impressão de antes-e-depois não tem como diferir.

A razão de ser um e não três não é carga de trabalho. É prova. Um trilho só conta quando um script imprime um antes e um depois, e três trilhos meio ligados produzem zero disso enquanto um trilho terminado produz um recibo. Escolha o trilho cujo antes-e-depois você consegue de fato deixar visível em um terminal, e ligue esse direito.

![Uma tabela de comparação dos quatro trilhos de economia ensinados, listando o que cada um move, a prova que um script precisa imprimir, e o brief de produto em que cada um encaixa melhor.](assets/v07-comparison.png)

### Nomear a troca que você aceitou é o memo

Tudo acima colapsa em um meta-trade-off, e escrevê-lo é o entregável.

Toda escolha de primitiva troca poder por compatibilidade. Um mint Token-2022 com um delegado permanente te compra controle de compliance e é recusado por uma pool sem permissão. Um ativo Core é barato e rico em plugins e não é um token fungível, então nenhuma quantidade de esperteza com plugin o torna divisível. Um cNFT é quase de graça em um milhão de unidades e precisa de um RPC com DAS para ser lido, e não carrega nenhum estado de conta arbitrário em que você possa pendurar um programa.

Não existe posição de graça naquela curva. O memo não é onde você afirma que achou uma. É onde você escreve, em um parágrafo, de qual poder você abriu mão e o que ganhou em troca, para que a pessoa que herdar o seu token em dezoito meses consiga distinguir uma restrição de um acidente.

![Um diagrama de hub com o capstone no centro, alimentado por nove artefatos rotulados de partes anteriores do curso, e marcado como terminal, sem consumidor a jusante.](assets/v08-diagram.png)

## Lab: memo, ativo, trilho, prova

O critério desta lição é o `npx tsx capstone/verify.ts $ASSET_ADDRESS` sair com 0. Tudo antes disso é seu para rotear.

Uma regra permanente, e ela é avaliada: reutilize só habilidades que este curso ensinou. Nenhuma dependência que você já não tenha instalado em um lab anterior. Se o seu design precisa de um pacote que este curso nunca apresentou, o design está fora de escopo para o capstone, e essa restrição está te fazendo um favor ao manter a superfície pequena o suficiente para você realmente terminar.

**1. Prepare o ambiente, com os mesmos pins que os labs usaram.** Instale no seu workspace do curso, não em um novo, porque o `verify.ts` importa o leitor que você já escreveu:

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0
npm install -D tsx@4.23.12 typescript@5.9.3 @types/node@24
```

Nota de atualidade, já que estes pins são os que você vai re-checar primeiro quando algo quebrar daqui a um ano. Em 2026-09-05 o `latest` do npm para `@solana/kit` era 8.2.0, e este curso fixa a 7.1.1 mesmo assim: `@solana-program/token-2022@0.15.0` declara um peer range de `^7.0.0`, então o par acima é a combinação peer-válida para o código que você já escreveu. Nunca instale "latest" aqui. Confira o peer range e fixe o par exato.

**2. Escreva primeiro a metade legível por máquina do memo.** Memos em prosa se descolam da realidade; um memo JSON é comparado contra a chain. Este é o arquivo que o `verify.ts` lê:

```typescript
// capstone/memo.ts: the memo's machine-readable half (R13).
// The prose memo is for humans. This file is the part verify.ts reads.
import { readFileSync } from 'node:fs';

export type PrimitiveFamily = 'spl-token' | 'token-2022' | 'core-asset' | 'cnft';

export interface SelectionMemo {
  /** One line of product, in the brief's own words. */
  product: string;
  primitiveFamily: PrimitiveFamily;
  /**
   * Token families: extension kinds, spelled the way the Token-2022 client
   * spells them. core-asset: plugin names. cnft: the structural tags DAS
   * can actually return.
   */
  declaredSet: string[];
  /** Exactly one, from the rails this course taught. */
  rail: 'fee-route' | 'gate' | 'airdrop' | 'buyback';
  /** The power you gave up, or the compatibility you gave up. In writing. */
  tradeoff: string;
  /** The address you shipped. verify.ts cross-checks its argv against this. */
  assetAddress: string;
}

// The full cNFT tag menu verify.ts understands. Leave this constant alone:
// your declaration happens in memo.json, and it must list only the tags YOUR
// mint will actually produce. 'compressed' always appears; 'collection'
// appears only if you mint the leaf INTO a collection. A badge minted
// collectionless comes back as ['compressed'] alone, so declare just that, or
// mint under a collection and declare both. Neither is wrong for the badge
// brief; verify.ts holds you to whichever you declared.
export const CNFT_TAGS = ['compressed', 'collection'];

export function loadMemo(path = 'capstone/memo.json'): SelectionMemo {
  let raw: string;
  try {
    raw = readFileSync(path, 'utf8');
  } catch {
    throw new Error(`${path} not found. Fill it from the template in this lesson.`);
  }
  const memo = JSON.parse(raw) as SelectionMemo;
  if (memo.declaredSet.length === 0 && memo.primitiveFamily !== 'spl-token') {
    throw new Error(
      `${path}: declaredSet is empty for ${memo.primitiveFamily}. An empty set is a claim too, so make it on purpose.`,
    );
  }
  return memo;
}
```

Essa única trava existe por causa de um modo de falha que eu quero que você encontre aqui em vez de depois. Um `declaredSet` vazio em um mint Token-2022 normalmente significa que alguém pulou o passo de design, então ele lança a menos que a família seja SPL clássico, onde vazio é a resposta correta. Repare no que o `loadMemo` deliberadamente *não* checa: `assetAddress`. O memo é legitimamente sem endereço até o passo 5, porque o trabalho de design dos passos 3 e 4 acontece antes de qualquer coisa ser cunhada. A exigência de endereço pertence ao `verify.ts`, o único consumidor para o qual um memo que verifica contra nada é um documento, não uma prova, e você vai ver ele impor isso sozinho.

**3. Preencha o `capstone/memo.json` antes de cunhar.** A ordem importa. Este arquivo é uma previsão, e cunhar é o experimento:

```json
{
  "product": "Cafe loyalty currency for a six-store chain",
  "primitiveFamily": "token-2022",
  "declaredSet": ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
  "rail": "fee-route",
  "tradeoff": "no PermanentDelegate, so no clawback if a wallet is compromised; in exchange the mint stays poolable on CP-Swap with no whitelist request",
  "assetAddress": ""
}
```

Depois passe o conjunto pela matriz antes que ele chegue a uma transação. Este aqui ganha o próprio arquivo, já que pular ele é o deslize que te custa um mint: salve como `capstone/precheck.ts` e rode `npx tsx capstone/precheck.ts`. Um import, uma chamada, e ele pega os erros de combinação que de outro jeito te custariam um mint:

```typescript
// capstone/precheck.ts - run before any lamport moves: npx tsx capstone/precheck.ts
import { checkCombo } from '../check-combo';
import { loadMemo } from './memo';

const memo = loadMemo();
const combo = checkCombo(memo.declaredSet);
if (!combo.valid) {
  throw new Error(`declared set is invalid on chain: ${combo.reason}`);
}
```

**4. Calcule a seção de venue.** Esta é a parte do memo que consome a config de lançamento pelo nome, e ela é curta porque o trabalho foi feito dois módulos atrás:

```typescript
// capstone/venue.ts: the memo's launch-venue section, computed not asserted.
// Run: npx tsx capstone/venue.ts
// Consumes R10 (sprout-launch/derive-graduation.ts) by name.
import {
  PUMP_REFERENCE_CURVE,
  VENUES,
  checkGraduationVenue,
  finalReserves,
  graduationSol,
  spotPrice,
} from '../sprout-launch/derive-graduation';
import { loadMemo } from './memo';

function fmt(n: number, places = 3): string {
  return n.toLocaleString('en-US', {
    minimumFractionDigits: places,
    maximumFractionDigits: places,
  });
}

function main(): void {
  const memo = loadMemo();
  const c = PUMP_REFERENCE_CURVE;
  const f = finalReserves(c);
  const grad = graduationSol(c);
  const open = spotPrice(c.virtualSolReserves, c.virtualTokenReserves);
  const close = spotPrice(f.finalVirtualSol, f.finalVirtualToken);

  console.log('## Launch venue\n');
  console.log(`Product:            ${memo.product}`);
  console.log(`Primitive family:   ${memo.primitiveFamily}`);
  console.log(`Declared set:       ${memo.declaredSet.join(', ') || '(none)'}\n`);

  console.log('Reference curve, derived live from the published constants:');
  console.log(`  virtual SOL / virtual tokens / real tokens: ${c.virtualSolReserves} / ${fmt(c.virtualTokenReserves, 0)} / ${fmt(c.realTokenReserves, 0)}`);
  console.log(`  final virtual SOL:      ${fmt(f.finalVirtualSol)} SOL`);
  console.log(`  SOL added to graduate:  ${fmt(grad)} SOL`);
  console.log(`  price multiple:         ${fmt(close / open, 2)}x\n`);

  if (memo.primitiveFamily === 'core-asset' || memo.primitiveFamily === 'cnft') {
    console.log(
      `A ${memo.primitiveFamily} is not a fungible mint, so no curve venue applies. Say that in the memo and name where it trades instead.`,
    );
    return;
  }

  if (memo.primitiveFamily === 'spl-token') {
    console.log(
      'Declared family is classic SPL. checkGraduationVenue was written against SPROUT, a Token-2022 mint, and it hardcodes that assumption in its reason strings, so running it here prints a refusal that is about SPROUT rather than about you. A classic mint carries no extensions for a venue to refuse, so both taught venues accept it by construction. Write that sentence in the memo instead of a verdict table.',
    );
    return;
  }

  console.log('Venue verdicts for the declared set:\n');
  let anyAccepted = false;
  for (const venue of VENUES) {
    const verdict = checkGraduationVenue(venue, memo.declaredSet);
    anyAccepted = anyAccepted || verdict.accepted;
    console.log(`  ${verdict.accepted ? 'ACCEPTS ' : 'REFUSES '} ${verdict.venue}`);
    for (const reason of verdict.reasons) {
      console.log(`            ${reason}`);
    }
    console.log(`            source: ${venue.source}`);
  }

  if (!anyAccepted) {
    console.log(
      '\nNo taught venue accepts this set. That is a finding, not a bug: write it in the memo, or change the set.',
    );
  }
}

main();
```

O `npx tsx capstone/venue.ts` imprime 85.005 SOL para a curva de referência e uma linha de veredicto por venue. Se o seu brief é fungível, cole essa saída direto no `memo.md`. Se você pegou o drop do músico ou o badge de dispositivo, cole só a linha de venue-sem-curva: a derivação da curva de referência acima dela imprime para calibragem em toda rodada, e matemática de graduação em um memo de badge é exatamente o tipo de resíduo fora da spec que um revisor sinaliza.

Dois dos branches naquele arquivo existem porque um artefato reutilizado fora da premissa original dele vai mentir para você em vez de dar erro. Se você pegou o drop do músico ou o badge de dispositivo, nenhum venue de curva se aplica, e dizer isso vale mais que inventar um. E se você pegou SPL clássico, repare que o `checkGraduationVenue` não é um oráculo geral: ele foi escrito para o SPROUT, então o primeiro teste dele pergunta se o venue fala Token-2022 e a string de razão dele nomeia o SPROUT em voz alta. Aponte ele para um mint clássico e ele recusaria a pump.fun, exatamente o venue em que um mint clássico é bem-vindo. É por isso que o branch spl-token no código acima pula a chamada inteira: a sua rodada imprimiu a frase honesta em vez de uma recusa fora da spec. Isso não é um bug na função, é uma função usada fora da spec dela, e pegar isso é o tipo de coisa para o que um capstone serve.

**5. Entregue o ativo, a partir de uma receita que você já escreveu.** Nenhum mecanismo novo aqui, que é o ponto de um capstone. Mint Token-2022: a sua receita do módulo 2 com o seu conjunto declarado. Coleção Core com Royalties e impressões numeradas: a sua receita do módulo 6, e o plugin Edition é a peça para a qual o brief do músico estava apontando esse tempo todo. Badge cNFT: a sua árvore do módulo 7, dimensionada pela sua própria matemática, com folhas soulbound se o brief disser não transferível.

Escreva o endereço resultante no `memo.json` no momento em que ele aterrissar, e depois deixe o memo em paz. Editar o memo depois de ver a chain é como uma prova vira formalidade.

A devnet é o cluster certo para isso e você não deveria se sentir mal com isso. A verificação é idêntica, o estado das extensões é idêntico, e a única coisa que a mainnet acrescentaria é um custo. A única ressalva honesta é o campo de preço: um mint de devnet vai resolver, classificar como fungível, e voltar sem `price_info`, porque o conjunto precificado é mais ou menos os dez mil tokens do topo por volume de 24 horas. Esse é o caso normal e o seu script não depende disso.

**6. Ligue exatamente um trilho.** Um. Não dois porque você tem tempo. Rota de taxa: faça harvest dos valores retidos para uma tesouraria e mostre o saldo se mover. Cancela: cheque a carteira em busca da posse e recuse acesso a ela. Airdrop: o caminho de claim de Merkle com um claim real. Buyback: a chamada de swap do lado do cliente contra o venue, depois a queima. Prove em um script que imprime um antes e um depois, porque um trilho que ninguém consegue ver executar é um diagrama.

Nomeie o script conforme o trilho, `capstone/rail-fee-route.ts` ou `capstone/rail-gate.ts`, e faça ele sair com código diferente de zero quando o depois não diferir do antes. Essa última parte importa mais do que parece. Um script de trilho que loga e sempre sai com 0 vai alegremente reportar sucesso contra uma transação que nunca aterrissou, e você não vai perceber até outra pessoa rodar ele.

**7. Escreva o `capstone/verify.ts`.** O scaffold está abaixo, completo e executável. Leia o branch de três vias em `liveState` antes de rodar, porque esse branch é a lição inteira comprimida em trinta linhas: três famílias de primitiva, três definições completamente diferentes do que "estado ao vivo" sequer significa.

```typescript
// capstone/verify.ts: R13's integration proof.
// Run: npx tsx capstone/verify.ts <ASSET_ADDRESS>
// Exit 0 means: DAS resolved the asset, its interface matches the family the
// memo claims, and its live on-chain state equals the memo's declared set.
import { address, createSolanaRpc } from '@solana/kit';
import { fetchMint } from '@solana-program/token-2022';
import { das } from '../overgrowth/das';
import { classifyAsset, type DasAsset } from '../overgrowth/classify';
import { readExtensionState } from '../overgrowth/extension-state';
import { CNFT_TAGS, loadMemo, type SelectionMemo } from './memo';

const TOKEN_2022 = 'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb';
const TOKEN_CLASSIC = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

interface CapstoneAsset extends DasAsset {
  id: string;
  content?: { metadata?: { name?: string } };
  ownership?: { owner?: string };
  grouping?: { group_key?: string; group_value?: string }[];
  compression?: { compressed?: boolean; tree?: string; leaf_id?: number };
  token_info?: {
    decimals?: number;
    token_program?: string;
    price_info?: { price_per_token?: number };
  };
  plugins?: Record<string, unknown>;
}

interface LiveState {
  set: string[];
  notes: string[];
}

function expectedInterfaceFamily(memo: SelectionMemo): (asset: CapstoneAsset) => string | null {
  return (asset) => {
    const c = classifyAsset(asset);
    switch (memo.primitiveFamily) {
      case 'core-asset':
        return asset.interface === 'MplCoreAsset'
          ? null
          : `memo says core-asset, DAS says interface=${asset.interface}`;
      case 'cnft':
        return c.compressed
          ? null
          : `memo says cnft, DAS says compressed=false (interface=${asset.interface})`;
      case 'token-2022':
      case 'spl-token': {
        if (c.category !== 'fungible') {
          return `memo says ${memo.primitiveFamily}, DAS classifies this as ${c.category}`;
        }
        const want = memo.primitiveFamily === 'token-2022' ? TOKEN_2022 : TOKEN_CLASSIC;
        const got = asset.token_info?.token_program;
        if (!got) {
          // Same philosophy as the Core branch below: a silent pass on an
          // unread field is worse than no verification at all.
          return `DAS returned no token_info.token_program to check the declared family against; use an endpoint that returns it`;
        }
        if (got !== want) {
          return `memo says ${memo.primitiveFamily}, token_program is ${got}`;
        }
        return null;
      }
    }
  };
}

async function liveState(memo: SelectionMemo, asset: CapstoneAsset): Promise<LiveState> {
  if (memo.primitiveFamily === 'core-asset') {
    if (!asset.plugins) {
      throw new Error(
        'this endpoint returned no plugins object for a Core asset, so the memo cannot be checked against it. Try another DAS provider.',
      );
    }
    return { set: Object.keys(asset.plugins), notes: ['plugin names read from the DAS payload'] };
  }

  if (memo.primitiveFamily === 'cnft') {
    const set: string[] = [];
    if (asset.compression?.compressed === true) set.push('compressed');
    if ((asset.grouping ?? []).some((g) => g.group_key === 'collection')) set.push('collection');
    return {
      set,
      notes: [
        `a cNFT leaf has no extension or plugin account, so the live set is the structural tags DAS returns: ${CNFT_TAGS.join(', ')}`,
        `tree=${asset.compression?.tree ?? '(none)'} leaf_id=${asset.compression?.leaf_id ?? '(none)'}`,
        'BLIND SPOT, on record: these tags cannot see soulbound-ness. If the brief hinges on non-transferable (brief 4), this gate cannot check it; prove it with a transfer-must-fail script the way m07-l1 taught, and say so in the memo.',
      ],
    };
  }

  const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
  // Off-spec reuse, named, since venue.ts just got a lecture about exactly
  // this: for the 'spl-token' family this calls the token-2022 client's
  // fetchMint against a classic mint. That works because the 82-byte base
  // layout is shared between the two programs and a classic mint simply has
  // no TLV region, so extensions decode as None; the family check above has
  // already pinned the owning program, which is what makes the reuse safe.
  const mint = await fetchMint(rpc, address(asset.id));
  const extensions = mint.data.extensions.__option === 'Some' ? mint.data.extensions.value : [];
  const states = readExtensionState(extensions);
  return {
    set: states.map((s) => s.kind),
    notes: states.map((s) => `${s.active ? 'ACTIVE ' : 'DORMANT'} ${s.kind}: ${s.detail}`),
  };
}

function compare(declared: string[], live: string[]): { missing: string[]; extra: string[] } {
  const declaredSet = new Set(declared);
  const liveSet = new Set(live);
  return {
    missing: declared.filter((x) => !liveSet.has(x)),
    extra: live.filter((x) => !declaredSet.has(x)),
  };
}

async function main(): Promise<void> {
  const argAddress = process.argv[2];
  if (!argAddress) {
    throw new Error('usage: npx tsx capstone/verify.ts <ASSET_ADDRESS>');
  }
  const memo = loadMemo();
  if (!memo.assetAddress) {
    throw new Error('memo.json: assetAddress is empty. Ship the asset (step 5) before you verify it.');
  }
  if (memo.assetAddress !== argAddress) {
    throw new Error(
      `memo.json declares ${memo.assetAddress}, you passed ${argAddress}. Verify the thing you wrote down.`,
    );
  }

  const showFungible = memo.primitiveFamily !== 'core-asset' && memo.primitiveFamily !== 'cnft';
  const asset = await das<CapstoneAsset>('getAsset', {
    id: argAddress,
    options: { showFungible },
  });

  const c = classifyAsset(asset);
  console.log(`asset      ${asset.id}`);
  console.log(`name       ${asset.content?.metadata?.name ?? '(unnamed)'}`);
  console.log(`interface  ${asset.interface}  category=${c.category}  das-rpc-required=${c.requiresDasRpc}`);
  console.log(`owner      ${asset.ownership?.owner ?? '(n/a for a fungible mint)'}`);
  console.log(`rail       ${memo.rail}`);

  const familyError = expectedInterfaceFamily(memo)(asset);
  if (familyError) {
    throw new Error(familyError);
  }

  const state = await liveState(memo, asset);
  console.log('\nlive state');
  for (const note of state.notes) console.log(`  ${note}`);

  const { missing, extra } = compare(memo.declaredSet, state.set);
  console.log('\nmemo vs chain');
  console.log(`  declared  ${memo.declaredSet.join(', ') || '(empty)'}`);
  console.log(`  live      ${state.set.join(', ') || '(empty)'}`);
  if (missing.length > 0) console.log(`  MISSING   ${missing.join(', ')}`);
  if (extra.length > 0) console.log(`  EXTRA     ${extra.join(', ')}`);

  if (missing.length > 0 || extra.length > 0) {
    throw new Error(
      'live state does not equal the memo. Either the memo is wrong or the asset is, and only one of those is cheap to fix.',
    );
  }

  console.log(`\nOK  memo matches chain. Tradeoff on record: ${memo.tradeoff}`);
}

main().catch((err: unknown) => {
  console.error(`FAIL  ${err instanceof Error ? err.message : String(err)}`);
  process.exit(1);
});
```

Três decisões naquele arquivo valem o porquê delas, e o resto é encanamento.

A checagem `EXTRA` não é decoração simétrica. Uma extensão faltando significa que o seu mint não conseguiu o que você pediu. Uma a mais significa que você ganhou algo que nunca declarou, normalmente porque alguma receita preencheu um campo por padrão, e essa é a direção mais perigosa: um poder não declarado em um mint é exatamente a coisa que te faz ser recusado em um venue com o qual você presumiu que era compatível.

A comparação de `token_program` é um seguro barato contra o erro mais constrangedor possível, que é entregar um mint SPL clássico enquanto o seu memo diz Token-2022. O DAS te entrega o programa dono, então pergunte, e se recuse a prosseguir quando o campo voltar ausente, pela mesma razão que o branch do Core recusa um objeto plugins faltando.

E o branch do Core lança em vez de passar quando o endpoint não devolve objeto `plugins`. Uma verificação que não consegue ver a coisa que está verificando precisa falhar alto. Um passe silencioso em um campo não lido é pior que verificação nenhuma, porque você vai acreditar nele.

![Um fluxograma vertical de quatro cancelas para o script de verificação, da checagem de endereço passando por resolução DAS e correspondência de interface até a comparação de conjuntos, com branches por família nas duas últimas cancelas.](assets/v09-flowchart.png)

**8. Rode a cancela.** Aponte `DAS_RPC_URL` para um endpoint com suporte a DAS (o `RPC_URL` pode ficar na devnet para a leitura do mint) e rode:

```bash
export DAS_RPC_URL="https://<your-das-endpoint>"
npx tsx capstone/verify.ts <YOUR_ASSET_ADDRESS>
```

Uma rodada que passa imprime a linha do ativo, a interface e a categoria, o estado ao vivo, a comparação memo-versus-chain, e uma linha `OK` terminando no trade-off que você anotou. Código de saída 0. Se sair com 1, leia qual das quatro cancelas disparou, porque cada uma nomeia um erro diferente e só uma delas é bug de código.

## Challenge

Quebre a sua própria prova, de propósito, três vezes. Este é um exercício de cinco minutos e é a diferença entre uma verificação em que você confia e uma verificação que você executou.

**Um.** Adicione um nome de extensão falso ao `declaredSet` no `memo.json` e rode de novo. Você deveria vê-lo sob `MISSING` e sair com 1. Se passar, a sua comparação não está comparando.

**Dois.** Remova uma extensão real do `declaredSet` e rode de novo. Ela deveria aparecer sob `EXTRA` e sair com 1. A primeira versão desse script da maioria das pessoas só checa uma direção, e ela sempre passa, e ela é sempre inútil.

**Três.** Aponte `DAS_RPC_URL` para um RPC comum sem suporte a DAS e rode de novo. Se o seu ativo é um cNFT você vai pegar o caminho de método-não-encontrado do JSON-RPC vindo do seu próprio transporte, que é o modo de falha sobre o qual o módulo 7 te avisou. Se o seu ativo é um mint Token-2022, repare no que acontece e anote: alguns endpoints comuns respondem `getAsset` para fungíveis e alguns não, e saber o que o seu faz é um fato de portabilidade sobre a sua stack.

Depois restaure o memo e volte para uma rodada verde. Opcionalmente, e esta é a versão que vale colocar em um portfólio: escreva o memo de um segundo brief sem cunhar nada, rode o `capstone/venue.ts` contra ele, e ponha os dois memos lado a lado. Duas escolhas defensáveis para dois produtos diferentes, a partir do mesmo toolkit, é um artefato mais forte que um token publicado.

## Checkpoint

Você terminou quando quatro coisas existem juntas.

Um memo, prosa mais JSON, nomeando a família de primitiva, o conjunto de extensões ou plugins, o trade-off que você aceitou em um parágrafo, e uma seção de venue cujos números saíram do `capstone/venue.ts` e não de um post de blog. O ativo, publicado só a partir de receitas ensinadas. Um trilho, ligado e provado em um script que imprime um antes e um depois. E um `npx tsx capstone/verify.ts $ASSET_ADDRESS` verde, saída 0, com o conjunto ao vivo igual ao conjunto declarado, mais, se você pegou o brief 4, a prova de transferência-tem-que-falhar ao lado dele, porque o conjunto de tags do cNFT não consegue ver a condição soulbound e a cancela imprime esse ponto cego sozinha.

Diga a resposta de uma pergunta em voz alta antes de fechar o terminal, porque é a coisa para a qual este curso inteiro estava construindo: de qual poder você abriu mão, e o que você ganhou em troca? Se você consegue responder isso em uma frase sem olhar as suas anotações, você não só entregou um token. Você tomou uma decisão de arquitetura e deixou o recibo.

Uma nota sobre como isso se sente, já que o recuo foi total aqui e recuos totais são desconfortáveis. O desconforto é o currículo. Nove módulos de construções trabalhadas existem para que esta lição pudesse tirá-las, e se você entregou algo que resolve via DAS com um memo que bate com ele, você fez a coisa que o trabalho de fato pede.

Você entregou um produto escolhido por você e provou que ele resolve. Falta uma coisa: fechar o ciclo. Na próxima lição você re-deriva a decisão a frio, sem anotações, contra três briefs a frio: um que você nunca viu, um que refaz o formato de badge em massa que esta lição precificou mas não construiu para você, e um de propósito ao lado do memo trabalhado do café acima, porque prática de recuperação precisa de uma âncora contra a qual você consiga calibrar. Depois você mapeia para onde este toolkit te leva em seguida.
