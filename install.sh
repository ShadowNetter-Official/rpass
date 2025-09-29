#!/bin/bash

echo
echo "rpass"
echo "by ShadowNetter"
echo
echo "cloning into repo..."
git clone https://github.com/ShadowNetter-Official/rpass
cd rpass
echo "done"
echo "installing..."
cargo build --release
sudo cp target/release/rpass /bin/
echo "done"
echo
echo "to uninstall do: "
echo "sudo rm /bin/rpass"
echo
