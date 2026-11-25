# 007: UI principles

- **Few, calm surfaces.** A floating toolbar at the bottom, a document bar at the top left (file actions, export and recents sit behind its menu button), and a style panel that only appears when something is selected. White panels, soft shadows, one accent colour (`#0a84ff`) and the system font.
- **Direct manipulation.** Drag to create, move or resize. Arrows snap to shapes and stay bound to them. Dragging the midpoint dot of a segment adds a waypoint. Everything snaps to a 0.25 cm grid.
- **Typst everywhere.** Double-click a shape or line to edit its label in place, right where it renders. Double-click the empty canvas to write free text.
- **Keyboard first.** V R O D L A T for tools and I for an image (shown under each toolbar button). ⌘S exports CeTZ, ⌘/ toggles the code panel. ⌘Z / ⇧⌘Z, ⌘C / ⌘V / ⌘D, ⌘S to save as CeTZ, arrow keys to nudge, Enter to edit a label, Space or the middle mouse button to pan, ⌘/pinch + scroll to zoom.
- **Controls appear with the tool.** Picking a tool shows its style panel right away. Those values become the defaults for new elements of that tool and are remembered across sessions. All controls are custom (no native selects or spinners), so they look the same everywhere.
- **No modes to remember.** After creating a shape or arrow, the tool returns to select. Work autosaves to localStorage.
