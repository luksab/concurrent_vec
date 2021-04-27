use concurrent_vec::ConcVec;

fn base_one() {
    let base = Arc::new(ConcVec::<_, 1>::with_capacity(1));
    let mut vec = ConcVec::get_appender(base.clone());
    let n = 1024;

    for i in 0..n {
        vec.push(i);
    }

    // for i in 0..n {
    //     assert_eq!(vec[i], i);
    // }
}

fn base_thirtytwo() {
    let base = Arc::new(ConcVec::<_, 32>::with_capacity(32));
    let mut vec = ConcVec::get_appender(base.clone());
    let n = 1_000_000;

    for i in 0..n {
        vec.push(i);
    }

    // for i in 0..n {
    //     assert_eq!(vec[i], i);
    //     assert_eq!(vec.get(i), Some(&i));
    //     unsafe {
    //         assert_eq!(vec.get_unchecked(i), &i);
    //     }
    // }
}

use std::{
    collections::HashSet,
    sync::{atomic::AtomicUsize, Mutex},
    thread,
    time::Duration,
    usize,
};
use std::{fs, sync::Arc, time::Instant};

const N: usize = 10_000_000;

fn multithreading(n_threads: usize) {
    let conc_vec = Arc::new(ConcVec::<_, 1024>::with_capacity(1));
    //let vec = Arc::new(Aoavec::with_capacity(N));
    //let vec = Arc::new(Aoavec::new());

    let mut handles = vec![];

    for t in 0..n_threads {
        let mut vec = ConcVec::get_appender(conc_vec.clone());
        handles.push(thread::spawn(move || {
            for i in 0..N {
                if i % n_threads == t {
                    vec.push(i);
                }
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }

    //assert_eq!(conc_vec.len(), N);

    // let mut set = HashSet::new();

    // for i in conc_vec.into_iter() {
    //     set.insert(i);
    // }

    // assert_eq!(set.len(), N);
}

fn multithreading_vec(n_threads: usize) {
    let vec = Arc::new(Mutex::new(Vec::with_capacity(32)));

    let mut handles = vec![];

    for t in 0..n_threads {
        let vec = vec.clone();
        handles.push(thread::spawn(move || {
            for i in 0..N {
                if i % n_threads == t {
                    vec.lock().unwrap().push(i);
                }
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }

    // let mut set = HashSet::new();

    // let vec = vec.lock().unwrap();
    // for i in 0..N {
    //     set.insert(vec[i]);
    // }

    // assert_eq!(set.len(), N);
}

// fn multithreading_dash_map(n_threads: usize) {
//     use dashmap::DashSet;
//     let vec = Arc::new(DashSet::with_capacity(32));

//     let mut handles = vec![];

//     for t in 0..n_threads {
//         let vec = vec.clone();
//         handles.push(thread::spawn(move || {
//             for i in 0..N {
//                 if i % n_threads == t {
//                     vec.insert(i);
//                 }
//             }
//         }))
//     }

//     for h in handles {
//         h.join().unwrap();
//     }

//     // let mut set = HashSet::new();

//     // let vec = vec.lock();
//     // for i in 0..n {
//     //     set.insert(vec[i]);
//     // }

//     // assert_eq!(set.len(), n);
// }

fn multithreading_atomic_usize(n_threads: usize) {
    let vec = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for t in 0..n_threads {
        let vec = vec.clone();
        handles.push(thread::spawn(move || {
            for i in 0..N {
                if i % n_threads == t {
                    vec.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
}

// struct SharedUsize {
//     num: *mut usize,
// }

// impl SharedUsize {
//     pub fn add(&self) {
//         unsafe { *self.num += 1 };
//     }
// }

// unsafe impl Sync for SharedUsize {}
// unsafe impl Send for SharedUsize {}

// fn multithreading_usize_mem(n_threads: usize) {
//     let mut num: usize = 0;
//     let vec = Arc::new(SharedUsize {
//         num: &mut num as *mut _,
//     });

//     let mut handles = vec![];

//     for t in 0..n_threads {
//         let vec = vec.clone();
//         handles.push(thread::spawn(move || {
//             for i in 0..N {
//                 if i % n_threads == t {
//                     vec.add();
//                 }
//             }
//         }))
//     }

//     for h in handles {
//         h.join().unwrap();
//     }
// }

// fn multithreading_usize(n_threads: usize) {
//     let mut _num: usize = 0;

//     let mut handles = vec![];

//     for _ in 0..n_threads {
//         handles.push(thread::spawn(move || {
//             let mut num_int = 0;
//             for _ in 0..N / n_threads {
//                 num_int += 1;
//                 std::hint::black_box(num_int);
//             }
//             num_int
//         }))
//     }

//     for h in handles {
//         _num += h.join().unwrap();
//     }
// }

use std::fs::OpenOptions;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        match args[1].as_str() {
            "mTh" => multithreading(12),
            "BaseOne" => base_one(),
            "BaseThrityTwo" => base_thirtytwo(),
            "Run10Min" => {
                let now = Instant::now();
                while now.elapsed() < Duration::from_secs(600) {
                    for i in 1..=12 {
                        multithreading(i);
                    }
                }
            }
            "Benchmark" => {
                const NUM_CORES: usize = 6;
                let mut benchmark_nums = [0.; NUM_CORES];
                let mut benchmark_nums_vec = [0.; NUM_CORES];
                // let mut benchmark_nums_dash_map = [0.; NUM_CORES];
                let mut benchmark_nums_atomic_usize = [0.; NUM_CORES];
                // let mut benchmark_nums_usize_mem = [0.; NUM_CORES];
                // let mut benchmark_nums_usize = [0.; NUM_CORES];
                let num_runs = if args.len() > 2 {
                    args[2].parse().unwrap_or(1)
                } else {
                    1
                };
                print!("Running Benchmark {} times", num_runs);

                for run in 0..num_runs {
                    print!(", {}", run);
                    for i in 0..NUM_CORES {
                        let now = Instant::now();
                        multithreading(i + 1);
                        benchmark_nums[i] += now.elapsed().as_secs_f64();

                        let now = Instant::now();
                        multithreading_vec(i + 1);
                        benchmark_nums_vec[i] += now.elapsed().as_secs_f64();

                        // let now = Instant::now();
                        // multithreading_dash_map(i + 1);
                        // benchmark_nums_dash_map[i] += now.elapsed().as_secs_f64();

                        let now = Instant::now();
                        multithreading_atomic_usize(i + 1);
                        benchmark_nums_atomic_usize[i] += now.elapsed().as_secs_f64();

                        // let now = Instant::now();
                        // multithreading_usize_mem(i + 1);
                        // benchmark_nums_usize_mem[i] += now.elapsed().as_secs_f64();

                        // let now = Instant::now();
                        // multithreading_usize(i + 1);
                        // benchmark_nums_usize[i] += now.elapsed().as_secs_f64();
                    }
                }
                println!();

                fn write_plot(data: &[f64], name: &str, num_runs: usize) {
                    fs::create_dir("./plots").unwrap_or_else(|err| println!("{:?}", err));
                    let path = String::from("plots/") + name;
                    if fs::remove_file(&path).is_err() {};
                    let mut plot_file = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&path)
                        .unwrap();

                    for i in 0..NUM_CORES {
                        if let Err(e) =
                            writeln!(plot_file, "{} {}", i + 1, (N * num_runs) as f64 / data[i],)
                        {
                            eprintln!("Couldn't write to file: {}", e);
                        };
                    }
                    match plot_file.flush() {
                        std::result::Result::Ok(_) => {}
                        std::result::Result::Err(_) => {
                            println!("didn't work")
                        }
                    };
                }

                write_plot(&benchmark_nums, "aoaVec", num_runs);
                //write_plot(&benchmark_nums_vec, "MutexVec", num_runs);
                //write_plot(&benchmark_nums_dash_map, "dashSet", num_runs);
                //write_plot(&benchmark_nums_atomic_usize, "atomic", num_runs);
                //write_plot(&benchmark_nums_usize_mem, "usizeMem", num_runs);

                let _ = std::process::Command::new("gnuplot")
                    .arg("plot.gnu")
                    .status()
                    .unwrap();
            }
            _ => println!("This is not the answer."),
        }
    }
}
