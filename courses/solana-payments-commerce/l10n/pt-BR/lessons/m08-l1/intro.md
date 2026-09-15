# Gasless com Kora: ninguém leva SOL para uma feira de discos

Na lição passada você pôs a API de preço da prensagem atrás de um gate com `pay`, um endpoint respondendo x402 e MPP ao mesmo tempo, e viu o seu agente pagador liquidar direto por ele. Quem carregava essa liquidação, como carrega desde que você construiu o agente, era o facilitador: um serviço que liquida o pagamento de um agente e, sem alarde, paga a taxa da transação enquanto faz isso. O modo pull do MPP, o outro protocolo atrás daquele gate, senta um co-signatário do servidor exatamente no mesmo lugar, que é o fio que a lição passada mandou você segurar. Esse assento de fee payer está prestes a importar muito mais do que importava.

Imagine a banca da Wavelength numa feira de discos de fim de semana. Um colecionador chega, quer a prensagem limitada de agosto, e tem 30 USDC parados na carteira. Ele também tem exatamente zero SOL, porque comprou stablecoin numa exchange e ninguém contou para ele que existia um token de gás. No checkout que você construiu até aqui, a transação dele não consegue nem pagar a própria taxa de 5000 lamports. Ele vai embora. Você vê uma venda morrer por meio centavo de um token que o comprador não tinha motivo nenhum para ter.

A correção não é "faça o comprador arrumar SOL". O fee payer não precisa ser o comprador. Nunca precisou. Monte o workspace agora para a instalação rodar enquanto você lê, e já aproveite para gerar a estrela desta lição: uma carteira de comprador que nunca vai segurar um único lamport.

```bash
cd ~/wavelength   # the workspace root; gasless-checkout must sit beside transfer-kit and checkout-txreq or the ../../ imports below cannot resolve
mkdir -p gasless-checkout/src gasless-checkout/verify
cd gasless-checkout
npm init -y
npm pkg set type=module   # the smoke script uses top-level await and import.meta
npm install @solana/kit@6.10.0 @solana-program/token@0.14.0 @solana/kora@0.2.1 express@5 \
  @solana/kit-plugin-instruction-plan@^0.6.0 @solana/kit-plugin-payer@^0.6.0 \
  @solana/kit-plugin-rpc@^0.6.0 @solana-program/compute-budget@0.16.0 --legacy-peer-deps
npm install -D tsx@4 typescript @types/express @types/node --legacy-peer-deps
solana-keygen new --no-bip39-passphrase -o buyer.json
```

As duas linhas de install carregam `--legacy-peer-deps`, e a segunda não é um deslize de copiar e colar. O npm revalida a árvore de dependências inteira a cada install, não só os pacotes que você nomeou, então o conflito abaixo é reavaliado quando você acrescenta quatro ferramentas de dev que não têm nada a ver com ele. Tire a flag da segunda linha e ele sai com `ERESOLVE` antes de instalar qualquer coisa.

Notas de pin, checadas em 2026-08-31. O `@solana/kora` 0.2.1 é o `latest` do npm, publicado em 2026-03-27 (os betas mais novos de 0.3.0 ficam só na tag `beta`). Ele peera `@solana/kit` ^6.1.0, que o nosso 6.10.0 satisfaz sem ruído, e este workspace fica na linha v6 do kit como todo degrau de checkout antes dele, então não vá buscar o client de subscriptions do kit ^7 aqui. A complicação é que duas das outras faixas de peer do SDK são mais velhas do que o que este curso fixa: ele pede `@solana-program/token` ^0.12.0 contra o nosso 0.14.0, e `@solana-program/compute-budget` ^0.13.0 contra o nosso 0.16.0. A mensagem de erro do npm nomeia a primeira que ele encontrar, normalmente a compute-budget, então não se assuste quando o texto divergir deste parágrafo. Os nomes que o Kora de fato importa dos dois pacotes existem sem mudança nos nossos pins, que é por que `--legacy-peer-deps` é seguro neste workspace específico e não um hábito para levar adiante. O `express` 5 e o `tsx` 4 são os mesmos majors que o servidor de transaction request já roda. O `@solana-program/compute-budget` está fixado em 0.16.0, o último minor cuja faixa de peer aceita um kit v6 (o 0.17.0 pulou o peer dele para ^7) — o mesmo dígito que a lição da receita de taxa fixa mais adiante neste módulo, então o repo declara um único número de compute-budget em todo lugar. O `--legacy-peer-deps` também desliga a instalação automática de peers do npm, que é por que os quatro pacotes de plugin são nomeados explicitamente.

Mais um arquivo antes da teoria, porque um checkpoint mais adiante no lab se apoia nele. Este workspace alcança de volta o `transfer-kit` e o `checkout-txreq` por caminhos relativos, e o `tsconfig.json` da raiz, lá do módulo 2, só incluía `transfer-kit/src`, então o `tsc` rodado aqui não checaria o tipo de nenhum dos arquivos que você está prestes a escrever. Dê ao workspace a config dele:

```bash
cat > tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "types": ["node"]
  },
  "include": ["src", "verify", "../transfer-kit/src", "../checkout-txreq/src"]
}
JSON
```

`moduleResolution: "bundler"` é a configuração honesta aqui, em vez do `NodeNext` da raiz: todo import entre pacotes neste curso é escrito sem extensão de arquivo, que é o que o `tsx` resolve e o que o `NodeNext` rejeita. A lista `include` é o ponto — ela nomeia os dois pacotes irmãos que este workspace importa, então um caminho relativo velho ou uma assinatura de função que andou lá do outro lado falha aqui em vez de se esconder até o runtime.

## Resumo

O índice de achados, cada linha acionável:

- O fee payer é um assento, não uma identidade. O módulo 1 perguntou quem paga a taxa e a resposta era o comprador. Na lição passada um facilitador pagou por um agente. Hoje um paymaster paga por um humano, e a carteira do comprador não precisa de SOL nenhum, nunca.
- O Octane, a resposta antiga para este problema, foi arquivado em 2026-04-20 e o README dele agora aponta para o sucessor. O Kora é esse sucessor: um nó paymaster JSON-RPC 2.0 da Solana Foundation, com uma auditoria da Runtime Verification (relatório 20251119) e pagamento de taxa em tokens SPL embutido.
- Você entrega o **gasless-checkout**: um nó Kora rodando local com uma config trancada, um client de patrocínio com teto de taxa, e um builder patrocinado que reusa o `finalizeTransaction` do checkout-txreq com uma entrada trocada: o fee payer agora é o signatário do Kora, não o comprador.
- A transação carrega duas signatures: o Kora assina como fee payer, o comprador assina só a transferência. A taxa base é de 5000 lamports por signature, então o seu patrocinador paga 10,000 lamports por checkout, mais qualquer taxa de prioridade que você anexar.
- O custo maior se esconde em outro lugar: se o patrocínio cria uma conta de token, isso é o mínimo isento de aluguel de 165 bytes — pergunte ao `getMinimumBalanceForRentExemption(165)`, que respondeu 1,488,440 lamports na devnet em 2026-09-07 e está caindo conforme o SIMD-0437 desce a alíquota em degraus — e o comprador pode reaver isso depois fechando a conta. Orce como gasto, nunca como empréstimo.
- Um paymaster sem regras de validação é um ralo aberto. As allowlists do Kora e a política de fee payer dele, que assume negar tudo quando é omitida, são a história de segurança inteira: você vai configurar as allowlists e deixar de propósito a política de fee payer no negar-por-padrão dela.
- Verifique: `npx tsx verify/gasless.smoke.ts` imprime `sponsored=true`, um fee payer igual ao signatário do Kora e não ao comprador, um delta de lamports do comprador de exatamente 0, e uma signature do comprador.

A divisão de trabalho, dita sem rodeio: este é o módulo 8, território solo. A config do nó e o builder patrocinado vêm resolvidos porque a fiação do Kora é a última superfície de integração nova deste curso. O tratamento da cotação de taxa e a montagem da dupla signature são TODOs de completion contra critérios de aceitação, não passo a passo. A regra de validação que prova que o seu paymaster recusa transações estranhas é inteiramente sua no challenge.

![Detalhamento de custos empilhados de um checkout patrocinado: uma taxa base de 10,000 lamports por duas signatures, uma taxa de prioridade opcional e um depósito de rent de conta de token condicional, muito maior.](assets/v01-chart.png)

## O assento do fee payer

### Terceiro inquilino, mesmo assento

Toda transação Solana nomeia uma conta como o fee payer dela: a primeira conta da message, aquela cuja signature serve também de id da transação, aquela de quem o runtime debita a taxa base. Nada no protocolo diz que essa conta precisa se beneficiar da transação. Ela só precisa assinar e ter lamports.

O curso vem rondando esse assento desde o módulo 1. Lá atrás a pergunta "quem paga a taxa" tinha uma resposta sem graça: o comprador, porque a carteira do comprador construía a transação e se colocava em primeiro. Na lição passada a resposta ficou interessante: o facilitador x402 liquidava o pagamento do agente e sentava ele mesmo no assento de fee payer, que é exatamente por que o seu agente precisava de saldo em USDC e de SOL nenhum. Hoje o padrão ganha um nome e uma ferramenta de produção. Um paymaster (alguns ecossistemas dizem relayer) é um serviço cujo trabalho inteiro é ocupar o assento de fee payer nas transações dos outros, sob regras que o operador dele controla. Mesmo assento nas três vezes. A única coisa que mudou é quem senta.

Um cuidado vindo da costura entre a lição passada e esta, já que as duas ferramentas são vizinhas naturais: um facilitador x402 pode ele mesmo rodar em cima do Kora, e o guia oficial desse par ainda importa os nomes de pacote `x402` e `x402-express` no estilo v1. O SDK vivo é a linha v2 com escopo `@x402/*` contra a qual você construiu na lição passada. Construa v2, e leia os nomes pelados do guia como deriva de documentação, não como instrução.

![Um assento de fee payer com três inquilinos em sequência: o comprador no módulo 1, o facilitador x402 no módulo 7 e o paymaster Kora do lojista aqui.](assets/v02-diagram.png)

### O Octane morreu e nomeou o sucessor

Durante anos a resposta padrão para gasless na Solana era o Octane, um relayer da comunidade que co-assinava transações em troca de tokens SPL. Se você pesquisar "gasless Solana" hoje, ainda vai achar tutoriais construídos em cima dele. Não siga eles. O Octane foi arquivado em 2026-04-20, e o README dele agora diz "Check out Kora." Um paymaster que morreu e apontou para o próprio sucessor é mais ou menos o sinal de depreciação mais limpo que este ecossistema produz; aceite a dica pelo valor de face.

O Kora é esse sucessor: um nó paymaster da Solana Foundation, escrito em Rust, falando JSON-RPC 2.0 sobre HTTP puro. Você roda ele (ou aluga um hospedado), dá um signatário para ele, e ele expõe um conjunto pequeno de métodos para o seu backend: `getConfig`, `getPayerSigner`, `getSupportedTokens`, `estimateTransactionFee`, `signTransaction`, `signAndSendTransaction`, e alguns parentes. Dois detalhes importam para um lojista decidindo se coloca isso no caminho do dinheiro. Primeiro, ele consegue precificar taxas em tokens SPL, então um comprador só com USDC pode pagar a própria taxa em USDC se você configurar esse modo; hoje a gente roda o modo mais simples, em que o lojista simplesmente come a taxa. Segundo, ele carrega uma auditoria da Runtime Verification (relatório 20251119), que é o dado de credibilidade que separa "um signatário atrás de uma porta aberta" de uma infraestrutura que você consegue defender colocando no assento de fee payer de toda venda que você faz.

O lado TypeScript é um pacote só, o `@solana/kora`, que você já instalou: um client tipado e fino em que cada método é uma chamada JSON-RPC. Sem mágica, e você vai ler as respostas você mesmo no lab.

![Linha do tempo da era Octane até a auditoria do Kora em 2025 e os lançamentos do começo de 2026 do kora-cli 2.0.5 e do client @solana/kora 0.2.1, terminando no arquivamento do Octane em 2026-04-20.](assets/v03-timeline.png)

### O ida e volta da dupla signature

Aqui está o coração mecânico da lição, e ele é menor do que parece. Uma transação Solana é uma message mais uma signature por signatário exigido, e todo signatário assina os mesmos bytes. Então um checkout patrocinado é só uma transação com dois signatários exigidos em vez de um: o paymaster, listado primeiro como fee payer, e o comprador, exigido porque ele é a autoridade na transferência de USDC. Nenhuma das duas signatures é especial além da posição. A montagem é uma corrida de revezamento:

1. O seu servidor constrói a transação exatamente como o checkout-txreq sempre construiu, com uma entrada trocada: o `feePayer` é o endereço do signatário do Kora (você pergunta ao nó via `getPayerSigner`), não o comprador.
2. O servidor manda a transação base64 não assinada para o `signTransaction` do Kora. O Kora simula ela, roda ela contra as regras de validação dele e, se passar, devolve a mesma transação com a signature de fee payer preenchida. O slot do comprador continua vazio. É por isso que o método assume `sig_verify: false` por padrão: ele precisa conseguir assinar uma transação que ainda não está totalmente assinada.
3. A carteira recebe essa transação parcialmente assinada, mostra ao comprador o que ela faz, coleta a signature do comprador e envia. Isso não é um truque de protocolo pregado na lateral: a spec de transaction request que você implementou no módulo 3 permite explicitamente que o servidor devolva uma transação parcialmente assinada, e este é o caso para o qual essa previsão existe.

Por que `signTransaction` e não `signAndSendTransaction`, se o nó oferece os dois? Por causa de quem está faltando. No momento em que o Kora vê a transação, o comprador ainda não assinou, então o nó não consegue enviar ela; ele só consegue contribuir com a signature dele e devolver os bytes. O sign-and-send existe para o fluxo inverso, em que o client já segura uma transação totalmente assinada pelo comprador e quer que o paymaster co-assine e retransmita num salto só. Para um checkout, o sign-only também é o padrão mais seguro: o envio fica com a carteira, o que quer dizer que a recusa do comprador é um veto de verdade, não uma corrida contra um relay. A config do lab desabilita a variante de send de vez, um verbo a menos para um atacante sondar.

A ordem importa numa direção só: o Kora assina antes do comprador porque a signature do Kora é computada sobre a message, e a message precisa estar final (fee payer, instruções, blockhash) antes de alguém assinar. Depois disso, as signatures podem ser anexadas em qualquer ordem; elas não cobrem umas às outras. E o blockhash lá dentro acerta o seu relógio: cerca de 150 blocos de validade, uns 45 segundos no tempo de slot alvo atual de 300ms (o estágio de 300ms do SIMD-0525 entrou em vigor na epoch 1024, em 2026-08-28), o que sobra para construir, co-assinar e um toque num celular, e é exatamente por que você constrói a transação por requisição em vez de pré-assinar uma pilha delas.

![Fluxo de checkout patrocinado: o servidor constrói uma transação não assinada com o Kora como fee payer, o Kora assina primeiro, depois a carteira do comprador assina e envia, e o patrocinador é debitado.](assets/v04-flowchart.png)

### Quando o comprador paga a taxa em USDC

Antes da conta do lojista-come-a-taxa, conheça o modo que esta lição deliberadamente não constrói, porque você vai encontrar ele por aí e o capstone pode querer ele. O Kora consegue cobrar a taxa do comprador em um token SPL em vez de absorver ela. O fluxo acrescenta uma instrução: o seu servidor chama `estimateTransactionFee` com um `fee_token`, recebe de volta tanto `fee_in_lamports` quanto `fee_in_token` (o mesmo custo, denominado nas unidades base do próprio mint), depois pede ao `getPaymentInstruction` uma transferência pequena desse token do comprador para o endereço de pagamento do nó, anexa ela à transação e segue exatamente como antes. O comprador continua segurando zero SOL; ele só paga alguns centavos de USDC pela carona, e o seu float de patrocínio vira uma conta de capital de giro que recicla em vez de um subsídio que drena.

Dois botões de config governam o preço dessa carona. O `price_source` nomeia o oráculo que converte lamports em tokens, e o modelo `[validation.price]` define a margem: `free` (o modo de hoje, o comprador não paga nada), `margin` (custo mais um percentual) ou `fixed` (um valor fixo por transação, num token que você nomeia). O lab fixa `price_source = "Mock"` por um motivo que vale lembrar: mints de devnet não têm mercado ao vivo, então uma fonte de oráculo de verdade cotaria besteira; produção vira isso para a precificação da Jupiter e o resto da config continua de pé. Cobrar-do-comprador é o modo que transforma um paymaster de custo de marketing em recurso de pagamentos, e cada linha do que você constrói hoje é reaproveitável nele: só mudam o modelo de preço e uma instrução anexada.

### O que o patrocínio custa, honestamente

Agora a parte que o seu contador vai perguntar, porque gasless não é de graça, é pré-pago por você.

A linha visível é pequena. A taxa base é de 5000 lamports por signature, e o checkout de dupla signature carrega duas, então cada venda patrocinada custa 10,000 lamports do seu float, mais qualquer taxa de prioridade que você anexar. Um esclarecimento que vai te poupar um diagnóstico errado mais adiante: essa taxa base de 5000 lamports é fixa e não se mexe com congestionamento. Quando a rede está cheia, o que sobe é a taxa de prioridade opcional que você escolhe anexar, nunca a base. Se a sua carteira de patrocínio drena mais rápido do que a sua conta de taxa prevê, a taxa base não é a suspeita. Nesse ritmo, um airdrop de 2 SOL banca dezenas de milhares de checkouts, e se a história parasse aí, o patrocínio seria um erro de arredondamento.

Não acaba aí. O evento caro é a conta de token associada. Se o seu fluxo patrocinado alguma vez criar uma ATA para o comprador (a primeira conta de USDC dele, um mint novo, um token de fidelidade), o depósito de rent é o mínimo isento de aluguel para a conta de 165 bytes, e você lê ele do `getMinimumBalanceForRentExemption(165)` em vez de ler de qualquer página, incluindo esta — na devnet em 2026-09-07 isso deu 1,488,440 lamports, mais ou menos 150 vezes a taxa da transação de dupla signature inteira. Leia de novo antes de orçar, porque o SIMD-0437 está descendo a alíquota por byte em degraus e os dois clusters já se mexeram; o que é durável aqui é a razão, não os lamports. E aqui está a ressalva que o briefing de todo deploy de paymaster deveria carregar em negrito: esse rent não sumiu, ele está parado numa conta que o comprador é dono. O comprador pode fechar aquela conta de token quando quiser e ficar com o rent reavido. Não existe mecanismo para devolver ele para você. Então trate o rent patrocinado como gasto, precificado na venda como as taxas de processamento de cartão, e nunca lance ele como um empréstimo recuperável. Faça a conta de guardanapo você mesmo para uma feira de cem compradores: cem checkouts de dupla signature são 0.001 SOL de taxas, e cem ATAs de comprador de primeira viagem são cem vezes o que aquele curl acabou de te dizer — duas ordens de grandeza de distância em qualquer alíquota que a rede já tenha cobrado. A linha de rent é o orçamento; a linha de taxa é ruído.

![Gráfico de barras em escala logarítmica comparando uma taxa base de 10,000 lamports de dupla signature contra mais ou menos 1.5 milhão de lamports de rent de criação de ATA, cerca de 150 vezes maior e reavível só pelo comprador.](assets/v05-chart.png)

A outra linha honesta: quando o comprador já tem SOL, o patrocínio é custo puro. Você paga 10,000 lamports para poupar meio centavo de alguém que ele mesmo poderia ter pago, e alarga a sua superfície de ataque fazendo isso. O deploy maduro patrocina seletivamente (primeira compra, fluxos de onboarding, carteiras com zero SOL) em vez de por reflexo. A comparação abaixo é a decisão numa olhada, e é o trade-off desta lição inteira: gasless tira a exigência de SOL do comprador, e você paga por isso duas vezes, uma em rent que você deveria dar por perdido e uma numa carga de validação que agora é obrigatória.

![Tabela comparando o checkout pago-pelo-comprador contra o patrocinado em exigência de SOL, contagem de signatures, taxas, rent de ATA, conversão, superfície de ataque e quando cada modo ganha.](assets/v06-comparison.png)

### A validação é o produto

Por que tanta cerimônia com regras? Porque um paymaster é uma máquina que assina transações dos outros com uma chave cheia de dinheiro. Tire a validação e você construiu um faucet. Qualquer um que alcance a porta pode enviar a transação que quiser com o seu signatário no assento de fee payer: dreno de taxa no mínimo, e muito pior se uma instrução conseguir gastar de contas que o seu signatário controla. A auditoria acima cobre o código do Kora. Ninguém audita a sua config além de você.

A config do Kora te dá controles em camadas, e o lab define cada um deles de propósito. O `allowed_programs` é o muro externo: o Kora recusa patrocinar qualquer transação que invoque um programa fora da lista, e o seu checkout precisa de exatamente três (o Token program para o TransferChecked, o Memo para o carimbo do pedido, o compute budget porque as carteiras anexam ele). O `allowed_tokens` limita em quais mints ele vai cotar taxas. O `max_signatures` e o `max_allowed_lamports` limitam o raio de impacto de qualquer transação isolada. E aí tem o `fee_payer_policy`, o sutil: chaves finas para dizer se o próprio fee payer pode ser a origem de uma transferência do System, o dono numa transferência de token, uma autoridade de nonce e mais uma dúzia de papéis parecidos. É essa política que barra o dreno mais afiado de todos, uma transação cuja instrução interna move lamports para fora, de quietinho, da conta de patrocínio que está assinando ela.

A postura padrão é a certa: no código atual, toda chave do `fee_payer_policy` assume negar quando o bloco é omitido da config. A config de exemplo do repo escreve o bloco explicitamente — as chaves dela por acaso estão em false hoje, mas um exemplo copiado está a uma edição upstream de te inscrever em poderes que você nunca escolheu. No lab a gente omite o bloco de propósito e deixa o negar-por-padrão fazer o trabalho dele.

![kora.toml do lab anotado: allowlists de programa e de token, tetos de signature e de lamports, transações duráveis desligadas, precificação free e um bloco de política de fee payer omitido para que todo poder assuma negar.](assets/v07-annotated-code.png)

Essa é a superfície de confiança, dita sem alarde: você está rodando (ou alugando) um serviço que segura uma chave cheia de dinheiro e assina o que estranhos mandam para ele, e as regras de validação são a diferença inteira entre um paymaster e uma doação. Sóbrio, não assustador. Configure como quem fala sério e os modos de falha acima continuam teóricos.

A decisão de rodar-ou-alugar em si é conta de infraestrutura comum, e vale trinta segundos agora porque ela molda o deploy que você faz depois desta lição. Rodar o seu próprio nó, o caminho de hoje, quer dizer que você segura a chave de patrocínio, dimensiona o float, acompanha o saldo dele (o Kora vem com um endpoint de métricas exatamente para isso) e põe um `api_key` ou segredo HMAC em `[kora.auth]` antes de a porta ver a internet, porque um paymaster sem autenticação é um faucet público com passos a mais. Alugar um paymaster hospedado passa a carga de ops e a custódia da chave para outra pessoa, e leva a sua confiança junto: a config de validação deles, não a sua, decide o que é patrocinado em seu nome, então leia ela como um contrato. De um jeito ou de outro o modelo mental é o mesmo que você construiu para o facilitador na lição passada: um terceiro no caminho do dinheiro cujas regras você precisa conseguir recitar. A diferença é que este assina com uma carteira que você abastece.

## Lab: construa o gasless-checkout

O que você está montando, e onde isso fica no workspace da Wavelength:

![Diagrama do workspace mostrando o gasless-checkout reusando transfer-kit e checkout-txreq, conversando com um nó Kora local na porta 8080, e exportando buildSponsoredOrder para o capstone.](assets/v08-diagram.png)

1. **Instale o paymaster e gere o signatário dele.** O nó é um binário Rust; o SDK de client que você já instalou conversa com ele. Depois crie a carteira de patrocínio, a única carteira do lab que segura SOL, e abasteça ela na devnet:

   ```bash
   cargo install kora-cli
   solana-keygen new --no-bip39-passphrase -o sponsor.json
   solana airdrop 2 $(solana-keygen pubkey sponsor.json) --url devnet
   ```

   Nota de pin: `cargo install kora-cli` resolve para 2.0.5 em 2026-08-31 (publicado em 2026-03-11; os betas 2.2.0 são pré-lançamento, e o `cargo install` ignora eles); rode `kora --version` e espere a linha 2.x. Este é o único passo do curso inteiro que precisa de uma toolchain Rust: se o `cargo` não está na sua máquina, o `rustup` (rustup.rs) instala ele num comando só, um desvio de cinco minutos — e se você nunca encostou em Rust e prefere conhecer a toolchain direito em vez de instalar ela às cegas, o curso Rust & TypeScript Fundamentals levanta ela do zero no começo do módulo quatro dele, que abre prometendo "Em dez minutos você vai instalar a toolchain de Rust" e um scaffold de brinde. Mantenha o `sponsor.json` fora do git e longe da sua carteira de lojista: o patrocinador é uma conta de float que você recarrega, dimensionada para que perder ela doa em vez de arruinar. Checkpoint: o airdrop confirma e `solana balance $(solana-keygen pubkey sponsor.json) --url devnet` imprime 2 SOL.

2. **Configure o nó.** Dois arquivos na raiz do projeto. Primeiro o `kora.toml`, que é a seção de validação da teoria virada literal (o esqueleto é aparado da própria config de exemplo do repo; toda divergência daquele exemplo está comentada):

   ```toml
   # gasless-checkout/kora.toml
   [kora]
   rate_limit = 100

   [kora.auth]
   # open for local dev; set api_key or hmac_secret before this port faces the internet

   [kora.enabled_methods]
   liveness = true
   estimate_transaction_fee = true
   get_supported_tokens = true
   sign_transaction = true
   sign_and_send_transaction = false  # the wallet submits; this node only ever co-signs
   transfer_transaction = false       # not a checkout verb; off
   get_blockhash = true
   get_config = true
   get_payer_signer = true
   get_version = true

   [validation]
   max_allowed_lamports = 1000000
   max_signatures = 2                 # sponsor + buyer, nothing else
   price_source = "Mock"              # devnet mints have no live oracle price; Jupiter in prod
   allow_durable_transactions = false # stays off: next lesson's offline queue signs durable-nonce txs, and it deliberately does NOT route them through Kora (the merchant pays those fees directly)
   allowed_programs = [
       "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", # Token program: the TransferChecked
       "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr", # Memo v2: the order stamp
       "ComputeBudget111111111111111111111111111111", # compute budget: wallets attach it
   ]
   allowed_tokens = [
       "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU", # devnet USDC
   ]
   allowed_spl_paid_tokens = []
   disallowed_accounts = []

   [validation.price]
   type = "free"                      # merchant eats the fee; buyers pay zero

   # No [validation.fee_payer_policy] block on purpose: omitted means every
   # fee-payer power defaults to DENY. The repo's sample sets them all true.
   ```

   Depois o `signers.toml`, que diz ao nó onde a chave dele mora:

   ```toml
   # gasless-checkout/signers.toml
   [signer_pool]
   strategy = "round_robin"

   [[signers]]
   name = "wavelength_sponsor"
   type = "memory"
   private_key_env = "KORA_PRIVATE_KEY"
   weight = 1
   ```

   Um signatário de memória lê a chave dele da variável de ambiente nomeada, e o valor pode ser base58, um array de bytes `[0, 1...]` ou um caminho para um arquivo JSON de par de chaves. A forma de caminho quer dizer que a saída do seu `solana-keygen` funciona como está. Suba o nó num terminal só dele:

   ```bash
   KORA_PRIVATE_KEY=./sponsor.json kora --rpc-url https://api.devnet.solana.com \
     --config kora.toml rpc start --signers-config signers.toml
   ```

   O Kora escuta em :8080 por padrão. Checkpoint, de um segundo terminal:

   ```bash
   curl -s http://localhost:8080 -H 'content-type: application/json' \
     -d '{"jsonrpc":"2.0","id":1,"method":"getPayerSigner","params":[]}'
   ```

   O `signer_address` da resposta precisa ser igual a `solana-keygen pubkey sponsor.json`. Esse endereço está prestes a virar o fee payer de toda venda na feira.

   Segundo checkpoint, e pegue o hábito: pergunte ao nó no que ele acredita. Troque o método por `getConfig` no mesmo curl e leia o JSON de volta. Você deve ver os seus três programas permitidos, o único token permitido, `max_signatures` em 2, e um `fee_payer_policy` com todas as chaves em false, a postura de negar-por-padrão fazendo o trabalho dela sem você escrever uma única regra de negação, o que é honestamente uma mão na roda no dia em que alguém editar este arquivo com pressa. Se a resposta mostrar a política tudo-true da config de exemplo, o nó carregou um `kora.toml` diferente do que você acabou de escrever; conserte o caminho antes de seguir, porque toda afirmação de segurança desta lição depende de qual arquivo aquele processo de fato leu.

3. **O client de patrocínio.** Crie o `src/sponsor.ts`. A construção do client vem pronta; o tratamento da cotação de taxa é o seu primeiro TODO de completion, com as regras de aceitação dele bem em cima:

   ```typescript
   // gasless-checkout/src/sponsor.ts
   // One KoraClient for the whole package, plus the quote-and-cap gate every
   // sponsored order passes through before we ask the node to sign anything.
   import { KoraClient } from '@solana/kora';

   const KORA_URL = process.env.KORA_URL ?? 'http://localhost:8080';

   export const USDC_DEVNET = '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU';

   // The most SOL we will ever sponsor for one checkout. Two signatures at the
   // 5000-lamport base fee is 10_000; the cap leaves headroom for a priority fee
   // without letting one weird transaction bite the float.
   export const MAX_SPONSOR_LAMPORTS = 20_000;

   export const kora = new KoraClient({ rpcUrl: KORA_URL });

   export interface SponsorQuote {
     feeToken: string;
     feeInLamports: number;
     signerAddress: string;
   }

   export async function quoteSponsorship(transactionBase64: string): Promise<SponsorQuote> {
     // TODO(completion) 1: three calls, three rules.
     // a) const { tokens } = await kora.getSupportedTokens();
     //    This reports [validation].allowed_tokens from kora.toml (the mints the
     //    node will validate inside transactions), NOT allowed_spl_paid_tokens
     //    (the mints buyers may pay fees in, empty in our free mode), so under
     //    the config you just wrote the list is exactly one entry: devnet USDC.
     //    Pick USDC_DEVNET out of it; if it is absent, throw. An empty or
     //    surprising list means the node config drifted, and you want that loud.
     // b) const est = await kora.estimateTransactionFee({
     //      transaction: transactionBase64, fee_token: <the mint from a> });
     // c) if (est.fee_in_lamports > MAX_SPONSOR_LAMPORTS) throw with both numbers
     //    in the message; otherwise return { feeToken, feeInLamports:
     //    est.fee_in_lamports, signerAddress: est.signer_pubkey }. (Yes, the
     //    node spells the same key `signer_address` on getPayerSigner and
     //    `signer_pubkey` on the estimate and sign calls; two spellings, one
     //    sponsor key, and under type = "free" the estimate's fee_in_token is
     //    informational only, since the buyer is never charged.)
     throw new Error('Your turn: quote the fee and enforce the cap per a, b, c above.');
   }
   ```

   Por que pôr teto em algo que a config já precifica como free? Porque "free" é o preço do comprador, não o seu, e o `estimateTransactionFee` reporta o que o patrocinador vai ser debitado de verdade. O teto é o seu disjuntor para o dia em que uma carteira anexar uma taxa de prioridade absurda a uma transação que você está prestes a co-assinar.

   Pronto é assim, e o passo 7 é onde você descobre: o `quoteSponsorship` devolve um `SponsorQuote` cujo `signerAddress` bate com o endereço do `getPayerSigner` do passo 2, e joga um throw com os dois números na mensagem no momento em que uma estimativa passar de `MAX_SPONSOR_LAMPORTS`.

4. **O builder patrocinado.** Crie o `src/build-sponsored-order.ts`. Totalmente resolvido, e vale ler com atenção pelo tanto de coisa nova que não tem: a precificação é o `priceOrder` do módulo 3, a montagem é o `finalizeTransaction` do módulo 3, e a única mudança estrutural é de quem é o endereço que cai no slot `feePayer`:

   ```typescript
   // gasless-checkout/src/build-sponsored-order.ts
   // Same pricing, same assembly tail as checkout-txreq. One changed input:
   // the fee payer is the Kora signer, and Kora co-signs before the buyer sees it.
   import { address, generateKeyPairSigner, type Address } from '@solana/kit';
   import { getTransferCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta } from '../../transfer-kit/src/index';
   import { priceOrder, type OrderLine } from '../../checkout-txreq/src/catalog';
   import { finalizeTransaction } from '../../checkout-txreq/src/build-order-transaction';
   import { kora, quoteSponsorship } from './sponsor';

   const USDC_MINT = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const USDC_DECIMALS = 6;

   function merchantAddress(): Address {
     const configured = process.env.MERCHANT_ADDRESS;
     if (!configured) throw new Error('set MERCHANT_ADDRESS to the wallet checkout already pays');
     return address(configured);
   }

   export interface SponsoredOrderInput {
     account: string;      // the buyer's base58 pubkey, straight from the wallet POST
     lines: OrderLine[];
     orderId?: string;
   }

   export interface SponsoredOrder {
     transactionBase64: string; // already carries the sponsor's signature
     feePayer: string;          // the Kora signer; verify prints this
     reference: Address;
     memo: string;
     totalUsdc: string;
   }

   export async function buildSponsoredOrder(input: SponsoredOrderInput): Promise<SponsoredOrder> {
     const priced = priceOrder(input.lines);
     const buyer = address(input.account);
     const reference = (await generateKeyPairSigner()).address;
     const orderId = input.orderId ?? `fair-${Date.now().toString(36)}`;
     const memo = `wavelength:${orderId}:${priced.description}`;

     const { signer_address } = await kora.getPayerSigner();

     // Third seed since the roster lesson: the owning token program. Devnet
     // USDC is classic Token, so it is static here, exactly as in m03's builder.
     const sourceAta = await resolveAta(buyer, USDC_MINT, TOKEN_PROGRAM_ADDRESS);
     const destinationAta = await resolveAta(merchantAddress(), USDC_MINT, TOKEN_PROGRAM_ADDRESS);

     const transferIx = getTransferCheckedInstruction({
       source: sourceAta,
       mint: USDC_MINT,
       destination: destinationAta,
       authority: buyer,          // the buyer stays the transfer authority
       amount: priced.baseUnits,
       decimals: USDC_DECIMALS,
     });

     // The m03 tail, unchanged: reference injected, memo stamped, blockhash set.
     // The fee payer was always an input; today is the day that pays off.
     const unsignedBase64 = await finalizeTransaction({
       feePayer: address(signer_address),
       transferIx,
       reference,
       memo,
     });

     const quote = await quoteSponsorship(unsignedBase64);
     console.log(`[gasless] quote: ${quote.feeInLamports} lamports, cap ok`);

     const signed = await kora.signTransaction({ transaction: unsignedBase64 });

     return {
       transactionBase64: signed.signed_transaction,
       feePayer: signed.signer_pubkey,
       reference,
       memo,
       totalUsdc: priced.totalUsdc,
     };
   }
   ```

   O capstone importa o `buildSponsoredOrder` por este nome, então o export é estrutural do mesmo jeito que o `finalizeTransaction` era no módulo 3. Ele também monta o servidor que você está prestes a escrever dentro da stack montada, então mantenha as rotas abaixo em `/gasless` — esta é uma superfície por direito próprio, não um galho do app de transaction request que por acaso reusa a cauda dele. Repare também no que não mudou: nenhum campo de valor na entrada, nunca. Um checkout patrocinado continua sendo um checkout, e o servidor continua dono do preço.

   Checkpoint: `npx tsc --noEmit -p tsconfig.json` a partir de `gasless-checkout` checa os tipos limpo. Aponte para a config deste workspace, não para a da raiz — o `-p` é o que torna a checagem real, porque a config da raiz, lá do módulo 2, inclui só `transfer-kit/src` e sairia com 0 num arquivo que ela nunca abriu. Com a config certa é o jeito mais barato de pegar um caminho relativo velho nos três imports entre pacotes (`transfer-kit`, o catálogo do m03, o builder do m03), ou um helper lá do outro lado cuja assinatura andou desde a última vez que você chamou ele, antes que o servidor esconda qualquer um dos dois atrás de um 400.

5. **O servidor da feira.** Crie o `src/server.ts`. Totalmente resolvido; é o par de transaction request que você conhece, numa porta só dele, devolvendo uma transação que já carrega uma das duas signatures dela:

   ```typescript
   // gasless-checkout/src/server.ts
   // The gasless endpoint: GET for display metadata, POST {account} returns a
   // PARTIALLY SIGNED base64 transaction. Kora signed as fee payer; the buyer's
   // wallet adds the second signature and submits.
   import express from 'express';
   import type { Request, Response } from 'express';
   import { buildSponsoredOrder } from './build-sponsored-order';
   import type { OrderLine } from '../../checkout-txreq/src/catalog';

   const app = express();
   app.use(express.json());

   // :3200 is also where module 6's ramp-embed session route listens, and that
   // port is registered in your CDP allowlist, so do not renumber it there.
   // Stop the ramp server before starting this one, or run this on another
   // port with PORT=3210 -- nothing here hardcodes 3200 but this default.
   const PORT = Number(process.env.PORT ?? 3200);

   const ORDERS = new Map<string, { lines: OrderLine[] }>([
     // The limited August pressing, priced at 30 in the m03 catalog and kept
     // at 30 exactly ever since. This is the sale from the lesson's opener.
     ['fair-045', { lines: [{ sku: 'WVL-045', quantity: 1 }] }],
   ]);

   app.get('/gasless', (_req: Request, res: Response) => {
     res.json({ label: 'Wavelength Records (fees on us)', icon: 'http://localhost:3100/icon.png' });
   });

   app.post('/gasless', async (req: Request, res: Response) => {
     const account: unknown = (req.body as { account?: unknown } | undefined)?.account;
     if (typeof account !== 'string' || account.length === 0) {
       res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
       return;
     }
     const orderId = typeof req.query.order === 'string' ? req.query.order : 'fair-045';
     const order = ORDERS.get(orderId);
     if (!order) {
       res.status(404).json({ message: `unknown order: ${orderId}` });
       return;
     }
     try {
       const built = await buildSponsoredOrder({ account, lines: order.lines, orderId });
       console.log(
         `[gasless] order ${orderId}: ${built.totalUsdc} USDC, fee payer ${built.feePayer}, ref ${built.reference}`,
       );
       res.json({
         transaction: built.transactionBase64,
         message: `Wavelength Records: ${built.totalUsdc} USDC, network fee on us`,
         feePayer: built.feePayer,
       });
     } catch (err) {
       const message = err instanceof Error ? err.message : 'could not build the sponsored order';
       console.log(`[gasless] REFUSED order ${orderId}: ${message}`);
       res.status(400).json({ message });
     }
   });

   app.listen(PORT, () => {
     console.log(`gasless-checkout listening on :${PORT}`);
   });
   ```

   Rode ele num terceiro terminal, com a mesma carteira de lojista que o checkout vem pagando o curso inteiro:

   ```bash
   MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
   ```

   Se isso sair com `EADDRINUSE`, a rota de sessão do ramp-embed do módulo 6 ainda está segurando a porta: pare ela (ctrl-C no terminal dela), ou suba esta com `PORT=3210` e ajuste o `SERVER_URL` para bater na checagem de smoke abaixo. A rota do ramp fica com a :3200 porque aquela origem exata está registrada na sua allowlist da CDP e mudar ela significa editar um dashboard; este servidor não tem esse laço, então é ele que se muda. Checkpoint: `gasless-checkout listening on :3200`, e o terminal do Kora fica quieto até chegar um POST. A linha de log `REFUSED` no bloco catch é a evidência de negação que o challenge e o gate desta lição pedem que você produza.

6. **Abasteça o comprador com USDC e nada mais.** A carteira de comprador que você gerou na montagem não tem SOL, e continua assim. Dê o dinheiro do disco para ela usando o seu próprio kit do módulo 2, a partir da raiz do workspace:

   ```bash
   npm run --workspace transfer-kit pay -- $(solana-keygen pubkey gasless-checkout/buyer.json) 31
   ```

   O dólar a mais sobre o preço de 30 USDC da prensagem é deliberado: depois da venda o comprador deve terminar em 1 USDC, não em 0, para que um débito de preço exato se leia como sucesso e um saldo drenado a zero se leia como bug.

   Duas coisas acontecem aqui que ecoam a teoria. A sua carteira de lojista paga a taxa de transferência, e também custeia a ATA de USDC do comprador, que é o evento de rent da seção de custos, só que pago na perna de abastecimento em vez de na perna do checkout. Mesma economia, mesmo dono: aquele rent agora mora numa conta que o comprador controla. Checkpoint: `solana balance $(solana-keygen pubkey gasless-checkout/buyer.json) --url devnet` imprime exatamente 0 SOL, e o recibo do script de pay mostra 31 USDC entregues. Uma carteira com dinheiro e sem gás: o colecionador da abertura, reproduzido.

7. **A checagem de smoke.** Crie o `verify/gasless.smoke.ts`. Ele faz o papel da carteira do comprador: pedir a transação patrocinada, verificar quem é o fee payer, acrescentar a signature do comprador, enviar e provar que o comprador pagou zero lamports. A montagem da dupla signature é o seu segundo TODO de completion:

   ```typescript
   // gasless-checkout/verify/gasless.smoke.ts
   // Plays the wallet for a SOL-less buyer. Proves: fee payer is the Kora signer,
   // the buyer contributed exactly one signature, and the buyer paid 0 lamports.
   import { readFile } from 'node:fs/promises';
   import {
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     getBase64Encoder,
     getBase64EncodedWireTransaction,
     getCompiledTransactionMessageDecoder,
     getTransactionDecoder,
     partiallySignTransaction,
   } from '@solana/kit';
   import { KoraClient } from '@solana/kora';

   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
   const kora = new KoraClient({ rpcUrl: process.env.KORA_URL ?? 'http://localhost:8080' });
   const SERVER = process.env.SERVER_URL ?? 'http://localhost:3200';

   const bytes = new Uint8Array(JSON.parse(await readFile(new URL('../buyer.json', import.meta.url), 'utf8')));
   const buyer = await createKeyPairSignerFromBytes(bytes);

   const before = (await rpc.getBalance(buyer.address).send()).value;

   const res = await fetch(`${SERVER}/gasless?order=fair-045`, {
     method: 'POST',
     headers: { 'content-type': 'application/json' },
     body: JSON.stringify({ account: buyer.address }),
   });
   if (!res.ok) throw new Error(`server said ${res.status}: ${await res.text()}`);
   const { transaction } = (await res.json()) as { transaction: string };

   const sponsoredTx = getTransactionDecoder().decode(getBase64Encoder().encode(transaction));

   // Who is the fee payer? First static account of the message, by layout.
   const message = getCompiledTransactionMessageDecoder().decode(sponsoredTx.messageBytes);
   const feePayer = message.staticAccounts[0];
   const { signer_address } = await kora.getPayerSigner();
   if (feePayer !== signer_address) throw new Error(`fee payer ${feePayer} is not the Kora signer ${signer_address}`);
   if (feePayer === buyer.address) throw new Error('buyer is paying its own fee; sponsorship failed');

   // TODO(completion) 2: the dual-signature assembly.
   // a) const dualSigned = await partiallySignTransaction([buyer.keyPair], sponsoredTx);
   //    partiallySignTransaction ADDS the buyer's signature and preserves Kora's.
   // b) assert dualSigned.signatures[buyer.address] is non-null (the buyer signed),
   //    and that ALL entries in dualSigned.signatures are non-null (2 of 2 present).
   const dualSigned = sponsoredTx; // replace me

   const buyerSigned = dualSigned.signatures[buyer.address] != null;
   const totalSigs = Object.values(dualSigned.signatures).filter((s) => s != null).length;

   const signature = await rpc
     .sendTransaction(getBase64EncodedWireTransaction(dualSigned), { encoding: 'base64' })
     .send();

   for (let i = 0; i < 30; i++) {
     const status = (await rpc.getSignatureStatuses([signature]).send()).value[0];
     if (status?.confirmationStatus === 'confirmed' || status?.confirmationStatus === 'finalized') break;
     await new Promise((resolve) => setTimeout(resolve, 2000));
   }

   const after = (await rpc.getBalance(buyer.address).send()).value;

   console.log('sponsored=true');
   console.log(`feePayer=${feePayer} (kora signer, not the buyer)`);
   console.log(`buyerLamportDelta=${(after - before).toString()}`);
   console.log(`buyerSignatures=${buyerSigned ? 1 : 0} of ${totalSigs} total`);
   console.log(`signature=${signature}`);
   ```

   Com os dois TODOs preenchidos e os três terminais rodando (o Kora na :8080, o servidor na :3200, este script), rode o gate:

   ```bash
   npx tsx verify/gasless.smoke.ts
   ```

   Saída esperada, com os seus próprios endereços:

   ```
   sponsored=true
   feePayer=Ay5u...your sponsor pubkey (kora signer, not the buyer)
   buyerLamportDelta=0
   buyerSignatures=1 of 2 total
   signature=4dJx...a devnet signature
   ```

   Esse zero é a lição inteira. Uma carteira que nunca segurou um lamport acabou de comprar uma prensagem limitada, e a venda liquidou on-chain como qualquer outra. Se você ver uma falha de verificação de signature no lugar, o seu TODO 2 enviou a transação antes de o comprador assinar; se o terminal do Kora mostrar uma recusa, leia o motivo, porque é a sua config de validação falando, e é exatamente a voz que você quer alta.

## Challenge

**Completion.** Preencha os dois pontos de TODO: o `quoteSponsorship` em `src/sponsor.ts` pelas três regras dele, e a montagem da dupla signature em `verify/gasless.smoke.ts` pelas duas dela. A aceitação é o gate acima, ao pé da letra: um comprador de devnet sem SOL conclui a compra, o fee payer impresso é igual ao signatário do Kora e não ao comprador, `buyerLamportDelta=0`, uma signature de comprador presente. Guarde a signature de devnet e a pubkey de fee payer decodificada; são os artefatos de resposta desta lição.

**Solo, sem passo a passo.** O seu paymaster hoje confia no seu servidor. Prove que ele recusa todo o resto. Monte na mão uma transação que o seu checkout nunca emitiria, uma transferência do System program é a clássica (repare que a allowlist da config nunca incluiu o System program, de propósito: ATAs já existentes fazem o checkout nunca encostar nele). Construa ela com as mesmas chamadas de kit que você usou no módulo 3, aponte o fee payer para o endereço do signatário do Kora e mande direto para o `signTransaction` do nó. Depois aperte mais um parafuso: acrescente uma entrada em `disallowed_accounts` ou baixe o `max_signatures` para 1, reinicie o nó e veja o seu próprio checkout honesto falhar também, depois restaure. Aceitação: a requisição fora da allowlist é recusada com um erro do Kora nomeando a violação, a linha de log `REFUSED` do seu servidor captura uma negação de ponta a ponta, e o fluxo honesto continua passando depois. Guarde a linha de log da requisição negada junto com a signature de devnet; o gate pede as duas.

Se a transação montada for patrocinada em vez de recusada, confira qual arquivo de config o nó em execução de fato carregou antes de duvidar das regras; um caminho de `kora.toml` velho é o alarme falso clássico aqui, e o `getConfig` do SDK vai te mostrar exatamente no que o nó acredita.

Um pedido antes de você fechar os terminais. Esta lição levantou mais peças móveis do que qualquer degrau até aqui: um nó Rust, dois arquivos de config, três processos. Se alguma delas brigou com você, anote qual (o cargo install, o env do signatário, a complicação de peer-deps), porque o capstone presume que esta stack sobe limpa e as notas de atrito são como ela chega lá. E se o seu dreno montado foi recusado de primeira, diga isso em voz alta em algum lugar; você acabou de ver uma config de validação valer o que custa, que é uma coisa que a maioria das pessoas só aprende no postmortem do incidente.

A sua banca agora consegue vender para um comprador sem SOL. A próxima lição tira uma coisa maior: a rede. Uma feira de discos num subsolo sem sinal, vendas que ainda precisam ser assinadas, e uma fila que escoa quando você volta a ficar online. O fee payer manteve a venda viva hoje; nonces duráveis mantêm ela viva offline.
