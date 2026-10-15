# WASI Bash guest

This guest embeds rust-bash at
`68d6dbaf7ae059e29cfe1570a421b0e710ec8572`. It is not yet the experiment's
working shell. The current Hibana WASI engine rejects its module at startup.
No comparison episodes have been scored with this guest.

The guest selects `ReadWriteFs`, a stateless adapter to guest `std::fs`.
Those operations are WASI imports. It does not instantiate the interpreter's
in-memory filesystem or use host shell commands, a native filesystem mount,
JS imports, or another Wasm engine. Compilation as a native host program is
rejected. The actual runtime must still supply every supported filesystem
operation through ChoreoFS and the projected protocol.

The recorded patch uses WASI clocks, stable WASI symlink bindings, and the
caller's existing filesystem. It does not provision fake `/bin` files or `/dev`
nodes in that filesystem. POSIX chmod is reported as unsupported instead of
maintaining an extra permission table. Displayed mode bits describe file kinds;
they do not authorize effects. Choreography remains the authority.

Prepare and compile:

```sh
python3 tools/prepare_bash_guest.py
cargo +1.95.0 build --locked --manifest-path guests/bash/Cargo.toml --target wasm32-wasip1 --release
cargo +1.95.0 clippy --locked --manifest-path guests/bash/Cargo.toml --target wasm32-wasip1 --release -- -D warnings
python3 tools/profile_wasi_guest.py guests/bash/target/wasm32-wasip1/release/choreofix-bash-guest.wasm
```

`wasip1.patch` records the exact dependency changes. The preparation script
rejects another revision or unrecorded modifications. Source and third-party
license remain in the prepared `vendor/rust-bash` checkout.

Each stdin request is a four-byte little-endian byte length followed by a UTF-8
command. This preserves multiline scripts. The terminal adapter still needs
to implement this framing; the older bounded shell uses newline input.
There is no fallback from this guest to that older shell.

The measured module has 101 function types, 7,108 defined functions, 1,937 table
entries, a maximum of 540 declared locals, and an 83,955-byte function body.
Its initial linear memory is 43 Wasm pages. All 25 imports are WASI Preview 1.
These are module declarations, not a claim of successful execution or complete
WASI support. Runtime workspace storage, body offsets, filesystem ABI support,
ordinary error recovery, and full end-to-end tests remain to be implemented.
