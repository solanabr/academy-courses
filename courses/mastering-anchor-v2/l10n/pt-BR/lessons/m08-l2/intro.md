# Comprove: os bytes on-chain casam com o seu código-fonte

Na lição passada você gerou um cliente kit e publicou o IDL on-chain, então qualquer coisa consegue chamar o swap agora através de um builder tipado. O cliente confia no IDL. O IDL confia no programa que recebeu deploy. E o programa que recebeu deploy? Agora mesmo você está pedindo para todo mundo aceitar a sua palavra de que os bytes rodando no cluster são os bytes que estão no seu repo.

É essa a lacuna que eu quero fechar. Qualquer pessoa consegue fazer deploy de um programa, apontar para um repo público no GitHub, e dizer "este é o código-fonte." Nada num programa que recebeu deploy força essa afirmação a ser verdadeira. A conta on-chain é só um blob de bytecode sBPF. Ela não carrega link nenhum de volta para um commit, assinatura nenhuma de um compilador, nada. Então o leitor do seu repo e o usuário do seu programa estão confiando em dois artefatos diferentes e torcendo para que sejam o mesmo.

Antes de qualquer teoria, comprove para você mesmo que os bytes até têm uma impressão digital. O programa embaixo do microscópio pelo resto desta lição é o R4, o crate de swap `token_ticket_swap` cujo IDL você publicou na lição passada. Pegue o hash do seu build local dele agora mesmo:

```bash
# Install once (Rust toolchain required). Pin the version. To see what is newer:
#   cargo search solana-verify   (reads crates.io, i.e. what is available)
# `solana-verify --version` only tells you what YOU installed, so it is the
# confirmation step, not the freshness check.
cargo install solana-verify --version 0.5.1   # latest as of 2026-08-22, re-check before you pin

# Fingerprint your compiled program. This is the "before" of everything that follows.
solana-verify get-executable-hash target/deploy/token_ticket_swap.so
```

Esse comando imprime um único hash sha256 do executável. Anote ele. É uma string de 64 caracteres que muda se até uma instrução só do binário mudar. A lição inteira é construída em cima de uma ideia: se dois hashes produzidos de forma independente casam, os dois builds são byte a byte idênticos, e se eles diferem, alguma coisa se moveu. Todo o resto é encanamento em volta dessa comparação.

## Resumo

Você vai comprovar, na devnet, que um programa que recebeu deploy foi construído a partir de um código-fonte específico num toolchain fixado. Depois você vai quebrar ele de propósito e ver a prova falhar. Depois você vai olhar firme para o que a prova não cobre, porque é nessa lacuna que a confiança de verdade mora.

A forma da coisa é esta:

- **Um build verificável é um build determinístico.** Compile o mesmo código-fonte em duas máquinas diferentes com a CLI do Solana e você consegue dois binários diferentes, porque caminhos de build e versões de toolchain vazam para dentro dos bytes. O `solana-verify build` roda a compilação dentro de uma imagem Docker fixada para que a saída seja reproduzível. Mesmo código-fonte mais mesmo toolchain fixado é igual a mesmo hash, em qualquer máquina.
- **O `verify-from-repo` é a prova inteira, e ele funciona na devnet.** Ele refaz o build do seu programa a partir de um repo público dentro daquela imagem fixada, tira o hash do resultado, busca o hash do programa on-chain no cluster que você nomear, e reporta casamento ou descasamento. Aponte ele para a devnet e ele comprova o seu deploy de devnet localmente e sem precisar confiar em ninguém. Nenhum terceiro é necessário.
- **Um casamento comprova procedência, não segurança.** Ele comprova que os bytes que receberam deploy vieram exatamente deste código-fonte neste toolchain. Ele não diz nada sobre o código-fonte estar correto, e nada sobre quem controla os upgrades. Essas são provas separadas que você já fez (a sua auditoria, a sua passada de fuzz) ou vai fazer (autoridade).
- **A cadeia de verify inteira se apoia num guardião só.** A OtterSec constrói o framework Anchor, publica os crates dele, e roda o registry de builds verificados contra o qual o Anchor verifica. Esse é um ponto único de confiança real e honesto, e eu vou te mostrar como ver isso você mesmo nos metadados do próprio npm.
- **O submit remoto para o registry e o repasse de autoridade do Squads são só de mainnet.** Você vai ler os dois, narrados de ponta a ponta, claramente rotulados como além do cluster deste curso. Nada na seção travada roda na devnet, e eu vou dizer isso toda vez.

O recuo desta lição: eu rodo o ciclo completo de build, deploy e verify de ponta a ponta no lab, com o casamento de hash da devnet na tela, e todo comando dele é um que você roda contra o seu próprio id de programa e o seu próprio repo. O degrau solo é o descasamento: mude uma linha, refaça o build, faça deploy de novo, e faça a prova ficar vermelha, depois diga em uma frase o que um resultado verde compra e o que ele não compra para você. Esta lição é um build e um julgamento, sem problema de completion no meio.

![Uma cadeia de quatro caixas mostra o cliente kit confiando no IDL publicado, que confia no programa que recebeu deploy, cujo link de volta para o repo de código-fonte fica sem prova.](assets/v01-flowchart.png)

## Do código-fonte para os bytes e de volta

Comece pela dor, porque ela não é óbvia até você bater nela. Você faz o build do `token_ticket_swap` no seu laptop, o seu colega de time faz o build do mesmo commit no dele, e os dois arquivos `.so` dão hashes diferentes. Ninguém editou o código-fonte. Então o que se moveu?

O build normal da CLI do Solana embute detalhes específicos da máquina dentro do binário. Caminhos de build absolutos, a versão exata do compilador, ordenação incidental, tudo isso consegue sangrar para dentro dos bytes. Isso não é uma mania da Solana, é como compilação nativa funciona. A consequência é que "aqui está o meu código-fonte" e "aqui está o meu binário" não conseguem ser conferidos um contra o outro a não ser que todo mundo concorde, até a versão, em como o binário foi produzido. Uma comparação de hash só tem significado se o build for determinístico.

Você pode recorrer às correções óbvias primeiro, e vale ver por que cada uma fica curta, porque é isso que força a solução de verdade. Commitar um `Cargo.lock` e fixar toda dependência? Necessário, mas não suficiente: duas máquinas com builds de rustc diferentes ainda divergem, e o lockfile não diz nada sobre o compilador. Publicar as suas versões exatas de rustc e de Solana no README e pedir para as pessoas casarem elas na mão? Melhor, mas agora você está confiando que todo verificador reconstrua um ambiente passo a passo, e qualquer biblioteca de sistema transitiva que se descole ainda consegue mover um byte. O padrão está claro. Fixação parcial sempre deixa uma variável livre, e uma variável livre só quebra o hash. A única correção que fecha todas elas de uma vez é entregar o ambiente em si.

A bala de prata é o Docker. O `solana-verify build` roda a compilação dentro de uma imagem fixada com um toolchain fixo e um ambiente fixo, então o mesmo código-fonte produz os mesmos bytes não importa a máquina de quem esteja embaixo. Você não está mais confiando no build, você está confiando nos pins. Esta é a mesma jogada que um engenheiro de pontes faz quando ele especifica o grau exato do aço em vez de "algum metal forte": determinismo vem de remover as variáveis livres, não de ser cuidadoso.

O custo é real e eu quero ele na mesa. Um build verificável é mais lento que um nativo, ele precisa do Docker rodando, e o primeiro build baixa uma imagem grande. Você está comprando reprodutibilidade com tempo de build e uma dependência local mais pesada. Para iteração do dia a dia você ainda usa o `cargo build-sbf` nativo e rápido. Você recorre ao build verificável quando você está prestes a fazer deploy de algo em que as pessoas vão confiar.

![Um build normal bifurca um mesmo código-fonte em dois hashes diferentes em duas máquinas; um build Docker fixado afunila o mesmo código-fonte num único hash reproduzível.](assets/v02-flowchart.png)

O que traz os pins à tona, e um número que eu quero desarmar antes que ele te engane. Um build verificável grava o toolchain exato que ele usou, e esse toolchain inclui uma versão do Solana. Essa versão é o ambiente de build destes bytes. Ela não é uma afirmação sobre o que "o Solana atual" é. Esses são dois fatos diferentes e confundir os dois é uma cilada de verdade.

| Pin | Valor para o build do `token_ticket_swap` | Freshness note |
|---|---|---|
| `solana-verify` | 0.5.1 | Mais nova no crates.io em 2026-08-22; `cargo search solana-verify` para ver se isso se moveu, `solana-verify --version` para confirmar o que você tem |
| Docker | 27.x ou mais nova | O build falha rápido se o daemon não estiver rodando |
| Anchor | 2.0.0-rc.1, git `otter-sec/anchor` rev `e4878b6d` (= tag `v2.0.0-rc.1`) | Não é um release cortado que o avm consegue buscar; fixe o commit, não o branch, e reconfira na hora do build |
| Toolchain de build do Solana (dentro da imagem) | 3.1.10 | PIN DE CI-LOCAL / DOCKER. Esta é a âncora determinística para os bytes, NÃO uma afirmação sobre o Solana atual |

Leia essa última linha duas vezes. Solana 3.1.10 é o toolchain assado dentro deste build para que o hash seja reproduzível. O release estável atual do Solana é uma coisa completamente diferente: Agave v4.2.1 em 2026-08-22 (re-verifique, ele se move). O RC do V2 do Anchor mira a linha 3.x do Solana, então um pin de build 3.1.10 está exatamente certo para estes bytes e não diz nada sobre o release de nó mais novo. Se você algum dia se pegar lendo uma tabela de pins e pensando "então o Solana atual é 3.1," pare. O pin é um fato de build, o release é um fato de rede, e eles se descolam de propósito.

Uma nota rápida sobre ferramental, já que você vai encontrar os dois. O Anchor entrega o `anchor build --verifiable`, que embrulha a mesma ideia usando uma imagem `solanafoundation/anchor:v<version>`, e o `anchor verify` do V2 sai direto por shell para o binário `solana-verify` por baixo do capô. A gente usa o `solana-verify` direto aqui porque ele é a ferramenta na qual o ecossistema se padronizou para o passo de verify, e porque manter o build e a prova numa ferramenta só quer dizer uma versão para fixar e um conjunto de flags para aprender.

Um porém para nomear antes de você comparar qualquer hash, porque ele derruba gente. O `.so` no seu disco e o programa como ele vive on-chain não têm o mesmo layout. Um programa upgradeable recebe deploy espalhado por duas contas: uma conta de programa, e uma conta ProgramData separada que de fato segura os bytes do executável. O `solana-verify get-executable-hash` tira a impressão digital do seu `.so` local. O `solana-verify get-program-hash` tira a impressão digital do executável puxado daquela conta ProgramData on-chain. O ferramental normaliza os dois para que fiquem diretamente comparáveis, que é exatamente por que um deploy limpo deixa os dois hashes iguais. Quando eles discordam e você sabe que não editou nada, a causa de sempre é mundana: você tirou o hash de um build novo mas fez deploy de um `.so` obsoleto de uma compilação anterior. Refaça o build, faça deploy de novo, tire o hash de novo, e eles se alinham.

Agora a prova em si. O `verify-from-repo` faz quatro coisas:

```bash
# The frozen skeleton. Fill in your program id, your library name, and your repo URL.
# Two of those are flags; the repo URL is a positional argument at the end.
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  <REPO_URL>

# For a workspace with several programs, name the one you are proving:
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  --library-name token_ticket_swap \
  <REPO_URL>
```

Ele refaz o build do repo dentro da imagem fixada, tira o hash desse binário novo, busca o programa on-chain no cluster que está no `-u`, e tira o hash do que de fato recebeu deploy. Dois hashes, computados de forma independente a partir de duas fontes: o seu código público e o cluster vivo. Se eles forem iguais, os bytes que receberam deploy vieram comprovadamente daquele código-fonte naquele toolchain. Se eles diferem, não vieram. É isso. Não existe intermediário de confiança neste caminho, que é exatamente por que ele funciona na devnet: você é quem roda o rebuild e quem roda a comparação.

![O verify-from-repo tira o hash de um rebuild em Docker do repo e do programa on-chain buscado de forma independente, depois compara os dois hashes localmente para emitir verificado ou descasamento.](assets/v03-diagram.png)

Deixe concreto por um segundo. Digamos que o seu build local dá hash `9f3c...a1` e o `get-program-hash` no seu deploy de devnet retorna o mesmo `9f3c...a1`. O `verify-from-repo` então refaz o build a partir do repo público, computa `9f3c...a1` uma terceira vez, e compara com o valor on-chain. Três computações independentes, um valor, e cada uma delas é uma coisa que um cético consegue reproduzir sem te pedir nada. Agora vire um basis point na taxa, refaça o build, e o hash local vira `2b77...e0` enquanto o repo ainda produz `9f3c...a1`. O descasamento não é um aviso leve. É aritmética: bytes diferentes, sha256 diferente, zero sobreposição.

Aqui é onde eu viro e nomeio a parte honesta, porque uma linha verde é sedutora e ela mente por omissão se você deixar. Um casamento comprova que os bytes na devnet foram construídos a partir deste código-fonte neste toolchain. Ele comprova procedência. Ele não comprova que o código-fonte é seguro. Um programa perfeitamente verificável consegue drenar todo vault dentro dele, porque verificação nunca lê a lógica, ela só tira a impressão digital da saída compilada. Procedência e segurança são ortogonais, e a razão de o seu programa ser confiável é a checklist de auditoria e a passada de fuzz que você rodou no módulo de segurança, não este hash. Verificação torna esses resultados portáteis. Ela deixa um estranho confirmar que o código que você auditou é o código que está rodando. Isso é enorme, e é também estritamente menos que "seguro."

![Um build verificado comprova que os bytes vieram deste código-fonte no toolchain fixado e é re-executável sem precisar confiar em ninguém, mas não comprova nada sobre ausência de bugs, segurança para conceder permissões, ou autoridade de upgrade.](assets/v04-comparison.png)

## O guardião embaixo da cadeia inteira

Até aqui a história é limpa. Build determinístico, comparação local, resultado que dispensa confiança. Agora eu quero derivar a pergunta desconfortável que um leitor cuidadoso já deveria estar formando, porque "não confie, verifique" corta dos dois lados e eu não vou te entregar a ferramenta sem a ressalva.

A pergunta é o que exatamente você está confiando quando você verifica um programa Anchor. Dá a sensação de nada, porque o build é determinístico, e essa leitura é verdadeira para a comparação e falsa para o ambiente. Siga a cadeia. Você confia que a imagem Docker fixada seja um toolchain honesto. Você confia nos crates do Anchor contra os quais você compilou. E se você usar o registry remoto, você confia em quem quer que rode ele. Esses pontos de confiança existirem não é notável; todo toolchain tem eles. O que importa é que aqui eles colapsam numa parte só.

A OtterSec constrói o framework Anchor, publica os pacotes dele, e roda o registry de builds verificados contra o qual o ferramental do próprio Anchor verifica. Um guardião só abrange o framework, os artefatos e o registry. Isso não é um boato e você não precisa aceitar a minha palavra nisso, que é o ponto inteiro: os dois registros gravam isso, e você consegue ler isso de qualquer um dos dois.

Seja preciso sobre qual comando comprova qual metade, porque os dois são artefatos separados. A caminhada de npm abaixo lê o campo `repository` do `@anchor-lang/core`, o cliente TypeScript, e o que ela comprova é *para onde o repositório de código-fonte se moveu*. O seu programa não compila contra aquele pacote; ele compila contra os crates de Rust. Para esses, pergunte direto para o crates.io:

```bash
# The Rust side: who owns the crate your program actually links against.
cargo owner --list anchor-lang
cargo info anchor-lang                      # the `repository:` row names the source repo
# (cargo search won't do here: it prints only name/version/description,
#  never the repository field — `cargo info` is the command that reads it.)

# The npm side: the repository field, version by version, is where the two
# custody transfers are legible.
npm view @anchor-lang/core repository.url
npm view @anchor-lang/core@1.1.1 repository.url   # the version where it changes
```

O campo repository do `@anchor-lang/core` aponta para otter-sec a partir da versão 1.1.1, publicada em 2026-06-25. Percorra o histórico e dá para ver a custódia se mover: o campo vai de coral-xyz para solana-foundation para otter-sec, sem anúncio nenhum em lugar nenhum. Duas transferências de custódia silenciosas, gravadas só num campo de metadados que quase ninguém lê. Quando eu rastreei isso pela primeira vez eu fiz exatamente do jeito que você acabou de fazer, um `npm view` de cada vez, porque eu também não acreditei numa afirmação de segunda mão. É essa a emenda que eu quero que você guarde: verifique a procedência da sua ferramenta de procedência.

![O campo repository do npm para @anchor-lang/core caminha de coral-xyz para solana-foundation para otter-sec, com duas transferências não anunciadas e otter-sec assumindo na v1.1.1 em 2026-06-25.](assets/v05-timeline.png)

Comparado a quê, porém? É essa a pergunta que mantém isto honesto em vez de alarmista. Comparado a verificação nenhuma, onde você aceita a palavra de um estranho de que o deploy dele casa com o repo dele, um guardião só, bem-conceituado, rodando um pipeline reproduzível é um passo grande para cima. Comparado a uma cadeia de suprimentos totalmente diversificada, várias partes independentes construindo o framework, publicando os crates, e rodando registries concorrentes, é um passo curto. As duas comparações são verdadeiras ao mesmo tempo. A resposta certa não é desconfiar da ferramenta. É saber a forma exata do que você está confiando, para que se a custódia trocar de mãos de novo você repare nisso, do mesmo jeito que você acabou de reparar nas duas últimas transferências.

Um guardião só faz a custódia do framework, publica os artefatos, roda o registry contra o qual o Anchor verifica, e assina com GPG a tag v2 sob a chave trixter-osec. Isso é um bocado da cadeia de suprimentos apoiado numa parte competente e bem-conceituada. "Bem-conceituada" está fazendo trabalho de verdade nessa frase, e não é a mesma coisa que "dispensar confiança." Um build verificável remove a sua necessidade de confiar em quem constrói o seu programa específico. Ele não remove a sua necessidade de confiar em quem constrói o framework. Os dois fatos são verdade ao mesmo tempo, e um engenheiro de segurança segura os dois sem piscar.

![A OtterSec fica no centro de três raios, construindo o framework, publicando os crates, e rodando o registry de builds verificados, então um guardião só abrange a cadeia de suprimentos inteira.](assets/v06-diagram.png)

## Só de mainnet, e só de leitura aqui

Mais duas peças do fluxo de trabalho de verdade pertencem à sua cabeça mesmo que você não vá rodar elas nesta lição. Eu estou cercando elas explicitamente. **Tudo nesta seção é só de mainnet e está além do cluster deste curso.** Você vai ler isso, não executar.

A primeira é o submit remoto para o registry. Ao lado do `verify-from-repo` local que você acabou de rodar, o `solana-verify` consegue enfileirar um job de verificação com os workers remotos da OtterSec, que escrevem um registro no registry on-chain. A antiga flag `--remote` do `verify-from-repo` está depreciada e agora só imprime o caminho atual: suba o seu PDA de verify com a autoridade de upgrade do programa, depois `solana-verify remote submit-job --program-id <PROGRAM_ID> --uploader <UPLOADER>`. Esse job leva algo entre um e trinta minutos e escreve um registro de verificação que exploradores e carteiras leem para mostrar o selinho de "verified". **O submit-job remoto é só de mainnet.** Se você apontar ele para um programa de devnet esperando um resultado, você não vai ter um, e a razão não é um deploy faltando ou um daemon do Docker parado, é que o caminho de registry só cobre a mainnet. Na devnet, o `verify-from-repo` local é a prova, ponto final.

Seja preciso sobre o que o job remoto acrescenta, porque ele é uma camada de conveniência, não uma prova mais forte. A prova que dispensa confiança é a local que você já rodou: qualquer pessoa consegue refazer o build e comparar. O que o registry compra é visibilidade. A OtterSec roda o build na infraestrutura deles, escreve o resultado num registro on-chain, e todo explorador e carteira que lê esse registro consegue mostrar um selo de verificado sem cada usuário refazer o build do seu programa por conta própria. Então o caminho remoto troca um pouco mais de confiança no guardião por muito mais alcance, e é a mesma OtterSec que você já encontrou rodando o registry. Na mainnet essa troca em geral vale a pena. Na devnet ela simplesmente não é oferecida, que é a razão inteira de o job voltar vazio ali.

A segunda é o repasse da autoridade de upgrade, e é aqui que verificação encontra governança. A autoridade de upgrade é a conta que tem permissão de substituir os bytes de um programa. Um programa que acabou de receber deploy tem uma, em geral um keypair só, que consegue trocar o executável à vontade. Um build verificado com uma autoridade quente de chave única é um programa que é comprovadamente este código-fonte agora e que pode estar silenciosamente diferente amanhã. O fim de jogo recomendado é mover essa autoridade para um multisig Squads v4, um programa que exige assinaturas de M-de-N membros antes de autorizar uma ação, para que chave nenhuma sozinha consiga empurrar um upgrade por conta própria. **Este fluxo é só de mainnet para este curso; eu estou narrando ele, não rodando ele.** O programa Squads v4 é o `SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf` (re-verifique antes de você algum dia agir em cima dele), e o repasse tem uma ordem específica:

![O repasse só de mainnet do Squads v4 vai de criar o Squad, para escrever um buffer, para transferir a autoridade de upgrade, para uma proposta aprovada até o limiar e executada.](assets/v07-flowchart.png)

A ordenação não é arbitrária, e fazer ela ao contrário é a cilada clássica. Você escreve os bytes novos num buffer e coloca a autoridade daquele buffer no Squad antes de você entregar a autoridade de upgrade do próprio programa. Se você transferisse a autoridade do programa para o Squad primeiro e só então descobrisse que o buffer era de propriedade da chave errada, você ficaria travado precisando de uma proposta de multisig para corrigir um erro que o multisig ainda não alcança. Buffer primeiro, programa segundo, executar por último. Cada passo te deixa em algum lugar do qual você ainda consegue se recuperar, bem até a aprovação final.

A ressalva honesta empilha de dois jeitos, e os dois pertencem à mesa antes de qualquer pessoa tocar na mainnet. Primeiro, o detentor de autoridade recomendado é ele mesmo não-upgradeable: o programa Squads v4 está imutável desde novembro de 2024, o que é uma feature, o multisig no qual você confia não pode ser trocado por baixo de você, e também um fato que você deveria dizer em voz alta. Segundo, o fim de jogo depois do multisig é colocar a autoridade do programa em `None`, deixando o seu próprio programa imutável. Essa é a garantia mais forte que você consegue oferecer aos usuários e ela é irreversível. Não existe desfazer. Uma movimentação de autoridade que você não consegue tomar de volta é uma troca, não uma vitória de graça. Torne ele imutável depois de você ter verificado ele, nunca antes, porque imutabilidade congela o que quer que esteja ali, seguro ou não.

![A escada de autoridade vai de um keypair só para um multisig Squads v4 para imutável, trocando controle por garantia em cada degrau, com o degrau final irreversível.](assets/v08-comparison.png)

## Lab: comprove o swap na devnet

Hora de rodar a coisa inteira. O recuo da ajuda é explícito aqui. Eu rodo o ciclo completo de caminho verde de ponta a ponta, do build até o verify na devnet, com os comandos e os checkpoints escritos por extenso. Você depois roda a completion no seu próprio deploy preenchendo as flags, e o descasamento solo é seu no challenge.

Primeiro, confirme o seu toolchain. Toda ferramenta mostra a instalação dela na primeira vez que você precisa dela.

```bash
# solana-verify (installed above): confirm the version you pinned
solana-verify --version

# Docker must be running; solana-verify builds inside it.
# Install Docker Desktop or the engine from the official docs at docker.com/get-started.
docker info >/dev/null && echo "docker up" || echo "start docker first"

# Anchor V2 RC, if you have not already installed it for this course. `avm install`
# 404s on the RC (no GitHub Release cut for the v2 tag, so its binary is missing), so
# the documented channel is the git build you ran in m01-l2.
# macOS needs LTO off or the release build blows up at link; harmless elsewhere.
CARGO_PROFILE_RELEASE_LTO=off \
cargo install --git https://github.com/otter-sec/anchor.git \
  --rev e4878b6d anchor-cli --locked --force
anchor --version   # expect anchor-cli 2.0.0-rc.1 (freshness 2026-08-22; RC, re-check)
```

Repare no `--rev` onde toda lição anterior escreveu `--tag v2.0.0-rc.1`. Os dois resolvem para o *mesmo* código-fonte — `v2.0.0-rc.1` é uma tag anotada cujo commit é `e4878b6d`, o que você consegue confirmar você mesmo:

```bash
git ls-remote https://github.com/otter-sec/anchor.git 'refs/tags/v2.0.0-rc.1*'
# 2f77733f...  refs/tags/v2.0.0-rc.1       <- the tag object
# e4878b6d...  refs/tags/v2.0.0-rc.1^{}    <- the commit it points at
```

Então por que escrever do jeito mais difícil aqui? Porque uma tag é uma *ref* e um commit é um *fato*. Uma tag pode ser movida ou apagada e recortada num commit diferente; o hash `e4878b6d` nomeia um objeto imutável e nada mais consegue jamais responder por ele. Em todo outro lugar deste curso a tag é precisa o suficiente, e ela lê melhor. Aqui o entregável inteiro é um hash que um estranho reproduz, então o toolchain é nomeado na granularidade mais apertada que existe — a mesma disciplina que o `solana-verify` aplica quando ele fixa a imagem de build que ele traz de fábrica em vez de deixar uma flutuando. Se o `anchor --version` reportar qualquer coisa que não seja o RC depois disso, o commit foi reescrito e você fixa de novo a partir da tag.

Também vale dizer sem rodeios, já que a tabela de pins faz ressalva nisso: a CLI do Anchor que você roda localmente não está dentro do envelope determinístico. O `solana-verify build` compila dentro da imagem Docker fixada, usando o toolchain daquela imagem, então o hash é uma função da imagem e do seu código-fonte, não do seu `anchor` de host. Fixar a sua CLI local mantém *você* consistente entre lições. Fixar a imagem é o que faz a prova funcionar.

Checkpoint: o `solana-verify --version` imprime `solana-verify 0.5.1`, e a linha do docker imprime `docker up`. Se o docker não estiver de pé, arrume isso agora, porque o passo de build vai falhar com um erro de daemon, não um erro de código-fonte, e isso rotula o problema errado.

1. **Faça o build de forma determinística.** A partir da raiz do workspace:

```bash
solana-verify build --library-name token_ticket_swap
```

Isso sobe a imagem Docker fixada e compila o `token_ticket_swap` dentro dela. A primeira rodada baixa a imagem e é lenta. Checkpoint: ele termina com `target/deploy/token_ticket_swap.so` escrito e sem erro.

2. **Tire a impressão digital do binário determinístico.**

```bash
solana-verify get-executable-hash target/deploy/token_ticket_swap.so
```

Checkpoint: você recebe um sha256 de 64 caracteres. Este é o hash que o verificador vai reproduzir de forma independente.

3. **Aponte para a devnet e financie o deploy.** Um deploy de programa não é de graça, então garanta que a CLI está na devnet com SOL para gastar:

```bash
solana config set -u devnet
solana airdrop 2        # devnet faucet; retry if the faucet rate-limits you
solana balance
```

Checkpoint: o `solana balance` mostra pelo menos uns dois SOL. Se o deploy mais tarde disser "insufficient funds," isso é um problema de saldo, não um problema de build, e é aqui que você arruma.

4. **Faça deploy na devnet.** Você já fez deploy do swap na lição passada, então isto é um upgrade no lugar, não um programa novo: passe o keypair de programa do workspace para que os bytes determinísticos aterrissem no mesmo endereço para o qual o seu IDL publicado e o seu cliente gerado já apontam.

```bash
solana program deploy target/deploy/token_ticket_swap.so -u devnet \
  --program-id target/deploy/token_ticket_swap-keypair.json
```

Checkpoint: o comando imprime o seu `Program Id`, o mesmo da lição passada. Esse é o seu `<SWAP_PROGRAM_ID>`. Confirme que o hash on-chain casa com o seu local:

```bash
solana-verify get-program-hash -u devnet <SWAP_PROGRAM_ID>
```

Checkpoint: este hash é igual ao do passo 2. Se eles diferem, você fez deploy de um binário diferente do que você tirou o hash, em geral um `.so` obsoleto, então refaça o build e faça deploy de novo antes de seguir.

5. **Verifique a partir do repo, contra a devnet.** Commite e dê push no seu código-fonte para um repo público primeiro, depois:

```bash
solana-verify verify-from-repo -u devnet \
  --program-id <SWAP_PROGRAM_ID> \
  --library-name token_ticket_swap \
  <REPO_URL>
```

Checkpoint: ele reporta um casamento, uma linha de "verified" para o programa na devnet. Essa linha única é o alvo da avaliação desta lição. Você comprovou agora, localmente e sem precisar confiar em ninguém, que os bytes na devnet foram construídos a partir do seu código-fonte público no toolchain fixado.

![Uma tabela de checkpoints emparelhando cada passo do lab com a cara que o sucesso tem e a correção específica se der errado, terminando com o verify-from-repo reportando um casamento na devnet.](assets/v09-table.png)

## Challenge: faça a prova ficar vermelha, depois diga o que verde quer dizer

O degrau solo tem duas partes, e as duas são o ponto.

Primeiro, quebre. Mude exatamente uma linha do código-fonte do `token_ticket_swap`. A escolha mais limpa é a trava de slippage que você escreveu na lição do swap, porque ela muda comportamento e portanto os bytes sem tocar na interface, nas contas, ou no IDL:

```diff
- require!(out >= min_out, SwapError::SlippageExceeded);
+ require!(out > min_out, SwapError::SlippageExceeded);   // one character, on purpose
```

**Não commite nem dê push nessa edição.** O exercício inteiro depende de o repo e a chain discordarem, e o passo 5 do lab te mandou dar push no seu código-fonte, então o reflexo está bem ali. Deixe a mudança local. Depois refaça o build com `solana-verify build --library-name token_ticket_swap`, faça deploy de novo daquele binário editado no *mesmo* `<SWAP_PROGRAM_ID>` (o upgrade com `--program-id target/deploy/token_ticket_swap-keypair.json` do passo 4, para que você esteja substituindo os bytes que você acabou de comprovar em vez de cunhar um programa novo), e rode o mesmo `verify-from-repo` contra o seu repo público ainda não editado. Se você tiver um casamento em vez de um descasamento, você deu push. Os bytes on-chain agora rejeitam um fill exatamente igual a `min_out`; o repo ainda aceita ele. Aceitação: o `verify-from-repo` reporta um MISMATCH. Depois reverta a linha, refaça o build, faça deploy de novo, e veja ele voltar para um casamento. Você viu agora os dois desfechos com as suas próprias mãos, que é a única forma de a linha verde algum dia significar alguma coisa.

Segundo, escreva uma frase. Com as suas próprias palavras, diga o que um casamento verificado comprova e o que ele não comprova. Uma resposta que passa nomeia as duas metades: ele comprova que os bytes que receberam deploy vieram exatamente deste código-fonte no toolchain fixado, e ele não comprova que o código-fonte é seguro nem que a autoridade de upgrade está travada. Se a sua frase só tem a primeira metade, você aprendeu a ferramenta e perdeu a lição.

Terceiro, faça a única movimentação de autoridade que a devnet *consegue* executar, porque o módulo anterior prometeu que você ia raciocinar sobre quem segura a chave de upgrade e ler um fluxo só de mainnet não é isso. O seu swap de devnet hoje tem uma autoridade de upgrade de keypair único: a sua. Olhe para ela, mova ela, e olhe de novo:

```bash
solana program show <SWAP_PROGRAM_ID> -u devnet     # read the Authority line
solana-keygen new -o /tmp/new-authority.json --no-bip39-passphrase
# Pass the new authority as a KEYPAIR, not a pubkey: the CLI requires the
# incoming authority to co-sign the handoff, the guard that stops you from
# typo-ing your program away to an address nobody holds. (A bare pubkey makes
# the command fail on a missing signature unless you add
# --skip-new-upgrade-authority-signer-check — a flag for hardware-wallet flows,
# and exactly the guard you should not rehearse turning off.)
solana program set-upgrade-authority <SWAP_PROGRAM_ID> -u devnet \
  --new-upgrade-authority /tmp/new-authority.json
solana program show <SWAP_PROGRAM_ID> -u devnet     # read it again
```

Esse é o primeiro degrau da escada, executado em vez de narrado: a chave que consegue substituir silenciosamente os seus bytes verificados agora é uma chave que você escolheu de propósito. O degrau do Squads e o degrau do `None` ficam acima dele e são só de mainnet para este curso, mas a forma é a mesma movimentação toda vez. Guarde o `/tmp/new-authority.json` se você quiser continuar fazendo upgrade deste programa, e repare no que acabou de acontecer se você não quiser: você entregou o seu programa para um keypair em `/tmp`, que é um ensaio pequeno, seguro e instrutivo exatamente do erro irreversível sobre o qual a escada avisa.

Por fim, em uma linha cada, identifique quais dois passos desta lição são só de mainnet e não conseguem ser comprovados na devnet. Se você nomeou o submit-job remoto para o registry e o repasse de autoridade do Squads, você acertou o escopo.

## Antes de seguir em frente

Confira você mesmo contra os quatro jeitos de isso dar errado na prática, porque eles são os quatro que a avaliação procura.

O seu `verify-from-repo` de fato rodou contra a devnet com o seu próprio id de programa, e reportou um casamento de verdade? Ler os fluxos narrados não conta como a prova. A trava é um casamento de hash que você produziu, mais o descasamento que você produziu depois de editar uma linha. Se você só leu, você ainda não passou.

O `verify-from-repo` no seu terminal usou `--library-name`? Num workspace de um programa só ele é opcional e no seu ele não é, porque o `quarter-vault` segura três programas a esta altura — cinco assim que o capstone aterrissar — e a ferramenta não tem como adivinhar a qual deles o seu id de programa pertence.

Você manteve a versão do Solana da tabela de pins no lugar dela? Fato de build, não fato de rede — se essa distinção não for instantânea a esta altura, releia a caminhada da tabela de pins acima antes de seguir em frente.

E você tentou o submit-job remoto na devnet e ficou confuso quando ele não retornou nada? Esse silêncio é exatamente o que o escopo de cluster prevê, porque o caminho de registry remoto é só de mainnet. Na devnet, o `verify-from-repo` local é a prova inteira, e a razão de o job remoto falhar ali é o escopo de cluster dele, não um deploy faltando e não um daemon parado.

Última checagem, e esta é do módulo, não da lição: feche as anotações e diga a sequência de entrega inteira de memória, em ordem. IDL para fora do programa, IDL para dentro da chain, cliente para fora do IDL, kit fixado no major do peer, build determinístico, deploy na devnet, `verify-from-repo`. Se um passo sumir, é esse que você re-roda, não que você relê.

Você entregou o swap e comprovou ele byte a byte, e você olhou de frente para o único guardião no qual a cadeia de verify inteira se apoia. No próximo módulo você arranca o framework de um degrau inteiro. Você reconstrói o R2, o quarter-vault do módulo 3, em pinocchio cru: sem macros, sem struct de accounts gerado, para você conseguir ver exatamente o que o Anchor estava escrevendo para você por baixo de tudo isso. Verificação comprovou que os bytes casavam com o código-fonte. O pinocchio te mostra o que o código-fonte estava escondendo.

Vejo você no módulo de pinocchio.
