-- migrate:up
CREATE TABLE auth.navigator_roles (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,
	navigator_id UUID NOT NULL REFERENCES auth.navigators(id) ON DELETE CASCADE,
	role_name VARCHAR(100) NOT NULL REFERENCES auth.roles(name) ON DELETE CASCADE,
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	CONSTRAINT navigator_roles_unique UNIQUE (navigator_id, role_name)
);

CREATE INDEX navigator_roles_nutty_id_idx ON auth.navigator_roles(nutty_id);
CREATE INDEX navigator_roles_navigator_id_idx ON auth.navigator_roles(navigator_id);
CREATE INDEX navigator_roles_role_name_idx ON auth.navigator_roles(role_name);

CREATE TRIGGER update_auth_navigator_roles_updated_at
BEFORE UPDATE ON auth.navigator_roles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- migrate:down
DROP TRIGGER IF EXISTS update_auth_navigator_roles_updated_at ON auth.navigator_roles;
DROP TABLE IF EXISTS auth.navigator_roles;
