# O panorama dos launchpads, anti-snipe e o que a graduação semeia

## Resumo

Na lição passada você derivou o limiar de graduação de ~85 SOL do SPROUT a partir das constantes da própria pump e fixou o alvo do migrate dele na PumpSwap. Esta lição amplia o quadro de um venue para o espaço de design: pump.fun, Raydium LaunchLab, Meteora Dynamic Bonding Curve e Metaplex Genesis, comparados nos eixos que de fato te custam dinheiro. Você vai sondar quatro program ids ao vivo, ler os botões que cada venue entrega ao criador, pesar as quatro defesas anti-snipe em circulação, e escrever exatamente o que um launchpad semeia na graduação contra o que você semearia na mão. Aí você faz a escolha do SPROUT: um venue, uma defesa, justificados contra o conjunto de extensões roteáveis que você congelou no R6. O recuo é amplo aqui. Eu te entrego a tabela de venues e a forma da função de decisão; a regra de pontuação, a escolha da defesa e a justificativa escrita são suas. Esta é a última lição em que o design do token te limita em vez do contrário.

Aqui vai uma coisa que vai te custar dinheiro de verdade se ninguém disser em voz alta. Todo launchpad de que você já ouviu falar publica uma página de marketing sobre a curva dele, e quase nenhum publica a coisa que de fato decide se o seu token consegue usá-lo: em qual AMM a pool pousa quando a curva se completa, e o que aquele AMM vai aceitar. Você pode rodar um lançamento impecável, bater o limiar, e ver a transação de migração reverter porque o seu mint carrega uma extensão que a pool de destino recusa. O risco nunca foi a curva. Foi a última instrução.

Então, antes de qualquer teoria, vá descobrir quantos programas existem de fato por trás das marcas. Volte para a pasta `sprout-launch/` da lição passada, ao lado do `derive-graduation.ts` que você escreveu lá, e jogue isto dentro:

```typescript
// probe-venues.ts: are these launchpads four programs, or fewer than they look?
const RPC = process.env.SOLANA_RPC_URL ?? "https://api.mainnet-beta.solana.com";

const VENUES: Record<string, string> = {
  "pump.fun": "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
  "Raydium LaunchLab": "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
  "LetsBonk": "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
  "Meteora DBC": "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN",
  "Metaplex Genesis": "GNS1S5J5AspKXgpjz6SvKL66kPaKWAhaGRhCqPRxii2B",
};

type AccountValue = { executable: boolean; owner: string } | null;

async function probe(address: string): Promise<AccountValue> {
  const res = await fetch(RPC, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      // dataSlice keeps the response tiny: we want the flags, not the bytecode.
      params: [address, { encoding: "base64", dataSlice: { offset: 0, length: 0 } }],
    }),
  });
  const json = (await res.json()) as { result?: { value: AccountValue } };
  return json.result?.value ?? null;
}

async function main(): Promise<void> {
  const seen = new Map<string, string[]>();
  for (const [name, id] of Object.entries(VENUES)) {
    const acct = await probe(id);
    const state = acct ? (acct.executable ? "executable" : "NOT A PROGRAM") : "NOT FOUND";
    console.log(name.padEnd(20), id.padEnd(46), state);
    seen.set(id, [...(seen.get(id) ?? []), name]);
  }
  console.log("");
  for (const [id, names] of seen) {
    if (names.length > 1) console.log(`same program id: ${names.join(" + ")}  ->  ${id}`);
  }
  console.log(`distinct programs: ${seen.size} for ${Object.keys(VENUES).length} brands`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

```bash
npx tsx probe-venues.ts
```

Cinco marcas. Quatro programas. Aquela última linha é a lição inteira em um número, e as próximas quatro mil palavras são sobre por que o colapso acontece, quais botões sobrevivem a ele, e como você escolhe.

## A mesma cozinha atrás de quatro placas diferentes

### Lendo a sondagem que você acabou de rodar

Os quatro program ids voltaram executáveis quando eu rodei aquele script em 2026-08-22, que é a única afirmação que eu topo fazer sobre eles sem você rodar de novo. Program ids são substituídos, sim. Só a Raydium já entregou várias gerações de programas de pool, e um curso que congela um endereço é um curso que mente para alguém em 2027. O script é o fato; os endereços dentro dele são um instantâneo.

A linha interessante é a do LetsBonk. Ele não é um fork do LaunchLab, e não é um concorrente do LaunchLab do jeito que a cobertura de imprensa da semana de lançamento enquadrou. Ele é o LaunchLab, vestindo outra placa. O programa da Raydium expõe um **Platform PDA**, uma conta de configuração por plataforma derivada sob o programa do LaunchLab, e um terceiro que cria uma ganha o próprio front end com marca própria, a própria estrutura de taxas e a própria fatia da arrecadação, rodando contra exatamente o mesmo conjunto de instruções e graduando exatamente no mesmo AMM. Foi isso que a sua sondagem pegou. Os dois nomes resolveram para uma única chave de 32 bytes, então tudo que é verdade da mecânica do LaunchLab é verdade da mecânica do LetsBonk, por construção.

Eu quero ser honesto sobre o que a sondagem provou e o que ela não provou, porque este é o tipo de afirmação que fica sendo repetida até parar de ser checada. O que você provou é que o id que eu rotulei de "LetsBonk" é um programa executável vivo e que ele é igual ao do LaunchLab. O que você não provou é que os lançamentos do LetsBonk de fato passam por ele, porque isso exige ler uma transação de lançamento de verdade, não uma flag de conta. Faça isso você mesmo antes de repetir a afirmação: abra qualquer token do LetsBonk num explorer, ache a transação de criação dele, e leia o program id invocado na instrução. Se disser `LanMV9sA...`, a afirmação fica nas suas mãos em vez de nas minhas.

Pense nisso como uma franquia, porque a analogia se sustenta até o fim e eu vou continuar usando ela. Uma cozinha, uma fritadeira, um contrato de fornecedor. O franqueado escolhe a placa em cima da porta, os preços no quadro, e quem fica com o caixa no fechamento. O que o franqueado não consegue mudar é a fritadeira. E essa é exatamente a troca que o Platform PDA oferece: controle total de marca e de divisão de taxa, zero controle da mecânica.

![Um diagrama de cubo e raios do programa Raydium LaunchLab com raios de Platform PDA para Raydium, LetsBonk e terceiros, compartilhando uma curva e uma regra de graduação enquanto cada um configura marca e taxas.](assets/v01-diagram.webp)

### O que um launchpad é de verdade

Tire a marca e um launchpad é quatro decisões empacotadas juntas e vendidas como um produto só. Todo venue que você avaliar está respondendo essas quatro, digam os docs dele ou não.

**Um, o mecanismo de preço.** Como o preço se move enquanto o token ainda está no venue. Uma curva de produto constante fixa, uma curva que você parametriza, uma curva por partes que você modela segmento a segmento, ou nenhuma curva se o venue roda um leilão.

**Dois, a tabela de taxa.** Quem leva o quê, de qual lado da graduação, e se a tabela pode mudar debaixo de você depois do lançamento. A taxa de bonding curve da pump ERA 100 basis points fixos sobre negociações contra a curva, até o dia da virada em 2025-09-01 que você datou na lição passada substituí-la por faixas de market cap editáveis por admin; faça o orçamento a partir da tabela de faixas ao vivo, nunca a partir da taxa fixa histórica. O Metaplex Genesis publica 0.50% de protocolo mais 0.60% de receita do criador no modo de bonding curve dele (o modo de destaque dele é o leilão que você vai conhecer abaixo; o modo de curva é o que tem esta tabela de taxas), e uma tabela diferente pós-graduação: 0.40% de protocolo, 0.42% para os LPs, 0.04% para a Raydium no CPMM da pool de lançamento. Esses números são os que estavam nos docs deles no dia em que eu li, e tabelas de taxas são a página que apodrece mais rápido em qualquer protocolo. Releia antes de se comprometer.

**Três, a defesa.** O que, se é que existe algo, fica entre o seu lançamento e os bots. Este é o eixo que a maioria dos criadores descobre tarde demais, e é nele que esta lição gasta mais tempo.

**Quatro, o alvo de graduação.** Em qual AMM a liquidez pousa quando a curva se completa, e portanto quais designs de token são sequer legais. Este é o eixo que te come se você pulou ele, e é a razão de o relatório do R6 que você escreveu no módulo passado ser um documento de lançamento e não um exercício de design.

Repare no que não está nessa lista: o padrão de token. Qualquer um desses venues vai cunhar um token SPL para você sem reclamar. Só alguns deles vão carregar um mint Token-2022 com extensões de poder até o fim da migração, e nenhum deles vai te avisar na hora do mint. A falha aparece na última instrução.

![Uma comparação de quatro colunas entre pump.fun, Raydium LaunchLab, Meteora DBC e Metaplex Genesis em oito eixos de lançamento, com a Meteora DBC sendo o único venue documentado como suportando configs de transfer hook.](assets/v02-comparison.webp)

### pump.fun: o venue sem botões

Você já conhece este por dentro, e é por isso que ele dá uma linha de base limpa. Quatro constantes, nenhum parâmetro, uma forma para todo token que já foi lançado ali. O supply é um bilhão com seis decimals, as reservas virtuais começam em 30 SOL e 1,073,000,000,000,000 unidades base, a reserva real de token é 793,100,000,000,000, e drenar essa reserva é o que faz `complete = true` e libera um migrate sem permissão e idempotente que queima o LP. O limiar cai em cerca de 85 SOL, e você derivou ele em vez de procurar.

O dia da virada em 2025-09-01 é a parte que vale carregar para dentro desta lição. Da noite para o dia, a tabela debaixo de todo token vivo da pump mudou, e nenhum criador foi consultado, porque nenhum criador nunca teve voto. Esse é o acordo, não um escândalo. Um venue sem botões é um venue cuja política é definida por outra pessoa, permanentemente, incluindo as partes em torno das quais você precificou o seu lançamento.

A vantagem de não ter botões é real e é subestimada por quem gosta de botões: você não consegue configurar errado. Todo lançamento é o mesmo lançamento, então liquidez, bots, front ends e dashboards sabem a forma de antemão, e a superfície de integração é enorme porque ela nunca muda. Configuração zero é uma feature quando a alternativa é você, às 3 da manhã, escolhendo uma duração de cliff que você não entende.

![Um gráfico de barras agrupadas de taxas lidas em 2026-08-22, com o 1.00% da pump.fun marcado como histórico e substituído por faixas dinâmicas, o modo de curva do Genesis empilhado em 1.10% e 0.86%, e dois venues deixados como barras de espaço reservado marcadas como sem fonte.](assets/v03-chart.webp)

### LaunchLab: botões, um contrato de franquia e um NFT que cobra aluguel

O LaunchLab da Raydium é a mesma primitiva com o painel de configurações destravado, e ele já vem com duas portas. **JustSendit** é a porta dos defaults: nenhuma configuração, e a curva gradua para uma pool de AMM da Raydium com 85 SOL arrecadados. Sim, os mesmos 85 da pump. Essa coincidência merece uma pausa, porque é exatamente o formato de suposição que queima gente: dois venues convergiram no mesmo número redondo por razões diferentes, então um leitor que generaliza "launchpads graduam em 85 SOL" vai estar certo duas vezes e errado em todo venue configurável do espaço. O limiar é por venue, e nos configuráveis é por lançamento.

**LaunchLab mode** é a outra porta: controles de supply, métricas de venda, parâmetros de curva, e vesting com um cliff e uma duração de desbloqueio. Uma propriedade desses botões de vesting merece a própria frase, porque é a diferença entre um erro e uma catástrofe. Períodos de cliff e de desbloqueio são fixados no lançamento e não podem ser mudados retroativamente. Você não está configurando uma opção de dashboard. Você está escrevendo uma cláusula em um contrato que vai continuar se impondo muito depois de você ter esquecido qual número digitou. (A lição de airdrop volta a esses mesmos botões de cliff e duração pelo lado do claim, onde o vesting é um claim merkle em vez de um parâmetro de lançamento.)

Aí tem a peça que reenquadra a economia inteira, e é a razão de esta lição existir no módulo de economia de token em vez de ficar ao lado da matemática da curva. Ative a divisão de taxa pós-migração na criação e, quando o token gradua, o LaunchLab cunha um **Fee Key NFT** para a carteira do criador. A carteira que segura esse NFT consegue fazer claim de 10% de todas as taxas de negociação de LP da pool graduada. O que significa que a queima do LP não é o que você provavelmente presumiu: 90% dos tokens LP são queimados, e os 10% restantes ficam travados no Burn and Earn da Raydium, com a Fee Key como o tíquete de claim.

Isso muda o que "o LP está queimado" significa como sinal de confiança. Um LP queimado é a promessa padrão de que ninguém consegue puxar o tapete. Sob a divisão de taxa, a versão honesta é que 90% do tapete está pregado no chão e os outros 10% são uma anuidade permanente e transferível que pertence a alguém. Os docs são diretos sobre a consequência: queime ou transfira a Fee Key e o direito de fazer claim é perdido permanentemente. Então o ativo de longo prazo mais valioso de um lançamento pode ser perdido em uma faxina de carteira, e também pode ser vendido, que é um mercado que ninguém planejou e todo mundo agora tem.

O trade-off, dito sem rodeios. O LaunchLab te compra configuração, um fluxo de taxa que sobrevive à graduação, e a opção de rodar o seu próprio launchpad com marca própria em cima do programa auditado de outra pessoa. O que ele custa é que cada um desses botões é uma decisão que agora é sua para sempre, a história da queima do LP fica mais complicada de explicar aos holders, e o AMM de graduação é o Raydium CPMM, cuja allowlist de Token-2022 você já sabe de cor desde o R6. Três extensões dentro, quatro fora.

### Meteora DBC: a curva como estrutura de dados

A Dynamic Bonding Curve da Meteora é o venue que trata a curva como configuração e não como produto. Em vez de uma única curva de produto constante, a config guarda um array de pares `(sqrt_price, liquidity)`, e a pool interpola comportamento de produto constante entre eles. Por partes, em outras palavras: você desenha um trecho barato e plano para os apoiadores iniciais, depois um trecho mais íngreme, depois qualquer forma que a sua distribuição de fato queira.

É aqui que a lição ganha o segundo momento de cor, e é um bom momento para um hábito em vez de um fato. Os docs descrevem uma curva customizável de 16 pontos. O array de config on-chain tem 20 de largura. Os dois números são reais, e os dois estão na fonte. Eu puxei o arquivo de constantes em 2026-08-22:

```rust
// programs/dynamic-bonding-curve/src/constants.rs (excerpt)
pub const MAX_CURVE_POINT: usize = 16;
pub const MAX_CURVE_POINT_CONFIG: usize = 20;
const_assert!(MAX_CURVE_POINT <= MAX_CURVE_POINT_CONFIG);

// programs/dynamic-bonding-curve/src/state/config.rs (excerpt, fields elided)
#[zero_copy]
pub struct LiquidityDistributionConfig {
    pub sqrt_price: u128,
    pub liquidity: u128,
}

#[account(zero_copy)]
pub struct PoolConfig {
    // ... quote_mint, fee_claimer, leftover_receiver, fee and vesting configs ...
    pub token_type: u8, // 0 = SplToken, 1 = Token2022
    // ... the rest of the u8 flag run ...
    pub padding_2: [u8; 7], // declared, not implied: a zero-copy struct may carry no implicit padding
    pub swap_base_amount: u64,
    pub migration_quote_threshold: u64,
    pub migration_base_threshold: u64,
    // ... migration_sqrt_price, locked vesting, supply and migrated-fee fields ...
    pub curve: [LiquidityDistributionConfig; MAX_CURVE_POINT_CONFIG],
}
```

Duas constantes, dois trabalhos, e o `const_assert!` entre elas é a entrega: a própria fonte declara uma delas um teto situado abaixo da outra. A conta reserva 20 slots para o layout ter folga; a curva validada limita mais abaixo. Se você tivesse confiado só nos docs, teria acreditado que o array tinha 16 de largura e teria dimensionado um deserializador errado. Se você tivesse confiado só na struct, teria acreditado que podia passar 20 pontos e teria levado uma rejeição de uma instrução. A regra que sobrevive aos dois erros é a que você vem rodando desde a matriz de conflitos de extensões: leia a fonte, e leia o suficiente dela para saber qual constante governa qual superfície. Depois recheque na hora em que você mesmo for escrever, porque eu estou citando um repositório em uma data e repositórios se movem.

Mais dois campos naquela struct decidem se o SPROUT consegue usar este venue, e aqui a precisão sobre QUAL lado do par cada afirmação cobre importa, porque são afirmações diferentes com evidências diferentes. O `token_type` é 0 para SPL clássico e 1 para Token-2022, e o que ele comprovadamente codifica, sentado no PoolConfig ao lado de `quote_mint`, é o token program do lado do QUOTE: é assim que a DBC aceita um quote mint Token-2022.

O lado BASE, o lado em que o SPROUT viveria, se apoia em uma afirmação de documentação separada: os docs da DBC descrevem tokens base Token-2022 incluindo configurações de transfer hook, o que a torna o contraponto da Raydium nesta comparação inteira, já que um token com hook recusado pelo CP-Swap tem um caminho documentado aqui. Mas documentado não é interrogado. Na lição passada você leu o IDL da pump linha por linha para provar o caminho de create dela; esta lição não fez isso para o create do lado base da DBC, então trate o suporte a base-Token-2022-com-taxa-de-transferência como documentado-mas-não-verificado até você ler o caminho de create da DBC ou levantar uma pool na devnet com um mint base que carrega taxa. O registro de venue do lab carrega essa flag, e a decisão que ele alimenta herda a ressalva.

O `migration_quote_threshold` é o gatilho de graduação, em unidades do token de quote, e é um número que você escolhe em vez de um número que você deriva.

Lado do custo, porque um venue tão flexível não é de graça. Todo segmento é uma decisão de distribuição que você agora tem de defender, e uma curva por partes te dá muito mais jeitos de errar do que uma fixa. A matemática de pool e a estratégia de LP que te deixariam modelar esses segmentos com inteligência estão genuinamente fora de escopo aqui, e eu não vou fingir que não: o curso planejado DeFi and RWA Engineering ensina provisão de liquidez com profundidade de verdade, e é ali que modelar curva deixa de ser um cardápio e vira uma disciplina. Esta lição te leva exatamente até escolher o venue e saber quais são os botões dele.

![Um trecho anotado da fonte da Dynamic Bonding Curve da Meteora mostrando MAX_CURVE_POINT em 16 e MAX_CURVE_POINT_CONFIG em 20, com chamadas nomeando a falha que cada número causa se for o único em que você confia.](assets/v04-annotated-code.webp)

### Genesis: quando uma curva é a forma completamente errada

O Metaplex Genesis pertence a esta comparação precisamente porque o modo de destaque dele recusa a premissa. (Uma reconciliação com a seção de taxas, para que as duas não sejam lidas como contradição: o Genesis também já vem com um modo de lançamento em bonding curve, e o número de 1.10% citado antes é da tabela de taxas DAQUELE modo. Esta seção cobre o diferencial, o leilão, que é o que o registro de venue do lab modela; o modo de curva existe e não é modelado aqui.) Uma bonding curve é um mecanismo de preço com um viés específico embutido: quem compra antes paga menos, mecanicamente, sempre. Esse viés é o ponto quando você está lançando um token de comunidade e quer os apoiadores iniciais recompensados. Ele é um bug quando você está lançando algo com demanda institucional de verdade, porque converte "ser cedo" em "ser rápido", e ser rápido é um serviço que bots vendem.

O Genesis oferece um **leilão de preço uniforme** no lugar. Os lances entram durante uma janela, são ordenados por preço, e todo participante vencedor paga o mesmo preço de fechamento, definido no menor lance vencedor. Ninguém é recompensado por pousar uma transação 40 milissegundos antes de outra pessoa, porque a ordem de chegada deixa de ser uma entrada de preço. O Genesis enquadra isso como o modo para projetos estabelecidos com interesse institucional, e a arquitetura em volta é um sistema de **buckets**: buckets de entrada arrecadam SOL dos participantes, buckets de saída roteiam fundos para uma tesouraria ou um destino de vesting através de comportamentos de fim configuráveis.

O que ele te custa é justamente aquilo em que curvas são boas. Um leilão precisa que a demanda apareça dentro de uma janela, e uma janela é um problema de coordenação: você tem de fazer marketing dela, e se a janela fechar magra, você descobriu a sua curva de demanda em público. Uma bonding curve nunca tem esse modo de falha, porque ela está sempre aberta e sempre cotando. Escolha o leilão quando o lançamento tem gravidade suficiente para encher uma sala em um horário marcado. Escolha uma curva quando não tem.

![Um diagrama de fluxo de dados do Metaplex Genesis roteando o SOL dos participantes através de buckets de entrada, um comportamento de fim e buckets de saída, com uma pista paralela alocando tokens para os vencedores a um único preço de fechamento.](assets/v05-diagram.webp)

### Quatro defesas contra a mesma janela disputada

Todo mecanismo anti-snipe neste espaço está atacando a mesma janela. Do slot em que a sua pool fica negociável até o slot em que um ser humano consegue reagir é um vão de algumas centenas de milissegundos nos tempos de slot atuais, e um bot com conexão quente e uma taxa de prioridade é dono do vão inteiro. As defesas diferem em qual alavanca puxam.

**A janela de depósito.** O Alpha Vault da Meteora fica na frente do lançamento e aceita depósitos antes de a negociação abrir, em modo por ordem de chegada ou pro-rata, e então compra como um único participante na abertura. Todo depositante recebe o mesmo preço de execução. A vantagem de velocidade do bot evapora porque não há nada para disputar: a compra já aconteceu, coletivamente, a um preço que ninguém conseguiu furar. O custo é um cronograma e um teto. Você está pedindo que apoiadores comprometam capital antes de um lançamento, o que é um pedido muito maior do que clicar em comprar, e no modo pro-rata ninguém sabe o fill exato dele até a janela fechar.

**A taxa decrescente.** Comece a taxa de negociação punitiva e deixe ela cair ao longo dos primeiros minutos ou blocos. Um sniper que compra no primeiro slot paga uma taxa que come a arbitragem; um comprador normal que chega quatro minutos depois paga perto do normal. Esta é a defesa menos intrusiva, porque não muda nenhum fluxo e não pede nada da sua comunidade, e é a mais fraca, porque uma alta esperada grande o bastante ainda justifica a taxa. Ela taxa o sniping em vez de impedir.

**A primeira compra reservada e sem taxa.** O criador, ou um conjunto na allowlist, ganha uma compra ao preço de abertura da curva com as taxas dispensadas antes de a pool abrir para todo mundo. Isso garante ao time ou à comunidade uma posição lá embaixo. Também concentra supply exatamente do jeito que uma plateia de fair launch está vigiando, então te compra defesa ao custo direto da imagem pela qual você provavelmente estava lançando.

**O leilão de preço uniforme.** O Genesis, como acima. O mais forte dos quatro, porque ele não taxa nem atrasa a corrida, ele apaga a corrida removendo o tempo da função de preço. E é o mais caro, porque exige que o lançamento se comporte como um evento.

Existe uma quinta opção que as pessoas esquecem: nenhuma defesa. É isso que um lançamento simples na pump é, e é uma escolha coerente se o token é pequeno, o lançamento é silencioso, e o custo de um bot conseguir um bom fill é genuinamente menor que o custo de pedir para a sua comunidade aprender uma janela de depósito. Nomear isso como escolha é diferente de tropeçar nisso.

![Uma linha do tempo de lançamento com os primeiros slots disputados por snipers depois de a negociação abrir, e quatro defesas posicionadas em volta dessa zona: janela de depósito, primeira compra sem taxa, taxa decrescente e leilão de preço uniforme.](assets/v06-timeline.webp)

### O que a graduação de fato semeia

Aqui está a parte que todo launchpad faz por você em silêncio, e é por isso que quase ninguém consegue listá-la quando perguntam. A graduação semeia exatamente três coisas.

**O par.** Uma conta de pool no AMM de destino segurando o seu token contra o quote mint. O par é escolhido pelo venue: a pump te dá SPROUT/SOL na PumpSwap, o LaunchLab te dá SPROUT/SOL no Raydium CPMM, a DBC te dá SPROUT contra qualquer quote mint que você tenha configurado no DAMM. Você não negocia o ativo de quote na graduação. Você escolheu ele quando escolheu o venue.

**O preço inicial.** Não é um número que você define. O preço implicado por onde a curva parou. A razão final de reserva virtual na conclusão é a cotação de abertura no AMM, e é por isso que o ponto final da curva e o primeiro tick da pool são o mesmo fato econômico visto duas vezes. Em um venue com curva fixa, esse preço é determinado no instante em que você lança. Na DBC, ele é determinado pelo seu último segmento.

**A posição de LP e o destino dela.** A migração cunha tokens LP contra a liquidez semeada e depois faz algo irreversível com eles. A pump queima eles. O LaunchLab queima 90% e trava 10% atrás da Fee Key quando a divisão de taxa está ligada. A DBC queima ou trava conforme a sua config. Seja qual for a regra, ela executa dentro da transação de migração, e depois disso a posição é exatamente tão imóvel quanto a regra disse que seria.

Agora o contrafactual, que é o único jeito de sentir o que você está ganhando. Sem um launchpad você criaria a conta de pool e pagaria o rent dela você mesmo, financiaria os dois lados a partir de uma carteira que você controla, escolheria um preço de abertura por julgamento em vez de por mecanismo, receberia os tokens LP naquela mesma carteira, e aí resolveria o problema de confiança na mão: queimar eles e provar isso, ou travar eles em algum lugar e provar aquilo. Você também seria dono do problema anti-snipe inteiro sozinho, porque uma pool nova sem defesa é uma pool que é snipada no primeiro slot dela por definição. Isso são quatro trabalhos e uma prova de confiança, em troca do controle de cada um deles.

![Uma tabela de três colunas listando as cinco coisas que um launchpad semeia na graduação, quem decide cada uma, e o equivalente manual desde a criação da pool até a cobertura anti-snipe.](assets/v07-table.webp)

### O conjunto de extensões vota primeiro

O que traz a comparação inteira de volta para uma decisão que você já tomou. O conjunto roteável do SPROUT vindo do R6 é TransferFeeConfig, MetadataPointer e TokenMetadata, e a razão de serem esses três é que a allowlist de Token-2022 do Raydium CP-Swap contém exatamente cinco extensões e recusa tudo que deixa um emissor rodar código ou mover os tokens dos outros.

Leia a tabela de comparação de novo com isso na mão e o campo de venues estreita sozinho. Três dos quatro alvos de graduação recusam um transfer hook. Se o SPROUT tivesse mantido o hook dele, a pump, o LaunchLab e o Genesis reverteriam todos na migração, e não no lançamento, que é a parte cruel. Você arrecadaria o SOL, bateria o limiar, e falharia na última instrução com um token que ninguém consegue negociar e uma curva que já está completa.

Então a ordem é fixa, e é o oposto de como a maioria dos lançamentos é planejada. O conjunto de extensões decide quais AMMs de graduação são legais. Os AMMs legais decidem quais venues estão disponíveis. Os venues disponíveis te oferecem um cardápio de defesas. Você escolhe desse cardápio. Quem escolhe o venue primeiro vai acabar mudando o token para caber nele, o que é um resultado ótimo desde que tenha sido uma decisão em vez de uma descoberta.

![Um fluxograma de quatro estágios indo do conjunto de extensões para os AMMs legais, para os venues disponíveis, para a escolha de defesa, com uma seta invertida marcando o plano de trás para a frente mais comum e uma chamada de reversão na migração.](assets/v08-flowchart.webp)

## Lab: escolha o venue do SPROUT e escreva a decisão

O artefato é `sprout-launch/choose-venue.ts`, e ele é a metade de decisão do pacote `sprout-launch` cuja metade de curva você construiu na lição passada. Ele ingere o conjunto de extensões do R6 do SPROUT e o limiar derivado, rejeita todo venue cujo AMM de graduação recusaria o mint, rejeita todo venue sem defesa se você pediu uma, e imprime a decisão com as rejeições dela anexadas. A cancela: `npx tsx sprout-launch/choose-venue.ts` precisa imprimir um venue, um AMM de graduação, uma defesa nomeada e uma lista de rejeições não vazia, e sair com 0.

**1.** Trabalhe na pasta `sprout-launch/` da lição passada, ao lado do `derive-graduation.ts`. Nada novo para instalar se você fez aquele lab. Começando do zero, o runner é a mesma dependência de dev única, rechecada hoje: `tsx@4.23.12` era o npm latest em 2026-08-22, e esse número apodrece como todo pin neste curso, então rode `npm view tsx version` você mesmo no dia em que fizer o scaffold.

```bash
npm init -y
npm install -D tsx@4.23.12
```

**2.** Coloque o espaço de design em disco como dados antes de escrever qualquer lógica. O sentido de uma tabela como esta é que todo campo é uma afirmação sobre a qual você pode estar errado em um lugar em vez de cinco, e os valores de `programId` são os que o seu script de sondagem já conferiu.

```typescript
// venues.ts: the launchpad design space as data. Every field is a decision you inherit.
export type VenueId = "pump" | "launchlab" | "dbc" | "genesis";
export type Defense = "alpha-vault-window" | "decaying-fee" | "fee-free-first-buy" | "uniform-price-auction" | "none";

export interface Venue {
  id: VenueId;
  label: string;
  programId: string;
  priceMechanism: "fixed-curve" | "configurable-curve" | "piecewise-curve" | "auction";
  /** how many liquidity segments the creator controls; 0 = none */
  curveSegments: number;
  /** the AMM the pool lands on at graduation */
  graduationAmm: string;
  /**
   * Which token program the venue's CREATE path can mint the base token under.
   * This gate runs before any extension talk: last lesson you proved from pump's
   * own IDL that its create pins the classic SPL Token program and mints the
   * token itself, so a Token-2022 mint cannot exist there at all.
   */
  baseTokenProgram: "spl" | "token2022" | "both";
  /** Token-2022 extensions the graduation AMM is documented to accept */
  acceptsTransferHook: boolean;
  acceptsToken2022Quote: boolean;
  defenses: Defense[];
  threshold: { kind: "fixed" | "derived" | "configurable"; sol?: number; field?: string };
  /** fraction of LP tokens burned at migration; the rest is locked, not free */
  lpBurnedAtMigration: number;
  seededAtGraduation: string[];
}

export const VENUES: Venue[] = [
  {
    id: "pump",
    label: "pump.fun",
    programId: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
    priceMechanism: "fixed-curve",
    curveSegments: 0,
    graduationAmm: "PumpSwap (pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA)",
    baseTokenProgram: "spl", // proven from pump's IDL last lesson: create mints classic SPL itself
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["none"],
    // ~85 SOL is DERIVED from pump's constants (derive-graduation.ts), never a spec constant:
    // recording it as "fixed" would be the repeated-number mistake last lesson buried.
    threshold: { kind: "derived", sol: 85, field: "virtual reserves at completion" },
    lpBurnedAtMigration: 1,
    seededAtGraduation: ["SPROUT/SOL pair", "price implied by the curve endpoint", "LP burned"],
  },
  {
    id: "launchlab",
    label: "Raydium LaunchLab (JustSendit)",
    programId: "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
    priceMechanism: "fixed-curve", // this record is the JustSendit defaults door; LaunchLab's other door unlocks the configurable curve
    curveSegments: 1,
    graduationAmm: "Raydium CPMM",
    baseTokenProgram: "spl", // LaunchLab's create mints the base token as classic SPL; verify against its IDL the way you did pump's
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["none"],
    threshold: { kind: "fixed", sol: 85 },
    // 90% burned, 10% locked in Burn & Earn when creator fee share is on
    lpBurnedAtMigration: 0.9,
    seededAtGraduation: [
      "SPROUT/SOL pair on CPMM",
      "price implied by the curve endpoint",
      "90% LP burned, 10% locked behind the Fee Key NFT",
    ],
  },
  {
    id: "dbc",
    label: "Meteora Dynamic Bonding Curve",
    programId: "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN",
    priceMechanism: "piecewise-curve",
    curveSegments: 16,
    graduationAmm: "DAMM v1 or v2",
    // DOCUMENTED-UNVERIFIED on the base side: PoolConfig's token_type field
    // demonstrably covers the QUOTE mint; Token-2022 BASE support (incl.
    // transfer-hook configs) is a docs claim this course has not read at IDL
    // depth. Verify the create path before shipping a fee-bearing base mint.
    baseTokenProgram: "both",
    acceptsTransferHook: true,
    acceptsToken2022Quote: true,
    defenses: ["alpha-vault-window", "decaying-fee"],
    threshold: { kind: "configurable", field: "migration_quote_threshold" },
    // Per config, not absolute: DBC's LP can be burned OR locked at migration.
    // 1 models the burn configuration; adjust to the config you would ship.
    lpBurnedAtMigration: 1,
    seededAtGraduation: [
      "SPROUT/quote pair on DAMM",
      "price implied by the last curve segment",
      "LP burned or locked per config",
    ],
  },
  {
    id: "genesis",
    label: "Metaplex Genesis",
    programId: "GNS1S5J5AspKXgpjz6SvKL66kPaKWAhaGRhCqPRxii2B",
    priceMechanism: "auction",
    curveSegments: 0,
    graduationAmm: "Raydium CPMM launch pool",
    baseTokenProgram: "spl", // no documented Token-2022 base path; treat as classic-only until you verify
    acceptsTransferHook: false,
    acceptsToken2022Quote: false,
    defenses: ["uniform-price-auction"],
    threshold: { kind: "configurable", field: "auction clearing price" },
    // UNSOURCED: no doc statement on Genesis LP disposal surfaced in this
    // course's reads; recorded as burn by analogy with the launch-pool
    // pattern. Verify against Genesis docs before this cell decides anything.
    lpBurnedAtMigration: 1,
    seededAtGraduation: [
      "SPROUT/SOL pair",
      "price set by the auction clearing price, not a curve",
      "LP disposal: unsourced, verify (recorded as burn by analogy)",
    ],
  },
];
```

**3.** Agora a função de decisão, e esta é a interface que as lições seguintes vão chamar, então a forma importa mais que a regra de pontuação dentro dela. O `chooseVenue` recebe o perfil, caminha pelos venues, e coleta rejeições no caminho. Leia o caminho de falha primeiro: quando nada sobrevive, ele lança com toda rejeição listada, porque uma ferramenta de decisão que retorna um default silencioso quando a resposta é "o seu token não pode lançar em lugar nenhum" é pior que ferramenta nenhuma.

```typescript
// choose-venue.ts: SPROUT's launch decision, derived from R6 instead of vibes.
import { VENUES, Venue, VenueId, Defense } from "./venues";

/** The two prior artifacts, reduced to what a venue choice actually needs. */
export interface SproutProfile {
  /** which token program the mint lives under; SPROUT is Token-2022 */
  baseTokenProgram: "spl" | "token2022";
  /** the final routable extension set from routability-report.ts (R6) */
  extensions: string[];
  /** the threshold derived in derive-graduation.ts, in SOL; printed as the report's derived-reference line */
  graduationThresholdSol: number;
  /** true if the launch needs the community to buy before bots do */
  wantsAntiSnipe: boolean;
}

export interface Rejection {
  venue: VenueId;
  reason: string;
}

export interface LaunchDecision {
  venue: VenueId;
  programId: string;
  graduationAmm: string;
  defense: Defense;
  thresholdSol: number | null;
  seeds: string[];
  rejected: Rejection[];
}

const POWER_EXTENSIONS = ["TransferHook", "PermanentDelegate", "DefaultAccountState", "ConfidentialTransferMint"];

/** Can this venue hold the mint at all, and will its graduation AMM accept it? */
export function venueAcceptsMint(venue: Venue, profile: SproutProfile): Rejection | null {
  // Gate 0, before any extension talk: the venue's create path must be able to
  // mint under the base token program at all. This is the check last lesson's
  // IDL read made unavoidable: pump pins classic SPL and mints the token
  // itself, so a Token-2022 fee mint like SPROUT can never exist there.
  if (profile.baseTokenProgram === "token2022" && venue.baseTokenProgram === "spl") {
    return {
      venue: venue.id,
      reason: "create path mints classic SPL only; a Token-2022 mint (SPROUT carries TransferFeeConfig) cannot exist on this venue",
    };
  }
  const extensions = profile.extensions;
  if (extensions.includes("TransferHook") && !venue.acceptsTransferHook) {
    return {
      venue: venue.id,
      reason: `${venue.graduationAmm} rejects TransferHook; pool creation reverts at migration`,
    };
  }
  const otherPower = extensions.filter((e) => e !== "TransferHook" && POWER_EXTENSIONS.includes(e));
  if (otherPower.length > 0) {
    return {
      venue: venue.id,
      reason: `${venue.graduationAmm} has no documented acceptance for ${otherPower.join(", ")}`,
    };
  }
  return null;
}

export function chooseVenue(profile: SproutProfile): LaunchDecision {
  const rejected: Rejection[] = [];
  const eligible: Venue[] = [];

  for (const venue of VENUES) {
    const ammVerdict = venueAcceptsMint(venue, profile);
    if (ammVerdict) {
      rejected.push(ammVerdict);
      continue;
    }
    if (profile.wantsAntiSnipe && venue.defenses.every((d) => d === "none")) {
      rejected.push({
        venue: venue.id,
        reason: "no first-party anti-snipe defense; you would be building one yourself",
      });
      continue;
    }
    eligible.push(venue);
  }

  if (eligible.length === 0) {
    throw new Error(
      `No venue survives SPROUT's extension set [${profile.extensions.join(", ")}].\n` +
        rejected.map((r) => `  ${r.venue}: ${r.reason}`).join("\n") +
        "\nChange the token or change the requirement. There is no third option.",
    );
  }

  // TODO (yours): this tie-break prefers curve control. Justify it or replace it.
  const winner = eligible.reduce((best, v) => (v.curveSegments > best.curveSegments ? v : best), eligible[0]);
  const defense = winner.defenses.find((d) => d !== "none") ?? "none";

  return {
    venue: winner.id,
    programId: winner.programId,
    graduationAmm: winner.graduationAmm,
    defense,
    // A configurable threshold has NO number to inherit: pump's derived 85 is
    // pump's, and printing it under DBC would be the repeated-number mistake.
    thresholdSol: winner.threshold.kind === "configurable" ? null : (winner.threshold.sol ?? null),
    seeds: winner.seededAtGraduation,
    rejected,
  };
}
```

**4.** Depois a metade de relatório e a cancela. A lista `seeds` não é decoração: é a linha do "o que é semeado" que a avaliação pede, impressa a partir do registro de venue para não conseguir se afastar do venue que você de fato escolheu.

```typescript
// choose-venue.ts, continued.
export function renderDecision(profile: SproutProfile, decision: LaunchDecision): string {
  const lines = [
    "# SPROUT launch decision",
    "",
    `Extension set (R6): ${profile.extensions.join(", ")}`,
    `Venue:              ${decision.venue}  (${decision.programId})`,
    `Graduation AMM:     ${decision.graduationAmm}`,
    `Anti-snipe:         ${decision.defense}`,
    `Threshold:          ${decision.thresholdSol !== null ? `${decision.thresholdSol} SOL` : "configurable: you set migration_quote_threshold; there is no venue default to inherit"}`,
    `Derived reference:  ~${profile.graduationThresholdSol} SOL (last lesson's pump-constants derivation; on a configurable venue it is your starting anchor, never an inherited default)`,
    "",
    "Seeded at graduation:",
    ...decision.seeds.map((s) => `  - ${s}`),
    "",
    "Rejected:",
    ...decision.rejected.map((r) => `  - ${r.venue}: ${r.reason}`),
  ];
  return lines.join("\n");
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const sprout: SproutProfile = {
    baseTokenProgram: "token2022",
    extensions: ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"],
    graduationThresholdSol: 85,
    wantsAntiSnipe: true,
  };
  const decision = chooseVenue(sprout);
  console.log(renderDecision(sprout, decision));

  if (decision.defense === "none") {
    console.error("\nGATE FAIL: wantsAntiSnipe is set but the chosen venue ships no defense.");
    process.exit(1);
  }
  console.log("\nAll gates pass.");
}
```

**5.** Rode, depois prove que as cancelas são reais.

```bash
npx tsx sprout-launch/choose-venue.ts
```

Você deve ver `Venue: dbc`, `Anti-snipe: alpha-vault-window`, uma linha de limiar que diz configurável em vez de pegar um número emprestado, três linhas de semeadura, três rejeições todas nomeando o caminho de create de SPL clássico, e `All gates pass` com saída 0.

Repare que esta resposta não é a resposta da lição passada, e a diferença é o sentido inteiro de ampliar o quadro. O `derive-graduation.ts` fez uma pergunta, "qual AMM consegue legalmente segurar o mint do SPROUT," comparou dois candidatos, e parou no Raydium CP-Swap, que é onde o LaunchLab gradua. Este script faz duas perguntas a mais em cima: "o caminho de create do próprio venue consegue cunhar um token de taxa Token-2022, para começar," que o pin de SPL clássico do LaunchLab reprova do mesmo jeito que o da pump, e "o venue sobrevivente defende os primeiros slots." A DBC é o que sobrevive às duas, o que move o destino para o DAMM. Nenhuma das duas rodadas está errada. A primeira respondeu uma pergunta mais estreita com uma lista de candidatos mais estreita, e uma decisão que muda quando você acrescenta um eixo é uma decisão que estava de fato escutando.

Agora faça falhar de três jeitos, porque um checkpoint que não pode falhar nunca foi um checkpoint. Adicione `"TransferHook"` a `extensions` e repare que a linha da DBC sobrevive enquanto as outras três rejeições continuam de pé. Defina `wantsAntiSnipe: false` e repare que a pump e o LaunchLab NÃO voltam para o conjunto elegível; a cancela do programa base rejeitou os dois antes de a cancela de defesa sequer rodar, e nenhuma flag de preferência consegue conjurar um mint Token-2022 em um venue de SPL clássico. Para ver a cancela de defesa morder de verdade, vire o perfil inteiro para um token clássico simples (`baseTokenProgram: "spl"`, `extensions` vazio, `wantsAntiSnipe: true`) e veja a pump e o LaunchLab serem rejeitados só pela defesa. Por fim defina `extensions` como `["PermanentDelegate"]` com o perfil Token-2022 e observe o throw: nenhum venue sobrevive, toda rejeição impressa, nenhum default retornado. Devolva os valores reais do SPROUT quando terminar.

![Um fluxograma do choose-venue.ts passando cada venue por uma cancela de extensão e uma cancela de preferência, com as rejeições coletadas à parte, um desempate, e uma cancela de saída para escolhas sem defesa.](assets/v09-flowchart.webp)

## Challenge

A metade solo é o julgamento que a ferramenta deliberadamente não faz por você, e ela sai como escrita, não como código.

Primeiro, substitua o desempate. A linha marcada com TODO prefere o venue elegível que expõe mais segmentos de curva, que é uma regra defensável e não a única. Escreva a regra em que você de fato acredita, em código, e ponha um comentário acima dela dizendo o que ela otimiza e o que ela sacrifica. Candidatas para as quais você tem material para argumentar: preferir a defesa mais forte, preferir o venue cuja queima de LP é total em vez de parcial, preferir a maior superfície de integração, preferir o venue cuja tabela de taxa não pode mudar debaixo de você. Qualquer uma delas ganha de "mais segmentos" para alguns lançamentos.

Segundo, escreva a decisão de lançamento do SPROUT como prosa, e faça ela sobreviver a uma leitura hostil. Quatro coisas têm de estar nela, e a barra de aceitação é a do brief. Nomeie o venue e a única defesa anti-snipe que você está adotando. Justifique por que o conjunto roteável do R6 permite o AMM de graduação daquele venue, nomeando as extensões e a allowlist em vez de acenar para elas. Diga em uma linha o que a graduação semeia para você: o par, o mecanismo de preço que define a cotação de abertura, e a regra de destino do LP incluindo se a queima é total. Depois diga o que você teria de semear na mão no lugar, nas mesmas unidades, para que o serviço real do launchpad seja uma quantidade e não um vibe.

Terceiro, e este é o que separa uma decisão de uma preferência: escreva o parágrafo que mudaria a sua cabeça. Nomeie o fato específico que, se acabasse sendo outro, vira a sua escolha de venue. O meu seria o suporte a transfer hook que eu atribuí à DBC. A minha ordenação inteira se apoia em isso ser um caminho documentado e funcionando hoje em vez de uma frase de roadmap, e se você verificar e descobrir que é um flag de config que ninguém nunca entregou através de uma migração, a comparação inteira se reembaralha e o design do SPROUT fica mais simples à força. Ache o seu. Uma decisão cujo autor não consegue nomear a condição que a quebra é só uma preferência.

Um pedido antes de você fechar a pasta. Todo número de venue nesta lição está datado de 2026-08-22 e foi lido de uma página de docs ou de um repositório, não de um lançamento que eu rodei: o limiar de 85 SOL do JustSendit, o claim de 10% da Fee Key, a divisão 90/10 do LP, a tabela de taxas do Genesis, as duas constantes da Meteora. A economia de launchpad apodrece mais rápido que quase qualquer outra coisa neste curso, porque a divisão de taxa é o produto. Se as suas próprias leituras discordarem das minhas, poste a afirmação exata e o que você encontrou no canal de feedback do curso. O hábito que você está construindo não é "conhecer os venues". É "re-derivar a tabela de venues antes de todo lançamento", e um aprendiz que pega um número velho é esse hábito funcionando em voz alta.

Você escolheu onde o SPROUT gradua e o que isso semeia. Agora o outro lado da distribuição: colocar o token em milhares de mãos sem um custo de rent do tamanho de uma casa.
