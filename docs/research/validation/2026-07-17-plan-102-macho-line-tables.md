# Plan 102: Mach-O line-table embed (structural fix)

Validation date: 2026-07-17  
Scope: post-link DWARF embed so `verify_object` accepts Apple release binaries.

## Problem

Apple `ld` (and the previous Linux zigbuild Mach-O path) leave only OSO
debug-map stabs in the final executable. Object files and `dsymutil` dSYM
companions contain real `__DWARF,__debug_line`, but the shipped single-file
archive had no line-table section. `cargo xtask release-package` therefore
failed with `release binary is missing line tables` on every Apple target.

Naive `llvm-objcopy --add-section` was already proven broken (filesize past
EOF). A symbol companion was rejected by the public archive contract.

## Fix

1. **Link-time header pad + packed dSYM (Apple targets only)** via
   [`.cargo/config.toml`](../../../.cargo/config.toml):
   - `-C split-debuginfo=packed` so rustc/dsymutil builds a companion while
     objects still exist
   - `-C link-arg=-Wl,-headerpad,0x10000` so a `__DWARF` load command fits
   - Workspace `profile.release.split-debuginfo = "off"` unchanged for Linux
     ELF (keeps `.debug_line` embedded)

2. **Post-link rewrite** in
   [`crates/parallax-xtask/src/release/macho_dwarf.rs`](../../../crates/parallax-xtask/src/release/macho_dwarf.rs),
   invoked from `release-package` / `validate_binary` for `*-apple-darwin`:
   - locate or run `dsymutil` → `binary.dSYM/.../DWARF/*`
   - insert `LC_SEGMENT_64` `__DWARF` **before** `__LINKEDIT` (Go layout:
     `vmaddr` shared with `__LINKEDIT`, `vmsize = 0`, file payload between
     data and linkedit)
   - relocate linkedit-relative load-command offsets
   - drop existing `LC_CODE_SIGNATURE`, then `codesign -s - -f`
   - re-verify `__debug_line` presence

3. **CI runner split**:
   - Apple targets: `macos-latest`, native `cargo build` (needs dsymutil +
     codesign + Apple ld headerpad)
   - Linux targets: `ubuntu-latest`, `cargo zigbuild` + glibc 2.17 floor
     (unchanged)
   - Removed unused cross macOS SDK step from release/preview package paths

4. **`scripts/release.sh`**: Apple → native cargo; Linux → zigbuild.

Archive contract stays **single top-level `parallax` executable** (no dSYM
in the tarball). Homebrew formula unchanged.

## Local proof (2026-07-17, aarch64-apple-darwin host)

```text
cargo test -p parallax-xtask --locked --offline --lib release::
# 15 passed, including:
#   release::macho_dwarf::tests::embeds_dsym_line_tables_into_apple_release_binary
#   release::macho_dwarf::tests::rejects_non_macho_bytes
#   release::tests::release_callers_use_one_packager_and_verified_sdk
```

The Apple integration test:

1. Builds a minimal release binary with `line-tables-only` + packed dSYM +
   headerpad
2. Confirms the linked executable has **no** `__debug_line`
3. Runs `ensure_line_tables` (embed + ad-hoc codesign)
4. Passes `verify_object` for `aarch64-apple-darwin` (section + symbolication
   via `addr2line` resolving `main` → `main.rs`)
5. Executes the rewritten binary successfully

## Plan 102 retirement

The four-target preview + tap acceptance evidence is in
[`2026-07-13-plan-102-release-baseline.md`](2026-07-13-plan-102-release-baseline.md)
§ Current-implementation four-target preview proof and § Retirement.
Plan 102 is retired (2026-07-17).

## Historical residual (closed)

- Publish one complete four-target preview from a green `main` head that
  includes this fix
- Per-target `cargo xtask release-verify` at the exact source SHA/ref
- Tap pull-workflow acceptance (sanitized evidence)

Do **not** treat the pre-verifier `4e8edfa` preview as proof of this path.
