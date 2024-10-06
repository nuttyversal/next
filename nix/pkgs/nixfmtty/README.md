# nixfmtty

`nixfmtty` is a simple wrapper around `nixfmt` that formats Nix files
with tabs instead of spaces. Conceptually, this is what it does:

```bash
nixfmt <filename>.nix
unexpand --tabs=2 <filename>.nix | sponge <filename>.nix
```

## Usage

```bash
# Apply formatting.
nixfmtty <filename>.nix

# Apply formatting (recursively).
nixfmtty **/*.nix

# Check formatting.
nixfmtty --check <filename>.nix

# Check formatting (recursively).
nixfmtty --check **/*.nix
```
