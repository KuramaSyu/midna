# Stage 1: Build the application
FROM ubuntu:noble
FROM rust:latest

# Install dependencies
RUN apt-get update && apt-get install -y \
    libopencv-dev \
    pkg-config \
    cmake \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/app

# Install Rust dependencies
RUN rustup update && \
    rustup component add rustfmt && \
    rustup component add clippy

# Download and install ONNX Runtime binary release
# Download and extract ONNX Runtime
RUN wget https://github.com/microsoft/onnxruntime/releases/download/v1.8.1/onnxruntime-linux-x64-1.8.1.tgz \
    && tar -xzf onnxruntime-linux-x64-1.8.1.tgz \
    && mv onnxruntime-linux-x64-1.8.1 /opt/onnxruntime \
    && ldconfig /opt/onnxruntime/lib

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy .env file
COPY .env .
# Copy the source code
COPY src ./src

# Build the Rust application
RUN cargo install --path .

CMD ["image2nord"]
