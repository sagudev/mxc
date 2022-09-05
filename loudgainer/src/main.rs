use log::debug;

fn main() {
    let opts = loudgainer::options::LoudgainOptions::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    debug!("{:#?}", opts);

    loudgainer::loudgain(opts, false);
}
