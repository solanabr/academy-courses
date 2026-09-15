# Estudos de caso em produção e a fronteira: o que de fato foi entregue

## Resumo

Na lição passada você fechou a economia da Overgrowth. A cancela do alpha responde "esta carteira tem um cNFT Founding-Farmer?" a partir de uma leitura do DAS em vez da promessa de um cliente, e pontos de compost viraram SPROUT de verdade pelo caminho de Merkle, claim marcado, segundo claim rejeitado. Mint, cNFT, airdrop, leitor, cancela. Cada peça que você construiu agora encosta em todas as outras.

Então aqui está a pergunta que fecha o arco: alguém de fato entrega essas coisas, ou você passou nove módulos aprendendo uma especificação?

Responda você mesmo, agora, antes de ler mais um parágrafo. Isto lê uma conta da mainnet e não precisa de nada instalado:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
  | grep -o '"extension":"[a-zA-Z]*"'
```

Voltam oito linhas. `mintCloseAuthority`, `permanentDelegate`, `transferFeeConfig`, `confidentialTransferMint`, `confidentialTransferFeeConfig`, `transferHook`, `metadataPointer`, `tokenMetadata`. Isso é o PayPal USD, lançado na Solana em maio de 2024 pela PayPal e pela Paxos, carregando oito extensões TLV sobre um dólar regulado ao vivo. Você leu essas mesmas oito na primeira lição deste curso, quando elas eram formas que você não conseguia explicar. Agora você consegue explicar as oito.

Aqui está o que você não conseguia fazer na época e vai fazer hoje. Quatro dessas oito estão desligadas. Não ausentes. Desligadas, com o interruptor ainda ligado na fiação e a chave de alguém ainda nele. Essa lacuna entre "a extensão está presente nos bytes" e "a extensão faz alguma coisa" é onde mora toda avaliação de verdade de um token entregue, e ler isso errado em qualquer das duas direções é como as pessoas se machucam.

O caminho por esta lição: primeiro o que o mint do PYUSD diz hoje e como derivar ativo-versus-dormente a partir de valores em vez de presença, depois a economia de um slot armado, que é a parte que ninguém precifica. Depois três coisas entregues medidas contra o que você construiu: o drop do JTO da Jito contra o seu caminho de Merkle, os trilhos de stablecoin que pagam por tudo isso, e o trabalho de identidade de agente que é genuinamente novo e genuinamente não comprovado. Você sai com o `dormancy-report.ts`, uma ferramenta que aponta para qualquer mint e diz quais dos poderes dele estão vivos.

O recuo da ajuda: o classificador é trabalhado por inteiro para as extensões que o PYUSD carrega, você escreve a regra para uma que ele não tem, e o memorando no fim é inteiramente seu. Esse memorando é a peça em que você se avalia, e ele tem a mesma forma do que o seu capstone vai pedir semana que vem.

## Armada, não disparada

### O que oito slots dizem hoje

Presença é barata de ler. Comportamento não é, e os quatro campos que carregam comportamento estão espalhados por quatro corpos de extensão diferentes com quatro nomes diferentes. Comece pelas duas em que o estudo de caso inteiro se apoia, lidas ao vivo da mainnet em 2026-09-01:

O `transferHook.programId` é `null`. Existe um slot de hook no mint do PYUSD, e nenhum programa dentro dele. Nada é invocado na transferência, não porque o Token-2022 recuse, mas porque o emissor não nomeou um programa para invocar. A sua própria lição de hook construiu o outro lado desse slot: um programa que recebe CPI em toda transferência e consegue rejeitar uma. O PYUSD tem a tomada e nenhum plugue.

O `transferFeeConfig` lê 0 basis points, taxa máxima 0, tanto na entrada de taxa mais velha quanto na mais nova, as duas carimbadas com a epoch 605, que é o jeito da extensão de dizer que nenhuma mudança de alíquota jamais foi agendada contra este mint e que toda transferência de PYUSD que já liquidou reteve exatamente nada. A extensão que deixaria um emissor regulado raspar uma fatia de todo movimento de um dólar está presente e configurada em zero.

Mais duas que leem como inertes assim que você olha além do rótulo. O `confidentialTransferMint` tem `autoApproveNewAccounts: false` e nenhuma auditor key, o que significa que nenhuma conta ganha saldos confidenciais até o emissor aprovar aquela conta específica. Os trilhos existem. A catraca está travada. E o `confidentialTransferFeeConfig` carrega um ciphertext retido só de zeros, que é exatamente o que você esperaria de uma tabela de taxa que nunca cobrou nada.

![Uma leitura JSON reduzida do mint do PYUSD traz chamadas no programId do hook, na entrada de taxa, nas flags de transferência confidencial e no delegado permanente, quatro valores inertes e um poder vivo.](assets/v01-annotated-code.png)

Agora o campo que não é nada inerte. O `permanentDelegate.delegate` nomeia um endereço, e o mesmo vale para o `mintCloseAuthority.closeAuthority`, o `metadataPointer.metadataAddress` e o corpo do `tokenMetadata` que resolve para "PayPal USD". Esses quatro fazem alguma coisa hoje. Um delegado permanente pode mover ou queimar PYUSD de qualquer conta sem o dono assinar, que é a forma on-chain de uma ordem judicial, e ele está ligado agora mesmo.

Quatro vivas, quatro dormentes, um mint.

### Presente, ativo e exercível são três perguntas

O reflexo que a maioria das pessoas tem é binário: a extensão está lá, então o token faz aquilo. Esse reflexo está errado nas duas direções e eu já errei nas duas direções. Anos atrás eu olhei a lista de extensões de um mint Token-2022, vi `transferFeeConfig`, disse a um colega de time "tem taxa, não roteie isso," e nunca abri o corpo. Zero bps. Eu custei um dia à gente por causa de um token que se comportava exatamente como uma transferência comum. O erro oposto é pior e eu já vi gente cometer: ver uma taxa em 0 e concluir que dá para tratar o token como livre de taxa para sempre.

Três perguntas, em ordem, e você precisa das três:

1. **A extensão está presente?** Leia a lista TLV. Barato, e é onde a maioria das pessoas para.
2. **O valor dela faz alguma coisa hoje?** Leia o campo específico. Program id do hook, basis points da taxa, endereço do delegado, flag de aprovação.
3. **Alguém consegue mudar esse valor?** Leia o campo de autoridade. Se existe uma autoridade, a resposta de hoje para a pergunta dois é um snapshot, não uma propriedade.

A pergunta três é aquela para a qual esta lição existe. O processador de transfer hook do Token-2022 te diz exatamente por quê: o `process_update` carrega a extensão, puxa `Option::<Address>::from(extension.authority)`, e devolve `NoAuthorityExists` quando essa option é `None`. Um slot de hook com autoridade nula é um slot morto, permanentemente. Um slot de hook com autoridade viva é um interruptor, e o do PYUSD está vivo: todas as oito extensões dele listam a mesma autoridade, `2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk`. Uma chave só segura todas as opções do mint.

Isso te dá três veredictos em vez de dois, e você consegue derivar os três a partir de dados que você já buscou.

![Um fluxo de decisão de três perguntas transforma presença de extensão, valor de campo e autoridade em um de três veredictos, separando as oito extensões do PYUSD em quatro ativas e quatro dormentes.](assets/v02-flowchart.png)

Dormente não é sinônimo de inofensiva. Quer dizer armada, e a diferença entre armada e disparando é uma assinatura de uma chave que você consegue nomear.

### Quem está vendido na opção

Aqui está o enquadramento que fez a ficha cair para mim, e é a razão de esta lição ficar no módulo de economia em vez do módulo de mecânica.

Uma extensão armada é uma opção, no sentido chato de finanças. O emissor tem o direito, não a obrigação, de ligar um comportamento. Carregar isso custa quase nada para ele: alguns bytes extras de rent na criação, e uma conta de mint um pouco maior. Isso paga opcionalidade a ele. E alguém está do outro lado dessa opção, porque opções não têm um lado só.

Você. Todo mundo que segura o token está vendido nela.

Caminhe com um número limpo. Digamos que a sua tesouraria mova 10,000 PYUSD para um parceiro, e digamos que a autoridade de taxa tivesse agendado 50 bps com um máximo de 100 tokens. Os seus 10,000 saem da sua conta e 9,950 chegam, os outros 50 ficando retidos na conta de token do destinatário até o emissor fazer harvest deles. Nada na sua instrução de transferência mudou. Nada na sua integração mudou. A sua contabilidade fica errada em 50 tokens por 10,000, para sempre, e se o seu produto cotou ao destinatário um valor exato, o seu produto agora está errado. Isso é a opção sendo exercida, e você estava vendido nela, soubesse ou não que a posição existia.

Dois dos slots dormentes do PYUSD têm timing de exercício muito diferente, o que importa mais do que o número da taxa em si:

- **A opção do hook liquida na hora.** O `TransferHookInstruction::Update` escreve o `extension.program_id` em uma instrução, só assinatura da autoridade. A próxima transferência depois daquele bloco faz CPI para um programa que não existia no seu modelo um minuto atrás.
- **A opção da taxa liquida duas epochs à frente.** O processador de taxa de transferência deliberadamente escreve uma alíquota nova à frente da epoch atual, com um comentário na fonte que diz isso sem rodeios: configurada duas epochs à frente para evitar rug pulls. O mint carrega `olderTransferFee` e `newerTransferFee` com carimbos de epoch precisamente para que uma mudança seja visível antes de morder. A 432,000 slots por epoch e com o alvo atual de 300ms de tempo de slot (o segundo estágio do SIMD-0525, vivo desde a epoch 1024; 400ms e 350ms são história agora), isso dá cerca de três dias de aviso, e o tempo de slot medido corre um tiquinho mais devagar, então trate isso como um piso.

Essa assimetria é uma decisão de design dos autores do Token-2022 e você deveria sentir isso. A extensão que pega o seu dinheiro te dá três dias. A extensão que consegue rejeitar a sua transferência de cara não te dá nenhum.

![Uma chave de autoridade conecta a oito slots de extensão no mint do PYUSD, quatro deles vivos e quatro armados, com o hook exercível imediatamente e a taxa só depois de duas epochs.](assets/v03-diagram.png)

### Por que armar um slot que você nunca pretende disparar

Objeção justa, e é a que eu levantaria: se o PYUSD nunca cobra uma taxa e nunca configura um hook, por que carregar isso? Peso morto, bytes extras, escrutínio extra de todo integrador que lê a lista e entra em pânico.

Por causa de uma restrição que você carrega desde o módulo um. Extensões são só no momento da criação. Não existe instrução que parafuse `transferHook` em um mint que foi entregue sem ele. A alternativa a armar um slot no dia um não é "adicionar depois." A alternativa é: cunhar um token novo, migrar todo holder, ser relistado em todo venue, e atualizar toda integração que deixou o seu endereço hardcoded.

Agora precifique os dois caminhos com honestidade. Armar oito slots na criação custa um pouco de rent extra em uma conta, para sempre, e um fardo permanente de explicação com integradores. Migrar um dólar regulado com centenas de milhões em supply custa coordenação com toda exchange, todo custodiante e toda carteira que encostou nele, mais a cauda de valor encalhada em contratos que ninguém atualiza. Essas duas coisas não são da mesma ordem de grandeza, e não chegam perto.

![Uma comparação de dois caminhos mostra que armar extensões na criação do mint custa bytes extras e escrutínio de integradores, enquanto a alternativa é uma migração completa de token depois.](assets/v04-comparison.png)

Então um emissor com formato de compliance arma tudo o que um regulador poderia plausivelmente exigir e não dispara nada disso. Isso não é indecisão. É o jeito mais barato de manter uma promessa que você ainda não consegue descrever: se chegar uma regra que exija uma taxa, um hook de allowlist, ou saldos privados com uma auditor key, a resposta é uma instrução em vez de uma migração. O brief do seu capstone vai te passar a mesma decisão numa escala menor, e a versão honesta dela é uma frase num memorando: este slot está armado, esta chave o segura, isto é o que nos faria usá-lo.

O trade-off corta para o outro lado, porém. Um slot armado é uma promessa para os seus integradores também, e eles leem isso como risco. Alguns venues recusam mints Token-2022 cujo conjunto de extensões eles não modelaram, e recusar por presença em vez de por valor é uma coisa perfeitamente racional para um programa de DEX fazer quando o valor pode mudar debaixo dele. Armar um hook para ficar seguro com reguladores pode te custar a listagem de que você precisava. Nomeie isso no memorando também.

### JTO: o seu caminho de Merkle, em escala de airdrop

Caso diferente, mesma jogada: confira a coisa entregue contra a coisa que você construiu.

A distribuição do JTO da Jito rodou sobre um distribuidor de Merkle com vesting linear. O programa está em `mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv`, está vivo e executável na mainnet hoje sob o upgradeable loader, e expõe `claim_locked` ao lado do claim comum: destinatários fizeram claim de uma porção destravada na hora enquanto o resto fez vesting linearmente, naquele caso até 2024-12-07.

Leia o código da sua própria lição de airdrop ao lado disso. Você construiu uma árvore de Merkle de destinatários, publicou uma raiz, deixou cada um que faz claim apresentar uma prova, marcou a folha com o claim feito para que a segunda tentativa falhe, e ligou o `claim_locked` para a porção com vesting do compost drop, que é o mesmo mecanismo na mesma interface da mesma família de programas que a Jito usou para distribuir um token de governança vivo com uma cauda de vesting. A diferença entre o seu drop da Overgrowth e uma das maiores distribuições de token da Solana é o tamanho da árvore e o valor das folhas.

Esta é a parte do estudo de caso que eu quero que você realmente leve: as primitivas não são escalonadas por porte. Não existe um mecanismo de airdrop "de verdade" para o qual você se gradua. Existe uma raiz de Merkle, uma prova, um marcador de claim, e um relógio de vesting opcional, e a razão de as pessoas ainda errarem airdrops nunca é o mecanismo. São a lista de folhas, o marcador de claim duplo, e a tokenomics que ninguém publicou.

![Uma tabela mapeia cinco primitivas do curso para as contrapartes entregues delas, incluindo o conjunto de extensões do PYUSD, o slot de hook dormente dele, o distribuidor de Merkle do JTO e o campo is_agent do DAS.](assets/v05-table.png)

### Os trilhos sobre os quais todo o resto anda

Afaste um nível, porque um token só é interessante se alguma coisa se move por ele.

Em 2026-09-01 existem cerca de $16.05B de stablecoins atreladas ao dólar circulando na Solana, segundo a série de stablecoins da DefiLlama, só as atreladas ao dólar, que é um número ao vivo que se move diariamente e que você deveria puxar de novo em vez de citar de um curso. Metodologia importa aqui tanto quanto o número: rastreadores diferentes contam pegs diferentes e wrappers diferentes, então duas fontes discordando por algumas centenas de milhões é normal, e um número sem a fonte e a data dele não é um fato, é achismo.

A direção é mais fácil de defender do que qualquer número isolado. A Stripe comprou a Bridge por $1.1B, com fechamento em fevereiro de 2025, com mais ou menos $1.5B de volume total de pagamentos mensal na época, como reportado no panorama de stablecoins da Helius, e a SpaceX vem agregando receita da Starlink em stablecoins. Quando uma empresa de pagamentos paga um bilhão de dólares por infraestrutura de stablecoin em vez de construí-la, isso é um mercado te dizendo que os trilhos já foram escolhidos.

![Uma linha do tempo vai do lançamento do PYUSD em maio de 2024, passando pelo fim do vesting do JTO e pelo fechamento da Bridge pela Stripe, até a fronteira de identidade de agente de 2026 e a leitura ao vivo do mint de hoje.](assets/v06-timeline.png)

Essa é a razão honesta de valer a pena fazer o seu capstone. Não que tokens sejam empolgantes. Que o encanamento que você vem construindo é o encanamento que um processador de pagamentos acabou de pagar. O curso Solana Payments and Commerce lê este exato mint do PYUSD pelo lado da integração, e os trilhos de compliance que ficam acima destas primitivas, o Token ACL entre eles, deliberadamente não são ensinados aqui; eles são território do curso planejado DeFi and RWA Engineering.

### A fronteira, datada e com ressalvas

Mais uma, e esta vem com um rótulo de aviso colado antes do conteúdo.

Existe trabalho real em 2026 sobre dar a agentes autônomos uma identidade on-chain. A Metaplex tem um Agent Registry, o Core tem um plugin `AgentIdentity`, e o DAS expõe `is_agent` como um booleano que pode ser nulo no nível do ativo. O encanamento está sendo assentado: um agente ganha um ativo, o ativo carrega um plugin de identidade, e um indexador consegue responder "essa coisa é um agente?" na mesma leitura que responde "quem é o dono dela?"

Agora o aviso. Isto é um assunto de fronteira, não um padrão de produção. Nada no seu capstone deveria depender disso. Um booleano que pode ser nulo em uma interface de leitura é exatamente com o que se parece um campo recente: ele pode ser nulo porque a maioria dos ativos não tem nada a dizer, e nulo te diz que o campo existe e não te diz que um ecossistema convergiu sobre o que ele significa. Se você construir uma cancela em cima do `is_agent` hoje, você está restringindo o acesso com base em um campo cuja semântica ainda pode mudar debaixo de você, que é uma classe de risco diferente de restringir o acesso com base na posse.

A razão de isto pertencer aqui é que é o mesmo músculo de avaliação, um degrau antes no ciclo de vida. Você acabou de passar uma seção decidindo se uma extensão entregue faz alguma coisa. Decidir se um campo de fronteira significa alguma coisa é a mesma leitura: quem escreve nele, o que ele diz hoje, e o que acontece com o seu produto se essa resposta mudar. Acompanhe, rode um spike se agentes são o seu produto, não coloque isso no caminho crítico.

### Três fontes, três confiabilidades

O que me leva à disciplina de leitura que esta lição inteira vem ensinando de lado, e a um exemplo que é quase bom demais.

A própria página de soluções de Token Extensions da Solana ainda diz que transferências confidenciais são "previstas para o fim de 2024." Página viva, afirmação morta, e transferências confidenciais estão na mainnet há um bom tempo. Aquela página não é inútil. Ela traz uma lista de cinco firmas de auditoria que revisaram o programa, o que genuinamente vale ter e não apodrece. Cite ela pela lista de auditoria. Não cite ela por uma data.

E uma segunda dobra na direção oposta: o Token-2022 continua sendo um programa atualizável. O HEAD do repo pode carregar uma extensão ou uma correção que o deployment na mainnet ainda não tem. Então o código que você lê no GitHub é um teto, não uma descrição do que vai executar no próximo bloco.

![Uma comparação de três vias classifica a conta de mint ao vivo, as páginas oficiais de documentação e o repositório do programa pelo que dá para confiar em cada um e onde cada um falha.](assets/v07-comparison.png)

### O trade-off, nomeado

Ler dormência ao vivo dá mais trabalho do que ler uma lista de features, e isso expira. O seu relatório é verdadeiro para o bloco em que você rodou ele, e uma autoridade consegue invalidar a linha do hook dele no bloco seguinte, sem aviso e sem anúncio. Esse é o custo honesto deste método: ele te dá uma resposta correta com prazo de validade curto, e te tenta a tratar um snapshot como uma propriedade.

A mitigação não é ler com mais afinco. É anotar quem segura cada opção e qual é a latência de exercício, e então decidir de uma vez se você consegue conviver com o pior caso. Três dias de aviso sobre uma taxa é uma coisa que um processo de tesouraria consegue absorver. Zero aviso sobre um hook é uma coisa que a sua integração ou sobrevive por design ou não.

## Lab: o relatório de dormência

Você vai construir o `dormancy-report.ts`: aponte ele para qualquer mint, e ele imprime veredictos por extensão derivados de valores e autoridades. É a ferramenta que responde ao passo de verificação do seu capstone, e é a ferramenta que eu ia querer em qualquer call de avaliação.

1. **Preparação.** Crie uma pasta e confira o seu Node. Você precisa do Node 20 ou mais novo para o `fetch` global que este script usa; eu estou na 23.9.

   ```bash
   mkdir -p labs/m09-l3 && cd labs/m09-l3 && node --version
   ```

   Nada de npm install. Este script tem zero dependências de propósito, e o propósito é uma decisão que vale declarar: uma ferramenta de avaliação que precisa de um workspace é uma ferramenta que você não vai rodar quando um colega de time colar um endereço de mint no chat. O `npx tsx@4.23.12` busca o runner de TypeScript sob demanda. Esse pin é de 2026-08-22 e já tinha sido passado pela 4.23.13 quando reconferido em 2026-09-01; confira `npm view tsx version` quando você ler isto, já que ele entrega versões novas com frequência.

2. **Escreva o classificador.** Salve isto como `dormancy-report.ts`. As duas extensões em que o estudo de caso se apoia são trabalhadas por inteiro, o resto segue a mesma forma.

   ```typescript
   // dormancy-report.ts - classify every extension on a live mint as ACTIVE, DORMANT, INERT or REVIEW.
   // Zero npm dependencies. Run: npx tsx@4.23.12 dormancy-report.ts <MINT_ADDRESS>
   // Read-only: two RPC calls, nothing signed, nothing sent.

   const RPC = process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com";
   const TOKEN_2022 = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
   const SLOT_SECONDS = 0.3; // 300ms target since SIMD-0525 stage 2 (epoch 1024); measured wall clock runs a little slower

   type Verdict = "ACTIVE" | "DORMANT" | "INERT" | "REVIEW";
   type State = Record<string, unknown>;

   interface Extension {
     extension: string;
     state?: State;
   }
   interface MintInfo {
     decimals: number;
     supply: string;
     extensions?: Extension[];
   }
   interface AccountValue {
     owner: string;
     space: number;
     data: { parsed: { info: MintInfo; type: string } };
   }
   interface EpochInfo {
     epoch: number;
     slotsInEpoch: number;
   }

   async function rpc<T>(method: string, params: unknown[]): Promise<T> {
     const res = await fetch(RPC, {
       method: "POST",
       headers: { "Content-Type": "application/json" },
       body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
     });
     const body = (await res.json()) as { result?: T; error?: unknown };
     if (body.error) throw new Error(`${method}: ${JSON.stringify(body.error)}`);
     if (body.result === undefined) throw new Error(`${method}: empty result`);
     return body.result;
   }

   function str(state: State | undefined, key: string): string | null {
     const v = state?.[key];
     return typeof v === "string" ? v : null;
   }

   function fee(state: State | undefined, key: string): { epoch: number; bps: number; max: string } {
     const f = (state?.[key] ?? {}) as Record<string, unknown>;
     return {
       epoch: Number(f.epoch ?? 0),
       bps: Number(f.transferFeeBasisPoints ?? 0),
       max: String(f.maximumFee ?? 0),
     };
   }

   // The authority is the option holder: the key that can CHANGE the config.
   // Different extensions spell that field differently, hence the fallbacks.
   // One conflation to know about before you reuse this on arbitrary mints:
   // "delegate" is in this list as a convenience, and a permanent delegate is
   // the key that HOLDS a power fixed at creation, not one that can change it.
   // On PYUSD every key happens to be the same party, so the line reads fine;
   // on a mint with split keys, print the delegate under its own label.
   function authorityOf(ext: Extension): string | null {
     const s = ext.state;
     return (
       str(s, "authority") ??
       str(s, "closeAuthority") ??
       str(s, "delegate") ??
       str(s, "transferFeeConfigAuthority") ??
       str(s, "updateAuthority")
     );
   }

   function classify(ext: Extension, epoch: number): { verdict: Verdict; reason: string } {
     const s = ext.state;
     switch (ext.extension) {
       case "transferHook": {
         const programId = str(s, "programId");
         if (programId) return { verdict: "ACTIVE", reason: `every transfer CPIs into ${programId}` };
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: "programId null; the hook authority can set one at any time" }
           : { verdict: "INERT", reason: "programId null and no authority: the slot can never fire" };
       }
       case "transferFeeConfig": {
         const newer = fee(s, "newerTransferFee");
         const older = fee(s, "olderTransferFee");
         const live = epoch >= newer.epoch ? newer : older;
         if (live.bps > 0) {
           return { verdict: "ACTIVE", reason: `${live.bps} bps withheld per transfer, max ${live.max}` };
         }
         const scheduled = newer.epoch > epoch ? ` (a ${newer.bps} bps fee lands at epoch ${newer.epoch})` : "";
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: `0 bps at epoch ${epoch}; the fee authority can schedule one${scheduled}` }
           : { verdict: "INERT", reason: "0 bps and no fee authority: the rate can never move" };
       }
       case "permanentDelegate": {
         const delegate = str(s, "delegate");
         return delegate
           ? { verdict: "ACTIVE", reason: `${delegate} can move or burn from any account, no owner signature` }
           : { verdict: "INERT", reason: "no delegate set" };
       }
       case "mintCloseAuthority": {
         const closeAuthority = str(s, "closeAuthority");
         return closeAuthority
           ? { verdict: "ACTIVE", reason: `${closeAuthority} can close this mint once supply hits 0` }
           : { verdict: "INERT", reason: "no close authority set" };
       }
       case "confidentialTransferMint": {
         const auto = s?.autoApproveNewAccounts === true;
         const auditor = str(s, "auditorElgamalPubkey");
         if (auto) {
           return {
             verdict: "ACTIVE",
             reason: `any account can self-configure; auditor ${auditor ?? "none"}`,
           };
         }
         return authorityOf(ext)
           ? { verdict: "DORMANT", reason: "autoApproveNewAccounts false: every account needs issuer approval first" }
           : { verdict: "INERT", reason: "no auto-approval and no authority to grant it" };
       }
       case "confidentialTransferFeeConfig": {
         const withheld = str(s, "withheldAmount") ?? "";
         // Base64 of all-zero bytes is a run of "A" characters (every 6-bit
         // group of zeros encodes as "A"), possibly "="-padded; that is what
         // the regex matches. Deliberate simplification alongside it: this
         // branch judges only the withheld pile and never consults an
         // authority, so it can return DORMANT or ACTIVE but never INERT.
         // The three-question framework's Q3 is skipped here because the
         // extension's arming is decided by the confidential pair around it.
         const empty = /^A*=*$/.test(withheld);
         return empty
           ? { verdict: "DORMANT", reason: "withheld ciphertext is all zeros: nothing collected yet" }
           : { verdict: "ACTIVE", reason: "confidential fees are being withheld" };
       }
       case "metadataPointer": {
         const target = str(s, "metadataAddress");
         return target
           ? { verdict: "ACTIVE", reason: `metadata resolves at ${target}` }
           : { verdict: "INERT", reason: "pointer set to nothing" };
       }
       case "tokenMetadata": {
         const name = str(s, "name") ?? "?";
         const symbol = str(s, "symbol") ?? "?";
         return { verdict: "ACTIVE", reason: `on-mint metadata: ${name} (${symbol})` };
       }
       default:
         return { verdict: "REVIEW", reason: "no rule written for this extension yet: read the state by hand" };
     }
   }

   async function main() {
     const mint = process.argv[2];
     if (!mint) throw new Error("usage: npx tsx@4.23.12 dormancy-report.ts <MINT_ADDRESS>");

     const epochInfo = await rpc<EpochInfo>("getEpochInfo", []);
     const account = await rpc<{ value: AccountValue | null }>("getAccountInfo", [
       mint,
       { encoding: "jsonParsed" },
     ]);
     if (!account.value) throw new Error(`no account at ${mint}`);

     const { owner, space, data } = account.value;
     const info = data.parsed.info;
     const extensions = info.extensions ?? [];

     console.log(`mint:      ${mint}`);
     console.log(`program:   ${owner}${owner === TOKEN_2022 ? " (Token-2022)" : " (not Token-2022)"}`);
     console.log(`size:      ${space} bytes, ${info.decimals} decimals`);
     console.log(`epoch:     ${epochInfo.epoch}`);
     console.log(`extensions: ${extensions.length}\n`);

     const tally: Record<Verdict, number> = { ACTIVE: 0, DORMANT: 0, INERT: 0, REVIEW: 0 };
     for (const ext of extensions) {
       const { verdict, reason } = classify(ext, epochInfo.epoch);
       tally[verdict] += 1;
       const holder = authorityOf(ext) ?? "none";
       console.log(`${verdict.padEnd(8)} ${ext.extension}`);
       console.log(`         why: ${reason}`);
       console.log(`         authority: ${holder}`);
     }

     const days = (epochInfo.slotsInEpoch * 2 * SLOT_SECONDS) / 86_400;
     console.log(
       `\nverdict: ${tally.ACTIVE} active, ${tally.DORMANT} dormant, ${tally.INERT} inert, ${tally.REVIEW} unreviewed`,
     );
     console.log(
       `a hook flip lands immediately; a fee change lands two epochs out, about ${days.toFixed(1)} days at ${SLOT_SECONDS}s slots`,
     );
   }

   main().catch((e: unknown) => {
     console.error(e instanceof Error ? e.message : e);
     process.exit(1);
   });
   ```

3. **Rode contra o PYUSD.**

   ```bash
   npx tsx@4.23.12 dormancy-report.ts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
   ```

   O final da minha rodada em 2026-09-01:

   ```text
   verdict: 4 active, 4 dormant, 0 inert, 0 unreviewed
   a hook flip lands immediately; a fee change lands two epochs out, about 3.0 days at 0.3s slots
   ```

   Acima disso você recebe oito blocos, cada um com o veredicto dele, o campo de onde o veredicto veio, e a autoridade que o segura. Toda linha de autoridade lê `2apBGMsS6ti9RyF5TwQTDswXBWskiJP2LD4cUEDqYJjk`. Uma chave, oito slots, quatro vivos hoje e quatro armados e esperando. (Armada é a palavra desta lição para as quatro DORMANT: configuradas, paradas, e a uma assinatura de disparar.)

   Se a sua rodada mostrar uma contagem diferente, não presuma que a lição está certa e o seu terminal está errado. Esta é uma conta ao vivo e emissores mudam coisas. Leia a linha `why:` da extensão que se moveu. Esse reflexo é o curso inteiro, sinceramente.

4. **Rode contra um mint que não tem nada a dizer.** O USDC clássico é o caso de controle.

   ```bash
   npx tsx@4.23.12 dormancy-report.ts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
   ```

   Você deve receber `(not Token-2022)`, 82 bytes, e zero extensões. Um mint clássico não tem slots para armar, o que é uma propriedade real e às vezes exatamente a que você quer. Não ter nada para ler também é informação.

5. **Aponte para o seu próprio mint de SPROUT.** O RPC é configurável, então mire no endpoint em que o seu mint da Overgrowth mora:

   ```bash
   RPC_URL=https://api.devnet.solana.com npx tsx@4.23.12 dormancy-report.ts <YOUR_SPROUT_MINT>
   ```

   A sua taxa de transferência deve voltar ACTIVE com os bps que você configurou, porque você de fato usa a sua. Essa diferença de uma linha só entre o seu mint e o da PayPal é toda a distinção entre entregar uma taxa e reservar uma.

6. **Passo de Completion: escreva você mesmo uma regra do classificador.** O branch `default` devolve `REVIEW`, que é a resposta honesta para uma extensão sobre a qual você não pensou. Escolha uma que você construiu antes no curso, `pausable` ou `defaultAccountState` ou `scaledUiAmount`, e adicione um `case` para ela. A regra tem que responder às mesmas três perguntas: que campo carrega o comportamento, que valor a torna inerte, e que autoridade consegue movê-la. Se você não consegue nomear o campo de autoridade, você ainda não tem uma regra, tem um palpite.

7. **Checkpoint.** Rode os três mints em sequência. Você deve conseguir apontar para qualquer linha da saída do PYUSD e dizer de qual campo do RPC ela veio, sem abrir o script.

## Challenge

Solo, e esta é a peça em que você se cobra um padrão de passa/não passa — nada recolhe isso, que é exatamente por que escrever com honestidade é o exercício. Escreva o memorando de dormência.

Rode o `dormancy-report.ts` contra o mint ao vivo do PYUSD e escreva cinco frases em que um colega conseguiria agir, uma por item abaixo. Quais das oito extensões TLV dele estão ativas e quais estão configuradas mas dormentes, com o program id do hook e os valores de taxa de transferência que você de fato leu. Quem segura as opções, por endereço. Qual é a latência de exercício do hook versus a da taxa, e por que essas diferem. Uma frase sobre o que você monitoraria se o seu produto liquidasse neste token. E uma frase nomeando a coisa que o seu relatório não consegue te dizer.

Considere aceito quando o memorando derivar os veredictos dele de valores em vez de presença, citar números que a sua própria rodada imprimiu, se datar, e nomear o endereço da autoridade. Considere rejeitado se ele disser que o PYUSD tem transferências confidenciais, logo os saldos de PYUSD são privados. Os trilhos estão configurados, a catraca está travada, e a diferença é a lição.

Passada opcional extra se agentes estiverem em qualquer lugar perto do seu roadmap: acrescente duas frases sobre `is_agent` e o Metaplex Agent Registry que sobreviveriam a um leitor cético em 2027. Dica: elas contêm a palavra "ainda."

## Checkpoint

O critério é um relatório que você rodou mais um memorando que você mandaria. Se você não consegue produzir o memorando sem rodar o script de novo, tudo bem, é para isso que o script existe.

A versão em uma frase, terminal fechado: uma extensão estar presente não te diz nada, os valores de campo dela te dizem o que acontece hoje, e a autoridade dela te diz quem pode mudar isso, então toda avaliação de verdade tem três leituras de profundidade e é verdadeira só para o bloco em que você a rodou.

Os erros que eu espero, na ordem em que eu os espero. Primeiro, a cilada da presença, que é aquela em que eu mesmo caí: ler `transferFeeConfig` como "este token cobra taxas" sem abrir o corpo. Segundo, a cilada da segurança, que é pior: ler 0 bps como propriedade do token em vez de um snapshot com uma chave nomeada por trás. Terceiro, a cilada da fronteira: escrever sobre identidade de agente como se o registry e o plugin e o campo `is_agent` somassem um padrão. Eles somam uma direção. Diga direção.

E aí está o arco de economia fechado. Você construiu um mint com extensões de verdade, um hook que roda na transferência, uma rota de taxa e um buyback, uma coleção de cNFTs, um leitor, uma cancela, um airdrop com vesting, e agora o julgamento de ler a versão de outra pessoa de tudo isso e dizer o que ela de fato faz.

Você construiu cada degrau. O último módulo te entrega um brief de produto, cinco para escolher, o quinto um espaço em branco que você pode preencher com o seu próprio produto: escolha a primitiva, entregue ela, ligue um trilho, e prove que ela resolve. Ninguém te diz qual primitiva desta vez, e o memorando de dormência que você acabou de escrever é o mesmo músculo que o memorando de seleção do capstone exercita: julgamento, declarado como frases datadas em que um colega consegue agir.
