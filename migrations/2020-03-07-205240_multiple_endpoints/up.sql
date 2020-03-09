CREATE TABLE tags (
  tag_id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
  url_suffix VARCHAR(32) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  active BOOLEAN NOT NULL
);
CREATE INDEX is_active ON tags (active, tag_id DESC);
CREATE INDEX url_match ON tags (url_suffix);
ALTER TABLE webhooks
ADD
  COLUMN tag_id INT NOT NULL REFERENCES tags ON DELETE CASCADE;

CREATE INDEX by_tag ON webhooks (tag_id, id DESC);
