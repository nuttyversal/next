-- Improve buffer handling by enabling hidden buffers, confirming
-- unsaved changes, and automatically writing changes to disk.
vim.opt.hidden = true
vim.opt.confirm = true
vim.opt.autowrite = true
vim.opt.autowriteall = true
vim.opt.autoread = true

vim.api.nvim_create_autocmd({ "BufEnter" }, {
	desc = "Reload buffers when they are changed outside of Neovim",
	group = vim.api.nvim_create_augroup("ReloadChangedBuffers", { clear = true }),
	pattern = { "*" },
	command = [[checktime]],
})

vim.api.nvim_create_autocmd({ "BufLeave", "FocusLost" }, {
	desc = "Automatically write buffers to disk when leaving or losing focus",
	group = vim.api.nvim_create_augroup("AutoWriteBuffers", { clear = true }),
	pattern = { "*" },
	command = [[silent! wall]],
})

if vim.fn.has("persistent_undo") then
	-- The forest of undos can be found in the cache directory.
	local undodir = vim.fn.expand("~/.cache") .. "/nvim/undodir"

	if vim.fn.isdirectory(undodir) == 0 then
		vim.fn.mkdir(undodir, "p")
	end

	-- When unloading a buffer, uproot the tree of undos and save to disk.
	-- When loading a buffer, plant the tree of undos from disk.
	vim.opt.undodir = undodir
	vim.opt.undofile = true
end

-- With persistent undo trees, autowriting, and version control,
-- there is no need for these annoying backup and swap files.
vim.opt.backup = false
vim.opt.writebackup = false
vim.opt.swapfile = false

-- Two spaces is too little, four spaces is too much.
-- Three spaces is just right — the Goldilocks of indentation.
vim.opt.listchars = "tab:▸ ,space:·"
vim.opt.smarttab = true
vim.opt.expandtab = true
vim.opt.shiftwidth = 3
vim.opt.tabstop = 3

vim.api.nvim_create_autocmd({ "BufWritePre" }, {
	desc = "Strip trailing whitespace when writing buffers to disk",
	group = vim.api.nvim_create_augroup("StripTrailingWhitespace", { clear = true }),
	pattern = { "*" },
	command = [[%s/\s\+$//e]],
})

-- Improve search by enabling regular expressions, incremental search,
-- highlighting search results, ignoring case, and using smart case.
vim.opt.magic = true
vim.opt.incsearch = true
vim.opt.hlsearch = true
vim.opt.ignorecase = true
vim.opt.smartcase = true

-- Improve responsiveness by reducing the timeout for key sequences.
vim.opt.timeout = false
vim.opt.ttimeout = true
vim.opt.ttimeoutlen = 10

-- Add padding around the edges of the window to improve readability
-- when scrolling and moving the cursor near the edge of the window.
vim.opt.scrolloff = 10
vim.opt.sidescrolloff = 10

-- Improve text navigation fluidity by allowing backspacing over
-- line breaks and allowing cursor movement to cross over lines.
vim.opt.wrap = false
vim.opt.backspace = "indent,eol,start"
vim.opt.whichwrap:append("<>[]~")

-- Use the system clipboard as the default register for copy and paste.
vim.opt.clipboard = "unnamedplus"

-- Add line number column and disable fold column to the left side of the window.
vim.opt.number = true
vim.opt.foldcolumn = "0"

-- Improve ✨ aesthetics ✨.
vim.opt.cursorline = true
vim.opt.termguicolors = true

-- Remove the tilde (~) characters at the end of the buffer.
vim.opt.fillchars = {
	eob = " ",
	diff = " ",
}

-- Set <space> as the leader key for custom mappings.
vim.g.mapleader = " "

-- Navigate between windows in Colemak.
vim.api.nvim_set_keymap("n", "<Leader>wm", "<C-w><C-h>", { noremap = true })
vim.api.nvim_set_keymap("n", "<Leader>wn", "<C-w><C-j>", { noremap = true })
vim.api.nvim_set_keymap("n", "<Leader>we", "<C-w><C-k>", { noremap = true })
vim.api.nvim_set_keymap("n", "<Leader>wi", "<C-w><C-l>", { noremap = true })

-- Toggle line wrapping.
vim.api.nvim_set_keymap("n", "<Leader>tl", ":set wrap!<CR>", { noremap = true })

-- Toggle whitespace visibility.
vim.api.nvim_set_keymap("n", "<Leader>tw", ":set list!<CR>", { noremap = true })

-- Toggle paste mode.
vim.api.nvim_set_keymap("n", "<Leader>tp", ":set paste!<CR>", { noremap = true })

-- Clear search highlighting.
vim.api.nvim_set_keymap("n", "//", ":noh<CR>", { noremap = true })

-- Write the buffer to disk.
vim.api.nvim_set_keymap("n", "<Leader>ww", ":w<CR>", { noremap = true, silent = true })

-- Did you forget to open the file as a superuser? No problem!
vim.api.nvim_set_keymap("c", "w!!", "w !sudo tee % >/dev/null", { noremap = true })

-- Load configuration modules.
require("config.lazy")
require("config.lsp")
require("config.neogit")
require("config.tree")
require("config.tree-sitter")
