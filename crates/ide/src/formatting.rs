use std::{
    io::Write,
    process::{Command, Stdio},
};

use base_db::source_db::SourceDb;
use dissimilar::Chunk;
use ide_db::root_db::RootDb;
use line_index::{TextRange, TextSize};
use utils::{lines::LineEnding, paths::Utf8PathBuf, text_edit::TextEdit};
use vfs::FileId;

#[derive(Debug)]
pub struct FmtConfig {
    pub executable: Option<Utf8PathBuf>,
    pub args: Vec<String>,
}

pub(crate) fn format(
    db: &RootDb,
    file_id: FileId,
    line_range: Option<(usize, usize)>,
    line_ending: LineEnding,
    config: FmtConfig,
) -> anyhow::Result<Option<TextEdit>> {
    let text = db.file_text(file_id);

    let verible_fmt_path = config
        .executable
        .map_or_else(|| which::which("verible-verilog-format"), |p| Ok(p.into()))?;

    let mut cmd = Command::new(verible_fmt_path);

    cmd.args(&config.args);
    if let Some((start, end)) = line_range {
        cmd.arg("--lines").arg(format!("{}-{}", start + 1, end + 1));
    }

    let mut fmt =
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).arg("-").spawn()?;

    fmt.stdin
        .as_mut()
        .ok_or(anyhow::format_err!("verible-verilog-format: could not open stdin"))?
        .write_all(text.as_bytes())?;

    let output = fmt.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(anyhow::format_err!("verible-verilog-format failed: {}", stderr));
    }

    let (new_text, new_line_endings) = LineEnding::normalize(String::from_utf8(output.stdout)?);

    if line_ending != new_line_endings {
        let range = TextRange::up_to(TextSize::of(&*text));
        Ok(Some(TextEdit::replace(range, new_text)))
    } else if *text == new_text {
        Ok(None)
    } else {
        Ok(Some(diff(&text, &new_text)))
    }
}

fn diff(old: &str, new: &str) -> TextEdit {
    let mut builder = TextEdit::builder();
    let mut pos = TextSize::default();
    let mut chunks = dissimilar::diff(old, new).into_iter().peekable();

    while let Some(chunk) = chunks.next() {
        match chunk {
            Chunk::Equal(text) => pos += TextSize::of(text),
            Chunk::Delete(deleted) => {
                let deleted = TextSize::of(deleted);
                if let Some(&Chunk::Insert(inserted)) = chunks.peek() {
                    chunks.next();
                    builder.replace(TextRange::at(pos, deleted), inserted.into());
                } else {
                    builder.delete(TextRange::at(pos, deleted));
                }
                pos += deleted;
            }
            Chunk::Insert(inserted) => builder.insert(pos, inserted.into()),
        }
    }

    builder.finish()
}
