use dicelet_core::{roll, RollOptions};
use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("Dicelet CLI — type a dice expression, or :q to quit");
    println!();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {}", e);
                break;
            }
        }

        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == ":q" || line == ":quit" {
            println!("bye!");
            break;
        }

        let result = roll(&line, RollOptions::default());
        match result {
            Ok(output) => {
                if output.consumed.is_empty() && output.tail.is_empty() {
                    println!("  (no parse)");
                } else {
                    println!("  consumed : {}", output.consumed);
                    if !output.tail.is_empty() {
                        println!("  tail     : {}", output.tail);
                    }
                    if output.is_set {
                        println!("  detail   : {}", output.detail);
                        println!("  summary  : {}", output.summary);
                    } else {
                        println!("  result   : {}", output.full);
                    }
                    if !output.values.is_empty() && output.values.len() <= 20 {
                        println!("  values   : {:?}", output.values);
                    }
                }
            }
            Err(e) => {
                println!("  error: {}", e);
            }
        }
        println!();
    }
}
