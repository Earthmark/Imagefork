

        INSERT INTO posters (userId)
        SELECT "047efc66-f81b-4eca-8a99-e6e87d8254a8"
        WHERE (
          SELECT (
            poster_limit - (
              SELECT COUNT(*)
              FROM posters
              WHERE userId = "047efc66-f81b-4eca-8a99-e6e87d8254a8"
            )) > 0
          FROM users
          WHERE id = "047efc66-f81b-4eca-8a99-e6e87d8254a8"
        )
        RETURNING
          id, creationTime, active, lockout, servable
