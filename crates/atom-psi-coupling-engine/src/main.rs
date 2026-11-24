//! Numeric CLI for externally scoped pairs of unit fractions.
use atom_psi_coupling_engine::{Chaos, CouplingOrchestrator, Engine, Event, Mask, Order};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: atom-psi-coupling-engine ORDER MASK CHAOS SEED; first,second unit fractions per line".into());
    }
    let order = match args[0].as_str() {
        "A" => Order::TransportThenSource,
        "B" => Order::SourceThenTransport,
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
        "logistic-correlated" => Chaos::LogisticCorrelated,
        "henon-binding" => Chaos::HenonBinding,
        "both" => Chaos::Both,
        "logistic-binding" => Chaos::LogisticBinding,
        "henon-correlated" => Chaos::HenonCorrelated,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    let mut engine = Engine::new(order, mask, chaos, seed).map_err(|error| error.to_string())?;
    let mut lines = Vec::new();
    for raw in io::stdin().lock().lines() {
        let raw = raw.map_err(|error| error.to_string())?;
        let mut fields = raw.split(',');
        let first = fields.next().ok_or("expected first,second")?;
        let second = fields.next().ok_or("expected first,second")?;
        if fields.next().is_some() {
            return Err("expected first,second".into());
        }
        let event = Event {
            first: first
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string())?,
            second: second
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| e.to_string())?,
        };
        let state = Engine::observe(&mut engine, event).map_err(|error| error.to_string())?;
        let mut line = format!(
            "{},{}",
            u8::from(state.crossed),
            u8::from(state.selected_chaos)
        );
        for value in [
            state.energy,
            state.occupation[3],
            state.occupation[0],
            state.occupation[1],
            state.occupation[2],
            state.coherence_03,
            state.trace,
            state.minimum_minor,
            state.entropy,
            state.binding,
            state.habituation,
            state.position,
            state.heat,
        ] {
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
