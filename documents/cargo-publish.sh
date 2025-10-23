#!/bin/sh
# Cargo publish

# Define packages in publish order
packages="fusion-common fusionsql-macros fusionsql-core fusionsql fusion-core-macros fusion-core fusion-security fusion-grpc fusion-db fusion-ai fusion-web fusions"

# Loop through packages and publish with sleep
for package in $packages; do
    echo "Publishing $package..."
    cargo publish --registry crates-io -p "$package"
    if [ $? -eq 0 ]; then
        echo "✅ $package published successfully"
    else
        echo "❌ Failed to publish $package"
        exit 1
    fi
    sleep 5
done

echo "🎉 All packages published successfully!"
