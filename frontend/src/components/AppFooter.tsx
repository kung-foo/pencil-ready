/**
 * Mirror of the worksheet footer (lib/footer.typ) so the web UI carries
 * the same attribution/branding the student sees on the printed page.
 */
export function AppFooter() {
  return (
    <footer className="border-t px-6 py-3 text-center text-xs text-muted-foreground">
      <span className="font-semibold">Pencil Ready</span>
      <span className="mx-2">—</span>
      made with{" "}
      <img
        src="/rainbow-heart.svg"
        alt="rainbow heart"
        className="inline size-4 align-middle -mt-0.5"
      />{" "}
      in Oslo
      <span className="mx-2">—</span>
      <a
        href="https://pencilready.com"
        className="hover:text-foreground hover:underline underline-offset-4"
      >
        pencilready.com
      </a>
    </footer>
  );
}
