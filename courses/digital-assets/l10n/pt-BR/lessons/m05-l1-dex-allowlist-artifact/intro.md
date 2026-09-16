# A allowlist da DEX como artefato de ensino

## Resumo

Na lição passada você construiu uma variante confidencial do SPROUT: auditor key, registro ElGamal, uma transferência encriptada em múltiplas transações, e depois a arquivou como um branch de emissor especializado. Agora de volta ao SPROUT principal, o que você de fato quer que as pessoas negociem.

Aqui está o momento para o qual esta lição existe. Você passou quatro módulos transformando o SPROUT exatamente no token que você queria: taxas que financiam uma tesouraria, metadados nativos que a carteira lê sem indexador, e ao lado dele a variante com hook cujo programa loga todo harvest. Depois você vai semear uma pool da Raydium com a variante com hook, para que o SPROUT com cancela finalmente possa ser negociado, e a transação de criação da pool reverte. Nenhum stack trace apontando para o seu código, porque o seu código está bom. O programa da Raydium leu a lista de extensões daquele mint e o recusou de propósito, e os docs da Raydium dizem a parte silenciosa em voz alta sobre exatamente o porquê. (Mantenha as duas variantes distintas durante toda a lição: o SPROUT principal leva taxa mais metadados e vai dar tudo certo; a variante com hook é a que está na porta.)

Antes da autópsia, suba o lab. O mesmo surfnet do módulo 1 (surfpool 1.2.1 aqui, checado em 2026-08-22; `brew install txtx/taps/surfpool` no macOS, outras plataformas compilam a partir da página de releases). O surfpool forka a mainnet, o que importa hoje: o programa Token-2022 real e o deployment real do CP-Swap estão os dois carregados.

```bash
mkdir -p labs/m05-l1 && cd labs/m05-l1
npm init -y && npm pkg set type=module
surfpool start --no-tui --no-studio
```

Um fato de continuidade antes que ele te custe uma hora: um fork é efêmero. Contas de mainnet ele puxa preguiçosamente sob demanda, mas os mints que VOCÊ criou existem só em um surfnet que ficou de pé desde que você os criou. Se o seu reiniciou (ou se este comando acabou de subir um novo), re-cunhe os dois locais de que o passo 5 precisa antes de chegar lá. Os dois comandos rodam a partir da RAIZ do workspace, então dê `cd` de volta para fora de `labs/m05-l1` primeiro: `npx tsx labs/m02-l4/add-metadata.ts` recria o SPROUT nomeado e reescreve `sprout-mint.json`, e a cerimônia de m03-l2 (`spl-token create-token --program-2022 --decimals 6 --transfer-hook $HOOK` depois de re-deployar o hook) recria a variante com hook. Cinco minutos, e os endereços mudam, o que é tranquilo; tudo aqui recebe endereços como argumentos.

Você já roçou nessa allowlist duas vezes: m02-l1 a citou quando o conjunto de economia acabou ficando inteiro dentro dela, e o encerramento do módulo confidencial te apontou de volta para ela. Hoje a gente para de citar e lê o código que a impõe. Negociabilidade não é um vibe e não é um ticket de suporte: é uma allowlist específica morando no código que a DEX de fato roda, e você consegue ler isso em uns noventa segundos. Você vai ler a allowlist de cinco extensões do CP-Swap da Raydium direto do fonte, aprender os três bypasses documentados que a tornam mais sutil do que "nenhuma extensão permitida," e construir o primeiro rascunho do R6, um preditor de roteabilidade que reproduz a decisão de aceitar-ou-rejeitar do programa a partir do perfil de um mint. Depois você o roda contra o seu próprio SPROUT e a variante com hook dele em um fork de mainnet e vê se a sua previsão sobrevive ao contato.

O recuo da ajuda, dito em voz alta: a leitura do fonte e o primeiro bypass são um walkthrough resolvido, eu digito junto com você. O preditor é um problema de completion, você recebe o arquivo com dois buracos e a teoria te diz o que vai neles. O coding challenge é solo, sem apoio, a regra completa de aceitar-e-rejeitar a partir de um perfil arbitrário de mint.

Deixe esse surfnet rodando e abra um segundo terminal, porque a primeira coisa que a gente faz é ler o Rust de outra pessoa.

## Lendo a regra que decide se o seu token é negociado

### As cinco que passam

A AMM de produto constante da Raydium mora no repositório `raydium-cp-swap`, e toda a política de Token-2022 é uma função em um arquivo. Clone e olhe:

```bash
git clone https://github.com/raydium-io/raydium-cp-swap.git
cd raydium-cp-swap
git rev-parse --short HEAD
sed -n '18,23p' programs/cp-swap/src/utils/token.rs
sed -n '225,236p' programs/cp-swap/src/utils/token.rs
```

No commit `244e124` (subido em 2026-08-19, que é o que eu li em 2026-08-22) esses dois trechos são a parte que todo mundo cita. O segundo caminha pela lista de extensões do mint e retorna `Ok(false)` no momento em que encontra uma extensão fora de um conjunto fixo de cinco. O primeiro é um array hardcoded de quatro endereços de mint que pulam a caminhada inteira. Imprima a sua própria saída de `git rev-parse` e anote, porque um número de linha em uma lição é uma promessa com prazo de validade curto.

As cinco que passam, na ordem do próprio programa: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`, `InterestBearingConfig`, `ScaledUiAmount`. É essa a lista. Toda outra extensão do catálogo que você passou três módulos construindo, cada uma delas, reverte a criação de pool neste venue.

Olhe o que essas cinco têm em comum antes de olhar o que está faltando. Uma taxa de transferência move valor, mas move por uma regra declarada no TLV do próprio mint, a uma alíquota que uma pool consegue ler e precificar em volta. A referência de Token-2022 da Raydium é explícita sobre como ela lida com isso: a matemática da pool subtrai a taxa de entrada, e o programa Token-2022 cuida da de saída. Interest-bearing é ainda mais mansa, já que a pool contabiliza em valores de principal e o multiplicador de UI é só decorativo. Scaled UI é exibição e nada mais. Metadata pointer e token metadata são strings e um endereço. Nenhuma das cinco consegue rodar código, ter uma chave sobre o saldo de outra pessoa, ou tornar um número ilegível.

![Comparação das cinco extensões Token-2022 que o CP-Swap da Raydium aceita contra as seis que ele recusa, cada recusa carregando a justificativa publicada pela própria Raydium.](assets/v01-comparison.webp)

Agora ponha isso contra o modelo ingênuo que a maioria das pessoas carrega, o que eu carreguei por mais tempo do que eu gostaria de admitir: "tokens Token-2022 não são negociados." Esse modelo está errado nas duas direções ao mesmo tempo. Cinco extensões roteiam tranquilo, então um mint Token-2022 que leva taxa, que carrega metadados e que acumula juros é um ativo de pool perfeitamente comum. E um mint com uma única extensão fora da lista não é negociado um pouco pior, ele não cria a pool de jeito nenhum. A falha é binária e acontece na criação, não na hora do swap.

Não aceite a minha lista por fé, tampouco. De dentro do clone, isto imprime os nomes que o arquivo de fato casa hoje:

```bash
sed -n '225,236p' programs/cp-swap/src/utils/token.rs | grep -o 'ExtensionType::[A-Za-z]*'
```

Se a forma derivou desde que eu li, esse comando te diz isso em uma linha, que é exatamente o hábito que esta lição está tentando instalar.

Um detalhe de implementação carrega peso diagnóstico de verdade, então repare nele enquanto o arquivo está aberto. A checagem de suporte não lança quando ela te recusa. Ela retorna um booleano, `Ok(false)`, e quem chama transforma isso na falha de criação da pool. O que chega ao seu terminal é o genérico do venue, "este mint não é suportado," não "a sua entrada TransferHook é o problema, todo o resto estava bom." Você já encontrou essa lacuna antes: em m01-l4 você descobriu todas as cinco regras de combinação do Token-2022 retornando o erro idêntico, então o programa conseguia te dizer que você quebrou uma regra mas nunca qual. Mesma arquitetura, mesmo silêncio, uma camada acima. E é o argumento inteiro para construir um preditor local em vez de aprender por revert. Às 2 da manhã, a distância entre "mint não suportado" e "tire o hook ou leve isso para a Meteora" é a distância entre um ticket de suporte e uma correção.

### Por que a linha cai exatamente ali

A pergunta interessante não é o que a lista contém. É por que um time de engenheiros de AMM desenhou a fronteira naquele lugar específico, porque, uma vez que você consegue derivar o raciocínio deles, você consegue prever a lista do próximo venue antes de ler.

Comece pelo que uma pool é, mecanicamente. Uma pool do CP-Swap é um programa que custodia duas contas de token, os vaults, e precifica swaps contra os saldos delas. (Essa única frase é toda a AMM de que a gente precisa. Matemática de pool, mecânica de tick e bin e estratégia de LP pertencem ao curso planejado DeFi and RWA Engineering; o que esta lição empresta é só o assento da pool na mesa.) Tudo o que ela consegue admitir com segurança sai de dois requisitos: ela tem que conseguir computar um preço a partir de um saldo, e ela tem que conseguir confiar que um saldo que ela guarda continua guardado.

Rode as regras candidatas ingênuas contra isso e veja elas falharem. Regra um: recuse qualquer coisa que mude os valores. Errado, porque TransferFeeConfig muda valores e passa; a pool consegue computar em volta de uma alíquota declarada. Regra dois: recuse qualquer coisa que toque nos números que uma UI mostra. Também errado, já que interest-bearing e scaled UI reescrevem os dois o número exibido e passam os dois; a pool lê valores crus por baixo e trata o multiplicador como decoração. Regra três: recuse qualquer coisa não auditada. Essa está mais perto, e é literalmente a razão declarada para os ponteiros de group e de member ("não revisados"), mas ela não explica por que um programa de hook bem auditado continua recusado.

![Três regras candidatas de allowlist, cada uma riscada pela extensão que a refuta, descendo até a regra sobrevivente baseada em capacidade em contraste total.](assets/v02-comparison.webp)

O que sobrevive é mais estreito, e é a frase para a qual este curso inteiro vem caminhando. Uma DEX admite extensões que só mudam exibição ou raspam uma taxa declarada, e recusa extensões que deixam alguém rodar código arbitrário dentro da transferência ou mover tokens que a pool está guardando.

Leia as três recusas emblemáticas com essa regra na mão. `PermanentDelegate` é recusada porque, nas palavras da Raydium, "um holder do delegado pode varrer qualquer conta de token, incluindo o vault da pool." O vault é o inventário da pool. Uma extensão cujo propósito inteiro é uma chave que consegue mover o saldo de qualquer pessoa é, do assento da pool, um risco de inventário impossível de hedgear. `ConfidentialTransfer` é recusada porque "valores encriptados impedem a precificação," que é o mesmo argumento que você derivou do outro lado no módulo confidencial: uma AMM que não consegue ler um valor não consegue cotar um preço. E `TransferHook` é recusada porque ela "invoca um programa customizado em toda transferência, com consumo arbitrário de CU."

Essa última merece uma pausa, porque é a que as pessoas entendem ao contrário. O hook não consegue roubar da pool. Você sabe disso do módulo 3: toda conta da transferência original é rebaixada para somente-leitura dentro do hook, então o programa de hook não consegue mover fundos, e o próprio guia de desenvolvedor da Solana diz isso. A recusa não é sobre roubo. É sobre custo e sobre encanamento. Todo programa que move um token com hook tem que resolver a lista de contas extras do mint e encaminhar essas contas em cada instrução de transferência, e o hook então queima um número ilimitado de compute units dentro do orçamento do swap. Uma pool que admite um mint com hook se voluntariou para carregar a resolução de contas de um estranho e o custo de compute de um estranho em todo swap, para sempre, sem pin de versão e sem limite superior. Recusar é um orçamento de compute com um nome nele.

![Diagrama mapeando seis extensões Token-2022 em três invariantes de pool, mostrando qual invariante cada extensão recusada quebra e por que as admitidas não quebram.](assets/v03-diagram.webp)

Duas objeções valem resposta aqui, porque qualquer engenheiro que já entregou uma AMM levanta as duas.

A primeira: por que não simplesmente limitar o compute do hook e admiti-lo? Porque o teto está no lugar errado. O orçamento de compute de uma transação pertence à transação, e o hook gasta do mesmo envelope de que o swap gasta, então uma pool que quer ser segura tem que reservar folga para o programa de um estranho em toda cotação. Essa folga não é grátis, ela é ou uma cotação pior ou um swap que morre no limite quando o hook decide fazer mais trabalho do que fez ontem. Um venue que admite dez mints com hook tem dez orçamentos desconhecidos diferentes contra os quais reservar.

A segunda, mais afiada: por que não simular uma transferência e ver quanto o hook custa de verdade? Porque simulação responde uma pergunta sobre o passado. Ela te diz o que aquele programa fez uma vez, contra o bytecode deployado naquele slot, com a lista de contas extras como ela estava. Programas de hook são atualizáveis por quem tem a autoridade de upgrade, e a lista de contas extras é estado de conta que a autoridade dela consegue reescrever. Então uma allowlist chaveada por comportamento observado é uma allowlist que uma transação de upgrade consegue invalidar silenciosamente, em um momento em que ninguém está olhando. Chaveá-la por capacidade em vez disso é desconfortável, grosseiro e estável, e estabilidade é para o que um programa que guarda a liquidez de estranhos está otimizando.

Vale nomear o que acabou de acontecer, porque é a parte reutilizável. A allowlist é um juízo de valor escrito como um match statement. Alguém decidiu quais capacidades um emissor pode manter e ainda assim ser aceito em um venue que guarda o dinheiro de outras pessoas, e depois compilou essa decisão. Quando você escolhe as extensões do SPROUT você não está escolhendo features, você está dando um lance por admissão, e a lista de preços é pública.

### Os três bypasses, e a stablecoin que não deveria rotear mas roteia

Aqui é onde uma allowlist impressa começa a mentir para você.

Se a regra fosse só "toda extensão tem que estar na lista," então uma stablecoin regulada levando `PermanentDelegate` para congelamento e confisco de compliance seria não negociável no CP-Swap. Várias delas são negociadas tranquilo. Eu passei uma noite embaraçosa convencido de que os docs estavam errados antes de voltar ao `token.rs` e ler as linhas acima da caminhada pelas extensões.

A checagem tem três saídas de emergência documentadas, e nenhuma delas é uma entrada de allowlist.

A primeira é no nível do programa. A checagem de extensões só existe porque o CP-Swap conhece o Token-2022; um mint cujo dono é o programa SPL Token clássico não tem TLV por onde caminhar e pula o branch inteiro. SPL clássico não está "na allowlist," ele está fora do escopo da pergunta.

A segunda é a `MINT_WHITELIST` hardcoded no topo do mesmo arquivo, com quatro endereços de comprimento no commit que eu li. Quatro. Uma lista nomeada de exceções, em produção, no fonte, sem nenhuma cerimônia de governança em volta dela. Se o seu mint é um dos quatro, a caminhada pelas extensões nunca roda.

A terceira é uma conta de associação de mint, e como ela é estrutural no fluxograma e no challenge, aqui está o que ela é de fato: a contabilidade própria do CP-Swap, não uma extensão do Token-2022. É uma conta pequena por mint do programa CP-Swap, inicializada pelo caminho de admin da Raydium para um mint específico, então ela funciona como a irmã crescida da whitelist: aprovação por mint registrada como uma conta em vez de um array hardcoded, o que significa que uma aprovação nova custa uma transação de admin em vez de um redeploy de programa. Se existe uma conta de associação inicializada para o seu mint, a caminhada pelas extensões nunca roda. Mesmo efeito que o array, porta diferente, e uma que você consegue verificar de fora checando se a conta existe.

Então a enunciação honesta da regra é uma coisa de dois branches, e é exatamente isso que o seu preditor tem que codificar. Primeiro pergunte se algum bypass se aplica. Só se nenhum se aplicar, pergunte se toda extensão está na lista de cinco. Erre essa ordem e você vai prever rejeição com toda a confiança para um token que está sendo negociado na sua frente.

![Fluxograma da checagem de criação de pool do CP-Swap da Raydium mostrando três branches de bypass para SPL clássico, mints na whitelist e mints com associação de mint, antes do teste de allowlist de cinco extensões e do caminho de rejeição.](assets/v04-flowchart.webp)

O valor de ensino daquela whitelist não são os quatro endereços, é o que a existência deles te diz sobre como a admissão em venue funciona de verdade. Alguns tokens entram porque o conjunto de extensões deles é chato. Outros entram porque alguém no venue tomou uma decisão sobre eles pelo nome. Se o seu plano de produto é "a gente vai levar um delegado permanente para compliance e entrar na whitelist como as stablecoins entraram," esse é um plano de desenvolvimento de negócios em vez de um de engenharia, e você deveria custeá-lo como tal.

### O que a checagem não consegue ver

Antes de você ir construir um preditor que reproduz essa regra, tenha clareza sobre o quanto a regra é estreita, porque dois dos pontos cegos dela vão moldar decisões que você toma na próxima lição.

Ela roda uma vez. A caminhada pelas extensões acontece na criação da pool, e depois disso a pool existe. Nada roda a allowlist de novo sobre uma pool viva quando o mint muda por baixo dela, e o mint pode mudar: a sua `transfer_fee_config_authority` consegue armar uma nova tabela de taxa para uma epoch futura na hora que quiser, que é exatamente o mecanismo que você construiu e viu aterrissar em m02-l1. Então "roteável" é uma afirmação sobre admissão, não uma promessa sobre comportamento para sempre. A admissão foi concedida a um tipo de extensão, e a configuração dentro daquele tipo continuou sua.

E ela lê tipos, não configurações. A caminhada casa variantes de extensão: ela pergunta se uma entrada `TransferFeeConfig` está presente, não se a taxa é zero ou cinco por cento. Siga isso até o fim e você chega em um resultado que as pessoas acham surpreendente na primeira vez. Um mint levando uma entrada `TransferHook` cujo program id é null, um slot de hook que não chama nada, ainda falha na caminhada, porque a entrada TLV está lá e a entrada é o que é casado. Dormente não é ausente. Essa é a imagem espelhada do design do PYUSD ao qual eu vou chegar em um instante, e é por isso que "a gente configurou a extensão mas deixou desligada" te compra boa vontade com um auditor e exatamente nada com um programa.

![Linha do tempo mostrando que a checagem de extensões da Raydium roda só na criação da pool, enquanto mudanças posteriores de tabela de taxa, ações de autoridade e um upgrade contrafactual de hook não disparam nenhuma re-checagem.](assets/v05-timeline.webp)

### Por venue, nunca por DEX

Mais uma correção antes do lab, e é a que vai te salvar de uma queda de verdade.

Tudo acima é verdade do CP-Swap e do CLMM da Raydium. Não é verdade da Raydium. A mesma marca roda venues com regras diferentes, e os mais antigos são mais rígidos: a AMM v4 e a Stable AMM aceitam só SPL clássico, e a razão da própria Raydium é que o programa é anterior ao Token-2022. Então o seu SPROUT que leva taxa, que navega tranquilo pela allowlist do CP-Swap, não pode ser colocado em pool na AMM v4 de jeito nenhum, e tirar uma extensão não vai ajudar. Enquanto isso mints de recompensa do Farm v6 podem ser Token-2022, e o LaunchLab cria os mints dele com um metadata pointer e uma taxa de transferência opcional limitada a cinco por cento. Quatro respostas diferentes, uma marca.

Saia da Raydium e a forma muda de novo.

A Orca publica uma tabela de suporte por extensão mais um processo de revisão de Token Badge, que é um mecanismo inteiramente diferente: não uma lista hardcoded em um programa, mas uma aprovação por mint que um humano concede. Na minha leitura de 2026-08-21 a tabela mostra transfer fee, memo transfer, metadata pointer, token metadata e interest-bearing como suportados, confidential transfer como suportado só para transferências não confidenciais, e permanent delegate como exigindo um Token Badge. O resto das linhas, incluindo a que você mais quer, a linha do transfer hook, não voltou na minha busca, e eu não vou chutar. Trate isso como um item aberto para verificar, não uma lacuna para preencher com vibes.

A Meteora é o contraponto que mantém isso honesto. A Dynamic Bonding Curve dela suporta explicitamente configs de token com transfer hook; a config dela tem opções de autoridade de atualização que o README dela descreve como válidas só para configs e pools com transfer hook. Um venue construído depois, com encaminhamento de hook projetado desde o início, tomou a decisão oposta à da Raydium. A própria orientação de integração da Meteora também diz aos builders para rejeitar extensões Token-2022 não suportadas antes de mostrar um fluxo de lançamento e para encaminhar contas de hook em cada instrução de transferência. Suporte defensivo, mas suporte de verdade.

E a Jupiter, para onde a maior parte do fluxo de varejo de fato roteia: eu não consegui encontrar uma política publicada de roteamento de Token-2022 nos docs de desenvolvedor dela em 2026-08-21. Não ter página de política não é o mesmo que não ter política. É um desconhecido, e vai para a sua lista de verificação com a data anexada. Agregação como disciplina de cliente pertence ao curso planejado Client-Side Mastery; o que pertence a você aqui é saber que a pergunta existe e que ninguém a respondeu para você por escrito.

![Tabela comparando seis venues de negociação em suporte a Token-2022 e aceitação de mint com hook, com duas células marcadas explicitamente como não resolvidas ou desconhecidas e cada linha carregando a fonte e a data de leitura dela.](assets/v06-table.webp)

O que me traz ao trade-off que eu te devo, e ele corta contra a lição que você está lendo. Ler a allowlist de uma DEX te diz a verdade para aquele um venue naquele um commit. Não é uma spec portável. A revisão de badge da Orca, a política de roteamento da Jupiter e o comportamento de exibição de cada carteira são regras separadas que você tem que checar você mesmo, e congelar as cinco da Raydium como "a regra do ecossistema" é precisamente o erro que esta lição existe para matar. A lista também se mexe. É por isso que o preditor que você está a ponto de construir carrega o commit de origem dele em um comentário de header, e por que reler `token.rs` no seu commit fixado é o passo zero de todo lançamento, não uma tarefa de uma vez só.

Duas histórias de produção fazem o mesmo ponto de pontas opostas, e depois a gente constrói.

A que ainda me faz rir: o programa de transfer hook da pump.fun, deployado em `333UA891CYPpAJAthphPT3hg1EkUBLhNFoP9HoWW3nug`, tem seis linhas. `#[program] pub mod transfer_hook_authority {}`, e isso é tudo. Eles apontaram o slot de hook só-no-nascimento para um no-op e deixaram lá, então o mint leva um hook que está garantido a não fazer nada em vez de um slot que poderia depois ser apontado para lógica viva. Um programa vazio protegendo bilhões é a ilustração mais limpa possível de por que allowlists existem: do assento de uma pool não tem como distinguir aquele programa de um que queima 200k compute units e chama três outros programas, a não ser lendo e fixando o bytecode dele.

A séria: o PYUSD, o deployment emblemático do Token-2022, entregue pela PayPal e pela Paxos em maio de 2024 com um conjunto de extensões com formato de compliance incluindo um delegado permanente e um transfer hook. O levantamento de panorama de stablecoins da Helius o colocou em $215.9M mantidos em só 20.4k contas de token em 2025-05-29; o supply muda todo dia, e o único número atual é o que o seu próprio `getAccountInfo` retorna. Cada uma dessas extensões de poder está configurada e dormente, o program id do hook null, a taxa em zero basis points. Mas ponha isso contra o que você acabou de aprender: dormente não é ausente, então aquelas entradas TLV ainda falham na caminhada pelas extensões, e no CP-Swap é a `MINT_WHITELIST` hardcoded, não a dormência, que deixa o PYUSD rotear. Esse é um token projetado por gente que entendeu o preço da admissão exatamente: segure a chave, deixe desligada, compre a boa vontade do auditor com dormência, e compre admissão venue por venue, pelo nome.

## Lab: preveja o veredicto, depois deixe o fork te checar

Você vai construir `predict-routability.ts`, o primeiro rascunho do R6, ligá-lo a bytes on-chain reais, e colocar as duas variantes do SPROUT na frente dele. A interface abaixo é um contrato: a próxima lição importa `isRoutable` deste arquivo exato por este nome exato.

1. **Instale os pins.** Em `labs/m05-l1`, com o seu surfnet ainda rodando. A lógica de pins não mudou desde o módulo 2 e eu a re-verifiquei contra o registry em 2026-09-05: o `latest` do npm para o kit é 8.2.0, mas um workspace fixa o major do kit contra o qual as próprias deps `@solana-program/*` dele fazem peer, e o `@solana-program/token-2022@0.15.0` deste aqui faz peer com `@solana/kit@^7.0.0`, com `@solana-program/system@0.13.0` como contraparte. Rode `npm view @solana-program/token-2022@0.15.0 peerDependencies` você mesmo antes de confiar nessa frase; esse trem parte todo mês.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.23.12 typescript@5.9.3
```

2. **Leia a regra, depois rode.** Você clonou o repo lá atrás quando leu o fonte. Agora compile a regra isolada para poder mexer nela. Salve isto como `allowlist.rs` ao lado do seu lab (é uma transcrição da forma, não uma cópia do programa: a função real caminha por um `StateWithExtensions<Mint>` e casa variantes de `ExtensionType`, esta aqui recebe os nomes já decodificados para compilar com `rustc` puro e nenhuma árvore de dependências):

```rust
// allowlist.rs: a standalone transcription of the RULE in raydium-cp-swap,
// programs/cp-swap/src/utils/token.rs L225-236 @ 244e124 (read 2026-08-22).
// The real function walks a StateWithExtensions<Mint> and matches ExtensionType
// variants; this one takes the already-decoded names so it runs with no deps:
//   rustc allowlist.rs && ./allowlist
const MINT_WHITELIST: &[&str] = &[/* the four addresses at token.rs L18-23 */];

fn is_supported_mint(mint: &str, extensions: &[&str]) -> bool {
    if MINT_WHITELIST.contains(&mint) {
        return true;
    }
    extensions.iter().all(|ext| {
        matches!(
            *ext,
            "TransferFeeConfig"
                | "MetadataPointer"
                | "TokenMetadata"
                | "InterestBearingConfig"
                | "ScaledUiAmount"
        )
    })
}

fn main() {
    let sprout = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"];
    let hooked = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"];
    println!("SPROUT       {}", is_supported_mint("SPROUT_MINT", &sprout));
    println!("SPROUT+hook  {}", is_supported_mint("HOOKED_MINT", &hooked));
}
```

```bash
rustc allowlist.rs -o allowlist && ./allowlist
```

Você deve ver `SPROUT true` e `SPROUT+hook false`. Preencha os quatro endereços da whitelist a partir da sua própria saída de `sed` se você quiser que o branch de bypass faça alguma coisa; eu deliberadamente não os imprimo aqui, pela mesma razão pela qual eu não imprimi as regras da matriz de conflitos para você em m01-l4. O território é o arquivo fonte, não esta página.

Enquanto os dois arquivos estão na sua frente, faça a comparação que faz isso colar: ponha a sua saída de `sed` ao lado da transcrição e marque o que a minha versão jogou fora. A função real recebe um mint decodificado e itera variantes reais de `ExtensionType`, então ela também carrega o desempacotamento, o encanamento de erro, e quem chama e transforma um `false` em uma instrução que falhou. O que sobrevive à redução é a decisão em si, e a decisão tem quatro linhas.

![Walkthrough anotado da checagem de suporte de criação de pool do CP-Swap da Raydium, mapeando o bypass da whitelist dela, o casamento de cinco extensões e o retorno false antecipado nas três partes do preditor em TypeScript construído nesta lição.](assets/v07-annotated-code.webp)

3. **Escreva o preditor, com dois buracos.** Crie `predict-routability.ts`. Este é o problema de completion: o tipo e a forma da função estão dados, o conteúdo da allowlist e os branches de bypass são seus.

```typescript
// predict-routability.ts (skeleton). Two holes to fill from the source you just read.
export interface MintProfile {
  tokenProgram: "spl" | "token2022";
  extensions: string[];
  whitelisted?: boolean;
}

// TODO 1: the five extension names CP-Swap accepts, exactly as token.rs lists them.
export const CP_SWAP_ALLOWLIST: ReadonlySet<string> = new Set([]);

export function isRoutable(mint: MintProfile): boolean {
  // TODO 2: the two bypass branches that run BEFORE the extension check.
  //   (The source has three doors; your profile has two branches, because the
  //   whitelist and the mint-association account both arrive collapsed into
  //   the single `whitelisted` flag. The classic-SPL door is the other branch.)
  return mint.extensions.every((e) => CP_SWAP_ALLOWLIST.has(e));
}
```

Duas coisas para acertar, e as duas são perguntas de ordem em vez de perguntas de digitação. Os bypasses rodam primeiro, ou um mint com delegado permanente que está na whitelist recebe um veredicto errado. E o teste de extensões é `every`, não `some`: uma extensão fora da lista contamina o mint inteiro, porque o programa retorna false na primeira entrada ruim e nunca se recupera.

4. **Preencha e dê uma boca a ele.** Aqui está o arquivo terminado. Ele roda sozinho e exporta direito, e é por isso que a auto-execução tem uma trava: a próxima lição importa deste módulo e não quer a sua saída de console.

```typescript
// predict-routability.ts: R6 draft. Reproduces Raydium CP-Swap's pool-creation
// verdict from a mint's token program plus its extension set.
// Modeled on raydium-io/raydium-cp-swap, programs/cp-swap/src/utils/token.rs
// L225-236 (allowlist) and L18-23 (MINT_WHITELIST), commit 244e124, read 2026-08-22.
// Re-read that file at YOUR pinned commit before trusting this file.
import { fileURLToPath } from "node:url";

export interface MintProfile {
  tokenProgram: "spl" | "token2022";
  extensions: string[];
  whitelisted?: boolean;
}

export const CP_SWAP_ALLOWLIST: ReadonlySet<string> = new Set([
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "InterestBearingConfig",
  "ScaledUiAmount",
]);

export function isRoutable(mint: MintProfile): boolean {
  if (mint.tokenProgram === "spl") return true;
  if (mint.whitelisted) return true;
  return mint.extensions.every((e) => CP_SWAP_ALLOWLIST.has(e));
}

export function explain(mint: MintProfile): string {
  if (mint.tokenProgram === "spl") return "classic SPL: extension check skipped";
  if (mint.whitelisted) return "bypass: MINT_WHITELIST or mint-association account";
  const offList = mint.extensions.filter((e) => !CP_SWAP_ALLOWLIST.has(e));
  return offList.length === 0
    ? `all ${mint.extensions.length} extensions on the allowlist`
    : `off-list: ${offList.join(", ")}`;
}

export const SPROUT: MintProfile = {
  tokenProgram: "token2022",
  extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
};

// The DESIGNED hook variant: the full base set plus TransferHook. The variant
// you actually minted in m03 carries TransferHook alone (kept minimal there on
// purpose); the verdict is identical either way, one off-list entry taints it.
export const SPROUT_HOOKED: MintProfile = {
  tokenProgram: "token2022",
  extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"],
};

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const cases: Array<[string, MintProfile]> = [
    ["SPROUT", SPROUT],
    ["SPROUT+hook", SPROUT_HOOKED],
  ];
  for (const [name, profile] of cases) {
    const verdict = isRoutable(profile) ? "ROUTABLE" : "REJECTED";
    console.log(`${name.padEnd(12)} ${verdict.padEnd(9)} ${explain(profile)}`);
  }
}
```

```bash
npx tsx predict-routability.ts
```

```
SPROUT       ROUTABLE  all 3 extensions on the allowlist
SPROUT+hook  REJECTED  off-list: TransferHook
```

Aquela string do `explain` não é decoração, e é o mesmo argumento que eu fiz para o campo `reason` do `check-combo` lá em m01-l4. Um booleano diz a um colega de equipe que ele não pode lançar. Uma razão diz a ele qual extensão tirar.

5. **Alimente com bytes reais.** Até aqui você julgou perfis digitados à mão, o que não prova nada sobre os seus mints de verdade. Ligue o preditor à chain. O `profile-from-mint.ts` lê um mint ao vivo através do seu surfnet e monta o perfil a partir do TLV que o seu inspetor `decode-mint` vem caminhando desde m01-l2:

```typescript
// profile-from-mint.ts: turn a live mint into a MintProfile the predictor can judge.
// Reads the same TLV bytes your decode-mint inspector has walked since m01-l2.
import { address, createSolanaRpc } from "@solana/kit";
import { fetchMint, TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";
import { explain, isRoutable, type MintProfile } from "./predict-routability";

// The four addresses live in token.rs L18-23 at your pinned commit. Read them
// yourself and paste them here; a printed whitelist in a lesson goes stale.
const MINT_WHITELIST: string[] = [];

const RPC_URL = process.env.RPC_URL ?? "http://127.0.0.1:8899";

export async function profileFromMint(mintAddress: string): Promise<MintProfile> {
  const rpc = createSolanaRpc(RPC_URL);
  const mint = await fetchMint(rpc, address(mintAddress));
  const extensions =
    mint.data.extensions.__option === "Some"
      ? mint.data.extensions.value.map((e) => e.__kind)
      : [];
  return {
    tokenProgram: mint.programAddress === TOKEN_2022_PROGRAM_ADDRESS ? "token2022" : "spl",
    extensions,
    whitelisted: MINT_WHITELIST.includes(mintAddress),
  };
}

const target = process.argv[2];
if (target) {
  const profile = await profileFromMint(target);
  console.log(profile);
  console.log(isRoutable(profile) ? "ROUTABLE" : "REJECTED", "-", explain(profile));
}
```

Rode contra três coisas: o mint do SPROUT que você construiu em m02-l4, a variante com hook que você cunhou no surfnet em m03-l2 (as duas re-cunháveis conforme a abertura, se o seu fork reiniciou), e um mint de mainnet que o seu fork já conhece, sendo o PYUSD o óbvio já que você o leu no módulo 1. Repare no que a terceira rodada faz com a sua confiança no preditor, e segure esse pensamento para o passo 7.

```bash
npx tsx profile-from-mint.ts <YOUR_SPROUT_MINT>
npx tsx profile-from-mint.ts <YOUR_HOOKED_SPROUT_MINT>
npx tsx profile-from-mint.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

Esperado para as duas primeiras: os mesmos veredictos do passo 4, SPROUT ROUTABLE e a variante com hook REJECTED, só que agora julgados a partir de bytes TLV ao vivo em vez de um perfil digitado à mão. Uma divergência é esperada e inofensiva: a variante com hook ao vivo imprime uma lista de extensões de uma entrada, só `TransferHook`, onde o perfil `SPROUT_HOOKED` do passo 4 modelou a variante projetada de quatro extensões. O m03 cunhou a variante mínima de propósito, e o veredicto não se importa, porque uma única entrada fora da lista contamina o mint qualquer que seja o conjunto em volta dela. O terceiro veredicto é o que você está segurando para o passo 7.

![Fluxograma de pipeline de um endereço de mint passando pelo preditor de roteabilidade até um veredicto, com três artefatos anteriores alimentando-o e uma tentativa de criação de pool em fork de mainnet fornecendo a verdade de campo.](assets/v08-flowchart.webp)

6. **Agora a parte que pode provar que você está errado.** Tudo até aqui é o seu modelo do programa. A verdade de campo é o programa. No seu fork de surfnet, o deployment do CP-Swap e as contas de config dele são os de mainnet de verdade, então uma tentativa de criação de pool é um teste genuíno. A Raydium fornece um repositório de demonstração cuja seção CPMM monta exatamente essa chamada com `raydium.cpmm.createPool({ programId: CREATE_CPMM_POOL_PROGRAM, poolFeeAccount: CREATE_CPMM_POOL_FEE_ACC, mintA, mintB... })`. Clone em uma pasta separada: o SDK da Raydium é código de terceiro que roda em cima do web3.js v1 e não vem com nenhuma superfície kit, então a dependência de v1 é inevitável aqui. A regra que este curso segue, enunciada com precisão suficiente para você conferir um lab posterior contra ela: **ponha um SDK de terceiro em quarentena na menor unidade que ainda compila, e deixe as duas pilhas se encontrarem na chain em vez de em um import compartilhado.** Às vezes essa unidade é um workspace inteiro, como aqui e no lab do Light em m08-l3, onde todo arquivo da pasta fala v1 porque o fornecedor fala e misturar um segundo SDK em uma pasta só seria pior. Às vezes é um único arquivo, como em m09-l1, onde `venue.ts` guarda o único import de web3.js do lab e entrega valores simples para código kit dos dois lados. O que a regra nunca permite é um arquivo de primeira parte importando os dois clients para se poupar de uma conversão:

```bash
cd .. && git clone https://github.com/raydium-io/raydium-sdk-V2-demo.git
cd raydium-sdk-V2-demo && npm install
```

   Depois três edições nos próprios arquivos da demo antes de você rodar qualquer coisa, porque essa configuração É o passo que produz a sua verdade de campo:

   - `src/config.ts` liga o cluster e o signer: aponte `connection` para o seu surfnet (`http://127.0.0.1:8899`) e carregue `owner` de `~/.config/solana/id.json`.
   - Em `src/cpmm/createCpmmPool.ts`, troque o par `DEVNET_PROGRAM_ID` que o arquivo já traz pelas constantes de mainnet `CREATE_CPMM_POOL_PROGRAM` / `CREATE_CPMM_POOL_FEE_ACC` que ele já importa; o seu fork carrega o deployment da mainnet, não o da devnet.
   - `mintA` é a sua variante do SPROUT; para `mintB` use WSOL (`So11111111111111111111111111111111111111112`), que o fork já conhece e que o seu payer financia com `spl-token wrap 1`.

   Depois rode duas vezes, uma por variante do SPROUT.

Esperado: o SPROUT cria uma pool, a variante com hook falha dentro do programa. Registre a falha exata, porque a forma da falha é o achado: o que conta é um custom program error atribuído ao program id do CP-Swap nos logs da transação, a renderização de quem chama a checagem retornando `Ok(false)`, não uma exceção lançada pelo SDK antes de qualquer coisa ser enviada. E leia isto honestamente. Existem dois jeitos de esse passo sair de lado e eles significam coisas diferentes. Se o lookup de token do SDK não conseguir resolver o seu mint local pela API hospedada da Raydium, essa é uma falha do lado do cliente e não o veredicto do programa; você aprendeu algo sobre o SDK, nada sobre a allowlist. Só um erro lançado pelo programa conta como o programa respondendo. Se você não conseguir rodar o caminho completo hoje, diga isso nas suas anotações em vez de promover a opinião do preditor a evidência, e pegue o caminho degradado no passo 7.

7. **Coloque a cancela.** Qualquer que tenha sido o caminho que você conseguiu, feche o loop com um assert-script, o mesmo padrão de `test-check-combo.ts`. Este é o teste de aceitação da lição e ele codifica a regra inteira, bypasses incluídos:

```typescript
// verify-routability.ts: this lesson's gate. Same assert-script pattern as
// test-check-combo.ts from m01-l4: plain asserts, exit 1 on the first miss.
import assert from "node:assert/strict";
import { isRoutable, type MintProfile } from "./predict-routability";

const t22 = (extensions: string[], whitelisted = false): MintProfile => ({
  tokenProgram: "token2022",
  extensions,
  whitelisted,
});

const cases: Array<[string, MintProfile, boolean]> = [
  ["classic SPL, no extensions", { tokenProgram: "spl", extensions: [] }, true],
  ["SPROUT: fee + metadata pair", t22(["TransferFeeConfig", "MetadataPointer", "TokenMetadata"]), true],
  ["SPROUT + harvest hook", t22(["TransferFeeConfig", "MetadataPointer", "TokenMetadata", "TransferHook"]), false],
  ["permanent delegate, unlisted", t22(["PermanentDelegate"]), false],
  ["permanent delegate, whitelisted", t22(["PermanentDelegate", "MetadataPointer"], true), true],
  ["confidential SPROUT branch", t22(["ConfidentialTransferMint"]), false],
  ["scaled UI display only", t22(["ScaledUiAmount"]), true],
];

let passed = 0;
for (const [label, profile, expected] of cases) {
  assert.equal(isRoutable(profile), expected, `${label}: expected ${expected}`);
  passed += 1;
}
console.log(`routability predictor: all ${passed} assertions passed`);
```

```bash
npx tsx verify-routability.ts
```

```
routability predictor: all 7 assertions passed
```

Repare no sexto caso. O seu SPROUT confidencial da lição passada está lá também, e ele falha, que é a aritmética da escolha que você já fez: a variante mais privada que você construiu é a que nenhuma AMM vai cotar nunca. Nada a corrigir. É essa a forma do espaço de design. E feche o pensamento que você segurou do passo 5: o PYUSD voltou REJECTED do seu preditor enquanto ele é negociado neste mesmo venue na mainnet, porque a sua constante `MINT_WHITELIST` ainda está vazia e a real não está. O PYUSD roteia pela whitelist hardcoded, não por uma caminhada pelas extensões na qual as extensões de poder dele falhariam. O preditor é tão atual quanto os quatro endereços que você cola nele, o que é o branch dois de bypass mostrando o seu valor.

## Challenge

Solo, sem apoio, e é o artefato desta lição provado em entradas que eu não escolhi para você. Implemente `isRoutable(tokenProgram, extensionList, whitelisted)` para que ele reproduza a decisão de criação de pool do CP-Swap para um perfil arbitrário de mint. Uma diferença em relação ao seu `predict-routability.ts` local: o grader passa o perfil achatado em três escalares posicionais em vez de um objeto. `tokenProgram` é `'spl'` ou `'token2022'`; `extensionList` são os nomes dos tipos de extensão separados por espaço em uma única string, `''` quando o mint não leva nenhuma, separe você mesmo antes de julgar; `whitelisted` é a flag de bypass, true para uma entrada da MINT_WHITELIST ou uma conta de associação de mint inicializada. Mesma regra, mesmos cinco nomes, encanamento diferente. O starter que você recebe codifica o modelo folclórico, "SPL clássico ou nenhuma extensão, todo o resto é rejeitado," e ele falha na suíte exatamente onde os casos interessantes moram.

Os critérios de aceitação, direto da cancela:

- um mint SPL clássico roteia independentemente do que o campo de extensões dele diga
- um mint cujas extensões estão todas na allowlist de cinco entradas roteia
- um mint com qualquer extensão fora da lista (`TransferHook`, `PermanentDelegate`, `ConfidentialTransferMint`) é rejeitado
- um mint `PermanentDelegate` que está na whitelist roteia via bypass
- um mint misturando uma extensão da allowlist e uma fora da lista é rejeitado

Três dicas, na ordem em que você vai precisar delas. A allowlist é exatamente cinco nomes. Os branches de bypass fazem curto-circuito antes da checagem de extensões. E o último critério é o que separa uma solução que passa de uma plausível: pense `every`, não `some`.

Depois uma extensão do challenge que nenhum teste consegue avaliar, e é a que importa na hora do lançamento. Escolha qualquer mint Token-2022 ao vivo que NÃO seja seu, leia-o com `profile-from-mint.ts`, e anote o veredicto dele mais a única frase que torna o veredicto acionável para o emissor dele. Se a sua frase nomear uma extensão específica e um venue específico, você está fazendo o trabalho. Se ela disser "suporte a Token-2022 é complicado," você está citando um ticket de suporte.

![Comparação de três cancelas por que um token tem que passar, legalidade de inicialização imposta pelo Token-2022, admissão em venue imposta por DEX, e exibição na carteira imposta por ninguém, cada uma com o modo de falha dela.](assets/v09-comparison.webp)

## Checkpoint

O critério desta lição: `npx tsx verify-routability.ts` verde em todas as sete asserções, e as duas variantes do SPROUT passando pelo `profile-from-mint.ts` contra o seu fork com os veredictos batendo com o que você previu, SPROUT roteável e a variante com hook rejeitada. Se você conseguiu rodar o caminho completo de criação de pool no passo 6, a resposta do fork e a resposta do seu preditor concordam e você tem evidência. Se você pegou o caminho degradado, você tem um modelo validado contra o fonte em vez de contra execução, e a frase honesta para escrever é "o preditor bate com o token.rs em 244e124; a tentativa de criação de pool está sem rodar." As duas passam. Só uma delas é prova, e saber qual você está segurando é a habilidade de verdade.

Os erros que eu espero, na ordem em que eles normalmente acontecem. Um caso de delegado permanente na whitelist retornando `false` significa que os seus branches de bypass estão abaixo da checagem de extensões em vez de acima dela. Um mint levando uma extensão fora da lista passando enquanto um mint simples sem extensões falha significa que `some` entrou onde `every` pertence. E uma rodada de `profile-from-mint` que reporta zero extensões em um mint que você sabe que leva três normalmente significa que você o apontou para um clone SPL clássico do seu mint, ou que o seu surfnet reiniciou e perdeu o mint local que você criou antes dele. Re-cunhe com os dois comandos que a abertura nomeia e rode de novo; o fork é barato.

Se a sua leitura do `token.rs` discordar da minha, os números de linha mudaram, as cinco viraram quatro ou seis, ou a whitelist cresceu para além de quatro entradas, isso não é um bug no seu trabalho, é o alvo móvel sobre o qual esta lição não para de avisar. Poste o hash do commit e o diff na discussão do curso. Eu prefiro que esta página seja corrigida por um aluno a ser acreditada por um.

Você já consegue ler se qualquer extensão isolada mantém o SPROUT negociável na Raydium, a partir do fonte que decide isso. A próxima lição transforma essa leitura em uma decisão de design: você escolhe o conjunto final de extensões de launch-venue do SPROUT, o defende do lado da pool na mesa, e escreve o relatório de roteabilidade que diz honestamente onde ele é negociado e o que você ainda tem que verificar você mesmo. Boa verificação.
