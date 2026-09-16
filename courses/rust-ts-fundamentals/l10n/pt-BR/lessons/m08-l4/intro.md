# A checagem de saúde do caminho de escrita: uma transferência

## Resumo

O m08-l3 deu ao poller Rust as sondas de blockchain dele: leituras tipadas com reqwest parseadas com serde_json, uma taxonomia de falhas com thiserror para tudo que o RPC consegue lançar, e um re-ship da imagem do GHCR pelo pipeline do M6 sem tocar em uma linha dele. Toda superfície da estação lê a blockchain agora. Nada escreve nela. Hoje isso muda: você entrega o `tx-check`, um script que assina uma transferência de SOL de verdade, aterrissa ela na devnet, imprime a assinatura, e sai com código diferente de zero se qualquer elo daquela cadeia mentir. Aquela assinatura confirmada é o SHIP #5, e é o sinal de saúde mais forte que este curso vai te ensinar a emitir. A ajuda recuando, em voz alta: este é o pico de dificuldade do módulo, e você recebe o fluxo do pipe percorrido uma vez na tela com cada passo explicado. Depois você monta o script você mesmo, incluindo a lógica de airdrop-com-fallback e a troca de alvo. A verificação do delta de saldo no fim é totalmente solo. O próximo módulo te entrega checklists, não passo a passo.

## A sonda que muta

Um comando primeiro. A transferência de hoje aterrissa na devnet, então pergunte à devnet diretamente se ela sequer está viva, com o mesmo envelope que você vem fazendo POST desde o M7:

```bash
curl -s https://api.devnet.solana.com -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'
```

`{"jsonrpc":"2.0","result":"ok","id":1}` quer dizer que o alvo de tudo abaixo está respondendo. Agora a teoria.

Três lições de leituras provaram que a estação consegue perguntar. Elas não provam nada sobre se esta stack consegue agir. Uma leitura pode dar certo enquanto o caminho de escrita está completamente quebrado: bytes de chave errados, uma mensagem malformada, um blockhash expirado, uma rede que aceita a sua transação e depois dá de ombros. Leituras dividem exatamente duas superfícies de falha com escritas, conectividade e saúde do RPC, e você já tem essas cobertas de seis jeitos. Tudo acima daquela linha, validade da chave, construção da mensagem, aceitação da assinatura, inclusão, confirmação, é invisível para uma leitura. Existe exatamente um jeito de ver isso: assine alguma coisa de verdade e veja ela aterrissar.

![Uma stack de sete camadas onde as leituras só cobrem alcançabilidade e saúde do RPC enquanto uma transferência confirmada prova toda camada até a confirmação.](assets/v01-diagram.webp)

Esse é o enquadramento do artefato de hoje, e vale dizer sem rodeios antes de qualquer código: o `tx-check` é uma checagem de saúde, não uma demo. Páginas de uptime no mundo inteiro mostram pontinhos verdes que querem dizer "o servidor respondeu a um ping", e você vem construindo exatamente essa classe de sonda desde o M2, de propósito, porque é o primeiro degrau certo. Mas um cliente de blockchain que só sabe ler é uma estação de monitoramento de um sistema que ele não consegue tocar. A sonda que muta é uma espécie diferente: ela põe uma afirmação assinada no mundo e pede para a rede se comprometer com ela. Quando esse commitment volta, você provou as suas chaves, o seu código de construção de mensagem, a sua assinatura, a disposição da rede de te incluir, e a maquinaria de confirmação dela, tudo numa única execução de 30 segundos. Nenhuma combinação de leituras te dá nada disso.

Então o plano é honesto e pequeno: pegue uma chave descartável, consiga um pouco de SOL sem valor da devnet para ela, mande uma fração disso de volta para fora, e exija uma assinatura confirmada como o comprovante. A transferência em si é deliberadamente minúscula, 0.001 SOL da sua descartável para um segundo endereço novo, porque o valor não é o ponto. O comprovante é. E o primeiro passo é pedir dinheiro a um faucet, que é por que a primeira coisa honesta que esta lição faz é planejar para o faucet dizer não.

### Implore antes de construir

Faça isto agora, antes de qualquer teoria. No repo da estação, faça um workspace para o script e instale os dois pacotes de que ele precisa:

```bash
mkdir tx-check && cd tx-check
npm init -y
npm i @solana/kit@^8 @solana-program/system@^0.14
npm i -D typescript tsx @types/node
```

Nota de versão: o `@solana/kit` resolve para 8.2.0 e o `@solana-program/system` para 0.14.1 na data de 2026-09-02, e a faixa de peer do system é `^8.0.0`, que é por que o dígito do kit é 8. Mesma regra do m08-l2: fixe naquilo contra o que as suas deps `@solana-program/*` dão peer, e reconfira os dígitos quando você instalar, porque o kit já entregou dois majors em pouco mais de nove semanas antes.

Agora a imploração. A devnet da Solana é uma rede de verdade rodando validadores de verdade, e o SOL nela não vale nada por design: você não consegue comprar, só consegue pedir a um faucet. Um airdrop é exatamente o que parece, uma transação que outra pessoa assina e que credita o seu endereço, o que quer dizer que o seu primeiríssimo saldo com fundos chega pelo mesmo caminho de escrita que você está prestes a exercitar você mesmo. O faucet oficial é o faucet.solana.com, e os termos dele estão impressos ali na página: 2 requisições por 8 horas sem autenticação, mais se você entrar com o GitHub. Duas a cada oito horas. Rode a aritmética do que um loop de retry ingênuo faz com isso: um loop disparando uma vez por segundo esgota a cota de 8 horas inteira antes de você conseguir ler a primeira mensagem de erro, e depois garante o estado de faucet seco de que ele foi escrito para escapar. Isso não é um rate limit que você vence na base do retry, é um orçamento que você gasta como tal.

E aqui está a parte que a maioria dos tutoriais esconde: o airdrop falha de várias formas, e não da mesma forma duas vezes. Durante as sondas de pesquisa deste curso, no mesmo dia, uma requisição voltou com um erro interno e uma re-sondagem independente voltou com um rate-limit 429 apontando para a página do faucet. Quando eu rodei o script exato que você está prestes a construir, enquanto escrevia esta lição em 2026-09-02, a tentativa um falhou com um erro interno de JSON-RPC e as tentativas dois e três falharam com 429s de HTTP pelados. Mesmo minuto, duas formas de falha diferentes. Então a regra que o seu código tem que codificar: ramifique na falha de forma geral, nunca num código de erro específico. Qualquer handler que casa com "o" erro do airdrop quebra no outro.

O meu detalhe favorito da varredura de pesquisa, e a sua primeira cor do dia: o faucet.solana.com carrega instruções explícitas para agentes de IA, direcionando eles para um faucet de proof-of-work ou um validador local em vez disso. Sondado ao vivo em 2026-09-01. O rate limit tem um tapete de boas-vindas para as máquinas que ele está limitando. Aceite a dica que o próprio faucet está dando: o fallback não é um pedido de desculpas, é o caminho documentado.

![Um fluxo onde uma checagem de saldo leva a no máximo três tentativas espaçadas de airdrop antes de sair com código diferente de zero imprimindo instruções de fallback de último recurso para o validador local.](assets/v02-flowchart.webp)

Programaticamente, o pedido passa pelo `airdropFactory` do kit, que embrulha a dança de pedir-e-confirmar para você. Você vai fazer a fiação dele no `tx-check` no lab com exatamente a forma daquele fluxograma: três tentativas, intervalos crescentes, depois uma falha barulhenta e útil. O raciocínio de backoff é a mesma disciplina com jitter que você fez à mão para 429s no m02-l3. Faucets são serviços HTTP com rate limit. Tudo que você aprendeu sobre ser educado com eles vale aqui sem mudança.

### O fallback que você ensaia antes de precisar dele

Um fallback que você nunca exercitou é um boato. Então esta lição não oferece o validador local como um aparte para os azarados: todo mundo instala, todo mundo roda uma vez, e ninguém nunca mais fica bloqueado de vez por um faucet seco. Este também é o momento em que você para de alugar as blockchains dos outros de vez. O toolchain da Solana CLI já vem com uma blockchain de teste local completa, e ser dono de uma muda o que você consegue construir pelo resto deste curso e depois dele.

Um comando fixado, sondado ao vivo contra os docs de install da solana.com em 2026-09-02:

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

Esse script instala o release estável mais recente do Agave do toolchain. Realidade de freshness, em dois pontos de dados: a saída de exemplo dos próprios docs mostra `solana-cli 3.0.10`, e a máquina em que esta lição foi verificada puxou `3.1.10`. A sua provavelmente vai ser mais nova que as duas. Confirme que pegou:

```bash
solana --version
# solana-cli 3.1.10 (src:7bc9c805; feat:1620780344, client:Agave)
```

Depois inicie a sua própria blockchain:

```bash
solana-test-validator
```

Esse único comando sobe um cluster Solana completo de nó único na sua máquina: RPC em `http://127.0.0.1:8899`, websockets em `ws://127.0.0.1:8900`, um bloco genesis cunhado segundos atrás, e um faucet sem orçamento porque o mint é seu. O nome naquela string de versão, Agave, é o cliente validador em si, o mesmo software que os clusters públicos rodam, que é por que a sua blockchain local responde exatamente a interface JSON-RPC que as suas leituras vêm batendo desde o m08-l1. Ele continua rodando naquele terminal até você dar Ctrl+C, com os slots tiqueteando no canto, e escreve o ledger dele num diretório `test-ledger/` na pasta de onde você iniciou ele. Apague esse diretório e o próximo boot cunha um genesis novo: uma blockchain novinha, histórico zero, todo saldo resetado. Tem algo de esclarecedor nisso. O objeto global intimidador que a sua estação vem sondando com cuidado por um módulo inteiro acaba sendo software que você consegue iniciar, parar e limpar como qualquer outro processo. Honestamente, ter uma blockchain inteira no localhost é uma dádiva, e custa um comando.

Pare um segundo para assimilar o que você agora possui, porque isso sobrevive a esta lição. Toda leitura que a sua estação faz, todo script deste módulo, funciona contra esta blockchain trocando um par de URLs. Quando um experimento mais adiante precisar de cinquenta contas com fundos, ou mil transações num loop apertado, ou um teste que tem que começar do estado vazio, a devnet pública ia te impor rate limit até a miséria e a sua blockchain local nem vai reparar. E para acertar as expectativas com honestidade: nenhum dos cursos on-chain da Academy vai exigir este install de você, eles avaliam em ambientes hospedados e in-process de propósito. O validador local é a sua ferramenta de força pessoal para experimentos gananciosos demais para infraestrutura compartilhada, não um pré-requisito que alguma coisa rio abaixo pressupõe.

O trade-off, porque sempre tem um: a sua blockchain local é uma blockchain privada de um. Airdrops sempre aterrissam, transações sempre confirmam, nada está congestionado, e nada disso prova nada sobre o caminho da rede pública. Verde contra o `solana-test-validator` prova o seu código. Verde contra a devnet prova o seu código e a rede entre você e ela. Saiba qual pergunta cada alvo responde, e faça o seu script conseguir fazer as duas.

![Duas colunas contrastando a devnet como o teste do caminho de rede real contra o validador local como o teste de corretude de código ilimitado e instantâneo.](assets/v03-comparison.webp)

### Uma chave sem nada a perder

Toda escrita precisa de um signer, e este curso te dá o mínimo honesto: uma seed aleatória nova de 32 bytes, salva num arquivo, carregada com o `createKeyPairSignerFromPrivateKeyBytes` do kit. Esse helper recebe exatamente 32 bytes de material de chave privada e deriva a metade pública ele mesmo, verificado contra o kit 8.2.0. A chave mora em `.keys/devnet-throwaway.seed` dentro da pasta tx-check, e o nome é a política de segurança: ela é descartável, ela só vai segurar SOL de devnet sem valor, e ela entra no `.gitignore` antes da primeira execução, não depois.

Essa ordem é a lição de verdade. Um arquivo de keypair commitado é uma chave vazada para sempre. O histórico do git não esquece, force-pushes não limpam de forma confiável, e o hábito de colocar material de chave no gitignore antes de ele existir vale mais do que qualquer chave isolada. Mesmo uma chave de devnet: o SOL não vale nada, mas o hábito transfere para chaves que valem.

![Um hub mostrando uma seed Ed25519 produzindo assinaturas idênticas seja a chave morando num arquivo no gitignore, numa carteira de navegador, ou num dispositivo de hardware.](assets/v04-diagram.webp)

Dois glossários antes de a gente construir com ela, os dois estruturais. Primeiro, o signer. Os 32 bytes são uma seed de chave privada Ed25519, e a matemática da assinatura é idêntica quer a chave more num arquivo, num dispositivo de hardware, ou numa carteira de navegador. Onde a chave mora muda custódia e UX, nunca força criptográfica, então nada neste signer baseado em arquivo é brinquedo: ele produz exatamente as assinaturas que os validadores da mainnet verificam. Segundo, os dois substantivos em que a próxima seção se apoia. Uma instrução é uma unidade de trabalho endereçada a um programa: "system program, mova N lamports de A para B". Uma mensagem de transação é o envelope: uma ou mais instruções mais os metadados de que a rede precisa, quem paga a taxa e por quanto tempo o envelope continua válido. Você constrói a mensagem, você assina a mensagem, a rede executa as instruções dentro dela. Mantenha esses dois níveis separados na sua cabeça e o pipe abaixo se lê sozinho.

O que você deliberadamente não está ganhando aqui é uma carteira. Nenhuma extensão de navegador, nenhum botão de conectar, nenhum popup de assinatura. Carteiras lidam com a devnet numa boa, então a exclusão não é técnica: é uma fronteira. Wallet-standard, UX de conexão, e tudo que um prompt de assinatura envolve pertencem ao curso de domínio do lado do cliente, em produção enquanto eu escrevo isto; espere a profundidade de carteira e de aterrissagem de transação lá. Um arquivo cheio de bytes aleatórios é o signer que mantém esta lição sobre a coisa que ela ensina: como uma mensagem de transação é construída, assinada e confirmada. O momento em que você se pegar querendo uma carteira de verdade é exatamente o momento em que aquele território começa, e até o curso ser entregue, wallet-standard é o termo para procurar.

### O pipe, um passo honesto de cada vez

Agora o centro da lição. O kit constrói transações como um pipeline de transformações pequenas e puras sobre um valor de mensagem, e a forma canônica é uma chamada `pipe`. Aqui está a coisa inteira, exatamente como ela vai ficar no seu script, e depois a gente desmonta ela passo a passo. Todo identificador aqui é verificado contra os pacotes instalados: este código exato compila sob TypeScript estrito e aterrissou uma transferência confirmada enquanto esta lição estava sendo escrita.

```typescript
const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const message = pipe(
  createTransactionMessage({ version: 0 }),
  (m) => setTransactionMessageFeePayerSigner(signer, m),
  (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
  (m) =>
    appendTransactionMessageInstruction(
      getTransferSolInstruction({
        source: signer,
        destination: recipient.address,
        amount: lamports(TRANSFER_LAMPORTS),
      }),
      m,
    ),
);

const signed = await signTransactionMessageWithSigners(message);
assertIsTransactionWithBlockhashLifetime(signed);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
await sendAndConfirm(signed, { commitment: "confirmed" });

const signature = getSignatureFromTransaction(signed);
```

**Passo 1: `createTransactionMessage({ version: 0 })`.** Uma mensagem de transação é a ordem de serviço não assinada: quem paga, quais instruções rodam, e por quanto tempo a ordem é válida. Ela começa vazia. O campo de versão escolhe o formato de mensagem moderno; a versão 0 é o que o ferramental atual produz, e isso é tudo que você precisa saber aqui.

**Passo 2: `setTransactionMessageFeePayerSigner(signer, m)`.** Toda transação nomeia uma conta que paga a taxa, e isto define a sua. Uma batida de atribuição, porque ela vai te salvar de uma falha clássica de copiar-e-colar: o kit já vem com dois setters de fee payer. O exemplo de transferência do próprio repositório do kit usa a forma de endereço, `setTransactionMessageFeePayer`, que recebe um endereço pelado. A documentação do kit em solanakit.com ensina a forma Signer usada acima, que recebe o objeto signer em si para que o passo de assinatura mais adiante saiba exatamente quem tem que assinar. Este curso ensina a forma Signer, com a autoridade da solanakit.com. As duas formas são reais. Misturar metades de tutoriais que escolheram diferente é o jeito clássico de este fluxo quebrar.

![Lado a lado dos setters de fee payer na forma de endereço e na forma Signer com a mistura de metades de tutoriais apontada como o modo de falha.](assets/v05-comparison.webp)

**Passo 3: `setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m)`.** Este é o passo com um relógio dentro. A mensagem ganha um carimbo com um blockhash recente, e a rede só aceita a transação enquanto aquele blockhash ainda for recente, uma janela de 150 blocos, que no tempo de slot de 300ms que você mediu no m08-l1 dá uns 45 segundos. (Era um minuto lá atrás, quando os slots miravam 400ms, que é por que você ainda vai encontrar 'cerca de um minuto' em textos mais velhos.) Por que uma rede faria isso com você? Porque a alternativa é pior: sem uma expiração, uma transação que não conseguiu aterrissar poderia ficar no vazio e executar horas depois, depois de você desistir e mandar uma substituta. O lifetime é por que transações que não aterrissaram morrem de forma limpa em vez de te assombrar. A regra prática cai direto disso: busque o blockhash dentro do caminho de envio, logo antes de você construir a mensagem, nunca no início do script. Um script que constrói a mensagem dele na subida, faz dois minutos de outro trabalho, e depois manda, vai falhar a confirmação todas as vezes, e agora você sabe por quê antes de acontecer com você.

![Uma linha do tempo mostrando uma transação assinada em segundos aterrissando em segurança enquanto uma mandada depois de dois minutos chega passada a expiração de blockhash de aproximadamente 45 segundos, 150 slots de 300ms cada, e morre.](assets/v06-timeline.webp)

**Passo 4: `appendTransactionMessageInstruction(getTransferSolInstruction({...}), m)`.** Uma instrução é uma unidade de trabalho para um programa; uma mensagem de transação carrega uma lista delas. A nossa carrega exatamente uma: uma transferência de SOL do system program, construída pelo `getTransferSolInstruction` do `@solana-program/system` com um signer de origem, um endereço de destino, e um valor. O helper `lamports()` marca o valor com o tipo certo; um lamport, do m08-l1, é a unidade base, um bilionésimo de um SOL, e os valores são bigints porque u64 não cabe num number de JavaScript.

**Passo 5: `signTransactionMessageWithSigners(message)`.** Porque o fee payer entrou como um objeto signer, esta única chamada encontra todo signer obrigatório anexado à mensagem e produz a transação assinada. Nenhum malabarismo manual de chaves. A linha `assertIsTransactionWithBlockhashLifetime` depois dela é uma trava em nível de tipo que diz ao compilador o que a gente sabe, que esta transação carrega um lifetime de blockhash, que o passo de confirmação exige.

**Passo 6: `sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })`.** A factory recebe as suas conexões de RPC uma vez e devolve uma função de enviar-e-confirmar que você consegue reusar. Repare que ela quer as duas conexões, e a segunda finalmente explica por que este script abre um websocket: em vez de fazer polling de "já aterrissou?" num loop, a metade de confirmação se inscreve numa notificação para a sua assinatura e espera a rede falar. O contrato dela é o ponto inteiro desta lição: a função devolvida submete a transação assinada e resolve só quando a rede confirmou ela no commitment que você escolheu, ou lança. Não "enviada". Confirmada. A gente passa `commitment: "confirmed"`, que quer dizer que um validador incluiu a sua transação num bloco e uma supermaioria do cluster votou naquele bloco. Existem níveis mais rasos e mais fundos nesse botão, e a história completa do commitment, junto com o que a finality de fato te compra, é território do lado do cliente mais fundo do que uma checagem de saúde precisa. Para uma checagem de saúde, "o cluster votou nela" é exatamente a barra certa: forte o bastante para significar alguma coisa, rápido o bastante para rodar sob demanda.

Mais uma batida de honestidade sobre esse contrato, porque ele define os seus códigos de saída. Resolver é prova. Lançar nem sempre é a imagem espelhada da prova: um throw pode querer dizer que a transação foi rejeitada, ou que ela expirou sem aterrissar, ou meramente que o seu websocket soluçou enquanto a rede foi em frente e confirmou ela do mesmo jeito. Para o tx-check essa assimetria está tudo bem, uma checagem de saúde deve ser paranoica, e um alarme falso te custa uma re-execução. Para qualquer coisa movendo valor de verdade, "o envio lançou, logo não aconteceu" é um bug com contagem de corpos, e a disciplina de retry-e-dedup que lida com isso direito é parte da ciência de aterrissagem que este curso repassa. Saiba que a costura existe; não atravesse ela hoje.

![Um envio resolvido prova a confirmação enquanto um throw se abre em rejeição, expiração, ou um mero soluço de websocket, que é por que uma checagem de saúde pode levantar alarmes falsos.](assets/v07-diagram.webp)

O pipe são seis chamadas, e cada uma carrega um porquê. Esse é o padrão inteiro que este curso ensina para escritas, e ele é deliberadamente o piso: signer de keypair cru, taxas padrão, nenhuma sofisticação de retry. Num dia congestionado uma transação de taxa padrão pode simplesmente não aterrissar, e esta lição aceita isso como um resultado didático em vez de contrabandear meio curso de aterrissagem.

O que nos leva à caixa dos 20%, como prosa de repasse em vez de links, porque estas costuras são do tamanho de um curso, não do tamanho de um parágrafo. Ciência de aterrissagem de transação, priority fees, estratégia de retry-e-blockhash, e tudo que tem forma de carteira são um ofício do lado do cliente por si só: no momento em que você precisar de uma garantia de aterrissagem ou de uma carteira de navegador, essa é a costura para ir estudar direito, como um curso de estudo, não um post de blog. Transferências de token, esta aqui moveu só SOL nativo, pertencem ao curso Digital Assets, que percorre os programas de token direito. Esta lição te passa a escrita honesta mínima; estudo mais profundo te ensina a fazer ela ficar de nível de produção.

### Confirme, veja, pronto

Uma batida, como prometido, porque o passo a passo do explorer foi cortado para pagar pelo seu validador local. A assinatura que o `getSignatureFromTransaction` devolve é uma string base58 que nomeia a sua transação de forma única para sempre. Cole ela em qualquer explorer da Solana com o cluster ajustado para devnet, ou só imprima o link que o script constrói: `https://explorer.solana.com/tx/<signature>?cluster=devnet`. Você vai ver a transferência, a taxa, o slot em que ela aterrissou, e os dois saldos mudando. Aquela página é o comprovante público, de terceiro, de que a sua stack consegue agir. Olhe para ela uma vez, aproveite, feche a aba.

## Lab: tx-check

O enquadramento, antes dos passos: o `tx-check` não é uma demo, é uma feature da estação. É a checagem de saúde do caminho de escrita, rodável sob demanda, respondendo a única pergunta que nenhuma leitura responde: esta stack consegue assinar e aterrissar, não só ler? Ele imprime uma assinatura confirmada e sai com 0, ou falha alto e sai com código diferente de zero, o que faz dele composável: o script de demo do m10-l1 vai chamar ele exatamente por esse contrato. Você viu o pipe percorrido uma vez acima. Agora você monta o script em volta dele você mesmo.

1. **Gitignore primeiro.** Na raiz do repo da estação, antes de qualquer chave existir:

```bash
echo "tx-check/.keys/" >> .gitignore
echo "test-ledger/" >> .gitignore
git add .gitignore && git commit -m "chore: ignore devnet throwaway keys and local ledger"
```

(A linha do ledger é deliberadamente não ancorada: o `solana-test-validator` escreve `test-ledger/` em qualquer diretório de onde você iniciar ele, e o passo 7 só diz "um segundo terminal", então um padrão não ancorado cobre a raiz do repo, o `tx-check/`, e qualquer outro lugar de onde você lançar ele. Um padrão fixado em diretório deixaria a prova de `git status` do checkpoint cheia de ruído na primeira vez que você iniciasse o validador uma pasta ao lado.)

2. **Faça o scaffold do script.** Crie `tx-check/tx-check.ts` com as constantes e os imports. A troca de alvo são duas env vars com defaults de devnet; esta é a interface, então mantenha os nomes exatos:

```typescript
import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { randomBytes } from "node:crypto";
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromPrivateKeyBytes,
  generateKeyPairSigner,
  airdropFactory,
  lamports,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  sendAndConfirmTransactionFactory,
  getSignatureFromTransaction,
  assertIsTransactionWithBlockhashLifetime,
  devnet,
  type KeyPairSigner,
} from "@solana/kit";
import { getTransferSolInstruction } from "@solana-program/system";

const RPC_URL = process.env.RPC_URL ?? "https://api.devnet.solana.com";
const WS_URL = process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com";
const SEED_PATH = ".keys/devnet-throwaway.seed";
const TRANSFER_LAMPORTS = 1_000_000n; // 0.001 SOL
const MIN_BALANCE = 5_000_000n; // transfer plus fees, with slack

const rpc = createSolanaRpc(devnet(RPC_URL));
const rpcSubscriptions = createSolanaRpcSubscriptions(devnet(WS_URL));
```

   Uma ruga que merece o comentário dela: `devnet()` é uma marca em nível de tipo. Ela não custa nada em tempo de execução e diz ao compilador que este endpoint suporta airdrops, que é o que os tipos do `airdropFactory` exigem. O seu validador local honra o mesmo contrato de airdrop, então a marca vale para `http://127.0.0.1:8899` também.

3. **Escreva `loadOrCreateSigner`.** Contrato: se `SEED_PATH` existir, leia os 32 bytes dele e retorne `createKeyPairSignerFromPrivateKeyBytes(seed)`. Se não, dê `mkdirSync` na pasta `.keys`, escreva `randomBytes(32)` no arquivo, logue que uma descartável nova foi criada, e depois carregue ela do mesmo jeito. A função retorna `Promise<KeyPairSigner>`. Recarga determinística importa: o mesmo arquivo de seed tem que produzir o mesmo endereço em toda execução, ou o seu saldo de devnet encalha num endereço para o qual você não consegue mais assinar.

4. **Escreva `ensureBalance(signer)`.** Contrato: leia o saldo com a mesma linha `rpc.getBalance(signer.address).send()` que o solana-panel já traz, e retorne antecipadamente se ele passar de `MIN_BALANCE`. Se não, construa `airdropFactory({ rpc, rpcSubscriptions })` e tente um airdrop de `lamports(1_000_000_000n)`, um SOL de devnet, no máximo três vezes com delays de 0, 2 e 8 segundos: a forma de backoff do m02-l3, dimensionada para um faucet cujo orçamento é 2 por 8 horas. Capture falhas de forma geral e logue a mensagem; não dê match em nenhum código específico, você sabe por quê. Depois da terceira falha, lance um erro rico que diz ao operador exatamente o que fazer em seguida: iniciar o `solana-test-validator` e re-rodar com `RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900`.

5. **Monte o `main`.** Logue a URL do alvo, carregue o signer, garanta o saldo, depois um `generateKeyPairSigner()` novo como o destinatário para que a transferência visivelmente mova valor para um segundo endereço, depois o pipe exatamente como percorrido, depois imprima `confirmed: <signature>` e o link do explorer quando o alvo for a devnet. Feche o arquivo com o contrato de código de saída:

```typescript
main().catch((err) => {
  console.error("tx-check FAILED:", (err as Error).message);
  process.exit(1);
});
```

![Quatro blocos anotados mostrando o carregador do signer, o airdrop orçado com fallback, o pipe de seis passos no main, e o contrato de código de saída de que lições rio abaixo dependem.](assets/v08-annotated-code.webp)

6. **Primeira execução, contra a devnet.** `npx tsx tx-check.ts`. Duas coisas podem acontecer, e as duas são conteúdo de lição. Se o faucet cooperar você ganha a recompensa na hora: a linha confirmed, o link do explorer, saída 0. Se ele estiver seco você ganha o que eu ganhei em 2026-09-02, colado textualmente da execução de verificação deste script exato:

```text
target: https://api.devnet.solana.com
new throwaway seed written to .keys/devnet-throwaway.seed (gitignored)
balance: 0 lamports at 77Stnr644XdriXqnvnt6ZF1TeSBv2PneRS4QoTJ788vX
airdrop attempt 1 failed: JSON-RPC error: Internal JSON-RPC error (Internal error)
airdrop attempt 2 failed: HTTP error (429): Too Many Requests
airdrop attempt 3 failed: HTTP error (429): Too Many Requests
tx-check FAILED: airdrop failed after 3 attempts. Faucet may be dry (budget: 2/8h unauthenticated). Fallback: start solana-test-validator, then re-run with RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900
```

   Leia aquele log como um operador. A tentativa um e a tentativa dois falharam de formas diferentes, dentro do mesmo minuto, que é a afirmação de falha heterogênea da caixa de honestidade observada ao vivo. O script não crashou, não fez loop, não deu match em nenhum dos códigos: ele gastou as três tentativas orçadas dele, imprimiu instruções que um humano às 2 da manhã conseguiria seguir, e saiu com 1. Isso não é um passo de lab que falhou. Isso é o seu script lidando com uma dependência best-effort e orçada exatamente como projetado, e é o menor sistema de produção que você já escreveu.

7. **A execução de fallback, obrigatória para todo mundo.** Mesmo se a devnet funcionou de primeira. Num segundo terminal, `solana-test-validator`, espere uns segundos para ele subir, depois:

```bash
RPC_URL=http://127.0.0.1:8899 RPC_WS_URL=ws://127.0.0.1:8900 npx tsx tx-check.ts
```

   Forma esperada, da execução de verificação deste código exato:

```text
target: http://127.0.0.1:8899
balance: 0 lamports at 77Stnr644XdriXqnvnt6ZF1TeSBv2PneRS4QoTJ788vX
airdrop landed
confirmed: 5Qe3VztdDPtXVcyzM7egpwMkgVL3itWfCkFo7Pt4AndgTDEfXAr9tM83nNzU6J4F3dsJiPFv5ZhuKuMLP8Do2mxN
```

   O seu endereço e a sua assinatura vão diferir, a forma não. O airdrop aterrissa na hora porque o faucet é seu. E agora o fallback não é um boato: você exercitou ele, a mesma lógica que o m09-l2 vai aplicar a alarmes. Um faucet seco nunca mais consegue te bloquear de vez, e como efeito colateral você agora é dono de uma blockchain local completa para tudo que você construir em seguida.

8. **O outro alvo.** Seja qual for a rede de que você ainda não conseguiu uma confirmação, rode contra ela agora, mudando nada além das duas URLs. Uma delas ainda pode te recusar, se o faucet continuou seco dentro da janela de 8 horas dele; a trava abaixo dá conta disso com honestidade. O ponto deste passo é a troca em si: um script, duas blockchains, zero mudanças de código.

![Uma escada de sondas da estação das checagens HTTP básicas até o tx-check como a sonda mais profunda, com o script de demo do módulo dez consumindo o código de saída dele.](assets/v09-diagram.webp)

## Challenge

Solo, e ele compõe tudo que este módulo construiu. Agora o tx-check prova que uma transação confirmou. Faça ele provar que a transferência quis dizer o que disse, semanticamente. Leia os dois saldos antes da transferência e depois dela, remetente e destinatário, usando a mesma leitura de `getBalance` que o m08-l2 te ensinou. Depois afirme duas coisas: o saldo do destinatário subiu exatamente `TRANSFER_LAMPORTS`, e o saldo do remetente caiu `TRANSFER_LAMPORTS` mais uma taxa que é maior que zero. Imprima a taxa que a sua transação de fato pagou, em lamports, e formate os valores de SOL com o seu formatador BigInt do m08-l2, nunca parseFloat. Não deixe uma taxa esperada hardcoded: meça ela, imprima ela, e deixe a asserção só exigir que ela exista. Você vai reparar que o remetente perdeu um pouco mais do que mandou. Leituras eram grátis porque não mutam nada; a taxa é o outro lado dessa assimetria, o preço da escrita, paga pelo fee payer que você definiu no passo 2 do pipe. Por que as taxas são o que são, e como dar lance para aterrissar quando importa, é ofício de aterrissagem de transação, território do curso de domínio do lado do cliente quando ele for entregue.

Duas notas de implementação, os dois lugares onde eu espero que as primeiras tentativas balancem. A ordem importa: tire os snapshots de antes depois de o `ensureBalance` retornar, ou o crédito do airdrop vai poluir o seu delta de remetente, e tire os snapshots de depois só quando o `sendAndConfirm` tiver resolvido, já que no commitment `confirmed` as duas leituras vão então refletir a transferência. E a aritmética é bigint de ponta a ponta: os deltas, a taxa, a comparação contra `TRANSFER_LAMPORTS`, tudo em matemática com sufixo `n`, exatamente a disciplina que o formatador do m08-l2 foi construído para proteger. Aceitação: o script ainda sai com 0 numa transferência confirmada e agora semanticamente verificada, sai com 1 se os deltas alguma vez discordarem do valor, e imprime a taxa medida tanto crua quanto formatada.

## Checkpoint

Trave no fazer, uma colagem de terminal: a linha `confirmed: <signature>`, da devnet se o faucet te deixou entrar nesta janela, do seu validador local caso contrário, mais o link do explorer se foi a devnet. Depois a evidência de que a troca funciona: o mesmo script passando contra o outro alvo com só as duas URLs mudadas, admitindo que um faucet seco pode manter a execução de devnet pendente até a sua janela de 8 horas resetar. E a prova negativa: `git status` não mostrando nenhum arquivo de chave perto do staging, porque `.keys/` foi ignorado antes de a seed existir. A execução do validador local não é evidência opcional; todo mundo tem um a esta altura.

Uma vitória de 30 segundos que levou oito módulos para merecer, então aproveite a batida. Todo módulo deste curso está de pé dentro daquela assinatura: o TypeScript e o ferramental do M1 ao M3, a disciplina de backoff do M2 embrulhada em volta de um faucet, o fio de ops que entregou as superfícies que agora leem a blockchain, e uma mensagem assinada que uma rede de verdade concordou em executar. Sessenta e quatro bytes de prova de que a sua stack consegue agir.

Se os modos de falha do faucet te surpreenderam em algum quarto jeito novo que esta lição não listou, isso é sinal genuinamente útil: largue o texto exato do erro no feedback do curso. A caixa de honestidade do airdrop é construída a partir de falhas observadas, e ela cresce do mesmo jeito que o seu ensaio de fallback cresceu, com alguém batendo na parede primeiro e escrevendo isso.

A estação agora lê de toda superfície e provou que consegue escrever. Isso faz dela um sistema de verdade, o que quer dizer que ela pode falhar de verdade. No próximo módulo você começa a rodar ela como alguém que já foi acionado: auditando a árvore de dependências em que ela se apoia, depois fazendo a fiação de logs e alarmes para que até a morte do próprio monitor seja alta.
