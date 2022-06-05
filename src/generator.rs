use anyhow::{anyhow, bail};
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

use tera::{Context, Tera};

pub struct Generator {
    opt: usvg::Options,
    tera: Tera,
}

impl Generator {
    pub fn new(workdir: &Path) -> crate::Result<Self> {
        let mut svg_files = vec![];

        load_svg_files(&mut svg_files, workdir);

        if svg_files.is_empty() {
            bail!("No valid svg file found");
        }

        let mut tera = Tera::default();

        for (svg_path, content) in svg_files {
            if let Some(url) = extract_svg_url(svg_path.as_path(), workdir) {
                tera.add_raw_template(&url, &content)
                    .map_err(|e| anyhow!("Failed to load svg `{}`, {}", svg_path.display(), e))?;
                info!("Mount `{}`", &url);
            }
        }

        let mut opt = usvg::Options::default();

        opt.resources_dir = Some(workdir.to_path_buf());
        opt.fontdb.load_system_fonts();
        opt.fontdb.load_fonts_dir(&workdir);

        Ok(Self { opt, tera })
    }

    pub fn generate(&self, raw_url: &str) -> crate::Result<Vec<u8>> {
        let url: Url = format!("http://localhost{}", raw_url).parse()?;
        let mut ctx = Context::new();
        for (k, v) in url.query_pairs() {
            ctx.insert(k, &v);
        }
        let svg_data = self.tera.render(url.path(), &ctx)?;
        let rtree = usvg::Tree::from_data(svg_data.as_bytes(), &self.opt.to_ref())?;
        let pixmap_size = rtree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| anyhow!("Invalid size {}", &pixmap_size))?;
        resvg::render(
            &rtree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .ok_or_else(|| anyhow!("Cannot generate bitmap"))?;
        let output = pixmap.encode_png()?;
        Ok(output)
    }
}

fn load_svg_files(svgs: &mut Vec<(PathBuf, String)>, dir: &Path) {
    let svgs_dir = match std::fs::read_dir(dir) {
        Ok(dir) => dir,
        Err(_) => return,
    };
    for entry in svgs_dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("svg") => match fs::read_to_string(&path) {
                        Ok(v) => {
                            svgs.push((path.to_path_buf(), v));
                        }
                        Err(e) => {
                            warn!("Failed to load '{}' cause {}.", path.display(), e);
                        }
                    },
                    _ => {}
                }
            } else if path.is_dir() {
                // TODO: ignore symlinks?
                load_svg_files(svgs, &path);
            }
        }
    }
}

fn extract_svg_url(file: &Path, root: &Path) -> Option<String> {
    let path = file.strip_prefix(root).ok()?;
    let path = path.to_str()?;
    let path = path.trim_end_matches(".svg");
    let uri = if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_string()
    };
    Some(format!("/{}", uri))
}
