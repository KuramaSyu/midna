# Use a pre-built Docker image with cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-1.78.0 AS chef
WORKDIR /usr/src/app

# Prepare the build environment using cargo-chef
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cook the build dependencies using cargo-chef
FROM chef AS builder 
COPY --from=planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# Build the application
COPY . .
RUN cargo build --release --bin image2nord

# Base image for the final application
FROM ubuntu:noble

# Update package lists and install wget
RUN apt-get update \
  && apt-get install -y wget \
  && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/app

# Download and install ONNX Runtime binary release
RUN wget https://github.com/microsoft/onnxruntime/releases/download/v1.8.1/onnxruntime-linux-x64-1.8.1.tgz \
    && tar -xzf onnxruntime-linux-x64-1.8.1.tgz \
    && mv onnxruntime-linux-x64-1.8.1 /opt/onnxruntime \
    && ldconfig /opt/onnxruntime/lib

# Copy the compiled Rust binary to the final image
COPY --from=builder /usr/src/app/target/release/image2nord ./image2nord

# Command to run the application
CMD ["./image2nord"]
