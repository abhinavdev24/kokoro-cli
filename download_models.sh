#!/usr/bin/env bash
set -euo pipefail

MODELS_DIR="./models"
mkdir -p "$MODELS_DIR"

echo "Downloading af_heart.pt..."
curl -L -o "$MODELS_DIR/af_heart.pt" \
  "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/voices/af_heart.pt"

echo "Downloading kokoro-quant-convinteger.onnx..."
curl -L -o "$MODELS_DIR/kokoro-quant-convinteger.onnx" \
  "https://github.com/taylorchu/kokoro-onnx/releases/download/v0.2.0/kokoro-quant-convinteger.onnx"

echo "Done. Models saved to $MODELS_DIR/"
