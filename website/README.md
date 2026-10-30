# Website

The SQLClean website: the homepage at `/` and the documentation at `/docs/`, built as one
[Astro Starlight](https://starlight.astro.build) site.

- `public/index.html` and `public/styles.css`: the homepage, plain static HTML and CSS. Its terminal
  examples are real `sqlclean` output from small example projects, trimmed to the relevant lines.
- `src/content/docs/docs/`: the documentation pages (MDX). `src/sidebar.json` holds the sidebar
  and `src/navigation.mjs` the redirects. `STYLE.md` is the style guide for these pages.
- `src/components/`: the page layout, header and page title, replacing Starlight's defaults.
  `src/components/docs/` has the `Card`, `CardGroup`, `Frame`, `Note` and `Warning` components the
  pages use.
- `public/fonts/`: self-hosted fonts, each with its licence.
- `recordings/`: the VHS tapes behind the README demo; see `recordings/README.md`.

Search uses Pagefind, and `llms.txt`, `llms-small.txt` and `llms-full.txt` are generated at build
time. The Website workflow builds the site and checks its links on every change; it does not deploy.

Requires Node 22.12 or newer.

```bash
npm install
npm run dev      # live preview
npm run build    # static site in dist/, including the search index
npm run preview  # serve dist/
python3 scripts/check_links.py dist
```
