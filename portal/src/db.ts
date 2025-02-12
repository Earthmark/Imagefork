import { authUserId } from "./auth";

type PosterMetadata = {
  id: number;
  userId: string;
  creationTime: string;
  active: boolean;
  lockout: boolean;
  servable: boolean;
};

export async function getPosters(
  db: D1Database
): Promise<Array<PosterMetadata>> {
  const userId = await authUserId();

  if (userId == undefined) {
    return [];
  }

  const posterInfo = await db
    .prepare(
      `
      SELECT
        id, creationTime, active, lockout, servable
      FROM posters AS p
      WHERE userId = ?1
      `
    )
    .bind(userId)
    .all<PosterMetadata>();
  return posterInfo.results;
}

export async function createPoster(
  db: D1Database
): Promise<PosterMetadata | null> {
  const userId = await authUserId();

  if (userId == undefined) {
    return null;
  }

  const posterInfo = await db
    .prepare(
      `
        INSERT INTO posters (userId)
        SELECT ?1
        WHERE (
          SELECT (
            poster_limit - (
              SELECT COUNT(*)
              FROM posters
              WHERE userId = ?1
            ) > 0
          FROM users
          WHERE id = ?1
        )
        RETURNING
          id, creationTime, active, lockout, servable
        `
    )
    .bind(userId)
    .first<PosterMetadata>();

  return posterInfo;
}

export async function deletePoster(
  db: D1Database,
  posterId: string
): Promise<boolean> {
  const userId = await authUserId();

  if (userId == undefined) {
    return false;
  }

  const result = await db
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

type Channel = "a" | "e" | "n";

type PosterMaterial = {
  channel: Channel;
  url: string;
};

type PosterInfo = {
  materials: Array<PosterMaterial>;
} & PosterMetadata;

export async function getPoster(
  db: D1Database,
  posterId: string
): Promise<PosterInfo | null> {
  const userId = await authUserId();

  if (userId == undefined) {
    return null;
  }

  const posterInfoP = db
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

  const posterMaterialsP = db
    .prepare(
      `
      SELECT
        pm.channel, pm.url
      FROM posters AS p
      INNER JOIN poster_materials AS pm ON p.id = pm.posterId
      WHERE p.id = ?1 AND p.userId = ?2
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
