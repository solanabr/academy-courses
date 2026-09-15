# Capstone de migração: porte um programa 0.31/1.0 para o V2

No m10-l2 você terminou o segundo mapa. Cada delta de reescrita de 1.x para 2.0, do `Pubkey` virando `Address` até o `AccountLoader` repropositado. Você também dirigiu dois ports pequenos: um programa de config de cinquenta linhas no lab, com a tabela de delta aberta ao seu lado, e um cálculo de space no challenge. Então: dois mapas e dois exercícios de rascunho. O que você não fez é levar uma base de código que você não escreveu para o outro lado da linha. Isso muda agora.

Porque aqui está a coisa sobre um mapa: ele não é o percurso. Você consegue ler os dois mapas de delta de cabo a rabo, concordar com cada flecha, e ainda congelar na primeira vez em que uma base de código de verdade te joga uma parede de texto de erro vermelho. Então a gente vai pegar um programa de verdade, um que não compila no V2, e dirigir ele até o verde juntos. Não admirar os deltas. Aplicar eles.

Antes de você ler outro parágrafo, rode isto e olhe o número que ele imprime:

```bash
anchor --version
```

Na máquina de referência deste curso isso diz `anchor-cli 1.1.2`. Segure aquele número — não porque o CLI decide o que o seu build é (ele não decide, e o fato mais importante desta lição é *o que de fato decide*), mas porque uma máquina que imprime 1.x é uma máquina cujos hábitos, e cujo vault entregue, continuam fixados nos crates 1.x. O pin é a história. A gente volta nele no passo 1.

## Resumo

Você recebe um vault de lamports Anchor 0.31/1.0 funcionando: uma conta de estado, um PDA que segura SOL, e três instruções (`initialize`, `deposit`, e um `withdraw` assinado por PDA). Ele compila bem no 1.x. Ele não compila no RC do Anchor V2. O seu trabalho é fazer o compilador ficar verde e o teste de LiteSVM passar, trabalhando os dois mapas de delta como um checklist.

O vault foi escolhido para ser chato de propósito. Ele ecoa o quarter-vault que você construiu lá no m03 e no m04, então quase nenhuma da sua atenção vai para "o que este programa faz". Toda ela vai para a migração em si. É esse o design inteiro: carga de domínio baixa, foco de migração alto.

A maior parte dos deltas é mecânica, e o programa fornecido marca eles para você com comentários `// TODO(migrate):`. Os dois que importam mais não carregam marcação nenhuma, porque no fim você não vai precisar de uma. O compilador vai te dizer. Um aviso de depreciação sublinha o constraint exato para mudar. Um método faltando aponta para a linha exata em que um hábito de v1 não tem mais nada para chamar. É esse o núcleo emocional deste capstone: você aprendeu o bastante para a saída própria do ferramental ser um guia bom o bastante. A gente chama isso de *deixar o compilador dirigir*, e é uma facilidade de verdade do V2, não um slogan motivacional. Você vai ver por quê.

Uma ressalva honesta de cara, porque ela molda tudo: este port compila hoje contra um alvo em movimento. A linha do Anchor V2 é a `2.0.0-rc.1`, e a branch anchor-next em que ela mora é rotulada de alpha pelos próprios mantenedores dela: não auditada, as APIs podem quebrar entre commits, e a documentação fica atrás do código (os crates do rc.1 chegaram no crates.io em 2026-08-12, e ainda assim a documentação de instalação continua descrevendo o mundo pré-publicação). Então a gente constrói isto não como um artefato eterno mas como um re-verificável. Quando um RC posterior renomear um constraint embaixo de você, você re-roda o checklist. Essa é a realidade de migração, e a lição final deste curso pergunta se você deveria se inscrever nisso de jeito nenhum.

## A migração, delta por delta

Vamos acertar o toolchain primeiro, porque cada outro delta vem depois dele.

### O toolchain é o jogo inteiro

Lembra daquele `1.1.2` de um minuto atrás? Aqui está a armadilha que ele arma, e ela é mais subtil que "binário errado". O vault entregue compila bem no 1.x, que quer dizer que o `Cargo.toml` dele fixa o `anchor-lang` na linha 1.x — e *aquele pin, não o CLI no seu PATH, é o que seleciona o major do framework*. O `anchor build` é um wrapper; embaixo dele, o cargo resolve o seu grafo de crates identicamente não importa qual anchor-cli invocou ele. Então se você clona o vault, começa a corrigir nomes de tipo, e nunca toca no manifesto, você não recebe um artefato silencioso de v1 — você recebe uma falha alta: o `Address`, o `.address()`, o `&mut Context` não existem em lugar nenhum dos crates 1.x, e o compilador diz isso em cada site que você acabou de editar. O reverso vale também: suba o pin para `2.0.0-rc.1` e até o CLI antigo do host traz à superfície as depreciações do V2 e o erro de método faltando, porque esses diagnósticos vêm das macros no grafo de dependências, não do binário que chamou o cargo. Você já encontrou essa inversão duas vezes — o reconhecimento do m10-l1 te fez rodar `rg "anchor_version|anchor-lang"` precisamente porque o pin é o fato que importa, e o m10-l2 te disse para fixar a versão exata no `Anchor.toml` e no `Cargo.toml`. A versão que o *build* é, é a versão que o *manifesto* diz.

Então a primeira jogada não é uma edição de handler. São dois pins, feitos juntos: a linha `anchor-lang = "2.0.0-rc.1"` no `Cargo.toml` do programa, que é a chave que de fato vira o major, e um CLI V2 isolado, que mantém cada comportamento de nível de wrapper — scaffolds, a bancada de teste, tratamento de IDL — na mesma linha que os crates, para o `anchor --version` continuar sendo um rótulo verdadeiro para o toolchain inteiro.

A instalação briga um pouco com você, e vale saber por quê. O V2 não tem objeto de GitHub Release. Existe uma tag git, a `v2.0.0-rc.1` na branch anchor-next, mas nenhum release publicado para aquela tag, que quer dizer que o `avm install` não consegue baixar um binário pré-compilado para ela do jeito que ele faz para versões estáveis — a URL do asset simplesmente dá 404. Os crates do rc.1 de fato aterrissaram no crates.io em 2026-08-12, mas a documentação fica atrás daquela publicação e o caminho documentado é uma instalação direta por git (a documentação aponta para a ponta da branch `anchor-next`; este curso fixa a tag que fica naquela branch, pela razão de reprodutibilidade que o m01-l2 expôs):

```bash
# Anchor V2 RC - installed straight from the anchor-next repo by tag.
# No GitHub Release cut for the tag, so `avm install` finds no binary to fetch.
# macOS needs LTO off or the release build blows up; harmless elsewhere.
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force
```

Freshness note: a `v2.0.0-rc.1` é o pin em 2026-08-22, e ela está numa branch que a própria documentação dela chama de alpha com APIs que podem quebrar entre commits. Antes de você confiar num build, re-cheque a tag atual na anchor-next e atualize o pin. Esta não é uma versão para memorizar; é uma para re-verificar.

Dois fatos de toolchain a mais que você precisa. O Rust mínimo suportado pelo V2 é o `1.89.0`, hardcoded como `ANCHOR_MSRV` no CLI e escrito dentro do `rust-toolchain.toml` que ele faz scaffold, então confirme o seu compilador e atualize se você está atrás:

```bash
rustc --version      # need >= 1.89.0 for V2
rustup update        # if you are below it
```

E você não pode deixar o 1.1.2 do ambiente voltar para dentro durante a verificação. O jeito de garantir isso é fixar o RC no contêiner de verify para o build nunca tocar o toolchain do host:

```dockerfile
# verify/Dockerfile - the port builds ONLY against the pinned RC.
FROM rust:1.89
ENV CARGO_PROFILE_RELEASE_LTO=off
RUN cargo install --git https://github.com/otter-sec/anchor \
      --tag v2.0.0-rc.1 anchor-cli --locked --force
WORKDIR /work
COPY . .
CMD ["anchor", "test"]
```

![Construir o vault editado contra um Cargo.toml que ainda fixa o anchor-lang 1.x falha em cada linha editada sob qualquer CLI; só um grafo fixado no 2.0.0-rc.1 emite as depreciações do V2 e o erro de método faltando, e um build V2 de verdade.](assets/v01-flowchart.png)

É essa a montagem em cima da qual tudo o mais fica de pé. Erre ela e cada edição de código abaixo é teatro. Acerte ela e o compilador começa a fazer o seu trabalho para você.

### O mapa de delta, numa página

Aqui está cada delta de reescrita que o vault toca, lado a lado. Este é o checklist. Mantenha ele aberto enquanto você trabalha.

| # | v1 (0.31/1.0) | V2 (2.0.0-rc.1) | como você acha |
|---|---|---|---|
| 1 | `Pubkey` | `Address` | erro de tipo no campo |
| 2 | `account.key()` | `account.address()` | erro de método-não-encontrado |
| 3 | `struct Foo<'info>` + `ctx: Context<Foo>` | largue o `<'info>`, o handler pega `&mut Context<Foo>` | erro de lifetime / de assinatura |
| 4 | `space = 8 + T::INIT_SPACE` | `space = T::DISCRIMINATOR.len() + T::INIT_SPACE` | ainda compila, mas o mapa diz para corrigir |
| 5 | `CpiContext::new(prog.key()..)` | `CpiContext::new(prog.address()..)`, as contas de CPI viram `.cpi_handle_mut()` | erro de tipo: esperava `&Address` |
| 6 | `has_one = authority` | `address = state.authority` na conta de authority | **aviso de depreciação sublinha isso** |
| 7 | `account.reload()?` depois de uma CPI | delete; nada muda embaixo de uma conta tipada carregada | **E0599: nenhum método chamado `reload`** |

Uma nota sobre a linha 5 antes de você usar a tabela. O vault que você recebe compila no 1.1.2, então a CPI dele já passa o programa como um `Pubkey` com o `.key()`; aquele salto foi a mudança dois do m10-l1. Se a base de código que você trouxer para um port de verdade ainda estiver no 0.31 ela vai ler `.to_account_info()` ali em vez disso, e você faz os dois saltos de uma vez.

Duas linhas estão faltando de propósito, e as duas valem uma frase para o seu mapa ficar completo mesmo que este vault não tropece nelas.

O `zero_copy` agora é o layout default no V2, então o atributo simplesmente desapareceu. O nosso vault nunca usou ele, então não tem nada para arrancar. Num programa que usava, você deletaria o atributo e a conta continua funcionando, porque o que costumava ser um opt-in agora é só como contas são dispostas.

O `unsafe(dup)` é o mais interessante, e o challenge vai fazer você usar ele, então entenda agora. O V2 desabilita contas mutáveis duplicadas por padrão. A razão é uma cilada de verdade: se a mesma conta chega em dois slots mutáveis, o seu handler acaba segurando duas referências `&mut` para uma conta, e edições através de uma caladamente atropelam edições através da outra. O v1 te deixava fazer isso e esperava que você soubesse o que estava fazendo. O V2 rejeita, na validação, antes de o seu handler rodar. Quando você genuinamente quer receber uma conta sob dois nomes mutáveis, você opta de volta por campo escrevendo o constraint `unsafe(dup)`. A palavra `unsafe` está fazendo trabalho honesto: ela é você dizendo ao compilador que você checou o invariante que ele não consegue mais checar para você, e assumindo a obrigação de escrever o handler para ele nunca segurar duas referências mutáveis conflitantes. Note o que *não* precisa do opt-out: dois slots mutáveis que sempre resolvem para dois endereços diferentes, do jeito que as duas reservas de um swap fazem, satisfazem a checagem de graça. O nosso vault de lab tem exatamente uma conta de cada tipo, então isso nunca aparece. A varredura de consolidação do challenge aparece.

![Uma tabela agrupada de antes/depois de sete deltas: cinco que o compilador sinaliza como erros de tipo, dois que ele traz à superfície como um aviso de depreciação e um erro de método faltando.](assets/v02-comparison.png)

### Por que o `.reload()` desapareceu (e por que isso é bom)

As linhas 1 até 5 são find-and-replace com um compilador checando o seu trabalho. As linhas 6 e 7 são as que valem entender, porque elas são onde o port para de ser mecânico.

Comece pelo `.reload()`, porque ele é o que derruba cada dev de v1 experiente, e o raciocínio atrás da remoção dele é a ideia mais interessante da migração inteira.

Aqui está o padrão de v1, e ele não é nem ruim:

```rust
// v1 withdraw tail: build the transfer, read state while it is pending, run it, reload.
let cpi_ctx = CpiContext::new_with_signer(sys, Transfer { from, to }, signer);
let before = ctx.accounts.state.total_withdrawn; // legal in v1: `from`/`to` are AccountInfo clones
system_program::transfer(cpi_ctx, amount)?;
ctx.accounts.state.reload()?;                    // re-deserialize after the CPI
let bal = ctx.accounts.state.total_deposited;    // read the "fresh" value
```

Você derivou a remoção na lição passada: leituras tipadas obsoletas através de uma fronteira de CPI eram um bug com forma de patch, e o rastreamento de borrow do `CpiHandle` deixa elas inexprimíveis — um borrow por conta entregue para a CPI, e nada mais largo, exatamente como o m04-l2 percorreu. Nenhuma re-derivação aqui. O que esta lição adiciona é a *borda* daquela regra, porque ela é mais estreita do que soa no começo e este vault fica exatamente nela. Porte aquela cauda linha por linha e exatamente uma linha para o build: o `.reload()`, que não existe mais como método para chamar. A leitura de `total_withdrawn` uma linha acima dele sobrevive, porque os handles desta transferência estão no `vault` e no `authority` enquanto o `state` é uma conta disjunta que o chamado nunca conseguiria ter escrito — nada para excluir ali, e nada que poderia ter ficado obsoleto. Isso é o modelo fazendo o trabalho dele, não falhando: a exclusão aterrissa nas contas que a CPI de fato consegue mudar. Entregue uma conta tipada para uma CPI, do jeito que a sondagem de token do m04-l2 fez, e uma leitura de *aquela* conta no meio da CPI é o erro de compilação.

Sente com isso por um segundo, porque é uma filosofia genuinamente diferente — e mantenha dois mecanismos separados, porque o borrow checker é só metade. O primeiro é o modelo de conta em si. O `Account<T>` default do V2 é uma *view* zero-copy em cima dos bytes da conta, não uma cópia decodificada uma vez no topo da instrução, então não existe uma segunda cópia que poderia derivar para fora de data. A camada borsh que você está a ponto de usar aqui, o `BorshAccount<T>`, de fato ainda decodifica numa cópia — mas ela segura o borrow de dado da conta por todo o tempo em que está carregada, e você tem que entregar aquele borrow explicitamente antes de uma CPI conseguir escrever aqueles bytes. De qualquer jeito, nada muda embaixo de uma conta tipada carregada sem a sua palavra, então não tem nada para uma chamada de re-desserializar re-ler. É por isso que o método não existe. O segundo mecanismo é o modelo de borrow, e ele cobre a janela restante: enquanto uma CPI segura um handle para uma conta, acesso tipado a *aquela* conta não vai compilar. O v1 te dava uma ferramenta para evitar uma cilada. O V2 removeu os lugares em que a cilada poderia ficar. A classe de bug desapareceu, não está guardada.

![No v1 uma cópia tipada decodificada uma vez fica obsoleta através de uma CPI e o .reload() re-lê ela; no V2 a conta tipada carregada segura o borrow de dado, então nada muda embaixo dela e não tem método de reload para chamar.](assets/v03-diagram.png)

Então a correção não é "ache o nome V2 do reload". Não tem nenhum. A correção é estrutural: não segure dado tipado de uma conta que esta CPI pega através da CPI. Leia os escalares de que você precisa (o bump, a chave de estado) para dentro de locais antes da transferência, rode a transferência, e depois pegue um borrow tipado novo depois de ela completar para atualizar os seus contadores — e aquela leitura pós-CPI já está viva, que é por que não sobra nada para um `.reload()` fazer. O erro não é um obstáculo. Ele é a instrução. É isso deixar o compilador dirigir.

![O build do V2 joga exatamente um erro no withdraw de v1 copiado, nenhum método chamado reload, enquanto a leitura tipada uma linha acima dele compila porque o state não é uma conta que esta transferência toca.](assets/v04-annotated-code.png)

### Por que o `has_one` ainda compila mas você corrige mesmo assim

A linha 6 é a outra edição sem marcação, e ela ensina um reflexo diferente.

O `has_one = authority` ainda funciona no V2. Ele parseia, ele checa, o teste passa com ele no lugar. Então por que tocar nele? Porque quando você compila, você recebe isto:

```text
warning: use of deprecated function `__deprecated_has_one`: `has_one` is
         deprecated; on the sibling field, use
         `#[account(address = owner.field)]` instead.
   --> programs/quarter_vault/src/lib.rs:126:9
    |
126 |         has_one = authority,
    |         ^^^^^^^
```

Duas coisas sobre aquele aviso valem notar. Primeira, ele nomeia a substituição exatamente, e ele te diz onde colocar ela: no campo irmão, como `#[account(address = owner.field)]`. Para o nosso vault, isso é `address = state.authority` colocado na conta `authority`, que checa que o endereço da authority passada é igual ao campo `authority` guardado no `state`. A mesma garantia, grafia nova. Segunda, e este é o toque de cor que eu quero que você segure: aquele sublinhado não é um acidente. Lá embaixo no parser, o `parse.rs` deliberadamente mantém o span de código da keyword `has_one` em volta para a geração de código conseguir emitir um aviso apontando direto de volta para aqueles caracteres exatos. Ninguém sublinha um token que não planejou depreciar. O ferramental foi construído para guiar a migração que ele criou. O aviso é uma feature, não ruído.

![O aviso de depreciação do has_one nomeia a substituição própria dele, sublinha o token exato para remover, e não falha o teste, deixando ele um item de checklist.](assets/v05-annotated-code.png)

E num RC em movimento, sintaxe depreciada é precisamente o que uma versão posterior tem mais chance de remover. Resolver depreciações até zero não é arrumação. É como você mantém o port compilando contra a tag do mês que vem. O aviso é um item de checklist que o framework te entrega de graça.

Aqui está o antes e o depois para aquele constraint único:

```rust
// v1: has_one lives on the state account (seeds elided).
#[account(mut, has_one = authority)]
pub state: Account<'info, VaultState>,
#[account(mut)]
pub authority: Signer<'info>,
```

```rust
// V2: the equivalence check moves onto the authority account as an address constraint.
#[account(mut)]
pub state: BorshAccount<VaultState>,
#[account(address = state.authority)]
pub authority: Signer,
```

Note a ordem de declaração: o `state` antes do `authority`, para a expressão `address = state.authority` conseguir resolver — o vault entregue já lista eles dessa forma, e a grafia do V2 é o que deixa aquela ordem estrutural. Note também o `<'info>` largado na conta tipada. Essas são as linhas 1 até 3 pegando carona. Os deltas se agrupam; corrigir um muitas vezes aterrissa três.

## Lab: dirija o vault até o verde

Hora de construir. Você tem o mapa de delta e você entende as duas linhas difíceis. Agora aplique elas. O programa entregue está impresso por inteiro abaixo. Faça o scaffold de um projeto com o CLI 1.1.2 que já está na sua máquina (`anchor init quarter_vault` — este é o único uso legítimo que o toolchain antigo tem sobrando neste curso), substitua o `programs/quarter_vault/src/lib.rs` gerado pela listagem, e garanta que o manifesto do programa carrega a linha de v1 que o scaffold escreveu:

```toml
# programs/quarter_vault/Cargo.toml — the row step 1 will flip.
[dependencies]
anchor-lang = "1.1.2"
```

Aqui está o vault, inteiro. As marcações `// TODO(migrate):` ficam nos sites mecânicos; trabalhe de cima para baixo.

```rust
// programs/quarter_vault/src/lib.rs — the handed 0.31/1.0 vault.
// Builds clean on anchor-lang 1.1.2. Does NOT build on the V2 RC.
// The `// TODO(migrate):` markers flag the mechanical sites (rows 1-5).
// Rows 6 and 7 carry no marker on purpose: the compiler finds them for you.

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("Quart3rVau1t1111111111111111111111111111111");

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub authority: Pubkey, // TODO(migrate): row 1 — Pubkey -> Address
    pub bump: u8,
    pub vault_bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}

#[error_code]
pub enum VaultError {
    #[msg("counter overflow")]
    Overflow,
}

#[program]
pub mod quarter_vault {
    use super::*;

    // TODO(migrate): row 3 — handlers take `&mut Context<T>` in V2.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let authority = ctx.accounts.authority.key(); // TODO(migrate): row 2 — .key() -> .address()
        let state = &mut ctx.accounts.state;
        state.authority = authority;
        state.bump = ctx.bumps.state; // canonical bumps, stored once
        state.vault_bump = ctx.bumps.vault;
        state.total_deposited = 0;
        state.total_withdrawn = 0;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.key(), // TODO(migrate): row 5 — &Address + cpi handles
                Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;
        let state = &mut ctx.accounts.state;
        state.total_deposited = state
            .total_deposited
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let state_key = ctx.accounts.state.key(); // TODO(migrate): row 2
        let vault_bump = ctx.accounts.state.vault_bump;

        let seeds: &[&[u8]] = &[b"vault", state_key.as_ref(), &[vault_bump]];
        let signer = &[seeds];

        // v1 tail: build the transfer, read state while it is pending, run it, reload.
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.key(), // TODO(migrate): row 5
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.authority.to_account_info(),
            },
            signer,
        );
        let already_out = ctx.accounts.state.total_withdrawn; // legal in v1: the CPI holds AccountInfo clones
        system_program::transfer(cpi_ctx, amount)?;

        ctx.accounts.state.reload()?; // v1 habit: re-deserialize after the CPI

        let state = &mut ctx.accounts.state;
        state.total_withdrawn = already_out
            .checked_add(amount)
            .ok_or(VaultError::Overflow)?;
        Ok(())
    }
}

// TODO(migrate): row 3 — drop the <'info> lifetimes on every struct below.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + VaultState::INIT_SPACE, // TODO(migrate): row 4 — no magic 8 in V2
        seeds = [b"state", authority.key().as_ref()],
        bump
    )]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"vault", state.key().as_ref()], bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"state", state.authority.as_ref()], bump = state.bump)]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut, seeds = [b"vault", state.key().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"state", authority.key().as_ref()],
        bump = state.bump,
        has_one = authority,
    )]
    pub state: Account<'info, VaultState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"vault", state.key().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}
```

Aquele arquivo passa no `cargo check` limpo contra o `anchor-lang 1.1.2` — zero erros, zero avisos de depreciação, porque as depreciações são uma história de V2 e trazer elas à superfície é para o que o passo 6 existe. E se você fez o From Bitcoin to Solana, este é o seu próprio vault da lição an-anchor-vault dele — a mesma divisão de registro-mais-vault, os mesmos bumps canônicos guardados, o mesmo gate de authority, com o campo `balance` único crescido nos dois contadores — então você está bem-vindo a portar o que as suas próprias mãos construíram em vez deste.

**1. Vire o pin, e depois levante o toolchain isolado.** Primeiro a edição que de fato seleciona o V2 — a jogada de manifesto que a seção de teoria acabou de deixar estrutural. Abra o `programs/quarter_vault/Cargo.toml` e mude a linha de `anchor-lang` da versão 1.x dela para o RC:

```toml
[dependencies]
anchor-lang = "2.0.0-rc.1"   # was a 1.x row; THIS line is what selects the framework major
```

Sem esta edição, nenhum dos passos 2 até 7 consegue nem começar a compilar: cada rename abaixo tem como alvo nomes que não existem nos crates 1.x. Depois instale o CLI do RC exatamente como acima, e confirme que o rótulo é verdadeiro:

```bash
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor \
  --tag v2.0.0-rc.1 anchor-cli --locked --force

anchor --version    # must now report 2.0.0-rc.1, NOT 1.1.2
```

Se isso ainda diz 1.1.2, o seu PATH está resolvendo o binário antigo primeiro. Isso não vai mudar contra qual framework o seu grafo de crates compila — o pin acima governa isso — mas um toolchain mal rotulado é como comportamento de scaffold, de bancada de teste e de IDL deriva para fora da linha em que os seus crates estão, então corrija antes de escrever uma linha só. Este é o passo 1 por uma razão.

**2. Renomeie os tipos (linhas 1 e 2).** Mude cada `Pubkey` para `Address` e cada `.key()` para `.address()`. Compile. O compilador vai listar os que você perdeu como erros de tipo e de método. Deixe ele. Aqui está a struct de estado depois desta passada:

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

declare_id!("Quart3rVau1t1111111111111111111111111111111");

#[account(borsh)]
#[derive(InitSpace)]
pub struct VaultState {
    pub authority: Address,   // was Pubkey
    pub bump: u8,
    pub vault_bump: u8,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
}
```

Note o `(borsh)` que você teve que adicionar, porque este é o delta que o mapa não lista e no qual o port tropeça imediatamente. `#[account]` simples no V2 quer dizer zero-copy Pod, e Pod quer dizer `#[repr(C)]` com uma asserção de tempo de compilação de que o tamanho da struct é igual à soma dos tamanhos dos campos dela. Os campos do `VaultState` somam 50 bytes, mas os dois `u64` forçam alinhamento de 8 bytes, então o `repr(C)` arredonda a struct para 56 e a asserção dispara: "account struct has padding bytes." Você tem duas saídas legais. Re-disponha o estado com campos de alinhamento 1 (o prelúdio entrega `PodU64`, `PodI128`, `PodBool` para exatamente isso) e mantenha o caminho zero-copy, ou mande esta conta única pelo caminho borsh com o `#[account(borsh)]` e o wrapper `BorshAccount<T>`. Um port 1:1 pega a segunda saída, que é também aquela contra a qual o `#[derive(InitSpace)]` está documentado. Re-arquitetar para Pod é o projeto separado sobre o qual a gente fala no fim.

**3. Arranque os lifetimes e corrija as assinaturas de handler (linha 3).** Largue o `<'info>` de cada struct de accounts e dos tipos de campo dela. Mude cada handler de `ctx: Context<T>` para `ctx: &mut Context<T>`. O handler `initialize` depois desta passada:

```rust
#[program]
pub mod quarter_vault {
    use super::*;

    pub fn initialize(ctx: &mut Context<Initialize>) -> Result<()> {
        let authority = *ctx.accounts.authority.address();   // .address() returns &Address
        let state = &mut ctx.accounts.state;
        state.authority = authority;
        state.bump = ctx.bumps.state;
        state.vault_bump = ctx.bumps.vault;
        state.total_deposited = 0;
        state.total_withdrawn = 0;
        Ok(())
    }
    // deposit, withdraw below
}
```

Compile de novo. Espere cada erro de lifetime e de assinatura limpar, e espere as linhas de CPI e de constraint abaixo ainda estarem vermelhas. Aquela lista de erros encolhendo é a sua barra de progresso.

**4. Corrija o cálculo de space (linha 4).** Este ainda compila como `8 + VaultState::INIT_SPACE`, então o compilador não vai te forçar. O mapa força. Substitua o `8` mágico pelo tamanho de verdade do discriminator:

```rust
#[account(
    init,
    payer = authority,
    space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,  // was 8 +
    seeds = [b"state", authority.address().as_ref()],
    bump
)]
pub state: BorshAccount<VaultState>,
```

**5. Corrija a CPI de deposit (linha 5).** Duas edições pegam carona juntas aqui. O `CpiContext::new` agora pega o programa como `&Address`, e o `.address()` já te entrega um, então não tem `&` para adicionar. As contas dentro do `Transfer` não são mais `AccountInfo` tampouco: o V2 declara elas como `CpiHandleMut`, que é o handle com borrow rastreado sobre o qual a linha 7 fala, então você monta elas com o `.cpi_handle_mut()`. A transferência de deposit, de quem deposita para dentro do PDA de vault:

```rust
pub fn deposit(ctx: &mut Context<Deposit>, amount: u64) -> Result<()> {
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.address(),   // was .key()
            Transfer {
                from: ctx.accounts.depositor.cpi_handle_mut(),
                to: ctx.accounts.vault.cpi_handle_mut(),
            },
        ),
        amount,
    )?;
    let state = &mut ctx.accounts.state;
    state.total_deposited = state
        .total_deposited
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    Ok(())
}
```

Compile. As linhas mecânicas estão prontas. Agora as duas que o compilador dirige.

Uma palavra rápida sobre o que você provavelmente está vendo agora, porque duas falhas são comuns exatamente neste ponto e as duas parecem mais assustadoras do que são. Se o build vomita dezenas de erros de tipo de `Address`-contra-`Pubkey` em arquivos que você nunca tocou, você perdeu um `.key()` em algum lugar mais acima e o tipo errado está se propagando. Corrija o mais antigo na lista do compilador primeiro, não o mais alto, porque os erros posteriores em geral são só consequência dele. E se ele vomita erros de nome não resolvido exatamente nas linhas que você já corrigiu — `Address` desconhecido, `.address()` faltando — o culpado é a outra metade do passo 1: o pin do `anchor-lang` continua no 1.x, então os nomes *para* os quais você renomeou não existem no grafo contra o qual você está compilando. O CLI no seu PATH não consegue nem causar nem curar nenhum dos dois sintomas; os diagnósticos vêm dos crates que o cargo resolveu, exatamente como o passo 1 disse. É o manifesto te mordendo, precisamente como prometido.

**6. Solo: resolva o aviso de depreciação (linha 6).** Não tem TODO para isto. Compile e leia o aviso. Ele sublinha o `has_one = authority` e nomeia a substituição. Mova a checagem para um constraint `address` na conta de authority, exatamente como mostrado antes. Recompile até a contagem de depreciações ser zero. Não pare em "o teste passa". Pare em "o aviso desapareceu".

**7. Solo: resolva o método faltando (linha 7).** Também sem TODO. O `withdraw` fornecido copia o padrão de v1: ele monta o `CpiContext` assinado num local, lê o `state` enquanto aquele valor ainda está sentado ali, roda a transferência, chama o `.reload()`, e depois lê de novo. No V2 exatamente uma daquelas linhas para o build: o `.reload()` não existe. A leitura de `state` uma linha acima dele está bem aqui, porque os handles desta transferência estão no `vault` e no `authority` e o `state` é uma conta disjunta que o chamado nunca toca. Delete o reload, e reestruture para a atualização de contador pegar um borrow tipado novo depois da chamada — e para nada de que as signer seeds fazem borrow sair de escopo antes de a CPI usar elas. Aqui está a forma que você está buscando; escreva ela antes de você ler ela, porque o passo 6 e o passo 7 são os dois que o compilador deveria dirigir:

```rust
pub fn withdraw(ctx: &mut Context<Withdraw>, amount: u64) -> Result<()> {
    // Copy the scalars into locals BEFORE the CPI: `seeds` borrows `state_key`,
    // so the local has to outlive the call that uses `signer`.
    let state_key = *ctx.accounts.state.address();
    let vault_bump = ctx.accounts.state.vault_bump;

    let seeds: &[&[u8]] = &[b"vault", state_key.as_ref(), &[vault_bump]];
    let signer = &[seeds];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.address(),
            Transfer {
                from: ctx.accounts.vault.cpi_handle_mut(),
                to: ctx.accounts.authority.cpi_handle_mut(),
            },
            signer,
        ),
        amount,
    )?;

    // NO .reload(). Take a FRESH typed borrow only after the CPI has completed.
    let state = &mut ctx.accounts.state;
    state.total_withdrawn = state
        .total_withdrawn
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;
    Ok(())
}
```

A struct de accounts `Withdraw` carrega a correção da linha 6:

```rust
#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut, seeds = [b"state", authority.address().as_ref()], bump = state.bump)]
    pub state: BorshAccount<VaultState>,
    #[account(mut, address = state.authority)]     // replaces has_one = authority
    pub authority: Signer,
    #[account(mut, seeds = [b"vault", state.address().as_ref()], bump = state.vault_bump)]
    pub vault: SystemAccount,
    pub system_program: Program<System>,
}
```

**8. Rode a trava.** O teste de aceitação é a mesma forma que cada degrau deste curso usou: um teste de LiteSVM que passa. O LiteSVM é o template de teste default do Anchor, então o `anchor init` fez scaffold de uma bancada em Rust embaixo de `tests/`. Alcance o LiteSVM do jeito que o resto deste curso alcançou, através do wrapper do scaffold em vez de um pin direto, para a sua bancada não conseguir derivar para fora da versão que o toolchain espera (o `anchor-v2-testing` do rc.1 fixa o `litesvm 0.11`; o latest do litesvm no crates.io é o 0.16.0 em 2026-09-07, quatro minors à frente, que é exatamente por que você não fixa ele você mesmo):

```toml
# programs/quarter_vault/Cargo.toml - dev-dependencies
[dev-dependencies]
anchor-v2-testing = { git = "https://github.com/otter-sec/anchor", tag = "v2.0.0-rc.1" }
```

O programa fornecido entrega três helpers de send em `tests/helpers.rs`, um por instrução, todos da mesma forma. Aqui está o `send_initialize` para você ver o que os outros dois fazem com um `amount: u64` adicionado nos dados de instrução deles:

```rust
// tests/helpers.rs (provided)
use anchor_lang::{
    prelude::Address, programs::System, solana_program::instruction::Instruction, Id,
    InstructionData, ToAccountMetas,
};
use anchor_v2_testing::{
    Keypair, LiteSVM, Message, Signer as _, VersionedMessage, VersionedTransaction,
};

pub fn send_initialize(svm: &mut LiteSVM, authority: &Keypair, state: Address, vault: Address) {
    let ix = Instruction {
        program_id: quarter_vault::ID,
        accounts: quarter_vault::accounts::Initialize {
            state,
            authority: authority.pubkey(),
            vault,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: quarter_vault::instruction::Initialize {}.data(),
    };
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&authority.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[authority]).unwrap();
    svm.send_transaction(tx).unwrap();
}
```

O teste em si dirige o ciclo de vida inteiro, init e depois deposit e depois withdraw assinado por PDA, e assere que o saldo do vault se moveu:

```rust
mod helpers;
use helpers::{send_deposit, send_initialize, send_withdraw};
use anchor_lang::prelude::Address;
use anchor_v2_testing::{svm, Keypair, Signer as _};

#[test]
fn init_deposit_withdraw_roundtrip() {
    let mut svm = svm();
    let program_id = quarter_vault::ID;
    let vault_so = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/deploy/quarter_vault.so");
    svm.add_program_from_file(program_id, vault_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 5_000_000_000).unwrap();

    let (state, _) =
        Address::find_program_address(&[b"state", authority.pubkey().as_ref()], &program_id);
    let (vault, _) =
        Address::find_program_address(&[b"vault", state.as_ref()], &program_id);

    // init -> deposit(1 SOL) -> withdraw(0.4 SOL), each through the helper above.
    send_initialize(&mut svm, &authority, state, vault);
    send_deposit(&mut svm, &authority, state, vault, 1_000_000_000);
    send_withdraw(&mut svm, &authority, state, vault, 400_000_000);

    let vault_lamports = svm.get_account(&vault).unwrap().lamports;
    assert_eq!(vault_lamports, 600_000_000, "vault should hold 0.6 SOL after the round-trip");
}
```

E depois a trava em si:

```bash
anchor test                              # LiteSVM suite: init, deposit, PDA-signed withdraw
touch programs/quarter_vault/src/lib.rs  # belt and braces: force a genuine recompile before the grep
cargo build 2>&1 | rg "deprecat"         # must print nothing (rg exits 1 on zero matches)
```

Aquele `touch` é precaução redundante em vez de estrutural, e a distinção vale uma frase: o cargo moderno cacheia os diagnósticos de um crate e *repete* eles em builds quentes, então o grep pegaria um `has_one` remanescente até contra um build que não recompilou nada. Dar touch no arquivo só deixa a linha que você grepa provadamente a saída fresca deste build em vez de uma repetição — seguro barato quando você está a ponto de reportar um número como final.

E depois rode os mesmos dois comandos dentro do contêiner de verify, para o resultado que você reporta ter sido produzido pelo RC fixado e nunca pelo que está no seu PATH:

```bash
docker build -t v2-port verify/ && docker run --rm v2-port
```

Teste verde, zero avisos de depreciação, no toolchain do RC, reproduzido no contêiner. É esse o port. É essa a prova.

![Um loop de seis passos: fixe o toolchain do RC, aplique os deltas mecânicos marcados, e depois compile e corrija o que o compilador imprimir até o anchor test ficar verde.](assets/v06-timeline.png)

## Challenge

O lab te entregou o vault. O challenge tira as rodinhas.

O **segundo** programa 0.31/1.0 você monta você mesmo, em `challenge/`, antes de portar ele — dez minutos da memória muscular de 1.x que você acabou de aposentar, e isso te compra uma base de código sem marcação de TODO nenhuma: uma instrução `sweep` de dois vaults que move lamports de um PDA de vault de origem para um PDA de vault de destino numa chamada, e atualiza um contador compartilhado depois da transferência, escrita em idioma v1 completo — `has_one` travando os dois estados de vault, e o contador lido de novo através de um `.reload()` depois da CPI (o vault do lab é o seu template de encanamento; confirme que o original compila limpo na linha 1.x antes de você tocar nele). Os operadores dele também usam ele para ajustar o contador de um vault único fazendo sweep daquele vault para dentro dele mesmo, então escreva o teste de LiteSVM para fazer exatamente isso — uma conta de fato chega nos dois slots mutáveis. Depois porte ele para o V2 e faça aquele teste passar com zero avisos de depreciação.

Três coisas deixam ele mais difícil que o lab, e cada uma mapeia para uma coisa que você agora sabe:

1. Ele usa o `has_one` em dois lugares. Resolva os dois só a partir dos avisos de depreciação.
2. O handler dele lê um contador, faz a CPI de transferência, e depois lê o contador de novo com um `.reload()` no meio. Mate o reload e reestruture as leituras em volta da chamada. O erro de método faltando é o seu mapa.
3. O self-sweep entrega **uma conta para dois slots mutáveis** (origem e destino). O V2 rejeita contas mutáveis duplicadas por padrão, então aquele teste falha na validação com `ConstraintDuplicateMutableAccount` antes de o seu handler rodar. Aplique o `unsafe(dup)` nos dois campos de vault, e, porque o nome diz `unsafe`, escreva uma frase num comentário justificando o aliasing: o handler tem que computar o movimento uma vez e aplicar uma atualização checada única, para ele nunca segurar duas referências mutáveis conflitantes para a conta única. Se você se pegar recorrendo ao `unsafe(dup)` no contador também, pare: aquela é uma conta num slot, e o opt-out estaria escondendo um bug diferente.

![Uma tabela de três linhas emparelhando cada obstáculo do challenge com o aviso, o erro de método faltando, ou a falha de validação que acha ele, mais uma cautela contra aplicar demais o opt-out de conta duplicada.](assets/v07-table.png)

Aceite quando o `anchor test` passar no toolchain do RC e o `cargo build` emitir zero avisos de depreciação. Sem dicas além dos seus dois mapas e do compilador. É esse o ponto.

## Antes de seguir em frente

Note o que acabou de acontecer com a forma desta lição. Os deltas iniciais vinham com marcações de TODO no código e código trabalhado que você conseguia ler direto da página. Os dois últimos passos de lab não tinham marcação nenhuma no código: o aviso e o erro de método faltando localizavam eles para você, e a resposta impressa estava ali para você se checar depois de escrever a sua própria. O challenge larga até isso. Esse recuo foi deliberado. Ele casa com onde você está: no começo da trilha de migração você precisava do delta nomeado e localizado para você; nesta altura o ferramental nomeia e localiza eles melhor do que um comentário conseguiria. Se o passo 7 pareceu menos como seguir instruções e mais como ler a mente do compilador, é essa a habilidade para a qual esta trilha inteira estava construindo. Isso vale mais que qualquer rename de constraint único.

E seja honesto consigo mesmo sobre o que este port é e não é. Uma migração dirigida por checklist é rápida e mecânica, e ela converte um programa um-para-um. O seu vault V2 mantém a forma de v1 dele. Ele não é re-arquitetado para as forças do V2, ele não é o layout Pod mais magro que ele poderia ser, ele é o design antigo que agora compila no framework novo. Isso é um trade-off de verdade, não uma falha: o 1:1 é exatamente o que você quer quando a meta é "fazer isso compilar com segurança", e re-arquitetar é um projeto separado que você assume depois, deliberadamente, não contrabandeado para dentro de uma migração. A outra metade do trade-off é o chão se movendo embaixo de você. Isto compila hoje contra o `2.0.0-rc.1` numa branch alpha cuja própria documentação avisa que as APIs podem quebrar entre commits. Um RC posterior pode renomear o `address` ou mudar como o `unsafe(dup)` é escrito. Você não briga com isso. Você re-roda o checklist.

O port compila e o teste está verde. Você levou uma base de código v1 de verdade até o V2 inteiro, na mão, deixando o compilador dirigir a última milha. Uma pergunta resta, e ela não é técnica: dada a tensão de RC-e-alpha, a data estável não comprometida, e o status não auditado, você *deveria* de fato migrar para o V2 hoje? Essa é uma chamada de julgamento, não um erro de compilação, e a próxima lição, a conclusão do curso, responde ela honestamente.
