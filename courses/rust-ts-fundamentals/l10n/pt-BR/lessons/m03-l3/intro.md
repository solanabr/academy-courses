# Vercel: a URL

## Resumo

A lição passada deu um rosto à estação: um painel Vite + React em `packages/pulse-board`, fazendo polling do `status.json` do cron e colorindo linhas com o classificador do pulse-core. Ele roda lindamente, no localhost, o que significa que a audiência total dele é uma pessoa, e essa pessoa já sabe o que os números dizem. Nove lições de trabalho real e cumulativo, e nenhum outro ser humano consegue ver nada disso.

Hoje isso acaba. Este é o SHIP #2 e a primeira URL do curso: você faz o deploy do pulse-board no Vercel a partir do workspace, e dentro do primeiro quarto desta lição existe um endereço de produção que você pode mandar por mensagem para alguém em outro continente cujo telefone vai renderizar os dados da SUA sonda. O resto da lição conquista o entendimento: o que de fato aconteceu quando você digitou uma palavra, o rewrite de um bloco só e a regra de env var que deixam uma SPA honesta em produção, e o contrato do tier gratuito lido dos números publicados dele em vez das vibes de um post de blog.

Faça isso agora mesmo, antes de ler mais um parágrafo:

```bash
npm i -g vercel   # Vercel CLI (59.11.2 as of 2026-09-02; the CLI moves fast, expect a higher digit)
vercel login
```

Escolha o login do GitHub quando o navegador abrir, e garanta que é a sua conta PESSOAL, aquela em que o repo da estação vive. Essa escolha foi feita para você lá no M1, e esta lição é onde você descobre por que ela importava.

O recuo da ajuda, em voz alta: a gente dirige o primeiro deploy junto com cada prompt narrado, os passos de reforço vêm como uma checklist que você executa sozinho, e o treino final te dá dois deploys quebrados e nenhum guarda-corpo.

## A entrega e as letras miúdas

### Entregue primeiro, entenda depois

A partir da RAIZ do workspace, não do diretório do pacote (a CLI prefere a raiz do repo e vai perguntar onde o código mora):

```bash
vercel
```

A CLI te leva por um questionário curto. Set up and deploy: sim. Scope: sua conta pessoal. Link to an existing project: não. Project name: `pulse-board` serve. E então a pergunta que importa, a que pergunta em qual diretório o seu código está: responda `packages/pulse-board`. A CLI detecta o Vite, roda o build remotamente, e devolve uma URL de preview. Cutuque, confirme que o painel renderiza, depois promova:

```bash
vercel --prod
```

Uma URL de produção é impressa. Essa é a entrega inteira.

Duas URLs em dois comandos merecem uma definição, porque a divisão é estrutural para o resto da sua vida de deploy. O `vercel` puro criou um **deploy de preview**: URL única própria, um build completo exatamente do que você mandou, seguro de compartilhar e seguro de jogar fora. O `vercel --prod` criou um **deploy de produção**, aquele para onde o endereço principal do seu projeto aponta. Mesmo pipeline, audiência diferente: previews são onde você olha uma mudança antes de acreditar nela, produção é o endereço que você dá para estranhos. Depois que o repo estiver conectado no lab, essa divisão automatiza: pushes para um branch ganham URLs de preview, merges para a main vão para produção, e você nunca mais vai ficar se perguntando se um revisor está olhando a versão que você acha que ele está.

Agora faça a coisa para a qual esta lição existe: abra essa URL num aparelho que nunca viu o seu código. Um celular em dados móveis serve. Melhor ainda, mande para um amigo e assista linhas de sonda de verdade, latências que o SEU cron mediu, renderizarem num hardware que você nunca tocou. A primeira URL que eu entreguei na vida foi para um amigo que abriu ela num ônibus, e eu atualizava o analytics como se fosse noite de apuração. Nada que eu tenha feito deploy desde então bateu igual. Este marco, a entrega da lição dez, te custou nove lições de TypeScript, uma suíte de testes, uma extração de workspace, e um cron que vem commitando JSON fielmente há semanas. Você mereceu a URL. Tire o minuto.

Beleza. Minuto acabou. O que de fato aconteceu?

![O comando vercel sobe o código-fonte, uma máquina remota faz o build dele, os arquivos emitidos aterrissam num CDN, e uma URL serve eles.](assets/v01-flowchart.webp)

A desmontagem, estágio por estágio. A CLI empacotou o seu código-fonte e mandou para cima. Uma máquina de build do Vercel olhou o repo, reconheceu o Vite pelo que o próprio repo declara (a dependência `vite`, o script de build, o arquivo de config), rodou `vite build`, e pegou o diretório `dist/` emitido. Esses arquivos foram para o CDN do Vercel, replicados para localizações de edge, e a URL que você recebeu é um nome apontando para eles. Repare no que NÃO está nesse pipeline: a pasta dist do seu laptop. O build rodou na máquina deles a partir do seu código-fonte, que é por que um build que falha vai aparecer nos logs DELES, e por que, depois que o repo estiver conectado, um git push pode disparar um deploy enquanto o seu laptop fica fechado.

As pessoas chamam isso de zero-config, e o mecanismo honesto merece ser dito sem o marketing de ninguém junto: o seu repo já declara o framework dele, o comando de build dele e o diretório de saída dele, então a plataforma lê essas declarações e provisiona infraestrutura correspondente. Você configurou bastante. Você só fez isso no `package.json` e no `vite.config.ts`, arquivos que você estava mantendo de qualquer jeito, e a plataforma tratou eles como a config. Sintetize até o fim: o deploy virou um artefato de build. O repo agora é a única fonte de verdade para como a produção é.

Um prompt merece um segundo olhar: a pergunta do diretório. Aquela resposta é a configuração **Root Directory** vestindo um prompt de terminal. O modelo de monorepo do Vercel é um projeto por diretório deployável: o seu repo guarda `pulse-core`, `pulse-fleet` e `pulse-board`, e o projeto que você acabou de criar aponta para exatamente um deles. A configuração é escolhida no import e editável depois em Settings, então Build and Deployment, então Root Directory. Isso foi a contingência em aberto do curso por um tempo (o plano de fallback era extrair o painel para um repo standalone), e a sonda de docs fechou isso: Root Directory é o caminho suportado e documentado, sem cirurgia nenhuma.

![Um projeto Vercel aponta o Root Directory dele para o pacote do painel enquanto os dois pacotes irmãos ficam sem deploy.](assets/v02-diagram.webp)

Tem um bônus enterrado aqui que você já pagou. Quando o repo está conectado ao GitHub, o Vercel pula builds de projetos de monorepo que um commit não afetou, e os requisitos documentados desse skip se leem como uma checklist de m03-l1: uma definição de workspace de verdade com os pacotes declarados, um `name` único por pacote, e dependências entre pacotes declaradas em cada `package.json`. A higiene de manifest que você fez duas lições atrás não foi cerimônia. É a razão de esse import Simplesmente Funcionar e a razão de deploys irmãos futuros não queimarem slots de build em commits que só tocaram na frota.

### Deixando isso honesto em produção

O seu painel está no ar mas ainda não é honesto. Duas lacunas, ambas invisíveis no localhost.

Lacuna um: deep links. Visite a sua URL de produção com qualquer caminho anexado, `your-board.vercel.app/history`, digamos. Você recebe a página 404 do Vercel. O mesmo caminho sob `npm run dev` renderiza o app numa boa. Eu mesmo já entreguei esse 404 exato, mais de uma vez, e a segunda vez foi mais constrangedora porque eu tinha escrito a correção num wiki na primeira. A assimetria é a lição: o servidor de dev silenciosamente reescreve caminhos desconhecidos para `index.html` como um favor, e um host estático de produção não faz favor nenhum. Não existe arquivo chamado `history` dentro de `dist/`, então um servidor de arquivos corretamente diz 404. A sua SPA roteia no cliente, o que significa que o host tem que passar `index.html` para TODO caminho e deixar o React seguir dali.

A correção inteira é um bloco. Crie `packages/pulse-board/vercel.json`:

```json
{
  "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]
}
```

Toda requisição, qualquer que seja o caminho, recebe o app shell; o roteador do cliente (hoje, só o seu painel de uma página; a partir do M8, rotas de verdade) decide o que renderizar. O painel nem tem uma segunda rota ainda e isso já importa: no dia em que ele ganhar uma, e o painel de Solana do M8 acrescentar outra, deep links compartilhados no chat vão ou funcionar ou dar 404 dependendo de se este bloco foi entregue hoje.

![Uma requisição de deep link tem sucesso em dev, falha num host estático puro, e tem sucesso de novo quando o rewrite serve a página index.](assets/v03-comparison.webp)

Lacuna dois: a URL hardcoded. Agora mesmo o painel busca `status.json` de uma URL raw do GitHub colada no código-fonte. Funciona, mas solda o seu deploy a um repo só: qualquer pessoa que dê fork na estação, e você mesmo quando o M8 acrescentar uma segunda fonte de dados, tem que editar código para reapontar. Configuração pertence ao ambiente. É aqui que o Vite tem uma regra que você precisa saber de cor: **só env vars prefixadas com `VITE_` chegam ao código do cliente.** Todo o resto fica do lado do build, invisível para o bundle.

O prefixo não é burocracia, é um termo de consentimento. Tudo em JavaScript de cliente é legível pelo mundo inteiro, para sempre, por qualquer um com uma aba de devtools. Então a exposição é opt-in, e o prefixo feio é você assinando o termo: este valor vai ser público. Leia a regra de trás para frente e ela é a lição de segurança: nada secreto pode NUNCA vestir `VITE_`. Nada de API keys, nada de tokens, nada que você não imprimiria numa camiseta. A varredura de segredos de quatro plataformas em m09-l2 volta a essa regra exata com uma checklist.

Faça a fiação. Primeiro a config tipada, em `packages/pulse-board/src/config.ts`:

```ts
const repo = import.meta.env.VITE_STATION_REPO;

export const statusUrl: string | null = repo
  ? `https://raw.githubusercontent.com/${repo}/main/status.json`
  : null;
```

E ensine o TypeScript sobre a variável em `packages/pulse-board/src/vite-env.d.ts`. Templates mais antigos do create-vite faziam o scaffold desse arquivo; o template react-ts 9.x atual não faz, então crie ele você mesmo, e o nome ainda importa, porque `vite-env.d.ts` é a casa convencional que a documentação do Vite e todo colega de time vão procurar:

```ts
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_STATION_REPO: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
```

O `string | undefined` é honestidade deliberada: a variável pode não estar definida, e o tipo força todo consumidor a dizer o que acontece então. No painel, um statusUrl `null` deveria renderizar um estado visível de erro de configuração, não uma página em branco. Falhe em alto e bom som. Um painel que não renderiza nada e não diz nada é o pior dos dois mundos.

Uma nota de mecanismo que vai te salvar uma noite confusa: o Vite faz inline desses valores em tempo de BUILD. O bundler literalmente substitui a string `import.meta.env.VITE_STATION_REPO` pelo valor durante o `vite build`. Um site estático não tem ambiente de execução para ler, então mudar a variável no painel não faz nada com os arquivos em produção até o próximo build cravar o novo valor. Defina uma var, faça o deploy de novo, sempre nessa ordem.

![Uma variável de ambiente passa por uma trava de prefixo em tempo de build e ou entra via inline no bundle público ou fica do lado do build.](assets/v04-diagram.webp)

Defina o valor onde a máquina de build consegue ver:

```bash
vercel env add VITE_STATION_REPO
# paste your owner/repo when prompted, e.g. yourname/pulse-station
# when it asks which environments, select all three: Production, Preview, Development
vercel env pull packages/pulse-board/.env.local
```

Ambientes importam aqui, e eles mapeiam na divisão de deploy que você acabou de aprender: o Vercel escopa toda variável para Production, Preview, Development, ou qualquer combinação, que é por que o prompt pergunta. Selecione os três para esta, porque o painel deveria renderizar os mesmos dados em todo lugar. O `pull` então sincroniza os valores de Development num `.env.local` gitignorado, para que o dev local leia a mesma configuração que a produção crava no build. Olhe o caminho de destino do pull: a CLI roda a partir da raiz do workspace como todo comando vercel aqui, mas o Vite só lê arquivos de env a partir da raiz própria do projeto Vite, então o arquivo tem que aterrissar dentro de `packages/pulse-board/`. Puxe ele para a raiz do repo em vez disso e o dev local nunca vê a variável, renderiza o seu estado de erro de configuração, e te manda caçar uma var "faltando" que está sentada um diretório alto demais. Uma variável, uma fonte de verdade, três ambientes. Se você tivesse escopado só para Production, os previews construiriam com a variável ausente e renderizariam o seu estado de erro, o que é uma configuração legítima para valores que diferem por ambiente, e uma surpresa confusa para os que não deveriam. (Regra da casa da própria página de limites da plataforma, lida em 2026-09-01: todas as suas env vars juntas têm um teto de 64 KB. Você nunca vai bater nele com um slug de repo; um time enfiando blobs de JSON em vars vai, e agora você sabe que o teto existe.)

### O contrato Hobby, das próprias páginas dele

Antes de confiar a sua estação a um tier gratuito, leia as páginas do fornecedor, datadas, e o resumo de ninguém. Esse hábito tem um motivo recente colado nele: em 2026-09-01 este curso sondou a documentação Pages da própria Cloudflare e achou ela te dizendo para começar projetos novos com Workers em vez disso. Um fornecedor aposentando um produto às claras, na própria documentação dele, enquanto tutoriais de três anos de idade continuam recomendando ele. A documentação se mexe; posts de blog fossilizam. Então aqui está o tier Hobby do Vercel a partir das páginas de preços e limites do vercel.com como lidas em 2026-09-01, números, não adjetivos.

Incluído por mês: 100 GB de Fast Data Transfer, 1M Edge Requests, 1M Function Invocations. Tetos operacionais: 100 deploys por dia, 100 builds por hora, 200 projetos, um build concorrente, um teto de build de 45 minutos. Essas duas listas são tipos diferentes de números. A primeira é consumo que você gasta sendo popular; a segunda é vazão que você gasta iterando. A sua estação não força nenhuma das duas: um arquivo JSON pequeno mais um bundle modesto contra 100 GB é uma margem enorme, e você precisaria entregar código mais rápido que um deploy a cada 15 minutos o dia inteiro para sentir o teto de deploys.

![Blocos de estatística listam as cotas mensais e os tetos operacionais do tier Hobby com os valores documentados e a data da fonte deles.](assets/v05-chart.webp)

Agora a parte que torna o Hobby genuinamente ensinável: o modelo de excedente. O Hobby não tem ciclo de cobrança. Não há nada contra o que medir, então não há fatura, nunca. Estoure um limite e o comportamento documentado é que a feature PAUSA até a janela de 30 dias passar, na maioria dos casos (a única exceção documentada que a passada de pesquisa achou: Web Analytics volta depois de 7 dias). Depois o serviço volta. Sente com o que isso significa: neste tier, o pior caso da sua estação viralizar é downtime. Nunca dívida. Todo outro modelo de preço que você vai encontrar na sua carreira deveria ser medido contra essa frase.

![Estourar um limite do Hobby pausa a feature até uma janela de trinta dias passar, enquanto nenhum ramo de cobrança existe.](assets/v06-flowchart.webp)

Leia o modelo contra o tráfego real da sua estação e a margem deixa de ser abstrata. O payload do painel é um JSON de status medido em kilobytes e um bundle compilado que é entregue uma vez por visitante e depois fica parado no cache dele. Para gastar 100 GB você precisaria de tráfego na casa dos milhões de carregamentos de página, e se a estação de alguma forma achar essa audiência, o que acontece é uma pausa e um problema muito bom, não uma fatura surpresa. Conhecer o modo de falha ANTES de ele disparar é o hábito operacional que este curso não para de treinar, e aqui o modo de falha é documentado, delimitado e sobrevivível por design.

A generosidade tem bordas, e elas são design, não letras miúdas para se ressentir. Três restrições documentadas se compõem aqui. O Hobby é para uso pessoal, não comercial, pela política de uso justo. O Hobby é SOLO: a tabela de comparação de planos mostra um traço para recursos de colaboração em equipe, então não tem como convidar um colaborador para dentro do projeto. E times Hobby não podem conectar repositórios pertencentes a organizações do Git. Conta pessoal, repo pessoal, um humano, nada à venda. Agora olhe de volta para o M1, quando o curso insistiu que a estação vivesse pública na sua conta PESSOAL do GitHub. Aquilo foi esta lição alcançando para trás. O formato inteiro da estação foi projetado para que a entrega da lição dez precise de zero gambiarra, que é o que um caminho principal honesto com o tier gratuito de fato custa: decisões tomadas meses antes.

E a pergunta do cartão de crédito, que merece ser respondida do jeito que este curso responde tudo. Você vai ler "não é preciso cartão de crédito" sobre o Vercel Hobby pela internet inteira. O Vercel nunca escreve essa frase. O que a documentação mostra: o plano Hobby não tem ciclo de cobrança, e o único lugar em que detalhes de cartão aparecem na documentação do plano é o passo cinco do fluxo de upgrade para Pro. Vários textos de terceiros de 2026 dizem que nenhum cartão é pedido no cadastro. Mas até 2026-09-02 este curso não verificou isso com um cadastro novo, então a gente formula exatamente até onde a evidência alcança: gratuito, sem ciclo de cobrança, o uso pausa em vez de cobrar. Quando você não consegue dar fonte para uma frase, não diga a frase. Se você afirma "sem cartão de crédito" para um amigo, você está citando blogueiros, não o fornecedor, e saber a diferença é uma habilidade profissional que esta lição está deliberadamente modelando.

O trade-off honesto, nas duas direções. Zero-config é um empréstimo, não um presente: o Vercel inferiu o seu build porque o seu repo bate com um padrão que ele conhece, e no dia em que você sair do padrão, um layout de monorepo esquisito, uma etapa de build exótica, a mágica vira configuração que você agora tem que aprender de qualquer jeito, com uma camada de inferência sentada entre você e a mensagem de erro. E as bordas do Hobby desqualificam casos de verdade por design: o app de produção de uma startup é comercial, colaborativo e provavelmente pertencente a uma org, o que é zero de três. O caminho de upgrade honesto existe (Pro, $20 por assento por mês). E o alternativo honesto também: `dist/` é só arquivos, e qualquer host estático da Terra consegue servir arquivos. O que você reconstruiria em outro lugar não é a hospedagem, é o loop, push para deploy, mais o CDN e o encanamento de ambiente. Esse loop é o verdadeiro divisor de águas, e vale saber que é DISSO que você sentiria falta, não da marca.

![Os arquivos estáticos são portáveis para qualquer host, enquanto o loop de push-para-deploy com encanamento de ambiente é a parte que uma plataforma de fato fornece, com a fronteira de Hobby para Pro anotada abaixo.](assets/v07-comparison.webp)

### O que a plataforma não fez

Mais um pedaço de honestidade, porque a plataforma que você acabou de usar é famosa por features que você não tocou. Nenhum servidor rodou hoje à noite. O seu deploy é arquivos estáticos num CDN, ponto final. A camada de compute do Vercel é real e grande: desde 2025-04-23, o Fluid compute é o default para projetos novos, significando funções serverless que se comportam como servidores que você não gerencia, concorrência dentro das instâncias, cobrança por CPU ativa, através de runtimes Node.js, Python, Edge, Bun e Rust (a parte de concorrência otimizada dentro da função é só Node.js e Python, uma distinção que vale manter clara quando alguém te vender isso com hype). O painel não precisa de nada disso. Um painel que lê um arquivo JSON público é o caso de site estático na forma mais pura, e saber precisamente qual camada de uma plataforma você NÃO está usando é o que separa posicionamento de cargo cult. A estação VAI ganhar uma API, e quando ganhar, no M7, ela vai para um edge completamente diferente, e você vai fazer aquela plataforma ler as próprias páginas de preço dela também.

**Vá mais fundo (os 20%).** esta lição ensinou o caminho de deploy que o nosso artefato exercita, e parou. Runtimes de Functions, profundidade de Fluid compute, e o mundo Next.js-no-Vercel são material de verdade que este curso deliberadamente sinaliza em vez de ensinar. A entrada canônica é a própria trilha de introdução do Vercel, que roda CLI-primeiro exatamente como esta lição rodou: [Getting started with Vercel](https://vercel.com/docs/getting-started-with-vercel) (URL sondada em 2026-09-01; a própria página foi atualizada pela última vez em 2026-08-11). Leia depois do lab se a plataforma te interessa; nada abaixo depende disso.

## Lab: reforce a entrega

Você fez o deploy com rodinhas. Agora a checklist, e é uma checklist, não um walkthrough: cada passo nomeia o objetivo e a prova, e você fornece as teclas.

1. **Prove o 404 primeiro.** Abra `https://<your-board>.vercel.app/history` (ou qualquer caminho inventado) e confirme que a página 404 aparece. Nunca conserte um bug que você não viu falhar; você quer a foto do antes para o depois do passo 2 fazer sentido.

2. **Entregue o rewrite.** Acrescente o `vercel.json` de um bloco só da seção de teoria em `packages/pulse-board/`, faça commit dele, e rode `vercel --prod` a partir da raiz do workspace. Aceitação: o mesmo caminho de deep link agora renderiza o painel, e qualquer outro caminho que você inventar também.

3. **Tire o hardcode da fonte de dados.** Coloque no lugar o `config.ts` e a extensão `vite-env.d.ts`, substitua a URL raw colada no fetch por `statusUrl`, e faça o caso null renderizar uma mensagem visível de erro de configuração em vez de um painel em branco. Aceitação: `pnpm run build` dentro de `packages/pulse-board` sai limpo, e rodar o dev SEM a variável definida mostra o seu estado de erro, não uma página branca.

4. **Defina a variável nos dois lugares.**

   ```bash
   vercel env add VITE_STATION_REPO
   vercel env pull packages/pulse-board/.env.local
   ```

   O primeiro pede um valor (o seu slug `owner/repo`) e quais ambientes; selecione os três, exatamente como a seção de teoria argumentou. Se você tivesse escopado só para Production, o `pull`, que sincroniza os valores de Development, te daria um `.env.local` vazio e uma sessão de dev confusa. O segundo escreve `.env.local` dentro do pacote do painel, onde o Vite de fato lê arquivos de env, para que o dev local concorde com a produção. Confirme que `.env.local` está gitignorado (está, se a sua higiene do M1 segurou; cheque mesmo assim). Faça o deploy de novo com `vercel --prod`. Aceitação: a produção renderiza linhas ao vivo de novo, e o view-source no bundle em produção encontra o slug do seu repo cravado no JavaScript, que é o inlining em tempo de build tornado visível, e uma prévia de por que segredos nunca vestem o prefixo.

5. **Feche o loop pelo GitHub.** No painel do Vercel, conecte o projeto ao repo da sua estação nas configurações de Git do projeto. Depois faça push de uma mudança trivial (mexa num título, conserte um typo) e veja o painel fazer o build e o deploy dela com o envolvimento do seu laptop terminando no `git push`. Aceitação: um novo deploy de produção aparece sem que você tenha rodado `vercel` para ele. Desse commit em diante, o repo É o botão de deploy.

6. **O checkpoint social.** Mande a URL para um humano que nunca viu o seu código, numa rede diferente da sua, e consiga a confirmação de que linhas de sonda ao vivo renderizaram. O celular de um estranho é o único teste de integração honesto que uma URL tem.

## Challenge: quebre duas vezes, leia onde sangra

Sem guia, e vale fazer devagar. Falhas de produção vêm em classes, e a primeira habilidade do operador é saber ONDE cada classe aparece antes de acontecer às 2 da manhã.

Quebra um: remova a env var (`vercel env rm VITE_STATION_REPO production`), faça o deploy de novo, e abra a URL. Ache a falha. Quebra dois: introduza uma mudança que quebra o build (apague o equivalente a um ponto e vírgula de corretude de tipos em algum lugar, um import ruim funciona bem) e faça push dela. Ache essa falha também.

Para cada quebra, escreva UMA frase dizendo onde a falha apareceu (os logs de build? a página em produção, em tempo de execução?) e que classe de falha isso faz dela. Depois repare as duas: restaure a variável, reverta o commit, confirme o verde.

![Um erro de build para nos logs enquanto o deploy antigo continua servindo, mas um valor de configuração faltando é entregue e falha na frente dos usuários.](assets/v08-comparison.webp)

Aceitação para a lição inteira: URL de produção no ar e renderizando dados de verdade num dispositivo alheio, um caminho de deep link carrega direto, e as suas duas frases classificam as falhas corretamente, tempo de build versus tempo de execução. Repare em qual falha foi mais segura: o build quebrado nunca tocou nos seus usuários, porque o deploy anterior continuou servindo. A variável faltando foi entregue. Erros de configuração são a classe mais sorrateira, que é exatamente por que o passo 3 deixou eles em alto e bom som.

## Checkpoint, e a vez do motor

O que você já consegue fazer, concretamente: fazer deploy de um pacote de dentro de um monorepo com o Root Directory fazendo a mira; deixar uma SPA honesta em produção com um rewrite e uma env var conscientemente pública; ler o contrato de um tier gratuito a partir dos números publicados dele e dizer o que cabe nele (uma estação pessoal, solo, não comercial: perfeitamente) e o que não cabe (qualquer coisa com clientes ou colegas de time); e classificar uma falha de produção por onde ela apareceu. A recuperação de 30 segundos antes de você fechar a aba: no dia em que a sua estação estourar 100 GB de transferência, o que acontece? Diga em voz alta. A feature pausa até a janela de 30 dias passar, nenhuma conta chega, porque não existe ciclo de cobrança para uma chegar em cima.

Dois pedidos enquanto está fresco. Primeiro, anote ONDE o lab brigou com você (as respostas dos prompts? a ida e volta da env var? a conexão com o Git?) nas suas notas do curso; o M7 repete essa dança inteira numa plataforma diferente e a sua lista de atrito vira a sua checklist. Segundo, ponha a URL em algum lugar onde você vai ver: bio, README, onde for. Artefatos públicos acumulam de um jeito diferente dos locais, e agora você tem um.

A estação tem um rosto público. E o pulse-core, o motor por trás de tudo naquela página, ainda está preso no seu workspace onde só os seus próprios pacotes conseguem importar ele. A próxima lição é a volta olímpica do módulo: fazer build dele com tsdown, publicar ele no npm como um pacote de verdade que outros humanos podem instalar, e aprender a ler os sinais de alerta de uma dependência moribunda antes de você adotar alguma. A volta olímpica tem um registro no fim.
