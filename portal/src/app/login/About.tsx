import { useTranslations } from "next-intl";

export default function About() {
  const t = useTranslations("About");

  return (
    <div className="hero min-h-full rounded-1-xl bg-base-200">
      <div className="hero-content py-12">
        <div className="max-w-md">
          <h1 className="text-3xl text-center font-bold">{t("title")}</h1>
          <h1 className="text-2xl mt-8 font-bold"> {t("subtitle")}</h1>
          <p>{t("announce-in-worlds")}</p>
        </div>
      </div>
    </div>
  );
}
