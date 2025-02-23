import { getPosters, createPoster, PosterMetadata } from "@/db";
import { getRequestContext } from "@cloudflare/next-on-pages";
import NewPosterButton from "./NewPosterButton";
import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import Link from "next/link";

function StatusBadge({ poster }: { poster: PosterMetadata }) {
  return poster.lockout ? (
    <div className="badge badge-warning">Blocked</div>
  ) : poster.servable ? (
    <div className="badge badge-success">Active</div>
  ) : (
    <div className="badge badge-neutral">Paused</div>
  );
}

export default async function Page() {
  const posters = await getPosters();
  const posterLimit = getRequestContext().env.POSTER_LIMIT;

  return (
    <div className="card w-full p-6 bg-base-100 shadow-xl mt-2">
      <div className="text-xl font-semibold inline-block">
        Posters{" "}
        <div className="text-base font-normal inline-block">
          {posters.length}/{posterLimit}
        </div>
        <div className="inline-block float-right">
          <NewPosterButton
            disabled={posters.length >= posterLimit}
            createPoster={async () => {
              "use server";
              const poster = await createPoster();
              if (poster?.id) {
                revalidatePath("/poster");
                redirect(`/poster/${poster.id}`);
              }
            }}
          />
        </div>
      </div>
      <div className="divider mt-2" />
      <div className="h-full w-full pb-6 bg-base-100">
        <div className="overflow-x-auto w-full">
          <table className="table w-full">
            <thead>
              <tr>
                <th>Id</th>
                <th>Created</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {posters.map((poster) => (
                <tr key={poster.id}>
                  <td>{poster.id}</td>
                  <td>{poster.creationTime}</td>
                  <td>
                    <StatusBadge poster={poster} />
                  </td>
                  <td>
                    <Link
                      href={`/poster/${poster.id}`}
                      className="btn btn-square btn-ghost"
                    >
                      Edit
                    </Link>
                  </td>
                </tr>
              ))}
              <tr></tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
