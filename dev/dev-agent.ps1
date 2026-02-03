$env:TAURI_DEV_MCP_PORT = (Get-Random -Minimum 5000 -Maximum 65535)
$env:TAURI_DEV_PORT = (Get-Random -Minimum 5000 -Maximum 65535)
$env:TAURI_CLI_PORT = (Get-Random -Minimum 5000 -Maximum 65535)

Write-Output "MCP Port: $env:TAURI_DEV_MCP_PORT"
Write-Output "Frontend Port: $env:TAURI_DEV_PORT"
Write-Output "CLI Port: $env:TAURI_CLI_PORT"


bun dev --config "{`\`"build`\`": { `\`"devUrl`\`": `\`"http://localhost:$env:TAURI_DEV_PORT`\`"}}"