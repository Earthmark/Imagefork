import { signIn } from "@/auth";
import { useTranslations } from "next-intl";

export function Login() {
  const t = useTranslations("Login");

  return (
    <div className="min-h-screen bg-base-200 flex items-center">
      <div className="card mx-auto w-full max-w-5xl shadow-xl">
        <div className="grid md:grid-cols-2 grid-cols-1 bg-base-100 rounded-x1">
          <div className="">
            <div className="hero min-h-full rounded-1-xl bg-base-200">
              <div className="hero-content py-12">
                <div className="max-w-md">
                  <h1 className="text-3xl text-center font-bold">
                    {t("title")}
                  </h1>
                  <h1 className="text-2xl mt-8 font-bold"> {t("subtitle")}</h1>
                  <p>{t("announce-in-worlds")}</p>
                </div>
              </div>
            </div>
          </div>
          <div className="py-24 px-10">
            <h2 className="text-2x1 font-semibold bm-2 text-center">
              {t("login")}
            </h2>
            <form
              action={async () => {
                "use server";
                await signIn("github");
              }}
            >
              <button type="submit" className="btn mt-2 w-full btn-primary">
                {t("login-via-github")}
              </button>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Login;
