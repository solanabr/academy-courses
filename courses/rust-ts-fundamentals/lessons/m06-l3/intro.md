# Two Dockerfiles, one idea: multi-stage honesty

## Summary

m06-l2 installed a container runtime, built the image-container-layer-registry mental model, and shipped the poller's first naive image. You poked `/status` from the host, and the image's multi-gigabyte weight was left sitting on your screen on purpose. Today that weight comes off. One idea, applied twice: build in one box, ship only what runs in another. The Rust image gets the cargo-chef treatment, three build stages plus a debian slim runtime; the TS fleet gets a node slim multi-stage build of its own; both get measured on screen against your recorded baseline. Along the way the fleet-runner grows up: an interval service mode, because a container wants one long-lived foreground process, not a script that exits. The scaffolding contract, out loud: the Rust Dockerfile is worked in full and annotated, the node Dockerfile you author yourself from the same pattern with TODOs only at the stage boundaries, and the `.dockerignore` challenge is entirely solo. The training wheels from Containers 101 are off.

## Build in one box, ship in another

Start with the evidence. Terminal open, before any reading:

```bash
docker images pulse-pollerd
```

There it is, the number you wrote down last lesson, still in the gigabytes, for a binary you could attach to an email. And the weight is not a one-time embarrassment sitting on your disk. Every machine that ever pulls this image pays it again: the CI runner on every push, every teammate's first `docker run`, the future you on a new laptop, all of them downloading a full Rust toolchain and your entire build cache to run a few megabytes of compiled poller. Minutes and bandwidth, multiplied by every pull, forever.

The fix is one idea, and I want to state it in its smallest form before any Dockerfile syntax. A single-stage Dockerfile cannot tell "needed to build" from "needed to run", so it ships both. A multi-stage Dockerfile is just two boxes in one file: a build box with the whole toolchain, and a runtime box that starts nearly empty. Between them, one instruction, `COPY --from`, carries artifacts forward. And here is the rule that decides everything about what ships: the final image is the final stage's base plus whatever you explicitly `COPY` into it. Nothing else. The build box, gigabytes of it, is left behind on the build machine like scaffolding after the building opens.

![A heavy build box full of toolchain and cache is discarded while one copied binary lands in a small runtime box that becomes the shipped image.](assets/v01-diagram.webp)

That is the whole idea. Everything else in this lesson is engineering around two follow-up questions: how do you keep the build box fast when you rebuild it fifty times a day, and how small should the runtime box honestly be?

### cargo-chef: stop recompiling the world

The naive image had a second problem hiding behind its size, and you felt it in the lab: every rebuild was a cold `cargo build --release` of the entire workspace. The culprit is layer cache invalidation order (Docker reuses a cached layer only if the instruction and every input above it are unchanged; the first changed layer invalidates everything below). Our naive file did `COPY . .` and then built, which means any edit to any file, a comment in `main.rs`, a README typo, invalidated the copy layer and forced the build layer to recompile every dependency crate from zero. The dependencies did not change. Tokio did not change. You paid for them anyway, every time.

The fix is dependency-layer caching: arrange the Dockerfile so dependencies compile in their own layer, keyed only on the manifests, with your source arriving afterwards. Then a code-only edit invalidates the cheap source layers and the expensive dependency layer stays cached. Cargo makes this awkward to do by hand, because `cargo build` wants real source files present, not just `Cargo.toml`. Which is exactly the gap cargo-chef exists to fill.

cargo-chef (current release 0.1.78, checked against crates.io on 2026-09-02) is a cargo subcommand with two verbs. `cargo chef prepare` scans your workspace and writes a recipe file, a JSON description of your dependency graph with your actual source stripped out. `cargo chef cook` takes that recipe and builds just the dependencies, producing a `target/` directory full of compiled crates and nothing of yours. The trick is that the recipe only changes when your manifests change, so the cook step's layer survives every code edit you will ever make. Its README claims up to 5x faster builds from this caching; that is the author's number, not mine, and step 3 of the lab has you measure your own. A credit while we are here, because the origin is good: Luca Palmieri wrote cargo-chef for the deployment chapter of Zero to Production in Rust, and the three-stage Dockerfile you are about to write is lifted from his README on purpose. Do not reinvent a wheel the ecosystem already load-tested. On a CI runner that rebuilds on every push, this one layer is a godsend.

The README canon is four stages, three for building (chef, planner, builder) plus the runtime box that ships, and the shape matters more than the syntax:

![A four stage build pipeline where a code edit leaves the dependency cooking layer cached and only a manifest change recompiles dependencies.](assets/v02-flowchart.webp)

Why does the planner stage exist at all, instead of cooking directly? Because the recipe is the cache key. `prepare` reruns on every edit, but it is cheap and its output is deterministic: same manifests in, byte-identical `recipe.json` out. The builder stage copies only that file before cooking, so Docker compares the recipe, sees no change, and skips the cook. Your dependency compile is now keyed on your dependency graph instead of on your source tree, which is what we wanted the whole time.

### The node side, by hand

The fleet has the same disease in a different body. A naive node image would `COPY . .` and `pnpm install`, and every source edit would re-download and re-link the entire dependency tree. The cure is the same idea, and here you get to see it without a helper tool, because node's manifests are enough on their own: copy the lockfile and the `package.json` files first, install into their own layer, and only then copy source. cargo-chef automates for Rust exactly what you are about to do manually for node. Once you have written both, neither is magic.

![Copying lockfile and manifests before running the install gives the expensive layer a long cache life while source edits stay cheap below it.](assets/v03-annotated-code.webp)

Two node-specific decisions in that image deserve their why. The base is `node:24-slim`: Node 24 is the active LTS line as I write this on 2026-09-02, with Node 26 taking over the LTS torch on 2026-10-28, at which point the digit in that tag bumps and nothing else about the pattern changes. And pnpm gets installed explicitly, `npm i -g pnpm@11.25.0`, instead of the `corepack enable` one-liner you may have seen in older Dockerfiles. That is not a preference. On 2025-03-19 Node's TSC voted corepack out of the distribution, and from Node 25 onward it simply is not in the box; the runtime decided package managers are not its job. Node 24 still carries it, so the old line works today and breaks on the next base bump, which is the worst kind of works. An image build wants deterministic, self-contained steps, so we install the tool we need, pinned. The pin digit itself is a probe-at-write-time value: 11.25.0 is what the npm `latest` tag points at today, matching the `packageManager` pin already in your workspace from m03-l1, while pnpm 12 is actively publishing on its own tag. Check yours before you copy mine.

One more seam, and it is the interesting one. Why not just `COPY --from` the built `node_modules` into the runtime stage, the way we copy the Rust binary? Because a pnpm workspace's `node_modules` is a web of symlinks into the workspace, and `pulse-fleet`'s dependencies include `pulse-core`, which lives outside anything you would naively copy. Carry the directory out of context and the links dangle. pnpm ships a command for exactly this: `pnpm deploy` extracts one package from a workspace into a self-contained directory, real files, production dependencies only, workspace packages included. The build stage's last line will be:

```bash
pnpm --filter pulse-fleet --prod deploy --legacy /out
```

We run it with `--prod` and with `--legacy`, and the legacy flag deserves honesty: pnpm's current deploy wants a workspace-wide setting called `inject-workspace-packages`, which trades away the symlink freshness your dev loop has leaned on since m03-l4. The legacy copier asks for no such trade and is precisely right inside a throwaway build stage. Both behaviors are documented on pnpm's deploy page, verified 2026-09-02.

I will confess where this pattern's tuition came from, because I paid it in the stupidest currency, waiting. An old dashboard project of mine had `COPY . .` sitting one line above the install. Every commit, and I mean every commit, CI re-downloaded the node universe, four extra minutes, dozens of times a week, for months. Nobody noticed because it had always been that slow. The fix was moving one line up two lines. Read your build logs the way you read `docker history` last lesson; slow that is evenly distributed feels like weather, and it is usually one misplaced layer.

### One process, running forever

There is a mismatch we have been politely ignoring: the poller is a daemon, born to run forever, but the fleet-runner is a script. It sweeps its targets once, writes its results, and exits, because GitHub Actions cron re-invokes it on a schedule and that was the right shape for that home. Put that shape in a container and you get a box that starts, works for two seconds, and dies. Compose, next lesson, would dutifully restart it forever, and I have watched teams ship exactly that: a restart policy cosplaying as a scheduler, logs that read like a crash chronicle, process startup paid on every sweep. You can recognize the anti-pattern from a single `docker ps` cell:

```text
STATUS
Restarting (0) 2 seconds ago
```

Exit code zero, restarting anyway, forever. Restart policies are for recovering from failure. Scheduling is the program's job. A container wants one long-lived foreground process, so the honest fix is to give the runner a real service mode, an interval loop in the code itself, built in this lesson, not assumed. That is lab step 4, and the flag it adds is an interface: next lesson's compose file drives it through an environment variable, so the names we choose today are frozen the moment we choose them.

### Image-size honesty

Now the question every container tutorial answers too fast: how small should the runtime box be? The internet's reflex answer is alpine, a musl-based distribution whose base image is around ten megabytes, and for the node image the numbers even look friendly: the docker-node README quotes `node:alpine` at roughly 25% smaller than `node:slim`. For Rust, the same move means building against `x86_64-unknown-linux-musl` and shipping a static binary into a tiny box. Smaller image, faster pulls, what is the catch?

The catch has a name and a date. In May 2020, Andy Grove, of Apache Arrow and DataFusion, published a post titled "Why does musl make my Rust code so slow?". His multi-threaded benchmark ran about 30x slower on musl than on glibc. Not percent. Times. The culprit is musl's default allocator, which degrades hard under multi-threaded allocation, and our poller is precisely a multi-threaded tokio process. Later measurements put the penalty anywhere from 2x to 20x depending on workload, but the order of magnitude is the point, and every one of those numbers belongs to its benchmark, not to yours. Grove tried the standard fix, swapping in jemalloc. It segfaulted. His actual fix was moving his runtime image to debian slim and abandoning musl entirely. Sit with that: the man who wrote the musl complaint landed on the exact base this course teaches. ripgrep, meanwhile, went the other way and made it work: it ships jemalloc on its musl builds to this day, visible in its `main.rs`. Both are honest engineering answers. Neither is "alpine is smaller, use alpine".

![A timeline from the 2020 thirty-fold musl slowdown measurement through a failed jemalloc fix to debian slim, with later benchmarks spanning two to twenty times.](assets/v04-timeline.webp)

There is a second alpine trap, and this one does not degrade, it detonates. A Rust binary built for the musl target links musl statically. Alpine's own system libraries, including its OpenSSL, link musl dynamically. Put a statically-musl'd binary that also links C OpenSSL into an alpine box and the two musls meet at TLS handshake time, and the documented result is a segfault. The openssl crate documents a way out, its `vendored` feature, which compiles OpenSSL into your binary. The course canon is blunter and matches every Rust rules file in this repo: use rustls and skip the C TLS dependency entirely, no linkage clash to have. And here is the satisfying part: you are already living by it. reqwest 0.13 defaults to rustls, checked against the 0.13.4 feature list on docs.rs today, so the poller you built in m06-l1 has no OpenSSL to clash. What it does still need is CA certificates to verify the servers it probes, which is why the runtime stage in the lab installs exactly one package, `ca-certificates`, and nothing else.

So the honest comparison, on the axes that matter:

![Debian slim trades tens of megabytes for a working allocator and a shell, alpine trades throughput and TLS safety for size, distroless trades debuggability for attack surface.](assets/v05-comparison.webp)

The trade-off, stated once and plainly: the smallest image is not the fastest binary. musl buys you a ten-megabyte base and can cost you an order of magnitude of allocator throughput; debian slim costs tens of megabytes more and just works; distroless cuts attack surface and also cuts the shell you would reach for at 2 a.m. Multi-stage builds add their own quiet cost too, the one you now know how to pay: cache invalidation order becomes a design concern, and a Dockerfile with its layers in the wrong order recompiles the world every build while looking perfectly correct. Base image choice is a measured trade-off. Measure, choose, write the why in a comment, move on.

## Lab: cut both images

Two terminals, two workspaces: `pulse-rs` for the poller, `pulse-station` for the fleet. Your naive baseline number from m06-l2 goes at the top of a scratch note; every measurement below lands next to it.

1. **Rewrite the poller's Dockerfile.** In `pulse-rs`, replace last lesson's naive Dockerfile wholesale. This one is worked in full; read the annotations against the four-stage flowchart above:

```dockerfile
FROM rust:1.98 AS chef
RUN cargo install cargo-chef --locked --version 0.1.78
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin pulse-pollerd

FROM debian:trixie-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/pulse-pollerd /usr/local/bin/pulse-pollerd
COPY pulse.config.json .
ENV POLLER_PORT=8080
EXPOSE 8080
CMD ["pulse-pollerd"]
```

   The glosses, stage by stage. `AS chef` names a stage so later stages can `FROM` or `COPY --from` it; the chef stage is the shared toolchain, `rust:1.98` (the stable pin from last lesson, tag re-checked 2026-09-02) plus cargo-chef installed once, `--locked` so its own lockfile is honored, `--version 0.1.78` so the layer does not silently change under you. The planner copies everything but produces only `recipe.json`. The builder copies the recipe alone, cooks the dependencies into the layer that will outlive your edits, and only then copies your source and builds your crates. The runtime stage starts from `debian:trixie-slim`, the cargo-chef README's own choice, installs `ca-certificates` because a probe daemon that cannot verify TLS certificates is a paperweight, and receives exactly two files: the binary and the config the poller reads from its working directory. `ENV POLLER_PORT=8080` keeps your m06-l2 challenge working. Everything the chef, planner, and builder stages ever created is left behind.

2. **Build, measure, verify.** Your `.dockerignore` from last lesson still fences `target/` and `.git/`, so the context transfer stays small. Then:

```bash
docker build -t pulse-pollerd:slim .
docker images pulse-pollerd
```

   The first build is honest about its cost: it compiles your dependency tree once inside the cook layer, so expect minutes, comparable to the naive build. The payoff is the SIZE column: `pulse-pollerd:slim` should land at least an order of magnitude under your naive number and likely closer to two, gigabytes down to double-digit megabytes, a slim Debian base plus one binary plus one JSON file. Write the number next to the baseline. Then prove it still works:

```bash
docker run --rm -p 8080:8080 pulse-pollerd:slim
```

   From the second terminal, `curl -s localhost:8080/status` must answer with the same JSON as always. If instead the container dies instantly with an error naming a missing `.so` file, you have met the classic multi-stage failure in the wild, the "file not found" that is really a linker error: the runtime stage lacks a shared library the binary links against. Rerunning the failing container (`docker run --rm pulse-pollerd:slim`) prints the missing library's name in the error itself, and for the full list of what the binary expects, run ldd where the tools and the Linux binary both live, the builder stage: `docker build --target builder -t pulse-pollerd:builder .` then `docker run --rm pulse-pollerd:builder ldd /app/target/release/pulse-pollerd`. (Your host is no help here: macOS has no ldd, and the binary on your host is not the Linux binary inside the image anyway.) Our stack avoids the common case by construction, rustls instead of C OpenSSL, but the diagnostic is yours for life.

3. **Prove the cache does what I claimed.** Open any source file in `pulse-engine` or `pulse-pollerd`, change a log message or add a comment, save, rebuild with the same command, and read the log. The planner reruns and emits a byte-identical recipe, the `cargo chef cook` step prints `CACHED`, and the only compile work is your own crates: seconds to low minutes instead of the full dependency build. The README's up-to-5x claim is Luca's measurement on his projects; the ratio you just produced is yours, and yours is the one you get to quote. Now change a line in `Cargo.toml`, rebuild, and watch the cook layer honestly invalidate: the recipe changed, so the dependencies recompile once. That is the entire caching contract, demonstrated in two rebuilds.

4. **Give the fleet-runner a service mode.** Over to `pulse-station`. The fleet's entry has been run-once since m01-l3, and the first move is to make that explicit: wrap the existing sweep body, config load through probe loop through results write, in one function, `async function runSweep(configPath: string): Promise<void>`, changing nothing inside it. Then replace the entry's argument handling with this:

```ts
const args = process.argv.slice(2);

const flagAt = args.indexOf("--interval");
const intervalRaw = flagAt === -1 ? process.env.FLEET_INTERVAL : args[flagAt + 1];
if (flagAt !== -1 && intervalRaw === undefined) {
  console.error("--interval needs a value in seconds");
  process.exit(1);
}
const positional =
  flagAt === -1 ? args : args.filter((_, i) => i !== flagAt && i !== flagAt + 1);

const configPath = positional[0];
if (!configPath) {
  console.error("usage: fleet [--interval <seconds>] <config-path>");
  process.exit(1);
}

let intervalSecs: number | undefined;
if (intervalRaw !== undefined) {
  intervalSecs = Number(intervalRaw);
  if (!Number.isFinite(intervalSecs) || intervalSecs <= 0) {
    console.error(`--interval wants a positive number of seconds, got "${intervalRaw}"`);
    process.exit(1);
  }
}

const stop = new AbortController();
process.on("SIGINT", () => stop.abort());
process.on("SIGTERM", () => stop.abort());

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted) {
      resolve();
      return;
    }
    const timer = setTimeout(resolve, ms);
    signal.addEventListener(
      "abort",
      () => {
        clearTimeout(timer);
        resolve();
      },
      { once: true },
    );
  });
}

if (intervalSecs === undefined) {
  await runSweep(configPath);
} else {
  console.error(`fleet-runner: service mode, sweeping every ${intervalSecs}s`);
  while (!stop.signal.aborted) {
    await runSweep(configPath);
    await sleep(intervalSecs * 1_000, stop.signal);
  }
  console.error("fleet-runner: received stop signal, exiting cleanly");
}
```

   Read the interface first, because it is now frozen and next lesson's compose file depends on it: one positional config path, an optional `--interval <seconds>` flag, a `FLEET_INTERVAL` environment variable as the flag's fallback, and run-once behavior when neither is set, so the Actions cron from m01-l3 keeps working untouched. The loop itself is old friends in a new job, with one deliberate divergence to name before someone quotes m06-l1 at me: that lesson argued the tokio ticker's tick-to-tick timing beats sleeping at the bottom of the loop, and this loop sleeps at the bottom of the loop. On purpose. Its period is sweep time plus interval, and for a runner sweeping on the order of once a minute that stretch is noise, while the abortable sleep buys the clean shutdown that matters in a container; the poller keeps the ticker because over there the schedule IS the product. The sleep is an `AbortController` pattern you have owned since m02-l3, pointed inward this time: `SIGTERM` or Ctrl-C aborts the signal, which both breaks the loop condition and wakes the sleep immediately, so a stop request during a ten-minute nap does not wait ten minutes to be honored. The signal handling is not optional decoration either. Inside a container your process runs as PID 1, and `docker stop` sends it SIGTERM; a node process that ignores SIGTERM gets ten silent seconds and then SIGKILL, which is how services end up with truncated writes and no goodbye in the logs.

![One entry point either sweeps once and exits or loops sweep and sleep until a stop signal wakes the sleep and ends the loop cleanly.](assets/v06-flowchart.webp)

   Test it on the host before boxing it: `npx tsx src/fleet.ts pulse.config.json --interval 10` from `packages/pulse-fleet` should print the service-mode line, complete a sweep, pause ten seconds, sweep again, and exit cleanly on Ctrl-C with the goodbye line. Two sweeps in the log is the acceptance evidence, and the checkpoint wants it.

5. **Author the node Dockerfile.** Yours this time. The pattern is everything above; the TODOs are exactly the stage boundaries, which is to say the three decisions multi-stage exists for. At the `pulse-station` repo root, create `Dockerfile.fleet`, the explicit name, not the bare default, because this file lives at the repo root (the build needs the whole pnpm workspace as its context) while the poller's own `Dockerfile` lives down in `pulse-rs/`, and next lesson's compose file and CI job will reference the fleet's file by exactly this name:

```dockerfile
FROM node:24-slim AS base
RUN npm i -g pnpm@11.25.0

FROM base AS build
WORKDIR /app
# Dependency layer first, so a code edit cannot invalidate the install.
# TODO 1: COPY exactly what pnpm needs to resolve the workspace:
#   pnpm-lock.yaml, pnpm-workspace.yaml, the root package.json,
#   and each packages/*/package.json at its own path.
COPY ??? ???
RUN pnpm install --frozen-lockfile
# Source arrives AFTER the install layer.
# TODO 2: COPY the rest of the workspace in.
COPY ??? ???
RUN pnpm --filter pulse-core build \
 && pnpm --filter pulse-fleet build
RUN pnpm --filter pulse-fleet --prod deploy --legacy /out

FROM node:24-slim AS runtime
WORKDIR /app
# TODO 3: COPY --from the deployed bundle at /out into /app,
# and COPY the fleet's config file in next to it.
COPY ??? ???
COPY ??? ???
CMD ["node", "dist/fleet.js", "pulse.config.json"]
```

   Three small pieces of wiring before it can build, all yours to place. `pulse-fleet` needs a `build` script that emits `dist/`; the boring, correct choice is `tsc` with a `tsconfig.build.json` that extends your strict config, flips `noEmit` off, and sets `outDir` to `dist` with `rootDir` at `src` (build `pulse-core` first, as the Dockerfile already does; its m03-l4 build emits the type declarations the fleet's compile reads). `pulse-fleet`'s `package.json` needs `"files": ["dist"]` so `pnpm deploy` knows what to carry. And TODO 1's answer preserves paths: each package manifest is copied to its own directory, `packages/pulse-fleet/package.json` to `packages/pulse-fleet/`, because pnpm resolves the workspace by shape, not just by content. If your install layer reruns on every code edit after you think you are done, you copied too much into TODO 1; the annotated layer-order visual above is the debugging map.

6. **Build it, run it, measure both.** From the `pulse-station` root:

```bash
docker build -f Dockerfile.fleet -t pulse-fleet-runner:slim .
docker run --rm -e FLEET_INTERVAL=15 pulse-fleet-runner:slim
```

   Fair warning on the first command: the build context transfer line will be embarrassing, likely hundreds of megabytes, because this repo has no `.dockerignore` yet and `node_modules` rides along. It works anyway; the challenge makes it stop being embarrassing, with numbers. The run should print the service-mode line and then sweep every fifteen seconds; let two sweeps land, then `docker stop` it from the other terminal (`docker ps` for the name) and watch the clean-exit line arrive inside a second, your abortable sleep doing its job against a real SIGTERM. Then the final accounting:

```bash
docker images
```

Read the SIZE column into the shape below, your baseline note supplying the naive row:

![A ledger comparing the recorded naive image size against both new slim images, with every size cell filled by the reader's own measurement.](assets/v07-table.webp)

   Both slim tags next to your naive baseline, order of magnitude on screen, in your own numbers. That table plus the two-sweep log excerpt is the lesson's whole burden of proof.

**Go deeper (the 20%).** multi-stage builds, `COPY --from`, and layer caching are the working 80% of the build system; the machinery underneath is BuildKit, the builder that has been Docker's default for years (Docker's own docs no longer even state a since-version, and its current release is v0.32.2, checked 2026-09-02). Its documentation at [https://docs.docker.com/build/buildkit/](https://docs.docker.com/build/buildkit/) (verified live 2026-09-02) is the bookmark: cache mounts, secrets that never touch a layer, multi-platform builds, custom frontends. Nothing in this lesson or the next depends on the bookmarked material; when a build need outgrows what you learned today, that page is where the answer lives.

## Challenge

Solo, and it closes the loop the lab deliberately left open: `pulse-station` still ships its whole working directory as build context. Write the `.dockerignore`, and prove it with numbers, not vibes. Time the current build cold (`time docker build --no-cache -f Dockerfile.fleet -t pulse-fleet-runner:slim .`) and note the `transferring context` size from the log. Then fence the obvious dead weight, `node_modules`, every `dist/`, `.git/`, and re-run both measurements. Acceptance: the context transfer drops from hundreds of megabytes to single-digit, the timed build gets measurably faster, and the image still builds and runs its interval loop. One subtlety worth discovering on purpose: the build compiles inside the container from a clean install, so nothing in `node_modules` was ever needed in the context. If ignoring something breaks the build, that something was a real input, and the error names it; that is the `.dockerignore` feedback loop, and it is much friendlier than the `.gitignore` one.

## Checkpoint

Gate on doing, two pastes. First, the `docker images` table: your recorded naive baseline against `pulse-pollerd:slim`, the order-of-magnitude-or-better cut visible, with `pulse-fleet-runner:slim` alongside on its own terms (no naive fleet build ever existed to compare it against). Second, a log excerpt showing the containerized fleet-runner completing two interval sweeps and then exiting cleanly on `docker stop`, no restarts involved. If you also caught the cook layer printing `CACHED` on a code-only rebuild in step 3, you have personally verified the claim this lesson attributed instead of asserting, which is the habit that outlasts any particular tool.

What you can now do, concretely: split any Dockerfile into build and runtime stages and predict what ships from the COPY lines alone; order layers so dependency installs survive code edits, by hand in node and via cargo-chef in Rust; pick a runtime base as a measured trade-off and say out loud what alpine would cost your allocator and your TLS; and turn a run-once script into a signal-respecting service, which is the difference between a program that can live in a container and one that merely starts in it.

If the pnpm deploy step fought you, or the stage-boundary TODOs took more attempts than felt fair, say so in the course feedback with the error text; the node Dockerfile is this lesson's newest scaffolding and the reports decide whether the TODOs are pitched right.

Two lean images that run anywhere. Also: two lean images that exist on exactly one laptop, and "anywhere" starts with a registry other machines can pull from. Next lesson wires the station together locally, one compose file replacing both of your hand-typed run commands, and then the course's one CI pipeline learns to build both images and push them to GHCR, where SHIP #3 becomes an image a stranger's machine can pull.
