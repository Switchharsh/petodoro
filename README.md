# Petodoro 🍅

[![Download .deb](https://img.shields.io/badge/Download-.deb%20for%20Linux%20Mint-ff5c8a?style=for-the-badge&logo=linuxmint&logoColor=white)](https://github.com/Switchharsh/petodoro/releases/latest/download/petodoro-amd64.deb)
[![Download tar.gz](https://img.shields.io/badge/Download-portable%20tar.gz-8a63d2?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/Switchharsh/petodoro/releases/latest/download/petodoro-x86_64-linux.tar.gz)
[![Latest release](https://img.shields.io/github/v/release/Switchharsh/petodoro?style=for-the-badge&label=latest&color=3fae6a)](https://github.com/Switchharsh/petodoro/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/Switchharsh/petodoro/ci.yml?style=for-the-badge&label=build)](https://github.com/Switchharsh/petodoro/actions/workflows/ci.yml)

A tiny desktop pet that floats on your screen and keeps you company with a
**pomodoro timer** and a **stopwatch**. Pick it up, drop it anywhere, pat it,
feed it, shake it (it gets dizzy), and watch it wander around. Made for Linux
Mint (and any other X11 desktop). Uses about 5 MB of RAM and almost no CPU.

![All 14 pets](assets/pets.png)

## Install: pick the easiest one for you

**1. Double-click a `.deb` (easiest).** Click the pink **Download .deb**
button above (or grab `petodoro-amd64.deb` from the
[Releases page](https://github.com/Switchharsh/petodoro/releases/latest)), double-click
it, press *Install Package*, then open **Petodoro** from the app menu. To
start it on every login, right-click the pet → *START ON LOGIN*.

**2. One command in a terminal.** This installs into your home folder, adds
an app-menu entry, starts on login, and launches the pet:

```sh
curl -fsSL https://raw.githubusercontent.com/Switchharsh/petodoro/HEAD/get.sh | bash
```

**3. Just the program.** Click **Download portable tar.gz** above,
unpack it, and run `./petodoro`. That's the whole app; nothing else is
needed. `./petodoro --install` copies it into your home folder and sets up
the app menu and autostart for you.

**4. From source.** `git clone` this repo and run `./install.sh`. It installs
Rust through [rustup](https://rustup.rs) if you don't have it (or use
`sudo apt install cargo`, Rust 1.75+), builds Petodoro (no `-dev` packages
needed), and runs `petodoro --install`.

Remove it with `petodoro --uninstall`, `./uninstall.sh` (add `--purge` to also
delete settings), or `sudo apt remove petodoro` if you used the `.deb`.

> Releases are built automatically by CI: every push to the default branch
> publishes the version in `Cargo.toml`. Bump that version to start a new release.

## The pets

Each pet has a **personality** that changes its face, how fast it moves, how
quickly it falls asleep, and everything it says.

| Pet        | Personality | Looks like                                   | Theme  |
|------------|-------------|----------------------------------------------|--------|
| **Sakura** | sweet       | pink twin-tails, sailor uniform              | light  |
| **Yuki**   | sweet       | silver hair, cat-ear hoodie                  | light  |
| **Hikari** | sweet       | blonde ponytail, maid outfit                 | light  |
| **Rin**    | tsundere    | orange twin buns, school blazer, fang        | light  |
| **Lilith** | tsundere    | gothic vampire girl with bat wings           | dark   |
| **Akane**  | angry       | red-haired delinquent with a bandage         | light  |
| **Nemu**   | sleepy      | pajamas, nightcap, hugs her pillow           | light  |
| **Mochi**  | sweet       | orange tabby cat                             | light  |
| **Kuro**   | tsundere    | pure-black neon cat, made for AMOLED screens | amoled |
| **Pochi**  | hyper       | shiba inu with endless zoomies               | light  |
| **Puru**   | hyper       | bouncy slime with a sprout                   | light  |
| **Boo**    | sweet       | little floating ghost                        | light  |
| **Hinote** | angry       | tiny fire spirit with a short fuse           | light  |
| **Bolt**   | sweet       | robot with a glowing LED face                | dark   |

- **Sweet**: smiles, hearts, cheers you on.
- **Tsundere**: pouts, crosses her arms, blushes and looks away when patted.
  *"B-BAKA! DON'T TOUCH ME!"* Gets softer as your friendship level rises.
- **Angry**: scowls, keeps her hands on her hips, shows anger marks 💢. *"WORK. OR ELSE."*
- **Sleepy**: half-closed eyes, nods off after 40 seconds, blows snot bubbles. *"FIVE MORE MINUTES..."*
- **Hyper**: bounces nonstop, sparkly eyes, walks around often, talks a lot.
  *"LET'S CRUSH IT!!!"* Great company if you have ADHD and like a bit of
  energy (and noise) next to your work.

The bubble and menu can be **light**, **dark** or **AMOLED** (pure black). By
default they follow the pet; change it from the menu or with `--theme`.

Every pet in every mood (idle, focus, break, asleep, cheering, patted, held,
walking, dizzy, eating):

![Every pet in every mood](assets/moods.png)

## Things to do with your pet

| Do this                       | And this happens                                         |
|-------------------------------|----------------------------------------------------------|
| **Drag** the pet              | pick it up and drop it anywhere                          |
| **Shake** it while dragging   | it gets dizzy (spiral eyes, stars)                       |
| **Click** the speech bubble   | start / pause the timer                                  |
| **Click** the pet             | pat it; click fast for a **combo counter** (fidget toy!) |
| **Middle-click**              | reset the timer                                          |
| **Scroll** on the pet         | ±1 minute on a phase that hasn't started yet             |
| **Right-click**               | menu                                                     |

- **Talks**: it greets you, reacts to pats, snacks and shakes, cheers when a
  session ends, and sometimes reminds you to drink water or stretch
  (turn this off with *CHATTER*).
- **Snacks**: *GIVE A SNACK* in the menu, and it munches a cookie. Overfeed it
  and it complains.
- **Friendship**: pats, snacks and finished pomodoros raise your friendship
  level with each pet, shown in the menu title. High levels unlock nicer
  lines (even from the tsunderes).
- **Walks around**: when nothing's running it wanders along the screen
  sometimes (never during a focus session). Turn it off with *WALK AROUND*.
- **Daily stats**: the menu shows today's pomodoros and focus minutes.
- **Break tips**: the end-of-focus notification suggests something healthy to
  do with your break.
- **Short sessions**: focus can be as short as 5 minutes, which helps for
  getting started on days when 25 feels like too much.

The menu also has pause/resume/reset/skip, mode (pomodoro / stopwatch / just
pet), pet, size (S–XL), theme, focus/break/long-break lengths, auto-start
next phase, sound, start on login, and quit. Left-click moves a setting
forward and right-click moves it back.

### Command line

```
petodoro --pet kuro            # choose a pet (see --list-pets)
petodoro --mode stopwatch      # pomodoro | stopwatch | pet
petodoro --theme amoled        # auto | light | dark | amoled
petodoro --scale 2             # bigger pet
petodoro --focus 50 --break 10 # custom pomodoro lengths
petodoro --reset-position      # lost your pet? bring it back to the corner
petodoro --install / --uninstall
petodoro --autostart on|off
petodoro --list-pets
petodoro --gallery pets.png    # render every pet
petodoro --preview sheet.png   # render every pet in every mood
```

Settings, friendship and stats live in `~/.config/petodoro/config` (plain
text). Only one pet runs at a time.

## Light on resources

About 5 MB of RAM and under 0.5% of one CPU core, even while a timer runs or
the pet walks. It's written in Rust and talks to X11 directly (no GTK, Qt or
GPU). It only redraws when something visible changes, reuses cached animation
frames, and sleeps in between. The program is about 1.3 MB.

## Troubleshooting

- **Black box around the pet**: your compositor is off. Petodoro then cuts
  the window to the pet's outline, so edges look slightly jaggier but it
  still works. Cinnamon always has compositing on.
- **No sound**: Petodoro uses `paplay` / `pw-play` with the freedesktop
  "complete" sound, falling back to `canberra-gtk-play` and then the system
  bell. You can turn sound off in the menu.
- **No notification**: run `sudo apt install libnotify-bin`.
- **Wayland**: it runs through XWayland, but some Wayland compositors won't
  keep it on top or let it move itself. Mint's default Cinnamon session is
  X11, where everything works.
- **`.deb` won't install on an older Mint**: release builds need glibc 2.35
  (Mint 21 or newer). On older systems, build from source.

## How it's built

```
src/
  main.rs       command line + event loop (sleeps on poll() until the next frame is due)
  app.rs        state: timers, moods, mouse, shaking, walking, speech, menu
  platform.rs   X11: ARGB window, always-on-top hints, SHAPE input regions, popup menu
  render.rs     speech bubble + themes, menu, pet-frame cache, preview sheets
  pets/         the pets, as vector shapes in a 100x100 box (girl rig, cat, dog, ...)
  lines.rs      what each personality says
  font.rs       5x7 pixel font (no font files to ship)
  timer.rs      pomodoro / stopwatch state machines (unit tested)
  config.rs     settings, friendship, daily stats
  system.rs     notifications, sounds, autostart, --install, single instance
scripts/build-deb.sh   packages the .deb (used by CI)
get.sh                 the one-line installer
```

Dependencies: [`x11rb`](https://crates.io/crates/x11rb) (a pure-Rust X11
client) and [`tiny-skia`](https://crates.io/crates/tiny-skia) (a CPU 2D
renderer). Run the tests with `cargo test`.

**Adding a pet**: add a file in `src/pets/` (or a new `Style` for the girl
rig in `girl.rs`), register it in `pets/mod.rs` with a personality, then
check every mood with `petodoro --preview out.png`.

## License

MIT. The pets are original artwork drawn in code for this project and are
covered by the same license.
