use concurrent_vec::ConcVec;

fn base_one() {
    let base = ConcVec::new(1, 1);
    let mut vec = base.clone().get_appender();
    let n = 1024;

    for i in 0..n {
        vec.push(i);
    }

    assert_eq!(base.len(), N);

    let mut set = HashSet::new();

    for i in base.take_iter() {
        set.insert(i);
    }

    assert_eq!(set.len(), N);
}

fn base_thirtytwo() {
    let base = ConcVec::new(1, 32);
    let mut vec = base.clone().get_appender();
    let n = 1_000_000;

    for i in 0..n {
        vec.push(i);
    }

    let mut set = HashSet::new();

    for i in base.take_iter() {
        set.insert(i);
    }

    assert_eq!(set.len(), N);

    // for i in 0..n {
    //     assert_eq!(vec[i], i);
    //     assert_eq!(vec.get(i), Some(&i));
    //     unsafe {
    //         assert_eq!(vec.get_unchecked(i), &i);
    //     }
    // }
}

use std::time::Instant;
use std::{collections::HashSet, thread, time::Duration, usize};

const N: usize = 10_000_000;

#[test]
fn test_mth() {
    multithreading(12);
}
fn multithreading(n_threads: usize) {
    let conc_vec = ConcVec::new(1, 1024);
    //let vec = Arc::new(Aoavec::with_capacity(N));
    //let vec = Arc::new(Aoavec::new());

    let mut handles = vec![];

    for t in 0..n_threads {
        let mut vec = conc_vec.clone().get_appender();
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

    assert_eq!(conc_vec.len(), N);

    let mut set = HashSet::new();

    for i in conc_vec.take_iter() {
        set.insert(i);
    }

    assert_eq!(set.len(), N);
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

fn multithreading_size(n_threads: usize, buf_size: usize, n: usize) {
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
    let num_runs = 50;
    //let mut benchmark_nums = vec![Vec::<f64>::new(); num_samples];

    println!("Running Benchmark {} times", num_runs);

    let mut fg = Figure::new();
    let x: Vec<_> = (0..num_samples).map(|x| get_buf_size(x) as f64).collect();

    //let mut data = Vec::new();
    let plot = fg
        .axes2d()
        .set_title("buf\\_size", &[])
        .set_legend(Graph(0.9), Graph(0.3), &[], &[])
        .set_x_label("buf\\_size", &[])
        .set_y_label("inserts per second", &[])
        .set_y_log(Some(10.0))
        .set_x_log(Some(2.0));

    for cores in &[1, 2, 6] {
        println!("{} cores", cores);
        let (values, stds) = benchmark_vec(
            |buf_size: usize| multithreading_size(*cores, buf_size, N),
            (0..num_samples).map(get_buf_size).collect(),
            num_runs,
        );

        let y_low: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v - s).collect();
        let y_hi: Vec<_> = values.iter().zip(stds.iter()).map(|(v, s)| v + s).collect();

        plot.lines(
            &x,
            &values,
            &[
                Caption(&format!("concurrentVec {} cores", cores)),
                Color(COLORS[*cores]),
            ],
        );
        plot.fill_between(
            &x,
            &y_low,
            &y_hi,
            &[FillRegion(FillRegionType::Below), Color(COLORS[*cores])],
        );
        //plot.y_error_lines(&x, &values, &stds, &[Caption(&cores.to_string())]);
    }

    for (i, cores) in (&[1, 6]).iter().enumerate() {
        let (values, stds) = benchmark_vec(
            |buf_size: usize| multithreading_size_just_vec(*cores, buf_size, N),
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
                Caption(&format!("std::Vec {} cores", cores)),
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
            "Benchmark_size" => buf_size(),
            _ => println!("This is not the answer."),
        }
    }
    println!("running took {:?}", now.elapsed());
}
