use std::collections::HashSet;

use itybity::StrToBits;

#[derive(Debug, Clone)]
pub enum Node {
    Input {
        name: String,
    },
    Output {
        name: String,
    },
    LinComb {
        inputs: Vec<String>,
        output: String,
        coefs: Vec<i8>,
        const_coef: i8,
    },
    Bootstrap {
        input: String,
        outputs: Vec<String>,
        tables: Vec<Vec<bool>>,
    },
}

type Circuit = Vec<Node>;

pub type ParseError = String;

fn parse_bootstrap(
    lines: Vec<&str>,
) -> Result<
    (
        String,         // input
        Vec<String>,    // outputs
        Vec<Vec<bool>>, // test vectors
    ),
    ParseError,
> {
    if lines.len() < 2 {
        return Err(format!(
            "Error parsing bootstrap, expected at least 1 line: {:?}",
            lines
        ));
    }

    let line = &lines[0];

    let mut it = line["bootstrap".len()..].split_ascii_whitespace();
    let input = it
        .next()
        .ok_or(format!("Expected one input exactly: {}", line))?
        .to_string();

    let outputs: Vec<_> = it.map(|e| e.to_string()).collect();
    if outputs.is_empty() {
        return Err(format!("Expected at least one output: {}", line));
    }
    if outputs.len() != lines.len() - 1 {
        return Err(format!(
            "Expected one truth table line per output: {}",
            line
        ));
    }

    let tables: Vec<_> = lines[1..]
        .iter()
        .map(|e| {
            e.chars()
                .filter(|e| (e == &'0') | (e == &'1'))
                .collect::<String>()
                .iter_bits()
                .collect()
        })
        .collect();

    Ok((input, outputs, tables))
}

fn parse_lincomb(
    lines: Vec<&str>,
) -> Result<
    (
        Vec<String>, // inputs
        String,      // output
        Vec<i8>,     // coefs
        i8,          // const_coef
    ),
    ParseError,
> {
    if lines.len() != 2 {
        return Err(format!(
            "Error parsing lincomb, expected 2 lines exactly: {:?}",
            lines
        ));
    }

    let line = &lines[0];

    let mut inputs: Vec<_> = line["lincomb".len()..]
        .split_ascii_whitespace()
        .map(|e| e.to_string())
        .collect();

    let output = inputs
        .pop()
        .ok_or(format!("Lincomb has no output: {:?}", lines))?;

    let mut coefs: Vec<_> = vec![];
    for e in lines[1].split_ascii_whitespace() {
        let v = e
            .parse::<i8>()
            .map_err(|err| format!("Error parsing i8: {} - {:?}", err, lines))?;
        coefs.push(v);
    }

    let const_coef = if inputs.len() + 1 == coefs.len() {
        coefs
            .pop()
            .ok_or(format!("Lincomb has no coefficients: {:?}", lines))?
    } else {
        0
    };

    if inputs.len() != coefs.len() {
        return Err(format!(
            "Lincomb inputs and coefficients count mismatch: {:?}",
            lines
        ));
    }

    Ok((inputs, output, coefs, const_coef))
}

fn check_circuit(circuit: Circuit) -> Result<Circuit, ParseError> {
    // traverse circuit and check if name exist or not defined twice
    //  and other consistency checks
    let mut visited_names: HashSet<String> = HashSet::new();

    fn add_new_name(visited_names: &mut HashSet<String>, name: &String) -> Result<(), ParseError> {
        if visited_names.contains(name) {
            return Err(format!("Name {} already defined", name));
        }
        visited_names.insert(name.clone());
        Ok(())
    }

    fn check_name_exists(visited_names: &HashSet<String>, name: &String) -> Result<(), ParseError> {
        if !visited_names.contains(name) {
            return Err(format!("Name {} does not exist", name));
        }
        Ok(())
    }

    for node in &circuit {
        match node {
            Node::Input { name } => {
                add_new_name(&mut visited_names, name)?;
            }
            Node::LinComb { inputs, output, .. } => {
                for input in inputs {
                    check_name_exists(&visited_names, input)?;
                }
                add_new_name(&mut visited_names, output)?;
            }
            Node::Bootstrap { input, outputs, .. } => {
                check_name_exists(&visited_names, input)?;
                for output in outputs {
                    add_new_name(&mut visited_names, output)?;
                }
            }
            _ => {}
        }
    }

    // check output once all nodes are visited
    for node in &circuit {
        match node {
            Node::Output { name } => {
                check_name_exists(&visited_names, name)?;
            }
            _ => {}
        }
    }

    Ok(circuit)
}

pub fn parse_lbf(input: &str) -> Result<Circuit, ParseError> {
    let mut circuit = Circuit::new();

    /// Remove end of line comment and trim spaces
    fn sanitize_line(s: &str) -> &str {
        let s1 = match s.find("#") {
            Some(idx) => &s[0..idx],
            None => s,
        };
        s1.trim()
    }

    // Sanitize input lines and merge lines ending with '\'
    let lines_no_comments = input
        .lines()
        .map(sanitize_line)
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\\\n", " ");

    for obj in lines_no_comments.split(".") {
        let obj = obj.trim();
        if obj.is_empty() {
            continue;
        }

        if obj.starts_with("inputs") {
            circuit.extend(obj["inputs".len()..].split_ascii_whitespace().map(|name| {
                Node::Input {
                    name: name.to_string(),
                }
            }));
        } else if obj.starts_with("outputs") {
            circuit.extend(obj["outputs".len()..].split_ascii_whitespace().map(|name| {
                Node::Output {
                    name: name.to_string(),
                }
            }));
        } else if obj.starts_with("lincomb") {
            let lines: Vec<&str> = obj.lines().collect();
            let (inputs, output, coefs, const_coef) = parse_lincomb(lines)?;
            circuit.push(Node::LinComb {
                inputs,
                output,
                coefs,
                const_coef,
            });
        } else if obj.starts_with("bootstrap") {
            let lines: Vec<&str> = obj.lines().collect();
            let (input, outputs, tables) = parse_bootstrap(lines)?;
            circuit.push(Node::Bootstrap {
                input,
                outputs,
                tables,
            });
        } else if obj.starts_with("end") {
            break;
        }
    }

    check_circuit(circuit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_file() {
        let s = r#".inputs a b c d
            .outputs e f g \
                h CONST1
            .lincomb CONST1     # CONSTANT
            1
            .lincomb a b \
             n1     # n1 = 2.a + b, n1 in {0, 1, 2, 3}, sq. norm2 = 2^2 + 1^2 = 5
            2 1
            .lincomb a b n2     # n2 = a - b + 1, n2 in {0, 1, 2}, sq. norm2 = 1^2 + 1^2 = 2
            1 -1 \
            1
            .bootstrap n1 e          # AND(a, b)
            000   \
            1
            .bootstrap n2 f          # XOR(a, b)
            101
            .bootstrap n2 g h        # 2-output bootstraping
            001                 # AND(a, NOT(b))
            0001                # XNOR(a, b)
            .end
            "#;

        let circuit = parse_lbf(s).unwrap();
        println!("{:?}", circuit);
    }
}
