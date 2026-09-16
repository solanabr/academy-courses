# Node 24, TypeScript 7, e sua primeira sonda

Na lição passada você rodou duas sondas sem instalar uma única coisa: um one-liner de `fetch` no console do devtools contra o rust-lang.org, e um snippet do Playground onde o compilador pegou um bug antes de o código sequer rodar. As duas funcionaram. As duas também já foram. Feche a aba e nenhuma das duas sondas jamais existiu: nenhum arquivo, nenhum histórico, nenhum jeito de rodar de novo amanhã e comparar números.

Esse é o problema com ambientes emprestados. Medição de verdade precisa de uma casa: um runtime que é seu, um compilador que pega o bug ANTES de a sonda mentir para você, e um arquivo num diretório que você pode commitar. Dez minutos a partir de agora essa casa existe, e o `pulse` v0 imprime a primeira latência dele.

Então vamos checar em que você está pisando. Abra um terminal e rode:

```bash
node --version
```

Se isso imprimir `v24.x.x`, você já está em casa. Se não imprimir nada, ou algo mais antigo, a próxima seção resolve em dois minutos. De qualquer jeito, mantenha o terminal aberto — tudo daqui para frente acontece nele.

E assim que a sonda rodar, a gente vai falar de uma coisa genuinamente estranha: o compilador que você está a ponto de instalar foi ele mesmo reescrito em outra linguagem para um speedup de 10x. A tese inteira deste curso, TypeScript para as superfícies e uma linguagem de sistemas para os hot paths, está rodando na sua própria máquina antes de você ter escrito cinquenta linhas.

## Resumo

- Você instala o Node 24 LTS e o TypeScript 7.0.2, e entrega o `pulse` v0: uma sonda que busca uma URL e imprime a latência dela, rodada com tsx, dentro dos primeiros dez minutos.
- Você percorre as ~5 flags estritas de tsconfig em que o código deste curso vai realmente tropeçar, cada uma com o erro exato de compilador que ela lança, e marca o resto.
- Você recebe a história honesta do TypeScript 7: o que o compilador nativo comprou, o que ele custa hoje, e por que repos de produção ainda fixam 5.x.
- O lab planta um bug de verdade de propósito e deixa o compilador pegar ele. O challenge te manda para multi-URL, solo.

## O toolchain de dez minutos

### Node 24 LTS, não "o mais novo"

Por que 24 e não qualquer número que seja o maior na página de download? Porque o Node sai em duas trilhas. Majors de número par são promovidos a LTS, long-term support: eles recebem correções por anos e são o que servidores de produção rodam de verdade. Majors de número ímpar são experimentos com prazo curto de validade, um lugar para o projeto testar mudanças antes de uma linha LTS herdar elas. "O mais novo" é um beta rolante; LTS é o chão em que você constrói.

Isso não é curiosidade sobre o Node, é um hábito que você está formando para todo runtime e todo toolchain deste curso. Quando algo de que você fez deploy quebra às 3 da manhã, você quer estar na linha que recebe patches de segurança por anos, contra a qual as plataformas de nuvem testam, que todo pacote na sua árvore de dependências afirma suportar. Essa linha tem um nome e um calendário publicado, e conferir o calendário antes de instalar é um ato de trinta segundos que poupa dor de verdade. "Maintenance" nesse calendário, aliás, não é morte: um LTS em maintenance ainda recebe correções críticas, ele só para de ganhar features novas, o que para um servidor normalmente é exatamente o que você quer.

Agora a linha Active LTS é o Node 24, codinome "Krypton". Uma nota de rodapé com data, porque esta passagem de bastão está agendada: o Node 26 se torna o novo Active LTS em 2026-10-28, com o Node 24 escorregando para maintenance uma semana antes, em 2026-10-20, conforme o calendário de releases publicado (ainda seguro, ainda com patches até 2028-04-30, só não mais a manchete). Em 2026-09-02, o Node 24 LTS é o install correto, e tudo neste curso roda sem mudança no 26 quando você fizer o upgrade depois.

![Uma linha do tempo em que o Node 24 mantém o Active LTS até hoje, entra em maintenance no dia 20, outubro de 2026, e passa o título de Active LTS para o Node 26 no dia 28, outubro de 2026.](assets/v01-timeline.webp)

Instale ele pelo nodejs.org (escolha o botão LTS, ele aponta para o 24) ou, se você já usa um gerenciador de versões como o nvm, `nvm install 24`. Então confira:

```bash
node --version
# v24.x.x
```

Esse binário único entrega mais que um motor JavaScript. `fetch` vem embutido. `performance.now()` vem embutido. Existe até uma flag `node --env-file` para carregar variáveis de ambiente nativamente, sem precisar de pacote; guarde esse nome, ele ganha o momento dele quando a gente chegar em higiene de secrets mais adiante no curso. A sonda que você está a ponto de escrever usa zero dependências para o trabalho real dela. Tudo que você instala em seguida é para os *tipos*, não para o runtime.

### TypeScript 7.0.2 e tsx

Transforme a sonda num projeto de verdade:

```bash
mkdir pulse-station && cd pulse-station
npm init -y
npm pkg set type=module
```

Três comandos, uma decisão que vale nomear. O nome do diretório é `pulse-station`, não `pulse`, porque ele é a estação inteira do diagrama de m01-l1 e não só a sonda de hoje: o repo do GitHub na próxima lição assume esse nome, o `npm init -y` estampa ele no `package.json`, e os dois voltam mais tarde (o painel busca `https://raw.githubusercontent.com/YOUR_USER/pulse-station/main/status.json`, e m03-l1 passa o nome adiante para a raiz do workspace). O `npm init -y` escreve um `package.json`, o arquivo que faz deste diretório um projeto que o npm entende. O `npm pkg set type=module` declara que os arquivos aqui são ES modules, a variante moderna de import/export. Por que isso importa ganha um parágrafo mais adiante; por agora é uma caixinha que a gente marca para o `await` de nível superior funcionar.

Agora o toolchain:

```bash
npm i -D typescript@7.0.2 tsx@4.23.13 @types/node@24
```

Versões conferidas contra o registro do npm em 2026-09-02; `latest` para typescript resolve para exatamente 7.0.2 hoje. Uma nota cosmética antes da armadilha: o npm 11 pode imprimir algumas linhas `npm warn install-scripts esbuild...` durante este install. Isso é a trava de install-script do npm sendo tagarela sobre uma dependência que ele escolheu não rodar, não uma falha; se o comando sair sem uma linha `npm error`, o toolchain aterrissou bem. E esse dígito merece uma caixa de aviso própria, porque é uma armadilha genuína: **não existe typescript 7.0.0 estável no registro.** A GA chegou com correções de patch já incorporadas, então o 7.x estável começa, e atualmente termina, em 7.0.2. Um script de setup que diz `typescript@7.0.0` falha toda única vez que roda. A prosa pode dizer "TypeScript 7.0"; linhas de install têm que dizer 7.0.2.

Três pacotes, três funções:

- **typescript** te dá o `tsc`, o verificador. Ele lê seu código, aplica as regras de tipos, e te diz o que está errado. Neste curso a gente roda ele como `tsc --noEmit`: checa tudo, não emite nada.
- **tsx** é o runner. Ele executa um arquivo `.ts` direto, sem etapa de compilação que você tenha que ver. Ferramenta de dev-loop, e o jeito que o `pulse` roda o curso inteiro.
- **@types/node** ensina ao verificador como são os globais do próprio Node, então `process.argv` tem um tipo em vez de ser um mistério.

A divisão é a coisa a internalizar: o tsx roda seu código e não se importa com seus tipos; o tsc checa seus tipos e nunca roda seu código. Você precisa dos dois, e confundir os dois está atrás de metade da confusão "mas rodou bem!" em times de TypeScript. Um arquivo pode rodar perfeitamente no tsx carregando um erro de tipo que vai morder a próxima pessoa que chamar sua função de outro jeito, que é por isso que o lab te faz rodar os dois, toda vez, até o par ser memória muscular.

Mais uma coisinha, já que você vai digitar isso constantemente: o prefixo `npx` roda um binário do `node_modules` do seu próprio projeto em vez de caçar um install global. `npx tsc` é o *seu* 7.0.2, aquele que você fixou, não o que algum outro projeto deixou na sua máquina. Pins por projeto, invocados por projeto: essa disciplina é por que dois projetos com versões diferentes de TypeScript conseguem coexistir em paz num só laptop.

![Um arquivo-fonte flui pelo tsx até um programa em execução e separadamente pelo tsc até erros de tipo, com as definições de tipos do Node alimentando só o verificador.](assets/v02-diagram.webp)

### A primeira latência em um arquivo

Hora do retorno. Crie `smoke.ts`:

```typescript
const started = performance.now();
const res = await fetch("https://www.rust-lang.org");
const elapsed = performance.now() - started;
console.log(`${res.status} in ${elapsed.toFixed(1)}ms`);
```

Quatro linhas. `performance.now()` dá um timestamp de milissegundos de alta resolução; chame ele antes e depois do fetch e a diferença é sua latência. Por que não `Date.now()`, que você talvez conheça do JavaScript em outros lugares? Porque `Date.now()` lê o relógio de parede, e relógios de parede são ajustados: seu SO sincroniza a hora em segundo plano, e um relógio que salta no meio da medição pode te entregar uma latência negativa. `performance.now()` é monotônico, ele só anda para frente, que é a propriedade de que uma ferramenta de medição realmente precisa. Escolha pequena, mas o `pulse` é uma ferramenta de medição para o resto do curso, então ele começa no relógio certo. Rode:

```bash
npx tsx smoke.ts
# 200 in 254.5ms
```

Esse número é da minha rodada enquanto eu escrevia isto, e aqui vai um detalhe que merece sua atenção: minha primeira rodada de todas imprimiu 498.4ms, a segunda 254.5ms. Mesma URL, segundos de diferença, metade da latência. Cache de DNS, reúso de conexão, humor da rede. Uma amostra é ruído. Segure esse pensamento, porque transformar ruído em sinal é exatamente para onde a ferramentaria deste curso está indo, e é também o coding challenge desta lição.

Você acabou de fazer tudo que o console do devtools fez ontem, só que é um arquivo, num projeto, na sua máquina, e vai rodar igual amanhã. Agora vamos fazer o compilador merecer a cadeira dele.

### As cinco flags em que você vai realmente tropeçar

Rode o gerador de config:

```bash
npx tsc --init
```

Isso escreve o `tsconfig.json`, o livro de regras do verificador. E eu quero ser direto sobre método aqui, porque eu mesmo já fiz a versão errada disso: copiar uma config de strict mode de um post de blog de 2023 em vez de ler o que o `tsc --init` atual emite. Culpado, mais de uma vez. Configs velhas desativam silenciosamente exatamente as flags de que este curso depende. A saída do init É o cânone atual; ela é gerada pelo mesmo time que entrega o compilador, e ela muda conforme a linguagem muda. Leia a sua, não a de um blog.

Duas edições pequenas antes da caminhada, as duas sugeridas por comentários dentro do próprio arquivo gerado: defina `"types": ["node"]` (o padrão do init é uma lista vazia, que esconde o `process` do verificador) e descomente `"lib": ["esnext"]`. É isso. Zero config inventada neste curso; todo o resto fica exatamente como o init escreveu.

Agora, o arquivo liga muita coisa. Você não vai tropeçar na maior parte. Aqui estão as cinco flags com que o próprio código deste curso vai realmente colidir, cada uma com a colisão:

**1. `strict`** é o guarda-chuva: ela liga uma família de checagens, e a mais importante é "null e undefined são tipos reais que você tem que tratar". Sem ela, isso compila e quebra em tempo de execução:

```typescript
function firstChar(s: string | null): string {
  return s.charAt(0); // strict says: s might be null. Handle it.
}
```

Sob `strict`, isso é um erro até você checar `s` primeiro. Toda lição daqui para frente assume que ela está ligada.

**2. `noUncheckedIndexedAccess`** faz a indexação dizer a verdade. `process.argv[2]` não tem garantia nenhuma de existir; o usuário pode rodar sua sonda sem URL nenhuma. Então sob esta flag o tipo dele é `string | undefined`, não `string`, e passar ele direto para uma função que quer uma `string` é um erro:

```text
error TS2345: Argument of type 'string | undefined' is not
assignable to parameter of type 'string'.
```

Esse é um erro de compilador de verdade, e você vai tropeçar nele de propósito no lab. A classe de bug que ela deleta: um batch vazio, um argumento faltando, um índice off-by-one, cada um deles uma quebra em tempo de execução que agora não pode ser escrita.

**3. `exactOptionalPropertyTypes`** governa uma mentira sutil. Um campo opcional como `label?: string` quer dizer "pode estar ausente". Escrever `undefined` nele não é ausência; é presença com um buraco dentro, e código que itera chaves ou serializa para JSON trata os dois de forma diferente. A sonda vai ganhar um objeto de opções logo, então aqui vai a colisão em miniatura:

```typescript
type ProbeOptions = { label?: string };
const opts: ProbeOptions = { label: undefined };
```

```text
error TS2375: Type '{ label: undefined; }' is not assignable to
type 'ProbeOptions' with 'exactOptionalPropertyTypes: true'.
```

Se um campo é opcional, omita ele. Se ele pode genuinamente guardar undefined, diga isso no tipo. O bug que isso deleta é silencioso e cruel: `JSON.stringify` descarta campos ausentes mas um spread copia os `undefined`, então os dois objetos "vazios" se comportam de forma diferente no momento em que cruzam uma fronteira.

**4. `verbatimModuleSyntax`** mantém tipos e valores honestos na linha de import. Quando o `pulse` ganhar um `types.ts` numa lição mais adiante, este import parece inocente:

```typescript
import { ProbeResult } from "./types.js";
```

```text
error TS1484: 'ProbeResult' is a type and must be imported using
a type-only import when 'verbatimModuleSyntax' is enabled.
```

A correção é `import type { ProbeResult }`. Por que se importar? Tipos desaparecem em tempo de execução. Um import que carrega só um tipo tem que ser apagável, e esta flag garante que a saída compilada nunca entregue um import fantasma que quebra em tempo de execução.

**5. `module: "nodenext"`** alinha o verificador com o jeito que o Node realmente resolve módulos. O tropeço mais comum dela: imports relativos precisam do nome de arquivo completo, extensão inclusa, e a extensão é `.js` mesmo num arquivo `.ts`, porque é isso que existe depois da compilação:

```typescript
import { probe } from "./probe";
```

```text
error TS2835: Relative import paths need explicit file extensions
in ECMAScript imports when '--moduleResolution' is 'node16' or
'nodenext'. Did you mean './probe.js'?
```

O compilador até sugere a correção. Aceite. Esta aqui parece pedante exatamente uma vez, e depois você nota que seus imports agora querem dizer a mesma coisa para o verificador, para o Node, e para todo bundler rio abaixo, e a categoria inteira de "funciona em dev, quebra na resolução em prod" desaparece.

Cada uma dessas mensagens de erro é do meu terminal, não parafraseada. Esse é o padrão que este curso mantém: quando uma lição diz "o compilador pega isso", você recebe o texto de erro real, e você consegue reproduzir.

![Uma tabela emparelhando cada uma das cinco flags estritas com o que ela impõe e o bug exato da sonda que dispara ela.](assets/v03-comparison.webp)

E o resto do arquivo gerado? É real e vale saber, e não vale uma travessia flag por flag. Aqui vai a saída restante do init no 7.0.2 como uma linha de referência para cada, para você saber o que está carregando:

| Ajuste | Uma linha sobre por que ele está ali |
|---|---|
| `target: "esnext"` | emitir e checar contra o JavaScript atual; o runtime é moderno, aja como tal |
| `isolatedModules` | todo arquivo tem que ser traduzível sozinho, que é o que ferramentas rápidas por arquivo exigem |
| `moduleDetection: "force"` | trate todo arquivo como um módulo, sem scripts globais acidentais |
| `noUncheckedSideEffectImports` | um `import "./x"` pelado tem que apontar para algo que existe |
| `jsx: "react-jsx"` | como compilar JSX se aparecer algum; inerte aqui e inerte para o curso inteiro, porque o painel do módulo três chega do create-vite carregando o tsconfig próprio dele em vez de estender este |
| `skipLibCheck` | não re-checar os tipos dos arquivos de declaração das suas dependências em toda rodada |
| `sourceMap`, `declaration`, `declarationMap` | saídas para debuggers e consumidores de biblioteca; inertes até você emitir |

Nenhuma dessas vai interromper sua semana do jeito que as cinco acima vão. Quando uma delas te surpreender, a referência do Handbook explica melhor que uma frase aqui consegue, e essa é a divisão de trabalho certa.

Aqui vai a síntese, e é a franca: as flags são o code review que você não pode pular. Um revisor humano pega o bug do argumento faltando num dia bom, se ele não estiver cansado, se o diff não estiver enorme. Esse revisor roda em milissegundos, a cada save, para sempre, e nunca fica cansado. Rigor é atrito que você compra de propósito. As cinco flags vão te interromper o curso inteiro, e cada interrupção é um bug que nunca foi entregue.

### O compilador que ficou dez vezes mais rápido

Uma história antes do lab, porque você acabou de instalar o final dela.

Em março de 2025 a Microsoft anunciou "A 10x Faster TypeScript": o compilador do TypeScript, ele mesmo escrito em TypeScript por mais de uma década, estava sendo portado para Go. Dezesseis meses depois, em 2026-07-08, esse port chegou a GA como TypeScript 7. O benchmark de manchete: o build completo de checagem de tipos do VS Code caiu de 125.7s para 10.6s. O uso de memória caiu entre 6% e 26% nas bases de código testadas. (Você vai ver "~18% menos memória" citado em posts de segunda mão; esse ponto médio não aparece em lugar nenhum do anúncio de GA. O número real é a faixa. Eu aprendi a desconfiar de números suspeitosamente arredondados, e você também deveria.)

![Um gráfico de barras mostrando a checagem de tipos completa do VS Code caindo de 125.7 segundos no TypeScript 6 para 10.6 segundos no TypeScript 7.](assets/v04-chart.webp)

Sente com o que essa história implica, porque ela é a tese deste curso vestindo as release notes de outra pessoa. O time do TypeScript, as pessoas melhor posicionadas na Terra para deixar o TypeScript rápido, concluiu que o hot path do compilador pertencia a uma linguagem de sistemas. Não porque TypeScript é ruim; porque camadas diferentes têm físicas diferentes. TypeScript para as superfícies onde tipos te compram correção, uma linguagem nativa para os caminhos onde o layout de memória te compra velocidade. Você está aprendendo as duas linguagens neste curso exatamente por essa razão.

Agora a ressalva, e ela é real, uma caixa, sem enterrar:

> **O que o TS 7 custa hoje.** O TypeScript 7 foi entregue SEM uma API programática; o post de GA diz na lata: "Esperamos que o TypeScript 7.1 seja entregue com uma API nova (e diferente)." Ferramentas que dirigem o compilador programaticamente, typescript-eslint, plugins de linguagem de framework, não conseguem sentar no 7 por enquanto, então repos no mundo real ainda fixam 5.x. O exemplo mais emblemático é um em que este curso vai se apoiar por semanas: o @solana/kit, a biblioteca cliente moderna de Solana, faz o build com typescript ^5.9.3. Em 2026-09-02, o 7.1 não tinha saído; a tag `next` do registro é um 7.1.0 dev build. Conferido ao vivo; quando o 7.1 chegar, este parágrafo é reescrito.

Não acredite na minha palavra sobre o estado das coisas, pergunte ao registro você mesmo; esse hábito de checar versões contra a fonte em vez de assumir elas é um que o curso vai treinar:

```bash
npm view typescript dist-tags.latest dist-tags.next
# dist-tags.latest = '7.0.2'
# dist-tags.next = '7.1.0-dev.20260902.1'
```

Então instalar o 7.0.2 é um erro? Não, e a distinção importa: para os *seus* projetos, onde você roda `tsc` e `tsx` direto, o 7 é o TypeScript mais rápido já entregue e completamente pronto. O atraso está no ecossistema de ferramentaria *em volta* do compilador. Repos de produção fixam 5.x não por preguiça mas porque compatibilidade de ferramentaria é parte do que uma versão significa: uma versão é uma promessa sobre tudo que se conecta a ela, não só sobre o binário em si. Velocidade do compilador e maturidade do ecossistema dele são, agora, um trade-off.

A regra de decisão, já que você vai encarar ela nos seus próprios projetos em breve: repo greenfield onde você controla o toolchain e precisa basicamente de `tsc` mais um runner, pegue o 7 e aproveite a velocidade. Repo que se apoia em typescript-eslint, num plugin de linguagem de framework, ou em qualquer outra coisa que dirige o compilador pela API dele, fique no 5.x até o 7.1 chegar e as ferramentas alcançarem. Nenhuma das duas escolhas está errada; são respostas para perguntas diferentes. Este curso pega o lado rápido, te diz onde está a costura, e você vai reconhecer o padrão toda vez que o produto principal de um ecossistema for entregue na frente da ferramentaria dele de novo, o que nesta indústria é mais ou menos a cada trimestre.

Um parágrafo sobre sistemas de módulos, porque é tudo que 2026 deve ao assunto: por uma década o JavaScript teve duas variantes concorrentes de módulo, CommonJS (`require`) e ES modules (`import`), e a dor de interop gerou mil threads raivosas. Essa guerra terminou. `require(esm)` é estável a partir do Node 24, o `tsc --init` emite configurações ESM-first, e você definiu `"type": "module"` dez minutos atrás sem cerimônia. Escreva ESM-first, consuma o que você precisar, e se um tutorial de 2022 te avisar sobre riscos de pacote duplo, confira a data dele e feche a aba.

**Vá mais fundo (os 20%).** esta lição percorreu as flags em que você vai tropeçar e pulou o tour pela linguagem de propósito; os recursos canônicos fazem isso melhor. O TypeScript Handbook, https://www.typescriptlang.org/docs/handbook/intro.html, é a verdade oficial e dá para ler em algumas noites. A própria introdução ao TypeScript do Node, https://nodejs.org/learn/typescript/introduction, cobre a visão do runtime sobre a mesma história. Marque os dois; este curso linka capítulos, nunca re-ensina eles.

## Lab: entregue o pulse v0

Tier totalmente resolvido, e vou dizer a parte silenciosa em voz alta: esta é a maior quantidade de mão na mão que você vai receber deste curso. Todo comando está impresso, a sonda é construída passo a passo, e seus únicos espaços em branco são dois TODOs. No próximo módulo você recebe esqueletos; nos módulos finais, especificações. O challenge multi-URL depois do lab é seu primeiro pequeno passo solo. Esse recuo é deliberado, e é assim que você fica forte.

O contrato do artefato, porque lições mais adiante vão te cobrar por ele: o `pulse` v0 é um arquivo TypeScript onde `probe(url)` busca o alvo com o fetch embutido, cronometra ele com `performance.now()`, e imprime URL, status HTTP, e latência em ms. Deliberadamente stringly e de alvo único. Isso não é elogio: na lição de tipos do TypeScript que vem a seguir, a gente vai alimentar essa sonda com um alvo malformado, ver ela mentir com educação, e trocar as strings dela por uma união tipada. A versão 0 deveria mesmo ter espaço para crescer.

**1. Confirme o scaffold.** Você deve estar dentro de `pulse-station/` com `package.json` (contendo `"type": "module"`), `tsconfig.json` (com as suas duas edições), e `node_modules` do install. Prove:

```bash
npx tsc --noEmit && echo ready
# ready
```

**2. Crie o `probe.ts` com o esqueleto.** Dois TODOs, todo o resto completo:

```typescript
// probe.ts - pulse v0
type ProbeResult = {
  url: string;
  status: number;
  latencyMs: number;
};

async function probe(url: string): Promise<ProbeResult> {
  // TODO 1: capture performance.now() into `started`,
  // await fetch(url) into `res`,
  // then compute latencyMs as the difference from a second performance.now()
  return { url, status: res.status, latencyMs };
}

const target = process.argv[2];

const result = await probe(target);
// TODO 2: print one line: the url, the status, and latencyMs
// with one decimal place, space-separated
```

Leia a forma antes de preencher ela. `ProbeResult` é o registro que toda sonda retorna: qual URL, qual status HTTP, quanto tempo. `process.argv` é o array do Node com as partes da linha de comando; o índice 0 é o node em si, o índice 1 o script, o índice 2 o primeiro argumento que você passou de verdade.

Uma decisão de design vale uma pausa: `probe()` retorna um registro em vez de imprimir a saída dele. Essa divisão, medir num lugar, apresentar em outro, parece cerimônia num arquivo de vinte linhas, e é a razão de o challenge abaixo ser fácil em vez de uma reescrita. Uma função que retorna valores `ProbeResult` pode ser chamada dez vezes e ter os resultados dela coletados, ordenados, resumidos; uma função que imprime já gastou a resposta dela. Lições mais adiante cobram de `probe(url)` exatamente este contrato, então a forma que você digita agora é estrutural.

**3. Preencha o TODO 1: o par de cronometragem.** O padrão é timestamp, await no trabalho, timestamp, subtrair:

```typescript
  const started = performance.now();
  const res = await fetch(url);
  const latencyMs = performance.now() - started;
```

A ordem é tudo aqui. As duas chamadas de `performance.now()` têm que cercar o `await`; ponha a segunda chamada antes do await e você mediria o custo de começar a requisição, não de terminar ela. Esse par de cercar-o-await é o padrão mais reutilizado deste curso. Você vai escrever ele em Rust com `std::time::Instant` antes do que imagina, mesma forma, linguagem diferente.

![Três linhas de código anotadas mostrando um timestamp antes de um fetch, a chamada aguardada, e a subtração que produz a latência.](assets/v05-annotated-code.webp)

**4. Preencha o TODO 2: a linha de saída.**

```typescript
console.log(`${result.url} ${result.status} ${result.latencyMs.toFixed(1)}ms`);
```

`toFixed(1)` mantém uma casa decimal: dígitos de sub-milissegundo são ruído na escala da rede. Uma linha por sonda, divisível por máquina nos espaços. Essa é uma decisão de design minúscula que se paga em duas lições, quando um workflow parseia esta saída.

**5. Dispare a flag de propósito.** Agora confira o arquivo:

```bash
npx tsc --noEmit
```

Ele falha, e deveria:

```text
probe.ts: error TS2345: Argument of type 'string | undefined' is
not assignable to parameter of type 'string'.
```

Isso é o `noUncheckedIndexedAccess` fazendo o trabalho dele, e eu plantei a colisão: `process.argv[2]` pode não existir. Rode a sonda sem URL e, sem esta flag, `fetch(undefined)` produziria um erro desconcertante em tempo de execução três camadas abaixo. O compilador se recusa a deixar a situação existir. Isso não é trote. A flag achou um buraco de verdade num programa de verdade com onze linhas.

**6. Conserte do jeito da flag.** Uma trava antes do uso:

```typescript
const target = process.argv[2];
if (!target) {
  console.error("usage: npx tsx probe.ts <url>");
  process.exit(1);
}
```

Depois do `if`, o TypeScript estreita `target` para `string` puro: o caso undefined sai do processo, então ele não consegue chegar em `probe()`. Você não silenciou o verificador; você tratou o caso, e ganhou uma mensagem de usage de graça. Confira de novo:

```bash
npx tsc --noEmit
# (silence; silence is a pass)
```

**7. Rode de verdade.**

```bash
npx tsx probe.ts https://www.rust-lang.org
# https://www.rust-lang.org 200 498.4ms
```

Seu número vai ser diferente; o meu foi entre duas rodadas com segundos de diferença. O que tem que bater é a forma: URL, status, latência com uma casa decimal. Tente uma segunda rodada e veja a latência cair conforme as conexões esquentam. Tente `npx tsx probe.ts` sem argumento e veja a sua linha de usage em vez de uma quebra.

Se em vez disso alguma coisa brigou com você, as duas falhas clássicas de setup produzem erros inconfundíveis, os dois do meu terminal:

- `error TS2304: Cannot find name 'performance'` (ou `'fetch'`, ou `'process'`): seu `tsconfig.json` ainda tem o padrão do init `"types": []`. Defina para `"types": ["node"]` e rode de novo.
- `error TS1309: The current file is a CommonJS module and cannot use 'await' at the top level`: falta `"type": "module"` no `package.json`. Rode `npm pkg set type=module` e rode de novo.

Qualquer outra coisa, leia o erro devagar antes de sair pesquisando. As mensagens do TypeScript 7 geralmente nomeiam a flag ou a correção sem rodeios, e construir o reflexo de ler-o-erro agora paga juros compostos o curso inteiro.

![Um fluxograma do argumento de comando passando por uma trava e um fetch cronometrado até uma única linha de resultado impressa.](assets/v06-flowchart.webp)

**Checkpoint, a trava da lição:** você agora deve ter uma linha de latência de verdade para uma URL de verdade, impressa pelo seu próprio toolchain checado com strict, mais a memória de um erro de compilador que você disparou e consertou. Uma linha de terminal e uma mensagem de erro. Esse par é a vitória inteira: a sonda funciona, e você viu a maquinaria que mantém ela honesta.

## Challenge: sonde uma frota

Solo agora. Estenda o `probe.ts` para aceitar múltiplas URLs:

```bash
npx tsx probe.ts https://www.rust-lang.org https://www.typescriptlang.org https://nodejs.org
```

Requisitos:

- Imprima uma linha de latência por alvo, mesmo formato do v0.
- Ordene a saída por latência, o mais lento por último.
- O caso sem argumentos ainda imprime a linha de usage e sai.
- `npx tsc --noEmit` continua silencioso.

Dicas, não passos: `process.argv.slice(2)` te entrega todas as URLs de uma vez. Você já tem um `probe()` que retorna um `ProbeResult`; um array desses pode ser ordenado com um comparador em `latencyMs`. Se você sonda sequencialmente ou dispara todos os fetches em concorrência é sua escolha, mas note que isso muda o que os números querem dizer: sondas sequenciais cada uma tem a rede só para si, enquanto as concorrentes dividem sua conexão e podem inflar a latência uma da outra. Nenhuma das duas está errada, elas medem coisas diferentes, e saber qual pergunta você está fazendo é a habilidade real. Vamos formalizar exatamente esse trade-off na lição de async.

E espere encontrar a flag de novo. No momento em que você indexar seu array de resultados, o `noUncheckedIndexedAccess` vai te lembrar de que o array pode estar vazio, e desta vez não tem correção impressa para copiar. Você já conhece o movimento dela; trate o caso do jeito da flag.

Se você quer um segundo treino, a página desta lição na plataforma do curso traz um coding challenge complementar no painel de editor interativo dela (código starter e grader incluídos, nada para baixar): `latencyStats`, que transforma um batch de amostras em min, max, mean, e p95. Uma amostra é ruído, um resumo é sinal; essa função exata é entregue na frota no próximo módulo, quando o relatório da frota de m02-l3 põe ela em serviço na estação. Lições mais adiante entregam os challenges delas do mesmo jeito, então quando uma disser "o starter", aquele painel é onde ele mora.

## Para onde o batimento vai agora

O `pulse` v0 funciona, e você viu ele funcionar. Essa última parte é o problema. Um batimento que você precisa ficar de babá não é um batimento; ele mede só quando você lembra de perguntar. Na próxima lição sua sonda se muda para uma máquina que não é sua e roda num agendamento: git para versionar, GitHub para hospedar, e sua primeira rodada verde do Actions para executar. O primeiro ship do curso.

![Um diagrama em escada mostrando o pulse v0 hoje, a mudança dele para rodadas agendadas, e a reescrita tipada posterior, com um contrato mantido do começo ao fim.](assets/v07-diagram.webp)

Antes de ir: rode a sonda contra um site com que você realmente se importa e olhe o número. Se alguma coisa acima brigou com você — uma versão incompatível, um erro de flag que você não conseguiu decodificar — é exatamente esse o feedback que eu quero; leve para a discussão do curso, na pior das hipóteses vira a caixa de troubleshooting da próxima turma. Te vejo no primeiro check verde.
