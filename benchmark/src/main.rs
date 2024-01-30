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

use bson::DateTime;
use eframe::Frame;
use egui::Context;
use egui_plot::{Line, Plot, PlotPoints};
use std::cell::Cell;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use artcord::server::client_msg::ClientMsg;
use artcord::server::server_msg::ServerMsg;
use futures_util::{future, pin_mut, SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Instant};
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

enum PlotType {
    Sin,
    Bell,
    Sigmoid,
}

struct MyApp {
    name: String,
    age: u32,
    rt: Arc<tokio::runtime::Runtime>,
    plot: PlotType,
    url: Arc<Url>,
    index: Arc<RwLock<f64>>,
    points1: Arc<RwLock<Vec<[f64; 2]>>>,
    //line1: Arc<RwLock<Line>>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            rt: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(24)
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
            plot: PlotType::Bell,
            url: Arc::new(url::Url::parse("ws://localhost:3000/ws/").unwrap()),
            index: Arc::new(RwLock::new(0.0)),
            points1: Arc::new(RwLock::new(Vec::new())),
            //line1: Arc::new(RwLock::new(Line::new(PlotPoints::new(vec![])))),
        }
    }
}

fn gaussian(x: f64) -> f64 {
    let var: f64 = 2.0;
    f64::exp(-(x / var).powi(2)) / (var * f64::sqrt(std::f64::consts::TAU))
}

fn sigmoid(x: f64) -> f64 {
    -1.0 + 2.0 / (1.0 + f64::exp(-x))
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let n = 128;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WOW");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            if ui.button("hello click me").clicked() {
                println!("wow ok?");
            }
            let points = (0..=n)
                .map(|i| {
                    use std::f64::consts::TAU;
                    let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
                    match self.plot {
                        PlotType::Sin => [x, x.sin()],
                        PlotType::Bell => [x, 10.0 * gaussian(x)],
                        PlotType::Sigmoid => [x, sigmoid(x)],
                    }
                })
                .collect::<PlotPoints>();

            let width = ui.available_width();
            let height = ui.available_height();
            {
                //let points1 = self.points1.read().unwrap();
                let line1 = Line::new(PlotPoints::new((*self.points1.read().unwrap()).clone()));
                egui_plot::Plot::new("test")
                    .width(width)
                    .height(if height - 150.0 > 150.0 {
                        height - 150.0
                    } else {
                        512.0
                    })
                    .show(ui, |plot_ui| {
                        plot_ui.line(line1);
                        //plot_ui.line(line2);
                    })
                    .response;
            }
            // let points = PlotPoints::new(vec![[0.0, 0.0], [5.0, 5.0]]);
            // let line = Line::new(points);
            //
            // let points = PlotPoints::new(vec![[0.0, 1.0], [4.0, 4.0]]);
            // let line2 = Line::new(points);

            //let height = ui.available_height();

            if ui.button("SIMULATE TRAFFIC").clicked() {
                let url = self.url.clone();
                let points1 = self.points1.clone();
                let index = self.index.clone();
                let pp_count = 1000;
                let pp_time = 10;
                let interval = 1;

                let rt = self.rt.clone();

                rt.clone().spawn(async move {
                    let url = url.clone();
                    let points1 = points1.clone();
                    let index = index.clone();
                    let rt = rt.clone();

                    for i in 0..100 {
                        let url = url.clone();
                        let points1 = points1.clone();
                        let index = index.clone();

                        rt.spawn(async move {
                            //println!("yo yo yo dog");

                            let t = Instant::now();
                            let mut conns: Vec<(
                                WebSocketStream<MaybeTlsStream<TcpStream>>,
                                Response,
                            )> = Vec::with_capacity(pp_count);
                            for _ in 0..pp_count {
                                let wow = connect_async(&*url).await.expect("Failed to connect");
                                conns.push(wow);
                            }

                            let elapsed = t.elapsed().as_millis();
                            {
                                let mut points1 = points1.write().unwrap();
                                let mut index = index.write().unwrap();
                                points1.push([*index, elapsed.clone() as f64]);
                                *index += 1.0;
                            }

                            println!("CONNECTING {}ms", elapsed);

                            sleep(Duration::from_secs(pp_time)).await;

                            let t = Instant::now();
                            for mut ws_stream in conns {
                                ws_stream.0.close(None).await.unwrap();
                            }

                            let elapsed = t.elapsed();
                            println!("CLOSING {}ms", elapsed.as_millis());

                            // {
                            //     let mut index = index.write().unwrap();
                            //     *index += 1.0;
                            // }
                        });

                        sleep(Duration::from_secs(interval)).await;
                    }
                });
            }

            if ui.button("SIMULATE TRAFFIC2").clicked() {
                let url = self.url.clone();
                let points1 = self.points1.clone();
                let index = self.index.clone();
                let pp_count = 1000000;
                let pp_time = 10;
                let interval = 1;

                let rt = self.rt.clone();

                rt.clone().spawn(async move {
                    let url = url.clone();
                    let points1 = points1.clone();
                    let index = index.clone();
                    let rt = rt.clone();

                    let mut conns: Vec<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response)> =
                        Vec::with_capacity(pp_count);
                    for _ in 0..pp_count {
                        let wow = connect_async(&*url).await.expect("Failed to connect");
                        conns.push(wow);
                    }
                    //sleep(Duration::from_secs(pp_time)).await;
                    for mut ws_stream in conns {
                        ws_stream.0.close(None).await.unwrap();
                    }
                });
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My DDD",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )

    // let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    // tokio::spawn(read_stdin(stdin_tx));
    //
    //
    //
    // let msg = ClientMsg::GalleryInit {
    //     from: DateTime::now(),
    //     amount: 500,
    // };
    // let bytes = msg.as_vec().unwrap();
    //
    // let (mut write, read) = ws_stream.split();
    //
    // let mmm = Message::binary(bytes);
    // write.send(mmm).await.unwrap();
    //
    // let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    // let ws_to_stdout = {
    //     read.for_each(|message| async {
    //         let data = message.unwrap().into_data();
    //         tokio::io::stdout().write_all(&data).await.unwrap();
    //     })
    // };
    //
    // pin_mut!(stdin_to_ws, ws_to_stdout);
    // future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
