# Finality vs. a pilha de cartões: o modelo mental de pagamentos

## Resumo

Na lição passada você decodificou uma transferência de USDC ao vivo e escaneou um QR code de pagamento gerado pelo seu próprio terminal. Esta aqui é a lição de modelo mental do curso: a última que é quase toda pensamento, embora ainda deixe dois scripts e um arquivo de política na sua pasta. Três movimentos. Primeiro a gente põe o seu vocabulário de cartão lado a lado com os commitment levels da Solana, um a um, e extrai a regra de decisão que você vai usar em todo pagamento: qual nível você espera, dado quanto o pagamento vale. Segundo a gente desmonta o que um pagamento custa aqui, porque o custo difere das taxas de cartão em formato e não só em tamanho, e é o formato que vira a economia das vendas de ticket pequeno e do float de crédito que todo lojista brasileiro financia caladinho. Terceiro a gente encara de frente a assimetria: transferências liquidadas não podem ser revertidas, por ninguém, nunca, o que apaga a fraude de chargeback e a rede de proteção do seu cliente no mesmo golpe. Fechamos com a pergunta de que todo modelo precisa: quando estes trilhos são a escolha errada?

Como o trabalho se divide hoje: leia o modelo com as mãos no colo, digite junto comigo no lab, e o desafio do final você faz sozinho. Cada lição te entrega um pouco mais do trabalho que a anterior, e a partir do próximo módulo você está construindo.

## O modelo mental dos trilhos do dinheiro

Você viu dinheiro se mover na lição passada. O que você ainda não tem é o modelo que explica por que ele se move do jeito que se move, e esse modelo é a diferença entre um checkout que funciona sem alarde e um que despacha US$ 2,000 de estoque contra um pagamento que nunca existiu.

Então, antes de qualquer teoria, rode isto. O mesmo RPC público da lição passada, sem carteira, sem chaves:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[{"commitment":"processed"}]}'
```

Você recebe de volta um número de **slot** de nove dígitos, e essa palavra precisa da definição dela antes que o número signifique qualquer coisa. Um slot é uma janela de tempo fixa, hoje de 300 milissegundos, na qual um validador designado pode produzir um bloco. Slots avançam para sempre, numerados desde o primeiro dia da rede, então um número de slot é uma leitura de relógio e a diferença entre dois deles é uma duração que você pode multiplicar. É esse o truque inteiro que você está prestes a fazer.

Agora rode o comando de novo com `"commitment":"finalized"` no lugar de `"processed"` e subtraia os dois resultados. A diferença fica normalmente em torno de trinta slots, o que a 300ms por slot dá uns nove segundos. Segure esse número. No fim desta lição você vai saber o que essa diferença compra, por que o seu PSP nunca te mostrou nada parecido, e por que um dos seus melhores instintos de cartão, "o banco sempre pode estornar", está prestes a virar a suposição mais cara do seu código.

O seu PSP te ensinou três palavras: autorização, captura, liquidação. Estes trilhos também têm três palavras: `processed`, `confirmed`, `finalized`. O mapeamento entre os dois vocabulários é real e vai te levar longe. O lugar onde ele quebra é onde o dinheiro está.

### O seu vocabulário de PSP, traduzido

Comece do que você já sabe, porque é um apoio genuinamente bom. Nos trilhos de cartão um pagamento é uma promessa em etapas. A autorização diz que os fundos existem e estão reservados. A captura diz que você pretende pegá-los. A liquidação, dias depois, diz que o dinheiro de fato se moveu entre bancos. Três etapas, confiança crescente, e o seu código de integração se guia por qual etapa você está.

A Solana também tem uma escada de confiança crescente. Só que ela é medida em outra unidade: o quanto a rede está certa de que o bloco que contém o seu pagamento vai sobreviver.

Três palavras carregam essa escada, então pegue-as agora. Um **validador** é uma das máquinas que rodam a rede. **Stake** é o SOL que aquele validador travou como pele em jogo, e a influência de um validador é proporcional a ele, então "o stake da rede" é o jeito certo de contar cabeças aqui, em vez de contar máquinas. Um **voto** é uma transação que um validador publica dizendo "eu vi este bloco e estou construindo em cima dele", e votos são o que transforma a opinião de um nó na opinião da rede.

- **`processed`**: algum nó executou a transação e a colocou em um bloco. Esse bloco ainda pode ser descartado se a rede discordar por um instante sobre a ponta da cadeia, o que se chama fork. Pense nisso como ver o cartão entrar na maquininha. Algo aconteceu. Nada está prometido.
- **`confirmed`**: validadores que detêm a supermaioria do stake da rede votaram no bloco. Na prática esse é o nível cavalo de batalha, e reversão nesse ponto deixa de ser um evento realista.
- **`finalized`**: blocos suficientes já foram construídos e votados em cima do seu bloco para que nenhum fork consiga deslocá-lo. É isso que preenche a diferença de nove segundos que você mediu: não é espera, é empilhamento. Voto após voto aterrissa em slots posteriores, e cada um eleva o custo de desfazer o seu até ficar fora do alcance de qualquer coalizão. Não existe nível mais profundo. Isto é liquidação, só que chega em segundos em vez de dias.

![Uma tabela de tradução pareando autorização com processed, captura com confirmed e liquidação com finalized, com nota de rodapé de que a liquidação de cartão é reversível por chargeback e finalized não é.](assets/v01-comparison.webp)

Então contra qual nível você constrói? A orientação oficial é qualitativa, e vale citar pelo formato: use `confirmed` para a maioria dos pagamentos, espere por `finalized` quando o pagamento for de valor alto ou sensível a compliance, e trate `processed` como só de UI, porque uma transação processed pode ser descartada num fork. Essa única frase é o seu motor de política. Todo o resto é ajuste fino.

Agora os números, com cuidado. Quanto tempo leva cada degrau? A experiência do ecossistema coloca `confirmed` em mais ou menos um a dois segundos, e os números que ela costumava citar para `finalized`, mais ou menos dez a treze segundos, foram colhidos com tempos de slot mais antigos. Quero ser preciso sobre o que esses números são: estimativas de gente que observa a rede, não números impressos na documentação oficial. A documentação te dá a tabela qualitativa e para por aí. E qualquer número que você derive de matemática de slot tem que usar o tempo-alvo de slot atual, 300ms, não os 400ms ou 350ms que você vai achar em artigos mais velhos, porque esse número está sendo cortado em etapas: 400ms caiu para 350ms em 2026-08-21, 350ms caiu para 300ms na epoch 1024 em 2026-08-28, e mais dois cortes, para 250ms e depois 200ms, já estão represados no código do validador e rodando na devnet. Um tutorial que deriva de 400ms hoje superestima todo número de relógio em um terço, e um que cravar os 300ms de hoje vai estar velho no dia em que a próxima comporta abrir. Alvo, repare, não medição: o tempo de slot real medido no relógio fica um pouco acima do alvo, então todo número que você deriva de 300 é um piso, não uma promessa. O seu experimento com curl da abertura já te deu a medição: uns trinta slots entre `processed` e `finalized`, e trinta slots no alvo de 300ms dá uns nove segundos, um pouco mais nos tempos de slot medidos. É esse o método inteiro, e os cortes em etapas são exatamente o motivo de o método valer mais do que qualquer número deste parágrafo: derive, nunca memorize, porque o número do mês que vem sai dos mesmos dois comandos de shell e uma multiplicação. Vale reparar no que você não precisou fazer: nos trilhos de cartão, o prazo de liquidação é uma coisa que o seu adquirente te conta num PDF, e aqui você derivou sozinho.

![Um pagamento vai de submetido a processed em um slot de 300ms, a confirmed em um a dois segundos, e então a finalized irreversível em cerca de nove segundos.](assets/v02-flowchart.webp)

Aqui está a derivação que faz da regra de política sua, e não minha. Por que não esperar sempre por `finalized` e ficar seguro? Porque nove segundos e pouco são uma eternidade num terminal de ponto de venda, e para um café de US$ 3 aquilo contra o que você está se segurando, um fork derrubando um bloco confirmed, não é um risco que valha nove segundos a mais do tempo da fila. Por que não usar sempre `confirmed` e ser rápido? Porque "não é um evento realista" e "impossível" são afirmações de engenharia diferentes, e quando o pagamento é de US$ 2,000 você compra a impossível. A escada existe para você precificar a espera contra o ticket. O seu PSP tomou essa decisão por você e te cobrou pelo privilégio. Aqui a decisão está exposta, e é sua. É essa a forma recorrente do curso inteiro: os trilhos te entregam o botão cru e você constrói a política.

Um reflexo para guardar, um para jogar fora. Guarde o instinto de confiança em etapas; ele mapeia lindamente. Jogue fora o instinto que diz que as etapas são problema de outra pessoa. Não existe adquirente rio abaixo de você rechecando nada.

### O que um pagamento custa aqui

Hora da anatomia de custo, e é aqui que o modelo deixa de ser um exercício de tradução e vira um business case.

Nos trilhos de cartão você orça uma porcentagem mais um corte fixo, algo como 2.9% mais 30 centavos num processador típico. A porcentagem é a parte estrutural: a rede fica com uma fatia do valor movimentado, então uma venda maior custa mais para se mover, e uma venda minúscula mal sobrevive às próprias taxas. Toda decisão de preço que você já tomou rio abaixo disso, pedido mínimo, sobretaxa, plaquinha de "cartão só acima de US$ 10" no balcão, existe porque a taxa é uma porcentagem.

A taxa base aqui é de 5000 lamports por signature, fixos, e isso precisa ser desempacotado antes de poder ser comparado com qualquer coisa. **SOL** é o token nativo da própria Solana, aquele em que a rede cobra as taxas dela; não é uma stablecoin, o preço dele flutua, e é uma coisa completamente separada do USDC em que os seus clientes te pagam. Um **lamport** é a menor unidade de SOL, um bilionésimo de um. Então 5000 lamports são 0.000005 SOL, e transformar isso em dinheiro é uma multiplicação pelo que o SOL estiver valendo na hora em que você rodar.

Faça a multiplicação você mesmo em vez de confiar num número cravado num curso: a US$ 150 por SOL a taxa é US$ 0.00075, a US$ 300 é US$ 0.0015. Em todo preço pelo qual o SOL já negociou, uma signature custa uma pequena fração de um centavo. Essa faixa é a forma defensável da afirmação, e é a forma que você deve citar numa reunião, porque o número em dólar se move com um mercado e o número em lamports não.

Mas barato não é a propriedade interessante. Fixo é a propriedade interessante. A rede está te cobrando por um slot de trabalho, não tirando uma porcentagem do valor movimentado, então uma transferência de US$ 2,000 e uma de US$ 0.50 custam para a rede a mesma fração de centavo para liquidar.

Rode um exemplo de número redondo, do tipo que você vai refazer de cabeça para toda decisão de produto deste curso. Um cliente compra um disco de US$ 12 da Wavelength Records, a loja que estamos construindo. Trilhos de cartão: 2.9% de US$ 12 dá uns 35 centavos, mais o corte fixo de 30 centavos, digamos 65 centavos, ou 5.4% da venda indo embora. Estes trilhos: uma fração de centavo, que nesse tamanho de ticket é um erro de arredondamento sobre um erro de arredondamento, umas três ordens de grandeza a menos. Agora encolha o ticket. Uma venda de US$ 0.50 nos trilhos de cartão custa 31 centavos para processar, então a taxa come 62 por cento da venda, e abaixo de uns 31 centavos a taxa excede o ticket inteiro e a venda para de ser possível. É por isso que ninguém vende coisas de 50 centavos uma a uma na internet. Aqui a taxa não está nem aí para o ticket ter encolhido. Categorias inteiras de negócio, micropagamentos, preço por artigo, APIs cobradas por chamada, param de ser piada e viram linha de orçamento.

![Um gráfico de barras em escala logarítmica das taxas de cartão subindo de 31 centavos a 58 dólares em quatro tamanhos de ticket, enquanto a taxa base da Solana fica fixa e abaixo de um centavo.](assets/v03-chart.webp)

Duas notas de rodapé antes do zoom para trás, porque um modelo de custo com linhas escondidas é pior que modelo nenhum. Primeiro, 5000 lamports é a taxa base; momentos de congestionamento podem somar uma pequena taxa de prioridade em cima, e você vai conhecer esse botão mais adiante no curso. Segundo, existe um custo de uma vez só que a taxa por pagamento esconde: contas de token custam rent para serem criadas. Pense no rent como o aluguel da maquininha na sua analogia de cartão, um custo fixo de montar o caixa e não um corte de cada venda. O que uma conta de token é de fato, e a linha exata de rent, chega no próximo módulo, onde você vai criar uma com as próprias mãos. Por hoje basta que o modelo tenha um espaço para isso: custo por pagamento perto de zero, custo de uma vez só para montar a conta pequeno mas real.

Agora dê um zoom para trás, porque a taxa fixa é o motor por trás de um número que você já conheceu. A lição passada te deu o estoque: cerca de US$ 15.87 bilhões de stablecoins paradas na Solana. Esta lição consegue te dar o fluxo. Em 2024, o volume de transferência de stablecoins entre blockchains foi reportado em US$ 27.6 trilhões, um número que o guia de pagamentos com stablecoin da Helius diz ter superado Visa e Mastercard somadas (afirmação datada deles, consultada em 2026-08-21). Na Solana especificamente, o US$ 1 trilhão de volume de 2025 da lição passada é um número de ano inteiro, não um contador ao vivo. Divida esse fluxo por esse estoque, e sim, isto é aritmética grosseira misturando um fluxo de 2025 com uma foto de 2026, mas cada dólar em circulação girou da ordem de sessenta vezes num ano. Isto não é dinheiro parado. É dinheiro fazendo o que dinheiro faz quando movê-lo não custa nada: ele se move.

E a taxa fixa decide quem consegue participar desse movimento. A versão mais forte desse argumento está do lado do caixa de toda loja brasileira que você já integrou. A linha visível primeiro: na média do mercado, o MDR fica em 2.13% no crédito e 1.08% no débito (cálculo meu a partir da série DESCONTODA da API do BCB, 1T 2026). O segundo eixo é o que esta lição ainda não precificou: tempo. Uma venda no crédito à vista liquida para o lojista em D+30, e uma venda parcelada em 12x pinga mês a mês. O cliente saiu com a mercadoria; a sua receita está no carnê.

Os adquirentes precificam essa espera em público, então você não precisa adivinhar quanto ela vale. A tabela da InfinitePay para a faixa de volume mais alta, lojistas acima de R$ 80,000 por mês (consultada em 2026-09-01; a própria página não tem data), oferece crédito à vista a 1.62% no prazo de liquidação e 2.69% em D+1. A linha de 12x marca 2.25% por parcela contra 8.99% pela venda inteira em D+1 — e leia "por parcela" do jeito que a tabela quer dizer: 2.25% tirados de cada parcela conforme ela liquida, o que totaliza 2.25% da venda, não 2.25% empilhados doze vezes. Os 8.99% compram o dinheiro da mesma venda em D+1 em vez de ele pingar ao longo de um ano — dinheiro que de outro jeito chegaria até você em seis meses e meio, na média. Os dois spreads dão cerca de 1.05% a.m. Isso não é taxa de processamento. É juro, e o principal é a sua própria receita: antecipação é a indústria te vendendo o seu dinheiro de volta, adiantado, a uma taxa mensal corrente.

![Um gráfico de barras agrupadas das taxas publicadas pela InfinitePay na faixa mais alta: crédito à vista a 1.62 por cento no prazo padrão contra 2.69 por cento em D+1, uma venda em 12x a 2.25 por cento por parcela contra 8.99 por cento em D+1, e uma faixa de conclusão precificando os dois spreads em cerca de 1.05 por cento ao mês.](assets/v04-chart.webp)

E quanto menor você for, pior o negócio. Aqueles números arrumadinhos pertencem à faixa de R$ 80,000 por mês; desça a tabela e a mesma venda em 12x custa 20.39% abaixo de R$ 3,000 de volume por mês e 11.51% acima de R$ 30,000 nas taxas publicadas da Ton, ou 22.59% contra 13.69% nas do Mercado Pago. Uma taxa fixa de 5000 lamports não sabe qual é o seu volume mensal, e é essa indiferença que é o ponto.

Nem a espera precificada é um produto de nicho para lojas sem caixa. A Núclea, a câmara onde os recebíveis de cartão são registrados, reporta R$ 614.9 bilhões de volume de antecipação contra R$ 428.6 bilhões na leitura anterior, com 33.4% mais estabelecimentos antecipando (números verificados para este curso em 2026-09-01). Se financiar com os próprios recebíveis é a norma operacional, e está crescendo nos dois eixos. Ponha os trilhos desta lição do lado dessa máquina: liquidado quer dizer gastável, em segundos, por uma fração fixa de centavo, em qualquer faixa de volume, porque o recebível que aquela indústria inteira monetiza nunca existe. Aqui o float é apagado, não descontado.

Antes de levar esse argumento para perto de um líder de finanças brasileiro, diga a próxima frase você mesmo, antes que ele diga para você. O PIX não custa nada para o cliente e custa ou nada ou uma fração de por cento para o lojista — o BCB só obriga gratuidade para pessoas físicas (Resolução BCB 19/2020), então os PSPs podem cobrar de CNPJ e cobram: a Stone publicou 0%, o Mercado Pago 0.49% para um vendedor novo, e a Ton 0.99% depois da janela de teste, quando isso foi lido nas próprias páginas de preço dos provedores (consultadas em 2026-09-01; preço de PSP é promocional e se mexe, então releia antes de citar qualquer coisa disso) — é instantâneo, e para uma venda doméstica simples em reais ele ganha dos trilhos de cartão e ganha do que este curso constrói. Essa é a linha de base honesta dos pagamentos brasileiros, e este curso não vai fingir o contrário. Se a venda na sua frente é uma que o PIX atende, um cliente pagando agora em reais, use o PIX, e não deixe ninguém te vender uma blockchain para isso. Um curso de pagamentos que não consegue dizer essa frase em voz alta é um anúncio.

Então por que o argumento do float sobrevive a essa concessão? Olhe para qual trilho o PIX de fato venceu. A ABECS colocou o volume do ano fechado de 2025 em R$ 3.1 trilhões no crédito contra R$ 1 trilhão no débito, com o parcelado sem juros (PSJ) respondendo por 42.6% desse total de crédito (release Balanço 2025 da ABECS, publicado em 2026-02-11). O PIX ganhou a briga do débito; a venda instantânea de pagar agora já pertence a ele. O que ele não substituiu foi o crédito, porque uma venda parcelada é o cliente comprando o atraso em si, e o PIX não vende atraso. Nada do que este curso constrói vende também, então seja preciso sobre o residual. Vire esses 42.6% do avesso: o resto do livro de crédito se divide entre à vista e parcelado com juros, e é a fatia do à vista — vendas de cobrança única sentadas no trilho de crédito por hábito, por pontos, ou por padrão de checkout, enquanto o lojista banca um float de D+30 que ninguém pediu — que um trilho de push consegue disputar sem fingir que vende parcelamento. Então a comparação não é o PIX, cuja briga acabou. O que resta é a pilha do crédito, e agora você sabe ler o preço cheio dela: a porcentagem visível, mais o float, em termos que pioram conforme você encolhe. Trilhos baratos não deixam só os pagamentos existentes mais baratos. Eles mudam quais custos de um lojista são leis da natureza e quais são linhas de orçamento que você pode recusar.

### A assimetria: ninguém consegue estornar

Aqui está a parte do modelo contra a qual os seus instintos de cartão mais vão brigar, então vamos derivá-la em vez de afirmá-la.

Nos trilhos de cartão um pagamento é um pull. O cliente te entrega credenciais e você, através do seu PSP, puxa os fundos da conta dele. Como o sistema é construído em cima de puxar, ele precisa de um desfazer: o cliente tem que conseguir contestar um pull que não autorizou, então a bandeira mantém um caminho institucional de reversão, emissor para bandeira para adquirente para você, e um chargeback consegue voltar por ele semanas depois da liquidação. Você já sentiu o custo dessa maquinaria: retenção por fraude, reserva rotativa, a taxa de chargeback de US$ 15 numa venda de US$ 12, fundos que você "tem" e não pode tocar por dias. O caminho de reversão nunca foi parafusado depois; é o preço que a pilha inteira paga por dinheiro baseado em pull.

Nestes trilhos um pagamento é um push. O cliente assina uma transação movendo os fundos dele para você; ninguém, inclusive você, jamais consegue enfiar a mão na conta dele. E uma vez que esse push está finalized, ele é final no sentido da física do livro-razão: não existe caminho institucional de reversão, nenhum operador de rede com um botão de desfazer, nenhum banco para ligar. Toda transferência liquidada é permanente, ponto final.

Agora derive as consequências em vez de parar no slogan, porque elas cortam dos dois lados.

Do lado do lojista, os ganhos são reais: a fraude de chargeback não é reduzida, ela é estruturalmente impossível; não existe reserva rotativa porque não há nada contra o que reservar; liquidado quer dizer gastável, em segundos. A indústria inteira de gestão de disputas que o seu PSP te cobra não tem nada para gerir.

![Trilhos de cartão puxam fundos com uma seta tracejada de chargeback correndo para trás por semanas; a Solana empurra fundos sem seta para trás, então um reembolso é um pagamento novo e separado.](assets/v05-diagram.webp)

Do lado do cliente, a mesma propriedade se lê de um jeito bem diferente. O desfazer em que o seu cliente foi treinado a confiar a vida adulta inteira acabou. Se ele cair num golpe, não tem processo de contestação esperando. Se ele errar o endereço com o dedo gordo, a rede não vai ajudá-lo; um envio equivocado não tem banco para ligar. E a sua realidade operacional muda junto: reembolsos ainda têm que existir, os clientes vão exigi-los com razão, mas a rede não te dá nada. Um reembolso vira um pagamento de push voluntário na direção contrária, que você projeta, financia, autoriza e constrói. Vá procurar a página de reembolsos na documentação oficial de pagamentos em solana.com/docs/payments. Não existe: a barra lateral vai de transfers até Solana Pay e para, e nenhuma página em lugar nenhum embaixo dela descreve reverter um pagamento liquidado. Essa ausência é a assimetria enunciada como documentação: a lógica de reversão aqui é uma funcionalidade de aplicação e não uma funcionalidade dos trilhos, e no módulo 4 você vai construí-la, casamento de pedidos, reembolsos parciais, tudo.

Vou confessar o reflexo que esta lição está de fato mirando, porque eu já fui culpado dele: tratar finality como um detalhe a anotar em vez de uma arquitetura para projetar em torno dela, arquivando mentalmente "sem chargebacks" como puro ganho durante semanas até cair a ficha de que eu também tinha apagado em silêncio toda a rede de proteção dos meus clientes e jogado a reconstrução no meu próprio roadmap. A assimetria do sem-chargeback não é um desconto. É uma transferência de responsabilidade, da máquina de disputas da bandeira para o seu código. Precifique direito e é uma troca que muitos negócios deveriam aceitar. Precifique como almoço grátis e vira uma fila de tickets de suporte com o seu nome nela.

### Quando estes trilhos são a escolha errada

Todo modelo mental que este curso te entrega vem com a mesma seção final, e é ela que te mantém confiável numa revisão de design.

Se o negócio que você está integrando depende de direitos de contestação garantidos por banco, de reversões iniciadas pelo comprador, ou da proteção de chargeback da bandeira, então a finality sem chargeback remove exatamente aquilo de que se depende. Um vertical de consumo com muita fraude, onde os compradores esperam reversão sob demanda, é um encaixe ruim, não porque a tecnologia falha, mas porque a expectativa central do cliente é justamente a propriedade que estes trilhos apagam por projeto. A mesma irreversibilidade que derruba o seu custo de fraude para perto de zero também remove o desfazer institucional que o seu comprador pode acreditar, com razão, que lhe é devido. Você pode reconstruir maquinaria de confiança na camada de aplicação, e módulos mais adiante fazem isso, mas você deve entrar sabendo que está reconstruindo algo que os trilhos de cartão te davam de graça.

Onde estes trilhos se encaixam melhor? Fluxos de taxa baixa e liquidação instantânea: as vendas em que o intercâmbio baseado em porcentagem come o ticket, em que um cronograma de recebíveis fica entre a receita e a folha de pagamento, e, mais para o fim da lista, vendas que cruzam fronteira, onde o banco correspondente ainda come dias. Onde eles se encaixam pior? Onde quer que a reversibilidade seja o produto. Se o que o seu cliente está de fato comprando é a possibilidade de mudar de ideia depois que o dinheiro se moveu, venda trilhos de cartão para ele. Saber quando não usar a sua ferramenta nova é o modelo funcionando.

![Uma matriz de duas colunas: encaixe forte para vendas de ticket pequeno, fluxos que precisam escapar do float de liquidação e, por último, pagamentos que cruzam fronteira; encaixe fraco para vendas com muita fraude, negócios que vendem reversibilidade e regimes de compliance que exigem um desfazer institucional.](assets/v06-comparison.webp)

É esse o modelo inteiro: uma escada em que você escolhe um degrau, uma taxa cujo formato vira a sua economia unitária, e uma assimetria que troca custo de fraude por responsabilidade. Hora de fazê-lo produzir decisões.

## Lab: sonde a escada e precifique um pagamento

O lab formaliza o seu experimento de abertura num pequeno utilitário, lê uma taxa real da mainnet, e termina com você escrevendo o primeiro rascunho de uma política de confirmação de verdade. Digite junto. Você precisa do Node 24 que configurou na lição passada; os dois scripts aqui usam o `fetch` embutido do Node, então não tem nada para instalar.

1. Trabalhe na pasta `wavelength-rails` que você fez na lição passada, e crie um arquivo chamado `commitment-ladder.mjs`:

   ```js
   // commitment-ladder.mjs
   // Probes mainnet's three commitment levels and prints how far each trails
   // the tip, in slots and in wall-clock time. Node 24 (built-in fetch), no deps.
   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";

   // Mainnet target slot time in ms. 300ms since epoch 1024 (2026-08-28),
   // the second SIMD-0525 staged cut after 350ms took force on 2026-08-21.
   // Two more cuts, 250ms and then 200ms, are gated in the validator code
   // and live on devnet, so re-check this constant before trusting it.
   const SLOT_TIME_MS = 300;

   async function getSlot(commitment) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({
         jsonrpc: "2.0",
         id: 1,
         method: "getSlot",
         params: [{ commitment }],
       }),
     });
     const json = await res.json();
     if (json.error) throw new Error(`RPC error: ${json.error.message}`);
     return json.result;
   }

   const levels = ["processed", "confirmed", "finalized"];
   const slots = {};
   for (const level of levels) {
     slots[level] = await getSlot(level);
   }

   console.log("commitment   slot          behind tip   approx wall-clock");
   for (const level of levels) {
     const behind = slots.processed - slots[level];
     const secs = ((behind * SLOT_TIME_MS) / 1000).toFixed(1);
     console.log(
       `${level.padEnd(12)} ${String(slots[level]).padEnd(13)} ${String(behind).padEnd(12)} ~${secs}s`
     );
   }
   ```

   Resultado esperado: um arquivo salvo do lado do `watch-a-dollar.ts` da lição passada. Nada rodou ainda.

2. Rode:

   ```bash
   node commitment-ladder.mjs
   ```

   Checkpoint: três linhas, com `processed` a zero atrás, `confirmed` de zero a um punhado de slots atrás, e `finalized` a uns 25 a 40 slots atrás, o que a 300ms por slot dá uns oito a doze segundos, a sua vizinhança dos nove segundos. Rode três ou quatro vezes. Os números de slot marcham para frente; as diferenças ficam mais ou menos no lugar. Você está vendo a escada respirar. (As três chamadas correm contra a ponta da cadeia entre as requisições, então uma diferença pode oscilar uns poucos slots de rodada para rodada, e `confirmed` pode até imprimir um número negativo pequeno quando a ponta avançou entre a sua primeira e a sua segunda chamada. Esse jitter é a medição, não um bug.)

3. Agora leia uma taxa real da cadeia. Crie `fee-anatomy.mjs`. Uma ressalva sobre o que ele pega: ele pega a transação limpa mais recente que tocou a conta do mint do USDC, e nem tudo que toca um mint é um pagamento. Você pode cair numa transferência, ou na contabilidade de algum protocolo. A anatomia da taxa é idêntica de qualquer jeito, que é o ponto, mas não narre a saída para ninguém como "um cliente pagou isto".

   ```js
   // fee-anatomy.mjs
   // Finds a recent finalized transaction touching the USDC mint and prints
   // its fee, split into base fee and priority tip. Node 24, no deps.
   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
   const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
   const BASE_FEE_PER_SIG = 5000; // lamports, flat, per signature

   async function rpc(method, params) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const json = await res.json();
     if (json.error) throw new Error(`RPC error: ${json.error.message}`);
     return json.result;
   }

   const sigs = await rpc("getSignaturesForAddress", [
     USDC_MINT,
     { limit: 10, commitment: "finalized" },
   ]);
   const sig = sigs.find((s) => s.err === null)?.signature;
   if (!sig) throw new Error("No clean signature in the last 10; run it again.");

   const tx = await rpc("getTransaction", [
     sig,
     { maxSupportedTransactionVersion: 1, commitment: "finalized", encoding: "json" },
   ]);

   const fee = tx.meta.fee;
   const sigCount = tx.transaction.signatures.length;
   const base = BASE_FEE_PER_SIG * sigCount;
   console.log(`signature:    ${sig}`);
   console.log(`signatures:   ${sigCount}`);
   console.log(`total fee:    ${fee} lamports`);
   console.log(`base fee:     ${base} lamports (5000 per signature)`);
   console.log(`priority tip: ${fee - base} lamports (anything above base)`);
   ```

   Resultado esperado: um segundo arquivo salvo. Dois scripts agora, um para a escada e um para a taxa.

4. Rode `node fee-anatomy.mjs`. Checkpoint: uma taxa total em lamports, decomposta. Com uma signature a linha base marca 5000; o que estiver acima dela é uma priority tip que o remetente escolheu adicionar. **Uma priority tip de 0 é um resultado correto e extremamente comum**, não um script quebrado: a maioria dos remetentes não adiciona nada quando a rede está calma, então `priority tip: 0 lamports` quer dizer que o remetente pagou o piso e nada mais. Converta o total usando a aritmética da seção de custo: divida os lamports por um bilhão para chegar em SOL, depois multiplique pelo preço do SOL hoje. Até uma gorjeta generosa deixa a coisa toda numa fração de centavo. A mesma ressalva de RPC público da lição passada vale: o endpoint gratuito da mainnet vai te limitar por taxa se você martelar nele, então se você vir um HTTP 429 ou um erro de RPC, espere uns segundos e rode de novo.

5. Estenda a tabela Rosetta que você construiu no desafio da lição passada. Abra: você já mapeou signature, sender, amount, mint, memo e settledAt para os equivalentes de trilho de cartão, com uma coluna de vazamento para cada. Esta lição introduziu mais três coisas que merecem um nome de trilho de cartão, então acrescente três linhas e preencha a coluna de vazamento você mesmo:

   | Coisa on-chain | Análogo de trilho de cartão | Onde a analogia vaza |
   |---|---|---|
   | taxa base (5000 lamports, fixa) | intercâmbio mais taxa do processador | ? |
   | rent de conta de token (próximo módulo) | aluguel da maquininha, um custo de montagem de uma vez só | ? |
   | `finalized` | fundos liquidados | ? |

   Resultado esperado: a sua tabela da lição passada, agora com nove linhas, com as três células novas de vazamento escritas por você. Se a coluna de vazamento da linha `finalized` não disser algo sobre estornos, o ponto desta lição inteira ainda está na sua frente.

6. Comece o artefato que esta lição de fato produz: crie `commitment-policy.md` com quatro títulos, `High-value orders`, `Everyday payments`, `Micro-payments` e `UI display`, e embaixo de cada um escreva uma frase nomeando o commitment level que você vai esperar e uma frase defendendo-a. Deixe rascunhado. O desafio preenche dois desses títulos direito e as lições de payment-ops do módulo 4 transformam o arquivo em código rodando.

   Resultado esperado: um arquivo markdown de quatro títulos com oito frases rascunhadas dentro. É a única coisa que você leva desta lição, então guarde onde você vai achar.

![Linhas do tempo paralelas de uma venda de 2,000 dólares: a Solana chega à finality irreversível em cerca de nove segundos, enquanto os trilhos de cartão liquidam em dias e deixam a janela de chargeback aberta por semanas.](assets/v07-timeline.webp)

## Challenge

Sem digitar junto aqui; este é seu. Dois pagamentos chegam na Wavelength Records.

Primeiro: um pedido de US$ 2,000 sob `High-value orders`, um colecionador comprando uma parede de primeiras prensagens, com envio hoje. Segundo: uma cobrança de US$ 0.40 sob `Micro-payments`, uma chamada à API de preço de prensagem que a Wavelength cota para outras gravadoras, cobrada por consulta. Esses dois títulos são os que você preenche; deixe `Everyday payments` e `UI display` com as frases rascunhadas que você já escreveu. Para cada um dos dois, escreva: o commitment level em que você libera a mercadoria, o risco concreto que você está aceitando por não esperar mais, e o custo concreto que você está se recusando a pagar por não esperar, nessa ordem. Use a regra qualitativa desta lição mais os seus próprios números medidos da escada, e lembre do que os tempos são: estimativas que você verificou, não evangelho que você herdou.

Depois feche o arquivo com duas frases que não têm nada a ver com commitment levels. Nomeie o único reflexo de cartão das suas integrações existentes que não porta para estes trilhos, e nomeie o que a ausência dele te obriga a construir mais adiante neste curso. Se as suas duas frases mencionarem estornos e reembolsos, você tem o modelo.

É essa a decisão produzida em que esta lição trava: uma política que você consegue defender em voz alta, não uma página que você leu concordando com a cabeça. Se a sua defesa do caso de US$ 2,000 não mencionar forks, releia a seção da escada; se a sua defesa da chamada de API de US$ 0.40 não mencionar formato de taxa, releia a anatomia de custo.

Esta aqui ficou longa para uma lição sem build nenhum, mas o modelo tinha que ser conquistado, não afirmado. Se algum degrau dele ainda parecer bambo, ou se o seu arquivo de política saiu diferente de onde você esperava, leve para a comunidade do curso e discuta; uma discordância defendida ensina melhor do que uma tabela lida concordando com a cabeça.

Agora você consegue defender uma política de confirmação, e sabe que reembolsos são trabalho seu, não da rede. No próximo módulo a gente para de falar e constrói: o transfer-kit que toda lição posterior importa, começando pela única primitiva que esta lição ficou adiando. A conta de token, e o rent que custa para criá-la, vêm primeiro. Guarde o arquivo de política: o verificador do lado do servidor do módulo 4 é onde esses três títulos param de ser prosa e viram o argumento de commitment que o seu código de fato passa.
