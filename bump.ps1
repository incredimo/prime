#!/usr/bin/env pwsh
# bump.ps1 - Automates version bumping, committing, tagging, and pushing changes

param (
    [Parameter(Mandatory=$false)]
    [ValidateSet("major", "minor", "patch")]
    [string]$VersionType = "patch",
    
    [Parameter(Mandatory=$false)]
    [string]$CommitMessage = "",
    
    [Parameter(Mandatory=$false)]
    [switch]$DryRun = $false
)

# Configuration
$VersionFile = "version.txt"
$DefaultCommitMessage = "Bump version to {0}"

# Function to get the current version
function Get-CurrentVersion {
    if (Test-Path $VersionFile) {
        $version = Get-Content $VersionFile -Raw
        return $version.Trim()
    } else {
        # If version file doesn't exist, create it with initial version
        $initialVersion = "0.1.0"
        Set-Content -Path $VersionFile -Value $initialVersion
        return $initialVersion
    }
}

# Function to bump the version
function Bump-Version {
    param (
        [string]$CurrentVersion,
        [string]$Type
    )
    
    $parts = $CurrentVersion.Split('.')
    if ($parts.Length -ne 3) {
        Write-Error "Invalid version format. Expected format: X.Y.Z"
        exit 1
    }
    
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($Type) {
        "major" {
            $major++
            $minor = 0
            $patch = 0
        }
        "minor" {
            $minor++
            $patch = 0
        }
        "patch" {
            $patch++
        }
    }
    
    return "$major.$minor.$patch"
}

# Function to execute a command
function Invoke-GitCommand {
    param (
        [string]$Command,
        [switch]$DryRun
    )
    
    if ($DryRun) {
        Write-Host "Would execute: $Command" -ForegroundColor Yellow
    } else {
        Write-Host "> $Command" -ForegroundColor Cyan
        Invoke-Expression $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Command failed with exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }
    }
}

# Main script execution
$currentVersion = Get-CurrentVersion
$newVersion = Bump-Version -CurrentVersion $currentVersion -Type $VersionType

Write-Host "Current version: $currentVersion" -ForegroundColor Green
Write-Host "New version: $newVersion" -ForegroundColor Green

# Prepare commit message
if ([string]::IsNullOrEmpty($CommitMessage)) {
    $CommitMessage = $DefaultCommitMessage -f $newVersion
}

# Update version file
if ($DryRun) {
    Write-Host "Would update $VersionFile with new version: $newVersion" -ForegroundColor Yellow
} else {
    Set-Content -Path $VersionFile -Value $newVersion
    Write-Host "Updated $VersionFile with new version: $newVersion" -ForegroundColor Green
}

# Check if we're in a git repository
$isGitRepo = git rev-parse --is-inside-work-tree 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "Not in a git repository. Please run this script from a git repository."
    exit 1
}

# Stage changes
Invoke-GitCommand -Command "git add ." -DryRun:$DryRun

# Commit changes
Invoke-GitCommand -Command "git commit -m `"$CommitMessage`"" -DryRun:$DryRun

# Create tag
$tagName = "v$newVersion"
Invoke-GitCommand -Command "git tag -a $tagName -m `"Version $newVersion`"" -DryRun:$DryRun

# Push changes and tags
Invoke-GitCommand -Command "git push" -DryRun:$DryRun
Invoke-GitCommand -Command "git push --tags" -DryRun:$DryRun

Write-Host "Successfully bumped version to $newVersion and pushed changes!" -ForegroundColor Green
