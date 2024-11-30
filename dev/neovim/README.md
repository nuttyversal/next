# Nutty's Neovim

## Options

* Buffer management
	* Automatic reloading of externally changed files
	* Automatic writing of changes to disk
	* Confirmation for unsaved changes
	* Hidden buffers enabled

* Persistent undo tree history
	* Stored in `~/.cache/nvim/undodir`

* Whitespace management
	* Automatically remove trailing whitespace
	* Visible whitespace characters
	* 3-width tab indentation

* Searching
	* Case-insensitive search with smart case
	* Regular expressions enabled
	* Search result highlighting
	* Incremental search

* User interface
	* Hide end-of-buffer symbol (`~`)
	* Cursor line highlighting
	* Line numbers
	* Fold column
	* True color support

* Navigation and editing
	* Scroll padding
	* Cross-line cursor movement
	* Colemak-friendly window navigation (keymap)

* Customization
	* `<Space>` leader key

* Convenience
	* System clipboard integration
	* Clear search highlighting (keymap)
	* Quick write to disk (keymap)
	* `sudo` write support (keymap)

## Plugins

* [nvim-surround](https://github.com/kylechui/nvim-surround)
	* Surround selections, stylishly

* [nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter)
	* Syntax highlighting
	* Incremental selection

* [nvim-tree](https://github.com/nvim-tree/nvim-tree.lua)
	* File explorer tab

* [neogit](https://github.com/NeogitOrg/neogit)
	* Magit, but for Neovim

* [telescope](https://github.com/nvim-telescope/telescope.nvim)
	* Gaze deeply into unknown regions using the power of the moon
	* Fuzzy finding over lists

* [blink.cmp](https://github.com/Saghen/blink.cmp)
	* Auto-completion plugin
	* Super fast — uses custom SIMD fuzzy searcher
	* Experimental (beta testing)

* [avante.nvim](https://github.com/yetone/avante.nvim)
	* AI-powered code assistance
	* Emulates [Cursor](https://www.cursor.com/) AI IDE

* [conform.nvim](https://github.com/stevearc/conform.nvim)
	* Formatter configuration
	* Applies text edits with LSP

* To be added… (managed by [lazy.nvim](https://lazy.folke.io/))
