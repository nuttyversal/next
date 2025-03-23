-- Instead of using tab to accept the completion …
vim.g.copilot_no_tab_map = true

-- … use <C-i> to accept the completion.
vim.keymap.set("i", "<C-i>", 'copilot#Accept("\\<CR>")', {
	replace_keycodes = false,
	expr = true,
})
