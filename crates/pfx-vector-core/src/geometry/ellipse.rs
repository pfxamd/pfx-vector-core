use crate::{Angle,Bounds,CoreError,CoreResult,Point2,Scalar};
#[derive(Clone,Copy,Debug,PartialEq)] pub struct Ellipse{pub center:Point2,pub radius_x:Scalar,pub radius_y:Scalar,pub rotation:Angle}
impl Ellipse{pub fn new(center:Point2,rx:Scalar,ry:Scalar,rotation:Angle)->CoreResult<Self>{if !center.is_finite()||!rx.is_finite()||!ry.is_finite(){return Err(CoreError::InvalidNumber)} if rx<0.0||ry<0.0{return Err(CoreError::InvalidGeometry)} Ok(Self{center,radius_x:rx,radius_y:ry,rotation})}
#[must_use] pub fn bounds(self)->Bounds{crate::EllipticalArc::new(self.center,self.radius_x,self.radius_y,self.rotation,Angle::radians(0.0),Angle::radians(core::f64::consts::TAU)).bounds()}}
