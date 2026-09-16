# Torne estados impossíveis irrepresentáveis

**Resumo:** O módulo 1 entregou o batimento: o pulse v0 sonda uma URL com o fetch embutido, e o cron do Actions de m01-l3 roda ele num agendamento e commita status.json, uma máquina que não é a sua, rodando o seu código. Ela também roda os seus bugs. Esta lição é sobre um bug em particular, o tipo que nunca quebra. Você vai forjar um registro de sonda malformado, ver o v0 publicar ele como healthy sem reclamar, e depois deletar a categoria inteira desse bug reconstruindo o tipo de resultado da frota como uma união discriminada com um switch exaustivo. No fim, um estado errado não vai ser pego. Ele não vai ser construível.

Meça primeiro. Abra o seu repo do pulse e jogue estas quatro linhas no topo do arquivo da sonda (o topo importa: a trava de uso do v0 sai cedo quando nenhuma URL é passada), depois rode ele com `npx tsx probe.ts`:

```ts
type ProbeRecord = { status: string; latencyMs?: number };

const forged: ProbeRecord = { status: 'okay' };
console.log(forged.status !== 'timeout' ? 'healthy' : 'down');
```

Ele imprime `healthy`, depois a trava de uso do v0 reclama da URL faltando e sai com 1. Ignore a linha da trava; ela está fazendo o trabalho antigo dela. A linha que importa é a primeira. Leia o registro forjado de novo. O status dele é a string `'okay'`, não `'ok'`. Nenhuma sonda rodou. Não existe latência. E mesmo assim: healthy. Sem exceção, sem texto vermelho, nada para o cron falhar em cima. Se este registro estivesse sentado em status.json agora, o seu workflow ia commitar ele verde, e o painel que você vai construir no módulo 3 ia renderizar como up um alvo que nunca foi sondado de verdade.

Nada quebrou. É esse o problema. Fique com essa frase por um segundo, porque ela é a lição inteira: no v0, um estado errado é representável, então ele flui. Todo consumidor rio abaixo ou re-checa ele ou confia nele, e o que confia mente para quem estiver olhando.

Seu instinto pode ser "então adiciona uma checagem". Segure esse pensamento. A gente vai fazer algo melhor do que checar. A gente vai tornar o estado errado impossível de escrever.

## O padrão: estados que você não consegue escrever errado

### Um formato que consegue mentir

Aqui está o formato de resultado que o v0 usa, reduzido aos dois campos que decidem a saúde. Ele é um primo próximo do registro que a sua sonda vem escrevendo desde m01-l2, aquele também carrega `url` e um timestamp, e ele comete o mesmo pecado com uma grafia diferente: m01-l3 entregou `latencyMs: number | string`, um timeout registrado como prosa:

```ts
type ProbeRecord = { status: string; latencyMs?: number };
```

Dois campos. Parece inofensivo. Agora conte o que ele consegue dizer. `status` é `string`, o que significa que ele pode guardar `'ok'`, `'timeout'`, `'okay'`, `'OK '` com um espaço no fim, ou o texto completo de Moby Dick. `latencyMs` é opcional, o que significa que cada um desses status vem em dois sabores: com um número, ou sem. O tipo codifica todos estes com alegria:

- `{ status: 'ok' }` sem latência. Uma sonda "ok" que não mediu nada.
- `{ status: 'okay', latencyMs: 200 }` o typo que você acabou de forjar, agora vestindo uma latência plausível.
- `{ status: 'timeout', latencyMs: 143 }` um timeout que de algum jeito tem uma latência.

Nenhum desses estados pode acontecer na realidade. Uma sonda bem-sucedida sempre tem uma latência. Um timeout nunca tem. Mas o tipo não consegue dizer isso, então todo consumidor de `ProbeRecord` tem que re-derivar a realidade no ponto de uso: checar a string de status, checar se a latência está lá, decidir o que um campo faltando significa. Toda checagem é um lugar para esquecer uma checagem. O seu registro forjado passou batido porque um consumidor, aquela linhazinha `!== 'timeout'`, fez uma suposição de aparência razoável que o tipo nunca prometeu.

![A realidade permite três resultados de sonda enquanto o tipo stringly também aceita muitas combinações inválidas, como um timeout carregando uma latência.](assets/v01-comparison.webp)

### Por que código que mexe com dinheiro não pode dar de ombros

Agora o porquê, porque este curso te prometeu o porquê e esta é a lição que ganha ele.

Comece por como a web2 lida com este bug exato. O registro forjado é entregue. Algum painel mostra um bloco verde desatualizado. Um usuário abre um ticket, alguém dá um grep nos logs, uma correção sai na terça. O estado errado te custou um pedido de desculpas. Esta é uma forma boa de viver, e é por isso que um monte de empresa de web2 ainda entrega feliz em JavaScript puro: quando erros são baratos e reversíveis, prova em tempo de compilação é um imposto que você pode recusar racionalmente.

A web3 quebra essa aritmética. O código para o qual este curso está te levando, o código do qual a maioria das bases de código web3 de verdade é feita, move valor. Uma transação que passa na checagem de tipos até existir com o estado errado não produz um ticket. Ela produz uma transferência que já aconteceu, num ledger cujo objetivo inteiro de design é que ninguém possa desacontecer ela caladinho. Uma conta drenada não se desdrena. O custo de um estado errado deixa de ser "um pedido de desculpas" e vira ilimitado e irreversível, e uma vez que isso é verdade, o seguro mais barato do mercado é uma prova que o compilador vai fazer de graça, todo build, para sempre.

Então faça a pergunta mais afiada: o que exatamente uma prova em tempo de compilação compra que uma checagem em tempo de execução não compra? Uma checagem em tempo de execução é uma frase que alguém tem que lembrar de escrever, em todo ponto, em toda refatoração, para sempre. Ela roda quando o valor ruim chega, o que significa que ela roda em produção, no pior momento possível, se é que ela foi escrita. Uma prova em tempo de compilação é diferente em espécie, não em grau: ela é validação que você nunca precisa lembrar, porque o programa que contém o erro não é um programa. Ele nunca compila, então ele nunca existe, então ele nunca roda. Validação que você nunca precisa lembrar bate validação que alguém vai acabar esquecendo. Esse é o trade-off inteiro, e a web3 é o ambiente onde o preço de esquecer finalmente fez todo mundo pagar pela prova.

![Um estado errado vira um ticket corrigível na web2, uma transferência irreversível na web3, e nunca é entregue quando o compilador rejeita ele.](assets/v02-flowchart.webp)

O ecossistema vem votando nisso com os pés já faz um tempo, e 2026 nos deu a cédula mais alta até agora. Entre março de 2025 e 2026-07-08, a Microsoft portou o compilador do TypeScript para Go e entregou ele como TS 7: um ecossistema tão comprometido com prova em tempo de compilação que reconstruiu o próprio provador por velocidade. Esse não é o comportamento de uma comunidade que acha que tipos são lint. Tipos são infraestrutura estrutural em 2026, e o compilador que checa a sua sonda hoje é o mais rápido já construído exatamente porque tanta coisa se apoia nele agora.

![Uma linha do tempo do anúncio de março de 2025 do port para Go até o TypeScript 7 sendo entregue em 8 de julho de 2026.](assets/v03-timeline.webp)

### A união: cada variante carrega exatamente os dados dela

Antes da correção, descarte as não-correções tentadoras, porque você vai encontrar as três em bases de código de verdade e cada uma falha por um motivo que vale a pena ter.

Correção ingênua um: validar em todo lugar. Escreva um helper `isValidRecord` e chame ele em todo ponto de uso. Isso funciona até o dia em que alguém adiciona um ponto de uso e não sabe que o helper existe, o que numa base de código que cresce é terça que vem. Você transformou um problema de design num teste de memória, e o modo de falha é silencioso, exatamente como o que você acabou de ver.

Correção ingênua dois: testar mais forte. Escreva um teste unitário para o caso `'okay'`. Bom instinto, ferramenta errada: um teste prova que as entradas que você pensou se comportam, e a identidade inteira deste bug é ser a entrada que ninguém pensou. Testes amostram o espaço de estados. A gente precisa encolher ele.

Correção ingênua três: comentários e disciplina. Documente que status tem que ser `'ok'` ou `'timeout'` e confie no time. Esta é a que todo mundo de fato entrega, e é o habitat natural da mentira educada, porque um comentário compila não importa o que o código faça.

Repare no formato das três falhas: cada uma deixa o estado errado representável e depois posta uma trava em algum lugar, torcendo para que a trava esteja sempre acordada. Então a correção de verdade inverte a abordagem. Não mais travas. Um formato que não tem codificação para as mentiras.

Pergunte o que um resultado de sonda de fato é. Ele é uma de exatamente três histórias: o alvo respondeu a tempo, o alvo nunca respondeu, ou o alvo respondeu com um erro HTTP. Cada história vem com a evidência dela, e crucialmente, só com a evidência dela. O TypeScript deixa você escrever isso direto:

```ts
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };
```

Isto é uma união discriminada. Três formatos de objeto unidos por `|`, compartilhando uma propriedade, `kind`, cujo tipo em cada ramo não é `string` mas um literal único: exatamente `'ok'`, exatamente `'timeout'`, exatamente `'http-error'`. Essa propriedade literal compartilhada se chama discriminante, e é ela que faz o padrão inteiro encaixar, porque o compilador consegue diferenciar as variantes olhando para ela.

Percorra as variantes como um auditor. `'ok'` carrega `latencyMs`, obrigatório, porque um sucesso sem uma medição não é um sucesso. `'timeout'` carrega `budgetMs`, o budget que ele estourou, e não tem campo de latência nenhum, nem um opcional, nenhum. `'http-error'` carrega o código `status` que o servidor devolveu. Nada opcional em lugar nenhum. Agora rode as mentiras de ontem de novo contra este tipo. Um ok sem latência? Propriedade obrigatória faltando, sem codificação. Um `'okay'` com typo? Não é um dos três literais, sem codificação. Um timeout com uma latência? `latencyMs` não existe naquela variante, sem codificação. As combinações incoerentes não foram pegas. Elas deixaram de ter grafia.

Vou confessar: eu já entreguei a versão com campo opcional disso mais vezes do que quero admitir. `latencyMs?: number` parece tão razoável quando você escreve, um campo, cobre os dois casos, indo rápido. Opcionalidade é exatamente como estados impossíveis voltam de fininho. Todo `?` num campo que é de verdade "presente em algumas variantes" é uma portinha que você deixou aberta, e alguma coisa vai acabar passando por ela vestindo a string de status errada. A disciplina da união, cada variante carrega exatamente os dados dela, é o hábito que mantém a porta fechada.

### Narrowing pra valer

Uma objeção justa aterrissa bem aqui: tá, o tipo é honesto, mas `result.latencyMs` não compila mais de jeito nenhum, porque `latencyMs` só existe em um dos três ramos. A gente acabou de tornar o tipo inutilizável?

Não. A gente fez ele exigir prova antes do uso, e o narrowing do TypeScript, o estreitamento de tipos, é como você fornece a prova. Cheque o discriminante e o compilador estreita a união para o único braço que combina, dentro daquele bloco só:

```ts
function describe(result: ProbeResult): string {
  if (result.kind === 'ok') {
    return `${result.latencyMs.toFixed(1)}ms`;
  }
  if (result.kind === 'http-error') {
    return `HTTP ${result.status}`;
  }
  return `no answer in ${result.budgetMs}ms`;
}
```

Dentro do primeiro ramo, `result` é `{ kind: 'ok'; latencyMs: number }` e nada mais, então `latencyMs` é garantido, sem checagem opcional, sem undefined. Dentro do segundo, `status` é garantido. E olhe para a última linha: depois que duas checagens eliminaram duas variantes, o compilador estreitou o que sobrou para `'timeout'` sozinho, então `budgetMs` simplesmente funciona. Você nunca falou para ele. Ele fez a eliminação.

O mesmo motor de narrowing roda com combustível mais simples também. `typeof value === 'string'` estreita um `unknown` para `string` dentro do bloco. `value === null` estreita um `T | null` para `T` no ramo else. Checagens de discriminante, checagens de `typeof`, checagens de igualdade: são todas a mesma jogada, um teste em tempo de execução que o compilador observa e transforma em informação de tipo. Você vai usar as três nas bordas da frota antes desta lição acabar.

E aqui está um retorno que você já ganhou sem perceber. Lembra do `'okay'` forjado da abertura, o typo que começou esta lição inteira? Escreva o mesmo typo contra a união e veja o que acontece:

```ts
function isUp(result: ProbeResult): boolean {
  return result.kind === 'okay';
}
```

```
error TS2367: This comparison appears to be unintentional because the types
  '"http-error" | "ok" | "timeout"' and '"okay"' have no overlap.
```

Contra o formato que o v0 tinha, `record.status === 'okay'` era uma comparação perfeitamente legal entre duas strings, e o compilador não tinha nada a dizer. Ele não conseguia. `string === string` é sempre uma pergunta razoável. Contra a união, o tipo do discriminante são três literais específicos, então comparar ele com `'okay'` é comprovadamente sempre falso, e o compilador marca a própria comparação como um bug. A classe do typo não ficou mais difícil de escrever. Ela ficou impossível de compilar. Esse antes e depois vale a pena reproduzir na sua cabeça, porque é a demonstração única mais clara do que você comprou: a checagem que você tinha que olhar no code review, uma máquina agora faz a cada tecla.

![Cada checagem de discriminante descasca uma variante da união até o ramo final ser conhecido só por eliminação.](assets/v04-diagram.webp)

### Exaustividade é uma feature que você escolhe ativar

O narrowing te dá acesso seguro. Existe uma segunda garantia disponível, e você tem que buscar ela deliberadamente: a garantia de que você tratou toda variante. É aqui que o padrão vai de arrumadinho a genuinamente estrutural, e o preço da entrada é uma função de quatro linhas:

```ts
function assertNever(value: never): never {
  throw new Error(`Unhandled variant: ${JSON.stringify(value)}`);
}
```

Nada mágico. Uma função comum cujo tipo de parâmetro é `never`, o tipo sem valores. Qualquer função que recebe `never` se comportaria de forma idêntica. A mágica está inteiramente no narrowing: dê switch no discriminante, trate toda variante, e no braço `default` o compilador eliminou tudo, então o tipo que sobrou é `never`, e a chamada passa na checagem de tipos. Esqueça uma variante, ou adicione uma nova depois, e o que sobrou não é mais `never`. A chamada para de compilar, e o erro nomeia a variante exata que você não tratou, na linha exata.

![Uma instrução switch anotada cujo braço default compila enquanto todas as variantes estão tratadas e falha por nome quando uma variante nova aparece.](assets/v05-annotated-code.webp)

Leia essa consequência de novo, porque ela inverte como refatorar parece. Adicionar uma variante `'dns-error'` a uma base de código cheia desses switches não cria uma caçada por todo lugar que precisa de atualização. Ela cria uma checklist de erros de compilação: todo ponto não tratado falha, por nome, até cada um decidir o que dns-error significa para ele. O compilador escreve a planilha de refatoração para você. Com honestidade, de tudo nesta lição, esta é a parte que eu mais queria que alguém tivesse me mostrado antes, e também não é sabedoria só de TypeScript. Esta jogada exata volta no módulo 4 vestindo uma bandeira de Rust, onde `match` é exaustivo por padrão e o compilador segura a caneta desde o começo. Aprenda ela aqui, colete ela de novo lá.

Um aviso, e é o footgun mais afiado da lição. Um braço `default:` que faz qualquer outra coisa além de `assertNever`, digamos `default: return 'down'`, silencia o compilador e vende caladamente a garantia que você acabou de comprar. Variantes futuras passam batido por ele, não classificadas, para sempre, e o tsc não diz nada, porque você mandou ele não dizer. Um default catch-all é a mentira educada com uma assinatura de tipo. Busque um só numa fronteira de verdadeiro não-me-importa, e saiba o que você está vendendo quando buscar.

### unknown em vez de any na fronteira

Existe mais uma disciplina para instalar antes do lab, e ela vive nas bordas do seu programa, onde arquivos JSON, respostas RPC e os dados de outras pessoas chegam.

O TypeScript te dá dois tipos para "eu não sei o que isto é", e eles são opostos vestindo nomes parecidos. `any` é o compilador se demitindo: toda operação num `any` é permitida, sem checagem, e todo valor que ele toca herda o dar de ombros, se espalhando pelo seu grafo de chamadas feito solvente. `unknown` é o compilador exigindo prova: nenhuma operação é permitida até você estreitar ele, com as ferramentas exatas que você acabou de aprender. Mesmo runtime, os dois apagados para nada, defaults opostos. Um permite tudo e não pede nada. O outro não permite nada até você ter mostrado o seu trabalho.

![Comparação lado a lado mostrando any permitindo tudo sem checagem enquanto unknown bloqueia todo uso até o formato do valor ser provado.](assets/v06-comparison.webp)

Isto importa para a frota agora mesmo porque `JSON.parse` te devolve dados sobre os quais você não provou nada, e o tipo de retorno dele deveria ser tratado como `unknown` em toda fronteira que é sua. Tipe ele como `any` para ir rápido e um ponto de fetch infecta todo consumidor rio abaixo com acesso sem checagem. Tipe ele como `unknown` e o compilador força a disciplina de parse na fronteira que você vai construir dentro de `parseProbe` no lab: prove o formato uma vez, na borda, e tudo lá dentro opera em tipos honestos. Essa disciplina é exatamente o que o arquivo de config que a frota ganha na próxima lição, e toda resposta RPC depois dele, vai precisar, e é exatamente aí que a próxima lição pega o fio.

Vamos usar isso pra valer uma vez, para virar hábito e não slogan. Suponha que um registro cru chega de um arquivo, formato desconhecido, confiança zero. Aqui está a trava da fronteira, construída inteiramente com as jogadas de narrowing que você já tem:

```ts
function readRecord(raw: unknown): ProbeResult | null {
  if (typeof raw !== 'object' || raw === null) return null;
  const kind = (raw as Record<string, unknown>)['kind'];
  const value = (raw as Record<string, unknown>)['value'];
  if (typeof kind !== 'string' || typeof value !== 'number') return null;
  return parseProbe(kind, value);
}
```

Rastreie as provas conforme elas acumulam. A checagem `typeof raw !== 'object'` mais a checagem de igualdade `raw === null` juntas provam que a gente está segurando um objeto de verdade antes de tocar nele, e repare que as duas são necessárias: `typeof null` é `'object'`, uma verruga de vinte anos do JavaScript que a checagem de igualdade remenda. Depois cada campo é puxado para fora como `unknown` e interrogado com `typeof` até ele confessar que é uma `string` ou um `number`. Só então, com toda afirmação provada, o valor ganha o direito de entrar em `parseProbe`. Tente pular qualquer checagem e o compilador te para na linha seguinte, porque você está operando num valor cujo formato você ainda não demonstrou. Essa exigência constante de recibos é irritante por mais ou menos um dia. Depois algum registro malformado chega às três da manhã, quica nesta função como um `null`, e você para de notar a irritação para sempre.

Sim, isto é verboso. Cinco linhas de interrogatório para dois campos, e um objeto de config de verdade tem vinte. Sinta essa fricção e lembre dela, porque ela é a dor exata que faz a ferramenta da próxima lição acertar: uma biblioteca de schema escreve esta função inteira a partir de uma declaração, e o tipo sai de bônus. Você tem permissão de ficar irritado. A irritação é o currículo.

O que nos traz ao limite franco, e ele merece o próprio parágrafo em vez de uma nota de rodapé. Os tipos do TypeScript são apagados em tempo de execução. A união prova teoremas sobre os valores que o seu próprio código constrói, mas ela não prova nada, nada mesmo, sobre os bytes que chegam de um arquivo ou da rede. Declarar `const data: ProbeResult = JSON.parse(raw)` não é uma prova, é uma fantasia. As garantias do sistema de tipos começam só depois que uma checagem real em tempo de execução ganhou elas, e é por isso que `parseProbe` retorna `ProbeResult | null` em vez de asserir, e por que a versão sistemática dessa ideia, schemas que geram tanto a checagem em tempo de execução quanto o tipo de uma fonte só, é o assunto inteiro da próxima lição.

![Uma linha de fronteira separa tipos de tempo de compilação apagados de bytes de tempo de execução sem tipo, com parseProbe como a única trava que converte um no outro.](assets/v07-diagram.webp)

### O trade-off

Toda lição deste curso nomeia o custo, então aqui está ele. Modelar com união é cerimônia que você paga adiantado: mais declarações de tipo do que o formato v0 de uma linha, um passo de parse em toda fronteira, e uma variante nova quebra todo switch na base de código até cada ponto decidir o que fazer com ela. Num domingo de hackathon, esse barulho é fricção de verdade, e `{ status: string }` genuinamente vai te levar à demo mais rápido. A conta chega depois, exatamente no momento em que você menos pode pagar por ela, e no domínio deste curso a conta não vem com política de reembolso. Barulhenta por velocidade, magnífica por corretude: você agora sabe de que lado desse trade-off este curso está, e mais útil ainda, você sabe como escolher por projeto em vez de por hábito.

**Vá mais fundo (os 20%).** esta lição te ensinou por que uniões existem e os padrões que você vai usar diariamente: o discriminante, o switch exaustivo, `unknown` nas bordas. A taxonomia completa de narrowing, type guards, o operador `in`, assertion functions, vive no capítulo Narrowing do TypeScript Handbook, e a seção discriminated-unions dele é o tratamento canônico do padrão de hoje: [https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions](https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions). Salve como bookmark, leia esta semana, e quando este módulo acabar, o repo type-challenges é o pátio de treino onde esses músculos são construídos de verdade. O lab abaixo não precisa de nada do material de bookmark.

## Lab: delete a categoria

É assim que as rodinhas ficam neste módulo: eu trabalho os passos 1 a 8 com você, todo arquivo mostrado e explicado. O passo 9 você completa só com os erros do compilador como guia, sem walkthrough em prosa. O challenge depois disso é inteiramente seu. Esse recuo é deliberado, e ele fica mais íngreme no próximo módulo.

Tudo isto acontece no seu repo do pulse do módulo 1, em `probe.ts`, o mesmo arquivo de sonda que você vem crescendo desde m01-l2. Uma nota de organização antes de começar: aquele arquivo já declara o registro `ProbeResult` do v0 (o formato `{ url, status, latencyMs }`, com um timestamp pegando carona na cópia da frota). A união do passo 2 assume o nome, então delete a declaração antiga quando adicionar a nova, dois tipos com um nome é um erro de compilação, e desta vez o erro estaria certo.

Uma segunda nota de organização, sobre o arquivo que você NÃO está editando. `fleet.ts`, o escritor do cron de m01-l3, declara o próprio tipo de resultado separado com formato do v0 (é lá que `url` e `checkedAt` vivem) e não importa nada de `probe.ts`, então nada do que você fizer hoje encosta nele: ele continua compilando, a trava de typecheck do CI de m01-l3 continua verde, e `status.json` mantém o contrato congelado `{ url, status, latencyMs, checkedAt }` dele até m03-l2 religar o escritor, logo antes de o painel colocar um schema no arquivo. Sim, isso significa que o escritor da frota ainda fala o dialeto mentiroso do v0 depois de hoje. Deliberado: esta lição deleta a categoria dentro de `probe.ts`; o lado da frota é reconstruído em schemas ao longo das próximas duas lições.

1. **Reproduza a mentira primeiro.** Se você pulou o registro forjado da abertura, faça agora: adicione `const forged: ProbeRecord = { status: 'okay' };` e a checagem `!== 'timeout'`, depois rode as duas coisas:

```bash
npx tsx probe.ts     # prints: healthy
                     # then: usage: npx tsx probe.ts <url>, exit 1
npx tsc --noEmit     # exits clean. green.
```

   O erro de usage e o exit 1 são a trava de nenhum-argumento do v0 fazendo o trabalho normal dela; a linha forjada imprime antes de a trava disparar, e aquele primeiro `healthy` é a mentira que importa para a gente. Veja ela imprimir, e confirme que o compilador está verde também. Ele não tem objeção, porque o tipo que você deu para ele genuinamente permite isto. Esse check verde é a sua foto do antes.

2. **Modele a união.** No topo do arquivo, delete o registro `ProbeResult` antigo do v0 e adicione a nova camada de tipos da frota no lugar dele. Delete as linhas forjadas do passo 1 também, as três (o tipo `ProbeRecord`, a const `forged` e o `console.log` dela); elas eram a foto do antes, e deixadas ali elas imprimiriam um `healthy` perdido antes de toda rodada de sonda para sempre. (o tsc fica vermelho no momento em que você faz estas edições, porque a sonda do v0 ainda retorna o formato antigo; os passos 5 a 7 levam cada um desses erros de volta ao verde, que é exatamente o workflow que esta lição está vendendo.)

```ts
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };

type Verdict = 'up' | 'degraded' | 'down';

function assertNever(value: never): never {
  throw new Error(`Unhandled variant: ${JSON.stringify(value)}`);
}
```

   Repare que `Verdict` é ele mesmo uma pequena união de literais. O mesmo truque que consertou o tipo de resultado também impede `'degarded'` de sair do seu classificador algum dia.

3. **Coloque o formato antigo e o novo numa tela só.** Este lado a lado é a lição inteira em duas declarações, então de fato olhe para os dois juntos antes de deletar qualquer coisa:

```ts
// v0: one shape, many lies
type ProbeRecord = { status: string; latencyMs?: number };

// typed fleet: three shapes, no spare states
type ProbeResult =
  | { kind: 'ok'; latencyMs: number }
  | { kind: 'timeout'; budgetMs: number }
  | { kind: 'http-error'; status: number };
```

   O diff é a lição. O campo stringly virou três discriminantes literais. O campo opcional virou um campo obrigatório que existe só onde ele é verdade. Tudo que o v0 conseguia escrever errado, a união não consegue escrever de jeito nenhum.

4. **Faça o parse na fronteira.** Pares de `(kind, value)` não confiáveis vêm de fora; esta função é o checkpoint de fronteira que transforma eles em prova ou manda eles de volta:

```ts
function parseProbe(kind: string, value: number): ProbeResult | null {
  switch (kind) {
    case 'ok':
      return { kind: 'ok', latencyMs: value };
    case 'timeout':
      return { kind: 'timeout', budgetMs: value };
    case 'http-error':
      return { kind: 'http-error', status: value };
    default:
      return null;
  }
}
```

   Repare no que o tipo de retorno diz em voz alta: `ProbeResult | null`. O parse pode falhar, então o tipo admite isso, e todo chamador é forçado pelo compilador a tratar o `null` antes de tocar no resultado. O alvo malformado da abertura morre bem aqui, na fronteira, como um `null` que você tem que tratar em alto e bom som, em vez de fundo num painel como um bloco verde.

![Registros não confiáveis passam por um único checkpoint de parse onde entradas forjadas caem fora como null e só resultados provados seguem para dentro.](assets/v08-flowchart.webp)

5. **Reescreva o classificador exaustivamente.** Substitua qualquer lógica do v0 que decidia a saúde por isto:

```ts
function classifyProbe(result: ProbeResult): Verdict {
  switch (result.kind) {
    case 'ok':
      if (result.latencyMs < 400) return 'up';
      if (result.latencyMs <= 1000) return 'degraded';
      return 'down';
    case 'timeout':
      return 'down';
    case 'http-error':
      return result.status === 429 ? 'degraded' : 'down';
    default:
      return assertNever(result);
  }
}
```

   Duas decisões de julgamento aqui valem o porquê delas. As faixas: abaixo de 400ms é saudável, 400 até 1000 inclusive é degraded, e só estritamente acima de 1000 é down, porque uma resposta lenta ainda é uma resposta. E 429: uma resposta de rate-limit significa que o alvo está vivo e falando, só cansado de você, então é `'degraded'`, não `'down'`. O resto do switch é encanamento, e repare em quão pouco código defensivo ele contém. Dentro de cada braço, os campos simplesmente existem. O narrowing já provou eles.

6. **Ligue a própria sonda à união.** A função da sonda agora retorna o tipo honesto de ponta a ponta:

```ts
async function probe(url: string, budgetMs = 5000): Promise<ProbeResult> {
  const started = performance.now();
  try {
    const res = await fetch(url, { signal: AbortSignal.timeout(budgetMs) });
    const latencyMs = performance.now() - started;
    if (!res.ok) {
      return { kind: 'http-error', status: res.status };
    }
    return { kind: 'ok', latencyMs };
  } catch {
    return { kind: 'timeout', budgetMs };
  }
}
```

   Tente, só como experimento, retornar o estado forjado desta função: `return { kind: 'ok' }` sem latência. O compilador recusa antes de você conseguir salvar o arquivo. Esse é o antes e depois desta lição inteira comprimido em um rabisco vermelho: a mentira que você viu imprimir `healthy` no passo 1 agora não consegue sair da função que a teria contado.

7. **Reescreva o driver, o último pedaço de v0 que restou.** Rode `npx tsc --noEmit` agora e os erros que restam, cinco deles, todos apontam para o fim do arquivo: o loop do driver de m01-l2 ainda coleta `results`, ordena por `latencyMs`, e imprime `result.url`/`result.status`/`result.latencyMs.toFixed(1)`, nada disso existe em todo braço da união (e `url` em nenhum deles). Isso é o compilador te dizendo que o formato de saída da CLI foi desenhado para o formato antigo, então o driver é redesenhado, não remendado. Primeiro, se você ainda não fez, adicione a função `describe` da seção de teoria ao arquivo, literal da seção de narrowing; ela está prestes a virar o formatador de detalhe da CLI. Depois mantenha as linhas de `targets`/trava-de-uso e substitua tudo abaixo delas por:

```ts
for (const target of targets) {
  const result = await probe(target);
  console.log(`${target} ${classifyProbe(result)} (${describe(result)})`);
}
```

   O array `results`, a ordenação e a linha de log antiga vão todos embora. A nova saída é a linha com o veredito primeiro que a frota de fato quer, com a evidência entre parênteses:

```bash
npx tsx probe.ts https://www.rust-lang.org
# https://www.rust-lang.org up (88.7ms)
```

   A sua latência vai ser diferente; o formato não. Repare no que o redesenho derrubou: ordenar por latência fazia sentido quando todo registro tinha um `latencyMs`, e sob a união honesta não faz, porque um timeout não tem latência para ordenar. O tipo não só encontrou o bug, ele aposentou a feature em que o bug morava.

8. **Verifique, depois extraia a prova negativa.** Primeiro a checagem positiva: `npx tsc --noEmit` deve sair limpo. Verde. Mas verde-quando-correto é só metade do que você comprou, então agora prove que a garantia é real quebrando ela de propósito. Comente o braço `case 'http-error':` inteiro em `classifyProbe` e rode `npx tsc --noEmit` de novo:

```
probe.ts: error TS2345: Argument of type '{ kind: "http-error"; status: number; }'
  is not assignable to parameter of type 'never'.
```

   Olhe para o que ele fez. Ele não disse "tem algo errado em algum lugar". Ele nomeou a variante faltando, na linha do `assertNever`, na função que parou de tratar ela. O compilador está fazendo a review. Restaure o braço, confirme que está limpo, e esse é o seu checkpoint: você deveria agora conseguir produzir os dois estados sob demanda, verde quando exaustivo, um erro nomeado quando não.

9. **O treino do dns-error. Sua vez, compilador como guia.** Agora, uma falha de DNS, sondando `https://definitely-not-a-real-host.example`, cai no `catch` e é registrada como um timeout, o que é uma pequena mentira por si só: o host não deu timeout, ele não existe. Adicione uma quarta variante, `{ kind: 'dns-error'; host: string }`, a `ProbeResult`, depois rode `npx tsc --noEmit` e conserte nada além do que o compilador nomear, um erro de cada vez, até ficar verde. Sem walkthrough para este, e sem necessidade: espere que os erros te marchem para exatamente dois pontos, o `assertNever` do classificador e o fall-through da função `describe` (ela dependia de `'timeout'` ser o único que sobrava, e o passo 7 tornou `describe` parte da CLI). Quando o tsc estiver verde de novo, todo switch na sua frota decidiu conscientemente o que uma falha de DNS significa. Aquela planilha que você acabou de seguir foi escrita pelo compilador, e é a experiência exata de manter código tipado num time de verdade.

![Adicionar uma variante irradia erros de compilador nomeados para todo ponto não tratado até cada um ser corrigido e o build ficar verde.](assets/v09-diagram.webp)

## Challenge: faça o parse uma vez, classifique exaustivamente

Agora a repetição sem guia. O challenge classify-probe-result vive no painel interativo de coding-challenge na página desta lição, igual ao `latencyStats` de m01-l2: starter, grader e hints todos no editor do navegador, nada para baixar. O starter que ele te entrega é pensamento v0 puro: só sondas `'ok'` são consideradas, a faixa degraded não existe, e todo o resto é jogado em `'down'`. Reconstrua ele do jeito que você acabou de reconstruir a frota. Modele `ProbeResult` como uma união discriminada, faça o parse do par `(kind, value)` que chega uma vez na fronteira (kinds desconhecidos dão parse para `null`, exatamente como `parseProbe` no passo 4), depois classifique com um switch exaustivo fechado por `assertNever`. Para ser preciso sobre `'invalid'`, já que ele não é um quarto veredito: mantenha `Verdict` como a união de três membros do passo 2, e faça a função voltada para o grader retornar `Verdict | 'invalid'`, onde `'invalid'` é o que ela responde quando o parse voltou `null`. Falha de parse e classificação continuam dois fatos diferentes, e o tipo de retorno diz isso. Preste atenção nas bordas que o grader presta: 400 e 1000 caem os dois em `'degraded'`, um 429 significa que o alvo respondeu então é `'degraded'` também, e os oito testes incluem os valores de fronteira e o caso do kind desconhecido. Tudo que você precisa está acima; nada do que você deixou como bookmark é obrigatório. Se você quiser o flex extra depois, delete um braço e preveja o erro antes de rodar o tsc.

## Checkpoint

Antes de fechar a aba, a recuperação de 30 segundos, em voz alta ou numa nota, sem espiar: o que `assertNever` prova, e quando ele dispara? Você está buscando algo como: se toda variante está tratada, o valor do braço default estreita para `never`, então a chamada compila; uma variante nova torna ele não-never e a compilação falha bem ali. Se essa frase saiu limpa, você domina o mecanismo. Se não saiu, releia o passo 8, rode a prova negativa mais uma vez, e ela vai sair.

E me conte como foi o treino, com honestidade: os erros do compilador de fato te levaram a todo ponto no passo 9, ou você achou uma lacuna onde a planilha deixou passar algo? Esse feedback molda quanto os próximos módulos vão se apoiar nesta jogada. Onde você ficar preso, diga isso na thread da comunidade desta lição; um ponto de travamento nomeado cedo salva cinco aprendizes atrás de você.

Os seus resultados são tipados agora. Mas a frota está prestes a ganhar um arquivo de config, JSON em disco chegando como dado não confiável e sem tipo, e o mesmo vale para toda resposta RPC que você vai buscar na vida, e você agora sabe exatamente por que uma união não ajuda ali: tipos são apagados, e uma fantasia não é uma prova. Uma união não consegue provar nada sobre bytes que você não fez o parse. Próxima lição: zod na fronteira, onde o schema é a checagem em tempo de execução e a fonte única do tipo ao mesmo tempo, e `parseProbe` cresce para algo que consegue travar a fronteira inteira. A trava da fronteira está prestes a ganhar um schema.
