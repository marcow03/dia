import { createStore, del, get, set, values } from 'idb-keyval'
import type { Diagram } from './types'

export type Doc = { id: string; name: string; diagram: Diagram; updated: number }
export type DocMeta = Omit<Doc, 'diagram'>

const store = createStore('dia', 'docs')

export const saveDoc = (doc: Doc) => set(doc.id, doc, store)
export const loadDoc = (id: string) => get<Doc>(id, store)
export const deleteDoc = (id: string) => del(id, store)

export async function listDocs(): Promise<DocMeta[]> {
  const docs = await values<Doc>(store)
  return docs.map(({ id, name, updated }) => ({ id, name, updated })).sort((a, b) => b.updated - a.updated)
}
