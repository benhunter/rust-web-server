use std::{fs, thread};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use rust_web_server::ThreadPool;

fn main() -> std::io::Result<()> {
    let server = TcpListener::bind("127.0.0.1:8080")?;
    let mut pool = ThreadPool::new(4);
    for stream in server.incoming().take(6) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle(stream)
        });
        pool.updated_job_count();
    }
    println!("Final job count {}", pool.job_count);
    Ok(())
}

fn handle(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    println!("Request: {:#?}", request);

    if request.len() == 0 {
        return;
    }

    let request_line = &request[0];

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::TcpStream;
    use std::thread::spawn;

    use crate::main;

    #[test]
    fn test_listens_for_tcp_connections() {
        let main = spawn(main);
        for i in 0..6 {
            let stream = TcpStream::connect("127.0.0.1:8080");
            assert!(stream.is_ok());

            stream.unwrap().write_fmt(format_args!("Request {}", i)).expect("write should succeed");
        }
        main.join().expect("main should join").expect("main should exit");
    }
}
