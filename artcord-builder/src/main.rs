use std::{ffi::OsStr, pin::Pin, process::ExitStatus, time::Duration};

use futures::{future::join_all, Future, FutureExt};
use notify::{event::{AccessKind, AccessMode}, Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{sync::{broadcast, mpsc}, time::sleep};
use tokio::sync::oneshot;
use tracing::{debug, debug_span, info, error, Level};
use cfg_if::cfg_if;
use tokio::process::Command;
use tokio::select;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ProjectKind {
    Front,
    Back
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ManagerEventKind {
    File(ProjectKind),
    CompilerStarted(ProjectKind),
    CompilerEnded(ProjectKind),
    CompilerKilled(ProjectKind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerState {
    Starting(ProjectKind),
    Compiling(ProjectKind),
    Killing(ProjectKind),
    Ready
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompilerEventKind {
    Start(ProjectKind),
    Kill,
}

// async fn do_stuff_async() {
//     loop {
//         debug!("boop");
//         sleep(Duration::from_secs(1)).await;
        
//     }
// }

// async fn more_async_work() {
//     debug!("beep");
//     sleep(Duration::from_secs(3)).await;
// }


#[tokio::main]
async fn main() {
   
    cfg_if! {
        if #[cfg(debug_assertions)] {
            tracing_subscriber::fmt().with_max_level(Level::DEBUG).try_init().unwrap();
        } else {
            tracing_subscriber::fmt().with_max_level(Level::INFO).try_init().unwrap();
        }
    }

    debug!("Started");

    let (send_compiler_event, recv_compiler_event) = broadcast::channel::<CompilerEventKind>(1);
    let (send_manager_event, recv_manager_event) = mpsc::channel::<ManagerEventKind>(1000);
    let mut futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = Vec::new();

    let paths_backend = [
        "artcord",
        "artcord-actix",
        "artcord-mongodb",
        "artcord-serenity",
        "artcord-state",
        "artcord-tungstenite",
    ];

    for path in paths_backend {
        let fut = watch_dir(
            path,
            send_manager_event.clone(),
            ProjectKind::Back
        );
        futs.push(fut.boxed());
    }

    let manager_fut = manager(recv_manager_event, send_compiler_event);
    futs.push(manager_fut.boxed());

    let compiler_fut = compiler(send_manager_event.clone(), recv_compiler_event);
    futs.push(compiler_fut.boxed());

    // tokio::select! {
    //     _ = do_stuff_async() => {
    //         debug!("do_stuff_async() completed first")
    //     }
    //     _ = more_async_work() => {
    //         debug!("more_async_work() completed first")
    //     }
    // };

    
    join_all(futs).await;
}

async fn compiler(send_manager_event: mpsc::Sender<ManagerEventKind>, mut recv_compiler_event: broadcast::Receiver<CompilerEventKind>) {
    let mut commands_backend = build_commands([
        vec!["cargo", "build", "--package", "artcord-leptos"],
        vec!["rm", "-r", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
        vec![
            "cp",
            "./style/output.css",
            "./target/site/pkg/leptos_start5.css",
        ],
        vec![
            "wasm-bindgen",
            "./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm",
            "--no-typescript",
            "--target",
            "web",
            "--out-dir",
            "./target/site/pkg",
            "--out-name",
            "leptos_start5",
        ],
        vec!["cargo", "build", "--package", "artcord"],
    ])
    .await;

    let mut commands_frontend = build_commands([
        vec!["cargo", "build", "--package", "artcord-leptos"],
        vec!["rm", "-r", "./target/site"],
        vec!["mkdir", "./target/site"],
        vec!["mkdir", "./target/site/pkg"],
        vec!["cp", "-r", "./assets/.", "./target/site/"],
        vec![
            "cp",
            "./style/output.css",
            "./target/site/pkg/leptos_start5.css",
        ],
        vec![
            "wasm-bindgen",
            "./target/wasm32-unknown-unknown/debug/artcord_leptos.wasm",
            "--no-typescript",
            "--target",
            "web",
            "--out-dir",
            "./target/site/pkg",
            "--out-name",
            "leptos_start5",
        ],
    ])
    .await;

    
    loop {
        let result = recv_compiler_event.recv().await;
        let Ok(compiler_event_kind) = result else {
            let err = result
            .err()
            .and_then(|e| Some(e.to_string()))
            .unwrap_or_else(|| "uwknown error".to_string());
            error!("Compiler: recv: error: {}", err);
            continue;
        };
        let  CompilerEventKind::Start(project_kind) = compiler_event_kind else {
            error!("Compiler: recv: error: received: {:?} before compiler even started.", compiler_event_kind);
            continue;
        };
        debug!("Compiler: recv: compiler_event_kind: {:?}", compiler_event_kind);

        // match compiler_event_kind {
        //     CompilerEventKind::Kill => {

        //     }
        //     CompilerEventKind::Start(project_kind) => {
               
        //     }
        // };

        debug!("Compiler: sent: file: CompilerStarted({:?})", project_kind);
        let send_result = send_manager_event.send(ManagerEventKind::CompilerStarted(project_kind)).await;
        if let Err(e) = send_result {
            error!("Compiler: sent: error: {}", e);
            continue;
        }

        // sleep(Duration::from_secs(5)).await;

       

        
        let commands: &mut [(Command, String)] = match project_kind {
            ProjectKind::Back => {
                commands_backend.as_mut()
            }
            ProjectKind::Front => {
                commands_backend.as_mut()
            }
        };

        let commands_count = commands.len();

        for (i, (command, command_name)) in commands.iter_mut().enumerate() {
            let mut command = command.spawn().unwrap();


            select! {
                command_return = command.wait() => {
                    let good = compiler_on_finish(i == commands_count - 1,project_kind, command_return, command_name, send_manager_event.clone()).await;
 
                    if !good {
                        break;
                    }
                 },
                _ = compiler_on_run(&mut recv_compiler_event, project_kind, send_manager_event.clone()) => {
                    break;
                 }
             };
        }

        
    }
}

async fn compiler_on_finish(last: bool, project_kind: ProjectKind, command_return: Result<ExitStatus, std::io::Error>, command_name: &str, send_manager_event: mpsc::Sender<ManagerEventKind>) -> bool {
    let Ok(command_return) = command_return else {
        let err = command_return
        .err()
        .and_then(|e| Some(e.to_string()))
        .unwrap_or_else(|| "uwknown error".to_string());
        error!("Compiler(COMMAND-{:?}): error: compiler_event_kind: {:?}", project_kind, err);
        return false;
    };

    let good = command_return.success();
    if good {
        debug!("Compiler(COMMAND-{:?}): finished: {}", project_kind, command_name);
        if last {
            debug!("Compiler(COMMAND-{:?}): sent: CompilerEnded({:?})", project_kind, project_kind);
            let send_result = send_manager_event.send(ManagerEventKind::CompilerEnded(project_kind)).await;
            if let Err(e) = send_result {
                error!("Compiler(RUNNING-{:?}): sent: error: {}", project_kind, e);
            }
        }
        true
    } else {
        error!("Compiler(COMMAND-{:?}): failed: {}", project_kind, command_name);
        false
    }
}

async fn compiler_on_run(recv_compiler_event: &mut broadcast::Receiver<CompilerEventKind>, project_kind: ProjectKind, send_manager_event: mpsc::Sender<ManagerEventKind>) {
    while let Ok(compiler_event_kind) = recv_compiler_event.recv().await {
        match compiler_event_kind {
            CompilerEventKind::Kill => {
                debug!("Compiler(RUNNING-{:?}): sent: CompilerKilled({:?})", project_kind, project_kind);
                let send_result = send_manager_event.send(ManagerEventKind::CompilerKilled(project_kind)).await;
                if let Err(e) = send_result {
                    error!("Compiler(RUNNING-{:?}): sent: error: {}", project_kind, e);
                }
                break;
            }
            CompilerEventKind::Start(_project_kind) => {
                error!("Compiler(RUNNING-{:?}): recv: error: received: {:?} while the compiller is already running.", project_kind, compiler_event_kind);
            }
        }
    }
}

async fn manager(mut recv_manager_event: mpsc::Receiver<ManagerEventKind>, send_compiler_event: broadcast::Sender<CompilerEventKind>) {
    // let mut back_is_compiling = false;
    // let mut back_is_starting = false;
    // let mut back_is_being_killed = false;
    let mut compiler_state = CompilerState::Ready;
    let mut compile_next: Option<ProjectKind> = None;

    let mut back_is_running = false;

    while let Some(event_kind) = recv_manager_event.recv().await {
        match event_kind {
            ManagerEventKind::File(project_kind) => {
                debug!("Manager: recv: file: {:?}, current compiler state: {:?}", project_kind, compiler_state);
                match project_kind {
                    ProjectKind::Back => {
                        match compiler_state {
                            CompilerState::Compiling(project_kind) => {
                                //debug!("Manager: skipped: compiler is busy compiling");
                                debug!("Manager: sent: compile: {:?}", CompilerEventKind::Kill);
                                let send_result = send_compiler_event.send(CompilerEventKind::Kill);
                                if let Err(e) = send_result {
                                    error!("Manager: sent: error: {}", e);
                                    continue;
                                } else {
                                    debug!("Manager: set: compiling state: Killing({:?})", project_kind);
                                    compiler_state = CompilerState::Killing(project_kind);
                                    debug!("Manager: set: compiling next: Some({:?})", project_kind);
                                    compile_next = Some(project_kind);
                                }
                            },
                            CompilerState::Killing(project_kind) => {
                                debug!("Manager: skipped: compiler is busy killing");
                            },
                            CompilerState::Starting(project_kind) => {
                                debug!("Manager: skipped: compiler is busy starting up");
                            },
                            CompilerState::Ready => {
                                debug!("Manager: sent: compile: {:?}", CompilerEventKind::Start(ProjectKind::Back));
                                let send_result = send_compiler_event.send(CompilerEventKind::Start(ProjectKind::Back));
                                if let Err(e) = send_result {
                                    error!("Manager: sent: error: {}", e);
                                    continue;
                                } else {
                                    debug!("Manager: set: compiling state: Starting({:?})", project_kind);
                                    compiler_state = CompilerState::Starting(project_kind);
                                }
                            }
                        }
                        // if back_is_compiling {
                        //     debug!("Manager: skipped: backend is already compiling");
                        // } else {
                            
                        // }
                    }
                    ProjectKind::Front => {

                    }
                }
                
            },
            ManagerEventKind::CompilerStarted(project_kind) => {
                debug!("Manager: recv: compilation_started: {:?}, current compiler state: {:?}", project_kind, compiler_state);
                match project_kind {
                    ProjectKind::Back => {
                        match compiler_state {
                            CompilerState::Compiling(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Starting, not Compiling({:?})", current_project_kind);
                            }
                            CompilerState::Killing(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Starting, not Killing({:?})", current_project_kind);
                            }
                            CompilerState::Starting(current_project_kind) => {
                                if current_project_kind == project_kind {
                                    debug!("Manager: set: compiler state to Compiling({:?})", project_kind);
                                    compiler_state = CompilerState::Compiling(project_kind);
                                } else {
                                    error!("Manager: sync: error: compiler state Starting project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                                }
                            }
                            CompilerState::Ready => {
                                error!("Manager: sync: error: compiler state suppose to be Starting, not ready");
                            }
                        }
                    },
                    ProjectKind::Front => {
                        // if !back_is_ {
                        //     error!("Manager: recv: error: back_is_compiling suppose to be true");
                        // }
                    }
                }
            },
            ManagerEventKind::CompilerEnded(project_kind) => {
                debug!("Manager: recv: compilation_ended: {:?}", project_kind);
                match project_kind {
                    ProjectKind::Back => {
                        match compiler_state {
                            CompilerState::Compiling(current_project_kind) => {
                                if current_project_kind == project_kind {
                                    debug!("Manager: set: compiler state to Ready");
                                    compiler_state = CompilerState::Ready;
                                } else {
                                    error!("Manager: sync: error: compiler state Compiling project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                                }
                            }
                            CompilerState::Killing(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Compiling, not Killing({:?})", current_project_kind);
                            }
                            CompilerState::Starting(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Compiling, not Starting({:?})", current_project_kind);
                            }
                            CompilerState::Ready => {
                                error!("Manager: sync: error: compiler state suppose to be Compiling, not ready");
                            }
                        }
                    },
                    ProjectKind::Front => {
                        // if !back_is_ {
                        //     error!("Manager: recv: error: back_is_compiling suppose to be true");
                        // }
                    }
                }
            }
            ManagerEventKind::CompilerKilled(project_kind) => {
                debug!("Manager: recv: compilation_ended: {:?}", project_kind);
                match project_kind {
                    ProjectKind::Back => {
                        match compiler_state {
                            CompilerState::Compiling(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Compiling({:?})", project_kind, current_project_kind);
                            }
                            CompilerState::Killing(current_project_kind) => {
                                if current_project_kind == project_kind {
                                    
                                    if let Some(next_project_knd) = compile_next {
                                        debug!("Manager: sent: compile next: {:?}", CompilerEventKind::Start(next_project_knd));
                                        let send_result = send_compiler_event.send(CompilerEventKind::Start(next_project_knd));
                                        if let Err(e) = send_result {
                                            error!("Manager: sent: error: {}", e);
                                            continue;
                                        } else {
                                            debug!("Manager: set: compiling state: Starting({:?})", project_kind);
                                            compiler_state = CompilerState::Starting(project_kind);
                                            debug!("Manager: set: compiling next: None");
                                            compile_next = None;
                                        }
                                    } else {
                                        debug!("Manager: set: compiler state to Ready");
                                        compiler_state = CompilerState::Ready;
                                    }
                                    
                                } else {
                                    error!("Manager: sync: error: compiler state Killing project kind do not match, recv {:?}, current: {:?}", project_kind, current_project_kind);
                                }
                            }
                            CompilerState::Starting(current_project_kind) => {
                                error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not Starting({:?})", project_kind, current_project_kind);
                            }
                            CompilerState::Ready => {
                                error!("Manager: sync: error: compiler state suppose to be Killing({:?}), not ready", project_kind);
                            }
                        }
                    },
                    ProjectKind::Front => {
                        // if !back_is_ {
                        //     error!("Manager: recv: error: back_is_compiling suppose to be true");
                        // }
                    }
                }
            }
        }
    }
}

async fn watch_dir(path: &str, send_manager_event: mpsc::Sender<ManagerEventKind>, event_kind: ProjectKind) {
    debug!("Watcher: watching: {}", path);

    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )
    .unwrap();

    watcher
        .watch(path.as_ref(), RecursiveMode::Recursive)
        .unwrap();

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                if let EventKind::Access(kind) = event.kind {
                    if let AccessKind::Close(kind) = kind {
                        if let AccessMode::Write = kind {
                            debug!("Wachter({}): sent: {:?}",path, event_kind);
                            let send_result = send_manager_event.send(ManagerEventKind::File(event_kind)).await;
                            if let Err(e) = send_result {
                                error!("Watcher: sent: error: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}


async fn build_commands<I>(commands_parts: I) -> Vec<(Command, String)>
where
    I: IntoIterator,
    I::Item: IntoIterator,
    <I::Item as IntoIterator>::Item: AsRef<OsStr> + AsRef<str>,
{
    let mut commands: Vec<(Command, String)> = Vec::new();

    for command_parts in commands_parts {
        let mut command: Option<Command> = None;
        let mut command_str: String = String::new();
        for part in command_parts {
            
            match command.as_mut() {
                Some(command) => {
                    command_str.push_str(part.as_ref());
                    command.arg(part);
                }
                None => {
                    command_str.push_str(part.as_ref());
                    command_str.push(' ');
                    command = Some(Command::new(part));
                }
            }
        }
        if let Some(command) = command {
            commands.push((command, command_str));
        }
    }

    commands
}