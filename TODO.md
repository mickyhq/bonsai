# TODO

## Highest Value

- [x] Add `--stats` to show true token savings:
  - `raw_tokens`
  - `shrunk_tokens`
  - `tokens_saved`
  - `saving_percent`
  - `files_scanned`
- [x] Add `--summary` output after each run:
  - output file path
  - files included
  - selected compression level
  - output tokens vs budget

## Tests

- Add parser snapshot tests for:
  - Rust skeleton output
  - TypeScript skeleton output
  - Python skeleton output
  - tree map output
- Add budget tests for downgrade behavior:
  - full to skeleton
  - skeleton to tree map
  - stable file ordering
- Add formatter tests for XML escaping.
- Add walker tests for ignored files and supported extensions.

## VS Code Extension

- Add a clearer command for end-to-end use:
  - `ContextShrink: Generate and Ask`
- Improve command names so they are not Copilot-only.
- Show better success output:
  - generated file path
  - approximate token count
  - next step for Copilot, ChatGPT, or Codex in VS Code
- Make binary discovery simpler and more reliable.

## Language And File Support

- Add more code languages:
  - `.go`
  - `.java`
  - `.cs`
  - `.swift`
  - `.kt`
- Add useful project context files:
  - `.md`
  - `.json`
  - `.yaml`
  - `.yml`
  - `.toml`
- Treat docs and config differently from source code.

## Install Story

- Support install with:

```sh
cargo install --path .
```

- Update plugins and VS Code extension to prefer the installed `contextshrink` binary.
- Add one clear smoke test for each integration:
  - CLI
  - Codex plugin
  - Claude Code plugin
  - VS Code extension

## Plugins

- Make Codex and Claude instructions more explicit about when ContextShrink must run.
- Add expected behavior examples:
  - ask for whole project summary
  - see ContextShrink command execute
  - inspect `/tmp/contextshrink.xml`
- Add marketplace/install docs for Claude Code if publishing.

## CLI Quality

- Add `--include` and `--exclude` flags.
- Add `--respect-gitignore` toggle.
- Add `--print-files` to show selected files.
- Add `--fail-on-empty` for automation.
- Add clearer errors when no supported files are found.

## Output Quality

- Add metadata to XML:
  - generated time
  - repo root
  - token budget
  - compression level
  - file count
- Add per-file token counts.
- Add a short project map section before file contents.
- Consider JSON output as an alternative to XML.
