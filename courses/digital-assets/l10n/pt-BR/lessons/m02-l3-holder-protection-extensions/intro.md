# Extensões de proteção ao holder: o que as contas podem recusar

## Resumo

Na lição passada você configurou as autoridades do lado do mint, PermanentDelegate, Pausable, DefaultAccountState, PermissionedBurn, MintCloseAuthority, e viu o delegado permanente passar direto pelo CpiGuard como se a trava não estivesse lá. Aqueles eram poderes que o mint tem sobre o holder. Cada um deles dizia: não importa o que a sua conta queira, o mint decide.

Agora a imagem espelhada. Algumas extensões não são poderes sobre você. Elas são o veto da sua própria conta.

Antes de a gente falar de qualquer uma delas, coloque a sua bancada de volta no estado em que a deixamos. Suba o seu simnet local e rode de novo o critério do m02-l2, porque esta lição empilha direto em cima daquele artefato:

```bash
# surfpool 1.2.1, installed in m02-l1 (brew install txtx/taps/surfpool if you
# skipped it; other OSes: github.com/txtx/surfpool). Checked 2026-08-22.
surfpool start --no-tui --no-studio

# in a second terminal, from your course workspace:
npx tsx labs/m02-l2/verify-authorities.ts
```

Se isso ainda imprimir verde (em um simnet, a única linha nomeada `SKIPPED: PermissionedBurn` da sonda de cluster do critério conta como verde), a sua camada de autoridade está intacta e a gente pode construir em cima dela. Se não imprimir, conserte primeiro; nada nesta lição faz sentido sobre um piso quebrado.

Enquanto isso roda, segure a virada na cabeça, porque ela é a lição inteira. Um badge SPROUT soulbound que fisicamente não pode ser transferido, por ninguém, dono incluído. Uma conta de tesouraria que rejeita qualquer depósito que chegue sem um memo anexado. Uma conta cujo campo de dono está soldado para sempre. A tesouraria com memo é uma trava do lado da conta que um holder habilita. O dono soldado também é do lado da conta, mas ninguém o habilita: toda ATA que você já criou na vida já carrega ImmutableOwner automaticamente, e a lição prova isso para você. A primeira é a exceção deliberada na taxonomia desta lição: NonTransferable é uma extensão do lado do mint que o emissor configura, arquivada aqui de todo jeito porque a parte que ela protege é a integridade da credencial, não o controle do emissor, e é o holder, não o emissor, quem tem a transferência recusada. Todas elas são recusas, e hoje você vai configurar quatro delas e depois provar, com transações revertendo, que cada uma de fato recusa.

Esta lição desenvolve as proteções do lado da conta do Token-2022: NonTransferable, MemoTransfer, CpiGuard (revisitado do outro lado do escudo) e ImmutableOwner. Você vai aprender por que habilitar NonTransferable em um mint força um PAR específico de extensões em toda conta de holder, por que o MemoTransfer cobra silenciosamente um imposto de todo remetente que algum dia transferir para você, e por que toda ATA que você já criou na vida já carrega ImmutableOwner sem você pedir. No lab você estende o toolkit do SPROUT com um mint de badge soulbound e uma tesouraria com memo obrigatório, e o critério não é "funciona": o critério é que as operações proibidas revertam, de propósito, com os códigos de erro exatos da página.

Onde você está na rampa de autonomia, em voz alta: em m02-l1 você preencheu algumas lacunas de TODO dentro de um build que no resto era trabalhado, e em m02-l2 eu te entreguei o código de configuração completo e você rodou. Hoje a config ainda é trabalhada para você, mas as duas asserções estruturais são problemas de completion: eu vou te dizer o que provar e você escreve a linha que prova antes de ver a minha. O Challenge no final é totalmente solo, sem walkthrough, e essa é a forma de toda lição daqui para frente. As rodinhas estão sendo tiradas um parafuso por vez, conforme o cronograma.

## O veto da conta

### Dois lados do mesmo TLV

Aqui está a forma mais limpa que eu conheço de manter o catálogo do Token-2022 na cabeça: toda extensão responde a uma de duas perguntas. Quem pode agir sobre este token mesmo contra a vontade do holder? Esse é o lado do mint, da lição passada. E: o que esta conta pode recusar, mesmo quando todo mundo diz sim? Esse é o lado da conta, desta lição.

A distinção é física, não retórica. Você viu em m01-l2 que um mint e uma conta de token são os dois uma base de 165 bytes mais um byte de tipo mais uma caminhada TLV. Extensões do lado do mint moram no TLV do mint; proteções do lado da conta moram no TLV da conta do holder. Quando o seu inspetor `decode-mint` caminha pelo PYUSD, ele imprime entradas de mint. Quando você apontar ele para a sua própria ATA mais para frente no lab, você vai ver entradas de conta. Mesmos bytes, mesma caminhada, políticas opostas.

![Duas caixas contrastam extensões de poder do lado do mint com extensões de recusa do lado da conta, com uma seta mostrando NonTransferable no mint forçando NonTransferableAccount e ImmutableOwner nas contas de holder.](assets/v01-diagram.png)

Uma nota de pé de página, e curta porque m02-l1 já contou a história do arquivo: as proteções do lado da conta que você está ligando hoje são mecânicas da era 2026 que os cursos canônicos congelados nunca alcançaram. Os docs que existem descrevem cada extensão isolada; o que eles não ensinam é a parte que morde, os pareamentos forçados e os impostos de integração. Então é aí que a gente vai gastar o nosso tempo.

### NonTransferable: soulbound por construção

Primeiro o problema. Digamos que a Overgrowth, a co-op de farming cuja economia on-chain este curso vem construindo desde que o SPROUT foi especificado, queira premiar badges de SPROUT por completar uma temporada de harvest: um token fungível, decimals 0, uma unidade por conquista. O sentido inteiro de um badge é que VOCÊ o conquistou. No momento em que badges são transferíveis existe um mercado, e no momento em que existe um mercado, o badge para de significar "esta carteira fez a coisa" e passa a significar "esta carteira pagou pela prova de que alguma outra carteira fez a coisa." Para credenciais, a transferibilidade não é uma funcionalidade sendo removida. Ela é o ataque.

O destravamento é uma única extensão de nível de mint: NonTransferable, extensão tipo 9 no catálogo que o seu inspetor já mapeia. Habilitada na criação do mint (como a maioria das extensões, ela não pode ser adicionada depois da inicialização), ela faz toda transferência deste token reverter no nível do programa. Não "reverter a menos que seja admin", não "reverter a menos que você roteie com esperteza". O próprio token program recusa. Cunhar e queimar continuam funcionando, que é exatamente o ciclo de vida que um badge quer: o emissor cunha ele para você, ninguém move ele, você ou o emissor pode queimar ele para fazer a limpeza.

Agora derive a parte que os docs afirmam mas nunca explicam. Suponha que o programa só bloqueasse `transfer` e parasse aí. Você guarda um badge em uma conta de token. Contas de token têm uma autoridade de dono, e o token program base sempre deixou você reatribuí-la com SetAuthority. Então você "vende o seu badge" vendendo a conta inteira: assine um SetAuthority entregando a conta para o comprador. Nenhuma instrução de transferência jamais rodou, o saldo nunca se moveu entre contas, e a garantia soulbound está morta. O token não se moveu; a alma, sim.

É por isso que o pareamento é forçado. Quando você inicializa uma conta de token para um mint NonTransferable, o Token-2022 recusa criá-la a menos que a conta carregue ImmutableOwner, e ele estampa na conta uma extensão marcadora, NonTransferableAccount (tipo 13), registrando que esta conta guarda tokens soulbound. O par [NonTransferableAccount, ImmutableOwner] aparece em toda conta de holder, sempre, ou a conta não pode existir. Feche a porta da transferência e você precisa soldar a porta da propriedade também, senão a primeira porta era decoração. Isso não é uma convenção que você segue. O programa a impõe, e no lab você vai ler as duas entradas do TLV da sua própria conta.

![Fluxograma mostrando um mint não transferível bloqueando transferências com erro 0x25 e, via o par forçado NonTransferableAccount mais ImmutableOwner, bloqueando também a reatribuição de dono com erro 0x22, fechando a brecha da venda da conta.](assets/v02-flowchart.png)

A forma tem um nome que vale carregar: soulbound-fungível. Não um NFT com supply 1 e um padrão de metadados aparafusado, que é para onde o módulo 6 vai. Um mint fungível comum, decimals 0, com o supply que você quiser, cujas unidades estão soldadas a quem as recebeu. Um badge aqui é só um número que não pode se mover.

E a imposição roda mais cedo do que você imaginaria. A inicialização normal de conta adiciona ImmutableOwner para você, então no caminho feliz a checagem nunca dispara visivelmente. O Token-2022 protege o lado do mint de todo jeito: `MintTo` para uma conta sem propriedade imutável falha com erro de programa customizado 0x26 (decimal 38), e a mensagem é a decisão de design escrita por extenso, "Non-transferable tokens can't be minted to an account without immutable ownership". A própria suíte de testes do programa tem que dar trabalho para chegar nesse erro, recriando um mint no mesmo endereço com um conjunto de extensões diferente. O caminho de mint confidencial carrega a trava idêntica, com o ataque explicado em um comentário de código-fonte: sem ela, alguém poderia cunhar para uma conta de dono mutável e depois ir de SetAuthority até o controle dos tokens. Dois caminhos de código independentes recusando o mesmo buraco é um sinal decente de que o buraco é real.

Uma pergunta de design que você deveria estar fazendo, já que você construiu a alternativa na lição passada: por que não simplesmente congelar? DefaultAccountState(Frozen) com uma autoridade de congelamento que nunca descongela também produz tokens que ninguém pode mover. O guia de extensões do Token-2022 faz a comparação ele mesmo e nomeia a diferença: NonTransferable "é muito parecido com emitir um token e depois congelar a conta, mas permite que o dono queime e feche a conta se quiser." Uma conta congelada é inerte para todo mundo, o holder dela incluído, então o seu aprendiz fica preso pagando rent em um badge que ele não consegue nem limpar. Uma conta soulbound continua sendo dele para queimar e fechar. E congelar exige uma autoridade viva que você tem que manter, poderia abusar e pode perder. NonTransferable não exige ninguém. Para uma credencial o veredicto é fácil: coloque a garantia nos bytes do mint, não no seu bom comportamento continuado.

Uma mina de compatibilidade antes de a gente seguir, direto da matriz de conflitos que você construiu em m01-l4: NonTransferable com ConfidentialTransferMint é inválido a menos que ConfidentialMintBurn também esteja presente. Faz sentido quando você diz em voz alta (um token que não pode transferir não tem uso para transferências confidenciais, a menos que a maquinaria confidencial esteja ali para valores de mint e de burn). O seu validador check-combo daquela lição já sinaliza isso como a regra 5 da matriz; confie nele quando você compor.

### ImmutableOwner: o default que você nunca notou

O ImmutableOwner merece o seu próprio momento, porque você vem usando ele há anos sem consentir com isso, e isso é uma coisa boa.

A extensão faz um trabalho: ela imobiliza a autoridade de dono da conta para que SetAuthority nunca possa reatribuí-la. Por que isso importaria fora de badges soulbound? Por causa de como as ATAs funcionam. O endereço de uma conta de token associada é derivado deterministicamente de (dono, token program, mint). Todo mundo, carteiras, DEXes, scripts de airdrop, computa o endereço da sua ATA e manda fundos para lá sem te perguntar. Agora imagine que a propriedade fosse reatribuível: você re-assume a posse da sua ATA para outra pessoa, o endereço ainda deriva da SUA pubkey, e todo remetente futuro que computa "a ATA da carteira X" agora está financiando uma conta controlada por alguém que não é X. Uma classe inteira de truques de tomada de conta e de fundos mal direcionados vive nessa brecha.

Então o programa de ATA fechou isso: toda conta de token associada já vem com ImmutableOwner por padrão. No Token-2022 ela é uma entrada TLV real fazendo imposição real. E aqui está um detalhe que eu genuinamente amo: o programa SPL Token clássico não consegue armazenar extensões de jeito nenhum, então quando o programa de ATA manda para ele um InitializeImmutableOwner, o token clássico aceita a instrução como um no-op e loga "Please upgrade to SPL Token 2022 for immutable owner support". Um encolher de ombros polido, preservado em toda criação de ATA clássica que você já simulou na vida. O invariante do endereço derivado importa tanto que um programa o impõe e o outro pelo menos gesticula na direção dele. Entre os defaults silenciosos, esse é uma bênção.

![Muitos remetentes computam o mesmo endereço derivado de ATA, então reatribuir o dono dela redirecionaria depósitos futuros, e o ImmutableOwner faz essa reatribuição reverter com erro 0x22.](assets/v03-diagram.png)

A recusa que ela te compra é concreta, e você vai dispará-la no lab: SetAuthority com tipo de autoridade AccountOwner contra uma conta ImmutableOwner reverte com erro de programa customizado 0x22 (decimal 34). Em uma conta assim, a reatribuição de dono está eliminada, não meramente restrita.

### MemoTransfer: a conta que exige um recibo

O MemoTransfer inverte a direção do controle de um jeito que nenhuma outra extensão faz. Todo o resto que a gente tocou configura o que um mint ou uma conta pode fazer. O MemoTransfer configura o que todos os OUTROS têm que fazer para te alcançar.

A mecânica: MemoTransfer (tipo 8) é uma extensão de conta, habilitada pelo dono da conta, na conta, depois da criação. Uma vez habilitada, qualquer transferência que chega precisa ser imediatamente precedida na transação por uma instrução de memo, o programa SPL Memo escrevendo uma string no log da transação. Sem memo, sem depósito: a transferência reverte com erro de programa customizado 0x24 (decimal 36), e o log do programa explica tudo em inglês claro: "Error: No memo in previous instruction required for recipient to receive a transfer". (A string de Display do próprio tipo de erro carrega um ponto e vírgula depois de "instruction"; a string que o programa de fato loga não. Bata com o que o log imprime quando você fizer grep nela.) O caso de uso da Overgrowth se escreve sozinho: uma tesouraria de co-op em que todo pagamento que entra precisa carregar uma referência de liquidação, imposta pelo runtime em vez de por uma planilha e esperança. Exchanges rodam o mesmo padrão para etiquetar depósitos, e mesas de compliance amam isso porque a trilha de auditoria está no próprio livro-razão.

Mas olhe quem paga. Não você: você virou uma instrução e ganhou contabilidade imposta pelo runtime. O custo cai em todo remetente, para sempre. Um parceiro integrando a sua tesouraria escreve uma transferência normal e correta, testa ela contra contas normais, entrega, e ela reverte em produção contra a sua. Nada na API de transferência avisou ele; a exigência mora no TLV da SUA conta, e o código dele nunca olhou. Isso não é hipotético. O checklist de integração Token-2022 da Meteora diz aos integradores, literalmente, para "garantir que os destinos aceitem memo obrigatório", que é uma DEX documentando a configuração da sua conta como um perigo que os parceiros dela têm que contornar no código. Quando o checklist de um venue ao vivo nomeia a sua extensão, acredite no checklist.

![Duas pistas de transação mostram uma transferência sem memo revertendo com erro 0x24 na cancela de memo do destino, enquanto uma transferência idêntica precedida por uma instrução de memo aterrissa.](assets/v04-flowchart.png)

Antecipando a pergunta que você deveria estar fazendo: o dono pode desligar? Pode. O MemoTransfer é simétrico, habilitar e desabilitar existem os dois, os dois assinados pelo dono. É o veto do holder no sentido mais puro: entra quando quer, sai quando quer, e enquanto está ligado, o runtime faz a imposição da sua papelada por você.

### CpiGuard, revisitado do outro lado

Você conheceu o CpiGuard (tipo 11) na lição passada como a coisa que o PermanentDelegate humilha. Deixe eu dar a ele uma audiência mais justa agora que estamos do lado da conta na mesa, porque dentro da jurisdição real dele ele é uma proteção séria.

A ameaça que ele tem como alvo: você assina uma transação para algum programa, um jogo, um marketplace, um botão de claim de aparência inocente, e enterrado na execução daquele programa está uma CPI que chama o token program com autoridades para as quais você tecnicamente assinou. Aprovar um delegado, mudar o destino de fechamento, transferir com a sua assinatura de dono. Você autorizou UMA coisa no nível de topo; o programa gastou a sua autoridade em outras. O CpiGuard, habilitado com assinatura do dono na conta (como o MemoTransfer, alternável nos dois sentidos), bloqueia as operações perigosas em forma de autoridade quando elas chegam via CPI em vez de vir de uma instrução de nível de topo que você assinou visivelmente. Ações que a trava cobre precisam acontecer onde você pode ver, ou não acontecem.

A ressalva honesta, e ela continua estrutural desde a l2: o CpiGuard defende a superfície de autoridade da própria conta. Um PermanentDelegate no mint não é a autoridade da conta. Ele é um poder de nível de mint com o qual a conta nunca consentiu, e ele passa direto pela trava toda vez, o que você provou você mesmo com as suas duas transações na lição passada. Então coloque o CpiGuard no lugar certo do seu modelo mental: proteção real contra programas usando mal autoridades que você delegou, proteção zero contra poderes que o mint reservou acima de você. Uma trava na sua porta da frente, em uma casa onde o proprietário guardou uma chave mestra. Se a lista de ciladas diz "presumir que o CpiGuard é uma defesa completa", o conserto é segurar os dois fatos ao mesmo tempo, e nunca deixar uma alegação de segurança de carteira se apoiar só na trava.

![Diagrama do CpiGuard como um escudo bloqueando operações de autoridade invocadas por CPI enquanto ações de nível de topo assinadas pelo dono passam por uma cancela e um movimento de PermanentDelegate de nível de mint passa por cima do escudo intocado.](assets/v05-diagram.png)

### O que essas proteções custam

Toda lição deste módulo nomeia o seu trade-off, e esta tem o mais claro do curso até agora: proteções de holder tornam um token mais seguro e mais auditável ao estreitar quem pode transacionar com ele, e o custo é sempre empurrado para fora, para alguém que não é você.

NonTransferable mata mercados secundários por design; para um badge esse é o ponto, para qualquer coisa feita para ser negociada é fatal, e nenhuma DEX vai rotear ele um dia. O MemoTransfer te torna incompatível com todo remetente que não anexa memos, um imposto de integração arrecadado de parceiros que nunca leram o TLV da sua conta, que é por isso que o checklist da Meteora existe. O CpiGuard estreita quais programas conseguem compor utilmente com a sua conta, e a proteção é real mas parcial. O ImmutableOwner é o mais barato dos quatro, custo quase zero precisamente porque as ATAs o tornaram universal antes de alguém conseguir construir sobre o comportamento inseguro.

Então aqui está a regra de decisão, do jeito mais direto que eu consigo colocar. Vá de NonTransferable só quando a negociabilidade for a ameaça em vez da funcionalidade, porque você não pode desfazer isso depois de `initializeMint`. Vá de MemoTransfer só quando você controla as duas pontas do fio, ou quando as contrapartes são poucas o bastante para você avisar cada uma na mão. O CpiGuard é quase de graça em contas que você controla e uma coisa ruim de presumir em contas que você não controla. O ImmutableOwner você já tem e não escolheu. Se você não consegue nomear a operação exata que você quer recusada e a pessoa exata que vai ser incomodada pela recusa, você não está escolhendo uma proteção. Você está decorando um mint.

![Tabela de comparação de NonTransferable, ImmutableOwner, MemoTransfer e CpiGuard mostrando onde cada um mora, o que ele recusa, o código de erro observado dele e quem arca com o custo.](assets/v06-comparison.png)

Note o tema: todas as quatro tornam coisas impossíveis em vez de possíveis, seletivamente, e a disciplina de engenharia que elas exigem é provar a impossibilidade em vez de afirmá-la. Que é precisamente o que o lab faz.

## Lab: transforme a recusa em teste

O artefato que esta lição adiciona ao toolkit da Overgrowth é o `sprout-mint-protections`: um mint de badge soulbound com o seu par de conta forçado, e uma tesouraria com memo obrigatório que rejeita depósitos sem etiqueta, tudo provado por um script de critério em que as asserções são reversões. Ele é construído ao lado dos seus mints de autoridade de m02-l2; você está empilhando uma segunda camada, não substituindo a primeira. Rodei esse critério exato quatro vezes enquanto escrevia esta lição, em um simnet novo cada vez: as mesmas três recusas, os mesmos códigos de erro, em toda rodada. A sua deveria ser tão sem graça quanto.

![Pipeline dos oito passos do lab, do financiamento até a criação do mint soulbound, a asserção do par forçado, três reversões esperadas, o depósito com memo e o CpiGuard, terminando em um critério verde.](assets/v07-flowchart.png)

1. **Workspace e pins.** Trabalhe na raiz do workspace, o layout que m02-l2 estabeleceu (deps compartilhadas no `package.json` da raiz, código da lição embaixo de `labs/`), com o simnet do começo ainda rodando. Os pins são o conjunto de m02-l1 mais um recém-chegado, o memo, e a mesma regra do parágrafo de pins daquela lição decide toda versão aqui: minor atual que faz peer com o kit ^7, re-verifique quando você ler isto.

   ```bash
   npm install @solana/kit@7.1.1 @solana-program/token-2022@0.15.0 \
               @solana-program/memo@0.12.0 @solana-program/system@0.13.0
   npm install -D tsx@4.23.12 typescript@5.9.3   # already there if you did the m02-l2 root install
   # memo 0.12.0 and system 0.13.0: the kit-^7-peer versions of each,
   # verified against npm 2026-09-05. Newer minors peer kit ^8.
   ```

2. **Monte o esqueleto do critério.** Crie `labs/m02-l3/verify-protections.ts`. Imports e três helpers: um enviador de transações (o mesmo pipe do kit que você vem construindo desde m01-l3, agora fatorado para fora porque a gente vai enviar nove transações), um `expectRevert` que FALHA se a operação tiver sucesso, e um criador de ATA. Leia `expectRevert` duas vezes; ele é a postura de engenharia da lição em oito linhas. A op proibida passando é a condição de erro.

   ```ts
   // labs/m02-l3/verify-protections.ts
   // Gate for m02-l3: every holder-protection extension must refuse the op it exists to refuse.
   // Run against a local surfpool simnet: `surfpool start --no-tui --no-studio` in another terminal, then
   // `npx tsx labs/m02-l3/verify-protections.ts`.
   import {
     airdropFactory,
     assertIsTransactionWithBlockhashLifetime,
     appendTransactionMessageInstructions,
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
   import { getCreateAccountInstruction } from '@solana-program/system';
   import { getAddMemoInstruction } from '@solana-program/memo';
   import {
     AuthorityType,
     ExtensionType,
     TOKEN_2022_PROGRAM_ADDRESS,
     extension,
     fetchToken,
     findAssociatedTokenPda,
     getCreateAssociatedTokenIdempotentInstructionAsync,
     getEnableCpiGuardInstruction,
     getEnableMemoTransfersInstruction,
     getInitializeMintInstruction,
     getInitializeNonTransferableMintInstruction,
     getMintSize,
     getMintToInstruction,
     getReallocateInstruction,
     getSetAuthorityInstruction,
     getTransferCheckedInstruction,
   } from '@solana-program/token-2022';

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   const airdrop = airdropFactory({ rpc, rpcSubscriptions });

   async function sendTx(feePayer: KeyPairSigner, instructions: Instruction[]) {
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const tx = await pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions(instructions, m),
       (m) => signTransactionMessageWithSigners(m),
     );
     assertIsTransactionWithBlockhashLifetime(tx);
     await sendAndConfirm(tx, { commitment: 'confirmed' });
   }

   async function expectRevert(label: string, run: () => Promise<void>) {
     try {
       await run();
     } catch {
       console.log(`PASS  ${label}: reverted as required`);
       return;
     }
     throw new Error(`FAIL  ${label}: the disallowed op went through`);
   }

   async function createAta(payer: KeyPairSigner, mint: KeyPairSigner, owner: KeyPairSigner) {
     const [ata] = await findAssociatedTokenPda({
       owner: owner.address,
       mint: mint.address,
       tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
     });
     await sendTx(payer, [
       await getCreateAssociatedTokenIdempotentInstructionAsync({
         payer,
         owner: owner.address,
         mint: mint.address,
         tokenProgram: TOKEN_2022_PROGRAM_ADDRESS,
       }),
     ]);
     return ata;
   }
   ```

3. **O mint do badge soulbound.** Abra o `main()`, financie dois atores e crie o mint. A ordem dentro da transação de criação é a mesma regra que você aprendeu em m02-l1 e ela ainda morde: os inicializadores de extensão rodam ANTES de `initializeMint`, porque uma vez que o mint é inicializado o conjunto de extensões dele está selado. `getMintSize` com a lista de extensões computa o espaço exato incluindo o TLV, a mesma matemática que o seu inspetor fez engenharia reversa de `try_calculate_account_len` em m01-l2.

   ```ts
   async function main() {
     const payer = await generateKeyPairSigner();
     const alice = await generateKeyPairSigner();
     const bob = await generateKeyPairSigner();
     await airdrop({
       commitment: 'confirmed',
       recipientAddress: payer.address,
       lamports: lamports(5_000_000_000n),
     });
     await airdrop({
       commitment: 'confirmed',
       recipientAddress: alice.address,
       lamports: lamports(1_000_000_000n),
     });

     // ---- 1. Soulbound badge mint: NonTransferable forces the account-extension pair ----
     const badgeMint = await generateKeyPairSigner();
     const badgeSpace = BigInt(getMintSize([extension('NonTransferable', {})]));
     const badgeRent = await rpc.getMinimumBalanceForRentExemption(badgeSpace).send();
     await sendTx(payer, [
       getCreateAccountInstruction({
         payer,
         newAccount: badgeMint,
         space: badgeSpace,
         lamports: badgeRent,
         programAddress: TOKEN_2022_PROGRAM_ADDRESS,
       }),
       getInitializeNonTransferableMintInstruction({ mint: badgeMint.address }),
       getInitializeMintInstruction({
         mint: badgeMint.address,
         decimals: 0,
         mintAuthority: payer.address,
       }),
     ]);
   ```

4. **Contas de holder, e o par forçado: a sua asserção primeiro.** Crie ATAs para a alice e o bob e cunhe para a alice o badge dela. Depois pare, porque este é o primeiro problema de completion. Você sabe pela teoria que a conta da alice tem que carregar agora tanto NonTransferableAccount quanto ImmutableOwner, ou a teoria está errada. `fetchToken` retorna a conta decodificada com `data.extensions` como uma Option sobre um array de entradas `{ __kind: ... }`. Escreva a asserção você mesmo antes de rolar a tela: busque, desembrulhe a option, colete os kinds e jogue erro se qualquer um dos kinds exigidos estiver faltando. Depois compare com a minha:

   ```ts
     const aliceBadge = await createAta(payer, badgeMint, alice);
     const bobBadge = await createAta(payer, badgeMint, bob);
     await sendTx(payer, [
       getMintToInstruction({
         mint: badgeMint.address,
         token: aliceBadge,
         mintAuthority: payer,
         amount: 1n,
       }),
     ]);

     // The forced pair: NonTransferableAccount + ImmutableOwner on the holder account.
     const aliceBadgeAccount = await fetchToken(rpc, aliceBadge);
     const exts = aliceBadgeAccount.data.extensions;
     const kinds = exts.__option === 'Some' ? exts.value.map((e) => e.__kind) : [];
     for (const required of ['NonTransferableAccount', 'ImmutableOwner'] as const) {
       if (!kinds.includes(required)) {
         throw new Error(`FAIL  forced pair: holder account is missing ${required}`);
       }
     }
     console.log(`PASS  forced pair: holder account carries [${kinds.join(', ')}]`);
   ```

   Na minha rodada a linha de pass imprimiu o par na ordem de criação:

   ```text
   PASS  forced pair: holder account carries [ImmutableOwner, NonTransferableAccount]
   ```

   Você nunca pediu nenhuma das duas extensões. Você inicializou um mint NonTransferable e uma ATA comum, e o programa colocou as duas entradas ali porque a conta não poderia existir legalmente sem elas. Para uma segunda opinião direto dos bytes, aponte o seu próprio inspetor para a conta (`npx tsx decode-mint.ts <aliceBadge address> http://127.0.0.1:8899`): a caminhada TLV que você escreveu em m01-l2 lê contas de token exatamente como mints, e ela vai imprimir tipo 7 e tipo 13 ao lado dos nomes.

![Saída anotada do inspetor da conta de holder do badge mostrando uma base de 165 bytes, byte de tipo de conta 2 e duas entradas TLV forçadas de comprimento zero, ImmutableOwner tipo 7 e NonTransferableAccount tipo 13.](assets/v08-annotated-code.png)

5. **Duas recusas, provadas.** Agora torne a teoria falsificável. A alice, a dona legítima, assina uma transferência do próprio badge dela para o bob: tem que reverter. Depois ela tenta entregar a conta em si para o bob via SetAuthority: tem que reverter. As duas passam por `expectRevert`, então se qualquer uma tiver sucesso, o critério morre aos berros.

   ```ts
     // A soulbound badge cannot move, even with the owner signing.
     await expectRevert('NonTransferable transfer', () =>
       sendTx(payer, [
         getTransferCheckedInstruction({
           source: aliceBadge,
           mint: badgeMint.address,
           destination: bobBadge,
           authority: alice,
           amount: 1n,
           decimals: 0,
         }),
       ]),
     );

     // ImmutableOwner refuses owner reassignment on the same account.
     await expectRevert('ImmutableOwner reassignment', () =>
       sendTx(payer, [
         getSetAuthorityInstruction({
           owned: aliceBadge,
           owner: alice,
           authorityType: AuthorityType.AccountOwner,
           newAuthority: bob.address,
         }),
       ]),
     );
   ```

   Se você quiser ver as recusas cruas em vez do catch engolindo elas, simule qualquer uma das transações e leia os logs. A transferência morre com `custom program error: 0x25` (decimal 37, a recusa de não transferível do Token-2022) e a reatribuição com `custom program error: 0x22` (decimal 34, a recusa de dono imutável). Eu tirei os dois códigos de logs de simulação na minha própria rodada de simnet de 2026-08-22; vale reconhecer eles de bate-pronto, porque em produção eles chegam sem lição nenhuma anexada.

6. **A tesouraria com memo obrigatório.** Segunda metade do artefato. Crie um mint transferível comum fazendo o papel do SPROUT (decimals 6, sem extensões de mint: a proteção que a gente está testando mora na CONTA), uma conta de remetente financiada para o payer, e uma ATA de tesouraria cujo dono é a alice. Depois a adesão do lado do dono, e note a dança de dois passos: a ATA foi criada no tamanho mínimo dela, então a alice primeiro a cresce com `Reallocate` nomeando os tipos de extensão para os quais ela quer espaço, depois vira o `EnableMemoTransfers`. As duas instruções são dela para assinar, de mais ninguém. É isso que "proteção do holder" significa nos bytes.

   ```ts
     // ---- 2. Memo-required treasury: MemoTransfer rejects memo-less deposits ----
     const sproutMint = await generateKeyPairSigner();
     const sproutSpace = BigInt(getMintSize());
     const sproutRent = await rpc.getMinimumBalanceForRentExemption(sproutSpace).send();
     await sendTx(payer, [
       getCreateAccountInstruction({
         payer,
         newAccount: sproutMint,
         space: sproutSpace,
         lamports: sproutRent,
         programAddress: TOKEN_2022_PROGRAM_ADDRESS,
       }),
       getInitializeMintInstruction({
         mint: sproutMint.address,
         decimals: 6,
         mintAuthority: payer.address,
       }),
     ]);

     const senderSprout = await createAta(payer, sproutMint, payer);
     const treasury = await createAta(payer, sproutMint, alice);
     await sendTx(payer, [
       getMintToInstruction({
         mint: sproutMint.address,
         token: senderSprout,
         mintAuthority: payer,
         amount: 1_000_000_000n,
       }),
     ]);

     // Holder-side opt-in: grow the account, then flip the requirement on.
     await sendTx(alice, [
       getReallocateInstruction({
         token: treasury,
         payer: alice,
         owner: alice,
         newExtensionTypes: [ExtensionType.MemoTransfer],
       }),
       getEnableMemoTransfersInstruction({ token: treasury, owner: alice }),
     ]);
   ```

7. **O remetente ingênuo falha; o remetente informado paga o imposto.** Segundo problema de completion, e este é sobre ser o remetente. Primeira transação: um `TransferChecked` perfeitamente normal de 25 SPROUT para a tesouraria, embrulhado em `expectRevert`, porque você está fazendo o papel do parceiro que nunca leu o TLV. Segunda transação: a mesma transferência, corrigida. Antes de você olhar a minha versão, responda pela teoria: o que exatamente o conserto exige, e em que lado do fio ele mora? Escreva a transação corrigida, depois compare:

   ```ts
     await expectRevert('memo-less deposit', () =>
       sendTx(payer, [
         getTransferCheckedInstruction({
           source: senderSprout,
           mint: sproutMint.address,
           destination: treasury,
           authority: payer,
           amount: 25_000_000n,
           decimals: 6,
         }),
       ]),
     );

     // Same transfer, memo attached first: the sender pays the integration tax.
     await sendTx(payer, [
       getAddMemoInstruction({ memo: 'harvest-settlement:2026-08-22' }),
       getTransferCheckedInstruction({
         source: senderSprout,
         mint: sproutMint.address,
         destination: treasury,
         authority: payer,
         amount: 25_000_000n,
         decimals: 6,
       }),
     ]);
     console.log('PASS  memo-carrying deposit landed');
   ```

   O conserto inteiro é um `getAddMemoInstruction` colocado antes da transferência, na transação do remetente. A tesouraria não mudou nada entre o depósito que falha e o que aterrissa. Simule a versão que falha e o log do programa te entrega a história inteira: `Error: No memo in previous instruction required for recipient to receive a transfer`, erro de programa customizado 0x24. Aquela linha de log é do que o checklist da Meteora está defendendo os integradores dele.

8. **CpiGuard, habilitado e verificado, e rode o critério.** Última camada: o payer endurece a conta de remetente com o CpiGuard, a mesma dança de realoca-depois-habilita, e a gente faz assert de que a extensão de fato aterrissou no TLV. O que a gente não faz é demonstrar o bloqueio em si, e eu quero ser franco sobre por quê: o CpiGuard recusa operações que chegam via CPI, então disparar ele honestamente precisa de um programa deployado fazendo a chamada, e você provou tanto o bloqueio quanto o bypass do PermanentDelegate com exatamente esse setup em m02-l2. Esta asserção é presença; a de m02-l2 era comportamento; juntas elas são o quadro completo.

   ```ts
     // ---- 3. CpiGuard: enabled and present (the bypass demo lives in m02-l2) ----
     await sendTx(payer, [
       getReallocateInstruction({
         token: senderSprout,
         payer,
         owner: payer,
         newExtensionTypes: [ExtensionType.CpiGuard],
       }),
       getEnableCpiGuardInstruction({ token: senderSprout, owner: payer }),
     ]);
     const guarded = await fetchToken(rpc, senderSprout);
     const guardedKinds =
       guarded.data.extensions.__option === 'Some'
         ? guarded.data.extensions.value.map((e) => e.__kind)
         : [];
     if (!guardedKinds.includes('CpiGuard')) {
       throw new Error('FAIL  CpiGuard: extension not present after enable');
     }
     console.log('PASS  CpiGuard enabled on the sender account');

     console.log('\nAll holder-protection assertions hold. m02-l3 gate: green.');
   }

   main().catch((err) => {
     console.error(err);
     process.exit(1);
   });
   ```

   Rode:

   ```bash
   npx tsx labs/m02-l3/verify-protections.ts
   ```

   Saída esperada, literal da minha rodada de 2026-08-22:

   ```text
   PASS  forced pair: holder account carries [ImmutableOwner, NonTransferableAccount]
   PASS  NonTransferable transfer: reverted as required
   PASS  ImmutableOwner reassignment: reverted as required
   PASS  memo-less deposit: reverted as required
   PASS  memo-carrying deposit landed
   PASS  CpiGuard enabled on the sender account

   All holder-protection assertions hold. m02-l3 gate: green.
   ```

   Seis passes, três dos quais são falhas se comportando corretamente. Esse é o critério.

## Challenge

Solo, sem walkthrough, e este é o exercício de config de proteção de holder para o qual o módulo vem construindo: escolha uma extensão de controle que a gente cobriu neste módulo, qualquer uma delas, do lado do mint ou do lado da conta, e escreva `labs/m02-l3/challenge-control.ts` que a configura em um mint ou conta descartável e prova com uma asserção no estilo `expectRevert` que ela bloqueia uma operação proibida. DefaultAccountState(Frozen) rejeitando uma transferência para uma conta que nunca foi descongelada é uma escolha limpa; uma variante nova de MemoTransfer com o caminho de desabilitar também verificado também é. A sua barra de aceitação, a mesma do lab: a op proibida tem que reverter, a asserção tem que FALHAR aos berros se ela algum dia parar de reverter, e um comentário de uma linha tem que nomear o código de erro que você observou e como o programa o chama. Se o seu script passar na primeira tentativa, desconfie; delete a instrução que habilita e confirme que o critério fica vermelho pelo motivo certo antes de confiar no verde.

## O que o SPROUT recusa agora

Faça o balanço da escada de artefatos, porque ela vai se acumulando quieta. O R1 lê qualquer mint ou conta até o TLV. A camada de economia roteia valor. A camada de autoridade de m02-l2 diz quem pode mover, congelar e reaver. E a partir de hoje, o `sprout-mint-protections` adiciona a outra voz na conversa: um badge que não pode deixar quem o conquistou, uma tesouraria que recusa dinheiro sem documentação, contas cuja propriedade não pode ser reatribuída, e uma trava cujos limites você consegue enunciar com precisão porque você os mediu dos dois lados. Você não leu isso em uma matriz. Você fez cada recusa acontecer e a pegou em um teste.

Se alguma reversão não disparou na sua máquina, ou disparou com um código diferente dos que estão nesta página, esse é exatamente o tipo de relato que eu quero ouvir, com os seus logs de simnet anexados; o trem da toolchain se move mensalmente e o critério existe para pegar ele se movendo.

O SPROUT agora impõe quem pode movê-lo e o que as contas dele recusam. Mas procure ele em qualquer carteira e ele ainda é uma pubkey com um saldo: sem nome, sem símbolo, sem imagem, nada que um humano consiga renderizar. Na próxima lição a gente conserta isso onde o Token-2022 quer que seja consertado, metadados nativos armazenados no próprio mint, e o seu inspetor ganha a chance de ler um token que finalmente se apresenta.
