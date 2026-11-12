# Construye alrededor: ingenieros de frontend, mobile y backend

Stamp tiene un programa con tres instrucciones y una suite de pruebas. Nadie en la caja puede usarlo todavía. En esta lección tres agentes construyen las partes que la gente toca: **una página web para la tarjeta del cliente, una app de teléfono y un servicio que le da a la tienda un dashboard**. La parte web corre solo con Node, así que todos siguen al teclado. Unos veinte minutos.

## Por qué importa

El programa es una parte pequeña de un producto, y el kit lo sabe: seis de los quince agentes trabajan *alrededor* del programa. Tres de ellos están en esta lección. **solana-frontend-engineer** construye React y Next.js sobre `@solana/kit`, con conexión de billetera a través de Wallet Standard y una lista fija de reglas de UX para transacciones. **mobile-engineer** construye apps en React Native y Expo sobre Mobile Wallet Adapter. **rust-backend-engineer** construye los servicios que leen la cadena y mantienen una base de datos honesta ante reintentos y forks. El archivo de cada uno trae *los errores que los hábitos viejos hacen mal*, y esas listas son el valor real, porque son los bugs que de otro modo pondrías en producción.

## Manos a la obra

1. **Haz el scaffold de la app web.** Escribe dentro de Claude Code:

```text
/scaffold stamp-web, solo frontend, con una plantilla de Kit que no tenga programa
```

El comando corre `create-solana-dapp` con una plantilla de Kit, convierte la carpeta en un repositorio git e instala el kit adentro con el mismo instalador que corriste en la lección 1. El plugin de seguridad pregunta ante esa instalación por pipe, así que espera una pregunta. Abre Claude Code ahí cuando termine.

2. **Pide la página de la tarjeta del cliente.** Escribe:

```text
Usa el agente solana-frontend-engineer. Construye la página del cliente para Stamp: conectar una billetera, mostrar la tarjeta del cliente para una tienda con el conteo de sellos y cuántos sellos faltan para un café gratis, y un botón de redeem. Lee la tarjeta desde el PDA de tarjeta del programa Stamp en devnet. Hasta que el programa esté desplegado, lee desde un archivo fixture para que la página funcione hoy. Sigue tus reglas de UX para transacciones en el flujo de redeem y corre /build-app y /test-ts antes de devolverme el trabajo.
```

Espera que lea primero la referencia de frontend del pack solana-dev, y después construya sobre **`@solana/kit` 8 con los plugins de billetera y RPC y `@solana/react`**, dejando afuera los paquetes legacy de wallet-adapter. El flujo de redeem debería seguir las reglas de su archivo: **simular antes de pedir una firma**, mostrar esperando firma, después enviada con la firma y un enlace al explorador, después **confirmada**, y liberar la UI en confirmada en vez de finalizada. Una firma rechazada es un resultado normal, no un error. Cierra corriendo `/build-app` y `/test-ts` y reportando componentes cambiados, billeteras probadas y brechas de accesibilidad que quedaron abiertas, porque su vara es **WCAG 2.2 AA**.

3. **Genera el cliente en vez de escribir las llamadas a mano.** Cuando el programa exista, escribe:

```text
/generate-idl-client
```

Construye el IDL y genera un cliente TypeScript tipado con Codama, para que la página llame a `redeem` a través de una función con tipos reales, y los errores del programa se decodifiquen en mensajes. *Las instrucciones armadas a mano son lo que este comando existe para reemplazar.*

![El carril alrededor del programa: solana-frontend-engineer construye la página web sobre Kit con las reglas de UX para transacciones, mobile-engineer construye la app de Expo sobre Mobile Wallet Adapter, y rust-backend-engineer construye el indexador por webhooks y la API del dashboard, todos leyendo el mismo cliente generado y design.md.](assets/v01-app-lane.webp)

4. **Describe la app de teléfono, constrúyela después.** Escribe:

```text
Usa el agente mobile-engineer. Planea la app de cliente de Stamp en Expo: conectar a través de Mobile Wallet Adapter, mostrar la tarjeta, hacer redeem. Dime qué pack de skills necesitas, qué funciona en iOS y qué necesita un development build. Solo el plan por ahora.
```

Su respuesta va a pedir el pack `solana-mobile` y decirte tres cosas de su propio archivo que ahorran una tarde: **Mobile Wallet Adapter es solo Android**, así que iOS firma a través del protocolo de deep link o el SDK de una billetera. **Necesita un development build**, porque Expo Go no puede cargarlo. Y el polyfill de Web Crypto tiene que cargarse primero o la firma falla antes de que la billetera se abra, de una manera que parece un bug de la billetera.

5. **Dale a la tienda un dashboard.** Escribe:

```text
Usa el agente rust-backend-engineer. Diseña un servicio pequeño en Axum para Stamp: consumir webhooks de Helius para los eventos stamp y redeem del programa, mantener una tabla de Postgres de sellos por cliente por tienda, y servirle a la tienda un endpoint de dashboard. Dime cómo mantienes la tabla correcta cuando un webhook se vuelve a entregar. Solo el diseño.
```

La respuesta debería contener las reglas que su archivo llama la parte difícil: **las entregas se repiten**, así que haz upsert por la firma más la posición del evento en la transacción. **Las transacciones fallidas también están on-chain**, así que revisa el campo de error antes de aplicar efectos. **Confirmada está bien para mostrar y finalizada para cualquier cosa irreversible.** Guarda el slot en cada fila y haz el backfill por el mismo camino idempotente. Va a pedir el pack `helius` para los webhooks, y la clave que las llamadas de ese pack necesitan vive en `.env`, que `/setup-mcp` configura.

## Está listo cuando

- `stamp-web` construye y muestra una tarjeta desde el fixture con una billetera conectada en devnet.
- Puedes repetir las tres etapas de transacción que el botón de redeem debe mostrar.
- Tienes un plan escrito para la app móvil y el indexador, cada uno nombrando el pack o la clave que necesita.

## Ojo con

- **Los valores de entorno públicos viajan en el bundle.** `NEXT_PUBLIC_*` y `EXPO_PUBLIC_*` son visibles para cada usuario, así que una URL de RPC con clave va detrás de una ruta de servidor.
- **Un blockhash dura unos 60 a 90 segundos.** Retransmitir la misma transacción firmada es seguro hasta entonces. Después de eso, reconstruye y pide una firma nueva.
- **Transacción v1.** El cliente planea v1 por defecto, y una billetera que no la admite necesita un cliente v0 o un aviso de actualización. El archivo de frontend tiene el chequeo.
- **El cliente RPC bloqueante traba un runtime de Tokio.** El archivo de backend nombra el no bloqueante. Un ejemplo más viejo no.

## Lo que queda

Tres ingenieros construyen las partes que la gente toca, cada uno con **una lista de las trampas en su propio archivo**: confirmada y no finalizada en la web, adaptador de billetera solo Android en mobile, upserts idempotentes en el indexador. *El cliente generado es lo que mantiene a los tres hablando con el mismo programa.*

## Siguiente

En la próxima lección los sellos de Stamp se vuelven un token, un cliente paga con cualquier moneda, y la tarjeta se convierte en un juego, que son el ingeniero de tokens, el ingeniero de DeFi y los dos agentes de juegos. Mayormente lectura y prompts, sin toolchain.
