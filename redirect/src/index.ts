import { zValidator } from '@hono/zod-validator';
import { Hono } from 'hono';
import { z } from 'zod';

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

async function getResponseForToken(db: D1Database, assets: Fetcher, token: string, channel: Channels): Promise<Response> {
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
		return Response.redirect(results.url);
	}

	const static_file = await BakedPosterRedirect(assets, results.has_poster ? DefaultChannelPoster : DefaultPoster, channel);
	return new Response(static_file.body, static_file);
}

async function getResponseForId(db: D1Database, assets: Fetcher, id: string, channel: Channels): Promise<Response> {
	const query = `
	SELECT url
	FROM poster_materials AS pm
	WHERE pm.posterId = ?1
	  AND pm.channel = ?2
	;
	`;
	const results = await db.prepare(query).bind(id, channel).first<{
		url: string;
	}>();

	if (results != null) {
		return Response.redirect(results.url);
	}

	const static_file = await BakedPosterRedirect(assets, DefaultChannelPoster, channel);
	return new Response(static_file.body, static_file);
}

type Channels = 'a' | 'e' | 'n';
const channelsSchema = z.enum(['a', 'e', 'n']).default('a');

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

const app = new Hono<{
	Bindings: Env;
}>();

app.get(
	'/redirect/yote/:channel?',
	zValidator(
		'param',
		z.object({
			channel: channelsSchema,
		})
	),
	async ({ req, env }) => {
		const channel = req.valid('param').channel;
		// Yote is a clean check poster, it should always be available.
		return await BakedPosterRedirect(env.ASSETS, YotePoster, channel);
	}
);

app.get(
	'/redirect/:token/:channel?',
	zValidator(
		'param',
		z.object({
			token: z.string(),
			channel: channelsSchema,
		})
	),
	async ({ req, env, executionCtx }) => {
		const cacheKey = new Request(req.url);
		const cacheResp = await caches.default.match(cacheKey);
		if (cacheResp) {
			return cacheResp;
		}

		const token = req.valid('param').token;
		const channel = req.valid('param').channel;

		try {
			let response: Response | undefined;

			// Circuit breaker to turn off backend serving, only returning default posters.
			if (env.NO_BACKEND_SERVE.toUpperCase() == 'TRUE') {
				response = await BakedPosterRedirect(env.ASSETS, DefaultPoster, channel);
			} else {
				response = await getResponseForToken(env.DB, env.ASSETS, token, channel);
			}

			response.headers.append('Cache-Control', 's-maxage=' + env.REDIRECT_RESPONSE_CACHE_SECONDS);
			executionCtx.waitUntil(caches.default.put(cacheKey, response.clone()));
			return response;
		} catch (except) {
			console.log(except);
			let response = await BakedPosterRedirect(env.ASSETS, ErrorPoster, channel);
			response = new Response(response.body, response);
			response.status = 500;
			return response;
		}
	}
);

app.get(
	'/poster/:poster/:channel?',
	zValidator(
		'param',
		z.object({
			poster: z.string(),
			channel: channelsSchema,
		})
	),
	async ({ req, env }) => {
		const poster = req.valid('param').poster;
		const channel = req.valid('param').channel;

		try {
			return await getResponseForId(env.DB, env.ASSETS, poster, channel);
		} catch (except) {
			console.log(except);
			let response = await BakedPosterRedirect(env.ASSETS, ErrorPoster, channel);
			response = new Response(response.body, response);
			response.status = 500;
			return response;
		}
	}
);

export default app;
