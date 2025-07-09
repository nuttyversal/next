-- migrate:up
ALTER TABLE content.blocks
ADD COLUMN owner_id UUID REFERENCES auth.navigators(id) ON DELETE SET NULL;

CREATE INDEX blocks_owner_id_idx ON content.blocks(owner_id);

-- migrate:down
DROP INDEX IF EXISTS blocks_owner_id_idx;
ALTER TABLE content.blocks DROP COLUMN IF EXISTS owner_id;
