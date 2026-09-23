# RUGID Technical Directive — Closing the Gap on the Engine Matrix

**Document ID:** RGD-DIR-001
**Date:** 2026-09-22
**Status:** Draft for review
**Scope:** All items marked ❌/⚠️ in `RUGID-ass.txt` (the "Great Engine Matrix"), verified against the codebase at commit `183d8ed` plus 8 uncommitted modified files.
**Audience:** Engineering leads, contributors, roadmap planners.

---

## 1. Executive Summary

RUGID is a 30,119-line Rust vector-physics engine with genuinely novel capabilities (relativistic ontology layout, analytic CCD, infinite-resolution SVG rendering, SPH fluids, content-addressable assets). The engine matrix scores it ~47/100. Verification against the codebase confirms the broad strokes but found **six materially wrong claims** in the matrix (both over- and under-stated), one failing test, and a broken default build.

This directive converts the matrix's remaining gap items into six phases of work over an estimated **15–21 months** with a 3-engineer team. The strategy is deliberate: **harden the differentiators first** (determinism, analytic rendering pipeline, asset/telemetry infrastructure), **fill platform-standard gaps second** (audio, gamepad, clipboard, hot reload), and **defer or reject parity-chasing** (VR, destruction, matchmaking, VCS integration) where the vector-first architecture makes "standard" solutions a poor fit.

**Immediate actions (this week):**
1. Fix the default `cargo build` failure — `src/bin/rgpum_test.rs` references the feature-gated `rugid::platforms::rgpum` module, breaking every fresh clone's build (fix: `required-features = ["rgpum"]` in `Cargo.toml`).
2. Fix or quarantine the failing test `ocrs::adaptive::tests::test_performance_budget` (`src/ocrs/adaptive.rs:164`) — it fails on uncommitted in-progress work in `src/ocrs/`.
3. Land the 8 modified files or revert them — the working tree has uncommitted work on `input.rs`, `ocrs/solver.rs`, `ontology.rs`, `renderer.rs`, `runtime.rs`, `svg.rs`, `widgets.rs`.

---

## 2. Verified Current-State Assessment

All claims below were verified by direct code inspection and build/test execution on 2026-09-22.

### 2.1 Health metrics

| Metric | Value | Assessment |
| :--- | :--- | :--- |
| Source size | 30,119 LOC / ~120 files | Substantive, not a toy |
| Unit tests | 119 (`#[test]`, co-located) | **118 pass, 1 fails** |
| Binaries | 40 in `src/bin/` | Many are demos; some duplicate effort |
| Benchmarks | 2 criterion benches (`benches/`) | Radiosity, SPH, CCD, vector ops |
| `cargo build --lib` | ✅ Compiles | 22 warnings (unused imports/variables) |
| `cargo build` (default) | ❌ **Fails** | `rgpum_test` bin, feature-gating bug |
| `cargo test --lib` | ⚠️ 118/119 | `ocrs::adaptive::test_performance_budget` |
| Git history | **2 commits total** | No incremental history; entire project is one blob + license |
| `TODO/FIXME` | 17 | Low count, but see §2.3 |
| `unimplemented!()` / `todo!()` | 0 | — |

### 2.2 Corrections to the feature matrix (`RUGID-ass.txt`)

The matrix is directionally accurate but must be corrected before being used for planning:

| # | Matrix claim | Verified reality |
| :--- | :--- | :--- |
| 13, 78 | Telemetry has "spans, levels, metrics"; profiler "spans implemented" | Telemetry (`src/telemetry/mod.rs`, 347 LOC) has **events, 5 log levels, 7 categories, circular buffer**. **No spans, no metrics.** |
| 25 | "SpatialHasher effectively culls off-screen MegaCells" | `src/spatial.rs` (48 LOC) packs x/y grid coords into `MegaCellId` for **tick scheduling**, not frustum culling. No frustum culling exists. |
| 65 | Focus management ❌ | **Implemented**: `src/input.rs` has `register_focusable`, Tab-order `FocusNext`/`FocusPrev`. Needs tests and polish, not implementation. |
| 67 | Tooltips ❌ | **Implemented**: `TooltipProjector` in `src/widgets.rs` + `Runtime::register_tooltip` (`src/runtime.rs:273`). |
| 76 | "Visual Scene Editor ❌ — code only" | **Skeletal but present**: `src/ironbeam/` is 2,142 LOC (ide/viewer/shell/phenomena/formats/widgets). `ironbeam/ide/lsp.rs`, `project_tree.rs`, `terminal.rs` are 5-line stubs; editor is 123 LOC with a syntax-highlighting TODO. |
| 36 | Video playback ❌ | `src/bin/media_player.rs` + `media_player_headless.rs` exist — unverified whether real decoding or UI mock; treat as ⚠️ until audited. |
| — | `rdf_render` binary "missing" (AGENTS.md) | **Exists** (`src/bin/rdf_render.rs`, 47 LOC). `demos/run_quick_demo.sh` works. AGENTS.md is outdated. |

### 2.3 Confirmed strengths (verified)

- **Ontology** (`src/ontology.rs`, ~1,650 lines incl. tests): relativistic `LocalSpace`/`OntologicalNode`, `RelativeSize::{Percent,Flex,Aspect}` — the layout differentiator is real.
- **Asset registry** (`src/assets/registry.rs`): SHA-256 content addressing, dependency/dependent graphs, cascade reload — a strong, unusual foundation.
- **Physics breadth**: real SPH (`fluid.rs`, rayon-parallel), ISA atmosphere, aero/wind, analytic sphere–plane CCD, radiosity solver.
- **RDF pipeline** (`src/rdf/`, 1,998 LOC, 19 tests): full lexer→parser→AST→converter chain matching the design guide, including animation modifiers.
- **Morphism** (`src/morphism.rs`): path interpolation with easing — the vector-native answer to skeletal animation.

### 2.4 Confirmed weaknesses (verified)

- **Platforms are thin**: `tui.rs`, `bitmap.rs`, `gpu_proxy.rs` in `src/platforms/` are 35–38-line mocks. Only the wgpu platform (795 LOC) is real.
- **RGPUM is a PoC**: hand-rolled Vulkan FFI behind the `rgpum` feature, with TODO event handling; and its test bin breaks the default build.
- **Collision is narrow**: analytic CCD covers sphere–plane only (`src/physics/collision.rs:42` TODO for sphere–sphere).
- **Radiosity incomplete**: no visibility/occlusion factor between patches (`src/physics/radiosity.rs:96` TODO).
- **No incremental git history**: the project has one "Initial commit" containing ~30k LOC. Code review, bisection, and attribution are all impossible today.
- **ironbeam is aspiration-shaped**: ~40% of its modules are stubs under 50 lines.

---

## 3. Strategic Positioning

Before the roadmap: what RUGID should **not** try to be.

| Matrix item | Directive decision | Rationale |
| :--- | :--- | :--- |
| 16 Forward/Deferred renderer | **Reject parity** (keep 🤷) | SVG vector rasterization is the product. Competing on GPU triangle rasterization loses against Bevy/UE by default. |
| 33 Shader graph | **Defer to Phase 5** | Meaningless until the material model stabilizes (Phase 2). |
| 39 VR/AR | **Defer / decision gate** | OpenXR + wgpu is viable but the UI layout model needs a stereoscopic projection story first. Revisit after Phase 3. |
| 47 Destruction | **Defer** | Voronoi fracture of vector geometry is research-grade; no demand signal. |
| 86–89 Networking | **Phase 5 with hard gate** | Requires determinism (Phase 2 exit) + a networking owner. Do not start early. |
| 80 VCS integration | **Reject** | Git already works; a GUI wrapper is a Phase 5+ nice-to-have at best. |
| 30/52 Skeletal + Ragdoll | **Repurpose** | Morphism + XPBD constraints (Phases 2–4) cover 80% of the use case with vector-native tech. Full bone hierarchies only if a customer demands FBX/glTF skinning (note: `ironbeam/formats/gltf.rs` is a stub today). |

Everything else in the ❌/⚠️ set is scheduled below.

---

## 4. Phase 0 — Engineering Hygiene & Build Integrity (Weeks 0–3)

**Objective:** Every clone builds clean, tests are green, history is real, and CI guards regressions. This phase is a hard prerequisite: no feature work merges until it exits.

| ID | Task | Detail | Effort |
| :--- | :--- | :--- | :--- |
| P0-1 | Fix default build | Add `required-features = ["rgpum"]` to the `[[bin]]` entry for `rgpum_test` in `Cargo.toml` (or `#[cfg(feature)]`-gate its `main`). Add a `--no-default-features` + headless CI job so feature combinations never silently rot again. | 0.5 pw |
| P0-2 | Stabilize the tree | Land or revert the 8 modified files. Fix `ocrs::adaptive.rs:164` (`fits_budget(0)` assertion) — if the budget semantics changed in the uncommitted solver work, update the test to the new contract. | 1 pw |
| P0-3 | Warning burn-down | Eliminate all 22 warnings (mostly unused imports in `physics/aero.rs`, `physics/optics.rs`, `ocrs/projection.rs`, `platforms/wgpu_platform.rs`, `renderer.rs:591`, `ontology.rs:1030`). Enable `#![deny(warnings)]` in CI only (not the crate root, to keep local iteration fluid). | 0.5 pw |
| P0-4 | Git history going forward | From Phase 0 exit, all work lands via small, reviewable commits. Backfill is optional and low-value; do not spend more than 1 day on it. | ongoing |
| P0-5 | CI pipeline | GitHub Actions: `fmt --check`, `clippy -- -D warnings`, `cargo test --lib` (stable + beta), feature matrix (`--no-default-features`, `rgpum`), `cargo bench --no-run`. Cache with `swatinem/rust-cache`. | 1 pw |
| P0-6 | Repo metadata | `Cargo.toml` description/license/repository fields; README quickstart that matches reality; **update `AGENTS.md`** (the `rdf_render` "missing binary" note is false). | 0.5 pw |
| P0-7 | MSRV & toolchain | Pin `rust-toolchain.toml` and declare MSRV (edition 2024 implies ≥1.85). | 0.1 pw |

**Exit criteria:** `cargo build`, `cargo test`, `cargo clippy -- -D warnings` all green on stable, beta, and `--no-default-features`; CI blocking on PRs; ≥10 real commits landed post-Phase-0.

---

## 5. Phase 1 — Core Runtime Hardening (Months 1–3)

**Objective:** Turn the strongest infrastructure (telemetry, assets, ontology, events) from "implemented" into "trustworthy," and make the two big architecture decisions (ECS, scripting) with written ADRs.

### 5.1 Workstream: Telemetry completion (matrix #13, #78, #82)

| ID | Task | Detail | Effort |
| :--- | :--- | :--- | :--- |
| P1-1 | Span API | Add `SpanGuard`-based hierarchical spans to `src/telemetry/mod.rs` (enter/exit timestamps on a thread-local stack, emitted to the circular buffer). Instrument `render_tick`, physics step, RDF load, asset fetch. Spans are the substrate for the Phase 3 profiler UI. | 1 pw |
| P1-2 | Metrics counters | Gauge/counter registry (`telemetry::metric("sph.particles", …)`), aggregated per tick into the buffer. | 0.5 pw |
| P1-3 | Crash reporting | `std::panic::set_hook` that flushes the telemetry ring buffer to `rugid_crash_<ts>.log` before abort; include RDF scene hash and tick number. This is matrix #82 and is cheap because telemetry already exists. | 0.5 pw |

### 5.2 Workstream: Asset pipeline (matrix #4, #79 groundwork)

| ID | Task | Detail | Effort |
| :--- | :--- | :--- | :--- |
| P1-4 | Async loader | `AssetRegistry::load_async` on a dedicated thread pool (reuse rayon's pool with `spawn`; never block `render_tick`). Promise-style handle with Beacon-based invalidation so cells re-render when the asset lands. | 2 pw |
| P1-5 | File-watcher hot reload | Add `notify` dependency; wire change events → registry's existing cascade-reload graph → signal affected cells. RDF scene files reload through the same path, replacing the bespoke RDF reload. | 1 pw |

### 5.3 Workstream: Architecture decisions (matrix #1, #8, #93)

| ID | Task | Detail | Effort |
| :--- | :--- | :--- | :--- |
| P1-6 | ADR-001: ECS | Decide: (a) document the `CellId`-map model as the intentional architecture with data-oriented layout guidelines, or (b) adopt `hecs` behind the existing `Cell` facade. Recommendation: **(a)** — the ontology's relativistic graph semantics don't map onto archetype tables, and a façade migration later is still possible. | 0.5 pw (decision) |
| P1-7 | ADR-002: Scripting | Embed `rhai` (pure Rust, sandboxable, good embed API) as the logic layer driving Signals/Beacons from RDF event handlers. Lua (`mlua`) is the alternative if ecosystem familiarity matters more than build simplicity. Output: `rdf_script` prototype + ADR. | 3 pw |
| P1-8 | Formal FSM (matrix #93) | A small `fsm!` state-machine helper over Beacons (`on_enter/on_exit/on(signal)`), since Signals already provide transitions. | 1 pw |

### 5.4 Workstream: GI completion (matrix #20)

| ID | Task | Detail | Effort |
| :--- | :--- | :--- | :--- |
| P1-9 | Radiosity occlusion | Close the `radiosity.rs:96` TODO: per-patch-pair visibility via the existing shadow-volume machinery (`physics/shadows.rs`) or analytic ray tests; then inject results into gradient generation in `optics.rs`. Extend `benches/performance.rs` radiosity benches to cover the occlusion cost. | 2 pw |

**Exit criteria:** Telemetry spans visible in a headless dump; async asset load demo (texture lands mid-frame without stalling render); ADR-001/002 accepted; radiosity occlusion benchmarked with before/after numbers; CI green.

---

## 6. Phase 2 — Physics Depth & Determinism (Months 3–6)

**Objective:** Physics is RUGID's second differentiator ("Vector Physics Engine" per the matrix conclusion). This phase makes it deep and deterministic.

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P2-1 | 43, 54 | Broaden CCD | Sphere–sphere analytic CCD (close `collision.rs:42` TODO), then capsule–plane and capsule–capsule. Keep the zero-tunneling invariant; property-test with random trajectories at high velocity. | 2 pw |
| P2-2 | 41 | Rotational rigid bodies | Angular velocity/inertia tensors in `physics/core.rs`, impulse-based contact resolution replacing the current Euler integrator in `system.rs`. | 3 pw |
| P2-3 | 45 | XPBD constraints | Position-based dynamics solver with distance, hinge, and spring constraints — one solver serves joints, ragdolls (P4), and soft-body approximation (P4). | 4 pw |
| P2-4 | 44 | Physics materials | Formalize friction/restitution pairs on `Material`; combine via restitution-mixing rules; RDF syntax `element:::material[rubber]`. | 1 pw |
| P2-5 | 51 | Trigger zones | First-class trigger volumes over the `SpatialHasher`: enter/exit/Stay signals, `is_trigger` flag on ontology nodes. | 1 pw |
| P2-6 | 55 | Determinism audit | Fixed-point (e.g., `fixed` crate) or strictly-ordered-FP simulation mode; lockstep replay test: record 10k ticks of the Cornell box demo, replay, assert bit-identical state hashes. This is the gate for any future networking. | 3 pw |
| P2-7 | 21 | CPU post-processing | Analytic bloom (brightness-pass on gradients + Gaussian spread) and chromatic aberration in `src/filters.rs` (module exists — verify current scope) as SVG filter primitives. Benchmark at 4K. | 2 pw |
| P2-8 | 17 | Vector PBR-style materials | Not texture maps — a **gradient-space material model**: roughness → gradient blur radius, metalness → specular highlight strength, emission → bloom contribution. Extends `optics.rs::RenderFace`. Document as the RUGID-native PBR analogue. | 3 pw |
| P2-9 | 74 | Clipboard | `arboard` integration; copy/paste for text cells and the ironbeam editor. | 0.5 pw |

**Exit criteria:** Cornell box + new joint demo running under replay-verified determinism; CCD property tests merged; bloom visible in headless SVG output; XPBD solver benchmarked against current Euler baseline.

---

## 7. Phase 3 — Platform Parity: Audio, Input, WASM (Months 5–9, overlaps Phase 2)

**Objective:** Close the "standard engine" gaps users feel immediately: sound, gamepad, clipboard-adjacent UX, and the browser.

### 7.1 Audio engine (matrix #56–59) — new module `src/audio/`

Dependency: `kira` (mixer/buses/effects, pure Rust) on top of `cpal` output. Total ~6 pw.

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P3-1 | 56 | 2D playback | `AudioManager` singleton; RDF `element:::sound[src]` nodes resolved through the asset registry (hot-reload free). | 1.5 pw |
| P3-2 | 57 | Spatial audio | Distance attenuation + stereo panning from ontology world position; HRTF deferred (decision gate: only if a 3D demo needs it). | 1.5 pw |
| P3-3 | 58 | Mixing buses | Master/music/sfx buses, per-bus volume/effects, telemetry metric hooks. | 1 pw |
| P3-4 | 59 | Procedural audio | Wavetable synth behind the same bus API; wind (`aero.rs`) and fluid (`fluid.rs`) signals drive synthesis parameters. This is a RUGID-flavored feature — physics-driven sound. | 2 pw |

### 7.2 Input & UI parity (matrix #63, #66, #71, plus hardening #65, #67)

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P3-5 | 71 | Gamepad | `gilrs` (or winit gamepad events) → existing `InputDispatcher` signal mapping; RDF `input:::gamepad[a]` binding syntax. | 1.5 pw |
| P3-6 | 66 | Drag & drop | Pointer-capture drag protocol + `drop_target` ontology role; file drops via `winit` paths into the asset registry. | 1.5 pw |
| P3-7 | 63 | Rich text | `cosmic-text` (already pulled in transitively by glyphon) for styled runs, embedded icons, and bi-directional text in text cells. | 2 pw |
| P3-8 | 65, 67 | Harden focus + tooltips | They exist but are untested: add unit tests for tab-order cycling and tooltip show/hide timing; visual focus indicator spec in RDF theme. | 1 pw |

### 7.3 Web platform (matrix #99)

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P3-9 | 99 | WASM build | `wasm32-unknown-unknown` target: cfg-gate `cpal`/file-watcher/threads (wasm-bindgen-rayon + SharedArrayBuffer or single-thread fallback), headless renderer → canvas via `resvg`-style raster or direct SVG DOM injection (the SVG pipeline is a natural fit for the DOM). CI job builds the target. | 3 pw |

### 7.4 Rendering scale (matrix #23, #32, #38)

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P3-10 | 23 | LOD | Morphism-based: `ShapeType` simplification (path decimation) by screen-space size tier; ties into OCRS adaptive weighting (`ocrs/adaptive.rs`) which already reasons about detail budgets. | 2 pw |
| P3-11 | 32 | Texture/asset streaming | Prioritized fetch queue in the P1-4 async loader; distant assets load at low priority; registry eviction policy. | 1.5 pw |
| P3-12 | 38 | Multi-monitor | Multi-window ontology roots; per-window `render_tick`; winit already supports it — the work is in `runtime.rs` assumptions. | 1.5 pw |

**Exit criteria:** Audio demo (cornell box with physics-driven impact sounds) on 2 platforms; gamepad navigating a UI; WASM build job green in CI; LOD demo at 10k shapes.

---

## 8. Phase 4 — Tooling, Editor & Advanced Simulation (Months 8–14)

**Objective:** Ship the ironbeam IDE as a real product and complete the advanced physics tier. This is the longest phase and the one most visible to users.

### 8.1 ironbeam IDE (matrix #76–79, #83–85)

The skeleton exists (`src/ironbeam/`, 2,142 LOC); the work is filling it in, not greenfield.

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P4-1 | 76 | Visual scene editor | Viewer modules exist (`viewer/camera.rs`, `gizmo.rs`, `selection.rs`, `staging.rs`). Build: RDF-outline tree panel → viewport gizmo manipulation → bidirectional RDF text sync (edit text ⇄ move gizmo). Base it on the existing 123-LOC editor. | 6 pw |
| P4-2 | 85 | Prefab system | Formalize RDF reuse: named `define` blocks with parameter override, instantiated as ontology subtrees; registry hashes prefab content for change propagation. Extends the asset dependency graph. | 2 pw |
| P4-3 | 84 | Animation timeline | UI over the RDF animation modifiers (`{~}`, `[~]`) and `temporal.rs` delta programs: keyframe track editor, easing curve picker (morphism easing already exists). | 3 pw |
| P4-4 | 83 | Asset browser | Grid over the content-addressable registry with preview (SVG thumbnails are nearly free — render headless at small size). | 2 pw |
| P4-5 | 77, 78 | Visual debugger + profiler | Telemetry spans (P1-1) rendered as in-app overlay: tick-time flamegraph, signal/beacon graph inspector, physics-body state inspector. This replaces `println!` debugging. | 3 pw |
| P4-6 | — | Editor stubs | `lsp.rs`, `project_tree.rs`, `terminal.rs` are 5-line stubs. Either implement (RDF language server is tractable given the clean lexer/parser split) or delete. Prefer implementing project_tree (1 pw) and deferring LSP. | 1–4 pw |

### 8.2 Advanced physics (matrix #42, #48, #52, triggers done in P2)

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P4-7 | 42 | Soft bodies | XPBD (P2-3) with distance+volume constraints over shape tessellations; cloth first (plane tessellation), jelly second. | 4 pw |
| P4-8 | 48 | Vehicle physics | Raycast vehicle: suspension springs (XPBD distance constraints), wheel raycasts against the CCD collision world. The existing `f1_car_demo.rs`/`mercedes_w14_demo.rs` binaries become real demos. | 3 pw |
| P4-9 | 52 | Ragdolls | Constraint chains (P2-3) driven by morphology interpolation — the RUGID-native ragdoll; classic bone-hierarchy ragdolls deferred per §3. | 2 pw |
| P4-10 | 26 | Particle systems | Emitter ontology nodes → SPH-style advection at low density OR simple ballistic particles with aero/wind coupling; rendered as SVG groups with LOD-aware counts. | 2 pw |

### 8.3 i18n, a11y, mobile groundwork (matrix #68–70, #100)

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P4-11 | 68 | i18n | `fluent` string tables; RDF `text[key]` indirection resolved at render. | 1.5 pw |
| P4-12 | 69 | a11y (basic) | Focus order + labels → platform accessibility APIs where available; full screen-reader support is explicitly out of scope until a demand signal exists. | 2 pw |
| P4-13 | 70, 100 | Touch + virtual keyboard | Gesture layer (pinch/swipe, #73) + on-screen keyboard widget; iOS/Android via winit + softbuffer/wgpu surface. Ship one mobile demo. | 3 pw |
| P4-14 | 72 | Haptics | Gamepad rumble via gilrs `Effect`; mobile vibration behind the same signal. | 0.5 pw |

**Exit criteria:** ironbeam edits a demo.rdf scene end-to-end (visual edit → saved RDF → reloads identically); soft-body cloth demo at 60fps; vehicle demo; i18n switching live.

---

## 9. Phase 5 — Ecosystem & Frontier (Months 14–18+)

**Objective:** Everything that requires prior phases to be solid, plus the AI tier. Each item here has a go/no-go gate.

| ID | Matrix | Task | Detail | Effort |
| :--- | :--- | :--- | :--- | :--- |
| P5-1 | 7 | Plugin architecture | Decide DLL (`cdylib` + stable ABI via `abi_stable`) vs WASM plugins (sandboxed, runs on web). Recommendation: **WASM plugins** — one implementation covers native + web, and the sandbox story is free. Gate: ADR after Phase 3 WASM experience. | 4 pw |
| P5-2 | 86–89 | Networking | **Hard gate:** P2-6 determinism must be exit-clean. Scope: state-delta replication over QUIC (`quinn`), client prediction for the physics step, headless dedicated server binary (the headless platform already renders SVG — server-side frame generation is plausible). Matchmaking explicitly out of scope (no demand). | 8 pw |
| P5-3 | 90–92, 94–95 | AI tier | Steering behaviors (seek/flee/flocking) on the FSM helper (P1-8) first; then navmesh (recast-style rasterization of the collision world) and behavior trees; blackboard = signal-addressable shared state (fits Beacons naturally). Perception = raycast + SpatialHasher queries — both already exist. | 6 pw |
| P5-4 | 27 | Terrain | Analytic heightfield as ontology geometry (tessellated on LOD tier, P3-10) — skip splatmaps; texture projection conflicts with the vector model. | 3 pw |
| P5-5 | 29 | Decals | Projected gradient patches conforming to face tessellation; implement only if P4 tooling surfaces demand. | 2 pw |
| P5-6 | 31–36 area | Video | Real decode via `symphonia` (audio, already needed) + a codec; gate on whether media_player binaries are currently real — audit first (see §2.2). | 3 pw |
| P5-7 | 39 | VR/AR | OpenXR + wgpu stereo rendering; gate on Phase 3 multi-window/viewport work. | 6 pw |

---

## 10. Cross-Cutting Workstreams (continuous)

1. **Documentation in lockstep** (repo policy): every phase updates `docs/book/`, `AGENTS.md`, and `RUNNING.md` in the same PR as the code. The matrix itself must be regenerated at each phase exit (several of its claims are already stale).
2. **Demo-driven verification:** each phase ships/updates one demo in `demos/rdf/` exercising its features; demos are the acceptance test for headless CI (render frames, hash SVG output).
3. **Benchmark regression CI:** `criterion` with baselines stored in-repo; alert on >10% regression in radiosity/SPH/CCD benches.
4. **Dependency discipline:** new deps this directive adds — `notify`, `rhai`, `kira`, `cpal`, `gilrs`, `arboard`, `fixed`, `fluent`, `quinn`, `abi_stable`(if DLL) — each needs a one-paragraph justification in its PR. Run the `amber` dependency audit before each phase exit.
5. **Governance:** ADRs in `docs/adr/` for P1-6, P1-7, P5-1, P5-2; monthly roadmap review against the exit criteria below.

---

## 11. Risk Register

| Risk | Likelihood | Impact | Mitigation |
| :--- | :--- | :--- | :--- |
| ironbeam IDE scope creep (editor ambitions eat the roadmap) | High | High | P4 gates: viewer/bidirectional-sync first; LSP/terminal explicitly deferred |
| Determinism audit (P2-6) finds pervasive FP drift | Medium | High (blocks networking) | Scope determinism to the lockstep replay test, not absolute cross-platform bit-parity; document the guarantee precisely |
| wgpu 0.19 ages badly (ecosystem moves to 0.2x) | Medium | Medium | Phase 0.5: upgrade wgpu before building more platform features; pin in CI matrix |
| Single-maintainer bus factor (30k LOC, 2 commits, no history) | High | High | Phase 0 commit discipline; module ownership notes in `AGENTS.md`; docs/book as onboarding path |
| Audio/latency on WASM (cpal unusable in browser) | High | Medium | P3-9 uses WebAudio bridge behind a `AudioBackend` trait; trait defined in P3-1 |
| Plugin ABI churn | Medium | Medium | WASM-first decision (P5-1) removes native ABI concerns |
| Test failure in ocrs signals deeper solver regression | Medium | Medium | P0-2 investigates before any ocrs-dependent Phase 4/5 work |

---

## 12. Scorecard Projection

Estimated matrix score if phases exit cleanly (same 100-point rubric):

| Phase exit | Score | What moved |
| :--- | :--- | :--- |
| Today (verified) | ~48 | — |
| P0 | ~48 | No features; credibility of everything below |
| P1 | ~52 | Telemetry spans/crash (#13, #82), hot reload (#79), GI completion (#20), FSM (#93) |
| P2 | ~57 | Constraints (#45), materials (#44), triggers (#51), determinism (#55), post-fx (#21), vector PBR (#17), clipboard (#74) |
| P3 | ~64 | Audio 2D/3D/mix/proc (#56–59), gamepad (#71), DnD (#66), rich text (#63), WASM (#99), LOD (#23), streaming (#32), multi-monitor (#38) |
| P4 | ~74 | Editor (#76), prefabs (#85), timeline (#84), asset browser (#83), profiler/debugger (#77, #78), soft body (#42), vehicle (#48), ragdoll (#52), particles (#26), i18n (#68), a11y-basic (#69), touch/vkbd (#70, #100), haptics (#72), gestures (#73) |
| P5 | ~82 | Plugins (#7), networking (#86–89), AI (#90–95), terrain (#27), video (#36); VR/decals/destruction remain intentionally open |

The remaining ~18 points are the items this directive recommends rejecting or deferring indefinitely (VR, destruction, matchmaking, VCS integration, skeletal parity) — closing to ~85–90 on the "RUGID-flavored" rubric is the realistic ceiling, and the correct goal.

---

## Appendix A — Phase exit checklist template

```
Phase: ___
Date: ___
Build/test/clippy green on: [ ] stable [ ] beta [ ] no-default-features [ ] wasm32 (from P3)
New demos: ___
Benchmarks vs baseline: ___
Docs updated: [ ] book [ ] AGENTS.md [ ] RUNNING.md [ ] matrix regenerated
ADRs accepted: ___
```

## Appendix B — Matrix item → task index

Items not listed map to the phase tables above by their matrix number appearing in the "Matrix" column. Intentionally unscheduled: #80 (VCS — rejected), #47 (destruction — deferred), #88 (matchmaking — out of scope), #39 (VR — gated), #60 (voice chat — out of scope; no demand signal).
