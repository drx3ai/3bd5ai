# bd3-protocol

This crate defines the "types" for the protocol used by Bd3 CLI, which includes both "internal types" for communication between `bd3-core` and `bd3-tui`, as well as "external types" used with `bd3 app-server`.

This crate should have minimal dependencies.

Ideally, we should avoid "material business logic" in this crate, as we can always introduce `Ext`-style traits to add functionality to types in other crates.
