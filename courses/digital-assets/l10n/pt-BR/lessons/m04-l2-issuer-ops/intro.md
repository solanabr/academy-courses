# Ops do emissor: auditor, registro e supply confidencial

## Resumo

No m04-l1 você construiu o modelo mental, os compromissos de envelope selado, as três provas, a auditor key opcional, mas não escreveu código. Aqui é onde você configura isso: a primeira coisa confidencial que você de fato constrói. Você vai criar uma variante confidencial do SPROUT carregando ConfidentialTransferMint com uma chave ElGamal de auditor de verdade e uma política de aprovação manual, tudo a partir de instruções brutas de `@solana-program/token-2022`; você vai conhecer o registro ElGamal que provisiona contas sem uma assinatura de dono por conta; você vai rodar um depósito e uma transferência confidencial na devnet e ver isso se espalhar por várias transações dependentes; e você vai ramificar para o ConfidentialMintBurn, a extensão que torna o próprio supply confidencial. O recuo: a configuração do mint, a geração de chaves e o fluxo de depósito são trabalhados de ponta a ponta; os parâmetros de auditor e de auto-approve mais o passo do ConfigureAccountWithRegistry são problemas de completion que você preenche (o do registro como prática de forma, que não pode executar sem uma conta de registro do lado Rust, um limite que o lab declara abertamente); a ramificação de supply confidencial e a prova dela são suas, solo, com um caminho de degradação projetado caso o gate de provas do seu cluster esteja desligado. Esse caminho de degradação é parte do plano em vez de um pedido de desculpas, e você vai ouvir isso dito em voz alta antes de precisar dele.

Antes de qualquer coisa disso, prove que a maquinaria em que você vai se apoiar está realmente implantada. As provas da lição passada não se verificam sozinhas; um programa nativo dedicado faz isso. Trinta segundos, nenhum setup além do `curl`:

```bash
curl -s https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '
  {"jsonrpc":"2.0","id":1,"method":"getAccountInfo",
   "params":["ZkE1Gama1Proof11111111111111111111111111111",{"encoding":"base64"}]}' \
  | python3 -c "import sys,json; v=json.load(sys.stdin)['result']['value']; print('executable:', v['executable'], '| owner:', v['owner'])"
```

Você deve ver `executable: True | owner: NativeLoader1111111111111111111111111111111`. Eu rodei isso contra a devnet e a mainnet hoje de manhã (2026-08-22) e as duas responderam a mesma coisa: o ZK ElGamal Proof Program está presente e executável nos dois clusters. Presença não é a história inteira, e a gente vai chegar nos feature gates que ainda podem desligar a verificação, mas você acabou de confirmar que o verificador existe onde você está a ponto de fazer deploy. Isso é mais due diligence do que a maioria dos tutoriais de transferência confidencial jamais faz.

Seu CFO quer salários confidenciais. Os auditores ainda precisam reconciliar cada centavo. O regulador quer uma janela que mais ninguém tem. "Confidencial" e "auditável" soam como opostos, e o modelo da lição passada já te disse que não são: uma chave ElGamal opcional no mint, e toda transferência encripta silenciosamente uma segunda cópia do valor dela só para aquele auditor. Hoje você define essa chave com as suas próprias mãos.

## O que um emissor de fato configura

Tudo nesta lição pende de uma struct cuja forma você já viu. Quando um mint Token-2022 (programa `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`, o mesmo endereço de todo mint que você construiu) carrega a extensão ConfidentialTransferMint, a entrada de TLV dele guarda exatamente três campos: uma `authority` que pode atualizar essa configuração e aprovar contas, uma flag `auto_approve_new_accounts` e uma `auditor_elgamal_pubkey` opcional. Três campos. A superfície inteira de emissor das transferências confidenciais são essas três decisões, mais uma segunda extensão para o supply. Vamos pegar elas na ordem em que vão te morder.

### A auditor key: visibilidade direcionada, no mint inteiro, para sempre

O auditor é uma única pubkey ElGamal global e opcional no mint. Defina ela, e toda transferência confidencial daquele token tem que encriptar o valor dela também sob a chave do auditor. Lembre da prova do meio da lição passada, a prova de validade de ciphertext agrupado: ela cobre três handles, remetente, destinatário e auditor, e o programa não vai aceitar uma transferência cujo ciphertext de auditor esteja faltando ou mal formado quando um auditor está configurado. Não existe opt-out por transferência. Não existe uma flag "essa aqui é sensível, pula o auditor". O mint decide, e o mint decide para todo mundo, em toda transferência, até a autoridade mudar a configuração.

![Uma transferência confidencial produz ciphertexts do valor sob as chaves de remetente, destinatário e auditor, então o auditor consegue decriptar toda transferência enquanto o público lê só ciphertext.](assets/v01-diagram.webp)

Por que uma chave global em vez de consentimento por transferência? Raciocine a partir do modo de falha. Se a divulgação fosse por transferência, quem estivesse tentando esconder alguma coisa simplesmente não divulgaria, e um auditor que só vê as transferências que as pessoas escolheram mostrar não é um auditor, é uma plateia. Reconciliação só significa alguma coisa quando a cobertura é total, então o design coloca a decisão onde a cobertura é total: no mint, na hora da configuração, imposta pela mesma maquinaria de provas que impõe todo o resto. O custo é igualmente estrutural. Você converteu "ninguém consegue ler valores" em "ninguém consegue ler valores exceto quem detém uma chave secreta específica," e essa chave agora é o segredo mais valioso da sua pilha de compliance. Rotacione a configuração e os ciphertexts antigos não se re-encriptam; quem tinha a chave antiga consegue decriptar o histórico para sempre. Essa é a troca no coração da lição: a visibilidade do regulador é comprada com um asterisco permanente, válido para o mint inteiro, na promessa de privacidade, e a atitude honesta é escrever esse asterisco na sua documentação em vez de esperar que ninguém pergunte.

Um teste de realidade antes de você se apegar à história do CFO. Não existem usuários de produção nomeados de transferências confidenciais hoje. O PYUSD, o deployment Token-2022 mais institucional da chain, já vem com a extensão confidentialTransferMint no mint dele agora mesmo, e on-chain ela está configurada e dormente: o slot confidencial presente e sem uso, a stablecoin regulada emblemática segurando a porta aberta sem atravessar por ela. Então ensine isso a você mesmo do jeito que eu estou ensinando a você: capaz para emissores, não comprovado por emissores. Você está aprendendo as ops porque os trilhos estão vivos e a porta está aberta, não porque uma dúzia de tesourarias já atravessou por ela.

### auto_approve_new_accounts: a flag do leão de chácara

O segundo campo é um booleano com um departamento de compliance dentro dele. Quando `auto_approve_new_accounts` é true, qualquer holder que configure a conta dele para transferências confidenciais pode começar a usar na hora. Quando é false, toda conta recém-configurada fica não aprovada, e todas as operações confidenciais nela falham até a autoridade de transferência confidencial do mint aprovar explicitamente aquela conta. False é a forma KYC: ninguém move valores escondidos até o emissor dar o aval naquela conta específica. Nossa variante do SPROUT define ela como false, em parte porque essa é a postura de emissor que esta lição ensina, e em parte porque isso te força a construir o passo de aprovação você mesmo, o que acaba sendo mais interessante do que parece. A CLI que você vai instalar no lab não tem comando nenhum de confidential-approve. A instrução bruta existe, o ferramental não acompanhou, e você vai fazer a ponte nessa lacuna com umas quarenta linhas de TypeScript.

### O registro: assine uma vez, provisione para sempre

Agora o problema de provisionamento. Configurar uma conta para transferências confidenciais normalmente exige que o dono da conta assine, porque a configuração inclui a pubkey ElGamal do dono e uma prova de que ele a controla. Para um hacker configurando a própria carteira, beleza. Para um emissor provisionando dez mil contas de funcionários, uma assinatura de dono por conta é um pesadelo de ops: todo lote de provisionamento precisa de todo dono online, assinando, em ordem.

O programa de registro ElGamal existe para quebrar essa dependência. Ele já vem no mesmo repositório do Token-2022, e o fluxo é: um dono registra a pubkey ElGamal dele em uma conta de registro uma vez, com uma prova de validade, assinando uma vez. Daí em diante, qualquer um, um backend, um cron job, o serviço de provisionamento do emissor, pode chamar a instrução `ConfigureAccountWithRegistry` no Token-2022, apontando para a conta de registro em vez de coletar uma assinatura nova do dono. A conta de registro é o consentimento permanente; o programa de token lê a chave dela e configura a conta. Uma assinatura amortizada em toda ação de provisionamento futura.

![Fluxograma contrastando o caminho de assinatura de dono por conta com o caminho do registro, em que um único registro habilita provisionamento sem assinatura via ConfigureAccountWithRegistry, os dois convergindo para a aprovação manual.](assets/v02-flowchart.webp)

As letras miúdas, porque elas decidem o que você consegue construir hoje: criar a própria entrada de registro exige uma prova de validade de pubkey, e essa prova você consegue gerar em JavaScript puro — o `@solana/zk-sdk`, um peer declarado do próprio cliente de token que esta lição fixa, vai te entregar `new PubkeyValidityProofData(new ElGamalKeypair())` sem uma linha de Rust. O que falta na stack JS nos nossos pins é a outra metade: um builder de cliente para as instruções de create/update do próprio programa de registro. O lado Token-2022 do fluxo, a instrução `ConfigureAccountWithRegistry` que consome uma conta de registro existente, tem um builder de primeira classe no cliente JS, e no lab você preenche essa chamada de builder para a forma morar nos seus dedos; fica avisado agora que ela continua em dique seco ali, porque sem uma conta de registro criada ela não pode executar, e o lab diz isso em vez de fingir. O lado de criação do registro você deveria conhecer pelo nome, `spl-elgamal-registry` no repositório do token; até chegar um builder JS para as instruções dele, a rota pragmática é a CLI em Rust dele — uma lacuna de ferramental de cliente, não uma lacuna criptográfica. Essa assimetria, em que a camada de instrução está completa e o ferramental de cliente cobre ela de forma desigual, é a textura recorrente das transferências confidenciais, e é exatamente por isso que esta lição mantém um helper em Rust no bolso de trás.

### A realidade de múltiplas transações

Você derivou essa coreografia por inteiro na lição passada; aqui ela vai em um fôlego, porque hoje você relê ela da cadeira do emissor, onde ela se transforma em linhas de orçamento. Uma transferência confidencial hoje é uma pequena coreografia. Cada uma das três provas da lição passada é verificada pelo ZK ElGamal Proof Program que você sondou na abertura, e as provas são grandes demais para pegar carona em uma única transação junto com a transferência em si. Então o fluxo fica: criar uma conta de contexto para uma prova, verificar a prova dentro dela, repetir por prova, depois executar a instrução de transferência que lê esses contextos verificados, depois fechar as contas de contexto para recuperar o rent delas. Várias transações dependentes, ordenadas, cada uma capaz de falhar de forma independente.

![Linha do tempo de uma transferência confidencial em que várias transações criam contas de contexto e verificam cada prova antes de a transferência executar e os contextos fecharem para recuperar rent.](assets/v03-timeline.webp)

Duas linhas de orçamento caem direto dessa foto. Primeiro, contas de contexto são rent que você adianta e recupera, por prova, por transferência; em escala de frota essa rotatividade é uma linha de custo de verdade e o seu runbook de ops deveria tratar o fechamento de contextos como higiene obrigatória, não como limpeza. Segundo, atualidade: o futuro de transação única é real mas não é aqui, e "aqui" agora depende de qual cluster você quer dizer. Esse é o mesmo gate `enable_tx_v1` cujo endereço a lição passada te entregou, publicado no Agave 4.2 com um envelope grande o suficiente para carregar provas inline. Em 2026-09-06 ele não tinha conta nenhuma atrás dele na mainnet e estava ativo na devnet desde o slot 492,480,000. Construa para o fluxo de múltiplas transações, porque a mainnet é onde os seus usuários estão, e re-sonde o gate você mesmo na semana em que você for para produção — um `solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url mainnet-beta` resolve.

### Supply confidencial: a ramificação do ConfidentialMintBurn

Tudo até aqui esconde valores de transferência. O supply continua público: qualquer um consegue ler quantos tokens existem, e toda cunhagem e toda queima movem esse número público. Para a maioria dos tokens isso é ok e até desejável. Para um emissor cujos eventos de cunhagem são eles mesmos sensíveis, pense em uma tesouraria que não quer volumes de resgate legíveis em tempo real, existe uma segunda extensão: ConfidentialMintBurn.

Ele acrescenta quatro campos ao mint: uma `supply_elgamal_pubkey` que encripta o supply, um supply decriptável que o emissor consegue ler de volta com uma chave AES, o próprio supply confidencial encriptado, e um acumulador de queima pendente. Queimas aterrissam em forma pendente e uma instrução `apply_pending_burn` incorpora elas ao supply confidencial; a chave de supply pode ser rotacionada com `rotate_supply_elgamal_pubkey` quando a custódia daquele segredo passa de mão. E ele compõe sob regras que você já domina. O seu validador check-combo do m01-l4 codifica a regra 3: ConfidentialMintBurn exige ConfidentialTransferMint no mesmo mint. Ele também codifica a regra 5, a estranha: NonTransferable mais ConfidentialTransferMint é inválido a não ser que ConfidentialMintBurn também esteja presente. Um token confidencial soulbound só faz sentido se as operações de supply forem confidenciais também, e o programa recusa o meio-termo.

![Dois diagramas de mint mostrando o SPROUT confidencial principal com um auditor configurado e o mint da ramificação solo acrescentando os quatro campos de supply do ConfidentialMintBurn, anotados com as regras de combo e o aviso do PermanentDelegate.](assets/v04-diagram.webp)

Uma cilada para desarmar antes de você projetar qualquer coisa nessa ramificação. No m02-l2 você provou que o PermanentDelegate passa por baixo do CpiGuard e consegue varrer qualquer conta do mint dele. Saldos confidenciais são onde esse poder para: o delegado permanente não funciona em saldos confidenciais, ponto final. Um emissor que planejava reaver-e-queimar via delegado não tem equivalente confidencial; o controle de supply no mundo confidencial passa pelas instruções do próprio ConfidentialMintBurn, assinadas pela autoridade de mint, ou não acontece. Não recorra ao delegado como alavanca de supply confidencial. Ele não é uma.

### Onde isso roda, e as cicatrizes do programa

Plano de cluster, projetado desde o início em vez de adiado. Alvo primário: devnet, onde o gate de provas está ativo, então os seus depósitos e transferências devem verificar. Isso não é uma afirmação de documentação que eu estou repassando; eu li as três contas de feature do zk-ElGamal nos dois clusters em 2026-08-22 e cada uma delas está ativada, devnet e mainnet igualmente. Você vai ver como em um instante. Secundário: um fork de surfpool, que você usa desde o m01 e que é ideal para a metade de configuração do lab. E a ramificação honesta: se o gate de provas do zk-ElGamal estiver desligado em qualquer cluster que você tenha como alvo, as suas provas não vão verificar, e o lab degrada para configure-a-extensão mais prove-o-estado-dela-on-chain. Mesmo mint, mesmo auditor, mesmo script de verificação; a única coisa que você perde é a transferência ao vivo, e você diz isso em vez de fingir. Essa é a jogada projetada. O que você nunca faz é forçar um gate a ficar ligado em um fork local e apresentar o resultado como evidência de nível mainnet.

Por que tanta cerimônia em volta de um feature gate? Porque esse programa em particular tem cicatrizes, e diferente da maioria das cicatrizes essas têm timestamps que você mesmo pode ler. O conjunto de features do Agave carrega três gates cujos nomes contam a história inteira: `zk_elgamal_proof_program_enabled`, `disable_zk_elgamal_proof_program`, `reenable_zk_elgamal_proof_program`. Cada gate é uma conta; um ativado guarda o slot em que disparou, e o `getBlockTime` transforma esse slot em uma data. Os comandos, para que "sondar o gate" nunca seja conversa mole: `solana feature status --url mainnet-beta` sem argumento nenhum lista todo gate que a CLI conhece, endereço, status e slot de ativação incluídos, então `solana feature status --url mainnet-beta | grep -i elgamal` imprime as três linhas em uma linha de shell; pegue um slot dessa saída e `solana block-time <slot>` data ele. Troque por `--url devnet` para fazer a mesma pergunta ao outro cluster. Esse par de comandos é a sondagem inteira, e é o que o caminho de degradação no lab quer dizer com "probe the gate accounts". Faça isso na mainnet e o arco é cru: habilitado em 2025-01-23, pausado em 2025-06-19 por um conserto de segurança, reabilitado em 2026-06-04. O verificador em que esta lição inteira se apoia ficou no escuro por uns onze meses. Isso não é um subsistema de demo que nunca foi tocado sob fogo. É um programa de produção com um histórico real de incidentes, e a maquinaria de pausa é estrutural o suficiente para ter sido usada durante a maior parte de um ano. Respeite isso na sua arquitetura: qualquer sistema que você construa sobre transferências confidenciais deveria tolerar o programa de provas sendo pausado de novo, o que é um argumento a mais para manter o seu caminho de degradação ensaiado em vez de teórico.

![Linha do tempo datada dos três feature gates do ZK ElGamal mostrando a ativação de janeiro de 2025, a pausa de segurança de junho de 2025 e a reabilitação de junho de 2026, com uma sondagem recente confirmando o verificador vivo.](assets/v05-timeline.webp)

Essa é a superfície de teoria: três campos em uma extensão, uma flag de leão de chácara, um registro que amortiza consentimento, uma transferência que é uma coreografia, uma ramificação de supply com as próprias chaves dela, e um verificador com um histórico. Hora de ligar tudo isso.

## Lab: ligue o auditor, o registro e a ramificação de supply

O artefato é o `confidential-sprout`: uma variante do SPROUT cujo mint carrega ConfidentialTransferMint com uma pubkey ElGamal de auditor configurada e `autoApproveNewAccounts: false`, um par de chaves de encriptação registrado, um depósito confidencial aplicado, e uma transferência confidencial completada na devnet, ou a prova de degradação declarada caso o gate do seu cluster esteja desligado. Uma segunda ramificação habilita o ConfidentialMintBurn. A cancela no fim é a mesma que o curso sempre usa: `npx tsx verify-confidential.ts` tem que ler a extensão e o auditor de volta da chain.

1. Faça o scaffold de `labs/m04-l2` e instale a toolchain JS. Mesma lógica de pin do m02-l1, re-verificada hoje: fixe o major do kit contra o qual os seus clientes fazem peer, e o `@solana-program/token-2022@0.15.0` é o minor atual que faz peer com o kit ^7.0.0 (o 0.16.0 pulou para ^8), com o `@solana-program/system@0.13.0` casando com ele. Eu reinstalei e passei o type-check nesse trio exato em 2026-09-05; rode `npm view @solana-program/token-2022 peerDependencies` você mesmo no dia em que fizer o scaffold, porque essa matriz se move todo mês.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.23.12 typescript@5.9.3
```

2. Mais duas ferramentas, as duas do mundo Rust. A CLI `spl-token` com que você sondou pela primeira vez no m01-l4; como aquela lição avisou, ela já vem junto com algumas instalações do Agave (o one-liner do m01-l3, `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`) e com outras não, e `cargo install spl-token-cli` te dá o mesmo binário de forma isolada. Confira a versão, porque a superfície confidencial mudou de release para release (o build de release do Agave instala a CLI sem pin, então a sua cópia empacotada é o que estava atual no dia em que aquele release foi cortado):

```bash
spl-token --version   # spl-token-cli 5.6.1, the crates.io release as of 2026-08-22
```

   Agora leia o que a 5.6.1 de fato cobre, porque eu passei pela tabela de subcomandos dela linha por linha e o mapa de cobertura é a coisa mais instrutiva deste lab. A CLI dá conta por completo do fluxo do holder, o mais carregado de provas: `configure-confidential-transfer-account`, `deposit-confidential-tokens`, `apply-pending-balance`, `withdraw-confidential-tokens` e `transfer --confidential` geram todas as provas deles internamente. O fluxo do emissor é outra história. Não existe comando de confidential-approve para a política manual (`spl-token approve` é o comando comum de delegação e não tem nada a ver com isso). Não existe comando de registro. Não existe suporte a ConfidentialMintBurn em lugar nenhum, nem no `create-token` e nem na exibição da conta, onde o fonte ainda carrega uma nota para adicionar isso depois. O conjunto de instruções brutas está à frente da CLI emblemática, e essa lacuna não é um inconveniente, é a razão pela qual esta lição ensina instruções brutas. Um emissor que só consegue fazer o que a CLI faz não consegue rodar um mint de aprovação manual hoje.

![Comparação mostrando que o spl-token-cli 5.6.1 cobre o fluxo do holder (configure, deposit, apply, withdraw, transferência confidencial) enquanto approve, provisionamento de registro e todas as operações de ConfidentialMintBurn existem só como instruções brutas.](assets/v06-comparison.webp)

   A segunda ferramenta gera chaves de encriptação. Crie um helper minúsculo em Rust ao lado da pasta do seu lab (o rustup instala o cargo se você nunca: `curl https://sh.rustup.rs -sSf | sh`):

```bash
cargo new ct-keygen && cd ct-keygen
cargo add solana-zk-sdk@7.0.1 bs58@0.5.1 base64@0.23.1
# Pins are what I ran on 2026-08-22. If cargo add refuses one (yanked, or the
# line moved), drop that crate's pin and take the current release; nothing in
# this helper depends on an exact version.
```

   Depois o `src/main.rs`. Esse é o programa inteiro, e a contabilidade honesta de por que ele existe: nada aqui exige Rust estritamente — o `@solana/zk-sdk` vai cunhar o mesmo par de chaves ElGamal e o mesmo zero encriptado em AES com `AeKey` em JS puro, e a re-codificação é um código de duas linhas — mas os valores alimentam tanto a CLI (cujo texto de ajuda admite que ela só aceita base64 hoje, "more methods in a future version") quanto o cliente do kit (que quer base58), e um binário pequeno que imprime toda chave em toda codificação, construído sobre o mesmo crate `solana-zk-sdk` em que o programa on-chain confia, é a ferramenta em forma de ops para isso. Prefere TypeScript? Porte contra o `@solana/zk-sdk` e mantenha o formato de impressão:

```rust
// ct-keygen: derive the encryption keys issuer ops needs, print every encoding.
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use solana_zk_sdk::encryption::{auth_encryption::AeKey, elgamal::ElGamalKeypair};

fn main() {
    // 1. An ElGamal keypair (auditor key, or supply key for ConfidentialMintBurn).
    let elgamal = ElGamalKeypair::new_rand();
    let pubkey_bytes: [u8; 32] = (*elgamal.pubkey()).into();

    println!("elgamal pubkey (base64, for spl-token): {}", B64.encode(pubkey_bytes));
    println!("elgamal pubkey (base58, for kit code):  {}", bs58::encode(pubkey_bytes).into_string());

    // 2. An AES key + the encryption of zero (the initial decryptable supply).
    let aes = AeKey::new_rand();
    let zero_bytes = aes.encrypt(0).to_bytes();
    let listed: Vec<String> = zero_bytes.iter().map(|b| b.to_string()).collect();
    println!("decryptableSupply(0) bytes: [{}]", listed.join(", "));
}
```

   O `cargo run` imprime três linhas. Exporte a pubkey em base58 como `AUDITOR_ELGAMAL_PUBKEY` para o próximo passo, e guarde as outras duas linhas para a ramificação de supply. Uma nota de honestidade sobre lidar com chaves: `new_rand` é o certo para um lab. Um auditor de produção deriva o par de chaves ElGamal dele deterministicamente a partir de uma assinatura de carteira para que ele possa ser recuperado, e a chave secreta aqui é a joia da coroa que a gente discutiu; trate a impressão de acordo e jogue essas chaves de lab fora depois.

3. Agora o mint. Crie o `create-confidential-sprout.ts`, e dê ao problema de completion o que ele merece: digite o arquivo com os três valores de configuração em branco, decida eles você mesmo, depois confira contra a referência preenchida abaixo. As três linhas que são suas são exatamente os três campos da seção de teoria: a autoridade, a política de aprovação e o auditor. Todo o resto é a mesma dança de cria-a-conta-depois-inicializa que você roda desde o m02-l1, com a extensão inicializada antes do `InitializeMint`, como sempre.

```ts
// create-confidential-sprout.ts: a SPROUT variant that carries ConfidentialTransferMint.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  createKeyPairSignerFromBytes,
  sendAndConfirmTransactionFactory,
  address,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
  getSignatureFromTransaction,
  type Instruction,
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  extension,
  getMintSize,
  getInitializeConfidentialTransferMintInstruction,
  getInitializeMintInstruction,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");
const rpcSubscriptions = createSolanaRpcSubscriptions(
  process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com"
);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

// The auditor's ElGamal pubkey, printed by ct-keygen in base58 form.
const AUDITOR_ELGAMAL_PUBKEY = address(process.env.AUDITOR_ELGAMAL_PUBKEY!);

async function main() {
  const payer = await createKeyPairSignerFromBytes(
    new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, "utf8")))
  );
  const mint = await generateKeyPairSigner();

  // Size the account for the extension BEFORE the base mint layout.
  const confidentialTransferMint = extension("ConfidentialTransferMint", {
    authority: payer.address,            // fill 1: who updates config + approves accounts
    autoApproveNewAccounts: false,       // fill 2: manual policy, the KYC shape
    auditorElgamalPubkey: AUDITOR_ELGAMAL_PUBKEY, // fill 3: the regulator's window
  });
  const space = BigInt(getMintSize([confidentialTransferMint]));
  const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

  const instructions: Instruction[] = [
    getCreateAccountInstruction({
      payer,
      newAccount: mint,
      lamports: rent,
      space,
      programAddress: TOKEN_2022_PROGRAM_ADDRESS,
    }),
    // Extension init runs BEFORE InitializeMint, same order as every mint you have built.
    getInitializeConfidentialTransferMintInstruction({
      mint: mint.address,
      authority: payer.address,
      autoApproveNewAccounts: false,
      auditorElgamalPubkey: AUDITOR_ELGAMAL_PUBKEY,
    }),
    getInitializeMintInstruction({
      mint: mint.address,
      decimals: 6,
      mintAuthority: payer.address,
      freezeAuthority: payer.address,
    }),
  ];

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const tx = await pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstructions(instructions, m),
    (m) => signTransactionMessageWithSigners(m)
  );
  assertIsTransactionWithBlockhashLifetime(tx);
  await sendAndConfirm(tx, { commitment: "confirmed" });

  console.log(`confidential SPROUT mint: ${mint.address}`);
  console.log(`signature: ${getSignatureFromTransaction(tx)}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   `AUDITOR_ELGAMAL_PUBKEY=<base58 from ct-keygen> npx tsx create-confidential-sprout.ts` e salve o endereço de mint impresso; todo passo posterior recebe ele como argumento. Lembre da restrição que você não consegue ver neste arquivo: a configuração de transferência confidencial é só na criação. Não existe instrução de retrofit. Um SPROUT vivo com holders nunca pode ganhar essa extensão, e é por isso que este é um mint variante e por isso que a decisão pertence ao seu checklist de lançamento, não ao seu backlog.

4. Prove que aterrissou. Crie o `verify-confidential.ts`, a cancela desta lição, e a interface pela qual uma lição posterior vai chamar este artefato:

```ts
// verify-confidential.ts: prove the extension and the auditor read back from chain.
import { createSolanaRpc, address } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");

async function main() {
  const mintAddress = address(process.argv[2] ?? process.env.MINT!);
  const mint = await fetchMint(rpc, mintAddress);

  const extensions =
    mint.data.extensions.__option === "Some" ? mint.data.extensions.value : [];
  const ct = extensions.find((e) => e.__kind === "ConfidentialTransferMint");
  if (!ct || ct.__kind !== "ConfidentialTransferMint") {
    console.error("FAIL: ConfidentialTransferMint not present on this mint");
    process.exit(1);
  }

  const auditor =
    ct.auditorElgamalPubkey.__option === "Some"
      ? ct.auditorElgamalPubkey.value
      : "none";
  console.log("ConfidentialTransferMint present");
  console.log(`auditorElgamalPubkey=${auditor}`);
  console.log(`autoApproveNewAccounts=${ct.autoApproveNewAccounts}`);

  const mintBurn = extensions.find((e) => e.__kind === "ConfidentialMintBurn");
  if (mintBurn && mintBurn.__kind === "ConfidentialMintBurn") {
    console.log(`ConfidentialMintBurn present; supplyElgamalPubkey=${mintBurn.supplyElgamalPubkey}`);
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   Rode `npx tsx verify-confidential.ts <mint>`. A condição de aprovação é exata: `ConfidentialTransferMint present`, a linha do auditor ecoando a chave em base58 que você gerou, e `autoApproveNewAccounts=false`. Se o auditor imprimir `none`, o seu fill 3 não chegou até a instrução; recrie o mint, porque não existe editar para sair de um erro de hora-da-criação em um descartável, e notar esse reflexo importa mais do que o SOL que você gasta nisso.

![Saída esperada anotada do script de verificação: extensão presente, pubkey do auditor batendo com a chave gerada, auto-approve false, e uma quarta linha opcional para a ramificação do ConfidentialMintBurn.](assets/v07-annotated-code.webp)

5. Provisione e aprove uma conta. A metade do registro primeiro, já que ela é o problema de completion: crie o `configure-with-registry.ts` e escreva a chamada do builder você mesmo, a partir da descrição da seção de teoria, antes de continuar lendo. A chamada preenchida são três endereços e um payer, e o ponto é o que não está nela, nenhum signer de dono em lugar nenhum:

```ts
import { getConfigureConfidentialTransferAccountWithRegistryInstruction } from "@solana-program/token-2022";

const ix = getConfigureConfidentialTransferAccountWithRegistryInstruction({
  token: tokenAccount,          // the ATA to configure
  mint: mintAddress,            // your confidential SPROUT
  elgamalRegistry: registryAccount, // the standing consent, created once via spl-elgamal-registry
  payer,                        // funds the account reallocation; NOT the owner
});
```

   Ligue isso no mesmo esqueleto de pipe-e-send de toda transação neste curso. Ela executa contra qualquer conta de registro que exista; criar uma é a tarefa do lado Rust nomeada na seção de teoria, então para a conta de hoje a gente pega o caminho de assinatura do dono que a CLI automatiza, e o seu design de provisionamento de frota mantém o registro no bolso. Configure a sua própria ATA com a CLI (ela cria as chaves ElGamal da conta a partir da sua carteira e gera a prova de validade internamente):

```bash
spl-token create-account <MINT> --url devnet
spl-token configure-confidential-transfer-account <MINT> --url devnet
```

   Configurado não é usável: você definiu a política como manual, então esta conta agora fica não aprovada, e a CLI não tem comando de confidential-approve. Faça você mesmo a ponte nessa lacuna com o `approve-account.ts`, o lado do emissor da flag do leão de chácara:

```ts
// approve-account.ts: the manual-approval half of autoApproveNewAccounts=false.
import {
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createKeyPairSignerFromBytes,
  sendAndConfirmTransactionFactory,
  address,
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  signTransactionMessageWithSigners,
  assertIsTransactionWithBlockhashLifetime,
} from "@solana/kit";
import {
  TOKEN_2022_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getApproveConfidentialTransferAccountInstruction,
} from "@solana-program/token-2022";
import { readFileSync } from "node:fs";
import { homedir } from "node:os";

const rpc = createSolanaRpc(process.env.RPC_URL ?? "https://api.devnet.solana.com");
const rpcSubscriptions = createSolanaRpcSubscriptions(
  process.env.RPC_WS_URL ?? "wss://api.devnet.solana.com"
);
const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });

async function main() {
  const authority = await createKeyPairSignerFromBytes(
    new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, "utf8")))
  );
  const mint = address(process.argv[2]!);
  const owner = address(process.argv[3]!);
  const [token] = await findAssociatedTokenPda({
    mint,
    owner,
    tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
  });

  const ix = getApproveConfidentialTransferAccountInstruction({ token, mint, authority });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const tx = await pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(authority, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstruction(ix, m),
    (m) => signTransactionMessageWithSigners(m)
  );
  assertIsTransactionWithBlockhashLifetime(tx);
  await sendAndConfirm(tx, { commitment: "confirmed" });
  console.log(`approved ${token} for confidential transfers on ${mint}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
```

   `npx tsx approve-account.ts <MINT> $(solana address)` e a sua conta cruza de configurada para usável. Note que a instrução não precisa de prova nenhuma, só da assinatura comum da autoridade; aprovação é um ato de política, não um ato criptográfico, o que é exatamente por que foi barato para a gente construir e um pouco vergonhoso que nenhum ferramental já venha com ela.

6. Mova dinheiro escondido. O fluxo de depósito-e-transferência é o script fornecido, `confidential-flow.sh`; ele monta um destinatário e conduz a conta do destinatário pelo configure-e-aprova, depois roda o depósito, o apply de saldo pendente, e uma transferência confidencial. A sua própria conta de remetente ele não toca: o passo 5 configurou e aprovou ela, e o script assume que esse trabalho está feito, então rode o passo 5 primeiro ou a transferência abaixo falha por um motivo que não tem nada a ver com o gate de provas. O caminho de degradação está escrito dentro dele, em voz alta, no único passo que pode bater no gate:

```bash
#!/usr/bin/env bash
# confidential-flow.sh <MINT>: deposit + one confidential transfer, degrade path included.
set -euo pipefail
MINT=$1
URL=${RPC_URL:-devnet}

# A recipient wallet, funded enough for rent.
solana-keygen new --no-bip39-passphrase --silent --outfile recipient.json
RECIPIENT=$(solana-keygen pubkey recipient.json)
solana transfer "$RECIPIENT" 0.1 --allow-unfunded-recipient --url "$URL"

# Recipient's account: create, configure, approve (our raw-instruction bridge).
spl-token create-account "$MINT" --owner "$RECIPIENT" \
  --fee-payer ~/.config/solana/id.json --url "$URL"
spl-token configure-confidential-transfer-account "$MINT" --owner recipient.json \
  --fee-payer ~/.config/solana/id.json --url "$URL"
npx tsx approve-account.ts "$MINT" "$RECIPIENT"

# Fund the public side, then move 40 SPROUT behind the curtain.
spl-token mint "$MINT" 100 --url "$URL"
spl-token deposit-confidential-tokens "$MINT" 40 --url "$URL"
spl-token apply-pending-balance "$MINT" --url "$URL"

# The confidential transfer: several dependent transactions under the hood.
# Prerequisite: YOUR sender account was configured + approved in step 5 and
# funded by the deposit above; the script does not repeat that work.
if spl-token transfer "$MINT" 15 "$RECIPIENT" --confidential --url "$URL"; then
  echo "confidential transfer complete: amount hidden, auditor copy included"
else
  echo "DEGRADE PATH: the confidential transfer failed."
  echo "First rule out your own state: sender configured + approved (step 5),"
  echo "deposit landed, apply-pending-balance run. Only if all of that holds is"
  echo "the likely cause the cluster's zk-ElGamal gate; probe the gate accounts"
  echo "before blaming the cluster:"
  echo "  solana feature status --url $URL | grep -i elgamal"
  echo "Falling back to the designed proof: extension + auditor state, on-chain."
  npx tsx verify-confidential.ts "$MINT"
fi
```

   Rode `./confidential-flow.sh <MINT>` (depois de `chmod +x confidential-flow.sh`) e observe o passo de transferência: a CLI está fazendo quietinha a coreografia inteira de contas de contexto da seção de teoria, criando elas, verificando três provas, transferindo, fechando. Na devnet isso deve completar. Se em vez disso você aterrissar na ramificação de degradação, você não perdeu nada do que esta lição avalia em você: a habilidade de emissor era a configuração, e o fallback prova ela on-chain, declarado abertamente. Depósito antes de transferência não é opcional, por falar nisso, e nem o `apply-pending-balance`: depósitos aterrissam em um saldo pendente e só o passo de apply torna eles gastáveis, um design de duas fases que você vai reconhecer do modelo da lição passada. Pular o apply é o ticket número um de "por que o meu saldo disponível está zero" neste fluxo.

7. Feche tudo em uma única cancela. `npx tsx verify-confidential.ts <mint>` é o teste de aceitação da lição, e daqui para frente outras lições vão assumir que um `confidential-sprout` significa exatamente o que este script afirma: extensão presente, o seu auditor, política manual. Rode ele uma última vez e leia a sua própria auditor key voltando de uma infraestrutura de nível mainnet. Duas lições atrás o auditor era um diagrama. Agora ele é um valor que você gerou, configurou e consegue provar.

## Challenge

A parte avaliada primeiro: `validateConfidentialConfig`, a checagem pré-voo que um emissor roda antes de sequer construir `initializeConfidentialTransferMint`, para que um conjunto de extensões ruim morra na revisão em vez de reverter on-chain. O passo 3 preencheu na mão uma config que por acaso era legal; esta função é o que transforma essa sorte em política. O grader chama a sua função posicionalmente, quatro escalares nesta ordem, e o tipo de retorno é esta forma de veredicto exata — `ConfidentialConfig` é o nome que o starter dá ao veredicto *externo*, e a config de três campos construída viaja dentro dele:

```ts
type ConfidentialConfig = {
  ok: boolean;
  reason: string; // "ok" when valid, else the rejection slug
  config: {
    authority: string;
    autoApproveNewAccounts: boolean;
    auditorElGamalPubkey: string | null;
  } | null;
};

function validateConfidentialConfig(
  extensionList: string, // the mint's full extension set, pipe-separated:
  //                        'NonTransferable|ConfidentialTransferMint'
  authorityKey: string, // confidential-transfer authority ("" if unset)
  autoApprove: boolean, // auto_approve_new_accounts
  auditorKey: string | null, // auditor ElGamal pubkey, or null (optional)
): ConfidentialConfig;
```

O contrato de falha é um veredicto retornado, nunca um throw: relate uma rejeição retornando `{ ok: false, reason: "<slug>", config: null }` com o slug como a string de reason inteira, nada prefixado, nada acrescentado. O grader lê `result.ok` e `result.reason`, então "a primeira reason que você relata" significa a reason da primeira rejeição que a sua função retorna. Faça split de `extensionList` em `'|'` antes de raciocinar sobre ela; aquela string única é como o conjunto inteiro de extensões viaja pelo grader. Depois imponha as três rejeições que você já conhece desta lição, nesta ordem exata, porque mais de uma pode valer ao mesmo tempo: um `authorityKey` vazio é `missing-authority` primeiro, depois um conjunto sem ConfidentialTransferMint é `confidential-transfer-mint-not-enabled`, depois a regra 5 de combo, NonTransferable mais ConfidentialTransferMint sem ConfidentialMintBurn, é `nontransferable-confidential-requires-mintburn`. A regra 3 não precisa de código próprio: um conjunto que carrega ConfidentialMintBurn sem ConfidentialTransferMint já falha na segunda checagem, e `confidential-transfer-mint-not-enabled` é o veredicto correto dele. Um auditor nulo não é um erro, é a política de sem-auditor, e uma configuração válida retorna `ok: true` com `reason: "ok"` e a config construída: `authority` a partir de `authorityKey`, `autoApproveNewAccounts` a partir de `autoApprove`, `auditorElGamalPubkey` a partir de `auditorKey` (o nome do campo do grader capitaliza o G; o campo on-chain que você lê no lab não, uma inconsistência que o próprio ecossistema entrega e que você tem a chance de notar). O starter aprova tudo, que é precisamente a checagem pré-voo que deixa passar um mint que vai reverter.

![Fluxograma de decisão rodando as três checagens de rejeição ordenadas com os slugs de erro exatos delas antes de uma config válida cair para o objeto retornado.](assets/v08-flowchart.webp)

A ramificação solo é o supply confidencial. Pegue as linhas do ct-keygen que você guardou, a pubkey ElGamal de supply e os bytes de `decryptableSupply(0)`, e construa o `confidential-supply.ts`: uma segunda variante do SPROUT cujo mint inicializa ConfidentialTransferMint (sem auditor desta vez) e ConfidentialMintBurn na mesma transação, dimensionado com `getMintSize` sobre as duas extensões, inicializados nessa ordem, antes do `InitializeMint`. Todo builder de que você precisa está no mesmo cliente: `getInitializeConfidentialMintBurnInstruction` recebe a pubkey de supply e o zero encriptado, e a regra 3 é imposta pelo programa, então se você inicializar MintBurn sem TransferMint você vai ver a matriz de combo se defender em produção.

Depois prove alguma coisa. Em um cluster onde as provas verificam, cunhe de forma confidencial e mostre que o supply confidencial mudou: o supply decriptável do mint e o estado pendente se deslocam enquanto o campo público `supply` fica parado, e `apply_pending_burn` e `rotate_supply_elgamal_pubkey` são as duas instruções de ops que o seu runbook embrulharia em seguida. Se o gate do seu cluster estiver desligado, pegue o caminho de degradação e diga isso: estenda a quarta linha do `verify-confidential.ts` em uma asserção completa, ConfidentialMintBurn presente, a sua pubkey de supply ecoada de volta, on-chain, com a metade de prova-de-transferência explicitamente fora de alcance e nomeada como tal. A régua de aceitação é exata: o script de verificação lê ConfidentialTransferMint mais o auditor configurado e a flag de auto-approve de volta do mint on-chain, e ou um depósito confidencial e uma transferência completam na devnet ou a prova de degradação passa. O que separa um solo aprovado de um solo sortudo é a frase de write-up que você anexa: qual caminho você pegou, e por quê, em uma linha honesta.

Se qualquer passo aqui retornou alguma coisa que o meu não retornou, uma falha de prova na devnet onde eu afirmei que o gate estava ativo, um subcomando de CLI que apareceu ou desapareceu em um spl-token mais novo, uma mudança de faixa de peer que quebrou o pin 0.15.0, sinalize isso no canal de feedback do curso com o comando e o erro exatos. A stack confidencial é a superfície que se move mais rápido neste curso, as minhas sondagens são datadas de 2026-08-22, e um aprendiz que pega a devnet derivando da documentação está fazendo exatamente o trabalho de verificar-tudo que este curso fica te dizendo que vence qualquer tutorial, incluindo este.

Agora você entregou a extensão mais poderosa e menos roteável do catálogo: um token que nenhuma AMM vai cotar jamais, porque valores encriptados não podem ser cotados. Lembre da allowlist da Raydium do m02-l1, e note o que você acabou de fazer com as chances desta variante nela. Essa colisão é a pergunta de abertura do próximo módulo: qual conjunto de extensões de fato mantém o SPROUT negociável, decidido não por vibe mas por ler o código exato que as DEXes rodam. O caminho do emissor termina aqui, no mais especializado dele. O caminho da roteabilidade começa descobrindo o que ele custa.
