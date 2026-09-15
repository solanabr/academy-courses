# Ensamblaje: la estación, entera

## Resumen

m09-l2 puso la capa de ops sobre cada superficie: el poller emite logs JSON estructurados de los que grepeaste un incidente de verdad, la realidad de logs de cada plataforma está mapeada y escrita, la alarma de notificación de fallas de Actions está prendida, y la tabla del barrido de secretos de cuatro plataformas volvió limpia. Lo que quiere decir que no queda nada por construir antes de que eso que llevas diez módulos construyendo se ensamble y quede demostrado. Hoy verificas la Pulse Station completa de punta a punta, arista por arista, contra un script de demo que escribes a medida que avanzas; construyes la única pieza de cableado nueva del capstone; escribes el README desde el que otro dev podría operar la estación; y después entregas una extensión sin ningún apoyo. El repliegue, dicho en voz alta porque acá se completa: el ensamblaje va guiado por checklist, el panel nuevo va semiguiado (yo nombro la composición, tú escribes el código), y la extensión es totalmente en solitario. Nada de ejemplos trabajados en ninguna parte de esta lección salvo una excepción deliberada, el check de frescura del paso 3, trabajado completo porque es el punto exacto donde nacen los scripts de demo en falso verde. No has necesitado más que eso desde hace dos módulos, y pretender lo contrario ahora sería un insulto.

## Dibújalo antes de construirlo

Antes de cablear nada, baja la tapa de la laptop sobre los docs y dibuja tu estación de memoria. Papel, una pizarra, el reverso de un recibo, lo que sea. Cada componente, su lenguaje, su plataforma, y cada arista de flujo de datos, en la forma de hub. Tienes treinta segundos y acá está la forma de la respuesta por adelantado, porque este checkpoint está diseñado para ser una victoria:

```text
      [ spoke ]        [ spoke ]
            \            /
[ spoke ] -- ( one pipeline ) -- [ spoke ]
                  |
              [ spoke ]

arrows only where data actually moves
```

Un pipeline en el medio como el latido, radios independientes alrededor, flechas solo donde los datos de verdad se mueven. Si puedes dibujarlo en treinta segundos, ya entiendes un sistema distribuido políglota lo bastante bien como para ensamblarlo. Si una flecha se siente borrosa, esa borrosidad es exactamente lo que queman las próximas tres horas.

Anda. Dibújalo. Después vuelve y revísate contra la referencia.

![Un diagrama de hub y radios con un pipeline central, cinco componentes alrededor, flujos de datos reales dibujados en línea sólida y tres flechas prohibidas tachadas.](assets/v01-diagram.webp)

Date el puntaje con honestidad. Los componentes son la mitad fácil; la mayoría saca los seis. Las aristas son donde el dibujo se gana sus treinta segundos, y las flechas equivocadas importan más que las correctas. El error clásico, y lo dibujé yo mismo la primera vez que bosquejé este diagrama para el temario del curso, es una flecha del poller al panel. Se siente como que debería existir. El poller tiene los datos más ricos de la estación, lecturas de la blockchain incluidas, y el panel es la cara. Pero no existe tal arista, y tampoco existe ninguna arista hacia adentro del poller: corre en tu máquina, golpeado desde tu máquina, sin ninguna superficie pública. Las otras dos trampas: una flecha de cualquiera de los dos workers hacia el poller (los workers son deliberadamente independientes, ese es todo su punto), y una flecha de GHCR a un contenedor corriendo en algún lado de la nube (un registro es almacenamiento, no hosting, que es la forma corta que vale la pena acuñar acá para lo que enseñó m06-l4; el pull pasa en tu máquina).

Acá está la síntesis que vale la pena llevarte de este curso: un sistema distribuido son solo programas que se ponen de acuerdo sobre un diagrama. Eso no es una metáfora. Cada componente que desplegaste cumple su parte de exactamente un contrato, las flechas de este dibujo, y nada más. Dibujaste el diagrama de memoria. El resto de esta lección es hacer que la realidad le coincida, una flecha a la vez, con un recibo para cada una.

## La estación, arista por arista

Dos definiciones antes del recorrido, las dos las vas a usar por el resto de tu carrera.

Una **topología de hub** es la forma que tiene tu estación: un latido en el medio, radios independientes alrededor. La alternativa que vale la pena nombrar es una cadena, donde A alimenta a B, que alimenta a C, que alimenta a D, y que muera cualquier salto se lleva todo lo que está aguas abajo. Tu estación no tiene cadenas más largas de un salto. Que el panel se caiga no afecta a nada más que al panel. Que un worker se caiga deja intactos a su gemelo, al poller y al pipeline. Solo el hub es estructural para el sistema como un todo, y m09-l2 gastó una lección en asegurarse de que la muerte del hub sea ruidosa.

Un **script de demo** es el ejercicio del runbook que demuestra el sistema: un check por arista, cada uno imprimiendo OK o FAIL, que se puede volver a correr a demanda. Es la diferencia entre "creo que mi estación funciona" y "acá está la transcripción." Lo vas a escribir a medida que verificas, un check por arista, lo que quiere decir que para el final del lab la demostración y el sistema existen como un par. Ese emparejamiento es el entregable de verdad de un capstone. Cualquiera puede ensamblar algo una vez; el script de demo es lo que lo vuelve operable.

Como estás por escribir un script entero de ellos, uno por arista y alrededor de diez en total una vez que la arista 5 recibe un check por worker y la arista 8 uno por herramienta de auditoría, la pregunta de gusto vale treinta segundos: ¿qué hace que un check sea confiable? Tres propiedades. Observa la afirmación, no el transporte: un HTTP 200 desde un CDN dice "un cache tiene bytes," mientras que un timestamp adentro del payload dice "mi cron corrió dentro de la hora," y solo una de esas dos cosas es la que de verdad te importa. Se puede volver a correr sin estado manual: nada de "primero borra el contenedor viejo," nada de "funciona si corriste el otro script hace poco," porque un check con instrucciones de preparación es una tarea doméstica, no un check. Y falla a gritos con una razón, porque `FAIL` sin explicación solo mueve la depuración a un momento peor. Cada check que escribas hoy debería sobrevivir las tres preguntas, y el que te trabajo en el paso 3 está elegido precisamente porque es donde la mayoría escribe la versión vanidosa.

![Dos tarjetas contrastan un check que solo demuestra que un cache respondió con un check que demuestra que los datos mismos están frescos.](assets/v02-comparison.webp)

Entonces: las ocho aristas, cada una nombrada con el módulo que la construyó, porque este recorrido funciona además como la última recuperación espaciada del curso. Lee la columna del medio despacio y fíjate que es una tabla de contenidos de tus últimas diez semanas.

| # | Arista | Construida por | Demostración |
|---|---|---|---|
| 1 | Pipeline de Actions en verde, seis jobs | m01-l3, barreras de m02-l4 + m04-l3, jobs de m05-l3 + m06-l4 | la última ejecución completada tuvo éxito |
| 2 | El cron publica status.json | m01-l3, tipado por m02 | timestamp del payload con menos de 60 min |
| 3 | El panel renderiza los paneles de flota + Solana | m03-l2, m03-l3, m08-l2 | los dos paneles en vivo en la URL de Vercel |
| 4 | El panel consulta al worker de TS (NUEVO) | hoy, desde m03-l2 + m07-l1 | tercer panel en vivo |
| 5 | Dos workers, independientes | m07-l1, m07-l2, lecturas de la blockchain m08-l2 | las dos URLs de workers.dev responden con JSON por target |
| 6 | El poller de GHCR corre local | m06-l2 hasta m06-l4, lecturas de la blockchain m08-l3 | localhost:8080/status responde con datos de la blockchain |
| 7 | Camino de escritura sano | m08-l4 | tx-check sale 0 con una firma confirmada |
| 8 | Auditoría + alarma en verde | m09-l1, m09-l2 | auditorías limpias o con veredicto, notificaciones confirmadas prendidas |

La arista 1 se merece un párrafo porque es el hub, y porque verificarla es un ejercicio de lectura, no de construcción. Tu único workflow creció durante diez módulos: la barrera de vitest llegó en m02-l4, las barreras de cargo test, clippy y fmt en m04-l3, el job del binario de release en m05-l3, los pushes de imagen a GHCR en m06-l4. Las pruebas en los dos lenguajes le ponen una barrera al cron. El cron sondea y hace commit de `status.json`. Nada de esto se construye hoy; hoy lees la última ejecución como un operador y revisas que cada job de la cadena se puso en verde. Y mientras estás ahí adentro, fíjate en la cosa que parece un bug y no lo es: el commit de `status.json` del propio cron nunca vuelve a disparar el workflow. Esa es la guarda de recursión de GitHub funcionando. Los eventos creados con el `GITHUB_TOKEN` del workflow no generan ejecuciones nuevas del workflow, precisamente para que un workflow que hace commit no pueda dispararse a sí mismo para siempre por accidente. Hay una escotilla de escape documentada (usa un PAT o un token de GitHub App cuando de verdad quieras ejecuciones aguas abajo), y tu estación no quiere nada de eso. Una feature, no un arreglo.

Las aristas 3 y 4 son la historia del panel, y vale la pena ver que el panel ahora es un resumen de una sola página de todo el curso:

![Tres carriles llevan datos de la flota, lecturas en vivo de la blockchain y estado de workers hacia un solo panel, cada carril anotado con su propio delay de frescura.](assets/v03-flowchart.webp)

La arista nueva, la número 4, es la única construcción de este capstone, y la razón de que exista se dice en voz alta: no requiere ninguna habilidad nueva. Es el patrón de polling de m03-l2 apuntado al endpoint de m07-l1. Tu panel lleva consultando una URL JSON pública en un intervalo desde el módulo 3; tu worker de TS lleva sirviendo su snapshot de KV como JSON público desde el módulo 7. Apunta el primero al segundo y al panel le crece un panel de estado de workers. Ninguna API nueva, ninguna plataforma nueva, ningún paquete nuevo. Esa es la tesis del curso en un solo panel: en algún punto la capacidad nueva deja de venir de herramientas nuevas y empieza a venir de componer las que ya son tuyas.

Ahora la sección de honestidad, porque un capstone que esconde sus costuras es una demo, y nombrarlas es lo que hace que esto sea un sistema. Tres costuras, todas deliberadas.

El poller es solo local. El tier gratis te compró tres superficies públicas (Vercel, dos URLs de workers.dev), no cuatro. Exponer el poller significaría tunneling o hosting pago, los dos fuera de alcance a propósito; al contenedor se le golpea desde tu propia máquina y ese es el diseño, no un atajo. Segundo, el camino de flota del panel tolera la desactualización por diseño: un cron de 30 minutos más un cache de CDN de 5 minutos quiere decir que el panel de flota puede ir detrás de la realidad por más de media hora, cosa que sabes desde que m03-l2 te enseñó a leer `cache-control: max-age=300` en devtools. Tercero, y el más grande: el hub entero confía en un solo pipeline. Si Actions se cae, o si salta la desactivación automática por 60 días de inactividad en tu repo público, el latido se detiene. Cada mitigación que tienes para eso vino de m09: la notificación de falla es tu alarma de último recurso, y la política de desactivación es la razón de que el runbook que escribes hoy lleve adentro un ejercicio para volver a habilitarlo. Un solo pipeline es un punto único de falla de verdad y la estación lo carga con los ojos abiertos, porque la alternativa en un tier gratis es un segundo scheduler que también tendrías que monitorear.

Una cosa que esta lección deliberadamente no tiene: un cuadro de Profundiza. No hay capítulo canónico de libro para "ensambla el sistema que ya construiste." La fila de lectura adicional del runbook simplemente apunta de vuelta al mapa de enseñado-contra-bookmark de m01-l1, y la lección siguiente, la conclusión, reimprime ese mapa con ojos frescos.

## Lab: armar, verificar, documentar

La estructura, para que te dosifiques: el paso 1 construye el harness, los pasos 2 al 9 recorren las ocho aristas, guiados por checklist, y escribes un check de script de demo por arista a medida que la verificas. El paso 10 es la ejecución doble. El paso 11 es el README. Presupuesta el grueso de tu tiempo para la arista 4 (el paso 5, el panel nuevo) y el README (el paso 11); todo lo demás es verificación de cosas que ya funcionan.

1. **El harness.** Crea `scripts/demo.sh` en el repo de la estación. Te doy el harness y un check trabajado; cada otro check es tuyo para escribirlo, y esa es la tarea, no un hueco. El contrato: cada check imprime una línea `OK` o `FAIL`, y el script sale distinto de cero si algo falló.

```bash
#!/usr/bin/env bash
set -u
PASS=0; FAIL=0

# --- edit these four lines to your station ---
REPO="YOUR_USER/pulse-station"
DASHBOARD_URL="https://your-board.vercel.app"
WORKER_TS_URL="https://pulse-edge-ts.your-subdomain.workers.dev"
WORKER_RS_URL="https://pulse-edge-rs.your-subdomain.workers.dev"

check () {
  local name="$1"; shift
  if "$@" >/dev/null; then
    echo "OK   $name"; PASS=$((PASS+1))
  else
    echo "FAIL $name"; FAIL=$((FAIL+1))
  fi
}

# checks get authored here, one per edge, as you verify

echo
echo "$PASS OK, $FAIL FAIL"
[ "$FAIL" -eq 0 ]
```

Una sola decisión de redirección en `check` es estructural: el stdout de cada comando se tira, porque las herramientas charlatanas enterrarían el conteo, pero el stderr se deja deliberadamente en paz. Esa es la propiedad tres vestida de bash. Cuando un check falla, su razón, la línea `console.error` de la arista 2, los mensajes `-S` de curl, se imprime justo encima del veredicto FAIL en vez de desvanecerse en `/dev/null`; agrégale `2>&1` a esa redirección y cada check que escribas hoy se queda mudo exactamente en el momento en que te debe una explicación.

![Una función auxiliar chica de bash anotada para mostrar que cada check es una etiqueta más cualquier comando cuyo código de salida decide el veredicto impreso.](assets/v04-annotated-code.webp)

2. **Arista 1: el latido.** Abre la pestaña Actions y lee las últimas ejecuciones, en plural, porque ninguna ejecución sola muestra jamás los seis jobs juntos: las condiciones `if:` son mutuamente excluyentes por diseño. Las barreras más la sonda viajan en los pushes a main y en el calendario; `release` se dispara solo en v-tags, donde la sonda se saltea; `images` viaja en los pushes. Así que una ejecución programada que muestra release e images como salteados es una ejecución sana, no una rota, y el censo que estás levantando es "cada job en verde en el trigger al que pertenece," leído a lo largo de la historia reciente. Después scriptéalo. Tu repo es público, así que la REST API de GitHub le responde a un `curl` pelado sin autenticar en `https://api.github.com/repos/$REPO/actions/runs?per_page=1&status=completed`; el campo `conclusion` de la última ejecución completada debería decir `success`, y `node -e` con un `fetch` es tu herramienta de JSON-sobre-HTTP desde el módulo 1. Escribe el check. Mientras la pestaña Actions está abierta, encuentra el commit del propio cron en la historia de ejecuciones y confirma lo que no está: ninguna ejecución disparada por él. Estás mirando la guarda de recursión de `GITHUB_TOKEN` portándose bien, y tu check acaba de codificar el estado sano.

3. **Arista 2: un status.json fresco.** Este lo trabajo completo, porque su trampa es la que produce scripts de demo en falso verde: revisar HTTP 200 en un archivo cacheado por CDN demuestra que el CDN tiene bytes, no que tu cron está vivo. El check tiene que leer el timestamp propio del payload. El umbral de 60 minutos es la regla de incidentes de m03-l2: una ejecución de cron perdida es un hipo, dos son un incidente.

```bash
STATUS_URL="https://raw.githubusercontent.com/$REPO/main/status.json"

status_fresh () {
  node -e '
    fetch(process.argv[1]).then(r => r.json()).then(j => {
      const age = (Date.now() - Date.parse(j.generatedAt)) / 60000;
      if (!(age < 60)) throw new Error("stale: " + age.toFixed(1) + " min old");
    }).catch(e => { console.error(e.message); process.exit(1); });
  ' "$STATUS_URL"
}
check "edge 2: status.json younger than 60 min" status_fresh
```

Corre el script ahora. Dos líneas OK y un conteo limpio, y el patrón del harness queda demostrado. Todo de acá en adelante eres tú.

4. **Arista 3: los dos primeros paneles del panel.** Abre tu URL de Vercel en un navegador. El panel de flota renderiza filas reales coloreadas por el clasificador de pulse-core; el panel de Solana muestra lecturas con kit en vivo. Mira el panel de Solana un segundo más de lo que necesitas, porque calladamente es el mejor medidor de la estación: la blockchain que tu estación vigila cambió su latido a mitad del curso. La etapa 2 de SIMD-0525 llevó mainnet a slots de 300ms en el epoch 1024 el 2026-08-28, un cuarto menos de intervalo y por lo tanto un tercio más de slots por segundo, la distinción exacta que ejercitó m08-l1, días antes de que se congelara la investigación de este curso, y la sonda del 2026-09-01 midió 316ms contra esa meta de 300ms. Tu panel está vigilando un latido que cambió mientras aprendías a medirlo, y muestra lo que hay, no lo que promete la spec. Esa es la disciplina de todo este curso en un par de números. Check de script: `curl -fsS` sobre la URL del panel, y nómbralo con honestidad, algo como `edge 3: dashboard deploy answers (transport only)`, porque según la taxonomía de esta misma lección este es del tipo vanidoso: demuestra que el deploy sirve bytes, no que los paneles renderizan. Los paneles son un hecho del navegador y el screenshot de cierre es su evidencia, así que el script carga este único check de transporte conscientemente etiquetado como la excepción aceptada en vez de una contradicción callada de la sección de gusto.

![Dos barras horizontales comparan una meta de tiempo de slot de 300 milisegundos con un promedio medido de 316 milisegundos, un hueco de alrededor de cinco por ciento.](assets/v05-chart.webp)

5. **Arista 4: el panel de estado de workers.** La única construcción del capstone. Semiguiada, como se prometió: acá está la composición, y el código es tuyo.

   - **El patrón:** m03-l2, completo. Un schema de zod en la frontera, una unión discriminada `BoardState`, `useState` más `useEffect`, un poll con `setInterval` y su función de limpieza, parse-don't-validate al llegar.
   - **El target:** el endpoint JSON público de tu worker de TS, la ruta raíz de `pulse-edge-ts.<your-subdomain>.workers.dev`, sirviendo una entrada por target con el veredicto de `solana-rpc` incluido, exactamente como lo entregó m07-l1.
   - **La pared de la lección uno, ahora desde el lado del servidor:** este panel es la primera lectura del curso deliberadamente cross-origin de una superficie que es tuya, así que recoge el tratamiento que m07-l1 aplazó al capstone. El origen de tu panel es su URL de vercel.app; el worker responde desde workers.dev; y la regla de m01-l1 gobierna sin cambios: una página siempre puede leer su propio origen, y una respuesta cross-origin solo se puede leer cuando el servidor da el opt-in. El opt-in es el header `access-control-allow-origin: *` que m07-l1 congeló adentro de la respuesta JSON del worker, la configuración honesta para un snapshot público de solo lectura, y para este panel es toda la superficie de CORS: el poll es un GET pelado sin headers propios, que la spec clasifica como simple request, así que nunca se dispara un `OPTIONS` de preflight y ese único header hace todo el trabajo. Demuestra el opt-in desde afuera antes de escribir una línea de React: `curl -s -D - -o /dev/null "$WORKER_TS_URL" | grep -i access-control-allow-origin` imprime el header o te detienes acá. Y sé preciso sobre quién es dueño de la pared: borra ese header del worker y la misma URL le sigue respondiendo a curl mientras el panel se muere con un error de CORS en la consola del navegador, porque la pared es del navegador, nunca de la red. El panel de flota nunca necesitó este opt-in de tu parte solo porque raw.githubusercontent.com manda el mismo `*` sin condiciones, el header que leíste en devtools en m03-l2; esta vez el servidor diciendo que sí es tuyo.
   - **Las ediciones:** un segundo archivo de schema para la forma del payload del worker, un tercer efecto de fetch-y-poll, y un componente de panel que reutiliza los colores del clasificador del panel. `VERDICT_COLOR` en `StatusRow.tsx` hoy es una const local al módulo, así que ponle `export` adelante primero, y después impórtala.
   - **El presupuesto:** consulta en el mismo intervalo de 60 segundos que el panel ya usa para status.json. El snapshot de KV del worker solo cambia con el latido del propio cron del worker, así que consultar más caliente no te compra nada, y esta vez el presupuesto que quemarías no es el de un CDN, es tu propio tier gratis de Workers, los 100k requests por día que dimensionaste en m07-l1 (el medidor de REQUESTS; la cifra de 200,000 por día de m09-l2 es el medidor aparte de eventos de Workers Logs, y los dos nunca comparten presupuesto). Un poll de 60 segundos desde una pestaña o tres vive cómodamente adentro de ese presupuesto para siempre. Un loop caliente no.
   - **Aceptación:** tercer panel en vivo en el panel desplegado, los targets del worker visibles con sus estados, y un check de script de demo que hace fetch a la URL del worker y falla si el payload no incluye el target `solana-rpc`. Presencia, no veredicto, a propósito: si el egress de tu worker está en la blocklist del RPC público, esa fila muestra un `down` honesto con un 403, el cambio a endpoint de fallback de m07-l1 es el arreglo, y un panel que reporta un rechazo verdadero es un monitor funcionando, no un check para ablandar.

6. **Arista 5: dos workers, independientemente.** Hazle `curl` a las dos URLs de workers.dev, y lee cada una por lo que de verdad es, porque los dos tienen trabajos deliberadamente distintos; m07-l2 lo dijo cuando se negó a darle un cron al worker de Rust, con el argumento de que el mismo trabajo dos veces enseña copiar y pegar, no arquitectura. El worker de TS responde con el snapshot de la estación: entradas por target que incluyen el veredicto de getHealth de `solana-rpc`, escritas por su propio cron, desde su propio KV, cada entrada sellada con `checkedAt`. El worker de Rust responde el contrato de clasificación: `GET /` vuelve a pasar por el motor las últimas muestras de fixture guardadas en KV como `[{"name","latency_ms","verdict"}]`, y su estado cambia solo cuando algo le hace POST de muestras frescas. El punto de esta arista es lo que no contiene: ninguno de los dos workers consume al poller, al panel, ni al otro. Y la independencia es verificable, no solo afirmable, porque los dos se mueven en relojes completamente distintos: los valores de `checkedAt` del worker de TS avanzan con el latido de su cron sin que nadie lo toque, mientras que el payload del worker de Rust se queda perfectamente quieto hasta que tú lo alimentas (pruébalo: haz POST del `fixture.json` de la estación con una latencia cambiada, mira cómo su GET da vuelta los veredictos mientras los timestamps del worker de TS te ignoran por completo). Dos superficies haciendo de proxy sobre una sola fuente de datos no podrían comportarse así. Si quieres el ejercicio completo, la versión del runbook va más allá: baja un worker (despliega una ruta rota a propósito, o simplemente imagínatelo durante una semana más calmada) y confirma que las otras tres superficies públicas no parpadearon. Escribe un check por worker, cada uno contra su propio contrato: el check de TS sobre la entrada `solana-rpc` del snapshot, el check de RS sobre que el JSON de veredicto por target responde (haz POST de `fixture.json` y grepea buscando un veredicto, o afirma que el GET devuelve un array). Que el worker de Rust se gane un check en el mismo script, con cero código compartido entre los dos en tiempo de ejecución, es la recompensa de m07-l2 sentada a la vista.

7. **Arista 6: el poller, desde el registro, en tu máquina.** La jugada de m06-l4, ahora como operador:

```bash
docker run --rm -p 8080:8080 ghcr.io/<your-username>/pulse-pollerd:latest
```

Después, desde otra terminal, `curl -s localhost:8080/status`. El JSON que vuelve incluye las sondas de la blockchain que cableó m08-l3: lecturas de slot y de balance, tipadas con serde, fallas taxonomizadas con thiserror. Di la frontera en voz alta una vez más, porque tu README la va a declarar: este contenedor no tiene superficie pública, nada en internet puede alcanzarlo, y ni el tunneling ni el hosting para él se enseñan en ninguna parte de este curso. A propósito. Escribe el check contra localhost.

8. **Arista 7: el camino de escritura.** La estación puede mirar. ¿Puede actuar? m08-l4 construyó la respuesta como un contrato que a tu script de demo le prometieron por nombre: `tx-check` imprime una firma confirmada y sale 0, o falla a gritos y sale distinto de cero. Así que el check es una sola línea: `check "edge 7: write path lands" bash -c 'cd tx-check && npx tsx tx-check.ts'`. Sí, eso quiere decir que la ejecución doble del paso 10 aterriza dos transferencias de devnet de verdad con unos minutos de diferencia, y está bien: 0.001 SOL de plata de devnet sin valor por ejecución es exactamente lo que la clave desechable existe para gastar, y un check del camino de escritura que es demasiado precioso para correr dos veces no es un chequeo de salud. Si el faucet está seco hoy, conoces el ejercicio, lo construiste: el fallback del validador local con `RPC_URL` y `RPC_WS_URL` apuntados a 127.0.0.1, ejercitado por todos una vez ya, y el runbook registra los dos modos. Una firma confirmada acá quiere decir que tus claves, tu construcción de mensajes, tu firmado y la maquinaria de inclusión de la red funcionan todos. Los puntos verdes que solo demuestran lecturas son la cosa que tu estación dejó atrás.

9. **Arista 8: auditoría y alarma.** Dos auditorías, una confirmación manual. `pnpm audit --audit-level=high` en la raíz del workspace (m09-l1 corrió el `pnpm audit` pelado; la flag `--audit-level=high` es el apriete del capstone, que le pone una barrera al código de salida según los hallazgos de severidad alta) y `cargo audit` en el workspace de Rust, los dos como checks del script de demo; si tu archivo de veredictos de m09-l1 acepta un advisory específico, codifica esa aceptación en el check en vez de bajarle la vara a la auditoría para que pase, porque un check que se pone en verde haciendo preguntas más fáciles es peor que ningún check. Mecánicamente, por herramienta: `cargo audit` toma `--ignore RUSTSEC-XXXX-NNNN` por cada id aceptado (o una lista `[advisories] ignore` en un `.cargo/audit.toml` commiteado, la escritura durable, y el camino no es decoración: cargo-audit lee la config del proyecto solo desde `.cargo/audit.toml`, y un `audit.toml` en la raíz pelada del proyecto se ignora en silencio, con TOML válido y todo); el paralelo de pnpm de esa escritura durable es `pnpm.auditConfig.ignoreCves` (y `ignoreGhsas`) en package.json: lista ahí los ids de advisories aceptados, hazle commit al lado de tus veredictos, y el código de salida del `pnpm audit --audit-level=high` pelado se vuelve confiable de nuevo. La alarma no se puede scriptear desde afuera, así que se vuelve la única línea manual del runbook: la configuración de notificaciones de GitHub, el canal de Actions, entrega prendida, solo-workflows-fallidos marcado, exactamente donde lo dejó m09-l2. Confirma que sigue prendida y registra la confirmación en el README.

10. **Córrelo dos veces.** `bash scripts/demo.sh && bash scripts/demo.sh`. La vara de aceptación está redactada a propósito: todos los checks en verde en una segunda ejecución consecutiva con cero arreglos manuales entre ejecuciones. Espera que la primera ejecución falle en algún lado; averiguar dónde es todo el trabajo de esa ejecución. Los sospechosos de siempre, en el orden en que suelen aparecer: una URL que todavía carga mi texto de placeholder en el bloque de config, el contenedor del poller que en realidad no está corriendo porque le hiciste `Ctrl-C` hace una hora, un check de frescura escrito contra un nombre de campo que tu flota escribe distinto, y el más taimado, un check que pasó solo porque tu navegador calentó el cache del CDN treinta segundos antes. Arregla cada uno en el script o en la estación, nunca en tu cabeza, y corre de nuevo. Si la ejecución dos se pone en verde sin que la toques, detente y disfrútalo un segundo. Un script de demo que pasa una vez es una anécdota. Dos veces, una atrás de la otra, es un sistema.

![Una línea de tiempo muestra una primera ejecución de demo fallando dos checks, arreglos aterrizando en el repo, y después dos ejecuciones limpias consecutivas produciendo la transcripción guardada.](assets/v06-timeline.webp)

11. **El README/runbook.** El último compás enseñado del curso, y el que vuelve transferible a la estación. Escríbelo para un lector imaginario específico: un dev competente que nunca vio este repo y al que le tocó atenderlo de guardia. Cuatro secciones. El diagrama del sistema, que es tu dibujo del checkpoint del principio de esta lección, corregido y commiteado. Operaciones por superficie: para cada una de las cinco superficies, el comando de correrla, el comando de desplegarla, y el comando que la golpea para demostrar que vive, la mayoría de los cuales puedes levantar directo de tu script de demo. Sé concreto hasta el aburrimiento acá: la fila del panel dice la URL de Vercel y `npm run build` para la prueba de humo local; las filas de los workers dicen `npx wrangler deploy` y su `curl`; la fila del poller dice la línea `docker run` completa con el mapeo de puertos, porque el lector de guardia no se acuerda de tus elecciones de puerto; la fila del pipeline dice dónde vive la pestaña Actions, los seis ids de job, y cuál trigger corre cuál, porque ninguna ejecución sola muestra jamás los seis y un lector que no sepa eso va a cazar una falla fantasma en los jobs salteados de cada ejecución programada. La prueba para esta sección es mecánica: ¿podría alguien operar la estación con tu repo y este archivo, sin ti en la sala? Cada lugar donde la respuesta es "bueno, también necesitarían saber...", ese conocimiento va en el archivo. Ejercicios de incidente: el recorrido de grep de logs de m09-l2, el fallback de faucet seco, y los dos ejercicios de Actions que exige la honestidad del hub, qué hacer cuando llega el email de alarma, y cómo volver a habilitar el workflow cuando salta la desactivación automática de 60 días (el botón Enable workflow de la pestaña Actions, más un commit de keepalive como la contramedida a la que el ecosistema recurre en la práctica). Y la tabla de pins. Más la absorción que m09-l2 prometió en voz alta: funde `SECRETS.md` adentro del README como su sección de secretos, la tabla de cuatro plataformas y la lista de incident queries que empezó tu challenge, o quédatelo como archivo de nivel superior que el README enlaza en su primera pantalla; de cualquiera de las dos formas el lector de guardia encuentra dónde vive cada secreto, y las primeras incident queries, desde un solo punto de entrada.

![Una tabla de cuatro columnas que lista cada herramienta fijada, dónde vive el pin, su valor actual, y el disparador concreto para volver a revisarlo.](assets/v07-table.webp)

Copia la forma, no mis valores: todo el punto, martillado desde que m05-l2 te enseñó a leer los pins de agave, es que la columna de dígitos es lo menos durable de la tabla y la columna de re-chequeo es lo más. El encabezado de la tabla lleva su fecha. Una tabla de pins sin fecha es un rumor. Y la fila de lectura adicional al pie del README apunta a exactamente una cosa: el mapa de enseñado-contra-bookmark de m01-l1, que la lección siguiente reabre.

Después hazle commit al capstone, porque nada en este lab hizo commit solo y la lección siguiente asume que el repo está al día:

```bash
git add -A
git commit -m "capstone: demo script, worker panel, README/runbook"
git push
```

## Challenge

La extensión en solitario. El repliegue de la ayuda se completa acá: sin apoyo, sin composición nombrada, sin interfaz dada. Elige exactamente una:

(a) Un tipo nuevo de target de sonda, de punta a punta: una variante nueva en la unión de targets de la flota, la lógica del check en un worker, una fila en el panel. Construiste una versión más chica de esto en el challenge de m07-l1; esta cruza la stack completa.

(b) Un panel nuevo en el panel sobre datos que la estación ya produce. La estación emite más de lo que muestra: la historia de sondeos está sentada en el git log como un status.json por ejecución de cron, el /status del poller lleva lecturas de la blockchain que ninguna superficie pública muestra, los workers guardan timestamps por target en KV. Un panel de historia construido a partir de un puñado de commits recientes es la entrada fuerte clásica acá, y fíjate en la frontera antes de elegir la opción del poller: el panel no puede alcanzar tu localhost, así que sacar datos del poller a la superficie quiere decir que el pipeline los lleva, no una arista nueva hacia adentro de tu casa.

(c) Una alerta: algún camino por el cual un estado malo se vuelve un estado ruidoso. La forma dentro de los límites que vale la pena robar: un worker escribe una flag de degradado adentro de su snapshot de KV, y un step de Actions lee el endpoint público y hace fallar la ejecución cuando la flag está puesta, lo que dispara la alarma de m09-l2 que acabas de confirmar.

![Tres tarjetas comparan un tipo nuevo de sonda, un panel nuevo y una alerta, arriba de un solo banner compartido que declara que solo se permiten habilidades enseñadas.](assets/v08-comparison.webp)

La única regla es la tarea misma: solo habilidades enseñadas. Un SDK de bot de Telegram es una buena idea y una dependencia no enseñada, así que falla. Desplegar el poller a un cluster de Kubernetes quedó señalizado fuera de alcance en la barrera de tiers del M6 y ahí se queda; la orquestación es problema de otro curso. La regla no es modestia, es la prueba: este curso gastó diez módulos reemplazando el reflejo de agarrar una herramienta nueva por la habilidad de seleccionar entre las que ya son tuyas. Demuestra que prendió.

Aceptación, las cinco: la extensión es visible en una superficie desplegada; aparece en el README, diagrama incluido si agregó una arista; el script de demo sigue pasando dos veces consecutivas con cero arreglos manuales; `git grep` buscando cualquier cosa con forma de secreto en cada repo sigue volviendo limpio; y la extensión está commiteada y pusheada con el resto del capstone.

## Checkpoint

Lo que ahora puedes hacer, concretamente, y vale la pena leer esta lista despacio porque es el estado terminal del curso: dibujar un sistema distribuido políglota de memoria y saber cuáles flechas no existen; verificar un sistema de ocho aristas de punta a punta contra un script de demo que escribiste tú; componer dos patrones enseñados en una arista de producción nueva sin un tutorial; operar todo el asunto desde un README con una tabla de pins fechada; y extender un sistema vivo en solitario, adentro de los límites de tu propia stack. Hace diez módulos instalaste Node.

El par de evidencias de cierre, como se prometió arriba: la transcripción de tu script de demo, todos los checks en OK en la segunda ejecución consecutiva, y un screenshot del panel mostrando los tres paneles en vivo. La recuperación de 30 segundos antes de que cierres la pestaña: ¿cuáles tres flechas de tu diagrama están deliberadamente ausentes? (Nada hacia adentro del poller, los workers no consumen nada, el poller no alimenta ninguna superficie pública.) ¿Y por qué el commit del propio cron no vuelve a disparar el pipeline? (La guarda de GITHUB_TOKEN: los eventos escritos por workflows no generan ejecuciones, y tu estación cuenta con eso.)

Un pedido mientras el sudor está fresco. Esta lección apostó todo a la forma de checkpoint-después-verificar, nada de ejemplos trabajados, confiando en que dos módulos de repliegue te llevaran. Dime dónde se sostuvo y dónde te dejó caer, y nombra la única arista cuyo check fue el más difícil de escribir. Si una arista se come consistentemente una hora del tiempo de ensamblaje de todo el mundo, ese es exactamente el feedback que le cambia la forma a este capstone.

La estación está entera y el script de demo lo demuestra, dos veces. Queda una lección, y no cablea nada: ninguna herramienta nueva, ningún código nuevo. Un mapa de exactamente dónde estás parado ahora, leído contra los cursos que vienen después, cuáles bookmarks del módulo 1 recién se volvieron urgentes, y cuál puerta del catálogo desbloquea primero tu estación. Construiste el sistema. Ahora leemos el mapa que te deja en la mano.
