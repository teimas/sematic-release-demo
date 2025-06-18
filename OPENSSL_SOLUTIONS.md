# OpenSSL Build Solutions for GitHub Actions

## Problem
When building Rust projects with OpenSSL dependencies on macOS in GitHub Actions, you may encounter:
```
error: failed to run custom build command for `openssl-sys v0.9.109`
warning: openssl-sys@0.9.109: Could not find directory of OpenSSL installation
```

## Solutions Implemented

### 1. 🎯 **Vendored OpenSSL Feature** (Primary Solution)
Added `vendored-openssl` feature flag in `Cargo.toml`:
```toml
[features]
vendored-openssl = ["openssl/vendored"]

[dependencies]
openssl = "0.10"
```

**Benefits:**
- ✅ Self-contained builds (no system dependencies)
- ✅ Consistent across all platforms
- ✅ No external OpenSSL installation required
- ✅ Works in any CI environment

**Usage:**
```bash
cargo build --release --features vendored-openssl
```

### 2. 🔧 **Enhanced GitHub Actions Workflow**
Improved the CI workflow with:

#### Linux Dependencies
```yaml
- name: Install dependencies (Linux)
  if: runner.os == 'Linux'
  run: |
    sudo apt-get update
    sudo apt-get install -y pkg-config libssl-dev
```

#### macOS OpenSSL Setup with Fallback
```yaml
- name: Install OpenSSL (macOS)
  if: runner.os == 'macOS'
  run: |
    # Try to install OpenSSL via Homebrew
    brew install openssl@3 pkg-config || true
    
    # Set OpenSSL environment variables
    if [ -d "$(brew --prefix openssl@3 2>/dev/null)" ]; then
      echo "OPENSSL_DIR=$(brew --prefix openssl@3)" >> $GITHUB_ENV
      echo "OPENSSL_LIB_DIR=$(brew --prefix openssl@3)/lib" >> $GITHUB_ENV
      echo "OPENSSL_INCLUDE_DIR=$(brew --prefix openssl@3)/include" >> $GITHUB_ENV
      echo "PKG_CONFIG_PATH=$(brew --prefix openssl@3)/lib/pkgconfig" >> $GITHUB_ENV
    else
      echo "Warning: Failed to install OpenSSL via Homebrew, will use vendored OpenSSL"
    fi
```

#### Smart Build Strategy
**Unix Systems (Linux/macOS):**
```yaml
- name: Build (Unix)
  if: runner.os != 'Windows'
  run: |
    # Try building with system OpenSSL first
    if ! cargo build --release --target ${{ matrix.target }}; then
      echo "System OpenSSL build failed, trying with vendored OpenSSL..."
      cargo build --release --target ${{ matrix.target }} --features vendored-openssl
    fi
```

**Windows:**
```yaml
- name: Build (Windows)
  if: runner.os == 'Windows'
  run: cargo build --release --target ${{ matrix.target }} --features vendored-openssl
```

### 3. 🏗️ **Build Strategy by Platform**
The workflow uses different strategies per platform:

#### Unix Systems (Linux/macOS)
1. **Try system OpenSSL** (faster, smaller binary)
2. **Fallback to vendored OpenSSL** (always works, larger binary)

#### Windows
1. **Always use vendored OpenSSL** (Windows doesn't have system OpenSSL)

## Environment Variables Set

### macOS
- `OPENSSL_DIR`: Path to OpenSSL installation
- `OPENSSL_LIB_DIR`: Path to OpenSSL libraries
- `OPENSSL_INCLUDE_DIR`: Path to OpenSSL headers
- `PKG_CONFIG_PATH`: Path for pkg-config to find OpenSSL

## Testing Locally

### Test System OpenSSL Build
```bash
cargo build --release
```

### Test Vendored OpenSSL Build
```bash
cargo build --release --features vendored-openssl
```

### Test Cross-Platform Build (macOS)
```bash
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Troubleshooting

### If builds still fail:
1. **Check OpenSSL installation:**
   ```bash
   brew --prefix openssl@3
   echo $OPENSSL_DIR
   ```

2. **Force vendored build:**
   ```bash
   cargo clean
   cargo build --release --features vendored-openssl
   ```

3. **Check environment variables:**
   ```bash
   env | grep OPENSSL
   ```

### Common Issues:
- **Missing pkg-config**: Install with `brew install pkg-config`
- **Old OpenSSL version**: Use `openssl@3` specifically
- **Environment not set**: Restart terminal after setting variables

## Binary Size Comparison
- **System OpenSSL**: ~10-15MB smaller
- **Vendored OpenSSL**: ~10-15MB larger but more portable

## Recommendation
✅ **Use vendored OpenSSL in CI** for reliability
✅ **Use system OpenSSL locally** for faster builds
✅ **The workflow automatically handles both** scenarios 

## Platform-Specific Notes

### Windows
- **Always uses vendored OpenSSL** since Windows doesn't ship with OpenSSL
- **No fallback needed** - vendored is the reliable approach
- **Avoids shell syntax issues** between PowerShell and bash

### Linux/macOS  
- **Tries system OpenSSL first** for optimal performance
- **Automatic fallback** ensures builds always succeed 