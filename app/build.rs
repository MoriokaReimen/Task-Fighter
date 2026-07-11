use std::error::Error;
use vergen_gitcl::{Emitter, Gitcl}; // 【修正】Gitcl をインポートします

fn main() -> Result<(), Box<dyn Error>> {
    // Gitの情報をすべて有効化（SHAやdescribeを含みます）
    let gitcl = Gitcl::all_git();

    Emitter::default().add_instructions(&gitcl)?.emit()?;

    Ok(())
}
