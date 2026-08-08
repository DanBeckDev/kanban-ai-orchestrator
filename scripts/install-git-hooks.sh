#!/usr/bin/env sh
set -eu

git config core.hooksPath githooks
printf '%s\n' 'Installed repository Git hooks.'
