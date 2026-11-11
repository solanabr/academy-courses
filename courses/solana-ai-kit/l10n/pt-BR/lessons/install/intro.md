# Instale o kit e conheça o time

No fim do workshop você terá entregado um projeto Solana para **os especialistas do kit**, um de cada vez, da primeira pergunta de pesquisa até o deploy. Esta lição instala o kit que carrega esses especialistas e mostra como um deles é construído. Uns quinze minutos.

## Por que isso importa

O Solana AI Kit é uma pasta de configuração para o Claude Code, mantida pela Superteam Brasil em [github.com/solanabr/ai-kit](https://github.com/solanabr/ai-kit). Como lido em 2026-11-09 na branch `main` do kit, no commit `0a2f0d9`, ele carrega 15 agentes, 54 slash commands, um hub de skills, quatro servidores MCP e um firewall. O instalador busca a `main` até que uma release mais nova que a v2.4.0 seja marcada, e a tag v2.4.0 em si tem 32 comandos e nada da cadeia de workflow da lição 7, *então espere que os números deste curso mudem*. Este curso é sobre os agentes. **Um agente é um prompt especialista**: um arquivo markdown com um nome, uma descrição de uma linha, um modelo, as referências que ele precisa ler antes de trabalhar, os erros que não pode cometer e uma tabela de para quem ele passa o trabalho. O Claude Code inicia um agente como subagente quando o seu pedido bate com a descrição dele, ou quando você diz o nome. *Dizer o nome é o hábito que este curso ensina.*

## Mão na massa

1. **Abra o Claude Code na pasta onde você guarda projetos.** Não no seu diretório home, e não dentro de outro repositório, porque o próximo passo cria a pasta do projeto aqui.

```bash
claude
```

2. **Cole este prompt.** Ele cria a pasta do projeto, lê o instalador, roda o instalador de dentro dessa pasta e confere o que chegou. Aprove cada comando conforme aparecer, e leia o resumo do instalador antes de dizer sim para rodá-lo.

```text
Monte um projeto de workshop do Solana AI Kit para mim. Mostre cada comando antes de rodar, e pare se algum passo falhar.
1. Crie uma pasta chamada kit-workshop dentro do diretório atual e inicialize um repositório git vazio nela.
2. Baixe https://aikit.superteam.codes/install.sh para um arquivo temporário, leia o arquivo e me diga em poucas linhas o que ele escreve, o que ele adiciona ao .gitignore e qual plugin ele instala.
3. Rode esse arquivo com bash de dentro de kit-workshop, para que o kit seja instalado nessa pasta e em nenhum outro lugar, e me mostre a caixa de encerramento que ele imprime.
4. Liste kit-workshop/.claude/agents e me diga quantos arquivos há nela.
5. Não inicie o Claude Code você mesmo. Me diga para sair desta sessão, entrar em kit-workshop com cd e rodar claude lá.
```

A caixa de encerramento diz *Installation complete*, nomeia o nível do firewall (**Relaxed**, o nível relaxado) e lista os próximos passos. O que chegou: `.claude/`, `CLAUDE.md`, `.mcp.json`, `.env` e `.env.schema` na pasta, com `.claude/`, `.mcp.json`, `.gitmodules` e `.safe-ai-skill/` adicionados ao `.gitignore` e `CLAUDE.md` e `.env.schema` deixados rastreados, mais o plugin de segurança **safe-ai-skill** instalado pela CLI `claude`. O instalador não pergunta nada. Se preferir digitar você mesmo, os mesmos cinco passos são `mkdir` e `git init`, `curl -fsSL` desse endereço para um arquivo, `less` no arquivo, depois `bash` nele de dentro da pasta.

3. **Reinicie dentro do projeto e aprove os servidores.** Saia da sessão, depois:

```bash
cd kit-workshop && claude
```

O Claude Code pergunta uma vez se deve iniciar os quatro servidores MCP de `.mcp.json`. Aprove. Depois digite:

```text
/doctor
```

Treze verificações numeradas, cada uma OK, WARN, FAIL ou n/a, com um comando de correção por falha. O Claude Code traz um `/doctor` próprio, então se o que voltar não forem treze verificações numeradas, escolha o comando do projeto no menu de slash, aquele que lê `.claude/commands/doctor.md`. Hoje a linha da Solana CLI pode mostrar FAIL e as chaves podem aparecer vazias. *As duas coisas estão bem para a próxima lição.* A lição 3 é onde Rust, Anchor e a Solana CLI entram. O `/doctor` já imprimiu o comando de instalação da Solana CLI, e imprime os de Rust e Anchor quando o projeto tiver um workspace Anchor.

4. **Abra um arquivo de agente do começo ao fim.** Abra `.claude/agents/solana-architect.md` no seu editor, ou peça dentro do Claude Code:

```text
Me mostre .claude/agents/solana-architect.md na íntegra, sem alterações.
```

Leia como uma descrição de cargo. O cabeçalho traz o nome, a descrição com que o Claude Code faz a correspondência, e **`model: opus`**, que todo agente do kit define, seja qual for o modelo em que a sua sessão roda. Depois, as seções que a maioria dos quinze arquivos compartilha: **o que ler antes de trabalhar** (links para o pack solana-dev e para as skills do próprio kit), **os detalhes fáceis de errar** (aqui: seeds são permanentes, contas graváveis compartilhadas serializam toda transação, poderes de admin atrás de um multisig), **Handoffs** (qual agente recebe qual continuação, uma tabela neste arquivo e uma lista nos outros), e uma seção de fechamento, **Output** no arquiteto e no pesquisador, **Before handing back** nos engenheiros. Essa seção de fechamento é a que vale guardar: *ela diz o que você tem a receber quando o agente termina*.

5. **Conheça os outros catorze em cinco faixas.** Eles se agrupam pela etapa do projeto que atendem:

![Os quinze agentes em cinco faixas: design com solana-architect, game-architect e solana-researcher, programas com anchor-engineer e pinocchio-engineer, em volta do programa com solana-frontend-engineer, mobile-engineer, rust-backend-engineer, token-engineer, defi-engineer e unity-engineer, entrega com solana-qa-engineer, devops-engineer e tech-docs-writer, e aprendizado com solana-guide.](assets/v01-five-lanes.webp)

As lições seguem essas faixas com **um projeto de exemplo**: *Stamp*, um cartão fidelidade de uma cafeteria, em que o cliente ganha um carimbo por compra e dez carimbos valem um café. A cafeteria e o cliente têm uma carteira cada. É uma ideia pequena de propósito, para que cada agente tenha a sua vez em algo que você consegue visualizar.

6. **Diga o nome.** Digite isto dentro do Claude Code e observe a transcrição:

```text
Use o agente solana-researcher para me dizer, em três frases com uma fonte citada em cada uma, o que é o Token-2022 e quando uma loja pequena iria preferi-lo ao programa de token clássico.
```

Você deve ver o agente iniciar, depois uma resposta curta com links e um rótulo de confiança por achado, que é o formato de saída que o arquivo dele exige. A resposta termina com até cinco linhas rotuladas, **Verified**, **Decisions made**, **Decisions needed**, **Assumptions** e **Next owner**, porque as regras da casa pedem isso de todo agente e um hook manda o agente de volta uma vez quando falta alguma. Você vai ver essas cinco linhas depois de cada agente neste curso. Se o Claude responder de memória, sem agente e sem links, diga "use o agente" e pergunte de novo.

## Está pronto quando

- `ls .claude/agents` lista quinze arquivos e o `/doctor` rodou.
- Você consegue nomear as seções que um arquivo de agente carrega, e a de fechamento, que diz o que você tem a receber.
- O pesquisador iniciou no seu prompt e citou as fontes.

## Fique atento

- **O instalador instala na pasta de onde é rodado.** Por isso o prompt roda ele dentro de kit-workshop. Ele não tem ideia de qual pasta você queria.
- **Opus custa Opus.** Todo agente fixa o Opus, seja qual for o modelo da sua sessão. Para uma pergunta rápida, pergunte à sessão principal, e para design, código e pesquisa deixe o agente rodar.
- **Windows nativo.** O plugin de segurança não tem build do engine lá e o instalador pula ele. WSL2 funciona.
- **Node 22.5 ou mais novo.** O `/doctor` aprova o Node 18, mas o servidor context-mode, entre os quatro padrão, quer 22.5 ou mais novo, então essa é a versão para ter.
- **`.env` guarda chaves, `.mcp.json` guarda configuração.** O servidor da Helius lê a chave do ambiente, e o `/setup-mcp` adiciona a chave para você. Nada de hoje precisa de chave.

## O que fica

O kit instala na pasta de onde o instalador é rodado, e os agentes dele são **descrições de cargo em markdown** com um modelo, referências, armadilhas, handoffs e um formato de saída obrigatório. *Você recebe o especialista que pede pelo nome.*

## A seguir

Na próxima lição o pesquisador e o arquiteto levam o Stamp de três linhas de ideia a um design com contas nomeadas, e você guarda o arquivo de design, porque todo agente depois dele lê esse arquivo.
