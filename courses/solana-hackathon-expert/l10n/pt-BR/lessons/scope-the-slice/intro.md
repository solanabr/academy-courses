# Recorte o escopo para a demo de três minutos

Na última lição você reconstruiu um brief da Colosseum a partir da página arquivada da Frontier contra o relógio, escreveu os minutos no topo de colosseum-brief-2.md e repontuou o pacote de evidências do Fiado de 0 a 10 em fatores que você copiou em vez de lembrar. Funciona **levou um 3**, sobre um plano, porque nada tinha sido construído. Hoje esse número começa a se mover, *e a primeira coisa que você constrói continua não sendo código*.

## Por que a demo vem antes do código

Os projetos que se classificaram não estavam 100 por cento completos. Estavam **completos o suficiente para uma demo** ser feita, e a demo tinha sido decidida antes do código. Você escreve os três minutos que um jurado vai assistir, *e depois constrói só o que esses três minutos mostram*.

Três minutos é o número da página. Na temporada da World's Fair de 2026, a Colosseum pede um vídeo de demonstração do produto de **no máximo três minutos** explicando como o produto funciona, e um vídeo de apresentação separado, de dois a três minutos, que ela chama de um dos primeiros recursos que os jurados revisam. Três minutos, *com uma transação real dentro*.

## Faça isto

1. **Crie demo-script.md** ao lado do pacote de evidências e coloque quatro linhas nele, uma por marca, com o que está na tela em cada uma. Nada mais, e nenhum editor de código aberto:

```text
demo-script.md, v0, <data de hoje>
0:00  na tela:
0:30  na tela:
1:30  na tela:
2:45  na tela:
a única transação:
```

Dê **dez minutos** a isso. Se uma marca ficar em branco, deixe em branco. *Uma marca em branco é informação sobre o seu escopo*, e é a razão de este arquivo existir antes do repositório. A skill de hackathon do kit coloca a barra que o arquivo mira no checklist dela: **uma demo de menos de três minutos** com uma transação real, um repositório público com um quickstart funcionando, um link de devnet e um program ID, e um deck se o hackathon exigir. Confira essa lista contra o README atual do kit, *porque checklists mudam*.

2. **Escolha a tarefa única** com duas perguntas: um jurado consegue verificar isso na tela em três minutos, e qual suposição do seu pacote de evidências isso testa. A maioria das ideias começa larga demais, e a minha posição, desde a planilha de MVP, é *escolher uma tarefa para validar primeiro*. Um MVP é uma fatia de **4 a 6 semanas**, e um hackathon te dá duas, então a sua fatia é menor do que aquela para a qual a planilha foi escrita.

Para o Fiado a tarefa é um freguês **pagar parte do fiado** do próprio celular, já que abrir um fiado é um formulário e *um jurado não distingue um formulário que escreve numa chain de um formulário que não escreve em nada*, e o lembrete ainda carrega a nota do memorando de decisão, "precisa de um fiado". O registro de fornecedores e os selos de fidelidade do memorando de ideia não testam nada que o pitch afirma.

![Das cinco tarefas candidatas do Fiado, só pagar parte do fiado confirma na tela e testa uma afirmação do pacote de evidências, com o lembrete acrescentado por último como instrumento para a próxima suposição.](assets/v01-diagram.webp)

3. **Escreva o caminho feliz** do jeito que um usuário o percorreria, com o mercadinho aberto e a dona atrás do caixa, e não edite enquanto escreve. Depois conte. Se o caminho feliz passar de **sete passos** você está escopando demais, e a correção é cortar passos inteiros para a lista de não-metas até caber, *nunca dobrar três passos num só para a conta fechar*.

O primeiro rascunho do Fiado deu **onze**. Instalação, conta, perfil do mercadinho e conexão da carteira foram para as não-metas como passos inteiros, o freguês instalando uma carteira foi junto, e a data de vencimento e o lembrete voltaram como passos 6 e 7, o instrumento para a afirmação que ninguém conseguiu testar na semana 1. Isso dá sete, *sem nada fundido*.

![O primeiro rascunho de onze passos do Fiado perde cinco passos inteiros de configuração para a lista de não-metas e ganha a data de vencimento e o lembrete, chegando a sete passos sem nada fundido.](assets/v02-flowchart.webp)

4. **Escreva o cartão de escopo** numa página com quatro campos: a tarefa única, o caminho feliz, a única transação e as não-metas. Diga as não-metas em voz alta, *porque uma não-meta que vive na cabeça de alguém é construída na semana 3*. O cartão do Fiado, como vai no repositório ao lado do pacote de evidências:

```text
scope-card.md, Fiado, semanas 2 e 3, <data>

tarefa única:   um freguês paga parte do fiado do próprio celular

caminho feliz:
  1  a dona abre o Fiado no celular dela e abre um fiado para um freguês:
     o primeiro nome dele e o telefone dele
  2  ela adiciona a compra de hoje, 40, e o fiado marca 40 devidos; um link vai para o celular dele
  3  o freguês abre o link e vê os mesmos 40 no celular dele, sem instalar, sem login
  4  ele toca em "pagar parte", digita 15, e a carteira dele pede para aprovar uma
     transferência de USDC na devnet
  5  a transferência confirma, os dois celulares marcam 25 devidos, e um link embaixo do saldo
     abre a assinatura num explorer
  6  a dona define uma data de vencimento para os 25
  7  o lembrete chega no celular do freguês com os 25 e o link

a única transação:
  a transferência de USDC no passo 4, da carteira do freguês para a carteira da dona,
  na devnet, com um memo nomeando o fiado; a assinatura dela está na tela no passo 5;
  o saldo que os dois celulares mostram é derivado do histórico de transferências do fiado

não-metas (não agora):
  contas, login, cadastro, um perfil do mercadinho, uma tela de configurações
  colocar o freguês numa carteira, ou colocar fundos nela
  reais entrando ou saindo (o on-ramp e o off-ramp de moeda fiduciária)
  pagar um fiado inteiro, ou fechar um
  mercadinhos se cadastrando sozinhos, um segundo dono por mercadinho
  o registro de fornecedores e os selos de fidelidade do memorando de ideia
  um programa próprio do Fiado para o fiado (os pagamentos já são o registro
  on-chain, uma transferência marcada com memo cada; o diretório de clientes, o valor
  de abertura, a data de vencimento e o lembrete ficam num backend simples; um programa
  de fiado vai para o slide de próximos passos)
```

Os valores são do próprio roteiro, **40 no fiado e 15 pagos**, para que dois celulares mostrem o mesmo número mudando, e qualquer par pequeno serve. Leia cada passo do jeito que uma câmera leria. Se um passo não tem tela, não é um passo, é encanamento, *e encanamento vai embaixo de um passo, nunca ao lado*. Embaixo do cartão escreva a nota da Lucia da rodada 1 de feedback, de que os fregueses dela nunca deveriam encontrar **a palavra stablecoin**, como restrição para toda tela que o freguês vê: a tela dele diz pagar parte e um valor, a carteira dele diz USDC porque carteiras dizem, e a narração diz Solana uma vez, em 1:30, para o jurado e não para ele.

5. **Dê um destino a cada não-meta**. Cada linha diz não neste mês, e as linhas melhores também dizem para onde a coisa vai, *porque uma não-meta sem destino é discutida de novo na semana 3* por alguém que não estava na sala quando ela foi cortada. A lista tem **pelo menos três linhas**, e a primeira é a que o time mais queria construir. Se a lista parecer curta e confortável, volte ao seu caminho de onze passos, ou qualquer que tenha sido a contagem, e leia o que foi removido, *porque essas são as suas primeiras linhas*. A lista do Fiado se divide em **três destinos**: o slide de próximos passos, depois da temporada, e de volta ao memorando de ideia.

![Cada uma das sete não-metas do Fiado tem um destino, a maioria o slide de próximos passos, duas delas depois da temporada e uma de volta ao memorando de ideia, com o programa de fiado marcado para o tradeoff.](assets/v03-table.webp)

6. **Nomeie a única transação** com duas perguntas: é o dinheiro se movendo de verdade, na direção que o pitch diz, e o espectador consegue ver a confirmação. Para o Fiado é uma **transferência de USDC na devnet** da carteira do freguês para a da dona, uma transferência de token com um memo nomeando o fiado e não uma chamada de programa, *porque é a integração mais leve que ainda prova o valor da frase*. O saldo que os dois celulares mostram é derivado do histórico de transferências daquele fiado, então o número que o freguês lê é um que qualquer um pode recalcular pelo explorer *sem confiar no app*.

Escreva **três notas** no cartão para a semana de build. USDC de devnet é um token de teste que vem de um faucet, então a carteira do freguês é **abastecida antes de gravar**. A assinatura entra no log narrativo no dia em que existir, com o link do explorer, *porque é a única linha em que um jurado pode clicar*. E a confirmação fica em **1:30** e não no fim, onde um espectador que parou de prestar atenção a perderia. Se o seu projeto não tem dinheiro se movendo, a única transação é qualquer mudança de estado que o pitch afirma e que um estranho consegue verificar: um registro escrito, um token mintado, uma assinatura que prova quem fez o quê e quando.

7. **Preencha as quatro marcas** a partir dos sete passos. Uma marca é uma tela mais uma frase de narração e nada mais, *e nenhuma funcionalidade é narrada se não está na tela naquele momento*. O roteiro v0 do Fiado, com a marca de 1:30 deixada para você:

```text
demo-script.md, Fiado, v0, <data>

0:00  na tela: o celular da dona, o Fiado aberto numa lista de fiados vazia. Ela toca em novo fiado e
      digita o primeiro nome e o telefone do freguês. Narração, uma frase: o que é um fiado num
      mercadinho de bairro. Em 0:25 ela já adicionou a compra de hoje e o fiado marca 40 devidos.
      (passos 1 e 2)

0:30  na tela: o segundo celular. O freguês abre o link nas mensagens dele e vê
      40 devidos, o mesmo número, sem instalar, sem login. Narração: a dona e o freguês
      estão lendo um saldo só, que é a coisa que a caderneta nunca conseguiu fazer. (passo 3)

1:30  na tela: <escreva esta marca: passos 4 e 5. Nomeie a transação, o que o espectador
      vê enquanto ela confirma, e o que os dois celulares marcam depois que confirma>

2:15  na tela: o celular da dona. Ela define uma data de vencimento para os 25. O celular do freguês
      acende com o lembrete, os 25 e o link. Narração: o lembrete é a
      afirmação que o time está testando com seus design partners (os clientes parceiros). (passos 6 e 7)

2:45  na tela: a lista de fiados da dona, uma linha, 15 pagos com o link da assinatura, 25 vencendo
      na data. Narração, uma frase: o que não está nesta demo e onde vive.
      Corte em 3:00 ou antes.
```

**Escreva a marca de 1:30 do Fiado** a partir dos passos 4 e 5 do cartão, no formato que as outras marcas usam: o que está na tela, uma frase de narração, os números dos passos entre parênteses. Ela tem que nomear a transação, dizer o que o espectador vê enquanto confirma, e dizer o que os dois celulares marcam depois. Depois leia o roteiro inteiro em voz alta **com um cronômetro rodando** e veja onde o 1:30 cai de verdade. Se a transferência confirmar depois de 2:00 na sua leitura, *as marcas antes dela estão longas demais, não a transferência*.

Repare no que cada marca remove do repositório: o passo 2 precisa de **um campo de valor e nenhum catálogo**, o passo 3 precisa de um link que abre num navegador e nenhum app de cliente, e o passo 7 precisa de uma mensagem num celular, então qual serviço a envia é *uma decisão da semana de build*.

8. **Agora o seu**, a partir do seu pacote de evidências e do seu pitch v1, sozinho: a tarefa única, o caminho feliz contado e cortado, as não-metas com destinos, a única transação e a direção dela, depois as quatro marcas com a transação em 1:30 ou perto disso. No mesmo dia **escreva o slide de próximos passos**, duas linhas, nomeando a arquitetura em que a demo para antes e por que ela ainda não está lá, para o Fiado um programa de fiado que coloca o valor de abertura e a data de vencimento ao lado dos pagamentos.

Um jurado pontuando Product + Execution (produto e execução) lê esse slide como um time que escolheu, as funcionalidades priorizadas estrategicamente que a revisão do repositório pede, *e um jurado que encontra a lacuna na entrevista lê como um time que a escondeu*. **Date os dois arquivos**, faça commit deles ao lado do pacote de evidências, e espere uma v1 quando o build os mudar.

![A demo do Fiado coloca o fiado em 0:00, o saldo compartilhado em 0:30, a transferência na devnet em 1:30, o lembrete em 2:15 e a lista de fiados de fechamento em 2:45, abaixo do limite de três minutos.](assets/v04-timeline.webp)

## Está pronto quando

- O caminho feliz tem **sete passos ou menos** e cada passo é algo que uma câmera consegue ver.
- A lista de não-metas tem **pelo menos três linhas** e a primeira linha dói um pouco.
- O roteiro nomeia **a única transação** e o que o espectador vê quando ela confirma.

## Fique atento

- **Um caminho feliz com um login**, um cadastro e uma tela de configurações dentro é a armadilha que esta lição existe para nomear.
- Um agente constrói **uma tela de configurações em segundos** e um jurado dá zero pontos a ela, então escolha a tarefa pelo que um jurado consegue avaliar, *não pelo que o agente constrói mais rápido*.
- Se nada na demo deixa uma assinatura, ela é **um vídeo de um site** e é pontuada como tal, já que um jurado não consegue verificar um mock.

O tradeoff: uma fatia que demonstra bem em três minutos muitas vezes **não é a arquitetura que você lançaria**, e um mês de hackathon não constrói as duas, então *eu prefiro mostrar uma transferência que confirma e um slide que diz o que vem a seguir do que uma arquitetura que ninguém consegue assistir*.

## O que fica

A demo é decidida antes do código, e o código é só o que os três minutos mostram. Uma fatia bem escopada cabe numa respiração, algo como *"ela abre um fiado, ele paga parte, o lembrete dispara"*, e se precisa de duas respirações um passo está carregando uma segunda tarefa, e esse passo é **a sua próxima não-meta**. Toda não-meta ganha um destino, para ninguém discutir de novo na semana 3.

## A seguir

A próxima lição abre com **o kit clonado e contado**, depois um aquecimento de uma hora em que você roda uma transcrição de plan mode (modo de planejamento) já pronta para os três primeiros passos do Fiado e vê a transação chegar, e só então os seus sete passos entram numa sessão de plan mode e o time de agentes os constrói na devnet. Você vai se surpreender com a velocidade, depois com o que ele errou de um jeito plausível, *e o roteiro de hoje é como você vai saber qual é qual*.
