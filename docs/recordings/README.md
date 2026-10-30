# CLI recordings

Terminal recordings of `sqlclean` for the README and the website, rendered with
[VHS](https://github.com/charmbracelet/vhs) from the `.tape` scripts in `tapes/`. The tapes and
scripts are the source; the rendered GIF, MP4 and PNG files go to `out/`, which is not committed.

| Demo | Shows |
|------|-------|
| `check-fix` | `sqlclean check` reports built-in and custom findings, and `sqlclean fix` fixes and formats. |

The README uses `check-fix`, copied into `.github/demos/`.

## Requirements

- `vhs`, `ttyd` and `ffmpeg` on `PATH`, plus a Chromium or Chrome that VHS can find.
- DejaVu Sans Mono, the font the tapes use.
- `sqlclean` on `PATH` with its Python package in the same environment, or `SQLCLEAN_BIN` pointing at
  a directory that contains the `sqlclean` to record.

## Record

```bash
cd docs/recordings
./record.sh check-fix     # out/check-fix.gif, out/check-fix.mp4 and out/check-fix.png
cp out/check-fix.gif ../../.github/demos/
```

`record.sh` runs `prepare.sh <demo>` to build a clean project under `scratch/<demo>/`, then renders
`tapes/<demo>.tape`. Shared terminal settings and the colour theme are in `tapes/common.tape`.

Look at every recording before publishing it.
