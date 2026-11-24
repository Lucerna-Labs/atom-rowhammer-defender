//! Numeric trace surface for a caller-validated scalar pressure stream.
use atom_pressure_engine::{Chaos, Engine, Mask, Order, PressureOrchestrator};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "usage: atom-pressure-engine ORDER MASK CHAOS SEED; one fraction per stdin row".into(),
        );
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
        "logistic-rotation" => Chaos::LogisticRotation,
        "henon-binding" => Chaos::HenonBinding,
        "both" => Chaos::Both,
        "logistic-binding" => Chaos::LogisticBinding,
        "henon-rotation" => Chaos::HenonRotation,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|e| e.to_string())?;
    let mut engine = Engine::new(order, mask, chaos, seed).map_err(|e| e.to_string())?;
    let mut lines = Vec::new();
    for row in io::stdin().lock().lines() {
        let row = row.map_err(|e| e.to_string())?;
        let fraction = row.trim().parse::<f64>().map_err(|e| e.to_string())?;
        let s = Engine::observe(&mut engine, fraction).map_err(|e| e.to_string())?;
        lines.push(format!(
            "{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            u8::from(s.crossed),
            u8::from(s.selected_chaos),
            s.energy,
            s.population,
            s.coherence,
            s.trace,
            s.determinant,
            s.entropy,
            s.binding,
            s.habituation,
            s.position,
            s.heat,
        ));
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
