#![allow(dead_code)]
#![feature(plugin)]

#[plugin]
#[macro_use]
extern crate log;

use std::default::Default;
use std::io;
use inftrees::{Code, LENS, DISTS, inflate_table};

/*
   Write out the inffixed.h that is #include'd above.  Defining MAKEFIXED also
   defines BUILDFIXED, so the tables are built on the fly.  makefixed() writes
   those tables to stdout, which would be piped to inffixed.h.  A small program
   can simply call makefixed to do this:

    void makefixed(void);

    int main(void)
    {
        makefixed();
        return 0;
    }

   Then that can be linked with zlib built with MAKEFIXED defined and run:

    a.out > inffixed.h
 */

fn makefixed(w: &mut Writer) {
    let mut fixed: [Code; 544] = [Default::default(); 544];
    let mut work: [u16; 288] = [Default::default(); 288];         // work area for code table building

    // build fixed huffman tables
    let mut lens: [u16; 320] = [Default::default(); 320];         // temporary storage for code lengths

    /* literal/length table */
    {
        let mut sym :usize = 0;
        while sym < 144 { lens[sym] = 8; sym += 1; }
        while sym < 256 { lens[sym] = 9; sym += 1; }
        while sym < 280 { lens[sym] = 7; sym += 1; }
        while sym < 288 { lens[sym] = 8; sym += 1; }
    }

    let mut next :usize = 0;     // index into 'fixed' table
    let lenfix: usize = 0;       // index into 'fixed' table
    let (err, _) = inflate_table(LENS, &lens, 288, &mut fixed, &mut next, 9, work.as_mut_slice());
    assert!(err == 0);

    /* distance table */
    {
        let mut sym :usize = 0;
        while sym < 32 { lens[sym] = 5; sym += 1; }
    }
    let distfix: usize = next;      // index into 'fixed' table

    let (err, _) = inflate_table(DISTS, &lens, 32, &mut fixed, &mut next, 5, work.as_mut_slice());
    assert!(err == 0);

    let lencode = fixed.slice_from(lenfix);
    // let lenbits: usize = 9;
    let distcode = fixed.slice_from(distfix);
    // let distbits: usize = 5;

    w.write_str("
// WARNING -- GENERATED CODE -- DO NOT EDIT
//
// This file contains the generated \"fixed\" tables for zlib.
// It is generated by build.rs.  DO NOT EDIT THIS FILE.
// Instead, edit build.rs.

use super::inftrees::Code;

").unwrap();

    let size = 1 << 9;
    w.write_str(format!("pub static LENFIX: [Code; {}] = [\n", size).as_slice()).unwrap();
    for low in range(0, size) {
        w.write_str(format!("    Code {{ op: 0x{:02x}, bits: {:2}, val: 0x{:04x} }},\n", 
            if (low & 127) == 99 { 64 } else { lencode[low].op },
                lencode[low].bits,
                lencode[low].val).as_slice()).unwrap();
    }
    w.write_str("];\n\n").unwrap();

    let size = 1 << 5;
    w.write_str(format!("pub static DISTFIX: [Code; {}] = [\n", size).as_slice()).unwrap();
    for low in range(0, size) {
        w.write_str(format!("    Code {{ op: 0x{:02x}, bits: {:2}, val: 0x{:04x} }},\n",
            distcode[low].op,
            distcode[low].bits,
            distcode[low].val).as_slice()).unwrap();
    }
    w.write_str("];\n").unwrap();
}

// Return state with length and distance decoding tables and index sizes set to
// fixed code decoding.  Normally this returns fixed tables from inffixed.h.
// If BUILDFIXED is defined, then instead this routine builds the tables the
// first time it's called, and returns those tables the first time and
// thereafter.  This reduces the size of the code by about 2K bytes, in
// exchange for a little execution time.  However, BUILDFIXED should not be
// used for threaded applications, since the rewriting of the tables and virgin
// may not be thread-safe.

#[path = "src/inflate/inftrees.rs"]
mod inftrees;

fn main() {
    let gen_path = Path::new("src/inflate/inffixed.rs");
    let mut gen_file = io::File::create(&gen_path);
    makefixed(&mut gen_file);
}
