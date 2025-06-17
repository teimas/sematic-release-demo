# GitHub Actions Setup for Semantic Release

This guide will help you set up your GitHub repository to use semantic-release with automated releases.

## 🔧 Prerequisites

1. **GitHub Repository**: Ensure your project is in a GitHub repository
2. **Main Branch**: Your default branch should be `main` (as configured in `.releaserc.json`)
3. **Node.js**: The repository needs Node.js for semantic-release
4. **Rust Project**: Your Rust project should build successfully with `cargo build --release`

## 🔑 Required GitHub Secrets

You need to set up the following secrets in your GitHub repository:

### 1. GITHUB_TOKEN
- **What**: GitHub Personal Access Token for creating releases
- **How to create**:
  1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
  2. Click "Generate new token (classic)"
  3. Select these scopes:
     - `repo` (Full control of private repositories)
     - `write:packages` (Upload packages)
     - `read:org` (Read org and team membership)
  4. Copy the generated token

### 2. NPM_TOKEN (Optional)
- **What**: NPM token for publishing packages (if needed)
- **How to create**:
  1. Login to npmjs.com
  2. Go to Access Tokens → Generate New Token
  3. Select "Automation" type
  4. Copy the generated token

### Setting up secrets in GitHub:
1. Go to your repository on GitHub
2. Click "Settings" tab
3. In the left sidebar, click "Secrets and variables" → "Actions"
4. Click "New repository secret"
5. Add each secret with the exact name shown above

## 📝 Branch Protection Rules

Set up branch protection for your `main` branch:

1. Go to repository Settings → Branches
2. Click "Add rule" for `main` branch
3. Enable:
   - ✅ Require a pull request before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Include administrators

## 🚀 How It Works

### Workflow Triggers
- **Pull Requests**: Runs tests and builds on all PRs to `main`
- **Push to Main**: Runs tests, builds, and creates releases on push to `main`

### Release Process
1. **Test Phase**: Runs formatting checks, clippy, tests, and builds
2. **Cross-Platform Build**: Creates binaries for:
   - Linux x64
   - Windows x64
   - macOS x64
   - macOS ARM64 (Apple Silicon)
3. **Release Phase**: 
   - Analyzes commits since last release
   - Determines version bump (patch/minor/major)
   - Generates changelog
   - Creates GitHub release with binaries
   - Updates version in `Cargo.toml` and `package.json`

### Commit Message Format
Use conventional commits for automatic version bumping:

```bash
feat: add new feature        # → minor version bump
fix: resolve bug             # → patch version bump
feat!: breaking change       # → major version bump
docs: update documentation   # → no version bump
chore: update dependencies   # → no version bump
```

## 📦 Release Artifacts

Each release will include:
- **GitHub Release**: With release notes and version tags
- **Binary Assets**: 
  - `semantic-release-tui-linux-x64.tar.gz` - Linux binary (compressed)
  - `semantic-release-tui` - Linux binary (raw)
  - Cross-platform binaries uploaded as artifacts

## 🔍 Monitoring Releases

You can monitor releases by:
1. **GitHub Actions**: Check the "Actions" tab for workflow runs
2. **Releases**: Check the "Releases" section for published releases
3. **Issues/PRs**: Semantic-release will comment on relevant issues/PRs

## 🛠️ Customization

### Modifying Release Assets
Edit `.releaserc.json` to add/remove assets:

```json
{
  "assets": [
    {
      "path": "path/to/your/asset",
      "label": "Your Asset Label"
    }
  ]
}
```

### Changing Version Calculation
Edit the `@semantic-release/commit-analyzer` configuration in `.releaserc.json`:

```json
["@semantic-release/commit-analyzer", {
  "preset": "conventionalcommits",
  "releaseRules": [
    {"type": "refactor", "release": "patch"},
    {"type": "style", "release": "patch"}
  ]
}]
```

## 🚨 Troubleshooting

### Common Issues:

1. **"No release published"**
   - Check if commits follow conventional format
   - Ensure commits are on `main` branch
   - Verify no `[skip ci]` in commit messages

2. **Build failures**
   - Check Rust code compiles locally with `cargo build --release`
   - Verify all tests pass with `cargo test`
   - Check formatting with `cargo fmt --check`

3. **Permission errors**
   - Verify `GITHUB_TOKEN` has correct permissions
   - Check repository settings allow Actions

### Debug Mode
Add this to your workflow for debugging:

```yaml
- name: Debug semantic-release
  env:
    DEBUG: semantic-release:*
  run: npx semantic-release --dry-run
```

## 📊 Success Indicators

Your setup is working correctly when:
- ✅ PRs trigger test workflows
- ✅ Pushes to `main` trigger release workflows  
- ✅ Semantic-release creates GitHub releases automatically
- ✅ Binary assets are attached to releases
- ✅ Version numbers in `Cargo.toml` update automatically
- ✅ CHANGELOG.md is updated with each release

## 🎯 Next Steps

1. Install dependencies: `npm install`
2. Set up GitHub secrets (GITHUB_TOKEN)
3. Create a PR with conventional commit messages
4. Merge to `main` and watch the magic happen! ✨ 