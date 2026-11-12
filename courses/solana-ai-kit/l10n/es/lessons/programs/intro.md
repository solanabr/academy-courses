# Construye el programa: anchor-engineer, pinocchio-engineer, solana-qa-engineer

La lección pasada terminó con `design.md`: una cuenta de tienda, una tarjeta por cliente, tres instrucciones. En esta lección un ingeniero las construye, un ingeniero de QA las prueba y una auditoría lee el resultado. **Rust, Anchor y la Solana CLI entran aquí.** Si no están en tu máquina, sigue en la pantalla e instálalos esta noche. El comando de la Solana CLI está en tu reporte de `/doctor`. El toolchain de Anchor aparece ahí cuando existe un workspace de Anchor, así que para esta noche es `rustup` para Rust, y después `cargo install --git https://github.com/solana-foundation/anchor avm --force && avm install latest && avm use latest` para Anchor. Unos veinte minutos.

## Por qué importa

El código del programa es donde un tutorial copiado más duele, porque la API de Anchor cambió en 1.0 y la mayor parte de la web todavía muestra la forma vieja. El kit tiene **dos implementadores y un tester**. **anchor-engineer** es el predeterminado: Anchor 1.x, constraints en vez de chequeos escritos a mano, IDL generado, pruebas en Rust con LiteSVM. **pinocchio-engineer** existe para los casos medidos, *una instrucción que sigue por encima de su presupuesto de cómputo después de las propias palancas de Anchor, comisiones de cómputo a volumen, o un límite de tamaño del binario*, y escribe cada chequeo a mano. **solana-qa-engineer** es dueño de cómo se prueba el programa: qué harness para qué prueba, la umbral de cobertura, las líneas base de cómputo, y quitar el ruido que un modelo deja en un diff. Tres comandos los unen: `/build-program`, `/test-rust`, `/audit-solana`.

## Manos a la obra

1. **Entrégale el diseño al ingeniero.** Escribe dentro de Claude Code:

```text
Usa el agente anchor-engineer. Prepara un workspace de Anchor 1.x para Stamp con la plantilla de pruebas LiteSVM e implementa las tres instrucciones de design.md: create_shop, stamp y redeem. Una cuenta de tarjeta por cliente como está diseñado, aritmética con chequeo, bumps guardados, y una prueba en Rust por instrucción. Corre /build-program y /test-rust antes de devolverme el trabajo.
```

Fíjate en el archivo que lee primero: la referencia de Anchor dentro del pack solana-dev. Después, las formas en las que su propio archivo insiste, las que **los hábitos viejos hacen mal**: el espacio de la cuenta a partir de `DISCRIMINATOR.len() + INIT_SPACE`, `CpiContext::new` recibiendo una clave de programa en vez de un account info, PDAs guardados validados con `bump = card.bump`, códigos de error que empiezan en 6000. Las reglas de la casa en `CLAUDE.md` suman aritmética con chequeo y ningún `unwrap` fuera de las pruebas. *Su regla de cierre es build, test y una auditoría para el código que mueve valor*, y reporta las instrucciones y cuentas que cambió junto con los resultados de las pruebas.

2. **Lee una falla de prueba a la manera del kit.** Si una prueba falla, el error llega en hexadecimal. El archivo de `/test-rust` lo decodifica: **2000 a 2999** son violaciones de constraints, **3000 a 3999** errores de cuenta, **4100** significa que el programa se cargó en una dirección distinta a su `declare_id`, y **6000 en adelante** son tu propio enum de errores. Pide los logs del programa antes de que alguien cambie código:

```text
Muéstrame los logs del programa de la prueba de redeem que falla y dime si está mal la prueba o el programa, antes de cambiar nada.
```

![El carril de programas: design.md va a anchor-engineer para instrucciones y pruebas LiteSVM, /build-program y /test-rust corren antes de devolver el trabajo, solana-qa-engineer agrega pruebas negativas y quita el slop, /audit-solana escribe un reporte de hallazgos, y pinocchio-engineer es la rama que se toma ante un problema de cómputo medido.](assets/v01-program-lane.webp)

3. **Trae al tester.** Escribe:

```text
Usa el agente solana-qa-engineer. Revisa las pruebas de Stamp contra tu umbral de cobertura: cada instrucción, cada código de error alcanzado por una prueba negativa, y los bordes, incluyendo un redeem con nueve sellos y un stamp sobre una tarjeta en el conteo máximo. Agrega lo que falte, corre /test-rust, y quita cualquier slop de IA del diff.
```

Su archivo le da una **tabla de harnesses**: Mollusk para una instrucción y cómputo exacto, LiteSVM para flujos de varias instrucciones, Surfpool para estado real de mainnet, Trident para fuzzing antes de mainnet. Para Stamp se va a quedar en LiteSVM. La parte que la gente no espera es el **pase de slop**: comentarios que repiten el código, chequeos defensivos alrededor de llamadas internas confiables, logging de depuración olvidado, strings de error verbosos. Los quita y te dice qué quitó en una o dos oraciones.

4. **Audítalo.** Escribe:

```text
/audit-solana
```

El comando delimita cada instrucción, corre `cargo audit` y clippy con los lints de aritmética y unwrap, recorre las checklists del auditor-skill y escribe `docs/security-audit-<date>.md`: conteos por severidad, cada hallazgo con `file:line`, el escenario de explotación y el arreglo, una lista separada de sospechas sin confirmar, y un veredicto. Para Stamp espera que mire con atención **quién puede llamar a stamp**, porque una clave de tienda que firma desde un teléfono es el riesgo que el arquitecto ya nombró, y la aritmética del redeem.

5. **Ten claro cuándo existe el otro ingeniero.** Stamp nunca va a necesitar Pinocchio, y ese es el punto de conocer la regla: el archivo del arquitecto dice *optimiza primero una instrucción caliente en su lugar y pórtala solo si sigue por encima del presupuesto*. Cuando llegue ese día, `/profile-cu` mide el cómputo por instrucción, `/benchmark` lo compara contra una línea base, y pinocchio-engineer cierra con ambos. Su archivo es también la lista honesta de lo que cuesta la validación manual: discriminadores de un byte, alineación zero-copy, sin IDL generado.

## Está listo cuando

- `/build-program` reporta el programa construido, su tamaño y su id, y el IDL escrito.
- `/test-rust` pasa con una prueba negativa por código de error.
- `docs/security-audit-<date>.md` existe y leíste su línea de veredicto.

## Ojo con

- **Reconstruye antes de probar.** LiteSVM incrusta el binario del programa en tiempo de compilación, así que `cargo test` solo prueba lo que se construyó la última vez. El comando de pruebas lo hace por ti.
- **Dos enums de error chocan.** Los dos empiezan en 6000 salvo que uno fije un offset, y el choque es silencioso. El archivo del ingeniero lo sabe. Un tutorial copiado no.
- **Una auditoría no es una auditoría externa.** `/audit-solana` recomienda una en su propia salida para programas que guardan fondos de usuarios.
- **Sin toolchain hoy está bien.** Lee la transcripción en la pantalla y corre los comandos de instalación después. La lección 4 es solo Node otra vez.

## Lo que queda

Un ingeniero construye desde el diseño con la API actual de Anchor, un tester sostiene la umbral de cobertura y quita el slop, y un comando escribe un reporte de auditoría, **con build, test y auditoría como la rutina fija de cierre**. *Pinocchio es una decisión medida, nunca un valor por defecto.*

## Siguiente

En la próxima lección el ingeniero de frontend pone la tarjeta de un cliente en una página web con una billetera conectada en devnet, que corre solo con Node. Los ingenieros de mobile y backend tienen su turno en la misma lección.
