//! Numerical trace surface for a fixed-scale, two-pass external evaluator.
use atom_rhythm_four_engine::{Chaos, Engine, Event, Mask, Order, RhythmFourOrchestrator};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("usage: atom-rhythm-four-engine ORDER MASK CHAOS SEED TAU_NS".into());
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
        "logistic-phase" => Chaos::LogisticAtPhase,
        "henon-binding" => Chaos::HenonAtBinding,
        "both" => Chaos::Both,
        "logistic-binding" => Chaos::LogisticAtBinding,
        "henon-phase" => Chaos::HenonAtPhase,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    let tau = args[4].parse::<f64>().map_err(|error| error.to_string())?;
    let mut engine =
        Engine::new(order, chaos, mask, seed, tau).map_err(|error| error.to_string())?;
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
        let o = RhythmFourOrchestrator::observe(&mut engine, Event { time_ns, bytes_out })
            .map_err(|error| error.to_string())?;
        let mut values = vec![
            o.energy,
            o.purity,
            o.trace,
            o.entropy,
            o.binding,
            o.adaptation,
            o.phase,
            o.phase_norm,
        ];
        values.extend(o.occupation);
        values.extend(o.harmonics);
        values.extend(o.strain);
        values.extend(o.heat);
        let mut text = u8::from(o.rupture).to_string();
        for value in values {
            text.push(',');
            text.push_str(&format!("{value:.17e}"));
        }
        lines.push(text);
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
