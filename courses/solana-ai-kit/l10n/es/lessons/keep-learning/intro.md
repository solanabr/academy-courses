# Seguir aprendiendo: solana-guide, /explain-code y mantenerse al día

En la lección pasada escribiste tu propia cadena de agentes. Catorce de los quince ya fueron presentados, y el decimoquinto es el de las semanas después del taller, cuando estás solo con un programa que entiendes a medias. Esta lección es **el agente guía, el comando que explica código, el researcher como hábito, y cómo el kit se actualiza solo**, y después un quiz sobre todo el curso. Unos diez minutos.

## Por qué importa

El kit va a construir más rápido de lo que aprendes, y esa brecha es la que yo miraría. **solana-guide** existe para la brecha. Su archivo dice que enseña para que *quien aprende pueda resolver el próximo problema sin él*: primero revisa tu nivel y pregunta en vez de suponer, da una definición simple y el problema que el concepto resuelve, después un ejemplo mínimo, después el error que cometen los principiantes, y comprueba que puedas predecir antes de seguir. Enseña **el tooling actual** y te lo señala cuando el tutorial que trajiste es anterior. El código de producción va a los ingenieros, y esa división es deliberada.

## Manos a la obra

1. **Pregúntale al guía algo que te saltaste.** Escribe dentro de Claude Code:

```text
Usa el agente solana-guide. Construí Stamp con el kit y no entiendo bien por qué la cuenta de la tarjeta es un PDA en vez de una cuenta normal. Sé JavaScript y un poco de Rust.
```

Espera que confirme tu nivel, defina un PDA en palabras simples con el problema que resuelve, dibuje un diagrama Mermaid de tienda, tarjeta y cliente, muestre la derivación, nombre el error del principiante, y después te pida predecir algo, *qué pasa si alguien pasa una cuenta de tarjeta distinta*, antes de seguir. Si se salta la pregunta, pídesela. A lo largo de varias sesiones puede correr un track del currículo de la incubadora en el pack solana-new, concepto, código funcionando, ejercicio, revisión línea por línea, hasta que construyas sin ayuda y te pase a los ingenieros.

2. **Explica código que no escribiste.** Escribe:

```text
/explain-code programs/stamp/src/lib.rs
```

El comando ajusta su profundidad al código, y un programa entero recibe la forma completa: un resumen en dos o tres oraciones, un **diagrama Mermaid** del flujo de instrucciones o de las relaciones entre cuentas, un recorrido por las secciones importantes con las razones detrás de las decisiones no obvias, los conceptos de Solana usados, **notas de seguridad** sobre los chequeos que el código hace y los que parece que le faltan, y para quien aprende dos o tres preguntas de *qué pasa si*. Córrelo sobre la salida del anchor-engineer de la lección 3 y vas a entender tu propio programa mejor que cuando se escribió.

![El carril de aprendizaje: solana-guide revisa tu nivel, define, dibuja, muestra el error y te pide predecir, /explain-code da un resumen, un diagrama, un recorrido y notas de seguridad, solana-researcher mantiene al día versiones y estado, y /update, memsearch y /dream mantienen al día el kit y su memoria.](assets/v01-learning-lane.webp)

3. **Haz del researcher un hábito.** Antes de cualquier decisión que dependa de una versión, un estado de deploy o el estado de un protocolo, pregúntale al researcher en vez de a tu memoria o a la de Claude:

```text
Usa el agente solana-researcher. Antes de construir sobre <programa>, revisa su upgrade authority en mainnet, si el binario desplegado es un build verificado, y su historial de auditorías. Cita y fecha todo.
```

Ese es su propio chequeo previo a recomendar, y las tres respuestas son lo que quieres antes de confiarle a un programa el dinero de una tienda.

4. **Mantén el kit al día.** Escribe:

```text
/update
```

Trae el tag de release más nuevo del kit, o `main` mientras no exista un tag más nuevo que v2.4.0, y sobrescribe los agentes, comandos, skills y scripts que el kit trae, incluidos los que editaste. **Conserva `.env`, tus settings, `.mcp.json` y los archivos de memoria**, deja tu `CLAUDE.md` en su lugar y escribe la versión del kit al lado como `CLAUDE.md.upstream` cuando los dos difieren, conserva las extensiones que instalaste, y vuelve a aplicar tu tier de firewall. Reinicia Claude Code después. Los agentes que leíste en este curso son archivos del kit, así que una actualización puede cambiarlos, y `/add-skill` sin id lista cada pack con si está instalado y cuándo instalarlo.

5. **Dale una memoria.** Dos comandos y un reinicio:

```text
/plugin marketplace add zilliztech/memsearch
```

```text
/plugin install memsearch
```

Es un plugin, no un servidor MCP, con un store local y un embedder local, así que sin llave y sin cuenta, al costo de una descarga de modelo de unos 558 MB en el primer arranque. Después de algunas sesiones aparecen archivos markdown diarios bajo `.memsearch/memory/`, y ahí es donde el program id que desplegaste y la decisión de diseño que descartaste dejan de rederivarse. `/dream` consolida un store distinto, `MEMORY.md` y la sección Project Learnings de `CLAUDE.md`, cuando envejecen, y pregunta antes de tocar el archivo de instrucciones.

6. **Sabe dónde viven las respuestas largas.** La carpeta `docs/` del kit nunca se copia a tu proyecto. `agents-and-commands.md` tiene los quince agentes y los 54 comandos, `firewall.md` los tiers y sus límites, `skill-packs.md` los packs fijados, `configuration.md` los servidores MCP y plugins con lo que cuesta cada uno por sesión. Los skill packs oficiales están alojados como markdown en [kit.superteam.codes](https://kit.superteam.codes), donde cualquier agente puede leerlos sin el kit, y los archivos del propio kit se sirven en `aikit.superteam.codes/<path>`.

## Está listo cuando

- El guía te pidió predecir algo antes de seguir.
- `/explain-code` te dio un diagrama y una nota de seguridad sobre tu propio programa.
- memsearch está instalado y Claude Code fue reiniciado.

## Ojo con

- **Reinicia después de memsearch y después de `/update`.** Los plugins se activan al reiniciar, y las reglas de permisos se leen al inicio de la sesión.
- **Los datos de este curso tienen fecha.** Cantidades, versiones y nombres de comandos se leyeron el 2026-11-09 de la rama `main` del kit en el commit `0a2f0d9`, que es lo que el instalador trae hasta que se etiquete un release más nuevo que v2.4.0. El tag v2.4.0 tiene 32 comandos y nada de la cadena de workflow, así que las lecciones sobre `/crew`, `/ship` y `/inspect-mint` describen `main`. Cuando el README del kit y esta lección no coincidan, *el README tiene razón*.
- **El guía enseña, los ingenieros construyen.** Pídele a cada uno lo que su propio archivo promete.

## Lo que queda

Un agente enseña a tu nivel y te hace predecir, un comando explica código con un diagrama y notas de seguridad, y el researcher revisa autoridad, build verificado y auditorías antes de que confíes en un programa. *El kit se actualiza solo y conserva tus archivos, y el plugin de memoria evita que rederive lo que ya decidiste.*

El quiz de abajo cubre todo el curso. Después, construye algo pequeño con el kit esta semana, y cuéntale a la gente que organizó tu taller qué agente te sorprendió y cuál te decepcionó.
