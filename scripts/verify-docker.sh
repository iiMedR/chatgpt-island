#!/bin/sh
set -eu
step() {
  printf '\n== %s ==\n' "$1"
  shift
  "$@" || { result=$?; printf 'FAILED (exit %s)\n' "$result" >&2; exit "$result"; }
}
step 'TypeScript check' npm run typecheck
step 'JavaScript and security configuration tests' npm test
step 'Frontend production build' npm run build
