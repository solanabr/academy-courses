# O mapa: o que ensinamos, o que deixamos como bookmark, e por que as duas linguagens

Lição um. Nada foi construído ainda. Você chega sabendo programar em alguma linguagem, novo no Rust, novo no TypeScript, novo em web3, e eu não vou abrir com uma definição. Eu vou fazer você medir a internet com o navegador que você já tem aberto.

Abra uma aba nova, vá em https://rust-lang.org (o endereço apex, sem www), aperte F12 (ou clique com o botão direito, Inspecionar), clique na aba Console e cole isto:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

Aperte Enter. Dentro de um segundo ou coisa assim você recebe uma linha tipo `rust-lang.org: 238.5 ms (status 200)`. Foi isso que a minha imprimiu enquanto eu escrevia isto, de uma conexão residencial no meio do dia. (Duas coisas nesse snippet são deliberadas. `location.origin` é o endereço da aba em que você está parado, então você está sondando o site em que você está em vez de uma URL que eu cravei no código, e uma página sempre tem permissão de ler respostas da própria origem; ler uma resposta de uma origem *diferente* só funciona quando aquele servidor opta por permitir com um header CORS, uma regra de segurança do navegador que você contorna por completo ao perguntar para a aba sobre ela mesma. E a instrução de não usar www importa: a grafia com www deste site faz um redirect 301 para o apex sem avisar, o `fetch` segue redirects em silêncio, e uma sonda que atravessa origens no meio do voo pode ser bloqueada por essa mesma regra mesmo quando o site está perfeitamente saudável.) A sua vai ser diferente, porque é uma medição real de um servidor real pela sua rede real. Sem instalação, sem conta, sem framework. Você acabou de sondar infraestrutura ao vivo e ler a latência dela, e esse único reflexo, apontar uma sonda para algo real e ler o número, é o curso inteiro em miniatura.

Agora a segunda demo, porque este curso tem duas linguagens e cada uma ganha um argumento de abertura. Vá em https://www.typescriptlang.org/play/ (a barra no final importa, essa é a URL final), limpe o editor e digite estas três linhas:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeout;
console.log(`waiting ${wait} ms`);
```

Olhe a linha dois. Antes de você rodar qualquer coisa, um rabisco vermelho aparece embaixo de `timeout`, e passar o mouse em cima mostra:

```
Property 'timeout' does not exist on type '{ url: string; timeoutMs: number; }'.
Did you mean 'timeoutMs'?
```

JavaScript puro rodaria isso feliz da vida e imprimiria `waiting undefined ms`. Ele mentiria com educação, e você descobriria em produção, às 3 da manhã, quando o timeout que você achou que tinha configurado nunca disparou. O compilador pegou o bug enquanto você ainda estava digitando, e ele até chutou a correção. Esse rabisco vermelho é a tese da metade TypeScript inteira deste curso: a máquina consegue provar coisas sobre seu código antes de o código existir em qualquer lugar que não seja seu editor.

Duas sondas, dois minutos, zero instalações. Tudo o que vem depois é um mapa.

## Resumo

Esta lição te entrega três coisas e pede uma decisão honesta. Primeiro, o mapa: uma tabela completa do que este curso ensina versus o que ele deixa como bookmark para recursos canônicos gratuitos, e por que essa divisão é deliberada e não preguiça. Segundo, a evidência para ensinar duas linguagens de uma vez, com contagens de bytes reais de um repo Solana emblemático e uma razão de site de vagas lida do jeito cuidadoso. Terceiro, a promessa: o artefato que você vai construir ao longo de dez módulos, uma estação pessoal de uptime e latência chamada Pulse Station, desenhada como o diagrama exato que você vai redesenhar de memória no módulo final. A decisão honesta é a checagem de pré-requisito no fim da seção de teoria: um recurso gratuito específico, uma pergunta específica, e permissão para sair e voltar.

Uma coisa dita em voz alta antes de começarmos, porque este curso diz as regras dele em voz alta. Agora, na lição um, tudo está totalmente trabalhado: todo comando mostrado, todo snippet completo, você roda em vez de derivar. Essa ajuda recua num cronograma. Uns módulos adiante você vai receber interfaces e constraints em vez de arquivos prontos, e no capstone você vai receber uma spec e silêncio. O recuo é o currículo.

Sem Node, sem compiladores, sem contas hoje. A toolchain chega na próxima lição. Hoje é orientação, e orientação bem feita vale mais que qualquer instalação isolada.

## O mapa é o produto

Aqui está o problema que esta lição existe para resolver. Você já consegue fazer um programa funcionar em alguma linguagem. Entre você e entregar software de verdade numa stack de blockchain, a internet oferece mais ou menos mil horas de material: um livro de Rust de 20 capítulos, um handbook de TypeScript, quatro sites de documentação de plataforma, e uma blockchain que muda sob os seus pés. Ninguém termina essa pilha. Quem entrega nunca terminou. Uma pilha de tutoriais não é um currículo; é um backlog, e backlogs não ensinam. O que quem entrega de fato fez foi aprender 80 por cento específicos e deixar o resto como bookmark, e a maioria montou esse mapa na base da tentativa, do erro e de anos.

Este curso te entrega o mapa de cara e depois caminha nele com você. O acordo, nos termos do próprio dono: já existem ótimos recursos para aprender Rust e TypeScript, então a gente linka eles em vez de reensinar. O que este curso acrescenta é a parte que esses recursos não carregam: como você usa cada padrão no mundo real, e por que você precisa de cada conceito, por causa do que quebra sem ele. Seleção em vez de cobertura. Chamamos a fronteira entre os dois de costura 80/20: o lado ensinado cobre os padrões que carregam o ciclo de vida diário do dev, escrever, testar, empacotar, fazer deploy, operar; o lado dos bookmarks é o material canônico e profundo, linkado em nível de capítulo no momento exato em que você pode querer. Toda lição de linguagem daqui em diante traz uma caixa fixa de aprofundamento, que a gente chama de caixa dos 20%, com o bookmark daquela lição.

![Duas faixas mostram padrões ensinados do mundo real ao lado de recursos canônicos de bookmark, unidas por links em nível de capítulo colocados exatamente onde uma lição precisa deles.](assets/v01-diagram.webp)

O custo desse acordo é real, e você deveria ouvir isso agora em vez de descobrir no meio de um erro. Você VAI encontrar sintaxe de Rust e de TypeScript que este curso nunca ensinou, às vezes dentro de uma mensagem do compilador, três módulos adiante, quando uma anotação de lifetime (tempo de vida) aparece num erro de código que você não escreveu. O mapa é a mitigação, não uma isenção mágica. O acordo só funciona se você realmente abrir o capítulo do bookmark quando uma lição o sinaliza. Ler The Rust Book inteiro de capa a capa antes de começar é a cilada de mil horas; recusar ler o único capítulo para o qual o mapa aponta, no momento em que ele aponta, é a falha oposta e igualmente caro.

### A tabela completa, impressa como conteúdo

Esta tabela não é um apêndice. É o gêmeo franco da ementa, e o módulo final vai te trazer de volta para ela para perguntar quais bookmarks ficaram urgentes. Toda URL abaixo foi verificada ao vivo em 2026-09-02, e esta tabela específica é reverificada antes de o curso entregar atualizações.

| Conceito | Ensinado aqui | Bookmark para |
|---|---|---|
| Fundamentos de JS | Nunca. Este é o pré-requisito | MDN Learn Core Scripting, gratuito, 34 lições: https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting |
| Config estrita de TypeScript | M1: as ~5 flags que o nosso próprio código dispara | intro do TS Handbook: https://www.typescriptlang.org/docs/handbook/intro.html |
| Uniões discriminadas, narrowing, exaustividade | M2 | capítulo de narrowing do Handbook; o repo type-challenges como o treino depois |
| Generics que você realmente usa | M2, na hora exata, motivado pelo zod | capítulo de generics do Handbook |
| Validação nas fronteiras | M2, zod | tutorial gratuito de Zod do Total TypeScript: https://www.totaltypescript.com/tutorials |
| Async de verdade: limites, backoff, cancelamento | M2 | unidade de async do MDN |
| Prática de testes | M2 (vitest), M4 (cargo test) | guia do vitest; docs do node:test |
| Saber empacotar e publicar | M3 | trilha de aprendizado do Node: https://nodejs.org/learn |
| React, nível consumidor | M3 | quick start do react.dev; a profundidade pertence a um curso irmão, nomeado abaixo |
| Ownership e borrowing (posse e empréstimo), o porquê e o uso diário | M4 | O fork interativo da Brown do Rust Book, com quizzes e visualizações de ownership: https://rust-book.cs.brown.edu/ |
| Erros como valores | M4 | The Rust Book, capítulo de tratamento de erros: https://doc.rust-lang.org/book/ |
| Enums, structs, traits na prática | M4 | capítulos de enums e traits do Rust Book |
| serde, domínio do cargo, clap, uma CLI de verdade | M5 | serde.rs; o Cargo book; Rustlings depois de cada módulo de Rust: https://rustlings.rust-lang.org/ |
| tokio, na profundidade de saber-quando-precisa | M6 | capítulo de async do Rust Book (o Book cobre async nativamente agora) |
| Amplitude de sintaxe de Rust | Nunca reensinado | Rust by Example: https://doc.rust-lang.org/rust-by-example/ ; Comprehensive Rust, o curso de onboarding do próprio time de Android do Google: https://google.github.io/comprehensive-rust/ ; o speed-run de sintaxe de meia hora no fasterthanli.me |
| Lifetimes além da leitura, unsafe, autoria de macros | Nunca, sinalizado | The Rustonomicon, que abre dizendo para você não ler ainda: https://doc.rust-lang.org/nomicon/ |
| Contêineres | M6 | Docker Get Started: https://docs.docker.com/get-started/ |
| Fazer deploy do app TS | M3 | getting started do Vercel: https://vercel.com/docs/getting-started-with-vercel |
| A edge, as duas linguagens | M7 | get started do Cloudflare Workers: https://developers.cloudflare.com/workers/get-started/guide/ |
| Profundidade de Solana, Anchor, UX de carteira | Só o modelo mínimo de cliente, M8 | Cursos irmãos deste catálogo, nomeados na prosa abaixo |

Olhe as linhas de nunca. Um curso que consegue dizer "nunca" em voz alta, e te dizer exatamente onde aquele material mora em vez disso, está te fazendo uma promessa: nada no lado ensinado é enchimento, e nada no lado dos bookmarks é secretamente obrigatório para passar num lab. Todo lab deste curso roda com material ensinado mais, no máximo, um capítulo sinalizado. Esse é o aha que vale parar um segundo para sentir: um curso pode ser honesto sobre o que ele não ensina. A lista de bookmarks É o produto.

![Um diagrama de fronteira mostra que os labs podem exigir apenas material ensinado mais um capítulo sinalizado, enquanto o resto do mapa de bookmarks fica fora.](assets/v02-diagram.webp)

### Por que as duas linguagens, com comprovantes

Pergunta justa: por que não só Rust, já que a blockchain roda Rust? Ou só TypeScript, já que essa é a rampa de entrada mais curta? Porque um produto Solana de verdade não é escrito em uma linguagem. É escrito em duas.

O melhor número que a pesquisa encontrou: drift-labs/protocol-v2, um repo emblemático de produção em Solana, se divide quase exatamente ao meio por bytes. TypeScript 5,746,893 bytes, Rust 5,533,370 bytes. Isso não é uma migração pega no meio do voo. O Rust é o programa on-chain; o TypeScript é o SDK dele, os clientes dele, os testes dele, a superfície inteira que usuários e integradores de fato tocam. As duas metades são estruturais. Um produto, as duas linguagens, quase cinquenta-cinquenta.

![Uma única barra para um repositório Solana de produção se divide quase igualmente entre TypeScript e Rust por bytes.](assets/v03-chart.webp)

O mercado de trabalho conta a mesma história de outro ângulo, se você ler com cuidado. No web3.career em 2026-09-01, uma busca por Rust mais Solana devolveu 950 vagas e uma busca por TypeScript mais Solana devolveu 505. Não leia isso como contagens absolutas; sites de vagas fazem multi-tag de forma agressiva e o mesmo anúncio aparece sob várias buscas. Leia a razão de mais ou menos 2:1 a favor de Rust pelo que ela diz sem rodeios: o trabalho central da blockchain é Rust, e é ali que o volume de anúncios, e os degraus mais altos da escada, se concentram. Os 505 de TypeScript não são uma oportunidade menor, e sim uma rampa de entrada mais curta: como a divisão de bytes acabou de mostrar, cada um daqueles times de Rust também entrega uma superfície TypeScript, então TS é a metade em que você pode ser contratado mais cedo enquanto a metade de Rust rende juros compostos. Um dev segurando as duas pontas dessa escada é a forma que os times realmente precisam. Este curso existe porque essa forma não tem um curso dedicado. O que me leva aos vizinhos.

### Os vizinhos, nomeados sem enfeite

Eu quero nomear os outros cursos deste espaço, porque vários deles são bons, e porque três deles estão literalmente dentro dos nossos 20 por cento de bookmark. Isso não é uma coisa normal de um curso fazer, e o fato de parecer incomum vale ser notado.

A Cyfrin entrega um curso gratuito de Solana no Updraft, anunciado pelo Codex da Colosseum em 2026-01-16, e é trabalho sério: todo programa construído duas vezes, uma em Anchor e uma em Rust nativo. O pré-requisito dele é o próprio curso gratuito Rust Programming Basics da Cyfrin, para o qual os nossos módulos de Rust vão apontar de bom grado. A School of Solana da Ackee também é gratuita, uma turma de nove semanas, com entrada por inscrição, e a frase de público dela é quase palavra por palavra a nossa. Nenhum dos dois ensina TypeScript como trilha de primeira classe, nenhum dos dois toca em Docker, Vercel ou Cloudflare, e os dois vão mais fundo on-chain do que nós. Mapas diferentes, desenhados com honestidade.

O mercado também cobra dinheiro de verdade pelo material que a gente linka de graça. A RareSkills precifica o bootcamp de Rust dela em $900 por 3 semanas, o bootcamp de ZK em $2,600 por 14 semanas, e o material de Circom em até $5,500. Eu não estou zoando esses preços; turmas e code review valem pagar. Eu estou te calibrando: o conhecimento bruto é gratuito e linkado na tabela acima, e o que você escolhe pagar, aqui ou em qualquer lugar, deveria ser seleção, sequenciamento e feedback, nunca acesso.

![Uma coluna paga de bootcamps com preços de novecentos a vários milhares de dólares fica ao lado de uma coluna gratuita de recursos canônicos que este curso linka.](assets/v04-comparison.webp)

Um recurso pago merece uma divulgação, uma vez, porque ele mira exatamente o público deste curso. Em 2023-04-25, o ThePrimeagen entregou "Rust for TypeScript Developers" no que hoje é o Master.dev, 5 horas 19 minutos, pago com preview gratuito. O mercado nomeou o nosso público três anos antes deste curso. Se você terminar aqui e quiser uma segunda voz na metade de Rust, essa é a opção paga nomeada, e é a única que este curso vai nomear algum dia.

### Pulse Station: a coisa que você vai construir de verdade

Mapas não motivam ninguém. Artefatos motivam. Então aqui está a promessa, concreta o suficiente para você me cobrar.

Ao longo de dez módulos você constrói a Pulse Station, uma estação pessoal de uptime e latência. Ela começa vergonhosamente pequena: na próxima lição, uma CLI em TypeScript que sonda uma URL e imprime a latência, que você vai notar ser exatamente o one-liner que você rodou no console hoje, promovido a programa de verdade. Depois ela cresce. A sonda passa para um agendamento no GitHub Actions, numa máquina que não é a sua, e commita as medições dela num arquivo. Os resultados ganham tipos, depois validação, depois concorrência disciplinada, depois testes que travam o agendamento. Um painel no Vercel dá a ela uma cara que qualquer estranho pode abrir. O motor é reescrito em Rust, lado a lado com o TypeScript que ele espelha, ganha uma CLI de verdade, depois um poller de longa duração num contêiner Docker publicado no GHCR. As sondas vão para a edge nos Cloudflare Workers, nas duas linguagens, uma delas compilada para WebAssembly. E então a estação aponta para o alvo mais interessante disponível: a própria Solana, leituras ao vivo e uma transação real, uma blockchain cujo próprio batimento é uma história de latência. Toda sonda, do one-liner de console de hoje até o capstone, acerta um alvo real. Não existem feeds sintéticos em nenhum lugar deste curso. E quando os diagramas abaixo dizem "Solana RPC" e "devnet transaction", leia eles por ora de forma aproximada como o endpoint público de consulta da blockchain e a rede gratuita de prática dela; o módulo 8 define os dois direito antes de você tocar em qualquer um.

![Uma linha do tempo de dez paradas faz uma sonda de console crescer até uma frota tipada, um painel, um motor em Rust, um contêiner, edge workers e uma estação que observa a Solana.](assets/v05-timeline.webp)

A estação terminada tem uma forma, e a forma importa o suficiente para o capstone te pedir para redesenhá-la de memória. Este é esse desenho. Estude agora, de leve; você vai encontrar ele de novo no módulo dez.

![Um sistema de hub e raios se centra num repositório público alimentado por sondas agendadas e lido por um painel, edge workers e um poller em contêiner observando alvos reais.](assets/v06-diagram.webp)

Note a palavra public no hub, porque ela é um requisito, não um default que eu esqueci de mudar. O seu repo de estação vai ser público, e três caminhos críticos do curso dependem disso: o GitHub Actions é gratuito e sem medição para repositórios públicos em runners padrão, que é toda a matemática de CI gratuito do agendamento em que a sua sonda vive; o painel busca o `status.json` direto da URL raw pública do repo; e o lab de deploy no Vercel conecta um repo público pessoal no plano gratuito. Um repo privado quebra os três de forma silenciosa, com semanas de diferença, de formas confusas. O custo é igualmente real e é dito em voz alta: o código da sua estação e o histórico completo de status dela são públicos. Para um artefato de portfólio, e este é um, esse custo é uma feature. Empregadores podem ver a sua estação rodando.

### A caixa da honestidade, e as portas que este curso abre

Primeiro, a checagem de pré-requisito, e eu vou citar ela exatamente como o curso a enuncia em todo lugar: abra o MDN Learn Core Scripting, incluindo a unidade de async dele. Se você consegue acompanhar com tranquilidade, você está pronto. Se não, comece por ali. É gratuito, e este curso vai continuar aqui.

Este curso não é para quem é iniciante absoluto em programação; a capa diz isso e eu estou repetindo. A checagem inclui a unidade de async de propósito. O módulo dois constrói disciplina de concorrência direto sobre fluência em promises, com só uma recapitulação de cinco minutos do modelo de promises, e uma recapitulação não é um primeiro curso de async. Pular a checagem não te deixa mais rápido. Realoca o atraso para o módulo dois e deixa ele mais caro.

![Um fluxo de decisão curto encaminha leitores tranquilos para dentro do curso e leitores honestamente ainda-não para um desvio gratuito no MDN antes de voltar.](assets/v07-flowchart.webp)

Segundo, as portas. Este é o curso de entrada do catálogo, o que significa que ele termina onde cursos mais profundos começam, e essas fronteiras são desenhadas de propósito. Profundidade de Solana, o modelo de contas como sistema, programas, PDAs, histórico da blockchain, mora no curso btc-to-sol-evolution; a gente ensina só o modelo mínimo do lado do cliente no módulo oito. UX de carteira, aterrissagem de transação, e tudo sobre conseguir incluir uma transação sob pressão pertence ao curso de domínio do lado do cliente, que está em produção enquanto eu escrevo isto; até ele ser entregue, a porta é o próprio tópico, e o painel do nosso módulo três constrói o piso de consumidor de React em que esse tipo de trabalho se apoia. Anchor e autoria de Rust on-chain moram no curso Anchor V2 e aquela porta merece uma placa direta: a barra dele é que você já entrega programas Anchor e quer o delta que o V2 traz, então a nossa saída de Rust, ler e escrever structs, enums, traits e Result, com os lifetimes sinalizados, te compra o nível de leitura para a prosa dele, não um lugar na barra dele. Quando uma lição aqui se recusa a ir mais fundo num desses tópicos, ela vai nomear a porta em vez disso. A mesma costura, em tamanho de catálogo.

## Lab: rode as sondas, tome a decisão

Numerado e curto, porque o ponto de hoje é a decisão, não o ferramental. Tudo aqui é zero instalação por design; nenhum passo exige Node, uma conta ou um download.

1. **Rode a sonda de latência.** Abra https://rust-lang.org numa aba (endereço apex, sem www, igual na abertura), abra o devtools (F12, aba Console), cole o one-liner de `fetch` do topo desta lição, aperte Enter. Aqui está ele de novo para você não ter que rolar a tela:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

   Copie a linha impressa, algo com a forma de `rust-lang.org: 238.5 ms (status 200)`, numa nota de rascunho. Essa linha é o seu primeiro artefato. Como o snippet sonda `location.origin`, o site em que a aba está, você pode colar ele no console de qualquer outro site e ele simplesmente sonda aquele site em vez deste, e continua funcionando lá pelo mesmo motivo que funcionou aqui: uma página sempre pode ler as respostas da própria origem. Sondar uma URL de terceiros da aba de outra pessoa é outro jogo. O navegador só te entrega uma resposta cross-origin legível quando o servidor alvo opta por permitir com um header CORS, e a maioria não opta; adicionar `mode: "no-cors"` junto de `cache` impede a falha escancarada, mas o navegador então devolve uma resposta opaca e o snippet imprime `status 0`. Um 0 ali quer dizer "opaco de propósito", não um site morto. Sondar na mesma origem é a versão que sempre funciona e sempre mostra um status real, e é por isso que o passo um começa ali.

2. **Rode mais quatro vezes.** Mesma colagem, mais quatro Enters. Veja o número se mover. Conexões frias, cache de DNS, clima de rota; latência é uma distribuição, não um valor, e você acabou de descobrir isso com a paciência de um for-loop. Anote a sua mais rápida e a mais lenta. A CLI da próxima lição transforma exatamente essa repetição em código.

3. **Dispare a captura do compilador.** Abra https://www.typescriptlang.org/play/ e digite o snippet de três linhas da abertura. Não cole; digite, e veja quão cedo o rabisco aparece. Passe o mouse em cima e leia o erro completo, incluindo a correção sugerida. Tire um print ou copie o texto do erro para a mesma nota de rascunho. Segundo artefato.

4. **Quebre mais ainda.** Ainda no Playground, mude a linha dois para o arquivo ficar assim:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeoutMs + probe.url;
console.log(`waiting ${wait} ms`);
```

   Leia o que o compilador diz sobre somar um número com uma string. Ele permite (as regras do JavaScript permitem) mas passe o mouse no tipo do resultado e note que `wait` agora é uma string. O compilador não é um linter gritando não; é um guarda-livros que sempre sabe qual tipo você tem de verdade. Dois minutos de cutucar aqui compensam por todo o módulo dois.

5. **Abra a checagem de honestidade.** Vá na página MDN Learn Core Scripting da tabela, passe o olho na lista de lições, e abra especificamente a unidade de async dela. Leia uma página ou duas. Depois tome a decisão, uma palavra na sua nota de rascunho: pronto, ou MDN-primeiro. As duas respostas passam neste lab. A única resposta que falha é a não examinada.

6. **Guarde o mapa.** Salve a tabela desta lição como bookmark no sistema que você realmente revisita. É a única peça de hoje que você vai usar por meses.

Checkpoint: a sua nota de rascunho guarda uma linha de latência real, um erro de compilador real, e um veredito de uma palavra. É essa a trava inteira. Trinta segundos de artefatos, uma decisão honesta.

## Challenge

**Completion (todos):** aponte a sonda para outro lugar. Qualquer site que te interesse: seu próprio projeto, seu site de docs favorito, seu banco. Mesmo one-liner, aba nova, alvo novo, número novo. Se o fetch falhar onde o rust-lang.org teve sucesso, leia o erro do console e arrisque um chute sobre o porquê; agora você tem um mistério que a discussão de política de segurança num módulo mais adiante vai resolver.

**Solo (opcional, sem walkthrough):** no Playground, escreva um objeto `probe` que guarda um array de URLs alvo e um `timeoutMs`, depois escreva uma linha que acessa uma propriedade que você não definiu e uma que indexa além do que você sabe que o array guarda. Veja qual das duas o compilador captura nas configurações padrão e qual ele deixa passar. Você acabou de encontrar, por conta própria, a lacuna exata que uma flag de strict mode no passeio pelo tsconfig da próxima lição existe para fechar. Traga a sua descoberta com você.

## Confira seu rumo

Três perguntas francas antes de você seguir. Você consegue rodar as duas sondas do zero, sem esta lição aberta? Você consegue explicar a costura 80/20 para outro dev em duas frases, incluindo o que é a caixa dos 20%? Você realmente abriu o MDN e tomou a decisão, ou você balançou a cabeça para o parágrafo e seguiu rolando? O quiz desta lição sonda as mesmas bordas: o que o contrato manda fazer quando um erro de compilador menciona território de bookmark, de que a divisão de bytes da drift é evidência, qual é a atitude honesta quando a unidade de async é nova para você, e o que de fato depende de o repo de estação ser público. Se qualquer resposta parecer mole, o lab leva cinco minutos para rodar de novo.

Me alonguei na seção do mapa; foi de propósito, e não vai ser o padrão. Você acabou de sondar a internet de um console de navegador, e aquela medição morre quando a aba fecha. Na próxima lição você instala a toolchain de verdade, Node 24 LTS e TypeScript 7 (a tocha de LTS do Node passa para 26 em 2026-10-28; a lição fixa o que ela verifica no dia em que você roda), e você transforma aquele one-liner em `pulse` v0: uma sonda que vive num repo, com tipos e tudo, com nada entre você e ela além de uma instalação. Deixe a aba do console aberta até lá se você quiser; no fim da próxima lição você não vai precisar dela.
