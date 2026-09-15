# Contêineres 101: ponha o poller numa caixa

## Resumo

O m06-l1 transformou o CLI em `pulse-pollerd`: um poll loop de tokio embrulhando o crate do motor, com o drop-in de axum `/status` já resolvido respondendo na porta 8080. Ele roda para sempre, mas só na sua máquina. Hoje a gente conserta a parte do "só na sua máquina". Você vai instalar um runtime de contêiner, rodar o seu primeiro contêiner dentro do primeiro quarto desta lição, construir o único modelo mental que faz o Docker parar de ser mágica, e depois escrever o Dockerfile mais honesto possível para o poller: ingênuo, single-stage e gloriosamente acima do peso. Uma nota de apoio, em voz alta: este curso supõe que você nunca tocou em Docker, então este é um primeiro contato totalmente resolvido. O Dockerfile é um problema de completion com exatamente três TODOs, e o challenge solo é uma única mudança de variável de ambiente. As rodinhas saem de novo na próxima lição.

Primeiro, trinta segundos de reconhecimento. Abra um terminal e pergunte se já existe um runtime de contêiner na sua máquina:

```bash
docker version
```

Números de versão para Client e Server, os dois, querem dizer que já existe um runtime instalado e rodando; você vai passar o olho no passo de instalação abaixo. `command not found`, ou um client que responde enquanto a metade do server dá erro, é o resultado esperado para a maioria de vocês, e instalar um runtime é o primeiro trabalho desta lição. De qualquer jeito, agora você sabe em qual caminho está.

## A caixa com que a indústria concordou

Pegue o binário de release que o CI construiu para você no m05-l3 e passe para um amigo. Se ele roda uma distro Linux diferente, existe uma chance decente de ele morrer numa versão de glibc mais velha que a que o seu runner de CI linkou, com uma mensagem de erro que nomeia uma versão de símbolo e não ajuda ninguém. Se ele está no macOS e o CI construiu para Linux, não inicia de jeito nenhum; formato de executável errado, ponto final. E mesmo quando o binário roda, o seu poller lê o `pulse.config.json` do diretório de trabalho dele e espera que a porta 8080 esteja livre, suposições que a sua máquina satisfaz e a dele pode não satisfazer. "Funciona na minha máquina" para de ser piada no momento em que alguém abre isso como bug report. O conserto em que a indústria convergiu não é entregar o binário. É entregar a caixa em forma de máquina em que o binário roda: o sistema de arquivos, as bibliotecas, a config, as expectativas de porta, tudo congelado junto para que a única coisa que a máquina de destino contribui seja um kernel.

Primeiro, o runner de caixas. Instale agora.

### Escolha um runtime, rode um contêiner

O Docker como *formato* é aberto e padrão. O Docker como *aplicativo de desktop* é um produto com licença, e existem três jeitos sãos de ter um runtime na sua máquina:

![Docker Desktop, OrbStack e colima comparados por preço e plataforma, com todos os três gratuitos para aprendizes individuais.](assets/v01-comparison.webp)

Por que existem três opções é uma história com data. Em 2021-08-31, o Docker anunciou que o Docker Desktop deixaria de ser gratuito no trabalho: qualquer empresa com mais de 250 funcionários ou mais de $10M de receita precisaria de uma assinatura paga, com um período de carência até 2022-01-31. Aquele único anúncio é o motivo de OrbStack e colima terem virado nomes conhecidos entre desenvolvedores mac. Os limites continuam valendo hoje e, para você, agora, são irrelevantes: uso pessoal, educação e pequenas empresas seguem gratuitos nos três. Escolha por gosto (versões checadas em 2026-09-02; o Docker entrega todo mês, então os seus dígitos podem estar mais altos):

**Docker Desktop** (macOS, Windows, Linux) é o caminho default e o que eu suponho nesta lição. Baixe o instalador da página Get Started do Docker linkada no fim desta seção, rode, abra o app uma vez para o engine subir. O release atual é o 4.89.0, rodando o Engine 29.7.2 por baixo.

**OrbStack** (macOS): `brew install orbstack`. Gratuito para uso pessoal, $8/user/mo quando uma empresa paga.

**colima** (macOS/Linux): `brew install colima docker`, depois `colima start`. OSS gratuito, sem GUI, e o CLI `docker` conversa com ele exatamente como conversaria com o Desktop.

No Linux a matemática é mais simples: o Docker Engine em si é open source e gratuito em todo lugar, inclusive no trabalho, e instala pelo caminho de pacotes da sua distro conforme os docs do Get Started. A história de licenciamento acima é sobre o app Desktop, que no Linux é uma conveniência, não uma necessidade.

Seja qual for a sua escolha, a prova de vida é a mesma:

```bash
docker run hello-world
```

Na primeira execução, você recebe barras de progresso de pull, depois uma mensagem que começa:

```text
Hello from Docker!
This message shows that your installation appears to be working correctly.
```

Rode uma segunda vez. Sem barras de progresso, saída instantânea. Essa diferença não é um cache aquecido do lado do Docker; é a arquitetura inteira da coisa se mostrando no seu primeiro comando, e vale um diagrama antes de a gente ir mais longe.

Enquanto a evidência está fresca, um comando a mais:

```bash
docker ps -a
```

`docker ps` sozinho lista os contêineres *rodando*, e agora essa lista está vazia, porque o processo do hello-world imprimiu e saiu. O `-a` mostra os que saíram também, e são dois: um por `docker run`, cada um com um nome gerado automaticamente, os dois parados, os dois criados da mesma imagem única. Nada foi reusado entre as execuções, a não ser o template. Limpe eles com `docker rm` e os nomes ou IDs que a listagem mostra, ou comece a formar o hábito que o lab usa: `--rm` na própria execução, para o cadáver nunca ficar por aí.

![O primeiro docker run dá pull na imagem a partir do registro e coloca ela em cache, enquanto a segunda execução reusa a imagem em cache e só cria um contêiner novo.](assets/v02-flowchart.webp)

### Imagem, contêiner, camada, registro

Quatro palavras carregam este módulo inteiro, então vamos fixar elas enquanto a saída do hello-world ainda está na sua tela.

Uma **imagem** é um sistema de arquivos imutável e em camadas mais alguns metadados: qual comando rodar, quais portas o autor pretendia, quais variáveis de ambiente definir. Ela é um template. Ela não faz nada por si só.

Um **contêiner** é um processo rodando a quem foi passado aquele sistema de arquivos como raiz, mais uma camada gravável fina em cima para ele poder rabiscar sem tocar no template. O `docker run` estampa um a partir de uma imagem do mesmo jeito que o `cargo run` estampa um processo a partir de um binário. Três execuções, três contêineres, uma imagem.

Uma **camada** é uma etapa em cache da construção de uma imagem. Toda instrução num Dockerfile produz uma, empilhada em somente leitura em cima da anterior, e a camada gravável do contêiner rodando fica acima da pilha inteira: quando o processo escreve um arquivo, a mudança cai ali, e quando ele modifica um arquivo de uma camada mais baixa, o arquivo é copiado para cima primeiro e mudado na cópia. O template debaixo nunca é tocado, que é o motivo de três contêineres poderem compartilhar uma imagem sem pisar um no outro, e o motivo de tudo que um contêiner escreve morrer com ele, a não ser que você arranje outra coisa deliberadamente. Você vai ver camadas rolando na tela no lab, precificadas individualmente.

Um **registro** é onde imagens moram para que outras máquinas possam dar pull nelas. Você já conhece esta forma duas vezes: o npm é um registro de pacotes e o crates.io é um registro de crates; o Docker Hub é um registro de sistemas de arquivos. Publique uma vez, dê pull em qualquer lugar, resolva por nome e tag em vez de nome e semver. A analogia é próxima o bastante para se apoiar nela e honesta o bastante para ter limite: tags de imagem são rótulos mutáveis, não versões imutáveis, então `rust:1.98` pode apontar para uma imagem reconstruída amanhã de um jeito que `serde@1.0.229` nunca vai. O Docker Hub é o registro default, é de onde vêm o `hello-world` e a imagem base `rust`, e é um serviço com rate limits com que a gente vai lidar honestamente em um minuto. Uma **tag** é o rótulo legível por humanos depois dos dois-pontos, o `:naive` em `pulse-pollerd:naive`. Uma **imagem base** é simplesmente a imagem de onde a sua imagem parte, o argumento da linha `FROM`, contribuindo com as camadas dela como a sua fundação.

![Um registro serve uma imagem feita de camadas somente leitura empilhadas, e cada contêiner rodando é um processo separado com a própria camada gravável fina sobre essa mesma imagem.](assets/v03-diagram.webp)

Agora o modelo que faz tudo isso desabar em algo sobre o qual você consegue raciocinar. A figura tentadora, a que a palavra "contêiner" planta na sua cabeça, é uma pequena máquina virtual: um computadorzinho que você inicia, loga e fuça. Complete essa figura e você espera entrar por ssh, instalar coisas, reiniciar.

Não é isso. Um contêiner é um processo vestindo um sistema de arquivos. Um processo, iniciado pelo seu kernel como qualquer outro, só que o kernel mostra para ele um diretório raiz diferente e uma visão cercada do mundo. Aqui está o teste que resolve isso: o que acontece quando o processo sai? O contêiner acabou. Nada ficou de pé, porque não havia máquina, só o processo. O `hello-world` imprimiu a mensagem dele, saiu, e o contêiner dele terminou no mesmo suspiro. É também por isso que o segundo `docker run` criou um contêiner *novo* em vez de se reconectar ao antigo: contêineres são tão descartáveis quanto processos, porque é isso que eles são.

Se o arranjo inteiro precisa de uma figura de fora do software: o contêiner intermodal de carga. Antes da caixa de aço padronizada, carregar um navio de carga significava estivadores empilhando barris e engradados à mão, cada navio um caso especial. A caixa padronizou a *interface*, e de repente o guindaste, o navio, o caminhão e o porto pararam de se importar com o que havia dentro. O Docker é essa caixa para software: o registro é o porto, a imagem é o contêiner selado, e qualquer host com um runtime é um navio que consegue carregar ele. Onde a analogia quebra, e ela quebra: uma caixa de aço é carga inerte, enquanto a nossa caixa vem com a instrução de iniciar exatamente um processo. Leve a analogia até a logística e solte ela antes do comportamento.

Uma nota de pé de página de honestidade antes que alguém num Mac me pegue: no macOS e no Windows, contêineres Linux não conseguem rodar direto no kernel do host, então Desktop, OrbStack e colima gerenciam cada um, em silêncio, uma VM Linux e rodam os seus contêineres dentro dela. O modelo continua valendo; os seus contêineres são processos *naquele* kernel. Você só pagou pela ilusão com alguma RAM, que é parte da conta que a gente vai somar logo. E note o que o modelo muda nos seus reflexos de debug: o sistema de arquivos, o ambiente e a rede da caixa são o mundo do autor da imagem, não o da sua máquina, então "funciona no contêiner" e "funciona no meu host" agora são afirmações separadas com evidências separadas. Essa separação é a vitória de portabilidade vestindo a roupa de trabalho.

### Faça login antes do primeiro pull de verdade

O lab abaixo dá pull na imagem base `rust` a partir do Docker Hub, e eu quero que você faça login primeiro, porque a falha que você evita é genuinamente ruim de diagnosticar. Cenário: você está num coworking, num campus, ou dentro de um runner de CI. O seu build morre dando pull numa imagem base com um erro 429. Em casa, o build idêntico funciona. Nada no erro menciona o porquê.

O que está acontecendo: pulls não autenticados no Docker Hub são limitados a 100 pulls por 6 horas *por endereço IPv4* (ou por sub-rede IPv6 /64) e, atrás de um NAT compartilhado, todo mundo no prédio está gastando a mesma cota. Você não fez nada; os cinquenta laptops ao seu redor fizeram. Uma conta gratuita do Docker Hub te move para a sua própria cota de 200 pulls por 6 horas, presa à sua conta em vez do IP do prédio. Os dois números são do próprio Docker, lidos em docs.docker.com/docker-hub/usage/ em 2026-09-06, e são os que sobreviveram à saga de políticas de 2025: só os planos pagos Pro, Team e Business ganham uma taxa de pull ilimitada. Confira a tabela você mesmo antes de citar um número para um colega; o hábito é isso, não o dígito. A variante de CI dessa falha é a que morde os times: runners hospedados compartilham os endereços de egress do provedor de nuvem deles com milhares de estranhos, então um pull anônimo que funcionou o sprint inteiro começa a ficar flaky na semana em que algo popular é entregue. Mesma causa, mesmo conserto, e agora você consegue diagnosticar isso só pelo padrão de sintoma: dependente de localização, reproduzível, e 429 em vez de not-found.

![Muitos laptops atrás de um IP compartilhado esgotam uma cota comum de pull de cem por seis horas, enquanto um usuário logado ganha os próprios duzentos.](assets/v04-diagram.webp)

Então: crie a conta gratuita em hub.docker.com, depois:

```bash
docker login
```

Digite o usuário e o token ou a senha que ele pedir; `Login Succeeded` é o seu Checkpoint. (O plano Personal gratuito também carrega um repositório de imagem privado, que é mais hospedagem do que este curso vai pedir dele.) O conserto é esse inteiro. Quando até 200 por 6 horas não é o bastante, ou você quer as suas próprias imagens hospedadas ao lado do seu código, o GHCR, o registro do GitHub, é a saída de emergência, e é exatamente para lá que a gente dá push na imagem do poller no m06-l4. Não hoje.

Vou confessar para onde foram as minhas próprias horas neste território, porque não foi o rate limit. Foi rodar um contêiner, dar curl em `localhost:8080`, não receber nada, e concluir que o meu app estava quebrado. O app estava bem. Eu tinha esquecido o mapeamento de portas, então a minha requisição nunca entrou na caixa. Você vai ligar esse mapeamento deliberadamente no lab, e quando chegarmos lá você vai ver por que o dentro e o fora de um contêiner são redes diferentes.

### O que a caixa custa

O trade-off, dito antes de você construir qualquer coisa, porque este aqui é medido em gigabytes. Uma imagem ingênua entrega o seu ambiente de build inteiro: toolchain, caches de dependência, fonte. Só a imagem base `rust:1.98` tem uns 600 MB comprimidos no Docker Hub (eu vi ela descer pelo fio enquanto checava esta lição em 2026-09-02), e ela desempacota para consideravelmente mais; adicione o seu diretório `target/` e a imagem para um binário de poucos megabytes cai nos gigabytes. Mapeamentos de portas e caches de camada são lugares novos para bugs morarem que não existiam quando você rodava `cargo run`. E no macOS existe aquela VM Linux gerenciada de bobeira na sua RAM. Você aceita tudo isso porque "roda igual em todo lugar" é a fundação sobre a qual CI, registros e todo módulo posterior deste curso se apoiam. O peso, pelo menos, é consertável, e consertar ele é literalmente a próxima lição.

**Vá mais fundo (os 20%).** esta lição te dá o modelo mental, o login e a sua primeira imagem; o tour guiado da plataforma mais ampla, volumes, rede de contêineres, o CLI completo, mora no caminho oficial Get Started do Docker: [https://docs.docker.com/get-started/](https://docs.docker.com/get-started/). Deixe como bookmark, caminhe pelas duas primeiras seções dele esta semana. Uma meta-lição vem de brinde: o Docker hospedava este material numa URL de "workshop", e o próprio verificador de links deste curso pegou aquela URL redirecionando em silêncio para outro lugar enquanto esta lição passava pela checagem de fatos. Verifique links antes de confiar nos bookmarks do ano passado, inclusive nos meus. O lab abaixo não precisa de nada do material dos bookmarks.

## Lab: pulse-pollerd:naive

Terminal aberto na raiz do workspace `pulse-rs`, a que contém o `Cargo.toml` com `[workspace]`, `crates/pulse-engine`, `crates/pulse-cli`, `crates/pulse-pollerd` e o `pulse.config.json`. A gente vai encaixotar o poller com o Dockerfile mais óbvio que pode possivelmente funcionar, de propósito, e ler os destroços com honestidade.

1. **Cerque o contexto de build primeiro.** Quando o Docker constrói uma imagem, ele entrega o "contexto de build", por padrão o seu diretório atual inteiro, para o engine. O seu workspace contém um diretório `target/` com gigabytes de cache de build, e o `COPY . .` arrastaria ele para dentro da imagem com todo o prazer. Crie o `.dockerignore` na raiz do workspace:

```text
target/
.git/
```

Mesma ideia do `.gitignore`, público diferente: isso reduz o que o build consegue sequer ver. (Nota honesta sobre a segunda linha: o `.git/` da estação na verdade mora um nível acima, na raiz do repo, desde que o m04-l3 moveu o `pulse-rs/` para dentro do repo da estação, então ele nunca entra neste contexto de build. A linha de ignore não custa nada e salva quem constrói a partir de um clone avulso, que é o motivo de ela ficar.) Faça isso antes do primeiro build e você nunca vai saber o quanto a alternativa era lenta.

2. **Complete o Dockerfile.** Crie um arquivo chamado `Dockerfile` na raiz do workspace. Aqui está o esqueleto, com os três TODOs da lição:

```dockerfile
# TODO 1: pick the base image. We need a full Rust toolchain to compile,
# and we pin the stable minor: rust:1.98
FROM ???

WORKDIR /app

# TODO 2: what does the build need copied in? The whole workspace: every
# crate, the root Cargo.toml, and pulse.config.json the poller reads.
COPY ??? ???

RUN cargo build --release --bin pulse-pollerd

EXPOSE 8080

# TODO 3: the command the container runs when it starts. One process,
# remember: this IS the container.
CMD ???
```

Trabalhe os três TODOs contra o que você sabe, depois confira contra a versão preenchida:

```dockerfile
FROM rust:1.98

WORKDIR /app

COPY . .

RUN cargo build --release --bin pulse-pollerd

EXPOSE 8080

CMD ["/app/target/release/pulse-pollerd"]
```

![Cada instrução do Dockerfile anotada com o propósito dela, mostrando que a imagem base e a etapa de cargo build contribuem com as camadas pesadas enquanto EXPOSE e CMD são metadados.](assets/v05-annotated-code.webp)

Uma checagem de pré-voo antes das glosas, e ela vai poupar alguns de vocês de um crash desconcertante. Lá no m05-l1 você criou o `pulse.config.json` na raiz da estação e apontou o lado Rust para ele, um arquivo, não duas cópias, e a jogada sugerida era um symlink. O `COPY` copia o link, não os bytes: o alvo do link mora fora deste contexto de build, então dentro da imagem ele fica pendurado no vazio, e o poller morre na inicialização sem conseguir abrir um arquivo que o `ls` jura que está ali mesmo. Rode `ls -l pulse.config.json` na raiz do `pulse-rs`; se você vê uma flecha, substitua o link por uma cópia de verdade (`cp` do alvo em cima dele) antes de construir, e COMMITE a substituição, não só o conserto local: o job de CI do m06-l4 constrói esta imagem exata a partir de um checkout novo num runner, onde um symlink rastreado apontando para fora do contexto de build fica pendurado igualzinho, duas lições depois de qualquer um lembrar por quê. A disciplina de um-arquivo-só, com honestidade, termina na fronteira da imagem, porque um sistema de arquivos selado não consegue seguir um ponteiro de volta até o seu laptop; manter as duas cópias de acordo agora é um dever de manutenção real (pequeno), e é o preço da caixa, não o bug da caixa.

Quatro glosas que os comentários do esqueleto não conseguiram acomodar. O `rust:1.98` fixa o minor do toolchain do mesmo jeito que o `rust-toolchain` fixa ele localmente; a tag existe no Docker Hub e acompanha o stable atual, então suba ela quando a sua máquina de trabalho subir. O `WORKDIR /app` é estrutural para nós, em silêncio: o poller lê o `pulse.config.json` relativo ao diretório de trabalho dele, e porque o `COPY . .` põe o arquivo em `/app` e o processo do `CMD` começa ali, o mesmo caminho relativo que funcionava no seu host resolve dentro da caixa; mude o WORKDIR sem mover a config e você construiu uma imagem que inicia e imediatamente não consegue achar os próprios alvos. O `EXPOSE 8080` é documentação pura, e isso importa: ele *não* abre porta nenhuma. Publicar uma porta é uma decisão de runtime tomada com `-p`, que é o trabalho inteiro do passo 4. E o `CMD` usa a forma de array JSON para o seu binário rodar como o único processo do contêiner diretamente, sem shell no meio.

3. **Construa, e leia as camadas.** A partir da raiz do workspace:

```bash
docker build -t pulse-pollerd:naive .
```

O `-t` marca a tag do resultado; o `.` é o contexto de build que você acabou de cercar. As primeiríssimas linhas de saída são o seu comprovante do `.dockerignore`: uma linha `transferring context` com um tamanho. Um workspace limpo transfere em megabytes; se você vê centenas de megabytes ou pior, `target/` ou `.git/` vazaram para o contexto, e consertar o arquivo de ignore agora salva todo build daqui até o capstone. Depois espere minutos: o pull da imagem base, seguido por um `cargo build --release` frio do workspace inteiro dentro da caixa. Observe a estrutura da saída enquanto ela roda: uma etapa numerada por instrução que toca o sistema de arquivos, `[1/4] FROM`, `[2/4] WORKDIR`, e assim por diante, cada uma virando uma camada, com a etapa do cargo fazendo essencialmente toda a espera. (Seis instruções, um denominador de 4: `EXPOSE` e `CMD` são metadados, então o BuildKit não dá etapa numerada para eles, a mesma história de 0B que o `docker history` conta no passo 5.) A cauda deve terminar com algo como:

```text
 => exporting to image
 => => naming to docker.io/library/pulse-pollerd:naive
```

Salve essa cauda; o Checkpoint quer ela. Depois reconstrua imediatamente sem mudar nada: `docker build -t pulse-pollerd:naive .` de novo. Segundos, não minutos, com `CACHED` impresso ao lado das etapas. Camadas são o cache, e o log de build é onde você vê ele trabalhar.

4. **Rode com a porta aberta.** O poller escuta na 8080 *dentro* do contêiner, e o dentro é uma rede diferente do seu host. `curl localhost:8080` na sua máquina bate na porta 8080 do seu host, onde nada está escutando. A flag `-p` constrói a ponte:

```bash
docker run --rm -p 8080:8080 pulse-pollerd:naive
```

Leia `-p 8080:8080` como `host:container`: requisições para a porta 8080 do host são encaminhadas para a 8080 do contêiner. Os dois números não precisam bater, que é exatamente a costura em que o Challenge puxa. (O `--rm` deleta o contêiner quando ele sai, um hábito de educação que vale formar agora.) Enquanto ele roda, um `docker ps` pelado em outro terminal mostra o contêiner vivo com o mapeamento de portas dele impresso na coluna PORTS, que é onde eu olho primeiro sempre que um serviço conteinerizado "quebrado" cruza a minha mesa. Depois, daquele segundo terminal:

```bash
curl -s localhost:8080/status
```

Você deve receber o mesmo JSON de `/status` do m06-l1: estado por alvo, latência, timestamp do último poll, agora servido de dentro da caixa. Se o curl fica preso ou reseta enquanto os logs do contêiner parecem saudáveis, confira dois suspeitos em ordem. Primeiro: você passou o `-p` de verdade? (A confissão acima é sua para pular agora.) Segundo, o endereço de bind: um servidor com bind em `127.0.0.1` dentro do contêiner só é alcançável de dentro do contêiner, que para um processo é um lugar muito silencioso. O drop-in do m06-l1 faz bind em `0.0.0.0:8080`; se o seu diz `127.0.0.1`, mude para `0.0.0.0` e reconstrua. No seu host essa distinção quase não importava. Na caixa ela é tudo.

![Uma requisição do host alcança o poller só através do mapeamento de portas publicado e de um bind zero-ponto-zero-ponto-zero-ponto-zero, com becos sem saída quando qualquer um dos dois falta.](assets/v06-flowchart.webp)

5. **Pese, e escreva o número.** Pare o contêiner (Ctrl-C no terminal dele), depois:

```bash
docker images pulse-pollerd
```

Olhe a coluna SIZE. O binário do seu poller tem poucos megabytes. A imagem em que você acabou de entregar ele vai ficar nos gigabytes, três ordens de magnitude de embalagem em volta da coisa que você de fato fez. Registre o número exato em algum lugar que você vai achar de novo na próxima lição; a gente remede esta mesma imagem depois do rebuild multi-stage, e eu quero que o seu antes seja o seu.

Não aceite o total por fé também; pergunte para a própria imagem onde o peso mora:

```bash
docker history pulse-pollerd:naive
```

Uma linha por camada, a mais nova primeiro, cada uma com o próprio tamanho. Leia isso contra o Dockerfile que você escreveu: a linha do `RUN cargo build` é a mais pesada que VOCÊ causou, uns meio gigabyte de saída congelada do `target/`, enquanto as linhas do toolchain base abaixo dela são os gigantes de verdade, cada uma na mesma classe de meio gigabyte ou maior, porque um userland Debian mais rustc mais cargo pesa mais que o build de qualquer projeto sozinho. A linha do `COPY . .` carrega a sua árvore de fontes, e as linhas de classe `EXPOSE`, `CMD` e `ENV` todas reportam 0B porque metadados não pesam nada. Esta é a mesma contabilidade por camada que o log de build insinuou, agora com uma balança do lado.

De onde veio o peso? Nada de misterioso: a imagem final é toda camada que você viu rolar na tela. O toolchain Rust completo do `FROM`. A sua árvore de fontes inteira do `COPY . .`. E a mais pesada, a camada do `RUN cargo build`, que congelou o diretório `target/` inteiro, artefatos de dependência e tudo, dentro do sistema de arquivos entregue. Nada disso é necessário para *rodar* o binário; tudo isso era necessário para *construir* o binário, e um Dockerfile single-stage não consegue ver a diferença. Essa frase é a próxima lição.

![A imagem ingênua empilha um toolchain completo, a árvore de fontes e o cache de build inteiro em volta do único binário pequeno que o contêiner de fato precisa rodar.](assets/v07-diagram.webp)

É esse o lab: o poller responde de dentro de uma caixa que qualquer host Docker na Terra consegue rodar, e a caixa está comicamente acima do peso. As duas metades dessa frase são o ponto.

## Challenge

Solo, uma costura, nenhum conceito novo: deixe a porta do poller configurável por variável de ambiente. Por que variáveis de ambiente, quando o poller já tem um arquivo de config perfeitamente bom? Porque uma imagem é imutável e uma imagem deve servir muitos deploys: mesma caixa, porta diferente no seu laptop, no CI e em qualquer host que venha a rodar ela. Variáveis de ambiente são o botão que o operador de um contêiner consegue girar sem reconstruir, que é o motivo de elas serem o idioma de config de todo serviço conteinerizado que você vai ler na vida. Três jogadas. Na inicialização do `pulse-pollerd`, leia `POLLER_PORT` e caia de volta para 8080 quando ele estiver ausente ou não parseável; `std::env::var("POLLER_PORT")` te dá um começo em forma de `Result`, e a cauda da cadeia é uma que você já escreveu: a demo de overflow do m05-l3 leu o argumento dela com `.nth(1).and_then(|s| s.parse().ok()).unwrap_or(250)`. Roube isso e adapte, porque duas coisas diferem e as duas vêm de onde o valor vem. `args().nth(1)` te passa um `Option` enquanto o `env::var` te passa um `Result`, então o seu precisa de um `.ok()` na frente para entrar nos mesmos trilhos; e o fallback é 8080, não 250. Aterrissar em `.ok().and_then(|s| s.parse().ok()).unwrap_or(8080)` é o ponto do exercício, não a linha de partida. No Dockerfile, adicione `ENV POLLER_PORT=8080` acima do `CMD` para documentar o default na própria imagem. Reconstrua, depois rode ela movida:

```bash
docker run --rm -e POLLER_PORT=9090 -p 9090:9090 pulse-pollerd:naive
```

Aceitação: `curl -s localhost:9090/status` responde, e rodar sem `-e` ainda responde na 8080. Se a 9090 fica presa, releia os dois suspeitos do passo 4; o segundo não consegue te machucar duas vezes, mas o primeiro absolutamente consegue.

## Checkpoint

Trave no fazer, quatro colagens: a saída do `Hello from Docker!` da sua checagem de instalação; a cauda do log de build ingênuo; uma resposta de `curl -s localhost:8080/status` do lado do host servida a partir do contêiner; e a linha SIZE do `docker images` que você registrou para a próxima lição. O `docker login` deveria ter dito `Login Succeeded` no caminho, mesmo que nada tenha te forçado.

O que você consegue fazer agora, concretamente: instalar e verificar um runtime de contêiner e explicar o que você pagou por ele no seu OS; ler `docker ps -a`, um log de build e `docker history` como evidência em vez de barulho; explicar imagem, contêiner, camada, registro e tag com um diagrama; e levar qualquer binário seu do `cargo run` até responder por uma porta publicada de dentro de uma caixa.

A recuperação de 30 segundos antes de você fechar o terminal: qual dos dois o `docker run` cria, uma imagem ou um contêiner? (Um contêiner; imagens só são criadas por builds.) E os quatro substantivos em um suspiro: registro guarda imagens, imagem é o template imutável em camadas, camada é uma etapa de build em cache, contêiner é um processo rodando vestindo aquele sistema de arquivos.

Se a instalação do runtime brigou com você, e em algumas máquinas corporativas ela genuinamente briga, me diga qual OS e qual runtime no feedback do curso; a tabela de escolha de runtime no topo desta lição é a seção que eu mais espero que precise de ajuste por turma, e relatos de falha reais são como ela ganha o pão.

Você entregou uma caixa funcionando que pesa gigabytes para um binário medido em megabytes, e você tem o número exato escrito. Próxima lição: dois Dockerfiles, uma ideia. Builds multi-stage cortam essa imagem em uma ordem de magnitude, com o seu próprio antes-e-depois em Rust fazendo o argumento, e a mesma técnica levada direto para uma imagem Node slim para a frota. Mantenha esse número à mão.
