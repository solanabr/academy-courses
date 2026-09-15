# Cadena de suministro: el ataque de 86 minutos

## Resumen

m08-l4 demostró el camino de escritura. La estación firmó y aterrizó una transferencia real de SOL en devnet con el flujo de pipe de kit, `tx-check` guarda el recibo, el cuadro de honestidad del airdrop se ejercitó de verdad, y el fallback de validador local está instalado y se usó al menos una vez. Tu stack lee la blockchain desde los dos lenguajes y puede demostrar que aterriza transacciones. Así que hoy miramos sobre qué está parado todo eso: cinco lockfiles (el del workspace de TS, el del workspace de Rust, los propios de los dos proyectos de edge, y el de `tx-check`, el scaffold independiente que m08-l4 armó alrededor del par de claves de firma), varios cientos de paquetes que no escribiste, y un changelog al que otras personas, algunas de ellas hostiles, pueden agregarle cosas. En esta lección corres `npm audit` y `cargo audit` contra tu propio árbol, aprendes a leer lo que vuelve hasta el nivel de id del advisory, rango vulnerable y camino de dependencias, formalizas en una checklist la lectura de letreros que hiciste informalmente en m03-l4, eliges una filosofía de pins que puedas defender, y haces commit de `AUDIT.md` al repo de la estación. Cómo corren las reps: una lectura de advisory de punta a punta conmigo, la checklist pasada a ti como artefacto, cada veredicto después del primero escrito sin guía. El capstone también te va a quitar la checklist.

## La superficie de ataque con changelog

El 2026-08-20, el mismo día en que se entregó Rust 1.98.0, un atacante secuestró la cuenta de un autor de crate y republicó `arrayref` como 0.3.10, con un script de build que descargaba un payload. Estuvo en vivo 86 minutos antes de que le hicieran yank. Por qué esa fecha pertenece a un curso de Solana: el manifest del programa de SPL Token depende de `arrayref = "0.3.9"`. La versión maliciosa quedaba a una arista de dependencia de cada token en Solana.

Antes de desempacar nada de eso, apunta las herramientas a tu propio árbol. Desde la raíz del repo de la estación, donde vive `pnpm-lock.yaml`:

```bash
pnpm audit
```

Ese es el `npm audit` canónico para un workspace de pnpm: el mismo endpoint de auditoría del registro, leído desde el lockfile de pnpm. En cualquier repo con lockfile de npm la grafía es `npm audit`, y esa es la grafía que describen los docs del ecosistema más amplio; aprendiste en m03-l1 a leer el campo `packageManager` de un repo antes de teclear, y acá vuelve a rendir. Después el lado de Rust. `cargo audit` es un subcomando de cargo que instalas una sola vez:

```bash
cargo install cargo-audit --locked
cd pulse-rs
cargo audit
```

(cargo-audit está en 0.22.2 mientras escribo esto, 2026-09-02; toma lo que te dé `cargo install`. La flag `--locked` construye la herramienta desde su propio lockfile commiteado, que, dado de qué se trata esta lección, es la única forma respetable de instalar una herramienta de auditoría.) Trae la base de datos de advisories que mantiene RustSec y revisa cada crate de `Cargo.lock` contra ella, que es por qué la corres donde vive el lockfile, la raíz del workspace, y no dentro de un crate miembro que no tiene ninguno.

Los dos comandos van a imprimir o hallazgos o un resultado limpio. Guarda lo que te salga; para el final de la teoría vas a poder leerlo como corresponde, y cualquiera de los dos resultados pasa esta lección. La barrera son los veredictos escritos, no un árbol con suerte.

Ahora la historia, contada derecho, porque la línea de tiempo es la lección.

### 86 minutos, minuto a minuto

El ataque de arrayref no fue código ingenioso. Fue una toma de control de cuenta: el atacante tomó el control de la cuenta que el autor tenía en crates.io y publicó una nueva versión de patch de un crate que no necesitaba ninguna. La 0.3.10 maliciosa ni siquiera cargaba el payload en su propio código: agregó una dependencia a un crate de proc-macro con typosquatting, y el script de build de ESE crate descargaba y ejecutaba el payload en tiempo de compilación, lo que quiere decir que el blanco no eran los usuarios de los usuarios del crate, era cada desarrollador y cada máquina de CI que fuera a compilar el árbol en la ventana. Dos crates hermanos del mismo autor corrieron la misma suerte en el mismo incidente, append-only-vec por 107 minutos e internment por 90. Nextron Systems lo detectó; la respuesta de seguridad de Rust les hizo yank a los tres. El write-up vive en blog.rust-lang.org, fechado 2026-08-20, y vale tus diez minutos.

Quédate un segundo con el detalle de SPL Token, porque es la parte que hace de esto una lección de Solana y no un sermón general de higiene. La línea del manifest `arrayref = "0.3.9"` parece un pin. No lo es. La versión pelada de Cargo es semántica de caret, lo que quiere decir que acepta cualquier upgrade compatible con semver, y 0.3.10 es compatible con 0.3.9 según las reglas. Cualquier resolución fresca de dependencias hecha durante esos 86 minutos, un clon nuevo, un job de CI sin lockfile commiteado, un `cargo update`, habría seleccionado la versión maliciosa sin que nadie la pidiera. Todo `Cargo.lock` commiteado mantuvo la línea en la versión que tenía registrada y nunca estuvo en peligro. Mismo rango, misma ventana, dos resultados completamente distintos, y la única variable era si un lockfile se paraba entre el manifest y el registro.

![Una línea de tiempo de un día sombrea los 86 minutos en que el release malicioso de arrayref estuvo en vivo, con las resoluciones frescas expuestas arriba y los lockfiles commiteados a salvo abajo.](assets/v01-timeline.webp)

Ese es uno de los dos incidentes fundacionales. El otro es una década más viejo y falló en la dirección opuesta.

### Dos formas en que un árbol te falla

Marzo de 2016. Un desarrollador en una disputa de nombres despublica sus 273 paquetes de npm, uno de ellos una utilidad de once líneas llamada left-pad, y los builds se rompen por todo el ecosistema en menos de una hora, porque miles de paquetes, incluidos los frameworks más grandes de la época, dependían de él transitivamente. El post-mortem de npm, titulado "kik, left-pad, and npm" y fechado 2016-03-23, sigue en vivo, y se lee como el ecosistema descubriendo, en público, que su grafo de dependencias era un muro estructural compartido que nadie había inspeccionado.

Fíjate que son clases de falla distintas. arrayref fue inserción maliciosa: algo nuevo y hostil entró al árbol a través de un rango de versiones. left-pad fue desaparición súbita: algo viejo y confiable se fue del árbol, y todo lo que se apoyaba en él se cayó. El árbol de tu estación está expuesto a las dos, en los dos ecosistemas, y piden defensas distintas. La inserción es para lo que existen las herramientas de auditoría y los lockfiles. La desaparición es para lo que existe la checklist de abandono más adelante en esta lección, porque un paquete no tiene que esfumarse en una tarde para desaparecer; la mayoría simplemente dejan de mantenerse calladamente, y el README es el último en enterarse.

![Paneles lado a lado contrastan el ataque de inserción de arrayref con la desaparición de left-pad, cada uno mapeado a su propia defensa.](assets/v02-comparison.webp)

### Leer un reporte de auditoría como un adulto

Volvamos a lo que sea que imprimieron tus dos comandos. Un hallazgo de auditoría, de npm o de cargo, tiene las mismas cinco partes estructurales, y quiero que las leas en un orden deliberado, porque el layout del reporte sugiere el equivocado.

Un hallazgo carga un id de advisory: con prefijo GHSA en el mundo de npm, con prefijo RUSTSEC en el mundo de Rust, un nombre estable que puedes buscar, citar en `AUDIT.md` y volver a revisar el mes que viene. Carga una severidad, una palabra sola como moderate o high. Carga un rango vulnerable, las versiones exactas afectadas, escrito en el mismo lenguaje de comparadores que tus manifests, algo como `>=0.3.0 <0.3.11`. Carga una versión parcheada, el upgrade más chico que sale del rango. Y carga un camino de dependencias, la cadena de aristas desde algo que elegiste hasta la cosa que es vulnerable.

Esta es la forma que toma un hallazgo de cargo audit, con los campos etiquetados como los imprime la herramienta:

```text
Crate:     <name>
Version:   <the version your Cargo.lock resolved>
Title:     <one-line description of the vulnerability>
Date:      <advisory publication date>
ID:        RUSTSEC-<year>-<number>
Solution:  upgrade to >= <patched version>
Dependency tree:
<name> <version>
└── <the chain of crates that pulled it in, up to your own>
```

El orden en que hay que leer: id, rango, versión parcheada, después el camino, y solo después la severidad. La severidad califica el bug en abstracto, como si cada usuario del crate lo corriera en la peor posición. El camino califica tu exposición, y tu exposición es la cosa sobre la que de verdad estás decidiendo. Un advisory de severidad alta sentado en una devDependency del tooling de build que usa tu panel nunca se entrega en el bundle de producción; igual corre en tu laptop y en CI, así que no es poca cosa, pero es una evaluación, no una caída. El mismo advisory en un camino de runtime en `pulse-fleet`, código que se ejecuta cada treinta minutos con tus credenciales al alcance, es otra mañana. La misma cadena de severidad, decisiones distintas. Lee el camino antes de actuar; las herramientas que imprimen la severidad en rojo están optimizando para tu atención, no para tu criterio.

El camino es también el campo que puedes interrogar directamente, y estos tres comandos son los mejores amigos de quien lee auditorías:

```bash
pnpm why <package>
npm ls <package>
cargo tree -i <crate>
```

`pnpm why` y `npm ls` recorren el camino desde tu manifest hacia abajo hasta el paquete marcado; `cargo tree -i` invierte el árbol y sube desde el crate marcado hasta lo que sea tuyo que dependa de él. Diez segundos con estos le ganan a diez minutos de scrollear salida de reporte.

![Un diagrama de flujo rutea un hallazgo de auditoría a través de una verificación de rango y un trazado de camino hacia una respuesta urgente de runtime o una evaluación programada de build-time.](assets/v03-flowchart.webp)

Una advertencia sobre el botón que te ofrece el reporte. `npm audit fix` va a reescribir tu árbol para salir de los rangos vulnerables, y bajo `--force` va a aplicar bumps de major de semver para lograrlo. Eso es una herramienta proponiendo migraciones, no aplicando parches. Lee qué quiere cambiar antes de dejarla; un arreglo de seguridad que salta en silencio una dependencia dos majors cambió una vulnerabilidad conocida por roturas desconocidas, y vas a averiguar cuál cuesta más en el peor momento posible.

Y la salvedad más profunda, la que hace que exista la segunda mitad de esta lección: una auditoría es señal, no demostración. La base de datos de advisories contiene lo que alguien encontró, verificó y presentó. Un reporte limpio quiere decir que ningún advisory conocido coincide con tu lockfile. No quiere decir que tu árbol sea seguro; un crate abandonado que nadie está vigilando puede ser vulnerable en silencio para siempre, acumulando no seguridad sino silencio. cargo audit revisa cada crate de `Cargo.lock` contra la base de datos, nada se saltea por estar sin mantenimiento, así que el punto ciego nunca está en lo que la herramienta escanea. Está en lo que la base de datos sabe.

### La checklist de abandono

Por eso mismo la herramienta recibe un socio. En m03-l4 viste a tsup, el bundler de TypeScript por defecto que reinó tanto tiempo, anunciar su propio retiro en una línea de README: "Este proyecto ya no se mantiene activamente. Por favor considera usar tsdown." Leíste ese letrero en el mundo real una vez. Ahora sistematizamos la lectura, porque estabas haciendo cinco chequeos a tanteo y una checklist es cómo una habilidad sobrevive a que se la pasen a alguien más, incluido el tú del futuro a las 2 a.m.

Para cualquier dependencia que estés evaluando, existente o prospectiva, corre estas cinco lecturas:

| Señal | Dónde mirar | Qué te dice |
|---|---|---|
| Aviso en el README | la portada del repo, primera pantalla | Los mantenedores que se van suelen decirlo. tsup lo hizo. Créeles la primera vez. |
| Fecha del último publish | `npm view <pkg> time.modified`, la página de crates.io | Qué tan reciente es el release más nuevo. Viejo por sí solo no condena; viejo más las otras señales, sí. |
| Cadencia de releases | la página de releases, el CHANGELOG | Un proyecto vivo tiene un ritmo. Un proyecto cuyo ritmo se detuvo tiene fecha de muerte, incluso sin anuncio. |
| Deriva de issues abiertos | issues abiertos vs cerrados en los últimos meses | Que los issues se acumulen sin respuesta quiere decir que no hay nadie en casa, diga lo que diga el README. |
| Qué fijan los repos emblemáticos | los manifests de los proyectos serios del ecosistema | La señal más fuerte, sigue leyendo, porque corta para los dos lados. |

Esa última fila merece su propio párrafo, y ya conociste la evidencia. La línea actual de reqwest es 0.13.4, y agave, la implementación de validador, todavía fija 0.12.28. agave también fija `clap = "2.33.1"` con las features por defecto apagadas, la major de una década que leíste en frío en m05-l2, mientras la línea actual de clap es 4.x. Leíste los dos pins en m05-l2 y aprendiste la lectura en frío: un emblemático que se queda en una versión vieja no es negligencia, es una decisión costeada por gente con más en juego que tú, y te dice que la línea vieja todavía funciona y que el upgrade perdió la pelea costo-beneficio hasta ahora. Ahora abre `docs/pin-reads.md`, los tres veredictos que m05-l2 te hizo dejar commiteados, y marca cada uno contra la checklist de cinco filas de arriba, que no tenías cuando los escribiste. Donde una fila habría cambiado tu veredicto, dilo en el archivo y fecha la revisión en vez de editar el original hasta borrarlo. Calificar tu propia lectura en frío contra una checklist que adquiriste después es el ejercicio de calibración más barato de este curso, y solo funciona porque escribiste el veredicto antes de saber la respuesta. "Lee lo que fija tu ecosistema" es evidencia sobre la realidad del mantenimiento y el costo de migración en las dos direcciones: una dep de la que los emblemáticos están huyendo es una advertencia, y una dep que los emblemáticos mantienen felices en una major vieja es estructural y estable ahí.

Ahora la corrección que mantiene honesta a la checklist, y es una que la propia investigación de este curso tuvo que hacer a mitad de camino: los conteos crudos de dependencias no están en la lista, a propósito. Es tentador argumentar "este crate arrastra 30 paquetes, es pesado y riesgoso". Contar es cómo nuestra investigación juzgó mal al principio los crates de cliente de Solana; el cliente de RPC "delgado" resultó cargar más dependencias directas que el "todo incluido", porque sus deps eran docenas de crates de tipos diminutos mientras el grande cargaba una stack de red completa en menos piezas, más pesadas. Un aprendiz que cuenta va a llegar al veredicto equivocado con total confianza. Lee qué son las dependencias, no cuántas hay. Los conteos engañan; los contenidos informan.

![Barras emparejadas muestran a agave manteniendo reqwest una major atrás y clap dos majors atrás de sus líneas actuales, enmarcado como decisiones costeadas.](assets/v04-chart.webp)

Corre las cinco lecturas y aterrizas en uno de cuatro veredictos: keep, upgrade, replace o accept-risk. Todo veredicto se escribe, y la forma es fija, porque un veredicto que vive en tu cabeza es un humor, y un veredicto en el repo es una decisión que la próxima persona puede auditar:

```markdown
### <dependency>
- signal read: <the one or two signals that decided it>
- exposure path: <runtime, build-time, or dev-only, and the edge it enters through>
- decision: keep | upgrade | replace | accept-risk
- action: <the concrete next step, or "none, revisit <date>">
```

Ese veredicto escrito es el entregable real de esta lección, y es la pieza que a los equipos de verdad les falta. Todo el mundo corre auditorías. Casi nadie escribe qué decidió sobre los resultados, así que cada alerta se vuelve a litigar desde cero por quien la vea después.

![Cinco señales de la checklist desembocan en uno de cuatro veredictos, cada uno registrado en la misma entrada de cuatro líneas de AUDIT.md, mientras los conteos crudos de dependencias quedan excluidos por fuera del embudo.](assets/v05-diagram.webp)

### Pins, rangos y el lockfile que manda sobre los dos

Última pieza de teoría, y es la que en secreto era el tema de los 86 minutos. Tienes tres instrumentos para fijar, y cada uno de ellos es apenas elegir de qué lado preferirías estar equivocado.

Los rangos de semver, el `^1.4.0` de package.json y el `"0.3.9"` pelado de Cargo.toml, se auto-curan: se entrega una versión parcheada y tu próxima resolución la toma sin editar el manifest. El mismo mecanismo auto-ingiere: se entrega una versión maliciosa dentro del rango y tu próxima resolución también la toma. La ventana de 86 minutos existe porque los rangos resuelven hacia adelante; eso no es una falla de semver, es el trato entero que firmaste.

Los pins exactos, `=0.3.9` en Cargo, `1.4.2` pelado en npm, congelan lo bueno conocido. También congelan lo malo conocido: el día en que se parchea una vulnerabilidad real, tu pin exacto te deja clavado en la versión vulnerable, en silencio, hasta que un humano edite un archivo. Durante la ventana de arrayref un pin exacto era armadura. Durante los meses después de algún advisory futuro, el mismo pin es la exposición.

Y el lockfile es el pin de verdad, con una condición atada. El rango de tu manifest expresa intención; el lockfile registra una resolución real, byte por byte; y una instalación que honra el lockfile reproduce esa resolución exactamente, sea lo que sea que el rango preferiría hoy. La condición: el lockfile solo protege instalaciones que de verdad lo leen.

```bash
npm ci
pnpm install --frozen-lockfile
cargo build --locked
```

Esas son las grafías que lo honran, y dos de ellas ya están en tu estación: el workflow de Actions de M1 corrió `npm ci` desde el día en que lo escribiste hasta que m03-l1 lo recableó a `pnpm install --frozen-lockfile` cuando el workspace pasó a pnpm, y el Dockerfile de M6 también instala con `--frozen-lockfile`. Una asimetría para anotar antes de que auditees cualquier job de CI contra esta lista: cargo honra un `Cargo.lock` commiteado por defecto, así que un `cargo build` o un `cargo test` pelados en un clon ya instalan las versiones registradas, y `--locked` agrega estrictez (fallar a gritos en vez de actualizar calladamente cuando el manifest y el lockfile no coinciden) en vez de prender la lectura del lockfile. npm es lo contrario; un `npm install` pelado va a reescribir alegremente el lockfile, que es por qué existe `npm ci`. Así que un `cargo test` sin flags en un workflow no es exposición a resolución hacia adelante como sí lo es un `npm install` pelado; clasifícalo como corresponde cuando escribas la línea de CI en el paso 5. Un `npm install` pelado en un clon fresco sin lockfile presente resuelve hacia adelante y se habría comido arrayref. Pasa el ejemplo trabajado una vez, con números de npm esta vez: tu manifest dice `^1.4.0`, tu lockfile registró 1.4.2, y una 1.4.3 maliciosa se publicó esta mañana. El build de CI de esta noche bajo `npm ci` instala 1.4.2 y está bien. El momento peligroso no es el publish. Es el próximo `pnpm update`, la próxima regeneración del lockfile, el próximo "déjame refrescar las deps mientras estoy acá", hecho mientras la versión maliciosa está en vivo. La ventana se abre de tu lado del registro.

![Un diagrama en capas muestra instalaciones que honran el lockfile reproduciendo la versión registrada mientras los comandos de update se saltan el lockfile y resuelven hacia adelante hacia el riesgo.](assets/v06-diagram.webp)

¿Entonces qué instrumento usas? Por clase de dependencia, y a propósito. Rangos más un lockfile commiteado más instalaciones que lo honran es el default sensato: recibes auto-cura en los momentos que eliges, y reproducción en todos los demás. Los pins exactos se ganan su lugar en dependencias donde cualquier sorpresa es inaceptable y te comprometes a seguir los advisories a mano, el mismo trade-off que hizo agave con clap. Y un lockfile sin `npm ci` en CI es una decoración; revisa tus workflows, no tus intenciones. No hay ninguna configuración en la que no estés equivocado en algún lado. Un atacante que publica dentro de tu rango le gana al rango; un parche que nunca adoptas le gana al pin exacto; un lockfile regenerado le gana al lockfile. Estás eligiendo qué modo de falla prefieres por clase de dependencia y escribiendo la elección, y esa elección escrita es precisamente lo que es un veredicto. La seguridad por papeleo suena desalentadora hasta que el papeleo es lo único en la sala que se acuerda de por qué está el pin ahí.

**Profundiza (el 20%).** esta lección te enseñó a correr las herramientas y leer su salida; las herramientas van más a fondo que una sola lección. Los docs de cargo-audit en docs.rs/cargo-audit cubren el subcomando fix, la integración con CI y los auto-chequeos; cargo-deny en embarkstudios.github.io/cargo-deny extiende la auditoría a política de licencias y de fuentes para equipos que necesitan bans y allowlists; y la referencia de npm audit en docs.npmjs.com (CLI commands, npm-audit) documenta la verificación de firmas y el comportamiento exacto del endpoint de auditoría. Las tres URLs verificadas en vivo el 2026-09-02. Nada en el lab de abajo depende del material marcado como bookmark.

Una nota honesta de alcance, sin ningún hand-off atado: todo lo de arriba es higiene de cadena de suministro para el ciclo de vida del desarrollo, protegiendo el código que construyes y entregas. La seguridad de smart contracts, la auditoría de la lógica de programas on-chain, es un campo completamente distinto y este curso no la enseña ni pretende hacerlo.

## Lab

La capa de auditoría pasa por todo el parque: el workspace de TS (flota, core, panel), el workspace de Rust (motor, CLI y el daemon pollerd), los dos proyectos de edge que viajan por fuera de los dos, `pulse-edge-ts` (su propio proyecto de npm desde m07-l1, con su propia instalación de `@solana/kit` de la que el lockfile raíz no sabe nada) y `pulse-edge-rs`, y `tx-check`, el scaffold que m08-l4 armó para el camino de escritura, también su propio proyecto de npm por fuera del workspace y el árbol cuyas dependencias se sientan más cerca de una clave de firma. Cinco lockfiles, cinco pasadas. Unos 45 minutos. La parte trabajada es el paso 3; del paso 4 en adelante, la checklist es tuya y yo ya no estoy.

1. **Corre la auditoría de TS en la raíz del repo de la estación.** La raíz es donde vive `pnpm-lock.yaml`, y la auditoría lee el lockfile, así que la ubicación importa:

   ```bash
   pnpm audit
   ```

   Lee la línea de resumen antes que nada: cuántos advisories, en qué severidades, a lo largo de cuántos paquetes. En un repo con lockfile de npm el mismo comando es `npm audit`; nuestro workspace pasó a pnpm en m03-l1, y la auditoría sigue al lockfile.

   Después hazlo de nuevo en `pulse-edge-ts`. Ese worker es su propio proyecto de npm, creado por fuera del workspace en m07-l1, así que el lockfile raíz que acabas de auditar no dice nada sobre él, incluida su propia instalación de `@solana/kit`. (Kit vive en el workspace también, desde M8: la instalación raíz de m08-l1 y la de pulse-board de m08-l2; `pnpm why @solana/kit` dibuja esa mitad del mapa en segundos. Mismo paquete, y para el final de este paso, las jurisdicciones de tres lockfiles, que es exactamente la lectura de quién-cubre-qué que esta pasada existe para enseñar.) Hazle `cd` ahí y corre `npm audit`. Después la pasada que más importa por byte: hazle `cd` a `tx-check`, el proyecto independiente que m08-l4 armó con `npm init -y`, y corre `npm audit` sobre su lockfile también. Ese árbol guarda el pipeline de kit que toca tu par de claves de firma, y ni el lockfile raíz ni el de pulse-edge-ts dicen una palabra sobre él; sáltatelo y el barrido auditó todo excepto el código parado al lado de una clave privada. Tres comandos, tres lockfiles, y la mitad de TypeScript del parque queda cubierta. Dos lockfiles siguen intactos, los dos del lado de Rust, los dos en el próximo paso; "audité la estación" no es cierto hasta que estén hechos, y todo el punto de contar lockfiles es notar eso antes de decirlo.

2. **Corre la auditoría de Rust en la raíz del workspace de pulse-rs.** Instala primero si te salteaste la línea de instalación de la teoría:

   ```bash
   cargo install cargo-audit --locked
   cd pulse-rs
   cargo audit
   ```

   Escanea `Cargo.lock` contra la base de datos de RustSec, así que la raíz del workspace, donde vive el lockfile, es el único lugar correcto donde pararse. Un directorio de crate miembro sin su propio lockfile no te da nada. Después la quinta pasada, y no es opcional exactamente por la razón por la que `tx-check` no lo era: `pulse-edge-rs` se sienta por fuera del workspace con su propio `Cargo.lock` de m07-l2, así que nada de lo que corriste hasta ahora dice una palabra sobre él. Hazle `cd` ahí y corre `cargo audit` de nuevo. Cinco lockfiles, cinco pasadas, y ahora la oración del paso 1 es cierta.

3. **Lee un hallazgo de punta a punta, conmigo.** Si alguna de las dos herramientas marcó algo, ese es tu ejemplar. Lee el id del advisory y dilo en voz alta. Lee el rango vulnerable y contrasta contra él la versión que tienes en el lockfile. Lee la versión parcheada. Después traza el camino: `pnpm why <package>` o `cargo tree -i <crate>`, y clasifica la exposición: runtime, build-time o dev-only. Ahora escribe el veredicto en la forma de cuatro líneas de la teoría. Si las dos herramientas volvieron limpias, probable, ya que esta stack es fresca, la rep trabajada corre sobre la checklist en cambio: elige una dependencia de cualquiera de los dos lockfiles, corre las cinco lecturas de abandono sobre ella de verdad (`npm view <pkg> time.modified`, la página de releases, el issue tracker, los manifests emblemáticos), y escribe el mismo veredicto de cuatro líneas. De cualquiera de las dos formas, ya produjiste un veredicto con supervisión. Ese fue el último supervisado.

4. **Escribe `AUDIT.md` en la raíz del repo de la estación.** Estructúralo así: las dos líneas de resumen de las herramientas con la fecha de hoy, después tus veredictos. Mínimo dos veredictos, cada uno un encabezado de dependencia sobre los cuatro campos de la plantilla: lectura de señales, camino de exposición, decisión, acción. Los hallazgos reales primero si los tienes; si el árbol está limpio, elige dos dependencias deliberadamente, una de cada lockfile, y corre la checklist contra ellas. Elige al menos una en la que nunca pensaste conscientemente. La barrera nunca depende del humor que tenga la base de datos de advisories.

5. **La pasada de clasificación de pins.** Abre los dos manifests y los dos lockfiles. Para cada dependencia directa de los paquetes de TS y del workspace de Rust, clasifica qué dice el MANIFEST: un rango (caret, tilde o comparadores), un pin exacto, o un especificador tan suelto (pelado, wildcard o sin restricción) que el lockfile está haciendo todo el trabajo de fijar; llama a esa tercera etiqueta lockfile-only. Sí, con un lockfile commiteado toda dep con rango está TAMBIÉN fijada en la práctica; la clasificación es sobre la intención declarada del manifest, y la tercera etiqueta está reservada para deps cuyo manifest no declara ninguna. Una tabla en `AUDIT.md`, una fila por dep directa. Después responde la pregunta que decide si algo de esto importa, en una línea escrita: ¿instala CI desde el lockfile? Revisa el workflow de M1 buscando `pnpm install --frozen-lockfile` (el recableo de m03-l1; el trabajo de `npm ci` en grafía de pnpm), el Dockerfile de M6 buscando `--frozen-lockfile`, y anota lo que encuentres. Un lockfile del que nadie instala no hace cumplir nada.

![Una tarjeta de anatomía muestra las cuatro secciones requeridas de AUDIT.md, alimentadas por los dos comandos de auditoría y consumidas aguas abajo por el runbook de m10-l1.](assets/v07-diagram.webp)

6. **Haz commit.**

   ```bash
   git add AUDIT.md
   git commit -m "audit layer: tool reports, verdicts, pin classification"
   git push
   ```

   Este archivo no es de una sola vez. El runbook de m10-l1 lo consume directamente.

7. **Extensión opcional, claramente opcional: un reporte de CI que no hace de barrera.** Agrega un step de auditoría al workflow de M1 que reporte pero que nunca haga fallar el build:

   ```yaml
   - name: dependency audit (report only)
     run: pnpm audit || true
   ```

   (`npm audit || true` en un repo con lockfile de npm.) Que no haga de barrera es deliberado por ahora: todavía no decidiste, como política, qué hallazgos deberían bloquear un merge, y una barrera sobre la que no razonaste es una barrera que vas a saltarte la primera vez que te moleste. La lección del runbook revisita la pregunta con tus veredictos en la mano. Esta edición aterriza después del commit del paso 6, así que dale el suyo: `git add .github && git commit -m "ci: report-only dependency audit"` y push.

La vara de aceptación, sin rodeos: los dos comandos de auditoría corrieron en las raíces correctas con salida que puedes mostrar; `AUDIT.md` existe y está commiteado, con al menos dos veredictos en la forma de cuatro partes y la tabla de clasificación de pins cubriendo cada dependencia directa de los dos manifests; y la respuesta de CI de una línea está escrita.

## Challenge

Abre `pnpm-lock.yaml` y elige una dependencia transitiva de la que nunca oíste hablar. No una que elegiste; una que llegó como dependencia de una dependencia, del tipo de nombre que te hace decir "qué es eso y por qué lo entrego". Corre la checklist completa de cinco señales contra ella, en frío: README, último publish, cadencia, deriva de issues, quién más la fija. Escribe su veredicto de cuatro líneas y agrégalo a `AUDIT.md`. Sin apoyo y sin espiar mi lectura trabajada. El punto del ejercicio es que el método funciona sobre desconocidos totales, porque tu árbol es casi todo desconocidos totales, y después de hoy eso deja de ser un hecho incómodo que evitas y empieza a ser una lista que vas trabajando.

Después una segunda rep, en código, porque la verificación de rango que hiciste a ojo en el paso 3 es exactamente una función: el challenge semver-vuln-matcher, en el panel de coding-challenge como cada rep calificada de este curso. Un advisory nombra un rango vulnerable como `>=0.3.0 <0.3.11`, y la pregunta del auditor es si tu versión instalada se sienta adentro. Una nota de honestidad sobre ese rango antes de que confíes en él como historia: está escrito en gramática de advisory y toma prestados los números de versión de arrayref, pero es un rango de ejercicio, no el advisory del incidente real. El incidente de verdad tuvo exactamente un release hostil, 0.3.10, al que le hicieron yank en vez de parchearlo (nunca se entregó ningún 0.3.11 como arreglo), y la multitud que tenía 0.3.9 fijado en el lockfile nunca estuvo en peligro. El rango del ejercicio existe porque pone un patch de un dígito y un patch de dos dígitos a cada lado de una frontera, que es precisamente donde está el bug plantado: el `isVulnerable` del starter y su parseo de rangos están listos, y el bug vive en `compareVersions`, que compara las cadenas de versión como cadenas, y lexicográficamente `'0.3.9'` ordena por encima de `'0.3.10'`, porque el carácter `'9'` le gana a `'1'`. Arréglalo para que compare numéricamente, componente por componente, major después minor después patch. Siete pruebas lo califican, abriendo con los dígitos del incidente: la `0.3.10` maliciosa debe caer adentro del rango del ejercicio y la `0.3.11` afuera. Las pistas del starter escalan desde dónde está la mentira de la comparación de cadenas hasta el caso borde del componente faltante; gástalas en orden.

![Cuatro cadenas de versión ordenadas de dos formas, mostrando que la comparación de cadenas pone a 0.3.9 por encima de 0.3.10 mientras que la comparación numérica correctamente la pone por debajo.](assets/v08-table.webp)

## Checkpoint

Lo que ahora puedes hacer, concretamente: correr `npm audit` y `cargo audit` en las raíces donde viven sus lockfiles y decir por qué la raíz importa; leer un hallazgo en el orden correcto, id, rango, parche, camino, y solo después severidad, y clasificar la exposición como runtime, build-time o dev-only antes de reaccionar; correr la checklist de abandono de cinco señales sobre cualquier paquete, incluido uno que nunca viste; explicar qué te dice y qué no te dice una auditoría limpia; y defender una filosofía de pins por clase de dependencia, incluido exactamente qué comandos de instalación vuelven real el lockfile.

La recuperación de 30 segundos antes de que cierres la pestaña: ¿durante los 86 minutos en que arrayref 0.3.10 estuvo en vivo, quién estaba expuesto y quién no, y qué artefacto único hizo la diferencia? (Las resoluciones frescas dentro del rango compatible estaban expuestas; toda instalación que honró un lockfile commiteado no lo estaba. El lockfile fue la diferencia, y solo porque algo instaló desde él.) Si tuviste que mirar hacia arriba, vuelve a leer el diagrama en capas de instalación; ese único dibujo es esta lección.

Un pedido mientras está fresco: si las dos auditorías te volvieron limpias, dime en el feedback si los veredictos manejados por checklist del paso 4 se sintieron como trabajo de verdad o como trabajo inútil. El paso existe para que la barrera nunca dependa de que la base de datos de advisories tenga una mala semana, pero si se leyó como relleno para la mayoría de ustedes, la próxima revisión elige las dos deps por ti y las hace más desagradables.

Tu árbol de dependencias ahora tiene un rastro de auditoría escrito: herramientas corridas en las raíces correctas, veredictos que un desconocido podría seguir, pins clasificados y aplicados a propósito. Pero la estación misma sigue corriendo sobre confianza. Si el poller se muere en silencio esta noche, si el cron deja de dispararse, nada en ninguna parte hace un sonido. La próxima lección la estación aprende a observarse: logs estructurados que responden preguntas en vez de narrar, la realidad de logs de cada plataforma leída con honestidad, y una alarma cableada a la única falla que ningún panel muestra, la muerte del monitor mismo.
