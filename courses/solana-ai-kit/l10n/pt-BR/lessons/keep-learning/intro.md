# Continue aprendendo: solana-guide, /explain-code e como se manter atualizado

Na lição passada você escreveu a sua própria cadeia de agentes. Quatorze dos quinze foram apresentados, e o décimo quinto é o das semanas depois do workshop, quando você está sozinho com um programa que entende pela metade. Esta lição é **o agente guia, o comando que explica código, o pesquisador como hábito, e como o kit se atualiza**, depois um quiz sobre o curso inteiro. Uns dez minutos.

## Por que isso importa

O kit vai construir mais rápido do que você aprende, e é nessa distância que eu ficaria de olho. **solana-guide** existe para essa distância. O arquivo dele diz que ensina para que *quem aprende consiga resolver o próximo problema sem ele*: confere o seu nível primeiro e pergunta em vez de supor, dá uma definição simples e o problema que o conceito resolve, depois um exemplo mínimo, depois o erro que iniciantes cometem, e confere se você consegue prever antes de seguir. Ele ensina **as ferramentas atuais** e avisa quando o tutorial que você trouxe é anterior a elas. Código de produção vai para os engenheiros, e essa separação é deliberada.

## Mão na massa

1. **Pergunte ao guia algo que você pulou.** Digite dentro do Claude Code:

```text
Use o agente solana-guide. Eu construí o Stamp com o kit e não entendo de verdade por que a conta do cartão é uma PDA em vez de uma conta normal. Eu sei JavaScript e um pouco de Rust.
```

Espere que ele confirme o seu nível, defina PDA em palavras simples com o problema que resolve, desenhe um diagrama Mermaid de cafeteria, cartão e cliente, mostre a derivação, nomeie o erro de iniciante, e então peça que você preveja algo, *o que acontece se alguém passar uma conta de cartão diferente*, antes de seguir. Se ele pular a pergunta, peça que ele a faça. Ao longo de várias sessões ele consegue percorrer uma trilha do currículo da incubadora no pack solana-new, conceito, código funcionando, exercício, revisão linha a linha, até você construir sem ajuda e ele entregar você aos engenheiros.

2. **Explique código que você não escreveu.** Digite:

```text
/explain-code programs/stamp/src/lib.rs
```

O comando ajusta a profundidade ao código, e um programa inteiro recebe o formato completo: uma visão geral em duas ou três frases, um **diagrama Mermaid** do fluxo de instruções ou das relações entre contas, um passo a passo pelas seções importantes com as razões por trás das escolhas não óbvias, os conceitos de Solana usados, **notas de segurança** sobre as checagens que o código faz e as que parece deixar passar, e para quem está aprendendo duas ou três perguntas de *o que acontece se*. Rode no resultado do anchor-engineer da lição 3 e você vai entender o seu próprio programa melhor do que quando ele foi escrito.

![A faixa de aprendizado: solana-guide confere o seu nível, define, desenha, mostra o erro e pede que você preveja, /explain-code dá uma visão geral, um diagrama, um passo a passo e notas de segurança, solana-researcher mantém versões e status atualizados, e /update, memsearch e /dream mantêm o kit e a memória dele atualizados.](assets/v01-learning-lane.webp)

3. **Faça do pesquisador um hábito.** Antes de qualquer decisão que dependa de uma versão, de um status de deploy ou do estado de um protocolo, pergunte ao pesquisador em vez de à sua memória ou à do Claude:

```text
Use o agente solana-researcher. Antes de eu construir em cima de <program>, confira a upgrade authority dele na mainnet, se o binário implantado é um build verificado, e o histórico de auditorias. Cite e date tudo.
```

Essa é a checagem pré-recomendação do próprio agente, e as três respostas são o que você quer antes de confiar o dinheiro de uma cafeteria a um programa.

4. **Mantenha o kit atualizado.** Digite:

```text
/update
```

Ele puxa a tag de release mais nova do kit, ou a `main` enquanto não existir tag mais nova que a v2.4.0, e sobrescreve os agentes, comandos, skills e scripts que o kit distribui, incluindo os que você editou. Ele **mantém o `.env`, as suas configurações, o `.mcp.json` e os arquivos de memória**, deixa o seu `CLAUDE.md` no lugar e escreve a versão do kit ao lado como `CLAUDE.md.upstream` sempre que os dois diferem, mantém as extensões que você instalou e reaplica o seu nível de firewall. Reinicie o Claude Code depois. Os agentes que você leu neste curso são arquivos do kit, então uma atualização pode mudá-los, e `/add-skill` sem id lista todos os packs, dizendo se cada um está instalado e quando instalar.

5. **Dê uma memória a ele.** Dois comandos e um reinício:

```text
/plugin marketplace add zilliztech/memsearch
```

```text
/plugin install memsearch
```

É um plugin, não um servidor MCP, com armazenamento local e embedder local, então sem chave e sem conta, ao custo de um download de modelo de uns 558 MB no primeiro uso. Depois de algumas sessões, arquivos markdown diários aparecem em `.memsearch/memory/`, e é ali que o program id que você implantou e a decisão de design que você rejeitou deixam de ser rederivados. `/dream` consolida um armazenamento diferente, o `MEMORY.md` e a seção Project Learnings do `CLAUDE.md`, quando ficam desatualizados, e pergunta antes de mexer no arquivo de instruções.

6. **Saiba onde ficam as respostas longas.** A pasta `docs/` do kit nunca é copiada para o seu projeto. `agents-and-commands.md` tem todos os quinze agentes e todos os 54 comandos, `firewall.md` os níveis e seus limites, `skill-packs.md` os packs com versão fixada, `configuration.md` os servidores MCP e plugins com o que cada um custa por sessão. Os packs de skill oficiais ficam hospedados como markdown em [kit.superteam.codes](https://kit.superteam.codes), onde qualquer agente pode lê-los sem o kit, e os arquivos do próprio kit são servidos em `aikit.superteam.codes/<path>`.

## Está pronto quando

- O guia pediu que você previsse algo antes de seguir.
- `/explain-code` deu a você um diagrama e uma nota de segurança sobre o seu próprio programa.
- O memsearch está instalado e o Claude Code foi reiniciado.

## Fique atento

- **Reinicie depois do memsearch e depois do `/update`.** Plugins ativam no reinício, e regras de permissão são lidas no início da sessão.
- **Os fatos deste curso têm data.** Contagens, versões e nomes de comando foram lidos em 2026-11-09 da branch `main` do kit no commit `0a2f0d9`, que é o que o instalador busca até uma release mais nova que a v2.4.0 receber uma tag. A tag v2.4.0 tem 32 comandos e nada da cadeia de workflow, então as lições sobre `/crew`, `/ship` e `/inspect-mint` descrevem a `main`. Quando o README do kit e esta lição discordarem, *o README está certo*.
- **O guia ensina, os engenheiros constroem.** Peça a cada um o que o próprio arquivo dele promete.

## O que fica

Um agente ensina no seu nível e faz você prever, um comando explica código com um diagrama e notas de segurança, e o pesquisador confere autoridade, build verificado e auditorias antes de você confiar em um programa. *O kit se atualiza e mantém os seus arquivos, e o plugin de memória evita que ele rederive o que você já decidiu.*

O quiz abaixo cobre o curso inteiro. Depois dele, construa algo pequeno com o kit esta semana, e conte para quem organizou o seu workshop qual agente surpreendeu você e qual decepcionou.
