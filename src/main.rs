use std::{env, fs};

use crate::persistent_btree::PBTree;

mod persistent_btree;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    assert!(
        args.len() == 2,
        "ERRO: Entrada esperada: ./programa <nome_do_arquivo>",
    );

    let path = args.pop().unwrap();
    let contents = fs::read_to_string(path)
        .map_err(|err| err)
        .expect("ERRO: Não foi possível ler o arquivo");

    let mut tree = PBTree::<i64>::new();

    // Get commands line by line
    let commands = contents.split("\n");
    commands.enumerate().for_each(|(l, c)| {
        execute_command(c, &mut tree).expect(&format!("ERRO NA LINHA {}: \"{}\"", l + 1, c))
    });
}

fn execute_command(command: &str, tree: &mut PBTree<i64>) -> Result<(), String> {
    // Divides the command into tokens, then get them one by one
    let mut tokens = command.split(" ");
    let op = tokens.next().unwrap_or("");
    // Empty line or comment - no op
    if op.replace(" ", "") == "" || op.starts_with("#") {
        return Ok(());
    }
    let num1 = tokens.next().unwrap_or("");
    let num2 = tokens.next().unwrap_or("");

    // Matches the operation then executes it on the tree
    match op {
        "INC" => {
            tree.push(parse_int(num1)?);
        }
        "REM" => {
            tree.remove(&parse_int(num1)?);
        }
        "SUC" => {
            let successor = tree
                .successor(&parse_int(num1)?, parse_int(num2)? as usize)
                // Transforms possible int into string to translate None -> "infinito"
                .map(|n| n.to_string())
                .unwrap_or("INFINITO".to_string());
            println!("{successor}");
        }
        "IMP" => {
            println!("{}", tree.list(parse_int(num1)? as usize));
        }
        _ => return Err("Operação inválida.".to_string()),
    }

    Ok(())
}

fn parse_int(num: &str) -> Result<i64, String> {
    num.parse::<i64>()
        .map_err(|_| "Número inválido. É esperado um inteiro.".to_string())
}
