use anyhow::{anyhow, bail};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use tiny_http::{Header, Request, Response, StatusCode};
use url::Url;

use tera::{Context, Tera};

use crate::filters::register;

pub(crate) const MIME_SVG: &str = "image/svg+xml";
pub(crate) const MIME_PNG: &str = "image/png";

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
        register(&mut tera);

        for (svg_path, content) in svg_files {
            if let Some(url) = extract_svg_url(svg_path.as_path(), workdir) {
                tera.add_raw_template(&url, &content)
                    .map_err(|e| anyhow!("Failed to load svg `{}`, {}", svg_path.display(), e))?;
                info!("Mount `{}`", &url);
            }
        }

        let mut opt = usvg::Options {
            resources_dir: Some(workdir.to_path_buf()),
            ..Default::default()
        };

        opt.fontdb.load_system_fonts();
        opt.fontdb.load_fonts_dir(&workdir);

        Ok(Self { opt, tera })
    }

    pub fn handle(&self, request: &Request) -> Response<Cursor<Vec<u8>>> {
        let url = request.url().to_owned();
        match self.handle_impl(&url) {
            Ok((mime, data)) => {
                info!(r#"Generate `{}`, len {}"#, &url, data.len());
                Response::from_data(data).with_header(Header {
                    field: "Content-Type".parse().unwrap(),
                    value: mime.parse().unwrap(),
                })
            }
            Err(e) => {
                let r400 = || {
                    let errs: Vec<String> = e.chain().skip(1).map(|v| v.to_string()).collect();
                    let cause = errs.join(", ");
                    error!(r#"Failed to generate `{}`, {}, {}"#, &url, e, cause,);
                    Response::from_data(b"Bad Request".to_vec()).with_status_code(StatusCode(400))
                };
                if let Some(e) = e.downcast_ref::<tera::Error>() {
                    if let tera::ErrorKind::TemplateNotFound(_) = e.kind {
                        Response::from_data(b"Not Found".to_vec()).with_status_code(StatusCode(404))
                    } else {
                        r400()
                    }
                } else {
                    r400()
                }
            }
        }
    }

    fn handle_impl(&self, raw_url: &str) -> crate::Result<(&str, Vec<u8>)> {
        trace!("Start handle {}", &raw_url);
        let url: Url = format!("http://localhost{}", raw_url).parse()?;
        let mut ctx = Context::new();
        for (k, v) in url.query_pairs() {
            ctx.insert(k, &v);
        }
        let svg_data = self.tera.render(url.path(), &ctx)?;
        trace!("Done template svg {}", &raw_url);
        if ctx.get("export_svg").is_some() {
            return Ok((MIME_SVG, svg_data.as_bytes().to_vec()));
        }
        let png_data = self.svg_to_png(raw_url, &svg_data)?;
        Ok((MIME_PNG, png_data))
    }

    fn svg_to_png(&self, raw_url: &str, svg_data: &str) -> crate::Result<Vec<u8>> {
        let rtree = usvg::Tree::from_data(svg_data.as_bytes(), &self.opt.to_ref())?;
        let pixmap_size = rtree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| anyhow!("Invalid size {}", &pixmap_size))?;
        trace!("Start pixmap svg {}", &raw_url);
        resvg::render(
            &rtree,
            usvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .ok_or_else(|| anyhow!("Cannot generate bitmap"))?;
        trace!("Start encode png {}", &raw_url);
        let output = pixmap.encode_png()?;
        Ok(output)
    }
}

fn load_svg_files(svgs: &mut Vec<(PathBuf, String)>, dir: &Path) {
    let svgs_dir = match std::fs::read_dir(dir) {
        Ok(dir) => dir,
        Err(_) => return,
    };
    for entry in svgs_dir.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some("svg") = path.extension().and_then(|e| e.to_str()) {
                match fs::read_to_string(&path) {
                    Ok(v) => {
                        svgs.push((path.to_path_buf(), v));
                    }
                    Err(e) => {
                        warn!("Failed to load '{}' cause {}.", path.display(), e);
                    }
                }
            }
        } else if path.is_dir() {
            // TODO: ignore symlinks?
            load_svg_files(svgs, &path);
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
