use concurrent_vec::ConcVec;
use std::time::Instant;
use std::{collections::HashSet, thread, time::Duration, usize};

const N: usize = 10_000_000;

#[test]
fn test_mth() {
    multithreading(12, 1024, 10_000);
}

fn benchmark_vec<F>(f: F, mut inputs: Vec<usize>, num_runs: usize) -> (Vec<f64>, Vec<f64>)
where
    F: Fn(usize),
{
    fn mean(data: &[f64]) -> Option<f64> {
        let sum = data.iter().sum::<f64>() as f64;
        let count = data.len();

        match count {
            positive if positive > 0 => Some(sum / count as f64),
            _ => None,
        }
    }

    fn std_deviation(data: &[f64]) -> Option<f64> {
        match (mean(data), data.len()) {
            (Some(data_mean), count) if count > 0 => {
                let variance = data
                    .iter()
                    .map(|value| {
                        let diff = data_mean - (*value as f64);

                        diff * diff
                    })
                    .sum::<f64>()
                    / count as f64;

                Some(variance.sqrt())
            }
            _ => None,
        }
    }

    let mut benchmark_nums = vec![Vec::<f64>::new(); inputs.len()];

    //warmup
    for j in inputs.iter_mut() {
        f(*j);
    }

    for _run in 0..num_runs {
        for (i, j) in inputs.iter_mut().enumerate() {
            let now = Instant::now();
            f(*j);
            //multithreading_size(NUM_CORES, get_buf_size(j), N);
            benchmark_nums[i].push(N as f64 / now.elapsed().as_secs_f64());
        }
    }

    let stds: Vec<f64> = benchmark_nums
        .iter()
        .map(|vec| std_deviation(vec))
        .flatten()
        .collect();

    let values: Vec<_> = benchmark_nums
        .iter()
        .map::<f64, _>(|v| v.iter().sum::<f64>() / (num_runs as f64))
        .collect();
    (values, stds)
}

fn multithreading(n_threads: usize, buf_size: usize, n: usize) {
    let conc_vec = ConcVec::new(6, buf_size);
    let mut handles = vec![];

    for _ in 0..n_threads {
        let mut vec = conc_vec.clone().get_appender();
        handles.push(thread::spawn(move || {
            for i in 0..n / n_threads {
                vec.push(i);
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
}

fn multithreading_size_just_vec(n_threads: usize, buf_size: usize, n: usize) {
    let mut handles = vec![];

    for _ in 0..n_threads {
        handles.push(thread::spawn(move || {
            let mut vec = Vec::new();
            for i in 0..n / n_threads {
                vec.push(i);
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
}

struct LageStruct {
    data: [usize; 0],
}
fn multithreading_struct(n_threads: usize, buf_size: usize, n: usize) {
    let conc_vec = ConcVec::new(1, buf_size);

    let mut handles = vec![];

    for _ in 0..n_threads {
        let mut conc_vec = conc_vec.clone().get_appender();
        handles.push(thread::spawn(move || {
            for i in 0..n / n_threads {
                conc_vec.push(LageStruct { data: [i; 0] });
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }

    // let mut set = HashSet::new();
    // let len = conc_vec.len();
    // for i in conc_vec.take_iter() {
    //     set.insert(i.counter);
    // }
    // assert_eq!(set.len(), len);
}

const COLORS: [&str; 8] = [
    "#3b4cc0", "#688aef", "#99baff", "#c9d8ef", "#edd1c2", "#f7a789", "#e36a53", "#b40426",
];

fn buf_size() {
    use gnuplot::*;

    fn get_buf_size(j: usize) -> usize {
        1 << j
    }

    const N: usize = 1_000_000;
    const NUM_CORES: usize = 6;
    let num_samples = 15;
    let num_runs = 20;
    //let mut benchmark_nums = vec![Vec::<f64>::new(); num_samples];

    println!("Running Benchmark {} times", num_runs);

    let mut fg = Figure::new();
    let x: Vec<_> = (0..num_samples).map(|x| get_buf_size(x) as f64).collect();

    //let mut data = Vec::new();
    let plot = fg
        .axes2d()
        .set_title("buf\\_size", &[])
        .set_legend(Graph(0.9), Graph(0.5), &[], &[])
        .set_x_label("buf\\_size", &[])
        .set_y_label("inserts per second", &[])
        .set_y_log(Some(10.0))
        .set_x_log(Some(2.0));

    println!("concurrentVec");
    for threads in &[1, 2, 6] {
        println!("{} threads", threads);
        let (values, stds) = benchmark_vec(
            |buf_size: usize| multithreading(*threads, buf_size, N),
            (0..num_samples).map(get_buf_size).collect(),
            num_runs,
        );

        let y_low: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v - s).collect();
        let y_hi: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v + s).collect();

        plot.lines(
            &x,
            &values,
            &[
                Caption(&format!("concurrentVec {} threads", threads)),
                Color(COLORS[*threads]),
            ],
        );
        plot.fill_between(
            &x,
            &y_low,
            &y_hi,
            &[FillRegion(FillRegionType::Below), Color(COLORS[*threads])],
        );
        //plot.y_error_lines(&x, &values, &stds, &[Caption(&cores.to_string())]);
    }

    println!("std::vec");
    for (i, threads) in (&[1, 6]).iter().enumerate() {
        println!("{} threads", threads);
        let (values, stds) = benchmark_vec(
            |buf_size: usize| multithreading_size_just_vec(*threads, buf_size, N),
            (0..num_samples).map(get_buf_size).collect(),
            num_runs,
        );

        let y_low: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v - s).collect();
        let y_hi: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v + s).collect();

        //let _: Vec<_> = y_low.iter().zip(&y_hi).map(|(l, h)| println!("{} {}", l, h)).collect();
        //plot.y_error_lines(&x, &values, &stds, &[Caption("Just Vec 6 cores"), ]);
        plot.lines(
            &x,
            &values,
            &[
                Caption(&format!("std::Vec {} threads", threads)),
                Color(COLORS[3 + i * 2]),
            ],
        );
        plot.fill_between(
            &x,
            &y_low,
            &y_hi,
            &[FillRegion(FillRegionType::Below), Color(COLORS[3 + i * 2])],
        );
    }

    println!("concurrentVec_large");
    for (i, threads) in (&[1, 2, 6]).iter().enumerate() {
        println!("{} threads", threads);
        let (values, stds) = benchmark_vec(
            |buf_size: usize| multithreading_struct(*threads, buf_size, N),
            (0..num_samples).map(get_buf_size).collect(),
            num_runs,
        );

        let y_low: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v - s).collect();
        let y_hi: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v + s).collect();

        //let _: Vec<_> = y_low.iter().zip(&y_hi).map(|(l, h)| println!("{} {}", l, h)).collect();
        //plot.y_error_lines(&x, &values, &stds, &[Caption("Just Vec 6 cores"), ]);
        plot.lines(
            &x,
            &values,
            &[
                Caption(&format!("concurrentVec\\_struct {} threads", threads)),
                Color(COLORS[[0, 7, 4][i]]),
            ],
        );
        plot.fill_between(
            &x,
            &y_low,
            &y_hi,
            &[
                FillRegion(FillRegionType::Below),
                Color(COLORS[[0, 7, 4][i]]),
            ],
        );
    }

    //.lines(&x, &values, &[]);
    //fg.echo_to_file("test.gnu");
    fg.save_to_svg("buf_size.svg", 600, 480).unwrap();
    //fg.show().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let now = Instant::now();
    if args.len() >= 2 {
        match args[1].as_str() {
            "mTh" => multithreading(12, 1024, N),
            "Run10Min" => {
                let now = Instant::now();
                while now.elapsed() < Duration::from_secs(600) {
                    for i in 1..=12 {
                        multithreading(i, 1024, N);
                    }
                }
            }
            "Benchmark_size" => buf_size(),
            "Large_struct" => multithreading_struct(12, 32, 10_000),
            _ => println!("This is not the answer."),
        }
    }
    println!("running took {:?}", now.elapsed());
}
