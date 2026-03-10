PRAGMA foreign_keys = ON;

-- Choose your embedding dimensions up front (required for F32_BLOB(dims))
-- Replace these with your real dims:
--   D_INTRO    = e.g. 1536 (OpenAI text-embedding-3-small), 3072, etc.
--   D_INTEREST = could be same as D_INTRO (recommended) or different.
--
-- IMPORTANT: dims are part of the column type; changing dims later is a migration.
--
-- Below uses placeholders:
--   BLOB
--   F32_BLOB(D_INTEREST)


/* ------------------------------------------------------------
   1) Main "scores" table (fully flattened)
   ------------------------------------------------------------ */
CREATE TABLE IF NOT EXISTS scores (
  user_id   TEXT PRIMARY KEY,
  username  TEXT NOT NULL,

  -- Personality (HEXACO)
  honesty_humility        REAL NOT NULL,
  emotionality            REAL NOT NULL,
  extraversion            REAL NOT NULL,
  agreeableness           REAL NOT NULL,
  conscientiousness       REAL NOT NULL,
  openness_to_experience  REAL NOT NULL,

  -- Communication
  agency    REAL NOT NULL,
  communion REAL NOT NULL,

  -- Values (Schwartz)
  self_direction REAL NOT NULL,
  stimulation    REAL NOT NULL,
  hedonism       REAL NOT NULL,
  achievement    REAL NOT NULL,
  power          REAL NOT NULL,
  security       REAL NOT NULL,
  conformity     REAL NOT NULL,
  tradition      REAL NOT NULL,
  benevolence    REAL NOT NULL,
  universalism   REAL NOT NULL,

  -- Optional: score-level embedding
  introduction_embedding  BLOB,

  -- Timestamps (optional but useful)
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
