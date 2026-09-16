# Construa o swap de token-para-ticket

Na lição passada você moveu o quarter-vault e o prize-escrow para o `token_interface`. Os dois agora fazem custódia de tokens SPL de verdade em vez de lamports, e os testes de withdraw e de release deles estão verdes. É essa a razão inteira de esta lição ser possível: você tem dois programas que conseguem segurar um saldo de token e assinar transferências para fora dele com um PDA. Ligue dois daqueles juntos e você tem um mercado.

Então aqui está a dor. Os players ganham tokens de fliperama do cabinet. Eles querem tickets para gastar no balcão de prêmios. Alguém tem que definir a taxa de câmbio. Deixe `1 token = 1 ticket` hardcoded numa constante e você construiu uma torneira, não um swap: o primeiro player que notar que o pool segura mais tickets que tokens troca o lado barato até as reservas ficarem vazias, e você pagou por isso. A taxa não pode ser uma constante. Ela tem que vir das reservas próprias do pool e se mover cada vez que alguém troca.

Antes de você ler outro parágrafo, faça a aritmética que um swap faz em cada troca. Rode ela, não aceite a minha palavra. Salve isto como `curve.py` e rode `python3 curve.py`, sem pacotes, sem instalação, só o `python3` que já está na sua máquina. Quatro linhas são a troca inteira:

```python
def swap_out(reserve_in, reserve_out, amount_in):
    fee_in = amount_in * 997                                  # 0.3% fee, taken before the curve
    return (fee_in * reserve_out) // (reserve_in * 1000 + fee_in)

print("naive:", 10_000 * 1_000_000 // 1_000_000)             # 10000
print("real :", swap_out(1_000_000, 1_000_000, 10_000))      # 9871
```

Aí está a sua lacuna de 129 tickets, impressa: uma taxa ingênua cota 10,000, a curva paga 9,871. Segure essa lacuna. No fim desta lição você vai saber exatamente para onde cada um daqueles tickets foi, e por que devolver eles para o trader é como pools são drenados.

## Resumo

O que você constrói, e a forma de cada peça, para você conseguir passar o olho antes de cavar:

- **O artefato é o R4, um swap de produto constante.** Duas reservas de token, uma cotação computada a partir daquelas reservas, e duas transferências assinadas por PDA por troca. É o padrão canônico de composição do Anchor: um programa dirigindo duas contas de token que ele controla.
- **O preço é o invariante.** `x * y = k`. O produto das duas reservas continua (quase) constante através de uma troca, que quer dizer que quanto mais você compra, pior a sua taxa fica. Sem oráculo, sem admin, sem número hardcoded. As reservas *são* o preço.
- **A taxa é de 0.3%, expressa como `997/1000`.** Ela é subtraída da entrada antes de a curva ver, e ela fica no pool, que empurra o `k` para cima em cada troca.
- **A matemática precisa de um intermediário `u128`.** Duas reservas `u64` multiplicadas conseguem passar longe do `u64`. Promova, multiplique, divida, converta de volta para baixo. Este é o único lugar em que um swap dá panic se você errar.
- **A trava de slippage é o `min_out`.** O trader declara o pior fill que ele vai aceitar; o programa reverte se a cotação vier abaixo dele. Ela reusa a forma exata da release condicional do escrow do R3. Ela é a única proteção de trader que este swap oferece.
- **As duas reservas são contas distintas.** Então o default de duplicate-mutable do Anchor V2 é satisfeito de graça. Você não vai recorrer ao `unsafe(dup)`, e esta lição te mostra por que recorrer a ele aqui seria um bug de design.

O recuo desta lição: eu derivo a curva e te entrego a transferência de token-para-dentro trabalhada por inteiro. A transferência de token-para-fora é um preenchimento que espelha ela. A trava de slippage é sua para escrever solo, porque nessa altura você vai ter visto a gêmea dela no escrow e você não deveria precisar de mim para isso.

Uma linha de escopo antes de a gente começar. Este é um swap didático, um padrão de Anchor, não um protocolo de DeFi. Se você quer criação de mercado automatizada em venue de verdade com profundidade viva de provedor de liquidez, esse é o território do curso de DeFi e RWA Engineering. Aqui o swap existe para ensinar composição de PDA de dois lados, não para negociar contra.

## O preço mora nas reservas

Uma ideia gera tudo o que segue. Um pool de produto constante segura dois ativos e trata o produto dos saldos deles como um número que ele tem que proteger. Chame as reservas de `x` e `y`. A lei do pool é `x * y = k`, e o `k` é (quase) sagrado: qualquer troca tem que deixar o `k` pelo menos tão grande quanto ela achou ele.

Essa regra única é o mecanismo de preço. Quando um trader adiciona `dx` do primeiro ativo, o pool tem que devolver o bastante do segundo ativo, `dy`, para o produto ainda valer. Resolva para `dy` e a taxa não é um número guardado em lugar nenhum: ela é o que quer que mantenha a curva intacta. Quanto mais profundo o pool, menos uma troca dada move ele; quanto mais raso o pool, mais cada troca custa. É a geometria fazendo o policiamento para você.

![Uma troca puxa os tokens de fliperama do trader para dentro da reserva do pool, cota uma saída, checa ela contra o min_out, e depois empurra tickets de volta sob a assinatura do PDA do pool.](assets/v01-diagram.webp)

Por que cotar a partir das reservas em vez de uma taxa que você define? Porque uma taxa que você define é uma taxa da qual um atacante consegue escolher um lado. Imagine o pool como uma gangorra curvada com uma área fixa embaixo dela. Cada troca desliza um peso ao longo da barra, e a barra tem que inclinar para manter a área constante. Um trader consegue empurrar o peso, mas ele não consegue mudar a área, e a área é de onde ele precisaria roubar. A curva não está protegendo um preço que você escolheu. Ela é o preço, e ela se reprecificou no instante em que a última troca aterrissou. É essa a razão inteira de um AMM não precisar de oráculo: o pool cota a si mesmo.

Agora a fórmula exata. Sem taxa, manter o `k` constante dá:

```
out = (amount_in * reserve_out) / (reserve_in + amount_in)
```

Leia e note o que ela faz por conta própria. Conforme o `amount_in` cresce em relação ao `reserve_in`, o denominador cresce também, então cada token adicional de entrada compra menos tokens de saída. Tente comprar o `reserve_out` inteiro e o denominador foge de você; você nunca consegue drenar ele com uma entrada finita. A curva se recusa.

A taxa é um corte na entrada antes de a curva ver. Uma taxa de 0.3% quer dizer que o pool fica com 3 de cada 1000 unidades de entrada e só 997 delas contam para a troca:

```
amount_in_with_fee = amount_in * 997
out = (amount_in_with_fee * reserve_out) / (reserve_in * 1000 + amount_in_with_fee)
```

O `1000` no denominador está ali para manter tudo na mesma escala inteira que o `997`, para você nunca tocar num float. Matemática on-chain é matemática de inteiros, sempre. Os tokens de taxa ficam no `reserve_in`, então o `k` depois da troca é um pouco maior que o `k` antes. O pool cresce. Esse crescimento é o que um provedor de liquidez de verdade ganharia; aqui ele só quer dizer que o teste de invariante assere `k_after >= k_before`, nunca igualdade.

A curva de produto constante como uma figura, porque a forma é a intuição:

![Uma cotação linear ingênua e a curva de produto constante concordam em trocas minúsculas mas divergem forte conforme o tamanho cresce, com a curva dobrando muito abaixo da linha.](assets/v02-chart.webp)

Essa lacuna entre a linha reta e a curva é os 129 tickets da introdução, escalados. Numa troca de 10,000 tokens ela é pequena. Numa troca de 500,000 tokens contra uma reserva de 1,000,000 a taxa ingênua entregaria 500,000 tickets enquanto a curva dá 332,665. Um pool que cotasse a linha reta seria drenado pela primeira baleia que fizesse a aritmética. A curva está se recusando a te vender o pool inteiro no preço marginal.

### O único lugar em que isto dá panic: `u64` vezes `u64`

Olhe o numerador: `amount_in_with_fee * reserve_out`. Os dois derivam de valores `u64`. Multiplique dois números perto do topo do `u64` e o produto precisa de até 128 bits. Faça isso em `u64` e o programa dá panic numa troca grande contra um pool profundo, que é exatamente a troca na qual você menos quer falhar. A correção não é limitar as suas reservas, e sim promover para `u128`, multiplicar lá, dividir de volta para baixo, e converter o resultado para `u64` só depois de você saber que ele cabe (e ele sempre cabe, porque a saída nunca consegue exceder o `reserve_out`, que já é um `u64`).

A função de cotação, percorrida linha por linha. Este é o artefato de interface que um lab posterior vai chamar, então a assinatura está congelada: `swap_out(reserve_in, reserve_out, amount_in) -> u64`.

```rust
/// Quote a constant-product swap output with a 0.3% fee.
///
/// Uses a u128 intermediate so the product of two u64 reserves cannot
/// overflow. Integer division truncates, and truncation favors the pool
/// (the trader is never rounded up). Returns 0 for a zero input or an
/// empty reserve, so the caller can treat 0 as "no trade".
pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    // Nothing to trade, or a side of the pool is empty: no quote.
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }

    // 0.3% fee: only 997 of every 1000 input units reach the curve.
    // amount_in <= u64::MAX, so * 997 stays well inside u128.
    let amount_in_with_fee = (amount_in as u128) * 997;

    // The one multiply that can exceed u64. Guard it; on overflow, no quote.
    let numerator = match amount_in_with_fee.checked_mul(reserve_out as u128) {
        Some(n) => n,
        None => return 0,
    };

    // reserve_in > 0 here, so the denominator is never zero.
    let denominator = (reserve_in as u128) * 1000 + amount_in_with_fee;

    // Output is bounded by reserve_out (a u64), so this cast never truncates.
    (numerator / denominator) as u64
}
```

![As cinco linhas estruturais do swap_out, cada uma emparelhada com a falha específica que ela impede: cotação de pool vazio, taxa-antes-da-curva, overflow de u128 na multiplicação, divisão por zero, e truncamento seguro a favor do pool.](assets/v03-annotated-code.webp)

Note a direção do arredondamento. Divisão de inteiros joga o resto fora, então o trader sempre recebe o piso, nunca o teto. Isso é deliberado. Arredondamento tem que favorecer o pool em cada troca, porque um swap roda milhões de vezes e uma fração arredondada para o lado errado, repetida, é um vazamento lento. O pool ficar com a poeira é correto. O trader ficar com ela é um bug que você acharia meses depois como um déficit que você não consegue explicar.

É aqui que a economia ganha uma frase, e só uma, porque esta é uma lição de Anchor e não de mercados. A razão de um pool conseguir cotar a si mesmo sem oráculo e sem operador é que a curva transforma liquidez numa função de preço: profundidade vira estabilidade, e cada troca paga o pool pelo privilégio de mover ele. Isso é uma peça genuinamente elegante de design de mecanismo, e é por isso que o mesmo invariante de duas linhas aparece embaixo do Uniswap, embaixo de uma bonding curve da pump.fun, e embaixo do brinquedo que você está construindo agora. Você não está inventando ele, só ligando ele no Anchor.

### Para onde os 129 tickets foram

Eu prometi que você ia saber para onde cada um daqueles 129 tickets foi, então aqui está a contabilidade inteira daquela troca de 10,000 tokens contra o pool balanceado de 1,000,000 / 1,000,000. Ela se divide limpo em duas peças, e nenhuma das duas é um vazamento.

Rode a curva sem taxa nenhuma e você recebe 9,900 tickets, não os 10,000 ingênuos. Aqueles 100 faltando são impacto de preço. No instante em que os seus 10,000 tokens aterrissam na reserva o pool não está mais balanceado um para um, então a curva reprecifica os tickets que você está comprando enquanto você está comprando eles. Você moveu o mercado e você pagou por mover ele. Ninguém colocou aqueles 100 tickets no bolso; a cotação linear ingênua só fingia que eles estavam na mesa.

Agora coloque a taxa de 0.3% de volta e a saída cai de 9,900 para 9,871. Aqueles últimos 29 tickets são a taxa, e ela fica atrás como reserva extra. Cem para impacto de preço, vinte e nove para a taxa, cento e vinte e nove no total. Os dois são a curva fazendo precisamente o que ela foi projetada para fazer, e os dois são números que um trader ia querer na frente dele antes de assinar, que é a razão inteira de o `min_out` existir.

Aqueles mesmos 29 tickets de taxa são o que levanta o invariante. Antes da troca, `k = 1,000,000 * 1,000,000 = 1,000,000,000,000`. Depois dela, `reserve_in = 1,010,000` e `reserve_out = 990,129`, então `k = 1,000,030,290,000`, um fio acima de onde ele começou. O `k` nunca cai. A taxa é a coisa que empurra ele para cima, e o teste de invariante da trava assere exatamente isso: `k_after >= k_before`, nunca igualdade.

![Uma cascata da cotação ingênua de 10,000 tickets para baixo 100 tickets por impacto de preço e 29 pela taxa, aterrissando no fill de verdade de 9,871 tickets.](assets/v04-chart.webp)

### Movendo os tokens: duas transferências, dois signers

A cotação é uma função pura. Ela não toca conta nenhuma. A troca de verdade são duas transferências SPL em direções opostas, e a parte interessante é quem assina cada uma.

A transferência de entrada é fácil: o trader está gastando os tokens próprios dele, então o trader assina. Tokens de fliperama se movem do `trader_arcade` para dentro do `reserve_arcade`.

A transferência de saída é o padrão que deixa isto uma lição de Anchor. Os tickets moram no `reserve_ticket`, e a authority daquela conta é o PDA do pool, um endereço sem chave privada. Ninguém consegue assinar por ele exceto o programa que é dono das seeds dele. Então o programa assina, usando o `with_signer` e as seeds do pool, exatamente do jeito que o seu quarter-vault assinava os withdrawals próprios dele na lição anterior. Tickets se movem do `reserve_ticket` para o `trader_ticket` sob a assinatura do pool.

O handler roda uma sequência fixa, e a ordem não é cosmética: as leituras têm que acontecer antes de os handles existirem, e a trava pertence na frente das duas transferências. Erre a primeira e o compilador te para (uma leitura depois de um handle). Erre a segunda e — seja preciso aqui — a atomicidade ainda salva o trader: um `require!` que falha depois das transferências reverte as duas, então um fill ruim nunca chega a assentar. O que uma trava tardia te custa é diferente. Você gasta duas CPIs inteiras para descobrir o que uma comparação podia ter dito de cara, e você escreve um handler em que a única proteção do trader lê como uma coisa pensada depois sobre a qual um revisor tem que raciocinar de trás para frente. Travas vão antes do dinheiro por custo e por legibilidade, não porque o runtime deixaria um fill revertido ficar de pé.

![O handler lê as duas reservas primeiro, cota a saída, reverte se ela estiver abaixo do min_out, e depois roda a CPI de puxar assinada pelo trader e a CPI de empurrar assinada pelo PDA do pool, nessa ordem fixa.](assets/v05-flowchart.webp)

Aqui está o handler de swap inteiro. A direção de token-para-dentro está trabalhada; a direção de token-para-fora é o preenchimento e está mostrada aqui para você ver o espelho, mas no lab você vai digitar ela você mesmo contra um stub.

```rust
use anchor_lang::prelude::*;
use anchor_spl::{
    // `token` rides along as a MODULE, not for a type: the `token::mint = ...`
    // constraints below expand to code that names the module by path, so it has
    // to be in scope or the derive fails to resolve — m05-l1's import rule.
    token,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

// Placeholder. Keep the id `anchor new token-ticket-swap` generated for you.
declare_id!("<your generated program id>");

pub const POOL_SEED: &[u8] = b"pool";

#[program]
pub mod token_ticket_swap {
    use super::*;

    pub fn swap_arcade_for_tickets(
        ctx: &mut Context<SwapArcadeForTickets>,
        amount_in: u64,
        min_out: u64,
    ) -> Result<()> {
        // 1. Read the reserves BEFORE opening any CPI handle. In V2 you cannot
        //    hold a typed reference to a token account while a CpiHandle from it
        //    is live, so the read happens here, up front, once.
        let reserve_in = ctx.accounts.reserve_arcade.amount();
        let reserve_out = ctx.accounts.reserve_ticket.amount();

        // 2. Quote the output from the pre-trade reserves.
        let out = swap_out(reserve_in, reserve_out, amount_in);
        require!(out > 0, SwapError::ZeroOutput);

        // 3. Slippage guard (this is the SOLO piece in the lab).
        require!(out >= min_out, SwapError::SlippageExceeded);

        // 4. Pull the arcade tokens IN. The trader signs for their own account.
        let pull = TransferChecked {
            from: ctx.accounts.trader_arcade.cpi_handle_mut(),
            mint: ctx.accounts.mint_arcade.cpi_handle(),
            to: ctx.accounts.reserve_arcade.cpi_handle_mut(),
            authority: ctx.accounts.trader.cpi_handle(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.address(), pull),
            amount_in,
            ctx.accounts.mint_arcade.decimals(),
        )?;

        // 5. Push the tickets OUT. The pool PDA signs with its own seeds.
        //    Copy the bump into an owned local FIRST, exactly as the vault did:
        //    `signer_seeds` borrows this array, so it has to outlive the CPI
        //    below. That is a lifetime requirement, not a borrow conflict --
        //    `pool` goes into `push` as a shared `cpi_handle()`, which leaves
        //    reads of `pool` legal. The exclusion applies to the accounts handed
        //    over with `cpi_handle_mut()`.
        let bump = [ctx.accounts.pool.bump];
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, &bump]];
        let push = TransferChecked {
            from: ctx.accounts.reserve_ticket.cpi_handle_mut(),
            mint: ctx.accounts.mint_ticket.cpi_handle(),
            to: ctx.accounts.trader_ticket.cpi_handle_mut(),
            authority: ctx.accounts.pool.cpi_handle(),
        };
        token_interface::transfer_checked(
            CpiContext::new(ctx.accounts.token_program.address(), push)
                .with_signer(signer_seeds),
            out,
            ctx.accounts.mint_ticket.decimals(),
        )?;

        Ok(())
    }
}

pub fn swap_out(reserve_in: u64, reserve_out: u64, amount_in: u64) -> u64 {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }
    let amount_in_with_fee = (amount_in as u128) * 997;
    let numerator = match amount_in_with_fee.checked_mul(reserve_out as u128) {
        Some(n) => n,
        None => return 0,
    };
    let denominator = (reserve_in as u128) * 1000 + amount_in_with_fee;
    (numerator / denominator) as u64
}

#[derive(Accounts)]
pub struct SwapArcadeForTickets {
    pub trader: Signer,

    #[account(seeds = [POOL_SEED], bump = pool.bump)]
    pub pool: Account<Pool>,

    // Pinned to what the pool recorded at init. Without these two lines the mint
    // fields on Pool would be decoration: a caller could hand you any mint whose
    // reserves happen to be pool-owned, and the token:: constraints below would
    // happily agree with them. `address = parent.field` is the has_one replacement,
    // and this is the second place in two lessons it does real work.
    #[account(address = pool.arcade_mint @ SwapError::WrongMint)]
    pub mint_arcade: InterfaceAccount<Mint>,
    #[account(address = pool.ticket_mint @ SwapError::WrongMint)]
    pub mint_ticket: InterfaceAccount<Mint>,

    #[account(mut, token::mint = mint_arcade, token::authority = pool)]
    pub reserve_arcade: InterfaceAccount<TokenAccount>,
    #[account(mut, token::mint = mint_ticket, token::authority = pool)]
    pub reserve_ticket: InterfaceAccount<TokenAccount>,

    #[account(mut, token::mint = mint_arcade, token::authority = trader)]
    pub trader_arcade: InterfaceAccount<TokenAccount>,
    #[account(mut, token::mint = mint_ticket, token::authority = trader)]
    pub trader_ticket: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<'static, TokenInterface>,
}

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,  // 32
    pub ticket_mint: Address,  // 32
    pub bump: u8,              //  1
    pub _pad: [u8; 7],         //  7  explicit Pod padding (65 -> 72)
}

#[error_code]
pub enum SwapError {
    #[msg("Swap output is zero")]
    ZeroOutput,
    #[msg("Slippage tolerance exceeded: output below min_out")]
    SlippageExceeded,
    #[msg("Mint does not match the one this pool was initialized with")]
    WrongMint,
}
```

Uma escolha de constraint merece uma nota antes das especificidades do V2. As quatro contas de token carregam a família `token::` da lição passada, não `associated_token::`. A razão é a mesma que colocou o `token::` no recipient do escrow: um constraint de ATA *deriva* um endereço a partir de um owner e de um mint, e reservas não são ATAs. O pool segura duas contas de token que ele criou e atribuiu a si mesmo, em endereços que ninguém deriva, então não tem nada contra o que derivar. O `token::mint` e o `token::authority` restringem o que a conta segura e quem controla ela, que é exatamente a afirmação de que você precisa aqui.

Algumas coisas naquele código são especificidades do Anchor V2 que valem uma pausa, porque elas são novas desde a linha 0.x que você talvez tenha escrito antes, e este é um curso de framework.

A forma da CPI mudou. O `CpiContext::new` agora pega o programa como um `&Address`, que você consegue do `ctx.accounts.token_program.address()`. Na linha 0.x você passava um `AccountInfo` (um `.to_account_info()`); no V2 isso é um erro de compilação, `expected Address, found AccountInfo`. As contas que você passa para dentro da struct `TransferChecked` não são `AccountInfo` tampouco, e sim valores `CpiHandle`, produzidos pelo `cpi_handle()` para uma conta somente-leitura e pelo `cpi_handle_mut()` para uma mutável. O handle é o ticket da conta para dentro da CPI, e ele carrega um borrow do Rust do wrapper tipado do qual ele veio.

Esse borrow é o ponto da próxima seção, e ele é a cilada que costumava morder todo mundo.

![Lado a lado: na linha 0.x uma conta desserializada ficava obsoleta a não ser que você chamasse o reload, enquanto o handle de CPI com borrow checado do V2 transforma aquela leitura obsoleta num erro de compilação.](assets/v06-comparison.webp)

### Por que o V2 não vai te deixar ler um saldo no meio de uma CPI

Na linha 0.x, este era o bug clássico. Você fazia uma CPI que movia tokens, depois lia o `token_account.amount` e agia em cima dele, esquecendo que o Anchor desserializou aquela conta *uma vez*, no topo da instrução. A CPI mudou o saldo on-chain, mas a sua cópia em memória ainda segurava o número antigo. Você tinha que chamar o `.reload()` para atualizar ela, e se você esquecesse, você tomava uma decisão em cima de dado obsoleto. Era silencioso, era fácil, e era entregue.

O V2 mata a classe. Um `CpiHandle` segura um borrow do Rust do wrapper tipado de conta. Enquanto aquele handle está vivo, o borrow checker não vai te deixar tocar a conta tipada, então o `reserve_ticket.amount()` durante um handle em voo a partir do `reserve_ticket` não é uma surpresa de runtime, e sim um erro de compilação. A correção é estrutural, não um `.reload()`: leia cada reserva de que você precisa *antes* de você abrir um handle, que é exatamente por que o passo 1 no handler lê os dois saldos lá em cima, antes de qualquer `cpi_handle_mut()` existir. Não tem leitura no meio da CPI para errar, porque a linguagem removeu a habilidade de escrever uma.

Então não recorra ao `.reload()` aqui por memória muscular de 0.x. Se você se pegar querendo ele, você estruturou as leituras na ordem errada. Mova elas para mais cedo.

### A armadilha de duplicate-mutable na qual você não vai cair

Mais um default do V2 que vale nomear, porque um swap é exatamente a forma que tropeça nele. O V2 rejeita uma instrução que recebe a mesma conta mutável em dois campos. Passe a mesma conta de token como uma origem `mut` e um destino `mut` e a validação falha antes de o seu handler rodar. Aquela checagem existe para pegar bugs de aliasing em que você acidentalmente lê e escreve a mesma conta através de dois nomes e corrompe ela.

Um swap tem duas reservas, e a tentação, se você está pensando no pool como uma coisa só, é rotear as duas direções através de uma conta. Faça isso e o V2 te para. A reação errada é silenciar a checagem com o `unsafe(dup)`, a saída de emergência para contas genuinamente duplicadas. A reação certa é notar que um pool de produto constante tem duas reservas por definição, então cada lado é a conta de token distinta dele. O `reserve_arcade` e o `reserve_ticket` são contas diferentes segurando mints diferentes. O default de duplicate-mutable é satisfeito de graça, e você nunca toca no `unsafe(dup)`. Recorrer a ele aqui não seria um opt-out. Seria remendar um design em que você colapsou duas reservas em uma, que é um bug que a checagem acabou de pegar para você.

![A correção errada colapsa as duas reservas numa conta silenciada pelo unsafe(dup); o design certo mantém duas contas de reserva distintas, então nem duplicação nem opt-out existem.](assets/v07-comparison.webp)

O time do Anchor não adicionou esses defaults por pontos de estilo. Quando eles fizeram benchmark do V2 contra o Quasar e o Pinocchio antes da conferência Accelerate no começo de maio de 2026 (o enquadramento está ali na issue #4355, onde o esforço inteiro do V2 foi justificado como existencial), os programas que postaram as maiores reduções de compute foram os programas da família de AMM, o benchmark `prop-amm` especificamente, com uma maior redução reportada de 50.4x. É por isso que um swap é a peça de exibição: o padrão que você está construindo é o que o V2 foi afinado para deixar barato. Eu não vou imprimir um número de compute para este programa exato, porque aquelas cifras se moveram conforme o projeto afinava elas e a coisa honesta é medir o seu próprio, mas a direção era o pitch inteiro.

![Uma linha do tempo da justificativa de benchmark da issue #4355 até as rodadas pré-Accelerate do começo de maio de 2026, onde o programa prop-amm postou a maior redução reportada.](assets/v08-timeline.webp)

### A trava de slippage, e por que ela é o ponto inteiro

Existe uma lacuna entre o momento em que um trader lê uma cotação e o momento em que a transação dele aterrissa. Naquela lacuna, outras trocas conseguem bater no pool e mover as reservas. O trader que foi cotado em 9,871 tickets um momento atrás, contra um pool de 1,000,000 / 1,000,000, pode aterrissar num pool que uma venda grande já deslocou, e receber 9,400. Se o seu programa preenche qualquer troca que a curva produz, o trader come aquela diferença, e um ator hostil consegue *fabricar* aquela diferença ordenando uma troca na frente da dele e uma atrás. Isso é um sanduíche, e a única defesa que um swap tão simples assim tem contra ele é o `min_out`.

O `min_out` é o trader dizendo "reverta a não ser que eu receba pelo menos isto". O programa computa o `out` a partir das reservas vivas, compara ele com o `min_out`, e reverte se ele estiver curto. É essa a linha `require!(out >= min_out, SwapError::SlippageExceeded)`, e a forma dela é a mesma release condicional que você escreveu dentro do prize-escrow no R3: compute um valor, compare ele com um limite fornecido por quem chama, reverta se o limite não for atendido. Você já escreveu este fluxo de controle. É por isso que ele é o seu solo.

Deixe ela de fora e cada troca é preenchível em qualquer preço, que é dizer que cada troca é sanduichável. É uma linha, e ela é a diferença entre um swap que um player consegue usar e um swap que um bot cultiva.

## Lab: construa o R4, o swap de token-para-ticket

O R4 é um programa novo. Diga sem enfeite o que ele reusa e não reusa, porque a tentação é recorrer ao vault: as reservas do pool **não** são instâncias do vault do R2. A conta de token de um vault é uma ATA cuja authority é o PDA do vault, e a authority de uma reserva tem que ser o PDA do pool, então um vault não consegue ser uma reserva sem parar de ser um vault. O R4 compõe em nada. Ele é o primeiro degrau desde o módulo 1 que fica sozinho, e essa é a forma honesta dele: o que passa adiante é o padrão, o `transfer_checked` assinado por PDA que você aprendeu no vault, não o artefato. O vault e o escrow continuam fazendo os trabalhos deles em outro lugar do salão, e o capstone no módulo 9 é onde os quatro degraus finalmente se encontram.

O recuo da ajuda é explícito: o passo 1 é uma especificação que você implementa, o passo 2 você digita a partir da fórmula, os passos 3 e 4 estão trabalhados, a segunda CPI do passo 5 é um completion que você digita contra um stub, e os passos 6 e 7 são solo.

Primeiro, o toolchain, uma linha:

```bash
anchor --version       # confirm you are on the V2 line (2.0.0-rc.1 as of 2026-08-22), not 1.1.2
```

Se ele reportar 1.x, re-fixe com o bloco de instalação do m01-l2 — `--tag v2.0.0-rc.1`, `--locked`, o canal git, já que o `avm` ainda não consegue buscar o RC. Não verifique lições de V2 na linha 1.1.2; as APIs de CPI e de conta diferem e o seu código não vai compilar contra a antiga.

1. **Levante o estado do pool (especificação, nenhum código dado).** Escreva uma instrução `init_pool`. Ela cria a conta `Pool` em `seeds = [POOL_SEED]` com um `bump` simples, guarda os dois endereços de mint e o `ctx.bumps.pool`, e faz `init` de duas contas de token cuja `token::authority` é o PDA do pool, uma por mint, pagas por quem chama. Você escreveu cada uma daquelas linhas antes: `init` mais `seeds` mais `bump` é o módulo 3, guardar o bump canônico é o módulo 3, e criar uma conta de token de propriedade do programa é o `Initialize` da lição passada com uma authority diferente. Checkpoint: o `anchor build` compila. O R4 compõe em nada, então não tem suíte herdada para rodar ainda e nada contra o que asserir até o passo 7 escrever uma — momento em que a primeira coisa que aquele teste comprova é exatamente isto: a conta de pool existe e as duas contas de token de reserva reportam o PDA do pool como a authority delas. Anote o passo agora para o passo 7 ter uma asserção esperando por ele.

2. **Adicione a função de cotação.** Digite o `swap_out` você mesmo a partir das duas linhas de fórmula acima, com a assinatura congelada; role de volta para a versão trabalhada só depois de a sua compilar. Escreva um teste de unidade que chama ela com `reserve_in = 1_000_000`, `reserve_out = 1_000_000`, `amount_in = 10_000` e assere que ela retorna `9_871`. Se você receber `10_000`, você esqueceu a taxa. Se ela der panic num caso de reserva grande, você multiplicou em `u64`. Checkpoint: o teste de unidade está verde e os 9,871 computados na mão casam.

3. **Leia as reservas de cara.** No handler de swap, leia o `reserve_arcade.amount()` e o `reserve_ticket.amount()` para dentro de locais antes de você montar qualquer conta de CPI. Esse é o único lugar em que o compilador vai te deixar ler elas, porque uma vez que um `cpi_handle_mut()` a partir de uma reserva existe, o acesso tipado `.amount()` naquela reserva não vai compilar. Checkpoint: comprove essa afirmação em vez de confiar nela. Mova as duas leituras para ficarem *entre* a ligação `let pull = TransferChecked { .. };` e a chamada de `transfer_checked` que consome ela, que é a única janela em que o handle está genuinamente vivo, e rode o `anchor build`. Você deve receber um erro de borrow nomeando o `reserve_arcade`, não uma surpresa de runtime. Coloque elas em qualquer lugar depois da chamada de `transfer_checked` em vez disso e o build fica verde, porque o handle já caiu, que vale ver também: a regra é sobre o tempo de vida do handle, não sobre o número da linha. Mova elas de volta para o topo e continue.

4. **Ligue a CPI de token-para-dentro (trabalhada).** Monte o `TransferChecked` para `trader_arcade -> reserve_arcade` com o trader como authority, e invoque ele com o `token_interface::transfer_checked` em cima do `CpiContext::new(token_program.address(), pull)`. Sem `with_signer` aqui: o trader é um signer de verdade na transação. Checkpoint: depois desta CPI a reserva de arcade do pool cresceu em `amount_in`.

5. **Complete a CPI de token-para-fora (preenchimento).** Você recebe o stub:

```rust
// TODO(you): move `out` tickets from reserve_ticket to trader_ticket,
// signed by the pool PDA. Mirror the token-in CPI, but:
//   - from/to are the ticket accounts, not the arcade accounts
//   - the mint is mint_ticket
//   - the authority is the pool PDA, so you must attach with_signer
let bump = [ctx.accounts.pool.bump];             // read out first, same as the vault
let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, &bump]];
let push = TransferChecked {
    // fill in the four accounts using cpi_handle_mut() / cpi_handle()
};
// invoke transfer_checked over CpiContext::new(...).with_signer(signer_seeds)
// with `out` and mint_ticket.decimals()
```

Preencha ele contra a versão trabalhada acima. A única coisa que você não pode perder é o `.with_signer(signer_seeds)`. Sem ele o runtime não tem assinatura nenhuma para o PDA do pool e a transferência falha com um erro de signer faltando. Checkpoint: uma troca move saldos de ticket de verdade para dentro do `trader_ticket`.

6. **Adicione a trava de slippage (solo).** Antes de qualquer das duas CPIs, depois de você computar o `out`, reverta quando o `out < min_out`. Você tem a variante de erro (`SwapError::SlippageExceeded`) e você escreveu esta forma exata de release condicional no escrow. Checkpoint: uma troca com o `min_out` definido um acima da saída cotada reverte com o erro de slippage, e uma troca com um `min_out` razoável preenche.

7. **Escreva a trava (solo).** Dois testes de LiteSVM em `programs/token-ticket-swap/tests/swap_invariant.rs`, construídos em cima da forma de `spl_setup` da lição passada: crie dois mints, faça init do pool, financie as duas reservas, financie um trader. Depois:

   - `invariant_holds`: leia o `k_before = reserve_in * reserve_out` como `u128`, mande um swap com um `min_out` permissivo, releia as duas reservas, e assere que `k_after >= k_before`. Maior-ou-igual, não igual: divisão de inteiros arredonda a saída do trader para baixo, então o pool fica com o resto e o `k` só cresce.
   - `slippage_reverts`: cote a troca com o `swap_out` no teste, mande ela com `min_out = quote + 1`, e assere que a transação dá erro.

Quando todos os sete estiverem prontos, rode a trava:

```bash
anchor build && cargo test --test swap_invariant
```

Os dois verdes. Se o `invariant_holds` falhar com `k_after < k_before`, o seu arredondamento está favorecendo o trader em algum lugar. Se o `slippage_reverts` preencher em vez de reverter, a sua trava está comparando na direção errada ou está faltando.

## Challenge: cote um swap de produto constante

O lab ligou o swap dentro de um programa, onde uma cotação errada aparece como uma asserção de saldo que falha três camadas longe. O challenge levanta o `swap_out` para fora do framework por completo para a aritmética ser a única coisa que consegue estar errada. O starter, em `lessons/m05-l2/cp-swap-out/`, é a cotação linear ingênua: ele ignora tanto a taxa quanto o deslocamento de reserva, cota demais, e deixaria um trader drenar o pool.

Implemente o `swap_out(reserve_in, reserve_out, amount_in) -> u64` para que ele:

- aplique a taxa de 0.3% (`997/1000`) sob o invariante de produto constante, não a razão de preço ingênua;
- use um intermediário `u128` para duas reservas `u64` não conseguirem dar overflow na multiplicação;
- retorne `0` para uma entrada zero ou uma reserva vazia.

Os oito casos que a bancada assere:

| reserve_in | reserve_out | amount_in | esperado |
|---|---|---|---|
| 1_000 | 1_000 | 100 | 90 |
| 1_000_000 | 1_000_000 | 1_000 | 996 |
| 5_000 | 10_000 | 500 | 906 |
| 1_000 | 1_000 | 0 | 0 |
| 1_000_000 | 1_000_000 | 10_000 | 9_871 |
| 0 | 1_000_000 | 10_000 | 0 |
| u64::MAX | u64::MAX | u64::MAX | 0 — sem panic, sem overflow |
| 1e18 | 1e18 | 1e12 | 996_999_005_991 |

Três empurrõezinhos se você travar:

- `amount_in_with_fee = amount_in * 997`
- `out = (amount_in_with_fee * reserve_out) / (reserve_in * 1000 + amount_in_with_fee)`
- promova para `u128` antes de multiplicar para reservas `u64` não conseguirem dar overflow

A função é uma `const fn` com asserções de tempo de compilação embaixo dela (o dispositivo do m03-l3, e o que avaliação só-de-compilação de fato impõe), então o starter anuncia os bugs próprios dele em tempo de build: as linhas de cotação-errada falham em asserções cujas mensagens nomeiam o caso, e as linhas grandes falham mais forte — a avaliação de const recusa a multiplicação `u64` ingênua de cara com `attempt to multiply with overflow`, que é o rustc fazendo o argumento da lição para você. As duas últimas linhas são as que separam uma cotação que funciona de uma correta. A linha de 1e18 dá overflow numa multiplicação `u64` de cara: em runtime isso dá panic em debug e dá wrap em release, e uma cotação com wrap é um saque de graça. A linha de `u64::MAX` então dá overflow no `u128` também, e vale ser exato sobre por quê, porque a razão *não* é que dois fatores `u64` param de caber. Eles sempre cabem: `(u64::MAX)²` é `2¹²⁸ − 2⁶⁵ + 1`, que aterrissa logo abaixo do teto do `u128` com menos de um bit único de sobra. O numerador aqui é `amount_in * 997 * reserve_out` — aqueles dois fatores de tamanho `u64` *mais* o multiplicador de taxa — e o `× 997` é o que gasta a lasca que tinha sobrado. Então promover é necessário e ainda não suficiente: a multiplicação tem que continuar `checked` e degradar para `0`. Um `u128 *` simples passa em cada outra linha e morre naquela. Compute a primeira linha na mão antes de você codificar ela. Quando a sua aritmética e o programa concordarem, você entende a curva, não só o código.

## Onde isso te deixa

Você tem o R4: um pool de dois lados que cota a si mesmo a partir das reservas dele, move tokens nas duas direções sob a assinatura de um programa, e recusa uma troca que preencheria pior do que o trader aceita. Você derivou a curva, você viu por que o intermediário `u128` não é opcional, e você viu a antiga cilada do `.reload()` virar uma coisa que o compilador simplesmente se recusa a te deixar escrever. Essa última parte é o tema deste curso inteiro: o V2 move classes de bug de runtime para tempo de compilação, e o swap é onde você sentiu três delas de uma vez.

Você construiu tudo isso contra um mint SPL simples. Aqui está a pergunta que abre a próxima lição. O que acontece no momento em que um player traz um mint Token-2022 que cobra uma taxa de transferência e roda um transfer hook? O seu `transfer_checked` ainda chama, mas o número que você cotou ainda casa com o número que chega, e as contas extra do hook cabem sequer pela CPI que você acabou de escrever? Na próxima lição você aprende a responder isso a partir do próprio mint, lendo um vivo antes de confiar nele. Corrigir um swap para um mint com hook é trabalho de padrões do qual o curso de Digital Assets é dono; saber que você teria que fazer isso é a parte que é sua, e ela está a uma leitura de distância.
