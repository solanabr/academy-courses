![Cartão da aula: Do toque no botão ao pagamento confirmado. Antes de pagar, confira. Antes de assinar, entenda. Depois de enviar, verifique. Pagamento pendente? Não aperte enviar de novo.](assets/card-aula-7.webp)

Você já fez um pagamento pelo celular e ficou esperando aparecer "concluído"? Na Solana também existe um caminho entre tocar no botão e ter o pagamento confirmado. Vamos acompanhar esse processo?

Então, o nosso exemplo é o seguinte: você estacionou o carro e precisa pagar 5 USDC, 5 dólares digitais, pelo aplicativo. O USDC, lembrando, é um token que busca acompanhar o dólar.

## Primeiro passo: conferir os dados

O primeiro passo é conferir de qual conta sairá o pagamento, qual endereço receberá, qual ativo você está enviando e em qual quantidade. No nosso exemplo, 5 USDC da sua conta para o estacionamento, usando a rede Solana. É como conferir uma maquininha antes de aproximar o cartão.

## Segundo passo: entender o pedido

O segundo passo é entender o pedido. Esse pedido é chamado de transação. Dentro dele, nós temos as ordens, chamadas instruções. Imagine um envelope com uma lista de tarefas: transfira o pagamento e registre o período de estacionamento comprado. Muito bem, então uma aplicação pode reunir essas tarefas na mesma transação.

## Terceiro passo: ler a prévia

O terceiro passo é ler a prévia. Antes de aprovar, a carteira pode mostrar quanto sairá da sua conta, para onde irá e quais permissões serão dadas. Ela também pode fazer uma simulação, um ensaio, para identificar problemas como falta de saldo. Pense na revisão de uma compra antes de fechar o pedido. É o que você faz lá no seu carrinho de compras.

## Quarto passo: assinar e enviar

O quarto passo é assinar e enviar. Quando você aprova, a carteira usa a sua chave privada, lembra dela?, para criar uma assinatura digital. É como um selo que comprova a sua autorização. A rede verifica esse selo sem precisar receber a sua chave privada. Portanto, você autoriza o pagamento sem entregar o seu segredo.

Depois, a transação é enviada. Mas atenção: enviada ainda não significa concluída.

## Quinto passo: acompanhar a confirmação

O quinto passo é acompanhar a confirmação. Você encontra três termos.

**Processed**, ou processada. O computador da rede processou a operação em um bloco, mas esse registro ainda pode mudar.

**Confirmed**, ou confirmada. O bloco recebeu votos suficientes dos validadores, considerando o peso de sua participação na rede.

**Finalized**, ou finalizada. O registro alcançou o nível mais forte de confirmação usado pela Solana.

Para lembrar: processamento inicial, confirmação coletiva e consolidação.

> **Nota editorial (2 de novembro de 2026).** O vídeo descreve os três nomes como eles funcionavam antes do Alpenglow, ativo na rede principal desde 21 de outubro de 2026. Os três continuam existindo nas ferramentas, mas, segundo a Solana Foundation, `confirmed` e `finalized` agora descrevem o mesmo estado, e `confirmed` deve ser removido em uma versão futura.

E confira duas coisas separadamente. O pagamento teve sucesso? O registro foi finalizado? Uma operação que falhou também pode ter seu registro de erro finalizado.

## Quem paga a taxa da rede?

Agora vamos entender as taxas. O estacionamento recebe USDC, lembra? Mas a taxa da rede é paga em SOL, por você ou por um serviço que pague a taxa em seu lugar. Existe uma taxa base relacionada às assinaturas exigidas.

## Por último: conferir o comprovante

Por último, confira o comprovante. Use a assinatura da transação como o número de protocolo no explorador da Solana. Confira destino, quantidade, taxa e resultado. Se a tela travar, consulte antes de pagar novamente. Uma transferência realizada não tem um botão geral de desfazer. Para um endereço novo, considere testar uma pequena quantia antes do restante.

## Confira, entenda e verifique

Nossa rotina é simples. Antes de pagar, confira. Antes de assinar, entenda. E, depois de enviar, verifique.

Última dica. Se o pagamento do estacionamento aparecer como pendente, não aperte "enviar" várias vezes. Cuidado. Primeiro, consulte a transação no explorador. Se você criar e autorizar um novo pagamento, pode acontecer de os dois serem concluídos.

E, quando fizer uma transferência de teste, confirme que o destinatário realmente recebeu antes de enviar o restante. Isso é importante, hein? O teste ajuda a conferir o caminho, mas não garante que um site ou uma pessoa seja confiável. Cuidado.

## Para guardar

Antes de pagar, confira. Antes de assinar, entenda. Depois de enviar, verifique. Enviada ainda não significa concluída, a rede verifica o selo sem receber a sua chave privada e, se o pagamento estiver pendente, não aperte "enviar" de novo.
