import { useTranslations } from "next-intl";

export default function Loading() {
  const t = useTranslations("Loading");

  return (
    <div className="w-full h-screen text-gray-300 dark:text-gray-200 gb-base-100 text-center">
      {t("content")}
    </div>
  );
}
