-- Open neogit.
vim.api.nvim_set_keymap("n", "<Leader>gg", ":Neogit<CR>", { noremap = true })

require("neogit").setup({
	disable_hint = true,
	graph_style = "unicode",

	signs = {
		-- { CLOSED, OPENED }
		hunk = { "", "" },
		item = { " ", " " },
		section = { " ", " " },
	},

	integrations = {
		telescope = true,
		diffview = true,
	},
})
