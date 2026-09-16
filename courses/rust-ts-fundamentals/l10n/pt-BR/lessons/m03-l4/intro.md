# Publique algo de verdade: tsdown e os sinais de uma dep morrendo

## Resumo

A lição passada entregou a primeira URL: pulse-board ao vivo no Vercel, cutucado do celular de um estranho. A estação tem um rosto público, mas ainda não um motor público: o `pulse-core`, o classificador com que toda parte da estação concorda, ainda vive só dentro do seu workspace, importável pelos seus pacotes e por mais ninguém. Hoje é a volta olímpica do módulo. Você constrói o `pulse-core` com uma ferramenta de build de verdade, lê o tarball que você está prestes a entregar para o mundo, publica ele no npm como um pacote público com escopo, e prova que ele instala para um estranho. Depois a lição ensina a habilidade escondida dentro da escolha da ferramenta: ler se uma dependência está viva antes de adotar ela, usando a ferramenta de build que a gente acabou de escolher como o exemplo trabalhado, porque a ferramenta que ela substituiu está ensinando essa lição sobre si mesma no próprio README dela.

A maioria dos devs publica o primeiro pacote sem nunca olhar dentro dele. Você não vai ser um deles. Faça isto agora mesmo, a partir da raiz do workspace:

```bash
cd packages/pulse-core
pnpm add -D tsdown@0.22.14 typescript
npx tsdown src/index.ts
npm pack --dry-run
```

Esse pin está atual em 2026-09-02; o tsdown já tem um release candidate 0.23 tagueado, então espere um dígito mais alto na hora em que você digitar isto, e aceite o que `pnpm add -D tsdown` te der se o pin tiver envelhecido. A linha do `typescript` está ali porque o layout estrito do pnpm quer dizer que um pacote tem que declarar o que o build dele usa, mesmo quando a raiz do workspace já tem.

O último comando imprimiu uma listagem de arquivos. Leia ela devagar, cada linha, como um estranho leria, porque daqui a uns vinte minutos um estranho PODE. A minha mostrou `dist/index.mjs`, `package.json`, e depois todo arquivo de `src/` pegando carona sem ser convidado (três num repo que seguiu os labs: `index.ts`, o classificador, os helpers de backoff). A sua vai ser parecida. Repare no que NÃO está lá também: nenhuma declaração de tipo, o que para um pacote cujo valor inteiro são os tipos de união dele seria uma catástrofe silenciosa; a seção de build abaixo explica por que a rodada sem config não conseguiu emitir elas e conserta isso. Essa listagem é o conteúdo exato do que o `npm publish` subiria hoje, e hoje é uma bagunça. A primeira metade inteira desta lição é transformar essa listagem num contrato.

O recuo da ajuda, dito em voz alta: a configuração de build e a fiação dos exports a gente faz junto, percorrida linha por linha. O publish e a prova do projeto de rascunho você roda sozinho a partir de um checklist. O treino de fechamento, ler os sinais vitais de três pacotes de verdade, é totalmente sem guia, e é o último degrau da escada do tier de TypeScript. O próximo módulo recomeça a escada de baixo para uma linguagem nova.

## O tarball e os sinais vitais

### O que um passo de build compra, e o que é o tsdown

Até agora o `pulse-core` entregava código-fonte TypeScript cru e se safava, porque todo consumidor vivia no mesmo workspace e falava TypeScript pelas mesmas ferramentas. m03-l1 disse que a lição de publish ia mudar isso, e esta é a lição de publish. Não dá para presumir que o projeto de um estranho compile os seus arquivos `.ts`; alguns runtimes experimentam rodar TypeScript direto, mas uma biblioteca publicada que exige isso encolheu o público dela sem motivo. Então um pacote a caminho do registro entrega dois artefatos: JavaScript compilado para todo runtime, e declarações de tipo `.d.ts` para que consumidores TypeScript mantenham toda garantia que os tipos de união conquistaram no M2. Uma fonte, duas saídas, e uma ferramenta cujo trabalho inteiro é emitir as duas corretamente.

Essa ferramenta, para a gente, é o tsdown. Ele é o sucessor do tsup, o default que reinou por muito tempo exatamente neste trabalho, e a razão de a gente passar por cima do incumbente é a segunda metade desta lição, então segure a pergunta por algumas seções. Mecanicamente: o tsdown empacota o seu entry point com o Rolldown, o bundler em Rust que também move o Vite 8, e consegue emitir declarações ao lado do JavaScript. Você já rodou ele uma vez sem config nenhuma. Duas observações daquela rodada valem ser fixadas antes de a gente configurar ele.

Primeiro, ele escreveu `index.mjs`, não `index.js`. O tsdown usa por padrão extensões que gritam "ESM" independente do que o `package.json` ao redor diz, o que é engenharia defensiva para pacotes que entregam os dois formatos. O nosso pacote declarou `"type": "module"` lá em m03-l1, `.js` puro já é ESM aqui, e casar com a extensão que o resto do nosso workspace espera mantém o mapa de exports chato. Então a gente desliga esse default.

Segundo, as declarações faltando. Emitir declarações é trabalho do TypeScript, o tsdown só orquestra isso, e a maquinaria do TypeScript se recusa a rodar sem um `tsconfig.json` no pacote. O `pulse-core` nunca teve um: a extração de m03-l1 moveu o único tsconfig do repo para dentro do `pulse-fleet` junto com todo o resto, e nada desde então precisou de um aqui, porque o vitest e os consumidores do workspace todos resolviam código-fonte cru. O passo de build é onde essa carona grátis acaba. Peça declarações sem um tsconfig e o build morre com `ERROR Error: tsgo generator requires a tsconfig file to be specified.` Então o pacote ganha primeiro o próprio contrato-com-o-compilador, `packages/pulse-core/tsconfig.json`, mínimo e estrito, casando com as configurações que a frota vem usando o curso inteiro:

```json
{
  "compilerOptions": {
    "target": "es2023",
    "module": "nodenext",
    "moduleResolution": "nodenext",
    "strict": true,
    "declaration": true,
    "skipLibCheck": true
  },
  "include": ["src"]
}
```

Agora a configuração do tsdown, `packages/pulse-core/tsdown.config.ts`:

```ts
import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/index.ts"],
  format: "esm",
  dts: true,
  fixedExtension: false,
});
```

Quatro linhas, cada uma ganhando o lugar dela: o entry é o mesmo `src/index.ts` curado que é a superfície pública desde a extração, `format` é ESM porque este curso entrega um único sistema de módulos, `dts` pede declarações (que é por que o tsconfig acima tinha que existir), e `fixedExtension: false` é o opt-out que nos dá `.js` e `.d.ts`. Uma nota cosmética para esta rodada e toda depois dela: espere uma linha `WARN TypeScript 7.0 does not yet have a stable API and is experimental`. No momento em que escrevo (2026-09-02, tsdown 0.22.14 contra typescript 7.0.2, o estável atual) é exatamente o que ela diz, um aviso; as declarações emitem bem. Anote a versão e siga em frente. Adicione o script ao `packages/pulse-core/package.json`:

```json
{
  "scripts": {
    "build": "tsdown"
  }
}
```

Rode `pnpm build` a partir do diretório do pacote e `dist/` agora guarda `index.js` e `index.d.ts`. Saída total para o nosso motorzinho: dois ou três kilobytes. Pequeno está certo; este pacote é três módulos de lógica pura, e o tamanho do tarball está prestes a virar algo que você lê, não algo que você chuta.

![O código-fonte flui pelo tsdown até dist, é empacotado num tarball e é entregue ao registro, com uma inspeção de dry run antes do upload.](assets/v01-flowchart.webp)

### A dupla preocupação: tipos têm que viajar junto com o JavaScript

O mapa de exports é a porta da frente do pacote desde m03-l1: uma allowlist de entry points, imports profundos recusados com `ERR_PACKAGE_PATH_NOT_EXPORTED`. Por enquanto ele aponta para o código-fonte TypeScript. Aponte ele para a saída do build no lugar, e repare que o valor de `"."` cresce de uma string para um objeto com duas condições:

```json
{
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "default": "./dist/index.js"
    }
  }
}
```

Esta é a dupla preocupação no coração de publicar pacotes tipados, e ela merece a ênfase. Sob resolução moderna (a família `nodenext` que os tsconfigs dos seus consumidores usam), o TypeScript percorre o mapa de exports para achar tipos, o mesmo mapa que o Node percorre para achar JavaScript. Dois resolvedores, um mapa. A condição `types` tem que ficar bem ali ao lado da entrada de JavaScript que ela descreve, e a ordem importa: `types` vem primeiro no objeto de condições, porque resolvedores pegam a primeira condição que casa e `default` casa com tudo.

Pule isso e você fabrica o bug report mais confuso que um autor de biblioteca recebe: o JavaScript importa e roda perfeitamente, e os consumidores TypeScript não conseguem achar o seu módulo. Eu quebrei isso de propósito num projeto de rascunho enquanto escrevia esta lição, estacionando as declarações em algum lugar para onde o mapa não aponta, e vale a pena ler o erro inteiro porque um dia um consumidor vai colar ele na sua cara:

```text
error TS7016: Could not find a declaration file for module '@kaue/pulse-core'.
  There are types at '.../node_modules/@kaue/pulse-core/types/index.d.ts',
  but this result could not be resolved when respecting package.json "exports".
  The '@kaue/pulse-core' library may need to update its package.json or typings.
```

Leia a linha do meio duas vezes. O TypeScript ACHOU as declarações. Ele se recusou a usar elas, porque quando um mapa de exports existe ele governa tudo, e um arquivo que o mapa não expõe poderia muito bem não existir. O JavaScript continuou rodando o tempo inteiro. Essa assimetria, JS funcionando com tipos invisíveis, é exatamente o que a prova do projeto de rascunho no lab existe para pegar antes que um estranho pegue.

![O mesmo pacote funciona para um consumidor Node dos dois jeitos, mas consumidores TypeScript perdem todos os tipos quando o mapa de exports omite a condição types dele.](assets/v02-comparison.webp)

Uma consequência de reapontar o mapa, dita sem rodeios para ela nunca te surpreender: os consumidores do seu workspace agora resolvem `dist/` também. O painel e a frota precisam do `pulse-core` construído antes de as ferramentas deles conseguirem ver ele, então um clone fresco roda o build do core uma vez antes de qualquer outra coisa, e enquanto você estiver editando ativamente o código do core você deixa `npx tsdown --watch` rodando para os consumidores sempre verem saída fresca. E "clone fresco" não é hipotético: dois dos seus consumidores automatizados são clones frescos em toda execução, então ligue o build nos dois AGORA, antes de o próximo push deixar eles vermelhos. Em `.github/workflows/pulse.yml`, adicione um step logo depois da linha `pnpm install --frozen-lockfile` nos três jobs:

```yaml
      - run: pnpm --filter pulse-core build
```

(O filtro casa com o campo `name` do pacote; quando o lab renomear o core para o seu escopo npm, atualize os três filtros para o nome com escopo.) E para o deploy do Vercel conectado ao git, que constrói o painel a partir do próprio clone fresco dele, prefixe o script de build do painel em `packages/pulse-board/package.json` para o core ser sempre construído primeiro:

```json
{
  "scripts": {
    "build": "pnpm --filter pulse-core build && tsc -b && vite build"
  }
}
```

Essa única edição também conserta o primeiro `npm run build` local de qualquer colaborador futuro. Pule uma das duas fiações e a falha chega no próximo push, nos logs de outra pessoa, a duas superfícies de distância deste parágrafo. Esse é o custo honesto de um mapa servindo os dois públicos. O padrão mais fundo, deixar o workspace resolver código-fonte enquanto só o artefato publicado aponta para dist, existe nos overrides de `publishConfig` do pnpm, e é território de bookmark, não o caminho de hoje.

### O tarball é o produto

Agora conserte a listagem que você leu lá em cima. O que o `npm publish` sobe é exatamente o que o `npm pack` monta, e o que o npm empacota é, por padrão, quase tudo na pasta: código-fonte, arquivos de config, anotações perdidas, qualquer coisa com cara de `.env` que tenha entrado ali. Nada é removido depois. O tarball É o entregável; o que a listagem mostrar é o que aterrissa, byte por byte, no `node_modules` de todo consumidor, e graças à política fortemente restrita de unpublish do npm isso é efetivamente para sempre. O conserto é o campo `files`, uma allowlist, a mesma filosofia do mapa de exports uma camada abaixo:

```json
{
  "files": ["dist"]
}
```

O `package.json` em si, o README e arquivos de licença sempre são entregues de qualquer jeito, que é o que você quer. Rode o pré-voo de novo, `npm pack --dry-run`, e a listagem encolhe para três entradas:

```text
npm notice 📦  pulse-core@0.1.0
npm notice Tarball Contents
npm notice 510B dist/index.d.ts
npm notice 607B dist/index.js
npm notice 374B package.json
```

Os tamanhos vão diferir; o formato não deveria. Saída compilada, declarações, manifest, nada mais. Nenhum `src/`, nenhuma config, nenhum arquivo de teste, nada com cara de ambiente. Esse hábito de dry-run-antes-do-publish custa vinte segundos e é o hábito mais barato de profissionalismo-e-segurança deste curso. A história do que acontece quando o lado do registro disso dá errado é a abertura da lição de auditoria de dependências lá no fim do curso; por enquanto, a regra basta: leia a listagem, toda vez, antes de o tarball virar permanente.

Dois fatos sobre publicar completam o padrão. Primeiro, nomes: `pulse-core` como nome pelado pertence a quem registrou primeiro, então você publica sob o seu escopo, o prefixo `@username/` que toda conta npm ganha de graça, onde você é dono do namespace por inteiro. Segundo, uma pegadinha do lab que vale antecipar: pacotes com escopo vêm PRIVADOS por padrão no primeiro publish, pacotes privados são uma feature paga, e por isso um `npm publish` pelado de um pacote com escopo numa conta gratuita falha com um erro sobre pagamento que parece um bug de cobrança. Não é. É o npm perguntando qual visibilidade você quis dizer. A flag `--access public` é a resposta, e esquecer dela é um rito de passagem do qual este parágrafo acabou de te salvar.

**Vá mais fundo (os 20%).** esta lição ensinou a mecânica de publish que o nosso artefato exercita: build, exports, files, pack, publish. O resto do mundo de entregar uma biblioteca, watch workflows, múltiplos entry points, alvos de plataforma, escolhas de unbundling, vive na [documentação do tsdown](https://tsdown.dev/guide/) (URL sondada em 2026-09-02), e a camada de automação acima de tudo isso, changesets, publicação dirigida por CI, atestados de proveniência, é maquinaria de verdade que este curso deliberadamente sinaliza em vez de ensinar. Nada no lab depende de nada disso.

### Lendo os sinais vitais de uma dependência

Agora a pergunta que eu deixei estacionada: por que tsdown e não tsup, a ferramenta com anos de reinado e, até hoje, mais downloads?

Porque o próprio README do tsup responde ela na primeira frase. No topo, acima do nome do projeto, fica um aviso do mantenedor: "Este projeto não é mais mantido ativamente. Considere usar o tsdown no lugar." Uma frase, escrita pela pessoa que saberia, em 2025, e pode ser a frase mais honesta que o ecossistema produziu naquele ano. Os números em volta dela deixam o estudo de caso perfeito. O último publish do tsup é 8.5.1, datado de 2025-11-12. Os downloads dele na semana encerrada em 2026-08-29: uns 8.5 milhões. O tsdown, o sucessor nomeado, publicando ativamente, mesma semana: uns 5.7 milhões. A ferramenta abandonada ainda tem mais downloads que a própria substituta dela, dez meses depois do release final dela, e repos sérios ainda dependem dela; tanto o workspace do kit da anza quanto o SDK gill fazem o build com tsup hoje.

Sente-se com essa tensão, porque ela é a meta-habilidade desta lição inteira: contagens de download são um indicador atrasado com anos de inércia embutidos. Todo lockfile, tutorial e template existente continua puxando tsup muito depois de o autor dele ter dito a todo mundo para ir embora. O voto do mercado é informação velha. O README é a de hoje. Você está sempre a uma frase de README de distância de uma dependência abandonada, e a habilidade inteira é saber procurar a frase antes de instalar, não depois.

E uma coisa merece ser dita em defesa do tsup, porque a lição é sobre ler sinais, não sobre zombar dos caídos: aquele aviso é BOA manutenção. O autor entregou uma ferramenta em que o ecossistema inteiro se apoiou, e quando ele parou de mantê-la disse isso, sem rodeios, no topo, com um sucessor nomeado e um guia de migração linkado. Compare com a alternativa que você vai encontrar o tempo todo no mundo real: pacotes que simplesmente param em silêncio, sem aviso, issues se acumulando, downloads seguindo firmes. Uma nota honesta de abdicação é um presente. Aprenda a recebê-la.

![O tsup sem manutenção ainda registra mais downloads semanais que o sucessor dele tsdown, mostrando a popularidade sobrevivendo ao adeus de um mantenedor.](assets/v03-chart.webp)

Então sistematize isso. Antes de adotar uma dependência, cinco sinais, em ordem de ranking:

1. **Avisos de README.** As palavras do próprio mantenedor superam toda métrica desta lista. Avisos de deprecação, "procurando mantenedores", ponteiros para sucessores. Trinta segundos na página do repo.
2. **Data do último publish, lida contra a cadência natural do projeto.** `npm view <pkg> time.modified` dá a data; o julgamento é contextual. Um utilitário com features completas pode ficar quieto e estar tudo bem; um bundler acompanhando um ecossistema em movimento que fica em silêncio por um ano é outra história. A data é o dado, a cadência é a lente.
3. **Tendência de download versus contagem absoluta.** `curl -s https://api.npmjs.org/downloads/point/last-week/<pkg>` para o instantâneo, o gráfico da página do npm para o formato. Os 5.7 milhões do tsdown como sucessor jovem são um sinal de vitalidade mais forte que o número maior, mais velho e em queda do tsup.
4. **Ponteiros para sucessores.** Quando o README, as issues ou a conversa do ecossistema todos apontam para algum lugar específico, a sucessão já aconteceu socialmente mesmo que os números não tenham alcançado.
5. **Preferências reveladas dos repos em que você confia.** De que dependem as bases de código que você já lê? Quando os repos que você respeita começam a migrar, esse é o ecossistema votando com os lockfiles dele, na frente do gráfico de downloads.

![Cinco sinais ranqueados para julgar uma dependência, das palavras do próprio mantenedor até as preferências reveladas do ecossistema, cada um com o comando de checagem dele.](assets/v04-table.webp)

Mais duas leituras trabalhadas calibram o checklist contra os modos de falha dele, porque um checklist que você só rodou num pacote ensina a lição errada.

Express, o modo de falha da impaciência. Express 4.0.0 publicado em 2014-04-09. Express 5.0.0: 2024-09-10, datas do registro, uma década e cinco meses entre majors. Por um teste de "versão major recente", o Express passou dez anos parecendo morto enquanto movimentava o que hoje são uns 133 milhões de downloads por semana, nove dígitos, com releases de manutenção continuando por todo este verão. Lento não é morto. Um pacote maduro em repouso muitas vezes está só pronto, e o checklist lê sinais do mantenedor contra a cadência precisamente para nunca confundir estabilidade com abandono.

esbuild, o modo de falha da superstição com número de versão. O esbuild está em 0.28.2 no momento em que escrevo, zero-ponto-x depois de anos como uma das ferramentas mais estruturais do mundo JavaScript; é o motor sobre o qual o próprio tsup foi construído, e ele publicou dentro do último mês. Enquanto isso um monte de pacotes ostentando um confiante 2.x ou 3.x não vê um commit há anos. Dígitos de versão são marca. Eles não são sinais vitais, e nada na lista de cinco sinais pede um.

![O Express se arrasta entre majors e mesmo assim prospera, o tsup termina numa nota de despedida apesar do uso enorme, e o esbuild continua saudável sem nunca sair do zero ponto x.](assets/v05-timeline.webp)

O trade-off que completa o quadro, e ele aponta para você agora: publicar é um compromisso vestido de marco. No momento em que a versão 0.1.0 existe no registro, o lockfile de todo consumidor é uma promessa que você está cumprindo, disciplina de semver, changelogs, resposta a segurança, o pacote completo, e um pacote abandonado COM usuários é pior que nenhum pacote, porque nesse ponto você virou o README do tsup, tomara que com a honestidade dele. Então nomeie o escopo honesto de hoje: você publica para aprender a mecânica e para reivindicar uma peça de portfólio de verdade, algo defensável para um pacote 0.x com um consumidor conhecido, você. Assuma o fardo de manutenção deliberadamente só para código que você genuinamente quer que estranhos rodem. O checklist corta dos dois lados; um dia alguém roda ele em você, e a coisa mais gentil que o seu futuro pacote fantasma pode fazer é dizer isso no topo do README dele.

Esta leitura de cinco sinais volta lá no fim do curso, sistematizada num veredito de auditoria escrito no lab de auditoria de dependências, onde as apostas param de ser escolha de ferramenta e passam a ser cadeia de suprimentos.

### A trava do tier: o que o M1 até o M3 pulou, e onde isso mora

O tier de TypeScript acaba nesta lição, então o curso te deve o mapa que ele prometeu em m01-l1: o que a gente deliberadamente não ensinou, e a casa nomeada de cada peça. Isto é território devido, não pedido de desculpas. Os 80% que você já tem são de verdade: uniões e narrowing, fronteiras e zod, disciplina de async, um cron testado, um workspace, um painel, uma URL, e a partir de hoje um pacote publicado. Os 20% nunca estiveram faltando. Eles estavam arquivados:

- **Programação em nível de tipos e autoria de genéricos.** Você consome genéricos com fluência (`z.infer`, `ReturnType`, o kit de m02-l2); você ainda não escreve tipos condicionais e mapeados. O pátio de treino é o type-challenges (a academia pós-M2 de m02-l1) mais o capítulo de Generics do Handbook. Vá quando a assinatura de tipo de uma biblioteca te deixar curioso em vez de cansado.
- **A superfície completa do tsconfig.** Você domina o cânone estrito da tabela de m01-l2; as várias dezenas de flags restantes são consultadas flag a flag, sob demanda, para sempre. Ninguém decora elas. Agora você sabe disso.
- **node:test.** O runner de zero dependências ganhou o sidebar honesto dele em m02-l4; o vitest é a pista deste curso. Se um contexto sem dependências quiser testes, o sidebar é a rampa de entrada.
- **Bun e Deno.** Os dois vivos e entregando (Bun 1.4, Deno 2.9, os dois com releases no fim de agosto de 2026). Este curso roda Node porque o ecossistema Solana pesquisado roda: todo repo estudado declara engines do Node e um pin de packageManager pnpm, nenhum declara Bun ou Deno. Posicionamento, não desdém; revisite o sinal cinco daqui a um ano e veja se os lockfiles se moveram.
- **React além da fatia do consumidor de dados, e tudo que é cliente.** Routing, forms, bibliotecas de estado, UX de carteira, aterrissagem de transação: isso é território do curso de maestria do lado cliente, em produção enquanto eu escrevo, e a força de TS que você já tem é exatamente a fundação sobre a qual esse tipo de trabalho se constrói.
- **jest.** Nomeado, não ensinado: é o irmão mais velho do vitest e você vai encontrar ele nos repositórios da anza. A superfície de API é parecida o bastante para a sua fluência em vitest transferir na maior parte.

É essa a trava inteira. Todo bookmark tem um endereço, todo endereço tem um trigger para quando visitar, e o mapa que você ganhou na abertura do curso acabou de ganhar o primeiro pin de "você está aqui": forte em TS, com as localizações dos 20% decoradas.

![Uma região central assentada de habilidades ensinadas fica cercada por seis territórios deixados como bookmark, cada um rotulado com exatamente para onde ir quando precisar.](assets/v06-diagram.webp)

## Lab: pulse-core, publicado

A metade trabalhada está pronta: o build roda, o mapa de exports carrega tipos ao lado do JavaScript, o tarball está limpo. O que sobra é seu, a partir de um checklist. Você vai precisar de uma conta npm gratuita: cadastre-se em npmjs.com se ainda não tiver, e anote o seu username, porque ele está prestes a virar um namespace.

1. **Reivindique o seu escopo no nome.** Em `packages/pulse-core/package.json`, mude o name para o seu escopo: `"name": "@YOUR_NPM_USERNAME/pulse-core"`. Nomes de registro têm que ser únicos; o seu escopo é o canto do registro onde a unicidade é problema só seu.

2. **Religue os consumidores sem tocar em um import.** A frota e o painel importam de `"pulse-core"`, e o protocolo de workspace do pnpm tem uma forma de alias feita exatamente para este rename. Em `packages/pulse-fleet/package.json` e `packages/pulse-board/package.json`, mude a linha de dependência:

   ```json
   {
     "dependencies": {
       "pulse-core": "workspace:@YOUR_NPM_USERNAME/pulse-core@*"
     }
   }
   ```

   Depois `pnpm install` a partir da raiz. O alias diz: o especificador local `pulse-core` resolve para o pacote do workspace agora chamado `@YOUR_NPM_USERNAME/pulse-core`. Toda linha `import { classifyProbe } from "pulse-core"` na estação inteira continua funcionando, textualmente. Checkpoint: `pnpm -r test` verde a partir da raiz, zero linhas de import mudadas.

3. **Faça o build e o pré-voo.** A partir de `packages/pulse-core`: `pnpm build`, depois `npm pack --dry-run`. A aceitação é o formato limpo da seção de teoria: `dist/index.js`, `dist/index.d.ts`, `package.json`, nada mais. Se aparecer qualquer coisa extra, o campo `files` é a sua allowlist; conserte e rode o dry-run de novo até a listagem ficar chata.

4. **Publique.** Dois comandos, uma flag que importa:

   ```bash
   npm login
   npm publish --access public
   ```

   O `npm login` rebate pelo navegador. A flag `--access public` é a pegadinha da seção de teoria: sem ela, um primeiro publish com escopo falha com um erro sobre planos de pagamento, porque pacotes com escopo vêm privados por padrão e privado é pago. Com ela, o terminal imprime o nome e a versão do seu pacote, e essa é a cerimônia inteira. `@YOUR_NPM_USERNAME/pulse-core@0.1.0` agora existe no registro público. Vá olhar a página dele em npmjs.com; você tem uma página de artefatos entregues agora. Ela vai dizer que nenhum README foi encontrado, com verdade, porque o tarball limpo que você inspecionou no passo 3 não contém nenhum; escrever um é o primeiro polimento pós-entrega que esta lição deixa para você.

5. **Prove como um estranho.** Em algum lugar FORA do workspace, o seu diretório home, qualquer lugar:

   ```bash
   mkdir pulse-scratch && cd pulse-scratch
   npm init -y
   npm pkg set type=module
   npm i @YOUR_NPM_USERNAME/pulse-core
   ```

   Depois `smoke.mjs`:

   ```js
   import { classifyProbe } from "@YOUR_NPM_USERNAME/pulse-core";

   console.log(classifyProbe("ok", 240));
   console.log(classifyProbe("ok", 700));
   ```

   `node smoke.mjs` deve imprimir `up` e depois `degraded`, direto do contrato de m02-l1: abaixo de 400 é up, de 400 a 1000 é degraded. Este é o seu código, instalado da internet pública, rodando o mesmo julgamento com que o seu cron publica. Se você quiser a prova completa, rode `npm i -D typescript` (um `npx tsc` pelado num projeto sem ele resolve o pacote errado do registro, um stub deprecado chamado `tsc`), adicione um `tsconfig.json` com configurações estritas de `"module": "nodenext"` e um arquivo `.ts` importando `ProbeResult`; o `npx tsc --noEmit` passando prova que os tipos viajaram. Este passo é o que pega a falha de tipos invisíveis da seção de teoria, e é por isso que ele é um passo e não uma sugestão.

6. **Diga em voz alta a verdade da fiação.** De volta ao workspace: `pnpm -r test`, ainda verde. Abra o painel, ainda renderizando. E já que o rename acabou de acontecer, termine a fiação de pipeline da seção de teoria: atualize o `pnpm --filter` nos três jobs do `pulse.yml` e no script de build do painel para o novo nome com escopo, dê push, e veja tanto a execução do Actions quanto o deploy do Vercel continuarem verdes a partir dos clones frescos deles. Só aí a afirmação é honesta em todo lugar, não só no laptop onde `dist/` já existe. Nada na estação consome a cópia do npm; o cron e o painel resolvem a cópia do workspace pelo alias, mesmo symlink de ontem. Publicar mudou o ALCANCE do pacote, não a fiação da estação. Agora existem duas cópias da verdade, workspace para você, registro para estranhos, e mantê-las honestas uma com a outra é para isso que servem números de versão, uma disciplina que o tier de Rust vai encontrar de novo pelo lado do cargo.

![O mesmo pacote alcança consumidores do workspace ao vivo por um symlink enquanto estranhos recebem a cópia congelada do registro publicada no npm.](assets/v07-diagram.webp)

## Challenge: três vereditos, sem guarda-corpos

O treino sem guia, e a última rep do tier. Rode o checklist de cinco sinais nestes três pacotes de verdade: `request`, `body-parser` e `zod`. Para cada um, escreva um veredito de um parágrafo, adotar, evitar ou adotar-de-olhos-abertos, citando pelo menos dois sinais concretos que você mesmo checou: um aviso de README ou do registro, uma data de último publish lida contra a cadência, um número de downloads com a data dele, um ponteiro para sucessor, ou a preferência revelada de um repo nomeado. Os comandos já estão nas suas mãos: `npm view <pkg>`, `npm view <pkg> time.modified`, o endpoint de downloads, e a página do repo. Um destes três está com saúde forte, um vem dizendo às pessoas para irem embora há anos enquanto milhões continuam chegando toda semana, e um fica em algum lugar mais interessante; eu não vou te dizer qual é qual, porque ler isso no frio é a habilidade inteira. Aceitação: três parágrafos, toda afirmação checável, e pelo menos um veredito que te surpreendeu o bastante para você conferir duas vezes.

## Checkpoint, e o repasse de linguagem

O que você já consegue fazer, concretamente: levar um pacote de workspace do código-fonte TypeScript até um artefato npm público, com escopo e instalável, com tipos que viajam; ler uma listagem de tarball como um pré-voo e mantê-la limpa com uma allowlist; e ler os sinais vitais de uma dependência em ordem de ranking antes de adotar ela. A recuperação de 30 segundos antes de você fechar a aba: nomeie os dois sinais que superam contagens de download ao julgar a saúde de uma dependência. Diga eles em voz alta. O aviso de README do próprio mantenedor, e a data do último publish lida contra a cadência natural do projeto. Se esses dois vieram na hora, a meta-habilidade está instalada.

Dois pedidos enquanto está fresco. Primeiro, coloque a URL npm do seu pacote ao lado da URL do Vercel da lição passada, bio ou README, onde quer que a primeira tenha ido; o portfólio rende juros compostos. Segundo, anote no seu diário do curso qual passo do lab brigou com você, o rename, a flag de access, a prova de rascunho, porque o tier de Rust publica num registro também e a sua lista de atritos é o checklist que você vai querer aberto quando o crates.io fizer as mesmas perguntas com uma grafia diferente.

O tier de TypeScript está completo: frota tipada, cron testado, workspace, painel, URL, pacote publicado. Toda promessa que o mapa de m01-l1 fez, cumprida e entregue. No próximo módulo o mesmo motor de sonda é reconstruído sob um compilador que se recusa a chutar: ownership, o borrow checker, e o seu primeiro E0382, de propósito, dentro de dez minutos. Vai parecer o code review mais rígido da sua vida, e é o único review que você não pode pular. Traga a tag do pacote publicado e uma casca grossa.
