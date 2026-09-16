# Token-2022 da cadeira do framework

Na lição passada você entregou o swap de token-para-ticket: matemática de produto constante em cima de duas reservas de token SPL, o invariante se mantendo através de uma troca, a trava de slippage disparando no momento em que um fill entrou abaixo da cotação. Funciona. Você comprovou que funciona. Depois um player chega no fliperama com um token contra o qual você não testou.

O mint dele é Token-2022. Ele carrega uma taxa de transferência e um transfer hook. E aqui está a pergunta honesta, a que esta lição inteira gira em volta: a sua chamada de `transfer_checked` ainda funciona, entrega menos silenciosamente, ou falha de cara?

Não responda de memória. Abra um terminal, porque a gente vai apontar um leitor minúsculo para um mint Token-2022 de verdade e deixar ele nos contar. É essa a jogada inteira desta lição: rode, observe, raciocine. Comece agora, antes de continuar lendo, com um comando:

```bash
# PYUSD, a live Token-2022 mint. A classic SPL mint is 82 bytes. Watch this one.
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta
```

Leia a linha `Length:` que ele imprime. Se ela disser qualquer coisa diferente de 82, você já tem a consequência um nas mãos, e o resto desta lição é o porquê. Você vai ler um mint vivo e reportar duas coisas de volta, o número de bytes que ele de fato ocupa e se uma transferência contra ele precisa de contas extra que você não está mandando atualmente. Se você conseguir reportar essas duas coisas a partir da leitura e não de um post de blog, você entendeu a cadeira em que está sentado.

## Resumo

O seu swap já tem os dois programas de token como alvo. Você construiu ele em cima de `InterfaceAccount<T>` e `transfer_checked` lá na lição do swap, e essa não foi uma escolha descartável: o mesmo caminho de código já roda contra o Token clássico do SPL e contra o Token-2022 sem um branch. Então o sistema de tipos está pronto. Essa é a boa notícia, e ela é de graça.

A má notícia é que o Token-2022 caladamente move duas peças de trabalho real para cima do seu programa, e o sistema de tipos não vai te lembrar de nenhuma das duas:

- **Tamanhos de conta param de ser constantes.** Um mint ou uma conta de token Token-2022 não tem tamanho fixo. Extensões fazem eles crescer. Assuma o tamanho clássico e as suas leituras correm para além do fim dos dados.
- **Um mint com hook quer dizer que uma transferência pode precisar de contas extra.** Se o mint declara um transfer hook, uma transferência contra ele roda um segundo programa, e aquele programa precisa das contas dele encaminhadas na sua instrução. Esqueça elas e a transferência não completa.

É essa a lição inteira. Duas consequências, observadas ao vivo, raciocinadas a partir da cadeira do programa. Eu vou percorrer a primeira leitura com você passo a passo no lab. O Challenge no fim você roda sozinho, contra um mint que você escolhe. É esse o recuo: guiado agora, solo em quinze minutos.

Uma fronteira dura, dita de cara para nenhum dos dois derivar. Esta lição ensina Token-2022 como *consequência para o seu programa*. Ela não te ensina a projetar extensões, e ela não percorre a interface de transfer hook. O curso de Digital Assets percorre a interface de transfer hook de ponta a ponta e é dono da profundidade de padrões de extensão — o módulo 3 dele é onde você escreve o hook que esta lição só lê. Quando a gente bater nessa linha, a gente para e aponta para lá. De propósito.

## O que de fato muda para o seu programa

O que não muda é mais do que você imaginaria, então pegue isso primeiro. Um mint Token-2022 usa o *mesmo layout base* que um mint clássico do SPL. Mesmo campo de supply, mesmos decimais, mesmos slots de autoridade, mesmos primeiros 82 bytes. Uma conta de token Token-2022 compartilha a mesma base clássica de 165 bytes também. Se o Token-2022 tivesse reescrito o layout base, cada carteira e cada indexador da rede teria quebrado no primeiro dia. Ele não reescreveu. Ele manteve a base e acrescentou dados novos depois dela.

Esse acréscimo é a coisa para internalizar. Tudo o que o Token-2022 adiciona mora numa seção TLV colada na cauda da conta: tipo, tamanho, valor, repetido. Um mint que opta por uma taxa de transferência, um ponteiro de metadados e um transfer hook carrega três entradas TLV depois da base dele. Um mint que não opta por nada não carrega nenhuma e lê exatamente como um mint clássico.

![Layouts base e a primitiva transfer_checked vêm do SPL clássico; o tamanho total da conta e a necessidade de contas extra de transferência precisam ser observados ao vivo no Token-2022.](assets/v01-comparison.webp)

### Consequência um: tamanho agora é dado, não constante

Um número deixa isso concreto. Um mint clássico do SPL tem 82 bytes. Ponto final, sempre, para sempre. Uma conta de token clássica tem 165 bytes, mesmo acordo. Essas são constantes que você pode deixar hardcoded e nunca pensar nelas de novo.

Agora a leitura ao vivo que você está a ponto de rodar você mesmo: o mint PYUSD da mainnet, um mint Token-2022 de verdade, ocupa 866 bytes agora. Mesma base, mesmos 82 bytes na frente, mais uma cauda TLV carregando as extensões dele. Isso é mais de dez vezes o tamanho clássico, e não é um número mágico que eu quero que você memorize. É um número que você *lê*, porque um mint Token-2022 diferente carrega um conjunto diferente de extensões e aterrissa num tamanho diferente.

![Um mint clássico do SPL tem 82 bytes e uma conta de token clássica 165 bytes, os dois fixos, enquanto o mint PYUSD Token-2022 ao vivo tem 866 bytes por causa da cauda de extensões dele.](assets/v02-chart.webp)

Por que o padrão fez desse jeito, acrescentando uma cauda auto-descritiva em vez de só alargar a struct? Porque você não consegue renumerar um formato binário que o ecossistema inteiro já está parseando. Os campos base ficam em offsets fixos dos quais milhares de clientes dependem. Então features novas não podiam ir *dentro* do layout antigo, elas tinham que ir *depois* dele, cada uma anunciando o próprio tipo e o próprio tamanho para um leitor conseguir caminhar a cauda sem um schema assado de antemão. A elegância é real. O custo é igualmente real e ele cai em cima de você: tamanho agora é um valor carregado nos dados, não uma constante que você pode confiar a partir do header. Leia ele.

É também exatamente por isso que eu te disse para não responder a pergunta de abertura de memória. O tamanho de um mint não é estável nem para um mint só ao longo do tempo: uma autoridade pode adicionar uma extensão depois da emissão e a conta fica mais longa, então um número que estava certo quando você escreveu o seu programa pode estar errado quando ele roda. A disciplina é chata e correta: leia a conta viva, raciocine a partir do que ela diz.

O framework ajuda aqui, e ele ajuda mais do que o caminho clássico ajudava. No seu swap, o mint e os vaults são tipados como `anchor_spl::token_interface::InterfaceAccount<Mint>` e `InterfaceAccount<TokenAccount>`. Esse tipo aceita uma conta de propriedade *de qualquer um* dos dois, o programa Token clássico ou o Token-2022, e ele desserializa os campos base corretamente nos dois. Quando você alcança para além da base, dentro da cauda de extensões, a segunda camada é o `anchor_spl::extensions`, que parseia as structs de extensão TLV de tamanho fixo suportadas a partir da conta. Duas camadas, uma para a base e uma para a cauda.

![Um mint Token-2022 mantém o layout clássico de 82 bytes, preenche até 165 bytes, marca o tipo de conta no byte 165, e depois carrega uma cauda de extensões TLV.](assets/v03-annotated-code.webp)

Um detalhe nesse diagrama surpreende as pessoas, então nomeie ele antes de ele morder: a cauda não começa no byte 82. Um mint estendido é preenchido até 165 bytes, o tamanho da *conta de token* clássica, e o byte 165 carrega uma tag de tipo de conta de um byte, `1` para um mint. Só então o TLV roda, do byte 166 até o fim. O preenchimento existe para um leitor nunca conseguir confundir um mint estendido com uma conta de token pelo tamanho sozinho: um mint simples de 82 bytes nunca foi ambíguo, mas uma base de 82 bytes mais uma cauda TLV poderia aterrissar em exatamente 165 bytes — o tamanho de uma conta de token — e é essa a colisão que o preenchimento mais a tag de tipo descartam. No PYUSD isso deixa 700 bytes de cauda embaixo dos 866.

Então o que de fato quebra se você ignorar isso e assumir os 82 bytes clássicos? De dois jeitos, os dois feios. Se você fatiar a conta num tamanho fixo e ler um campo por offset, você ou aterrissa num byte que quer dizer outra coisa agora ou você corre limpo para além do buffer, e o seu programa está tomando decisões em cima de lixo. Se você desserializar com um decodificador de tamanho fixo, ele ou rejeita a conta ou te entrega uma struct base que caladamente larga tudo o que está na cauda. Nenhuma das duas falhas se anuncia como "você assumiu o tamanho errado". Elas aparecem como valores de lixo e rejeições misteriosas, que é o pior tipo de bug de caçar.

Você consegue ver a constante quebrar em uma linha. Aponte a mesma leitura para um mint clássico do SPL e para o Token-2022, um atrás do outro:

```bash
# USDC, a classic SPL mint. Then PYUSD, Token-2022. Compare the Length: lines.
solana account EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v --url mainnet-beta | grep Length
solana account 2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo --url mainnet-beta | grep Length
```

O primeiro imprime a constante que você poderia ter deixado hardcoded. O segundo imprime um número que você teve que ler. A mesma superfície de instrução, o mesmo `transfer_checked`, dois tamanhos diferentes: essa lacuna é a consequência um, e disciplina de tipagem nenhuma na sua struct de accounts fecha ela para você.

É também por isso que o seu swap tipa as contas dele como `InterfaceAccount<T>` e não como `Account<TokenAccount>`. O `Account<TokenAccount>` prende o owner da conta ao programa Token clássico. Entregue para ele uma conta Token-2022 e ele não lê a cauda errado, ele recusa a conta de cara, porque o owner é um id de programa diferente. O `InterfaceAccount<T>` é a versão que aceita uma conta de propriedade de qualquer um dos dois programas e lê a base compartilhada nos dois. A cilada da qual ele te salva é a mais antiga neste canto da Solana: deixar hardcoded o id do programa Token clássico em algum lugar nas suas contas ou nas suas checagens, então no instante em que um usuário de verdade trouxer um mint Token-2022 o seu programa expulsa ele por uma razão que ele não consegue ver. Tipe como a interface e essa classe inteira de bug nunca chega a ser escrita.

### Consequência dois: um hook pode exigir contas que você não está mandando

A segunda mudança é a que falha alto em vez de caladinho. Um mint pode declarar um *transfer hook*: um programa que o programa Token-2022 chama em cada transferência daquele token, depois de a lógica própria da transferência rodar. Quem cria o token escreve e faz o deploy daquele programa, e depois aponta o mint para ele através da extensão TransferHook.

Para o seu swap, a consequência é estreita e específica. Quando uma transferência roda contra um mint com hook, o programa de hook executa, e ele precisa das contas dele. Essas contas extra não são contas que você conhece em tempo de compilação. Elas são resolvidas a partir de uma lista on-chain que o hook publica para aquele mint. A sua instrução tem que encaminhar elas. Se você monta um `transfer_checked` com só as quatro contas que ele sempre pegou, from, to, mint, authority, e o mint tem um hook vivo, a transferência não vai completar. O `transfer_checked` continua sendo a primitiva certa; o que está curto é a lista de contas.

Então, da cadeira do programa, existe exatamente uma pergunta que você tem que conseguir responder antes de uma transferência: este mint declara um hook, e se declara, aquele hook está vivo? E o campo no qual você chaveia isso é o `programId` da extensão TransferHook. Se o mint não carrega extensão TransferHook nenhuma, não tem nada para encaminhar. Se ele carrega a extensão mas o `programId` está sem valor, o hook está *declarado mas dormente*, ainda nada para encaminhar. Só quando o `programId` é um endereço de verdade é que uma transferência precisa das contas extra do hook resolvidas e acrescentadas.

Sem valor é uma coisa específica aqui, não um gesto vago. No wire aquele campo é uma pubkey opcional-não-zero: sempre trinta e dois bytes, todos zeros querendo dizer "nenhum". Então "sem valor" lê de volta como o endereço default todo-zero, `11111111111111111111111111111111`, que importa no momento em que você escreve a checagem, porque ele não é `null` e não é vazio.

Esse caso dormente não é um canto que eu inventei para ser minucioso. O mint PYUSD vivo carrega uma extensão TransferHook agora, e o `programId` dele é o default todo-zero. Oito extensões presentes, hook entre elas, e ainda assim um `transfer_checked` simples contra ele não precisa de contas extra, porque o hook está armado e parado em vez de ativo. É por isso que "ele tem a extensão" é a pergunta errada e "o `programId` está com valor" é a certa. Você vai ver exatamente isso num minuto quando rodar o leitor.

![Um branch de sim/não: um mint sem hook, ou com um programId de hook deixado no endereço default todo-zero, transfere normalmente; um programId de verdade exige resolver contas extra primeiro.](assets/v04-flowchart.webp)

Duas camadas, de novo, e vale nomear elas juntas porque elas são a forma da história inteira do Token-2022 no Anchor.

![O Anchor te dá compatibilidade com os dois programas de graça através do InterfaceAccount e do transfer_checked, mas ler a cauda de extensões para tamanho e estado de transfer hook é trabalho que o seu programa tem que fazer.](assets/v05-diagram.webp)

Esse rótulo de emenda é o trade-off, dito sem enfeite. O `token_interface` te compra compatibilidade com os dois programas de graça, no nível do tipo, e é genuinamente um alívio comparado a deixar um id de programa hardcoded e ramificar. Mas o Token-2022 move trabalho real para cima de você em troca: você não pode assumir um tamanho fixo, e um mint com hook quer dizer que uma transferência que *parece* completa pode falhar a não ser que você encaminhe as contas do hook. O framework te entrega a cadeira. Ele não te entrega o padrão.

Mais uma armadilha de leia-não-assuma pertence logo ao lado do hook, porque o mint do player na abertura carregava as duas. Um mint também pode declarar uma taxa de transferência, e quando ele declara, a quantia que de fato aterrissa no destino é menor que a quantia que você entregou para o `transfer_checked`. Para um envio simples de carteira para carteira isso é um incômodo de arredondamento. Para o seu swap é um bug de corretude: a sua matemática de produto constante assume que o vault recebeu exatamente o que você mandou para ele, e uma taxa caladamente anula essa suposição, então o seu invariante deriva e a sua precificação sai errada. A mecânica é idêntica a tudo o mais aqui, leia o mint, não assuma a quantia. A matemática própria da taxa, como os basis points e o teto máximo de fato computam, é o catálogo de extensões, e aquele catálogo é do curso de Digital Assets. Notar que o líquido-recebido pode diferir da quantia-mandada é a parte que é sua.

![Um mint Token-2022 que carrega taxa entrega menos que a quantia mandada, então o invariante do swap é computado em cima da reserva errada e o saldo guardado do vault superestima a custódia real.](assets/v06-diagram.webp)

E essa é a linha que a gente não cruza. Projetar uma extensão, escrever um programa de transfer hook, ligar a interface de resolução de contas dele de ponta a ponta, isso é profundidade de padrões, e isso mora num lugar só por design. O curso de Digital Assets percorre a interface de transfer hook de ponta a ponta e ensina profundidade de padrões de extensão. Aqui, da cadeira do framework, o seu trabalho para em notar que o hook existe e em saber que você teria que encaminhar as contas dele. Notar é mecânica. Autoria é o padrão. Curso diferente, de propósito.

![Esta lição ensina ler tamanho ciente de extensões, detectar um transfer hook e raciocinar sobre contas extra; projetar extensões e autorar a interface de transfer hook pertencem ao curso de Digital Assets.](assets/v07-comparison.webp)

Você pode ficar tentado a arquivar tudo isso como "caso extremo que eu vou tratar quando alguém reclamar". Resista a isso. Os mints Token-2022 soltos por aí são desproporcionalmente os que você menos quer que falhem. As stablecoins reguladas e os ativos de valor mais alto recorrem a delegados permanentes, taxas de transferência e hooks precisamente porque dinheiro real e compliance real estão em cima deles. A memecoin descartável nunca vai exercitar este caminho. O mint com o qual a sua tesouraria de fato se importa vai. Essa assimetria é o argumento inteiro: um hábito de leia-não-assuma é seguro barato, e um tamanho hardcoded é uma bomba-relógio com o nome da sua maior contraparte escrito nela.

## Lab: leia o mint antes de confiar nele

Hora de deixar as duas consequências visíveis. A gente vai escrever um leitor pequeno, apontar ele para o mint Token-2022 fornecido, e reportar o tamanho observado e o estado do hook. Este é o compasso de rode-observe-raciocine, e ele é deliberadamente um script de cliente: a pergunta é sobre o que o mint *é*, e o jeito mais limpo de responder ela é ler a conta viva.

Eu estou percorrendo cada passo aqui. Copie junto.

**1. Instale as dependências de cliente.** Nenhum Rust e nenhum Anchor neste lab: a pergunta é sobre o que um mint *é*, então o leitor é um script Node e o seu toolchain V2 fica de fora desta. A gente usa o `@solana/kit` e o cliente Token-2022 nativo de kit. Olhe o pin, e olhe o *par*: em 2026-09-07 o `latest` do kit no npm é 8.2.0 enquanto o `latest` do `@solana-program/token-2022` é 0.16.1, que faz peer com kit `^8`. A instalação abaixo deliberadamente segura o par mais antigo — o `token-2022@^0.15` faz peer com kit `^7` — porque um par casado é o que tem que resolver, não o mais novo de cada um. Rode a checagem de freshness abaixo antes de você mexer nesses, e mova os dois juntos ou nenhum.

```bash
npm install @solana/kit@^7 @solana-program/token-2022@^0.15
npm install -D tsx    # runs a TypeScript file directly
# Freshness check before you ever bump these:
#   npm view @solana-program/token-2022 peerDependencies
```

Fixe aquele range de cliente explicitamente. Deixe o `@solana-program/token-2022` sem versão e o resolvedor do npm consegue retroceder para um `0.11.x` mais antigo, que faz peer com kit `^6`, e a instalação morre num conflito de peer `ERESOLVE` contra o `^7` que você acabou de pedir. Nomear os dois majors é o que faz o par resolver.

Checkpoint: a instalação termina sem nenhum `ERESOLVE` e o `npm ls @solana/kit` imprime um único `7.x` no nível de topo. Duas versões de kit naquela árvore querem dizer que alguma coisa trouxe o `^6` de volta, e o leitor vai falhar num descasamento de tipo e não no mint.

**2. Pegue o leitor.** Este script é dado para você rodar, não TypeScript que você está sendo pedido para autorar. Você escreve Rust neste curso; a única lição em que você autora um cliente em TS é a m08-l1, e esta não é ela. Copie ele como ele está, e leia ele do jeito que você leria o script de um colega: duas leituras, casando com as duas consequências. Primeiro a conta crua, para medir o tamanho real dela. Depois o mint decodificado, para inspecionar a extensão de transfer hook. Salve isto como `inspect.ts`:

```typescript
import {
  address,
  createSolanaRpc,
  fetchEncodedAccount,
  unwrapOption,
} from "@solana/kit";
import { fetchMint } from "@solana-program/token-2022";

const RPC_URL = "https://api.mainnet-beta.solana.com";
const rpc = createSolanaRpc(RPC_URL);

// The provided specimen: a live Token-2022 mint on mainnet.
const MINT = address("2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo");

// On the wire the hook's program is an "optional non-zero" pubkey: 32 bytes,
// all-zero meaning unset. The client decodes those zero bytes to the default
// address below, NOT to null, so this is what "no hook" actually looks like.
const UNSET_HOOK = address("11111111111111111111111111111111");

async function inspect(): Promise<void> {
  // Consequence one: read the real length. Never assume the classic 82 bytes.
  const raw = await fetchEncodedAccount(rpc, MINT);
  if (!raw.exists) {
    throw new Error(`mint ${MINT} not found`);
  }
  console.log("owner program:", raw.programAddress);
  console.log("account bytes:", raw.data.length);

  // Consequence two: does this mint declare a transfer hook, and is it live?
  const mint = await fetchMint(rpc, MINT);
  const extensions = unwrapOption(mint.data.extensions) ?? [];
  console.log("extensions present:", extensions.length);

  const hook = extensions.find((ext) => ext.__kind === "TransferHook");

  if (hook === undefined) {
    console.log("transfer hook: none -> transfer needs no extra accounts");
    return;
  }

  if (hook.programId === UNSET_HOOK) {
    console.log("transfer hook: present but programId unset -> dormant, no extra accounts");
  } else {
    console.log(`transfer hook: ACTIVE (${hook.programId}) -> forward its extra accounts`);
  }
}

inspect().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

**3. Rode ele.**

```bash
npx tsx inspect.ts
```

**4. Leia a saída.** Você deve ver algo que se alinha com isto, com alguma variação nos endereços exatos:

```text
owner program: TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
account bytes: 866
extensions present: 8
transfer hook: present but programId unset -> dormant, no extra accounts
```

Pare e leia isso, porque é a lição inteira aterrissando de uma vez. O `owner program` é o programa Token-2022, que é por que os campos base decodificaram e a cauda existe. O `account bytes` é 866, não 82, que é a consequência um em uma linha: esta conta tem mais de dez vezes o tamanho clássico, e o seu programa corromperia cada leitura se assumisse a constante. Oito extensões voltaram, que é a cauda de que aqueles bytes extra são feitos. E a linha do hook é a consequência dois com a nuance assada dentro: a extensão está *presente*, mas o `programId` dela está sem valor, então um `transfer_checked` contra este mint não precisa de contas extra. A extensão estar ali não respondeu a pergunta. O `programId` respondeu.

Essa comparação contra o `UNSET_HOOK` vale mais um compasso, porque é o lugar exato onde um leitor descuidado entrega um bug. "Sem valor" no wire é trinta e dois bytes zero, e o cliente entrega eles de volta para você como o endereço default no qual aqueles zeros codificam, `11111111111111111111111111111111`, não como `null` e não como `undefined`. Então um teste de veracidade em `hook.programId` é sempre verdadeiro e reportaria cada hook dormente como vivo:

```typescript
// WRONG: a 32-zero-byte pubkey decodes to a non-empty string, so this is
// always truthy and reports every dormant hook as active.
if (hook.programId) { /* resolve extra accounts */ }

// RIGHT: compare against the address those zero bytes actually encode to.
if (hook.programId !== UNSET_HOOK) { /* resolve extra accounts */ }
```

Troque a linha errada para dentro do `inspect.ts` e re-rode ele se você quiser ver o modo de falha: o mesmo hook dormente do PYUSD volta `ACTIVE`, e um programa confiando nisso começaria a encaminhar contas que ninguém pediu. Compare contra o endereço default explicitamente, do jeito que o script faz, ou você escreveu uma checagem que só consegue responder sim.

Se o seu leitor imprimiu um endereço de verdade naquela última linha em vez da mensagem de dormente, você estaria olhando um mint cujas transferências você não consegue completar com quatro contas, e o campo que te disse é o mesmo, o `programId`. É essa a decisão inteira, e você acabou de fazer o seu programa declarar isso em voz alta a partir de uma leitura viva em vez de um chute.

Duas notas antes de você sair caçando por conta própria. Primeira, tudo o que você acabou de fazer com um mint se aplica a contas de token também. Uma conta de token Token-2022 é a base clássica de 165 bytes mais a cauda de extensões própria dela, então a regra de leia-o-tamanho-real vale toda vez que o seu programa toca um saldo, não só quando ele inspeciona um mint. Aponte a mesma chamada de `fetchEncodedAccount` para uma conta de token e você vai ver o mesmo tamanho variável. Segunda, mantenha claro por que isto foi um script de cliente e não uma leitura on-chain. On-chain, o `InterfaceAccount<T>` e o `anchor_spl::extensions` fazem este trabalho exato dentro do seu handler. Off-chain, o kit faz. As mesmas duas perguntas, as mesmas duas respostas, uma cadeira diferente. O ponto nunca foi a linguagem. Foi o hábito de perguntar para a conta em vez de para a sua memória.

## Challenge: ache um mint que diz sim

O lab te entregou um mint cuja resposta é "nenhuma conta extra". Agora você faz um que diz "sim". Caçar na mainnet por um hook vivo é um exercício de agulha no palheiro sem jeito de distinguir falha de má sorte, então você vai cunhar o espécime você mesmo na devnet, onde você controla cada entrada.

O `programId` na extensão TransferHook é só uma pubkey guardada; nada valida que ela aponta para um programa de hook de verdade na hora de cunhar. Então qualquer id de programa que seja seu faz o campo ler como com valor, que é exatamente o estado que você está tentando observar — ele não precisa nem estar deployado. O id do seu swap a partir do `Anchor.toml` funciona mesmo que aquele programa só tenha rodado dentro do LiteSVM, e o greeter R0 que você de fato entregou no m01-l2 funciona igualmente bem:

```bash
solana config set --url devnet
solana airdrop 2

# Any program id works as the hook target; use your own swap's, from Anchor.toml.
spl-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb \
  create-token --transfer-hook <YOUR_SWAP_PROGRAM_ID> --decimals 6
```

Isso imprime um endereço de mint novo. Coloque ele no `inspect.ts` como `MINT`, troque o `RPC_URL` para `https://api.devnet.solana.com`, e rode. Se o `spl-token` não está na sua máquina, o `cargo install spl-token-cli` coloca ele lá.

Aceitação: a última linha lê `ACTIVE` e imprime o id de programa que você passou. Se ela lê `none` em vez disso, você criou o mint sem o `--transfer-hook`. Se o leitor der erro antes de imprimir qualquer coisa — uma falha de owner ou de decodificação — você criou um mint clássico; cheque a flag `--program-id`, que é o que seleciona o Token-2022. (`dormant` você não deveria ver aqui: o `create-token --transfer-hook` escreve um id de programa de verdade no campo.)

Quando a última linha ler `ACTIVE`, sente com o que isso quer dizer para o swap que você construiu. A sua instrução atual manda quatro contas. As transferências deste mint precisam de mais, resolvidas a partir da lista on-chain do hook, e o seu swap do jeito que está escrito falharia contra ele. Você não tem que corrigir isso hoje. Construir de fato a resolução pertence à profundidade de padrões que a gente está deliberadamente deixando para o curso de Digital Assets. Notar que você teria que fazer isso é a habilidade para a qual esta lição existiu.

## O que você deveria conseguir dizer agora

Aqui está o checkpoint, e ele é a forma exata da resposta que esta lição foi construída para produzir. Aponte o seu leitor para o mint fornecido e, a partir da leitura e não da memória, reporte de volta:

- o tamanho de conta ciente de extensões que você observou, em bytes, e
- sim ou não sobre se uma transferência precisa de contas extra de hook, nomeando o único campo no qual você chaveou isso.

Para o mint do lab isso é: 866 bytes, nenhuma conta extra necessária, chaveado no `programId` da extensão TransferHook ainda parado no endereço default todo-zero. Se você conseguir produzir essa forma para o mint que você fez no Challenge também, você terminou.

O que finalmente responde o player na porta do fliperama. O mint dele carregava uma taxa e um hook, então todos os três resultados estavam vivos e você agora consegue dizer qual é qual. Se o `programId` do hook está com valor, o seu `transfer_checked` **falha de cara**, e ele falha de forma limpa em vez de mover metade de qualquer coisa, porque a CPI do hook aborta a transferência. Se o hook está dormente mas um `TransferFeeConfig` está presente, ele **entrega menos silenciosamente**: a chamada tem sucesso, chega menos do que você mandou, e o seu invariante é a coisa que nota, tarde. E se nenhum dos dois está armado, ele simplesmente **funciona**, que é o caso PYUSD que você acabou de ler. Três respostas, uma leitura, e a leitura é a habilidade inteira. Você consegue mover um token através dos dois programas e você consegue olhar um mint que nunca viu e dizer, da cadeira do seu próprio programa, exatamente o que uma transferência contra ele exigiria.

Na próxima lição a gente para de ler as contas dos outros e começa a fazer raio-X nas nossas. Você liga os instrumentos first-party do V2 neste swap exato, flamegraphs, um debugger de step e cobertura, para ver para onde o compute dele de fato vai. Você mediu quanto um mint te custa em bytes. Depois você mede quanto o seu programa custa em compute.
