# La feria de discos: la terminal (POS), y la realidad del hardware móvil

La lección pasada moviste la construcción de la transacción a tu backend: checkout-txreq construye la transacción del lado del servidor y estampa el memo y la reference por su cuenta, así que el precio vive en tu código y no en una URL que el cliente controla. Ese endpoint solo se ha manejado desde una pestaña del navegador y un script de humo en tu propia máquina. Hoy se enfrenta a un cliente.

Es sábado. Tienes una mesa plegable en una feria de discos, un cajón de prensados de Wavelength, ningún lector de tarjetas y una fila que se forma. Tu checkout es una página web. ¿Puede cobrar dinero del otro lado de una mesa?

Sí, y no vas a escribir un frontend nuevo para lograrlo. El repo de Solana Pay trae una app de punto de venta propia, `examples/point-of-sale`, una app de Next.js con teclado numérico, pantalla de QR y flujo de confirmación. Un solo interruptor adentro apunta todo el asunto al endpoint de transaction request que ya construiste. Arranca la clonación ahora para que la instalación corra mientras lees:

```bash
# the old solana-labs/solana-pay URL redirects here; cloned fresh 2026-08-22
git clone https://github.com/solana-foundation/pay.git
cd pay && git checkout 94b3627   # the POS example this lesson was written against; main moved to kit v8 on 2026-08-31

# the example consumes the repo's core package by path, and that package ships unbuilt.
# build it from the repo's own pnpm workspace first, or the app 500s on a missing import.
cd typescript && pnpm install && pnpm --filter @solana/pay build

cd packages/solana-pay/examples/point-of-sale
npm install   # Node 24+, the course floor, more than covers this repo's own pins
```

Ese bloque asume que `pnpm` está en tu path; si no lo está, `corepack enable pnpm` prende el shim que Node ya trae.

Sobre la versión de Node, porque si no te va a morder antes de que termine la instalación: el `package.json` del propio ejemplo fija `engines.node >=18`, pero consume el paquete core del repo por ruta (`"@solana/pay": "file:../../core"`), y ese paquete fija `engines.node >= 20` para Ed25519 en `crypto.subtle`. Así que el piso que este repo impone es 20 — y el Node 24+ que corres desde el módulo 1 lo pasa con margen. Confírmalo ahora y no en el primer import que falle:

```bash
node --version
```

Checkpoint: `v24.x` o más nuevo, el piso del curso desde el módulo 1. En 18 la instalación puede muy bien funcionar y después la primera comprobación de firma lanza un error adentro de `crypto.subtle`, que es una falla confusa de depurar con el mensaje de error solo.

Un arreglo de pre-vuelo más, y es de upstream, no tuyo. `@solana/connector`, la capa de billetera que usa el POS, lista `@solana/web3.js` como peer *opcional* y lo busca con `await import('@solana/web3.js')` adentro de una rama de transacción legacy que esta app nunca toma. Los peers opcionales no se instalan, y el ejemplo queda por fuera del pnpm workspace del repo, así que npm lo resuelve sin lock y ese import no tiene a qué apuntar. A Webpack no le importa que la rama esté muerta: resuelve `import()` en tiempo de build y hace fallar la build. Te llevas un 500 en la primera carga de página que dice `Module not found: Can't resolve '@solana/web3.js'`. Sigue abierto en `main` al 2026-09-01, así que el pin no lo causó.

El arreglo es una línea de config del bundler, y es el honesto: le estás diciendo la verdad a webpack, que una dependencia opcional no está. Agrega un hook `webpack` al objeto de config en `next.config.js`, al lado de `reactStrictMode`:

```js
webpack(config) {
    // @solana/connector's optional peer, reached only by a legacy code path this app
    // never takes. It is not installed; resolve it to nothing instead of failing the build.
    config.resolve.fallback = { ...config.resolve.fallback, '@solana/web3.js': false };
    return config;
},
```

No "arregles" esto instalando `@solana/web3.js`. Nada en este puesto corre sobre él, y meter el SDK deprecado en un árbol de kit v6 para satisfacer un import muerto es el hábito equivocado de aprender. Checkpoint con los dos arreglos puestos: `npm run dev`, y después en otra terminal `curl -o /dev/null -w '%{http_code}\n' 'http://localhost:3000/new?recipient=<any address>&label=Test'` imprime `200`.

Mientras eso se instala, una promesa sobre el resto de esta lección: la mitad es la construcción, y la otra mitad es un barrido honesto de qué hardware de Solana para ventas en persona existe de verdad en 2026. La segunda mitad importa tanto como la primera, porque la forma más rápida de perder la confianza de un comercio es prometerle pago sin contacto en una blockchain que no lo tiene.

## Resumen

Esto es lo que hoy queda establecido, línea por línea:

- El repo pay trae un POS propio en `examples/point-of-sale` (Next.js, teclado numérico, QR, flujo de confirmación). Lo configuras con una URL: `/new?recipient=<address>&label=<name>`.
- Una sola línea comentada en `App.tsx` cambia el POS de transfer requests a transaction requests. Apunta ese `link` a tu endpoint de checkout-txreq y el puesto reusa la lógica de precios del lado del servidor que construiste la lección pasada.
- En modo link el POS le agrega los parámetros de la venta (`recipient`, `amount`, un `reference` fresco, `label`) a la URL de tu endpoint antes de codificar el QR. El precio del teclado es input del comercio desde el propio dispositivo del comercio, que es un modelo de confianza distinto al de una URL que el cliente puede editar. Tu endpoint igual lo valida, y nunca toma al destinatario del query.
- La pantalla de confirmado es un poll, no un push: `findReference` recorre `getSignaturesForAddress` sobre la clave de reference de la venta hasta que aparece una firma, y después el pago se valida. Conocer ese loop es cómo depuras un "pending" trabado.
- No hay primitiva de NFC ni de pago sin contacto en Solana. Ningún spec define un protocolo de lector y ninguna documentación oficial describe uno. El spec sí dice que una URL de Solana Pay "may be encoded in QR codes or NFC tags", pero eso es transporte, una etiqueta que sostiene la misma cadena, no un flujo de tap. El código QR es la superficie estándar de checkout. No prometas tap.
- Seeker, Seed Vault y la dApp Store son productos reales, pero las cifras de su página de inicio (una línea de "150,000+ users" con un referente ambiguo, una línea de "0% platform fees") son afirmaciones de página de inicio. Los números de unidades despachadas no están divulgados. No cites ninguna de ellas como hecho verificado.
- Decaf, que alguna vez fue el nombre del POS de festivales en Solana, ahora vende links de pago globales que aceptan tarjeta, transferencia bancaria o crypto, con retiro en efectivo y desembolsos transfronterizos en más de 180 países, y sin ningún POS de Solana que quede en su sitio (decaf.so, vuelto a comprobar el 2026-08-22). No hay proveedor de terminales a quien comprarle, así que construyes sobre el ejemplo propio del repo.
- Commerce Kit está en beta ("APIs may change"). Acá recibe una mención y ningún rol que cargue peso.

Cómo se reparte el trabajo hoy: el lab guiado te entrega cada comando para clonar, recablear y correr el puesto. El peldaño de completion deja el destinatario del comercio, el link del endpoint y el carrito como tres TODOs en la config de `pos-stall`. El peldaño solo es la cosa de verdad: una venta completa al estilo en persona, del escaneo a la firma liquidada, con el comprobante registrado. La comprobación de esta lección es ese despliegue, no un quiz.

## Un interruptor de la página web al punto de venta

### Qué acabas de clonar

El ejemplo de punto de venta es una app pequeña con forma de producción: un frontend de Next.js, una capa de API, rate limiting y un `.env.example` que pone `CLUSTER_ENDPOINT` en devnet por defecto. Su `package.json` en el commit fijado corre sobre `@solana/kit ^6.9.0` y consume `@solana/pay` del propio paquete core del repo, cuyo release en npm es 1.0.26 (publicado el 2026-07-31, todavía `latest` en una comprobación del 2026-08-22, con pares en kit ^6.9). Eso mantiene al puesto adentro del mismo workspace de kit v6 que este curso usa desde el módulo 2. En `main` ya no sería así: un commit del 2026-08-31 movió el ejemplo a kit v8, y por eso exactamente el clone de arriba saca `94b3627` en vez de ir montado en `main`. En el pin, ninguna línea nueva de SDK, ningún acantilado de versiones.

Tal como viene, toda la app se configura con una sola URL, sin ningún archivo de config involucrado:

```
/new?recipient=<your merchant address>&label=<your stall name>
```

Abre eso y aparece un teclado numérico; teclas un monto y renderiza un QR que codifica un transfer request, la forma `solana:<recipient>` de dos lecciones atrás. El cliente escanea, su billetera construye la transferencia, y la app le hace polling a `findReference` hasta que el pago aterriza, y después salta a una pantalla de confirmado. Útil, pero tiene exactamente la limitación que te empujó a los transaction requests: la billetera construye la transacción, así que un destinatario fijo a un monto tecleado es todo lo que puede expresar. Ningún memo, ningún id de pedido, ninguna lógica de carrito.

El desbloqueo está nueve líneas adentro del componente de la app. Abre `src/client/components/pages/App.tsx` y busca esto:

```tsx
// If you're testing without a mobile wallet, set this to true to allow a browser wallet to be used.
const connectWallet = false;

// Toggle comments on these lines to use transaction requests instead of transfer requests.
const link = undefined;
// const link = useMemo(() => new URL(`${baseURL}/api/`), [baseURL]);
```

Los mantenedores dejaron la costura ahí a propósito. Cuando `link` es una URL, el POS deja de codificar transfer requests y empieza a codificar transaction requests, `solana:<https-link>`, apuntados a donde sea que `link` apunte. La línea comentada lo apunta a la propia API empaquetada de la app. Tú lo vas a apuntar a checkout-txreq en cambio, porque ya construiste el mejor endpoint: pone el precio del lado del servidor, estampa el memo y la reference, y la prueba de humo de la lección pasada demuestra que devuelve una transacción en base64 para un `{account}` que le llega por POST.

![Fragmento anotado de App.tsx que muestra la bandera connectWallet y el interruptor de link: link en undefined quiere decir que la billetera construye una transferencia, link puesto quiere decir que tu servidor la construye.](assets/v01-annotated-code.png)

### Cómo fluye la venta en realidad

Rastrea una venta por el puesto recableado, porque dos de los saltos son nuevos y uno de ellos cambia el trabajo de tu endpoint.

Teclas 0.15 en el teclado numérico y le das a generar. El POS toma tu URL de `link` y le agrega la venta como parámetros de consulta antes de codificar nada: el `recipient` configurado, el `amount` tecleado, el `label` de tu puesto, y `reference`, una clave pública fresca de un solo uso que acuña para esta venta con `generateKeyPairSigner` (la misma disciplina de reference base58 de 32 bytes que usas desde la primera lección de QR). Después codifica todo el conjunto con `encodeURL({ link })` y pinta el QR. El cliente escanea. Su billetera hace el paso doble que implementaste la lección pasada: GET a tu endpoint por el label y el icon, y después POST de `{account}` a la misma URL, con query string y todo. Tu servidor construye la transacción, la billetera firma y envía, y el POS le hace polling a `findReference` sobre la reference que acuñó hasta que aparece la firma, y después valida y salta a la pantalla de confirmado.

![Diagrama de flujo de una venta en tres carriles (POS, billetera del cliente, servidor), desde teclear un monto y acuñar una reference hasta firmar, enviar y el POS confirmando vía findReference.](assets/v02-flowchart.png)

Acá está el salto que cambia el trabajo de tu endpoint: el monto llegó en la URL. Dos lecciones de este curso te han taladrado "nunca confíes en un precio que manda el cliente", y ahora el precio vuelve a viajar en un query string. ¿Entonces qué es?

Mira quién acuñó la URL. En el módulo 3 lección 1, el cliente sostenía un link que podía editar antes de que su billetera lo usara, así que un precio en esa URL era input del cliente. En el puesto, el POS corre en tu dispositivo, detrás de tu mesa. El monto del teclado es input del comercio: lo tecleaste tú, tu dispositivo acuñó el QR, y el cliente solo llega a escanear lo que le mostraste. Ese es el mismo modelo de confianza que teclear un precio en una terminal de tarjetas. La regla no cambió, el autor de la URL sí. Tu endpoint igual debería validar la forma con dureza (finito, positivo, límites sensatos), porque un endpoint https es alcanzable por cualquiera, no solo por tu POS, y una llamada malformada u hostil tiene que fallar a los gritos en vez de construir una transacción. Lo que el endpoint ya no tiene que hacer para las ventas del puesto es buscar un carrito por id de pedido, porque el total se compuso en la mesa:

```typescript
// checkout-txreq: the stall branch of your POST handler's pricing step.
// Keypad sales arrive with amount + reference minted by YOUR pos device;
// validate the shape, then price from them instead of a stored cart.
import { toBaseUnits } from '../../transfer-kit/src/index';

export function priceFromStallLink(query: URLSearchParams): {
  baseUnits: bigint;
  reference: string;
} {
  const amount = query.get('amount');
  const reference = query.get('reference');
  if (!amount || !reference) {
    throw new Error('stall sales carry amount + reference minted by the POS');
  }
  // Number() here is a RANGE check on the string, never the money math.
  const bound = Number(amount);
  if (!Number.isFinite(bound) || bound <= 0 || bound > 100) {
    throw new Error(`rejected amount: ${amount}`);
  }
  // Exact base units straight from the decimal string, using the same helper
  // transfer-kit has used since module 2: devnet USDC at 6 decimals, the same
  // mint and decimals the web-cart path prices in. No float touches the amount.
  return { baseUnits: toBaseUnits(amount, 6), reference };
}
```

Cablea eso en el handler de POST que construiste la lección pasada como una rama: si el query lleva `amount` y `reference`, es una venta del puesto, pon el precio desde el link; si no, es el camino del carrito web que ya tienes. Fíjate en cuál función llama la rama del puesto, porque no es `buildOrderTransaction`: esa función sigue negándose a aceptar un monto de cualquier llamador, exactamente como se diseñó la lección pasada, y el diseño se sostiene. La rama del puesto construye su propio `TransferChecked` a partir del monto validado del teclado y se lo entrega derecho a `finalizeTransaction`, el tramo final compartido que la lección pasada exportó para exactamente este tipo de llamador, y hace pasar el `reference` acuñado por el POS en vez de acuñar uno nuevo. Esa última parte carga peso: el POS le hace polling a `findReference` sobre la clave que él acuñó, así que un servidor que mete su propio reference deja la pantalla de confirmado en pending para siempre. Todo lo que está aguas abajo, el estampado del memo, la inyección de la reference, la respuesta en base64, sigue siendo el código que ya escribiste. Esa es la acreción que este módulo no deja de prometer: pos-stall no reemplaza a checkout-txreq, lo consume.

Un parámetro merece una regla más dura que la validación. El POS también le agrega `recipient` al query, y tu endpoint debería ignorarlo por completo. El beneficiario es configuración en tu servidor, puesta una sola vez, no un valor que llega en cada request; un endpoint que le paga a cualquier destinatario que nombre el query es un open redirect para el dinero, porque cualquiera que alcance la URL https puede poner ahí su propia dirección. La misma historia con `memo` si aparece: tu endpoint estampa su propio memo con su propio id de pedido, y eso sigue siendo cierto en el puesto. Al query se le permite decirte cuánto es esta venta. Nunca se le permite decirte quién cobra ni qué dicen los libros.

![Un desglose etiquetado de la URL que va en el payload del QR: el esquema solana, el link https que la marca como transaction request, el camino /txreq y los parámetros por venta que agrega el POS.](assets/v03-diagram.png)

### Qué sabe en realidad la pantalla de confirmado

El último salto merece una mirada más de cerca, porque es el que te vas a quedar viendo cuando una venta se cuelgue. ¿Cómo sabe una página web en tu laptop que una transacción que nunca vio, firmada en un teléfono que nunca tocó, acaba de aterrizar en devnet?

Hace polling. En este flujo no hay canal de push: el POS llama a `findReference` sobre la clave de reference que acuñó para la venta, y por debajo eso es `getSignaturesForAddress` contra tu RPC, repetido cada cierto intervalo. La reference viaja en la transacción como una clave no firmante en la instrucción de transferencia (tu endpoint la inyecta, ese fue el trabajo de la lección pasada), así que en el momento en que la transacción aterriza, la clave de reference tiene un historial de firmas de exactamente una entrada. Hasta entonces, `findReference` lanza `FindReferenceError`, el POS lo atrapa, espera y pregunta otra vez. El pending no es un estado que la blockchain reporte; el pending es el loop que todavía no ha encontrado nada.

Dos consecuencias caen de ese diseño, y las dos te van a ahorrar tiempo de depuración el sábado. Primero, una pantalla de pending trabada tiene exactamente tres sospechosos: el cliente nunca aprobó (mira su teléfono), la transacción falló on-chain (la billetera muestra el error), o tu RPC todavía no ha indexado la firma (espera, o revisa tú mismo la clave de reference en un explorador). El POS no puede distinguirlos, pero tú sí, en unos diez segundos, revisando en ese orden. Segundo, la confirmación que el POS te muestra corre a un nivel de commitment; la biblioteca no acepta ni siquiera `processed` para esta consulta, que es la API haciendo cumplir calladamente la política que este curso repite desde el módulo 1: nunca entregues mercancía con un estado que todavía se puede revertir. Si `confirmed` alcanza para entregar un disco, o si esperas a `finalized` mientras charlas, es una decisión de política de verdad con números de latencia de verdad pegados, y la lección de liquidación del próximo módulo la hace rigurosa. Para un puesto de sábado en devnet, el valor por defecto está bien.

Después de que aparece la firma, el POS valida la transacción encontrada antes de dar vuelta la pantalla, comprobando que lo que aterrizó coincida con la venta que codificó. Mantén ese orden en la cabeza: encontrada, después validada, después confirmada-en-pantalla. Que exista una firma no es lo mismo que exista el pago correcto.

![Un diagrama de flujo del loop de confirmación del POS, findReference haciendo polling hasta que aparece una firma y después validando antes de mostrar confirmado, con una lista ordenada de tres sospechosos para diagnosticar una pantalla de pending trabada.](assets/v04-flowchart.png)

### La realidad del hardware: qué existe, qué no

Ahora la segunda mitad de la lección, y quiero ser honesto contigo, porque acá es donde mucho contenido de comercio en Solana sobrevende calladamente. Estás a punto de correr un punto de venta con una laptop y un código QR, y un vendedor de la mesa de al lado te va a hacer la pregunta obvia: "¿no pueden simplemente apoyar el teléfono?"

No. No hay primitiva de NFC ni de pago sin contacto en Solana: nada en el spec de Solana Pay, en la documentación oficial ni en el stack de Solana Mobile define una, y ninguna fuente primaria describe un flujo de tap que pudieras entregar.

Sé preciso con una línea que se lee mal como una promesa, porque un comercio que haga grep la va a encontrar. La sección de motivación del spec dice que las URLs de Solana Pay "may be encoded in QR codes or NFC tags, or sent between users and applications to request payment and compose transactions." Esa oración es sobre *transporte*: una etiqueta NFC es otra forma de pasarle a alguien la misma URL `solana:`, exactamente igual que imprimirla en una calcomanía. No define ningún protocolo de lector, ningún handshake, ningún elemento seguro, nada que pase cuando un cliente acerca un teléfono a tu mesa. El pago sin contacto como lo conoces de una terminal de tarjetas es una capacidad del mundo EMV, construida sobre elementos seguros, redes adquirentes y lectores certificados, y nada de esa plomería tiene hoy un equivalente en Solana. Una URL en una etiqueta NFC es un código QR que no puedes ver; no es pago sin contacto. El código QR es la superficie estándar de checkout, punto. Cuando le prometes a un comercio "checkout con crypto", la forma honesta de esa promesa es una cámara apuntada a una pantalla.

Antes de que leas eso como una degradación, mira a dónde se fueron de verdad los pagos en persona en esta década. Los sistemas de escanear-para-pagar más grandes del planeta son sistemas de código QR: Pix, UPI, Alipay y WeChat Pay todos terminaron eligiendo la cámara como superficie de checkout, en exactamente este tipo de mesa, precisamente porque un código QR necesita cero hardware especial del lado que vende. Un vendedor ambulante imprime un código una vez y ya está en el negocio. El tap exige un elemento seguro certificado en un lector certificado dentro de una relación certificada con un adquirente; el escaneo exige una pantalla, o papel. Así que el encuadre honesto para tu vecino vendedor no es "Solana todavía no puede hacer tap", es "el checkout de Solana funciona como funcionan los rieles de pago más nuevos del mundo". La brecha que sí es real, y sobre la que vale la pena ser honesto, es el pulido: esos sistemas nacionales tienen una década de UX de billetera detrás, y el primer escaneo de Solana Pay de un cliente va a sentirse menos ensayado que su centésimo escaneo de Pix. Esa es una brecha de software, no de hardware, y las brechas de software se cierran.

¿Y del lado del teléfono? Solana Mobile vende Seeker, un teléfono con Seed Vault, que es custodia de claves en hardware, no una función de pagos, y corre una dApp Store. Productos reales, y la propuesta de comisiones de la dApp Store es genuinamente interesante para la economía de una app. Pero cuidado con los números de la página de inicio: la cifra de "150,000+ users" tiene un referente ambiguo (usuarios de qué, exactamente, no se dice), la línea de "0% platform fees" es una afirmación de precios, y los números de unidades despachadas no están divulgados. Trata todo eso como afirmaciones de página de inicio, no presentes nada de eso como datos de adopción verificados, y fíjate en lo que falta: nada en el stack de Seeker le da pago sin contacto a tu puesto tampoco. Un cliente con Seeker en tu mesa sigue escaneando el mismo código QR que un cliente con iPhone.

![Una matriz de capacidades que marca los transfer y transaction requests por QR y el ejemplo propio de POS como reales, Commerce Kit como beta, y las terminales de pago sin contacto como inexistentes en Solana.](assets/v05-comparison.png)

La evidencia más filosa de lo delgado que es el nicho en tienda viene de la empresa que lo tenía. Decaf era el niño mimado del POS de festivales de este ecosistema: el nombre que oías cada vez que alguien pagaba comida con USDC en un evento de Solana. Entra a decaf.so hoy (lo volví a comprobar el 2026-08-22) y el POS de Solana ya no está. El producto es un link de pago que creas en dos minutos, pagable con tarjeta, transferencia bancaria o crypto, con retiro en efectivo y desembolsos en más de 180 países, apuntado exactamente al remitente que Stripe y PayPal no van a atender. Misma empresa, mismos rieles por debajo, cliente completamente distinto.

Lee ese giro como dato de mercado, porque es lo que es. La demanda es un actor acá, y votó: el comercio parado frente a una terminal resultó ser un cliente mucho más chico que el trabajador que manda dinero a casa o la empresa que factura del otro lado de una frontera. La demanda de POS de crypto en tienda era más delgada que la demanda de desembolsos, así que el capital y el producto siguieron a los desembolsos. La misma forma aparece en todas las comunidades de builders de América Latina: el pago con crypto que pasa todos y cada uno de los días es el desembolso transfronterizo a un colaborador, no el café comprado con una billetera. Nada de esto quiere decir que tu puesto sea una mala idea. Quiere decir que nadie te va a vender una terminal para él, que no hay catálogo de proveedores en el que apoyarte, y que el ejemplo propio del repo que clonaste es la base sancionada precisamente porque la capa comercial que estaba encima se vació. Construye en consecuencia, y ten claro que el mismo endpoint que le da potencia a tu mesa es la pieza que se traslada a donde vive de verdad la demanda.

![Línea de tiempo de Decaf moviéndose del punto de venta de festivales en Solana, pasando por la demanda delgada en tienda, hasta los links de pago globales y los desembolsos transfronterizos en más de 180 países.](assets/v06-timeline.png)

Hay una pieza más de honestidad sobre el hardware, y es la que nadie pone en una diapositiva: la red en la feria. Camina el flujo de la venta otra vez y cuenta las conexiones que necesita. Tu laptop tiene que ser alcanzable por el teléfono del cliente (el GET y el POST a tu endpoint) y tiene que alcanzar el RPC de devnet (el poll de confirmación). El teléfono del cliente tiene que tener datos, porque su billetera le envía la transacción firmada a la blockchain misma. Esas son tres dependencias de red para una sola venta, y una feria de discos en un salón parroquial con paredes de concreto y doscientos teléfonos en un solo punto de acceso va a poner a prueba cada una de ellas. Esta no es una debilidad específica de crypto, las terminales de tarjetas también se mueren con mala conectividad, pero un proveedor de terminales de tarjetas lleva veinte años haciendo ingeniería de store-and-forward alrededor de eso, y tú no. Todavía. Más adelante en este curso construyes exactamente eso: una fila offline que firma ventas en la mesa y las drena cuando la red vuelve, sobre una primitiva que se llama durable nonce. Por ahora, las mitigaciones prácticas son aburridas y efectivas: tu propio hotspot para la laptop, un código QR impreso de respaldo para un artículo de precio fijo, y saber cuál falla se parece a cuál en la pantalla de pending.

![Una lista de verificación previa a la feria que cubre el alcance por LAN, el acceso al RPC, los datos del teléfono del cliente, un hotspot, la aceptación del certificado y un código QR impreso de respaldo, más cómo se presenta cada falla de red en el puesto.](assets/v07-table.png)

Un último aparte antes del lab. Hay un Commerce Kit en el ecosistema, y está en beta, con la advertencia "APIs may change" de su propia documentación adjunta. Ya sabes lo suficiente para decodificar lo que eso quiere decir para un puesto del que dependes: un sábado de ventas no es el lugar para una superficie de API que se reserva el derecho de moverse debajo de ti. Sabe que existe, míralo madurar, y construye la mesa de hoy sobre el ejemplo propio del repo y tu propio endpoint. Esa es toda la mención.

## Lab: monta el puesto

Peldaño Worked: cada comando de abajo viene dado. Clonas (ya hecho arriba), recableas y cobras una venta localmente.

1. **Corre el POS de fábrica, una vez.** Desde `pay/typescript/packages/solana-pay/examples/point-of-sale`, arranca el servidor de desarrollo y, en una segunda terminal, el proxy SSL (viene como dependencia de desarrollo, tu `npm install` ya lo bajó):

   ```bash
   npm run dev     # Next.js on http://localhost:3000
   npm run proxy   # local-ssl-proxy: https://localhost:3001 -> 3000
   ```

   Abre `https://localhost:3001/new?recipient=<YOUR_MERCHANT_ADDRESS>&label=Wavelength%20Records`, acepta el certificado firmado localmente, y checkpoint: deberías ver el teclado numérico con el nombre de tu puesto arriba. Teclea un monto y genera un código para ver el flujo de transfer request de fábrica una vez. Conocer el comportamiento de fábrica hace visible el cambio del paso 3.

2. **Pon checkout-txreq detrás de https.** Las billeteras exigen https para los links de transaction request, y tu endpoint de la lección pasada corre en http pelado localmente (el mío escucha en 3100; sustituye tu puerto). Ponle un proxy igual que el POS se lo pone a sí mismo:

   ```bash
   npx local-ssl-proxy --source 3443 --target 3100
   ```

   Checkpoint: `curl -k https://localhost:3443/txreq` devuelve la respuesta GET de tu endpoint, el JSON de label e icon de la prueba de humo de la lección pasada. La bandera `-k` se salta la comprobación de confianza del certificado, y el navegador no se la va a saltar: abre `https://localhost:3443/txreq` en el navegador también y acepta ahora el certificado autofirmado, o el fetch de la página del POS se muere después con ERR_CERT_AUTHORITY_INVALID antes de que el flujo de la venta llegue a empezar.

3. **Mueve el interruptor.** En `src/client/components/pages/App.tsx`, haz las dos ediciones de la sección de teoría:

   ```tsx
   const connectWallet = true;  // browser-wallet dev loop for this lab

   // Toggle comments on these lines to use transaction requests instead of transfer requests.
   // const link = undefined;
   const link = useMemo(() => new URL('https://localhost:3443/txreq'), []);
   ```

   Después una tercera edición, una que la sección de teoría no cubrió porque es sobre el token, no sobre el link: la app de fábrica viene configurada para SOL nativo. Acuérdate del orden de la sección de la pantalla de confirmado, encontrada, después validada. El POS va a encontrar tu firma, después va a validar la transacción aterrizada contra su propia config, y un `validateTransfer` configurado para SOL que comprueba el `TransferChecked` de USDC que construye tu endpoint la rechaza y pinta **Invalid**. En el mismo archivo, agrega `USDCIcon` a los imports (el componente ya viene en el ejemplo, al lado de `SOLIcon`):

   ```tsx
   import { USDCIcon } from '../images/USDCIcon';
   ```

   y en el `<ConfigProvider>` más abajo, agrega `splToken` y cambia las cuatro props con sabor a SOL por sus valores de USDC (`address` ya está importado arriba del archivo):

   ```tsx
   splToken={address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU')}
   symbol="USDC"
   icon={<USDCIcon />}
   decimals={6}
   minDecimals={2}
   ```

   Agrega la rama del puesto de la sección de teoría (`priceFromStallLink`) a tu handler de POST de checkout-txreq si todavía no lo hiciste. Checkpoint: recarga la página del POS, teclea un monto, genera, y el QR ahora codifica `solana:https://localhost:3443/txreq?amount=...&reference=...`. Aparece la pantalla de pending y el log de tu endpoint muestra el GET entrando.

4. **Cobra una venta en devnet.** Con `connectWallet = true`, paga desde una billetera de navegador en la misma máquina (fondeada con SOL de devnet para las comisiones y USDC de devnet del faucet de Circle, como en el módulo 2; la transacción que construye tu endpoint es el mismo `TransferChecked` de USDC que la del carrito web). Aprueba la transacción que el POS te entrega. Checkpoint: el POS pasa de pending a confirmado y el anillo de progreso se completa; la firma en sí vive detrás de "Recent Transactions", no en la pantalla de confirmado. El log de tu endpoint muestra el POST `{account}` y la transacción en base64 que devolvió. Esa firma liquidó una venta a la que tu servidor le puso precio desde el teclado. La página web acaba de cobrar dinero del otro lado de una mesa. Si en cambio el anillo dice **Invalid**, `validateTransfer` rechazó la transferencia: comprueba que `splToken` esté puesto en App.tsx y que tu memo vaya antes de la transferencia en `finalizeTransaction`. Y si el anillo nunca sale de pending mientras el log de tu endpoint muestra GETs pero ningún POST, el navegador bloqueó el fetch cross-origin: el middleware de CORS del servidor de la lección pasada tiene que estar en el proceso que corre detrás del proxy 3443 — `local-ssl-proxy` reenvía headers, no los agrega.

5. **Arma el andamiaje del artefacto.** El artefacto del curso para esta lección es `pos-stall`, un directorio delgado que fija la configuración de tu puesto y la demuestra con una prueba de humo, sentado al lado de `checkout-txreq` en tu workspace de Wavelength:

   ```bash
   mkdir pos-stall && cd pos-stall
   npm init -y
   npm pkg set type=module
   npm install @solana/pay@1.0.26 @solana/kit@^6.10.0 qrcode
   npm install -D tsx typescript @types/node @types/qrcode
   ```

   La línea `type=module` importa: `@solana/pay` publica sus tipos de TypeScript bajo su entrada ESM, y un paquete ESM es lo que deja que `tsc --strict` los resuelva sin ruido (yo mismo me llevé el error de declaración faltante antes de agregarla, así que considera esos diez minutos donados). Notas de pins, comprobadas el 2026-08-22: `@solana/pay` 1.0.26 es `latest` en npm (publicado el 2026-07-31) y tiene pares en kit ^6.9, así que `@solana/kit@^6.10.0` mantiene esto en el workspace v6 del curso; `qrcode` (1.5.x en el momento de la comprobación) renderiza códigos QR en una terminal pelada de Node, que es algo que la biblioteca de estilos atada al navegador que usa el POS no puede hacer.

6. **Escribe la config, con los TODOs de completion dejados abiertos.** Crea `pos-stall/config.ts`:

   ```typescript
   export interface CartLine {
     sku: string;
     title: string;
     priceUsdc: number; // UI units with cents precision, e.g. 12.5
   }

   export const STALL: {
     label: string;
     recipient: string;
     txreqLink: string;
     cart: CartLine[];
   } = {
     label: 'Wavelength Records',
     // TODO(completion): your merchant wallet, the same recipient checkout-txreq pays
     recipient: '11111111111111111111111111111111',
     // TODO(completion): where your transaction-request endpoint lives (https, /txreq path)
     txreqLink: 'https://localhost:3443/txreq',
     // TODO(completion): the crate you are selling today, priced in devnet USDC
     cart: [],
   };
   ```

7. **Escribe y corre la prueba de humo.** Crea `pos-stall/smoke.ts`. Hace exactamente lo que hace el POS por venta, acuñar una reference, agregar `amount` y `reference` al link, codificar, así que una corrida que pasa demuestra que tu config produciría un código QR escaneable del puesto:

   ```typescript
   import { encodeURL } from '@solana/pay';
   import { generateKeyPairSigner } from '@solana/kit';
   import QRCode from 'qrcode';
   import { STALL } from './config.js'; // .js extension: ESM resolution rule, even from .ts

   async function main(): Promise<void> {
     const link = new URL(STALL.txreqLink);
     if (!link.pathname.endsWith('/txreq')) {
       throw new Error(`POS must point at the transaction-request endpoint, got ${link.pathname}`);
     }
     if (link.protocol !== 'https:') {
       throw new Error(`transaction requests require https, got ${link.protocol}`);
     }

     // Summed in integer cents so no float drift ever reaches the QR amount;
     // the URL speaks UI units (decimal USDC), same convention as every
     // Solana Pay amount this module has written.
     const totalCents = STALL.cart.reduce(
       (sum, line) => sum + Math.round(line.priceUsdc * 100),
       0,
     );
     if (totalCents <= 0) {
       throw new Error('cart is empty; fill the completion TODOs in config.ts first');
     }
     const totalUsdc = (totalCents / 100).toFixed(2);

     // one fresh reference per sale, exactly as the POS mints one per payment
     const referenceSigner = await generateKeyPairSigner();
     link.searchParams.append('amount', totalUsdc);
     link.searchParams.append('reference', referenceSigner.address);

     const url = encodeURL({ link });
     console.log(await QRCode.toString(url.toString(), { type: 'terminal', small: true }));
     console.log(`POS points at /txreq; QR generated for the cart total (${totalUsdc} USDC, ${STALL.cart.length} items)`);
   }

   main().catch((error) => {
     console.error(error instanceof Error ? error.message : error);
     process.exit(1);
   });
   ```

   Corre `npx tsx smoke.ts`. Con los TODOs todavía abiertos falla con `cart is empty; fill the completion TODOs in config.ts first`, que es lo correcto: la prueba de humo que falla es la lista de pendientes de tu peldaño de completion. (Este archivo exacto, con estos pins exactos, pasa el chequeo de tipos bajo `npx tsc --strict --noEmit smoke.ts config.ts` y corre; si a ti no, la extensión del import y la línea `type=module` del paso 5 son los dos sospechosos de siempre.)

![Diagrama de despliegue del puesto: una laptop corriendo el POS y el endpoint detrás de proxies SSL locales, un teléfono de cliente alcanzándolos por la LAN, y devnet liquidando la transacción.](assets/v08-diagram.png)

## Challenge

Dos peldaños, y el segundo es la comprobación de verdad de la lección.

**Completion.** Llena los tres TODOs en `pos-stall/config.ts`: el destinatario de tu comercio, el link https de tu endpoint y un carrito real (tres o cuatro prensados con precios en USDC alcanza). Aceptación: `npx tsx smoke.ts` imprime un código QR en la terminal y la línea `POS points at /txreq; QR generated for the cart total`, y el monto codificado es igual a la suma de las líneas de tu carrito. Si la prueba de humo rechaza tu link, lee su error antes de tocar código: los dos modos de falla que comprueba (camino equivocado, http pelado) son los dos que matan ventas calladamente en una mesa real.

**Solo.** Corre una venta completa al estilo en persona de punta a punta y registra el comprobante. Al estilo en persona quiere decir que el QR cruza el aire: una billetera de teléfono escaneando la pantalla de tu laptop. Cambia `localhost` por la dirección LAN de tu laptop tanto en el link de App.tsx como en tu configuración del proxy para que el teléfono pueda alcanzar el endpoint, y espera fricción del certificado autofirmado, las billeteras de teléfono son más estrictas con los certificados que tu navegador de escritorio (este es el costo honesto de un lab de https local; un endpoint desplegado con un certificado de verdad lo hace desaparecer — un despliegue que este curso te deja a ti, ya que incluso el capstone se queda a propósito en una sola máquina). Si tu billetera de teléfono rechaza el certificado de plano, el camino de la billetera de navegador del paso 4 del lab sigue siendo tu respaldo para la venta liquidada; dilo en tu comprobante. Aceptación: un código QR escaneado liquida un carrito en devnet a través del POS, y tu comprobante registra el monto tecleado, la reference que acuñó el POS y la firma liquidada. Ese artefacto de venta completada, no un quiz, es la barrera de esta lección.

Antes de que desarmes la mesa, fíjate en lo que no construiste hoy: un frontend. Toda la superficie vino del ejemplo propio del repo pay, y cada línea que de verdad escribiste o lo configuró o extendió el endpoint que ya tenías. Esa es la proporción correcta para el trabajo de comercio, y vale la pena sentirla al menos una vez: infraestructura que alguien más mantiene, lógica de precios que es tuya. Si la venta se liquidó, estás adelante de la mayoría de las propuestas de "crypto POS" que cruzaron una mesa de demo este año, y si algo en el flujo te dio pelea, escribe dónde mientras la memoria está fresca.

Tu endpoint ya vendió un disco a través de una página web y del otro lado de una mesa plegable, dos superficies, un solo núcleo de pagos. Pero una pantalla de puesto todavía hace que el cliente venga a ti. El mismo endpoint puede ser más que eso: puede ser un link que sueltas en redes y que ES la tienda, ejecutando la compra donde sea que se publique. La próxima lección lo conviertes en un blink, y encaramos, con la misma honestidad que esta lección te debía sobre el hardware, dónde renderizan de verdad los blinks.
