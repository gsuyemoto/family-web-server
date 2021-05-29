CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    username VARCHAR NOT NULL,
    points INTEGER NOT NULL,
    is_admin INTEGER NOT NULL
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY NOT NULL,
    username VARCHAR NOT NULL,
    addr_mac VARCHAR NOT NULL,
    addr_ip VARCHAR,
    name VARCHAR,
    is_watching INTEGER NOT NULL,
    watch_start VARCHAR
);
