use policy_carrying_data::define_schema;
use policy_core::{col, cols, expr::count};
use policy_execution::{context::PcdAnalysisContext, lazy::IntoLazy};

#[cfg(debug_assertions)]
static LIB_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/debug/libexecutor_lib.so"
);
#[cfg(not(debug_assertions))]
static LIB_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../target/release/libexecutor_lib.so"
);

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();
    let schema = define_schema! {
        "column_1" => DataType::UInt32,
        "column_2" => DataType::Float64,
    };

    print!("[+] Creating new analysis context...");
    let mut ctx = PcdAnalysisContext::new();
    ctx.initialize(LIB_PATH)?;
    println!("\tOK");

    print!("[+] Registering data in the analysis context...");
    ctx.register_data_from_path(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../test_data/simple_csv.csv"
        ),
        schema,
    )?;
    println!("\tOK");

    let df = ctx
        .lazy()
        .select(cols!("column_1", "column_2"))
        .filter(col!("column_1").ge(4u32).and(col!("column_2").lt(10000.0)))
        .groupby([col!("column_2")])
        .agg([
            col!("column_1").min().alias("min value"),
            col!("column_1").sum(),
            count(),
        ]);

    println!("[+] Explaining the plan {}", df.explain());

    let df = df.collect().unwrap();
    println!("[+] Dataframe => {df:?}");

    Ok(())
}
