import type { ExternalLink } from "./types";

type ExternalLinksProps = Readonly<{
  links: readonly ExternalLink[];
}>;

export function ExternalLinks({ links }: ExternalLinksProps) {
  if (links.length === 0) return null;

  return (
    <section className="external-links">
      <h5>Linear links</h5>
      <ul>
        {links.map((link) => (
          <li key={link.id}>
            <a href={link.url} rel="noreferrer" target="_blank">
              {link.displayIdentifier}
            </a>
            <span>{link.connectionMode.replaceAll("_", " ")}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
