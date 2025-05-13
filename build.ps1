#!/usr/bin/env pwsh
# build.ps1 - Builds prime.sh from source files

# Output file
$outputFile = "prime.sh"

# Function to append content to the output file
function Append-Content {
    param (
        [string]$FilePath,
        [string]$OutputFile
    )

    if (Test-Path $FilePath) {
        Get-Content $FilePath | Out-File -Append -FilePath $OutputFile -Encoding utf8
    } else {
        Write-Error "File not found: $FilePath"
        exit 1
    }
}

# Function to append a directory of files in alphabetical order
function Append-Directory {
    param (
        [string]$DirectoryPath,
        [string]$OutputFile,
        [string]$FilePattern = "*.*"
    )

    if (Test-Path $DirectoryPath) {
        $files = Get-ChildItem -Path $DirectoryPath -Filter $FilePattern | Sort-Object Name
        foreach ($file in $files) {
            Write-Host "Including: $($file.FullName)"
            Append-Content -FilePath $file.FullName -OutputFile $OutputFile
            # Add a newline after each file
            "`n" | Out-File -Append -FilePath $OutputFile -Encoding utf8
        }
    } else {
        Write-Error "Directory not found: $DirectoryPath"
        exit 1
    }
}

# Start with a clean output file
Write-Host "Creating $outputFile..."
"" | Out-File -FilePath $outputFile -Encoding utf8

# Build the script in the correct order
Write-Host "Building prime.sh from source files..."

# 1. Header
Write-Host "Adding header..."
Append-Content -FilePath "src/header/main.sh" -OutputFile $outputFile

# 2. Utility functions
Write-Host "Adding utility functions..."
Append-Directory -DirectoryPath "src/functions" -OutputFile $outputFile -FilePattern "*.sh"

# 3. Ollama installation and service
Write-Host "Adding Ollama components..."
Append-Directory -DirectoryPath "src/ollama" -OutputFile $outputFile -FilePattern "*.sh"

# 4. Python environment setup
Write-Host "Adding Python setup..."
Append-Directory -DirectoryPath "src/python" -OutputFile $outputFile -FilePattern "setup.sh"

# 5. Python logger module
Write-Host "Adding Python logger module..."
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# 8.  immutable logger (never self-modified)" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"cat > infra/logger.py <<'PY'" | Out-File -Append -FilePath $outputFile -Encoding utf8
Append-Content -FilePath "src/python/logger.py" -OutputFile $outputFile
"PY" | Out-File -Append -FilePath $outputFile -Encoding utf8
"`n" | Out-File -Append -FilePath $outputFile -Encoding utf8

# 6. Python agent
Write-Host "Adding Python agent..."
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# 9.  agent.py  —  now powered by Ollama with Web UI" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"cat > agent.py <<'PY'" | Out-File -Append -FilePath $outputFile -Encoding utf8
Append-Content -FilePath "src/python/agent.py" -OutputFile $outputFile
"PY" | Out-File -Append -FilePath $outputFile -Encoding utf8
"chmod +x agent.py" | Out-File -Append -FilePath $outputFile -Encoding utf8
"`n" | Out-File -Append -FilePath $outputFile -Encoding utf8

# 7. UI components
Write-Host "Adding UI components..."
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# 10. Create UI templates" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"log `"Creating UI templates...`"" | Out-File -Append -FilePath $outputFile -Encoding utf8
"`n" | Out-File -Append -FilePath $outputFile -Encoding utf8

# 7.1 CSS
"# Create CSS file" | Out-File -Append -FilePath $outputFile -Encoding utf8
"mkdir -p `"`$WORKDIR/ui/static`"" | Out-File -Append -FilePath $outputFile -Encoding utf8
"cat > `"`$WORKDIR/ui/static/styles.css`" <<'CSS'" | Out-File -Append -FilePath $outputFile -Encoding utf8
Append-Content -FilePath "src/ui/styles.css" -OutputFile $outputFile
"CSS" | Out-File -Append -FilePath $outputFile -Encoding utf8
"`n" | Out-File -Append -FilePath $outputFile -Encoding utf8

# 7.2 JavaScript
"# Create JavaScript file for UI functionality" | Out-File -Append -FilePath $outputFile -Encoding utf8
"cat > `"`$WORKDIR/ui/static/app.js`" <<'JS'" | Out-File -Append -FilePath $outputFile -Encoding utf8
Append-Content -FilePath "src/ui/app.js" -OutputFile $outputFile
"JS" | Out-File -Append -FilePath $outputFile -Encoding utf8
"`n" | Out-File -Append -FilePath $outputFile -Encoding utf8

# 7.3 HTML Templates
$templates = Get-ChildItem -Path "src/ui/templates" -Filter "*.html" | Sort-Object Name
foreach ($template in $templates) {
    $templateName = $template.Name
    $templateTitle = $templateName -replace "\.html$", ""

    "# $templateTitle.html template" | Out-File -Append -FilePath $outputFile -Encoding utf8
    "cat > `"`$WORKDIR/ui/templates/$templateName`" <<'HTML'" | Out-File -Append -FilePath $outputFile -Encoding utf8
    Append-Content -FilePath $template.FullName -OutputFile $outputFile
    "HTML" | Out-File -Append -FilePath $outputFile -Encoding utf8
    "`n" | Out-File -Append -FilePath $outputFile -Encoding utf8
}

# 8. Scripts
Write-Host "Adding scripts..."
Append-Directory -DirectoryPath "src/scripts" -OutputFile $outputFile -FilePattern "*.sh"

# 9. Final message
Write-Host "Adding final message..."
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# 12.  Start the agent" | Out-File -Append -FilePath $outputFile -Encoding utf8
"# ------------------------------------------------------------" | Out-File -Append -FilePath $outputFile -Encoding utf8
Append-Content -FilePath "src/header/footer.sh" -OutputFile $outputFile

# Make the output file executable
Write-Host "Making $outputFile executable..."
if ($IsLinux -or $IsMacOS) {
    chmod +x $outputFile
}

Write-Host "Build completed successfully!"
Write-Host "Generated: $outputFile"
