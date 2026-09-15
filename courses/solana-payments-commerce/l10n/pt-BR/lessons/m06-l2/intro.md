# Aceitação e corredores: Stripe, MoonPay Commerce e PIX

## Resumo

Na lição passada você embutiu um onramp headless da Coinbase na vitrine e percorreu o offramp hospedado para o pagamento de um artista. O dinheiro entra e sai da Wavelength agora, e você sabe dizer quem é merchant-of-record em cada costura. Então a questão do encanamento está resolvida. A questão de hoje é uma questão de estrutura de mercado, e é ela que decide se a loja de fato vende discos: um comprador americano segurando um cartão de crédito, um comprador europeu que vive no SEPA e um comprador brasileiro que há anos não toca em nada além do PIX são três problemas diferentes atrás de um único botão de checkout. Escolha o processador errado para um corredor e você ou perde a venda de cara ou come a margem de cada unidade caladinho. É essa a lição inteira: qual trilho para qual comprador, e em que você de fato liquida.

Pense do jeito que uma gravadora pequena pensa em distribuição. Ninguém em sã consciência assina um distribuidor mundial exclusivo para uma prensagem de vinil. Você assina um distribuidor americano que conhece as lojas americanas, um distribuidor europeu que conhece as lojas europeias e um distribuidor brasileiro que conhece o Brasil, cada um nos próprios termos, cada um levando o próprio corte, cada um te pagando no próprio calendário. Processadores de aceitação são distribuidores de território para dinheiro. Esta lição compara três deles em números, e depois te faz assinar os contratos por escrito.

Antes de qualquer coisa, faça uma coisa agora. O primeiro fato do dia dá para conferir do seu terminal, então confira. O `curl` vem pré-instalado no macOS e em quase toda distro Linux (`brew install curl` se a sua for a exceção):

```bash
curl -sIL https://hel.io | grep -i '^location'
```

Você deve ver a cadeia de redirects sair do domínio antigo da Helio e aterrissar numa propriedade da MoonPay. Se ela aterrissar em outro lugar, alguma história de aquisição se mexeu de novo desde que isto foi escrito, e você acabou de aprender a regra mais profunda desta lição um passo mais cedo: em pagamentos, todo fato de fornecedor tem uma data em cima.

Os achados, logo de cara:

- **Stripe pay-with-crypto** aceita USDC na Solana no checkout e liquida em fiat no seu saldo Stripe. Disponibilidade geral nos EUA, um limite de US$ 10,000 por transação por cliente, e reembolsos devolvidos como stablecoins para a carteira de origem, que é exatamente a construção de reembolso que você montou dois módulos atrás, agora rodando dentro do adquirente de outra pessoa.
- **MoonPay Commerce** é a antiga Helio: checkout, pay links, cartão para cripto, com liquidação configurável para cripto, stablecoin ou fiat convertido. A Wavelength gira esse seletor para stablecoin e fica com a tesouraria dela. O número de volume dela é reportado pelo próprio fornecedor e está em movimento, então a gente cita com fonte e data, nunca como constante.
- **Sphere** é a âncora Brasil: trilhos que incluem PIX ao lado de SEPA e ACH, liquidação cotada em menos de 30 minutos.
- **A fronteira que não é técnica**: um método de cinco perguntas para localizar a linha regulatória de um corredor, rodado no Brasil como caso de verdade — duas fronteiras, duas datas de vigência, dois prazos de agenda, e um fluxo de lojista que fica em cima da linha em vez de dentro dela.
- **Precificação de exibição** para discos precificados em dólar e cobrados em USDC não precisa de oráculo nenhum, porque o USDC tem paridade com o dólar e é cobrado 1:1. O caso do ativo volátil é nomeado e repassado.
- A entrega é o **registro de decisão de corredores**: uma tabela escrita de três linhas mais um esqueleto de config, um trilho por geografia de comprador, com ativo de liquidação e merchant-of-record declarados por linha.

Como o trabalho se divide: esta é uma lição de conceito lá no fim do curso, então a proporção se inverte. A gente percorre os três processadores e a lógica de precificação junto, com números. O registro de decisão e o esqueleto de config são só seus, sem scaffold, porque uma tabela de corredores que outra pessoa preencheu não decide nada por você. Ela é um default, e defaults são como margens morrem.

## Três territórios, três distribuidores

### Um processador de aceitação não é um onramp

Primeiro, uma definição que o ecossistema adora borrar, posta na hora certa. O onramp e o offramp da lição passada movem o fiat de um comprador para cripto que ele possui, ou a cripto de um lojista para fiat que ele possui. O cliente de uma rampa é quem quer o ativo trocado. Um **processador de aceitação** fica em outro lugar completamente: o cliente dele é o lojista, e o trabalho dele é pegar o que quer que o comprador tenha e entregar o que quer que o lojista queira segurar, cobrando uma taxa por ficar no meio. Um **corredor** é o par que esta lição não para de pontuar: uma geografia de comprador mais o trilho que alcança ela. E o **ativo de liquidação** é a coisa que de fato aterrissa na sua conta no fim, fiat ou stablecoin, que é a coluna mais consequente da tabela de hoje, porque é ela que decide se você roda uma tesouraria em cripto ou não.

![Uma rampa troca ativos para quem os possui, enquanto um processador de aceitação fica entre comprador e lojista entregando o ativo de liquidação do lojista; um corredor casa geografia com trilho.](assets/v01-diagram.png)

Por que a distinção merece a própria seção? Porque a leitura errada mais comum neste mercado é olhar para um produto de liquidação em fiat para lojista e descrever assim: "a loja faz offramp dos fundos dela". Não faz. Quando um processador te liquida em fiat, a sua loja nunca segura cripto naquela venda, então não tem nada para tirar por offramp. O offramp que você percorreu na lição passada e a liquidação em fiat que você vai conhecer daqui a pouco são máquinas diferentes que por acaso terminam na mesma moeda, e confundir as duas vai te fazer construir infraestrutura de tesouraria de que você não precisa. Eu já vi times fazerem exatamente isso. Não é um desperdício pequeno.

### Stripe: o adquirente que liquida em fiat

Comece pelo território que você já sabe pensar, porque você integrou a Stripe numa vida passada. Stripe pay-with-crypto aceita USDC na Solana no checkout e liquida em fiat no seu saldo Stripe. Está em disponibilidade geral nos EUA. Deixe essa frase assentar: o comprador paga em stablecoin on-chain, e na manhã seguinte o seu dashboard da Stripe mostra dólares, no mesmo saldo em que caem as suas vendas de cartão, no calendário da Stripe. Você mantém os seus livros inteiros, o seu contador nunca descobre o que é uma ATA, e a perna de cripto vira um detalhe de implementação do adquirente.

As travas te dizem o que a Stripe pensa de dinheiro irreversível. Existe um limite de US$ 10,000 por transação por cliente. E reembolsos são devolvidos como stablecoins para a carteira de origem, o que deveria soar um alarme: esse é o pagamento de push ao contrário que você construiu na mão na lição de conciliação, a mesma regra de carteira de origem, a mesma geometria sem chargeback. Você não chegou nesse formato por conta própria, e seria lisonjeiro fingir o contrário: a lição de conciliação copiou o precedente da Stripe de propósito, porque era o único que existia. O que é novo aqui é ver a mesma regra valer num trilho em que o lojista nunca toca em cripto, o que te diz que a devolução para a carteira de origem é uma propriedade do dinheiro de push, e não uma preferência de tesouraria que você herdou. Pegue o teto de US$ 10,000 como a outra metade do recado: quando a Stripe limita um trilho com esse aperto, ela está precificando a irreversibilidade que você já sabe que está ali.

![O USDC do comprador na Solana passa pela Stripe até o saldo em fiat do lojista sob um teto por transação, enquanto reembolsos voltam como stablecoins empurradas para a carteira de origem.](assets/v02-diagram.png)

A economia deste contrato é a economia do conforto. Liquidação em fiat te poupa uma tesouraria em cripto, te poupa a costura do offramp, te poupa toda pergunta de conciliação sobre segurar stablecoins no balanço. Em troca você aceita o teto, o calendário de liquidação do processador e uma pegada geográfica que é, pelos fatos congelados para esta lição, disponibilidade geral nos EUA. O seu comprador europeu não está nessa frase. O seu comprador brasileiro não chega nem perto. Sondar de novo a própria documentação da Stripe em 2026-08-22 acrescenta uma ruga viva que vale carregar para o seu registro sem mudar a linha: compradores podem pagar de qualquer lugar, o que é restrito é a localização do *negócio*, e ao lado da disponibilidade geral nos EUA a Stripe lista a UE, Hong Kong, o México e a Suíça em private preview. Private preview é lista de espera, não corredor, então não ganha célula numa tabela contra a qual você vai despachar de verdade. Ponha na sua lista de reverificação, porque é exatamente o tipo de linha que vira. E uma cláusula do contrato de distribuição importa aqui para a loja de discos: você continua merchant-of-record da venda. A Stripe é o seu adquirente, não a vendedora dos seus discos. Segure isso ao lado do mapa de costuras da lição passada sem vacilar, porque as duas lições não estão discordando: lá, o *onramp* da Stripe pôs a Stripe na cadeira de merchant-of-record, porque a coisa vendida era a própria cripto; aqui, a coisa vendida é o seu disco, e a *aceitação pay-with-crypto* da Stripe é um produto diferente numa cadeira diferente, adquirente da sua venda. Mesmo logo, duas cadeiras. Nomear o produto antes de nomear a cadeira é exatamente a disciplina que o mapa de costuras estava instalando. A reclamação do comprador sobre uma prensagem empenada é com a Wavelength, neste trilho e em todo trilho da tabela de hoje.

Mais uma coisa sobre este distribuidor em particular, porque ela explica o mercado em que você está operando. A Stripe hoje ocupa uma posição de quatro frentes: ela aparece como logo de "trusted by" do x402 (o padrão HTTP-nativo para pagamentos máquina a máquina que você conhece no próximo módulo), ela coescreveu o ACP, o Agentic Commerce Protocol para checkout de agentes de IA, com a OpenAI, ela coescreveu o esquema de autenticação HTTP "Payment" em que o MPP é construído (o outro trilho de pagamentos de máquina do próximo módulo), e ela roda este trilho de adquirência de USDC na Solana que liquida em fiat. Uma empresa com essa paciência está te dizendo para onde ela acha que os pagamentos estão indo. Segure esse pensamento até o fim desta lição; a frente do x402 é a próxima coisa que você constrói.

![A Stripe ocupa quatro posições ao mesmo tempo: apoiadora do x402, coautora do ACP com a OpenAI, coautora do esquema de autenticação HTTP Payment por trás do MPP, e adquirente de USDC na Solana liquidando em fiat.](assets/v03-diagram.png)

### MoonPay Commerce: o checkout com um seletor de liquidação

Agora o contrato oposto. MoonPay Commerce é a antiga Helio, e você mesmo verificou a aquisição nos primeiros cinco minutos: hel.io agora redireciona para o braço de comércio da MoonPay. O formato do produto é um checkout em cripto com pay links e um caminho de cartão para cripto, e aqui o ativo de liquidação é um seletor em vez de um dado: a própria página de produto oferece liquidação em cripto, em stablecoins, ou convertida automaticamente para fiat (USD, EUR, GBP) nas regiões suportadas, consultada em 2026-08-22. Esse seletor é a razão inteira de ela ganhar uma coluna. Gire para fiat e você comprou uma segunda Stripe com geografia diferente; gire para stablecoin, que é o que a Wavelength faz, e a venda aterrissa como tokens que você segura, alimentando exatamente a maquinaria de tesouraria e conciliação que você construiu no módulo 4. O seu conciliador, o seu livro-razão e o seu construtor de reembolsos mantêm todos o emprego. Escreva a posição do seletor no seu registro, não o nome do fornecedor, porque "a gente liquida em USDC" é a decisão e "MoonPay Commerce" é só onde você configurou ela.

Aqui é onde a disciplina de números que esta lição não para de pregar ganha o caso de teste dela. O número atrelado a este produto, do jeito que o roundup de ecossistema de abril de 2026 da Solana reporta, recuperado em 2026-08-22, é que a MoonPay Commerce reportou mais de US$ 40 milhões em volume de "single-payment", o rótulo do próprio fornecedor para pagamentos avulsos de checkout em contraste com os recorrentes, desde o lançamento em outubro de 2025, com 88% disso na Solana. Carregue o rótulo entre aspas no seu registro, precisamente porque foi o fornecedor que o definiu. Repare em tudo o que eu acabei de fazer com esse número. Ele tem fonte, e a fonte é um relato do relato da própria MoonPay, então a cadeia tem dois elos e você deveria dizer isso. Ele tem data de recuperação. Ele é um número escolhido pelo fornecedor, ou seja, um número lisonjeiro, porque fornecedores escolhem números lisonjeiros. E ele é um número de fluxo de um produto jovem, o que quer dizer que vai estar velho quando você ler isto, possivelmente quando eu terminar o parágrafo. Os 88% são sinal genuinamente útil sobre onde mora a demanda por checkout em cripto. Os US$ 40 milhões são um retrato de um objeto em movimento. O seu registro de decisão cita números assim com fonte e data, ou não cita, porque uma tabela de corredores cheia de números de fornecedor sem data não é pesquisa; é um folheto com o seu nome nele.

Quanto custa o contrato que liquida em cripto? Você entrega a UX do checkout e a tabela de taxas ao processador, e na perna de cartão para cripto o comprador é momentaneamente cliente da MoonPay para a conversão, a mesma costura que você mapeou no onramp na lição passada, antes de os tokens resultantes pagarem a sua fatura. Em troca você ganha alcance de corredor que não depende da lista de países de um adquirente, liquidação num ativo que você já sabe conciliar, e pay links que você joga numa DM, o que para uma loja de discos fazendo drops de pré-venda é um encaixe genuinamente bom. Para completar: Transak e Meso também rondam este território, e ficam só nomeadas neste curso porque a cobertura de Solana delas ficou sem verificação quando os fatos desta lição foram conferidos. Um distribuidor não verificado não ganha linha na tabela. Deixar ele de fora é o que faz as outras linhas valerem confiança.

![Três processadores comparados lado a lado em ativo de liquidação, trilhos, cobertura e limites, com Transak e Meso mostradas como excluídas porque a cobertura de Solana delas ficou sem verificação.](assets/v04-comparison.png)

### Sphere e o corredor do PIX

O que nos leva ao comprador que os dois distribuidores anteriores deixam plantado no caixa. Vou gastar aqui a minha única ficha de credibilidade de casa: na Superteam Brazil eu vejo pagamentos aterrissarem todo dia, e em São Paulo eu passo meses sem ver um cartão físico, porque o camelô, o barbeiro e a bilheteria da casa de show aceitam PIX pelo celular. Um comprador brasileiro que chega num checkout que só oferece campos de cartão não pensa "inconveniente". Pensa "estrangeiro", e uma fatia relevante desses compradores vai embora. Um corredor não é um mimo para esta geografia; é a diferença entre ter clientes brasileiros e ter visitantes brasileiros.

Sphere é a âncora Brasil da pesquisa. O site dela se vende bem mais largo do que um país, liquidação em mais de 160 mercados, e a razão de ela ainda entrar nesta tabela como a escolha Brasil é o trilho, e não a contagem de mercados: PIX é o que um comprador brasileiro exige, e PIX é o trilho que a Sphere carrega e que nenhuma das outras linhas carrega, ao lado de SEPA e ACH, com um número de liquidação cotado em menos de 30 minutos. Repare no verbo, cotado. É o número do fornecedor, então ele viaja com a mesma disciplina do número de volume da MoonPay: date, cite a fonte, e trate como uma afirmação que você reverifica antes do capstone, não como uma constante que você herdou. Mas mesmo descontado, menos de meia hora entre o toque de um comprador de PIX e os fundos liquidados é outro esporte, comparado com a liquidação internacional de cartão de vários dias que ele substitui, e o SEPA na mesma lista faz da Sphere, caladinho, uma candidata também para o seu corredor UE, não só para o Brasil. Uma célula que a página pública não vai preencher para você: a Sphere cota *prazo* de liquidação, não *ativo* de liquidação. O seu registro de decisão ainda precisa dessa célula, então anote o que a Wavelength pede (USDC, para manter uma tesouraria só entre UE e BR) e marque a célula como confirmar-com-fornecedor; uma célula que você não consegue citar é uma célula que você sinaliza, nunca uma célula que você chuta.

O trade-off é o espelho da força. A Sphere ganha a linha BR porque carrega o trilho que a aceitação em cripto da Stripe não toca, e isso vale ser dito como a cilada que é, porque eu já vi essa suposição solta por aí: Stripe pay-with-crypto não oferece PIX. A aceitação em cripto da Stripe é baseada em rede de stablecoin; PIX é o trilho da Sphere neste elenco. Enquanto isso, nada no elenco da Sphere muda a linha US, onde o comprador quer um checkout de nível adquirente, colado em cartão, que era o território inteiro da Stripe. Nenhum distribuidor cobre o mapa. Culpe o mapa, e não os fornecedores: essa assimetria é exatamente por que o registro de decisão de corredores existe como uma tabela por geografia em vez de uma resposta de uma linha.

![Um fluxograma de decisão roteando compradores dos EUA, da UE e do Brasil para os trilhos deles, com todo desfecho registrado ao lado do ativo de liquidação, do merchant-of-record e de uma data de verificação.](assets/v05-flowchart.png)

### A fronteira que não é técnica

Uma célula da tabela que você está prestes a assinar continua marcada como confirmar-com-fornecedor: em que a Sphere de fato te liquida. Antes de mandar esse e-mail, seja honesto sobre que tipo de pergunta você está segurando. Nada na sua stack responde isso. Se um corredor pode ou não te liquidar em USDC é decidido por textos publicados em diário oficial, textos que mudam em datas de vigência e não em ciclos de release, e a disciplina para lidar com eles é a mesma que você acabou de aplicar a números de fornecedor, rodada com mais cuidado porque aqui o custo não é margem. Então aqui vai essa disciplina como método, e depois uma jurisdição rodada nele de verdade.

Cinco perguntas, feitas nesta ordem, porque cada uma escopa a seguinte — embora um caso vivo muitas vezes te faça responder a pergunta 3 primeiro, como o passo a passo do Brasil abaixo faz:

1. **Nomeie a perna.** Nunca "a gente aceita cripto" — qual movimento de valor, de quem, para quem, em qual ativo? Uma linha de corredor esconde várias pernas (comprador para processador, processador para você, o reembolso de volta), e regras se prendem a pernas, não a produtos.
2. **Nomeie o ator, e a licença dele.** Para cada perna, quem está de fato movendo o dinheiro, e autorizado como o quê, onde? "O processador cuida disso" só vira resposta quando você consegue dizer o que o processador é autorizado a ser naquela geografia.
3. **Pergunte como o regulador classifica o seu ativo de liquidação.** Não como o fornecedor comercializa ele — como a lei define ele. O mesmo USDC pode ser ativo virtual num regulamento e outra coisa no seguinte, e toda obrigação lá na frente se prende a essa classificação.
4. **Pergunte a residência de cada parte.** Fluxos transfronteiriços e domésticos ficam sob regras diferentes, e a direção surpreendente é a que vale conferir: alguns perímetros alcançam fluxos que não têm fronteira nenhuma dentro.
5. **Date e agende.** Fatos jurídicos decaem como fatos de fornecedor, só que aqui o decaimento é programado: datas de vigência, janelas de protocolo, períodos de transição. Toda resposta recebe a data em que você conferiu, e as datas que vão mudar a resposta vão para um calendário, não para um rodapé.

Agora o Brasil, porque é a minha casa, porque é a linha BR da tabela, e porque é o exemplo vivo mais afiado que eu conheço de uma fronteira que se mexeu enquanto um curso estava sendo escrito. Eu puxei todo texto citado abaixo das fontes primárias — o Diário Oficial da União para as resoluções do Banco Central, o Planalto para as leis, o acompanhamento da própria Câmara para o projeto de lei — em 2026-09-02. Essa data importa mais do que a minha leitura; puxe os textos de novo antes de confiar em qualquer uma das duas.

**Pergunta 3 primeiro, porque tudo se pendura numa palavra.** A Lei 14.478/2022, art. 3º define ativo virtual e exclui expressamente moeda nacional e moeda estrangeira da definição. Uma stablecoin referenciada em dólar é, portanto, na lei brasileira, ativo virtual e não moeda estrangeira, e essa classificação decide em qual regulamento o resto desta seção mora. Você também vai ver o PL 4.308/2024 citado em torno deste tema, então situe ele com precisão: ele mudaria quem pode emitir e distribuir tokens referenciados em moeda fiduciária — um perímetro de emissão — e não toca na classificação do art. 3º nem no texto original nem no substitutivo aprovado em comissão. E ele ainda é um projeto de lei: em 2026-09-02 está numa comissão da Câmara esperando o parecer do relator, ainda sem passar pela Casa de origem. Um projeto de lei é uma entrada na agenda, não uma regra.

**Perguntas 1, 2 e 4: a fronteira vem em duas peças, com duas datas.** A primeira peça é a Resolução BCB 521, de 10 de novembro de 2025, que escreveu os serviços de ativos virtuais para dentro do mercado de câmbio inserindo os arts. 76-A e 76-B na Resolução BCB 277. Esses dois artigos estão em vigor desde 2026-02-02; a maquinaria de reporte da resolução entrou em vigor depois, em 2026-05-04, então date o artigo que você está citando, não a resolução. O art. 76-A é o teste de perímetro, e ele é mais largo do que o ar internacional do enquadramento dele sugere. Quatro grupos de atividade ficam dentro do mercado de câmbio quando um PSAV está no fluxo: pagamento ou transferência internacional com ativos virtuais; transferências ligadas ao uso internacional de cartão; transferências de ou para carteira de autocustódia, expressamente aquelas sem nenhum pagamento internacional dentro; e a compra, a venda ou a permuta, por um PSAV, de ativos virtuais referenciados em moeda fiduciária, com "fiduciária" sem qualificação, então um token referenciado em real é capturado igual a um referenciado em dólar.

Leia esse terceiro grupo de novo, porque é o caso de confira-não-suponha mais afiado que eu consigo te entregar. Um comprador em São Paulo pagando de uma carteira de autocustódia, através de um PSAV, para um lojista em São Paulo não cruza fronteira nenhuma — e mesmo assim fica dentro do perímetro do câmbio, porque o 76-A III alcança transferências de autocustódia sempre que um PSAV é uma perna. Esse é o formato padrão de um checkout Solana-nativo. Se o seu instinto disse doméstico-logo-de-fora, ele acabou de falhar exatamente no fluxo que este curso despacha, e só ler o artigo pega isso. O instinto falha na outra direção também: o art. 76-B, que define o grupo internacional, cobre mais do que residente-paga-não-residente. Uma mudança de titularidade entre dois não residentes conta, e um movimento de mesmo dono também — um residente brasileiro mandando a própria stablecoin para a própria carteira no exterior faz uma transferência internacional sob o 76-B II, sem contraparte nenhuma em vista. Residência, pergunta 4, tem que ser perguntada de cada parte, inclusive de você.

Duas cláusulas do 76-A então fazem o trabalho específico do lojista. O §2º proíbe que a compra ou a venda de ativos virtuais por um PSAV seja paga ou recebida em moeda estrangeira — repare na forma operativa, uma proibição de uma perna em moeda estrangeira, e não um comando sobre o que a perna em fiat tem que ser, e a diferença não é preciosismo: uma proibição deixa aberto o que um mandamento fecharia, e parafrasear uma na outra é como resumos de segunda mão dão errado. E o §3º veda movimentar recursos de terceiros através de um serviço de ativos virtuais dentro do escopo, com uma exceção: um PSAV que atende uma instituição que seja ela própria autorizada no mercado de câmbio e esteja agindo pelos clientes dela. Agora caminhe a linha BR da Wavelength por essas cláusulas. Um provedor que pega a stablecoin do seu comprador e entrega valor para você está, nos termos definidos pela própria resolução, comprando um ativo virtual referenciado em moeda fiduciária — grupo quatro, dentro do perímetro — enquanto movimenta recursos no seu interesse, que é a coisa que o §3º proíbe a menos que o provedor esteja dentro daquela exceção. Ou seja, o fluxo intermediado de lojista que a tabela desta própria lição roteia não está confortavelmente dentro da fronteira. Ele está em cima dela, e de que lado o seu provedor está é exatamente a pergunta 2, a pergunta da licença que você agora sabe que tem que fazer para a Sphere no mesmo e-mail que a célula do ativo de liquidação.

A segunda peça é a Resolução BCB 561, de 30 de abril de 2026, em vigor em 2026-10-01 numa vigência única e indivisível, voltada para o eFX — o regime de serviços de pagamento e de transferência internacional. O art. 50, I dela exige que a liquidação entre um provedor de eFX e a contraparte estrangeira dele passe por uma operação de câmbio ou por conta de não residente em reais, com o uso de ativos virtuais expressamente vedado. Mantenha a precisão: isto não é "eFX não pode tocar em cripto" — a mesma resolução cria um código de finalidade para aquisição de ativos virtuais via eFX. O que ela veda é a perna de liquidação entre o provedor e a contraparte estrangeira ser denominada em ativos virtuais. Uma regra de perna de liquidação, em outras palavras, mirando exatamente a coluna da sua tabela que ainda está marcada como confirmar.

**Pergunta 5, a agenda.** Duas datas vão nela, cada uma atribuída à fonte de verdade dela, porque argumentar a partir do normativo errado é um modo de falha por si só. **2026-10-30** é o prazo para um PSAV já em atividade pedir autorização, fixado pela Resolução BCB 520, art. 88, I — expresso lá como 270 dias contados da vigência da resolução em 2026-02-02; um provedor que perde o prazo tem que encerrar as atividades em trinta dias, e a partir dessa mesma data uma regra separada no art. 91 impede as instituições reguladas do país de operar com PSAVs que não estejam nem autorizados nem na fila de autorização. **2027-05-31** é o prazo para provedores de eFX fora da lista de instituições enumeradas da resolução pedirem autorização como instituição de pagamento, conforme a Resolução BCB 561, art. 56-B, com a própria regra de encerrar em trinta dias atrás dele. As duas datas vão ao lado da linha BR com uma nota do que reperguntar quando cada uma passar. E quando você mandar para a Sphere o e-mail de confirmar-o-ativo-de-liquidação, pergunte na mesma mensagem sob qual desses regimes ela opera e o que ela protocolou, com data. Um fornecedor que responde com precisão te disse alguma coisa. Um fornecedor que responde "a gente está 100% em compliance" também te disse alguma coisa.

Para dizer com todas as letras mais uma vez, nas mesmas palavras que a lição um deste módulo usou: este é um enquadramento de engenharia de onde ficam as costuras, não aconselhamento jurídico, e um produto de serviços financeiros de verdade despacha com um advogado de verdade. O que as cinco perguntas te compram é a capacidade de instruir esse advogado em uma página, com datas, em vez de descobrir as perguntas na reunião, ao preço-hora da reunião.

### Precificação de exibição: o oráculo de que você não precisa

Mais uma peça de estrutura de mercado antes de você assinar qualquer coisa, porque ela parece um problema difícil e o ponto inteiro é que, para esta loja, não é. A Wavelength precifica discos em dólar. Compradores pagam em USDC. Qual é a taxa de câmbio?

Não existe uma, e esse é o projeto. O USDC tem paridade com o dólar, então a prensagem de agosto de US$ 30 do seu catálogo é cobrada como exatamente 30 USDC, 1:1, sem cotação, sem spread, sem política de arredondamento. **Precificação de exibição**, o preço na etiqueta, e o **valor cobrado**, os tokens que se movem, são o mesmo número. É precisamente por isso que o curso inteiro rodou sobre trilhos de stablecoin: a paridade apaga o problema de câmbio na camada do checkout. Então quando você se pegar rascunhando uma integração de price feed para uma stablecoin atrelada ao dólar cobrada 1:1, pare. Você estaria construindo um oráculo para descobrir um número que a paridade já te prometeu. Eu sinalizo isso porque é uma cilada de verdade com vítimas de verdade: o cérebro que casa padrões vê "pagamento em cripto" e estica a mão para "price feed" antes de conferir se alguma coisa de fato flutua.

Onde a precificação de exibição fica interessante é no momento em que o ativo cobrado flutua contra a moeda da etiqueta: precificar um disco em SOL, ou aceitar um token volátil no checkout. Aí você precisa de um preço ao vivo, uma regra de defasagem, uma política de spread e um oráculo que você consiga defender, e essa maquinaria é uma disciplina genuína com os próprios modos de falha. Ela também é, de propósito, não a disciplina deste curso. Precificação baseada em oráculo — janelas de defasagem, intervalos de confiança, política de spread — é território de DeFi e RWA Engineering, e o seu registro de corredores vai anotar o repasse em vez de contrabandear uma versão meio ensinada. Um curso de pagamentos que te ensinasse um quarto de um oráculo não estaria te fazendo favor nenhum; o quarto que faltaria é o quarto que perde dinheiro.

![Precificação de checkout atrelada e flutuante lado a lado, onde um disco precificado em dólar cobra o mesmo número de USDC enquanto ativos voláteis precisam de oráculo, regras de defasagem e política de spread.](assets/v06-comparison.png)

### O trade-off de que ninguém escapa

Dê um zoom out e pontue os três contratos num eixo só, porque toda escolha de corredor é a mesma troca em proporções diferentes: cobertura contra custo e controle. Aceitação que liquida em fiat te compra uma tesouraria simples e livros que o seu contador já entende, e te cobra o teto, o calendário do processador e a geografia dele. Aceitação que liquida em cripto te compra a sua própria tesouraria e alcance agnóstico de trilho, e te cobra a UX de checkout e a tabela de taxas. O especialista de corredor único te compra uma geografia inteira, e te cobra todas as outras geografias. Rode o formato de brinquedo na prensagem de US$ 30 vendida três vezes, uma por corredor: a venda US liquida como fiat no calendário da Stripe menos a taxa da Stripe; as vendas EU e BR aterrissam como 30 USDC cada, menos a taxa de cada trilho, na tesouraria do módulo 4. Três vendas, três tabelas de taxas, dois ativos de liquidação, e depois da liquidação o seu dinheiro está sentado em dois tipos diferentes de conta sob três conjuntos diferentes de termos. Não existe configuração do mercado de hoje em que um processador ganhe as três linhas no mérito. Quem te disser o contrário está vendendo uma das linhas.

Que é por que a entrega honesta é uma tabela e não uma recomendação. E por que a própria tabela decai: as afirmações de cobertura, o prazo de liquidação cotado e o volume divulgado nela são todos retratos datados de fornecedores em movimento. Um registro de decisão sem datas de verificação está errado; ele só não te contou quando.

Nem tudo nele decai na mesma velocidade, porém, e ordenar os fatos pelo relógio de cada um é o que o lab te pede primeiro.

![Os fatos desta lição ordenados em dois relógios de decaimento, números de fornecedor que precisam de fonte e data contra fatos estruturais de produto declarados direto.](assets/v07-table.png)

## Lab: assine os contratos de distribuição

Hora de escrever. O artefato é o `corridor-decision`: um registro de decisão mais um esqueleto de config. De propósito, não é código importável. Nada no capstone vai dar `import` neste arquivo; o time do capstone, ou seja, você daqui a três módulos, vai ler ele e configurar de acordo. Ele consome o `ramp-embed` no sentido honesto de que as linhas dele têm que concordar com as costuras de rampa que você mapeou na lição passada.

**Passo 1: scaffold.** A partir da raiz `wavelength`, a mesma raiz de workspace que todo módulo construiu:

```bash
mkdir -p corridor-decision
cd corridor-decision
touch DECISION.md corridors.config.ts
```

**Passo 2: rode a sonda de novo e date.** Você rodou a checagem de redirect do `hel.io` na abertura. Cole o comando e a saída dele no `DECISION.md` sob um cabeçalho chamado `Verified facts`, com a data de hoje ao lado. Depois acrescente os outros números em movimento que você vai citar, cada um com fonte e data: o número de volume da MoonPay Commerce (mais de US$ 40 milhões em single-payment desde o lançamento em outubro de 2025, 88% na Solana, como reportado no roundup de ecossistema de abril de 2026 da Solana, recuperado em 2026-08-22), e a liquidação em menos de 30 minutos cotada pela Sphere (spherepay.co: liquidação "em menos de 30 minutos em mais de 160 mercados", recuperado em 2026-08-22). Fatos estruturais congelados como a liquidação em fiat da Stripe, o limite de US$ 10,000 por transação e os reembolsos em stablecoin para a carteira de origem vão numa lista separada, porque eles mudam em ciclo de produto, não em ciclo de notícia. As duas listas decaindo em velocidades diferentes é a razão de serem duas listas.

**Passo 3: preencha as três linhas.** No `DECISION.md`, escreva a tabela que a avaliação pede: US, EU, BR na lateral; trilho, ativo de liquidação, merchant-of-record no topo; uma linha de justificativa por linha. Discuta com o fluxograma acima, não a partir dele. Se você rotear a UE pelo trilho SEPA da Sphere em vez do MoonPay Commerce, ótimo, defenda isso na linha de justificativa. O registro é seu; o requisito é que toda célula seja uma decisão que você consegue sustentar em voz alta.

**Passo 4: codifique o esqueleto.** O arquivo de config deixa o registro legível para o você-do-futuro sem fingir que é uma biblioteca. TypeScript já é devDependency do repo do curso desde o `npm install` do módulo 2, fixado lá em `^5.6.0`; fora do repo, `npm i -D typescript` te dá o compilador, e uma instalação pelada hoje aterrissa na linha 7.x (o `latest` do npm era 7.0.2 em 2026-08-22). Os dois servem aqui, porque este arquivo não importa nada e não usa sintaxe mais nova que a 5.x. Essa deriva de versão é ela mesma a lição: como todo pin deste curso, o número no `package.json` envelhece, e a tag chamada `latest` se mexe debaixo dele.

```ts
// corridor-decision/corridors.config.ts
// Decision record skeleton. Read by humans configuring the capstone; imported by nothing.

export type Corridor = 'US' | 'EU' | 'BR';

export type Rail = 'stripe-pay-with-crypto' | 'moonpay-commerce' | 'sphere';

export type SettlementAsset = 'fiat-via-processor' | 'usdc';

export interface CorridorDecision {
  corridor: Corridor;
  rail: Rail;
  settlementAsset: SettlementAsset;
  /** Who the buyer's contract of sale is with. For every rail in this roster,
   *  Wavelength remains merchant-of-record for the record itself; conversion
   *  legs (e.g. card-to-crypto) briefly interpose the processor, per lesson 1. */
  merchantOfRecord: 'wavelength';
  /** Processor-imposed per-transaction ceiling in USD, or null if none stated. */
  perTxLimitUsd: number | null;
  /** One sentence you are prepared to defend to an accountant. */
  rationale: string;
  /** ISO date every moving fact in this row was last checked. Stale row, stale decision. */
  verifiedOn: string;
}

export const corridors: readonly CorridorDecision[] = [
  {
    corridor: 'US',
    rail: 'stripe-pay-with-crypto',
    settlementAsset: 'fiat-via-processor',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: 10_000,
    rationale:
      'US buyers get acquirer-grade checkout; fiat settlement keeps US books processor-side; refunds return as stablecoins to the originating wallet.',
    verifiedOn: '2026-08-22',
  },
  {
    corridor: 'EU',
    rail: 'moonpay-commerce',
    settlementAsset: 'usdc',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: null,
    rationale:
      'Stripe pay-with-crypto is US-GA; crypto settlement keeps EU sales in the module-4 treasury; Sphere SEPA is the recorded alternative.',
    verifiedOn: '2026-08-22',
  },
  {
    corridor: 'BR',
    rail: 'sphere',
    // Sphere's public page quotes settlement timing, not asset; 'usdc' here is
    // Wavelength's REQUESTED setting, flagged confirm-with-vendor before capstone.
    settlementAsset: 'usdc',
    merchantOfRecord: 'wavelength',
    perTxLimitUsd: null,
    rationale:
      'Brazilian buyers expect PIX; Sphere carries PIX alongside SEPA and ACH with settlement quoted under 30 minutes; one treasury asset across EU and BR.',
    verifiedOn: '2026-08-22',
  },
];
```

Aquelas três linhas são os meus contratos, não o gabarito. Os seus podem diferir, e uma linha diferente com uma justificativa mais afiada ganha de concordar comigo toda vez.

**Passo 5: prove que compila.** Um esqueleto que ninguém checa com tipos apodrece e vira prosa:

```bash
npx tsc --noEmit corridors.config.ts
```

Silêncio é sucesso. Se o compilador reclamar, leia o erro; a superfície de tipos é pequena o bastante para que toda falha aqui seja uma inconsistência de verdade no seu registro, que é a razão inteira de o esqueleto ser tipado em vez de ser um segundo arquivo markdown.

**Passo 6: feche o ciclo com as rampas.** Acrescente uma seção final ao `DECISION.md` intitulada `Seams with ramp-embed`, e responda em duas ou três frases: em quais corredores o onramp da lição passada ainda importa (um corredor que liquida em cripto ainda precisa de compradores que segurem USDC, ou de uma perna de cartão para cripto), e em qual corredor o offramp de pagamento de artista interage com a sua escolha de ativo de liquidação? Se a sua linha US liquida fiat via processador, repare no que você nunca faz para essas vendas: offramp. Escrever essa frase é a inoculação mais barata contra a leitura errada da liquidação em fiat com que esta lição abriu.

![O registro de decisão de corredores fica entre o ramp embed que o informa e o capstone que ele informa, segurando três linhas datadas que humanos leem mas nenhum código importa.](assets/v08-diagram.png)

## Challenge

O gate desta lição é o próprio registro, medido pelo formato da avaliação. Produza o registro de decisão de corredores para os compradores de US, EU e Brasil da Wavelength: uma tabela de três linhas com as colunas trilho, ativo de liquidação e merchant-of-record, uma linha de justificativa por linha, mais o esqueleto de config compilando. Barra de aceite: toda linha nomeia as três colunas explicitamente; todo número em movimento que você citar em qualquer lugar do registro carrega fonte e data; a linha da Stripe declara o limite de US$ 10,000 e o caminho de reembolso em stablecoin; a linha BR consegue dizer em uma frase por que PIX é inegociável naquele corredor; `npx tsc --noEmit` continua em silêncio.

Depois o stretch, que é onde o músculo de verdade da lição é construído: revisão adversarial da sua própria tabela. Para cada linha, escreva o argumento de uma frase a favor do trilho que você não escolheu. Se você não consegue escrever um argumento genuíno para a alternativa, você não tomou uma decisão, você deu um chute que por acaso caiu numa casa defensável. E rode o exercício de obsolescência: marque quais células da sua tabela você apostaria que ainda valem daqui a seis meses, e quais você reverificaria antes de apostar o almoço. O número de volume divulgado e o prazo de liquidação cotado não deveriam sobreviver a essa separação sem marca. Se sobreviveram, releia a seção da MoonPay.

## Checkpoint, e o próximo cliente na porta

Você já deve conseguir olhar para qualquer pitch de aceitação e localizar ele em dois eixos em uns dez segundos: em que o lojista liquida, e quais corredores ele de fato alcança. Esse reflexo, mais a disciplina de datar todo número de fornecedor, vale mais do que qualquer linha específica da tabela de hoje, porque as linhas vão derivar e os eixos não. Se a lição funcionou, a frase "a gente suporta pagamentos em cripto" agora soa para você como um distribuidor dizendo "a gente distribui discos", e a sua resposta imediata é: para quais territórios, liquidando em quê, no papel de quem. É esse o instinto de estrutura de mercado que este módulo existe para instalar, e se escrever as linhas de justificativa foi mais difícil do que ler as comparações, ótimo. É para ser. A leitura era a parte barata.

Onde a loja está: dinheiro entra, dinheiro sai, e agora toda geografia de comprador humano mapeada para um trilho com os termos por escrito. O que traz à tona a coisa estranha sobre o próximo cliente na porta. Ele não tem geografia. Ele não tem cartão, banco nem chave PIX, e nunca vai ver a sua página de checkout, porque os próximos clientes não são humanos: são agentes e máquinas pagando por chamada de API, milhares de vezes, em valores pequenos demais para qualquer processador da tabela de hoje se importar. A API de preço de prensagem da Wavelength está prestes a ganhar um paywall cujos compradores são bots, e o trilho para isso é aquele que a posição de quatro frentes da Stripe não parava de sugerir. Próximo módulo: x402.
