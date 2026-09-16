# React como consumidor: o painel

A lição passada dividiu o repo num workspace de verdade: `pulse-core` extraído com um `package.json` honesto, a frota importando ele através da fronteira, a suíte de m02-l4 ainda verde, e uma promessa na saída de que um segundo consumidor estava vindo. Esta é essa lição. A estação vem commitando `status.json` a cada 30 minutos desde o módulo 1, noites e fins de semana, e em todo esse tempo nenhum pixel jamais mostrou isso. Hoje ela ganha um rosto.

E a gente faz o rosto primeiro, teoria depois. A partir da raiz do repo:

```bash
cd packages
pnpm create vite pulse-board --template react-ts
```

(Isso roda o `create-vite`, 9.2.0 no momento em que escrevo isto, em 2026-09-02; a flag `--template react-ts` deixa ele não interativo.) Agora estripe o demo e substitua `packages/pulse-board/src/App.tsx` pela menor coisa que consegue mostrar os seus dados. Troque pelo seu próprio nome de usuário do GitHub:

```tsx
import { useEffect, useState } from "react";

const RAW_URL =
  "https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json";

export default function App() {
  const [raw, setRaw] = useState("loading...");

  useEffect(() => {
    fetch(RAW_URL)
      .then((res) => res.text())
      .then(setRaw)
      .catch((err) => setRaw(String(err)));
  }, []);

  return <pre>{raw}</pre>;
}
```

Instale e rode:

```bash
cd pulse-board
pnpm install
npm run dev
```

(Sobre as grafias misturadas, invocando a nota da casa de m03-l1 uma vez para nunca mais coçar: installs dentro do workspace são trabalho do pnpm, enquanto `npm run` e `pnpm run` leem o mesmo bloco `scripts` e são intercambiáveis; as linhas de script deste curso usam a que o toolchain de verificação repetiu, e `pnpm run dev` aqui se comportaria de forma idêntica.)

Abra a URL de localhost impressa. Aquela parede de JSON no seu navegador não é dado de exemplo, é a sua frota: os alvos que você escolheu, latências que o seu cron mediu numa máquina que não é sua, buscadas cross-origin do seu repo público com zero backend e zero chaves. Semanas de sondagem sem ninguém olhando, numa página, em quinze minutos. Deixe essa aba aberta; a lição inteira é sobre transformar isso de um despejo de `<pre>` num painel que você mostraria para alguém.

## Resumo

As descobertas logo de cara:

- Um painel é uma função pura de um arquivo JSON; o React é só o loop de renderização. Componente = função de props para UI, o estado a única entrada que dispara uma repintura. Esta lição ensina React em nível de consumidor só, de propósito: ela assenta o piso de consumidor em que o trabalho de cliente de dApp de verdade se apoia, e a profundidade fica com o curso de domínio do lado do cliente.
- O seu repo público já é uma API de dados. `raw.githubusercontent.com` manda `access-control-allow-origin: *` incondicionalmente (sondado em 2026-09-02), faz cache por 5 minutos (`cache-control: max-age=300`, Fastly), e serve `.json` como `text/plain`. Os três fatos moldam o código de hoje.
- A primeira jogada do lab desconta a promissória de m02-l4: o escritor da frota tem a fiação refeita para emitir linhas da união `ProbeResult`, então `status.json` finalmente fala o dialeto tipado que o resto da estação fala desde o M2.
- Os bytes buscados cruzaram uma fronteira de rede, então eles passam por um schema zod como toda fronteira desde m02-l2: um arquivo corrompido produz um estado de erro visível, nunca uma página em branco.
- O painel importa `classifyProbe` do `pulse-core`, então a frota e o painel comprovadamente rodam o mesmo código de classificação: a extração de m03-l1 demonstrada, não afirmada.
- A ajuda recua num cronograma: eu dirijo o scaffold e o efeito de polling, você constrói `StatusRow` a partir de uma spec em nível de assinatura, e o indicador de desatualização do challenge é só seu.

## O loop de renderização e o caminho dos dados

Uma cerca antes de qualquer outra coisa, dita sem pedir desculpa: este não é um curso de React. React é um tópico do tamanho de uma carreira, e a casa dele neste catálogo, o curso de domínio do lado do cliente, está em produção enquanto eu escrevo; espere UX de carteira, aterrissagem de transação e profundidade de cliente de dApp de verdade lá. O nosso trabalho é o nível de consumidor de onde esse tipo de trabalho parte: componentes, props, estado, um efeito. Isso acaba sendo o bastante para entregar um painel de verdade, o que te diz alguma coisa sobre onde os 80 por cento realmente moram.

### Um componente é uma função, o estado é a campainha

Tire a mística primeiro. O melhor modelo de um componente React é a coisa que você vem escrevendo o curso inteiro: uma função pura. Ela recebe um objeto de entradas, chamado props, e retorna uma descrição de UI. Mesmas entradas, mesma UI. Sem humor escondido.

```tsx
function Greeting({ name }: { name: string }) {
  return <p>hello, {name}</p>;
}
```

A sintaxe de sinais de maior e menor é JSX, e ela merece exatamente um parágrafo: é açúcar compilado para chamadas de função. `<p>hello, {name}</p>` vira uma chamada que constrói `{ type: "p", props: { children: [...] } }`, um objeto simples descrevendo o que deveria existir. O toolchain do Vite faz a compilação; você nunca configura isso. Essa é a cerimônia inteira que o JSX ganha neste curso.

Então, se componentes são funções puras, o que faz a página mudar alguma vez? Uma coisa: o estado. O `useState` te dá um valor mais um setter, e chamar o setter é a única campainha que o React atende. Setou o estado, o React re-roda a sua função com o valor novo, faz o diff da descrição contra o DOM, aplica o patch da diferença. Os dados fluem numa direção só, sempre: estado entra, renderização sai, pixels por último. O React nunca lê a sua tabela de volta do DOM, e reatribuir alguma variável de nível de módulo ao lado do componente é invisível para ele. Só o setter agenda uma repintura.

![Bytes buscados fluem pelo parse até o estado e daí para pixels com patch aplicado, enquanto setas do DOM ou de variáveis de módulo de volta para o loop estão riscadas.](assets/v01-flowchart.webp)

O que rende o aha que dá nome a esta lição: um painel é uma função pura de um arquivo JSON. `status.json` é o estado do mundo; o painel é `render(state)`. Todo o resto, buscar, fazer polling, cachear, é encanamento para manter aquela única entrada fresca. Segure esse modelo e a maioria dos tutoriais de React desaba em detalhes sobre o encanamento.

Mais uma primitiva, porque "buscar um arquivo a cada minuto" é um efeito colateral, não uma computação pura. O `useEffect` é o contêiner do React para exatamente isso: código que roda depois da renderização, tocando o mundo. Ele recebe uma closure e um array de dependências; array vazio `[]` quer dizer rodar uma vez no mount. Um asterisco de modo de desenvolvimento antes de você contar qualquer coisa no devtools: o template do create-vite envolve o app no StrictMode do React, que em desenvolvimento deliberadamente monta, desmonta e remonta cada componente uma vez para sacudir cleanups faltando, então "uma vez no mount" aparece como duas vezes na aba Network enquanto você desenvolve; builds de produção rodam isso uma vez. Crucialmente, a closure pode retornar uma função de cleanup, chamada no unmount ou no hot-reload; pule ela num interval e todo save de desenvolvimento empilha outro poller, um bug que o lab te faz encontrar de propósito. Essa é a API de hooks inteira que este curso ensina: `useState`, `useEffect`, pronto. Context, reducers, refs, server components, suspense: todos reais, todos adiados pelo nome para o curso de domínio do lado do cliente.

Um footgun preventivo. Digite "react fetch data" numa caixa de busca e vão te dizer que efeitos feitos à mão são coisa de amador e que uma biblioteca de data fetching é o mínimo. Essas bibliotecas são excelentes, e elas resolvem problemas que esta lição não tem: deduplicação entre dezenas de componentes, invalidação de cache, escritas otimistas. O seu requisito é uma URL, um interval, um schema; `useState` mais `useEffect` mais zod é o trabalho inteiro, e saber disso é a habilidade. O caso da biblioteca se faz direito quando mutations e estado de servidor compartilhado chegam, e isso é território de domínio do lado do cliente.

### O caminho dos dados: o seu repo público já é uma API

Agora o encanamento, e é aqui que a decisão de seu-repo-é-público do módulo 1 se paga. O repo da sua estação é público, o que quer dizer que todo arquivo nele é servido em `raw.githubusercontent.com/<user>/<repo>/<branch>/<path>`. Sem token, sem SDK, sem servidor que você roda. A pergunta que um dev que trabalha faz antes de confiar nesse caminho: o que aquele endpoint faz de verdade? Não o que um post de blog diz que ele faz. Então eu sondei ele, e tudo nesta seção é ensinado a partir dos headers observados, datados de 2026-09-02.

![Um cron commita dados de status num repositório público enquanto um navegador lê eles de volta através de um cache de CDN de cinco minutos, uma checagem de schema e o estado do React.](assets/v02-diagram.webp)

Três descobertas importam. Primeiro, CORS. Navegadores bloqueiam fetches cross-origin a menos que o servidor opte por permitir, e o raw.githubusercontent opta por permitir até o fim: `access-control-allow-origin: *`, incondicionalmente. A sonda checou o caso sorrateiro, mandando uma requisição pelada e depois uma marcada com Origin, e os headers voltaram idênticos, então a permissividade não é refletida por origem, ela é só aberta. É por isso que o seu despejo de `<pre>` de quinze minutos funcionou de primeira em vez de morrer com um erro de CORS no console.

Segundo, o cache, que muda o seu modelo mental de "ao vivo". O endpoint retorna `cache-control: max-age=300` com um `etag` forte, servido através da Fastly; a sonda viu um MISS virar um HIT um segundo depois. Cinco minutos de cache de CDN, empilhados num cron de 30 minutos: o seu painel pode ficar atrás da realidade pelo intervalo do cron mais a janela de cache, e nenhum código React muda isso, porque os bytes desatualizados chegam desatualizados. Quando o seu painel "não está atualizando", olhe os headers de resposta no devtools primeiro, não o componente.

![Uma linha do tempo mostra um intervalo de cron de trinta minutos com uma janela de cache de cinco minutos sobrepondo um commit fresco, então quem olha pode ler linhas velhas depois de dados novos já existirem.](assets/v03-timeline.webp)

Terceiro, o content type. O endpoint serve o seu arquivo `.json` como `content-type: text/plain; charset=utf-8`. E mesmo assim `await res.json()` faz o parse dele sem reclamar, porque a spec de fetch da WHATWG faz o parse do corpo que você pediu para ela parsear; o header MIME vai junto sem ser lido. Conveniente, e ligeiramente desonesto. No dia em que você trocar por uma biblioteca HTTP que fareja o content-type antes de fazer o parse, esta resposta exata vira um bug report, então você aprende o fato agora, enquanto é barato.

Uma linha de honestidade para completar o quadro. O GitHub não documenta nenhum rate limit para este endpoint, e eu não vou inventar um; o que você está consumindo são bytes estáticos não autenticados atrás de um CDN com controles de abuso não documentados. E o GitHub não abençoa raw.* como produto de hospedagem de jeito nenhum. Neste curso ele é sempre e somente o endpoint de dados. O painel em si, o HTML e o JS, é entregue no Vercel na próxima lição, que é o caminho sancionado.

Então os bytes chegam: CORS aberto, possivelmente desatualizados, MIME rotulado errado. Você confia neles? Você já sabe a resposta, porque é a mesma resposta de m02-l2: eles cruzaram uma fronteira de rede, então eles são parseados, não afirmados. Um schema zod espelhando a união `ProbeResult` do `pulse-core` fica na fronteira do fetch, e um arquivo corrompido à mão morre ali como um estado de erro visível em vez de lá no fundo de uma renderização como uma página em branco. Parse, don't validate, agora guardando pixels.

Nomeie o trade-off enquanto está fresco: um negócio genuinamente ótimo com data de validade impressa. Fazer polling de um arquivo raw não custa nada (sem backend, sem chaves, sem fatura, o uptime do seu repo) pelo preço que a conta do CDN acabou de te mostrar: até cron-mais-cinco-minutos atrás da realidade, sem push, sem auth, só público. No momento em que você precisar de atualizações em tempo real, linhas privadas, ou um caminho de escrita, você precisa de uma API de verdade; o edge worker do M7 começa essa história. Até lá, um backend aqui seria pura cerimônia.

Tem um trade-off com formato de React se escondendo aqui também. Um loop de renderização de framework compra UI declarativa: descreva como o painel fica para um dado estado, e o diffing é problema de outra pessoa. O preço é um passo de build e uma dependência que sobrevive ao seu interesse nela. Para uma tabela estática só, DOM puro daria conta; para um painel que ganha painéis novos do M8 ao M10 (uma pista de Solana, um gráfico de latência, uma tira de saúde de worker), você aceita o negócio, porque cada painel novo é outra função pura do mesmo estado. Escolha frameworks por onde o artefato está indo, não pelo que o commit atual precisa.

### O segundo consumidor: o import que prova a fronteira

O movimento três é curto porque m03-l1 fez o trabalho pesado. O painel tem que decidir que cor cada linha recebe, e "up versus degraded" é um julgamento que a estação já faz, em `classifyProbe`, dentro do `pulse-core`. Reimplementar aquelas três linhas localmente funcionaria hoje e descolaria amanhã: alguém reajusta a faixa de latência na frota, esquece o painel, e os pixels começam a discordar dos alertas sobre o que "degraded" quer dizer. Então o painel faz a única coisa defensável:

```ts
import { classifyProbe, type ProbeResult } from "pulse-core";
```

A mesma linha de import que a frota usa, resolvida através do mesmo symlink `workspace:*`, executando o mesmo código. Na lição passada a extração era um argumento; esta linha faz dela um pixel. Um classificador, dois consumidores, zero deriva possível.

![O pacote pulse core fica no centro enquanto a frota, o painel novo e dois consumidores fantasmas futuros todos importam o mesmo classificador.](assets/v04-diagram.webp)

Uma pequena jogada de TypeScript em nível de consumidor justifica o lugar dela aqui. O painel quer um tipo para as chaves do mapa de cores dele: o veredito que `classifyProbe` retorna. O seu próprio `index.ts` por acaso re-exporta `Verdict`, então um import direto funciona, mas faça a jogada que você precisaria contra um pacote de terceiros cuja superfície você não controla: `type Verdict = ReturnType<typeof classifyProbe>`. `ReturnType` é um utility type embutido (um genérico que você consome, exatamente a habilidade de m02-l2) que extrai o tipo de retorno de uma função, e aqui ele também pega uma coisa que o alias exportado não anuncia, como o próximo parágrafo mostra. O mapa de cores fica tipado de qualquer jeito.

Um detalhe honesto decorre de extrair o tipo desse jeito. `classifyProbe` é a forma de fronteira que m02-l4 congelou: ela recebe o par `(kind, value)` não confiável e responde `'invalid'` para um kind que ela não reconhece. Então o tipo que você acabou de extrair tem quatro membros, não os três que a união `Verdict` exportada carrega, e um `Record` sobre ele precisa de uma entrada `invalid` ou o compilador vai nomear a chave faltando.

Duas batidas datadas fecham a teoria. O seu `pulse-board` roda no Vite 8, e o Vite 8.0.0 (entregue em 2026-03-12) é movido a Rolldown: o bundler que tritura o seu TypeScript é escrito em Rust, fixado como `rolldown ~1.2.4` no manifest do próprio Vite. A tese das duas linguagens está sentada no seu `node_modules` agora mesmo, e esse é o tour de bundler inteiro que você ganha. E quando este painel ganhar o painel de Solana dele em m08-l2, o `@solana/kit` dele vai ser fixado lendo faixas de peer, não de memória: a regra de m03-l1, já compondo juros.

**Vá mais fundo (os 20%).** esta lição ensinou a fatia componentes-props-estado-efeito que um consumidor de dados precisa, e para. Profundidade de hooks, context, routing, forms, tudo com formato de framework, fica deliberadamente como bookmark. A rampa de entrada canônica é o [Quick Start](https://react.dev/learn) do próprio React (URL sondada em 2026-09-02): interativo, gratuito, mantido pelo time do React. Leia depois do lab se o React fez sentido e você quer o vocabulário completo; nada abaixo depende disso, e a profundidade séria de lado cliente mora no curso de domínio do lado do cliente de qualquer forma.

## Lab: pulse-board

Meta: o despejo de `<pre>` vira um painel de status tipado, parseado, colorido pelo classificador, que faz polling num interval e falha alto em cima de lixo. Eu dirijo os passos 1 a 4 com o devtools aberto; o passo 5 te entrega uma spec em vez de um diff; o challenge depois do lab é sem guia.

1. **Ligue o scaffold no workspace.** O `pnpm create vite` da abertura já criou `packages/pulse-board`, e porque `pnpm-workspace.yaml` faz glob em `packages/*`, ele já é membro do workspace. Limpe o demo (`src/App.css`, `src/assets`, os imports de logo) e adicione as duas dependências que o painel realmente precisa, a partir de `packages/pulse-board`:

   ```bash
   pnpm add zod
   pnpm add pulse-core --workspace
   ```

   (Freshness: `pnpm add zod` resolveu para 4.5.4 em 2026-09-02; a flag `--workspace` força o protocolo `workspace:*` para que `pulse-core` linke do seu repo, nunca do registro.) Checkpoint: `packages/pulse-board/package.json` agora lista `"pulse-core": "workspace:*"`, e `npm run dev` ainda serve.

2. **Desconte a promissória de m02-l4: refaça a fiação do escritor da frota para a união.** O rodapé de m02-l4 congelou as linhas v0 achatadas do `status.json` e prometeu que o escritor teria a fiação refeita no M3 assim que houvesse um painel para manter verde. O painel está a vinte minutos de distância, então a refação da fiação acontece agora, primeiro, ou o schema que você escreve no próximo passo vai recusar o seu próprio arquivo. Abra `packages/pulse-fleet/fleet.ts`, o escritor do cron. A lista `TARGETS` dele, o envelope de relatório e o caminho de escrita `../../status.json` todos ficam; o que muda é o tipo da linha, do mentiroso `{ url, status, latencyMs: number | string, checkedAt }` para um wrapper carregando a união de verdade:

   ```ts
   import { writeFile } from "node:fs/promises";
   import type { ProbeResult } from "pulse-core";

   type TargetStatus = {
     url: string;
     checkedAt: string;
     result: ProbeResult;
   };

   async function probeOne(url: string): Promise<TargetStatus> {
     const checkedAt = new Date().toISOString();
     const start = performance.now();
     try {
       const res = await fetch(url, { signal: AbortSignal.timeout(10_000) });
       const latencyMs = Math.round((performance.now() - start) * 10) / 10;
       if (!res.ok) {
         return { url, checkedAt, result: { kind: "http-error", status: res.status } };
       }
       return { url, checkedAt, result: { kind: "ok", latencyMs } };
     } catch (err) {
       if (err instanceof Error && err.name === "TimeoutError") {
         return { url, checkedAt, result: { kind: "timeout", budgetMs: 10_000 } };
       }
       return { url, checkedAt, result: { kind: "dns-error", host: new URL(url).hostname } };
     }
   }
   ```

   Percorra o catch, porque ele é o mapa de saída de m02-l3 comprimido em dois braços. O `AbortSignal.timeout` rejeita com um erro *chamado* `TimeoutError`, então aquela checagem de nome é a saída "o nosso próprio timer disparou" e vira a variante `timeout` com o budget que ela estourou. Todo o resto no catch é a própria rede falhando, DNS, conexão recusada, TLS, antes de qualquer budget poder expirar; o `fetch` rejeita esses imediatamente como um `TypeError`, e eles caem no braço `dns-error` exatamente como o treino de m02-l1 ensinou. Um `catch` pelado aqui publicaria um host morto como um timeout de dez segundos, a pequena mentira precisa que esta refação de fiação existe para despejar. Delete a declaração local v0 de `ProbeResult` enquanto você está aí; o tipo agora chega do `pulse-core`, type-only, custando nada ao runtime. Renomeie o tipo do elemento do array de resultados do escritor para `TargetStatus` e o resto compila intocado. Rode uma vez a partir de `packages/pulse-fleet`, `npx tsx fleet.ts`, e abra o `status.json` fresco na raiz do repo: toda linha agora lê `{ "url", "checkedAt", "result": { "kind": ... } }`. O dialeto v0 que m02-l1 tornou irrepresentável em `probe.ts` finalmente foi despejado do único arquivo que ainda tinha permissão de mentir nele. Commite e dê push antes de construir o painel, para que a próxima execução do cron publique linhas de união para a sua URL ao vivo também.

3. **Faça o schema da fronteira.** Crie `src/status.ts`, o checkpoint de fronteira do painel. O schema espelha o relatório que o escritor que você acabou de refazer emite: `generatedAt`, mais uma entrada por alvo envolvendo a união `ProbeResult`:

   ```ts
   import { z } from "zod";

   const probeResultSchema = z.discriminatedUnion("kind", [
     z.object({ kind: z.literal("ok"), latencyMs: z.number() }),
     z.object({ kind: z.literal("timeout"), budgetMs: z.number() }),
     z.object({ kind: z.literal("http-error"), status: z.number() }),
     z.object({ kind: z.literal("dns-error"), host: z.string() }),
   ]);

   const targetStatusSchema = z.object({
     url: z.string(),
     checkedAt: z.string(),
     result: probeResultSchema,
   });

   export const statusFileSchema = z.object({
     generatedAt: z.string(),
     targets: z.array(targetStatusSchema),
   });

   export type StatusFile = z.infer<typeof statusFileSchema>;
   export type TargetStatus = StatusFile["targets"][number];
   ```

   Note o que `z.infer` compra nesta fronteira: o `result` parseado é estruturalmente idêntico ao `ProbeResult` do `pulse-core`, então fazer narrowing nele no próximo passo é narrowing de verdade contra a união de verdade, sem casts em lugar nenhum. Se os nomes de campo da sua frota diferirem dos meus, o schema é o único lugar onde você reconcilia eles; é para isso que serve um checkpoint de fronteira.

4. **O efeito de polling, com o devtools aberto.** Substitua `src/App.tsx`. Antes de colar, abra a aba Network do devtools do navegador e mantenha ela visível; o ponto deste passo é ver a teoria acontecer.

   ```tsx
   import { useEffect, useState } from "react";
   import { statusFileSchema, type StatusFile } from "./status";
   import { StatusBoard } from "./StatusBoard";

   const RAW_URL =
     "https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json";
   const POLL_MS = 60_000;

   type BoardState =
     | { phase: "loading" }
     | { phase: "error"; message: string }
     | { phase: "ready"; data: StatusFile };

   export default function App() {
     const [state, setState] = useState<BoardState>({ phase: "loading" });

     useEffect(() => {
       let cancelled = false;

       async function poll() {
         try {
           const res = await fetch(RAW_URL);
           if (!res.ok) throw new Error(`HTTP ${res.status}`);
           const parsed = statusFileSchema.safeParse(await res.json());
           if (cancelled) return;
           if (parsed.success) {
             setState({ phase: "ready", data: parsed.data });
           } else {
             const message = parsed.error.issues
               .map((i) => `${i.path.join(".")}: ${i.message}`)
               .join("; ");
             setState({ phase: "error", message });
           }
         } catch (err) {
           if (!cancelled) setState({ phase: "error", message: String(err) });
         }
       }

       poll();
       const id = setInterval(poll, POLL_MS);
       return () => {
         cancelled = true;
         clearInterval(id);
       };
     }, []);

     if (state.phase === "loading") return <p>loading fleet status...</p>;
     if (state.phase === "error") return <p>board error: {state.message}</p>;
     return <StatusBoard data={state.data} />;
   }
   ```

   Uma coisa para notar antes de você me culpar por me contradizer: aqueles dois imports locais não carregam extensão `.js`, e m03-l1 foi enfático que os tsconfigs deste curso tornam a extensão não opcional. Os dois são verdade, porque eles são tsconfigs diferentes. A regra de m03-l1 é sobre resolução `nodenext`, onde um especificador nomeia o arquivo emitido. O painel é um pacote create-vite com o próprio conjunto de tsconfigs configurado para resolução `bundler`, onde o bundler resolve o especificador e sem extensão é o idioma. A regra durável não é "sempre escreva `.js`", é "escreva o que o seu modo de resolução exige", e o jeito rápido de saber em qual você está é ler `moduleResolution` no tsconfig sob o qual você está de fato compilando. Não "conserte" essas duas linhas.

   Ossos familiares, deliberadamente: `BoardState` é uma união discriminada (a jogada de m02-l1, agora moldando UI), e a renderização lá embaixo é só narrowing. `StatusBoard` ainda não existe, então o dev server mostra um erro de import; tudo bem por um passo. Duas leituras antes de seguir. Na aba Network, clique na requisição `status.json` e leia os headers de resposta você mesmo: `cache-control: max-age=300`, o `etag`, a linha `via` do varnish. (Sem navegador à mão? `curl -s -D - -o /dev/null https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json` despeja os headers idênticos, `access-control-allow-origin: *` incluído.) Segunda leitura: por que 60 segundos? Os dados mudam a cada 30 minutos e o CDN reusa uma cópia por 5, então fazer polling mais rápido compra só releituras cacheadas; 60s mantém a aba honesta dentro de um minuto de o cache ficar fresco. O interval e o cron são relógios diferentes, e confundir eles é o footgun.

   Agora o bug que você precisa encontrar uma vez. Comente as duas linhas de cleanup (`cancelled = true; clearInterval(id);`), salve, e edite qualquer arquivo algumas vezes para disparar hot reloads. Veja a aba Network encher: cada reload empilhou outro poller, nenhum dos antigos morreu. Eu já entreguei exatamente isso, e o bug report do tipo a-aba-come-um-núcleo-de-CPU que vem depois não tem graça nenhuma. Restaure o cleanup, veja as requisições caírem de volta uma por minuto, e nunca mais escreva um efeito de interval sem o return dele. (Este treino precisa de um navegador de verdade, porque o hot reload é o trigger; trabalhando headless, leia o fluxograma abaixo como a transcrição do treino e solte um `console.count("poll tick")` dentro de `poll` para que o empilhamento apareça como um contador disparando na frente na próxima vez que você tiver devtools.)

![Com cleanup cada hot reload substitui o interval de polling, enquanto sem cleanup cada reload adiciona outro poller vivo até as requisições se empilharem.](assets/v05-flowchart.webp)

5. **StatusRow e as cores do classificador, a partir de uma spec.** Sua vez, nível de assinatura só. Construa `src/StatusRow.tsx` exportando `StatusRow({ target }: { target: TargetStatus })`, uma linha de tabela que: deriva o veredito dela do `classifyProbe` importado, que recebe o par `(kind, value)` congelado, então você faz narrowing em `target.result.kind` e entrega a ele a leitura daquela variante; colore a célula do veredito a partir de um mapa `Record<Verdict, string>` (pegue `Verdict` via `ReturnType<typeof classifyProbe>`, e lembre que ele carrega `'invalid'`); renderiza uma string de detalhe legível por variante fazendo narrowing naquele mesmo `target.result.kind` (latência para `ok`, budget para `timeout`, código para `http-error`, host para `dns-error`); e mostra `checkedAt` como hora local. Uma variante precisa de uma decisão sua: um `dns-error` carrega um hostname, não uma leitura numérica, então ele não tem nada para entregar ao classificador. A minha, para depois de você ter tentado:

   ```tsx
   import { classifyProbe } from "pulse-core";
   import type { TargetStatus } from "./status";

   type Verdict = ReturnType<typeof classifyProbe>;

   const VERDICT_COLOR: Record<Verdict, string> = {
     up: "#22c55e",
     degraded: "#eab308",
     down: "#ef4444",
     invalid: "#a1a1aa",
   };

   export function StatusRow({ target }: { target: TargetStatus }) {
     // classifyProbe is the frozen (kind, value) boundary form. A dns-error
     // has a hostname and no reading, so the board decides that one here.
     const verdict: Verdict =
       target.result.kind === "ok"
         ? classifyProbe("ok", target.result.latencyMs)
         : target.result.kind === "timeout"
           ? classifyProbe("timeout", target.result.budgetMs)
           : target.result.kind === "http-error"
             ? classifyProbe("http-error", target.result.status)
             : "down";

     const detail =
       target.result.kind === "ok"
         ? `${target.result.latencyMs} ms`
         : target.result.kind === "timeout"
           ? `no answer in ${target.result.budgetMs} ms`
           : target.result.kind === "http-error"
             ? `HTTP ${target.result.status}`
             : `DNS failed for ${target.result.host}`;

     return (
       <tr>
         <td>{target.url}</td>
         <td style={{ color: VERDICT_COLOR[verdict] }}>{verdict}</td>
         <td>{detail}</td>
         <td>{new Date(target.checkedAt).toLocaleTimeString()}</td>
       </tr>
     );
   }
   ```

   E o painel que mapeia as linhas, `src/StatusBoard.tsx`, que é honestamente simples demais para especificar:

   ```tsx
   import type { StatusFile } from "./status";
   import { StatusRow } from "./StatusRow";

   export function StatusBoard({ data }: { data: StatusFile }) {
     return (
       <table>
         <thead>
           <tr>
             <th>target</th>
             <th>verdict</th>
             <th>detail</th>
             <th>checked</th>
           </tr>
         </thead>
         <tbody>
           {data.targets.map((t) => (
             <StatusRow key={t.url} target={t} />
           ))}
         </tbody>
       </table>
     );
   }
   ```

   Checkpoint, e é o ponto inteiro da lição: o dev server agora mostra linhas dos seus alvos de verdade, latências que o seu cron mediu, coloridas pela função exata que trava as publicações da frota. Um `solana.com` verde na sua tela e um `solana.com` verde nos logs do cron nunca podem discordar, porque eles são uma função só.

6. **O treino do arquivo corrompido.** Prove a fronteira antes de confiar nela. Copie uma resposta de verdade para `public/corrupt.json`, depois quebre ela na mão: mude o `"kind": "ok"` de uma linha para `"kind": "okay"` (a mentira forjada de m02-l1, de volta para se vingar). Aponte `RAW_URL` para `/corrupt.json` temporariamente e recarregue. Esperado: nenhuma página em branco, nenhum choro só no console, mas o seu estado de erro, na tela, nomeando o caminho e o discriminante que falhou. Essa mensagem é o zod recusando na fronteira, exatamente como projetado. Aponte `RAW_URL` de volta para a sua URL raw e confirme que as linhas voltam.

7. **Build limpo.** A partir de `packages/pulse-board`:

   ```bash
   npm run build
   ```

   O script de build do template react-ts roda `tsc -b` antes de `vite build`, então esta é a trava de tipos e o bundler em uma linha só. Esperado: zero erros de tipo e uma pasta `dist/`. Essa pasta é um site totalmente estático, que é precisamente o que faz a próxima lição ser tão curta.

## Challenge

Sem guia, fechando o loop na conta da desatualização. Adicione um indicador de "última atualização" que: (a) mostra `generatedAt` como hora local; (b) computa a idade do dado mais novo a partir dos valores buscados, não de quando você buscou eles; (c) vira visualmente para desatualizado (cor, badge, sua escolha) depois de dois intervalos do cron, 60 minutos, porque uma execução perdida é um soluço e duas são um incidente. Já que você está aí, adicione uma query string de cache-busting ao fetch (`` `${RAW_URL}?t=${Date.now()}` `` faz de cada poll uma chave de cache distinta, trocando a gentileza do CDN por frescor). A bala de prata para painéis com cara de desatualizado? Não existe uma; existem duas estratégias honestas, e agora o seu painel faz as duas.

![Um cartão de duas colunas compara cache busting contra um indicador honesto de desatualização em frescor, custo e modos de falha, terminando com os dois adotados.](assets/v06-comparison.webp)

Aceitação, todas as quatro: o painel renderiza linhas de verdade da frota coloridas pelo classificador importado; o treino do arquivo corrompido mostra o estado de erro do zod na tela; o indicador de desatualização vira com dados velhos (teste isso alimentando um arquivo local adulterado com timestamps de uma hora atrás); `npm run build` sai limpo.

## Checkpoint, e a primeira URL visível para estranhos

Faça o balanço do que ficou provado. A extração sobreviveu a um segundo consumidor de verdade: uma linha de import, e a frota e o painel nunca podem descolar no que "degraded" quer dizer. A aposta do repo público pagou o dividendo dela: um app de navegador com zero backend, lendo dados de verdade entre origens porque os headers sondados permitem. E a disciplina de fronteira aguentou em território novo: bytes lixo morrem no schema com uma mensagem, não na renderização com uma página em branco. A pergunta de recuperação, a frio, sem notas: nomeie os dois atrasos empilhados entre uma sonda rodar e um pixel mudar. Se você disse o agendamento do cron e o cache de 5 minutos do CDN, o modelo mental está instalado.

Se o painel renderiza mas as linhas parecem erradas, confie na ordem de depuração que a lição ensinou: headers de resposta primeiro (é o cache?), estado de erro do schema em segundo (é o formato?), componente por último. O componente quase nunca é o mentiroso; ele é uma função pura do que quer que tenha sido entregue a ele.

O painel roda lindamente, no localhost, onde exatamente uma pessoa na Terra consegue ver ele. Aquela pasta `dist/` do passo 7 está ali parada, totalmente estática, precisando de nada além de um host. Próxima lição: `vercel login`, `vercel`, e uma URL que você pode mandar por mensagem para um estranho. A primeira URL do curso, na lição dez. Traga um telefone.
