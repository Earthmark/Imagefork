import NextAuth from "next-auth";
import { D1Adapter } from "@auth/d1-adapter";
import GitHub from "next-auth/providers/github";
import { unauthorized } from "next/navigation";

export async function authUserId(){
  const session = await auth();
  const userId = session?.user?.id;
  if (!userId) {
    unauthorized();
  }
  return userId;
}

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [
    GitHub({
      profile(profile) {
        return {
          id: profile.id.toString(),
          // Use login, because actual names can be sensitive.
          name: profile.login,
          email: profile.email,
          image: profile.avatar_url,
        };
      },
    }),
  ],
  adapter: D1Adapter(process.env.DB),
  callbacks: {
    session({ session, user }) {
      session.user.id = user.id;
      return session;
    },
  },
  pages: {
    signIn: "/login",
  },
});
