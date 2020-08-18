# select build image
FROM rust:1.43 as build

# create a new empty shell project
RUN USER=root cargo new --bin iftem_backend
WORKDIR /iftem_backend

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/chromitube_backend*
RUN cargo build --release

# our final base
FROM rust:1.43

# copy the build artifact from the build stage
COPY --from=build /iftem_backend/target/release/chromitube_backend .

# set the startup command to run your binary
CMD ["./chromitube_backend"]