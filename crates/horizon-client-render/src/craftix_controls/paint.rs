//! Vector-only round glass HUD artwork; no legacy square button sprites.
use super::{input::Router, layout::{Id, Layout}};
use crate::hud::{HudAtlas, HudBatch, rgba};
use std::f32::consts::TAU;

fn disk(b: &mut HudBatch, a: &HudAtlas, center: [f32; 2], radius: f32, color: u32) {
    const N: usize = 40;
    for i in 0..N {
        let p = |j: usize| { let t=TAU*j as f32/N as f32; [center[0]+radius*t.cos(),center[1]+radius*t.sin()] };
        b.fill_quad(a, [center,p(i),p(i+1),center], color);
    }
}
fn ring(b: &mut HudBatch, a: &HudAtlas, center: [f32;2], radius: f32, width: f32, color: u32) {
    const N: usize = 40;
    for i in 0..N {
        let point=|j: usize,r:f32| { let t=TAU*j as f32/N as f32; [center[0]+r*t.cos(),center[1]+r*t.sin()] };
        b.fill_quad(a,[point(i,radius-width),point(i,radius),point(i+1,radius),point(i+1,radius-width)],color);
    }
}
fn stroke(b: &mut HudBatch, a: &HudAtlas, from: [f32;2], to: [f32;2], width: f32, color: u32) {
    let v=[to[0]-from[0],to[1]-from[1]]; let len=v[0].hypot(v[1]);
    if len <= f32::EPSILON { return; }
    let n=[-v[1]*width/(2.0*len),v[0]*width/(2.0*len)];
    b.fill_quad(a,[[from[0]-n[0],from[1]-n[1]],[to[0]-n[0],to[1]-n[1]],
        [to[0]+n[0],to[1]+n[1]],[from[0]+n[0],from[1]+n[1]]],color);
}
fn icon(b: &mut HudBatch, a: &HudAtlas, id: Id, center: [f32;2], radius: f32, alpha: u8) {
    let c=rgba(250,245,255,alpha); let w=(radius*0.085).max(0.9);
    let p=|v:[f32;2]| [center[0]+v[0]*radius,center[1]+v[1]*radius];
    let lines: &[([f32;2],[f32;2])] = match id {
        Id::Jump => &[([0.0,0.48],[0.0,-0.48]),([-0.32,-0.14],[0.0,-0.48]),([0.0,-0.48],[0.32,-0.14]),([-0.30,0.50],[0.30,0.50])],
        Id::Attack => &[([-0.42,0.45],[0.38,-0.35]),([0.17,-0.54],[0.42,-0.35]),([0.42,-0.35],[0.44,-0.07]),([-0.47,0.13],[-0.10,0.48]),([-0.45,0.52],[-0.32,0.39])],
        Id::Use => &[([-0.39,-0.24],[0.0,-0.46]),([0.0,-0.46],[0.39,-0.24]),([0.39,-0.24],[0.39,0.25]),([0.39,0.25],[0.0,0.47]),([0.0,0.47],[-0.39,0.25]),([-0.39,0.25],[-0.39,-0.24]),([-0.39,-0.24],[0.0,-0.02]),([0.0,-0.02],[0.39,-0.24]),([0.0,-0.02],[0.0,0.47])],
        Id::Sprint => &[([-0.05,-0.29],[0.15,-0.1]),([0.15,-0.1],[-0.01,0.14]),([-0.01,0.14],[-0.30,0.43]),([-0.01,0.14],[0.28,0.37]),([0.15,-0.1],[0.38,0.0]),([-0.05,-0.29],[-0.25,-0.04]),([-0.57,-0.10],[-0.34,-0.10]),([-0.57,0.08],[-0.36,0.08])],
        Id::Sneak => &[([-0.10,-0.25],[0.18,0.0]),([0.18,0.0],[-0.08,0.18]),([-0.08,0.18],[-0.38,0.42]),([-0.08,0.18],[0.34,0.42]),([0.18,0.0],[0.40,0.12])],
        Id::Pause => &[([-0.18,-0.38],[-0.18,0.38]),([0.18,-0.38],[0.18,0.38])],
        Id::Chat => &[([-0.40,-0.33],[0.40,-0.33]),([0.40,-0.33],[0.40,0.25]),([0.40,0.25],[0.0,0.25]),([0.0,0.25],[-0.20,0.43]),([-0.20,0.43],[-0.20,0.25]),([-0.20,0.25],[-0.40,0.25]),([-0.40,0.25],[-0.40,-0.33])],
        Id::Inventory => &[([-0.35,-0.27],[0.35,-0.27]),([0.35,-0.27],[0.35,0.40]),([0.35,0.40],[-0.35,0.40]),([-0.35,0.40],[-0.35,-0.27]),([-0.20,-0.27],[-0.12,-0.46]),([-0.12,-0.46],[0.12,-0.46]),([0.12,-0.46],[0.20,-0.27]),([-0.22,0.13],[0.22,0.13])],
        Id::Perspective => &[([-0.45,0.0],[-0.25,-0.23]),([-0.25,-0.23],[0.0,-0.35]),([0.0,-0.35],[0.25,-0.23]),([0.25,-0.23],[0.45,0.0]),([0.45,0.0],[0.25,0.23]),([0.25,0.23],[0.0,0.35]),([0.0,0.35],[-0.25,0.23]),([-0.25,0.23],[-0.45,0.0])],
        Id::Settings => &[([-0.45,0.0],[0.45,0.0]),([0.0,-0.45],[0.0,0.45]),([-0.32,-0.32],[0.32,0.32]),([-0.32,0.32],[0.32,-0.32])],
        Id::Players => &[([-0.42,0.42],[-0.36,0.18]),([-0.36,0.18],[-0.08,0.08]),([-0.08,0.08],[0.18,0.18]),([0.18,0.18],[0.24,0.42]),([0.24,0.42],[-0.42,0.42]),([0.30,0.08],[0.43,0.20]),([0.43,0.20],[0.45,0.36])],
        _ => &[],
    };
    for (x,y) in lines { stroke(b,a,p(*x),p(*y),w,c); }
    match id {
        Id::Sprint|Id::Sneak => disk(b,a,p([0.08,-0.47]),radius*0.12,c),
        Id::Players => { ring(b,a,p([-0.08,-0.20]),radius*0.18,w,c); ring(b,a,p([0.29,-0.22]),radius*0.13,w,c); }
        Id::Perspective => disk(b,a,center,radius*0.14,c),
        Id::More => for x in [-0.32,0.0,0.32] { disk(b,a,p([x,0.0]),radius*0.09,c); },
        Id::Settings => { disk(b,a,center,radius*0.25,rgba(18,12,30,alpha)); ring(b,a,center,radius*0.25,w,c); },
        _ => {}
    }
}
/// Draw the entire replacement HUD from its data model. The caller controls
/// the frame timing and routes pointer events separately.
pub fn paint(b: &mut HudBatch, a: &HudAtlas, layout: &Layout, router: &Router) {
    for button in &layout.buttons {
        if !button.enabled || button.opacity <= 0.0 { continue; }
        let center=[button.circle.center[0]*b.width,button.circle.center[1]*b.height];
        let radius=button.circle.radius*b.height;
        let alpha=(button.opacity.clamp(0.0,1.0)*255.0) as u8;
        let held=button.action.as_ref().is_some_and(|a| router.active(a));
        ring(b,a,center,radius+2.2,2.2,rgba(158,42,255,alpha/8));
        disk(b,a,center,radius,if held { rgba(83,25,120,alpha) } else { rgba(12,9,22,alpha) });
        ring(b,a,center,radius,(radius*0.035).max(1.0),rgba(195,119,255,alpha));
        ring(b,a,center,radius+1.0,1.0,rgba(203,121,255,alpha/3));
        if button.id==Id::Joystick {
            let m=router.movement();
            let knob=[center[0]+m[0]*radius*0.47,center[1]-m[1]*radius*0.47];
            disk(b,a,knob,radius*0.32,rgba(105,49,157,alpha));
            ring(b,a,knob,radius*0.32,(radius*0.025).max(0.8),rgba(225,178,255,alpha));
        } else if matches!(button.id,Id::Custom(_)) && !button.label.is_empty() {
            let width=HudBatch::text_width(a,&button.label); let start=b.vertices.len();
            b.text(a,&button.label,center[0]-width/2.0,center[1]-4.0,rgba(255,255,255,alpha),false);
            let scale=((radius*1.5)/width.max(1.0)).min(1.0);
            b.scale_since(start,center[0],center[1],scale,scale);
        } else { icon(b,a,button.id,center,radius*0.95,alpha); }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paint_round_controls_has_bounded_geometry() {
        let mut b=HudBatch::new([1920,1080]); let a=HudAtlas::boot();
        paint(&mut b,&a,&Layout::default(),&Router::default());
        assert!(!b.vertices.is_empty());
        assert!(b.vertices.len()<40000,"unexpected HUD geometry growth");
        assert!(b.vertices.iter().all(|v| v.pos[0].is_finite() && v.pos[1].is_finite()));
    }
}
