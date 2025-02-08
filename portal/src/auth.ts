import NextAuth from "next-auth"
import { D1Adapter } from "@auth/d1-adapter"
 
export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [],
  adapter: D1Adapter(process.env.DB),
})
