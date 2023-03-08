INSERT INTO Posters (Creator, Url, Height, Width, Hash)
SELECT 1, 'https://tacos.txt', 1920, 1080, 'AAAAAAA'
WHERE (SELECT COUNT(*) FROM Posters WHERE Creator = 1) < (SELECT PosterLimit FROM Creators WHERE ID = 1 LIMIT 1)
RETURNING *;
