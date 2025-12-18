# toml-and-jerry — Polyglot Configuration Validator

`toml-and-jerry` is an enterprise‑grade CLI that enforces a single, authoritative JSON Schema (or OpenAPI component schema) across **JSON, TOML, YAML, and HCL** configuration files. Ship reproducible builds, catch drift early, and tame config sprawl, all with one tiny binary.

---

## Why toml-and-jerry?

* **Polyglot by design** – validate four major config “languages” in a single pass.
* **Schema‑first** – uses standards compliant JSON Schema 2020‑12 for maximum interoperability.
* **CI‑ready** – simple non‑zero exit codes, streaming output, and `--json`/`--sarif` modes for automated pipelines.
* **Blazing‑fast** – parallel file walking (via `rayon`) and schema caching for monorepos.
* **Extendable** – library crate under the hood for editor plugins or custom rules.

---

## Quick start

```bash
# Install the latest release
cargo install toml-and-jerry --locked

# Validate a single file
toml-and-jerry check config/settings.toml --schema schemas/settings.schema.json

# Validate an entire repo in CI (JSON output for tooling)
toml-and-jerry check ./configs/**/* --schema schemas/settings.schema.json \
      --format json
```

---

## Command overview

| Command    | Purpose                                                        |
| ---------- | -------------------------------------------------------------- |
| `check`    | Validate one or more config files against a schema.            |
| `scaffold` | Generate a starter JSON Schema from Rust structs (`schemars`). |

Run `toml-and-jerry --help` for full flag details.

---

## Roadmap

* **YAML 1.2 line‑number diagnostics**
* **`diff` mode** – highlight schema breaking changes between two config versions
* **Kubernetes manifest presets**
* **VS Code extension** (on‑save validation)

Contributions are welcome—see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

Distributed under the MIT License. See `LICENSE` for details.

---

## Security & support

If you discover a security vulnerability, please **email coryjanowski\@gmail.com**. I follow responsible disclosure and aim to patch within **72 hours** for critical issues.
