# O modelo confidencial: saldos encriptados numa chain pública

## Resumo

Na lição passada você resolveu as contas extras de um transfer hook do lado do cliente e viu exatamente por que metade do ecossistema de DEX recusa código arbitrário na transferência. Aquilo era código que você conseguia ler. Esta extensão esconde os próprios números.

Porque agora mesmo você consegue ler cada byte de uma transferência de SPROUT: remetente, destinatário, valor, tudo em aberto. Um token de folha de pagamento não pode ser entregue assim. A empresa inteira veria cada salário. Então a pergunta que esta lição responde é: como você coloca um valor num livro-razão público que os validadores conseguem verificar mas ninguém consegue ler?

Antes de qualquer termo técnico, aqui está a intuição de 30 segundos. Você consegue somar dois envelopes lacrados de dinheiro e saber que o total está certo sem abrir nenhum dos dois. Segure essa imagem. Ela é o truque inteiro, e tudo que vem depois é maquinaria construída em volta dela.

Primeiro, algo para rodar. A maquinaria tem um verificador on-chain, e ele está no ar na mainnet agora mesmo. Sonde ele (o curl já vem com o macOS e com todo Linux mainstream; no Debian, `apt install curl`):

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["ZkE1Gama1Proof11111111111111111111111111111",{"encoding":"base64"}]}'
```

Você deve receber de volta `"executable": true`, owner `NativeLoader1111111111111111111111111111111`, e 24 bytes de dados. Essa conta é o ZK ElGamal Proof Program, o programa nativo que confere toda prova de conhecimento zero desta lição. A gente vai decodificar esses 24 bytes no lab.

O recuo da ajuda: esta é uma lição de conceito, então o exemplo trabalhado é uma derivação, não um programa. Eu percorro o modelo de ponta a ponta com você, incluindo uma transferência confidencial de SPROUT desmontada campo por campo. No lab você produz o artefato você mesmo: uma tabela de campos público-versus-encriptado mais as três provas, cada uma derivada da trapaça que ela fecha. O challenge é solo: você ataca o modelo com uma prova removida e prevê exatamente o que quebra. Nenhum código novo é entregue hoje. A próxima lição configura tudo isso de verdade.

## Derivando o modelo confidencial

### Envelopes lacrados, depois o termo técnico

Imagine um tabelião que liquida dívidas entre pessoas que se recusam a revelar seus salários. Cada pessoa entrega um envelope lacrado com dinheiro dentro. O tabelião não consegue abrir envelope nenhum. Mas esses envelopes são especiais: empilhe dois deles e a pilha se comporta como um único envelope contendo a soma. O tabelião consegue verificar que o envelope A mais o envelope B pesa exatamente o que o envelope C pesa, sem nunca ver uma única nota.

Essa propriedade tem um nome: um compromisso homomórfico. "Compromisso" porque o envelope prende você a um valor que você não pode mudar depois. "Homomórfico" porque operações sobre os envelopes lacrados correspondem a operações sobre os valores escondidos: some os ciphertexts, e você somou os valores lá dentro.

![Um tabelião empilha dois envelopes lacrados e verifica que o envelope combinado contém a soma sem abrir nenhum, mapeando envelopes para ciphertexts e o empilhamento para a adição de ciphertext.](assets/v01-diagram.webp)

A extensão de transferência confidencial do Token-2022 é esse tabelião, industrializado. Todo saldo confidencial on-chain é um envelope lacrado. Toda transferência confidencial é o validador empilhando envelopes: subtraia este ciphertext do saldo do remetente, some aquele à pilha pendente do destinatário. A chain faz aritmética sobre números que ela nunca vê.

A construção de envelope se chama Twisted ElGamal. Você não precisa da álgebra dela para usá-la bem, mas precisa de três das propriedades dela, porque toda decisão de design rio abaixo decorre delas:

1. A encriptação é feita contra uma chave pública, então qualquer um pode lacrar um envelope endereçado a você. Um remetente encripta o valor da transferência sob a sua chave sem o seu envolvimento.
2. Ciphertexts somam. A propriedade homomórfica da intuição lá em cima é real: o programa literalmente chama adição e subtração de ciphertext sobre saldos (`ciphertext_arithmetic::add` e `subtract_from` no processor).
3. A decriptação é cara. Abrir o seu próprio envelope significa resolver um logaritmo discreto, o que só é tratável quando o número escondido é pequeno. Essa única propriedade, a decriptação sendo o caminho lento, molda mais do design da extensão do que qualquer outra. Mantenha ela carregada.

### Duas chaves por conta: uma para a chain, uma para você

Aqui está a primeira consequência. Se decriptar um ciphertext ElGamal é lento, como é que uma carteira te mostra o seu próprio saldo sem moer uma busca de logaritmo discreto toda vez que você abre o app?

A resposta da extensão: toda conta confidencial carrega duas encriptações do mesmo saldo, sob duas chaves diferentes, servindo a dois senhores diferentes. Olhe o estado real da conta, do `spl_token_2022_interface` (aparado para os campos que importam hoje):

```rust
pub struct ConfidentialTransferAccount {
    /// `true` if this account has been approved for use.
    pub approved: Bool,
    /// The public key associated with ElGamal encryption
    pub elgamal_pubkey: PodElGamalPubkey,
    /// The low 16 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_lo: EncryptedBalance,
    /// The high 48 bits of the pending balance (encrypted by `elgamal_pubkey`)
    pub pending_balance_hi: EncryptedBalance,
    /// The available balance (encrypted by `elgamal_pubkey`)
    pub available_balance: EncryptedBalance,
    /// The decryptable available balance
    pub decryptable_available_balance: DecryptableBalance,
    /// If `false`, the account rejects incoming confidential transfers
    pub allow_confidential_credits: Bool,
    /// If `false`, the base account rejects any incoming transfers
    pub allow_non_confidential_credits: Bool,
    /// Number of Deposit and Transfer instructions that have credited pending
    pub pending_balance_credit_counter: U64,
    /// Max credits allowed before ApplyPendingBalance must run
    pub maximum_pending_balance_credit_counter: U64,
    // ...expected/actual credit counter bookkeeping trimmed
}
```

O `available_balance` é a cópia do mint: um ciphertext Twisted ElGamal sobre o qual a chain consegue fazer aritmética, e aquele contra o qual toda prova é conferida. O `decryptable_available_balance` é seu: o mesmo número lacrado com uma chave AES que só você tem (`AeCiphertext` na fonte). A decriptação AES é instantânea. Sua carteira lê esse campo, decripta em um microssegundo, e te mostra o seu salário. A chain nunca toca nele; o programa só guarda qualquer ciphertext decriptável novo que você entregar a ele sempre que o seu saldo muda, porque só você consegue produzir ele.

Um saldo, dois envelopes, duas plateias. O ciphertext ElGamal é a verdade que a chain impõe. O ciphertext AES é um cache de conveniência para o dono dele. Se os dois um dia se descolarem, a cópia da chain ganha, e a carteira tem que recorrer a abrir o envelope ElGamal do jeito difícil. A propriedade 3 deveria te fazer estremecer nessa frase, e com razão: uma busca de logaritmo discreto sobre o intervalo inteiro de 64 bits não é prática. A saída de emergência é que a busca é limitada pelo que o saldo plausivelmente pode ser, funciona nos mesmos chunks pequenos que o resto da extensão impõe, e pode ser pré-computada e retomada offline, então a recuperação é lenta-mas-finita para saldos realistas em vez de instantânea. Trate o cache AES como estrutural, não decorativo, e trate perder a chave AES como um incidente.

![Diagrama dividindo um saldo escondido em dois ciphertexts armazenados, uma cópia ElGamal sobre a qual a chain computa mas que os donos decriptam devagar, e uma cópia AES que os donos leem instantaneamente.](assets/v02-diagram.webp)

### Pendente versus disponível: por que o dinheiro que entra fica numa sala de espera

Agora a segunda consequência, e ela explica os campos mais estranhos daquela struct: por que existe um `pending_balance`, para começo de conversa, dividido em metades `lo` e `hi`?

Percorra isso. Alguém te manda uma transferência confidencial. O valor chega encriptado sob a sua chave ElGamal, e a chain o soma homomorficamente ao seu saldo. Beleza. Mas o seu `decryptable_available_balance`, o cache AES, agora está desatualizado, e o remetente não consegue consertar: produzir um ciphertext AES novo exige a sua chave AES, que o remetente não tem e nunca pode ter.

Pior: se as transferências que entram aterrissassem direto no `available_balance`, elas correriam contra os seus próprios gastos. Você gera uma prova contra o saldo X, alguém te credita no meio do voo, o seu saldo agora é X mais algo que você não consegue ver, e a sua prova não bate mais com o ciphertext on-chain. Todo pagamento que entra invalidaria todo pagamento que sai que você tivesse em andamento.

Então a extensão dá a toda conta uma sala de espera. Os créditos confidenciais que entram aterrissam no `pending_balance`, e só o dono os move para o `available_balance` assinando uma instrução `ApplyPendingBalance`, que também entrega ao programa um cache AES recém re-encriptado. O `pending_balance_credit_counter` conta os depósitos desde o último apply, e o `maximum_pending_balance_credit_counter` limita quantos podem se acumular (65,536 por padrão) antes de a conta parar de aceitar créditos até o dono varrer a pilha. A divisão em um ciphertext `lo` e um `hi` existe pelo motivo que você já tem: a decriptação é uma busca de logaritmo discreto, então todo chunk encriptado precisa continuar pequeno o suficiente para o dono dele abrir. Seja preciso sobre qual divisão é qual, porque elas diferem. O valor de uma transferência que entra chega como um chunk baixo de 16 bits mais um chunk alto de 32 bits (esse formato 16 + 32 é exatamente de onde vem o teto de transferência abaixo de 2^48 mais adiante nesta lição), e cada chunk é somado ao seu próprio balde pendente. Os baldes em si carregam peso posicional, `lo` para os 16 bits baixos do saldo e `hi` para os 48 bits acima deles, e eles são deliberadamente mais folgados que qualquer transferência isolada para que até 65,536 créditos possam se acumular entre os applies enquanto os dois baldes ficam dentro de um intervalo de decriptação pesquisável.

![Fluxograma de um crédito confidencial que entra aterrissando no saldo pendente, esperando o ApplyPendingBalance do dono incorporá-lo ao saldo disponível e atualizar o cache AES.](assets/v03-flowchart.webp)

Se você já usou um banco que mostra depósitos "em processamento" separados do seu saldo gastável, você já tem o formato disso. A diferença é o motivo: o banco está rodando checagens de fraude, enquanto esta conta está esperando a única pessoa viva que consegue re-lacrar o envelope legível.

### As três provas, derivadas das três trapaças

Aqui é onde o modelo se paga, e onde eu quero você derivando em vez de decorando. A chain é um tabelião fazendo aritmética sobre envelopes que ele não consegue abrir. Então faça a pergunta adversarial: se ninguém consegue ver os valores, o que me impede de mentir?

Tente os consertos ingênuos primeiro. "Faça os validadores decriptarem e conferirem" se autorrefuta; o ponto inteiro é que eles não conseguem. "Confie na matemática do remetente" morre em um bloco; alguém transfere para si mesmo um envelope alegando menos um milhão e o supply infla em silêncio. A resposta de verdade é a que faltava na história do envelope lacrado: junto com os envelopes, o remetente precisa anexar provas de conhecimento zero, afirmações que convencem o verificador de que uma alegação sobre os valores escondidos é verdadeira sem revelar mais nada sobre eles.

Beleza. Mas por que TRÊS provas? Por que não uma prova só que diz "esta transferência é honesta"? Porque "honesta" não é uma alegação só. Sente e tente de verdade trapacear este sistema e você vai encontrar exatamente três mentiras distintas disponíveis para um remetente, e cada uma precisa da própria refutação. Esta é a derivação pela qual a lição existe, então vá devagar.

Trapaça um: provar o intervalo de um restante fabricado. Aqui está a sutileza que torna esta trapaça possível, para começar. A subtração homomórfica da chain produz o seu saldo novo como um ciphertext, mas uma prova de intervalo (a refutação da trapaça três) não roda sobre esse ciphertext diretamente: ela prova afirmações sobre compromissos que o REMETENTE fornece, incluindo um para o saldo que o remetente alega ter sobrado depois do débito. A chain não consegue abrir o próprio ciphertext pós-subtração para conferir a alegação, então nada até aqui amarra o restante alegado à realidade. Eu poderia ter 3 SPROUT, te mandar 5, e entregar ao verificador um "saldo restante" de 10 lindamente bem formado e confortavelmente dentro do intervalo que eu inventei para a ocasião, enquanto o meu saldo verdadeiro dava a volta para negativo por baixo. A refutação é uma prova de igualdade, `CiphertextCommitmentEqualityProof` na fonte: ela certifica que o seu ciphertext novo de saldo disponível, aquele produzido pela subtração on-chain, se compromete com o mesmo valor que o compromisso de restante sobre o qual o resto do pacote de provas está testemunhando. O restante alegado É o restante real, então toda garantia que as outras provas dão se prende aos livros de verdade, não a uma história sobre eles.

![A prova de igualdade solda o compromisso de restante alegado do remetente ao ciphertext de saldo pós-débito da chain, fechando a trapaça do restante fabricado.](assets/v04-diagram.webp)

Trapaça dois: mandar lixo. Ciphertexts ElGamal são só pontos de curva; nada nos bytes os obriga a ser uma encriptação bem formada de qualquer coisa sob a chave de quem quer que seja. Eu poderia te entregar um "ciphertext" que decripta para besteira sob a sua chave, ou pior, encriptar o valor real para você mas anexar bytes deformados para o auditor, então o compliance vê ruído enquanto a transferência passa batido. A refutação é uma prova de validade de ciphertext agrupado, `BatchedGroupedCiphertext3HandlesValidityProof`: o valor está corretamente encriptado, como um único ciphertext agrupado com três handles, sob a chave do remetente, a chave do destinatário E a auditor key opcional do mint. Mesmo número, três leitores, comprovadamente. Esta é a prova que faz o assento de auditor no `ConfidentialTransferMint` significar alguma coisa; a gente configura esse assento na próxima lição.

Trapaça três: ficar negativo. A aritmética de ciphertext é aritmética módulo a ordem de um grupo, e aritmética modular não sabe o que é um número negativo. Subtrair um "valor" que dá a volta é a versão confidencial de um underflow de inteiro, e você já sabe o que um underflow compra para um atacante em texto plano: subtraia um de um saldo zero e aterrisse em quase 2^64. A refutação é uma prova de intervalo, `BatchedRangeProofU128`: todo valor escondido na transferência é não-negativo e está dentro dos limites. O U128 no nome é contabilidade honesta. Uma única prova em lote cobre o seu saldo restante (64 bits, o compromisso de restante fornecido pelo remetente que a prova de igualdade da trapaça um solda aos livros de verdade) mais o chunk baixo (16 bits) e o chunk alto (32 bits) do valor da transferência, preenchido com 16 até uma potência de dois: 128 bits de valores comprometidos, provados em intervalo juntos.

Três mentiras, três provas, e a atribuição é exata. Remova qualquer uma e a trapaça dela reabre; você vai demonstrar isso você mesmo no challenge.

![Diagrama de mapeamento pareando cada uma das três trapaças do remetente com a prova de conhecimento zero que a fecha e a garantia que cada uma dá, todas verificadas pelo ZK ElGamal Proof Program.](assets/v05-diagram.webp)

A conferência, notavelmente, não é feita pelo próprio Token-2022. O SIMD-0153 deu à rede um programa nativo dedicado para isso, o ZK ElGamal Proof Program que você sondou no resumo, no ar em `ZkE1Gama1Proof11111111111111111111111111111`. O Token-2022 confirma que cada prova foi verificada por esse programa e então faz a aritmética de envelopes. Divisão de trabalho: um programa que sabe criptografia, um programa que sabe tokens.

### Por que uma transferência é várias transações, e por que os valores param em 2^48

Então uma transferência confidencial é ciphertexts mais três provas. Agora o fato operacional feio: essas provas são grandes. Uma prova de intervalo sozinha chega a centenas de bytes, e o trio junto estoura com folga o que cabe ao lado de uma instrução de transferência dentro da transação de 1,232 bytes da Solana. As provas são grandes demais para pegar carona, então uma transferência lógica vira várias transações dependentes hoje.

O mecanismo que torna isso viável é a conta de estado de contexto: uma conta de vida curta, de propriedade do programa de provas, que registra "a prova X foi verificada" para que uma transação posterior possa apontar para ela em vez de carregar a prova. A dança, em ordem:

1. Criar e verificar: para cada prova, uma transação entrega a prova ao ZK ElGamal Proof Program, que a verifica e escreve uma conta de contexto (a prova de intervalo geralmente precisa de uma transação só para ela).
2. Transferir: a instrução `Transfer` do Token-2022 de fato executa, referenciando as três contas de contexto em vez de provas inline.
3. Fechar: as contas de contexto são fechadas e o rent delas recuperado.

![Fluxograma de uma transferência confidencial dividida em transações dependentes: provas verificadas em contas de contexto primeiro, depois a transferência referenciando elas, depois a limpeza das contas de contexto, restringida pelo limite de transação de 1,232 bytes.](assets/v06-flowchart.webp)

Isso não é para sempre, e já está em movimento. O formato de transação v1 (a linha do SIMD-0296, agora carregada pelo SIMD-0385) aumenta o envelope precisamente para que fluxos como este possam colapsar em uma única transação, e ele já foi entregue no Agave. Mas entregue não é ativado, e ativado é por cluster. O feature set do Agave nomeia o gate `enable_tx_v1` e declara o endereço dele em `feature-set/src/lib.rs`:

```bash
solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url mainnet-beta
solana account txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL --url devnet
```

Em 2026-09-06 a mainnet respondeu `Error: AccountNotFound` — sem conta, então não ativada e nem sequer preparada — enquanto a devnet devolveu uma conta de propriedade de `Feature111111111111111111111111111111111111` cujos nove bytes de dados decodificam como uma tag `1` seguida do slot de ativação 492,480,000. A devnet tem. A mainnet não. Então 1,232 bytes continuam sendo a lei onde os seus usuários estão, a dança de múltiplas transações continua sendo a realidade para a qual você projeta, e a devnet agora é um cluster onde essa restrição em particular silenciosamente não se reproduz — o que é uma cilada por si só se você só testar lá.

Duas notas sobre a sondagem em si, porque a óbvia não funciona. O `solana feature status` imprime só os gates compilados na CLI que você está segurando: 76 linhas no solana-cli 3.1.10, e este gate não está entre eles, então tanto `| grep -i tx_v1` quanto `solana feature status <that address>` voltam vazios ou como `Unknown feature`. Ler a conta diretamente é a sondagem que não fica desatualizada, porque ela pergunta para a chain em vez de para o binário. E re-confira antes de citar este parágrafo para alguém: ele virou em um cluster entre a redação desta lição e a última revisão dela.

A última restrição é o teto do valor, e a essa altura você consegue derivar ele sozinho. Os valores são encriptados em chunks pequenos o suficiente para decriptar (um chunk baixo de 16 bits, um chunk alto de 32 bits), então um único depósito ou transferência tem teto abaixo de 2^48. A fonte declara isso como uma constante:

```rust
/// Maximum bit length of any deposit or transfer amount
///
/// Any deposit or transfer amount must be less than 2^48
pub const MAXIMUM_DEPOSIT_TRANSFER_AMOUNT: u64 =
    (u16::MAX as u64) + (1 << 16) * (u32::MAX as u64);
```

Isso resulta em 281,474,976,710,655 unidades base. Para o SPROUT com 6 decimals, uma transferência confidencial chega no máximo a uns 281 milhões de tokens inteiros, o que é de sobra para folha de pagamento. Mas não é um u64 de intervalo completo, e um mint com 9 decimals perde três ordens de grandeza. Matemática de teto pertence à sua revisão de design, não a notas de incidente de produção.

### A derivação trabalhada: uma transferência confidencial de SPROUT, campo por campo

Agora monte o modelo inteiro dissecando uma transferência. Digamos que eu te mando 5 SPROUT confidencialmente. Aqui está tudo que chega ao livro-razão, ordenado por quem consegue ler. Esta tabela é o padrão para o artefato que você produz no lab, então leia ela como um gabarito trabalhado.

| Campo no fio | Público ou encriptado | Quem consegue ler, e qual prova o condiciona |
| --- | --- | --- |
| Conta de token do remetente (e pubkey do dono) | Público | Todo mundo. Nenhuma prova envolvida; assinaturas autorizam como de costume |
| Conta de token do destinatário (e pubkey do dono) | Público | Todo mundo. Encriptado não é anônimo |
| Endereço do mint, programa, o fato de uma transferência ter acontecido | Público | Todo mundo. A análise de tráfego vê a aresta, não o peso |
| Valor da transferência, ciphertext agrupado (lo + hi) | Encriptado | Só as chaves do remetente, do destinatário e do auditor; condicionado à prova de validade (bem formado sob os três handles) |
| Ciphertext de saldo disponível novo do remetente | Encriptado | Só o remetente; condicionado à prova de igualdade (se compromete com o valor pós-débito verdadeiro) |
| Saldo decriptável novo do remetente (AES) | Encriptado | Só o remetente; nenhuma prova, a chain guarda ele às cegas |
| Não-negatividade e limites de todo valor escondido | Provado, não revelado | Condicionado à prova de intervalo sobre 64 + 16 + 32 bits comprometidos |

Repare no que a coluna pública soma: as duas identidades, o token, o tempo, a taxa da transação paga em SOL visível. Confidencialidade aqui é exatamente uma propriedade, valores escondidos, e nada mais. Um analista ainda consegue desenhar o seu grafo de pagamentos inteiro; ele só não consegue pesar as arestas. Se o seu modelo de ameaça precisa de participantes escondidos, esta extensão não fornece isso, ponto final, e fingir o contrário é como times de compliance levam surpresas desagradáveis.

![Comparação lado a lado de uma transferência de SPROUT comum e de uma confidencial, em que os campos de identidade e de mint continuam públicos e só os campos de valor e de saldo passam para a forma encriptada com três provas.](assets/v07-comparison.webp)

### O que a confidencialidade custa

Toda extensão neste curso veio com uma etiqueta de preço, e a desta é salgada: a confidencialidade é comprada com composabilidade e simplicidade.

Composabilidade primeiro. Uma AMM cota uma troca lendo saldos e valores de pool. Valores encriptados significam que não há nada para ler, então nenhuma AMM consegue cotar o token. Isso não é cautela hipotética: você já encontrou a cancela de mints da Raydium duas vezes, primeiro como a allowlist de cinco extensões do m02-l1 e de novo na lição de transfer hook, e a política dela em relação a esta extensão é rejeição categórica, sob o argumento declarado de que valores encriptados impedem a precificação. Seja preciso sobre o escopo disso, porque o Módulo 5 vai te cobrar: o que nenhuma AMM consegue cotar é um valor encriptado, não necessariamente um mint que meramente carrega a extensão. A allowlist da Raydium recusa a extensão em si, enquanto a tabela publicada da Orca suporta esses mints "somente para transferências não confidenciais". De um jeito ou de outro, o caminho confidencial é o não-roteável. Para folha de pagamento isso é irrelevante. Para qualquer coisa que precisa de um mercado líquido, é desqualificante, e nenhuma quantidade de engenharia do seu lado muda isso.

Simplicidade em segundo. Uma transferência lógica são várias transações dependentes com geração de prova no meio, o que significa fluxos de cliente, retentativas e estados de falha que você não tem com uma transferência comum. Os valores têm teto abaixo de 2^48. Os dois lados de uma transferência precisam de contas confidenciais configuradas, com chaves ElGamal e AES derivadas e gerenciadas. E empilhe quatro ciladas por cima, cada uma uma restrição de design que você herda no momento em que você recorre a esta extensão:

- A transferência confidencial precisa ser habilitada na criação do mint. Você não pode adicionar ela a um mint existente depois; não existe retrofit, só um mint novo e uma migração.
- Encriptado não é anônimo. Os endereços de remetente e destinatário continuam totalmente públicos; só o valor fica escondido.
- Um transfer hook não consegue ver nem agir sobre valores confidenciais. As duas extensões compõem mecanicamente — uma transferência confidencial ainda invoca o hook, entregando a ele o valor sentinela `u64::MAX`, e o próprio mint do PYUSD carrega as duas — mas o seu hook fica cego para valores no caminho confidencial. Qualquer hook cuja lógica se condicione a valores precisa ser projetado para esse sentinela, ou a combinação é uma cilada.
- O teto abaixo de 2^48 significa que um saldo confidencial não é um u64 de intervalo completo, e a sua validação de valor precisa dizer isso.

Quando NÃO recorrer a ela, então, se reduz a um teste: qualquer token que precise ser negociado numa DEX ou liquidar em uma única transação está fora. O que sobra é o caso da folha de pagamento, liquidação B2B, operações de tesouraria: fluxos entre contrapartes que já se conhecem e simplesmente não querem os valores num outdoor.

Uma nota de honestidade antes do lab, porque este curso não exagera adoção. Não existe, na hora em que escrevo isto, nenhum emissor nomeado em produção rodando transferências confidenciais em escala para o qual eu possa te apontar. O modelo é real e está no ar; o deployment emblemático não está. O PYUSD já vem com o conjunto confidencial entre as oito extensões TLV dele, configurado mas dormente, e você vai ler a config dormente dele você mesmo em uns dois minutos. A trilha de papel do ecossistema é estranha do mesmo jeito: solana.com/solutions/token-extensions, uma página oficial no ar, ainda diz que as transferências confidenciais vão chegar assim que o Agave 2.0 "tiver sido adotado pela rede, o que se espera que aconteça até o fim de 2024". Pergunte à rede o que ela de fato roda; uma sondagem resolve:

```bash
curl -s https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}'
```

Em 2026-08-22 isso devolveu `"solana-core": "4.2.0"`, a linha Agave. Duas versões maiores depois da promessa, a promessa ainda está na página. A mesma página continua útil como a citação para as cinco firmas de auditoria que revisaram o programa da extensão (Halborn, Zellic, Trail of Bits, NCC Group, OtterSec). E a educação oficial da Solana congelou no meio do enredo: o repositório solana-foundation/developer-content foi arquivado como somente leitura em 2025-01-24, então todo curso oficial é anterior ao formato atual desta suíte. O que é mais ou menos por que esta lição existe. Você está aprendendo um material cuja trilha de documentação parou de andar antes da maquinaria.

![Card de trade-off de dois painéis listando o que as transferências confidenciais entregam, valores verificáveis escondidos, contra custos como a composabilidade em DEX perdida, a liquidação em múltiplas transações e o teto de valor, terminando em uma regra de decisão.](assets/v08-comparison.webp)

## Lab: sonde a maquinaria, depois derive o modelo no papel

O artefato produzido nesta lição é uma tabela de campos preenchida mais as três provas com as garantias delas, escritas por você mesmo. Os passos 1 e 2 são leituras ao vivo contra a mainnet; os passos 3 a 5 são a derivação. Você precisa do `curl` (já comprovadamente funcionando pela abertura) e do `python3` (já vem com o macOS; no Debian, `apt install python3`). As respostas RPC abaixo foram capturadas em 2026-08-22 contra um node reportando Agave 4.2.0; contas ao vivo mudam, então espere que os seus bytes batam e os seus números de slot não.

1. Decodifique o verificador que você sondou no resumo. Aqueles 24 bytes de dados de conta são base64; abra eles:

   ```bash
   echo "emtfZWxnYW1hbF9wcm9vZl9wcm9ncmFt" | base64 -d
   ```

   Saída esperada: `zk_elgamal_proof_program`. Programas nativos carregam o nome deles como dados de conta, então você acabou de ler a placa de identificação do verificador on-chain. Repare no que a existência dele significa: a verificação de provas é uma primitiva no nível da rede (SIMD-0153), não algo que cada programa de token reimplementa.

2. Leia uma config confidencial dormente na natureza. O mint do PYUSD carrega o par confidencial entre as oito extensões TLV dele. Puxe o mint parseado e filtre:

   ```bash
   curl -s https://api.mainnet-beta.solana.com -X POST \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo",{"encoding":"jsonParsed"}]}' \
   | python3 -c "import json,sys; exts=json.load(sys.stdin)['result']['value']['data']['parsed']['info']['extensions']; print(json.dumps([e for e in exts if e['extension']=='confidentialTransferMint'], indent=2))"
   ```

   Esperado: uma entrada `confidentialTransferMint` com `auditorElgamalPubkey: null`, um `authority`, e `autoApproveNewAccounts: false`. Leia isso contra a lição: nenhuma auditor key está definida, e com a flag em false, os holders ainda conseguem configurar contas confidenciais livremente, mas toda conta configurada fica inutilizável até o emissor aprovar ela explicitamente. Configurado mas dormente, verificado pela sua própria leitura; a próxima lição percorre esse fluxo de configurar-e-depois-aprovar de ponta a ponta.

3. Monte a tabela de campos. Pegue a transferência "você manda 5 SPROUT confidencialmente para um colega de time" e escreva uma tabela de duas colunas: todo campo que chega ao livro-razão na coluna um, `public` ou `encrypted` na coluna dois. Trabalhe a partir da seção de dissecação mas escreva de cabeça primeiro; você está conferindo se o modelo está na sua cabeça ou ainda na página. Linhas mínimas: conta do remetente, conta do destinatário, mint, o fato da transferência, o valor da transferência, o ciphertext de saldo novo do remetente, o cache AES.

4. Derive as três provas. Embaixo da tabela, escreva as três trapaças que um remetente poderia tentar, com as suas próprias palavras. Para cada trapaça, nomeie a prova que a fecha (nomes de tipo exatos: `CiphertextCommitmentEqualityProof`, `BatchedGroupedCiphertext3HandlesValidityProof`, `BatchedRangeProofU128`) e declare a única garantia que ela dá, uma linha cada. Se alguma garantia te tomar mais de uma linha, você está descrevendo o mecanismo, não a garantia; comprima até virar uma alegação.

5. Feche o loop com os contadores. Acrescente uma última linha ao seu artefato respondendo: por que o remetente não consegue atualizar o seu saldo decriptável, e qual instrução conserta isso? Se a sua resposta nomeia a chave AES e o `ApplyPendingBalance`, a maquinaria de saldo pendente aterrissou.

Checkpoint: a sua tabela marca exatamente uma família de campos como encriptada (os ciphertexts de valor e de saldo) e tudo que tem formato de identidade como público; as suas três linhas de prova pareiam cada uma uma trapaça com um nome de tipo e uma garantia. Esse artefato é o critério de avaliação desta lição, e a próxima lição assume que você consegue reproduzir ele de memória.

## Challenge

Solo, sem apoio: quebre o modelo três vezes, no papel.

Para cada uma das três provas, assuma que o verificador pulou aquela prova e só aquela prova, e escreva o ataque concreto que um remetente malicioso roda: o que ele submete, o que a chain aceita, e qual é o dano (supply inflado, saldo corrompido, auditor cegado). Três ataques, um parágrafo curto cada. O exercício força o ponto que a lição derivou: as provas não são defesa em profundidade, elas são três fechaduras em três portas diferentes, e qualquer porta aberta é fatal.

Extra, para quem mergulha na fonte: o processor de transferência no `token-2022` aceita dois offsets de prova opcionais adicionais, `fee_sigma_proof` e uma prova de validade de ciphertext de taxa, usados quando o mint também carrega taxas de transferência confidenciais. Antes de ler mais a fundo na extensão de taxa, preveja a partir de primeiros princípios quais duas trapaças novas uma taxa escondida introduz. Você tem todas as ferramentas de que precisa: uma taxa é só mais um valor escondido sobre o qual alguém pode mentir.

## Checkpoint e o que vem a seguir

Você agora consegue enunciar o modelo confidencial sem enrolação: saldos são envelopes Twisted ElGamal que a chain soma sem abrir, os donos mantêm um cache AES de leitura rápida, os créditos que entram esperam num balde pendente até serem aplicados, e toda transferência arrasta três provas de conhecimento zero até o ZK ElGamal Proof Program, uma por trapaça disponível. Você também sabe o custo: nenhuma DEX vai cotar ele, uma transferência são várias transações hoje, os valores param abaixo de 2^48, e nada disso pode ser parafusado depois da criação do mint. Esse é um modelo mental completo, produzido por você, no papel, e honestamente isso te coloca à frente da maior parte do material escrito do ecossistema sobre esta extensão.

Agora você tem o modelo: compromissos, três provas, um assento de auditor opcional ainda vazio. Na próxima lição você configura isso de verdade: uma auditor key, o registro ElGamal, supply confidencial, e o muro de múltiplas transações que você vai de fato bater quando o SPROUT apagar as luzes. Mantenha as três provas na ponta dos dedos; o poder inteiro do auditor na próxima lição depende da do meio.
