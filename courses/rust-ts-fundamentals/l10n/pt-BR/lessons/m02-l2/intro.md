# Parse, don't validate: zod na fronteira

## Resumo

Na lição passada você trocou os resultados de sonda stringly do v0 pela união `ProbeResult`. Estados impossíveis perderam a codificação deles, e o switch do classificador agora é exaustivo com prova do compilador. Mas essa prova cobre só valores nascidos dentro do programa. Hoje você a estende para valores nascidos fora: o arquivo de config da frota, e a primeira resposta RPC Solana de verdade da frota. Você vai construir a camada de config da frota pulse: um schema zod v4 que recusa lixo na inicialização com um erro no nível do campo, um tipo `FleetConfig` derivado desse schema para que tipo e validação nunca possam entrar em descompasso, um único helper genérico honesto usado nas duas fronteiras, e um parser de `getBalance` que traz os lamports à tona como `bigint` porque um número JavaScript mentiria para você caladinho. No fim, `npx tsx src/check-config.ts pulse.config.json` imprime um resumo tipado para uma config boa e morre fazendo barulho, nomeando o campo quebrado exato, para uma ruim.

## O padrão: um parser retorna o tipo

Antes de qualquer teoria, rode o bug que esta lição existe para matar. Os controles da frota se movem hoje de arrays hardcoded para um `pulse.config.json`, e o jeito óbvio de carregar isso é como metade da produção faz: `JSON.parse` mais um cast. Cometa o crime em miniatura, um arquivo descartável, `src/naive-load.ts`. (Uma nota de layout, já que esta é a primeira vez que você vê `src/`: deste módulo em diante, código novo da frota mora em um diretório `src/`, então `mkdir -p src` se você não tiver um. `probe.ts` e `fleet.ts` ficam na raiz do repo, onde os caminhos `npx tsx` do workflow de m01-l3 esperam eles; os dois vão sendo costurados juntos conforme o módulo avança.) O arquivo é um loader baseado em cast segurando uma config com um campo com typo, `"intervalSeconds"` onde o código lê `intervalSecs`:

```ts
type FleetConfig = { intervalSecs: number };
const raw = '{ "intervalSeconds": 60 }'; // the typo: Seconds, not Secs
const config = JSON.parse(raw) as FleetConfig;
const waitMs = (config.intervalSecs || 0) * 1000;
console.log(`waiting ${waitMs}ms between probes`);
```

```bash
npx tsx src/naive-load.ts   # prints: waiting 0ms between probes
npx tsc --noEmit            # exits clean. green.
```

Nada falha. Nem no load, nem no cast, nem na compilação. O `intervalSecs` ausente lê como `undefined`, o fallback `|| 0` converte ele para zero, e o tempo de espera do loop de sondagem agora é nada. Ponha esse loader dentro do cron do GitHub Actions do módulo um e a frota sondaria alegremente em um loop quente, martelando os seus alvos tão rápido quanto o fetch consegue disparar, e cada linha individual de código envolvida parece correta.

Aqui está a parte que deveria te incomodar. A união da lição passada não consegue te salvar aqui. O `ProbeResult` protege valores que o seu próprio código constrói. A config nunca foi construída pelo seu código. Ela foi lida do disco, `JSON.parse`ada virando uma papa com formato de `any`, e então em algum lugar existe uma linha assim:

```ts
const config = JSON.parse(readFileSync(path, "utf8")) as FleetConfig;
```

Esse `as FleetConfig` é o bug. Eu já entreguei exatamente essa linha, mais vezes do que quero contar, e ela sempre parece segura porque o editor autocompleta lindamente depois. Mas uma asserção de tipo é um comentário que o compilador é obrigado a acreditar. Os tipos do TypeScript são apagados em tempo de execução; a asserção não checa nada, não converte nada, não protege nada. É uma promessa que ninguém checa, e o lixo de runtime passa direto pelo compilador usando o crachá do seu tipo.

A bala de prata? Um parser.

A distinção tem um nome, e ele é o título desta lição: parse, don't validate — faça parse, não valide. Um **validador** abençoa os dados no lugar: ele olha para um valor, talvez lance uma exceção, e te devolve a mesma coisa sem tipo que você entregou, mais uma sensação boa. Um **parser** é uma função que ou retorna o valor tipado ou recusa. Depois que um parser roda, o tipo é VERDADEIRO, não uma asserção. É esse o padrão inteiro: faça parse onde os dados entram, confie nos tipos depois.

![Um arquivo de config flui por uma asserção não checada até virar um bug em tempo de execução, enquanto o caminho do parser se bifurca ou em um valor tipado ou em uma recusa barulhenta.](assets/v01-flowchart.webp)

### zod v4, o subconjunto de trabalho

Você poderia escrever parsers à mão, e em Rust mais adiante neste curso você efetivamente vai. Em TypeScript o ecossistema já resolveu a questão. Instale o zod:

```bash
npm i zod@4.5.4
```

Esse dígito é o último release em 2026-09-02; re-cheque com `npm view zod version` antes de fixar. A prosa deste curso diz "zod v4" e qualquer 4.x que você instalar hoje vai rodar este lab.

Um schema zod é um valor que descreve um formato e sabe como checar ele. Aqui está o schema de config da frota, inteiro, porque você vai construir ele no lab e eu quero que você tenha visto o destino primeiro:

```ts
import { z } from "zod";

export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine((t) => t.timeoutMs < t.intervalSecs * 1000, {
    message: "timeoutMs must be under the probe interval, or probes pile up on a slow target",
    path: ["timeoutMs"],
  });

export const configSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(targetSchema).min(1),
});
```

Leia de cima para baixo. `z.strictObject` descreve um objeto e rejeita chaves que ele não conhece, que é exatamente o que pega o typo `intervalSeconds`: uma chave desconhecida é um erro, não um dar de ombros. `z.url()` e `z.number().int().positive()` empurram para dentro do schema regras que de outra forma viveriam como `if` espalhados em cinco consumidores. E `.refine` é a saída de emergência entre campos: qualquer predicado sobre o objeto inteiro, com uma mensagem que você escreve e um `path` para que o erro caia no campo em que um humano olharia. Repare nas unidades fazendo trabalho de verdade nesse refinamento: o intervalo está em segundos, o timeout em milissegundos, então a comparação multiplica por 1000. Regras entre campos são precisamente onde a validação feita à mão apodrece primeiro, porque nenhum campo sozinho é dono delas.

### Error maps: escreva a mensagem para o operador às 2 da manhã

As mensagens default do zod são corretas e levemente robóticas: "Invalid input: expected number, received undefined" te diz o que a máquina viu, não o que o humano deveria fazer a respeito. Numa fronteira interna isso está ok, ninguém lê essas coisas. Numa fronteira de CLI o texto do erro É a interface de usuário, e o zod v4 deixa você substituir qualquer mensagem no ponto em que a regra é declarada, com um parâmetro `error`:

```ts
const s = z.strictObject({
  url: z.url({ error: "url must be a full URL, scheme included (https://...)" }),
  intervalSecs: z.number({ error: "intervalSecs is the probe cadence in SECONDS, as a number" })
    .int()
    .positive({ error: "intervalSecs must be a positive number of seconds" }),
});
```

Alimente isso com uma config com `"url": "example.com"` e `"intervalSecs": -5` e a árvore impressa lê como se um colega tivesse escrito:

```
✖ url must be a full URL, scheme included (https://...)
  → at url
✖ intervalSecs must be a positive number of seconds
  → at intervalSecs
```

A regra da casa que eu uso para essas: uma boa mensagem de fronteira nomeia a unidade, a restrição e, quando dá, o PORQUÊ, porque a pessoa lendo ela está editando um arquivo de config sob pressão de tempo e tem zero interesse no seu sistema de tipos. Dobre erros customizados para dentro do `targetSchema` do lab onde uma mensagem default deixaria o operador adivinhando; os critérios de aceitação não dependem deles, o seu eu do futuro depende. Existe um tier inteiro a mais dessa maquinaria (error maps por schema, localização) de que a frota não precisa; se um dia você entregar um produto onde erros de validação chegam aos usuários finais, esse é o momento de ir ler sobre isso.

Dois jeitos de rodar um schema, e a diferença importa numa fronteira de CLI:

![Cartões lado a lado contrastam parse, que lança na falha, com safeParse, que retorna um objeto de resultado que quem chama tem que inspecionar e tratar.](assets/v02-comparison.webp)

A inicialização da frota quer `safeParse`. Um typo de config não é uma exceção na sua lógica, é o erro do operador, e a coisa mais gentil que uma CLI pode fazer é imprimir exatamente o que está errado e sair com código diferente de zero para que o cron marque a execução em vermelho. Você vai ligar isso no lab com `z.prettifyError`, que transforma o erro do zod na árvore legível com que um humano conserta um arquivo de config.

### O schema é de onde o tipo vem

Agora a jogada que torna esse padrão sistemático em vez de apenas organizado. Você não escreve uma interface `FleetConfig` do lado do schema. Você deriva ela:

```ts
export type FleetConfig = z.infer<typeof configSchema>;
```

`z.infer` lê o tipo estático de dentro do schema de runtime. Uma fonte de verdade. Isso é, com honestidade, uma mão na roda, e aqui está o bug de descompasso que ela mata. Suponha que o schema e uma interface escrita à mão vivam lado a lado. Alguém renomeia `intervalSecs` para `intervalMs` na interface durante um refactor, atualiza todo consumidor que o compilador aponta, entrega. O schema continua validando o nome ANTIGO do campo. Agora configs válidas falham na validação, ou pior, a interface afirma um campo que o validador nunca checa. Duas fontes de verdade não entram em descompasso porque o seu time é desleixado; elas entram em descompasso porque são duas, e toda renomeação é uma moeda no ar sobre qual das duas vai ser atualizada. Com `z.infer` não existe moeda. Renomeie o campo no schema e todo consumidor do tipo fica vermelho na hora, porque o tipo É o schema. Você vai rodar esse treino de propósito no lab e ver os erros caírem em cascata.

![O schema zod no centro alimenta um tipo TypeScript derivado para cima até todos os consumidores e dados de runtime validados para baixo, substituindo uma interface escrita à mão separada.](assets/v03-diagram.webp)

Vale um instante de história, porque essa ideia é maior que esta biblioteca. O zod entregou a v3 em 2021-05-17 e levou quatro anos para entregar um major, a v4 em 2025-07-09. Nessa janela, "parse, don't validate" saiu de slogan de post de blog para a cultura de fronteira de um ecossistema inteiro, o equivalente a 274.7M de downloads semanais. A ideia cresceu para além da biblioteca. Você está aprendendo a ideia; o zod é só a melhor ferramenta atual para ela em TypeScript, e quando este curso chegar em Rust você vai encontrar a mesma disciplina rodando em tempo de compilação com serde.

![Uma linha do tempo vai do lançamento do zod 3 em 2021 e atravessa quatro anos de adoção do ecossistema até o zod 4 em julho de 2025 e 274.7 milhões de downloads semanais hoje.](assets/v04-timeline.webp)

### Genéricos, na hora certa

Este é o momento na hora certa que o módulo prometeu: os genéricos chegam aqui, exatamente quando você precisa deles, porque você vem consumindo eles há três parágrafos sem um nome.

Olhe de novo para o que você escreveu. `z.array(targetSchema)`: você entregou um valor com formato de tipo para uma função e recebeu de volta um schema para arrays DAQUELE formato. `z.infer<typeof configSchema>`: você aplicou uma função em nível de tipo a um tipo e saiu um tipo novo. Os dois são APIs genéricas, e nos dois casos você CONSUMIU o parâmetro de tipo que outra pessoa declarou. Aqui está a proporção honesta que ninguém põe na embalagem: ler e aplicar os genéricos de outra pessoa é mais ou menos 90% dos genéricos que um desenvolvedor em atividade toca. `Array<string>`, `Promise<Response>`, `Map<string, ProbeResult>`, `z.infer<typeof T>`. Você faz isso desde o módulo um, toda vez que `await fetch(...)` te entregou uma `Promise<Response>` e o compilador sabia o que saía do `await`.

O modelo mental que torna assinaturas genéricas legíveis na documentação de qualquer biblioteca: um parâmetro de tipo é um argumento de função que por acaso é um tipo. `Array<T>` é uma fábrica que recebe um tipo e retorna um tipo de array; `z.ZodType<T>` recebe o tipo de saída e retorna "um schema que produz aquilo". Quando uma assinatura parece intimidadora, leia ela do jeito que você lê uma chamada de função: ache o que entra, ache onde volta a sair, ignore a maquinaria no meio. Essa habilidade de leitura, não a de autoria, é o que te destrava em bases de código de verdade.

Os outros 10% são autorar os seus próprios, e esta lição precisa de exatamente um, porque a frota agora tem duas fronteiras fazendo a mesma dança: ler dados crus, dar safeParse neles, imprimir a árvore e morrer na falha, retornar o valor tipado no sucesso. Duas vezes é um padrão:

```ts
export function parseOrExit<T>(schema: z.ZodType<T>, raw: unknown): T {
  const result = schema.safeParse(raw);
  if (!result.success) {
    console.error("boundary refused this input:");
    console.error(z.prettifyError(result.error));
    process.exit(1);
  }
  return result.data;
}
```

Leia a assinatura devagar, ela é a lição de genéricos inteira. `<T>` declara uma variável de tipo. `schema: z.ZodType<T>` diz "um schema que produz T", e `raw: unknown` é a disciplina da lição passada segurando a linha: a entrada é intocável até que se prove. O tipo de retorno `T` fecha o loop: o que quer que o schema produza, quem chama recebe, totalmente tipado. Chame ele com `configSchema` e `T` vira `FleetConfig`; chame com um schema de saldo mais tarde e `T` vira aquilo. Um helper, as duas fronteiras, zero casts. Essa também é a promessa de encerramento da lição passada resgatada: o `parseProbe` protegia um formato de par feito à mão, e o `parseOrExit` mais um schema é aquela mesma trava de fronteira generalizada para qualquer fronteira que você consiga descrever, que é no que "travar a fronteira inteira" acaba dando. Repare no que NÃO fizemos: nada de torres de `<T extends ...>`, nada de tipos condicionais, nada de truques espertos de inferência. Assinaturas genéricas elaboradas são uma habilidade que você pode adquirir quando uma biblioteca te obrigar; hoje você aprende o formato que você vai de fato usar toda semana.

### `satisfies`: checado, não alargado

Mais uma ferramenta e a teoria acaba. A frota quer uma config default no código, para dev local quando nenhum arquivo é passado. Três jeitos de escrever isso:

```ts
// 1. No annotation: narrow types, zero shape checking. A typo'd key sails through
//    until something consumes it.
export const defaultConfig = { ... };

// 2. Annotation: shape checked, but WIDENED. fleetName is now just `string`;
//    the compiler forgot what you wrote.
export const defaultConfig: FleetConfig = { ... };

// 3. satisfies: shape checked AND every field keeps its narrow literal type.
export const defaultConfig = {
  fleetName: "pulse-dev",
  targets: [
    { name: "local", url: "http://localhost:3000/health", intervalSecs: 30, timeoutMs: 2000 },
  ],
} satisfies FleetConfig;
```

`satisfies` é checagem sem alargamento. O compilador verifica que o literal está de acordo com `FleetConfig`, exatamente como a anotação faria, mas o tipo inferido do próprio valor sobrevive: passe o mouse em `defaultConfig.fleetName` e você vê o literal `"pulse-dev"`, não `string`. Com a anotação você leva o pior trade-off possível numa constante: você queria precisão E a checagem, e ela vendeu a precisão sem avisar. Por que a frota se importa: código a jusante pode ramificar em `cfg.fleetName === "pulse-dev"` com o compilador rastreando o valor exato, e uma chave com typo no default continua falhando na compilação, o que a opção 1 teria deixado passar até algum consumidor tropeçar nela em produção. Para ser claro sobre o que `satisfies` não é: ele é puramente em tempo de compilação. Ele não roda nada, não refina nada em tempo de execução, nunca chama o seu `.refine`. O schema protege o arquivo no disco; o `satisfies` protege o literal no seu código-fonte. Fronteiras diferentes, ferramentas diferentes, e a frota agora usa as duas no mesmo formato.

![Três versões do mesmo literal de config mostram nenhuma anotação como não checada, uma anotação como checada mas alargada, e satisfies como checada mantendo os tipos estreitos.](assets/v05-annotated-code.webp)

**Vá mais fundo (os 20%).** esta lição te ensinou a disciplina de fronteira e os genéricos que você consome todo dia. O resto da superfície do zod (transforms, brands, refinements assíncronos) e a arte de autorar assinaturas genéricas elaboradas ficam deliberadamente como bookmark. Quando você quiser: o tutorial gratuito de Zod do Total TypeScript (totaltypescript.com/tutorials, 10 exercícios, gratuito no momento em que escrevo) é o melhor conjunto de treinos sobre a biblioteca, e o capítulo de Generics do TypeScript Handbook (typescriptlang.org/docs/handbook/2/generics.html) é o tratamento canônico sobre autoria. Faça o tutorial depois deste módulo, não no lugar do lab.

## Lab: a fronteira que recusa

O recuo da ajuda, dito em voz alta: o passo 1 é totalmente resolvido, você digita junto e eu explico cada linha. Os passos 2 e 3 são completions, eu te entrego um esqueleto e você autora a parte estrutural. O passo 4 é um treino guiado onde o compilador faz o ensino. Nada de challenge sem guia nesta lição; os challenges de código do módulo ficam nas lições dos dois lados desta, então o lab e o quiz carregam a avaliação aqui.

Você está trabalhando no repo da frota do módulo um. Node 24 LTS é o pressuposto (esse é o LTS ativo hoje; o Node 26 assume a linha LTS em 2026-10-28, e nada neste lab muda com isso). `tsx` e `typescript` são dependências de dev desde o build do v0; se você está chegando agora:

```bash
npm i -D tsx typescript @types/node
npm i zod@4.5.4
```

### 1. Faça o schema da config, ligue a fronteira, mate o bug da abertura (resolvido)

Crie `src/config.ts` com o schema que você viu na seção de teoria, mais o tipo derivado, o default e o helper. Arquivo completo, nada elidido:

```ts
import { z } from "zod";

export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine((t) => t.timeoutMs < t.intervalSecs * 1000, {
    message: "timeoutMs must be under the probe interval, or probes pile up on a slow target",
    path: ["timeoutMs"],
  });

export const configSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(targetSchema).min(1),
});

export type FleetConfig = z.infer<typeof configSchema>;

export const defaultConfig = {
  fleetName: "pulse-dev",
  targets: [
    { name: "local", url: "http://localhost:3000/health", intervalSecs: 30, timeoutMs: 2000 },
  ],
} satisfies FleetConfig;

export function parseOrExit<T>(schema: z.ZodType<T>, raw: unknown): T {
  const result = schema.safeParse(raw);
  if (!result.success) {
    console.error("boundary refused this input:");
    console.error(z.prettifyError(result.error));
    process.exit(1);
  }
  return result.data;
}
```

Duas linhas merecem comentário. `path: ["timeoutMs"]` mira o erro do refinamento no campo que um operador de fato editaria; sem isso a mensagem cai no objeto alvo inteiro, o que é tecnicamente verdade e praticamente inútil. E `process.exit(1)` dentro do `parseOrExit` é o que torna o helper honesto sobre o próprio nome: o tipo diz "retorna T", e o único jeito de isso ser sempre verdade é se o ramo de falha nunca retornar coisa nenhuma. O TypeScript entende isso porque `process.exit` retorna `never`, então o compilador prova que o ramo de sucesso é o único ramo que chega no `return`.

Agora o script de fronteira, `src/check-config.ts`:

```ts
import { readFileSync } from "node:fs";
import { configSchema, parseOrExit, type FleetConfig } from "./config.js";

const path = process.argv[2] ?? "pulse.config.json";
const raw: unknown = JSON.parse(readFileSync(path, "utf8"));

const config: FleetConfig = parseOrExit(configSchema, raw);

console.log(`fleet "${config.fleetName}": ${config.targets.length} target(s)`);
for (const t of config.targets) {
  console.log(`  ${t.name} -> ${t.url} every ${t.intervalSecs}s, timeout ${t.timeoutMs}ms`);
}
```

Dois detalhes antes de você rodar, os dois já custaram uma tarde a alguém. O import diz `./config.js` mesmo que o arquivo no disco seja `config.ts`; isso são as regras de resolução de ESM, onde os especificadores de import nomeiam o arquivo de SAÍDA, e o `tsx` resolve isso corretamente, então não "conserte" a extensão. E repare no tipo em `raw`: `unknown`, nunca `any`. Essa é a regra da lição passada encontrando a ferramenta desta lição; `JSON.parse` retorna `any`, e anotar o binding como `unknown` o desintoxica para que nada a jusante consiga tocar nele sem parse, o que quer dizer que o ÚNICO caminho daqui até uma config usável passa pelo parser. O compilador agora obriga o padrão que dá nome a esta lição. E um `pulse.config.json` de verdade na raiz do repo:

```json
{
  "fleetName": "pulse-prod",
  "targets": [
    {
      "name": "docs",
      "url": "https://example.com",
      "intervalSecs": 60,
      "timeoutMs": 3000
    },
    {
      "name": "api",
      "url": "https://example.org/health",
      "intervalSecs": 30,
      "timeoutMs": 2000
    }
  ]
}
```

Rode a fronteira:

```bash
npx tsx src/check-config.ts pulse.config.json
```

Você deve ver o resumo parseado e tipado:

```
fleet "pulse-prod": 2 target(s)
  docs -> https://example.com every 60s, timeout 3000ms
  api -> https://example.org/health every 30s, timeout 2000ms
```

Agora o momento pelo qual este lab existe. Copie a config para `pulse.config.broken.json` e cometa o typo da abertura de verdade: renomeie a chave `intervalSecs` do primeiro alvo para `intervalSeconds`. Rode a fronteira contra ele:

```bash
npx tsx src/check-config.ts pulse.config.broken.json
```

```
boundary refused this input:
✖ Unrecognized key: "intervalSeconds"
  → at targets[0]
✖ Invalid input: expected number, received undefined
  → at targets[0].intervalSecs
```

Código de saída 1. Compare isso com a execução do naive-load da abertura, porque esse é o antes/depois que importa: o mesmo arquivo em que o v0 rodou verde agora não consegue entrar no programa. A falha não se moveu para um lugar mais agradável; ela deixou de existir em tempo de execução e virou uma recusa na inicialização com o campo exato nomeado duas vezes, uma como a chave desconhecida que você escreveu e uma como a chave obrigatória que você deixou faminta. A minha execução quebrada imprimiu exatamente esses dois erros e mais nada, que é a outra coisa que uma boa árvore de erros te compra: sem rolagem, sem arqueologia de stack trace, só o conserto.

![A inicialização da frota flui do cron pela leitura do arquivo e pelo parse até o parseOrExit, que ou admite uma config tipada no loop de sondagem ou sai em vermelho para o operador.](assets/v06-flowchart.webp)

### 2. O refinamento entre campos (completion)

Sua vez de autorar a regra. Delete o `.refine` do `targetSchema` e reconstrua ele você mesmo a partir deste esqueleto:

```ts
export const targetSchema = z
  .strictObject({
    name: z.string().min(1),
    url: z.url(),
    intervalSecs: z.number().int().positive(),
    timeoutMs: z.number().int().positive(),
  })
  .refine(
    (t) => /* your predicate: the timeout budget must fit inside the probe interval */,
    {
      message: /* your message: say WHY, not just what */,
      path: [/* aim it at the field the operator should edit */],
    },
  );
```

Cuidado com as unidades; o intervalo é em segundos e o timeout em milissegundos, então o predicado honesto é `t.timeoutMs < t.intervalSecs * 1000`. Um predicado com unidades misturadas é exatamente o tipo de regra que nunca sobrevive como conhecimento tribal, que é por isso que ela mora no schema e não num comentário de code review. Depois prove que funciona. Faça uma cópia da config boa com um alvo ajustado para `"intervalSecs": 2, "timeoutMs": 5000` e rode o verificador contra ela:

```
boundary refused this input:
✖ timeoutMs must be under the probe interval, or probes pile up on a slow target
  → at targets[0].timeoutMs
```

Aceitação: a execução sai com código diferente de zero e o erro cai em `targets[0].timeoutMs` com a sua mensagem, como a saída acima. Se a sua mensagem só reafirma a conta, reescreva ela; daqui a seis meses o operador que estiver lendo não vai lembrar por que a regra existe, e "probes pile up on a slow target" é a diferença entre um conserto e uma gambiarra.

### 3. A fronteira com cara de blockchain: getBalance como bigint (completion)

Agora a segunda fronteira, e o motivo de este curso estar indo à deriva na direção da Solana. A frota vai acabar vigiando infraestrutura de blockchain, então a primeira leitura de blockchain dela acontece aqui, sem SDK, porque uma chamada JSON-RPC é só um POST e você já é dono de uma disciplina de parser.

Uma frase para te orientar e nada mais: na Solana, saldos moram em contas e são denominados em lamports, uma contagem inteira da menor unidade da blockchain, e tudo o que for mais fundo sobre o que uma conta É pertence ao curso de evolução do Bitcoin à Solana, que percorre esse modelo de ponta a ponta. A chamada em si é o formato JSON-RPC que você imaginaria: faça POST de um nome de método e params, receba um result de volta. Aqui está um corpo de resposta de verdade do endpoint que você está prestes a acessar, capturado enquanto eu escrevia esta lição:

```json
{"jsonrpc":"2.0","result":{"context":{"apiVersion":"4.2.1","slot":443610065},"value":1},"id":1}
```

Esse `"value":1` é o saldo em lamports, e ele chega como um número JSON pelado, o que nos traz ao que de fato importa neste passo: o tipo daquele inteiro. Saldos de lamport são u64 no fio: um inteiro sem sinal de 64 bits cujo máximo é 18446744073709551615. Números JavaScript são doubles, exatos só até `Number.MAX_SAFE_INTEGER`, que é 9007199254740991. Qualquer u64 além disso arredonda em silêncio. Um saldo de um lamport atravessa o `JSON.parse` intocado; o saldo de uma baleia não tem essa obrigação. Rode a mentira você mesmo, uma linha:

```bash
node -e "console.log(JSON.parse('{\"value\":9007199254740993}').value)"
```

```
9007199254740992
```

Errado por um, sem erro, sem aviso, e isso aconteceu dentro do `JSON.parse` antes que qualquer schema pudesse olhar para o valor. Um saldo errado por alguns lamports sem erro nenhum em lugar nenhum é a mentira mais educada deste curso até aqui. Então o conserto não pode ser "validar o número depois"; o dano é anterior à validação. O conserto é interceptar o texto cru antes que ele vire um double.

![Uma reta numérica marca o limite de inteiro seguro do JavaScript, o primeiro valor que arredonda em silêncio, e o máximo bem maior de u64 que saldos de lamport podem alcançar.](assets/v07-chart.webp)

O Node 21 e posteriores dá o ponto de interceptação: o `JSON.parse` passa ao seu reviver um objeto de contexto carregando o texto-fonte cru de cada primitivo, então você consegue construir um `BigInt` a partir dos dígitos antes que o double sequer exista. Os tipos que vêm com o TypeScript ainda não alcançaram esse terceiro argumento do reviver, então o arquivo faz a ponte com um único alias tipado, o que é em si uma pequena lição honesta: runtimes entregam antes dos tipos. Crie `src/balance.ts` a partir deste esqueleto e complete as duas partes marcadas:

```ts
import { z } from "zod";
import { parseOrExit } from "./config.js";

// Plain z.object here, not strictObject, on purpose: this boundary reads
// someone ELSE's shape, and the RPC server may add fields (apiVersion already
// rides along) without that being your bug. Strictness is for shapes you own,
// like the config; tolerance of unknown keys is for shapes you only consume.
const balanceResponseSchema = z.object({
  jsonrpc: z.literal("2.0"),
  id: z.number(),
  result: z.object({
    context: z.object({ slot: z.number() }),
    // YOUR SCHEMA (a): the balance field. It must come out as bigint, not number.
  }),
});

const RPC_URL = "https://api.mainnet.solana.com";
const address = process.argv[2] ?? "Vote111111111111111111111111111111111111111";

const res = await fetch(RPC_URL, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "getBalance",
    params: [address],
  }),
});

const text = await res.text();

// Node 21+ passes a { source } context to the reviver; TypeScript's lib types
// have not caught up yet, so bridge the gap with one typed alias.
type ReviverWithSource = (
  this: unknown,
  key: string,
  value: unknown,
  context?: { source?: string },
) => unknown;

const parseWithSource = JSON.parse as (text: string, reviver?: ReviverWithSource) => unknown;

const raw: unknown = parseWithSource(text, (key, value, ctx) =>
  /* YOUR REVIVER (b): when the key is "value" and ctx.source exists,
     build the BigInt from ctx.source; otherwise return value unchanged */
);

const body = parseOrExit(balanceResponseSchema, raw);

console.log(`slot ${body.result.context.slot}`);
console.log(`balance: ${body.result.value} lamports (${typeof body.result.value})`);
```

Para (a) a resposta é uma linha, `value: z.bigint()`, e ela está fazendo mais do que parece: se o seu reviver parar de rodar algum dia, o schema falha fazendo barulho em vez de deixar um double arredondado se passar por um saldo. Para (b): `key === "value" && ctx?.source !== undefined ? BigInt(ctx.source) : value`. O reviver vê toda chave chamada `value` no documento; aqui só o saldo casa, e a trava em `ctx?.source` te mantém honesto porque o contexto só carrega texto-fonte para primitivos.

Rode exatamente uma vez:

```bash
npx tsx src/balance.ts
```

```
slot 443609276
balance: 1 lamports (bigint)
```

O seu slot vai diferir; o saldo do vote program genuinamente é 1 lamport, e a palavra entre parênteses é a checagem de aceitação: `bigint`. Alimente ele com um endereço mais movimentado como primeiro argumento se você quiser um número grande, mas repare no uma vez: `https://api.mainnet.solana.com` é o endpoint público, ele tem rate limit, e a documentação da Solana diz sem rodeios que ele não é destinado a aplicações de produção. Um fetch neste lab é uma visita de cortesia. Cinquenta alvos num cron é ban, e o formato desse limite é precisamente onde a próxima lição pega o fio.

### 4. O treino do descompasso (descoberta guiada)

Último passo, e o ponto do `z.infer` tornado físico. Em `src/config.ts`, renomeie o campo do schema `intervalSecs` para `intervalMs`. Não mude mais nada. Agora rode o compilador sobre o projeto:

```bash
npx tsc --noEmit
```

Observe a cascata: o `check-config.ts` fica vermelho onde imprime `t.intervalSecs`, o refinamento fica vermelho dentro do próprio schema, o `defaultConfig` fica vermelho debaixo do `satisfies` dele. Todo consumidor do tipo soube da renomeação instantaneamente, porque existe exatamente um lugar de onde o tipo vem. Esse é o bug de descompasso da seção de teoria rodando ao contrário: com duas fontes de verdade essa renomeação teria sido uma divergência silenciosa; com uma fonte ela é uma checklist escrita pelo compilador de todo ponto que precisa decidir. Reverta a renomeação, rode `npx tsc --noEmit` de novo, confirme verde.

**Verifique antes de seguir em frente**: `npx tsx src/check-config.ts pulse.config.json` imprime o resumo tipado de dois alvos e sai com 0. O mesmo comando contra `pulse.config.broken.json` imprime a árvore de erros nomeando `intervalSeconds` e `intervalSecs` e sai com código diferente de zero. A sua cópia com timeout ruim é recusada com a sua mensagem de refinamento em `targets[0].timeoutMs`. E `npx tsx src/balance.ts` imprime uma linha de lamports terminando em `(bigint)`.

## Challenge

A frota publica `status.json` a cada execução do cron; você construiu esse arquivo no módulo um e vem confiando na sua própria saída desde então. Pare. Escreva `src/check-status.ts`: um schema zod para `status.json` do jeito que a sua frota de fato escreve ele, um tipo `StatusReport` derivado com `z.infer`, e um parse do arquivo pelo mesmo helper `parseOrExit`, imprimindo uma linha de resumo por alvo no sucesso. Restrições: o schema tem que ser strict, pelo menos um campo precisa de uma regra mais apertada que o tipo primitivo dele (um formato de timestamp via `z.iso.datetime()`, o único validador aqui que a lição não ensinou, então essa opção te custa uma consulta à documentação; um array não vazio; uma latência que não pode ser negativa), e nenhum helper novo; o `parseOrExit` foi escrito genérico precisamente para que esta terceira fronteira te custe zero encanamento novo.

```bash
npx tsx src/check-status.ts status.json
```

Aceitação: o seu `status.json` real atual parseia limpo, e editar um campo à mão até virar lixo faz ele ser recusado com um erro legível, no nível do campo. Ele roda inteiramente local, então nenhum RPC envolvido. Se o schema brigar com você porque o seu próprio formato de saída é inconsistente entre execuções, parabéns: o parser acabou de achar um bug de verdade, e consertar o escritor faz parte do challenge.

## Onde a fronteira termina

Hora de ser franco sobre o que você comprou e o que custou. Um parser em toda fronteira custa uma dependência, um schema para manter junto de toda mudança de config, e trabalho na inicialização, e as árvores de erro do zod conseguem genuinamente sobrecarregar quando os schemas aninham fundo: uma falha quatro níveis abaixo numa união aninhada imprime uma árvore que dá trabalho ler, que é por isso que a frota mantém a config dela rasa e as mensagens escritas à mão. A disciplina também tem um limite, e saber onde ela para importa tanto quanto adotá-la. Faça parse nas FRONTEIRAS, os lugares onde os dados entram de fora do seu sistema de tipos: um arquivo de config no disco, uma resposta HTTP, o ambiente (quando a frota ganhar segredos nos módulos de deploy, o `process.env` ganha um schema também, e pelo mesmo motivo). Em nenhum outro lugar. Funções internas passando umas às outras valores parseados pelo schema deveriam confiar nos tipos delas; revalidar entre as suas próprias funções é ruído que diz que você não acredita no seu próprio compilador, e se isso for verdade os tipos eram inúteis. E um parse que passou prova formato, nunca verdade. Uma config bem formada ainda pode apontar sondas para a URL errada; uma resposta RPC bem formada ainda pode estar velha no momento em que você age sobre ela; o schema não tem como saber a sua intenção, só a sua estrutura. Parsear te compra exatamente uma frase: "estes dados são o formato sobre o qual eu raciocinei". Essa por acaso é a única frase de que o compilador precisava para tornar real toda garantia a jusante.

A sua vitória de trinta segundos, diga em voz alta antes de fechar a aba: um validador abençoa os dados no lugar; um parser RETORNA o valor tipado, então depois que ele roda o tipo é verdadeiro por construção. Se você consegue dizer isso e apontar para a linha no `parseOrExit` onde isso acontece, você tem esta lição.

Se alguma coisa no lab resistiu, ou se o truque do reviver pareceu merecer um porquê mais fundo, me conte: esse feedback orienta onde o curso gasta a profundidade dele, e as lições de fronteira são as que eu mais quero afinadas com onde as pessoas de fato escorregam. As suas fronteiras recusam lixo agora. Então aponte a frota para cinquenta alvos de verdade de uma vez, e descubra que a internet recusa VOCÊ: rate limits, sockets travados, e uma parede de 429s. A próxima lição é async que sobrevive ao contato: concorrência como orçamento, backoff com jitter, e cancelamento. Leve um rate limit na bagagem.
