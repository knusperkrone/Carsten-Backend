# select build image
FROM rust:1.65 as build

# create a new empty shell project
RUN USER=root cargo new --bin carsten_backend
WORKDIR /carsten_backend

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN cargo build --release

# our final base
FROM rust:1.65

# copy the build artifact from the build stage
COPY --from=build /carsten_backend/target/release/carsten_backend .

# set the startup command to run your binary
CMD ["./carsten_backend"]
