use super::{CreateOptions, MbrCreateMode};
use anyhow::Context;
use diskutil::disk::Disk;
use diskutil::part::gpt::Gpt;
use diskutil::part::mbr::Mbr;

pub fn create(disk: &mut dyn Disk, opt: &CreateOptions) -> anyhow::Result<()> {
    match opt.mbr_mode {
        MbrCreateMode::Protective => Mbr::create_protective(disk)
            .update()
            .context("failed to write MBR")?,
    }

    Gpt::create(disk)
        .context("failed to create GPT")?
        .update(disk)
        .context("failed to write GPT")?;

    Ok(())
}
