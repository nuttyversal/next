#!/usr/bin/env fish --no-config

python -m venv .venv
source .venv/bin/activate.fish
pip install -r requirements.txt
pip freeze > requirements.lock
