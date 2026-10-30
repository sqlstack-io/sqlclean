#!/usr/bin/env bash
# Record one demo to out/<demo>.gif and out/<demo>.mp4. Usage: record.sh <demo>
# Uses the sqlclean on PATH; set SQLCLEAN_BIN to a directory containing another sqlclean to use that.
set -euo pipefail
demo="$1"
here="$(cd "$(dirname "$0")" && pwd)"
if [[ -n "${SQLCLEAN_BIN:-}" ]]; then
  export PATH="$SQLCLEAN_BIN:$PATH"
fi
"$here/prepare.sh" "$demo"
mkdir -p "$here/out"
cd "$here"
# Headless Chromium's sandbox is unavailable in many containers and CI runners.
VHS_NO_SANDBOX="${VHS_NO_SANDBOX:-true}" vhs "tapes/$demo.tape"
