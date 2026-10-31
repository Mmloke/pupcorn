# Pupcorn

Desktop bug-bounty assistant: AI-driven recon/scanning engine in Rust.

## Status

- `crates/pupcorn-core`: tool trait + registry + SQLite schema + 7 modules + Repo (CRUD) + audit log + screenshot (Firefox headless backend). Builds and runs.
- `crates/pupcorn-ai`: LLM client trait (OpenAI-compatible + Mock), Planner, Executor with approval gate, Agent (multi-turn loop), ResearcherProfile (feedback weighting). Builds and runs.
- `crates/pupcorn-gui`: egui desktop UI (tool list, goal input, agent run, log). Builds.

## Build

```bash
source "$HOME/.cargo/env"
cargo build --workspace
cargo run -p pupcorn-core --example demo
cargo run -p pupcorn-core --example repo
cargo run -p pupcorn-ai --example demo
cargo run -p pupcorn-ai --example agent
cargo run -p pupcorn-gui
```

## Architecture

- `tool::Tool` trait: unified `run(target, options) -> ToolOutput`. `is_intrusive()` flags active/exploitation actions requiring explicit approval.
- `engine::ToolEngine`: registry + `run_tool(name, target, options, approval)`. Intrusive tools blocked without `Approval { granted: true }`.
- `db::Database`: SQLite with targets, sessions, findings, screenshots, decisions, feedback, audit_log.
- `repo::Repo`: CRUD for targets/sessions/findings/screenshots + audit logging.
- `screenshot`: Screenshotter trait + FirefoxHeadless + Noop backends.
- `pupcorn-ai`: Planner (LLM -> JSON Plan), Executor (runs plan, enforces approval), Agent (multi-turn loop), ResearcherProfile (feedback weighting).

## Notes

- Desktop UI uses egui (pure Rust) instead of Tauri (requires webkit2gtk dev libs not available in this env).
- LLM provider configurable via OPENAI_BASE_URL / OPENAI_API_KEY / OPENAI_MODEL (works with OpenAI and Ollama).