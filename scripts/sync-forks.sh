#!/usr/bin/env bash
# The forks of bdhtml2pdf's core dependencies under github.com/bahadirdogru,
# kept in case an upstream repository goes away. GitHub does not update a
# fork by itself: this brings the default branch of each one up to its
# upstream's with `gh repo sync`. That only fast-forwards. A fork whose
# default branch has commits of its own is reported and left alone, and the
# branches we work on (fulgur blitz-master, Blitz public-layout-pass, pr/*)
# are never touched.
#
#   scripts/sync-forks.sh            sync every fork
#   scripts/sync-forks.sh NAME...    sync only these forks
#   scripts/sync-forks.sh --list     print each fork and its upstream
#
# Needs the GitHub CLI, logged in as the fork owner (`gh auth login`).
set -euo pipefail

owner=bahadirdogru
# Rendering, text, fonts, PDF, SVG and JavaScript: the projects bdhtml2pdf
# could not do without. General-purpose crates of large teams (serde, tokio,
# clap, url, image) are left out; crates.io keeps their published versions.
forks=(
  fulgur blitz taffy                                   # patched by us
  stylo html5ever rust-cssparser                       # HTML and CSS
  parley harfrust fontations icu4x wuff woff2-rs fontdb # text and fonts
  krilla pdf-writer subsetter hayro                    # PDF
  resvg tiny-skia vello anyrender                      # SVG and drawing
  boa                                                  # JavaScript
)

if [[ ${1:-} == --list ]]; then
  for name in "${forks[@]}"; do
    gh repo view "$owner/$name" --json nameWithOwner,parent,defaultBranchRef \
      --jq '"\(.nameWithOwner) <- \(.parent.owner.login)/\(.parent.name) [\(.defaultBranchRef.name)]"' \
      || echo "$owner/$name: not found"
  done
  exit 0
fi

(($#)) && forks=("$@")
failed=()
for name in "${forks[@]}"; do
  if out=$(gh repo sync "$owner/$name" 2>&1); then
    echo "$name: ${out:-synced}"
  else
    echo "$name: NOT SYNCED: $out" >&2
    failed+=("$name")
  fi
done

if ((${#failed[@]})); then
  echo "not synced: ${failed[*]} (diverged from upstream or missing; see above)" >&2
  exit 1
fi
