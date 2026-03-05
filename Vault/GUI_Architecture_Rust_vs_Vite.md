# GUI Architecture Rust vs Vite

For a large Rust-based agent simulation, prefer a split architecture:

- Rust simulation core
- API boundary (local IPC or HTTP/WebSocket)
- Vite frontend for visualization and controls

Why:

- Faster UI iteration and richer charting/interaction ecosystem in web stack.
- Cleaner separation between deterministic simulation engine and presentation layer.
- Easier remote/headless runs with a reusable API.

Use a pure Rust GUI only if you need:

- Fully native desktop packaging with minimal web tooling.
- Tight GPU/native integration that is hard to expose through an API.
- A very small team already deep in Rust UI frameworks.
