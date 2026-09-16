# Entregue um cliente: o IDL, o Program Metadata, e um cliente Codama/kit

Na lição passada você rodou o checklist de auditoria contra o swap com um número de linha em cada linha, e depois dirigiu o `anchor fuzz` até um bug semeado produzir um artefato de crash que você repetiu, corrigiu e re-fuzzou limpo. O R4 está endurecido. Ele sobrevive às trocas que você joga nele e reverte as que ele deve. E ele é completamente inalcançável por qualquer pessoa que não seja você, sentado neste terminal, rodando uma bancada de teste em Rust. Um programa endurecido que só os seus próprios testes conseguem chamar é um vault trancado com a chave ainda no seu bolso.

Então o trabalho agora é um cliente. Um chamador de verdade. E aqui está a armadilha na qual você entra no momento em que você recorre a um, então deixe eu fazer você sentir ela antes de eu explicar. Abra um terminal no seu workspace e rode estes dois comandos:

```bash
npm view @solana/kit version
npm view @solana-program/token@0.15.0 peerDependencies
```

O primeiro te diz o `@solana/kit` mais novo no npm, que em 2026-08-22 é o `8.0.0`, e o segundo te diz o que o `@solana-program/token` de fato pede: `{ '@solana/kit': '^7.0.0' }`. Leia aquelas duas saídas uma ao lado da outra. O kit mais novo é a única versão que as suas dependências não querem. Instale o número que o npm chama de `latest` e o seu `npm install` joga um erro de peer-dependency antes de você escrever uma linha só de código de cliente. Essa lacuna, e como entregar mesmo assim sem mentir para si mesmo sobre ela, é a maior parte desta lição.

## Resumo

Você vai dar ao R4 um chamador. Quatro jogadas, em ordem de dependência, porque cada uma precisa da anterior:

1. Construa o **IDL** do programa, o contrato JSON que descreve cada instrução e cada conta.
2. **Publique aquele IDL on-chain** através do Program Metadata Program, para qualquer cliente conseguir buscar a sua interface no cluster em vez de no seu repositório.
3. **Gere um cliente `@solana/kit` tipado** a partir do IDL com o `anchor codama`, porque não existe pacote TypeScript oficial do Anchor V2 e a coisa que parece um não é um.
4. **Fixe o kit corretamente** e mande exatamente um swap através do builder gerado contra o seu deploy de devnet.

O recuo desta lição roda assim: os passos 1 até 4 do Lab estão completamente trabalhados, cada comando real e cada checkpoint checável. O passo 5, o tubo de send do kit, é um problema de completion: você recebe o esqueleto inteiro com as três linhas estruturais em branco. Depois o Challenge é solo, um arquivo autocontido em que você resolve qual versão de kit fixar a partir de uma faixa de peer e monta a chamada sem resposta trabalhada na sua frente.

Uma nota que colore tudo: o Anchor V2 é um release candidate de semanas de idade, e o ferramental de cliente em volta dele se move mais rápido que o framework. Cada número de versão aqui carrega a data em que eu verifiquei ele. Quando você chegar nesta lição, re-rode os dois comandos acima. Os números vão ter se movido. A *regra* não vai, e a regra é a coisa para a qual você está aqui.

## O caminho de entregar-um-cliente

Antes de você dirigir a rota, olhe o mapa. As quatro jogadas não são independentes. O IDL é a entrada para publicar ele e a entrada para gerar o cliente. O cliente gerado é a entrada para o send. Tire elas de ordem e você vai estar regerando um cliente contra um IDL que você nunca atualizou, que é o jeito mais comum de um cliente gerado entregar uma chamada que não casa mais com o programa.

![O IDL do anchor idl build alimenta tanto a publicação on-chain quanto a geração do Codama; o cliente gerado força o pin do kit, e o pin deixa o send possível.](assets/v01-flowchart.webp)

Note a forma. Publicar (B) e gerar (C) os dois se ramificam do IDL, e eles são independentes um do outro. Você consegue gerar um cliente sem nunca publicar o IDL on-chain, e você consegue publicar sem gerar. A gente faz os dois porque eles servem chamadores diferentes: publicar serve *qualquer pessoa*, gerar serve *você*. Mantenha essa divisão em mente, ela é a resposta para duas das perguntas de checagem no fim.

### O IDL é o contrato, e o v2 deliberadamente deixou ele em paz

Um arquivo IDL (Interface Description Language) é uma descrição em JSON do seu programa: o endereço dele, as instruções dele com os argumentos e as contas exigidas, os layouts de conta dele, os códigos de erro dele, e os discriminators de cada um. Ele é a versão legível por máquina de tudo o que você de outro jeito teria que ler do `lib.rs` na mão. Construa ele com o CLI:

```bash
anchor idl build --program-name token_ticket_swap -o target/idl/token_ticket_swap.json
```

Aqui está a parte que importa para um curso de framework, porque a expectativa corre para o outro lado. Uma reescrita `no_std` do zero soa como se ela devesse ter forkado o formato de interface, e ela não forkou: o IDL do v2 mantém os **mesmos discriminators de 8 bytes** que o v1 por padrão, e a *spec* do IDL em si está intocada — o código-fonte da spec na tag fixada v2.0.0-rc.1 é byte-por-byte idêntico ao da baseline 1.1.2. Essa identidade inclui os campos `serialization` e `repr`, os que descrevem como tipos serializam e como enums são dispostos na memória: os dois antecedem o V2 (eles chegaram com a reescrita de spec da era 0.30) e as duas linhas carregam eles no mesmo lugar do mesmo arquivo. O delta de verdade do V2 neste módulo não está no JSON de jeito nenhum; ele está em como o JSON chega no mundo — o caminho de publicação do CLI através do Program Metadata Program, que a próxima seção percorre. A estabilidade é a feature: uma chamada montada contra um IDL da era v1 ainda tem como alvo a instrução certa embaixo de um programa v2, e um IDL de v2 encaixa em qualquer ferramenta que já lê a spec moderna.

Se o discriminator continuar estável soa como uma nota de pé, lembre o que você fez quando computou um preimage de discriminator na mão mais cedo neste curso: você hasheou o nome com namespace da instrução e viu os primeiros oito bytes virarem o seletor no qual o runtime roteia. Aqueles oito bytes exatos são o que o IDL carrega no array `discriminator` de cada instrução. É por isso que uma chamada da era v1 ainda aterrissa contra um programa v2: o seletor não se moveu, e nem a descrição embrulhada em volta dele. Um cliente gerado lê aqueles bytes direto do IDL, então você nunca digita um discriminator na mão de novo, e você nunca erra o dedo em um dentro de uma chamada que caladamente tem a instrução errada como alvo.

![A spec do IDL é idêntica entre a baseline 1.x e a tag do v2 — discriminators, campos de serialization e de repr todos inalterados; um gerador de cliente que ignora serialization/repr é seguro só em tipos borsh default em qualquer uma das linhas.](assets/v02-annotated-code.webp)

Então esta é uma sondagem de tempo de escrita, não um fato que você pode congelar de mim. Antes de você confiar num cliente gerado para um programa que usa serialização não default ou um `repr` customizado — campos que a spec moderna carrega desde a era 0.30, nas duas linhas — confirme que a sua versão de gerador consome eles. Para o swap que você está entregando, o `Pool` é uma struct borsh simples e o `swap_arcade_for_tickets` pega dois `u64`, então você está seguramente dentro do que cada gerador trata. No momento em que você entregar um programa que não está, aquela sondagem é sua.

### Coloque o IDL on-chain para qualquer pessoa conseguir te chamar

Você tem um arquivo JSON. Um arquivo JSON no seu repositório ajuda exatamente as pessoas que têm o seu repositório. Para deixar uma carteira, um explorador, ou o script de um estranho resolver a sua interface a partir de nada além do id do seu programa, o IDL tem que morar na cadeia.

Pense do jeito que uma cidade trata um prédio. Qualquer pessoa consegue desenhar uma planta, mas a planta que *conta*, a que um construtor consegue puxar e construir contra, é a que está arquivada na prefeitura sob o endereço do prédio, emendável só pelo proprietário de registro. A versão da Solana daquele arquivo de arquivos é o **Program Metadata Program**. O Anchor largou as instruções de IDL embutidas dele lá no 1.0, e o V2 herda essa remoção: o IDL é guardado através deste programa, num endereço determinístico derivado do id do seu programa, gravável só pela autoridade de upgrade do programa. Então quando você digitar `anchor idl init` num momento, os verbos são antigos mas a maquinaria não é — os comandos da era 0.x do mesmo nome escreviam nas contas de IDL on-chain próprias do Anchor, o mecanismo que o 1.0 removeu, enquanto este CLI reusa os nomes de verbo como um front end para o Program Metadata Program, que é por que ensinar eles aqui não ressuscita o caminho aposentado.

![O Program Metadata Program guarda o IDL num PDA canônico derivado do id do programa; qualquer pessoa consegue ler ele, mas só a autoridade de upgrade pode escrever ou dar upgrade nele.](assets/v03-diagram.webp)

Os comandos são o subcomando `idl` do CLI do Anchor. A primeira publicação cria a conta on-chain; edições posteriores dão upgrade nela:

```bash
# First time: create the on-chain IDL account and write the IDL into it
anchor idl init -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet

# Later, after any program change that touches the interface
anchor idl upgrade -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet

# Prove it: fetch the IDL back from the chain, by program id alone
anchor idl fetch -o fetched.json <YOUR_SWAP_PROGRAM_ID> --provider.cluster devnet
```

Por baixo do capô estes escrevem através do Program Metadata Program. Checkpoint: depois do `idl init`, aquele último comando de `idl fetch` puxa o seu IDL para fora do cluster para dentro do `fetched.json` usando nada além do id do programa. Diffe ele contra o `target/idl/token_ticket_swap.json` e ele deve casar. Se o `idl init` falhar dizendo que a conta já existe, o culpado de sempre não é alguma sessão esquecida sua: o `anchor deploy` do RC sobe o IDL por padrão sempre que o `target/idl/<name>.json` existe, então um deploy simples já publicou para você. Ou passe o `--no-idl` na hora do deploy para manter publicar um passo explícito (o que o lab deste curso faz), ou aceite a subida automática e use o `idl upgrade` para cada edição posterior. Se ele falhar na autoridade, a carteira com a qual você está assinando não é a autoridade de upgrade do programa, e só aquela chave pode escrever.

Existe um trade-off de verdade em publicar, e eu prefiro que você ouça ele de mim e não de uma thread de suporte. A conta de IDL custa aluguel, e a cópia on-chain é atual só na medida da última publicação dela. No fluxo de `--no-idl` que este lab roda, isso quer dizer o seu último `idl upgrade`: publique uma vez, mude o programa, esqueça de dar upgrade no IDL, e agora cada cliente que confia na cadeia monta chamadas contra uma interface obsoleta. IDL on-chain é um compromisso de manter ele fresco, não um atire-e-esqueça.

### O estado honesto do cliente TypeScript do Anchor

Agora o cliente em si, e é aqui que eu vi mais gente perder uma tarde do que em qualquer outro lugar da história do V2. Deixe eu te levar escada acima do jeito que você de fato subiria, degrau errado primeiro.

Você já escreveu clientes de Anchor antes. No mundo 0.x você importava o pacote TypeScript do Anchor, entregava o seu IDL para ele, e chamava o `program.methods.swap(...)`. Então a jogada óbvia é achar a versão V2 daquele pacote e fazer o mesmo. Você busca, você acha o `@anchor-lang/core`, latest no npm `1.1.2` em 2026-08-22, ele é o sucessor renomeado do antigo `@coral-xyz/anchor`, e ele parece exatamente certo.

Ele não está certo. O `@anchor-lang/core` no `1.1.2` ainda depende do `@solana/web3.js` v1. Ele é o cliente da era v1 vestindo um nome novo. O Anchor publica um cliente TypeScript, mas não um de V2: o `@anchor-lang/core` é real, oficial, e está na linha v1, e nada substituiu ele para o V2. Recorra à coisa que parece oficial e você caladamente se fixou de volta no web3.js v1, a linha de SDK da qual o V2 existe para sair.

A jogada mais óbvia seguinte é escrever o cliente na mão: codifique o discriminator, serialize em borsh o `amountIn` e o `minOut`, monte a lista de `AccountMeta` na ordem exata que o programa espera, derive o PDA do pool você mesmo. Funciona. É também como uma chamada de swap acaba carregando as contas na ordem errada ou um off-by-one nos account metas, uma classe de bug que compila limpo e falha só on-chain. O cliente feito na mão é boilerplate que você reescreve, e erra sutilmente, para cada instrução, e esse custo é uma parte grande de por que geração first-party de cliente existe.

Carregue uma dissidência com você para dentro do caminho que a gente está a ponto de pegar, porque ela é apontada para aquele caminho e não para o que você acabou de rejeitar. O ChewingGlass, na discussão #3742 do Anchor: "Codama doesn't resolve has_ones. Anchor does... I still feel quite boilerplate-y fetching and passing heaps of accounts in codama. Boilerplate kills new devs." Essa é uma acusação justa e vale segurar ela enquanto você usa a ferramenta. Um cliente kit gerado ainda te faz buscar e passar contas que uma chamada de `program.methods` da era v1 teria resolvido a partir de constraints para você. Geração compra corretude na ordem das contas, nas flags e na codificação. Ela não compra de volta a ergonomia do cliente TypeScript antigo. Pegue a corretude, e continue irritado com o resto.

Então o caminho de verdade, o que o CLI do Anchor entrega, é o **Codama**. O Codama é um gerador de cliente: ele lê um IDL e emite um cliente tipado. O CLI do Anchor embrulha ele em dois subcomandos, então você não instala nem configura o Codama separadamente, o CLI fixa a versão que ele usa (`CODAMA_VERSION = 1.6.0` dentro do CLI em 2026-08-22) e dirige ele para você.

![O @anchor-lang/core é a linha de SDK errada, fazer na mão convida bugs silenciosos de conta, e o anchor codama generate produz um cliente kit cujo único custo de verdade é disciplina de regeração.](assets/v04-comparison.webp)

Dois comandos. O primeiro converte o IDL do Anchor na árvore de IDL própria do Codama. O segundo roda aquela conversão in-process e depois renderiza o cliente:

```bash
# Convert the Anchor IDL to a Codama IDL (inspect it if you like)
anchor codama convert target/idl/token_ticket_swap.json --out codama-idl.json

# Convert + render a JavaScript/TypeScript client into clients/
anchor codama generate -l js -p clients target/idl/token_ticket_swap.json
```

Dois comandos, não um, e a divisão é deliberada. O `convert` é a metade honesta que você consegue inspecionar: ele escreve o IDL do Codama num arquivo que você consegue abrir e diffar, então quando uma chamada gerada parece errada você consegue ver se a culpa está na conversão ou no seu programa. O `generate` roda aquela mesma conversão in-process e depois entrega ela para os renderizadores do Codama, então no trabalho do dia a dia você roda só o `generate`. Recorra ao `convert` no dia em que um builder gerado te surpreender e você quiser ler o que o Codama pensa que o seu programa é.

O que aterrissa embaixo de `clients/` não é um blob. A flag `-p` nomeia um diretório base e o CLI escreve cada linguagem em `<base>/<language>`, então `-p clients -l js` renderiza para dentro de `clients/js/`. Dentro dele, o Codama emite um diretório que você consegue ler, uma pasta por tipo de coisa no seu programa:

![O cliente gerado é um diretório de instructions, accounts, pdas, types, programs e errors; quem chama usa o builder de instrução de instructions, o achador de PDA do pool de pdas, e o decodificador de conta de accounts.](assets/v05-diagram.webp)

O builder é nomeado com base na sua instrução. O `swap_arcade_for_tickets` vira o `getSwapArcadeForTicketsInstructionAsync`. O sufixo `-Async` é a convenção do Codama para a variante que resolve o que ela consegue para você: ela deriva o PDA do `pool` a partir das seeds dele e preenche endereços de programa default, então você passa as contas que só você consegue saber (o trader, os mints, as contas de token de reserva e de trader) e ela monta o resto. É esse o ponto inteiro de geração. A ordem das contas, o discriminator, a codificação borsh do `amountIn` e do `minOut`, a derivação do PDA, tudo isso sai do seu IDL em vez de sair da sua memória.

Coloque a razão na página, porque é aqui que fazer na mão de fato morde. O `#[derive(Accounts)]` do swap lista nove contas numa ordem fixa, e o runtime casa elas posicionalmente, por slot, não por nome. Faça a chamada na mão e você está redigitando aquela ordem para dentro de um array de `AccountMeta` de memória, onde trocar o `reserve_arcade` e o `reserve_ticket`, ou marcar o `trader` como somente-leitura quando ele tem que assinar, compila limpo e falha só quando a troca bate na cadeia. O builder gerado lê a ordem e as flags de gravável/signer do IDL e pede cada conta para você pelo nome. As duas reservas que você nunca pode confundir chegam como `reserveArcade` e `reserveTicket`, rotuladas, no único lugar em que um typo de outro jeito seria invisível.

![Fazer na mão as nove contas posicionais do swap falha em silêncio quando dois slots de reserva são trocados, enquanto o builder gerado pega contas por nome e deriva o PDA do pool ele mesmo.](assets/v06-comparison.webp)

### O pin do kit: case com os seus peers, nunca persiga o latest

Agora de volta à armadilha que você sentiu no topo desta lição, porque agora você tem as peças para entender ela. O cliente gerado é um cliente `@solana/kit`, e ele se apoia nos pacotes `@solana-program/*` (`@solana-program/system`, `@solana-program/token`) para as peças de system e de token. Aqueles pacotes declaram uma peer dependency no kit. Em 2026-08-22, o `@solana-program/system@0.13.0` e o `@solana-program/token@0.15.0` os dois fazem peer no `@solana/kit` `^7.0.0`. E o kit `latest` do npm é o `8.0.0`, publicado em 2026-08-21.

Então o kit mais novo e o kit que as suas dependências querem são majors diferentes. Isso não é um acaso de uma semana ruim, é a textura normal de um SDK que se move rápido: a biblioteca central entrega um major novo à frente do ecossistema que faz peer nela. Olhe a semana em que aconteceu.

![Em 2026-08-21 o web3.js legado ainda batia o kit 1,882,726 contra 1,738,844 em downloads semanais, e o kit entregou o 8.0.0 no mesmo dia, enquanto o ecossistema ainda fazia peer no kit ^7.](assets/v07-chart.webp)

As duas metades daquele gráfico são verdade na mesma semana: o cruzamento de downloads diz que o kit é para onde o ecossistema está indo, e as faixas de peer dizem para não perseguir o número de versão dele.

Então a regra durável, a única coisa para carregar para fora desta lição se você não carregar nada mais: **fixe o `@solana/kit` no major que as suas dependências de `@solana-program` declaram, nunca no `latest`.** É a mesma lei que dois cursos irmãos ensinam das próprias cadeiras deles — o Payments e Commerce aplica ela por workspace, onde dois workspaces num repositório só legitimamente fixam majors de kit diferentes porque cada um casa com os peers próprios dele, e o Rust & TypeScript Fundamentals deriva ela de como faixas de peer do npm resolvem para começo de conversa. Hoje aquele major é o 7. Instale ele explicitamente:

```bash
# Freshness: verified 2026-08-22. kit latest is 8.0.0, but the @solana-program
# packages below peer on ^7, so we pin ^7. Re-run `npm view ... peerDependencies`
# when you reach this; the numbers move, the rule does not.
npm install @solana/kit@^7 @solana-program/system@0.13.0 @solana-program/token@0.15.0
```

Por que não fixar só no `latest` e deixar o npm resolver? Por causa do que o npm faz com uma peer dependency. Quando o `@solana-program/token@0.15.0` declara `peerDependencies: { '@solana/kit': '^7.0.0' }`, ele está dizendo para o seu gerenciador de pacotes "eu só vou rodar contra um kit 7". Instale o kit 8 ao lado dele e o npm não consegue satisfazer aquela restrição, então ele para com um erro de peer-conflict `ERESOLVE` antes de qualquer coisa compilar. O `latest` é um alvo em movimento que uma subida de major do kit consegue transformar em exatamente aquele conflito da noite para o dia, e transformou, no dia em que o kit 8 foi entregue. Fixe o número em que os seus peers concordam e a sua instalação é reprodutível até *você* decidir mover ela, de propósito, depois de você ter checado que as faixas de peer se moveram também. É essa a troca que você está fazendo em todo lugar deste curso: você desiste do "sempre o mais novo" e você recebe o "sempre resolvível".

### Mais uma interface, já nas suas mãos: o declare_program!

Existe um segundo consumidor do IDL que você já encontrou do outro lado. No v1, um programa conseguia consumir o IDL de *outro* programa em tempo de compilação através do `declare_program!`, gerando uma interface de CPI a partir de um IDL vendorado — um programa Anchor chamando outro pela interface publicada dele em vez de por versões de código que casam. É exatamente isso que o seu escrow vem fazendo com o vault desde o m04-l3, e está verificado funcionando na linha 2.0.0-rc.1 que este curso fixa: o JSON que você vem extraindo para dentro de `idls/` é um IDL fazendo a metade de tempo de compilação do trabalho que a metade de lado de cliente desta lição descreve. O capstone se apoia nele com quatro degraus de largura no m09-l3.

Vale saber enquanto você está aqui: a razão de o curso consumir programas por IDL em vez de por código não é gosto. A feature `cpi` de nível de código do scaffold para em um programa consumido por binário no RC — um segundo colide na hora do link num símbolo de despacho sem mangling — então o caminho de IDL é tanto o mecanismo próprio do V2 quanto o único que escala para o salão de quatro degraus. Onde a mesma jogada é usada de verdade além deste curso: consumir o IDL publicado de um protocolo vivo diretamente, que é o território do curso de DeFi e RWA Engineering.

## O Lab

Checagem de recuo antes de começar: os passos 1 até 4 estão trabalhados, você roda eles e vê cada checkpoint ficar verde. O passo 5 é o problema de completion, o tubo do kit com as mesmas três linhas em branco para você preencher. Depois o Challenge é solo.

Instalação de ferramental no primeiro uso. Você construiu o RC do Anchor V2 a partir do canal git dele lá no R0, então isto é um confirmar, não uma instalação nova. Lembre por que não tem linha de `avm use` aqui: nenhum GitHub Release foi cortado para a tag v2, então o binário pré-compilado que o `avm install` baixa não está lá e o fetch dá 404.

```bash
# If the RC is not on this machine, rebuild it from the documented channel:
# cargo install --git https://github.com/otter-sec/anchor.git \
#   --tag v2.0.0-rc.1 anchor-cli --locked --force
which anchor       # ~/.cargo/bin/anchor either way (the avm shim lives at the same path) — only proves it's on PATH
anchor --version   # the real check: anchor-cli 2.0.0-rc.1 (freshness 2026-08-22; RC, re-check)
node --version     # anchor codama drives @codama/cli via npx, so Node must be present
```

**1. Faça o deploy do swap, e depois construa e publique o IDL dele.** O R4 só rodou em LiteSVM e em Surfpool, então ele precisa estar no cluster antes de qualquer coisa aqui funcionar: a conta de IDL é chaveada a um id de programa real, e um cliente precisa de um endereço para chamar. O `anchor deploy` usa o keypair de programa do workspace, então o id que ele imprime continua seu pelo resto do curso.

```bash
# -p: deploy ONLY the swap. A bare `anchor deploy` loops every program in the
#     workspace, prints one Program Id per program, and pays ProgramData rent for
#     each — far more than one airdrop covers at this point in the course.
# --no-idl: the RC uploads target/idl/<name>.json during deploy by default; skip
#     that here so the publish stays the explicit `idl init` two lines down.
anchor deploy -p token_ticket_swap --no-idl --provider.cluster devnet
                                          # prints Program Id -> <YOUR_SWAP_PROGRAM_ID>
                                          # short on funds? solana airdrop 2 -u devnet, retry
anchor idl build --program-name token_ticket_swap -o target/idl/token_ticket_swap.json
anchor idl init -f target/idl/token_ticket_swap.json <YOUR_SWAP_PROGRAM_ID> \
  --provider.cluster devnet
anchor idl fetch -o fetched.json <YOUR_SWAP_PROGRAM_ID> --provider.cluster devnet
```

Checkpoint: o `anchor deploy` imprime um Program Id, registre ele no seu arquivo de pins como `<YOUR_SWAP_PROGRAM_ID>`; depois o `fetched.json` existe e diffa limpo contra o `target/idl/token_ticket_swap.json`. Qualquer cliente na terra agora consegue resolver a interface do seu swap a partir do id do programa sozinho.

**1b. Levante um workspace node para o cliente.** O cliente gerado é TypeScript, e nada num workspace de Anchor cria um. Faça agora, na raiz do repositório, para o `npx tsc` ter alguma coisa para olhar:

```bash
npm init -y
cat > tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "target": "es2022",
    "module": "es2022",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["clients/**/*.ts", "app/**/*.ts"]
}
JSON
mkdir -p app
```

Checkpoint: o `tsconfig.json` e o `package.json` existem na raiz do repositório. Aquele `include` é o que faz o `tsc --noEmit` do passo 4 checar os tipos do cliente gerado em vez de reportar "No inputs were found", e o `app/` é onde o seu script de send aterrissa no passo 5, que é o que faz o import de `../clients/js` dele resolver.

**2. Gere o cliente kit.** Converta e renderize num comando.

```bash
anchor codama generate -l js -p clients target/idl/token_ticket_swap.json
```

Checkpoint: o `clients/js/instructions/` contém um builder `getSwapArcadeForTicketsInstructionAsync`, o `clients/js/pdas/` contém o `findPoolPda`, o `clients/js/accounts/` contém o `fetchPool`, e o `clients/js/errors/` contém as suas variantes de `SwapError` — o renderizador JS do Codama separa achadores de PDA na pasta `pdas/` própria deles, então não saia caçando o achador embaixo de `accounts/`. Se a pasta está vazia, o `npx` não conseguiu buscar o `@codama/cli`, cheque que o Node está no seu PATH e rode de novo.

**3. Fixe o kit no major dos peers.** Instale as dependências de runtime do cliente, fixadas no major em que os seus pacotes `@solana-program` fazem peer.

```bash
npm install @solana/kit@^7 @solana-program/system@0.13.0 @solana-program/token@0.15.0
```

Checkpoint: a instalação completa sem erro de peer-dependency. Se ela joga um mencionando o `@solana/kit@8`, alguma coisa puxou o `latest`, corrija de volta para `^7`.

**4. Cheque os tipos do cliente gerado.** Esta é a trava de verificação da metade de cliente inteira da lição. O TypeScript é a única ferramenta que este lab ainda não instalou, então adicione ele como dependência de dev em vez de deixar o `npx` buscar uma flutuante:

```bash
npm install -D typescript@5.9.2   # pinned on purpose; use whatever your workspace already pins
npx tsc --noEmit
```

Checkpoint: zero erros. Um cliente tipado que não passa na checagem de tipos não é um cliente, é um passivo. Este é o mesmo `tsc --noEmit` que trava a tarefa, então deixar ele verde aqui é deixar ele verde lá.

**5. Mande um swap (problema de completion).** Aqui está o tubo de send do kit, e ele vai em `app/send-swap.ts`. Três linhas estruturais estão em branco. Preencha elas: quem paga a taxa é quem chama, o lifetime é o blockhash recente, e a única instrução acrescentada é a que o seu cliente gerado monta. Este é o esqueleto exato que o send segue.

Antes de ele conseguir rodar, seis daqueles endereços têm que existir na devnet, e nada até aqui criou eles. Faça isso primeiro: o mesmo CLI de `spl-token` que você usou para a leitura de Token-2022 no módulo 5, depois uma chamada para o `init_pool` próprio do swap, depois uma leitura de duas linhas para descobrir o que o `init_pool` criou, e depois mais duas linhas de `spl-token` que precisam do que a leitura te disse.

```bash
solana config set --url devnet

# The two mints, and the trader's token accounts.
spl-token create-token --decimals 6            # -> <ARCADE_MINT>
spl-token create-token --decimals 6            # -> <TICKET_MINT>
spl-token create-account <ARCADE_MINT>         # -> <TRADER_ARCADE_ATA>
spl-token create-account <TICKET_MINT>         # -> <TRADER_TICKET_ATA>
spl-token mint <ARCADE_MINT> 1000              # give the trader something to swap
# Read that last line precisely: `spl-token mint` takes an optional third argument,
# the recipient TOKEN ACCOUNT, and its default is "the mint authority's own ATA" —
# you. Nothing you have typed so far puts a single token inside the pool.

# The pool and its two reserves. `init_pool` is the instruction you wrote in the
# swap lab; the generated client has a builder for it too, so send it the same way
# the pipe below sends the swap — same pipe, different builder. It prints nothing,
# which is exactly why the next step exists: you cannot fund what you cannot name.
```

O `init_pool` criou duas contas de token de reserva e não te disse nenhum dos dois endereços, e as próximas duas linhas de shell precisam dos dois. Leia eles do registro do pool com a outra metade do cliente que você acabou de gerar — o `findPoolPda` de `pdas/`, o `fetchPool` de `accounts/`, os dois re-exportados da raiz do cliente. Sem borsh manual, sem explorer:

```typescript
// app/read-pool.ts — run this once, right here, before you fund anything.
import { createSolanaRpc } from '@solana/kit';
import { fetchPool, findPoolPda } from '../clients/js';

const rpc = createSolanaRpc('https://api.devnet.solana.com');
const [poolPda] = await findPoolPda();
const pool = await fetchPool(rpc, poolPda);
// pool.data.arcadeMint / .ticketMint / .arcadeReserve / .ticketReserve / .bump, all typed
console.log(pool.data.arcadeReserve, pool.data.ticketReserve);
```

Aqueles dois endereços impressos são o `<POOL_ARCADE_RESERVE>` e o `<POOL_TICKET_RESERVE>` abaixo — a correção de auditoria do m07-l3 é o que fez o pool guardar os dois endereços de reserva ao lado dos mints e do bump, e esta é a lição em que você cobra isso. O decode dobra como a sua prova de que o `init_pool` aterrissou: se o `pool.data.bump` ler de volta como o bump canônico guardado e os dois mints casarem com o que você fez deploy, o registro é real. É também a metade de leitura do cliente gerado fazendo trabalho de verdade em vez de demo — você precisou dela para avançar, não para admirar.

```bash
# Now fund the reserves, because `init_pool` CREATES the two reserve token accounts
# and leaves them empty, and R4 has no deposit instruction — you never wrote one.
# An empty reserve makes swap_out return 0 and the `require!(out > 0, ZeroOutput)`
# guard reject every trade, so skip these two lines and the swap below cannot land.
# You are still both mints' authority, so mint straight in by naming the reserve as
# the recipient: the third argument the `mint 1000` line above deliberately left off.
spl-token mint <ARCADE_MINT> 1 <POOL_ARCADE_RESERVE>   # 1.000000 -> 1_000_000 base units
spl-token mint <TICKET_MINT> 1 <POOL_TICKET_RESERVE>   # the same, so the pool starts balanced
```

Aquelas duas linhas de mint deixam o pool segurando 1,000,000 / 1,000,000 unidades base, que é deliberadamente o par de reserva que o exemplo trabalhado do m05-l2 usou: um `amountIn` de `10_000` cota 9,871 tickets de saída, confortavelmente livre do piso de `minOut` de `9_800` no send abaixo. Semeie uma profundidade diferente e recompute aquele piso antes de mandar, ou a sua própria trava de slippage vai te rejeitar — que é a trava funcionando, não um bug. Também: o `secretKey` na assinatura abaixo são os 64 bytes do seu arquivo de keypair de devnet, que você consegue carregar com `new Uint8Array(JSON.parse(fs.readFileSync(process.env.HOME + '/.config/solana/id.json', 'utf8')))`.

```typescript
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromBytes,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
  getSignatureFromTransaction,
  assertIsTransactionWithBlockhashLifetime,
  address,
} from '@solana/kit';
import { getSwapArcadeForTicketsInstructionAsync } from '../clients/js';

async function sendSwap(secretKey: Uint8Array): Promise<string> {
  const rpc = createSolanaRpc('https://api.devnet.solana.com');
  const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');
  const trader = await createKeyPairSignerFromBytes(secretKey);

  // The generated async builder derives the pool PDA and default programs for us.
  const swapIx = await getSwapArcadeForTicketsInstructionAsync({
    trader,
    mintArcade: address('<ARCADE_MINT>'),
    mintTicket: address('<TICKET_MINT>'),
    reserveArcade: address('<POOL_ARCADE_RESERVE>'),
    reserveTicket: address('<POOL_TICKET_RESERVE>'),
    traderArcade: address('<TRADER_ARCADE_ATA>'),
    traderTicket: address('<TRADER_TICKET_ATA>'),
    amountIn: 10_000n,
    minOut: 9_800n,
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => /* FILL: set the fee payer to the trader */ m,
    (m) => /* FILL: set the lifetime to latestBlockhash */ m,
    (m) => /* FILL: append the generated swapIx */ m,
  );

  const signedTx = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signedTx);

  const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
  await sendAndConfirm(signedTx, { commitment: 'confirmed' });
  return getSignatureFromTransaction(signedTx);
}
```

Os três preenchimentos, para você se checar depois de ter tentado eles: `(m) => setTransactionMessageFeePayerSigner(trader, m)`, depois `(m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m)`, depois `(m) => appendTransactionMessageInstruction(swapIx, m)`. Note que o `amountIn` e o `minOut` são `bigint`s, não números; aquele sufixo `n` é como o kit carrega um `u64` sem perder precisão acima de 2^53.

Duas especificidades do kit que valem nomear enquanto elas estão na sua frente. O `sendAndConfirmTransactionFactory` pega tanto o `rpc` quanto o `rpcSubscriptions`, porque o kit confirma escutando num websocket pela assinatura em vez de ficar consultando, que é por que você criou um cliente de subscriptions ao lado do de RPC. E o `assertIsTransactionWithBlockhashLifetime` não é cerimônia: ele é uma guarda de tipo que se recusa a compilar o send a não ser que a mensagem de fato carregue um lifetime de blockhash, então esquecer a linha de lifetime vira um erro de tipo na sua mesa em vez de uma transação derrubada na devnet. Fazer a troca *aterrissar* de forma confiável sob carga real é um ofício separado, e ele pertence ao curso de Client-Side Mastery. Aqui você está comprovando que a chamada é bem-formada e confirmável, não afinando ela para um líder congestionado.

![Uma transação de kit é montada definindo quem paga a taxa, o lifetime de blockhash e a instrução, e depois assinada, protegida, mandada e confirmada, e a assinatura dela lida de volta.](assets/v08-flowchart.webp)

Checkpoint para o lab inteiro: o `sendSwap` retorna uma assinatura, e aquela assinatura resolve num explorador de devnet como um swap confirmado. Isso é um chamador, diferente de você, movendo o R4. A chave do vault está fora do seu bolso.

E a metade de leitura já está comprovada, porque você não teria chegado aqui sem ela: o `findPoolPda` derivou o pool e o `fetchPool` decodificou ele lá no passo 5, tipado e sem borsh manual, e os dois endereços de reserva que ele te entregou são as contas que você financiou e contra as quais acabou de negociar. O builder escreve chamadas, o decodificador lê estado, e os dois saíram do IDL único.

## O Challenge

Agora solo, sem resposta trabalhada na sua frente. O challenge é o `wire-kit-swap-client`, um arquivo TypeScript autocontido (`starter.ts` e o `tests.json` dele embaixo do diretório de challenge desta lição). Ele descasca o send até as duas decisões que são de fato suas, para ele avaliar deterministicamente sem RPC e sem assinatura.

O seu `planSwapClient` pega seis argumentos posicionais, nesta ordem: `owner`, o endereço de quem chama; os dois números da troca, `amountIn` e `minOut` (os dois `bigint`); `recentBlockhash`; `splPeer`, uma string como `"^7.0.0"` (a faixa que o `@solana-program/token` declara); e `kitLatest`, uma string como `"8.0.0"` (o `latest` do npm, a armadilha). Abaixo dele no mesmo arquivo fica o `swapInstruction(owner, amountIn, minOut)`, um builder fornecido fazendo o papel do gerado — trate ele como um dado. Você retorna quatro coisas:

- `pinnedKitMajor`: o major do `splPeer`, mesmo quando o `kitLatest` é mais novo. O pin vem da faixa de peer, nunca do latest.
- `feePayer`: quem chama (`owner`).
- `lifetime`: o blockhash recente.
- `instructions`: exatamente uma, o swap do builder fornecido `swapInstruction(owner, amountIn, minOut)`, carregando tanto o `amountIn` quanto o `minOut`.

Não reconstrua a instrução de swap na mão, essa é a razão inteira de você ter gerado um cliente. Chame o builder fornecido. Aceitação: o `pinnedKitMajor` é igual ao major no `splPeer` mesmo quando o `kitLatest` é `9.9.9`, quem paga a taxa é o owner e o lifetime é o blockhash recente, e exatamente uma instrução é acrescentada carregando tanto a quantia de entrada quanto o piso de slippage. Os testes checam cada um dos quatro campos retornados separadamente, então uma resposta meio preenchida te diz exatamente qual subobjetivo você perdeu em vez de só ficar vermelha.

## Antes de seguir em frente

Pare e cheque a forma de resposta que você devia produzir: um diretório `clients/` que passa na checagem de tipos sob o `npx tsc --noEmit`, um `package.json` fixando o `@solana/kit` em `^7`, e uma assinatura de swap de devnet confirmada do builder gerado. Se você tem essas três, você entregou um cliente honestamente em ferramental de fronteira. Se você só leu esta lição, você não entregou, isto é um build, e o swap tem que aterrissar.

A troca que você fez, dita sem enfeite para você carregar para frente: um cliente gerado é atual só na medida do IDL que você publicou e da versão do Codama que você fixou. Pule um `idl upgrade` depois de uma mudança de programa e chamadores montam contra uma interface obsoleta. Persiga o `latest` no kit e você quebra o grafo de peers do `@solana-program` no dia em que um major novo é entregue. Você trocou controle escrito na mão por disciplina de regeração, e essa disciplina é a data `verified` em cada pin.

Mais uma direção, para você saber para onde este cliente vai em seguida. Conseguir uma transação bem-formada *montada* é esta lição. Conseguir que ela *aterrisse* de forma confiável sob carga, taxas de prioridade, retentativas, a arte inteira do pouso de transação, é o trabalho do curso de Client-Side Mastery. Este curso é dono do framework e da interface, ele repassa estratégia de pouso para o curso que é dono dela.

Você consegue chamar o swap agora. Mas alguém consegue comprovar que os bytes rodando na devnet foram montados a partir do seu código, e não trocados por outra coisa depois de você olhar para o outro lado? Na próxima lição você roda um build determinístico em Docker, faz deploy dele, e verifica os bytes on-chain contra o seu código com um comando, e depois encontra o fato incômodo de que a cadeia de verify inteira se apoia num guardião único.
