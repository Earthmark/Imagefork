import type { Metadata } from "next";
import "./globals.css";
import { Inter } from "next/font/google";
import { NextIntlClientProvider } from "next-intl";
import { getLocale, getMessages } from "next-intl/server";
import { auth } from "@/auth";
import Login from "@/components/Login";
import Dash from "@/components/Dash";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "Imagefork",
  description: "A poster network for Resonite",
};

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const locale = await getLocale();
  const messages = await getMessages();
  const session = await auth();

  return (
    <html lang={locale}>
      <body className={inter.className}>
        <NextIntlClientProvider messages={messages}>
          {session?.user?.id ? <Dash>{children}</Dash> : <Login />}
        </NextIntlClientProvider>
      </body>
    </html>
  );
}

export const runtime = "edge";
