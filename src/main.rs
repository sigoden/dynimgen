mod cli;
mod executor;
mod filters;
mod generator;
mod state;

use std::sync::{mpsc, Arc};
use std::time::Duration;
use std::{env, fs, thread};

use crate::state::STATE;
use crate::{cli::Args, executor::Executor, generator::Generator};

use anyhow::{anyhow, bail};
use clap::Parser;
use tiny_http::Server;

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
    env_logger::builder().format_timestamp_millis().init();

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

    STATE.write().unwrap().set_allow_urls(&args.allow_urls);

    let generator = Arc::new(Generator::new(&workdir)?);

    let server = Server::http(&addr).map_err(|e| anyhow!("Cannot bind addr `{}`, {}", &addr, e))?;
    info!("Listen on {}", &addr);

    let executor = match args.pool_size.as_ref() {
        Some(size) => Executor::with_size(*size),
        None => Executor::default(),
    };

    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        tx.send(()).expect("Failed to notify exit");
    })
    .expect("Error setting Ctrl-C handler");

    let handle = thread::spawn(move || {
        while rx.try_recv().is_err() {
            // In order to reduce CPU load wait 1s for a recv before looping again
            while let Ok(Some(request)) = server.recv_timeout(Duration::from_secs(1)) {
                let generator = generator.clone();
                executor.execute(move || {
                    let url = request.url().to_owned();
                    let response = generator.handle(&request);
                    if let Err(e) = request.respond(response) {
                        error!(r#"Failed to response `{}`, {}"#, &url, e)
                    }
                });
            }
        }
        executor.join();
    });

    handle.join().unwrap();

    Ok(())
}
