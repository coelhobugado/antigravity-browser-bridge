[CmdletBinding(SupportsShouldProcess = $true)]
param([switch]$Apply)

$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$targets = @("cli\target", "node_modules", "packages\dashboard\node_modules", ".pnpm-store", "artifacts")
$manifest = Join-Path $env:LOCALAPPDATA "agent-browser\native-messaging\host-manifest.json"
$installedExecutable = $null
if (Test-Path -LiteralPath $manifest) {
  try { $installedExecutable = (Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json).path } catch { Write-Warning "Could not inspect native host manifest; cli\target will be preserved." }
}

foreach ($relative in $targets) {
  $target = Join-Path $workspace $relative
  if (-not (Test-Path -LiteralPath $target)) { Write-Output "[missing] $relative"; continue }
  $resolved = (Resolve-Path -LiteralPath $target).Path
  if ($resolved -notlike "$workspace\*") { throw "Refusing target outside workspace: $resolved" }
  $size = (Get-ChildItem -LiteralPath $resolved -Recurse -File -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum
  $protected = $installedExecutable -and ((Resolve-Path -LiteralPath $installedExecutable -ErrorAction SilentlyContinue).Path -like "$resolved\*")
  if ($protected) { Write-Warning "[protected] $relative contains the installed native host; rebuild or reinstall before cleaning."; continue }
  if ($Apply) { if ($PSCmdlet.ShouldProcess($resolved, "Remove cache")) { Remove-Item -LiteralPath $resolved -Recurse -Force } } else { Write-Output ("[dry-run] {0} ({1:N0} bytes)" -f $relative, $size) }
}
