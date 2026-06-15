# Homebrew Packaging

Parallax follows the same Homebrew shape as Jackin:

- `Formula/parallax.rb` is the stable formula and stays disabled until the
  first stable release.
- `Formula/parallax-preview.rb` is CI-owned and points at binary tarballs on
  the rolling `preview` GitHub Release.
- `Aliases/parallax@preview` points to `../Formula/parallax-preview.rb`.
- The tap repository is per project: `tailrocks/homebrew-parallax`, matching
  `tailrocks/homebrew-holla`.

Before enabling preview publishing, create `tailrocks/homebrew-parallax` with:

```text
Formula/parallax.rb
Formula/parallax-preview.rb
Aliases/parallax@preview -> ../Formula/parallax-preview.rb
```

Then configure `GH_PARALLAX_HOMEBREW_TAP_TOKEN` in `tailrocks/parallax`. The
token needs contents read/write permission on `tailrocks/homebrew-parallax`.

Formulae must never build Parallax from source during install. Preview and
stable release workflows build archives with Zig/`cargo-zigbuild`, publish
them to GitHub Releases, and Homebrew only downloads those binaries. CI and
release tool dependencies are installed through `mise`, matching Jackin.
