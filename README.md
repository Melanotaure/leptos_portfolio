# leptos_portfolio

This is a demo site using RUST and leptos to create a website executing RUST code thanks to WASM.\
WASM compilation is fully integrated in RUST framework.\
With the help of trunk, you'll get a "dist" directory that you can copy/paste to deploy your site.

## How to build
`trunk build --release`

Or with a local server:
`trunk serve --open --release`

## Dependencies

- You'll need wasm:
`rustup target add wasm32-unknown-unknown`

- You'll need TRUNK:
`cargo install trunk`
