vim.api.nvim_create_autocmd({ "FileType" }, {
	desc = "Start the Lua language server when editing Lua files",
	group = vim.api.nvim_create_augroup("StartLuaLanguageServer", { clear = true }),
	pattern = "lua",

	callback = function()
		local function infer_root_dir()
			-- These files mark a project root.
			local markers = { "init.lua" }

			-- Begin the search for the project root.
			local buffer_file_dir = vim.fn.expand("%:p:h")
			local current_dir = buffer_file_dir

			while current_dir do
				-- Does the current directory contain a marker file?
				for _, marker in ipairs(markers) do
					if vim.fn.glob(current_dir .. "/" .. marker) ~= "" then
						return current_dir
					end
				end

				-- If not, traverse up the directory tree.
				current_dir = vim.fn.fnamemodify(current_dir, ":h")

				-- If we reach the root directory, then stop.
				if current_dir == "/" then
					break
				end
			end

			return buffer_file_dir
		end

		vim.lsp.start({
			name = "lua",
			cmd = { "lua-language-server" },
			root_dir = infer_root_dir(),

			settings = {
				Lua = {
					diagnostics = {
						enable = true,
						globals = { "vim" },
					},

					workspace = {
						library = {
							[vim.fn.expand("$VIMRUNTIME/lua")] = true,
							[vim.fn.stdpath("data")] = true,
						},
					},
				},
			},
		})
	end,
})
