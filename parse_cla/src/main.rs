//! A small CLI that parses a name, IP address, and port from the command line.
//!
//! Usage: `prog -n NAME -i IP -p PORT [-v]`

use std::env;
use std::fmt::Display;
use std::net::IpAddr;
use std::process;
use std::str::FromStr;

/// Validated command line configuration.
///
/// Built by [`parse_args`]. Every field has already been checked, so
/// consumers can use the values without re-validating them.
struct ConfigCtx {
    /// Label for this run, from `-n` / `--name`.
    name: String,
    /// IPv4 or IPv6 address, from `-i` / `--ip`.
    ip_addr: IpAddr,
    /// Port number, from `-p` / `--port`.
    port: u16,
    /// Enables extra diagnostic output on stderr, from `-v` / `--verbose`.
    verbose: bool,
}

/// Prints the usage text to stderr.
fn print_usage() {
    eprintln!("Usage: prog -n NAME -i IP -p PORT [-v]");
    eprintln!("  -n, --name NAME   name (required)");
    eprintln!("  -i, --ip IP       IP address (required)");
    eprintln!("  -p, --port PORT   port, 0-65535 (required)");
    eprintln!("  -v, --verbose     verbose output");
    eprintln!("  -h, --help        show this help");
}

/// Prints an error message and the usage text, then exits with status 1.
///
/// Returns `!` (never), so it can be used anywhere a value of any type
/// is expected, such as inside `unwrap_or_else`.
fn die(msg: &str) -> ! {
    eprintln!("{}", msg);
    print_usage();
    process::exit(1);
}

/// Takes the next token from `args` as the value for `flag`.
///
/// Exits via [`die`] if the iterator is empty, e.g. `prog --name` with
/// nothing after it. `flag` is only used for the error message.
fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> String {
    args.next()
        .unwrap_or_else(|| die(&format!("{} requires a value", flag)))
}

/// Parses `raw` into any type that implements [`FromStr`].
///
/// Exits via [`die`] with a message naming `flag` if parsing fails.
/// `T::Err` must implement [`Display`] so the parse error can be shown.
fn parse_value<T>(raw: &str, flag: &str) -> T
where
    T: FromStr,
    T::Err: Display,
{
    raw.parse()
        .unwrap_or_else(|e| die(&format!("invalid value '{}' for {}: {}", raw, flag, e)))
}

/// Parses the process command line into a [`ConfigCtx`].
///
/// Recognized flags: `-n/--name`, `-i/--ip`, `-p/--port` (all required),
/// plus `-v/--verbose` and `-h/--help`.
///
/// # Exits
///
/// Does not return an error. Instead the process exits when:
/// - `-h` / `--help` is given (status 0)
/// - an argument is unknown, a value is missing, a value fails to parse,
///   or a required flag is absent (status 1)
fn parse_args() -> ConfigCtx {
    // Optional while parsing so we can tell "not given" from "given".
    let mut name: Option<String> = None;
    let mut ip_addr: Option<IpAddr> = None;
    let mut port: Option<u16> = None;
    let mut verbose = false;

    // Skip argv[0], which is the program path.
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-n" | "--name" => name = Some(next_value(&mut args, &arg)),
            "-i" | "--ip" => {
                let v = next_value(&mut args, &arg);
                ip_addr = Some(parse_value(&v, &arg));
            }
            "-p" | "--port" => {
                let v = next_value(&mut args, &arg);
                port = Some(parse_value(&v, &arg));
            }
            "-v" | "--verbose" => verbose = true,
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            other => die(&format!("unknown argument: {}", other)),
        }
    }

    // Convert the Options into plain values, failing on any missing flag.
    ConfigCtx {
        name: name.unwrap_or_else(|| die("missing required argument: --name")),
        ip_addr: ip_addr.unwrap_or_else(|| die("missing required argument: --ip")),
        port: port.unwrap_or_else(|| die("missing required argument: --port")),
        verbose,
    }
}

fn main() {
    let ConfigCtx { name, ip_addr, port, verbose } = parse_args();

    if verbose {
        eprintln!("name={} ip={} port={}", name, ip_addr, port);
    }
    println!("{} -> {}:{}", name, ip_addr, port);
}