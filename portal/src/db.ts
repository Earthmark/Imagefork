"use server";

import { getRequestContext } from "@cloudflare/next-on-pages";
import { authUserId } from "./auth";

function db() {
  return getRequestContext().env.DB;
}

export type PosterMetadata = {
  id: number;
  userId: string;
  creationTime: string;
  active: boolean;
  lockout: boolean;
  servable: boolean;
};

export async function getPosters(): Promise<Array<PosterMetadata>> {
  const userId = await authUserId();

  const posterInfo = await db()
    .prepare(
      `
      SELECT
        id, creationTime, active, lockout, servable
      FROM posters AS p
      WHERE userId = ?1
      ORDER BY creationTime
      `
    )
    .bind(userId)
    .all<PosterMetadata>();
  return posterInfo.results;
}

export async function createPoster(): Promise<PosterMetadata | null> {
  const userId = await authUserId();

  const posterInfo = await db()
    .prepare(
      `
        INSERT INTO posters (userId)
        SELECT u.id AS userId
        FROM users AS u
        WHERE u.id = $1
        AND NOT u.lockout
        AND (
            u.poster_limit - (
                SELECT COUNT(*)
                FROM posters AS p
                WHERE p.userId = u.id
            )
        ) > 0
        RETURNING
            id, creationTime, active, lockout, servable
        `
    )
    .bind(userId)
    .first<PosterMetadata>();

  return posterInfo;
}

export async function deletePoster(posterId: string): Promise<boolean> {
  const userId = await authUserId();

  const result = await db()
    .prepare(
      `
        DELETE FROM posters 
        WHERE userId = ?1 AND id = ?2
      `
    )
    .bind(userId, posterId)
    .run();

  return result.success;
}

export type Channel = "a" | "e" | "n";

export type PosterMaterial = {
  channel: Channel;
  url: string;
};

export type PosterInfo = {
  materials: Array<PosterMaterial>;
} & PosterMetadata;

export async function getPoster(posterId: string): Promise<PosterInfo | null> {
  const userId = await authUserId();

  const posterInfoP = db()
    .prepare(
      `
      SELECT
        id, creationTime, active, lockout, servable
      FROM posters AS p
      WHERE id = ?1 AND userId = ?2
      `
    )
    .bind(posterId, userId)
    .first<PosterMetadata>();

  const posterMaterialsP = db()
    .prepare(
      `
      SELECT
        pm.channel, pm.url
      FROM posters AS p
      INNER JOIN poster_materials AS pm ON p.id = pm.posterId
      WHERE p.id = ?1 AND p.userId = ?2
      ORDER BY channel
      `
    )
    .bind(posterId, posterId)
    .all<PosterMaterial>();

  const posterInfo = await posterInfoP;
  const posterMaterials = await posterMaterialsP;

  if (posterInfo == null) {
    return null;
  }

  return {
    materials: posterMaterials.results,
    ...posterInfo,
  };
}
