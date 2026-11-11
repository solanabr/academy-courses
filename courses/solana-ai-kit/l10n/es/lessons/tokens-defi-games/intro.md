# Tokens, DeFi y juegos: cuatro especialistas más

En la lección pasada tres ingenieros construyeron la página de la tarjeta, planearon la app del teléfono y diseñaron el indexer de la tienda. Esta lección hace tres preguntas que una tienda real haría a continuación: **¿los sellos pueden ser un token en la billetera del cliente, el cliente puede pagar con cualquier moneda, y la tarjeta puede ser un juego?** Cada pregunta tiene un agente, y el archivo de cada agente carga la regla que detiene el error obvio. Prompts y lectura, sin toolchain. Unos quince minutos.

## Por qué importa

Estos cuatro agentes cubren las partes del ecosistema que cambian a menudo, y donde la memoria de un modelo suele estar desactualizada: extensiones de token que eran nuevas el año pasado, una API de swap que pasó a una versión nueva, un SDK de juegos con límites de plataforma. **token-engineer** construye mints, extensiones de Token-2022, programas de transfer hook, colecciones NFT y planes de lanzamiento. **defi-engineer** integra Jupiter, Kamino, Marginfi, Raydium, Orca, Meteora y los oráculos, por SDK, API o CPI. **game-architect** y **unity-engineer** diseñan y construyen juegos en Solana. Los cuatro leen skills de protocolo que el kit fija e instala bajo demanda, y el archivo del ingeniero DeFi lo dice sin rodeos: *construye desde el skill del protocolo, no desde la memoria*.

## Manos a la obra

1. **Convierte los sellos en un token.** Escribe dentro de Claude Code:

```text
Usa el agente token-engineer. Los sellos de Stamp deben aparecer en la billetera del cliente como un token que no puede vender. Propón el mint para devnet: programa de token, el conjunto de extensiones, quién tiene cada autoridad y qué va a mostrar una billetera. Las menos extensiones que hagan el trabajo.
```

La frase *las menos extensiones* es la primera regla del propio agente, citada de vuelta. Su archivo explica por qué: **las extensiones del mint quedan fijas en la creación, y cada extensión reduce dónde puede operarse el token**, porque muchos pools, billeteras y agregadores rechazan hooks, delegados permanentes y cuentas congeladas por defecto. Para los sellos, espera un mint Token-2022 con la extensión **non-transferable** y la **extensión de metadata**, la tienda como autoridad del mint, y una nota de que un café gratis es un burn. Espera también la trampa: la associated token account se deriva del program id de Token-2022, y el id clásico da la dirección equivocada.

Si alguna vez lanzas un token que la gente pueda operar, el mismo agente trae los **chequeos previos al lanzamiento**: asignaciones documentadas, tokens del equipo con vesting on-chain, cada autoridad restante en un multisig de Squads o revocada, y *revocar es permanente, así que te pregunta primero*. `/inspect-mint` decodifica las extensiones y las autoridades vivas de cualquier mint desde los bytes crudos, que es como revisas el token de otro antes de integrarlo.

2. **Deja que el cliente pague con cualquier token.** Escribe:

```text
Usa el agente defi-engineer. Un cliente quiere pagar el café con el token que tenga, y la tienda quiere USDC. Diseña el paso de swap con Jupiter: qué API, cómo se protege la tienda de una mala tasa, y cómo lo probarías. Solo diseño.
```

Su archivo nombra la API actual, **swap v2 con API key**, y tres chequeos que una integración rápida se salta: **exigir un output mínimo on-chain**, y después recargar la cuenta de destino tras el swap y revisar el delta del saldo en vez de la cotización. **Precio desde un oráculo** con un feed id y una antigüedad máxima, nunca desde las reservas spot de un pool que un flash loan puede mover. Y **probar en un fork de mainnet con Surfpool**, para que carguen las cuentas reales del protocolo. Va a pedir el pack `jupiter`, y `sendai` si aparecen lending u oráculos.

![El carril de token, DeFi y juego: token-engineer construye el mint de sellos con las menos extensiones, defi-engineer agrega un swap de Jupiter con output mínimo on-chain y precio de oráculo probado en un fork de Surfpool, y game-architect escribe concept.md y plan.md para unity-engineer con pagos verificados on-chain.](assets/v01-token-defi-game-lane.webp)

3. **Convierte la tarjeta en un juego.** Escribe:

```text
Usa el agente game-architect. Stamp quiere un juego pequeño: después de cada café el cliente juega una ronda de diez segundos en la app de la tienda y un buen puntaje gana un sello extra. Decide qué vive on-chain y qué no, elige la plataforma, y escribe concept.md y plan.md. Pregúntame primero lo que necesites saber.
```

El archivo del agente lo hace preguntar por la jugabilidad, la plataforma y qué debe estar on-chain antes de escribir nada, y después le da la división: **on-chain lo que es valioso, intercambiable o confiado por otros jugadores**, off-chain la jugabilidad por frame y el estado de sesión, escrituras en checkpoints en vez de por acción. La oración a buscar en su respuesta es esta: **los puntajes reportados por el cliente están bajo control del atacante**, así que el sello extra se paga solo contra un chequeo que el cliente no puede falsificar. Escribe `concept.md` con números en vez de adjetivos y `plan.md` con pasos numerados, y espera tu aprobación. **unity-engineer** construye después desde esos dos archivos en pasos pequeños, editar, compilar, probar, y corre `/build-unity` y `/test-dotnet` antes de devolver el trabajo, y su archivo carga las trampas de Unity: objetos solo en el hilo principal, nada de hilos de C# en WebGL, texturas que nunca se recolectan.

4. **Elige tu variante.** El token, el swap y el juego son tres versiones de Stamp que podrías construir a continuación. Escribe la que construirías bajo `design.md`, con el agente que es su dueño.

## Está listo cuando

- Tienes un conjunto de extensiones propuesto para el token de sellos con un dueño por cada autoridad.
- Puedes decir cuáles tres chequeos protegen a la tienda en el diseño del swap.
- `concept.md` y `plan.md` existen para el minijuego, con la división on-chain escrita.

## Ojo con

- **Cada uno de estos agentes pide un pack.** `metaplex`, `jupiter`, `sendai`, `solana-game`. El hub imprime el comando de instalación, `/add-skill <id>` lo ejecuta, y `/update` lo mantiene.
- **Las acciones que mueven valor se detienen.** Los archivos de token y DeFi dicen ambos que el plugin de seguridad controla las acciones de mainnet, de movimiento de valor y de autoridad, y que un ask o un deny es tu política, reportada en vez de reintentada por otro camino.
- **Las comisiones de transferencia reducen lo que recibe el destinatario.** Acredita el delta del saldo, no el argumento de monto. El archivo de token lo dice, y el archivo DeFi dice lo mismo sobre el output de un swap: revisa el delta del saldo, nunca la cotización.
- **Mobile y PSG1 solo cuando lo pides.** El arquitecto de juegos arranca por defecto con desktop primero y WebGL segundo.

## Lo que queda

Cuatro agentes para cuatro partes del ecosistema que se mueven rápido, y cada archivo carga **la única regla que detiene el error obvio**: las menos extensiones, output mínimo on-chain, oráculo y no spot, verificar lo que paga. *Construyen desde skills de protocolo fijados y piden el pack que necesitan.*

## Siguiente

En la próxima lección Stamp se lanza: un deploy en devnet con la parada de aprobación, un pipeline de CI, docs que declaran el program id y cada layout de cuenta, y una revisión de producto por alguien que conoce la app por primera vez. El firewall aparece donde importa, en el deploy.
