use axiom_engine::Engine;
use axiom_primitives::R;
use std::io::{self,BufRead,Write};
fn run()->R<()> {
    let args:Vec<String>=std::env::args().skip(1).collect();
    if args.len()!=5 {return Err("usage: axiom-engine ORDER MODE CHAOS SEED THERMAL; eight floats per stdin row".into());}
    let seed=args[3].parse::<u64>().map_err(|e|e.to_string())?;
    let init=Engine::new(&args[0],&args[1],&args[2],seed,&args[4])?;
    let mut out=io::BufWriter::new(io::stdout().lock());
    io::stdin().lock().lines().try_fold(init,|s,line|->R<Engine>{
        let line=line.map_err(|e|e.to_string())?;
        let v:Vec<f64>=line.split_whitespace().map(|x|x.parse::<f64>().map_err(|e|e.to_string())).collect::<R<_>>()?;
        let x:[f64;8]=v.try_into().map_err(|_|"expected eight floats")?;
        let s=s.step(x)?;
        writeln!(out,"{}",s.values().iter().map(|x|format!("{x:.17e}")).collect::<Vec<_>>().join(",")).map_err(|e|e.to_string())?;
        Ok(s)
    })?;
    out.flush().map_err(|e|e.to_string())?;Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1);}}
