# macOS observability research (Workstream F)

Status: harness + verification shipped 2026-09-13. One-trace-id correlation
(failure/slow-op/lifecycle × traces/logs/metrics/exemplars/session/release)
proven on the stub. Parallax-side dSYM symbolication and MetricKit ingestion
specified, not implemented.

- `native-macos-observability.md` — what excellent native macOS observability
  requires: signals, Apple APIs, SDK comparison (Sentry Cocoa 9.28.0, Embrace,
  opentelemetry-swift), dSYM workflow, and what Parallax must build.
- `gap-matrix.md` — macOS workflow/capability matrix vs best implementations
  (playground column matches shipped harness).
- `verification-report.md` — what the harness proves on real macOS hardware,
  exact commands, and the honest blocked list.

Harness: `parallax-telemetry-playground/macos/` (README + `tools/verify.py`,
38/38). Live Parallax join is a separate step when a server is up.

Proven on macOS 26.6.2 arm64, Swift 6.3.3. HEADs in the verification report.
