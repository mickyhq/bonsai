For repo-wide analysis, first run ContextShrink:

```sh
target/release/contextshrink . --max-tokens 12000 --level 2 --output file --output-file /tmp/contextshrink.xml
```

Then inspect `/tmp/contextshrink.xml` before answering.

Repo-wide analysis includes architecture review, full project summary, broad bug hunting, onboarding, and questions that need many files.
