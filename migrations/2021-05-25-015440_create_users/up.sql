CREATE TABLE users (
    user_id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL UNIQUE,
    points INTEGER NOT NULL,
    is_admin INTEGER NOT NULL
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL,
    nickname VARCHAR NOT NULL,
    addr_mac VARCHAR NOT NULL UNIQUE,
    addr_ip VARCHAR,
    manufacture_name VARCHAR,
    is_watching INTEGER NOT NULL,
    watch_start INTEGER
);
