FROM rust:1.88.0-bookworm AS build

# Setup env
RUN <<EOF
    apt-get update
    apt-get install -y --no-install-recommends \
        libasound2-dev \
        libudev-dev \
        libwayland-dev \
        libxkbcommon-dev
    rustup target add wasm32-unknown-unknown
EOF
ADD . /project/
RUN <<EOF
    cd /project &&\
    cargo build --locked --package lib-level &&\
    cargo build --locked --package lib-anim &&\
    cargo build --target wasm32-unknown-unknown --profile wasm-release --locked
EOF
# Put all files
COPY /assets/ /dist/assets
COPY /static/* /dist
RUN cp /project/target/wasm32-unknown-unknown/wasm-release/quad-jam-2024.wasm /dist/game.wasm
RUN mkdir /dist/levels && \
    /project/target/debug/lib-level \
        --assets /project/assets \ 
        compile-dir \
            -d /project/tiled-project \
            -o /dist/levels
RUN mkdir /dist/animations && \
    /project/target/debug/lib-anim \
        compile-dir \
            -d /project/art-project \
            -o /dist/animations

FROM httpd:trixie 
COPY --from=build /dist /usr/local/apache2/htdocs/ 
