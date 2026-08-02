import type { Plugin } from "vite";
import { access, readFile } from "node:fs/promises";
import { globSync } from "glob";
import { minify } from "html-minifier-terser";
import { ParsedIsland, parseMeldTemplate } from "./parser";
import path from "path";

const MELD_ROOT_FOLDER = path.join(process.cwd(), "src/pages");
const ASSET_EXT = "html";
const FILE_RE = /\.meld$/;

const toProjectRelative = (absId: string): string =>
  path.relative(MELD_ROOT_FOLDER, absId).replace(/\\/g, "/");

const toBaseName = (absId: string): string =>
  toProjectRelative(absId).replace(/\.meld$/, "");

export interface MeldPluginOptions {
  /** File extension(s) this plugin should transform. Defaults to .meld */
  include?: RegExp;
  /** Glob pattern to auto-discover .meld files. Defaults to src/pages/**\/*.meld */
  pagesGlob?: string;
  /** Whether to minify emitted HTML. Defaults to true in production */
  minifyHtml?: boolean;
}

export default function meld(options: MeldPluginOptions = {}): Plugin {
  const include = options.include ?? FILE_RE;
  const pagesGlob = options.pagesGlob ?? "src/pages/**/*.meld";
  const minifyHtml = options.minifyHtml ?? true;
  const virtualHydratePrefix = "virtual:meld-hydrate:";
  const devHydratePrefix = "/@meld/";

  // Store islands data by virtual hydration module id
  const hydrationModules = new Map<string, ParsedIsland[]>();

  const buildHydrationModule = (islands: ParsedIsland[]): string => {
    if (islands.length === 0) return "export {};";

    const frameworks = new Set(islands.map((island) => island.framework));

    const runtimeImports = [
      frameworks.has("react")
        ? 'import { createElement } from "react";\nimport { createRoot } from "react-dom/client";'
        : "",
      frameworks.has("vue") ? 'import { createApp } from "vue";' : "",
      frameworks.has("svelte") ? 'import { mount } from "svelte";' : "",
    ]
      .filter(Boolean)
      .join("\n");

    const imports = islands
      .map(
        (island, i) =>
          `import Comp${i} from ${JSON.stringify(`/${island.importPath.replace(/^\/+/, "")}`)};`,
      )
      .join("\n");

    const componentMap = islands
      .map((island, i) => `${JSON.stringify(island.tagName)}: Comp${i}`)
      .join(",\n  ");

    const islandsJson = JSON.stringify(islands);

    return `${runtimeImports}
${runtimeImports ? "\n" : ""}${imports}

const components = {
  ${componentMap}
};

const islands = ${islandsJson};

const roots = new Map();

function hydrateIsland(island) {
  const el = document.getElementById(island.id);
  if (!el) return;

  const Component = components[island.tagName];
  if (!Component) return;

  const propsScript = document.querySelector(
    \`script[data-for="\${island.id}"]\`
  );
  const props = propsScript?.textContent ? JSON.parse(propsScript.textContent) : {};

  if (island.framework === "react") {
    const root = roots.get(island.id) ?? createRoot(el);
    roots.set(island.id, root);
    root.render(createElement(Component, props));
    return;
  }

  if (island.framework === "vue") {
    createApp(Component, props).mount(el);
    return;
  }

  if (island.framework === "svelte") {
    mount(Component, { target: el, props });
  }
}

function scheduleHydration(island) {
  if (island.directive.type === "idle" && "requestIdleCallback" in window) {
    window.requestIdleCallback(() => hydrateIsland(island));
    window.setTimeout(() => hydrateIsland(island), 200);
    return;
  }

  if (island.directive.type === "idle") {
    window.setTimeout(() => hydrateIsland(island), 1);
    return;
  }

  if (island.directive.type === "load") {
    hydrateIsland(island);
    return;
  }

  hydrateIsland(island);
}

for (const island of islands) {
  scheduleHydration(island);
}
`;
  };

  const resolvePageRequest = async (
    urlPath: string,
  ): Promise<string | null> => {
    const cleanPath = urlPath.replace(/\/+$/, "") || "/";
    const candidate = cleanPath === "/pages" ? "/pages/index" : cleanPath;
    const relativePage = candidate.replace(/^\//, "").replace(/\.html$/, "");
    const sourceFile = path.join(process.cwd(), "src", `${relativePage}.meld`);

    try {
      await access(sourceFile);
      return sourceFile;
    } catch {
      return null;
    }
  };

  return {
    name: "vite-plugin-meld",
    enforce: "pre",
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        if (!req.url) return next();

        const pathname = req.url.split("?")[0] ?? "/";

        if (pathname.startsWith(devHydratePrefix) && pathname.endsWith(".js")) {
          const baseName = pathname
            .slice(devHydratePrefix.length)
            .replace(/\.js$/, "");
          const sourceFile = path.join(
            process.cwd(),
            "src",
            `${baseName}.meld`,
          );

          try {
            const source = await readFile(sourceFile, "utf8");
            const { islands } = parseMeldTemplate(source);
            hydrationModules.set(`${virtualHydratePrefix}${baseName}`, islands);
            const transformed = await server.transformRequest(pathname);

            res.statusCode = 200;
            res.setHeader(
              "Content-Type",
              "application/javascript; charset=utf-8",
            );
            res.end(transformed?.code ?? buildHydrationModule(islands));
            return;
          } catch {
            return next();
          }
        }

        const sourceFile = await resolvePageRequest(pathname);
        if (!sourceFile) return next();

        const source = await readFile(sourceFile, "utf8");
        const { html } = parseMeldTemplate(source);
        const modulePath = `${devHydratePrefix}${toBaseName(sourceFile)}.js`;
        const htmlWithScript = html.replace(
          /(<\/body>)/i,
          `<script type="module" src="${modulePath}"></script>\n$1`,
        );

        const transformed = await server.transformIndexHtml(
          pathname,
          htmlWithScript,
        );
        res.statusCode = 200;
        res.setHeader("Content-Type", "text/html; charset=utf-8");
        res.end(transformed);
      });
    },
    resolveId(id) {
      if (id.startsWith(devHydratePrefix) && id.endsWith(".js")) {
        const baseName = id.slice(devHydratePrefix.length).replace(/\.js$/, "");
        return `${virtualHydratePrefix}${baseName}`;
      }

      if (id.startsWith(virtualHydratePrefix)) return id;
      return null;
    },
    load(id) {
      if (!id.startsWith(virtualHydratePrefix)) return null;

      const fromCache = hydrationModules.get(id);
      if (fromCache) return buildHydrationModule(fromCache);

      const baseName = id.slice(virtualHydratePrefix.length);
      const sourceFile = path.join(process.cwd(), "src", `${baseName}.meld`);

      return readFile(sourceFile, "utf8")
        .then((source) => {
          const { islands } = parseMeldTemplate(source);
          hydrationModules.set(id, islands);
          return buildHydrationModule(islands);
        })
        .catch(() => "export {};");
    },
    config(config) {
      // Auto-discover .meld files and inject them as Rollup inputs
      const meldFiles = globSync(pagesGlob);
      if (meldFiles.length > 0) {
        if (!config.build) config.build = {};
        if (!config.build.rollupOptions) config.build.rollupOptions = {};
        config.build.rollupOptions.input = meldFiles;
      }
    },
    transform(source, id) {
      if (!include.test(id)) return undefined;

      const { html, islands } = parseMeldTemplate(source);

      const baseName = toBaseName(id);

      this.emitFile({
        source: html,
        fileName: `${baseName}.${ASSET_EXT}`,
        type: "asset",
      });

      if (islands.length > 0) {
        const virtualHydrateId = `${virtualHydratePrefix}${baseName}`;
        hydrationModules.set(virtualHydrateId, islands as ParsedIsland[]);
      }

      // Entry modules must still return valid JS so Rollup can continue.
      // Import the virtual hydrate module so the emitted entry chunk is useful
      // instead of an empty placeholder produced for each .meld Rollup input.
      return {
        code: [
          islands.length > 0
            ? `import ${JSON.stringify(`${virtualHydratePrefix}${baseName}`)};`
            : "",
          `export default ${JSON.stringify(`${baseName}.${ASSET_EXT}`)};`,
        ]
          .filter(Boolean)
          .join("\n"),
        map: null,
      };
    },
    async generateBundle(_output, bundle) {
      // Map each emitted .{ASSET_EXT} file to its corresponding compiled entry chunk.
      const htmlToEntryChunk = new Map<string, string>();
      for (const chunk of Object.values(bundle)) {
        if (chunk.type !== "chunk") continue;
        if (!chunk.isEntry || !chunk.facadeModuleId) continue;
        if (!include.test(chunk.facadeModuleId)) continue;

        const baseName = toBaseName(chunk.facadeModuleId);
        htmlToEntryChunk.set(`${baseName}.${ASSET_EXT}`, chunk.fileName);
      }

      // Inject compiled hydration entry chunk paths into emitted HTML assets.
      for (const [fileName, asset] of Object.entries(bundle)) {
        if (fileName.endsWith(`.${ASSET_EXT}`) && asset.type === "asset") {
          const compiledChunkFile = htmlToEntryChunk.get(fileName);

          if (compiledChunkFile) {
            const fromDir = path.posix.dirname(fileName);
            const relScriptPath = path.posix.relative(
              fromDir,
              compiledChunkFile,
            );
            let html = asset.source as string;
            const scriptTag = `<script src="${relScriptPath}" type="module"></script>`;
            // Inject script tag before closing body
            html = html.replace(/(<\/body>)/i, `${scriptTag}\n$1`);
            asset.source = html;
          }
        }
      }

      // Minify emitted HTML assets
      if (!minifyHtml) return;

      for (const [fileName, asset] of Object.entries(bundle)) {
        if (fileName.endsWith(`.${ASSET_EXT}`) && asset.type === "asset") {
          try {
            const minified = await minify(asset.source as string, {
              removeComments: true,
              collapseWhitespace: true,
              removeAttributeQuotes: false,
            });
            asset.source = minified;
          } catch (err) {
            this.warn(`Failed to minify ${fileName}: ${err}`);
          }
        }
      }
    },
  };
}
