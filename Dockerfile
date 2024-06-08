# Use the official Rust image as a base
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the Rust application
RUN cargo build --release

# Set the entrypoint to run the built application
CMD ["./target/release/image2nord"]
