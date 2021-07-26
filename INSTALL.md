# Installation

Installation assumes that you already have installed the latest rust toolchain. If not please install it from here: https://rust-lang.org. 

# Dependancies:
You will also need to have several X development libraries. They're usually under something like `libxcb-devel` and `libx11-devel`. These dependancies have different names under different package managers, however we have listed them down below.

#### Ubuntu-based dependancy install:
```sh
sudo apt-get install libxcb-randr0-dev libxcb-xtest0-dev libxcb-xinerama0-dev libxcb-shape0-dev libxcb-xkb-dev libx11-dev rofi
```

#### Fedora dependancy install:
```sh
sudo dnf install libxcb-devel libX11-devel rofi alacritty
```

StarWM also requires [Alacritty](https://github.com/alacritty/alacritty/blob/master/INSTALL.md#debianubuntu) at the moment to function as intended, which isn't available on certain package managers (such as Ubuntu-based distributions).

# Build from source

```sh
git clone https://github.com/StarWM/StarWM.git
cd StarWM
cargo build --release
sudo cp target/release/starwm /usr/bin/starwm
```

After following the above guide, you will have installed StarWM.
If you wish to add it into a desktop manager, then you can't just yet.

The best way to get it started is to add this to your ~/.xinitrc file:

```sh
exec starwm
```
