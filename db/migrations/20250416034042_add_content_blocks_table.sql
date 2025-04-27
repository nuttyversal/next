-- migrate:up
CREATE TABLE content.blocks (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,
	parent_id UUID,
	f_index TEXT NOT NULL,
	content JSONB NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	CONSTRAINT blocks_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES content.blocks(id)
);

CREATE INDEX blocks_nutty_id_idx ON content.blocks(nutty_id);
CREATE INDEX blocks_parent_id_idx ON content.blocks(parent_id);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
	NEW.updated_at = NOW();
	RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_content_blocks_updated_at
BEFORE UPDATE ON content.blocks
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- migrate:down
DROP TABLE IF EXISTS content.blocks;
DROP TRIGGER IF EXISTS update_objects_updated_at ON content.blocks;
DROP FUNCTION IF EXISTS update_updated_at_column;
