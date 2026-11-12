# Coloque no ar: deploy, devops-engineer, tech-docs-writer

O Stamp tem um programa, testes, um relatório de auditoria, uma página web e, da lição passada, uma proposta de token, um design de swap e um plano de jogo. Esta lição coloca ele na devnet com a parada de aprovação do kit, dá um pipeline a ele, escreve a documentação que um estranho precisa e deixa um revisor encontrar o app sem contexto. Uns quinze minutos, e o deploy precisa da Solana CLI da lição 3.

## Por que isso importa

Lançar é onde dinheiro e chaves aparecem, então é onde o firewall do kit importa mais. **O firewall é um conjunto de regras de permissão e de sandbox geradas a partir de um nível**, nomeado em `.claude/security.json`, mais guard hooks que rodam antes de um comando, mais o plugin safe-ai-skill. O nível padrão, **Relaxed**, deixa a toolchain trabalhar e **para deploys de programa, upgrades, mudanças de autoridade e transferências para a sua aprovação**, nomeando o cluster, exceto em um validador local. Quatro ações irreversíveis são recusadas de cara em todos os níveis, um deploy final, `program close --bypass-warning`, `program-v4 finalize` e desabilitar uma autoridade de token, e o agente entrega a você o comando para rodar por conta própria. Dois agentes trabalham nesta faixa: **devops-engineer** para CI, fixação de versão da toolchain, infraestrutura de RPC e monitoramento, e **tech-docs-writer** para a documentação que tira cada fato do código, não da memória.

## Mão na massa

1. **Faça o deploy na devnet.** Digite dentro do Claude Code:

```text
/deploy devnet
```

Os passos do próprio comando: confirmar o RPC e o deployer com `solana config get`, rodar `anchor keys sync` em um programa nunca implantado para que `declare_id` bata com o keypair, abastecer o deployer com um airdrop ou pelo faucet web, e então fazer o deploy **com o cluster no comando**, para que o prompt de aprovação mostre qual cluster ele resolveu. Esse prompt é o firewall. Diga sim, e o comando publica a IDL no Program Metadata, confere o programa com `solana program show` e registra o id em `.program-id-devnet` e no env do seu app. *Mainnet é um passo separado*: o comando só começa quando você pede, e primeiro relata o que falta, uma auditoria sem achados Critical ou High abertos, testes e fuzzing passando, uma rodada na devnet dos fluxos reais, um plano para a upgrade authority e uma auditoria externa para programas que guardam fundos de usuários.

2. **Veja em que nível você está.** Digite:

```text
/firewall
```

Ele relata o nível declarado e o aplicado e a quantidade de regras. **Off** (desligado) desliga o sandbox em uma máquina que você já isolou, **Medium** (médio) é para ler código que você não escreveu, **High** (alto) nega escritas na mainnet de cara e é o único nível que impede o agente de editar a configuração do próprio kit, e uma troca vale a partir da próxima sessão. Peça ao Claude para ler o `.env` uma vez e veja o plugin de segurança recusar. Essa recusa é do plugin, então se o instalador pulou o safe-ai-skill, o Relaxed deixa o `.env` legível e a leitura passa. O jeito do kit de saber quais chaves estão definidas é `bash .claude/bin/env-keys.sh`, que imprime nomes e `set` ou `empty`, nunca um valor.

![A faixa de entrega: /deploy devnet para no prompt do firewall que nomeia o cluster, devops-engineer fixa a versão da toolchain e escreve o CI com chaves só de devnet, tech-docs-writer declara program id, autoridade, seeds, layouts e erros a partir do código, e /product-review encontra o app como seu próximo usuário.](assets/v01-ship-lane.webp)

3. **Dê um pipeline a ele.** Digite:

```text
Use o agente devops-engineer. Configure o CI do Stamp com /setup-ci-cd: build, testes e os lints de auditoria em todo pull request, um build verificável para release, e deploy na devnet em tags. Me diga onde as chaves ficam e quais versões você fixou.
```

O arquivo dele carrega os fatos de CI da Solana que um pipeline genérico erra: **fixar a versão da toolchain** (Anchor 1.1.x com Solana CLI 3.1.10, Rust 1.89 ou mais novo no host, GLIBC 2.39 ou mais nova), condicionar merges a build mais testes, gerar o artefato de release com `solana-verify build` para que o programa on-chain possa ser verificado, instalar o Surfpool porque `anchor test` roda nele, e **só chaves de devnet entram no CI**. A upgrade authority de mainnet fica em uma hardware wallet ou em um multisig da Squads, e o CI pode escrever o buffer e entregá-lo ao multisig, que executa o upgrade. Ele fecha com `/audit-infra`, a auditoria de infraestrutura, e com `/doctor` quando a toolchain mudou, e relata pipelines, nomes de segredos sem valores, versões fixadas e a lista de alertas.

4. **Escreva a documentação que um estranho precisa.** Digite:

```text
Use o agente tech-docs-writer para escrever o README do Stamp para desenvolvedores de frontend e de indexador, a partir do código, da IDL e dos testes.
```

O arquivo dele diz o que a documentação de um programa precisa declarar: o **program id por cluster e a upgrade authority**, conferidos com `solana program show`, as contas de cada instrução com as flags de signer e writable, **seeds de PDA como bytes exatos** e se o bump é armazenado, layouts de conta com offsets porque indexadores filtram por eles, unidades de compute por instrução, e **erros por nome, decimal e hex**, para que `0x1770` seja encontrável. Ele tira os exemplos dos testes ou do cliente gerado e os roda antes de publicar.

5. **Deixe um estranho encontrar o app.** Digite:

```text
/product-review
```

Ele pergunta o produto, o usuário-alvo, a única ação central e o estágio, depois percorre o app como esse usuário e pontua oito dimensões com evidência: onboarding, loop central, estados vazios, estados de erro, performance, sinais de confiança, mobile, docs. Cada nota vem com a linha que a justificou, e o roadmap é agrupado por esforço. `--harsh` transforma isso em um teste de estresse com dez dimensões ponderadas e exatamente três correções. *Para o Stamp o ponto a observar é se o valor aparece antes de conectar a carteira*, e para um cartão fidelidade a resposta deve ser sim.

6. **Faça o commit e descreva a mudança.** `/quick-commit` formata, linta e faz o commit com uma mensagem convencional e sem trailer de atribuição, propondo antes uma branch com data quando você está na `main`. `/pr-description` escreve um corpo de pull request que um revisor consegue verificar, a partir do diff, do log e da saída de testes que de fato rodaram, *e deixa de fora qualquer número que ninguém rodou*.

## Está pronto quando

- `.program-id-devnet` guarda o id do Stamp e `solana program show` nele aponta você como upgrade authority.
- Existe um workflow de CI só com chaves de devnet e versões fixadas.
- O README declara o program id, as seeds como bytes e os códigos de erro.
- Você tem um scorecard com uma linha de evidência por dimensão.

## Fique atento

- **Para a mainnet, o relatório de pré-condições vem antes do prompt de aprovação.** O prompt é a segunda checagem.
- **Um deploy que falha deixa SOL em um buffer.** O comando sabe retomar ou recuperar. Nunca imprima o keypair nem a seed phrase sobre os quais ele pergunta.
- **Chaves de RPC públicas vão no bundle.** O arquivo do devops as roteia por um Worker ou por uma rota de servidor.
- **O firewall para o erro caro e pergunta.** Se o Stamp deveria estar na mainnet, isso continua sendo decisão sua.

## O que fica

Deploys **param para a sua aprovação e nomeiam o cluster**, a mainnet começa com um relatório de pré-condições, o CI guarda só chaves de devnet, e a documentação declara o id, a autoridade, as seeds e os códigos de erro a partir do código. *O firewall fica no deploy porque é lá que o dinheiro está.*

## A seguir

A próxima lição é sua: três caminhos de ideia montados como cadeias de agentes, do primeiro prompt de pesquisa até o resultado de um fim de semana, e um template para escrever o seu. Traga uma ideia, mesmo que ruim.
