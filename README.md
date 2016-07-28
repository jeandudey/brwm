# BRWM - Basic Rust Window Manager
BRWM is a [1][stacking window manager] designed to be blazing fast using modern techonogies like [2][Rust] and [3][XCB].

This project is in an early stage of development so there are tons of bugs (or features) to play with. 

[1]: https://en.wikipedia.org/wiki/Stacking_window_manager
[2]: https://www.rust-lang.org/en-US/
[3]: https://xcb.freedesktop.org/

## Usage
First of all you need to have:
```
Rust >= 1.9
libxcb >= 1.12
xorg-server-xephyr >= 1.18
```

First clone the project:
```bash
git clone https://github.com/jeandudey/brwm
cd brwm
```

Then compile and run it:
```bash
chmod +x ./run.sh
./run.sh --release
```

## License
GPL-3.0
```
BRWM - Basic Rust Window Manager
Copyright (C) 2016  Jean Pierre Dudey

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
```
