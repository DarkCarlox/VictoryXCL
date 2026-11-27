# VictoryXCL Mandatory Security & Anti-Slop Gate

**NEVER compile (`cargo build` / `cargo test`) or run (`VictoryXCL.exe`) an untrusted external Pull Request on the maintainer's machine until Phase 1 (Static Security Audit) passes completely.**

Rust `build.rs` scripts, procedural macros, and compiled binaries execute arbitrary code on the host OS. Therefore, **Antigravity and `scripts/eval-pr.ps1` MUST enforce two separate phases**:

---

## Phase 1 — Mandatory Static Audit (Before Any Build or Execution)

Before checking out executable PR code or running Cargo:

1. **Inspect the full diff (`git diff main...PR_HEAD`) — never skip this step.**
2. **High-Risk File Path Blocklist (Auto-Flag / Immediate Reject unless explicitly authorized):**
   - `build.rs`, `**/build.rs` (executes arbitrary code at compile time!)
   - `Cargo.toml`, `Cargo.lock` (untrusted crate dependencies or `[patch]` overrides)
   - `.github/workflows/**`, `scripts/**`, `*.ps1`, `*.bat`, `*.cmd`, `*.py`, `*.sh`
   - Any binary blob, `.exe`, `.dll`, `.pdb`, `.bin`, `.rpf`, or game assets (`pr_check.py` rules apply)
3. **Malicious / Dangerous Pattern Blocklist in Added Lines (`+`):**
   - **Process / Shell Spawning:** `std::process::Command`, `ShellExecute`, `CreateProcess`, `WinExec`, `powershell`, `cmd.exe`, `wscript`, `mshta`, `rundll32`.
   - **Unauthorized Network / Exfiltration:** `std::net`, `TcpStream`, `UdpSocket`, `reqwest`, `ureq`, `hyper`, `WinHttp`, `wininet`, `WSAStartup`, `socket(`, hardcoded IPs/URLs/webhooks (Discord/Telegram/etc.).
   - **Host Tampering / Persistence / Credential Theft:** writes to `HKLM` / `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, Scheduled Tasks, `System32`, browser profiles, `.ssh`, `.git-credentials`, `gh` tokens, `.env`, or reading `%USERPROFILE%` outside `XCL` data/game VFS roots.
   - **Injection / Evasion / Obfuscation:** `CreateRemoteThread`, `WriteProcessMemory`, `NtUnmapViewOfSection`, `VirtualAllocEx`, `SetWindowsHookEx` (outside existing `kernelx`/loader scope), base64/hex-encoded executable blobs, `include_bytes!` of unknown binaries, obfuscated control flow, or disabling Windows Defender / AMSI / firewall / logging.
4. **Anti-AI-Slop Rules:**
   - Reject PRs that delete or `#[ignore]` existing tests, relax assertions, or suppress compiler/runtime error handling silently (`.ok()` hiding failures).
   - Reject PRs that claim FPS gains by disabling real rendering passes, skipping draw/compute calls unconditionally, or adding fake sleep/frame counters.
   - Reject massive unrequested refactors or unrelated file edits.

Only after **both** `eval-pr.ps1` static scan passes **and** Antigravity / the maintainer verifies the diff has zero malicious or slop code may Phase 2 run with `-SecurityApproved`.

> [!TIP]
> For maximum host isolation when evaluating PRs from unknown contributors, run Phase 2 inside Windows Sandbox or a dedicated non-admin test VM with no GitHub credentials (`gh` logged out) or personal files mounted.

---

## Phase 2 — Build, Test & Hardware Benchmark Criteria

Once `-SecurityApproved` is passed:
1. Build and test inside an isolated temporary `git worktree` (never dirtying the main working tree).
2. `cargo test --release -p xcl --lib -- --test-threads=1` must pass **100%** (zero failures).
3. `cargo build --release --bin VictoryXCL` must exit `0`.
4. Timed live run (`scripts/smoke.ps1` or `eval-pr.ps1 -GameDir ...`) must satisfy:
   - `crashes == 0` (no `0xc0000005`, `panic`, or unhandled exception)
   - `device_removals == 0` (no D3D12 device removal / GPU hang)
   - Average/window FPS `>= baseline` (or significant `write-watch` / commit / shader-compile / stability improvement without FPS loss)
   - Peak commit within 16 GB host budget (no `VirtualAlloc(MEM_COMMIT)` failures)
