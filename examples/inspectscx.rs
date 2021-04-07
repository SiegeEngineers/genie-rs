use genie::Scenario;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use std::fs::File;

fn main() {
    let log_level = std::env::var("LOG")
        .ok()
        .and_then(|value| match value.as_str() {
            "info" => Some(LevelFilter::Info),
            "debug" => Some(LevelFilter::Debug),
            "trace" => Some(LevelFilter::Trace),
            _ => None,
        })
        .unwrap_or(LevelFilter::Warn);
    let infile = std::env::args().nth(1).expect("usage: inspectscx <input>");

    TermLogger::init(
        log_level,
        Default::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let f = File::open(infile).expect("could not read file");
    let scen = Scenario::read_from(f).expect("invalid scenario file");

    println!("Scenario: {}", scen.filename());
    println!("Version:");
    println!("  Format: {}", scen.version().format);
    println!("  Header: {}", scen.version().header);
    match scen.version().dlc_options {
        None => println!("  DLC: absent"),
        Some(x) => println!("  DLC: {}", x),
    };
    if let Some(mod_name) = scen.mod_name() {
        println!("  UP Mod: {}", mod_name);
    }
    println!("  Data: {}", scen.version().data);
    println!("  Victory: {}", scen.version().victory);
    println!("  Map: {}", scen.version().map);
    match scen.triggers() {
        Some(_) => println!("  Triggers: {}", scen.version().triggers.unwrap()),
        None => println!("  Triggers: absent"),
    };
    println!();

    println!("Map:");
    println!("  Size: {}x{}", scen.map().width(), scen.map().height());
}
