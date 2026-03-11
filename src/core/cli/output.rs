const ASCII_ART: &str = r"                                                                                                             
                            GGGGGGGGG                                       GGGG                             
                            GGGGGGGGG                                       GGGG                             
                            GGGGGGGGG                                       GGGG                             
                            GGGG                                            GGGG                             
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG HGGG  GGG GGGGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG GGGG  GGGHGGGGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG GGGG  GGGHGGGGGGG                            
                            GGGG GGGG GGGG  GGG  GGG  GGGG  GGG GGGG  GGGH  GGGG                             
                            GGGGGGGGG GGGGGGGGG  GGG  GGGG  GGG GGGGGGGGGH  GGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGG  GGGG  GGG GGGGGGGGGH  GGGGG                            
                                                                                                             ";

pub fn help_text() -> String {
    format!(
        "{ASCII_ART}\n\nUsage:\n  {} [OPTIONS]\n\nOptions:\n  -h, --help          Show this help message\n  -v, --version       Show version\n      --toggle        Toggle the launcher (default)\n      --daemon        Run the daemon process\n      --preferences   Open the preferences window\n      --quit          Ask the running daemon to quit\n",
        env!("CARGO_PKG_NAME"),
    )
}

pub fn version_text() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

pub fn print_help() {
    println!("{}", help_text());
}

pub fn print_version() {
    println!("{}", version_text());
}
