# Async que sobrevive ao contato: limites, backoff, cancelamento

## Resumo

A m02-l2 colocou um parser em cada fronteira: a config passa pelo zod (tipada por `z.infer`, refinada, checada com `satisfies`), e a frota até parseou uma resposta real de `getBalance` com lamports como bigint. Dados não conseguem mais entrar malformados de fininho. Mas a frota ainda sonda um alvo por vez, e no momento em que você aponta ela para cinquenta alvos de uma vez, você descobre que a internet tem opiniões sobre o jeito como você pede. Esta lição constrói na mão a disciplina de concorrência da frota: um pool de workers que limita quantas sondas estão em voo, backoff exponencial com jitter para 429s, timeouts de AbortController que transformam sockets presos em resultados tipados, e um relatório agregado em que todo alvo termina em exatamente uma variante de `ProbeResult`. Você vai causar uma parede de 429s de propósito, depois fazer ela sumir, e vai medir as duas coisas.

## Concorrência é um orçamento

Cause o problema primeiro. Nada para instalar hoje; tudo roda com o que você já tem (Node 24 LTS da m01-l2, `tsx` como runner, zod da lição passada). Dois arquivos, três minutos.

Primeiro, um alvo que você tem permissão para martelar. É um servidor local que se comporta como toda API com limite de taxa que você vai encontrar na vida: ele serve um número limitado de requisições por janela, depois responde 429 até a janela virar. Salve como `src/limited-server.ts`:

```ts
// A local target that behaves like every rate-limited API you will ever meet.
// CAP requests per WINDOW_MS, then 429s until the window rolls over.
import { createServer } from "node:http";

const CAP = Number(process.env.CAP ?? 25);
const WINDOW_MS = Number(process.env.WINDOW_MS ?? 1000);
const LATENCY_MS = Number(process.env.LATENCY_MS ?? 250);

let windowStart = Date.now();
let seen = 0;

const server = createServer((req, res) => {
  const now = Date.now();
  if (now - windowStart >= WINDOW_MS) {
    windowStart = now;
    seen = 0;
  }
  seen += 1;
  if (seen > CAP) {
    res.writeHead(429, { "content-type": "application/json", "retry-after": "1" });
    res.end(JSON.stringify({ error: "too many requests" }));
    return;
  }
  setTimeout(() => {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify({ ok: true, path: req.url }));
  }, LATENCY_MS);
});

server.listen(8787, () => {
  console.log(`limited target on http://localhost:8787 (cap ${CAP}/${WINDOW_MS}ms, latency ${LATENCY_MS}ms)`);
});
```

Suba ele num terminal (`npx tsx src/limited-server.ts`) e deixe rodando. Agora a frota ingênua, `src/burst.ts`:

```ts
// The naive fleet: fifty probes, one instant.
const targets = Array.from({ length: 50 }, (_, i) => `http://localhost:8787/t/${i}`);

const statuses = await Promise.all(
  targets.map(async (url) => {
    const res = await fetch(url);
    return res.status;
  }),
);

const walls = statuses.filter((s) => s === 429).length;
console.log(`429s: ${walls} / ${targets.length}`);
```

Rode:

```bash
npx tsx src/burst.ts
```

Na minha máquina:

```text
429s: 25 / 50
```

Metade da frota foi recusada. Olhe o que aconteceu do lado do alvo: cinquenta requisições chegaram no mesmo instante. O servidor admite 25 por segundo, então as 25 primeiras passaram e as outras 25 bateram na parede, tudo dentro de um tick do event loop. A frota que existe para medir disponibilidade acabou de se tornar o problema de disponibilidade. Seu "monitoramento" chegou com a forma exata de um ataque, e o servidor tratou ele como um.

Aqui está a frase que esta lição inteira desempacota: concorrência é um orçamento que você gasta, não uma velocidade que você ganha. A versão ingênua gastou o orçamento inteiro num instante só. Hoje você aprende a distribuir ele aos poucos.

### Cinco minutos sobre o modelo de promise

Tempo marcado, um diagrama, e seguimos. Se você veio de uma linguagem só síncrona (Python sem asyncio, PHP, Java pura e simples), este é o modelo mental sobre o qual tudo abaixo se apoia. Se promises já são confortáveis para você, passe o olho até o diagrama e siga em frente.

Uma promise é um valor que representa um resultado que ainda não existe. Não o resultado: a ficha de retirada dele. Ela está em exatamente um de três estados: pending (trabalho em voo), fulfilled (aqui está seu valor) ou rejected (aqui está seu erro). Ela se estabelece uma vez, de um jeito só, e nunca mais muda.

`await` é onde a cabeça síncrona se queima, então diga com precisão: `await` suspende esta função até a promise se estabelecer. Ele não bloqueia o programa. A função estaciona, o event loop segue rodando todo o resto (timers, outros fetches, o servidor que você acabou de escrever), e quando a promise se estabelece, a função retoma daquela linha exata com o valor na mão. Uma thread, muitas funções suspensas, e I/O que se sobrepõe porque ninguém fica parado esperando um socket.

![Uma promise passa de pending para fulfilled ou rejected, enquanto o await pausa só a função que está esperando e o event loop segue rodando.](assets/v01-diagram.webp)

Essa é a recapitulação inteira. Se alguma coisa aí pareceu nova em vez de enferrujada, esta lição ainda vai estar aqui amanhã: a caixa da honestidade da m01-l1 apontou para a trilha Learn Core Scripting do MDN (developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting) exatamente por esse motivo, e as lições de async dela são a rota respeitável mais rápida para a alfabetização em promises. Faça aquelas, volte, e tudo abaixo vai ler com metade do esforço.

### Promise.all não começa nada

Agora a rajada da abertura, explicada em uma linha: quando `Promise.all` roda, toda sonda já começou.

As pessoas tratam `Promise.all` como um escalonador, algum despachante esperto que vai soltar requisições num ritmo razoável. Não é. É um join. O `targets.map(...)` criou cinquenta promises, o que significa cinquenta chamadas `fetch` já disparadas, no mesmo instante síncrono, antes de `Promise.all` sequer receber o array. Tudo o que o join faz é esperar por todas e te entregar os resultados em ordem. A debandada aconteceu no `.map`. `Promise.all` só assistiu.

Mais uma propriedade, já que estamos sendo precisos, porque ela decide a forma de agregação da frota. `Promise.all` é tudo ou nada: no momento em que qualquer promise rejeita, o join inteiro rejeita com aquele primeiro erro, e os outros quarenta e nove resultados, incluindo os que já tinham dado certo, simplesmente somem. Para uma frota cujo trabalho inteiro é "um resultado para cada alvo", isso é exatamente o contrário; uma consulta DNS instável não deveria vaporizar quarenta e nove boas medições. A resposta da plataforma é `Promise.allSettled`, que espera por tudo e te entrega um objeto wrapper por promise, `{ status: "fulfilled", value }` ou `{ status: "rejected", reason }`, onde `reason` é tipado como um unknown que você ainda tem que interrogar. A nossa é melhor para esta base de código, e você já construiu ela: sondas que nunca rejeitam, porque todo desfecho cai na união `ProbeResult` com seu próprio braço nomeado e seu próprio payload tipado. Mesmo espírito do allSettled, sem nenhum `reason` sem tipo para garimpar. Saiba que `allSettled` existe para o dia em que você estiver agregando promises que não controla; dentro da frota, a união é a agregação.

![Cinquenta requisições simultâneas sobrecarregam um teto de taxa e metade quica, enquanto as mesmas cinquenta em ondas de cinco passam todas.](assets/v02-comparison.webp)

### O pool: N workers, uma fila

A correção é constrangedoramente pequena, e construir ela na mão é justamente o ponto. Bibliotecas como `p-limit` existem e são boas; depois de hoje você vai saber exatamente o que elas fazem, que é umas doze linhas:

```ts
export async function probeAll(targets: string[], config: FleetConfig): Promise<ProbeReport[]> {
  const reports = new Array<ProbeReport>(targets.length);
  let next = 0;

  async function worker(): Promise<void> {
    while (next < targets.length) {
      const i = next;
      next += 1;
      const url = targets[i]!; // i < length, but noUncheckedIndexedAccess can't see it
      reports[i] = await probeWithRetry(url, config);
    }
  }

  const size = Math.min(config.concurrency, targets.length);
  await Promise.all(Array.from({ length: size }, worker));
  return reports;
}
```

Leia como um canteiro de obras: uma fila compartilhada de trabalho (`next` é só um índice dentro dos alvos), e `size` workers, cada um rodando um loop de "pegue o próximo índice, faça a sonda, guarde o resultado naquele índice, repita". Cada worker é uma função async, então enquanto a sonda dele está suspensa num `await`, as sondas dos outros workers também estão em voo. No máximo `size` sondas existem em qualquer instante. Os mesmos cinquenta alvos, o mesmo trabalho total, mas a taxa de rajada agora é limitada por um número que você escolheu.

Repare que `Promise.all` voltou, e agora está sendo usado para o que ele é: um join sobre exatamente `size` promises de worker, não cinquenta fetches sem limite. E repare que os workers nunca lançam. `probeWithRetry` (construímos ele a seguir) retorna um resultado tipado para todo desfecho, então uma rejeição nunca consegue derrubar o join. Essa é a forma de agregação de que a frota precisa: todo alvo termina em exatamente um `ProbeResult`, falhas incluídas.

Já que estamos em rejeições, um footgun específico do Node merece seu próprio parágrafo, porque ele não falha com educação. Uma promise que ninguém aguarda é chamada de fire-and-forget, e quando ela rejeita, não existe catch nenhum na cadeia dela. A resposta default do Node a uma rejeição não tratada é imprimir o erro e matar o processo. Não a sonda. O processo. Uma frota de monitoramento que morre porque a sonda 37 topou com um soluço de DNS que ninguém estava escutando é um relatório de incidente genuinamente vergonhoso, e eu já escrevi uma versão mais branda dele: um scraper antigo meu rodou bem por dois dias, aí um único retry sem await rejeitou às 3 da manhã e levou o loop inteiro junto. A estrutura de pool que você acabou de ler é a cura tanto quanto o medidor: toda promise de sonda é criada dentro de um worker, todo worker é aguardado pelo join, então toda rejeição tem onde cair. Se você algum dia se pegar digitando `void somePromise()` ou chamando uma função async sem dar await nem coletar ela, pare e pergunte quem é o dono daquela promise quando ela rejeita. Nesta frota a resposta é sempre: o agregado.

Duas notas honestas sobre esse trecho. O contador `next` é seguro sem locks porque o JavaScript é single-threaded: as duas linhas que leem e incrementam ele rodam de forma síncrona, e nenhum outro worker consegue se intercalar entre elas. Esse raciocínio é um presente do modelo de event loop, aproveite, ele não viaja para o Rust. E o `!` em `targets[i]` somos nós passando por cima do `noUncheckedIndexedAccess` do nosso tsconfig estrito: a flag não consegue provar `i < targets.length` através das duas instruções, nós conseguimos, e um comentário de uma linha carrega a prova.

O botão importa mais que o mecanismo. Quanto deve ser `concurrency`? Aqui está o reenquadramento que separa quem já foi limitado por taxa de quem está prestes a ser: o pool não é dimensionado pela sua máquina. O Node segura milhares de sockets numa boa. A restrição é o orçamento do alvo, o teto publicado de quem quer que você esteja sondando. Um pool de 5 contra o nosso teto local de 25/seg, com cada requisição levando 250ms, produz no máximo 20 requisições por segundo: abaixo da parede por projeto, não por sorte. Sua CPU nunca entrou na conta.

![Cinco loops de worker idênticos puxam índices de uma fila compartilhada e alimentam um único join que produz um resultado por alvo.](assets/v03-flowchart.webp)

### Backoff, e por que jitter não é opcional

O pool limita sua taxa de rajada, mas os tetos são atingidos mesmo assim: outro processo divide seu IP, a janela fica a cavalo das suas ondas, alguém abaixa o teto numa terça-feira. Quando um 429 chega, a resposta educada é esperar e tentar de novo, e o cronograma dessas esperas é onde a engenharia acontece.

O cronograma canônico é exponencial: espere um delay base, depois dobre ele a cada falha seguinte, com um teto para que ele não cresça para sempre. A tentativa 0 espera `baseMs`, a tentativa 1 espera `2 * baseMs`, a tentativa n espera `min(capMs, baseMs * 2^n)`. Com uma base de 500ms e um teto de 5000ms, o cronograma roda 500, 1000, 2000, 4000, 5000. A duplicação dá ao alvo espaço para respirar; o teto impede que uma queda longa produza esperas de uma hora. Determinístico, dez linhas, você mesmo vai escrever ele no challenge:

```ts
export function backoffDelay(attempt: number, baseMs: number, capMs: number): number {
  return Math.min(capMs, baseMs * 2 ** attempt);
}
```

Agora derive a peça que falta em vez de decorar ela. Imagine o desastre de teto baixo: cinco workers disparam, os cinco tomam 429 na mesma janela, porque o mesmo teto recusou todos de uma vez. Os cinco calculam o mesmo delay da tentativa 0: 500ms. Os cinco dormem 500ms. Os cinco acordam no mesmo instante e disparam de novo, uma debandada sincronizada de cinco, contra o mesmo teto que acabou de recusar uma debandada de cinco. Falham juntos, dormem 1000ms juntos, debandam de novo. Falhas sincronizadas tentam de novo sincronizadas, e a manada re-dispara exatamente o limite que a criou, para sempre. O cronograma é perfeito e a frota nunca escoa.

A correção é ruído. Jitter quer dizer que cada worker aleatoriza sua espera em volta do delay programado, então a manada se borra pela janela em vez de chegar como uma coisa só. Usamos o sabor comum "equal jitter" no ponto de chamada: guarde metade do delay, aleatorize a outra metade.

```ts
const delay = backoffDelay(attempt, baseMs, capMs);
const jittered = delay / 2 + Math.random() * (delay / 2);
await sleep(jittered);
```

Repare que o jitter não deixou nada mais rápido. Espalhada em volta do delay base, a espera média é mais ou menos a de antes. Jitter não é uma ferramenta de latência, é uma ferramenta de dessincronização: ele existe para impedir que seus próprios clientes coordenem um ataque acidental contra a coisa que eles estão tentando de novo. Mais para frente, na execução do lab com teto baixo, você vai ver o borrão no seu próprio log: um aglomerado de 429s cai junto, e os retries voltam espalhados (os meus caíram em 437, 282, 382ms) em vez de num bloco só. Seus dígitos exatos vão ser diferentes, isso é o `Math.random()` fazendo o único trabalho dele; a forma, nenhuma espera batendo com outra, é o que você procura.

![Retries sem jitter chegam em aglomerados simultâneos que se repetem e falham todos, enquanto retries com jitter se espalham pela linha do tempo e dão certo.](assets/v04-timeline.webp)

Mais uma regra antes de ligarmos isso, porque é aqui que o trabalho de tipos da l1 se paga: política de retry é por variante. Um 429 é o servidor dizendo "agora não", então ele merece backoff. Um 404 é o servidor dizendo "nunca", e tentar de novo é um bug que custa cinco delays para descobrir. Um timeout é ambíguo e, para uma frota de status, a jogada honesta é registrar ele e deixar a próxima rodada agendada decidir. A união discriminada é o que torna essa política expressável como código em vez de vibe: faça o match na variante, tente de novo exatamente uma delas.

### AbortController: um timeout é um cancelamento

O último modo de falha é o pior de todos: o alvo que nem responde nem recusa. Um socket preso mantém seu slot do pool refém; com cinco workers, cinco sockets presos são uma frota morta. Timeouts são o jeito de um slot recuperar a vida, e no `fetch` timeout se escreve AbortController.

A fiação são três jogadas: criar um controller, entregar o signal dele para o `fetch`, e combinar que `abort()` seja chamado quando o orçamento expirar. O abort faz o `fetch` em voo rejeitar, e a gente captura essa rejeição e transforma ela num desfecho tipado de primeira classe:

```ts
async function probeOnce(url: string, timeoutMs: number): Promise<ProbeResult> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  const started = performance.now();
  try {
    const res = await fetch(url, { signal: controller.signal });
    if (res.ok) {
      return { kind: "ok", latencyMs: Math.round(performance.now() - started) };
    }
    return { kind: "http-error", status: res.status };
  } catch {
    if (controller.signal.aborted) {
      return { kind: "timeout", budgetMs: timeoutMs };
    }
    return { kind: "dns-error", host: new URL(url).hostname };
  } finally {
    clearTimeout(timer);
  }
}
```

Percorra as saídas, porque cada uma delas é uma lição. O caminho feliz retorna `ok` com uma latência medida. Uma resposta não-2xx retorna `http-error` com o status (o loop de retry lá em cima decide se 429 merece outra tentativa). Se o bloco catch encontrar `controller.signal.aborted` verdadeiro, a rejeição foi o nosso próprio timer disparando, e ela vira a variante `timeout` com o orçamento que ela estourou, não uma rejeição não tratada chacoalhando pilha acima. Qualquer outra coisa no catch é a própria rede falhando (DNS, conexão recusada), e isso cai no braço `dns-error` que o treino da l1 te fez adicionar. Se esse nome te incomoda agora que falhas de conexão moram lá também, renomeie ele para `network-error` e deixe o compilador te levar a cada switch que precisa ser atualizado. Essa tarefa custar minutos em vez de uma tarde é exatamente o que você comprou na m02-l1.

E o `finally`: `clearTimeout` roda em toda saída. Pule ele e a sonda funciona mesmo assim, que é o que torna esse footgun tão bem escondido. O timer sobrevive à requisição terminada, dispara depois, e aborta um controller que ninguém está usando. Inofensivo hoje; aí alguém reusa o controller, ou o processo deveria ter saído e não saiu porque um timer estava pendente. Limpeza faz parte do padrão, não é um floreio.

![Quatro linhas anotadas mostrando a criação do controller, a fiação do timer, a entrega do signal, e o clearTimeout no finally que é fácil de esquecer.](assets/v05-annotated-code.webp)

Agora a parte honesta, e o trade-off real desta lição. Cancelamento é cooperativo e local. Abortar o fetch libera seu slot, estabelece sua promise, e dá ao seu relatório uma linha tipada limpa. Ele não atravessa o fio para des-enviar nada: a requisição que você já mandou pode terminar no servidor mesmo assim. Eu medi isso no lab que você está prestes a rodar: com o timeout espremido para 100ms, as cinquenta sondas voltaram `timeout`, e o servidor ainda queimou orçamento servindo requisições que ninguém estava esperando, o bastante para 10 retries serem disparados pelo caminho. Toda disciplina desta lição troca latência por civilidade. O pool termina em dez ondas em vez de uma. O backoff faz alvos que falham demorarem mais para reportar. Um timeout converte um alvo lento-mas-vivo numa falha declarada num limiar que você escolheu. Não existe tamanho de pool nem timeout correto, existe o orçamento do alvo, seu prazo, e um botão para girar; o pecado é não saber contra qual limite você está trocando.

![Os quatro desfechos possíveis de uma sonda, sucesso, erro HTTP, timeout e falha de rede, cada um desaguando numa linha de resultado tipada, com só o caminho do 429 voltando pelo backoff.](assets/v06-flowchart.webp)

### A parede sob a qual esta frota realmente vive

Tudo até aqui usou um teto de brinquedo para você poder medir sem incomodar ninguém. Agora o número de verdade, porque esta frota está mirando a infraestrutura da Solana do M8 em diante. Os endpoints RPC públicos da Solana publicam seus limites na referência oficial de clusters (solana.com/docs/references/clusters, verificado em 2026-09-02): 100 requisições por 10 segundos por IP, e 40 por 10 segundos para qualquer método RPC isolado, com 40 conexões concorrentes por IP e 100 MB de dados por 30 segundos. A mesma página diz sem rodeios que esses endpoints "não são destinados a aplicações de produção". Esses quatro números são um orçamento de alvo, exatamente como `CAP=25` era, e o único fetch de `getBalance` da lição passada já vivia sob eles sem saber.

Faça a conta do jeito que você dimensionaria qualquer pool. Uma frota de status sondando via `getBalance` queima primeiro o orçamento por método: 40 por 10 segundos. Cinquenta alvos através de um pool de 5 com nossas latências de 250ms empurrariam 20 requisições por segundo, cinco vezes acima daquele teto de método. A mesma frota com `concurrency: 3` e uma pausa modesta por rodada fica abaixo dele. O ponto não são esses dígitos específicos; o ponto é que o botão tem uma entrada correta, e ela é o orçamento publicado do alvo, nunca o apetite da sua máquina.

Aqui está o hábito que este curso não para de treinar, e vale nomear ele como hábito: alvos documentados, realidade medida. A Solana mira slots de 300ms e uma sonda de 20 amostras em 2026-09-01 mediu 316ms em média. A documentação diz 100 requisições por 10 segundos; seu log diz onde os 429s realmente começaram. Sistemas publicam intenções, e engenheiros verificam elas com os próprios instrumentos. Hoje seu instrumento é um contador numa frota de cinquenta linhas. No M8 você vai apontar o mesmo hábito para a chain em si e construir um medidor que mede tempos de slot ao vivo.

Uma fronteira, dita sem rodeios para ninguém aplicar demais os padrões de hoje: tudo nesta lição é boas maneiras de caminho de leitura, a etiqueta de GETs e leituras JSON-RPC que você pode reenviar às cegas. Tentar de novo uma transação é outro esporte com outras apostas (o primeiro envio de fato aterrissou?), e essa profundidade, aterrissagem de transação e tudo que é vizinho de carteira, é território do curso de maestria do lado cliente; aquele curso está em produção enquanto eu escrevo, e até ele ser entregue o ponteiro honesto é o próprio tópico. Sondas são idempotentes; pagamentos não são; não porte este loop de retry para dinheiro.

**Vá mais fundo (os 20%).** tudo aqui foi a camada do dia a dia. As entranhas do event loop, microtasks versus macrotasks, iteradores async e `for await`, combinadores de promise além do `all`: salve como bookmark a unidade Asynchronous JavaScript do MDN (developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Async_JS, gratuita, verificada ao vivo em 2026-09-02) e vá mais fundo quando um bug te mandar. Esta lição deliberadamente não re-ensina o que aquelas páginas dominam.

## Lab: a frota, concorrente e educada

O recuo que este módulo vem aplicando continua: o pool e a rajada foram totalmente trabalhados acima; o loop de backoff no passo 3 é uma completion, esqueleto dado, dois órgãos são seus; a fiação do abort no passo 4 é evocação, escrita de memória contra uma listagem que você já leu; o challenge depois disso é só seu.

1. **Reconstrua o schema de config para a frota concorrente.** Os botões da frota pertencem ao `pulse.config.json`, atrás do parser da lição passada, não hardcoded. O lab sonda uma lista plana de URLs locais sob um orçamento compartilhado, então remodele o schema da l2 em `src/config.ts`: os alvos viram URLs simples, `timeoutMs` move para o nível superior, e dois campos novos, `concurrency` e `retry`, carregam os botões. Mesma disciplina de fronteira, forma nova, ainda `strictObject` porque uma config é uma forma que é sua e a regra da l2 continua de pé: uma chave desconhecida nela é um erro, não um dar de ombros. E `z.infer` atualiza `FleetConfig` de graça:

   ```ts
   import { z } from "zod";

   const retrySchema = z.strictObject({
     maxRetries: z.number().int().min(0),
     baseMs: z.number().int().positive(),
     capMs: z.number().int().positive(),
   });

   export const configSchema = z.strictObject({
     targets: z.array(z.url()).min(1),
     timeoutMs: z.number().int().positive(),
     concurrency: z.number().int().min(1).max(50),
     retry: retrySchema,
   });

   export type FleetConfig = z.infer<typeof configSchema>;
   ```

   Três consequências da remodelagem, resolvidas agora para que `npx tsc --noEmit` e a trava de CI da sua m01-l3 continuem verdes em vez de apodrecerem caladas. Primeiro, mantenha `parseOrExit` em `src/config.ts` quando você trocar o schema; o trecho acima mostra só o que muda, e tanto o passo 5 deste lab quanto seus scripts da l2 ainda importam o helper. Segundo, `src/check-config.ts` imprime `config.fleetName` e `t.intervalSecs`, campos que o novo schema não tem mais, então ou você apara ele para a forma nova (uma linha: contagem de alvos, tamanho do pool, timeout) ou deleta ele junto com `pulse.config.broken.json` e, se o schema dele brigou com a remodelagem, o `src/check-status.ts` do seu challenge da l2; aqueles eram os adereços de ensino que a l2 usava, e a disciplina que eles ensinavam agora mora dentro da própria frota. Terceiro, repare no que saiu caladinho da config e por quê: `intervalSecs` por alvo (e o refine construído em cima dele) não tem em que se prender numa frota que sonda todo alvo numa rodada compartilhada; a cadência agora pertence ao cron que dispara a rodada, não a alvos individuais, e o `timeoutMs` compartilhado é o orçamento que sobreviveu.

2. **Gere a config do lab.** Cinquenta alvos locais, pool de 5, o cronograma de backoff da seção de teoria. Um gerador descartável ganha de digitar cinquenta URLs na mão:

   ```ts
   // src/make-targets.ts
   import { writeFileSync } from "node:fs";

   const targets = Array.from({ length: 50 }, (_, i) => `http://localhost:8787/t/${i}`);
   const config = {
     targets,
     timeoutMs: 3000,
     concurrency: 5,
     retry: { maxRetries: 5, baseMs: 500, capMs: 5000 },
   };
   writeFileSync("pulse.config.json", JSON.stringify(config, null, 2));
   console.log("wrote pulse.config.json");
   ```

   Rode `npx tsx src/make-targets.ts` uma vez.

3. **Escreva o loop de retry (completion).** Em `src/fleet.ts`, o loop abaixo é dado com dois buracos. Tudo em volta deles está completo; preencha eles a partir da seção de teoria sem rolar de volta, se você conseguir.

   ```ts
   async function probeWithRetry(url: string, config: FleetConfig): Promise<ProbeReport> {
     const { maxRetries, baseMs, capMs } = config.retry;
     let retries = 0;
     for (let attempt = 0; ; attempt++) {
       const result = await probeOnce(url, config.timeoutMs);
       // TODO 1: set `retryable` from the variant. Retry policy is per-variant,
       // and exactly one of the four earns another attempt.
       if (!retryable || attempt >= maxRetries) {
         return { url, result, retries };
       }
       retries += 1;
       // TODO 2: compute `jittered` from `backoffDelay(attempt, baseMs, capMs)`
       // run through the equal-jitter line: keep half, randomize the other half.
       console.log(`  429 from ${url}: attempt ${attempt}, waiting ${Math.round(jittered)}ms`);
       await sleep(jittered);
     }
   }
   ```

   As versões preenchidas, depois de você escrever as suas:

   ```ts
   const retryable = result.kind === "http-error" && result.status === 429;
   ```

   ```ts
   const delay = backoffDelay(attempt, baseMs, capMs);
   const jittered = delay / 2 + Math.random() * (delay / 2);
   ```

   (`ProbeReport` é `{ url: string; result: ProbeResult; retries: number }`: a união da l1 carregando seu alvo e seu custo. `sleep` é aquele de duas linhas, `new Promise((resolve) => setTimeout(resolve, ms))`. Uma nota de fiação antes de o compilador perguntar: declare você mesmo a união `ProbeResult` de quatro variantes e este tipo `ProbeReport` no topo de `src/fleet.ts`. Não há nada para importar ainda, de propósito: a união da l1 mora no `probe.ts` da raiz, que é um script de CLI, não um módulo, então a frota ganha sua própria cópia local hoje. A m02-l4 move a cópia canônica para dentro de `src/classify.ts` e o M3 extrai ela para um pacote; esta cópia local é a duplicação que motiva as duas.)

4. **Ligue o abort (de memória).** `probeOnce` está impresso completo na seção de teoria, então este aqui é evocação, não completion: feche esta página ou role para longe dela e escreva você mesmo a função dentro de `src/fleet.ts` a partir das três jogadas, a criação do controller, o `signal` nas opções do fetch, e o `clearTimeout` no `finally`, mais o mapeamento das saídas (`signal.aborted` no catch vira a variante `timeout`, todo o resto no catch vira `dns-error`). Depois role de volta e faça o diff da sua contra a listagem. A linha que você mais provavelmente deixou cair é o `clearTimeout`, e a seção de teoria diz por que aquela se esconde.

5. **Monte e rode.** `probeAll` da seção de teoria mais um pequeno rodapé de CLI: parseie a config com o `parseOrExit` que a l2 deixou pronto, chame `probeAll`, depois faça o fold dos relatórios em contagens. Um objeto chaveado pelo kind da variante é a versão rápida mostrada abaixo; reescrever o fold como o switch exaustivo da l1 com `assertNever` é a versão mais sólida, e vale os cinco minutos.

   ```ts
   const path = process.argv[2];
   if (!path) {
     console.error("usage: npx tsx src/fleet.ts pulse.config.json");
     process.exit(1);
   }

   const config = parseOrExit(configSchema, JSON.parse(readFileSync(path, "utf8")));

   const startedAt = performance.now();
   const reports = await probeAll(config.targets, config);
   const elapsed = Math.round(performance.now() - startedAt);

   const counts = { ok: 0, timeout: 0, "http-error": 0, "dns-error": 0 };
   let retriesTotal = 0;
   let finished429 = 0;
   for (const { result, retries } of reports) {
     counts[result.kind] += 1;
     retriesTotal += retries;
     if (result.kind === "http-error" && result.status === 429) finished429 += 1;
   }

   console.log(`${reports.length} targets in ${elapsed}ms with pool of ${config.concurrency}`);
   console.log(`ok: ${counts.ok}  timeout: ${counts.timeout}  http-error: ${counts["http-error"]}  dns-error: ${counts["dns-error"]}`);
   console.log(`429s in final report: ${finished429}  (retries spent absorbing them: ${retriesTotal})`);
   ```

   Mais uma linha entra no rodapé, e ela é um retorno para casa. A m01-l2 vendeu seu challenge complementar, `latencyStats`, com uma promessa: aquela função exata é entregue na frota. É entregue agora. Cole sua solução corrigida dentro de `src/fleet.ts` (ou escreva ela do zero a partir do mesmo contrato: min, max, mean arredondada para duas casas decimais, p95 por nearest-rank), depois faça o fold das sondas bem-sucedidas por ela abaixo das contagens. O contrato dela recebe uma string separada por vírgulas, então o adaptador é um `join`:

   ```ts
   const okLatencies = reports.flatMap(({ result }) =>
     result.kind === "ok" ? [result.latencyMs] : [],
   );
   if (okLatencies.length > 0) {
     const s = latencyStats(okLatencies.join(","));
     console.log(`latency: min ${s.min}  max ${s.max}  mean ${s.mean}  p95 ${s.p95}  (ms, over ${okLatencies.length} ok probes)`);
   }
   ```

   A função de estatísticas que você construiu na m01-l2, agora em serviço na estação: uma amostra era ruído, e cinquenta por varredura é exatamente o batch que ela foi construída para resumir. A trava de `length` não é educação, e vale saber exatamente do que ela te salva. O starter dela diz "a entrada é garantidamente não vazia", e essa promessa é do chamador para cumprir: rode `latencyStats([].join(","))` e você não recebe erro nenhum, você recebe `{ min: 0, max: 0, mean: 0, p95: 0 }`, porque `"".split(",")` é `[""]` e `Number("")` é `0`. Então uma varredura em que toda sonda falhou imprimiria uma linha confiante de `min 0 max 0 mean 0 p95 0`, o tipo mais perigoso de errado: um monitor reportando latência perfeita para uma frota que não respondeu nada. Pular a linha é o relatório honesto. Funções de fronteira herdam suas pré-condições de quem chama elas, e essa trava é onde essa aí é cumprida.

   Com o servidor da abertura ainda rodando:

   ```bash
   npx tsx src/fleet.ts pulse.config.json
   ```

   Minha execução:

   ```text
   50 targets in 2562ms with pool of 5
   ok: 50  timeout: 0  http-error: 0  dns-error: 0
   429s in final report: 0  (retries spent absorbing them: 0)
   latency: min 251  max 279  mean 254.7  p95 263  (ms, over 50 ok probes)
   ```

   O espalhamento de latência fica um fio acima do piso de 250ms do servidor, com o setup de conexão sendo o que é; seus dígitos vão ser diferentes, a forma não.

   Ponha isso ao lado da execução de rajada da abertura e do `429s: 25 / 50` dela. Os mesmos cinquenta alvos, o mesmo servidor, o mesmo teto. A única coisa que mudou é quem limita o trabalho em voo: ninguém, ou você. Esse lado a lado é o artefato desta lição e a sua trava de verificação: cinquenta alvos, um resultado tipado cada, zero 429s.

6. **Deixe o backoff visível.** Zero retries é uma vitória chata, então abaixe a parede até a disciplina ter que trabalhar. Reinicie o servidor com um quinto do orçamento (`CAP=5 npx tsx src/limited-server.ts`), rode a frota de novo, e leia o log enquanto ela briga:

   ```text
   429 from http://localhost:8787/t/6: attempt 0, waiting 437ms
   429 from http://localhost:8787/t/5: attempt 0, waiting 282ms
   429 from http://localhost:8787/t/8: attempt 0, waiting 382ms
   429 from http://localhost:8787/t/7: attempt 1, waiting 816ms
   429 from http://localhost:8787/t/8: attempt 2, waiting 1920ms
   ```

   Está aí a seção de teoria inteira em cinco linhas de log: as esperas da tentativa 0 se agrupam em volta da base de 500ms, as da tentativa 1 em volta do dobro disso, a tentativa 2 dobra de novo, e nenhuma espera bate com outra porque o jitter borrou a manada. Minha execução terminou todas as cinquenta em 13647ms com 59 retries absorvidos e, de novo, zero 429s no relatório final. Mais lenta, e civilizada: esse é o trade-off que você escolheu quando ajustou os botões.

7. **Dois treinos de falha, trinta segundos cada.** Mate o servidor e rode a frota: cinquenta linhas `dns-error`, na hora, sem crash, porque toda saída é tipada. Reinicie ele, ponha `timeoutMs` em 100 na config (abaixo da latência de 250ms do servidor), rode de novo: cinquenta linhas `timeout` em cerca de 2.3 segundos, com o pool nunca empacando porque todo abort liberou seu slot. Uma frota que reporta suas falhas na mesma forma calma que reporta seus sucessos é a coisa contra a qual os testes da próxima lição são construídos.

![Em três execuções medidas, a rajada ingênua falha metade das suas sondas enquanto as duas execuções com pool não falham nenhuma, pagando em vez disso com tempos de relógio maiores.](assets/v07-chart.webp)

## Challenge: o cronograma de backoff, exatamente

A rep sem orientação. O grader te entrega um cronograma ingênuo, do tipo que renderia a uma frota o banimento dela se você algum dia o entregasse: ele começa a dobrar imediatamente, então a primeira espera é o dobro da base, e ele nunca aplica o teto, então tentativas tardias esperam um tempo absurdo. Conserte os dois. O contrato: a tentativa n (base 0) espera `min(capMs, baseMs * 2^n)` milissegundos, os delays unidos numa string separada por vírgulas, e `retries = 0` retorna a string vazia. O cronograma continua determinístico; não existe `Math.random()` na função corrigida, e o arquivo starter diz por quê: o jitter mora no ponto de chamada para que o cronograma em si possa ser testado dígito a dígito, que é precisamente o que os seis testes fazem. Olho na borda que os testes vigiam: um teto abaixo da base prende todo delay no teto.

Se você quiser a checagem de sanidade de quinze segundos antes de enviar, base 500 e teto 5000 ao longo de cinco retries têm que imprimir `500,1000,2000,4000,5000`, o cronograma exato que sua frota rodou no lab.

## Onde você está

Vitória de trinta segundos, em voz alta: concorrência é um orçamento que você gasta, não uma velocidade que você ganha. E os dois botões que gastam ele: tamanho do pool, e o cronograma de retry. Se você também consegue dizer por que o jitter existe sem a palavra "aleatório" aparecer antes da palavra "debandada", você tem a lição inteira.

A frota agora sonda cinquenta alvos tipada, parseada, limitada e educada: ela respeita um teto publicado, absorve 429s com backoff com jitter, converte sockets presos em timeouts tipados, e termina com um resultado para cada alvo, não importa o que a rede fez. E aqui está a parte desconfortável: ela é completamente não comprovada. Toda fronteira de classificador, todo delay de backoff, todo mapeamento de timeout é uma afirmação que ninguém testou; a evidência até agora é "parecia certo no meu terminal", que é exatamente o padrão que você rejeitaria de qualquer outra pessoa. Próxima lição: vitest. Sondas para o seu próprio código, ligadas ao cron antes de ele publicar outro status.json. Hora de provar.
