use crate::{Bounds, Point2, Scalar, Tolerance, Vector2, solve_quadratic};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier { pub p0: Point2, pub p1: Point2, pub p2: Point2, pub p3: Point2 }
impl CubicBezier {
    #[must_use] pub const fn new(p0: Point2,p1:Point2,p2:Point2,p3:Point2)->Self{Self{p0,p1,p2,p3}}
    #[must_use] pub fn point_at(self,t:Scalar)->Point2{let mt=1.0-t;Point2::new(mt.powi(3)*self.p0.x+3.0*mt*mt*t*self.p1.x+3.0*mt*t*t*self.p2.x+t.powi(3)*self.p3.x,mt.powi(3)*self.p0.y+3.0*mt*mt*t*self.p1.y+3.0*mt*t*t*self.p2.y+t.powi(3)*self.p3.y)}
    #[must_use] pub fn derivative_at(self,t:Scalar)->Vector2{let mt=1.0-t;(self.p1-self.p0)*(3.0*mt*mt)+(self.p2-self.p1)*(6.0*mt*t)+(self.p3-self.p2)*(3.0*t*t)}
    pub fn tangent_at(self,t:Scalar,tol:Tolerance)->crate::CoreResult<Vector2>{self.derivative_at(t).normalized(tol)}
    #[must_use] pub fn split(self,t:Scalar)->(Self,Self){let a=self.p0.lerp(self.p1,t);let b=self.p1.lerp(self.p2,t);let c=self.p2.lerp(self.p3,t);let d=a.lerp(b,t);let e=b.lerp(c,t);let m=d.lerp(e,t);(Self::new(self.p0,a,d,m),Self::new(m,e,c,self.p3))}
    #[must_use] pub fn bounds(self)->Bounds{let mut b=Bounds::from_points(&[self.p0,self.p3]);for axis in 0..2{let(p0,p1,p2,p3)=if axis==0{(self.p0.x,self.p1.x,self.p2.x,self.p3.x)}else{(self.p0.y,self.p1.y,self.p2.y,self.p3.y)};let a=-p0+3.0*p1-3.0*p2+p3;let bb=2.0*(p0-2.0*p1+p2);let c=p1-p0;for t in solve_quadratic(3.0*a,bb,c){if t>0.0&&t<1.0{b=b.include(self.point_at(t));}}}b}
    #[must_use]pub fn reversed(self)->Self{Self::new(self.p3,self.p2,self.p1,self.p0)}
}
