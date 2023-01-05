cargo build --examples --release

arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/examples/min1 target/min1.gba
gbafix -p -t target/min1.gba

arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/examples/min1 target/min2.gba
gbafix -p -t target/min2.gba
