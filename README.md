# prettyblk
lsblk, but prettier

![img](https://github.com/fobbidennotis/prettyblk/blob/master/assets/pblk.jpg?raw=true)

# Installation
## Compiling from source
Requirements
- Rust
- Cargo

Clone the repostiory
```
git clone https://github.com/fobbidennotis/prettyblk.git
cd prettyblk
```

Compile
```
cargo build --release
```

Copy the binary  to use system-wide
```
sudo cp ./target/release/pblk /usr/bin
```
Use 
```
pblk
```
## Downloading release
```
sudo curl -L https://github.com/fobbidennotis/prettyblk/releases/download/1.0/pblk -o /usr/bin/pblk && sudo chmod +x /usr/bin/pblk
pblk
```
