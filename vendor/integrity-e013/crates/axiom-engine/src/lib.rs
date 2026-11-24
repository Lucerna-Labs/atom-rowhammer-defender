use axiom_primitives::{allocate,check_matrix,moment,source,trace,transport,C,Matrix,Orbit,R,Site};
pub const MODES:[&str;17]=["full","no-distance","no-transport","population","coherent-pump","no-string","no-flux","no-modes","no-physics","no-chemistry","no-information","no-habituation","no-hebbian","geometry-only","quantum-only","disconnected","no-thermal"];
#[derive(Clone)]pub struct Engine{pub sites:[Site;4],pub occupation:Matrix,pub control_ket:[C;4],pub receipts:[f64;7],orbit:Orbit,order:String,mode:String,chaos:String,temperature:Option<f64>}
impl Engine{
 pub fn new(order:&str,mode:&str,chaos:&str,seed:u64,thermal:&str)->R<Self>{
  if !["A","B"].contains(&order)||!MODES.contains(&mode){return Err("composition".into());}let temperature=match thermal{"off"=>None,"cold"=>Some(0.1),"warm"=>Some(1.),"hot"=>Some(4.),_=>return Err("thermal".into())};Orbit::new(seed).factors(chaos)?;
  Ok(Self{sites:std::array::from_fn(|_|Site::default()),occupation:[[C::default();4];4],control_ket:[C::default();4],receipts:[0.;7],orbit:Orbit::new(seed),order:order.into(),mode:mode.into(),chaos:chaos.into(),temperature})
 }
 pub fn step(mut self,x:[f64;8])->R<Self>{
  let(early,late)=self.orbit.factors(&self.chaos)?;let disconnected=self.mode=="disconnected";let temp=if self.mode=="no-thermal"{None}else{self.temperature};let mut s=self.sites.clone();
  for i in 0..4{s[i]=s[i].clone().prepare(x[2*i],x[2*i+1],temp,self.mode=="no-distance",disconnected)?;}
  let theta=0.08*(1.+if disconnected{0.}else{0.05*s.iter().map(|s|s.weight).sum::<f64>()/4.});
  if self.mode=="geometry-only"{self.sites=s;self.receipts=[theta,0.,0.,0.,0.,0.,0.];return Ok(self);}
  let pump=std::array::from_fn(|i|C::new(s[i].phase.cos(),s[i].phase.sin()).scale(s[i].distance.sqrt()*(1.+if disconnected{0.}else{0.05*s[i].weight})));
  let coherent=self.mode=="coherent-pump";let population=self.mode=="population";let absent=self.mode=="no-transport";
  let(mut a,mut psi)=(self.occupation,self.control_ket);let(sb,sa,power,wb,wa);
  match self.order.as_str(){
   "A"=>{(a,psi,sb,sa,power)=source(a,psi,pump,coherent,population)?;(a,psi,wb,wa)=transport(a,psi,theta,absent,population,coherent)?;},
   "B"=>{(a,psi,wb,wa)=transport(a,psi,theta,absent,population,coherent)?;(a,psi,sb,sa,power)=source(a,psi,pump,coherent,population)?;},_=>return Err("order".into())}
  for i in 0..4{s[i]=s[i].clone().observe(a[i][i].re)?;}
  if self.mode!="quantum-only"{
   for i in 0..4{let site=s[i].clone().flux(["no-string","no-flux"].contains(&self.mode.as_str()))?;let force=site.intensity*early[i];let(site,out)=site.modes(force,["no-string","no-modes"].contains(&self.mode.as_str()),disconnected)?;s[i]=site.material(out,late[i],self.mode=="no-physics",self.mode=="no-chemistry",self.mode=="no-habituation",self.mode=="no-hebbian")?;}
   s=allocate(s,self.mode=="no-information")?;
  }
  check_matrix(&a)?;if !coherent&&trace(&a)>10.584+1e-10{return Err("statistical trace bound".into());}
  self.sites=s;self.occupation=a;self.control_ket=psi;self.receipts=[theta,wb,wa,sb,sa,power,moment(&a)];Ok(self)
 }
 pub fn values(&self)->Vec<f64>{let mut v:Vec<f64>=self.sites.iter().flat_map(Site::values).collect();for z in self.occupation.iter().flatten().chain(self.control_ket.iter()){v.extend([z.re,z.im]);}v.extend(self.receipts);v}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn clean_updates(){let mut e=Engine::new("A","full","off",1,"hot").unwrap();for _ in 0..50{e=e.step([0.,0.,1.,1.,0.,0.,1.,1.]).unwrap();}assert!(e.sites.iter().all(|s|s.output==0.));assert_eq!(trace(&e.occupation),0.);assert_eq!(e.values().len(),179);}
 #[test]fn base_has_no_material(){let mut e=Engine::new("A","quantum-only","off",1,"off").unwrap();for _ in 0..30{e=e.step([0.,1.,0.,0.,0.,0.,0.,0.]).unwrap();}assert!(e.sites[0].intensity>0.);assert!(e.sites.iter().all(|s|s.output==0.&&s.weight==0.));}
 #[test]fn unknown_refused(){assert!(Engine::new("Q","full","off",1,"off").is_err());}
}
