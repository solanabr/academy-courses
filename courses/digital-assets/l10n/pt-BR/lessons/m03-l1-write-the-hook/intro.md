# Escreva o hook: a interface + o seu único programa Rust

## Resumo

A lição passada terminou o SPROUT: taxa de transferência, harvest de taxa retida e o TLV de metadados nativo, todos ligados a um mint só, o degrau R3 da escada de artefatos deste curso. Toda extensão até aqui remodela o token passivamente. Taxas acumulam. Metadados ficam parados no TLV e esperam ser lidos. Hoje você conhece a única extensão que roda o SEU código a cada movimento do token, e você escreve esse código: `harvest-hook`, um programinha Anchor que recusa qualquer destino que não esteja na allowlist dele e, depois que você terminar o challenge solo, registra cada transferência num livro-razão de tesouraria. (O nome é sobre o que ele protege, os fluxos de harvest do Overgrowth; o harvest das taxas em si continua com a sequência de harvest do m02-l1, porque um hook nunca consegue mover fundos.) Você é o autor da interface de transfer hook de três instruções, inicializa a conta de validação que diz ao runtime quais contas extras encaminhar, cunha uma variante nova do SPROUT com hook, e prova ela numa bancada LiteSVM onde uma transferência aterrissa e uma reverte com o seu próprio erro. Checagem do recuo: a interface, o PDA de validação e a lista de account-meta são construídos junto com você; a cancela de allowlist dentro do `Execute` é um TODO que você preenche; o log da tesouraria é totalmente solo.

Antes de qualquer teoria, coloque as três strings de bytes na sua tela. A interface nomeia as instruções dela hasheando strings, então um shell deriva todas as três:

```bash
# macOS ships shasum, most Linux boxes ship sha256sum; take whichever is there.
sha256() { if command -v shasum >/dev/null; then shasum -a 256; else sha256sum; fi; }

for s in execute initialize-extra-account-metas update-extra-account-metas; do
  printf '%-32s ' "$s"
  printf '%s' "spl-transfer-hook-interface:$s" | sha256 | head -c 16
  echo
done
```

Você deve ver exatamente isto:

```
execute                          692565c54bfb661a
initialize-extra-account-metas   2b220d31a758ebeb
update-extra-account-metas       9d692a926655f1ae
```

Esse é o contrato inteiro que você está prestes a implementar. Não é um arquivo ABI, não é um registro, não é uma interface que você herda: três prefixos de oito bytes derivados de três strings ASCII, e qualquer programa em Solana que responda a eles é um transfer hook. Mantenha o `692565c54bfb661a` à vista pelo resto da lição. Ele é a campainha que o Token-2022 toca no seu programa, e no fim você vai ter visto ela tocar num log de verdade.

Você já leu uma extensão TransferHook que apontava para nada: a do PYUSD, cuja linha `transferHook.programId: null` você imprimiu pela primeira vez lá atrás no script de abertura do m01-l1, configurada no nascimento do mint com um program id nulo. Vale dar uma segunda olhada nesse formato antes de você construir, porque ele reconcilia dois fatos que soam contraditórios. A extensão em si é de tempo de criação: ou um mint carrega o slot de hook desde o primeiro bloco dele, ou nunca vai carregar. O program id dentro do slot não é: a autoridade do hook consegue mirar ele, ou anular ele, depois. Então um slot dormente é uma arma carregada com a câmara vazia, o slot é a arma e o program id é o cartucho, e um emissor como a Paxos instala a arma vazia de propósito. No momento em que um programa É colocado na câmara, o Token-2022 deixa de ser uma biblioteca e vira um chamador: ele faz CPI naquele programa em toda transferência, para sempre, e se o programa devolver um erro a transferência morre junto.

## A interface, o PDA de validação e os 35 bytes

### Três strings, hasheadas em oito bytes

Comece pela restrição que espreme o design inteiro. O Token-2022 tem que invocar um programa que ele nunca viu, escrito por alguém que ele nunca conheceu, sem um crate compartilhado, um registro ou uma negociação de versão. A única coisa em que os dois lados conseguem concordar de antemão é um nome. Então a interface hasheia nomes: `sha256("spl-transfer-hook-interface:execute")[0..8]` prefixa os dados da instrução, o seu programa dá match nesses oito bytes, e o dispatch funciona. É isso que um discriminador de string hasheada é, e é o mesmo truque que o Anchor faz com `sha256("global:<method_name>")[0..8]` para as instruções dele. Duas convenções independentes, um mecanismo, e daqui a um minuto você vai ter um programa que responde às duas.

As três instruções se dividem de forma limpa por quem chama elas:

![Comparação das três instruções da interface de transfer hook, mostrando qual delas o Token-2022 chama em toda transferência e quais duas o emissor chama.](assets/v01-comparison.webp)

Repare na assimetria. Duas das três instruções são chamadas de gestão comuns que um emissor roda a partir de um script, num dia bom duas vezes na vida de um mint. A terceira roda no caminho quente de toda transferência que algum dia encostar no token, e ela é a única cujo custo outra pessoa paga. Essa assimetria é o argumento ético inteiro sobre hooks, e a gente vai voltar nela com números.

O `Execute` recebe o valor da transferência como dados de instrução e um prefixo fixo de contas: a origem no índice 0, o mint no 1, o destino no 2, o dono ou delegado da origem no 3, depois a conta de validação, depois toda conta extra que o runtime resolveu no seu lugar. Segure esses índices. Eles não são documentação, são posições endereçáveis sobre as quais a codificação de account-meta faz hash.

Vale ser preciso sobre o que registro significa aqui, porque é menos do que as pessoas esperam. Não existe registro global de hooks. O Token-2022 não mantém uma tabela de programas aprovados, não existe allowlist para entrar, e nada valida que o program id na extensão TransferHook de um mint seja sequer um programa. A extensão do próprio mint nomeando o seu program id é a fiação inteira. O que também significa que o modo de falha quando o seu programa não responde à interface não é um erro prestativo: o Token-2022 entrega ao seu programa oito bytes que ele não reconhece, o dispatch cai para fora do fim do seu match, e você recebe um erro de fallback de um programa que parece nunca ter sido chamado. Se você algum dia vir uma transferência com hook morrer dentro do seu próprio programa sem nada no log além de uma reclamação de fallback, você tem um problema de discriminador, não um problema de lógica.

![A tabela de dispatch de um programa segurando três discriminadores do namespace do Anchor e três do namespace da interface, com prefixos sem match caindo para um erro de fallback.](assets/v02-diagram.webp)

### A conta de validação mora no SEU programa

Aqui está a peça que derruba quase todo mundo na primeira vez, inclusive eu na primeira vez em que liguei um desses. O hook precisa de contas extras. O Token-2022 não tem como saber quais, porque o seu programa é arbitrário. Programas Solana não conseguem buscar contas em tempo de execução, então ninguém rio abaixo consegue descobrir elas também. A resposta da interface é um manifesto publicado: uma conta on-chain, por mint, cujos dados listam exatamente de quais contas extras uma chamada de `Execute` precisa e como derivar cada uma.

Essa conta é o `ExtraAccountMetaList`, ela fica num PDA semeado pelo literal `extra-account-metas` e pelo mint, e ela pertence ao SEU programa de hook. Não ao mint. Não ao Token-2022.

![A extensão TransferHook do mint aponta para o programa de hook, e o PDA de validação pendura naquele programa de hook em vez de pendurar no mint ou no Token-2022.](assets/v03-diagram.webp)

A implementação de referência te dá a derivação como uma função, `get_extra_account_metas_address(&mint, &program_id)`, e o argumento `program_id` é o que as pessoas preenchem errado. Passe o Token-2022 ali e você recebe um endereço perfeitamente válido que nenhuma conta vai ocupar jamais, então toda transferência falha na resolução com um erro que não diz nada sobre o erro de verdade. Se você tirar uma regra de derivação desta lição, tire esta: o manifesto pertence ao programa que precisa das contas, porque ele é a única parte que sabe quais elas são.

Por que um por mint em vez de um por conta de token? Porque as necessidades são uma propriedade da política, não do holder. Um hook que restringe o acesso com base numa allowlist precisa da conta de allowlist quer o destino seja uma baleia ou uma carteira nova. Um por mint mantém a lista pequena, mantém as atualizações atômicas, e mantém o caminho de leitura do cliente em exatamente uma conta.

### 35 bytes para uma conta que ninguém criou ainda

Cada entrada nessa lista é um `ExtraAccountMeta`, uma struct fixa de 35 bytes. O tamanho fixo está fazendo trabalho de verdade aqui: um cliente consegue caminhar pela lista sem um schema, e o lado on-chain consegue indexar ela sem alocar. Um byte de discriminador seleciona como o endereço é encontrado, trinta e dois bytes carregam ou o endereço em si ou um conjunto compactado de configurações de seed, e dois bytes carregam as flags de signer e de gravável.

O caso interessante é o que o seu hook usa. As duas contas de que o `harvest-hook` precisa são PDAs do programa de hook derivados do mint, e o mint não é conhecido quando você escreve a lista, ele é conhecido quando a transferência acontece. Então, em vez de um endereço, a entrada guarda uma receita: um seed literal, depois "a chave da conta no índice 1 da lista de contas do Execute," que é o mint.

![As duas entradas de account-meta do hook codificam uma receita de PDA, um seed literal mais a chave do índice 1 de conta do Execute, o mint, com a entrada de tesouraria gravável.](assets/v04-annotated-code.webp)

Duas consequências decorrem dos seeds baseados em índice, e as duas mordem em produção. Primeiro, a resolução é posicional: se um cliente resolver a lista fora de ordem ou deixar cair uma entrada, todo seed baseado em índice depois dela deriva um endereço diferente, silenciosamente, e a transferência reverte com uma divergência que não aponta para nada útil. Segundo, a flag de gravável na entrada é da própria entrada, não herdada da transferência, que é por isso que o seu log de tesouraria pode ser escrito mesmo com tudo que chega da transferência sendo somente leitura. A gente prova essa afirmação de somente leitura na próxima lição, a partir do próprio builder de instrução do crate da interface; hoje, aceite ela como o motivo de o design ser seguro o bastante para entregar.

### O que o hook consegue fazer, e o que ele custa para todo mundo

Antes do preço, o limite, porque é ele que torna o preço pagável. O seu hook recebe a conta de origem, a conta de destino e o dono. Ele não consegue gastar de nenhuma delas. Não porque ele recusa educadamente, mas porque o Token-2022 despe essas contas antes da CPI: elas chegam somente leitura e não signatárias, então uma escrita é um erro de runtime e uma assinatura não fica disponível para reutilizar. O inventário completo de poderes do hook tem três itens. Ele consegue ler qualquer coisa nas contas que recebeu. Ele consegue escrever nos extras declarados por ele, que é por isso que o log da tesouraria funciona. E ele consegue devolver um erro, o que aborta a transferência inteira atomicamente.

Derive essa restrição em vez de decorar ela, porque o raciocínio generaliza. Um hook é código escolhido pelo emissor, rodando dentro de uma transação de autoria de outra pessoa, com as contas dessa pessoa em escopo. Se o hook pudesse assinar ou escrever nessas contas, então segurar um token significaria conceder ao emissor permissão permanente para mover os seus outros saldos no meio da transferência, e toda integração teria que auditar um programa arbitrário antes de cotar um swap. A única versão desta feature que consegue existir sem essa auditoria é um assento de veto: o hook vê tudo e não toca em nada. É por isso que "um hook consegue me drenar?" tem uma resposta de uma linha, e é o formato que você deve procurar em qualquer design parecido com hook. Observe, registre no seu próprio terreno, recuse. As quatro linhas do crate da interface que definem essas flags de conta são lidas linha a linha na próxima lição.

A troca, então, é nítida e vale ser precificada antes de você escrever uma linha. O que o emissor compra é real: controle por transferência, expressável como lógica de programa arbitrária, imposto pelo próprio programa de token em vez de por um app que um usuário consegue contornar. O que todo mundo paga também é real, e vem em duas moedas.

A primeira é compute. Eu medi esta bancada nos dois caminhos. Um `TransferChecked` simples do Token-2022 num mint sem extensões queimou 1,790 CU. A mesma transferência através do mint com hook aterrissou entre uns 23,000 e 35,000 CU entre as rodadas, com o `Execute` do próprio hook respondendo por uns 9,400 a 13,500 disso. Dez a vinte vezes o custo da transferência que ele está protegendo, para um hook cuja lógica inteira é uma varredura booleana de um array de oito entradas.

![Um TransferChecked simples custa 1,790 compute units contra 23,108 a 35,292 do que tem hook, dos quais o Execute é 9,448 a 13,448.](assets/v05-chart.webp)

Vale pausar nessa dispersão, porque ela é uma lição em si. A variância não é a varredura da allowlist, que não custa nada. É o `find_program_address`: toda constraint de seeds que não carrega um bump armazenado caminha pela busca, e cada iteração custa compute de verdade. Guardar bumps canônicos é a correção padrão e o curso Master Anchor V2 cobre isso como um padrão de framework. Eu estou deixando um bump não armazenado neste programa de propósito para a variância aparecer nos seus próprios logs.

A segunda moeda é coordenação, e ela é a cara. Como as contas extras têm que estar na transação antes de ela ser enviada, toda carteira, toda DEX, toda integração de pagamento que algum dia encostar no seu token tem que buscar a sua conta de validação, decodificar aquelas entradas de 35 bytes, resolver cada uma e acrescentar elas em ordem. Para sempre. Um hook não gasta só o compute do emissor; ele empurra uma obrigação permanente de encaminhamento para estranhos que nunca concordaram com ela. Esse é o fato com que a próxima lição abre, e é por isso que uma fatia séria do ecossistema simplesmente recusa tokens com hook.

![Antes de toda transferência de um token com hook um cliente precisa buscar a conta de validação, decodificar as entradas dela, resolver cada uma e acrescentar elas em ordem.](assets/v06-flowchart.webp)

Dois recibos para colocar a feature no mundo real antes de a gente construir. O PYUSD, o lançamento emblemático do Token-2022 em Solana em maio de 2024 da PayPal e da Paxos, já vem com um conjunto de oito extensões TLV com cara de compliance, e uma delas é um transferHook cujo `programId` é nulo. Configurado, dormente, reservado. Os emissores recorrem a este slot no momento em que compliance entra na mesa, mesmo quando eles não estão prontos para usar ele. E o estado do material oficial, para você saber o que existe antes de a gente construir: o solana.com hospeda um guia de transfer hook — um walkthrough de Anchor com build, deploy e testes — mais um guia de integração para o caminho de envio do cliente, o solana-program.com documenta a interface ao lado de uma implementação de referência, e o solana-developers/program-examples carrega exemplos de transfer hook. Walkthroughs para copiar existem, em outras palavras. O que esta lição acrescenta é a parte que copiar não te dá: versões fixadas, compute medido, um hook ligado ao token SPROUT que você carregou por dois módulos, e uma cancela que você mesmo escreve e defende contra uma suíte de testes vermelha.

## Lab: de crate vazio a uma transferência revertida

O plano: um crate, um programa, um arquivo de teste. Você vai construir o hook, inicializar o manifesto dele, cunhar uma variante do SPROUT com hook dentro da bancada, e conduzir duas transferências por ele. Tudo neste lab foi construído e rodado nesta máquina em 2026-08-22 com os pins exatos abaixo, incluindo os números de compute que você acabou de ler.

Uma nota de escopo antes do primeiro comando, porque ela muda como você deve ler o código. Este lab usa Anchor e não ensina Anchor. A camada de framework, as macros, as constraints de conta, a mecânica de CPI, os padrões de teste, pertence ao curso Master Anchor V2 (`mastering-anchor-v2`), e se um bloco `#[derive(Accounts)]` aqui te der vontade de uma explicação mais completa do que o sistema de constraints está fazendo, aquele é o curso para levar isso — o módulo de anatomia de abertura dele desmonta exatamente essa sintaxe. Só respeite a barra declarada do curso: ele assume que você já entrega programas Anchor, então trate esse módulo como a referência que você consulta a partir daqui, não como uma primeira aula de Anchor. O que você está aprendendo aqui é a interface e a extensão, não o framework. O programa tem umas quarenta linhas de lógica vestindo um casaco fino de Anchor, e tudo que é específico de hooks ficaria igual em Rust puro com mais cerimônia.

A escada de autonomia do lab, dita sem rodeios para você saber quando está por sua conta: os passos 1 a 6 são trabalhados junto com você, o passo 7 deixa a função `gate` deliberadamente vazia para você escrever, o passo 9 é onde a suíte de testes fica vermelha contra essa cancela vazia, e o Challenge não tem apoio.

**1. Pegue a toolchain de build.** Você precisa de Rust e do compilador SBF da toolchain de Solana. Nenhuma toolchain de Rust ainda? O curso Rust & TypeScript Fundamentals instala ela do zero no m04-l1 dele, e o módulo 4 dele de forma mais ampla é de onde vem a fluência de leitura de Rust em que esta lição se apoia. Se o `cargo-build-sbf` ainda não estiver no seu path de trabalhos anteriores:

```bash
# Rust, if you do not have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# The Agave toolchain, which brings cargo-build-sbf and the solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

cargo-build-sbf --version
```

Você não precisa da CLI `anchor` para este lab e ela não está na lista de instalação acima. Se você já tem ela via avm de outros trabalhos, `anchor build` produz o mesmo `.so`; tudo aqui usa `cargo build-sbf` direto para o crate continuar um pacote Cargo comum, sem scaffolding de workspace em volta dele.

**2. Crie o crate.** Um diretório, um pacote. Chame ele de `hook/` ao lado da pasta `labs/` que você vem usando.

```bash
mkdir -p hook/src hook/tests && cd hook
```

O `hook/Cargo.toml`, na íntegra:

```toml
[package]
name = "harvest-hook"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "harvest_hook"

[dependencies]
anchor-lang = "=1.1.2"
spl-tlv-account-resolution = "=0.11.1"
spl-transfer-hook-interface = "=2.1.0"
spl-discriminator = "0.5"

[workspace]

[dev-dependencies]
litesvm = "=0.15.2"
solana-program-runtime = "=4.1.1"
solana-builtins = "=4.1.1"
solana-instruction = "3"
solana-keypair = "3"
solana-signer = "3"
solana-system-interface = "2"
solana-transaction = "4"
spl-token-2022 = "=11.0.0"
```

Quatro notas sobre esses pins, todas elas checadas em 2026-08-22 em vez de lembradas, com uma re-checagem datada de 2026-09-01 onde marcado.

O `anchor-lang` está fixado em 1.1.2, que é contra o que o código deste lab foi verificado e NÃO é mais o release mais novo da linha V1: a 1.2.0 foi entregue em 2026-09-04, depois de esta lição ser escrita; a re-checagem do stamp em 2026-09-05 viu essa versão e manteve 1.1.2 mesmo assim, porque um pin registra contra o que o código foi verificado, não o que é mais novo. O pin continua valendo, porque um pin é uma declaração sobre o que foi testado e não sobre o que é mais novo, mas não repita "1.1.2 é a atual" para ninguém. Uma 2.0.0-rc.1 também existe e ela é assunto do curso Master Anchor V2, não nosso; a V2 muda a superfície de tipos de conta o bastante para este arquivo não compilar contra ela sem mudanças. O `spl-transfer-hook-interface` 2.1.0 é o release atual da interface. O `spl-tlv-account-resolution` está fixado em 0.11.1, a versão contra a qual as afirmações de nível de byte deste lab foram verificadas; a crates.io entregou desde então a 0.11.2/0.11.3 (re-checadas em 2026-09-01), que retrabalham as entranhas do resolvedor sem encostar no formato serializado. A tabela `[workspace]` vazia não é decoração: ela impede que o workspace de um diretório pai adote este crate e arraste resolução de dependências incompatível.

Os dois pins de dev-dependency de aparência estranha são do tipo feio honesto. O LiteSVM 0.15.2 não compila contra a linha 4.2 dos crates de runtime do Agave que o Cargo escolheria para ele, então `solana-program-runtime` e `solana-builtins` estão fixados de volta em 4.1.1 para segurar o resolvedor numa versão contra a qual o LiteSVM foi construído. Se um release futuro do LiteSVM consertar isso, apague as duas linhas. Este é exatamente o tipo de coisa que apodrece, então cheque isso em vez de confiar num curso.

**3. Instale a bancada, e conheça ela.** O LiteSVM é a dev-dependency que você acabou de adicionar, então ele já está instalado. Vale dez segundos sobre o que ele é, porque ele é novo neste curso: o LiteSVM é uma VM de Solana in-process. Ele te dá um runtime SBF de verdade com programas de verdade e medição de compute de verdade, numa biblioteca, sem processo de validador, sem RPC e sem ledger. Os testes rodam em milissegundos em vez de dezenas de segundos. Ele também já vem com os programas SPL, que é por isso que a bancada consegue criar um mint Token-2022 sem você fazer deploy de nada. A troca é que ele não é um cluster: sem cronograma de líder, sem mercado de taxas, sem rede. Para um hook, que é lógica pura em nível de instrução, esse é o instrumento certo.

**4. Escreva o estado e os erros.** Crie o `hook/src/lib.rs` e comece com os imports, o program id, os seeds e as contas que o seu hook possui:

```rust
use anchor_lang::prelude::*;
use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::{
    ExecuteInstruction, InitializeExtraAccountMetaListInstruction,
    UpdateExtraAccountMetaListInstruction,
};

declare_id!("HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ");

pub const CONFIG_SEED: &[u8] = b"hook-config";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const META_LIST_SEED: &[u8] = b"extra-account-metas";
pub const MAX_ALLOWED: usize = 8;

pub const TOKEN_2022_ID: Pubkey =
    Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

#[account]
#[derive(InitSpace)]
pub struct HookConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub paused: bool,
    pub bump: u8,
    pub allowed_len: u8,
    pub allowed: [Pubkey; MAX_ALLOWED],
}

#[account]
#[derive(InitSpace)]
pub struct TreasuryLog {
    pub mint: Pubkey,
    pub bump: u8,
    pub transfers: u64,
    pub total_amount: u128,
    pub last_amount: u64,
    pub last_destination: Pubkey,
}

#[error_code]
pub enum HarvestHookError {
    #[msg("Transfers are paused by the hook authority")]
    Paused,
    #[msg("Destination is not on the hook allowlist")]
    NotAllowed,
    #[msg("The allowlist is full")]
    AllowlistFull,
    #[msg("Only the hook authority may call this")]
    Unauthorized,
    #[msg("Source account is not owned by Token-2022")]
    NotTokenAccount,
}
```

Um array fixo de oito slots em vez de um `Vec` é uma escolha deliberada: o `Execute` roda em toda transferência, e um layout fixo significa nenhuma realocação, nenhuma surpresa de comprimento, e uma varredura cujo custo você consegue raciocinar. Um emissor de verdade com milhares de destinos na allowlist usaria um PDA por destino e deixaria a lista de metas derivar ele a partir da chave de destino no índice 2, o que a codificação suporta. Oito slots mantêm a contagem de contas da lição honesta.

**5. Publique o manifesto.** Esta é a função que escreve aquelas duas entradas de 35 bytes. Acrescente ao `lib.rs`:

```rust
fn harvest_metas() -> Result<Vec<ExtraAccountMeta>> {
    Ok(vec![
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: CONFIG_SEED.to_vec() },
                Seed::AccountKey { index: 1 },
            ],
            false,
            false,
        )?,
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: TREASURY_SEED.to_vec() },
                Seed::AccountKey { index: 1 },
            ],
            false,
            true,
        )?,
    ])
}
```

Leia os dois argumentos booleanos como o que eles são: `is_signer` e depois `is_writable`. A config é somente leitura porque o `Execute` só lê ela. O log da tesouraria é gravável porque o exercício solo vai escrever nele, e declarar ele agora significa que a conta já está em toda transferência quando você precisar dela.

**6. As três instruções da interface, mais as duas de gestão.** Agora o módulo do programa. A linha para encarar é o atributo `#[instruction(discriminator = ...)]`: é assim que um programa Anchor responde à convenção de nomes de outra pessoa em vez da dele.

```rust
#[program]
pub mod harvest_hook {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.mint = ctx.accounts.mint.key();
        config.paused = false;
        config.bump = ctx.bumps.config;
        config.allowed_len = 0;
        config.allowed = [Pubkey::default(); MAX_ALLOWED];

        let log = &mut ctx.accounts.treasury_log;
        log.mint = ctx.accounts.mint.key();
        log.bump = ctx.bumps.treasury_log;
        log.transfers = 0;
        log.total_amount = 0;
        log.last_amount = 0;
        log.last_destination = Pubkey::default();
        Ok(())
    }

    pub fn allow_destination(ctx: Context<Manage>, destination: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        let len = config.allowed_len as usize;
        require!(len < MAX_ALLOWED, HarvestHookError::AllowlistFull);
        config.allowed[len] = destination;
        config.allowed_len = config
            .allowed_len
            .checked_add(1)
            .ok_or(HarvestHookError::AllowlistFull)?;
        Ok(())
    }

    pub fn set_paused(ctx: Context<Manage>, paused: bool) -> Result<()> {
        ctx.accounts.config.paused = paused;
        Ok(())
    }

    #[instruction(discriminator = InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn initialize_extra_account_metas(ctx: Context<InitializeMetas>) -> Result<()> {
        let metas = harvest_metas()?;
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &metas)?;
        Ok(())
    }

    #[instruction(discriminator = UpdateExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn update_extra_account_metas(ctx: Context<UpdateMetas>) -> Result<()> {
        let metas = harvest_metas()?;
        let mut data = ctx.accounts.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::update::<ExecuteInstruction>(&mut data, &metas)?;
        Ok(())
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        require_keys_eq!(
            *ctx.accounts.source.owner,
            TOKEN_2022_ID,
            HarvestHookError::NotTokenAccount
        );
        let config = &ctx.accounts.config;
        gate(config, &ctx.accounts.destination.key())?;
        msg!(
            "harvest-hook: allowed {} to {}",
            amount,
            ctx.accounts.destination.key()
        );
        Ok(())
    }
}
```

Três coisas que acontecem ali dentro merecem uma frase cada.

O `SPL_DISCRIMINATOR_SLICE` é uma const nos tipos marcadores do crate da interface, e o valor dele é o prefixo sha256 que você imprimiu no seu shell no começo da lição. Você não está copiando um literal hexadecimal, você está referenciando a mesma constante que o programa de token vai computar. É essa a diferença entre uma interface e uma coincidência.

O `ExtraAccountMetaList::init::<ExecuteInstruction>` escreve uma entrada TLV cujo tipo é o discriminador de execute. A lista não é só "umas contas," ela é "as contas de que o `execute` precisa," e a tag de tipo diz isso. É isso que torna o manifesto autodescritivo no lado do cliente.

A primeira linha do `execute` checa que a conta de origem pertence ao Token-2022. O seu programa é publicamente chamável: qualquer um consegue invocar ele direto com quatro contas quaisquer e afirmar que uma transferência está acontecendo. A checagem de dono é a trava mais barata que impede um estranho de alimentar o seu log com lixo. Ela não é uma defesa completa, e a interface oferece uma mais forte: o `TransferHookAccount`, a pequena extensão de conta que um mint com hook força em toda conta de holder (você vai dimensionar contas para ela no passo 8), carrega uma flag de transferring que só é setada enquanto uma transferência de verdade está em voo, então um hook consegue checar ela e recusar chamadas diretas de cara. Para um hook de allowlist-e-log a checagem de dono é proporcional; para um hook que move valor com base no que ele observou, ela não seria.

**7. Ligue as contas, e deixe a cancela vazia.** A struct de contas do `Execute` tem que bater com a ordem da interface exatamente, porque quem constrói essa lista é o Token-2022, não você.

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: read as a key only; the mint is validated by Token-2022 at transfer time.
    pub mint: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = HookConfig::DISCRIMINATOR.len() + HookConfig::INIT_SPACE,
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, HookConfig>,
    #[account(
        init,
        payer = authority,
        space = TreasuryLog::DISCRIMINATOR.len() + TreasuryLog::INIT_SPACE,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump
    )]
    pub treasury_log: Account<'info, TreasuryLog>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Manage<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [CONFIG_SEED, config.mint.as_ref()],
        bump = config.bump,
        has_one = authority @ HarvestHookError::Unauthorized
    )]
    pub config: Account<'info, HookConfig>,
}

#[derive(Accounts)]
pub struct InitializeMetas<'info> {
    #[account(
        init,
        payer = authority,
        space = ExtraAccountMetaList::size_of(2)?,
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: written as raw TLV by spl-tlv-account-resolution.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: key only.
    pub mint: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateMetas<'info> {
    #[account(
        mut,
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: written as raw TLV by spl-tlv-account-resolution.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: key only.
    pub mint: UncheckedAccount<'info>,
    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = config.bump,
        has_one = authority @ HarvestHookError::Unauthorized
    )]
    pub config: Account<'info, HookConfig>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Execute<'info> {
    /// CHECK: source token account, read-only by interface contract.
    pub source: UncheckedAccount<'info>,
    /// CHECK: mint, read-only by interface contract.
    pub mint: UncheckedAccount<'info>,
    /// CHECK: destination token account, read-only by interface contract.
    pub destination: UncheckedAccount<'info>,
    /// CHECK: source owner or delegate, read-only by interface contract.
    pub owner: UncheckedAccount<'info>,
    #[account(
        seeds = [META_LIST_SEED, mint.key().as_ref()],
        bump
    )]
    /// CHECK: the validation account Token-2022 resolved for us.
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, HookConfig>,
    #[account(
        mut,
        seeds = [TREASURY_SEED, mint.key().as_ref()],
        bump = treasury_log.bump
    )]
    pub treasury_log: Account<'info, TreasuryLog>,
}
```

O `ExtraAccountMetaList::size_of(2)` é o espaço para exatamente duas entradas, que é o comprimento de `harvest_metas`. Mude um e você tem que mudar o outro, e uma divergência aqui aparece como uma falha de serialização no init em vez de na hora da transferência, que é a ordem misericordiosa.

Uma decisão de design no `InitializeMetas` vale ser sinalizada em vez de pulada, porque você vai ter que responder por ela em review. A documentação da própria interface lista a terceira conta de initialize-extra-account-metas como a autoridade de mint, assinando. Este programa aceita qualquer signer e faz dele o pagador, e não checa ele contra o mint. Para uma lição isso está de bom tamanho e para um mint que você controla também está, porque a conta é um PDA do seu programa chaveado pelo mint, então ela pode ser criada exatamente uma vez e quem criar ela publica a mesma lista fixa de qualquer jeito. Deixa de estar de bom tamanho no momento em que o seu programa serve mints que você não possui: aí o primeiro chamador decide o que o manifesto diz para sempre, o que é um problema de squatting sem conserto que não seja um redeploy. Se você entregar um hook de uso geral, restrinja essa conta contra a autoridade do mint e guarde a autoridade na sua config. Custo da versão honesta: mais uma conta e mais uma checagem.

Agora a cancela, que é a parte que você escreve. Adicione esta função fora do módulo `#[program]`, exatamente como escrita:

```rust
fn gate(config: &HookConfig, destination: &Pubkey) -> Result<()> {
    // TODO(you): the pause kill-switch first, then the allowlist.
    //   1. if config.paused is true, fail with HarvestHookError::Paused
    //   2. if `destination` is not among the first config.allowed_len entries
    //      of config.allowed, fail with HarvestHookError::NotAllowed
    let _ = (config, destination);
    Ok(())
}
```

Isso compila e está errado, deliberadamente. A ordem importa de um jeito que vale dizer em voz alta: pause é um botão de desligar global e tem que ganhar da allowlist, porque a situação em que você recorre ao pause é a situação em que você não confia mais na sua própria allowlist.

**8. Construa a bancada.** Crie o `hook/tests/hook.rs`. Este é o arquivo mais longo da lição e ele está fazendo algo específico: levantar um mint Token-2022 COM a extensão TransferHook usando builders de instrução crus, não constraints de Anchor. Extensões são de tempo de criação e de nível de instrução, que é o padrão que você usou o curso inteiro a partir de TypeScript; aqui são as mesmas instruções a partir de Rust.

```rust
use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use harvest_hook::{HookConfig, CONFIG_SEED, META_LIST_SEED, TREASURY_SEED};
use litesvm::{types::TransactionMetadata, LiteSVM};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022::{
    extension::{transfer_hook, ExtensionType},
    state::{Account as TokenAccount, Mint},
    ID as TOKEN_2022_ID,
};

const SO_PATH: &str = "target/deploy/harvest_hook.so";

struct Fixture {
    svm: LiteSVM,
    payer: Keypair,
    mint: Pubkey,
    source: Pubkey,
    destination: Pubkey,
    config: Pubkey,
    treasury: Pubkey,
    validation: Pubkey,
}

fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Result<TransactionMetadata, String> {
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(ixs, Some(&payer.pubkey()), signers, blockhash);
    svm.send_transaction(tx).map_err(|e| {
        for line in &e.meta.logs {
            println!("{line}");
        }
        format!("{:?}", e.err)
    })
}

fn setup() -> Fixture {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    svm.add_program_from_file(harvest_hook::ID, SO_PATH).unwrap();

    let mint_kp = Keypair::new();
    let mint = mint_kp.pubkey();
    let mint_len =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap();
    let create_mint = solana_system_interface::instruction::create_account(
        &payer.pubkey(),
        &mint,
        svm.minimum_balance_for_rent_exemption(mint_len),
        mint_len as u64,
        &TOKEN_2022_ID,
    );
    let init_hook = transfer_hook::instruction::initialize(
        &TOKEN_2022_ID,
        &mint,
        Some(payer.pubkey()),
        Some(harvest_hook::ID),
    )
    .unwrap();
    let init_mint =
        spl_token_2022::instruction::initialize_mint2(&TOKEN_2022_ID, &mint, &payer.pubkey(), None, 0)
            .unwrap();
    send(&mut svm, &payer, &[create_mint, init_hook, init_mint], &[&payer, &mint_kp]).unwrap();

    let acc_len = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferHookAccount,
    ])
    .unwrap();
    let mut token_accounts = Vec::new();
    for _ in 0..2 {
        let kp = Keypair::new();
        let create = solana_system_interface::instruction::create_account(
            &payer.pubkey(),
            &kp.pubkey(),
            svm.minimum_balance_for_rent_exemption(acc_len),
            acc_len as u64,
            &TOKEN_2022_ID,
        );
        let init = spl_token_2022::instruction::initialize_account3(
            &TOKEN_2022_ID,
            &kp.pubkey(),
            &mint,
            &payer.pubkey(),
        )
        .unwrap();
        send(&mut svm, &payer, &[create, init], &[&payer, &kp]).unwrap();
        token_accounts.push(kp.pubkey());
    }
    let (source, destination) = (token_accounts[0], token_accounts[1]);

    let mint_to = spl_token_2022::instruction::mint_to(
        &TOKEN_2022_ID,
        &mint,
        &source,
        &payer.pubkey(),
        &[],
        1_000,
    )
    .unwrap();
    send(&mut svm, &payer, &[mint_to], &[&payer]).unwrap();

    let (config, _) = Pubkey::find_program_address(&[CONFIG_SEED, mint.as_ref()], &harvest_hook::ID);
    let (treasury, _) =
        Pubkey::find_program_address(&[TREASURY_SEED, mint.as_ref()], &harvest_hook::ID);
    let (validation, _) =
        Pubkey::find_program_address(&[META_LIST_SEED, mint.as_ref()], &harvest_hook::ID);

    let init = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::Initialize {
            authority: payer.pubkey(),
            mint,
            config,
            treasury_log: treasury,
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::Initialize {}.data(),
    };
    let init_metas = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::InitializeMetas {
            extra_account_meta_list: validation,
            mint,
            authority: payer.pubkey(),
            system_program: solana_system_interface::program::ID,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::InitializeExtraAccountMetas {}.data(),
    };
    send(&mut svm, &payer, &[init, init_metas], &[&payer]).unwrap();

    Fixture { svm, payer, mint, source, destination, config, treasury, validation }
}
```

Coisa pequena com um retorno grande no fim daquela função: o `harvest_hook::instruction::InitializeExtraAccountMetas {}.data()` emite o discriminador da interface, não um do Anchor. A macro gerou aquele tipo a partir do seu handler, viu o override, e assou o `2b220d31a758ebeb` dentro da serialização dele. Então o teste constrói uma instrução padrão da interface usando os tipos gerados do seu próprio programa, e se você algum dia mudar o override o teste quebra em tempo de compilação em vez de em tempo de execução. Consistência de graça, vale saber que ela está aí.

O mint é criado com `ExtensionType::TransferHook` no cálculo de comprimento de conta dele e com `transfer_hook::instruction::initialize` antes do `initialize_mint2`. Essa ordenação não é estilística. Extensões têm que ser inicializadas depois de a conta existir e antes de o mint ser inicializado, e a extensão TransferHook é só de tempo de criação, uma das quatro ciladas que o Checkpoint junta numa tabela, e o motivo de a gente estar cunhando uma variante nova do SPROUT em vez de retrofitar o mint que você terminou na lição passada. Um mint existente sem a extensão nunca consegue ganhar uma. Uma honestidade de escopo sobre esta variante: ela carrega TransferHook sozinha, decimals 0, sem taxas, sem metadados. Ela é o dublê de teste do SPROUT com cancela, não uma recunhagem da pilha R3 completa; compor as extensões do R3 num mint com hook é a mesma ordenação de tempo de criação com mais inicializadores, e nada neste módulo depende dessa composição. Se você quiser o seu SPROUT ao vivo com cancela, você cunha uma variante nova e migra os holders, e nenhuma quantidade de `update-extra-account-metas` muda isso.

Repare também que as duas contas de token alocam espaço para `ExtensionType::TransferHookAccount`. O Token-2022 exige essa extensão em toda conta de holder de um mint com hook, e se você dimensionar elas como contas simples, o `initialize_account3` falha antes de você chegar no hook.

Agora as duas transferências e os asserts:

```rust
fn hooked_transfer(f: &Fixture, amount: u64) -> Instruction {
    let mut ix = spl_token_2022::instruction::transfer_checked(
        &TOKEN_2022_ID,
        &f.source,
        &f.mint,
        &f.destination,
        &f.payer.pubkey(),
        &[],
        amount,
        0,
    )
    .unwrap();
    ix.accounts.extend_from_slice(&[
        AccountMeta::new_readonly(f.config, false),
        AccountMeta::new(f.treasury, false),
        AccountMeta::new_readonly(harvest_hook::ID, false),
        AccountMeta::new_readonly(f.validation, false),
    ]);
    ix
}

fn allow(f: &mut Fixture, destination: Pubkey) {
    let ix = Instruction {
        program_id: harvest_hook::ID,
        accounts: harvest_hook::accounts::Manage {
            authority: f.payer.pubkey(),
            config: f.config,
        }
        .to_account_metas(None),
        data: harvest_hook::instruction::AllowDestination { destination }.data(),
    };
    let payer = f.payer.insecure_clone();
    send(&mut f.svm, &payer, &[ix], &[&payer]).unwrap();
}

#[test]
fn allowlisted_transfer_passes_the_hook() {
    let mut f = setup();
    let destination = f.destination;
    allow(&mut f, destination);
    let ix = hooked_transfer(&f, 100);
    let payer = f.payer.insecure_clone();
    let meta = send(&mut f.svm, &payer, &[ix], &[&payer]).expect("allowlisted transfer should land");
    for line in meta.logs.iter().filter(|l| l.contains("harvest-hook") || l.contains("consumed")) {
        println!("{line}");
    }

    let raw = f.svm.get_account(&f.config).unwrap();
    let config = HookConfig::try_deserialize(&mut raw.data.as_slice()).unwrap();
    assert_eq!(config.allowed_len, 1);
}

#[test]
fn stranger_transfer_fails_the_hook() {
    let mut f = setup();
    let ix = hooked_transfer(&f, 100);
    let payer = f.payer.insecure_clone();
    let err = send(&mut f.svm, &payer, &[ix], &[&payer])
        .expect_err("a non-allowlisted destination must be rejected by the hook");
    assert!(err.contains("Custom"), "expected the hook's own error, got {err}");
}
```

O `hooked_transfer` é onde a bancada está mentindo para você discretamente, e vale nomear isso agora para a próxima lição aterrissar. Aquelas quatro contas acrescentadas, os dois extras na ordem da lista, depois o programa de hook, depois a conta de validação, são exatamente o que um cliente precisa fornecer, exatamente nessa ordem. Aqui eu digitei elas na mão porque eu conheço o meu próprio hook. Uma carteira não conhece. Essa lacuna é o assunto inteiro da próxima lição.

![Uma transferência com hook atravessa o Token-2022 até o Execute do hook na profundidade dois, com uma aresta de falha antes de a sua lógica rodar e uma dentro da própria cancela.](assets/v07-flowchart.webp)

**9. Rode isso, e leia o vermelho.** Construa o programa para bytecode SBF primeiro, porque a bancada carrega o `.so` compilado:

```bash
cargo build-sbf
cargo test -p harvest-hook -- --nocapture
```

O Cargo roda dois binários de teste: primeiro os testes unitários do próprio crate, que dependendo da sua toolchain podem estar vazios ou carregar um teste gerado trivial, e de todo jeito não te dizem nada, depois o `tests/hook.rs`, que é o que vale ler. Com a cancela ainda vazia, essa segunda suíte volta assim (os dois testes rodam em paralelo, então a ordem inverte de rodada para rodada):

```
running 2 tests
test allowlisted_transfer_passes_the_hook ... ok
test stranger_transfer_fails_the_hook ... FAILED

failures:
    stranger_transfer_fails_the_hook

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Um verde, um vermelho, e o vermelho é a sua tarefa. O hook está ligado corretamente: o Token-2022 encontrou ele, resolveu as contas, chamou o `Execute`, e o seu código disse sim para todo mundo. Agora vá implementar o `gate` para ele dizer não. Duas linhas de `require!`, pause primeiro. Quando você tiver isso, o mesmo comando imprime:

```
running 2 tests
Program log: harvest-hook: allowed 100 to BtqSbosaGCZZczgs6oAVRoRkMYLTi3v8qs6t8DzyG88Y
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ consumed 11948 of 184840 compute units
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb consumed 27792 of 200000 compute units
test allowlisted_transfer_passes_the_hook ... ok
test stranger_transfer_fails_the_hook ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Os seus números de compute vão diferir dos meus por alguns milhares, pelo motivo da busca de bump acima. E quando a transferência do estranho falhar, o log é a coisa que vale tirar print, porque é o nome do seu programa no meio de uma transferência de token:

```
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb invoke [1]
Program log: Instruction: TransferChecked
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ invoke [2]
Program log: Instruction: Execute
Program log: AnchorError thrown in src/lib.rs:261. Error Code: NotAllowed. Error Number: 6001.
    Error Message: Destination is not on the hook allowlist.
Program HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ failed: custom program error: 0x1771
Program TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb failed: custom program error: 0x1771
```

Leia as duas últimas linhas de novo. O seu erro propagou para fora do `Execute` na profundidade de invoke 2 e matou o `TransferChecked` do Token-2022 na profundidade 1. 0x1771 é 6001, que é o offset de erro do Anchor mais o índice de `NotAllowed` no seu enum. Uma nota de calibração antes de você comparar este log com o seu: o número de linha `src/lib.rs:261` acompanha o SEU arquivo, não o meu, porque quem escreveu a cancela foi você, e a pubkey de destino é o keypair que a sua rodada gerou. O que tem que bater exatamente é o par de nomes de instrução, `Error Code: NotAllowed. Error Number: 6001.`, e o 0x1771 nas duas linhas de fechamento. O programa de token não tem opinião sobre a sua allowlist e nenhum jeito de sobrepor ela. Ele perguntou, você disse não, a transferência acabou.

## Challenge

Solo, sem apoio: torne o log da tesouraria real.

Agora mesmo o `Execute` escreve um `msg!` e segue em frente. O seu hook já encaminha a conta de log da tesouraria em toda transferência, declarada gravável no manifesto, sem fazer nada. Mude isso. Em toda transferência permitida, atualize o `TreasuryLog` no lugar: incremente `transfers`, some `amount` em `total_amount` com aritmética checada, e registre `last_amount` e `last_destination`. Depois estenda a bancada para provar isso: depois de duas transferências permitidas de tamanhos diferentes, desserialize a conta de log e faça assert dos quatro campos. Depois rode uma transferência não permitida e faça assert de que o log NÃO se moveu, que é o assert que realmente importa, porque ele prova que o revert desfez a sua escrita junto com a transferência.

Aceito quando: o `cargo test -p harvest-hook` estiver verde com os seus novos asserts, o contador sobreviver a duas transferências, e uma transferência rejeitada deixar todo campo intocado. Quebre isso uma vez de propósito para ter certeza, colocando o destino na allowlist e fazendo assert do total errado.

Existe também um exercício focado só no núcleo de decisão: o `hook-execute-gate`, o coding challenge desta lição no painel de challenge da plataforma do curso, onde você implementa `hook_execute(destination_allowed, is_paused, amount)` contra cinco casos. Dois deles existem especificamente para pegar o bug de ordenação: um hook pausado tem que rejeitar um destino que está na allowlist, e pause tem que ganhar quando as duas condições são hostis. Se o seu `gate` passou na bancada mas você quer ter certeza de que a precedência está certa na sua cabeça, faça esse primeiro; leva dois minutos.

## Checkpoint

O critério desta lição é um comando e uma afirmação.

```bash
cargo build-sbf && cargo test -p harvest-hook -- --nocapture
```

Verde nos dois testes, com `harvest-hook: allowed` no log que passa e `Error Code: NotAllowed` no que falha. Se você fez o challenge, mais os seus quatro asserts de log. A afirmação que você deve conseguir fazer sem consultar nada: a conta de validação para o mint M e o programa de hook H é o PDA dos seeds `["extra-account-metas", M]` em H, e existe exatamente uma delas por mint.

Quatro jeitos de este build dar errado, reunidos num lugar só porque são os que custam horas em vez de minutos:

![Quatro ciladas de hook pareadas com causa e conserto: program id errado no PDA, contas de holder sem tamanho, esperar que o Execute mova fundos, e retrofitar uma extensão de tempo de criação.](assets/v08-comparison.webp)

Duas falhas que eu espero durante a própria rodada, para você conseguir se autodiagnosticar em vez de bisseccionar.

Se a transferência reverter antes de o `Execute` logar qualquer coisa, você está na aresta de falha A: a lista de contas está errada. Cheque primeiro a lista no `hooked_transfer` — todo extra do manifesto presente, mais o programa de hook e a conta de validação; presença é o que importa, já que o Token-2022 resolve os extras por pubkey, não por posição — e cheque que o `ExtraAccountMetaList::size_of` bate com o número de entradas que `harvest_metas` devolve. Uma divergência de tamanho corrompe a lista silenciosamente no init e só aparece aqui.

Se o `cargo build-sbf` tiver sucesso mas a bancada não conseguir encontrar o programa, cheque o `SO_PATH`. O `cargo test` roda com a raiz do pacote como diretório de trabalho, então `target/deploy/harvest_hook.so` está certo para o layout acima e errado se você aninhou o crate dentro de um workspace com um diretório target compartilhado. Se você aninhou mesmo, aponte o `SO_PATH` para o target do workspace em vez disso.

![A escada de artefatos vai do decode-mint ao mint SPROUT terminado até o harvest-hook desta lição, e daí para o resolvedor do cliente, a roteabilidade e o roteamento de taxas.](assets/v09-timeline.webp)

Aproveite o marco. Você escreveu um programa Solana que o software de outras pessoas agora é obrigado a chamar, e você provou ele contra um programa de token de verdade com uma transferência de verdade. Esse é um tipo de artefato diferente de todo o resto neste curso: o SPROUT é uma configuração, o `harvest-hook` é código com um endereço, e a diferença é que código consegue dizer não.

E é aí que fica desconfortável. O seu hook passa na bancada dele, mas a bancada entregou cada conta a ele numa bandeja. No momento em que uma carteira de verdade ou uma DEX constrói uma transferência comum de quatro contas do seu SPROUT com hook, as contas de que o seu programa precisa simplesmente não estão na transação, e a transferência reverte antes de a sua lógica rodar. Então quem deveria colocar elas ali, e por que tanta gente do ecossistema decidiu que a resposta é "nós não"? Na próxima lição você senta na cadeira do integrador, vê esse revert acontecer, e escreve o resolvedor que conserta isso.
