# Docs style guide

These rules apply to every page under `website/src/content/docs/`. Pages that follow them well:
`rules/overview.mdx`, `cli/check.mdx`, `formatting.mdx` and `custom-rules/overview.mdx`.

## Write for users

- Document what a user does, sees and must decide: commands, flags, config keys, defaults, limits,
  Rule codes and messages, output, exit codes, and the fix for each problem.
- Leave out how SQLClean works inside: cache files and their formats, hashing, parallelism, the
  custom Rule host protocol and internal structures. Include a mechanism only when the user must act
  on it, and then say what to do (for example, which flag forces a fresh check).
- Never document behaviour you haven't checked in the source or with the real `sqlclean` command.
  Paste real output, trimmed.

## Structure

- Lead with what the feature does and a short example: a command, a config block or the output a
  user sees.
- Organise sections around the user's tasks ("Recording an exception", "Formatting in CI"), not
  around the implementation.
- One topic per section. Put flags, config keys and codes in reference tables near the top or
  bottom, and details that rarely matter in a short "Details" or "Limitations" list at the end.
- Keep headings that other pages link to. Before renaming or removing one, run
  `grep -rn "<page>#" website/src` and update the links.

## Tables

- Use a table for anything that varies by dialect or Rule: support, defaults, fixes and caveats.
  Don't write a paragraph per dialect or per Rule.
- Use tables for flags, config keys with defaults, exit codes, Rule codes and fields.
- Keep cells short. Put a long explanation in the text below the table.

## Sentences

- Write short, plain sentences in the present tense. One idea per sentence.
- Give every Rule an example where it matters, such as the SQL that triggers a finding and the SQL
  that fixes it.
- Use neutral orders-style examples (orders, customers, products, payments).

## No history

- Describe the product as it is. Don't write "now", "no longer", "used to", "earlier releases",
  "as before", "upgrade note" or "previously". History belongs in `CHANGELOG.md`.
- Drop notes about legacy syntax and migrations from old versions once the error message itself
  tells the user what to do.

## Changing a page

- When a feature changes a page, edit the page into shape: put the new fact in the section it
  belongs to, update tables and examples, and remove what is no longer true. Don't append a
  paragraph to the end of a section or page.
- Don't repeat a rule that another page owns. Link to it instead, for example to
  `rules/overview#exceptions` for how exceptions match findings.
