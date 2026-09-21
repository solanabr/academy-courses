# O que significa ganhar na Colosseum

Na lição passada você leu o registro: duas temporadas de side tracks e os quatro hackathons em que a Unruggable entrou antes de ganhar um, lidos no card dela em colosseum.com/hackathon. Nesta lição você escreve o arquivo que diz *quanto esse registro vale para você*.

## Por que isso importa

A Colosseum revisa o seu repositório, e a página do hackathon, na temporada World's Fair de 2026, diz com todas as letras que a revisão não olha linguagem, framework, padrões nem qualidade de código. *Leia isso duas vezes.* A mesma página diz que os hackathons da Colosseum são competições de startups. É essa a única ideia desta lição: **os critérios de julgamento dizem exatamente o que é pontuado**, e o que é pontuado é uma empresa, não o código.

Sempre dou o exemplo de um hackathon presencial que ganhei em Dubai, com uma ideia que eu achava simples demais para se classificar e sem nenhuma demo planejada. Ela batia exatamente com o que os organizadores procuravam, o pitch vendeu, e *nada disso tinha a ver com o código*. Sorte não é um plano que dá para passar para um colega de time. O brief é.

## Mão na massa

1. Abra colosseum.com/hackathon, encontre a seção de julgamento e **copie os nomes dos sete fatores** para um arquivo novo, colosseum-brief.md, exatamente como estão escritos, com a data na primeira linha. Lido em **2026-09-06**:

```text
Critérios de julgamento da Colosseum, colosseum.com/hackathon, lido em 2026-09-06
Founder + Market Fit
Insight
Product + Execution
Potential Market Size
Founder Communication
Viability
Traction
```

Se a página ao vivo mostrar nomes diferentes, **vale a página**, e anote o que mudou. Repare em Viability (viabilidade): *se as suas anotações têm outra palavra nesse lugar, elas são de outra temporada*. A lista também muda de página para página: a página Eternal da Colosseum, lida no mesmo dia, traz **seis fatores, sem Traction** (tração). Copie a lista da página do hackathon em que você vai entrar.

2. Embaixo de cada nome, **escreva a pergunta que a própria página faz**, nas suas palavras, curta.

Founder + Market Fit (encaixe entre fundador e mercado) pergunta se o time tem as habilidades e a experiência certas e por que está motivado. Insight (a sacada) pede uma percepção única, ou uma tecnologia ou tendência nova. Product + Execution (produto e execução) pergunta se o produto funciona bem, como ele se compara com a concorrência e **com que velocidade o time entrega**. Potential Market Size (tamanho potencial do mercado) pergunta o tamanho do TAM, o mercado total que o produto poderia atender, e se ele é grande *ou pequeno mas crescendo rápido*. Founder Communication (comunicação dos fundadores) pergunta se os fundadores comunicam a visão com clareza. Viability pergunta se isso pode virar um negócio escalável e sustentável. Traction pergunta se o produto já tem demanda ou receita, e se isso dura.

Conte quantas dessas perguntas um jurado responde abrindo o seu código: **uma**, e só em parte.

3. **Copie a revisão de repositório** embaixo dos fatores, as duas metades:

```text
revisão de repositório, colosseum.com/hackathon, lida em 2026-09-06
procura:       trabalho significativo dentro da janela do hackathon
               trabalho feito pelo time, não por terceiros
               funcionalidades priorizadas com estratégia
não olha:      linguagem, framework, padrões, qualidade de código
```

Priorizadas com estratégia diz exatamente o que o seu recorte precisa mostrar: eles querem ver que você **deixou coisas de fora de propósito**, e *um histórico de commits mostra isso com mais honestidade do que qualquer deck*.

4. Registre **como a nota é dada**. A página, lida em 2026-09-06, diz que a submissão passa por várias rodadas internas de avaliação, que uma lista curta segue para a banca de jurados, que um grupo menor é chamado para uma entrevista de 15 minutos no Zoom, e que os vencedores saem **cerca de um mês** depois do prazo de submissão. Ela também diz qual entrega é aberta primeiro: o vídeo de apresentação de dois a três minutos é, nas palavras dela, 'one of the first resources judges review' (um dos primeiros recursos que os jurados revisam). Escreva as duas coisas no brief, e *diga quem do time fica disponível no mês depois do prazo*.

![Uma submissão à Colosseum chega aos jurados primeiro pelo vídeo, passa por rodadas internas e uma lista curta sem o time, depois uma entrevista de 15 minutos, com os vencedores anunciados um mês depois.](assets/v01-flowchart.webp)

5. **Distribua os sete fatores** entre as quatro perguntas que um jurado precisa responder, e escreva as quatro embaixo dos fatores, uma linha cada:

```text
o problema é real:        Insight, Potential Market Size, Viability (perguntado com dinheiro junto)
o produto funciona:       Product + Execution (a concorrência e a velocidade de entrega são a mesma pergunta ao longo do tempo)
a história convence:      Founder Communication (respostas escritas, vídeo, entrevista de 15 minutos)
esse time aguenta:        Founder + Market Fit, a revisão de repositório (trabalho feito pelo time), a entrevista
```

Traction não é uma pergunta nova. São as duas primeiras respondidas com **evidência em vez de argumento**: alguém já usa, então o problema é real, e o uso se sustenta, então o produto funciona. Viability e Traction são as duas linhas que uma competição de startups acrescenta. Ao lado de cada pergunta, anote o que o seu projeto já consegue dizer e **escreva "em aberto" onde não consegue**. A linha de Viability do Fiado diz "quem paga pelo fiado: ainda em aberto", e a linha de Traction é uma meta, "cinco mercadinhos rodando fiados reais até o prazo, e quantos voltam", nunca uma afirmação, *porque número inventado no dia 0 é pior do que espaço em branco*.

**Traction é o fator que os times mais subestimam**, e é o que um mês ainda consegue mover. Jurado nenhum dá valor à tecnologia se ninguém usa o produto construído em cima dela. Traction é **gente usando o produto**, gente contada, e não espera a mainnet: mercadinhos rodando fiados de verdade enquanto os pagamentos liquidam na devnet é tração, *e um deploy na mainnet que ninguém abre não é*. O curso vai montando essa contagem desde a semana 1.

![Os sete fatores da Colosseum se distribuem em quatro perguntas, com Traction alimentando duas delas como evidência e Viability e Traction marcados como os acréscimos da competição de startups.](assets/v02-flowchart.webp)

6. **Copie o prazo e converta** com uma ferramenta. A página, lida em 2026-09-06, diz que o hackathon World's Fair vai de 14 de setembro a **12 de outubro de 2026**, e que o prazo de submissão é a data de encerramento da temporada. Ela mostra a abertura como timestamp e o fim só como data, então o brief guarda a data, *um lembrete para reler a hora na página ao vivo na última semana*, e a linha convertida para o horário local assim que ela existir. Pratique com a string que a página mostra, o momento da abertura:

```bash
python3 -c "from datetime import datetime; print(datetime.fromisoformat('2026-09-14T11:00Z').astimezone())"
```

Isso imprime o horário no fuso da sua máquina, com o offset junto. **Python 3.11 ou mais novo** entende o Z do final sozinho, em versão mais antiga troque o Z por +00:00, e confira o comportamento na documentação do datetime da sua versão. Quando a hora do prazo aparecer na página, cole no lugar dessa string e guarde o resultado com o original em UTC ao lado, *para um colega em outro fuso poder refazer a conta*.

![A temporada World's Fair abre em 2026-09-14T11:00Z e vai até 12 de outubro de 2026, cuja hora ainda precisa ser relida e convertida, com os vencedores anunciados cerca de um mês depois.](assets/v03-timeline.webp)

7. **Cite as linhas que desclassificam**, tiradas da página:

```text
desclassifica, colosseum.com/hackathon, lido em 2026-09-06
uma submissão de produto por time, e portanto uma por pessoa: ninguém entra com um segundo projeto por fora
o líder do time completa a submissão antes do prazo: o brief diz quem ocupa esse papel
deturpar o histórico de desenvolvimento, ou não declarar código que já existia: desclassifica, bane, revoga um prêmio
```

A página acrescenta que violar o Código de Conduta também desclassifica, uma quarta linha que você copia sem comentar. Se você constrói com um time de agentes e reaproveita código antigo seu, **a terceira linha é com você**: anote o que já existia antes de a janela abrir e declare isso na submissão, *porque um histórico de commits que começa na véspera da temporada parece exatamente o que a página proíbe*.

8. **Dedique quinze minutos à página de outro hackathon** e escreva três linhas: o que ela pede, em qual das quatro perguntas isso cai, e o que ela desclassifica. Um hackathon sazonal ou uma side track tem página própria, com entregas e prazo próprios. Leia essa página no dia em que decidir entrar, e reaproveite o material de submissão que você já tem. Seja lá o que essa página pedir, **marque com uma das quatro perguntas**, *porque uma página mais curta nunca faz uma quinta*. O formato, em que só o que está entre colchetes angulares você substitui:

```text
<hackathon>, <url>, lido em <data>
pede:          <os critérios como nomeados na página>, cada um marcado com uma das quatro perguntas
desclassifica: <as próprias linhas da página>
```

**Não é uma tabela**: três linhas bastam para decidir se entra e o que adaptar.

## Está pronto quando

- Cada fator em **colosseum-brief.md** é uma citação da página ao vivo e tem a data.
- Cada uma das **quatro perguntas** aponta para um ou mais fatores, com uma nota ou um "em aberto" embaixo de cada.
- O prazo está no seu fuso horário **com o offset escrito**, e uma segunda ferramenta (o relógio do celular vale) dá a mesma resposta.
- O que os jurados abrem primeiro, a etapa da entrevista e **as linhas que desclassificam** estão no arquivo, com data.
- Os quinze minutos na página de outro hackathon renderam **três linhas, não uma tabela**.

## Fique atento

- A Colosseum aceita builders de qualquer ecossistema blockchain, com tracks de prêmio por ecossistema, e a Accelerator exige alguma integração com Solana, então **Solana é uma track em que você entra**, e o motivo de o seu projeto estar em Solana vai para a linha de Insight.
- Datas, prêmios e tracks mudam a cada temporada, e os fatores só se mantiveram até agora, então releia a página na última semana: *meu palpite, de quem observa times e não de quem contou*, é que um prazo **convertido uma vez e nunca relido** perde mais hackathons do que uma demo ruim.
- Os sete fatores da Colosseum cobrem tudo que uma página mais curta pede, mas Viability e Traction custam ao builder solo do Fiado quase uma semana do mês conversando com donas de mercadinho. Uma temporada da Colosseum pontua essa semana duas vezes, em Traction e de novo em Founder Communication quando a história abre o vídeo, e uma página de ensaio sem essa linha **não pontua isso em lugar nenhum**, *então escolha de propósito e escreva o porquê*.

## O que fica

A Colosseum pontua uma empresa, não o código: os sete fatores cabem em quatro perguntas, **o problema é real, o produto funciona, a história convence, esse time aguenta**, e Viability e Traction fazem duas delas com dinheiro e evidência na mesa. O código mora em uma linha das sete, e o jurado conhece o seu projeto primeiro por um vídeo de dois a três minutos, sem você na sala. *Cada entrega que você fizer daqui em diante passa por essas quatro perguntas antes de seguir.*

## Próxima lição: uma frase antes do relógio

Na próxima lição o relógio ainda não começou e você já tem um pitch. **Uma frase, escrita no dia 0**, antes de existir cartão do time ou plano do mês. *Você vai odiar essa frase, e a ideia é essa mesmo*, porque as quatro perguntas que você acabou de distribuir são as que essa frase precisa aguentar.
