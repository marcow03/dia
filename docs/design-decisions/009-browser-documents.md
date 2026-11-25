# 009: Documents and recents in IndexedDB

## Decision
Each diagram is a document `{ id, name, diagram, updated }`, stored in IndexedDB through `idb-keyval`. The current document autosaves 300 ms after a change. The menu lists recents sorted by `updated`. Empty untitled documents are not saved.

## Why
localStorage holds only ~5 MB of strings, and embedded images quickly exceed that. IndexedDB stores structured objects with no practical size limit, and idb-keyval is a ~1 kB wrapper.
