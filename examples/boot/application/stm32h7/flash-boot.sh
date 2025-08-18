#!/bin/bash
probe-rs erase --chip STM32H755ZITx
mv ../../bootloader/stm32/memory.x ../../bootloader/stm32/memory-old.x
cp memory-bl.x ../../bootloader/stm32/memory.x

cargo +nightly flash --manifest-path ../../bootloader/stm32/Cargo.toml --release --features embassy-stm32/stm32h755zi-cm7 --chip STM32H755ZITx --target thumbv7em-none-eabihf

rm ../../bootloader/stm32/memory.x
mv ../../bootloader/stm32/memory-old.x ../../bootloader/stm32/memory.x
