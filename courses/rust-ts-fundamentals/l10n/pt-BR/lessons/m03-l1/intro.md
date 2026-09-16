# package.json é um contrato: workspaces, engines, peers

O M2 fechou com uma frota testada: resultados de sonda tipados, config validada com zod, concorrência disciplinada, e uma suíte vitest travando o cron do Actions antes que ele sonde. A frota funciona. Ela também é um blob indiferenciado de código, e em três lições parte dela é entregue ao npm, onde a máquina de cada consumidor passa a cobrar promessas que você ainda não fez conscientemente.

Então, antes de qualquer teoria, quebre uma dessas promessas de propósito. Abra o `package.json` da frota e adicione uma cláusula engines reivindicando um major do Node que não existe:

```json
{
  "engines": {
    "node": ">=99"
  }
}
```

Agora rode um install novo com a checagem de engine ligada (o npm só avisa por padrão; a flag faz ele cobrar):

```bash
rm -rf node_modules
npm install --engine-strict
```

```
npm error code EBADENGINE
npm error engine Unsupported engine
npm error engine Not compatible with your version of node/npm: pulse-station@1.0.0
npm error notsup Not compatible with your version of node/npm: pulse-station@1.0.0
npm error notsup Required: {"node":">=99"}
npm error notsup Actual:   {"node":"v24.20.0","npm":"11.19.0"}
```

(Transcrição do npm 11.19; o nome do pacote é o que o `npm init -y` pegou do seu diretório `pulse-station` lá em m01-l2, os seus dígitos vão ser outros, e um npm mais antigo escreve o rótulo `notsup` por extenso.) Leia isso como um dev de verdade. `Required` é o que o seu package.json reivindica; `Actual` é a máquina em que ele aterrissou; o install recusou porque uma promessa escrita e um ambiente real discordaram. Não é uma mensagem de erro, é uma cláusula de contrato disparando, e todo campo que a gente cobre hoje dispara exatamente assim na máquina de alguém, mais cedo ou mais tarde. Reverta a sabotagem e reinstale antes de seguir.

## Resumo

Os achados logo de cara:

- `package.json` é um contrato com máquinas e consumidores que você nunca vai conhecer: `exports` promete uma superfície de API, `engines` uma faixa de runtime, `packageManager` fixa a ferramenta, faixas de dependência escolhem quais futuros você aceita, `peerDependencies` nomeia o que o CONSUMIDOR tem que fornecer.
- Você vai extrair o motor da frota (a união `ProbeResult`, `classifyProbe`, os helpers de backoff) para o `pulse-core`, um pacote de workspace pnpm, com a suíte de m02-l4 verde antes e depois: a estrutura muda, o comportamento não.
- A extração merece a cerimônia dela porque o segundo consumidor é real, não especulativo: na próxima lição um painel React importa o classificador, e no M7 um worker de edge importa o core puro.
- O artefato de peer ao vivo: `helius-sdk` 3.1.0 dá peer em `@solana/kit ^6.9.0` enquanto o latest do kit está em 8.2.0 (ambos sondados em 2026-09-02). Instale os dois juntos e o npm recusa. A regra durável, que se paga no M8: fixe aquilo contra o que as suas deps dão peer, por workspace, nunca um dígito decorado.
- A ajuda recua no cronograma: eu conduzo a primeira jogada de extração diff por diff, você move os módulos restantes com a mesma receita, e o diagnóstico do conflito de peer mais o coding challenge são só seus.

## O contrato, cláusula por cláusula

Aqui vai a síntese que faz o arquivo inteiro fazer sentido: um pacote é uma promessa sobre ambientes que você nunca vai ver. O seu código vai rodar num laptop que você nunca tocou, num Node que você nunca instalou, ao lado de versões que você nunca escolheu, importado por uma pessoa que você nunca vai conhecer. O `package.json` escreve quais desses futuros você promete sobreviver; todo campo abaixo é uma cláusula, e a sabotagem de abertura já te mostrou a cobrança.

### Por que extrair agora: o teste do segundo consumidor

Tudo na frota mora num único `src/`. O classificador que decide "up" versus "degraded", o tipo união que torna estados errados irrepresentáveis, a matemática de backoff que mantém a frota educada: tudo isso fica ao lado da fiação da CLI e do entry point do cron. Isso estava certo. Um consumidor, um blob, zero cerimônia.

Eu já estive do outro lado disso, e feio: dois repos, cada um com a própria cópia colada da mesma função de classificação, e na semana em que uma cópia levou uma correção de fronteira e a outra não, os nossos painéis discordaram sobre se a produção estava saudável. Ninguém notou por dias porque as duas cópias estavam verdes nos testes delas. Cópias divergem. Esse é o argumento inteiro.

A regra franca não é "sempre extraia". Uma fronteira de pacote custa cerimônia: dois arquivos `package.json` para manter verdadeiros, um mapa de exports para manter, todo refactor perguntando "essa API é pública?" Copiar e colar não tem nenhum desses custos, até bem na hora em que o segundo consumidor aparece. Então: extraia quando o segundo consumidor for REAL. O nosso está agendado: na próxima lição o painel renderiza o `status.json` com o mesmo classificador com que a frota publica, e se os dois discordarem sobre "degraded", o painel mente para olhos humanos. O worker de edge do M7 faz três. A extração acontece agora porque a janela de divergência abre agora.

![O repo da frota se divide em pulse-core segurando os módulos do motor e pulse-fleet segurando o app, unidos por uma única seta de dependência de workspace.](assets/v01-diagram.webp)

A forma que a gente usa é deliberadamente pequena, e tem um nome que vale cunhar uma vez: monorepo-lite. Um repo, um diretório `packages/`, um arquivo de workspace de duas linhas, e nada mais. Sem Nx, sem turborepo, sem grafo de tarefas. Essas ferramentas resolvem orquestração de build para dezenas de pacotes; a gente tem dois. Recorrer a elas aqui seria adotar um trem de carga para atravessar a rua. Quando o seu workspace crescer além do ponto em que `pnpm -r` parece lento, você vai saber, e as ferramentas vão continuar lá.

### O workspace e o protocolo

A fiação são dois arquivos. Primeiro, o `pnpm-workspace.yaml` na raiz do repo diz ao pnpm onde os pacotes moram:

```yaml
packages:
  - "packages/*"
```

Segundo, o consumidor declara a dependência dele usando o protocolo de workspace:

```json
{
  "dependencies": {
    "pulse-core": "workspace:*"
  }
}
```

`workspace:*` quer dizer: resolva isso pelo workspace, nunca pelo registro, seja qual for a versão atual. No install, o pnpm cria um symlink de `packages/pulse-core` dentro do `node_modules` da frota, então a linha de import no código da frota se lê exatamente como qualquer dependência de terceiros:

```ts
import { classifyProbe, type ProbeResult } from "pulse-core";
```

Com honestidade: esse link é uma dádiva. Edite um arquivo no `pulse-core`, e a frota vê a mudança na hora, sem publish, sem version bump, sem reinstall. Você fica com a disciplina da fronteira de pacote sem nenhuma das idas e voltas do registro. Na hora de publicar, o `pnpm publish` reescreve `workspace:*` numa faixa de versão de verdade, então a ergonomia fica local e o artefato publicado fica honesto. Uma ressalva que vira carga estrutural em m03-l4, a volta olímpica do módulo: o `npm publish` puro não faz reescrita nenhuma, e aquela lição publica com npm; ela só se safa porque o pulse-core já vem com zero dependências de runtime, então não existe linha `workspace:*` para vazar para o registro. Publique um pacote que depende de um irmão de workspace, e a grafia pnpm para de ser opcional.

Pergunta justa antes de a gente se comprometer: o npm também tem workspaces, então por que pnpm? Duas razões honestas. O ecossistema escolheu: todo repo Solana sério em TypeScript que você vai ler é um workspace pnpm, e ler o mundo real com fluência é um objetivo declarado aqui. E o layout mais rígido do pnpm (pacotes só enxergam o que declaram, não o que quer que tenha sido içado ao alcance) faz com que uma linha de dependência faltando falhe na sua máquina hoje em vez de na máquina de um consumidor depois do publish; para um pacote a caminho do npm daqui a três lições, esse rigor é uma feature apontada para nós mesmos.

Uma nota da casa, dita sem enfeite para que nunca se leia como desleixo: as linhas de install dos labs deste curso padronizam em `npm i`, porque o nosso toolchain de verificação colhe e repete elas. pnpm é o que o ecossistema Solana de TypeScript roda de verdade, então esta lição ensina isso como verdade de ecossistema e usa onde o workspace exige (`workspace:*` e `pnpm -r` são idiomas pnpm). Você está aprendendo as duas grafias de propósito: alfabetização em npm para qualquer lugar, fluência em pnpm para os repos que você vai ler de verdade.

### exports: a fronteira da API pública tornada literal

Antes de o mapa de exports existir na prática, "API pública" era um comentário e uma esperança. Qualquer um podia enfiar a mão nas tripas do seu pacote com `import { thing } from "pulse-core/src/classify"` e aí o seu layout interno de arquivos vira carga estrutural para estranhos. Renomeie um arquivo, quebre o mundo.

O campo `exports` proíbe exatamente isso. Ele é uma allowlist de entry points; qualquer coisa não listada não resolve, ponto final:

```json
{
  "exports": {
    ".": "./src/index.ts"
  }
}
```

Com esse mapa, `import ... from "pulse-core"` funciona e `import ... from "pulse-core/src/classify"` lança `ERR_PACKAGE_PATH_NOT_EXPORTED`. A sua superfície pública agora é um arquivo que você cura, `src/index.ts`, re-exportando exatamente o que os consumidores podem tocar. Todo o resto é privado por mecanismo em vez de por etiqueta. A gente exporta código-fonte TypeScript por ora porque todo consumidor neste workspace fala TypeScript; a lição de publish adiciona um passo de build e aponta esse mapa para a saída compilada, e nada na fronteira muda.

### engines, packageManager, e o runtime que você roda de verdade

`engines` declara quais runtimes você alega suportar. O verbo importa: alegar. Nada testa o seu código no Node 20 porque você digitou `>=20`. O mundo real mostra a lacuna: o ecossistema roda Node 24 (o LTS atual em 2026-09-02; a passagem de bastão para o Node 26 chega em 2026-10-28), enquanto os pisos das bibliotecas pesquisadas ficam em `>=20` ou `^22`, porque os mantenedores continuam suportando runtimes que os usuários deles ainda não deixaram. O próprio `@solana/kit` declara `engines.node >=20.18.0` enquanto os mantenedores dele certamente desenvolvem em algo mais novo.

Por que os pisos ficam duas linhas de LTS atrás do runtime? Porque um campo engines é a interseção de todo ambiente em que os usuários de um pacote ainda fazem deploy. Um time numa imagem base de Node 20 lê `>=20` como "isso não vai nos deixar na mão"; um bump por capricho corta esses times das correções sem razão técnica nenhuma. O piso se move quando o código precisa de uma API mais nova ou quando a linha antiga sai do LTS, não antes. O `>=20.18.0` do kit diz "a gente ainda carrega a frota que não migrou", uma gentileza deliberada e cara.

![Uma coluna mostra o runtime Node 24 que os desenvolvedores usam de verdade enquanto a outra empilha os pisos de suporte mais antigos que as bibliotecas ainda prometem, com a regra de alegar-o-que-você-testa embaixo.](assets/v02-comparison.webp)

Para o `pulse-core` a alegação honesta é a estreita: `"node": ">=24"`, porque o Node 24 é o único runtime que os nossos testes tocaram. Alargar para `>=20` sem testar no 20 seria decorar o contrato com uma promessa que ninguém verificou. Quando o CI ganhar uma matriz de versões (essa linha continua no M6), a alegação pode alargar para corresponder à evidência.

`packageManager` é um tipo diferente de pin: não qual runtime, mas qual gerenciador de pacotes e exatamente qual versão, fixável por hash, legível por ferramenta:

```json
{
  "packageManager": "pnpm@11.25.0"
}
```

Esse campo é a convenção de verdade do ecossistema. Na pesquisa por trás deste curso, 5 de 5 repos Solana em TypeScript pesquisados fixam o pnpm via `packageManager`, e o kit vai além: o repo dele bloqueia `npm install` e `yarn` de cara com uma trava de preinstall cujo nome diz tudo, "please-use-pnpm". Copie para um repo desses as linhas de lab `npm i` deste curso e ele vai te recusar. Leia o campo `packageManager` primeiro; o repo te conta as regras dele.

Agora a batida de honestidade sobre o caminho de install, porque ele mudou faz pouco e a maioria dos tutoriais não acompanhou. A ferramenta que historicamente auto-ativava o pnpm certo a partir desse campo era o corepack, que já vinha dentro do Node; em 2025-03-19 o TSC do Node votou pela saída dele (fora do Node 25+, restando só no Node 24 LTS). O runtime decidiu que gerenciadores de pacote não são trabalho dele, então a linha de install durável é a sem graça:

```bash
npm i -g pnpm@11.25.0
```

Fixada, explícita, funciona em todo Node que tem npm, que são todos. (Freshness de versão, e a resposta mudou enquanto este curso estava sendo escrito: 11.25.0 segurava a dist-tag `latest` em 2026-09-02, mas o pnpm 12 tomou ela em 2026-09-04, e uma nova checagem em 2026-09-06 lê `latest` = 12.3.4 com a linha 11 sobrevivendo sob `latest-11` em 11.26.0. O curso continua fixando 11.25.0 mesmo assim, de propósito: esse dígito tem que bater com o campo `packageManager` abaixo, com o job de CI do passo 7, e com o Dockerfile do M6, e `npm i -g pnpm@11.25.0` instala ele não importa para onde o `latest` tenha vagado. É para isso que serve um pin. Para um repo seu, rode `npm view pnpm dist-tags` e escolha deliberadamente.) `corepack enable` ainda funciona no seu laptop com Node 24 hoje e morre na próxima imagem base; um hábito com data de validade é um hábito ruim, e o Dockerfile do M6 vai usar a linha npm exatamente por isso.

![Uma linha do tempo vai da votação de remoção do corepack em 2025, passando pelo Node 25 largando ele, até o comando de install npm fixado que sobrevive à mudança.](assets/v03-timeline.webp)

### Faixas semver: quais futuros você aceita

Toda linha de dependência no `package.json` é uma faixa, e uma faixa é uma política sobre o futuro: quais versões, publicadas depois que você parou de olhar, o resolvedor tem permissão de te entregar? Quatro formas cobrem o uso de verdade.

**Exata** (`6.9.0`): esta versão e mais nada. Proteção máxima, zero correções.

**Caret** (`^6.9.0`): na base ou acima, dentro do mesmo major. `6.9.1`, `6.10.0`, `6.44.0` todas satisfazem; `7.0.0` nunca satisfaz. O caret confia na promessa central do semver, a de que mudanças que quebram só são entregues atrás de um bump de major, então ele para exatamente na fronteira onde a quebra tem permissão de morar. Essa é a forma que o npm escreve por padrão, e a forma que você vai ler mais.

**Tilde** (`~6.9.0`): na base ou acima, mesmo major E mesmo minor. Só caminhadas de patch: `6.9.4` sim, `6.10.0` não. A confiança mais apertada para quando você quer correções mas não features.

**Piso** (`>=6.9.0`): qualquer coisa na base ou acima, majors inclusive. Quase nunca o que um app quer, porque aceita futuros pelos quais o semver explicitamente se recusa a responder. Aparece em `engines` (onde "este runtime ou mais novo" é a intenção de verdade) muito mais do que em dependências.

Note quais dessas você de fato escreve. Quase nenhuma: `npm i some-dep` e `pnpm add some-dep` escrevem uma faixa caret para você, então a maioria das linhas de dependência na maioria dos repos é uma política que o autor nunca escolheu conscientemente. O default é defensável (correções fluem, majors bloqueados), mas times que querem pins exatos viram isso deliberadamente com `npm config set save-exact true` e se apoiam no lockfile mais uma ferramenta de update. As duas posturas são coerentes; derivar para uma delas porque uma ferramenta escreveu por você é a única opção errada.

E a armadilha dentro do caret, que merece parágrafo próprio porque ler `^` como "mais ou menos esta versão" uma hora vai te machucar: sob major zero, o minor vira o slot de quebra. `^0.3.9` admite `0.3.10` e recusa `0.4.0`, porque pacotes pré-1.0 reservam bumps de minor para mudanças que quebram e o caret respeita isso. `^0.3.9` e `^1.3.9` parecem irmãos e admitem futuros completamente diferentes. A regra tem mais um piso abaixo disso: com major E minor os dois zerados, o npm trata o patch como o slot de quebra, então `^0.0.3` admite `0.0.3` e mais nada. Os pacotes `@solana-program/*` que você vai encontrar no M8 moram na terra do 0.x, então essa regra não é curiosidade.

![Uma matriz mostra as faixas exata, tilde, caret e piso admitindo progressivamente mais versões a partir de uma base 6.9.0, com uma nota de rodapé sobre a regra do caret-zero.](assets/v04-comparison.webp)

Nomeie o trade-off antes de seguir, porque faixas são política e toda política custa alguma coisa. Pins apertados te protegem de majors surpresa e te deixam sem correções; faixas largas entregam correções e de vez em quando entregam uma quebra de terça-feira de manhã que você não agendou. De um jeito ou de outro a faixa no `package.json` é só metade da história: o lockfile registra a resolução exata que o seu install de fato produziu, e é ele o pin de verdade nos dois mundos. Essa linha, e o que o CI deve fazer com ela, é retomada direito em m09-l1.

### peerDependencies: a cláusula mais profunda

Dependências normais dizem "eu preciso disto, instale para mim." Peer dependencies dizem algo mais estranho e mais forte: "eu trabalho ao lado de uma dependência que VOCÊ fornece, NESTA faixa." Uma biblioteca que dá peer em `@solana/kit` está te dizendo que vai chamar as APIs do kit em tempo de execução mas se recusa a ser dona de qual instância do kit existe no seu app, porque tem que existir exatamente uma e ela tem que ser sua.

Por que recusar a propriedade, afinal? Suponha que o helius-sdk declarasse o kit como dependência normal. O seu app instala o próprio kit, o helius-sdk instala um segundo, privado, e todo valor que atravessa entre os dois (um objeto rpc, um tipo address) foi construído por uma cópia e inspecionado pela outra; checagens de identidade se desfazem com os erros menos úteis do ecossistema, porque o objeto É válido, só que da cópia errada. A declaração de peer diz "a gente tem que compartilhar a única instância"; a faixa diz quais instâncias ela de fato foi testada compartilhando.

Aqui vai o artefato ao vivo, sondado em 2026-09-02, não de memória:

```bash
npm view helius-sdk version peerDependencies
```

```
3.1.0
{
  '@solana-program/compute-budget': '^0.15.0',
  '@solana-program/stake': '^0.6.1',
  '@solana-program/system': '^0.12.0',
  '@solana-program/token': '^0.13.0',
  '@solana/kit': '^6.9.0'
}
```

Enquanto isso o latest do `@solana/kit` está em 8.2.0, dois majors à frente daquela faixa de peer `^6.9.0`. Instale `helius-sdk` e depois peça o kit latest no mesmo pacote, e o resolvedor do npm recusa com um erro `ERESOLVE`. O lab dispara isso de propósito, porque a recusa protege algo de verdade: o helius-sdk chama APIs do kit da linha 6.x contra a qual ele foi testado, e forçar o kit 8 ao lado dele só roda uma biblioteca contra uma superfície de API que ela nunca viu, movendo a falha do tempo de install (barulhenta) para o runtime (silenciosa, em produção).

![Um fluxograma traça o npm checando uma versão do kit solicitada contra uma faixa de peer instalada e se ramificando numa recusa segura ou num install forçado arriscado.](assets/v05-flowchart.webp)

Então o que você fixa de verdade? Não o mais novo de tudo, e não um dígito que alguém decorou. A regra, e é a frase mais durável do curso inteiro: **fixe aquilo contra o que as suas deps dão peer, por workspace.** Leia as faixas de peer das suas dependências, e dê a cada workspace a versão em que essas faixas concordam. Por workspace importa porque a frota, o painel e um futuro bot são pacotes separados com conjuntos de dependências separados; um único dígito para o repo inteiro é como você fabrica um conflito que pacote nenhum tem sozinho.

Por que uma regra em vez de um número? Porque os dígitos apodrecem numa escala de tempo que os próprios timestamps do npm provam: o kit entregou 6.10.0 em 2026-06-16, 7.0.0 em 2026-06-30, 8.0.0 em 2026-08-21. Qualquer wiki que congelou "use o kit 6" estava errada duas vezes antes de a estação do ano mudar. As faixas de peer no seu `node_modules` de verdade são o único conselho de versão que se atualiza sozinho. A lição dois do M8 constrói o workspace Solana onde essa regra vira o passo de setup.

![Marcadores de release mostram os majors sete e oito do kit chegando com semanas de diferença enquanto a faixa de peer de uma biblioteca fica ancorada na linha seis abaixo deles.](assets/v06-chart.webp)

### Leia um manifest do mundo real antes de escrever o seu

A habilidade que esta lição está de fato instalando é alfabetização em manifest, então feche a teoria lendo um de verdade, só com os olhos. Abra qualquer repo Solana sério em TypeScript (o kit é o que este curso não para de citar) e leia o `package.json` dele fazendo uma pergunta por campo: o que esta linha está prometendo, e para quem? Você vai achar o pin de `packageManager` (todos os cinco repos pesquisados carregam um), um piso de engines mais velho que o seu Node (dado de público, não negligência), uma trava de preinstall recusando o gerenciador de pacotes errado, e mapas de `exports` com entradas condicionais por ambiente (profundidade de bookmark, não ensinada hoje). Dois minutos disso por repo desconhecido: o repo te conta as regras dele antes de você rodar um comando nele.

**Vá mais fundo (os 20%).** esta lição ensinou os campos que o nosso caminho de entrega exercita e o porquê de cada um. O resto da superfície do npm, semântica de scripts, precedência de config, flags de publicação, dist-tags, casos de borda do protocolo de workspace, é material de verdade que a gente deliberadamente deixa como bookmark em vez de reensinar. O caminho canônico é o material de gerenciadores de pacote da trilha de aprendizado do Node.js: [Uma introdução ao gerenciador de pacotes npm](https://nodejs.org/learn/getting-started/an-introduction-to-the-npm-package-manager) (URL sondada em 2026-09-02). Leia depois do lab, não no lugar dele; nada abaixo depende disso.

## Lab: extraia o pulse-core

O recuo da ajuda, em voz alta: os passos 1 a 4 são totalmente trabalhados, diffs na tela, porque a primeira jogada de extração é a receita. Os passos 5 e 6 te entregam a mesma receita sem narração para os módulos restantes; o passo 7 refaz a fiação do pipeline de CI como um diff trabalhado, porque quebrar o batimento da estação não é lugar para praticar. A repetição de diagnóstico do passo 8 e o challenge depois dela são inteiramente seus. Até m03-l4 você vai fazer essa dança sem a partitura.

1. **Instale o pnpm, fixado.** Primeira ferramenta da lição, então aqui vai o install dela (freshness: `latest` era 11.25.0 em 2026-09-02; confira de novo com `npm view pnpm version`):

   ```bash
   npm i -g pnpm@11.25.0
   pnpm --version
   ```

2. **Declare o workspace e mova a frota para dentro dele.** A partir da raiz do repo, crie o layout e realoque tudo que tem forma de frota para `packages/pulse-fleet` (os seus nomes de arquivo podem ser outros; mova o que você tem):

   ```bash
   mkdir -p packages/pulse-fleet packages/pulse-core/src
   git mv src tests package.json tsconfig.json pulse.config.json probe.ts fleet.ts smoke.ts packages/pulse-fleet/
   ```

   Três observações sobre a jogada:

   - `tests` está na lista: a suíte de m02-l4 importa `../src/config.js` e lê `./fixtures/`, então ela tem que continuar irmã de `src/` ou o checkpoint abaixo roda zero testes.
   - Os três arquivos `.ts` soltos são os scripts de nível raiz da frota. Tudo que tem forma de frota se move; `status.json` e `.github/` ficam na raiz de propósito.
   - Se um arquivo listado não existir no seu repo, tire ele do comando em vez de deixar o `git mv` recusar o batch inteiro.

   Depois abra o `packages/pulse-fleet/package.json` movido e faça duas edições:

   1. Defina `"name"` como `"pulse-fleet"`. O manifest raiz abaixo está prestes a reusar o nome antigo para a cola privada, e o pnpm chaveia tudo pelo campo name, não pelo diretório: prefixos de `pnpm -r`, seletores `--filter` (o passo 7 precisa de um), e o build-skip do Vercel de m03-l3 todos querem um nome único por pacote.
   2. Substitua o stub do npm-init em `scripts` por `"test": "vitest run"`. m02-l4 rodava a suíte como `npx vitest run` e nunca precisou do script; o `pnpm -r` abaixo roda o script `test` de cada pacote, e sem essa linha ele rodaria o stub, que imprime `Error: no test specified` e sai com 1.

   Crie o `pnpm-workspace.yaml` na raiz:

   ```yaml
   packages:
     - "packages/*"
   ```

   E um `package.json` raiz mínimo (a raiz herda o nome antigo de nível de repo, `pulse-station`, liberado pela renomeação que você acabou de fazer; ele é cola privada, nunca publicada):

   ```json
   {
     "name": "pulse-station",
     "private": true,
     "packageManager": "pnpm@11.25.0"
   }
   ```

   O seu diretório `.github/workflows` fica na raiz, porque o Actions só lê workflows de lá. Mas note o que a jogada fez com o pipeline: todo job em `pulse.yml` ainda roda `npm ci` contra uma raiz cujo `package.json` agora é cola privada sem lockfile. Dê push agora mesmo e os três jobs ficam vermelhos; isso é esperado, e o passo 7 refaz a fiação antes de qualquer coisa ir para o push. Já que você está aqui, apague o `package-lock.json` velho na raiz: o pnpm escreve o próprio `pnpm-lock.yaml` no próximo install, e esse arquivo (commite ele) é o pin de verdade do workspace daqui em diante. Checkpoint: `pnpm install` a partir da raiz completa e `pnpm -r test` roda a suíte de m02-l4 verde na casa nova dela. Um provável obstáculo: o pnpm 11 se recusa a rodar scripts de build de dependências em que não mandaram ele confiar, então o primeiro install pode abortar com `ERR_PNPM_IGNORED_BUILDS` nomeando `esbuild` (o motor do vitest, que compila um binário nativo no install). O conserto é um comando, `pnpm approve-builds esbuild`, que registra uma entrada `allowBuilds` no `pnpm-workspace.yaml`; commite esse arquivo e rode o install de novo. (Documentação mais antiga menciona uma chave `onlyBuiltDependencies`; o pnpm 11.25 ignora ela silenciosamente, então use o comando.) Nada foi extraído ainda; a gente só provou que a jogada não quebrou nada antes de mudar qualquer outra coisa.

3. **Crie o pulse-core e mova o primeiro módulo do motor.** O classificador e a união dele vão primeiro, porque eles são o módulo que a próxima lição importa. Se você seguiu a consolidação de m02-l4, tudo isso mora num arquivo só, `src/classify.ts`: a união `ProbeResult`, `parseProbe`, o `classify` na forma de união, o `classifyProbe` na forma de fronteira, e `assertNever`. (O `probe.ts` da raiz é o wrapper de CLI em volta deles; é fiação de app e já se moveu junto com a frota no passo 2.) Um arquivo, uma jogada:

   ```bash
   git mv packages/pulse-fleet/src/classify.ts packages/pulse-core/src/
   ```

   Dê ao `pulse-core` o contrato dele, todo campo da seção de teoria preenchido com honestidade:

   ```json
   {
     "name": "pulse-core",
     "version": "0.1.0",
     "private": false,
     "type": "module",
     "exports": {
       ".": "./src/index.ts"
     },
     "engines": {
       "node": ">=24"
     },
     "packageManager": "pnpm@11.25.0"
   }
   ```

   E a superfície pública curada, `packages/pulse-core/src/index.ts`:

   ```ts
   export type { ProbeResult, Verdict } from "./classify.js";
   export { classify, classifyProbe, parseProbe, assertNever } from "./classify.js";
   ```

   Aquele `.js` nos especificadores não é typo e não é opcional: sob a resolução `nodenext` que os tsconfigs deste curso rodam, imports ESM relativos têm que nomear a extensão emitida, exatamente como todo arquivo de m02 já fazia. Escreva `"./classify"` pelado e o vitest ainda vai rodar feliz (o bundler dele resolve mais frouxo), o que torna o erro extra traiçoeiro: a primeira ferramenta a recusar é `tsc --noEmit`, com um TS2835 por import, na trava de CI do passo 7, dois passos de distância do arquivo que você digitou errado.

![Cada campo do manifest do pulse-core carrega uma nota de margem explicando a promessa que aquela linha faz a ferramentas e consumidores.](assets/v07-annotated-code.webp)

4. **Ligue a frota para importar através da fronteira.** Em `packages/pulse-fleet/package.json`, adicione a dependência de workspace:

   ```json
   {
     "dependencies": {
       "pulse-core": "workspace:*"
     }
   }
   ```

   Depois rode `pnpm install` a partir da raiz para criar o symlink, e atualize todo import da frota do módulo movido. A CLI de sonda em `packages/pulse-fleet/probe.ts`:

   ```ts
   // before
   import { classify, type ProbeResult } from "./src/classify.js";

   // after
   import { classify, type ProbeResult } from "pulse-core";
   ```

   Note o efeito colateral da fronteira sobre a grafia: imports relativos dos seus próprios arquivos precisam da extensão `.js`, mas um especificador de pacote pelado nunca carrega uma; o mapa de exports resolve isso. Os seus arquivos de teste mudam essa mesma única linha e mais nada (`../src/classify.js` vira `pulse-core`), então a suíte de m02-l4 agora exercita o `pulse-core` através da fronteira pública dele, exatamente do jeito que o painel da próxima lição vai fazer. Checkpoint: `pnpm -r test` verde de novo, e a saída te ensina como o `-r` pensa: o script `test` de cada pacote do workspace, saída prefixada com o nome do pacote, com esta forma:

   ```
   Scope: all 2 workspace projects
   packages/pulse-fleet test$ vitest run
   ...
   Test Files  3 passed (3)
        Tests  14 passed (14)
   ```

   As suas contagens vão bater com o que quer que a sua suíte de m02-l4 tenha virado; a forma é o que reconhecer. Só o `pulse-fleet` roda testes porque só ele tem um script `test`, e isso está ok hoje: a suíte atravessa a fronteira, então o core é exercitado, e quando o `pulse-core` ganhar o próprio script o `-r` pega ele com zero config. Se o TypeScript não conseguir resolver `pulse-core`, você pulou o `pnpm install` da raiz que cria o link; se ele resolver mas reclamar do entry point, o seu caminho de `exports` não bate com onde o `index.ts` está.

5. **Mova os módulos restantes do motor você mesmo.** Os helpers de backoff de m02-l3 pertencem ao core (o worker de edge do M7 vai querer eles; a config zod não se move, porque parsear config é fiação de app, não motor). Mesma receita dos passos 3 e 4: `git mv` no arquivo, re-exporte as peças públicas pelo `index.ts`, atualize os imports da frota, `pnpm -r test`. Sem diff dado desta vez; você tem o padrão.

![Um loop de cinco passos move um módulo, re-exporta ele, refaz a fiação dos imports, reinstala e testa, repetindo por módulo até o código de motor morar só no pacote core.](assets/v08-flowchart.webp)

6. **Prove que a fronteira proíbe a porta dos fundos.** De qualquer arquivo da frota, tente o import profundo que o mapa de exports existe para matar, `import { classifyProbe } from "pulse-core/src/classify"`, e rode os testes. A recusa veste duas fantasias: pelo vitest você recebe a frase do Vite, `"./src/classify" is not exported under the conditions ["node", "development", "import"]`, enquanto a resolução pura do Node (rode o import pelo `npx tsx` e olhe) lança o canônico `ERR_PACKAGE_PATH_NOT_EXPORTED`. Mesma lei, dois tribunais; reconheça as duas grafias, depois apague a linha. Aceitação da extração: `pnpm -r test` verde a partir da raiz, e `grep -r "classifyProbe" packages/pulse-fleet/src` mostra só linhas de import, zero corpos de função. Código de motor mora em exatamente um lugar.

7. **Refaça a fiação do pipeline (o terceiro consumidor do workspace).** O aviso do passo 2 vence: os jobs do `pulse.yml` ainda instalam com `npm ci` e rodam as ferramentas deles a partir de uma raiz que não guarda mais a frota. Ensine ao workflow o layout que você acabou de ensinar a si mesmo. Os três jobs trocam `npm ci` por um par de linhas; o job probe, o único que carrega uma linha `cache: npm`, apaga ela também:

   ```yaml
         - uses: actions/setup-node@v7
           with:
             node-version: 24              # CHANGED (probe job): cache: npm line deleted
         - run: npm i -g pnpm@11.25.0        # CHANGED: was `npm ci`
         - run: pnpm install --frozen-lockfile
   ```

   A deleção do cache não é limpeza opcional: `cache: npm` faz o setup-node sair procurando `package-lock.json`, o arquivo que o passo 2 deliberadamente apagou, e num runner de verdade ele falha duro com `Dependencies lock file is not found` antes de o primeiro step `run:` daquele job executar. Faça grep no seu próprio `pulse.yml` antes de editar: essa linha existe em exatamente um lugar, o job probe que m01-l3 escreveu, porque os jobs typecheck e test que m02-l4 adicionou nunca ganharam ela. E note quando a falha chegaria até você, porque `needs: [typecheck, test]` decide isso: o job probe não começa até as duas travas ficarem verdes, então essa aqui aparece como uma execução vermelha cujos dois primeiros jobs passaram, vários minutos adentro. (`cache: pnpm` existe, mas o setup-node pergunta ao pnpm o caminho do store dele, então só funciona se o pnpm estiver instalado ANTES do setup-node; largar a linha é o mínimo honesto hoje.) Depois aponte cada job para o workspace: a trava de typecheck vira `- run: pnpm --filter pulse-fleet exec tsc --noEmit`, a trava de test vira `- run: pnpm -r test` (o comando exato que você vem rodando localmente), e os steps de run do job probe ganham `working-directory: packages/pulse-fleet`, com um contrato deliberadamente inalterado: `status.json` fica na RAIZ DO REPO. Esse caminho é carga estrutural, porque o painel de m03-l2 e a config de m03-l3 os dois buscam o arquivo raw em `.../main/status.json`, então aponte a escrita da frota para a raiz (escreva em `../../status.json`, ou receba o caminho de saída como argumento) e mantenha o step de commit adicionando `status.json` a partir da raiz do repo; se uma segunda cópia algum dia aparecer dentro de `packages/pulse-fleet/`, a escrita está mirada errado e o painel renderizaria silenciosamente a cópia congelada da raiz. `--frozen-lockfile` é o trabalho do `npm ci` na grafia pnpm: pegue exatamente o que o `pnpm-lock.yaml` registrou ou falhe alto. Commite a mudança do workflow junto com o `pnpm-lock.yaml`, dê push, e assista à execução. Checkpoint: as duas travas verdes, o job probe commita um `status.json` novo, e as arestas `needs: [typecheck, test]` de m02-l4 sobrevivem intactas. O pipeline é o terceiro consumidor da extração, e porque o `-r` percorre o que quer que o workspace declare, todo pacote futuro já está dentro da trava.

8. **A repetição de diagnóstico (sem guia).** Num diretório de rascunho, reproduza o conflito da seção de teoria com as suas próprias mãos e leia a recusa:

   ```bash
   mkdir peer-scratch && cd peer-scratch
   npm init -y
   npm i helius-sdk
   npm i @solana/kit@latest
   ```

   O primeiro install dá certo (o npm instala automaticamente os peers declarados, todos das linhas compatíveis com 6.x). O segundo recusa, mais barulhento que a saída arrumadinha de `npm view` da seção de teoria: espere umas trinta linhas `npm warn ERESOLVE overriding peer dependency` primeiro (a linha do helius que você conhece, `peer @solana/kit@"^6.9.0" from helius-sdk@3.1.0`, passa rolando no meio delas), e então o bloco que importa, abrindo com `npm error code ERESOLVE`. Ele nomeia o conflito através dos peers TRANSITIVOS do helius-sdk, linhas com a forma `peer @solana/kit@"^6.4.0" from @solana-program/system@0.12.2`, e fecha com `Conflicting peer dependency: @solana/kit@6.10.0`, o kit mais novo com que a árvore inteira consegue concordar: os pacotes `@solana-program/*` em que o helius-sdk dá peer carregam as próprias faixas de kit, e qualquer um deles basta para recusar o kit 8. O seu entregável é UMA frase escrita diagnosticando o conserto em termos da regra de pin. Ela tem que nomear a faixa e a regra; um dígito de versão pelado como resposta vai estar velho antes de o módulo acabar.

## Challenge

A lógica de semver que você acabou de usar no olho vira código: implemente `satisfiesRange(version, range)` para as quatro formas de faixa que esta lição ensinou, exata, `>=`, tilde e caret, incluindo a regra do caret-zero em que o major 0 faz do minor o slot de quebra. `parseSemver` e `compare` são dados no starter, no painel coding-challenge da página desta lição; o braço de correspondência exata já está pronto para você. Onze testes avaliam isso, um deles o lab desta lição em miniatura: `7.0.2` satisfaz `^6.9.0`? A sua implementação deve concordar com o resolvedor do npm nessa chamada: não satisfaz. O treino não é todo o npm: a regra do caret para no piso do major zero; a sub-regra `^0.0.z`, em que o patch vira o slot de quebra, não é modelada nem testada aqui, então não trate o treino como uma reimplementação completa do matcher do npm. As hints escalam de ordenação de operadores até o ramo do caret-zero; gaste elas em ordem.

Uma nota de design antes de você começar, a primeira hint disfarçada: a ORDEM em que você testa os operadores é carga estrutural. Cheque o `>=` antes de qualquer coisa de um caractere só, ou você vai cortar o prefixo errado e todo teste de piso falha de uma vez, um bug de string com cara de bug de lógica. Depois deste challenge, um caret em qualquer manifest é algo que você computa, não algo que você aperta os olhos para ler: a diferença entre ler um conflito de peer e ser lido por um.

## Checkpoint, e o primeiro consumidor de fora

O que você já consegue fazer: ler qualquer `package.json` e dizer o que cada campo promete e para quem; rodar um workspace pnpm de dois pacotes onde uma suíte verde prova que a estrutura mudou e o comportamento não; diagnosticar um conflito de peer lendo faixas em vez de passar por cima delas na base da flag de força. A recuperação de 30 segundos, em voz alta: o que `^6.9.0` admite, e onde ele para? (Qualquer 6.x na 6.9.0 ou acima. Nunca 7. Com uma base `^0.9.0`, a parada se move para o minor.)

Dois pedidos enquanto está fresco: se um passo do lab brigou com você, anote ONDE ele brigou (o symlink? o caminho de exports? o bloco ERESOLVE?) nas suas notas do curso, e se a frase do conflito de peer levou mais de uma tentativa, guarde os seus rascunhos falhos; o M8 vai te mostrar o mesmo diagnóstico com apostas mais altas e a sua redação antiga é evidência útil de como o seu modelo melhorou.

`pulse-core` agora é um pacote de verdade com uma fronteira de verdade, e a fronteira recebe o primeiro teste de fora dela na hora: na próxima lição um painel React importa o classificador através dela e renderiza o `status.json` do cron para olhos humanos, o que quer dizer que a extração que você acabou de fazer para de ser um argumento e vira um pixel. Vejo você na renderização.
