# Un blink para el drop: actions que se ejecutan donde sea (que las renderice)

La lección pasada, `pos-stall` llevó el mismo endpoint de transaction request al otro lado de una mesa con un código QR. Ese endpoint ya vendió un disco a través de una página y a través de un puesto. Hoy vende a través de un link que puedes pegar donde sea, y nos ponemos honestos sobre qué quiere decir "donde sea".

Antes de construir nada nuevo, demuestra que el núcleo todavía contesta. Arranca tu servidor checkout-txreq de la lección de transaction requests, con la misma línea `MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts` que corriste allá, y golpea su POST directo (escucha en el 3100 en `/txreq`; sustituye tu propio puerto y ruta si los moviste):

```bash
curl -s -X POST http://localhost:3100/txreq \
  -H 'Content-Type: application/json' \
  -d '{"account":"'$(solana address)'"}'
```

Deberías recibir de vuelta un JSON con una transacción en base64 adentro. Mira esa respuesta un segundo. Una billetera hace POST de `{account}`, tu servidor le pone precio al pedido y devuelve una transacción lista para firmar. Ese es todo el truco de esta lección: el spec de Actions es ese mismo par solicitud-respuesta, más una capa de metadatos para que cualquier superficie pueda renderizar un botón a su alrededor. Ya construiste la parte difícil.

## Resumen

Esta lección convierte tu núcleo de pagos en un link. No un link a una tienda: un link que ES la tienda. Los hallazgos por delante, porque algunos de ellos no son lo que prometió el marketing de 2024:

- Entregas **drop-blink**: un `actions.json` en la raíz de tu dominio, un endpoint GET que devuelve los metadatos de la action y un endpoint POST que devuelve un `ActionPostResponse` conforme al spec. La transacción de adentro sale del constructor exacto que escribiste en la lección de transaction requests. Cero código de pagos nuevo.
- Tres reglas de hosting deciden si una billetera va a cargar tu action alguna vez: `actions.json` se sienta en la raíz del dominio, toda ruta de action manda `Access-Control-Allow-Origin: *` y `actions.json` mismo también. Sáltate una sola y te llevas la falla clásica de "funciona en curl, muerta en una billetera".
- El tooling está congelado: `@dialectlabs/blinks` 0.22.5 (publicado el 2025-04-04) y `@solana/actions` 1.6.6 (publicado el 2024-11-05) siguen siendo las versiones más nuevas que existen al 2026-08-22. Este curso no instala ninguno de los dos; construyes contra el contrato de cable vía `@solana/actions-spec` 2.4.2, y si alguna vez adoptas los SDKs, fija esas versiones exactas con una nota de obsolescencia.
- Dónde se renderizan los blinks en realidad en 2026 es incierto. El renderizado en X está mediado por una extensión de Chrome, no es nativo. Así que el lab pone como barrera la conformidad con el spec más un cliente local, y tus afirmaciones de alcance van en una caja de verificado-al-escribir, no en un pitch deck.

![El único constructor de transaction request de la lección de checkout alimenta tres superficies, la página de checkout, el puesto del POS y ahora el drop blink.](assets/v01-diagram.webp)

## El protocolo de actions, de cerca

### Del link de pago al protocolo

Si alguna vez entregaste algo con Stripe, hiciste un Payment Link: una URL que codifica "vende esta cosa", que los servidores de Stripe convierten en una página de checkout alojada. Una Solana Action es esa idea con el renderizado separado. Tu servidor describe el checkout (los metadatos) y construye la transacción (el POST que ya tienes). Cualquier superficie que muestre el link, una billetera, un feed, un cliente de chat, un sitio intersticial, es libre de renderizar su propio botón de compra a partir de tus metadatos y ejecutar la compra ahí mismo. Un **blink** (blockchain link) es la forma renderizada: la URL más cualquier cliente que la despliegue en UI.

El protocolo son dos verbos sobre una URL, más un archivo de descubrimiento:

1. **GET** a la URL de la action: devuelve metadatos. Icon, title, description, label y, opcionalmente, una lista de sub-actions parametrizadas. Esto es todo lo que un cliente necesita para dibujar el botón.
2. **POST** de `{account}` a la misma URL: devuelve un `ActionPostResponse` que lleva una transacción codificada en base64 para que la firme ese usuario específico. El mismo contrato que tu transaction request, y eso no es coincidencia: el spec de Actions generaliza el flujo de transaction request de Solana Pay que ya implementaste.
3. **`actions.json`** en la raíz de tu dominio: les dice a los clientes qué rutas de tu dominio son actions, para que un link pelado a tu sitio pueda mapearse a su endpoint de action.

![Flujo desde pegar un blink pasando por el descubrimiento con actions.json, el GET de metadatos, el botón renderizado, el POST con account, la transacción firmada y el paso de agradecimiento encadenado con links.next, con CORS y actions.json como barreras de falla.](assets/v02-flowchart.webp)

¿Por qué importa esto para una tienda de discos? Distribución. Cada checkout hasta ahora exigía que el cliente viniera a ti: tu página, tu puesto. Un blink lo invierte. La tienda viaja hasta donde ya está la conversación. Para un prensado limitado de 200 copias, la diferencia entre "entra a nuestro sitio" y "cómpralo aquí mismo" es conversión que puedes sentir. Ese fue el pitch de 2024, y el pitch era bueno. Guarda ese pensamiento, porque la sección de la realidad de 2026 más abajo es donde le ponemos precio honestamente.

### GET: los metadatos que se vuelven una UI

Esta es la forma que un GET tiene que devolver, espejada de `@solana/actions-spec` 2.4.2 (el paquete de tipos; es la fuente de verdad del spec y puedes importar estos en vez de escribirlos):

```ts
// drop-blink/src/types.ts
// Mirrored from @solana/actions-spec 2.4.2, trimmed to the fields this lesson
// exercises; the spec package carries more (optional `type` on the top-level
// action, an `error` field, parameter pattern/min/max, a wider LinkedActionType
// union). Diff against node_modules and you will find those extras on the spec
// side — deliberate omissions, not drift. The package is frozen alongside the
// rest of the tooling; these shapes are the live contract blink clients check.

export interface ActionParameter {
  name: string;      // the template variable this fills, e.g. {qty}
  label?: string;    // placeholder text the client shows
  required?: boolean;
  type?:
    | 'text' | 'number' | 'email' | 'url' | 'date'
    | 'datetime-local' | 'textarea' | 'checkbox' | 'radio' | 'select';
  options?: Array<{ label: string; value: string; selected?: boolean }>;
}

export interface LinkedAction {
  type: 'transaction'; // this action's POST returns a transaction to sign
  href: string;        // relative or absolute; may carry {param} templates
  label: string;       // the button text
  parameters?: ActionParameter[];
}

export interface ActionGetResponse {
  type: 'action';
  icon: string;        // absolute URL, not a path; clients will not resolve relatives
  title: string;
  description: string;
  label: string;       // fallback button text when links.actions is absent
  disabled?: boolean;
  links?: { actions: LinkedAction[] };
}
```

Recórrelo campo por campo, porque cada campo es UI estructural. `icon` tiene que ser una URL **absoluta**; una ruta relativa se renderiza como una imagen rota en todos los clientes, y es, de lejos, el bug cosmético más común de las actions ya entregadas. `label` es el fallback de un solo botón; `links.actions` lo reemplaza con varios botones cuando está presente. Y `parameters` es cómo a un botón le crece un input: un `href` de `/api/actions/drop?qty={qty}` más un parámetro llamado `qty` le dice al cliente que renderice un campo numérico y sustituya el valor en la URL antes de hacer POST. El `type` de un parámetro es una pista de renderizado (`select` y `radio` llevan `options`); los clientes que no reconocen un tipo caen de vuelta a texto. La validación de la entrada se queda en tu servidor. Siempre. Los tipos de parámetro le dan estilo a un formulario, no te protegen de lo que llega en el query string.

Después está `disabled`, el campo que un blink de comercio de verdad ejercita. Los metadatos se piden en vivo en cada render, lo que le da a un blink una propiedad que ninguna página de tienda estática tiene: el link se actualiza solo en todos los lugares donde alguna vez se publicó. Cuando se vende la copia 200 del prensado, tu GET empieza a devolver `disabled: true` con una description reescrita que dice agotado, y cada tarjeta que ya está sentada en cada publicación vieja pone su botón en gris en el próximo render. Nadie edita un tuit, nadie sale a cazar un link vencido. Agotado se vuelve un estado que tu endpoint reporta, no un 404 que esperas que la gente note. Para un drop, ese solo booleano es la mitad del argumento del protocolo entero: el marketing de escasez funciona exactamente cuando el artefacto que dice "se acabó" es el mismo artefacto que decía "compra".

### POST: {account} adentro, transacción afuera

El lado del POST ya lo conoces. El body es `{account}`, la clave pública en base58 del cliente. El POST de transaction request devuelve una transacción codificada en base64, y el spec de Actions envuelve ese mismo payload en un sobre con nombre:

![Tres formas JSON de ActionPostResponse, la mínima solo con type y transaction, una que agrega message, una que agrega links.next, anotadas con la regla de que las claves opcionales se omiten por completo cuando no están.](assets/v03-annotated-code.webp)

Dos agregados encima de tu endpoint existente. `message` es un string humano opcional que la billetera puede mostrar después de firmar, tu confirmación de pedido en miniatura. `links.next` es **el encadenamiento de actions**: un objeto `{ type: 'post', href }` que le dice al cliente "después de que esta transacción confirme, haz POST acá para el próximo paso". El POST encadenado incluye la firma confirmada, lo que lo vuelve el lugar natural para una tarjeta de agradecimiento, un paso para reclamar o la próxima action en un flujo de varios pasos. Lo vamos a usar para una pantalla de gracias que le hace eco al comprobante.

Una función más del spec que conviene conocer por su nombre: **action identity**. Un proveedor de actions puede adjuntar una instrucción de SPL Memo con la forma `solana-action:<identity>:<reference>:<signature>`, donde la identity es un keypair que firma la reference. Existe para que los indexadores y los registros puedan atribuir transacciones on-chain de vuelta a la action que las produjo. No la necesitas para el lab, y tu constructor ya estampa el memo del pedido y la clave de reference que usa tu propia conciliación. Archívala bajo "qué es ese memo raro cuando ves uno en un explorador".

¿De dónde sale la transacción misma? De `buildOrderTransaction`, sin cambios. Esta es la acreción hacia la que ha venido trepando todo el módulo, así que déjame decirlo sin adornos: el blink agrega una capa de metadatos y un sobre de respuesta. El precio, el memo, la clave de reference, la transferencia segura en decimales, todo eso es el camino de código de la lección de transaction requests, importado. Si te encuentras reescribiendo el ensamblado de la transacción adentro de un handler de blink, para; estás bifurcando tu lógica de pagos en una segunda copia que va a derivar.

### Descubrimiento y CORS: las dos reglas que matan blinks

Ahora la parte que genera más hilos de soporte. Tus endpoints pueden estar perfectos y ninguna billetera los va a cargar nunca, porque los blinks se cargan **cross-origin**. El cliente que renderiza tu blink vive en el dominio de otra persona, así que el navegador hace cumplir CORS en cada solicitud al tuyo, y el descubrimiento pasa por un archivo que tal vez te olvidaste de servir.

Regla uno: `actions.json` vive en la raíz del dominio. `https://shop.example/actions.json`, no `/api/actions.json`, no detrás de una redirección a una ruta. Mapea patrones de URL de tu dominio a rutas de la API de actions:

```json
{
  "rules": [{ "pathPattern": "/api/actions/**", "apiPath": "/api/actions/**" }]
}
```

Los patrones son globs: `*` coincide dentro de un solo segmento de ruta, `**` coincide a cualquier profundidad. Y ese mapeo se gana su lugar. Una regla puede emparejar una página para humanos con la action que está detrás, digamos `pathPattern: "/drop/**"` sobre `apiPath: "/api/actions/drop/**"`, para que un comprador que pega la URL común y corriente de la página de producto del prensado igual reciba un botón de compra renderizado, porque el cliente resolvió la página hasta su action. Si las URLs de actions solo se pegaran directo, el descubrimiento podría vivir en la URL misma; `actions.json` existe para que tus links normales se vuelvan blinks también.

Regla dos: toda respuesta de action manda `Access-Control-Allow-Origin: *`. Incluida, y esta es la que todo el mundo se salta, la de `actions.json` mismo. El fetch de descubrimiento también es cross-origin. Curl no hace cumplir CORS, los navegadores sí, que es exactamente por qué la firma de la falla es un endpoint que pasa limpio en tu terminal y no muestra nada en una billetera.

![Tabla comparativa de cuatro fallas de hosting, falta el actions.json en la raíz, falta CORS en las rutas, falta CORS en actions.json, URL de icon relativa, cada una bien en curl y rota en un cliente de verdad.](assets/v04-comparison.webp)

El preflight también importa: los clientes mandan OPTIONS antes del POST, así que tu middleware de CORS contesta OPTIONS con los mismos headers y un 204 vacío. El spec también define dos headers de respuesta informativos, `X-Action-Version` (la versión del spec que implementas) y `X-Blockchain-Ids` (un chain id CAIP-2; CAIP-2 es el estándar de nombres cross-chain de namespace más reference, acá `solana:` más el hash de génesis truncado a 32 caracteres, así que devnet es `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1`). Los clientes conformes los leen para decidir compatibilidad; mandarlos cuesta dos líneas.

### El baño de realidad: ¿dónde se renderiza esto en realidad?

Hora de ponerle precio a la contrapartida, porque te vendí el sueño dos secciones atrás y te mereces la factura.

Los blinks se lanzaron a mediados de 2024 con una demo que se le quedó pegada a todo el mundo en la cabeza: un link expandiéndose en un botón de compra en un feed de X. Yo repetí ese pitch ante una sala de comercios en su momento, con plena convicción. La convicción sobrevivió a los hechos. Lo que de verdad pasó es que el renderizado en X estaba, y sigue estando, mediado por una extensión del navegador Chrome: quien mira sin la extensión ve un link pelado, no un botón. La demo del feed era real, la implicación de "para todos los que lo ven" no.

Y el historial del tooling cuenta su propia historia. Acá está la línea de tiempo de releases, que puedes verificar en npm en treinta segundos:

![Línea de tiempo desde el lanzamiento de los blinks en 2024 pasando por los últimos releases de los SDKs, el giro de Dialect hacia una librería alojada, y la fecha de escritura de 2026 sin ninguna versión más nueva entregada.](assets/v05-timeline.webp)

`@solana/actions` no entrega nada desde el 2024-11-05. `@dialectlabs/blinks` no entrega nada desde el 2025-04-04. Dieciséis meses de silencio del SDK del cliente no son un bache de mantenimiento que rodeas, son una señal sobre a dónde se fue la atención del proveedor: Dialect giró hacia una Standard Blinks Library alojada, un servicio gestionado, en vez del SDK abierto. Así que este curso construye contra el contrato de cable en vez de contra los SDKs congelados; si algún proyecto tuyo sí los adopta, fija las dos versiones congeladas que el resumen nombra exactamente, escribe la nota de obsolescencia en el comentario de tu package.json o en el README, y trata "esperar al próximo release" como algo que no es un plan.

La Standard Blinks Library alojada puede parecer la salida del problema del SDK congelado: deja que Dialect corra el blink por ti. Dos razones para construir tu propio endpoint igual. Primero, tu drop es tu inventario, tu precio, tu conciliación de memo y reference; un servicio alojado es el hogar equivocado para la lógica de pagos de la que depende todo tu back office, y este curso lleva tres lecciones construyendo esa lógica adentro de un solo camino de código propio. Segundo, y esta es la razón duradera: el artefacto de esta lección es la conformidad con el spec, y ningún host puede ser dueño de eso por ti. El spec de Actions es un contrato de cable, metadatos por GET para adentro, POST de `{account}` para afuera; un SDK congelado no cambia el contrato que hablan tus endpoints, y cualquier renderizador construido el año que viene contra el mismo spec ejecuta tu tienda sin que tú entregues una línea. Estás construyendo contra el protocolo, no contra el roadmap de Dialect. Esa es la dependencia correcta para asumir con un proveedor cuyo SDK ha estado en silencio dieciséis meses.

Después está el registro. Dialect opera un registro de blinks donde las actions llevan un estado, y la documentación define los tres de un tirón: **trusted** es "registered by the developer and accepted by the registration committee" y se renderiza completa en los clientes participantes, **none** quiere decir "the action has not been registered" y típicamente se renderiza con advertencias o con la UI degradada, y **blocked** ha sido "flagged as malicious by the registration community" y no se renderiza. Quedar registrado no es una llamada a una API que haces tú: la ruta documentada de Dialect es un envío por correo, y la documentación dice sin adornos que "currently registration review is a manual process." Leer el registro programáticamente es otra historia, y en parte está detrás de una API key: la lista pública en `registry.dial.to/v1/list` le contesta a cualquiera, mientras que el endpoint de consulta por URL devuelve 403 sin una API key de Dialect (los dos sondeados el 2026-08-22). También vas a leer afirmaciones sobre cuándo empezó o empieza la aplicación del registro; esa fecha no tiene fuente, así que este curso no cita ninguna, y tú tampoco deberías. Lo que da un hecho operativo duro: no puedes poner como barrera de un lanzamiento, ni de esta lección, la fila manual de un tercero.

![Diagrama del flujo del registro de Dialect: un envío por correo entra a revisión manual, y los estados trusted, none y blocked se mapean a renderizado completo, degradado y rechazado.](assets/v06-diagram.webp)

¿Con qué superficies puedes contar en realidad, entonces? Acá está la caja honesta.

> **Verificado al escribir, 2026-08-22.** El renderizado en X está mediado por una extensión de Chrome, no es nativo. Los SDKs centrales están congelados en `@dialectlabs/blinks` 0.22.5 y `@solana/actions` 1.6.6. La revisión del registro de Dialect es un proceso manual al que se llega por correo, y su API de consulta por URL está detrás de una API key. Más allá de eso, la lista de 2026 de billeteras y superficies que renderizan blinks de forma nativa está **sin verificar**: las páginas de ecosistema que nombran billeteras específicas son de la ola de lanzamiento de 2024, y no pudimos confirmar el comportamiento actual de ninguna billetera específica al momento de escribir. Trata cada afirmación de "se renderiza en la billetera X" que leas, incluidas versiones viejas de afirmaciones como estas, como vencida hasta que la pruebes en esa billetera, esa semana.

La contrapartida, dicha una vez y llevada a todo lo que construyas hoy: un blink convierte cualquier superficie en un checkout, pero solo se ejecuta donde algo lo renderice. El tooling se congeló en 2025, X necesita una extensión, la admisión al registro es una revisión manual que no puedes agendar. "Se renderiza en todos lados" es falso. Lo que sí es cierto, y sigue siendo genuinamente valioso, es más angosto: un blink es un endpoint de checkout conforme al spec y autodescriptivo. Cualquier superficie actual o futura que hable el spec puede ejecutar tu tienda. Estás comprando una opción sobre distribución, barata, porque el costo marginal sobre el endpoint que ya tienes es una tarde. Ese es un buen trato mientras le pongas precio como una opción y no como un feed prometido. Y hay una segunda razón por la que el patrón tiene recorrido: el propio repositorio canónico de Solana Pay ahora abre con una CLI de pagos agénticos cuyo README llama a x402 y a MPP "both live payment standards on Solana" (los dos son estándares para pagar sobre HTTP pelado, construidos para compradores máquina; el módulo 7 los enseña como se debe), mientras la librería clásica de checkout sigue viva en un subdirectorio (README del repo y redirecciones, revisados de nuevo el 2026-08-22). La apuesta del ecosistema es que las cosas que ejecutan pagos desde donde sea que estén publicadas, para humanos o para agentes, son el rumbo. Los blinks son el extremo de cara a los humanos de ese mismo cambio, y tu extremo de cara a las máquinas llega en el módulo 7.

## Lab: entrega el drop blink

El reparto de hoy: yo recorro contigo el scaffold, los endpoints y las reglas de hosting (guiado). Tú ensamblas el constructor del `ActionPostResponse` por tu cuenta contra tres reglas enunciadas, con el desafío de código como tu verificador (completion). Después la compra desde el cliente local y la lista de verificación apta para el registro son solo tuyas (solo). Para el final, `npx tsx drop-blink/smoke.ts` pasa y un cliente de blinks local completa una compra en devnet del prensado destacado.

**1. Genera el scaffold del workspace.** El drop blink vive junto a tu proyecto checkout-txreq para poder importar el constructor. La misma línea de kit que el workspace de checkout-txreq (`@solana/kit` 6.10.0, el último release de la v6; este workspace se queda en v6 porque el constructor que importa también):

```bash
mkdir -p drop-blink/src drop-blink/public
cd drop-blink
npm init -y
npm pkg set type=module
npm install express@5.2.1 @solana/kit@6.10.0 @solana/actions-spec@2.4.2
npm install -D tsx@4 typescript @types/express @types/node
```

Los pins, con sus notas de frescura: `express` 5.2.1 es el `latest` de npm en la línea 5.x al momento de escribir (revisado de nuevo el 2026-08-22; cualquier 5.x funciona). `@solana/actions-spec` 2.4.2 es el más nuevo y, como el resto del tooling de blinks, está congelado; lo instalamos para que puedas hacerle diff a los tipos espejados a mano contra la fuente de verdad. `tsx` es el runner de TypeScript que has usado todo el curso; si esta es una máquina nueva, la línea de dev-install de arriba es su instalación.

Checkpoint: `npm ls @solana/actions-spec express` imprime 2.4.2 y un express 5.x, sin advertencias de unmet-peer. Una queja de peer sobre `@solana/kit` acá quiere decir que te saliste de la línea v6 en la que vive el constructor que estás por importar.

**2. Crea los tipos.** Guarda el archivo de `ActionParameter` / `LinkedAction` / `ActionGetResponse` de la sección de teoría como `drop-blink/src/types.ts`, y agrega al final las formas del lado del POST:

```ts
// drop-blink/src/types.ts (continued)

export interface ActionPostRequest {
  account: string; // base58 public key of the user who will sign
}

export interface NextActionLink {
  type: 'post';
  href: string; // the client POSTs here after the transaction confirms
}

export interface ActionPostResponse {
  type: 'transaction';
  transaction: string; // base64-encoded transaction
  message?: string;    // optional post-sign confirmation text
  links?: { next: NextActionLink };
}

export interface ActionsJson {
  rules: Array<{ pathPattern: string; apiPath: string }>;
}
```

**3. El constructor de la respuesta, tu peldaño de completion.** Esta es la función que el handler del POST va a llamar, y la única cosa de este lab que escribes sin mí. El contrato es exactamente el del desafío de código, así que puedes revisar tu trabajo allá antes de cablearlo acá:

```ts
// drop-blink/src/action-post-response.ts
import type { ActionPostResponse } from './types';

export function buildActionPostResponse(
  transactionBase64: string,
  message?: string | null,
  nextActionHref?: string | null,
): ActionPostResponse {
  // Rule 1: always set type: 'transaction' and transaction from transactionBase64.
  // Rule 2: add message ONLY when message is provided (not null, not undefined);
  //         the minimal response has no message key at all, not a message key
  //         set to undefined.
  // Rule 3: add links.next as { type: 'post', href } ONLY when nextActionHref
  //         is provided; otherwise the response has no links key.
  throw new Error('Your turn: assemble the response per the three rules above.');
}
```

Los argumentos son posicionales, y los dos opcionales pueden llegar como `null` o venir omitidos, así que trata `null` y `undefined` por igual como "ausente". Así exactamente es como lo llama el verificador del desafío: `buildActionPostResponse('B64', null, 'https://.../thanks')` es una respuesta con una action encadenada y sin message.

¿Por qué tan estricto con ausente-versus-undefined? Porque los clientes validan la forma de la respuesta, y una clave `links` con basura adentro falla la validación donde ninguna clave `links` pasa. Construye el objeto condicionalmente; no lo construyas maximal y después borres.

Checkpoint: tal como se entrega, este archivo lanza `Your turn: assemble the response per the three rules above.` en cada llamada. Ese throw es el peldaño de completion esperándote, y la corrida de humo del paso 8 es donde aparece.

**4. CORS y el esqueleto del servidor.** Un solo middleware, aplicado antes de toda ruta, que contesta el preflight:

```ts
// drop-blink/src/server.ts
import express from 'express';
import type { Request, Response, NextFunction } from 'express';
import { buildOrderTransaction } from '../../checkout-txreq/src/build-order-transaction';
import { buildActionPostResponse } from './action-post-response';
import type { ActionGetResponse, ActionPostRequest, ActionsJson } from './types';

const app = express();
app.use(express.json());

const ACTION_HEADERS: Record<string, string> = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET,POST,PUT,OPTIONS',
  'Access-Control-Allow-Headers':
    'Content-Type, Authorization, Content-Encoding, Accept-Encoding',
  'Access-Control-Expose-Headers': 'X-Action-Version, X-Blockchain-Ids',
  'X-Action-Version': '2.4.2',
  'X-Blockchain-Ids': 'solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1', // devnet CAIP-2
};

app.use((req: Request, res: Response, next: NextFunction) => {
  res.set(ACTION_HEADERS);
  if (req.method === 'OPTIONS') {
    res.status(204).end();
    return;
  }
  next();
});

// Static AFTER the CORS middleware, never before it: the icon is fetched
// cross-origin too, and a static route that matches first would ship it
// without the headers. This ordering is the works-in-curl table's row one.
app.use(express.static('public'));
```

La ruta de import de `buildOrderTransaction` asume el layout de módulos de la lección de transaction requests sentado un directorio más allá. Si dejaste el ensamblado de la transacción en línea adentro del handler de POST de esa lección en vez de extraerlo a una función, tómate cinco minutos ahora y extráelo. El blink es precisamente por qué: un constructor, muchas superficies. Su contrato acá es el mismo del que también va a depender tu capstone: toma la cuenta del comprador más lo que está comprando, le pone precio del lado del servidor, estampa el memo y la reference, y resuelve a la transacción en base64.

**5. El descubrimiento y el icon.** El archivo de la raíz y una imagen estática (mete cualquier PNG cuadrado en `public/drop-icon.png`; cualquier arte de relleno sirve):

```ts
// drop-blink/src/server.ts (continued)

const BASE_URL = process.env.BASE_URL ?? 'http://localhost:3000';
const DROP_SKU = 'WVL-045';
const DROP_PRICE_USDC = 30;

app.get('/actions.json', (_req: Request, res: Response) => {
  const payload: ActionsJson = {
    rules: [{ pathPattern: '/api/actions/**', apiPath: '/api/actions/**' }],
  };
  res.json(payload);
});
```

Lo sirve Express en la raíz de la app, que en el despliegue tiene que SER la raíz de tu dominio. Si tu sitio corre detrás de un prefijo de ruta o de un proxy, el archivo igual tiene que contestar en `https://yourdomain/actions.json`; esa es una regla de proxy inverso, no una ruta de aplicación, y es el detalle de despliegue que más se pierde entre "funcionaba en local" y producción.

**6. El GET: metadatos para el prensado destacado.**

```ts
// drop-blink/src/server.ts (continued)

app.get('/api/actions/drop', (_req: Request, res: Response) => {
  const payload: ActionGetResponse = {
    type: 'action',
    icon: `${BASE_URL}/drop-icon.png`,
    title: 'Wavelength Records: the August pressing',
    description: `Limited pressing ${DROP_SKU}. ${DROP_PRICE_USDC} USDC on devnet, 200 copies, gone when they are gone.`,
    label: `Buy for ${DROP_PRICE_USDC} USDC`,
    links: {
      actions: [
        {
          type: 'transaction',
          label: `Buy 1 for ${DROP_PRICE_USDC} USDC`,
          href: '/api/actions/drop',
        },
        {
          type: 'transaction',
          label: 'Buy more than one',
          href: '/api/actions/drop?qty={qty}',
          parameters: [
            { name: 'qty', label: 'How many copies (max 5)', required: true, type: 'number' },
          ],
        },
      ],
    },
  };
  res.json(payload);
});
```

Dos botones desde un solo endpoint: una compra fija de un clic, y una compra por cantidad parametrizada cuya plantilla `{qty}` el cliente sustituye en el query string. URL de icon absoluta. Fíjate en lo que NO está acá: nada de aritmética de precios, nada de lógica de inventario. Los metadatos describen; el POST decide.

**7. El POST y el paso de gracias encadenado.** El handler valida la entrada, acota la cantidad del lado del servidor (recuerda: los tipos de parámetro le dan estilo a un formulario, no validan), reusa el constructor y ensambla la respuesta a través de tu función del peldaño de completion:

```ts
// drop-blink/src/server.ts (continued)

app.post('/api/actions/drop', async (req: Request, res: Response) => {
  const body = req.body as ActionPostRequest;
  if (!body?.account || typeof body.account !== 'string') {
    res.status(400).json({ message: 'Body must be { "account": "<base58 pubkey>" }' });
    return;
  }
  const qty = Math.min(Math.max(Number(req.query.qty ?? 1) || 1, 1), 5);

  try {
    const { transactionBase64 } = await buildOrderTransaction({
      account: body.account,
      sku: DROP_SKU,
      quantity: qty,
    });
    res.json(
      buildActionPostResponse(
        transactionBase64,
        `Order placed: ${qty}x ${DROP_SKU}. Sign to complete the purchase.`,
        '/api/actions/drop/thanks',
      ),
    );
  } catch (err) {
    res.status(400).json({
      message: err instanceof Error ? err.message : 'Could not build the order transaction',
    });
  }
});

app.post('/api/actions/drop/thanks', (req: Request, res: Response) => {
  const signature =
    typeof req.body?.signature === 'string' ? req.body.signature : undefined;
  res.json({
    type: 'completed',
    icon: `${BASE_URL}/drop-icon.png`,
    title: 'You got the pressing',
    description: signature
      ? `Payment landed. Signature ${signature.slice(0, 8)}... is your receipt.`
      : 'Payment landed. Your order is in.',
    label: 'Done',
  });
});

app.listen(3000, () => {
  console.log('drop-blink listening on :3000');
});
```

El camino de error devuelve `{ message }` con un estado distinto de 200, que es la forma `ActionError` del spec; los clientes muestran ese message, así que escríbelo para el comprador y no para tus logs. El handler de gracias es el destino de `links.next`: después de que la billetera confirma la transacción, el cliente hace POST acá con la firma, y un payload `type: 'completed'` cierra el flujo con una tarjeta de comprobante. El encadenamiento va más profundo de lo que lo llevamos (un próximo paso puede ser una action entera más con su propia transacción), pero un solo salto alcanza para hacer tuyo el patrón.

Checkpoint: `MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts` imprime `drop-blink listening on :3000`. La variable de entorno no es opcional: el constructor que importaste sigue leyendo `MERCHANT_ADDRESS` del entorno, exactamente como lo hacía en su workspace de origen, y sin ella cada POST tira 400 con el propio mensaje de set-MERCHANT_ADDRESS del constructor antes de que se llegue siquiera a tu throw de relleno. En una segunda terminal, `curl -s http://localhost:3000/actions.json` devuelve tu objeto de una sola regla y `curl -s http://localhost:3000/api/actions/drop` devuelve los metadatos con las dos etiquetas de botón adentro. Se espera que el POST falle por ahora, en el throw de relleno del paso 3.

**8. Pásale la prueba de humo.** El banco de pruebas de verificación de este artefacto, y tu checkpoint:

```ts
// drop-blink/smoke.ts
import { generateKeyPairSigner } from '@solana/kit';

const BASE = process.env.BLINK_URL ?? 'http://localhost:3000';

function fail(msg: string): never {
  console.error(`SMOKE FAIL: ${msg}`);
  process.exit(1);
}

async function main() {
  const testAccount =
    process.env.TEST_ACCOUNT ?? (await generateKeyPairSigner()).address;

  const aj = await fetch(`${BASE}/actions.json`);
  if (aj.headers.get('access-control-allow-origin') !== '*') {
    fail('actions.json is missing Access-Control-Allow-Origin: *');
  }
  const discovery = (await aj.json()) as { rules?: unknown[] };
  if (!Array.isArray(discovery.rules) || discovery.rules.length === 0) {
    fail('actions.json has no rules');
  }

  const get = await fetch(`${BASE}/api/actions/drop`);
  if (get.headers.get('access-control-allow-origin') !== '*') {
    fail('GET metadata is missing CORS');
  }
  const meta = (await get.json()) as Record<string, unknown>;
  for (const field of ['icon', 'title', 'description', 'label']) {
    if (typeof meta[field] !== 'string') fail(`GET metadata is missing ${field}`);
  }
  if (!String(meta.icon).startsWith('http')) fail('icon must be an absolute URL');

  const post = await fetch(`${BASE}/api/actions/drop`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ account: testAccount }),
  });
  if (post.status !== 200) fail(`POST returned ${post.status}`);
  const body = (await post.json()) as Record<string, unknown>;
  if (body.type !== 'transaction') fail("POST response type must be 'transaction'");
  if (typeof body.transaction !== 'string' || body.transaction.length === 0) {
    fail('POST response has no base64 transaction');
  }

  console.log('SMOKE PASS: actions.json + GET metadata + POST response all conformant');
}

main().catch((err) => fail(err instanceof Error ? err.message : String(err)));
```

Corre el servidor en una terminal:

```bash
MERCHANT_ADDRESS=$(solana address) npx tsx src/server.ts
```

Espera su línea de listening, después corre la comprobación de humo en una segunda terminal:

```bash
npx tsx smoke.ts
```

Dos terminales en vez de un solo comando en segundo plano, a propósito: `smoke.ts` abre con un `fetch` que no reintenta, y `tsx` se toma un segundo o dos para compilar y hacer bind. Encadenado detrás de un `&` el fetch normalmente pierde esa carrera y te entrega un `ECONNREFUSED` que no tiene nada que ver con tu código.

Deberías ver `SMOKE PASS`. Si en cambio falla en el POST, tu `buildActionPostResponse` todavía lanza su relleno, que es el lab diciéndote que el peldaño de completion es genuinamente tuyo. Termínalo, o trabájalo primero en el desafío de código y pega acá de vuelta tu implementación que pasa.

## Challenge

**El desafío de código** (en el widget de challenge de esta lección) es `buildActionPostResponse` en aislamiento, llamado posicionalmente como `buildActionPostResponse(transactionBase64, message?, nextActionHref?)`: la transacción en base64 sola produce exactamente `type` más `transaction` y nada más, `message` aparece solo cuando se provee un segundo argumento no nulo, `links.next` aparece solo como `{ type: 'post', href }` cuando se provee un tercer argumento `nextActionHref` no nulo. Pásalo, después trae el código a casa, al paso 3.

**El peldaño solo, en dos partes.** Sin recorrido guiado esta vez; tienes todo lo que necesitas.

Primero, completa una compra real siendo tú mismo el cliente que renderiza, porque eso es todo lo que es un cliente de blinks: GET, renderizar, POST, firmar, enviar, seguir `links.next`. Escribe `drop-blink/client.ts` en el workspace. Hace fetch de los metadatos del GET e imprime el title y las dos etiquetas de botón (ese es tu paso de render), hace POST de `{account}` con la dirección de tu billetera de devnet fondeada, decodifica la transacción en base64 devuelta, la firma y la envía exactamente como te hizo hacerlo el solo de transaction requests (carga la clave con `createKeyPairSignerFromBytes`, firma con `signTransaction`, manda el base64 a través de `rpc.sendTransaction`), después hace POST de la firma confirmada al href de `links.next` e imprime el title de la tarjeta de completado. El éxito es concreto: una transacción de devnet liquida llevando el memo de tu pedido y tu clave de reference, y tu terminal termina en el title de la tarjeta de gracias. Una nota de honestidad mientras lo construyes: un cliente de Node no hace cumplir CORS, que es exactamente por qué la prueba de humo revisa los headers explícitamente; la tabla de cuatro filas de funciona-en-curl es lo que golpearía un cliente basado en navegador, y tus volcados de headers de abajo son la demostración de que sobrevivirías a uno.

Segundo, produce una **lista de verificación apta para el registro** para el drop blink: un archivo markdown corto en el repo que afirme, con evidencia, (1) la conformidad con el spec, tu salida de humo; (2) CORS en toda ruta de action y en `actions.json`, con volcados de headers; (3) `actions.json` alcanzable en la raíz de dominio de tu destino de despliegue; (4) la disposición para el envío, la URL pública de la action y el contacto que le mandarías a la revisión manual de Dialect, más qué variable de entorno llevaría una API key de Dialect si alguna vez consumes el endpoint de consulta que está detrás de una API key. Fíjate en lo que la lista deliberadamente no afirma: ningún renderizado en superficies externas, ni ninguna línea de tiempo del registro. Esa contención es el punto. La lista de verificación es el artefacto que un tú futuro, o un cliente, puede entregarle a la revisión de Dialect sin una sola promesa que no puedas cumplir.

La evidencia de los headers toma un comando por endpoint; `-D -` vuelca los headers de respuesta a stdout y `-o /dev/null` descarta el body:

```bash
curl -s -D - -o /dev/null http://localhost:3000/actions.json | grep -i access-control
curl -s -D - -o /dev/null http://localhost:3000/api/actions/drop | grep -i access-control
```

Pega las dos salidas en la lista de verificación, textuales. La evidencia que puedes regenerar en diez segundos les gana a las garantías en prosa todas las veces, y cuando vuelvas a desplegar detrás de un proxy distinto en el capstone, volver a correr dos líneas de curl vuelve a demostrar la afirmación.

![Tabla de las cuatro afirmaciones aptas para el registro (conformidad con el spec, CORS en todos lados, actions.json en la raíz, disposición para el envío), cada una con evidencia regenerable y una insignia de barrera o de preparación.](assets/v07-table.webp)

Aceptación: el GET valida, el POST devuelve una respuesta conforme al spec que reusa el constructor de transaction request, tu script de cliente completa una compra en devnet y la lista de verificación existe con los cuatro puntos de evidencia.

## Checkpoint, y dónde aterriza el módulo

Si la prueba de humo te peleó, la falla es casi con seguridad una de cuatro: headers de CORS aplicados después de que una ruta ya coincidió (mueve el middleware por encima de toda ruta, estáticos incluidos), `actions.json` montado bajo `/api` en vez de en la raíz, la ruta de import del constructor que no coincide con tu layout de checkout-txreq, o `MERCHANT_ADDRESS` faltando en el entorno del servidor (el constructor importado la exige y tira 400 sin ella). Diez minutos, según mi experiencia, casi siempre la tercera. Y si algo más sutil se rompió, hazle diff a tu archivo de tipos contra `@solana/actions-spec` en `node_modules`; el paquete del spec está congelado, así que cualquier campo que espejamos y que no concuerde con la fuente está de nuestro lado por definición (los campos que existen solo del lado del spec son los recortes deliberados que nombra el encabezado del archivo de tipos). Cuando consigas el pass, tómate la victoria en serio: entregaste una tienda-en-un-link conforme al protocolo, con afirmaciones de alcance que puedes defender línea por línea, y esa combinación es más rara en la vida real que el endpoint mismo.

Tres superficies, un solo núcleo de pagos: el checkout con código QR, el puesto de la feria y ahora un blink que se ejecuta donde sea que algo lo renderice. Wavelength puede vender un disco a través de una página, al otro lado de una mesa y adentro de un link. Lo que quiere decir que el frente de la tienda está listo, y la pregunta honesta se mueve hacia adentro: está llegando dinero desde tres superficies y todavía estás confiando en que los frontends te cuenten sobre eso. El próximo módulo deja el frente de la tienda por el back office, donde no le crees a ningún frontend y verificas cada pago del lado del servidor.
