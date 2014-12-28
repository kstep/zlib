#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate zlib;

use std::io;
use zlib::{WINDOW_BITS_DEFAULT,ZStream};
use zlib::inflate::{InflateState,InflateResult};
use zlib::inflate::InflateReader;
use std::io::IoErrorKind;
use std::io::IoError;

/*
fn main()
{
    let in_bufsize: uint = 0x1000;
    let out_bufsize: uint = 512;

    let input_path = Path::new("tests/hamlet.tar.gz");
    let check_path = Path::new("tests/hamlet.tar");			// contains the expected (good) output

    // open compressed input file
    let input_file = io::BufferedReader::new(io::File::open(&input_path).unwrap());

    // open known-good input file
    let mut check_file = io::BufferedReader::new(io::File::open(&check_path).unwrap());

	println!("successfully opened test files");

    // create an InflateReader over the input file
    let mut inflater = InflateReader::new(in_bufsize, 2, box input_file);

    let mut output_buffer: Vec<u8> = Vec::with_capacity(out_bufsize);
    let mut check_buffer: Vec<u8> = Vec::new();

    let mut total_out: u64 = 0;

    loop {
        match inflater.push(out_bufsize, &mut output_buffer) {
            Ok(output_bytes_written) => {
                debug!("inflate reader returned {} bytes", output_bytes_written);

                // Check the data that we just received against the same data in the known-good file.
                if output_bytes_written != 0 {
                	assert!(check_buffer.len() == 0);
                	let check_bytes_read = check_file.push(output_bytes_written, &mut check_buffer).unwrap();
                	assert!(check_bytes_read == output_bytes_written);
                	for i in range(0, output_bytes_written) {
                		if check_buffer[i] != output_buffer[i] {
                			panic!("outputs differ!  at output offset {}, expected {} found {}", total_out + (i as u64), check_buffer[i], output_buffer[i]);
                		}
                	}

					check_buffer.clear();
                    output_buffer.clear();
                }

                total_out += output_bytes_written as u64;
            }
            Err(_) => {
                println!("push() returned error, assuming EOF for now");
                break;
            }
        }
    }
}
*/

fn main()
{
    let in_bufsize: uint = 0x1000;
    let out_bufsize: uint = 0x1000;

    let input_path = Path::new("tests/hamlet.tar.gz");
    let check_path = Path::new("tests/hamlet.tar");			// contains the expected (good) output

    // open compressed input file
    let mut input_file = io::BufferedReader::new(io::File::open(&input_path).unwrap());

    // open known-good input file
    let mut check_file = io::BufferedReader::new(io::File::open(&check_path).unwrap());

    let mut input_buffer: Vec<u8> = Vec::with_capacity(in_bufsize);
    let mut output_buffer: Vec<u8> = Vec::with_capacity(out_bufsize);
    output_buffer.grow(out_bufsize, 0);
    let mut check_buffer: Vec<u8> = Vec::new();

    let mut input_pos: uint = 0; // index of next byte in input_buffer to read

    let out_data = output_buffer.as_mut_slice();

    let mut strm = ZStream::new();
    let mut state = InflateState::new(WINDOW_BITS_DEFAULT, 2);
    let mut input_eof = false;
    let mut cycle: uint = 0;

    // Main loop
    loop {
        println!("cycle = {}", cycle);
        // println!("decode loop: avail_in = {}, next_in = {}, avail_out = {}, next_out = {}",
        //     strm.avail_in, strm.next_in, strm.avail_out, strm.next_out);

        if input_pos == input_buffer.len() && !input_eof {
            // println!("input buffer is empty; loading data");
            input_buffer.clear();
            input_pos = 0;
            match input_file.push(in_bufsize, &mut input_buffer) {
                Ok(bytes_read) => {
                    println!("zlibtest: loaded {} input bytes", bytes_read);
                }
                Err(err) => {
                    if err.kind == IoErrorKind::EndOfFile {
                        println!("input stream EOF");
                        input_eof = true;
                    }
                    else {
                        println!("unexpected input error: {}", err.desc);
                        break;
                    }
                }
            };
        }

        let total_out: u64 = strm.total_out;

        println!("calling inflate, cycle = {}, input_pos = {}, input_buffer.len = {}", cycle, input_pos, input_buffer.len());
        match state.inflate(&mut strm, None, input_buffer.slice_from(input_pos), out_data) {
            InflateResult::Eof(_) => {
                println!("zlib says Z_STREAM_END");
                break;
            }

            InflateResult::InvalidData => {
                println!("InvalidData");
                break;
            }

            InflateResult::Decoded(input_bytes_read, output_bytes_written) => {
                // println!("InflateDecoded: input_bytes_read: {} output_bytes_written: {}", input_bytes_read, output_bytes_written);                
                // println!("zlibtest: in_read={}, out_written={}", input_bytes_read, output_bytes_written);
                println!("zlibtest: cycle = {}, input_bytes_read = {}, output_bytes_written = {}", cycle, input_bytes_read, output_bytes_written);
                println!("total_in = {}", strm.total_in);
                print_block(out_data.slice(0, output_bytes_written));

                assert!(input_bytes_read + input_pos <= input_buffer.len());
                input_pos += input_bytes_read;

                // Check the data that we just received against the same data in the known-good file.
                if output_bytes_written != 0 {
                	assert!(check_buffer.len() == 0);

                    // read chunks from the check stream and verify them
                    let mut cpos = 0;
                    while cpos < output_bytes_written {
                        let clen_want = output_bytes_written - cpos;
                	    assert!(check_buffer.len() == 0);
                	    let clen_got = check_file.push(clen_want, &mut check_buffer).unwrap();
                        assert!(clen_got <= clen_want);

                	    for i in range(0, clen_got) {
                		    if check_buffer[i] != out_data[cpos + i] {
                			    panic!("outputs differ!  at output offset {}, expected {} found {}",
                                    total_out + (i as u64),
                                    check_buffer[i],
                                    out_data[cpos + i]);
                		    }
                	    }

                        cpos += clen_got;
					    check_buffer.clear();
                    }
                }
            }

            InflateResult::NeedInput => {
                println!("NeedInput");
                unimplemented!();
            }
        }

        cycle += 1;
    }
}

fn print_block(data: &[u8]) {
    let mut s = String::new();

    let width = 32;

    static HEX: [char; 0x10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f' ];

    println!("print_block: len={}", data.len());

    for i in range(0, data.len()) {
        let b = data[i];
        s.push(' ');
        s.push(HEX[(b >> 4) as uint]);
        s.push(HEX[(b & 0xf) as uint]);
        if ((i + 1) % width) == 0 {
            println!("{}", s);
            s.clear();
        }
    }

    if (data.len() % width) != 0 {
        println!("{}", s);
    }
}
