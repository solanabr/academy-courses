# Montagem: uma loja, todos os trilhos

## Resumo

Na lição passada você colocou a stack inteira na frente de um checklist de entrada no ar e deixou ela falhar no papel: toda linha vermelha virou uma tarefa de correção, e você trabalhou a lista até cada peça estar individualmente digna de produção. Aquilo foi a auditoria. O que ela não conseguiu testar é a coisa para a qual esta lição existe. Quinze degraus, cada um verde no próprio canto do repositório com o próprio smoke test, não é uma loja. Um comprador não liga para as suas pastas. Um cliente de verdade consegue entrar com zero SOL, comprar um disco, ser atendido, assinar, sobreviver a uma renovação que falha e levar um reembolso, tudo sem você tocar em uma única coisa na mão?

Hoje você responde essa pergunta com um arquivo de log. Antes de qualquer teoria, faça o inventário. A partir da raiz do seu repositório:

```bash
find . -mindepth 2 -maxdepth 3 -name package.json -not -path '*/node_modules/*' | wc -l
```

Conte eles (o mindepth pula o manifesto da raiz, e o nível extra de profundidade pegaria qualquer manifesto aninhado uma pasta mais fundo; nesta árvore ele não deve achar nenhum, porque todo workspace fica no nível de cima). Cada um desses manifestos é um workspace que você construiu e provou — a maioria dos degraus ganhou um próprio, embora não todos, e a nota sobre o elenco logo abaixo nomeia os que moram dentro de outro workspace ou em código nenhum — e a contagem volta em quatorze, um a menos que o elenco abaixo porque o workspace `stack` de hoje à noite ainda não existe. Depois abra o seu `gate/report.md` mais recente ao lado da contagem: as tarefas de correção que você fechou na lição passada são a razão de hoje à noite poder ser sem graça. Nenhuma delas, sozinha, consegue vender um disco para um estranho. Esse vão entre "todos os testes verdes" e "um negócio roda" é a última habilidade que este curso ensina e, sinceramente, é aquela pela qual engenheiros de integração são pagos.

As descobertas logo de cara:

- Você entrega a **wavelength-stack**: um repositório onde toda rota importa o transfer-kit como o núcleo de pagamento compartilhado, as superfícies de transaction-request, blink e x402/MPP ficam montadas em UM servidor (a página de checkout por QR do módulo 3 continua uma página estática autônoma de propósito; o caminho de pagamento dela é o mesmo núcleo de transaction-request que a superfície montada exercita), o worker de webhook e o crank de cobrança rodam como processos de background, e `npm run journey` dirige uma jornada de comprador roteirizada de sete pernas contra a devnet.
- Não existe conceito novo nesta lição. Por design. Toda linha estrutural é um import de algo que você já construiu, e o trabalho inteiro da lição é deixar isso visível: o blink reusa o builder de transação do módulo 3, os ids de fatura em memo do x402 conciliam no mesmo livro-razão do back office que os seus checkouts, e o verificador do módulo 4 é o único juiz de toda perna.
- Os dois pins de kit da lição de assinaturas sobrevivem intactos à montagem: os workspaces wavelength-checkout, backoffice e x402 ficam na linha kit-6 e o workspace subscriptions fica no kit 7.1.1. Hoje à noite é a noite em que eles finalmente dividem uma árvore, porque a montagem registra `subscriptions` no elenco da raiz — então espere o `ERESOLVE`, e entenda que o que protege o crank em tempo de execução é onde o arquivo de entrada dele mora, nunca a fronteira do workspace.
- A trava é brutal e simples: sete linhas PASS do verificador, código de saída 0, e o checklist de prod-gate do módulo 8 reavaliado contra a stack montada, com toda linha ou passando ou carregando uma tarefa de correção escrita.

## Passagem de som: quinze degraus, uma noite de estreia

Pense em hoje à noite como a noite de estreia de uma casa de shows. Todo instrumento chegou no próprio case e passou no próprio teste de bancada. A passagem de som não é sobre nenhum instrumento; é sobre se a sala funciona quando tudo toca junto. Aqui é igual: montagem é uma disciplina própria, com modos de falha próprios, e nenhum deles mora dentro de um único degrau.

![Diagrama de arquitetura de três processos: um servidor montando as superfícies de transaction-request, gasless, blink e x402, um worker de webhook e um crank de subscriptions isolado como a ilha kit-7, todos compartilhando o transfer-kit e um único livro-razão de pedidos.](assets/v01-diagram.webp)

### A jornada do comprador é a spec

As apostas, ditas como a diferença entre dois desfechos. Se a stack montada funciona, o dinheiro de um estranho vira um pedido atendido, uma assinatura rodando e uma linha auditável no livro-razão enquanto você dorme. Se ela quase funciona, você ganha o pior desfecho do comércio: o dinheiro chega e nada acontece, e agora um humano tem que conciliar na mão aquilo sobre o que os seus sistemas discordam. A jornada de sete pernas existe para tornar impossível esconder o "quase funciona".

As pernas, na ordem em que o script roda elas. Um comprador, roteirizado, na devnet:

1. **Stub do ramp.** O comprador parte de moeda fiduciária. A Coinbase não vai onboardar uma conta de teste headless, então esta perna afirma o contrato de session token que você construiu na lição de onramp: existe uma URL pay.coinbase.com, ela vincula `defaultNetwork=solana`, e o endereço da carteira não aparece em lugar nenhum nela.
2. **Primeira compra patrocinada pela Kora.** O comprador tem USDC de devnet e zero SOL, e compra um disco mesmo assim. O fee payer na transação que aterrissou é o signer da Kora, não o comprador. Um recibo, e o diagrama de paymaster do módulo 8 é real.
3. **Pedido atendido por webhook.** Uma segunda compra aterrissa pelo fluxo de transaction-request, e o evento da Helius chega no seu worker pelo mesmo túnel que você expôs na lição de webhook; subir esse túnel pertence ao ritual de boot do terminal um, não a mãos no meio da execução. Aí o script da jornada dá replay do payload entregue direto no endpoint do worker mais duas vezes, o substituto local da reentrega da Helius, e o pedido é atendido exatamente uma vez, incluindo a venda que a sua fair queue drenou no boot.
4. **Ciclo de assinatura mais dunning forçado.** O comprador entra no clube do disco do mês, um pull de cobrança aterrissa como fatura conciliada, e aí o script esvazia a ATA do comprador e força uma renovação a falhar. A falha precisa virar uma fatura em aberto. Não uma nova tentativa.
5. **Compra por blink.** O blink do drop serve os metadados dele, recebe `{account}` e devolve uma transação construída pelo builder exato do módulo 3.
6. **O agente paga a API três vezes.** Um agente pagante bate no endpoint pressing-price, engole o 402, liquida e faz isso mais duas vezes. Três ids de fatura em memo conciliam no livro-razão.
7. **Um reembolso.** Um pagamento de push reverso pelo transfer-kit, registrado contra a signature de origem.

![Fluxograma das sete pernas da jornada, do stub do ramp até o reembolso, cada uma alimentando o verificador compartilhado do lado do servidor que checa programa de token, mint, delta de saldo e memo antes de imprimir PASS.](assets/v02-flowchart.webp)

A jornada não é um passeio pela UI, e nenhuma perna jamais confia num toast de carteira, num payload de webhook ou numa resposta 200 como prova. O curso tem uma única bancada de aceitação, o verificador do m04, e a jornada chama ela uma vez por perna: buscar de novo a transação com `getTransaction`, checar o programa de token, depois o mint, depois o delta de saldo na conta de token PERTENCENTE ao lojista (chaveada no dono, do jeito que o verificador chaveia desde o m04 — essa escolha é o que pega a fixture de mint errado), depois o memo. Uma transação patrocinada recebe o mesmo tratamento que uma comum. A Kora co-assinar muda quem pagou a taxa; não muda nada sobre o que merece ser acreditado.

### O layout, e a costura que você já resolveu

O formato-alvo é um monorepo npm único cujo `package.json` da raiz lista todo degrau como workspace. Você vem construindo em direção a isso desde o módulo 2 sem cerimônia; a montagem só deixa o elenco explícito. E um aviso antecipado, porque um colega de time organizado vai absolutamente propor colapsar tudo num workspace só numa versão de kit só enquanto você estiver lá dentro. Recuse, com educação, com os peer ranges na mão: o `@solana/pay` 1.0.26 tem peer em kit ^6.9 e o `@solana/subscriptions` 0.5.0 tem peer em kit ^7.0.0, e os dois estão certos. Essa costura foi o ponto inteiro da lição de assinaturas, e não é rederivada aqui: os workspaces wavelength-checkout, backoffice e x402 mantêm os pins de kit-6 e o workspace subscriptions mantém o kit 7.1.1.

Seja exato sobre o que mantém os dois separados, no entanto, porque a lição de assinaturas foi direta nisso e hoje é a noite em que isso é testado. Workspaces npm registrados **não** são isolamento. O npm faz hoist de todo pacote registrado para uma única resolução de raiz compartilhada, então no momento em que o elenco abaixo nomeia `subscriptions`, os dois majors de kit estão numa árvore só e o npm tem que reconciliar eles — que é precisamente o `ERESOLVE` que o passo de instalação abaixo espera em vez de torcer para evitar. Até agora `subscriptions/` ficava fora do elenco e nunca encontrou a árvore kit-6; registrar ele te compra scripts com `--workspace` e um lockfile só, e te custa essa reconciliação. O que sobrevive à fusão é o pin, não um muro: o npm estaciona um major na raiz e aninha o outro sob `subscriptions/node_modules`, que é o que o checkpoint abaixo verifica, e o Node então resolve os imports do crank a partir dali por um motivo só — é ali que o arquivo de entrada dele mora. O passo 4 diz a mesma coisa sobre o processo do crank, e vale ler duas vezes, porque "workspace diferente" e "processo diferente" são as duas respostas erradas para por que a ilha se sustenta.

Esses peer ranges foram reverificados contra o npm na lição de assinaturas em 2026-08-22; rode `npm view @solana/subscriptions@0.5.0 peerDependencies` você mesmo antes de instalar qualquer coisa hoje, porque este canto do npm se mexeu duas vezes neste trimestre e nunca, em lugar nenhum, fixe em `latest`.

![Diagrama do monorepo listando as quinze pastas de workspace com o workspace subscriptions isolado como a única ilha kit-7 e o novo workspace stack destacado no lado kit-6.](assets/v03-diagram.webp)

Nem todo degrau ganhou a própria pasta, e vale dizer isso em voz alta: o embed de ramp mora dentro do workspace wavelength-checkout porque nasceu daquele servidor, o gate MPP é um arquivo de config parado na frente do workspace x402 em vez de uma base de código própria, e o registro de decisão de corredor é um documento, não um processo. Degraus são capacidades, não diretórios. O seu elenco pode diferir do meu nos nomes; o array workspaces é a fonte da verdade, e ele precisa listar o que você de fato construiu.

### Importe, não reimplemente

Aqui está o acúmulo, mostrado em vez de afirmado, porque uma alegação tipo "tudo compõe" é exatamente o tipo de coisa que este curso te ensinou a não aceitar por fé. Três recibos:

**O transfer-kit é o núcleo de pagamento, por import.** O builder de txreq chama o `resolveAta` dele e cunha as reference keys dele; o builder de reembolso emite o push reverso dele pelo `sendStablecoin`; os pulls do crank liquidam em ATAs que ele resolve. Um módulo, escrito na semana um, movendo todo dólar da stack hoje à noite. Se você se pegar redigitando uma transferência checada em qualquer lugar deste lab, pare; você está reimplementando a sua própria dependência.

**O blink nunca aprendeu a construir uma transação.** O handler de POST dele chama `buildOrderTransaction` do workspace checkout-txreq do módulo 3 e embrulha o resultado num `ActionPostResponse`. Mesmo catálogo, mesma função de precificação, mesmo formato de memo. Quando a perna de blink da jornada passa no verificador sem nenhum código de pagamento específico de blink no diff, é a escada de artefatos dando retorno.

**Três protocolos, um livro-razão.** Um checkout por QR, um pull de assinatura e uma chamada de agente x402 são portas de entrada radicalmente diferentes, e cada uma delas aterrissa como uma linha no mesmo livro-razão de pedidos do back office, chaveada do mesmo jeito. Os ids de fatura do `extra.memo` da perna de x402 (256 bytes no máximo, da spec do x402 v2) conciliam pelo mesmo caminho que um memo de checkout. Uma história de conciliação para o negócio inteiro.

![Linha do tempo mostrando artefatos dos módulos dois a oito, cada um alimentando a montagem final da wavelength-stack, do transfer-kit como núcleo compartilhado até o checklist de prod-gate no fim.](assets/v04-timeline.webp)

### A bancada é enxuta de propósito

O script da jornada é deliberadamente sem graça: não spawne nada sofisticado, dirija cada perna e termine toda perna on-chain do mesmo jeito, com uma chamada para dentro do verificador e um erro lançado em qualquer veredito que não seja ok. A tentação na hora do capstone é construir um framework de teste esperto. Resista. Um driver enxuto sobre um juiz confiável vale mais que um framework rico que você escreveu na noite anterior à demo, porque quando uma perna falha às 2h da manhã você quer que a falha seja sobre a stack, nunca sobre a bancada. Uma nota de convenção antes do código: todo import entre workspaces nesses arquivos é um caminho TypeScript sem extensão, que só resolve porque a stack inteira roda sob o tsx; aponte um `node` pelado para qualquer um deles e os imports falham, o que é esperado, não quebrado.

A mesma austeridade vale para os dois loops de background. O worker e o crank são exatamente os processos que você já construiu; a stack não embrulha eles, não faz monkey-patch neles, não funde eles. Ela inicia eles. A única regra de integração que os dois precisam honrar é a guarda de idempotência da lição de webhook: o worker faz claim de uma signature antes de atender e o crank escreve faturas pelo mesmo caminho de claim, porque uma entrega repetida da Helius ou um tick de crank re-executado contra uma stack sem essa guarda atende um pedido em dobro, e a perna de webhook da jornada foi feita para pegar exatamente isso.

Existe uma ruga genuinamente nova que a bancada tem que respeitar, e ela é sobre tempo, não sobre dinheiro. Até agora todo smoke test que você escreveu afirmava uma coisa que o seu próprio código acabou de fazer: mande, depois cheque. Duas das pernas de hoje à noite afirmam coisas que um *processo diferente* faz no cronograma dele. A linha de livro-razão da perna de webhook aparece sempre que a entrega da Helius chega e o worker termina de verificar, o que, da cadeira do script da jornada, é um número imprevisível de segundos depois de o pagamento aterrissar. A fatura em aberto da perna de dunning aparece sempre que o próximo tick do crank percebe o pull que falhou. Se o script afirmar no instante em que a transação dele confirma, ele vai reprovar uma stack que está funcionando perfeitamente, só que funcionando de forma assíncrona. Então a bancada carrega uma segunda primitiva de espera ao lado do wrapper de retry: um poll limitado que observa uma condição virar verdadeira e desiste em alto e bom som depois de um prazo. Retry é para leituras que dão erro; polling é para efeitos que ainda não aconteceram. Manter os dois separados mantém as suas mensagens de falha honestas, porque "o RPC deu timeout" e "o worker nunca atendeu o pedido" são bugs diferentes com donos diferentes.

### Um comprador, saldos preparados

Leia a ordem das pernas de novo e você vai reparar que ela não é arbitrária; a jornada é uma pequena máquina de estados sobre os saldos de um comprador, e toda perna afirma alguma coisa e prepara a próxima. O script cunha um keypair de comprador fresco no início (persista ele em `/tmp/buyer.json` e exporte `BUYER_ADDRESS` a partir dele, para que tanto a checagem de vazamento quanto os seus comandos de shell no meio do debug consigam alcançar ele), financia a ATA dele com USDC de devnet pelo fluxo de faucet que você montou no módulo 2, e deliberadamente não dá SOL nenhum para ele. Essa pobreza é o ponto da perna 2: a compra patrocinada pela Kora precisa ter sucesso a partir de uma carteira que não conseguiria pagar a própria taxa base, e a afirmação de que o delta de lamports do comprador é exatamente zero só quer dizer alguma coisa se o saldo era zero para começo de conversa. Depois que a perna patrocinada passa, o script dá uma recarga no comprador com um airdrop pequeno de SOL, porque da perna 3 em diante o comprador se auto-assina e se auto-paga como qualquer cliente comum. O formato de CLI dessa recarga, se você quiser checar a sanidade de um comprador travado na mão no meio do debug:

```bash
solana airdrop 0.1 $(solana-keygen pubkey /tmp/buyer.json) --url devnet
```

A coreografia continua até o fim. A perna de assinatura drena a ATA do comprador de propósito para forçar a renovação que falha, o que quer dizer que a perna de blink que vem depois precisa financiar USDC de novo primeiro ou ela falharia pelo motivo errado, uma rejeição por pagamento a menor nascida da sua própria coreografia de teste em vez de uma superfície quebrada. Uma perna que falha pelo motivo errado é pior que uma perna que falha honestamente; ela te manda depurar uma superfície que funciona. Então o corpo de cada perna abre preparando o estado de saldo exato de que precisa e fecha afirmando o estado que criou. Escreva as linhas de preparação com o mesmo cuidado que as afirmações. Quando uma execução fica vermelha às 2h da manhã, a primeira pergunta é sempre "a perna falhou, ou a preparação antes dela mentiu?", e um script de jornada que loga os passos de preparação responde isso só pelo log.

Mais uma escolha deliberada: a jornada nunca reusa ids de pedido entre execuções. Toda execução carimba um run id fresco nos ids de pedido e nas strings de memo dela, então re-rodar a jornada contra um livro-razão que já guarda as linhas de ontem afirma só as linhas desta execução. O livro-razão é histórico append-only; a jornada é a fatia de uma noite dele. Chaveie as suas afirmações no run id e o script vira seguro de re-rodar para sempre, que é exatamente o que você quer da coisa que vai demonstrar com as mãos suando.

### O que quebra na hora da montagem

Quatro modos de falha respondem pela maior parte da dor neste lab, e eu estou te entregando eles logo de cara porque, na minha experiência com semanas de integração, isso muda o debug de horas para minutos.

![Tabela ligando quatro ciladas de montagem, contaminação cruzada de kit, guarda de idempotência ausente, fair queue não drenada e rate limits da devnet, aos sintomas observáveis e às correções delas.](assets/v05-comparison.webp)

A última linha merece uma frase extra, porque é a que engana as pessoas sob pressão de demo: o RPC público da devnet vai te dar rate limit no meio da jornada, e uma leitura que dá timeout parece exatamente uma perna que falhou. A distinção que importa é *qual lado disse não*. Um timeout é a infraestrutura de leitura dando de ombros; você dá retry nele. Uma rejeição do verificador é a sua bancada de aceitação falando; nessa você nunca dá retry, você investiga.

### O trade-off, e quem já roda nesses trilhos

Nomeie o trade-off antes do lab, como sempre. O monorepo montado roda todo serviço numa árvore de processos numa máquina só contra a devnet, e isso é exatamente certo para um capstone de ensino e errado para produção. Um deployment de verdade separa o worker, o crank e a API com paywall em serviços de vida longa distintos, com monitoramento próprio e políticas de restart próprias, e nunca compartilha um signer entre todos eles: o raio de impacto de uma chave vazada deveria ser um serviço, não a sua loja inteira. O capstone prova a fiação e a disciplina de verificar do lado do servidor. Ele não prova uma postura de ops, e as disciplinas mais fundas de aterrissagem e indexação que uma versão de alto volume precisa são território do curso Client-Side Mastery, como têm sido toda vez que este curso encostou nelas.

![Comparação da stack de ensino com um deployment de produção em processos, signers, monitoramento e rede, terminando com os invariantes que se transferem, verificação do lado do servidor, idempotência, um livro-razão e pins por workspace.](assets/v06-comparison.webp)

Alguma coisa disso é real fora de um repositório de curso? É sim, e com números. A Helius roda a própria cobrança no mesmo programa oficial de Subscriptions que você integrou, programa `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` (o prefixo vanity `De1eg` nomeia o programa Delegation on-chain sobre o qual o produto Subscriptions roda, a mesma nomenclatura que os códigos de erro do client dele usam), e declara a política de dunning dela nas mesmas palavras que a sua máquina de estados codifica: uma renovação que falha não é repetida contra a carteira, ela vira uma fatura em aberto (o blog de engenharia deles, buscado em 2026-08-21). A sua perna de falha forçada afirma exatamente o comportamento em que uma empresa de infraestrutura de verdade aposta a receita dela. E a perna de agente também não é especulativa: o dashboard do x402.org, a mesma janela móvel de 30 dias que você leu na lição de x402, reportou 75.41 milhões de transações e 24.24 milhões de dólares em volume quando eu puxei ele em 2026-08-21. Os trilhos em que você está passando o som hoje à noite estão carregando peso de verdade no mundo real, na escala que está em jogo, agora mesmo.

Já que estamos contando, conte o custo da própria noite, porque o número ainda surpreende gente que viveu nos trilhos de cartão. A jornada completa aterrissa em umas dez transações: duas compras, uma compra por blink, uma configuração e um pull de assinatura, três chamadas de agente, um reembolso, mais a venda drenada da fila. Cada uma carrega a taxa base de 5000 lamports e, pela faixa com que este curso abriu, liquida por uma pequena fração de um centavo — US$ 0.00075 a US$ 150 por SOL, US$ 0.0015 a US$ 300, e você cita a faixa em vez de um número cravado. Um item de linha ofusca todos eles e não é uma taxa: o rent de ATA da perna patrocinada, uns 0.0015 SOL na cotação da devnet em 2026-09-07 e ainda caindo, o gasto orçado que a sua linha de prod-gate já limita. Taxas à parte, a noite inteira de sete pernas, uma entrada de cliente, quatro vendas, um ciclo de cobrança, um cliente máquina e um reembolso, custa menos em taxas de rede que o erro de arredondamento de uma única transação de cartão. Essa aritmética é por que todo trilho nesta stack consegue existir, e vale ter ela na ponta da língua na próxima vez que alguém perguntar por que uma loja de discos se daria ao trabalho.

## Lab: levante a stack

Este é o último lab do curso, então deixa eu dizer a parte silenciosa em voz alta: o apoio acabou. O módulo 8 já começou a tirar ele, te entregando um runner de gate e quatro linhas prontas e fazendo você escrever o resto. Hoje à noite ele vai o resto do caminho. Você ganha a ordem de fiação, as duas peças de cola que são genuinamente novas (o supervisor de processos e a bancada da jornada) e uma perna trabalhada como o padrão. Todo o resto você monta a partir dos seus próprios degraus, porque depois desta lição não existe repositório de curso em que se apoiar, só o seu repositório.

**1. Unifique os workspaces.** O seu `package.json` da raiz cresceu organicamente desde o módulo 2. Faça dele o elenco explícito. O meu:

```json
{
  "name": "wavelength",
  "private": true,
  "type": "module",
  "workspaces": [
    "transfer-kit",
    "wavelength-checkout",
    "checkout-txreq",
    "pos-stall",
    "drop-blink",
    "verifier",
    "backoffice",
    "backoffice-refunds",
    "club-crank",
    "subscriptions",
    "dunning",
    "x402",
    "gasless-checkout",
    "fair-queue",
    "stack"
  ],
  "scripts": {
    "journey": "npm run --workspace stack journey"
  }
}
```

Depois crie o único workspace novo e reinstale a árvore:

```bash
mkdir -p stack/src
npm init -y --workspace stack
npm pkg set type="module" --workspace stack
npm install --workspace stack express@5.1.0 @solana/kit@6.10.0
npm install --workspace stack -D tsx@4 typescript @types/express @types/node
npm pkg set scripts.serve="tsx src/server.ts" scripts.boot="tsx src/boot.ts" scripts.journey="tsx src/journey.ts" --workspace stack
npm install
```

Pins, com as notas de frescor deles: o `express` fica em 5.1.0 para que o repositório inteiro compile contra uma versão só (o 5.x atual do npm é 5.2.1 em 2026-08-23; resista ao upgrade até conseguir subir todo workspace junto), e o `@solana/kit` 6.10.0 é o último release v6, o mesmo par que todo workspace kit-6 do repositório já carrega. O `tsx` é o runner que você usou o curso inteiro; a linha de dev-install é a instalação dele para este workspace fresco. Espere aquele último `npm install` falhar, e leia a falha em vez de sair pegando uma flag por reflexo. Registrar `subscriptions` põe o kit 6 e o kit 7 numa árvore só pela primeira vez neste curso, então o npm reporta o `ERESOLVE` entre os dois exatamente como a lição de assinaturas disse que reportaria; a saída de emergência é a que você já usou na lição de gasless, `npm install --legacy-peer-deps` na raiz. Aí o checkpoint: `npm ls --workspaces --depth 0` imprime todo workspace, e `subscriptions` é a única árvore mostrando kit 7.1.1 — aninhada sob o próprio `node_modules` em vez de ter sofrido hoist, que é o mecanismo que a seção da costura descreveu. Confirme que cada workspace ainda resolve a linha de kit que o próprio `package.json` dele fixa antes de seguir adiante; se qualquer workspace kit-6 agora reportar 7.1.1, a flag encobriu um conflito de verdade e você precisa consertar o pin, não a flag.

**2. Exporte os apps, trave os listens.** Cada workspace de superfície hoje termina o arquivo de servidor com um `app.listen` pelado. Importar um arquivo desses iniciaria um listener perdido, então dê a cada um a mesma edição de duas partes: exporte o app, e só escute quando rodado diretamente. Aqui está ela no checkout-txreq; repita verbatim (com os nomes certos) no drop-blink, no servidor x402 e no gasless-checkout — esse último é uma superfície de verdade com rotas próprias, não um caminho dentro de outro app, e a perna 2 da jornada de hoje à noite chama ele:

```typescript
// checkout-txreq/src/server.ts, the bottom of the file.
// Replace the bare app.listen call with an export plus a direct-run gate.
export { app as txreqApp };

const runDirectly =
  process.argv[1] !== undefined &&
  import.meta.url === new URL(`file://${process.argv[1]}`).href;

if (runDirectly) {
  app.listen(PORT, () => {
    console.log(`checkout-txreq listening on :${PORT}`);
  });
}
```

Uma correção companheira enquanto você está em cada arquivo: `express.static('public')` resolve contra o diretório de trabalho do processo, e hoje à noite um processo serve três degraus a partir da raiz do repositório. Ancore cada mount estático na localização do próprio arquivo em vez disso:

```typescript
// near the top of each surface's server file
import { fileURLToPath } from 'node:url';

const publicDir = fileURLToPath(new URL('../public', import.meta.url));
app.use(express.static(publicDir));
```

Checkpoint: `npx tsx src/server.ts` dentro de cada workspace de superfície ainda inicia aquela superfície sozinha, exatamente como antes. A trava quer dizer que nada mudou para execuções autônomas.

**3. Um servidor.** Agora a montagem, e ela é menor do que você espera, que é o ponto:

```typescript
// stack/src/server.ts
import express from 'express';
import { txreqApp } from '../../checkout-txreq/src/server';
import { blinkApp } from '../../drop-blink/src/server';
import { x402App } from '../../x402/src/server';
import { gaslessApp } from '../../gasless-checkout/src/server';

const app = express();
const PORT = Number(process.env.PORT ?? 3000);

app.get('/healthz', (_req, res) => {
  res.json({ ok: true, surfaces: ['txreq', 'blink', 'x402', 'gasless'] });
});

// Express apps are middleware: mounting at the root preserves each
// surface's own paths, including actions.json at the domain root.
app.use(txreqApp);
app.use(blinkApp);
app.use(x402App);
app.use(gaslessApp);

app.listen(PORT, () => {
  console.log(`wavelength-stack listening on :${PORT}`);
});
```

Montar na raiz importa para uma superfície em particular: o `actions.json` do blink precisa ficar na raiz do domínio ou as carteiras nunca renderizam ele, e um mount em sub-caminho quebraria caladamente a regra de hospedagem que você aprendeu na lição de blink. A superfície de gasless ganha uma linha própria aqui, e vale saber por quê, porque é fácil lembrar errado: o builder patrocinado reusa o `finalizeTransaction` do checkout-txreq, mas as rotas não moram lá. `GET`/`POST /gasless` eram servidas pelo próprio app Express delas no workspace `gasless-checkout`, na própria porta, e nada antes de hoje à noite jamais montou elas em outro lugar. Pule a linha `app.use(gaslessApp)` e a perna 2 da jornada ganha um 404 de uma stack que no resto parece saudável. Checkpoint: `npm run --workspace stack serve`, depois `curl localhost:3000/healthz`, `curl localhost:3000/txreq`, `curl localhost:3000/gasless` e `curl localhost:3000/actions.json` respondem todos de uma porta só.

Um pré-requisito que a superfície de gasless carrega e as outras não: ela conversa com um nó Kora. O builder dela cota e co-assina contra `http://localhost:8080`, então esse nó tem que estar rodando antes de a jornada começar, exatamente como estava na lição de gasless. Ele não é um dos três filhos do boot.ts abaixo, porque é um binário externo em vez de um processo que você inicia, ao lado do pay gate nesse aspecto.

![Mapa de rotas do servidor único na porta 3000 ramificando para health check, transaction request, a rota gasless da Kora montada, as actions de blink montadas na raiz e a rota de pagamento x402 (o gate MPP roda como um processo separado e não é montado aqui).](assets/v07-diagram.webp)

Uma palavra sobre a superfície mais quieta do módulo de protocolos, porque é fácil lembrar errado dela como já cabeada. O caminho de desafio MPP NÃO anda dentro do app x402: no módulo 7 o desafio `WWW-Authenticate: Payment` era servido pelo processo `pay gate` separado, dirigido pelo `paywall.yml` e fazendo proxy de um upstream sem pagamento, e nada hoje à noite muda essa arquitetura — exatamente o "arquivo de config parado na frente do workspace x402" da nota sobre o elenco lá em cima. O boot.ts spawna três processos, servidor, worker e crank, e um pay gate não é um deles, então a stack montada fala só x402. Se você quiser o lado MPP no ar, é mais um terminal, não código novo: exponha uma rota pressing-price pelada para o gate fazer proxy (a rota montada no x402 não pode ser o upstream dele, já que o gate exige uma sem pagamento), aponte o `paywall.yml` para ela e rode o gate na :4021 exatamente como no módulo 7. Você construiu para o trilho que tem tráfego; o que está chegando fica a um comando documentado de distância, que é a postura honesta para uma spec de método de pagamento que ainda se mexe no repositório dela em vez de ficar no relógio de algum órgão de padronização.

**4. Os loops de background.** O worker e o crank ficam como processos separados. Um modelo mental para corrigir antes de você cabear eles, porque é comum: não é a fronteira de processo, nem o diretório de trabalho, que preserva a costura de kit em tempo de execução. O Node resolve um import pelado subindo da localização do *arquivo que importa* até o `node_modules` mais próximo, então os imports do crank aterrissam na ilha kit-7 por exatamente um motivo — o `crank.ts` mora dentro de `subscriptions/`, cujo próprio `node_modules` guarda o kit 7. Ele resolveria igual lançado de qualquer diretório, e um processo spawnado com o cwd "certo" que importasse um arquivo fora da ilha ainda ganharia o kit 6. O que o processo separado te compra é isolamento de ciclo de vida — um crank que caiu não consegue derrubar o servidor — não isolamento de import; o endereço do arquivo de entrada faz isso. Primeiro aponte o workspace de cada loop para o arquivo de entrada dele (os meus são `src/worker.ts` no backoffice e `src/crank.ts` no subscriptions; use os seus nomes de arquivo reais):

```bash
npm pkg set scripts.start="tsx src/worker.ts" --workspace backoffice
npm pkg set scripts.start="tsx src/crank.ts" --workspace subscriptions
```

Depois o supervisor:

```typescript
// stack/src/boot.ts
import { spawn, type ChildProcess } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// npm's --workspace flag only resolves from the repo root, and boot itself
// runs with cwd inside stack/, so every child is spawned from the root
// explicitly. npm then executes each script with the workspace itself as
// cwd -- a convenience for the scripts' relative paths, nothing more:
// import resolution never depends on cwd, only on where each entry file
// lives.
const REPO_ROOT = fileURLToPath(new URL('../..', import.meta.url));

interface Proc {
  name: string;
  workspace: string;
  script: string;
}

// Three long-lived processes. The crank's entry file lives inside the
// kit-7 island, so its imports resolve there, never against our kit-6 tree.
const PROCS: Proc[] = [
  { name: 'server', workspace: 'stack', script: 'serve' },
  { name: 'worker', workspace: 'backoffice', script: 'start' },
  { name: 'crank', workspace: 'subscriptions', script: 'start' },
];

const children: ChildProcess[] = [];

for (const proc of PROCS) {
  const child = spawn(
    'npm',
    ['run', '--workspace', proc.workspace, proc.script],
    { stdio: ['ignore', 'pipe', 'pipe'], env: process.env, cwd: REPO_ROOT },
  );
  children.push(child);

  const prefix = `[${proc.name}]`;
  child.stdout?.on('data', (chunk: Buffer) => {
    process.stdout.write(`${prefix} ${chunk.toString()}`);
  });
  child.stderr?.on('data', (chunk: Buffer) => {
    process.stderr.write(`${prefix} ${chunk.toString()}`);
  });
  child.on('exit', (code) => {
    console.log(`${prefix} exited (${code ?? 'signal'})`);
  });
}

function shutdown(): void {
  for (const child of children) child.kill('SIGTERM');
  process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
```

Checkpoint: `npm run --workspace stack boot` mostra três linhas de inicialização prefixadas, e o Ctrl-C derruba as três. Se o crank logar um erro de tipo de kit aqui, você está com a cilada da contaminação cruzada: alguma coisa fora da pasta subscriptions está importando de dentro dela. A correção nunca é uma mudança de versão; é deletar o import.

**5. Drene a fila antes de as portas abrirem.** A fair queue da lição de offline guarda pelo menos uma venda assinada da sua última sessão de barraca. Rode o seu drain (`cd fair-queue && npx tsx drain.ts`) e espere a linha de fila vazia antes de você iniciar a jornada. A venda drenada flui pelo caminho de webhook como qualquer outra compra, que é por que a perna de webhook da jornada vai contar com ela, e por que afirmar antes de o drain terminar é a cilada três. Checkpoint: o drain imprime uma signature aterrissada por venda enfileirada e depois a linha de fila vazia, e ele parou de imprimir antes de você tocar no terminal dois. Se signatures ainda estiverem chegando, a jornada ainda não ganhou o direito de rodar.

**6. A bancada da jornada, e a única perna trabalhada.** O driver abaixo é o apoio inteiro que você ganha: o verificador compartilhado, um wrapper de retry que sabe distinguir um timeout de uma rejeição, o poll limitado para as duas pernas assíncronas, o impressor de PASS/FAIL, e a perna 1 trabalhada por inteiro como o padrão. As pernas 2 a 7 estão nomeadas, comentadas e são suas.

```typescript
// stack/src/journey.ts
import { execFile } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { createKeyPairSignerFromPrivateKeyBytes } from '@solana/kit';
import { createVerifier } from '../../verifier/src/verify';
import { createRpcFetchTransaction } from '../../verifier/src/rpc';
import { createMemoryStore } from '../../verifier/src/store';

const run = promisify(execFile);

// One fresh verifier for the whole journey. Its processed-signature store
// is OURS, separate from the worker's: the journey re-asserts every leg
// independently instead of trusting any ledger row the app already wrote.
export const verify = createVerifier({
  fetchTransaction: createRpcFetchTransaction({ commitment: 'confirmed' }),
  store: createMemoryStore(),
});

// The commitment policy rides into the harness too: ordinary legs assert
// at confirmed, and any leg your checklist marked high-value gets this
// stricter judge instead (in my seven, the refund leg). Same adapter,
// same RPC_URL-or-devnet default it has carried since the verifier lesson.
export const verifyFinalized = createVerifier({
  fetchTransaction: createRpcFetchTransaction({ commitment: 'finalized' }),
  store: createMemoryStore(),
});

export interface LegResult {
  leg: string;
  ok: boolean;
  detail: string;
}

// Footgun four lives here: a devnet read timeout is retried with backoff,
// but a verifier rejection is returned immediately and never retried.
export async function retryRead<T>(
  read: () => Promise<T>,
  tries = 3,
  delayMs = 2_000,
): Promise<T> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= tries; attempt += 1) {
    try {
      return await read();
    } catch (err) {
      lastError = err;
      if (attempt < tries) {
        await new Promise((resolve) => setTimeout(resolve, delayMs * attempt));
      }
    }
  }
  throw lastError;
}

// Polling is for effects another process has not produced YET (a ledger row
// the worker writes, an invoice the crank opens). Distinct from retryRead on
// purpose: an RPC error and a missing effect are different bugs.
export async function waitFor(
  condition: () => Promise<boolean>,
  label: string,
  timeoutMs = 60_000,
  pollMs = 2_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await condition()) return;
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  }
  throw new Error(`timed out waiting for ${label}`);
}

export async function assertLeg(
  name: string,
  body: () => Promise<string>,
): Promise<LegResult> {
  try {
    const detail = await body();
    console.log(`PASS ${name}: ${detail}`);
    return { leg: name, ok: true, detail };
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err);
    console.log(`FAIL ${name}: ${detail}`);
    return { leg: name, ok: false, detail };
  }
}

// Leg 1, worked in full: the ramp stub. Coinbase will not onboard a
// headless test buyer, so this leg asserts the session-token contract:
// the URL exists, it binds the network, and it leaks no wallet address.
async function legRampStub(): Promise<string> {
  const rampDir = fileURLToPath(new URL('../../wavelength-checkout/ramp-embed', import.meta.url));
  const { stdout } = await run('npx', ['tsx', 'smoke.ts'], { cwd: rampDir });

  if (!stdout.includes('pay.coinbase.com')) {
    throw new Error('no onramp URL printed');
  }
  if (
    !stdout.includes('sessionToken') ||
    !stdout.includes('defaultNetwork=solana')
  ) {
    throw new Error('URL missing sessionToken or defaultNetwork=solana');
  }
  // main() exports BUYER_ADDRESS from the keypair it mints at start. If it
  // is missing, fail the leg: an assertion that silently skips is worse
  // than no assertion, because it prints PASS while checking nothing.
  const buyer = process.env.BUYER_ADDRESS ?? '';
  if (buyer === '') {
    throw new Error('BUYER_ADDRESS unset: set it from the minted buyer before this leg');
  }
  if (stdout.includes(buyer)) {
    throw new Error('wallet address leaked into the onramp URL');
  }
  return 'session-token URL shaped correctly, wallet address absent';
}

// One buyer for the whole journey (see "One buyer, staged balances").
// Minted fresh in main() before any leg runs; legs 2-7 sign and pay with it.
let buyer: Awaited<ReturnType<typeof createKeyPairSignerFromPrivateKeyBytes>>;

async function mintBuyer() {
  // The staged-balances plan, made real: a fresh buyer per run, persisted
  // to /tmp/buyer.json in solana-keygen's 64-byte format so the mid-debug
  // airdrop command works, with BUYER_ADDRESS exported before any leg runs
  // (the ramp leg's leak check reads it, and child processes inherit it).
  // Funding it -- USDC to its ATA, deliberately zero SOL -- is your leg
  // bodies' staging work, not the mint's.
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const signer = await createKeyPairSignerFromPrivateKeyBytes(seed, true);
  const pubkeyBytes = new Uint8Array(
    await crypto.subtle.exportKey('raw', signer.keyPair.publicKey),
  );
  writeFileSync(
    '/tmp/buyer.json',
    JSON.stringify(Array.from(seed).concat(Array.from(pubkeyBytes))),
  );
  process.env.BUYER_ADDRESS = signer.address;
  return signer;
}

async function main(): Promise<void> {
  buyer = await mintBuyer();

  const results: LegResult[] = [];

  results.push(await assertLeg('ramp-stub', legRampStub));

  // Legs 2 through 7 are yours. Each chain leg's body ends the same way:
  // re-fetch through `verify` (wrapped in retryRead) and throw on any
  // result where ok is false.
  //
  // results.push(await assertLeg('gasless-first-purchase', legGasless));
  // results.push(await assertLeg('webhook-fulfilled-order', legWebhookOrder));
  // results.push(await assertLeg('subscription-and-dunning', legSubscription));
  // results.push(await assertLeg('blink-purchase', legBlink));
  // results.push(await assertLeg('agent-pays-3x', legAgentApi));
  // results.push(await assertLeg('refund', legRefund));

  const failed = results.filter((r) => !r.ok);
  console.log(
    `journey: ${results.length - failed.length}/${results.length} legs passed`,
  );
  process.exit(failed.length === 0 ? 0 : 1);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

Checkpoint antes de você escrever um único corpo de perna: com a stack subida no terminal um e só a perna 1 cabeada, `npm run journey` imprime uma linha `PASS ramp-stub:`, depois `journey: 1/1 legs passed`, e sai com 0. Prove que a bancada funciona enquanto ela ainda está julgando uma perna fácil; um driver que você depura pela primeira vez na perna 6 é um driver em que você não confia na perna 6.

O formato de fechamento de toda perna on-chain é sempre as mesmas quatro linhas, então aqui vai o padrão uma vez, precificado no pressing do catálogo, e depois ele nunca mais aparece:

```typescript
// the tail of every chain leg: one verifier verdict decides PASS
const result = await retryRead(() =>
  verify(signature, {
    recipient: STORE_WALLET,
    recipientAta: STORE_USDC_ATA,
    mint: '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
    amountBaseUnits: 12_500_000n,
    orderId,
  }),
);
if (!result.ok) throw new Error(result.reason);
```

Doze e meio USDC de devnet — o preço de catálogo do pressing, inalterado desde que a lição de transaction-request fixou ele — em unidades base, contra o mint de devnet que a config do seu transfer-kit fixa desde o módulo 2. Para a perna de gasless, adicione as duas leituras específicas de patrocínio na mesma transação buscada: o fee payer precisa ser igual ao signer da Kora e não pode ser igual ao comprador, e o delta de lamports do comprador precisa ser exatamente zero. Para a metade de dunning da perna 4, a afirmação não é sobre uma transação de jeito nenhum; é uma leitura de livro-razão provando que a falha forçada virou uma fatura em aberto e que nenhuma transação de nova tentativa contra a carteira do comprador existe.

![Fluxograma de uma renovação forçada a falhar em que o caminho que passa registra uma fatura em aberto sem nova tentativa contra a carteira, enquanto uma nova tentativa contra a carteira ou um revoke de autoridade reprovam.](assets/v08-flowchart.webp)

Para a perna de agente, lembre que a afirmação tem três lados: a chamada não paga precisa voltar 402, as três chamadas pagas precisam liquidar na devnet, e os três ids de fatura do `extra.memo` precisam aparecer no livro-razão do back office. Dinheiro que aterrissa mas nunca concilia reprova a perna. Isso é deliberado, e é a mesma lição que o livro-razão vem ensinando desde o módulo 4: no comércio, um pagamento não conciliado é um passivo vestido de fantasia de sucesso.

![Fluxo de um agente recebendo um 402, pagando com o header de signature de pagamento, depois lendo uma resposta de pagamento cujo id de fatura em memo concilia no livro-razão, repetido três vezes.](assets/v09-flowchart.webp)

É esse o lab inteiro, e dizer isso é o movimento pedagógico final da lição: seis passos, dois arquivos novos, zero código de pagamento novo. Todo o resto que você vai escrever hoje à noite são corpos de perna da jornada chamando interfaces que você já é dono.

## Challenge

Modo solo, dito sem rodeios: sem apoio, sem arquivos trabalhados, sem marcadores de TODO. Você ganha o layout-alvo acima e os sete critérios de aceitação abaixo, e cabeia e roda a stack sem ajuda.

O run-book são dois terminais. Terminal um: `npm run --workspace stack boot`, depois espere três linhas de inicialização saudáveis e a mensagem de drain de fila vazia. Terminal dois: `npm run journey`. Nada mais é tocado na mão entre esses dois comandos e a linha final de resumo; se você se pegar dando curl num endpoint no meio da execução para empurrar uma perna, a stack não é o que está inacabado, o script da jornada é. Financie a carteira do seu lojista com SOL de devnet antes de começar, prepare o comprador inteiramente de dentro do script, e carimbe todo id de pedido com o run id para que a jornada continue re-executável contra um livro-razão que já guarda execuções anteriores.

1. **ramp-stub**: a URL de onramp impressa contém `sessionToken` e `defaultNetwork=solana`, e o endereço de carteira do comprador não aparece em lugar nenhum nela.
2. **gasless-first-purchase**: o comprador começa com zero SOL; a transação buscada de novo mostra o signer da Kora como fee payer, exatamente uma signature do comprador, um delta de lamports do comprador de zero, e a ATA da loja creditada com o preço do disco; o verificador passa ela.
3. **webhook-fulfilled-order**: incluindo a venda drenada da fair queue, um evento entregue três vezes ainda produz exatamente uma linha de pedido atendida no livro-razão, e o atendimento aconteceu só depois de um veredito do verificador.
4. **subscription-and-dunning**: um pull de cobrança é conciliado como linha de fatura; a renovação forçada a falhar vira uma fatura em aberto sem transação de nova tentativa contra a carteira, e a assinatura não é desmontada.
5. **blink-purchase**: os metadados do GET e o POST com `{account}` devolvem respostas de action conformes à spec, a transação vem do builder do módulo 3, e a compra que aterrissou passa no verificador.
6. **agent-pays-3x**: a chamada não paga devolve 402; três chamadas pagas liquidam; três ids de fatura distintos do `extra.memo` conciliam no livro-razão.
7. **refund**: um pagamento de push reverso emitido pelo transfer-kit, registrado contra a signature de origem, passando no verificador, com o livro-razão ligando as duas direções.

Aceite quando o `npm run journey` imprimir sete linhas PASS e sair com 0, E o checklist de prod-gate do módulo 8 for reavaliado contra a stack montada em vez dos degraus individuais; rode o runner dele exatamente como você fez no módulo passado (`npx tsx gate/run.ts` a partir da raiz do repositório); a pasta `gate` não precisa de lugar nenhum no elenco de workspaces. A reavaliação não é formalidade, e ela tem permissão para terminar RED: algumas linhas que estavam verdes por degrau ficam honestamente mais fracas na montagem, porque o checklist agora vê um signer compartilhado e uma árvore de processos onde antes via serviços isolados. O seu runner só fala pass e fail, então pontue uma linha honestamente degradada como fail, escreva a tarefa de correção e deixe ela dizer "separe antes de produção" onde essa é a verdade. A barra de aceitação da reavaliação é que toda linha fail carregue uma tarefa de correção verdadeira, não que a linha de veredito diga GREEN; um checklist que só passa parou de medir qualquer coisa. A política de commitment também faz parte desse checklist, então segure a linha que o curso traçou: `confirmed` para as pernas comuns, `finalized` onde o seu checklist marcou um fluxo como de alto valor.

## Checkpoint: sete linhas

Se a jornada estiver vermelha, trabalhe a tabela de ciladas antes de ler uma única linha de código de perna: um erro de tipo no crank é contaminação cruzada, um atendimento em dobro é a guarda de claim ausente, um not-found na venda enfileirada é uma fila não drenada, e um timeout que parece uma falha é a devnet dando rate limit nas suas leituras. Vou confessar a que me pegou quando eu rodei uma demo montada minha pela primeira vez: eu vi uma perna "falhar" três vezes, reescrevi um handler perfeitamente bom duas vezes, e a transação tinha aterrissado bem todas as vezes. O RPC público estava limitando as minhas leituras de verificação, não os meus pagamentos. O wrapper retryRead na sua bancada existe por causa exatamente daquela noite.

E para você saber o alvo em direção ao qual está depurando, aqui está o que uma noite verde parece, o último checkpoint do curso:

```
[server] wavelength-stack listening on :3000
[worker] backoffice worker ready
[crank] crank armed on plan wavelength-motm
PASS ramp-stub: session-token URL shaped correctly, wallet address absent
PASS gasless-first-purchase: Kora fee payer, buyer lamports unchanged, 12.5 USDC verified
PASS webhook-fulfilled-order: exactly one ledger row across three deliveries
PASS subscription-and-dunning: pull reconciled; forced failure -> open invoice, no retry
PASS blink-purchase: ActionPostResponse tx from the module-3 builder, verified
PASS agent-pays-3x: 402 gate live, three settlements, three memo ids reconciled
PASS refund: reverse push recorded against origin signature
journey: 7/7 legs passed
```

Passadas as ciladas, cada perna falha no próprio dialeto, e a essa altura você já encontrou cada um deles uma vez. Uma perna de gasless em que o saldo de lamports do comprador se mexeu quer dizer que a Kora patrocinou alguma coisa fora do seu allowlist ou que o fee payer caiu de volta no comprador; cheque a config de patrocínio antes do código da transação. Uma perna de assinatura recusando com `too-early` é a guarda de janela de período fazendo o trabalho dela contra o seu relógio de teste, a mesma aritmética de segundos-versus-horas que a lição de cobrança treinou. Uma perna de blink que funciona no curl e morre no client da jornada é o par CORS-e-`actions.json`-na-raiz da lição de blink, ressurgindo porque o mount mudou. Uma perna de agente em loop de 402 para sempre geralmente quer dizer que a liquidação aterrissou mas o id de memo nunca conciliou, então o gate continua tratando o agente como não pago; leia o livro-razão antes de ler os logs do facilitador. E uma perna de reembolso rejeitada como `wrong-reference` é uma deriva de formato de memo entre o builder de reembolso e o que o verificador espera, que é um diff de uma linha contra o helper de memo do transfer-kit. Nenhum desses é um bug novo. Essa é a recompensa silenciosa de construir sobre os seus próprios degraus: todo modo de falha na stack montada é um que você já consertou uma vez, em algum lugar, com uma lição anexada.

E quando estiver verde, leia o log de verdade antes de seguir em frente. Sete linhas. Um estranho chegou da moeda fiduciária, comprou sem SOL, foi atendido exatamente uma vez por uma máquina, assinou, falhou uma renovação numa fatura em aberto limpa, comprou de novo por um link compartilhado, foi cobrado três vezes por um agente de software, e levou um reembolso que concilia com o pagamento original. Toda linha foi afirmada por um verificador que você escreveu contra estado da blockchain que você buscou, e nenhuma linha precisou de você. Quinze degraus cooperando não é mais uma alegação; é um arquivo de log, e você pode re-rodar ele amanhã.

Uma loja roda de ponta a ponta, e uma jornada de comprador completa passa sozinha. O que sobra não é código. Na próxima lição você nomeia o que de fato construiu, pesa os trilhos que agora tem contra os que o ecossistema ainda está assentando, e decide para onde um engenheiro de pagamentos aponta esse conjunto de habilidades em seguida. Traga o log da jornada; ele ganhou o lugar dele nessa conversa.
