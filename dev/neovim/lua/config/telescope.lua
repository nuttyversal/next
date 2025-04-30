--- @diagnostic disable: undefined-field
local telescope = require("telescope")
local telescope_builtin = require("telescope.builtin")

vim.keymap.set("n", "<Leader><Space>", telescope_builtin.find_files, {})
vim.keymap.set("n", "<Leader>fg", telescope_builtin.live_grep, {})
vim.keymap.set("n", "<Leader>fb", telescope_builtin.buffers, {})
vim.keymap.set("n", "<Leader>fh", telescope_builtin.help_tags, {})

telescope.setup({
	pickers = {
		find_files = {
			disable_devicons = true,
		},
	},
})
