declare module "markdown-it-task-lists" {
  import type MarkdownIt from "markdown-it";
  const plugin: (md: MarkdownIt, opts?: { enabled?: boolean; labelAfter?: boolean }) => void;
  export default plugin;
}
