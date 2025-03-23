local telescope_builtin = require("telescope.builtin")

--- Infer the project root directory of the current buffer.
---
--- The project root directory is identified by the presence of a marker file,
--- such as `package.json`, which serves as a heuristic for classifying a root
--- directory. This function begins the search in the current directory and
--- traverses up the directory tree until a marker file is found. If none is
--- found, then the buffer directory is assumed to be the project root.
---
--- @param markers table<string> A list of files that mark a project root.
--- @return string project_root The project root directory (or buffer directory).
local function infer_project_root_directory(markers)
	-- Start at the current buffer directory.
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

vim.api.nvim_create_autocmd({ "LspAttach" }, {
	callback = function(args)
		local lsp_client = vim.lsp.get_client_by_id(args.data.client_id)

		if lsp_client ~= nil then
			if lsp_client.supports_method("textDocument/definition") then
				vim.keymap.set("n", "gd", telescope_builtin.lsp_definitions)
			end

			if lsp_client.supports_method("textDocument/references") then
				vim.keymap.set("n", "gr", telescope_builtin.lsp_references)
			end

			if lsp_client.supports_method("textDocument/implementation") then
				vim.keymap.set("n", "gi", telescope_builtin.lsp_implementations)
			end

			if lsp_client.supports_method("textDocument/codeAction") then
				vim.keymap.set("n", "<Space>ca", vim.lsp.buf.code_action)
			end

			if lsp_client.supports_method("textDocument/codeLens") then
				vim.keymap.set("n", "<Space>cc", vim.lsp.codelens.run)

				-- Auto-refresh code lenses.
				vim.api.nvim_create_autocmd({ "TextChanged", "InsertLeave" }, {
					callback = function()
						vim.lsp.codelens.refresh()
					end,
				})
			end

			if lsp_client.supports_method("textDocument/rename") then
				vim.keymap.set("n", "<Space>cr", vim.lsp.buf.rename)
			end
		end

		-- Toggle inlay hints (if supported by the LSP).
		vim.keymap.set("n", "<Leader>ti", function()
			vim.lsp.inlay_hint.enable(not vim.lsp.inlay_hint.is_enabled())
		end, { noremap = true })

		-- Draw borders around floating documentation windows.
		vim.lsp.handlers["textDocument/hover"] = vim.lsp.with(vim.lsp.handlers.hover, {
			border = "rounded",
		})
	end,
})

vim.api.nvim_create_autocmd({ "FileType" }, {
	desc = "Start the Lua language server when editing Lua files",
	group = vim.api.nvim_create_augroup("StartLuaLanguageServer", { clear = true }),
	pattern = "lua",

	callback = function()
		vim.lsp.start({
			name = "lua",
			cmd = { "lua-language-server" },
			root_dir = infer_project_root_directory({ "init.lua" }),

			settings = {
				Lua = {
					codeLens = {
						enable = true,
					},

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

vim.api.nvim_create_autocmd({ "FileType" }, {
	desc = "Start the Rust language server (rust-analyzer) when editing Rust files",
	group = vim.api.nvim_create_augroup("StartRustLanguageServer", { clear = true }),
	pattern = "rust",

	callback = function()
		vim.lsp.start({
			name = "rust-analyzer",
			cmd = { "rust-analyzer" },
			root_dir = infer_project_root_directory({ "Cargo.toml" }),

			settings = {
				["rust-analyzer"] = {
					cargo = {
						allFeatures = true,
					},

					checkOnSave = {
						command = "clippy",
					},

					-- Enable procedural macro support.
					-- Used by crates such as Tokio and Serde.
					procMacro = {
						enable = true,
					},

					inlayHints = {
						-- Show the variable binding mode in patterns.
						-- By value? By reference? By mutable reference?
						bindingModeHints = {
							enable = true,
						},

						-- Show the inferred return types of closures.
						-- E.g., |x| x * 2 might be hinted as |x: &i32| -> i32 x * 2.
						closureReturnTypeHints = {
							enable = "always",
						},

						-- Show implicit expression adjustments.
						-- Dereferencing. Borrowing. Coercion.
						expressionAdjustmentHints = {
							enable = "always",
						},

						-- Show implicit lifetime parameters omitted from signatures.
						lifetimeElisionHints = {
							enable = "always",
							useParameterNames = true,
						},

						-- Show inferred types of variables and expressions.
						typeHints = {
							enable = true,
							hideClosureInitialization = false,
							hideNamedConstructor = false,
						},
					},
				},
			},
		})
	end,
})
