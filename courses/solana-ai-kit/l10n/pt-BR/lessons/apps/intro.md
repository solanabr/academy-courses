# Construa em volta dele: engenheiros de frontend, mobile e backend

O Stamp tem um programa com três instruções e uma suíte de testes. Ninguém no caixa consegue usar ainda. Nesta lição três agentes constroem as partes que as pessoas tocam: **uma página web para o cartão do cliente, um app de celular e um serviço que dá um dashboard à cafeteria**. A parte web roda só com Node, então todo mundo acompanha no teclado. Uns vinte minutos.

## Por que isso importa

O programa é uma parte pequena de um produto, e o kit sabe disso: seis dos quinze agentes trabalham *em volta* do programa. Três deles estão nesta lição. **solana-frontend-engineer** constrói React e Next.js sobre `@solana/kit`, com conexão de carteira pelo Wallet Standard e uma lista fixa de regras de UX de transação. **mobile-engineer** constrói apps React Native e Expo sobre o Mobile Wallet Adapter. **rust-backend-engineer** constrói os serviços que leem a chain e mantêm um banco de dados honesto sob retentativas e forks. O arquivo de cada um carrega *os erros que hábitos antigos cometem*, e essas listas são o valor de verdade, porque são os bugs que você entregaria de outro jeito.

## Mão na massa

1. **Faça o scaffold do app web.** Digite dentro do Claude Code:

```text
/scaffold stamp-web, só frontend, com um template Kit que não tem programa
```

O comando roda `create-solana-dapp` com um template Kit, transforma a pasta em um repositório git e instala o kit nela com o mesmo instalador que você rodou na lição 1. O plugin de segurança pergunta nessa instalação via pipe, então espere uma pergunta. Abra o Claude Code lá quando terminar.

2. **Peça a página do cartão do cliente.** Digite:

```text
Use o agente solana-frontend-engineer. Construa a página do cliente para o Stamp: conectar uma carteira, mostrar o cartão do cliente para uma cafeteria com a contagem de carimbos e quantos carimbos faltam para um café grátis, e um botão de resgate. Leia o cartão da PDA de cartão do programa Stamp na devnet. Até o programa ser implantado, leia de um arquivo de fixture para a página funcionar hoje. Siga as suas regras de UX de transação para o fluxo de resgate e rode /build-app e /test-ts antes de devolver.
```

Espere que ele leia primeiro a referência de frontend no pack solana-dev, depois construa sobre **`@solana/kit` 8 com os plugins de carteira e RPC e `@solana/react`**, deixando de fora os pacotes legados de wallet-adapter. O fluxo de resgate deve seguir as regras do arquivo dele: **simular antes de pedir uma assinatura**, mostrar aguardando assinatura, depois enviada com a assinatura e um link do explorer, depois **confirmada**, e liberar a interface em confirmed, não em finalized. Uma assinatura recusada é um resultado normal, não um erro. Ele fecha rodando `/build-app` e `/test-ts` e reportando componentes alterados, carteiras testadas e lacunas de acessibilidade deixadas em aberto, porque a barra dele é **WCAG 2.2 AA**.

3. **Gere o cliente em vez de escrever chamadas à mão.** Quando o programa existir, digite:

```text
/generate-idl-client
```

Ele gera a IDL e renderiza um cliente TypeScript tipado com o Codama, para que a página chame `redeem` por uma função com tipos de verdade, e os erros do programa sejam decodificados em mensagens. *Instruções montadas à mão são o que este comando existe para substituir.*

![A faixa em volta do programa: o solana-frontend-engineer constrói a página web sobre o Kit com as regras de UX de transação, o mobile-engineer constrói o app Expo sobre o Mobile Wallet Adapter, e o rust-backend-engineer constrói o indexador de webhooks e a API do dashboard, todos lendo o mesmo cliente gerado e o design.md.](assets/v01-app-lane.webp)

4. **Descreva o app de celular, construa depois.** Digite:

```text
Use o agente mobile-engineer. Planeje o app de cliente do Stamp em Expo: conectar pelo Mobile Wallet Adapter, mostrar o cartão, resgatar. Me diga de qual pack de skills você precisa, o que funciona no iOS e o que precisa de um development build. Por agora, só o plano.
```

A resposta dele vai pedir o pack `solana-mobile` e te dizer três coisas do próprio arquivo que poupam uma tarde: **o Mobile Wallet Adapter é só Android**, então no iOS a assinatura passa pelo protocolo de deep link ou pelo SDK de uma carteira. Ele **precisa de um development build**, porque o Expo Go não consegue carregá-lo. E o polyfill de Web Crypto precisa carregar primeiro, ou a assinatura falha antes de a carteira abrir, de um jeito que parece bug da carteira.

5. **Dê um dashboard à cafeteria.** Digite:

```text
Use o agente rust-backend-engineer. Projete um serviço Axum pequeno para o Stamp: consumir webhooks da Helius para os eventos stamp e redeem do programa, manter uma tabela Postgres de carimbos por cliente por cafeteria, e servir à cafeteria um endpoint de dashboard. Me diga como você mantém a tabela correta quando um webhook é entregue de novo. Só o design.
```

A resposta deve conter as regras que o arquivo dele chama de parte difícil: **entregas se repetem**, então faça upsert pela assinatura mais a posição do evento na transação. **Transações que falharam também estão on-chain**, então confira o campo de erro antes de aplicar efeitos. **Confirmed basta para exibição, e finalized para qualquer coisa irreversível.** Guarde o slot em cada linha e faça o backfill pelo mesmo caminho idempotente. Ele vai pedir o pack `helius` para os webhooks, e a chave de que as chamadas desse pack precisam mora em `.env`, que o `/setup-mcp` define.

## Está pronto quando

- `stamp-web` compila e mostra um cartão a partir da fixture com uma carteira conectada na devnet.
- Você consegue repetir os três estágios de transação que o botão de resgate precisa mostrar.
- Você tem um plano escrito para o app mobile e para o indexador, cada um nomeando o pack ou a chave de que precisa.

## Fique atento

- **Valores públicos de env vão no bundle.** `NEXT_PUBLIC_*` e `EXPO_PUBLIC_*` ficam visíveis para todo usuário, então uma URL de RPC com chave vai atrás de uma rota de servidor.
- **Um blockhash dura uns 60 a 90 segundos.** Retransmitir a mesma transação assinada é seguro até então. Depois disso, monte de novo e peça uma nova assinatura.
- **Transação v1.** O cliente planeja v1 por padrão, e uma carteira que não a suporta precisa de um cliente v0 ou de um aviso para atualizar. O arquivo de frontend tem a verificação.
- **O cliente RPC bloqueante trava um runtime Tokio.** O arquivo de backend nomeia o não bloqueante. Um exemplo antigo não.

## O que fica

Três engenheiros constroem as partes que as pessoas tocam, cada um com **uma lista das armadilhas no próprio arquivo**: confirmed e não finalized na web, wallet adapter só Android no mobile, upserts idempotentes no indexador. *O cliente gerado é o que mantém os três falando com o mesmo programa.*

## A seguir

Na próxima lição os carimbos do Stamp viram um token, um cliente paga com qualquer moeda, e o cartão vira um jogo, o que significa o engenheiro de token, o engenheiro de DeFi e os dois agentes de jogo. Quase só leitura e prompts, sem precisar de toolchain.
