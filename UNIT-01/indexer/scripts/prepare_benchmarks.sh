#!/bin/bash
# scripts/prepare_benchmarks.sh

# Create benchmarks directory if it doesn't exist
mkdir -p "C:\Momo\Indexer\Yomi/benchmarks"
cd "C:\Momo\Indexer\Yomi/benchmarks"

echo "🚀 Starting dataset preparation..."

# 1. Clone Linux Kernel (Depth 1 for speed)
echo "📦 Cloning Linux Kernel..."
git clone --depth 1 https://github.com/torvalds/linux.git linux || echo "Linux clone failed or already exists"

# 2. Clone VS Code (Depth 1 for speed)
echo "📦 Cloning VS Code..."
git clone --depth 1 https://github.com/microsoft/vscode.git vscode || echo "VS Code clone failed or already exists"

# 3. Generate Synthetic Repositories
echo "🧪 Generating synthetic repositories..."
for size in 10000 50000 100000; do
  echo "Generating synthetic_$size (files: $size)..."
  mkdir -p "synthetic_$size"
  cd "synthetic_$size"
  # Use a faster way to generate files than a slow loop
  for i in $(seq 1 $size); do
    echo "fn test_$i() { println!(\"hello\"); }" > "file_$i.rs"
  done
  cd ".."
done

echo "✅ Dataset preparation complete!"
