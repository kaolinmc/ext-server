#!/bin/sh
# Exit immediately if a command exits with a non-zero status.
set -e

# Define the directory where the model will be stored
MODEL_ID="bert-base-uncased"
MODEL_DIR="models/${MODEL_ID}"

echo "Creating directory ${MODEL_DIR}"
mkdir -p "${MODEL_DIR}"

echo "Downloading tokenizer for ${MODEL_ID}..."

# Download the core tokenizer file and its config
curl -L "https://huggingface.co/${MODEL_ID}/resolve/main/tokenizer.json" -o "${MODEL_DIR}/tokenizer.json"
curl -L "https://huggingface.co/${MODEL_ID}/resolve/main/config.json" -o "${MODEL_DIR}/config.json"

echo "Download complete."