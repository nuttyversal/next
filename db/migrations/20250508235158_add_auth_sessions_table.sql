-- migrate:up
CREATE TABLE auth.sessions (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,
	navigator_id UUID NOT NULL,
	user_agent VARCHAR(255) NOT NULL,
	expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	CONSTRAINT sessions_navigator_id_fkey FOREIGN KEY (navigator_id) REFERENCES auth.navigators(id) ON DELETE CASCADE
);

CREATE INDEX sessions_nutty_id_idx ON auth.sessions(nutty_id);
CREATE INDEX sessions_navigator_id_idx ON auth.sessions(navigator_id);
CREATE INDEX sessions_expires_at_idx ON auth.sessions(expires_at);

CREATE TRIGGER update_auth_sessions_updated_at
BEFORE UPDATE ON auth.sessions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- migrate:down
DROP TRIGGER IF EXISTS update_auth_sessions_updated_at ON auth.sessions;
DROP TABLE IF EXISTS auth.sessions;
