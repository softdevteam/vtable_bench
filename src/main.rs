use std::collections::HashMap;
use std::env;
use std::fs::read_dir;
use std::io::{self, stderr, Write};
use std::path::Path;
use std::process::{self, Command};

use getopts::Options;
use rand::seq::SliceRandom;
use rand::thread_rng;
use statistical::{mean, standard_deviation};

const REPS_DEFAULT: usize = 30;
const ITERS_DEFAULT: usize = 100;
const VEC_SIZE_DEFAULT: usize = 10000000;


fn mean_ci(d: &Vec<f64>) -> (f64, f64) {
    let m = mean(d);
    let sd = standard_deviation(d, None);
    // Calculate a 99% confidence based on the mean and standard deviation.
    (m, 2.58 * (sd / (d.len() as f64).sqrt()))
}

fn usage(prog: &str, msg: &str) -> ! {
    let path = Path::new(prog);
    let leaf = match path.file_name() {
        Some(m) => m.to_str().unwrap(),
        None => "vtable_bench"
    };
    if !msg.is_empty() {
        writeln!(&mut stderr(), "{}", msg).ok();
    }
    writeln!(
        &mut stderr(),
        "Usage: {} [-h] [<#reps> <#iters> <#vec size>]",
        leaf
    )
    .ok();
    process::exit(1)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { usage(prog, f.to_string().as_str()) }
    };
    if matches.opt_present("h") {
        usage(&prog, "");
    }
    let (reps, iters, vec_size) = if matches.free.is_empty() {
        (REPS_DEFAULT, ITERS_DEFAULT, VEC_SIZE_DEFAULT)
    } else if matches.free.len() == 3 {
        (matches.free[0].parse().unwrap(), matches.free[1].parse().unwrap(), matches.free[2].parse().unwrap())
    } else {
        usage(&prog, "");
    };

    let bmark_names = read_dir("src/bin/")
        .unwrap()
        .map(|x| {
            x.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    let mut bmark_data: HashMap<_, Vec<f64>> = HashMap::new();
    for bn in &bmark_names {
        bmark_data.insert(bn, vec![]);
    }
    let mut rng = thread_rng();
    let mut done = 0;
    while done < bmark_names.len() * reps {
        // Randomly select a benchmark to run next
        let bn = loop {
            let cnd = bmark_names.choose(&mut rng).unwrap();
            if bmark_data[cnd].len() < reps {
                break cnd;
            }
        };

        let output = Command::new(format!("target/release/{}", bn))
            .args(&[iters.to_string(), vec_size.to_string()])
            .output()
            .expect(&format!("Couldn't run {}", bn));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let t = stdout.trim().parse::<f64>().unwrap();
        bmark_data.get_mut(&bn).unwrap().push(t);
        done += 1;
        print!(".");
        io::stdout().flush().ok();
    }
    println!();
    let mut bmark_names_sorted = bmark_names.clone();
    bmark_names_sorted.sort();
    for bn in &bmark_names_sorted {
        let (mean, ci) = mean_ci(&bmark_data[&bn]);
        println!("{}: {:.3} +/- {:.4}", bn, mean, ci);
    }
}
