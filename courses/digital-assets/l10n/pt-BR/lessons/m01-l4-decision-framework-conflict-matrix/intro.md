# SPL clássico vs Token-2022: o framework de decisão + a matriz de conflitos

## Resumo

Em m01-l3 você aprendeu que o SPL clássico é uma interface congelada sobre o novo motor p-token, e que toda capacidade genuinamente nova mora nas extensões do Token-2022. O que levanta a pergunta em que toda conversa de produto acaba caindo: "esse mint pode ter uma taxa de transferência E saldos confidenciais?" Você esperaria uma tabela em algum lugar da documentação respondendo isso. Não existe tabela completa nem aplicada. Algumas páginas de documentação afirmam coisas sobre pares individuais (você vai levar uma dessas afirmações a julgamento no fim desta lição), mas nada autoritativo, nada inteiro. A resposta honesta mora numa função Rust chamada `check_for_invalid_mint_extension_combinations`, e nesta lição você vai lê-la.

Comece agora, antes da teoria. Clone o código-fonte e fixe ele no commit de que esta lição inteira é derivada:

```bash
git clone https://github.com/solana-program/token-2022.git
cd token-2022
git checkout 426400f
```

Abra `interface/src/extension/mod.rs` e role até a linha 1326. Essa função, umas cinquenta linhas de Rust puro, é todo o código legal das combinações de mint do Token-2022. Deixe ela aberta num painel dividido. Tudo abaixo é uma caminhada pelo que você já está olhando.

Duas coisas se constroem em cima dessa leitura. Primeiro, o framework de decisão: quando os requisitos de um produto apontam para o SPL clássico e quando eles forçam o Token-2022, formulado como uma escolha de CONJUNTO de extensões em vez de uma preferência de programa. Segundo, o artefato emblemático deste curso: `check-combo`, um validador em TypeScript que aplica as cinco regras da fonte a qualquer conjunto de extensões e devolve um veredicto com a regra que disparou. Ele consome o inspetor `decode-mint` que você construiu em m01-l2, o que significa que, no fim desta lição, você consegue apontar ele para qualquer mint ao vivo e perguntar "esse conjunto é sequer legal, e por quê?"

O recuo da ajuda nesta lição: a leitura das regras na fonte é trabalhada por inteiro, eu caminho por cada linha com você. O validador é um exercício de completion: a regra 4 já vem implementada, as regras 1, 2, 3 e 5 são suas para portar. E o único coding challenge do módulo, no fim, é solo: o validador completo provado por testes, sem apoio.

Esta é uma lição-chave. Todo mint que a gente construir do Módulo 2 em diante passa pela cancela do artefato que você escreve hoje.

## Derivando a matriz

### O framework: qual conjunto de extensões, não qual programa

A decisão sobre a qual as pessoas discutem costuma ser formulada errado. "A gente deveria usar o SPL clássico ou o Token-2022?" soa como preferência de marca, como escolher fornecedor de banco de dados. Não é. E desempenho é um diferencial ruim para se discutir a partir dele: a troca pelo p-token sobre a qual você leu na lição passada tirou de cena o motor antigo do SPL clássico, e os custos de compute por extensão do Token-2022 não são publicados, que é por isso que uma lição mais adiante mede eles num lab em vez de citar uma tabela. Até você ter medido, o argumento que de fato decide é exatamente uma pergunta:

Algum requisito do seu produto nomeia um comportamento que só uma extensão oferece?

Se não, o SPL clássico vence por padrão. É a conta mais barata de criar, toda carteira renderiza ele, toda DEX roteia ele, toda integração já escrita assume ele. Uma interface congelada não é uma limitação quando seus requisitos cabem dentro dela. É uma garantia.

E mais barato aqui é mensurável, não retórico. Seu inspetor de m01-l2 já imprimiu os números para você: um mint clássico simples tem 82 bytes, e no instante em que uma única extensão existe o mint troca para o layout estendido, onde o menor mint estendido possível que as suas próprias asserções de `mintLen` mediram é 170 bytes. Mais ou menos o dobro, por uma extensão, antes de você ter configurado qualquer coisa. Daí para cima ele sobe a cada entrada TLV. Rent é cobrado sobre bytes, e o mint é a metade barata dessa história de qualquer jeito, porque as extensões de conta forçadas que a gente vê mais adiante nesta lição colocam bytes em toda conta de holder que você um dia criar. Algumas dezenas de bytes vezes cem mil holders deixa de ser preferência de design, vira uma linha numa planilha de tesouraria. Então usar o SPL clássico por padrão é a resposta certa sempre que a lista de requisitos não forçar o contrário, e "a gente pode querer uma taxa um dia" não é um requisito.

Se sim, você está no Token-2022, e a decisão de verdade começa. Taxas na transferência, exibição de juros, saldos confidenciais, transfer hooks, intransferibilidade, metadados nativos, pausabilidade: cada uma dessas palavras numa spec de produto é um nome de extensão disfarçado. No instante em que uma delas aparece, a pergunta deixa de ser "qual programa" e vira "qual conjunto de extensões", porque os slots de extensão de um mint são reivindicados na inicialização. A regra precisa, que o módulo 2 vai demonstrar em mints ao vivo: toda extensão de poder do lado do mint é só-na-init — uma taxa, um hook, um delegado não podem ser aparafusados depois. As exceções são estreitas e todas do mesmo formato. O TLV do TokenMetadata é escrito depois do `initialize_mint`, mas só atrás de um MetadataPointer que você escolheu na criação; os TLVs de group e de group-member pegam o mesmo caminho ponteiro-e-depois-realloc atrás dos ponteiros deles; e as extensões em nível de conta pousam nas contas de token dos holders via reallocate, nunca no mint. Repare que cada uma delas ainda começa com um ponteiro escolhido no nascimento, e que nenhuma delas é uma extensão de poder. Então, para efeito de design, o conjunto de extensões de um mint é uma certidão de nascimento, não uma página de configurações.

![Fluxograma com um único critério sobre comportamento exclusivo de extensão, o SPL clássico como o ramo do não, e um ramo do sim que rascunha um conjunto de extensões, valida ele contra as cinco regras da fonte e então inicializa permanentemente.](assets/v01-flowchart.webp)

Duas consequências caem dessa permanência, e elas moldam a lição inteira. Uma: o conjunto de extensões tem que ser projetado de saída, contra o roadmap completo do produto, porque "a gente adiciona transferências confidenciais no próximo trimestre" não existe. O conserto para uma extensão faltante é um mint novo e uma migração, que é uma lição inteira de dor no Módulo 9. Duas: o conjunto tem que ser LEGAL, e a legalidade é decidida pelo programa na inicialização, não por você e não pela documentação. Algumas combinações são rejeitadas de cara. Algumas combinações forçam outras extensões a virem junto. O mapa completo dessas interações é o que este curso chama de matriz de conflitos.

Então cadê a matriz? Aqui está a parte que garante a esta lição o lugar dela no módulo.

### Por que não existe matriz para copiar

Não existe matriz de conflitos oficial e atual para os 29 tipos de extensão do Token-2022. Eu fui procurar, o passo de pesquisa deste curso foi procurar, e o resultado honesto é: as cinco regras que decidem a legalidade moram no `interface/src/extension/mod.rs` do `token-2022`, linhas 1326-1374, e em nenhum outro lugar (solana-program/token-2022 @ 426400f, um commit datado de 2026-08-17 e lido para esta lição em 2026-08-22).

Os catálogos que existem são páginas explicativas por extensão, e nenhuma delas publica as regras. Elas também se mexem debaixo de você, o que eu consigo demonstrar com um vexame meu. Quando o passo de pesquisa deste curso varreu os sites de documentação em 2026-08-21, o solana.com/docs/tokens/extensions não tinha página para PermissionedBurn, a variante mais nova, enquanto o solana-program.com já tinha uma. Eu escrevi essa lacuna num rascunho anterior desta lição como o exemplo principal da deriva da documentação. Rechecando as duas páginas em 2026-08-22 antes de entregar: o solana.com lista Permissioned Burn na barra lateral de extensões dele, logo depois de Pausable Mint. A lacuna fechou dentro de um dia, e a minha observação "atual" sobre uma página de documentação estava obsoleta antes de a lição em que ela morava ficar pronta.

Guarde isso, porque é a lição mais útil e não é a que eu me propus a ensinar. Uma afirmação de catálogo copiada não apodrece em anos; ela apodreceu em vinte e quatro horas, e o único motivo de você não estar lendo a versão errada agora é que alguém rerodou a checagem. O enum da fonte é a coisa que não faz isso com você: o `ExtensionType` tem 29 variantes de produção, PermissionedBurn entre elas, mais 3 variantes só-de-teste escondidas atrás de um flag `cfg(test)` que nunca são entregues, e essa afirmação carrega um hash de commit para que você consiga dizer exatamente qual mundo ela descreve. Uma página de documentação descreve o que quer que ela descreva hoje, sem nenhum carimbo de versão em lugar nenhum dela. Código é a verdade, e mais importante, código é a verdade *num momento nomeável*.

Não aceite o 29 de mim também. Você tem o repo aberto no commit fixado, então role para cima no mesmo arquivo até o enum `ExtensionType` e conte as variantes você mesmo. Subtraia as que ficam atrás de atributos `#[cfg(test)]`, e você pousa em 29. Trinta segundos contando, e agora você tem na mão o número sobre o qual as duas páginas de documentação não conseguiram concordar, com um hash de commit anexado. Essa jogada, checar o enum em vez de citar uma página, é a menor versão possível de tudo o que esta lição faz.

Os antigos desenhistas de mapas tinham um nome para esse modo de falha, meio que ao contrário: ruas-armadilha. Cartógrafos desenhavam uma rua falsa nos mapas deles para que, quando o mapa de um rival mostrasse a mesma rua falsa, a cópia estivesse provada. A deriva da documentação é o mesmo mecanismo rodando para frente: uma página omite uma extensão, tutoriais rio abaixo copiam a página, e logo o modelo mental de metade do ecossistema está sem uma variante de produção real. Ninguém plantou o erro de propósito. A cópia propagou ele mesmo assim. A única defesa é a que as vítimas dos cartógrafos nunca tiveram: você pode ir levantar o território você mesmo, porque o território é um repositório Git público.

![Uma comparação mostrando a mesma página de documentação omitindo PermissionedBurn num dia e listando ele no dia seguinte, ao lado do enum da fonte fixado por commit, cujas 29 variantes de produção continuam como registradas.](assets/v02-comparison.webp)

É por isso que a habilidade emblemática deste curso é derivação e não memorização. Uma lista de conflitos copiada tem uma data de validade impressa em tinta invisível. Uma derivada carrega a própria proveniência: esta matriz é verdadeira no commit 426400f, e aqui está a função de onde ela veio, e aqui está como re-derivar ela quando o commit se mover.

Com isso em mente, vamos ler a função.

### Lendo as cinco regras na fonte

Aqui está o coração da coisa, na íntegra de `interface/src/extension/mod.rs` em 426400f. A função coleta sete booleanos da lista de extensões do mint proposto e então roda cinco travas:

```rust
/// Check for invalid combination of mint extensions
pub fn check_for_invalid_mint_extension_combinations(
    mint_extension_types: &[Self],
) -> Result<(), TokenError> {
    let mut transfer_fee_config = false;
    let mut confidential_transfer_mint = false;
    let mut confidential_transfer_fee_config = false;
    let mut confidential_mint_burn = false;
    let mut interest_bearing = false;
    let mut scaled_ui_amount = false;
    let mut non_transferable = false;

    for extension_type in mint_extension_types {
        match extension_type {
            ExtensionType::TransferFeeConfig => transfer_fee_config = true,
            ExtensionType::ConfidentialTransferMint => confidential_transfer_mint = true,
            ExtensionType::ConfidentialTransferFeeConfig => {
                confidential_transfer_fee_config = true
            }
            ExtensionType::ConfidentialMintBurn => confidential_mint_burn = true,
            ExtensionType::InterestBearingConfig => interest_bearing = true,
            ExtensionType::ScaledUiAmount => scaled_ui_amount = true,
            ExtensionType::NonTransferable => non_transferable = true,
            _ => (),
        }
    }

    if confidential_transfer_fee_config && !(transfer_fee_config && confidential_transfer_mint)
    {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if transfer_fee_config && confidential_transfer_mint && !confidential_transfer_fee_config {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if confidential_mint_burn && !confidential_transfer_mint {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if scaled_ui_amount && interest_bearing {
        return Err(TokenError::InvalidExtensionCombination);
    }

    if non_transferable && confidential_transfer_mint && !confidential_mint_burn {
        return Err(TokenError::InvalidExtensionCombination);
    }

    Ok(())
}
```

Repare no que NÃO está aqui antes de a gente numerar o que está. Só sete dos 29 tipos de extensão sequer aparecem no match. MetadataPointer, PermanentDelegate, TransferHook, MintCloseAuthority, PermissionedBurn: nenhum deles participa de regra de conflito nenhuma. A esmagadora maioria dos pares de extensão é simplesmente legal, o que já é por si só uma descoberta que você não conseguiria de uma documentação que nunca publicou a função. A matriz é quase toda verde, com cinco linhas vermelhas atravessando um canto dela.

Agora as cinco travas, em ordem de fonte, cada uma com o seu porquê. A numeração é nossa, a lógica é deles.

**Regra 1: ConfidentialTransferFeeConfig exige AMBOS TransferFeeConfig E ConfidentialTransferMint.** A extensão de taxa confidencial é uma ponte. Ela existe para fazer a arrecadação de taxas funcionar quando os valores estão criptografados, então um mint carregando a ponte sem as duas pontas dela é incoerente: ou não haveria taxa para arrecadar, ou não haveria criptografia sob a qual arrecadar. O programa se recusa a inicializar uma ponte para lugar nenhum.

**Regra 2: TransferFeeConfig mais ConfidentialTransferMint juntos EXIGEM ConfidentialTransferFeeConfig.** Este é o espelho da regra 1, e é a direção mais interessante. Pense no que aconteceria sem ela. As transferências de um mint com taxa têm que reter uma porcentagem do valor. As transferências de um mint confidencial criptografam o valor. Uma transferência que é as duas coisas não consegue computar a taxa retida sobre um número que ela não tem permissão para ver, a menos que um mecanismo dedicado cuide de taxas no domínio criptografado. Esse mecanismo é exatamente o ConfidentialTransferFeeConfig. Então o par não é proibido, só incompleto, e o conserto é aditivo: adicione a terceira extensão e o conjunto fica legal. Guarde essa distinção, porque "esses dois conflitam" e "esses dois exigem um terceiro" parecem idênticos numa mensagem de rejeição e querem dizer coisas completamente diferentes para o seu produto.

**Regra 3: ConfidentialMintBurn exige ConfidentialTransferMint.** O ConfidentialMintBurn torna as mudanças de supply confidenciais. Supply confidencial num mint cujos saldos e transferências são todos públicos não protegeria nada: os deltas de cunhagem e de queima seriam reconstruíveis a partir das movimentações visíveis de conta em volta deles. A dependência corre num sentido só. Transferências confidenciais sem supply confidencial é um produto ótimo (saldos escondidos, emissão pública, é exatamente isso que a maioria das stablecoins confidenciais quer). Supply confidencial sem transferências confidenciais é uma porta de mosquiteiro num submarino.

**Regra 4: ScaledUiAmount e InterestBearingConfig são mutuamente exclusivos.** Essas são as únicas duas extensões que reescrevem como um valor bruto é EXIBIDO, cada uma aplicando a própria matemática de multiplicador. Dois multiplicadores de exibição num mint só significa que toda carteira e todo indexador têm que responder "qual deles vence, e em que ordem?", e qualquer resposta seria arbitrária. O programa se recusa a criar a ambiguidade. Esta é a única exclusão mútua de verdade na matriz inteira: não é uma peça faltando, não é uma companheira forçada, são só duas extensões que nunca podem dividir um mint.

**Regra 5: NonTransferable mais ConfidentialTransferMint é inválido A MENOS QUE ConfidentialMintBurn também esteja presente.** A regra mais estranha e a minha favorita, porque dá para sentir a conversa de design por trás dela. Um token não transferível (soulbound) com saldos confidenciais: o que isso esconderia, afinal? As transferências são a coisa que a criptografia protege, e não existe nenhuma. Mas adicione o ConfidentialMintBurn e o conjunto entra em foco: os valores de cunhagem e de queima agora são a superfície confidencial. Um emissor consegue operar uma credencial soulbound cujos tamanhos de emissão são privados. Então a trava é condicional: o par sozinho é sem sentido, o trio é um produto.

![Tabela das cinco regras de conflito com as extensões envolvidas, cada restrição e a classe dela (ponte, aditiva, dependência de mão única, exclusão mútua, condicional), citando as linhas da fonte fixada.](assets/v03-table.webp)

Mais uma coisa que a fonte te conta e que nenhuma página de documentação teria. Olhe o que cada trava de fato retorna: `Err(TokenError::InvalidExtensionCombination)`, o mesmo erro, cinco vezes seguidas. O programa nunca te diz qual regra você quebrou. Ele te diz que você quebrou uma. On-chain isso aparece como um número de erro customizado de programa dentro de uma transação que falhou, sem número de regra, sem nome de extensão e sem nenhuma pista sobre se o seu conjunto era contraditório ou apenas incompleto. Que é precisamente a lacuna que o `check-combo` existe para fechar. O validador que você está prestes a escrever retorna um `reason` nomeando a trava que disparou, para que uma revisão de design receba uma frase em vez de um código de erro, e receba isso antes de alguém gastar um lamport. Mesma lógica, diagnóstico melhor. Segure isso quando você escrever as strings de reason no lab: elas são o argumento inteiro para rodar um validador local em vez de mandar a transação e ler os destroços.

Leia as cinco como um conjunto e um padrão aparece: quatro das cinco orbitam a suíte de transferências confidenciais. Isso não é acidente. Criptografia é a única capacidade que muda o que as OUTRAS extensões conseguem saber, então é a única capacidade que gera lei entre extensões. Taxas precisam ver valores, auditorias de supply precisam ver cunhagens e queimas, soulbound precisa de algo que valha a pena esconder. Se você não guardar mais nada da teoria, guarde: confidencial é o centro gravitacional da matriz de conflitos, e a regra 4 é a exceção solitária, uma colisão de matemática de exibição sem criptografia nenhuma por perto.

### Três briefs de produto, passados pela máquina

Framework mais regras é o loop de avaliação inteiro, então rode ele quente três vezes antes de a gente construir qualquer coisa. Leia cada brief, traduza as frases de requisito em nomes de extensão, e então deixe as cinco regras julgarem o conjunto rascunhado. Esta é exatamente a conversa que você vai ter em toda revisão de design do Módulo 2 em diante.

Brief um: SPROUT, a moeda in-game que as nossas construções Overgrowth começam a cunhar no próximo módulo. A spec diz que holders pagam um pequeno imposto em toda transferência, que os saldos devem exibir o rendimento que o co-op paga sobre o grão estocado, e que o token carrega o próprio nome e símbolo sem nenhum programa de metadados externo. "Imposto em toda transferência" é TransferFeeConfig. "Exibe rendimento" é uma das duas extensões de exibição, InterestBearingConfig ou ScaledUiAmount, um assento só pela regra 4, e a escolha entre elas é a conversa de design do próximo módulo. "O próprio nome e símbolo" é MetadataPointer mais TokenMetadata, com o ponteiro mirando o próprio mint. Nada mais na spec nomeia um comportamento, então o conjunto rascunhado são esses quatro (com o assento de exibição preenchido por exatamente um ocupante), e repare que o SPL clássico morreu na primeira frase: a palavra "imposto" sozinha forçou o Token-2022. Rode as regras: nenhuma trava dispara contra o conjunto, então ele é legal como rascunhado. Este é o caso do dia a dia, e o núcleo taxa-mais-metadados dele é o `expectValid` no topo do seu arquivo de teste.

O passo de tradução, escrito por extenso uma vez para você ver o formato dele:

```text
SPROUT spec phrase              -> extension name
"tax on every transfer"         -> TransferFeeConfig
"displays yield"                -> InterestBearingConfig OR ScaledUiAmount (rule 4: one seat)
"own name and symbol"           -> MetadataPointer + TokenMetadata (pointer aimed at the mint)
verdict: no guard fires         -> legal as drafted
```

Brief dois: uma stablecoin de folha de pagamento para um estúdio que paga colaboradores on-chain, em que salários não podem ser legíveis por todo colega, e o emissor quer uma pequena taxa de transferência para financiar as operações. "Salários não legíveis" é ConfidentialTransferMint. "Taxa de transferência" é TransferFeeConfig. Conjunto rascunhado: esses dois. As regras dizem não: a regra 2 dispara, porque uma taxa não pode ser computada sobre um valor que o programa não tem permissão para ver sem um mecanismo para taxas no domínio criptografado. O conserto é aditivo. Adicione o ConfidentialTransferFeeConfig e o trio é legal. Ninguém teve que largar um requisito; o conjunto estava incompleto, não contraditório.

Brief três: uma credencial de harvest soulbound, não transferível por design, em que o emissor quer manter os tamanhos de emissão privados. "Não transferível" é NonTransferable. "Tamanhos de emissão privados" puxa a suíte confidencial, então o rascunho ingênuo é NonTransferable mais ConfidentialTransferMint. A regra 5 rejeita ele, e agora você sabe por quê: sem transferências para criptografar, o par não protege nada. Adicione o ConfidentialMintBurn, que é onde a privacidade de fato mora para este produto, e o trio passa. Mais uma reviravolta já que estamos aqui: se essa credencial também quisesse exibir um saldo com rebase, ela leva ScaledUiAmount OU InterestBearingConfig, nunca os dois. A regra 4 não tem conserto aditivo. Alguém na revisão de design tem que escolher, e é melhor que esse alguém seja você, hoje, do que um erro `InvalidExtensionCombination` no dia do lançamento.

![Tabela passando três briefs de produto pelo framework, mostrando requisitos traduzidos em conjuntos de extensões, a regra julgando cada rascunho e os consertos aditivos que tornaram duas linhas legais.](assets/v04-table.webp)

Três briefs, duas rejeições, zero requisitos largados. Essa proporção é o ponto. A maior parte do que as regras fazem na prática não é proibir produtos, é te dizer qual terceira extensão o seu par esqueceu, e uma mensagem de rejeição que nomeia a regra dela transforma um mistério de dia de lançamento num item de pauta de revisão de design.

### A camada dos pares forçados

As cinco regras respondem "estas extensões de mint podem coexistir?" Uma segunda função no mesmo arquivo responde uma pergunta diferente: "dado este mint, o que toda CONTA de token precisa carregar?" Ela se chama `required_init_account_extensions`, fica na linha 1296, e é uma consulta direta:

```rust
fn required_init_account_extensions(&self) -> &'static [Self] {
    match self {
        ExtensionType::TransferFeeConfig => &[ExtensionType::TransferFeeAmount],
        ExtensionType::NonTransferable => &[
            ExtensionType::NonTransferableAccount,
            ExtensionType::ImmutableOwner,
        ],
        ExtensionType::TransferHook => &[ExtensionType::TransferHookAccount],
        ExtensionType::Pausable => &[ExtensionType::PausableAccount],
        #[cfg(test)]
        ExtensionType::MintPaddingTest => &[ExtensionType::AccountPaddingTest],
        _ => &[],
    }
}
```

Quatro pares de produção. O TransferFeeConfig força o TransferFeeAmount em toda conta, porque taxas retidas têm que se acumular em algum lugar por holder. O NonTransferable força dois: NonTransferableAccount, mais ImmutableOwner para que um holder não consiga driblar o caráter soulbound reatribuindo a posse da conta (transferir o recipiente em vez do token: uma brecha que o par fecha no nascimento). O TransferHook força o TransferHookAccount, o flag por conta que o seu programa de hook lê. O Pausable força o PausableAccount. E ali no meio de código de produção tem um braço `#[cfg(test)]`: uma das três variantes só-de-teste que você excluiu da conta de 29, visível exatamente onde a documentação nunca mostraria ela para você.

Ponha as duas funções lado a lado e você finalmente consegue dimensionar o orçamento de restrições do programa inteiro. Sete das 29 variantes de produção aparecem nas travas de conflito. Quatro aparecem na consulta de pares forçados. Duas delas, TransferFeeConfig e NonTransferable, aparecem nas duas, então nove extensões distintas carregam alguma lei entre extensões e as outras vinte se combinam livremente com tudo. Esse é um design notavelmente permissivo, e vale dizer em voz alta porque a expressão "matriz de conflitos" faz as pessoas imaginarem um campo minado. Não é um campo minado. É um campo quase todo aberto com nove pedras marcadas nele, e agora você sabe onde cada uma delas está.

![Barra empilhada dividindo as 29 variantes de extensão de produção em 20 sem restrição, 5 só nas travas de conflito, 2 só na consulta de pares forçados e 2 nas duas.](assets/v05-chart.webp)

Repare que as duas funções respondem perguntas em camadas diferentes, e que a segunda morde mais tarde. As cinco regras rodam na inicialização do mint, então um conjunto ilegal falha imediatamente e ruidosamente, uma vez, na frente da pessoa que o escolheu. Os pares forçados rodam na inicialização da conta de token, que acontece toda vez que um holder novo aparece, potencialmente meses depois de o mint ser entregue e muito depois de aquela pessoa ter seguido em frente. Um cliente que aloca espaço de conta a partir de um comprimento hardcoded em vez de consultar esta tabela se safa por exatamente o tempo em que o mint não carregar extensões forçadas, e aí começa a falhar em usuários reais no dia em que alguém habilita uma taxa de transferência num mint novo e copia o código de cliente antigo para lá. Isso é um ticket de suporte sem autor óbvio. Leia o comprimento da tabela, nunca da memória.

Esta camada importa para custo e para decodificação. Toda extensão de conta forçada é bytes em cada uma das contas de holder, rent pago por usuário, para sempre. Quando o `decode-mint` de m01-l2 te mostrar uma conta de holder mais gorda do que você esperava, esta tabela é o motivo. No validador, a gente entrega ela como dado em vez de lógica: o `check-combo` exporta a tabela para que lições posteriores possam precificar a criação de conta antes de se comprometer com um design de mint.

### O trade-off, e o que um check verde não quer dizer

Toda lição deste curso nomeia seu trade-off, e esta tem dois, os dois afiados.

Primeiro: uma matriz derivada da fonte é exatamente certa e exatamente perecível. O que você derivou está correto no commit 426400f e pode se mover na próxima vez que o enum crescer ou a função de regras mudar. Vinte e nove variantes hoje era um número menor não faz muito tempo, e PermissionedBurn é prova de que o enum ainda cresce. Então a habilidade que você está comprando nesta lição não é a matriz. A matriz é um subproduto. A habilidade é a derivação, repetível contra qualquer commit que estiver fixado no dia em que você precisar. Quando uma regra muda lá em cima, os testes do seu validador falham contra a fonte nova, e essa falha É o alerta que a documentação nunca teria te mandado.

Porque a re-derivação é a habilidade durável, aqui está o loop inteiro como procedimento, para que daqui a seis meses ele te custe uma noite e não uma escavação arqueológica:

1. Escolha o novo pin. No seu clone: `git fetch origin && git log --oneline -3 origin/main`, escolha o commit, anote o hash e a data dele ao lado dos antigos.
2. Faça o diff só do arquivo que importa: `git diff 426400f..<newpin> -- interface/src/extension/mod.rs`. O `--` é o que diz ao git que o resto é um caminho e não outra revisão, e deixar ele de fora é como você consegue um `unknown revision or path not in the working tree`. A maior parte da atividade lá em cima nunca toca neste arquivo, e um diff vazio significa que a sua matriz continua atual, resolvido em dois minutos.
3. Se o enum `ExtensionType` mudou: reconte as variantes de produção (subtraia as `#[cfg(test)]`), e cheque se alguma variante nova aparece na função de travas ou na consulta de pares forçados.
4. Se o `check_for_invalid_mint_extension_combinations` mudou: releia as travas em ordem de fonte, porte o delta para o `check-combo.ts` e atualize as strings de `reason` para que as tags continuem batendo com as regras.
5. Rerode `npx tsx test-check-combo.ts`, adicione um caso de teste para qualquer coisa nova e atualize o hash fixado no comentário de header do seu arquivo. O comentário de header é a proveniência; um validador que não diz qual commit ele modela é uma página de documentação esperando para acontecer.

![Linha do tempo desde derivar e fixar a matriz no commit 426400f, passando por um evento sem data de deriva lá em cima, até o loop curto de diff-portar-retestar que a fixa de novo, rotulado como uma noite de trabalho.](assets/v06-timeline.webp)

Segundo, e este aqui é a cancela de um módulo inteiro lá na frente: as regras do código são necessárias, mas não completas. A documentação sinaliza combinações que a função de init nunca rejeita. NonTransferable mais TransferFeeConfig é o exemplo canônico: páginas de documentação chamam o par de incompatível (uma taxa sobre transferências que não podem acontecer), mas as cinco regras extraídas não dizem nada sobre isso, e regras são a única coisa que o programa aplica na init. Então qual afirmação vence? Não aceite a minha resposta para isso, e não aceite a resposta da documentação também. Esta é uma pergunta que tem um terminal, e você tem um. Você vai resolver isso você mesmo no Challenge perguntando direto ao programa, porque "a documentação diz" e "o código aplica" são classes diferentes de afirmação e o ponto inteiro é que agora você sabe como testar a lacuna entre elas.

Mantenha as duas classes de afirmação separadas com nomes: init-inválido significa que o programa rejeita o conjunto, ponto final, é isso que o check-combo detecta. Não-faz-sentido significa uma advertência em nível de documentação que pode ou não corresponder a alguma aplicação. E depois dessas duas fica uma terceira cancela que nenhum validador consegue ver: a aceitação por venue. Um conjunto pode ser perfeitamente legal de inicializar e ainda assim não ser roteável, porque allowlists de DEX rejeitam extensões como PermanentDelegate por política, não por legalidade. Legalidade é um piso, não roteabilidade. Essa frase é a ponte para o Módulo 5, onde a gente desmonta legal-mas-não-roteável direito.

![Três cancelas ficam entre um conjunto de extensões e um token negociável: a legalidade de init aplicada pelo código, que o check-combo cobre, depois as advertências de documentação não aplicadas, depois a política de aceitação por venue.](assets/v07-diagram.webp)

Essa é a teoria inteira. Um framework de decisão de um critério só, cinco regras lidas da fonte, quatro pares forçados e um mapa honesto do que as regras não cobrem. Agora a gente torna isso executável.

## Lab: portar as cinco regras para o check-combo

O artefato é o R2 na escada deste curso: uma função pura, sem RPC, sem dependências de chain, o que é deliberado. Regras extraídas da fonte deveriam ser testáveis em quatro milissegundos sem rede. A chain entra só no fim, quando o R2 consome o R1.

1. Faça o scaffold do lab ao lado do seu trabalho de m01-l2 e instale o runner. Duas dependências de dev, zero dependências de runtime, e esse é o footprint inteiro (tsx 4.20.5, o mesmo pin que m01-l1 introduziu, mais @types/node 26.2.0; as duas verificadas em 2026-08-22, e as duas valendo uma rechecada quando você fizer o scaffold):

```bash
mkdir -p labs/m01-l4 && cd labs/m01-l4
npm init -y
npm pkg set type=module
npm install -D tsx@4.20.5 @types/node@26.2.0
```

O tsx roda TypeScript direto sem passo de build; as definições de tipo do Node estão ali para que as chamadas de `process.exit` no script de teste passem no typecheck no seu editor em vez de brilhar em vermelho para você. Nada mais é instalado neste lab, e essa ausência é deliberada: regras extraídas da fonte deveriam ser prováveis sem um único pacote que fale com uma chain.

A linha `type=module` não é cerimônia. O script de cola do passo 7 usa top-level await, o `npm init -y` deixa o pacote em CommonJS por padrão, e o tsx recusa top-level await sob CommonJS com um erro de transform do esbuild que não nomeia nenhum desses dois fatos. Se você um dia vir `Top-level await is currently not supported with the "cjs" output format` no meio do curso, esta linha é o conserto.

2. Crie o `check-combo.ts` com a interface congelada e a tabela de pares forçados. A assinatura abaixo é um contrato: lições posteriores importam o `checkCombo` exatamente por este nome, e o mint de economia de m02-l1 passa pela cancela dele. (`REQUIRED_ACCOUNT_EXTENSIONS` também é exportado, mas só a cancela desta própria lição o lê.)

```ts
// check-combo.ts: R2. The five rules of
// check_for_invalid_mint_extension_combinations, ported 1:1.
// Source of truth: solana-program/token-2022 @ 426400f,
// interface/src/extension/mod.rs (lines 1326-1374).

export interface ComboResult {
  valid: boolean;
  reason?: string;
}

// The forced-pair layer from required_init_account_extensions (same file,
// line 1296): initializing a token ACCOUNT for a mint with the left-hand
// extension forces the right-hand account extensions to exist.
export const REQUIRED_ACCOUNT_EXTENSIONS: Record<string, string[]> = {
  TransferFeeConfig: ["TransferFeeAmount"],
  NonTransferable: ["NonTransferableAccount", "ImmutableOwner"],
  TransferHook: ["TransferHookAccount"],
  Pausable: ["PausableAccount"],
};

export function checkCombo(extensions: string[]): ComboResult {
  const has = (name: string): boolean => extensions.includes(name);

  // TODO rule 1: ConfidentialTransferFeeConfig requires BOTH
  //   TransferFeeConfig AND ConfidentialTransferMint.
  // TODO rule 2: TransferFeeConfig + ConfidentialTransferMint together
  //   REQUIRE ConfidentialTransferFeeConfig.
  // TODO rule 3: ConfidentialMintBurn requires ConfidentialTransferMint.

  // Rule 4: ScaledUiAmount and InterestBearingConfig are mutually exclusive.
  if (has("ScaledUiAmount") && has("InterestBearingConfig")) {
    return {
      valid: false,
      reason:
        "rule 4: ScaledUiAmount and InterestBearingConfig are mutually exclusive",
    };
  }

  // TODO rule 5: NonTransferable + ConfidentialTransferMint is invalid
  //   UNLESS ConfidentialMintBurn is also present.

  return { valid: true };
}
```

3. Olhe o formato da regra 4 antes de escrever as outras, porque a função inteira é esse formato repetido: cada regra é uma cláusula de trava que retorna `{ valid: false, reason }` quando a condição dela dispara, e um conjunto que sobrevive às cinco travas cai até `{ valid: true }`. Uma escolha estrutural que vale nomear: o original em Rust coleta os booleanos primeiro e só então roda as travas, porque ele itera uma slice de variantes de enum uma vez, por eficiência dentro do programa. A gente não está dentro de um programa, então a closure `has()` sobre o `includes` lê mais perto dos próprios enunciados das regras. Porte a LÓGICA, não o loop.

4. Agora preencha as regras 1, 2, 3 e 5, trabalhando direto do Rust que você tem aberto no painel dividido. Escreva cada uma onde o TODO dela já está: os slots estão dispostos em ordem de fonte, três acima da trava da regra 4 e um abaixo, para que um arquivo completo se leia de cima para baixo na mesma sequência em que o Rust se lê. Este é o exercício de completion da lição, então eu não vou te entregar os quatro corpos, mas aqui está a disciplina de mapeamento que faz cada um deles ser um port de dois minutos. Pegue a trava em Rust da regra 3: `if confidential_mint_burn && !confidential_transfer_mint`. Leia ela como uma frase ("mint-burn presente e a dependência dela ausente"), e então escreva a mesma frase com `has()`. A regra 1 quer uma conjunção negada, cuidado com os parênteses: o estado inválido é a ponte presente enquanto NÃO estão as duas pontas. A regra 2 e a regra 5 são as duas travas de três termos, duas presenças e uma ausência. Comece cada string de `reason` com a tag da regra dela, `"rule 1: ..."` até `"rule 5: ..."`, porque o arquivo de teste faz assert na tag: um validador que rejeita o conjunto certo pela razão errada é um validador em que você não pode confiar para explicar uma decisão de produto.

![Lado a lado da trava da regra 4 em Rust e do port dela em TypeScript, anotado para mostrar o mapeamento de flag para has() e que o validador adiciona razões por regra onde o programa tem um erro só, colapsado.](assets/v08-annotated-code.webp)

5. Escreva o assert-script, `test-check-combo.ts`. Mesmo padrão de linha de testes que o `test-decode-mint.ts` de m01-l2: asserts simples, exit 1 no primeiro erro, uma linha de resumo no sucesso. Os casos abaixo são o critério de aceitação da lição, incluindo a virada da regra 5 em que adicionar uma extensão torna legal um par ilegal:

```ts
// test-check-combo.ts: the assert-script for R2.
import { checkCombo, REQUIRED_ACCOUNT_EXTENSIONS } from "./check-combo";

let passed = 0;

function expectValid(extensions: string[]): void {
  const result = checkCombo(extensions);
  if (!result.valid) {
    console.error(`FAIL: expected [${extensions}] legal, got: ${result.reason}`);
    process.exit(1);
  }
  passed += 1;
}

function expectInvalid(extensions: string[], ruleTag: string): void {
  const result = checkCombo(extensions);
  if (result.valid) {
    console.error(`FAIL: expected [${extensions}] illegal (${ruleTag}), got valid`);
    process.exit(1);
  }
  if (!result.reason?.startsWith(ruleTag)) {
    console.error(
      `FAIL: [${extensions}] rejected, but by "${result.reason}" instead of ${ruleTag}`,
    );
    process.exit(1);
  }
  passed += 1;
}

// Legal set: a fee token with metadata. The everyday case.
expectValid(["TransferFeeConfig", "MetadataPointer"]);

// Rule 4: the two rebasing-display extensions are mutually exclusive.
expectInvalid(["ScaledUiAmount", "InterestBearingConfig"], "rule 4");

// Rule 3: confidential supply without confidential transfers is nonsense.
expectInvalid(["ConfidentialMintBurn", "MetadataPointer"], "rule 3");

// Rule 2: fees + confidential balances demand the confidential-fee bridge.
expectInvalid(["TransferFeeConfig", "ConfidentialTransferMint"], "rule 2");

// Rule 5: soulbound + confidential is illegal on its own...
expectInvalid(["NonTransferable", "ConfidentialTransferMint"], "rule 5");

// ...and becomes legal once ConfidentialMintBurn joins the set.
expectValid(["NonTransferable", "ConfidentialTransferMint", "ConfidentialMintBurn"]);

// Rule 1: the confidential-fee bridge cannot stand alone.
expectInvalid(["ConfidentialTransferFeeConfig"], "rule 1");

// The forced-pair table is data, not logic. Sanity-check its shape.
if (REQUIRED_ACCOUNT_EXTENSIONS["NonTransferable"].length !== 2) {
  console.error("FAIL: NonTransferable must force two account extensions");
  process.exit(1);
}
passed += 1;

console.log(`check-combo: all ${passed} assertions passed`);
```

6. Rode:

```bash
npx tsx test-check-combo.ts
```

Antes de você preencher os TODOs, a rodada falha no caso da regra 3, e esse starter que falha é o ponto: ele prova que os testes conseguem pegar um validador incompleto. Depois que as suas quatro regras entrarem, a saída esperada:

```
check-combo: all 8 assertions passed
```

Se você está falhando numa string de reason em vez de num veredicto, esse é o assert de tag fazendo o trabalho dele. Cheque qual regra a sua ordem de travas dispara primeiro: um conjunto pode violar duas regras de uma vez (adicione o ConfidentialMintBurn ao par da regra 4 e as regras 3 e 4 se aplicam as duas), e a ordem da fonte decide qual razão vence. Bata a ordem da fonte e as tags se alinham.

7. Agora feche o loop com o R1. Tudo até aqui validou conjuntos hipotéticos; o contrato do artefato diz que o check-combo roda contra o conjunto REAL de um mint de verdade, que é exatamente o que o seu inspetor de m01-l2 emite. E tem uma emenda para atravessar primeiro, aquela sobre a qual m01-l2 te avisou na nota de nomenclatura dele: o `checkCombo` é um port 1:1 do Rust, então ele fala as grafias do Rust, enquanto o `decode-mint` nomeia extensões a partir do enum `ExtensionType` do cliente JS fixado, e em três entradas os dois discordam. O tipo 16 é `ConfidentialTransferFeeConfig` na fonte e `ConfidentialTransferFee` no cliente. O tipo 25 é `ScaledUiAmount` e `ScaledUiAmountConfig`. O tipo 26 é `Pausable` e `PausableConfig`. Alimente as regras do Rust com as strings do cliente e a regra 2 dispara no PYUSD, o mint emblemático do próprio curso, o que é bem longe do que um mint real que já inicializou deveria produzir.

Então a cola tem um trabalho antes de entregar a lista, e m01-l2 já te disse em qual campo confiar: o u16 é a identidade, nomes são peles por toolchain em cima dele. Normalize no NÚMERO. Escreva o `check-live.ts` (ajuste o caminho de import e o nome de export para casar com o seu próprio arquivo decode-mint; este trecho assume que o resultado decodificado carrega o array `extensions: {name, type, length}[]` que o seu assert-script do R1 já checa):

```ts
// check-live.ts: R1 feeds R2. Decode a live mint, judge its set.
import { decodeMint } from "../m01-l2/decode-mint";
import { checkCombo } from "./check-combo";

// The three type codes where the pinned client's enum and the Rust source
// disagree on spelling. Keyed on the u16 rather than on the client's string,
// because the number is the extension's identity and the string is a skin:
// a name-to-name table would go stale the next time a client renames one, and
// this table can only go stale if a type code changes meaning, which is a far
// louder event.
const RUST_NAME_BY_TYPE: Record<number, string> = {
  16: "ConfidentialTransferFeeConfig", // client: ConfidentialTransferFee
  25: "ScaledUiAmount", //               client: ScaledUiAmountConfig
  26: "Pausable", //                     client: PausableConfig
};

const mintAddress = process.argv[2];
if (!mintAddress) {
  console.error("usage: npx tsx check-live.ts <mint-address>");
  process.exit(1);
}

const decoded = await decodeMint(mintAddress);
const names = decoded.extensions.map((ext) => RUST_NAME_BY_TYPE[ext.type] ?? ext.name);
console.log(`extensions: [${names.join(", ")}]`);
console.log(checkCombo(names));
```

Aponte ele para o mint fixado de m01-l2 e você deve ver a lista de extensões dele, com o tipo 16 agora impresso na grafia da fonte, seguida de `{ valid: true }`. Faça o desvio se isso te surpreender: comente a normalização, rode de novo e veja o seu validador novinho rejeitar a stablecoin do PayPal com `rule 2: TransferFeeConfig + ConfidentialTransferMint require ConfidentialTransferFeeConfig` — uma rejeição perfeitamente correta sobre a regra e perfeitamente errada sobre o mint. Duas ferramentas, dois vocabulários, um código de tipo, e uma hora depurando regras que nunca estiveram quebradas. Toda integração que você escrever entre dois SDKs tem uma emenda como esta em algum lugar dentro dela. Aí resgate a nota que você guardou do passo 10 de m01-l2: rode o mint mais estranho que você achou pelo mesmo pipeline. Ele também vai voltar válido, porque ele inicializou, e essa é a parte interessante: leia a lista dele contra as nove pedras marcadas e veja sob qual lei entre extensões ele vive, se alguma. Claro que ele é válido: ele inicializou, então passou por essas mesmas cinco regras dentro do programa no dia em que nasceu. Que é a sacada silenciosa do lab inteiro. Todo mint Token-2022 ao vivo na mainnet é uma testemunha que já passou pela função que você acabou de portar. O seu validador move esse julgamento de depois do fato para antes da revisão de design.

![Fluxograma de pipeline do endereço do mint passando pelo decode-mint (R1) para dentro do checkCombo (R2), bifurcando para seguir-se-válido ou corrigir-o-conjunto-se-rejeitado, servindo de cancela para toda construção de mint posterior no curso.](assets/v09-flowchart.webp)

Esse é o lab. Uma confissão antes do challenge: o primeiro passo deste curso fixou uma tabela de conflitos digitada à mão num doc de planejamento, três regras lembradas de um changelog. Escrever o passo de extração contra a função de verdade achou as outras duas dentro de uma hora, incluindo o condicional da regra 5 que a tabela tinha achatado num "incompatível" seco. Uma matriz copiada não só fica obsoleta. Ela já começa obsoleta. O validador que você escreveu é como ela se mantém fresca: os testes quebram quando a fonte se move, e a derivação leva uma noite para refazer.

## Challenge

Duas partes, uma solo e uma empírica.

**Solo: o coding challenge.** Este é o único coding challenge do módulo e é o artefato desta lição provado de ponta a ponta: implemente o `checkCombo` para que todas as cinco regras disparem exatamente como a função da fonte as dispara. O starter é um primo enxuto do arquivo do passo 2 do lab: mesmo resultado `{ valid, reason? }`, a regra 4 já implementada e as outras quatro faltando. Uma diferença deliberada em relação ao contrato do lab: o grader chama a sua função com cada nome de extensão como argumento de string próprio, então a assinatura publicada é `checkCombo(ext1: string, ext2?: string, ext3?: string)`, a primeira linha do starter junta esses argumentos no mesmo array `extensions` que a sua versão do lab recebe, e a lógica das regras é idêntica daí para baixo. Ele é deliberadamente mais enxuto que a versão do lab, sem tabela de pares forçados e sem tags de regra na única string de reason dele, então nada é aproveitado além da lógica que você derivou. Ele falha na suíte de testes do jeito que vem. A sua solução passa nela inteira. Os critérios de aceitação, direto da cancela:

- `checkCombo("TransferFeeConfig", "MetadataPointer").valid === true`
- `checkCombo("ScaledUiAmount", "InterestBearingConfig").valid === false`
- `checkCombo("ConfidentialMintBurn", "MetadataPointer").valid === false`
- `checkCombo("TransferFeeConfig", "ConfidentialTransferMint").valid === false`
- `checkCombo("NonTransferable", "ConfidentialTransferMint").valid === false`
- `checkCombo("NonTransferable", "ConfidentialTransferMint", "ConfidentialMintBurn").valid === true`

A cancela checa só os veredictos. Se cobre a régua mais alta mesmo assim e faça cada rejeição nomear a regra da fonte que disparou, do jeito que a sua versão do lab faz, porque um validador que rejeita o conjunto certo pela razão errada passa nos testes e mesmo assim perde uma revisão de design. Se você completou o lab, você já fez o trabalho; o challenge é onde você prova isso sem o apoio à vista.

**Empírico: sonde o delta docs-vs-código.** Esta metade roda na sua própria máquina — ela precisa de duas CLIs locais e de um cluster com quem conversar; a cancela avaliada lá em cima não depende disso, então, se você está trabalhando só pelo navegador, leia, anote a dependência e volte quando tiver as duas. Fique avisado sobre exatamente o que "as duas" quer dizer, porque o ambiente de lab permanente do módulo 2 não é isso: o módulo 2 instala pacotes npm e o surfpool, e nenhum dos dois te dá as duas ferramentas abaixo. Você precisa da CLI `solana` (m01-l3 te entrega o instalador de uma linha dela como uma sondagem opcional) e do `spl-token-cli`, que é um cargo install e portanto quer uma toolchain Rust — a que a lição de hook do módulo 3 monta. Então o alvo honesto de adiamento é o módulo 3, não o módulo 2, e o jeito mais barato de resolver isso cedo é rodar o instalador de m01-l3 agora e adicionar `cargo install spl-token-cli` ao lado. A seção de teoria deixou NonTransferable mais TransferFeeConfig sem resolução de propósito: a documentação chama o par de incompatível, as cinco regras extraídas nunca mencionam ele. O seu validador, seguindo o código fielmente, aceita ele. Então pergunte direto ao programa. A CLI `spl-token` vem junto em algumas instalações da toolchain Solana e em outras não, então cheque primeiro com `spl-token --version`; se ela estiver faltando, `cargo install spl-token-cli` te dá uma (a 5.6.1 é a atual no crates.io em 2026-08-22, e o gabarito abaixo foi produzido na 5.5.0, então qualquer versão nessa vizinhança serve). Seja qual for a que você pegar, anote o número, porque ele importa para como você lê o resultado. A CLI consegue tentar exatamente esta init contra a devnet:

```bash
solana config set --url https://api.devnet.solana.com
solana airdrop 2   # devnet faucet; if rate-limited, use faucet.solana.com
spl-token create-token --program-2022 \
  --enable-non-transferable \
  --transfer-fee-basis-points 50 \
  --transfer-fee-maximum-fee 5000
```

Se o faucet da devnet te limitar por taxa (ele limita, com frequência, e o airdrop simplesmente falha), qualquer validador local serve no lugar: aponte o `--url` para um mainnet-fork do `surfpool start --no-tui --no-studio` e faça o airdrop lá. O surfpool é um validador local que faz um fork do estado da mainnet sob demanda (no macOS `brew install txtx/taps/surfpool`, em outras plataformas pegue ele na página de releases do surfpool); no próximo módulo ele vira o ambiente de lab permanente, então instalar ele agora não é movimento desperdiçado. O fork roda o mesmo binário do Token-2022 implantado, então o veredicto dele sobre esta pergunta é o mesmo veredicto que a mainnet daria.

Registre o que acontece: criado, ou rejeitado, e por quem. Esses são três desfechos diferentes, não dois. O programa pode rejeitar. A CLI pode recusar do lado cliente antes de qualquer coisa ser enviada, o que te diria que o "conflito" mora numa ferramenta e não no consenso. Ou pode passar, caso em que você decodifica o mint resultante e confirma com o seu próprio inspetor que as duas extensões realmente pousaram, porque "o comando saiu com 0" não é a mesma afirmação que "a conta carrega as duas". Não pule esse último passo; ele é a diferença entre acreditar numa CLI e ler os bytes, que é o hábito inteiro para o qual este módulo existe.

![Uma tentativa de init se abre em três desfechos, cada um provando algo diferente: uma recusa do lado cliente, uma rejeição on-chain, ou um sucesso que só os bytes decodificados conseguem confirmar.](assets/v10-flowchart.webp)

Eu deliberadamente não estou imprimindo o desfecho aqui em cima. Rode primeiro, anote o que você conseguiu ao lado da data e do seu `spl-token --version`, e só então leia o gabarito no fim do Checkpoint. O que quer que você ache, repare que o check-combo continua inalterado. Ele modela o que a init aplica, e adicionar uma regra que o programa não aplica faria o seu validador discordar do programa que ele existe para prever.

## Checkpoint

O critério desta lição: o seu `check-combo` aceita um conjunto legal de extensões e rejeita cada um dos quatro casos ilegais com a regra da fonte correta, batendo com o `check_for_invalid_mint_extension_combinations` em 426400f, e os testes do coding challenge passam, com o starter falhando e a solução verde. Ao lado da rodada que passa, escreva a única frase que a avaliação pede: a sua própria separação entre "legal de inicializar" e "aceito por um venue ou uma carteira". Se a sua frase mencionar allowlists ou PermanentDelegate, você já internalizou o argumento de abertura do Módulo 5.

### Gabarito: o que a sondagem retorna

Leia isto só depois de ter rodado. Em 2026-08-22, spl-token-cli 5.5.0 contra um mainnet-fork rodando o binário do Token-2022 implantado, o `create-token` acima **funcionou**. Sim, 5.5.0: um minor atrás da 5.6.1 que o crates.io listava como atual no mesmo dia, porque o meu bundle de toolchain estava atrasado em relação ao registro, que é exatamente por que o challenge te fez anotar a sua própria versão. Se a sua sondagem rodou na 5.6.1 e qualquer comportamento abaixo for diferente, a sua rodada vence; registre o delta com a versão anexada. Ele imprimiu um endereço e uma assinatura como qualquer criação de mint comum. Decodificar a conta nova confirma que não foi ilusão da CLI. Aponte o seu próprio inspetor de m01-l2 para o endereço que a CLI imprimiu e a leitura volta com este formato:

```text
npx tsx decode-mint.ts <THE_MINT_THE_CLI_PRINTED> <YOUR_FORK_RPC_URL>

owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
data length:   282 bytes -> extended-165+1
extensions (2):
  { name: NonTransferable, type: 9, length: 0 }
  { name: TransferFeeConfig, type: 1, length: 108 }
```

282 bytes, owner `Tokenz...`, e duas entradas TLV, `NonTransferable` e `TransferFeeConfig` na grafia do cliente fixado, sentadas no mesmo mint: exatamente os 166 + (4 + 0) + (4 + 108) em que a sua aritmética de `compute-len` precifica o par. O programa aceitou.

Então o delta se resolve a favor do código, e o seu validador estava certo em aceitar. As páginas de documentação que chamam esse par de "incompatível" estão fazendo uma afirmação *semântica*, não uma afirmação de legalidade, e a afirmação semântica é justa: um mint NonTransferable rejeita toda transferência, então uma tabela de taxas nele nunca consegue arrecadar nada. É configuração morta sobre a qual você paga rent. Isso é uma coisa real para avisar um designer, e genuinamente não é uma regra que o programa aplica na inicialização. As duas afirmações são verdadeiras ao mesmo tempo, que é exatamente por que elas precisam de nomes separados.

Dois achados laterais da mesma rodada, os dois valendo mais que a manchete. Primeiro, a CLI aplica algumas regras ela mesma: peça `--interest-rate` e `--ui-amount-multiplier` juntos e ela recusa antes de tocar a rede, um conflito de parser de argumentos e não um erro de programa. A regra 4 tem trava dupla, em dois lugares diferentes, e só um deles é consenso. Segundo, peça uma taxa de transferência mais transferências confidenciais, o par da regra 2 que a sua suíte de testes espera que seja ilegal, e funciona mesmo assim, porque a CLI adiciona `ConfidentialTransferFeeConfig` para você sem alarde e inicializa o trio legal. Os dois fazem o mesmo ponto: uma ferramenta sentada entre você e o programa pode adicionar regras que o programa não tem, e satisfazer regras que você nunca pediu para ela satisfazer. Nenhum dos dois é visível pelo exit code. Só a conta decodificada te diz o que você de fato construiu, e você é dono desse decodificador desde m01-l2.

![Entre o conjunto de extensões que você pediu e os bytes on-chain ficam a CLI, que pode recusar ou adicionar extensões em silêncio, e o programa aplicando só as cinco regras dele.](assets/v11-diagram.webp)

Se a sua rodada discordou de qualquer coisa disso, a sua rodada vence e eu quero saber disso, com a versão e a data anexadas. Isso não é educação. É o quinto passo do loop de re-derivação.

Falhas por razão errada e escorregões de parênteses na regra 1 são os dois erros que eu espero; se o assert de tag continuar mordendo depois de uma checagem de ordem de fonte, leve o caso que falha e a sua ordem de travas para a discussão do curso e a gente lê o seu port contra o Rust junto. Esse tipo de diff, a sua derivação contra a fonte, é precisamente o músculo de revisão que este curso está construindo.

Agora você consegue dizer quais conjuntos de extensões são LEGAIS de construir, e consegue provar isso contra qualquer mint ao vivo a partir dos bytes. Mas legal não é o mesmo que negociável, e antes de a gente chegar nesse acerto de contas, o próximo módulo constrói cada extensão de verdade, começando pelo conjunto de economia: taxas, juros, UI escalada. O seu mint do SPROUT vai carregar TransferFeeConfig mais uma das duas extensões de exibição, o seu validador vai ser a coisa que te impede de escolher as duas, e aí a gente persegue a pergunta que o check-combo não consegue responder: quando uma transferência retém uma taxa, onde o dinheiro de fato fica, e quem tem permissão para varrer ele? Boa derivação! 🌱
