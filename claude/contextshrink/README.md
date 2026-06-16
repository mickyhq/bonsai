# ContextShrink Claude Code Plugin

Claude Code plugin for generating token-budgeted ContextShrink XML before broad repo analysis.

Test from the repo root:

```sh
claude --plugin-dir ./claude/contextshrink
```

Use in Claude Code:

```text
/contextshrink:contextshrink
```

The skill writes `/tmp/contextshrink.xml`, then Claude reads it before answering.
