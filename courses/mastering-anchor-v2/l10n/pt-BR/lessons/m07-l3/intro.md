# A checklist de auditoria como um lab, e o fuzz como a avaliação

Na lição passada você drenou o escrow com os seus próprios testes de exploit, depois corrigiu ele: um pin de `address` no chamador, um pin de `address` no `UncheckedAccount` substituível, e `checked_sub` no débito do vault, até todo ataque que você conseguia imaginar falhar. Você não atacou o swap. Você mapeou as mesmas sete classes nos campos dele e deixou por isso mesmo. É essa última oração a armadilha. "Todo ataque que você conseguia imaginar" é uma cerca construída na altura exata da sua própria imaginação, e a imaginação de um atacante não é a sua.

Então, antes de você ler mais um parágrafo, faça isto. Abra `programs/token-ticket-swap/src/lib.rs`, ache `swap_arcade_for_tickets`, e responda uma pergunta sobre a struct de contas dele: existe uma linha, uma linha de verdade para a qual você consegue apontar, que comprove que `reserve_ticket` é a reserva do próprio pool e não uma conta de token que o chamador escolheu? Não "o Anchor provavelmente trata disso." Um número de linha, ou a palavra `FAIL`. Escreva isso num arquivo de rascunho. Esse é o primeiro item da sua checklist de auditoria, e você acabou de começar o lab.

```bash
# Find the swap handler you are about to audit, and start the checklist file.
grep -n "fn swap_arcade_for_tickets" programs/token-ticket-swap/src/lib.rs
: > audit-checklist.txt   # one row per check: write a line number, or the word FAIL
```

Se a linha que você foi buscar é um constraint `token::authority = pool`, olhe de novo, porque é essa a armadilha da pergunta. Qualquer um consegue criar uma conta de token SPL cuja authority é o PDA do pool — o `InitializeAccount` recebe o owner como um argumento simples, sem exigir assinatura do owner — e as duas reservas dividem essa authority de qualquer jeito. Authority diz quem pode gastar de uma conta, não qual conta o pool queria dizer, que é precisamente a classe de substituição da lição passada. A linha que *responderia* a pergunta é o pin que o mapeamento da lição passada exigia para o R4: o *address* da reserva checado contra uma reserva que o pool registrou no init. Vá procurar por ela e você acha uma coisa mais afiada que um constraint faltando. O `Pool` armazena os dois mints e o bump dele e nada mais — a m05-l2 disse isso mesmo quando notou que as reservas ficam "em endereços que ninguém deriva" — então não existe address registrado on-chain contra o que checar. A linha não existe, e ela não pode ser escrita até o pool começar a registrar um. Escreva `FAIL`. Você acabou de fazer o primeiro achado de verdade da checklist antes de a checklist sequer começar, e é uma correção de duas partes em vez de uma linha só, que é exatamente o tipo de coisa que uma lida corrida passa batido e uma checklist não. O ponto não é a dificuldade. O ponto é que você olhou, e que agora você tem um item de um arquivo que diz isso.

Esta lição transforma segurança de uma sensação num procedimento de duas partes. A parte um é uma checklist que você roda na mão, item por item, contra o swap: ela deixa a sua revisão repetível e te força a nomear a linha que satisfaz cada garantia. A parte dois é o `anchor fuzz`, que roda o ataque por você. Você aponta ele para o swap, vai embora, e volta para um artefato de crash de uma entrada que você nunca teria digitado. O reenquadramento que carrega a lição inteira: uma rodada limpa de fuzz não é tranquilidade, é silêncio. O crash é a vitória, porque um crash que você achou é um bug que o atacante não achou.

![Um card de duas colunas comparando o que a checklist manual de auditoria pega, deixa passar e custa, contra os mesmos três itens para rodadas automatizadas de anchor fuzz.](assets/v01-comparison.webp)

Aqui está o que eu te entrego e o que eu não entrego. Eu rodo a checklist inteira com você e levanto o primeiro loop de fuzz passo a passo, incluindo semear um bug de propósito para você ver o fuzzer pegar ele. A asserção de invariante no centro da bancada de teste, você termina sozinho a partir de um scaffold com um buraco nele. Depois, semear um bug novinho em folha, prever se o fuzzer vai achar ele, e dar replay no crash para confirmar é inteiramente seu. O entregável no fim é uma checklist preenchida, um artefato de crash e o replay dele, um patch, e um re-fuzz limpo com um relatório de cobertura.

## Transformando segurança em procedimento

### Rodando a checklist como um lab

Uma code review que vive na sua cabeça não é repetível, e não é revisável por mais ninguém. A correção é chata e funciona: um conjunto fixo de itens, e para cada item você ou escreve o número da linha que o satisfaz ou escreve `FAIL` e vai consertar. Sete itens cobrem as classes de bug que respondem pela maior parte do que auditorias de Solana de verdade acham. Eles não são os mesmos sete das classes sobreviventes da lição passada, e a diferença é deliberada: a lista da lição passada era organizada por *classe de bug*, esta aqui por *coisa que você olha num arquivo*. Duas das classes da lição passada se dobram no item 1 aqui, o reuso de `init_if_needed` não tem item porque o swap não tem `init_if_needed`, e dois itens abaixo (bumps canônicos, discriminators) são checagens que o compilador faz por você em boa parte mas que não custam nada para confirmar. Rode eles contra o swap em ordem.

| # | Checagem | Condição de aprovação | Linha no swap |
|---|-------|----------------|-----------|
| 1 | Signer e owner presentes | Toda authority é um `Signer`; toda conta tipada tem uma checagem de owner (implícita em `Account<T>`, explícita para `UncheckedAccount`) | ? |
| 2 | Todo `UncheckedAccount` fixado | Cada conta crua carrega `address`, `owner`, ou um `constraint` do lado do handler | ? |
| 3 | Alvos de CPI validados | O token program que você invoca é um `Interface`/`Program` tipado, não uma chave fornecida pelo chamador | ? |
| 4 | Matemática checada em todo lugar | Nenhum `+ - * /` cru em saldos ou reservas; só `checked_*` com um erro no `None` | ? |
| 5 | Fechar-e-zerar na derrubada | Fechar uma conta zera os dados dela e recupera os lamports para que ela não possa ser ressuscitada | ? |
| 6 | Bumps canônicos armazenados | Os bumps de PDA são lidos do estado armazenado, nunca re-encontrados a cada chamada | ? |
| 7 | Discriminators sensatos | Os discriminators de conta são distintos e não triviais, então confusão de tipo é impossível | ? |

![Um item de checklist resolvido citando a linha 41 de swap.rs, ao lado do UncheckedAccount que passa, fixado por um constraint address, e da versão que falha, que não fixa nada.](assets/v02-annotated-code.webp)

O ponto de escrever o número da linha é que ele defende contra o autoengano mais comum em revisão, que é assumir que o framework fez uma coisa que ele não fez. O Anchor V2 de fato mata várias destas em tempo de compilação. O `Account<T>` exige `T: Pod` com um layout sem preenchimento, então uma leitura de confusão de tipo falha ao compilar em vez de ler bytes errado em silêncio. Contas mutáveis duplicadas são rejeitadas durante a validação de contas, a não ser que você faça opt-in com o deliberadamente feio `unsafe(dup)`. Essas são garantias de verdade que você pode citar. Mas o item 2 é exatamente aquele em que o framework não vai te salvar: os docs do V2 são diretos em dizer que o `UncheckedAccount` continua sem fazer validação nenhuma, e que você mesmo tem que parear ele com `address`, `owner` ou um `constraint`. A checklist existe para fazer você olhar aquela linha e confirmar que ela está lá.

Alguns itens merecem um olhar específico no swap, porque são os que as pessoas passam batido. Item 3, alvos de CPI: o swap move tokens através de uma CPI, e o token program que ele invoca tem que ser um `Interface` ou `Program` tipado, nunca um address pelado que o chamador passou, ou um atacante te entrega um programa sósia e a sua "transferência" roda o código dele. Item 5, fechar-e-zerar: se o pool pode ser derrubado, fechar ele tem que zerar os dados além de recuperar os lamports, senão um ataque de ressurreição reinicializa bytes obsoletos numa conta nova com saldos antigos. Item 6, bumps canônicos: você armazenou o bump do pool no estado lá quando construiu o pool; o item 6 confirma que o handler de `swap` lê aquele bump armazenado em vez de chamar `find_program_address` de novo, o que é tanto um custo de CU quanto uma armadilha sutil de corretude se as seeds mudarem algum dia. Escreva a linha, ou escreva `FAIL`. Um item do qual você "está bem certo" é um `FAIL` que você ainda não admitiu.

![Uma tabela marcando quais dos sete itens de auditoria o compilador do V2 ajuda e quais continuam inteiramente sob responsabilidade do desenvolvedor.](assets/v03-comparison.webp)

### Por que o fuzzer pega o que a sua revisão não consegue

Comece pelo limite honesto do que você acabou de fazer. A checklist é determinística e rápida, mas ela só consegue checar as classes de bug que você já nomeou. O bug que é entregue é, quase por definição, aquele que ninguém nomeou. Então a pergunta é: como você acha um bug que você não consegue imaginar? Percorra as respostas ingênuas primeiro, porque cada uma falha de um jeito que aponta para a ferramenta de verdade.

A primeira resposta ingênua é "escreva mais testes." Ela falha pela mesma razão que a checklist tem um teto: um teste afirma um comportamento que você pensou, então a sua suíte de testes é exatamente tão imaginativa quanto você, e nada mais. A segunda resposta ingênua é "jogue entradas aleatórias em cada instrução." Melhor, porque a aleatoriedade não é limitada pela sua imaginação, mas ela falha em qualquer coisa que precise de uma sequência: dispare um `swap` aleatório contra um pool novo dez milhões de vezes e você nunca vai alcançar o estado em que o pool foi drenado até virar uma lasca e uma troca arredonda para o lado errado, porque esse estado só existe depois de uma cadeia específica de trocas anteriores. A terceira resposta ingênua, "sequências aleatórias de instruções," está mais perto, mas aleatoriedade pura fica vagando: ela gasta quase todo o tempo re-explorando estados rasos e quase nunca tropeça no estado profundo e estreito onde o bug mora.

A ferramenta que sobrevive às três falhas é fuzzing stateful guiado por cobertura, e cada palavra é estrutural. Stateful, para que o mundo persista através de uma cadeia de ações e estados profundos se tornem alcançáveis afinal. Guiado por cobertura, para que o fuzzer não fique vagando: ele observa quais branches do seu programa compilado cada entrada alcançou e direciona para entradas que alcançam branches novos, transformando uma caminhada aleatória numa busca dirigida. É essa combinação que o `anchor fuzz` te dá, e é por isso que uma máquina acha entradas que você nunca acharia.

![Uma comparação descartando mais testes, entradas aleatórias e sequências aleatórias, um de cada vez, deixando o fuzzing stateful guiado por cobertura como a ferramenta sobrevivente.](assets/v04-comparison.webp)

### O que de fato roda quando você digita `anchor fuzz`

Antes de confiar a segurança do seu programa a uma ferramenta, saiba o que ela é. O `anchor fuzz` não é um wrapper fino em volta de bytes aleatórios. Ele roda o Crucible, um fuzzer guiado por cobertura construído pela Asymmetric Research e ligado na CLI do Anchor como um subcomando. Embaixo do Crucible fica um motor de fuzzing LibAFL dirigindo um runtime LiteSVM em processo, com cobertura de arestas sBPF realimentando a seleção de entradas. Essa última parte é a diferença entre um fuzzer que se debate e um que aprende: cobertura de arestas quer dizer que o fuzzer vê quais branches do seu programa compilado cada entrada alcançou, e ele direciona para entradas que alcançam branches novos. Aleatório vira dirigido.

![Um diagrama de pilha colocando o anchor fuzz sobre o Crucible, um motor LibAFL e um runtime LiteSVM, com cobertura de arestas sBPF realimentando a mutação.](assets/v05-diagram.webp)

O motor tem um trabalho que um teste unitário não consegue fazer: ele gera sequências de ações, não entradas isoladas. Você descreve as ações que o seu programa suporta (deposit, swap, withdraw) e as propriedades que sempre têm que valer (os invariantes), e o fuzzer escolhe quais ações disparar, em que ordem, com quais argumentos, e depois checa cada invariante depois de cada ação. Um bug que precisa de três chamadas específicas numa ordem específica para aparecer é um bug que os seus testes escritos na mão quase nunca alcançam, porque você teria que imaginar ele primeiro para escrever ele.

### Stateless versus stateful, e por que a flag importa

O default do Crucible é *stateless*, e o nome é mais preciso do que soa. Stateless não quer dizer uma ação por rodada. Cada iteração clona o snapshot pós-`setup` e executa uma sequência mutada inteira contra aquela cópia nova, até `--max-actions` (default 8), e depois joga o mundo fora. Então sequências estão na mesa por padrão. O que *não* está na mesa é profundidade: cada iteração recomeça do mesmo snapshot raso, então um estado que leva quarenta trocas para alcançar está quarenta vezes mais longe do que o orçamento permite, e a busca re-paga os primeiros oito passos em cada tentativa isolada.

Então a pergunta de verdade é mais estreita que "a entrada quebra isso." Ela é: algum *estado* alcançável quebra isso, incluindo estados que só existem lá no fundo de uma cadeia de chamadas por outro lado válidas? Um pool drenado-e-depois-recarregado, uma posição parcialmente inicializada, um resto de arredondamento que se acumula ao longo de quarenta trocas. É isso que o `--stateful` liga. Em modo stateful o Crucible mantém um pool indexado por cobertura de estados vivos do programa (`--pool-size`, default 256,000) e aplica uma ação mutada por iteração a um estado que ele pegou daquele pool, então o progresso se acumula em vez de resetar: as cadeias crescem até `--max-depth` (default 15) e os estados profundos se tornam alcançáveis afinal. A Asymmetric Research reporta um ganho de vazão de aproximadamente uma ordem de magnitude com isso, pago em memória conforme o pool cresce com a cobertura.

![Uma comparação de fuzzing stateless, que descarta o snapshot dele a cada iteração, contra fuzzing stateful, que mantém um pool de estados vivos e estende eles.](assets/v06-comparison.webp)

Esquecer o `--stateful` é o modo de falha silencioso. A sua rodada volta limpa, você se sente seguro, e a metade profunda do espaço de estados nunca esteve no orçamento. Limpo sem `--stateful` quer dizer "nenhum bug achado dentro de oito ações a partir de um pool novo," que é uma afirmação muito menor do que a que você acha que está fazendo.

## Lab: o loop de crash-depois-limpo

Tudo abaixo roda contra o swap, o único programa que a lição passada classificou mas nunca atacou de verdade. Trabalhe nisso em ordem. Os primeiros seis passos eu faço com você; o buraco de invariante no passo cinco é onde o recuo começa.

### 1. Instale a CLI que de fato carrega o `anchor fuzz`

Acerte este fato de toolchain antes de digitar qualquer coisa, porque errar ele custa uma tarde. O Anchor tem duas linhas vivas que este curso não para de nomear: o release candidate 2.0.0-rc.1 contra o qual você vem construindo, e a linha V1, que passou do 1.1.2 para o **1.2.0, lançado em 2026-09-04**. **O `anchor fuzz` anda na linha V1, não no RC.** Checado em 2026-09-07 contra o crate publicado: o manifesto do `anchor-cli` 1.2.0 depende de `crucible-fuzz-cli = "0.2.1"` e o enum de comandos dele despacha `Command::Fuzz` para lá, enquanto a CLI 2.0.0-rc.1 entrega `anchor test --profile`, `anchor debugger` e `anchor coverage` e não tem subcomando `fuzz` nenhum. O 1.1.2 que muitas máquinas carregam também não tem: o Crucible aterrissou depois daquela tag.

Então você instala uma segunda CLI. Até o 1.2.0 ser entregue, isso queria dizer um build git com `--branch master`; agora é um release fixado, o que é estritamente melhor porque uma versão lançada não pode se mover embaixo de você. Os dois builds instalam um binário chamado `anchor`, então mande este aqui para a raiz dele mesmo em vez de deixar ele sobrescrever o seu RC, e ponha essa raiz primeiro no `PATH` pela duração desta lição:

```bash
# The CLI that carries `anchor fuzz` (Crucible) is the V1 line, not the V2 RC.
cargo install anchor-cli --version 1.2.0 --locked --root ~/.anchor-fuzz
export PATH="$HOME/.anchor-fuzz/bin:$PATH"   # this shell only; drop it to get the RC back
anchor fuzz --help   # unknown-subcommand error = you are running the wrong CLI
```

Freshness note, e leia ela antes de fixar: as duas linhas se movem, e qual delas carrega o fuzzer é exatamente o tipo de coisa que muda entre releases. Rode `anchor fuzz --help` de novo contra qualquer CLI que você tenha antes de concluir que o subcomando está faltando; quando a árvore do V2 adotar o Crucible, essa dança de duas CLIs volta a ser uma só. Nada rio abaixo nesta lição depende de qual CLI hospeda ele, porque a bancada de teste, os invariantes e os artefatos de crash são todos do Crucible. Se a sua CLI não tem `anchor fuzz`, instale a CLI própria do Crucible e ponha `crucible` no lugar de `anchor fuzz` em todo comando abaixo:

```bash
git clone https://github.com/asymmetric-research/crucible
cd crucible && cargo install --path crates/crucible-fuzz-cli
```

Uma coisa que *não* muda com a CLI que você instala: o Crucible rastreia arestas sBPF no `.so` compilado e gera os bindings tipados de chamada dele a partir de uma IDL padrão do Anchor, então ele não liga para qual linha do Anchor construiu o programa para o qual você aponta ele. O que te dá a regra prática para o resto desta lição: construa o swap num shell *sem* o override de `PATH`, para que o `anchor build` continue sendo o V2 RC, e rode todo comando `anchor fuzz` no shell que tem ele.

### 2. Rode a checklist de auditoria e registre cada item

Volte para a tabela acima e preencha a última coluna. Vários itens já deveriam passar no swap porque o módulo 5 construiu eles desse jeito: o token program é um `Interface` tipado, a matemática de reserva é `checked_*`, o bump do pool está armazenado. A lição passada só *mapeou* as classes restantes nos campos do R4 — ela corrigiu o escrow, não o swap. Não aceite nada disso na confiança, que é a disciplina inteira do item.

O item 1 é o `FAIL` para o qual a abertura já te levou, e é aqui que você corrige ele, antes de fuzzar, porque o fuzzer está a ponto de se apoiar exatamente neste tipo de brecha. São três edições, todas formas que você já escreveu antes:

```rust
// 1. programs/token-ticket-swap/src/lib.rs - the pool records what it owns.
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,     // 32
    pub ticket_mint: Address,     // 32
    pub arcade_reserve: Address,  // 32  NEW
    pub ticket_reserve: Address,  // 32  NEW
    pub bump: u8,                 //  1
    pub _pad: [u8; 7],            //  7  explicit Pod padding (129 -> 136)
}

// 2. in init_pool, beside the two mints and ctx.bumps.pool:
pool.arcade_reserve = *ctx.accounts.reserve_arcade.address();
pool.ticket_reserve = *ctx.accounts.reserve_ticket.address();

// 3. in SwapArcadeForTickets, pin each reserve to the record. Keep the token::
//    lines: they say what the account holds, the address says WHICH one it is.
#[account(
    mut,
    address = pool.arcade_reserve @ SwapError::WrongReserve,
    token::mint = mint_arcade,
    token::authority = pool,
)]
pub reserve_arcade: InterfaceAccount<TokenAccount>,
#[account(
    mut,
    address = pool.ticket_reserve @ SwapError::WrongReserve,
    token::mint = mint_ticket,
    token::authority = pool,
)]
pub reserve_ticket: InterfaceAccount<TokenAccount>,
```

Acrescente uma variante `WrongReserve` ao `SwapError` enquanto você está aí dentro. Uma consequência para esperar em vez de descobrir: o `Pool` acabou de crescer 64 bytes, então qualquer conta de pool criada antes desta edição não casa mais com o `INIT_SPACE` e não vai carregar. Todo pool que você fez até agora mora dentro de um teste LiteSVM que constrói um novo a cada rodada, então aqui isso não te custa nada — mas note a forma da armadilha para depois, porque o PDA do pool deriva do `[POOL_SEED]` sozinho e é portanto um endereço fixo por programa: ponha um pool num cluster e os únicos caminhos de volta são fechar ele ou fazer deploy num program id novo. Não existe resize no lugar neste caminho. O lab de cliente do módulo 8 decodifica este registro com `fetchPool`, então os dois addresses de reserva que você começa a armazenar aqui são os que ele lê de volta.

Agora o resto. Escreva o número da linha que comprova cada item restante. O item 2 é o próximo para olhar com força: ache todo `UncheckedAccount` na struct de contas e confirme que cada um tem um `address`, `owner` ou `constraint`. Se algum estiver pelado, isso é um `FAIL` também, e ele é corrigido aqui também.

Checkpoint: o `audit-checklist.txt` tem sete itens e cada item carrega um número de linha, não um branco e não um talvez. Todo `FAIL` que você escreveu está corrigido e re-checado antes do passo 3.

Uma checklist verde é o bilhete de entrada, não a linha de chegada. Ela comprova que as classes estruturais estão tratadas. Ela não diz nada sobre se a matemática do seu swap preserva o produto constante, e é para isso que o fuzzer serve.

### 3. Gere o scaffold da bancada de teste de fuzz

```bash
# Generate a fuzz harness template for the `swap` program.
anchor fuzz init token_ticket_swap
```

Isso escreve um workspace de fuzz autônomo em `fuzz/token_ticket_swap/` (bancada de teste em `src/main.rs`, a IDL do programa em `idls/`, artefatos de crash depois em `crashes/`) com um `Cargo.toml` que depende da biblioteca de bancada do Crucible. Duas coisas naquele arquivo para checar na mão, porque as duas mordem em silêncio:

```toml
# fuzz/token_ticket_swap/Cargo.toml
[dependencies]
crucible-fuzzer = "0.2.1"    # harness library; version-locked with the CLI (crucible-fuzz-cli 0.2.1)

[features]
constant_product_holds = []  # ONE feature per fuzz test, named EXACTLY like the test function
```

A linha de feature é aquela na qual as pessoas perdem uma hora: todo teste de fuzz tem que ser declarado como uma feature cujo nome casa com o nome da função de teste caractere por caractere, ou a CLI não vai achar o teste que você está tentando rodar.

Freshness note: `0.2.1` é o stable atual tanto do `crucible-fuzzer` quanto do `crucible-fuzz-cli` no crates.io em 2026-08-22, e uma linha `0.3.0-alpha.1` já está publicada. Eles se movem juntos, então re-cheque o que o scaffold escreve depois de qualquer rebuild da CLI em vez de assumir este pin.

Gerar o scaffold também libera o resto da família de comandos do `anchor fuzz`, e vale ver o mapa inteiro agora para você saber para que serve cada um quando precisar deles depois no loop.

![Uma tabela dos subcomandos do anchor fuzz (init, run, list, show, cmin, tmin) com as flags deles, notando o anchor coverage como uma leitura separada.](assets/v07-table.webp)

### 4. Semeie um overflow conhecido para você ver o fuzzer merecer o salário

Não fuzze um programa limpo primeiro. Semeie um bug que você entende, confirme que o fuzzer pega ele, e só então confie numa rodada limpa. Esta é a mesma disciplina de ver um teste falhar antes de fazer ele passar.

Aqui está a matemática de produto constante do swap. Este é o `swap_out`, a mesma função que você carrega desde que construiu o R4, movida para o `src/math.rs` dela mesma nesta lição, para que a edição semeada seja um diff de uma linha num arquivo que mais nada toca. O invariante é `k = reserve_in * reserve_out`, e uma troca nunca pode deixar o `k` encolher. Como o swap cobra 0.3%, a taxa fica no pool, então na prática o `k` cresce um pouco a cada troca; `k_now >= k_before` é a asserção que é verdadeira de qualquer jeito.

Aquela asserção também cobra uma dívida do módulo 5. A m05-l3 avisou que um mint com taxa de transferência anula caladamente a suposição de recebeu-o-que-você-mandou, então o pool credita menos do que o trader mandou "e o seu invariante deriva" — e depois largou o fio, porque você ainda não tinha invariante nenhum. É este. Aponte o swap para um mint de arcade carregando uma taxa de transferência e a reserva cresce menos que `amount_in` enquanto o lado do ticket paga um `out` cotado a partir do `amount_in` inteiro: o `k` genuinamente encolhe, e `k_now >= k_before` é a linha que fica vermelha. O aviso órfão do módulo 5 e a asserção que você está a ponto de escrever são o mesmo fato, dois módulos de distância.

```rust
// programs/token-ticket-swap/src/math.rs  (correct: the swap_out you built, with its fee)
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> Result<u64> {
    // 997/1000 is the 0.3% fee: the withheld 3/1000 stays in the pool, which is
    // exactly why k grows rather than staying equal.
    let amount_in_with_fee = (amount_in as u128)
        .checked_mul(997)
        .ok_or(SwapError::Overflow)?;

    // The product is held in u128 so it cannot wrap a u64.
    let numerator = amount_in_with_fee
        .checked_mul(reserve_out as u128)
        .ok_or(SwapError::Overflow)?;

    let denominator = (reserve_in as u128)
        .checked_mul(1000)
        .ok_or(SwapError::Overflow)?
        .checked_add(amount_in_with_fee)
        .ok_or(SwapError::Overflow)?;

    let out = numerator
        .checked_div(denominator)
        .ok_or(SwapError::DivByZero)?;

    u64::try_from(out).map_err(|_| SwapError::Overflow.into())
}
```

Agora semeie o bug. Troque a linha `numerator` checada por um multiply `u64` cru:

```rust
// programs/token-ticket-swap/src/math.rs  (seeded bug - DO NOT SHIP)
let numerator = ((amount_in * 997) * reserve_out) as u128; // u64 math wraps FIRST; the cast launders it
```

Um `u64 * u64` que excede o `u64::MAX` dá wrap em vez de promover, então para reservas grandes-mas-plausíveis o numerador colapsa para um valor pequeno, o `out` volta errado, e o `k` do pool cai. O `as u128` no fim é o que mantém o crime quieto: ele roda *depois* do estrago, convertendo o valor que já deu wrap no tipo que o `checked_div(denominator)` sobrevivente espera, então a edição fica em uma linha e o build fica verde. Um humano lendo esta linha vê um multiply e um cast. O fuzzer vê uma reta numérica e vai andar direto para fora da borda dela.

Um detalhe de build decide se isto dá wrap ou dá panic, e é o mesmo da lição passada: o workspace gerado pelo Anchor põe `overflow-checks = true` no profile de release, então num scaffold intocado isto dá panic. Um panic ainda dispara o fuzzer, então o exercício funciona de qualquer jeito, mas o crash que você recebe é um abort em vez de uma violação de invariante. Para ver a versão de wrap silencioso, a que é genuinamente mais assustadora, ponha `overflow-checks = false` no `[profile.release]` do workspace antes de compilar, e ponha de volta depois. Anote isso como uma linha própria no arquivo da checklist: qual das duas você viu é um fato sobre o seu build, não sobre o bug.

Checkpoint: o `anchor build` tem sucesso e os seus testes de swap existentes continuam passando, porque todos eles trocam contra um pool de 1,000,000 / 1,000,000 onde nada chega perto do `u64::MAX`. É essa a parte inquietante e a razão de você ter semeado ele: o bug está dentro, a suíte está verde, e nada do que você já escreveu notou. Se em vez disso o build falhar, você também mudou os casts nas linhas em volta dele, e o bug semeado precisa ser exatamente uma linha.

![Um card de código anotado mostrando um multiply cru de reservas u64 dando wrap em release, colapsando o produto constante, ao lado da correção checada em u128.](assets/v08-annotated-code.webp)

### 5. Complete o invariante e rode ele (o recuo começa aqui)

Abra a bancada de teste que o scaffold escreveu. Ela tem ações já descobertas a partir do seu programa e um invariante com um buraco nele. A forma do Crucible é pequena: uma struct de fixture, um bloco `impl` onde qualquer método chamado `action_*` vira uma transição de estado que o fuzzer pode disparar, e uma função `#[invariant_test]` que roda depois de cada ação.

<!-- verify: expect-fail fuzz scaffold with a deliberate TODO - the reader adds prev_k and its initializer -->
```rust
// fuzz/token_ticket_swap/src/main.rs
use crucible_fuzzer::*;

#[derive(Clone)]
struct SwapFixture {
    ctx: TestContext,
    pool: Pubkey,
    reserve_arcade: Pubkey,
    reserve_ticket: Pubkey,
    prev_k: u128,          // YOU add this field; nothing else in the scaffold needs it
}

#[fuzz_fixture]
impl SwapFixture {
    pub fn setup() -> Self {
        // Deploys the swap, creates a pool with starting reserves, funds traders,
        // and returns the fixture. (scaffolded, except the last line.)
        let mut f = Self { /* scaffolded */ };
        f.prev_k = f.k();      // YOU add this: seed the baseline before any trade
        f
    }

    // Any `action_*` method is auto-discovered as an action the fuzzer can choose.
    pub fn action_swap(&mut self, #[range(0..4)] trader: usize, amount_in: u64) {
        // Fires one swap with a fuzzer-chosen trader and amount. (scaffolded.)
        // The invariant below runs AFTER this returns, so do not update prev_k here;
        // the invariant updates it once it has compared.
    }

    // Reads the two reserve token accounts back and returns their amounts.
    pub fn reserves(&self) -> (u64, u64) {
        let a = self.ctx.token_amount(&self.reserve_arcade);
        let b = self.ctx.token_amount(&self.reserve_ticket);
        (a, b)
    }

    pub fn k(&self) -> u128 {
        let (a, b) = self.reserves();
        (a as u128) * (b as u128)
    }
}

#[invariant_test]
fn constant_product_holds(fixture: &mut SwapFixture) {
    let k_now = fixture.k();

    // TODO (yours): assert the constant product never SHRINKS across a trade,
    // then update prev_k so the next action compares against this one.
    // Use the fuzz_assert_* macros, NOT assert!: a bare assert! panics the whole
    // fuzzer process, while fuzz_assert_* records the violation as a crash and
    // lets the run continue.
    //
    //   fuzz_assert_ge!(k_now, fixture.prev_k);
    //   fixture.prev_k = k_now;
    let _ = k_now;
}
```

O buraco é a asserção, e ela é o ponto inteiro da bancada, então pense no que "correto" quer dizer antes de escrever ela. Um swap nunca pode deixar o `k` encolher. O seu swap cobra 0.3%, então o `k` normalmente vai crescer; um sem taxa manteria ele exatamente igual. A asserção que é verdadeira nos dois casos é `k_now >= k_before`, que é `fuzz_assert_ge!`. Repare onde o `prev_k` é atualizado: no invariante, depois da comparação, não no `action_swap`. Atualize ele na ação e você compara um valor com ele mesmo e a asserção nunca consegue falhar, que é a maneira mais comum de uma bancada de fuzz voltar limpa sem fazer nada. Escreva essas duas linhas, e depois rode o build semeado:

```bash
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful
```

Você está esperando a rodada parar e reportar um crash. Com o multiply `u64` semeado no lugar, ela vai, e rápido, porque o fuzzer é guiado por cobertura na direção do branch onde as reservas ficam grandes o bastante para dar wrap. Ele te entrega um artefato de crash: uma sequência de entradas concreta e minimizada, na qual você consegue dar replay, que violou o seu invariante.

![Um fluxograma do loop de crash-depois-limpo, da semeadura e do invariante até o artefato de crash, o replay, o patch, e o re-fuzz limpo com exportação de LCOV.](assets/v09-flowchart.webp)

### 6. Dê replay no crash, corrija, e re-fuzze até limpar

O replay não é opcional. Um crash que você não consegue reproduzir é um boato. O Crucible escreveu a sequência que falha em `fuzz/token_ticket_swap/crashes/constant_product_holds/`, então liste o que ele salvou, e depois dê replay em um exatamente:

```bash
anchor fuzz show token_ticket_swap                          # list the saved crashes
anchor fuzz show token_ticket_swap <crash_file> --replay    # re-run that exact sequence
```

Veja ele re-rodar a mesma sequência de trader/quantia e disparar a mesma asserção. Essa é a sua prova de que o artefato é real e determinístico. Agora corrija: ponha de volta o numerador `u128` checado exatamente como na versão correta acima, e restaure o `overflow-checks` se você desligou ele. Rode de novo o mesmo comando do passo cinco:

```bash
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful
```

Desta vez ela deveria rodar sem produzir um crash. E aqui está a disciplina que a lição inteira foi construída para instalar: aquela rodada limpa não quer dizer "seguro." Ela quer dizer "os invariantes que eu escrevi, sobre as ações que eu defini, pelo tempo que eu deixei rodar, não acharam nada." Diga essa frase para você mesmo toda vez que uma rodada voltar verde. Depois rode de novo com cobertura ligada, para você ver quanto do programa o fuzzer de fato exercitou:

```bash
# Same run, with LCOV coverage written out.
anchor fuzz run token_ticket_swap constant_product_holds --release --stateful --coverage \
  --lcov-out ./fuzz-coverage.lcov
```

Aquele arquivo LCOV é uma leitura de quais linhas o fuzzer alcançou, e o `genhtml` vai transformar ele em algo navegável. Note qual comando produziu ele: a cobertura de fuzz vem do `run --coverage`, enquanto o comando separado `anchor coverage` reporta sobre os traces do `anchor test`, não sobre os do fuzzer. De qualquer jeito, a cobertura te diz onde a busca esteve e onde não esteve; cobertura baixa num branch crítico quer dizer que você não explorou ele, não que o branch é seguro. A cobertura é o mapa, o `--stateful` é o veículo.

## Challenge

Duas partes. A primeira termina o loop; a segunda é você sozinho.

**Completion.** `k_now >= k_before` é uma asserção fraca. O seu swap cobra 0.3%, então não é meramente verdade que o `k` não encolhe, é verdade que o `k` cresce pelo menos a contribuição da taxa em qualquer troca não-zero. Aperte o invariante para dizer isso: afirme `k_now > k_before` sempre que a ação de fato moveu tokens, e `k_now == k_before` quando não moveu. Você vai precisar que o `action_swap` registre se a troca teve sucesso, já que um revert de slippage é um no-op legítimo.

Depois rode `anchor fuzz run token_ticket_swap constant_product_holds --release --stateful` e leve isso ou até um crash no qual você dá replay e corrige, ou até uma rodada limpa com um relatório LCOV. Fique de olho no modo de falha interessante aqui: uma asserção mais apertada pode dar crash numa troca *legítima*, porque divisão inteira quer dizer que um `amount_in` pequeno o bastante arredonda a taxa para fora inteiramente e o `k` genuinamente não se move. Se isso acontecer, o fuzzer achou um bug no seu invariante, não no seu programa, e saber qual dos dois você está olhando é a habilidade.

**Solo.** Semeie exatamente um bug novo no swap. Escolha uma coisa que uma checklist não pegaria: um passo de arredondamento que trunca em favor do pool a cada troca, ou uma taxa aplicada ao `amount_out` em vez do `amount_in`. Antes de rodar qualquer coisa, escreva a sua previsão: o fuzzer vai achar isso, e se for achar, vai precisar do `--stateful`? Depois rode `anchor fuzz run token_ticket_swap constant_product_holds --release --stateful`, e se der crash, dê replay com `anchor fuzz show token_ticket_swap <crash_file> --replay` para confirmar a sequência exata. Corrija ele. Re-fuzze até limpar. Compare o que aconteceu com a sua previsão. O bug de arredondamento em particular é um bom professor: uma troca isolada perde uma fração de um lamport, invisível para um teste de uma troca só, mas encadeado ao longo de dezenas de trocas stateful o resto se acumula até o seu invariante disparar.

Aceite quando: um artefato de crash for produzido e receber replay para o seu bug semeado, o bug estiver corrigido, o alvo re-fuzzar limpo com um relatório LCOV, e a sua checklist de auditoria estiver totalmente verde com um número de linha em cada item.

## Deu certo, e o que isso não comprova

Você terminou esta lição quando conseguir mostrar quatro coisas: uma checklist verde, um artefato de crash com replay, um re-fuzz limpo, e um relatório LCOV. Se a sua rodada nunca deu crash no bug semeado, a causa usual é um `--stateful` faltando ou um invariante que não afirma nada de verdade (um `assert!(true)` disfarçado). Se ela dá crash e você não consegue dar replay, você corrigiu antes de salvar o artefato. Conserte a ordem: crash, replay, patch, re-fuzz.

![Uma checklist de quatro itens pareando cada artefato exigido com o erro que explica a ausência dele, sob uma faixa fixando a ordem como crash, replay, patch, e depois re-fuzz.](assets/v10-table.webp)

Agora a parte que te mantém honesto, porque é fácil sair de uma rodada verde se sentindo pronto. Fuzzing e uma checklist elevam a sua confiança. Eles nunca comprovam a ausência de bugs. Uma rodada limpa quer dizer "ainda não achado", que é uma afirmação real e útil, e estritamente mais fraca que "seguro". Vale ser preciso sobre a lacuna. Uma rodada limpa de fuzz é uma afirmação de caso médio: sobre as entradas e sequências que o fuzzer por acaso explorou no tempo que você deu a ele, nenhum invariante quebrou. O bug que te arruína normalmente é um objeto de pior caso, uma entrada única e estreita num canto que a busca não alcançou antes de você dar o dia por encerrado. Fuzzing guiado por cobertura estreita essa lacuna direcionando para branches inexplorados, mas não fecha ela, e não existe duração de rodada que transforme "limpo em caso médio" em "seguro em pior caso".

O argumento mais forte para essa humildade vem do framework em cima do qual você está de pé. A suíte de testes do próprio Anchor carrega witnesses de Miri, que checam por comportamento indefinido em código unsafe, e configs de Kani, que fazem verificação de modelos sobre propriedades específicas. E fuzzing achou quatro bugs de corretude no próprio Anchor, rastreados como a issue #4431. O framework é fuzzado e checado contra comportamento indefinido tão duro quanto ele pede que você cheque o seu programa, e ele *ainda assim* achou quatro coisas. Se isso é verdade para código escrito e revisado pelas pessoas que construíram o framework, assuma que é verdade para o seu.

![Um diagrama vertical em camadas da superfície de confiança, descendo do seu programa pelas checagens de Miri e Kani do Anchor e pela guarda da OtterSec até uma ressalva de revisão que este curso fornece por autoridade própria, porque a tag fixada não entrega nenhuma.](assets/v11-timeline.webp)

Esse guardião único é em si um fato que vale sentar com ele. A OtterSec faz a custódia do framework, publica os crates, roda o registry de builds verificados contra o qual o `anchor verify` checa, e assina a tag v2 com uma chave GPG (trixter-osec). Uma organização só detém boa parte da cadeia de suprimentos, o que é eficiente e também uma concentração que você deveria conhecer. Isso vem junto com uma ressalva que este curso tem que fornecer por autoridade própria, porque o projeto não fornece: vá olhar na tag fixada e você não vai achar página de ressalva nenhuma — o README do lang-v2 diz "v2 is secure by default for users" e para por aí. Então tire a frase da auditoria que você acabou de rodar em vez de tirar de uma citação: os defaults não são substituto para revisão, fuzzing e modelagem de ameaças específica de produção. Um guardião só, um release candidate não auditado, e quatro bugs achados por fuzzer no próprio framework são o argumento inteiro, e eles bastam.

Mais duas notas honestas para fechar a superfície de confiança. Primeira, os guardrails que você tentou desligar lá na lição de CU — o flip que a unificação de features do cargo engoliu — são uma rede de proteção de runtime default-on: eles pegam coisas como um despacho com program id errado ou acesso mutável a uma conta somente leitura em tempo de execução. Desligar eles economiza um pouco de tamanho de binário e de CU, e tira uma rede precisamente quando um fuzzer tem mais chance de estar empurrando o seu programa para um estado ruim. Fuzze com os guardrails ligados. Entregue com eles desligados só depois que fuzzing e revisão tiverem mostrado que nada que eles teriam pego continua vivo. Essa é uma decisão de segurança contra velocidade, e agora você consegue tomar ela deliberadamente em vez de por padrão.

Segunda, sobre ferramental: você vai ouvir falar do Trident, o fuzzer da Ackee, e ele é um projeto de verdade. Mas ele não é o caminho embutido, e a cadência de release dele estagnou: o último stable é o 0.12.0 de 2025-11-27, com um pré-release 0.13.0-rc.4 parado desde 2026-05-14. O Anchor escolheu o Crucible e ligou ele na CLI. Busque o `anchor fuzz` primeiro; o Trident é um fallback para avaliar, não o default.

O programa agora está tão duro quanto você consegue deixar ele na mão e na máquina, com a checklist e o fuzzer os dois verdes e os dois honestamente rotulados como "nenhum bug achado ainda". É esse o lugar certo de parar, porque a próxima ameaça não está no código. No próximo módulo, o swap sai da sua máquina: você gera um cliente tipado para que outras pessoas consigam chamar ele, comprova um build verificável para que elas possam confiar que o bytecode casa com o código-fonte, e raciocina sobre quem detém a chave de upgrade, que é a única superfície de ataque que nenhuma quantidade de fuzzing verde consegue fechar.
