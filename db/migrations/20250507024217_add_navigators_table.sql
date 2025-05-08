-- migrate:up
CREATE SCHEMA IF NOT EXISTS auth;

CREATE TABLE auth.navigators (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,
	name VARCHAR(255) NOT NULL UNIQUE,
	pass VARCHAR(255) NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX navigators_nutty_id_idx ON auth.navigators(nutty_id);
CREATE INDEX navigators_name_idx ON auth.navigators(name);

CREATE TRIGGER update_auth_navigators_updated_at
BEFORE UPDATE ON auth.navigators
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- migrate:down
DROP TRIGGER IF EXISTS update_auth_navigators_updated_at ON auth.navigators;
DROP TABLE IF EXISTS auth.navigators;
