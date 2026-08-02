export type Framework = "svelte" | "vue" | "react";

export type Directive =
  | { type: "load" }
  | { type: "idle" }
  | { type: "visible" }
  | { type: "media"; query: string }
  | { type: "only"; framework: Framework };

export interface ParsedIsland {
  id: string;
  tagName: string;
  importPath: string;
  framework: Framework;
  props: Record<string, unknown>;
  directive: Directive;
}

export interface ParsedTemplate {
  html: string;
  islands: ParsedIsland[];
}

const FRONTMATTER_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?/;
const IMPORT_RE = /import\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?/g;

// Matches <Tag ...attrs /> or <Tag ...attrs>children</Tag>. Tag must start
// uppercase, matching the convention used for island components.
const TAG_RE = /<([A-Z]\w*)((?:\s+[\s\S]*?)?)(\/>|>([\s\S]*?)<\/\1>)/g;

// Matches both name="value"/name={value} pairs and bare boolean attributes
// (e.g. client:load, client:idle with no value at all, mirroring Astro's
// directive syntax where most directives don't take a value).
const ATTR_RE = /([\w:-]+)(?:=(\{[^{}]*\}|"[^"]*"|'[^']*'))?/g;

function frameworkFromPath(path: string): Framework {
  if (path.endsWith(".svelte")) return "svelte";
  if (path.endsWith(".vue")) return "vue";
  if (path.endsWith(".jsx") || path.endsWith(".tsx")) return "react";
  throw new Error(`[meld] cannot infer framework from import path: "${path}"`);
}

function parseAttrValue(raw: string): unknown {
  if (raw.startsWith("{") && raw.endsWith("}")) {
    const inner = raw.slice(1, -1).trim();
    try {
      return JSON.parse(inner);
    } catch {
      // Handle the common case of a bare quoted string inside braces,
      // e.g. title={"Just a regular title"}
      const asString = inner.match(/^["'](.*)["']$/s);
      if (asString) return asString[1];
      // Anything more complex (member expressions, ternaries, etc.) is left
      // as a raw JS source string; the codegen layer decides what to do with it.
      return { __expr: inner };
    }
  }
  return raw.slice(1, -1);
}

function parseAttrs(raw: string): {
  directive: Directive;
  props: Record<string, unknown>;
} {
  const props: Record<string, unknown> = {};
  let directive: Directive = { type: "load" }; // default: hydrate once the DOM is ready

  let m: RegExpExecArray | null;
  ATTR_RE.lastIndex = 0;
  while ((m = ATTR_RE.exec(raw))) {
    const [, name, rawValue] = m;
    if (!name) continue; // skip zero-width matches between whitespace

    switch (name) {
      case "client:load":
        directive = { type: "load" };
        break;
      case "client:idle":
        directive = { type: "idle" };
        break;
      case "client:visible":
        directive = { type: "visible" };
        break;
      case "client:media":
        if (!rawValue)
          throw new Error(
            '[meld] client:media requires a value, e.g. client:media="(max-width: 600px)"',
          );
        directive = { type: "media", query: String(parseAttrValue(rawValue)) };
        break;
      case "client:only":
        if (!rawValue)
          throw new Error(
            '[meld] client:only requires a framework value, e.g. client:only="svelte"',
          );
        directive = {
          type: "only",
          framework: parseAttrValue(rawValue) as Framework,
        };
        break;
      default:
        if (rawValue !== undefined) props[name] = parseAttrValue(rawValue);
    }
  }

  return { directive, props };
}

/**
 * Parses a .meld source file into a stripped template (islands replaced with
 * <meld-island> placeholders + a JSON props script) and the list of island
 * descriptors needed to generate their client entries.
 */
export function parseMeldTemplate(source: string): ParsedTemplate {
  const fmMatch = source.match(FRONTMATTER_RE);
  const frontmatter = fmMatch ? fmMatch[1] : "";
  const body = fmMatch ? source.slice(fmMatch[0].length) : source;

  const imports = new Map<string, string>();
  let im: RegExpExecArray | null;
  IMPORT_RE.lastIndex = 0;
  while ((im = IMPORT_RE.exec(frontmatter))) {
    imports.set(im[1], im[2]);
  }

  const islands: ParsedIsland[] = [];
  const counters = new Map<string, number>();

  const html = body.replace(
    TAG_RE,
    (full, tagName: string, attrsRaw: string) => {
      const importPath = imports.get(tagName);
      if (!importPath) return full; // not an imported island component — leave untouched

      const framework = frameworkFromPath(importPath);
      const { directive, props } = parseAttrs(attrsRaw ?? "");

      const n = counters.get(tagName) ?? 0;
      counters.set(tagName, n + 1);
      const id = `${tagName.toLowerCase()}-${framework}-${n}`;

      islands.push({ id, tagName, importPath, framework, props, directive });

      return [
        `<meld-island id="${id}" data-framework="${framework}"></meld-island>`,
        `<script type="application/json" data-for="${id}">`,
        JSON.stringify(props, null, 2),
        `</script>`,
      ].join("\n");
    },
  );

  return { html, islands };
}
