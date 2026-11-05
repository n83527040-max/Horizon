//! Explicit JSON transfer via the app's existing Control layouts document root.
//! Import changes only staged edits; Save is still required to activate them.
use super::editor::{Editor, Layouts};
use std::{fs, io::{self, Read}, path::{Path, PathBuf}};

const LIMIT: u64 = 2 * 1024 * 1024;
const TRANSFER: &str = "transfer-v1.json";

fn transfer_path(home: &Path) -> io::Result<PathBuf> {
    let dir = home.join("craftix-controls");
    match fs::symlink_metadata(&dir) {
        Ok(meta) if !meta.file_type().is_symlink() && meta.is_dir() => {},
        Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Control layouts is not a directory")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&dir)?,
        Err(error) => return Err(error),
    }
    Ok(dir.join(TRANSFER))
}

/// Export staged layouts atomically to Control layouts/transfer-v1.json.
pub fn export(home: &Path, editor: &Editor) -> io::Result<()> {
    let path=transfer_path(home)?;
    if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink() || !m.is_file()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Transfer file is not a regular file"));
    }
    editor.staged.save(&path)
}

/// Validate before replacing the staged edits. The saved layout is untouched.
pub fn import(home: &Path, editor: &mut Editor) -> io::Result<()> {
    let path=transfer_path(home)?;
    let meta=fs::symlink_metadata(&path)?;
    if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > LIMIT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Transfer file is invalid or too large"));
    }
    let mut bytes=Vec::new();
    fs::File::open(path)?.take(LIMIT+1).read_to_end(&mut bytes)?;
    let layouts=Layouts::from_json(&bytes)?;
    editor.cancel_drag();
    editor.staged=layouts;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transfer_is_transactional_bounded_and_preserves_cancel() {
        let root=std::env::temp_dir().join(format!("craftix-transfer-{}",std::process::id()));
        let _=fs::remove_dir_all(&root); fs::create_dir(&root).unwrap();
        let original=Layouts::default();
        let mut editor=Editor::new(original.clone()).unwrap();
        editor.create("Second").unwrap();
        export(&root,&editor).unwrap();
        editor.rename("Changed").unwrap();
        import(&root,&mut editor).unwrap();
        assert_eq!(editor.staged.items[1].name,"Second");
        fs::write(root.join("craftix-controls").join(TRANSFER),b"{broken").unwrap();
        let before=editor.staged.clone(); assert!(import(&root,&mut editor).is_err()); assert_eq!(editor.staged,before);
        fs::write(root.join("craftix-controls").join(TRANSFER),vec![b' '; LIMIT as usize+1]).unwrap();
        assert!(import(&root,&mut editor).is_err()); assert_eq!(editor.staged,before);
        assert_eq!(editor.cancel(),original);
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_transfer_directory() {
        use std::os::unix::fs::symlink;
        let root=std::env::temp_dir().join(format!("craftix-transfer-link-{}",std::process::id()));
        let _=fs::remove_dir_all(&root); fs::create_dir(&root).unwrap();
        symlink(std::env::temp_dir(),root.join("craftix-controls")).unwrap();
        let mut editor=Editor::new(Layouts::default()).unwrap();
        assert!(export(&root,&editor).is_err()); assert!(import(&root,&mut editor).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
