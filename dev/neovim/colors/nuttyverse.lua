local colors = {
	text = "#272727",
	background = "#070707",
	gray01 = "#070707",
	gray02 = "#171717",
	gray03 = "#212121",
	gray04 = "#292929",
	gray05 = "#313131",
	gray06 = "#3a3a3a",
	gray07 = "#484848",
	gray08 = "#606060",
	gray09 = "#6d6d6d",
	gray10 = "#7a7a7a",
	gray11 = "#b3b3b3",
	gray12 = "#eeeeee",
	green01 = "#0e1512",
	green11 = "#3dd68c",
	red01 = "#191111",
	red11 = "#ff9592",
	yellow02 = "#1b180f",
	yellow10 = "#ffff57",
	yellow11 = "#f5e147",
	yellow12 = "#f6eeb4",

	-- A light white-yellow.
	wellow = "#ffffdd",
}

local function set_highlights()
	-- Editor background.
	vim.api.nvim_set_hl(0, "Normal", { fg = colors.gray12, bg = colors.background })

	-- Any comment.
	vim.api.nvim_set_hl(0, "Comment", { fg = colors.gray10 })

	-- Any constant.
	vim.api.nvim_set_hl(0, "Constant", { fg = colors.gray11 })

	-- A string constant (e.g., "this is a string").
	vim.api.nvim_set_hl(0, "String", { fg = colors.gray11 })

	-- A character constant (e.g., 'c', '\n').
	vim.api.nvim_set_hl(0, "Character", { fg = colors.gray11 })

	-- A number constant (e.g., 234, 0xff).
	vim.api.nvim_set_hl(0, "Number", { fg = colors.gray11 })

	-- A boolean constant (e.g., TRUE, false)
	vim.api.nvim_set_hl(0, "Boolean", { fg = colors.gray11 })

	-- A floating point constant (e.g., 2.3e10).
	vim.api.nvim_set_hl(0, "Float", { fg = colors.gray11 })

	-- Any variable name.
	vim.api.nvim_set_hl(0, "Identifier", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "@variable", { fg = colors.gray12 })

	-- Function names & class methods.
	vim.api.nvim_set_hl(0, "Function", { fg = colors.yellow12, italic = true })

	-- Any statement.
	vim.api.nvim_set_hl(0, "Statement", { fg = colors.yellow11 })

	-- if, then, else, endif, switch, etc.
	vim.api.nvim_set_hl(0, "Conditional", { fg = colors.yellow11 })

	-- for, do, while, etc.
	vim.api.nvim_set_hl(0, "Repeat", { fg = colors.yellow11 })

	-- case, default, etc.
	vim.api.nvim_set_hl(0, "Label", { fg = colors.yellow11 })

	-- "sizeof", "+", "*", etc.
	vim.api.nvim_set_hl(0, "Operator", { fg = colors.gray12 })

	-- Any other keyword.
	vim.api.nvim_set_hl(0, "Keyword", { fg = colors.yellow11, bold = true, italic = true })

	-- try, catch, throw
	vim.api.nvim_set_hl(0, "Exception", { fg = colors.red11 })

	-- Generic preprocessor.
	vim.api.nvim_set_hl(0, "PreProc", { fg = colors.gray12 })

	-- Preprocessor #include.
	vim.api.nvim_set_hl(0, "Include", { fg = colors.gray12 })

	-- Preprocessor #define.
	vim.api.nvim_set_hl(0, "Define", { fg = colors.gray12 })

	-- Same as Define.
	vim.api.nvim_set_hl(0, "Macro", { fg = colors.gray12 })

	-- Preprocessor #if, #else, #endif, etc.
	vim.api.nvim_set_hl(0, "PreCondit", { fg = colors.gray12 })

	-- int, long, char, etc.
	vim.api.nvim_set_hl(0, "Type", { fg = colors.gray11, italic = true })

	-- static, register, volatile, etc.
	vim.api.nvim_set_hl(0, "StorageClass", { fg = colors.yellow11 })

	-- struct, union, enum, etc.
	vim.api.nvim_set_hl(0, "Structure", { fg = colors.yellow11 })

	-- A typedef.
	vim.api.nvim_set_hl(0, "Typedef", { fg = colors.yellow11 })

	-- Any special symbol.
	vim.api.nvim_set_hl(0, "Special", { fg = colors.gray12 })

	-- A special character in a constant.
	vim.api.nvim_set_hl(0, "SpecialChar", { fg = colors.gray12 })

	-- You can use Ctrl-] on this.
	vim.api.nvim_set_hl(0, "Tag", { fg = colors.yellow12 })
	vim.api.nvim_set_hl(0, "@tag.html", { fg = colors.yellow11, italic = true })

	-- A character that needs attention.
	vim.api.nvim_set_hl(0, "Delimiter", { fg = colors.gray12 })

	-- Special things inside a comment.
	vim.api.nvim_set_hl(0, "SpecialComment", { fg = colors.gray12 })

	-- Debugging statements.
	vim.api.nvim_set_hl(0, "Debug", { fg = colors.gray12 })

	-- Text that stands out & HTML links.
	vim.api.nvim_set_hl(0, "Underlined", { fg = colors.gray12 })

	-- Left blank, hidden, hl-Ignore.
	vim.api.nvim_set_hl(0, "Ignore", { fg = colors.gray12 })

	-- Any erroneous construct.
	vim.api.nvim_set_hl(0, "Error", { fg = colors.red11 })

	-- Anything that needs extra attention (e.g., TODO, FIXME, XXX).
	vim.api.nvim_set_hl(0, "Todo", { fg = colors.yellow10 })

	-- Added line in a diff.
	vim.api.nvim_set_hl(0, "Added", { fg = colors.gray12 })

	-- Changed line in a diff.
	vim.api.nvim_set_hl(0, "Changed", { fg = colors.gray12 })

	-- Removed line in a diff.
	vim.api.nvim_set_hl(0, "Removed", { fg = colors.gray12 })

	-- Used for the columns set with 'colorcolumn'.
	vim.api.nvim_set_hl(0, "ColorColumn", { fg = colors.gray12 })

	-- Placeholder characters substituted for concealed.
	vim.api.nvim_set_hl(0, "Conceal", { fg = colors.gray12 })

	-- Copilot completion suggestion.
	vim.api.nvim_set_hl(0, "CopilotSuggestion", { fg = colors.gray07 })

	-- Current match for the last search pattern (see 'hlsearch').
	vim.api.nvim_set_hl(0, "CurSearch", { fg = colors.gray01, bg = colors.yellow10 })

	-- Character under the cursor.
	vim.api.nvim_set_hl(0, "Cursor", { fg = colors.gray12, bold = true, force = true })

	-- Character under the cursor when language mapping.
	vim.api.nvim_set_hl(0, "lCursor", { fg = colors.gray12, bold = true })

	-- Like Cursor, but used when in IME mode.
	vim.api.nvim_set_hl(0, "CursorIM", { fg = colors.gray12, bold = true })

	-- Screen-column at the cursor, when 'cursorcolumn' is set.
	vim.api.nvim_set_hl(0, "CursorColumn", { fg = colors.gray12 })

	-- Screen-line at the cursor, when 'cursorline' is set.
	vim.api.nvim_set_hl(0, "CursorLine", { bg = colors.yellow02 })

	-- Directory names (and other special names in listings).
	vim.api.nvim_set_hl(0, "Directory", { fg = colors.gray12 })

	-- Diff mode: Added line. diff.txt
	vim.api.nvim_set_hl(0, "DiffAdd", { fg = colors.gray12 })

	-- Diff mode: Changed line. diff.txt
	vim.api.nvim_set_hl(0, "DiffChange", { fg = colors.gray12 })

	-- Diff mode: Deleted line. diff.txt
	vim.api.nvim_set_hl(0, "DiffDelete", { fg = colors.gray12 })

	-- Diff mode: Changed text within a changed line. diff.txt
	vim.api.nvim_set_hl(0, "DiffText", { fg = colors.gray12 })

	-- Filler lines (~) after the end of the buffer.
	vim.api.nvim_set_hl(0, "EndOfBuffer", { fg = colors.gray12 })

	-- Cursor in a focused terminal.
	vim.api.nvim_set_hl(0, "TermCursor", { fg = colors.gray12 })

	-- Cursor in an unfocused terminal.
	vim.api.nvim_set_hl(0, "TermCursorNC", { fg = colors.gray12 })

	-- Error messages on the command line.
	vim.api.nvim_set_hl(0, "ErrorMsg", { fg = colors.gray12 })

	-- Separators between window splits.
	vim.api.nvim_set_hl(0, "WinSeparator", { fg = colors.gray06 })

	-- Line used for closed folds.
	vim.api.nvim_set_hl(0, "Folded", { fg = colors.gray12 })

	-- 'foldcolumn'
	vim.api.nvim_set_hl(0, "FoldColumn", { fg = colors.gray12 })

	-- Column where signs are displayed.
	vim.api.nvim_set_hl(0, "SignColumn", { fg = colors.gray12 })

	-- 'incsearch' highlighting; also used for the text replaced with.
	vim.api.nvim_set_hl(0, "IncSearch", { fg = colors.gray01, bg = colors.yellow10 })

	-- :substitute replacement text highlighting.
	vim.api.nvim_set_hl(0, "Substitute", { fg = colors.green11 })

	-- Line number for ":number" and ":#" commands, and when 'number'.
	vim.api.nvim_set_hl(0, "LineNr", { fg = colors.gray09 })

	-- Line number for when the 'relativenumber'.
	vim.api.nvim_set_hl(0, "LineNrAbove", { fg = colors.gray12 })

	-- Line number for when the 'relativenumber'.
	vim.api.nvim_set_hl(0, "LineNrBelow", { fg = colors.gray12 })

	-- LSP inlay hints.
	vim.api.nvim_set_hl(0, "LspInlayHint", { fg = colors.gray08 })

	-- Like LineNr when 'cursorline' is set and 'cursorlineopt'.
	vim.api.nvim_set_hl(0, "CursorLineNr", { fg = colors.gray12 })

	-- Like FoldColumn when 'cursorline' is set for the cursor line.
	vim.api.nvim_set_hl(0, "CursorLineFold", { fg = colors.gray12 })

	-- Like SignColumn when 'cursorline' is set for the cursor line.
	vim.api.nvim_set_hl(0, "CursorLineSign", { fg = colors.gray12 })

	-- Character under the cursor or just before it.
	vim.api.nvim_set_hl(0, "MatchParen", { fg = colors.gray12, bg = colors.gray10 })

	-- 'showmode' message (e.g., "-- INSERT --").
	vim.api.nvim_set_hl(0, "ModeMsg", { fg = colors.yellow11, bold = true })

	-- Area for messages and command-line, see also 'cmdheight'.
	vim.api.nvim_set_hl(0, "MsgArea", { fg = colors.gray12 })

	-- Separator for scrolled messages msgsep.
	vim.api.nvim_set_hl(0, "MsgSeparator", { fg = colors.gray12 })

	-- more-prompt.
	vim.api.nvim_set_hl(0, "MoreMsg", { fg = colors.gray12 })

	-- '@' at the end of the window, characters from 'showbreak'.
	vim.api.nvim_set_hl(0, "NonText", { fg = colors.gray12 })

	-- Normal text.
	vim.api.nvim_set_hl(0, "Normal", { fg = colors.gray12 })

	-- Normal text in floating windows.
	vim.api.nvim_set_hl(0, "NormalFloat", { fg = colors.gray11 })
	vim.api.nvim_set_hl(0, "@markup.raw.block", { fg = colors.gray11 })

	-- Border of floating windows.
	vim.api.nvim_set_hl(0, "FloatBorder", { fg = colors.gray06 })

	-- Title of floating windows.
	vim.api.nvim_set_hl(0, "FloatTitle", { fg = colors.gray12 })

	-- Footer of floating windows.
	vim.api.nvim_set_hl(0, "FloatFooter", { fg = colors.gray12 })

	-- Normal text in non-current windows.
	vim.api.nvim_set_hl(0, "NormalNC", { fg = colors.gray12 })

	-- Popup menu: Normal item.
	vim.api.nvim_set_hl(0, "Pmenu", { fg = colors.gray11 })

	-- Popup menu: Selected item.
	vim.api.nvim_set_hl(0, "PmenuSel", { fg = colors.yellow10 })

	-- Popup menu: Normal item "kind".
	vim.api.nvim_set_hl(0, "PmenuKind", { fg = colors.gray12 })

	-- Popup menu: Selected item "kind".
	vim.api.nvim_set_hl(0, "PmenuKindSel", { fg = colors.yellow10 })

	-- Popup menu: Normal item "extra text".
	vim.api.nvim_set_hl(0, "PmenuExtra", { fg = colors.gray12 })

	-- Popup menu: Selected item "extra text".
	vim.api.nvim_set_hl(0, "PmenuExtraSel", { fg = colors.gray12 })

	-- Popup menu: Scrollbar.
	vim.api.nvim_set_hl(0, "PmenuSbar", { bg = colors.yellow10 })

	-- Popup menu: Thumb of the scrollbar.
	vim.api.nvim_set_hl(0, "PmenuThumb", { bg = colors.yellow10 })

	-- Popup menu: Matched text in normal item.
	vim.api.nvim_set_hl(0, "PmenuMatch", { fg = colors.yellow10 })

	-- Popup menu: Matched text in selected item.
	vim.api.nvim_set_hl(0, "PmenuMatchSel", { fg = colors.yellow10 })

	-- hit-enter prompt and yes/no questions.
	vim.api.nvim_set_hl(0, "Question", { fg = colors.gray12 })

	-- Current quickfix item in the quickfix window.
	vim.api.nvim_set_hl(0, "QuickFixLine", { fg = colors.gray12 })

	-- Last search pattern highlighting (see 'hlsearch').
	vim.api.nvim_set_hl(0, "Search", { fg = colors.gray01, bg = colors.gray11 })

	-- Tabstops in snippets. vim.snippet
	vim.api.nvim_set_hl(0, "SnippetTabstop", { fg = colors.gray12 })

	-- Unprintable characters.
	vim.api.nvim_set_hl(0, "SpecialKey", { fg = colors.gray12 })

	-- Word that is not recognized by the spellchecker. spell
	vim.api.nvim_set_hl(0, "SpellBad", { fg = colors.gray12 })

	-- Word that should start with a capital. spell
	vim.api.nvim_set_hl(0, "SpellCap", { fg = colors.gray12 })

	-- Word that is recognized by the spellchecker as one that is
	vim.api.nvim_set_hl(0, "SpellLocal", { fg = colors.gray12 })

	-- Word that is recognized by the spellchecker as one that is
	vim.api.nvim_set_hl(0, "SpellRare", { fg = colors.gray12 })

	-- Status line of current window.
	vim.api.nvim_set_hl(0, "StatusLine", { fg = colors.gray12 })

	-- Status lines of not-current windows.
	vim.api.nvim_set_hl(0, "StatusLineNC", { fg = colors.gray12 })

	-- Status line of terminal window.
	vim.api.nvim_set_hl(0, "StatusLineTerm", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "StatusLineTermN", { fg = colors.gray12 })

	-- Tab pages line, not active tab page label.
	vim.api.nvim_set_hl(0, "TabLine", { fg = colors.gray12 })

	-- Tab pages line, where there are no labels.
	vim.api.nvim_set_hl(0, "TabLineFill", { fg = colors.gray12 })

	-- Tab pages line, active tab page label.
	vim.api.nvim_set_hl(0, "TabLineSel", { fg = colors.gray12 })

	-- Titles for output from ":set all", ":autocmd" etc.
	vim.api.nvim_set_hl(0, "Title", { fg = colors.gray12 })

	-- Visual mode selection.
	vim.api.nvim_set_hl(0, "Visual", { fg = colors.gray01, bg = colors.gray11 })

	-- Visual mode selection when vim is "Not Owning the Selection".
	vim.api.nvim_set_hl(0, "VisualNOS", { fg = colors.gray12 })

	-- Warning messages.
	vim.api.nvim_set_hl(0, "WarningMsg", { fg = colors.gray12 })

	-- "nbsp", "space", "tab", "multispace", "lead" and "trail".
	vim.api.nvim_set_hl(0, "Whitespace", { fg = colors.gray06 })

	-- Current match in 'wildmenu' completion.
	vim.api.nvim_set_hl(0, "WildMenu", { fg = colors.gray12 })

	-- Window bar of current window.
	vim.api.nvim_set_hl(0, "WinBar", { fg = colors.gray12 })

	-- Window bar of not-current windows.
	vim.api.nvim_set_hl(0, "WinBarNC", { fg = colors.gray12 })
end

local function set_highlights_for_diagnostics()
	vim.api.nvim_set_hl(0, "DiagnosticError", { fg = colors.red11 })
	vim.api.nvim_set_hl(0, "DiagnosticWarn", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "DiagnosticInfo", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "DiagnosticHint", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "DiagnosticOk", { fg = colors.green11 })
	vim.api.nvim_set_hl(0, "DiagnosticUnderlineError", { fg = colors.red11, underline = true })
	vim.api.nvim_set_hl(0, "DiagnosticUnderlineWarn", { fg = colors.gray12, underline = true })
	vim.api.nvim_set_hl(0, "DiagnosticUnderlineInfo", { fg = colors.gray12, underline = true })
	vim.api.nvim_set_hl(0, "DiagnosticUnderlineHint", { fg = colors.gray12, underline = true })
	vim.api.nvim_set_hl(0, "DiagnosticUnderlineOk", { fg = colors.green11, underline = true })
end

local function set_highlights_for_mini_diff()
	vim.api.nvim_set_hl(0, "MiniDiffOverAdd", {
		fg = colors.green11,
		bg = colors.green01,
	})

	vim.api.nvim_set_hl(0, "MiniDiffOverDelete", {
		fg = colors.red11,
		bg = colors.red01,
	})

	vim.api.nvim_set_hl(0, "MiniDiffOverChange", {
		fg = colors.gray10,
		bg = colors.gray03,
	})
end

local function set_highlights_for_neogit()
	vim.api.nvim_set_hl(0, "NeogitCursorLine", {
		fg = colors.gray12,
		bg = colors.gray04,
	})

	vim.api.nvim_set_hl(0, "NeogitSectionHeader", {
		fg = colors.gray12,
	})

	vim.api.nvim_set_hl(0, "NeogitHunkHeader", {
		fg = colors.gray01,
		bg = colors.gray11,
	})

	vim.api.nvim_set_hl(0, "NeogitHunkHeaderHighlight", {
		fg = colors.gray01,
		bg = colors.gray12,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffContext", {
		fg = colors.gray12,
		bg = colors.gray02,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffContextHighlight", {
		fg = colors.gray12,
		bg = colors.gray03,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffAdd", {
		fg = colors.green11,
		bg = colors.green01,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffAddHighlight", {
		fg = colors.green11,
		bg = colors.green01,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffDelete", {
		fg = colors.red11,
		bg = colors.red01,
	})

	vim.api.nvim_set_hl(0, "NeogitDiffDeleteHighlight", {
		fg = colors.red11,
		bg = colors.red01,
	})

	vim.api.nvim_set_hl(0, "NeogitChangeUpdated", {
		fg = colors.gray10,
		bold = true,
		italic = true,
	})

	vim.api.nvim_set_hl(0, "NeogitChangeRenamed", {
		fg = colors.gray10,
		bold = true,
		italic = true,
	})

	vim.api.nvim_set_hl(0, "NeogitChangeDeleted", {
		fg = colors.red11,
		bold = true,
		italic = true,
	})

	vim.api.nvim_set_hl(0, "NeogitChangeAdded", {
		fg = colors.green11,
		bold = true,
		italic = true,
	})

	vim.api.nvim_set_hl(0, "NeogitChangeModified", {
		fg = colors.gray10,
		bold = true,
		italic = true,
	})
end

local function set_highlights_for_nvim_tree()
	vim.api.nvim_set_hl(0, "NvimTreeFolderIcon", { fg = colors.gray12 })
	vim.api.nvim_set_hl(0, "NvimTreeIndentMarker", { fg = colors.gray09 })
	vim.api.nvim_set_hl(0, "Directory", { fg = colors.yellow12 })
end

local function set_highlights_for_telescope()
	vim.api.nvim_set_hl(0, "TelescopeBorder", { fg = colors.gray06 })
	vim.api.nvim_set_hl(0, "TelescopeMatching", { fg = colors.yellow11 })
end

set_highlights()
set_highlights_for_diagnostics()
set_highlights_for_mini_diff()
set_highlights_for_neogit()
set_highlights_for_nvim_tree()
set_highlights_for_telescope()
