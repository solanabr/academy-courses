# Vercel: la URL

## Resumen

La lección pasada le dio una cara a la estación: un panel de Vite + React en `packages/pulse-board`, sondeando el `status.json` del cron y coloreando filas con el clasificador de pulse-core. Corre hermoso, en localhost, lo que significa que su audiencia total es una persona, y esa persona ya sabe lo que dicen los números. Nueve lecciones de trabajo real que se acumula, y ni un solo otro humano puede ver nada de eso.

Hoy eso termina. Esta es SHIP #2 y la primera URL del curso: despliegas pulse-board a Vercel desde el workspace, y adentro del primer cuarto de esta lección hay una dirección de producción que le puedes mandar por mensaje a alguien en otro continente cuyo teléfono va a renderizar TUS datos de sondeo. El resto de la lección se gana el entendimiento: qué pasó de verdad cuando tipeaste una sola palabra, la reescritura de un bloque y la regla de variables de entorno que vuelven honesta en producción a una SPA, y el contrato del tier gratis leído desde sus números publicados en vez de desde las sensaciones de un blog post.

Haz esto ahora mismo, antes de leer otro párrafo:

```bash
npm i -g vercel   # Vercel CLI (59.11.2 as of 2026-09-02; the CLI moves fast, expect a higher digit)
vercel login
```

Elige el login de GitHub cuando se abra el navegador, y asegúrate de que sea tu cuenta PERSONAL, la que tiene el repo de la estación. Esa elección se tomó por ti allá en M1, y esta lección es donde descubres por qué importaba.

El repliegue de la ayuda, en voz alta: manejamos el primer deploy juntos con cada prompt narrado, los pasos de endurecimiento llegan como una checklist que ejecutas tú, y el ejercicio de cierre te entrega dos deploys roscos y ninguna baranda.

## La entrega y la letra chica

### Entrega primero, entiende después

Desde la RAÍZ del workspace, no desde el directorio del paquete (el CLI prefiere la raíz del repo y te va a preguntar dónde vive el código):

```bash
vercel
```

El CLI te lleva por un cuestionario corto. Set up and deploy: yes. Scope: tu cuenta personal. Link to an existing project: no. Project name: `pulse-board` está bien. Y después la pregunta que importa, la de en qué directorio está ubicado tu código: responde `packages/pulse-board`. El CLI detecta Vite, corre el build de forma remota, y te devuelve una URL de preview. Tantéala, confirma que el panel renderiza, y después promuévela:

```bash
vercel --prod
```

Se imprime una URL de producción. Esa es toda la entrega.

Dos URLs en dos comandos merecen una definición, porque la división es estructural para el resto de tu vida de deploys. El `vercel` pelado creó un **deployment de preview**: su propia URL única, un build completo de exactamente lo que mandaste, seguro para compartir y seguro para tirar. `vercel --prod` creó un **deployment de producción**, el que apunta la dirección principal de tu proyecto. Mismo pipeline, audiencia distinta: los previews son donde mirás un cambio antes de creerle, producción es la dirección que le das a desconocidos. Una vez que el repo esté conectado en el lab, esta división se automatiza: los pushes a una branch reciben URLs de preview, los merges a main van a producción, y nunca más te vas a preguntar si alguien que revisa está mirando la versión que crees.

Ahora haz la cosa para la que existe esta lección: abre esa URL en un dispositivo que nunca vio tu código. Un teléfono con datos móviles sirve. Mejor, mándasela a alguien y mira filas de sondeo reales, latencias que midió TU cron, renderizando en hardware que nunca tocaste. La primera URL que entregué en mi vida se la mandé a un amigo que la abrió en un bus, y refresqué las analíticas como si fuera la noche de las elecciones. Nada que haya desplegado desde entonces pegó igual. Este hito, la entrega de la lección diez, te costó nueve lecciones de TypeScript, una suite de pruebas, una extracción a workspace, y un cron que viene commiteando JSON fielmente desde hace semanas. Te ganaste la URL. Tómate el minuto.

Bueno. Minuto terminado. ¿Qué pasó de verdad?

![El comando vercel sube el código fuente, una máquina remota lo construye, los archivos emitidos aterrizan en un CDN, y una URL los sirve.](assets/v01-flowchart.webp)

El desglose, etapa por etapa. El CLI empaquetó tu código fuente y lo mandó para arriba. Una máquina de build de Vercel miró el repo, reconoció Vite desde lo que el repo mismo declara (la dependencia `vite`, el script de build, el archivo de config), corrió `vite build`, y tomó el directorio `dist/` emitido. Esos archivos fueron al CDN de Vercel, replicados a ubicaciones de edge, y la URL que recibiste es un nombre apuntando a ellos. Fíjate en lo que NO está en ese pipeline: la carpeta dist de tu laptop. El build corrió en la máquina de ellos desde tu código fuente, que es por qué un build que falla va a aparecer en SUS logs, y por qué, una vez que el repo esté conectado, un git push puede disparar un deploy mientras tu laptop sigue cerrada.

La gente le dice zero-config, y el mecanismo honesto vale enunciarlo sin el marketing de nadie encima: tu repo ya declara su framework, su comando de build y su directorio de salida, así que la plataforma lee esas declaraciones y provisiona infraestructura que coincida. Configuraste bastante. Solo que lo hiciste en `package.json` y en `vite.config.ts`, archivos que mantenías igual, y la plataforma los trató como la config. Sintetízalo hasta el fondo: el deployment se volvió un artefacto de build. El repo ahora es la única fuente de verdad de cómo se ve producción.

Un prompt merece una segunda mirada: la pregunta del directorio. Esa respuesta es la configuración **Root Directory** vestida de prompt de terminal. El modelo de monorepo de Vercel es un proyecto por directorio desplegable: tu repo tiene `pulse-core`, `pulse-fleet` y `pulse-board`, y el proyecto que acabas de crear apunta a exactamente uno de ellos. La configuración se elige al importar y se puede editar después bajo Settings, después Build and Deployment, después Root Directory. Esta fue la contingencia abierta del curso por un rato (el plan de respaldo era extraer el panel a un repo independiente), y la sonda de docs la cerró: Root Directory es el camino soportado y documentado, sin necesidad de cirugía.

![Un proyecto de Vercel apunta su Root Directory al paquete del panel mientras los dos paquetes hermanos quedan sin desplegar.](assets/v02-diagram.webp)

Hay un bonus enterrado acá que ya pagaste. Cuando el repo está conectado a GitHub, Vercel se saltea builds de proyectos de monorepo que un commit no afectó, y los requisitos documentados para ese salteo se leen como una checklist de m03-l1: una definición de workspace real con los paquetes declarados, un `name` único por paquete, y las dependencias entre paquetes enunciadas en cada `package.json`. La higiene del manifest que hiciste hace dos lecciones no era ceremonia. Es la razón por la que esta importación Simplemente Funciona y la razón por la que los deploys hermanos futuros no van a quemar slots de build en commits que solo tocaron la flota.

### Volverlo honesto en producción

Tu panel está en vivo pero todavía no es honesto. Dos brechas, las dos invisibles en localhost.

Brecha uno: los deep links. Visita tu URL de producción con cualquier path agregado, `your-board.vercel.app/history`, digamos. Recibes la página 404 de Vercel. El mismo path bajo `npm run dev` renderiza la app perfecto. Yo entregué este 404 exacto, más de una vez, y la segunda fue más vergonzosa porque la primera vez había escrito el arreglo en una wiki. La asimetría es la lección: el servidor de dev reescribe en silencio los paths desconocidos a `index.html` como favor, y un host estático de producción no hace favores. No hay ningún archivo llamado `history` en `dist/`, así que un servidor de archivos correctamente dice 404. Tu SPA rutea en el cliente, lo que significa que el host tiene que entregar `index.html` a TODO path y dejar que React siga desde ahí.

El arreglo entero es un bloque. Crea `packages/pulse-board/vercel.json`:

```json
{
  "rewrites": [{ "source": "/(.*)", "destination": "/index.html" }]
}
```

Cada request, cualquiera sea el path, recibe el shell de la app; el router del cliente (hoy, solo tu panel de una página; desde M8, rutas reales) decide qué renderizar. El panel todavía ni tiene una segunda ruta y esto igual importa: el día en que le crezca una, y el panel de Solana de M8 agregue otra, los deep links compartidos por chat van a funcionar o dar 404 según si este bloque se entregó hoy.

![Un request de deep link tiene éxito en dev, falla en un host estático pelado, y vuelve a tener éxito una vez que el rewrite sirve la página index.](assets/v03-comparison.webp)

Brecha dos: la URL hardcodeada. Ahora mismo el panel hace fetch de `status.json` desde una URL cruda de GitHub pegada en el código fuente. Funciona, pero suelda tu deployment a un repo: cualquiera que forkee la estación, y tú mismo cuando M8 agregue una segunda fuente de datos, tiene que editar código para reapuntarlo. La configuración pertenece al entorno. Acá es donde Vite tiene una regla que tienes que saber de memoria: **solo las variables de entorno con prefijo `VITE_` llegan alguna vez al código del cliente.** Todo lo demás queda del lado del build, invisible para el bundle.

El prefijo no es burocracia, es un formulario de consentimiento. Todo lo que está en el JavaScript del cliente es legible por todo el mundo, para siempre, por cualquiera con una pestaña de devtools. Así que la exposición es opt-in, y el prefijo feo eres tú firmando el formulario: este valor va a ser público. Lee la regla al revés y es la lección de seguridad: cualquier cosa secreta NUNCA debe usar `VITE_`. Ninguna API key, ningún token, nada que no imprimirías en una camiseta. El barrido de secretos de cuatro plataformas de m09-l2 vuelve a esta regla exacta con una checklist.

Cabléalo. Primero la config tipada, en `packages/pulse-board/src/config.ts`:

```ts
const repo = import.meta.env.VITE_STATION_REPO;

export const statusUrl: string | null = repo
  ? `https://raw.githubusercontent.com/${repo}/main/status.json`
  : null;
```

Y enséñale a TypeScript sobre la variable en `packages/pulse-board/src/vite-env.d.ts`. Las plantillas viejas de create-vite armaban este archivo; la plantilla react-ts 9.x actual no, así que créalo tú mismo, y el nombre sigue importando, porque `vite-env.d.ts` es el hogar convencional que los docs de Vite y todos tus compañeros de equipo van a buscar:

```ts
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_STATION_REPO: string | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
```

El `string | undefined` es honestidad deliberada: la variable podría no estar seteada, y el tipo obliga a cada consumidor a decir qué pasa entonces. En el panel, un statusUrl en `null` debería renderizar un estado visible de error-de-configuración, no una página en blanco. Falla a gritos. Un panel que no renderiza nada y no dice nada es lo peor de los dos mundos.

Una nota de mecanismo que te va a salvar una tarde de confusión: Vite incrusta estos valores en tiempo de BUILD. El bundler literalmente reemplaza como string `import.meta.env.VITE_STATION_REPO` por el valor durante `vite build`. Un sitio estático no tiene entorno en runtime que leer, así que cambiar la variable en el panel no le hace nada a los archivos desplegados hasta que el próximo build hornee el valor nuevo. Setea una var, redespliega, siempre en ese orden.

![Una variable de entorno pasa una barrera de prefijo en tiempo de build y o se incrusta en el bundle público o se queda del lado del build.](assets/v04-diagram.webp)

Setea el valor donde la máquina de build pueda verlo:

```bash
vercel env add VITE_STATION_REPO
# paste your owner/repo when prompted, e.g. yourname/pulse-station
# when it asks which environments, select all three: Production, Preview, Development
vercel env pull packages/pulse-board/.env.local
```

Los entornos importan acá, y mapean sobre la división de deployments que acabas de aprender: Vercel limita cada variable a Production, Preview, Development, o cualquier combinación, que es por qué el prompt pregunta. Elige los tres para esta, porque el panel debería renderizar los mismos datos en todos lados. El `pull` después sincroniza los valores de Development a un `.env.local` que está en el gitignore, así el dev local lee la misma configuración que producción hornea. Fíjate en el path de destino del pull: el CLI corre desde la raíz del workspace como todo comando de vercel acá, pero Vite solo lee archivos de env desde la raíz del propio proyecto de Vite, así que el archivo tiene que aterrizar adentro de `packages/pulse-board/`. Tíralo a la raíz del repo en cambio y el dev local nunca ve la variable, renderiza tu estado de error-de-configuración, y te manda a cazar una var "que falta" y está sentada un directorio más arriba. Una variable, una fuente de verdad, tres entornos. Si la hubieras limitado solo a Production, los previews construirían con la variable ausente y renderizarían tu estado de error, que es una configuración legítima para valores que difieren por entorno, y una sorpresa confusa para los que no deberían. (Regla de la casa desde la propia página de límites de la plataforma, leída el 2026-09-01: todas tus variables de entorno juntas tienen un techo de 64 KB. Nunca lo vas a alcanzar con un slug de repo; un equipo metiendo blobs de JSON en variables sí, y ahora sabes que el techo existe.)

### El contrato de Hobby, desde sus propias páginas

Antes de confiarle tu estación a un tier gratis, lee las páginas del vendor, fechadas, y el resumen de nadie. Este hábito tiene una razón fresca adjunta: el 2026-09-01 este curso sondeó los propios docs de Pages de Cloudflare y los encontró diciéndote que empieces proyectos nuevos con Workers en cambio. Un vendor dando de baja un producto a la vista de todos, en su propia documentación, mientras tutoriales de hace tres años siguen recomendándolo. Los docs se mueven; los blog posts se fosilizan. Así que acá va el tier Hobby de Vercel desde las páginas de precios y de límites de vercel.com tal como se leyeron el 2026-09-01, números, no adjetivos.

Incluido por mes: 100 GB de Fast Data Transfer, 1M Edge Requests, 1M Function Invocations. Techos operativos: 100 deployments por día, 100 builds por hora, 200 proyectos, un build concurrente, un tope de build de 45 minutos. Esas dos listas son clases distintas de números. La primera es consumo que gastas siendo popular; la segunda es throughput que gastas iterando. Tu estación no tensiona ninguna: un archivo JSON chico más un bundle modesto contra 100 GB es un margen enorme, y necesitarías entregar código más rápido que un deploy cada 15 minutos todo el día para sentir el techo de deployments.

![Mosaicos de datos listan las asignaciones mensuales del tier Hobby y sus techos operativos con sus valores documentados y la fecha de la fuente.](assets/v05-chart.webp)

Ahora la parte que vuelve a Hobby genuinamente enseñable: el modelo de excedente. Hobby no tiene ciclo de facturación. No hay nada contra lo que medir, así que no hay factura, nunca. Excede un límite y el comportamiento documentado es que la feature se PAUSA hasta que pase la ventana de 30 días, en la mayoría de los casos (la única excepción documentada que encontró la pasada de investigación: Web Analytics se reanuda después de 7 días). Después el servicio se reanuda. Quédate con lo que eso significa: en este tier, el peor caso de que tu estación se vuelva viral es downtime. Nunca deuda. Cada otro modelo de precios que vayas a conocer en tu carrera debería medirse contra esa oración.

![Exceder un límite de Hobby pausa la feature hasta que pase una ventana de treinta días, mientras no existe ninguna rama de facturación.](assets/v06-flowchart.webp)

Lee el modelo contra el tráfico real de tu estación y el margen deja de ser abstracto. El payload del panel es un JSON de estado medido en kilobytes y un bundle construido que se entrega una vez por visitante y después se queda en su cache. Para gastar 100 GB necesitarías tráfico en los millones de cargas de página, y si la estación de algún modo encuentra esa audiencia, lo que pasa es una pausa y un problema muy bueno, no una factura sorpresa. Conocer el modo de falla ANTES de que se dispare es el hábito operativo que este curso sigue ejercitando, y acá el modo de falla está documentado, acotado y es sobrevivible por diseño.

La generosidad tiene bordes, y son diseño, no letra chica para resentir. Tres restricciones documentadas se combinan acá. Hobby es para uso no comercial y personal, según la política de uso justo. Hobby es EN SOLITARIO: la tabla de comparación de planes muestra un guion para las features de colaboración en equipo, así que no hay invitar a alguien a colaborar en el proyecto. Y los equipos de Hobby no pueden conectar repositorios propiedad de organizaciones de Git. Cuenta personal, repo personal, un humano, nada en venta. Ahora mira atrás a M1, cuando el curso insistió en que la estación viviera pública en tu cuenta PERSONAL de GitHub. Esa era esta lección estirándose hacia atrás. Toda la forma de la estación se diseñó para que la entrega de la lección diez no necesite ningún workaround, que es lo que de verdad cuesta un camino central honesto con el tier gratis: decisiones tomadas meses antes.

Y la pregunta de la tarjeta de crédito, que merece responderse como este curso responde todo. Vas a leer "no credit card required" sobre Vercel Hobby por todo internet. Vercel nunca escribe esa oración. Lo que muestran los docs: el plan Hobby no tiene ciclo de facturación, y el único lugar donde los datos de tarjeta aparecen en la documentación del plan es el paso cinco del flujo de upgrade a Pro. Varios análisis de terceros de 2026 dicen que no se pide tarjeta al registrarse. Pero al 2026-09-02 este curso no lo verificó con un registro nuevo, así que lo formulamos exactamente hasta donde llega la evidencia: gratis, sin ciclo de facturación, el uso se pausa en vez de facturarse. Cuando no puedes citar la fuente de una oración, no digas la oración. Si le afirmas "no pide tarjeta" a alguien, estás citando a blogueros, no al vendor, y saber la diferencia es una habilidad profesional que esta lección está modelando a propósito.

El trade-off honesto, en las dos direcciones. Zero-config es un préstamo, no un regalo: Vercel infirió tu build porque tu repo coincide con un patrón que conoce, y el día en que te desvíes del patrón, un layout de monorepo raro, un paso de build exótico, la magia se vuelve configuración que ahora tienes que aprender igual, con una capa de inferencia sentada entre ti y el mensaje de error. Y los bordes de Hobby descalifican casos reales por diseño: la app de producción de una startup es comercial, colaborativa y probablemente propiedad de una organización, que es cero de tres. El camino honesto de upgrade existe (Pro, $20 por asiento por mes). También existe la alternativa honesta: `dist/` son solo archivos, y cualquier host estático de la tierra puede servir archivos. Lo que reconstruirías en otro lado no es el hosting, es el loop, push para desplegar, más el CDN y la plomería de entorno. Ese loop es lo que de verdad cambia el juego, y vale saber que ESO es lo que te faltaría, no la marca.

![Los archivos estáticos son portables a cualquier host, mientras el loop de push-para-desplegar con plomería de entorno es la parte que una plataforma de verdad provee, con el límite de Hobby a Pro anotado abajo.](assets/v07-comparison.webp)

### Lo que la plataforma no hizo

Una pieza más de honestidad, porque la plataforma que acabas de usar es famosa por features que no tocaste. Ningún servidor corrió esta noche. Tu deployment son archivos estáticos en un CDN, punto. La capa de cómputo de Vercel es real y grande: desde el 2025-04-23, Fluid compute es el default para proyectos nuevos, o sea funciones serverless que se comportan como servidores que no administras, concurrencia adentro de las instancias, facturación sobre CPU activa, a lo largo de runtimes de Node.js, Python, Edge, Bun y Rust (la parte de concurrencia optimizada dentro-de-la-función es solo de Node.js y Python, una distinción que vale mantener derecha cuando alguien te la hype). El panel no necesita nada de eso. Un panel que lee un archivo JSON público es el caso de sitio estático en su forma más pura, y saber precisamente cuál capa de una plataforma NO estás usando es lo que separa el posicionamiento del culto al cargo. La estación SÍ va a crecer una API, y cuando lo haga, en M7, va a un edge completamente distinto, y a esa plataforma también le vas a hacer leer sus propias páginas de precios.

**Profundiza (el 20%).** esta lección enseñó el camino de deploy que ejercita nuestro artefacto, y paró. Los runtimes de Functions, la profundidad de Fluid compute, y el mundo de Next.js-sobre-Vercel son material real que este curso deliberadamente señaliza en vez de enseñar. La entrada canónica es el propio track de getting-started de Vercel, que va CLI-primero exactamente como fue esta lección: [Getting started with Vercel](https://vercel.com/docs/getting-started-with-vercel) (URL sondeada el 2026-09-01; la página misma se actualizó por última vez el 2026-08-11). Léela después del lab si la plataforma te interesa; nada de abajo depende de ella.

## Lab: endurece la entrega

Desplegaste con rueditas. Ahora la checklist, y es una checklist, no un walkthrough: cada paso nombra la meta y la demostración, y tú aportas las teclas.

1. **Demuestra el 404 primero.** Abre `https://<your-board>.vercel.app/history` (o cualquier path inventado) y confirma la página 404. Nunca arregles un bug que no viste fallar; quieres la foto del antes para el después del paso 2.

2. **Entrega el rewrite.** Agrega el `vercel.json` de un bloque de la sección de teoría a `packages/pulse-board/`, commitéalo, y corre `vercel --prod` desde la raíz del workspace. Aceptación: el mismo path de deep link ahora renderiza el panel, y también lo hace cualquier otro path que inventes.

3. **Des-hardcodea la fuente de datos.** Aterriza `config.ts` y la extensión `vite-env.d.ts`, reemplaza la URL cruda pegada en el fetch por `statusUrl`, y haz que el caso null renderice un mensaje visible de error-de-configuración en vez de un panel en blanco. Aceptación: `pnpm run build` adentro de `packages/pulse-board` está limpio, y correr dev SIN la variable seteada muestra tu estado de error, no una página blanca.

4. **Setea la variable en los dos lugares.**

   ```bash
   vercel env add VITE_STATION_REPO
   vercel env pull packages/pulse-board/.env.local
   ```

   El primero pide un valor (tu slug `owner/repo`) y para qué entornos; elige los tres, exactamente como argumentó la sección de teoría. Si la hubieras limitado solo a Production, el `pull`, que sincroniza los valores de Development, te entregaría un `.env.local` vacío y una sesión de dev confusa. El segundo escribe `.env.local` adentro del paquete del panel, donde Vite de verdad lee archivos de env, así el dev local coincide con producción. Confirma que `.env.local` está en el gitignore (lo está, si tu higiene de M1 se mantuvo; revisa igual). Redespliega con `vercel --prod`. Aceptación: producción vuelve a renderizar filas en vivo, y un view-source sobre el bundle desplegado encuentra tu slug de repo horneado en el JavaScript, que es la incrustación en tiempo de build hecha visible, y un adelanto de por qué los secretos nunca usan el prefijo.

5. **Cierra el loop por GitHub.** En el panel de Vercel, conecta el proyecto a tu repo de la estación en la configuración de Git del proyecto. Después empuja un cambio trivial (sube un encabezado, arregla un typo) y mira al panel construirlo y desplegarlo con la participación de tu laptop terminando en `git push`. Aceptación: aparece un deployment de producción nuevo para el que no corriste `vercel`. Desde este commit en adelante, el repo ES el botón de deploy.

6. **El checkpoint social.** Manda la URL a un humano que nunca vio tu código, en una red distinta a la tuya, y consigue confirmación de que renderizaron filas de sondeo en vivo. El teléfono de un desconocido es la única prueba de integración honesta que tiene una URL.

## Challenge: rómpelo dos veces, lee dónde sangra

Sin guía, y vale hacerlo despacio. Las fallas de producción vienen en clases, y la primera habilidad de quien opera es saber DÓNDE aparece cada clase antes de que pase a las 2 a.m.

Rotura uno: quita la variable de entorno (`vercel env rm VITE_STATION_REPO production`), redespliega, y abre la URL. Encuentra la falla. Rotura dos: introduce un cambio que rompa el build (borra el equivalente a un punto y coma de corrección de tipos en algún lado, un import malo funciona bien) y empújalo. Encuentra esa falla también.

Para cada rotura, escribe UNA oración enunciando dónde apareció la falla (¿los logs del build? ¿la página desplegada en runtime?) y qué clase de falla la vuelve eso. Después repara las dos: restaura la variable, revierte el commit, confirma el verde.

![Un error de build se detiene en los logs mientras el deploy viejo sigue sirviendo, pero un valor de configuración que falta se entrega y falla frente a los usuarios.](assets/v08-comparison.webp)

Aceptación para toda la lección: URL de producción en vivo y renderizando datos reales en un dispositivo ajeno, un path de deep link carga directo, y tus dos oraciones clasifican las fallas correctamente, tiempo de build contra runtime. Fíjate en cuál falla fue más segura: el build roto nunca tocó a tus usuarios, porque el deployment anterior siguió sirviendo. La variable que faltaba se entregó. Los errores de configuración son la clase más furtiva, que es exactamente por qué el paso 3 los hizo ruidosos.

## Checkpoint, y el turno del motor

Lo que ahora puedes hacer, concretamente: desplegar un paquete de un monorepo con Root Directory haciendo la puntería; volver honesta en producción a una SPA con un rewrite y una variable de entorno conscientemente pública; leer el contrato de un tier gratis desde sus números publicados y decir qué entra en él (una estación personal, en solitario, no comercial: perfecto) y qué no (cualquier cosa con clientes o compañeros de equipo); y clasificar una falla de producción por dónde apareció. La recuperación de 30 segundos antes de cerrar la pestaña: el día en que tu estación pase volando los 100 GB de transferencia, ¿qué pasa? Dilo en voz alta. La feature se pausa hasta que pase la ventana de 30 días, no llega ninguna cuenta, porque no hay ciclo de facturación para que llegue una.

Dos pedidos mientras está fresco. Primero, anota DÓNDE te peleó el lab (¿las respuestas de los prompts? ¿el viaje de ida y vuelta de la variable de entorno? ¿la conexión de Git?) en tus notas del curso; M7 repite todo este baile en otra plataforma y tu lista de fricción se vuelve tu checklist. Segundo, pon la URL en algún lugar donde la vayas a ver: bio, README, donde sea. Los artefactos públicos se acumulan distinto que los locales, y ahora tienes uno.

La estación tiene una cara pública. Y pulse-core, el motor detrás de todo lo que está en esa página, sigue atrapado en tu workspace donde solo tus propios paquetes pueden importarlo. La lección que viene es la vuelta de honor del módulo: constrúyelo con tsdown, publícalo en npm como un paquete real que otros humanos pueden instalar, y aprende a leer las señales de advertencia de una dependencia que se está muriendo antes de adoptar alguna. La vuelta de honor tiene un registro al final.
