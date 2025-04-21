#![allow(dead_code)]

use std::cmp;

use crate::{
    circle::{circle, Circle},
    dist_point_circle::distance_point_circle,
    dist_point_segment::distance_point_segment,
    int_segment_segment::intersect_segment_segment,
    point::point,
    segment::Segment,
    Arc, Point,
};






















pub fn distance_segment_segment(seg0: Segment, seg1: Segment) -> (f64, Point, Point) {
    
    
    
    
    
    
    
    let result_parameter0;
    let result_parameter1;

    let p0 = seg0.p0;
    let p1 = seg0.p1;
    let q0 = seg1.p0;
    let q1 = seg1.p1;

    let p1m_p0 = p1 - p0;
    let q1m_q0 = q1 - q0;
    let p0m_q0 = p0 - q0;
    let a = p1m_p0.dot(p1m_p0);
    let b = p1m_p0.dot(q1m_q0);
    let c = q1m_q0.dot(q1m_q0);
    let d = p1m_p0.dot(p0m_q0);
    let e = q1m_q0.dot(p0m_q0);

    
    let f00 = d;
    let f10 = f00 + a;
    let f01 = f00 - b;
    let f11 = f10 - b;

    
    let g00 = -e;
    let g10 = g00 - b;
    let g01 = g00 + c;
    let g11 = g10 + c;

    const ZERO: f64 = 0f64;
    const ONE: f64 = 1f64;
    if a > ZERO && c > ZERO {
        
        
        
        
        
        
        

        let s_value = [get_clamped_root(a, f00, f10), get_clamped_root(a, f01, f11)];

        let mut classify: [i8; 2] = [0; 2];
        for i in 0..2 {
            if s_value[i] <= ZERO {
                classify[i] = -1;
            } else if s_value[i] >= ONE {
                classify[i] = 1;
            } else {
                classify[i] = 0;
            }
        }

        if classify[0] == -1 && classify[1] == -1 {
            
            result_parameter0 = ZERO;
            result_parameter1 = get_clamped_root(c, g00, g01);
        } else if classify[0] == 1 && classify[1] == 1 {
            
            result_parameter0 = ONE;
            result_parameter1 = get_clamped_root(c, g10, g11);
        } else {
            
            
            
            
            
            let (edge, end0, end1) = compute_intersection(&s_value, &classify, &b, &f00, &f10);

            
            
            
            
            
            
            
            let (param0, param1) = compute_minimum_parameters(
                &edge,
                &end0,
                &end1,
                &b,
                &c,
                &e,
                &g00,
                &g10,
                &g01,
                &g11,
            );
            result_parameter0 = param0;
            result_parameter1 = param1;
        }
    } else {
        if a > ZERO {
            
            
            
            
            
            result_parameter0 = get_clamped_root(a, f00, f10);
            result_parameter1 = ZERO;
        } else if c > ZERO {
            
            
            
            
            
            result_parameter0 = ZERO;
            result_parameter1 = get_clamped_root(c, g00, g01);
        } else {
            
            result_parameter0 = ZERO;
            result_parameter1 = ZERO;
        }
    }

    let result0 = p0 * (ONE - result_parameter0) + p1 * result_parameter0;
    let result1 = q0 * (ONE - result_parameter1) + q1 * result_parameter1;
    let distance = (result0-result1).norm_imp();
    return (distance, result0, result1);
}




fn get_clamped_root(slope: f64, h0: f64, h1: f64) -> f64 {
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    const ZERO: f64 = 0f64;
    const ONE: f64 = 1f64;

    let mut r: f64;
    if h0 < ZERO {
        if h1 > ZERO {
            r = -h0 / slope;
            if r > ONE {
                r = 0.5;
            }
            
            
        } else {
            r = ONE;
        }
    } else {
        r = ZERO;
    }
    return r;
}





fn compute_intersection(
    s_value: &[f64; 2],
    classify: &[i8; 2],
    b: &f64,
    f00: &f64,
    f10: &f64,
) -> ([i8; 2], [f64; 2], [f64; 2]) {
    
    
    
    
    
    
    
    
    
    
    
    
    
    let mut edge: [i8; 2] = [0; 2];
    let mut end0: [f64; 2] = [0.0; 2];
    let mut end1: [f64; 2] = [0.0; 2];

    const ONE: f64 = 1.0;
    const ZERO: f64 = 0.0;
    const HALF: f64 = 0.5;
    if classify[0] < 0 {
        edge[0] = 0;
        end0[0] = ZERO;
        end0[1] = f00 / b;
        if end0[1] < ZERO || end0[1] > ONE {
            end0[1] = HALF;
        }

        if classify[1] == 0 {
            edge[1] = 3;
            end1[0] = s_value[1];
            end1[1] = ONE;
        } else {
            
            edge[1] = 1;
            end1[0] = ONE;
            end1[1] = f10 / b;
            if end1[1] < ZERO || end1[1] > ONE {
                end1[1] = HALF;
            }
        }
    } else if classify[0] == 0 {
        edge[0] = 2;
        end0[0] = s_value[0];
        end0[1] = ZERO;

        if classify[1] < 0 {
            edge[1] = 0;
            end1[0] = ZERO;
            end1[1] = f00 / b;
            if end1[1] < ZERO || end1[1] > ONE {
                end1[1] = HALF;
            }
        } else if classify[1] == 0 {
            edge[1] = 3;
            end1[0] = s_value[1];
            end1[1] = ONE;
        } else {
            edge[1] = 1;
            end1[0] = ONE;
            end1[1] = f10 / b;
            if end1[1] < ZERO || end1[1] > ONE {
                end1[1] = HALF;
            }
        }
    } else {
        
        edge[0] = 1;
        end0[0] = ONE;
        end0[1] = f10 / b;
        if end0[1] < ZERO || end0[1] > ONE {
            end0[1] = HALF;
        }

        if classify[1] == 0 {
            edge[1] = 3;
            end1[0] = s_value[1];
            end1[1] = ONE;
        } else {
            edge[1] = 0;
            end1[0] = ZERO;
            end1[1] = f00 / b;
            if end1[1] < ZERO || end1[1] > ONE {
                end1[1] = HALF;
            }
        }
    }
    (edge, end0, end1)
}



fn compute_minimum_parameters(
    edge: &[i8; 2],
    end0: &[f64; 2],
    end1: &[f64; 2],
    b: &f64,
    c: &f64,
    e: &f64,
    g00: &f64,
    g10: &f64,
    g01: &f64,
    g11: &f64,
) -> (f64, f64) {
    const ZERO: f64 = 0.0;
    const ONE: f64 = 1.0;
    let parameter0;
    let parameter1;

    let delta = end1[1] - end0[1];
    let h0 = delta * (-b * end0[0] + c * end0[1] - e);
    if h0 >= ZERO {
        if edge[0] == 0 {
            parameter0 = ZERO;
            parameter1 = get_clamped_root(*c, *g00, *g01);
        } else if edge[0] == 1 {
            parameter0 = ONE;
            parameter1 = get_clamped_root(*c, *g10, *g11);
        } else {
            parameter0 = end0[0];
            parameter1 = end0[1];
        }
    } else {
        let h1 = delta * (-b * end1[0] + c * end1[1] - e);
        if h1 <= ZERO {
            if edge[1] == 0 {
                parameter0 = ZERO;
                parameter1 = get_clamped_root(*c, *g00, *g01);
            } else if edge[1] == 1 {
                parameter0 = ONE;
                parameter1 = get_clamped_root(*c, *g10, *g11);
            } else {
                parameter0 = end1[0];
                parameter1 = end1[1];
            }
        } else
        
        {
            let z = min(max(h0 / (h0 - h1), ZERO), ONE);
            let omz = ONE - z;
            parameter0 = omz * end0[0] + z * end1[0];
            parameter1 = omz * end0[1] + z * end1[1];
        }
    }
    (parameter0, parameter1)
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


pub fn distance_segment_segment_fake(seg0: Segment, seg1: Segment) -> (Point, f64) {
    let inter = intersect_segment_segment(seg0, seg1);
    match inter {
        crate::int_segment_segment::SegmentConfig::NoIntersection() => {
            let mut v: Vec<f64> = Vec::new();
            v.push(distance_point_segment(seg0.p0, seg1).1);
            v.push(distance_point_segment(seg0.p1, seg1).1);
            v.push(distance_point_segment(seg1.p0, seg0).1);
            v.push(distance_point_segment(seg1.p1, seg0).1);
            let mm;
            let m = v.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap());
            match m {
                Some(d) => {
                    mm = d;
                }
                None => {
                    panic!(); 
                }
            }
            return (point(0.0, 0.0), mm);
        }
        _ => return (point(0.0, 0.0), 0f64),
    }
}

#[cfg(test)]
mod tests_distance_segment_segment {
    use super::*;

    #[test]
    fn test_distance_segment_segment() {}
}
