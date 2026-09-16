# Onramps, e o caminho de volta

## Resumo

O módulo de assinaturas fechou com a máquina de estados do dunning transformando uma renovação falha numa fatura em aberto; o clube do disco do mês agora se vira sozinho. Mas todo degrau, da primeira transferência até o clube, assumiu que o USDC já tinha chegado numa carteira. Numa feira de discos de verdade, metade do público tem cartão e nenhuma cripto, e um dos seus artistas quer sacar os royalties do mês passado para uma conta bancária. O dinheiro precisa entrar, e precisa voltar a sair. Nenhuma das duas direções é sua para custodiar, e esta lição é sobre ligar as duas sem nunca encostar em nenhuma.

Antes de qualquer teoria, rode este one-liner no seu terminal (o `node` está na sua máquina desde o módulo 1):

```bash
node -e "const u = new URL('https://pay.example/buy?address=BuyerWa11etXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX&asset=USDC'); u.searchParams.set('address', 'AttackerXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'); console.log(u.toString())"
```

Uma linha de JavaScript acabou de reescrever para onde o dinheiro de um comprador iria, em qualquer fluxo descuidado o bastante para carregar o destino numa URL. Essa é a lição de segurança do dia inteira, em miniatura: qualquer coisa numa URL de cliente é gravável pelo atacante, então o endereço de destino nunca pode viajar numa. A correção se chama session token, e ela é a espinha dorsal de tudo o que a gente constrói abaixo.

As descobertas logo de cara:

- **O caminho de entrada** é o fluxo headless embutido do Coinbase Onramp. O seu servidor emite um **session token** de uso único que vincula o endereço de carteira do comprador e o ativo a receber; a URL do cliente carrega só esse token. Uma URL adulterada não consegue redirecionar dinheiro, porque o endereço não está na URL para ser adulterado.
- Desde 31 de julho de 2025, toda URL do Coinbase Onramp e do Offramp precisa ser inicializada com um session token. A era do endereço-na-URL acabou por determinação, não só por bom gosto.
- **O onramp fiat-para-cripto da Stripe** é o outro formato de integração: a Stripe é merchant-of-record, engolindo fraude, disputas e KYC. Ele está em preview público, o que quer dizer que os limites dele são os seus limites.
- **O caminho de saída** é o Coinbase Offramp, e ele é hosted-only: você redireciona o usuário para o fluxo da própria Coinbase em vez de embutir. O trilho de payout depende de onde o artista está: ACH é um trilho bancário dos EUA, o PayPal atende uma lista de países selecionados, e o cardápio por região ao vivo é config que você busca, não fato que você decora.
- A distinção estrutural do módulo: uma processadora liquidando as suas vendas num saldo em fiat e um usuário fazendo offramp do próprio USDC para o banco dele são dois atores diferentes com duas superfícies de KYC diferentes. Saber qual é qual em cada costura é todo o seu trabalho de compliance como dev. E isso também não é aconselhamento jurídico, e eu vou repetir isso onde importa.

O artefato é o `ramp-embed`, nascido do mesmo workspace `wavelength-checkout`: uma rota de servidor que emite o session token, uma passagem de bastão para o cliente que abre a URL do onramp, e um smoke test que prova que o endereço nunca vaza. Como o trabalho de hoje está dividido: o handler de sessão chega como um esqueleto com dois buracos de TODO bem barulhentos, e a teoria contém as duas respostas na íntegra; o offramp é um passo a passo guiado, porque não dá para automatizar de forma útil o fluxo de KYC hospedado de outra pessoa; e o desafio de código solo te entrega uma integração que funciona mas vaza, para consertar, em vez de um arquivo em branco.

![O checkout existente fica no centro, com um comprador de cartão entrando pelo Coinbase Onramp embutido e um artista saindo pelo Coinbase Offramp hospedado até um banco.](assets/v01-diagram.webp)

## A fronteira fiat

### Os ramps, e quem eles atendem

Um ramp é uma casa de câmbio com um departamento de compliance. A direção **onramp**, a porta de entrada para a moeda fiduciária, pega um pagamento de cartão ou bancário de um usuário e entrega cripto num endereço que ele indica; a direção **offramp**, a porta de saída, pega a cripto dele e entrega fiat numa conta bancária que ele possui. Nas duas direções o cliente do ramp é a pessoa que quer o ativo trocado, não você. A Wavelength nunca segura o número do cartão, nunca segura o fiat e nunca segura o USDC do comprador. Você está ligando uma porta, não um cofre.

Existem dois formatos de integração, e o vocabulário importa para o resto do módulo. Uma integração **hosted** quer dizer que você redireciona o usuário para as páginas do próprio provedor e ele volta quando termina. Uma integração **headless** (ou embutida) quer dizer que o fluxo do provedor roda dentro do seu produto, com o seu estilo e as suas telas, enquanto o provedor continua tocando o dinheiro e o KYC por baixo. Hosted é uma indicação; headless é um componente. O Coinbase Onramp oferece o caminho headless, e é por isso que é ele que ganha a construção hoje. O Coinbase Offramp, como a gente vai ver, deliberadamente não oferece.

Por que uma loja de discos se importa com isso? Porque a matemática do comprador é brutal. Toda lição até aqui assumiu uma carteira que já tinha USDC, o que numa feira física descreve talvez a fatia cripto-nativa do público. Todo o resto está com um cartão na mão. Se a resposta para "dá para eu pagar?" é "primeiro vá criar uma conta numa exchange, faça o KYC lá, compre USDC, saque para uma carteira que você também precisa instalar, e depois volte aqui", você não perdeu uma venda, você perdeu o segmento inteiro. O onramp comprime isso em: toca em comprar, Apple Pay, o USDC aparece na carteira, paga o checkout. Os mesmos trilhos que você já construiu. Público novo.

E isso deixou de ser hipotético para o consumidor faz um tempo. A onda de trilhos de consumo de abril de 2026 levou o padrão para o mainstream: a Meta começou a pagar criadores em USDC na Solana na Colômbia e nas Filipinas, o MetaMask Card começou a gastar USDC da Solana sobre a Mastercard em terminais comuns, e a Solflare entregou onramps do Coinbase com Apple Pay direto dentro da carteira (as três notícias vêm do resumo de ecossistema de abril de 2026 da Solana Foundation no solana.com, o mesmo resumo que lições posteriores citam; são afirmações datadas, então confira de novo antes de repetir). O encanamento que você está prestes a construir é o mesmo encanamento, uma vitrine menor.

![Uma linha do tempo de três trilhos de consumo de abril de 2026, os payouts de criadores da Meta, os gastos do MetaMask Card e os onramps com Apple Pay da Solflare, com setas apontando para o embed de vitrine desta lição.](assets/v02-timeline.webp)

### O session token: vincule no servidor, nunca vaze

Aqui está a espinha dorsal didática da lição, e ela é uma propriedade de segurança de verdade, não boilerplate de integração.

O embed ingênuo é igual ao one-liner que você rodou lá em cima: monte uma URL no cliente, anexe o endereço de carteira do comprador e o seu app ID como query params, abra. Ele até funciona. O problema é que uma URL é dado na mão do atacante. Uma extensão de navegador maliciosa, uma dependência comprometida, um man-in-the-middle numa rede ruim, qualquer um deles consegue reescrever `address=` antes de a janela abrir, e o cartão do comprador financia alegremente a carteira de um estranho. O comprador culpa a sua loja, e a disputa cai em algum lugar caro. Vou ser honesto: a primeira integração de onramp que eu esbocei na vida tinha o endereço na query string, porque todo quickstart daquela época tinha. A indústria aprendeu a fazer melhor em público.

O fluxo de session token tira o endereço da superfície de ataque por completo. Três passos:

1. O seu **servidor** chama a API de session token da Coinbase, autenticado com a sua chave de API do CDP, e o corpo da requisição vincula o destino: o endereço de carteira do comprador, as blockchains em que ele vale, e opcionalmente quais ativos a sessão pode receber.
2. A Coinbase devolve um **session token**: uma credencial de uso único, que expira em cinco minutos, e que referencia internamente tudo o que a requisição vinculou.
3. O **cliente** abre a URL do onramp carregando só esse token. O endereço nunca aparece na URL, então não tem o que adulterar. Reescreva o token e a sessão simplesmente falha; ela não pode ser redirecionada, só quebrada.

Seja preciso sobre o que o token protege e o que ele não protege, porque uma afirmação de segurança que se estica demais convida a própria refutação de trinta segundos. O token tira o endereço de tudo o que vem depois da emissão: a URL, a janela aberta, qualquer link copiado, logado ou vazado, que é onde destinos vivem mais tempo e são reescritos com mais facilidade. O que ele não faz é autenticar a intenção. A rota `/session` do lab ainda aceita o endereço vindo de um POST do cliente, então um atacante capaz de reescrever requisições dentro do navegador do comprador poderia adulterar um salto antes, no corpo em vez de na URL. Uma vitrine de produção fecha esse salto vinculando o endereço a algo que o cliente não consegue forjar, uma sessão logada cuja carteira foi ligada no cadastro, ou uma signature de carteira conectada provando controle do endereço; o lab deixa essa camada de auth de fora porque ela pertence ao seu app, não ao ramp. Afirmação mais estreita, e ainda assim vale a construção.

A propriedade para guardar: **vincule no servidor, nunca vaze**. O valor sensível mora numa chamada autenticada de servidor para servidor; o cliente carrega uma referência opaca. Se você já usou os PaymentIntents da Stripe, este é o mesmo formato (um intent criado no servidor, um secret do lado do cliente que referencia ele), e essa sobreposição é a resposta padrão para "o cliente quer iniciar um fluxo que o cliente não pode dirigir".

![Um fluxo de quatro saltos em que o cliente pede uma sessão, o seu servidor vincula o endereço dentro de um token da Coinbase, e o cliente abre uma URL em que a adulteração morre num beco sem saída.](assets/v03-flowchart.webp)

Os formatos concretos, verificados contra a documentação ao vivo da Coinbase hoje, são pequenos o bastante para decorar. A emissão é um POST para `https://api.developer.coinbase.com/onramp/v1/token` com um JWT Bearer gerado a partir da sua chave de API do CDP, e o corpo que vincula um destino USDC na Solana é exatamente este:

```json
{
  "addresses": [{ "address": "<buyer wallet address>", "blockchains": ["solana"] }],
  "assets": ["USDC"]
}
```

A resposta é um `{ "token": "...", "channel_id": "..." }` plano (`channel_id` é metadado do fluxo de guest checkout da Coinbase; o caminho do widget precisa só de `token`, e é por isso que a rota de servidor do lab descarta o resto). E a URL de cliente que a sua vitrine abre é montada a partir do token mais os defaults de exibição:

```
https://pay.coinbase.com/buy/select-asset
  ?sessionToken=<token>
  &defaultNetwork=solana
  &defaultAsset=USDC
  &presetFiatAmount=12.5
  &fiatCurrency=USD
```

Leia essa URL duas vezes e note o que falta: nenhum endereço, nenhum app ID. `defaultNetwork` e `defaultAsset` são presets de experiência de usuário (eles escolhem em qual tela de ativo o widget abre), e `presetFiatAmount` pré-preenche a compra com o preço do disco, para o comprador cair numa tela que já diz o número certo. Nenhum deles é relevante para segurança. O único param estrutural é `sessionToken`, e ele é opaco. Essa assimetria, presets chatos na URL, o vínculo sensível atrás do token, é o design.

![A URL do onramp anotada linha a linha, com o sessionToken marcado como estrutural, quatro presets de exibição marcados como cosméticos, e o endereço de carteira e o app ID ausentes por design.](assets/v04-annotated-code.webp)

Os dois tempos de vida do token também fazem parte da propriedade, não são curiosidade. Uso único quer dizer que uma URL capturada não pode ser repetida para abrir uma segunda sessão de financiamento contra o mesmo vínculo, e a expiração de cinco minutos quer dizer que um link vazado morre antes de conseguir circular. O seu servidor emite por clique, no momento da intenção. Faça cache de um session token do jeito que você faria cache de uma cotação de preço e o melhor caso é um link morto, expirado ou já consumido, servido a um comprador real no momento da compra; o pior caso é um vínculo emitido para um comprador entregue a outro. A emissão custa uma ida e volta autenticada, então não tem nada que valha a pena economizar.

![A vida de um session token, da emissão por clique passando por um único uso e pela expiração de cinco minutos, com tokens repetidos e cacheados mostrados morrendo em becos sem saída fora da linha.](assets/v05-timeline.webp)

Uma nota prática sobre o que o comprador recebe. O destino que você vincula é o endereço de carteira do comprador, e a Coinbase entrega USDC na conta de token associada derivada dele, a mesma derivação de ATA que você aprendeu quando a Wavelength recebeu USDC pela primeira vez no módulo 2. O comprador não precisa pré-criar nada. Ele sai do fluxo segurando exatamente o saldo que o seu checkout sabe cobrar. O USDC de mainnet na Solana é o mint `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`; o seu checkout de devnet cobra o mint substituto da devnet, e é por isso que a execução ao vivo do lab é uma sessão em sandbox e não uma de devnet. Onramps são um produto de mainnet. Cartões de verdade compram dólares de verdade.

Mais duas peças da superfície da Coinbase pertencem ao seu mapa, as duas no nível de conceito, porque o caminho do widget acima é o que a gente constrói. Primeiro, o **caminho headless do Apple Pay**: além do widget, a Onramp Order API deixa você ir totalmente headless. O seu servidor cota e cria a ordem e recebe de volta um link de pagamento, você renderiza esse link numa webview ou num iframe, e o que o comprador vê é o botão de Apple Pay ou Google Pay da Coinbase sentado dentro da sua própria UI. Uma correção que vale carregar, porque é fácil supor o contrário: esse caminho autentica com a mesma chave de API do CDP assinada num JWT Bearer, mas ele não recebe session token. Session tokens inicializam as URLs hospedadas do widget; a order API é uma chamada de API autenticada comum. Quando a sua vitrine ficar grande demais para as telas do widget, é essa a porta; confira a referência de API atual quando você passar por ela, porque a superfície headless é a parte do produto que se move mais rápido. Segundo, o **modo trial**: a documentação de onboarding da Coinbase descreve um nível trial para integrações novas, com tamanhos de transação limitados antes da aprovação completa. Trate isso como recurso e não como incômodo: é o que dá a este lab uma compra com cartão de ponta a ponta de verdade, com rodinhas nos valores. Para o que o seu app está liberado mora no dashboard do CDP, não nesta lição.

### O outro formato: a Stripe como merchant-of-record

O fluxo headless da Coinbase não é o único jeito de parafusar uma porta de fiat numa vitrine, e a alternativa ensina a segunda palavra de vocabulário da lição por contraste.

**Merchant-of-record** é a entidade cujo nome está na cobrança: aquela que a bandeira responsabiliza, aquela contra quem a disputa é aberta, aquela que precisa conhecer o próprio cliente. A Stripe toca o onramp fiat-para-cripto dela, hoje em preview público, com a própria Stripe como merchant-of-record. Embuta o onramp da Stripe e a compra de cripto no cartão é venda da Stripe, não sua. A Stripe toca o KYC, a Stripe engole a fraude, a Stripe absorve o chargeback quando um comprador contesta a cobrança três dias depois. O que você entregou em troca foi controle: a cobertura geográfica da Stripe é a sua cobertura geográfica, a lista de ativos da Stripe é a sua lista de ativos, e preview público quer dizer que as duas podem mudar debaixo de você, com o rótulo de preview como o seu único aviso. Os tetos específicos, quais países, quais ativos, quais valores, moram atrás da documentação de preview atual da Stripe e se movem sem aviso, que é exatamente por que esta lição não imprime eles; a próxima lição transforma vá-ler-a-matriz-atual-do-fornecedor num hábito avaliado em vez de um dar de ombros.

Para sentir por que essa cadeira importa, rode um cenário de brinquedo com números redondos, ilustrativos e inventados de propósito: a feira vende 1,000 discos a US$ 12.50 por onramps financiados por cartão, a fraude de cartão não presente roda em uns 1%, e as bandeiras cobram do merchant-of-record uma taxa fixa de disputa em cada uma, muitas vezes maior que a própria venda; digamos US$ 15.

```typescript
// dispute-math.ts - who eats the chargebacks? Whoever is merchant-of-record.
const sales = 1_000; // records sold
const price = 12.5; // USD each
const disputeRate = 0.01; // card-not-present, typical-ish
const feePerDispute = 15; // fixed network fee, USD - often more than the sale

const disputes = sales * disputeRate;
const reversed = disputes * price;
const fees = disputes * feePerDispute;
console.log(`${disputes} disputes -> $${reversed} reversed sales + $${fees} in network fees`);
// 10 disputes -> $125 reversed sales + $150 in network fees
```

Dez compras contestadas. Quem senta na cadeira fica no prejuízo dos US$ 125 de vendas revertidas mais US$ 150 em taxas mais o tempo de ops de brigar dez disputas, e se a taxa de disputa subir o bastante, as bandeiras colocam a conta inteira num programa de monitoramento. Agora note quem é esse quem: não é você. A compra de cripto foi venda da Coinbase ou venda da Stripe, e a máquina de disputas mastiga elas. O que você vendeu foi um disco, pago em USDC que já tinha compensado com a finality da Solana embaixo. Essa é a razão silenciosa e estrutural pela qual uma vitrine liquidada em cripto quer um parceiro de ramp na frente dela em vez do próprio adquirente de cartão: a máquina de chargeback continua existindo, ela só não está apontada para você.

O instinto aqui é perguntar qual formato ganha, e é um instinto levemente errado. Os dois formatos colocam o provedor na cadeira de merchant-of-record para a compra de cripto; você está escolhendo profundidade de integração e superfície de provedor, não responsabilidade. A comparação honesta:

![O Coinbase Onramp e o onramp da Stripe comparados: os dois fazem do provedor o merchant-of-record e o dono do KYC; eles diferem em formato de integração, status do produto, e em qual dos dois esta lição usa para construir.](assets/v06-comparison.webp)

O padrão generaliza para além destes dois fornecedores, que é por que vale internalizar agora: quem é merchant-of-record é dono da fraude, das disputas e das verificações de identidade, e em troca é dono do mapa de cobertura. Você vai encontrar a mesma troca com um conjunto diferente de fornecedores na próxima lição, e no fim dela, "quem é merchant-of-record aqui?" deveria ser a primeira pergunta que você faz a qualquer fornecedor de pagamentos, logo antes de "e em quais corredores?".

### O caminho de volta

Agora o artista com os royalties do mês passado. A Wavelength paga os artistas dela do jeito que ela é paga, em USDC para uma carteira que o artista controla, então o saldo de royalties já está sentado na custódia dele, nas chaves dele, sem nada seu sobrando ali dentro. O passo que falta é do USDC até uma conta bancária, e a ferramenta é o Coinbase Offramp.

A restrição de integração que molda tudo: **o Offramp é hosted-only.** Não existe embed headless para o caminho de saída. O seu produto emite um session token exatamente como antes, mesmo endpoint, mesma disciplina de vínculo, e depois redireciona o artista para o fluxo hospedado da própria Coinbase em `https://pay.coinbase.com/v3/sell/input`, carregando o `sessionToken`, um `partnerUserRef` (a sua referência opaca por usuário, com menos de 50 caracteres) e uma `redirectUrl` num domínio seu na allowlist, para onde a Coinbase manda o artista de volta quando termina. Dentro do fluxo hospedado, o artista se autentica com a Coinbase, o KYC roda como relação dele com a Coinbase e não com você, ele manda o USDC, e o payout cai no trilho de saque que a Coinbase oferecer onde o artista está. Esse cardápio é regional, e a documentação da própria Coinbase declara ele exatamente no formato em que você deveria anotar: **transferências bancárias ACH (EUA)** e **PayPal (países selecionados)**, mais um saldo Coinbase simples, com uma API de config devolvendo a lista por país (reconferido em 2026-09-02). Um artista dos EUA e um artista de São Paulo caminhando pelo mesmo fluxo veem saídas diferentes, então nunca prometa um trilho específico na sua UI de payout; prometa a caminhada. Depois ele volta para a sua `redirectUrl` e o seu produto retoma o fio.

Por que a Coinbase embutiria o caminho de entrada e hospedaria o caminho de saída? Siga o risco. Fraude de onramp é fraude de cartão, um problema que os provedores conseguem precificar e engolir em escala. O offramp é onde a lavagem de dinheiro sai para o sistema bancário, e o provedor quer esse fluxo inteirinho nas páginas dele, sob a sessão dele, sem nenhuma UI controlada por parceiro em volta. Você perde a UX embutida para a saída; em troca o compliance de payout nunca encosta no seu produto. Como troca, aceite, todas as vezes.

![A caminhada do offramp, em que o seu produto emite um token e redireciona para fora, e depois disso o login, o KYC, o envio do USDC e o payout bancário acontecem todos nas páginas da Coinbase.](assets/v07-flowchart.webp)

### Dois fluxos que parecem iguais e não são

Aqui está a distinção que este módulo não vai deixar você borrar, porque borrar ela é como desenvolvedores se convencem a assumir superfícies de compliance que eles não têm, ou pior, a abrir mão de superfícies que eles têm.

**Liquidação em fiat do lojista** é uma processadora agindo por você, o lojista: ela aceita as suas vendas e liquida elas num saldo em fiat no seu nome. O KYC que importa ali é a processadora fazendo o onboarding *do seu negócio* (as verificações de conheça-seu-negócio que você assina quando abre a conta), e a processadora está na cadeira de merchant-of-record ou de adquirente para essas vendas. **Um offramp de usuário** é o comprador ou o artista agindo por conta própria: o ativo dele, a conta bancária dele, a verificação de identidade dele com o provedor do ramp. Ator diferente, direção diferente, superfície de KYC diferente, responsabilidade diferente. O offramp que você acabou de caminhar é do segundo tipo. O fluxo de liquidação da Stripe que você vai encontrar na próxima lição é do primeiro tipo. Os dois "transformam cripto em fiat", e essa frase é exatamente o borrão a recusar: nomeie o ator e os fluxos se separam limpo.

A sua superfície de compliance como dev cabe honestamente numa frase: você precisa conseguir dizer, em cada costura do seu produto onde fiat e cripto se tocam, qual ator está movendo dinheiro e quem é merchant-of-record para esse movimento. Isso é um trabalho de descrever, não de operar. Você não toca nenhum dos dois fluxos. E para falar claro, porque este canto do curso encosta em território regulado: esta é uma leitura de engenharia de onde as costuras estão, não aconselhamento jurídico, e um produto de serviços financeiros de verdade sobe com um advogado de verdade.

![Três costuras mapeadas, com a Coinbase como merchant-of-record tanto do onramp quanto do offramp dela, a Stripe do onramp dela, e a Wavelength descrevendo todas as costuras sem operar nenhuma.](assets/v08-diagram.webp)

### O que eu verifiquei, e no que você não pode confiar em mim

> **Verificado na escrita, 2026-08-23.** A API de session token (`POST https://api.developer.coinbase.com/onramp/v1/token`, auth por JWT Bearer, corpo `addresses`/`blockchains`/`assets`, resposta plana `{ token }`, uso único, expiração de cinco minutos), a obrigatoriedade do session token em todas as URLs de Onramp e Offramp desde 2025-07-31, a URL base do offramp e os params `sessionToken`, `partnerUserRef` e `redirectUrl` que ela exige, ACH mais PayPal entre os métodos de saque do offramp, o host de sandbox, e o fato de que a Order API headless autentica com um JWT do CDP e não recebe session token — tudo isso foi conferido contra a documentação de desenvolvedor ao vivo da Coinbase nesta data.
>
> **Deliberadamente não congelado aqui: a cobertura de ativos e geográfica da Coinbase na Solana.** Quais ativos o onramp vende na Solana, em quais países e estados americanos, com quais limites, é configuração de provedor que se move sem aviso, e qualquer lista impressa num curso está velha na semana seguinte. Na hora da integração, consulte: as Onramp APIs expõem um endpoint de buy-options (`GET https://api.developer.coinbase.com/onramp/v1/buy/options?country=US`, mesmo host e mesma auth por JWT Bearer da emissão do token) que devolve a matriz ao vivo de ativos e meios de pagamento para um país dado. Trate cobertura como config que você busca, nunca como fato que você decora. Um corredor que o provedor não atende é um corredor que você não consegue atender, e você quer descobrir isso por uma resposta de API em staging, não por um comprador em produção.

Essa caixa também é a troca da lição declarada honestamente, então não vou enterrar ela: ramps te entregam alcance ao preço do controle. A processadora é dona do KYC, da cobertura geográfica, dos trilhos de payout, do status de preview e dos limites de trial, e cada um desses é um teto no seu produto que você não pode negociar em código. Embutir um ramp também planta uma costura de compliance dentro da sua vitrine que você precisa conseguir descrever mesmo sem operar ela. A alternativa, virar você mesmo o transmissor de dinheiro regulado, é tão pior para uma loja de discos que a troca mal merece o nome. Mas é uma troca, e a caixa de cobertura é onde ela morde.

## Lab: ligue a porta do fiat

Construção numerada, no workspace `wavelength-checkout` que você criou no módulo 3. `ramp-embed/` é uma pasta dentro do `wavelength-checkout`, não um workspace novo: ela entra ao lado da pasta `checkout/` que o módulo 3 construiu ali, que é o que deixa ela importar o preço do disco direto; os workspaces de ops e de billing dos módulos 4 e 5 ficam em outro lugar do repositório e não entram na história hoje. O handler de sessão chega com dois buracos de TODO, e a construção roda até uma falha nomeada com eles no lugar; o Challenge fecha eles. Uma dependência nova, necessária só para a rota de servidor (o smoke test roda limpo sem ela):

```bash
npm install @coinbase/cdp-sdk@1.55.0
```

Esse é o CDP SDK da Coinbase (linha 1.x em agosto de 2026), usado aqui para exatamente uma coisa: gerar o JWT de vida curta que autentica o seu servidor no endpoint do token. Assinar eles na mão é possível e não vale a pena.

1. **O módulo de sessão, com os dois buracos.** Crie o `ramp-embed/session.ts`. Funções puras, sem I/O, que é o que torna o smoke test possível:

   ```typescript
   export interface SessionTokenRequest {
     addresses: { address: string; blockchains: string[] }[];
     assets?: string[];
   }

   export interface OnrampUrlOptions {
     presetFiatAmount: number;
     fiatCurrency?: string;
     defaultAsset?: string;
   }

   // The server-side binding: this body is what pins the destination.
   export function buildSessionRequest(destinationAddress: string): SessionTokenRequest {
     // TODO(completion): return the body that binds destinationAddress to
     // USDC on Solana. The exact shape appears in the theory section.
     throw new Error('TODO: buildSessionRequest');
   }

   // The client handoff: the URL carries the token and display presets ONLY.
   export function buildOnrampUrl(sessionToken: string, opts: OnrampUrlOptions): string {
     // TODO(completion): build the pay.coinbase.com URL. sessionToken,
     // defaultNetwork, defaultAsset, presetFiatAmount, fiatCurrency.
     // The address does not appear. The theory section shows every param.
     throw new Error('TODO: buildOnrampUrl');
   }
   ```

2. **A rota de servidor.** Crie o `ramp-embed/server.ts`. Mesmo formato `node:http` puro do servidor de checkout, uma rota POST: o cliente manda o endereço de carteira do comprador, o servidor vincula ele num session token e responde com a URL de onramp pronta. As chaves vêm do dashboard do CDP (crie uma chave de API dentro do seu projeto; ela tem um ID e um secret) e moram em variáveis de ambiente, nunca no código:

   ```typescript
   import { createServer } from 'node:http';
   import { generateJwt } from '@coinbase/cdp-sdk/auth';
   import { RECORD } from '../checkout/record.ts';
   import { buildOnrampUrl, buildSessionRequest } from './session.ts';

   const KEY_ID = process.env.CDP_API_KEY_ID;
   const KEY_SECRET = process.env.CDP_API_KEY_SECRET;

   async function mintSessionToken(destinationAddress: string): Promise<string> {
     if (!KEY_ID || !KEY_SECRET) {
       throw new Error('set CDP_API_KEY_ID and CDP_API_KEY_SECRET');
     }
     const jwt = await generateJwt({
       apiKeyId: KEY_ID,
       apiKeySecret: KEY_SECRET,
       requestMethod: 'POST',
       requestHost: 'api.developer.coinbase.com',
       requestPath: '/onramp/v1/token',
       expiresIn: 120,
     });
     const res = await fetch('https://api.developer.coinbase.com/onramp/v1/token', {
       method: 'POST',
       headers: {
         Authorization: `Bearer ${jwt}`,
         'Content-Type': 'application/json',
       },
       body: JSON.stringify(buildSessionRequest(destinationAddress)),
     });
     if (!res.ok) {
       throw new Error(`session token mint failed: ${res.status} ${await res.text()}`);
     }
     const body = (await res.json()) as { token: string };
     return body.token;
   }

   const server = createServer((req, res) => {
     if (req.method !== 'POST' || req.url !== '/session') {
       res.writeHead(404);
       res.end();
       return;
     }
     let raw = '';
     req.on('data', (chunk) => {
       raw += chunk;
     });
     req.on('end', async () => {
       try {
         const { address } = JSON.parse(raw) as { address: string };
         const token = await mintSessionToken(address);
         const url = buildOnrampUrl(token, { presetFiatAmount: RECORD.priceUsdc });
         res.writeHead(200, { 'Content-Type': 'application/json' });
         res.end(JSON.stringify({ url }));
       } catch (err) {
         res.writeHead(500, { 'Content-Type': 'application/json' });
         res.end(JSON.stringify({ error: (err as Error).message }));
       }
     });
   });

   server.listen(3200, () => {
     console.log('ramp-embed session route on :3200');
   });
   ```

   Note a única linha fazendo o trabalho da escada de artefatos: `presetFiatAmount: RECORD.priceUsdc`. A sessão de onramp é precificada pela mesma definição de disco que o checkout cobra, com 12.5 USDC sendo 12.5 dólares porque uma paridade com o dólar cobrada 1:1 não precisa de passo de conversão. Uma checagem de unidade antes de seguir, porque este curso martelou unidades base por cinco módulos: `presetFiatAmount` é em unidades inteiras de fiat, e o `RECORD.priceUsdc` no seu checkout é o `12.5` em escala humana, nunca unidades base; se o seu arquivo de disco algum dia guardasse `12_500_000`, converta antes desta linha ou o widget vai educadamente oferecer uma prensagem de doze milhões de dólares. Dentro do workspace `wavelength-checkout`, o `ramp-embed/` consome o irmão `checkout/`; nada é duplicado.

3. **O smoke test.** Crie o `ramp-embed/smoke.ts`, a verificação offline padrão do módulo. Ele exercita as duas funções puras e afirma a propriedade de segurança diretamente, sem rede e sem chaves:

   ```typescript
   import { RECORD } from '../checkout/record.ts';
   import { buildOnrampUrl, buildSessionRequest } from './session.ts';

   const DEMO_ADDRESS = 'Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS';

   const request = buildSessionRequest(DEMO_ADDRESS);
   const binds = request.addresses.some(
     (entry) => entry.address === DEMO_ADDRESS && entry.blockchains.includes('solana'),
   );
   if (!binds) {
     throw new Error('session request must bind the address on solana');
   }

   const url = new URL(
     buildOnrampUrl('demo-session-token', { presetFiatAmount: RECORD.priceUsdc }),
   );
   if (url.hostname !== 'pay.coinbase.com') {
     throw new Error('onramp URL must live on pay.coinbase.com');
   }
   if (url.searchParams.get('sessionToken') !== 'demo-session-token') {
     throw new Error('client URL must carry the sessionToken');
   }
   if (url.searchParams.get('defaultNetwork') !== 'solana') {
     throw new Error('defaultNetwork must be solana');
   }
   if (url.searchParams.get('presetFiatAmount') !== String(RECORD.priceUsdc)) {
     throw new Error('the preset fiat amount must carry through');
   }
   if (url.toString().includes(DEMO_ADDRESS)) {
     throw new Error('the wallet address leaked into the client URL');
   }

   console.log('session request binds the address server-side');
   console.log('wallet address absent from the client URL');
   console.log(url.toString());
   ```

4. **Rode até a falha nomeada.**

   ```bash
   npx tsx ramp-embed/smoke.ts
   ```

   Com os buracos no lugar isso morre com `TODO: buildSessionRequest`, e essa falha exata é o checkpoint da parte guiada. Qualquer outra coisa quer dizer um erro de digitação mais acima: o suspeito de sempre é o caminho relativo de import para `checkout/record.ts`, que precisa subir para fora do `ramp-embed/` com `../`.

![Os três arquivos do ramp-embed, construtores de sessão puros, uma rota de servidor autenticada, e um smoke test provando que o endereço é vinculado no servidor e está ausente da URL do cliente.](assets/v09-diagram.webp)

5. **A caminhada da sessão, sandbox por padrão.** Este passo precisa das chaves do CDP, e a caminhada do offramp no passo 6 também, então guarde os dois juntos para quando você tiver credenciais; só a caminhada opcional com cartão real no fim precisa, além disso, da liberação de modo trial do seu app. Se você está offline ou sem chaves hoje, o smoke test sozinho completa a construção da lição, e os passos 5 e 6 esperam. Exporte `CDP_API_KEY_ID` e `CDP_API_KEY_SECRET`, suba a rota com `npx tsx ramp-embed/server.ts`, e depois faça o papel do cliente da vitrine a partir de um segundo terminal:

   ```bash
   curl -s -X POST http://localhost:3200/session \
     -H 'Content-Type: application/json' \
     -d '{"address":"<your own mainnet wallet address>"}'
   ```

   O JSON que volta guarda uma URL de onramp ao vivo, e o token cru está sentado nela como o query param `sessionToken`. Lembre do que a teoria disse sobre os tempos de vida dele: uso único, cinco minutos. Cada curl emite um token bom para uma tentativa, então rode o curl de novo sempre que precisar de outro; cada caminhada abaixo e o passo 6 recebem a própria emissão fresca.

   A caminhada padrão é o sandbox, porque nada nesta lição precisa do seu cartão: emita um token fresco e entregue ele para `https://pay-sandbox.coinbase.com/?sessionToken=<token>` com os mesmos presets de exibição. Nenhum dinheiro de verdade se move, e o ponto de integração que está sendo avaliado é o mesmo que o host ao vivo avalia: a URL que você abriu saiu da sua rota de servidor, vinculada antes de o navegador ver qualquer coisa. Uma nota de honestidade antes de você digitar qualquer coisa naquele formulário de cartão. Esta lição já imprimiu os valores de teste aceitos pelo sandbox, e parou, porque eles são fatos de fornecedor exatamente do tipo que este módulo não para de mandar você datar: a documentação de sandbox de guest checkout que a gente verificou na escrita (2026-08-23) não era mais encontrável no índice de documentação da Coinbase numa reconferência em 2026-09-02. Então pegue os valores de teste do que a documentação atual da Coinbase disser, e se o host de sandbox rejeitar o seu token ou o fluxo tiver mudado de lugar por completo, emita de novo, reconfira a documentação, e caia para a caminhada opcional abaixo ou para o smoke test; o trabalho avaliado desta lição nunca dependeu desse formulário.

   A sessão com cartão real é a opcional, não a padrão. Se você tem liberação de modo trial e quer ver produção, abra a URL do JSON num navegador e caminhe por ela como o comprador: o widget abre em USDC na Solana com 12.5 dólares pré-preenchidos, e a tela de Apple Pay ou de cartão que a Coinbase mostra é exatamente a superfície que um comprador da Wavelength veria. Se você vai completar a comprinha é decisão sua e do seu limite de trial. Qualquer um dos dois hosts é uma conclusão legítima deste passo.

6. **A caminhada do offramp, guiada.** Nenhum código novo, de propósito: o caminho de saída é hospedado, então a construção é um redirecionamento que você monta na mão e o aprendizado está em caminhar por ele. Como no passo 5, isto precisa das suas chaves do CDP. Emita um session token fresco do mesmo jeito (mesmo endpoint; a rota de servidor do passo 5 já faz isso, e para uma sessão de venda o endereço vinculado é a carteira de onde o USDC sai, e não um destino), e depois forme a URL hospedada: `https://pay.coinbase.com/v3/sell/input` com o seu `sessionToken`, um `partnerUserRef` nomeando o artista nos seus livros (qualquer string opaca com menos de 50 caracteres, nunca a identidade real dele) e uma `redirectUrl` que você controla. Um pré-requisito se esconde nesse último param: domínios de redirecionamento entram numa allowlist nas configurações de Onramp do seu dashboard do CDP antes de a Coinbase aceitar eles, então registre `http://localhost:3200` ali primeiro, e quando o fluxo hospedado se recusar a abrir, um redirecionamento rejeitado e não registrado é a primeira coisa a checar. Abra ele e narre o que você vê contra a teoria: o login é a relação do artista com a Coinbase, as verificações de identidade são dele e não suas, o envio de USDC sai da carteira dele, e as opções de payout que aparecem para você são a fatia da sua região da lista de saque da teoria — ACH se você está sentado numa conta bancária dos EUA, PayPal onde a Coinbase oferece, raramente a lista inteira de uma vez. Depois feche o ciclo em voz alta, porque este é o músculo da avaliação: diga quem é merchant-of-record para o onramp embutido, para este offramp hospedado, e para uma compra com cartão no onramp da Stripe, uma linha cada. Se alguma das três te tomar mais de uma frase, releia o mapa de costuras antes de seguir.

## Challenge

**Completion.** Feche os dois buracos no `session.ts`. `buildSessionRequest` devolve o corpo do vínculo, `buildOnrampUrl` monta a URL só com token; os dois formatos estão na íntegra na seção de teoria, e o objetivo de digitar eles você mesmo é notar qual metade de vincule-no-servidor-nunca-vaze cada um impõe. A aceitação é o smoke test passando inteiro:

```bash
npx tsx ramp-embed/smoke.ts
```

Três linhas: a confirmação do vínculo, a confirmação de não vazamento, e uma URL `pay.coinbase.com` contendo `sessionToken` e `defaultNetwork=solana` sem nenhum endereço de carteira em lugar nenhum dela. Código de saída 0.

**Solo.** O desafio de código junta as duas metades do lab numa função só, chamada no momento da ordem de três saltos que você acabou de caminhar em que a emissão já respondeu: `initHeadlessOnramp(destinationAddress, sessionToken, fiatAmount)` recebe a carteira de destino, o token que o seu servidor recebeu de volta da emissão de sessão e o valor de fiat pré-definido, nessa ordem, e devolve `{ requestBody, onrampUrl }`, o corpo do vínculo que o seu servidor mandou por POST para ganhar aquele token mais a URL só com token que o navegador do comprador abre. O que te entregam não é um esqueleto, e sim a integração ingênua do começo desta lição, feita concreta. Salve como `ramp-embed/naive-onramp.ts` e conserte no lugar:

```typescript
// ramp-embed/naive-onramp.ts: the working-but-leaky integration, as promised.
// Four repairs, and the grader checks all of them: it binds the wrong chain,
// it leaks the raw address into the client URL, the URL never carries the
// sessionToken, and defaultNetwork points at the wrong network. The first two
// are the conceptual sins from the top of the lesson; the last two are what
// the fix has to put in their place.

interface OnrampInit {
  requestBody: {
    addresses: { address: string; blockchains: string[] }[];
    assets: string[];
  };
  onrampUrl: string;
}

function initHeadlessOnramp(
  destinationAddress: string,
  sessionToken: string,
  fiatAmount: number
): OnrampInit {
  const requestBody = {
    addresses: [{ address: destinationAddress, blockchains: ['ethereum'] }],
    assets: ['USDC'],
  };
  // Query built by hand rather than URLSearchParams: the challenge grader
  // runs in a bare JS realm without the web URL APIs.
  const params: [string, string][] = [
    ['address', destinationAddress],
    ['defaultNetwork', 'ethereum'],
    ['presetFiatAmount', String(fiatAmount)],
  ];
  const query = params
    .map(([k, v]) => `${k}=${encodeURIComponent(v)}`)
    .join('&');
  const onrampUrl = `https://pay.coinbase.com/buy/select-asset?${query}`;
  return { requestBody, onrampUrl };
}
```

Conserte as duas metades contra os mesmos critérios de aceitação que o lab usou. A requisição precisa vincular o destino na Solana, a URL do cliente precisa carregar o `sessionToken` com `defaultNetwork=solana` e o valor de fiat pré-definido, e o endereço cru nunca pode aparecer na URL. Reescrever uma integração que vaza é a versão deste exercício que você vai de fato encontrar no trabalho, e o padrão viaja para muito longe deste curso: todo provedor que te entrega uma API do tipo "crie uma sessão no servidor, referencie ela no cliente" é exatamente este formato com nomes de campo diferentes.

## Checkpoint, e a porta abre para os dois lados

Se o smoke test brigar com você depois do completion, as mensagens de falha são o diagnóstico: uma assertion de `blockchains` quer dizer que o corpo do vínculo está malformado (ele é um array de entradas de endereço, cada uma com o próprio array `blockchains`, um aninhamento fácil de achatar sem querer), uma assertion de `sessionToken` ou de `defaultNetwork` quer dizer que um nome de param derivou (eles são case-sensitive, `sessionToken` e não `sessiontoken`), e a assertion de vazamento disparando quer dizer que o endereço achou o caminho até os argumentos do construtor de URL, que é o vazamento que esta lição existe para tornar estruturalmente impossível em tudo que vem depois da emissão. E se a rota de sandbox ao vivo responder 401, as claims do seu JWT não batem com a requisição: método, host e path no `generateJwt` precisam ser exatamente o método, o host e o path que você depois vai buscar com o fetch.

Veja a vitrine funcionando de ponta a ponta: um comprador com nada além de um cartão entra, o seu servidor vincula uma sessão, a Coinbase transforma o cartão em USDC na ATA do próprio comprador, e o seu checkout, watcher e livro-razão existentes seguem daí sem aprender nada de novo, assim que a mainnet for onde eles rodam: a maquinaria é agnóstica de cluster, e hoje o seu checkout ainda cobra o mint substituto da devnet, então o ciclo completo de cartão até checkout fecha quando o capstone mover a stack para a configuração de mainnet. Os royalties de um artista saem pelo outro lado até um banco por trilhos que você nunca toca. Dinheiro entrando, dinheiro saindo, custódia longe de você, e você consegue nomear o merchant-of-record em cada costura, uma linha cada. Essa última habilidade parece curiosidade e na verdade é a régua de contratação: muitos devs conseguem montar um widget, poucos conseguem te dizer quem engole o chargeback.

Agora você consegue mover dinheiro para dentro e para fora, mas a porta que você construiu assume um tipo só de comprador. Para um comprador dos EUA no cartão de crédito, um comprador da UE que vive no SEPA e um comprador brasileiro esperando PIX, o trilho certo é diferente em cada caso, e escolher errado ou perde a venda ou come a margem. A próxima lição é sobre processadoras de aceitação e corredores: três provedores comparados em números, e um registro de decisão escrito sobre qual trilho atende qual comprador.
