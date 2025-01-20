CREATE TABLE creators (
  id BIGSERIAL PRIMARY KEY,
  creation_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  email TEXT NOT NULL UNIQUE,
  email_hash BYTEA NOT NULL,
  lockout BOOLEAN NOT NULL DEFAULT FALSE,
  moderator BOOLEAN NOT NULL DEFAULT FALSE,
  poster_limit INTEGER NOT NULL DEFAULT 3
);
CREATE TABLE posters (
  id BIGSERIAL PRIMARY KEY,
  creator BIGSERIAL NOT NULL REFERENCES creators (id) ON DELETE CASCADE,
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
  poster_id BIGSERIAL NOT NULL REFERENCES posters (id) ON DELETE CASCADE,
  kind texture_kind NOT NULL DEFAULT 'albedo',
  url TEXT NOT NULL,
  PRIMARY KEY (poster_id, kind)
);