CREATE TABLE Creators (
  id BIGSERIAL PRIMARY KEY,
  creation_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  email TEXT NOT NULL UNIQUE,
  token TEXT UNIQUE,
  minting_time TIMESTAMP,
  lockout BOOLEAN NOT NULL DEFAULT FALSE,
  moderator BOOLEAN NOT NULL DEFAULT FALSE,
  poster_limit INTEGER NOT NULL DEFAULT 3
);

CREATE TABLE Posters (
  id BIGSERIAL PRIMARY KEY,
  creator BIGSERIAL NOT NULL,
  creation_time TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
  url TEXT NOT NULL,
  height INTEGER NOT NULL DEFAULT 0,
  width INTEGER NOT NULL DEFAULT 0,
  hash TEXT NOT NULL DEFAULT '',
  dead_url BOOLEAN NOT NULL DEFAULT FALSE,
  life_last_checked TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
  start_time TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
  end_time TIMESTAMP,
  stopped BOOLEAN NOT NULL DEFAULT FALSE,
  lockout BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY(Creator) REFERENCES Creators(id)
);
