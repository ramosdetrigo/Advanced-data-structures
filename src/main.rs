pub mod veb;

use std::{env, fs, process};
use veb::Veb;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("ERRO: Entrada esperada: ./programa <nome_do_arquivo>");
        process::exit(1);
    }

    let path = &args[1];
    let contents = fs::read_to_string(path).expect("ERRO: Não foi possível ler o arquivo");

    let mut veb = Veb::new();

    for (lineno, line) in contents.lines().enumerate() {
        if let Err(e) = execute_command(line, &mut veb) {
            eprintln!("ERRO NA LINHA {}: \"{}\"", lineno + 1, e);
            process::exit(1);
        }
    }
}

fn execute_command(command: &str, veb: &mut Veb) -> Result<(), String> {
    let mut tokens = command.split_whitespace();
    let op = tokens.next().unwrap_or("");
    // ignora linhas vazias e comentários
    if op.is_empty() || op.starts_with('#') {
        return Ok(());
    }

    match op {
        "INC" => {
            let num = tokens.next().ok_or("Esperado um número após INC")?;
            let x = parse_int(num)?;
            veb.include(x);
            // println!("INC {x}")
        }
        "REM" => {
            let num = tokens.next().ok_or("Esperado um número após REM")?;
            let x = parse_int(num)?;
            veb.remove(x);
            // println!("REM {x}")
        }
        "SUC" => {
            let num = tokens.next().ok_or("Esperado um número após SUC")?;
            let x = parse_int(num)?;
            println!("SUC {}", x);
            match veb.successor(x) {
                Some(s) => println!("{}", s),
                None => println!("+INF"),
            }
        }
        "PRE" => {
            let num = tokens.next().ok_or("Esperado um número após PRE")?;
            let x = parse_int(num)?;
            println!("PRE {}", x);
            match veb.predecessor(x) {
                Some(p) => println!("{}", p),
                None => println!("-INF"),
            }
        }
        "IMP" => {
            println!("IMP");
            println!("{}", veb.imp())
        }
        _ => return Err(format!("Operação inválida: {}", op)),
    }
    Ok(())
}

fn parse_int(s: &str) -> Result<u32, String> {
    s.parse::<u32>()
        .map_err(|_| format!("Número inválido: {}", s))
}
