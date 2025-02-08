async function getHash(token: string): Promise<string> {
	const encoder = new TextEncoder().encode(token);
	const digest = await crypto.subtle.digest(
		{
			name: 'SHA-256',
		},
		encoder
	);
	const digestBytes = Array.from(new Uint8Array(digest));
	return digestBytes.map((b) => b.toString(16).padStart(2, '0')).join('');
}

type GetPosterForTokenResult =
	| {
			// The token was bound to a poster, and it had a texture.
			type: 'found';
			url: string;
	  }
	| {
			// The token was bound to a poster, but there was no texture.
			type: 'no-texture';
	  }
	| {
			// The token was not bound to a poster.
			type: 'no-poster';
	  };

async function getUrlForToken(db: D1Database, token: string, channel: string): Promise<GetPosterForTokenResult> {
	const hash = await getHash(token);

	const query = `
	INSERT INTO poster_tokens AS pt (hash, posterId)
		SELECT
		?1 AS hash,
			(
				SELECT id
				FROM posters
				WHERE servable
				ORDER BY RANDOM() LIMIT 1
			) AS posterId
	ON CONFLICT (hash)
		DO UPDATE SET lastUsed=CURRENT_TIMESTAMP
	RETURNING (
	SELECT url
		FROM poster_materials AS pm
		WHERE pm.posterId = posterId
		AND pm.channel = ?2
		) AS url,
		posterId IS NOT NULL AS has_poster
	;
	`;
	const results = await db.prepare(query).bind(hash, channel).first<{
		url: string | null;
		has_poster: 1 | 0;
	}>();

	console.log('Attempted to create or insert token', results);

	if (results == null) {
		throw 'Error loading poster query, it returned no rows which should not happen';
	}

	if (results.url) {
		return {
			type: 'found',
			url: results.url,
		};
	}
	return {
		type: results.has_poster ? 'no-texture' : 'no-poster',
	};
}

type Channels = 'a' | 'e' | 'n';
type BakedPoster = Record<Channels, string>;

const DefaultChannelPoster: BakedPoster = {
	a: 'black_pixel.png',
	e: 'black_pixel.png',
	n: 'default_normal_pixel.png',
};
const DefaultPoster: BakedPoster = {
	a: 'safe.webp',
	e: 'black_pixel.png',
	n: 'default_normal_pixel.png',
};
const ErrorPoster: BakedPoster = {
	a: 'error.webp',
	e: 'black_pixel.png',
	n: 'default_normal_pixel.png',
};
const YotePoster: BakedPoster = {
	a: 'yote_albedo.webp',
	e: 'yote_emissive.webp',
	n: 'yote_normal.webp',
};

async function BakedPosterRedirect(assets: Fetcher, poster: BakedPoster, channel: Channels): Promise<Response> {
	return assets.fetch('http://fakehost/' + poster[channel]);
}

async function getResponse(assets: Fetcher, token_result: GetPosterForTokenResult, channel: Channels): Promise<Response> {
	if (token_result.type == 'found') {
		return Response.redirect(token_result.url);
	}

	const static_file = await BakedPosterRedirect(assets, token_result.type == 'no-texture' ? DefaultChannelPoster : DefaultPoster, channel);

	return new Response(static_file.body, static_file);
}

export default {
	async fetch(req, env, ctx): Promise<Response> {
		const cacheKey = new Request(req.url);
		const cacheResp = await caches.default.match(cacheKey);
		if (cacheResp) {
			return cacheResp;
		}

		const url = new URL(req.url);
		const path = url.pathname.split('/');

		if (path.length < 3 || path[1] != 'redirect') {
			return new Response('Not found', {
				status: 404,
			});
		}
		const token = path[2];
		const channel = (path[3] as Channels) ?? 'a';

		// Validate we actually have a valid channel.
		if (!DefaultChannelPoster[channel]) {
			return await BakedPosterRedirect(env.ASSETS, DefaultChannelPoster, 'a');
		}

		// Yote is a clean check poster, it should always be available.
		if (token == 'yote') {
			return await BakedPosterRedirect(env.ASSETS, YotePoster, channel);
		}

		// Circuit breaker to turn off backend serving, only returning default posters.
		if (env.NO_BACKEND_SERVE.toUpperCase() == 'TRUE') {
			return await BakedPosterRedirect(env.ASSETS, DefaultPoster, channel);
		}

		try {
			const token_result = await getUrlForToken(env.DB, token, channel);

			const response = await getResponse(env.ASSETS, token_result, channel);

			response.headers.append('Cache-Control', 's-maxage=' + env.REDIRECT_RESPONSE_CACHE_SECONDS);
			ctx.waitUntil(caches.default.put(cacheKey, response.clone()));
			return response;
		} catch (except) {
			console.log(except);
			let response = await BakedPosterRedirect(env.ASSETS, ErrorPoster, channel);
			response = new Response(response.body, response);
			response.status = 500;
			return response;
		}
	},
} satisfies ExportedHandler<Env>;
