#!/usr/bin/env fish --no-config

python -m venv .venv
source .venv/bin/activate.fish
beet bad
beet duplicates
beet fetchart
beet missing
