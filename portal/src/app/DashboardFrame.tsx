import { Suspense } from "react";
import Header from "./Header";
import Loading from "./Loading";
import PosterViewerLazy from "./PosterViewerLazy";

export default function Dashboard() {
  return (
    <div className="drawer-content flex flex-col">
      <Header />
      <main className="flex-1 overflow-y-auto md:pt-4 pt-4 px-6 bg-base-200">
        <Suspense fallback={<Loading />}>{<PosterViewerLazy />}</Suspense>
      </main>
    </div>
  );
}
