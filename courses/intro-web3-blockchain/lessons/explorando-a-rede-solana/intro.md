![Cartão da aula: Explorando a rede Solana. Mainnet, a cidade em operação. Devnet, a oficina dos aplicativos. Testnet, o campo de testes da infraestrutura. Para começar, basta navegar.](assets/card-aula-2.webp)

O professor continua na cidade inteligente: ao chegar numa cidade nova, primeiro você observa, depois experimenta os aplicativos e só então paga. Na Solana é igual: para começar, basta navegar, sem instalar carteira nem colocar dinheiro na rede.

## Três ambientes

Com o exemplo de um aplicativo de estacionamento a céu aberto, ele apresenta:

- **Mainnet:** a rede principal, a cidade em operação. As aplicações funcionam para valer, os ativos podem ter valor econômico e os pagamentos são feitos.
- **Devnet:** a oficina em que a equipe experimenta o aplicativo antes de oferecê-lo aos motoristas. Usa token de teste, sem valor econômico: ter 10 SOL de teste não significa ter 10 SOL para gastar na rede principal. É como receber créditos de demonstração.
- **Testnet:** o campo de testes da infraestrutura, da própria rede e dos validadores, os computadores que verificam as operações e chegam a um acordo sobre os registros.

## O Solana Explorer

Para observar o movimento da cidade, usa-se um explorador de blocos como o Solana Explorer, um painel público de consulta dos dados on-chain, sem conectar carteira. Os termos da tela:

- **Endereço:** identifica uma conta na rede.
- **Saldo:** a quantidade de determinado ativo.
- **Slot:** a janela de tempo em que um validador tem a oportunidade de produzir um bloco.
- **Bloco:** reúne registros de transações.
- **Assinatura:** funciona, para consulta, como um número de protocolo.
- **Status:** faça duas perguntas, se a operação deu certo e se o registro já foi confirmado pela rede. Uma operação pode estar finalizada e ter dado erro: nesse caso, o registro da falha foi consolidado.

## Experimente

Abra o [Solana Explorer](https://explorer.solana.com/?cluster=devnet), selecione Devnet e pesquise a assinatura desta transferência simples de teste, conferida na Devnet em 2 de novembro de 2026:

`33R4sDVSKdCdLHNJzBGLRKtAKormi4WuiomRjPCbpBx5dALFutNGsxvpZoEUjF8VqvWYRorkNBMdvuDgpEtbmfJu`

Se a Devnet não mostrar mais esse histórico, pesquise no mesmo Explorer o endereço `9B5XszUGdMaxCZ7uSQhPzdks5ZQSmWxrmzCSvtJ6Ns6g`, abra o histórico de transações dele (Transaction History) e escolha uma transferência simples da lista.

Identifique origem, destino, quantidade, taxa, slot e status, e observe as alterações de saldo: quem paga a taxa pode perder um pouco mais do que o valor enviado.
