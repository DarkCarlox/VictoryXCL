# VictoryXCL Contributors & Credits Leaderboard

Every contributor whose Pull Request passes the **Static Security & Anti-Slop Audit (`SECURITY-GATE.md`)** and demonstrates verified improvement on target hardware (`scripts/eval-pr.ps1`) is recorded here.

## Upstream Foundations & Credits

| Project | Repository | Key Contributions Integrated |
|---|---|---|
| **XCL** | `https://github.com/xcl-dev/xcl` | Core `amd-dev` + `main` compatibility layer, RDNA2→DXIL recompiler, UMA/sparse memory, HLE `kernelx`/`d3d12_xs`/`DirectStorage`/`xapu` |
| **VIctory** | `https://github.com/Marvisak/VIctory` | `XCL_NAN_GUARD` concept and D3D12 execution architecture |
| **cool_fox** | `https://github.com/DoubleGhosty/cool_fox` | Loader FPS/resolution patches, `Renderer Port 1`, 16 GB UMA sizing, `lane_masks`, `XCL_WATCH_GAP_MB=64` |
| **VIctory-Belobug-AMD** | `https://github.com/Belobug/VIctory-Belobug-AMD` | ThinLTO + `codegen-units = 1` (PR #96), AMD shader & sky-dome ambient stability insights |

## Verified Pull Request Leaderboard

| Rank | GitHub Handle | Points | Merged PRs | Verified Improvements |
|---|---|---|---|---|
| 1 | `@CarlosB456` | Baseline | Core Fusion | Background shader prep + HUD, `prepared-records-v1.bin`, `XCL_WATCH_COALESCE_MS`, 16 GB commit throttling, `xg.rs` GPU-view & reserved-page crash fix |
| 2 | `@DarkCarlox` | 75 | [#1](https://github.com/CarlosB456/VictoryXCL/pull/1) | `perf-gpu-descriptor-lock`: stack-inline vertex binding snapshots & single `MIRRORS.read()` per draw in `list_x.rs` |
