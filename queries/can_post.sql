SELECT poster_limit > (
    SELECT COUNT(*)
    FROM Posters
    WHERE creator = id
  ) AS can_add
FROM Creators
WHERE id = 1
LIMIT 1