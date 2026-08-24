import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";

const LINKS = [
  { href: "/shadow-it/", label: "Shadow IT" },
  { href: "/personal/", label: "Personal" },
  { href: "/ai/", label: "AI" },
];

const GITHUB_URL = "https://github.com/conradkleinespel/option63";

function usePathname() {
  const [path, setPath] = useState(() =>
    typeof window === "undefined" ? "" : window.location.pathname
  );
  useEffect(() => {
    const onChange = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onChange);
    window.addEventListener("pageshow", onChange);
    return () => {
      window.removeEventListener("popstate", onChange);
      window.removeEventListener("pageshow", onChange);
    };
  }, []);
  return path;
}

function Nav() {
  const [open, setOpen] = useState(false);
  const path = usePathname();
  const wrapperRef = useRef(null);
  const buttonRef = useRef(null);

  const isActive = (href) =>
    href === "/" ? path === "/" : path.startsWith(href);

  useEffect(() => {
    if (!open) return;
    const onKey = (e) => {
      if (e.key === "Escape") {
        setOpen(false);
        buttonRef.current?.focus();
      }
    };
    const onClick = (e) => {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target)) {
        setOpen(false);
      }
    };
    document.addEventListener("keydown", onKey);
    document.addEventListener("click", onClick);
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("click", onClick);
    };
  }, [open]);

  useEffect(() => {
    if (open) wrapperRef.current?.querySelector("a")?.focus();
  }, [open]);

  const linkClasses = (active) =>
    `transition-colors ${
      active ? "text-ink" : "text-gray-600 hover:text-ash"
    }`;

  return (
    <div ref={wrapperRef} className="relative">
      <nav className="hidden md:flex items-center gap-8 text-sm font-medium">
        {LINKS.map((l) => (
          <a key={l.href} href={l.href} className={linkClasses(isActive(l.href))}>
            {l.label}
          </a>
        ))}
        <a href={GITHUB_URL} target="_blank" rel="noopener noreferrer" className="flex items-center gap-1.5 text-gray-600 hover:text-ash transition-colors">
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
          GitHub
        </a>
      </nav>

      <button
        ref={buttonRef}
        onClick={() => setOpen((o) => !o)}
        className="md:hidden p-2 -mr-2"
        aria-label={open ? "Close navigation" : "Open navigation"}
        aria-expanded={open}
        aria-controls="mobile-menu"
      >
        <svg
          className="w-6 h-6"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          viewBox="0 0 24 24"
        >
          {open ? (
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 6l12 12M6 18L18 6" />
          ) : (
            <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
          )}
        </svg>
      </button>

      {open && (
        <div
          id="mobile-menu"
          className="md:hidden absolute right-0 top-full mt-2 w-60 rounded-lg border border-line bg-paper shadow-lg py-2 z-50"
        >
          <nav className="flex flex-col text-sm font-medium">
            {LINKS.map((l) => {
              const active = isActive(l.href);
              return (
                <a
                  key={l.href}
                  href={l.href}
                  onClick={() => setOpen(false)}
                  className={`px-4 py-2 transition-colors ${
                    active
                      ? "text-ink bg-gray-50"
                      : "text-gray-600 hover:text-ash hover:bg-gray-50"
                  }`}
                >
                  {l.label}
                </a>
              );
            })}
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              onClick={() => setOpen(false)}
              className="px-4 py-2 flex items-center gap-2 text-gray-600 hover:text-ash hover:bg-gray-50 transition-colors"
            >
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
              GitHub
            </a>
          </nav>
        </div>
      )}
    </div>
  );
}

createRoot(document.getElementById("nav-root")).render(<Nav />);
