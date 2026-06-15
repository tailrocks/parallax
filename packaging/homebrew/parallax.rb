class Parallax < Formula
  desc "Local-first observability for agent-assisted development"
  homepage "https://github.com/tailrocks/parallax"
  license "Apache-2.0"

  disable! date: "2026-06-15", because: "parallax has not reached a stable release yet; use the rolling preview channel"

  conflicts_with "tailrocks/parallax/parallax-preview", because: "stable and preview install the same binary"

  def install
    odie "Stable binary releases are not available yet; install tailrocks/parallax/parallax-preview"
  end

  def caveats
    <<~EOS
      Start the local server (downloads a pinned GreptimeDB on first run):
        parallax serve
      Then open http://127.0.0.1:4000 — quickstart:
        https://github.com/tailrocks/parallax/blob/main/docs/guide/quickstart.md
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/parallax --version")
  end
end
