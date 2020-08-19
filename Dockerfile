# select build image
FROM rust:1.43 as build

# create a new empty shell project
RUN USER=root cargo new --bin chromi_tube_backend
WORKDIR /chromi_tube_backend

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/chromi_tube_backend*
RUN cargo build --release

# our final base
FROM alpine:latest

# copy the build artifact from the build stage
COPY --from=build /chromi_tube_backend/target/release/chromi_tube_backend .

# set the startup command to run your binary
CMD ["./chromi_tube_backend"]
