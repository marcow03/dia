<script lang="ts">
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete'
  import { defaultKeymap, history, historyKeymap, insertTab } from '@codemirror/commands'
  import { bracketMatching, indentOnInput } from '@codemirror/language'
  import { EditorState, type Extension } from '@codemirror/state'
  import { drawSelection, EditorView, keymap, placeholder as placeholderExt } from '@codemirror/view'
  import { vim } from '@replit/codemirror-vim'
  import { untrack } from 'svelte'
  import { app } from './state.svelte'
  import { typstSupport } from './typst-lang'

  let {
    value,
    onchange,
    onsubmit,
    oncancel,
    onblur,
    autofocus = false,
    placeholder = '',
  }: {
    value: string
    onchange?: (v: string) => void
    onsubmit?: () => void
    oncancel?: () => void
    onblur?: () => void
    autofocus?: boolean
    placeholder?: string
  } = $props()

  let host: HTMLDivElement
  let view: EditorView
  let destroyed = false

  $effect(() => {
    const extensions: Extension[] = [
      ...(app.settings.vim ? [vim()] : []),
      keymap.of([
        { key: 'Mod-Enter', run: () => (onsubmit?.(), true) },
        { key: 'Tab', run: insertTab },
        ...(app.settings.vim || !oncancel ? [] : [{ key: 'Escape', run: () => (oncancel(), true) }]),
        ...closeBracketsKeymap,
        ...historyKeymap,
        ...defaultKeymap,
      ]),
      history(),
      drawSelection(),
      closeBrackets(),
      bracketMatching(),
      indentOnInput(),
      typstSupport,
      placeholderExt(placeholder),
      EditorView.updateListener.of((u) => u.docChanged && onchange?.(u.state.doc.toString())),
      // Destroying the view blurs it, and vim's `:` prompt takes focus inside the editor;
      // neither counts as leaving it.
      EditorView.domEventHandlers({
        blur: () => void queueMicrotask(() => !destroyed && !host.contains(document.activeElement) && onblur?.()),
      }),
    ]
    view = new EditorView({
      parent: host,
      state: EditorState.create({ doc: untrack(() => value), extensions }),
    })
    if (autofocus) {
      view.focus()
      // Vim starts in normal mode at the end; a programmatic selection would confuse its mode.
      const end = view.state.doc.length
      view.dispatch({ selection: app.settings.vim ? { anchor: end } : { anchor: 0, head: end } })
    }
    return () => {
      destroyed = true
      view.destroy()
    }
  })

  // External changes (e.g. the diagram changed while the code panel is open).
  $effect(() => {
    const v = value
    if (view && !view.hasFocus && v !== view.state.doc.toString())
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: v } })
  })
</script>

<!-- Keys typed here must not reach the canvas shortcuts (a free Escape still closes panels). -->
<div
  class="code-editor"
  bind:this={host}
  role="presentation"
  onkeydown={(e) => (e.key !== 'Escape' || oncancel || app.settings.vim) && e.stopPropagation()}
  onkeyup={(e) => e.stopPropagation()}
></div>

<style>
  .code-editor {
    display: contents;
  }
  .code-editor :global(.cm-editor) {
    height: 100%;
    font: 13px/1.5 var(--mono);
    color: var(--text);
  }
  .code-editor :global(.cm-editor.cm-focused) {
    outline: none;
  }
  .code-editor :global(.cm-content) {
    padding: 8px 0;
    caret-color: var(--accent);
  }
  .code-editor :global(.cm-line) {
    padding: 0 12px;
  }
  .code-editor :global(.cm-scroller) {
    font-family: var(--mono);
  }
  .code-editor :global(.cm-selectionBackground),
  .code-editor :global(.cm-focused .cm-selectionBackground) {
    background: rgb(10 132 255 / 0.18) !important;
  }
  .code-editor :global(.cm-matchingBracket) {
    background: rgb(10 132 255 / 0.15);
    outline: none;
  }
  .code-editor :global(.cm-placeholder) {
    color: var(--muted);
  }
  .code-editor :global(.cm-vim-panel) {
    padding: 2px 12px;
    font: 11px var(--mono);
    background: var(--subtle);
    color: var(--muted);
  }
  .code-editor :global(.cm-fat-cursor) {
    background: rgb(10 132 255 / 0.5) !important;
  }
</style>
