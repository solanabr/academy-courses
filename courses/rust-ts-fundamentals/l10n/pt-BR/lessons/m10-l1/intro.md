# Montagem: a estação, inteira

## Resumo

m09-l2 estendeu a camada de ops sobre cada superfície: o poller emite logs JSON estruturados dos quais você tirou um incidente de verdade com um grep, a realidade de logs de cada plataforma está mapeada e escrita, o alarme de notificação de falha do Actions está ligado, e a tabela da varredura de segredos de quatro plataformas voltou limpa. O que quer dizer que não sobrou nada para construir antes de a coisa que você vem construindo há dez módulos ser montada e provada. Hoje você verifica a Pulse Station completa de ponta a ponta, aresta por aresta, contra um script de demo que você mesmo escreve conforme avança; você constrói a única e exclusiva peça de fiação nova do capstone; você escreve o README a partir do qual outro dev conseguiria operar a estação; e então você entrega uma extensão sem apoio nenhum. O recuo, dito em voz alta porque ele se completa aqui: a montagem é guiada por checklist, o painel novo é semiguiado (eu nomeio a composição, você escreve o código), e a extensão é totalmente solo. Nenhum exemplo trabalhado em lugar nenhum desta lição, exceto uma exceção deliberada, o check de freshness do passo 3, trabalhado por inteiro porque é o ponto exato onde nascem os scripts de demo de falso verde. Você não precisou de mais que isso por dois módulos, e fingir o contrário agora seria um insulto.

## Desenhe antes de construir

Antes de fiar qualquer coisa, baixe a tampa do laptop em cima dos docs e desenhe a sua estação de memória. Papel, quadro branco, o verso de um recibo, qualquer coisa. Todo componente, a linguagem dele, a plataforma dele, e toda aresta de fluxo de dados, na forma de hub. Você tem trinta segundos e aqui está a forma da resposta de antemão, porque este checkpoint foi desenhado para ser uma vitória:

```text
      [ spoke ]        [ spoke ]
            \            /
[ spoke ] -- ( one pipeline ) -- [ spoke ]
                  |
              [ spoke ]

arrows only where data actually moves
```

Um pipeline no meio como o batimento, raios independentes ao redor, setas só onde os dados de fato se movem. Se você consegue desenhar isso em trinta segundos, você já entende um sistema distribuído poliglota bem o bastante para montar ele. Se uma seta parece embaçada, esse embaçamento é exatamente o que as próximas três horas queimam.

Vá. Desenhe. Depois volte e se confira contra a referência.

![Um diagrama de hub e raios com um pipeline central, cinco componentes ao redor, fluxos de dados reais desenhados em linha sólida e três setas proibidas riscadas.](assets/v01-diagram.webp)

Se pontue com honestidade. Os componentes são a metade fácil; a maioria acerta todos os seis. As arestas são onde o desenho ganha os seus trinta segundos, e as setas erradas importam mais que as certas. O erro clássico, e eu mesmo desenhei ele na primeira vez que esbocei este diagrama para a ementa do curso, é uma seta do poller para o painel. Parece que ela deveria existir. O poller tem os dados mais ricos da estação, leituras de blockchain e tudo, e o painel é a cara. Mas nenhuma aresta dessas existe, e nenhuma aresta entrando no poller existe também: ele roda na sua máquina, cutucado da sua máquina, sem superfície pública nenhuma. As outras duas ciladas: uma seta de qualquer um dos dois workers para dentro do poller (os workers são deliberadamente independentes, esse é o ponto inteiro deles), e uma seta do GHCR para um contêiner rodando em algum lugar na nuvem (um registro é armazenamento, não hosting, que é a forma curta que vale a pena cunhar aqui para o que m06-l4 ensinou; o pull acontece na sua máquina).

Aqui está a síntese que vale a pena levar deste curso: um sistema distribuído é só programas que concordam com um diagrama. Isso não é uma metáfora. Todo componente do qual você fez deploy cumpre a parte dele de exatamente um contrato, as setas deste desenho, e nada mais. Você desenhou o diagrama de memória. O resto desta lição é fazer a realidade bater com ele, uma seta de cada vez, com um comprovante para cada uma.

## A estação, aresta por aresta

Duas definições antes do percurso, as duas você vai usar pelo resto da sua carreira.

Uma **topologia de hub** é a forma que a sua estação tem: um batimento no meio, raios independentes ao redor. A alternativa que vale a pena nomear é uma cadeia, onde A alimenta B, que alimenta C, que alimenta D, e qualquer salto morrendo leva junto tudo o que está a jusante. A sua estação não tem cadeias mais longas que um salto. O painel cair não afeta nada além do painel. Um worker cair deixa intactos o gêmeo dele, o poller e o pipeline. Só o hub é estrutural para o sistema como um todo, e m09-l2 gastou uma lição garantindo que a morte do hub seja barulhenta.

Um **script de demo** é o exercício de runbook que prova o sistema: um check por aresta, cada um imprimindo OK ou FAIL, rodável de novo sob demanda. É a diferença entre "acredito que a minha estação funciona" e "aqui está a transcrição." Você vai escrever ele conforme verifica, um check por aresta, o que quer dizer que no fim do lab a prova e o sistema existem como um par. Esse pareamento é o entregável de verdade de um capstone. Qualquer um consegue montar alguma coisa uma vez; o script de demo é o que torna ela operável.

Já que você está prestes a escrever um script inteiro deles, um por aresta e mais ou menos dez no total uma vez que a aresta 5 ganha um check por worker e a aresta 8 um por ferramenta de auditoria, a pergunta de gosto vale trinta segundos: o que faz um check ser confiável? Três propriedades. Ele observa a afirmação, não o transporte: um HTTP 200 vindo de um CDN diz "um cache tem bytes," enquanto um timestamp dentro do payload diz "meu cron rodou dentro da hora," e só uma dessas duas é a coisa com que você de fato se importa. Ele é rodável de novo sem estado manual: nada de "primeiro apague o contêiner velho," nada de "funciona se você rodou o outro script faz pouco tempo," porque um check com instruções de preparação é uma tarefa doméstica, não um check. E ele falha alto com um motivo, porque `FAIL` sem explicação só empurra a depuração para um momento pior. Todo check que você escrever hoje deveria sobreviver às três perguntas, e o que eu trabalho para você no passo 3 está escolhido precisamente porque é onde a maioria escreve a versão vaidosa.

![Dois cartões contrastam um check que só prova que um cache respondeu com um check que prova que os próprios dados estão frescos.](assets/v02-comparison.webp)

Então: as oito arestas, cada uma nomeada com o módulo que construiu ela, porque este percurso funciona também como a última recuperação espaçada do curso. Leia a coluna do meio devagar e note que ela é um índice das suas últimas dez semanas.

| # | Aresta | Construída por | Prova |
|---|---|---|---|
| 1 | Pipeline do Actions em verde, seis jobs | m01-l3, travas de m02-l4 + m04-l3, jobs de m05-l3 + m06-l4 | a última execução completada teve sucesso |
| 2 | O cron publica status.json | m01-l3, tipado por m02 | timestamp do payload com menos de 60 min |
| 3 | O painel renderiza os painéis de frota + Solana | m03-l2, m03-l3, m08-l2 | os dois painéis no ar na URL do Vercel |
| 4 | O painel faz polling no worker TS (NOVO) | hoje, a partir de m03-l2 + m07-l1 | terceiro painel no ar |
| 5 | Dois workers, independentes | m07-l1, m07-l2, leituras de blockchain m08-l2 | as duas URLs workers.dev respondem com JSON por alvo |
| 6 | O poller do GHCR roda localmente | m06-l2 até m06-l4, leituras de blockchain m08-l3 | localhost:8080/status responde com dados de blockchain |
| 7 | Caminho de escrita saudável | m08-l4 | tx-check sai com 0 e uma assinatura confirmada |
| 8 | Auditoria + alarme em verde | m09-l1, m09-l2 | auditorias limpas ou com veredito, notificações confirmadas ligadas |

A aresta 1 merece um parágrafo porque ela é o hub, e porque verificar ela é um exercício de leitura, não de construção. O seu único workflow cresceu por dez módulos: a trava do vitest chegou em m02-l4, as travas de cargo test, clippy e fmt em m04-l3, o job do binário de release em m05-l3, os pushes de imagem para o GHCR em m06-l4. Testes nas duas linguagens travam o cron. O cron sonda e commita `status.json`. Nada aqui é construído hoje; hoje você lê a última execução como um operador e confere se todo job da chain ficou verde. E enquanto você está lá dentro, note a coisa que parece um bug e não é: o commit de `status.json` do próprio cron nunca redispara o workflow. Essa é a trava de recursão do GitHub funcionando. Eventos criados com o `GITHUB_TOKEN` do workflow não geram novas execuções de workflow, precisamente para que um workflow que commita não consiga se disparar sozinho para sempre por acidente. Existe uma saída de emergência documentada (use um PAT ou um token de GitHub App quando você genuinamente quiser execuções a jusante), e a sua estação não quer nada disso. Uma feature, não um conserto.

As arestas 3 e 4 são a história do painel, e vale a pena ver que o painel agora é um resumo de uma página do curso inteiro:

![Três pistas levam dados de frota, leituras de blockchain ao vivo e status de workers para um único painel, cada pista anotada com o próprio delay de freshness dela.](assets/v03-flowchart.webp)

A aresta nova, a número 4, é a única construção deste capstone, e a razão de ela existir é dita em voz alta: ela não exige nenhuma habilidade nova. É o padrão de polling de m03-l2 apontado para o endpoint de m07-l1. O seu painel faz polling numa URL JSON pública num intervalo desde o módulo 3; o seu worker TS serve o snapshot de KV dele como JSON público desde o módulo 7. Aponte o primeiro para o segundo e o painel ganha um painel de status de workers. Nenhuma API nova, nenhuma plataforma nova, nenhum pacote novo. Essa é a tese do curso em um painel só: em algum ponto a capacidade nova para de vir de ferramentas novas e passa a vir de compor as que você já tem.

Agora a seção de honestidade, porque um capstone que esconde as costuras dele é uma demo, e nomear elas é o que faz disto um sistema. Três costuras, todas deliberadas.

O poller é só local. O tier gratuito te comprou três superfícies públicas (Vercel, duas URLs workers.dev), não quatro. Expor o poller significaria tunneling ou hosting pago, os dois fora de escopo de propósito; o contêiner é cutucado da sua própria máquina e esse é o design, não um atalho. Segundo, o caminho de frota do painel tolera desatualização por design: um cron de 30 minutos mais um cache de CDN de 5 minutos quer dizer que o painel de frota pode ficar atrás da realidade por mais de meia hora, coisa que você sabe desde que m03-l2 te ensinou a ler `cache-control: max-age=300` no devtools. Terceiro, e maior: o hub inteiro confia em um pipeline só. Se o Actions cair, ou se a auto-desabilitação por 60 dias de inatividade disparar no seu repo público, o batimento para. Toda mitigação que você tem para isso veio do m09: a notificação de falha é o seu alarme de último recurso, e a política de desabilitação é a razão de o runbook que você escreve hoje ter um exercício de reativação dentro dele. Um pipeline só é um ponto único de falha de verdade e a estação carrega ele de olhos abertos, porque a alternativa num tier gratuito é um segundo escalonador que você também teria que monitorar.

Uma coisa que esta lição deliberadamente não tem: um box de Vá mais fundo. Não existe capítulo canônico de livro para "monte o sistema que você já construiu." A linha de leitura adicional do runbook simplesmente aponta de volta para o mapa de ensinado-versus-bookmark de m01-l1, e a próxima lição, a conclusão, reimprime esse mapa com olhos novos.

## Lab: montar, verificar, documentar

A estrutura, para você se dosar: o passo 1 constrói o harness, os passos 2 até 9 percorrem as oito arestas, guiados por checklist, e você escreve um check de script de demo por aresta conforme verifica ela. O passo 10 é a execução dupla. O passo 11 é o README. Reserve o grosso do seu tempo para a aresta 4 (o passo 5, o painel novo) e o README (o passo 11); todo o resto é verificação de coisas que já funcionam.

1. **O harness.** Crie `scripts/demo.sh` no repo da estação. Eu estou te dando o harness e um check trabalhado; todo outro check é seu para escrever, e essa é a tarefa, não uma lacuna. O contrato: cada check imprime uma linha `OK` ou `FAIL`, e o script sai com código diferente de zero se alguma coisa falhou.

```bash
#!/usr/bin/env bash
set -u
PASS=0; FAIL=0

# --- edit these four lines to your station ---
REPO="YOUR_USER/pulse-station"
DASHBOARD_URL="https://your-board.vercel.app"
WORKER_TS_URL="https://pulse-edge-ts.your-subdomain.workers.dev"
WORKER_RS_URL="https://pulse-edge-rs.your-subdomain.workers.dev"

check () {
  local name="$1"; shift
  if "$@" >/dev/null; then
    echo "OK   $name"; PASS=$((PASS+1))
  else
    echo "FAIL $name"; FAIL=$((FAIL+1))
  fi
}

# checks get authored here, one per edge, as you verify

echo
echo "$PASS OK, $FAIL FAIL"
[ "$FAIL" -eq 0 ]
```

Uma escolha de redirecionamento em `check` é estrutural: o stdout de cada comando é jogado fora, porque ferramentas tagarelas enterrariam a contagem, mas o stderr é deliberadamente deixado em paz. Essa é a propriedade três vestida de bash. Quando um check falha, o motivo dele, a linha `console.error` da aresta 2, as mensagens `-S` do curl, aparece bem acima do veredito FAIL em vez de sumir no `/dev/null`; adicione `2>&1` a esse redirecionamento e todo check que você escrever hoje fica mudo exatamente no momento em que ele te deve uma explicação.

![Uma pequena função auxiliar de bash anotada para mostrar que cada check é um rótulo mais qualquer comando cujo código de saída decide o veredito impresso.](assets/v04-annotated-code.webp)

2. **Aresta 1: o batimento.** Abra a aba Actions e leia as últimas execuções, no plural, porque nenhuma execução sozinha jamais mostra os seis jobs juntos: as condições `if:` são mutuamente exclusivas por design. As travas mais a sonda pegam carona nos pushes para a main e no agendamento; `release` dispara só em v-tags, onde a sonda é pulada; `images` pega carona nos pushes. Então uma execução agendada mostrando release e images como pulados é uma execução saudável, não uma quebrada, e o censo que você está levantando é "cada job em verde no trigger a que ele pertence," lido ao longo da história recente. Depois escreva o script disso. O seu repo é público, então a REST API do GitHub responde a um `curl` puro sem autenticação em `https://api.github.com/repos/$REPO/actions/runs?per_page=1&status=completed`; o campo `conclusion` da última execução completada deveria dizer `success`, e `node -e` com um `fetch` é a sua ferramenta de JSON-sobre-HTTP desde o módulo 1. Escreva o check. Enquanto a aba Actions está aberta, encontre o commit do próprio cron no histórico de execuções e confirme o que não está lá: nenhuma execução disparada por ele. Você está olhando a trava de recursão do `GITHUB_TOKEN` se comportando, e o seu check acabou de codificar o estado saudável.

3. **Aresta 2: um status.json fresco.** Este aqui eu trabalho por inteiro, porque o footgun dele é o que produz scripts de demo de falso verde: checar HTTP 200 num arquivo cacheado por CDN prova que o CDN tem bytes, não que o seu cron está vivo. O check tem que ler o timestamp do próprio payload. O limiar de 60 minutos é a regra de incidente de m03-l2: uma execução de cron perdida é um soluço, duas são um incidente.

```bash
STATUS_URL="https://raw.githubusercontent.com/$REPO/main/status.json"

status_fresh () {
  node -e '
    fetch(process.argv[1]).then(r => r.json()).then(j => {
      const age = (Date.now() - Date.parse(j.generatedAt)) / 60000;
      if (!(age < 60)) throw new Error("stale: " + age.toFixed(1) + " min old");
    }).catch(e => { console.error(e.message); process.exit(1); });
  ' "$STATUS_URL"
}
check "edge 2: status.json younger than 60 min" status_fresh
```

Rode o script agora. Duas linhas OK e uma contagem limpa, e o padrão do harness está provado. Tudo daqui em diante é você.

4. **Aresta 3: os dois primeiros painéis do painel.** Abra a sua URL do Vercel num navegador. O painel de frota renderiza linhas reais coloridas pelo classificador do pulse-core; o painel de Solana mostra leituras do kit ao vivo. Olhe o painel de Solana por um segundo a mais do que você precisa, porque ele é caladamente o melhor medidor da estação: a blockchain que a sua estação observa mudou o batimento dela no meio do curso. A stage 2 do SIMD-0525 levou a mainnet para slots de 300ms na epoch 1024 em 2026-08-28, um quarto a menos no intervalo e portanto um terço a mais de slots por segundo, a distinção exata que m08-l1 treinou, dias antes de a pesquisa deste curso congelar, e a sonda de 2026-09-01 mediu 316ms contra essa meta de 300ms. O seu painel está observando um batimento que mudou enquanto você aprendia a medir ele, e ele mostra o que é, não o que a spec promete. Essa é a disciplina deste curso inteiro em um par de números. Check de script: `curl -fsS` na URL do painel, e nomeie ele com honestidade, algo como `edge 3: dashboard deploy answers (transport only)`, porque pela taxonomia da própria lição este é do tipo vaidoso: ele prova que o deploy serve bytes, não que os painéis renderizam. Painéis são um fato do navegador e o screenshot de fechamento é a evidência deles, então o script carrega este único check de transporte conscientemente rotulado como a exceção aceita em vez de uma contradição calada da seção de gosto.

![Duas barras horizontais comparam uma meta de tempo de slot de 300 milissegundos com uma média medida de 316 milissegundos, uma diferença de cerca de cinco por cento.](assets/v05-chart.webp)

5. **Aresta 4: o painel de status de workers.** A única construção do capstone. Semiguiada, como prometido: aqui está a composição, e o código é seu.

   - **O padrão:** m03-l2, por inteiro. Um schema zod na fronteira, uma união discriminada `BoardState`, `useState` mais `useEffect`, um poll com `setInterval` e a função de limpeza dele, parse-don't-validate na chegada.
   - **O alvo:** o endpoint JSON público do seu worker TS, a rota raiz de `pulse-edge-ts.<your-subdomain>.workers.dev`, servindo uma entrada por alvo com o veredito de `solana-rpc` incluído, exatamente como m07-l1 entregou.
   - **A parede da lição um, agora do lado do servidor:** este painel é a primeira leitura do curso deliberadamente cross-origin de uma superfície que é sua, então recolha o tratamento que m07-l1 adiou para o capstone. A origem do seu painel é a URL vercel.app dele; o worker responde de workers.dev; e a regra de m01-l1 governa sem mudanças: uma página sempre pode ler a própria origem, e uma resposta cross-origin só é legível quando o servidor opta por permitir. O opt-in é o header `access-control-allow-origin: *` que m07-l1 congelou dentro da resposta JSON do worker, a configuração honesta para um snapshot público somente de leitura, e para este painel ele é a superfície de CORS inteira: o poll é um GET puro sem headers customizados, que a spec classifica como simple request, então nenhum `OPTIONS` de pré-voo jamais dispara e o único header faz todo o trabalho. Prove o opt-in de fora antes de escrever uma linha de React: `curl -s -D - -o /dev/null "$WORKER_TS_URL" | grep -i access-control-allow-origin` imprime o header ou você para aqui. E seja preciso sobre quem é dono da parede: apague esse header do worker e a mesma URL continua respondendo ao curl enquanto o painel morre com um erro de CORS no console do navegador, porque a parede é do navegador, nunca da rede. O painel de frota nunca precisou desse opt-in da sua parte só porque raw.githubusercontent.com manda o mesmo `*` incondicionalmente, o header que você leu no devtools em m03-l2; desta vez o servidor dizendo sim é seu.
   - **As edições:** um segundo arquivo de schema para a forma do payload do worker, um terceiro efeito de fetch-e-poll, e um componente de painel que reusa as cores do classificador do painel. `VERDICT_COLOR` em `StatusRow.tsx` é uma const local ao módulo hoje, então ponha `export` na frente dela primeiro, e depois importe ela.
   - **O orçamento:** faça polling no mesmo intervalo de 60 segundos que o painel já usa para status.json. O snapshot de KV do worker só muda no batimento do cron do próprio worker, então fazer polling mais quente não te compra nada, e desta vez o orçamento que você queimaria não é o de um CDN, é o seu próprio tier gratuito de Workers, as 100k requisições por dia que você dimensionou em m07-l1 (o medidor de REQUESTS; a cifra de 200,000 por dia de m09-l2 é o medidor separado de eventos do Workers Logs, e os dois nunca compartilham orçamento). Um poll de 60 segundos de uma aba ou três vive confortavelmente dentro desse orçamento para sempre. Um loop quente não.
   - **Aceitação:** terceiro painel no ar no painel com deploy, alvos do worker visíveis com os status deles, e um check de script de demo que busca a URL do worker e falha se o payload não incluir o alvo `solana-rpc`. Presença, não veredito, de propósito: se o egress do seu worker estiver na blocklist do RPC público, aquela linha mostra um `down` honesto com um 403, a troca de endpoint de fallback de m07-l1 é o conserto, e um painel reportando uma recusa de verdade é um monitor funcionando, não um check para amaciar.

6. **Aresta 5: dois workers, independentemente.** Dê `curl` nas duas URLs workers.dev, e leia cada uma pelo que ela de fato é, porque os dois carregam trabalhos deliberadamente diferentes; m07-l2 disse isso quando se recusou a dar um cron ao worker Rust, sob o argumento de que o mesmo trabalho duas vezes ensina copiar-e-colar, não arquitetura. O worker TS responde com o snapshot da estação: entradas por alvo incluindo o veredito de getHealth de `solana-rpc`, escritas pelo cron dele mesmo, a partir do KV dele mesmo, cada entrada carimbada com `checkedAt`. O worker Rust responde o contrato de classificação: `GET /` passa de novo pelo motor as últimas amostras de fixture guardadas em KV como `[{"name","latency_ms","verdict"}]`, e o estado dele muda só quando alguma coisa faz POST de amostras frescas nele. O ponto desta aresta é o que ela não contém: nenhum dos dois workers consome o poller, o painel, nem o outro. E a independência é checável, não só afirmável, porque os dois se movem em relógios completamente diferentes: os valores de `checkedAt` do worker TS avançam no batimento do cron dele sem ninguém tocar nele, enquanto o payload do worker Rust fica perfeitamente parado até você alimentar ele (experimente: faça POST do `fixture.json` da estação com uma latência mudada, veja o GET dele virar os vereditos enquanto os timestamps do worker TS te ignoram por completo). Duas superfícies fazendo proxy de uma fonte de dados só não conseguiriam se comportar assim. Se você quiser o exercício completo, a versão do runbook vai mais longe: derrube um worker (faça deploy de uma rota quebrada de propósito, ou só imagine isso durante uma semana mais calma) e confirme que as outras três superfícies públicas não piscaram. Escreva um check por worker, cada um contra o contrato dele: o check de TS na entrada `solana-rpc` do snapshot, o check de RS sobre o JSON de veredito por alvo responder (faça POST de `fixture.json` e dê grep procurando um veredito, ou afirme que o GET devolve um array). O worker Rust ganhando um check no mesmo script, com zero código compartilhado entre os dois em tempo de execução, é a recompensa de m07-l2 sentada à vista de todos.

7. **Aresta 6: o poller, a partir do registro, na sua máquina.** A jogada de m06-l4, agora como operador:

```bash
docker run --rm -p 8080:8080 ghcr.io/<your-username>/pulse-pollerd:latest
```

Depois, de outro terminal, `curl -s localhost:8080/status`. O JSON que volta inclui as sondas de blockchain que m08-l3 fiou: leituras de slot e de saldo, tipadas com serde, falhas taxonomizadas com thiserror. Diga a fronteira em voz alta mais uma vez, porque o seu README vai declarar ela: este contêiner não tem superfície pública, nada na internet consegue alcançar ele, e nem tunneling nem hosting para ele são ensinados em lugar nenhum deste curso. De propósito. Escreva o check contra o localhost.

8. **Aresta 7: o caminho de escrita.** A estação consegue observar. Ela consegue agir? m08-l4 construiu a resposta como um contrato que foi prometido ao seu script de demo pelo nome: `tx-check` imprime uma assinatura confirmada e sai com 0, ou falha alto e sai com código diferente de zero. Então o check é uma linha só: `check "edge 7: write path lands" bash -c 'cd tx-check && npx tsx tx-check.ts'`. Sim, isso quer dizer que a execução dupla do passo 10 aterrissa duas transferências de devnet de verdade com alguns minutos de diferença, e tudo bem: 0.001 SOL de dinheiro de devnet sem valor por execução é exatamente o que a chave descartável existe para gastar, e um check de caminho de escrita precioso demais para rodar duas vezes não é uma checagem de saúde. Se o faucet estiver seco hoje, você conhece o exercício, você construiu ele: o fallback do validador local com `RPC_URL` e `RPC_WS_URL` apontados para 127.0.0.1, já exercitado por todo mundo uma vez, e o runbook registra os dois modos. Uma assinatura confirmada aqui quer dizer que as suas chaves, a sua construção de mensagem, a sua assinatura e a maquinaria de inclusão da rede funcionam todas. Pontinhos verdes que só provam leituras são a coisa que a sua estação deixou para trás.

9. **Aresta 8: auditoria e alarme.** Duas auditorias, uma confirmação manual. `pnpm audit --audit-level=high` na raiz do workspace (m09-l1 rodou o `pnpm audit` puro; a flag `--audit-level=high` é o aperto do capstone, travando o código de saída nos achados de severidade alta) e `cargo audit` no workspace Rust, os dois como checks de script de demo; se o seu arquivo de vereditos de m09-l1 aceita um advisory específico, codifique essa aceitação no check em vez de baixar a barra da auditoria para fazer ela passar, porque um check que fica verde fazendo perguntas mais fáceis é pior que nenhum check. Mecanicamente, por ferramenta: `cargo audit` aceita `--ignore RUSTSEC-XXXX-NNNN` por id aceito (ou uma lista `[advisories] ignore` num `.cargo/audit.toml` commitado, a grafia durável, e o caminho não é decoração: o cargo-audit lê a config de projeto só do `.cargo/audit.toml`, e um `audit.toml` na raiz pura do projeto é ignorado em silêncio, TOML válido e tudo); o paralelo do pnpm para essa grafia durável é `pnpm.auditConfig.ignoreCves` (e `ignoreGhsas`) no package.json: liste ali os ids de advisory aceitos, commite isso ao lado dos seus vereditos, e o código de saída do `pnpm audit --audit-level=high` puro volta a ser confiável. O alarme não dá para ser scriptado de fora, então ele vira a única linha manual do runbook: Notification settings do GitHub, o canal Actions, entrega ligada, only-failed-workflows marcado, exatamente onde m09-l2 deixou. Confirme que ele continua ligado e registre a confirmação no README.

10. **Rode duas vezes.** `bash scripts/demo.sh && bash scripts/demo.sh`. A barra de aceitação está redigida de propósito: todos os checks em verde numa segunda execução consecutiva com zero consertos manuais entre as execuções. Espere que a primeira execução falhe em algum lugar; descobrir onde é o trabalho inteiro dessa execução. Os suspeitos de sempre, na ordem em que costumam aparecer: uma URL ainda carregando o meu texto de placeholder no bloco de config, o contêiner do poller não estando de fato rodando porque você deu `Ctrl-C` nele uma hora atrás, um check de freshness escrito contra um nome de campo que a sua frota grafa diferente, e o mais matreiro de todos, um check que passou só porque o seu navegador esquentou o cache do CDN trinta segundos antes. Conserte cada um no script ou na estação, nunca na sua cabeça, e rode de novo. Se a execução dois ficar verde sem você tocar nela, pare e curta isso por um segundo. Um script de demo que passa uma vez é uma anedota. Duas vezes, uma atrás da outra, é um sistema.

![Uma linha do tempo mostra uma primeira execução de demo falhando dois checks, consertos aterrissando no repo, e depois duas execuções limpas consecutivas produzindo a transcrição guardada.](assets/v06-timeline.webp)

11. **O README/runbook.** O último compasso ensinado do curso, e o que torna a estação transferível. Escreva ele para um leitor imaginário específico: um dev competente que nunca viu este repo e foi acionado de plantão por causa dele. Quatro seções. O diagrama do sistema, que é o seu desenho do checkpoint do começo desta lição, corrigido e commitado. Operações por superfície: para cada uma das cinco superfícies, o comando de rodar, o comando de deploy, e o comando de cutucar que prova que ela vive, a maior parte dos quais você consegue levantar direto do seu script de demo. Seja concreto até o ponto do tédio aqui: a linha do painel diz a URL do Vercel e `npm run build` para o teste de fumaça local; as linhas dos workers dizem `npx wrangler deploy` e o `curl` delas; a linha do poller diz a linha `docker run` completa com o mapeamento de porta, porque o leitor de plantão não lembra das suas escolhas de porta; a linha do pipeline diz onde vive a aba Actions, os seis ids de job, e qual trigger roda qual, porque nenhuma execução sozinha jamais mostra os seis e um leitor que não sabe disso vai caçar uma falha fantasma nos jobs pulados de toda execução agendada. O teste para esta seção é mecânico: alguém conseguiria operar a estação com o seu repo e este arquivo, sem você na sala? Todo lugar onde a resposta é "bom, também precisariam saber...", esse conhecimento vai no arquivo. Exercícios de incidente: o percurso de grep de logs de m09-l2, o fallback de faucet seco, e os dois exercícios de Actions que a honestidade do hub exige, o que fazer quando o e-mail de alarme chega, e como reabilitar o workflow quando a auto-desabilitação de 60 dias dispara (o botão Enable workflow da aba Actions, mais um commit de keepalive como a contramedida que o ecossistema busca na prática). E a tabela de pins. Mais a absorção que m09-l2 prometeu em voz alta: dobre o `SECRETS.md` para dentro do README como a seção de segredos dele, a tabela de quatro plataformas e a lista de incident queries que o seu challenge começou, ou mantenha ele como um arquivo de nível superior que o README linka na primeira tela; de um jeito ou de outro o leitor de plantão encontra onde mora cada segredo, e as primeiras incident queries, a partir de um ponto de entrada só.

![Uma tabela de quatro colunas listando cada ferramenta fixada, onde mora o pin, o valor atual dele, e o trigger concreto para reconferir ele.](assets/v07-table.webp)

Copie a forma, não os meus valores: o ponto inteiro, martelado desde que m05-l2 te ensinou a ler os pins do agave, é que a coluna dos dígitos é a coisa menos durável da tabela e a coluna de reconferir é a mais. O cabeçalho da tabela carrega a data dele. Uma tabela de pins sem data é um boato. E a linha de leitura adicional no pé do README aponta para exatamente uma coisa: o mapa de ensinado-versus-bookmark de m01-l1, que a próxima lição reabre.

Depois commite o capstone, porque nada neste lab se commitou sozinho e a próxima lição assume que o repo está em dia:

```bash
git add -A
git commit -m "capstone: demo script, worker panel, README/runbook"
git push
```

## Challenge

A extensão solo. O recuo da ajuda se completa aqui: sem apoio, sem composição nomeada, sem interface dada. Escolha exatamente uma:

(a) Um tipo novo de alvo de sonda, de ponta a ponta: uma variante nova na união de alvos da frota, a lógica do check num worker, uma linha no painel. Você construiu uma versão menor disso no challenge de m07-l1; esta aqui atravessa a stack inteira.

(b) Um painel novo no painel sobre dados que a estação já produz. A estação emite mais do que mostra: o histórico de sondagem está sentado no git log como um status.json por execução de cron, o /status do poller carrega leituras de blockchain que nenhuma superfície pública mostra, os workers guardam timestamps por alvo em KV. Um painel de histórico construído a partir de um punhado de commits recentes é a entrada forte clássica aqui, e note a fronteira antes de escolher a opção do poller: o painel não consegue alcançar o seu localhost, então trazer dados do poller para a superfície quer dizer que o pipeline carrega eles, não uma aresta nova para dentro da sua casa.

(c) Um alerta: algum caminho pelo qual um estado ruim vira um estado barulhento. A forma dentro dos limites que vale a pena roubar: um worker escreve uma flag de degradado no snapshot de KV dele, e um step do Actions lê o endpoint público e falha a execução quando a flag está setada, o que dispara o alarme de m09-l2 que você acabou de confirmar.

![Três cartões comparam um tipo novo de sonda, um painel novo e um alerta, acima de um único banner compartilhado declarando que só habilidades ensinadas são permitidas.](assets/v08-comparison.webp)

A única regra é a própria tarefa: só habilidades ensinadas. Um SDK de bot do Telegram é uma boa ideia e uma dependência não ensinada, então ele falha. Fazer deploy do poller num cluster Kubernetes foi sinalizado como fora de escopo na trava de tier do M6 e continua lá; orquestração é problema de outro curso. A regra não é modéstia, é o teste: este curso gastou dez módulos substituindo o reflexo de agarrar uma ferramenta nova pela habilidade de selecionar entre as que você já tem. Prove que pegou.

Aceitação, as cinco: a extensão está visível numa superfície com deploy; ela aparece no README, diagrama incluído se ela adicionou uma aresta; o script de demo continua passando duas vezes consecutivas com zero consertos manuais; `git grep` procurando qualquer coisa com forma de segredo em todo repo continua voltando limpo; e a extensão está commitada e pushada com o resto do capstone.

## Checkpoint

O que você consegue fazer agora, concretamente, e vale a pena ler esta lista devagar porque ela é o estado terminal do curso: desenhar um sistema distribuído poliglota de memória e saber quais setas não existem; verificar um sistema de oito arestas de ponta a ponta contra um script de demo que você escreveu; compor dois padrões ensinados numa aresta de produção nova sem um tutorial; operar a coisa toda a partir de um README com uma tabela de pins datada; e estender um sistema vivo solo, dentro das fronteiras da sua própria stack. Dez módulos atrás você instalou o Node.

O par de evidências de fechamento, como prometido lá em cima: a transcrição do seu script de demo, todos os checks em OK na segunda execução consecutiva, e um screenshot do painel mostrando os três painéis no ar. A recuperação de 30 segundos antes de você fechar a aba: quais três setas do seu diagrama estão deliberadamente ausentes? (Nada entrando no poller, os workers não consomem nada, o poller não alimenta nenhuma superfície pública.) E por que o commit do próprio cron não redispara o pipeline? (A trava do GITHUB_TOKEN: eventos escritos por workflow não geram execuções, e a sua estação conta com isso.)

Um pedido enquanto o suor está fresco. Esta lição apostou tudo na forma de checkpoint-depois-verificar, nenhum exemplo trabalhado, confiando em dois módulos de recuo para te carregar. Me conte onde ela se sustentou e onde ela te derrubou, e nomeie a única aresta cujo check foi o mais difícil de escrever. Se uma aresta come uma hora do tempo de montagem de todo mundo de forma consistente, esse é exatamente o feedback que remodela este capstone.

A estação está inteira e o script de demo prova isso, duas vezes. Resta uma lição, e ela não fia nada: nenhuma ferramenta nova, nenhum código novo. Um mapa de exatamente onde você está agora, lido contra os cursos que vêm depois, quais bookmarks do módulo 1 acabaram de ficar urgentes, e qual porta do catálogo a sua estação destranca primeiro. Você construiu o sistema. Agora a gente lê o mapa que ele te deixa na mão.
