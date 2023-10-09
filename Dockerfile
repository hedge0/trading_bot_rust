# Use the official Rust image as a parent image
FROM rust:latest as builder

# Set the current directory in the container
WORKDIR /usr/src/trading_bot_rust

# Copy the Cargo.toml and Cargo.lock files to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Create a dummy source to build dependencies and cache them
RUN mkdir -p ./src && echo "fn main() {}" > ./src/main.rs
RUN cargo build --release

# Copy the rest of the code
COPY . .

# Set the RUSTFLAGS environment variable and build the app
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

# Set up a new, lightweight stage without the build tools
FROM debian:buster-slim

# Copy the binary from the builder stage
COPY --from=builder /usr/src/trading_bot_rust/target/release/trading_bot_rust /usr/local/bin/

# Run the application
CMD ["trading_bot_rust"]
