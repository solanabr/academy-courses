# Resolvendo hooks, e por que eles quebram a composabilidade

## Resumo

Na lição passada você escreveu o harvest-hook e provou ele numa bancada LiteSVM: uma transferência na allowlist passou, uma não permitida falhou. Mas a bancada entregou as contas ao hook na mão, e essa é a ficção que esta lição quebra. Hoje você senta na cadeira do consumidor: você constrói uma transferência Token-2022 comum de SPROUT com hook do jeito que uma carteira construiria e vê ela reverter com uma conta faltando, depois você escreve o resolvedor do lado do cliente que lê o ExtraAccountMetaList do programa do hook, reconstrói a transferência com as contas certas, e faz ela aterrissar. Com essa evidência na mesa, a gente deriva o fato de design em que todo o argumento de composabilidade se apoia: dentro do Execute, toda conta da transferência original chega somente leitura e não signatária, então um hook nunca consegue gastar aquilo que ele inspeciona. Checagem do recuo: o resolvedor é trabalhado junto com você linha a linha, os scripts de transferência são seus para ligar e rodar, e o challenge é totalmente solo. Nota de escopo: a interface de transfer hook é ensinada de ponta a ponta aqui; cursos irmãos que encostam em transferências com hook apontam de volta para cá em vez de re-derivar ela.

O seu hook passou nos testes. Eu quero estragar essa sensação nos primeiros cinco minutos, porque a bancada estava mentindo para você com educação: o LiteSVM deixou você mesmo colocar cada conta na transação, feito uma equipe de palco arrumando os objetos de cena antes de o ator entrar. Carteiras de verdade não sabem que o seu hook existe. DEXes de verdade não sabem que o seu hook existe. No momento em que qualquer uma das duas constrói um TransferChecked padrão de quatro contas para a sua variante do SPROUT, o runtime vai procurar contas que não estão na transação, e a coisa toda morre na plataforma de lançamento.

Antes de qualquer teoria, rode isto. Ele calcula oito bytes que você conheceu na lição passada pelo lado do programa:

```bash
node -e 'const {createHash} = require("node:crypto");
console.log(createHash("sha256").update("spl-transfer-hook-interface:execute").digest().subarray(0, 8).toString("hex"))'
```

Você deve ver `692565c54bfb661a`. Na lição passada esses bytes eram a campainha que o Token-2022 toca no seu programa. Nesta lição eles são uma dívida: cada integrador que um dia encostar no seu token precisa achar as contas que esse discriminador exige, ou a transferência dele reverte. Mantenha esse hex à vista; você vai dar um grep nele dentro de uma conta de verdade daqui a alguns minutos.

## O imposto de resolução

### Quem deveria montar as contas?

Comece pela restrição que você já domina desde o primeiro módulo deste curso: uma transação Solana precisa declarar, de saída, cada conta em que ela vai tocar. O runtime não descobre contas no meio do voo; ele escalona em volta da lista declarada. Agora some o que você construiu na lição passada: quando um mint carrega a extensão TransferHook, o Token-2022 faz CPI no programa do hook em toda transferência, e o Execute do seu harvest-hook precisa das próprias contas dele para fazer o trabalho dele, o estado de allowlist que ele checa e o log de tesouraria que ele escreve.

Junte esses dois fatos e uma pergunta salta com força de verdade. A carteira que constrói a transferência nunca ouviu falar do seu programa. O Token-2022 sabe o endereço do hook pelo mint, mas não sabe que contas um programa arbitrário pode querer. O hook conhece as próprias necessidades, mas programas não conseguem esticar o braço e buscar contas em runtime. Alguém tem que colocar as contas do hook na transação antes de ela ser enviada, e nada na pilha se voluntaria.

Percorra as respostas ingênuas, porque cada uma falha de um jeito instrutivo. "O hook deveria carregar o que ele precisa" falha primeiro: em Solana não existe carregar, uma conta que não está na transação não existe do ponto de vista do programa. "O Token-2022 deveria deixar as contas extras hardcoded" falha em segundo: o ponto inteiro do slot de hook é que o programa por trás dele é arbitrário, então nenhuma lista fixa serve todo hook. "Faça a carteira do remetente simplesmente saber" não é uma resposta, é o problema reescrito.

A resposta de verdade é a única peça da interface em que você ainda não encostou: o hook publica um manifesto legível por máquina das contas de que ele precisa, on-chain, num endereço conhecido, e espera-se que todo cliente leia ele. Esse manifesto é o ExtraAccountMetaList que você inicializou na lição passada, morando no seu programa de hook no PDA semeado pelo literal `extra-account-metas` e pelo mint. A convenção para ler ele e agir sobre ele se chama TLV Account Resolution, e a implementação de referência é o crate Rust `spl-tlv-account-resolution`; este curso fixa a 0.11.1, a mesma versão que o hook da lição passada fixa, e o layout de bytes abaixo foi lido do código-fonte desse crate e está inalterado da 0.11.0 até a 0.11.3, a mais nova no crates.io em 2026-09-01 (os patches 0.11.2/0.11.3 retrabalham as entranhas do resolvedor, não o formato serializado). O fluxo que toda carteira, DEX e script precisa executar:

![Fluxograma de uma transferência com hook: o cliente busca, decodifica e resolve o ExtraAccountMetaList e acrescenta os extras; pular a resolução reverte com um erro MissingAccount.](assets/v01-flowchart.webp)

Duas palavras dessa imagem merecem precisão, porque o código do lab depende delas.

Primeiro, a conta em si. Os dados da conta de validação são TLV, a mesma disciplina de tipo-comprimento-valor que você vem decodificando desde as lições de mint, mas não aponte o seu decodificador de mint para ela: as larguras do header diferem. O tipo de uma entrada de mint é um inteiro u16 pequeno (o seu inspetor imprimiu 18 e 19) com um comprimento u16; o tipo desta conta é um hash completo de 8 bytes (exatamente o `692565c54bfb661a` que você calculou acima, o discriminador do execute, porque esta lista responde à pergunta "do que o execute precisa") com um comprimento u32, depois o valor. O valor é um arrayzinho com prefixo de comprimento: uma contagem u32, depois essa quantidade de entradas de exatamente 35 bytes cada.

Segundo, a entrada. Cada ExtraAccountMeta é uma struct fixa de 35 bytes, e esses 35 bytes são uma pequena maravilha de compressão:

![Layout anotado do ExtraAccountMeta de 35 bytes, com um discriminador de um byte selecionando uma de quatro codificações, 32 bytes de endereço ou de config empacotada, e um byte para cada uma das flags de signer e de gravável.](assets/v02-annotated-code.webp)

Leia a linha do discriminador de novo, porque ela esconde a cilada mais afiada desta lição. O discriminador 0 é fácil: os 32 bytes são o endereço, pronto. O discriminador 1 diz: derive um PDA no programa do hook, usando seeds desempacotadas dos bytes de config. O discriminador 2 diz: leia a pubkey dos dados da instrução ou da conta; o harvest-hook nunca emite ele, e o resolvedor abaixo recusa ele em alto e bom som em vez de adivinhar. E 128 mais i diz: derive um PDA em qualquer programa que estiver no índice i da lista de contas da instrução Execute. Esse último significa que a resolução é posicional. As contas da instrução Execute são, em ordem: source no 0, mint no 1, destination no 2, owner no 3, a conta de validação no 4, depois cada extra na ordem da lista. Uma config de seed que diz "chave da conta 1" significa o mint porque o mint está no índice 1 daquela lista. Resolva as metas fora de ordem, ou derrube uma, e toda seed baseada em índice depois dela deriva um endereço diferente. Nada te avisa. O PDA derivado está simplesmente errado, a checagem on-chain falha, a transferência reverte. Ordem aqui não é uma convenção, é uma entrada do hash.

![A lista ordenada de contas da instrução execute, índices 0 a 6, onde omitir ou reordenar um extra desloca todo índice posterior e deriva o PDA errado.](assets/v03-diagram.webp)

Existe mais uma consequência de guardar o manifesto numa conta mutável: ela pode mudar. A autoridade do hook pode chamar update-extra-account-metas e remodelar a lista, e um cliente que cacheou a resolução dele da semana passada está agora encaminhando as contas da semana passada. Um conjunto desatualizado reverte exatamente como um faltando. Resolva fresco, a cada transferência, ou aceite que a sua integração quebra no dia em que o emissor encostar na lista. Na prática isso coloca a resolução no mesmo caminho de código que buscar o seu blockhash: as duas são leituras de atualidade contra o estado vivo da chain, as duas envelhecem do mesmo jeito, e uma transferência construída a partir de qualquer um dos dois valores cacheados é uma transferência construída para uma chain que não existe mais. O crate já vem com helpers de resolução do lado Rust, e o cliente JS de referência faz o mesmo trabalho; no lab você vai escrever o resolvedor você mesmo em TypeScript, porque depois de decodificar aqueles 35 bytes uma vez na mão, toda biblioteca de helper que você chamar na vida para de ser mágica.

### O hook que só sabe dizer não

Agora a outra metade da pergunta, a que o seu colega de time faz no momento em que você propõe entregar isso: a gente acabou de ligar código arbitrário em toda transferência do nosso token. O que impede um hook malicioso, ou o nosso próprio hook depois de um upgrade ruim, de drenar o remetente no meio da transferência? Ele tem a conta de origem. Ele tem o dono. O dono assinou.

A resposta não é uma promessa, são quatro linhas de construção no crate da interface, e eu quero que você veja as linhas de verdade em vez de confiar na minha paráfrase. É assim que a `spl-transfer-hook-interface` constrói a instrução Execute que o Token-2022 manda para o seu hook:

```rust
// spl-transfer-hook-interface, instruction.rs: the execute() builder.
let accounts = vec![
    AccountMeta::new_readonly(*source_pubkey, false),
    AccountMeta::new_readonly(*mint_pubkey, false),
    AccountMeta::new_readonly(*destination_pubkey, false),
    AccountMeta::new_readonly(*authority_pubkey, false),
    // …the builder then appends the validation account and the resolved extras.
];
```

Toda conta que veio da transferência original, a origem, o mint, o destino, o dono que assinou, é reconstruída como `AccountMeta::new_readonly(pubkey, false)`. Leia os dois argumentos como dois poderes revogados. Somente leitura: o hook não pode debitar a origem, creditar a si mesmo, nem mutar nenhum estado em que a transferência toca, porque o runtime impõe a gravabilidade por instrução, e esta instrução não concede nenhuma. O `false` é a flag de signer: mesmo que o dono tenha assinado a transação externa, aquela assinatura não é estendida ao frame do hook, então o hook não consegue se virar e fazer CPI no Token-2022 fingindo agir pelo dono. No modelo de privilégios de CPI, permissões só se estreitam conforme as chamadas descem. A interface escolheu o ajuste mais estreito para toda conta herdada, e essa escolha se chama de-escalação.

O estreitamento corre numa direção só, e a alternativa mostra por que tem que ser assim. Se um programa chamado pudesse escalar, então chamar qualquer programa significaria confiar em todo programa que ele pudesse chamar, transitivamente, para sempre. Então o runtime da Solana deixa um chamador conceder só privilégios que ele já tem, e sempre permite que ele conceda menos. A assinatura que você entregou para a sua carteira autoriza um frame; todo frame abaixo herda no máximo o que o frame acima escolheu passar adiante, e o Token-2022 escolheu não passar nada adiante. O argumento de segurança inteiro dos hooks se apoia nessa escolha, e você acabou de ler ele em quatro linhas do crate da interface.

![Na transferência externa a origem e o destino são graváveis e o dono assina, mas dentro do Execute as quatro contas chegam somente leitura e não signatárias.](assets/v04-diagram.webp)

Esse é o inventário de poderes da lição passada, observar, registrar no próprio território e vetar, agora lido nas próprias linhas da interface em vez de afirmado; a única adição que vale fazer é que o seu log de tesouraria funciona porque o PDA do log é uma conta do hook, declarada gravável nas metas, não herdada da transferência. Quando o seu colega de time fizer a pergunta da drenagem, a resposta de uma linha é que a interface entrega ao hook toda conta da transferência pré-despida para somente leitura e não signatária, então não tem nada que ele possa gastar e nenhuma assinatura que ele possa reusar.

A honestidade exige o outro lado do inventário, porém, porque "não consegue roubar" não é "não consegue machucar". Um veto é poder. Um hook que reverte incondicionalmente congela todo holder do token, permanentemente se o hook for imutável, arbitrariamente se a autoridade dele virar hostil. Um hook pode queimar compute: o Execute roda dentro do orçamento da transferência sem teto nenhum próprio aquém do limite da transação. E um hook malicioso pode, sim, mover fundos para fora de contas que o próprio programa dele controla; a garantia cobre só as contas da transferência. A de-escalação deixa o hook seguro para o saldo do remetente. Ela não deixa o hook seguro para a liveness do token, e ela não faz nada quanto ao custo. Segure as duas metades, porque o ecossistema com certeza segura.

### A divisão, com recibos

Então precifique a extensão com honestidade, das duas cadeiras. Da cadeira do emissor, um hook é controle por transferência: allowlists, logging, cancelas de compliance, qualquer política que caiba num programa. De toda outra cadeira à mesa, esse mesmo hook é uma fatura de imposto. Cada integrador precisa buscar, decodificar e encaminhar as contas extras certas em toda transferência, para sempre, e uma conta desatualizada ou faltando reverte a transação de um usuário no pior momento possível. A seção da de-escalação te disse que o hook não consegue roubar o integrador. Nada na interface impede que ele esgote o integrador.

Veja como os três atores mais instrutivos em produção precificam isso, e nenhum deles é hipotético:

![Quatro posturas de produção diante de transfer hooks: a Raydium recusa eles, a pump.fun estaciona um programa vazio no slot, o PYUSD deixa o program id nulo, e a DBC da Meteora encaminha.](assets/v05-comparison.webp)

Cada linha recompensa um olhar mais de perto. A Raydium é a direta: a referência Token-2022 dela rejeita o TransferHook porque ele "invoca um programa customizado em toda transferência, com consumo arbitrário de CU", e rejeita o PermanentDelegate no mesmo fôlego porque "um holder do delegado pode varrer qualquer conta de token, incluindo o vault da pool". Olhe as cinco extensões que DE FATO entraram na allowlist dela: uma config de taxa, duas formas de metadados, exibição de juros, exibição escalada. Todas elas são passivas. O padrão é a tese de compatibilidade do curso em miniatura: extensões que remodelam a exibição ou acumulam taxas entram na whitelist, extensões que rodam código ou têm poder sobre contas são recusadas, pelo nome, em produção.

A pump.fun é a engraçada, e aí ela deixa de ser engraçada. O transfer hook de produção deles é literalmente `#[program] pub mod transfer_hook_authority {}`: seis linhas de anchor-lang 0.31.1, deployado em `333UA891CYPpAJAthphPT3hg1EkUBLhNFoP9HoWW3nug`, sem nada dentro. Tome cuidado com o PORQUÊ, porque a leitura óbvia está errada e a m05-l1 coloca a correção na página: ninguém mais poderia ter reivindicado aquele slot de qualquer jeito. O slot de hook de um mint é definido na criação DELE por quem o cria, então não existe corrida para vencer nem nada para ocupar. O que a pump fez é mais estreito e mais interessante: eles apontaram o slot de hook dos próprios mints deles, que só existe no nascimento, para um programa que comprovadamente não faz nada, em vez de deixar o slot vazio para uma versão futura deles mesmos preencher com lógica viva. É um dispositivo de compromisso mirado no próprio futuro deles, e aquela câmara vazia protege volume real de transferência hoje: o slot é valioso o bastante para preencher e perigoso o bastante para preencher com nada. E repare no que até um hook vazio custaria se armado: no momento em que um program id de verdade aterrissar naquele slot, todo integrador do planeta deve a dança de resolução que a gente acabou de ensinar, mesmo que o Execute não faça nada. Um hook no-op não é de graça. O imposto é cobrado sobre o slot, não sobre a lógica.

![Um espectro de quatro estados do slot de transfer hook, de ausente passando por nulo até no-op e armado, com o imposto de resolução começando no momento em que um program id aterrissa.](assets/v06-diagram.webp)

O PYUSD, o lançamento emblemático do Token-2022 de maio de 2024, roda a mesma jogada pelo lado do compliance num mint de stablecoin regulada que carrega centenas de milhões de dólares de supply (774M de PYUSD numa leitura ao vivo, 2026-09-01): o mint carrega a extensão TransferHook com `transferHook.programId` definido como nulo. O slot está configurado, a arma está carregada, a câmara está vazia. Emissores recorrem a esse padrão porque ele preserva a opção de controle por transferência sem cobrar dos integradores de hoje o imposto; no challenge você relê aquele nulo com o seu próprio decodificador em vez do script emprestado que imprimiu ele pela primeira vez na m01-l1. E a DBC da Meteora é o contraponto que prova que encaminhar é possível em escala de protocolo: ela explicitamente encaminha as contas restantes de transfer hook em toda instrução de transferência. O imposto pode ser pago. A maioria dos venues simplesmente decidiu que o seu hook não vale a fatura.

Dê um passo atrás uma vez, porque o formato desse imposto não é uma esquisitice da Solana, é o custo de uma escolha de design que você já conhece. A Solana exige que toda conta seja declarada antes da execução; é isso que torna o escalonamento paralelo possível, e é por isso que a transferência p-token que você mediu no módulo um roda em CU de dois dígitos. Uma chain com despacho dinâmico não cobra imposto de resolução nenhum: o ERC-777 do Ethereum deixava contratos de token chamarem hooks de recebedor sem nada pré-declarado, e integradores pagaram zero adiantado. Eles pagaram depois. Aqueles hooks conseguiam reentrar no contrato chamador no meio da transferência, e a pool imBTC na Uniswap V1 foi drenada exatamente por essa porta em abril de 2020. O transfer hook é a mesma ideia com os modos de falha deslocados: o modelo de contas declaradas exporta a contabilidade para o cliente, o que é chato, e em troca o hook chega num frame em que ele não consegue tocar em nada, que é a garantia que o ERC-777 nunca teve. Você paga o imposto em código de integração em vez de em exploits. Qual lado dessa troca parece melhor depende de em qual cadeira você senta, e a tabela acima é o mercado votando de todas as cadeiras ao mesmo tempo.

![O despacho dinâmico do ERC-777 não custou nada adiantado aos integradores, mas permitiu a drenagem por reentrância do imBTC, enquanto o Token-2022 cobra um imposto de resolução por um frame de hook somente leitura e não signatário.](assets/v07-comparison.webp)

Esse é o trade-off, dito da forma mais direta que eu consigo: um hook compra para o emissor controle por transferência e exporta um imposto de resolução para todo integrador rio abaixo, e a preferência revelada do mercado é recusar o imposto. Se o SPROUT precisa ser negociado nos venues principais, o hook é a extensão da qual você mais vai se arrepender. Se o SPROUT é um instrumento permissionado onde você controla os venues, o hook é exatamente a ferramenta certa. O curso Payments & Commerce fica do segundo lado dessa linha: o trilho de assinaturas dele consome esse mesmo comportamento de encaminhamento de hook contra um fluxo controlado pelo lojista, e ele aponta de volta para esta lição em vez de re-derivar ela.

## Lab: ensine uma carteira a carregar as contas do seu hook

O plano: suba um surfnet local, faça o deploy do harvest-hook da lição passada, cunhe uma variante fresca de SPROUT com hook, depois mande o mesmo TransferChecked duas vezes, uma vez nu e uma vez resolvido. Tudo que é TypeScript abaixo passou por type-check em modo strict contra a toolchain fixada em 2026-08-22, e o caminho de decode do resolvedor foi adicionalmente testado contra bytes de lista sintéticos construídos na mão; um punhado de números de chain ao vivo nesta lição foi relido em 2026-09-01 durante a edição final, e onde quer que uma data importe ela é impressa ao lado do número dela. As partes que precisam do seu hook deployado são as partes que só você pode rodar.

1. **Suba um surfnet local.** O Surfpool te dá um validador local que se comporta como a mainnet e não precisa de chave nenhuma; é a mesma ferramenta que você instalou lá na m02-l1 (surfpool 1.2.1 na minha máquina, verificado em 2026-08-22; `curl -sL https://run.surfpool.run/ | bash` se esta máquina ainda não tiver ele):

   ```bash
   surfpool --version
   surfpool start --no-tui --no-studio
   ```

   Deixe rodando. O RPC aterrissa em `http://127.0.0.1:8899`, os websockets em `ws://127.0.0.1:8900`. Num segundo terminal, aponte a CLI `solana` (ela veio junto com a instalação da toolchain Agave da lição passada) para ele e garanta que o seu keypair default tem lamports: `solana config set --url http://127.0.0.1:8899`, depois `solana airdrop 100` se o seu saldo for zero.

2. **Faça o deploy do harvest-hook.** A partir do repositório do programa da lição passada (o build com anchor-lang 1.1.2; a camada de framework em si é o curso Master Anchor V2, a gente só entrega o programa). Um passo de reconciliação primeiro, porque esta é a cilada clássica do auto-deploy: o código da lição passada deixa `declare_id!("HookH1FQuTU21GVAjJZDLXPjXWLQFPJ5FLpwGKZLkYQ")` hardcoded, e nenhum keypair para esse endereço existe na sua máquina. O `solana program deploy` faz o deploy no endereço do keypair autogerado em `target/deploy/harvest_hook-keypair.json`, então o id de runtime e o id declarado ficariam em desacordo, e o entrypoint gerado pelo Anchor rejeita toda chamada com `DeclaredProgramIdMismatch`. (A sua bancada LiteSVM nunca esbarrou nisso porque ela carregava o `.so` em `harvest_hook::ID` direto; um deploy de verdade não tem essa cortesia.) Sincronize os dois antes de fazer o deploy:

   ```bash
   solana address -k target/deploy/harvest_hook-keypair.json
   # paste that address into declare_id!(...) in src/lib.rs, then:
   cargo build-sbf
   solana program deploy target/deploy/harvest_hook.so
   ```

   Copie o program id impresso, que agora bate com o seu `declare_id!`; ele é `$HOOK` pelo resto do lab.

3. **Cunhe a variante do SPROUT com hook.** Você provou na lição passada que o TransferHook só existe na criação, então a gente cunha do zero em vez de adaptar depois. A CLI `spl-token` da m01-l4 dá conta da cerimônia inteira (spl-token-cli 5.6.1 no crates.io em 2026-08-22):

   ```bash
   spl-token create-token --program-2022 --decimals 6 --transfer-hook $HOOK
   # copy the mint address -> $MINT
   spl-token create-account $MINT
   spl-token mint $MINT 1000
   solana-keygen new --no-bip39-passphrase -o stranger.json
   # copy the stranger's pubkey -> $DEST
   spl-token create-account $MINT --owner $DEST
   ```

   Checkpoint: `spl-token display $MINT` mostra a extensão TransferHook com o seu program id dentro dela. Repare no que acabou de acontecer silenciosamente: as duas contas de token foram criadas com a extensão TransferHookAccount, porque um mint com hook força ela em todo holder.

4. **Rearme o estado on-chain do hook.** Dentro do LiteSVM você inicializou o ExtraAccountMetaList na bancada; este surfnet nunca viu ele. A chamada de initialize é padrão da interface, então eu consigo te entregar ela byte a byte. Monte um workspace de cliente primeiro (o kit fixado em 7.1.1 exatamente, o mesmo pin de faixa de peers que o workspace do curso monta; repare que a linha kit-^7 do `@solana-program/token-2022` é a 0.15.0, que a gente deliberadamente não precisa, tudo abaixo é kit puro):

   ```bash
   mkdir sprout-client && cd sprout-client
   npm init -y && npm pkg set type=module
   npm install @solana/kit@7.1.1
   npm install -D tsx@4.20.5
   export MINT=... HOOK=... DEST=...
   ```

   Salve `init-metas.ts`:

   ```typescript
   // init-metas.ts: one call to the harvest-hook's own
   // initialize-extra-account-metas instruction, so the validation account
   // exists on the surfnet the way it existed inside last lesson's harness.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstruction,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';
   import { getExtraAccountMetaAddress } from './resolve.js';

   const SYSTEM_PROGRAM = address('11111111111111111111111111111111');
   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   const validationAccount = await getExtraAccountMetaAddress(MINT, HOOK);

   // Same hashed-string discriminator scheme as execute:
   // sha256("spl-transfer-hook-interface:initialize-extra-account-metas")[0..8].
   const initDiscriminator = new Uint8Array(
     createHash('sha256')
       .update('spl-transfer-hook-interface:initialize-extra-account-metas')
       .digest()
       .subarray(0, 8),
   );

   const ix: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: validationAccount, role: AccountRole.WRITABLE },
       { address: MINT, role: AccountRole.READONLY },
       { address: payer.address, role: AccountRole.WRITABLE_SIGNER },
       { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },
     ],
     data: initDiscriminator,
   };

   const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
   const tx = await signTransactionMessageWithSigners(
     pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstruction(ix, m),
     ),
   );
   assertIsTransactionWithBlockhashLifetime(tx);
   await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
     commitment: 'confirmed',
   });
   console.log('ExtraAccountMetaList initialized at', validationAccount);
   ```

   Um dos imports, `getExtraAccountMetaAddress`, vem do `resolve.js`, que você escreve no próximo passo, então não rode nada ainda. Mais duas chamadas de gestão NÃO são padrão da interface, e as duas precisam rodar antes de qualquer transferência: o `initialize` do seu próprio hook, depois a allowlist dele. Este surfnet nunca viu os seus PDAs de hook-config ou de treasury-log, assim como não tinha visto a lista de metas; o `allow_destination` não consegue rodar sem a config (a checagem `has_one` dele desserializa ela), e o Execute do Ato 2 desserializa os dois PDAs com os bumps armazenados deles, então pular o `initialize` mata a transferência resolvida com a mesma certeza que uma lista de metas faltando. As duas instruções pertencem à API do seu próprio programa da lição passada, e como nada até aqui te mostrou como codificar na mão uma instrução Anchor com argumento, esta também é trabalhada. Dois fatos carregam o script inteiro: um discriminador Anchor deriva como `sha256("global:<method_name>")[0..8]`, o mesmo truque de string hasheada da interface sob um namespace diferente, e um argumento `Pubkey` codificado em borsh é só os 32 bytes crus dele acrescentados depois do discriminador. Preste atenção em QUAL chave você coloca na allowlist: a cancela que você escreveu compara `ctx.accounts.destination.key()`, a CONTA DE TOKEN de destino no índice 2 do Execute, então a entrada a adicionar é a ATA do estranho, não a pubkey da carteira dele (`spl-token address --token $MINT --owner $DEST --verbose` imprime ela; exporte ela como `$DEST_ATA`). Salve `init-hook.ts`:

   ```typescript
   // init-hook.ts: the two non-interface management calls, hand-encoded.
   // initialize (no args), then allow_destination(destination: Pubkey).
   import {
     AccountRole,
     address,
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getAddressEncoder,
     getProgramDerivedAddress,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';

   const SYSTEM_PROGRAM = address('11111111111111111111111111111111');
   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);
   const DEST_ATA = address(process.env.DEST_ATA!); // the stranger's TOKEN ACCOUNT

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const enc = getAddressEncoder();

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   // Anchor namespace, not the interface namespace: sha256("global:<name>")[0..8].
   const anchorDisc = (name: string): Uint8Array =>
     new Uint8Array(createHash('sha256').update(`global:${name}`).digest().subarray(0, 8));

   const pda = async (seed: string) => {
     const [addr] = await getProgramDerivedAddress({
       programAddress: HOOK,
       seeds: [seed, enc.encode(MINT)],
     });
     return addr;
   };
   const config = await pda('hook-config');
   const treasury = await pda('treasury');

   // initialize: no args; accounts in the Initialize struct's exact order.
   const initIx: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: payer.address, role: AccountRole.WRITABLE_SIGNER },
       { address: MINT, role: AccountRole.READONLY },
       { address: config, role: AccountRole.WRITABLE },
       { address: treasury, role: AccountRole.WRITABLE },
       { address: SYSTEM_PROGRAM, role: AccountRole.READONLY },
     ],
     data: anchorDisc('initialize'),
   };

   // allow_destination(destination: Pubkey): 8-byte discriminator + 32 raw
   // borsh bytes of the pubkey. The Manage struct's order: authority, config.
   const allowData = new Uint8Array(40);
   allowData.set(anchorDisc('allow_destination'), 0);
   allowData.set(new Uint8Array(enc.encode(DEST_ATA)), 8);
   const allowIx: Instruction = {
     programAddress: HOOK,
     accounts: [
       { address: payer.address, role: AccountRole.READONLY_SIGNER },
       { address: config, role: AccountRole.WRITABLE },
     ],
     data: allowData,
   };

   const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
   const tx = await signTransactionMessageWithSigners(
     pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions([initIx, allowIx], m),
     ),
   );
   assertIsTransactionWithBlockhashLifetime(tx);
   await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
     commitment: 'confirmed',
   });
   console.log('hook config + treasury initialized; allowlisted', DEST_ATA);
   ```

   Ordem de execução assim que o `resolve.ts` existir no próximo passo: `npx tsx init-metas.ts`, depois `npx tsx init-hook.ts`. Se você renomeou métodos, seeds, ou reordenou as structs de conta no seu próprio harvest-hook, ajuste este script para casar com o seu programa, não o contrário. Faça tudo isso antes do ato um, porque o ponto do próximo ato é que o encaminhamento, e só o encaminhamento, está entre o estranho e os tokens dele.

5. **Escreva o resolvedor.** Este é o coração do lab, e é uma transcrição direta em TypeScript do que a `spl-tlv-account-resolution` 0.11.1 faz em Rust. Salve `resolve.ts`:

   ```typescript
   // resolve.ts: client-side TLV Account Resolution for a Token-2022 transfer hook.
   // Mirrors what spl-tlv-account-resolution 0.11.1 does in Rust, byte for byte.
   import {
     AccountRole,
     getAddressDecoder,
     getAddressEncoder,
     getProgramDerivedAddress,
     type Address,
     type AccountMeta,
     type Rpc,
     type SolanaRpcApi,
   } from '@solana/kit';
   import { createHash } from 'node:crypto';

   const enc = getAddressEncoder();
   const dec = getAddressDecoder();

   // The TLV entry we are looking for is typed by the execute instruction's
   // hashed-string discriminator: sha256("spl-transfer-hook-interface:execute")[0..8].
   export const EXECUTE_DISCRIMINATOR = new Uint8Array(
     createHash('sha256')
       .update('spl-transfer-hook-interface:execute')
       .digest()
       .subarray(0, 8),
   ); // 69 25 65 c5 4b fb 66 1a

   // One validation account per mint, on the HOOK program:
   // seeds = ["extra-account-metas", mint].
   export async function getExtraAccountMetaAddress(
     mint: Address,
     hookProgram: Address,
   ): Promise<Address> {
     const [pda] = await getProgramDerivedAddress({
       programAddress: hookProgram,
       seeds: ['extra-account-metas', enc.encode(mint)],
     });
     return pda;
   }

   // The fixed 35-byte entry: discriminator u8 | address_config [u8;32] |
   // is_signer u8 | is_writable u8.
   export interface ExtraAccountMeta {
     discriminator: number;
     addressConfig: Uint8Array;
     isSigner: boolean;
     isWritable: boolean;
   }

   // Account layout: [8-byte TLV type][u32 LE length][u32 LE count][count * 35 bytes].
   export function unpackExtraAccountMetaList(data: Uint8Array): ExtraAccountMeta[] {
     const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
     let offset = 0;
     while (offset + 12 <= data.length) {
       const tlvType = data.subarray(offset, offset + 8);
       const length = view.getUint32(offset + 8, true);
       if (equalBytes(tlvType, EXECUTE_DISCRIMINATOR)) {
         const count = view.getUint32(offset + 12, true);
         const entries = data.subarray(offset + 16, offset + 16 + count * 35);
         const metas: ExtraAccountMeta[] = [];
         for (let i = 0; i < count; i++) {
           const e = entries.subarray(i * 35, (i + 1) * 35);
           metas.push({
             discriminator: e[0],
             addressConfig: e.subarray(1, 33),
             isSigner: e[33] === 1,
             isWritable: e[34] === 1,
           });
         }
         return metas;
       }
       offset += 12 + length;
     }
     throw new Error('no TLV entry under the execute discriminator: is the list initialized?');
   }

   // Resolution is positional against the EXECUTE instruction, which the runtime
   // will build as: [0] source, [1] mint, [2] destination, [3] owner,
   // [4] validation account, then each extra in list order. Seeds that reference
   // "account at index N" mean N in THAT list, which is why order is not optional.
   export async function resolveTransferHookAccounts(
     rpc: Rpc<SolanaRpcApi>,
     mint: Address,
     hookProgram: Address,
     source: Address,
     destination: Address,
     owner: Address,
     amount: bigint,
   ): Promise<AccountMeta[]> {
     const validationAccount = await getExtraAccountMetaAddress(mint, hookProgram);
     const { value: account } = await rpc
       .getAccountInfo(validationAccount, { encoding: 'base64' })
       .send();
     if (!account) {
       throw new Error(`validation account ${validationAccount} does not exist on this cluster`);
     }
     const raw = Uint8Array.from(Buffer.from(account.data[0], 'base64'));
     const metas = unpackExtraAccountMetaList(raw);

     const executeKeys: Address[] = [source, mint, destination, owner, validationAccount];
     const executeData = buildExecuteData(amount);

     const resolved: AccountMeta[] = [];
     for (const meta of metas) {
       const resolvedAddress = await resolveMeta(meta, executeKeys, executeData, hookProgram);
       executeKeys.push(resolvedAddress);
       resolved.push({
         address: resolvedAddress,
         role: meta.isSigner
           ? meta.isWritable
             ? AccountRole.WRITABLE_SIGNER
             : AccountRole.READONLY_SIGNER
           : meta.isWritable
             ? AccountRole.WRITABLE
             : AccountRole.READONLY,
       });
     }

     // Canonical append order for the transferring instruction:
     // the resolved extras in list order, then the hook program, then the
     // validation account. This mirrors the reference client helper.
     return [
       ...resolved,
       { address: hookProgram, role: AccountRole.READONLY },
       { address: validationAccount, role: AccountRole.READONLY },
     ];
   }

   async function resolveMeta(
     meta: ExtraAccountMeta,
     executeKeys: Address[],
     executeData: Uint8Array,
     hookProgram: Address,
   ): Promise<Address> {
     // discriminator 0: address_config IS the pubkey.
     if (meta.discriminator === 0) {
       return dec.decode(meta.addressConfig);
     }
     // discriminator 1: PDA on the hook program.
     // discriminator 128 + i: PDA of the program whose address sits at
     // execute-instruction account index i.
     if (meta.discriminator === 1 || meta.discriminator >= 128) {
       const programAddress =
         meta.discriminator === 1 ? hookProgram : executeKeys[meta.discriminator - 128];
       if (!programAddress) {
         throw new Error(`seed program index ${meta.discriminator - 128} is out of range`);
       }
       const seeds = unpackSeeds(meta.addressConfig, executeKeys, executeData);
       const [pda] = await getProgramDerivedAddress({ programAddress, seeds });
       return pda;
     }
     // discriminator 2 (pubkey stored in account/instruction data) exists in the
     // crate but the harvest-hook never uses it; fail loudly instead of guessing.
     throw new Error(`ExtraAccountMeta discriminator ${meta.discriminator} not supported here`);
   }

   // address_config for a PDA holds packed seed configs, zero-padded to 32 bytes:
   // 1 = literal (len, bytes), 2 = instruction-data slice (index, length),
   // 3 = account key at index, 4 = account-data slice.
   function unpackSeeds(
     config: Uint8Array,
     executeKeys: Address[],
     executeData: Uint8Array,
   ): Uint8Array[] {
     const seeds: Uint8Array[] = [];
     let i = 0;
     while (i < 32) {
       const tag = config[i];
       if (tag === 0) break; // zero padding: no more seeds
       if (tag === 1) {
         const len = config[i + 1];
         seeds.push(config.subarray(i + 2, i + 2 + len));
         i += 2 + len;
       } else if (tag === 2) {
         const index = config[i + 1];
         const length = config[i + 2];
         seeds.push(executeData.subarray(index, index + length));
         i += 3;
       } else if (tag === 3) {
         const index = config[i + 1];
         const key = executeKeys[index];
         if (!key) {
           throw new Error(`seed wants account index ${index}, which is not resolved yet`);
         }
         seeds.push(new Uint8Array(enc.encode(key)));
         i += 2;
       } else {
         // tag 4 reads another account's data; the harvest-hook does not use it.
         throw new Error(`seed config tag ${tag} not implemented in this lab`);
       }
     }
     return seeds;
   }

   // The execute instruction's data, needed for instruction-data seeds:
   // [8-byte execute discriminator][u64 LE amount].
   function buildExecuteData(amount: bigint): Uint8Array {
     const data = new Uint8Array(16);
     data.set(EXECUTE_DISCRIMINATOR, 0);
     new DataView(data.buffer).setBigUint64(8, amount, true);
     return data;
   }

   function equalBytes(a: Uint8Array, b: Uint8Array): boolean {
     return a.length === b.length && a.every((byte, i) => byte === b[i]);
   }
   ```

   Três detalhes merecem uma segunda leitura. O array `executeKeys` começa como as cinco contas base e cresce conforme cada extra resolve, que é exatamente como seeds baseadas em índice podem legitimamente referenciar um extra anterior: ordem, de novo, é uma entrada. O mapeamento de role mantém quaisquer flags de signer e de gravável que as metas declararam para as contas PRÓPRIAS do hook, é assim que o seu log de tesouraria continua gravável. E o `resolveMeta` recusa o discriminador 2 em alto e bom som em vez de adivinhar; o crate suporta uma variante de pubkey-nos-dados que o harvest-hook nunca usa, e um resolvedor que trata mal uma codificação silenciosamente é pior do que um que para.

6. **Rode o conto de duas transferências.** Agora o script da recompensa, e repare que o revert on-chain vai nomear a falha para você. Se você ainda não rodou o `init-metas.ts` no passo 4, rode ele agora (`npx tsx init-metas.ts`; o tsx roda TypeScript direto, e o passo 4 colocou o pin 4.20.5 que a m01-l1 introduziu nas devDependencies deste próprio workspace em vez de confiar no que quer que o `npx` ache). Rodar ele uma SEGUNDA vez não é inofensivo e a falha é opaca: o `init` do Anchor lá dentro cria a conta de validação, então uma re-execução volta como um mero `Custom program error: #0`, que é `AccountAlreadyInUse` e significa que a conta que você queria já existe. Então salve `transfer.ts`:

   ```typescript
   // transfer.ts: the same TransferChecked, twice: once the way a naive wallet
   // builds it (simulated, watch it die), once with the resolved extras appended.
   import {
     AccountRole,
     address,
     appendTransactionMessageInstruction,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getAddressEncoder,
     getBase64EncodedWireTransaction,
     getProgramDerivedAddress,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Address,
     type AccountMeta,
     type Instruction,
   } from '@solana/kit';
   import { readFileSync } from 'node:fs';
   import { homedir } from 'node:os';
   import { resolveTransferHookAccounts } from './resolve.js';

   const TOKEN_2022 = address('TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb');
   const ATA_PROGRAM = address('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

   const MINT = address(process.env.MINT!);
   const HOOK = address(process.env.HOOK!);
   const DEST_OWNER = address(process.env.DEST!);
   const AMOUNT = 5_000_000n; // 5 SPROUT at 6 decimals
   const DECIMALS = 6;

   const rpc = createSolanaRpc('http://127.0.0.1:8899');
   const rpcSubscriptions = createSolanaRpcSubscriptions('ws://127.0.0.1:8900');
   const enc = getAddressEncoder();

   const payer = await createKeyPairSignerFromBytes(
     new Uint8Array(JSON.parse(readFileSync(`${homedir()}/.config/solana/id.json`, 'utf8'))),
   );

   async function ata(owner: Address): Promise<Address> {
     const [addr] = await getProgramDerivedAddress({
       programAddress: ATA_PROGRAM,
       seeds: [enc.encode(owner), enc.encode(TOKEN_2022), enc.encode(MINT)],
     });
     return addr;
   }

   const source = await ata(payer.address);
   const destination = await ata(DEST_OWNER);

   // TransferChecked is Token-2022 instruction 12: [12][amount u64 LE][decimals u8].
   const data = new Uint8Array(10);
   data[0] = 12;
   new DataView(data.buffer).setBigUint64(1, AMOUNT, true);
   data[9] = DECIMALS;

   const baseAccounts: AccountMeta[] = [
     { address: source, role: AccountRole.WRITABLE },
     { address: MINT, role: AccountRole.READONLY },
     { address: destination, role: AccountRole.WRITABLE },
     { address: payer.address, role: AccountRole.READONLY_SIGNER },
   ];

   async function buildAndSign(accounts: AccountMeta[]) {
     const ix: Instruction = { programAddress: TOKEN_2022, accounts, data };
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const message = pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(payer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstruction(ix, m),
     );
     return await signTransactionMessageWithSigners(message);
   }

   // Act 1: the naive transfer. Four accounts, exactly what a hookless wallet sends.
   const naive = await buildAndSign(baseAccounts);
   const sim = await rpc
     .simulateTransaction(getBase64EncodedWireTransaction(naive), { encoding: 'base64' })
     .send();
   // kit decodes numeric RPC fields as bigints unless the field is on its
   // allowed-numeric list, and the instruction index inside InstructionError
   // is not on it. JSON.stringify throws TypeError on any bigint it meets, so
   // without this replacer act 1 dies right here with a stack trace pointing
   // at your own file, and act 2 never runs. Number() is safe for an
   // instruction index and keeps it unquoted in the output.
   //
   // Worth knowing WHY this is a kit problem specifically: m01-l1 stringified
   // the same InstructionError shape with no replacer and no crash, because it
   // spoke to the RPC with a bare fetch and JSON.parse never produces bigints.
   // The hazard arrives with the typed client, not with the JSON.
   const bigintSafe = (_key: string, value: unknown): unknown =>
     typeof value === 'bigint' ? Number(value) : value;
   console.log('naive transfer err:', JSON.stringify(sim.value.err, bigintSafe));
   for (const line of sim.value.logs ?? []) console.log('  ', line);

   // Act 2: resolve the hook's extras off-chain and forward them.
   const extras = await resolveTransferHookAccounts(
     rpc, MINT, HOOK, source, destination, payer.address, AMOUNT,
   );
   console.log('\nforwarding', extras.length, 'extra accounts:');
   for (const meta of extras) console.log('  ', meta.address, 'role', meta.role);

   const resolved = await buildAndSign([...baseAccounts, ...extras]);
   assertIsTransactionWithBlockhashLifetime(resolved);
   const sendAndConfirm = sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions });
   await sendAndConfirm(resolved, { commitment: 'confirmed' });
   console.log('\nresolved transfer landed:', getSignatureFromTransaction(resolved));
   ```

   Rode: `npx tsx transfer.ts`. Checkpoint, ato um: o erro da simulação deve dizer `{"InstructionError":[0,"MissingAccount"]}`, com uma linha de log nomeando o seu hook como `Unknown program $HOOK` e uma linha final dizendo `An account required by the instruction is missing`. Leia qual conta está faltando, porque não é a que a maioria das pessoas chuta. O Token-2022 procura a conta de validação entre as contas que você mandou, não acha ela, e por isso nunca resolve nada; aí ele faz CPI num programa de hook que também não está na transação, e o runtime não consegue invocar um programa que nunca lhe foi entregue. Essa é a falha com formato de carteira da abertura desta lição, reproduzida sob demanda. Checkpoint, ato dois: uma lista impressa de contas encaminhadas (os extras do seu hook, depois o programa do hook, depois a conta de validação) seguida de `resolved transfer landed:` e uma assinatura. Confirme com `spl-token balance $MINT --owner $DEST`: o estranho tem 5. Mesma instrução, mesmo signer, mesmo valor; a única diferença entre a morte e a aterrissagem foi a lista de contas.

![O TransferChecked ingênuo de quatro contas reverte, enquanto a versão resolvida acrescenta os extras do hook, o programa do hook e a conta de validação com dados de instrução idênticos.](assets/v08-comparison.webp)

7. **Veja um cliente que resolve fazer isso por você.** Uma última sondagem, agora que você sabe o que a maquinaria custa:

   ```bash
   spl-token transfer $MINT 1 $DEST --allow-unfunded-recipient
   ```

   Essa flag não é sobre hooks e ela não é opcional aqui: o passo 3 gerou a chave do estranho sem financiar ele, e a CLI dá erro duro do lado do cliente num destinatário sem SOL antes de construir qualquer coisa. Tire a flag e você ganha uma recusa que não tem nada a te ensinar sobre transfer hooks. Com ela, a transferência aterrissa sem nenhuma flag específica de hook, porque a CLI é um integrador bem-comportado que roda essa mesma resolução online antes de mandar (a flag `--transfer-hook-account` dela existe para assinatura offline, onde não tem RPC ali para resolver contra). Toda ferramenta que "simplesmente funciona" com o seu token com hook está silenciosamente pagando o imposto que você acabou de itemizar.

## Challenge

Solo, três partes, sem apoio.

Primeiro, quebre a transferência resolvida de três jeitos e diagnostique cada um pela camada que matou ele. Mande de novo a transferência ingênua de quatro contas do ato um. Depois mande uma com tudo encaminhado MENOS a última conta acrescentada, o PDA de validação. Depois mande uma totalmente encaminhada para um destino que você NÃO colocou na allowlist. Os três morrem, e não tem dois que morram no mesmo lugar: um nunca chega no seu programa, um chega nele e é rejeitado antes de a sua lógica rodar, e um é o seu próprio veto disparando. Escreva uma frase por falha nomeando a camada e citando a linha de log que prova isso. Dois avisos, porque o caso do meio é a cilada: a mensagem de erro dele contém a frase "not enough account keys", que soa como o runtime reclamando mas não é, e o sinal mais seguro do primeiro caso é uma linha que está ausente em vez de presente. Se você quiser uma dica de por que essa distinção importa operacionalmente, repare em qual dos três os seus usuários culpariam a carteira deles e em qual culpariam o seu token.

Segundo, o critério de avaliação, em uma linha: a partir da de-escalação para somente leitura, diga por que o seu hook, ou qualquer hook, não consegue gastar os tokens do remetente. Se a sua frase não mencionar os dois poderes revogados, a gravabilidade e a assinatura, ela ainda não é a resposta inteira.

Terceiro, leve isso para a mainnet. Aponte o seu inspetor `decode-mint` da m01-l2 para o mint do PYUSD, `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`, caminhe pelo TLV até a entrada TransferHook, e leia o program id: trinta e dois bytes zero, o nulo que deixa a extensão inteira dormente. O script emprestado da m01-l1 imprimiu esse nulo para você no primeiro dia; hoje você verificou ele byte a byte com um decodificador que você escreveu, numa stablecoin regulada de nove dígitos, e você sabe exatamente o que mudaria para todo integrador de PYUSD na Terra no dia em que aquele campo deixar de ser zero.

Se algum checkpoint aqui imprimiu algo que o meu não imprimiu, ou se o layout de contas do seu hook te forçou a adaptar um script, sinalize isso no canal de feedback do curso com o comando e a saída colados. Bugs de resolução são exatamente a classe de falha que só aparece em máquinas de verdade, e uma execução quebrada de um leitor ensina mais a este curso do que uma limpa.

O próximo módulo aumenta a aposta sobre o que uma transferência sequer vai mostrar. Você acabou de provar que um hook não consegue agir sobre tokens que ele não controla; a próxima extensão vai além e esconde o valor em si. Saldos confidenciais criptografam o número que se move dentro de um envelope lacrado que só o remetente, o recebedor e um auditor opcional conseguem abrir, e fique avisado antes de esboçar um design que quer os dois: o pareamento é traiçoeiro. Uma transferência confidencial ainda invoca o seu hook, mas entrega a ele um valor sentinela (`u64::MAX`) em vez do número real, então qualquer hook cuja lógica restrinja com base em valores fica cego exatamente quando o valor está escondido. É para lá que a gente vai agora: como o Token-2022 move valor que ele se recusa a mostrar.
