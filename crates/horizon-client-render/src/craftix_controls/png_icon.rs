//! Bounded PNG import from the Android Control layouts document folder.
//! The stored, portable icon is a 16x16 RGBA preview encoded in JSON.
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{fs, io::{self, Read}, path::Path};
use crate::hud::{rgba, HudAtlas, HudBatch};
const SIDE: usize = 16;
const ENCODED_LEN: usize = 1368;
const FILE_LIMIT: u64 = 1024 * 1024;

pub fn valid(data: &str) -> bool {
    data.len() == ENCODED_LEN && STANDARD.decode(data).is_ok_and(|b| b.len() == SIDE*SIDE*4)
}

pub fn load(home: &Path, filename: &str) -> io::Result<String> {
    let bad = || io::Error::new(io::ErrorKind::InvalidData, "Invalid PNG icon");
    if filename.is_empty() || filename.len() > 180 || filename.starts_with('.')
        || filename.chars().any(|c| c.is_control() || "/\\:".contains(c))
        || !filename.to_ascii_lowercase().ends_with(".png") { return Err(bad()); }
    let dir=home.join("craftix-controls");
    let meta=fs::symlink_metadata(&dir)?;
    if meta.file_type().is_symlink() || !meta.is_dir() { return Err(bad()); }
    let file=dir.join(filename);
    let meta=fs::symlink_metadata(&file)?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len()>FILE_LIMIT { return Err(bad()); }
    let mut bytes=Vec::new();
    fs::File::open(file)?.take(FILE_LIMIT+1).read_to_end(&mut bytes)?;
    if bytes.len() as u64>FILE_LIMIT || bytes.len()<24 || &bytes[..8]!=b"\x89PNG\r\n\x1a\n" || &bytes[12..16]!=b"IHDR" { return Err(bad()); }
    let width=u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| bad())?);
    let height=u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| bad())?);
    if width==0 || height==0 || width>512 || height>512 { return Err(bad()); }
    let (w,h,pixels)=crate::hud::decode_bytes(&bytes).map_err(|_| bad())?;
    if (w,h)!=(width,height) || pixels.len() != (w as usize)*(h as usize)*4 { return Err(bad()); }
    let mut out=vec![0;SIDE*SIDE*4];
    for y in 0..SIDE { for x in 0..SIDE {
        let source=((y*height as usize/SIDE)*width as usize+(x*width as usize/SIDE))*4;
        let target=(y*SIDE+x)*4;
        out[target..target+4].copy_from_slice(&pixels[source..source+4]);
    }}
    Ok(STANDARD.encode(out))
}

pub fn paint(batch: &mut HudBatch, atlas: &HudAtlas, data: &str, rect: [f32;4], opacity: u8) {
    let Ok(pixels)=STANDARD.decode(data) else { return; };
    if pixels.len()!=SIDE*SIDE*4 { return; }
    let [x,y,w,h]=rect;
    for row in 0..SIDE { for col in 0..SIDE {
        let p=(row*SIDE+col)*4;
        let alpha=(u16::from(pixels[p+3])*u16::from(opacity)/255) as u8;
        if alpha>0 { batch.fill(atlas,x+col as f32*w/SIDE as f32,y+row as f32*h/SIDE as f32,
            w/SIDE as f32,h/SIDE as f32,rgba(pixels[p],pixels[p+1],pixels[p+2],alpha)); }
    }}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_bounded_png_and_rejects_bad_names_and_malformed_data() {
        let root=std::env::temp_dir().join(format!("craftix-png-{}",std::process::id()));
        let _=fs::remove_dir_all(&root); fs::create_dir_all(root.join("craftix-controls")).unwrap();
        let mut data=Vec::new();
        { let mut encoder=png::Encoder::new(&mut data,2,2); encoder.set_color(png::ColorType::Rgba);
          let mut writer=encoder.write_header().unwrap(); writer.write_image_data(&[255,0,0,255, 0,255,0,255, 0,0,255,255, 0,0,0,0]).unwrap(); }
        fs::write(root.join("craftix-controls/icon.png"),data).unwrap();
        let icon=load(&root,"icon.png").unwrap(); assert!(valid(&icon));
        let mut editor=crate::craftix_controls::editor::Editor::new(crate::craftix_controls::editor::Layouts::default()).unwrap();
        let id=editor.add_key("Icon","key.keyboard.k",crate::craftix_controls::layout::Icon::Text).unwrap();
        let button=editor.staged.items[0].buttons.iter_mut().find(|b| b.id==id).unwrap();
        button.icon_rgba_16=Some(icon.clone());
        let json=editor.staged.to_json().unwrap();
        let loaded=crate::craftix_controls::editor::Layouts::from_json(&json).unwrap();
        assert_eq!(loaded.items[0].buttons.iter().find(|b| b.id==id).unwrap().icon_rgba_16.as_deref(),Some(icon.as_str()));
        let mut batch=HudBatch::new([640,360]); let atlas=HudAtlas::boot();
        paint(&mut batch,&atlas,&icon,[0.0,0.0,32.0,32.0],255);
        assert!(!batch.vertices.is_empty());
        assert!(load(&root,"../icon.png").is_err()); assert!(!valid("bad"));
        fs::write(root.join("craftix-controls/bad.png"),b"not a png").unwrap(); assert!(load(&root,"bad.png").is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
