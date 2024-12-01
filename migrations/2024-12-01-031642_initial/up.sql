CREATE TABLE Creators (
  id BIGSERIAL PRIMARY KEY,
  creation_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  email TEXT NOT NULL UNIQUE,
  token TEXT NOT NULL UNIQUE DEFAULT '',
  minting_time TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
  lockout BOOLEAN NOT NULL DEFAULT FALSE,
  moderator BOOLEAN NOT NULL DEFAULT FALSE,
  poster_limit INTEGER NOT NULL DEFAULT 3
);

CREATE TABLE Posters (
  id BIGSERIAL PRIMARY KEY,
  creator BIGSERIAL NOT NULL REFERENCES Creators (id) ON DELETE CASCADE,
  creation_time TIMESTAMP NOT NULL DEFAULT (now() at time zone 'utc'),
  url TEXT NOT NULL,
  stopped BOOLEAN NOT NULL DEFAULT TRUE,
  lockout BOOLEAN NOT NULL DEFAULT FALSE,
  servable BOOLEAN NOT NULL GENERATED ALWAYS AS (
    NOT (
      stopped
      OR lockout
    )
  ) STORED
);

INSERT INTO Creators (id, email, poster_limit) VALUES (0, 'SYSTEM', 0);
INSERT INTO Posters (id, creator, url, stopped) VALUES (0, 0, 'ERROR', TRUE);
INSERT INTO Posters (id, creator, url, stopped) VALUES (1, 0, 'SAFE', TRUE);
