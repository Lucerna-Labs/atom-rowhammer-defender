//! Numeric trace CLI for four-salted-mode fingerprint phase material.
use atom_row_quorum_engine::{Chaos, Engine, Event, Mask, Order, QuorumOrchestrator};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "usage: atom-row-quorum-engine ORDER MASK CHAOS SEED; time_ns,fingerprint rows".into(),
        );
    }
    let order = match args[0].as_str() {
        "A" => Order::InformationFirst,
        "B" => Order::HabituationFirst,
        _ => return Err("invalid order".into()),
    };
    let full = Mask::FULL;
    let mask = match args[1].as_str() {
        "full" => full,
        "no-quantum" => Mask {
            quantum: false,
            ..full
        },
        "diffusion" => Mask {
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
        _ => return Err("invalid mask".into()),
    };
    let chaos = match args[2].as_str() {
        "plain" => Chaos::Plain,
        "logistic-phase" => Chaos::LogisticPhase,
        "henon-chemistry" => Chaos::HenonChemistry,
        "both" => Chaos::Both,
        "logistic-chemistry" => Chaos::LogisticChemistry,
        "henon-phase" => Chaos::HenonPhase,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    let mut engine = Engine::new(order, mask, chaos, seed).map_err(|error| error.to_string())?;
    let mut lines = Vec::new();
    for raw in io::stdin().lock().lines() {
        let raw = raw.map_err(|error| error.to_string())?;
        let mut parts = raw.split(',');
        let time = parts.next().ok_or("expected time_ns,fingerprint")?;
        let fingerprint = parts.next().ok_or("expected time_ns,fingerprint")?;
        if parts.next().is_some() {
            return Err("expected time_ns,fingerprint".into());
        }
        let event = Event {
            time_ns: time
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?,
            fingerprint: fingerprint
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?,
        };
        let state = Engine::observe(&mut engine, event).map_err(|error| error.to_string())?;
        let mut line = format!(
            "{},{}",
            u8::from(state.crossed),
            u8::from(state.selected_chaos)
        );
        for value in [state.energy, state.collective] {
            line.push(',');
            line.push_str(&format!("{value:.17e}"));
        }
        for value in state
            .powers
            .into_iter()
            .chain(state.reals)
            .chain(state.imags)
            .chain([
                state.entropy,
                state.binding,
                state.habituation,
                state.position,
                state.heat,
                state.alpha,
            ])
        {
            line.push(',');
            line.push_str(&format!("{value:.17e}"));
        }
        lines.push(line);
    }
    if lines.is_empty() {
        return Err("empty trace".into());
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
