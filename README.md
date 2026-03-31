# leptos_portfolio

This is a demo site using RUST and leptos to create a website executing RUST code thanks to WASM.\
WASM compilation is fully integrated in RUST framework.\
With the help of trunk, you'll get a "dist" directory that you can copy/paste to deploy your site.

You can also check my site: https://www.tapcul.org to see the final result.

## Dependencies

- You'll need wasm:
`rustup target add wasm32-unknown-unknown`
- You'll need TRUNK:
`cargo install trunk`

## How to build
`trunk build --release`

Or with a local server (with a preview each time the code is saved):
`trunk serve --open --release`

## What you'll get

- A first page with a Mandelbrot Explorer, all coded in Rust and real-time executed thanks to Wasm.
- A second page with a Physics Engine renderer (Rapier2D) to simulate falling balls with collisions. All done in Rust and Wasm.
