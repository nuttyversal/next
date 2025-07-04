local tree = require("nvim-tree")

-- Toggle the file explorer.
vim.api.nvim_set_keymap("n", "<Leader>te", ":NvimTreeToggle<CR>", { noremap = true })

tree.setup({
	filters = {
		dotfiles = false,
	},

	disable_netrw = true,
	hijack_netrw = true,
	hijack_cursor = true,
	hijack_unnamed_buffer_when_opening = false,
	sync_root_with_cwd = true,

	sort = {
		sorter = "case_sensitive",
	},

	update_focused_file = {
		enable = true,
		update_root = false,
	},

	view = {
		adaptive_size = true,
		side = "left",
		preserve_window_proportions = true,
	},

	git = {
		enable = true,
		ignore = true,
	},

	filesystem_watchers = {
		enable = true,
	},

	actions = {
		open_file = {
			resize_window = true,
		},
	},

	renderer = {
		root_folder_label = false,
		highlight_git = true,
		highlight_opened_files = "none",

		indent_markers = {
			enable = true,
		},

		icons = {
			show = {
				file = false,
				folder = true,
				folder_arrow = false,
				git = true,
			},

			glyphs = {
				default = "󰈚 ",
				symlink = " ",

				folder = {
					default = " ",
					empty = " ",
					empty_open = " ",
					open = " ",
					symlink = " ",
					symlink_open = " ",
					arrow_open = " ",
					arrow_closed = " ",
				},

				git = {
					unstaged = "✗",
					staged = "✓",
					unmerged = "",
					renamed = "➜",
					untracked = "★",
					deleted = "",
					ignored = "◌",
				},
			},
		},
	},
})
