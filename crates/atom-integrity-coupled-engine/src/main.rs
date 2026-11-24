//! Native numerical trace surface for the external frozen evaluator.
use std::io::{self, BufRead};

use atom_integrity_coupled_engine::{Chaos, Engine, Mask, MaterialOrchestrator, Order};

fn parse_args() -> Result<(Order, Mask, Chaos, u64), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: atom-integrity-material-engine ORDER MASK CHAOS SEED".into());
    }
    let order = match args[0].as_str() {
        "A" => Order::BindBeforeHabituate,
        "B" => Order::HabituateBeforeBind,
        _ => return Err("invalid order".into()),
    };
    let full = Mask::FULL;
    let mask = match args[1].as_str() {
        "full" => full,
        "no-quantum" => Mask {
            quantum: false,
            ..full
        },
        "classical-diffusion" => Mask {
            quantum: false,
            diffusion: true,
            ..full
        },
        "no-information" => Mask {
            information: false,
            ..full
        },
        "no-chemistry" => Mask {
            chemistry: false,
            ..full
        },
        "no-cognition" => Mask {
            cognition: false,
            ..full
        },
        "no-mechanics" => Mask {
            mechanics: false,
            ..full
        },
        "no-physics" => Mask {
            physics: false,
            ..full
        },
        "base-only" => Mask::BASE_ONLY,
        _ => return Err("invalid mask".into()),
    };
    let chaos = match args[2].as_str() {
        "plain" => Chaos::Plain,
        "logistic-transport" => Chaos::LogisticAtTransport,
        "henon-binding" => Chaos::HenonAtBinding,
        "both" => Chaos::Both,
        "logistic-binding" => Chaos::LogisticAtBinding,
        "henon-transport" => Chaos::HenonAtTransport,
        _ => return Err("invalid chaos setting".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    Ok((order, mask, chaos, seed))
}

fn run() -> Result<(), String> {
    let (order, mask, chaos, seed) = parse_args()?;
    let mut engine = Engine::new(order, chaos, mask, seed);
    let mut lines = Vec::new();
    for line in io::stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        let input: Vec<f64> = line
            .split_whitespace()
            .map(str::parse::<f64>)
            .collect::<Result<_, _>>()
            .map_err(|error| error.to_string())?;
        let input: [f64; 8] = input.try_into().map_err(|_| "expected eight bits")?;
        let observed =
            MaterialOrchestrator::observe(&mut engine, input).map_err(|error| error.to_string())?;
        let mut values = vec![
            observed.energy,
            observed.base_trace,
            observed.base_coherence,
            observed.information,
        ];
        values.extend(observed.occupation);
        values.extend(observed.binding);
        values.extend(observed.habituation);
        values.extend(observed.material);
        values.extend(observed.heat);
        let mut text = format!("{}", u8::from(observed.rupture));
        for value in values {
            text.push(',');
            text.push_str(&format!("{value:.17e}"));
        }
        lines.push(text);
    }
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
