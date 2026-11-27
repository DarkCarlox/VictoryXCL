# Join VictoryXCL – Collaborative AI Optimization & Verification

`VictoryXCL` (`https://github.com/CarlosB456/VictoryXCL`) is an open collaborative effort where anyone can point **any** AI coding agent (Antigravity, Claude Code, Cursor, Codex, Gemini CLI, DeepSeek, Qwen, etc.) at real bottlenecks in the Xbox Series X|S → Windows compatibility layer.

> **Note on `AGENTS.md` / `CONTRIBUTING.md`:** All AI models are welcome in `VictoryXCL`. You do **not** need a local copy of any retail game (`xcl-smoke` is optional in PRs) — the maintainer's hardware gatekeeper (`scripts/eval-pr.ps1`) runs the real benchmark automatically.

Nothing is merged on trust. Every Pull Request passes through two strict gates on the maintainer's hardware:
1. **Static Security & Anti-Slop Audit (`SECURITY-GATE.md`)** before any untrusted code is compiled or executed.
2. **Real Hardware Verification (`scripts/eval-pr.ps1`)** — Rust unit tests + release build + timed `VictoryXCL.exe` execution comparing FPS, `write-watch` ms, memory commit, and crash/device-removal counts against the current baseline.

---

## Rules for Contributors & AI Agents

Read `SECURITY-GATE.md` before writing code. Any PR violating these rules is rejected immediately:

1. **Zero AI Slop:**
   - Do NOT disable, weaken, or `#[ignore]` existing tests to make checks pass.
   - Do NOT fake FPS or pass metrics by skipping real rendering work, stubbing out active systems without an explicit flag, or altering `smoke.ps1` / `eval-pr.ps1` / `pr_check.py`.
   - Every optimization or fix must be backed by real unit tests (`cargo test`) and a clear technical explanation in the PR body.
2. **Zero Untrusted Side Effects (Strict Security):**
   - Do NOT modify `build.rs`, `.github/workflows/`, `scripts/`, or add third-party crates to `Cargo.toml` / `Cargo.lock` unless the task explicitly requires it and explains why.
   - Do NOT add network requests, telemetry, dynamic command execution (`std::process::Command`, PowerShell, `cmd.exe`), registry/persistence writes, or file system access outside the documented `XCL` data/VFS paths.
   - Keep `unsafe` blocks minimal, documented with `// SAFETY:` invariants, and scoped to the task.
3. **One Task, One Clean PR:**
   - Pick **one** open task from `TASKS.json` / `TASKS.md`.
   - Keep changes localized to the relevant module(s). Never commit game files, `.exe`/`.dll`/`.rpf` binaries, logs, or API keys.

---

## Quick Start for Your AI Agent

Tell your AI agent:

> "Fork `CarlosB456/VictoryXCL`, read `JOIN.md`, `SECURITY-GATE.md`, and `TASKS.json`, pick one open task, implement the fix with unit tests, verify locally, and prepare a clean Pull Request."

Exact workflow:

```powershell
# 1. Fork & clone (or if you are the repo owner testing locally, clone directly and push a branch)
gh repo fork CarlosB456/VictoryXCL --clone
cd VictoryXCL

# 2. Pick a task ID from TASKS.json / TASKS.md and branch from main
git checkout -b task/<task-id>

# 3. Implement your change + unit tests (see target files in TASKS.json)

# 4. Run tests and release build locally
cargo test --release -p xcl --lib -- --test-threads=1
cargo build --release --bin VictoryXCL

# 5. (Optional, if you own a local title) Run smoke comparison against main
.\scripts\smoke.ps1 -Game "D:\Path\To\Title" -Seconds 60 -Xcl .\target\release\VictoryXCL.exe

# 6. Push and open Pull Request against CarlosB456/VictoryXCL (base: main)
git push -u origin task/<task-id>
gh pr create --repo CarlosB456/VictoryXCL --base main --title "[<task-id>] Concise summary"
```

---

## How PRs Are Judged

The maintainer runs `.\scripts\eval-pr.ps1` on real target hardware (AMD Radeon RX 6600 8 GB VRAM, 16 GB RAM, Windows 10 `lockhart` mode):

- **REJECTED** if any security rule fails, tests fail, build fails, crashes/device removals > 0, or FPS/memory regresses.
- **APPROVED & CREDITED** if static security passes, all tests pass, `0` crashes/device removals occur, and metrics improve (higher FPS, lower `write-watch`/lock contention, lower DXC/PSO stalls, or fixed stub/crash) — your GitHub handle and contribution are recorded in `LEADERBOARD.md`.
