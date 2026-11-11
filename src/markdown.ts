import MarkdownIt from "markdown-it";
import taskLists from "markdown-it-task-lists";
import DOMPurify from "dompurify";
import katex from "katex";
import "katex/dist/katex.min.css";

/**
 * Markdown rendering pipeline (preview mode).
 * Core GFM + task lists + wikilinks + math + mermaid placeholders.
 */

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  breaks: true,
});

md.use(taskLists, { enabled: true, labelAfter: true });

// --- Wikilinks: [[Note Title]] -> <a data-wikilink="Note Title"> ---
md.inline.ruler.before("link", "wikilink", (state, silent) => {
  const start = state.pos;
  if (state.src.charCodeAt(start) !== 0x5b /* [ */) return false;
  if (state.src.charCodeAt(start + 1) !== 0x5b) return false;
  const end = state.src.indexOf("]]", start + 2);
  if (end === -1) return false;
  const title = state.src.slice(start + 2, end).trim();
  if (!title) return false;
  if (!silent) {
    const token = state.push("wikilink_open", "a", 1);
    token.attrs = [
      ["href", "#"],
      ["data-wikilink", title],
      ["class", "wikilink"],
    ];
    const text = state.push("text", "", 0);
    text.content = title;
    state.push("wikilink_close", "a", -1);
  }
  state.pos = end + 2;
  return true;
});

// --- Math: $$block$$ and $inline$ -> KaTeX HTML placeholders ---
// We render to HTML strings and stash them, then re-inject after md.render.
const mathPlaceholders: string[] = [];

function stash(html: string): string {
  const id = `%%MATH${mathPlaceholders.length}%%`;
  mathPlaceholders.push(html);
  return id;
}

md.inline.ruler.before("link", "math_inline", (state, silent) => {
  const start = state.pos;
  if (state.src.charCodeAt(start) !== 0x24 /* $ */) return false;
  // Block math handled at block level; inline is single $.
  if (state.src.charCodeAt(start + 1) === 0x24) return false;
  const end = state.src.indexOf("$", start + 1);
  if (end === -1) return false;
  const tex = state.src.slice(start + 1, end).trim();
  if (!tex || tex.includes("\n")) return false;
  if (!silent) {
    let html: string;
    try {
      html = katex.renderToString(tex, { throwOnError: false });
    } catch {
      html = `<code>$${tex}$</code>`;
    }
    const token = state.push("text", "", 0);
    token.content = stash(html);
  }
  state.pos = end + 1;
  return true;
});

md.block.ruler.before("code", "math_block", (state, startLine, endLine, silent) => {
  const startPos = state.bMarks[startLine] + state.tShift[startLine];
  if (state.src.charCodeAt(startPos) !== 0x24) return false;
  if (state.src.charCodeAt(startPos + 1) !== 0x24) return false;
  const end = state.src.indexOf("$$", startPos + 2);
  if (end === -1) return false;
  const tex = state.src.slice(startPos + 2, end).trim();
  if (!silent) {
    let html: string;
    try {
      html = katex.renderToString(tex, { throwOnError: false, displayMode: true });
    } catch {
      html = `<pre>${tex}</pre>`;
    }
    const token = state.push("html_block", "", 0);
    token.content = stash(html);
    token.map = [startLine, state.line + 1];
  }
  state.line = state.src.slice(0, end).split("\n").length - 1;
  return true;
});

// --- Mermaid: ```mermaid blocks -> <div class="mermaid"> ---
const defaultFence = md.renderer.rules.fence!;
md.renderer.rules.fence = (tokens, idx, options, env, slf) => {
  const token = tokens[idx];
  if (token.info.trim() === "mermaid") {
    const code = token.content.trim();
    return `<div class="mermaid">${DOMPurify.sanitize(code)}</div>`;
  }
  return defaultFence(tokens, idx, options, env, slf);
};

export function renderMarkdown(source: string): string {
  mathPlaceholders.length = 0;
  let html = md.render(source);
  // Re-inject stashed math HTML (placeholders survive as text).
  for (let i = 0; i < mathPlaceholders.length; i++) {
    html = html.replaceAll(`%%MATH${i}%%`, mathPlaceholders[i]);
  }
  return DOMPurify.sanitize(html, {
    ADD_ATTR: ["data-wikilink", "target"],
    ADD_TAGS: ["div"],
  });
}

/** Render mermaid diagrams inside a container (lazy, cached). */
let mermaidReady = false;
export async function renderMermaidDiagrams(container: HTMLElement): Promise<void> {
  const nodes = container.querySelectorAll(".mermaid");
  if (!nodes.length) return;
  if (!mermaidReady) {
    const mermaid = (await import("mermaid")).default;
    mermaid.initialize({ startOnLoad: false, theme: "default" });
    mermaidReady = true;
  }
  const mermaid = (await import("mermaid")).default;
  await mermaid.run({ nodes: Array.from(nodes) as HTMLElement[] });
}

/** Extract wikilink targets from markdown source. */
export function extractWikilinks(source: string): string[] {
  const out: string[] = [];
  const re = /\[\[([^\]]+)\]\]/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(source)) !== null) out.push(m[1].trim());
  return out;
}
