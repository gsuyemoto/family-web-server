CREATE TABLE "users" (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    fname           TEXT NOT NULL,
    lname           TEXT,
    is_admin        INTEGER NOT NULL DEFAULT 1,
    num_bucks       INTEGER,
    date_created    TEXT NOT NULL
);
