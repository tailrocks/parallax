# macOS observability research (Workstream F)

Status: harness + verification shipped 2026-09-13. Parallax-side dSYM
symbolication and MetricKit ingestion specified, not implemented.

- `native-macos-observability.md` — what excellent native macOS observability
  requires: signals, Apple APIs, SDK comparison (Sentry Cocoa, Embrace,
  opentelemetry-swift), dSYM workflow, and what Parallax must build.
- `gap-matrix.md` — macOS workflow/capability matrix vs best implementations.
- `verification-report.md` — what the harness proves on real macOS hardware,
  exact commands, and the honest blocked list.

Harness: `parallax-telemetry-playground/macos/` (README + `tools/verify.py`).

Baseline: parallax `a3a240f30c935021f243bb47f1c38d277c9d8a81`; playground HEAD
recorded in the verification report. Proven on macOS 26.6.2 arm64,
Swift 6.3.3, Parallax debug build of current checkout + GreptimeDB 1.1.2.
