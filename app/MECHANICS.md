# X11 Studio — mechanics sheet (verified)

Everything below was read from the original program itself, not guessed from
the XML alone. The program is `AttackShark.exe` (32-bit PE, sha256
`e0d78c3a…879daa`) from the installer payload. Each fact names the function it
came from; the pseudo-C was produced by the Deep engine (Ghidra 12.1.4).

The rebuild now talks to a physical X11 through its vendor HID collection.
Reading uses the `0xA0` read-permission handshake and writing sends whole
feature reports. The layouts are documented in §11; they were checked against
a connected `1D57:FA60` receiver with a read → write-identical → read-back test.

## 0. The page and its controls

`FUN_0049db00` (`0x49db00`) creates the X11 page. It resolves these controls by
name, and every later handler works on those pointers:

| Purpose | Control names | Profile offsets |
|---|---|---|
| 8 DPI sliders / labels / edits / radios / colour buttons | `ms_4_slider_dpi_1..8`, `ms_4_label_dpi_1..8`, `ms_4_edit_dpi_1..8`, `ms_4_option_dpi_step_1..8`, `ms_4_btn_dpi_color_1..8` | `0x8c4`, `0x960`, `0x304` |
| Lighting | `ms_4_combo_light`, `ms_4_slider_brightness`, `ms_4_slider_breathspeed`, `ms_4_btn_light_color` | `0x94c`, `0x924`, `0x928`, `0x95c` |
| Polling | `ms_4_btn_prate_1..4`, `ms_4_label_prate` | `0x938` |
| Performance | `ms_4_slider_key_debounce`, four option pairs | `0x958`, `0x8b4/0x940/0x944/0x948` |
| Power | `ms_4_slider_light_off`, `ms_4_slider_sleep_timer`, `ms_4_check_movemode` | `0x930`, `0x92c`, `0x93c` |
| Buttons | `ms_4_btn_key_1..18` | 18 × `0x4f8` bytes from `0x9a0` |
| Profiles | `ms_4_list_profile`, add/delete/rename/import/export/reset | vector at `+0x50cd0` |

The original window is 1127 × 760 (`res/main.xml`, line 2). Its accent colours
are red `0xFFE2211C` and cyan `0xFF28B4E6`. The X11 entry is
`option_device_4` (`X11 Mouse`) and its virtual window is `ver_ms_4`, which
includes `res/MS/MS_4/ms_4_page.xml`.

## 1. DPI

`FUN_004a07d0` (`0x4a07d0`) converts the number typed into a stage edit box into
a slider position. It is not a linear range:

```
dpi < 51            -> position 1
51 <= dpi <= 10000  -> position = dpi / 50      (integer division)
10001 <= dpi < 22000 -> position = 200 + (dpi - 10000) / 100
dpi >= 22000        -> position = 320           (22000 is clamped first)
```

The inverse, used by the rebuild for the sliders:

```
position 1..200     -> dpi = position * 50
position 201..320   -> dpi = 10000 + (position - 200) * 100
```

Because of the integer division, typed values between 51 and 99 collapse to
position 1 (50 DPI), and 10001–10099 collapse to position 200 (10000 DPI). The
original then overwrites the edit box with the position-derived value.

`FUN_0049e7a0` (`0x49e7a0`) walks **eight** stage slots. For each slot it reads
the stored position at `profile + 0x8c4 + i*4`, clamps it to the slider's own
min/max (1..320) and, if out of range, replaces it with the compiled table
`DAT_004ec398 = 16, 32, 48, 64, 100, 320, 0, 0`. It then sets the slider, the
label and the edit box, and paints the colour button from
`profile + 0x960 + i*4`.

`FUN_004a4670` (`0x4a4670`, "restore defaults") writes the factory stage data:

| Slot | Position | DPI | Enabled | Colour |
|---|---|---|---|---|
| 1 | 16 | 800 | yes | `0x0000FF` (blue) |
| 2 | 32 | 1600 | yes | `0x00FF00` (green) |
| 3 | 48 | 2400 | yes | `0xFF0000` (red) |
| 4 | 64 | 3200 | yes | `0x00FFFF` (cyan) |
| 5 | 100 | 5000 | yes | `0xFFFF00` (yellow) |
| 6 | 320 | 22000 | yes | `0xFF00FF` (magenta) |
| 7 | 0 | — | no | `0x0040FF` |
| 8 | 0 | — | no | `0xFFFFFF` |

The enable flags are at `profile + 0x904 + i*4` (1,1,1,1,1,1,0,0 by default).
The page hides stages 7 and 8 with `visible="false"`, and the active-stage
index (`profile + 0x934`) defaults to **1** (the second slot). The on-mouse DPI
button handler `FUN_004a3000` (`0x4a3000`) also allows stage numbers 1..8.

`FUN_0049f040` (`0x49f040`), the "Reset DPI" branch, resets the same eight
positions, eight colours and eight enable flags, and sets the active stage
(`profile + 0x934`) back to 1. That is what the original button does, even
though the label only mentions DPI.

**In the rebuild:** 8 slots, first 6 enabled, factory positions above, colours
above, `DPI_MIN = 50`, `DPI_MAX = 22000`, 320 positions.

## 2. Polling rate

`FUN_0049ffe0` (`0x49ffe0`) stores the selected radio index (0..3) in
`profile + 0x938` and updates `ms_4_label_prate`. `FUN_0049e7a0` clamps that
value to 0..3 and falls back to **3**. `FUN_004a4670` also restores 3.

| Index | Rate | Label (`EN.xml`) |
|---|---|---|
| 0 | 125 Hz | Power Saving |
| 1 | 250 Hz | Office |
| 2 | 500 Hz | Gaming |
| 3 | 1000 Hz | E-sports |

The device reports the rate as a raw period byte, not as an index.
`FUN_004a38a0` (`0x4a38a0`) and `FUN_004a3000` (`0x4a3000`) map it like this:

```
report byte 1 -> index 3 (1000 Hz)
report byte 2 -> index 2 (500 Hz)
report byte 4 -> index 1 (250 Hz)
report byte 8 -> index 0 (125 Hz)
anything else -> index 0
```

The query is feature-report class `6`, command `9`, 9 bytes long. It is retried
up to 5 times in `FUN_004a38a0`.

## 3. Lighting

`FUN_0049e7a0` clamps the mode at `profile + 0x94c` to **0..6** and falls back
to **0**. Rainbow Wave (index 7) is present in the language file but hidden in
the combo and rejected by this clamp. Seven modes are real:

| Index | `EN.xml` name |
|---|---|
| 0 | LED Off |
| 1 | Static |
| 2 | Breathing |
| 3 | Neon |
| 4 | Color Breathing |
| 5 | Static DPI |
| 6 | Breathing DPI |

Brightness is `profile + 0x924`, clamped **1..8**, fallback **8**. Breathing
speed is `profile + 0x928`, intended range **4..8**, fallback **6**.

There is a genuine bug in `FUN_0049e7a0`: when the speed is out of range it
writes `6` into `profile + 0x924` (the brightness field) and leaves `0x928`
unchanged, then sets the speed slider from the still-invalid value. The rebuild
implements the intended clamp (4..8) rather than reproducing the bug.

`FUN_004a4670` restores mode 0 (LED Off), brightness 8, speed 6 and the main
light colour `0x00FF00` (green). The BLE combo (`ms_4_combo_light_ble`) has its
own value at `0x950`, default 3.

The 16 swatches in `res/MS/MS_4/ms_4_lightpage.xml` are exactly:

```
00FFFF  6CFF00  7F00FF  007FFF  0000FF  7FFF00  FF5400  FF7F00
FFB400  FFFF00  FF0000  FF00FF  1EB40A  48B1FF  FF48B0  FFFFFF
```

## 4. Performance / sensor

`FUN_0049e7a0` fills these values; `FUN_0049ffe0` writes the option pairs; and
`FUN_004a4670` restores the defaults below.

| Setting | Range | Default | Profile |
|---|---|---|---|
| Key response / debounce | 2..25 ms | 4 ms | `0x958` |
| Ripple control | off (0) / on (1) | on | `0x940` |
| Angle snap | off (0) / on (1) | off | `0x944` |
| Motion sync | off (0) / on (1) | on | `0x948` |
| Lift-off distance | 1 mm (0) / 2 mm (1) | 1 mm | `0x8b4` |

When any slider moves, `FUN_004a04f0` (`0x4a04f0`) stores the new value and
restarts timer `0x231` with a **500 ms** delay. Combos use the same timer with
**200 ms**. The option/checkbox handler `FUN_004a02c0` uses timer `0x232` at
**10 ms** only to reveal the matching option row.

## 5. Power

`FUN_0049e7a0` clamps both timers to the slider's own 1..60 range.

| Setting | Range | Default | Profile |
|---|---|---|---|
| Light-off timer | 1..60 min | 1 min | `0x930` |
| Sleep timer | 1..60 min | 10 min | `0x92c` |
| Move wake | off (0) / on (1) | off | `0x93c` |

`ms_4_check_movemode` is `visible="false"` in the original page, and
`FUN_004a02c0` writes its state to `0x93c`. `FUN_004a4670` does not set `0x93c`,
so a fresh (zeroed) profile has Move Wake off. The rebuild keeps the switch
visible but defaults it to off.

Battery/status refresh: `FUN_004a38a0` sets timer `0x230` to **6000 ms**.
`FUN_0043bc50` (`0x43bc50`) starts three more 6000 ms timers (`0x2c0`, `0x2c4`,
`0x2c5`) for polling, DPI and lighting queries. The battery is read with feature
report id `0x0B`, 8 bytes, and stored as a 16-bit value at `+0x67634`
(`FUN_004a38a0`). Charging is inferred from `Open_FeatureDevice` in
`FUN_0049e4c0`. Which byte is the percentage is not yet proven, so the rebuild
shows the battery figure as a preview.

## 6. Buttons and command bytes

`FUN_0049db00` creates 18 button rows (`ms_4_btn_key_1..18`). The button record
stride is `0x4f8` bytes; the command byte is at `+0x9a1`, the shortcut key at
`+0x9a3`, the modifiers at `+0x9a7`, the macro index at `+0xdd0`, the macro mode
at `+0x9ac` and the macro loop count at `+0x9e0`.

The first five rows in the XML are `ms_4_btn_key_1`, `_2`, `_3`, `_7`, `_8` —
the five physical buttons of the X11 (left, right, middle, forward, back). The
rest are the remaining assignable slots.

`FUN_004a4670` initialises all 18 slots with the compiled table
`DAT_004ec8f0`:

```
slot:  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
byte: 01 02 03 00 00 06 04 05 00 00 00 00 00 00 00 00 0B 0C
```

So the factory map is: left click, right click, middle click, off, off,
DPI cycle, forward, backward, off ×8, scroll up, scroll down.

`FUN_004a09f0` (`0x4a09f0`) turns a chosen menu entry into one byte:

| Byte | Command | Byte | Command |
|---|---|---|---|
| `0x00` | Button off | `0x20` | Cut |
| `0x01` | Left click | `0x21` | Copy |
| `0x02` | Right click | `0x22` | Paste |
| `0x03` | Middle click | `0x23` | Open |
| `0x04` | Forward | `0x24` | Save |
| `0x05` | Backward | `0x25` | Find |
| `0x06` | DPI cycle | `0x26` | Redo |
| `0x07` | DPI + | `0x27` | Undo * |
| `0x08` | DPI − | `0x28` | Select all |
| `0x09` | Double click | `0x29` | Print |
| `0x0A` | Fire button | `0x2A` | Close window |
| `0x0B` | Scroll up | `0x2B` | Swap windows |
| `0x0C` | Scroll down | `0x2C` | Show desktop |
| `0x0D` | Scroll left * | `0x2D` | Run command |
| `0x0E` | Scroll right * | `0x2E` | Lock PC |
| `0x10` | Free shortcut | `0x32` | Screen capture |
| `0x11` | Macro | `0x33` | Task view * |
| `0x16` | Easy aim | `0x36` | Profile cycle * |
| `0x17` | LED loop * | `0x37` | Profile up * |
| `0x40`–`0x47` | Media player, play/pause, stop, previous, next, volume +, volume −, mute | `0x38` | Profile down * |
| `0x50`–`0x59` | Browser home, favourites, forward, backward, stop, refresh, search, e-mail, calculator, my computer | | |

`*` = label exists in `FUN_00483880`, but the X11 menu hides it or does not list
it. This table is from `FUN_004a09f0`; the labels are from `FUN_00483880`
(`0x483880`).

**Left-click rule.** After writing the new byte, `FUN_004a09f0` copies the first
16 command bytes and checks whether any is `0x01`. If none is, it opens a modal
message box with `one_click_text` ("At least one button must be \"Left
Click\"!") and aborts. Slot indices above 16 are ignored. This is a real rule,
not a suggestion.

**Shortcut encoding.** A free shortcut stores four modifier bits at `+0x9a7`
(1 = Ctrl, 2 = Shift, 4 = Alt, 8 = Win) and the key byte at `+0x9a3`. A macro
stores the vector index at `+0xdd0`, the mode at `+0x9ac` and the loop count at
`+0x9e0`; `0xFFFFFFFF` means "no macro".

## 7. Macros

`FUN_0049f040` creates the macro window with `local_62 = 10` (default delay
**10 ms**) and `local_60 = 1` (default loop count **1**). A macro name is up to
24 wide characters (`wcscpy_s(..., 0x18, ...)`) and its action block is
`0x2e` dwords = **184 bytes** (`FUN_004a09f0`).

The rebuild uses the same defaults and keeps its own ordered action list
(key down/up, mouse down/up, delay, move, wheel). The wheel action is a rebuild
extension; the original's macro model only shows key, mouse, delay and move in
the editor.

## 8. Profiles

Each profile is a fixed **`0x80e0` = 32,992-byte** block.

- `FUN_0049e4c0` (`0x49e4c0`) copies `0x80e0` bytes from the selected profile
  into the live block and refreshes the page.
- `FUN_0049edd0` (`0x49edd0`) allocates a zeroed `0x80e0` block, calls
  `FUN_004a4670` to fill the defaults, copies the name (100 wide characters),
  and appends it to the vector. There is no fixed maximum.
- `FUN_004a4970` (`0x4a4970`) saves the profile vector to
  `<app data>\ms_4\pro.data`.
- The profile list selection handler is `FUN_004a01c0` (`0x4a01c0`); it swaps
  the current profile, refreshes the page and posts the apply message.

## 9. Apply and timers

The two Apply buttons (`ms_4_apply_btn` and `ms_4_btn_apply_dpi`) do not change
the model. In `FUN_0049f040` they fall through to
`PostMessageW(hwnd, 0x635)`; the values were already stored live by the slider,
combo and option handlers. The rebuild follows the same idea: edits are stored
immediately and the "apply" action is a visual confirmation only.

Timer summary:

| Timer | Period | Set by | Purpose |
|---|---|---|---|
| `0x231` | 500 ms | `FUN_004a04f0` | push slider changes |
| `0x231` | 200 ms | `FUN_004a01c0` | push combo changes |
| `0x232` | 10 ms | `FUN_004a02c0` | reveal the selected option row |
| `0x230` | 6000 ms | `FUN_004a38a0` | refresh battery/settings |
| `0x2c0/0x2c4/0x2c5` | 6000 ms | `FUN_0043bc50` | polling/DPI/lighting query |

## 10. App-side choices

These are rebuild choices, not decoded original rules:

- The "latency" figure `1000 / polling_hz + debounce` ms is an estimate.
- The Precision and Gaming profiles are rebuild presets; the original's factory
  profile data is the table in §1.
- The UI exposes the five physical buttons (hardware ids 1, 2, 3, 7, 8)
  rather than all 18 slots; the full factory map is documented above and stored
  in `mechanics.json`.
- Stage colours are stored as screen RGB. The original stores `COLORREF`
  values (`0x00BBGGRR`), so its `0x0000FF` is displayed as red.
- The active DPI stage is editable in the DPI screen (factory: stage 2).

## 11. HID transport

The X11 exposes three collections on interface 2: commands (usage page
`0x000B`), events (`0x000A`) and consumer control (`0x000C`). Reads request
permission with `A0 <id> <len> 00 01 00 00 00`, poll `A0` until `A0 01`, then
read the target report.

| Report | Size | Fields |
|---|---|---|
| `0x04` | 56 | `3` LOD/angle snap, `4` motion sync/ripple, `5` enabled-stage mask, `6`/`7` double/triple flags, `8..15` low DPI byte, `16..23` high DPI byte, `24` active stage (1-based), `25..48` RGB, `49` indicator mode, `50..51` checksum |
| `0x05` | 15 | `3` lighting mode << 4, `4` deep-sleep high + LED speed, `5` deep-sleep low + brightness, `6..8` RGB, `9` light-off timer, `10` key response, `11..12` checksum |
| `0x06` | 9 | `3` period code (8/4/2/1), `4` complement |
| `0x08` | 59 | 18 slots × 3 bytes (action, modifiers, usage) from offset 3; checksum at `57..58` |
| `0x09` | 3 × 64 | macro pages 0/1/2; checksum at page 2 offset `10..11` |
| `0x0C` | 10 | `2` active profile, `3` complement, `4` profile count, `5` complement |

Checksums are 16-bit big-endian sums: `0x04` sums bytes 3–49, `0x05` sums
3–10, `0x08` sums 3–56. The DPI transfer table (`DPI_3311`) covers 50–12000 DPI;
above 12000 the value is halved and the double flag is set. Battery events are
input reports `03 <device> 40|41 <state> <percent>` on the events collection.

These layouts match the public X11 protocol documentation and were confirmed
against the connected device's own reports (§12).

## 12. Evidence

| Function | Address | Evidence id |
|---|---|---|
| Page creation | `0x49db00` | `ev_923ab04bf53ce24f5c21c206f221d6365bba36217d0b72fb56a1b40614093f82` |
| Page fill + clamps | `0x49e7a0` | `ev_9e7dfc0edbbc5458e25331deecf4fcb2a972ffd8c8a770574160f436325d9484` |
| Click / reset / macro defaults | `0x49f040` | `ev_b570e1e92b725464a56fc1bf553c25e52e2b09a7a3510c0cc26aa0c1bd08b267` |
| Restore-all defaults | `0x4a4670` | `ev_1fd426a2ebb79e732da9823a0c14dcdf6990ee7390494003096b042d86d1b43e` |
| Typed DPI → position | `0x4a07d0` | `ev_c332cdb1944066a76b7d69f2d86d1ef526e3caf5bcf7e96e5e8689225e4a8eaa` |
| Slider handler | `0x4a04f0` | `ev_97cea2f3ed9957187b0eaa8eabc57d85ee8302a99f941a5de77f9d4b9f39475c` |
| Combo / profile selection | `0x4a01c0` | `ev_ec6568bc7009a69ffa7e3621e687905c9ba989e9427706e42484202a7a49096f` |
| Option radios | `0x49ffe0` | `ev_dd30f12ed303edd19487f6b88f6c5cf2732b9abda55fee4435b09827bb786a5a` |
| Checkboxes / move wake | `0x4a02c0` | `ev_0a6d89e4ecb2ed472b6a7d4426b0bc13e26d0f6fdb1c0bc78a00a66f1e80dd02` |
| Button map + left-click rule | `0x4a09f0` | `ev_22f7fabcd425e0a78b81e34ded2f898f07fb308a144112a9ec59896567699a94` |
| Command labels | `0x483880` | `ev_6dc8906e747ec06a42e9c5905d02b384db9c7f3717e11f0931b766ca3a1f79a1` |
| Profile switch | `0x49e4c0` | `ev_1e25549e16b9f2594b0d8ac18143ce4cf2e7456cd06edafe23e7ffad0b106f45` |
| Profile add | `0x49edd0` | `ev_fcc2eafb78f06a5d32264c48cc7ae164088b69e8621e97fd4f3993fd573b4749` |
| Profile save | `0x4a4970` | `ev_63ae7e12f9b35b5f73f15cb1fbac7f6e88a71eaeadb796a8d92f6a5a720cb148` |
| Refresh / poll report | `0x4a38a0` | `ev_d7d4c8413343d7a0d27c08e84851ac472701cca444a8226cb33debe047b5f5b6` |
| Device events | `0x4a3000` | `ev_fa2046eded0e627f041366e8f8119e4b4f9cf51c32c54799ef3870b7f425c508` |
| Three 6 s timers | `0x43bc50` | `ev_65a88d1006e7d4d223126f4524bd075f72f03af169a38c303f61d04031d0a09b` |
