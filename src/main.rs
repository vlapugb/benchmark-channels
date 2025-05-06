use async_channel::{self, Receiver, Sender};
use std::fs::File;
use std::io;
use std::io::Write;
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::Instant;
#[warn(unused_assignments)]
#[tokio::main]

async fn main() -> io::Result<()> {
    let mut str = String::new();
    let mut str2 = String::new();
    let mut sum_a: f32 = 0.0;
    let mut sum_b: f32 = 0.0;
    let mut sum_c: f32 = 0.0;
    io::stdin().read_line(&mut str).expect("error");
    io::stdin().read_line(&mut str2).expect("error");
    let producers_vec: Vec<i64> = str
        .split_whitespace()
        .map(|s| s.parse::<i64>().expect("error"))
        .collect();
    let mes_per_prod_vec: Vec<i64> = str2
        .split_whitespace()
        .map(|s| s.parse::<i64>().expect("error"))
        .collect();
    let mut file = File::create("sum_output.txt")?;
    for producer in producers_vec.iter() {
        for mes_per_prod in mes_per_prod_vec.iter() {
            for _ in 0..9 {
                let res_a = async_channel_bench(*producer, *mes_per_prod).await;
                let res_b = async_channel_bench2(*producer, *mes_per_prod).await;
                let res_c = tokio_channel_bench(*producer, *mes_per_prod).await;
                sum_a += res_a as f32;
                sum_b += res_b as f32;
                sum_c += res_c as f32;
            }
            writeln!(
                file,
                "{} AND {} TIME: {} ASYNC_B",
                *producer,
                *mes_per_prod,
                sum_a / 10.0
            )?;
            writeln!(
                file,
                "{} AND {} TIME: {} ASYNC_U",
                *producer,
                *mes_per_prod,
                sum_b / 10.0
            )?;
            writeln!(
                file,
                "{} AND {} TIME: {} TOKIO",
                *producer,
                *mes_per_prod,
                sum_c / 10.0
            )?;
            sum_a = 0.0;
            sum_b = 0.0;
            sum_c = 0.0;
        }
    }
    Ok(())
}

async fn tokio_channel_bench(producer: i64, mes_per_prod: i64) -> i64 {
    let (txtokio, mut rxtokio) = mpsc::channel(2000000);
    let now = Instant::now();

    let producers: Vec<_> = (0..producer)
        .map(|_| {
            let tx_clone = txtokio.clone();
            task::spawn(async move {
                for i in 0..mes_per_prod {
                    let _ = tx_clone.send(i).await;
                }
            })
        })
        .collect();
    let consumer = task::spawn(async move {
        while let Some(message) = rxtokio.recv().await {
            let _ = message;
        }
    });
    drop(txtokio);
    let _ = futures::future::join_all(producers).await;
    consumer.await.unwrap();
    let end = now.elapsed();
    println!(
        "{} milliseconds for {} producerss {} msg's TOKIO",
        end.as_millis(),
        producer,
        mes_per_prod
    );
    end.as_millis() as i64
}

async fn async_channel_bench(producer: i64, mes_per_prod: i64) -> i64 {
    let (tx, rx): (Sender<i64>, Receiver<i64>) = async_channel::unbounded();

    let now1 = Instant::now();
    let consumer = task::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = msg;
        }
    });
    let producers: Vec<_> = (0..producer)
        .map(|_| {
            let tx_clone = tx.clone();
            task::spawn(async move {
                for i in 0..mes_per_prod {
                    let _ = tx_clone.send(i).await;
                }
            })
        })
        .collect();
    drop(tx);

    let _ = futures::future::join_all(producers).await;
    consumer.await.unwrap();
    let end1 = now1.elapsed();
    println!(
        "{} milliseconds for {} producers {} msg's ASYNC_UNBDOUNDED",
        end1.as_millis(),
        producer,
        mes_per_prod
    );
    end1.as_millis() as i64
}

async fn async_channel_bench2(producer: i64, mes_per_prod: i64) -> i64 {
    let (tx, rx): (Sender<i64>, Receiver<i64>) = async_channel::unbounded();

    let now1 = Instant::now();
    let consumer = task::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = msg;
        }
    });
    let producers: Vec<_> = (0..producer)
        .map(|_| {
            let tx_clone = tx.clone();
            task::spawn(async move {
                for i in 0..mes_per_prod {
                    let _ = tx_clone.send(i).await;
                }
            })
        })
        .collect();
    drop(tx);

    let _ = futures::future::join_all(producers).await;
    consumer.await.unwrap();
    let end1 = now1.elapsed();
    println!(
        "{} milliseconds for {} producers and {} msg's ASYNC_BOUNDED",
        end1.as_millis(),
        producer,
        mes_per_prod
    );
    end1.as_millis() as i64
}
