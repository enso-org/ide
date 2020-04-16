

use crate::prelude::*;

use crate::data::color;
use std::str::FromStr;


#[derive(Clone,Debug)]
pub enum Data {
    Invalid(String),
    Number(f32),
    Srgba(color::Srgba),
}


impl From<f32> for Data {
    fn from(t:f32) -> Self {
        Self::Number(t)
    }
}

impl From<i32> for Data {
    fn from(t:i32) -> Self {
        Self::Number(t as f32)
    }
}

impl From<color::Srgba> for Data {
    fn from(t:color::Srgba) -> Self {
        Self::Srgba(t)
    }
}


pub fn data<T:Into<Data>>(t:T) -> Data {
    t.into()
}

impl Data {
    fn red(&self) -> Data {
        match self {
            Data::Srgba(color) => Data::Number(color.red),
            Data::Invalid(t)   => Data::Invalid(t.into()),
            _ => Data::Invalid(format!("Cannot read read component of {:?}.", self)),
        }
    }

    fn green(&self) -> Data {
        match self {
            Data::Srgba(color) => Data::Number(color.green),
            Data::Invalid(t)   => Data::Invalid(t.into()),
            _ => Data::Invalid(format!("Cannot read green component of {:?}.", self)),
        }
    }

    fn blue(&self) -> Data {
        match self {
            Data::Srgba(color) => Data::Number(color.blue),
            Data::Invalid(t)   => Data::Invalid(t.into()),
            _ => Data::Invalid(format!("Cannot read blue component of {:?}.", self)),
        }
    }

    fn alpha(&self) -> Data {
        match self {
            Data::Srgba(color) => Data::Number(color.alpha),
            Data::Invalid(t)   => Data::Invalid(t.into()),
            _ => Data::Invalid(format!("Cannot read alpha component of {:?}.", self)),
        }
    }
}

impl Add<Data> for Data {
    type Output = Data;
    fn add(self, rhs:Data) -> Self::Output {
        match (self,rhs) {
            ( Data::Number(lhs) , Data::Number(rhs) ) => Data::Number(lhs + rhs),
            ( Data::Invalid(t)  , _                 ) => Data::Invalid(t),
            ( _                 , Data::Invalid(t)  ) => Data::Invalid(t),
            (lhs,rhs) => Data::Invalid(format!("Cannot add {:?} to {:?}.",lhs,rhs)),
        }
    }
}

impl Sub<Data> for Data {
    type Output = Data;
    fn sub(self, rhs:Data) -> Self::Output {
        match (self,rhs) {
            ( Data::Number(lhs) , Data::Number(rhs) ) => Data::Number(lhs - rhs),
            ( Data::Invalid(t)  , _                 ) => Data::Invalid(t),
            ( _                 , Data::Invalid(t)  ) => Data::Invalid(t),
            (lhs,rhs) => Data::Invalid(format!("Cannot subtract {:?} from {:?}.",rhs,lhs)),
        }
    }
}

impl Mul<Data> for Data {
    type Output = Data;
    fn mul(self, rhs:Data) -> Self::Output {
        match (self,rhs) {
            ( Data::Number(lhs) , Data::Number(rhs) ) => Data::Number(lhs * rhs),
            ( Data::Invalid(t)  , _                 ) => Data::Invalid(t),
            ( _                 , Data::Invalid(t)  ) => Data::Invalid(t),
            (lhs,rhs) => Data::Invalid(format!("Cannot multiply {:?} by {:?}.",lhs,rhs)),
        }
    }
}

impl Div<Data> for Data {
    type Output = Data;
    fn div(self, rhs:Data) -> Self::Output {
        match (self,rhs) {
            ( Data::Number(lhs) , Data::Number(rhs) ) => Data::Number(lhs * rhs),
            ( Data::Invalid(t)  , _                 ) => Data::Invalid(t),
            ( _                 , Data::Invalid(t)  ) => Data::Invalid(t),
            (lhs,rhs) => Data::Invalid(format!("Cannot divide {:?} by {:?}.",lhs,rhs)),
        }
    }
}


// ============
// === Expr ===
// ============

#[derive(Clone,Debug)]
pub enum Expr {
    Var(String),
    Data(Data),
    Mul(Box<Expr>,Box<Expr>),
    Div(Box<Expr>,Box<Expr>),
    Add(Box<Expr>,Box<Expr>),
    Sub(Box<Expr>,Box<Expr>),
}

impl From<f32> for Expr {
    fn from(t:f32) -> Self {
        Self::Data(t.into())
    }
}

impl From<i32> for Expr {
    fn from(t:i32) -> Self {
        Self::Data(t.into())
    }
}

impl From<color::Srgba> for Expr {
    fn from(t:color::Srgba) -> Self {
        Self::Data(t.into())
    }
}

impl Mul<Expr> for Expr {
    type Output = Expr;
    fn mul(self, rhs:Expr) -> Expr {
        Expr::Mul(Box::new(self),Box::new(rhs))
    }
}

impl Div<Expr> for Expr {
    type Output = Expr;
    fn div(self, rhs:Expr) -> Expr {
        Expr::Div(Box::new(self),Box::new(rhs))
    }
}

impl Add<Expr> for Expr {
    type Output = Expr;
    fn add(self, rhs:Expr) -> Expr {
        Expr::Add(Box::new(self),Box::new(rhs))
    }
}

impl Sub<Expr> for Expr {
    type Output = Expr;
    fn sub(self, rhs:Expr) -> Expr {
        Expr::Sub(Box::new(self),Box::new(rhs))
    }
}


pub fn expr<T:Into<Expr>>(t:T) -> Expr {
    t.into()
}


#[derive(Debug,Default)]
pub struct VarMap {
    map : HashMap<String,Expr>
}


#[derive(Debug,Default)]
pub struct PartiallyResolvedVarMap {
    resolved   : HashMap<String,Data>,
    unresolved : HashMap<String,Expr>,
}



#[derive(Debug,Default)]
pub struct Resolver {
    during_resolution : HashSet<String>,
    to_be_resolved    : Vec<String>,
    map               : PartiallyResolvedVarMap,
}

impl Resolver {
    fn resolve_var(&mut self, target:&str) -> Data {
        if self.during_resolution.get(target).is_some() {
            todo!()
        }

        let expr = self.map.unresolved.remove(target).unwrap();
        self.map.unresolved.remove(target);

        self.resolve_expr(&expr)
        //                self.map.resolved.insert(target.into(),t.clone());
//                self.during_resolution.remove(target);
    }

    fn resolve_expr(&mut self, expr:&Expr) -> Data {
        match expr {
            Expr::Data(t)      => t.clone(),
            Expr::Add(lhs,rhs) => self.resolve_expr(lhs) + self.resolve_expr(rhs),
            Expr::Sub(lhs,rhs) => self.resolve_expr(lhs) - self.resolve_expr(rhs),
            Expr::Mul(lhs,rhs) => self.resolve_expr(lhs) * self.resolve_expr(rhs),
            Expr::Div(lhs,rhs) => self.resolve_expr(lhs) / self.resolve_expr(rhs),
            _ => todo!()
        }
    }
}

//impl Expr {
//    pub fn red(&self) ->
//}

pub fn test() {
    let e1 = expr(2.0) - expr(color::Srgba::new(0.0,1.0,0.0,1.0));
    let mut r = Resolver::default();
    r.map.unresolved.insert("t".into(),e1);
    println!("{:?}",r.resolve_var("t"));
}
