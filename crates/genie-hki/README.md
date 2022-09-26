# genie-hki

[![docs.rs](https://img.shields.io/badge/docs.rs-genie--hki-blue?style=flat-square&color=blue)](https://docs.rs/genie-hki/)
[![crates.io](https://img.shields.io/crates/v/genie-hki.svg?style=flat-square&color=orange)](https://crates.io/crates/genie-hki)
[![GitHub license](https://img.shields.io/github/license/SiegeEngineers/genie-rs?style=flat-square&color=darkred)](https://github.com/SiegeEngineers/genie-rs/blob/default/LICENSE.md)
![MSRV](https://img.shields.io/badge/MSRV-1.64.0%2B-blue?style=flat-square)

Read Age of Empires 2 hotkey files.

## Hotkeys Descriptions in Language Files

Hotkeys have a keycode that the game translates into a string for displaying in
the hotkey menu.
Some of these strings are contained in the language file.
Other keys are turned into strings using the [Windows API `GetKeyNameTextA`](https://docs.microsoft.com/en-us/windows/desktop/api/winuser/nf-winuser-getkeynametexta).

The following list summarizes the strings that are displayed for each keycode in the hotkey menu for HD.
Unlisted keycodes are blank or whitespace in the hotkey menu.

* 0 -> ???
* 3 -> Scroll Lock
* 8 -> Backspace
* 9 -> Tab
* 12 -> Num 5
* 13 -> Enter
* 16 -> Shift
* 17 -> Ctrl
* 18 -> Alt
* 20 -> Caps Lock
* 27 -> Esc
* 32 -> Space
* 33 -> Page Up
* 34 -> Page Down
* 35 -> End
* 36 -> Home
* 37 -> Left
* 38 -> Up
* 39 -> Right
* 40 -> Down
* 44 -> Sys Req
* 45 -> Insert
* 46 -> Delete
* 47 -> ?UnknownKey?
* 48 -> 0
* 49 -> 1
* ...
* 57 -> 9
* 65 -> A
* 66 -> B
* ...
* 90 -> Z
* 91, 92, 93, 95 -> ?UnknownKey?
* 96 -> Num 0
* 97 -> Num 1
* ... (including another Num 5)
* 105 -> Num 9
* 106 -> Num *
* 107 -> Num +
* 109 -> Num -
* 110 -> Num Del
* 111 -> Num /
* 112 -> F1
* ...
* 120 -> F9 (Note 121 is blank, not F10)
* 122 -> F11
* ...
* 135 -> F24
* 144 -> Pause
* 145 -> Scroll Lock
* 160 -> Shift
* 161 -> Shift
* 162 -> Ctrl
* 163 -> Ctrl
* 164 -> Alt
* 165 -> Alt
* 166 -> ?UnknownKey?
* ...
* 171 -> ?UnknownKey?
* 172 -> M
* 173 -> D
* 174 -> C
* 175 -> B
* 176 -> P
* 177 -> Q
* 178 -> J
* 179 -> G
* 180 -> ?UnknownKey?
* 181 -> ?UnknownKey?
* 182 -> ?UnknownKey?
* 183 -> F
* 186 -> ;
* 187 -> =
* 188 -> ,
* 189 -> -
* 190 -> .
* 191 -> /
* 192 -> `
* 193 -> ?UnknownKey?
* 194 -> F15 (again)
* 220 -> \
* 221 -> ]
* 222 -> '
* 226 -> \ (again)
* 233 -> ?UnknownKey?
* 234 -> ?UnknownKey?
* 235 -> ?UnknownKey?
* 237 -> ?UnknownKey?
* 238 -> ?UnknownKey?
* 241 -> ?UnknownKey?
* 243 -> ?UnknownKey?
* 245 -> ?UnknownKey?
* 249 -> ?UnknownKey?
* 251 -> Extra Button 2
* 252 -> Extra Button 1
* 253 -> Middle Button
* 254 -> Wheel Down
* 255 -> Wheel Up

## License

[GPL-3.0](../../LICENSE.md)
