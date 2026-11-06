//! Vector-only round glass HUD artwork; no legacy square button sprites.
use super::{input::Router, layout::{Icon, Id, Layout}};
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
    paint_for(b, a, layout, animation, false);
}
pub fn paint_for(b: &mut HudBatch, a: &HudAtlas, layout: &Layout, animation: &Animator, inventory: bool) {
    for button in &layout.buttons {
        if !button.enabled || layout.hidden_groups.contains(&button.group)
            || !button.visibility.permits(inventory) || button.opacity <= 0.0 { continue; }
        let press=animation.press(button.id);
        let alpha=(button.opacity.clamp(0.0,1.0)*255.0) as u8;
        if let Some(rect) = button.rect {
            paint_rect_button(b, a, button, rect, press, alpha, animation);
            continue;
        }
        let center=[button.circle.center[0]*b.width,button.circle.center[1]*b.height];
        let radius=button.circle.radius*b.height*(1.0-0.05*press);
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
            let png = button.icon_rgba_16.as_deref();
            if let Some(png) = png {
                let side=radius*0.95;
                super::png_icon::paint(b,a,png,[center[0]-side/2.0,center[1]-side*0.67,side,side],alpha);
            }
            let icon_id = match button.icon {
                Icon::Sword => Some(Id::Attack), Icon::Use => Some(Id::Use), Icon::Jump => Some(Id::Jump),
                Icon::Sneak => Some(Id::Sneak), Icon::Sprint => Some(Id::Sprint), _ => None,
            };
            if png.is_none() && let Some(id) = icon_id { icon(b,a,id,[center[0],center[1]-radius*0.18],radius*0.65,alpha); }
            if png.is_none() && button.icon == Icon::Map {
                let c=rgba(255,255,255,alpha); let x=center[0]; let y=center[1]-radius*0.16; let r=radius*0.38;
                for (from,to) in [([x-r,y-r],[x+r,y-r]),([x+r,y-r],[x+r,y+r]),([x+r,y+r],[x-r,y+r]),([x-r,y+r],[x-r,y-r]),([x-r*0.33,y-r],[x-r*0.33,y+r]),([x+r*0.33,y-r],[x+r*0.33,y+r])] { stroke(b,a,from,to,(radius*0.06).max(0.8),c); }
            }
            if !button.label.is_empty() {
                let width=HudBatch::text_width(a,&button.label); let start=b.vertices.len();
                let y=if png.is_none() && button.icon==Icon::Text { center[1]-4.0 } else { center[1]+radius*0.43-4.0 };
                b.text(a,&button.label,center[0]-width/2.0,y,rgba(255,255,255,alpha),false);
                let scale=((radius*1.5)/width.max(1.0)).min(1.0);
                b.scale_since(start,center[0],y+4.0,scale,scale);
            }
        } else { icon(b,a,button.id,center,radius*0.95,alpha); }
    }
}

/// Zalith-style flat widget: translucent rounded panel, pressed state
/// brightens the fill, centred label text. Rectangles cost one fill plus
/// two accent lines instead of the 40-segment disks used by legacy
/// circle buttons, which keeps the per-frame HUD vertex count low.
fn paint_rect_button(
    b: &mut HudBatch, a: &HudAtlas,
    button: &super::layout::Button, rect: super::layout::Rect,
    press: f32, alpha: u8, animation: &Animator,
) {
    let [x, y, w, h] = rect.box_px(b.width, b.height);
    if button.id == Id::Joystick {
        let cx = x + w * 0.5;
        let cy = y + h * 0.5;
        let base = w.min(h) * 0.5;
        disk(b, a, [cx, cy], base, rgba(14, 10, 26, ((alpha as f32) * 0.62) as u8));
        ring(b, a, [cx, cy], base, (base * 0.03).max(1.0), rgba(195, 119, 255, (alpha as f32 * 0.55) as u8));
        let m = animation.joystick;
        let knob = [cx + m[0] * base * 0.62, cy - m[1] * base * 0.62];
        let kr = base * 0.30;
        disk(b, a, knob, kr, rgba(105, 49, 157, alpha));
        ring(b, a, knob, kr, (kr * 0.06).max(0.8), rgba(225, 178, 255, alpha));
        return;
    }
    // Pressed widgets brighten toward the reference pressed style.
    let fill = if press > 0.01 {
        let t = press.clamp(0.0, 1.0);
        rgba(
            (18.0 + 54.0 * t) as u8,
            (12.0 + 30.0 * t) as u8,
            (30.0 + 74.0 * t) as u8,
            ((alpha as f32) * (0.72 + 0.20 * t)) as u8,
        )
    } else {
        rgba(18, 12, 30, ((alpha as f32) * 0.72) as u8)
    };
    b.fill(a, x, y, w, h, fill);
    b.fill(a, x, y + h - 1.5, w, 1.5, rgba(195, 119, 255, (alpha as f32 * 0.55) as u8));
    b.fill(a, x, y, w, 1.0, rgba(203, 121, 255, (alpha as f32 / 3.0) as u8));
    let png = button.icon_rgba_16.as_deref();
    if let Some(png) = png {
        let side = w.min(h) * 0.52;
        super::png_icon::paint(b, a, png, [x + (w - side) * 0.5, y + (h - side) * 0.62, side, side], alpha);
    }
    if button.label.is_empty() { return; }
    let label_w = HudBatch::text_width(a, &button.label);
    let start = b.vertices.len();
    let scale = ((w - 4.0) / label_w.max(1.0)).min(1.0);
    let text_y = match png {
        Some(_) => y + h * 0.72 - 4.0,
        None => y + (h - 8.0) * 0.5,
    };
    b.text(a, &button.label, x + (w - label_w * scale) * 0.5, text_y, rgba(250, 245, 255, alpha), false);
    if (scale - 1.0).abs() > f32::EPSILON {
        b.scale_since(start, x + w * 0.5, text_y + 4.0, scale, 1.0);
    }
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
