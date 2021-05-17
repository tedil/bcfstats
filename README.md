1. install wasm-pack: `cargo install wasm-pack` (use `--force` to update)
2. install miniserve: `cargo install miniserver`
3. build: `wasm-pack build --target web`
4. serve: `miniserve ./pkg --index index.html`
5. open `localhost:8080`
