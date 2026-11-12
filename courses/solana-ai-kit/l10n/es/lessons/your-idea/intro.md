# Tu propia idea: tres hilos para adaptar

Stamp fue de tres líneas a un deploy en devnet pasando por trece de los quince agentes. Esta lección muestra **la misma ruta sobre otras tres ideas**, cada una como una cadena de agentes con su primer prompt, y termina con una plantilla para la tuya. Lectura y un texto para escribir. Unos diez minutos.

## Por qué importa

El orden que siguió Stamp es la forma del kit para cualquier idea: investigar, diseñar, planear, construir, probar, auditar, las partes que la gente toca, deploy, docs, revisión. Lo que cambia entre ideas es **qué agentes tienen turno y qué pack piden**. Cuando puedas escribir esa cadena para una idea en diez líneas, puedes empezar a construir a la mañana siguiente, y los agentes te van a decir cuando elegiste el equivocado, porque cada archivo trae una sección de Handoffs.

## El método, en un bloque

```text
1. idea.md, tres líneas: una persona, lo que no puede hacer hoy, lo que hace tu cosa
2. solana-researcher: qué existe, citado, fechado, etiquetado
3. solana-architect (game-architect para un juego): cuentas, instrucciones, invariantes, autoridad, riesgos → design.md
4. /plan-feature: fases, riesgos, tests, preguntas abiertas → respóndelas en design.md
5. anchor-engineer: construye desde design.md, cierra con /build-program, /test-rust, /audit-solana
6. solana-qa-engineer: umbral de cobertura, tests negativos, pase de slop
7. las partes que la gente toca: solana-frontend-engineer, mobile-engineer, rust-backend-engineer
8. los especialistas que una idea necesita: token-engineer, defi-engineer, unity-engineer
9. /deploy devnet, con la parada de aprobación
10. devops-engineer para CI, tech-docs-writer para el README, /product-review para una lectura en frío
```

## Hilo uno: Vaquinha, una colecta entre amigos para un regalo

Amigos juntan SOL para un regalo. El fondo le paga al organizador cuando se alcanza la meta, o le devuelve el dinero a todos en una fecha límite.

- **solana-researcher**: qué existe para fondos grupales y escrow en Solana, y cómo maneja cada uno el camino del reembolso. Espera etiquetas Low en la mitad.
- **solana-architect**: un PDA de fondo por regalo que guarda meta, fecha límite y organizador, un PDA de contribución por contribuyente, tres instrucciones, contribute, release, refund. La lista de riesgos empieza con *quién puede llamar a release y cuándo*, y el chequeo que yo miraría es la **fecha límite**: léela del clock sysvar dentro del programa, nunca de un argumento, porque un argumento es lo que quien llama diga que es.
- **anchor-engineer, después solana-qa-engineer**: los tests de borde son la fecha límite menos un segundo y la meta menos un lamport.
- **solana-frontend-engineer**: una página por fondo con una barra de progreso y un botón de contribuir, simular primero, confirmada y no finalizada.
- **defi-engineer, como variante**: contribuciones en cualquier token, convertidas a USDC con un swap de Jupiter con el output mínimo exigido on-chain.
- Lo que tienes al final: un fondo en devnet y una página que un amigo puede abrir.

Primer prompt para pegar:

```text
Usa el agente solana-researcher. ¿Qué existe en Solana para fondos de pago grupales o escrow de crowdfunding, cómo maneja cada uno los reembolsos después de una fecha límite, y cuáles cerraron? Cita, fecha y etiqueta cada hallazgo.
```

## Hilo dos: Biblioteca de herramientas, depósitos en un galpón del barrio

Un galpón presta taladros y escaleras. Quien pide prestado bloquea un depósito, y lo recupera cuando devuelve la herramienta.

- **solana-researcher**: patrones de depósito y devolución, y quién guarda el depósito mientras la herramienta está afuera.
- **solana-architect**: un PDA de ítem por herramienta, un PDA de préstamo por préstamo activo, instrucciones para prestar, devolver y reclamar un depósito después de un período de gracia. La pregunta que te va a hacer es *quién confirma la devolución*, y el diseño honesto hace que el voluntario del galpón la firme.
- **token-engineer, como variante**: una membresía que no se puede vender, un mint Token-2022 con la extensión non-transferable, revisada antes de permitir un préstamo.
- **mobile-engineer**: el voluntario escanea un código en la puerta del galpón, así que la app que importa está en un teléfono, Android primero porque ahí es donde corre Mobile Wallet Adapter.
- **rust-backend-engineer**: un indexador pequeño para que el galpón vea qué herramientas están afuera y desde cuándo, idempotente ante reenvíos.
- Lo que tienes al final: prestar y devolver funcionando en devnet, y una lista de miembros.

Primer prompt para pegar:

```text
Usa el agente solana-architect. Diseña un programa de préstamo de herramientas: ítems, préstamos con un depósito bloqueado, devolución confirmada por un voluntario, y reclamo del depósito después de un período de gracia. Cada cuenta con dueño, seeds, tamaño y quién paga la renta, cada instrucción con sus signers, los invariantes, el modelo de autoridad, los riesgos abiertos. Anchor salvo que se mida lo contrario. Escribe design.md.
```

## Hilo tres: Lista de la puerta, entradas que no se pueden revender

Un evento pequeño vende una cantidad fija de entradas que no se pueden revender, solo devolver al organizador, que las vende otra vez a precio nominal.

- **solana-researcher**: ticketing en Solana, y qué proyectos intentaron limitar la reventa y qué les pasó.
- **token-engineer**: esta idea es su trabajo. Una entrada es un token, y la regla vive en un **programa de transfer hook** que corre en cada transferencia y ve origen, mint, destino y owner, así que puede permitir una transferencia de vuelta al organizador y rechazar todas las demás, y el ejemplo de hook del propio kit es una allowlist. La decisión se toma antes de que exista el mint porque las extensiones quedan fijas en la creación. Su archivo advierte que muchos venues rechazan los mints con hook, *lo que para una entrada que no debería operarse es una ventaja*.
- **token-engineer, otra vez**: el programa del hook también está en su carril, con el discriminador de la interfaz de hook que su archivo nombra, y anchor-engineer construye el resto del programa alrededor.
- **solana-frontend-engineer**: la página de la puerta, que escanea una billetera y muestra una entrada válida, con la conexión de billetera postergada hasta que el valor se vea.
- **devops-engineer**: el único hilo donde una fecha de mainnet es real, así que aplican el plan de upgrade authority y las reglas de llaves de CI de la lección 6.
- Lo que tienes al final: un mint con hook en devnet y una página de puerta que dice sí o no.

Primer prompt para pegar:

```text
Usa el agente token-engineer. Entradas para un evento de 200 personas como un mint Token-2022 que los tenedores pueden devolver al organizador y no transferir a ningún otro lado. Propón el conjunto de extensiones, la regla del transfer hook, quién tiene cada autoridad, y qué le muestra una billetera al tenedor. Las mínimas extensiones que hagan el trabajo, y di qué venues van a rechazar el mint.
```

![Tres hilos de ideas como cadenas de agentes: Vaquinha pasa por researcher, architect, anchor, QA, frontend y una variante DeFi, Biblioteca de herramientas por researcher, architect, token, mobile y backend, Lista de la puerta por researcher, token-engineer con un transfer hook, anchor, frontend y devops, y todos terminan en /deploy devnet.](assets/v01-three-threads.webp)

## Escribe el tuyo

Copia el bloque y llénalo. Mantén honestas las tres líneas de la idea, y pon un nombre en cada paso, porque *un agente pedido por nombre hace el trabajo que su archivo describe, y un pedido sin nombre va a donde caiga la coincidencia de descripción*.

```text
<nombre>
<una persona>, <lo que no puede hacer hoy>, <lo que hace tu cosa al respecto>

1. solana-researcher: <la única pregunta que necesitas responder primero>
2. <arquitecto>: <las dos o tres cuentas que esperas, en palabras>
3. anchor-engineer, solana-qa-engineer: <el caso de borde que un test tiene que tocar>
4. <el agente de cara a la gente>: <la única pantalla que tiene que existir>
5. <el especialista, si hay>: <el pack que va a pedir>
6. /deploy devnet, después tech-docs-writer y /product-review
```

Cuando un backlog es real y tienes un día para mirarlo, el kit también encadena los pasos por ti: `/intake` lee issues en una lista de trabajo, `/research`, `/decide` y `/plan-work` cierran el plan, `/implement`, `/review-round` y `/fix-wave` lo construyen y lo revisan, y **`/crew` corre esos siete y se detiene en tus puntos de control**, siempre en el plan y siempre antes de lanzar. `/ship` es el único paso de esa cadena que `/crew` nunca corre por ti. Yo empezaría con la cadena de arriba a mano, y usaría `/crew` en el segundo proyecto.

## Está listo cuando

- Tu hilo tiene tres líneas de idea, un agente con nombre por paso y un primer prompt pegado.
- Puedes decir qué agente es dueño de una entrada sin reventa, un reembolso grupal y una devolución confirmada por un voluntario.

## Ojo con

- **Un juego empieza con game-architect**, no con solana-architect. La descripción del arquitecto de sistemas lo dice y hace el handoff.
- **La mayoría de los especialistas pide un pack.** Presupuesta la instalación en la primera hora.
- **Las fechas límite vienen del clock sysvar.** Un diseño que recibe la hora como argumento se la confía a quien llama, y esa es una regla mía, no una línea del kit.

## Lo que queda

Una ruta, diez pasos, **un agente con nombre en cada uno**, y lo único que cambia entre ideas es qué especialistas tienen turno. *Escribe la cadena antes de escribir el prompt.*

## Siguiente

La última lección trata de seguir siendo bueno en esto después del taller: el agente guía que enseña a tu nivel, el comando que explica código que no escribiste, y cómo el kit se actualiza solo. Termina con un quiz sobre todo el curso.
