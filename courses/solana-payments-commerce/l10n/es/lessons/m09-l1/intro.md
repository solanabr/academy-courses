# Ensamblaje: una tienda, todos los rieles

## Resumen

La lección pasada pusiste todo el stack frente a una checklist de salida al aire y la dejaste fallar en el papel: cada fila roja se volvió una tarea de arreglo, y trabajaste la lista hasta que cada pieza fue individualmente digna de producción. Esa fue la auditoría. Lo que no pudo probar es la cosa para la que existe esta lección. Quince peldaños, cada uno verde en su propio rincón del repo con su propia prueba de humo, no son una tienda. A un comprador no le importan tus carpetas. ¿Puede entrar un cliente de verdad con cero SOL, comprar un disco, que se lo despachen, suscribirse, sobrevivir a una renovación fallida y llevarse un reembolso, todo sin que toques una sola cosa a mano?

Hoy contestas esa pregunta con un archivo de log. Antes de cualquier teoría, haz inventario. Desde la raíz de tu repo:

```bash
find . -mindepth 2 -maxdepth 3 -name package.json -not -path '*/node_modules/*' | wc -l
```

Cuéntalos (mindepth se salta el manifiesto de la raíz, y el nivel extra de profundidad atraparía cualquier manifiesto anidado una carpeta más abajo; en este árbol no debería encontrar ninguno, porque cada workspace está en el nivel de arriba). Cada uno de esos manifiestos es un workspace que construiste y demostraste — la mayoría de los peldaños se llevó uno propio, aunque no todos, y la nota del roster de abajo nombra los que viven dentro de otro workspace o en nada de código — y el conteo vuelve en catorce, uno menos que el roster de abajo porque el workspace `stack` de esta noche todavía no existe. Después abre tu `gate/report.md` más reciente al lado del conteo: las tareas de arreglo que cerraste la lección pasada son la razón por la que esta noche puede ser aburrida. Ninguna de ellas, sola, puede venderle un disco a un desconocido. Esa brecha entre "todas las pruebas en verde" y "un negocio funciona" es la última habilidad que enseña este curso y, honestamente, es por la que a los ingenieros de integración les pagan.

Los hallazgos por delante:

- Entregas **wavelength-stack**: un repo donde cada ruta importa transfer-kit como el core de pago compartido, las superficies de transaction-request, blink y x402/MPP están montadas en UN solo servidor (la página de checkout con QR del módulo 3 se queda como página estática standalone a propósito; su camino de pago es el mismo core de transaction-request que ejercita la superficie montada), el worker del webhook y el crank de facturación corren como procesos de fondo, y `npm run journey` maneja un journey de comprador guionado de siete tramos contra devnet.
- No hay conceptos nuevos en esta lección. A propósito. Cada línea estructural es un import de algo que ya construiste, y el trabajo entero de la lección es hacer eso visible: el blink reusa el constructor de transacciones del módulo 3, los ids de factura del memo de x402 se concilian en el mismo libro mayor del back office que tus checkouts, y el verificador del módulo 4 es el único juez de cada tramo.
- Los dos pins de kit de la lección de suscripciones sobreviven al ensamblaje intactos: los workspaces wavelength-checkout, backoffice y x402 se quedan en su línea kit-6 y el workspace subscriptions se queda en kit 7.1.1. Esta noche es la noche en que por fin comparten un árbol, porque el ensamblaje registra `subscriptions` en el roster de la raíz — así que espera el `ERESOLVE`, y entiende que lo que protege al crank en runtime es dónde vive su archivo de entrada, nunca la frontera del workspace.
- La barrera es brutal y simple: siete líneas PASS del verificador, código de salida 0, y la checklist de prod-gate del módulo 8 re-puntuada contra el stack ensamblado, cada fila o pasando o cargando una tarea de arreglo escrita.

## Prueba de sonido: quince peldaños, una noche de estreno

Piensa en esta noche como la noche de estreno de una sala. Cada instrumento llegó en su propio estuche y pasó su propia prueba de banco. La prueba de sonido no es sobre ningún instrumento; es sobre si la sala funciona cuando todo suena a la vez. Igual aquí: el ensamblaje es una disciplina propia, con sus propios modos de falla, y ninguno de ellos vive dentro de un solo peldaño.

![Diagrama de arquitectura de tres procesos: un servidor montando las superficies de transaction-request, gasless, blink y x402, un worker de webhook, y un crank de subscriptions aislado como la isla kit-7, todos compartiendo transfer-kit y un solo libro mayor de pedidos.](assets/v01-diagram.webp)

### El journey del comprador es la spec

Lo que está en juego, dicho como una diferencia entre dos resultados. Si el stack ensamblado funciona, el dinero de un desconocido se vuelve un pedido despachado, una suscripción corriendo y una fila auditable del libro mayor mientras duermes. Si casi funciona, te llevas el peor resultado del comercio: el dinero llega y no pasa nada, y ahora un humano tiene que conciliar a mano aquello en lo que tus sistemas no están de acuerdo. El journey de siete tramos existe para hacer que "casi funciona" sea imposible de esconder.

Los tramos, en el orden en que el script los corre. Un comprador, guionado, en devnet:

1. **Ramp stub.** El comprador arranca desde fiat. Coinbase no va a dar de alta una cuenta de prueba headless, así que este tramo afirma el contrato de session-token que construiste en la lección de onramp: existe una URL de pay.coinbase.com, fija `defaultNetwork=solana`, y la dirección de la billetera no aparece en ninguna parte de ella.
2. **Primera compra patrocinada por Kora.** El comprador tiene USDC de devnet y cero SOL, y compra un disco igual. El fee payer en la transacción aterrizada es el firmante de Kora, no el comprador. Un comprobante, y el diagrama del paymaster del módulo 8 es real.
3. **Pedido despachado por webhook.** Una segunda compra aterriza por el flujo de transaction-request, y el evento de Helius llega a tu worker por el mismo túnel que expusiste en la lección de webhooks; arrancar ese túnel le pertenece al ritual de arranque de la terminal uno, no a manos a mitad de corrida. Después el script del journey repite el payload entregado directo contra el endpoint del worker dos veces más, el sustituto local de la reentrega de Helius, y el pedido se despacha exactamente una vez, incluida la venta que tu fila de la feria drenó al arrancar.
4. **Ciclo de suscripción más dunning forzado.** El comprador se une al club del disco del mes, un pull de facturación aterriza como una factura conciliada, y después el script vacía la ATA del comprador y fuerza a que una renovación falle. La falla tiene que volverse una factura abierta. No un reintento.
5. **Compra por blink.** El blink del drop sirve sus metadatos, toma `{account}`, y devuelve una transacción construida por el constructor exacto del módulo 3.
6. **El agente le paga a la API tres veces.** Un agente que paga le pega al endpoint de precios de prensado, se come el 402, liquida, y lo hace dos veces más. Tres ids de factura de memo se concilian en el libro mayor.
7. **Un reembolso.** Un pago push inverso a través de transfer-kit, registrado contra la firma de origen.

![Diagrama de flujo de los siete tramos del journey desde el ramp stub hasta el reembolso, cada uno alimentando al verificador compartido del lado del servidor que comprueba el programa de tokens, el mint, el delta de saldo y el memo antes de imprimir PASS.](assets/v02-flowchart.webp)

El journey no es un recorrido de UI, y ningún tramo confía nunca en un toast de billetera, un payload de webhook o una respuesta 200 como prueba. El curso tiene un solo harness de aceptación, el verificador de m04, y el journey lo llama una vez por tramo: vuelve a traer la transacción con `getTransaction`, comprueba el programa de tokens, después el mint, después el delta de saldo en la cuenta de token PROPIEDAD del comercio (con clave en el owner, la manera en que el verificador le ha puesto clave desde m04 — esa elección es lo que atrapa el fixture de mint equivocado), después el memo. Una transacción patrocinada recibe el mismo trato que una común. La co-firma de Kora cambia quién pagó la comisión; no cambia nada sobre qué merece que se le crea.

### La distribución, y la costura que ya resolviste

La forma objetivo es un solo monorepo de npm cuyo `package.json` de la raíz lista cada peldaño como un workspace. Has estado construyendo hacia esto desde el módulo 2 sin ceremonia; el ensamblaje solo hace explícito el roster. Y una anticipación, porque un compañero de mente ordenada va a proponer, con toda seguridad, colapsar todo en un solo workspace sobre una sola versión de kit mientras andas por ahí. Niégate, con educación, con los rangos de peers en la mano: `@solana/pay` 1.0.26 tiene como peer a kit ^6.9 y `@solana/subscriptions` 0.5.0 tiene como peer a kit ^7.0.0, y los dos están correctos. Esa costura fue el punto entero de la lección de suscripciones, y no se vuelve a derivar aquí: los workspaces wavelength-checkout, backoffice y x402 se quedan con sus pins de kit-6 y el workspace subscriptions se queda con kit 7.1.1.

Sé exacto sobre qué los mantiene separados, eso sí, porque la lección de suscripciones fue directa en esto y esta noche es la noche en que se pone a prueba. Los workspaces registrados de npm **no** son aislamiento. npm hace hoisting de cada paquete registrado hacia una única resolución compartida en la raíz, así que en el momento en que el roster de abajo nombre `subscriptions`, los dos majors de kit están en un solo árbol y npm tiene que reconciliarlos — que es precisamente el `ERESOLVE` que el paso de instalación de abajo espera en vez de tener la esperanza de evitar. Hasta ahora `subscriptions/` estaba fuera del roster y nunca llegó siquiera a encontrarse con el árbol de kit-6; registrarlo te compra scripts `--workspace` y un solo lockfile, y te cuesta esa reconciliación. Lo que sobrevive a la fusión es el pin, no una pared: npm estaciona un major en la raíz y anida el otro bajo `subscriptions/node_modules`, que es lo que verifica el checkpoint de abajo, y Node después resuelve los imports del crank desde ahí por una sola razón — ahí es donde vive su archivo de entrada. El paso 4 dice lo mismo sobre el proceso del crank, y vale la pena leerlo dos veces, porque "workspace distinto" y "proceso distinto" son los dos la respuesta equivocada a por qué la isla se sostiene.

Esos rangos de peers se volvieron a verificar contra npm en la lección de suscripciones el 2026-08-22; corre `npm view @solana/subscriptions@0.5.0 peerDependencies` tú mismo antes de instalar nada hoy, porque este rincón de npm se ha movido dos veces este trimestre y nunca, en ningún lado, fijes a `latest`.

![Diagrama del monorepo listando las quince carpetas de workspace con el workspace subscriptions aislado como la única isla kit-7 y el nuevo workspace stack resaltado del lado kit-6.](assets/v03-diagram.webp)

No todos los peldaños se llevaron su propia carpeta, y vale la pena decirlo en voz alta: el embed del ramp vive dentro del workspace wavelength-checkout porque creció a partir de ese servidor, el gate de MPP es un archivo de config parado delante del workspace x402 en vez de un código propio, y el registro de decisión de corredores es un documento, no un proceso. Los peldaños son capacidades, no directorios. Tu roster puede diferir del mío en los nombres; el array workspaces es la fuente de verdad, y tiene que listar lo que de verdad construiste.

### Importa, no reimplementes

Aquí está la acumulación, mostrada en vez de afirmada, porque una afirmación como "todo compone" es exactamente el tipo de cosa que este curso te ha enseñado a no tomar por fe. Tres comprobantes:

**transfer-kit es el core de pago, por import.** El constructor de txreq llama a su `resolveAta` y acuña sus reference keys; el constructor de reembolsos emite su push inverso por `sendStablecoin`; los pulls del crank liquidan en ATAs que él resuelve. Un solo módulo, escrito en la semana uno, moviendo cada dólar del stack esta noche. Si te encuentras volviendo a tipear un checked-transfer en cualquier parte de este lab, para; estás reimplementando tu propia dependencia.

**El blink nunca aprendió a construir una transacción.** Su handler de POST llama a `buildOrderTransaction` del workspace checkout-txreq del módulo 3 y envuelve el resultado en un `ActionPostResponse`. Mismo catálogo, misma función de precios, mismo formato de memo. Cuando el tramo de blink del journey pasa el verificador sin nada de código de pago específico del blink en el diff, eso es la escalera de artefactos rindiendo.

**Tres protocolos, un solo libro mayor.** Un checkout con QR, un pull de suscripción y una llamada de agente x402 son puertas de entrada salvajemente distintas, y cada una de ellas aterriza como una fila en el mismo libro mayor de pedidos del back office, con la misma clave. Los ids de factura de `extra.memo` del tramo de x402 (256 bytes máximo, de la spec v2 de x402) se concilian por el mismo camino que un memo de checkout. Una sola historia de conciliación para todo el negocio.

![Línea de tiempo mostrando los artefactos de los módulos dos a ocho, cada uno alimentando el ensamblaje final de wavelength-stack, desde transfer-kit como el core compartido hasta la checklist de prod-gate al final.](assets/v04-timeline.webp)

### El harness es delgado a propósito

El script del journey es deliberadamente aburrido: no hagas spawn de nada sofisticado, maneja cada tramo, y termina cada tramo de cadena de la misma manera, con una sola llamada al verificador y un error lanzado ante cualquier veredicto que no sea ok. La tentación a la hora del capstone es construir un framework de pruebas ingenioso. Resístela. Un driver delgado sobre un juez confiable vale más que un framework rico que escribiste la noche antes de la demo, porque cuando un tramo falla a las 2am quieres que la falla sea sobre el stack, nunca sobre el harness. Una nota de convención antes del código: cada import entre workspaces en estos archivos es una ruta de TypeScript sin extensión, que resuelve solo porque todo el stack corre bajo tsx; apúntale `node` pelado a cualquiera de ellos y los imports fallan, lo cual es esperado, no roto.

La misma austeridad aplica a los dos loops de fondo. El worker y el crank son los procesos exactos que ya construiste; el stack no los envuelve, no les hace monkey-patch, ni los fusiona. Los arranca. La única regla de integración que los dos tienen que honrar es la guarda de idempotencia de la lección de webhooks: el worker toma el claim de una firma antes de despachar y el crank escribe las facturas por el mismo camino de claim, porque una entrega de Helius reintentada o un tick de crank vuelto a correr contra un stack sin esa guarda despacha un pedido dos veces, y el tramo de webhook del journey está construido para atrapar exactamente eso.

Hay una arruga genuinamente nueva que el harness tiene que respetar, y es sobre el tiempo, no sobre el dinero. Hasta ahora cada prueba de humo que escribiste afirmaba una cosa que tu propio código acababa de hacer: manda, después comprueba. Dos de los tramos de esta noche afirman cosas que un *proceso distinto* hace según su propio calendario. La fila del libro mayor del tramo de webhook aparece cuando llega la entrega de Helius y el worker termina de verificar, que desde el asiento del script del journey es un número impredecible de segundos después de que aterriza el pago. La factura abierta del tramo de dunning aparece cuando el siguiente tick del crank nota el pull fallido. Si el script afirma en el instante en que su transacción confirma, va a hacer fallar un stack que funciona perfectamente, solo que funciona de forma asíncrona. Así que el harness lleva una segunda primitiva de espera al lado del wrapper de reintentos: un poll acotado que vigila que una condición se vuelva verdadera y se rinde ruidosamente después de un deadline. El reintento es para lecturas que dan error; el polling es para efectos que todavía no pasaron. Mantener los dos separados mantiene honestos tus mensajes de falla, porque "el RPC dio timeout" y "el worker nunca despachó el pedido" son bugs distintos con dueños distintos.

### Un comprador, saldos preparados

Lee otra vez el orden de los tramos y vas a notar que no es arbitrario; el journey es una pequeña máquina de estados sobre los saldos de un solo comprador, y cada tramo a la vez afirma algo y prepara el siguiente. El script acuña un keypair de comprador fresco al arrancar (persístelo en `/tmp/buyer.json` y exporta `BUYER_ADDRESS` desde ahí, para que tanto la comprobación de fuga como tus comandos de shell a mitad de debug puedan alcanzarlo), le financia la ATA con USDC de devnet desde el flujo de faucet que montaste en el módulo 2, y deliberadamente no le da nada de SOL en absoluto. Esa pobreza es el punto del tramo 2: la compra patrocinada por Kora tiene que tener éxito desde una billetera que no podría pagar su propia comisión base, y la afirmación de que el delta de lamports del comprador es exactamente cero solo quiere decir algo si el saldo era cero de entrada. Después de que pasa el tramo patrocinado, el script le recarga al comprador un pequeño airdrop de SOL, porque del tramo 3 en adelante el comprador se firma y se paga solo como cualquier cliente común. La forma de CLI de esa recarga, si quieres hacerle a mano una comprobación de cordura a un comprador atascado a mitad de debug:

```bash
solana airdrop 0.1 $(solana-keygen pubkey /tmp/buyer.json) --url devnet
```

La coreografía sigue hasta el fondo. El tramo de suscripción drena la ATA del comprador a propósito para forzar la renovación fallida, lo que quiere decir que el tramo de blink que sigue tiene que volver a financiar USDC primero o fallaría por la razón equivocada, un rechazo por underpaid nacido de tu propia coreografía de pruebas y no de una superficie rota. Un tramo que falla por la razón equivocada es peor que un tramo que falla honestamente; te manda a debuguear una superficie que funciona. Así que el cuerpo de cada tramo abre preparando el estado de saldo exacto que necesita y cierra afirmando el estado que creó. Escribe las líneas de preparación con el mismo cuidado que las afirmaciones. Cuando una corrida se pone roja a las 2am, la primera pregunta siempre es "¿falló el tramo, o mintió la preparación anterior?", y un script de journey que loguea sus pasos de preparación contesta eso solo desde el log.

Una elección deliberada más: el journey nunca reusa ids de pedido entre corridas. Cada corrida estampa un run id fresco en sus ids de pedido y sus strings de memo, así que volver a correr el journey contra un libro mayor que ya tiene las filas de ayer afirma solo las filas de esta corrida. El libro mayor es historia de solo-agregar; el journey es el recorte de una noche de ella. Ponles clave a tus afirmaciones sobre el run id y el script se vuelve re-corrible con seguridad para siempre, que es exactamente lo que quieres de la cosa de la que vas a hacer la demo con las manos sudadas.

### Qué se rompe a la hora del ensamblaje

Cuatro modos de falla explican la mayor parte del dolor de este lab, y te los entrego por delante porque, en mi experiencia con las semanas de integración, eso cambia el debugging de horas a minutos.

![Tabla que empareja cuatro trampas del ensamblaje, contaminación cruzada de kit, guarda de idempotencia faltante, fila de la feria sin drenar y límites de tasa de devnet, con sus síntomas observables y sus arreglos.](assets/v05-comparison.webp)

La última fila merece una oración extra, porque es la que engaña a la gente bajo presión de demo: el RPC público de devnet te va a limitar la tasa a mitad del journey, y una lectura que da timeout se ve exactamente igual que un tramo que falló. La distinción que importa es *qué lado dijo que no*. Un timeout es la infraestructura de lectura encogiéndose de hombros; lo reintentas. Un rechazo del verificador es tu harness de aceptación hablando; eso nunca lo reintentas, lo investigas.

### La contrapartida, y quién ya corre sobre estos rieles

Nombra la contrapartida antes del lab, como siempre. El monorepo ensamblado corre cada servicio en un solo árbol de procesos sobre una sola máquina contra devnet, y eso es exactamente lo correcto para un capstone de enseñanza y lo equivocado para producción. Un despliegue de verdad separa el worker, el crank y la API con paywall en servicios separados y de vida larga, con su propio monitoreo y sus propias políticas de reinicio, y nunca comparte un solo firmante entre todos ellos: el radio de impacto de una llave filtrada debería ser un servicio, no toda tu tienda. El capstone demuestra el cableado y la disciplina de verificar-del-lado-del-servidor. No demuestra una postura de ops, y las disciplinas más profundas de aterrizaje e indexado que necesita una versión de alto volumen son territorio del curso Client-Side Mastery, como lo han sido cada vez que este curso las tocó.

![Comparación del stack de enseñanza contra un despliegue de producción a través de procesos, firmantes, monitoreo y red, terminando con las invariantes que se trasladan, verificación del lado del servidor, idempotencia, un solo libro mayor y pins por workspace.](assets/v06-comparison.webp)

¿Algo de esto es real fuera de un repo de curso? Sip, y con números. Helius corre su propia facturación sobre el mismo programa oficial de Subscriptions que integraste, el programa `De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44` (el prefijo vanity `De1eg` nombra al programa de Delegation on-chain sobre el que corre el producto Subscriptions, el mismo nombrado que usan los códigos de error de su cliente), y declara su política de dunning con las mismas palabras que codifica tu máquina de estados: una renovación fallida no se reintenta contra la billetera, se vuelve una factura abierta (su blog de ingeniería, traído el 2026-08-21). Tu tramo de falla forzada afirma el comportamiento exacto sobre el que una empresa de infraestructura de verdad apuesta sus ingresos. Y el tramo del agente tampoco es especulativo: el dashboard de x402.org, la misma ventana móvil de 30 días que leíste en la lección de x402, reportó 75.41M transacciones y 24.24 millones de dólares en volumen cuando lo consulté el 2026-08-21. Los rieles a los que les estás haciendo la prueba de sonido esta noche están cargando peso real en el mundo, a escala de juego, ahora mismo.

Ya que estamos contando, cuenta el costo de la noche misma, porque el número todavía sorprende a la gente que ha vivido sobre rieles de tarjeta. El journey completo aterriza en unas diez transacciones: dos compras, una compra por blink, un alta de suscripción y su pull, tres llamadas de agente, un reembolso, más la venta drenada de la fila. Cada una lleva la comisión base de 5000 lamports y, según la banda con la que abrió este curso, se liquida por una pequeña fracción de un centavo — $0.00075 a $150 por SOL, $0.0015 a $300, y tú citas la banda en vez de una cifra horneada. Un solo renglón los empequeñece a todos y no es una comisión: el rent de la ATA del tramo patrocinado, alrededor de 0.0015 SOL a la tasa de devnet del 2026-09-07 y todavía cayendo, el gasto presupuestado al que tu fila de prod-gate ya le pone tope. Comisiones aparte, la noche entera de siete tramos, un onboarding, cuatro ventas, un ciclo de facturación, un cliente máquina y un reembolso, cuesta menos en comisiones de red que el error de redondeo de una sola transacción con tarjeta. Esa aritmética es por lo que cada riel de este stack puede existir siquiera, y vale la pena tenerla lista la próxima vez que alguien pregunte por qué una tienda de discos se molestaría.

## Lab: levanta el stack

Este es el último lab del curso, así que déjame decir la parte callada en voz alta: el apoyo se acabó. El módulo 8 ya empezó a quitártelo, entregándote un runner del gate y cuatro filas terminadas y haciéndote escribir el resto. Esta noche se va el resto del camino. Te llevas el orden de cableado, las dos piezas de pegamento que son genuinamente nuevas (el supervisor de procesos y el harness del journey), y un tramo trabajado como el patrón. Todo lo demás lo ensamblas desde tus propios peldaños, porque después de esta lección no hay repo de curso en el que apoyarte, solo tu repo.

**1. Unifica los workspaces.** Tu `package.json` de la raíz creció orgánicamente desde el módulo 2. Vuélvelo el roster explícito. El mío:

```json
{
  "name": "wavelength",
  "private": true,
  "type": "module",
  "workspaces": [
    "transfer-kit",
    "wavelength-checkout",
    "checkout-txreq",
    "pos-stall",
    "drop-blink",
    "verifier",
    "backoffice",
    "backoffice-refunds",
    "club-crank",
    "subscriptions",
    "dunning",
    "x402",
    "gasless-checkout",
    "fair-queue",
    "stack"
  ],
  "scripts": {
    "journey": "npm run --workspace stack journey"
  }
}
```

Después crea el único workspace nuevo y reinstala el árbol:

```bash
mkdir -p stack/src
npm init -y --workspace stack
npm pkg set type="module" --workspace stack
npm install --workspace stack express@5.1.0 @solana/kit@6.10.0
npm install --workspace stack -D tsx@4 typescript @types/express @types/node
npm pkg set scripts.serve="tsx src/server.ts" scripts.boot="tsx src/boot.ts" scripts.journey="tsx src/journey.ts" --workspace stack
npm install
```

Pins, con sus notas de frescura: `express` se queda en 5.1.0 para que todo el repo compile contra una sola versión (la 5.x actual de npm es 5.2.1 al 2026-08-23; resiste la actualización hasta que puedas subir todos los workspaces juntos), y `@solana/kit` 6.10.0 es el último release de v6, el mismo par que ya lleva cada workspace de kit-6 del repo. `tsx` es el runner que has usado todo el curso; la línea de dev-install es su instalación para este workspace fresco. Espera que ese último `npm install` falle, y lee la falla en vez de manotear una bandera por reflejo. Registrar `subscriptions` pone kit 6 y kit 7 en un solo árbol por primera vez en este curso, así que npm reporta el `ERESOLVE` entre ellos exactamente como la lección de suscripciones dijo que lo haría; la vía de escape es la que ya usaste en la lección de gasless, `npm install --legacy-peer-deps` en la raíz. Después el checkpoint: `npm ls --workspaces --depth 0` imprime cada workspace, y `subscriptions` es el único árbol que muestra kit 7.1.1 — anidado bajo su propio `node_modules` en vez de estar hoisteado, que es el mecanismo que describió la sección de la costura. Confirma que cada workspace todavía resuelve la línea de kit que fija su propio `package.json` antes de seguir más adelante; si algún workspace de kit-6 ahora reporta 7.1.1, la bandera tapó un conflicto real y tienes que arreglar el pin, no la bandera.

**2. Exporta las apps, ponles barrera a los listens.** Cada workspace de superficie termina ahora mismo su archivo de servidor con un `app.listen` pelado. Importar un archivo así arrancaría un listener perdido, así que dale a cada uno la misma edición en dos partes: exporta la app, y escucha solo cuando se corre directamente. Aquí está sobre checkout-txreq; repítelo literal (con los nombres correctos) en drop-blink, el servidor de x402 y gasless-checkout — ese último es una superficie de verdad con sus propias rutas, no un camino dentro de otra app, y el tramo 2 del journey de esta noche la llama:

```typescript
// checkout-txreq/src/server.ts, the bottom of the file.
// Replace the bare app.listen call with an export plus a direct-run gate.
export { app as txreqApp };

const runDirectly =
  process.argv[1] !== undefined &&
  import.meta.url === new URL(`file://${process.argv[1]}`).href;

if (runDirectly) {
  app.listen(PORT, () => {
    console.log(`checkout-txreq listening on :${PORT}`);
  });
}
```

Un arreglo acompañante mientras andas en cada archivo: `express.static('public')` resuelve contra el directorio de trabajo del proceso, y esta noche un solo proceso sirve tres peldaños desde la raíz del repo. En vez de eso, ancla cada mount estático a la ubicación del archivo mismo:

```typescript
// near the top of each surface's server file
import { fileURLToPath } from 'node:url';

const publicDir = fileURLToPath(new URL('../public', import.meta.url));
app.use(express.static(publicDir));
```

Checkpoint: `npx tsx src/server.ts` dentro de cada workspace de superficie sigue arrancando esa superficie sola, exactamente como antes. La barrera quiere decir que nada cambió para las corridas standalone.

**3. Un solo servidor.** Ahora el montaje, y es más pequeño de lo que esperas, que es el punto:

```typescript
// stack/src/server.ts
import express from 'express';
import { txreqApp } from '../../checkout-txreq/src/server';
import { blinkApp } from '../../drop-blink/src/server';
import { x402App } from '../../x402/src/server';
import { gaslessApp } from '../../gasless-checkout/src/server';

const app = express();
const PORT = Number(process.env.PORT ?? 3000);

app.get('/healthz', (_req, res) => {
  res.json({ ok: true, surfaces: ['txreq', 'blink', 'x402', 'gasless'] });
});

// Express apps are middleware: mounting at the root preserves each
// surface's own paths, including actions.json at the domain root.
app.use(txreqApp);
app.use(blinkApp);
app.use(x402App);
app.use(gaslessApp);

app.listen(PORT, () => {
  console.log(`wavelength-stack listening on :${PORT}`);
});
```

Montar en la raíz importa para una superficie en particular: el `actions.json` del blink tiene que estar en la raíz del dominio o las billeteras nunca lo renderizan, y un mount en sub-ruta rompería en silencio la regla de hosting que aprendiste en la lección del blink. La superficie gasless se gana una línea propia aquí, y vale la pena saber por qué, porque es fácil recordarlo mal: el constructor patrocinado reusa `finalizeTransaction` de checkout-txreq, pero las rutas no viven ahí. `GET`/`POST /gasless` los servía su propia app de Express en el workspace `gasless-checkout`, en su propio puerto, y nada antes de esta noche los montó nunca en ningún otro lado. Sáltate la línea `app.use(gaslessApp)` y el tramo 2 del journey se lleva un 404 de un stack que por lo demás se ve sano. Checkpoint: `npm run --workspace stack serve`, después `curl localhost:3000/healthz`, `curl localhost:3000/txreq`, `curl localhost:3000/gasless` y `curl localhost:3000/actions.json` contestan todos desde un solo puerto.

Un prerrequisito que la superficie gasless carga y las otras no: habla con un nodo de Kora. Su constructor cotiza y co-firma contra `http://localhost:8080`, así que ese nodo tiene que estar corriendo antes de que arranque el journey, exactamente como era en la lección de gasless. No es uno de los tres hijos de boot.ts de abajo, porque es un binario externo y no un proceso que arrancas tú, al lado del pay gate en ese sentido.

![Mapa de rutas del servidor único en el puerto 3000 ramificándose hacia la comprobación de salud, la transaction request, la ruta gasless de Kora montada, las actions del blink montadas en la raíz, y la ruta de pago de x402 (el gate de MPP corre como un proceso separado y no está montado aquí).](assets/v07-diagram.webp)

Una palabra sobre la superficie más callada del módulo de protocolos, porque es fácil recordarla mal como si ya estuviera cableada. El camino del desafío de MPP NO viaja dentro de la app de x402: en el módulo 7 el desafío `WWW-Authenticate: Payment` lo servía el proceso `pay gate` separado, manejado por `paywall.yml` y proxeando un upstream libre de pago, y nada esta noche cambia esa arquitectura — exactamente el "archivo de config parado delante del workspace x402" de la nota del roster de arriba. boot.ts hace spawn de tres procesos, servidor, worker, crank, y un pay gate no es uno de ellos, así que el stack ensamblado habla x402 nada más. Si quieres el lado de MPP en vivo es una terminal más, no código nuevo: expón una ruta pelada de precios de prensado para que el gate la proxee (la ruta montada en x402 no puede ser su upstream, porque el gate exige una libre de pago), apunta `paywall.yml` hacia ella, y corre el gate en :4021 exactamente como en el módulo 7. Construiste para el riel que tiene tráfico; el que viene se queda a un comando documentado de distancia, que es la postura honesta para una spec de método de pago que todavía se mueve en su propio repo en vez de estar sentada en el reloj de algún organismo de estándares.

**4. Los loops de fondo.** El worker y el crank se quedan como procesos separados. Un modelo mental que hay que corregir antes de cablearlos, porque es común: no es la frontera del proceso, ni el directorio de trabajo, lo que preserva la costura de kit en runtime. Node resuelve un import pelado caminando hacia arriba desde la ubicación del *archivo que importa* hasta el `node_modules` más cercano, así que los imports del crank aterrizan en la isla kit-7 por exactamente una razón — `crank.ts` vive dentro de `subscriptions/`, cuyo propio `node_modules` tiene kit 7. Resolvería idéntico lanzado desde cualquier directorio, y un proceso spawneado con el cwd "correcto" que importara un archivo fuera de la isla igual se llevaría kit 6. Lo que el proceso separado te compra es aislamiento de ciclo de vida — un crank crasheado no puede derribar el servidor — no aislamiento de imports; eso lo hace la dirección del archivo de entrada. Primero apunta el workspace de cada loop a su archivo de entrada (los míos son `src/worker.ts` en backoffice y `src/crank.ts` en subscriptions; usa tus nombres de archivo de verdad):

```bash
npm pkg set scripts.start="tsx src/worker.ts" --workspace backoffice
npm pkg set scripts.start="tsx src/crank.ts" --workspace subscriptions
```

Después el supervisor:

```typescript
// stack/src/boot.ts
import { spawn, type ChildProcess } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// npm's --workspace flag only resolves from the repo root, and boot itself
// runs with cwd inside stack/, so every child is spawned from the root
// explicitly. npm then executes each script with the workspace itself as
// cwd -- a convenience for the scripts' relative paths, nothing more:
// import resolution never depends on cwd, only on where each entry file
// lives.
const REPO_ROOT = fileURLToPath(new URL('../..', import.meta.url));

interface Proc {
  name: string;
  workspace: string;
  script: string;
}

// Three long-lived processes. The crank's entry file lives inside the
// kit-7 island, so its imports resolve there, never against our kit-6 tree.
const PROCS: Proc[] = [
  { name: 'server', workspace: 'stack', script: 'serve' },
  { name: 'worker', workspace: 'backoffice', script: 'start' },
  { name: 'crank', workspace: 'subscriptions', script: 'start' },
];

const children: ChildProcess[] = [];

for (const proc of PROCS) {
  const child = spawn(
    'npm',
    ['run', '--workspace', proc.workspace, proc.script],
    { stdio: ['ignore', 'pipe', 'pipe'], env: process.env, cwd: REPO_ROOT },
  );
  children.push(child);

  const prefix = `[${proc.name}]`;
  child.stdout?.on('data', (chunk: Buffer) => {
    process.stdout.write(`${prefix} ${chunk.toString()}`);
  });
  child.stderr?.on('data', (chunk: Buffer) => {
    process.stderr.write(`${prefix} ${chunk.toString()}`);
  });
  child.on('exit', (code) => {
    console.log(`${prefix} exited (${code ?? 'signal'})`);
  });
}

function shutdown(): void {
  for (const child of children) child.kill('SIGTERM');
  process.exit(0);
}

process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
```

Checkpoint: `npm run --workspace stack boot` muestra tres líneas de arranque con prefijo, y Ctrl-C derriba las tres. Si el crank loguea un error de tipos de kit aquí, tienes la trampa de la contaminación cruzada: algo fuera de la carpeta subscriptions está importando desde adentro. El arreglo nunca es un cambio de versión; es borrar el import.

**5. Drena la fila antes de que abran las puertas.** La fila de la feria de la lección offline tiene al menos una venta firmada de tu última sesión de puesto. Corre tu drain (`cd fair-queue && npx tsx drain.ts`) y espera la línea de fila-vacía antes de siquiera arrancar el journey. La venta drenada fluye por el camino del webhook como cualquier otra compra, que es por lo que el tramo de webhook del journey la va a contar, y por lo que afirmar antes de que el drain termine es la trampa tres. Checkpoint: el drain imprime una firma aterrizada por venta en la fila y después su línea de fila-vacía, y ha dejado de imprimir antes de que toques la terminal dos. Si todavía están llegando firmas, el journey todavía no se ha ganado el derecho a correr.

**6. El harness del journey, y el único tramo trabajado.** El driver de abajo es el scaffold entero que te llevas: el verificador compartido, un wrapper de reintentos que distingue un timeout de un rechazo, el poll acotado para los dos tramos asíncronos, el impresor de PASS/FAIL, y el tramo 1 trabajado en pleno como el patrón. Los tramos 2 al 7 están nombrados, comentados, y son tuyos.

```typescript
// stack/src/journey.ts
import { execFile } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { createKeyPairSignerFromPrivateKeyBytes } from '@solana/kit';
import { createVerifier } from '../../verifier/src/verify';
import { createRpcFetchTransaction } from '../../verifier/src/rpc';
import { createMemoryStore } from '../../verifier/src/store';

const run = promisify(execFile);

// One fresh verifier for the whole journey. Its processed-signature store
// is OURS, separate from the worker's: the journey re-asserts every leg
// independently instead of trusting any ledger row the app already wrote.
export const verify = createVerifier({
  fetchTransaction: createRpcFetchTransaction({ commitment: 'confirmed' }),
  store: createMemoryStore(),
});

// The commitment policy rides into the harness too: ordinary legs assert
// at confirmed, and any leg your checklist marked high-value gets this
// stricter judge instead (in my seven, the refund leg). Same adapter,
// same RPC_URL-or-devnet default it has carried since the verifier lesson.
export const verifyFinalized = createVerifier({
  fetchTransaction: createRpcFetchTransaction({ commitment: 'finalized' }),
  store: createMemoryStore(),
});

export interface LegResult {
  leg: string;
  ok: boolean;
  detail: string;
}

// Footgun four lives here: a devnet read timeout is retried with backoff,
// but a verifier rejection is returned immediately and never retried.
export async function retryRead<T>(
  read: () => Promise<T>,
  tries = 3,
  delayMs = 2_000,
): Promise<T> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= tries; attempt += 1) {
    try {
      return await read();
    } catch (err) {
      lastError = err;
      if (attempt < tries) {
        await new Promise((resolve) => setTimeout(resolve, delayMs * attempt));
      }
    }
  }
  throw lastError;
}

// Polling is for effects another process has not produced YET (a ledger row
// the worker writes, an invoice the crank opens). Distinct from retryRead on
// purpose: an RPC error and a missing effect are different bugs.
export async function waitFor(
  condition: () => Promise<boolean>,
  label: string,
  timeoutMs = 60_000,
  pollMs = 2_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await condition()) return;
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  }
  throw new Error(`timed out waiting for ${label}`);
}

export async function assertLeg(
  name: string,
  body: () => Promise<string>,
): Promise<LegResult> {
  try {
    const detail = await body();
    console.log(`PASS ${name}: ${detail}`);
    return { leg: name, ok: true, detail };
  } catch (err) {
    const detail = err instanceof Error ? err.message : String(err);
    console.log(`FAIL ${name}: ${detail}`);
    return { leg: name, ok: false, detail };
  }
}

// Leg 1, worked in full: the ramp stub. Coinbase will not onboard a
// headless test buyer, so this leg asserts the session-token contract:
// the URL exists, it binds the network, and it leaks no wallet address.
async function legRampStub(): Promise<string> {
  const rampDir = fileURLToPath(new URL('../../wavelength-checkout/ramp-embed', import.meta.url));
  const { stdout } = await run('npx', ['tsx', 'smoke.ts'], { cwd: rampDir });

  if (!stdout.includes('pay.coinbase.com')) {
    throw new Error('no onramp URL printed');
  }
  if (
    !stdout.includes('sessionToken') ||
    !stdout.includes('defaultNetwork=solana')
  ) {
    throw new Error('URL missing sessionToken or defaultNetwork=solana');
  }
  // main() exports BUYER_ADDRESS from the keypair it mints at start. If it
  // is missing, fail the leg: an assertion that silently skips is worse
  // than no assertion, because it prints PASS while checking nothing.
  const buyer = process.env.BUYER_ADDRESS ?? '';
  if (buyer === '') {
    throw new Error('BUYER_ADDRESS unset: set it from the minted buyer before this leg');
  }
  if (stdout.includes(buyer)) {
    throw new Error('wallet address leaked into the onramp URL');
  }
  return 'session-token URL shaped correctly, wallet address absent';
}

// One buyer for the whole journey (see "One buyer, staged balances").
// Minted fresh in main() before any leg runs; legs 2-7 sign and pay with it.
let buyer: Awaited<ReturnType<typeof createKeyPairSignerFromPrivateKeyBytes>>;

async function mintBuyer() {
  // The staged-balances plan, made real: a fresh buyer per run, persisted
  // to /tmp/buyer.json in solana-keygen's 64-byte format so the mid-debug
  // airdrop command works, with BUYER_ADDRESS exported before any leg runs
  // (the ramp leg's leak check reads it, and child processes inherit it).
  // Funding it -- USDC to its ATA, deliberately zero SOL -- is your leg
  // bodies' staging work, not the mint's.
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const signer = await createKeyPairSignerFromPrivateKeyBytes(seed, true);
  const pubkeyBytes = new Uint8Array(
    await crypto.subtle.exportKey('raw', signer.keyPair.publicKey),
  );
  writeFileSync(
    '/tmp/buyer.json',
    JSON.stringify(Array.from(seed).concat(Array.from(pubkeyBytes))),
  );
  process.env.BUYER_ADDRESS = signer.address;
  return signer;
}

async function main(): Promise<void> {
  buyer = await mintBuyer();

  const results: LegResult[] = [];

  results.push(await assertLeg('ramp-stub', legRampStub));

  // Legs 2 through 7 are yours. Each chain leg's body ends the same way:
  // re-fetch through `verify` (wrapped in retryRead) and throw on any
  // result where ok is false.
  //
  // results.push(await assertLeg('gasless-first-purchase', legGasless));
  // results.push(await assertLeg('webhook-fulfilled-order', legWebhookOrder));
  // results.push(await assertLeg('subscription-and-dunning', legSubscription));
  // results.push(await assertLeg('blink-purchase', legBlink));
  // results.push(await assertLeg('agent-pays-3x', legAgentApi));
  // results.push(await assertLeg('refund', legRefund));

  const failed = results.filter((r) => !r.ok);
  console.log(
    `journey: ${results.length - failed.length}/${results.length} legs passed`,
  );
  process.exit(failed.length === 0 ? 0 : 1);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
```

Checkpoint antes de que escribas un solo cuerpo de tramo: con el stack arrancado en la terminal uno y solo el tramo 1 cableado, `npm run journey` imprime una línea `PASS ramp-stub:`, después `journey: 1/1 legs passed`, y sale con 0. Demuestra que el harness funciona mientras todavía está juzgando un tramo fácil; un driver que debugueas por primera vez en el tramo 6 es un driver en el que no confías en el tramo 6.

La forma de cierre de cada tramo de cadena son las mismas cuatro líneas, así que aquí está el patrón una vez, con el precio del prensado del catálogo, y después no se muestra nunca más:

```typescript
// the tail of every chain leg: one verifier verdict decides PASS
const result = await retryRead(() =>
  verify(signature, {
    recipient: STORE_WALLET,
    recipientAta: STORE_USDC_ATA,
    mint: '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
    amountBaseUnits: 12_500_000n,
    orderId,
  }),
);
if (!result.ok) throw new Error(result.reason);
```

Doce USDC y medio de devnet — el precio de catálogo del prensado, sin cambios desde que la lección de transaction-request lo fijó — en unidades base, contra el mint de devnet que tu config de transfer-kit tiene fijado desde el módulo 2. Para el tramo gasless, agrega las dos lecturas específicas del patrocinio sobre la misma transacción traída: el fee payer tiene que ser igual al firmante de Kora y no tiene que ser igual al comprador, y el delta de lamports del comprador tiene que ser exactamente cero. Para la mitad de dunning del tramo 4, la afirmación no es sobre una transacción en absoluto; es una lectura del libro mayor que demuestra que la falla forzada se volvió una factura abierta y que no existe ninguna transacción de reintento contra la billetera del comprador.

![Diagrama de flujo de una renovación fallida forzada donde el camino que pasa registra una factura abierta sin reintento de billetera, mientras que un reintento de billetera o un revoke de autoridad fallan.](assets/v08-flowchart.webp)

Para el tramo del agente, recuerda que la afirmación tiene tres lados: la llamada sin pagar tiene que volver 402, las tres llamadas pagadas tienen que liquidar en devnet, y los tres ids de factura de `extra.memo` tienen que aparecer en el libro mayor del back office. El dinero que aterriza pero nunca se concilia hace fallar el tramo. Eso es deliberado, y es la misma lección que el libro mayor viene enseñando desde el módulo 4: en el comercio, un pago sin conciliar es un pasivo con disfraz de éxito.

![Flujo de un agente recibiendo un 402, pagando con el header de firma de pago, y después leyendo una respuesta de pago cuyo id de factura de memo se concilia en el libro mayor, repetido tres veces.](assets/v09-flowchart.webp)

Ese es el lab entero, y decirlo así es la última jugada didáctica de la lección: seis pasos, dos archivos nuevos, cero código de pago nuevo. Todo lo demás que vas a escribir esta noche son cuerpos de tramos del journey llamando a interfaces que ya son tuyas.

## Challenge

Modo en solitario, dicho sin rodeos: sin scaffold, sin archivos trabajados, sin marcadores TODO. Te llevas la distribución objetivo de arriba y los siete criterios de aceptación de abajo, y cableas y corres el stack sin ayuda.

El run-book son dos terminales. Terminal uno: `npm run --workspace stack boot`, después espera tres líneas de arranque sanas y el mensaje de fila-vacía del drain. Terminal dos: `npm run journey`. Nada más se toca a mano entre esos dos comandos y la línea final de resumen; si te encuentras curleando un endpoint a mitad de corrida para empujar un tramo, el que no está terminado no es el stack, es el script del journey. Financia tu billetera de comercio con SOL de devnet antes de arrancar, prepara al comprador enteramente desde adentro del script, y estampa cada id de pedido con el run id para que el journey siga siendo re-corrible contra un libro mayor que ya tiene corridas anteriores.

1. **ramp-stub**: la URL de onramp impresa contiene `sessionToken` y `defaultNetwork=solana`, y la dirección de la billetera del comprador no aparece en ninguna parte de ella.
2. **gasless-first-purchase**: el comprador arranca con cero SOL; la transacción vuelta a traer muestra al firmante de Kora como fee payer, exactamente una firma del comprador, un delta de lamports del comprador de cero, y la ATA de la tienda acreditada con el precio del disco; el verificador la pasa.
3. **webhook-fulfilled-order**: incluida la venta drenada de la fila de la feria, un evento entregado por triplicado sigue produciendo exactamente una fila despachada del libro mayor, y el despacho pasó solo después de un veredicto del verificador.
4. **subscription-and-dunning**: un pull de facturación se concilia como una fila de factura; la renovación fallida forzada se vuelve una factura abierta sin ninguna transacción de reintento contra la billetera, y la suscripción no se desarma.
5. **blink-purchase**: los metadatos del GET y el POST `{account}` devuelven respuestas de action conformes a la spec, la transacción viene del constructor del módulo 3, y la compra aterrizada pasa el verificador.
6. **agent-pays-3x**: la llamada sin pagar devuelve 402; tres llamadas pagadas liquidan; tres ids de factura de `extra.memo` distintos se concilian en el libro mayor.
7. **refund**: un pago push inverso emitido a través de transfer-kit, registrado contra la firma de origen, pasando el verificador, con el libro mayor enlazando las dos direcciones.

Acepta cuando `npm run journey` imprima siete líneas PASS y salga con 0, Y la checklist de prod-gate del módulo 8 esté re-puntuada contra el stack ensamblado en vez de contra los peldaños individuales; corre su runner exactamente como lo hiciste el módulo pasado (`npx tsx gate/run.ts` desde la raíz del repo); la carpeta `gate` no necesita lugar en el roster de workspaces. La re-puntuación no es una formalidad, y tiene permitido terminar en RED: algunas filas que estaban verdes por peldaño se vuelven honestamente más débiles en el ensamblaje, porque la checklist ahora ve un solo firmante compartido y un solo árbol de procesos donde antes veía servicios aislados. Tu runner solo habla pass y fail, así que puntúa una fila honestamente degradada como fail, escribe la tarea de arreglo, y déjala decir "separar antes de producción" donde esa sea la verdad. La barrera de aceptación para la re-puntuación es que cada fila de fail cargue una tarea de arreglo veraz, no que la línea de veredicto diga GREEN; una checklist que solo pasa siempre ha dejado de medir algo. La política de commitment es parte de esa checklist también, así que sostén la línea que trazó el curso: `confirmed` para los tramos ordinarios, `finalized` donde tu checklist marcó un flujo de alto valor.

## Checkpoint: siete líneas

Si el journey está en rojo, trabaja la tabla de trampas antes de leer una sola línea de código de tramo: un error de tipos en el crank es contaminación cruzada, un despacho doblado es la guarda de claim que falta, un not-found sobre la venta en la fila es una fila sin drenar, y un timeout que parece una falla es devnet limitándole la tasa a tus lecturas. Voy a confesar la que me agarró a mí la primera vez que corrí una demo ensamblada propia: vi un tramo "fallar" tres veces, reescribí un handler perfectamente bueno dos veces, y la transacción había aterrizado bien todas y cada una de las veces. El RPC público estaba estrangulando mis lecturas de verificación, no mis pagos. El wrapper retryRead de tu harness existe por exactamente esa noche.

Y para que sepas el objetivo hacia el que estás debugueando, aquí está cómo se ve una noche verde, el último checkpoint del curso:

```
[server] wavelength-stack listening on :3000
[worker] backoffice worker ready
[crank] crank armed on plan wavelength-motm
PASS ramp-stub: session-token URL shaped correctly, wallet address absent
PASS gasless-first-purchase: Kora fee payer, buyer lamports unchanged, 12.5 USDC verified
PASS webhook-fulfilled-order: exactly one ledger row across three deliveries
PASS subscription-and-dunning: pull reconciled; forced failure -> open invoice, no retry
PASS blink-purchase: ActionPostResponse tx from the module-3 builder, verified
PASS agent-pays-3x: 402 gate live, three settlements, three memo ids reconciled
PASS refund: reverse push recorded against origin signature
journey: 7/7 legs passed
```

Pasadas las trampas, cada tramo falla en su propio dialecto, y a estas alturas te has topado con cada uno de ellos una vez. Un tramo gasless donde el saldo de lamports del comprador se movió quiere decir que Kora patrocinó algo fuera de tu allowlist o que el fee payer cayó de vuelta al comprador; comprueba la config de patrocinio antes que el código de la transacción. Un tramo de suscripción que se niega con `too-early` es la guarda de la ventana del período haciendo su trabajo contra tu reloj de prueba, la misma aritmética de segundos-contra-horas que machacó la lección de facturación. Un tramo de blink que funciona en curl y muere en el cliente del journey es el par CORS-y-`actions.json`-en-la-raíz de la lección del blink, resurgiendo porque el mount se movió. Un tramo de agente en loop de 402 para siempre normalmente quiere decir que la liquidación aterrizó pero el id de memo nunca se concilió, así que el gate sigue tratando al agente como sin pagar; lee el libro mayor antes de leer los logs del facilitador. Y un tramo de reembolso rechazado como `wrong-reference` es una deriva de formato de memo entre el constructor de reembolsos y lo que el verificador espera, que es un diff de una línea contra el helper de memo de transfer-kit. Ninguno de estos es un bug nuevo. Ese es el pago callado de construir sobre tus propios peldaños: cada modo de falla del stack ensamblado es uno que ya arreglaste una vez, en algún lado, con una lección adosada.

Y cuando esté verde, de verdad lee el log antes de seguir adelante. Siete líneas. Un desconocido llegó desde fiat, compró sin SOL, fue despachado exactamente una vez por una máquina, se suscribió, hizo fallar una renovación hacia una factura abierta limpia, volvió a comprar por un link compartido, fue facturado tres veces por un agente de software, y se llevó un reembolso que concilia con el pago original. Cada línea la afirmó un verificador que escribiste tú contra estado de la blockchain que trajiste tú, y ni una sola línea te necesitó. Quince peldaños cooperando ya no es una afirmación; es un archivo de log, y lo puedes volver a correr mañana.

Una sola tienda corre de punta a punta, y un journey de comprador completo pasa por sí solo. Lo que queda no es código. La próxima lección nombras lo que de verdad construiste, pesas los rieles que ahora tienes contra los que el ecosistema sigue tendiendo, y decides hacia dónde apunta un ingeniero de pagos este conjunto de habilidades después. Trae el log del journey; se ha ganado su lugar en esa conversación.
