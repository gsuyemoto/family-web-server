CREATE TABLE users (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    points INTEGER NOT NULL,
    is_admin INTEGER NOT NULL
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    addr_mac VARCHAR NOT NULL,
    addr_ip VARCHAR,
    device VARCHAR,
    is_watching INTEGER NOT NULL,
    watch_start VARCHAR
);
