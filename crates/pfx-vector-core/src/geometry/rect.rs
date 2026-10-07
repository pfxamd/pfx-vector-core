use crate::{Bounds,CoreError,CoreResult,Point2,Scalar};
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Rect{pub x:Scalar,pub y:Scalar,pub width:Scalar,pub height:Scalar}
impl Rect{pub fn new(x:Scalar,y:Scalar,width:Scalar,height:Scalar)->CoreResult<Self>{if![x,y,width,height].iter().all(|v|v.is_finite()){return Err(CoreError::InvalidNumber)} if width<0.0||height<0.0{return Err(CoreError::InvalidGeometry)} Ok(Self{x,y,width,height})}
#[must_use]pub fn bounds(self)->Bounds{Bounds::from_points(&[Point2::new(self.x,self.y),Point2::new(self.x+self.width,self.y+self.height)])}}
