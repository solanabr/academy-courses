# MPP, AP2, ACP: a guerra dos padrões (e pondo o gate na API com o pay)

## Resumo

A lição passada fechou o loop de vendas para máquinas: a API de preço de prensagem da Wavelength fica atrás do @x402/express, um agente pagante liquida cada chamada pelo facilitador da devnet, e todo id de fatura em extra.memo cai no mesmo livro-razão do back office por onde passa uma venda humana. Uma API, um protocolo, conciliados.

Antes de qualquer teoria, prove que você ainda tem a ferramenta em que esta lição se apoia. Você instalou ela na primeiríssima lição deste curso, quando ela ainda parecia uma curiosidade:

```bash
npm i -g @solana/pay@1.0.26   # the opening lesson installed it locally; go global so `pay` is on your PATH
pay --version
```

Leia com atenção o número que você recebe, porque ele ensina o tema inteiro da lição antes de a lição começar. Em 2026-08-22 o release mais novo da CLI estava com a tag `pay-v0.27.0` (2026-08-02) e o `pay-v0.28.0` veio em 2026-08-26, mas o wrapper npm `@solana/pay` 1.0.26 fixa o build da CLI que ele baixa em 0.26.0, então uma instalação global nova ainda imprime `pay 0.26.0`. Três números de versão para uma ferramenta só, todos atuais, nenhum deles errado: a versão do pacote npm, o build fixado da CLI, e a tag de release mais nova. Tudo abaixo está escrito contra o 0.26.0 porque é isso que o comando de instalação deste curso de fato te entrega; se o seu `pay --version` for maior, trate as flags e os campos daqui como hipótese inicial e deixe o `--help` da própria ferramenta resolver qualquer divergência. (Ficar com o `npx pay` local da lição um também funciona; é só prefixar todo `pay` desta lição com `npx`.) Essa identidade dupla, biblioteca de checkout e CLI agêntica num pacote só, foi a cena de abertura deste curso inteiro, e hoje ela dá retorno.

Um protocolo só é o problema. Esta semana um agente construído pelo Google, um fluxo de checkout da OpenAI e um client nativo da Solana podem todos bater na mesma porta, cada um segurando uma credencial diferente e uma suposição diferente sobre quem está de fato vendendo o disco. Aqui está o que você leva desta lição:

- **MPP (Machine Payments Protocol)** vem em duas camadas, e juntar as duas numa só é de longe a coisa mais comum que se fala errado sobre ele. A base é o `draft-httpauth-payment-00`, "The 'Payment' HTTP Authentication Scheme", um Internet-Draft da IETF saído da Tempo Labs e da Stripe; a contribuição da Solana Foundation é o `draft-solana-charge-00`, uma spec de método de pagamento registrada sob essa base que define o método `solana/charge`. Juntas, elas movem o pagamento para a maquinaria nativa de auth do HTTP: um desafio `WWW-Authenticate: Payment`, uma credencial `Authorization`, uma prova `Payment-Receipt`.
- **AP2** (o Agent Payments Protocol do Google, v0.2, sendo padronizado por grupos de trabalho da FIDO Alliance) e **ACP** (o Agentic Commerce Protocol da Stripe e da OpenAI, Apache-2.0, com o ChatGPT como primeira plataforma) são centrados em cartão, e o ACP é explicitamente projetado para que o lojista continue sendo merchant-of-record.
- A parte prática que nenhum outro curso tem: `pay gate api paywall.yml` põe um gate na frente da API de preço de prensagem para que a mesma API atenda x402 e MPP, e o `pay curl`, do lado do client, negocia sozinho qualquer protocolo que o servidor ofereça.
- A recomendação honesta de 2026, argumentada em vez de afirmada: não aposte exclusivamente em nenhum deles ainda. Ponha o gate uma vez e negocie, e aceite os dois custos desse hedge: uma dependência de CLI que muda rápido, e nenhum memo por chamada no caminho do gate, então as vendas roteadas pelo gate são trabalho de livro-razão ainda devido.

Como o trabalho se divide hoje, dito sem rodeios: a teoria abaixo vem servida inteira, o lab é uma transcrição guiada que você digita, e o Challenge é puro trabalho de julgamento, sem passo a passo, porque avaliar padrões sob incerteza é a habilidade que esta lição de fato ensina.

## A guerra das velocidades

Em 1948, a Columbia Records lançou o disco long-playing de 33⅓ rpm. Um ano depois a RCA respondeu com o 45. Os dois eram melhorias reais sobre o 78 de goma-laca, os dois eram incompatíveis entre si, e os dois lados gastaram dinheiro de marketing insistindo que o outro estava condenado. Os compradores de discos fizeram a coisa racional: muitos pararam de comprar toca-discos de vez e esperaram. A guerra não terminou com um vencedor. Terminou quando os fabricantes de toca-discos passaram a entregar aparelhos de várias velocidades, e aí cada formato achou o nicho em que era de fato melhor, o LP para álbuns, o 45 para singles.

Segure esse toca-discos na cabeça pela próxima meia hora. Pagamento agêntico em 2026 é uma guerra de velocidades: quatro e tantos padrões, cada um bancado por alguém enorme, cada um girando no seu próprio rpm. O seu trabalho como integrador da Wavelength não é escolher a velocidade vencedora, é entregar o toca-discos de várias velocidades. O que está em jogo é concreto: chute errado e você reescreve a sua integração de pagamento quando o mercado se mexer; recuse-se a escolher e você atende todo agente que aparecer enquanto os seus concorrentes ainda estão lendo drafts de spec. Então o roteiro é este: primeiro o desafiante nativo da Solana de perto, depois os dois incumbentes centrados em cartão, depois a única linha da comparação que decide tudo (quem é merchant-of-record), depois a aposta.

![Linha do tempo em duas faixas separando o esquema base do MPP, registrado na IETF, com sua submissão e sua expiração, da spec de método da Solana rastreada no git, acima de uma faixa com marcadores de x402, AP2, ACP e desafiantes.](assets/v01-timeline.webp)

### MPP: pagamento como credencial HTTP

O MPP é o Machine Payments Protocol, e a primeira coisa a internalizar é que ele são dois documentos com dois donos, e eles não têm o mesmo peso. A base é o `draft-httpauth-payment-00`, "The 'Payment' HTTP Authentication Scheme": um Internet-Draft de verdade da IETF, no datatracker, submetido em 2026-06-19 e expirando em 2026-12-21, saído da Tempo Labs e da Stripe e não de algum lugar perto da Solana. Ele define a dança desafio-credencial-recibo no abstrato e fica deliberadamente agnóstico quanto ao método de pagamento; nas palavras dele, "specific payment methods are defined in separate payment method specifications." A parte da Solana é uma dessas: o `draft-solana-charge-00`, "Solana Charge Intent for HTTP Payment Authentication", escrito por Ludo Galabru e Ilan Gitter, da Solana Foundation, definindo o método `solana/charge`, o que um charge intent carrega, como ele é assinado e como ele liquida.

O que essa spec de método *não* é: um Internet-Draft da IETF. Busque `draft-solana-` no datatracker e ele devolve zero documentos; só o esquema base chega a ser submetido lá. Isso vale um hábito, e ele vai te impedir de citar errado alguma coisa na sua própria documentação. Specs de método renderizam páginas bonitas em estilo RFC com uma data de publicação e uma expiração impressas no topo, e é o build do site que produz essas datas: refaça o build e elas mudam, porque nada as registrou em lugar nenhum. Então não cite essas datas. Cite o histórico do git no lugar: o `solana/charge` caiu no repo da spec em 2026-03-24, e a mudança substantiva mais recente dele foi o suporte a confidential transfers do Token-2022 em 2026-08-07. Congelei isso em 2026-08-22 e reconferiria no dia em que você for construir.

Mecanicamente, o MPP faz uma coisa que o x402 deliberadamente não fez: move o pagamento para a maquinaria nativa de autenticação do HTTP em vez de headers customizados. O fluxo se lê como um Basic Auth com dinheiro dentro. O seu servidor recusa uma requisição não paga com um desafio `WWW-Authenticate: Payment` descrevendo o que ele quer. O client responde tentando de novo com um header `Authorization` carregando um charge intent da Solana assinado como sua credencial. Quando o pagamento cai, a resposta do servidor inclui um header `Payment-Receipt`, a prova que o client arquiva. Desafio, credencial, recibo. Qualquer biblioteca HTTP que entende fluxos de auth já entende o formato dessa dança, e essa é a aposta de design: deixar pagamento de máquina entediante para todo proxy, cache e stack de middleware que lida com `WWW-Authenticate` há trinta anos.

![Diagrama de sequência de uma chamada MPP em pull mode, do desafio Payment, passando pelo charge intent assinado do agente, até a transmissão co-assinada pelo servidor e o header de recibo.](assets/v02-flowchart.webp)

Dois modos, e o default importa. No **pull mode**, o client assina o charge intent e entrega; o servidor pode co-assinar como fee payer e transmitir a transação ele mesmo. No **push mode**, o client leva a transação para a blockchain por conta própria e apresenta o resultado. Pull é o default, e repare no que a cláusula do co-signatário contrabandeia: o patrocínio de taxa está embutido no caminho feliz do protocolo. Aqui, o servidor pagando a taxa de rede do próprio cliente é a postura default. Segure esse pensamento por mais uma lição; ele está prestes a virar o assunto inteiro do módulo 8.

A spec de método da Solana carrega mais duas features com cara de Solana que valem ser nomeadas. Os payment splits deixam um charge se abrir em leque para vários destinatários, com teto de 8 transferências adicionais, que é uma gravadora pagando um artista e uma fábrica de prensagem na mesma liquidação sem um segundo salto. E os confidential transfers do Token-2022 passam por um charge `type='bundle'`, então um agente consegue pagar sem transmitir o valor para o mundo (os confidential transfers em si são território do curso Digital Assets, Tokenization and Token Extensions; aqui você só precisa saber que o MPP deixou a porta aberta para eles).

Ponha o MPP ao lado do protocolo que você já entregou. O x402, de que você construiu as duas pontas na lição passada, carrega o pagamento em headers customizados `PAYMENT-SIGNATURE` e `PAYMENT-RESPONSE` e liquida através de um facilitador em quem você precisa confiar com visibilidade sobre cada transação. O MPP dobra o mesmo momento 402 dentro da semântica padrão de auth e, no pull mode, faz do seu próprio servidor o co-signatário em vez de um terceiro. Formato de confiança diferente, mesmo instinto comercial: cobrar por chamada, sobre HTTP, em stablecoins, sem cadastro de conta.

### AP2 e ACP: os incumbentes constroem para cartão

Agora o outro lado da loja. O AP2 é o Agent Payments Protocol do Google, na v0.2, sendo padronizado por grupos de trabalho da FIDO Alliance, o mesmo órgão que transformou passkeys de demo em default da indústria. Os primitivos centrais dele são credenciais digitais verificáveis e um par de mandatos assinados: um Checkout Mandate que captura o que o humano autorizou o agente a comprar, e um Payment Mandate que captura como ele tem permissão de pagar. O centro do design é responsabilização para cartão. Quando um agente compra a coisa errada com o seu Visa, a resposta do AP2 é um rastro de papel criptográfico de quem autorizou o quê. Os trilhos de pagamento em si continuam sendo o que eram, o que hoje quer dizer principalmente bandeiras de cartão.

O ACP é o Agentic Commerce Protocol, coescrito pela Stripe e pela OpenAI, lançado sob Apache-2.0, com o ChatGPT como primeira plataforma em produção. A jogada característica dele é o Shared Payment Token: uma credencial com escopo que representa o método de pagamento do comprador, que a plataforma passa para o lojista de modo que o lojista, e esta é a cláusula estrutural, **continua sendo merchant-of-record**. O cliente compra dentro do ChatGPT, mas quem vende o disco continua sendo a Wavelength: o seu nome na fatura, a sua política de reembolso, as suas obrigações fiscais, o seu relacionamento com o cliente. Para quem já tocou uma loja através de uma processadora de cartão, o ACP é de propósito o menos estranho dos quatro padrões.

![Diagrama em duas faixas contrastando os mandatos assinados por humanos do AP2, apresentados como credenciais verificáveis, com o Shared Payment Token do ACP, que deixa o lojista cobrando como merchant-of-record.](assets/v03-diagram.webp)

Vale parar em quem está onde, porque um desses nomes está num lugar que você talvez não tenha registrado. Você mapeou o hedge em quatro frentes da Stripe duas lições atrás: a parede de "trusted by" do x402, o ACP coescrito com a OpenAI, a adquirente de USDC que liquida em fiat da lição do corredor, e o esquema de autenticação HTTP "Payment". Essa quarta frente é o draft base que você acabou de ler. A Stripe coescreveu o `draft-httpauth-payment-00`, o que a coloca em todos os trilhos desta lição, incluindo aquele normalmente descrito como a resposta nativa da Solana. Quando a maior empresa de infraestrutura de pagamentos em campo se recusa a escolher um único vencedor, isso te diz alguma coisa sobre quão resolvida está essa guerra.

E é uma guerra de verdade, não uma de slides. A OKX lançou um padrão concorrente que ela chama de APP. Enquanto isso a atxp.ai, uma plataforma de pagamentos para agentes, migrou para x402 mais MPP na Solana, um movimento que a Foundation destacou no seu apanhado de ecossistema de abril de 2026. Padrões se multiplicando de um flanco enquanto integradores consolidam de outro é exatamente a cara de um campo disputado visto de dentro.

### Merchant-of-record: a linha que decide a sua aposta

Tire a criptografia e cada padrão é uma resposta para uma única pergunta comercial: quando um agente compra um disco, quem vendeu? Você conheceu merchant-of-record nas lições de fiat deste curso; é a entidade que vende legalmente, o nome na disputa, a parte que carrega as obrigações de reembolso e de compliance. Alinhe os quatro nessa linha e a guerra fica muito mais fácil de ler.

![Matriz comparando x402, MPP, AP2 e ACP em merchant-of-record, transporte e liquidação, separando o par cripto-nativo do par centrado em cartão, com a coluna do MPP marcada como spec de método sob o draft base em vez de documento próprio da IETF.](assets/v04-comparison.webp)

Leia as colunas e os campos se organizam sozinhos. Sob x402 e MPP você está vendendo diretamente: o agente paga o seu endereço em stablecoins, e a pergunta interessante é em quem você confia no meio (um facilitador no x402; ninguém além do seu próprio servidor que co-assina no pull mode default do MPP). O draft do MPP nem se dá ao trabalho de reformular merchant-of-record, porque autenticação-de-pagamento-sobre-HTTP não muda quem é o vendedor. Sob AP2, o seu relacionamento com a processadora persiste e a contribuição do protocolo é evidência de autorização; o Google não está entrando como o vendedor dos seus discos. Sob ACP, manter você como merchant-of-record não é um acidente do design, é a manchete: Stripe e OpenAI construíram a maquinaria de credencial especificamente para que plataformas possam hospedar checkout sem absorver o papel legal do lojista.

### A aposta

Então a qual velocidade você compromete a loja? Leve cada aposta exclusiva até o seu modo de falha.

Aposte tudo no MPP e você está apostando em dois documentos de uma vez, o que é uma aposta mais fina do que parece à primeira vista. A Foundation controla o `solana/charge`, mas ela não controla o esquema debaixo: o `draft-httpauth-payment-00` pertence à Tempo Labs e à Stripe, expira no datatracker em 2026-12-21, e define o desafio, a credencial e o recibo que de fato dão forma à sua integração. Um `-01` de qualquer uma das camadas pode mexer num campo contra o qual você construiu, e a spec de método nem está no relógio da IETF, então a sucessão dela é uma decisão de repo e não um processo que você consegue acompanhar de fora. Autoria da Foundation é um sinal de verdade. É um sinal sobre uma das duas camadas.

Aposte tudo no x402 e você ganha o tráfego, o alcance cross-chain e o ecossistema de facilitadores que você conheceu na lição passada, mais a fronteira de confiança que você também conheceu na lição passada: um intermediário de liquidação que vê cada transação e consegue filtrar o que liquida. Uma cadeira boa, honestamente. Só que não neutra.

Aposte tudo em AP2 ou ACP e você apostou uma API nativa da Solana medida por chamada em trilhos centrados em cartão governados pelo Google ou pela dupla Stripe-e-OpenAI. Para a vitrine da Wavelength no ChatGPT algum dia, o ACP é provavelmente a porta certa, e a cláusula de merchant-of-record faz dela uma porta genuinamente amiga do lojista. Para a API de preço de prensagem, onde o comprador é um script com carteira e sem cartão, é o fornecedor certo e a ferramenta errada.

O tl;dr é: apostar num padrão só em agosto de 2026 é prematuro, e você não precisa. O toca-discos de várias velocidades existe. O `pay gate` põe um único gate na frente da sua API que atende x402 e MPP simultaneamente, e o `pay curl`, do lado do client, negocia o que o servidor oferecer. Você para de prever o vencedor e começa a atender quem aparecer.

Nomeie o custo, porém, porque o hedge não é de graça. Você está criando uma dependência da CLI pay: esta lição é escrita contra o build 0.26.0 fixado no npm, e o repo já tinha marcado a tag `pay-v0.28.0` em 2026-08-26 (verificado em 2026-09-07). O feed de releases dele mostra versões minor caindo com dias de diferença ao longo de junho, julho e agosto, e ela vai continuar mudando — leia o seu próprio `pay --version` e a página de releases do repo em vez desta frase. Então não grave o subcomando na marra dentro da sua aplicação: mantenha o `pay gate` na camada de deploy, um processo que os seus scripts de ops sobem, nunca uma string para a qual a sua lógica de negócio chama o shell. Se um release futuro renomear ou remodelar o gate, o seu raio de impacto é um arquivo de config e uma unit do systemd, e no pior caso esta lição degrada de forma graciosa: a demo do lado do client com `pay curl` continua funcionando contra o middleware x402 puro que você entregou na lição passada. Essa é a diferença entre depender de uma ferramenta em movimento e fazer dela uma peça estrutural.

![Mapa de decisão pesando apostas exclusivas em MPP, x402, AP2 ou ACP contra pôr o gate uma vez com a CLI pay, cada nó carregando o seu próprio modo de falha ou custo.](assets/v05-diagram.webp)

## Lab: ponha o gate uma vez, atenda os dois

A transcrição-alvo: um gate na frente da API de preço de prensagem, uma chamada não paga recusada com os desafios dos dois protocolos, e uma chamada paga negociada por MPP concluída pelo `pay curl`. Digite junto; a transcrição é o artefato que o critério de aceite abaixo pede que você tenha em mãos — a plataforma corrige o quiz, a transcrição é a sua própria evidência.

Uma nota de honestidade sobre schema antes do passo um, no mesmo espírito de todo pin de versão deste curso: a superfície de config do gate pertence a uma CLI que entrega rápido. Os campos abaixo foram lidos do `pay 0.26.0` em 2026-08-22 pedindo para a ferramenta escrever o próprio config, que é o passo 3, e esse arquivo gerado tem mais autoridade que esta página em tudo, menos num bug conhecido de scaffold do 0.26.0 (um campo `forward_url` obsoleto) que o passo 3 te guia a corrigir.

**1. Confirme o toolchain.** O `pay --version` do topo da lição responde pela CLI. Mais duas coisas de que ela precisa antes de conseguir pagar qualquer coisa: uma conta, e um lugar para onde mandar dinheiro. O `pay setup` gera um par de chaves, guarda ele no keystore do seu SO e oferece para fundear; no macOS o backend é o keychain, então um shell não interativo precisa de `pay setup --backend keychain` (Linux: `gnome-keyring`, Windows: `windows-hello`, CI headless: `file`). Pule isso e o primeiro `pay curl` vai parar e rodar o setup na sua cara no meio do lab. A API pelada abaixo precisa do mesmo ferramental de workspace da lição passada: Node com `express` (`npm i express`) e `npx tsx` para rodar TypeScript direto.

**2. Suba a API de preço de prensagem pelada.** Não o servidor embrulhado em x402 da lição passada: a rota de cotação nua debaixo dele. O gate está prestes a ser dono da camada de pagamento, então o upstream precisa ser livre de pagamento.

```bash
mkdir -p ~/wavelength/pay-gate && cd ~/wavelength/pay-gate
```

```ts
// ~/wavelength/pay-gate/price-api.ts
// The pressing-price quote, with zero payment code. The gate in front handles that.
import express from 'express';

const app = express();

app.get('/price', (req, res) => {
  const record = String(req.query.record ?? 'WVL-UNSPECIFIED');
  // `run` is the alias last lesson's x402 agent already sends; accepting both
  // is what lets that agent hit this API through the gate in step 7 unchanged.
  const runSize = Number(req.query.runSize ?? req.query.run ?? 0);
  if (!Number.isInteger(runSize) || runSize < 100) {
    res.status(400).json({ error: 'runSize (min 100) required' });
    return;
  }
  // The price model is reworked from last lesson on purpose: the setup fee is
  // now explicit and amortized over the run instead of folded into a flat unit
  // rate, so quotes for the same run WILL differ from last lesson's numbers.
  const setupFeeUsd = 900;
  const perUnitUsd = runSize >= 500 ? 4.1 : 5.6;
  const totalUsd = setupFeeUsd + perUnitUsd * runSize;
  res.json({
    record,
    runSize,
    unitPriceUsd: Number((totalUsd / runSize).toFixed(2)),
    totalUsd: Number(totalUsd.toFixed(2)),
    currency: 'USD',
  });
});

app.listen(3000, () => console.log('pressing-price API (bare) on :3000'));
```

Rode e faça o smoke test:

```bash
npx tsx ~/wavelength/pay-gate/price-api.ts &
curl -s 'http://localhost:3000/price?record=WVL-014&runSize=500'
```

Você deve ver uma cotação em JSON. Repare no que você acabou de provar: o endpoint hoje dá a resposta de graça, para qualquer um, que é o problema de abertura da lição passada de novo.

**3. Escreva o config do gate.** Não escreva ele na mão do zero: peça para a ferramenta escrever o dela, depois aplique a única correção conhecida abaixo e compare com a minha versão editada:

```bash
cd ~/wavelength/pay-gate
pay server scaffold          # writes paywall.yml
pay gate api --help          # the flags, which are a separate surface from the file
```

Duas superfícies, duas fontes de verdade: o arquivo gerado pelo scaffold define os nomes dos campos, o `--help` define as flags. Agora edite o scaffold até o único endpoint da Wavelength. A versão abaixo é o que o 0.26.0 aceita, e ela inclui uma correção que você bateria como erro de outra forma, porque o template de scaffold neste build omite um bloco que o binário exige:

```yaml
# paywall.yml - edited from `pay server scaffold` output, pay 0.26.0, 2026-08-22
name: wavelength-price
subdomain: wavelength
title: "Wavelength pressing-price API"
description: "Vinyl pressing quotes, per call"
category: other
version: v1
accounting: pooled
routing:                        # the scaffold writes a flat `forward_url:` here;
  type: proxy                   # the 0.26.0 binary wants this block instead and
  url: http://localhost:3000    # errors "Invalid paywall: missing field `routing`"
endpoints:
  - method: GET
    path: "price"
    description: "Pressing-price quote"
    metering:
      dimensions:
        - direction: usage
          unit: requests
          scale: 1
          tiers:
            - price_usd: 0.10   # per call, deliberately under the $1
                                # spendControls default you met last lesson
```

Esse descompasso entre o scaffold da própria ferramenta e o parser dela mesma vale trinta segundos de atenção, porque é o bug mais instrutivo no caminho até um gate funcionando: até o template first-party deriva do binário first-party num projeto que entrega tão rápido. O `forward_url` do template é um rename que o parser já deixou para trás. Leia o erro, troque o bloco, siga. Repare também no que não está neste arquivo: nenhum campo de memo por chamada. A conciliação por id de fatura nesta camada pertence a qualquer protocolo que negocie o pagamento, `extra.memo` do lado do x402 como você construiu na lição passada, e o gate não te oferece um botão para isso. E nenhum campo de payee também: nada neste arquivo nomeia a carteira em que o dinheiro cai, então antes de confiar qualquer coisa de verdade ao gate, abra num explorer a signature de uma chamada liquidada do passo 6 e confirme qual conta foi de fato creditada. Trate ligar isso ao endereço de lojista da Wavelength como um item de antes de entrar no ar, não como uma suposição.

**4. Suba o gate.** Primeiro libere a porta dele: o servidor de middleware da lição passada, e a API mock da lição anterior a ela, os dois escutavam em `:4021`, então pare qualquer um deles ainda rodando (ctrl-C nos terminais deles) ou o gate morre com `EADDRINUSE` no momento em que faz o bind.

```bash
pay gate api paywall.yml --bind 127.0.0.1:4021 --rpc-url https://api.devnet.solana.com
```

As duas flags são deliberadas. O `--bind` tira o gate do seu default `0.0.0.0:1402` e o põe na porta que esta lição usa, e fazer bind no localhost mantém fora da sua rede um paywall com que você está experimentando. O `--rpc-url` importa mais: o gate valida o destinatário do pagamento contra o RPC de mainnet a menos que você aponte ele para outro lugar, e este lab é um lab de devnet. (Se fundear uma conta de devnet for um perrengue, a CLI também tem um modo `pay --sandbox gate api ...` que roda contra um Surfpool hospedado com carteiras efêmeras fundeadas automaticamente; ele precisa desse host alcançável, então trate ele como a alternativa, não como o default.) O gate faz proxy de `:4021` na frente da sua API pelada em `:3000`. A sua lógica de negócio não mudou uma linha sequer; a camada de pagamento agora mora inteiramente na frente dela.

**5. Bata sem pagar.** Acerte o gate com curl puro e leia a recusa com atenção:

```bash
curl -i 'http://localhost:4021/price?record=WVL-014&runSize=500'
```

O status é 402, e a parte interessante é que a resposta fala duas vezes, as duas em headers (o texto exato é da CLI mudar; o formato é o que você está conferindo):

```text
HTTP/1.1 402 Payment Required
WWW-Authenticate: Payment ...challenge fields: amount, asset, recipient...
PAYMENT-REQUIRED: eyJ4NDAyVmVyc2lvbiI6Miwi...   <- base64 x402 v2 challenge
Content-Length: 2

{}
```

A linha `WWW-Authenticate` é o desafio do MPP, no header de auth que esta lição acabou de apresentar. A linha `PAYMENT-REQUIRED` é o desafio do x402 v2 que você já sabe decodificar da lição passada, e ela viaja num header pela razão que a lição passada deu. Então não vá procurar um array `accepts` no corpo aqui também. O corpo é `{}`, exatamente como era contra o seu próprio middleware. Uma recusa, dois protocolos, dois headers, os dois anunciando o mesmo preço. Essa fala dupla é o produto inteiro desta lição.

![Topologia mostrando agentes x402, clients MPP e chamadores não pagos todos batendo num único pay gate na porta 4021, que encaminha só as chamadas liquidadas para a API de preço de prensagem pelada na porta 3000.](assets/v06-diagram.webp)

**6. Deixe a CLI negociar.** Agora a chamada paga, com o lado client da mesma ferramenta:

```bash
pay curl 'http://localhost:4021/price?record=WVL-014&runSize=500'
```

Veja a sequência que ela narra: primeira requisição, 402 recebido, protocolo escolhido (MPP aqui, já que o `pay curl` é falante nativo), charge intent assinado, retry com a credencial `Authorization`, e aí o seu JSON de cotação com um header `Payment-Receipt` na resposta. Capture a transcrição inteira; é o primeiro item do critério de aceite abaixo. Vou admitir que, na primeira vez que rodei esse fluxo de ponta a ponta, a parte que me pegou não foi o pagamento cair, foi o quanto a transcrição parece entediante. Um desafio de auth, uma credencial, um recibo. Trinta anos de memória muscular de HTTP, agora com dinheiro dentro.

**7. Prove que a outra velocidade ainda toca.** O gate alega atender x402 também, então verifique com o agente pagante que você construiu na lição passada, apontado para o gate em vez do middleware antigo. A URL base dele é a variável de ambiente `API_URL` que você ligou na lição passada, e ele acrescenta o próprio `?run=...&invoice=...`, que é exatamente por que a API do passo 2 aceita `run` além de `runSize`:

```bash
cd ~/wavelength/x402
API_URL=http://localhost:4021 npx tsx src/agent.ts
```

O agente deve liquidar exatamente como liquidou na lição passada, headers e facilitador e tudo, sem nunca notar que o servidor atrás da porta mudou. Três coisas de fato diferem, então espere por elas em vez de sair debugando.

Primeiro, o gate mede $0.10 por chamada contra o preço de rota de $0.05 da lição passada (ainda bem abaixo do teto do spendControls), e os números da cotação refletem o modelo de preço retrabalhado do passo 2.

Segundo, a quarta chamada do agente é a deliberadamente acima do teto, e ela vai para `/price/rush`. Essa rota não existe aqui: a API pelada do passo 2 serve só `/price`, e o `paywall.yml` declara só o endpoint `price`, então o gate responde 404 e a chamada nunca chega longe o bastante para ser recusada pelo `spendControls`. Esse é o comportamento correto para esta lição, cujo assunto é negociação de protocolo e não limites — a recusa por teto é o checkpoint da lição passada e ela já passou lá. Se você quiser a transcrição idêntica de quatro linhas mesmo assim, são duas adições pequenas: um `app.get('/price/rush', ...)` no `price-api.ts` devolvendo o mesmo formato de cotação, e uma segunda entrada sob `endpoints:` no `paywall.yml` com `path: "price/rush"` e um `price_usd` acima do teto do agente. Opcional, e vale fazer uma vez se você quiser ver uma rota com gate recusar em vez de dar 404.

Terceiro, lembre da ausência que o passo 3 nomeou: o caminho do gate não escreve nenhuma linha no livro-razão, então essa liquidação aparece na sua transcrição e no explorer, não no `orders.jsonl`. Mesma API, os dois protocolos, um arquivo de config. Checkpoint: você agora tem uma transcrição de terminal com um 402 não pago mostrando os dois desafios, uma chamada negociada por MPP com um `Payment-Receipt`, e uma liquidação x402 pelo mesmo gate.

Repare no que não aconteceu com a sua escada enquanto você fazia isso. Nenhum artefato novo nasceu hoje.

![A API de preço de prensagem como um degrau de artefato em duas camadas: o middleware de pagamento que ainda é dono da conciliação por memo, e o pay gate de hoje atendendo os dois protocolos.](assets/v07-diagram.webp)

## Challenge

Sem passo a passo neste aqui. Você é o integrador, e o julgamento de um integrador é o entregável.

Escreva para a Wavelength um **memorando de aposta**, em três partes:

1. **A transcrição.** A sua captura do lab: o 402 de desafio duplo, a liquidação MPP do `pay curl`, o agente x402 liquidando pelo mesmo gate.
2. **A tabela de merchant-of-record.** Quatro linhas, uma por padrão (x402, MPP, AP2, ACP), cada uma nomeando quem é merchant-of-record e uma cláusula sobre por quê. Escreva a partir da comparação que você acabou de estudar, com as suas palavras; você vai defendê-la para um lojista que nunca ouviu falar de nenhuma dessas siglas.
3. **A escolha.** Uma frase nomeando em que a Wavelength deve apostar hoje, com a razão dela. Existe uma resposta defensável nos quatro padrões que você acabou de comparar, e ela não é o nome de um protocolo. Se a sua frase nomear um único padrão, releia os modos de falha na seção da aposta e discuta comigo na margem do memorando por que a sua sobrevive a eles.

Aceite quando: a transcrição mostrar os dois protocolos liquidando por um gate só, as quatro afirmações de merchant-of-record da tabela estiverem corretas, e a frase da escolha nomear tanto a estratégia quanto o custo dela.

## Checkpoint, e a pergunta por baixo

Se o lab brigou com você, faça a triagem nesta ordem. O `pay --version` falhando quer dizer que a instalação global está faltando ou está sombreada; reinstale e confira o seu PATH. Um comando que para para rodar o `pay setup` quer dizer que você pulou a conta do passo 1; dê um backend de keystore para ele e rode de novo. Um gate que recusa o `paywall.yml` com `Invalid paywall: missing field <name>` é deriva de schema, e a correção é mecânica: rode `pay server scaffold` de novo para um arquivo de rascunho, faça o diff contra o seu, e concilie campo a campo, porque o arquivo gerado acompanha o binário muito melhor do que qualquer tutorial consegue. E se o próprio `pay gate` tiver mudado além do reconhecível na hora em que você ler isto, degrade de forma graciosa exatamente como planejado: rode `pay curl` contra o middleware x402 da lição passada, e você ainda fica com a metade do lado client da lição, uma CLI negociando um 402 sem você escrever código de protocolo. Documente o caminho que você tomar; uma nota datada de "isto funcionou em 2026-08-22 com o pay 0.26.0 vindo do `@solana/pay` 1.0.26" é exatamente a disciplina que este módulo inteiro vem ensinando.

Dê um passo atrás e olhe o que a loja consegue fazer agora. Humanos pagam por páginas de checkout, barracas com QR code e blinks. Máquinas pagam por x402 e MPP, negociados por um gate que você configura em vez de código que você mantém. Uma costura honestamente ainda está aberta, e você deveria conseguir nomeá-la: as vendas que passam pelo middleware da lição passada se conciliam no livro-razão por id de fatura, porque você ligou esse hook você mesmo, enquanto a contabilidade `pooled` do gate não te entrega nenhum memo por chamada, então as chamadas roteadas pelo gate são trabalho de livro-razão ainda devido antes de entrar no ar. Todo o resto está montado. O comprador de discos de 1949 esperou a guerra de formatos passar; o fabricante de toca-discos de 1950 vendeu através dela. Você acabou de construir o aparelho de várias velocidades, e você sabe qual parafuso ainda está solto.

Uma pergunta continua aberta, e ela está escondida à vista desde a seção do MPP. O default do pull mode deixa o servidor co-assinar como fee payer, o que quer dizer que alguém que não é o comprador cobriu a taxa de rede. Generalize isso e você tem o assunto inteiro do próximo módulo: quem paga quando o cliente tem zero SOL? O papel de co-signatário que você acabou de conhecer cresce até virar o patrocínio da Kora e o checkout gasless, porque ninguém leva SOL para uma feira de discos. Vejo você lá.
