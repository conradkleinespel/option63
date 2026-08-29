import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

const ELEMENTS = [
  { n: 1, s: "H", x: 1, y: 1 },
  { n: 2, s: "He", x: 18, y: 1 },
  { n: 3, s: "Li", x: 1, y: 2 },
  { n: 4, s: "Be", x: 2, y: 2 },
  { n: 5, s: "B", x: 13, y: 2 },
  { n: 6, s: "C", x: 14, y: 2 },
  { n: 7, s: "N", x: 15, y: 2 },
  { n: 8, s: "O", x: 16, y: 2 },
  { n: 9, s: "F", x: 17, y: 2 },
  { n: 10, s: "Ne", x: 18, y: 2 },
  { n: 11, s: "Na", x: 1, y: 3 },
  { n: 12, s: "Mg", x: 2, y: 3 },
  { n: 13, s: "Al", x: 13, y: 3 },
  { n: 14, s: "Si", x: 14, y: 3 },
  { n: 15, s: "P", x: 15, y: 3 },
  { n: 16, s: "S", x: 16, y: 3 },
  { n: 17, s: "Cl", x: 17, y: 3 },
  { n: 18, s: "Ar", x: 18, y: 3 },
  { n: 19, s: "K", x: 1, y: 4 },
  { n: 20, s: "Ca", x: 2, y: 4 },
  { n: 21, s: "Sc", x: 3, y: 4 },
  { n: 22, s: "Ti", x: 4, y: 4 },
  { n: 23, s: "V", x: 5, y: 4 },
  { n: 24, s: "Cr", x: 6, y: 4 },
  { n: 25, s: "Mn", x: 7, y: 4 },
  { n: 26, s: "Fe", x: 8, y: 4 },
  { n: 27, s: "Co", x: 9, y: 4 },
  { n: 28, s: "Ni", x: 10, y: 4 },
  { n: 29, s: "Cu", x: 11, y: 4 },
  { n: 30, s: "Zn", x: 12, y: 4 },
  { n: 31, s: "Ga", x: 13, y: 4 },
  { n: 32, s: "Ge", x: 14, y: 4 },
  { n: 33, s: "As", x: 15, y: 4 },
  { n: 34, s: "Se", x: 16, y: 4 },
  { n: 35, s: "Br", x: 17, y: 4 },
  { n: 36, s: "Kr", x: 18, y: 4 },
  { n: 37, s: "Rb", x: 1, y: 5 },
  { n: 38, s: "Sr", x: 2, y: 5 },
  { n: 39, s: "Y", x: 3, y: 5 },
  { n: 40, s: "Zr", x: 4, y: 5 },
  { n: 41, s: "Nb", x: 5, y: 5 },
  { n: 42, s: "Mo", x: 6, y: 5 },
  { n: 43, s: "Tc", x: 7, y: 5 },
  { n: 44, s: "Ru", x: 8, y: 5 },
  { n: 45, s: "Rh", x: 9, y: 5 },
  { n: 46, s: "Pd", x: 10, y: 5 },
  { n: 47, s: "Ag", x: 11, y: 5 },
  { n: 48, s: "Cd", x: 12, y: 5 },
  { n: 49, s: "In", x: 13, y: 5 },
  { n: 50, s: "Sn", x: 14, y: 5 },
  { n: 51, s: "Sb", x: 15, y: 5 },
  { n: 52, s: "Te", x: 16, y: 5 },
  { n: 53, s: "I", x: 17, y: 5 },
  { n: 54, s: "Xe", x: 18, y: 5 },
  { n: 55, s: "Cs", x: 1, y: 6 },
  { n: 56, s: "Ba", x: 2, y: 6 },
  { n: 57, s: "La", x: 3, y: 9 },
  { n: 58, s: "Ce", x: 4, y: 9 },
  { n: 59, s: "Pr", x: 5, y: 9 },
  { n: 60, s: "Nd", x: 6, y: 9 },
  { n: 61, s: "Pm", x: 7, y: 9 },
  { n: 62, s: "Sm", x: 8, y: 9 },
  { n: 63, s: "Eu", x: 9, y: 9 },
  { n: 64, s: "Gd", x: 10, y: 9 },
  { n: 65, s: "Tb", x: 11, y: 9 },
  { n: 66, s: "Dy", x: 12, y: 9 },
  { n: 67, s: "Ho", x: 13, y: 9 },
  { n: 68, s: "Er", x: 14, y: 9 },
  { n: 69, s: "Tm", x: 15, y: 9 },
  { n: 70, s: "Yb", x: 16, y: 9 },
  { n: 71, s: "Lu", x: 17, y: 9 },
  { n: 72, s: "Hf", x: 4, y: 6 },
  { n: 73, s: "Ta", x: 5, y: 6 },
  { n: 74, s: "W", x: 6, y: 6 },
  { n: 75, s: "Re", x: 7, y: 6 },
  { n: 76, s: "Os", x: 8, y: 6 },
  { n: 77, s: "Ir", x: 9, y: 6 },
  { n: 78, s: "Pt", x: 10, y: 6 },
  { n: 79, s: "Au", x: 11, y: 6 },
  { n: 80, s: "Hg", x: 12, y: 6 },
  { n: 81, s: "Tl", x: 13, y: 6 },
  { n: 82, s: "Pb", x: 14, y: 6 },
  { n: 83, s: "Bi", x: 15, y: 6 },
  { n: 84, s: "Po", x: 16, y: 6 },
  { n: 85, s: "At", x: 17, y: 6 },
  { n: 86, s: "Rn", x: 18, y: 6 },
  { n: 87, s: "Fr", x: 1, y: 7 },
  { n: 88, s: "Ra", x: 2, y: 7 },
  { n: 89, s: "Ac", x: 3, y: 10 },
  { n: 90, s: "Th", x: 4, y: 10 },
  { n: 91, s: "Pa", x: 5, y: 10 },
  { n: 92, s: "U", x: 6, y: 10 },
  { n: 93, s: "Np", x: 7, y: 10 },
  { n: 94, s: "Pu", x: 8, y: 10 },
  { n: 95, s: "Am", x: 9, y: 10 },
  { n: 96, s: "Cm", x: 10, y: 10 },
  { n: 97, s: "Bk", x: 11, y: 10 },
  { n: 98, s: "Cf", x: 12, y: 10 },
  { n: 99, s: "Es", x: 13, y: 10 },
  { n: 100, s: "Fm", x: 14, y: 10 },
  { n: 101, s: "Md", x: 15, y: 10 },
  { n: 102, s: "No", x: 16, y: 10 },
  { n: 103, s: "Lr", x: 17, y: 10 },
  { n: 104, s: "Rf", x: 4, y: 7 },
  { n: 105, s: "Db", x: 5, y: 7 },
  { n: 106, s: "Sg", x: 6, y: 7 },
  { n: 107, s: "Bh", x: 7, y: 7 },
  { n: 108, s: "Hs", x: 8, y: 7 },
  { n: 109, s: "Mt", x: 9, y: 7 },
  { n: 110, s: "Ds", x: 10, y: 7 },
  { n: 111, s: "Rg", x: 11, y: 7 },
  { n: 112, s: "Cn", x: 12, y: 7 },
  { n: 113, s: "Nh", x: 13, y: 7 },
  { n: 114, s: "Fl", x: 14, y: 7 },
  { n: 115, s: "Mc", x: 15, y: 7 },
  { n: 116, s: "Lv", x: 16, y: 7 },
  { n: 117, s: "Ts", x: 17, y: 7 },
  { n: 118, s: "Og", x: 18, y: 7 },
];

function ElementCell({ el }) {
  const isEu = el.n === 63;

  return (
    <div
      className="relative border border-line group cursor-default bg-paper"
      style={{ gridColumn: el.x, gridRow: el.y }}
    >
      <div className="flex flex-col items-center justify-center w-full h-full py-[2px]">
        <span className="text-[8px] leading-none text-gray-500">{el.n}</span>
        <span className="text-sm font-bold leading-none text-ink">
          {el.s}
        </span>
      </div>
      {isEu && (
        <div className="absolute inset-0 border-2 border-ink rounded-sm pointer-events-none" />
      )}
    </div>
  );
}

function PeriodicTable() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const btn = document.getElementById("why-link");
    if (!btn) return;
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      setOpen(true);
    });
    return () => btn.removeEventListener("click", () => {});
  }, []);

  useEffect(() => {
    if (!open) return;
    document.body.style.overflow = "hidden";
    const onKey = (e) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    };
    document.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = "";
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <>
      {open && (
        <div
          className="fixed inset-0 z-50"
          role="dialog"
          aria-modal="true"
          aria-label="Periodic table"
        >
          <div
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={() => setOpen(false)}
          />
          <div className="relative min-h-screen flex items-start justify-center py-8 px-4 overflow-y-auto pointer-events-none"
            >
              <div className="relative bg-paper border border-line rounded-xl shadow-2xl max-w-5xl w-full my-auto z-10 pointer-events-auto">
              <button
                onClick={() => setOpen(false)}
                className="absolute top-4 right-4 p-2 hover:bg-gray-100 rounded-lg transition-colors cursor-pointer"
                aria-label="Close"
              >
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
              <div className="flex items-start justify-between px-6 pt-5 pb-3">
                <div>
                  <h2 className="text-xl font-bold tracking-tight">
                    Why "option63"?
                  </h2>
                  <p className="text-sm text-gray-600 mt-1">
                    The 63rd element is interesting for several reasons. You can pick any one you like.{" "}
                    <a
                      href="https://en.wikipedia.org/wiki/Europium"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-ink underline underline-offset-2 decoration-line-through"
                    >
                      Read more
                    </a>
                  </p>
                </div>
              </div>

              <div className="px-6 pb-6">
                <div className="overflow-x-auto -mx-6 px-6">
                  <div
                    className="grid min-w-[600px]"
                    style={{
                      gridTemplateColumns: "repeat(18, minmax(0, 1fr))",
                      gridTemplateRows: "repeat(8, auto)",
                      gap: "0px",
                    }}
                  >
                    {ELEMENTS.map((el) => (
                      <ElementCell key={el.n} el={el} />
                    ))}

                    {/* Spacer row label */}
                    <div
                      className="col-span-18 flex items-center gap-4 py-1"
                      style={{ gridColumn: 1, gridRow: 8 }}
                    >
                      <span className="font-mono text-[10px] text-gray-400 w-4 text-right">
                        57–71
                      </span>
                      <span className="text-gray-300">▾</span>
                      <span className="font-mono text-[10px] text-gray-400 w-4 text-right">
                        89–103
                      </span>
                      <span className="text-gray-300">▾</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

const container = document.getElementById("why-root");
if (container) {
  createRoot(container).render(<PeriodicTable />);
}