For repo-wide analysis, first run ContextShrink. This is required for full project summaries, architecture review, onboarding, broad bug hunting, and any question that needs many files:

```sh
target/release/contextshrink . --max-tokens 12000 --level 2 --output file --output-file /tmp/contextshrink.xml
```

Then inspect `/tmp/contextshrink.xml` before answering.
