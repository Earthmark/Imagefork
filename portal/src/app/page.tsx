import { auth } from "@/auth";
import Dashboard from "./Dashboard";

export default async function Home() {
  const session = await auth();
  return <Dashboard />;
}
