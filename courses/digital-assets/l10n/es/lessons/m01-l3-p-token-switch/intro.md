# El cambio a p-token: interfaz congelada, motor nuevo

La lección pasada decodificaste un mint que nunca creaste y te adueñaste de la lectura. Allá en m01-l1 viste una transferencia clásica costar 76 unidades de cómputo. Acá está la parte inquietante, y es la lección entera: esa misma transferencia solía costar 4,645 CU. El layout de 82 bytes que decodificaste no cambió. Tu código de cliente no cambió. El código de cliente de nadie cambió. Y aun así el programa que corre en `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` fue cambiado en silencio bajo los pies de todos.

Antes de explicar nada, ve a mirar el interruptor mismo: una cuenta real de mainnet, legible ahora mismo. Esta es la primera lección que echa mano del CLI `solana` — y déjame ser preciso con la palabra "necesita": cada sondeo de abajo es una lectura RPC simple, y el sondeo 3 del lab hace exactamente la misma lectura del gate con el código de kit que ya tienes, así que nada en esta lección está condicionado a una instalación local. Las vistas formateadas del CLI son la forma cómoda de correr los otros sondeos, eso sí, y los labs posteriores de este curso se apoyan en el toolchain local de verdad, así que si lo quieres ahora, una línea te trae el toolchain entero de Agave (yo estoy en solana-cli 3.1.10, línea Agave, verificado 2026-08-22):

```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

Después pregúntale a mainnet por una dirección muy particular:

```bash
solana feature status ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
```

Deberías ver:

```text
Feature                                      | Status                  | Activation Slot | Description
ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP  | active since epoch 971  | 419472000       | SIMD-0266: Efficient Token program
```

Esa fila es el cambio de motor. Un feature gate, activo desde el primer slot del epoch 971, y el programa de token que sostiene cada saldo SPL en Solana se volvió una pieza de software distinta. Volví a sondear ese gate esta mañana, 2026-08-22, antes de escribir una palabra de esta lección; la salida de arriba es lo que devolvió mainnet. Deja esa terminal abierta. Esa fila formateada es cómo el CLI representa una cuenta diminuta de 9 bytes, y en el lab vas a volcar esos 9 bytes en crudo y leerlos tú mismo.

## Resumen

En m01-l2 construiste el inspector `decode-mint` y leíste los campos base de un mint pelado clásico de 82 bytes directo de los bytes. Esta lección explica algo extraño que ya viste dos veces sin saberlo: el programa que procesó tu transferencia de 76 CU no es el programa que procesó las transferencias de todos durante los seis años anteriores. SIMD-0266 reemplazó la implementación del SPL Token clásico en la misma dirección por p-token, la reescritura en Pinocchio de Anza, y lo hizo sin cambiar un solo byte de la interfaz. Acabas de sondear el feature gate que accionó el interruptor; en el lab vas a leerle el cambio a la cuenta del programa misma, y te vas a llevar el modelo mental que 2026 le impone al "SPL clásico": una interfaz congelada corriendo un motor completamente nuevo. Hoy no hay peldaño nuevo en la escalera de artefactos, ni TODO de completion tampoco: el lab son sondeos guiados que corres tal como se muestran, y el challenge es totalmente solo, en palabras y no en código. Ese es el repliegue para una lección de concepto.

## Interfaz versus implementación

Empieza por el modelo mental que probablemente trajiste a este curso, porque es el que enseña casi todo tutorial. Un programa vive en una dirección. La dirección identifica el código. "Conozco el programa SPL Token" quiere decir "sé qué hace el código en TokenkegQ...". Bajo ese modelo, una caída de costo de 61x con cero cambios de cliente debería ser imposible. Así que algo en el modelo está mal, y averiguar qué es más valioso que el número mismo.

Primero agota las explicaciones ingenuas, porque cada falla afila la pregunta.

**Respuesta ingenua uno: entregaron un programa nuevo y todos migraron.** No lo hicieron. No hay dirección nueva. El mint clásico de USDC que leyó tu inspector `decode-mint` la lección pasada sigue siendo propiedad de la misma dirección TokenkegQ... que billeteras, DEXes y puentes tienen hardcodeada desde 2020, y ninguno de ellos entregó una migración. Si hubiera pasado una migración de este tamaño, la habrías sentido: actualizaciones coordinadas en cada cliente de la blockchain, ventanas de deprecación, integraciones rotas. El silencio del ecosistema es evidencia.

**Respuesta ingenua dos: el runtime se volvió más rápido, así que todo se abarató.** También no, y tus propios números lo refutan. Las unidades de cómputo no son tiempo de reloj de pared; son un conteo medido del trabajo que el runtime cobra por operación. Un validador más rápido ejecuta la misma transferencia de 4,645 CU en menos tiempo, pero sigue cobrando 4,645 CU. Para que el número medido baje, el trabajo mismo tiene que encogerse. Algo cambió dentro de lo que se ejecuta.

**Respuesta ingenua tres: el programa se actualizó en el lugar, de la forma normal.** Más cerca, pero todavía equivocado de una forma instructiva. Los programas actualizables comunes reciben código nuevo a través de su autoridad de actualización. El programa de token clásico es, sin competencia, el programa más estructural de Solana; entregarle a un único titular de clave el poder de cambiarlo en caliente sería una historia de seguridad, no de eficiencia. Lo que sea que lo reemplazó necesitaba algo más fuerte que una clave de actualización: consenso.

![Tres explicaciones fallidas para la caída de CU, cada una tachada junto a la evidencia que la refuta, embudando hacia la única pregunta que sobrevive a la eliminación.](assets/v01-comparison.png)

Así que la pregunta real es más acotada que "por qué es barato ahora". Es: **¿qué mecanismo puede reemplazar el código detrás de una dirección, con el acuerdo de toda la red, sin romper un solo llamador?** Esa pregunta tiene exactamente una respuesta en Solana, y acabas de sondearla.

![Los clientes llaman a la misma dirección TokenkegQ y a la misma interfaz congelada, pero detrás de ella el motor spl-token original fue reemplazado por el p-token de Anza en el epoch 971 mediante un feature gate.](assets/v02-diagram.png)

### El mecanismo: un feature gate sobre una interfaz congelada

Un **feature gate** es un interruptor on-chain. Es una cuenta diminuta, propiedad del programa Feature, cuya existencia y cuyo slot de activación le dicen a cada validador "de este slot en adelante, compórtate de la forma nueva". Los validadores llevan los dos comportamientos en sus binarios; el gate decide cuál está vivo, y como el gate es estado on-chain, cada validador cambia en el mismo slot. Así es como Solana entrega cambios críticos para el consenso sin un hard fork, y es el mecanismo que cambió tu motor de token.

Para sentir por qué ese diseño es notable, contrástalo con las alternativas con las que otros ecosistemas viven de verdad. La opción uno es la migración: desplegar el programa nuevo en una dirección nueva y pedirle a cada billetera, DEX, puente e indexador del planeta que se mueva, en sus propios calendarios, con una ventana de deprecación y una larga cola de rezagados. Esa es la respuesta ingenua uno, hecha a propósito, y los ecosistemas que toman este camino pasan años en la transición. La opción dos es el hard fork coordinado: cada operador actualiza antes de una fecha de corte o se cae de la red. El feature gate es un tercer camino que conserva las partes buenas de ambos: el código nuevo ya viene dentro del release normal del validador, dormido, a veces por meses, y el gate on-chain elige el momento. Un solo límite de slot, sin ventana de motores mezclados, sin migración, y la dirección que cada cliente hardcodeó se queda exactamente donde estaba.

La propuesta detrás del cambio es **SIMD-0266**. Se mergeó el 2026-03-13. Di mergeado, y no "aceptado" ni "aprobado", y acá va un hábito de precisión que vale la pena construir: mergeado es un hecho de Git que puedes verificar (el PR entró, con fecha), mientras que aceptado y aprobado son afirmaciones de gobernanza, y el front-matter del propio documento todavía dice "Review" hoy, así que el rastro documental no sostiene ninguna de las dos. El encabezado es una etiqueta que va con retraso. Lo que de verdad gobierna la activación es el gate on-chain que acabas de sondear, y ese gate está activo. Un compañero que lee "Review" y concluye que p-token todavía no está vivo le creyó a un encabezado de documento por encima del estado de la blockchain, que en Solana siempre es el orden equivocado.

![Línea de tiempo desde el merge de SIMD-0266 el 2026-03-13, pasando por la activación del gate en el slot 419,472,000 (primer slot del epoch 971), hasta el re-sondeo en vivo del gate el 2026-08-22.](assets/v03-timeline.png)

Lo que el gate activó es **p-token**: una reescritura desde cero del programa SPL Token clásico hecha por Anza, escrita en Pinocchio, un framework de programas de cero dependencias y cero copias, construido exactamente para este tipo de trabajo de ruta caliente. El contrato de la reescritura con el ecosistema fue brutal y simple: idéntico byte por byte en layouts de cuenta, discriminadores de instrucción y códigos de error. Instrucción por instrucción y error por error. Cada offset que recorre tu inspector `decode-mint`, cada discriminador que manda una billetera, cada código de error contra el que hace match una integración: idénticos. Esa superficie idéntica es la **interfaz**. El código que la honra es la **implementación**. SIMD-0266 reemplazó la implementación y congeló la interfaz, y esa separación es todo el truco.

Un lector agudo debería objetar justo acá, así que tomemos las dos objeciones más fuertes en orden. Primera: "idéntico byte por byte" es una afirmación, no una propiedad que te dan gratis, y el costo de equivocarse no es un ticket de bug. Si p-token discrepara del motor viejo aunque sea en una sola entrada, los validadores que corren un comportamiento calcularían un estado distinto que los validadores que corren el otro, en el programa más llamado que existe en la blockchain. Eso es territorio de riesgo de consenso. La interfaz congelada es lo que hace que la afirmación sea siquiera verificable: el comportamiento observable del motor viejo ES la especificación, ejecutable y exhaustiva, así que para cualquier instrucción puedes darle a los dos motores los mismos bytes y exigir las mismas salidas, las mismas transiciones de estado y los mismos códigos de error. Lo mismo entra, lo mismo sale, o la reescritura está mal. Y el gate es lo que vuelve seguro actuar sobre la afirmación: en vez de un despliegue progresivo donde el código viejo y el nuevo se superponen por horas, hay un solo slot inequívoco antes del cual todos corren el motor viejo y después del cual todos corren el nuevo.

Segunda objeción, y merece una respuesta directa: mecánicamente, nada impide que el consenso meta algo malicioso detrás de esa misma dirección. No pases de largo. Un feature gate se activa porque el conjunto de validadores, ponderado por stake, adopta releases que llevan el cambio y el proceso de activación corre; no hay garantía criptográfica de que lo que se activa sea benigno. Pero fíjate que este fue siempre el modelo de confianza. El consenso ha definido qué es la blockchain desde el bloque génesis; una supermayoría del stake siempre pudo haber cambiado cualquier regla. El gate no creó ese poder. Lo hizo visible, sellado con un slot y legible en una cuenta de 9 bytes, que es estrictamente mejor que invisible. Tu trabajo como builder no es fingir que el poder no existe; es saber qué capa del sistema lo tiene.

Acá es donde el viejo modelo mental se corrige en vez de descartarse. La dirección nunca identificó el código. Identificó el *contrato*: los layouts de bytes y los comportamientos en los que puede confiar cualquiera que llame a esa dirección. El código es apenas el inquilino actual que honra ese contrato. Hay un viejo experimento mental sobre un barco cuyas tablas se reemplazan una por una hasta que no queda nada de la madera original, y los filósofos discuten si sigue siendo el mismo barco. La respuesta de Solana no es nada sentimental: si cada tabla de la interfaz es idéntica byte por byte, es el mismo programa, sin importar quién escribió la madera. La analogía se rompe en un punto que vale la pena señalar, eso sí: las tablas de Teseo se cambiaron poco a poco y por accidente del mantenimiento. Este cambio pasó en toda la red en un solo slot, por diseño, con el reemplazo probado contra el comportamiento exacto del original antes de que el gate se accionara siquiera. Identidad deliberada, no identidad a la deriva.

![Una división en dos columnas que muestra la interfaz congelada (dirección, layouts, discriminadores, errores, comportamiento) frente a la implementación reemplazada (código del motor, costos de CU, binario), más tres instrucciones agregadas.](assets/v04-comparison.png)

### El beneficio, medido

Ahora los números pueden significar algo. Un Transfer clásico costaba 4,645 CU bajo el motor viejo. Bajo p-token cuesta 76 CU. TransferChecked, la variante que vas a usar en todos lados porque valida el mint y los decimals, cayó de 6,200 a 105 CU. Esas no son optimizaciones incrementales; son lo que pasa cuando un programa Rust de propósito general de la era 2020 lo reescribe gente que cuenta cada syscall. Y como las transferencias de token son la clase de instrucción más común que hay en la blockchain, el efecto agregado es de escala macro: alrededor de 12 a 13 por ciento del espacio total de bloque se recuperó, según Anza (su ingeniero Febo recorrió los números en una entrevista publicada en mayo de 2026). Doce por ciento de la capacidad de una blockchain, devuelto a la red por una reescritura a la que nadie tuvo que sumarse.

Detente en qué es esa recuperación en realidad, porque el encuadre importa. Los bloques no se hicieron más grandes, y ningún parámetro de consenso se movió. El mismo presupuesto de CU por bloque simplemente dejó de gastarse en overhead de transferencias de token: el trabajo que antes facturaba 4,645 unidades por transferencia ahora factura 76, y la diferencia es capacidad que cada otra transacción de la blockchain puede usar. Es el tipo raro de victoria de escalabilidad que no le cuesta nada al resto del sistema. Eso sí, el porcentaje mismo es una cifra de una era medida, no una constante: refleja cuánto del tráfico de la blockchain eran transferencias de token cuando Anza lo midió, así que cítalo como su número, con la fecha, de la forma en que acabo de hacerlo.

![Gráfico de barras que muestra Transfer cayendo de 4,645 a 76 CU y TransferChecked de 6,200 a 105 CU después del cambio a p-token, recuperando alrededor de 12 a 13 por ciento del espacio de bloque.](assets/v05-chart.png)

Ten cuidado con la atribución, porque esta es la trampa que te va a hacer quedar en ridículo en una revisión de código. La caída es obra del motor, no tuya. Si mediste una transferencia el año pasado en 4,645 CU y mides el mismo código de cliente hoy en 76, tu código no mejoró. Nada de lo que despliegues, ningún flag que actives, ninguna actualización de SDK que entregues se lleva crédito alguno por esos números. El motor cambió por debajo de ti. Lo cual corta para el otro lado también, y esta es la advertencia honesta: el 76 es una medición dependiente del motor, no una constante de la naturaleza. Congela "una transferencia cuesta 76 CU" en un config o en un doc y lo vas a citar mal la próxima vez que el motor o el modelo de costos del runtime se muevan. Cítalo como "76 CU a partir del motor p-token, epoch 971", y vuelve a medir cuando importe. El número que el motor viejo le enseñó a todos a memorizar acabó de volverse un cuento con moraleja; no crees el siguiente.

Un límite que respetar, y es deliberado. Esta lección enseña la caída de CU como un hecho de la capa de token: qué cambió, cuándo, y qué quiere decir para tu modelo mental. No deriva *por qué* 76 CU es físicamente alcanzable, porque ese porqué vive en el loader, en la máquina virtual sBPF y en la maquinaria de medición de cómputo, y toda esa capa le pertenece al curso planificado Low-Level Solana. Si te descubres queriendo saber a dónde va cada una de las 76 unidades, ese curso es el lugar; acá, el número es evidencia del modelo interfaz-versus-implementación, y ese modelo es la carga útil.

### Congelado significa congelado: dónde viven las nuevas capacidades

La palabra que Anza usa ahora para el SPL clásico es **feature-complete**, y la traducción práctica es: congelado. No hay funcionalidad de token nueva planeada para el programa clásico, nunca. La reescritura de p-token sí agregó tres instrucciones nuevas, `batch`, `withdraw_excess_lamports` y `unwrap_lamports`, lo que suena a contradicción hasta que notas qué tipo de instrucciones son: comodidades operativas que se atornillan a la superficie existente sin perturbar un solo byte existente. Agrupar en lote lo que ya podías hacer uno a la vez no es una capacidad nueva; es plomería. La regla que te importa a ti como diseñador es esta: **el comportamiento de token genuinamente nuevo aterriza solo en Token-2022.** Transfer hooks, saldos confidenciales, metadatos nativos, comisiones de transferencia, todo eso, territorio de extensiones. El SPL clásico en 2026 es un contrato congelado con un inquilino muy rápido, y si te descubres esperando que el SPL clásico crezca una funcionalidad, estás esperando un tren que fue formalmente cancelado.

![Flujo de decisión: si el SPL clásico ya hace lo que necesitas, úsalo tal cual; sus únicas adiciones son tres instrucciones de plomería de p-token; cada capacidad genuinamente nueva se enruta a Token-2022.](assets/v06-flowchart.png)

Este reencuadre también te entrega un filtro para cada pieza de contenido de Solana escrita antes de 2026, y lo vas a necesitar, porque internet no le pone fecha a sus modelos mentales. Cuando un tutorial viejo, una nota de auditoría o una respuesta de foro hace una afirmación sobre "el programa de token", pásala por una sola pregunta: ¿es esta una afirmación sobre la interfaz, o sobre la implementación? Las afirmaciones de interfaz envejecieron perfecto. El layout de mint de 82 bytes, el conjunto de instrucciones, los discriminadores, los códigos de error: todos siguen siendo ciertos, byte por byte, porque congelarlos era todo el trato. Las afirmaciones de implementación envejecieron mal de la noche a la mañana. Cualquier cosa sobre la estructura interna del programa, sus características de rendimiento, sus costos de CU por instrucción: ese contenido ahora describe un programa que ya no corre en ningún lado. Las afirmaciones estaban bien cuando se escribieron. El inquilino cambió. Una pregunta, dos cajones, y puedes rescatar seis años de escritura del ecosistema en vez de desconfiar de toda ella.

El congelamiento también se ve en los repositorios y en los crates, y te vas a tropezar con esto en labs posteriores si no lo escuchas ahora. El viejo monorepo SPL de solana-labs, ese al que enlaza cada tutorial de la era 2022, fue disuelto; los programas de token ahora viven en la organización solana-program de GitHub, mantenida por Anza, un repo por programa. La misma interfaz, una casa nueva y un motor nuevo, y la división de repos no es cosmética: un monorepo tenía sentido cuando un solo equipo entregaba todo junto, y a un programa congelado y feature-complete no le queda ningún "junto" que entregar. Cada programa ahora versiona y publica por su cuenta. Y del lado de Rust, la división tiene su propio crate: `spl-token-2022-interface` ahora entrega los tipos y los layouts por separado de `spl-token-2022`, que sigue siendo el crate del programa. Lee esa división con tu vocabulario nuevo: el ecosistema está literalmente dividiendo sus crates para decir de qué lado de la línea interfaz/implementación se sienta cada uno, y los dos versionan de forma independiente — el crate de interfaz estaba en 3.1.1 y el crate del programa en 11.0.0 cuando revisé crates.io el 2026-09-06. Sé preciso sobre lo que eso es y lo que no es: el crate del programa no está yanked y no lleva aviso de deprecación a nivel de crate; lo que está deprecado es a nivel de ítem, re-exports individuales dentro de él que te apuntan al crate de interfaz. Cuando una lección posterior te haga calcular tamaños de cuenta contra tipos de Token-2022, el import viene del crate de interfaz. Una dependencia sobre la implementación es una dependencia que no necesitabas.

### El trade-off, nombrado

Todo regalo en este diseño tiene su sombra, así que nombra los dos con franqueza. Una interfaz congelada con un motor intercambiable es un regalo para la compatibilidad: tu código nunca se rompe, tus integraciones nunca migran, y toda la red hereda una mejora de 61x mientras duerme. También es una trampa para los modelos mentales: la dirección que llamas es estable, pero lo que corre detrás no lo es, así que "conozco el programa SPL Token" ahora quiere decir "conozco su interfaz", y nunca más puede querer decir "conozco su implementación". Cualquiera cuyo razonamiento de seguridad, presupuesto de CU o intuición de rendimiento dependía en silencio de detalles de implementación del motor viejo vio esas suposiciones invalidadas en el slot 419,472,000, y la blockchain no le mandó ninguna notificación. La garantía de compatibilidad es real y la degradación epistémica es real, y sostienes las dos a la vez. Ese es el modelo 2026 del SPL clásico, y es el diferenciador que casi nadie enseña: las interfaces son promesas, las implementaciones son inquilinos, y en Solana el inquilino puede cambiar bajo consenso en un solo límite de slot.

## Lab: lee el cambio directo de la blockchain

Cuatro sondeos, todos guiados, nada que completar. Hoy no estás construyendo; estás verificando que todo lo de arriba es legible en la blockchain y no folclore. Tiempo total: unos quince minutos una vez instalado el toolchain de Agave de la apertura; la instalación es un costo de una sola vez que puede durar más que los sondeos. ¿Te saltaste la instalación? Los sondeos 1, 2 y 4 son representaciones que hace el CLI de lecturas de cuenta simples — corre el sondeo 3, que hace la misma lectura del gate en kit, y toma por buenos los checkpoints del CLI impresos acá.

1. **Sondea el gate con el CLI.** Si corriste el comando feature-status de arriba, ya hiciste este paso; si no, córrelo ahora:

   ```bash
   solana feature status ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
   ```

   Checkpoint: la fila dice `active since epoch 971` con slot de activación `419472000`. Si tu CLI imprime un error de conexión, probablemente estás detrás de un límite de tasa de RPC por defecto; espera unos segundos y reintenta.

2. **Vuelca la cuenta del gate en crudo.** La vista de feature del CLI es un envoltorio de conveniencia; la verdad son 9 bytes de datos de cuenta, y después de m01-l2 estás calificado para leer 9 bytes con tus propios ojos:

   ```bash
   solana account ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP --url mainnet-beta
   ```

   Checkpoint: el owner es `Feature111111111111111111111111111111111111`, la longitud es 9 bytes, y el volcado hex dice `01 80 a2 00 19 00 00 00 00`.

![Volcado hex anotado de los 9 bytes que guarda la cuenta de feature: una etiqueta Some de 1 byte seguida por el slot de activación u64 little-endian 419,472,000, el primer slot del epoch 971.](assets/v07-annotated-code.png)

3. **Decodifícala programáticamente, al estilo kit.** La misma lectura, pero a través del stack sobre el que construiste `decode-mint`, así la habilidad se acumula. Trabaja dentro de `labs/m01-l2`, el workspace que levantaste la lección pasada: kit 7.1.1 y tsx 4.20.5 ya están fijados ahí, versiones exactas según la regla de rango de peers que fija esa lección, y su `package.json` lleva el `type=module` que necesita el top-level await de este script. En esa carpeta, crea `read-gate.ts`:

   ```typescript
   import { createSolanaRpc, address } from '@solana/kit';

   const GATE = address('ptokFjwyJtrwCa9Kgo9xoDS59V4QccBGEaRFnRPnSdP');
   const rpc = createSolanaRpc(process.env.RPC_URL ?? 'https://api.mainnet-beta.solana.com');

   const { value: account } = await rpc
     .getAccountInfo(GATE, { encoding: 'base64' })
     .send();

   if (!account) {
     console.log('No account at the gate address: the feature is not even pending.');
     process.exit(1);
   }

   const data = Buffer.from(account.data[0], 'base64');
   console.log(`owner:  ${account.owner}`);
   console.log(`bytes:  ${data.toString('hex')} (${data.length} bytes)`);

   // Feature account layout: 1-byte Option tag, then u64 LE activation slot.
   if (data[0] === 0) {
     console.log('status: pending activation (no slot set)');
   } else {
     const activatedAt = data.readBigUInt64LE(1);
     console.log(`status: ACTIVE since slot ${activatedAt.toLocaleString('en-US')}`);
   }
   ```

   Córrelo:

   ```bash
   npx tsx read-gate.ts
   ```

   Checkpoint: tres líneas, terminando en `status: ACTIVE since slot 419,472,000`. Fíjate en el `readBigUInt64LE`: la misma honestidad de BigInt que tu campo supply en m01-l2, porque un slot de activación es un u64 y a los números de JavaScript no se les puede confiar uno.

4. **Atrapa al motor con las manos en la masa.** La cuenta del programa misma registró el cambio:

   ```bash
   solana program show TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA --url mainnet-beta
   ```

   Checkpoint, de mi sondeo del 2026-08-22: `Last Deployed In Slot: 419472000`, `Authority: none`, longitud de datos 108,600 bytes. Quédate un segundo con esa primera línea. El programa más llamado de Solana reporta su último despliegue exactamente en el slot de activación del gate, el primer slot del epoch 971: la huella digital de un cambio de código impulsado por consenso, escrita donde cualquiera puede leerla. Y `Authority: none` responde la respuesta ingenua tres de la sección de teoría de una vez por todas: no existe ninguna clave de actualización para cambiar este programa de la forma ordinaria. Solo un gate pudo haberlo hecho, y uno lo hizo.

   Quinto sondeo opcional, del lado de Rust: `cargo info spl-token-2022` (cargo ya viene con rustup; `curl https://sh.rustup.rs -sSf | sh` si nunca lo instalaste) muestra que el crate ahora vive bajo los repos de solana-program, e imprime su versión actual — 11.0.0 en mi lectura del 2026-09-06, sin aviso de deprecación en el crate mismo. Corre `cargo info spl-token-2022-interface` al lado y vas a ver el crate de interfaz versionado por separado (3.1.1 en la misma lectura), que es de donde van a leer nuestros labs posteriores adyacentes a Rust. Dos crates, dos líneas de versión, una interfaz: exactamente la división que esta lección ha estado defendiendo.

## Challenge

Hoy no hay código. El criterio de evaluación de esta lección es una oración, y es más difícil de lo que suena. Escribe, con tus propias palabras y sin volver a mirar el texto, una respuesta de tres partes sobre la que un colega pueda actuar: una oración que separe la interfaz de la implementación del SPL clásico, una que enuncie su estado en mainnet con precisión (qué se activó, y cuándo, en epochs), y una que atribuya la caída de CU de transferencia a la causa correcta. Después ponte a prueba contra las dos trampas que esta lección armó: si tu oración de estado dice "aprobado" o "aceptado", hiciste una afirmación de gobernanza que el rastro documental no sostiene (el PR se mergeó el 2026-03-13, un hecho de Git, y el front-matter todavía dice "Review"); si tu oración de CU deja que un cambio del lado del cliente se lleve algún crédito, vuelve a leer la sección del beneficio. Cuando tus tres oraciones sobrevivan a las dos trampas, te adueñas del modelo.

Si un colega te rebate con "pero la página del SIMD dice Review", ahora conoces la corrección, y generaliza: los encabezados de documento van con retraso, el estado de la blockchain no, y tú personalmente leíste los 9 bytes que lo zanjan.

¿Algo de acá no te cuadra, o tus sondeos devolvieron algo que los míos no? Dímelo: márcalo en el canal de feedback del curso, idealmente con el comando y la salida pegados. Un lector que caza un número viejo en esta lección hace exactamente lo que esta lección enseña.

La próxima lección, la pregunta que este módulo entero ha estado rodeando deja de ser retórica. El SPL clásico está congelado, cada capacidad genuinamente nueva vive en Token-2022, y Token-2022 ofrece 29 tipos de extensión con reglas reales sobre qué combinaciones son siquiera legales. ¿Entonces cómo decides? Derivas el framework de decisión desde el código fuente, y tu inspector `decode-mint` vuelve al trabajo.

¡Feliz sondeo! 🌱
