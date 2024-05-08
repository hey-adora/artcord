//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.

use artcord_state::message::prod_client_msg::ClientMsg;
use bson::DateTime;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use dotenv::dotenv;
use egui::Context;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{block, Block, Gauge, LineGauge, List, ListItem};
use ratatui::{symbols, Terminal, Viewport};
use std::cell::Cell;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{env, thread};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, Level};

use futures_util::{future, pin_mut, SinkExt, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ratatui::prelude::Line;
use ratatui::TerminalOptions;
use std::io;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Instant};
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tokio_util::task::TaskTracker;
use url::Url;

enum PlotType {
    Sin,
    Bell,
    Sigmoid,
}

enum Master {
    Con,
    Disc,
    Spawn,
    CloseStarted,
    CloseCompleted,
    Resize,
}

struct DrawData {
    spawned: u64,
    connected: u64,
    disconnected: u64,
    max: u64,
}

impl Default for DrawData {
    fn default() -> Self {
        Self {
            spawned: 0,
            connected: 0,
            disconnected: 0,
            max: 200,
        }
    }
}

impl DrawData {
    pub fn new() -> Self {
        Self::default()
    }
}

fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    crossterm::terminal::enable_raw_mode().unwrap();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )
    .unwrap();

    // crossterm::terminal::enable_raw_mode()?;
    // let stdout = io::stdout();
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::with_options(
    //     backend,
    //     TerminalOptions {
    //         viewport: Viewport::Inline(8),
    //     },
    // )?;

    // let m = MultiProgress::new();
    // let sty = ProgressStyle::with_template(
    //     "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    // )
    // .unwrap()
    // .progress_chars("##-");

    // let pb_spawn = m.add(ProgressBar::new(100));
    // pb_spawn.set_style(sty.clone());
    // pb_spawn.set_message("spawn");
    // let pb_con = m.add(ProgressBar::new(100));
    // pb_con.set_style(sty.clone());
    // pb_con.set_message("cons");
    // let pb_dis = m.add(ProgressBar::new(100));
    // pb_dis.set_style(sty.clone());
    // pb_dis.set_message("disc");
    // let pb_total = m.add(ProgressBar::new(100));
    // pb_total.set_style(sty.clone());
    // pb_total.set_message("total");

    let cancelation_token = CancellationToken::new();
    let (master_tx, mut master_rx) = mpsc::channel::<Master>(1000);
    let runtime = Runtime::new().unwrap();

    runtime.spawn(controller_task(
        master_tx.clone(),
        cancelation_token.clone(),
    ));

    let input_thread = thread::spawn(move || input_thread(master_tx));

    let mut draw_data = DrawData::new();
    let mut redraw = true;

    //let mut ttt = 0;

    loop {
        if redraw {
            terminal
                .draw(|f: &mut ratatui::prelude::Frame| draw(f, &mut draw_data))
                .unwrap();
            redraw = false;
        }

        let msg = master_rx.blocking_recv();
        let Some(msg) = msg else {
            break;
        };
        match msg {
            Master::Con => {
                //pb_con.inc(1);
                draw_data.connected += 1;
                redraw = true;
            }
            Master::Disc => {
                draw_data.disconnected += 1;
                //pb_dis.inc(1);
                redraw = true;
            }
            Master::Spawn => {
                //pb_spawn.inc(1);
                //ttt += 1;
                draw_data.spawned += 1;
                redraw = true;
            }
            Master::Resize => {
                terminal.autoresize().unwrap();
                redraw = true;
            }
            Master::CloseStarted => {
                println!("\n\n");
                cancelation_token.cancel();
            }
            Master::CloseCompleted => {
                break;
            }
        }
    }

    input_thread.join().unwrap();

    //pb_con.finish_with_message("done");

    //m.clear().unwrap();
    //println!("end: {}", ttt);
}

fn draw(f: &mut ratatui::prelude::Frame, data: &mut DrawData) {
    let area = f.size();
    let block = Block::new().title(block::Title::from("Progress").alignment(Alignment::Center));
    f.render_widget(block, area);

    let vertical = Layout::vertical([Constraint::Length(2), Constraint::Length(4)]).margin(1);
    let horizontal = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]);
    let [progress_area, main] = vertical.areas(area);
    let [list_area, gauge_area] = horizontal.areas(main);

    // let progress = LineGauge::default()
    //     .gauge_style(Style::default().fg(Color::Blue))
    //     .label(format!("{}/{}", data.connected, data.max))
    //     .ratio((data.max as f64 / data.connected as f64).clamp(0.0, 1.0));
    // f.render_widget(progress, progress_area);

    // format!(" connected {}/{}", data.connected, data.max)

    let create_item = |text: String| {
        ListItem::new(Line::from(vec![
            Span::raw(symbols::DOT),
            Span::styled(
                text,
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
    };

    let items: Vec<ListItem> = vec![
        create_item(format!(" spawned {}/{}", data.spawned, data.max)),
        create_item(format!(" connected {}/{}", data.connected, data.max)),
        create_item(format!(" disconnected {}/{}", data.disconnected, data.max)),
    ];
    let list = List::new(items);
    f.render_widget(list, list_area);

    let mut render_gauge = |i: u16, ratio: f64| {
        let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(ratio);
        f.render_widget(gauge, Rect {
            x: gauge_area.left(),
            y: gauge_area.top().saturating_add(i),
            width: gauge_area.width,
            height: 1,
        });
    };

    render_gauge(0, (data.spawned as f64 / data.max as f64).clamp(0.0, 1.0));
    render_gauge(1, (data.connected as f64 / data.max as f64).clamp(0.0, 1.0));
    render_gauge(2, (data.disconnected as f64 / data.max as f64).clamp(0.0, 1.0));
}

async fn controller_task(master_tx: mpsc::Sender<Master>, cancelation_token: CancellationToken) {
    //info!("controller task started");
    let task_tracker = TaskTracker::new();

    for i in 0..200 {
        task_tracker.spawn(node(master_tx.clone(), cancelation_token.clone()));
    }

    //info!("waiting for close signal...");
    // let close = signal::ctrl_c().await;
    // if let Err(close) = close {
    //     error!("closing: {}", close);
    // }
    cancelation_token.cancelled().await;
    info!("closing....");
    task_tracker.close();
    task_tracker.wait().await;
    info!("runtime closed");
    master_tx.send(Master::CloseCompleted).await.unwrap();
}

fn input_thread(master_tx: mpsc::Sender<Master>) {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout).unwrap() {
            let event = crossterm::event::read().unwrap();
            match event {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers == KeyModifiers::CONTROL
                    {
                        //info!("cancel event sent");
                        master_tx.blocking_send(Master::CloseStarted).unwrap();
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    master_tx.blocking_send(Master::Resize).unwrap();
                }
                _ => {}
            }
            //debug!("event: {:?}", event);
        }
        if last_tick.elapsed() >= tick_rate {
            //master_tx.send(Tick)
            last_tick = Instant::now();
        }
    }
}

async fn node(master_tx: mpsc::Sender<Master>, cancellation_token: CancellationToken) {
    master_tx.send(Master::Spawn).await.unwrap();

    let url = url::Url::parse("ws://localhost:3420").unwrap();
    let con = connect_async(url).await;
    let Ok(con) = con else {
        master_tx.send(Master::Disc).await.unwrap();
        return;
    };
    let (ws_stream, res) = con;

    let (write, mut read) = ws_stream.split();
    let (send_tx, mut recv_tx) = mpsc::channel::<ClientMsg>(1);

    master_tx.send(Master::Con).await.unwrap();

    loop {
        select! {
            msg = read.next() => {
                let Some(msg) = msg else {
                    break;
                };
            }
            _ = cancellation_token.cancelled() => {
                break;
            }
            msg = recv_tx.recv() => {
                let Some(msg) = msg else {
                    break;
                };
                let result = on_client_msg(msg).await;
                if let Err(err) = result {
                    error!("on client msg failed: {}", err);
                }
            }
        }
    }

    master_tx.send(Master::Disc).await.unwrap();
}

async fn on_client_msg(msg: ClientMsg) -> Result<(), OnClientMsgError> {
    let bytes = ClientMsg::as_vec(&(0, msg))?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum OnClientMsgError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

// struct MyApp {
//     name: String,
//     age: u32,
//     rt: Arc<tokio::runtime::Runtime>,
//     plot: PlotType,
//     url: Arc<Url>,
//     index: Arc<RwLock<f64>>,
//     points1: Arc<RwLock<Vec<[f64; 2]>>>,
//     //line1: Arc<RwLock<Line>>,
// }

// impl Default for MyApp {
//     fn default() -> Self {
//         Self {
//             name: "Arthur".to_owned(),
//             age: 42,
//             rt: Arc::new(
//                 tokio::runtime::Builder::new_multi_thread()
//                     .worker_threads(24)
//                     .enable_all()
//                     .build()
//                     .unwrap(),
//             ),
//             plot: PlotType::Bell,
//             url: Arc::new(url::Url::parse("ws://localhost:3000").unwrap()),
//             index: Arc::new(RwLock::new(0.0)),
//             points1: Arc::new(RwLock::new(Vec::new())),
//             //line1: Arc::new(RwLock::new(Line::new(PlotPoints::new(vec![])))),
//         }
//     }
// }

// fn gaussian(x: f64) -> f64 {
//     let var: f64 = 2.0;
//     f64::exp(-(x / var).powi(2)) / (var * f64::sqrt(std::f64::consts::TAU))
// }

// fn sigmoid(x: f64) -> f64 {
//     -1.0 + 2.0 / (1.0 + f64::exp(-x))
// }

// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &Context, frame: &mut Frame) {
//         let n = 128;
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("WOW");
//             ui.horizontal(|ui| {
//                 let name_label = ui.label("Your name: ");
//                 ui.text_edit_singleline(&mut self.name)
//                     .labelled_by(name_label.id);
//             });
//             if ui.button("hello click me").clicked() {
//                 println!("wow ok?");
//             }
//             let points = (0..=n)
//                 .map(|i| {
//                     use std::f64::consts::TAU;
//                     let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
//                     match self.plot {
//                         PlotType::Sin => [x, x.sin()],
//                         PlotType::Bell => [x, 10.0 * gaussian(x)],
//                         PlotType::Sigmoid => [x, sigmoid(x)],
//                     }
//                 })
//                 .collect::<PlotPoints>();

//             let width = ui.available_width();
//             let height = ui.available_height();
//             {
//                 //let points1 = self.points1.read().unwrap();
//                 let line1 = Line::new(PlotPoints::new((*self.points1.read().unwrap()).clone()));
//                 egui_plot::Plot::new("test")
//                     .width(width)
//                     .height(if height - 150.0 > 150.0 {
//                         height - 150.0
//                     } else {
//                         512.0
//                     })
//                     .show(ui, |plot_ui| {
//                         plot_ui.line(line1);
//                         //plot_ui.line(line2);
//                     })
//                     .response;
//             }
//             // let points = PlotPoints::new(vec![[0.0, 0.0], [5.0, 5.0]]);
//             // let line = Line::new(points);
//             //
//             // let points = PlotPoints::new(vec![[0.0, 1.0], [4.0, 4.0]]);
//             // let line2 = Line::new(points);

//             //let height = ui.available_height();

//             if ui.button("SIMULATE TRAFFIC").clicked() {
//                 let url = self.url.clone();
//                 let points1 = self.points1.clone();
//                 let index = self.index.clone();
//                 let pp_count = 10;
//                 let pp_time = 10;
//                 let interval = 1;

//                 let rt = self.rt.clone();

//                 rt.clone().spawn(async move {
//                     let url = url.clone();
//                     let points1 = points1.clone();
//                     let index = index.clone();
//                     let rt = rt.clone();

//                     for i in 0..100 {
//                         let url = url.clone();
//                         let points1 = points1.clone();
//                         let index = index.clone();

//                         rt.spawn(async move {
//                             //println!("yo yo yo dog");

//                             let t = Instant::now();
//                             let mut conns: Vec<(
//                                 WebSocketStream<MaybeTlsStream<TcpStream>>,
//                                 Response,
//                             )> = Vec::with_capacity(pp_count);
//                             for _ in 0..pp_count {
//                                 let wow = connect_async(&*url).await.expect("Failed to connect");
//                                 conns.push(wow);
//                             }

//                             let elapsed = t.elapsed().as_millis();
//                             {
//                                 let mut points1 = points1.write().unwrap();
//                                 let mut index = index.write().unwrap();
//                                 points1.push([*index, elapsed.clone() as f64]);
//                                 *index += 1.0;
//                             }

//                             println!("CONNECTING {}ms", elapsed);

//                             sleep(Duration::from_secs(pp_time)).await;

//                             let t = Instant::now();
//                             for mut ws_stream in conns {
//                                 ws_stream.0.close(None).await.unwrap();
//                             }

//                             let elapsed = t.elapsed();
//                             println!("CLOSING {}ms", elapsed.as_millis());

//                             // {
//                             //     let mut index = index.write().unwrap();
//                             //     *index += 1.0;
//                             // }
//                         });

//                         sleep(Duration::from_secs(interval)).await;
//                     }
//                 });
//             }

//             if ui.button("SIMULATE TRAFFIC2").clicked() {
//                 let url = self.url.clone();
//                 let points1 = self.points1.clone();
//                 let index = self.index.clone();
//                 let pp_count = 1000000;
//                 let pp_time = 10;
//                 let interval = 1;

//                 let rt = self.rt.clone();

//                 rt.clone().spawn(async move {
//                     let url = url.clone();
//                     let points1 = points1.clone();
//                     let index = index.clone();
//                     let rt = rt.clone();

//                     let mut conns: Vec<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)> =
//                         Vec::with_capacity(pp_count);
//                     for _ in 0..pp_count {
//                         let wow = connect_async(&*url).await.expect("Failed to connect");
//                         conns.push(wow);
//                     }
//                     //sleep(Duration::from_secs(pp_time)).await;
//                     for mut ws_stream in conns {
//                         ws_stream.0.close(None).await.unwrap();
//                     }
//                 });
//             }
//         });
//     }
//}

// fn main() -> Result<(), eframe::Error> {
//     // env_logger::init();
//     // let options = eframe::NativeOptions {
//     //     viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 720.0]),
//     //     ..Default::default()
//     // };

//     // eframe::run_native(
//     //     "My DDD",
//     //     options,
//     //     Box::new(|cc| {
//     //         egui_extras::install_image_loaders(&cc.egui_ctx);
//     //         Box::<MyApp>::default()
//     //     }),
//     // )

//     // let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
//     // tokio::spawn(read_stdin(stdin_tx));
//     //
//     //
//     //
//     // let msg = ClientMsg::GalleryInit {
//     //     from: DateTime::now(),
//     //     amount: 500,
//     // };
//     // let bytes = msg.as_vec().unwrap();
//     //
//     // let (mut write, read) = ws_stream.split();
//     //
//     // let mmm = Message::binary(bytes);
//     // write.send(mmm).await.unwrap();
//     //
//     // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
//     // let ws_to_stdout = {
//     //     read.for_each(|message| async {
//     //         let data = message.unwrap().into_data();
//     //         tokio::io::stdout().write_all(&data).await.unwrap();
//     //     })
//     // };
//     //
//     // pin_mut!(stdin_to_ws, ws_to_stdout);
//     // future::select(stdin_to_ws, ws_to_stdout).await;
// }

// Our helper method which will read data from stdin and send it along the
// sender provided.
// async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
//     let mut stdin = tokio::io::stdin();
//     loop {
//         let mut buf = vec![0; 1024];
//         let n = match stdin.read(&mut buf).await {
//             Err(_) | Ok(0) => break,
//             Ok(n) => n,
//         };
//         buf.truncate(n);
//         tx.unbounded_send(Message::binary(buf)).unwrap();
//     }
// }
