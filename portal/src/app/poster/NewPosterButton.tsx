"use client";

import { useActionState } from "react";

export default function NewPosterButton({
  disabled,
  createPoster,
}: {
  disabled: boolean;
  createPoster(): void;
}) {
  const [_, formAction, pending] = useActionState(createPoster, null);

  console.log("rendered child");

  return (
    <form action={formAction}>
      <button
        className="btn px-6 btn-sm normal-case btn-primary"
        disabled={disabled || pending}
        type="submit"
      >
        New Poster
      </button>
    </form>
  );
}
