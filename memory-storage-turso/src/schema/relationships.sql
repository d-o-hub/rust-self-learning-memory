-- Episode relationships table
-- Stores relationships between episodes for dependency tracking,
-- hierarchical organization, and workflow modeling.
CREATE TABLE IF NOT EXISTS episode_relationships (
    relationship_id TEXT PRIMARY KEY NOT NULL,
    from_episode_id TEXT NOT NULL,
    to_episode_id TEXT NOT NULL,
    relationship_type TEXT NOT NULL,
    reason TEXT,
    created_by TEXT,
    priority INTEGER CHECK (priority >= 1 AND priority <= 10),
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (from_episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE,
    FOREIGN KEY (to_episode_id) REFERENCES episodes(episode_id) ON DELETE CASCADE,
    UNIQUE(from_episode_id, to_episode_id, relationship_type)
);

-- Indexes for fast queries
-- Index on relationships for efficient outgoing relationship queries
CREATE INDEX IF NOT EXISTS idx_relationships_from 
    ON episode_relationships(from_episode_id);

-- Index on relationships for efficient incoming relationship queries
CREATE INDEX IF NOT EXISTS idx_relationships_to 
    ON episode_relationships(to_episode_id);

-- Index on relationships for efficient type-based queries
CREATE INDEX IF NOT EXISTS idx_relationships_type 
    ON episode_relationships(relationship_type);

-- Index on relationships for efficient bidirectional queries
CREATE INDEX IF NOT EXISTS idx_relationships_bidirectional 
    ON episode_relationships(from_episode_id, to_episode_id);

-- Index on relationships for time-based queries
CREATE INDEX IF NOT EXISTS idx_relationships_created 
    ON episode_relationships(created_at);
