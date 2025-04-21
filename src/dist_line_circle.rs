#![allow(dead_code)]

use core::num;

use crate::circle::Circle;
use crate::line::Line;
use crate::point::point;
use crate::Point;

#[derive(Debug, PartialEq)]
pub enum DistLineCircleConfig {
    OnePair(f64, Point, Point),
    TwoPairs(f64, Point, Point, Point, Point),
}


















pub fn distance_line_circle(line: &Line, circle: &Circle) -> DistLineCircleConfig {
    let mut parameter: [f64; 2] = [0.0; 2];
    let mut closest: [[Point; 2]; 2] = [[point(0.0, 0.0); 2]; 2];
    let num_closest_pairs;
    
    
    let delta = line.origin - circle.c;

    
    

    
    

    
    
    
    
    
    
    
    
    const ZERO: f64 = 0.0;
    let direction = line.dir;
    let radius = circle.r;

    let dot_dir_dir = direction.dot(direction);
    let dot_dir_del = direction.dot(delta);
    let dot_perp_dir_del = direction.perp(delta);
    let r_sqr = radius * radius;
    let test = dot_perp_dir_del * dot_perp_dir_del - r_sqr * dot_dir_dir;
    if test >= ZERO {
        
        
        
        
        
        
        
        
        

        
        num_closest_pairs = 1;
        parameter[0] = -dot_dir_del / dot_dir_dir;
        closest[0][0] = delta + direction * parameter[0];
        closest[0][1] = closest[0][0];

        
        if test > ZERO {
            closest[0][1].normalize();
            closest[0][1] = closest[0][1] * radius;
        }
    } else {
        

        
        
        
        
        let a0 = delta.dot(delta) - radius * radius;
        let a1 = dot_dir_del;
        let a2 = dot_dir_dir;
        let discr = max(a1 * a1 - a0 * a2, ZERO);
        let sqrt_discr = discr.sqrt();

        
        
        let temp = -dot_dir_del
            + if dot_dir_del > ZERO {
                -sqrt_discr
            } else {
                sqrt_discr
            };
        num_closest_pairs = 2;
        parameter[0] = temp / dot_dir_dir;
        parameter[1] = a0 / temp;
        if parameter[0] > parameter[1] {
            (parameter[1], parameter[0]) = (parameter[0], parameter[1]);
        }

        
        closest[0][0] = delta + direction * parameter[0];
        closest[0][1] = closest[0][0];
        closest[1][0] = delta + direction * parameter[1];
        closest[1][1] = closest[1][0];
    }

    
    
    for j in 0..num_closest_pairs {
        for i in 0..2 {
            closest[j][i] = closest[j][i] + circle.c;
        }
    }

    let diff = closest[0][0] - closest[0][1];
    let distance = (closest[0][0] - closest[0][1]).norm_imp();

    if num_closest_pairs == 2 {
        DistLineCircleConfig::OnePair(distance, closest[0][0], closest[0][1])
    } else {
        DistLineCircleConfig::TwoPairs(distance, closest[0][0], closest[0][1], closest[1][0], closest[1][1])
    }
}

fn min(a: f64, b: f64) -> f64 {
    if a <= b {
        a
    } else {
        b
    }
}
fn max(a: f64, b: f64) -> f64 {
    if a >= b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests_distance_line_circle {
    use super::*;

    #[test]
    fn test_distance_line_circle() {}
}
