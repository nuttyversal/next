# Homebrew
eval "$(/opt/homebrew/bin/brew shellenv)"

# GnuPG
set --global --export GPG_TTY (tty)
set --global --export GNUPGHOME ~/.config/gnupg
