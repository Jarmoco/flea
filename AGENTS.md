# Flea

A GUI file manager for Linux, keyboard first, native to Omarchy.
P0 is the local browser, and remotes, search, disk operations and Flea's own terminal
interface (`flea --tui`, `src/tui`) have since landed on top of it. An encrypted drive
unlocks and locks from its rail row through `src/unlock.rs`.

## Index of modules

The full text of each section below lives in `docs/agents/<slug>.md`.

- [The seven load-bearing rules](docs/agents/the-seven-load-bearing-rules.md)
- [Two-phase listing](docs/agents/two-phase-listing.md)
- [The open directory is watched](docs/agents/the-open-directory-is-watched.md)
- [Mount workers](docs/agents/mount-workers.md)
- [The listing swap](docs/agents/the-listing-swap.md)
- [The preview swap](docs/agents/the-preview-swap.md)
- [Why the listing is an arena](docs/agents/why-the-listing-is-an-arena.md)
- [The listing arena returns to the OS](docs/agents/the-listing-arena-returns-to-the-os.md)
- [Prewarm](docs/agents/prewarm.md)
- [The first window](docs/agents/the-first-window.md)
- [Predictable path writes](docs/agents/predictable-path-writes.md)
- [The state file](docs/agents/the-state-file.md)
- [The theme](docs/agents/the-theme.md)
- [Modes](docs/agents/modes.md)
- [The file chooser portal](docs/agents/the-file-chooser-portal.md)
- [Show in folder](docs/agents/show-in-folder.md)
- [Updates](docs/agents/updates.md)
- [Make Flea the default, in Settings > About](docs/agents/make-flea-the-default-in-settings-about.md)
- [Module map](docs/agents/module-map.md)
- [Where the backend binary comes from](docs/agents/where-the-backend-binary-comes-from.md)
- [Every component needs a qmldir line](docs/agents/every-component-needs-a-qmldir-line.md)
- [Where the UI patterns come from](docs/agents/where-the-ui-patterns-come-from.md)
- [How the list renders](docs/agents/how-the-list-renders.md)
- [The list model](docs/agents/the-list-model.md)
- [File budget](docs/agents/file-budget.md)
- [The key table is generated](docs/agents/the-key-table-is-generated.md)
- [Testing](docs/agents/testing.md)
- [Fixtures](docs/agents/fixtures.md)
- [MIME aliases](docs/agents/mime-aliases.md)
- [Thumbnail cache](docs/agents/thumbnail-cache.md)
- [Thumbnailer specs](docs/agents/thumbnailer-specs.md)
- [Thumbnail sandbox](docs/agents/thumbnail-sandbox.md)
- [Thumbnail worker](docs/agents/thumbnail-worker.md)
- [Markdown figures](docs/agents/markdown-figures.md)
- [Thumbnail pool](docs/agents/thumbnail-pool.md)
- [Thumbnail trace](docs/agents/thumbnail-trace.md)
- [Thumbnail requests](docs/agents/thumbnail-requests.md)
- [A drag does not survive a workspace switch](docs/agents/a-drag-does-not-survive-a-workspace-switch.md)
- [Drag to Trash](docs/agents/drag-to-trash.md)
- [Backend memory levers that were measured and dropped](docs/agents/backend-memory-levers-that-were-measured-and-dropped.md)
- [Write operations and the undo journal](docs/agents/write-operations-and-the-undo-journal.md)
- [Deliberate corners](docs/agents/deliberate-corners.md)
- [The image flasher](docs/agents/the-image-flasher.md)

## Quick reference

- Module map: `docs/agents/module-map.md`
- Protocol: `docs/protocol.md`
- File budget: `docs/agents/file-budget.md`
- Testing: `docs/agents/testing.md`
