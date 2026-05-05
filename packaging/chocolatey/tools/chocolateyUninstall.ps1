$ErrorActionPreference = 'Stop'

$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
Remove-Item "$toolsDir\vaultship.exe" -Force -ErrorAction SilentlyContinue
