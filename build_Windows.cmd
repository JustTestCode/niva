
setlocal enabledelayedexpansion

for /f "tokens=* delims=" %%v in ('git describe --tags --always') do set "VERSION=%%v"
set "VERSION=!VERSION:.=_!"

rd /s /q dist
mkdir dist

yarn
cd packages\devtools
rd /s /q build
call npm run build
cd ..\..

rd /s /q target\release
cargo build --release

target\release\tauri_lite.exe ^
	--resource-dir=packages\devtools\build ^
	--project=packages\devtools\build ^
	--build=dist\TauriLiteDevTools.exe

powershell Compress-Archive -Path dist\TauriLiteDevTools.exe -DestinationPath dist\TauriLiteDevTools_%VERSION%_Windows.zip