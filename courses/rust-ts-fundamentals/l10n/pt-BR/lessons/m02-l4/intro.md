# Testes são sondas para o seu código: vitest

## Resumo

A frota sonda cinquenta alvos e publica o que encontra. Nada sonda a frota. Esta lição vira isso do avesso: você escreve uma suíte vitest que fixa as fronteiras do classificador como uma tabela, afirma o cronograma de backoff sem esperar um único milissegundo real, e tranca o bug de config de m02-l2 para fora para sempre, depois liga a suíte inteira ao workflow pulse.yml para que um classificador quebrado nunca mais consiga publicar status.json. Gate #2 no pipeline, e a estação começa a monitorar a si mesma.

## Primeiro, quebre alguma coisa

m02-l3 ensinou boas maneiras à frota sob carga: um pool feito à mão, backoff com jitter, timeouts com AbortController. Cinquenta alvos sondados com zero 429s, cada desfecho chegando tipado. A frota funciona. Nada ainda prova que ela continua funcionando.

Essa distinção é a lição inteira, então vamos torná-la concreta antes de qualquer teoria. Instale a ferramenta:

```bash
npm i -D vitest@4.1.11
```

(Re-sondado em 2026-09-06, e isto é uma lição sobre dist-tags por si só: quando esta lição foi escrita em 2026-09-02, a v5 ainda era um release candidate e era a linha v4 que a `latest` servia. O Vitest 5.0.0 entrou em GA em 2026-09-03 e tomou a `latest`; a linha v4 agora vive na sua própria tag `V4`, em 4.1.11. O pin acima é exatamente o motivo de a instalação nomear uma versão em vez de confiar na `latest`: toda transcrição desta lição é da v4 mesmo. Comece na v5 se preferir, leia as notas de migração dela primeiro, e tudo o que esta lição ensina sobre testes como trava continua valendo. `npm view vitest dist-tags` imprime o mapa atual em uma linha.)

Uma peça de setup para que a sonda tenha um alvo estável, e ela carrega uma renomeação que você tem que fazer de propósito, porque os dois classificadores que você escreveu até agora querem o mesmo nome. O seu classificador de l1 ainda mora onde aquele lab o deixou, chamado `classifyProbe` e chaveado no objeto de união; o challenge de m02-l1 avaliou uma assinatura diferente com o mesmo nome. Mova o arquivo para `src/classify.ts` e divida o nome em dois:

- **`classify(result: ProbeResult): Verdict`** — a forma de união de l1, renomeada. Ela fica, e não por sentimentalismo: é a única forma capaz de julgar um `dns-error`, que carrega um hostname e nenhum número, e o painel do M3 vai precisar exatamente disso.
- **`classifyProbe(kind: string, value: number)`** — a forma de fronteira que o challenge avaliou. Ela roda o `parseProbe` primeiro (kinds desconhecidos voltam como `'invalid'`), depois entrega o resultado parseado para o `classify`.

Dois nomes exportados, um switch exaustivo por baixo: escreva a forma de fronteira em cima da forma de união, nunca ao lado dela, ou você acabou de construir dentro de um único arquivo a deriva sobre a qual este curso não para de te avisar. Faça a renomeação em uma passada só e deixe o `npx tsc --noEmit` te levar a cada ponto de chamada, incluindo a linha do driver de l1; essa tarefa é o que m02-l1 te vendeu e esta é a primeira vez que você a gasta. Estes são também exatamente os nomes que a extração de pacote de m03-l1 move e que o board de m03-l2 importa, então acertá-los hoje é uma renomeação que você não faz depois. Dez minutos, nenhuma lógica nova, e todo teste desta lição importa esse único arquivo. (O `probe.ts` da raiz encolhe para uma CLI que importa de `./src/classify.js`. Uma ruga honesta para notar em vez de consertar: o `src/fleet.ts` mantém a cópia local de `ProbeResult` que ele declarou em l3, então o repo agora guarda duas uniões estruturalmente idênticas. O `src/classify.ts` é o canônico a partir de hoje, a duplicação é exatamente o risco de deriva que a extração de pacote do M3 existe para fechar, e você tem permissão de se incomodar com isso até lá.)

Agora escreva um teste. Crie `tests/classify.test.ts` ao lado da sua frota:

```ts
import { expect, test } from 'vitest';
import { classifyProbe } from '../src/classify.js';

test('a 400ms probe is degraded, not up', () => {
  expect(classifyProbe('ok', 400)).toBe('degraded');
});
```

Rode:

```bash
npx vitest run
```

Se a sua lógica de fronteira estiver certa, você ganha um check verde. Se a sua checagem de fronteira disser `> 400` onde a spec de m02-l1 dizia que a faixa degraded começa EM 400, você acabou de pegar o tipo de bug que o cron teria publicado a cada trinta minutos, para sempre, com um check verde do lado. De um jeito ou de outro você aprendeu algo real em cinco minutos, que é o discurso de venda inteiro.

Duas grafias desse comando, e a diferença importa a lição inteira: `npx vitest run` executa a suíte uma vez e sai, que é o que o CI quer. `npx vitest` puro inicia o watch mode: ele fica vivo, re-roda os testes afetados toda vez que você salva um arquivo, e transforma a suíte em uma leitura ao vivo enquanto você trabalha. Use o watch mode na sua mesa pelo resto deste lab; a forma com `run` é a que vai para o workflow depois. E note que a gente invoca o vitest com `npx` em todo lugar neste módulo, nunca por um script `test` do `package.json`: o stub do npm-init em `scripts.test` fica intocado hoje, deliberadamente, e m03-l1 conecta `"test": "vitest run"` no momento em que o `pnpm -r test` do workspace realmente precisar de um script para achar.

Aqui vai a síntese que dá o título a esta lição: uma suíte de testes é um monitor de uptime apontado para o seu próprio código. Você já construiu a versão voltada para fora. Uma asserção é uma sonda com uma leitura esperada. Um teste que falha é um 429 vindo da sua própria lógica. Mesma disciplina, apontada para dentro, e você já conhece a disciplina.

## Sondas apontadas para dentro

Você vem fazendo "teste" manualmente desde m01-l2: rodar a frota, bater o olho na saída, balançar a cabeça. Isso funciona até o código mudar enquanto você não está olhando a saída, que é o que o resto deste curso é. Todo módulo daqui em diante adiciona código do qual outro código depende. A suíte é como uma mudança descuidada na forma da saída da frota é pega antes de quebrar o painel que você vai entregar no módulo 3, que lê essa forma e nada mais.

![Cinco conceitos de monitoramento como sondas e leituras esperadas mapeiam um a um para conceitos de teste como chamadas de função e asserções.](assets/v01-diagram.webp)

O vitest é o runner que este curso usa: ele fala TypeScript nativamente com zero config, e encontra qualquer coisa que case com `*.test.ts`. Cerca de 99.9 milhões de downloads por semana na hora em que isto foi escrito, pelo que quer que contagens de download valham; não é um veredito do ecossistema Solana, no entanto, e esta lição vai te mostrar o outro campo antes de terminar. Os padrões abaixo são os 80% do dia a dia: tabelas, fake timers, fixtures, cobertura. Todo o resto fica como bookmark no fim desta seção.

### A tabela é a spec

O seu classificador tem um contrato, e você já o sabe de cor porque o challenge de m02-l1 te avaliou nele: latência abaixo de 400 é `up`, de 400 até 1000 é `degraded`, acima de 1000 é `down`, um 429 significa que o alvo respondeu, então é `degraded` e não `down`, kinds desconhecidos são `invalid`. Cinco regras de fronteira. Você poderia escrever cinco funções de teste separadas e repetir a cerimônia cinco vezes, ou poderia notar que todas são a mesma frase com números diferentes:

```ts
import { expect, test } from 'vitest';
import { classifyProbe } from '../src/classify.js';

const rows: Array<[kind: string, value: number, expected: string]> = [
  ['ok', 399, 'up'],
  ['ok', 400, 'degraded'],
  ['ok', 1000, 'degraded'],
  ['ok', 1001, 'down'],
  ['http-error', 429, 'degraded'],
  ['http-error', 500, 'down'],
  ['timeout', 0, 'down'],
  ['gopher', 200, 'invalid'],
];

test.each(rows)('classifyProbe(%s, %d) is %s', (kind, value, expected) => {
  expect(classifyProbe(kind, value)).toBe(expected);
});
```

`test.each` (um teste de tabela: um corpo de teste, rodado uma vez por linha) transforma o contrato em dados. Leia as linhas em voz alta e você está lendo a spec. Essa é a vitória de verdade, não a digitação economizada: quando o challenge de m02-l1 acrescentou a regra de que 429 é degraded, isso foi uma linha. Quando um bug de fronteira um dia aparecer em produção, o pin de regressão é uma linha. Os testes mais baratos de estender são os que têm mais chance de serem estendidos, e uma tabela custa uma linha por lição aprendida.

Repare em quais linhas estão aqui. Não entradas aleatórias: os valores exatos em que o comportamento muda. 399 e 400. 1000 e 1001. Fronteiras são onde moram os bugs de off-by-one, então fronteiras são para onde as sondas apontam.

Uma palavra rápida sobre a asserção em si, porque você vai recorrer a ela duzentas vezes neste curso. `toBe` checa identidade: resposta certa para strings, números, booleanos, qualquer coisa que o classificador retorne. No momento em que você afirmar sobre um objeto ou um array, mude para `toEqual`, que compara estrutura. `expect({ a: 1 }).toBe({ a: 1 })` falha, dois objetos diferentes, mesma forma; `toEqual` passa. Esse par cobre a maior parte da sua vida de asserções. O catálogo de matchers vai muito mais fundo (`toMatchObject`, `toThrow`, `resolves`, e você vai encontrar `resolves` no passo 3), mas toBe-para-valores e toEqual-para-formas é o reflexo do dia a dia que vale instalar agora.

### Fake timers: afirme o cronograma, pule a espera

O cronograma de backoff de m02-l3 também é um contrato: a tentativa n espera `min(capMs, baseMs * 2^n)`. Com uma base de 500ms, um teto de 8000ms e cinco retries, isso dá 500, 1000, 2000, 4000, 8000. (A frota do lab rodava um teto de 5000ms; o teste sobe para 8000 para que todo delay exercite a duplicação antes do clamp.) Teste isso com timers reais e cada execução da suíte gasta quinze segundos reais dormindo, o que quer dizer que você para de rodar a suíte, o que quer dizer que você não tem mais uma suíte.

![Cinco barras dobram de 500 a 8000 milissegundos, somando mais de quinze segundos de espera que os fake timers eliminam.](assets/v02-chart.webp)

`vi.useFakeTimers()` (a substituição de relógio do vitest: ele intercepta `setTimeout` e companhia para que callbacks agendados disparem quando VOCÊ avança o relógio, não quando o relógio de parede avança) foi feito exatamente para esta forma de código. O modelo mental principal, e o que o quiz vai cutucar: fake timers não encolhem os delays. Os horários agendados mantêm seus valores exatos. Você pula o relógio para cada instante agendado e afirma o que disparou. Essa precisão é o motivo de o teste conseguir fixar o cronograma valor por valor em vez de afirmar que "mais ou menos cinco esperas aconteceram".

O padrão, primeiro num brinquedo:

```ts
import { afterEach, beforeEach, expect, test, vi } from 'vitest';

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

test('the callback fires at 500ms, not before', async () => {
  const fired = vi.fn();
  setTimeout(fired, 500);

  await vi.advanceTimersByTimeAsync(499);
  expect(fired).not.toHaveBeenCalled();

  await vi.advanceTimersByTimeAsync(1);
  expect(fired).toHaveBeenCalledTimes(1);
});
```

`vi.fn()` é um spy: uma função falsa que registra como foi chamada. `advanceTimersByTimeAsync` move o relógio falso e deixa qualquer promise que estivesse esperando por esses timers se resolver. A dança 499-depois-1 é a mesma disciplina de fronteira da tabela: afirme que nada acontece um milissegundo antes, depois afirme que acontece exatamente na hora. Você vai fazer isso contra o loop de retry de verdade no lab.

Uma cilada antes de você encontrá-la: fake timers só ajudam se o código sob teste for puro o suficiente para ser dirigido. O seu classificador e a sua fórmula de backoff recebem valores e devolvem valores; eles nunca tocam a rede. Essa foi uma decisão de projeto que m02-l1 e m02-l3 tomaram antes de você saber por quê, e este é o porquê. Sondar URLs reais dentro de testes unitários deixa a suíte flaky, lenta e limitada por rate limit, e você já sabe exatamente o que o alvo acha de rajadas. O motor Rust do M4 vai fazer a mesma jogada de manter-o-núcleo-puro de propósito, e a gente vai dizer isso de novo lá.

### Fixtures: o bug que nunca mais pode voltar

Em m02-l2 você deu um typo em um campo de config, `intervalSeconds` onde o código lê `intervalSecs`, e rastreou como a versão sem parse da frota engoliria isso educadamente. Depois você construiu a fronteira zod e viu o mesmo arquivo morrer na inicialização com um erro em nível de campo. Esse arquivo com typo está prestes a ganhar uma promoção: de história de guerra a fixture (um arquivo de entrada versionado no repo que os testes carregam; evidência congelada, repetida para sempre).

A jogada é pequena e é um dos hábitos de maior valor do curso: todo bug que você corrige vira um teste que falha se o bug voltar. No lab você vai cometer de novo aquele crime contra o schema ATUAL da frota (m02-l3 remodelou a config para a frota concorrente, então o arquivo original é um schema desatualizado), estacionar a cópia sabotada em `tests/fixtures/` e afirmar que o `safeParse` a RECUSA. Não "a frota parece bem". Recusa, afirmada, pelo nome. Se alguém um dia afrouxar o schema e a mentira educada voltar a ser representável, a suíte fica vermelha antes de o cron conseguir publicar uma única linha errada.

![Um pipeline de cinco estágios mostrando um bug sendo corrigido, congelado como um arquivo de fixture, fixado por um teste de recusa e permanentemente pego se ele um dia voltar.](assets/v03-diagram.webp)

### Cobertura: sinal, não ídolo

Rode uma suíte com cobertura e você ganha uma porcentagem: quantas linhas do seu fonte executaram enquanto os testes rodavam. Pergunta útil de se fazer, número péssimo de se idolatrar, e você precisa das duas metades dessa frase.

A metade útil: uma branch não coberta é um alvo de sonda que ninguém está observando. O seu classificador tem braços para `timeout`, para `http-error`, para kinds desconhecidos. Se a cobertura mostrar que o braço do `timeout` nunca rodou, nenhum teste da suíte notaria se ele começasse a retornar `up`. Esse é exatamente o instinto de monitoramento para fora que você já tem: um alvo sem nenhuma sonda apontada para ele pode ficar down por uma semana sem ninguém saber. Leia o relatório de cobertura do jeito que você lê a lista de alvos da frota, procurando a lacuna que importa.

A metade do ídolo: 100% de cobertura de linhas prova que toda linha RODOU sob algum teste. Não prova nada sobre se as asserções nessas linhas pegariam uma resposta errada. Uma suíte que chama toda função e não afirma nada tira nota máxima. E as últimas branches não cobertas muitas vezes são inalcançáveis de propósito: o seu braço `assertNever` existe precisamente para NÃO PODER rodar, e perseguir um número que o penaliza significa apagar o seu próprio guarda-corpo para agradar uma métrica. A cobertura aponta o que olhar. Humanos decidem o que importa.

### O ecossistema, nomeado sem enfeite

Uma batida para cada um, porque você vai encontrar os três no mundo real:

![Três runners de teste comparados pelo que são e por quando um dev os encontra, com o vitest escrito aqui e o jest lido no mundo real.](assets/v04-comparison.webp)

`node:test` é a nota lateral de zero dependências: um runner de teste de verdade que já vem dentro do próprio Node, estável desde o Node 20, sem instalação nenhuma. A história de cobertura dele ainda é experimental, que é por isso que ele é a nota lateral e não a lição. Vale saber que existe; algumas ferramentas pequenas genuinamente não precisam de mais nada.

o jest é o incumbente, e honestidade importa mais do que lealdade tribal aqui: você vai encontrar a linha jest 30 nos repos da anza, o kit e o gill, os dois, testam com ele. As APIs são parecidas por projeto, então ler os testes deles vai parecer familiar. As configs não são parecidas, e essa é a cilada: copiar config do jest de um repo do mundo real para dentro de um projeto vitest produz falhas misteriosas, porque a semelhança está nos arquivos de teste, não no encanamento.

Se a escala de décadas dos ecossistemas te surpreende, a essa altura não deveria: o Express 5 sucedeu o Express 4 depois de dez anos (2014-04-09 a 2024-09-10), e o ecossistema rodou o major antigo felizinho o tempo todo. Testes são a mesma história. Você aprende a ferramenta atual e lê o incumbente, porque o mundo real roda os dois, e "ler o mundo real" é uma habilidade que este curso não para de comprar para você de propósito.

### A parte honesta

Testes são código que você também tem que manter, e fingir o contrário é como times acabam odiando suas suítes. Toda mudança no classificador agora quebra linhas de tabela. O teste de backoff fixa os delays tão apertado que reajustar o cronograma de propósito significa editar testes, o que é levemente irritante exatamente quando você está com pressa. A trava de CI que você está prestes a construir adiciona minutos entre o merge e a publicação. Todo esse atrito é o ponto: atrito no caminho de publicar uma mentira é o produto.

Mas nomeie onde isso se inverte, porque se inverte. Testes superespecificados, do tipo que afirmam strings de log incidentais ou enfiam a mão em detalhes internos privados, encarecem toda refatoração sem pegar nenhum bug de verdade: o teste quebra a cada reescrita de texto e nunca num erro de lógica. E um ALVO de cobertura, "exigimos 95%", persegue linhas em vez de risco e te dá testes que executam código sem checá-lo. A bússola para os dois: teste o contrato, não a implementação; trave a publicação, não cada tecla digitada.

**Vá mais fundo (os 20%).** os padrões do dia a dia acima são o que a frota precisa; o resto do vitest é uma caixa de ferramentas profunda que você deve saquear sob demanda, não decorar. O [guia do vitest](https://vitest.dev/guide/) (verificado ao vivo em 2026-09-02) cobre o que deixamos de propósito como bookmark: a taxonomia de mocking (mocks, spies, module mocks), snapshot testing e browser mode. Nota lateral para o caminho de zero dependências: a [documentação do node:test](https://nodejs.org/api/test.html). Nada no lab abaixo depende do material que ficou como bookmark.

## Lab: a suíte, depois a trava

O cronograma de rodinhas do M2 se encerra neste lab, e aqui está o recuo, em voz alta: os testes de tabela a gente constrói junto linha por linha, o teste de fake timer e o teste de config você completa a partir de setups dados, e a fiação de CI é trabalhada de novo MAS você dirige cada push. No próximo módulo os apoios começam a afinar de verdade.

### 1. O primeiro teste que falha, de propósito

Você escreveu `tests/classify.test.ts` na abertura. Agora faça ele mentir para você deliberadamente, porque você nunca deveria confiar em um teste que não viu falhar. Inverta a expectativa:

```ts
expect(classifyProbe('ok', 400)).toBe('up'); // wrong on purpose
```

```bash
npx vitest run
```

```
 FAIL  tests/classify.test.ts > a 400ms probe is degraded, not up
AssertionError: expected 'degraded' to be 'up'
```

Leia essa falha do jeito que você lê um resultado de sonda: leitura esperada, leitura real, delta. Inverta a expectativa de volta para `'degraded'`, veja ela ficar verde. Esse ritmo vermelho-depois-verde é o ciclo de confiança, e você vai rodá-lo no pipeline inteiro no fim deste lab.

![Uma mensagem de falha do vitest anotada para mostrar onde aparecem o nome do teste que falhou, o valor real e o valor esperado.](assets/v05-annotated-code.webp)

### 2. A tabela do classificador, linha por linha

Substitua o teste único pela tabela `test.each` da seção de teoria, todas as oito linhas. Construa nesta ordem e observe o que cada acréscimo compra: primeiro as quatro linhas `ok` (os dois lados das duas fronteiras de latência), rode, verde. As duas linhas `http-error` (429 versus 500, a regra que o challenge avaliou), rode, verde. Depois `timeout` e o kind desconhecido. Oito linhas, oito checks verdes, e o contrato que você vem carregando na cabeça desde m02-l1 agora mora em algum lugar que o compilador e o runner conseguem alcançar.

### 3. O teste de backoff (você escreve o relógio)

Setup dado, asserções suas. Uma extração primeiro: m02-l3 deixou o loop de retry inline no `probeWithRetry`, com `Math.random()` embutido na linha do jitter, e um cronograma com um termo aleatório dentro não pode ser fixado. Puxe o loop para `src/backoff.ts` como `retryOn429(fn, { baseMs, capMs, retries, jitter, isRetryable })`: curva base determinística no código, e as duas decisões de julgamento injetadas no ponto de chamada. A injeção do jitter você esperava; o predicado `isRetryable` é o que torna a extração possível, porque o helper é genérico sobre o que quer que `fn` retorne e não tem como saber o que "ocupado" significa para ele. A frota passa a função de equal-jitter mais `isRetryable: (r) => r.kind === 'http-error' && r.status === 429`, mapeando o `maxRetries` da sua config na grafia mais curta `retries` da opção na chamada (o nome da config mora na fronteira de parse; o helper está livre para grafar as opções do jeito dele). O teste passa `jitter: () => 0`, um `fn` de brinquedo que responde a string `'429'`, e um predicado de uma linha que combina, e então fixa valores exatos. Aponte o `probeWithRetry` para o novo helper, confirme que a frota ainda roda, e volte. Aqui está o harness:

```ts
import { afterEach, beforeEach, expect, test, vi } from 'vitest';
import { retryOn429 } from '../src/backoff.js';

beforeEach(() => {
  vi.useFakeTimers();
});
afterEach(() => {
  vi.useRealTimers();
});

test('five retries wait exactly 500, 1000, 2000, 4000, 8000 ms', async () => {
  const alwaysBusy = vi.fn(async () => '429' as const);
  const run = retryOn429(alwaysBusy, {
    baseMs: 500,
    capMs: 8000,
    retries: 5,
    jitter: () => 0,
    isRetryable: (r) => r === '429',
  });

  await vi.advanceTimersByTimeAsync(0); // flush the first attempt
  expect(alwaysBusy).toHaveBeenCalledTimes(1);

  // YOUR TURN from here: advance to one ms BEFORE the first retry,
  // assert nothing fired, then land each retry on its exact instant.
  await vi.advanceTimersByTimeAsync(499);
  expect(alwaysBusy).toHaveBeenCalledTimes(1);

  await vi.advanceTimersByTimeAsync(1); // t = 500
  expect(alwaysBusy).toHaveBeenCalledTimes(2);

  // ... continue: t = 1500, 3500, 7500, 15500 ...

  await expect(run).resolves.toBe('429');
});
```

(Se o seu arquivo de m02-l3 grafa o helper de outro jeito, mantenha os seus nomes. O teste fixa comportamento, não grafia.)

Descubra sozinho os avanços de relógio restantes antes de rodar: cada retry cai no instante anterior mais o próximo delay do cronograma, então 500, depois 1500, depois 3500, depois 7500, depois 15500. Seis chamadas no total: a primeira tentativa mais cinco retries. Quando as suas asserções ficarem verdes, olhe o tempo de execução que o vitest reporta para o arquivo. Milissegundos. Você acabou de verificar quinze segundos e meio de comportamento agendado sem esperar por nada disso.

![Seis tentativas de retry caem em instantes exatos do relógio falso, de zero a 15500 milissegundos, enquanto o tempo real mal passa.](assets/v06-timeline.webp)

### 4. O teste de fronteira da config (a fixture se paga)

Recrie o crime de m02-l2 contra o schema atual da frota: pegue o seu `pulse.config.json` bom, renomeie um campo obrigatório, `timeoutMillis` no lugar de `timeoutMs`, e salve a cópia sabotada como `tests/fixtures/pulse.bad.json`. É a mesma classe de typo do original `intervalSeconds`; o schema foi reconstruído desde então para a frota concorrente, então o pin mira um campo que ele ainda possui. Setup dado, asserção sua:

```ts
import { readFileSync } from 'node:fs';
import { expect, test } from 'vitest';
import { configSchema } from '../src/config.js';

test('the m02-l2 typo class is refused at the boundary', () => {
  const raw = JSON.parse(
    readFileSync(new URL('./fixtures/pulse.bad.json', import.meta.url), 'utf8'),
  );

  const result = configSchema.safeParse(raw);

  expect(result.success).toBe(false);
  if (!result.success) {
    const paths = result.error.issues.map((issue) => issue.path.join('.'));
    expect(paths.join('\n')).toContain('timeoutMs');
  }
});
```

Duas asserções, duas garantias diferentes. A primeira diz que o schema recusa o arquivo, ponto. A segunda diz que a recusa NOMEIA o campo faltante, porque uma fronteira que falha sem dizer onde é pouco melhor do que uma que mente. Este é o pin de regressão: a mentira educada de m02-l2 agora não pode voltar sem este teste ficar vermelho antes.

Acrescente o teste espelho você mesmo antes de seguir: uma segunda fixture, `pulse.good.json` (uma cópia da sua config de verdade), e um teste afirmando que o `safeParse` a ACEITA, `expect(result.success).toBe(true)`. Parece redundante hoje. Para de parecer redundante na primeira vez que alguém aperta um refinamento e acidentalmente tranca a config de produção do lado de fora; uma fronteira que rejeita tudo é tão quebrada quanto uma que admite tudo, e agora as duas direções têm uma sonda.

Vou confessar a origem do meu entusiasmo aqui: eu já rodei um monitor por meses com um bug de fronteira de classificador quase idêntico ao do passo 1, e o achei não por nenhum alarme, mas lendo o log cru à toa num domingo. Todo relatório que ele tinha publicado naquela janela estava sutilmente errado, e cada um tinha sido entregue com um deploy verde do lado. Não me custou nada além de confiança, que é a coisa cara. O hábito da fixture é o que eu faço a respeito dessa memória.

### 5. Leia a cobertura, corrija uma lacuna

```bash
npm i -D @vitest/coverage-v8@4.1.11
npx vitest run --coverage
```

(O pacote de cobertura versiona em sincronia com o próprio vitest; mantenha os dois fixados juntos.)

Leia o relatório como uma lista de alvos, não como um placar. Olhe o `src/classify.ts` primeiro, e uma nota de honestidade sobre toolchain antes de você sair caçando: nos pins atuais (vitest 4.1.11 com coverage-v8 4.1.11 no Node 24), o `src/backoff.ts` pode simplesmente não aparecer na tabela de cobertura, mesmo quando o arquivo de teste dele roda, então se a linha estiver faltando, isso é a lacuna de relatório da ferramenta, não prova de cobertura perfeita nem de cobertura zero, e o `src/classify.ts` é onde gastar o exercício. Em algum lugar da sua frota existe uma branch que a suíte nunca executa; na maioria dos builds deste projeto é um braço de mapeamento de erro (o braço `dns-error` do classificador é um achado comum) ou, se a sua tabela mostrar o backoff, a borda teto-abaixo-da-base da fórmula, o caso em que `capMs` é menor que `baseMs` e todo delay é achatado no teto. Ache a SUA lacuna, pergunte se um bug ali chegaria ao status.json, e se sim, acrescente a linha ou o caso que a cobre. Se a linha não coberta for o seu braço `assertNever`, deixe ela sem cobertura e aproveite o lembrete de por que o número é um sinal e não uma meta.

### 6. O re-ship: os testes travam o cron

Agora a costura do módulo: a suíte se junta ao pipeline que você construiu em m01-l3, e este workflow é o único artefato que este curso faz crescer até o capstone. Travas de Rust se juntam a ele no M4, builds de release mais tarde ainda. Hoje ele aprende a recusar.

Abra `.github/workflows/pulse.yml`. Você está adicionando um job e uma aresta:

```yaml
name: pulse

on:
  push:
    branches: [main]
  schedule:
    - cron: "*/30 * * * *"

permissions:
  contents: write

jobs:
  typecheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm ci
      - run: npx tsc --noEmit

  test: # NEW: the suite as a job
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm ci
      - run: npx vitest run

  probe:
    needs: [typecheck, test] # CHANGED: was `needs: typecheck`
    runs-on: ubuntu-latest
    steps:
      # your existing probe-and-commit steps, unchanged
```

(As tags de action `checkout@v7` e `setup-node@v7` foram sondadas como os majors atuais em 2026-09-02; o Node 24 é o Active LTS na hora em que isto foi escrito, com a v26 agendada para assumir em 2026-10-28.)

A linha estrutural é `needs: [typecheck, test]` (a chave `needs:` declara dependências entre jobs: o `probe` não vai começar a menos que todo job que ele nomeia tenha tido sucesso). Sem essa aresta, o job de test roda AO LADO do probe e não trava nada; uma suíte vermelha e um commit novo de status.json cairiam na mesma execução, o que é decoração, não trava. O diff é a lição. Leia.

![Triggers de push e de schedule alimentam as travas de typecheck e de test, cujas setas de needs apontam as duas para o job probe que faz commit do status json.](assets/v07-flowchart.webp)

Agora prove que a trava existe, vermelho primeiro, porque o passo 1 te ensinou a nunca confiar em uma checagem não testada. Plante o bug do classificador de propósito: inverta a sua fronteira de 400 para `> 400` no `src/classify.ts`, faça commit em uma branch, faça merge (ou dê push direto na main; a estação é sua, e esta é a única ocasião para quebrar a main deliberadamente). Observe a execução do Actions: o job de test fica vermelho na linha `classifyProbe(ok, 400) is degraded`, e o job probe aparece como pulado. Abra o repo: o status.json não tem commit novo. O pipeline se recusou a publicar a mentira. Tire o screenshot; esta execução vermelha é a evidência de aceitação e, com honestidade, é um artefato satisfatório de se guardar.

Depois reverta o bug plantado, dê push, e assista à sequência verde: typecheck passa, test passa, probe roda, status.json atualiza. Mais uma coisa que você agora sabe e que a maioria nunca confere: porque workflows agendados rodam no último commit da branch padrão, o PRÓXIMO tick do cron depois de um merge vermelho teria rodado a mesma suíte vermelha e recusado de novo, a cada trinta minutos, até alguém consertar. Um merge vermelho sem trava vira um status.json errado no agendamento, sem ninguém olhando. Um com trava vira uma publicação parada e um X vermelho que alguém vai ver. Parado é melhor que mentiroso. Esse é o projeto inteiro.

Agora a nota de rodapé franca sobre o que exatamente ficou travado, porque um módulo que gastou quatro lições em tipos verdadeiros não deveria ser vago sobre o próprio caminho de publicação. O job chamado `probe` ainda roda o escritor original do `fleet.ts` de m01-l3, intocado: na forma v0 dele, `latencyMs` ainda tipado de forma frouxa, ainda a única coisa que escreve `status.json`. Tudo o que você retipou em l1 e reconstruiu de forma concorrente em l3 mora ao lado dele, e o que as duas travas protegem é aquele código. Isso é deliberado, não um descuido: a forma `{ url, status, latencyMs, checkedAt }` do `status.json` é um contrato congelado que o painel de m03-l2 está prestes a renderizar, e trocar o publicador debaixo de um consumidor que ainda não existe é como você quebra os dois de uma vez. O escritor é religado em m03-l2, do outro lado da mudança para o workspace, como a primeira jogada de lab daquela lição, assim que houver um schema de painel na tela para manter verde enquanto você faz isso.

Verifique local e remotamente antes de seguir: `npx vitest run` verde na sua máquina, e a execução do Actions do commit que você deu push mostrando o job de test terminando antes de o step do probe começar.

## Challenge

Sem challenge avaliado nesta lição; os mais fortes do módulo vivem em l1 e l3, e escrever testes está provado pela suíte que o lab acabou de travar. Em vez disso, uma rep sem orientação e com coisa de verdade em jogo: plante um bug DIFERENTE, um que a suíte atual NÃO pegue. Quebre o ponto de chamada do jitter, ou faça o schema de config aceitar um `timeoutMs` negativo, e confirme que o pipeline continua verde até um status.json publicado. Fique sentado com a sensação disso. Depois escreva o teste que o teria pego, veja ele falhar contra o bug plantado, reverta o bug, veja ele passar, e deixe o teste ali. Você acabou de fazer o loop profissional completo: achar o alvo não observado, apontar uma sonda para ele, manter a sonda. Repita para sempre, em todo emprego que você tiver.

## Onde isso deixa a estação

A vitória de 30 segundos antes de você fechar o terminal: em uma frase, por que o job de test tem que travar o CRON especificamente, e não só rodar em pushes? Diga em voz alta. Se a sua frase contiver "branch padrão", "em um agendamento" e "status.json errado sem ninguém olhando", você tem a lição inteira. E a promessa feita para a frente, textualmente para você reconhecê-la quando ela chegar: no M4 você vai encontrar o cargo test. Mesma ideia, flag diferente.

Se a suíte pegou um bug de verdade seu hoje, mesmo um parente do plantado, eu genuinamente quero saber; essa primeira salvada é o momento em que este hábito deixa de ser dever de casa.

A frota está tipada, parseada, disciplinada e se automonitorando, e tudo isso mora em uma árvore de arquivos crescente que só você consegue usar. No próximo módulo o motor vira um pacote de verdade: workspaces, package.json como contrato, um painel React, e a primeira URL que um estranho pode abrir. As rodinhas saem na porta do workspace.
