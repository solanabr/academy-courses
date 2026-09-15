# Leituras em produção: o painel da Solana

## Resumo

Na lição passada você construiu o medidor de bancada: `chain-probe.ts` mede o tempo de slot contra o alvo de 300ms, deriva a contagem regressiva da epoch a partir da epoch fixa de 432,000 slots, e te deu o modelo de cliente de quatro ideias, com tudo o que é mais profundo passado ao curso btc-to-sol pelo nome. Funciona. Também roda em exatamente uma máquina, a sua, num terminal que mais ninguém vai ver. Um medidor que ninguém consegue ver é um medidor que não existe.

Hoje é o maior re-ship do curso, e eu quero dizer a parte silenciosa primeiro: nada aqui é novo, exceto o alvo. A leitura é uma linha que você já entende. O polling é o do m03-l2. O cache é o KV do m07-l1. O backoff é o do m02-l3. O que muda é onde tudo isso roda: a mesma leitura da blockchain, promovida para toda superfície com deploy de que a estação é dona. No fim, o painel no Vercel renderiza um painel ao vivo da Solana e o JSON público do worker carrega um snapshot da blockchain em cache, os dois nas URLs que você já entregou.

Prove a leitura primeiro. No repo da estação, onde o `@solana/kit@^8` está instalado desde a lição passada, largue isto em `packages/pulse-fleet/balance.ts` ao lado de `chain-probe.ts` e rode a partir desse diretório (a casa dos scripts de bancada da lição passada, com o `"type": "module"` e o tsx de que os scripts precisam):

```ts
import { createSolanaRpc, address } from "@solana/kit";

const rpc = createSolanaRpc("https://api.mainnet.solana.com");
const watched = address("So11111111111111111111111111111111111111112");

const { value: lamports } = await rpc.getBalance(watched).send();
console.log(lamports);
```

```bash
npx tsx balance.ts
# 1807515117625n
```

Esse endereço é o mint do wrapped SOL, uma conta movimentada da mainnet que vai continuar existindo no ano que vem; o número que você receber vai ser diferente da minha execução de 2026-09-02. Note o `n`. Esse saldo é um `bigint`, está em lamports, e a razão de o kit se recusar a te passar um number puro e simples é a primeira coisa que a produção ensina hoje.

Como esta lição roda, em voz alta: a leitura canônica e o esqueleto do painel são trabalhados na tela uma vez; a fiação do polling, a chave de cache no KV e o orçamento de backoff são seus para compor a partir de padrões que você já possui; a extensão do segundo-endereço-observado no fim é totalmente solo. Esse é o recuo do M8 e ele não volta atrás.

## A mesma leitura, toda superfície

### Uma leitura da blockchain é só mais uma sonda

Aqui está a síntese que deixa esta lição pequena: uma leitura da blockchain é só mais uma sonda. É um POST HTTP para um endpoint com teto de taxa que em geral responde rápido, às vezes responde devagar e de vez em quando te recusa. O que quer dizer que toda pergunta de produção que ela levanta foi respondida semanas atrás, no módulo dois, antes de este curso ter dito a palavra Solana. Com que frequência eu posso chamar? Orçamento. E se falhar? Backoff, depois degrade para o último status conhecido. Quem paga quando cinquenta navegadores perguntam de uma vez? Todo mundo atrás do teto compartilhado, junto. O único material genuinamente novo hoje é um tipo de dado e uma disciplina, e os dois cabem numa seção cada.

![Uma única leitura da blockchain alimenta três superfícies, com setas em negrito promovendo ela para o painel e o edge worker com deploy enquanto o script de bancada fica local.](assets/v01-diagram.webp)

O formato da estação depois de hoje, concretamente: o `pulse-board` (Vercel) faz polling da blockchain diretamente e renderiza slot, tempo de slot medido e um saldo observado. O `pulse-edge-ts` (Cloudflare) dobra as leituras de slot e de saldo no cron de 15 minutos que ele já tem, escreve o resultado no KV sob uma chave `chain`, e serve isso no JSON público ao lado do bloco `targets` que ele já publica. Dois deploys, zero plataformas novas, e o teste de aceitação é duas abas de navegador mostrando o mesmo slot dentro de um refresh uma da outra.

### O número que mente educadamente

O JavaScript tem um único tipo numérico, um float de 64 bits, e ele é exato para inteiros só até `Number.MAX_SAFE_INTEGER`: 2^53 menos 1, que é 9,007,199,254,740,991. Em lamports, isso é mais ou menos 9 milhões de SOL. Acima dessa linha, a aritmética de `number` não lança, não avisa, nem balança visivelmente. Ela arredonda. `9007199254740993` vira `9007199254740992` e o console imprime isso com confiança total. Para um contador de visualizações, quem liga. Para um display de saldo, essa é a mentira educada que entrega um número errado para a tela de alguém.

Contas de verdade da mainnet ficam acima dessa linha, carteiras de exchange e stake pools entre elas, que é por isso que o kit tipa todo u64 como `bigint` e nunca como `number`. Isso não é uma escolha de performance; o parse de bigint é, se for alguma coisa, mais lento. É uma política de porta de corretude: nenhum saldo pode ser corrompido em silêncio no caminho para dentro do seu programa. A oportunidade de corrupção se move para o seu lado da porta, e ela tem exatamente um formato: no momento em que você roteia um valor em lamports através de `Number` para poder fazer matemática de float com ele, você reintroduziu o bug que o kit existe para prevenir.

![Uma reta numérica em escala logarítmica mostra saldos abaixo da fronteira de dois elevado a cinquenta e três renderizando exatamente, enquanto valores maiores arredondam em silêncio.](assets/v02-chart.webp)

Então o contrato de formatação do painel é matemática de BigInt até a string. A divisão dá o SOL inteiro, o resto dá a fração, e a convenção de exibição é fixa: nove dígitos fracionários, preenchidos à esquerda, zeros à direita aparados, sem ponto decimal em valores inteiros.

```ts
export function lamportsToSol(lamports: bigint): string {
  const whole = lamports / 1_000_000_000n;
  const frac = lamports % 1_000_000_000n;
  if (frac === 0n) return whole.toString();
  const digits = frac.toString().padStart(9, "0").replace(/0+$/, "");
  return `${whole}.${digits}`;
}
```

O `padStart` é estrutural e é onde esta função costuma ser escrita errado: um resto de `2_500_000n` não é ".25", é nove dígitos de fração com dois zeros à esquerda, ".0025". A minha execução exatamente desta função contra o saldo de mainnet de hoje: `1807515117625n` entrando, `"1807.515117625"` saindo, e `123456789123456789n` faz round-trip para `"123456789.123456789"` sem perder um dígito. Aquele último valor está acima de 2^53 de propósito; ele é a fixture com que o coding challenge vai bater na sua versão. Um footgun vizinho já que estamos aqui, porque ele morde na seção do worker: `JSON.stringify` lança um `TypeError` em bigint. Em qualquer lugar em que um valor em lamports atravessa para dentro do JSON, você converte para string explicitamente antes. O painel nunca passa os bigints dele por JSON, mas o snapshot do KV precisa.

### O painel: o painel aprende a dar as horas

O painel já sabe fazer polling: o m03-l2 construiu o intervalo de `useEffect` com cleanup e uma flag de cancelamento, e esse esqueleto transfere inteiro. O que muda é a fonte dos dados (o RPC, através do kit, em vez de um arquivo de status cru) e uma métrica derivada: o tempo de slot medido. Duas amostras consecutivas de `getSlot` te dão um delta de slot e um delta em tempo de relógio; divida e você tem o batimento da blockchain como o seu painel observa ele, sentado ao lado do alvo de 300ms. E sim, este é o método ingênuo de duas amostras que a lição passada construiu como descartável, revivido de propósito, então diga o trade-off em voz alta em vez de deixar ele parecer amnésia: o medidor de bancada queria vinte minutos do histórico gravado do próprio nó para JULGAR a blockchain, e o seu borrão de ida e volta teria poluído aquele veredito; o painel quer um batimento ao vivo aproximado a um refresh de dez segundos por zero requisições extras, e `getRecentPerformanceSamples` por tick gastaria uma terceira requisição por aba contra os tetos compartilhados para comprar uma precisão de que um painel de status não precisa. O borrão pega carona em cada leitura do painel, alguns milissegundos de barulho nessas janelas, e o trabalho do rótulo é apresentar o número como a observação que ele é, não como a medição da lição passada. A aritmética de slot acontece em bigint, e só o delta pequeno final atravessa para `number` para a divisão, que é a direção correta de atravessar: um delta de algumas dezenas de slots está longe do penhasco.

Instale o kit onde o painel mora (a partir de `packages/pulse-board`):

```bash
pnpm add @solana/kit
# resolved to 8.2.0 on 2026-09-02; the digit rule below explains why yours may differ
```

O esqueleto trabalhado, `src/SolanaPanel.tsx`; ele compila limpo sob strict mode contra o kit para o qual aquela linha de install acabou de resolver, porque eu conferi antes de colar ele aqui:

```tsx
import { useEffect, useState } from "react";
import { createSolanaRpc, address } from "@solana/kit";
import { lamportsToSol } from "./lamports";

const RPC_URL = "https://api.mainnet.solana.com";
const WATCHED = address("So11111111111111111111111111111111111111112");
const POLL_MS = 10_000;
const TARGET_SLOT_MS = 300;

type PanelState =
  | { phase: "loading" }
  | { phase: "error"; message: string }
  | { phase: "ready"; slot: bigint; slotTimeMs: number | null; lamports: bigint };

const rpc = createSolanaRpc(RPC_URL);

export function SolanaPanel() {
  const [state, setState] = useState<PanelState>({ phase: "loading" });

  useEffect(() => {
    let cancelled = false;
    let last: { slot: bigint; at: number } | null = null;

    async function poll() {
      try {
        const slot = await rpc.getSlot().send();
        const { value: lamports } = await rpc.getBalance(WATCHED).send();
        const now = Date.now();
        let slotTimeMs: number | null = null;
        if (last && slot > last.slot) {
          slotTimeMs = (now - last.at) / Number(slot - last.slot);
        }
        last = { slot, at: now };
        if (!cancelled) setState({ phase: "ready", slot, slotTimeMs, lamports });
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

  if (state.phase === "loading") return <p>reading the chain...</p>;
  if (state.phase === "error") return <p>chain read failed: {state.message}</p>;

  return (
    <section>
      <h2>Solana</h2>
      <p>slot {state.slot.toString()}</p>
      <p>
        slot time{" "}
        {state.slotTimeMs === null
          ? "measuring..."
          : `${state.slotTimeMs.toFixed(0)}ms (target ${TARGET_SLOT_MS}ms)`}
      </p>
      <p>watched balance {lamportsToSol(state.lamports)} SOL</p>
    </section>
  );
}
```

Cada batida estrutural é o painel do m03-l2: o `PanelState` discriminado, a flag de cancelamento, o cleanup que mantém o hot reload em exatamente um poller vivo. O primeiro tick mostra "measuring..." para o tempo de slot porque uma taxa precisa de duas amostras; isso é honestidade, não um bug.

Agora o orçamento, porque este componente gasta um recurso compartilhado a cada tick. Os tetos do RPC público são 100 requisições por 10 segundos por IP, e 40 por método na mesma janela. Este painel faz 2 requisições por tick, e num `POLL_MS` de dez segundos isso dá 2 requisições por 10 segundos por aba aberta, 1 por método. Tranquilo para você, tranquilo para o punhado de pessoas para quem você manda a URL. Mas rode a aritmética do jeito do m02-l3 antes de confiar nisso: a pista por método enche por volta de 40 abas fazendo polling na mesma cadência atrás de um NAT, um escritório, um local de evento, um alojamento. Passando disso, todo visitante atrás daquele IP começa a comer 429s, e o seu painel mostra erros causados pela própria popularidade dele. A lição passada chamou esse descompasso de problema do painel com deploy e prometeu ele; a seção do worker é a resposta, e o status honesto DESTE painel é: leituras diretas estão corretas no seu tráfego, e no momento em que não estiverem, o snapshot em cache que você está prestes a construir é para onde o painel aponta em vez disso. Essa refiação está deliberadamente deixada para o capstone, que faz polling do JSON público do worker a partir do painel como a sua única aresta nova.

![Um tick de polling gasta duas requisições contra tetos compartilhados por IP, faz loop a cada dez segundos e desvia por um estado de erro em caso de falha.](assets/v03-flowchart.webp)

### O worker: faça cache da blockchain, sirva o cache

A relação do worker com a blockchain é diferente em espécie, e ela vem da arquitetura do m07-l1 e não de qualquer coisa específica da Solana. O painel faz polling enquanto um humano olha para ele. O worker roda num cron sem ninguém olhando, escreve o que aprendeu no KV, e serve a última verdade conhecida para quem perguntar, de memória do mundo em vez de um olhar fresco para ele. Dados da blockchain encaixam nesse modelo sem uma única mudança estrutural: mais um refresh no handler agendado, mais uma chave no KV, mais um bloco no JSON público.

O contrato do snapshot, e o único lugar em que a disciplina de bigint encontra o JSON:

```ts
export interface ChainSnapshot {
  slot: string;      // bigint, stringified: JSON.stringify throws on raw bigint
  balances: Record<string, string>;  // address -> lamports as string, same reason
  fetchedAt: string; // ISO timestamp: the snapshot's honesty field
}
```

`fetchedAt` não é decoração. Um cache que esconde a idade dele é um painel que mente; o consumidor do JSON decide o que desatualizado quer dizer, e ele só consegue decidir se você carimbar. O refresh em si é o kata do m02-l3 vestindo roupa de blockchain: embrulhe cada leitura em backoff (equal jitter, o `backoffDelay` que você extraiu para o pulse-core no m03-l1), e na falha final não escreva nada, para que o snapshot anterior continue servindo enquanto o próximo tick do cron tenta de novo. Degrade para desatualizado, nunca para vazio. Para uma superfície de status essa propriedade é o jogo inteiro, e você construiu ela no módulo dois sem saber que esta lição estava vindo.

![Uma pista dirigida por cron faz o refresh do snapshot da blockchain para dentro do KV com backoff e duas saídas de falha, enquanto uma pista separada serve o snapshot em cache para as requisições.](assets/v04-flowchart.webp)

Esta arquitetura assume que o kit roda dentro do workerd, uma suposição que vale desconfiar: o worker não é node, e metade do registro npm descobre isso do jeito difícil. Eu rodei o teste de fumaça enquanto escrevia isto: um worker hello-world, `npm i @solana/kit`, `createSolanaRpc(...).getSlot().send()` no fetch handler, `wrangler dev`. O kit carregou, resolveu e executou limpo dentro do isolate. O pacote já traz uma export condition `workerd` explícita, e o runtime fornece o Web Crypto que o kit quer, então isto é território suportado, não sorte. A resposta de verdade da minha sonda ainda foi um erro, e vale a pena ler ela: HTTP 403, "Your IP or provider is blocked from this endpoint." O mesmo 403 voltou para um POST de `fetch` pelado do mesmo isolate, então não foi o kit, foi a blocklist do endpoint público não gostando do egress do ambiente da minha sonda. Duas lições num corpo de erro só. Primeiro, transporte não é o seu risco; a política do endpoint compartilhado é, e um provedor bloqueado é mais um modo de falha que o seu caminho de degradação já absorve. Segundo, é precisamente por isso que a arquitetura de snapshot ganha: quando um caminho de leitura é recusado, o JSON público do worker continua servindo a última verdade que ele aprendeu em vez de encaminhar a recusa para todo visitante.

Duas praticidades antes de o lab fazer a fiação disso. O build workerd do kit resolve para o build node dele, então a data de compatibilidade do seu `wrangler.jsonc` importa: datas de 2026-08-04 ou posteriores habilitam a compatibilidade com node por padrão, e o scaffold do m07-l1 é mais novo que isso, então você está coberto; uma data mais antiga precisa da flag `nodejs_compat` acrescentada na mão. E a saída de emergência, dita porque uma promessa ganha de um mistério: se o kit-no-worker um dia brigar com você, o fallback é uma edição de uma tabela, não uma reescrita. O seu worker já fala JSON-RPC cru para este endpoint exato, já que a sonda `getHealth` do m07-l1 é um POST de fetch pelado. `getSlot` e `getBalance` são mais duas strings de método no mesmo estilo, e a arquitetura (cron, KV, backoff, degradar) não muda uma linha. A razão de o kit ser o caminho ensinado mesmo assim é o tipo de dado: o kit te passa lamports como bigint, enquanto `JSON.parse` numa resposta de RPC crua te passa um `number` que já arredondou qualquer coisa acima do penhasco antes de o seu código chegar a ver.

### Fixe a regra, não o dígito

Você pode ter notado que esta lição não te disse qual versão do kit instalar, só mostrou para o que o `pnpm add` resolveu na minha máquina numa data declarada. Isso é deliberado, e é a lição de peers do m03-l1 chegando na recompensa dela. Entre 2026-06-16 e 2026-08-21, o kit entregou três releases em pouco mais de nove semanas, dois deles majors: o minor 6.10.0, depois 7.0.0, depois 8.0.0, pelos timestamps do próprio npm. Qualquer tutorial que congelou um dígito na prosa durante esse trecho apodreceu antes do próximo café do autor, duas vezes, e este curso se recusa a se juntar a eles. A linha de install que você roda é verificada no dia em que você roda; dígitos em prosa não são.

![Uma linha do tempo de dez semanas marca um minor e dois releases major sucessivos da biblioteca kit, terminando na versão observada na data de escrita.](assets/v05-timeline.webp)

Então o pin se resume a uma regra: leia contra o que as próprias dependências do seu workspace dão peer, e fixe nisso. Os pacotes de cliente gerado sob `@solana-program/*` declaram, nas `peerDependencies` deles, exatamente contra quais majors do kit eles foram construídos, e o npm impõe esse contrato na hora do install. A leitura leva um comando. Aqui está a saída ao vivo da minha sonda de hora de escrita:

```bash
npm view @solana-program/system version peerDependencies
# version = '0.14.1'
# peerDependencies = { '@solana/kit': '^8.0.0' }
```

Hoje, aquela saída nomeia o major para o qual as minhas linhas de install resolveram. No dia em que você rodar, ela pode nomear o próximo, e então AQUELE é a sua resposta, não importa o que qualquer tutorial ou esta própria página diga. A regra corta dos dois lados, que é o que faz dela uma regra em vez de um conselho: fixe abaixo da faixa de peer e o install falha alto; force para além de um erro de peer com uma flag de override e você entrega um descompasso de versão que falha em tempo de execução em vez disso, o que é estritamente pior, porque o gerenciador de pacotes estava lendo o contrato por você e você mandou ele parar. A sua estação ainda não tem nenhum pacote `@solana-program/*`, leituras não precisam de nenhum, então hoje o pin honesto é simplesmente o que `pnpm add @solana/kit` resolve. A regra está nas suas mãos para o dia em que um cliente gerado entrar na árvore, e o hábito mais profundo generaliza para muito além da Solana: o fato durável em qualquer ecossistema rápido nunca é o dígito, é onde o dígito está autoritativamente escrito. Siga em frente neste catálogo e você vai encontrar a mesma regra ensinada mais duas vezes, de propósito: o curso de pagamentos ensina ela como uma costura por workspace, dois workspaces em um repo fixados em majors diferentes do kit porque as dependências deles dão peer de forma diferente, e o m08 do curso Anchor V2 treina ela contra a faixa de peer declarada de um cliente gerado. Três cursos, uma regra, de três direções, porque é o único hábito que sobrevive ao ritmo de release deste ecossistema.

![Fixar no dígito de versão de um tutorial apodrece sob o retrabalho do ecossistema enquanto fixar nas faixas de peer das suas próprias dependências atualiza junto com o contrato.](assets/v06-comparison.webp)

### Para onde o kit está indo, e quando o gratuito deixa de bastar

Uma caixa sobre o futuro, para que a documentação não te embosque. O guia de upgrade do site do kit agora lidera com uma API em estilo de plugin: `createClient()` com composição via `.use(...)`. É trabalho do próprio kit, é para onde a biblioteca está indo, e o painel não usa isso. Os pacotes de plugin que dão suporte a ela estavam em 0.19.0 na minha sonda de hora de escrita, e você gastou uma lição inteira no que pré-1.0 quer dizer: minors carregam direitos de breaking change. O estilo pipe-e-RPC que esta lição ensina é o caminho estável documentado, o dialeto que os exemplos e os clientes gerados do próprio ecossistema do kit falam hoje, então tudo o que você fiou aqui é fundação, não um beco sem saída. Quando os plugins cruzarem o 1.0, rode esta matemática de novo; a ordenação da página de documentação é marketing, versões de pacote são evidência.

E o livro-razão de custos do que você construiu, porque toda arquitetura é uma conta. O snapshot que o worker serve fica desatualizado em até um intervalo de cron, quinze minutos no seu agendamento atual, mais o que quer que a consistência eventual acrescente por cima. Esse é o preço, e o que ele compra é sobrevivência: a alternativa era gastar o orçamento compartilhado de 100-por-10-segundos em visualizações de página, o que em qualquer tráfego de verdade converte o seu painel num gerador de 429 para todo mundo atrás do mesmo IP. Para um painel de status, valores-da-última-vez-conhecida com um `fetchedAt` honesto é o lado certo desse trade-off. Saiba o que você NÃO construiu, porém: no momento em que o seu produto precisar de dados da blockchain frescos por push, websockets, streaming, histórico indexado, você já cresceu para além de polling-e-cache inteiramente. Essa profundidade, junto com tudo sobre aterrissar transações, pertence ao curso de domínio do lado do cliente, em produção enquanto escrevo; até ele ser entregue, aqueles nomes de tópico são os seus termos de busca. Esta lição lê estado, ponto final.

Quando o próprio endpoint público deixar de bastar, existe um próximo passo sancionado que cabe na regra de sem-cartão deste curso: a Helius oferece um tier gratuito a $0 com 1,000,000 de créditos por mês e 10 requisições por segundo, sem cartão de crédito, segundo o preço publicado dela em 2026-09-02. Trocar para ele é uma mudança de URL em uma constante, a mesma disciplina de sonda vale, e esse parágrafo único é tudo o que este curso tem a dizer sobre provedores. A migração inteira, quando o dia chegar:

```typescript
// the one line that changes when you outgrow the public door
const RPC_URL = "https://api.mainnet.solana.com"; // -> your provider URL, nothing else moves
console.log(new URL(RPC_URL).host);
```

**Vá mais fundo (os 20%).** esta lição te ensinou o padrão de leitura em produção; a superfície completa da API, todo método de RPC, subscriptions e o roadmap de plugins moram na documentação do kit em https://solanakit.com, que é o recurso para deixar como bookmark, sondada ao vivo em 2026-09-02. Tudo com formato de RPC que você encontrar daqui em diante é uma variação do formato ler-poll-cache-degradar que agora é seu.

## Lab: promova a leitura

Estimados 45 minutos de construção. Os passos 1 e 2 são trabalhados acima; a partir do passo 3 você está compondo padrões que já possui contra contratos declarados.

1. **Checagem de bancada (feito).** Se `npx tsx balance.ts` imprimiu um bigint na abertura, a leitura funciona a partir da sua máquina e o seu install do kit está atual. Se imprimiu um 403 com "blocked from this endpoint," leia de novo a história do teste de fumaça da seção do worker: o egress da sua rede está na blocklist do endpoint, e o lab continua funcionando porque as superfícies com deploy rodam a partir de outras redes. Note qual erro você recebeu; essa alfabetização é a lição.

2. **O formatador.** Crie `packages/pulse-board/src/lamports.ts` com `lamportsToSol` exatamente como especificado no contrato (nove dígitos preenchidos, aparados, sem ponto em inteiros). Este arquivo também é o alvo do coding-challenge, e a lista de fixtures dele é o teste de aceitação: `2500000n` renderiza `0.0025`, `1n` renderiza `0.000000001`, `123456789123456789n` faz round-trip exatamente.

3. **O painel.** Acrescente `SolanaPanel.tsx` a partir do esqueleto trabalhado, monte ele em `App.tsx` ao lado do painel que já existe, e monte ele FORA dos retornos antecipados do painel: o App trabalhado retorna cedo no loading e no erro, e um painel colocado abaixo desses retornos some sempre que a fonte de status está fora do ar, que é precisamente quando você quer o painel da blockchain ainda visível. Depois rode `npm run dev`. Checkpoint: o slot renderiza dentro de um tick, o tempo de slot lê "measuring..." uma vez e depois um número nos 300 e poucos, e o saldo observado aparece com uma fração plausível. Depois dê push. O pipeline do m03-l3 faz o resto, e a sua URL de produção vercel.app agora é um medidor da blockchain. Abra ela no seu celular.

4. **O refresh do worker, seu para escrever.** No `pulse-edge-ts`: `npm i @solana/kit` (npm, não pnpm, e isso está correto aqui: o worker é um projeto npm próprio dele fora do workspace pnpm da estação, exatamente como o m07-l1 fez o scaffold dele), depois um `refreshChain(kv)` que implementa o contrato `ChainSnapshot`. A composição é completamente especificada por coisas que você possui: cada leitura embrulhada em retries dirigidos pelo `backoffDelay` do pulse-core com equal jitter (base 500, teto 5000, a base e o teto do m02-l3; 3 retries no máximo é o orçamento mais apertado do próprio worker, já que um tick de cron não tem razão para esperar os cinco da frota), bigints convertidos em string antes do `JSON.stringify`, `fetchedAt` carimbado, `PULSE_KV.put("chain"...)` em caso de sucesso, e em retries esgotados: retorne sem escrever. Chame ele a partir do handler agendado depois do loop de alvos que já existe.

5. **Sirva isso.** Estenda o JSON do fetch handler: leia a chave `chain` e retorne ela ao lado de `targets`. Checagem local primeiro: `npx wrangler dev`, bata na rota de teste agendada (`curl "http://localhost:8787/cdn-cgi/handler/scheduled"`), depois `curl http://localhost:8787/` e encontre o bloco chain. Depois `npx wrangler deploy`.

6. **Cutuque a produção.** A trava, textualmente do verificador do curso:

   ```bash
   curl -s https://pulse-edge-ts.<your-subdomain>.workers.dev/ | grep -o '"chain"'
   ```

   Duas abas: o painel do Vercel e o JSON do workers.dev. Mesmo slot, dentro de um refresh uma da outra. Essa é a vitória de 30 segundos.

   Uma saída de emergência, porque a história do teste de fumaça pode vir atrás do seu deploy também: se o grep não encontrar nada e `npx wrangler tail` mostrar todo refresh morrendo em erros 403 "blocked from this endpoint", o egress do seu worker está na blocklist do endpoint público, e degradar-para-desatualizado não tem nada para o que degradar, já que nenhum snapshot chegou a ser escrito num deploy novo. Três saídas honestas. Troque `RPC_URL` para o fallback sem chave que o m07-l1 documentou, `https://solana-rpc.publicnode.com` (sem conta, uma constante, verificado respondendo getHealth de dentro do workerd em 2026-09-04); ou troque por uma URL de provedor (o tier gratuito da Helius da seção de custo: cadastro, sem cartão, uma constante muda); ou mantenha o endpoint público e submeta a saída do tail mostrando os 403s como a sua evidência de trava em vez disso. Um worker construído corretamente e recusado pela política de IP de um endpoint demonstrou tudo o que este passo existe para testar, inclusive a alfabetização em falhas, e um worker que faz failover para um fallback documentado demonstrou uma coisa a mais.

7. **Mate a blockchain, veja ela degradar.** No dev local, dê um typo na constante da URL de RPC, dispare a rota agendada, e confirme que o JSON servido ainda carrega o snapshot anterior com o `fetchedAt` mais velho dele. Um aviso de barulho para você não caçar um bug fantasma: com o host irresolvível, a rota de teste agendada do wrangler pode responder `exception` e o workerd pode logar linhas de "internal error" não capturado mesmo enquanto o seu `refreshChain` captura e degrada corretamente; a condição de aprovação é o JSON servido ainda carregando o snapshot velho, não uma resposta limpa da rota de teste. Tire o typo. Se em vez disso um refresh que falhou apagou o seu bloco chain, o seu `refreshChain` escreveu alguma coisa no caminho de falha; conserte isso antes de qualquer outra coisa, é a propriedade para a qual o design inteiro existe.

![O JSON público do worker ganha um bloco chain segurando um slot em string, um mapa de balances e o timestamp fetched-at próprio dele ao lado dos targets que já existem.](assets/v07-annotated-code.webp)

## Challenge: o segundo endereço pega carona

Solo, sem apoio. Acrescente um segundo endereço observado ao painel e ao worker, escolhido por você, qualquer conta da mainnet que você ache interessante. A restrição que faz disso um exercício de design, dita de forma apertada para que a solução preguiçosa falhe nela: o worker mantém exatamente UMA passada de refresh, um orçamento de backoff e uma escrita no KV por tick, com todo saldo observado pegando carona dentro dessa mesma passada. Parafusar uma segunda chamada de `refreshChain`, um segundo envelope de backoff ou uma segunda chave de KV satisfaz a letra de "funciona" e falha o exercício; a chamada extra de `getBalance` em si é a única requisição nova legítima. Você vai achar a costura no seu código do passo 4 em menos de um minuto olhando: reestruturar `WATCHED` de constante para lista, se você já não construiu dessa forma, é o truque inteiro, e decidir se o polling direto do painel também deveria fazer batch é a sua decisão para tomar e defender num comentário de código.

Ao lado disso, o coding challenge `lamports-to-sol` está no ar no runner do curso: o starter já vem com o clássico bug de resto sem padding, renderizando `2500000n` como `0.2500000`, duas ordens de magnitude errado num display de dinheiro. Os testes incluem a fixture acima-de-2^53, então uma conversão sorrateira com `Number()` não consegue passar. Divisão de BigInt, resto, trabalho de string, nada mais.

Aceitação da lição inteira: as duas URLs com deploy respondem com dados ao vivo da blockchain; um alvo de RPC morto degrada para último-status-conhecido em vez de dar erro; um saldo acima de 2^53 lamports renderiza corretamente pelo seu formatador.

## Checkpoint, e o que a estação acabou de virar

Diga o que você já consegue fazer, porque é muito vestido de pouco: ler estado da blockchain com o kit em toda superfície que é sua, manter inteiros do tamanho de dinheiro honestos do RPC ao pixel, orçar um teto de taxa compartilhado por uma frota de navegadores de estranhos, cachear uma leitura atrás de um cron com um timestamp honesto e um caminho de degradação, e escolher uma versão de dependência lendo o contrato que a sua própria árvore declara em vez de confiar no dígito congelado de um estranho. A pergunta de recuperação antes de você fechar a aba: o seu painel mostra um saldo de exatamente `9007199254740993` lamports; por que você pode confiar nele? Diga a resposta em uma frase, e se a palavra bigint não estiver nela, releia a seção do penhasco.

Pedido de feedback, específico desta vez: o passo 4 foi o maior bloco de composição sem guia que o curso te deu. Me diga onde ele rangeu. Se você recorreu à lição m02-l3 para re-derivar o formato do backoff, isso é o recuo funcionando; se você recorreu a ela porque o contrato aqui subespecificou alguma coisa, isso é um bug nesta lição, e eu quero o número da linha.

A metade em TS da estação agora lê a blockchain em produção, da bancada ao navegador ao edge. Mas a estação tem uma segunda linguagem e uma terceira superfície: o poller em Docker do M6 continua cego para a blockchain, e ele não ganha o kit, porque ninguém embrulhou o fio para Rust do jeito que o kit embrulha para TS, e na próxima lição isso acaba sendo a melhor coisa a respeito dele. O caminho do Rust vai direto no fio JSON-RPC com reqwest e serde_json, e a descoberta esperando lá é que tudo o que M4 e M5 te ensinaram sobre parsear JSON não confiável e modelar erros É código de cliente da Solana.
