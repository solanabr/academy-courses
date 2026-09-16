# O que o V2 mata em tempo de compilação

Na lição passada você rodou o loop de medir-mudar-remedir no swap. O flip de guardrails mediu um delta de exatamente zero — a aresta de dependência do anchor-spl manteve as redes ligadas, e você atribuiu o zero em vez de entregar a flag — enquanto o `const-rent` e a cotação reescrita produziram deltas que você podia guardar, cada um comprovado com um número antes e um número depois. Você confiou no compilador para manter o programa correto enquanto você deixava ele mais rápido.

Agora transforme essa confiança em arma. O `quarter-vault` é o workspace no qual você está construindo desde o módulo 3 — ele mantém o nome que o `anchor init` deu para ele por causa do primeiro programa dele, e agora ele guarda o R2, o R3 e o R4 — e o swap de token-para-ticket dentro dele é o quarto degrau da escada do Quarters. Abra um terminal lá, corte um branch de exploit do seu R4 limpo, e escreva o primeiro ataque clássico de Anchor contra o seu próprio swap.

```bash
cd quarter-vault
git checkout -b exploit/compile-time-kills
```

Você vai autorar quatro ataques deliberados, um commit cada, contra o swap de token-para-ticket que você já construiu: o pool lido através de uma lente de config, dois slots mutáveis de reserva marcados com `dup` simples, uma reserva tipada lida enquanto uma CPI está em voo, e um bump de pool recomputado na mão. Depois você bate o `anchor build`. Três deles nunca produzem um binário. Um deles compila bem e não faz nada. Essa lacuna, entre "compila" e "explora", é a lição inteira.

Esta é a lição que era genuinamente impossível de ensinar antes de agosto de 2026. As quatro classes de vulnerabilidade abaixo precisavam de uma checagem em runtime, um teste, ou um auditor com um checklist. Nos defaults do V2, três delas morrem onde você escreve elas, em vermelho, em tempo de compilação. Você tem a chance de ver cada uma morrer.

Uma ressalva honesta antes de você escrever uma linha de maldade: este é um alpha não auditado. Tudo aqui é um ganho de tempo de compilação de verdade e nada disso é uma prova. Segure as duas coisas.

## Resumo

No fim desta lição você vai ter re-derivado quatro classes da taxonomia arquivada de segurança de programas da Solana Foundation (o curso de onze diretórios no qual uma geração inteira de auditores treinou) e re-rodado quatro delas contra os defaults do V2. Você vai demonstrar que três agora são erros de compilação em vez de riscos de runtime, e que a quarta compila mas não tem mais emenda nenhuma para puxar.

As quatro classes:

- **Type cosplay**: carregar os bytes de uma conta como um tipo diferente.
- **Duplicate-mutable**: passar a mesma conta gravável em dois slots para gastar ela duas vezes.
- **Aliasing de CPI / obsoleto-depois-de-CPI**: ler um campo tipado cujos bytes uma chamada entre programas está a ponto de mudar embaixo de você.
- **Recálculo de bump**: recomputar um bump de PDA para um errado passar.

O seu entregável é concreto: três erros de compilador nomeados, capturados com o texto da mensagem deles, mais uma rodada verde de `anchor test` depois de você restaurar o R4. O registro aqui é cautela, não celebração. O compilador é um aliado forte e uma desculpa ruim.

![Uma tabela de quatro linhas emparelhando cada classe de vulnerabilidade do Anchor com o risco de runtime dela no v1, o status dela nos defaults do V2, e o julgamento do qual o desenvolvedor continua sendo dono.](assets/v01-comparison.webp)

## As quatro classes, e por que três viram erros de tipo

Aqui está o recuo para o resto da lição, dito sem enfeite para você saber o que vem. Eu percorro a primeira classe, type cosplay, de ponta a ponta: o risco no v1, as correções ingênuas, e a razão exata de o default do V2 recusar ela. As duas seguintes, duplicate-mutable e aliasing de CPI, você termina a partir de stubs no Lab. A última classe você ataca por conta própria, com uma variante que ninguém te entregou, e você prevê o resultado antes de compilar. O apoio cai de propósito. É assim que você descobre o que de fato entendeu.

Uma palavra sobre de onde essas quatro vêm, para isto não ler como uma lista que eu inventei. A Solana Foundation rodou um curso de segurança de programas cujo repositório guarda onze diretórios de vulnerabilidade, uma classe por diretório, e uma geração de auditores aprendeu a taxonomia a partir dele. Type cosplay, duplicate-mutable, CPI arbitrária, checagens de owner e de signer faltando, substituição de conta, e o resto, cada um ganhou um programa vulnerável e um corrigido. O que a gente está fazendo nesta lição é pegar quatro daquelas onze e re-rodar elas contra os defaults do V2 para ver quais o framework agora responde por você. Quatro de onze. Mantenha essa razão à vista: ela é o escopo honesto do que um upgrade de compilador compra.

Comece com a pergunta que fica embaixo de todas as quatro: o que é uma conta, para um programa? No Anchor v1, uma conta chega como bytes crus mais um discriminator de oito bytes, e o `Account<'info, T>` desserializa aqueles bytes para dentro da sua struct depois de checar que o discriminator casa com `T`. Esse passo de desserializar é a emenda. Cada uma dessas quatro classes é um jeito de fazer o programa agir em cima de bytes que não são o que o tipo diz que eles são. A pergunta interessante não é "existe uma checagem", é "quando a checagem roda, e eu consigo pular ela por acidente". Uma checagem de runtime roda quando a transação roda, que é tarde, e ela roda só se o caminho de código alcançar ela. Uma checagem de tempo de compilação roda antes de um binário existir. Essa diferença de momento é o assunto inteiro desta lição.

### Type cosplay, percorrido de ponta a ponta

O status quo, no v1: a sua instrução espera um `FeeConfig`, o atacante te entrega um `Pool`, e se as duas structs por acaso se alinharem na memória, o seu programa lê os bytes do pool através da lente de config e confia em campos que querem dizer outra coisa completamente. A checagem de discriminator era o que parava a versão crua. A versão subtil passava quando dois tipos de conta compartilhavam um prefixo ou quando um programa usava `AccountInfo` e desserializava na mão sem checar.

A pergunta motivadora: se o tipo está fixo na definição da struct, por que o runtime é livre para me entregar os bytes errados?

Descarte as correções ingênuas primeiro, porque elas são o que o v1 entregou. Correção ingênua um: adicione um discriminator e cheque ele em cada load. Funciona, mas é uma checagem de runtime, que quer dizer uma checagem que você pode esquecer, desligar, ou contornar com `AccountInfo`. Correção ingênua dois: compare uma string de nome de tipo guardada no load. Mais lento, ainda runtime, e agora você está pagando para guardar um nome. As duas correções compartilham a mesma falha: elas detectam o descasamento depois de o programa já estar segurando uma referência tipada para a memória errada.

O V2 afia o requisito para dentro de algo que o compilador consegue impor. Nos defaults do V2, o `Account<T>` (note o lifetime largado) é uma **view zero-copy tipada como Pod** dos dados da conta. O `T` tem que implementar `Pod`, que quer dizer que ele não tem preenchimento e tem um layout completamente determinístico, e os bytes são convertidos direto para `T` em vez de parseados campo por campo. Esta é a mesma decisão de design pela qual a issue #4390 argumentou sob a bandeira "zero-copy account deserialization by default", que nomeou o antigo `Account<T>` de parse-no-load como "the slow path" e "the #1 performance complaint". O ponto que vale sentar com: Pod-por-padrão é uma jogada de segurança tanto quanto uma jogada de velocidade. Um layout determinístico e sem preenchimento é exatamente o que deixa "estes bytes são um `FeeConfig`" uma afirmação que o sistema de tipos consegue segurar em vez de uma afirmação que você re-checa em runtime.

![Um diagrama contrastando a checagem de discriminator em runtime do v1, que pode ser pulada, com o cast tipado como Pod em tempo de compilação do V2, onde o tipo da conta está fixo na struct e é rastreado pelo compilador.](assets/v02-diagram.webp)

Então, quando você escreve o cosplay, o descasamento de tipo não tem onde se esconder. Uma peça de montagem honesta primeiro, porque a forma do seu próprio programa importa aqui: o R4 entrega exatamente **um** tipo de conta, o `Pool` que você escreveu no m05-l2, e cosplay precisa de dois. Então o branch de exploit adiciona um segundo — um `FeeConfig` que o seu swap não tem e não vai crescer. Ele é um adereço, e nomear ele como um é parte da lição: o que o erro de compilação abaixo comprova é um fato sobre o sistema de tipos, não uma afirmação sobre um campo que o seu programa de fato guarda.

```rust
// programs/token-ticket-swap/src/lib.rs  (R4, clean) - your swap's ONLY account type
#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub arcade_mint: Address,  // 32
    pub ticket_mint: Address,  // 32
    pub bump: u8,              //  1
    pub _pad: [u8; 7],         //  7
}

// programs/token-ticket-swap/src/exploits.rs - the prop, exploit branch only.
// An admin-config shape is the classic cosplay target: it carries an authority
// worth stealing.
#[account]
#[derive(InitSpace)]
pub struct FeeConfig {
    pub authority: Address,  // 32
    pub fee_bps: u16,        //  2
    pub bump: u8,            //  1
    pub _pad: [u8; 5],       //  5
}
```

Agora o cosplay. Você segura o pool e tenta ler ele como um `FeeConfig` para levantar a authority:

```rust
// ATTACK 1: type cosplay - read the pool's bytes through the FeeConfig lens
pub fn cosplay(pool: &Account<Pool>) -> Address {
    let stolen: &FeeConfig = pool.as_ref(); // will not compile
    stolen.authority
}
```

O `Account<Pool>` é um `Slab`, e um `Slab` implementa `AsRef` para exatamente dois alvos — o `AccountView` cru e o `Address` — nunca para algum *outro* tipo Pod. Você pediu para ele um `&FeeConfig`, um impl que não existe. O compilador para seco:

```text
error[E0277]: the trait bound `Slab<Pool>: AsRef<FeeConfig>` is not satisfied
  --> programs/token-ticket-swap/src/exploits.rs
   |
   |     let stolen: &FeeConfig = pool.as_ref();
   |                                   ^^^^^^ the trait `AsRef<FeeConfig>` is not implemented
   |
   = help: the following other types implement trait `AsRef<T>`:
             `Slab<T, H>` implements `AsRef<AccountView>`
             `Slab<T, H>` implements `AsRef<Address>`
```

Isso é type cosplay convertido em `E0277`: a superfície de trait simplesmente recusa oferecer uma lente que não seja o tipo próprio da conta. Os bytes do pool nunca chegam a ser lidos através da struct errada, porque a struct errada é um tipo Pod diferente e os dois não se interconvertem. Note o que fez o trabalho: não uma checagem de runtime nova, mas o sistema de tipos comum do Rust, dado um layout determinístico para se segurar.

Agora seja honesto sobre o que aquele trecho fez e não fez, porque por si só ele comprova menos do que parece. Ninguém nunca entregou aquela linha. O `let x: &FeeConfig = y.as_ref()` num `&Pool` falha em compilar no v1, no 0.29, em qualquer Rust já escrito; é um erro de tipo, não um exploit. A linha está ali porque ela é a tentativa de *lavagem*, a coisa que um atacante tenta primeiro quando o wrapper tipado está no caminho, e ela mostra o wrapper tipado se mantendo. O ataque de verdade do v1 nunca escreveu aquela linha. Ele foi em volta do wrapper tipado por completo:

```rust
// ATTACK 1, the shape it actually shipped in: hand-read raw bytes through
// the wrong struct, so no wrapper and no discriminator is ever consulted.
pub fn cosplay_v1(any_account: &AccountInfo) -> Result<Pubkey> {
    let data = any_account.try_borrow_data()?;
    let cfg = FeeConfig::try_from_slice(&data[8..])?;  // is this REALLY a FeeConfig?
    Ok(cfg.authority)                                  // whatever bytes sat there, read as one
}
```

Entregue para ele um `Pool` e ele felizmente retorna os primeiros 32 bytes — `arcade_mint` — como uma `authority`, porque o `try_from_slice` decodifica o que for dado para ele. Note como o resultado parece plausível: um mint é um endereço bem-formado, então nada mais adiante fede até alguém assinar contra ele. É essa a classe. Duas coisas têm que valer para o V2 responder ela, e elas respondem em dois relógios diferentes. No *load*, uma conta `Pool` passada para um slot declarado `Account<FeeConfig>` é rejeitada pela checagem de discriminator, em runtime, exatamente como o `Account<T>` do v1 rejeitava ela: a tag diz `account:Pool` e o wrapper queria `account:FeeConfig`. Essa metade não é nova. O que *é* novo é que a saída de emergência no nível dos bytes fechou: com o `Account<T>` como uma view Pod não tem `try_from_slice` num slice solto para recorrer, e o cast de bytemuck para o qual você recorreria em vez disso, o `from_bytes::<FeeConfig>(&data[8..])`, é um cast que você tem que escrever deliberadamente, em cima de bytes que você não comprovou que são um `FeeConfig`, em código que um revisor consegue grepar numa passada. O discriminator sempre foi a trava de runtime. A contribuição do V2 é que o caminho comum não te oferece mais um jeito de contornar ele, que é por que o `E0277` acima é a falha interessante em vez de uma óbvia.

### Duplicate-mutable

Mesmo motor, emenda diferente. O gasto-duplo clássico: uma instrução pega duas contas graváveis e o atacante passa a *mesma* conta para as duas. O programa lê um saldo através de um nome, lê ele de novo através do outro, credita a primeira, debita a segunda, e escreve as duas de volta. As duas cópias em memória divergem, qualquer escrita que aterrisse por último ganha, e o crédito sobrevive enquanto o débito evapora. Isso é cunhar do nada. No v1 o dispatcher rodava uma checagem de duplicata para parar exatamente isso, mas a checagem era fácil de optar por sair por acidente, e muitos programas optaram, em geral recorrendo a um tipo de conta cru para raspar um constraint.

Tente imaginar isso no seu swap e uma coisa útil acontece: você não consegue. O `SwapArcadeForTickets` carrega `token::mint = mint_arcade` no `reserve_arcade` e `token::mint = mint_ticket` no `reserve_ticket`, e nenhuma conta de token guarda dois mints, então os dois slots graváveis de reserva são não-aliasáveis antes de a checagem de duplicata ter voto. Um constraint que você escreveu no m05-l2 por uma razão de precificação fechou esta porta por uma razão de segurança. É esse o estado honesto do R4, e é por isso que o ataque abaixo é uma struct de accounts *nova* que você adiciona no branch de exploit em vez de uma edição no swap: você tem que arrancar os pins para a emenda existir.

Nos defaults do V2, o conjunto de contas graváveis que uma instrução toca é um const associado de tempo de compilação, o bitset **`MUT_MASK`**. Optar por sair da proteção de duplicate-mutable é uma coisa de verdade que você às vezes precisa, e ela tem um nome: `unsafe(dup)`. Escrever `dup` simples sem `unsafe` é um erro de compilação duro cuja mensagem te diz a correção. Você não consegue nem construir a grafia unsafe por acidente, porque a grafia que parece segura não compila.

Leia aquela cilada com cuidado, porque é a que as pessoas lembram errado: a checagem de duplicata em runtime continua rodando no dispatcher. O V2 não deletou ela. O que o V2 adicionou é uma trava de compilador na frente da grafia unsafe, então você alcança a checagem de runtime só no caminho que você marcou explicitamente como unsafe. "O build está verde" agora quer dizer "eu não desliguei isto por typo".

![Um cartão de código anotado mostrando dois slots mutáveis de conta marcados com dup simples, o erro de compilação do V2 rejeitando isso, e a grafia unsafe(dup) exigida que o erro nomeia como a correção.](assets/v03-annotated-code.webp)

### Aliasing de CPI e a morte do `.reload()`

Esta é a classe que aposenta um hábito do v1 para o qual você tem memória muscular. No v1, se você lia o saldo de uma conta de token, depois fazia uma CPI que mudava aquele saldo, depois lia o campo de novo, você recebia o valor *obsoleto* a não ser que você lembrasse de chamar `.reload()`. O bug era invisível: o código parecia correto, o campo tinha um número plausível nele, e o número era simplesmente velho. Auditorias inteiras existiam para achar chamadas de `.reload()` faltando.

A pergunta motivadora: por que o programa tem permissão de segurar uma referência tipada através de uma chamada que muta os mesmos bytes?

Descarte as respostas do v1 em camadas, porque o ecossistema tentou todas elas. Camada um: lembre de chamar `.reload()` depois de cada CPI. Isto é disciplina, e disciplina é a coisa que falha às 2 da manhã sob um prazo. Camada dois: documente, coloque "always reload after CPI" no guia de contribuição. Documentação pega o leitor que lê ela. Camada três: escreva um linter que grepa CPIs sem um reload em seguida. Melhor, mas um linter modela um padrão, e no momento em que a CPI e a leitura estão em funções diferentes o padrão quebra e o linter fica calado. Todas as três camadas compartilham uma falha: elas tentam pegar um erro que o sistema de tipos já estava em posição de deixar impossível.

A resposta do V2 é um borrow, não um lembrete. Um **`CpiHandle`** é um handle com borrow rastreado para as contas que uma CPI vai tocar. Enquanto o handle está vivo, ele segura um borrow do Rust em cima daquelas contas, e acesso tipado aos mesmos dados não compila até o handle ser dropado. Você fisicamente não consegue ler o campo obsoleto, porque a leitura não compila enquanto a CPI está pendente. A classe inteira de obsoleto-depois-de-CPI colapsa dentro do borrow checker, que é a única parte do Rust que nunca esquece.

![Uma linha do tempo vertical da janela de borrow de um CpiHandle, marcando cada leitura tipada da reserva de ticket dentro dela como um erro de compilação, contra a leitura obsoleta do v1.](assets/v04-diagram.webp)

### Recálculo de bump, o que compila

A quarta classe é a interessante, porque ela não produz um erro. No v1, um programa que recomputava um bump de PDA em cada chamada, em vez de guardar o canônico, podia ser guiado para assinar com um bump não-canônico, e a família de recompute-o-bump-errado morava naquela emenda. Nos defaults do V2 a emenda é mais apertada, mas seja preciso sobre o mecanismo, porque é fácil exagerar: a macro pré-computa um bump como um const de tempo de compilação *só quando cada seed é um literal de string de bytes* — um `b"..."` escrito por extenso no derive e nada mais. A lista de seeds do seu pool lê `seeds = [POOL_SEED]`, e o `POOL_SEED` é um `const` nomeado, não um literal, então o pool perde essa otimização por um fio e a geração de código cai de volta para derivar durante a validação. (Se você quiser checar em vez de aceitar a minha palavra: o `lang-v2/derive/src/pda.rs` da tag fixada trava a coisa inteira no `seeds_as_byte_literals`, que casa com um literal de string de bytes e retorna `None` para uma expressão de caminho — os testes de unidade dele mesmo dizem isso por extenso.) O que o framework nunca faz, em nenhuma forma de seed, é aceitar um bump que você entrega para ele: a validação re-deriva o resultado canônico e compara.

Então quando você recomputa um bump na mão no seu exploit, ele compila. O `Address::find_program_address` é código comum. Mas o framework valida e assina contra a derivação canônica dele, então o seu valor recomputado é ou idêntico, caso em que você não mudou nada, ou diferente, caso em que a validação de PDA rejeita ele em runtime. O ataque compila e não vai a lugar nenhum. Mantenha esse resultado por perto, porque ele é a ponte para a próxima lição: compilar não é explorar, e existe um conjunto inteiro de classes onde código compila *e* drena um escrow.

![Um funil mostrando quatro ataques entrando no anchor build, três saindo como erros de compilação rejeitados, e só o ataque de bump emergindo como um binário.](assets/v05-flowchart.webp)

Esse conjunto é onde a honestidade mora, então deixe eu nomear a armadilha agora em vez de no fim.

Converter quatro classes em erros de compilação estreita a superfície de ataque. Isso não aposenta a auditoria. O V2 é um release candidate não auditado, e a postura de "defaults não são substituto para revisão" é uma que este curso afirma por autoridade própria — o projeto não vai dizer isso por você. Vá olhar e você acha o README da tag fixada batendo o tom oposto ("v2 is secure by default for users"), sem página de ressalva nenhuma atrás dele. É exatamente essa a superfície de marketing que um time cita para si mesmo enquanto pula a revisão, então o ceticismo tem que ser seu. A leitura errada confortável, "secure by default" ouvido como "secure", é exatamente como um time se convence a sair da revisão que pega tudo na próxima lição. Uma garantia de tempo de compilação é confiável só na medida do compilador que faz ela, e este compilador é um alpha. Trate as quatro mortes como afirmações de design que você verifica contra o RC fixado, não como provas. O V2 não é a bala de prata para segurança de programas; ele é um compilador muito bom com um changelog muito honesto.

Existe um risco de segunda ordem aqui que é pior que qualquer bug isolado. Um time que internaliza "o compilador pega os nossos bugs de segurança" revisa menos, e revisa menos precisamente na região onde o compilador está calado, que é a região de onde o dinheiro de fato sai. Então a disciplina é invertida em relação ao que ela parece: as classes que o compilador mata são as nas quais você pode gastar a menor atenção na revisão, e as classes que ele não consegue tocar são para onde o orçamento inteiro de auditoria deveria ir. Ganhos de tempo de compilação são uma realocação de onde você olha, não uma razão para olhar menos. A divisão vale manter em algum lugar onde você possa ver ela.

![Uma tabela de duas faixas separando as classes que os defaults do V2 pegam das classes de signer, de substituição e de lógica que compilam, rodam e continuam sendo trabalho do desenvolvedor.](assets/v06-table.webp)

Esse changelog vale uma olhada, porque ele modela a postura. O PR #4914, mergeado em 2026-08-13, revisou os benchmarks da manchete *para baixo*: economia de bytecode de 95% para 94%, e o ganho de compute de 9.9x para 8.8x, com a ressalva de que "This version is alpha and exact values can move as codegen, pinocchio, and tooling change." Cite os 8.8x como contexto de quanto mais rápido o caminho Pod roda, nunca como um número de segurança. A mesma honestidade que revisa um benchmark para baixo é a honestidade que proíbe tratar qualquer default do V2 como auditado.

Um nome para arquivar e não desenvolver: a classe de substituição de conta que você viu na faixa de baixo tem um caso de guerra canônico, o dreno do `.mint` faltando do Cashio, e esse é o território do curso de DeFi e RWA Engineering. A gente aponta para lá em vez de recontar, e a gente retoma a classe de substituição de conta em si na próxima lição.

## Lab: quatro ataques, um branch

Você está no `exploit/compile-time-kills`. Primeiro, fixe o toolchain, porque nada disso é real na linha V1.

```bash
# Install the Anchor V2 release candidate. Freshness note (2026-08-22):
# 2.0.0-rc.1 is the pinned RC for this course, tagged on the `anchor-next` branch. It is an
# UNAUDITED alpha. `avm` CANNOT install the V2 RC: it only tracks published GitHub releases,
# and there is no release object for the v2 tag. Install the CLI straight from the tag:
cargo install --git https://github.com/otter-sec/anchor.git --tag v2.0.0-rc.1 anchor-cli --locked --force
# macOS: prefix with CARGO_PROFILE_RELEASE_LTO=off if the release build fails to link.
# The tag is a fixed point (commit e4878b6d) where the branch head is not, which is what keeps
# this in step with m08-l2's verify Dockerfile. Do NOT verify V2 content on the 1.1.x line.
anchor --version   # expect the 2.0.0-rc.1 line, NOT anchor-cli 1.1.x
```

O recuo da ajuda começa aqui. O ataque 1 está feito para você, para você conseguir ver a forma de um erro capturado. Os ataques 2 e 3 chegam como stubs que você termina. O ataque 4 você já entende da derivação, então você só roda ele e lê o (não-)resultado.

**Passo 1: aterrisse o ataque de type cosplay e capture o erro.** Crie `programs/token-ticket-swap/src/exploits.rs`, cole o Ataque 1 da derivação acima, ligue ele no `lib.rs` com `mod exploits;`, e faça o build:

```bash
anchor build 2>&1 | tee /tmp/attack1.log
grep -A6 'E0277' /tmp/attack1.log
```

Você deve ver o bloco de `trait bound` nomeando `Slab<Pool>: AsRef<FeeConfig>` como não satisfeito, com a nota de help listando `AccountView` e `Address` como os únicos alvos de `AsRef` que um `Slab` oferece. Commite o estado que falha para o branch registrar a tentativa:

```bash
git add -A && git commit -m "attack 1: type cosplay (does not compile)"
```

Checkpoint: o `git log --oneline` mostra um commit, e o `/tmp/attack1.log` contém `E0277`. Se o build *teve sucesso*, você acidentalmente deixou as duas structs sendo o mesmo tipo, então re-cheque que o `Pool` e o `FeeConfig` são distintos.

**Passo 2: termine o stub de duplicate-mutable.** Esta é uma segunda struct de accounts no `exploits.rs`, não uma edição no `SwapArcadeForTickets` — como a derivação disse, os pins de `token::mint` do swap de verdade deixam os slots de reserva dele não-aliasáveis, então o ataque tem que largar eles para ter uma emenda. Complete o segundo slot para os dois serem mutáveis e os dois carregarem o opt-out de `dup` simples:

<!-- verify: expect-fail the V2 default rejects bare `dup`; that compile error IS this lesson's point -->
```rust
// STUB - finish this so both slots are `mut` and marked plain `dup`
#[derive(Accounts)]
pub struct DrainPool {
    pub trader: Signer,
    #[account(seeds = [POOL_SEED], bump = pool.bump)]
    pub pool: Account<Pool>,
    #[account(mut, dup)]
    pub reserve_a: InterfaceAccount<TokenAccount>,
    // TODO: add reserve_b as a second mutable InterfaceAccount<TokenAccount>,
    //       also marked plain `dup`
}
```

Faça o build, e confirme que o compilador nomeia a correção. Esta é a linha exata que o passo de verify da lição procura:

```bash
anchor build 2>&1 | grep -c 'unsafe(dup)'
# expect: at least 1
```

Checkpoint: a contagem é pelo menos 1. O erro te disse para escrever `unsafe(dup)`, e você vai deixar como `dup` simples, porque o ponto é a rejeição, não a correção. Commite como uma tentativa que falha.

**Passo 3: termine o stub de aliasing de CPI.** Você já rodou este experimento uma vez, no passo 3 do m05-l2, onde mover as leituras de reserva para dentro da janela do handle era um checkpoint. A mesma jogada, enquadrada como um ataque: complete o stub para uma leitura tipada da reserva de ticket ficar *entre* a criação do handle e o `invoke` dele:

```rust
// STUB - read reserve_ticket.amount() while the CpiHandles are still live
pub fn drain(ctx: &mut Context<SwapArcadeForTickets>) -> Result<()> {
    let push = TransferChecked {
        from: ctx.accounts.reserve_ticket.cpi_handle_mut(),
        mint: ctx.accounts.mint_ticket.cpi_handle(),
        to: ctx.accounts.trader_ticket.cpi_handle_mut(),
        authority: ctx.accounts.pool.cpi_handle(),
    };
    // TODO: read ctx.accounts.reserve_ticket.amount() HERE, while `push` still holds the handles
    token_interface::transfer_checked(
        CpiContext::new(ctx.accounts.token_program.address(), push),
        1,
        ctx.accounts.mint_ticket.decimals(),
    )?;
    Ok(())
}
```

Faça o build. O borrow checker rejeita a leitura com uma mensagem de classe `E0502`: o `reserve_ticket` está com borrow mutável pelo `CpiHandle` dentro do `push`, então você não consegue tirar uma segunda referência para ler o saldo dele. Capture isso, commite a tentativa que falha.

Checkpoint: o build falha num erro de borrow que nomeia o handle e o `reserve_ticket`. Se ele *compilou*, a sua leitura aterrissou depois de os handles saírem de escopo ou depois do `transfer_checked`, que é a ordem segura, então mova a leitura para cima.

**Passo 4: rode o ataque de bump e leia o não-resultado.** Este compila. Adicione o recálculo na mão e faça o build:

```rust
// ATTACK 4: hand-recompute the bump instead of trusting the one the pool stored
pub fn wrong_bump(ctx: &mut Context<SwapArcadeForTickets>) -> Result<()> {
    let (_pda, bump) = Address::find_program_address(&[POOL_SEED], &crate::ID);
    msg!("recomputed bump = {}, stored bump = {}", bump, ctx.accounts.pool.bump);
    Ok(())
}
```

```bash
anchor build   # this one succeeds
```

Checkpoint: o build está verde, e os dois bumps no log são iguais. Não tinha emenda nenhuma para explorar, só o bump canônico para re-derivar por conta própria em CU. Esse build verde é o ponto do exercício inteiro: ele compilou, e ele não drenou nada.

**Passo 5: restaure o R4 para limpo e comprove.** Tire os exploits de volta para fora e confirme que o swap ainda passa:

```bash
git checkout main -- programs/token-ticket-swap/src   # restore clean R4 source
rm -f programs/token-ticket-swap/src/exploits.rs
anchor test
```

Checkpoint: o `anchor test` está verde. O seu artefato de avaliação agora está completo: três erros de compilação capturados no branch de exploit (type cosplay, duplicate-mutable, aliasing de CPI) mais uma rodada verde de teste no R4 restaurado. O ataque de bump é o quarto commit registrado que compilou e não fez nada.

![Uma linha do tempo de commits de cinco nós: três ataques falhando em compilar, um compilando como um no-op de runtime, e um commit final restaurando a suíte verde.](assets/v07-timeline.webp)

## Challenge

O trabalho de completion é o Lab que você acabou de terminar: três ataques expressos a partir de stubs, três erros de compilador capturados, o R4 restaurado para verde. Agora o degrau solo, onde ninguém te entrega o ataque.

Escolha uma classe e escreva uma variante *nova* dela contra o swap. Alguns pontos de partida, mas invente o seu se um ocorrer para você:

- Um par diferente de contas com alias para a classe de CPI: segure uma leitura tipada do `reserve_arcade`, não do `reserve_ticket`, enquanto um handle em cima do `reserve_arcade` está vivo.
- Um cosplay na outra direção: leia um `FeeConfig` como um `Pool` e tente levantar o `ticket_mint` dele.
- Um duplicate-mutable através de três slots em vez de dois.

Antes de compilar, anote a sua previsão: o default do V2 mata isto em tempo de compilação, ou ele deixa passar? Depois faça o build e se cheque. A previsão é a parte avaliada, não a compilação. Se você consegue cantar o resultado antes de bater o build, você internalizou o mecanismo em vez de memorizar os quatro exemplos. Se a sua previsão estava errada, a pergunta interessante não é "qual é a correção" mas "qual dos quatro mecanismos eu entendi errado", e a seção de derivação é para onde você vai descobrir.

## Onde isso te deixa

Você acabou de fazer uma coisa que o ecossistema não conseguia fazer um mês atrás: você escreveu quatro exploits de livro-texto de Anchor contra o seu próprio programa e viu o compilador recusar três deles pelo nome. Isso é um estreitamento de verdade da superfície de ataque, e vale ficar genuinamente satisfeito com isso.

Agora segure a outra metade. Três ataques se recusaram a compilar. Mas você escreveu quatro, e um compilou bem. As classes das quais o compilador não consegue te salvar são as próximas, checagens de signer e de owner faltando, substituição de conta, os bugs de lógica e de aritmética, e essas são as que de fato drenam escrows. Um build verde é um piso, não uma linha de chegada. Traga o branch de exploit e o mesmo olho suspeito para a próxima lição, onde a gente ataca exatamente o que ainda morde, e onde "compilou" para de ser consolo nenhum.
