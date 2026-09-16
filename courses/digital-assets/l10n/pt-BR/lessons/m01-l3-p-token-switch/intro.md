# A troca do p-token: interface congelada, motor novo

Na lição passada você decodificou um mint que nunca criou e dominou a leitura. Lá em m01-l1 você viu uma transferência clássica custar 76 compute units. Eis a parte inquietante, e ela é a lição inteira: essa mesma transferência já custou 4,645 CU. O layout de 82 bytes que você decodificou não mudou. Seu código de cliente não mudou. O código de cliente de ninguém mudou. E mesmo assim o programa rodando em `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` foi trocado sem alarde debaixo dos pés de todo mundo.

Antes de explicar qualquer coisa, vá olhar a chave em si: uma conta real de mainnet, legível agora mesmo. Esta é a primeira lição que recorre à CLI `solana` — e deixa eu ser preciso sobre a palavra "precisa": toda sondagem abaixo é uma leitura RPC simples, e a sondagem 3 do lab faz exatamente a mesma leitura do gate com o código kit que você já tem, então nada nesta lição fica restrito a quem tem uma instalação local. As visões formatadas da CLI são o jeito conveniente de rodar as outras sondagens, mas os labs mais adiante neste curso se apoiam de verdade na toolchain local, então se você quiser instalá-la agora, uma linha entrega a toolchain Agave inteira (eu estou no solana-cli 3.1.10, linha Agave, verificado em 2026-08-22):

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Depois pergunte à mainnet sobre um endereço bem específico:

```bash
solana feature status ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
```

Você deve ver:

```text
Feature                                      | Status                  | Activation Slot | Description
ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP  | active since epoch 971  | 419472000       | SIMD-0266: Efficient Token program
```

Essa linha é a troca de motor. Um feature gate, ativo desde o primeiro slot da epoch 971, e o programa de token que sustenta todo saldo SPL na Solana virou um software diferente. Eu sondei aquele gate de novo hoje de manhã, 2026-08-22, antes de escrever uma palavra desta lição; a saída acima é o que a mainnet retornou. Deixe aquele terminal aberto. Aquela linha formatada é a renderização que a CLI faz de uma conta minúscula de 9 bytes, e no lab você vai despejar esses 9 bytes crus e lê-los você mesmo.

## Resumo

Em m01-l2 você construiu o inspetor `decode-mint` e leu os campos base de um mint simples clássico de 82 bytes direto dos bytes. Esta lição explica algo estranho que você já viu duas vezes sem saber: o programa que processou sua transferência de 76 CU não é o programa que processou as transferências de todo mundo nos seis anos anteriores. O SIMD-0266 substituiu a implementação do SPL Token clássico no mesmo endereço pelo p-token, a reescrita em Pinocchio da Anza, e fez isso sem mudar um único byte da interface. Você acabou de sondar o feature gate que virou a chave; no lab você vai ler a troca na própria conta do programa, e sair com o modelo mental que 2026 impõe ao "SPL clássico": uma interface congelada rodando um motor novinho em folha. Nenhum degrau novo na escada de artefatos hoje, e nenhum completion TODO tampouco: o lab é uma sequência de sondagens guiadas que você roda como mostrado, e o challenge é totalmente solo, em palavras e não em código. Esse é o recuo para uma lição de conceito.

## Interface versus implementação

Comece pelo modelo mental que você provavelmente trouxe para este curso, porque é o que quase todo tutorial ensina. Um programa mora em um endereço. O endereço identifica o código. "Eu conheço o programa SPL Token" quer dizer "eu sei o que o código em TokenkegQ... faz." Sob esse modelo, uma queda de custo de 61x com zero mudanças de cliente deveria ser impossível. Então alguma coisa no modelo está errada, e descobrir o quê vale mais do que o número em si.

Leve as explicações ingênuas até o fim primeiro, porque cada fracasso afia a pergunta.

**Resposta ingênua um: eles entregaram um programa novo e todo mundo migrou.** Não entregaram. Não existe endereço novo. O mint clássico de USDC que seu inspetor `decode-mint` leu na lição passada continua sendo de propriedade do mesmo endereço TokenkegQ... que carteiras, DEXes e pontes têm hardcoded desde 2020, e nenhuma delas entregou uma migração. Se uma migração desse tamanho tivesse acontecido, você teria sentido: upgrades coordenados em todo cliente da chain, janelas de depreciação, integrações quebradas. O silêncio do ecossistema é evidência.

**Resposta ingênua dois: o runtime ficou mais rápido, então tudo ficou mais barato.** Também não, e seus próprios números refutam isso. Compute units não são tempo de relógio; são uma contagem medida do trabalho que o runtime cobra por operação. Um validador mais rápido executa a mesma transferência de 4,645 CU em menos tempo, mas ainda cobra 4,645 CU. Para o número medido cair, o trabalho em si tem que encolher. Alguma coisa mudou dentro do que executa.

**Resposta ingênua três: o programa foi atualizado no lugar, do jeito normal.** Mais quente, mas ainda errado de um jeito instrutivo. Programas atualizáveis comuns recebem código novo pela autoridade de upgrade deles. O programa de token clássico é o programa mais estrutural que existe na Solana; entregar a um único detentor de chave o poder de trocá-lo a quente seria uma história de segurança, não de eficiência. O que quer que o tenha substituído precisava de algo mais forte que uma chave de upgrade: consenso.

![Três explicações fracassadas para a queda de CU, cada uma riscada ao lado da evidência que a refuta, afunilando até a única pergunta que sobrevive à eliminação.](assets/v01-comparison.webp)

Então a pergunta de verdade é mais estreita que "por que está barato agora." Ela é: **que mecanismo consegue substituir o código por trás de um endereço, com a concordância da rede inteira, sem quebrar um único chamador?** Essa pergunta tem exatamente uma resposta na Solana, e você acabou de sondá-la.

![Clientes chamam o mesmo endereço TokenkegQ e a mesma interface congelada, mas por trás dele o motor spl-token original foi substituído pelo p-token da Anza na epoch 971 via um feature gate.](assets/v02-diagram.webp)

### O mecanismo: um feature gate sobre uma interface congelada

Um **feature gate** é uma chave on-chain. É uma conta minúscula, de propriedade do programa Feature, cuja existência e slot de ativação dizem a todo validador "deste slot em diante, se comporte do jeito novo." Validadores carregam os dois comportamentos nos binários deles; o gate decide qual está valendo, e porque o gate é estado on-chain, todo validador vira no mesmo slot. É assim que a Solana entrega mudanças críticas para o consenso sem um hard fork, e é o mecanismo que trocou o seu motor de token.

Para sentir por que esse design é notável, coloque-o contra as alternativas com que outros ecossistemas de fato convivem. A opção um é a migração: fazer deploy do programa novo em um endereço novo e pedir para toda carteira, DEX, ponte e indexador do planeta se mudar, cada um no seu cronograma, com uma janela de depreciação e uma longa cauda de retardatários. Essa é a resposta ingênua um, feita de propósito, e ecossistemas que seguem esse caminho passam anos na transição. A opção dois é o hard fork coordenado: todo operador atualiza até uma data de virada ou cai fora da rede. O feature gate é um terceiro caminho que fica com as partes boas dos dois: o código novo já vem dentro do release normal do validador, dormente, às vezes por meses, e o gate on-chain escolhe o momento. Uma fronteira de slot, nenhuma janela de motor misturado, nenhuma migração, e o endereço que todo cliente deixou hardcoded fica exatamente onde estava.

A proposta por trás da troca é o **SIMD-0266**. Ele foi mergeado em 2026-03-13. Diga mergeado, e não "aceito" ou "aprovado," e aqui vai um hábito de precisão que vale construir: mergeado é um fato do Git que você pode verificar (o PR entrou, com data), enquanto aceito e aprovado são alegações de governança, e o status no próprio front-matter do documento ainda diz "Review" hoje, então o rastro de papel não sustenta nenhum dos dois. O header é um rótulo atrasado. O que de fato governa a ativação é o gate on-chain que você acabou de sondar, e esse gate está ativo. Um colega de time que lê "Review" e conclui que o p-token ainda não está no ar confiou no header de um documento acima do estado da chain, o que na Solana é sempre a ordem errada.

![Linha do tempo desde o merge do SIMD-0266 em 2026-03-13, passando pela ativação do gate no slot 419,472,000 (primeiro slot da epoch 971), até a re-sondagem ao vivo do gate em 2026-08-22.](assets/v03-timeline.webp)

O que o gate ativou é o **p-token**: uma reescrita do zero do programa SPL Token clássico feita pela Anza, escrita em Pinocchio, um framework de programas zero-dependência e zero-copy construído exatamente para esse tipo de trabalho de caminho quente. O contrato da reescrita com o ecossistema era brutal e simples: idêntico byte a byte em layouts de conta, discriminadores de instrução e códigos de erro. Instrução por instrução e erro por erro. Todo offset por onde seu inspetor `decode-mint` caminha, todo discriminador que uma carteira manda, todo código de erro em que uma integração casa: idêntico. Essa superfície idêntica é a **interface**. O código que a honra é a **implementação**. O SIMD-0266 substituiu a implementação e congelou a interface, e essa separação é o truque inteiro.

Um leitor atento deveria objetar exatamente aqui, então vamos pegar as duas objeções mais fortes em ordem. Primeira: "idêntico byte a byte" é uma alegação, não uma propriedade que vem de graça, e o custo de estar errado não é um ticket de bug. Se o p-token discordasse do motor antigo em uma única entrada que fosse, validadores rodando um comportamento calculariam um estado diferente dos validadores rodando o outro, no programa mais chamado da chain inteira. Isso é território de risco de consenso. A interface congelada é o que torna a alegação verificável: o comportamento observável do motor antigo É a especificação, executável e exaustiva, então para qualquer instrução você pode alimentar os dois motores com os mesmos bytes e exigir as mesmas saídas, as mesmas transições de estado e os mesmos códigos de erro. Mesma entrada, mesma saída, ou a reescrita está errada. E o gate é o que torna a alegação segura para agir em cima: em vez de um deploy gradual em que código antigo e novo se sobrepõem por horas, existe um slot inequívoco antes do qual todo mundo roda o motor antigo e depois do qual todo mundo roda o novo.

Segunda objeção, e ela merece uma resposta direta: mecanicamente, nada impede o consenso de colocar algo malicioso por trás desse mesmo endereço. Não passe correndo por isso. Um feature gate ativa porque o conjunto de validadores, ponderado por stake, adota releases que carregam a mudança e o processo de ativação roda; não existe garantia criptográfica de que o que ativa é benigno. Mas repare que esse sempre foi o modelo de confiança. O consenso define o que a chain é desde o bloco gênese; uma supermaioria do stake sempre pôde mudar qualquer regra. O gate não criou esse poder. Ele o tornou visível, carimbado com slot e legível em uma conta de 9 bytes, o que é estritamente melhor que invisível. Seu trabalho como builder não é fingir que o poder não existe; é saber qual camada do sistema o segura.

É aqui que o modelo mental antigo é corrigido em vez de descartado. O endereço nunca identificou o código. Ele identificou o *contrato*: os layouts de bytes e comportamentos com que qualquer um que chama aquele endereço pode contar. O código é só o inquilino atual honrando esse contrato. Existe um velho experimento mental sobre um navio cujas tábuas são trocadas uma a uma até não restar nada da madeira original, e filósofos discutem se ele ainda é o mesmo navio. A resposta da Solana é sem sentimentalismo: se cada tábua da interface é idêntica byte a byte, é o mesmo programa, seja quem for que escreveu a madeira. A analogia quebra em um ponto que vale sinalizar, porém: as tábuas de Teseu foram trocadas gradualmente e por acidente de manutenção. Essa troca aconteceu na rede inteira em um único slot, por design, com o substituto testado contra o comportamento exato do original antes de o gate sequer virar. Identidade deliberada, não identidade à deriva.

![Uma divisão em duas colunas mostrando a interface congelada (endereço, layouts, discriminadores, erros, comportamento) contra a implementação substituída (código do motor, custos de CU, binário), mais três instruções adicionadas.](assets/v04-comparison.webp)

### O retorno, medido

Agora os números podem significar alguma coisa. Um Transfer clássico custava 4,645 CU sob o motor antigo. Sob o p-token ele custa 76 CU. TransferChecked, a variante que você vai usar em todo lugar porque ela valida o mint e os decimals, caiu de 6,200 para 105 CU. Isso não são otimizações incrementais; é o que acontece quando um programa Rust de propósito geral da era 2020 é reescrito por gente que conta cada syscall. E porque transferências de token são a classe de instrução mais comum que existe na chain, o efeito agregado é de escala macro: cerca de 12 a 13 por cento do espaço total de bloco foi recuperado, segundo a Anza (o engenheiro deles, Febo, percorreu os números em uma entrevista publicada em maio de 2026). Doze por cento da capacidade de uma blockchain, devolvidos à rede por uma reescrita em que ninguém precisou optar por entrar.

Pare um instante no que essa recuperação de fato é, porque o enquadramento importa. Blocos não ficaram maiores, e nenhum parâmetro de consenso se moveu. O mesmo orçamento de CU por bloco simplesmente parou de se gastar em overhead de transferência de token: trabalho que cobrava 4,645 unidades por transferência agora cobra 76, e a diferença é capacidade que toda outra transação da chain passa a usar. É o tipo raro de ganho de escala que não custa nada ao resto do sistema. A porcentagem em si é um número de uma era medida, porém, não uma constante: ela reflete quanto do tráfego da chain era transferência de token quando a Anza mediu, então cite-a como o número deles, com a data, do jeito que eu acabei de fazer.

![Gráfico de barras mostrando Transfer caindo de 4,645 para 76 CU e TransferChecked de 6,200 para 105 CU depois da troca do p-token, recuperando cerca de 12 a 13 por cento do espaço de bloco.](assets/v05-chart.webp)

Cuidado com a atribuição, porque essa é a cilada que vai te fazer passar vergonha num code review. A queda é obra do motor, não sua. Se você fez benchmark de uma transferência no ano passado em 4,645 CU e faz benchmark do mesmo código de cliente hoje em 76, seu código não melhorou. Nada de que você faz deploy, nenhuma flag que você seta, nenhum upgrade de SDK que você entrega reivindica qualquer crédito por esses números. O motor mudou por baixo de você. O que corta para o outro lado também, e esta é a ressalva honesta: o 76 é uma medição dependente do motor, não uma constante da natureza. Congele "uma transferência custa 76 CU" em um config ou num doc e você vai citar errado na próxima vez que o motor ou o modelo de custo do runtime se mover. Cite como "76 CU a partir do motor p-token, epoch 971," e meça de novo quando importar. O número que o motor antigo ensinou todo mundo a decorar acabou de virar exemplo do que não fazer; não crie o próximo.

Uma fronteira a respeitar, e ela é deliberada. Esta lição ensina a queda de CU como um fato da camada de token: o que mudou, quando, e o que isso significa para o seu modelo mental. Ela não deriva *por que* 76 CU é fisicamente alcançável, porque esse porquê mora no loader, na máquina virtual sBPF e na maquinaria de medição de compute, e essa camada inteira pertence ao curso planejado Low-Level Solana. Se você se pegar querendo saber para onde vai cada uma das 76 unidades, aquele curso é o lugar; aqui, o número é evidência para o modelo interface-versus-implementação, e esse modelo é o payload.

### Congelado quer dizer congelado: onde moram as capacidades novas

A palavra que a Anza usa para o SPL clássico agora é **feature-complete**, e a tradução prática é: congelado. Nenhuma funcionalidade nova de token está planejada para o programa clássico, nunca. A reescrita p-token de fato adicionou três instruções novas, `batch`, `withdraw_excess_lamports` e `unwrap_lamports`, o que soa como contradição até você reparar que tipo de instruções elas são: conveniências operacionais que se encaixam na superfície existente sem perturbar um único byte existente. Fazer em lote o que você já podia fazer um de cada vez não é uma capacidade nova; é encanamento. A regra que importa para você como designer é esta: **comportamento de token genuinamente novo aterrissa só no Token-2022.** Transfer hooks, saldos confidenciais, metadados nativos, taxas de transferência, tudo isso, território de extensão. O SPL clássico em 2026 é um contrato congelado com um inquilino muito rápido, e se você se pegar esperando o SPL clássico ganhar uma feature, você está esperando um trem que foi formalmente cancelado.

![Fluxo de decisão: se o SPL clássico já faz o que você precisa, use como está; as únicas adições dele são três instruções de encanamento do p-token; toda capacidade genuinamente nova é roteada para o Token-2022.](assets/v06-flowchart.webp)

Esse reenquadramento também te entrega um filtro para todo conteúdo de Solana escrito antes de 2026, e você vai precisar dele, porque a internet não carimba data nos modelos mentais dela. Quando um tutorial mais antigo, uma nota de auditoria ou uma resposta de fórum faz uma afirmação sobre "o programa de token," passe a afirmação por uma pergunta: isso é uma afirmação sobre a interface, ou sobre a implementação? Afirmações de interface envelheceram perfeitamente. O layout de mint de 82 bytes, o conjunto de instruções, os discriminadores, os códigos de erro: tudo ainda verdadeiro, byte por byte, porque congelá-los era o acordo inteiro. Afirmações de implementação envelheceram mal da noite para o dia. Qualquer coisa sobre a estrutura interna do programa, suas características de performance, seus custos de CU por instrução: esse conteúdo agora descreve um programa que não roda mais em lugar nenhum. As afirmações estavam certas quando foram escritas. O inquilino mudou. Uma pergunta, dois baldes, e você consegue salvar seis anos de escrita do ecossistema em vez de desconfiar de tudo.

O congelamento é visível nos repositórios e crates também, e você vai tropeçar nisso em labs mais adiante se não ouvir agora. O velho monorepo SPL da solana-labs, aquele em que todo tutorial da era 2022 aponta, foi dissolvido; os programas de token agora moram na organização solana-program mantida pela Anza no GitHub, um repo por programa. Mesma interface, uma casa nova e um motor novo, e a divisão de repos não é cosmética: um monorepo fazia sentido quando um time entregava tudo junto, e um programa congelado e feature-complete não tem mais "junto" para entregar. Cada programa agora versiona e lança por conta própria. E do lado do Rust, a divisão tem o crate dela: o `spl-token-2022-interface` agora entrega os tipos e os layouts separados do `spl-token-2022`, que segue sendo o crate do programa. Leia essa divisão com seu vocabulário novo: o ecossistema está literalmente dividindo os crates dele para dizer de que lado da linha interface/implementação cada um se senta, e os dois versionam de forma independente — o crate de interface estava em 3.1.1 e o crate de programa em 11.0.0 quando eu conferi o crates.io em 2026-09-06. Seja preciso sobre o que isso é e o que não é: o crate do programa não está yanked e não carrega nenhum aviso de depreciação no nível do crate; o que está depreciado é no nível de item, re-exports individuais dentro dele te apontando para o crate de interface. Quando uma lição mais adiante te fizer calcular tamanhos de conta contra tipos do Token-2022, o import vem do crate de interface. Uma dependência na implementação é uma dependência de que você não precisava.

### O trade-off, nomeado

Todo presente neste design tem uma sombra, então nomeie os dois com honestidade. Uma interface congelada com um motor trocável é um presente para a compatibilidade: seu código nunca quebra, suas integrações nunca migram, e a rede inteira herda uma melhora de 61x enquanto dorme. É também uma cilada para modelos mentais: o endereço que você chama é estável, mas o que roda por trás dele não é, então "eu conheço o programa SPL Token" agora quer dizer "eu conheço a interface dele," e nunca mais pode querer dizer "eu conheço a implementação dele." Qualquer um cujo raciocínio de segurança, orçamento de CU ou intuição de performance dependia silenciosamente de detalhes de implementação do motor antigo teve essas suposições invalidadas no slot 419,472,000, e a chain não mandou notificação nenhuma. A garantia de compatibilidade é real e o rebaixamento epistêmico é real, e você segura os dois ao mesmo tempo. Esse é o modelo de 2026 do SPL clássico, e é o diferencial que quase ninguém ensina: interfaces são promessas, implementações são inquilinos, e na Solana o inquilino pode mudar sob consenso na fronteira de um único slot.

## Lab: leia a troca direto da chain

Quatro sondagens, todas guiadas, nada para preencher. Você não está construindo hoje; você está verificando que tudo acima é legível na chain e não folclore. Tempo total: cerca de quinze minutos depois que a toolchain Agave da abertura estiver instalada; a instalação é um custo único que pode sobreviver às sondagens. Pulou a instalação? As sondagens 1, 2 e 4 são renderizações da CLI de leituras simples de conta — rode a sondagem 3, que faz a mesma leitura do gate no kit, e aceite na confiança os checkpoints da CLI impressos aqui.

1. **Sonde o gate pela CLI.** Se você rodou o comando feature status lá em cima, já fez este passo; se não, rode agora:

   ```bash
   solana feature status ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
   ```

   Checkpoint: a linha diz `active since epoch 971` com slot de ativação `419472000`. Se sua CLI imprimir um erro de conexão, você provavelmente está atrás de algum rate limit default de RPC; espere alguns segundos e tente de novo.

2. **Despeje a conta do gate crua.** A visão de feature da CLI é um wrapper de conveniência; a verdade são 9 bytes de dados de conta, e depois de m01-l2 você está qualificado para ler 9 bytes com os próprios olhos:

   ```bash
   solana account ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
   ```

   Checkpoint: o owner é `Feature111111111111111111111111111111111111`, o comprimento é 9 bytes, e o hex dump diz `01 80 a2 00 19 00 00 00 00`.

![Hex dump anotado da conta de feature de 9 bytes: uma tag Some de 1 byte seguida do slot de ativação u64 little-endian 419,472,000, o primeiro slot da epoch 971.](assets/v07-annotated-code.webp)

3. **Decodifique programaticamente, no estilo kit.** Mesma leitura, mas pela stack sobre a qual você construiu o `decode-mint`, para a habilidade se acumular. Trabalhe dentro de `labs/m01-l2`, o workspace que você montou na lição passada: kit 7.1.1 e tsx 4.20.5 já estão fixados lá, versões exatas conforme a regra de peer-range daquela lição, e o `package.json` dele carrega o `type=module` de que o top-level await deste script precisa. Nessa pasta, crie `read-gate.ts`:

   ```typescript
   import { createSolanaRpc, address } from '@solana/kit';

   const GATE = address('ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP');
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.mainnet-beta.solana.com');

   const { value: account } = await rpc
     .getAccountInfo(GATE, { encoding: 'base64' })
     .send();

   if (!account) {
     console.log('No account at the gate address: the feature is not even pending.');
     process.exit(1);
   }

   const data = Buffer.from(account.data[0], 'base64');
   console.log(`owner:  ${account.owner}`);
   console.log(`bytes:  ${data.toString('hex')} (${data.length} bytes)`);

   // Feature account layout: 1-byte Option tag, then u64 LE activation slot.
   if (data[0] === 0) {
     console.log('status: pending activation (no slot set)');
   } else {
     const activatedAt = data.readBigUInt64LE(1);
     console.log(`status: ACTIVE since slot ${activatedAt.toLocaleString('en-US')}`);
   }
   ```

   Rode:

   ```bash
   npx tsx read-gate.ts
   ```

   Checkpoint: três linhas, terminando em `status: ACTIVE since slot 419,472,000`. Repare no `readBigUInt64LE`: a mesma honestidade de BigInt do seu campo supply em m01-l2, porque um slot de ativação é um u64 e números do JavaScript não são de confiança com um deles.

4. **Pegue o motor no flagra.** A própria conta do programa registrou a troca:

   ```bash
   solana program show TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA --url mainnet-beta
   ```

   Checkpoint, da minha sondagem de 2026-08-22: `Last Deployed In Slot: 419472000`, `Authority: none`, comprimento de dados 108,600 bytes. Sente com essa primeira linha por um segundo. O programa mais chamado da Solana reporta seu último deploy exatamente no slot de ativação do gate, o primeiro slot da epoch 971: a impressão digital de uma troca de código dirigida por consenso, escrita onde qualquer um pode ler. E `Authority: none` responde à resposta ingênua três da seção de teoria de uma vez por todas: nenhuma chave de upgrade existe para trocar este programa do jeito comum. Só um gate poderia ter feito isso, e um fez.

   Quinta sondagem, opcional, do lado do Rust: `cargo info spl-token-2022` (o cargo já vem com o rustup; `curl https://sh.rustup.rs -sSf | sh` se você nunca instalou) mostra que o crate agora mora sob os repos da solana-program, e imprime a versão atual dele — 11.0.0 na minha leitura de 2026-09-06, sem nenhum aviso de depreciação no crate em si. Rode `cargo info spl-token-2022-interface` ao lado e você vai ver o crate de interface versionado separadamente (3.1.1 na mesma leitura), que é de onde os nossos próximos labs ligados a Rust vão ler. Dois crates, duas linhas de versão, uma interface: exatamente a divisão que esta lição vem defendendo.

## Challenge

Nada de código hoje. O critério de avaliação desta lição é uma frase, e é mais difícil do que parece. Escreva, com suas próprias palavras e sem voltar ao texto, uma resposta em três partes que um colega conseguiria usar para agir: uma frase separando a interface da implementação do SPL clássico, uma declarando o status dele na mainnet com precisão (o que ativou, e quando, em epochs), e uma atribuindo a queda de CU da transferência à causa certa. Depois teste a si mesmo contra as duas ciladas que esta lição armou: se sua frase de status diz "aprovado" ou "aceito," você fez uma alegação de governança que o rastro de papel não sustenta (o PR foi mergeado em 2026-03-13, um fato do Git, e o front-matter ainda diz "Review"); se sua frase de CU deixa uma mudança do lado do cliente levar qualquer crédito, releia a seção do retorno. Quando suas três frases sobreviverem às duas ciladas, você domina o modelo.

Se um colega retrucar com "mas a página do SIMD diz Review," você agora sabe a correção, e ela generaliza: headers de documento atrasam, o estado da chain não, e você pessoalmente leu os 9 bytes que resolvem a questão.

Alguma coisa aqui não te soou bem, ou suas sondagens retornaram algo que as minhas não? Me conte: sinalize no canal de feedback do curso, de preferência com o comando e a saída colados. Um leitor que pega um número desatualizado nesta lição está fazendo exatamente o que esta lição ensina.

Na próxima lição, a pergunta em torno da qual este módulo inteiro vem circulando para de ser retórica. O SPL clássico está congelado, toda capacidade genuinamente nova mora no Token-2022, e o Token-2022 oferece 29 tipos de extensão com regras reais sobre quais combinações são sequer legais. Então como você decide? Você deriva o framework de decisão a partir do código-fonte, e seu inspetor `decode-mint` volta ao trabalho.

Boa sondagem! 🌱
