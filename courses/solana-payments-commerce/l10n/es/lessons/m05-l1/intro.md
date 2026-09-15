# La primitiva del delegado: aprueba una vez, haz el pull según el calendario

El módulo 4 cerró con un back office que verifica pagos del lado del servidor, ingesta webhooks de forma idempotente, concilia por reference key y emite reembolsos como pagos push. Wavelength puede tomar dinero y rendir cuentas de él, de punta a punta. Lo que no puede hacer es tomar dinero OTRA VEZ el mes que viene. Todos los pagos hasta acá empezaron con el cliente haciendo algo: escanear un QR, aprobar una transacción, firmar. Un club del disco del mes necesita lo contrario: el cliente firma una vez en enero y la tienda recibe su pago en febrero, marzo y abril mientras duerme.

Stripe resuelve esto guardando una tarjeta y cobrándola según un calendario. En Solana no hay tarjeta guardada ni custodia: entonces, ¿cómo hace un comercio para hacerle un pull de 15 USDC a un cliente el mes que viene sin tener sus claves ni su dinero? La respuesta es una aprobación de delegado, y viene con una restricción brutal que le da forma a todo el resto de este módulo.

Arranca el workspace ahora para que la instalación corra mientras lees. Esto es un hermano de transfer-kit, dentro de la misma raíz del npm workspace que usas desde el módulo 2:

```bash
mkdir club-crank && cd club-crank
npm init -y
npm pkg set type=module
npm i @solana/kit@6.10.0 @solana-program/token@0.14.0 @solana-program/memo@0.11.2
npm i -D tsx typescript
```

Después salta a la raíz de `wavelength` y agrega la carpeta nueva al roster de workspaces que extendiste en el módulo 4, que es lo que les deja a los scripts de abajo hacer `import { resolveAta } from 'transfer-kit'` por nombre:

```bash
cd ~/wavelength
npm pkg set --json workspaces='["transfer-kit","verifier","backoffice","backoffice-refunds","club-crank"]'
npm pkg set type="module"
npm install
```

Los pins, con su nota de frescura: el workspace se queda en la línea kit v6 sobre la que corre todo el curso, y `@solana-program/token` 0.14.0 es el último release de ese cliente que hace peer con kit v6 (su rango de peers es `^6.5.0`, comprobado en npm 2026-08-22; 0.15.0 saltó a kit `^7` y 0.16.0, que es a lo que resuelve `latest` hoy, ya saltó otra vez a `^8`, así que instalar npm-latest rompe este workspace de plano). La misma historia para el cliente de memo: 0.11.2 es el más nuevo que hace peer con kit `^6.4.0`, y la línea 0.12+ pide kit v7 o más nuevo. La próxima lección convierte este split v6/v7 en un tema propio, porque el cliente oficial de Subscriptions está del otro lado; hoy es una línea de disciplina de pins. `tsx` corre TypeScript directo, como en todo este curso.

Mientras npm trabaja, acá está el mapa en una oración: esta lección es el permiso de gasto crudo de SPL Token, la suscripción no custodial más pequeña posible, y su única limitación dura es exactamente lo que el programa oficial de Subscriptions existe para arreglar la próxima lección.

## Resumen

- `ApproveChecked` pone un delegado en una cuenta de token: una dirección con permiso para mover hasta un monto aprobado de un mint específico, con los decimales declarados para que una suposición equivocada falle ruidosamente. El dueño conserva la propiedad y puede hacer `Revoke` en cualquier momento, unilateralmente, con una sola instrucción.
- Cada cuenta de token tiene exactamente UN slot de delegado activo. Aprobar un delegado nuevo revoca automáticamente al anterior. No hay libro mayor por delegado, no hay estado de pausa, no hay fila. Esta única regla es la restricción de diseño alrededor de la que orbita todo el módulo.
- El monto aprobado es un saldo corriente, no un tope que se reinicia. Cada transferencia firmada por el delegado lo decrementa, y la transferencia que lo deja en exactamente cero también limpia el slot de delegado: una aprobación de 60 USDC cubre cuatro pulls de 15 USDC, y el quinto no encuentra ningún delegado en la cuenta.
- Un "crank" de backend que tiene el keypair del delegado firma `TransferChecked` para hacer el pull de los fondos según el calendario. El suscriptor no firma nada después de la aprobación inicial. El crank paga la comisión base de 5000 lamports por pull; el costo por ciclo del suscriptor es cero firmas y cero comisiones.
- El delegado puede mover solo hasta-el-monto-aprobado del mint aprobado desde esa única cuenta. Ese límite lo impone el programa Token on-chain, no la buena voluntad del crank. Esta es la respuesta honesta a "¿pueden vaciarme la billetera?": no, y no porque lo prometamos.
- Antes de cada pull, vuelve a leer la cuenta. El delegado puede que ya no seas tú, el límite puede estar más abajo de lo que cree tu base de datos, y el estado on-chain es el único libro mayor que importa.

El contrato del scaffold, dicho en voz alta: el camino del pull del crank se entrega como un scaffold trabajado, completo y listo para correr. La guarda que decide si un pull está permitido se entrega como tres TODOs, y llenarlos es el peldaño de completion; las respuestas se derivan, a la vista, en la teoría de abajo. Detectar el desalojo por delegado competidor es el peldaño solo, tuyo y de nadie más.

## Un permiso de gasto, no una tarjeta guardada

### Lo que el dueño firma en realidad

Cuando un cliente de Stripe guarda una tarjeta, entrega una credencial. Quien la tenga decide cuánto cobrar y cada cuánto; los límites viven en la base de datos de Stripe y en el derecho de disputas. Cuando un suscriptor de Wavelength se anota en el club del disco, firma una sola instrucción:

```ts
getApproveCheckedInstruction({
  source: subscriberAta,   // THEIR token account, which they keep owning
  mint: USDC_DEVNET,       // the only token this permission touches
  delegate: CRANK,         // the address allowed to pull
  owner: subscriber,       // the owner signs; nobody else can grant this
  amount: toBaseUnits('60', DECIMALS), // the hard ceiling, in base units
  decimals: DECIMALS,      // stated so a decimals mismatch fails the ix
})
```

Lee eso como una oración: "esta dirección puede mover como máximo 60 USDC de esta única cuenta mía." No "puede administrar mi billetera." No "puede cobrarle a mi cuenta." Puede mover, como máximo, ese monto, de ese mint, desde esa cuenta. El sufijo `Checked` es la misma disciplina que usas desde el módulo 2: la instrucción lleva el mint y los decimales, así que si algo no está de acuerdo sobre lo que quiere decir una unidad base, la transacción falla en vez de mover la magnitud equivocada.

Después de que esto aterrice, la cuenta de token del suscriptor lleva tres hechos que antes no llevaba: una dirección `delegate`, un `delegatedAmount`, y nada más. No hay nombre de plan, no hay cadencia de facturación, no hay metadatos. El programa Token guarda un número y una dirección, y todo lo que una "suscripción" quiere decir más allá de eso es tu problema, off-chain. Guarda ese pensamiento; la factura por eso vence al final de la lección.

![ApproveChecked escribe solo una dirección de delegado y un delegatedAmount en la propia cuenta de token del suscriptor; la propiedad y el saldo quedan intactos, y el poder del crank está acotado por esos dos campos.](assets/v01-diagram.png)

La salida es incluso más chica. `Revoke` toma la cuenta de origen y la firma del dueño, limpia los dos campos, y no necesita el permiso de nadie:

```ts
getRevokeInstruction({ source: subscriberAta, owner: subscriber })
```

Una instrucción, solo la comisión base. Compara eso con cancelar una membresía de gimnasio alguna vez.

### Un solo slot, y la regla de desalojo

Ahora la restricción. Cada cuenta de token tiene exactamente un slot de delegado activo. No uno por comercio, no una lista. Uno. Cuando el dueño firma un `ApproveChecked` nuevo, el programa sobrescribe el slot: delegado nuevo, monto nuevo, y el delegado anterior desapareció. No pausado, no encolado detrás del nuevo. Desapareció, en silencio, sin ninguna notificación al comercio que acaba de perderlo.

Corre la cinta hacia adelante. A tu suscriptor le encanta el club del disco. En marzo también se suscribe a, digamos, un drop de café que corre el mismo diseño de delegado crudo sobre la misma cuenta de USDC. En el momento en que su billetera firma el `ApproveChecked` de la cafetería, la aprobación de tu crank deja de existir. Tu pull de abril falla. Nadie hizo nada mal: el suscriptor consintió a los dos comercios, los dos comercios escribieron código correcto, y la primitiva simplemente no puede sostener dos permisos vivos en una cuenta de token.

![Estados de la cuenta antes y después, mostrando que un suscriptor que aprueba a un segundo comercio sobrescribe el delegado y el límite restante del primer comercio sin ninguna notificación.](assets/v02-comparison.png)

Por eso la lección no deja de decir "la primitiva cruda." Una suscripción viva por (usuario, mint) es un techo de producto real, y ninguna cantidad de código de backend ingenioso lo levanta, porque el techo está en el layout de la cuenta mismo. Lo que el código de backend SÍ puede hacer es detectar el desalojo honestamente en vez de tirar un error a ciegas, y ese es tu challenge solo de hoy. Levantar el techo pide un programa que ocupe el slot una vez y multiplexe acuerdos de facturación reales por detrás, que es precisamente la próxima lección.

### El límite solo baja

La segunda cosa que la intuición entrenada por Stripe entiende mal: el monto aprobado no es un tope mensual y no se reinicia el primero del mes. Es un tanque de combustible, llenado una vez por la firma del dueño, drenado por cada transferencia del delegado, rellenable solo por otra firma del dueño.

El club del disco cobra 15 USDC por ciclo. El suscriptor aprobó 60. Así que:

![Un límite de 60 USDC baja escalonadamente por 45, 30 y 15 a lo largo de cuatro pulls exitosos; el cuarto lo vacía y el programa Token limpia el delegado en la misma instrucción, así que el quinto pull encuentra un slot vacío, y solo una aprobación fresca firmada por el dueño restaura los dos.](assets/v03-chart.png)

Cuatro pulls y el tanque está seco, y el tanque se lleva la llave de paso con él. El programa Token decrementa `delegatedAmount` dentro de la transferencia firmada por el delegado, y cuando esa resta cae en exactamente cero devuelve el `delegate` de la cuenta a ninguno en la misma instrucción, que es el `null` que tu guarda lee el ciclo que viene: agotar una aprobación también la limpia. Así que la quinta transacción no falla por un límite vacío; falla porque la cuenta ya no tiene delegado, lo que convierte la firma del crank en la firma de un desconocido cualquiera, y el programa Token lo dice con `OwnerMismatch`, custom program error `0x4`. `InsufficientFunds`, custom program error `0x1`, es el caso vecino: un límite demasiado chico para este pull pero todavía no en cero, digamos 10 restantes contra un cargo de 15 USDC, donde el slot sigue siendo tuyo. De cualquier manera la cuenta todavía tiene USDC de sobra; lo que se gasta es el permiso para moverlo. No hay nada que el crank pueda hacer al respecto salvo pedirle al suscriptor que firme de nuevo. Esto se lee como una molestia y en realidad es un feature: el suscriptor pre-consintió un total acotado, y el límite está haciendo su trabajo. Una aprobación de 60 USDC son cuatro meses del club, una cadencia de re-consentimiento natural. Podrías pedir 600 por adelantado y hacer pulls por años; algunos productos lo van a hacer, y sus usuarios dados de baja van a descubrir un límite vivo que olvidaron. Dónde pones el techo es una decisión de producto que la blockchain no va a tomar por ti. La blockchain solo hace cumplir el número que el dueño haya firmado.

Una consecuencia que vale la pena costear: después de la aprobación inicial, el costo por ciclo del suscriptor es cero. Ninguna firma, ninguna comisión, nada que recordar. El crank paga la comisión base de 5000 lamports por pull, que a cualquier precio plausible del SOL es un error de redondeo frente a una suscripción de 15 USDC. Compara eso con el 2 a 3 por ciento que una red de tarjetas se lleva de cada renovación, y ves por qué esta forma vale la molestia.

### El crank: la misma transferencia, otro firmante

"Crank" es jerga de Solana que vale la pena adoptar: un proceso más o menos permissionless que le da vuelta a la manivela según el calendario, haciendo el trabajo que la blockchain no va a hacer sola. Solana no tiene cron nativo; nada on-chain se dispara el primero del mes. Algo off-chain tiene que despertarse, decidir que toca un pull, y mandarlo. El nuestro es un script de backend sobre un scheduler.

Acá está la parte que debería sentirse casi anticlimática. El pull es un `TransferChecked`, la instrucción exacta que transfer-kit construye desde el módulo 2, con un campo distinto:

```ts
getTransferCheckedInstruction({
  source: subscriberAta,      // the subscriber's account, as always
  mint: USDC_DEVNET,
  destination: merchantAta,
  authority: crank,           // the DELEGATE signs, not the owner
  amount: toBaseUnits('15', DECIMALS),
  decimals: DECIMALS,
})
```

El campo `authority` siempre quiso decir "quien tenga el derecho de mover estos fondos." Hasta hoy ese era el dueño. El programa Token comprueba: ¿es el firmante el dueño? No. ¿Es el firmante el delegado de la cuenta, y está el monto dentro de `delegatedAmount`? Sí: transferir, después decrementar el límite, atómicamente, en la misma instrucción. No hay un paso de contabilidad separado que olvidar. El decremento ES el efecto secundario de la transferencia.

Como es la misma forma de instrucción, todo lo que enseñaron los módulos 3 y 4 sigue funcionando sin cambios. El crank adjunta una reference key fresca para que el back office pueda conciliar el pull en el libro mayor de pedidos, y un memo para que el cargo se nombre a sí mismo on-chain. Tu receptor de webhooks del módulo 4 va a ver este pull como cualquier otro pago. Los ingresos recurrentes caen en el pipeline que ya construiste, que es la recompensa por construirlo en este orden.

El verdadero trabajo del crank, entonces, no es la transferencia sino el párrafo anterior: decidir si hacer el pull sigue siendo legítimo. Voy a confesar el error para que te lo puedas saltar: el primer crank que cableé cacheó el estado de la aprobación en el alta, porque ¿por qué iba a cambiar? Una billetera de prueba reaprobó a un delegado distinto a mitad de ciclo, mi crank mandó la transacción igual, y me pasé una noche mirando un custom program error 0x4 en un log de transacción antes de caer en lo obvio. El estado de la cuenta es el libro mayor. Tu base de datos es un caché con opiniones. Así que el crank vuelve a leer la cuenta de token todos y cada uno de los ciclos, antes de cada pull, y contesta tres preguntas:

![Tres comprobaciones previas al pull se mapean a resultados: un delegado ausente rechaza como delegate-revoked, ya sea que el dueño lo haya revocado o que un pull que agotó el límite haya limpiado el slot; un delegado ajeno rechaza igual; un límite demasiado chico rechaza como insufficient-allowance; y solo un todo-en-orden sigue adelante.](assets/v04-table.png)

¿Podría el crank saltarse la guarda y mandar la transacción sin más, dejando que la blockchain rechace los pulls malos? Mecánicamente sí, y los fondos estarían exactamente igual de seguros: el programa Token hace cumplir todo lo que la guarda comprueba. La guarda existe porque "transaction failed: custom program error 0x4" y "este suscriptor nos revocó, marca la suscripción como caducada" son hechos distintos para un sistema de facturación, y solo uno de los dos le dice a tu back office qué hacer después. La blockchain te da un no. La guarda te da el motivo, antes de que gastes una comisión en averiguarlo. Esos strings de motivo, `delegate-revoked` y `insufficient-allowance`, son el vocabulario de la primitiva cruda, y las próximas dos lecciones mantienen los dos nombres significativos una capa más abajo: la guarda del programa oficial agrega sus propios motivos encima, y la nota de continuidad en el Challenge de la próxima lección recorre el mapeo explícitamente.

### Lo que el delegado nunca puede hacer

Corre el peor escenario del suscriptor honestamente, porque un cliente va a preguntar, y "confía en nosotros" es una respuesta de Stripe, no una respuesta de Solana.

Supón que Wavelength se vuelve malvada, o más realista, que el keypair del crank se filtra. ¿Qué puede hacer quien lo tenga? Firmar `TransferChecked` contra la cuenta de USDC del suscriptor, hasta el límite restante. Si ya pasaron tres pulls, eso es como máximo 15 USDC. ¿Qué le puede hacer al SOL del suscriptor? Nada; el delegado está sobre una sola cuenta de token. Sus otros saldos de SPL y sus NFTs viven en cuentas completamente distintas, cada una con su propio slot de delegado intacto. ¿Puede aprobarse a sí mismo un límite más grande? No: `ApproveChecked` pide la firma del dueño. ¿Puede impedirle al suscriptor revocar? No: `Revoke` pide solo al dueño. El radio de impacto de un crank completamente comprometido es el límite no gastado sobre exactamente las cuentas que lo aprobaron, y cada uno de esos dueños puede ponerlo en cero unilateralmente en el momento en que se anuncie el compromiso.

![Una clave de crank filtrada alcanza solo el límite restante en la única cuenta de USDC aprobada; el SOL, los otros tokens, los NFTs, la auto-aprobación y el bloqueo de revocación quedan todos fuera de ese perímetro.](assets/v05-diagram.png)

Esa es la promesa no custodial, dicha sin romance: no que el comercio sea honesto, sino que la honestidad del comercio no es estructural. El límite vive en el programa Token, el mismo camino de código auditado que ha liquidado todas las transferencias de SPL que hizo este curso. Hoy no desplegaste un programa, y ese es el punto: no hay ningún contrato nuevo que un suscriptor tenga que auditar. El permiso que otorga lo hace cumplir código en el que ya confía por el solo hecho de tener el token.

El ecosistema notó esta forma. Cuando Superteam corrió su tema de bounty Solana Native "Subscriptions and Allowances" en junio de 2026, docenas de repos de demo convergieron exactamente en esta primitiva de delegado cruda, aprobar-y-después-crank, como la respuesta por defecto para ingresos recurrentes sin custodia. No estás aprendiendo una curiosidad; estás aprendiendo el patrón alrededor del que el ecosistema se está estandarizando, una lección antes de conocer el programa que lo lleva a producción.

## Lab: factura al club del disco del mes

El club: 15 USDC de devnet por ciclo, aprobados en 60, así que el libro mayor cuenta toda la historia en cuatro pulls y un rechazo. Vas a jugar los dos lados, suscriptor y comercio, con dos keypairs.

![El suscriptor firma una aprobación, el crank firma y paga la comisión de cada pull, y el comercio solo recibe 15 USDC por ciclo.](assets/v06-diagram.png)

1. **Keypairs y fondeo.** En el workspace `club-crank`, acuña dos identidades. La instalación del comienzo de la lección ya debería estar lista.

   ```bash
   solana-keygen new --no-bip39-passphrase -o crank.json
   solana-keygen new --no-bip39-passphrase -o subscriber.json
   solana airdrop 1 $(solana-keygen pubkey crank.json) --url devnet
   solana airdrop 1 $(solana-keygen pubkey subscriber.json) --url devnet
   ```

   El suscriptor también necesita USDC de devnet contra el que facturar, y esto merece un párrafo de aritmética en vez de un número, porque el USDC de devnet es el único recurso que este curso no puede conjurar. Mándalo desde tu billetera de comercio con el script `pay` de transfer-kit, el flujo exacto del lab del módulo 2 lección 1 (que también crea la ATA del suscriptor, y te cobra el mínimo exento de rent de 165 bytes una vez — el número que el módulo 2 te hizo sacar con curl en vez de memorizar):

   ```bash
   npm run --workspace transfer-kit pay -- $(solana-keygen pubkey subscriber.json) 60
   ```

   Sesenta no es decoración: el plan de esta lección aprueba 60 USDC y hace pulls de 15 por ciclo, así que 60 son exactamente cuatro ciclos, y que el quinto pull encuentre un slot de delegado vacío es el remate de la lección. Tu billetera de comercio casi con seguridad no tiene 60 USDC de devnet ahora mismo. El faucet de Circle gotea un cupo pequeño por visita y le limita la tasa a las visitas repetidas, así que llegar a 60 quiere decir varias visitas repartidas a lo largo del día. Dos maneras honestas de pasar, elige una antes de seguir:

   - **Gotea y espera.** Visita faucet.circle.com cuando el cooldown te deje, hasta que `spl-token balance 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU --url devnet` pase de 60, y después mándalo. Máxima fidelidad a los números impresos abajo, lo más lento.
   - **Baja la escala del plan.** Divide cada cifra de USDC en esta lección por diez: aprueba `6` en vez de `60`, pon `PLAN = '1.5'` en vez de `'15'`, fondea al suscriptor con 6. La aritmética del límite es idéntica — cuatro pulls lo agotan, el quinto no encuentra delegado — y cada checkpoint de abajo lee la misma historia a un décimo de la escala. Este es el camino que yo tomaría en una billetera de devnet fresca, y el único costo es que las líneas de log imprimen números más chicos que los míos.

   Si los airdrops de SOL te limitan la tasa, espera un minuto y vuelve a intentar; necesitas SOL en los dos keypairs porque el suscriptor paga la comisión de la aprobación y el crank paga la comisión de cada pull.

   Checkpoint: `solana balance $(solana-keygen pubkey subscriber.json) --url devnet` imprime alrededor de 1 SOL, lo mismo para el crank, y el saldo de USDC del suscriptor lee la cifra que hayas elegido arriba. Nada más adelante en el lab funciona sin los tres.

2. **La guarda, como scaffold de completion.** Crea `crank/guard.ts`. Esto es lógica pura, sin red, que es exactamente lo que lo hace testeable offline antes de que se mueva cualquier dinero de devnet:

   ```ts
   // crank/guard.ts: the decision the crank makes before every pull.

   export type PullDecision =
     | { ok: true; pullBase: bigint; remainingAfter: bigint }
     | { ok: false; reason: 'delegate-revoked' | 'insufficient-allowance' };

   export function checkPull(input: {
     /** delegate currently set on the subscriber's token account, or null if none */
     delegate: string | null;
     /** remaining approved amount on the account, in base units */
     delegatedAmount: bigint;
     /** the crank's own address */
     crank: string;
     /** this cycle's pull, in base units */
     pullBase: bigint;
   }): PullDecision {
     // TODO 1: if no delegate is set, or the delegate is not our crank,
     //         refuse with reason 'delegate-revoked'.
     // TODO 2: if pullBase exceeds delegatedAmount, refuse with
     //         reason 'insufficient-allowance'.
     // TODO 3: otherwise return ok with pullBase and the decremented
     //         remainingAfter the pull will leave on-chain.
     throw new Error('TODO: implement the guard');
   }
   ```

   Cada input es un `bigint` en unidades base porque la regla del módulo 2 no venció: los montos son enteros exactos, los floats nunca tocan el dinero. Los tres TODOs son las tres filas de la tabla de decisión de arriba. Checkpoint: nada que correr todavía, y ese es el resultado esperado. El archivo existe, exporta una función, y esa función lanza hasta que la llenes en el Challenge.

3. **La prueba que te juzga.** Crea `crank/guard.test.ts`. Corre la aritmética trabajada de 60/15 y los dos casos de rechazo, completamente offline:

   ```ts
   // crank/guard.test.ts: offline proof the guard behaves before devnet money moves.
   import { checkPull } from './guard';

   const CRANK = 'CrankAddr1111111111111111111111111111111111';
   const OTHER = 'OtherAddr1111111111111111111111111111111111';
   const PULL = 15_000_000n; // 15 USDC at 6 decimals

   let failures = 0;
   function expectCase(name: string, pass: boolean) {
     if (!pass) {
       failures += 1;
       console.error(`FAIL: ${name}`);
     }
   }

   // 1. Fresh 60-USDC approval: four pulls succeed, 60 -> 45 -> 30 -> 15 -> 0.
   let allowance = 60_000_000n;
   for (let cycle = 1; cycle <= 4; cycle += 1) {
     const d = checkPull({ delegate: CRANK, delegatedAmount: allowance, crank: CRANK, pullBase: PULL });
     expectCase(`cycle ${cycle} pulls`, d.ok);
     if (d.ok) allowance = d.remainingAfter;
   }
   expectCase('allowance exhausted after four pulls', allowance === 0n);

   // 2. Two histories, one account state. The fourth pull zeroed the allowance and the
   //    Token program cleared the delegate in the same instruction; an owner running
   //    Revoke empties the same slot by hand. Cycle five reads null either way and is
   //    refused BEFORE submission. The account cannot tell you which happened, so
   //    "cancelled" versus "tank empty, ask for a renewal" is your ledger's call, not
   //    the chain's.
   const emptySlot = checkPull({ delegate: null, delegatedAmount: allowance, crank: CRANK, pullBase: PULL });
   expectCase('empty delegate slot rejected', !emptySlot.ok && emptySlot.reason === 'delegate-revoked');

   // 3. A partial allowance never over-pulls: 10 remaining cannot cover 15. Non-zero
   //    means the delegate is still set, so this is the other refusal reason.
   const partial = checkPull({ delegate: CRANK, delegatedAmount: 10_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('over-pull on partial allowance rejected', !partial.ok && partial.reason === 'insufficient-allowance');

   // 4. Owner approved a competing delegate: the slot holds someone else, and their
   //    60 USDC is not yours to spend.
   const evicted = checkPull({ delegate: OTHER, delegatedAmount: 60_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('evicted crank rejected', !evicted.ok && evicted.reason === 'delegate-revoked');

   // 5. Both refusals are true at once: evicted, and what the new delegate holds would
   //    not have covered this cycle anyway. Identity is checked before amount, so the
   //    reason must be delegate-revoked. Your back office routes on these strings, so
   //    the order the guard tests them in is part of the contract.
   const superseded = checkPull({ delegate: OTHER, delegatedAmount: 5_000_000n, crank: CRANK, pullBase: PULL });
   expectCase('eviction outranks a short allowance', !superseded.ok && superseded.reason === 'delegate-revoked');

   if (failures > 0) {
     console.error(`guard: ${failures} case(s) failed`);
     process.exit(1);
   }
   console.log('guard: all crank cases passed (over-pull and revoked-delegate rejected)');
   ```

   Córrelo: `npx tsx crank/guard.test.ts`. Checkpoint: muere en `TODO: implement the guard`. Correcto. Esa falla es la costura entre el scaffold y tu trabajo de completion; la vas a cerrar en el Challenge, y la barrera de verificación de la lección es que esta prueba imprima su última línea.

4. **Plomería compartida.** Crea `crank/send.ts`, el pipeline de envío de kit del que escribiste variantes desde el módulo 2, más un cargador de keypairs para los archivos generados por la CLI:

   ```ts
   // crank/send.ts: load a CLI keypair, send a list of instructions on devnet.
   import { readFileSync } from 'node:fs';
   import {
     appendTransactionMessageInstructions,
     assertIsTransactionWithBlockhashLifetime,
     createKeyPairSignerFromBytes,
     createSolanaRpc,
     createSolanaRpcSubscriptions,
     createTransactionMessage,
     getSignatureFromTransaction,
     pipe,
     sendAndConfirmTransactionFactory,
     setTransactionMessageFeePayerSigner,
     setTransactionMessageLifetimeUsingBlockhash,
     signTransactionMessageWithSigners,
     type Instruction,
     type KeyPairSigner,
   } from '@solana/kit';

   export const rpc = createSolanaRpc('https://api.devnet.solana.com');
   const rpcSubscriptions = createSolanaRpcSubscriptions('wss://api.devnet.solana.com');

   export async function loadSigner(path: string): Promise<KeyPairSigner> {
     const bytes = new Uint8Array(JSON.parse(readFileSync(path, 'utf8')));
     return createKeyPairSignerFromBytes(bytes);
   }

   export async function sendIxs(
     feePayer: KeyPairSigner,
     ixs: Instruction[],
   ): Promise<string> {
     const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
     const tx = await pipe(
       createTransactionMessage({ version: 0 }),
       (m) => setTransactionMessageFeePayerSigner(feePayer, m),
       (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
       (m) => appendTransactionMessageInstructions(ixs, m),
       (m) => signTransactionMessageWithSigners(m),
     );
     assertIsTransactionWithBlockhashLifetime(tx);
     await sendAndConfirmTransactionFactory({ rpc, rpcSubscriptions })(tx, {
       commitment: 'confirmed',
     });
     return getSignatureFromTransaction(tx);
   }
   ```

   Checkpoint: `npx tsx crank/send.ts` no imprime nada y sale limpio. Este módulo solo exporta; si lanza al importar, una instalación salió mal, así que vuelve a comprobar los pins del comienzo de la lección antes de escribir otra línea.

5. **La aprobación: el suscriptor se anota.** Crea `crank/approve.ts`. Esto es lo único que el suscriptor llega a correr:

   ```ts
   // crank/approve.ts: the SUBSCRIBER runs this once. It is the whole sign-up flow.
   import { address } from '@solana/kit';
   import { getApproveCheckedInstruction, TOKEN_PROGRAM_ADDRESS } from '@solana-program/token';
   import { resolveAta, toBaseUnits } from 'transfer-kit';
   import { loadSigner, sendIxs } from './send';

   const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const DECIMALS = 6;
   const CRANK = address(process.env.CRANK_ADDRESS ?? '');

   async function main() {
     const subscriber = await loadSigner(process.env.SUBSCRIBER_KEYPAIR ?? 'subscriber.json');
     // Third seed, the owning token program, required since the roster lesson.
     // Devnet USDC is a classic Token mint, so it is static here.
     const subscriberAta = await resolveAta(subscriber.address, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);

     const approveIx = getApproveCheckedInstruction({
       source: subscriberAta,
       mint: USDC_DEVNET,
       delegate: CRANK,
       owner: subscriber,
       amount: toBaseUnits('60', DECIMALS), // four months of the 15-USDC plan
       decimals: DECIMALS,
     });

     const signature = await sendIxs(subscriber, [approveIx]);
     console.log(`approved: delegate=${CRANK} allowance=60 USDC sig=${signature}`);
   }

   main().catch((e) => {
     console.error(e);
     process.exit(1);
   });
   ```

   Córrelo con la dirección del crank en el entorno:

   ```bash
   CRANK_ADDRESS=$(solana-keygen pubkey crank.json) npx tsx crank/approve.ts
   ```

   Checkpoint: una línea `approved:` con una firma. Busca la transacción en un explorador en devnet y lee el `ApproveChecked` parseado: source, delegate, 60 USDC. Esa es toda la huella contractual del suscriptor.

6. **El pull: el ciclo programado del comercio.** Crea `crank/pull.ts`, el scaffold trabajado que esta lección te entrega entero. Lee, aplica la guarda, hace el pull, y vuelve a leer:

   ```ts
   // crank/pull.ts: the MERCHANT backend runs this once per billing cycle.
   import {
     AccountRole,
     address,
     generateKeyPairSigner,
     unwrapOption,
     type Instruction,
   } from '@solana/kit';
   import {
     fetchToken,
     getTransferCheckedInstruction,
     TOKEN_PROGRAM_ADDRESS,
   } from '@solana-program/token';
   import { getAddMemoInstruction } from '@solana-program/memo';
   import { fromBaseUnits, resolveAta, toBaseUnits } from 'transfer-kit';
   import { checkPull } from './guard';
   import { loadSigner, rpc, sendIxs } from './send';

   const USDC_DEVNET = address('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
   const DECIMALS = 6;
   const PLAN = '15'; // USDC per cycle
   const SUBSCRIBER = address(process.env.SUBSCRIBER_ADDRESS ?? '');
   const MERCHANT = address(process.env.MERCHANT_ADDRESS ?? '');

   async function main() {
     const crank = await loadSigner(process.env.CRANK_KEYPAIR ?? 'crank.json');
     const subscriberAta = await resolveAta(SUBSCRIBER, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);
     const merchantAta = await resolveAta(MERCHANT, USDC_DEVNET, TOKEN_PROGRAM_ADDRESS);

     // 1. Read the account. Never pull on a cached view of the delegate slot.
     const tokenAccount = await fetchToken(rpc, subscriberAta);
     const delegate = unwrapOption(tokenAccount.data.delegate);
     const delegatedAmount = tokenAccount.data.delegatedAmount;

     // 2. Guard. The chain would reject a bad pull anyway; the guard names WHY first.
     const decision = checkPull({
       delegate,
       delegatedAmount,
       crank: crank.address,
       pullBase: toBaseUnits(PLAN, DECIMALS),
     });
     if (!decision.ok) {
       console.log(`refused: ${decision.reason}`);
       process.exit(1);
     }

     // 3. Build the pull: TransferChecked signed by the CRANK, not the owner,
     //    with a fresh reference key and a memo, the same shape transfer-kit taught.
     const reference = (await generateKeyPairSigner()).address;
     const transferIx = getTransferCheckedInstruction({
       source: subscriberAta,
       mint: USDC_DEVNET,
       destination: merchantAta,
       authority: crank, // the delegate signs; the subscriber signs nothing today
       amount: decision.pullBase,
       decimals: DECIMALS,
     });
     const transferWithReference: Instruction = {
       ...transferIx,
       accounts: [...transferIx.accounts, { address: reference, role: AccountRole.READONLY }],
     };
     const memoIx = getAddMemoInstruction({ memo: 'WVL-CLUB cycle pull' });

     const signature = await sendIxs(crank, [transferWithReference, memoIx]);

     // 4. Re-read: the on-chain allowance is the ledger, our math is a preview.
     const after = await fetchToken(rpc, subscriberAta);
     console.log(`pulled ${PLAN} USDC sig=${signature} ref=${reference}`);
     console.log(
       `allowance remaining: ${fromBaseUnits(after.data.delegatedAmount, DECIMALS)} USDC (expected ${fromBaseUnits(decision.remainingAfter, DECIMALS)})`,
     );
   }

   main().catch((e) => {
     console.error(e);
     process.exit(1);
   });
   ```

   Tres detalles ahí adentro se ganan sus líneas. `fetchToken` decodifica la cuenta cruda en campos tipados, y `delegate` vuelve como una option que desenvuelves para sacar una dirección o null: null y "alguien más" son historias de rechazo distintas, y tu guarda las distingue. La reference key es una dirección recién generada agregada a la lista de cuentas de la transferencia como un meta readonly, el mismo truco de conciliación que tu checkout usa desde el módulo 3, así que el back office de m04 puede encontrar este pull por reference como cualquier venta. Y el crank es el fee payer: los ingresos recurrentes le cuestan al comercio 5000 lamports por ciclo en comisiones base.

   La decisión interesante es lo que NO está acá: el `sendStablecoin` de transfer-kit está ausente, a propósito, porque esa función firma como el dueño de la cuenta de origen, y hoy todo el punto es que el dueño está dormido. El crank le pide prestados los helpers del kit (`resolveAta`, `toBaseUnits`, `fromBaseUnits`) y reconstruye el envío con `authority: crank`. Cuando la diferencia de una línea fuerza una función nueva, esa línea es la lección.

   Todavía no puedes correr `pull.ts` con éxito; su guarda sigue lanzando. Ese orden es deliberado. Ve a llenar los TODOs.

![Cada ciclo del crank lee la cuenta fresca, rechaza con delegate-revoked o insufficient-allowance, o deja pasar un TransferChecked firmado por el delegado, y después vuelve a leer para confirmar el límite decrementado.](assets/v07-flowchart.png)

## Challenge

**Worked.** Hecho arriba: la aprobación aterrizó, el scaffold del crank existe, y `npx tsx crank/guard.test.ts` falla en el TODO nombrado. Si falla en cualquier otra cosa, una ruta de import o un typo te está mintiendo; arregla eso primero.

**Completion.** Llena los tres TODOs de la guarda desde la tabla de decisión: rechaza con `delegate-revoked` cuando el slot está vacío o tiene una dirección ajena, rechaza con `insufficient-allowance` cuando el pull excede el monto restante, y si no devuelve `ok` con el `remainingAfter` decrementado. Son alrededor de una docena de líneas. Aceptación, en dos etapas. Offline primero: `npx tsx crank/guard.test.ts` imprime exactamente `guard: all crank cases passed (over-pull and revoked-delegate rejected)`. Después el libro mayor de devnet:

```bash
export SUBSCRIBER_ADDRESS=$(solana-keygen pubkey subscriber.json)
export MERCHANT_ADDRESS="<your module-2 merchant wallet>"
npx tsx crank/pull.ts   # allowance remaining: 45.000000
npx tsx crank/pull.ts   # allowance remaining: 30.000000
```

Dos pulls, dos firmas, y el límite restante impreso bajando a 45 y después a 30 con el valor esperado coincidiendo. Ahora fuerza el tercer rechazo sin esperar a que se vacíe el tanque: pon `PLAN` temporalmente en `'45'` y corre de nuevo. La guarda tiene que imprimir `refused: insufficient-allowance` y, críticamente, ninguna transacción aparece en devnet; rechazaste antes de mandarla, ninguna comisión gastada. Pon `PLAN` de vuelta en `'15'`.

**Solo.** Detecta el desalojo. Corre un segundo `ApproveChecked` desde el suscriptor aprobando una dirección DISTINTA como delegado (genera un keypair desechable para hacer de cafetería). La próxima corrida de `pull.ts` de tu crank no puede tirar un error a ciegas ni puede decir solamente "revocado": extiende el camino del pull para que un delegado que está puesto-pero-ajeno (desalojo) y un delegado que está ausente (revoke pelado) queden registrados por separado: deja el tipo de dos motivos de la guarda intacto, y en un archivo JSON de estado chico al lado de los scripts escribe `{ "state": "delegate-revoked", "cause": "evicted" }` frente a `"cause": "revoked"`, y después omite la cuenta en los ciclos futuros hasta que aparezca una aprobación fresca al crank. Aceptación: dos pulls consecutivos tienen éxito y decrementan; un tercer pull por encima del tope se rechaza antes de mandarlo; reaprobar a un delegado distinto voltea el estado guardado del crank a `delegate-revoked`; y un `ApproveChecked` fresco de vuelta al crank hace que los pulls se reanuden. Ese rastro de comprobantes, dos firmas que decrementan, un rechazo razonado, un desalojo detectado, es la barrera de maestría de esta lección. El libro mayor lo demuestra; ningún quiz puede.

Si un pull falla con un custom program error `0x4` del programa Token, esa es la forma on-chain de escribir "owner does not match," que para una transferencia firmada por el delegado quiere decir que el slot de delegado de la cuenta no tiene a tu crank: o fue revocado, desalojado, o vaciado a cero por un pull anterior, o tu guarda leyó una cuenta y tu transferencia apuntó a otra, casi siempre una variable de entorno `SUBSCRIBER_ADDRESS` apuntando al keypair equivocado.

Una nota para el loop de feedback: este es el módulo donde el curso empieza a confiarte scaffolds en vez de archivos terminados, y los TODOs de la guarda están calibrados a la tabla de decisión de arriba. Si te llevaron más de veinte minutos, o la detección del desalojo se sintió sub-especificada, dilo en el feedback del curso; cuánto apoyo te entrega la próxima revisión se ajusta con exactamente estos reportes. Las partes que te pelearon son las partes que la próxima revisión afila.

Da un paso atrás y mira lo que entregaste: un comercio que factura a un cliente dormido, no puede exceder un techo firmado por el cliente, pierde su permiso en el instante en que el cliente cambia de opinión, y sabe cómo decir por qué se rechazó un pull. Eso son ingresos recurrentes reales sin custodia en ninguna parte, y dos pulls que decrementan en devnet demuestran que la primitiva funciona. Ahora vence la factura de antes. On-chain, toda esta suscripción es un número y una dirección: no hay nombre de plan, no hay cadencia, no hay vencimiento, no hay reinicio por período, no hay registro de por qué existe el delegado. Cada uno de esos lo reconstruiste tú mismo, en un archivo JSON que está al lado de los scripts, lo cual está bien para un club y es poco serio con mil suscriptores. La primitiva cruda está auditada por construcción, porque ES el programa Token, pero no modela nada; o sigues reconstruyendo la semántica de facturación off-chain para siempre, o te pasas a un programa que la lleve on-chain. Y hay un problema más difícil que los metadatos. Tu suscriptor puede tener exactamente un delegado vivo, así que en el momento en que se suscribe a un segundo comercio tu crank queda desalojado en silencio, y acabas de verlo pasar en el challenge solo. Una primitiva que castiga a tu cliente por gustarle dos productos todavía no es un sistema de facturación. La próxima lección: el programa oficial de Subscriptions, que ocupa ese único slot una vez y hace que una cuenta de token lleve muchos acuerdos de facturación.
