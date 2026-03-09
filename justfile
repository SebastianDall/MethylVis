set shell := ["fish", "-c"]

invo_dir := invocation_directory()

deploy:
    rm -rd deploy; or true
    mkdir deploy
    cargo build --release
    mv target/release/mag_server deploy/
    cd contam-map-frontend && npm run build
    cp -r contam-map-frontend/build deploy/

run_server:
    cd {{invo_dir}}/deploy && ./mag_server
