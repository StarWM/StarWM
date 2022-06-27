<p align="center">
  <a href="#">
    <img src="media/banner.jpg" width="100%">
  </a>
</p>

# Installation

Installation assumes that you already have installed the latest rust toolchain. If not please install it from here: https://rust-lang.org. 

# Dependencies:
You will also need to have several X development libraries. They're usually under something like `libxcb-devel` and `libx11-devel`. These dependancies have different names under different package managers, however we have listed them down below.

#### Debian-based dependancy install (includes Ubuntu):
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
If you want to use graphical desktop managers, should copy the `starwm.dekstop` file to `/usr/share/xsessions/`. You will then see starwm in your desktops list:

```sh
sudo cp starwm.desktop /usr/share/xsessions/
```

Another way to get it started is to add this:

```sh
exec starwm
```

To the bottom of your `~/.xinitrc` file.

From here you can either reboot into it, provided that you have disabled your desktop manager and added `startx` to your shells profile file.
Alternatively, you can boot into a TTY and then run `startx` and you'll be good to go.

### If you followed these steps correctly you should have installed StarWM on your machine!
