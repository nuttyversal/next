local outline = require("outline")

-- Toggle the symbol outline pane.
vim.keymap.set("n", "<Leader>to", "<Cmd>Outline<CR>")

outline.setup({
	outline_window = {
		-- Open the outline on the right side.
		position = "right",

		-- By default, take up 20% of total width.
		width = 20,
		relative = true,

		-- Automatically scroll when navigating the outline.
		auto_jump = true,

		-- Apply temporary highlight when auto-jumping.
		jump_highlight_duration = 16,

		-- Execute `zz` when auto-jumping.
		center_on_jump = true,
	},

	symbols = {
		icons = {
			File = { icon = "󰈔 ", hl = "Identifier" },
			Module = { icon = "󰆧", hl = "Include" },
			Namespace = { icon = "󰅪 ", hl = "Include" },
			Package = { icon = "󰏗 ", hl = "Include" },
			Class = { icon = "𝓒", hl = "Type" },
			Method = { icon = "ƒ", hl = "Function" },
			Property = { icon = " ", hl = "Identifier" },
			Field = { icon = "󰆨 ", hl = "Identifier" },
			Constructor = { icon = " ", hl = "Special" },
			Enum = { icon = "ℰ", hl = "Type" },
			Interface = { icon = "󰜰 ", hl = "Type" },
			Function = { icon = " ", hl = "Function" },
			Variable = { icon = " ", hl = "Constant" },
			Constant = { icon = " ", hl = "Constant" },
			String = { icon = "𝓐", hl = "String" },
			Number = { icon = "#", hl = "Number" },
			Boolean = { icon = "⊨", hl = "Boolean" },
			Array = { icon = "󰅪 ", hl = "Constant" },
			Object = { icon = "⦿", hl = "Type" },
			Key = { icon = "🔐 ", hl = "Type" },
			Null = { icon = "NULL", hl = "Type" },
			EnumMember = { icon = " ", hl = "Identifier" },
			Struct = { icon = "𝓢", hl = "Structure" },
			Event = { icon = "🗲", hl = "Type" },
			Operator = { icon = "+", hl = "Identifier" },
			TypeParameter = { icon = "𝙏", hl = "Identifier" },
			Component = { icon = "󰅴 ", hl = "Function" },
			Fragment = { icon = "󰅴 ", hl = "Constant" },
			TypeAlias = { icon = " ", hl = "Type" },
			Parameter = { icon = " ", hl = "Identifier" },
			StaticMethod = { icon = " ", hl = "Function" },
			Macro = { icon = " ", hl = "Function" },
		},
	},
})
