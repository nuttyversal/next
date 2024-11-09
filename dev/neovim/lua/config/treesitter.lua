local treesitter = require("nvim-treesitter.configs")

treesitter.setup({
	-- Prefer to install parsers asynchronously.
	sync_install = false,

	-- Automatically install missing parsers when entering a buffer.
	auto_install = true,

	highlight = {
		enable = true,
	},
})
