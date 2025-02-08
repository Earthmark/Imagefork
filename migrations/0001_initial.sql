-- Migration number: 0001 	 2025-02-08T05:05:08.258Z
CREATE TABLE IF NOT EXISTS "users" (
    "id" INTEGER NOT NULL DEFAULT '',
    "name" TEXT DEFAULT NULL,
    "email" TEXT DEFAULT NULL,
    "emailVerified" DATETIME DEFAULT NULL,
    "image" TEXT DEFAULT NULL,
    PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS "accounts" (
    "id" INTEGER NOT NULL,
    "userId" INTEGER NOT NULL DEFAULT NULL,
    "type" TEXT NOT NULL DEFAULT NULL,
    "provider" TEXT NOT NULL DEFAULT NULL,
    "providerAccountId" TEXT NOT NULL DEFAULT NULL,
    "refresh_token" TEXT DEFAULT NULL,
    "access_token" TEXT DEFAULT NULL,
    "expires_at" NUMBER DEFAULT NULL,
    "token_type" TEXT DEFAULT NULL,
    "scope" TEXT DEFAULT NULL,
    "id_token" TEXT DEFAULT NULL,
    "session_state" TEXT DEFAULT NULL,
    "oauth_token_secret" TEXT DEFAULT NULL,
    "oauth_token" TEXT DEFAULT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "sessions" (
    "id" INTEGER NOT NULL,
    "sessionToken" TEXT NOT NULL,
    "userId" INTEGER NOT NULL DEFAULT NULL,
    "expires" DATETIME NOT NULL DEFAULT NULL,
    PRIMARY KEY (sessionToken),
    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS "verification_tokens" (
    "identifier" TEXT NOT NULL,
    "token" TEXT NOT NULL DEFAULT NULL,
    "expires" DATETIME NOT NULL DEFAULT NULL,
    PRIMARY KEY (token)
);
CREATE TABLE "posters" (
    "id" INTEGER NOT NULL,
    "userId" INTEGER NOT NULL,
    "creationTime" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "active" boolean NOT NULL DEFAULT FALSE,
    "lockout" boolean NOT NULL DEFAULT FALSE,
    "servable" BOOLEAN NOT NULL GENERATED ALWAYS AS (
        active
        AND (NOT lockout)
    ) STORED,
    PRIMARY KEY (id),
    FOREIGN KEY (userId) REFERENCES users(id) ON DELETE CASCADE
);
CREATE TABLE "poster_materials" (
    "posterId" INTEGER NOT NULL,
    "channel" TEXT DEFAULT 'albedo',
    "url" TEXT NOT NULL,
    PRIMARY KEY (posterId, channel),
    FOREIGN KEY (posterId) REFERENCES posters(id) ON DELETE CASCADE
);
CREATE TABLE "poster_tokens" (
    "posterId" INTEGER,
    "hash" TEXT NOT NULL,
    "lastUsed" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(hash),
    FOREIGN KEY(posterId) REFERENCES posters(id) ON DELETE SET NULL
);