# 0.3x -> 1.0: as dores que quem migra ainda encontra

Você acabou de entregar o capstone no m09-l3: o floor-registry fazendo CPI para o counter, o quarter-vault, o prize-escrow e o swap de token-para-ticket, levado até o fim por teste, fuzz, profile, uma rodada de localnet em Surfpool, um deploy em devnet, e uma passada local de verify-a-partir-do-repo. Cada linha daquilo saiu de um arquivo em branco. Nada do que você tocou foi herdado.

Agora a gente se vira para as bases de código que não começaram em branco.

Aqui está o cenário, e ele não é hipotético. Você herda um programa que compilou e fez deploy limpo no Anchor 0.32 oito meses atrás. O seu trabalho é pequeno: subir o toolchain para a linha 1.x e seguir em frente. Então você faz a coisa óbvia. Aponte o `avm` para a linha atual e recompile.

```bash
# avm ships with the Anchor installer; if you don't have it:
#   cargo install --git https://github.com/otter-sec/anchor avm --force
# (otter-sec/anchor is the repo's current home; the coral-xyz and
#  solana-foundation URLs still redirect there.)
# Toolchain is the 1.x line. 1.1.2 is the version this recon is written against;
# the line moved on 2026-09-04, when 1.2.0 shipped. Pin 1.1.2 here so the recon
# output matches the lesson, and re-check `avm list` before you pin anything else.
avm install 1.1.2
avm use 1.1.2
anchor build
```

Não vai compilar. O `#[interface]` é um atributo desconhecido. O `CpiContext::new` rejeita o `AccountInfo` de programa que você vem passando para ele há dois anos. O `anchor login` desapareceu por completo. E aqui está a parte que importa: nada do que você escreveu está errado. O framework se moveu embaixo de você, e o compilador vai te dizer *o que* estalou sem nunca te dizer *por quê*. Esta lição é o mapa do que se moveu entre o 0.3x e o 1.0, e a razão atrás de cada movimento. Pegue o *por quê* e o port para de ser um jogo de adivinhação.

## Resumo

Este é o primeiro de dois deltas de migração. Ele é um tour de por-que-não-só-o-que sobre o release do Anchor 1.0.0 (entregue em 2026-04-02) e as quebras de 0.32-para-1.0 que continuam vivas em bases de código reais hoje. Não tem build para completar aqui. Essa é uma troca deliberada: você gasta a hora de mão na massa de uma lição normal num platô em vez disso, lendo o delta no frio, para que o port de verdade no m10-l3 vire um checklist em vez de uma briga com o compilador. Esta lição abre a trilha de quem migra do módulo, as três lições do m10-l1 até o m10-l3, que existem para leitores carregando uma base de código mais antiga. Se você é novo de novo no Anchor e nunca escreveu uma linha de 0.32, você pode passar o olho na trilha e ir direto para a conclusão no m10-l4. O custo de passar o olho é perder o contexto de trajetória no qual o resto do módulo se apoia.

A gente vai percorrer seis mudanças. Para cada uma: a quebra exata, a razão de o framework fazer ela, e a única edição que corrige. No fim você deve conseguir olhar um trecho de 0.32 e nomear de memória a mudança de 1.0 que ele bate. Esse reconhecimento é o ponto inteiro.

Uma palavra sobre como estas lições repassam responsabilidade. No começo deste curso eu te percorri comando por comando. Aqui eu te entrego a subida do toolchain e um grep, e você lê o compilador você mesmo. No m10-l2 você recebe o código e uma tabela de delta e dirige um port pequeno você mesmo. No m10-l3 você recebe um repositório quebrado cujas edições mecânicas estão marcadas e cujas duas edições mais difíceis não estão, porque nessa altura a saída própria do compilador é a marcação. As rodinhas saem ao longo do módulo, e esta lição é onde a primeira sai.

## O mapa do que se moveu, e por quê

O jeito honesto de ler uma lista de mudanças que quebram é perguntar, para cada item, *que problema a forma antiga estava causando que a forma nova resolve?* Um framework não quebra código no valor de um milhão de downloads por diversão. Cada uma destas tinha um limite motivador. Então a gente começa cada mudança a partir daquele limite, do jeito que você faria se você fosse quem decide se entrega a quebra.

### 1. O rename que ninguém terminou de fazer

A mudança mais visível é também a que te ensina mais sobre migração como prática. O pacote de cliente se moveu de `@coral-xyz/anchor` para `@anchor-lang/core` (PR #4141), e o 1.0 é onde o nome antigo parou de ser o que a documentação te entrega. Leia as datas em ordem, porque elas são a pista: o pacote `@anchor-lang/core` foi criado no npm em 2025-12-19, o primeiro publish de verdade dele aterrissou em 2026-01-06 com o número de versão da linha antiga `0.32.1`, e o 1.0.0 em si só foi entregue em 2026-04-02. O nome se moveu na linha 0.32, meses à frente do release sob o qual ele é geralmente arquivado.

Então o rename é notícia velha. Ele teve quase um ano para se propagar. Aqui está a pergunta motivadora que quem migra deveria fazer: *se o nome canônico mudou oito meses atrás, o nome antigo está morto?*

A resposta ingênua é sim, claro, a documentação diz para usar o novo. Essa resposta vai caladamente quebrar a sua migração. Porque "canônico" e "o que você de fato vai ler" divergiram, e divergiram forte.

![O pacote antigo @coral-xyz/anchor puxou cerca de 602k downloads semanais contra os aproximadamente 15k do novo @anchor-lang/core, uma lacuna perto de quarenta para um.](assets/v01-chart.webp)

Mais ou menos oito meses depois do rename, o nome antigo ainda supera o novo em downloads por algo perto de quarenta para um. O número exato não importa e ele muda toda semana. A forma dele é o que você carrega: o pacote que você é mandado importar não é o pacote que o ecossistema está importando. A maior parte do código de exemplo que você copia de um blog, a maior parte das respostas de Stack Overflow, a maior parte dos repositórios meio-migrados que você herda, continuam recorrendo ao `@coral-xyz/anchor`.

É por isso que o colega que diz "o rename é cosmético, só atualize o import" está te entregando uma armadilha. O rename em si de fato é só um movimento de nome, a API dentro da caixa não mudou *por causa do rename*. A armadilha é a realidade do ecossistema em volta dele. Se você assumir um import canônico só e grepar um escopo só, você vai perder metade dos call sites, porque o código que você está portando foi escrito contra o nome que ainda ganha a contagem de downloads. A correção é um hábito, não uma edição: grepe os dois escopos antes de assumir qualquer coisa.

```bash
# Before you touch a line, inventory BOTH names.
# ripgrep (rg) ships with most dev setups; else: brew install ripgrep
rg -l "@coral-xyz/anchor" .
rg -l "@anchor-lang/core" .
```

Tem uma peça de cor pequena e sombria que deixa isso concreto. O caminho de aprendizado oficial do próprio ecossistema, o que está em solana.com/developers/courses, agora serve um redirect 308 para dentro de um repositório do GitHub arquivado que foi congelado em 2025-01-24. O curso canônico de Anchor é conteúdo mais ou menos da era 0.30 sentado numa árvore somente-leitura. Então quando quem migra vai procurar orientação autoritativa de migração e acha uma lápide, isso não é um acidente da sua busca. É o estado do mundo, e é exatamente por isso que esta lição existe num curso pago e essencialmente em lugar nenhum a mais.

### 2. O CpiContext::new parou de pegar um AccountInfo

Aqui está uma quebra que para o build, não só o linter. No 0.32 você montava uma chamada entre programas entregando para o `CpiContext::new` o programa como um `AccountInfo`:

![No 0.32 o CpiContext::new pegava o token program como um AccountInfo e usava Transfer; no 1.0 ele pega um Pubkey via .key() e usa TransferChecked com decimais.](assets/v02-annotated-code.webp)

O Anchor 1.0 mudou o `CpiContext::new` para pegar o programa como um `Pubkey` (PR #2762). Passe um `.to_account_info()` ali agora e ele falha em compilar com um descasamento de tipo seco: esperava `Pubkey`, achou `AccountInfo`.

Por que fazer essa quebra? Raciocine a partir do que uma CPI de fato precisa. Quando você montou a transferência de token lá no m05-l1, o runtime identificava o programa chamado pelo endereço dele e nada mais, porque a identidade de um programa na Solana simplesmente *é* a chave pública dele. Quando você entregava para o construtor um `AccountInfo` inteiro, você estava passando um handle gordo onde uma chave só era a única parte estrutural dele, e o Anchor se virava e puxava a chave de volta para fora internamente de qualquer jeito. Mover o argumento para `Pubkey` remove essa indireção redundante e alinha o construtor com como o runtime já pensa sobre o programa chamado. É um aperto pequeno, e é a forma em cima da qual o modelo de borrow do 2.0 depois constrói. A correção é exatamente dois caracteres de intenção: troque o `.to_account_info()` pelo `.key()`.

Um leitor afiado contesta aqui, e a contestação vale responder porque é a objeção que você vai ouvir em code review. Se o Anchor ia extrair a chave de qualquer jeito, o que passar o `AccountInfo` inteiro de fato custava, além de alguns bytes na pilha? A resposta honesta é que no 0.32 isso custava quase nada em runtime, e se custo de runtime fosse a história inteira esta quebra não valeria a rotatividade de código no valor de um milhão de downloads. A motivação de verdade é que o handle gordo te deixava passar uma conta que não era o programa de jeito nenhum, e o construtor pegaria ela, adiando o descasamento para uma falha de runtime em vez de um erro de compilação. Estreitar o tipo para `Pubkey` move uma classe inteira de erros de "eu passei a conta errada aqui" de um erro on-chain confuso para uma mensagem seca do seu próprio compilador, que é exatamente a troca para a qual um framework tipado existe.

Pegando carona com esta tem a forma idiomática de transferência do SPL. O `transfer` simples é depreciado em favor do `transfer_checked`, que pega o mint e os decimais para o programa de token poder verificar que você está movendo o que você pensa que está movendo na precisão que você pensa que ele tem. Então a migração mecânica são duas jogadas de uma vez: o argumento de programa vai para um `Pubkey`, e a chamada de transferência ganha uma conta de mint e um valor de decimais. Perca a segunda metade e você corrigiu o erro de tipo só para entregar uma chamada depreciada.

Uma cautela para você não confundir dois deltas que parecem iguais. O modelo de borrow do `CpiHandle`, onde o próprio argumento de contas muda de forma, é uma mudança do 2.0, o território da próxima lição. No 1.0 a quebra é especificamente e somente o argumento de programa se movendo de `AccountInfo` para `Pubkey`. Se você vê conselho sobre o `CpiHandle`, você está lendo sobre um mundo diferente.

### 3. O literal de space virou uma expressão

No 0.32, metade dos blocos de `init` do ecossistema carregava um cálculo de space feito na mão que começava com um `8` mágico:

![O literal de space contado na mão do 0.32, 8 mais os tamanhos de campo, vira a expressão derivada DISCRIMINATOR.len() mais INIT_SPACE no 1.0.](assets/v03-annotated-code.webp)

O `8` era o discriminator da conta, e tudo depois dele era você, contando bytes de campo na mão e esperando ter acertado o preenchimento. O limite motivador é óbvio depois de você ter entregado um bug por causa dele: um literal contado na mão deriva. Adicione um `u64` na struct, esqueça de subir o literal, e você recebe uma falha de runtime que não tem nada a ver com o código que você acabou de mudar.

O 1.0 substitui o literal por `DISCRIMINATOR.len() + INIT_SPACE`. O `INIT_SPACE` vem do `#[derive(InitSpace)]` na sua struct de conta e é computado a partir dos campos em si, então ele acompanha a struct automaticamente. O `DISCRIMINATOR.len()` substitui o `8` mágico pelo tamanho de verdade do discriminator de verdade, que importa porque o 1.0 também te deixa definir discriminators customizados que não têm oito bytes. A correção é deletar a aritmética e deixar o framework derivar ela. Esta é a quebra rara que é puro lado positivo: você está removendo uma classe de bug, não trocando uma forma por outra.

### 4. Um enum #[error_code] por programa

O 0.32 te deixava espalhar definições de erro por vários enums `#[error_code]`, um por módulo se você quisesse. A regra do 1.0 (PR #4300) é um enum por programa — mas ouça a imposição corretamente, porque é esta a armadilha: um segundo enum `#[error_code]` ainda *compila verde* na linha 1.x. Sem erro, sem aviso, verificado contra o toolchain fixado. Se o programa que você herdou dividiu os erros dele num `VaultError` e num `EscrowError`, o build que deveria ter reclamado é entregue bem, e o risco aterrissa caladinho em runtime em vez disso.

O risco é colisão de discriminante, e vale percorrer uma instância concreta para sentir. O Anchor atribui um código numérico para cada erro pela posição dele no enum, deslocado dentro de um espaço compartilhado de códigos de erro que começa em `6000`. Imagine o programa herdado: o `VaultError` declara `Overflow` primeiro, então ele vira `6000`, e o `EscrowError` em outro módulo também declara a primeira variante dele, que *também* quer ser `6000`. Agora um cliente pega o erro `6000` de uma transação que falhou e não tem jeito de saber se o vault estourou ou se o escrow rejeitou, porque dois enums independentes contaram os dois a partir da mesma base e produziram o mesmo código para significados diferentes. Colapsar para exatamente um enum por programa deixa o código de erro um índice único e sem ambiguidade para dentro de uma lista única, então o `6000` quer dizer uma coisa para sempre. A correção é um merge que você mesmo impõe, porque o compilador não vai impor por você: mova cada variante para dentro de um enum, e se dois subsistemas entregaram uma variante com o mesmo nome, renomeie um deles. É tedioso em vez de difícil, e o retorno é que cada um dos seus códigos de erro finalmente quer dizer exatamente uma coisa, que é o que quem decodifica eles off-chain precisava desde o começo.

### 5. O #[interface] e as instruções de interface desapareceram

Esta é a quebra no hook. No 0.32, uma instrução de interface do SPL, o caso clássico sendo o `execute` de um transfer hook, era declarada com a macro de atributo `#[interface]`. No 1.0 aquela macro e a maquinaria inteira de instruções de interface foram removidas. Não existe um `#[program_interface]` para o qual você renomeia, e não existe feature flag que traga isso de volta. Desapareceu.

O que substituiu, e por quê? A razão é unificação. Cada instrução comum do Anchor já é despachada casando o discriminator dela, os bytes iniciais dos dados de instrução. Instruções de interface precisavam da *mesma* coisa, despacho por um discriminator específico e definido de fora, mas elas tinham uma macro feita sob medida para fazer isso. O 1.0 colapsa o caso especial para dentro do geral. O `execute` de um transfer hook agora é declarado como qualquer outra instrução, exceto que você diz para o Anchor qual discriminator casar:

![A macro de interface do 0.32 num transfer hook vira um atributo de instrução do 1.0 carregando um slice de discriminator do SPL explícito, usando despacho comum por discriminator.](assets/v04-annotated-code.webp)

O discriminator vem da própria interface do SPL, exposto como uma constante `SPL_DISCRIMINATOR_SLICE`, então você está casando os bytes exatos que a interface define em vez de confiar numa macro para saber eles por você. A correção: delete o atributo `#[interface]` e declare a instrução normalmente com `#[instruction(discriminator = ...SPL_DISCRIMINATOR_SLICE)]`.

Uma nota de fronteira, porque é aqui que os cursos se sobrepõem e eu quero manter as linhas limpas. Esta lição ensina a *migração* da declaração do hook, o atributo que mudou. Ela não ensina a interface de transfer hook em si — as contas que ele encaminha, o contrato de execute, o programa inteiro. Esse é o território do curso de Digital Assets, Tokenização e Token Extensions, e se você está encontrando transfer hooks pela primeira vez aqui, é esse o curso que ensina eles. Aqui a gente só se importa com qual atributo você troca.

### 6. O toolchain mudou de forma embaixo do código

As primeiras cinco mudanças são coisas que você edita no código. A sexta não é uma edição de código nenhuma, e sim o chão em cima do qual o código fica de pé, e ela é a que embosca as pessoas na hora do deploy em vez de na hora do build.

Comece pela emboscada. O seu programa de 0.32 compila no 1.x depois de você corrigir o código, você aponta ele para a devnet, você faz o deploy, e o *deploy* dá erro no IDL on-chain. Nada no seu Rust está errado. O problema é uma conta obsoleta do mundo antigo.

![Um build de 1.x passa mas o deploy tropeça numa conta de IDL on-chain legada; fechar ela uma vez com o CLI 0.32.1 limpa o caminho.](assets/v05-flowchart.webp)

O 1.0 removeu as instruções legadas de IDL on-chain (PR #3798). IDLs agora vão para on-chain através do Program Metadata Program em vez disso. Mas um programa que você herdou provavelmente teve deploy com uma conta de IDL criada do jeito antigo, e o caminho de deploy do 1.x não sabe como passar em volta dela. A correção é precisa e é uma jogada de uma vez: troque para o CLI 0.32.1, feche a conta de IDL legada com o `anchor idl close`, troque de volta para o 1.x, e faça o deploy. Você usa o CLI antigo exatamente uma vez, para exatamente isso. Não é um downgrade e não é permanente. O 1.x escreve IDLs perfeitamente bem, só através de um programa diferente.

Enquanto a gente está aqui embaixo, várias outras peças do toolchain mudaram de forma, e saber que elas se moveram te salva de caçar fantasmas:

- **O `anchor login` e o `[registry]` desapareceram.** O fluxo antigo onde você fazia login num registry para publicar um IDL não existe mais. IDLs vão para on-chain via o Program Metadata Program. Se a sua memória muscular recorre ao `anchor login`, pare, aquela porta está emparedada.
- **O LiteSVM é o template de teste default** (PR #4316). Scaffolds novos de `anchor init` levantam testes em Rust baseados em LiteSVM em vez da forma antiga de validador-num-loop.
- **O Surfpool é o validador default** (PR #4106), e ele precisa de pelo menos 1.1.2. O `anchor test` e o `anchor localnet` dirigem o Surfpool agora, não o `solana-test-validator`.
- **O CLI se desacoplou de um CLI solana externo** (PR #4099). O toolchain do Anchor empacota o que ele precisa agora em vez de chamar um binário solana instalado separadamente, que é por que o instalador do 1.x não te incomoda mais para casar uma versão específica de solana primeiro. Note a direção disso com cuidado, porque ela muda como você lê um número de versão. A versão do CLI do Anchor e a versão do CLI do Agave que você instalou agora são fatos independentes, então um pin como "Solana CLI 3.1.10" sentado no bloco de toolchain de um projeto é uma declaração sobre o ambiente de integração contínua daquele projeto, nunca uma afirmação sobre qual é o release atual da Solana. Leia isso como um pin local, não como uma manchete.
- **O `declare_program!` moveu os helpers gerados dele** de `utils` para `parsers`. O `declare_program!` é como um programa consome o IDL on-chain de outro programa para gerar um módulo de CPI e de cliente, e no 0.32 os parsers de conta gerados moravam embaixo de um submódulo `utils`. O 1.0 moveu eles para `parsers`, que lê como uma mudança cosmética de caminho até você perceber que é o tipo de quebra que o compilador pega instantaneamente e um grep pega mais rápido. Se o código que você herdou consome outro programa via `declare_program!` e alcança dentro do módulo `utils` gerado, atualize o caminho para `parsers` e siga em frente.

![Um lado a lado de seis preocupações de toolchain mostrando a ferramenta do 0.3x e a substituta dela no 1.0, de publicação de IDL até caminhos de módulo do declare_program!.](assets/v06-comparison.webp)

### O delta como evidência, não trivia

Dê um passo atrás dos seis itens e pergunte no que eles somam. Cada quebra rastreia para um limite que a forma antiga estava batendo: um handle gordo de CPI onde uma chave bastava, um literal contado na mão que deriva, códigos de erro colidindo, uma macro sob medida duplicando um despacho que já existia, um fluxo de IDL que ficou grande demais para um registry. Nenhuma delas é arbitrária. É essa a leitura que deixa uma decisão de migração legível em vez de assustadora.

![Uma tabela de referência de seis linhas mapeando cada quebra de 0.32-para-1.0 para a razão motivadora dela e a correção exata dela, cobrindo o rename, o CpiContext, o cálculo de space, o enum de error-code, a remoção de interface, e o IDL legado.](assets/v07-table.webp)

Vale amarrar isso de volta na trajetória que a gente montou no m01-l2, porque é isso que deixa quem migra e um leitor novo de novo no Anchor compartilharem uma história em vez de duas. Lá atrás o arco inteiro do framework foi enquadrado como um aperto lento: cada versão troca um pouco da antiga folga por um compilador que pega mais dos seus erros antes de eles chegarem num validador. O delta de 0.32-para-1.0 é esse mesmo arco, visto de dentro do único salto onde o aperto por acaso quebrou código. Um leitor que nunca escreveu uma linha de 0.32 ainda se beneficia de ler dessa forma, porque as *razões* são os princípios de design do framework que ele está aprendendo, não trivia de migração que ele pode esquecer. Quem migra recebe os mesmos princípios mais um plano de port. Uma narrativa, duas audiências.

Isso também comprova uma coisa que a conclusão deste módulo (m10-l4) vai formalizar numa árvore de decisão: mudanças que quebram custam horas de verdade. Você acabou de contar as horas. Quem migra bate em cada uma destas num programa não trivial, e o dado de que o nome antigo do pacote ainda supera o novo em downloads por quarenta para um comprova que a audiência para este trabalho é real e grande. Pessoas estão rodando código de 0.32 em produção agora mesmo e vão estar portando ele muito depois de esta lição ficar velha. O ponto de segurar o *por quê* de cada mudança é que, quando você portar no m10-l3, o "isto quebrou" seco do compilador vira o seu "certo, essa é a mudança três, aqui está a edição", sem um desvio por documentação que pode ela mesma ser uma lápide.

![Uma linha do tempo da criação do pacote novo no npm em 2025-12-19, passando pelo release do 1.0.0 em 2026-04-02, terminando onde o nome antigo ainda lidera por cerca de quarenta para um.](assets/v08-timeline.webp)

## Lab: reconhecimento de port

Nada é construído. A atividade é diagnóstico, e é trabalho de verdade: você vai fazer um programa de 0.32 falhar na linha 1.x de propósito, depois ler cada quebra que ele produz e mapear ela para as seis mudanças acima antes de corrigir uma linha só. Este é o reconhecimento que você faria no primeiro dia de um port de verdade, e fazer ele uma vez aqui é o que transforma o m10-l3 num checklist.

Lições anteriores te entregaram cada comando com a saída dele. Aqui você recebe as jogadas e lê o compilador você mesmo. É esse o recuo da ajuda em ação: no m10-l3 a única orientação que resta é o texto de erro do próprio compilador.

1. **Coloque um programa de 0.32 na sua frente.** Qualquer programa de Anchor 0.32 não trivial serve, e não vai ser um dos seus: cada degrau da escada do Quarters foi escrito contra o RC do V2 a partir de um arquivo em branco. Clone qualquer programa público de Anchor da era 0.32 que faça transferências de token. Confirme a versão que ele tem como alvo antes de começar:

   ```bash
   # Look at Anchor.toml [toolchain] and the anchor-lang pin in Cargo.toml
   rg "anchor_version|anchor-lang" Anchor.toml Cargo.toml
   ```

   Espere um pin de 0.3x nas duas linhas. Se ele já lê 1.x, este programa foi portado e é o sujeito errado para o exercício; ache um que não foi.

2. **Faça o inventário dos dois escopos de pacote.** Antes de tocar em Rust, ache cada import do lado do cliente, nos dois nomes:

   ```bash
   rg -l "@coral-xyz/anchor" .
   rg -l "@anchor-lang/core" .
   ```

   Anote qual escopo o código usa. Se é o antigo, isso é normal, essa é a realidade do quarenta para um. Você não está corrigindo isso ainda, você está contando.

3. **Suba o toolchain e force a quebra.** Aponte o avm para a linha 1.x e compile:

   ```bash
   avm install 1.1.2
   avm use 1.1.2
   anchor --version   # confirm you are on the 1.x line
   anchor build
   ```

   Espere o build falhar, alto e em vários lugares de uma vez. Essa falha é o entregável deste passo, não um problema para resolver ainda.

4. **Catalogue cada erro contra as seis mudanças.** Não corrija nada. Para cada erro de compilador, escreva o número da mudança para a qual ele mapeia. Você está procurando as impressões digitais: um atributo `#[interface]` desconhecido (mudança 5), um descasamento de `Pubkey` contra `AccountInfo` no `CpiContext::new` (mudança 2). Duas das seis não deixam erro de compilador nenhum, então elas têm que ser pegas por grep, não pelo build. Um literal de space feito na mão com `8 + ...` (mudança 3) não vai sempre dar erro alto; e um segundo enum `#[error_code]` (mudança 4) nunca dá erro — dois enums compilam verde, que é exatamente a armadilha sobre a qual a seção da mudança 4 avisou:

   ```bash
   rg "space\s*=\s*8\s*\+" .
   rg -n "#\[error_code\]" .   # more than one hit inside a single program crate: change 4
   ```

5. **Faça um dry-run da armadilha de deploy na sua cabeça, ou na devnet.** Você não vai corrigir o deploy hoje, mas localize o risco. Se o programa já teve deploy com um IDL on-chain do jeito antigo, note que um deploy de v1 vai tropeçar na conta de IDL obsoleta até você fechar ela com o CLI 0.32.1. Escreva o comando exato de close que você *rodaria*:

   ```bash
   # the one-time close, run on the 0.32.1 CLI, NOT 1.x:
   #   avm use 0.32.1
   #   anchor idl close <program-id> --provider.cluster devnet
   #   avm use 1.1.2
   ```

**Checkpoint.** Você terminou quando você tem uma lista escrita: cada quebra que o build produziu *mais* as duas silenciosas que os greps trouxeram à superfície, cada uma etiquetada com o número da mudança dela e a correção de uma linha dela, mais uma nota sobre se o close de IDL legado se aplica. Essa lista é um plano de port. Você não escreveu uma linha do port, e você já sabe exatamente o que ele vai dar. É essa a troca inteira que esta lição fez por você.

## Challenge

Feche o loop de memória. Aqui estão três trechos tirados de um programa de 0.32. Para cada um, sem olhar de volta para cima na página, nomeie a mudança que quebra do 1.0 que ele bate, a razão de a mudança existir, e a edição exata que corrige. Três triplas de mudança-para-razão-para-correção.

**Trecho A:**
```
let cpi_ctx = CpiContext::new(
    ctx.accounts.token_program.to_account_info(),
    Transfer { from, to, authority },
);
transfer(cpi_ctx, amount)?;
```

**Trecho B:**
```
#[interface(spl_transfer_hook_interface::execute)]
pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> { Ok(()) }
```

**Trecho C:**
```
#[account(init, payer = authority, space = 8 + 32 + 8)]
pub vault: Account<'info, Vault>,
```

Para ir mais longe: o seu programa herdado fez deploy bem no 0.32 mas o primeiro deploy dele no 1.x dá erro no IDL on-chain, e não tem bloco `#[interface]` nem enum `#[error_code]` extra em lugar nenhum do código. Qual é o passo de migração que falta, e por que "rode o `anchor login` e re-registre" é o instinto errado? Escreva a resposta antes de conferir ela contra a mudança 6.

## Antes de seguir em frente

Você não precisa ter corrigido nada para ter acertado esta lição. Você precisa da lista de reconhecimento do Lab e de três triplas limpas do Challenge. Se alguma tripla saiu difusa, a pista mais rápida é que você nomeou a *correção* mas não a *razão*, essa é a lacuna exata que deixa um port parecendo adivinhação, então volte naquela mudança e releia por que o framework se moveu, não só para o que ele se moveu. Se a sua lista de erros de build não incluiu um descasamento de tipo de `CpiContext` ou uma reclamação de `#[interface]`, o seu programa de teste provavelmente não exercitava CPIs ou hooks; rode o reconhecimento mais uma vez contra um que exercite, porque essas são as duas quebras que comem mais tempo num port de verdade. Você não precisa carregar aquele programa para frente: o m10-l3 coloca um vault de v1 na bancada e porta ele linha por linha.

Esse foi o primeiro delta: as dores de chegar *até* o 1.0. Mas a linha 1.x e o `anchor-next` são dois mundos paralelos, e o salto de 1.x para 2.0 é uma reescrita do zero, não um rename. O modelo de borrow do `CpiHandle` que eu te disse para deixar de lado duas vezes nesta lição mora lá. Na próxima lição a gente cruza essa linha e percorre cada lugar onde o código muda quando você vai de 1.x para dentro do V2. Bom port.
