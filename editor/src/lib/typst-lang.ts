import { HighlightStyle, StreamLanguage, syntaxHighlighting } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'

type State = { math: boolean; comment: boolean }

const KEYWORDS = /^#(let|set|show|import|include|if|else|for|while|return|context)\b/

/** Small Typst tokenizer: markup, math, `#` code, strings, raw, comments. */
const typst = StreamLanguage.define<State>({
  name: 'typst',
  startState: () => ({ math: false, comment: false }),
  token(stream, state) {
    if (state.comment) {
      if (stream.skipTo('*/')) {
        stream.match('*/')
        state.comment = false
      } else stream.skipToEnd()
      return 'comment'
    }
    if (stream.match('//')) {
      stream.skipToEnd()
      return 'comment'
    }
    if (stream.match('/*')) {
      state.comment = true
      return 'comment'
    }
    if (stream.match('$')) {
      state.math = !state.math
      return 'keyword'
    }
    if (stream.match(/^\\./) || stream.match('\\')) return 'operator'
    if (stream.match(/^"(?:[^"\\]|\\.)*"?/)) return 'string'
    if (state.math) {
      if (stream.match(/^[a-zA-Z][a-zA-Z.]+/)) return 'variableName'
      if (stream.match(/^\d+(\.\d+)?/)) return 'number'
      if (stream.match(/^[\^_/&=<>+\-*]/)) return 'operator'
      stream.next()
      return 'atom'
    }
    if (stream.match(KEYWORDS)) return 'keyword'
    if (stream.match(/^#[a-zA-Z_][\w-]*(\.[a-zA-Z_][\w-]*)*/)) return 'def'
    if (stream.sol() && stream.match(/^=+\s/)) {
      stream.skipToEnd()
      return 'heading'
    }
    if (stream.match(/^`[^`]*`?/)) return 'string'
    if (stream.match(/^\*[^*\n]+\*/)) return 'strong'
    if (stream.match(/^_[^_\n]+_/)) return 'emphasis'
    if (stream.match(/^(@[\w-]+|<[\w-]+>)/)) return 'link'
    stream.next()
    return null
  },
  languageData: { closeBrackets: { brackets: ['(', '[', '{', '"', '$'] }, commentTokens: { line: '//' } },
})

const style = HighlightStyle.define([
  { tag: t.comment, color: '#8e8e93', fontStyle: 'italic' },
  { tag: t.keyword, color: '#af52de' },
  { tag: t.definition(t.variableName), color: '#0a84ff' },
  { tag: t.string, color: '#c7522a' },
  { tag: t.operator, color: '#ff2d55' },
  { tag: t.variableName, color: '#0071a4' },
  { tag: t.number, color: '#1c00cf' },
  { tag: t.atom, color: '#0071a4' },
  { tag: t.heading, fontWeight: '700' },
  { tag: t.strong, fontWeight: '700' },
  { tag: t.emphasis, fontStyle: 'italic' },
  { tag: t.link, color: '#34a853' },
])

export const typstSupport = [typst, syntaxHighlighting(style)]
