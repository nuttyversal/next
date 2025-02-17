# Homebrew
eval "$(/opt/homebrew/bin/brew shellenv)"

# GnuPG
set --global --export GPG_TTY (tty)
set --global --export GNUPGHOME ~/.config/gnupg

# Zoxide
zoxide init fish | source

# Nix
if test -e '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.fish'
	. '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.fish'
end
