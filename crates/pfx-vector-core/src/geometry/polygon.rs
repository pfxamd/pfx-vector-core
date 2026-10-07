use crate::{Bounds,Point2};
#[derive(Clone,Debug,PartialEq)]pub struct Polygon{pub points:Vec<Point2>}
impl Polygon{#[must_use]pub fn new(points:Vec<Point2>)->Self{Self{points}} #[must_use]pub fn bounds(&self)->Bounds{Bounds::from_points(&self.points)}}
