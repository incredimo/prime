param (
    [switch]$local
)

# Define the path to Cargo.toml 
$cargoTomlPath = "./Cargo.toml"

# Ensure Cargo.toml exists
if (-Not (Test-Path $cargoTomlPath)) {
    Write-Output "❌ Cargo.toml not found at path: $cargoTomlPath"
    Write-Output "Please ensure the script is run from the root directory of your Rust project."
    exit 1
}

# Read the Cargo.toml content into a variable
$cargoTomlContent = Get-Content -Path $cargoTomlPath -Raw

# Use a regular expression to find the version line
$matched = $cargoTomlContent -match 'version\s*=\s*"(\d+\.\d+\.\d+)"'
if (-Not $matched) {
    Write-Output "❌ Version line not found in Cargo.toml"
    Write-Output "Please ensure the Cargo.toml file contains a valid version line."
    exit 1
}
$versionLine = $matches[1]

# Split the version into major, minor, and patch
$versionParts = $versionLine.Split('.')
$major = $versionParts[0]
$minor = $versionParts[1]
$patch = [int]$versionParts[2]

# Increment the patch version
try {
    $patch += 1
} catch {
    Write-Output "❌ Failed to increment patch version"
    exit 1
}

# Construct the new version string
$newVersion = "$major.$minor.$patch"

# Replace the old version with the new version in the Cargo.toml content
$newCargoTomlContent = $cargoTomlContent -replace ('version\s*=\s*"' + [regex]::Escape($versionLine) + '"'), ('version = "' + $newVersion + '"')

# Write the new Cargo.toml content back to the file
Set-Content -Path $cargoTomlPath -Value $newCargoTomlContent
Write-Output "✅ Updated version to $newVersion in Cargo.toml"

# Check if tag already exists and increment version if needed
$tagExists = git tag --list "v$newVersion"
if ($tagExists) {
    Write-Output "⚠️ Tag v$newVersion already exists. Incrementing patch version again..."
    # Increment patch version again
    $patch += 1
    $newVersion = "$major.$minor.$patch"
    # Update Cargo.toml with new version
    $newCargoTomlContent = $cargoTomlContent -replace ('version\s*=\s*"' + [regex]::Escape($versionLine) + '"'), ('version = "' + $newVersion + '"')
    Set-Content -Path $cargoTomlPath -Value $newCargoTomlContent
    Write-Output "✅ Updated version to $newVersion in Cargo.toml"
}

# Get the current date
$publishDate = Get-Date -Format "yyyy-MM-dd"

# Commit messages with publish date
if ($local) {
    $commitMessage = "🔧 Bump version to $newVersion ($publishDate)"
} else {
    $commitMessage = "🚀 Bump version to $newVersion ($publishDate) and release 📦"
}
$releaseMessage = "Release v$newVersion ($publishDate)"

# build in release mode and move the binaries to the release folder
# delete the release folder if it exists
$releaseFolder = "./release"
if (Test-Path $releaseFolder) {
    try {
        Remove-Item -Recurse -Force $releaseFolder
    } catch {
        Write-Output "❌ Failed to remove existing release folder"
        exit 1
    }
}
# create a release folder if it doesn't exist
if (-not (Test-Path $releaseFolder)) {
    try {
        New-Item -ItemType Directory -Path $releaseFolder | Out-Null
    } catch {
        Write-Output "❌ Failed to create release folder"
        exit 1
    }
}

# Function to check if a target is installed
function Test-TargetInstalled {
    param (
        [string]$target
    )
    $installedTargets = rustup target list | Where-Object { $_ -like "*installed*" }
    return $installedTargets -like "*$target*"
}


# build for windows
Write-Output "🔨 Building Windows binary..."
if (Test-TargetInstalled "x86_64-pc-windows-msvc") {
    cargo build --release --bin prime --target x86_64-pc-windows-msvc 
    if ($LASTEXITCODE -eq 0) {
        Write-Output "✅ Successfully built Windows binary"
    } else {
        Write-Output "❌ Failed to build Windows binary"
        exit 1
    }
} else {
    Write-Output "⚠️ Windows target (x86_64-pc-windows-msvc) not installed. Skipping Windows build."
    Write-Output "To install the target, run: rustup target add x86_64-pc-windows-msvc"
}





# Add ALL files to git
Write-Output "📝 Adding files to git..."
git add .
if ($LASTEXITCODE -ne 0) {
    Write-Output "❌ Failed to add files to git"
    exit 1
}

# Commit the change with the commit message
Write-Output "📝 Committing changes..."
git commit -m "$commitMessage"
if ($LASTEXITCODE -ne 0) {
    Write-Output "❌ Failed to commit changes"
    exit 1
}

# Tag the commit as a release with the release message
Write-Output "🔖 Creating tag v$newVersion..."
git tag -a "v$newVersion" -m "$releaseMessage"
if ($LASTEXITCODE -ne 0) {
    Write-Output "❌ Failed to create tag"
    exit 1
}

if ($local) {
    Write-Output "🏠 Running in local mode, building binaries for Windows and Linux..."

    # Build for Windows
    Write-Output "🔨 Building Windows binary for local release..."
    cargo build --release --bin prime --target x86_64-pc-windows-msvc
    $windowsBuildSuccess = $LASTEXITCODE -eq 0

    # Build for Linux
    $linuxBuildSuccess = $true
    if (Test-TargetInstalled "x86_64-unknown-linux-gnu") {
        Write-Output "🔨 Building Linux binary for local release..."
        cargo build --release --bin prime --target x86_64-unknown-linux-gnu
        $linuxBuildSuccess = $LASTEXITCODE -eq 0
    } else {
        Write-Output "⚠️ Linux target (x86_64-unknown-linux-gnu) not installed. Skipping Linux build."
        Write-Output "To install the target, run: rustup target add x86_64-unknown-linux-gnu"
        $linuxBuildSuccess = $false
    }

    # Check if at least one build succeeded
    if (-not ($windowsBuildSuccess -or $linuxBuildSuccess)) {
        Write-Output "❌ Failed to build any binaries for local release"
        exit 1
    }

    # Create a new release
    try {
        $releaseId = New-RandomGuid
        $releasePath = "releases/$releaseId"
        New-Item -ItemType Directory -Path $releasePath | Out-Null
    } catch {
        Write-Output "❌ Failed to create release directory"
        exit 1
    }

    # Copy Windows binary to release directory if build succeeded
    if ($windowsBuildSuccess) {
        try {
            $windowsBinaryPath = "./target/x86_64-pc-windows-msvc/release/prime.exe"
            Copy-Item -Path $windowsBinaryPath -Destination "$releasePath/prime-windows.exe"
        } catch {
            Write-Output "❌ Failed to copy Windows binary to release directory"
            exit 1
        }
    }

    # Copy Linux binary to release directory if build succeeded
    if ($linuxBuildSuccess -and (Test-TargetInstalled "x86_64-unknown-linux-gnu")) {
        try {
            $linuxBinaryPath = "./target/x86_64-unknown-linux-gnu/release/prime"
            Copy-Item -Path $linuxBinaryPath -Destination "$releasePath/prime-linux"
        } catch {
            Write-Output "❌ Failed to copy Linux binary to release directory"
            exit 1
        }
    }

    Write-Output "🎉 Release v$newVersion completed locally! Binaries are available in $releasePath"
    exit 0
}

# Push the commit and tag to your repository
Write-Output "🎉 Pushing changes and tags to the repository..."
# Try to push normally first
Write-Output "📤 Pushing commits..."
git push
if ($LASTEXITCODE -ne 0) {
    # If push fails, try to set upstream and push
    Write-Output "⚠️ Setting upstream branch and pushing..."
    $branchName = git rev-parse --abbrev-ref HEAD
    git push --set-upstream origin $branchName
    if ($LASTEXITCODE -ne 0) {
        Write-Output "❌ Failed to push commits"
        exit 1
    }
}

Write-Output "📤 Pushing tags..."
git push --tags
if ($LASTEXITCODE -ne 0) {
    Write-Output "❌ Failed to push tags"
    exit 1
}

# Check if CARGO_TOKEN is available
$cargoToken = $env:CARGO_TOKEN
if (-not $cargoToken) {
    Write-Output "⚠️ CARGO_TOKEN not found in environment variables. Skipping publishing to crates.io."
} else {
    # Publish the package to crates.io
    Write-Output "📦 Publishing package to crates.io..."
    cargo publish
    if ($LASTEXITCODE -eq 0) {
        Write-Output "✨ Package successfully published to crates.io!"
    } else {
        Write-Output "❌ Failed to publish package to crates.io."
        Write-Output "Please check the output above for more details."
        exit 1
    }
}

Write-Output "🎉 Release v$newVersion completed!"