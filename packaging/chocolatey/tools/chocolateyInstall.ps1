$ErrorActionPreference = 'Stop'

$packageName = 'vaultship'
$toolsDir    = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url64       = 'https://github.com/cyberxdefend/vaultship/releases/download/VERSION_PLACEHOLDER/vaultship-VERSION_PLACEHOLDER-x86_64-pc-windows-msvc.zip'

$packageArgs = @{
    packageName    = $packageName
    unzipLocation  = $toolsDir
    url64bit       = $url64
    checksum64     = 'SHA256_PLACEHOLDER'
    checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs
