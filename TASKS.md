# VictoryXCL Open Task Board ("The Desk")

**Project Goal:** Stable, playable **30 FPS** gameplay with **zero visual/graphical bugs**, fast shader compilation, and eliminated **CPU / GPU / RAM / HDD / NVMe** bottlenecks.

Pick **one** open task below (machine-readable mirror in [`TASKS.json`](TASKS.json)), implement it cleanly with unit tests, and submit a PR against `CarlosB456/VictoryXCL` (`main`).

**Current Verified Baseline (`cc255b5` on RX 6600 8 GB + 16 GB RAM, `lockhart`):**
- Steady gameplay/menus stretches: **`29.8–30.0 fps`**
- Bottleneck dips during heavy world streaming / cold PSOs: **`6.9–15.5 fps`**
- `write-watch` cumulative time at `2,048` submits: **`22,328 ms`** (`2,438 ms` lock wait, `32,768` GPU-view textures)
- Command list resubmit warnings: **`#256–#512`** (`a command list is submitted while its previous submit is still on the GPU`)

| ID | Category | Level | Pts | Goal | Target Files | Status |
|---|---|---|---|---|---|---|
| `perf-write-watch-stretch` | CPU / RAM | Hard | 100 | Reduce `GetWriteWatch` (`22.3 s`) via contiguous stretch / dirty-page optimizations | `title_textures.rs`, `kernelx.rs`, `uma.rs` | Open |
| `perf-bitcode-cleanup` | Shader Compiler / CPU | Hard | 90 | Clean-up passes over translated shaders before DXC to cut compile spikes & commit | `gpu/recomp/` | Open |
| `perf-shader-prep-cache` | Shader Compiler / Stutter | Hard | 85 | Speed up background shader/PSO prep & `.plib` cache hits to eliminate menu/cutscene stalls | `shaders.rs`, `pso_async.rs`, `real.rs` | Open |
| `visual-flicker-nan-audit` | Visual Bugs / Flicker | Hard | 85 | Eliminate flicker, bright flashes & NaN/Inf propagation in compute/mesh/post-process passes | `emit.rs`, `ambient.rs`, `standin.rs` | Open |
| `perf-nvme-hdd-streaming` | HDD / NVMe / Storage | Hard | 80 | Optimize DirectStorage batching & parallel zlib/BCPack decompression (`65k+` requests) | `d3dstorage.rs`, `title_textures.rs` | Open |
| `perf-gpu-descriptor-lock` | GPU / CPU Frame Time | Solid | 75 | Reduce per-draw allocations and descriptor/view mutex contention | `bind.rs`, `console_heap.rs`, `list_x.rs` | Open |
| `perf-resubmit-wait` | GPU / Sync | Solid | 65 | Eliminate `#256+` command-list resubmit waits safely without D3D12 ID 553 faults | `alloc_pool.rs`, `real.rs` | Open |
| `perf-retire-copies` | Visual Bugs / GPU | Solid | 60 | Retire replaced buffer/texture copies behind GPU fences instead of immediate Drop | `vram.rs`, `title_textures.rs`, `real.rs` | Open |
| `perf-commit-vram-budget` | RAM 16GB / VRAM 8GB | Solid | 60 | Tighten staging, upload ring & heap reuse to stop low-commit thrashing on 16 GB RAM | `texture.rs`, `vram.rs`, `sparse.rs` | Open |
| `compat-cmdlist10-slot101` | Compatibility / Visuals | Solid | 55 | Implement `ID3D12GraphicsCommandList10::#101`, `CopyMemoryToMemoryX` non-UMA & unaligned UAVs | `list_x.rs`, `real.rs`, `console_heap.rs` | Open |
| `perf-texture-min-lod` | VRAM 8GB / RAM 16GB | Warmup | 35 | Add configurable `ResourceMinLODClamp` on placed texture views for 8 GB VRAM | `title_textures.rs`, `vram.rs` | Open |

> See [`JOIN.md`](JOIN.md) for how to contribute and [`SECURITY-GATE.md`](SECURITY-GATE.md) for mandatory security, scope, and anti-slop rules.
