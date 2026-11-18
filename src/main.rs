mod pool;
mod process;

use pool::{MAX_LIMIT, Pool};
use std::{env, io};

const HELP: &str = "Usage: bulky -- COMMAND [ARG]...
       bulky --status
       bulky --set-limit N
       bulky --help | --version

Share a command limit across projects on this machine. The default is one slot.";

fn main() {
    let code = run().unwrap_or_else(|error| {
        eprintln!("bulky: {error}");
        1
    });
    std::process::exit(code);
}

fn run() -> io::Result<i32> {
    let mut args = env::args_os().skip(1);
    let flag = args.next();
    match flag.as_deref().and_then(|s| s.to_str()) {
        Some("--") => {
            if let Some(program) = args.next() {
                let admission = Pool::open()?.acquire(&program)?;
                return Ok(process::run(&program, args, &admission));
            }
        }
        Some("--help") if args.next().is_none() => {
            println!("{HELP}");
            return Ok(0);
        }
        Some("--version") if args.next().is_none() => {
            println!("bulky {}", env!("CARGO_PKG_VERSION"));
            return Ok(0);
        }
        Some("--status") if args.next().is_none() => {
            println!("{}", Pool::open()?.status()?);
            return Ok(0);
        }
        Some("--set-limit") => {
            let n = args.next().and_then(|s| s.to_str()?.parse::<u16>().ok());
            if let Some(n) = n.filter(|n| (1..=MAX_LIMIT).contains(n))
                && args.next().is_none()
            {
                Pool::open()?.set_limit(n)?;
                return Ok(0);
            }
            eprintln!("bulky: expected a limit between 1 and {MAX_LIMIT}");
            return Ok(2);
        }
        _ => {}
    }
    eprintln!("{HELP}");
    Ok(2)
}
