import { useState } from "react";

export default function DemoTransform({ inputVcard, defaultProps, accentLabel }) {
  const [vcard, setVcard] = useState(inputVcard);
  const [props, setProps] = useState(defaultProps);
  const [lines, setLines] = useState(null);
  const [error, setError] = useState(null);
  const [parsing, setParsing] = useState(false);

  async function run(e) {
    e.preventDefault();
    setParsing(true);
    setError(null);
    setLines(null);
    const parsedProps = props
      .split(",")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    try {
      const res = await fetch("/api/try/parse", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vcard, props: parsedProps }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || "Parse failed");
      setLines(data.lines || []);
    } catch (err) {
      setError(err.message);
    } finally {
      setParsing(false);
    }
  }

  return (
    <div className="grid lg:grid-cols-2 gap-8">
      <div>
        <form onSubmit={run}>
          <label className="block text-sm font-semibold mb-2">
            Synthetic vCard input
          </label>
          <textarea
            value={vcard}
            onChange={(e) => setVcard(e.target.value)}
            rows="14"
            spellCheck="false"
            className="w-full font-mono text-xs p-4 border border-line bg-white focus:outline-none focus:border-ink resize-y"
          />

          <label className="block text-sm font-semibold mb-2 mt-4">
            Properties to keep{" "}
            <span className="text-gray-400 font-normal">
              (comma-separated, applied by option63)
            </span>
          </label>
          <input
            type="text"
            value={props}
            onChange={(e) => setProps(e.target.value)}
            className="w-full font-mono text-sm p-3 border border-line bg-white focus:outline-none focus:border-ink"
          />

          <button
            type="submit"
            disabled={parsing}
            className="mt-4 px-6 py-3 bg-ink text-paper font-medium hover:bg-gray-800 transition-colors disabled:opacity-60"
          >
            {parsing ? "Transforming…" : "Transform with option63"}
          </button>
        </form>

        {error && (
          <div className="mt-4 p-4 border border-ink bg-gray-50 text-sm font-mono">
            {error}
          </div>
        )}
      </div>

      <div>
        <label className="block text-sm font-semibold mb-2">
          Result after {accentLabel}
        </label>
        <div className="border border-line bg-white p-4 min-h-[24rem]">
          {lines === null ? (
            <p className="text-gray-400 text-sm">
              The transformed vCard will appear here.
            </p>
          ) : lines.length === 0 ? (
            <p className="text-gray-500 text-sm">No properties matched.</p>
          ) : (
            <table className="w-full border-collapse text-sm">
              <thead>
                <tr className="border-b border-gray-300 text-left">
                  <th className="py-2 pr-4 font-mono font-semibold whitespace-nowrap">
                    Property
                  </th>
                  <th className="py-2 font-semibold">Value</th>
                </tr>
              </thead>
              <tbody>
                {lines.map((line, i) => (
                  <tr key={i} className="border-b border-gray-100 align-top">
                    <td className="py-2 pr-4 font-mono text-xs">
                      {line.name}
                    </td>
                    <td className="py-2 font-mono text-xs break-all">
                      {line.value}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
