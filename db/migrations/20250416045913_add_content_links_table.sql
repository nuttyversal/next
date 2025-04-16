-- migrate:up
CREATE TABLE content.links (
	id UUID PRIMARY KEY,
	nutty_id VARCHAR(7) NOT NULL,
	source_id UUID NOT NULL,
	target_id UUID NOT NULL,
	created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	CONSTRAINT links_source_id_fkey FOREIGN KEY (source_id) REFERENCES content.blocks(id) ON DELETE CASCADE,
	CONSTRAINT links_target_id_fkey FOREIGN KEY (target_id) REFERENCES content.blocks(id) ON DELETE CASCADE,
	CONSTRAINT links_source_target_unique UNIQUE (source_id, target_id)
);

CREATE INDEX links_nutty_id_idx ON content.links(nutty_id);
CREATE INDEX links_source_id_idx ON content.links(source_id);
CREATE INDEX links_target_id_idx ON content.links(target_id);

CREATE TRIGGER update_content_links_updated_at
BEFORE UPDATE ON content.links
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- migrate:down
DROP TRIGGER IF EXISTS update_content_links_updated_at ON content.links;
DROP TABLE IF EXISTS content.links;
