# Lánzalo: deploy, devops-engineer, tech-docs-writer

Stamp tiene un programa, tests, un reporte de auditoría, una página web, y desde la lección pasada una propuesta de token, un diseño de swap y un plan de juego. Esta lección lo pone en devnet con la parada de aprobación del kit, le da un pipeline, escribe los docs que un desconocido necesita, y deja que un revisor conozca la app en frío. Unos quince minutos, y el deploy necesita la CLI de Solana de la lección 3.

## Por qué importa

Lanzar es donde aparecen el dinero y las llaves, así que es donde más importa el firewall del kit. **El firewall es un conjunto de reglas de permisos y sandbox generadas desde un solo tier**, nombrado en `.claude/security.json`, más guard hooks que corren antes de un comando, más el plugin safe-ai-skill. El tier por defecto, **Relaxed** (relajado), deja trabajar al toolchain y **detiene deploys de programas, upgrades, cambios de autoridad y transferencias para tu aprobación**, nombrando el cluster, salvo en un validador local. Cuatro acciones irreversibles se rechazan de plano en todos los tiers, un deploy final, `program close --bypass-warning`, `program-v4 finalize` y desactivar una autoridad de token, y el agente te entrega el comando para que lo corras tú. Dos agentes trabajan en este carril: **devops-engineer** para CI, fijado del toolchain, infraestructura RPC y monitoreo, y **tech-docs-writer** para la documentación que toma cada dato del código y no de la memoria.

## Manos a la obra

1. **Haz el deploy a devnet.** Escribe dentro de Claude Code:

```text
/deploy devnet
```

Los pasos del propio comando: confirmar el RPC y el deployer con `solana config get`, correr `anchor keys sync` para un programa nunca desplegado de modo que `declare_id` coincida con el keypair, fondear el deployer con un airdrop o el faucet web, y después desplegar **con el cluster en el comando**, para que el prompt de aprobación muestre qué cluster resolvió. Ese prompt es el firewall. Di sí, y el comando publica el IDL en Program Metadata, revisa el programa con `solana program show`, y registra el id en `.program-id-devnet` y en el env de tu app. *Mainnet es un paso aparte*: el comando lo arranca solo cuando lo pides, y primero reporta lo que falta, una auditoría sin hallazgos Critical o High abiertos, tests y fuzzing pasando, una corrida en devnet de los flujos reales, un plan de upgrade authority, y una auditoría externa para programas que guardan fondos de usuarios.

2. **Mira en qué tier estás.** Escribe:

```text
/firewall
```

Reporta el tier declarado y el aplicado, y la cantidad de reglas. **Off** (apagado) apaga el sandbox para una máquina que ya aislaste, **Medium** (medio) es para leer código que no escribiste, **High** (alto) niega de plano las escrituras en mainnet y es el único tier que impide que el agente edite la configuración del propio kit, y un cambio aplica en la próxima sesión. Pídele a Claude que lea `.env` una vez para ver al plugin de seguridad rechazarlo. Ese rechazo es del plugin, así que si el instalador se saltó safe-ai-skill, Relaxed deja `.env` legible y la lectura pasa. La forma del kit de saber qué llaves están configuradas es `bash .claude/bin/env-keys.sh`, que imprime nombres y `set` o `empty`, nunca un valor.

![El carril de lanzamiento: /deploy devnet se detiene en el prompt del firewall que nombra el cluster, devops-engineer fija el toolchain y escribe CI con llaves solo de devnet, tech-docs-writer declara program id, autoridad, seeds, layouts y errores desde el código, y /product-review conoce la app como su próximo usuario.](assets/v01-ship-lane.webp)

3. **Dale un pipeline.** Escribe:

```text
Usa el agente devops-engineer. Configura CI para Stamp con /setup-ci-cd: build, tests y los lints de auditoría en cada pull request, un build verificable para release, y un deploy a devnet en los tags. Dime dónde viven las llaves y qué versiones fijaste.
```

Su archivo carga los datos de CI en Solana en los que un pipeline genérico se equivoca: **fijar el toolchain** (Anchor 1.1.x con Solana CLI 3.1.10, Rust 1.89 o más nuevo en el host, GLIBC 2.39 o más nueva), condicionar los merges a build más tests, construir el artefacto de release con `solana-verify build` para que el programa on-chain se pueda verificar, instalar Surfpool porque `anchor test` corre sobre él, y **en CI solo van llaves de devnet**. La upgrade authority de mainnet se queda en una billetera de hardware o en un multisig de Squads, y CI puede escribir el buffer y entregárselo al multisig, que ejecuta el upgrade. Cierra con `/audit-infra`, la auditoría de infraestructura, y con `/doctor` cuando el toolchain cambió, y reporta pipelines, nombres de secretos sin valores, versiones fijadas y la lista de alertas.

4. **Escribe los docs que un desconocido necesita.** Escribe:

```text
Usa el agente tech-docs-writer para escribir el README de Stamp para desarrolladores de frontend e indexer, desde el código, el IDL y los tests.
```

Su archivo dice qué deben declarar los docs de un programa: el **program id por cluster y la upgrade authority**, revisados con `solana program show`, las cuentas de cada instrucción con sus flags de signer y writable, **las seeds de los PDA como bytes exactos** y si el bump se guarda, los layouts de cuenta con offsets porque los indexers filtran sobre ellos, las compute units por instrucción, y **los errores por nombre, decimal y hex**, para que `0x1770` se pueda encontrar. Toma los ejemplos de los tests o del cliente generado y los corre antes de publicar.

5. **Deja que un desconocido conozca la app.** Escribe:

```text
/product-review
```

Pide el producto, el usuario objetivo, la única acción central y la etapa, después recorre la app como ese usuario y puntúa ocho dimensiones con evidencia: onboarding, loop central, estados vacíos, estados de error, rendimiento, señales de confianza, mobile, docs. Cada puntaje viene con la línea que lo ganó, y el roadmap se agrupa por esfuerzo. `--harsh` lo convierte en una prueba de estrés con diez dimensiones ponderadas y exactamente tres arreglos. *Para Stamp el punto a mirar es si el valor se ve antes de conectar la billetera*, y para una tarjeta de fidelidad la respuesta debería ser sí.

6. **Haz commit y describe el cambio.** `/quick-commit` formatea, pasa el lint y hace commit con un mensaje convencional y sin trailer de atribución, proponiendo primero una rama con fecha cuando estás en `main`. `/pr-description` escribe el cuerpo de un pull request que un revisor puede verificar, desde el diff, el log y la salida de tests que realmente corrió, *y deja afuera cualquier número que nadie corrió*.

## Está listo cuando

- `.program-id-devnet` guarda el id de Stamp y `solana program show` sobre él te nombra como upgrade authority.
- Existe un workflow de CI con llaves solo de devnet y versiones fijadas.
- El README declara el program id, las seeds como bytes y los códigos de error.
- Tienes un scorecard con una línea de evidencia por dimensión.

## Ojo con

- **Para mainnet, el reporte de precondiciones viene antes del prompt de aprobación.** El prompt es el segundo chequeo.
- **Un deploy fallido deja SOL en un buffer.** El comando sabe cómo retomarlo o recuperarlo. Nunca imprimas el keypair ni la frase semilla por los que pregunta.
- **Las llaves de RPC públicas viajan en el bundle.** El archivo de devops las enruta por un Worker o una ruta de servidor.
- **El firewall detiene el error caro y pregunta.** Si Stamp debería estar en mainnet o no sigue siendo tu decisión.

## Lo que queda

Los deploys **se detienen para tu aprobación y nombran el cluster**, mainnet arranca con un reporte de precondiciones, CI guarda solo llaves de devnet, y los docs declaran el id, la autoridad, las seeds y los códigos de error desde el código. *El firewall está en el deploy porque ahí es donde está el dinero.*

## Siguiente

La próxima lección es tuya: tres hilos de ideas armados como cadenas de agentes, desde el primer prompt de investigación hasta la salida de un fin de semana, y una plantilla para escribir el tuyo. Trae una idea, aunque sea mala.
