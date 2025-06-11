-- migrate:up
CREATE TABLE auth.permissions (
	name VARCHAR(100) PRIMARY KEY,
	description TEXT NOT NULL
);

-- migrate:down
DROP TABLE IF EXISTS auth.permissions;
