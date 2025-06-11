-- migrate:up
CREATE TABLE auth.role_permissions (
	role_name VARCHAR(100) NOT NULL REFERENCES auth.roles(name) ON DELETE CASCADE,
	permission_name VARCHAR(100) NOT NULL REFERENCES auth.permissions(name) ON DELETE CASCADE,
	PRIMARY KEY (role_name, permission_name)
);

CREATE INDEX role_permissions_permission_name_idx ON auth.role_permissions(permission_name);

-- migrate:down
DROP TABLE IF EXISTS auth.role_permissions;
