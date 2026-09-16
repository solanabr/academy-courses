# Conclusão: o mapa que agora é seu

Na lição passada você montou a estação inteira e provou isso: o script de demo passou de ponta a ponta, a extensão solo foi entregue sem apoio nenhum por perto, e o README mais o runbook querem dizer que outro dev conseguiria operar o que você construiu sem você na sala. Não sobrou nada para construir. Então esta lição abre do jeito que o curso abriu: te fazendo medir alguma coisa. Desta vez, a alguma coisa é você.

Abra uma aba nova, vá em https://rust-lang.org (apex, sem www, igual à lição um), aperte F12, clique na aba Console, e cole o snippet exato da lição um:

```js
const t0 = performance.now();
fetch(location.origin, { cache: "no-store" })
  .then(r => console.log(`${location.host}: ${(performance.now() - t0).toFixed(1)} ms (status ${r.status})`));
```

Depois vá em https://www.typescriptlang.org/play/ (com barra no fim, igual sempre) e cole o segundo:

```ts
const probe = { url: "https://www.rust-lang.org", timeoutMs: 3000 };
const wait = probe.timeout;
console.log(`waiting ${wait} ms`);
```

O mesmo punhado de linhas. O mesmo rabisco vermelho embaixo de `timeout`. O código não mudou um caractere desde o módulo um. Agora faça a parte que importa: pegue qualquer coisa em que você possa escrever e anote cinco linhas sobre o que você vê AGORA e não conseguia ver antes. Não capriche nelas. As minhas, rodando de novo enquanto escrevo isto: a forma da resposta é uma união que eu modelaria como variantes ok ou error antes de tocar nela; este fetch não tem timeout, não tem retry, não tem backoff, e eu sei exatamente qual das minhas próprias funções resolve isso; `performance.now` é um relógio monotônico e eu sei por que isso importa para medir; este código roda em um lugar só e eu já entreguei a mesma sonda para quatro; e o rabisco não é um linter sendo chato, é uma prova sobre o meu programa para a qual eu agora projeto de propósito. O código não mudou. Você mudou. O resto desta lição é o mapa de exatamente quanto, e de para onde as estradas levam a seguir.

## Resumo

Esta é a conclusão, e ela não ensina nada de novo, de propósito. Ela faz quatro coisas. Ela reafirma a construção degrau por degrau, uma frase franca cada, o que é recuperação espaçada vestindo fantasia de volta olímpica. Ela reimprime a tabela de ensinado-versus-bookmark da lição um, fechando a promessa que aquela lição fez, e depois reordena ela por urgência para a rota que você escolher. Ela lê o seu nível de saída contra a frase de pré-requisito que cada curso irmão declara, com evidências em vez de vibe. E ela entrega a última habilidade: o hábito de reverificar todo número que este curso imprimiu e que vai apodrecer. O lab produz três artefatos pequenos e nenhum deles é passivo. Depois a porta.

## O mapa, fechado

### A construção, degrau por degrau

Aqui está o que você de fato fez, módulo por módulo. Leia devagar; toda linha é uma coisa que você pode abrir no seu próprio GitHub agora.

R0, módulo um: o `pulse` v0, uma sonda estrita em TypeScript que buscava uma URL e imprimia a latência dela, promovida a um cron do GitHub Actions que commita `status.json` de volta num repo público. A sua primeira entrega foi um batimento numa máquina que não é sua. R1, módulo dois: a frota ganhou tipos, uma união discriminada para resultados de sonda, validação com zod na fronteira de config, um pool de concorrência feito à mão com cancelamento de verdade, e uma suíte vitest que trava o cron. R2 e R3, módulo três: o motor foi extraído para o `pulse-core` dentro de um workspace pnpm, um painel React entrou no ar numa URL do Vercel que qualquer estranho pode abrir, e o pacote publicou no npm onde qualquer pessoa pode instalar ele. R4, módulo quatro: o motor foi reescrito em Rust, a união virou um enum, erros viraram `Result` com thiserror, a máquina de estados ganhou testes de transição, e clippy mais fmt entraram na trava do CI. R5, módulo cinco: o serde leu o mesmo arquivo de config que a frota TypeScript lê, o workspace se dividiu em crates, uma CLI com clap virou a fachada dele, e o CI produziu um binário de release que uma máquina limpa consegue rodar. R6, módulo seis: o poller se mudou para uma imagem Docker multi-stage, uma ordem de magnitude menor que a ingênua, publicada no GHCR, com o compose rodando a estação local. R7, módulo sete: sondas chegaram ao edge nos Cloudflare Workers duas vezes, uma em TypeScript e uma em Rust compilado para WebAssembly, as duas guardando o último status conhecido em KV. R8, módulo oito: a Solana entrou na lista de alvos, leituras do kit no painel, sondas de blockchain no poller, e uma transação real na devnet confirmada com a sua própria chave descartável. R9, módulo nove: auditorias sobre a árvore inteira, logs estruturados que você consegue passar no grep durante um incidente, e um alarme no próprio monitor. R10, última lição: a estação inteira, verificada aresta por aresta de uma máquina só, transcrição guardada, com um runbook.

![Uma escada sobe de uma única sonda agendada, passando por frotas tipadas, motores Rust, contêineres, edge workers e alvos Solana, até uma estação montada.](assets/v01-timeline.webp)

Note o que a escada não é: ela não é dez projetos. Ela é um artefato que nunca foi jogado fora. O one-liner que você rodou de novo dez minutos atrás ainda está reconhecivelmente dentro do binário de release, dos workers e do contêiner. Esse é o argumento do acréscimo em que este curso apostou, e você é a evidência de que funcionou.

A forma da estação, uma última vez, porque a lição um prometeu que você encontraria este desenho de novo e a lição passada te fez redesenhar ele de memória. Duas notas francas antes de você comparar com o gabarito da lição passada, porque os dois desenhos deliberadamente não são a mesma figura. Esta reimpressão é o desenho da ABERTURA DO CURSO, centrado no repo: o repo público no hub, quatro raios, desenhado antes de o módulo oito existir, então sem raio de `tx-check`. A referência operacional da m10-l1 redesenhou o mesmo sistema centrado no pipeline: o Actions no hub, os workers divididos em dois raios, o `tx-check` contado, cinco raios no total. Mesmos componentes, mesmas arestas reais, dois centros de gravidade, e os dois são verdade sobre um sistema só: a abertura pergunta onde os dados moram (o repo), o runbook pergunta o que bate (o pipeline). Ver que os dois desenhos descrevem a sua única estação, e saber para qual correr por pergunta, já é uma habilidade de nível de formatura; se você quiser que este bata com o do runbook, adicionar o raio do `tx-check` é o único delta.

![Um sistema de hub e raios se centra num repositório público com todo raio agora construído, verificado e marcado como completo em volta de alvos de sonda reais.](assets/v02-diagram.webp)

### A tabela, reimpressa

A lição um imprimiu uma tabela do que este curso ensina versus o que ele deixa como bookmark e chamou ela de gêmeo franco da ementa. Ela também prometeu que o módulo final te traria de volta para ela. Esta é essa visita, e a costura 80/20 é conteúdo do curso até o fim. É uma reimpressão com um delta franco, sinalizado em vez de contrabandeado: a linha de concorrência sem medo abaixo é nova hoje, adicionada na saída porque dez módulos de prática de ownership finalmente conquistaram um lugar para ela; a tabela da lição um nunca carregou ela. Os links abaixo são os mesmos recursos canônicos, verificados ao vivo durante a passada de pesquisa deste curso e sondados de novo em 2026-09-02, a data que a lição um imprime. Um deles é também uma pequena piada às custas do curso: em meados de junho de 2026, enquanto a nossa pesquisa estava sendo montada, o Frontend Masters virou Master.dev e toda URL antiga começou a redirecionar. Um rebrand caiu dentro da nossa própria lista de links e demonstrou, ao vivo, por que o hábito de reverificar vale para bookmarks também. Links apodrecem. Links verificados apodrecem com atraso.

| Conceito | Onde morou | Bookmark para |
|---|---|---|
| Fundamentos de JS | Nunca ensinado; o pré-requisito | MDN Learn Core Scripting: https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting |
| Config estrita de TypeScript | M1 | intro do TS Handbook: https://www.typescriptlang.org/docs/handbook/intro.html e Everyday Types: https://www.typescriptlang.org/docs/handbook/2/everyday-types.html |
| Uniões discriminadas, narrowing, exaustividade | M2 | narrowing do Handbook, seção de uniões: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#discriminated-unions ; o repo type-challenges no GitHub como o treino |
| Generics que você realmente usa | M2 | generics do Handbook: https://www.typescriptlang.org/docs/handbook/2/generics.html |
| Validação nas fronteiras | M2 | tutorial gratuito de Zod do Total TypeScript: https://www.totaltypescript.com/tutorials |
| Async, concorrência, cancelamento | M2 | unidade de async do MDN; TRPL ch17 para o lado Rust: https://doc.rust-lang.org/book/ch17-00-async-await.html |
| Prática de testes | M2 (vitest), M4 (cargo test) | guia do vitest; docs do node:test |
| Saber empacotar e publicar | M3 | trilha de aprendizado do Node: https://nodejs.org/learn |
| React, nível consumidor | M3 | quick start do react.dev; a profundidade de cliente pertence ao curso irmão do lado do cliente (em produção), leia contra a seção das portas abaixo |
| Ownership e borrowing | M4 | O fork interativo da Brown, quizzes e visualizações de ownership: https://rust-book.cs.brown.edu/ |
| Erros como valores | M4 | capítulo de tratamento de erros do The Rust Book: https://doc.rust-lang.org/book/ch09-00-error-handling.html |
| Enums, structs, traits na prática | M4 | capítulo de enums do Book: https://doc.rust-lang.org/book/ch06-00-enums.html e generics e traits: https://doc.rust-lang.org/book/ch10-00-generics.html |
| serde, cargo, clap, uma CLI de verdade | M5 | serde.rs; o Cargo book; Rustlings depois de cada módulo de Rust: https://rustlings.rust-lang.org/ |
| tokio, profundidade de saber-quando-precisa | M6 | TRPL ch17 async, mesmo capítulo de cima: https://doc.rust-lang.org/book/ch17-00-async-await.html |
| Concorrência sem medo, threads de verdade | Nunca ensinado; linha adicionada hoje | o ch16 do Book, concorrência sem medo, a partir do sumário em https://doc.rust-lang.org/book/ |
| Amplitude de sintaxe de Rust | Nunca reensinado | Rust by Example: https://doc.rust-lang.org/rust-by-example/ ; Comprehensive Rust: https://google.github.io/comprehensive-rust/ |
| Lifetimes além da leitura, unsafe, macros | Nunca, sinalizado | The Rustonomicon, que abre dizendo para você não ler ainda: https://doc.rust-lang.org/nomicon/ |
| Contêineres | M6 | Docker Get Started: https://docs.docker.com/get-started/ |
| Fazer deploy do app TS | M3 | getting started do Vercel: https://vercel.com/docs/getting-started-with-vercel |
| O edge, as duas linguagens | M7 | get started do Cloudflare Workers: https://developers.cloudflare.com/workers/get-started/guide/ |
| Profundidade de Solana, Anchor, UX de carteira | Modelo mínimo de cliente, M8 | Cursos irmãos deste catálogo, leia contra as próprias placas deles abaixo |

Na lição um esta tabela era uma promessa. Hoje ela é uma ordem de leitura, e ordens de leitura são pessoais. Então reordene ela. Quais bookmarks acabaram de ficar urgentes depende inteiramente de para onde você vai, mas três promoções valem para quase todo mundo que sai deste curso.

Primeiro: TRPL ch16, concorrência sem medo. No momento em que o seu próximo serviço Rust precisar de threads de verdade em vez de tasks do tokio, aquele capítulo para de ser opcional. Você já tem tudo o que ele pressupõe, ownership incluído, que é exatamente por que ele pode entrar no mapa hoje como bookmark em vez de capítulo. Segundo: lifetimes além da leitura. Se a sua rota é autoria de programas, leia a relação real do curso Anchor com eles antes de estudar em pânico: os labs V2 dele tiram as anotações `<'info>` antigas das structs de conta em vez de exigir novas, e o que os checkpoints dele exigem de fato é raciocínio de borrow, seguindo o argumento do compilador sobre qual handle está vivo quando. O hábito dele de rampas de entrada na hora exata cobre traits e marker types, não lifetimes, então o capítulo profundo de lifetime continua sendo o seu bookmark, vencendo na hora da necessidade, não antes. Terceiro: type-challenges, a academia de TS. Você escreve uniões discriminadas todo dia agora; os treinos transformam isso de um padrão que você usa num músculo que é seu. Escolha os seus três. Amarre cada um a uma rota. Ignore o resto até vencerem, porque um acúmulo de bookmarks é só a cilada de mil horas com organização melhor.

![Bookmarks idênticos aparecem primeiro como uma lista plana de promessas, depois reordenados em tiers de urgência que um formando atribui com base numa rota escolhida.](assets/v03-comparison.webp)

### Lendo as portas contra as próprias placas delas

Agora a parte que eu me recuso a resolver com acenos de mão, porque reivindicar demais a sua saída desconquistaria tudo o que as caixas da honestidade construíram desde a lição um. Cada curso irmão deste catálogo declara o próprio pré-requisito. O jeito certo de sair daqui é ler essas frases do jeito que você agora lê um Cargo.toml: como afirmações a verificar contra evidências que são suas. Então vamos fazer exatamente isso, porta por porta.

O curso de domínio do lado do cliente, a trilha avançada deste catálogo para carteiras, aterrissagem de transação e dados de frontend, está em produção enquanto eu escrevo, o que quer dizer que a placa dele ainda não foi impressa e eu não vou inventar uma para citar. O que você pode comparar com qualquer barra de trilha de cliente são comprovantes, não adjetivos: uma frota de sondas tipada com um núcleo de união discriminada, fronteiras com zod que falham alto na inicialização, um pool de concorrência com cancelamento que você escreveu à mão e depois testou com fake timers, e o painel React que você entregou no Vercel no módulo três e entregou de novo com painéis novos duas vezes depois disso. Se o seu objetivo é a metade de cliente da Solana, aquele curso é a porta; até ele ser entregue, a porta é o próprio tópico, UX de carteira e aterrissagem de transação, e você chega carregando evidências.

O curso Anchor, a trilha de framework para escrever programas on-chain, usa a placa mais direta do catálogo, e ler ela como ela está escrita é o momento de integridade desta lição: ele pressupõe que você já entrega programas Anchor e quer o delta do V2. Essa é uma barra de entrega, e nenhum curso de fundamentos vence uma barra de entrega por si só. O que você tem daqui é real e parcial: você escreve structs, enums, traits, `Result` com thiserror e serde todo dia, e você lê Rust idiomático bem o bastante para revisar ele, que é o nível de leitura que a prosa daquele curso pressupõe. O que você não tem, ele divide em dois: contas, PDAs e transações como conceitos, que moram no curso da evolução do Bitcoin à Solana, o curso de conceitos que caminha por como o modelo de dados de uma blockchain funciona e por que o da Solana tem a forma que tem; e reps de programa entregue, que nenhum curso te dá. Então a rota franca para autoria de programas passa primeiro pelo curso de conceitos, depois pelos primeiros programas escritos e entregues, e só então pela porta do Anchor V2. Só confiança em Rust não vence uma barra de entrega, e fingir o contrário te manda para labs avançados sem o modelo nem as reps que eles pressupõem.

Leia a própria placa do curso do Bitcoin à Solana com cuidado, porque ela é mais estreita do que o folclore faz parecer: o curso nunca afirma que não tem código, a descrição dele promete que você vai "rodar, escrever scripts e explicar" e entregar um bot de ops, e a tranquilização que ele de fato imprime tem escopo de lição e é só de Rust, "Nunca escreveu uma linha de Rust? Ótimo, você não precisa." Contra as suas evidências o veredito continua confortável: você se sobrequalifica em todo piso de código que ele tem, Rust mais que tudo, onde o pedido dele é zero e a sua prática é diária. Pegue ele pelos conceitos, passe rápido pelo que você já sabe, e trate os labs dele como reps rápidas em vez de reps puladas; um curso estar abaixo do seu nível de código não põe as ideias dele abaixo do seu nível de ideias.

![Um fluxograma roteia um formando por três portas de curso, mostrando uma barra atendida, uma barra de duas partes com um desvio de conceitos, e um caminho de conceitos aberto.](assets/v04-flowchart.webp)

Mais duas portas do catálogo merecem ter as placas lidas, porque a estação mapeia nelas de forma desigual e dizer como é para isso que esta seção existe. O curso de pagamentos e comércio na Solana é a porta seguinte que melhor encaixa no conjunto de habilidades da própria estação: o meio de back-office dele, um serviço de checkout, tratamento de webhook, um worker de liquidação com retries e backoff, roda exatamente nos músculos que este curso treinou, um serviço TS tipado com parse na fronteira, um loop de worker agendado, backoff com jitter, e disciplina de retry no caminho de leitura, então você chegaria lá gastando a sua atenção em semântica de pagamentos em vez de encanamento. O curso Digital Assets é uma porta de verdade com uma placa mais alta, e a leitura franca é uma ordem de leitura em vez de um muro: a metade de PDA da barra dele pertence ao curso de conceitos, e o pano de fundo de token account em que o próprio primeiro módulo dele se apoia é algo que este curso conscientemente não forneceu, então você chega lá devendo isso. m04-l2 gastou a cifra do rent de ATA como constante trabalhada e disse na hora que o que uma ATA de fato É pertence àquele curso, não a este. Eu não vou inventar uma rota para você: nada aqui preenche essa lacuna e eu não conferi a ementa de todo curso irmão em busca de uma, então trate isso como uma noite de leitura que você agenda antes do módulo um dele em vez de durante.

Antes da última porta, a parte do mapa que faz o resto dele ser confiável: o que você AINDA NÃO é. Você não é um autor de programas; essa estrada passa por conceitos que este curso nunca ensinou e por reps de programa entregue que ele nunca passou, antes de a porta do curso Anchor nem abrir. Você não é um dev de sistemas Rust fluente em lifetimes; você lê lifetimes quando o compilador imprime eles, e o material profundo está como bookmark, não absorvido. Você não é um engenheiro de UX de carteira ou de aterrissagem de transação; você entregou um painel que lê uma blockchain, que é uma coisa diferente de pastorear a transação de um usuário até dentro de um bloco, e o curso do lado do cliente existe porque essa diferença é uma disciplina inteira. Escrever essas três frases me custa alguma coisa, porque todo curso quer afirmar que os formandos dele conseguem fazer tudo. Mas o valor de um mapa SÃO as bordas dele. Um mapa de saída sem coluna de ainda-não é um anúncio, e você passou dez módulos aprendendo a desconfiar desses.

Mais uma porta merece a menção pelo nome, e uma divulgação repetida da lição um. Em 2023-04-25, o ThePrimeagen entregou Rust for TypeScript Developers no que hoje é o Master.dev, 5 horas 19 minutos, pago com preview gratuito. O mercado nomeou o público exato deste curso três anos antes deste curso existir. Você acabou de caminhar a estrada inteira para onde aquele título aponta, nas duas direções. Se você quiser uma segunda voz na metade de Rust agora que você já entregou com ela, essa segue sendo a única opção paga que este curso nomeia.

### Os números que vão apodrecer, e o hábito que não vai

Todo dígito de versão que este curso imprimiu foi verificado no dia em que foi escrito, e cada um deles está morrendo num cronograma que ninguém publica. Isso não é um defeito do curso. Isso é o terreno, e a última coisa que este curso ensina é o reflexo que sobrevive a ele. Três casos, cada um carregando a sua regra.

Caso um, o mais agudo do arquivo de pesquisa inteiro: entre 2026-06-16 e 2026-08-21, o @solana/kit entregou um minor e depois dois majors. 6.10.0 em meados de junho, 7.0.0 duas semanas depois, 8.0.0 no fim de agosto. Pouco mais de nove semanas, dois bumps de major. Durante a própria janela de pesquisa deste curso, as nossas notas internas ficaram desatualizadas naquele dígito duas vezes. Eu tive o prazer de ver a nossa própria documentação apodrecer em tempo real enquanto escrevia um curso sobre não confiar em documentação desatualizada, o que é mais ou menos tão humilhante quanto parece. A regra que sobrevive: fixe aquilo contra o que as suas dependências dão peer, por workspace, e sonde de novo antes de todo install novo. O dígito nunca foi o conhecimento. A sonda é o conhecimento.

![Um release minor e dois major de um pacote caem em pouco mais de nove semanas ao longo de uma linha do tempo de verão, cada marcador datado a partir do registro.](assets/v05-chart.webp)

Caso dois, runtimes. Este curso fixou o Node 24 LTS e te disse, com uma nota de rodapé datada, que a tocha do LTS passa para a v26 em 2026-10-28. Essa data vem do cronograma de releases publicado do Node, que é o ponto inteiro: runtimes apodrecem com educação, em calendários que você consegue ler. Então leia o cronograma, não um post de blog sobre o cronograma. Quando você fizer o scaffold de um projeto em março, a pergunta nunca é o que o meu curso disse, é o que o cronograma diz hoje.

Caso três, a própria blockchain. A Solana tem como meta slots de 300ms e, quando este curso mediu vinte amostras recentes em 2026-09-01, a rede ficou numa média de 316ms. Metas são marketing até serem medidas, e a sua estação mede. Esse hábito, rodar a sua própria sonda em vez de citar o número de alguém, é o mesmo reflexo numa camada diferente, e você agora construiu ele dentro de uma máquina que exercita ele a cada trinta minutos sem você.

Onde os dígitos moram, então? Na sua tabela de pins, a que o seu runbook carrega desde a lição passada: toda versão de que esta estação depende, em um lugar só, cada uma com a data em que você verificou. Alguns dos seus vão se parecer com isto, com as suas próprias datas na última coluna:

```markdown
| surface        | pin                        | why                              | verified   |
|----------------|----------------------------|----------------------------------|------------|
| Node           | 24 LTS                     | active LTS line; v26 2026-10-28  | 2026-09-02 |
| @solana/kit    | what deps peer against     | probe peers before install       | 2026-09-02 |
| rust toolchain | current stable via rustup  | six-week train; clippy in CI     | 2026-09-02 |
```

A coluna do meio é que está fazendo o trabalho de verdade. Uma tabela de pins que só guarda dígitos é uma lista de mentiras futuras; uma tabela de pins que guarda a regra ao lado de cada dígito é um manual de manutenção. Refixar é uma edição de uma linha e o CI que você construiu julga cada novo pin de graça. O hábito, dito uma vez, sem rodeios, para você poder repetir ele para outra pessoa: números em sistemas rodando são snapshots; guarde eles em um arquivo só, date eles, e sonde de novo na hora da necessidade em vez de confiar em qualquer dígito congelado, inclusive os deste curso. Especialmente os deste curso. Daqui a um ano, desconfie de todo número de versão impresso aqui e confie no método que produziu eles.

![Um loop pequeno vai de um trigger, passando por uma sonda ao vivo e uma comparação, atualizando ou reestampando uma tabela de pins datada de um jeito ou de outro.](assets/v06-flowchart.webp)

## Lab: três artefatos, nenhum passivo

O apoio desapareceu desde a lição passada; este lab é instruções, não passos que você segue comigo. Ele produz três artefatos escritos pequenos. Trinta minutos, nada para instalar, e tudo o que você escreve cai no repo da estação, então é entregue como todo o resto foi.

1. **Termine o diff antes-versus-agora.** Você escreveu cinco linhas cruas no topo desta lição. Limpe elas em cinco linhas de verdade e salve como `docs/then-vs-now.md` no repo da estação. O teste para cada linha: ela precisa nomear algo específico que você agora vê naqueles dois snippets, uma união, um backoff faltando, uma escolha de runtime, um custo, uma prova. "Eu sei mais TypeScript agora" não passa no teste. "A forma desta resposta é uma união e eu modelaria ela antes de tocar nela" passa.

2. **Escreva a afirmação de rota.** Um curso irmão, nomeado, com uma afirmação de duas linhas de que você atende a barra declarada dele, ou de exatamente como você vai atender. Evidência quer dizer apontar para coisas: o painel entregue, a máquina de estados em enum, o curso de conceitos que você vai fazer primeiro. Adicione no fim do mesmo arquivo. Se você não consegue escrever a afirmação em duas linhas, você ainda não escolheu uma rota, e isso vale mais saber hoje que três semanas dentro do curso errado.

3. **Monte a lista de leitura dos próximos três.** Adicione ela embaixo da afirmação de rota, mesmo arquivo, `docs/then-vs-now.md`; os três artefatos viajam como um registro de formatura só. Da tabela reimpressa, escolha exatamente três bookmarks. Para cada um, uma linha: o bookmark, e uma cláusula de porque amarrando ele à sua rota. "TRPL ch16, porque a minha rota é o poller Rust ganhando threads de verdade" é a forma. Três, não sete. A disciplina é o artefato.

4. **Faça o commit.**

   ```bash
   git add docs/then-vs-now.md
   git commit -m "docs: graduation record (then-vs-now, route, next three)"
   git push
   ```

   A sua estação agora carrega o próprio registro de formatura, publicamente, ao lado do código que conquistou ele.

5. **Confira o batimento.** Abra a aba Actions do seu repo e confirme que o cron continua verde e que o `status.json` se moveu na última hora. A estação continua batendo, esteja você olhando ou não. É isso que você construiu.

## Challenge

Rode o hábito de reverificar uma vez, de verdade, antes que ele tenha chance de sumir. Abra o workspace TS da sua estação e sonde aquilo contra o que as suas dependências dão peer hoje: `npm view` (o npm está na sua máquina desde que o Node chegou no módulo um) contra o `peerDependencies` dos seus pacotes vizinhos do kit, a jogada exata do módulo oito. Compare a resposta com a sua tabela de pins. Se elas concordam, adicione a data de hoje na coluna verified da tabela e você terminou em cinco minutos. Se elas discordam, você pegou a sua primeira deriva de verdade, e você sabe exatamente o que fazer: atualize o pin, anote a data, rode a suíte, deixe o CI julgar. Qualquer um dos desfechos é uma vitória; o ponto é que você rodou a sonda num dia em que ninguém te mandou.

## Confira seu rumo

O último checkpoint do curso tem trinta segundos e é em voz alta. Leia os seus três artefatos para alguém, ou para a sala vazia: cinco linhas de diff, uma afirmação de rota, três cláusulas de porque. Se soarem como evidência, você terminou aqui. Essa leitura em voz alta é também o autoteste franco: uma linha de diff que você atropela resmungando é uma que você deveria afiar, e uma afirmação de rota que você não consegue dizer com a cara séria é uma rota que você não escolheu de verdade. O quiz desta lição caminha pelo mesmo terreno: a leitura franca da porta do Anchor, o que o hábito de reverificar diz quando uma linha de install antiga falha, e o que um erro de lifetime surpresa quer dizer para um formando que é dono do mapa.

E com isso, o aha que este curso inteiro foi construído para entregar, dito sem rodeios: os 20% nunca estiveram faltando. Nunca foram um buraco na sua educação. Você agora sabe exatamente onde cada peça deles mora, capítulo por capítulo, e, mais importante, você sabe quando vai precisar de cada peça, porque a sua rota te diz. Um erro de lifetime num lab de Anchor não é uma lacuna. É um bookmark vencendo, e você vai retirar ele como um item reservado.

Não existe próxima lição. Existe um próximo curso, e você agora consegue ler a frase de pré-requisito dele com evidências na mão. Enquanto isso a estação segue batendo no cron dela enquanto você vai: pública, no seu GitHub, uma peça de portfólio que responde à única pergunta de entrevista que importa, você consegue entregar, com uma URL em vez de um parágrafo. Escolha a sua rota. Abra o seu primeiro bookmark. Entregue.
