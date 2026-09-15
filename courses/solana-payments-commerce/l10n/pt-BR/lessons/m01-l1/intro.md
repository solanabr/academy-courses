# Veja um dólar liquidar: seus primeiros cinco minutos nos trilhos da Solana

## Resumo

Esta é a primeira lição, então não há nada para recapitular: nada foi construído ainda. Você chega fluente em trilhos de cartão e integração de PSP, o tipo de engenheiro que sabe como é uma tempestade de retries de webhook, e a gente vai abrir lendo dinheiro de verdade se mover on-chain antes de qualquer teoria. Nos próximos cinco minutos, sem carteira, sem chaves e sem cadastro, você vai ler um pagamento real denominado em dólar liquidar no livro-razão público da Solana. Depois, numa sessão mais longa, você vai gerar um QR code de pagamento que o seu próprio celular consegue escanear. Sem slides antes.

## Os trilhos que você consegue ler

Copie isto no seu terminal e rode. Funciona em qualquer máquina que tenha `curl`, ou seja, na sua:

```bash
curl -s -X POST https://api.mainnet-beta.solana.com \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getTransaction","params":["3qE5iCo5uGGfGXZd4rwfgJryLse6EpTA2SQMtX3X3GbWsS6hDRw8JwcYb4wpj77Aqj5mQBJg52LawnaZyT688kXA",{"encoding":"jsonParsed","maxSupportedTransactionVersion":1}]}'
```

Aquela parede de JSON que acabou de voltar é o pagamento liquidado de um estranho. Um de verdade. Alguém, em algum lugar, moveu USDC na mainnet da Solana, aquilo liquidou, e você acabou de puxar o registro completo disso de um endpoint público com uma requisição HTTP de uma linha. Ninguém perguntou quem você é. Ninguém poderia.

Fique com isso por um segundo, porque o seu cérebro de Stripe deveria estar coçando. No mundo do cartão, um pagamento liquidado é uma linha no banco de dados de um processador. Você vê as SUAS linhas, pelo SEU painel, depois que a SUA API key autentica você. A ideia de que você poderia buscar o registro de liquidação de um pagamento entre dois completos estranhos não é um bug de permissão que alguém esqueceu de fechar. Aqui, é o design.

Um punhado de definições para o JSON deixar de ser ruído, e depois a gente vai merecer as afirmações grandes. Elas nomeiam os campos em que a lição realmente se apoia; o resto daquela resposta (`slot`, `computeUnitsConsumed`, `innerInstructions`, `logMessages` e companhia) continua ruído por enquanto, de propósito, e o lab abre com um mapa de campos mostrando exatamente onde cada valor impresso mora dentro da árvore, então não saia caçando eles ainda.

- **mainnet**: a rede Solana ao vivo, onde moram os saldos de verdade. Existe também uma rede de testes gratuita chamada devnet, que é onde todo pagamento que você enviar neste curso vai cair. Hoje você só está lendo a mainnet, nunca escrevendo nela.
- **RPC endpoint**: a URL que você acabou de chamar é uma porta pública para o livro-razão. Qualquer um pode bater. E, crucialmente, a porta só abre para um lado em requisições como essa: a demo lê, ela nunca escreve. Você não consegue mover, reverter nem re-disparar os fundos de ninguém buscando a transação deles, assim como ler um recibo não gasta o dinheiro que está nele.
- **USDC**: uma stablecoin atrelada ao dólar emitida por uma empresa regulada, a Circle, que mantém reservas contra cada token em circulação e os resgata um para um. É essa a razão inteira de um valor em dólar significar alguma coisa nestes trilhos: 1 USDC é um direito sobre 1 dólar americano junto ao emissor, e o mercado precifica de acordo. Quando este curso diz "um dólar liquidou", quer dizer que um token USDC se moveu. A paridade é uma promessa do emissor, não uma lei da física, e precificar um negócio em cima dela é uma decisão de verdade que a gente vai tomar explicitamente na fronteira fiat.
- **endereço do mint do USDC**: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` é o identificador on-chain daquele token na Solana, a conta que define a própria moeda. Pense nele como o código da moeda, só que é um endereço globalmente único, não uma string de três letras. O USDC na Solana carrega 6 casas decimais, então o valor inteiro bruto `36115` naquele JSON significa 0.036115 USDC. Aquelas strings longas são **base58**, o alfabeto em que a Solana imprime endereços e signatures: dígitos e letras menos as que os humanos confundem, ou seja, sem `0`, `O`, `I` nem `l`.
- **`spl-token` e `transferChecked`**: `spl-token` é o programa on-chain que é dono desse saldo de USDC, e da maioria dos saldos de token na Solana, do jeito que um serviço de livro-razão é dono das linhas de saldo. A maioria, não todos: um segundo programa de token chamado Token-2022 é dono dos próprios saldos (o PYUSD mora lá, por exemplo), e o módulo 2 te ensina a checar a qual programa um mint pertence antes de decodificar qualquer coisa. `transferChecked` é a única instrução desse programa que este curso usa para mover tokens; a parte "checked" quer dizer que quem chama precisa declarar também o mint e a contagem de casas decimais dele, e o programa recusa se os dois não baterem. No JSON, o campo `authority` dele é a conta que autorizou o movimento, que num pagamento comum é a própria carteira do remetente. Os saldos em si moram em **contas de token**, uma por dono por moeda; o próximo módulo constrói elas direito.
- **`spl-memo` e o memo**: `spl-memo` é um segundo programa, minúsculo, cujo único trabalho é anexar uma nota legível por humanos a uma transação. A nota no seu JSON diz `079cea64791142a59e12a3491a425f90`, a referência interna de algum sistema. Mais para frente neste curso, os memos viram o jeito de um lojista casar um pagamento com um pedido. Guarde isso.

![Uma transação liquidada contendo uma instrução transferChecked do spl-token que move 0.036115 USDC e uma instrução spl-memo carregando a nota de referência, empacotadas sob uma única signature.](assets/v01-diagram.png)

Então a transação que você buscou se decompõe em: o remetente `BhFRCUXHVm76PmXkSzus8T4LUGrD2MTW9Au6bocBox5U` moveu 0.036115 USDC, com aquele memo anexado, e ela liquidou em 2026-08-22. Isso é 3.6 centavos, e o tamanho é o ponto em vez de um constrangimento: em trilhos de cartão uma transferência de 3.6 centavos não é um pagamento pequeno, é um pagamento impossível, porque o piso da taxa excede o valor. Aqui alguém moveu e a economia ainda fechou. A gente coloca um número exato nas taxas na próxima lição; por enquanto o ponto é que o registro é público, completo, e seu para ler.

Repare, também, no que NÃO está naquele registro, porque as ausências ensinam tanto quanto os campos. Não tem número de cartão, então não tem nada com formato de PCI para guardar em cofre. Não tem CVV, não tem validade, não tem endereço de cobrança, não tem campo dizendo "a janela de chargeback fecha em 120 dias". O endereço do remetente identifica uma chave, não uma pessoa, e é por isso que "quem de fato me pagou" vira uma pergunta diferente nestes trilhos do que era nos cartões, e casar pagamentos com clientes vai se apoiar naquele campo memo em vez de qualquer coisa parecida com o nome do portador do cartão. Por enquanto, só registre o formato: tudo que a liquidação precisa está presente, e nada do que a responsabilidade por fraude de cartão precisa está lá.

![Um terminal envia uma requisição HTTPS não autenticada para um endpoint RPC público, que lê uma transação liquidada do livro-razão compartilhado e devolve o registro JSON; o caminho é somente leitura.](assets/v02-diagram.png)

### A inversão que o seu PSP nunca ofereceu

Este é o formato do que você integra hoje. Um pagamento de cartão viaja por um adquirente, uma bandeira e um emissor, e o artefato que você recebe no fim é um webhook mais uma linha que você pode consultar, escopada na sua conta de lojista. A verdade da liquidação mora dentro do processador. Quando o financeiro pergunta "o pedido 4412 foi mesmo pago", a resposta é o que o painel disser, e o painel é infraestrutura privada que você aluga.

A Solana inverte isso. A verdade da liquidação mora num livro-razão público compartilhado, e a coisa com formato de processador no meio é opcional. Qualquer parte de um pagamento, ou qualquer terceiro curioso, consegue verificar a liquidação direto, por qualquer endpoint RPC, para sempre. O seu futuro código de conciliação neste curso não vai perguntar a um provedor "por favor, me diga se eu recebi". Ele vai ler o próprio livro-razão.

![Comparação lado a lado mostrando registros de liquidação de cartão como linhas privadas de processador atrás de uma API key, versus registros de liquidação da Solana como entradas de livro-razão público que qualquer um pode ler por qualquer RPC.](assets/v03-comparison.png)

Público por padrão é o divisor de águas aqui, e eu quero ser preciso sobre o porquê. Não é que público seja virtuoso. É que público mais legível por máquina colapsa categorias inteiras de trabalho de integração que você faz hoje: APIs de conciliação, exportação de relatório de liquidação, "fale com o suporte para rastrear este pagamento". O livro-razão é o relatório.

Torne isso concreto com o memo que você acabou de ler. Mais para frente neste curso, quando a Wavelength Records vender uma prensagem, o checkout carimba o id do pedido no memo da transferência, exatamente como a referência `079cea...` no seu JSON. Quando o financeiro perguntar "o pedido 4412 foi mesmo pago", a resposta não vai ser uma chamada de API para um provedor que pode estar com um incidente. Vai ser uma consulta contra o mesmo livro-razão público que você chamou dois minutos atrás, casando memo com pedido, e qualquer um que duvide da resposta pode rodar a mesma consulta. É o módulo de conciliação inteiro em uma frase; a gente vai gastar lições de verdade para merecer isso direito.

### O repo que mudou de lado

Segunda demo, e essa vem com uma reviravolta. A biblioteca oficial de pagamentos da Solana é publicada no npm como `@solana/pay`. Instale e algo inesperado aterrissa do lado. O passo 5 do lab roda essa instalação de novo com mais dois pacotes junto, então rode isto agora para a demo e rode mesmo assim o comando mais completo do passo 5 quando chegar lá:

```bash
npm i @solana/pay@1.0.26
npx pay --help
```

Um número de versão, dito uma vez e usado em todo lugar neste curso: o pacote npm é `@solana/pay` **1.0.26** (npm latest, checado em 2026-08-23; re-cheque com `npm view @solana/pay version`, porque este pacote se mexe). Junto com a biblioteca TypeScript pela qual você presumivelmente veio, o pacote larga um executável `pay` dentro de `node_modules/.bin`, que é o que `npx pay` encontra. Rode ele e, na primeira vez, ele baixa um binário nativo para a sua plataforma e depois te cumprimenta com um toolchain para, e eu cito o banner dele mesmo, "agentic payments": comandos para travar APIs atrás de pagamentos em stablecoin, embrulhar `curl` e agentes de IA de código para que eles possam pagar pelo que buscam, gerenciar contas, enviar stablecoins pela linha de comando.

Espere que o binário reporte um número diferente do pacote. `npx pay --version` imprime a linha de release do próprio CLI, que era 0.26.0 na minha máquina quando escrevi (o `npx` não é opcional aqui: este curso instala localmente, então o executável mora em `node_modules/.bin` e um `pay` puro é command-not-found até o módulo 7 instalar ele globalmente); a tag de CLI mais nova do repo era pay-v0.28.0, cortada em 2026-08-26 (verificada contra a API de releases do GitHub em 2026-09-07); ela se mexe mais ou menos todo mês, então cheque em vez de assumir. Duas linhas de versão em um pacote é confuso na primeira vez que você encontra e completamente normal depois: 1.0.26 é a biblioteca que você instala, e o número 0.2x é o executável baixado que ela publica junto. Nenhum dos dois está errado quando eles discordam.

A história por trás desse binário vale sessenta segundos, porque ela é o ecossistema inteiro em miniatura. Por anos, o repositório canônico do Solana Pay morou em solana-labs/solana-pay e o carro-chefe dele era o checkout por QR code: o cliente escaneia, a carteira paga, pronto. Hoje aquela URL do GitHub redireciona para um repo sob a Solana Foundation chamado simplesmente "pay", e o produto de destaque é o CLI de agentic payments que você acabou de cutucar. A energia de pagamentos da Foundation visivelmente se moveu de "humanos escaneando QR codes" para "agentes de software pagando por HTTP".

Leia o movimento com precisão, porque isso importa: nada foi deletado. O Solana Pay clássico, a biblioteca de checkout por QR, sobrevive como um subpacote dentro desse mesmo monorepo, mantido e publicado, e este curso constrói um checkout de verdade em cima dele no módulo 3, o módulo de superfícies de pagamento. Um destaque que mudou não é um produto removido. Aprender a ler movimentos de ecossistema nessa resolução, o que de fato mudou versus o que só deixou de ser o garoto-propaganda, é uma habilidade de sobrevivência em trilhos tão novos assim, e você vai ter bastante prática.

![Linha do tempo mostrando o repositório canônico do Solana Pay redirecionando para o repositório pay da Foundation, cujo destaque é um CLI de agentic payments, enquanto a biblioteca clássica de checkout por QR continua como subpacote o tempo todo.](assets/v04-timeline.png)

### Isto é dinheiro de verdade, não uma economia de demo

Pergunta justa neste ponto: alguma coisa disso é produção, ou é um sandbox com marketing bom? Vou te dar dois dados e deixar eles argumentarem por mim.

Primeiro, o incumbente: o "Pay with crypto" da Stripe aceita USDC na Solana no checkout e liquida o lojista em moeda fiduciária. Leia isso de novo da sua cadeira de integrador. O PSP mais mainstream do seu mundo trata estes trilhos como um método de pagamento de primeira classe, e absorve a parte cripto para que o lojista nunca segure um token. Sejam quais forem as suas crenças prévias sobre pagamentos em cripto, o time de risco da Stripe aprovou esta aqui.

Segundo, a trajetória. A oferta de stablecoin na Solana, ou seja, dólares tokenizados e parados nestes trilhos, foi de cerca de US$ 1.5 bi em dezembro de 2023 para US$ 11.7 bi em fevereiro de 2025 (segundo um artigo de pesquisa da Helius), e está em aproximadamente US$ 15.87 bi em 2026-08-23 (DefiLlama, contando só stablecoins atreladas ao dólar; este número se mexe todo dia, então trate qualquer valor que você ler, inclusive este, como um retrato datado). A própria documentação de pagamentos da Solana afirma que a rede processou mais de US$ 1 trilhão em volume de stablecoin em 2025. A oferta é o float; o volume é a vazão; as duas curvas apontam para o mesmo lado. O dinheiro foi para onde a liquidação era barata e rápida, do jeito que a água encontra o ralo. O float parado nestes trilhos cresceu mais de dez vezes em menos de três anos, e os incumbentes entraram atrás dele em vez de esperar passar.

![Gráfico da oferta de stablecoin na Solana subindo de 1.5 bilhão de dólares em dezembro de 2023 para 11.7 bilhões em fevereiro de 2025 e cerca de 15.87 bilhões em agosto de 2026.](assets/v05-chart.png)

### Estes trilhos estão se mexendo debaixo de você

Agora o trade-off, porque sempre tem um, e este curso vai nomear ele todas as vezes. A mesma virada de repo que deixa a demo empolgante está te dizendo que o chão não está parado. Versões giram. Ferramental gira. Até URLs canônicas giram: o repo que você teria salvo nos favoritos dois anos atrás agora redireciona para outro lugar. "Funciona hoje" é uma afirmação datada nestes trilhos, não uma garantia, e um curso que fingisse o contrário estaria mentindo para você com educação.

Então aqui vão as regras da casa para tudo que vem a seguir, ditas uma vez e cumpridas do começo ao fim:

- **Todo pin de versão vem com uma nota de frescor.** Quando eu digo `@solana/pay` 1.0.26, eu te digo quando aquilo era verdade e como re-checar. Versões fixadas em scripts são deliberadas, porque uma demo sem pin pode quebrar em silêncio quando uma dependência transitiva publica uma mudança incompatível.
- **Afirmações datadas são datadas.** Números de oferta, comportamento de produto, layout de repo: cada um chega com a data de referência e a fonte. Quando você bater numa divergência, e um dia você vai bater, a data te diz se o curso está velho ou se o seu ambiente está.
- **Infraestrutura ao vivo ganha um fallback.** O RPC público de mainnet que você usou acima vai te aplicar rate limit se você varrer com agressividade, o que é justo, ele é de graça. É exatamente por isso que esta lição fixa uma signature de transferência real e re-verificada: a demo dos primeiros cinco minutos decodifica aquela transação mesmo quando a varredura ao vivo é estrangulada. Você vai ver o padrão no script do lab.

### Para onde este curso vai, e a loja que a gente está construindo

Você já tocou nas duas pontas do arco: leu um pagamento liquidado e conheceu a ferramenta que deixa software pagar por chamadas HTTP. O curso entre esses dois pontos abre no módulo 2 construindo o próprio kit de transferência, as contas de token, as casas decimais e as primitivas de enviar-e-verificar que toda lição posterior importa, e depois corre em seis movimentos. Módulo 3, superfícies de pagamento: checkout por QR, links de pagamento, os caminhos de vitrine que um cliente humano de fato toca. Módulo 4, operações do lojista: política de confirmação, webhooks, conciliação contra o livro-razão público que você acabou de ler, e reembolsos, que nestes trilhos são um pagamento novo na direção contrária em vez de uma reversão. Módulo 5, receita recorrente: assinaturas em trilhos que não têm cartão salvo. Módulo 6, a fronteira fiat: on-ramps, off-ramps, precificação de exibição e as pontes com formato de Stripe entre estes trilhos e a sua contabilidade. Módulo 7, pagamentos de máquina: o lado agentic que você vislumbrou no CLI `pay`, onde o cliente é software. Módulo 8, endurecimento para produção: checkout gasless, vendas offline, fazer transações aterrissarem quando a rede está ocupada. O módulo 9 é o capstone, onde cada peça roda como uma loja só.

A gente constrói tudo isso para um lojista só. Conheça a **Wavelength Records**, uma loja independente de vinil que existe só neste curso: loja online, uma maquininha (POS) de barraca de feira, um clube de assinatura do disco do mês e, mais para frente, uma API que cota preços de prensagem para outros negócios. Cada lição acrescenta uma peça real à stack da Wavelength, e no fim você vai ter construído uma operação de comércio de ponta a ponta nos trilhos da Solana, não uma pilha de snippets desconexos. Quando você gerar um QR no lab de hoje, ele vai estar denominado como um pedido da Wavelength, porque é um.

![Mapa do curso em seis etapas, de superfícies de pagamento passando por operações, receita recorrente, a fronteira fiat, pagamentos de máquina e endurecimento para produção, tudo ancorado na construção da loja da Wavelength Records de ponta a ponta.](assets/v06-flowchart.png)

Mais uma peça de orientação antes do lab, sobre como as lições deste curso te entregam trabalho. Cada lição faz a ajuda recuar em três passos, em voz alta: a visão geral que você acabou de terminar é legível sem tocar num teclado; o lab a gente faz junto, passo a passo, com saída esperada em cada checkpoint; o desafio no fim você faz sozinho, e é a parte que faz a lição grudar. Hoje o lab é deliberadamente leve, e repare no que ele não faz: nada de carteira até bem no fim. Você já leu a mainnet sem uma, que é o ponto. A carteira chega só quando você tiver construído algo que valha a pena escanear.

## Lab: nos trilhos, de ponta a ponta

Os passos 1 a 4 são os cinco minutos prometidos lá em cima: checagem de toolchain, pasta, script, rodar. Os passos 5 a 8 são uma sessão mais longa, porque incluem um npm install, o download de um binário na primeira execução, instalar uma carteira de uma loja de aplicativos e escrever uma frase de recuperação no papel. Reserve quarenta minutos para a coisa toda e não deixe a promessa de abertura te apressar pelo ritual da carteira.

A gente vai decodificar uma transferência direito com um script, gerar um QR code de pagamento da Wavelength e só então configurar uma carteira e escanear o nosso próprio QR com ela. Antes do script, um mapa: o JSON cru que o seu curl devolveu aninha os campos interessantes alguns níveis abaixo, e o decodificador que você está prestes a escrever não é nada além de uma caminhada até esses pontos. Aqui é onde mora cada campo impresso.

![Uma árvore JSON abreviada do getTransaction com setas de chamada mapeando blockTime para settledAt, a instrução transferChecked do spl-token para sender, amount e mint, e a instrução spl-memo para a string do memo.](assets/v07-annotated-code.png)

1. **Cheque o seu toolchain.** Você precisa de Node 24 ou mais novo, que é o que todo lab deste curso assume daqui para frente. `node -v` deve imprimir v24.x ou maior; se imprimir algo mais velho, atualize agora em vez de na primeira instalação que recusar. Os scripts abaixo rodam via `tsx`, um runner de TypeScript sem configuração; a gente invoca ele por `npx` com uma versão fixada (`tsx@4.23.12`, npm latest em 2026-08-23), então o próprio `tsx` não precisa de instalação.

2. **Crie uma pasta de trabalho e um manifesto.** Isto vira o espaço de rascunho da Wavelength, e todo comando daqui até o fim da lição roda dentro dela:

   ```bash
   mkdir wavelength-rails && cd wavelength-rails
   npm init -y
   ```

   Resultado esperado: um diretório contendo exatamente um arquivo, um `package.json` que o npm gerou com os padrões. Ele parece vazio e isso está correto; o passo 5 preenche as dependências. A gente cria ele explicitamente em vez de deixar o `npm install` conjurar um depois, para que nada na sua pasta seja surpresa.

3. **Salve o decodificador.** Crie `watch-a-dollar.ts` com exatamente este conteúdo:

   ```ts
   // watch-a-dollar.ts: read a settled USDC transfer straight off Solana mainnet.
   // No wallet, no keys, no signup. Run: npx -y tsx@4.23.12 watch-a-dollar.ts

   const RPC = "https://api.mainnet-beta.solana.com";
   const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC on Solana, 6 decimals

   // A real USDC transfer, pinned as a fallback in case the public RPC rate-limits
   // the live scan. Re-verified on mainnet 2026-08-23.
   const FALLBACK_SIG =
     "3qE5iCo5uGGfGXZd4rwfgJryLse6EpTA2SQMtX3X3GbWsS6hDRw8JwcYb4wpj77Aqj5mQBJg52LawnaZyT688kXA";

   async function rpc(method: string, params: unknown[]) {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const json = await res.json();
     if (json.error) throw new Error(json.error.message);
     return json.result;
   }

   async function decode(signature: string) {
     const tx = await rpc("getTransaction", [
       signature,
       { encoding: "jsonParsed", maxSupportedTransactionVersion: 1 },
     ]);
     if (!tx) return null;
     const instructions = tx.transaction.message.instructions;
     // Deliberately narrow: USDC is an spl-token mint. A Token-2022 mint
     // (PYUSD, say) reports a different program here and this filter would
     // skip it -- module 2 builds the detector that handles both.
     const transfer = instructions.find(
       (ix: any) =>
         ix.program === "spl-token" &&
         ix.parsed?.type === "transferChecked" &&
         ix.parsed.info.mint === USDC_MINT,
     );
     if (!transfer) return null;
     const memoIx = instructions.find((ix: any) => ix.program === "spl-memo");
     return {
       signature,
       settledAt: new Date(tx.blockTime * 1000).toISOString(),
       sender: transfer.parsed.info.authority,
       amountUSDC: transfer.parsed.info.tokenAmount.uiAmountString,
       mint: transfer.parsed.info.mint,
       memo: memoIx ? memoIx.parsed : "(none attached)",
     };
   }

   async function main() {
     // Live scan: recent transactions that touched the USDC mint account.
     try {
       const sigs = await rpc("getSignaturesForAddress", [USDC_MINT, { limit: 20 }]);
       for (const s of sigs.filter((s: any) => s.err === null)) {
         const result = await decode(s.signature);
         if (result) {
           console.log("LIVE: a stranger's payment, settled moments ago.");
           console.log(result);
           return;
         }
       }
     } catch {
       console.log("Live scan rate-limited. Falling back to the pinned transfer.");
     }
     console.log("PINNED: a real settled transfer (verified 2026-08-23).");
     console.log(await decode(FALLBACK_SIG));
   }

   main();
   ```

   Resultado esperado: um arquivo, `watch-a-dollar.ts`, parado em `wavelength-rails`. Nada rodou ainda.

4. **Rode.**

   ```bash
   npx -y tsx@4.23.12 watch-a-dollar.ts
   ```

   Saída esperada: ou um bloco `LIVE` com uma transferência de USDC que liquidou segundos atrás, ou o bloco `PINNED` mostrando o remetente `BhFRCUXHVm76PmXkSzus8T4LUGrD2MTW9Au6bocBox5U`, valor `0.036115` USDC, o mint do USDC e o memo `079cea64791142a59e12a3491a425f90`. Os dois são igualmente reais; o fixado só tem garantia de estar lá. Quando eu rodei o caminho ao vivo enquanto escrevia isto, ele pegou uma transferência de 0.10 USDC que tinha liquidado vinte segundos antes, o que nunca deixa de ser um pouco surreal. **Checkpoint: você tem um objeto decodificado com signature, settledAt, sender, amountUSDC, mint e memo impressos no seu terminal.** Se você vir `Live scan rate-limited` primeiro, é o RPC público gratuito fazendo exatamente o que a seção de teoria avisou, e o fallback é a lição funcionando como projetada, não falhando.

5. **Instale a biblioteca de pagamentos e conheça o clandestino.** Ainda em `wavelength-rails`:

   ```bash
   npm i @solana/pay@1.0.26 @solana/kit@6.10.0 qrcode-terminal@0.12.0
   npx pay --help
   ```

   `@solana/pay` 1.0.26 e `qrcode-terminal` 0.12.0 são npm latest em 2026-08-23. `@solana/kit` está fixado em 6.10.0 de propósito, e o motivo é o seu primeiro gosto ao vivo da rotatividade que as regras da casa acabaram de avisar: a tag `latest` do kit estava na linha 8.x quando eu chequei (8.2.0, publicada em 2026-08-29 — rode `npm view @solana/kit version` e espere um número diferente), enquanto `@solana/pay` 1.0.26 declara um range de peer de `^6.9.0`, então 6.10.0 é a release de kit mais nova que ele de fato aceita. A distância é o fato durável aqui; o dígito exato na frente do pelotão não é. A gente instala ele explicitamente em vez de se apoiar no auto-install de peer do npm porque `qr.ts` importa dele direto, e tudo que você importa pertence ao seu próprio `package.json`. O primeiro comando instala a biblioteca oficial de pagamentos mais um pequeno renderizador de QR para o terminal. Ele também, como conversado, larga o executável `pay` em `node_modules/.bin`, que é o que `npx pay --help` encontra: na primeira execução ele baixa o build do CLI para a sua plataforma e depois te mostra o toolchain de agentic payments. Passe os olhos no texto de ajuda, repare em `gate`, `curl`, `send` e `account`, e siga em frente. A gente não vai usar ele hoje; você só precisava ver que ele é real.

   Esta é uma instalação local, e local é como todo workspace que você construir neste curso instala as dependências dele; ignore qualquer conselho em outro lugar de acrescentar `-g` aqui. O módulo 7 é a única exceção deliberada, instalando o CLI `pay` globalmente para que um `pay` puro fique no seu PATH no trabalho de gate daquela lição, e ele avisa quando faz isso. **Checkpoint: `npm ls --depth=0` lista exatamente três dependências, e `npx pay --help` imprime um banner nomeando "agentic payments" seguido de uma lista de comandos incluindo `gate`, `curl`, `send` e `account`.** Se em vez disso você receber "command not found" ou o npx se oferecer para instalar um pacote chamado `pay` do registro, o executável não aterrissou: rode `ls node_modules/.bin/pay` para confirmar e, se estiver faltando, rode a instalação de novo e leia a saída dela atrás de uma falha de download (o binário é baixado pela rede na primeira execução, então um proxy ou uma máquina offline te para aqui). Se `npx pay --version` imprimir um número `0.2x` em vez de 1.0.26, nada está errado; essa é a linha de release do próprio executável, como a seção de teoria avisou. Mantenha o `npx`: um `pay` puro aqui falha com command-not-found por um motivo chato (instalações locais colocam binários em `node_modules/.bin`, não no seu PATH) que parece exatamente com a falha que este parágrafo está te mandando diagnosticar.

6. **Gere a primeira solicitação de pagamento da Wavelength.** Crie `qr.ts`:

   ```ts
   // qr.ts: turn a Solana Pay request into a scannable QR, right in your terminal.
   // Run: npx -y tsx@4.23.12 qr.ts
   import { encodeURL } from "@solana/pay";
   import { address } from "@solana/kit";
   import qrcode from "qrcode-terminal";

   const url = encodeURL({
     recipient: address("4NDXfTUeUnCVvzTvGVUAEBAHzWkadwv2zubvBHEHgmVi"), // Wavelength's till; leave it alone, step 8 explains why
     amount: 24, // UI units: twenty-four whole USDC, NOT 24000000 base units
     splToken: address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC on Solana
     memo: "WAV-0001", // Wavelength order reference
   });

   console.log(url.toString());
   qrcode.generate(url.toString(), { small: true });
   ```

   Esse `amount: 24` merece um aviso, porque contradiz o reflexo que o parágrafo das casas decimais acabou de instalar em você. Antes, lendo a blockchain, o inteiro bruto `36115` significava 0.036115 USDC, porque valores on-chain são sempre unidades base e você escala pelas casas decimais do mint. O campo `amount` do Solana Pay é a convenção oposta: ele é um **UI amount**, o número que um humano digitaria, então `24` significa vinte e quatro USDC inteiros. Escrever `24000000` aqui pediria vinte e quatro milhões de dólares ao seu cliente. As duas APIs diferem porque servem leitores diferentes: a blockchain fala em inteiros para que nenhum arredondamento de ponto flutuante jamais encoste num saldo, enquanto uma solicitação de pagamento é um documento voltado para humanos e a spec escolheu unidades voltadas para humanos. A regra para levar adiante é simplesmente que todo valor que você manipula tem uma unidade declarada, e você checa isso em cada fronteira em vez de assumir.

   Rode com `npx -y tsx@4.23.12 qr.ts`. Saída esperada: uma URL `solana:` carregando o destinatário, `amount=24`, o mint do USDC como `spl-token` e `memo=WAV-0001`, seguida de um QR escaneável desenhado em ASCII. Essa URL é a solicitação de pagamento inteira: 24 USDC para um endereço específico, etiquetada com uma referência de pedido. O `encodeURL` construiu ela; o qrcode-terminal só desenhou (crédito a quem merece, esse pacotinho é discretamente útil há uma década). **Checkpoint: um QR renderiza no seu terminal e a URL acima dele contém `memo=WAV-0001`.**

7. **Agora, e só agora, instale uma carteira.** No seu celular, instale uma carteira Solana padrão: Phantom e Solflare são as escolhas comuns, as duas na loja de aplicativos da sua plataforma. Crie uma carteira nova e leve o ritual da frase de recuperação a sério mesmo que esta carteira não vá guardar nada hoje: escreva a frase, no papel, nunca num print. Dois minutos, e agora você segura o tipo de chave que assinou a transferência que você decodificou no passo 4. **Checkpoint: o app da sua carteira mostra saldo zero e um endereço que você pode copiar; esta é a carteira de celular que escaneia e liquida a venda presencial quando o módulo 3 põe uma maquininha no seu balcão.**

8. **Escaneie a solicitação.** Na sua carteira, copie o seu endereço novo (a carteira chama ele de endereço ou chave pública; é uma string base58 como as que você vem lendo a lição inteira). Agora deixe o `qr.ts` apontado para o destinatário placeholder e simplesmente rode de novo, depois aponte a carteira do seu celular para o QR na sua tela. A carteira parseia a URL e mostra uma prévia de pagamento: 24 USDC para aquele destinatário, memo anexado, e uma recusa em prosseguir porque a sua carteira novinha não guarda nada. Não pague nada; a prévia é a linha de chegada.

   Mantenha o destinatário como alguém que não seja você nesta prévia. Apontar uma solicitação para o seu próprio endereço é o único caso em que as carteiras discordam: a Phantom hoje renderiza uma prévia de transferência para si mesmo com um banner de aviso, a Solflare recusa de cara, e as duas ainda vão reclamar separadamente que você não tem SOL para cobrir a taxa de rede. Nada disso é bug seu, são só três avisos sem relação empilhados em uma tela, e isso encobre a coisa que você veio ver. **Checkpoint: a sua própria carteira, escaneando um QR que o seu próprio código gerou, exibe corretamente o destinatário, o valor `24 USDC` e o memo do pedido `WAV-0001`.** Você agora já esteve dos dois lados do balcão.

![Fluxo do script qr.ts passando pela URL codificada e pelo QR no terminal até uma carteira de celular que parseia a URL e mostra a prévia de um pagamento de 24 USDC sem pagar.](assets/v08-flowchart.png)

## Challenge: a Rosetta dos trilhos de cartão

Este aqui você faz sozinho; foi o acordo que a gente fez na visão geral. Pegue o objeto decodificado do passo 4 (ao vivo ou fixado, tanto faz) e anote cada campo com o equivalente dele nos trilhos de cartão e uma frase sobre onde a analogia se sustenta e onde ela vaza. Trabalhe a partir do que você observou, não de um buscador.

Formato da resposta, com uma linha já feita para você:

| Campo do livro-razão | Equivalente nos trilhos de cartão | Onde a analogia vaza |
|---|---|---|
| `signature` | id de transação | Um id de txn de cartão é escopado ao sistema de um processador; esta signature é globalmente única e qualquer um pode consultar ela no livro-razão compartilhado. |
| `sender` | ? | ? |
| `amountUSDC` | ? | ? |
| `mint` | ? | ? |
| `memo` | ? | ? |
| `settledAt` | ? | ? |

Preencha as cinco linhas restantes. Âncoras de sanidade, para você se corrigir sozinho: amount mapeia para o valor autorizado, mint para a moeda, memo para uma referência de pedido. A coluna interessante é a terceira; "sender", em particular, deveria te fazer pensar no que uma bandeira te mostra sobre um pagador versus o que este livro-razão acabou de te mostrar sobre um completo estranho.

Uma tabela completa mais as demos do lab rodando é a trava desta lição. Se a terceira coluna de toda linha disser "nenhuma diferença", volte e olhe com mais atenção; se você achou um vazamento em cada uma das linhas, você já entendeu mais sobre estes trilhos do que a maioria dos guias de integração jamais vai te contar.

Você atravessou a primeira lição inteira, demos e tudo, então deixa eu nomear o limite dela: o que você tem hoje é observação, ainda não um modelo. Você viu dinheiro liquidar e escaneou um QR que o seu próprio código construiu, e se a inversão do livro-razão público ainda parece levemente ilegal, ótimo, esse instinto quer dizer que você entendeu. A próxima lição, "Finality vs. a pilha de cartões: o modelo mental de pagamentos", fornece o porquê: o modelo que explica por que estes trilhos se comportam de outro jeito, onde o seu vocabulário de PSP mapeia limpo em `processed`, `confirmed` e `finalized`, e o único reflexo de cartão que vai te custar mais caro se você mantiver. Traga a transação decodificada do lab de hoje; a gente vai interrogar o que "liquidado" de fato significava.
