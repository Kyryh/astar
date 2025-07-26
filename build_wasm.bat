wasm-pack build --target web
py -m http.server --directory pkg 8000
