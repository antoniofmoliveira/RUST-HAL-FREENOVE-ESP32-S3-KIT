# Rust ESP32 Example

    // generator version: 1.3.0
    // generator parameters: --chip esp32 -o esp32-wrover-e -o vscode -o unstable-hal -o esp-backtrace
```bash

esp-generate --chip esp32 project
/home/oliveira/export-esp.sh
cd project
code .
```

## settings.json

```json
{
    "editor.selectionClipboard": false,
}
```

## list ports

```bash
cargo espflash list-ports 
```

## flash

```bash
cargo espflash flash --baud=921600 --monitor /dev/ttyUSB0
```

