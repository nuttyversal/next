local treesitter = require("nvim-treesitter.configs")

-- Use tree-sitter for folding.
vim.opt.foldmethod = "expr"
vim.opt.foldexpr = "nvim_treesitter#foldexpr()"
vim.opt.foldenable = false

treesitter.setup({
	-- Prefer to install parsers asynchronously.
	sync_install = false,

	-- Automatically install missing parsers when entering a buffer.
	auto_install = true,

	highlight = {
		enable = true,
	},

	fold = {
		enable = true,
		custom_foldtext = true,
	},

	indent = {
		enable = true,
	},

	-- Incremental selection enables a visual selection to be intelligently
	-- expanded or shrunk based on the language syntax tree.
	incremental_selection = {
		enable = true,

		keymaps = {
			node_incremental = "v",
			node_decremental = "V",
		},
	},
})
