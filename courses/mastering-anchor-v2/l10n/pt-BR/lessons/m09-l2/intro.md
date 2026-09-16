# Desmonte o framework II: o diff, e o que o Anchor gera

Você acabou de reconstruir o quarter-vault nativo em pinocchio: um discriminator manual de um byte, validação com `TryFrom`, um `invoke_signed` feito na mão, e ele passou na mesma trava de withdraw em LiteSVM em que o R2 passou. Lamports saíram do PDA sob autoridade do programa, o over-withdraw foi recusado, e não tinha Anchor em lugar nenhum do crate. Aquela era a metade de construir do construa-duas-vezes. Esta é a metade de reenquadrar.

Aqui está a afirmação estrutural, dita primeiro para você sair com ela mesmo que não leia mais nada. O `#[derive(Accounts)]` que você deletou na lição passada é exatamente o load, o check e o despacho que você acabou de escrever na mão, mais uma varredura de contas duplicadas e uma trava de borrow que você não consegue omitir por acidente. Você consegue optar por sair da varredura, deliberadamente, por campo, e a grafia faz você dizer isso em voz alta. Ler a expansão comprova essa frase, linha por linha, contra o seu próprio código.

Então vamos ler. A ferramenta que imprime o código que uma macro gera é o `cargo-expand`. Instale ele uma vez e aponte ele para o seu vault de framework:

```bash
cargo install cargo-expand   # re-check crates.io for a newer version at build time
# The expansion itself runs under a nightly rustc (-Zunpretty=expanded is a nightly
# flag): `rustup toolchain install nightly` once, and cargo-expand finds it on its
# own. The workspace stays pinned to stable 1.89.0; only the expansion borrows nightly.
cargo expand --package quarter-vault > expanded.rs
```

Abra o `expanded.rs` e procure blocos `impl` perto da sua struct `Withdraw`. O que você está olhando é uma função `try_accounts` que o derive escreveu para você, e ela está fazendo os mesmos trabalhos de carregar-e-checar que o seu `TryFrom` nativo fazia, numa ordem que você agora consegue nomear. Esse arquivo, ao lado do seu vault nativo, é a lição inteira. Tudo abaixo anota o diff.

Uma expectativa para ajustar antes de você olhar, porque ela vai te salvar de caçar uma coisa que não está lá. Cada bloco gerado impresso abaixo é uma versão **estilizada** da saída de verdade: a mesma estrutura e a mesma ordem, com o ruído removido. A expansão de verdade são milhares de linhas de caminhos completamente qualificados, lifetimes gerados e blocos `#[automatically_derived]`, e ler ela ao pé da letra te ensina menos que ler ela contra um esboço. Então os esboços são um mapa, não uma transcrição, e os identificadores neles são descritivos em vez de exatos. Quando o Lab te pedir para achar uma coisa na sua própria saída, ele vai te dar o padrão para grepar em vez de um nome de símbolo para casar, exatamente por essa razão.

O recuo da ajuda desta lição: a caminhada pelo diff está completamente trabalhada, feita para você peça por peça. O Lab te faz rodar o `cargo expand` no seu próprio vault e anotar a saída de verdade contra o seu código nativo. A trava de três linhas no fim é solo, sem apoio.

![Uma tabela emparelhando cada passo nativo do pinocchio com a peça da expansão do V2 que substitui ele, terminando com uma varredura de contas duplicadas que o seu código nativo só aproxima para um par e uma trava de borrow da qual ele não tem versão nenhuma.](assets/v01-comparison.webp)

## O diff

### Ordem de load: as quatro checagens, geradas numa função só

Abra o seu `TryFrom` nativo para o withdraw. Ele rodava seis travas antes de o handler rodar, e ele rodava elas numa ordem que você escolheu. Aqui está a metade de load dele, aparada para as quatro travas que têm uma gêmea gerada:

```rust
impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(
        (data, accounts): (&'a [u8], &'a [AccountInfo]),
    ) -> Result<Self, ProgramError> {
        let [authority, config, vault, _system_program, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 1. signer check on the authority
        if !authority.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        // 2. owner check: the config PDA must be owned by THIS program
        if !config.is_owned_by(&crate::ID) {
            return Err(ProgramError::IncorrectProgramId);
        }

        let cfg = config.try_borrow_data()?;

        // 3. data-length check, then 4. discriminator check
        if cfg.len() != CONFIG_LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        if cfg[0] != VAULT_DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        // gates 5 (stored authority) and 6 (duplicate-mutable) follow; see m09-l1
        // ...
    }
}
```

Uma ponte de nomenclatura, para o diff ler limpo, e vale ser exato porque os dois builds não compartilham endereços. Os dois programas têm os mesmos dois *papéis*: um registro de propriedade do programa segurando o discriminator, a authority e um bump, e uma conta de propriedade do System segurando o SOL de verdade. Nativamente você chamava eles de `config` e `vault`, com seeds `[b"config", authority]` e `[b"vault", authority]`. A versão de framework chama eles de `state` e `sol_vault`, com seeds `[b"vault", authority]` e `[b"sol", authority]`. Ids de programa diferentes e literais de seed diferentes, então estes são quatro endereços distintos, não dois. O que mapeia através é o papel, e essa é a única coisa que o diff abaixo está afirmando. Onde a versão nativa guarda um bump (o do vault de SOL) e deriva o do config a partir dos dados de instrução, a versão de framework guarda os dois, que é a única diferença estrutural que vale carregar: é por isso que `state.bump` e `state.sol_bump` os dois aparecem nos constraints abaixo e só um byte aparece no seu layout nativo.

Agora olhe o código V2 que gerou o equivalente. São quatro campos e dois atributos:

```rust
#[derive(Accounts)]
pub struct Withdraw {
    #[account(mut)]
    pub authority: Signer,

    #[account(
        mut,
        seeds = [b"vault", authority.address().as_ref()],
        bump = state.bump,
        constraint = state.owner == *authority.address() @ VaultError::Unauthorized,
    )]
    pub state: Account<Vault>,

    #[account(
        mut,
        seeds = [b"sol", authority.address().as_ref()],
        bump = state.sol_bump,
    )]
    pub sol_vault: SystemAccount,

    pub system_program: Program<System>,
}
```

As suas quatro checagens nativas estão todas ali dentro, e a expansão deixa isso óbvio. Você abriu essa mesma maquinaria de `try_accounts` no módulo 1, no greeter, no abstrato. Aqui está ela de novo, mas agora cada linha gerada tem uma gêmea escrita na mão para a qual você consegue apontar. O `Account<Vault>` no campo `state` gera a checagem de owner (a sua checagem 2) e a checagem de discriminator de 8 bytes (a sua checagem 4) dentro do load dele, antes de qualquer constraint rodar. O `Signer` no `authority` gera a sua checagem de signer (checagem 1). A trava de `data.len()` que você escreveu na mão (checagem 3) é o próprio load se recusando a ler um slice curto demais para o tipo. Note a única assimetria: a sua era igualdade exata, `cfg.len() != CONFIG_LEN`, então uma conta de propriedade do programa que é *longa* demais falha na sua checagem e passa na do framework. Nenhuma das duas está errada, elas são afirmações diferentes. Igualdade exata é uma declaração mais forte sobre a forma da conta; um mínimo é o que um framework consegue prometer para cada tipo de conta que ele não conhece. Quatro declarações `if` escritas na mão colapsam em duas anotações de tipo, e o tipo é a checagem.

Note a grafia dos constraints enquanto você está aqui, porque ela é um delta do V2 que o módulo de migração vai fazer você aplicar na mão. Na linha v1 esta checagem de chave guardada era a keyword `has_one = owner`. O V2 deprecia o `has_one` em favor das formas de expressão. Ela ainda parseia (e o derive deliberadamente mantém o span de código da keyword só para a geração de código poder sublinhar ela com um aviso), mas as grafias recomendadas agora são `address = expr` para o caso comum de "esta conta tem que ser igual àquela chave guardada" e `constraint = expr @ err` quando você precisa de uma comparação que a keyword estreita nunca conseguiria expressar. O módulo 3 ensinou a troca. Aqui você está só lendo a saída dela.

Um fato estrutural que a expansão fixa, e é o que as pessoas erram. Ordem de campo é ordem de load. O derive caminha a sua struct de cima para baixo, então o `authority` carrega antes do `state`, que é por que o constraint no `state` consegue com segurança comparar contra um campo declarado acima dele. Reordene os campos e você genuinamente muda qual checagem roda contra dado carregado contra dado não carregado. Na sua versão nativa você controlava essa ordem na mão, escolhendo onde colocar cada `if`. O framework controla ela pela posição do campo. Mesma ordem, alavanca diferente.

Deixe isso concreto, porque é o único lugar em que ordem de campo não é cosmética. O constraint no `state` lê o campo `owner` de dentro da conta `state` carregada e compara ele contra o `authority` declarado acima dele. Se você movesse o `state` para cima do `authority` na struct, o constraint referenciaria um campo que ainda não tinha carregado, e o derive rejeitaria a struct em tempo de compilação em vez de rodar uma checagem contra nada. Nativo, o mesmo erro é um reordenamento silencioso de dois blocos `if` que compilador nenhum jamais sinalizaria. O framework transformou uma disciplina de ordem numa garantia de ordem.

![O try_accounts gerado carrega cada campo com checagens de owner e de discriminator, roda os hooks de constraint, e depois varre por contas mutáveis duplicadas, uma varredura que código nativo não tem.](assets/v02-annotated-code.webp)

### Hooks de constraint: as checagens que rodam depois de tudo carregar

A fase de load comprova que cada conta é o que ela diz ser. Os hooks de constraint comprovam que as contas se relacionam corretamente entre si. No código acima, o `seeds`, o `bump = state.bump` e a cláusula `constraint = state.owner == ...` são hooks de constraint, e a expansão coloca eles num bloco distinto que roda só depois de cada campo ter carregado.

Você escreveu estas na mão também, só que fundidas dentro do seu handler em vez de separadas. O seu withdraw nativo re-derivava o PDA e comparava ele, ou confiava nas seeds que você passava para o `invoke_signed`; ele lia o `state.owner` e recusava um chamador que não casasse. O framework puxa essa lógica para fora do handler por completo e roda ela como uma fase separada, que é por que um constraint do V2 nunca consegue disparar "tarde demais". Ele fisicamente não consegue rodar depois do seu handler, porque ele mora numa função que termina antes de o seu handler começar. Nativo, essa garantia era a sua disciplina. Gerada, ela é estrutural.

![Uma comparação mapeando os constraints de bump, de authority guardada e de seeds para o que cada um gera e para a checagem nativa escrita na mão que ele substitui.](assets/v03-comparison.webp)

### O dispatcher: o seu match de u8, crescido para oito bytes

O seu programa nativo roteava instruções com um match no primeiro byte dos dados de instrução: `0` era init, `1` era withdraw. Aquele era o dispatcher inteiro. O dispatcher do V2 faz o trabalho idêntico com uma tag de 8 bytes em vez de um byte. Ele lê os oito bytes iniciais dos dados de instrução, casa eles contra o discriminator de instrução de cada handler, e roteia para o handler certo. Depois, e só depois, o `try_accounts` roda para aquela instrução.

A diferença é largura, não tipo. Um byte te dá 255 instruções; oito bytes de `sha256` sobre `global:withdraw` te dão uma tag resistente a colisão que um cliente consegue montar sem coordenar um registro de números com você. Você trocou um byte por um hash e comprou compatibilidade de wire. É esse o upgrade inteiro. Se você internalizou a armadilha do namespace `global:` no greeter no módulo 1, você já sabe o único lugar em que isto morde: não existe namespace `instruction:`, e uma tag montada na mão a partir do preimage errado roteia para nada.

### O caminho de init: criação, e o discriminator que você tinha que escrever você mesmo

O caminho de withdraw mostra load, check e despacho. Existe um trabalho que o withdraw nunca toca, e diffar ele é onde a classe de bug de discriminator volta para casa: criação de conta. O seu `Init::process` nativo criava a conta de config na mão. Você computava os lamports isentos de aluguel, rodava uma CPI de `CreateAccount` para alocar o espaço e atribuir o seu programa como owner, e depois, criticamente, você escrevia a tag de tipo no byte zero você mesmo:

```rust
// native Init::process: fund rent, allocate, assign owner, THEN tag the account by hand
let lamports = Rent::get()?.minimum_balance(CONFIG_LEN);
CreateAccount {
    from: self.authority,
    to: self.config,
    lamports,
    space: CONFIG_LEN as u64,
    owner: &crate::ID,
}
.invoke_signed(&signer)?;
// forget this next line and the account has NO discriminator: type cosplay is now open
let mut cfg = self.config.try_borrow_mut_data()?;
cfg[0] = VAULT_DISCRIMINATOR;
```

O código V2 que gera tudo aquilo é um constraint só:

```rust
#[account(
    init,
    payer = authority,
    space = Vault::DISCRIMINATOR.len() + Vault::INIT_SPACE,
    seeds = [b"vault", authority.address().as_ref()],
    bump,
)]
pub state: Account<Vault>,
```

O `init` gera a CPI de `CreateAccount`, computa e financia o mínimo isento de aluguel a partir do `payer`, e escreve o discriminator na conta nova, todos os três. A única linha que você mais provavelmente esqueceria nativamente, marcar o byte zero, é a que o framework nunca vai pular. Perca a tag no seu init feito na mão e qualquer conta de propriedade do programa do mesmo tamanho pode depois ser desserializada como um vault, a classe exata de type cosplay que a checagem de discriminator no lado do load existe para fechar. Nativo, criação e validação são dois lugares que você tem que manter de acordo na mão. Gerado, o `init` na entrada e a checagem de load na saída são duas metades de uma garantia só que o derive escreve juntas.

![Uma comparação mostrando o constraint init gerando alocação, financiamento isento de aluguel e a escrita do discriminator, que é a linha nativa mais facilmente esquecida.](assets/v04-comparison.webp)

### MUT_MASK: a trava que o seu vault nativo não tem

Aqui está o primeiro lugar em que a expansão faz uma coisa que o seu código nativo só gesticula. Procure o `try_accounts` gerado e você vai achar uma varredura sobre os campos mutáveis. Você escreveu *uma* linha nessa família: a trava 6 do seu `TryFrom`, a comparação `if vault.key() == authority.key()`, que fecha exatamente um par porque aquele é o par que você por acaso pensou.

Essa é a diferença, e ela é a diferença inteira. A sua trava 6 é uma comparação escolhida a dedo sobre um par de contas que você escolheu. A varredura do derive é exaustiva: cada campo mutável contra cada outro, computada a partir da struct em vez de a partir da sua atenção, e mesclada através de structs aninhadas para uma colisão entre um campo direto e um enterrado num composto ser pega também. Adicione uma terceira conta mutável na sua struct nativa e a trava 6 não cresce com ela; adicione uma numa struct do V2 e a máscara cresce, caladinho, em tempo de compilação. A classe não é fechada lembrando de checar um par. Ela é fechada nunca tendo que lembrar.

Você encontrou o `MUT_MASK` no módulo 1, então o mecanismo não é novo: o derive computa um const associado de 256 bits em tempo de compilação marcando quais campos são mutáveis, e o dispatcher varre aquela máscara contra um bitvec de endereços em runtime conforme as contas carregam, retornando `ConstraintDuplicateMutableAccount` se dois campos mutáveis carregarem o mesmo endereço. A forma é fixada quando você compila; a colisão só é conhecível quando a transação aterrissa. O que é novo é ver a lacuna na cobertura. Coloque a sua trava 6 nativa ao lado da expansão e a diferença é escopo: um par que você nomeou, contra cada par que a struct implica.

Essa classe é real, ou ela é uma trava cosmética que você poderia pular? Ela é real. Imagine um handler de batch que toca dois pools de prêmio, os dois graváveis. Passe o mesmo pool para os dois e um handler ingênuo debita ele uma vez, credita ele duas vezes, e a contabilidade está errada de um jeito que nenhum teste com contas distintas jamais traria à superfície. O framework rejeita aquela chamada antes de o seu handler rodar. Quando você genuinamente quer passar uma conta duas vezes, você opta por sair por campo, e o opt-out é escrito para você sentir: `unsafe(dup)`. O `dup` simples sem o invólucro `unsafe` é um erro de compilação, de propósito. A keyword é a luz do cinto de segurança.

Tem um detalhe aqui que importa mais na próxima lição do que importa agora, então arquive. A varredura de duplicatas não é por struct, ela é por árvore. O derive emite um trait que reporta as chaves mutáveis que uma struct serializa na saída, e quando você aninha uma struct de Accounts dentro de outra, a struct de fora chama a implementação de cada struct de dentro e mescla as chaves num conjunto só. Então a trava pega uma colisão até quando a mesma conta chega uma vez como campo direto e uma vez enterrada dentro de um composto. É exatamente essa a forma que o floor-registry do capstone tem: ele compõe o cabinet-counter, o vault, o escrow e o swap, e uma composição ingênua feita na mão é precisamente onde um bug de aliasing de duplicate-mutable se esconderia. O seu vault nativo nunca teve essa varredura num nível. Um registry nativo precisaria dela em cada nível, mesclada, e teria ela em lugar nenhum.

![Um diagrama de duas pistas contrastando a varredura exaustiva da trava de duplicate-mutable do V2 contra a comparação de um par só escrita na mão do pinocchio nativo.](assets/v05-diagram.webp)

### CpiHandle: o borrow que substituiu uma cilada que você tinha que lembrar

A segunda linha sem gêmea nativa não é uma linha de jeito nenhum, e sim um erro de compilação que o framework consegue produzir e o seu código nativo não.

Na linha 0.x, e no v1, você conseguia segurar uma conta desserializada, invocar uma CPI que mutava os bytes daquela conta on-chain, e depois ler a sua cópia obsoleta em memória como se nada tivesse mudado. A correção era chamar `.reload()` depois da CPI, e esquecer era um jeito clássico de entregar um bug que raciocinava sobre estado pré-CPI. O seu vault nativo tem a mesma exposição com a disciplina arrancada: você segura borrows crus, e nada te impede de re-ler um valor que você capturou antes da transferência como se ele fosse atual.

O V2 deixa aquele erro não-representável. Você não entrega mais para uma CPI um clone de `AccountInfo`; você entrega para ela um `CpiHandle`, obtido com `.cpi_handle()` ou `.cpi_handle_mut()`, e um `CpiHandle` é um borrow vivo do Rust da conta segurado por todo o tempo em que o handle está em escopo. Enquanto ele está vivo, o borrow checker não vai te deixar formar um segundo borrow das mesmas contas para ler os dados tipados delas. A leitura e o handle não conseguem coexistir. Você encontrou isto de cara no módulo 4, onde reordenar uma leitura para depois do drop do handle era a correção inteira. No diff, o significado dele afia: esta é a trava que substituiu o `.reload()`, e ela mora em tempo de compilação. Você não lembra dela. O compilador lembra.

```rust
// V2 withdraw: the CpiHandle borrow is live for exactly this call
let owner = *ctx.accounts.authority.address();  // deref: an owned copy, not a borrow
let signer_seeds: &[&[&[u8]]] =
    &[&[b"sol", owner.as_ref(), &[ctx.accounts.state.sol_bump]]];

let cpi = CpiContext::new(
    ctx.accounts.system_program.address(),   // hands back an &Address directly
    Transfer {
        from: ctx.accounts.sol_vault.cpi_handle_mut(),
        to:   ctx.accounts.authority.cpi_handle_mut(),
    },
).with_signer(signer_seeds);
transfer(cpi, amount)?;
// only after `transfer` consumes `cpi` and the handles drop
// can you read ctx.accounts typed data again, and it is live.
```

O seu equivalente nativo assinava o mesmo withdrawal na mão, no vocabulário de seeds que o seu próprio programa usava, e o runtime aceitava ele exatamente do mesmo jeito:

```rust
// From Withdraw::process last lesson. `self.vault_bump` is the byte you read out
// of config[33]; `b"vault"` is the SOL PDA's seed literal in the native build,
// which is `b"sol"` in the framework build. Same role, different word.
let bump = [self.vault_bump];
let seeds = [
    Seed::from(b"vault"),
    Seed::from(self.authority.key().as_ref()),
    Seed::from(&bump),
];
let signers = [Signer::from(&seeds)];
Transfer {
    from: self.vault,
    to: self.authority,
    lamports: self.amount,
}
.invoke_signed(&signers)?;
```

Os dois movem lamports para fora de um PDA sem chave sob autoridade do programa. A diferença é invisível no caminho felizinho e decisiva no caminho com bug: a versão nativa não tem compilador nenhum de pé entre você e uma leitura obsoleta pós-CPI. O framework construiu um a partir do sistema de tipos.

Os riscos daquela trava sobem no momento em que programas compõem, que é o assunto inteiro da próxima lição. Um vault de instrução única lê o estado próprio dele, transfere e retorna; a janela para uma leitura obsoleta é estreita. O floor-registry do capstone faz CPI para o vault, o escrow e o swap, e depois de cada uma daquelas chamadas retornar, qualquer campo tipado que você estava segurando é candidato a obsolescência. É precisamente essa a situação em que o v1 entregava bugs, porque o reload que você devia estava uma chamada mais fundo numa composição que você também estava tentando raciocinar sobre. No V2 o modelo de borrow escala com a composição de graça: cada `CpiHandle` que você pega faz borrow de exatamente as contas que aquela chamada toca, por exatamente o escopo dela, e o compilador rastreia todos eles de uma vez. Quanto mais fundo você compõe, mais a trava está fazendo, e mais a versão nativa estaria te pedindo para lembrar.

![Duas linhas do tempo contrastando a cilada de leitura obsoleta do v1 e do nativo com o V2, onde ler através de um borrow vivo de CpiHandle é um erro de compilação em vez de uma surpresa de runtime.](assets/v06-timeline.webp)

### O trade-off: ler isso não é licença para fazer na mão

Agora a parte honesta, porque esta lição pode ser lida errado. Você consegue ler a expansão. Isso é uma habilidade de verdade e desmistifica a macro para sempre. Mas não confunda "eu consigo ler isso" com "eu deveria escrever isso". As checagens geradas são o ponto. A varredura do MUT_MASK e o borrow do CpiHandle não são overhead que o framework impõe em você; eles são duas classes de bug que o framework deixa impossíveis de esquecer, e o seu vault nativo, elegante como é, esqueceu as duas. A versão nativa é um espelho didático, não um alvo de entrega. O valor inteiro do framework é a trava que ele não vai te deixar omitir.

Que é também por que confiar no código gerado é razoável em vez de preguiçoso. A cola que varre as suas contas é ela mesma fuzzada e checada contra comportamento indefinido, do mesmo jeito que você checaria um programa atrás do qual você estava a ponto de colocar dinheiro. Você viu isso no módulo 1: a suíte de testes do próprio framework carrega as testemunhas. O código que o derive escreve para você é verificado, não afirmado. O seu vault nativo, em contraste, é confiável só na medida em que você pessoalmente testou ele, e você testou uma instrução num caminho felizinho mais uma rejeição. O `try_accounts` do derive é exercitado através do ecossistema inteiro e checado por ferramental que você não teve que escrever. Essa assimetria é o argumento calado a favor do framework: não que você não consegue escrever as checagens, mas que a versão delas do framework é testada mais duro do que a sua jamais vai ser.

Um número mantém essa confiança honesta. O código gerado tem um custo medido e em movimento, e o time do V2 mede ele na aberta. O PR #4914, mergeado em 2026-08-13, revisou os benchmarks da manchete do V2 para baixo, de 95% para 94% de redução de bytecode e de 9.9x para 8.8x de melhoria de CU. Um framework que corrige o marketing próprio dele para baixo é um framework do qual você pode confiar nos números para cima. O código que você está diffando é rápido, e ele é honestamente rápido.

Então quando o nativo é de fato a chamada certa, e não só um exercício? A resposta honesta é estreita mas real: um caminho quente onde você perfilou uma instrução específica, comprovou que o overhead por conta do framework é o seu gargalo, e decidiu que a CU que você compra de volta vale possuir cada checagem na mão para sempre. Essa é uma decisão rara e medida, não um default. E note a pista no design próprio do V2: o framework é ele mesmo uma reescrita no_std em cima do pinocchio, e ele oferece o `asm-v2` para exatamente esses caminhos quentes, então você consegue descer para o metal por uma instrução sem abandonar as travas em todas as outras. O framework não é o inimigo da CU que você quer de volta; ele é o jeito de gastar esse orçamento onde importa e manter os cintos de segurança em todo lugar mais. Ler a expansão é o que te ganha esse julgamento. Agora você consegue olhar um `try_accounts` gerado, ver quanto cada linha custa e qual bug ela fecha, e decidir, com números, quais linhas você jamais ia querer possuir você mesmo. Para quase todas elas, a resposta é não.

![Uma tabela listando cada peça gerada da expansão, o passo nativo que ela substitui, a classe de bug que ela fecha, e se ela dispara em tempo de compilação ou em runtime.](assets/v07-table.webp)

### Um nome que sobrevive, e um compasso embaixo do piso

Duas notas de pé antes do Lab, porque as duas são armadilhas.

Primeira, o `AccountLoader`. Se você grepar a documentação do V2 você ainda vai achar ele, e é tentador ler isso como "nada mudou". Errado do outro lado também: não leia a reescrita do modelo de conta como "o AccountLoader desapareceu". Não é nem um nem outro. No V2 o `AccountLoader` é repropositado como um cursor sequencial de contas, e a documentação avisa que ele quer dizer outra coisa agora do que queria dizer na linha 0.x. Mesmo nome, trabalho diferente. Carregue isso com cuidado.

Segunda, o piso deste curso tem um alçapão, e você ganha exatamente uma olhada através dele. O Anchor V2 consegue linkar sBPF escrito na mão: o `asm-v2` te deixa descer um caminho quente para assembly e fazer o framework linkar ele dentro do programa que a VM roda. Olhe uma vez. Isso te diz que o framework tem uma saída de emergência até o fundo, até a instrução que a máquina executa. Depois pare, porque caçar profundidade de sBPF aqui é o curso errado. Por que a VM roda daquele jeito — o loader, os syscalls, o verificador, programas sem framework nenhum — pertence ao curso de Low-Level Solana, não a este. Se você quer ir para baixo do sBPF, essa é a porta. Aqui, a gente fica no nível do framework: no que a macro expande, e por quê.

## Lab: anote a sua própria expansão

O recuo da ajuda: os passos 1 até 4 estão trabalhados, você roda os comandos e lê a saída; o passo 5 você escreve as anotações você mesmo contra o seu arquivo de verdade.

1. **Gere a expansão.** A partir do crate do seu quarter-vault de framework, rode os dois comandos do topo da lição. Se o `cargo expand` der erro numa macro, não vá olhar o seu binário `anchor` — o `cargo expand` nunca consulta o CLI do Anchor de jeito nenhum. Expansão é o rustc rodando as proc macros do *grafo de dependências* deste crate, então a única coisa que decide qual gramática expande é a linha `anchor-lang` no `Cargo.toml`. Se aquela linha lê uma versão 1.x, código de `CpiHandle` e de `Account` Pod não consegue expandir não importa qual CLI está no seu PATH; fixe `anchor-lang = "2.0.0-rc.1"` do crates.io, exatamente como o m01-l2 mostrou (`2.0.0-rc.1` em 2026-08-22; re-cheque se tem um rc mais novo ou uma tag estável), e rode de novo. Essa inversão — o pin do crate seleciona o framework, nunca o CLI — é a lição de macro reafirmada como uma regra de ferramental, e ela volta como o desfecho do m10.

2. **Ache a fase de load.** Procure no `expanded.rs` por `try_accounts` perto de `Withdraw`. Marque a linha que carrega o `state` como um `Account<Vault>`. Abra o seu `TryFrom` nativo ao lado e trace uma linha daquele load gerado único até as suas duas checagens escritas na mão: a checagem de owner e a checagem de discriminator. Confirme que elas carregam antes de qualquer constraint rodar.

3. **Ache o bloco de constraint.** Abaixo dos loads, ache onde as cláusulas `constraint`, `seeds` e `bump` são impostas. Mapeie cada uma para a checagem fundida na mão que ela substituiu no seu handler nativo. Confirme que o bloco roda depois de todos os campos carregarem e antes do handler.

4. **Ache as travas que faltam.** Grepe a expansão por `MUT_MASK`, que é um const associado de verdade e vai estar lá ao pé da letra, e depois leia para fora dele para achar onde ele é testado contra as contas que o chamador mandou. Não grepe por um nome de função; os esboços acima nomearam um por legibilidade e o código emitido de verdade pode inlinar ele ou chamar ele de outra coisa. Depois coloque a sua trava 6 nativa ao lado dele e confirme a diferença em escopo: a sua compara um par, esta compara cada par que a máscara marca. Depois olhe o seu `invoke_signed` nativo e confirme que não tem trava de compilador nenhuma impedindo uma leitura obsoleta pós-CPI, o trabalho que o borrow do `CpiHandle` faz no V2.

![Uma planilha de cinco linhas com duas linhas trabalhadas e três em branco, emparelhando cada linha gerada com o passo nativo que ela substitui e se ela dispara em tempo de compilação ou em runtime.](assets/v08-table.webp)

5. **Escreva as anotações.** Nas suas próprias palavras, num comentário ao lado de cada uma de cinco linhas geradas, nomeie o passo nativo que ela substitui e escreva `compile-time` ou `runtime` ao lado. Checkpoint: você deve conseguir apontar para cada linha do seu `TryFrom` e do seu despacho nativos e achar a gêmea gerada dela, e apontar para exatamente duas travas geradas, a varredura de duplicatas e o modelo de borrow, que não têm gêmea nenhuma. Se você consegue fazer isso, você leu o framework.

## Challenge: a trava de três linhas

Solo, sem apoio. Pegue estas três linhas de uma expansão do V2. Para cada uma, declare qual passo nativo ela substitui (uma checagem de ordem de load, um hook de constraint, ou um despacho ou varredura de duplicatas) e se ela dispara em tempo de compilação ou em runtime.

```text
(a)  let state: Account<Vault> = Account::try_from(next_account_info)?;
(b)  const MUT_MASK: [u64; 4] = /* bits set for state, sol_vault, authority */;
(c)  /* the walk over every mutable pair, run in the dispatcher */
```

Escreva uma frase por linha. A distinção sobre a qual a trava gira é a entre (b) e (c): elas são a mesma feature em dois momentos diferentes. Se as suas três respostas se mantêm, e você consegue dizer por que a (b) é de tempo de compilação e a (c) é de runtime sem embaçar elas, você possui o diff.

Sem gabarito aqui. Três testes, se você quer se checar sem ser informado: para cada linha, você consegue nomear o arquivo e mais ou menos o número de linha no seu próprio vault nativo onde o equivalente mora, ou dizer honestamente que não tem nenhum? Você consegue dizer de qual informação a linha depende, a definição da struct ou as contas de verdade do chamador? E para a (b) e a (c) especificamente, você consegue explicar por que elas não conseguem as duas disparar no mesmo momento? Se as três respostas vêm fácil, você possui o diff. Se a terceira é a que fica presa, releia a seção do MUT_MASK, porque essa emenda é o ponto inteiro da trava.

Essa é a metade de reenquadrar do construa-duas-vezes, fechada. Você construiu o vault na mão, você diffou ele contra a máquina que gera ele, e nenhum dos dois é uma caixa preta mais. Você agora consegue prever o que o derive escreve e por quê, que quer dizer que você finalmente consegue fazer o framework carregar o peso inteiro dele em vez de uma instrução por vez. Na próxima lição você monta o salão de fliperama inteiro, um programa que roda o counter, o vault, o escrow e o swap por CPI, e você leva ele pelo ciclo de vida inteiro até um deploy verificado em devnet. Agora vá fazer ele carregar o salão inteiro.
