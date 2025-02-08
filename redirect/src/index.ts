import { Redis } from '@upstash/redis/cloudflare';

type Channels = 'a' | 'e' | 'n';
type PosterId = number;
type BakedPoster = Record<Channels, string>;

const FallbackPoster: BakedPoster = {
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

class RedirectWorker {
	env: Env;
	redis: Redis;
	ex: number;
	constructor(env: Env) {
		this.env = env;
		this.redis = Redis.fromEnv(env);
		this.ex = parseInt(env.REDIRECT_CACHE_SECONDS);
	}

	async GetCachedPoster(hash: string): Promise<number | null> {
		return await this.redis.getex(hash, {
			ex: this.ex,
		});
	}

	async TrySetCachedPoster(hash: string, id: number): Promise<PosterId> {
		const result = (await this.redis.set(hash, id, {
			nx: true,
			get: true,
			ex: this.ex,
		})) as number | null;
		return result == null ? id : result;
	}

	async GetProposedPoster(): Promise<PosterId | null> {
		return await this.env.DB.prepare('SELECT id FROM posters WHERE servable').bind().first<number>('id');
	}

	async GetTexture(id: number, channel: string): Promise<string | null> {
		return await this.env.DB.prepare('SELECT url FROM poster_materials WHERE id = ? AND channel = ?').bind(id, channel).first<string>();
	}

	async GetHash(token: string): Promise<string> {
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

	async GetIdForToken(token: string): Promise<number | null> {
		const hash = await this.GetHash(token);

		let id = await this.GetCachedPoster(hash);
		if (id == null) {
			id = await this.GetProposedPoster();
			if (id != null) {
				id = await this.TrySetCachedPoster(hash, id);
			}
		}

		return id;
	}

	BakedPosterRedirect(poster: BakedPoster, variant: Channels): Promise<Response> {
		return this.env.ASSETS.fetch("http://fakehost/" + poster[variant]);
	}
}

export default {
	async fetch(req, env, ctx): Promise<Response> {
		const worker = new RedirectWorker(env);

		const url = new URL(req.url);
		const path = url.pathname.split('/');

		if (path.length < 3 || path[1] != 'redirect') {
			return new Response('Not found', {
				status: 404,
			});
		}
		const token = path[2];
		const channel = path[3] as Channels ?? "a";

		// Validate we actually have a valid channel.
		if (!DefaultPoster[channel]) {
			return worker.BakedPosterRedirect(FallbackPoster, 'a');
		}

		if (token == 'yote') {
			return worker.BakedPosterRedirect(YotePoster, channel);
		}

		try {
			const id = await worker.GetIdForToken(token);

			if (id != null) {
				const poster = await worker.GetTexture(id, channel);
				if (poster != null) {
					return Response.redirect(poster);
				}
				return worker.BakedPosterRedirect(DefaultPoster, channel);
			}

			return worker.BakedPosterRedirect(FallbackPoster, channel);
		} catch (except) {
			console.log(except);
			return worker.BakedPosterRedirect(ErrorPoster, channel);
		}
	},
} satisfies ExportedHandler<Env>;
