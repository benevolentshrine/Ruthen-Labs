# Homebrew formula for SANDBOX — Security Cage Engine (Project RUTHENLABS)
# Install: brew install --build-from-source sandbox
# Or tap:  brew tap sayan5069/ruthenlabs && brew install sandbox

class Sandbox < Formula
  desc "Security Cage engine for AI-generated code — Project RUTHENLABS"
  homepage "https://github.com/sayan5069/RuthenLabs.co"
  url "https://github.com/sayan5069/RuthenLabs.co/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "Apache-2.0"
  head "https://github.com/sayan5069/RuthenLabs.co.git", branch: "main"

  depends_on "rust" => :build

  # SANDBOX is Unix-only (Linux + macOS)
  # No Windows support — Unix sockets are a core requirement
  depends_on :macos

  def install
    cd "sandbox" do
      system "cargo", "build", "--release"
      bin.install "target/release/sandbox"
    end

    # Install documentation
    doc.install "sandbox/README.md"
    doc.install "sandbox/ARCHITECTURE.md"
    doc.install "sandbox/AGENTS.md"
    doc.install "sandbox/CHANGELOG.md"
  end

  def post_install
    # Create runtime directories
    (var/"log/sandbox").mkpath
    (var/"lib/sandbox").mkpath

    # Socket directory (in /tmp, recreated at boot)
    ohai "SANDBOX installed successfully 🥊"
    ohai "Run 'sandbox daemon' to start the socket server"
    ohai "Run 'sandbox tui' to launch the dashboard"
    ohai "Socket will be created at /tmp/ruthenlabs/sandbox.sock"
  end

  def caveats
    <<~EOS
      SANDBOX — Security Cage Engine for Project RUTHENLABS
      "What runs here, stays here."

      Quick Start:
        sandbox daemon &          # Start background socket daemon
        sandbox tui               # Launch visual dashboard
        sandbox cage --input f.wasm --mode strict  # Sandbox a WASM binary

      Socket path: /tmp/ruthenlabs/sandbox.sock (created automatically)
      Audit logs:  #{var}/log/sandbox/
      Hash DB:     #{var}/lib/sandbox/

      The /tmp/ruthenlabs directory is created on first run.
      On macOS, /tmp is cleaned on reboot — the socket will be recreated.
    EOS
  end

  test do
    assert_match "SANDBOX Security Cage", shell_output("#{bin}/sandbox --help")
    assert_match version.to_s, shell_output("#{bin}/sandbox --version")
  end
end
