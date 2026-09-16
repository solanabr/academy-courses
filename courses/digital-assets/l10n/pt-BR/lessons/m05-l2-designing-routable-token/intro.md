# Projetando um token roteável e em conformidade

## Resumo

Na lição passada você leu a regra no próprio programa da Raydium que aceita um SPROUT de taxa+metadados e rejeita a variante com transfer hook dele, construiu um preditor que a reproduz, e colocou os dois mints na frente dela em um fork da mainnet. Esta lição transforma aquela leitura em uma decisão e em um entregável. Você vai enunciar a tese de compatibilidade em uma frase, defendê-la do lado da pool na mesa, escolher o conjunto final de extensões de launch-venue do SPROUT contra a allowlist, e produzir o relatório de roteabilidade: cada extensão incluída marcada como whitelisted ou refused com a sua justificativa, os venues em que o SPROUT é verificado-roteável, e uma lista explícita das afirmações que você precisa verificar no seu próprio alvo em vez de confiar na matriz congelada de qualquer pessoa, este curso incluído. O recuo está quase completo: a tese e a montagem do conjunto são trabalhadas com apoio leve, a seção de honestidade do relatório é sua, solo, e não há coding challenge porque o relatório em si é a checagem. Esta é uma lição de julgamento com a forma de uma lição de build.

Você passou quatro módulos dando capacidades ao SPROUT. Taxas que financiam uma tesouraria. Metadados nativos. Uma variante com hook que loga todo harvest. Uma ramificação confidencial com uma auditor key. Cada uma delas foi um bom build. E na lição passada uma DEX olhou a soma de todas e disse não. Entregar agora é uma decisão, não uma consulta: cada extensão que você mantém é uma capacidade que você quer e um venue que você pode perder, e você é quem assina a troca.

Então comece onde uma decisão deveria começar, com a lista completa de candidatas e um veredicto por linha. Na sua pasta de lab do m05-l1, ao lado de `predict-routability.ts`, solte isto e rode:

```typescript
// audit-candidates.ts: every extension the SPROUT design has worn or weighed
// (the delegate and frozen-default rows were m02 catalog considerations, never
// minted onto SPROUT), one verdict each.
import { isRoutable } from "./predict-routability";

const CANDIDATES = [
  "TransferFeeConfig",
  "MetadataPointer",
  "TokenMetadata",
  "TransferHook",
  "PermanentDelegate",
  "DefaultAccountState",
  "ConfidentialTransferMint",
];

for (const ext of CANDIDATES) {
  const ok = isRoutable({ tokenProgram: "token2022", extensions: [ext] });
  console.log(ext.padEnd(26), ok ? "whitelisted" : "refused");
}
```

```bash
npx tsx audit-candidates.ts
```

Três whitelisted, quatro refused. Essa saída é a lição inteira; as próximas três mil palavras são sobre por que a linha cai exatamente ali, e como fica um relatório honesto construído em cima dela.

## Quem tem o direito de rodar código em toda transferência

### A tese, uma frase

Aqui está ela: uma DEX coloca na whitelist extensões que só mudam exibição ou raspam uma taxa declarada, e recusa extensões que deixam o emissor rodar código arbitrário ou mover tokens de outras pessoas. Taxa, exibição e contabilidade de um lado: TransferFeeConfig, MetadataPointer, TokenMetadata, InterestBearingConfig, ScaledUiAmount, exatamente as cinco da allowlist do CP-Swap da Raydium. Poder e controle do outro: TransferHook, PermanentDelegate, DefaultAccountState congelado, ConfidentialTransfer, todas refused. Diga isso nessas palavras e não nas mais curtas e tentadoras, porque "compliance entra na whitelist" é falso de um jeito que vai custar um lançamento a alguém: PermanentDelegate e DefaultAccountState são as duas primitivas favoritas do time de compliance e as duas ficam do lado refused. O capstone treina exatamente essa frase de uma linha. Você verificou a mecânica disso na lição passada lendo o fonte. A pergunta de hoje é outra. Por que a linha está ALI, e não em outro lugar?

### Derivando a linha do assento da pool

Sente na cadeira da pool por um minuto. Uma pool é uma pilha de dois tokens mais uma invariante, e o seu único trabalho não negociável é que a pilha fique onde a matemática diz que ela deveria ficar. Agora caminhe pelos designs ingênuos e veja cada um falhar.

A DEX poderia revisar tokens caso a caso? Esse é um design de verdade, e a Orca o entrega. Um Token Badge é uma aprovação por mint que o time da Orca concede depois de realmente olhar o seu mint, que é como um token carregando algo como PermanentDelegate consegue ser negociado lá afinal: não passando por uma lista publicada, mas passando por uma pessoa. Precifique esse design honestamente, porque ele não é estritamente pior que uma lista. Um revisor vê coisas que um match statement nunca vai ver: quem é o emissor, se a chave do delegate é uma multisig ou um laptop só, se a autoridade da taxa já foi revogada. O que um revisor não consegue fazer é escalar, responder em um tempo limitado, ou te dizer o veredicto antes de você ter se comprometido com um design e cunhado. E uma fábrica de pools sem permissão não consegue usar um de jeito nenhum. A criação de pool ali é uma transação que qualquer pessoa pode enviar em qualquer slot sem nenhum humano no caminho, então para o CP-Swap a regra de aceitação tem que ser código. Código não consegue ler um whitepaper ou um parecer jurídico. Ele só consegue ler o mint.

![Tabela de sete eixos comparando a revisão humana do Token Badge da Orca com a allowlist compilada da Raydium, com a linha da fábrica sem permissão marcada como a razão pela qual a regra do CP-Swap precisa ser código.](assets/v01-table.webp)

Bom, então a pool poderia simplesmente aceitar tudo e lidar com as consequências? Caminhe pela lista refused e as consequências não são gerenciáveis. Um TransferHook significa que um programa que a DEX nunca auditou roda dentro de todo swap, com o compute que ele quiser; os docs da Raydium o recusam exatamente nessas palavras, um programa customizado invocado em toda transferência com consumo arbitrário de CU. Um PermanentDelegate significa que alguma chave por aí pode mover tokens de qualquer conta, e o vault da pool é uma conta; a Raydium de novo, literalmente: um detentor do delegate pode varrer qualquer conta de token, incluindo o vault da pool. DefaultAccountState congelado significa que o emissor decide se as próprias contas da pool podem transacionar afinal. ConfidentialTransfer significa que os valores são encriptados, e você não consegue rodar uma invariante sobre ciphertext. Cada recusa protege uma suposição estrutural diferente, mas todas são do mesmo gênero: a extensão dá a alguém de fora da pool autoridade sobre o que acontece dentro dela.

### Por que a resposta tem que ser binária

Vale fechar mais uma saída de emergência, porque é a próxima para a qual um bom engenheiro vai correr. A regra poderia ser graduada em vez de binária? Deixe um mint arriscado entrar, mas cobre dele uma taxa maior, limite o tamanho da pool dele, coloque-o em quarentena atrás de um aviso. Graduar exige que a pool coloque um número no risco, e as extensões refused não têm números. Qual é a sobretaxa correta para um transfer hook cuja autoridade pode apontá-lo para um código completamente diferente amanhã? Não existe resposta, porque a exposição não é uma distribuição sobre a qual você possa integrar, é "o que aquele programa decidir fazer em seguida". Uma taxa declarada de 500 basis points é um custo. Um hook atualizável é uma responsabilidade em aberto, e responsabilidades em aberto não têm preços, elas têm aceite-ou-recuse. É por isso que a regra cai binária, e por que ela precisa ser uma função pura do conjunto de extensões, nada mais.

Agora olhe o lado aceito com os mesmos olhos. Uma taxa de transferência é poder do emissor também, tecnicamente, mas ela é declarada, limitada por teto e legível no mint, então a pool consegue precificá-la, e a interface te entrega o `calculate_fee`, o helper de matemática de taxa no estado do TransferFeeConfig cuja fórmula você espelhou em TypeScript como `transferFee` lá no m02-l1, para fazer exatamente isso. Metadados mudam o que humanos veem, nunca o que o token faz. InterestBearingConfig e ScaledUiAmount são aritmética de exibição pura; os valores brutos que a pool contabiliza nunca se movem. O padrão não é "extensões inofensivas passam". O padrão é: qualquer coisa que a pool consiga precificar por completo a partir de dados on-chain passa, qualquer coisa que reserve discricionariedade para o emissor falha. Esse é um juízo de valor sobre quem tem o direito de rodar código em toda transferência, codificado como um match statement em Rust, e eu genuinamente acho isso mais honesto que um formulário de listagem. O código não pode ser lobbyado.

![Diagrama de duas colunas separando as cinco extensões de exibição e taxa na whitelist das quatro extensões de poder refused, divididas por se a pool consegue computar a sua invariante sem confiar no emissor.](assets/v02-diagram.webp)

Mais uma volta na manivela, porque a tese tem um corolário afiado que você encontrou na lição passada. Peça ao seu próprio preditor para enunciá-lo, na mesma pasta de antes:

```typescript
// additive-check.ts: does a whitelisted extension rescue a refused one?
import { isRoutable } from "./predict-routability";

const CASES: [string, string[]][] = [
  ["fee alone", ["TransferFeeConfig"]],
  ["hook alone", ["TransferHook"]],
  ["fee + metadata + hook", ["TransferFeeConfig", "TokenMetadata", "TransferHook"]],
  ["all five allowlisted + delegate", [
    "TransferFeeConfig",
    "MetadataPointer",
    "TokenMetadata",
    "InterestBearingConfig",
    "ScaledUiAmount",
    "PermanentDelegate",
  ]],
];

for (const [label, extensions] of CASES) {
  const verdict = isRoutable({ tokenProgram: "token2022", extensions }) ? "ROUTABLE" : "REJECTED";
  console.log(label.padEnd(32), verdict);
}
```

```bash
npx tsx additive-check.ts
```

Só o primeiro caso volta roteável. Cinco extensões na allowlist mais uma refused ainda é rejeitado, e esse é o corolário: estar na allowlist não é aditivo. Uma extensão fora da lista contamina o mint inteiro. Não existe a defesa "mas ele também tem TransferFeeConfig", porque a exposição da pool a um permanent delegate não encolhe quando um fee config se senta ao lado. O seu preditor codifica isso como `every`, não `some`, e essa única palavra é a diferença entre o modelo do folclore e o de verdade.

### O que o conjunto chato te custa

Deixa eu dizer o trade-off em voz alta, porque este curso prometeu que sempre diria: o conjunto roteável é o conjunto chato. Projetar para negociabilidade máxima significa desistir de tudo de interessante que você construiu ou pesou desde o catálogo de autoridade do módulo dois em diante. Nenhuma lógica em transferência no mint que as pessoas negociam. Nenhum permanent delegate, nenhum onboarding congelado por padrão, os dois poderes do m02 que o SPROUT considerou e nunca vestiu. Nenhum valor escondido. Se o seu produto genuinamente precisa de uma extensão de poder, essa necessidade é real e esta lição não está te dizendo para abandoná-la. Está te dizendo para precificá-la: um hook significa território Meteora DBC em vez de CP-Swap, ou uma revisão de Token Badge na Orca que pode ou não dar certo para você, ou uma arquitetura de dois mints em que a variante com poder nunca toca uma pool. Escolher uma superfície menor de venues é um design legítimo. Descobrir uma superfície menor de venues no lançamento é um incidente.

![Fluxograma caminhando com cada extensão candidata por necessidade, pertencimento à allowlist, e colocação emissor-versus-detentor, terminando em descartar, manter, mover para uma variante de emissor, ou aceitar conscientemente uma superfície menor de venues.](assets/v03-flowchart.webp)

### Montando o conjunto do SPROUT

Aplique o procedimento à auditoria que você rodou no topo. TransferFeeConfig: a taxa financia a tesouraria, esse é o motor econômico do SPROUT desde o m02-l1, e ela está na whitelist. Mantém. MetadataPointer mais TokenMetadata: os metadados nativos que você ligou no m02-l4, na whitelist, só exibição. Mantém os dois. Esse é o mint negociável inteiro. Três extensões, todas chatas, todas precificáveis.

O hook é o adeus difícil. Você o escreveu você mesmo no m03, ele funciona, e ele é exatamente o código-arbitrário-em-toda-transferência que uma pool não consegue carregar. Ele sai do mint negociável. Se logar o harvest ainda importa para o produto, o hook vive em uma variante de emissor separada e não colocada em pool, o mesmo padrão da ramificação confidencial que você arquivou no m04: mints com poder para fluxos do emissor, um mint chato para o mercado. E aqui está um detalhe que faz a divisão em dois mints ser menos irritante do que parece: a matriz de combinações que você construiu no m01-l4 teria lutado contra um merge de qualquer jeito. Dobre o ConfidentialTransferMint dentro do mint de taxa e a regra 2 do `check-combo` exige o ConfidentialTransferFeeConfig em cima, o que arrasta o aparato confidencial de taxa inteiro. O próprio sistema de extensões fica empurrando poder e comércio para lados opostos. Eu lutei contra esse empurrão por um tempo nos meus próprios designs antes de aceitar que ele era estrutural.

![Comparação de três colunas do SPROUT de taxa-mais-metadados que está lançando (roteável, verificado em fork) contra a variante com transfer hook (rejeitada, só para o emissor) e a ramificação confidencial arquivada (impossível de colocar em pool por construção).](assets/v04-comparison.webp)

### As duas extensões da allowlist que o SPROUT ainda não está levando

Aqui está a objeção que eu levantaria relendo isto: o conjunto do SPROUT tem três extensões, e a allowlist tem cinco. InterestBearingConfig e ScaledUiAmount estão sentadas ali, pré-aprovadas, custando zero de roteabilidade. Por que não levá-las? Um token de farming com sabor de yield poderia plausivelmente querer acúmulo de juros, e um multiplicador de exibição te dá uma história de rebase de graça.

Porque a allowlist é um piso, não uma lista de compras. Passar por ela significa que o venue não vai te recusar por causa daquela extensão. Nunca significa que a extensão é de graça. As duas fazem a mesma coisa esperta: elas mudam o número que toda interface exibe sem mudar o número que o programa de fato move. Essa lacuna é a feature inteira e é também o custo inteiro. No momento em que o SPROUT carrega o ScaledUiAmount, o seu explorer, a sua exportação de CSV, a sua planilha de contabilidade, as suas respostas de suporte, o gráfico de preço da DEX e o saldo bruto em uma resposta de `getTokenAccountBalance` não são mais obviamente o mesmo número, e cada um desses leitores agora tem direito a uma explicação que você precisa escrever e manter verdadeira. O InterestBearingConfig acumula esse valor de UI continuamente; o ScaledUiAmount o multiplica por um fator que uma autoridade pode mudar depois. A matemática de tesouraria do SPROUT não precisa de nenhum dos dois.

Então a regra de design generaliza para além da roteabilidade, e esta é a versão que vale guardar: uma extensão tem dois custos, e a allowlist precifica só um deles. O custo um é o risco de recusa, que o venue publica. O custo dois é todo leitor downstream a quem você agora deve uma explicação, que ninguém publica e você paga para sempre. Eu já entreguei uma extensão porque ela estava na lista de aprovados de alguém, e depois gastei mais horas explicando-a em um canal de suporte do que eu jamais gastei usando-a. Duas extensões de que você não precisa são dois parágrafos em um runbook que você vai estar escrevendo às 2 da manhã.

### O PYUSD faz a mesma matemática em escala de bilhões de dólares

Você não precisa aceitar o padrão de dois trilhos de um token de curso. Leia-o no emblemático cuja história fechou a lição passada, e desta vez conte: o mint do PYUSD carrega oito extensões TLV, mintCloseAuthority, permanentDelegate, transferFeeConfig, o par confidentialTransfer, transferHook, metadataPointer, tokenMetadata. Configurado mas dormente: eu reli o mint da mainnet enquanto redigia isto em 2026-08-23 e o `programId` do hook é null e a taxa está em 0 basis points, máximo 0 — toda opção de poder comprada, toda uma desligada. Como um mint com permanent delegate é negociado no CP-Swap afinal você já sabe: o bypass da whitelist, e você consegue resolver isso em cinco segundos porque o endereço do mint do PYUSD é uma das quatro strings em `MINT_WHITELIST` no `token.rs` L18-23, ali mesmo na saída do `sed` que você já imprimiu. Entregue as extensões com forma de compliance, mantenha as de poder inativas, e mesmo então a roteabilidade veio de uma porta especial, não da regra geral.

![Diagrama das oito extensões TLV do PYUSD como lidas ao vivo em 2026-08-23, com a taxa de transferência em zero basis points e o program ID do transfer hook null, ilustrando extensões de poder configuradas-mas-dormentes.](assets/v05-diagram.webp)

### A roteabilidade é por venue, e a maior parte do mapa está sem luz

Tudo até aqui é a lei de um venue. Segure esse limite com firmeza, porque no momento em que o SPROUT rotear no CP-Swap o seu cérebro vai querer escrever a frase "o SPROUT é negociável", e essa frase não é algo que você sabe. O mapa de venues em si é material da lição passada, então segure só as manchetes dele: o AMM v4 e o Stable AMM sob a mesma marca Raydium aceitam só SPL clássico; a tabela de suporte no nível de docs da Orca mais o Token Badge revisado por humanos fazem dessa resposta uma decisão por mint que você pode solicitar mas nunca prever por completo a partir do seu conjunto de extensões; o Meteora DBC roda a contra-tese e suporta configurações de transfer hook; e a política de roteamento por extensão da Jupiter simplesmente não é encontrada nos docs dela, em nenhuma varredura que este curso rodou. O que hoje acrescenta é a consequência de design: a única afirmação honesta sobre qualquer venue é verifique na hora de escrever, contra o seu mint, na API ao vivo deles.

As carteiras são ainda mais escuras. Se a Phantom mostra um aviso de taxa, se a Backpack renderiza o metadata pointer, se a Solflare sinaliza um hook: não verificado, nada disso, em todas as passagens de pesquisa atrás deste curso. Eu poderia colar uma matriz de compatibilidade plausível aqui e você acreditaria nela, e é precisamente por isso que eu não vou. Uma matriz congelada de afirmações não medidas é pior que nenhuma matriz, porque ela falha em silêncio no único lugar em que você parou de checar.

![Comparação de cinco venues, da allowlist em código do CP-Swap da Raydium e da regra de SPL clássico do AMM v4 até a revisão de Token Badge da Orca, a política ausente da Jupiter, e o suporte a hook da Meteora, sinalizando as linhas de verifique-você-mesmo.](assets/v06-comparison.webp)

Por que tanta escuridão em um ecossistema que está amadurecendo? Em parte porque o terreno de fato se move, e em parte porque as pessoas que costumavam mapeá-lo pararam. O repositório developer-content da Solana Foundation, a fonte por trás de anos de material oficial de curso, foi arquivado em 2025-01-24. Todo curso oficial congelou antes de as regras de venue contra as quais você está projetando existirem. Não existe matriz canônica porque ninguém é pago para manter uma verdadeira, e os terceiros que publicam uma estão congelando os mesmos fatos em movimento que você. Isso não é motivo para desespero; é a restrição de design em volta da qual o seu relatório é construído. O entregável durável é uma matriz mais o método datado para re-derivar cada célula.

![Linha do tempo do lançamento do PYUSD em maio de 2024, passando pelo arquivamento da educação oficial da Solana em janeiro de 2025, até as leituras datadas de 2026 desta lição, terminando em uma flecha de reverificar-no-lançamento.](assets/v07-timeline.webp)

### O que um item de verificação deve ao leitor

"Verifique no seu alvo" pode ser uma disciplina profissional ou um encolher de ombros que moveu o trabalho sem mover nenhum conhecimento, e a diferença é mecânica. Um item de verificação merece o seu lugar quando carrega três partes. Um **alvo**, nomeado de forma específica o bastante para ser aberto: não "carteiras" mas a Phantom na versão que você testou. Uma **afirmação**, enunciada de forma precisa o bastante para que o resultado de um leitor possa contradizê-la: não "a exibição pode variar" mas "se a dedução da taxa de transferência é mostrada antes de assinar". E um **procedimento**, que é o que um leitor roda mais o que cada resposta possível significa.

Tire qualquer uma das partes e veja o item apodrecer de um jeito previsível. Alvo faltando, e você escreveu um aviso legal, que te protege e não ajuda ninguém. Afirmação faltando, e um leitor que roda o seu procedimento não consegue dizer se o que ele viu concorda com você ou te refuta, então o resultado dele nunca volta. Procedimento faltando, e você passou uma tarefa adiante em vez de um método, que é a versão polida de chutar.

Existe uma regra correspondente para as afirmações que você verificou, e é a mais curta: date-as. Uma afirmação verificada precisa de uma data para poder apodrecer de forma visível; uma afirmação não verificada precisa de um método para que alguém possa resolvê-la. Toda afirmação datada nesta lição segue a primeira regra, inclusive as que eu li da mainnet hoje de manhã.

![Tabela contrastando versões decorativas e utilizáveis de um item de verificação em alvo, afirmação e procedimento, com a regra de que afirmações verificadas carregam uma data e afirmações não verificadas um método.](assets/v08-table.webp)

Essa forma de três partes não é uma convenção de escrita, é uma estrutura de dados, e no lab que você está a ponto de construir ela se torna uma interface TypeScript com exatamente três campos. O que é a coisa boa de codificar honestidade em um programa: um encolher de ombros não passa na checagem de tipos.

## Lab: produza o relatório de roteabilidade

O artefato é o `routability-report.ts`, a forma acabada do R6. Para fixar o nome uma vez, já que ele agora apareceu duas vezes: o R6 É o relatório de roteabilidade; o preditor que você construiu na lição passada era o rascunho dele, e ele viaja dentro deste arquivo como o marcador. Ele consome o seu preditor e o seu verificador de combinações, marca o conjunto final do SPROUT, enuncia os venues verificados, e se recusa a enunciar os não verificados. O critério é a forma usual do curso: o `npx tsx routability-report.ts` precisa emitir o conjunto final de extensões com cada extensão marcada como whitelisted ou refused conforme a allowlist, os venues verificados-roteáveis, e uma lista não vazia de verifique-no-seu-alvo cobrindo Orca, Jupiter e exibição em carteira.

1. Trabalhe na pasta de lab do m05-l1, onde o `predict-routability.ts` já vive, e traga o verificador de combinações do lab dele para que os dois imports resolvam de um diretório só: `cp ../m01-l4/check-combo.ts .`. Nada novo para instalar se você fez aquele lab; se você está começando do zero, os pins do runner são as mesmas duas ferramentas de dev, rechecadas hoje (2026-08-23). O `tsx@4.23.12` era o npm latest naquela leitura; o `typescript@5.9.3` é uma retenção deliberada, já que o npm latest já passou para a linha 7 e este curso fixa a versão contra a qual os labs dele foram verificados. Os dois números apodrecem, então rode `npm view tsx version` você mesmo no dia em que fizer o scaffold:

```bash
npm install -D tsx@4.23.12 typescript@5.9.3
```

2. Antes de marcar qualquer coisa, puxe a verdade de campo sobre o que o seu mint negociável de fato carrega. Você tem a CLI do `spl-token` da sondagem docs-versus-código do m01-l4 (o bundle Agave do m01-l3 pode ter incluído ela; se o seu não incluiu, `cargo install spl-token-cli` fecha a lacuna). Aponte-a para onde você de fato cunhou o SPROUT, que no caminho default é o seu surfnet local; forks são efêmeros, então se o seu reiniciou desde que você cunhou, cunhe de novo primeiro com os dois comandos da abertura do m05-l1 (e troque para `--url devnet` se você seguiu o fallback de devnet do m02-l4):

```bash
spl-token display <YOUR_SPROUT_MINT> --url http://127.0.0.1:8899
```

   Passe pela supply e pelos decimals até o bloco de extensões. Para o mint que você quer, ele lista um transfer fee config com os seus basis points e máximo, um metadata pointer cujo endereço é o próprio mint, e o token metadata guardando o nome, o símbolo e a URI do SPROUT. Três entradas, nenhuma quarta. Se uma linha `Transfer Hook` ainda estiver sentada ali porque você cunhou a variante do m03 e nunca recunhou, pare aqui: aquele mint é a variante com hook, não o mint de lançamento, e nenhuma quantidade de marcação cuidadosa no relatório vai mudar o que a checagem da pool lê. Se a saída discordar do conjunto na sua cabeça, o mint ganha, e o seu relatório descreve o mint em vez das suas intenções. Essa checagem de trinta segundos é a diferença entre um relatório e um desejo, e é o único passo deste lab que eu me recusaria a pular.

3. Agora o truque central do relatório: derive cada marca do preditor em vez de copiar a allowlist para um segundo arquivo. Duas cópias de uma lista divergem; uma função sondada duas vezes não consegue. Comece o `routability-report.ts` com os imports e o marcador. O marcador é o único movimento esperto do relatório, duas linhas, então leia em vez de passar o olho:

```typescript
// routability-report.ts: SPROUT's launch-venue routability report (R6, final).
import { isRoutable, type MintProfile } from "./predict-routability";
import { checkCombo } from "./check-combo";

type Tag = "whitelisted" | "refused";

interface ExtensionRow {
  extension: string;
  tag: Tag;
  rationale: string;
}

interface VerifyItem {
  target: string;
  claim: string;
  howToVerify: string;
}

// Tag an extension by asking the PREDICTOR: a hypothetical Token-2022 mint
// carrying only this extension either passes CP-Swap's check or it does not.
function tagOf(extension: string): Tag {
  const probe: MintProfile = { tokenProgram: "token2022", extensions: [extension] };
  return isRoutable(probe) ? "whitelisted" : "refused";
}
```

4. Dê a toda linha uma justificativa, porque uma marca sem razão é folclore com formatação melhor. As entradas da allowlist dizem o que a pool ainda consegue fazer; as entradas refused carregam a classe de razão do venue, aquelas que você consegue defender a partir do fonte. Depois declare o conjunto de lançamento e o conjunto descartado, com cada descarte registrando para onde a capacidade foi em vez de fingir que ela nunca existiu:

```typescript
const RATIONALE: Record<string, string> = {
  TransferFeeConfig: "declared, capped fee the pool can read and price in",
  MetadataPointer: "display only; changes nothing about how a transfer executes",
  TokenMetadata: "display only; name/symbol/URI live on the mint itself",
  InterestBearingConfig: "UI-level accrual; raw token amounts are untouched",
  ScaledUiAmount: "UI multiplier; raw token amounts are untouched",
  TransferHook: "issuer code runs on every transfer, arbitrary CU; refused",
  PermanentDelegate: "issuer can move tokens from any account, pool vault included; refused",
  DefaultAccountState: "issuer decides whether new accounts can transact at all; refused",
  ConfidentialTransferMint: "encrypted amounts cannot be priced by an AMM; refused",
};

function buildRows(extensions: string[]): ExtensionRow[] {
  return extensions.map((extension) => ({
    extension,
    tag: tagOf(extension),
    rationale: RATIONALE[extension] ?? "no rationale recorded; justify before shipping",
  }));
}

// The tradeable mint: fees fund the treasury, metadata is native. All allowlisted.
const LAUNCH_SET = ["TransferFeeConfig", "MetadataPointer", "TokenMetadata"];

// Considered and dropped, with the decision recorded.
const DROPPED: ExtensionRow[] = [
  {
    extension: "TransferHook",
    tag: "refused",
    rationale:
      "dropped from the tradeable mint: costs CP-Swap outright; hook lives on the non-pooled issuer variant only",
  },
  {
    extension: "ConfidentialTransferMint",
    tag: "refused",
    rationale:
      "stays on the shelved issuer branch from m04; the confidential path cannot be pooled, and venues that admit such mints at all (Orca's table) do so for public transfers only",
  },
  {
    extension: "PermanentDelegate",
    tag: "refused",
    rationale:
      "an m02 catalog consideration, never adopted: refused by the allowlist, and SPROUT's product has no clawback requirement to justify the venue cost",
  },
  {
    extension: "DefaultAccountState",
    tag: "refused",
    rationale:
      "an m02 catalog consideration, never adopted: the classic just-in-case trap the decision flowchart warns about; gated onboarding is not a SPROUT requirement",
  },
];
```

5. As duas seções de venue são onde a honestidade fica estrutural. O `VERIFIED_ROUTABLE` guarda só venues em que a roteabilidade foi demonstrada, que para você é exatamente uma entrada, o pool-create no fork da mainnet da lição passada. O `VERIFY_YOURSELF` guarda toda afirmação que você não tem permissão para afirmar, cada uma com o procedimento concreto que um leitor roda no próprio alvo dele. Este apoio carrega os três alvos que o critério exige; a redação de cada `howToVerify` é sua para afiar no challenge:

```typescript
// Venues where routability was DEMONSTRATED, not inferred. If you took m05-l1's degrade
// path and never landed the pool-create, this entry is a claim you have not earned: say
// "predictor verdict ROUTABLE, matched against token.rs @ 244e124; pool-create unrun" instead.
const VERIFIED_ROUTABLE = [
  "Raydium CP-Swap: predictor verdict ROUTABLE, confirmed by a mainnet-fork pool-create (m05-l1 lab)",
];

// Claims we do NOT assert. Each names the target, the unverified claim, and
// how the reader verifies it in the venue they actually ship to.
const VERIFY_YOURSELF: VerifyItem[] = [
  {
    target: "Orca",
    claim: "whether SPROUT's extension set clears Orca's Token Badge review",
    howToVerify:
      "check the Token Badge requirements in Orca's current docs and submit the mint for review; the badge is a per-mint decision, not a published allowlist",
  },
  {
    target: "Jupiter",
    claim: "whether Jupiter routes this Token-2022 extension set",
    howToVerify:
      "no per-extension routing policy was found in Jupiter's docs at the time of writing. Jupiter's live API sees mainnet only and your SPROUT lives on a local fork, so the runnable version is two-step: today, quote a mainnet mint with the same extension shape (fee + metadata pointer + metadata) to learn the policy; at launch, quote your own mint the moment it exists on mainnet. A returned route is a yes; token-not-found or no-route is the no",
  },
  {
    target: "Wallets (Phantom, Backpack, Solflare)",
    claim: "how each wallet displays the transfer fee and whether it warns on any extension",
    howToVerify:
      "load the mint in the wallet you target and observe; per-extension display behavior is not standardized and was not measured by this course",
  },
];
```

6. Emita e se autoavalie. O relatório imprime as suas quatro seções, e então vira as próprias regras contra si mesmo: uma extensão refused no conjunto de lançamento, uma violação da matriz de combinações, uma rejeição do preditor, ou uma lista de verifique-você-mesmo vazia, cada uma sai com código diferente de zero. Esse último critério importa mais; um relatório sem nada sobrando para verificar não é minucioso, é congelado:

```typescript
function main(): void {
  const rows = buildRows(LAUNCH_SET);
  const combo = checkCombo(LAUNCH_SET);
  const verdict = isRoutable({ tokenProgram: "token2022", extensions: LAUNCH_SET })
    ? "ROUTABLE"
    : "REJECTED";

  console.log("# SPROUT routability report\n");
  console.log("## Final launch-venue extension set\n");
  for (const r of rows) {
    console.log(`- ${r.extension} [${r.tag}]: ${r.rationale}`);
  }
  console.log(`\nCombo matrix: ${combo.valid ? "valid" : `INVALID (${combo.reason})`}`);
  console.log(`CP-Swap predictor verdict: ${verdict}\n`);

  console.log("## Considered and dropped\n");
  for (const r of DROPPED) {
    console.log(`- ${r.extension} [${r.tag}]: ${r.rationale}`);
  }

  console.log("\n## Verified routable\n");
  for (const v of VERIFIED_ROUTABLE) console.log(`- ${v}`);

  console.log("\n## Verify in your target (not asserted by this report)\n");
  for (const v of VERIFY_YOURSELF) {
    console.log(`- ${v.target}: ${v.claim}`);
    console.log(`  how: ${v.howToVerify}`);
  }

  const refusedInSet = rows.filter((r) => r.tag === "refused");
  if (refusedInSet.length > 0) {
    console.error(`\nGATE FAIL: refused extension(s) in the launch set: ${refusedInSet.map((r) => r.extension).join(", ")}`);
    process.exit(1);
  }
  if (!combo.valid) {
    console.error(`\nGATE FAIL: combo matrix violation: ${combo.reason}`);
    process.exit(1);
  }
  if (verdict !== "ROUTABLE") {
    console.error("\nGATE FAIL: predictor rejects the launch set");
    process.exit(1);
  }
  if (VERIFY_YOURSELF.length === 0) {
    console.error("\nGATE FAIL: the verify-yourself section is empty; that is a frozen-matrix report");
    process.exit(1);
  }
  console.log("\nAll gates pass: allowlist-clean set, valid combo, non-empty verify list.");
}

main();
```

   Rode:

```bash
npx tsx routability-report.ts
```

   Você deve ver as quatro seções em ordem, `Combo matrix: valid`, `CP-Swap predictor verdict: ROUTABLE`, três entradas de verifique-você-mesmo, e a linha final `All gates pass` com código de saída 0. Depois prove que os gates são reais: adicione `"DefaultAccountState"` ao `LAUNCH_SET`, rode de novo, e veja o mesmo script se recusar a entregar o próprio relatório: uma linha marcada refused, gate fail, exit 1. Um checkpoint que não pode falhar nunca foi um checkpoint. Tire a extensão de volta.

![Fluxograma do script de relatório consumindo o preditor e o verificador de combinações, emitindo quatro seções, e depois falhando o próprio build em uma extensão refused, combinação inválida, conjunto rejeitado, ou lista de verifique-você-mesmo vazia.](assets/v09-flowchart.webp)

## Challenge

A metade solo é a seção de honestidade, e você a escreve sem apoio. Pegue as três entradas de `VERIFY_YOURSELF` e transforme cada `howToVerify` da minha redação de exemplo em um procedimento que você passaria para um colega: para a Orca, o que você de fato submeteria e por onde a resposta do Token Badge volta; para a Jupiter, a requisição exata de cotação que você faria uma vez que o SPROUT existisse na mainnet, mais o mint de mainnet com a mesma forma que você sondaria hoje como substituto dele, e como se parece uma resposta roteada versus não roteada; para as carteiras, quais telas você abriria e o que você registraria. Depois adicione pelo menos um item de verificação que eu não te dei. Candidatos em que você já roçou: se a allowlist do CP-Swap ainda bate com o fonte no commit que você fixou, já que uma allowlist impressa apodrece como qualquer matriz, ou se as entradas do bypass da whitelist mudaram. A barra de aceitação é a do brief, palavra por palavra: toda extensão incluída é justificada contra a allowlist, e nenhuma afirmação não verificada de carteira ou agregador é enunciada como fato. Leia o seu relatório acabado caçando uma frase que afirme algo que você nunca mediu. Se você não encontrar nenhuma, e os gates passarem, o R6 está feito e o design do SPROUT está fechado.

Um pedido antes de você fechar a pasta. Se as suas próprias rodadas de verificação contradisserem qualquer coisa datada nesta lição, uma página de política da Jupiter que agora existe, um fluxo de badge da Orca que mudou de lugar, uma sexta extensão na allowlist do CP-Swap, poste a afirmação exata e o que você encontrou no canal de feedback do curso. As minhas leituras são datadas em 2026-08-23 e o argumento inteiro desta lição é que afirmações datadas decaem; um aprendiz que pega uma decaindo é o sistema funcionando, e o formato de relatório que você acabou de construir é exatamente onde tal captura pertence.

O SPROUT está resolvido: um token de economia que você consegue construir, rotear e defender no papel, com um relatório que diz onde ele é negociado e admite o que ele não sabe. Isso fecha a metade fungível deste curso. O próximo módulo deixa os fungíveis para trás em direção à camada de colecionáveis: a pilha de NFT de 2026, os metadados que as carteiras de fato leem, e a realidade de royalties que ninguém anuncia. Traga o mesmo ceticismo; o lado dos colecionáveis tem ainda mais folclore para queimar.
