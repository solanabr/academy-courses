# Extensões de autoridade: quem pode tocar nos seus tokens

## Resumo

Na lição passada você construiu a economia do SPROUT a partir de instruções brutas e fez harvest das taxas retidas dela para uma tesouraria, então o valor agora flui exatamente para onde você o roteia. Esta lição faz a pergunta mais difícil: quem tem permissão para mover, congelar ou reaver esse valor, para começo de conversa. Você vai configurar cinco extensões com formato de autoridade em mints descartáveis (PermanentDelegate, Pausable, DefaultAccountState, PermissionedBurn, MintCloseAuthority) e, para cada uma delas menos a MintCloseAuthority, vai fazer assert de que a operação bloqueada realmente bloqueia; a autoridade de fechamento é decodificada em vez de exercida, porque fechar exige supply zero. Um asterisco: a PermissionedBurn é mais nova que o build de programa empacotado do surfnet, então a prova dela roda como uma simulação em mainnet mais um desvio opcional pela devnet; o passo 5 explica a emenda. Então a demonstração emblemática: um mint carregando ao mesmo tempo um PermanentDelegate e uma conta com CpiGuard habilitado, onde você prova o que a trava bloqueia e o que o delegado faz passar. O recuo: toda extensão ganha uma configuração trabalhada, a prova trava/delegado é trabalhada linha a linha, e o challenge de fechamento é totalmente solo: reconstruir aquela prova como duas transações, sem apoio.

Agora veja uma suposição de proteção de valor falhar. Você habilita o CpiGuard numa conta de token, o trilho no nível da conta que impede um programa de mover seus fundos pelas suas costas através de uma CPI. Você está, com razão, convencido de que ela está lacrada. Aí uma única instrução esvazia ela mesmo assim, invocada através do PermanentDelegate do mint, que o CpiGuard não tem poder para impedir. Algumas autoridades ficam *acima* do holder da conta.

Antes de qualquer teoria, vá olhar uma dessas autoridades num token que você quase certamente já segurou. O PYUSD, a stablecoin da PayPal e da Paxos, carrega um delegado permanente no mint dele agora mesmo. Você tem o `solana` do setup anterior; se você pulou, instale as ferramentas Agave (solana-cli 3.1.10, verificado em 2026-08-22):

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Depois leia o mint do PYUSD direto da mainnet:

```bash
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta
```

O que volta é um dump hexadecimal, não uma listagem amigável, e o sinal está na linha do header: `Length: 866 (0x362) bytes` (verificado em 2026-08-22). Um mint SPL clássico tem 82 bytes. Tudo além disso é TLV, e dentro dele ficam uma entrada `permanentDelegate` de verdade e um `mintCloseAuthority` de verdade. Se você quer elas nomeadas em vez de contadas, aponte o seu próprio inspetor `decode-mint` para o mesmo endereço; nomear essas entradas é o trabalho para o qual você o construiu. As autoridades que você está prestes a configurar em mints descartáveis são exatamente as que um emissor regulado usa num token que guarda centenas de milhões de dólares. Tenha isso em mente. Esta é a superfície de compliance do token mais institucional da chain, não um conjunto de features de brinquedo.

## As autoridades que ficam acima do holder da conta

Comece pelo modelo mental, porque é a coisa que a maioria dos tutoriais nunca desenha. Uma conta de token tem um dono, e seu instinto diz que o dono é soberano sobre aquela conta. Para o SPL puro, esse instinto está mais ou menos certo. O Token-2022 quebra isso de propósito. Várias extensões instalam autoridades no nível do *mint*, e uma autoridade no nível do mint age sobre contas que o holder nunca consentiu em entregar. O dono da conta não assinou a configuração do mint. Ele optou por segurar o token, e segurar o token significou herdar quaisquer poderes que o emissor embutiu no mint. Essa assimetria é o assunto inteiro desta lição.

![Um diagrama de dois níveis com autoridades no nível do mint em cima alcançando para baixo as contas dos holders, mostrando que travas no nível da conta defendem apenas a camada da conta e não conseguem anular um delegado permanente no nível do mint.](assets/v01-diagram.png)

Um fato operacional antes do catálogo, porque ele molda toda decisão de emissão que você vai tomar com estas ferramentas. Todas as cinco dessas extensões são extensões de pré-inicialização: a instrução de config delas precisa rodar depois que a conta de mint é criada e antes do `initializeMint`, numa conta já dimensionada para a entrada TLV. Você não consegue parafusar uma autoridade num mint que já está vivo. Não existe "adicionar um delegado permanente depois, quando o compliance pedir um". O conjunto de autoridades é decidido no dia em que o mint nasce, sob incerteza, e você convive com ele pela vida inteira do token. Emissores que pulam esta análise não ganham uma segunda passada nela.

### PermanentDelegate, e a trava que ele sempre derrota

Um **delegado permanente** é um único endereço, definido uma vez no mint, que pode assinar `Transfer` e `Burn` para *qualquer* conta daquele mint. Não uma conta que o holder delegou para ele. Qualquer conta. É a primitiva de confisco: um emissor que precisa congelar-e-reaver por ordem judicial, reverter uma cunhagem equivocada ou drenar uma conta comprometida usa exatamente isto. O PYUSD carrega um. E é a autoridade mais afiada do catálogo de extensões inteiro, porque torna toda conta de holder trivialmente varrível por quem quer que segure a chave do delegado.

Não o confunda com o delegado que você já conhece do SPL clássico. Um delegado comum é concedido pelo holder: o dono assina um `approve` na própria conta, limita a um valor e pode dar `revoke` quando quiser. O escopo dele é uma conta, o orçamento dele é finito, e ele existe só enquanto o dono o tolerar. O delegado permanente inverte cada uma dessas propriedades. O emissor o define na criação do mint, nenhum holder jamais assina nada, não existe limite de valor, não existe revoke disponível para o holder, e o escopo dele é toda conta daquele mint que algum dia vai existir. Mesma palavra, espécie diferente. Uma é uma permissão que o holder estende. A outra é um poder que o holder herda ao escolher segurar o token.

![Contraste em duas colunas do delegado-por-approve concedido pelo dono, limitado e revogável, contra o delegado permanente definido pelo emissor, que vale para o mint inteiro, é ilimitado, irrevogável e sempre passa por cima do CpiGuard.](assets/v02-comparison.png)

Agora a revelação, e esta é a batida que vale desacelerar. Você pensaria que o CpiGuard impede isso. O CpiGuard é a extensão no nível da conta que um holder liga para dizer "nenhum programa pode mover fundos para fora da minha conta através de uma invocação entre programas sem a minha assinatura direta de nível superior". Ele existe precisamente para matar os truques de delegar-e-fechar que programas maliciosos aplicam. Então um holder cuidadoso habilita o CpiGuard e supõe que uma varredura por delegado permanente agora está bloqueada como qualquer outra jogada sorrateira via CPI.

Não está. E aqui a pergunta vale ser derivada em vez de afirmada, então leia o que a trava de fato promete. A especificação da própria extensão não diz "nenhum fundo sai durante uma CPI". Ela enuncia uma regra sobre *quem precisa estar assinando*:

![Painel de regra mostrando que o CpiGuard bloqueia uma transferência via CPI assinada pelo dono com CpiGuardTransferBlocked, enquanto o delegado permanente pode sempre transferir ou queimar, passando por cima da trava.](assets/v03-annotated-code.png)

Leia a regra e a resposta cai sozinha. A trava não pergunta "esta é uma jogada via CPI de que eu não gosto". Ela pergunta "o signer é um delegado". Isso inverte a leitura ingênua: a única autoridade que a trava recusa durante uma CPI é o *dono*, porque uma assinatura de dono é exatamente o que um programa malicioso colhe quando consegue que você assine uma instrução opaca. A delegação é visível e limitada, então a trava insiste nela. A autoridade geral do próprio dono é a coisa que está sofrendo engenharia social, então a trava a revoga dentro de uma CPI.

Agora ponha um delegado permanente contra essa regra e o resultado é superdeterminado pelos dois lados. Estruturalmente, o delegado permanente não é o dono, então a cláusula que bloqueia o dono nunca se aplica a ele. E o guia de extensões do Token-2022 fecha a porta explicitamente em vez de deixar para inferência: se um mint carrega a extensão de delegado permanente, aquele delegado pode sempre queimar ou transferir tokens, passando por cima do CPI Guard. A trava defende a camada da conta com honestidade e por completo. O delegado permanente opera uma camada acima, onde a trava não tem jurisdição. Esse é o limite do que uma defesa no nível da conta consegue prometer, não um bug no CpiGuard.

É por isso que a formulação importa e por que "PermanentDelegate sempre passa por cima do CpiGuard" não é um slogan, e sim um fato documentado e estrutural. Não existe configuração do CpiGuard que mude isso, porque a regra da trava é sobre delegação e o delegado permanente é excluído nominalmente.

A Raydium é direta sobre isso. A política de suporte a Token-2022 dela rejeita o PermanentDelegate, e o motivo declarado é: um holder do delegado consegue varrer qualquer conta de token, incluindo o vault do pool. (Leia o programa do pool, não só a página de documentação, e a rejeição tem uma porta dentro dela: a checagem é pulada para mints SPL clássicos, para uma `MINT_WHITELIST` curta e hardcoded, e para mints com uma conta de associação de mint inicializada. A política real é "nenhum, exceto os que a gente avaliou à mão".) Aquela frase única é a razão inteira pela qual um punhado de stablecoins de compliance entra naquela lista avaliada à mão pelo nome enquanto todo mundo que carrega a mesma extensão é recusado. Repare qual substantivo faz o trabalho: o TOKEN entra na whitelist, um endereço por vez. A EXTENSÃO com formato de compliance é recusada com a mesma dureza que qualquer outro poder, o que é uma distinção à qual m05-l2 e o capstone voltam. Se o seu token pode ter a liquidez dele varrida por uma única chave, um market maker automatizado que custodia liquidez numa conta vault não consegue listá-lo com segurança. Escolher o PermanentDelegate é escolher quais venues algum dia vão tocar no seu token.

E mesmo assim o PYUSD já vem com um, o que te diz que a troca é deliberada, não descuidada. A PayPal e a Paxos escolheram um conjunto de extensões com formato de compliance de propósito, e até 2025-05-29 o token guardava $215.9M em apenas 20.4k contas de token (Helius, "Solana's stablecoin landscape", 2025-05-29; supply circulante na Solana, não o total multi-chain; o campo `supply` daquele mint marcava 688,176,370,728,435 unidades base com 6 decimals em 2026-08-22). Essa proporção é o sinal de um instrumento institucional: valor enorme, poucos holders, um emissor que precisa da capacidade legal de congelar-e-reaver sob demanda. Um delegado permanente é um passivo para uma DEX e um ativo para um emissor regulado que responde a um departamento de compliance, e as duas leituras estão corretas ao mesmo tempo. A extensão é um seletor que aponta seu token para um tipo de lar e o afasta de outro. Quando você o define, você não está adicionando uma feature, está escolhendo um lado do ecossistema para o qual ser legível.

### Pausable: uma chave paralisa o mint inteiro

**Pausable** instala uma parada global. Quando a autoridade de pausa a aciona, toda transferência, cunhagem e queima daquele mint reverte de uma vez, na chain inteira, até alguém retomar. A cilada aqui é um erro de categoria: desenvolvedores buscam o Pausable esperando um congelamento por conta, um jeito de colocar um holder ruim em quarentena. Não é isso. É um botão de desligar para o token inteiro. Acione e você congelou todo holder simultaneamente, incluindo a sua própria liquidez, a sua própria tesouraria, todo usuário honesto no meio de uma transação. No processador, um mint pausado faz os caminhos de queima e transferência retornarem `MintPaused` incondicionalmente. Não existe argumento "pausar conta X", porque a pausa mora no mint, não na conta.

![Acionar o Pausable no mint paralisa toda transferência, cunhagem e queima para todos os holders simultaneamente até a mesma autoridade retomar, diferente de um congelamento, que mira uma única conta.](assets/v04-diagram.png)

Use para o que ele é: um freio de emergência para o token inteiro, uma ferramenta de resposta a incidentes, um jeito de estancar o sangramento durante um exploit. Nunca como aplicação direcionada. Se você precisa parar uma conta, isso é um congelamento, que é o estado no nível da conta que o DefaultAccountState governa. Busque a ferramenta de conta para um problema de conta.

A chave é simétrica, o que é um fardo operacional em si. A mesma autoridade de pausa retoma o mint com uma instrução de resume correspondente, e até ela fazer isso, nada se move para ninguém. Isso faz da custódia da chave de pausa uma questão de resposta a incidentes, não uma conveniência. Uma autoridade de pausa vazada é uma chave de negação de serviço contra o seu token inteiro, e uma autoridade de pausa parada no notebook de um engenheiro é um ponto único de falha para a liquidez de todo holder. Se você entregar o Pausable, ponha a chave atrás de um multisig e ensaie o caminho de retomada antes do dia em que você vai precisar da pausa. Um freio de emergência que ninguém consegue soltar é pior que nenhum freio.

### DefaultAccountState: onboarding congelado por padrão

**DefaultAccountState** definido como `Frozen` é a primitiva mais limpa para um formato específico de compliance: toda conta nova abre congelada e fica congelada até uma autoridade de congelamento a descongelar. Esse é o padrão "nenhum holder transaciona até o KYC passar", e ele é genuinamente elegante, porque inverte o default. Normalmente uma conta é usável no instante em que existe e você tem que pegar os maus atores depois do fato. Com DefaultAccountState(Frozen), a conta é inerte ao nascer e um holder se torna ativo só através de um descongelamento deliberado. O onboarding é opt-in pelo emissor, não opt-out.

![A conta de um holder abre congelada, transferências revertem com AccountFrozen até a autoridade de congelamento descongelar aquela conta específica depois das checagens, após o que transferências normais funcionam.](assets/v05-flowchart.png)

O contraste com o Pausable é a coisa a fixar, porque um quiz vai absolutamente tentar trocar um pelo outro com você. O Pausable é uma parada global que você aciona para o mint inteiro. O DefaultAccountState(Frozen) é uma cancela por conta que você libera um holder por vez com um descongelamento. Um é um botão de desligar. O outro é uma catraca. Eles parecem adjacentes e são ferramentas completamente diferentes.

Mais um grau de liberdade: o default não é para sempre. A autoridade de congelamento pode atualizar o estado default do mint depois, então um emissor pode lançar com acesso restrito e relaxar para onboarding aberto assim que o cenário de compliance clarear, sem tocar em uma única conta existente. Contas mantêm qualquer estado que já tenham; só contas criadas depois da atualização herdam o novo default. Lance rígido, afrouxe deliberadamente. Esse é o caminho de migração que a maioria dos times de compliance realmente quer, e é a única autoridade desta lição cuja postura pode suavizar ao longo da vida do token em vez de ficar congelada ao nascer.

### PermissionedBurn: a extensão que a documentação esqueceu

**PermissionedBurn** faz toda queima exigir a co-assinatura da autoridade de queima. Uma queima padrão, onde o holder torra os próprios tokens, para de funcionar no instante em que esta extensão está presente. O processador rejeita uma queima padrão contra um mint que carrega PermissionedBurn com `InvalidInstruction`, e te força pelo caminho permissionado, onde um signer extra, a autoridade de queima, precisa assinar junto. Emissores usam quando a destruição de supply precisa ser autorizada centralmente: pense em fluxos de resgate onde só o emissor pode retirar tokens de circulação.

O formato que isso serve é a contabilidade de resgate. Um holder faz off-ramp enviando tokens para a conta de custódia do emissor, o fiat sai por um trilho bancário, e aí o emissor, e só o emissor, retira o supply de circulação com uma queima permissionada a partir da custódia. O supply on-chain continua sendo um espelho honesto dos passivos off-chain porque ninguém mais consegue encolhê-lo: nenhum terceiro queima unilateralmente, e nenhum holder consegue desinflar o float às escondidas torrando tokens que os livros do emissor ainda contam como em circulação. Para um token cujo número de supply é uma afirmação auditada, aquela co-assinatura é a diferença entre um livro-razão e uma sugestão.

Aqui está a cilada, e é uma cilada de documentação, não uma cilada de código. Em m01-l4 você já conheceu o fato de que o catálogo de extensões do solana.com omite a PermissionedBurn por completo, enquanto o enum `ExtensionType` da fonte a lista como uma das 29 variantes de produção. Se você for procurar esta extensão na documentação oficial e concluir que ela não existe, você confiou numa página acima do código. O enum é a verdade. A documentação é o retrato que alguém tirou da verdade, envelhecendo em silêncio. Toda vez que você constrói contra o Token-2022, o enum no código-fonte fixado decide o que é real, e uma entrada faltando na documentação não decide nada. Esta é a segunda vez que este curso pega o catálogo oficial atrasado em relação ao código, e não vai ser a última.

### MintCloseAuthority: recuperando rent, e a cilada da ressurreição

**MintCloseAuthority** deixa uma autoridade designada fechar um mint assim que o supply dele é zero, recuperando os lamports de rent que estavam presos para manter a conta viva. O PYUSD carrega uma. Na maior parte do tempo é arrumação mundana: você subiu um mint, ele serviu ao propósito dele, você o fecha e recebe o rent de volta.

A cilada é sutil e morde em produção. Quando você fecha uma conta, os lamports dela drenam e os dados dela são zerados, mas o *endereço* não desaparece. Qualquer um pode mandar lamports de volta para aquele endereço e recriar uma conta ali. Uma conta fechada e depois revivida pode ser confundida com estado novo e confiável por código que supõe "este endereço existia antes, então é legítimo". A defesa é a higiene de marcar-como-fechado: escreva um byte sentinela na conta antes de fechar, para que uma conta revivida seja reconhecível como cadáver, não como recém-nascida. Se o seu sistema lê a mera existência de uma conta como prova de procedência, um ataque de ressurreição transforma essa suposição num buraco.

![Depois que um mint fecha, o endereço dele pode ser financiado de novo numa conta nova em que código ingênuo confia, então um byte sentinela escrito antes do fechamento marca ressurreições como reúso.](assets/v06-flowchart.png)

### O trade-off, nomeado honestamente

Cada uma dessas autoridades te compra a mesma moeda: controle. Congelar, pausar, reaver, restringir o onboarding, autorizar destruição. E cada uma delas gasta a mesma moeda em troca: descentralização, que uma exchange, um auditor ou um market maker vai ler como risco de contraparte. Não existe autoridade de graça. Um engenheiro de integração de DEX escaneando as entradas TLV do seu mint está lendo um perfil de risco, e cada extensão de autoridade é uma linha nele. A PermanentDelegate é a linha mais vermelha, que é exatamente por que a Raydium a recusa e por que stablecoins de compliance que a carregam entram na whitelist de venues específicos em vez de serem listadas em todo lugar.

Se você quer o procedimento de decisão em vez do feeling, você já o construiu: estas cinco encaixam direto na matriz de conflitos de m01-l4. O conjunto de perguntas é curto. Quem precisa conseguir agir contra um holder, sob qual gatilho legal, e em quais venues o token precisa viver? Uma stablecoin de folha de pagamento que responde a um regulador cai em PermanentDelegate mais DefaultAccountState(Frozen) e engole as restrições de listagem, porque os holders dela são contrapartes antes de serem usuários. Um token de comunidade que precisa de liquidez na Raydium não pode carregar um delegado permanente de jeito nenhum, seja lá o que os advogados preferissem. Escreva a lista de venues primeiro. Depois escolha só as autoridades que aquela lista permite, e documente as que você deliberadamente deixou de fora, porque "a gente podia ter tomado este poder e escolheu não tomar" é em si um sinal de confiança que auditores leem.

![Uma tabela de comparação das cinco extensões de autoridade listando o que cada uma controla, a cilada nomeada dela, e como uma DEX ou um auditor a lê como risco.](assets/v07-comparison.png)

Essa é a lente de design. Você não está escolhendo features, está escolhendo uma postura de confiança e, com ela, o conjunto de lugares onde seu token pode viver. Agora construa elas.

## Lab: configure as autoridades e depois quebre a trava

Sete passos, rodando contra um surfnet local para você ter o programa Token-2022 vivo sem gastar lamports de verdade. Os passos 1 a 5 são configurações trabalhadas que você roda como mostrado. O passo 6, o emblemático, é trabalhado por inteiro, e o challenge faz você reconstruí-lo sem o apoio. O passo 7 liga tudo ao critério que as lições posteriores presumem que roda verde. Reserve uns quarenta e cinco minutos, a maior parte no passo 6.

O Surfpool te dá um simnet que puxa contas da mainnet preguiçosamente conforme você toca nelas, com os programas SPL carregados e prontos. Uma ressalva que vale conhecer antes de ela te morder no passo 5: os programas SPL que um surfnet serve são os builds empacotados do próprio surfpool, não cópias byte a byte do que está deployado na mainnet, então uma instrução bem nova pode estar viva na mainnet e ausente do seu simnet. Instale se você ainda não instalou (eu estou no surfpool 1.2.1, verificado em 2026-08-22):

```bash
brew install txtx/taps/surfpool
```

Suba um surfnet num terminal e deixe rodando (as mesmas flags `--no-tui --no-studio` da lição passada, para a TUI não tomar conta do terminal que você está deixando aberto):

```bash
surfpool start --no-tui --no-studio
```

Uma nota sobre o workspace antes da instalação, porque o layout muda aqui e continua mudado pelo resto do curso. Dependências compartilhadas agora moram na RAIZ do workspace, a pasta que contém `labs/`. Rode as instalações abaixo a partir dessa raiz (se a raiz ainda não tiver `package.json`: `npm init -y && npm pkg set type=module` primeiro). O código das lições continua morando em pastas por lição como `labs/m02-l2/`, e todo comando de execução daqui em diante é dado a partir da raiz. O pacote autocontido `labs/m02-l1` da lição passada fica exatamente como está: imports relativos como `../m01-l2/decode-mint` resolvem por localização de arquivo, não por onde você roda, então nada ali quebra.

Os pins são o mesmo trio de m02-l1, pelos motivos discutidos em detalhe lá (kit 7.1.1 porque essa é a major contra a qual os clients deste workspace fazem peer; `@solana-program/token-2022@0.15.0` e `@solana-program/system@0.13.0` são as minors atuais que fazem peer com kit ^7, e as minors seguintes pulam para ^8 e falham duro com `ERESOLVE`). O client de token 0.15.0 já vem com todos os builders que esta lição precisa (`getInitializePermanentDelegateInstruction`, `getInitializePausableConfigInstruction`, `getInitializePermissionedBurnInstruction`, e o resto). Se o npm reclamar de um peer irresolúvel em `@solana/kit`, essa é exatamente esta emenda: fixe os três exatamente em vez de brigar com isso.

```bash
npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 @solana-program/system@0.13.0
npm install -D tsx@4.23.12 typescript@5.9.3
```

1. **Monte um helper que cria e inicializa um mint Token-2022 com um conjunto escolhido de extensões.** Crie `mkdir -p labs/m02-l2` e comece o arquivo da lição exatamente em `labs/m02-l2/verify-authorities.ts`; todo trecho dos passos 1 a 6 vai sendo acrescentado a este único arquivo, e o passo 7 o transforma no critério que uma lição posterior re-executa exatamente por esse caminho. Você construiu o apoio de criação de mint no lab de economia de m02-l1 (alocar a conta no tamanho certo, rodar as instruções de extensão de pré-init, depois `initializeMint`). Reaproveite. A única coisa nova por autoridade é qual instrução de pré-init você coloca na frente. Aqui está o formato, com o encanamento que o seu lab de economia já estabeleceu dobrado dentro de `createExtendedMint`, mais os dois signers em que todo passo posterior se apoia: `payer`, financiado por um airdrop no momento em que o arquivo começa, e `mintAuthority`, que nunca precisa de lamports porque o `payer` paga as taxas de tudo:

   ```typescript
   import {
     airdropFactory,
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     generateKeyPairSigner,
     lamports,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
     type KeyPairSigner,
   } from '@solana/kit';
   import { readFileSync } from 'node:fs';
   import {
     getInitializeMintInstruction,
     getMintSize,
     TOKEN_2022_PROGRAM_ADDRESS,
   } from '@solana-program/token-2022';
   import { getCreateAccountInstruction } from '@solana-program/system';

   // Cluster-agnostic on purpose: step 5 will want to re-run this whole file
   // against devnet, so the endpoints yield to env vars.
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions(process.env.RPC_WS_URL ?? 'ws://127.0.0.1:8900');
   const send = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   const airdrop = airdropFactory({ rpc, rpcSubscriptions });

   // The two signers the whole lab leans on. `payer` fee-pays and funds
   // everything; `mintAuthority` only ever signs, so it needs no lamports.
   //
   // The payer is env-overridable for one concrete reason. A throwaway signer
   // is perfect against a surfnet, which airdrops on demand, and useless
   // against devnet's faucet, which rate-limits: a funded address dies with
   // the process, so "top it up and retry" would fund a key the next run has
   // never heard of. Point PAYER_KEY at a file (`solana-keygen new -o
   // payer.json`), fund THAT address once, and step 5's devnet re-run works.
   const payer = process.env.PAYER_KEY
     ? await createKeyPairSignerFromBytes(
         new Uint8Array(JSON.parse(readFileSync(process.env.PAYER_KEY, 'utf8'))),
       )
     : await generateKeyPairSigner();
   const mintAuthority = await generateKeyPairSigner();
   console.log(`payer: ${payer.address}`);
   if (!process.env.PAYER_KEY) {
     await airdrop({
       recipientAddress: payer.address,
       lamports: lamports(5_000_000_000n),
       commitment: 'confirmed',
     });
   }

   async function submit(payer: KeyPairSigner, instructions: Instruction[]): Promise<void> {
     const { value: blockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(blockhash, m),
       (m) => appendTransactionMessageInstructions(instructions, m),
     );
     const signed = await signTransactionMessageWithSigners(message);
     // kit needs this narrowing: the signed tx's lifetime is a union until you assert it.
     assertIsTransactionWithBlockhashLifetime(signed);
     await send(signed, { commitment: 'confirmed' });
   }

   // preInit: extension instructions that must run BEFORE initializeMint.
   async function createExtendedMint(
     payer: KeyPairSigner,
     mintAuthority: KeyPairSigner,
     decimals: number,
     sizeExtensions: Parameters<typeof getMintSize>[0],
     preInit: (mint: KeyPairSigner) => Instruction[],
   ): Promise<KeyPairSigner> {
     const mint = await generateKeyPairSigner();
     const space = BigInt(getMintSize(sizeExtensions));
     const rent = await rpc.getMinimumBalanceForRentExemption(space).send();

     const create = getCreateAccountInstruction({
       payer,
       newAccount: mint,
       lamports: rent,
       space,
       programAddress: TOKEN_2022_PROGRAM_ADDRESS,
     });
     const initMint = getInitializeMintInstruction({
       mint: mint.address,
       decimals,
       mintAuthority: mintAuthority.address,
       freezeAuthority: mintAuthority.address,
     });

     await submit(payer, [create, ...preInit(mint), initMint]);
     return mint;
   }
   ```

   Checkpoint: `createExtendedMint` devolve um signer cujo `.address` é um mint vivo no seu surfnet. Nada faz assert ainda; os próximos passos alimentam ele com instruções de extensão de verdade.

2. **PermanentDelegate.** Coloque na frente uma instrução que nomeia o delegado:

   ```typescript
   import { getInitializePermanentDelegateInstruction } from '@solana-program/token-2022';
   import { extension } from '@solana-program/token-2022';

   const delegate = await generateKeyPairSigner();
   const pdMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('PermanentDelegate', { delegate: delegate.address })],
     (mint) => [
       getInitializePermanentDelegateInstruction({ mint: mint.address, delegate: delegate.address }),
     ],
   );
   ```

   Checkpoint: `decode-mint pdMint.address` (o seu inspetor de m01-l2) lista um TLV `PermanentDelegate` cujo `delegate` é igual a `delegate.address`.

3. **Pausable.** Mesmo padrão, mais a parada em si para você ver uma transferência reverter:

   ```typescript
   import {
     getInitializePausableConfigInstruction,
     getPauseInstruction,
   } from '@solana-program/token-2022';

   const pauseAuthority = mintAuthority;
   const pausableMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('PausableConfig', { authority: pauseAuthority.address, paused: false })],
     (mint) => [
       getInitializePausableConfigInstruction({ mint: mint.address, authority: pauseAuthority.address }),
     ],
   );

   // Flip the global halt.
   await submit(payer, [getPauseInstruction({ mint: pausableMint.address, authority: pauseAuthority })]);
   ```

   Checkpoint: depois da pausa, qualquer `transferChecked` contra `pausableMint` reverte com `MintPaused`, erro de programa customizado 0x43 (decimal 67). Faça assert da reversão, não de um sucesso, e note que ela dispara para *toda* conta do mint, não uma.

4. **DefaultAccountState(Frozen).** O argumento de estado é o enum `AccountState`:

   ```typescript
   import {
     getInitializeDefaultAccountStateInstruction,
     AccountState,
   } from '@solana-program/token-2022';

   const frozenDefaultMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('DefaultAccountState', { state: AccountState.Frozen })],
     (mint) => [
       getInitializeDefaultAccountStateInstruction({ mint: mint.address, state: AccountState.Frozen }),
     ],
   );
   ```

   Checkpoint: crie uma conta de token nova para este mint e tente enviar a partir dela. Ela reverte com `AccountFrozen`. Descongele aquela única conta com `getThawAccountInstruction` assinada pela autoridade de congelamento, tente de novo, e a transferência funciona. Você acabou de fazer o onboarding de um holder sem tocar em nenhum outro.

5. **PermissionedBurn e MintCloseAuthority.** Configure as duas — em mints separados, e com a mais nova atrás de uma sonda, porque este é o passo onde a ressalva do topo do lab morde:

   ```typescript
   import {
     getInitializePermissionedBurnInstruction,
     getInitializeMintCloseAuthorityInstruction,
     getBurnCheckedInstruction,
     getPermissionedBurnCheckedInstruction,
   } from '@solana-program/token-2022';

   const burnAuthority = await generateKeyPairSigner();

   // MintCloseAuthority on its own mint, unconditionally: every cluster's
   // Token-2022 build knows this extension, so step 7's TLV assert always
   // has a live mint to decode.
   const closableMint = await createExtendedMint(
     payer,
     mintAuthority,
     6,
     [extension('MintCloseAuthority', { closeAuthority: mintAuthority.address })],
     (mint) => [
       getInitializeMintCloseAuthorityInstruction({
         mint: mint.address,
         closeAuthority: mintAuthority.address,
       }),
     ],
   );

   // PermissionedBurn behind a probe: the newest extension in the catalog,
   // and this cluster's bundled build may predate it. On a build that does,
   // the extension initializer itself throws before the mint exists — catch
   // it, say so, and leave the mint null so step 7 can branch on it.
   let permissionedMint: KeyPairSigner | null = null;
   try {
     permissionedMint = await createExtendedMint(
       payer,
       mintAuthority,
       6,
       [extension('PermissionedBurn', { authority: burnAuthority.address })],
       (mint) => [
         getInitializePermissionedBurnInstruction({
           mint: mint.address,
           authority: burnAuthority.address,
         }),
       ],
     );
   } catch {
     console.log(
       'SKIPPED: PermissionedBurn (simnet build predates the extension; prove it on devnet with this step\'s re-run)',
     );
   }
   ```

   Checkpoint, em duas metades. Em qualquer cluster: `closableMint` está vivo e `decode-mint closableMint.address` lista o TLV `MintCloseAuthority` dele. Num cluster cujo build de Token-2022 conhece a PermissionedBurn: `permissionedMint` também está vivo, um `burnChecked` padrão contra ele reverte com `Error: Invalid instruction`, erro de programa customizado 0xc (decimal 12), e a queima permissionada, `getPermissionedBurnCheckedInstruction` com `burnAuthority` co-assinando, tem sucesso. A queima padrão morre no instante em que a PermissionedBurn está presente.

   Aqui está por que a sonda fica no código trabalhado em vez de ser deixada para você. A PermissionedBurn é a extensão mais nova do catálogo, e no surfpool 1.2.1 o build empacotado do Token-2022 ainda não a conhece: `getInitializePermissionedBurnInstruction` volta com `Error: Invalid instruction`, 0xc, do próprio inicializador da extensão, antes de o mint sequer ser criado. Isso é o seu simnet, não o seu código — e como este lab é um arquivo de awaits de nível superior rodados em ordem, um throw não capturado aqui mataria os passos 6 e 7 em toda execução no surfnet. O programa deployado na mainnet suporta sim, e você consegue provar isso sem gastar um lamport, porque uma simulação executa contra o programa real: monte a mesma lista de instruções e mande para o `simulateTransaction` na mainnet com `sigVerify: false` e `replaceRecentBlockhash: true`, e os logs voltam com `Instruction: PermissionedBurnExtension` / `PermissionedBurnInstruction::Initialize` / sucesso. Para ver o checkpoint completo rodar de verdade, a queima padrão morta e a queima co-assinada viva, as duas, re-execute o arquivo inteiro contra a devnet, o cluster onde você consegue escrever com o programa real — os endpoints e o payer ficaram sobrescrevíveis por env no passo 1 exatamente para este momento. Faça primeiro um payer que sobrevive a uma nova tentativa, porque o faucet da devnet tem rate limit e o signer descartável do passo 1 encalharia todo lamport que você desse a ele:

   ```bash
   solana-keygen new --no-bip39-passphrase -o labs/m02-l2/payer.json
   solana airdrop 2 "$(solana-keygen pubkey labs/m02-l2/payer.json)" --url devnet
   # rate-limited? paste that same address into faucet.solana.com, then retry
   PAYER_KEY=labs/m02-l2/payer.json \
     RPC_URL=https://api.devnet.solana.com RPC_WS_URL=wss://api.devnet.solana.com \
     npx tsx labs/m02-l2/verify-authorities.ts
   ``` Todo o resto neste lab roda no surfnet como está escrito; o Pausable, verificado no mesmo build, funciona normalmente.

6. **O emblemático: trava versus delegado.** Esta é a prova emblemática. Você tem um mint carregando um PermanentDelegate e uma conta de holder com CpiGuard habilitado. O CpiGuard só age *dentro de uma CPI*, então os dois movimentos passam pelo programa `spl-instruction-padding` (`iXpADd6AW1k5FaaXum5qHbSqyd7TtoN6AD7suVa83MF`), que embrulha uma instrução interna e a re-invoca via CPI.

   Primeiro, suba as contas sobre as quais a prova age, porque nenhum dos passos anteriores as criou: um dono, a conta de token dele segurando um pouco de `pdMint`, e um destino. O `payer` financia e paga as taxas de tudo, então nem `owner` nem `delegate` jamais precisa de um lamport próprio, e uma falha do pagador de taxas nunca pode se passar por um veredito da trava:

   ```typescript
   import {
     findAssociatedTokenPda,
     getCreateAssociatedTokenIdempotentInstruction,
     getMintToCheckedInstruction,
   } from '@solana-program/token-2022';

   const owner = await generateKeyPairSigner();
   const destinationOwner = await generateKeyPairSigner();

   const [ownerAccount] = await findAssociatedTokenPda({
     mint: pdMint.address,
     owner: owner.address,
     tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
   });
   const [destination] = await findAssociatedTokenPda({
     mint: pdMint.address,
     owner: destinationOwner.address,
     tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
   });

   await submit(payer, [
     getCreateAssociatedTokenIdempotentInstruction({
       payer,
       ata: ownerAccount,
       mint: pdMint.address,
       owner: owner.address,
     }),
     getCreateAssociatedTokenIdempotentInstruction({
       payer,
       ata: destination,
       mint: pdMint.address,
       owner: destinationOwner.address,
     }),
     getMintToCheckedInstruction({
       mint: pdMint.address,
       token: ownerAccount,
       mintAuthority,
       amount: 100n,
       decimals: 6,
     }),
   ]);
   ```

   Agora a prova em si:

   ```typescript
   import {
     ExtensionType,
     getEnableCpiGuardInstruction,
     getReallocateInstruction,
     getTransferCheckedInstruction,
   } from '@solana-program/token-2022';
   // `Instruction` is already imported at the top of this file (step 1); this
   // is one file, so re-importing the type is a TS2300 duplicate-identifier
   // error even though tsx strips it and runs fine.
   import { type Address, AccountRole } from '@solana/kit';

   const PADDING_PROGRAM =
     'iXpADd6AW1k5FaaXum5qHbSqyd7TtoN6AD7suVa83MF' as Address;

   // Wrap an inner token instruction so the padding program re-invokes it via CPI.
   // Wire format (PadInstruction::Wrap): [1][num_accounts u32 LE][data_len u32 LE][inner data].
   // Accounts: the inner accounts, then the inner program id as a readonly account.
   function wrapForCpi(inner: Instruction): Instruction {
     const innerAccounts = inner.accounts ?? [];
     const innerData = inner.data ?? new Uint8Array();
     const header = new Uint8Array(9);
     header[0] = 1; // Wrap
     new DataView(header.buffer).setUint32(1, innerAccounts.length, true);
     new DataView(header.buffer).setUint32(5, innerData.length, true);
     const data = new Uint8Array(header.length + innerData.length);
     data.set(header, 0);
     data.set(innerData, header.length);
     return {
       programAddress: PADDING_PROGRAM,
       accounts: [
         ...innerAccounts,
         { address: inner.programAddress, role: AccountRole.READONLY },
       ],
       data,
     };
   }

   // ownerAccount holds tokens, owned by `owner`, CpiGuard enabled.
   // pdMint carries the permanent delegate `delegate`.
   // `payer` fee-pays every transaction; kit collects the other signers
   // (owner, delegate) straight off the instructions they are embedded in.
   async function proveGuardVsDelegate(
     payer: KeyPairSigner,
     owner: KeyPairSigner,
     delegate: KeyPairSigner,
     ownerAccount: Address,
     destination: Address,
     pdMint: Address,
   ): Promise<{ ownerBlocked: boolean; delegatePassed: boolean }> {
     // The ATA was created at its minimal size, and an account extension needs
     // its bytes to exist before it can be enabled. So: grow the account with
     // Reallocate naming the extension, THEN flip the guard on. Skip the grow
     // and the enable fails on account size.
     await submit(payer, [
       getReallocateInstruction({
         token: ownerAccount,
         payer,
         owner,
         newExtensionTypes: [ExtensionType.CpiGuard],
       }),
       getEnableCpiGuardInstruction({ token: ownerAccount, owner }),
     ]);

     // Proof leg 1: the owner-signed transfer, wrapped for CPI. Expect it to
     // REVERT with CpiGuardTransferBlocked; ownerBlocked is true only if it did.
     const ownerMove = getTransferCheckedInstruction({
       source: ownerAccount,
       mint: pdMint,
       destination,
       authority: owner, // the owner is NOT a delegate -> the guard's must-be-a-delegate rule blocks this inside CPI
       amount: 1n,
       decimals: 6,
     });
     let ownerBlocked = false;
     try {
       await submit(payer, [wrapForCpi(ownerMove)]);
     } catch {
       ownerBlocked = true;
     }

     // Proof leg 2: the SAME transfer authorized by the permanent delegate,
     // wrapped identically. Expect it to SUCCEED; delegatePassed is true only if it confirmed.
     const delegateMove = getTransferCheckedInstruction({
       source: ownerAccount,
       mint: pdMint,
       destination,
       authority: delegate, // the permanent delegate is carved out by name -> always passes the guard
       amount: 1n,
       decimals: 6,
     });
     let delegatePassed = false;
     try {
       await submit(payer, [wrapForCpi(delegateMove)]);
       delegatePassed = true;
     } catch {
       delegatePassed = false;
     }

     return { ownerBlocked, delegatePassed };
   }
   ```

   Checkpoint, e este é o critério: `ownerBlocked === true` e `delegatePassed === true`. A transferência via CPI do próprio dono é parada pela trava que ele habilitou, e a transferência idêntica do delegado permanente passa direto pela mesma trava. Na minha execução a bloqueada voltou com erro de programa customizado 0x2a (decimal 42), `CpiGuardTransferBlocked`, e o log do programa `CPI Guard is enabled, and a program attempted to transfer user funds via CPI without using a delegate`, a mesma string que o painel de regra acima cita; o movimento do delegado confirmou. Vale rodar o controle também: mande a transferência do dono SEM o wrapper do padding e ela tem sucesso, porque a trava é inerte fora de uma CPI.

![A mesma conta protegida por CpiGuard bloqueia a própria transferência via CPI embrulhada de Alice, mas permite a transferência embrulhada idêntica do delegado permanente, porque a trava exige um signer delegado e o delegado permanente é excluído nominalmente.](assets/v08-diagram.png)

7. **Ligue tudo ao critério.** As cinco demonstrações já moram em um arquivo, `labs/m02-l2/verify-authorities.ts`; agora termine de transformá-lo em um critério: crie cada mint descartável, decodifique-o com o seu inspetor de m01-l2 para fazer assert de que o TLV da extensão dele está mesmo presente, depois rode as provas comportamentais: a transferência do Pausable revertendo, o congela-depois-descongela do DefaultAccountState, e o par trava-versus-delegado. A prova da PermissionedBurn é condicional, por causa da ressalva de simnet do passo 5 — e a sonda já existe: o código trabalhado do passo 5 deixa `permissionedMint` nulo num build que antecede a extensão e imprime a linha `SKIPPED: PermissionedBurn (simnet build predates the extension; prove it on devnet with this step's re-run)` para você. Ramifique nisso: quando `permissionedMint` for não-nulo, rode os asserts de queima-padrão-morta e queima-co-assinada-viva contra ele; quando for nulo, o skip impresso já contou a verdade. Um skip que nomeia o motivo dele mantém o critério honesto em todo cluster contra o qual este curso roda. Este arquivo é o artefato que a lição adiciona ao toolkit do SPROUT, `sprout-mint-authorities`, e ele consome as duas coisas que você já entregou: o encanamento de criação de mint do lab de economia e o inspetor `decode-mint`. A cauda de asserts do emblemático fica assim:

   ```typescript
   const { ownerBlocked, delegatePassed } = await proveGuardVsDelegate(
     payer, owner, delegate, ownerAccount, destination, pdMint.address,
   );
   if (!ownerBlocked) throw new Error('CpiGuard failed to block the owner-signed CPI move');
   if (!delegatePassed) throw new Error('permanent delegate did not bypass CpiGuard');
   console.log('authority gate: all assertions hold');
   ```

   Rode:

   ```bash
   npx tsx labs/m02-l2/verify-authorities.ts
   ```

   Checkpoint, e este é o critério da lição reformulado como script: toda extensão de autoridade está presente no mint dela, o movimento do PermanentDelegate tem sucesso através do CpiGuard enquanto o movimento CPI direto do dono é bloqueado, e a parada do Pausable faz uma transferência reverter. Quando aquela saída estiver verde, o artefato está na prateleira e o lab está feito. Mantenha o arquivo exatamente naquele caminho com exatamente esses asserts; uma lição posterior o chama pelo nome.

## Challenge

Nenhum apoio novo. Dado um mint que carrega um PermanentDelegate e uma conta com CpiGuard habilitado, escreva as duas transações, do zero, que provam qual operação a trava bloqueia e qual o delegado ultrapassa. Esta é a prova delegado-versus-trava, solo.

A sua barra de aceitação é exatamente o critério do lab, mas você constrói a coisa inteira sozinho: a trava precisa bloquear o movimento CPI sem delegado, assinado pelo dono, o movimento do delegado permanente precisa ter sucesso contra aquela mesma conta com trava, e os seus asserts precisam valer nos dois sentidos. Duas coisas separam um solo aprovado de um de sorte. Primeiro, os dois movimentos precisam ser embrulhados para executarem *dentro de uma CPI*, porque o CpiGuard é inerte numa instrução de nível superior; se você mandar uma transferência de dono sem embrulho e ela tiver sucesso, você não testou a trava, você contornou a condição. Segundo, a única diferença entre as suas duas transações é a autoridade que assina. Mesma conta de origem, mesmo destino, mesmo valor, mesmos decimals, mesmo wrapper. Se qualquer outra coisa diferir, você não isolou a variável, e a sua prova não prova nada. Quando o movimento CPI assinado pelo dono reverte com `CpiGuardTransferBlocked` e o movimento assinado pelo delegado confirma, e você consegue apontar a razão de uma linha na condição do processador, você domina isto.

Depois teste a si mesmo sob estresse contra a cilada que esta lição foi construída para desarmar: se você se pegar pensando "mas eu poderia apertar o CpiGuard para também bloquear o delegado", releia a regra. Não existe essa configuração. A única alavanca do CpiGuard é `lockCpi`, ligada ou desligada, e o que ela impõe quando está ligada é que a autoridade que assina durante uma CPI precisa ser um delegado. O guia da extensão então exclui o delegado permanente nominalmente. Apertar não está no cardápio.

Alguma coisa aqui devolveu um resultado que o meu não devolveu, ou o seu programa forkado recusou um init? Sinalize no canal de feedback do curso com o nome da extensão e o erro exato, de preferência com a instrução que falhou colada. Um leitor que pega outro inicializador de extensão que o build do simnet ainda não conhece, do jeito que a PermissionedBurn se comporta no passo 5, está fazendo reconhecimento de verdade do qual o resto da turma se beneficia, e é exatamente o hábito "verifique contra o programa vivo, não confie no tutorial" que este curso fica treinando.

Você agora lidou com o lado do MINT da autoridade: os poderes que um emissor embute no próprio token, sentados acima de todo holder. Isso deixa a outra metade da história. A próxima lição desce para o lado da conta e faz a pergunta invertida: o que a própria conta de um holder consegue *recusar*, mesmo quando o mint diz sim? O CpiGuard foi uma prévia daquela camada. Você vai conhecer o resto das extensões de proteção do holder, as travas que um holder habilita na própria conta, mais uma exceção deliberada que pertence a elas de qualquer jeito, NonTransferable, uma extensão do lado do mint cuja beneficiária é a integridade da credencial em vez do controle do emissor, e você vai ver onde o veto do holder termina e a autoridade do mint começa. Quando emissores compõem essas primitivas em trilhos de compliance reais, restrição de acesso baseada em congelamento empilhada em cima de delegados permanentes, esse é o território do curso planejado DeFi and RWA Engineering; aqui você construiu as primitivas que ele compõe.
