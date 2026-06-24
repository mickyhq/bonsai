# TODO

## Token Cost Improvements

- Add config allowlist tuning for common files: `package.json`, GitHub workflows, `Cargo.toml`, plugin manifests, VS Code manifests.
- Add `--map-only-under <tokens>` fallback for tiny budgets: output project map and directory summaries only.
- Add golden token-cost fixtures to catch regressions for large tables, large configs, import-heavy code, and many-file repos.
