-- migrate:up
INSERT INTO auth.roles (name, description) VALUES
('admin', 'System Administrator');

INSERT INTO auth.role_permissions (role_name, permission_name) VALUES
('admin', 'content_blocks:read:all'),
('admin', 'content_blocks:write:all');

-- migrate:down
DELETE FROM auth.role_permissions WHERE role_name = 'admin';
DELETE FROM auth.roles WHERE name = 'admin';
