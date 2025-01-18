CREATE TABLE crsf_tokens (
  token VARCHAR PRIMARY KEY,
  crsf VARCHAR NOT NULL,
  creation_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc')
);
CREATE TABLE creator_sessions (
  creator BIGSERIAL NOT NULL REFERENCES Creators (id) ON DELETE CASCADE,
  token VARCHAR NOT NULL,
  creation_time TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (now() at time zone 'utc'),
  PRIMARY KEY (creator, token)
);
