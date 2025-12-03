# Legend of the Red Desktop (LoRD-inspired launcher)

This project is a lightweight, text-driven hub inspired by **Legend of the Red Dragon**
and retold as a neon cyberpunk desktop. From the Town Square you can stride into the
crypt district, search for Tomb vaults, or retreat to your safehouse filled with
launchers for network tools, emulators, browsers, a Rust screensaver, and a customizable
character sheet (complete with clothing, pockets, and leveling).

## Prerequisites

- Rust 1.74+ (edition 2024)
- Optional external applications (e.g., `tomb`, email client, screensaver) if you
  want those actions to trigger real software
- A terminal that supports ANSI colors (most modern emulators do) for the neon UI

## Running the game hub

```bash
cargo run
```

You will be dropped into the Town Square. Press the highlighted letter for each
action:

| Location      | Keys & Actions                                                                 |
| ------------- | -------------------------------------------------------------------------------- |
| Town Square   | `G` go to graveyard, `R` go to room, `X` examine dossier, `Q` quit                |
| Graveyard     | `S` search for tombs, `T` back to town, `X` examine dossier, `Q` quit             |
| Safehouse     | `M` mail, `C` computer (VM launcher), `H` hardware chest (network tools), `O` open closet (game launcher), `E` explore (browser), `L` lay down, `B` screensaver, `T` back, `X` examine dossier, `Q` quit |

`X` is global: it opens the Operator Dossier showing your character name (derived from the
terminal hostname), current level/XP, clothing list, and cybernetic pockets. Using
pocket items (like the embedded Grin wallet) can launch their associated tools.

When "laying down in bed" the built-in Rust screensaver runs. Press `Enter` to wake up.

### Actions reference

- **Computer** &mdash; ties to `actions.computer_terminal` and is perfect for VMware,
  virt-manager, or any VM workflow.
- **Chest** &mdash; presents numbered slots defined under `[[actions.chest_tools]]` in the
  config file. Each slot gets a display name (e.g., `WireGuard`, `Suricata`, `nmap`) and a
  command array to spawn.
- **Closet** &mdash; single launcher tied to `actions.closet_launcher` for game hubs such as
  Steam, Heroic, or a favorite emulator frontend.
- **Explore** &mdash; points to `actions.explore_world` and is perfect for launching Firefox or
  another browser to "leave" the safehouse.
- **Lay Down** &mdash; can run any lock/sleep command via `actions.lay_down` (e.g., `swaylock`).
- **Dossier** &mdash; `X` shows your stats, XP progress, clothing, and pockets (including the
  Grin wallet launcher). Pocket actions can award XP, and level-ups are announced inline.

## Wiring external applications

Actions can be configured with a TOML file. The loader checks the paths below (first
match wins):

1. `./lord_config.toml`
2. `$HOME/.config/lord/config.toml`

Copy the sample file to one of those paths and edit as needed:

```bash
cp lord_config.example.toml lord_config.toml
```

Example contents:

```toml
[actions]
search_tombs = ["tomb", "list"]
check_mail = ["thunderbird"]
activate_screensaver = ["xscreensaver-command", "-activate"]
computer_terminal = ["vmware"]
closet_launcher = ["steam"]
explore_world = ["firefox"]
grin_wallet = ["grin-wallet", "listen"]
lay_down = ["swaylock"]

[[actions.chest_tools]]
name = "WireGuard"
command = ["wg-quick", "up", "office-net"]

[[actions.chest_tools]]
name = "Suricata"
command = ["suricata", "-D"]

[[actions.chest_tools]]
name = "Nmap Sweep"
command = ["nmap", "-sV", "10.0.0.0/24"]

[character]
clothing = [
  "Aurora-weave jacket",
  "Carbon-thread boots",
  "Optic visor"
]
```

- `search_tombs`: runs the command and streams its output back into the hub
- `check_mail`: spawns the command and leaves it attached to your terminal
- `activate_screensaver`: spawns an external screensaver, if preferred over the built-in Rust animation
- `computer_terminal`: launches your cyberdeck/VM environment (VMware, virt-manager, etc.)
- `closet_launcher`: launches any desktop game hub or emulator frontend
- `explore_world`: recommended for browsers; treat it as "going outside"
- `lay_down`: optional command for short rests (e.g., `swaylock`)
- `grin_wallet`: overrides the default `grin-wallet` command tucked into your pocket
- `[[actions.chest_tools]]`: repeatable blocks for naming and launching as many cyber tools as you like
- `[character].clothing`: customize the wardrobe that appears on the dossier

If an action has no configured command, the program falls back to built-in behavior
(listing `.tomb` files under common directories, printing a reminder, or running the
Rust screensaver).

## Adding more interactions

Extend `src/main.rs` with new locations or commands. The structure keeps the story
state machine simple, so new options can call `spawn_command` or `run_command_and_capture`
as needed. If they should award progress, route the reward through `Game::reward_xp`
so the leveling system and announcements remain consistent.