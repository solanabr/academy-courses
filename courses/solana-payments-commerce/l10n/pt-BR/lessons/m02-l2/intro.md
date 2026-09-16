# O elenco de stablecoins de 2026: PYUSD, Token-2022 e como o USDC viaja

Na lição passada você construiu o transfer-kit e mandou USDC de verdade na devnet: unidades base seguras quanto aos decimais, um memo, uma reference key, uma signature que você conseguia achar de novo. O kit funciona. Ele também tem uma mina dentro, e hoje um cliente pisa nela: ele paga em PYUSD, o kit monta e assina uma transação que parece perfeitamente bem-formada, e a rede recusa ela na porta: o RPC faz um dry-run de toda transação antes de transmitir ela (um passo chamado simulação de preflight), a simulação entrega ao Token program clássico um mint do qual ele não é dono, e o envio volta como uma promise rejeitada. Nenhuma transação falha em um explorer, nenhuma signature de que a blockchain já tenha ouvido falar, e nenhum dinheiro movido.

O PYUSD é uma stablecoin de dólar. Seis decimais, igual ao USDC. Mesma palavra no rótulo. Então por que o código exato que move USDC sem problema tem a transferência de PYUSD dele jogada para fora na porta? Rode isto antes de qualquer teoria. Dois curls, nenhuma carteira necessária:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",{"encoding":"jsonParsed"}]}' \
  | grep -o '"owner":"[^"]*"'
```

Esse é o mint do USDC. Você recebe de volta `"owner":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"`. Agora troque pelo mint do PYUSD, `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo`, e rode de novo:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
  | grep -o '"owner":"[^"]*"'
```

`"owner":"TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"`. Endereço diferente. Programa diferente. Mesma palavra no rótulo, uma máquina diferente por baixo, e o seu kit cravou a primeira máquina. Esse único campo, `owner`, é o bug inteiro, e consertar ele direito é esta lição.

## Resumo

Hoje o transfer-kit aprende a ler antes de assinar. Primeiro a teoria: o que significa um mint ter um token program como dono, o que é o Token-2022, e como o mint do PYUSD carrega oito extensões que dão ao emissor dele poderes que um lojista precisa conhecer, incluindo um que consegue tirar PYUSD de qualquer carteira. Depois um tour rápido pelo resto do elenco de 2026 que você vai de fato ser convidado a aceitar: EURC, USDG, USDT e a turma que rende juros, que a gente entrega de propósito para outro curso. Depois o CCTP, porque o USDC que te paga muitas vezes não nasceu na Solana e vale saber como ele chegou aqui. O lab estende o kit com `detectTokenProgram`, um relatório `readMint` e um `sendStablecoin` ciente do programa, e entrega um smoke check `verify:roster` que prova que os dois token programs funcionam de ponta a ponta.

Como o trabalho está dividido hoje: a teoria e a maior parte do lab são trabalhadas juntos, eu digito primeiro e você acompanha. O único buraco que eu deixo no scaffold, o switch de programa dono em si, você preenche como o desafio de completion. A enumeração ao vivo do PYUSD e o envio nos dois mints no final são só seus, sem guia.

## Mesmo ticker, máquina diferente

### O programa dono: um campo decide como o dinheiro se move

Toda conta na Solana tem um campo `owner` nomeando o programa que tem permissão de alterar ela. Você encostou nessa ideia de lado na lição passada, quando a gente derivou ATAs. Agora ela vira estrutural: um mint tem exatamente um token program como dono, e toda instrução que toca nesse mint precisa ser endereçada a esse programa. Não a "o token program" no abstrato. Àquele que está no campo `owner`.

Por anos houve efetivamente uma resposta só, o Token program clássico em `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`, então uma geração de código de pagamento cravou ele e se deu bem. Aí chegou o Token-2022 em `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb`: um segundo token program, separado, mesma interface central, mais um sistema de extensões que o programa clássico nunca teve. Ele não substituiu o programa clássico. Os dois rodam lado a lado, e um mint mora em um ou no outro, para sempre. O USDC é clássico. O PYUSD é Token-2022. Os dois são dólares; o dólar não é a máquina.

![Dois cartões de mint, USDC e PYUSD, cada um com uma seta de dono para um token program diferente, acima de uma regra dizendo que transferências têm que mirar no programa dono do mint.](assets/v01-diagram.webp)

Então o conserto do kit não é "adicionar suporte a PYUSD", mas algo mais simples: pare de assumir, comece a ler. Pergunte para a blockchain quem é dono do mint, depois monte a transferência contra essa resposta. Uma leitura de mint por pagamento, cacheada se você quiser, e a classe inteira de falhas de programa errado desaparece. O lab transforma isso numa função de cinco linhas, e as cinco linhas importam menos que o hábito: nestes trilhos, o mint é o arquivo de configuração, e ele é público. Leia.

Seja preciso sobre onde essa falha aterrissa, porque isso muda como você depura ela. Nada lança erro enquanto você monta. O seu `TransferChecked` da lição passada crava o id do programa clássico e deriva contas de token debaixo dele, e os dois são bytes de aparência válida, então a mensagem monta e assina sem problema. A recusa pertence ao Token program clássico: recebendo uma conta de mint da qual ele não é dono, ele confere o campo de dono e dá erro. Mas com a cauda de envio do kit ele nunca chega a dizer isso on-chain. Antes de transmitir, o RPC passa a sua transação assinada por um dry-run contra o estado atual — a **simulação de preflight** — e a checagem de dono dispara ali. O `sendAndConfirmTransactionFactory` expõe isso como uma promise rejeitada carregando `Transaction simulation failed` mais o erro do programa, e a transação nunca aterrissa: nenhuma entrada em explorer, e o `getSignatureStatuses` na sua signature retorna null mesmo com a busca no histórico ligada. Só se você tivesse desabilitado esse dry-run (`skipPreflight`) é que a mesma recusa aconteceria on-chain, te custaria a taxa e deixaria uma transação falha para trás. O passo 6 do lab faz você pisar na mesma mina pela CLI, o que pega ela numa terceira costura, ainda mais cedo.

![Uma transferência para o programa errado monta e assina sem problema, depois morre na simulação de preflight do RPC quando o Token program clássico encontra uma divergência de dono — recusada antes da transmissão, sem rastro on-chain.](assets/v02-flowchart.webp)

Irritante no pior momento, claro, mas este é o bom modo de falha: o programa recusou em vez de mover dinheiro errado. Segure esse instinto conforme a gente avança, porque recusas barulhentas são um recurso que você vai construir dentro do seu próprio kit hoje, desta vez localmente, antes de qualquer signature ser gasta.

A divisão de propriedade alcança mais um lugar que você não adivinharia: endereços. A linha sobre seeds da lição passada dizia que uma ATA é calculada a partir do dono e do mint. A verdade completa é que o token program que é dono também faz parte da derivação, então a conta de PYUSD do seu cliente e a conta de USDC dele diferem não só pela seed do mint, mas pela seed do programa. Derive a ATA de um mint Token-2022 com o programa clássico nas seeds e você recebe um endereço perfeitamente válido na aparência que nenhuma carteira jamais vai financiar. É por isso que o `resolveAta` ganha um terceiro parâmetro no lab, e por que o programa detectado tem que atravessar todo passo do envio, não só a instrução de transferência. Uma detecção, usada em todo lugar.

### Extensões TLV: o que o PYUSD carrega de verdade

Agora a metade interessante. Por que o Token-2022 existe, afinal? Porque emissores viviam precisando de poderes que o programa clássico não conseguia expressar: taxas na transferência, metadados no próprio mint, valores confidenciais, controles de compliance. A resposta do Token-2022 são extensões: registros tipados opcionais anexados a um mint ou a uma conta de token, codificados como TLV, tipo-comprimento-valor. Cada registro diz o que ele é, qual o comprimento dele, e depois a carga dele. Um mint opta por um conjunto de extensões na criação, e qualquer um consegue ler esse conjunto direto da conta.

Uma escolha de projeto torna o seu lab inteiro possível: as extensões são anexadas depois do layout clássico do mint, que continua byte a byte intacto na frente. É por isso que um único decodificador consegue ler os dois tipos de mint, e por que ferramentas antigas que só entendem o prefixo clássico ainda leem corretamente a oferta e os decimais de um mint Token-2022. Os poderes novos moram estritamente no apêndice.

O PYUSD é o exemplo trabalhado para o qual o ecossistema inteiro aponta. O mint dele carrega oito extensões TLV: mintCloseAuthority, permanentDelegate, transferFeeConfig, confidentialTransferMint, confidentialTransferFeeConfig, transferHook, metadataPointer, tokenMetadata. Oito é a contagem que você obtém contando entradas TLV; às vezes você vai ouvir sete, contando o par confidencial como uma suíte só. A gente lê tudo isso ao vivo no lab, mas três das oito merecem a atenção total de um lojista agora.

![O mint do PYUSD desenhado como um cartão com o layout clássico em cima e oito linhas de extensão TLV anexadas, três delas marcadas para atenção do lojista.](assets/v03-diagram.webp)

**permanentDelegate** é a que merece que você sente com ela. Ela nomeia uma autoridade permanente, aqui o emissor, que consegue tirar PYUSD da conta de token de qualquer holder. Qualquer carteira, qualquer saldo, nenhuma signature do holder. Isso é capacidade de confisco, e não é um bug nem um risco de hack: é política do emissor, a expressão on-chain da obrigação de uma empresa regulada de congelar e reaver fundos por ordem judicial. O seu código de transferência não adiciona isso, não consegue remover, e nunca aciona. Mas quando você precifica uma venda em PYUSD, você aceita um ativo cujo emissor mantém esse poder, e você deveria saber disso do mesmo jeito que você sabe que o seu adquirente de cartão consegue reverter uma liquidação.

**transferFeeConfig** significa que o mint pode cobrar uma taxa, em centésimos de ponto percentual, retida de toda transferência. Aqui está a armadilha de integração: a presença da extensão não te diz nada sobre a alíquota. A taxa é um número na conta do mint, a autoridade de taxa pode mudar ela, e uma mudança passa a valer numa virada de epoch — mas não na próxima. O Token-2022 carimba a alíquota que está entrando com `newer_fee_start_epoch = current epoch + 2`, então ela ativa duas viradas de epoch adiante, e o código-fonte do programa diz por quê com todas as letras: "set two epochs ahead to avoid rug pulls at the end of an epoch." Um atraso de uma epoch declarado nos últimos minutos de uma epoch não seria aviso nenhum. Uma **epoch** é a unidade de agendamento da Solana, um bloco de exatamente 432,000 slots (o `getEpochSchedule` vai te dizer isso), que no tempo-alvo de slot de 300ms que este curso usa desde o módulo 1 dá cerca de 36 horas, um dia e meio — derive isso em vez de decorar, porque tempo de slot é o número que não para de se mexer e material mais velho citando "cerca de dois dias" está citando a era dos 400ms. Depois dobre, porque duas epochs é o que você de fato ganha: uns três dias de aviso no tempo de slot de hoje. É esse o jeito de a rede dizer "não agora, mas na próxima troca combinada", e é por isso que uma mudança de taxa é anunciada em vez de aplicada. A taxa do PYUSD está atualmente configurada em zero centésimos de ponto percentual. Eu estou te contando isso como um fato sobre hoje, verificado ao vivo enquanto eu escrevia, e o lab faz o seu kit ler o valor atual do mint em tempo de execução, porque "atualmente zero" é exatamente o tipo de fato que você nunca crava. Uma taxa que é zero hoje pode ser diferente de zero depois, em silêncio, sem nenhuma mudança de código do seu lado.

E entenda onde uma taxa diferente de zero morderia: ela é retida do valor transferido. Mande 100 tokens num mint com uma taxa de 50 centésimos de ponto percentual e a conta do destinatário é creditada com 99.50; a meia unidade retida se acumula para a autoridade de taxa recolher. Para uma loja isso quer dizer que o preço que o seu checkout exibe e o valor que o seu livro-razão recebe param de bater no momento em que uma taxa é ligada, e todo relatório de conciliação rio abaixo herda a diferença. É essa a razão concreta de o `readMint` expor os centésimos de ponto percentual como campo de primeira classe: um checkout que conhece a alíquota ao vivo consegue reprecificar, avisar ou recusar. Um checkout que assumiu zero simplesmente perde margem em silêncio.

**transferHook** deixa um mint anexar um programa que roda em toda transferência, e que pode adicionar contas exigidas a mais na instrução. No PYUSD ela está configurada mas dormente: a extensão está presente e o id do programa de hook é null, então as transferências hoje não precisam de nada a mais. O seu kit vai conferir isso e recusar em alto e bom som se algum dia encontrar um mint com um hook vivo, porque uma transferência montada sem as contas do hook falha de jeitos confusos. Construir a interface de hook de ponta a ponta explicitamente não é o nosso trabalho: essa profundidade do lado de quem cria — a interface de transfer hook e o resto das entranhas das extensões — é território do curso Digital Assets, Tokenization and Token Extensions. Este curso lê e roteia, nada além disso.

![Uma tabela de três linhas de poderes do emissor: um delegado permanente que consegue confiscar tokens, uma taxa de transferência mutável atualmente em zero centésimos de ponto percentual, e um transfer hook dormente.](assets/v04-comparison.webp)

As cinco restantes, rapidinho: mintCloseAuthority deixa o emissor fechar a própria conta do mint; o par confidencial habilita transferências com valor criptografado (opt-in, e não é algo de que um checkout precise); metadataPointer e tokenMetadata põem o nome e o símbolo do token na conta do mint em vez de num registro externo. Poderes comuns, que vale nomear, nada sobre o que uma integração de pagamento precise agir.

Aqui está a troca honesta em torno da qual esta lição foi construída. Um kit que fala os dois token programs é mais ramificação, mais dependências, mais código do que o que você tinha ontem. E mints Token-2022 podem carregar poderes que um lojista precisa aceitar conscientemente: um delegado permanente quer dizer que o emissor consegue reaver fundos, e uma taxa de transferência pode ser ligada depois. A segurança não está em evitar o Token-2022, o que significaria recusar o PYUSD e metade do elenco abaixo. A segurança está em ler o mint em tempo de execução, toda vez, e nunca confiar no ticker. O ticker diz dólar. O mint diz de que tipo.

Vale perguntar por que o PayPal se deu ao trabalho de toda essa maquinaria. A resposta é que funcionou: o PYUSD chegou a cerca de US$ 332 milhões de valor de mercado em quatro meses do lançamento dele na Solana, com o PayPal registrado no Breakpoint 2024 falando sobre por que escolheram estes trilhos, e o conjunto de extensões do mint (poder de confisco, hook dormente, capacidade confidencial) é exatamente o que um emissor regulado precisa para satisfazer os reguladores dele enquanto liquida em segundos. As extensões são o departamento de compliance, compilado.

![Uma linha do tempo de quatro pontos, do lançamento do PYUSD na Solana, passando por uns 332 milhões de dólares de valor de mercado e pelo PayPal no Breakpoint 2024, até a leitura ao vivo do mint em 2026.](assets/v05-timeline.webp)

### O resto do elenco de 2026

O seu checkout vai ser chamado para mais do que USDC e PYUSD. Aqui está o resto do elenco, um parágrafo honesto para cada, e o hábito de cima vale para toda linha: leia o mint, acredite na leitura.

**EURC** é a stablecoin de euro da Circle, e ela importa porque precificar em euros sem uma perna de câmbio é um recurso de verdade para uma loja com clientes europeus. Na Solana ela é um mint de Token clássico em `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` (fixado aqui a partir de uma leitura ao vivo, 2026-08-22), seis decimais. O seu kit, depois de hoje, lida com ela pelo ramo clássico, sem nenhum caso especial. A única novidade está nos seus livros, não na blockchain: é uma moeda diferente, não um dólar diferente.

**USDG** é o Global Dollar, emitido pela Paxos. E aqui está um detalhe que eu sinceramente gostei de encontrar enquanto escrevia isto: leia o mint do USDG em `2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH` e você recebe Token-2022, seis decimais e as mesmas oito extensões do PYUSD, entrada por entrada (verificado ao vivo, 2026-08-22). Isso não é coincidência, é uma impressão digital: a Paxos também é a emissora por trás do PYUSD, e é assim que se parece o template padrão de um emissor regulado. Dois nomes de marca diferentes, uma forma de máquina só. O seu kit não está nem aí, o que é o ponto inteiro de hoje.

**USDT** é o mais velho da tabela, em `Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB`, um mint de Token clássico que você já adicionou ao kit na lição passada. Ele continua em todo lugar, especialmente em fluxos que começaram fora do país ou fora da Solana. Aceite ele pelo mesmo caminho de código do USDC e siga em frente; as complicações dele são questões de negócio e de jurisdição, não questões de integração.

Depois tem a turma que rende juros: stablecoins cujo saldo ou valor de resgate cresce porque a reserva gera juros. Elas parecem só mais um mint, e tratar elas como USDC puro é um erro, porque as mecânicas delas (saldos que rebaseiam, preços de cota que acumulam, restrições de transferência) alcançam exatamente a contabilidade que a sua loja faz. A gente não vai cobrir elas, de propósito; as mecânicas delas são território de DeFi and RWA Engineering. Se um parceiro te pedir para aceitar uma, essa profundidade é o pré-requisito, não este parágrafo.

![Uma tabela do elenco com USDC, PYUSD, EURC, USDG e USDT, com o programa dono e os decimais de cada mint, mais uma linha de repasse para tokens que rendem juros.](assets/v06-comparison.webp)

### Como o USDC viaja: CCTP em uma seção

Mais uma peça da história do elenco, porque o USDC que te paga muitas vezes começou a vida em outro lugar. Um comprador tem USDC na Ethereum ou na Base; o seu checkout está na Solana. Como o dólar dele vira um dólar aqui?

A velha resposta das pontes era lock-and-wrap: estacione o token de verdade num pool na blockchain de origem, cunhe um IOU no destino. Funciona até o pool ser drenado por um exploit, e o IOU wrapped só vale o que vale a ponte atrás dele. A resposta da Circle para o USDC é o CCTP, o Cross-Chain Transfer Protocol, e o mecanismo é diferente em espécie: burn-and-mint. O USDC é queimado na blockchain de origem, a Circle atesta a queima, e USDC nativo é cunhado no destino. Sem pools de colateral travado, sem substituto wrapped, e o que chega é o mesmo mint nativo de USDC que o seu kit já manda, indistinguível de qualquer outro USDC na Solana.

Dois fatos específicos da Solana para guardar. No esquema de endereçamento do CCTP toda blockchain é um domínio numerado, e a Solana é o domínio 5 (a Ethereum é o domínio 0); você vai ver esse número em mensagens e logs do CCTP quando estiver depurando uma chegada cross-chain. E a velocidade, com a direção enunciada com cuidado, porque este é o detalhe que todo resumo do CCTP entende ao contrário. Tanto o caminho padrão quanto o rápido esperam a blockchain de **origem** — a própria documentação da Circle escopa a disponibilidade do Fast Transfer a blockchains de origem, já que o que está sendo esperado é a queima se tornar irreversível onde ela aconteceu. Então os amplamente citados "cerca de 8 segundos" são o número para a Solana *como origem*, não para chegadas na Solana. Um comprador vindo da Ethereum espera a finality da Ethereum passar, e o caminho rápido dele é correspondentemente mais lento que 8 segundos, ainda sendo uma grande melhora sobre a rota padrão. A tabela da Circle torna a diferença concreta, e ela é maior do que a maioria dos textos admite. Com a Solana como origem, o padrão é 32 confirmações, cerca de 25 segundos. Com a Ethereum como origem, o padrão é aproximadamente 65 confirmações, cerca de 15 a 19 minutos. Então "a rota padrão leva cerca de um quarto de hora" é um fato da Ethereum sendo citado como um fato do CCTP; não existe uma duração única de rota padrão para citar. Leia a tabela por blockchain da Circle para qualquer que seja a origem de onde os seus clientes de fato pagam, e cite essa linha em vez da linha da Solana. O ponto de produto sobrevive de qualquer jeito: compradores cross-chain param de ser um ticket de suporte e viram um pagamento normal que chega um pouco atrasado.

![Um fluxograma contrastando o CCTP, que queima USDC e cunha ele nativamente na Solana, domínio 5, depois de esperar a finality da blockchain de origem, contra pontes lock-and-wrap segurando tokens num pool.](assets/v07-flowchart.webp)

Para a sua integração o desfecho é quase anticlimático, e anticlimático é o objetivo. Você não integra o CCTP neste curso; carteiras e on-ramps dirigem isso. Você só recebe USDC. Tudo o que você construiu na lição passada, e tudo o que você constrói hoje, já lida com a chegada.

## Lab: ensine o kit a ler antes de assinar

O kit ganha três habilidades, nesta ordem: detectar o programa dono de um mint, produzir um relatório completo do mint com dados de extensão ao vivo, e rotear um envio pelo programa correto. Depois um smoke check prova o elenco inteiro. Trabalhe dentro do workspace `transfer-kit` da lição passada: rode a linha `npm install --workspace` e os checkpoints de `tsc` a partir da raiz `wavelength`, e `cd transfer-kit` para todo o resto, porque todo caminho de arquivo abaixo é relativo a esse workspace.

1. Instale o cliente do Token-2022. O workspace está fixado em kit ^6.10.0 (esse pin veio da lição passada: os degraus de checkout rio abaixo importam este kit, e o @solana/pay tem peer em kit ^6.9), então o cliente do Token-2022 tem que ser um minor de kit v6:

```bash
npm install --workspace transfer-kit @solana-program/token-2022@0.12.0
```

   Nota de atualidade sobre esse pin: 0.12.0 é o último minor de @solana-program/token-2022 cuja faixa de peer aceita kit ^6 (ele tem peer em ^6.4.0); a partir do 0.13.0 o pacote tem peer em kit ^7 e o npm vai recusar a instalação neste workspace. Verificado contra o registro do npm em 2026-08-22; reconfira a faixa de peer se você estiver lendo isto muito depois, porque a onda do kit v7 no ecossistema é onde os minors novos aterrissam. Checkpoint: a instalação sai limpa. Um erro de peer `ERESOLVE` quer dizer que o pin derivou para além do kit v6, e nenhum passo posterior vai funcionar até isso estar certo.

2. Crie o `src/detect.ts`. Este é o desafio de completion: a leitura da conta está escrita para você, a classificação não. O arquivo compila do jeito que está, e todo passo rio abaixo vai lançar erro até você terminar ele:

```ts
// src/detect.ts: which token program owns this mint?
import type { Address, Rpc, GetAccountInfoApi } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { TOKEN_2022_PROGRAM_ADDRESS } from "@solana-program/token-2022";

export { TOKEN_PROGRAM_ADDRESS, TOKEN_2022_PROGRAM_ADDRESS };

/**
 * Reads the mint account and returns the token program that owns it.
 * Every transfer this kit builds MUST target this program, never a
 * hardcoded one.
 */
export async function detectTokenProgram(
  rpc: Rpc<GetAccountInfoApi>,
  mint: Address,
): Promise<Address> {
  const { value } = await rpc
    .getAccountInfo(mint, { encoding: "base64" })
    .send();
  if (!value) {
    throw new Error(`Mint ${mint} does not exist on this cluster`);
  }
  const owner = value.owner;

  // COMPLETION CHALLENGE: classify `owner`.
  // Return it when it matches one of the two exported program
  // addresses; throw for anything else, because an account owned by
  // neither token program is not a mint and the kit must refuse to
  // build a transfer against it. Two comparisons and one throw.
  throw new Error(`TODO: classify owner program ${owner}`);
}
```

   Preencha agora, antes de seguir. Os dois endereços contra os quais comparar já estão importados e reexportados no topo do arquivo; a mensagem de recusa deveria nomear o dono inesperado, porque o você-do-futuro depurando um mint esquisito vai querer isso. Não pule o ramo de recusa. Retornar um programa padrão para um dono desconhecido é como um kit assina algo que ele não entende.

3. Crie o `src/read-mint.ts`, o gerador de relatórios. Uma chamada, uma imagem honesta de qualquer mint:

```ts
// src/read-mint.ts: one live report per mint: program, decimals,
// extensions, and the CURRENT transfer fee. Never trust the ticker.
import { address, type Address, type Rpc, type GetAccountInfoApi } from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";
import { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";

// An unset hook program decodes as the all-zero pubkey, which prints
// as the same base58 string as the system program address.
const UNSET = address("11111111111111111111111111111111");

export interface MintReport {
  mint: Address;
  programAddress: Address;
  /** Extension names as found on the mint, empty for classic Token. */
  extensions: string[];
  decimals: number;
  /** Live transfer-fee basis points, null when no transferFeeConfig. */
  transferFeeBps: number | null;
  /** Issuer seizure power: the permanent delegate, when configured. */
  permanentDelegate: Address | null;
  /** Transfer-hook program, when one is actually wired. Can be null
   *  even when the extension is present: configured but dormant. */
  transferHookProgram: Address | null;
}

export async function readMint(
  rpc: Rpc<GetAccountInfoApi>,
  mint: Address,
): Promise<MintReport> {
  const programAddress = await detectTokenProgram(rpc, mint);

  // The token-2022 client's mint codec also decodes classic mints:
  // same base layout, just an empty extension list.
  // Second read of the same account, and yes, it could be one. The
  // codec wants a decoded account and detectTokenProgram wants the raw
  // owner field, so collapsing them means hand-rolling the fetch. For a
  // read that a real checkout caches per mint anyway, clarity wins.
  const account = await fetchMint(rpc, mint);
  const data = account.data;

  const report: MintReport = {
    mint,
    programAddress,
    decimals: data.decimals,
    extensions: [],
    transferFeeBps: null,
    permanentDelegate: null,
    transferHookProgram: null,
  };

  if (programAddress !== TOKEN_2022_PROGRAM_ADDRESS) return report;
  if (data.extensions.__option === "None") return report;

  for (const ext of data.extensions.value) {
    report.extensions.push(ext.__kind);
    if (ext.__kind === "TransferFeeConfig") {
      // Two fee schedules exist. The older stays in force until the
      // epoch stamped on the newer one arrives. We surface the newer:
      // the rate this mint is heading for.
      report.transferFeeBps = ext.newerTransferFee.transferFeeBasisPoints;
    }
    if (ext.__kind === "PermanentDelegate") {
      report.permanentDelegate = ext.delegate;
    }
    if (ext.__kind === "TransferHook") {
      report.transferHookProgram =
        ext.programId === UNSET ? null : ext.programId;
    }
  }
  return report;
}
```

   Repare no que a lógica da taxa não faz: ela nunca assume uma alíquota. Uma mudança de taxa é agendada contra uma epoch, então o mint carrega dois cronogramas, o mais velho e o mais novo, e o mais velho continua valendo até a epoch carimbada no mais novo chegar. A gente expõe o `newerTransferFee` porque ele é a alíquota para a qual o mint está indo, e no PYUSD hoje os dois cronogramas leem zero, então a distinção sai de graça.

   Diga a limitação em voz alta, porque o kit é seu e você deveria saber onde ele é aproximado: **o `readMint` reporta a alíquota agendada, não necessariamente a alíquota em vigor hoje.** Se você encontrar um mint cujo cronograma mais novo tem um carimbo de epoch no futuro, a taxa de fato retida agora é a mais velha, e um checkout cotando a partir do `transferFeeBps` estaria cotando o número de amanhã. Deixar isso exato são duas adições e nenhum conceito novo: guarde o `ext.olderTransferFee` ao lado do mais novo no `MintReport`, peça à blockchain `await rpc.getEpochInfo().send()` e leia o campo `epoch` dele, depois escolha o cronograma mais velho sempre que `epoch < ext.newerTransferFee.epoch`. A gente deixa isso fora do lab porque todo mint que este curso toca cobra zero nos dois cronogramas, então o ramo nunca executaria e você nunca veria ele falhar. Coloque isso antes de aceitar um mint que cobra taxa por dinheiro de verdade.

4. Leia o PYUSD ao vivo. Faça uma pasta `scripts/` ao lado de `src/` com `mkdir -p scripts`, depois adicione um runner minúsculo, `scripts/read.mts`:

```ts
// scripts/read.mts: usage: npx tsx scripts/read.mts <MINT_ADDRESS>
import { createSolanaRpc, address } from "@solana/kit";
import { readMint } from "../src/read-mint.js";

const rpc = createSolanaRpc(
  process.env.RPC_URL ?? "https://api.mainnet-beta.solana.com",
);
const report = await readMint(rpc, address(process.argv[2]));
console.log(
  JSON.stringify(report, (_k, v) => (typeof v === "bigint" ? v.toString() : v), 2),
);
```

   Rode ele contra os dois mints da abertura:

```bash
npx tsx scripts/read.mts EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
npx tsx scripts/read.mts 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo
```

   Checkpoint: o USDC reporta o programa clássico e uma lista de extensões vazia. O PYUSD reporta Token-2022 e oito extensões, `transferFeeBps: 0`, um endereço de verdade em `permanentDelegate`, e `transferHookProgram: null`. Se o switch do seu `detect.ts` estiver errado, é aqui que aparece: o PYUSD voltando "clássico" com zero extensões quer dizer que a sua classificação caiu num padrão. Uma ressalva de nomenclatura para a saída não te assustar: o cliente imprime nomes de codec como `ConfidentialTransferFee`, enquanto a visão jsonParsed do RPC da mesma entrada diz `confidentialTransferFeeConfig`. As mesmas oito entradas TLV, duas grafias; conte entradas, não grafias. (Levei um minuto apertando os olhos na primeira vez.)

Antes do código de envio, segure a rota completa na cabeça. O novo caminho de decisão do kit por pagamento:

![Um fluxograma do caminho de envio: detectar o programa dono do mint, recusar donos desconhecidos e transfer hooks vivos, depois rotear cada transferência para uma cauda compartilhada de assinar e confirmar.](assets/v08-flowchart.webp)

5. Reescreva o `src/send.ts` para a rota acima ser real. Isto substitui a versão cravada da lição passada; o pipe lá embaixo fica intocado, e é esse o ponto:

```ts
// src/send.ts: program-aware sendStablecoin. Four diffs from last
// lesson, all deliberate:
//   1. resolveAta moves here from ata.ts and takes a third seed, the
//      mint's owner program, which is read once and threaded through.
//   2. The idempotent-create helper becomes the ...Async variant, which
//      derives the ATA itself, so there is no explicit `ata:` argument.
//   3. `signature` is returned as a plain string, not kit's branded
//      Signature type, so callers can serialize it without ceremony.
//   4. The RPC clients are constructed inside from URLs. That is a
//      deliberate simplification for a course kit and it costs you the
//      ability to inject a fake RPC in a test; if you later want that
//      back, take rpc/rpcSubscriptions as optional overrides.
import {
  AccountRole,
  appendTransactionMessageInstructions,
  assertIsTransactionWithBlockhashLifetime,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getSignatureFromTransaction,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type Instruction,
  type KeyPairSigner,
} from "@solana/kit";
import {
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getTransferCheckedInstruction as getClassicTransferChecked,
} from "@solana-program/token";
import { getTransferCheckedInstruction as get2022TransferChecked } from "@solana-program/token-2022";
import { getAddMemoInstruction } from "@solana-program/memo";
import { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";
import { readMint } from "./read-mint.js";

/** ATA derivation now takes the owner PROGRAM as a seed: the same
 *  wallet has a different USDC address and PYUSD address partly
 *  because the token program is part of the derivation. */
export async function resolveAta(
  owner: Address,
  mint: Address,
  tokenProgram: Address,
): Promise<Address> {
  const [ata] = await findAssociatedTokenPda({
    owner,
    mint,
    tokenProgram,
  });
  return ata;
}

export interface SendResult {
  signature: string;
  reference: Address;
  tokenProgram: Address;
}

export async function sendStablecoin(opts: {
  rpcUrl: string;
  rpcSubscriptionsUrl: string;
  payer: KeyPairSigner;
  mint: Address;
  recipient: Address;
  /** exact base units from toBaseUnits, never a float */
  amount: bigint;
  memo: string;
  reference: Address;
}): Promise<SendResult> {
  const rpc = createSolanaRpc(opts.rpcUrl);
  const rpcSubscriptions = createSolanaRpcSubscriptions(
    opts.rpcSubscriptionsUrl,
  );

  // 1. The switch this whole lesson exists for. ONE read: readMint
  // already detects the owner program internally and hands it back on
  // the report, so calling detectTokenProgram here too would be a
  // second round trip for an answer we are already holding.
  const report = await readMint(rpc, opts.mint);
  const tokenProgram = report.programAddress;

  // Refuse surprises instead of eating them: a live transfer hook
  // means extra required accounts this kit does not resolve.
  if (report.transferHookProgram !== null) {
    throw new Error(
      `Mint ${opts.mint} has an active transfer hook ` +
        `(${report.transferHookProgram}); this kit does not resolve ` +
        `hook accounts. See the Digital Assets course for the interface.`,
    );
  }

  // 2. Both ATA derivations carry the detected program.
  const sourceAta = await resolveAta(
    opts.payer.address,
    opts.mint,
    tokenProgram,
  );
  const destinationAta = await resolveAta(
    opts.recipient,
    opts.mint,
    tokenProgram,
  );

  // 3. Idempotent ATA creation for first-time holders, same rung as
  // last lesson, now told which program will own the account.
  const createAtaIx = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer: opts.payer,
    owner: opts.recipient,
    mint: opts.mint,
    tokenProgram,
  });

  // 4. Route the transfer to the matching client. Same instruction
  // layout on both programs; different program id on the wire.
  const transferInput = {
    source: sourceAta,
    mint: opts.mint,
    destination: destinationAta,
    authority: opts.payer,
    amount: opts.amount,
    decimals: report.decimals,
  };
  const baseTransferIx =
    tokenProgram === TOKEN_2022_PROGRAM_ADDRESS
      ? get2022TransferChecked(transferInput)
      : getClassicTransferChecked(transferInput);

  // 5. Reference key: the read-only non-signer marker reconciliation
  // will search for, exactly as in last lesson.
  const transferIx: Instruction = {
    ...baseTransferIx,
    accounts: [
      ...baseTransferIx.accounts,
      { address: opts.reference, role: AccountRole.READONLY },
    ],
  };

  const memoIx = getAddMemoInstruction({ memo: opts.memo });

  // 6. The send pipe is untouched from last lesson.
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
  const message = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayerSigner(opts.payer, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) =>
      appendTransactionMessageInstructions(
        [createAtaIx, memoIx, transferIx],
        m,
      ),
  );
  const signed = await signTransactionMessageWithSigners(message);
  assertIsTransactionWithBlockhashLifetime(signed);
  await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(signed, {
    commitment: "confirmed",
    maxRetries: 0n,
  });

  return {
    signature: getSignatureFromTransaction(signed),
    reference: opts.reference,
    tokenProgram,
  };
}
```

   Duas dessas decisões importam para além deste arquivo. Os decimais agora vêm do relatório do mint em vez de uma constante, então um futuro token de 8 decimais não pode ser escalado errado em silêncio por um kit que assumia seis. E o valor de retorno ganhou um campo `tokenProgram`: o back office da sua loja vai logar qual máquina moveu cada pagamento, e quando uma dúvida de suporte chegar meses depois esse único campo logado responde ela antes de você abrir um explorer.

   Duas tarefas que esta reescrita cria, e elas são o preço de mudar uma interface compartilhada. Eu disse na lição passada que as interfaces do kit são estruturais, e elas são, e é precisamente por isso que uma mudança nelas é uma migração caminhada junto e não um exercício deixado para você.

   **Tarefa um: `src/index.ts`.** Ele ainda exporta o `resolveAta` de `./ata.js` e dois nomes de tipo que o novo `send.ts` não define, então o `tsc` falha na linha de export antes mesmo de chegar na sua lógica. Aposente o `src/ata.ts` (o `resolveAta` de dois argumentos dele não consegue derivar um endereço Token-2022) e substitua o arquivo barrel por exatamente isto:

```ts
// src/index.ts
export { toBaseUnits, fromBaseUnits } from "./amounts.js";
export { resolveAta, sendStablecoin } from "./send.js";
export type { SendResult } from "./send.js";
export { detectTokenProgram, TOKEN_2022_PROGRAM_ADDRESS } from "./detect.js";
export { readMint } from "./read-mint.js";
export type { MintReport } from "./read-mint.js";
export * from "./mints.js";
```

   **Tarefa dois: `src/pay.ts`.** O `sendStablecoin` agora recebe URLs de RPC em vez de clientes vivos, unidades base exatas em vez de uma string decimal, e uma `reference` que você gera no ponto de chamada, e ele retorna `tokenProgram` onde a forma antiga retornava `destinationAta` e `baseUnits`. Quem chama absorve tudo isso, e o `receipt.json` mantém a forma exata que o `verify` já lê. Substitua o arquivo:

```ts
// src/pay.ts
import { readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import {
  address,
  createKeyPairSignerFromBytes,
  generateKeyPairSigner,
} from "@solana/kit";
import { sendStablecoin, resolveAta } from "./send.js";
import { toBaseUnits } from "./amounts.js";
import { USDC_DEVNET, USDC_DECIMALS } from "./mints.js";

const recipientArg = process.argv[2];
const amountArg = process.argv[3] ?? "1.25";
const referenceArg = process.argv[4];
if (!recipientArg) {
  console.error(
    "usage: [PAYER_KEYFILE=<path>] npm run --workspace transfer-kit pay -- <recipient-wallet> [amount] [reference]",
  );
  process.exit(1);
}

// PAYER_KEYFILE lets this script send AS somebody else, which is the whole
// point of the pretend customer you made in lesson 1. Unset, it is your
// merchant identity, exactly as before.
const keyfile = process.env.PAYER_KEYFILE ?? `${homedir()}/.config/solana/id.json`;
const bytes = new Uint8Array(JSON.parse(await readFile(keyfile, "utf8")));
const payer = await createKeyPairSignerFromBytes(bytes);

// The caller owns these two now, on purpose: the checkout that
// generates a reference is the thing that must remember it. Pass a
// reference in when you are paying a checkout that already minted one;
// omit it and this script mints a throwaway of its own.
const baseUnits = toBaseUnits(amountArg, USDC_DECIMALS);
const reference = referenceArg
  ? address(referenceArg)
  : (await generateKeyPairSigner()).address;

const result = await sendStablecoin({
  rpcUrl: "https://api.devnet.solana.com",
  rpcSubscriptionsUrl: "wss://api.devnet.solana.com",
  payer,
  recipient: address(recipientArg),
  mint: USDC_DEVNET,
  amount: baseUnits,
  memo: "wavelength-order-0001",
  reference,
});

// Rebuild the two fields the new return shape dropped, so receipt.json
// stays byte-identical to what verify.ts already parses.
const destinationAta = await resolveAta(
  address(recipientArg),
  USDC_DEVNET,
  result.tokenProgram,
);

console.log("signature :", result.signature);
console.log("reference :", result.reference);
console.log("program   :", result.tokenProgram);
console.log("base units:", baseUnits.toString());

await writeFile(
  new URL("../receipt.json", import.meta.url),
  JSON.stringify(
    {
      signature: result.signature,
      reference: result.reference,
      destinationAta,
      baseUnits: baseUnits.toString(),
    },
    null,
    2,
  ),
);
```

   Dois botões entraram que o arquivo antigo não tinha, e o módulo 3 saca os dois. O `PAYER_KEYFILE` escolhe qual keypair assina: deixe ele sem valor e você é o lojista, aponte para `/tmp/customer.json` e este script vira o seu cliente de mentira pagando *você*. Um quarto argumento aceita uma reference que outra pessoa criou, que é como um script faz o papel do cliente para um checkout que já gerou o próprio número de rastreio. Nenhum dos dois botões muda a execução de hoje; os dois existem porque um lojista pagando a si mesmo não é um pagamento, e o checkout do módulo 3 precisa de uma segunda carteira de verdade do outro lado do balcão.

   Checkpoint: `npx tsc --noEmit` a partir da raiz `wavelength` fica em silêncio de novo, e `npm run --workspace transfer-kit verify` continua passando contra o recibo da lição passada.

6. Você precisa de um mint Token-2022 que você consiga de fato gastar na devnet, e o PYUSD não distribui saldos de devnet, então faça o seu próprio mint de teste. A CLI `spl-token` vem no mesmo release do Agave que você instalou para o `solana` na lição passada; confira com `spl-token --version` (se ela estiver faltando de algum jeito, `cargo install spl-token-cli` traz ela de volta):

```bash
spl-token create-token \
  --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb \
  --decimals 6 --url devnet
```

   Copie o endereço de mint impresso, depois crie a sua própria conta de token para ele e cunhe um saldo para você:

```bash
spl-token create-account <YOUR_T22_MINT> --url devnet
spl-token mint <YOUR_T22_MINT> 100 --url devnet
export T22_MINT=<YOUR_T22_MINT>
```

   Checkpoint: `spl-token balance $T22_MINT --url devnet` imprime 100. Rode `npx tsx scripts/read.mts $T22_MINT` com `RPC_URL=https://api.devnet.solana.com` e o seu próprio gerador de relatórios te conta o que você acabou de fazer: Token-2022, seis decimais, nenhuma extensão. Um mint Token-2022 cru é perfeitamente legal; extensões são opt-in na criação, e o curso Digital Assets é onde você aprenderia a optar por elas.

   Agora pise na mina de propósito, porque uma falha que você viu vale por dez sobre as quais te avisaram. Mire o caminho só-clássico da lição passada neste mint Token-2022:

```bash
spl-token transfer $T22_MINT 1 <ANY_WALLET> \
  --program-id TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  --url devnet
```

   Esse `--program-id` é o Token program clássico, que é precisamente a constante que o seu `send.ts` antigo cravava. Resultado esperado: a CLI recusa antes de qualquer coisa ser montada ou submetida — código de saída 1 e um erro de divergência de dono nomeando o seu mint, na forma de `Account <YOUR_T22_MINT> is owned by TokenzQd..., not configured program id Tokenkeg...`. Repare no que você **não** recebeu: uma signature, uma entrada em explorer, ou uma taxa gasta. A CLI lê o dono do mint e confere ele contra o programa pedido antes de montar qualquer coisa — do lado do cliente, exatamente o hábito que o seu kit acabou de aprender. O seu `send.ts` antigo não tinha checagem nenhuma dessas, e é por isso que a versão dele dessa falha aparece uma costura depois, na simulação de preflight do RPC. Mesma recusa, três costuras possíveis — checagem no cliente, preflight, on-chain — e quanto mais cedo você pega, mais barato fica. Esse vão entre "bem-formado" e "vai executar" é a razão inteira de o kit agora ler o mint antes de montar qualquer coisa.

7. Entregue o smoke check. Crie o `scripts/verify-roster.mts`:

```ts
// scripts/verify-roster.mts: the per-lesson smoke check.
// 1. Mainnet read: PYUSD reports eight extensions + live fee bps.
// 2. Devnet sends: one classic-Token mint, one Token-2022 mint,
//    each routed to the correct owner program.
import { readFile } from "node:fs/promises";
import {
  address,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  generateKeyPairSigner,
} from "@solana/kit";
import { readMint } from "../src/read-mint.js";
import { sendStablecoin } from "../src/send.js";
import {
  TOKEN_PROGRAM_ADDRESS,
  TOKEN_2022_PROGRAM_ADDRESS,
} from "../src/detect.js";

const PYUSD = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");
const DEVNET_USDC = address("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
if (!process.env.T22_MINT) {
  throw new Error("set T22_MINT to the devnet Token-2022 mint you made in step 6");
}
const T22_MINT = address(process.env.T22_MINT);

const mainnet = createSolanaRpc("https://api.mainnet-beta.solana.com");
const pyusd = await readMint(mainnet, PYUSD);
// Do NOT gate on the count. Two reasons, both from this lesson: the
// live mint's extension set is PayPal's to change, and the client and
// the RPC's jsonParsed view spell some entries differently, so a count
// is a spelling artifact. Print it, then assert on the properties that
// actually decide whether we can take the payment.
console.log(`PYUSD extensions (${pyusd.extensions.length}):`, pyusd.extensions);
if (pyusd.transferHookProgram !== null) {
  throw new Error(`PYUSD: transfer hook is live; this kit cannot route it`);
}
if (typeof pyusd.transferFeeBps !== "number") {
  throw new Error(`PYUSD: transfer fee did not read as a number`);
}
console.log(`PYUSD: ${pyusd.extensions.length} extensions, transfer fee ${pyusd.transferFeeBps} bps (live)`);

const keyBytes = new Uint8Array(
  JSON.parse(await readFile(`${process.env.HOME}/.config/solana/id.json`, "utf8")),
);
const payer = await createKeyPairSignerFromBytes(keyBytes);
const recipient = (await generateKeyPairSigner()).address;

for (const [label, mint, expected] of [
  ["classic", DEVNET_USDC, TOKEN_PROGRAM_ADDRESS],
  ["token-2022", T22_MINT, TOKEN_2022_PROGRAM_ADDRESS],
] as const) {
  const result = await sendStablecoin({
    rpcUrl: "https://api.devnet.solana.com",
    rpcSubscriptionsUrl: "wss://api.devnet.solana.com",
    payer,
    mint,
    recipient,
    amount: 250_000n, // 0.25 at 6 decimals, exact base units
    memo: `verify:roster ${label}`,
    reference: (await generateKeyPairSigner()).address,
  });
  if (result.tokenProgram !== expected) {
    throw new Error(`${label}: routed to ${result.tokenProgram}`);
  }
  console.log(`${label}: confirmed ${result.signature} via ${result.tokenProgram}`);
}
console.log("verify:roster PASS");
```

   Ligue ele nos scripts do `package.json` do workspace, ao lado do `verify` da lição passada:

```json
{
  "scripts": {
    "pay": "tsx src/pay.ts",
    "verify": "tsx src/verify.ts",
    "verify:roster": "tsx scripts/verify-roster.mts"
  }
}
```

   O envio do ramo clássico gasta o USDC de devnet que você ainda tem da execução de verify da lição passada; se o saldo secou, cunhe mais pelo fluxo de faucet de devnet que você usou lá. Não rode a checagem completa ainda. Rodar ela é a metade de trás do Challenge.

## Challenge

A metade de completion você já conheceu: o switch do `detectTokenProgram` no passo 2. Se você adiou, feche agora, e seja rigoroso consigo mesmo quanto ao terceiro ramo. A recusa para um dono desconhecido é a diferença entre um kit e um perigo.

A metade solo, sem guia: rode o elenco. Enumere as extensões do PYUSD a partir de uma leitura ao vivo do mint e imprima os centésimos de ponto percentual atuais da taxa de transferência, usando o seu próprio `scripts/read.mts`. Depois mande o mesmo valor por dois mints na devnet, o mint de USDC de Token clássico e o seu próprio mint de teste Token-2022, e termine com:

```bash
npm run --workspace transfer-kit verify:roster
```

Aceite quando as três valerem: as duas transferências aterrissam na devnet, o kit reporta o programa dono correto para cada mint, e a sua leitura imprime a lista de extensões ao vivo e a taxa em centésimos de ponto percentual a partir do próprio mint em vez de da memória, e a checagem de hook passa. Se o envio Token-2022 falhar enquanto o clássico passa, a sua derivação de ATA quase certamente está sem a seed do programa: rode o `scripts/read.mts` de novo no seu mint de teste, depois encare o `resolveAta`.

Uma pergunta de reflexão para fechar o ciclo, sem código: a sua loja quer aceitar USDG no trimestre que vem. O que, concretamente, o seu kit já sabe fazer, e qual único fato você ainda verificaria antes de ligar isso? Se a sua resposta inclui ler o mint ao vivo e conferir a taxa e o delegado, a lição pegou. Se a sua resposta for "nada, a tabela disse que está tudo bem", releia a troca.

Esta cobriu muito elenco para uma lição só, e se os poderes de extensão ainda parecerem abstratos, isso é esperado: você leu eles hoje, você não criou eles, e a profundidade de criação mora no curso Digital Assets por projeto. O catálogo completo de extensões, transfer hooks e as transferências confidenciais por trás deste mesmo mint são os módulos dois a quatro daquele curso; se você quer o lado do emissor do que acabou de ler, a porta é essa. Leve a sua saída de `verify:roster` para a comunidade do curso se alguma coisa rotear errado; uma signature falhando com um endereço de mint é um diagnóstico de cinco minutos quando outros construtores conseguem ver.

O seu kit agora move qualquer stablecoin do elenco, clássica ou Token-2022, e recusa as que ele não consegue assinar com segurança. No próximo módulo ele deixa de ser um script e vira uma loja: um núcleo de pagamento só por trás de um checkout com QR, uma barraca de feira e um link de drop compartilhável. As reference keys que você vem anexando obedientemente estão prestes a valer o que custam.
