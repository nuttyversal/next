-- migrate:up
CREATE TABLE auth.roles (
	name VARCHAR(100) PRIMARY KEY,
	description TEXT NOT NULL
);

-- migrate:down
DROP TABLE IF EXISTS auth.roles;
