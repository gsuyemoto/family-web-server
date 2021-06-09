CREATE TABLE users (
    user_id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL UNIQUE,
    points INTEGER NOT NULL CHECK(points >= 0),
    is_admin INTEGER NOT NULL
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(user_id),
    nickname VARCHAR NOT NULL,
    addr_mac VARCHAR NOT NULL UNIQUE,
    addr_ip VARCHAR NOT NULL,
    manufacture_name VARCHAR,
    is_watching INTEGER NOT NULL,
    is_blocked INTEGER NOT NULL,
    is_tracked INTEGER NOT NULL,
    watch_start INTEGER
);
