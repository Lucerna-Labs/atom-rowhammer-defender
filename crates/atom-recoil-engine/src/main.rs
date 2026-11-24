//! Numerical trace surface for the external frozen recoil evaluator.
use atom_recoil_engine::{Chaos, Engine, Event, Mask, Order, RecoilOrchestrator};
use std::io::{self, BufRead};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err("usage: atom-recoil-engine ORDER MASK CHAOS SEED TAU_NS VARIANCE".into());
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
        "logistic-source" => Chaos::LogisticAtSource,
        "henon-chemistry" => Chaos::HenonAtChemistry,
        "both" => Chaos::Both,
        "logistic-chemistry" => Chaos::LogisticAtChemistry,
        "henon-source" => Chaos::HenonAtSource,
        _ => return Err("invalid chaos".into()),
    };
    let seed = args[3].parse::<u64>().map_err(|error| error.to_string())?;
    let tau = args[4].parse::<f64>().map_err(|error| error.to_string())?;
    let variance = args[5].parse::<f64>().map_err(|error| error.to_string())?;
    let mut engine =
        Engine::new(order, chaos, mask, seed, tau, variance).map_err(|error| error.to_string())?;
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
        let o = RecoilOrchestrator::observe(&mut engine, Event { time_ns, bytes_out })
            .map_err(|error| error.to_string())?;
        let mut values = vec![
            o.energy,
            o.trace,
            o.purity,
            o.entropy,
            o.quorum,
            o.recoil,
            o.adaptation,
            o.strain,
            o.heat,
            o.source_norm,
        ];
        values.extend(o.modes);
        values.extend(o.binding);
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
