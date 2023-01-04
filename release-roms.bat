cargo build --examples --release

arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/examples/ex01 target/ex01.gba
gbafix -p -t target/ex01.gba
