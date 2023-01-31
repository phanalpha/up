use rustix::process;
use rustix::process::{Pid, Signal};
use std::env;
use std::process::{exit, Command, ExitCode};
use tokio::sync::oneshot;
use tokio::time::Instant;
use tokio::{select, signal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args();
    let up = args.next().unwrap();
    let p = args.next();
    if p.is_none() {
        anyhow::bail!("{} program [arguments]", up);
    }

    let mut command = Command::new(p.unwrap());
    command.args(args);

    let mut i = Instant::now();
    loop {
        let mut child = command.spawn().unwrap();
        let pid = Pid::from_child(&child);

        let (tx, mut rx) = oneshot::channel();
        tokio::spawn(async move {
            tx.send(child.wait()).unwrap();
        });

        loop {
            select! {
                result = &mut rx => {
                    if let Err(e) = result {
                        println!("{}", e);
                    }
                    break;
                }
                _ = signal::ctrl_c() => {
                    if i.elapsed().as_millis() < 350 {
                        process::kill_process(pid, Signal::Kill);
                        return Ok(())
                    }

                    process::kill_process(pid, Signal::Int);
                    i = Instant::now()
                }
            }
        }
    }
}
