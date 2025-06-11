-- migrate:up
CREATE TABLE auth.resource_roles (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,

	-- NULL for anonymous access.
	-- Allows wanderers to have resource permissions.
	navigator_id UUID REFERENCES auth.navigators(id) ON DELETE CASCADE,

	role_name VARCHAR(100) NOT NULL REFERENCES auth.roles(name) ON DELETE CASCADE,
	resource_type VARCHAR(50) NOT NULL,

	-- No FK constraint due to polymorphic nature.
	-- Instead, we handle clean-up via triggers.
	resource_id UUID NOT NULL,

	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX resource_roles_nutty_id_idx ON auth.resource_roles(nutty_id);
CREATE INDEX resource_roles_navigator_id_idx ON auth.resource_roles(navigator_id);
CREATE INDEX resource_roles_resource_idx ON auth.resource_roles(resource_type, resource_id);

CREATE TRIGGER update_auth_resource_roles_updated_at
BEFORE UPDATE ON auth.resource_roles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- A clean-up trigger for when resources are deleted.
CREATE OR REPLACE FUNCTION cleanup_resource_roles()
RETURNS TRIGGER AS $$
BEGIN
	DELETE FROM auth.resource_roles
	WHERE resource_type = TG_ARGV[0] AND resource_id = OLD.id;
	RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER cleanup_block_resource_roles_trigger
BEFORE DELETE ON content.blocks
FOR EACH ROW
EXECUTE FUNCTION cleanup_resource_roles('block');

-- migrate:down
DROP TRIGGER IF EXISTS cleanup_block_resource_roles_trigger ON content.blocks;
DROP FUNCTION IF EXISTS cleanup_resource_roles;
DROP TRIGGER IF EXISTS update_auth_resource_roles_updated_at ON auth.resource_roles;
DROP TABLE IF EXISTS auth.resource_roles;
