use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1B[0m";
const COLOR_TITLE: &str = "\x1B[1;36m";
const COLOR_ART: &str = "\x1B[38;5;213m";
const COLOR_OPTION_KEY: &str = "\x1B[1;33m";
const COLOR_OPTION_TEXT: &str = "\x1B[0;37m";
const COLOR_PROMPT: &str = "\x1B[38;5;159m";
const XP_SMALL: u32 = 5;
const XP_MEDIUM: u32 = 10;
const VIEW_WIDTH: usize = 60;
const SPLASH_ART: &str = r#"
██╗     ███████╗ ██████╗ ███████╗ ███╗   ██╗██████╗ 
██║     ██╔════╝██╔═══██╗██╔════╝ ████╗  ██║██╔══██╗
██║     █████╗  ██║   ██║█████╗   ██╔██╗ ██║██║  ██║
██║     ██╔══╝  ██║   ██║██╔══╝   ██║╚██╗██║██║  ██║
███████╗███████╗╚██████╔╝███████╗ ██║ ╚████║██████╔╝
╚══════╝╚══════╝ ╚═════╝ ╚══════╝ ╚═╝  ╚═══╝╚═════╝ 
██████╗ ███████╗██████╗     ██████╗ ███████╗██████╗ ███████╗████████╗ ██████╗ ██████╗ 
██╔══██╗██╔════╝██╔══██╗    ██╔══██╗██╔════╝██╔══██╗██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗
██║  ██║█████╗  ██████╔╝    ██║  ██║█████╗  ██████╔╝█████╗     ██║   ██║   ██║██████╔╝
██║  ██║██╔══╝  ██╔══██╗    ██║  ██║██╔══╝  ██╔══██╗██╔══╝     ██║   ██║   ██║██╔══██╗
██████╔╝███████╗██║  ██║    ██████╔╝███████╗██║  ██║███████╗   ██║   ╚██████╔╝██║  ██║
╚═════╝ ╚══════╝╚═╝  ╚═╝    ╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
"#;

const TOWN_SQUARE_ART: &str = r#"
╔════════════════════════════════════════════════════════════╗
║                             △ Neon Agora △                 ║
║  ╱╲ ╱╲ ╱╲    Plasma lanterns flicker above chrome cobbles  ║
║  ╲╱ ╲╱ ╲╱    Couriers barter packets, drones hum overhead  ║
╚════════════════════════════════════════════════════════════╝
"#;

const GRAVEYARD_ART: &str = r#"
╔════════════════════════════════════════════════════════════╗
║                     ☠ Crypt of Lost Processes ☠            ║
║  ┌─┐ ┌─┐ ┌─┐   Tomb relays glow with cold teal sigils      ║
║  └─┘ └─┘ └─┘   Binary incense drifts between obelisks      ║
╚════════════════════════════════════════════════════════════╝
"#;

const ROOM_ART: &str = r#"
╔════════════════════════════════════════════════════════════╗
║                      ◎ Safehouse 7 ◎                       ║
║  Neon vines crawl along carbon walls. A chest hums softly, ║
║  the closet door hides a dozen launchers, and the window   ║
║  overlooks a rain-soaked skyline.                          ║
╚════════════════════════════════════════════════════════════╝
"#;

fn main() {
    let config = Config::load();
    let mut game = Game::new(config);
    if let Err(err) = game.run() {
        eprintln!("An error occurred: {err}");
    }
}

struct Game {
    location: Location,
    config: Config,
    character: Character,
}

impl Game {
    fn new(config: Config) -> Self {
        let character = Character::new(&config);
        Self {
            location: Location::TownSquare,
            config,
            character,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        show_splash_screen()?;
        loop {
            let keep_playing = match self.location {
                Location::TownSquare => self.handle_town_square()?,
                Location::Graveyard => self.handle_graveyard()?,
                Location::Room => self.handle_room()?,
            };
            if !keep_playing {
                println!("Until next time, traveler.");
                break;
            }
        }
        Ok(())
    }

    fn handle_town_square(&mut self) -> io::Result<bool> {
        loop {
            clear_screen();
            show_location(Location::TownSquare, "== Town Square ==");
            print_option("G", "Go to the graveyard");
            print_option("R", "Return to your room");
            print_option("X", "Examine your dossier");
            print_option("Q", "Quit the adventure");
            match read_choice()? {
                Some('g') => {
                    self.location = Location::Graveyard;
                    return Ok(true);
                }
                Some('r') => {
                    self.location = Location::Room;
                    return Ok(true);
                }
                Some('x') => {
                    self.perform_character_sheet()?;
                }
                Some('q') => return Ok(false),
                None => return Ok(false),
                _ => println!("That action is not available."),
            }
        }
    }

    fn handle_graveyard(&mut self) -> io::Result<bool> {
        loop {
            clear_screen();
            show_location(Location::Graveyard, "== Graveyard ==");
            print_option("S", "Search for encrypted tombs");
            print_option("T", "Trek back to the town square");
            print_option("X", "Examine your dossier");
            print_option("Q", "Quit the adventure");
            match read_choice()? {
                Some('s') => {
                    self.perform_search_tombs()?;
                    self.reward_xp(XP_MEDIUM);
                    wait_for_continue()?;
                }
                Some('t') => {
                    self.location = Location::TownSquare;
                    return Ok(true);
                }
                Some('x') => {
                    self.perform_character_sheet()?;
                }
                Some('q') => return Ok(false),
                None => return Ok(false),
                _ => println!("Bones do not respond to that command."),
            }
        }
    }

    fn handle_room(&mut self) -> io::Result<bool> {
        loop {
            clear_screen();
            show_location(Location::Room, "== Your Safehouse ==");
            print_option("M", "Mail: check the courier satchel");
            print_option("C", "Computer: boot the virtual mainframe");
            print_option("H", "Hardware chest: deploy network tools");
            print_option("O", "Open the neon closet (games)");
            print_option("E", "Explore the world grid");
            print_option("L", "Lay down for a short rest");
            print_option("B", "Bedtime: start the screensaver");
            print_option("T", "Town square awaits");
            print_option("X", "Examine your dossier");
            print_option("Q", "Quit the adventure");
            match read_choice()? {
                Some('m') => {
                    self.perform_check_mail()?;
                    self.reward_xp(XP_SMALL);
                    wait_for_continue()?;
                }
                Some('c') => {
                    self.perform_use_computer()?;
                    self.reward_xp(XP_SMALL);
                    wait_for_continue()?;
                }
                Some('h') => {
                    self.perform_open_chest()?;
                    self.reward_xp(XP_MEDIUM);
                }
                Some('o') => {
                    self.perform_open_closet()?;
                    self.reward_xp(XP_SMALL);
                    wait_for_continue()?;
                }
                Some('e') => {
                    self.perform_explore_world()?;
                    self.reward_xp(XP_SMALL);
                    wait_for_continue()?;
                }
                Some('l') => {
                    self.perform_lay_down()?;
                    self.reward_xp(XP_SMALL);
                    wait_for_continue()?;
                }
                Some('b') => {
                    self.perform_screensaver()?;
                    self.reward_xp(XP_MEDIUM);
                }
                Some('t') => {
                    self.location = Location::TownSquare;
                    return Ok(true);
                }
                Some('x') => {
                    self.perform_character_sheet()?;
                }
                Some('q') => return Ok(false),
                None => return Ok(false),
                _ => println!("The room remains silent."),
            }
        }
    }

    fn perform_search_tombs(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.search_tombs_command() {
            match run_command_and_capture(command) {
                Ok(output) => {
                    if output.trim().is_empty() {
                        println!("The command completed without output.");
                    } else {
                        println!("{output}");
                    }
                }
                Err(err) => {
                    eprintln!("Failed to run configured search: {err}");
                    self.perform_builtin_tomb_search()?;
                }
            }
        } else {
            self.perform_builtin_tomb_search()?;
        }
        Ok(())
    }

    fn perform_builtin_tomb_search(&self) -> io::Result<()> {
        println!("You sift through dusty ledgers, looking for .tomb vaults...\n");
        let mut any_found = false;
        for dir in tomb_search_paths() {
            if let Ok(entries) = fs::read_dir(&dir) {
                let mut matches = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "tomb" {
                            matches.push(path);
                            continue;
                        }
                    }
                    if path.is_dir() {
                        matches.push(path);
                    }
                }
                if !matches.is_empty() {
                    any_found = true;
                    println!("{}:", dir.display());
                    for path in matches {
                        println!("  - {}", path.display());
                    }
                    println!();
                }
            }
        }
        if !any_found {
            println!(
                "No tombs were discovered. Configure a search command in lord_config.toml if you rely on the tomb CLI."
            );
        }
        Ok(())
    }

    fn perform_check_mail(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.check_mail_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("Unable to launch mail command: {err}");
            }
        } else {
            println!(
                "No mail command configured. Add one under [actions] -> check_mail in lord_config.toml."
            );
        }
        Ok(())
    }

    fn perform_lay_down(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.lay_down_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("Unable to start short rest command: {err}");
            }
        } else {
            println!("You stretch out on the cot. A moment of calm washes over you.");
        }
        Ok(())
    }

    fn perform_screensaver(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.activate_screensaver_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("Unable to start screensaver command: {err}");
            }
        } else {
            run_builtin_screensaver()?;
        }
        Ok(())
    }

    fn perform_use_computer(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.computer_terminal_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("The cyberdeck refuses to boot: {err}");
            }
        } else {
            println!(
                "No computer command configured. Assign actions.computer_terminal to launch VMware, virt-manager, etc."
            );
        }
        Ok(())
    }

    fn perform_open_chest(&mut self) -> io::Result<()> {
        let tools: Vec<&NamedCommand> = self
            .config
            .actions
            .chest_tools()
            .iter()
            .filter(|tool| tool.is_valid())
            .collect();
        if tools.is_empty() {
            println!(
                "The chest is empty. Populate [[actions.chest_tools]] entries in lord_config.toml."
            );
            return Ok(());
        }
        loop {
            clear_screen();
            println!();
            print_centered_colored("== Tech Chest ==", COLOR_TITLE);
            for (index, tool) in tools.iter().enumerate() {
                let slot = (index + 1).to_string();
                print_option(&slot, &tool.name);
            }
            print_option("Q", "Return to the room");
            match read_line_trimmed()? {
                None => break,
                Some(input) => {
                    if input.eq_ignore_ascii_case("q") {
                        break;
                    }
                    if input.is_empty() {
                        continue;
                    }
                    match input.parse::<usize>() {
                        Ok(choice) if choice >= 1 && choice <= tools.len() => {
                            if let Some(tool) = tools.get(choice - 1) {
                                if let Some(command) = tool.command() {
                                    if let Err(err) = spawn_command(command) {
                                        eprintln!("Failed to launch {}: {err}", tool.name);
                                    }
                                }
                            }
                        }
                        _ => println!("The chest stays locked unless you choose a valid slot."),
                    }
                }
            }
        }
        Ok(())
    }

    fn perform_open_closet(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.closet_launcher_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("Unable to open the neon closet: {err}");
            }
        } else {
            println!(
                "No closet launcher configured. Set actions.closet_launcher to your preferred game hub."
            );
        }
        Ok(())
    }

    fn perform_explore_world(&mut self) -> io::Result<()> {
        if let Some(command) = self.config.actions.explore_world_command() {
            if let Err(err) = spawn_command(command) {
                eprintln!("Exploration systems failed to boot: {err}");
            }
        } else {
            println!(
                "No exploration route configured. Point actions.explore_world at a browser like Firefox."
            );
        }
        Ok(())
    }

    fn perform_character_sheet(&mut self) -> io::Result<()> {
        loop {
            clear_screen();
            self.character.render_sheet();
            if self.character.pockets.is_empty() {
                print_centered_colored("Press ENTER to return.", COLOR_PROMPT);
                io::stdout().flush()?;
                let mut buffer = String::new();
                let _ = io::stdin().read_line(&mut buffer)?;
                break;
            } else {
                print_centered_colored(
                    "Select a pocket number or press ENTER to return.",
                    COLOR_PROMPT,
                );
                match read_line_trimmed()? {
                    None => break,
                    Some(input) if input.is_empty() || input.eq_ignore_ascii_case("q") => break,
                    Some(input) => match input.parse::<usize>() {
                        Ok(choice) if choice >= 1 && choice <= self.character.pockets.len() => {
                            if let Some(pocket) = self.character.pockets.get(choice - 1) {
                                pocket.use_item()?;
                                self.reward_xp(XP_SMALL);
                                wait_for_continue()?;
                            }
                        }
                        _ => print_centered_colored(
                            "That pocket is empty or inaccessible.",
                            COLOR_PROMPT,
                        ),
                    },
                }
            }
        }
        Ok(())
    }

    fn reward_xp(&mut self, amount: u32) {
        if amount == 0 {
            return;
        }
        if let Some(new_level) = self.character.gain_xp(amount) {
            println!(
                "{}You feel your skills sharpen. Level up! (Lv {}){}",
                COLOR_TITLE, new_level, RESET
            );
        } else {
            println!(
                "{}You gain {} XP. ({}/{}){}",
                COLOR_PROMPT,
                amount,
                self.character.xp,
                self.character.experience_to_next_level(),
                RESET
            );
        }
    }
}

#[derive(Clone, Copy)]
enum Location {
    TownSquare,
    Graveyard,
    Room,
}

#[derive(Default, Deserialize)]
struct Config {
    #[serde(default)]
    actions: ActionsConfig,
    #[serde(default)]
    character: CharacterConfig,
}

impl Config {
    fn load() -> Self {
        for path in config_paths() {
            if let Ok(contents) = fs::read_to_string(&path) {
                match toml::from_str::<Config>(&contents) {
                    Ok(config) => return config,
                    Err(err) => eprintln!("Failed to parse {}: {err}", path.display()),
                }
            }
        }
        Config::default()
    }
}

#[derive(Default, Deserialize)]
struct ActionsConfig {
    #[serde(default)]
    search_tombs: Option<Vec<String>>,
    #[serde(default)]
    check_mail: Option<Vec<String>>,
    #[serde(default)]
    activate_screensaver: Option<Vec<String>>,
    #[serde(default)]
    computer_terminal: Option<Vec<String>>,
    #[serde(default)]
    lay_down: Option<Vec<String>>,
    #[serde(default)]
    chest_tools: Vec<NamedCommand>,
    #[serde(default)]
    closet_launcher: Option<Vec<String>>,
    #[serde(default)]
    explore_world: Option<Vec<String>>,
    #[serde(default)]
    grin_wallet: Option<Vec<String>>,
}

impl ActionsConfig {
    fn search_tombs_command(&self) -> Option<&[String]> {
        self.search_tombs.as_deref().filter(|cmd| !cmd.is_empty())
    }

    fn check_mail_command(&self) -> Option<&[String]> {
        self.check_mail.as_deref().filter(|cmd| !cmd.is_empty())
    }

    fn activate_screensaver_command(&self) -> Option<&[String]> {
        self.activate_screensaver
            .as_deref()
            .filter(|cmd| !cmd.is_empty())
    }

    fn computer_terminal_command(&self) -> Option<&[String]> {
        self.computer_terminal
            .as_deref()
            .filter(|cmd| !cmd.is_empty())
    }

    fn lay_down_command(&self) -> Option<&[String]> {
        self.lay_down.as_deref().filter(|cmd| !cmd.is_empty())
    }

    fn chest_tools(&self) -> &[NamedCommand] {
        &self.chest_tools
    }

    fn closet_launcher_command(&self) -> Option<&[String]> {
        self.closet_launcher
            .as_deref()
            .filter(|cmd| !cmd.is_empty())
    }

    fn explore_world_command(&self) -> Option<&[String]> {
        self.explore_world.as_deref().filter(|cmd| !cmd.is_empty())
    }

    fn grin_wallet_command(&self) -> Option<&[String]> {
        self.grin_wallet.as_deref().filter(|cmd| !cmd.is_empty())
    }
}

#[derive(Default, Deserialize)]
struct CharacterConfig {
    #[serde(default)]
    clothing: Vec<String>,
}

#[derive(Clone, Default, Deserialize)]
struct NamedCommand {
    name: String,
    #[serde(default)]
    command: Vec<String>,
}

struct Character {
    name: String,
    level: u32,
    xp: u32,
    clothing: Vec<String>,
    pockets: Vec<PocketItem>,
}

impl Character {
    fn new(config: &Config) -> Self {
        let clothing = if config.character.clothing.is_empty() {
            vec![
                String::from("Aurora-weave jacket"),
                String::from("Carbon-thread boots"),
                String::from("Holographic lapel pin"),
            ]
        } else {
            config.character.clothing.clone()
        };

        let mut pockets = Vec::new();
        pockets.push(PocketItem::grin_wallet(
            config.actions.grin_wallet_command(),
        ));

        Self {
            name: determine_character_name(),
            level: 1,
            xp: 0,
            clothing,
            pockets,
        }
    }

    fn render_sheet(&self) {
        println!("\n{}== Operator Dossier =={}", COLOR_TITLE, RESET);
        println!("{}Name:{} {}", COLOR_OPTION_TEXT, RESET, self.name);
        println!(
            "{}Level:{} {}    {}XP:{} {}/{}",
            COLOR_OPTION_TEXT,
            RESET,
            self.level,
            COLOR_OPTION_TEXT,
            RESET,
            self.xp,
            self.experience_to_next_level()
        );
        println!("{}Clothing:{}", COLOR_OPTION_TEXT, RESET);
        for item in &self.clothing {
            println!("  - {}", item);
        }
        println!("{}Pockets:{}", COLOR_OPTION_TEXT, RESET);
        if self.pockets.is_empty() {
            println!("  (empty)");
        } else {
            for (index, pocket) in self.pockets.iter().enumerate() {
                println!("  [{}] {} — {}", index + 1, pocket.name, pocket.description);
            }
        }
    }

    fn gain_xp(&mut self, amount: u32) -> Option<u32> {
        self.xp += amount;
        let mut leveled = None;
        loop {
            let threshold = self.experience_to_next_level();
            if self.xp < threshold {
                break;
            }
            self.xp -= threshold;
            self.level += 1;
            leveled = Some(self.level);
        }
        leveled
    }

    fn experience_to_next_level(&self) -> u32 {
        25 + (self.level.saturating_sub(1) * 10)
    }
}

struct PocketItem {
    name: String,
    description: String,
    command: Option<Vec<String>>,
}

impl PocketItem {
    fn grin_wallet(command: Option<&[String]>) -> Self {
        let default_command = vec![String::from("grin-wallet")];
        Self {
            name: String::from("Grin Wallet"),
            description: String::from("Shielded grin-wallet client"),
            command: Some(command.unwrap_or(&default_command).to_vec()),
        }
    }

    fn use_item(&self) -> io::Result<()> {
        match &self.command {
            Some(cmd) => {
                if let Err(err) = spawn_command(cmd) {
                    eprintln!("{} refuses to activate: {err}", self.name);
                }
                Ok(())
            }
            None => {
                println!("This pocket item is ornamental only.");
                Ok(())
            }
        }
    }
}

impl NamedCommand {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.command.is_empty()
    }

    fn command(&self) -> Option<&[String]> {
        if self.command.is_empty() {
            None
        } else {
            Some(&self.command)
        }
    }
}

fn show_location(location: Location, title: &str) {
    println!();
    print_centered_colored(title, COLOR_TITLE);
    for line in location_art(location).lines() {
        print_centered_colored(line, COLOR_ART);
    }
}

fn location_art(location: Location) -> &'static str {
    match location {
        Location::TownSquare => TOWN_SQUARE_ART,
        Location::Graveyard => GRAVEYARD_ART,
        Location::Room => ROOM_ART,
    }
}

fn read_line_trimmed() -> io::Result<Option<String>> {
    print!("{}>{} ", COLOR_PROMPT, RESET);
    io::stdout().flush()?;
    let mut buffer = String::new();
    let bytes = io::stdin().read_line(&mut buffer)?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(buffer.trim().to_string()))
}

fn read_choice() -> io::Result<Option<char>> {
    match read_line_trimmed()? {
        None => Ok(None),
        Some(input) => {
            if let Some(ch) = input.chars().find(|c| !c.is_whitespace()) {
                Ok(Some(ch.to_ascii_lowercase()))
            } else {
                Ok(None)
            }
        }
    }
}

fn run_command_and_capture(command: &[String]) -> io::Result<String> {
    if command.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty command"));
    }
    let mut process = Command::new(&command[0]);
    if command.len() > 1 {
        process.args(&command[1..]);
    }
    let output = process.output()?;
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        combined.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(combined)
}

fn spawn_command(command: &[String]) -> io::Result<()> {
    if command.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty command"));
    }
    let mut process = Command::new(&command[0]);
    if command.len() > 1 {
        process.args(&command[1..]);
    }
    process
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map(|_| ())
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(PathBuf::from("lord_config.toml"));
    if let Some(mut home) = home_dir() {
        home.push(".config/lord/config.toml");
        paths.push(home);
    }
    paths
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn tomb_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("tombs"), PathBuf::from("vaults")];
    if let Some(mut home) = home_dir() {
        let mut alt = home.clone();
        home.push("tombs");
        alt.push(".local/share/tombs");
        paths.push(home);
        paths.push(alt);
    }
    paths
}

fn run_builtin_screensaver() -> io::Result<()> {
    println!("You lie down in bed. The lights dim. Press ENTER to wake up.");
    let running = Arc::new(AtomicBool::new(true));
    let worker_flag = running.clone();
    let handle = thread::spawn(move || {
        let mut frame = 0usize;
        while worker_flag.load(Ordering::Relaxed) {
            render_screensaver_frame(frame);
            frame = frame.wrapping_add(1);
            thread::sleep(Duration::from_millis(120));
        }
    });
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);
    running.store(false, Ordering::Relaxed);
    let _ = handle.join();
    print!("\x1B[2J\x1B[H");
    println!("You awaken feeling oddly refreshed.\n");
    Ok(())
}

fn render_screensaver_frame(frame: usize) {
    const WIDTH: usize = 50;
    const HEIGHT: usize = 14;
    print!("\x1B[2J\x1B[H");
    println!("~ Dreamless Sleep ~  (press ENTER to wake up)\n");
    for y in 0..HEIGHT {
        let mut line = String::with_capacity(WIDTH);
        for x in 0..WIDTH {
            let value = (x * 13 + y * 7 + frame * 5) % 97;
            let ch = match value {
                0..=10 => '.',
                11..=25 => '*',
                26..=40 => 'o',
                41..=55 => '+',
                56..=70 => ' ',
                71..=85 => '~',
                _ => '-',
            };
            line.push(ch);
        }
        println!("{line}");
    }
    io::stdout().flush().ok();
}

fn wait_for_continue() -> io::Result<()> {
    println!();
    print_centered_colored("Press ENTER to continue...", COLOR_PROMPT);
    io::stdout().flush()?;
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer)?;
    Ok(())
}

fn print_option(key: &str, description: &str) {
    println!(
        "{}[{}{}{}]{} {}{}{}",
        COLOR_OPTION_TEXT,
        COLOR_OPTION_KEY,
        key,
        COLOR_OPTION_TEXT,
        RESET,
        COLOR_OPTION_TEXT,
        description,
        RESET
    );
}

fn determine_character_name() -> String {
    fn format_candidate(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        trimmed
            .split(|c: char| c == '-' || c == '_' || c.is_whitespace())
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    let candidates = [
        env::var("HOSTNAME").ok(),
        env::var("HOST").ok(),
        env::var("USER").ok(),
        env::var("USERNAME").ok(),
    ];
    for candidate in candidates.into_iter().flatten() {
        let formatted = format_candidate(&candidate);
        if !formatted.is_empty() {
            return formatted;
        }
    }
    String::from("Traveler")
}

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().ok();
}

fn show_splash_screen() -> io::Result<()> {
    clear_screen();
    for line in SPLASH_ART.lines() {
        print_centered_colored(line, COLOR_ART);
    }
    print_centered_colored("Press ENTER to enter the Neon Agora...", COLOR_PROMPT);
    io::stdout().flush()?;
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer)?;
    Ok(())
}

fn padding_for_text(text: &str) -> String {
    let length = text.chars().count();
    let padding = VIEW_WIDTH.saturating_sub(length) / 2;
    " ".repeat(padding)
}

fn print_centered_colored(text: &str, color: &str) {
    if text.trim().is_empty() {
        println!();
        return;
    }
    let padding = padding_for_text(text);
    println!("{}{}{}{}", padding, color, text, RESET);
}
