import { auth, signIn } from "@/auth";
import { getPoster } from "@/db";

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
  await getPoster(posterId);

  return <></>;
}
