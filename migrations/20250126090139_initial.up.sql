CREATE TABLE users (
    id SERIAL,
    name VARCHAR(255),
    email VARCHAR(255),
    image TEXT,
    "emailVerified" TIMESTAMPTZ,
    PRIMARY KEY (id)
);
CREATE TABLE accounts (
    id SERIAL,
    "userId" INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    type VARCHAR(255) NOT NULL,
    provider VARCHAR(255) NOT NULL,
    "providerAccountId" VARCHAR(255) NOT NULL,
    refresh_token TEXT,
    access_token TEXT,
    expires_at BIGINT,
    id_token TEXT,
    scope TEXT,
    session_state TEXT,
    token_type TEXT,
    PRIMARY KEY (id)
);
CREATE TABLE sessions (
    id SERIAL,
    "userId" INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires TIMESTAMPTZ NOT NULL,
    sessionToken VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE posters (
    id BIGSERIAL PRIMARY KEY,
    "userId" BIGSERIAL NOT NULL REFERENCES creators (id) ON DELETE CASCADE,
    creation_time TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
    stopped BOOLEAN NOT NULL DEFAULT TRUE,
    lockout BOOLEAN NOT NULL DEFAULT FALSE,
    servable BOOLEAN NOT NULL GENERATED ALWAYS AS (
        NOT (
            stopped
            OR lockout
        )
    ) STORED
);
CREATE TYPE texture_kind AS ENUM ('albedo', 'emissive', 'normal');
CREATE TABLE poster_image (
    poster BIGSERIAL NOT NULL REFERENCES posters (id) ON DELETE CASCADE,
    kind texture_kind NOT NULL DEFAULT 'albedo',
    url TEXT NOT NULL,
    PRIMARY KEY (poster, kind)
);
