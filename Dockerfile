# Stage 1: Build the application
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy .env file
COPY .env ./.env
# Copy the source code
COPY src ./src

# Build the Rust application
RUN cargo install --path .

CMD ["image2nord"]
