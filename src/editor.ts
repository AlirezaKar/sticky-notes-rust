import { EditorState, StateEffect } from "@codemirror/state";
import {
  EditorView,
  keymap,
  Decoration,
  ViewPlugin,
  ViewUpdate,
  type DecorationSet,
} from "@codemirror/view";
import { markdown } from "@codemirror/lang-markdown";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { syntaxHighlighting, HighlightStyle } from "@codemirror/language";
import { tags } from "@lezer/highlight";

/**
 * CodeMirror 6 editor with Obsidian-style Live Preview:
 * markdown markers (#, **, etc.) are hidden via decorations and the
 * text is styled as headings/bold/italic/etc.
 */

const headingStyle = HighlightStyle.define([
  { tag: tags.heading1, class: "cm-h1" },
  { tag: tags.heading2, class: "cm-h2" },
  { tag: tags.heading3, class: "cm-h3" },
  { tag: tags.heading4, class: "cm-h4" },
  { tag: tags.heading5, class: "cm-h5" },
  { tag: tags.heading6, class: "cm-h6" },
  { tag: tags.strong, class: "cm-strong" },
  { tag: tags.emphasis, class: "cm-em" },
  { tag: tags.strikethrough, class: "cm-strike" },
  { tag: tags.link, class: "cm-link" },
  { tag: tags.monospace, class: "cm-code" },
  { tag: tags.quote, class: "cm-quote" },
]);

const hideMark = Decoration.mark({ class: "cm-hidden-mark" });

/** Hide markdown punctuation markers (#, *, `, etc.) on each line. */
const livePreview = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = this.build(view);
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = this.build(update.view);
      }
    }
    build(view: EditorView): DecorationSet {
      const builder: { from: number; to: number; value: Decoration }[] = [];
      const { from, to } = view.visibleRanges.reduce(
        (acc, r) => ({ from: Math.min(acc.from, r.from), to: Math.max(acc.to, r.to) }),
        { from: view.state.doc.length, to: 0 },
      );
      const text = view.state.doc.sliceString(from, to);
      const base = from;
      const lines = text.split("\n");
      let offset = base;
      for (const line of lines) {
        const lineLen = line.length;
        const hide = (re: RegExp) => {
          let m: RegExpExecArray | null;
          const r = new RegExp(re.source, "g");
          while ((m = r.exec(line)) !== null) {
            if (m[0].length > 0) {
              builder.push({
                from: offset + m.index,
                to: offset + m.index + m[0].length,
                value: hideMark,
              });
            }
          }
        };
        hide(/^#{1,6}\s/); // heading markers
        hide(/\*\*|\*\*/g); // bold markers
        hide(/(^|\W)\*([^*]+)\*/g); // italic markers (approx)
        hide(/~~/g); // strikethrough markers
        hide(/`{1,3}/g); // code ticks
        hide(/^\s*>\s?/); // blockquote marker
        hide(/^\s*[-*+]\s/); // list bullets
        hide(/^\s*\d+\.\s/); // ordered list numbers
        offset += lineLen + 1;
      }
      return Decoration.set(
        builder.map((b) => b.value.range(b.from, b.to)),
        true,
      );
    }
  },
  {
    decorations: (v) => v.decorations,
  },
);

export interface EditorHandle {
  view: EditorView;
  getValue: () => string;
  setValue: (v: string) => void;
  focus: () => void;
  destroy: () => void;
}

export function createEditor(
  parent: HTMLElement,
  initial: string,
  onChange: (value: string) => void,
): EditorHandle {
  const state = EditorState.create({
    doc: initial,
    extensions: [
      history(),
      markdown(),
      syntaxHighlighting(headingStyle),
      livePreview,
      keymap.of([...defaultKeymap, ...historyKeymap]),
      EditorView.lineWrapping,
      EditorView.updateListener.of((update) => {
        if (update.docChanged) onChange(update.state.doc.toString());
      }),
      EditorView.theme({
        "&": { height: "100%", fontSize: "14px" },
        ".cm-content": { padding: "12px 14px", fontFamily: "system-ui, sans-serif" },
        ".cm-line": { lineHeight: "1.55" },
        ".cm-hidden-mark": { display: "none" },
        ".cm-h1": { fontSize: "1.7em", fontWeight: "700" },
        ".cm-h2": { fontSize: "1.45em", fontWeight: "700" },
        ".cm-h3": { fontSize: "1.25em", fontWeight: "700" },
        ".cm-h4": { fontSize: "1.1em", fontWeight: "700" },
        ".cm-h5": { fontSize: "1em", fontWeight: "700" },
        ".cm-h6": { fontSize: "0.9em", fontWeight: "700", opacity: "0.8" },
        ".cm-strong": { fontWeight: "700" },
        ".cm-em": { fontStyle: "italic" },
        ".cm-strike": { textDecoration: "line-through" },
        ".cm-link": { color: "#2563eb", textDecoration: "underline" },
        ".cm-code": { fontFamily: "ui-monospace, monospace", fontSize: "0.9em" },
        ".cm-quote": { opacity: "0.85", fontStyle: "italic" },
      }),
    ],
  });
  const view = new EditorView({ state, parent });
  return {
    view,
    getValue: () => view.state.doc.toString(),
    setValue: (v: string) => {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } });
    },
    focus: () => view.focus(),
    destroy: () => view.destroy(),
  };
}

/** Plain textarea fallback (source mode). */
export function createSourceEditor(
  parent: HTMLElement,
  initial: string,
  onChange: (value: string) => void,
): EditorHandle {
  const ta = document.createElement("textarea");
  ta.value = initial;
  ta.className = "source-editor";
  ta.spellcheck = true;
  ta.addEventListener("input", () => onChange(ta.value));
  parent.appendChild(ta);
  return {
    view: null as unknown as EditorView,
    getValue: () => ta.value,
    setValue: (v: string) => {
      ta.value = v;
    },
    focus: () => ta.focus(),
    destroy: () => ta.remove(),
  };
}

export { StateEffect };
