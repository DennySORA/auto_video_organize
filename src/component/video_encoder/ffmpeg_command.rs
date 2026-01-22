use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FfmpegCommand {
    source_path: PathBuf,
    destination_path: PathBuf,
}

impl FfmpegCommand {
    #[must_use]
    pub fn new(source_path: &Path) -> Self {
        let destination_path = Self::generate_destination_path(source_path);
        Self {
            source_path: source_path.to_path_buf(),
            destination_path,
        }
    }

    fn generate_destination_path(source_path: &Path) -> PathBuf {
        let file_stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = source_path.parent().unwrap_or(Path::new("."));
        parent.join(format!("{file_stem}.convert.mkv"))
    }

    #[must_use]
    pub fn destination_path(&self) -> &Path {
        &self.destination_path
    }

    #[must_use]
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new("ffmpeg");

        cmd.args([
            "-hide_banner",
            "-nostdin",
            "-loglevel", "error",
            "-protocol_whitelist", "file,pipe,fd",
            "-max_streams", "8",
            "-probesize", "1000000",
            "-analyzeduration", "1000000",
            "-max_probe_packets", "512",
            "-err_detect", "careful",
            "-fflags", "+genpts+discardcorrupt+bitexact+igndts",
            "-flags:v", "+bitexact",
            "-flags:a", "+bitexact",
            "-i", &format!("file:{}", self.source_path.display()),
            "-map", "0:v:0",
            "-map", "0:a:0?",
            "-sn", "-dn",
            "-map", "-0:s",
            "-map", "-0:d",
            "-map", "-0:t",
            "-map", "-0:v:m:attached_pic",
            "-map_metadata", "-1",
            "-map_metadata:s", "-1",
            "-map_chapters", "-1",
            "-avoid_negative_ts", "make_zero",
            "-vf", "scale=round(iw*if(sar\\,sar\\,1)/2)*2:round(ih/2)*2,setsar=1,format=yuv420p10le",
            "-c:v", "libx265",
            "-profile:v", "main10",
            "-pix_fmt", "yuv420p10le",
            "-udu_sei", "0",
            "-preset", "fast",
            "-g", "60",
            "-keyint_min", "60",
            "-crf", "16",
            "-x265-params", "no-info=1:pmode=1:limit-sao=1:cutree=1:rc-lookahead=30:bframes=4:b-adapt=2:psy-rd=1.0:psy-rdoq=0.5:open-gop=0",
            "-bsf:v", "filter_units=remove_types=35|38-40",
            "-c:a", "flac",
            "-ar", "48000",
            "-ac", "2",
            "-f", "matroska",
        ]);
        cmd.arg(&self.destination_path);

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_destination_path() {
        let source = Path::new("/videos/test.mp4");
        let cmd = FfmpegCommand::new(source);
        assert_eq!(
            cmd.destination_path(),
            Path::new("/videos/test.convert.mkv")
        );
    }

    #[test]
    fn test_generate_destination_path_with_dots() {
        let source = Path::new("/videos/test.video.name.mp4");
        let cmd = FfmpegCommand::new(source);
        assert_eq!(
            cmd.destination_path(),
            Path::new("/videos/test.video.name.convert.mkv")
        );
    }
}
