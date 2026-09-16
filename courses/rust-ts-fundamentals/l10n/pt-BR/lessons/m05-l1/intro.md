# serde: o parser de fronteira

## Resumo

O m04-l3 completou o motor de Rust: a máquina ProbeState, erros tipados de ponta a ponta, a trava de cargo test, clippy e fmt verde ao lado do vitest no único pipeline. Mas a config do motor é hardcoded e as latências dela são fixtures. Hoje o problema da config morre. Você escreve um parser de config à mão, sente exatamente o que ele custa, e então apaga ele com uma linha de derive, porque o serde gera o parser a partir dos seus tipos do mesmo jeito que o zod inferiu os seus tipos a partir de um schema no M2. Mesmo arquivo em disco, as duas linguagens fazendo o parse dele, uma disciplina vestindo duas bandeiras. No caminho: enums com tag, a mini-linguagem de atributos, o seu primeiro pipeline de iterador. Arrumação da casa: a pegada de loop de completion do M4 acabou, de propósito. O M5 volta ao formato padrão: visão geral trabalhada comigo, apoios do lab afinando conforme você avança, o passo de pipeline seu, o challenge totalmente sem guia.

## Sinta a dor primeiro

Antes de o serde ganhar qualquer coisa, você paga o preço cheio. Aqui está um alvo de sonda como uma linha de config plana:

```text
name=api;url=https://example.com/health;timeout_ms=500
```

Faça o parse dele à mão, no `pulse-rs`, usando nada além de `split`, `match` e o encanamento de `Result` que você construiu em m04-l2. E faça com honestidade, porque a versão desonesta são dez minutos e uma mentira. Honesto quer dizer: separar os pares no `;`, separar cada par no PRIMEIRO `=` só (aquela URL está a uma query string de conter um `=`), recusar chaves desconhecidas como erros em vez de dar de ombros para elas, validar o scheme da url e os dígitos do timeout, e reportar chaves faltando numa ordem fixa. Toda falha volta como um valor `Err`, nunca um panic.

O painel de coding-challenge desta lição tem um starter, `kv-config-parser`, que compila e trapaceia em cada uma dessas regras; abra ele no editor do navegador e comece a deixar ele honesto agora. Ponha um timer, de verdade. O núcleo da versão honesta é assim:

```rust
let (key, value) = match pair.split_once('=') {
    Some((k, v)) => (k, v),
    None => return Err(format!("bad pair: {pair}")),
};
match key {
    "name" => name = Some(value.to_string()),
    "timeout_ms" => {
        let parsed = value
            .parse::<u64>()
            .map_err(|_| format!("invalid timeout_ms: {value}"))?;
        timeout_ms = Some(parsed);
    }
    other => return Err(format!("unknown key: {other}")),
}
```

E essa é só a metade por par. Cada campo também tem que ser rastreado como um `Option` ao longo do loop, porque "a chave nunca apareceu" é uma falha diferente de "a chave apareceu quebrada", e a spec quer as chaves faltando reportadas numa ordem fixa. O fim de jogo da versão honesta são três linhas da jogada `ok_or_else` de m04-l2:

```rust
let name = name.ok_or_else(|| "missing key: name".to_string())?;
let url = url.ok_or_else(|| "missing key: url".to_string())?;
let timeout_ms = timeout_ms.ok_or_else(|| "missing key: timeout_ms".to_string())?;
```

Vinte minutos, mais ou menos, de rastreio de `Option`, chamadas de `ok_or_else` e strings de erro. Para UM registro plano. Com TRÊS campos. Agora escale isso para uma config no nível da frota, um nome de frota mais um array de registros de alvo, o formato do m02-l2 que esta lição está prestes a ressuscitar (a história completa de para onde aquele arquivo foi mora algumas telas abaixo), e faça a conta de fazer o parse disso à mão. Esse número é o que esta lição apaga. Guarde o seu parser feito à mão, porém; ele volta como o challenge, e terminar ele é como você vai saber exatamente o que o derive te comprou.

## O compilador escreve o parser

Duas instalações, e repare que os dígitos carregam uma data:

```bash
cargo add serde@1.0.229 --features derive
cargo add serde_json@1.0.151
```

Essas versões foram checadas contra o crates.io em 2026-09-02; o serde vive na linha 1.x há anos, então o que quer que o `cargo add` resolva para você hoje está bom. Pequena história de guerra de quando a gente checou: o crates.io recusa chamadas de API que não mandam um header User-Agent. A nossa própria ferramenta de pesquisa bateu nisso durante a varredura de fatos deste curso, um curl pelado recebeu não-JSON de volta onde deveriam estar os dados de versão. A primeira coisa da qual você faz parse numa fronteira de verdade pode ser a página de erro de alguém. Parsing de fronteira existe porque fronteiras mentem, e isso inclui a fronteira que você consulta para aprender sobre parsers de fronteira.

A flag `--features derive` importa. O serde sem a feature derive compila numa boa como crate e aí falha na sua linha `#[derive(Deserialize)]` com um erro que aponta para o atributo, não para o `Cargo.toml`, que é exatamente o lugar errado para te mandar procurar. Se o seu primeiro build explodir no derive, cheque a lista de features antes de qualquer outra coisa.

Agora, apagar. Esta é a substituição inteira para o parser em cima do qual você acabou de suar, cobrindo o arquivo de config inteiro:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub fleet_name: String,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub url: String,
    pub interval_secs: u64,
    pub timeout_ms: u64,
}
```

Nenhum código de parsing. Você descreve o formato; a macro de derive gera o parser em tempo de compilação, checagens de campo, checagens de tipo, erros de chave faltando, tudo isso. `rename_all = "camelCase"` é a costura com o lado TypeScript: o arquivo em disco diz `intervalSecs` e `timeoutMs` porque a frota escreveu ele, os campos de Rust são `snake_case` porque o clippy tem opiniões, e um atributo traduz entre as duas convenções para que nenhuma das linguagens precise tapar o nariz.

Usar isso é uma chamada de função que retorna, e isto já deveria parecer familiar a esta altura, um `Result`:

```rust
pub fn parse_config(raw: &str) -> Result<Config, ProbeError> {
    serde_json::from_str(raw).map_err(ProbeError::BadConfig)
}
```

`serde_json::from_str` te dá `Result<Config, serde_json::Error>`, e a falha é um valor carregando o que deu errado e onde, até a linha e a coluna. Sem exceção, sem panic, nada que você já não tenha segurado antes. `map_err(ProbeError::BadConfig)` é o músculo do m04-l2 fazendo exatamente o trabalho dele, fazendo a ponte do erro estrangeiro para dentro da taxonomia do seu motor, e sim, esse é o nome da variante usado pelado como função, o truque que o clippy te ensinou quando sinalizou a closure redundante. O que quer dizer que um empréstimo acabou de vencer: o m04-l2 estacionou `#[allow(clippy::redundant_closure)]` no `parse_fixture_line` com uma validade escrita dizendo "reduza a closure no M5, apague este allow." É o M5. Abra o `engine.rs`, mude aquela linha para `.map_err(ProbeError::BadFixture)?`, apague o atributo e o comentário dele, e deixe o `cargo clippy` confirmar que a dívida está quitada. O `ProbeError` ganha uma variante para receber a falha de config:

```rust
#[error("config rejected: {0}")]
BadConfig(serde_json::Error),
```

![Um arquivo de config faz o parse ou para uma config Ok que alimenta o loop de sonda ou para um valor de erro carregando linha e coluna que o chamador trata.](assets/v01-flowchart.webp)

Uma nota de resista-ao-reflexo antes de a gente seguir. O ponto inteiro do m04-l2 era que uma falha de fronteira é um valor que você roteia, então `from_str(...).unwrap()` na fronteira da config seria gastar duas lições de disciplina para economizar nove caracteres. O caso de config malformada não é excepcional. É terça-feira. Ele ganha uma variante, uma mensagem e uma decisão, como todo o resto.

### Mesmo arquivo, os dois parsers, uma tela

Aqui está a parte que eu estava esperando para te mostrar desde o M2, com um pedaço da história da estação contado sem rodeios primeiro. Em m02-l2 você projetou uma config no nível da frota, `fleetName` mais um array de alvos nomeados, e um schema zod para travar isso. Aí o treino de burst do m02-l3 sobrescreveu os dois: o `packages/pulse-fleet/pulse.config.json` vivo da frota tem sido o formato de burst desde então (`targets` como uma lista plana de URLs, `timeoutMs`, `concurrency`, `retry`), e o `configSchema` dele tem formato de burst e é estrutural para o `fleet.ts` e a suíte de testes. O formato no nível da frota não sobreviveu em disco. Hoje ele volta de propósito, num endereço novo: a raiz do repo da estação, como a config da qual o arco de Rust faz parse daqui para a frente, enquanto a config de burst fica exatamente onde está. Dois arquivos, dois trabalhos, divididos com honestidade: a config de burst diz quais URLs martelar neste instante; a config raiz diz quais alvos a estação observa para sempre. Este é o arquivo que o lab te faz criar:

```json
{
  "fleetName": "pulse-prod",
  "targets": [
    {
      "name": "docs",
      "url": "https://example.com",
      "intervalSecs": 60,
      "timeoutMs": 3000
    },
    {
      "name": "api",
      "url": "https://example.org/health",
      "intervalSecs": 30,
      "timeoutMs": 2000
    }
  ]
}
```

E em m02-l2 você escreveu o schema para ele. O lab reconstrói aquele schema num módulo novo, `src/root-config.ts`, ao lado do schema de burst em vez de por cima dele, para que a frota consiga assinar este arquivo também:

```ts
export const rootTargetSchema = z.strictObject({
  name: z.string().min(1),
  url: z.url(),
  intervalSecs: z.number().int().positive(),
  timeoutMs: z.number().int().positive(),
});

export const rootConfigSchema = z.strictObject({
  fleetName: z.string().min(1),
  targets: z.array(rootTargetSchema).min(1),
});

export type RootConfig = z.infer<typeof rootConfigSchema>;
```

Ponha isso ao lado da struct `Target` acima e leia as duas devagar. zod: você escreveu um schema, um valor em tempo de execução que caminha pela entrada, e o `z.infer` derivou o tipo estático A PARTIR do schema. serde: você escreveu um tipo, e o derive gerou o parser A PARTIR do tipo. Mesmo arquivo. Mesma recusa na porta. Mesmo "depois desta linha, os dados são o formato sobre o qual eu raciocinei." A direção é oposta e a disciplina é idêntica: parse, don't validate, cruze a fronteira uma vez para dentro de um tipo que não consegue representar o lixo, e deixe o resto do programa confiar nele.

A síntese, e não é metáfora: derive(Deserialize) é o zod rodando em tempo de compilação. O zod paga pela flexibilidade dele com um objeto de schema em tempo de execução caminhando pelos seus dados; o serde paga pela velocidade dele com uma expansão de macro que você não consegue ajustar em tempo de execução. Por baixo, uma ideia só.

Por que este conceito carrega tanto peso em web3 especificamente: no lado Rust deste ecossistema, quase tudo que cruza uma fronteira de processo cruza ela através do serde. Quando a pesquisa deste curso levantou cinco repos de produção adjacentes a Solana, agave, yellowstone-grpc, photon, jito-relayer e carbon, o serde e o serde_json estavam em cada um dos manifests, cinco de cinco, no mesmo tier universal que o tokio, o thiserror, o anyhow e o clap. Requisições e respostas de RPC, arquivos de config, payloads de webhook, o JSON em que um arquivo de keypair é guardado: tudo isso entra no Rust tipado através da maquinaria que você está aprendendo agora mesmo. Este não é um capítulo que você está degustando. É infraestrutura estrutural para o resto da sua vida em Rust.

![O schema de TypeScript infere um tipo enquanto o tipo de Rust deriva um parser, e os dois consomem o mesmo arquivo de config em disco.](assets/v02-diagram.webp)

Essa disciplina tem uma história que vale trinta segundos. O zod 3.0.0 foi entregue em 2021-05-17; a versão 4 só foi para GA em 2025-07-09. Quatro anos num major só, e nessa janela "parse, don't validate" cresceu de slogan de post de blog para a cultura de fronteira padrão de um ecossistema inteiro. A ideia nunca foi específica de TypeScript. O Rust só impõe ela com mais força, porque aqui não existe saída de emergência em formato de `any` para contrabandear dados sem parse por cima da fronteira; o único caminho até um `Config` é através do parser que o compilador escreveu.

![Uma linha do tempo do release do zod 3 em 2021 até o GA da versão 4 em 2025, terminando no derive de Rust desta lição carregando a mesma disciplina.](assets/v03-timeline.webp)

### Enums com tag: tornando um kind ruim irrepresentável

A config está prestes a crescer, porque configs de verdade sempre crescem. Agora mesmo todo alvo é uma sonda HTTP, mas a lista de alvos de uma frota de monitoramento nunca fica num formato só por muito tempo: uma checagem de socket simples precisa de um host e uma porta onde uma sonda HTTP precisa de uma url. Kinds diferentes, campos diferentes, um conjunto fechado. Em m04-l3 você modelou exatamente este formato em memória: um conjunto fechado de variantes, cada uma carregando os próprios dados. A pergunta é como isso fica quando tem que sobreviver a uma viagem pelo JSON, e a resposta é um campo discriminante, a mesma jogada do `"kind"` que a sua união `ProbeResult` usa no lado TS desde o M2:

```rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ProbeKind {
    Http { url: String },
    Tcp { host: String, port: u16 },
}
```

`tag = "kind"` é a representação com tag interna: o serde lê o campo `"kind"` primeiro, despacha para exatamente uma variante, e faz o parse dos campos restantes contra o formato daquela variante. `rename_all = "lowercase"` mapeia `Http` para `"http"` em disco. Uma entrada de config que diz `"kind": "grpc"` falha com um erro nomeado listando as variantes legais, e você vai ler esse erro exato no lab. A struct `Target` absorve o enum inline:

```rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub interval_secs: u64,
    pub timeout_ms: u64,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(flatten)]
    pub kind: ProbeKind,
}

fn default_enabled() -> bool {
    true
}
```

Dois atributos novos, os dois de uso diário. `#[serde(flatten)]` emenda os campos do enum dentro do objeto JSON do alvo em vez de aninhar eles um nível abaixo, então o arquivo fica plano e editável por humanos. `#[serde(default = "default_enabled")]` torna `enabled` opcional no arquivo com um default explícito, que é a gentileza de migração que deixa toda entrada existente continuar funcionando quando um campo chega; rigor onde ele te protege, leniência onde você optou por ela, campo a campo.

![Uma entrada JSON de sonda de sete linhas com cada chave anotada para o campo de struct ou variante de enum em que ela é desserializada.](assets/v04-annotated-code.webp)

Você pode ter reparado que toda linha de derive nesta lição também diz `Serialize`. Essa é a passagem de volta, e não é decoração. Os mesmos atributos dirigem as duas direções, então um `Target` serializa de volta para o JSON exato, plano, em camelCase e com tag de kind do qual ele foi parseado:

```rust
let json = serde_json::to_string_pretty(&target)?;
```

```json
{
  "name": "rpc",
  "intervalSecs": 30,
  "timeoutMs": 1000,
  "enabled": false,
  "kind": "tcp",
  "host": "127.0.0.1",
  "port": 8899
}
```

Um conjunto de atributos, duas direções, zero deriva entre o que você lê e o que você escreve. Hoje o motor só lê; no próximo módulo, o poller de longa duração ganha um endpoint `/status` que tem que EMITIR JSON, e este derive é a razão de isso te custar uma chamada de função em vez de uma sessão de templating.

Agora a cilada, mostrada uma vez para você reconhecer ela no mundo real. Apague a tag e o serde ainda te oferece uma saída:

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LooseKind {
    Http { url: String },
    Tcp { host: String, port: u16 },
}
```

Untagged quer dizer que o serde tenta cada variante em ordem e fica com a primeira que serve. Dê lixo para ele e o erro degrada para `data did not match any variant of untagged enum LooseKind`, sem linha, sem campo, sem pista. Pior é o que acontece quando os dados servem para mais de um formato: um objeto carregando tanto uma `url` quanto um `host` faz o parse alegremente como `Http` e larga o resto no chão, primeira correspondência vence, em silêncio. Untagged é para consumir formatos que você não controla e não consegue consertar. No momento em que você é dono do formato, e você é dono deste, ponha a tag nele.

![Uma comparação mostrando que enums com tag falham com um erro de variante nomeada enquanto enums untagged adivinham pela primeira correspondência e produzem erros vagos.](assets/v05-comparison.webp)

O custo honesto de tudo isso, porque existe um. A linguagem de atributos, `tag`, `rename_all`, `default`, `flatten`, é uma mini-DSL, e a garantia em tempo de compilação cobre os seus TIPOS, não a grafia dos seus atributos. Escreva errado um valor de tag ou aponte o `rename_all` para o lado errado e o código compila limpo, e aí falha em tempo de execução. Tente: apague a linha `rename_all` do `Config` e rode contra o arquivo da frota. Build verde, e aí o parse morre com ``missing field `fleet_name` ``, nomeando um campo que está BEM ALI no arquivo, a uma convenção de nomenclatura de distância. Os atributos são configuração, não código que o compilador checa contra os seus dados, e um parser derivado é um contrato com o formato exato da config: evolução de schema vira uma falha de parse que você projeta de propósito. As regras de projeto são curtas: campos aditivos ganham `#[serde(default)]` para que arquivos com deploy continuem fazendo parse, conjuntos fechados ganham enums com tag para que uma variante nova seja um erro nomeado e barulhento, e renomeações são mudanças que quebram e que você agenda. Você vai sentir tudo isso no momento em que o arquivo velho encontrar o schema novo. Mais uma nota de honestidade: o serde ignora campos desconhecidos por padrão para formatos autodescritivos como JSON, mais frouxo que o seu `strictObject` do zod; `deny_unknown_fields` existe mas, segundo a documentação do próprio serde, não combina com `flatten`. Trade-offs até o fim; saiba qual postura cada fronteira tem.

### Seu primeiro pipeline de iterador

Conceito just-in-time, e o último de que esta lição precisa. O `Vec<Target>` parseado é matéria-prima; o motor quer uma lista de sondas. Só os alvos habilitados, mapeados para o formato do motor:

```rust
#[derive(Debug)]
pub struct ProbeTarget {
    pub name: String,
    pub endpoint: String,
    pub budget_ms: u64,
}
```

E esta é a transformação, a sua primeira cadeia de iteradores:

```rust
let probes: Vec<ProbeTarget> = config
    .targets
    .iter()
    .filter(|t| t.enabled)
    .map(|t| t.to_probe_target())
    .collect();
```

Você já escreveu este programa antes. `targets.filter(t => t.enabled).map(toProbeTarget)` está na frota desde o M2, e as closures lá dentro são as closures de `map_err` de m04-l2 com trabalhos diferentes. Leia como a mesma cadeia de array vestindo uma bandeira de Rust e você tem 90% disso. Os 10% restantes são uma palavra: lazy. `iter`, `filter` e `map` não fazem trabalho nenhum; eles constroem uma descrição de trabalho, e nada roda até um consumidor como o `collect` puxar os elementos através dela. É por isso que cadeias de Rust não alocam uma coleção intermediária por passo do jeito que métodos de array encadeados do JS fazem, e essa é toda a teoria de laziness de que você precisa hoje; o tratamento completo mora na caixa de vá-mais-fundo abaixo.

![Adaptadores de iterador formam uma descrição lazy do trabalho e o consumidor collect puxa cada elemento pela cadeia inteira numa única passagem.](assets/v06-flowchart.webp)

`collect` é um consumidor entre vários, e trocar o consumidor muda a pergunta que a mesma cadeia responde. Dois que você vai usar esta semana, direto da config da frota:

```rust
let enabled = config.targets.iter().filter(|t| t.enabled).count();
let slowest: u64 = config.targets.iter().map(|t| t.timeout_ms).max().unwrap_or(0);
```

`count` responde "quantos sobrevivem ao filtro", `max` responde "qual é o maior orçamento", e os dois drenam a cadeia sem construir coleção nenhuma. (Aquele `unwrap_or` não é um deslize: `max` retorna um `Option` porque uma lista vazia não tem máximo, e zero é uma resposta defensável para ela, a regra do m04-l2 sobre defaults que você consegue defender num comentário.)

Dois footguns antes do lab. `collect` precisa de um tipo de destino, porque ele consegue construir um Vec, um HashMap, uma String e mais a partir da mesma cadeia; deixe o tipo de fora e o compilador te para com o E0282, `type annotations needed`. Anote o binding como acima, ou use o turbofish, `collect::<Vec<_>>()`. Esse erro é o clássico rito de passagem do primeiro pipeline, e agora ele é informação, não obstrução. Segundo: `iter()` pega emprestado. As suas closures veem `&Target`, e é por isso que `to_probe_target` recebe `&self` e clona as strings que ele guarda, com as regras do m04-l1 mandando ali em silêncio.

**Vá mais fundo (os 20%).** esta lição ensinou o derive, a representação com tag e os quatro atributos que você vai de fato digitar este ano. O resto do serde, impls de `Deserialize` customizados, o modelo de dados, desserialização zero-copy, todo atributo, mora em [https://serde.rs/](https://serde.rs/), o livro do próprio crate; a página de enum-representations dele é o mapa canônico de com tag versus untagged e os dois estilos que a gente pulou. Para closures e iteradores com a história completa de laziness e performance, o capítulo do Book é [https://doc.rust-lang.org/book/ch13-00-functional-features.html](https://doc.rust-lang.org/book/ch13-00-functional-features.html), e a sacada de lá vale a viagem: iteradores compilam para o mesmo código que o loop feito à mão. As duas URLs checadas ao vivo em 2026-09-02. O lab não precisa de nada do material de bookmark.

## Lab: um arquivo, dois parsers

Numerado, apoios afinando conforme você desce. Os passos 1 e 2 a gente faz junto, o passo 3 te dá assinaturas, o passo 4 é um treino que você roda sozinho.

1. **Crie a config raiz, depois faça o parse dela (trabalhado).** Duas jogadas. Primeiro o arquivo em si: na raiz do repo da estação, um nível acima de `pulse-rs/` (a trava do m04-l3 moveu `pulse-rs/` para dentro do repo da estação), crie `pulse.config.json` com exatamente o conteúdo de dois alvos impresso na visão geral, `fleetName: "pulse-prod"`, `docs` e `api`. Este é um arquivo novo num endereço novo, a config no nível da frota que a seção de teoria ressuscitou; nada em `packages/pulse-fleet` move ou muda. Depois o lado Rust: em `pulse-rs`, crie `src/config.rs` com as primeiras structs `Config` e `Target` da visão geral, os formatos v1 com `url` direto no `Target`, mais o `parse_config` e a variante `BadConfig` adicionada ao `ProbeError` em `src/engine.rs`. Não copie nada para dentro de `pulse-rs`: a partir de `pulse-rs/`, um symlink mantém funcionando a leitura relativa ao cwd abaixo: `ln -s ../pulse.config.json pulse.config.json` (ou simplesmente leia `"../pulse.config.json"` direto; o ponto é um arquivo, não duas cópias). main temporária, e repare que ela inlina a mesma chamada de serde que o `parse_config` embrulha; isso é de propósito, o treino de fronteira do passo 4 é onde o `parse_config` assume, então a função que você acabou de escrever fica brevemente sem uso em vez de fora do lugar:

   ```rust
   mod config;
   mod engine;

   use engine::ProbeError;
   use std::fs;

   fn main() -> anyhow::Result<()> {
       let raw = fs::read_to_string("pulse.config.json")?;
       let config: config::Config =
           serde_json::from_str(&raw).map_err(ProbeError::BadConfig)?;

       println!("fleet \"{}\": {} target(s)", config.fleet_name, config.targets.len());
       for t in &config.targets {
           println!(
               "  {} -> {} every {}s, timeout {}ms",
               t.name, t.url, t.interval_secs, t.timeout_ms
           );
       }
       Ok(())
   }
   ```

   Uma expectativa ajustada antes de você rodar: esta main temporária orfana a superfície inteira do motor do m04, então o build chega vestindo um muro de avisos de dead-code, umas duas dúzias deles, cada um durando uma lição. Esperado, e temporário: a divisão de workspace da próxima lição põe o motor de volta num crate consumido. Só não dê push até lá, porque o CI da estação roda `cargo clippy -- -D warnings` e contaria cada um deles como um erro. `cargo run` deve imprimir:

   ```text
   fleet "pulse-prod": 2 target(s)
     docs -> https://example.com every 60s, timeout 3000ms
     api -> https://example.org/health every 30s, timeout 2000ms
   ```

   Agora dê a caneta ao lado TS. O `configSchema` de burst não consegue assinar este arquivo (formato errado, e ele é estrutural para o `fleet.ts` e os testes, então fica intocado); a config raiz ganha o próprio módulo. Crie `packages/pulse-fleet/src/root-config.ts` com o `rootTargetSchema`, o `rootConfigSchema` e o `RootConfig` da visão geral (mais `import { z } from "zod";` no topo), e um verificador de quatro linhas ao lado dele, `src/check-root-config.ts`, a jogada do m02-l2 repetida:

   ```ts
   import { readFileSync } from "node:fs";
   import { parseOrExit } from "./config.js";
   import { rootConfigSchema, type RootConfig } from "./root-config.js";

   const path = process.argv[2] ?? "../../pulse.config.json";
   const raw: unknown = JSON.parse(readFileSync(path, "utf8"));
   const config: RootConfig = parseOrExit(rootConfigSchema, raw);
   console.log(`fleet "${config.fleetName}": ${config.targets.length} target(s)`);
   ```

   Rode a partir de `packages/pulse-fleet` com `npx tsx src/check-root-config.ts ../../pulse.config.json`, e deixe o momento acontecer: duas linguagens, dois sistemas de tipos, um arquivo, as duas fronteiras recusando lixo. Checkpoint: os dois comandos verdes nos mesmos bytes.

2. **Evolua o contrato (guiado).** Troque pelo enum `ProbeKind` com tag e pelo `Target` evoluído da visão geral, com o `flatten` e o default de `enabled`. O compilador reclama antes de o serde ter a vez dele: a `main` temporária ainda imprime `t.url`, e `url` agora mora dentro da variante `Http`. Mude aquele println para mostrar `t.kind` com `{:?}` em vez da url, e refaça o build. Agora `cargo run` de novo, contra o arquivo NÃO ALTERADO, e leia a sua primeira falha de evolução de schema:

   ```text
   Error: config rejected: missing field `kind` at line 9 column 5
   ```

   (O prefixo `Error:` é do anyhow, imprimindo a falha que o seu `?` devolveu para fora da `main`.)

   O parser derivado é um contrato com o formato, e você acabou de mudar o contrato sem avisar o arquivo. Então avise o arquivo: adicione `"kind": "http"` aos dois alvos existentes, e adicione um terceiro alvo que a frota TS nunca viu:

   ```json
   {
     "kind": "tcp",
     "name": "rpc",
     "host": "127.0.0.1",
     "port": 8899,
     "intervalSecs": 30,
     "timeoutMs": 1000,
     "enabled": false
   }
   ```

   Ele já vem com `"enabled": false` porque nenhum braço da estação consegue rodar uma checagem de socket ainda, e uma config que nomeia uma sonda que ninguém consegue rodar deveria dizer isso. (Aquela porta é onde um validador local de Solana responde RPC, uma porta na qual a gente bate muito mais adiante no curso.) Quebre de propósito antes de consertar: ponha `"kind": "grpc"` naquela entrada e rode:

   ```text
   Error: config rejected: unknown variant `grpc`, expected `http` or `tcp` at line 26 column 5
   ```

   Uma recusa nomeada listando as variantes legais. Compare isso com o erro untagged na visão geral e você tem o argumento inteiro de com tag versus untagged em duas linhas de saída de terminal. Conserte o kind de volta. Um parser ainda reclama, porém: o `check-root-config.ts` agora recusa o arquivo, `Unrecognized key: "kind"`, e ele está CERTO em recusar, essa é a rigidez do `strictObject` que você pediu fazendo o trabalho dela num contrato evoluído. Os dois signatários re-assinam ou ninguém entrega. Em `src/root-config.ts`, atualize o `rootTargetSchema` para uma união discriminada, o formato que você já conhece do `ProbeResult`:

   ```ts
   const baseFields = {
     name: z.string().min(1),
     intervalSecs: z.number().int().positive(),
     timeoutMs: z.number().int().positive(),
     enabled: z.boolean().default(true),
   };

   export const rootTargetSchema = z.discriminatedUnion("kind", [
     z.strictObject({ kind: z.literal("http"), url: z.url(), ...baseFields }),
     z.strictObject({
       kind: z.literal("tcp"),
       host: z.string().min(1),
       port: z.number().int().min(1).max(65535),
       ...baseFields,
     }),
   ]);
   ```

   `z.discriminatedUnion("kind"...)` e `#[serde(tag = "kind")]` são a mesma máquina em lados opostos do arquivo. Checkpoint: `cargo run` imprime config no valor de três alvos, e o `check-root-config.ts` aceita o mesmo arquivo de novo.

![Evoluir a config compartilhada quebra o parser de Rust, depois o parser do zod, até os dois schemas re-assinarem o contrato novo e ficarem verdes.](assets/v07-flowchart.webp)

3. **O pipeline (seu).** Conecte o `ProbeTarget` e a cadeia filter-map-collect da visão geral à `main`, e então alimente o motor que você construiu em m04-l3 com o resultado. Uma aposentadoria primeiro: o `engine.rs` ainda exporta o `ProbeTarget { name, url }` esquelético do m04-l1, e o `ProbeTarget { name, endpoint, budget_ms }` do módulo de config é a substituição adulta dele. Dois tipos pub com um nome só num crate é deriva esperando para acontecer, então apague o do m04 do `engine.rs` agora; nada chama ele desde a main temporária do passo 1, e o `cargo check` vai confirmar. O tipo novo e o `Target::to_probe_target` pertencem a `src/config.rs`, ao lado das structs das quais eles derivam, que é exatamente onde a lista de re-export da próxima lição espera eles. Aí o pipeline: para cada sonda, dirija a máquina de estados sobre latências de fixture com o `timeout_ms` do próprio alvo como orçamento. As assinaturas de closure são o seu apoio, os corpos e a fiação não são: `filter` recebe `|t: &&Target| -> bool` (referência dupla, `iter` empresta e `filter` empresta de novo; `t.enabled` simplesmente funciona através das duas), `map` recebe `|t: &Target| -> ProbeTarget`. Escreva `to_probe_target(&self)` como um `match` no kind: `Http` entrega a url como endpoint, `Tcp` formata `host:port`. A minha `main` acaba assim; escreva a sua antes de comparar:

   ```rust
   let probes: Vec<ProbeTarget> = config
       .targets
       .iter()
       .filter(|t| t.enabled)
       .map(|t| t.to_probe_target())
       .collect();

   println!(
       "fleet \"{}\": probing {} of {} targets",
       config.fleet_name,
       probes.len(),
       config.targets.len()
   );

   for probe in &probes {
       let mut source = FixtureSource::new(vec![212, 487, 2400, 2600]);
       let state = drive(&mut source, probe.budget_ms);
       println!(
           "  {} -> {} settles {:?} (budget {} ms, fixture latencies)",
           probe.name, probe.endpoint, state, probe.budget_ms
       );
   }
   ```

   Saída do Checkpoint, vale ler com atenção:

   ```text
   fleet "pulse-prod": probing 2 of 3 targets
     docs -> https://example.com settles Up (budget 3000 ms, fixture latencies)
     api -> https://example.org/health settles Degraded (budget 2000 ms, fixture latencies)
   ```

   Dois de três: o filtro largou o alvo TCP desabilitado, então o pipeline é estrutural, não decoração. E as mesmas quatro latências de fixture assentam de formas diferentes sob orçamentos diferentes, 2400 e 2600 ms passam tranquilas sob o orçamento de 3000 do docs e estouram os 2000 da api duas vezes, que é a máquina do m04-l3 consumindo config de verdade pela primeira vez. Diga os limites em voz alta: as latências ainda são fixtures, o motor ainda não sonda nada, e o braço HTTP de verdade está a duas lições daqui, em m05-l3.

4. **O treino de fronteira (sozinho).** Copie a config para `pulse.config.broken.json` e ponha o `timeoutMs` da entrada api em `"fast"`. Ponha a cópia onde a leitura abaixo vai encontrar ela, o que depende de qual das duas fiações do passo 1 você pegou: se você fez o symlink, a cópia quebrada vai em `pulse-rs/` ao lado dele; se você está lendo `"../pulse.config.json"` direto, a cópia vai na raiz do repo e o caminho no trecho vira `"../pulse.config.broken.json"`. Erre isso e o `?` te dá um NotFound do `anyhow` antes de a mensagem de recusa que este treino existe para mostrar chegar a imprimir. Então faça a `main` carregar a cópia quebrada DEPOIS da real, reportando a falha sem morrer:

   ```rust
   let broken = fs::read_to_string("pulse.config.broken.json")?;
   if let Err(e) = parse_config(&broken) {
       println!("broken copy refused: {e}");
   }

   println!("run complete");
   ```

   Cauda esperada da execução:

   ```text
   broken copy refused: config rejected: invalid type: string "fast", expected u64 at line 16 column 25
   run complete
   ```

   A linha e a coluna vão bater com onde quer que o seu editor tenha posto aquele campo; o que importa é que elas estão LÁ, num valor de erro que você roteou, impresso por uma execução que então seguiu em frente. Reconhece o formato? É o treino de config quebrada do m02-l2, vestindo a outra bandeira. **Verifique antes de seguir em frente**: `cargo run` imprime o resumo de frota de três alvos com dois sondados, a recusa da cópia quebrada com linha e coluna, e `run complete`. Esse é o contrato inteiro desta lição numa tela só.

## Challenge

Agora vá terminar o que a abertura começou: `kv-config-parser`, totalmente sem guia, no painel de coding-challenge. O parser do starter é um mentiroso: ele separa em todo `=`, engole chaves desconhecidas, inventa defaults onde deveria recusar. Deixe ele honesto: separação no primeiro `=` via `split_once`, uma allowlist de chaves, validação de scheme de url e de timeout, chaves faltando reportadas na ordem fixa name, url, timeout_ms, toda falha um `Err`, nenhum unwrap no caminho de parse. Oito testes avaliam isso, incluindo a URL com query string que pune separação preguiçosa e o ponto e vírgula final que pune iteração preguiçosa. Os hints escalam de `split_once` até o fim de jogo do `ok_or_else` de chave faltando; gaste eles em ordem. Este é deliberadamente o último parser feito à mão que você escreve neste curso, que é exatamente por que vale a pena escrever ele bem: depois dele, você sabe linha por linha o que todo derive futuro faz por você.

## Checkpoint

O que você consegue fazer agora, concretamente: transformar uma definição de struct num parser de JSON com um derive e ler os quatro atributos que carregam o uso diário do serde; modelar um formato discriminado como um enum com tag e explicar, com duas mensagens de erro como evidência, por que com tag ganha de untagged em formatos dos quais você é dono; rotear uma falha de parse pela sua taxonomia de erros como um valor com linha e coluna anexadas; e transformar uma lista parseada com filter, map e collect, sabendo que nada roda até o consumidor puxar. Uma nota de interface para o seu eu do futuro: `parse_config(&str) -> Result<Config, ProbeError>` e `Target::to_probe_target` agora fazem parte da superfície pública do motor. A próxima lição move eles para dentro de um crate de biblioteca, e lições posteriores chamam eles por exatamente estas assinaturas, então resista à vontade de "arrumar" eles daqui até lá.

A recuperação de 30 segundos antes de você fechar a aba, em voz alta: o zod infere o quê a partir do schema, e o serde deriva o quê a partir do tipo? (O tipo; o parser. Direções opostas, uma disciplina.) E que palavra única explica por que a sua cadeia não fez trabalho nenhum antes do `collect`? (Lazy.)

Relatório de atrito, enquanto está fresco: a falha de evolução de schema no passo 2 do lab pareceu a lição quebrando ou a lição acertando? Essa batida é projetada para arder, a ideia de contrato-com-um-formato não gruda sem ela, mas existe uma versão dela que lê só como retrabalho, e o seu relatório é como eu descubro qual delas foi entregue. O mesmo para a referência dupla no `filter`; se `|t: &&Target|` te custou mais de um minuto, diga.

O seu motor agora lê a config de verdade da frota através de um parser que você não precisou escrever, e tudo mora num crate só, o que está prestes a virar o problema. A CLI que você faz crescer em seguida precisa do motor como biblioteca, e um binário de release não deveria arrastar fixtures de teste junto. Próxima lição: workspaces do cargo, editions, features e ler os pins de versão de outras pessoas, o Cargo.toml como uma negociação com toda máquina que um dia vai buildar o seu código. Traga o seu manifest; cada linha dele está prestes a significar alguma coisa.
