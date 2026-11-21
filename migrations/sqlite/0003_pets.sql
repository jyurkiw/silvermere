-- ============================================================================
-- Fork: pets (design/warden.md)
-- ============================================================================

-- A character's pets: a Warden's companion now; summoned pets and pets with their own hit
-- points later. Keyed by account, as characters are (one character per account). Pet kinds
-- are content (content/pets.toml), not a table.
CREATE TABLE pets (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,                                              -- surrogate key
    account_id INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE, -- the owner's account
    source     TEXT NOT NULL,                                    -- where it came from: class, spell, item, quest
    kind       INTEGER NOT NULL,                                 -- a pet kind's number (content/pets.toml)
    name       TEXT NOT NULL,                                    -- its name; empty until named
    summoned   INTEGER NOT NULL,                                 -- 1 while it's out with its owner
    immortal   INTEGER NOT NULL,                                 -- 1 when it can't die (dead is then ignored)
    dead       INTEGER NOT NULL,                                 -- 1 once killed, until raised
    UNIQUE (account_id, source)
);
