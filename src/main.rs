use argparse::{ArgumentParser, Store, StoreTrue};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const BUFSIZE: usize = 1024;
fn main() -> std::io::Result<()> {
    //Init argument parser options.
    let mut server = false;
    let mut client = false;
    let mut receive = false;
    let mut send = false;
    //Init argument parser arguments.
    let mut addr = String::from("0.0.0.0:4444");
    let mut filepath = String::new();
    {
        //Setup CLI argument parser.
        let mut ap = ArgumentParser::new();
        ap.set_description("Bonjour");
        ap.refer(&mut server)
            .add_option(&["-h", "--host"], StoreTrue, "host");
        ap.refer(&mut client)
            .add_option(&["-c", "--client"], StoreTrue, "client");

        ap.refer(&mut receive)
            .add_option(&["-r", "--receive"], StoreTrue, "receive file");
        ap.refer(&mut send)
            .add_option(&["-s", "--send"], StoreTrue, "send file");

        ap.refer(&mut addr).add_argument("addr", Store, "address");
        ap.refer(&mut filepath)
            .add_argument("file", Store, "file path");
        ap.parse_args_or_exit();
    }
    //Handle user input errors.
    if send == receive {
        println!("Please indicate send (-s) or receive (-r).");
        return Ok(());
    }

    if client == server {
        println!("Please indicate host (-h) or client (-c).");
        return Ok(());
    }
    //Initialize file according to transfer type (sender or receiver).
    let mut file: File = match receive {
        true => File::create(filepath)?,
        false => File::open(filepath)?,
    };

    //Initialize TCP stream according to computer role (client or host).
    let mut stream: TcpStream = match server {
        true => {
            //Host TCP stream initialization.
            println!("Listening for clients...");
            let listener = TcpListener::bind(addr)?;
            listener.accept()?.0
        }
        false => {
            //Client TCP stream initialization.
            println!("Connecting to host...");
            TcpStream::connect(addr)?
        }
    };
    println!("Connection etablished!");
    let size: usize;
    if send {
        //Get file size.
        size = file.metadata()?.len() as usize;
        //Send receiver file size.
        stream.write(&size.to_be_bytes())?;
    } else {
        let mut buf = [0; 8];
        //Receive file size.
        stream.read(&mut buf)?;
        size = usize::from_be_bytes(buf);
    };
    println!("Starting sending file ({} b).", size);
    copy(&mut stream, &mut file, size)?;

    println!("File transfert ended successfully!");
    Ok(())
}

fn copy(src: &mut impl Read, dst: &mut impl Write, size: usize) -> std::io::Result<()> {
    //Copy data from a reader (src) to a writer (dst).
    let mut buf;
    let mut n = 0;
    //Stop when the whole file has been sent.
    while n < size - BUFSIZE {
        buf = [0; BUFSIZE];
        n += src.read(&mut buf)?;

        dst.write(&buf)?;
    }
    //copy end of file (to avoid empty bytes at the end of the buffer).
    let mut buf: Vec<u8> = vec![];
    src.read_to_end(&mut buf)?;
    dst.write_all(&mut buf)?;
    Ok(())
}
