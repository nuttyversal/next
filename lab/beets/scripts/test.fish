#!/usr/bin/env fish

python -m venv .venv
source .venv/bin/activate.fish
beet duplicates
beet fetchart
