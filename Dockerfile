# select build image
FROM ekidd/rust-musl-builder

# create a new empty shell project
RUN USER=root cargo new --bin chromi_tube_backend
WORKDIR /chromi_tube_backend

# copy over your manifests
COPY ./Cargo.toml ./Cargo.toml

# copy your source tree
COPY ./src ./src

# build for release
ADD --chown=rust:rust . ./
RUN cargo build --release

# our final base
FROM alpine:latest

# copy the build artifact from the build stage
COPY --from=build /chromi_tube_backend/target/release/chromi_tube_backend .
COPY --from=build /chromi_tube_backend/certs ./certs

# set the startup command to run your binary
CMD ["./chromi_tube_backend"]
