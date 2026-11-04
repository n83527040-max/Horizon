//! Vector-only round glass HUD artwork; no legacy square button sprites.
use super::{input::Router, layout::{Button, Icon, Id, Layout}};
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
/// Independent visual interpolation. The actual movement/key edges remain immediate.
#[derive(Default)]
pub struct Animator {
    joystick: [f32; 2],
    presses: std::collections::BTreeMap<Id, f32>,
}
impl Animator {
    pub fn advance(&mut self, layout: &Layout, router: &Router, elapsed_seconds: f32) {
        let dt = if elapsed_seconds.is_finite() { elapsed_seconds.clamp(0.0, 0.1) } else { 0.0 };
        let blend = 1.0 - (-16.0 * dt).exp();
        let target = router.movement();
        for i in 0..2 { self.joystick[i] += (target[i] - self.joystick[i]) * blend; }
        self.presses.retain(|id, _| layout.buttons.iter().any(|b| b.id == *id));
        for button in &layout.buttons {
            let target = if button.action.as_ref().is_some_and(|action| router.active(action)) { 1.0 } else { 0.0 };
            let value = self.presses.entry(button.id).or_default();
            *value += (target - *value) * blend;
        }
    }
    fn press(&self, id: Id) -> f32 { self.presses.get(&id).copied().unwrap_or(0.0).clamp(0.0, 1.0) }
}
/// Draw the entire replacement HUD from its data model. The caller controls
/// the frame timing and routes pointer events separately.
pub fn paint(b: &mut HudBatch, a: &HudAtlas, layout: &Layout, animation: &Animator) {
    paint_filtered(b, a, layout, animation, &|_| true);
}
/// The same painting with a host-side filter (inventory context, keypad mode).
pub fn paint_filtered(
    b: &mut HudBatch,
    a: &HudAtlas,
    layout: &Layout,
    animation: &Animator,
    keep: &dyn Fn(&Button) -> bool,
) {
    for button in &layout.buttons {
        if !button.enabled || button.opacity <= 0.0 || !keep(button) { continue; }
        let center=[button.circle.center[0]*b.width,button.circle.center[1]*b.height];
        let press=animation.press(button.id);
        let radius=button.circle.radius*b.height*(1.0-0.05*press);
        let alpha=(button.opacity.clamp(0.0,1.0)*255.0) as u8;
        let fill_r=(12.0+71.0*press) as u8;
        let fill_g=(9.0+16.0*press) as u8;
        let fill_b=(22.0+98.0*press) as u8;
        ring(b,a,center,radius+2.2,2.2,rgba(158,42,255,alpha/8));
        disk(b,a,center,radius,rgba(fill_r,fill_g,fill_b,alpha));
        ring(b,a,center,radius,(radius*0.035).max(1.0),rgba(195,119,255,alpha));
        ring(b,a,center,radius+1.0,1.0,rgba(203,121,255,alpha/3));
        if button.id==Id::Joystick {
            let m=animation.joystick;
            let knob=[center[0]+m[0]*radius*0.47,center[1]-m[1]*radius*0.47];
            disk(b,a,knob,radius*0.32,rgba(105,49,157,alpha));
            ring(b,a,knob,radius*0.32,(radius*0.025).max(0.8),rgba(225,178,255,alpha));
        } else if matches!(button.id,Id::Custom(_)) {
            let icon_id = match button.icon {
                Icon::Sword => Some(Id::Attack), Icon::Use => Some(Id::Use), Icon::Jump => Some(Id::Jump),
                Icon::Sneak => Some(Id::Sneak), Icon::Sprint => Some(Id::Sprint), _ => None,
            };
            if let Some(id) = icon_id { icon(b,a,id,[center[0],center[1]-radius*0.18],radius*0.65,alpha); }
            if button.icon == Icon::Map {
                let c=rgba(255,255,255,alpha); let x=center[0]; let y=center[1]-radius*0.16; let r=radius*0.38;
                for (from,to) in [([x-r,y-r],[x+r,y-r]),([x+r,y-r],[x+r,y+r]),([x+r,y+r],[x-r,y+r]),([x-r,y+r],[x-r,y-r]),([x-r*0.33,y-r],[x-r*0.33,y+r]),([x+r*0.33,y-r],[x+r*0.33,y+r])] { stroke(b,a,from,to,(radius*0.06).max(0.8),c); }
            }
            if !button.label.is_empty() {
                let width=HudBatch::text_width(a,&button.label); let start=b.vertices.len();
                let y=if button.icon==Icon::Text { center[1]-4.0 } else { center[1]+radius*0.43-4.0 };
                b.text(a,&button.label,center[0]-width/2.0,y,rgba(255,255,255,alpha),false);
                let scale=((radius*1.5)/width.max(1.0)).min(1.0);
                b.scale_since(start,center[0],y+4.0,scale,scale);
            }
        } else { icon(b,a,button.id,center,radius*0.95,alpha); }
    }
}
/// Mining progress drawn as an outward ring on the attack button, which keeps
/// the round silhouette instead of the legacy square gauge.
pub fn paint_break_progress(b: &mut HudBatch, a: &HudAtlas, layout: &Layout, progress: f32) {
    if !progress.is_finite() || progress <= 0.0 { return; }
    let Some(attack) = layout.button(Id::Attack) else { return; };
    if !attack.enabled { return; }
    let center = [attack.circle.center[0] * b.width, attack.circle.center[1] * b.height];
    let radius = attack.circle.radius * b.height;
    let progress = progress.clamp(0.0, 1.0);
    ring(b, a, center, radius + 4.0, 2.5, rgba(255, 255, 255, (40.0 * progress) as u8));
    let filled = std::f32::consts::TAU * progress;
    const N: usize = 40;
    let steps = ((N as f32 * progress).ceil() as usize).clamp(1, N);
    for i in 0..steps {
        let t = |j: usize| filled * j as f32 / steps as f32 - std::f32::consts::FRAC_PI_2;
        let point = |j: usize, r: f32| [center[0] + r * t(j).cos(), center[1] + r * t(j).sin()];
        b.fill_quad(
            a,
            [point(i, radius + 2.0), point(i, radius + 6.0), point(i + 1, radius + 6.0), point(i + 1, radius + 2.0)],
            rgba(226, 176, 255, 210),
        );
    }
}

/// A short-lived marker showing where the last world tap landed.
pub fn paint_tap_marker(b: &mut HudBatch, a: &HudAtlas, point: [f32; 2], strength: f32) {
    let strength = strength.clamp(0.0, 1.0);
    let center = [point[0] * b.width, point[1] * b.height];
    let radius = (b.height * 0.018) * (0.6 + 0.4 * strength);
    ring(b, a, center, radius, 1.5, rgba(220, 190, 255, (200.0 * strength) as u8));
    disk(b, a, center, radius * 0.22, rgba(240, 225, 255, (220.0 * strength) as u8));
}

/// Highlight the button the editor has selected.
pub fn paint_selection(b: &mut HudBatch, a: &HudAtlas, layout: &Layout, selected: Option<Id>) {
    let Some(id) = selected else { return; };
    let Some(button) = layout.button(id) else { return; };
    let center = [button.circle.center[0] * b.width, button.circle.center[1] * b.height];
    ring(b, a, center, button.circle.radius * b.height + 5.0, 2.0, rgba(255, 236, 140, 220));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joystick_visual_returns_smoothly_without_delaying_input() {
        let mut r=Router::default(); let l=Layout::default(); let aspect=16.0/9.0;
        let joy=l.buttons.iter().find(|b| b.id==Id::Joystick).unwrap().circle.center;
        r.begin(&l,1,joy,aspect,0); r.moved(&l,1,[joy[0],joy[1]-0.1],aspect);
        let mut anim=Animator::default(); anim.advance(&l,&r,0.1); assert!(anim.joystick[1]>0.0);
        r.end(1); assert_eq!(r.movement(),[0.0,0.0]);
        let before=anim.joystick[1]; anim.advance(&l,&r,0.02);
        assert!(anim.joystick[1]>0.0 && anim.joystick[1]<before);
        for _ in 0..100 { anim.advance(&l,&r,0.1); }
        assert!(anim.joystick[1]<0.0001);
    }
    #[test]
    fn paint_round_controls_has_bounded_geometry() {
        let mut b=HudBatch::new([1920,1080]); let a=HudAtlas::boot();
        paint(&mut b,&a,&Layout::default(),&Animator::default());
        assert!(!b.vertices.is_empty());
        assert!(b.vertices.len()<40000,"unexpected HUD geometry growth");
        assert!(b.vertices.iter().all(|v| v.pos[0].is_finite() && v.pos[1].is_finite()));
    }
}
