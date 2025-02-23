import React from "react";
import Header from "./Header";

export default function Dash({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col h-screen w-screen">
      <Header />
      <main className="flex-1 overflow-y-auto md:pt-4 pt-4 px-6 bg-base-200">
        {children}
      </main>
    </div>
  );
}
