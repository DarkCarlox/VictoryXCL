# VictoryXCL Open Task Board ("The Desk")

Pick **one** open task below (machine-readable mirror in [`TASKS.json`](TASKS.json)), implement it cleanly with unit tests, and submit a PR against `CarlosB456/VictoryXCL` (`main`).

**Current Verified Baseline (`cc255b5` on RX 6600 8 GB + 16 GB RAM, `lockhart`):**
- Steady gameplay/menus stretches: **`29.8–30.0 fps`**
- Bottleneck dips during heavy world streaming / cold PSOs: **`6.9–15.5 fps`**
- `write-watch` cumulative time at `2,048` submits: **`22,328 ms`** (`2,438 ms` lock wait, `32,768` GPU-view textures)
- Command list resubmit warnings: **`#256–#512`** (`a command list is submitted while its previous submit is still on the GPU`)

| ID | Level | Points | Goal | Target Files | Status |
|---|---|---|---|---|---|
| `perf-write-watch-stretch` | Hard | 100 | Reduce `GetWriteWatch` (`22.3 s`) via contiguous stretch / dirty-page optimizations | `title_textures.rs`, `kernelx.rs`, `uma.rs` | Open |
| `perf-bitcode-cleanup` | Hard | 80 | Clean-up passes over translated shaders before DXC to cut compile spikes & commit | `gpu/recomp/` | Open |
| `perf-resubmit-wait` | Solid | 65 | Eliminate `#256+` command-list resubmit waits safely without D3D12 ID 553 faults | `alloc_pool.rs`, `real.rs` | Open |
| `perf-retire-copies` | Solid | 60 | Retire replaced buffer/texture copies behind GPU fences instead of immediate Drop | `vram.rs`, `title_textures.rs`, `real.rs` | Open |
| `compat-cmdlist10-slot101` | Solid | 50 | Implement `ID3D12GraphicsCommandList10::#101` & safe `CopyMemoryToMemoryX` fallback | `list_x.rs`, `real.rs` | Open |
| `perf-texture-min-lod` | Warmup | 35 | Add configurable `ResourceMinLODClamp` on placed texture views for 8 GB VRAM | `title_textures.rs`, `vram.rs` | Open |

> See [`JOIN.md`](JOIN.md) for how to contribute and [`SECURITY-GATE.md`](SECURITY-GATE.md) for mandatory security & anti-slop rules.
