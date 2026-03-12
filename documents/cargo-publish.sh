#!/bin/sh
# Cargo publish

# Define packages in publish order
packages="hetu-common hetusql-macros hetusql-core hetusql hetu-core-macros hetu-core hetu-security hetu-grpc hetu-db hetu-ai hetu-web hetus"

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
