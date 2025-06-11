-- migrate:up
INSERT INTO auth.permissions (name, description) VALUES
('content_blocks:read:all', 'Can view all content blocks.'),
('content_blocks:read:own', 'Can view own content blocks.'),
('content_blocks:write:all', 'Can create, update, and delete all content blocks.'),
('content_blocks:write:own', 'Can create, update, and delete own content blocks.');

-- migrate:down
DELETE FROM auth.permissions WHERE name IN (
	'content_blocks:read:all',
	'content_blocks:read:own',
	'content_blocks:write:all',
	'content_blocks:write:own'
);
