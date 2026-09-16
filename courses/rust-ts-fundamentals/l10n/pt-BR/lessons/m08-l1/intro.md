# Uma blockchain é um alvo: o modelo mínimo + as primeiras leituras

## Resumo

O M7 fechou o tier do edge: o motor Rust roda como WASM numa segunda URL workers.dev, e os dois edge workers já sondam o endpoint getHealth da Solana como um alvo entre vários. A rampa estrutural está pronta. Hoje o tier da Solana abre, e ele abre do jeito que este curso abriu: medindo alguma coisa. Você vai construir o `chain-probe.ts`, um script de bancada que lê os sinais vitais de uma blockchain ao vivo, mede o batimento real dela ao longo de 20 amostras, compara isso com a meta publicada de 300ms da rede, e deriva uma contagem regressiva de epoch em tempo de relógio a partir de uma única constante fixa. No caminho você ganha o menor modelo útil do que uma blockchain sequer é para um cliente: quatro ideias, uma frase cada, com tudo o que é mais fundo repassado por nome. Uma palavra sobre como o M8 funciona: guiado-mas-conduzido-pelo-aprendiz. Eu trabalho a configuração do RPC e uma amostra na tela; você escreve a agregação de 20 amostras, a linha de meta-versus-medido, e a aritmética de epoch você mesmo a partir de contratos declarados, e a extensão de min/max no fim é só sua. Você já entregou sete módulos de sondas. Você não precisa de mim pairando em cima.

## Meça alguma coisa primeiro

A sua estação vem sondando `https://api.mainnet.solana.com` desde que o m07-l1 colocou ele na lista de alvos do worker, e até agora a relação foi rasa no transporte: um POST que ou responde na hora ou, como o m07-l1 documentou, cumprimenta o seu isolate com um 403 e é trocado pelo fallback do publicnode. De todo jeito, ninguém fez uma pergunta de verdade para ele ainda. Cole isto no seu terminal agora:

```bash
curl -sS https://api.mainnet.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

```
{"jsonrpc":"2.0","result":"ok","id":1}
```

Isso é uma blockchain respondendo a uma pergunta JSON sobre HTTP puro. Sem carteira, sem chave, sem taxa, sem conta em lugar nenhum. A mesma forma de POST-com-corpo-JSON que você vem mandando desde as lições de HTTP do M2, apontada para uma rede que já processou mais de meio trilhão de transações (o próprio getTransactionCount dela me disse isso enquanto eu escrevia), e ela responde a qualquer um que pergunte. Agora faça isso do TypeScript, no repo da estação. A estação é um workspace pnpm desde o m03-l1, então as instalações passam pelo pnpm (a flag `-w` diz "sim, eu quero mesmo a raiz do workspace", para que todo pacote no workspace consiga resolver a instalação; um `npm i` perdido aqui rabiscaria um package-lock.json concorrente dentro de um repo pnpm):

```bash
pnpm add -w @solana/kit@^8
```

Nota de freshness: isso resolve para 8.2.0 em 2026-09-02, e o dígito importa mais que o normal aqui. O kit entregou dois majors em pouco mais de nove semanas neste verão (7.0.0 no fim de junho, poucas semanas depois do minor 6.10.0, depois 8.0.0 no fim de agosto), que é exatamente por que a regra deste curso é fixar aquilo contra o que as suas dependências dão peer e reconferir as linhas de install no dia em que você roda elas, não no dia em que um tutorial foi escrito.

O script em si vai onde moram os scripts de bancada da estação: `packages/pulse-fleet/`, ao lado do `probe.ts` e do `fleet.ts`. Essa colocação é estrutural, não arrumação: o pulse-fleet carrega o campo `"type": "module"` e o runner `tsx` de que estes scripts precisam, e a raiz do workspace não tem nenhum dos dois, então `npx tsx` rodado ali morre no top-level await ("Top-level await is currently not supported with the cjs output format"). Crie `packages/pulse-fleet/first-read.ts`:

```typescript
// packages/pulse-fleet/first-read.ts
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

console.log(await rpc.getHealth().send());
console.log(await rpc.getSlot().send());
```

```bash
cd packages/pulse-fleet
npx tsx first-read.ts
```

```
ok
443692227n
```

Três linhas funcionando. O sufixo `n` naquele segundo número é o kit te passando um `bigint`, porque contagens de slot são valores u64 e o kit se recusa a mentir sobre isso no nível de tipos. Arquive isso: hoje é um agrado, na próxima lição, quando os números forem saldos, é a diferença entre correto e silenciosamente errado. E aquele número grande em si? Aquele é o contador de batimento da blockchain, e o resto desta lição é sobre o que ele quer dizer e com que velocidade ele bate.

## O modelo mínimo, derivado

### O que um cliente de fato precisa saber?

Aqui está a pergunta que dá forma a este tier inteiro. A sua estação quer exibir dados ao vivo da Solana. Qual é o mínimo que você precisa entender sobre uma blockchain para fazer isso com honestidade?

A resposta máxima é um currículo: consenso, validadores, o modelo de contas como sistema, execução de programas, assinaturas criptográficas, mercados de taxa. Tópicos de verdade, e este catálogo ensina eles, mas não aqui. Exigir tudo isso antes de uma primeira leitura é como os tutoriais perdem gente na semana um e, pior, é desnecessário: você acabou de ler estado ao vivo da blockchain com três linhas e nenhum desse conhecimento.

A resposta mínima ingênua também falha, porém. "É só uma API" te deu a sonda getHealth no M7, mas ela desmorona no momento em que você faz uma segunda pergunta. Por que aquela leitura não custou nada se todo mundo diz que blockchains têm taxas? Por que a página de docs fala de contas e slots e epochs? Uma API cujo vocabulário você não consegue parsear é uma API que você vai usar errado. Então o mínimo honesto fica no meio: você precisa exatamente das ideias que deixam o lado de leitura da API legível, e de nada do que um autor de programa precisa. São quatro, e a regra de seleção vale ser dita porque é a mesma regra 80/20 que este curso já aplicou a duas linguagens: uma ideia entra na lista só se uma pergunta que você vai pessoalmente encontrar antes do fim deste módulo forçar ela. Não "importante para blockchains." Forçada, pelo seu próprio código, esta semana.

**Contas guardam lamports.** Uma conta é um endereço com um saldo e alguns dados. O saldo é denominado em lamports, a menor unidade de SOL: 1 SOL é 1,000,000,000 lamports, um inteiro, sem decimais no nível do ledger (a lição de matemática de dinheiro do M2 te disse por quê). Esse é o substantivo do sistema. Uma linha no node prova a forma:

```typescript
const LAMPORTS_PER_SOL = 1_000_000_000n;
console.log(2n * LAMPORTS_PER_SOL); // 2000000000n: two SOL, as the ledger stores it
```

**Programas são código.** A lógica que movimenta saldos por aí mora em programas. Um programa também é uma conta, uma cujos dados por acaso são código executável. Anotado, não explorado: essa uma frase é tudo de que o lado do cliente precisa.

**Transações mutam.** O estado muda de exatamente um jeito: uma transação assinada. Leituras são perguntas grátis; escritas são eventos assinados, que pagam taxa e passam por consenso. Esse é o verbo do sistema, e esta lição não contém nenhuma delas.

**RPC é a porta de leitura.** Um nó de RPC guarda uma cópia do estado da blockchain e responde perguntas JSON sobre ele, que é o que você acabou de fazer duas vezes. Sem transação, sem taxa, sem carteira. A porta na qual você vem batendo desde o M7.

Esse é o modelo inteiro, e a contagem sobrevive à pressão dos dois lados. Tente encolher ele para três derrubando "programas são código" e a primeira página de explorer que você abrir para de fazer sentido: metade das contas nela está marcada como executável e você não tem nenhum escaninho na cabeça para o que isso quer dizer. Tente crescer ele para cinco, com PDAs, digamos, ou contas de token, e você vai descobrir que nada no código deste módulo jamais toca a quinta ideia, o que pela regra de seleção desqualifica ela. Quatro não é um número redondo de que eu gostei; é o que as perguntas forçam.

Onde os saldos de token de fato moram, por que endereços podem ser derivados, o que exatamente um programa pode e não pode fazer, como funciona a história da blockchain? Todas perguntas de verdade, todas deliberadamente fora destas quatro frases, e todas com um irmão nomeado como dono: o curso de evolução do Bitcoin para a Solana neste catálogo percorre o modelo de contas como sistema, execução de programas, PDAs e a história da blockchain a partir de primeiros princípios. Esta lição ensina o que um cliente precisa. Fingir que quatro frases cobrem o resto é como tutoriais produzem confusão confiante, então eu não vou, e o repasse tem um nome no lugar.

![Quatro ideias dispostas em torno de um hub, contas, programas, transações e RPC, com uma seta tracejada passando tópicos mais profundos para outro curso.](assets/v01-diagram.webp)

### Leituras são perguntas, escritas são eventos

Das quatro ideias, a assimetria entre ler e escrever é a que dá forma a este módulo inteiro, então ela merece um segundo olhar antes de a gente começar a medir.

Quando a sua sonda chamou getSlot, nenhum validador registrou que isso aconteceu. Nada foi assinado, nada foi pago, nada tocou o consenso. Um nó de RPC olhou para a própria cópia de estado dele e respondeu, do jeito que qualquer servidor web responde a um GET. É por isso que o endpoint público pode ser grátis e aberto: responder perguntas é barato. As taxas existem para pagar pela coisa cara, que é mutar estado replicado com que milhares de máquinas precisam concordar. Uma escrita é uma transação assinada que compete por inclusão em um bloco; uma leitura nunca vira uma transação, ponto.

A consequência prática para a estação: três lições deste módulo são leituras grátis a partir de duas linguagens, e depois uma escrita cuidadosamente conquistada. A devnet, o cluster de treino onde essa escrita vai acontecer, é a única plataforma nova deste módulo, e ela chega no m08-l4 com um keypair e um faucet (um signer baseado em arquivo, deliberadamente não uma carteira; o m08-l4 deixa essa distinção afiada). Hoje e na próxima lição a gente fica do lado de leitura da porta, na mainnet, onde o pior que você pode fazer é perguntar coisas demais rápido demais. O que, como você vai ver daqui a pouco, o endpoint tem opiniões a respeito.

![Duas pistas comparam uma leitura grátis sem assinatura respondida a partir do estado do nó com uma escrita assinada que paga taxa e passa pelo consenso.](assets/v02-flowchart.webp)

### O batimento acelerou na semana em que este curso foi pesquisado

Agora o vocabulário para o que o getSlot de fato devolveu. Um slot é a unidade de agendamento da rede, o batimento dela: a cada slot, um validador tem o direito de produzir um bloco. Slot e bloco não são sinônimos, o slot é o tique do relógio e o bloco é o que aterrissa nele, mas para os fins de um cliente o contador de slot é o relógio, e ele só sobe. Uma epoch é 432,000 slots, uma constante fixa da rede, e clusters é a palavra para as redes em si: mainnet, onde o valor mora, mais devnet e testnet para treino e staging.

Com que velocidade bate o batimento? É aqui que a lição ganha uma data em cima. Em 2026-08-28, quatro dias antes da varredura de pesquisa deste curso, a stage 2 do SIMD-0525 ativou na mainnet e levou a meta de tempo de slot de 400ms para 300ms. Isso é um quarto a menos no intervalo, o que é um terço a mais de slots por segundo: a blockchain que você está sondando acelerou na semana em que este material foi escrito. Um curso de fundamentos lançando agora ensina uma blockchain cujo batimento mudou semana passada, o que te diz alguma coisa sobre por que todo número neste curso carrega uma data.

Um footgun em como esse fato é citado, porque ele vai morder qualquer um que confira as fontes. O frontmatter do próprio documento do SIMD-0525 ainda dizia Draft enquanto a mudança estava ao vivo na mainnet. A prova não é a spec, é a blockchain, e os dois números que o challenge vai cutucar merecem linhas próprias:

- O feature gate da stage 2 registra um slot de ativação de **441,936,000**. Divida por 432,000 e você tem exatamente 1023, sem resto: aquele slot é o tique de abertura da epoch 1023.
- O comportamento ligou no começo da epoch **1024**, uma fronteira depois, porque é assim que os feature gates da Solana funcionam: a ativação aterrissa durante uma epoch, a chave vira na fronteira seguinte. Guarde esse atraso de uma epoch na cabeça.

Qualquer um pode decodificar a conta do gate a partir do RPC público; eu reconferi enquanto escrevia isto em 2026-09-02, continua lá, continua ativo, uma chamada de getAccountInfo. Cite o gate on-chain, nunca a linha de status de uma spec. Existe também uma stage 3 mirando 250ms; na data em que escrevo isto o gate dela não estava ao vivo, e a medição que você está prestes a tomar vai confirmar que a blockchain ainda roda no ritmo da stage 2. Se a stage 3 aterrissar depois que esta lição for entregue, o seu medidor fica mais interessante, não errado. Esse é o ponto de construir um medidor em vez de decorar um número.

![Uma linha do tempo corre da ativação de agosto passando por duas medições datadas até uma seta aberta para a sonda do próprio leitor.](assets/v03-timeline.webp)

### Metas versus medições

Então a rede diz 300ms. É isso que ela faz?

A varredura de pesquisa perguntou, do jeito honesto: getRecentPerformanceSamples, um método de RPC que devolve os tempos de slot registrados pelo próprio nó em janelas de 60 segundos. Vinte amostras, na média, em 2026-09-01: 316ms. Não 300. Uns cinco por cento acima da meta, e essa diferença não é um escândalo, é a cara que sistemas ao vivo têm. Uma meta é uma intenção de engenharia; uma medição é o que aconteceu, clima de rede incluso. A minha própria re-execução enquanto escrevia esta lição voltou 313.6ms. Mesma história, dia diferente.

A objeção afiada primeiro, porque você deveria estar levantando ela: essa diferença de 16ms é real, ou é a sua própria ida e volta de HTTP vazando para dentro dos números? Ela é real, e vale ser dono do motivo antes do lab. O getRecentPerformanceSamples não cronometra nada do seu lado do fio; ele devolve o histórico registrado pelo próprio nó, quantos slots de fato aconteceram em cada janela de 60 segundos que ele já logou. A latência da sua conexão decide quando você recebe esses registros, não os valores dentro deles. Existe um esquema de medição em que a sua latência de fato poluiria o resultado, e você vai construir ele no lab como um descarte deliberado, precisamente para você sentir a diferença entre cronometrar uma coisa você mesmo e pedir os logs de um sistema.

Se isto parece familiar, deveria. É a lição mais velha deste curso vestida de blockchain: o M1 fez você medir a sua própria latência em vez de confiar num número num README, e o M2 te ensinou que declarado e medido são colunas diferentes. Agora o sistema sob teste é uma blockchain, e a disciplina transfere sem mudança. O seu próprio painel faz afirmações de uptime; a versão honesta da sua estação publica o que ela mede, não o que ela espera. A realidade dura é: todo sistema que você um dia vai operar tem uma diferença entre a meta dele e o comportamento dele, e os times que sabem o tamanho da diferença deles são aqueles em que você pode confiar.

Para uma estação que passou sete módulos aprendendo a medir os endpoints dos outros, um alvo que já vem com a própria API pública de medição parece receber a folha de respostas. A maior parte da infraestrutura te faz adivinhar. Esta blockchain te passa o getRecentPerformanceSamples e te desafia a conferir.

![Três barras mostram a meta de 300 milissegundos ao lado de medições de 316 e 313.6, uma diferença de aproximadamente cinco por cento.](assets/v04-chart.webp)

### O relógio de epoch cai da aritmética

Aqui está a parte que eu acho silenciosamente encantadora. As epochs são definidas em slots, não em tempo: 432,000 slots, sempre, antes da aceleração e depois dela. O que quer dizer que o comprimento da epoch em tempo de relógio é uma quantidade derivada, e quando o tempo de slot se moveu, toda epoch no calendário encolheu em silêncio.

Faça a aritmética uma vez na mão, porque o lab faz o seu script fazer isso para sempre depois. Na antiga meta de 400ms: 432,000 slots vezes 0.4 segundos são 172,800 segundos, que são 48 horas. Em 300ms: 432,000 vezes 0.3 são 129,600 segundos, 36 horas. Ninguém redimensionou as epochs, ninguém anunciou uma mudança de calendário, e mesmo assim tudo que é chaveado por fronteiras de epoch, ciclos de staking, cronogramas de validador, agora vira meio dia mais cedo. Uma constante fixa, uma variável medida, e a resposta em tempo de relógio cai de uma multiplicação. O seu medidor vai calcular as horas restantes da epoch atual a partir do tempo de slot que ele acabou de medir, o que quer dizer que o seu relógio de epoch continua correto mesmo se a stage 3 aterrissar e encolher as epochs de novo para 36 vezes cinco sextos, a meta de 250ms sobre a de 300ms, que você agora consegue calcular sozinho.

![Duas colunas multiplicam a contagem fixa de slots por dois tempos de slot, transformando 48 horas em 36 sem nada mais mudar.](assets/v05-comparison.webp)

### A porta de leitura é grátis, tem teto, e é honesta a respeito

Última peça do modelo antes de a gente construir: a porta em si. A URL que este curso imprime é `https://api.mainnet.solana.com`, a forma atual nos próprios docs de cluster da Solana. Tutoriais mais antigos, e são muitos, usam `api.mainnet-beta.solana.com`; esse alias legado ainda responde, então reconheça ele quando vir, mas escreva a forma atual. Daqui em diante, este curso escreve só o nome atual.

O endpoint público é grátis, e ele é honesto sobre o que grátis quer dizer. Os tetos documentados:

| Teto | Limite |
| --- | --- |
| Requisições por IP | 100 por 10 segundos |
| Requisições por IP, método único | 40 por 10 segundos |
| Conexões simultâneas por IP | 40 |
| Dados por IP | 100 MB por 30 segundos |

Os docs então dizem a parte silenciosa em palavras claras: esses endpoints "não são destinados a aplicações de produção." Grátis, aberto, com rate limit, explicitamente uma ferramenta de bancada. Que é precisamente o trade-off que você está aceitando hoje, e eu quero ele nomeado em vez de descoberto. Rode a aritmética de orçamento do jeito do m02-l3: uma execução do medidor que você está prestes a construir gasta três requisições, então o teto por IP toleraria o lab inteiro, o challenge, e trinta re-execuções paranoicas dentro de uma única janela de dez segundos. Um painel com deploy e cinquenta visitantes, cada navegador atualizando um painel ao vivo, estoura 100 requisições por 10 segundos antes de você terminar de ler esta frase. Mesmo endpoint, mesmos tetos, veredictos opostos. Esse desencontro é o problema de abertura da próxima lição, não um rodapé. Enquanto isso a disciplina de bancada: amostre com um delay, nunca martele o getSlot num loop apertado, e trate os 429s como o endpoint te dizendo a verdade sobre para que ele serve.

**Vá mais fundo (os 20%).** a referência canônica para clusters e os endpoints públicos deles, incluindo todo rate limit acima, URLs de devnet e testnet, e links de explorer, é a própria página de clusters da Solana: https://solana.com/docs/references/clusters (verificada ao vivo em 2026-09-02). Salve como bookmark; esta lição deliberadamente ensinou só o caminho de leitura da mainnet, e aquela página é dona do resto.

## Lab: solana-probes-v0, o medidor da blockchain

O artefato que este tier começa a construir é o `solana-probes-v0`, e a primeira peça dele é o `chain-probe.ts`: um script de bancada autônomo em `packages/pulse-fleet` que imprime os sinais vitais da blockchain, o batimento medido dela, e a contagem regressiva da epoch. Deliberadamente v0, deliberadamente um protótipo de bancada. A próxima lição coloca essas leituras em produção no painel com deploy e no edge worker; o trabalho de hoje é acertar a medição na sua própria máquina primeiro, o mesmo ritmo bancada-depois-deploy que a estação vem seguindo desde o M3.

O recuo, dito de forma simples: os passos 1 a 3 são trabalhados, eu mostro o código. Os passos 4 e 5 te passam um contrato e você escreve o código. Se você quer a versão honesta desta lição, não role para a frente para se conferir até a sua versão rodar.

![Três superfícies com deploy ficam acima de um script de bancada solitário, com uma seta prometendo que as leituras sobem na próxima lição.](assets/v06-diagram.webp)

**1. Faça o scaffold do arquivo.** No repo da estação, crie `packages/pulse-fleet/chain-probe.ts` ao lado do `first-read.ts` e dos outros scripts de bancada, e rode tudo neste lab a partir daquele diretório, pelo motivo de módulo-e-tsx que a abertura nomeou. Você já instalou `@solana/kit@^8` na raiz na abertura; a única outra ferramenta é o `tsx`, que está nas dev dependencies do pulse-fleet desde que os scripts de bancada mudaram para dentro do workspace (se você por algum motivo está numa pasta nova fora da estação: `npm i -D tsx`, e dê ao package.json dele `"type": "module"`). Comece com os sinais vitais que você já sabe ler:

```typescript
import { createSolanaRpc } from "@solana/kit";

const SLOTS_PER_EPOCH = 432_000n;
const TARGET_SLOT_MS = 300;

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

const health = await rpc.getHealth().send();
const slot = await rpc.getSlot().send();
console.log(`health: ${health} | slot: ${slot}`);
```

As duas constantes no topo são as duas âncoras da lição: o comprimento fixo da epoch como um `bigint` (ele vai dividir o número de slot em `bigint`, e aritmética misturando bigint e number é um erro de compilação do TypeScript, que é o sistema de tipos te fazendo um favor), e a meta como um number simples, porque a matemática de milissegundos fica confortavelmente na faixa de `number`.

**2. Tire uma amostra ingênua.** Antes de pegar o instrumento adequado, meça o batimento do jeito que você mediria qualquer coisa: duas leituras e um relógio. Este é o loop de amostragem trabalhado, e ele também respeita o teto de requisições, duas chamadas com dez segundos de intervalo, não um loop quente:

```typescript
// packages/pulse-fleet/naive.ts - a throwaway, not part of the gauge
import { createSolanaRpc } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");

const before = await rpc.getSlot().send();
await new Promise((r) => setTimeout(r, 10_000));
const after = await rpc.getSlot().send();

const slotMs = 10_000 / Number(after - before);
console.log(`${after - before} slots in 10s -> ~${slotMs.toFixed(0)}ms per slot`);
```

A minha execução: `34 slots in 10s -> ~294ms per slot`. Perto da meta, e barulhento, porque dez segundos é uma janela minúscula e a latência das suas próprias requisições borra as duas pontas dela. Isso prova o conceito e mostra a fraqueza num único arquivo descartável. Para fazer melhor você amostraria por minutos, e acontece que você não precisa, porque o nó vem amostrando por você.

**3. Busque as amostras de verdade.** O `getRecentPerformanceSamples` devolve as janelas de performance registradas pelo próprio nó, cada uma com 60 segundos de tempo de blockchain e o número de slots que de fato aconteceram nela. Vinte amostras são vinte minutos de histórico medido numa única requisição, sem loop, sem ansiedade de teto de requisições, e sem borrão de latência de requisição, porque os tempos dentro das amostras são os registros do nó, não as suas idas e voltas. Acrescente ao `chain-probe.ts`:

```typescript
const samples = await rpc.getRecentPerformanceSamples(20).send();
console.log(samples[0]);
```

Rode `npx tsx chain-probe.ts` uma vez para ver a forma de uma amostra. Os campos de que você precisa: `numSlots` (um `bigint`, slots produzidos na janela) e `samplePeriodSecs` (um `number`, o comprimento da janela). Apague a linha `console.log(samples[0])` depois de ter olhado; o medidor imprime conclusões, não matéria-prima.

**4. Escreva a agregação e a linha do medidor (você).** O contrato que o seu código precisa satisfazer:

- Some `numSlots` e `samplePeriodSecs` em todas as 20 amostras, depois calcule a média de milissegundos por slot como segundos totais sobre slots totais, vezes 1000. Some primeiro, depois divida uma vez: tirar a média das médias por amostra pesaria uma janela curta igual a uma longa.
- `numSlots` é um `bigint`; converta com `Number()` na divisão. As contagens de slot por janela são algumas centenas, nem perto de perda de precisão, e diga isso num comentário para o você-do-futuro não surtar.
- Imprima uma linha de medidor: a média medida com uma casa decimal, a meta de 300ms, a diferença percentual com sinal, e a contagem de amostras. A minha diz: `slot time: 313.6ms measured vs 300ms target (+4.5%) over 20 samples`.

**5. Escreva o relógio de epoch (você).** Segundo contrato, e ele é a aritmética da seção de teoria tornada executável:

- Epoch atual: o número do slot dividido por `SLOTS_PER_EPOCH`, em aritmética de `bigint`, que faz o floor de graça.
- Slots restantes: o comprimento da epoch menos a posição do slot dentro da epoch, via o operador `%`, que o `bigint` também suporta.
- Horas restantes: slots restantes vezes os seus milissegundos medidos, dividido por 3,600,000. Use o valor medido, não a meta; o relógio deve dizer a verdade que o seu medidor acabou de estabelecer.
- Imprima isso como uma linha com o número da epoch, slots restantes, e horas com uma casa decimal.

Antes de rodar o script terminado, confira a forma dele contra o mapa abaixo. Não o código, a estrutura: quatro estágios, três linhas impressas, e uma regra estrita sobre qual número alimenta qual. Se a sua versão calcular o relógio de epoch a partir da meta de 300ms em vez da média medida, ele compila, roda, e silenciosamente conta uma verdade pior.

![Quatro estágios do script do medidor se empilham na vertical, com o tempo de slot medido alimentando o relógio de epoch enquanto a constante da meta fica barrada dele.](assets/v07-annotated-code.webp)

**6. Rode ele.** Saída completa da minha execução na hora de escrever, 2026-09-02:

```
health: ok | slot: 443692920
slot time: 313.6ms measured vs 300ms target (+4.5%) over 20 samples
epoch 1027: 403080 slots / ~35.1h remaining
```

Os seus números de slot e de epoch vão estar mais altos, a sua média medida deve cair aproximadamente na faixa de 300-330ms na data da pesquisa, e a linha da diferença deve mostrar uma porcentagem positiva pequena. Qualquer coisa muito fora dessa faixa, confira a ordem some-depois-divida primeiro; é onde a maioria das versões deste script erra.

**7. Prove que é um relógio.** O checkpoint que separa um medidor de um print de sorte: rode ele de novo alguns minutos depois. O número do slot deve ter avançado aproximadamente o seu tempo decorrido dividido pelo seu tempo de slot medido. Cinco minutos são 300,000ms, que a ~314ms por slot dão cerca de 950 slots. Se as suas duas execuções cercarem essa aritmética, o seu script não está só lendo um contador, ele mediu a taxa do contador, e você agora conhece a velocidade do relógio de uma blockchain do mesmo jeito que você conhecia a latência da sua API no M1: porque você mediu ela você mesmo.

![Uma planilha de seis linhas: duas leituras de slot mais os minutos decorridos divididos pelo tempo de slot medido preveem o avanço, e a correspondência é a prova.](assets/v08-table.webp)

Mais uma coisa antes de você chamar de pronto: as três linhas de saída deste script são a interface dele, então mantenha elas limpas, rotuladas, e um-fato-por-linha. Seja preciso sobre o que é promovido, porém, para que a próxima lição não possa te decepcionar: o m08-l2 leva as leituras da classe sinais vitais (o slot, mais as leituras de saldo que ela apresenta) para as superfícies com deploy, enquanto o medidor de 20 amostras e o relógio de epoch ficam instrumentos de bancada de propósito, famintos demais por requisição para um painel por visitante vivendo sob os tetos públicos. O medidor continua pagando aluguel bem aqui, toda vez que você re-executa ele para conferir o ritmo da blockchain contra uma afirmação, que é uma coisa que você agora vai fazer por anos. v0 é um protótipo, não uma desculpa.

## Challenge

Dois degraus, sem orientação. Primeiro, estenda o medidor: reporte o tempo de slot mínimo e máximo ao longo da janela de 20 amostras ao lado da média, e sinalize qualquer amostra individual que tenha rodado mais devagar que o dobro da meta. Esse é o pensamento de latency-stats do M1, dispersão e outliers, apontado para dados de blockchain; a minha janela hoje foi de 309.3ms a 326.1ms sem nada sinalizado, e uma amostra sinalizada num dia tranquilo vale desconfiança (comece pela sua própria aritmética antes de culpar a blockchain).

Segundo, o challenge graduado `epoch-clock` na plataforma te passa uma função `epochClock(slot, slotMs)` com três bugs plantados: ela arredonda a epoch em vez de fazer floor, reporta slots decorridos em vez de restantes, e atrapalha a conversão de milissegundos para horas. Os três são bugs que o seu código do lab acabou de evitar; o conserto é transferir o que você fez no passo 5 para o rascunho quebrado de outra pessoa. A âncora de aceitação vale ser internalizada antes de você começar: no slot 441,936,000 com slots de 300ms ela precisa reportar a epoch 1023 com uma epoch inteira de 432,000 slots e 36.0 horas restando, porque 441,936,000 dividido por 432,000 é exatamente 1023 e um slot de fronteira pertence à epoch que ele abre. Se você consegue explicar isso, o bug do floor já está resolvido na sua cabeça. (E sim, este é o slot do gate da seção de teoria de propósito: o slot FICA na epoch 1023, enquanto a feature que ele ativou ligou na epoch 1024, o atraso de uma epoch da linha do tempo. O seu `epochClock` responde onde um slot está, não quando uma feature entra em vigor.)

## Checkpoint

Você já consegue fazer quatro coisas concretas: explicar uma blockchain para outro desenvolvedor em quatro frases sem enrolação, e nomear onde mora a história mais profunda; ler estado ao vivo da blockchain a partir do TypeScript com o kit contra o RPC público da mainnet; medir o tempo de slot real de uma rede e declarar a diferença para a meta dela com um número; e transformar uma contagem crua de slots numa contagem regressiva de epoch em tempo de relógio a partir de uma única constante fixa. O seu `chain-probe.ts` roda, e a segunda execução dele provou a própria aritmética.

A recuperação de 30 segundos, em voz alta antes de você fechar a aba: qual das quatro ideias explica por que hoje não te custou nada? (RPC é a porta de leitura; leituras nunca viram transações.) E por que as epochs ficaram mais curtas em agosto quando ninguém mudou a epoch? (As epochs são 432,000 slots, fixas; o tempo de slot é a variável, então o comprimento em tempo de relógio se moveu com ele.)

Um pedido enquanto está fresco: esta é a primeira lição do curso em que o sistema sob teste é uma blockchain em vez de algo com deploy feito por você, e eu quero saber se o modelo de quatro ideias se sustentou ou se uma quinta pergunta ficou te incomodando durante o lab. Me diga qual. Se o modelo precisa de uma quinta frase, esse é exatamente o feedback que remodela este tier.

Você consegue medir o batimento da blockchain a partir de um script de bancada. Mas olhe para o que está com deploy: o painel no Vercel, os edge workers, toda superfície com uma URL continua cega para a blockchain, sondando getHealth como se fosse qualquer outro endpoint. Na próxima lição as leituras vão para produção, um painel ao vivo da Solana em toda superfície com deploy, e a disciplina que você construiu para HTTP instável no M2 acaba sendo exatamente o que um RPC público com rate limit exige. Traga o seu medidor.
