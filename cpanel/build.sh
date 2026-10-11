#!/usr/bin/env bash
# ==============================================================================
# 1-Click Cross-Compiler & Packager for cPanel Shared Hosting
# ==============================================================================
set -e

APP_NAME="app"
DIST_DIR="dist-cpanel"
ZIP_FILE="cpanel-deploy.zip"

echo "⚡ Building OxideAdmin static Linux binary for cPanel..."

# Build statically linked musl binary (guaranteed to run on any Linux kernel/cPanel)
if command -v cross &> /dev/null; then
    echo "📦 Using 'cross' for x86_64-unknown-linux-musl..."
    cross build --release --target x86_64-unknown-linux-musl --bin "$APP_NAME"
    BIN_SRC="target/x86_64-unknown-linux-musl/release/$APP_NAME"
elif rustup target list 2>/dev/null | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "📦 Building native x86_64-unknown-linux-musl..."
    cargo build --release --target x86_64-unknown-linux-musl --bin "$APP_NAME"
    BIN_SRC="target/x86_64-unknown-linux-musl/release/$APP_NAME"
else
    echo "📦 Building release binary..."
    cargo build --release --bin "$APP_NAME"
    BIN_SRC="target/release/$APP_NAME"
fi

echo "📦 Assembling cPanel deployment package..."
rm -rf "$DIST_DIR" "$ZIP_FILE"
mkdir -p "$DIST_DIR/app"
mkdir -p "$DIST_DIR/public_html"

# Copy binary and scripts
if [ -f "$BIN_SRC" ]; then
    cp "$BIN_SRC" "$DIST_DIR/app/$APP_NAME"
    chmod +x "$DIST_DIR/app/$APP_NAME"
else
    echo "⚠️ Notice: $BIN_SRC not found. Make sure to compile the Linux binary before uploading."
    touch "$DIST_DIR/app/$APP_NAME"
fi

cp cpanel/app/watchdog.sh "$DIST_DIR/app/watchdog.sh"
chmod +x "$DIST_DIR/app/watchdog.sh"

if [ -f "cpanel/app/.env.example" ]; then
    cp cpanel/app/.env.example "$DIST_DIR/app/.env"
fi

# Copy public web files
cp cpanel/public_html/.htaccess "$DIST_DIR/public_html/.htaccess"
cp cpanel/public_html/index.php "$DIST_DIR/public_html/index.php"

# Create zip archive if zip is available
if command -v zip &> /dev/null; then
    (cd "$DIST_DIR" && zip -r "../$ZIP_FILE" app public_html)
    echo "✅ cPanel deployment bundle ready: $ZIP_FILE"
else
    echo "✅ cPanel distribution directory ready: $DIST_DIR/"
fi

echo ""
echo "🚀 Next steps for cPanel deployment:"
echo "1. Upload '$ZIP_FILE' (or contents of '$DIST_DIR/') to your cPanel File Manager in '/home/username/'"
echo "2. Extract the archive (places 'app/' in /home/username/ and 'public_html/' in /home/username/public_html/)"
echo "3. Ensure 'app/$APP_NAME' has execute permissions (chmod 755)"
echo "4. Add a Cron Job in cPanel: '* * * * * /home/username/app/watchdog.sh > /dev/null 2>&1'"
echo "5. Open your domain in browser!"
