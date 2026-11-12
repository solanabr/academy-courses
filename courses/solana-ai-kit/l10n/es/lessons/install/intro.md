# Instala el kit y conoce al equipo

Al final del taller habrás entregado un proyecto de Solana a **los especialistas del kit**, uno a la vez, desde la primera pregunta de investigación hasta el deploy. Esta lección instala el kit que los trae y te muestra cómo está hecho uno de ellos. Unos quince minutos.

## Por qué importa

El Solana AI Kit es una carpeta de configuración para Claude Code, mantenida por Superteam Brasil en [github.com/solanabr/ai-kit](https://github.com/solanabr/ai-kit). Leído el 2026-11-09 desde la rama `main` del kit en el commit `0a2f0d9`, trae 15 agentes, 54 slash commands, un hub de skills, cuatro servidores MCP y un firewall. El instalador descarga `main` hasta que se etiquete una release más nueva que v2.4.0, y el tag v2.4.0 en sí trae 32 comandos y nada de la cadena de workflow de la lección 7, *así que espera que los conteos de este curso se muevan*. Este curso trata de los agentes. **Un agente es un prompt especialista**: un archivo markdown con un nombre, una descripción de una línea, un modelo, las referencias que debe leer antes de trabajar, los errores que no debe cometer y una tabla de a quién le pasa el trabajo. Claude Code lo arranca como subagente cuando tu pedido coincide con su descripción, o cuando lo nombras. *Nombrarlo es el hábito que enseña este curso.*

## Manos a la obra

1. **Arranca Claude Code en la carpeta donde guardas tus proyectos.** No en tu directorio home, y no dentro de otro repositorio, porque el siguiente paso crea la carpeta del proyecto ahí.

```bash
claude
```

2. **Pega este prompt.** Crea la carpeta del proyecto, lee el instalador, lo corre desde dentro de esa carpeta y revisa lo que quedó. Aprueba cada comando cuando aparezca, y lee el resumen del instalador antes de decir sí a ejecutarlo.

```text
Prepara un proyecto de taller del Solana AI Kit para mí. Muéstrame cada comando antes de ejecutarlo, y detente si un paso falla.
1. Crea una carpeta llamada kit-workshop dentro del directorio actual e inicializa un repositorio git vacío adentro.
2. Descarga https://aikit.superteam.codes/install.sh a un archivo temporal, léelo, y dime en pocas líneas qué escribe, qué agrega a .gitignore y qué plugin instala.
3. Ejecuta ese archivo con bash desde dentro de kit-workshop, para que el kit se instale en esa carpeta y en ninguna otra, y muéstrame el cuadro de cierre que imprime.
4. Lista kit-workshop/.claude/agents y dime cuántos archivos tiene.
5. No arranques Claude Code tú mismo. Dime que salga de esta sesión, haga cd a kit-workshop y corra claude ahí.
```

El cuadro de cierre dice *Installation complete*, nombra el nivel del firewall (**Relaxed**, relajado) y lista los próximos pasos. Lo que quedó: `.claude/`, `CLAUDE.md`, `.mcp.json`, `.env` y `.env.schema` en la carpeta, con `.claude/`, `.mcp.json`, `.gitmodules` y `.safe-ai-skill/` agregados a `.gitignore` y `CLAUDE.md` y `.env.schema` dejados bajo seguimiento, más el plugin de seguridad **safe-ai-skill** instalado a través de la CLI `claude`. El instalador no te pregunta nada. Si prefieres escribirlo tú mismo, los mismos cinco pasos son `mkdir` y `git init`, `curl -fsSL` de esa dirección a un archivo, `less` sobre el archivo, y después `bash` sobre él desde dentro de la carpeta.

3. **Reinicia dentro del proyecto y aprueba los servidores.** Sal de la sesión, y después:

```bash
cd kit-workshop && claude
```

Claude Code pregunta una vez si arranca los cuatro servidores MCP de `.mcp.json`. Apruébalos. Después escribe:

```text
/doctor
```

Trece chequeos numerados, cada uno OK, WARN, FAIL o n/a, con un comando de arreglo por cada falla. Claude Code trae un `/doctor` propio, así que si lo que vuelve no son trece chequeos numerados, elige el comando del proyecto en el menú de slash, el que lee `.claude/commands/doctor.md`. Hoy la línea de la Solana CLI puede marcar FAIL y las claves pueden marcar vacías. *Las dos cosas están bien para la próxima lección.* La lección 3 es donde entran Rust, Anchor y la Solana CLI. `/doctor` ya imprimió el comando de instalación de la Solana CLI, e imprime los de Rust y Anchor cuando el proyecto tiene un workspace de Anchor.

4. **Abre un archivo de agente de punta a punta.** Abre `.claude/agents/solana-architect.md` en tu editor, o pídelo dentro de Claude Code:

```text
Muéstrame .claude/agents/solana-architect.md completo, sin cambios.
```

Léelo como una descripción de puesto. El encabezado trae el nombre, la descripción contra la que Claude Code compara, y **`model: opus`**, que todos los agentes del kit fijan sin importar en qué corra tu sesión. Después, las secciones que comparten la mayoría de los quince archivos: **qué leer antes de trabajar** (enlaces al pack solana-dev y a los skills propios del kit), **los detalles que es fácil hacer mal** (aquí: las seeds son permanentes, las cuentas escribibles compartidas serializan cada transacción, los poderes de admin detrás de una multisig), **Handoffs** (qué agente recibe qué seguimiento, una tabla en este archivo y una lista en los otros), y una sección de cierre, **Output** en el arquitecto y en el investigador, **Before handing back** en los ingenieros. Esa sección de cierre es la que hay que retener: *te dice qué se te debe cuando el agente termina*.

5. **Conoce a los otros catorce en cinco carriles.** Se agrupan por la etapa del proyecto a la que sirven:

![Los quince agentes en cinco carriles: diseño con solana-architect, game-architect y solana-researcher, programas con anchor-engineer y pinocchio-engineer, alrededor del programa con solana-frontend-engineer, mobile-engineer, rust-backend-engineer, token-engineer, defi-engineer y unity-engineer, entrega con solana-qa-engineer, devops-engineer y tech-docs-writer, y aprendizaje con solana-guide.](assets/v01-five-lanes.webp)

Las lecciones siguen esos carriles con **un proyecto de ejemplo**: *Stamp*, una tarjeta de fidelidad para una cafetería, donde un cliente gana un sello por compra y diez sellos compran un café. La tienda y el cliente tienen una billetera cada uno. Es una idea pequeña a propósito, para que cada agente tenga su turno en algo que puedas imaginar.

6. **Di el nombre.** Escribe esto dentro de Claude Code y mira la transcripción:

```text
Usa el agente solana-researcher para decirme, en tres oraciones con una fuente citada cada una, qué es Token-2022 y cuándo una tienda pequeña lo querría en vez del programa de tokens clásico.
```

Deberías ver arrancar al agente, y después una respuesta corta con enlaces y una etiqueta de confianza por hallazgo, que es la forma de salida que su archivo exige. La respuesta termina con hasta cinco líneas etiquetadas, **Verified**, **Decisions made**, **Decisions needed**, **Assumptions** y **Next owner**, porque las reglas de la casa se las piden a cada agente y un hook devuelve al agente una vez cuando falta alguna. Vas a ver esas cinco líneas después de cada agente en este curso. Si Claude responde de memoria, sin agente y sin enlaces, di "usa el agente" y pregunta de nuevo.

## Está listo cuando

- `ls .claude/agents` lista quince archivos y `/doctor` corrió.
- Puedes nombrar las secciones que trae un archivo de agente, y la de cierre que dice qué se te debe.
- El investigador arrancó con tu prompt y citó sus fuentes.

## Ojo con

- **El instalador instala en la carpeta desde donde corre.** Por eso el prompt lo corre dentro de kit-workshop. No tiene idea de qué carpeta querías.
- **Opus cuesta Opus.** Cada agente fija Opus, sin importar en qué corra tu sesión. Para una pregunta rápida pregúntale a la sesión principal, y para diseño, código e investigación deja correr al agente.
- **Windows nativo.** El plugin de seguridad no tiene build del motor ahí y el instalador lo salta. WSL2 funciona bien.
- **Node 22.5 o más nuevo.** `/doctor` aprueba Node 18, pero el servidor context-mode entre los cuatro por defecto quiere 22.5 o más nuevo, así que esa es la versión a tener.
- **`.env` guarda claves, `.mcp.json` guarda configuración.** El servidor de Helius lee su clave del entorno, y `/setup-mcp` la agrega por ti. Nada de hoy necesita una clave.

## Lo que queda

El kit se instala en la carpeta desde donde corre el instalador, y sus agentes son **descripciones de puesto en markdown** con un modelo, referencias, trampas, handoffs y una forma de salida obligatoria. *Obtienes el especialista que pides por su nombre.*

## Siguiente

En la próxima lección el investigador y el arquitecto llevan Stamp de tres líneas de idea a un diseño con cuentas nombradas, y tú te quedas con el archivo de diseño, porque todos los agentes que siguen lo leen.
