
function ParseZedWorkspace {
    $metadata = cargo metadata --no-deps --offline --format-version=1 | ConvertFrom-Json
    $env:ZED_WORKSPACE = $metadata.workspace_root
    $env:RELEASE_VERSION = $metadata.packages | Where-Object { $_.name -eq "zed" } | Select-Object -ExpandProperty version
}
