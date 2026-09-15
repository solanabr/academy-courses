# Desmonte o framework I: reconstrua o vault nativo

Na lição passada você comprovou um build. Você rodou o `solana-verify` localmente, depois verificou-a-partir-do-repo contra o seu programa na devnet, e os dois hashes casaram. Você viu o fluxo de autoridade só-de-mainnet passar pela OtterSec e pela Squads, demonstrado e claramente rotulado como a coisa que você não toca na devnet. O swap, o R4, está entregue e comprovável. Isso é um marco de verdade: qualquer pessoa agora consegue checar que os bytes on-chain vieram do seu código.

Então aqui está a pergunta incômoda. Cada linha de `#[account(...)]` na qual você se apoiou para chegar lá escreveu uma checagem para você. Uma checagem de owner. Uma checagem de discriminator. Uma checagem de signer. Você nunca viu elas, porque a macro de derive emitiu elas dentro de código que você nunca leu. Delete o framework e aquelas checagens não desaparecem. Elas viram linhas que você ou escreve na mão ou esquece. E uma checagem esquecida num programa que faz custódia de lamports não é um bug, é um exploit.

É essa a lição: você vai reconstruir o R2, o quarter-vault, sem Anchor nenhum, em cima de `pinocchio` cru, mantendo o mesmo comportamento e a mesma trava de aceitação. Lamports saem de um PDA sob autoridade do programa, e um over-withdraw é rejeitado. Macro nenhuma à vista.

Faça o scaffold agora, para o projeto existir enquanto você lê:

```bash
cargo new native-quarter-vault --lib
cd native-quarter-vault
# These three pin as a set: pinocchio-system 0.4 requires pinocchio ^0.9, and
# pinocchio-pubkey 0.3 requires ^0.9 too. Checked 2026-08-22; see the pins note
# at the end of the lesson before you bump any of them.
cargo add pinocchio@0.9 pinocchio-system@0.4 pinocchio-pubkey@0.3
# This crate has no anchor-lang in it, so it is the one place in the course where you
# pin litesvm by name: there is no anchor-v2-testing here to own the version for you.
# Checked 2026-09-01: litesvm 0.16 and solana-sdk 4 build clean on the pinocchio 0.9 line.
cargo add --dev litesvm@0.16 solana-sdk@4
```

Uma edição no `Cargo.toml` antes de qualquer coisa compilar para algo deployável. O `cargo new --lib` te dá um rlib, e o `cargo build-sbf` não vai emitir um `.so` a partir daquilo. Adicione o tipo de crate:

```toml
[lib]
crate-type = ["cdylib", "lib"]
```

O `cdylib` é o que produz o `.so` que o runtime carrega; o `lib` mantém o crate usável como uma biblioteca Rust normal — isso serve testes de unidade dentro do crate e o rust-analyzer, não o teste de integração do passo 5, que nunca linka o seu crate de jeito nenhum: ele carrega o `.so` compilado dentro do LiteSVM com o `add_program_from_file` e dirige ele pelo wire, exatamente como um validador faria. O Anchor escreve esta linha para você em cada scaffold, que é por que você nunca digitou ela.

Uma coisa sobre como esta lição roda. Aqueles quatro comandos são toda a digitação que a visão geral pede: daqui até o Lab eu percorro a forma e você lê. No Lab você codifica junto, passo a passo. O Challenge no fim você faz sozinho, no frio, sem resposta na página. O framework está saindo em estágios, e o apoio embaixo de você também.

## Resumo

Você vai construir o `native-quarter-vault`: um programa pinocchio `no_std` com duas instruções, `init` e um `withdraw` assinado por PDA, despachadas a partir de um discriminator de um byte. Você vai escrever na mão a validação que o `Account<T>` do Anchor gerava para você, assinar uma transferência de lamports para fora de um PDA com o `invoke_signed` usando seeds que você monta você mesmo, e passar exatamente a mesma barra de aceitação que a versão de framework passou, através de uma bancada de teste em LiteSVM que você escreve do zero.

`no_std` quer dizer que a biblioteca padrão está desligada. Nenhum alocador de heap que você não pediu, nada de `std::`, só o `core` e o que você escolher trazer. Seja preciso sobre o que isso é e não é, porque a versão do folclore exagera: programas Solana comuns — cada programa Anchor 1.x incluído — são linkados com `std`. O toolchain SBF entrega um `std` funcional, ainda que aparado, e o entrypoint de fábrica instala um alocador bump default; é exatamente essa a maquinaria escondida que o `no_std` recusa. Então este não é "o modo em que programas Solana de fato rodam". Ele é o modo que o V2 e o pinocchio *escolheram*, e ligar ele você mesmo não é uma escolha estética: é o que mantém o binário pequeno e o compute previsível, porque nada é trazido que você não pediu explicitamente. Aqui você vira essa chave na primeira linha do arquivo.

O trade-off é o ponto inteiro do módulo: o pinocchio nativo compra de volta compute e te entrega controle total, mas você agora escreve na mão e nunca pode esquecer cada checagem que o derive gerava. Discriminator, owner, duplicate-mutable, signer, aritmética checada. Omita uma e você entregou uma vulnerabilidade. É precisamente por isso que o framework existe. Você não está aprendendo que o Anchor é ruim. Você está aprendendo exatamente o que ele custa para ganhar, para você conseguir decidir quando a CU vale o risco.

## O que o derive estava fazendo por você

Aqui está a ressonância que deixa este exercício mais que uma acrobacia. O Anchor V2, o RC em cima do qual você vem construindo o curso todo, o `anchor-next` no `2.0.0-rc.1`, é ele mesmo uma reescrita `no_std` do zero construída em cima do pinocchio: o crate `lang-v2` dele depende do `pinocchio` e do `pinocchio-system 0.6` diretamente. Os crates que você acabou de adicionar na mão são essa mesma fundação, duas linhas de minor atrás (a árvore do V2 roda a linha pinocchio 0.11, onde os tipos centrais já foram renomeados; mais sobre isso na nota de pins no fim). Quando você reconstrói o vault cru, você não está fazendo um brinquedo sem relação. Você está escrevendo na mão exatamente a camada que o framework agora gera.

Por que o V2 foi por esse caminho? Leia a issue #4390 do GitHub, o manifesto de zero-copy que moldou a reescrita. Ela chama o `Account<T>` de hoje de "the slow path" e "the #1 performance complaint from Anchor developers." A tese inteira do V2 é que o wrapper ergonômico de conta faz trabalho de desserialização e de alocação que você muitas vezes não precisa, e que uma fundação mais magra e zero-copy deveria ser o default. Desmontar o framework na mão rima com a decisão de design do próprio V2. Você está seguindo o mesmo raciocínio que os mantenedores seguiram, uma camada abaixo.

Deixe eu ancorar o mecanismo novo naquele que você já conhece. No Anchor, esta era a sua conta de vault:

```rust
#[account(
    mut,
    seeds = [b"vault", authority.address().as_ref()],
    bump = vault.bump,
    constraint = vault.authority == *authority.address(),
)]
pub vault: Account<Vault>,
```

Cada atributo naquela struct é uma checagem que a macro transforma em código de runtime antes do corpo da sua instrução rodar. O `Account<Vault>` sozinho são três checagens: a conta é de propriedade do seu programa, os primeiros 8 bytes dela casam com o discriminator do `Vault`, e os bytes restantes são convertidos num `Vault`. O `seeds` mais o `bump` re-deriva o PDA e confirma o endereço. O hook `constraint` confirma que o campo de authority guardado é igual à conta de authority passada. Isso é bastante segurança empacotada em seis linhas.

(Duas grafias do V2 que valem re-notar, porque são exatamente as que o módulo de migração mapeia: os wrappers largaram os lifetimes `<'info>` deles, e o `.key()` virou `.address()`. A checagem de chave guardada em si costumava ser a keyword `has_one = authority`; o V2 deprecia ela em favor das formas de expressão `address = ...` e `constraint = ...` que você encontrou no módulo 3; a keyword ainda parseia, com um aviso.)

![Uma tabela mapeando cada atributo de conta do Anchor para a checagem explícita de pinocchio que substitui ele e o bug específico que aparece se você omitir aquela checagem.](assets/v01-comparison.png)

Leia a coluna da direita mais uma vez, porque aquelas não são hipóteses: cada uma é uma classe de exploit que drenou programas reais, e cada uma é um `if` só que o Anchor escrevia e você não. Agora você escreve elas.

### O modelo de conta, e por que ele são dois PDAs

O R2 no Anchor era arrumado o bastante para parecer uma coisa só. Nativo, você vê a emenda que o framework estava disfarçando: um vault de custódia de lamports e o registro de autoridade dele são duas contas diferentes, de propriedade de dois programas diferentes, e isso não é uma complicação, é a verdade que o Anchor estava escondendo.

Aqui está o constraint que força isso. O System Program só vai mover lamports para fora de uma conta que ele é dono, e aquela conta tem que ter dado zero. Então a conta que de fato segura o SOL tem que ser de propriedade do System e vazia. Mas você também precisa de algum lugar para guardar o seu próprio estado: o discriminator, a authority, o bump canônico. Isso tem que ser uma conta que o seu programa é dono e consegue escrever. Uma conta não consegue ser as duas. Então o vault é um par.

- O PDA de **config**, seeds `[b"config", authority]`, de propriedade do seu programa. Ele guarda `[discriminator][authority][vault_bump]`. Este é o seu análogo de `Account<Vault>`, a coisa que você valida.
- O PDA de **vault**, seeds `[b"vault", authority]`, de propriedade do System Program, segurando o SOL custodiado. O seu programa nunca escreve o dado dele (não tem nenhum). Ele assina para mover os lamports dele.

![Um diagrama de dois PDAs a partir de uma authority: um config de propriedade do programa segurando estado e um vault de propriedade do System segurando SOL, retirado assinando com invoke_signed.](assets/v02-diagram.png)

Se você está imaginando o R2 como um `Account<Vault>` único que tanto guardava o bump quanto segurava lamports, ele estava fazendo o truque de lamport direto: debitando o saldo da conta própria dele porque o programa era dono dela. Aquilo funciona, mas não é uma transferência assinada, e não é o que a gente quer ensinar aqui. O caminho do invoke_signed, onde um PDA apresenta as seeds dele para autorizar uma transferência de System de verdade, é o padrão para o qual você vai recorrer constantemente (vaults de token, escrows, qualquer coisa em que o PDA tem que ser um signer de CPI). Então a gente constrói a versão que assina.

### O discriminator: um byte, estrutural

O Anchor gasta 8 bytes num discriminator de conta, a tag derivada de SHA-256 que diz "isto é um `Vault`, não um `Config` nem um `Pool`". Nativo, você consegue gastar um. Um `u8` único te dá 255 tipos de conta, que é bastante, e o layout é simples demais: o byte 0 é a tag, o resto é dado.

![Uma faixa de layout de 34 bytes para a conta de config: byte 0 discriminator, bytes 1 até 32 a pubkey de authority, byte 33 o bump de vault guardado.](assets/v03-annotated-code.png)

A regra da casa importa aqui e ela não é opcional: leia e escreva estes campos como slices de array de bytes com lógica de acessor, nunca convertendo o buffer cru para uma struct packed com um ponteiro desalinhado. Um `&*(ptr as *const Config)` numa struct `#[repr(C, packed)]` produz referências desalinhadas, que é comportamento indefinido em Rust e uma das ciladas mais afiadas do ecossistema nativo inteiro. Slices com `copy_from_slice` e `from_le_bytes` são seguros, óbvios, e só um fio mais lentos, então é isso que a gente vai usar por toda parte.

Agora imagine o ataque exato que o discriminator para, porque uma classe de bug que você consegue ver é uma classe de bug que você vai lembrar. Suponha que a Mallory ache alguma outra instrução no seu programa que por acaso cria uma conta de propriedade do programa de 34 bytes para um propósito sem relação, uma linha de placar, digamos, do mesmo tamanho que o seu config. Se o seu withdraw pulasse a checagem de discriminator, ela conseguiria passar aquela linha de placar para dentro do slot ao qual o config pertence. A checagem de owner passaria, porque o seu programa genuinamente é dono da linha. A checagem de tamanho passaria também, porque ela tem 34 bytes. O seu código então leria os bytes 1 até 32 como uma authority e o byte 33 como um bump, a partir de dado que nunca foi um config de vault para começo de conversa. Agora seja honesto sobre o que acontece depois *neste programa em particular*, porque é mais subtil que um dreno. A trava 5 compara aqueles bytes contra a chave que de fato assinou, então uma linha que a Mallory consegue usar tem que carregar a pubkey dela nos bytes 1 até 32 — e então as seeds do withdraw, `[b"vault", her key, bump]`, derivam o vault dela mesma, aquele do qual ela já conseguia retirar legitimamente. As outras travas por acaso contêm o dano aqui. Mas essa contenção é acoplamento, não design: ela vale só porque o esquema de seeds deste programa amarra cada vault à mesma chave que a trava 5 checa. Reuse esta forma de despacho num programa em que o config nomeia um beneficiário, um destino de taxa, ou qualquer authority que não é quem assina, e a linha forjada vira um roubo de verdade. O byte único no offset 0 é o que deixa a questão inteira irrelevante: uma linha de placar carrega uma tag diferente, a checagem falha, e a transação reverte antes de um lamport se mover. Isso é type cosplay, e o discriminator é a checagem de fantasia na porta — a checagem que mantém cada trava mais adiante querendo dizer o que ela diz em vez de se apoiar nas vizinhas. Segure o mapeamento: o discriminator é a tag de tipo da conta, e a checagem de que a tag casa é a linha exata que o Anchor escreve antes de sequer te entregar dado tipado.

### Assinar como um PDA: seeds mais o bump guardado

Um PDA não tem chave privada. Ele "assina" uma CPI apresentando, na hora da chamada, as seeds exatas das quais ele foi derivado mais o bump dele, e o runtime reconstrói o endereço para confirmar que o programa tem permissão de assinar por ele. É esse o truque inteiro, e é a parte que o Anchor monta para você. Seja preciso sobre *quando*, porque é a mesma distinção que o m03-l1 traçou: o Anchor pré-computa o bump canônico em tempo de macro só quando cada seed é um literal de byte. As seeds deste vault carregam a chave da authority, então no lado do Anchor o bump era derivado durante a validação e depois guardado — que é exatamente o byte que você está a ponto de ler de volta na mão.

Nativo, você monta o signer você mesmo. E você usa o bump canônico **guardado**, o que você salvou no init, não um recém-derivado. Re-rodar o `find_program_address` dentro de cada instrução queima mais ou menos 1500 unidades de compute por chamada, porque ele mói candidatos de bump de 255 para baixo procurando o que está fora da curva. Você pagou por isso uma vez no init. Guarde, reuse. O Anchor guarda no account e lê de volta através do `bump = vault.bump`; você faz o mesmo na mão.

![Um fluxograma mostrando o withdraw lendo o bump guardado, montando as seeds, chamando o invoke_signed, o runtime re-derivando o endereço, e executando a transferência só se ele casar com a chave do vault.](assets/v04-flowchart.png)

### A checagem que o V2 adicionou: sem mutáveis duplicados

Tem mais uma checagem na lista, e ela é a mais nova, então vale destacar por conta própria. O Anchor V2 desabilita contas mutáveis duplicadas por padrão, uma trava que as versões mais antigas deixavam por completo para você. Aqui está por que ela ganha o lugar dela. O seu withdraw pega um `vault` gravável e um destino `authority` gravável, e nada do que você escreveu até aqui impede um chamador de passar a mesma conta para os dois. Se o `vault` e o `authority` resolverem para a mesma chave, você está movendo lamports de uma conta para dentro dela mesma, e dependendo da lógica embrulhada em volta de uma transferência assim você pode acabar contando um saldo duas vezes ou passando batido por uma trava que caladamente assumiu que as duas contas eram distintas. A correção nativa é uma comparação de chave que você adiciona na mão, `if self.vault.key() == self.authority.key() { return Err(ProgramError::InvalidArgument); }`, colocada no mesmo corredor de travas do `TryFrom` que as outras cinco. O Anchor V2 escreve aquela trava através da sua struct inteira, caladinho, em tempo de compilação — e a regra dele é mais ampla que a checagem de par feita na mão: uma conta com alias falha na validação no momento em que *qualquer* um dos slots dela é mutável, não só quando dois slots mutáveis colidem. Ela está na lista precisamente porque esquecer dela morde gente suficiente para os mantenedores ligarem ela para todo mundo.

É essa a forma. Visão geral pronta. Agora construa.

## Lab: construa o quarter-vault nativo

Codifique junto daqui. Cada passo é uma edição de verdade; os checkpoints te dizem o que "funcionando" parece.

### Passo 1: desligue o std e monte o despacho

Abra o `src/lib.rs` e substitua ele por inteiro. A primeira linha é a que o Anchor nunca te deixou ver.

```rust
// Not a bare `#![no_std]`. The on-chain build has no std; the host test harness in
// step 5 does, and needs it. `cfg_attr` turns no_std on for every build except the
// test one. Anchor's entrypoint does the same thing behind your back.
#![cfg_attr(not(test), no_std)]

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{CreateAccount, Transfer};

// Generate the keypair, print its pubkey, and paste that string BOTH here and in
// the test's PROGRAM_ID const in step 5. They have to be the same or the test loads
// your program at an address `crate::ID` does not recognize, and gate 2 rejects
// every account with IncorrectProgramId for a reason that has nothing to do with
// your code:
//   mkdir -p target/deploy
//   solana-keygen new --no-bip39-passphrase \
//     -o target/deploy/native_quarter_vault-keypair.json
//   solana address -k target/deploy/native_quarter_vault-keypair.json
pinocchio_pubkey::declare_id!("<paste your generated pubkey>");

/// Account type tag. Anchor spends 8 bytes on this; we spend one.
const VAULT_DISCRIMINATOR: u8 = 1;

/// [disc:1][authority:32][vault_bump:1]
const CONFIG_LEN: usize = 1 + 32 + 1;

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    match data.split_first() {
        Some((0, rest)) => Init::try_from((rest, accounts))?.process(),
        Some((1, rest)) => Withdraw::try_from((rest, accounts))?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
```

O `split_first` descasca o primeiro byte dos dados de instrução. Aquele byte é a **tag de instrução**: `0` é init, `1` é withdraw. Mantenha ele separado na sua cabeça do **discriminator de conta** duas linhas acima dele, o `VAULT_DISCRIMINATOR`, que marca o *tipo* da conta em vez da chamada. O Anchor gasta oito bytes em cada e chama os dois de discriminator; aqui eles têm um byte cada e os dois por acaso são números pequenos, então nomear eles separados é a única coisa mantendo eles separados. Esta é a sua tabela de despacho, a coisa que a macro `#[program]` do Anchor gerava a partir dos nomes das suas funções. Checkpoint: o `cargo build-sbf` **falha**, e deve. O `process_instruction` nomeia o `Init` e o `Withdraw`, que não existem ainda, então você recebe dois erros de `cannot find type in this scope` e nada mais. É esse o estado esperado depois do passo 1; o que você está confirmando é que os erros são aqueles dois e não alguma coisa sobre o `no_std`, o entrypoint, ou um import faltando. Os passos 2 até 4 preenchem aqueles dois nomes, e o primeiro build verde chega no fim do passo 4.

### Passo 2: valide com o TryFrom (esta é a parte trabalhada, leia cada linha)

Esta é a camada que o derive do Anchor substitui, e eu estou mostrando ela por inteiro porque esquecer uma linha aqui é o perigo inteiro. Adicione a struct `Withdraw` e o `TryFrom` dela:

```rust
struct Withdraw<'a> {
    authority: &'a AccountInfo,
    vault: &'a AccountInfo,
    vault_bump: u8,
    amount: u64,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, ProgramError> {
        let [authority, config, vault, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. signer check     (Anchor: Signer)
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 2. owner check      (Anchor: Account<T> proves program ownership)
        if !config.is_owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let cfg = config.try_borrow_data()?;

        // 3. data-length check (Anchor: deserialization would fail on short data)
        if cfg.len() != CONFIG_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        // 4. discriminator check (Anchor: the account type tag)
        if cfg[0] != VAULT_DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        // 5. stored-authority check (Anchor: constraint = vault.authority)
        if &cfg[1..33] != authority.key().as_ref() {
            return Err(ProgramError::InvalidAccountData);
        }
        // 6. duplicate-mutable check (Anchor V2: on by default)
        if vault.key() == authority.key() {
            return Err(ProgramError::InvalidArgument);
        }

        // read the stored canonical bump before the borrow drops
        let vault_bump = cfg[33];

        let amount = u64::from_le_bytes(
            data.try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Withdraw { authority, vault, vault_bump, amount })
    }
}
```

Seis checagens. As primeiras cinco se alinham uma para uma com a tabela de comparação acima, e a sexta é a trava de duplicate-mutable que o V2 liga por padrão, adicionada aqui ao mesmo corredor. O padrão `let [authority, config, vault..] = accounts else` é a sua ordenação de contas, a coisa que o `#[derive(Accounts)]` impunha pela ordem dos campos da struct. Erre a ordem aqui e tudo mais adiante lê a conta errada, que é ela mesma uma cilada que o framework removeu.

![Um fluxograma de falha-rápida de seis travas do TryFrom, cada uma rotulada com o erro que ela retorna e o exploit que ela bloqueia, convergindo numa struct Withdraw validada.](assets/v05-flowchart.png)

Note o que o TryFrom te compra: no momento em que o `process()` roda, a validação está pronta e a lógica de negócio nunca re-checa. Essa separação, validar-depois-agir, é exatamente o que o Anchor te dá separando a struct de accounts do corpo da instrução, e você acabou de construir ela na mão.

Checkpoint: o `cargo build-sbf` ainda falha, e a lista de erros deve ter encolhido em um: o `Withdraw` agora existe, então o que resta é o `Init` faltando mais um `Withdraw::process` não resolvido, que você escreve em seguida. Fique de olho num erro que não está naquela lista: uma reclamação do borrow checker sobre o `cfg` quer dizer que a sua leitura de `let vault_bump = cfg[33];` escapou do escopo onde a trava do `try_borrow_data` está viva. Copie o byte para fora enquanto o borrow está segurado, exatamente onde o comentário coloca ele, em vez de tentar ler através da trava depois de ela cair.

### Passo 3: o corpo do withdraw, e o completion que você preenche

Agora a assinatura. Este é o passo de completion: a trava e as seeds do invoke_signed são as linhas que você monta. Adicione o `impl`:

```rust
impl Withdraw<'_> {
    fn process(&self) -> ProgramResult {
        // --- the guard (you harden this cold in the Challenge) ---
        if self.amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }
        // Bound only. The transfer below does the actual debit; this line exists
        // to turn an over-withdraw into a clean error instead of a failed CPI.
        let _remaining = self
            .vault
            .lamports()
            .checked_sub(self.amount)
            .ok_or(ProgramError::InsufficientFunds)?;

        // --- the completion: assemble the signer from seeds + STORED bump ---
        let bump = [self.vault_bump];
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(&bump),
        ];
        let signer = [Signer::from(&seeds)];

        Transfer {
            from: self.vault,
            to: self.authority,
            lamports: self.amount,
        }
        .invoke_signed(&signer)?;

        Ok(())
    }
}
```

O array de seeds é o conjunto exato do qual o PDA foi derivado: o literal `b"vault"`, os bytes de pubkey da authority, e o bump guardado embrulhado como a própria seed de um byte dele. Aquele último `Seed::from(&bump)` é o slice do bump guardado. Erre este array, em ordem ou em conteúdo, e o `invoke_signed` deriva um endereço diferente, o runtime recusa assinar, e você recebe um `InvalidSeeds`. Não tem crédito parcial do runtime aqui: as seeds estão certas ou a transferência não acontece.

A trava usa o `checked_sub`, nunca o `self.vault.lamports() - self.amount`. Uma subtração ingênua dá underflow e panic (ou pior, dá wrap) num over-withdraw. O `checked_sub` retorna `None`, que você mapeia para um erro limpo. Esta é a checagem de aritmética checada que o Anchor nunca fez você pensar sobre, porque você raramente subtraía saldos de conta direto dentro de um constraint.

Checkpoint: um erro restante, o `Init` faltando, que o passo 4 fornece. Um `InvalidSeeds` neste estágio é impossível, porque nada rodou ainda; um erro de compilação nomeando o `Seed` ou o `Signer` quer dizer que a linha de import do Passo 1 está faltando.

### Passo 4: o init, a criação de conta feita na mão

O init cria a conta de config e guarda os bumps. Esta é a sua CPI de `CreateAccount`, financiando os lamports isentos de aluguel, e ela é assinada pelas seeds próprias do PDA de config porque um PDA tem que autorizar a criação dele mesmo.

```rust
struct Init<'a> {
    authority: &'a AccountInfo,
    config: &'a AccountInfo,
    config_bump: u8,
    vault_bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Init<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, ProgramError> {
        let [authority, config, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let [config_bump, vault_bump] = data else {
            return Err(ProgramError::InvalidInstructionData);
        };
        Ok(Init { authority, config, config_bump: *config_bump, vault_bump: *vault_bump })
    }
}

impl Init<'_> {
    fn process(&self) -> ProgramResult {
        // Blunt reinit guard: rejects ANY lamports at the address, including a
        // stranger's 1-lamport pre-fund (a griefing seam Anchor's init tolerates
        // by funding the shortfall instead -- see the note below the code).
        if self.config.lamports() > 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let bump = [self.config_bump];
        let seeds = [
            Seed::from(b"config"),
            Seed::from(self.authority.key().as_ref()),
            Seed::from(&bump),
        ];
        let signer = [Signer::from(&seeds)];

        let rent = Rent::get()?;
        CreateAccount {
            from: self.authority,
            to: self.config,
            lamports: rent.minimum_balance(CONFIG_LEN),
            space: CONFIG_LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer)?;

        // write [disc][authority][vault_bump] as bytes, no packed-struct cast
        let mut cfg = self.config.try_borrow_mut_data()?;
        cfg[0] = VAULT_DISCRIMINATOR;
        cfg[1..33].copy_from_slice(self.authority.key().as_ref());
        cfg[33] = self.vault_bump;
        Ok(())
    }
}
```

Aquela primeira trava está fazendo um trabalho que o framework fazia com mais cuidado, e a diferença vale ser nomeada com precisão. O `init` simples do Anchor se recusa a rodar numa conta que já está *inicializada* — dado alocado, ou um owner estranho — mas ele deliberadamente aceita uma meramente pré-financiada e não alocada: o código gerado dele completa a diferença de aluguel, aloca e atribui, precisamente para um estranho jogando um lamport por airdrop no endereço do seu PDA não conseguir inutilizar o seu init para sempre. A trava rude de `lamports() > 0` acima compra proteção de reinit ao custo de reabrir exatamente aquela emenda de griefing. Para um vault didático a troca é aceitável, dita em voz alta: o sinal no qual o Anchor de fato se chaveia é alocação e propriedade, não saldo, e um init nativo endurecido checa `data_len() == 0` mais propriedade do System e *financia a diferença* em vez de recusar. O `init_if_needed` é uma cilada completamente diferente — ele caladamente permite re-inicialização de estado vivo, que é por que as regras da casa proíbem ele. Nativo, cada uma dessas distinções é um `if`, e cada uma é sua para lembrar.

Os bumps chegam nos dados de instrução, computados no lado do cliente pelo `find_program_address`. Isso mantém a derivação caríssima fora da cadeia e fora do seu orçamento de compute, e é por isso que o cliente, não o programa, faz a moagem. Checkpoint: o `cargo build-sbf` compila limpo agora, primeiro build verde da lição, e o `target/deploy/native_quarter_vault.so` existe. Se não tem `.so`, a linha de `crate-type` do passo de scaffold está faltando.

### Passo 5: a trava de aceitação

Agora a mesma *barra* de aceitação que o R2 passou, através de uma bancada nova. Não o mesmo arquivo: o teste do R2 montava dados de instrução e structs de conta gerados pelo Anchor, e nenhum dos dois existe aqui, então as asserções passam adiante e a bancada é escrita do zero. O LiteSVM é a mesma VM da Solana in-process que você vem usando desde o módulo 2: sem validador, sem ledger, só o seu programa carregado num bank que você dirige a partir do Rust. Você adicionou ele com o `cargo add --dev litesvm@0.16 solana-sdk@4` no começo. Crie o `tests/withdraw.rs`:

```rust
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

// The SAME string you pasted into declare_id! in step 1.
const PROGRAM_ID: Pubkey = pubkey!("<paste your generated pubkey>");

#[test]
fn withdraw_signed() {
    let mut svm = LiteSVM::new();
    let program_so = concat!(env!("CARGO_MANIFEST_DIR"), "/target/deploy/native_quarter_vault.so");
    svm.add_program_from_file(PROGRAM_ID, program_so).unwrap();

    let authority = Keypair::new();
    svm.airdrop(&authority.pubkey(), 10_000_000_000).unwrap();

    let (config, config_bump) =
        Pubkey::find_program_address(&[b"config", authority.pubkey().as_ref()], &PROGRAM_ID);
    let (vault, vault_bump) =
        Pubkey::find_program_address(&[b"vault", authority.pubkey().as_ref()], &PROGRAM_ID);

    // init
    let init_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: vec![0, config_bump, vault_bump],
    };
    let bh = svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[init_ix], Some(&authority.pubkey()), &[&authority], bh);
    svm.send_transaction(tx).unwrap();

    // fund the SOL vault PDA (this teaching build has no deposit instruction)
    svm.airdrop(&vault, 2_000_000_000).unwrap();

    // a valid PDA-signed withdraw succeeds
    let mut data = vec![1u8];
    data.extend_from_slice(&500_000_000u64.to_le_bytes());
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    };
    let before = svm.get_balance(&vault).unwrap();
    let bh = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority.pubkey()), &[&authority], bh);
    svm.send_transaction(tx).unwrap();
    assert_eq!(before - svm.get_balance(&vault).unwrap(), 500_000_000);

    // an over-withdraw is rejected
    let mut data = vec![1u8];
    data.extend_from_slice(&999_000_000_000u64.to_le_bytes());
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    };
    let bh = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority.pubkey()), &[&authority], bh);
    assert!(svm.send_transaction(tx).is_err());
}
```

Faça o build do programa para um `.so`, e depois rode o teste:

```bash
cargo build-sbf
cargo test -p native-quarter-vault
```

Você quer ver:

```
test withdraw_signed ... ok
```

É essa a mesma trava. Lamports saíram do PDA sob autoridade do programa, e o over-withdraw foi rejeitado. O seu vault nativo faz exatamente o que o vault de framework fazia. Sente com isso por um segundo: sem `#[account]`, sem `#[program]`, sem mágica de `declare_id!` além de um const, e o teste de aceitação não sabe a diferença.

![Uma linha do tempo indo da reclamação de zero-copy da #4390, até o pinocchio como uma fundação magra, até a reescrita no_std do Anchor V2, até a reconstrução na mão desta lição.](assets/v06-timeline.png)

Antes de você sair do Lab, olhe de volta o trade-off no modelo de conta. Dois PDAs e mais ou menos oitenta linhas compraram para você o que seis atributos do Anchor davam de graça, mais o compute que você economizou por não desserializar através do `Account<T>`. É esse o acordo que o nativo oferece, e ele é um acordo de verdade, não uma bronca. O próximo visual é o de dar print, porque ele é a resposta para "quando isso vale a pena".

![Uma tabela de decisão comparando o Anchor V2 e o pinocchio nativo através de checagens, compute, auditabilidade, modo de falha, e quando escolher cada um.](assets/v07-table.png)

## Challenge: a trava de withdraw do vault nativo

Agora você vai solo. Sem resposta na página.

No Lab, a trava pegava carona dentro do `process()`. O Challenge faz você escrever ela no frio, como uma função pura autônoma com códigos de retorno explícitos, do jeito que um fuzzer ou um teste de unidade cutucaria ela. Esta é a lógica exata que fica entre o seu vault e um underflow, extraída para você conseguir ver ela com clareza.

O starter, em `lessons/m09-l1/native-vault-withdraw/starter.rs`:

```rust
/// Compute the vault's remaining balance after a withdraw, or an error code.
///
/// Contract (an i128 so the harness can signal failure without a Result):
///   -2  : the withdraw amount is zero (invalid input)
///   -1  : the withdraw exceeds the balance (over-withdraw)
///   >=0 : the remaining balance after a valid withdraw
///
/// The starter skips BOTH guards and subtracts naively. It happens to return the
/// right number for a normal withdraw, and is wrong (and unsafe) for both rejects.
const fn vault_withdraw(balance: u64, amount: u64) -> i128 {
    // TODO: reject a zero-amount withdraw with -2
    // TODO: reject an over-withdraw with -1 using balance.checked_sub(amount)
    (balance as i128) - (amount as i128)
}
```

O `const fn` está fazendo trabalho sem framework: o arquivo avaliado carrega asserções de tempo de compilação embaixo da função — o dispositivo do m03-l3, e o que avaliação só-de-compilação de fato impõe — então o starter sem trava não compila, e cada asserção que falha nomeia o caso de rejeição pelo qual ela está de pé. Apropriado, para a lição em que você escreve cada checagem na mão: aqui até a bancada não é nada além do compilador.

Critérios de aceitação:

- um withdraw de quantia zero retorna `-2`, e ele é rejeitado antes da checagem de saldo
- um over-withdraw retorna `-1`, computado via `checked_sub`, nunca uma subtração ingênua — incluindo `(100, 102)`, onde um `balance as i128 - amount as i128` ingênuo produz `-2`, que é o código de *quantia zero*. As sentinelas são decisões, não aritmética; se a sua subtração consegue produzir uma, você não tem uma trava
- retirar de um vault *vazio* é um over-withdraw, `-1`, não uma rejeição de quantia zero: a trava de zero é na quantia, nunca no saldo
- um withdraw válido retorna o saldo restante, através da faixa inteira de `u64` — `(u64::MAX, 1)` retorna `u64::MAX - 1`, que é por que o retorno é `i128` e por que fazer a subtração em `i64` falha
- drenar até exatamente zero é permitido (o PDA de SOL deste vault didático não carrega dado nenhum e o challenge é deliberadamente agnóstico de aluguel; um withdraw de produção também teria um piso no mínimo isento de aluguel, que é o que a trava própria do R2 fazia)

Duas dicas, e depois é seu. Primeira: um withdraw de quantia zero é entrada inválida, então rejeite ele antes de você tocar no saldo. Segunda: o `balance.checked_sub(amount)` retorna `None` exatamente quando o withdraw excede o saldo, que é o seu sinal de over-withdraw. Compile até as asserções pararem de disparar, e depois rode os vetores do `tests.json` até cada caso ficar verde.

Se você errar a ordem, olhe qual asserção falha. Uma quantia zero que retorna `0` em vez de `-2` quer dizer que você subtraiu antes de rejeitar. Esse bug de ordem é o mesmo que é entregue em programas reais, então aprender a ver ele aqui é o ponto.

## Onde você aterrissou, e o que vem

Você acabou de deletar o framework e o vault continua funcionando. Você escreveu as seis checagens que o derive escrevia para você, na ordem que importa, e você agora consegue apontar para cada linha e nomear o exploit que ela bloqueia. Você assinou uma transferência de lamports para fora de um PDA com seeds que você montou na mão e um bump que você guardou em vez de re-derivar. E você passou a mesma trava de LiteSVM em pinocchio cru que você passou no Anchor V2. É essa a fundação em cima da qual o framework é construído, nas suas próprias mãos.

Uma nota honesta sobre pins, e ela é a lição de disciplina de versão mais afiada deste módulo, então leia em vez de passar o olho. Tudo acima é verificado contra `pinocchio 0.9` / `pinocchio-system 0.4` / `pinocchio-pubkey 0.3`, Rust `1.89.0`, e Anchor V2 `2.0.0-rc.1`, checado em 2026-08-22. Aqueles três crates de pinocchio têm que se mover como um conjunto: o `pinocchio-system 0.4` declara `pinocchio ^0.9`, e o `pinocchio-pubkey 0.3` também. Misture um minor tipo-major e o cargo vai felizmente resolver duas cópias de `pinocchio` para dentro da sua árvore, momento em que o `AccountInfo` que o seu entrypoint te entrega é um tipo diferente daquele que o `Transfer` quer, e a mensagem de erro vai ser sobre trait bounds em vez de sobre versões.

O crate é pré-1.0 e a API está genuinamente mudando, e ela já mudou uma vez para além de onde você está de pé. O rename não é um rumor sobre uma branch main: **ele foi entregue**. Na linha `pinocchio 0.11` (0.11.0, 2026-04-08), o `AccountInfo` virou `AccountView`, o `Pubkey` virou `Address` (do `solana-address`), o `is_owned_by` virou `owned_by`, o `try_borrow_data`/`try_borrow_mut_data` virou `try_borrow`/`try_borrow_mut`, e o `Seed`/`Signer` se moveram de `instruction` para dentro de `cpi`. Aquela linha 0.11 é exatamente em cima do que o próprio `lang-v2` do Anchor V2 fica, via `pinocchio-system 0.6`, que é por que o código V2 que você vem escrevendo o curso todo diz `.address()` e não `.key()`. Este lab fixa a linha 0.9 deliberadamente, porque ela é a última linha em que o `pinocchio-pubkey` ainda resolve e em que os nomes pré-rename deixam o mapeamento um-para-um com os seus instintos antigos da era Anchor V1 legível. Porte para o 0.11 como exercício e você vai sentir o rename duas vezes: uma nos nomes de tipo, uma no `try_borrow_mut` pegando `&mut self`, que é a API te dizendo que ela parou de fingir que mutabilidade interior era de graça.

Então: fixe exatamente, mova os três como um conjunto, e releia a nota de pins antes de você subir. Acompanhar um latest flutuante num crate pré-1.0 é como você descobre do jeito difícil que um nome de tipo é uma versão.

O seu vault nativo funciona. Mas quais das suas linhas escritas na mão o Anchor gerava de graça, e quais ele deixava impossíveis de errar para começo de conversa? Na próxima lição a gente coloca o seu `lib.rs` lado a lado com a expansão de macro de verdade e lê o diff linha por linha. Você vai descobrir exatamente quão perto a sua reconstrução na mão chegou, e onde o framework caladamente te protege de jeitos que você não replicou.
