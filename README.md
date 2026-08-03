# Coding in Rust and Esp32 hal the Freenove Esp32-S3 Kit projects

Work in progress.

I use the Esp32 WROVER-E board with the GPIO expansion board.

I use esp-hal 1.1.0.

Some cannot be done because I dont have the hardware to test them.

## Observations

- generator parameters: --chip esp32 -o esp32-wrover-e -o unstable-hal -o log -o esp-backtrace -o vscode -o zed
- The project 21.2-ultrasonic_ranging with hcsr04 driver dont work correctly.
- The project 7.2-alertor needs the Buzzer lib that wants hall 1.0 but compiles locally without problems. Then the code dont need to be ported to 1.1.0.
- The pins used in the projects DONT are the same as the ones in the book that acompanies the kit because I dont have that kit. But the connectios are the same. Only pins are altered.

## How to test

- install the rust toolchain
- install the espup tool
- in each project

    ```bash
    cd project
    ~/export-esp.sh
    code . # or zed .
    cargo run # it will compile and flash the esp32
    ```
