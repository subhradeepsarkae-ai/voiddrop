use colored::Colorize;

pub fn print_banner() {
    println!();
    println!("  {}", "⚡ VOIDDROP".purple().bold());
    println!("  {}", "Secure Terminal Transfer".cyan());
    println!();
}

pub fn print_mode(mode: &str) {
    match mode {
        "fast" => println!("  {}\n", "⚡ FAST MODE".green().bold()),
        "secure" => println!("  {}\n", "🔒 Secure Mode".cyan().bold()),
        "blast" => println!("  {}\n", "💥 BLAST MODE".red().bold()),
        _ => println!(),
    }
}
