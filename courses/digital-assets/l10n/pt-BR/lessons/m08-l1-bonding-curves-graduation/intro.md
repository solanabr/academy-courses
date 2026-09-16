# Bonding curves e graduação: pump.fun a partir de quatro constantes

## Resumo

O módulo 7 fechou com compressão como conceito, uma prova de validade de 128 bytes e uma conta de 5,000 lamports, e prometeu que em breve você ia colocar um token comprimido para trabalhar. Antes disso, uma decisão que você não pode postergar: como o SPROUT entra no mundo. Você tem o mint do R3 e tem o relatório de roteabilidade do R6, que diz exatamente quais extensões mantêm o SPROUT negociável e quais o fazem ser recusado na porta. Esta lição pega o número mais repetido da cultura de lançamento em Solana, os 85 SOL que "graduam" uma moeda pump.fun, e se recusa a repeti-lo. Em vez disso, você vai derivá-lo, a partir das constantes publicadas da pump e de uma invariante (três das quatro constantes fazem a derivação; a quarta, o total supply, só precifica um resto à parte), e depois construir a config de lançamento que o calcula ao vivo e escolhe o venue de graduação do SPROUT perguntando se aquele venue consegue sequer segurar o token que você construiu. O recuo é abrupto aqui: a derivação é trabalhada por inteiro na visão geral, o lab faz você escrever a linha da invariante no papel antes de te mostrar a listagem, e o challenge te entrega `graduationSol` com nada além das constantes e um arquivo de teste. No fim você vai ter uma ferramenta que recalcula um limiar para qualquer curva, o que importa mais que o número, porque o número pertence ao programa de outra pessoa e pode mudar numa terça-feira.

Comece pela resposta, em uma linha. Abra um terminal em qualquer lugar que tenha Node:

```bash
node -e 'const vs=30,vt=1073e6,rt=793.1e6;console.log((vs*vt/(vt-rt)-vs).toFixed(3),"SOL")'
```

```
85.005 SOL
```

Três números entram, a constante do folclore sai. `vs` é a reserva virtual de SOL da curva, `vt` a reserva virtual de token dela, `rt` os tokens reais que ela vai te vender. Nada nessa linha lê uma chain, e nada nela contém um 85. O resto desta lição é sobre por que aquelas três entradas e aquela única expressão são a história inteira, o que cada uma delas está de fato fazendo, e o que significa para o seu token que a história pertença a um programa que você não controla.

## De onde os 85 SOL realmente vêm

### O número que ninguém armazena

Toda thread de lançamento, todo vídeo explicativo, toda thread de "como o pump funciona" repete a mesma forma: sua moeda negocia numa curva até entrar mais ou menos 85 SOL de pressão compradora, e então ela gradua para um AMM de verdade. Dito desse jeito, 85 soa como um limiar parado em algum campo, checado pelo programa em toda compra.

Então cheque. A teoria mais ingênua é que a graduação é um parâmetro armazenado, e leva uns trinta segundos para falsificá-la a partir do IDL da própria pump em vez do blog post de alguém:

```bash
npm pack @pump-fun/pump-sdk@1.36.0
tar xzf pump-fun-pump-sdk-1.36.0.tgz
node -e '
const idl = require("./package/src/idl/pump.json");
console.log("program:", idl.address);
const curve = idl.types.find(t => t.name === "BondingCurve");
console.log("BondingCurve fields:", curve.type.fields.map(f => f.name).join(", "));
'
```

```
program: 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P
BondingCurve fields: virtual_token_reserves, virtual_quote_reserves, real_token_reserves, real_quote_reserves, token_total_supply, complete, creator, is_mayhem_mode, is_cashback_coin, quote_mint
```

Dez campos. Quatro reservas, um supply, um booleano, um creator, duas flags de modo mais novas e um quote mint, e nenhum preço de graduação em lugar nenhum ali, nenhum limiar, nenhum market cap alvo, o que significa que a conta que governa toda a vida pré-AMM da sua moeda não tem ideia de que 85 é um número que interessa a alguém. Esse pin de versão merece uma nota, porque eu li em 2026-08-22 quando o npm latest era 1.36.0, e o SDK se move rápido o suficiente para que a lista de campos que você imprime possa ser mais longa que a minha. Se for, o argumento sobrevive: o que você está procurando é um limiar armazenado, e a ausência dele é o ponto.

Duas outras coisas saíram daquela leitura e vão importar depois. `virtual_quote_reserves` já foi `virtual_sol_reserves`, renomeado quando a pump passou a suportar quote mints que não fossem SOL, então textos mais antigos e o IDL atual discordam do nome enquanto falam do mesmo slot. E o program id é `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`, que é a coisa para grepar quando você quer saber se uma transação que você está olhando tocou a pump.

Se o limiar não está armazenado, ele tem que estar implícito. Implícito em quê?

### Reservas virtuais são uma tabela de preços vestida de saldo

Quatro constantes definem uma curva da pump ao nascer, e todas as quatro vivem na conta `Global` do programa, não no código-fonte dele: `initial_virtual_token_reserves`, `initial_virtual_sol_reserves`, `initial_real_token_reserves` e `token_total_supply`. Para a configuração de referência, essas são 1,073,000,000,000,000 unidades base de token virtual, 30,000,000,000 lamports de SOL virtual, 793,100,000,000,000 unidades base de token real, e um total supply de 1,000,000,000 tokens com 6 decimals.

Converta para tokens inteiros e os números ficam mais amigáveis: 1.073 bilhão de tokens virtuais, 30 SOL virtuais, 793.1 milhões de tokens reais, 1 bilhão de supply.

Olhe fixo para a primeira e para a última por um segundo. A curva alega 1.073 bilhão de tokens em reserva. O mint só cria 1 bilhão. Uma reserva que guarda mais que o supply inteiro não é um saldo, e esse é o sinal: reservas virtuais não são custódia, são os dois números de que uma fórmula de precificação precisa. Uma reserva real é o que o programa vai de fato te entregar, enquanto uma reserva virtual é só onde o programa finge estar na curva de preço, e o vão entre as duas é uma escolha de design em vez de um acidente, a escolha que define seu preço de abertura e, portanto, toda a forma do passeio que vem depois.

![A reserva virtual de token de 1.073 bilhão se estende além da linha de supply de 1 bilhão enquanto a reserva real de 793.1 milhões fica dentro dela, marcando as reservas virtuais como coordenadas de precificação.](assets/v01-diagram.webp)

A regra de precificação é a mais antiga dos mercados on-chain. O produto das duas reservas virtuais fica constante em toda troca:

```
k = virtualSol x virtualToken
```

Compre tokens e a reserva virtual de token cai enquanto a reserva virtual de SOL sobe, exatamente na proporção que mantém `k` onde estava. Essa é a invariante de produto constante, e vale nomeá-la com precisão justamente porque tudo o que vem depois é consequência dela. O preço spot em qualquer momento é só a razão entre as duas reservas, SOL por token. Ao nascer isso é 30 dividido por 1.073 bilhão, ou cerca de 2.796e-8 SOL por token. Barato de propósito. O primeiro comprador tem que se sentir cedo.

Se você quer escrever um swap de produto constante você mesmo em vez de ler um, o curso Master Anchor V2 constrói um de brinquedo como padrão de framework. Aqui a gente só precisa da invariante como fato contábil, não como programa para autorar.

### O modelo de preço ingênuo, e exatamente o quanto ele erra

Aqui está a estimativa que quase todo mundo escreve primeiro, eu incluído na primeira vez que tentei checar a alegação dos 85. Você sabe o preço de abertura. Você sabe quantos tokens reais a curva vai vender. Multiplique:

```
793,100,000 tokens x 2.7959e-8 SOL/token = 22.174 SOL
```

São 22 SOL, não 85. O vão não é um artefato de arredondamento nem uma taxa esquecida, é um fator de 3.8, e a tentação naquele momento é sair caçando os 63 SOL que faltam em algum lugar da tabela de taxa da pump. Não existe essa taxa. 100 basis points fixos sobre 85 SOL dá menos de um SOL.

A estimativa está errada por um motivo estrutural: ela precifica todo token ao preço do primeiro. A curva fica mais íngreme conforme drena. Cada token que você compra deixa o próximo token mais caro, porque a reserva de token caiu e a reserva de SOL subiu e `k` se recusa a mover. Precificar 793 milhões de tokens à taxa de abertura é como precificar uma escada inteira pela altura do primeiro degrau.

O conserto seguinte em ingenuidade é tirar a média do primeiro e do último preço, o que pelo menos admite que a curva se move. Esse também falha, de forma mais sutil: uma curva de produto constante não é linear, então a média aritmética dos extremos não é o preço médio pago. Você estaria integrando uma hipérbole com um trapézio, e numa curva cujo preço sobe quase quinze vezes de ponta a ponta, o erro é grande o suficiente para importar para quem estiver dimensionando um lançamento.

Existe um terceiro conserto ingênuo que também merece ser morto, porque é o conserto que as pessoas experientes procuram: olhar o market cap em que se observa moedas graduando, e deduzir o SOL a partir disso. Ele dá mais ou menos a resposta certa, e é isso que o torna perigoso. É uma medição de uma população, não uma propriedade do mecanismo, então ele absorve em silêncio qualquer era de taxa, quote mint e configuração de curva sob as quais as moedas amostradas por acaso lançaram. Mude qualquer uma dessas e seu número fica velho sem nenhuma mensagem de erro. Observação não consegue te dizer por quê, e só o porquê sobrevive a uma mudança de config.

Então a pergunta de verdade é mais estreita que "qual é o custo total." Ela é: em que estado a curva está no momento exato em que ela para de vender, e qual reserva de SOL a invariante exige para aquele estado? Responda isso e o total cai sozinho, sem integração nenhuma.

### Drenando a reserva, e a única linha que faz a integração por você

A graduação acontece quando a reserva real de token chega a zero, ou seja, cada um daqueles 793.1 milhões de tokens reais foi vendido e não sobrou nada para a curva entregar a ninguém, então ela vira um booleano e para.

Agora traduza "a reserva real está vazia" para a língua das reservas virtuais, porque essa é a língua que a invariante fala. Todo token real que sai da curva também sai da reserva virtual de token, já que eles se movem juntos em toda compra. Drene toda a reserva real e a reserva virtual de token caiu exatamente `realTokenReserves`:

```
finalVirtualToken = virtualToken - realToken
                  = 1,073,000,000 - 793,100,000
                  = 279,900,000
```

A invariante foi verdadeira todo esse tempo e continua verdadeira naquele instante, então a reserva virtual final de SOL é forçada:

```
finalVirtualSol = k / finalVirtualToken
                = (30 x 1,073,000,000) / 279,900,000
                = 32,190,000,000 / 279,900,000
                = 115.005 SOL
```

A curva começou com 30 SOL virtuais e termina segurando 115.005. A diferença é o SOL que teve que entrar dos compradores:

```
graduationSol = 115.005 - 30 = 85.005 SOL
```

Aí está. O número que as pessoas repetem como lei do universo é a aritmética de `30 x 1073 / (1073 - 793.1) - 30`, e ele não está armazenado em lugar nenhum porque não precisa estar. Ele está implícito em três das quatro constantes publicadas, SOL virtual, token virtual e token real, do mesmo jeito que a prestação de um financiamento está implícita numa taxa e num prazo; a quarta constante, o total supply, nunca entra nesta aritmética e só importa para o adendo do resto lá embaixo.

![Uma curva de preço de produto constante subindo 14.7 vezes da abertura até a graduação, com a área verdadeira embaixo dela marcada 85.005 SOL contra um retângulo de preço fixo bem menor marcado 22.174 SOL.](assets/v02-chart.webp)

Vale levar duas consequências desta seção, porque são elas que fazem da derivação uma ferramenta em vez de um truque.

A primeira: o limiar se move quando as constantes se movem. Pegue uma curva com 30 SOL virtuais, 1 bilhão de tokens virtuais e 800 milhões de tokens reais. Então a reserva virtual final de token é 200 milhões, a reserva virtual final de SOL é 30 bilhões sobre 200 milhões, ou 150, e o limiar é 120 SOL. Mesma fórmula, curva diferente, uma barra de graduação 41% mais alta, e nenhum folclore necessário. Uma reserva real maior em relação à virtual significa que você está drenando mais para cima na parte que fica íngreme da curva, e o SOL exigido sobe junto.

A segunda: o preço final também é uma derivação. Na graduação o preço spot é 115.005 dividido por 279.9 milhões, cerca de 4.109e-7 SOL por token, o que é 14.7 vezes o preço de abertura. Esse múltiplo não é número de marketing, é `(1073 / 279.9)` ao quadrado, e ele te conta a forma de todo o passeio pré-AMM em um número. Quem compra no topo da curva paga mais ou menos quinze vezes o que o primeiro comprador pagou, antes de qualquer coisa negociar num AMM.

E uma conta que surpreende as pessoas: 1 bilhão de supply menos 793.1 milhões de reserva real deixa 206.9 milhões de tokens, cerca de 20.7% do supply, que a curva nunca oferece a ninguém. Esse resto é o que é levado para a pool na migração junto com o SOL arrecadado. Eu verificaria isso contra uma transação de migração real antes de repetir num pitch, e este curso prefere que você cheque do que confie, mas a aritmética é a aritmética e ela explica de onde vem a profundidade inicial da pool de uma moeda graduada.

### A flag complete, e quem tem permissão de apertar o botão

O programa define `complete = true` quando `real_token_reserves` chega a zero, e daquele instante em diante compras e vendas contra a curva falham em vez de negociar a algum preço final. A tabela de erros da própria pump nomeia os dois lados da cerca, e ler códigos de erro é um jeito rápido de aprender a máquina de estados de um programa:

```
6005 BondingCurveComplete     "The bonding curve has completed and liquidity migrated to raydium."
6006 BondingCurveNotComplete  "The bonding curve has not completed."
```

Aquela primeira mensagem é um fóssil, por falar nisso. Ela ainda diz raydium, da era antes de a pump rodar o próprio AMM, enquanto a instrução `migrate` no mesmo IDL entrega liquidez para `pump_amm`. Strings de erro envelhecem mal em toda base de código; trate-as como história, não como documentação.

Agora a pergunta interessante. Quem chama `migrate`? Olhe as contas e a resposta é qualquer um:

```
migrate accounts (25): global, withdraw_authority (w), mint, bonding_curve (w),
  associated_bonding_curve (w), user (signer), system_program, token_program,
  pump_amm, pool (w), pool_authority (w), pool_authority_mint_account (w),
  pool_authority_wsol_account (w), amm_global_config, wsol_mint, lp_mint (w),
  user_pool_token_account (w), pool_base_token_account (w),
  pool_quote_token_account (w), token_2022_program, associated_token_program,
  pump_amm_event_authority, event_authority, program, rent
signers: user
args: []
```

Vinte e cinco contas, e exatamente uma delas assina. O único signer é `user`, e não existe restrição amarrando `user` ao criador. Nenhum argumento. A migração é sem permissão, e é idempotente: a liquidez se move para o AMM da PumpSwap em `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`, os tokens LP são queimados, e um segundo chamador correndo contra o primeiro não consegue migrar em dobro nem drenar nada. A migração em si carrega um `pool_migration_fee` de 15,000,001 lamports, um número estranhamente preciso que é ele mesmo um campo do `Global` em vez de uma constante, o que significa que ele está a uma transação de autoridade de ser outra coisa.

Sem permissão mais idempotente é o design certo aqui e vale entender como padrão, não só como curiosidade. Um passo que qualquer um pode correr para disparar, num momento imprevisível, não pode depender de uma parte específica estar acordada. Bots ficam de olho na flag e disparam migrações de graça. Se em vez disso o passo fosse restrito a uma autoridade, uma moeda graduada cujo criador saísse do ar ficaria com liquidez morta até o criador voltar. Se fosse sem permissão mas não idempotente, a corrida em si seria o exploit. Você quer as duas propriedades ou nenhuma.

![Um fluxo vertical de cinco estágios da criação da curva, passando pelo limiar derivado de 85.005 SOL, até a flag complete e um migrate sem permissão e idempotente que queima o LP na PumpSwap.](assets/v03-flowchart.webp)

### A curva é uma política, e a política mudou debaixo de todo mundo

Tudo até aqui trata as quatro constantes como física. Elas são política, definidas por uma autoridade, e a prova mais clara disso é o que aconteceu com as taxas da pump.

Durante a maior parte da vida da pump a taxa da bonding curve era 100 basis points fixos. Um por cento, igual para uma moeda que vale quatrocentos dólares e uma moeda que vale quatro milhões. Então em 2025-09-01 às 20:00 UTC isso deixou de ser verdade. As taxas ficaram dinâmicas, escaladas pela capitalização de mercado da moeda, e a forma da mudança é visível nos próprios tipos do SDK:

```
FeeConfig { bump: u8, admin: pubkey, flat_fees: Fees,
            fee_tiers: Vec<FeeTier>, stable_fee_tiers: Vec<FeeTier> }
FeeTier   { market_cap_lamports_threshold: u128, fees: Fees }
Fees      { lp_fee_bps: u64, protocol_fee_bps: u64, creator_fee_bps: u64 }
```

Leia aquela estrutura com atenção, porque ela diz mais do que o anúncio disse. A taxa não é mais um número, são três: uma parte do provedor de liquidez, uma parte do protocolo e uma parte do criador. E a faixa que se aplica é selecionada comparando o market cap da curva com uma lista de limiares armazenada numa conta `FeeConfig` na chain. Duas listas de limiares, na verdade, já que moedas cotadas num mint estável ganham a própria tabela `stable_fee_tiers`. As duas são `Vec`s, que é o detalhe em que vale parar: não um array fixo de comprimento conhecido, uma lista sem limite cujo comprimento é ele mesmo dado. O SDK já vem com a função de seleção. Ele não vem com os números. O que significa que qualquer tabela de faixas específica que você leia num blog post, esta lição incluída, é um instantâneo de uma conta que um admin pode reescrever, e a única resposta atual é a que você puxa você mesmo.

Aqui está o que está em jogo para você, e não é abstrato. Se você está modelando um lançamento, a taxa que você paga a 10 SOL de market cap e a taxa que você paga a 300 SOL podem cair em faixas diferentes, e uma planilha construída na era dos 100 bps fixos vai errar o preço das duas. Pior, vai errar numa direção que você não consegue prever de fora, porque os limites das faixas são dados.

![Um fluxograma traçando a taxa de uma troca desde a derivação do market cap, passando pela seleção de faixa no FeeConfig editável, até uma divisão em três vias roteada por oito destinatários rotativos.](assets/v04-flowchart.webp)

O mesmo dia da virada trouxe as moedas Cashback, em que as taxas de criador voltam para os traders em vez de ir para o criador, contabilizadas por PDAs de acumulador de volume por usuário que a instrução de compra toca em toda troca. Isso é um objeto econômico genuinamente diferente atrás da mesma interface: matemática de curva idêntica, incentivo oposto para quem está negociando. E a arrecadação de taxa em si rotaciona por oito endereços destinatários, um `fee_recipient` mais um array `fee_recipients` de sete entradas, que é um detalhe operacional até o dia em que você está indexando fluxos de taxa e se perguntando por que eles se espalham.

![Uma linha do tempo marcando 2025-09-01 20:00 UTC, com uma taxa fixa de 100 basis points antes dela e taxas escalonadas por market cap depois, acima de uma tarja notando que a invariante não mudou.](assets/v05-timeline.webp)

Então nomeie o trade-off com honestidade, porque é nesta parte que uma decisão de lançamento de fato gira. Uma bonding curve te compra descoberta de preço instantânea e sem permissão, sem contraparte com quem negociar, e uma pool garantida no fim com o LP queimado para que ninguém possa puxá-la. O que você paga é perda total de controle sobre a política econômica. A forma da curva é fixada por constantes que você não define, a tabela de taxa é uma conta que outra pessoa pode editar, o venue de graduação é escolhido pelo programa, e a maioria esmagadora das moedas lançadas desse jeito nunca chega ao limiar. Já vi taxas de graduação de um dígito sendo citadas, muitas vezes em torno de um ou dois por cento, e eu não construiria um plano sobre nenhum número que eu mesmo não tivesse medido numa janela escolhida por mim, porque esse número se move com cada ciclo de mercado. A direção não está em dúvida, no entanto: a maioria das curvas empaca, e as que empacam não são um bug no mecanismo. Elas são o mecanismo funcionando, ordenando demanda.

Esse é o acordo. É um bom acordo para uma moeda cuja tese inteira é "deixa o mercado decidir, imediatamente, sem porteiro," e um acordo terrível para um token que tem opiniões sobre como deveria se comportar, que é a maioria dos tokens que existem por um motivo que não seja negociar.

Antes da questão do venue, um checkpoint, porque a derivação tinha várias partes móveis e a próxima seção gasta todas de uma vez. Onde chegamos: 1) a curva precifica com `k = virtualSol x virtualToken`, mantido constante em toda troca; 2) reservas virtuais são coordenadas de precificação, reservas reais são estoque, e só a real pode chegar a zero; 3) drenar a reserva real derruba a reserva virtual de token exatamente naquele valor, o que força a reserva virtual final de SOL a `k / (virtualToken - realToken)`; 4) o SOL que teve que chegar é aquela reserva final menos a inicial, 85.005 para as constantes de referência; 5) nenhum daqueles quatro números é lei, todos os quatro são campos do `Global`, e a tabela de taxa sentada em cima deles é uma conta separada com o próprio admin.

Isso te dá uma ferramenta portátil, então torne-a portátil em voz alta. Quando você encontrar qualquer curva, em qualquer launchpad, faça três perguntas a ela. Quais são as quatro constantes dela, e onde elas vivem, no código ou numa conta que alguém pode editar? Qual condição encerra a curva, e essa condição é sobre um saldo real ou um implicado? E quem tem permissão de disparar a transição, com qual taxa atrelada? Responda essas três e você consegue precificar qualquer bonding curve que encontrar numa tarde, incluindo as que ainda não foram construídas. Deixe de perguntá-las e você está de volta a repetir um número que leu em algum lugar, que é onde esta lição começou.

![Uma tabela de quatro linhas separando números de protocolo em derivados, armazenados, fixados no código e meramente repetidos, com os 85 SOL do folclore arquivados em repetidos.](assets/v06-table.webp)

O que nos traz ao SPROUT.

### O venue veta antes de a matemática importar

O SPROUT tem opiniões. Do R6 você conhece o conjunto final dele para o venue de lançamento: `TransferFeeConfig`, `MetadataPointer`, `TokenMetadata`. Três extensões, todas na allowlist do Raydium CP-Swap, escolhidas precisamente para o token seguir negociável, e é um mint Token-2022, o que não é um detalhe do qual você possa abrir mão depois porque a taxa de transferência que financia a tesouraria só existe no Token-2022, para começo de conversa.

Agora pergunte ao IDL da pump se ele consegue receber aquele mint:

```bash
node -e '
const idl = require("./package/src/idl/pump.json");
const create = idl.instructions.find(i => i.name === "create");
console.log("create.token_program:", create.accounts.find(a => a.name === "token_program").address);
'
```

```
create.token_program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
```

Esse é o programa SPL Token clássico, fixado como endereço fixo na conta, o que significa que a instrução `create` não pode receber um mint Token-2022 de jeito nenhum. E vai além de um desencontro de program id: a pump cria o mint ela mesma. Você não leva um token para a pump, você pede à pump que faça um, e o conjunto de extensões da coisa que ela faz é escolha da pump, não sua. Não existe assento naquela mesa para um token com uma taxa de transferência que você configurou.

Esse é um veto de venue, e ele cai antes de qualquer conta. Você pode derivar o limiar da pump perfeitamente e ainda assim não conseguir usar a pump, que é exatamente o tipo de coisa óbvia em retrospecto e caro por antecipação. Já vi um time escolher um launchpad a partir de uma landing page, construir três semanas de tokenomics sobre a divisão de taxa dele, e descobrir o pin do token program durante a integração. A derivação transfere para qualquer curva que você encontrar. O venue não.

Então a config de lançamento que você está a ponto de construir tem dois trabalhos, e o segundo é o que salva sua semana: derivar o limiar a partir de quaisquer constantes que um venue publique, e recusar qualquer venue cujo caminho de lançamento não consiga representar o token que você já construiu.

![Uma comparação em duas colunas mostrando a pump.fun recusando o SPROUT porque a instrução create dela fixa o programa SPL Token clássico, contra o Raydium CP-Swap aceitando todas as três extensões do SPROUT a partir da allowlist de cinco entradas dele.](assets/v07-comparison.webp)

## Lab: derive o limiar do SPROUT e fixe o venue dele

O artefato é `sprout-launch/derive-graduation.ts`, e a cancela dele é a forma de sempre do curso: `npx tsx sprout-launch/derive-graduation.ts` imprime o limiar derivado em cerca de 85.005 SOL, as reservas virtuais finais, o alvo do migrate, e um veredicto de venue por candidato, saindo com código diferente de zero se nenhum candidato conseguir segurar o SPROUT ou se as constantes de referência pararem de derivar para 85.

1. Crie a pasta e fixe o runner. Uma ferramenta de dev faz toda a execução aqui, `tsx`, o mesmo pin do lab do R6 (aquele lab também segurava `typescript@5.9.3` para typecheck no editor; nada NESTE lab o invoca, então instale só se você quiser o suporte do editor). Os passos 2 a 5 trabalham dentro de `sprout-launch/`; o passo 6 roda a partir da pasta pai, e o passo diz isso quando você chegar lá:

```bash
mkdir -p sprout-launch && cd sprout-launch
npx --yes tsx@4.23.12 --version
```

   O pin carrega a mesma nota de atualidade que o relatório carregava, rechecada em 2026-08-22: `tsx@4.23.12` era o npm latest naquela leitura. Rode `npm view tsx version` você mesmo no dia em que fizer o scaffold. Um pin que você copiou de uma lição sem checar é um pin que você vai debugar depois.

2. Verifique os dois fatos de que a config depende, a partir do IDL de primeira mão em vez desta página. Você rodou os dois comandos na visão geral; rode-os de novo aqui para as saídas ficarem na pasta em que você está a ponto de construir:

```bash
npm pack @pump-fun/pump-sdk@1.36.0 && tar xzf pump-fun-pump-sdk-1.36.0.tgz
node -e '
const idl = require("./package/src/idl/pump.json");
console.log("program:", idl.address);
console.log("create.token_program:",
  idl.instructions.find(i => i.name === "create")
     .accounts.find(a => a.name === "token_program").address);
console.log("BondingCurve fields:",
  idl.types.find(t => t.name === "BondingCurve").type.fields.map(f => f.name).join(", "));
'
```

   Você quer três coisas na tela antes de escrever qualquer código: o program id, o token program SPL clássico fixado em `create`, e uma lista de campos de `BondingCurve` sem nenhum limiar nela. Se a sua versão do SDK imprimir uma lista de campos mais longa que a minha, anote a versão que você leu e siga. Se ela imprimir um campo de limiar de graduação, pare e avise o curso, porque isso significaria que o programa mudou de forma e a alegação central desta lição precisa de atualização.

3. Comece o módulo com as constantes e a invariante. `finalReserves` é onde a lição inteira vive, e a única expressão estrutural dele é algo que você derivou duas seções atrás. Então, antes de rolar até a listagem, escreva aquela expressão no papel: o SOL virtual final na conclusão, em termos de k e da reserva de token encolhida. Depois leia a listagem e se confira contra a última linha dela:

```typescript
// derive-graduation.ts: SPROUT's launch curve (R10).
// Derives a bonding curve's graduation threshold from the constant-product
// invariant, then checks which graduation venue SPROUT's R6 extension set allows.

/** A bonding curve's published constants. Token amounts in WHOLE tokens. */
export interface CurveConstants {
  /** SOL the curve pretends to hold at t=0. Never a real balance. */
  virtualSolReserves: number;
  /** Tokens the curve pretends to hold at t=0. Never a real balance. */
  virtualTokenReserves: number;
  /** Tokens actually available to buyers before the curve completes. */
  realTokenReserves: number;
}

/** pump.fun's Global-account constants, converted from base units at 6 decimals. */
export const PUMP_REFERENCE_CURVE: CurveConstants = {
  virtualSolReserves: 30, // 30_000_000_000 lamports
  virtualTokenReserves: 1_073_000_000, // 1_073_000_000_000_000 base units
  realTokenReserves: 793_100_000, // 793_100_000_000_000 base units
};

export interface FinalReserves {
  k: number;
  finalVirtualToken: number;
  finalVirtualSol: number;
}

/**
 * The curve at the moment realTokenReserves hits zero: k is unchanged, the
 * virtual token reserve has dropped by every real token sold.
 */
export function finalReserves(c: CurveConstants): FinalReserves {
  const k = c.virtualSolReserves * c.virtualTokenReserves;
  const finalVirtualToken = c.virtualTokenReserves - c.realTokenReserves;
  if (finalVirtualToken <= 0) {
    throw new Error(
      `realTokenReserves (${c.realTokenReserves}) must be smaller than virtualTokenReserves (${c.virtualTokenReserves})`,
    );
  }
  return { k, finalVirtualToken, finalVirtualSol: k / finalVirtualToken };
}
```

   A trava não é decoração. Uma curva configurada com uma reserva real maior que a reserva virtual de token dela não tem estado de graduação nenhum, e sem aquela checagem você retornaria em silêncio um limiar negativo e o imprimiria com cara de paisagem.

4. Adicione as duas quantidades derivadas. As duas são one-liners em cima de `finalReserves`, e mantê-las separadas é o que deixa o challenge e a próxima lição chamá-las de forma independente:

```typescript
/** SOL that must enter the curve to drain the real token reserve. */
export function graduationSol(c: CurveConstants): number {
  return finalReserves(c).finalVirtualSol - c.virtualSolReserves;
}

/** Spot price in SOL per token at the current reserve ratio. */
export function spotPrice(virtualSol: number, virtualToken: number): number {
  return virtualSol / virtualToken;
}
```

   Note o que `graduationSol` faz com uma curva cuja reserva real é zero: a reserva virtual final de token é igual à inicial, a reserva virtual final de SOL é igual à inicial, e a resposta é 0 SOL. Nenhum caso especial, nenhum branch. Uma curva sem nada para vender já se graduou, e a fórmula sabe disso.

5. Agora os registros de venue, que são a metade honesta deste artefato. Cada um carrega de onde vieram os fatos dele, porque uma tabela de venues sem procedência é exatamente o folclore contra o qual esta lição argumenta:

```typescript
export interface GraduationVenue {
  name: string;
  /** Where liquidity lands after migration, when we have a first-party id for it. */
  ammProgramId?: string;
  /** The token program the venue's launch path can represent. */
  baseTokenProgram: "spl-token" | "token-2022";
  /** Extensions the venue's pool program accepts on a Token-2022 mint. */
  extensionAllowlist: string[];
  /** Where each field above was read, and when. */
  source: string;
}

export interface VenueVerdict {
  venue: string;
  accepted: boolean;
  reasons: string[];
}

/** SPROUT's final launch-venue extension set, transcribed from the R6 report. */
export const SPROUT_ROUTABLE_SET = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
];

export const VENUES: GraduationVenue[] = [
  {
    name: "pump.fun -> PumpSwap",
    ammProgramId: "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA",
    baseTokenProgram: "spl-token",
    extensionAllowlist: [],
    source:
      "@pump-fun/pump-sdk IDL: the create instruction pins token_program to TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (read 2026-08-22)",
  },
  {
    name: "Raydium CP-Swap",
    baseTokenProgram: "token-2022",
    extensionAllowlist: [
      "TransferFeeConfig",
      "MetadataPointer",
      "TokenMetadata",
      "InterestBearingConfig",
      "ScaledUiAmount",
    ],
    source: "CP-Swap extension allowlist, verified on a mainnet fork in m05-l1",
  },
];

/** A venue accepts SPROUT only if it can hold the mint AND every extension on it. */
export function checkGraduationVenue(
  venue: GraduationVenue,
  routableSet: string[],
): VenueVerdict {
  const reasons: string[] = [];
  if (venue.baseTokenProgram !== "token-2022") {
    reasons.push(
      `venue mints/accepts ${venue.baseTokenProgram} only; SPROUT is a Token-2022 mint`,
    );
  }
  const refused = routableSet.filter(
    (e) => !venue.extensionAllowlist.includes(e),
  );
  if (refused.length > 0) {
    reasons.push(`extensions not on the venue allowlist: ${refused.join(", ")}`);
  }
  return { venue: venue.name, accepted: reasons.length === 0, reasons };
}
```

   A entrada do CP-Swap deliberadamente não tem `ammProgramId`. Eu não vou te entregar uma string base58 para colar de uma página de curso quando o próprio SDK do venue a exporta, e o campo é opcional exatamente por esse motivo: o id da PumpSwap é um fato de primeira mão congelado que este curso verificou, o do CP-Swap você lê de `CREATE_CPMM_POOL_PROGRAM` no SDK da Raydium que você já clonou no m05-l1.

6. Emita e se autoavalie. A saída é o entregável, então ela imprime a derivação com os valores intermediários dela em vez de só a resposta, que é o que a torna revisável por alguém que não estava nesta lição:

```typescript
function fmt(n: number, places = 3): string {
  return n.toLocaleString("en-US", {
    minimumFractionDigits: places,
    maximumFractionDigits: places,
  });
}

function main(): void {
  const c = PUMP_REFERENCE_CURVE;
  const f = finalReserves(c);
  const grad = graduationSol(c);

  console.log("# SPROUT launch curve (R10)\n");
  console.log("## Derived graduation threshold\n");
  console.log(`k (held constant):        ${fmt(f.k, 0)} SOL*tokens`);
  console.log(`final virtual token:      ${fmt(f.finalVirtualToken, 0)} tokens`);
  console.log(`final virtual SOL:        ${fmt(f.finalVirtualSol)} SOL`);
  console.log(`SOL added to graduate:    ${fmt(grad)} SOL`);

  const open = spotPrice(c.virtualSolReserves, c.virtualTokenReserves);
  const close = spotPrice(f.finalVirtualSol, f.finalVirtualToken);
  console.log(`opening spot price:       ${open.toExponential(4)} SOL/token`);
  console.log(`graduation spot price:    ${close.toExponential(4)} SOL/token`);
  console.log(`price multiple:           ${fmt(close / open, 2)}x`);
  console.log(
    `flat-price estimate:      ${fmt(c.realTokenReserves * open)} SOL (wrong by construction)\n`,
  );

  console.log("## Migrate target\n");
  const target = VENUES[0];
  const targetId = target.ammProgramId ?? "read the id from the venue's own SDK";
  console.log(`${target.name.split(" -> ")[1]} (${targetId})\n`);

  console.log("## Graduation venue check vs SPROUT's R6 set\n");
  console.log(`SPROUT routable set: ${SPROUT_ROUTABLE_SET.join(", ")}\n`);
  const verdicts = VENUES.map((v) => checkGraduationVenue(v, SPROUT_ROUTABLE_SET));
  for (const v of verdicts) {
    console.log(`- ${v.venue}: ${v.accepted ? "ACCEPTED" : "REFUSED"}`);
    for (const r of v.reasons) console.log(`    reason: ${r}`);
  }

  const chosen = verdicts.find((v) => v.accepted);
  if (chosen === undefined) {
    console.error("\nGATE FAIL: no candidate venue accepts SPROUT's R6 set");
    process.exit(1);
  }
  console.log(`\nSelected graduation venue: ${chosen.venue}`);

  if (Math.abs(grad - 85.005) > 0.01) {
    console.error(
      `\nGATE FAIL: reference constants should derive to ~85.005 SOL, got ${fmt(grad)}`,
    );
    process.exit(1);
  }
  console.log("All gates pass: threshold derived, venue selected.");
}

main();
```

   Rode a partir da pasta pai (`cd ..` para fora de `sprout-launch/` primeiro, a troca de diretório de trabalho sobre a qual o passo 1 avisou) para o caminho casar com o comando de verificação do curso:

```bash
npx tsx sprout-launch/derive-graduation.ts
```

```
# SPROUT launch curve (R10)

## Derived graduation threshold

k (held constant):        32,190,000,000 SOL*tokens
final virtual token:      279,900,000 tokens
final virtual SOL:        115.005 SOL
SOL added to graduate:    85.005 SOL
opening spot price:       2.7959e-8 SOL/token
graduation spot price:    4.1088e-7 SOL/token
price multiple:           14.70x
flat-price estimate:      22.174 SOL (wrong by construction)

## Migrate target

PumpSwap (pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA)

## Graduation venue check vs SPROUT's R6 set

SPROUT routable set: TransferFeeConfig, MetadataPointer, TokenMetadata

- pump.fun -> PumpSwap: REFUSED
    reason: venue mints/accepts spl-token only; SPROUT is a Token-2022 mint
    reason: extensions not on the venue allowlist: TransferFeeConfig, MetadataPointer, TokenMetadata
- Raydium CP-Swap: ACCEPTED

Selected graduation venue: Raydium CP-Swap
All gates pass: threshold derived, venue selected.
```

7. Checkpoint, e depois quebre de propósito, porque uma cancela que você nunca viu falhar é decoração. Primeiro prove que a derivação está viva: mude `PUMP_REFERENCE_CURVE` para `{ virtualSolReserves: 30, virtualTokenReserves: 1_000_000_000, realTokenReserves: 800_000_000 }` e rode de novo. Você deve ver `SOL added to graduate: 120.000` e depois `GATE FAIL`, porque o assert dos 85 SOL está checando as constantes de referência especificamente. Aquela falha é a prova: o número recalculado, por conta própria, a partir de constantes que você editou. Coloque a curva de referência de volta.

   Depois prove que a checagem de venue é real: remova `"TransferFeeConfig"` da `extensionAllowlist` do CP-Swap e rode de novo. Agora os dois venues recusam, nenhum candidato é selecionado, e o script sai com código diferente de zero em vez de entregar um plano de lançamento para um token que ninguém vai colocar em pool. Coloque de volta.

![A derivação de quatro linhas anotada linha por linha, levando um k de 32.19 bilhões por uma reserva final de token de 279.9 milhões até o limiar de graduação de 85.005 SOL.](assets/v08-annotated-code.webp)

## Challenge

O lab fez você derivar a linha-chave de `finalReserves` no papel antes de te entregar a listagem. O challenge tira o apoio.

Abra o coding challenge desta lição e você vai encontrar um starter que modela a graduação do jeito que a maioria das pessoas modela primeiro: ele pega o preço spot de abertura e multiplica pela reserva real de token. É a resposta dos 22 SOL, vestida de TypeScript. Seu trabalho é substituir aquele modelo por uma derivação de produto constante, implementando `graduationSol` para o limiar sair das reservas em vez de ser afirmado. Duas notas de interface antes de você começar. Primeiro, o grader chama sua função com três números posicionais em ordem fixa, `graduationSol(virtualSolReserves, virtualTokenReserves, realTokenReserves)`, em vez do objeto de config que o lab passava de um lado para o outro. Segundo, o challenge mantém as reservas de token dele em *milhões* de tokens em vez dos tokens inteiros que o lab usou, então as constantes da pump chegam como `graduationSol(30, 1073, 793.1)`. A invariante não se importa com a escala, o que é em si o ponto: escale as duas reservas de token pelo mesmo fator e a resposta em SOL fica inalterada.

Quatro testes, e o terceiro é o que dá o que pensar. As constantes de referência da pump têm que retornar cerca de 85.005 SOL. Uma curva alterada em 30 / 1000 / 800 tem que retornar 120. Uma curva que começa com uma reserva de SOL mais funda, 85 / 1073 / 793.1, tem que retornar cerca de 240.848, mesmas reservas de token, mesma forma, e o custo escala exatamente pelo fator que a reserva de SOL escalou, 85/30, porque `graduationSol` é linear na reserva inicial de SOL. Esse é o teste que um modelo de preço fixo erra pela margem mais larga. E uma curva com reserva real zero tem que retornar 0, o que o modelo ingênuo também passa, então ele não prova nada sozinho e está ali como âncora de sanidade. Se você se pegar escrevendo um loop que caminha a curva em passos pequenos e acumula, pare: isso vai passar todos os quatro testes e significa que você está integrando numericamente algo que a invariante já resolveu em forma fechada.

![Uma tabela dos quatro casos de teste do challenge pareando cada limiar de graduação esperado com a resposta errada de preço fixo, da curva de referência de 85.005 até a âncora de sanidade de reserva zero.](assets/v09-table.webp)

Depois uma peça de julgamento que nenhum teste consegue avaliar, e é o entregável que este módulo de fato quer. Escreva três frases sobre o lançamento do SPROUT. Frase um: o limiar de graduação que você modelaria para o SPROUT, e as constantes de que ele deriva, dado que o SPROUT não vai lançar na pump. Frase dois: por que a pump está indisponível para o SPROUT, nomeando o mecanismo específico em vez da vibe. Frase três: do que você teria que abrir mão no SPROUT para deixar a pump disponível, e se você abriria. Se a sua terceira frase concluir que derrubar a taxa de transferência para caber no venue está tudo bem, volte ao seu relatório do R6 e leia o que a taxa está financiando antes de se comprometer. Essa é uma decisão de tesouraria, e a restrição de ferramental é só o que a trouxe à superfície.

Mais uma coisa que vale fazer enquanto a derivação está fresca. Pegue os tipos `FeeConfig` e `FeeTier` de antes nesta lição e vá ler a tabela de faixas real direto da chain. Eu deliberadamente não imprimi os limiares aqui, porque eles são dados de conta com um admin, e um curso que os congela é um curso que mente para quem o ler em seis meses. Puxe-os você mesmo, anote a data ao lado do que encontrar, e você terá feito a coisa que esta lição inteira está de fato ensinando, que é dizer a diferença entre um número que é derivado, um número que é armazenado e um número que é repetido.

Se o seu limiar derivado discordar dos 85.005 impressos aqui, cheque as constantes primeiro, porque a conta `Global` da pump está viva e uma autoridade pode mudar qualquer uma das quatro. Se as constantes casarem e o número ainda diferir, sinalize no canal de feedback do curso com a sua saída e a data em que você leu o IDL. Minhas leituras estão estampadas em 2026-08-22 contra `@pump-fun/pump-sdk` no npm latest 1.36.0, e um aprendiz que pega isso derivando está fazendo exatamente o trabalho que a lição existe para instalar.

Agora você consegue derivar onde uma curva termina, a partir de constantes em vez de folclore, e você tem uma config que recusa um venue em que seu token não pode legalmente pousar. Mas a curva que você derivou é uma curva, e o venue que te recusou é um venue. Todo launchpad publica as próprias constantes, a própria divisão de taxa, e a própria opinião sobre quais token programs merecem um assento. Próximo: o panorama dos launchpads, e por que o LetsBonk é uma skin da Raydium.
