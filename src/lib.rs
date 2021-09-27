use bio::{alphabets::dna::revcomp, io::fasta};
use dashmap::DashMap;
use rayon::prelude::*;
use std::{
    cmp::min,
    env,
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    str,
};

pub struct Config {
    pub kmer_len: usize,
    pub filepath: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let kmer_len = match args.next() {
            Some(arg) => arg.parse().unwrap(),
            None => return Err("Problem with k-mer length input"),
        };
        let filepath = args.next().unwrap();

        Ok(Config { kmer_len, filepath })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    //  Get parameters
    //  Fasta filepath
    let filepath: String = config.filepath;
    //  K-mer length k
    let k: usize = config.kmer_len;

    //  Create fasta file reader
    let reader: fasta::Reader<std::io::BufReader<File>> =
        fasta::Reader::from_file(&filepath).unwrap();

    //  Create a DashMap
    let fasta_hash: DashMap<Vec<u8>, usize> = DashMap::new();

    //  Read fasta records into a Dashmap, a hashmap mutably accessible from different parallel processes
    reader
        .records()
        .into_iter()
        .par_bridge()
        .for_each(|result| {
            let seq: &[u8] = result.as_ref().unwrap().seq();

            for i in 0..(seq.len() + 1).saturating_sub(k) {
                *fasta_hash //  Make kmer for output lexicographically min(kmer, reverse-complement)
		    .entry(min(seq[i..i + k].to_vec(), revcomp(&seq[i..i + k])))
		    .or_insert(0) += 1;
            }
        });

    //  PRINTING OUTPUT
    //  Create handle and BufWriter for writing
    let handle = std::io::stdout();

    let mut buf = BufWriter::new(handle);

    fasta_hash.into_iter().for_each(|(k, f)| {
        let kmer: String = String::from_utf8(k).unwrap();
        //  Irradicate kmers containing 'N'
        if kmer.contains('N') {
        } else {
            //  Write:
            //  >frequency across fasta file for both kmer and its reverse complement
            //  k-mer (lexicographically smaller of k-mer, reverse complement pair)
            writeln!(buf, ">{}\n{}", f, kmer).expect("Unable to write data");
        }
    });
    buf.flush().unwrap();

    Ok(())
}
