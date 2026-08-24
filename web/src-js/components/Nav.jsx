import { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";

const LINKS = [
  { href: "/shadow-it/", label: "Shadow IT" },
  { href: "/personal/", label: "Personal" },
  { href: "/ai/", label: "AI" },
];

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
          </nav>
        </div>
      )}
    </div>
  );
}

createRoot(document.getElementById("nav-root")).render(<Nav />);
