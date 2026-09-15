# HTTP 402, revivido: o protocolo x402

## Resumo

Você acabou de comparar corredores fiat e escreveu o registro de decisão de corredores da Wavelength para os compradores dela nos EUA, na UE e no Brasil. Todo cliente até aqui foi um humano segurando um celular. Nesta lição, o cliente deixa de ser humano.

Aqui está a cena. Uma API responde a uma requisição com HTTP 402 Payment Required, um código de status que ficou morto por vinte e cinco anos. Desta vez quem chama não é um humano clicando num paywall e sim um bot: ele lê o 402, paga e repete exatamente a mesma requisição, tudo antes da próxima linha do seu log. Ninguém clicou em nada. O protocolo que faz essa ida e volta funcionar se chama x402, e no fim desta lição você consegue ler o tráfego v2 dele como nativo.

O que você leva hoje, já de cara:

- A superfície do x402 v2 no nível da spec: os três headers que carregam a troca inteira, PAYMENT-REQUIRED, PAYMENT-SIGNATURE e PAYMENT-RESPONSE, mais ids de rede CAIP-2, quatro esquemas, três transportes.
- O fluxo exact-SVM de ponta a ponta: quem constrói a transação, quem assina parcialmente e por que um facilitador acrescenta a última signature e submete.
- O panorama de facilitadores na Solana, incluindo uma correção a um palpite que você vai ouvir repetido: a Helius não é facilitadora.
- Disciplina de fonte para os números de tráfego do x402, porque duas cifras publicadas imploram para serem misturadas numa única estatística errada.

Lição de conceito, então o lab te entrega um apoio pronto e deixa toda decisão de critério com você; na próxima lição as rodinhas saem quando você constrói o agente que paga. Antes de qualquer teoria, suba um endpoint 402 de mentira para cutucar. O Node está na sua máquina desde o módulo 2, onde os clients `@solana-program/*` fixam o piso em Node 24; o mock de hoje não importa nada além do `node:http` embutido do Node, e o curl vem junto com macOS e Linux:

```bash
mkdir -p ~/wavelength/x402-lab && cd ~/wavelength/x402-lab
node --version
```

Deixe esse terminal aberto. Em uns quatro minutos ele vai estar falando o mesmo código de status que a sua API de preço de prensagem vai falar por dinheiro.

## O quatrocentos e dois, desmistificado

### Por que um código de status morto voltou

O HTTP carrega uma vaga para pagamentos desde os anos noventa. O código de status 402, Payment Required, foi reservado nas primeiras specs de HTTP e nunca ganhou um comportamento definido: um lote com zoneamento comercial em que ninguém construiu por vinte e cinco anos. Toda tentativa de monetizar um endpoint HTTP deu a volta nele. Paywalls te redirecionam para uma página de checkout. Chaves de API movem o pagamento para um portal de cobrança e um cartão de crédito cadastrado. Os dois padrões dividem uma premissa: em algum ponto do fluxo, um humano com um navegador vai aparecer para digitar coisas.

O comércio agêntico quebra essa premissa. Quando quem chama é um programa, uma página de checkout é um beco sem saída; não tem ninguém para clicar nela. O que um chamador máquina precisa é de um desafio de pagamento dentro da própria ida e volta do HTTP: uma resposta legível por máquina para "isto custa dinheiro" que carregue tudo que é preciso para pagar, para que quem chama consiga liquidar e repetir sem nunca sair do protocolo. É exatamente esse o buraco para o qual o 402 foi zoneado, e o x402 é o protocolo que finalmente construiu no lote. O resumo honesto em uma linha: o x402 é um header de paywall com recibo. O servidor diz "pagamento obrigatório, os termos são estes" em um header, o cliente reenvia a requisição com prova de pagamento em um segundo, e o servidor responde com a mercadoria mais um recibo de liquidação em um terceiro. Três headers, uma ida e volta, e o lado do pagamento da troca nunca encosta num corpo de resposta. Todo o resto desta lição é o detalhe por trás desses três tempos.

A governança por trás da spec vale trinta segundos, porque ela te diz que isto é infraestrutura, não o SDK de uma startup. O x402 nasceu dentro da Coinbase, incubado pelo time de Development Platform dela, e desde então se mudou para uma x402 Foundation que opera sob a Linux Foundation. A Solana Foundation entrou nela. Essa trajetória, de experimento de uma empresa a tutela em casa neutra, é o caminho padrão para protocolos que pretendem sobreviver aos próprios criadores.

![Linha do tempo traçando o HTTP 402 desde a sua reserva não usada dos anos 1990, passando pela incubação do x402 na Coinbase, até uma casa na Linux Foundation e a superfície v2 estável de hoje.](assets/v01-timeline.png)

Um aviso de data antes da mecânica, já que você vai esbarrar em números de versão imediatamente: a spec v2 é a que ensinamos aqui porque a superfície dela é estável, mas a data de lançamento dela em mainnet não está publicada até o momento em que isto é escrito (2026-08-22), e a v1 continua viva por aí. Você está aprendendo a spec atual enquanto o mundo deployado se equilibra entre duas versões. Segure esse pensamento; ele vira uma cilada de interoperabilidade de verdade mais abaixo.

### A superfície v2: headers, redes, esquemas, transportes

Comece pelos nomes no fio, porque a v2 moveu a conversa inteira para headers e confundir um com o outro produz falhas silenciosas. Na v1, a prova de pagamento do cliente viajava num header chamado X-PAYMENT, e o recibo de liquidação do servidor voltava em X-PAYMENT-RESPONSE. A convenção de prefixo X- é formalmente desencorajada no HTTP há mais de uma década, e a v2 aposentou os dois nomes: o header de requisição agora é PAYMENT-SIGNATURE, e o header de resposta é PAYMENT-RESPONSE. Mesmos trabalhos, nomes novos.

O terceiro header não é uma renomeação, é uma mudança de endereço, e é o que pega as pessoas. Na v1 o desafio em si, os termos do pagamento, chegava como o corpo JSON da resposta 402. Na v2 não. O servidor serializa o desafio para JSON, codifica em base64 e define isso como um header de resposta chamado PAYMENT-REQUIRED; o corpo de um 402 v2 são dois bytes, `{}`, sob um `Content-Length: 2`. O documento de transporte HTTP da v2 é direto sobre o porquê: corpos de resposta são assunto de implementação do servidor, e toda informação do protocolo x402 é comunicada pelos três headers. Então tome isto como regra e aplique a toda resposta x402 que você inspecionar na vida: **leia o header, nunca o corpo.** Isso vale no desafio de abertura, num pagamento que o facilitador rejeitou e numa liquidação que falhou; o corpo está vazio nos três casos, e tudo que você quer saber, incluindo por que a requisição falhou, está sentado num header.

A cilada agora se escreve sozinha, duas vezes. Um cliente que manda X-PAYMENT para um servidor v2 está falando o dialeto do ano passado, e o servidor vê uma requisição sem prova de pagamento nenhuma: ele responde 402 de novo, o seu agente paga de novo, e você passa uma tarde aprendendo o que este parágrafo acabou de te contar. E um cliente que parseia o corpo do 402 procurando os termos encontra um objeto vazio, conclui que o servidor está quebrado e passa essa mesma tarde depurando um servidor que está se comportando perfeitamente.

Em seguida, como um pagamento nomeia a blockchain dele. O x402 é deliberadamente multi-chain, então o campo `network` nos termos de pagamento dele usa ids de rede CAIP-2, um padrão de nomes agnóstico de blockchain no qual toda rede ganha um id na forma `namespace:reference`. As redes Solana vivem sob o namespace `solana:` com uma reference derivada do hash de gênese do cluster. A lição prática para quem faz engenharia de pagamentos: nunca presuma que um 402 está pedindo pagamento na blockchain que você espera. Leia o campo `network`, confira se ele bate com o id CAIP-2 em que você pretende pagar, e recuse qualquer outra coisa. Checagem barata, proteção real.

Acima do formato do fio ficam dois eixos de variedade. Primeiro, quatro esquemas de pagamento, que respondem "que forma o pagamento toma":

- **exact**: pague um valor preciso por chamada, liquidado por chamada. O esquema de API medida, e o foco desta lição.
- **upto**: autorize até um teto, liquide o que foi de fato usado.
- **auth-capture**: o padrão dos trilhos de cartão que você conhece do módulo 1, autorização agora, captura depois, como esquema de primeira classe.
- **batch-settlement**: acumule muitas obrigações pequenas e liquide todas juntas.

Segundo, três transportes, que respondem "que protocolo carrega o desafio": **http** puro, que é o que você vem imaginando esse tempo todo; **mcp**, o Model Context Protocol que frameworks de agentes usam para chamadas de ferramenta; e **a2a**, mensageria agente-a-agente. A grade de esquemas-vezes-transportes é por que a spec parece maior do que ela é na prática. A sua cadeira de lojista se importa com uma célula só hoje: o esquema exact sobre http, mirando a SVM. A spec chama essa combinação de exact-SVM.

![Cartão de referência dando a cada um dos três headers v2 a sua direção, o seu trabalho e a sua origem na v1, sob uma faixa de regra que diz leia o header, nunca o corpo, ao lado do formato de rede CAIP-2 e de uma grade de quatro esquemas por três transportes com exact sobre HTTP destacado.](assets/v02-comparison.png)

### Uma requisição, de ponta a ponta: o fluxo exact-SVM

Agora siga uma chamada até o fim, porque o fluxo é onde o x402 deixa de ser uma spec e passa a ser um sistema de pagamentos. Imagine o cenário da próxima lição um degrau antes: a API de preço de prensagem da Wavelength cota custos de prensagem de vinil, e o bot de compras de um distribuidor quer uma cotação.

**Tempo um, o desafio.** O bot chama `GET /price`. O servidor responde 402 com corpo vazio e um header PAYMENT-REQUIRED; decodifique esse header de base64 e você tem o desafio, legível por máquina. Três campos ficam no nível de cima dele. `x402Version` é o inteiro `2`, e é assim que você sabe qual dialeto está lendo antes de encostar em qualquer outra coisa. `error` é uma string dizendo por que este 402 aconteceu, que no desafio de abertura é só uma reafirmação de que pagamento é obrigatório e num pagamento rejeitado é o motivo real da falha. `resource` é um objeto nomeando o que quem chama estava tentando comprar, a `url`, a `description` e o `mimeType` dele.

Embaixo deles fica `accepts`, um array de um ou mais objetos PaymentRequirements, os termos de pagamento em si, e é aqui que o resto da lição não para de voltar: `scheme` ("exact"), `network` (um id CAIP-2), `amount` (uma string de valor em unidades base; você aprendeu no módulo 2 por que dinheiro viaja como unidades base inteiras, e repare no nome, porque a v1 chamava esse mesmo campo de `maxAmountRequired` e você vai esbarrar nos dois), `asset` (o mint do token que liquida o pagamento, USDC no nosso caso), `payTo` (o endereço de dono do lojista, não uma token account), `maxTimeoutSeconds` (por quanto tempo o servidor vai manter estes termos abertos para o pagamento se completar) e um objeto `extra` com dois membros que fazem o sabor SVM funcionar. `extra.feePayer` nomeia a conta que vai pagar a taxa da transação, e ela não é o bot. `extra.memo`, limitado a 256 bytes, carrega o id da fatura que o lojista vai usar para conciliação; o nosso diria algo como `WVL-PRESS-0042`, e quando a transação liquidada aterrissa on-chain, esse memo é como o seu back office casa pagamento com pedido. Você construiu exatamente esse padrão de conciliação com o verificador no módulo 4; o x402 só padroniza onde o id viaja.

**Tempo dois, o pagamento.** O bot lê os termos e constrói uma transação Solana versionada que transfere `amount` de `asset` para o dono em `payTo`, com o memo anexado e com o slot do fee payer apontando para a conta nomeada em `extra.feePayer`. Aí ele faz uma coisa que merece definição própria, porque é a dobradiça do design inteiro. Uma transação Solana lista toda conta que precisa assiná-la, e ela fica inerte até que todas tenham assinado. **Assinar parcialmente** quer dizer assinar os seus próprios slots obrigatórios e deixar o de outra pessoa vazio: o bot assina como dono do token autorizando a transferência, mas não consegue assinar como fee payer, porque o fee payer é a chave de outra pessoa. O que o bot segura agora é uma transação completa em cada detalhe e válida em nenhum, como um contrato com uma linha ainda em branco para assinar. Ele codifica essa transação parcialmente assinada em base64 e repete a requisição original com ela no header PAYMENT-SIGNATURE.

**Tempo três, a liquidação.** O servidor não encosta na blockchain. Ele encaminha o payload para um **facilitador**, um serviço que expõe dois endpoints. `/verify` inspeciona a transação parcialmente assinada e responde a uma pergunta: se isto fosse completado e submetido, satisfaria os termos de pagamento? Valor certo, asset certo, rede certa, destinatário certo, memo intacto. Contra o SDK fixado (`@x402/svm` 2.23.0), ele responde ensaiando, não só lendo: o verificador preenche o slot do fee payer na cópia dele e *simula* a transação completa contra o estado atual da blockchain. Assinada, sim — submetida, nunca; uma simulação não move dinheiro e não gasta taxa. Aí `/settle` faz a parte irreversível: o facilitador acrescenta a signature de fee payer que faltava, a que bate com `extra.feePayer` do tempo um, e submete a transação agora totalmente assinada para a rede. O bot pagou o preço; o facilitador pagou a taxa. Isso é patrocínio de taxa, o mesmo movimento econômico que você vai reencontrar no checkout gasless do módulo 8, empacotado aqui como infraestrutura de protocolo.

**Tempo quatro, o recibo.** Liquidação confirmada, o servidor finalmente faz o que o bot pediu lá no começo: responde 200 com a cotação, mais um header PAYMENT-RESPONSE carregando os detalhes da liquidação. O bot pegou os dados dele, o lojista recebeu, e a troca inteira coube dentro de uma única requisição HTTP repetida. Sem criação de conta, sem emissão de chave de API, sem cartão cadastrado. Do ponto de vista do seu log, um 402 seguido milissegundos depois por um 200.

![Diagrama de sequência seguindo uma chamada medida desde um 402 de corpo vazio carregando o desafio dele no header PAYMENT-REQUIRED, passando pela signature parcial do agente, até os passos de verify e settle do facilitador e o header de recibo.](assets/v03-flowchart.png)

A coreografia de signatures é a parte que as pessoas erram na primeira leitura, então fixe ela. O cliente assina como dono e nunca como fee payer. `/verify` nunca submete: ele preenche o slot do fee payer numa cópia de rascunho puramente para simular, e uma simulação não consegue mover dinheiro. `/settle` é a única transmissão: signature de fee payer posta, transação para fora. Se você consegue recitar essa frase, consegue depurar metade das threads confusas sobre x402 que você vai ler na vida.

![Diagrama de quem assina, mostrando o agente preenchendo o slot de dono, o verify assinando uma cópia de rascunho puramente para simular, e o facilitador preenchendo o slot do fee payer para transmissão só no settle, enquanto o lojista não assina nada.](assets/v04-diagram.png)

Um número que você não vai encontrar aqui, de propósito. A spec obriga a transação de settle a carregar instruções de limite do ComputeBudget, e ela limita o preço da compute unit, mas não declara contagem nenhuma de compute units para uma liquidação, nenhuma. Uma cifra de "mais ou menos 20,000 CU por settle" circula assim mesmo, e quando fomos atrás dela pesquisando para este curso ela desmontou a favor do leitor: os 20,000 são reais, mas não são um custo. São o `DEFAULT_COMPUTE_UNIT_LIMIT` no SDK de referência (`@x402/svm` 2.23.0, lido em 2026-08-22), o teto que o cliente pede quando prefixa a instrução SetComputeUnitLimit, que é um orçamento que você pede, não uma fatura que você paga. Citar isso como consumo é como citar o limite do seu cartão como o seu aluguel. Então esta lição não imprime custo nenhum de CU para a transação de settle, e a sua documentação de API também não deveria. Se um número importa para você, meça nas suas próprias transações liquidadas e date a medição. Qualquer absoluto que você imprime precisa de uma fonte independente e datada, ou não deveria ser impresso. Essa regra está prestes a trabalhar bem mais pesado na seção de tráfego.

### O facilitador em quem você precisa confiar

Hora de ser adulto sobre o trade-off, porque a versão desmistificada do x402 não pode ser "e aí a mágica liquida". O facilitador é um terceiro parado dentro do seu caminho de pagamento, e você deveria ver a superfície de confiança sem desfoque nenhum. Ele vê toda transação antes da submissão. Ele segura a chave do fee payer, o que quer dizer que ele decide o que chega a ser submetido: um facilitador pode se recusar a liquidar, o que é censura quando você está do lado errado dela, e os hospedados rodam triagem KYT, checagens de compliance no nível da transação, por design. Isto não é um defeito que alguém esqueceu de corrigir. Patrocínio de taxa exige um fee payer, verificação exige um inspetor, e pôr os dois num serviço só é o que faz o lado do bot no fluxo ser um header. Você está comprando conveniência com confiança, a troca mais antiga dos pagamentos. O seu registro de decisão de corredores da lição passada deixou essa mesma troca explícita para as rampas fiat; a coluna do facilitador pertence à mesma tabela.

A adequação do esquema é o segundo limite honesto. exact é liquidação por chamada on-chain, o que é exatamente certo para uma API medida em que cada chamada vale dinheiro de verdade, e exatamente errado para streaming de alta frequência, milhares de eventos abaixo de um centavo por minuto, onde o custo de liquidar na blockchain a cada chamada domina o próprio pagamento. Quando a adequação por chamada quebra, é para isso que existem os esquemas upto e batch-settlement. Case o esquema com a medição, não o contrário.

Então quais são as opções de facilitador de verdade na Solana? Aqui está o panorama, e ele contém uma correção que vale nomear em voz alta. As notas iniciais de planejamento deste curso chutaram que a Helius estaria nesta lista. Não está: a Helius não aparece em lista nenhuma de facilitadores, e os trabalhos dela neste curso continuam sendo os de sempre, webhooks, RPC e os trilhos de assinatura do módulo 5. A lista que existe:

- **Corbits**, **PayAI** e **Solvador**: facilitadores Solana, cada um rodando o serviço de /verify-e-/settle.
- **Dexter**: a mesma cadeira, e de graça, o que faz dele a primeira parada óbvia para os experimentos de um lojista pequeno.
- **Coinbase CDP**: a opção hospedada da incumbente, com triagem KYT e OFAC embutida. Se a sua postura de compliance exige um caminho de liquidação com triagem, é esta a desenhada para esse requisito; se resistência à censura é a sua prioridade, a mesma feature se lê como bug.
- **Faremeter**: não é um facilitador hospedado e sim um framework open-source, e é notável por negociar v1 e v2 automaticamente, mais o MPP (o Machine Payments Protocol, a família de pagamento sobre HTTP-auth cuja spec de método Solana a Foundation escreve, e que você vai pôr no mesmo gate do x402 daqui a duas lições). Lembra do mundo de dois dialetos lá de cima? O Faremeter é o adaptador para viver nele.
- **O facilitador do x402.org**: só devnet e testnet. Perfeito para o lab que você está prestes a rodar, e uma cilada se você ligar produção nele.

![Tabela-lista das opções de facilitador x402 na Solana com as ressalvas de confiança e de rede de cada uma, mais uma linha corrigida anotando que a Helius não é facilitadora.](assets/v05-comparison.png)

Como escolher? Do mesmo jeito que você escolheu corredores na lição passada: nomeie a restrição que domina. Liquidação com triagem de compliance obrigatória, CDP. Orçamento zero e mainnet, a cadeira de graça, Dexter, depois da sua própria diligência sobre ele. Contrapartes misturadas de v1 e v2, Faremeter na frente de qualquer facilitador que liquide. Bancada de teste, a do x402.org, e nada mais. Não existe um vencedor geral, o que é o sinal mais saudável possível para um cenário tão jovem.

### Ler o tráfego sem inventar números

O x402 deixou de ser curiosidade em 2026, e os números são genuinamente grandes. Eles também são publicados em duas janelas diferentes em duas páginas diferentes, e o pecado analítico mais comum no comentário sobre pagamentos agênticos é misturar os dois. Você vai aprender as duas cifras com as fontes grampeadas nelas, porque quem faz engenharia de pagamentos e cita número de volume errado queima uma credibilidade que é lenta de reconstruir.

Cifra um, do painel do x402.org, uma janela móvel de 30 dias, consultada em 2026-08-21: 75.41 milhões de transações, 24.24 milhões de dólares em volume, 94.06 mil compradores, 22 mil vendedores. Fique um segundo com o formato disso, porque o formato é a história. Divida o volume pelas transações e o pagamento médio fica na casa dos 32 centavos. Isso não é gente comprando disco; é máquina comprando chamada de API, exatamente o tráfego de micropagamento por chamada para o qual o esquema exact foi desenhado. Uns noventa e quatro mil compradores contra vinte e dois mil vendedores te dizem que o lado comprador supera os vendedores em quatro para um, e, com honestidade, é só isso que ele te diz: uma contagem de compradores não fala nada sobre concentração, um punhado de bots pesados ainda poderia estar dirigindo a maior parte desses 75 milhões de chamadas, e o painel não publica esse corte. Repare no que o número não consegue sustentar antes de citá-lo.

Cifra dois, da página do x402 no solana.com: 37 milhões ou mais de transações na Solana, e uma afirmação de que a Solana carrega 70 por cento do volume mensal de x402. Página diferente, janela diferente, métrica diferente: isso se lê como uma foto acumulada na Solana com uma afirmação de participação de mercado anexada, não um total móvel de 30 dias.

Agora a disciplina, dita como uma regra que você consegue aplicar numa revisão de documentação. Cite cada cifra com a fonte dela e a data dela, e nunca combine as duas. Não some as duas; 75.41M mais 37M dá um número que nenhuma fonte na Terra sustenta. Não divida uma pela outra para derivar uma participação; as janelas não batem. E date tudo, porque as duas são números de painel ao vivo que variam diariamente; as cifras acima eram verdade em 2026-08-21 e já estão velhas enquanto você lê isto. Se dois números não foram medidos na mesma janela pela mesma fonte, eles não pertencem à mesma aritmética. Essa frase é a regra inteira.

![Dois cartões de fonte mantendo separados os totais de 30 dias do x402.org e as cifras acumuladas na Solana do solana.com, com um painel proibindo qualquer aritmética entre os dois.](assets/v06-comparison.png)

Quem está por trás desse tráfego importa tanto quanto o tamanho dele. A lista de parceiros do x402.org inclui AWS, Cloudflare, Stripe e Vercel, que é o establishment de infraestrutura, não uma torcida cripto-nativa. Histórias de migração já começaram: a atxp.ai mudou a stack dela para x402 mais MPP na Solana. E a concorrência chegou na forma mais lisonjeira possível, com a OKX lançando um protocolo rival de pagamentos entre máquinas que ela chama de APP. Padrões que ninguém usa não ganham concorrentes.

A Stripe merece um tempo só dela, porque a posição dela é o sinal mais claro que existe de onde as incumbentes acham que isso vai dar. Conte as frentes dela. Ela está na lista de trusted-by do x402.org. Ela coassinou a ACP, a spec de checkout agêntico, com a OpenAI. Como você viu no trabalho de corredores do módulo 6, ela opera um adquirente de USDC na Solana que liquida para lojistas em fiat. E com a Tempo Labs ela coassinou o `draft-httpauth-payment-00`, o esquema de autenticação HTTP "Payment" sobre o qual o MPP é construído, que você conhece daqui a duas lições. Uma incumbente, quatro cadeiras em quatro mesas diferentes do comércio nativo de máquinas. A Stripe não está apostando num vencedor; está comprando todos os páreos. Para a Wavelength a leitura é mais simples e mais útil: os trilhos que você está aprendendo neste módulo são os mesmos trilhos em torno dos quais a maior incumbente de pagamentos do planeta está se posicionando, e a sua API de preço de prensagem vai falar a versão em protocolo aberto deles na próxima lição.

![Diagrama de hub posicionando o x402 entre os parceiros dele na Linux Foundation, com um destaque para as quatro frentes da Stripe, sendo a quarta o esquema base de auth HTTP Payment por trás do MPP, e setas nas bordas para os desafiantes.](assets/v07-diagram.png)

## Lab: anote um 402 como se a spec estivesse olhando

O gate desta lição é anotação, não construção: pegue uma resposta 402 com formato v2 e rotule cada header e cada campo de PaymentRequirements com o papel dele, depois diga quem assina no /verify contra quem assina no /settle. Você mesmo vai gerar a resposta a partir de um servidor mock, então você também sente a ida e volta da cadeira do servidor. O apoio vem pronto; cada anotação é sua.

1. No diretório `~/wavelength/x402-lab` do começo da lição, crie `x402-mock.mjs`. Isto é um stub didático da camada de pagamento da API de preço de prensagem: ele fala o envelope de desafio v2 inteiro, os três campos de nível de cima mais um PaymentRequirements de sete campos, sobre os três headers v2. Ele não verifica nada; em produção quem faz a checagem é um facilitador de verdade, e a spec continua sendo a fonte da verdade para o formato do fio.

```js
// x402-mock.mjs - a v2-shaped 402 teaching stub. Zero dependencies.
import { createServer } from "node:http";

// The challenge. In v2 this never travels in the body: it is JSON, base64'd,
// and set as the PAYMENT-REQUIRED response header.
const paymentRequired = {
  x402Version: 2, // which dialect this challenge speaks
  error: "Payment required", // WHY the 402 happened; the only place a reason appears in the challenge
  resource: {
    // what the caller was trying to buy
    url: "http://localhost:4021/price",
    description: "Wavelength pressing-price quote",
    mimeType: "application/json",
  },
  accepts: [
    {
      scheme: "exact",
      network: "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", // CAIP-2: solana namespace + mainnet genesis-hash reference
      amount: "10000", // base units: 0.01 USDC at 6 decimals
      asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC mint
      payTo: "MerchantownerPubkeyGoesRightHere11111111111", // the merchant OWNER; the scheme derives the ATA
      maxTimeoutSeconds: 300, // how long these terms stay payable
      extra: {
        feePayer: "FaciLitatorFeePayerPubkeyGoesRightHere11111", // stand-in: the sponsor who signs LAST
        memo: "WVL-PRESS-0042", // invoice id for reconciliation; 256-byte ceiling
      },
    },
  ],
};

const b64 = (value) => Buffer.from(JSON.stringify(value)).toString("base64");

createServer((req, res) => {
  const proof = req.headers["payment-signature"]; // Node lowercases incoming header names
  if (!proof) {
    res.writeHead(402, {
      "Content-Type": "application/json",
      "Content-Length": "2",
      "PAYMENT-REQUIRED": b64(paymentRequired),
    });
    res.end("{}"); // the body is empty on purpose; everything is in the header
    return;
  }
  // A real server forwards `proof` to a facilitator: /verify inspects, /settle signs + submits.
  // This stub accepts anything, so the header choreography is visible end to end.
  const receipt = b64({
    success: true,
    transaction: "5xSettLedSignatureStandin",
    network: paymentRequired.accepts[0].network,
    payer: "AgentPubkeyStandin111111111111111111111111",
  });
  res.writeHead(200, { "Content-Type": "application/json", "PAYMENT-RESPONSE": receipt });
  res.end(JSON.stringify({ quote: { sku: "12in-180g-black", unitPriceUsd: 7.4 } }));
}).listen(4021, () => console.log("mock pressing-price API on :4021"));
```

2. Rode ele, depois toque o primeiro tempo do bot de um segundo terminal:

```bash
node x402-mock.mjs
```

```bash
curl -i http://localhost:4021/price
```

```text
HTTP/1.1 402 Payment Required
Content-Type: application/json
Content-Length: 2
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6MiwiZXJyb3IiOiJQYXltZW50IHJlcXVpcmVkIiwicmVzb3VyY2Ui...
Date: Fri, 21 Aug 2026 16:41:09 GMT
Connection: keep-alive
Keep-Alive: timeout=5

{}
```

Fique um instante com essa tela cheia: um 402, dois bytes de corpo e um header carregando várias centenas de caracteres de base64, elididos acima nas reticências. Decodifique e os termos aparecem:

```bash
curl -sD - -o /dev/null http://localhost:4021/price \
  | grep -i '^payment-required:' | sed 's/^[^:]*: *//' | tr -d '\r' \
  | base64 -d | node -p "JSON.stringify(JSON.parse(require('fs').readFileSync(0,'utf8')),null,2)"
```

```json
{
  "x402Version": 2,
  "error": "Payment required",
  "resource": {
    "url": "http://localhost:4021/price",
    "description": "Wavelength pressing-price quote",
    "mimeType": "application/json"
  },
  "accepts": [
    {
      "scheme": "exact",
      "network": "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
      "amount": "10000",
      "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "payTo": "MerchantownerPubkeyGoesRightHere11111111111",
      "maxTimeoutSeconds": 300,
      "extra": {
        "feePayer": "FaciLitatorFeePayerPubkeyGoesRightHere11111",
        "memo": "WVL-PRESS-0042"
      }
    }
  ]
}
```

Este é o momento exato em que um agente pagador começa a ler, e repare onde ele lê.

3. Toque o tempo do retry. O valor do header aqui é um blob de mentira, não uma transação parcialmente assinada de verdade; o /verify de um facilitador rejeitaria na hora, o que é uma coisa boa de provar para você mesmo depois, no facilitador só-devnet do x402.org:

```bash
curl -i -H "PAYMENT-SIGNATURE: c3R1Yg==" http://localhost:4021/price
```

Você deve ver `HTTP/1.1 200 OK`, o corpo com a cotação e um header `PAYMENT-RESPONSE` carregando um recibo em base64.

4. Agora o gate de verdade. Crie `annotations.md` e rotule, com as suas próprias palavras, uma linha para cada, partindo do fio para fora: os três headers PAYMENT-REQUIRED, PAYMENT-SIGNATURE e PAYMENT-RESPONSE; os três campos de nível de cima do desafio `x402Version`, `error` e `resource`; e os sete campos de requisito `scheme`, `network`, `amount`, `asset`, `payTo`, `maxTimeoutSeconds` e `extra` (divida esse último em `feePayer` e `memo`), cada um com o papel dele no fluxo. Nada de copiar frases desta lição; o ponto é que os rótulos sobrevivam nas suas palavras.

5. Feche o arquivo com a nota de duas linhas que o gate exige, respondendo com precisão: quais signatures existem antes de o facilitador encostar na transação, o que o /verify faz com elas, e qual signature única o /settle acrescenta antes de submeter.

6. Autochecagem contra a seção do fluxo. A condição de aprovação: um colega de time que nunca viu x402 conseguiria ler o seu `annotations.md` ao lado da saída do curl e prever corretamente o que um facilitador faria com um pagamento de verdade.

## Challenge

A Wavelength vai precisar de uma decisão de facilitador antes da construção da próxima lição, então rascunhe ela agora, cinco linhas, no mesmo formato do registro de decisão de corredores: uma linha nomeando a restrição dominante para um lojista pequeno medindo uma API de preço de prensagem na mainnet, uma linha para a sua escolha principal com o motivo, uma linha para a alternativa de compliance e quando você trocaria para ela, uma linha para o que você usa em CI e por que aquilo nunca pode ser a configuração de produção, e uma linha declarando a confiança que você está aceitando, com as suas próprias palavras, com base na seção da superfície de confiança. Não existe uma resposta certa única; existe uma defensável, e na próxima lição você vai construir contra a que você escolheu.

Meta extra, se o mundo de dois dialetos te incomodou tanto quanto deveria: acrescente cinco linhas ao `x402-mock.mjs` que detectem um header `X-PAYMENT` chegando e respondam o 402 de sempre, corpo vazio e tudo, com o campo `error` do desafio reescrito para nomear o header v2 que o cliente deveria ter mandado. Mande esse fio de alarme pelo canal que um cliente v2 já está lendo e você terá construído a passagem de v1 para v2 mais amigável do ecossistema.

## Checkpoint: o que você já consegue fazer

Se a anotação brigou com você em algum ponto, o nó costuma estar num de dois lugares. Confundir qual parte assina no /settle quer dizer reler o tempo três; o lojista nunca assina, e o /verify nunca submete (a signature de fee payer dele vive numa cópia de rascunho, puramente para simular), então existe exatamente um lugar por onde o dinheiro pode sair: /settle. E se o seu rótulo de `extra.feePayer` diz algo como "a conta de onde o bot paga as taxas", é o cérebro-v1 falando: o ponto inteiro é que o bot não paga taxas, quem paga é o patrocinador nomeado naquele campo.

Aqui está o que você entrou sem ter e sai segurando. Você consegue ler uma resposta 402 v2 de cara, e começa no lugar certo, porque sabe que o corpo nunca vai te dizer nada e que o header PAYMENT-REQUIRED te diz tudo, até por que a requisição falhou. Você consegue nomear cada peça móvel do desafio que esse header carrega. Você consegue traçar um pagamento entre máquinas do desafio ao recibo e dizer com precisão onde a confiança fica e quem assina o quê. Você consegue nomear as opções reais de facilitador na Solana, incluindo a que não está na lista por mais vezes que você ouça chutarem ela. E você consegue citar tráfego de x402 sem cometer o pecado do número misturado, o que te põe à frente da maioria das pessoas que escrevem sobre este protocolo para viver. Nada mal para uma lição em que a única coisa que você deployou foi um mock de cinquenta linhas.

Na próxima lição o seu cliente é um bot. Você põe preço na API de preço de prensagem da Wavelength e constrói o agente que paga por ela, chamada por chamada: o 402 que você mockou hoje vira um desafio de verdade, o header do stub vira uma transação parcialmente assinada de verdade, e a coluna do facilitador do seu registro de decisão é descontada.
