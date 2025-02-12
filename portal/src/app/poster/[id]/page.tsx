import { auth, signIn } from "@/auth";
import { getPoster } from "@/db";
import { getRequestContext } from "@cloudflare/next-on-pages";

export default async function Page({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const userId = (await auth())?.user?.id;

  if (userId == undefined) {
    await signIn();
    return <></>;
  }

  const posterId = (await params).id;
  const { env } = getRequestContext();

  const p = getPoster(env.DB, posterId);

  return <></>;
}
