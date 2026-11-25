import { formatForFile, read, write } from './core'
import { app, setDiagram, toast } from './state.svelte'

export const exports = [
  { format: 'cetz', ext: 'typ', label: 'CeTZ (Typst)' },
  { format: 'tikz', ext: 'tex', label: 'TikZ (LaTeX)' },
  { format: 'drawio', ext: 'drawio', label: 'draw.io' },
]

export function download(format: string, ext: string) {
  try {
    const text = write(app.diagram, format)
    const a = document.createElement('a')
    a.href = URL.createObjectURL(new Blob([text], { type: 'text/plain' }))
    a.download = `${app.name || 'untitled'}.${ext}`
    a.click()
    setTimeout(() => URL.revokeObjectURL(a.href), 1000)
  } catch (e) {
    toast(String(e))
  }
}

export async function openFile(file: File) {
  const format = formatForFile(file.name)
  if (!format) return toast(`Unsupported file: ${file.name}`)
  try {
    const { diagram, warnings } = read(await file.text(), format)
    setDiagram(diagram, file.name.replace(/\.[^.]+$/, ''))
    if (warnings.length) toast(`Imported with ${warnings.length} warning(s): ${warnings[0]}`)
  } catch (e) {
    toast(String(e))
  }
}

export function pickFile() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.typ,.tex,.tikz,.drawio,.xml'
  input.onchange = () => input.files?.[0] && openFile(input.files[0])
  input.click()
}
