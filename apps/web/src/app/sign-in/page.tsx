import { Suspense } from "react";
import SignInClient from "./sign-in-client";

export const metadata = { title: "Sign in | DOODLE" };

export default async function SignInPage({
  searchParams,
}: {
  searchParams: Promise<{ role?: string }>;
}) {
  const { role } = await searchParams;
  const resolvedRole = role === "client" ? "client" : "freelancer";
  return (
    <Suspense>
      <SignInClient role={resolvedRole} />
    </Suspense>
  );
}
