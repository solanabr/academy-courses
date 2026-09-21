# Os dois vídeos

Na lição passada você montou o deck-judge.md com sete títulos, escreveu as notas de 30 a 60 segundos por slide, colocou quatro objeções no objections.md, leu tudo para três pessoas na rodada 3 e marcou a frase da capa como final. Deixe as notas abertas. *Elas são a forma longa de tudo o que você grava hoje.*

## Por que isso importa

As demos de que eu me lembro começavam com uma pessoa e um problema: uma carteira esvaziada da noite para o dia, alguém no telefone com um golpista sem saber ainda, uma assinatura que não dava mais para desfazer, e depois a sensação de estar ali parado sem poder fazer nada. *Só depois disso vinha o produto.* Esses são **os primeiros vinte segundos** do seu vídeo de apresentação.

A única ideia que esta lição acrescenta é que você está fazendo **dois objetos diferentes**. A apresentação conta a história, a demo mostra o recorte, e o erro que a maioria dos times comete na semana 4 é tratar os dois como uma gravação só, cortada de dois jeitos. O tradeoff é a hora: um vídeo bonito de uma demo que trava na cena 6 perde para uma gravação de tela simples de uma que não trava, *então o tempo de produção vai para o roteiro e para a tomada, não para efeito*.

## Mão na massa: o vídeo de apresentação

1. **Abra um arquivo novo chamado presentation-script.md**, coloque a data na primeira linha e escreva os seis blocos de tempo antes de qualquer palavra entrar debaixo deles:

```text
presentation-script.md, seu projeto, temporada World's Fair da Colosseum, começado em 2026-09-06
0:00 a 0:20   o problema: uma pessoa, o que aconteceu com ela, o que ela sentiu; nenhum nome de produto ainda
0:20 a 0:50   o que o produto faz, nas palavras da frase da capa
0:50 a 1:20   o recorte na devnet, uma tela, a transação que um jurado pode abrir, a linha de tração
1:20 a 1:45   o que é novo, e por que Solana, três linhas dos slides 4 e 5
1:45 a 2:15   este time, e quem paga
2:15 a 2:45   o que ainda não foi construído, e o pedido
```

2. **Preencha só o primeiro bloco**. Pegue a nota que você escreveu para o slide do problema, corte a linha de dimensionamento, *comece pela pessoa em vez de pela caderneta* e termine no que ela sentiu quando a caderneta sumiu. Os vinte segundos carregam uma pessoa, algo que aconteceu com ela e o que ela sentiu depois, nessa ordem: **nenhum nome de produto**, nenhum número de mercado, nenhum time. Se o pacote de evidências da semana 1 tem uma frase real de uma daquelas conversas, use.

   Para o Fiado a pessoa é a dona do mercadinho que perdeu a caderneta: uns **quarenta nomes** com o total correndo ao lado de cada um, o dia em que ela molhou, e a dona perguntando para cada freguês quanto ele devia e aceitando a palavra dele. *A versão de vinte segundos termina no que é ficar atrás de um caixa sabendo que o dinheiro está espalhado na memória de quarenta pessoas.*

3. **Leia o primeiro bloco em voz alta** com o cronômetro rodando. Se passar de **vinte segundos**, corte uma frase e leia de novo até caber. Vinte segundos num ritmo normal de fala é menos palavra do que você imagina, e só o cronômetro decide. *Eu prefiro que você entregue uma versão de quinze segundos que pega do que uma de trinta segundos que é boa.*

4. **Preencha os outros blocos** a partir das notas do deck, cortadas mais ou menos pela metade. Na temporada World's Fair de 2026 a página da Colosseum pede 'a two-to-three-minute presentation video' (um vídeo de apresentação de dois a três minutos) e chama esse vídeo de 'one of the first resources judges review' (um dos primeiros recursos que os jurados analisam) (colosseum.com/hackathon, 2026-09-06). É por isso que este vídeo ganha o tratamento dos primeiros vinte segundos e a demo não. Sete slides na ponta curta, 30 segundos cada, já dão três minutos e meio, e o vídeo tem que ficar **entre 2:00 e 3:00**, *então cada nota perde mais ou menos metade das palavras e a ordem dos slides se mantém*. O do Fiado fica assim do segundo bloco em diante:

```text
presentation-script.md, Fiado, temporada World's Fair da Colosseum, 2026-10-05
0:00 a 0:20   o problema: a dona do mercadinho e a caderneta (seu para escrever)
0:20 a 0:50   O Fiado transforma essa caderneta num fiado que os dois lados leem. A dona
              abre um fiado para um freguês e adiciona o que ele levou, o freguês vê o mesmo
              saldo no próprio celular, e quando vence, sai um lembrete que a dona nunca
              precisou mandar. O freguês paga parte dele em stablecoin do mesmo celular,
              e os dois lados veem o saldo mudar.
0:50 a 1:20   Isso está rodando na devnet hoje. Esta é a tela do fiado, e embaixo dela está a
              assinatura de uma transação real que você pode abrir. Quatro mercadinhos já
              rodam fiados reais nele, e três voltaram sem ninguém pedir. Construímos o fiado,
              o pagamento parcial e o lembrete em duas semanas, e deixamos de fora de propósito
              o programa de fiado, os trilhos de fiat e o onboarding de carteira. Eles estão
              no último slide.
1:20 a 1:45   O app do banco não conhece o freguês. O mercadinho conhece. O Fiado mantém o
              crédito entre essas duas pessoas e coloca os pagamentos on-chain, cada um uma
              transferência de stablecoin de um celular com o nome do fiado nela, para que os
              dois possam ler o registro sem confiar em nós. Essa é a transação que você acabou de ver.
1:45 a 2:15   Somos dois. Um de nós já ficou atrás daquele caixa. Ninguém paga ainda. O mercadinho
              vai pagar, quando dez mercadinhos tiverem rodado um fiado por um mês. O freguês
              nunca paga o Fiado.
2:15 a 2:45   Ainda não construído: o programa de fiado, os trilhos de fiat, o onboarding de
              carteira, depois a expansão pela cooperativa, nessa ordem. Uma cooperativa já
              respondeu. Queremos a aceleradora, e uma apresentação a uma segunda cooperativa.
```

5. **Leia o roteiro inteiro a partir do 0:00** com o cronômetro rodando e escreva o total na primeira linha. O bloco da demo, em 0:50, diz que a transação é real e aponta para uma assinatura, e a apresentação vai até aí.

   O roteiro não diz rápido, pelo motivo que a lição passada deu. Não diz o número de mercado em voz alta, porque o número está no slide atrás da voz e o jurado que quiser vai pausar. E não diz nada em 0:50 que o vídeo da demo não vá mostrar, para que os dois vídeos **nunca se contradigam**. Se o total não fica entre 2:00 e 3:00 sem correria, *o problema está num bloco mais para a frente, não no primeiro*.

![O roteiro de apresentação do Fiado percorre seis blocos cronometrados, do problema sentido em 0:00 ao pedido em 2:45, cada um cortado de um slide do deck, e termina dentro da janela de dois a três minutos.](assets/v01-timeline.webp)

## Mão na massa: o vídeo da demo e a rodada 4

6. **Abra o demo-shots.md** e escreva os sete passos do seu roteiro de demo da semana 2, cada um com o tempo em que começa e as palavras ditas por cima, se houver. *Uma lista de cenas não decide nada de novo.* Ela só diz quanto tempo cada passo fica na tela e quais deles ganham uma frase. A mesma página da Colosseum pede 'a product-demo video of no more than three minutes' (um vídeo de demo do produto de no máximo três minutos) (colosseum.com/hackathon, 2026-09-06), e é para esse limite que **a lista de cenas é cortada**. A lista de cenas do Fiado segue os sete passos, um a um:

```text
demo-shots.md, Fiado, 2026-10-05, alvo 2:40, limite duro 3:00
cena 1   0:00   a lista de fiados do mercadinho no celular da dona, dois nomes nela          dito: este é o lado do mercadinho
cena 2   0:15   abre o fiado do freguês, adiciona a compra de hoje, 40, o link enviado        dito: o valor, nada mais
cena 3   0:40   o celular do freguês, os mesmos 40, sem instalar, sem login                  dito: mesmo fiado, outro lado
cena 4   1:00   ele toca em pagar parte, digita 15, a carteira pede a assinatura             dito: nada
cena 5   1:30   a transferência confirma, os dois celulares mostram 25 devidos, o link aparece dito: nada, deixa a tela falar
cena 6   1:55   a dona define uma data de vencimento para os 25                              dito: uma data, escolha dela
cena 7   2:15   o lembrete chega no celular do freguês, depois a assinatura                  dito: esse é o recorte inteiro
                aberta no explorer na devnet, 25 ainda devidos
```

7. **Marque as cenas 4 e 5 como mudas**, e mantenha mudas. Uma carteira pedindo assinatura e uma transferência confirmando com dois celulares concordando nos 25: um jurado que constrói na Solana lê essas telas mais rápido do que você consegue descrever, *e um jurado que não constrói está vendo acontecer*.

   A regra da narração é curta. Se não está na tela naquele segundo, **não fale**. O programa de fiado, os trilhos de fiat e a expansão pela cooperativa ficam no slide de próximos passos e no vídeo de apresentação. A cena 7 termina na assinatura no explorer com 25 ainda devidos, e o vídeo termina ali também, na prova. O alvo de **2:40** na primeira linha é *a folga que você deixa para uma confirmação lenta da devnet*.

![A demo do Fiado percorre sete cenas, da lista de fiados do mercadinho, passando pelo pagamento parcial mudo e a confirmação dele, até a data de vencimento, o lembrete e a transação aberta no explorer, e termina na prova dentro de três minutos.](assets/v02-flowchart.webp)

8. **Prepare a máquina limpa** antes de qualquer tomada. Isso quer dizer um perfil de navegador novo sem extensão nenhuma além da carteira, uma carteira criada só para a gravação, com apenas os fundos de devnet que a demo precisa, notificações desligadas, toda aba que não é a demo fechada, e o histórico do terminal limpo se um terminal aparece na tela. A assinatura que confirma na cena 5 é pública por design e *não tem problema mostrar*. Uma seed phrase, um arquivo de chave numa listagem de pasta, o saldo da carteira que você usa de verdade ou uma mensagem de alguém que pipoca em 1:30 **não podem aparecer na tela**.

9. **Grave a demo** só depois que o recorte rodar de ponta a ponta na máquina limpa duas vezes seguidas sem você encostar em nada, e depois assista uma vez com uma pergunta só na cabeça: o que está na tela que não deveria estar. **Grave a apresentação** no mesmo perfil, sobre o seu rosto ou sobre os slides, o que você conseguir fazer numa tomada só, sem parar, *porque uma tomada só, meio tosca, soa como uma pessoa, e seis tomadas coladas soam como um produto*. Leia do roteiro. Ninguém liga.

10. **Legende os dois vídeos** e confira as legendas com o roteiro e com a lista de cenas. Uma demo legendada com o som desligado ainda mostra o valor digitado e a transação confirmando, mas uma legenda com o valor errado sobre a cena 2 é um erro num lugar que o jurado está lendo. Depois confira a duração **no arquivo exportado**, o que você vai subir, e não no contador do app de gravação, *porque a exportação pega um segundo aqui e ali nas bordas*.

    Um detalhe, se tem um ensaio no seu calendário: um hackathon sazonal ou uma side track (a trilha regional) vai ter a própria página, com as próprias entregas e o próprio prazo, então **leia essa página no dia em que decidir entrar** e reaproveite as exportações que você já tem, *sem filmagem nova*.

![A apresentação dura de dois a três minutos e abre na pessoa, a demo dura três minutos ou menos e termina na transação confirmada, e nenhuma das duas mostra um segredo.](assets/v03-table.webp)

11. **Mostre os dois vídeos para três pessoas** que ainda não viram, os mesmos três tipos da rodada passada: uma que tem o problema, um builder e uma pessoa que não entende nada nem de um nem de outro. Toque a apresentação e pare em **0:20**. Peça para a pessoa dizer qual é o problema, em uma frase, e anote a frase que ela disse. Depois toque o resto e a demo, e faça **uma pergunta só**: o que você viu confirmar? Se a resposta é o pagamento, ou a transação, ou o fiado indo a zero, a demo mostrou. *Se a resposta é uma descrição do app, a demo narrou.*

12. **Escreva três entradas no log** com quem, o que disse, o que mudou, e a duração exportada do arquivo que cada pessoa assistiu, porque uma regravação muda isso. Se a pessoa diz algo perto de "isso aconteceu comigo", ou nomeia alguém com quem aconteceu, *os vinte segundos funcionam*. Se **duas das três** descrevem a categoria em vez de uma pessoa, reescreva os vinte segundos e regrave no mesmo dia. A rodada mantém o número e ganha mais de três entradas. A última entrada do log é a duração do vídeo que você vai subir, **com data**. A rodada do Fiado, com as mesmas três pessoas de todas as rodadas anteriores:

```text
log de feedback, rodada 4, Fiado, 2026-10-08
1  Lucia, a dona do mercadinho   em 0:20: "um mercadinho como o meu perdendo a caderneta"
                                 depois da demo: "os 15 caindo, e os 25 que sobraram"
                                 mudou: nada; apresentação 2:38, demo 2:41
2  Jorge, um freguês que         em 0:20: "mercadinhos que dão fiado e perdem a conta"
   tem fiado                     depois da demo: "o pagamento, e um link que eu mesmo podia abrir"
                                 mudou: nada; mesmas exportações
3  Marcos, entrou num            em 0:20: "apps de crédito"; depois da demo: "a transação"
   hackathon uma vez             mudou: os primeiros vinte segundos, porque a história da caderneta
                                 demorou demais para pegar com ele; a página molhada agora pega
                                 no segundo dez; regravado, apresentação 2:34, demo 2:41
```

Um de três errou, o que pela regra não obriga a regravar, e o storyteller regravou mesmo assim, *porque o erro veio com um motivo que ele podia atacar na mesma tarde*. As durações da última entrada, **2:34 e 2:41**, são os números que o capstone copia sem medir de novo. Com três agendas, mostre os vídeos para **uma pessoa hoje e duas amanhã**, e se os vinte segundos foram regravados no meio, a segunda pessoa assiste à versão nova e isso vai para o log como o que mudou.

![A rodada 4 para a apresentação em vinte segundos, pergunta ao espectador o problema e o que a demo confirmou, registra a duração exportada, e regrava se dois de três erram.](assets/v04-flowchart.webp)

## Está pronto quando

- **presentation-script.md** tem os seis blocos preenchidos, os primeiros vinte segundos abrem numa pessoa sem nome de produto, e a exportação da apresentação dura entre 2:00 e 3:00.
- **demo-shots.md** tem os seus sete passos cronometrados com as cenas mudas marcadas, e a exportação da demo dura 3:00 ou menos, com a confirmação da transação visível na tela antes do fim.
- Os dois vídeos foram gravados numa máquina limpa, legendados e **cronometrados no arquivo exportado**.
- O log tem **três entradas da rodada 4**, cada uma com quem, o que disse, o que mudou e a duração exportada, e as durações da última entrada têm data.

## Fique atento

- Uma demo que **narra uma funcionalidade** enquanto a tela mostra outra coisa é uma demo da sua voz, e um jurado assistindo com o som desligado, o que alguns fazem, vê um vídeo que nunca mostrou o que afirma.
- **Passar do limite** em dez segundos: o portal declara o limite e o brief assume que ele é aplicado, então uma demo de 3:08 é uma demo que talvez ninguém assista, e nenhum dez segundos de filmagem vale isso.
- Gravar a demo no seu laptop do dia a dia com **a sua carteira do dia a dia**: o vídeo é público e continua público depois da temporada, e uma seed phrase ou um saldo real no canto não tem como desfazer depois de subido.

## O que fica

Você está fazendo **dois objetos diferentes**. A apresentação conta a história e abre numa pessoa por vinte segundos antes de qualquer palavra de produto, para que um desconhecido diga: isso aconteceu comigo. A demo mostra o recorte e termina na transação confirmada, narrando só o que a tela sustenta. *Os dois são cronometrados no arquivo exportado, porque um vídeo acima do limite talvez ninguém assista.*

## A seguir: os formulários, o prazo, as regras

A próxima lição é a que perde mais times do que qualquer critério de julgamento, e não tem vídeo nenhum nela. Você vai submeter, e depois vai submeter de novo, porque uma submissão que existe cedo pode ser corrigida, e uma que existe às 23:50 não pode. A primeira ação dela é **um arquivo que lista cada entrega que você construiu** com o caminho e a data, antes de abrir o portal. *Traga as durações exportadas, elas vão no formulário.*
