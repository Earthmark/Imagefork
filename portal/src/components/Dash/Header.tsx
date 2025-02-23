import { auth, signOut } from "@/auth";
import { getTranslations } from "next-intl/server";
import Link from "next/link";

export default async function Header() {
  const session = await auth();
  const t = await getTranslations("Dash");

  return (
    <div className="navbar sticky top-0 bg-base-100 z-10 shadow-md">
      <div className="flex-1">
        <Link href="/">
          <h1 className="text-2x1 font-semibold ml-2">{t("title")}</h1>
        </Link>
      <Link href="/poster" className="ml-6 btn btn-ghost">Posters</Link>
      </div>
      <div className="flex-none">
        <div className="dropdown dropdown-end ml-4">
          <label tabIndex={0} className="btn btn-ghost btn-circle avatar">
            <div className="w-10 rounded-full">
              <img src={session?.user?.image ?? ""} alt="profile" />
            </div>
          </label>
          <ul
            tabIndex={0}
            className="menu menu-compact dropdown-content mt-3 p-2 shadow bg-base-100 rounded-box w-52"
          >
            <li className="justify-between">
              <a
                onClick={async () => {
                  "use server";
                  await signOut({
                    redirectTo: "/",
                  });
                }}
              >
                {t("logout")}
              </a>
            </li>
          </ul>
        </div>
      </div>
    </div>
  );
}
