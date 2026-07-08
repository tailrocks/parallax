# Plan 047: GraphQL scenario family — field spans on, N+1 vs DataLoader contrast, partial errors, a real driver

> **Executor instructions**: Targets the **playground repository**
> (`parallax-telemetry-playground`). Follow step by step; run every
> verification. On any STOP condition, stop and report. When done, update the
> status row in the Parallax repo's `plans/README.md`.
>
> **Drift check (run first)**: in the playground repo,
> `git diff --stat ed1f975..HEAD -- services/catalog deploy scenarios graphql`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plan 042 (touches the same catalog service — land 042
  first to avoid churn); plan 037 (scenario catalog format)
- **Category**: direction
- **Planned at**: commit `408be17` (Parallax) / `ed1f975` (playground), 2026-07-07

## Why this matters

The GraphQL story (brief domain B; backlog A6b/A23/A24: field tree, N+1 vs
DataLoader, partial errors) is the playground's most complete-looking yet
least real family: catalog implements a genuine `@BatchMapping` DataLoader
and a subscription, but per-field data-fetcher spans are off (agent default),
the promised N+1 contrast doesn't exist, there is no partial-error case, and
**nothing ever drives the service** — no scenario, loadgen, CLI, or web code
calls `:8080`/GraphQL. Every GraphQL demo is therefore dark. This plan turns
the existing bones into reproducible trace shapes Parallax can visualize.

## Current state

Verified at playground commit `ed1f975`.

- `services/catalog/src/main/java/dev/tailrocks/catalog/CatalogApplication.java`:

  ```java
  // :59-64
  @QueryMapping
  List<Product> products() {
      productQueries.increment();
      boolean promo = flags.getBooleanValue("catalogPromo", false);
      return promo ? CATALOG : CATALOG;   // (plan 042 makes this real)
  }
  // :66-70 comment: reviews via @BatchMapping = ONE DataLoader call;
  //   "Contrast with a plain @SchemaMapping (which would be N+1)" — the
  //   contrast is only a comment, no N+1 path exists.
  // :78+ priceChanges subscription (GraphQL-over-WebSocket; yml sets
  //   spring.graphql.websocket.path=/graphql)
  ```

- Data-fetcher spans off: repo-wide grep for
  `data-fetcher`/`DATA_FETCHER` finds only the comment at
  `CatalogApplication.java:78`; the OTel Java agent's
  `otel.instrumentation.graphql.data-fetcher.enabled` defaults to false, so
  traces show only the operation span — no field tree.
- No driver: grep for `8080|graphql` across `scenarios/`, `loadgen/`,
  `cli/src`, `web/src` → zero hits; checkout's fan-out
  (`services/checkout/src/main.rs:163-165`) calls pricing/inventory/
  recommendation only. Compose publishes catalog at `8080:8080`
  (`deploy/docker-compose.yml:105`) with the comment "GraphQL (browser →
  catalog)" — aspirational.
- `graphql/` dir at the repo root holds the schema contract (check its
  content before adding fields).
- Java conventions: Spring Boot 4 + Spring GraphQL, GraalVM 25 toolchain,
  built via Gradle wrapper; the OTel agent is the upstream one
  (`deploy/Dockerfile.java`).

## Commands you will need

| Purpose | Command (playground root) | Expected |
|---------|---------------------------|----------|
| Java build | catalog's gradle wrapper (`./gradlew build -x test` from the service dir, or root wrapper if present) | exit 0 |
| Compose | `docker compose -f deploy/docker-compose.yml config` | exit 0 |
| Script lint | `bash -n scenarios/a6-graphql.sh` | exit 0 |

## Scope

**In scope** (playground repo):
- `services/catalog/` (schema + resolvers + yml if needed)
- `graphql/` (schema contract update, matching what's there)
- `deploy/docker-compose.yml` (catalog env: data-fetcher span flags)
- `scenarios/a6-graphql.sh` (create; drives all the cases), catalog rows
- optionally `web/` — ONLY a link/fetch already-planned in plan 050; do not
  wire web→catalog here

**Out of scope**:
- GraphQL→gRPC gateway (A23) and GraphQL→GraphQL (A24) — bigger topology
  additions; deferred (note in report).
- Parallax UI GraphQL-explorer surfaces (future; consumes these traces).
- flagd/promo behavior (plan 042).
- Subscription **load** driving beyond a smoke check (see Step 4 honesty
  rule).

## Git workflow

- Playground repo, `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Turn on field spans

Add to catalog's compose env:
`OTEL_INSTRUMENTATION_GRAPHQL_DATA_FETCHER_ENABLED: "true"` and
`OTEL_INSTRUMENTATION_GRAPHQL_ADD_OPERATION_NAME_TO_SPAN_NAME: "true"` if
the pinned agent supports it (check the agent version in
`deploy/Dockerfile.java` against the upstream instrumentation docs for the
exact property names — the data-fetcher flag also has a
`create-spans`/`create_or_add_link` variant in some versions; record what
the pinned version accepts).

**Verify**: compose config exit 0; after Step 4's driver runs, a `products`
trace shows child data-fetcher spans (record the observed span names).

### Step 2: N+1 contrast + partial error + high-cardinality case

In `CatalogApplication.java` (and the schema files in the service +
`graphql/` contract dir):
1. **N+1 contrast**: add `reviewsSlow: [Review!]!` on `Product` resolved via
   plain `@SchemaMapping` (one fetch per product — the comment at `:66-70`
   already names this shape). Same underlying data as `reviews`.
2. **Partial error**: add `Product.riskScore: Float` whose resolver throws
   for a known SKU (deterministic) — GraphQL returns 200 with `errors[]` +
   null field. 
3. **High-cardinality operation name**: nothing server-side — the driver
   sends `query lookup_<random-suffix> { ... }` to test span-naming policy
   (brief: server span name must stay the operation type, not the
   client-supplied name; record what the agent actually does).

**Verify**: Java build exit 0.

### Step 3: Driver scenario

Create `scenarios/a6-graphql.sh` (curl POSTs to
`http://localhost:8080/graphql`, JSON bodies):
- batched: `{ products { id name reviews { text stars } } }`
- N+1: `{ products { id name reviewsSlow { text stars } } }`
- partial error: query including `riskScore`
- high-cardinality op name: `query lookup_$RANDOM { products { id } }`
- print per case: "Check in Parallax: Traces → newest catalog trace —
  batched shows ONE reviews fetch span; N+1 shows one span per product;
  partial-error trace has 200 + error field span event/status; op-name case:
  span name stays `query`."
Register in `scenarios/run.sh` + `scenarios/README.md` (plan 037 format).

**Verify**: `bash -n` exit 0; live run against the stack + Parallax records
the four trace shapes (paste trace ids in the commit message).

### Step 4: Subscription smoke (honest scope)

The `priceChanges` subscription runs over GraphQL-WebSocket. Bash/curl can't
drive it; add the smallest honest check: a ~20-line Bun script
`scenarios/a7-subscription.ts` using the `graphql-ws` protocol over a plain
WebSocket (Bun has native WebSocket) that subscribes for ~10s, prints
received events, exits. Run via `bun scenarios/a7-subscription.ts`
(Bun-only rule — no Node). If the handshake protocol details rabbit-hole
past ~an hour, STOP on this step only, ship Steps 1-3, and record the
subscription driver as deferred.

**Verify**: script runs against the stack, receives ≥1 event; a long-lived
subscription span/trace appears (record). Register in the catalog with its
Bun invocation.

## Test plan

- Java build gate (no unit harness in the service — state it).
- The a6 driver's four recorded trace shapes ARE the acceptance tests;
  each maps to a brief backlog id (A6b, partial-error, naming policy).

## Done criteria

- [ ] Data-fetcher spans visible in catalog traces (recorded)
- [ ] `reviews` vs `reviewsSlow` produce visibly different trace shapes
      (1 batched span vs N per-product spans — recorded trace ids)
- [ ] Partial-error query returns 200 + `errors[]` and the trace marks the
      failed field
- [ ] High-cardinality op name does NOT become the server span name
      (recorded; if the agent misbehaves, report it as a finding)
- [ ] `a6-graphql.sh` (+ `a7-subscription.ts` or its recorded deferral) in
      the scenario catalog
- [ ] Java build green; compose config green
- [ ] Status row updated in Parallax repo `plans/README.md`

## STOP conditions

- The pinned OTel agent version ignores the data-fetcher flag (property
  renamed across versions) — report the version + correct property; do not
  upgrade the agent as a side effect (that's its own change).
- Schema changes conflict with the `graphql/` contract dir's role (it may
  be generated or hand-maintained — read it first) — align or report.
- Subscription driver exceeds the honesty budget (Step 4) — defer it,
  don't fake it.

## Maintenance notes

- Deferred explicitly: A23 (GraphQL→gRPC gateway: catalog calling payment)
  and A24 (GraphQL→GraphQL) — natural follow-up once this family is
  visible; also a browser→catalog fetch (plan 050 owns web work).
- Parallax's future GraphQL-explorer surface (brief: field tree, resolver
  latency) will consume exactly these traces — keep resolver/field names
  stable.
- Reviewer: `reviewsSlow` must hit the same data as `reviews` (the contrast
  must isolate the fetch pattern, not the data); the partial-error SKU must
  be documented in the script.
