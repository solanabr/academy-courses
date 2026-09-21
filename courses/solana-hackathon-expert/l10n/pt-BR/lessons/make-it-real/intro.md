# Faça parecer real, depois diga o que mudou

Na última lição você entregou o recorte de sete passos na devnet com um time de agentes, e a última linha do seu log é a assinatura da transação que um jurado pode abrir. Deixe o recorte rodando numa segunda tela, porque **hoje não se constrói nada novo**, com uma pequena exceção para os mercadinhos que começam a usar de verdade. Hoje ele é olhado, por você e por gente que não participou do build, e *o que essas pessoas veem vai para o papel*.

## Qual é a sua

Duas demos, a mesma funcionalidade. Uma tem um frontend que qualquer jurado que já usou uma ferramenta de agente reconhece num segundo: o mesmo layout, o mesmo gradiente, o mesmo texto de placeholder. A outra parece que alguém se importou com a pessoa do outro lado da tela. As demos que se classificaram **não pareciam feitas por IA**, e funcionavam o suficiente para uma demo, e *nada nessa frase diz bonita ou completa*. **Descubra qual é a sua** antes de continuar lendo.

## Mão na massa

1. **Mantenha o log narrativo**, começando com um screenshot de antes. Abra o recorte na tela em que a demo passa mais tempo, no Fiado a tela do fiado. Tire um screenshot dela *exatamente como está*, cole no log embaixo da data de hoje e escreva uma linha ao lado: antes da passada. Depois leia em voz alta cada palavra dessa tela, conte as que nomeiam uma pessoa ou um mercadinho, e **escreva a contagem** ao lado do screenshot.

```text
log narrativo, Fiado, dia 15
screenshot:  tab-screen-day15.png (antes da passada)
palavras na tela nomeando uma pessoa ou um mercadinho: 0
o que a tela diz em vez disso: Dashboard, Welcome back, Total Balance
```

A contagem é **quase sempre zero**, porque é isso que um subagente de frontend entrega *quando ninguém disse a ele quem é o usuário*. Um log narrativo é um registro do build, com data, **mantido enquanto o build acontece**: um screenshot sempre que uma tela mudou, com o antigo deixado em cima, a assinatura de qualquer transação que a demo vai mostrar, com o link, uma decisão sempre que algo foi cortado, nas palavras que o time usou na hora, e uma vez por semana o roteiro do vídeo de atualização. O deck e os dois vídeos da semana 4 são **cortados desse arquivo**.

A atualização semanal da Colosseum é opcional e fortemente recomendada, e o formato é **um vídeo de um minuto**, segundo a página do hackathon lida em 2026-09-06. O roteiro são **quatro frases**: o que mudou esta semana, *mostrado e não descrito*, por que mudou, com a decisão do log, o que um jurado veria agora se clonasse o repositório hoje, e o que vem na próxima semana. **Menos de 60 segundos**, câmera do celular ou gravação de tela, um take. Um segundo take, tudo bem. O terceiro take é a armadilha do polimento. O roteiro da semana 2 do Fiado, como entrou no log no dia 14:

```text
vídeo de atualização, semana 2, Fiado, gravado no dia 14, 52 segundos
0:00  Esta semana o fiado saiu de um mock e foi para a devnet. Aqui está a transação em que
      um freguês paga 15 de um fiado de 40, e aqui está o mesmo fiado marcando 25 no
      celular da dona.
0:18  Cortamos o cadastro do cliente. Dois donos nos disseram na semana 1 que o fiado vive
      no celular deles, então só a dona abre fiados e o freguês só vê um
      saldo.
0:35  Se você clonar o repositório hoje, o quickstart roda os sete passos e o
      primeiro pagamento parcial confirma na devnet.
0:46  Na próxima semana a tela para de dizer Dashboard, e uma página real de uma caderneta
      real entra nela.
```

A linha de 0:18, *uma decisão com a evidência da semana 1 atrás dela*, é a frase que um jurado querendo saber **como o time prioriza** quer ouvir.

![O log do Fiado ganha uma entrada em cada dia em que algo mudou entre o dia 10 e o dia 21, com os screenshots de antes e depois com um dia de diferença e um roteiro de vídeo por semana.](assets/v01-timeline.webp)

2. **Rode a passada de parece-real** nessa tela. Quatro itens, cada um perguntando se uma pessoa que tem esse problema *acreditaria que a tela foi feita para ela*. **O texto nomeia o usuário**: o nome da dona no cabeçalho, ou pelo menos o nome do mercadinho, e o primeiro nome do freguês em cada linha, não Dashboard nem Welcome back. **A demo mostra dados reais**: uma página real de uma caderneta real, primeiros nomes e valores, com a permissão da dona, porque a partir do momento em que Customer 1 a Customer 4 aparecem com números redondos, *o resto da demo é pontuado como mock*.

**Nenhuma cara de componente padrão**: a grade de cards, o cabeçalho com gradiente e os três tiles arredondados de estatística saem, e o único número que importa para a dona vai para onde o olho dela cai primeiro, uma fonte, uma cor de destaque, o total devido em tamanho grande, uns 30 minutos. **O estado vazio está tratado**: abra o app como um mercadinho novo, sem fiados, e se o que aparece é uma tabela com cabeçalhos e nada embaixo, *a demo tem um buraco*, porque essa é a primeira tela que um jurado que clona o repositório vai ver. **Não existe um quinto item** sobre gosto.

![Cada item da passada nomeia o que o frontend padrão entrega, o que um jurado lê nele, a correção, e um custo de minutos para três itens e de uma hora para os dados reais.](assets/v02-table.webp)

A tela do dia 15 do Fiado era a de sempre, três tiles de estatística todos em zero porque liam de outra tabela, e se a sua está parecida, *é isso que a ferramenta entrega, não uma falha sua*. O checklist dela, com **três itens feitos**:

```text
passada de parece-real, Fiado, tela do fiado, dia 16
texto nomeia o usuário    mudou: cabeçalho "Dashboard" -> "Mercadinho da Lucia"; saudação
                          removida; cada linha carrega o primeiro nome do freguês
dados reais na demo       mudou: linhas Customer 1 a 4 -> seis fregueses da página da
                          caderneta da Lucia, primeiros nomes e valores, com a permissão dela
sem cara de padrão        mudou: três tiles de estatística -> um número, o total devido,
                          grande no topo; grade de cards removida; uma cor de destaque
estado vazio tratado      (seu, para terminar)
```

A Lucia é a dona do mercadinho das conversas da semana 1, tia da Ana, já no log de feedback desde o dia 0 e a rodada 1, e o log **registra a permissão dela com a data**, *porque um jurado pode perguntar*. **Agora o quarto item**: abra o Fiado como um mercadinho sem fiados, do jeito que um jurado que acabou de rodar o quickstart veria, tire o screenshot e escreva o estado vazio nas palavras da Lucia, algo como "nenhum fiado ainda, abra o primeiro a partir da caderneta", com o botão que faz isso logo embaixo da frase. **Marque o item com o que mudou**, cole o screenshot de depois embaixo do de antes, e coloque a data.

3. **Faça o roast do recorte**, primeiro com o comando do kit e depois com uma pessoa de fora do build. Roast é uma revisão feita por alguém de fora do build, com a instrução de procurar *o que está errado, não o que está bom*. O kit traz um comando /product-review entre os seus 30 comandos, no repositório solanabr/solana-ai-kit lido em 2026-09-06, e esse comando é **o primeiro roaster**. **Confira o nome do comando** no README atual do kit antes de rodar, *porque o kit acompanha o branch main e nome muda*. Aponte o comando para o repositório e para o roteiro da demo.

**O segundo roaster** é uma pessoa que não construiu o produto, com a mesma instrução: o que está errado, o que confunde, em que você não acreditaria. **Registre três linhas por roaster**, *nas palavras de quem fez o roast*. As do Fiado:

```text
notas do roast, Fiado, dia 17
pessoa    o freguês não consegue saber pela tela o que acontece quando o fiado é
          pago por inteiro
pessoa    o total devido no topo não tem uma data ao lado
pessoa    a palavra devnet aparece no rodapé, onde uma dona de mercadinho leria
          e não saberia o que significa
comando   o quickstart assume uma carteira com fundos e nunca diz isso
```

Duas das quatro viraram decisões no log **na mesma tarde**. A terceira, a data ao lado do total, esperou a rodada 2 de feedback confirmar *que outra pessoa também queria isso*.

4. **Rode a passada de sanidade de segurança**, no formato que se aplicar, e a verificação à mão de qualquer jeito. Se o recorte tem um programa próprio, o comando /audit-solana do kit lê o programa, da mesma lista de 30 comandos e com a mesma ressalva de atualização do comando de roast. Se não tem programa, e um recorte cortado para sete passos muitas vezes não tem, a passada é **uma verificação de assinatura e de manejo de chaves** que uma pessoa faz à mão, *porque a demo assina alguma coisa e alguém segura a chave que assina*.

**Cinco perguntas**, com as respostas no log do jeito que forem. Onde mora a chave que assina a transação da demo: um arquivo no repositório, uma variável de ambiente ou uma carteira no navegador. Essa chave, ou qualquer outra, está **no histórico do git**, *o que uma busca pelos formatos usuais de chave responde em um minuto*. O bundle do frontend entrega algum segredo. A demo assina com a carteira da dona ou com uma chave do time no lugar dela, e o roteiro diz qual. A chave de devnet fica separada de qualquer chave que guarda valor na mainnet, numa máquina separada ou pelo menos num arquivo separado que **nunca está no laptop da demo**.

*Código construído por agente pode parecer certo e ainda assim ter um buraco de segurança*, e encontrar esse buraco é um custo que o humano paga, não a ferramenta: **cerca de uma hora no dia 17**. As respostas do Fiado: a demo assina com um keypair de devnet numa variável de ambiente que, no dia 13, ficou por pouco tempo num arquivo .env commitado. Foi removido, **a chave foi rotacionada**, e o log diz isso com a data, *porque um jurado que acha isso no histórico sem nota é pior do que um jurado que acha a nota*.

![Um recorte com programa roda o comando de auditoria do kit, um recorte sem programa responde cinco perguntas de manejo de chaves à mão, e os dois registram os achados no dia 17.](assets/v03-flowchart.webp)

5. **Coloque o recorte em mãos reais.** O recorte polido vai para os design partners (os clientes parceiros) da semana 1 **esta semana**, não depois do prazo, *porque Traction (tração) é contada em pessoas usando o produto, e essa contagem precisa de dias para crescer*. O builder configura cada mercadinho parceiro à mão, já que cadastro ainda é uma não-meta. O fiado é real: a dona abre fiados para os fregueses reais dela, adiciona compras reais, e os fregueses abrem o saldo nos próprios celulares.

   **A liquidação fica na devnet**, então um freguês real paga como sempre pagou, em dinheiro no caixa, e a dona marca como pago. Esse botão é **a única coisa construída hoje**, porque um mercadinho não consegue tocar um fiado que não pode ser abatido, e a transferência na devnet continua sendo o caminho da demo até o produto ir para a mainnet. Nomes e telefones reais agora estão no seu backend, *então a verificação de manejo de chaves do passo 4 cobre esse banco de dados também*.

   Depois **comece o traction-log.md** e preencha todo dia a partir dos registros do próprio app, nunca do que um parceiro disse por telefone. Cinco colunas: mercadinhos convidados, mercadinhos com pelo menos um fiado real, fiados abertos, fregueses que abriram o link do saldo, e mercadinhos que voltaram num outro dia sem ninguém pedir. A última é **uso recorrente**, o número em que um jurado mais confia, *porque qualquer um experimenta uma coisa uma vez, por educação*.

```text
traction-log.md, Fiado, semana 3 (contado dos registros do app)
dia  mercadinhos convidados  com fiado real  fiados abertos  fregueses abriram o link  voltaram sem pedir
16   4                       2               9               4                         n/a
18   6                       3               21              11                        1 de 2
21   7                       4               34              19                        3 de 4
```

   Quatro mercadinhos de sete usaram e **três voltaram** num outro dia sem lembrete do time. Essa linha alimenta o pitch v2, o slide de demo do deck e a resposta de validação de demanda do portal, e o log continua crescendo pela semana 4. A farmácia parou depois de dois fiados, então ela ganha uma ligação e uma linha no log de feedback, *porque a razão de um usuário real ter desistido é a frase mais útil que o mês produz*.

6. **Reescreva o pitch como v2**, a partir do que o recorte faz. O pitch v1 foi escrito no dia 7 a partir de cinco conversas, antes de existir qualquer código, *então só podia dizer o que o time esperava*. A regra da v2 é que **todo verbo da metade de produto** nomeia algo que a demo mostra na tela.

```text
pitch v1 (dia 7):   Uma dona de mercadinho que perde a caderneta perde quarenta
                    dívidas pequenas, então o Fiado guarda o fiado no celular dela,
                    mostra a cada freguês o mesmo saldo e liquida em stablecoin.
pitch v2 (dia 18):  Uma dona de mercadinho que perde a caderneta perde quarenta
                    dívidas pequenas, então o Fiado abre o fiado no celular dela,
                    mostra ao freguês o mesmo saldo, recebe parte dele em USDC e manda
                    o lembrete ao freguês por ela.
mudou:              "guarda o fiado" -> "abre o fiado" (a primeira tela que a demo
                    mostra; um formulário, não a transação); "liquida em
                    stablecoin" -> "recebe parte dele em USDC" (a única transação na
                    devnet, o que o freguês faz na tela); o lembrete
                    voltou, porque o recorte dispara um; "a cada freguês" -> "ao
                    freguês", um de cada vez na tela
```

A metade do problema **não se mexeu de novo**, e depois de duas rodadas de evidência e um build, *esse provavelmente é o problema certo*. O lembrete voltou porque o recorte dispara um, e só isso: se ele reduz atraso de pagamento **continua sendo a suposição não testada** do memorando de decisão, então a v2 afirma que o lembrete é enviado, que é o que a demo mostra.

7. **Faça a rodada 2 de feedback** com as mesmas três pessoas da rodada 1, *desta vez assistindo à demo em vez de ler uma frase*. O storyteller do time conduz, e o log recebe **o formato do dia 0**: o que cada pessoa não entendeu e o que mudou. As do Fiado:

```text
log de feedback, rodada 2, dia 18
1  Marcos, já entrou num        assistiu à demo; perguntou o que acontece quando o
   hackathon uma vez            freguês paga o fiado inteiro, se ele fecha
                                mudou: nada na frase; uma decisão no log de
                                que o fiado fecha em zero e reabre no
                                próximo lançamento; uma linha para o estado vazio
2  Lucia, a dona do mercadinho  assistiu à própria página na tela; perguntou para quem
                                o lembrete vai, para ela ou para o freguês
                                mudou: a frase. o rascunho dizia "manda o
                                lembrete"; a v2 diz "manda o lembrete ao freguês
                                por ela"
3  Jorge, um freguês que        assistiu à demo; perguntou como ele mesmo conferiria o
   tem fiado                    pagamento, sem confiar no app
                                mudou: nada na frase; o link do explorer
                                vai para a tela ao lado do total, então a
                                demo mostra a assinatura sem sair do app
```

Três entradas, uma mudança de uma palavra na frase, **duas mudanças no build**. A terceira entrada é **a que vale copiar**: a assinatura que você registrou na última lição estava num arquivo, e depois da rodada 2 está na tela, *a um toque do número que a dona olha*.

## Está pronto quando

O artefato é **o log narrativo** com seis coisas dentro, e está pronto quando:

- Todo item do checklist diz o que mudou, em palavras, com **o screenshot de depois**, com data, embaixo do de antes. Um item que já estava bom ganha isso escrito também, *com o porquê*.
- **Um vídeo de atualização de menos de 60 segundos** existe, o roteiro dele está no log, e uma pessoa que não viu o recorte *consegue repetir o que mudou esta semana*.
- **Notas de roast de dois roasters** e notas de segurança, verificação à mão incluída, estão no log.
- **O pitch v2** nomeia algo que o recorte faz de fato: cada verbo da metade de produto aponta para o passo da demo que o mostra, e um verbo que aponta para nada sai *até o recorte sustentar ele*.
- A rodada 2 de feedback tem **três entradas no formato do dia 0**.
- **traction-log.md** tem uma linha para cada dia desde que o primeiro parceiro começou, contada dos registros do app, com o uso recorrente numa coluna própria.

![A passada da semana 3 vai de um screenshot de antes, com data, passando pelo checklist, um vídeo de atualização, um roast, uma passada de segurança e o pitch v2, até uma rodada de feedback com três pessoas.](assets/v04-flowchart.webp)

## Fique atento

- **Screenshots tirados só no fim**. Um log escrito no último dia *só tem o último dia dentro*.
- **Roast feito pelo próprio time**. O time construiu o produto e já não consegue mais enxergar, *do mesmo jeito que você para de ver um erro de digitação numa página que já leu dez vezes*.
- **Pular a verificação de manejo de chaves** porque não tem programa. Sem programa não é sem chaves.
- **Contar um fiado que um parceiro prometeu** em vez de um que o app registrou. Promessa não é uso.

**Tempo de polimento é tempo de build**, então a passada é um checklist, não um redesign: um checklist tem quatro itens e um custo fixo, umas duas horas no total, enquanto um redesign acha um quinto item e um sexto, uma navegação nova e um sistema de cores, e ainda está aberto no dia 21, quando o vídeo de atualização da semana 3 deveria ser gravado. Esse time chega na semana 4 com uma tela mais bonita e sem vídeo. **A mesma troca** vive dentro do vídeo: o primeiro take geralmente está bom, o segundo geralmente melhor, e eu chuto que *o terceiro é onde a maioria dos times começa a perder a semana*, mas é só um palpite.

## O que fica

Polimento é um checklist com quatro itens e um custo fixo, não um redesign, e o log narrativo é escrito nos dias em que as coisas mudam. **A tração começa esta semana**, em mercadinhos reais com liquidação na devnet, *e o log de tração conta uso, nunca promessa*. Um estranho que assiste à sua atualização de 60 segundos deveria conseguir *dizer o que mudou esta semana*, e se ele descrevesse o produto em vez disso, o minuto foi um pitch, e **a diferença está na primeira frase**.

## Próxima lição: como alguém encontra isso

A semana 4 começa na próxima lição com a pergunta que todo jurado faz e a maioria dos times desvia: como alguém encontra isso, e por que Solana. A primeira coisa que você escreve são **os primeiros 100 usuários**, como um grupo com nome e um canal que chega até eles, *antes de qualquer argumento sobre a chain*. **Traga o log**, porque o parágrafo de por-que-Solana é escrito a partir do que o recorte faz, e o recorte está lá dentro, com a assinatura na tela.
