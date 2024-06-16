#!/bin/bash

# Define the URLs for the models
ISNET_ANIME_URL="https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-anime.onnx"
ISNET_GENERAL_USE_URL="https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx"
UNET_URL="https://github.com/danielgatis/rembg/releases/download/v0.0.0/unet.onnx"

# Define the local paths where the models will be saved
LOCAL_PATH="./models/"

# Create the directory if it doesn't exist
mkdir -p "$LOCAL_PATH"

# Download the models with progress bar
wget -O "$LOCAL_PATH/isnet-anime.onnx" "$ISNET_ANIME_URL" -q --show-progress
wget -O "$LOCAL_PATH/isnet-general-use.onnx" "$ISNET_GENERAL_USE_URL" -q --show-progress
wget -O "$LOCAL_PATH/unet.onnx" "$UNET_URL" -q --show-progress

echo "Download completed."
