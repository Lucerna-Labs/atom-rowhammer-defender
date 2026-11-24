//! Projector geometry, statistical occupation transport and intrinsic material memory.
pub type R<T> = Result<T,String>;
#[derive(Clone,Copy,Debug,Default)]
pub struct C {pub re:f64,pub im:f64}
impl C {
    pub fn new(re:f64,im:f64)->Self {Self{re,im}}
    pub fn plus(self,b:Self)->Self {Self::new(self.re+b.re,self.im+b.im)}
    pub fn times(self,b:Self)->Self {Self::new(self.re*b.re-self.im*b.im,self.re*b.im+self.im*b.re)}
    pub fn scale(self,k:f64)->Self {Self::new(self.re*k,self.im*k)}
    pub fn norm2(self)->f64 {self.re*self.re+self.im*self.im}
    pub fn conj(self)->Self {Self::new(self.re,-self.im)}
}
pub type Colour=[C;3];


pub fn bath(t:Option<f64>)->R<(f64,f64)>{match t{None=>Ok((0.,1.)),Some(t)=>{if !t.is_finite()||t<=0.{return Err("temperature".into());}let e=(-1./t).exp();let z=1.+2.*e;Ok((0.1*e/z,0.9+0.1*(1.-e)/z))}}}
pub fn projector_distance(a:Colour,b:Colour,bath_b:f64,bath_c:f64)->R<(f64,f64)>{
 for q in [a,b]{let n:f64=q.iter().map(|z|z.norm2()).sum();if !n.is_finite()||(n-1.).abs()>1e-10{return Err("ray norm".into());}}
 if !bath_b.is_finite()||!bath_c.is_finite()||bath_b<0.||bath_c<0.||(3.*bath_b+bath_c-1.).abs()>1e-10{return Err("bath state".into());}
 let mut distance=0.;for i in 0..3{for j in 0..3{let delta=a[i].times(a[j].conj()).plus(b[i].times(b[j].conj()).scale(-1.));distance+=delta.norm2()/2.;}}
 let purity=bath_c*bath_c+2.*bath_b*bath_c+3.*bath_b*bath_b;let mixed=distance*bath_c*bath_c/purity;
 if !mixed.is_finite()||distance>1.+1e-10||mixed>1.+1e-10{return Err("distance bound".into());}Ok((distance,mixed))
}
fn ray(x:f64,phase:f64)->Colour{let theta=std::f64::consts::FRAC_PI_2*x;[C::new(theta.cos(),0.),C::new(theta.sin()*phase.cos(),theta.sin()*phase.sin()),C::default()]}
#[derive(Clone,Debug)]pub struct Site{
 pub reference:Colour,pub observed:Colour,pub pure_distance:f64,pub distance:f64,pub bath_b:f64,pub bath_c:f64,pub purity:f64,pub heat:f64,pub reservoir:f64,
 pub intensity:f64,pub r:f64,pub v:f64,pub mq:[f64;2],pub mp:[f64;2],pub energy:f64,pub field:f64,pub bound:f64,pub habituation:f64,pub weight:f64,pub output:f64,pub phase:f64,
}
impl Default for Site{fn default()->Self{Self{reference:[C::new(1.,0.),C::default(),C::default()],observed:[C::new(1.,0.),C::default(),C::default()],pure_distance:0.,distance:0.,bath_b:0.,bath_c:1.,purity:1.,heat:0.,reservoir:0.,intensity:0.,r:0.,v:0.,mq:[0.;2],mp:[0.;2],energy:0.,field:0.,bound:0.,habituation:0.,weight:0.,output:0.,phase:0.}}}
impl Site{
 pub fn values(&self)->Vec<f64>{let mut v=Vec::with_capacity(33);for q in [self.reference,self.observed]{for z in q{v.extend([z.re,z.im]);}}v.extend([self.pure_distance,self.distance,self.bath_b,self.bath_c,self.purity,self.heat,self.reservoir,self.intensity,self.r,self.v]);v.extend(self.mq);v.extend(self.mp);v.extend([self.energy,self.field,self.bound,self.habituation,self.weight,self.output,self.phase]);v}
 pub fn check(&self)->R<()>{
  if self.values().iter().any(|v|!v.is_finite()){return Err("nonfinite".into());}
  projector_distance(self.reference,self.observed,self.bath_b,self.bath_c)?;
  if [self.pure_distance,self.distance,self.intensity,self.field,self.bound,self.habituation,self.weight,self.output].iter().any(|v|*v< -1e-10||*v>1.+1e-10){return Err("capacity".into());}
  if self.purity<1./3.-1e-10||self.purity>1.+1e-10||(self.heat-4.*self.bath_b).abs()>1e-12{return Err("thermal receipt".into());}
  if self.energy<0.||[self.r,self.v,self.mq[0],self.mq[1],self.mp[0],self.mp[1]].iter().any(|v|v.abs()>=20.){return Err("mechanical bound".into());}Ok(())
 }
 pub fn prepare(mut self,x:f64,y:f64,t:Option<f64>,no_distance:bool,disconnected:bool)->R<Self>{
  if [x,y].iter().any(|v|!v.is_finite()||*v<0.||*v>1.){return Err("input range".into());}
  self.phase=if disconnected{0.}else{0.1*self.r+0.1*self.mq[0]};self.reference=ray(x,self.phase);self.observed=ray(y,self.phase);
  let(b,c)=bath(t)?;self.bath_b=b;self.bath_c=c;self.purity=c*c+2.*b*c+3.*b*b;self.heat=4.*b;self.reservoir-=self.heat;
  let(pure,distance)=projector_distance(self.reference,self.observed,b,c)?;self.pure_distance=pure;self.distance=if no_distance{0.5}else{distance};self.check()?;Ok(self)
 }
 pub fn observe(mut self,occupation:f64)->R<Self>{if !occupation.is_finite()||occupation<0.{return Err("occupation".into());}self.intensity=occupation/(1.+occupation);self.check()?;Ok(self)}
 pub fn flux(mut self,absent:bool)->R<Self>{if absent{self.r=0.;self.v=0.;}else{self.v+=0.2*(0.2*self.intensity-(0.05+self.distance)*self.r/(0.04+self.r*self.r).sqrt()-0.1*self.v);self.r+=0.2*self.v;}self.check()?;Ok(self)}
 pub fn modes(mut self,force:f64,absent:bool,disconnected:bool)->R<(Self,f64)>{
  if absent{self.mq=[0.;2];self.mp=[0.;2];self.energy=0.;self.check()?;return Ok((self,force));}
  let t=1.+0.5*self.r*self.r+if disconnected{0.}else{0.1*self.bound+0.05*self.weight};self.energy=0.;
  for(n,sign,k)in[(1.,1.,0),(3.,-1.,1)]{let omega=0.3*n;self.mp[k]+=0.2*(sign*force-t*omega*omega*self.mq[k]-0.06*self.mp[k]);self.mq[k]+=0.2*self.mp[k];self.energy+=(self.mp[k]*self.mp[k]+t*omega*omega*self.mq[k]*self.mq[k])/2.;}
  let out=self.energy/(1.+self.energy);self.check()?;Ok((self,out))
 }
 pub fn material(mut self,out:f64,association:f64,no_physics:bool,no_chemistry:bool,no_habituation:bool,no_hebbian:bool)->R<Self>{
  self.habituation=if no_habituation{0.}else{0.96*self.habituation+0.04*self.intensity};let drive=out/(1.+2.*self.habituation);self.field=if no_physics{drive}else{0.8*self.field+0.2*drive};
  self.bound=if no_chemistry{self.field}else{self.bound+0.12*self.field*self.field*association*(1.-self.bound)-0.004*self.bound};self.weight=if no_hebbian{0.}else{0.995*self.weight+0.005*self.field*self.bound};self.check()?;Ok(self)
 }
}
pub fn allocate(mut s:[Site;4],absent:bool)->R<[Site;4]>{let e=s.each_ref().map(|s|(4.*s.bound).exp());let total:f64=e.iter().sum();for(i,site)in s.iter_mut().enumerate(){site.output=site.bound*if absent{1.}else{e[i]/total};site.check()?;}Ok(s)}
#[derive(Clone)]
pub struct Orbit {l:f64,hx:f64,hy:f64}
impl Orbit {
    pub fn new(seed:u64)->Self {
        let mut s=Self{l:0.13+(seed%97)as f64*0.001,hx:0.1+(seed%31)as f64*0.001,hy:0.1};
        for _ in 0..32 {s.next();}s
    }
    fn next(&mut self)->(f64,f64) {
        self.l=4.*self.l*(1.-self.l);let x=1.-1.4*self.hx*self.hx+self.hy;
        self.hy=0.3*self.hx;self.hx=x;(2.*self.l-1.,self.hx.tanh())
    }
    pub fn factors(&mut self,kind:&str)->R<([f64;4],[f64;4])> {
        let mut e=[1.;4];let mut a=[1.;4];
        for k in 0..2 {
            let(l,h)=self.next();let(u,v)=match kind {"off"=>(0.,0.),"logistic"=>(l,0.),"henon"=>(0.,h),"both"=>(l,h),"logistic-late"=>(0.,l),"henon-early"=>(h,0.),_=>return Err("unknown chaos".into())};
            e[2*k]=1.+0.08*u;e[2*k+1]=1.-0.08*u;a[2*k]=1.+0.08*v;a[2*k+1]=1.-0.08*v;
        }Ok((e,a))
    }
}



pub type Matrix=[[C;4];4];
pub fn trace(a:&Matrix)->f64{(0..4).map(|i|a[i][i].re).sum()}
pub fn moment(a:&Matrix)->f64{a.iter().flatten().map(|z|z.norm2()).sum()}
pub fn outer(v:&[C;4])->Matrix{std::array::from_fn(|i|std::array::from_fn(|j|v[i].times(v[j].conj())))}
fn determinant(a:&Matrix,rows:&[usize],cols:&[usize])->C{
 if rows.is_empty(){return C::new(1.,0.);}let mut sum=C::default();
 for(k,j)in cols.iter().enumerate(){let rest:Vec<usize>=cols.iter().copied().filter(|c|c!=j).collect();sum=sum.plus(a[rows[0]][*j].times(determinant(a,&rows[1..],&rest)).scale(if k%2==0{1.}else{-1.}));}sum
}
pub fn check_matrix(a:&Matrix)->R<()>{
 if a.iter().flatten().any(|z|!z.re.is_finite()||!z.im.is_finite()){return Err("matrix finite".into());}
 let t=trace(a);if !(0. ..64.).contains(&t){return Err("occupation trace".into());}let scale=t.max(1.);
 for i in 0..4{for j in 0..4{if a[i][j].plus(a[j][i].conj().scale(-1.)).norm2().sqrt()>1e-10*scale{return Err("Hermitian".into());}}}
 for mask in 1u32..16{let ix:Vec<usize>=(0..4).filter(|i|mask&(1<<i)!=0).collect();let d=determinant(a,&ix,&ix);let tol=1e-10*scale.powi(ix.len() as i32);if d.im.abs()>tol||d.re< -tol{return Err("PSD principal minor".into());}}
 let m=moment(a);if m<t*t/4.-1e-10*scale*scale||m>t*t+1e-10*scale*scale{return Err("second moment".into());}Ok(())
}
pub fn unitary(theta:f64)->R<Matrix>{if !theta.is_finite(){return Err("hopping angle".into());}let c=theta.cos();let s=theta.sin();Ok(std::array::from_fn(|i|std::array::from_fn(|j|if i==j{C::new(c*c,0.)}else if i^j==3{C::new(-s*s,0.)}else{C::new(0.,-c*s)})))}
pub fn source(a:Matrix,mut psi:[C;4],s:[C;4],coherent:bool,population:bool)->R<(Matrix,[C;4],f64,f64,f64)>{
 let before=trace(&a);let power=0.12*s.iter().map(|z|z.norm2()).sum::<f64>();let mut next=a;
 if coherent{for i in 0..4{psi[i]=psi[i].scale(0.95f64.sqrt()).plus(s[i].scale(0.12f64.sqrt()));}next=outer(&psi);}
 else{let pump=outer(&s);for i in 0..4{for j in 0..4{next[i][j]=a[i][j].scale(0.95).plus(pump[i][j].scale(if population&&i!=j{0.}else{0.12}));}}}
 check_matrix(&next)?;let after=trace(&next);if !coherent&&(after-0.95*before-power).abs()>1e-10{return Err("pump trace".into());}Ok((next,psi,before,after,power))
}
pub fn transport(a:Matrix,mut psi:[C;4],theta:f64,absent:bool,population:bool,coherent:bool)->R<(Matrix,[C;4],f64,f64)>{
 let before=trace(&a);let mut next=a;
 if !absent{let u=unitary(theta)?;
  if coherent{let old=psi;for i in 0..4{psi[i]=(0..4).fold(C::default(),|v,j|v.plus(u[i][j].times(old[j])));}next=outer(&psi);}
  else if population{next=[[C::default();4];4];for i in 0..4{next[i][i]=C::new((0..4).map(|j|u[i][j].norm2()*a[j][j].re).sum(),0.);}}
  else{let mut left=[[C::default();4];4];for i in 0..4{for j in 0..4{left[i][j]=(0..4).fold(C::default(),|v,k|v.plus(u[i][k].times(a[k][j])));}}
   for i in 0..4{for j in 0..4{next[i][j]=(0..4).fold(C::default(),|v,k|v.plus(left[i][k].times(u[j][k].conj())));}}}
 }check_matrix(&next)?;let after=trace(&next);if (before-after).abs()>1e-10{return Err("transport trace".into());}Ok((next,psi,before,after))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn mixed_rank_two(){let z=[[C::default();4];4];let a=source(z,[C::default();4],[C::new(1.,0.),C::default(),C::default(),C::default()],false,false).unwrap().0;let b=source(a,[C::default();4],[C::default(),C::new(1.,0.),C::default(),C::default()],false,false).unwrap().0;assert!(b[0][0].re*b[1][1].re>0.);assert!(moment(&b)/trace(&b).powi(2)<1.);}
 #[test]fn refuse_indefinite(){let mut a=[[C::default();4];4];a[0][0]=C::new(1.,0.);a[1][1]=C::new(1.,0.);a[0][1]=C::new(2.,0.);a[1][0]=C::new(2.,0.);assert!(check_matrix(&a).is_err());}
 #[test]fn uniform_commutes(){let a=outer(&[C::new(1.,0.);4]);let(b,_,p,q)=transport(a,[C::default();4],0.08,false,false,false).unwrap();assert!((p-q).abs()<1e-14);for i in 0..4{for j in 0..4{assert!(b[i][j].plus(a[i][j].scale(-1.)).norm2()<1e-28);}}}
 #[test]fn coherent_rank_one(){let (a,_,_,_,_)=source([[C::default();4];4],[C::default();4],[C::new(1.,0.);4],true,false).unwrap();assert!((moment(&a)-trace(&a).powi(2)).abs()<1e-14);}
 #[test]fn preparation(){let s=Site::default().prepare(0.,1.,Some(1.),false,false).unwrap();assert!((s.distance-s.bath_c*s.bath_c/s.purity).abs()<1e-14);assert_eq!(s.heat+s.reservoir,0.);assert!(Site::default().prepare(f64::NAN,0.,None,false,false).is_err());}
}
