mod cli;
mod executor;
mod filters;
mod generator;

use std::sync::Arc;
use std::{env, fs};

use crate::{cli::Args, executor::Executor, generator::Generator};

use anyhow::{anyhow, bail};
use clap::Parser;
use tiny_http::{Header, Response, Server, StatusCode};

pub use anyhow::Result;

#[macro_use]
extern crate log;

fn main() {
    if let Err(e) = run() {
        error!("error: {}", e);
    }
}

fn run() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::builder().init();

    let args = Args::parse();

    let addr = format!("0.0.0.0:{}", args.port);

    let workdir = args
        .workdir
        .unwrap_or_else(|| env::current_dir().expect("Cannot get currenct dir"));

    if !fs::metadata(&workdir)
        .map(|v| v.is_dir())
        .unwrap_or_default()
    {
        bail!("Invalid workdir `{}`", workdir.display());
    }

    let generator = Arc::new(Generator::new(&workdir)?);

    let server = Server::http(&addr).map_err(|e| anyhow!("Cannot bind addr `{}`, {}", &addr, e))?;
    info!("Listen on {}", &addr);

    let executor = Executor::default();

    for request in server.incoming_requests() {
        let generator = generator.clone();
        executor.execute(move || {
            let url = request.url().to_owned();
            let response = match generator.generate(&url) {
                Ok(data) => {
                    info!(r#"Generate `{}` {}"#, &url, data.len());
                    Response::from_data(data).with_header(Header {
                        field: "Content-Type".parse().unwrap(),
                        value: "image/png".parse().unwrap(),
                    })
                }
                Err(e) => {
                    let r400 = || {
                        error!(r#"Failed to generate `{}`, {}"#, &url, e);
                        Response::from_data(b"Bad Request".to_vec())
                            .with_status_code(StatusCode(400))
                    };
                    if let Some(e) = e.downcast_ref::<tera::Error>() {
                        if let tera::ErrorKind::TemplateNotFound(_) = e.kind {
                            Response::from_data(b"Not Found".to_vec())
                                .with_status_code(StatusCode(404))
                        } else {
                            r400()
                        }
                    } else {
                        r400()
                    }
                }
            };
            if let Err(e) = request.respond(response) {
                error!(r#"Failed to response `{}`, {}"#, &url, e)
            }
        });
    }

    Ok(())
}
