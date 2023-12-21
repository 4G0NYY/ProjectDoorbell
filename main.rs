use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::Sink;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use dotenv::dotenv;
use std::env;

fn main() {

    dotenv().ok();

    //import creds from .env (If I were to remove .env I'd have to comment these (should remember but who knows, perhaps my descent into total and utter insanity might go to fast?))
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set in .env");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD not set in .env");
    let email_to = env::var("EMAIL_TO").expect("EMAIL_TO not (correctly) set in .env")

    // If I decide I'm too lazy to use a .env, for example cause IT'S ONLY FOR ONE PERSON AND I'M USING A DEAD MAILBOX EITHER WAY
    //let smtp_username = "your_email@gmail.com";
    //let smtp_password = "your_email_password";
    //let email_to = "recipient@example.com";

    //For audio
    let alarm_sound_path = "path/to/alarm.wav";

    //dunno how loud it might be, might have to change that around Marco
    let noise_threshold = 0.5;

    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No input device available");
    let config = input_device.default_input_config().expect("Failed to get default input config");
    let sample_rate = config.sample_rate().0 as f32;
    let channels = config.channels() as usize;

    let detected_flag = Arc::new(Mutex::new(false));

    let callback_data = detected_flag.clone();
    let stream = input_device
        .build_input_stream(
            &config.into(),
            move |data, _| process_input_data(data, sample_rate, channels, &callback_data),
            move |err| eprintln!("Error: {}", err),
        )
        .expect("Failed to build input stream");

    println!("Listening for the specified noise...");

    let email_thread = thread::spawn(move || {
        while !*detected_flag.lock().unwrap() {
            thread::sleep(Duration::from_millis(100));
        }

        send_email(smtp_username, smtp_password, email_to);
    });

    stream.play().expect("Failed to play stream");

    play_alarm(alarm_sound_path);

    email_thread.join().unwrap();
}

fn process_input_data(data: &[f32], sample_rate: f32, _channels: usize, detected_flag: &Arc<Mutex<bool>>) {
    //some weird shit to detect noise, Idek wtf I'm doing anymore, it's 4 in the morning and I need to go to work soon yayyyyy (finna kms ong)
    let rms: f32 = data.iter().map(|&x| x * x).sum::<f32>().sqrt() / data.len() as f32;

    //check threshold
    if rms > noise_threshold {
        *detected_flag.lock().unwrap() = true;
    }
}

fn send_email(username: &str, password: &str, to: &str) {
    let email = Message::builder()
        .from(username.parse().unwrap())
        .to(to.parse().unwrap())
        .subject("Noise Detected!")
        .body("The specified noise has been detected.")
        .expect("Failed to build email");

    let creds = Credentials::new(username.to_string(), password.to_string());

    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => eprintln!("Failed to send email: {}", e),
    }
}

fn play_alarm(alarm_sound_path: &str) {
    let alarm_sink = Sink::new(&rodio::default_output_device().unwrap());
    let alarm_file = std::fs::File::open(alarm_sound_path).expect("Failed to open alarm sound file");
    let alarm_source = rodio::Decoder::new(std::io::BufReader::new(alarm_file)).unwrap();

    alarm_sink.append(alarm_source);
    alarm_sink.sleep_until_end();
}
