# Construa o programa: anchor-engineer, pinocchio-engineer, solana-qa-engineer

A lição passada terminou com `design.md`: uma conta da cafeteria, um cartão por cliente, três instruções. Nesta lição um engenheiro constrói as instruções, um engenheiro de QA as testa e uma auditoria lê o resultado. **Rust, Anchor e a Solana CLI entram aqui.** Se não estiverem na sua máquina, acompanhe pela tela e instale à noite. O comando da Solana CLI está no seu relatório do `/doctor`. A toolchain do Anchor aparece lá quando existir um workspace Anchor, então para hoje à noite é `rustup` para o Rust, depois `cargo install --git https://github.com/solana-foundation/anchor avm --force && avm install latest && avm use latest` para o Anchor. Uns vinte minutos.

## Por que isso importa

Código de programa é onde um tutorial copiado mais machuca, porque a API do Anchor mudou na 1.0 e a maior parte da web ainda mostra o formato antigo. O kit tem **dois implementadores e um testador**. **anchor-engineer** é o padrão: Anchor 1.x, constraints em vez de verificações escritas à mão, IDL gerada, testes Rust com LiteSVM. **pinocchio-engineer** existe para os casos medidos, *uma instrução ainda acima do orçamento de compute depois das alavancas do próprio Anchor, taxas de compute em volume, ou um limite de tamanho de binário*, e escreve cada verificação à mão. **solana-qa-engineer** é dono de como o programa é testado: qual harness para qual teste, a barra de cobertura, as linhas de base de compute e a limpeza do ruído que um modelo deixa em um diff. Três comandos amarram os três: `/build-program`, `/test-rust`, `/audit-solana`.

## Mão na massa

1. **Entregue o design ao engenheiro.** Digite dentro do Claude Code:

```text
Use o agente anchor-engineer. Monte um workspace Anchor 1.x para o Stamp com o template de testes LiteSVM e implemente as três instruções de design.md: create_shop, stamp e redeem. Uma conta de cartão por cliente como no design, aritmética com verificação, bumps armazenados e um teste Rust por instrução. Rode /build-program e /test-rust antes de devolver.
```

Observe o arquivo que ele lê primeiro: a referência de Anchor dentro do pack solana-dev. Depois, os formatos em que o próprio arquivo dele insiste, aqueles que **hábitos antigos erram**: espaço de conta a partir de `DISCRIMINATOR.len() + INIT_SPACE`, `CpiContext::new` recebendo uma chave de programa em vez de um account info, PDAs armazenadas validadas com `bump = card.bump`, códigos de erro começando em 6000. As regras da casa em `CLAUDE.md` acrescentam aritmética com verificação e nenhum `unwrap` fora dos testes. *A regra de fechamento dele é build, teste e uma auditoria para código que movimenta valor*, e ele reporta as instruções e contas que alterou junto com os resultados dos testes.

2. **Leia uma falha de teste do jeito do kit.** Se um teste falhar, o erro chega em hexadecimal. O arquivo do `/test-rust` decodifica: **2000 a 2999** são violações de constraint, **3000 a 3999** erros de conta, **4100** quer dizer que o programa foi carregado em um endereço diferente do `declare_id` dele, e **6000 para cima** é o seu próprio enum de erros. Peça os logs do programa antes de alguém mudar código:

```text
Me mostre os logs do programa do teste de redeem que está falhando e me diga se o errado é o teste ou o programa, antes de mudar qualquer coisa.
```

![A faixa de programa: design.md vai para o anchor-engineer, que devolve instruções e testes LiteSVM, /build-program e /test-rust rodam antes da devolução, o solana-qa-engineer acrescenta testes negativos e limpa o slop, o /audit-solana escreve um relatório de achados, e o pinocchio-engineer é o desvio tomado para um problema de compute medido.](assets/v01-program-lane.webp)

3. **Chame o testador.** Digite:

```text
Use o agente solana-qa-engineer. Revise os testes do Stamp contra a sua barra de cobertura: cada instrução, cada código de erro atingido por um teste negativo, e os limites, incluindo um redeem com nove carimbos e um stamp em um cartão na contagem máxima. Acrescente o que falta, rode /test-rust e limpe qualquer slop de IA do diff.
```

O arquivo dele traz uma **tabela de harness**: Mollusk para uma instrução e compute exato, LiteSVM para fluxos de várias instruções, Surfpool para estado real da mainnet, Trident para fuzzing antes da mainnet. Para o Stamp ele vai ficar no LiteSVM. A parte que as pessoas não esperam é a **passada de slop**: comentários que repetem o código, verificações defensivas em volta de chamadas internas confiáveis, logs de debug esquecidos, strings de erro prolixas. Ele remove tudo isso e diz o que removeu em uma ou duas frases.

4. **Audite.** Digite:

```text
/audit-solana
```

O comando delimita cada instrução, roda `cargo audit` e o clippy com os lints de aritmética e unwrap, percorre as checklists da skill de auditoria e escreve `docs/security-audit-<date>.md`: contagens por severidade, cada achado com `file:line`, o cenário de exploração e a correção, uma lista separada de suspeitas não confirmadas, e um veredito. Para o Stamp, espere que ele olhe com atenção para **quem pode chamar stamp**, porque uma chave da cafeteria que assina de um celular é o risco que o arquiteto já nomeou, e para a aritmética do redeem.

5. **Saiba para quando o outro engenheiro existe.** O Stamp nunca vai precisar de Pinocchio, e é esse o sentido de conhecer a regra: o arquivo do arquiteto diz *otimize uma instrução quente no lugar primeiro e porte só se ela continuar acima do orçamento*. Quando esse dia chegar, o `/profile-cu` mede o compute por instrução, o `/benchmark` compara com uma linha de base, e o pinocchio-engineer fecha com os dois. O arquivo dele também é a lista honesta do que a validação manual custa: discriminadores de um byte, alinhamento zero-copy, nenhuma IDL gerada.

## Está pronto quando

- O `/build-program` reporta o programa compilado, o tamanho e o id dele, e a IDL escrita.
- O `/test-rust` passa com um teste negativo por código de erro.
- `docs/security-audit-<date>.md` existe e você leu a linha do veredito.

## Fique atento

- **Recompile antes de testar.** O LiteSVM embute o binário do programa em tempo de compilação, então `cargo test` sozinho testa o que foi compilado por último. O comando de teste faz isso por você.
- **Dois enums de erro colidem.** Os dois começam em 6000 a menos que um defina um deslocamento, e a colisão é silenciosa. O arquivo do engenheiro sabe disso. Um tutorial copiado não.
- **Uma auditoria não é uma auditoria externa.** O `/audit-solana` recomenda uma na própria saída para programas que guardam fundos de usuários.
- **Sem toolchain hoje está bem.** Leia a transcrição na tela e rode os comandos de instalação depois. A lição 4 é só Node de novo.

## O que fica

Um engenheiro constrói a partir do design com a API atual do Anchor, um testador segura a barra de cobertura e limpa o slop, e um comando escreve um relatório de auditoria, **com build, teste e auditoria como a rotina fixa de fechamento**. *Pinocchio é uma decisão medida, nunca um padrão.*

## A seguir

Na próxima lição o engenheiro de frontend coloca o cartão de um cliente em uma página web com uma carteira conectada na devnet, o que roda só com Node. Os engenheiros de mobile e backend têm a vez deles na mesma lição.
