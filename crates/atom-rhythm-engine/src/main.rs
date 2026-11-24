//! Numerical portable trace surface for external matched evaluation.
use atom_rhythm_engine::{Chaos, Engine, Event, Mask, Order, RhythmOrchestrator};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: atom-rhythm-engine ORDER MASK CHAOS SEED".into());
    }
    let order = match args[0].as_str() {
        "A" => Order::BindBeforeAdapt,
        "B" => Order::AdaptBeforeBind,
        _ => return Err("invalid order".into()),
    };
    let full = Mask::FULL;
    let mask = match args[1].as_str() {
        "full" => full,
        "no-quantum" => Mask {
            quantum: false,
            ..full
        },
        "classical-circular" => Mask {
            quantum: false,
            circular: true,
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
        "logistic-phase" => Chaos::LogisticAtPhase,
        "henon-binding" => Chaos::HenonAtBinding,
        "both" => Chaos::Both,
        "logistic-binding" => Chaos::LogisticAtBinding,
        "henon-phase" => Chaos::HenonAtPhase,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    let mut engine = Engine::new(order, chaos, mask, seed);
    let mut lines = Vec::new();
    for raw in io::stdin().lock().lines() {
        let raw = raw.map_err(|error| error.to_string())?;
        let parts: Vec<&str> = raw.split(',').collect();
        if parts.len() != 2 {
            return Err("expected time_ns,bytes_out".into());
        }
        let time_ns = parts[0]
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("invalid time: {e}"))?;
        let bytes_out = parts[1]
            .trim()
            .parse::<u32>()
            .map_err(|e| format!("invalid bytes: {e}"))?;
        let o = RhythmOrchestrator::observe(&mut engine, Event { time_ns, bytes_out })
            .map_err(|error| error.to_string())?;
        lines.push(format!(
            "{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            u8::from(o.rupture),
            o.energy,
            o.purity,
            o.entropy,
            o.binding,
            o.adaptation,
            o.strain,
            o.heat,
            o.tau.unwrap_or(0.0),
            o.phase,
            o.phase_norm
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
