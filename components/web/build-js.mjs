import { build, context } from "esbuild";

const watch = process.argv.includes("--watch");

const entries = [
  { in: "src-js/components/Nav.jsx", out: "components/nav" },
  { in: "src-js/components/ShadowItDemo.jsx", out: "components/shadow-it-demo" },
  { in: "src-js/components/PersonalDemo.jsx", out: "components/personal-demo" },
  { in: "src-js/components/AiDemo.jsx", out: "components/ai-demo" },
  { in: "src-js/components/WhyOption63.jsx", out: "components/why-option63" },
];

const options = {
  entryPoints: entries,
  outdir: "static/js",
  bundle: true,
  minify: !watch,
  format: "esm",
  jsx: "automatic",
  sourcemap: watch ? "inline" : false,
  target: "es2022",
};

if (watch) {
  const ctx = await context(options);
  await ctx.watch();
  console.log("esbuild watching for changes...");
} else {
  await build(options);
  console.log("esbuild built", entries.length, "React bundles");
}
