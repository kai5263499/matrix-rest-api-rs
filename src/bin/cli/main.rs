use log::debug;
use env_logger::Env;
use std::{thread, time};
use rand::Rng;
use protobuf::Message;
use matrix_protos_rust::protos::io::{EverloopImage, LedValue};
use matrix_protos_rust::protos::driver::DriverConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "debug")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    debug!("starting up");

    let ctx = zmq::Context::new();

    let error_socket = ctx.socket(zmq::SUB).unwrap();
    error_socket.connect("tcp://127.0.0.1:20023").unwrap();
    error_socket.set_subscribe(b"").unwrap();

    let ping_socket = ctx.socket(zmq::PUSH).unwrap();
    ping_socket.connect("tcp://127.0.0.1:20022").unwrap();
    ping_socket.send("", 0).unwrap();

    let dataupdate_socket = ctx.socket(zmq::SUB).unwrap();
    dataupdate_socket.connect("tcp://127.0.0.1:20024").unwrap();
    dataupdate_socket.set_subscribe(b"").unwrap();

    let config_socket = ctx.socket(zmq::PUSH).unwrap();
    config_socket.connect("tcp://127.0.0.1:20021").unwrap();
    
    let mut msg = zmq::Message::new();
    dataupdate_socket.recv(&mut msg, 0).unwrap();
    let everloop_image: EverloopImage = Message::parse_from_bytes(&msg).unwrap();
    let everloop_length = everloop_image.get_everloop_length();

    let handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();

        loop{ 
            
            let mut leds = Vec::new();
            for _ in 0..everloop_length {
                let mut led = LedValue::new();
                led.set_red(rng.gen_range(0..255));
                led.set_green(rng.gen_range(0..255));
                led.set_blue(rng.gen_range(0..255));
                led.set_white(0);
                leds.push(led);
            }

            let mut everloop_image = EverloopImage::new();
            everloop_image.set_led(protobuf::RepeatedField::from_vec(leds));

            let mut everloopconfig = DriverConfig::new();
            everloopconfig.set_image(everloop_image);

            let config_msg = everloopconfig.write_to_bytes().unwrap();
            config_socket.send(config_msg, 0).unwrap();

            thread::sleep(time::Duration::from_millis(50));
        }
    });

    thread::spawn(move || {
        loop {
            let mut msg = zmq::Message::new();
            error_socket.recv(&mut msg, 0).unwrap();
            println!("error message: {:?}", msg);
        }
    });

    handle.join().unwrap();

    Ok(())
}