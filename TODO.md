# TODO

## Token Cost Improvements

- Add `--files-under <tokens>` / `--content-under <tokens>` fallback: keep project map plus bodies only for the highest-priority files that fit; drop bodies for the rest.
- Add repeated generated-boilerplate detection: collapse files with near-identical tree maps/content into one representative entry plus a count.
- Add symbol-aware tree-map budgets: cap per-directory and per-file tree-map lines, keeping exported/public symbols before private/internal symbols.
- Add dependency manifest thinning: for huge `package.json`, lock-like manifests, and config arrays, keep section names plus first/tail entries and total omitted count.
- Add Markdown section budgets: keep headings and first/tail lines per section, with stricter caps for changelogs, release notes, and API reference pages.
- Add output-overhead minimizer mode: force compact project map, omit token counts, omit metadata extras, and prefer text output when budget is very small.
- Add `--changed-context-budget <tokens>`: when incremental mode is active, reserve most budget for changed files and reduce unchanged files to project-map entries.
