#[macro_use]
extern crate lazy_static;
extern crate rayon;
#[macro_use]
extern crate clap;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;
use regex::Regex;
use pwhash::md5_crypt;
use clap::App;



fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn check_hash(word: &str, salt: &str, hash: &str) -> bool {
	let digest = md5_crypt::hash_with(salt, word);
	return digest.unwrap() == hash;
}

fn get_salt(hash: &str) -> String {
	lazy_static! {
		static ref RE: Regex = Regex::new(r"^(\$\d\$\w+)").unwrap();
	}
	return String::from(&RE.captures(hash).unwrap()[0]);
}

fn main() {
	let yaml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yaml).get_matches();

	let h = matches.value_of("hash").unwrap();
	let hash: String = String::from(h);
	let w = matches.value_of("wordlist").unwrap();
	let wordlist: String = String::from(w);
	let t = matches.value_of("threads").unwrap();
	let threads = t.parse::<usize>().unwrap();
	let salt = &get_salt(&hash);

	let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();

	if let Ok(lines) = read_lines(wordlist) {
		for line in lines {
			if let Ok(word) = line {
				pool.install(|| {
					if check_hash(&word, &salt, &hash) {
						println!("found hash: {}", word);
						process::exit(0);
					}
				});
			}
		}
	}
}
