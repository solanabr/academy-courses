# Cadeia de suprimentos: o ataque de 86 minutos

## Resumo

m08-l4 provou o caminho de escrita. A estação assinou e aterrissou uma transferência de SOL de verdade na devnet com o fluxo de pipe do kit, o `tx-check` guarda o comprovante, a caixa de honestidade do airdrop foi exercitada pra valer, e o fallback de validador local está instalado e foi usado pelo menos uma vez. A sua stack lê a blockchain a partir das duas linguagens e consegue provar que ela aterrissa transações. Então hoje a gente olha para o que tudo isso está apoiado: cinco lockfiles (o do workspace TS, o do workspace Rust, os dos dois projetos de edge, e o do `tx-check`, o scaffold standalone que o m08-l4 construiu em volta do keypair de assinatura), várias centenas de pacotes que você não escreveu, e um changelog em que outras pessoas, algumas delas hostis, podem dar append. Nesta lição você roda `npm audit` e `cargo audit` contra a sua própria árvore, aprende a ler o que volta até o nível de advisory id, faixa vulnerável e caminho de dependência, formaliza em uma checklist a leitura de sinais que você fez informalmente em m03-l4, escolhe uma filosofia de pin que você consiga defender, e commita o `AUDIT.md` no repo da estação. Como as reps rodam: uma leitura de advisory de ponta a ponta comigo, a checklist passada pra você como artefato, todo veredito depois do primeiro escrito sem guia. O capstone vai tirar a checklist também.

## A superfície de ataque com um changelog

Em 2026-08-20, no mesmo dia em que o Rust 1.98.0 foi entregue, um atacante tomou a conta de um autor de crate e republicou o `arrayref` como 0.3.10, com um build script que baixava um payload. Ficou no ar por 86 minutos antes de receber yank. Por que essa data pertence a um curso de Solana: o manifest do programa do SPL Token depende de `arrayref = "0.3.9"`. A versão maliciosa ficava a uma aresta de dependência de todo token na Solana.

Antes de desempacotar qualquer coisa disso, aponte as ferramentas para a sua própria árvore. A partir da raiz do repo da estação, onde o `pnpm-lock.yaml` mora:

```bash
pnpm audit
```

Esse é o `npm audit` canônico para um workspace pnpm: o mesmo endpoint de auditoria do registro, lido a partir do lockfile do pnpm. Em qualquer repo com lock de npm a grafia é `npm audit`, e é essa a grafia que a documentação do ecossistema mais amplo descreve; você aprendeu em m03-l1 a ler o campo `packageManager` de um repo antes de digitar, e isso se paga de novo aqui. Depois o lado Rust. `cargo audit` é um subcomando do cargo que você instala uma vez:

```bash
cargo install cargo-audit --locked
cd pulse-rs
cargo audit
```

(o cargo-audit está em 0.22.2 enquanto eu escrevo isto, 2026-09-02; pegue o que o `cargo install` te der. A flag `--locked` constrói a ferramenta a partir do lockfile commitado dela mesma, o que, dado o assunto desta lição, é o único jeito respeitável de instalar uma ferramenta de auditoria.) Ela busca o RustSec advisory database e checa todo crate no `Cargo.lock` contra ele, que é por isso que você roda ela onde o lockfile mora, a raiz do workspace, não dentro de um crate membro que não tem nenhum.

Os dois comandos vão imprimir ou achados ou um resultado limpo. Segure o que você conseguiu; no fim da teoria você vai conseguir ler isso direito, e qualquer um dos dois desfechos passa nesta lição. A trava são os vereditos escritos, não uma árvore com sorte.

Agora a história, contada de forma direta, porque a linha do tempo é a lição.

### 86 minutos, minuto a minuto

O ataque do arrayref não foi código esperto. Foi uma tomada de conta: o atacante ganhou controle da conta do autor no crates.io e publicou uma nova versão de patch de um crate que não precisava de nenhuma. A 0.3.10 maliciosa nem carregava o payload no próprio código dela: ela adicionou uma dependência de um crate proc-macro de typosquatting, e o build script DAQUELE crate baixava e executava o payload em tempo de compilação, o que quer dizer que o alvo não eram os usuários dos usuários do crate, eram todo desenvolvedor e toda máquina de CI que fosse compilar a árvore dentro da janela. Dois crates irmãos do mesmo autor foram pelo mesmo caminho no mesmo incidente, append-only-vec por 107 minutos e internment por 90. A Nextron Systems detectou; a resposta de segurança do Rust deu yank nos três. O relato mora em blog.rust-lang.org, datado de 2026-08-20, e vale os seus dez minutos.

Sente com o detalhe do SPL Token por um segundo, porque é a parte que faz disto uma lição de Solana e não um sermão genérico de higiene. A linha de manifest `arrayref = "0.3.9"` parece um pin. Não é. A versão pelada do Cargo é semântica de caret, quer dizer que ela aceita qualquer upgrade compatível com semver, e 0.3.10 é compatível com 0.3.9 pelas regras. Qualquer resolução fresca de dependências feita durante aqueles 86 minutos, um clone novo, um job de CI sem lockfile commitado, um `cargo update`, teria selecionado a versão maliciosa sem ninguém pedir por ela. Todo `Cargo.lock` commitado segurou a linha na versão registrada nele e nunca esteve em perigo. Mesma faixa, mesma janela, dois desfechos completamente diferentes, e a única variável era se um lockfile estava ou não entre o manifest e o registro.

![Uma linha do tempo de um dia sombreia os 86 minutos em que o release malicioso do arrayref ficou no ar, com resoluções frescas expostas acima e lockfiles commitados seguros abaixo.](assets/v01-timeline.webp)

Esse é um dos dois incidentes fundadores. O outro é uma década mais velho e falhou na direção oposta.

### Dois jeitos de uma árvore te falhar

Março de 2016. Um desenvolvedor numa disputa de nome despublica os 273 pacotes npm dele, um deles um utilitário de onze linhas chamado left-pad, e builds quebram pelo ecossistema inteiro dentro da hora, porque milhares de pacotes, incluindo os maiores frameworks da época, dependiam dele transitivamente. O post-mortem do próprio npm, intitulado "kik, left-pad, and npm" e datado de 2016-03-23, continua no ar, e lê como o ecossistema descobrindo, em público, que o grafo de dependências dele era uma parede estrutural compartilhada que ninguém tinha inspecionado.

Repare que essas são classes de falha diferentes. arrayref foi inserção maliciosa: algo novo e hostil entrou na árvore por uma faixa de versão. left-pad foi desaparecimento súbito: algo velho e confiável saiu da árvore, e tudo que se apoiava nele caiu. A árvore da sua estação está exposta às duas, nos dois ecossistemas, e elas pedem defesas diferentes. Inserção é para o que servem as ferramentas de auditoria e os lockfiles. Desaparecimento é para o que serve a checklist de abandono mais adiante nesta lição, porque um pacote não precisa sumir numa tarde para desaparecer; a maioria deles só para de ser mantida em silêncio, e o README é o último a saber.

![Painéis lado a lado contrastam o ataque de inserção do arrayref com o desaparecimento do left-pad, cada um mapeado para a própria defesa.](assets/v02-comparison.webp)

### Lendo um relatório de auditoria como gente grande

De volta ao que quer que os seus dois comandos tenham imprimido. Um achado de auditoria, npm ou cargo, tem as mesmas cinco partes estruturais, e eu quero que você leia elas numa ordem deliberada, porque o layout do relatório sugere a ordem errada.

Um achado carrega um advisory id: com prefixo GHSA no mundo npm, com prefixo RUSTSEC no mundo Rust, um nome estável que você pode procurar, citar no `AUDIT.md` e conferir de novo no mês que vem. Ele carrega uma severidade, uma palavra só como moderate ou high. Ele carrega uma faixa vulnerável, as versões exatas afetadas, escrita na mesma linguagem de comparadores dos seus manifests, algo como `>=0.3.0 <0.3.11`. Ele carrega uma versão corrigida, o menor upgrade que sai da faixa. E ele carrega um caminho de dependência, a cadeia de arestas de algo que você escolheu até a coisa que é vulnerável.

Esta é a forma que um achado do cargo audit toma, com os campos rotulados do jeito que a ferramenta imprime eles:

```text
Crate:     <name>
Version:   <the version your Cargo.lock resolved>
Title:     <one-line description of the vulnerability>
Date:      <advisory publication date>
ID:        RUSTSEC-<year>-<number>
Solution:  upgrade to >= <patched version>
Dependency tree:
<name> <version>
└── <the chain of crates that pulled it in, up to your own>
```

A ordem em que se lê: id, faixa, versão corrigida, depois o caminho, e só então a severidade. A severidade avalia o bug no abstrato, como se todo usuário do crate rodasse ele na pior posição possível. O caminho avalia a sua exposição, e a sua exposição é a coisa sobre a qual você está de fato decidindo. Um advisory de severidade alta sentado numa devDependency do tooling de build do seu painel nunca é entregue no bundle de produção; ele ainda roda no seu laptop e na CI, então não é zero, mas é uma avaliação, não uma queda. O mesmo advisory num caminho de runtime em `pulse-fleet`, código que executa a cada trinta minutos com as suas credenciais em escopo, é outra manhã. Mesma string de severidade, decisões diferentes. Leia o caminho antes de agir; as ferramentas que imprimem severidade em vermelho estão otimizando para a sua atenção, não para o seu julgamento.

O caminho também é o campo que você consegue interrogar diretamente, e estes três comandos são os melhores amigos de quem lê auditoria:

```bash
pnpm why <package>
npm ls <package>
cargo tree -i <crate>
```

`pnpm why` e `npm ls` caminham do seu manifest para baixo até o pacote sinalizado; `cargo tree -i` inverte a árvore e caminha para cima do crate sinalizado até o que quer que seja seu que dependa dele. Dez segundos com esses ganham de dez minutos rolando saída de relatório.

![Um fluxograma roteia um achado de auditoria por checagem de faixa e rastreamento de caminho até uma resposta urgente de runtime ou uma avaliação agendada de build-time.](assets/v03-flowchart.webp)

Um aviso sobre o botão que o relatório te oferece. O `npm audit fix` vai reescrever a sua árvore para sair de faixas vulneráveis, e sob `--force` ele vai aplicar bumps de major de semver para fazer isso. Isso é uma ferramenta propondo migrações, não aplicando patches. Leia o que ele quer mudar antes de deixar; um conserto de segurança que pula uma dependência dois majors em silêncio trocou uma vulnerabilidade conhecida por uma quebra desconhecida, e você vai descobrir qual das duas custa mais no pior momento possível.

E a ressalva mais funda, a que faz a segunda metade desta lição existir: uma auditoria é sinal, não prova. O banco de advisories contém o que alguém achou, verificou e reportou. Um relatório limpo quer dizer que nenhum advisory conhecido casa com o seu lockfile. Não quer dizer que a sua árvore está segura; um crate abandonado que ninguém está vigiando pode ficar vulnerável em silêncio para sempre, acumulando não segurança, mas silêncio. O cargo audit checa todo crate no `Cargo.lock` contra o banco, nada é pulado por estar sem manutenção, então o ponto cego nunca está no que a ferramenta escaneia. Está no que o banco sabe.

### A checklist de abandono

É por isso que a ferramenta ganha um parceiro. Em m03-l4 você viu o tsup, o bundler TypeScript que reinou por muito tempo como padrão, anunciar a própria aposentadoria em uma linha de README: "Este projeto não é mais mantido ativamente. Considere usar o tsdown." Você leu esse sinal no mundo real uma vez. Agora a gente sistematiza a leitura, porque você estava fazendo cinco checagens no feeling e uma checklist é como uma habilidade sobrevive a ser passada para outra pessoa, incluindo você do futuro às 2 da manhã.

Para qualquer dependência que você esteja avaliando, existente ou em perspectiva, rode estas cinco leituras:

| Sinal | Onde olhar | O que isso te diz |
|---|---|---|
| Aviso no README | a página inicial do repo, primeira tela | Quem larga costuma dizer que largou. O tsup disse. Acredite já na primeira vez. |
| Data da última publicação | `npm view <pkg> time.modified`, página do crates.io | Quão recente é o release mais novo. Velho sozinho não condena; velho mais os outros sinais, sim. |
| Cadência de release | a página de releases, o CHANGELOG | Um projeto vivo tem ritmo. Um projeto cujo ritmo parou tem data de morte, mesmo sem anúncio. |
| Deriva de issues abertas | issues abertas versus fechadas nos meses recentes | Issues se empilhando sem resposta quer dizer que não tem ninguém em casa, diga o README o que disser. |
| O que os repos emblemáticos fixam | os manifests dos projetos sérios do ecossistema | O sinal mais forte, siga lendo, porque ele corta dos dois lados. |

Essa última linha merece o próprio parágrafo, e você já conheceu a evidência. A linha atual do reqwest é 0.13.4, e o agave, a implementação de validador, ainda fixa 0.12.28. O agave também fixa `clap = "2.33.1"` com as features padrão desligadas, o major de uma década que você leu a frio em m05-l2, enquanto a linha atual do clap é 4.x. Você leu os dois pins em m05-l2 e aprendeu a leitura a frio: um projeto emblemático segurando uma versão velha não é negligência, é uma decisão com custo tomada por gente com mais em jogo que você, e ela te diz que a linha velha ainda funciona e que o upgrade perdeu a briga de custo-benefício até agora. Agora abra `docs/pin-reads.md`, os três vereditos que m05-l2 te fez commitar, e marque cada um contra a checklist de cinco linhas acima, que você não tinha quando escreveu eles. Onde uma linha teria mudado o seu veredito, diga isso no arquivo e date a revisão em vez de apagar o original editando por cima. Dar nota para a sua própria leitura a frio contra uma checklist que você adquiriu depois é o exercício de calibração mais barato deste curso, e ele só funciona porque você escreveu o veredito antes de saber a resposta. "Leia o que o seu ecossistema fixa" é evidência sobre realidade de manutenção e custo de migração nos dois sentidos: uma dep de que os emblemáticos estão fugindo é um aviso, e uma dep que os emblemáticos seguram felizes num major velho é estrutural e estável ali.

Agora a correção que mantém a checklist honesta, e é uma que a pesquisa deste próprio curso teve que fazer no meio do caminho: contagens cruas de dependências não estão na lista, de propósito. É tentador argumentar "este crate puxa 30 pacotes, é pesado e arriscado." Contar é como a nossa pesquisa julgou mal os crates de cliente de Solana no começo; o cliente de RPC "enxuto" acabou carregando mais dependências diretas que o "com tudo dentro", porque as deps dele eram dezenas de crates minúsculos de tipos enquanto o grande carregava uma stack de rede completa em peças menos numerosas e mais pesadas. Um aprendiz que conta vai chegar ao veredito errado com plena confiança. Leia o que as dependências são, não quantas elas são. Contagens enganam; conteúdos informam.

![Barras pareadas mostram o agave segurando o reqwest um major atrás e o clap dois majors atrás das linhas atuais deles, enquadrados como decisões com custo.](assets/v04-chart.webp)

Rode as cinco leituras e você aterrissa em um de quatro vereditos: keep, upgrade, replace ou accept-risk. Todo veredito é escrito, e a forma é fixa, porque um veredito que mora na sua cabeça é um humor, e um veredito no repo é uma decisão que a próxima pessoa consegue auditar:

```markdown
### <dependency>
- signal read: <the one or two signals that decided it>
- exposure path: <runtime, build-time, or dev-only, and the edge it enters through>
- decision: keep | upgrade | replace | accept-risk
- action: <the concrete next step, or "none, revisit <date>">
```

Esse veredito escrito é o entregável de verdade desta lição, e é a peça que os times de fato não têm. Todo mundo roda auditorias. Quase ninguém escreve o que decidiu sobre os resultados, então todo alerta é re-litigado do zero por quem o vir da próxima vez.

![Cinco sinais de checklist afunilam em um de quatro vereditos, cada um registrado na mesma entrada de quatro linhas do AUDIT.md, enquanto contagens cruas de dependências ficam excluídas fora do funil.](assets/v05-diagram.webp)

### Pins, faixas e o lockfile que supera os dois

Última peça de teoria, e é aquela sobre a qual os 86 minutos secretamente eram. Você tem três instrumentos de pin, e cada um deles é só escolher de que jeito você prefere estar errado.

Faixas de semver, o `^1.4.0` no package.json e o `"0.3.9"` pelado no Cargo.toml, se autocuram: uma versão corrigida é entregue e a sua próxima resolução pega ela sem edição de manifest. O mesmo mecanismo autoingere: uma versão maliciosa é entregue dentro da faixa e a sua próxima resolução pega essa também. A janela de 86 minutos existe porque faixas resolvem para a frente; isso não é um defeito no semver, é o acordo inteiro que você assinou.

Pins exatos, `=0.3.9` no Cargo, `1.4.2` pelado no npm, congelam o sabidamente bom. Eles também congelam o sabidamente ruim: no dia em que uma vulnerabilidade de verdade é corrigida, o seu pin exato te segura na versão vulnerável, em silêncio, até um humano editar um arquivo. Durante a janela do arrayref um pin exato era armadura. Durante os meses depois de algum advisory futuro, o mesmo pin é a exposição.

E o lockfile é o pin de verdade, com uma condição anexada. A faixa do seu manifest expressa intenção; o lockfile registra uma resolução de verdade, byte a byte; e um install que honra o lockfile reproduz essa resolução exatamente, seja qual for a que a faixa preferiria hoje. A condição: o lockfile só protege installs que de fato leem ele.

```bash
npm ci
pnpm install --frozen-lockfile
cargo build --locked
```

Essas são as grafias que honram, e duas delas já estão na sua estação: o workflow do Actions do M1 rodou `npm ci` desde o dia em que você escreveu ele até m03-l1 refazer a fiação para `pnpm install --frozen-lockfile` quando o workspace virou pnpm, e o Dockerfile do M6 instala com `--frozen-lockfile` também. Uma assimetria para anotar antes de você auditar qualquer job de CI contra esta lista: o cargo honra um `Cargo.lock` commitado por padrão, então um `cargo build` ou `cargo test` simples num clone já instala as versões registradas, e `--locked` acrescenta rigidez (falhar alto em vez de atualizar em silêncio quando manifest e lockfile discordam) em vez de ligar a leitura do lockfile. O npm é o oposto; um `npm install` pelado vai reescrever o lockfile com todo prazer, que é por isso que o `npm ci` existe. Então um `cargo test` sem flag num workflow não é exposição a resolução para a frente do jeito que um `npm install` pelado é; classifique ele de acordo quando você escrever a linha de CI no passo 5. Um `npm install` pelado num clone fresco sem lockfile presente resolve para a frente e teria comido o arrayref. Toque o exemplo trabalhado uma vez do começo ao fim, com números de npm desta vez: o seu manifest diz `^1.4.0`, o seu lockfile registrou 1.4.2, e um 1.4.3 malicioso publicado hoje de manhã. O build de CI de hoje à noite sob `npm ci` instala 1.4.2 e está tudo bem. O momento perigoso não é a publicação. É o próximo `pnpm update`, a próxima regeneração de lockfile, o próximo "deixa eu só atualizar as deps já que estou aqui," feito enquanto a versão maliciosa está no ar. A janela abre do seu lado do registro.

![Um diagrama em camadas mostra installs que honram o lockfile reproduzindo a versão registrada enquanto comandos de update contornam o lockfile e resolvem para a frente, entrando em risco.](assets/v06-diagram.webp)

Então qual instrumento você usa? Por classe de dependência, e de propósito. Faixas mais um lockfile commitado mais installs que honram é o padrão sensato: você ganha autocura nos momentos que escolher, e reprodução em todo o resto. Pins exatos ganham o lugar deles em dependências onde qualquer surpresa é inaceitável e você se compromete a acompanhar advisories na mão, o mesmo trade-off que o agave fez com o clap. E um lockfile sem `npm ci` na CI é decoração; confira os seus workflows, não as suas intenções. Não existe configuração em que você não esteja errado em algum lugar. Um atacante publicando dentro da sua faixa ganha da faixa; um patch que você nunca adota ganha do pin exato; um lockfile regenerado ganha do lockfile. Você está escolhendo qual modo de falha você prefere por classe de dependência e escrevendo a escolha, e essa escolha escrita é precisamente o que um veredito é. Segurança por papelada parece desanimador até a papelada ser a única coisa na sala que lembra por que o pin está ali.

**Vá mais fundo (os 20%).** esta lição te ensinou a rodar as ferramentas e ler a saída delas; as ferramentas vão mais fundo que uma lição. A documentação do cargo-audit em docs.rs/cargo-audit cobre o subcomando fix, integração com CI e self-checks; o cargo-deny em embarkstudios.github.io/cargo-deny estende a auditoria para política de licença e de origem, para times que precisam de bans e allowlists; e a referência do npm audit em docs.npmjs.com (CLI commands, npm-audit) documenta verificação de assinatura e o comportamento exato do endpoint de auditoria. As três URLs verificadas no ar em 2026-09-02. Nada no lab abaixo depende do material do bookmark.

Uma nota honesta de escopo, sem hand-off anexado: tudo acima é higiene de cadeia de suprimentos do ciclo de vida de desenvolvimento, protegendo o código que você constrói e entrega. Segurança de smart contract, a auditoria da lógica de programa on-chain, é um campo completamente diferente e este curso não ensina isso nem finge ensinar.

## Lab

A camada de auditoria passa por toda a propriedade: o workspace TS (frota, core, painel), o workspace Rust (motor, CLI e o daemon pollerd), os dois projetos de edge que pegam carona fora dos dois, `pulse-edge-ts` (projeto npm próprio desde m07-l1, com a instalação própria de `@solana/kit` sobre a qual o lockfile raiz não sabe nada) e `pulse-edge-rs`, e `tx-check`, o scaffold de caminho de escrita do m08-l4, também um projeto npm próprio fora do workspace e a árvore cujas dependências ficam mais perto de uma chave de assinatura. Cinco lockfiles, cinco passadas. Cerca de 45 minutos. A parte trabalhada é o passo 3; do passo 4 em diante, a checklist é sua e eu saí de cena.

1. **Rode a auditoria TS na raiz do repo da estação.** A raiz é onde o `pnpm-lock.yaml` mora, e a auditoria lê o lockfile, então o lugar importa:

   ```bash
   pnpm audit
   ```

   Leia a linha de resumo antes de qualquer outra coisa: quantos advisories, em que severidades, em quantos pacotes. Num repo com lock de npm o mesmo comando é `npm audit`; o nosso workspace virou pnpm em m03-l1, e a auditoria segue o lockfile.

   Depois faça de novo em `pulse-edge-ts`. Aquele worker é um projeto npm próprio, criado fora do workspace em m07-l1, então o lockfile raiz que você acabou de auditar não diz nada sobre ele, incluindo a instalação própria de `@solana/kit` dele. (O kit mora no workspace também, desde o M8: a instalação raiz de m08-l1 e a do pulse-board de m08-l2; `pnpm why @solana/kit` desenha essa metade do mapa em segundos. Mesmo pacote, e no fim deste passo, as jurisdições de três lockfiles, que é exatamente a leitura de quem-cobre-o-quê que esta passada existe para ensinar.) Dê `cd` até lá e rode `npm audit`. Depois a passada que mais importa por byte: dê `cd` para dentro de `tx-check`, o projeto standalone que o m08-l4 montou com `npm init -y`, e rode `npm audit` no lockfile dele também. Aquela árvore guarda o pipeline do kit que toca o seu keypair de assinatura, e nem o lockfile raiz nem o do pulse-edge-ts dizem uma palavra sobre ele; pule ele e a varredura auditou tudo menos o código que está de pé ao lado de uma chave privada. Três comandos, três lockfiles, e a metade TypeScript da propriedade está coberta. Dois lockfiles seguem intocados, os dois do lado Rust, os dois no próximo passo; "eu auditei a estação" não é verdade até eles estarem feitos, e o ponto inteiro de contar lockfiles é perceber isso antes de você dizer.

2. **Rode a auditoria Rust na raiz do workspace pulse-rs.** Instale primeiro se você pulou a linha de instalação da teoria:

   ```bash
   cargo install cargo-audit --locked
   cd pulse-rs
   cargo audit
   ```

   Ele escaneia o `Cargo.lock` contra o banco do RustSec, então a raiz do workspace, onde o lockfile mora, é o único lugar correto para ficar de pé. Um diretório de crate membro sem lockfile próprio não te dá nada. Depois a quinta passada, e ela não é opcional exatamente pela razão pela qual `tx-check` não era: `pulse-edge-rs` fica fora do workspace com o `Cargo.lock` próprio dele desde m07-l2, então nada do que você rodou até agora diz uma palavra sobre ele. Dê `cd` até lá e rode `cargo audit` de novo. Cinco lockfiles, cinco passadas, e agora a frase do passo 1 é verdadeira.

3. **Leia um achado de ponta a ponta, comigo.** Se qualquer uma das duas ferramentas sinalizou algo, esse é o seu espécime. Leia o advisory id e diga ele em voz alta. Leia a faixa vulnerável e confira contra ela a versão que o seu lockfile fixou. Leia a versão corrigida. Depois rastreie o caminho: `pnpm why <package>` ou `cargo tree -i <crate>`, e classifique a exposição: runtime, build-time ou dev-only. Agora escreva o veredito na forma de quatro linhas da teoria. Se as duas ferramentas voltaram limpas, o que é provável, já que esta stack é nova, a rep trabalhada roda na checklist em vez disso: escolha uma dependência de qualquer um dos lockfiles, rode as cinco leituras de abandono nela pra valer (`npm view <pkg> time.modified`, a página de releases, o issue tracker, os manifests emblemáticos), e escreva o mesmo veredito de quatro linhas. De um jeito ou de outro você agora produziu um veredito com supervisão. Esse foi o último supervisionado.

4. **Escreva o `AUDIT.md` na raiz do repo da estação.** Estruture assim: as duas linhas de resumo das ferramentas com a data de hoje, depois os seus vereditos. Mínimo de dois vereditos, cada um um cabeçalho de dependência sobre os quatro campos do template: signal read, exposure path, decision, action. Achados de verdade primeiro se você tiver; se a árvore estiver limpa, escolha duas dependências deliberadamente, uma de cada lockfile, e rode a checklist contra elas. Escolha pelo menos uma sobre a qual você nunca pensou conscientemente. A trava nunca depende do humor do banco de advisories.

5. **A passada de classificação de pin.** Abra os dois manifests e os dois lockfiles. Para toda dependência direta dos pacotes TS e do workspace Rust, classifique o que o MANIFEST diz: uma faixa (caret, tilde ou comparadores), um pin exato, ou um especificador tão frouxo (pelado, curinga ou sem restrição) que o lockfile está fazendo todo o trabalho de pin; chame esse terceiro rótulo de lockfile-only. Sim, com um lockfile commitado toda dep com faixa TAMBÉM está fixada na prática; a classificação é sobre a intenção declarada do manifest, e o terceiro rótulo é reservado para deps cujo manifest não declara nenhuma. Uma tabela no `AUDIT.md`, uma linha por dep direta. Depois responda a pergunta que decide se qualquer coisa disso importa, em uma linha escrita: a CI instala a partir do lockfile? Confira o workflow do M1 atrás de `pnpm install --frozen-lockfile` (a refiação do m03-l1; o trabalho do `npm ci` na grafia do pnpm), o Dockerfile do M6 atrás de `--frozen-lockfile`, e anote o que você achar. Um lockfile do qual ninguém instala não impõe nada.

![Um card de anatomia mostra as quatro seções obrigatórias do AUDIT.md, alimentadas pelos dois comandos de auditoria e consumidas rio abaixo pelo runbook de m10-l1.](assets/v07-diagram.webp)

6. **Commite.**

   ```bash
   git add AUDIT.md
   git commit -m "audit layer: tool reports, verdicts, pin classification"
   git push
   ```

   Este arquivo não é de uso único. O runbook de m10-l1 consome ele diretamente.

7. **Extensão opcional, claramente opcional: um relatório de CI que não trava.** Adicione um step de auditoria ao workflow do M1 que reporta mas nunca falha o build:

   ```yaml
   - name: dependency audit (report only)
     run: pnpm audit || true
   ```

   (`npm audit || true` num repo com lock de npm.) Não travar é deliberado por enquanto: você ainda não decidiu, como política, quais achados deveriam bloquear um merge, e uma trava sobre a qual você não raciocinou é uma trava que você vai furar na primeira vez que ela te irritar. A lição do runbook revisita a pergunta com os seus vereditos na mão. Esta edição aterrissa depois do commit do passo 6, então dê um commit próprio a ela: `git add .github && git commit -m "ci: report-only dependency audit"` e dê push.

Barra de aceitação, sem rodeios: os dois comandos de auditoria rodaram nas raízes corretas com saída que você consegue mostrar; o `AUDIT.md` existe e está commitado, com pelo menos dois vereditos na forma de quatro partes e a tabela de classificação de pin cobrindo toda dependência direta dos dois manifests; e a resposta de uma linha sobre CI está escrita.

## Challenge

Abra o `pnpm-lock.yaml` e escolha uma dependência transitiva de que você nunca ouviu falar. Não uma que você escolheu; uma que chegou como dependência de uma dependência, do tipo de nome que te faz dizer "o que é isso e por que eu entrego isso." Rode a checklist completa de cinco sinais contra ela, a frio: README, última publicação, cadência, deriva de issues, quem mais fixa ela. Escreva o veredito de quatro linhas dela e adicione ao `AUDIT.md`. Sem apoio e sem espiar a minha leitura trabalhada. O ponto do exercício é que o método funciona com estranhos totais, porque a sua árvore é quase toda de estranhos totais, e depois de hoje isso para de ser um fato desconfortável que você evita e começa a ser uma lista que você percorre.

Depois uma segunda rep, em código, porque a checagem de faixa que você fez a olho no passo 3 é exatamente uma função: o challenge semver-vuln-matcher, no coding-challenge panel como toda rep avaliada neste curso. Um advisory nomeia uma faixa vulnerável como `>=0.3.0 <0.3.11`, e a pergunta de quem audita é se a sua versão instalada fica dentro dela. Uma nota de honestidade sobre essa faixa antes de você confiar nela como história: ela está escrita em gramática de advisory e pega emprestado os números de versão do arrayref, mas é uma faixa de exercício, não o advisory do incidente de verdade. O incidente de fato teve exatamente um release hostil, 0.3.10, que recebeu yank em vez de correção (nenhum 0.3.11 jamais foi entregue como conserto), e a turma do 0.3.9 segurado por lockfile nunca esteve em perigo. A faixa de exercício existe porque ela põe um patch de um dígito e um patch de dois dígitos em lados opostos de uma fronteira, que é precisamente onde o bug plantado está: o `isVulnerable` do starter e o parse de faixa dele estão prontos, e o bug mora em `compareVersions`, que compara strings de versão como strings, e lexicograficamente `'0.3.9'` ordena acima de `'0.3.10'`, porque o caractere `'9'` ganha do `'1'`. Conserte para comparar numericamente, componente por componente, major, depois minor, depois patch. Sete testes avaliam isso, abrindo com os dígitos do incidente: o `0.3.10` malicioso tem que aterrissar dentro da faixa de exercício e o `0.3.11` fora dela. Os hints do starter escalam de onde a comparação de strings mente até a borda do componente faltante; gaste eles em ordem.

![Quatro strings de versão ordenadas de dois jeitos, mostrando que a comparação de strings põe 0.3.9 acima de 0.3.10 enquanto a comparação numérica corretamente põe ela abaixo.](assets/v08-table.webp)

## Checkpoint

O que você consegue fazer agora, concretamente: rodar `npm audit` e `cargo audit` nas raízes onde os lockfiles deles moram e dizer por que a raiz importa; ler um achado na ordem certa, id, faixa, patch, caminho, e só então severidade, e classificar exposição como runtime, build-time ou dev-only antes de reagir; rodar a checklist de abandono de cinco sinais em qualquer pacote, incluindo um que você nunca viu; explicar o que uma auditoria limpa te diz e o que ela não te diz; e defender uma filosofia de pin por classe de dependência, incluindo exatamente quais comandos de install fazem o lockfile ser de verdade.

A recuperação de 30 segundos antes de você fechar a aba: durante os 86 minutos em que o arrayref 0.3.10 ficou no ar, quem estava exposto e quem não estava, e qual artefato único fez a diferença? (Resoluções frescas dentro da faixa compatível estavam expostas; todo install que honrava um lockfile commitado não estava. O lockfile foi a diferença, e só porque alguma coisa instalou a partir dele.) Se você teve que olhar para cima de novo, releia o diagrama de install em camadas; essa única figura é esta lição.

Um pedido enquanto está fresco: se as duas auditorias suas voltaram limpas, me diga no feedback se os vereditos conduzidos por checklist no passo 4 pareceram trabalho de verdade ou encheção de linguiça. O passo existe para que a trava nunca dependa de o banco de advisories ter tido uma semana ruim, mas se isso leu como enchimento para a maioria de vocês, a próxima revisão escolhe as duas deps por você e faz elas mais desagradáveis.

A sua árvore de dependências agora tem um rastro de auditoria escrito: ferramentas rodadas nas raízes certas, vereditos que um estranho conseguiria seguir, pins classificados e aplicados de propósito. Mas a própria estação ainda está rodando na confiança. Se o poller morrer em silêncio hoje à noite, se o cron parar de disparar, nada em lugar nenhum faz barulho. Próxima lição a estação aprende a observar a si mesma: logs estruturados que respondem perguntas em vez de narrar, a realidade de logs de cada plataforma lida com honestidade, e um alarme ligado à única falha que o painel de ninguém mostra, a morte do próprio monitor.
