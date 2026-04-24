# Homebrew formula for BORU — Security Cage Engine (Project MOMO)
# Install: brew install --build-from-source boru
# Or tap:  brew tap sayan5069/momo && brew install boru

class Boru < Formula
  desc "Security Cage engine for AI-generated code — Project MOMO"
  homepage "https://github.com/sayan5069/Momo.co"
  url "https://github.com/sayan5069/Momo.co/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "Apache-2.0"
  head "https://github.com/sayan5069/Momo.co.git", branch: "main"

  depends_on "rust" => :build

  # BORU is Unix-only (Linux + macOS)
  # No Windows support — Unix sockets are a core requirement
  depends_on :macos

  def install
    cd "boru" do
      system "cargo", "build", "--release"
      bin.install "target/release/boru"
    end

    # Install documentation
    doc.install "boru/README.md"
    doc.install "boru/ARCHITECTURE.md"
    doc.install "boru/AGENTS.md"
    doc.install "boru/CHANGELOG.md"
  end

  def post_install
    # Create runtime directories
    (var/"log/boru").mkpath
    (var/"lib/boru").mkpath

    # Socket directory (in /tmp, recreated at boot)
    ohai "BORU installed successfully 🥊"
    ohai "Run 'boru daemon' to start the socket server"
    ohai "Run 'boru tui' to launch the dashboard"
    ohai "Socket will be created at /tmp/momo/boru.sock"
  end

  def caveats
    <<~EOS
      BORU — Security Cage Engine for Project MOMO
      "What runs here, stays here."

      Quick Start:
        boru daemon &          # Start background socket daemon
        boru tui               # Launch visual dashboard
        boru cage --input f.wasm --mode strict  # Sandbox a WASM binary

      Socket path: /tmp/momo/boru.sock (created automatically)
      Audit logs:  #{var}/log/boru/
      Hash DB:     #{var}/lib/boru/

      The /tmp/momo directory is created on first run.
      On macOS, /tmp is cleaned on reboot — the socket will be recreated.
    EOS
  end

  test do
    assert_match "BORU Security Cage", shell_output("#{bin}/boru --help")
    assert_match version.to_s, shell_output("#{bin}/boru --version")
  end
end
