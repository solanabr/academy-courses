# Conclusão: o que você construiu, e você deveria migrar para o V2 hoje?

Na lição passada você pegou um vault 0.31/1.0 de verdade e levou ele até um build V2 que compila e passa em LiteSVM, seguindo os avisos do compilador como uma lista de pendências até o port ficar verde. Essa era a última coisa que este curso tinha para te ensinar. Você agora construiu o Anchor V2 do zero e migrou uma base de código antiga para dentro dele. O salão do fliperama está pronto: um cabinet-counter, um quarter-vault, um prize-escrow, um swap de token-para-ticket, e um floor-registry que faz CPI para todos eles juntos. Testado, fuzzado, perfilado, verificado.

Falta uma coisa, e ela não é um lab. Escolha um projeto de que você se importa de verdade, agora. Não um de brinquedo. Algo com o seu nome nele, ou o do seu time. Escreva uma linha sobre ele num post-it: ele está segurando valor real hoje, ele é um build novo sem nada em jogo, ou ele é uma base de código grande já rodando na linha antiga? Deixe esse post-it do seu lado. No fim desta lição você vai rotear ele por uma árvore de decisão, em voz alta, e você vai conseguir defender a resposta a partir dos fatos de verdade. É esse o trabalho inteiro de uma conclusão que respeita o seu tempo. Não uma volta de vitória. Uma chamada honesta que agora você consegue fazer sozinho.

## O que esta lição faz, e o que ela repassa

Duas jogadas, e uma recusa deliberada.

![Um mapa de cubo e raios do plano de fechamento: encaixar os cinco programas num modelo só, rodar a árvore de decisão de adoção, repassar as camadas adjacentes para os cursos irmãos.](assets/v01-diagram.webp)

Primeiro, ela encaixa tudo o que você construiu num modelo conectado só, para os cinco programas pararem de ser cinco exercícios e virarem uma figura só de como o V2 pensa. Segundo, ela percorre a árvore de decisão de quem migra contra as tensões reais do projeto hoje, porque "o V2 é melhor" e "você deveria colocar o seu dinheiro de mainnet no V2 esta tarde" são duas perguntas diferentes e só uma delas é fácil. A recusa: este curso é dono da camada de framework e de nada mais, então o fechamento te aponta para fora, para os cursos irmãos que são donos das camadas sobre as quais o V2 se apoia.

Aqui está o recuo da ajuda, dito sem enfeite. Cada lição anterior te percorreu o constraint no momento em que você bateu nele. Esta não. Eu não vou fazer a chamada de adoção por você. Eu vou te entregar a árvore e as entradas honestas, e depois dar um passo atrás e deixar ela ser sua. É isso que terminar um curso deveria dar a sensação de ser: o apoio desce e a coisa continua de pé.

## O mapa inteiro, e depois o ambiente que você tem que ler

Você não aprendeu cinco truques sem relação. Você aprendeu quatro ideias, quatro vezes cada uma, com nomes diferentes. Enfileire elas.

`Account<T>` é Pod zero-copy por padrão. É essa a tese da issue #4390 do Anchor, a que reenquadrou a reescrita inteira: pare de desserializar a conta inteira para o heap em cada instrução, e comece a ler campos no lugar a partir de um layout fixo. O seu cabinet-counter foi a prova menor possível disso. É também de onde vêm a economia de bytes e a economia de compute, porque o trabalho que você pagava em cada chamada simplesmente não está mais ali.

CPIs são `CpiHandle`s com borrow rastreado. A cilada do reload — no V1, qualquer CPI que mutasse uma conta que você ainda estava segurando tipada forçava um `.reload()` ou uma leitura obsoleta silenciosa — foi embora por construção. O borrow checker agora sabe que uma conta mudou embaixo de uma CPI, então o sistema de tipos carrega a freshness que o código antigo carregava na sua memória. O seu floor-registry, chamando o vault e o swap, é onde isso parou de ser uma regra que você tinha que lembrar e virou uma coisa que o compilador lembra para você.

Os bumps de PDA chegam até você por uma struct tipada que a macro constrói em tempo de expansão. Você guardava bumps canônicos no V1 para economizar as ~1500 unidades de compute que uma re-derivação custa, e você continua guardando eles no V2, porque uma busca sobre a chave de quem chama só consegue rodar na validação. Diga o delta com precisão, porque é este que as pessoas atribuem errado: o campo tipado *não* é ele. O `ctx.bumps.vault` substituiu o antigo `ctx.bumps.get("vault").unwrap()` com chave de string no Anchor **0.29**, dois majors antes do V2, e a linha 1.x contra a qual você mediu lê ele exatamente como o V2 lê. A notícia de bump de verdade do V2 é mais estreita — quando cada seed é um literal de byte em tempo de compilação, o bump canônico é pré-computado em tempo de macro como um const, que é um caso que o seu quarter-vault não atinge. Então o que você carregou pelo port aqui foi continuidade, e a disciplina de CU continuou sua nos dois lados. O seu quarter-vault assinando por si mesmo é essa ideia em carne e osso.

E três classes nomeadas de vulnerabilidade agora são erros de compilação em vez de exploits de runtime. O módulo 7 colocou números em exatamente quais: o type cosplay, o alias de duplicate-mutable, e a leitura obsoleta depois de uma CPI. Você explorou todas as três em formas de v1 e depois viu os defaults do V2 se recusarem a construir elas. Mantenha o escopo honesto, porque é o escopo que você mesmo auditou: três das onze classes na taxonomia da Foundation, não todas elas, e a substituição de conta através de uma CPI continua sendo sua para validar na mão. O seu prize-escrow e o seu swap são onde você sentiu esse guarda-corpo empurrar de volta, e onde você achou a borda dele.

![Um mapa conceitual com o Anchor V2 no centro e quatro raios: contas Pod-por-padrão, CPIs com borrow rastreado, um const de bump para seeds todas literais em cima da struct tipada que o 0.29 já te deu, e segurança em tempo de compilação, cada um ligado a um programa que você construiu.](assets/v02-diagram.webp)

Note o que o mapa está de fato te dizendo. Estas não são quatro features aparafusadas, e sim uma decisão só, aplicada de forma consistente: mover o trabalho que o desenvolvedor fazia em runtime, e errava, para cima, para dentro do sistema de tipos e da geração de código. É esse o fio condutor da reescrita inteira.

### O framework que narra o próprio porquê

Dê um zoom para fora mais um clique e você consegue ver a trajetória que produziu isso. O Anchor antigo de que você talvez lembre era pesado em macros e faminto por compute, comprando ergonomia de desenvolvedor com custo de runtime. A linha 1.0 estabilizou essa barganha e colocou isso sob tutela de verdade. A linha 2.0, o trabalho do anchor-next, voltou e pagou o custo que ela tinha assumido, empurrando a ergonomia para dentro do compilador e os bytes para cima de um runtime mais magro.

![Uma linha do tempo de três paradas: o Anchor antigo pesado em macros, a linha 1.1.2 estabilizada e ainda mantida, e a linha 2.0 de candidata a release mais magra.](assets/v03-timeline.webp)

Um framework que revisita os próprios trade-offs em voz alta é raro, e é exatamente o tipo de coisa que você quer embaixo dos seus programas. Mas essa mesma honestidade é o que deixa a pergunta de adoção difícil, porque a honestidade se estende para as partes que ainda não estão prontas.

### A tensão, dita sem vacilar

Aqui é onde muitos textos ficam animadinhos e param de ser úteis. Deixe eu não fazer isso.

O mesmo release chama a si mesmo de duas coisas contraditórias. A versão é `2.0.0-rc.1`. "rc" quer dizer release candidate, que lê como "quase pronto, só espantando os bugs". Mas o projeto rotula esse mesmo release exato de "alpha" em outro lugar, que lê como "cedo, espere movimento". Os dois rótulos, um artefato. Isso é os mantenedores te dizendo, em duas palavras, que a coisa está genuinamente no meio, e não um typo que você tem permissão de arredondar.

A documentação diz, com as palavras dela, que o V2 não é auditado. Não "audit pending," não "audit in progress that we will link." Não auditado. Para um framework cujo pitch inteiro inclui matar classes de vulnerabilidade, essa é a frase mais importante da página.

Não existe data comprometida para a candidata a release virar estável. Não "Q4", não "no próximo trimestre". Enquanto isto é escrito não existe data nenhuma publicada em nenhum lugar. Ausência de data não é uma data curta. É a ausência de uma promessa, e você deveria ler isso como exatamente isso e nada mais suave.

E a linha antiga não está parada. O Anchor 1.1.2 é o estável atual, e a linha dele continua mantida e entregando. Esta é a realidade das duas linhas paralelas: um v1 em movimento, auditado pelo tempo, provado em produção, ao lado de um v2 mais rápido, não auditado, sem data. Você não está escolhendo entre uma opção viva e uma morta; você está escolhendo entre duas opções vivas com perfis de risco opostos.

Essa distinção importa mais do que parece, porque ela muda o que "esperar" te custa. Esperar no 1.1.2 não é a mesma coisa que estagnar nele. A linha continua recebendo correções, e continua acompanhando o runtime conforme a própria rede muda embaixo do seu programa. Você está estacionado numa estrada que continua sendo pavimentada, não largado numa abandonada, e é exatamente por isso que o branch paciente da árvore é uma opção de verdade e não um eufemismo para ficar para trás.

![Um lado a lado do Anchor 1.1.2, estável e endurecido em produção, contra o 2.0.0-rc.1, rotulado tanto de rc quanto de alpha e explicitamente não auditado mas muito mais magro em bytecode e CU.](assets/v04-comparison.webp)

### O número que ficou mais honesto

Agora, sobre aquela economia, porque ela é real e você deveria carregar ela do jeito certo.

Os próprios benchmarks do V2 reportam mais ou menos 94% menos bytecode implantado e cerca de uma redução média de 8.8x em unidades de compute. Esses são números grandes. Eles também não são os números que o projeto publicou primeiro. Um pull request, o #4914, aterrissou em 2026-08-13 e revisou as cifras da manchete para baixo: 95% virou 94% no bytecode, e 9.9x virou 8.8x no compute. O projeto deixou a própria vanglória dele menor.

Sente com isso por um segundo, porque é a coisa mais tranquilizadora desta lição inteira. Um projeto revisando os números de marketing dele para baixo, de propósito, é um projeto em que você pode confiar mais, não menos. É "não confie, verifique" aplicado pelos mantenedores a eles mesmos. A pegadinha é que isso também te diz que os números ainda podem se mover, porque a página de benchmark diz exatamente isso: os valores mudam conforme a geração de código e o runtime pinocchio subjacente mudam. Então o jeito certo de carregar eles é como evidência com ressalvas, direcional, re-verificável. O V2 é dramaticamente mais magro. Essa é a afirmação. Um multiplicador congelado não é.

![Duas figuras emparelhadas mostrando os benchmarks da manchete revisados para baixo, o bytecode de 95% para 94% e o compute de 9.9x para 8.8x, conforme o PR #4914.](assets/v05-chart.webp)

Tem mais um fato que vale pesar antes de você rotear qualquer coisa, e ele fica na superfície de confiança e não no código. Um guardião só agora faz a custódia da cadeia de suprimentos inteira. A OtterSec detém o framework, publica os crates dele, roda o registry de builds verificados contra o qual o Anchor checa os releases, e assinou com GPG a tag v2. Um guardião único, competente e focado em segurança através dos crates, do registry e das assinaturas é um ponto de verdade a favor do V2, e você consegue checar a última parte você mesmo em vez de aceitar a minha palavra. A gente vai fazer isso no lab.

Um registry de builds verificados vale entender, e não só notar, porque ele te compra uma coisa específica. Ele deixa qualquer pessoa confirmar que o bytecode rodando on-chain foi produzido a partir do código-fonte que você consegue de fato ler, em vez de confiar na palavra de um mantenedor de que os dois batem. Junte isso com tags de release assinadas e crates publicados sob um guardião só, e a cadeia de suprimentos que você está herdando é auditável de ponta a ponta. Mantenha isso separado na sua cabeça da auditoria que ainda é devida: você consegue verificar a procedência hoje, até a assinatura, enquanto a revisão de segurança da lógica do framework ainda não aconteceu. Dois tipos diferentes de confiança, e o V2 ganhou exatamente um deles até agora.

## A única chamada que falta: a árvore de decisão de quem migra

Tudo acima alimenta um fluxograma só. As entradas são as mesmas para cada projeto: rotulagem de rc-e-alpha, não auditado, sem data estável comprometida, uma linha v1 que continua se movendo. O que muda é o seu projeto, e isso muda o roteamento.

![Uma árvore de decisão roteando um projeto greenfield para sim, um protocolo vivo que segura valor para não-hoje, e uma base de código v1 grande para mapeie-agora-e-porte-quando-estável-e-auditado.](assets/v06-flowchart.webp)

Leia cada branch como uma frase que você poderia dizer para um colega cético.

Greenfield, aprendizado, ou um experimento sem nada em risco: construa no V2 hoje. A única coisa exposta é o seu tempo, os ganhos são imediatos, e você sai fluente no framework em que todo mundo vai estar quando o estável aterrissar. Este é o branch que este curso inteiro estava caladamente te preparando para pegar sem medo.

Produção, segurando valor real de usuário agora: fique no 1.1.2. A economia de compute é genuína e ela ainda não supera colocar fundos de usuário em cima de código não auditado, sem data, auto-rotulado de alpha. Fique de olho em dois sinais específicos, uma data estável comprometida e uma auditoria publicada, e não trate nem o ganho de CU nem um feature flag como substituto deles. Rodar o RC em produção atrás de um flag não resolve o risco de auditoria. Ele esconde o risco.

Base de código v1 grande já existente: divida a decisão em duas. Mapeie agora, porte depois. Faça a análise de migração hoje, exatamente como a que você rodou na lição passada, para você saber a sua superfície real de port e ela não te surpreender. Depois aperte o gatilho quando a data estável aterrissar e uma auditoria existir. Você já fez o ensaio difícil; este branch só está dizendo para não confundir o ensaio com a noite de estreia.

### Quatro jeitos de ler a árvore errado

A árvore é só tão boa quanto a leitura, então aqui estão as quatro leituras erradas que vão te rotear errado. Cada uma delas é uma armadilha na qual eu vi pessoas cuidadosas caminharem direto.

A primeira é ler "o V2 é mais rápido" como "o V2 está pronto para produção hoje". Essas são afirmações sem relação sentadas em eixos diferentes. Os benchmarks têm ressalva de alpha e o código não é auditado, então velocidade é um argumento para construir o seu próximo projeto de aprendizado no V2, e não é um argumento para mover fundos de usuário para cima dele neste trimestre. A árvore mantém os dois eixos separados de propósito, porque confundir eles é o jeito mais comum de um time de verdade se convencer de um deploy ruim.

A segunda é tratar a ausência de uma data estável como "logo". Uma data faltando não carrega informação nenhuma sobre prazo. Ela não é uma contagem regressiva que você por acaso não consegue ver. Planeje contra o que está de fato comprometido, que hoje é nada, e deixe uma data publicada de verdade mudar o seu plano quando ela aparecer, em vez de deixar a sua esperança mudar ele antes disso.

A terceira é ouvir "nenhum outro curso de V2 existe" como uma lei provada do universo. É um resultado de levantamento de uma busca exaustiva, não um teorema. Material novo pode aterrissar na semana que vem e caladamente deixar a afirmação falsa. Declare isso do jeito que você gostaria que uma afirmação sobre o seu próprio trabalho fosse declarada: como o que uma olhada cuidadosa e datada achou, e como algo honestamente aberto a estar errado.

A quarta, e a que caladamente custa mais, é assumir que um tópico que este curso pulou é um tópico que não importa. Cada omissão aqui foi um repasse, não um veredito. A interface de transfer hook, o pouso de transação, o runtime embaixo do loader: este curso recusou cada um deles porque um irmão nomeado é dono dele e ensina ele melhor do que um capítulo aparafusado conseguiria. Pulado não é a mesma coisa que sem importância, e confundir os dois é como você acaba reconstruindo, mal, uma coisa que alguém já ensinou bem.

![Uma tabela emparelhando cada um dos quatro jeitos de ler a árvore de adoção errado com por que ele está errado e a leitura corrigida.](assets/v07-table.webp)

## Lab: roteie três perfis, e depois verifique uma assinatura

O lab é um lab de raciocínio, não um lab de código, com um comando de verdade no fim. Trabalhe ele em ordem. O recuo da ajuda quer dizer que eu te dou os perfis e os checkpoints; as justificativas são suas para escrever.

1. Pegue três perfis de projeto: um build de aprendizado de fim de semana sem nada em jogo, um protocolo vivo segurando fundos de usuário hoje, e uma base de código v1 de 40,000 linhas em produção. Para cada um, nomeie o branch para o qual ele roteia.

2. Para cada roteamento, escreva uma frase de justificativa que cite as tensões de verdade pelo nome, não "parece mais seguro". Uma boa justificativa para o protocolo vivo lê assim: "não hoje, porque o V2 é rotulado tanto de rc quanto de alpha, a documentação diz que ele não é auditado, e nenhuma data estável está comprometida, então o valor em risco fica na linha 1.1.2 mantida." Checkpoint: se a sua justificativa não nomeia pelo menos duas das quatro tensões congeladas, ela é um clima, não uma decisão. Reescreva ela.

3. Agora faça o de verdade. Pegue o projeto do seu post-it do começo da lição e roteie ele. Escreva a justificativa dele do mesmo jeito. Esta é a chamada que você de fato veio aqui fazer. Checkpoint: o post-it agora carrega um branch (sim, não hoje, ou mapeie-agora-porte-depois) e uma frase nomeando as tensões que forçam ele. Se a frase não sobreviveria a um colega cético perguntando "por que não no mês que vem em vez disso", ela não está pronta.

4. Confirme o toolchain no qual os seus branches de "sim" vão aterrissar. Você instalou o RC lá no m10-l3; isto é uma checagem, não uma quarta instalação:

```bash
anchor --version   # expect 2.0.0-rc.1 (the pin as of 2026-08-22; no stable date is committed)
```

Checkpoint: o `anchor --version` imprime a string V2 que você fixou. Se ele imprime uma versão 1.x, o seu PATH está resolvendo o binário antigo primeiro; corrija o PATH, ou reinstale com o comando git fixado do m10-l3, e re-cheque antes de seguir em frente.

5. Faça a jogada de "não confie, verifique" na superfície de confiança. A OtterSec assinou com GPG a tag v2. Cada instalação neste curso passou por `cargo install --git`, que não te deixa nenhum repositório para inspecionar, então clone um e cheque a assinatura em vez de assumir:

```bash
git clone https://github.com/otter-sec/anchor.git anchor-src
cd anchor-src
git verify-tag v2.0.0-rc.1
```

Checkpoint: o comando reporta uma assinatura boa uma vez que a chave pública da OtterSec esteja no seu keyring. Se ele der erro com "no public key", isso é esperado até você importar a chave deles. O ponto não é que ele passe na primeira tentativa. O ponto é que a assinatura seja checável, ponto, que é a propriedade que você quer do guardião da sua cadeia de suprimentos.

## Challenge: faça a chamada de memória

Feche as anotações. Aqui está a trava.

Você recebe três projetos: um build de aprendizado de fim de semana, um protocolo vivo segurando fundos de usuário, e uma base de código v1 de 40,000 linhas. Coloque cada um na árvore de decisão de adoção do V2 e justifique o roteamento, de memória, usando a rotulagem de RC-e-alpha, o status não auditado, a data estável faltando, e a linha v1 ainda mantida. Produza três triplas de projeto-para-decisão-para-justificativa.

Você passa quando todos os três roteamentos estão corretos e cada justificativa nomeia as tensões específicas que forçam ele, não um desconforto geral. Se você consegue fazer isso sem olhar para trás, você consegue fazer isso numa reunião de planejamento de verdade, que é o único lugar onde isso importa.

## Feedback, e onde a camada de framework repassa

Um momento honesto antes da porta. Se o roteamento do protocolo vivo pareceu anticlimático, "a coisa nova e rápida, e a resposta é esperar", sente com o porquê de ele não ter parecido assim de escrever. Recomendar paciência para os fundos de usuário de outra pessoa, em cima de alpha não auditado, é a coisa mais otimista-de-builder deste curso, não a menos. Otimista na tecnologia, honesto no risco. É essa a postura inteira.

Aqui está uma coisa que vale saber, enquadrada como resultado de levantamento e não como um absoluto. O caminho de aprendizado oficial em solana.com/developers/courses agora faz redirect 308 para um repositório de conteúdo de desenvolvedor que foi arquivado e congelado em 2025-01-24. Em 2026-08 a gente foi olhar e não achou nenhum outro curso de Anchor V2 em lugar nenhum. Essa é a razão de esta conclusão ser uma rampa de entrada e não um competidor: você está, tanto quanto uma busca exaustiva consegue dizer, segurando o mapa atual de um lugar que quase ninguém escreveu ainda.

Este é o fim do curso, então o gancho para frente aponta para fora em vez de para uma próxima lição. A camada de framework é sua agora, e ela é deliberadamente só a camada de framework. As coisas que este curso recusou, ele recusou porque um irmão é dono delas e ensina elas do jeito certo.

![Uma tabela de repasse roteando cada próximo tópico para o curso irmão que é dono dele: Digital Assets, Client-Side, Low-Level Solana, DeFi e RWA, e Payments e Commerce.](assets/v08-table.webp)

Cada um desses cursos constrói exatamente em cima do que você acabou de aprender — a camada de framework é sua agora, e eles se apoiam nela. Dois deles já estão publicados, Digital Assets e Payments and Commerce; Client-Side, Low-Level Solana e DeFi and RWA Engineering são irmãos planejados ainda em produção, então leia as linhas deles como um mapa de onde cada assunto mora, não como portas para clicar já. O curso de Digital Assets é dono dos padrões de token que este curso tocou só da cadeira do programa, a interface de transfer hook incluída. O curso de Client-Side é dono de fazer uma transação de fato aterrissar e de ler dados da cadeia de volta para fora — a metade de cliente inteira que este curso nunca abriu uma vez. O Low-Level Solana vai para baixo do loader em cima do qual você esteve de pé esse tempo todo, para dentro do sBPF e dos syscalls, sem framework nenhum. O DeFi e RWA Engineering é dono do que design de protocolo de verdade e venues de verdade exigem além do swap de brinquedo que você escreveu como padrão de Anchor. E o Payments e Commerce transforma a pilha inteira em trilhos sobre os quais um negócio consegue rodar dinheiro. Cinco camadas, cinco donos, e um desses donos, o da camada de framework, agora é você.

Você terminou o mapa. Você construiu cada programa nele, migrou uma base de código de verdade para dentro dele, e você agora consegue olhar qualquer projeto e dizer, a partir dos fatos, se ele deveria migrar hoje. Essa última habilidade é a que vai continuar verdadeira depois de os números de versão mudarem. Vá fazer a chamada no seu post-it. Você ganhou a confiança para fazer ela, e você não precisa mais de mim na sala para checar o seu trabalho.
