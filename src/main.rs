use std::collections::HashMap;
use std::fs::read_dir;
use std::io::{self, Write};
use std::process::Command;

use rand::seq::SliceRandom;
use rand::thread_rng;
use statistical::{mean, standard_deviation};

const REPS: usize = 30;

fn mean_ci(d: &Vec<f64>) -> (f64, f64) {
    let m = mean(d);
    let sd = standard_deviation(d, None);
    // Calculate a 99% confidence based on the mean and standard deviation.
    (m, 2.58 * (sd / (d.len() as f64).sqrt()))
}

fn main() {
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
    while done < bmark_names.len() * REPS {
        // Randomly select a benchmark to run next
        let bn = loop {
            let cnd = bmark_names.choose(&mut rng).unwrap();
            if bmark_data[cnd].len() < REPS {
                break cnd;
            }
        };

        let output = Command::new(format!("target/release/{}", bn))
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
