# Construa com um time de agentes

Na última lição você escreveu o roteiro da demo v0 com suas quatro marcas de tempo e cortou o build para um cartão de escopo: uma tarefa, sete passos, sete não-metas e a única transação, um freguês pagando 15 de um fiado de 40 em USDC na devnet. **Deixe os dois arquivos abertos**, *porque todo prompt e toda revisão desta semana saem deles*.

## Por que isso importa

O FAQ da própria Colosseum, na temporada da World's Fair de 2026, diz com todas as letras: 'We have backed non-technical founders in our Accelerator who built MVPs entirely with AI coding tools' (já apoiamos no nosso Accelerator fundadores não técnicos que construíram MVPs inteiramente com ferramentas de código com IA) (colosseum.com/hackathon, 2026-09-06). *Os jurados não se importam que um agente escreveu o código.* Eles se importam que a coisa funcione, que o trabalho tenha acontecido entre a data de início e a de fim da temporada, e que eles consigam **rodar a partir do seu README**.

Então a semana tem uma ideia só: **o time de agentes constrói os passos**, e você guarda para si o caminho da demo, a única transação e o clone limpo. As ferramentas são o Claude Code e o plugin Solana AI Kit desde o dia 0, e *a árvore do próprio kit é a primeira coisa que você lê*. Clone o kit ao lado do seu repositório do toolkit:

```bash
git clone https://github.com/solanabr/solana-ai-kit
cat solana-ai-kit/.gitmodules
ls -l solana-ai-kit/plugin/skills
```

## Faça isto

![A semana vai do cartão de escopo, passando por uma sessão de plan mode e três subagentes, até uma transação que você envia e lê à mão, depois um quickstart em clone limpo, com a regra da janela da temporada embaixo.](assets/v01-flowchart.webp)

1. **Conte as entradas de submódulo** na primeira saída e os symlinks na última. Lido em 2026-09-06, num último commit datado de 2026-08-20, a primeira saída tinha **18 entradas**, entre elas colosseum, que aponta para ColosseumOrg/colosseum-copilot, solana-new, que aponta para sendaifun/solana-new, e helius, solana-dev, sendai, metaplex e jupiter. A última saída tinha três, hackathon, idea-sprint e pitch-deck, cada uma um symlink para dentro de .claude/skills, onde a skill wrapper vive.

2. Abra **uma das três skills wrapper** e leia o cabeçalho. Cada uma diz que foi adaptada de sendaifun/solana-new, MIT 2026 SendAI e Superteam, telemetria removida, e *é por isso que o wrapper é a versão que você roda sempre que os dois existem*.

Os wrappers são **a primeira camada do kit**. A segunda são os comandos e agentes do próprio kit, e os quatro em que esta semana se apoia são /plan-feature, /scaffold, /build-app e /diff-review.

A terceira é ext, que só chega com a instalação completa que o README descreve, e ext/solana-new guarda as três skills que uma semana de build busca quando um wrapper não cobre algo: scaffold-project, build-with-claude e debug-program. As skills upstream ali carregam um preâmbulo bash no topo que liga para casa antes de a skill rodar, e a página hub do kit avisa para nunca executar esses blocos de preâmbulo, então abra o arquivo da skill primeiro, **pule o bloco do topo**, depois rode a skill. ext/colosseum é o Colosseum Copilot e ainda precisa do PAT da semana 1.

Todo nome de comando, skill e modo nesta página foi lido em 2026-09-06 contra o main de solanabr/solana-ai-kit e contra a documentação do Claude Code, e os dois mudam com seus releases, então *se a sua saída for diferente destas, a sua saída vence*, e **verifique cada nome** no README atual do kit antes de digitar.

3. **Crie um repositório vazio** chamado fiado-warmup ao lado do toolkit. A Colosseum, na página lida em 2026-09-06, julga os times só pelo trabalho concluído entre as **datas de início e fim** da competição, código pré-existente precisa ser declarado, e mentir sobre qualquer um dos dois pode desclassificar um time, bani-lo e revogar um prêmio. *Um agente vai puxar de bom grado um template que você escreveu ano passado*, então o repositório começa vazio no primeiro dia da semana 2 e qualquer coisa mais antiga é nomeada no README.

4. **Abra o Claude Code dentro do repositório** e abra a sessão de planejamento com /plan-feature. Plan mode (modo de planejamento) é o modo do Claude Code em que o agente lê o repositório e escreve um plano e **não edita nada até você aprovar**. Cole a transcrição abaixo como está. É o cartão de escopo do Fiado da última lição no formato de prompt do kit, e *quando você voltar para a sua própria fatia, o seu cartão entra no lugar destas linhas*:

```text
/plan-feature
Construa a fatia de demo abaixo. Só o plano. Não escreva arquivos ainda.
Tarefa única: um freguês paga parte do fiado do próprio celular.
Passo 1: a dona abre o Fiado no celular dela e abre um fiado para um freguês: o primeiro nome dele e o telefone dele.
Passo 2: ela adiciona a compra de hoje, 40, e o fiado marca 40 devidos; um link vai para o celular dele.
Passo 3: o freguês abre o link e vê os mesmos 40 no celular dele, sem instalar, sem login.
Passo 4: ele toca em "pagar parte", digita 15, e a carteira dele pede para aprovar uma transferência de USDC na devnet, com um memo nomeando o fiado. Esta é a única transação real.
Passo 5: a transferência confirma, os dois celulares marcam 25 devidos, e um link embaixo do saldo abre a assinatura num explorer.
Passo 6: a dona define uma data de vencimento para os 25.
Passo 7: o lembrete chega no celular do freguês com os 25 e o link.
O saldo é derivado do histórico de transferências do fiado; o app guarda só o diretório de clientes, o valor de abertura, a data de vencimento e o lembrete.
Não-metas: contas, login, cadastro, um perfil do mercadinho, uma tela de configurações; colocar o freguês numa carteira, ou colocar fundos nela; reais entrando ou saindo; pagar um fiado inteiro, ou fechar um; mercadinhos se cadastrando sozinhos, um segundo dono por mercadinho; o registro de fornecedores e os selos de fidelidade; um programa próprio do Fiado para o fiado.
Ordene o plano para que o passo 4 seja construído e rodado antes de os passos 5 a 7 começarem.
```

5. **Leia o plano por dez minutos** e não faça mais nada, contra três coisas. Primeiro, as não-metas: o plano muito provavelmente vai propor um login, porque a maioria dos templates tem um, e *a linha de não-meta está ali para você cortar agora e não na semana 3*. Segundo, a ordem: **passo 4 antes** dos passos 5 a 7, porque tudo depois dele depende de uma transação que existe.

Terceiro, o formato da transação: o plano deve dizer USDC, devnet, e uma transferência real da carteira do freguês para a do mercadinho, e se disser **mock, stub ou simulate** em qualquer lugar perto do passo 4, é o plano te avisando que pretende construir uma demo que um jurado não consegue verificar. O cartão do Fiado não carrega programa próprio, então *se o plano propõe um, ele passou por cima do cartão*. Aprove só quando as três verificações passarem, **salve o plano como arquivo** com a data na primeira linha, e faça commit, porque um jurado lendo o repositório consegue ver que ele foi escrito dentro da janela.

```bash
git add plan.md
git commit -m "plan approved, 2026-09-22"
```

6. **Divida o plano em três subagentes** com uma tarefa cada, e escreva cada tarefa como uma spec de poucas linhas tirada do cartão, *nunca de memória*. Um subagente é um segundo agente que o Claude Code inicia para uma tarefa, e subagentes **não compartilham memória**, então a única coisa que eles compartilham é o repositório em disco.

O subagente de scaffold roda o comando /scaffold do kit contra o plano aprovado e para quando o repositório tem um README com uma seção de quickstart vazia, um arquivo de pacote e **uma pasta por passo**. O subagente de frontend constrói as telas com /build-app, e a spec dele são as linhas dos passos mais as marcas de 0:00 e 0:30 do roteiro da demo. O subagente de testes escreve **um teste por passo**, como o que uma pessoa vê quando o passo está pronto, e o teste que confere o saldo depois do passo 4 deve existir antes do passo 4, *para o passo ter algo em que falhar*.

**Rode o scaffold primeiro**, depois o frontend para o passo 1 sozinho, depois o passo 2, depois o passo 3, um passo por rodada, e leia o repositório entre as rodadas. *Eu acho que a aceleração não é uniforme entre os tipos de trabalho*: o scaffold chega em minutos, o passo da transação leva uma tarde, e a tarde é **a parte que um jurado consegue checar**.

7. O passo 4 é seu. **Suba o frontend**, abra o fiado do passo 1, toque em pagar parte no passo 3, digite 15 e pague com a carteira de devnet que você criou e abasteceu num faucet de devnet no dia 0. Se ela não tiver USDC de devnet, complete antes num faucet de USDC de devnet (confira o atual no README do kit), *porque uma transação que falha por saldo vazio não te ensina nada*. Quando a carteira confirmar, o app mostra **uma assinatura**, a string longa em base58 que identifica uma transação na devnet.

**Copie a assinatura, abra qualquer explorer**, mude para devnet e cole. **Leia a linha de status primeiro**, porque uma transação pode ser enviada e ainda assim falhar. Depois o slot e o horário do bloco, *que datam o trabalho dentro da janela da temporada de um jeito que nenhuma mensagem de commit consegue*. Depois a taxa, paga em SOL pela carteira que assinou. Depois as mudanças de saldo de token, USDC saindo da conta do freguês e chegando na do mercadinho, pelos 15 que o passo 3 escolheu. Depois o memo, que nomeia o fiado. Se o valor no explorer não é o valor na tela, **a tela está mentindo** e o explorer não.

![A leitura do explorer vai de assinatura, status, slot e horário, taxa, até a mudança de saldo de USDC comparada com a tela do app, e um resultado de não encontrado significa que a transação nunca chegou à devnet.](assets/v02-flowchart.webp)

8. **Comece um arquivo chamado narrative-log.md** na raiz do repositório e faça desta a primeira entrada:

```text
data        o dia em que o passo 4 rodou, escrito como 2026-09-22
evento      passo 4 rodou na devnet
assinatura  a assinatura completa, colada
explorer    o link do explorer, em devnet
valor       o valor em USDC que o passo 3 escolheu
visto       status, slot, taxa e a mudança de saldo de token, tudo lido no explorer
```

Esse arquivo cresce ao longo da passada de polimento, e o deck e os dois vídeos são cortados dele, mas *esta primeira entrada é a linha em que um jurado pode clicar*.

9. **Teste o quickstart** a partir de um clone limpo. Um quickstart é a parte do README que leva um estranho de um clone limpo até a fatia rodando em poucos comandos, e *o jurado lendo o seu repositório é esse estranho*. Clone o seu próprio repositório numa pasta que nunca o viu:

```bash
git clone <your-repo-url> fiado-clean
cd fiado-clean
```

Depois faça exatamente o que o README diz, e nada que ele não diz. Se ele esqueceu o arquivo de ambiente, a configuração da carteira ou o USDC de devnet, você para, **escreve a linha que falta** no README e clona de novo numa pasta nova. Repita até um clone que começa do nada chegar à tela do passo 1, e **cronometre a última rodada**, porque esse número vai para o README também.

O agente escreveu um primeiro quickstart durante o scaffold e ele vai parecer bom, porque foi escrito de dentro de uma sessão que tinha o arquivo de ambiente que o clone não tem. *Trabalho escrito por IA pode parecer correto e estar errado*, e o clone limpo é o instrumento mais barato que você tem para pegar isso, já que custa uma pasta e dez minutos.

Depois **escreva as duas linhas de declaração do README** embaixo do quickstart: o que no repositório é anterior à temporada, se houver algo, e de onde veio, ou a data em que o repositório foi criado se nada for. *As mesmas duas linhas vão para as respostas do portal no dia da submissão.*

![O clone limpo pega o arquivo de ambiente que falta, a carteira sem fundos e a dependência instalada à mão que a máquina do autor esconde, e o README também carrega a declaração de qualquer código mais antigo que a temporada.](assets/v03-diagram.webp)

10. Os passos 5 a 7 **vão para os mesmos subagentes** com o mesmo tipo de spec, um passo por rodada. No cartão do Fiado o subagente de frontend recebe o passo 5 e o passo 7, a tela que marca 25 devidos com o link do explorer embaixo e o lembrete chegando no celular do freguês, e o subagente de testes recebe o passo 6, *porque uma data de vencimento é fácil de falsificar e um teste que dispara o lembrete contra uma data fixa pega a falsificação*. Leia o repositório entre cada um, depois **pare de construir**.

11. **Rode o /diff-review do kit** em tudo que os subagentes escreveram desde que o plano foi aprovado, trate a saída como uma lista de lugares para olhar e *nunca como aprovação*, depois leia o caminho da demo você mesmo, à mão, da marca de 0:00 até a marca de 2:45 do roteiro, com o explorer aberto numa segunda tela.

No Fiado a leitura à mão encontra isto: o passo 5 mostra 25 devidos, o teste passa, e o número é os 15 do formulário de pagar parte subtraídos dos 40, com a transação confirmada **nunca lida**, então a tela mostraria o mesmo número se a transação tivesse falhado. Nada no diff parece errado, o teste foi escrito a partir da mesma suposição, e *um jurado que paga um valor diferente na entrevista assiste ao app mentir*. A correção é pequena, o app lê o valor da transação confirmada e mostra ele, e **encontrar isso levou mais tempo do que construir** o passo.

*A minha posição é que código escrito por IA precisa de uma revisão diferente e mais longa do que código que uma pessoa escreveu*, porque a pessoa que escreveu não está na sala para te dizer o que ela supôs, e os times subestimam esse tempo. **Coloque a revisão no calendário** como um bloco do mesmo tamanho do bloco de build, e quando ela terminar antes, pegue o tempo de volta.

12. **Grave a atualização semanal da Colosseum** no dia em que o passo 4 funcionar. Ela é opcional e fortemente recomendada, nas palavras da própria página em 2026-09-06, e é **um minuto de vídeo**, e a coisa na tela é a transação confirmando, antes de a passada de polimento deixar a tela mais bonita, *porque uma tela simples com uma assinatura real vale mais para um jurado do que uma tela desenhada com um mock*.

![A semana 2 vai de um repositório vazio, passando pelo plano, os subagentes e a transação na devnet, e a semana 3 dá os passos 5 a 7 e um bloco de revisão do mesmo tamanho do build antes da rodada final de clone limpo.](assets/v04-timeline.webp)

13. Agora o seu: **a sua própria fatia** pelos sete passos, sozinho, até a devnet. Um repositório vazio datado desta semana, o seu cartão de escopo no prompt de planejamento com os seus próprios sete passos e não-metas, as três verificações antes de aprovar, os mesmos três subagentes um passo por rodada, e *a mesma regra sobre quem fica com a transação*. Se a sua fatia tem um programa dentro, **/build-program e /deploy** são os comandos do kit para isso, e a passada de polimento roda a auditoria.

## Está pronto quando

- O quickstart funciona a partir de um clone limpo, seguindo só o README, e **a última rodada foi cronometrada**.
- Uma **assinatura de devnet** está em narrative-log.md com o link do explorer, o link abre, e o valor no explorer bate com o seu roteiro.
- A marca de **1:30** do roteiro da demo é alcançável na tela a partir da tela do passo 1 sem tocar em nada que o roteiro não mostra.

## Fique atento

- Um agente que esbarra num erro de devnet às vezes aponta o app para **um validador local ou um mock**, e a assinatura que ele mostra não abre nada. Não encontrado significa que nunca saiu do localhost: corrija a rota, envie de novo, leia de novo, depois escreva a entrada do log.
- As skills upstream em ext/solana-new rodam um **preâmbulo de telemetria** no topo do arquivo, e o dano é silencioso. Leia a skill, pule o bloco, rode a skill.
- Um time de agentes entrega rápido e entrega **plausível mas errado**, então a velocidade é comprada com tempo de revisão, e o bloco de revisão fica no calendário com o tamanho do bloco de build.

## O que fica

O time de agentes constrói os passos, e você guarda para si o caminho da demo, a única transação e o clone limpo. O que a semana de build produz é **uma assinatura que um jurado pode abrir e um README que um jurado pode rodar**, e a revisão que pega o passo plausível mas errado leva tanto tempo quanto o build. *Se a semana saiu como uma lista de funcionalidades, leia a primeira entrada de narrative-log.md de novo.*

## Próxima lição: faça parecer real

Na próxima lição você faz a fatia parecer real, que é um trabalho diferente de deixar ela mais bonita. A primeira ação é **abrir a tela do passo 1** que o subagente de frontend construiu, tirar um screenshot datado dela hoje e colocar em narrative-log.md embaixo da assinatura, *porque um frontend que um jurado reconhece num segundo como feito por agente custa mais pontos do que uma funcionalidade faltando*. Deixe a aba do explorer aberta.
