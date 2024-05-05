-- CbTable
CREATE TABLE IF NOT EXISTS cb_table (  
    id TEXT PRIMARY KEY,
    update_ts INTEGER NOT NULL,
    version INTEGER NOT NULL
 );
