# WASM Build with Parallel Execution

This crate can be built for WebAssembly with full parallel execution support using Rayon via Web Workers.

## Building for WASM

The WASM build requires the nightly Rust toolchain and some special flags to enable SharedArrayBuffer support:

```bash
# Install nightly toolchain if not already installed
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly

# Build the WASM package
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  cargo +nightly build --target wasm32-unknown-unknown --release \
  -Z build-std=std,panic_abort

# Generate JavaScript bindings
wasm-bindgen target/wasm32-unknown-unknown/release/formcalc.wasm \
  --out-dir pkg --target web
```

Or use the provided configuration in `.cargo/config.toml` which sets the flags automatically:

```bash
cargo +nightly build --target wasm32-unknown-unknown --release -Z build-std=std,panic_abort
wasm-bindgen target/wasm32-unknown-unknown/release/formcalc.wasm --out-dir pkg --target web
```

## Using in the Browser

To enable parallel execution in the browser, you need to:

1. **Serve with proper headers** - SharedArrayBuffer requires these HTTP headers:
   ```
   Cross-Origin-Opener-Policy: same-origin
   Cross-Origin-Embedder-Policy: require-corp
   ```

2. **Initialize the thread pool** before using the library:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>FormCalc WASM Demo</title>
</head>
<body>
    <script type="module">
        import init, { initThreadPool, Engine, Formula, Value } from './pkg/formcalc.js';

        async function run() {
            // Initialize the WASM module
            await init();
            
            // Initialize the Web Worker thread pool for parallel execution
            await initThreadPool(navigator.hardwareConcurrency || 4);
            
            // Now you can use the Engine with full parallel execution
            const engine = new Engine();
            // ... use the engine
        }

        run();
    </script>
</body>
</html>
```

## Simple HTTP Server with CORS Headers

For local development, you can use this simple Python server that sets the required headers:

```python
#!/usr/bin/env python3
from http.server import HTTPServer, SimpleHTTPRequestHandler

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

if __name__ == '__main__':
    HTTPServer(('localhost', 8000), CORSRequestHandler).serve_forever()
```

Save this as `server.py` and run:
```bash
python3 server.py
```

Then open http://localhost:8000 in your browser.

## Browser Compatibility

- Chrome/Edge 92+
- Firefox 95+
- Safari 15.2+

All browsers must support SharedArrayBuffer and Web Workers for parallel execution to work.
