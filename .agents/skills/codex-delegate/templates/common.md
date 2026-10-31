You are working in the Rust workspace at the current directory (GPUI port of Material Design 3).
Read AGENTS.md, docs/agents/headless.md, docs/agents/architecture.md, docs/agents/a11y.md, docs/agents/component-workflow.md,
docs/agents/gpui-notes.md and docs/agents/resources.md first and follow every rule there (no new dependencies,
no build.rs, forbid(unsafe_code), deny(missing_docs), no "Material" in type names, no std::thread/Mutex/blocking
in render paths — must work on wasm32, ElementId + a11y role/name/state on interactive elements, token values only
from primary sources with provenance appended to docs/TOKENS_SOURCE.md and Apache-2.0 attribution headers on
ported code).
gpui is pinned at rev 37faef8; its source is in
~/.cargo/git/checkouts/zed-a70e2ad075855582/37faef8/crates/gpui/src — grep it to confirm APIs instead of guessing.
Compose reference sources (Apache-2.0) are the docs/references/androidx submodule; Material 3 is under
compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ (token files in its tokens/ subfolder).
Behaviour lives in crates/gpui-headless/src and takes no visual values: control, focus, a11y, keys, press,
scroll (NestedScroll, ScrollScope, reveal_on_focus), scrollbar.
Looks live in crates/md3-components/src: state_layer.rs / focus.rs (.corners()), motion.rs (motion::color),
scroll.rs (content), scrollbar.rs.
Motion: Theme::spring(MotionToken) -> gpui::SpringConfig with AnimationExt::with_spring; every motion must respect
cx.reduce_motion() (gpui's with_spring / with_animation already do; custom timers must check it).

You are already running inside `nix develop` (toolchain on PATH; the Nix daemon is NOT reachable from the
sandbox): run cargo directly, never `nix develop`. The sandbox cannot open a window (no Wayland/X11), so do not
try to run the gallery; Claude checks the screen afterwards.
Do NOT commit; leave changes in the working tree. Do not touch .envrc, nix toolchain pins, rust-toolchain.toml,
Cargo profile settings or CI workflows unless the task says so.

Done criteria (run them all; they must pass):

- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- cargo build -p md3-components --target wasm32-unknown-unknown
- cargo xtask check-tokens
- gallery shows every new state/sample; docs/specs/\*.md, docs/guide.md, README and docs/a11y-audit.md are updated
  where relevant. Comments and docs are in Japanese — match the existing style.
  Finish with a concise summary per item, listing token values you used with their source, and anything you could
  not verify.
