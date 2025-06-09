# Prime Response
Timestamp: 2025-06-09 12:08:43

```{{.powershell data-action="execute"}}
try {
    $ipAddress = Invoke-Command -ComputerName localhost -ScriptBlock { Get-NetIPAddress | Where-Object {$_.Address -like '*.*.*.*'} | Select-Object -ExpandProperty Address }
    Write-Host "Your IP address is: $ipAddress"
} catch {
    Write-Error "Failed to retrieve IP address: $($_.Exception.Message)"
}
```
