# Token Metadata como legado: pNFTs e a realidade dos royalties

## Resumo

No m06-l2 você entregou o R7: a coleção Core Almanac verificada, ativos carregando os plugins Royalties, Edition e PermanentFreeze, e um badge Founding-Farmer soulbound, tudo no Metaplex Core, com royalties que você mesmo configurou. Esta lição é a contraparte incômoda. Você não vai entregar nada novo no Token Metadata, e no fim vai conseguir dizer com precisão por que ninguém deveria: você vai ler o padrão em que a maioria dos NFTs existentes ainda vive, decodificar o rule set emblemático que deveria impor os royalties deles, e descobrir com o seu próprio ferramental que ele hoje bloqueia zero programas. As habilidades aqui são habilidades de avaliação: ler o rule set de um pNFT ao vivo, emitir um veredicto de imposto-ou-não com evidência, e rotular `seller_fee_basis_points` por aquilo que ele é, um número que o runtime nunca toca. O recuo: a decodificação emblemática é trabalhada de ponta a ponta no lab, saída esperada incluída; a escavação do histórico de revisões e o memorando de veredicto por ativo no challenge são seus, solo, e são o trabalho de verdade.

Aqui está a situação. Um marketplace anuncia um NFT legado, `seller_fee_basis_points = 500`, e o texto do anúncio promete um royalty de criador garantido de 5%. Quinhentos basis points, on-chain, na conta de metadados. Parece que dá para impor, né? Você passou a lição passada configurando o plugin Royalties do Core, maquinaria que o próprio programa do ativo checa, embora você a tenha entregado com `ruleSet("None")` e ouvido exatamente o que isso deixa sem imposição, então a afirmação parece plausível por associação. Não é. E em vez de acreditar na minha palavra, vá tocar a evidência, a conta em que toda a história de royalties do legado se sustenta, o Metaplex Foundation Rule Set. Trinta segundos, nada para instalar, só `curl` e o `python3` que já está no seu sistema:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST -H "Content-Type: application/json" -d '
  {"jsonrpc":"2.0","id":1,"method":"getAccountInfo",
   "params":["eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9",{"encoding":"base64"}]}' \
  | python3 -c "import sys,json,base64; v=json.load(sys.stdin)['result']['value']; print('owner:', v['owner'], '| bytes:', len(base64.b64decode(v['data'][0])))"
```

Você deve ver `owner: auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg | bytes: 19001`. Esse owner é o programa Token Auth Rules, e esses 19,001 bytes guardam nove revisões do rule set que governa transferências para uma fatia enorme dos NFTs programáveis na mainnet. Eu desmontei essa conta hoje de manhã (2026-08-23), e a revisão mais recente define toda e qualquer operação, todas as catorze, como `Pass`. Armada, mas inativa. No fim do lab você mesmo vai ter decodificado isso, offsets de byte e tudo.

## A máquina de royalties que não bloqueia nada

### Legado

Primeiro, a declaração de status, porque esta lição é onde o curso a faz e o resto do curso aponta para cá. O Token Metadata é oficialmente legado; o Metaplex Core é o padrão recomendado para trabalho novo com NFT (Bubblegum v2 para comprimidos). pNFTs impõem através do Token Auth Rules, deprecado pela Metaplex e ainda assim o caminho de imposição vivo, e o rule set emblemático bloqueia zero programas, armado mas inativo; `seller_fee_basis_points` é puramente indicativo; royalties de trabalho novo são plugins do Core ou rulesets do Bubblegum v2. TM/pNFT é lido e integrado contra, nunca entregue novo.

Isso é muito veredicto em um parágrafo, então vamos merecê-lo pedaço por pedaço. A evidência mais limpa de que um padrão entrou em modo de manutenção não é um anúncio, é o trem de releases. O cliente JavaScript do mpl-token-metadata parou no v3.4.0, publicado em 2025-02-02, e o npm ainda serve 3.4.0 como `latest` hoje (eu checei o registry hoje de manhã, 2026-08-23). Dezoito meses, zero releases, na biblioteca cliente do padrão que a maioria dos NFTs da Solana de fato usa.

Seja preciso sobre a forma disso, porém, porque "abandonado" é a palavra errada e a palavra errada vai fazer você dizer algo falso numa reunião. O crate Rust não está congelado: o `mpl-token-metadata` cortou o 5.1.1 em 2025-08-18 e tem builds alpha desde então. Isso é exatamente a aparência do modo de manutenção visto de fora. O programa continua compilando contra toolchains atuais, correções entram quando precisam entrar, e nada novo é projetado. Compare o vizinho: as três linhas de release do `mpl-core`, aquelas que você leu na timeline do changelog da lição passada, estavam todas ainda subindo versões minor até meados de 2026, que é o que um código-base faz enquanto sua superfície de features ainda está crescendo. Um código-base recebe oxigênio. O outro recebe features. Observe duas bibliotecas quaisquer por dezoito meses e a que está em modo de manutenção se identifica sozinha sem nunca publicar um post de blog sobre isso.

![Linha do tempo comparando a atividade de releases em que o cliente JS do mpl-token-metadata parou no v3.4.0 em fevereiro de 2025 enquanto o mpl-core seguiu cortando releases até meados de 2026.](assets/v01-timeline.png)

Duas notas práticas antes de ir mais fundo, e as duas vão te morder se você pulá-las. Uma: legado não significa raro. A maioria dos NFTs já cunhados na Solana vive no Token Metadata, então um integrador encontra esse padrão constantemente; é exatamente por isso que o curso o ensina em profundidade de avaliação em vez de pular. Duas: a documentação mudou de casa. Os docs da Metaplex migraram de domínio, e o antigo subdomínio developers agora redireciona permanentemente para o novo hub de docs, então links em tutoriais mais antigos e respostas de Stack Exchange passam por um redirect ou morrem de vez. Quando você verificar qualquer coisa abaixo contra os docs, navegue a partir do hub atual em vez de confiar num favorito de 2023.

### O que um pNFT é de fato

O NFT programável existe por causa de uma briga. Para entender a maquinaria, você precisa do mecanismo primeiro e da história depois, então aqui está o mecanismo.

Um NFT Token Metadata comum é uma conta de token SPL mais um PDA de metadados. Nada impede o dono de transferi-lo com uma transferência simples do token program, o que significa que nada pode forçar uma transferência a passar por qualquer lógica de royalties. Um pNFT fecha esse buraco com um movimento brutal: a conta de token fica congelada todo o tempo. Não congelada como punição, congelada como arquitetura. Uma conta de token SPL congelada não pode se mover pela própria instrução de transferência do token program, ponto. O único caminho que funciona é a instrução de transferência do próprio Token Metadata, que descongela, move e recongela dentro de um único fluxo atômico, e que consulta um rule set antes de concordar em fazê-lo.

Esse rule set vive em um programa separado, o Token Auth Rules, o owner `auth9Sig...` que você viu na abertura. Uma conta de rule set mapeia nomes de operação, `Transfer:Owner`, `Delegate:Sale` e doze amigos, para regras. Uma regra pode ser um predicado de verdade (uma allow-list de programas, um composto de condições) ou a regra trivial `Pass`, que aprova tudo. Quando uma transferência de pNFT executa, o Token Metadata resolve o rule set configurado do ativo, procura a operação sendo tentada, e avalia a regra. Falhe na regra, falhe na transferência. Essa é a pilha de imposição inteira: congele tudo, canalize todo movimento por uma instrução, deixe uma conta de regras decidir.

![Diagrama de uma transferência de pNFT em que a conta de token congelada bloqueia o caminho simples, então toda transferência é canalizada pela instrução do Token Metadata e pela avaliação do seu Token Auth Rules.](assets/v02-diagram.png)

Fique um momento com o que esse funil te custa como integrador, porque este é o ponto em que um padrão abstrato se transforma num bug no seu código. Uma transferência SPL simples quer uma origem, um destino, uma autoridade e um mint. Uma transferência de pNFT quer tudo isso mais a conta de metadados, o master edition, um PDA de token record para a origem e outro para o destino, a conta do rule set, o próprio programa Token Auth Rules, e o sysvar de instruções para que o motor de regras possa ver o que mais viaja na transação. Esqueça um e você não recebe degradação graciosa, você recebe uma transação falha. E se o seu código nunca aprendeu nada disso, se ele só chama o token program do jeito que faz para todo outro ativo, ele morre no congelamento, que é a primeira parede e a menos informativa. O erro diz que a conta está congelada. Um dev que não sabe que pNFTs existem vai então passar uma tarde caçando quem a congelou. Ninguém a congelou. Ela nasceu assim.

Agora o paradoxo que você precisa segurar sem se abalar, porque ele derruba quase todo mundo que lê os docs com pressa: o próprio hub de desenvolvedores da Metaplex lista o Token Auth Rules como deprecado, e o Token Auth Rules ainda é o caminho de imposição vivo para todo pNFT que existe. As duas afirmações são verdadeiras ao mesmo tempo. Deprecado significa "não construa coisas novas sobre isso"; não significa que o programa foi desligado. O programa está deployado, os rule sets resolvem, o Token Metadata ainda chama nele em toda transferência de pNFT hoje. Se você internalizar um hábito deste curso, que seja este: a deprecação é uma recomendação sobre o futuro, não uma afirmação sobre o presente. Você lê a chain para aprender o presente.

### Por que os royalties precisaram de toda essa maquinaria

Hora de derivar o design em vez de memorizá-lo, porque a derivação é o que te diz onde ele quebra. Comece do status quo, 2021: a conta de metadados de um NFT carrega `seller_fee_basis_points`, um u16, em que 500 significa 5%. O que o runtime da Solana faz com esse número? Nada. Não é uma tabela de taxa, não é um parâmetro de protocolo, não é nada que o caminho de transferência leia. É um bilhete pregado no ativo dizendo "o criador gostaria de 5%". Marketplaces leram o bilhete e, por um tempo, o honraram voluntariamente.

Então a pergunta motivadora se escreve sozinha: se o campo é só um pedido, o que acontece quando alguém recusa o pedido? Exatamente o que você preveria. Marketplaces de royalty zero e de royalty opcional apareceram em 2022, rotearam trades em volta da taxa por construção, e o volume seguiu o desconto. Criadores viram sua linha de receita se aproximar de zero em ativos cujos metadados ainda prometiam 5%, porque a promessa nunca foi estrutural.

Caminhe pelas tentativas de correção em níveis, do jeito que o ecossistema de fato as caminhou. Correção ingênua um: pedir com jeitinho aos marketplaces, talvez remover coleções de agregadores que pulam royalties. Pressão social funciona até a economia crescer mais que ela; não se sustentou. Correção ingênua dois: fazer o contrato do marketplace impor a taxa. Mas o marketplace é a parte com o incentivo de pulá-la, e um vendedor pode sempre usar um contrato diferente, ou uma transferência simples de carteira para carteira disfarçada de venda. Imposição pelos dispostos não é imposição. O que estreita a pergunta à sua forma real: royalties só são imponíveis se o próprio ativo puder se recusar a se mover exceto por programas que pagam. E "o ativo se recusa a se mover" na Solana significa que a conta de token está congelada e algo com autoridade de descongelamento medeia toda transferência. Esse requisito estreitado força essencialmente todo o design de pNFT que você acabou de ler: congelamento permanente, uma instrução de transferência obrigatória, e um motor de regras decidindo quais chamadores são aceitáveis. O design não é barroco por diversão; é a forma mínima que satisfaz "o ativo se recusa".

![Fluxograma derivando o design do pNFT, em que correções falhas estreitam o problema até o ativo se recusar a se mover, forçando a arquitetura de congelamento mais rule set e deixando aberto quem mantém a lista.](assets/v03-flowchart.png)

Note a pergunta residual em que o fluxograma termina, porque ela é a dobradiça de toda esta lição. O rule set é uma lista de quem pode mover o ativo. Alguém tem que manter essa lista, defendê-la, atualizá-la conforme marketplaces aparecem e morrem, e absorver a política de excluir um venue. Para a maioria das coleções de pNFT esse alguém é a Metaplex, via o Metaplex Foundation Rule Set compartilhado que você sondou na abertura. O mecanismo é sólido. A pergunta sempre foi se a lista continuaria povoada. Você já sabe a resposta, você a viu na conta, mas vamos fazer a leitura direito.

### Lendo o rule set emblemático

Aqui está o que esses 19,001 bytes de fato são, caminhados campo por campo do jeito que você vai decodificá-los no lab. O byte 0 é um discriminador de conta, chave `1`, significando RuleSet. Os bytes 1 a 8 são um u64 little-endian guardando o offset em bytes do mapa de revisões, que vive na cauda da conta. Tudo entre o header e o mapa é uma pilha de revisões: um rule set é append-only, toda atualização empurra uma nova revisão completa, e o mapa no fim registra onde cada revisão começa. Esta conta guarda nove revisões, e uma nota de indexação antes de qualquer número: esta lição conta revisões começando em zero, rev0 até rev8, então a "revisão 8" É a nona e mais recente entrada, e uma futura décima entrada seria a revisão 9. A mais recente começa com um único byte `lib_version` (aqui `1`, significando que o corpo da revisão é serializado em msgpack), seguido por uma estrutura de quatro elementos: a versão de novo, a pubkey do owner, o nome legível por humanos, e o mapa de operações. O nome nesta conta diz, literalmente, `Metaplex Foundation Rule Set`.

E o mapa de operações, revisão mais recente, decodificado ao vivo na hora da escrita (2026-08-23):

![Tabela de todas as catorze operações na revisão mais recente do rule set emblemático, cinco operações Transfer e nove operações Delegate, toda e qualquer uma mapeada para a regra Pass.](assets/v04-table.png)

Catorze operações. Catorze `Pass`. Uma regra `Pass` aprova qualquer chamador incondicionalmente, então este rule set, avaliado em toda transferência de todo pNFT que aponta para ele, não bloqueia nada. A maquinaria roda: a conta resolve, o congelamento se sustenta, o Token Metadata obedientemente chama o Token Auth Rules em cada transferência, o motor de regras avalia, e a avaliação sempre tem sucesso. A imposição está armada mas presentemente inativa. Essa frase é a que se deve carregar para fora desta lição, porque as duas metades importam para um integrador: armada significa que você ainda precisa tratar o caminho de transferência de pNFT corretamente ou suas transferências falham de cara; inativa significa que você não deve dizer a ninguém que royalties são garantidos por ela.

### Armada, depois inativa: o que as revisões lembram

O design append-only tem um presente para nós: a conta lembra sua própria história, e a história é onde a coisa deixa de ser abstrata. Decodifique todas as nove revisões (o challenge te faz exatamente isso) e um arco limpo aparece. As revisões iniciais são árvores de regras de verdade. Operações como `Transfer:SaleDelegate` carregam regras compostas, condições estruturadas com allow-lists de programas embaixo delas em vez de uma aprovação geral, e fallbacks de namespace roteiam operações não listadas para regras base. Esta era a era da imposição de royalties em carne e osso: uma lista mantida de programas aprovados, com todo o resto recusado. Então a revisão 7 vira quase todo o mapa para `Pass`, deixando uma única operação com uma regra de verdade. A revisão 8 atual aposenta esse último resistente. Catorze de catorze.

![Gráfico das nove revisões do rule set, em que zero a seis carregam quinze ou dezesseis regras de verdade, a revisão sete cai para uma, e a mais recente carrega zero.](assets/v05-chart.png)

Um detalhe que o seu decodificador silenciosamente pula. O segundo elemento de toda revisão é o owner do rule set, a pubkey autorizada a empurrar a revisão nove. A desestruturação no lab o joga fora com uma vírgula vazia, o que é um default aceitável e um hábito ruim. Decodifique-o quando você estiver auditando de verdade. Um rule set não é uma constituição, é uma conta com uma autoridade de atualização, e saber quem detém essa autoridade te diz exatamente quão durável é a postura de imposição que você acabou de medir. Para o emblemático, essa autoridade é a Metaplex. Para uma coleção que fez o seu próprio rule set, pode ser um keypair no laptop de alguém.

Segure a honestidade aqui, porque ela corta para os dois lados. Isto não é evidência de que a imposição de royalties nunca funcionou; as revisões iniciais provam que funcionou, estruturalmente, por todo o tempo em que a lista foi mantida. É evidência de que a imposição era uma política expressa através de uma conta que uma autoridade controla, e a política mudou. A própria documentação da Metaplex é refrescantemente direta sobre o estado atual: ela admite que este rule set hoje não nega programa nenhum. Essa é a história inteira em uma oração, porque um rule set que não nega ninguém não pode fazer ninguém pagar. Todo mundo citando basis points; um impositor emblemático liberando tudo. Vou admitir que esta me machucou pessoalmente: eu escrevi um script de cunhagem na era do pNFT que logava `seller_fee_basis_points` sob o rótulo `royalty` como se a palavra fosse estrutural, e nada na minha stack nunca checou se algo o impunha. A maior parte do ferramental do ecossistema ainda imprime esse campo do mesmo jeito que o meu imprimia.

Então o que é `seller_fee_basis_points`, dito com exatidão? Um u16 na conta de metadados, denominado em basis points, que o programa Token Metadata armazena e serve e nunca gasta. O runtime não o desconta. O Token Metadata não o desconta. Na era da imposição, o rule set também não o descontava; a imposição funcionava recusando entrada a venues que não pagavam royalties, não coletando a taxa em si, e pagar o valor sempre foi o código do marketplace honrando o campo. Hoje, com as regras emblemáticas inativas, o campo é exatamente o que era em 2021: um pedido on-chain. Quando aquele marketplace hipotético promete 5% garantidos porque o campo diz 500, a leitura correta é: advisório. Indicativo apenas, honrado a critério de cada venue, imposto por nada que você possa apontar. Se uma contraparte quiser provar o contrário, o ônus está a uma decodificação de distância, e agora a ferramenta é sua.

### O enum que cresceu mais que a documentação

Mais uma aresta afiada, menor mas muito a mesma lição em miniatura. Toda conta Token Metadata carrega um valor `TokenStandard` dizendo aos integradores que tipo de ativo eles têm. A página de docs lista cinco variantes. O código-fonte do programa define seis: `NonFungible`, `FungibleAsset`, `Fungible`, `NonFungibleEdition`, `ProgrammableNonFungible`, e a que a página de docs nunca menciona, `ProgrammableNonFungibleEdition`. Um pNFT pode ter edições, edições de NFTs programáveis precisam do seu próprio valor de padrão, e o código cresceu a variante enquanto a página de docs ficou parada.

![Comparação mostrando o enum TokenStandard, os docs listam cinco variantes enquanto o código define seis, com ProgrammableNonFungibleEdition presente só no fonte.](assets/v06-comparison.png)

Contra qual você integra? O código, sem hesitar, e o raciocínio generaliza para todo sistema legado que você algum dia vai tocar. Documentação é um artefato mantido; modo de manutenção significa que ela para de ser mantida no mesmo relógio que o código, e o código é o que executa contra a sua transação. Monte um match a partir dos cinco documentados e a sexta variante chega no seu pipeline como um valor não tratado no pior momento possível, em produção, dentro da transferência de alguém. Esta é a mesma epistemologia do paradoxo deprecado-mas-vivo e a mesma epistemologia do próprio rule set: a chain e o fonte são o tempo presente; prosa sobre eles é o pretérito de quando alguém editou por último.

### A postura do integrador

Agora monte o veredicto numa postura de trabalho, porque o trade-off aqui é assimétrico e a assimetria é o ponto. O Token Metadata legado ainda é o padrão de NFT mais amplamente integrado na Solana, então você precisa ser capaz de lê-lo e integrar contra ele: o seu marketplace, carteira, índice ou jogo vai receber ativos TM e pNFTs por anos, e um pNFT mal tratado (digamos, tentar uma transferência simples de token contra uma conta permanentemente congelada) falha feio. Mas entregar pNFTs novos significa adotar uma pilha de imposição deprecada cuja regra emblemática hoje não impõe nada, apostando o seu produto em maquinaria de que o próprio fornecedor se afastou. E tratar `seller_fee_basis_points` como uma taxa garantida é simplesmente errado, do jeito que eventualmente se torna um ticket de suporte, um criador irritado, ou uma questão jurídica. O custo da compatibilidade mais ampla é um padrão que a própria Metaplex não recomenda mais. Então a postura: ler, criticar, integrar, nunca entregar novo. Os royalties do seu trabalho novo vivem onde você os construiu na lição passada, no plugin Royalties do Core, e onde o próximo módulo te leva para ativos comprimidos, rulesets do Bubblegum v2.

E quando um cliente te fizer a pergunta direta, e ele vai, a resposta honesta tem três partes e leva cerca de um minuto. A imposição on-chain de royalties para o Token Metadata legado não está acontecendo hoje, e você pode mostrar a decodificação em vez de afirmar, o que muda toda a temperatura da conversa. A imposição para trabalho novo está genuinamente disponível através do plugin Royalties do Core, cujas regras de allow-list e deny-list são uma checagem que o próprio programa do ativo roda, não uma conta de regras que outra pessoa tem que ficar mantendo. E nenhum padrão em nenhuma chain impede duas partes que querem liquidar fora do venue. O que a imposição de royalties compra é atrito contra o caminho casual, não uma lei. Diga isso em voz alta no começo e ninguém volta irritado em seis meses.

![Árvore de decisão em que ativos Token Metadata e pNFT existentes são lidos e integrados com um veredicto de imposição, enquanto drops novos levam royalties para plugins do Core ou rulesets do Bubblegum v2.](assets/v07-flowchart.png)

## Lab: decodifique você mesmo o rule set emblemático

O `curl` da abertura provou que a conta existe. Agora você constrói o decodificador que transforma esses bytes num veredicto, a mesma leitura que eu fiz para a tabela acima. Isto é deliberadamente leve em dependências: uma biblioteca msgpack, o `fetch` embutido do Node, nenhum SDK da Metaplex, porque o ponto é que você pode auditar a história da imposição a partir de bytes brutos mesmo que toda biblioteca cliente desapareça.

1. Prepare um workspace. Você precisa do Node (o piso do curso não mudou desde o m01-l1: Node 20 ou mais novo, qualquer um com `fetch` embutido) e de exatamente um pacote:

   ```bash
   mkdir ruleset-audit && cd ruleset-audit
   npm init -y
   npm install @msgpack/msgpack
   ```

   Isso instala o `@msgpack/msgpack` 3.1.3 no momento desta escrita (2026-08-23); qualquer 3.x funciona, é uma biblioteca de formato estável. Msgpack, se você não a conhece, é uma prima binária compacta do JSON, e é com ela que o formato de revisão V1 do rule set serializa.

2. Escreva o decodificador. Crie `decode-ruleset.mjs`:

   ```javascript
   // decode-ruleset.mjs <rule-set-address> [rpc-url]
   // Reads a Token Auth Rules RuleSet account and prints the LATEST revision's
   // operation map, then renders an enforcement verdict.
   import { decode } from "@msgpack/msgpack";

   const address = process.argv[2] ?? "eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9";
   const rpc = process.argv[3] ?? "https://api.mainnet-beta.solana.com";

   const res = await fetch(rpc, {
     method: "POST",
     headers: { "Content-Type": "application/json" },
     body: JSON.stringify({
       jsonrpc: "2.0", id: 1, method: "getAccountInfo",
       params: [address, { encoding: "base64" }],
     }),
   });
   const { result } = await res.json();
   if (!result.value) throw new Error("account not found: " + address);

   const buf = Buffer.from(result.value.data[0], "base64");
   console.log("owner program:", result.value.owner);
   console.log("account size :", buf.length, "bytes");

   // Header: byte 0 is the account key (1 = RuleSet), bytes 1..9 are a u64
   // pointing at the revision map that lives at the END of the account.
   const revMapLoc = Number(buf.readBigUInt64LE(1));
   const revCount = buf.readUInt32LE(revMapLoc + 1);
   const offsets = [];
   for (let i = 0; i < revCount; i++) {
     offsets.push(Number(buf.readBigUInt64LE(revMapLoc + 5 + 8 * i)));
   }
   console.log("revisions    :", revCount, "(latest wins)");

   // Latest revision: 1 lib_version byte, then a msgpack-serialized RuleSetV1:
   // [lib_version, owner_pubkey_bytes, name, { operation -> rule }].
   const start = offsets[revCount - 1];
   const libVersion = buf[start];
   if (libVersion !== 1) throw new Error("not a msgpack (V1) revision: lib " + libVersion);
   const [, , name, operations] = decode(buf.subarray(start + 1, revMapLoc));

   console.log("rule set name:", name);
   const ruleKind = (rule) => (typeof rule === "string" ? rule : Object.keys(rule)[0]);
   let blocking = 0;
   for (const [op, rule] of Object.entries(operations)) {
     const kind = ruleKind(rule);
     if (kind !== "Pass") blocking++;
     console.log(`  ${op.padEnd(28)} ${kind}`);
   }
   console.log(
     blocking === 0
       ? `VERDICT: ${Object.keys(operations).length} operations, ALL Pass. Armed but idle: nothing is blocked, royalties are NOT enforced by this rule set.`
       : `VERDICT: ${blocking} operation(s) carry real rules. Enforcement is live for those paths.`
   );
   ```

   Leia a estrofe do meio antes de rodar, porque os offsets são a verdadeira aula de anatomia. O header aponta para frente, para o mapa, o mapa aponta para trás, para cada revisão, e o decodificador não confia em mais nada: nenhuma IDL, nenhuma biblioteca cliente, só o layout. O helper `ruleKind` cobre as duas formas que uma regra assume no msgpack decodificado: as regras triviais chegam como strings simples (`"Pass"`) e toda regra estruturada chega como um objeto de chave única cuja chave nomeia o tipo da regra.

3. Rode contra o emblemático:

   ```bash
   node decode-ruleset.mjs
   ```

   Saída esperada, e isto é literalmente o que a conta retornou em 2026-08-23:

   ```text
   owner program: auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg
   account size : 19001 bytes
   revisions    : 9 (latest wins)
   rule set name: Metaplex Foundation Rule Set
     Transfer:WalletToWallet      Pass
     Transfer:Owner               Pass
     Transfer:MigrationDelegate   Pass
     Transfer:SaleDelegate        Pass
     Transfer:TransferDelegate    Pass
     Delegate:LockedTransfer      Pass
     Delegate:Update              Pass
     Delegate:Transfer            Pass
     Delegate:Utility             Pass
     Delegate:Staking             Pass
     Delegate:Authority           Pass
     Delegate:Collection          Pass
     Delegate:Use                 Pass
     Delegate:Sale                Pass
   VERDICT: 14 operations, ALL Pass. Armed but idle: nothing is blocked, royalties are NOT enforced by this rule set.
   ```

   Se a sua contagem de revisões ou uma regra diferir da minha, não assuma que você quebrou algo: esta é uma conta ao vivo com uma autoridade de atualização, e uma nova revisão pode ter entrado entre a minha decodificação e a sua. Essa possibilidade é a lição, não uma nota de pé de página nela. A postura de imposição de uma fatia enorme do ecossistema de pNFT (ninguém publica uma contagem exata, e este curso não vai inventar uma) está a uma transação de mudar, em qualquer direção, e a única resposta atual é a que você acabou de puxar.

4. Agora emita o veredicto de avaliação nas suas próprias palavras, em voz alta ou num arquivo de rascunho, na forma que este módulo avalia: imposto ou não, com a evidência do conjunto de operações, mais a razão pela qual o campo de basis points não pode ser confiado. O meu diz: "Não imposto. O rule set do ativo mapeia todas as catorze operações governadas para Pass, então todo caminho de transferência é aprovado incondicionalmente; `seller_fee_basis_points` é um pedido armazenado que nenhum programa no caminho de transferência lê ou coleta, então não pode funcionar como uma taxa." Se a sua versão nomeia a evidência e rotula o campo, você tem a habilidade que esta lição existe para instalar.

5. Checkpoint. Você deve ter agora: um `decode-ruleset.mjs` funcionando que aceita qualquer endereço de rule set, a decodificação do emblemático no seu próprio terminal, e um veredicto escrito com evidência. O decodificador é uma ferramenta de verdade, não uma demo; guarde-o no workspace do seu curso, porque o challenge o aponta para a história em seguida, e o você do futuro vai apontá-lo para qualquer coleção de pNFT que um cliente te entregar.

## Challenge

O lab decodificou o presente. O challenge decodifica o passado, e então faz a chamada para a qual o brief vem construindo. Primeiro, estenda o `decode-ruleset.mjs` para caminhar por todas as nove revisões em vez de só a última: você já coleta todo offset, então corte cada revisão do seu offset até o próximo (a revisão final termina onde o mapa de revisões começa), decodifique cada uma, e imprima uma linha por revisão com sua contagem de operações não-`Pass`. Confira o byte `lib_version` por revisão antes de decodificar e pule, com uma nota, qualquer revisão que não seja versão 1; esta conta é toda msgpack hoje, mas a sua ferramenta não deveria assumir que todo rule set é. A sua saída deve reproduzir o arco do gráfico: árvores de regras de verdade até a revisão 6, uma única regra viva na revisão 7, zero na revisão 8. Enquanto você estiver lá, expanda a regra `Transfer:SaleDelegate` de uma revisão inicial e olhe como a imposição de fato se parecia estruturalmente, uma árvore de regras composta onde um `Pass` geral agora se senta.

![Layout de bytes de uma conta RuleSet em que o header aponta para um mapa de revisões na cauda que aponta de volta para nove revisões empilhadas, cada uma começando com um byte lib_version.](assets/v08-diagram.png)

Depois o memorando de veredicto, três frases, o entregável que um integrador de fato passaria para um time. Frase um: se este rule set hoje impõe royalties, com a evidência das operações. Frase dois: o que o histórico de revisões mostra que ele fazia, e o que isso implica sobre confiar no estado atual de qualquer rule set. Frase três: para um ativo pNFT legado que o seu produto poderia encontrar (escolha qualquer coleção de pNFT que você conheça, ou só raciocine a partir do emblemático, já que uma fatia enorme aponta para cá), a chamada de padrões: ler-e-integrar, e sob que condições você algum dia entregaria algo novo nesta pilha. Se a sua terceira frase encontrou uma condição para entregar pNFTs novos, releia a lição; a resposta honesta é que não existe uma, e dizer isso com evidência é o entregável.

Se a sua decodificação do emblemático discordar dos números impressos nesta lição, uma décima revisão, uma operação que não é mais `Pass`, um nome de rule set que mudou, sinalize no canal de feedback do curso com a sua saída bruta e a data. As minhas decodificações estão estampadas em 2026-08-23 e esta é uma conta ao vivo sob uma autoridade ativa; um aprendiz que a pega derivando está fazendo exatamente o trabalho de ler-a-chain que esta lição ensina, e o curso vai atualizar a partir da sua evidência.

Você já consegue ler o padrão de NFT mais amplamente deployado na chain, julgar sua imposição honestamente, e entregar um ativo Core cuja maquinaria de royalties vive em um plugin que o próprio programa do ativo checa, armado com `ruleSet("None")` por ora, exatamente como a lição passada admitiu. Mas e um milhão de crates Harvest? Um milhão de ativos Core é um milhão de contas, e a aritmética de rent não se importa com o seu roadmap. Próximo módulo: compressão de estado, Bubblegum v2, e ler ativos de volta quando quase nada está armazenado on-chain.
