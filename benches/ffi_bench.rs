use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libloading::Library;
use std::ffi::{c_char, c_int};
use std::path::Path;
use std::time::Duration;
use wasmtime::*;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Numbers {
    a: c_int,
    b: c_int,
}

type AddFunc = unsafe extern "C" fn(a: i32, b: i32) -> i32;
type AddStructFunc = unsafe extern "C" fn(nums: Numbers) -> i32;
type AddStructPtrFunc = unsafe extern "C" fn(*mut Numbers) -> i32;
type HsInitFunc = unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char);
type HsExitFunc = unsafe extern "C" fn();

macro_rules! load_library {
    ($name:expr, $subdir:expr) => {
        unsafe {
            Library::new(Path::new($subdir).join($name))
                .expect(&format!("Failed to load {} library", $name))
        }
    };
}

macro_rules! load_symbol {
    ($lib:expr, $name:expr, $type:ty) => {
        unsafe {
            $lib.get::<$type>($name).expect(&format!(
                "Failed to load {} function",
                String::from_utf8_lossy($name)
            ))
        }
    };
}

macro_rules! bench_function {
    ($group:expr, $name:expr, $func:expr) => {
        $group.bench_function($name, |bencher| {
            bencher.iter(|| {
                black_box($func());
            });
        });
    };
}

macro_rules! bench_wasm_function {
    ($group:expr, $name:expr, $store:expr, $func:expr, $args:expr) => {
        $group.bench_function($name, |bencher| {
            let mut store_ref = $store.as_context_mut();
            bencher.iter(|| {
                let result = $func
                    .call(&mut store_ref, $args)
                    .expect("Failed Wasm function call");
                black_box(result);
            });
        });
    };
}

fn rs_add(a: i32, b: i32) -> i32 {
    a + b
}

fn rs_add_struct(nums: &Numbers) -> i32 {
    nums.a + nums.b
}

fn setup_wasm_module(wasm_file: &str) -> (Store<WasiP1Ctx>, Instance, Engine) {
    let engine = Engine::default();
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .build_p1();
    let mut store = Store::new(&engine, wasi);

    let mut linker = Linker::new(&engine);
    preview1::add_to_linker_sync(&mut linker, |s: &mut WasiP1Ctx| s).expect("Failed to link WASI");

    let module = Module::from_file(&engine, wasm_file)
        .expect(&format!("Failed to load Wasm module: {}", wasm_file));

    let instance = linker
        .instantiate(&mut store, &module)
        .expect("Failed to instantiate Wasm module");

    (store, instance, engine)
}

fn bench_wasm(
    group: &mut criterion::BenchmarkGroup<criterion::measurement::WallTime>,
    name: &str,
    wasm_file: &str,
    func_name: &str,
    init_func_name: Option<&str>,
    args: (i32, i32),
) {
    let (mut store, instance, _engine) = setup_wasm_module(wasm_file);

    if let Some(init_name) = init_func_name {
        let init_func = instance
            .get_typed_func::<(), ()>(&mut store, init_name)
            .expect(&format!(
                "Failed to get Wasm initialization function '{}'",
                init_name
            ));

        init_func.call(&mut store, ()).expect(&format!(
            "Failed to call Wasm initialization function '{}'",
            init_name
        ));
    }

    let wasm_func = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, func_name)
        .expect(&format!("Failed to get Wasm '{}' function", func_name));

    bench_wasm_function!(
        group,
        name,
        store,
        wasm_func,
        (black_box(args.0), black_box(args.1))
    );
}

fn bench_basic_addition(c: &mut Criterion) {
    let hs_lib = load_library!("libadd_hs.so", "foreign/hs");
    let go_lib = load_library!("libadd_go.so", "foreign/go");
    let nim_lib = load_library!("libadd_nim.so", "foreign/nim");
    let zig_lib = load_library!("libadd_zig.so", "foreign/zig");

    let hs_add = load_symbol!(&hs_lib, b"add\0", AddFunc);
    let hs_init = load_symbol!(&hs_lib, b"hs_init\0", HsInitFunc);
    let hs_exit = load_symbol!(&hs_lib, b"hs_exit\0", HsExitFunc);
    let go_add = load_symbol!(&go_lib, b"add\0", AddFunc);
    let nim_add = load_symbol!(&nim_lib, b"add\0", AddFunc);
    let zig_add = load_symbol!(&zig_lib, b"add\0", AddFunc);

    let mut group = c.benchmark_group("add");
    let (a, b) = (10, 20);

    unsafe {
        hs_init(std::ptr::null_mut(), std::ptr::null_mut());
    }
    bench_function!(group, "haskell", || unsafe { hs_add(a, b) });
    unsafe {
        hs_exit();
    }
    bench_function!(group, "go", || unsafe { go_add(a, b) });
    bench_wasm(
        &mut group,
        "go_wasm",
        "foreign/go/libadd_go.wasm",
        "add",
        Some("_initialize"),
        (a, b),
    );
    bench_wasm(
        &mut group,
        "go_tinygo_wasm",
        "foreign/go/libadd_tinygo.wasm", // Corrected filename based on Makefile
        "add",
        None,
        (a, b),
    );

    bench_function!(group, "nim", || unsafe { nim_add(a, b) });
    bench_function!(group, "zig", || unsafe { zig_add(a, b) });
    bench_function!(group, "rust", || rs_add(a, b));

    group.finish();
}

fn bench_struct_addition(c: &mut Criterion) {
    let hs_lib = load_library!("libadd_hs.so", "foreign/hs");
    let go_lib = load_library!("libadd_go.so", "foreign/go");
    let nim_lib = load_library!("libadd_nim.so", "foreign/nim");
    let zig_lib = load_library!("libadd_zig.so", "foreign/zig");

    let hs_add_struct = load_symbol!(&hs_lib, b"add_struct\0", AddStructPtrFunc);
    let hs_init = load_symbol!(&hs_lib, b"hs_init\0", HsInitFunc);
    let hs_exit = load_symbol!(&hs_lib, b"hs_exit\0", HsExitFunc);
    let go_add_struct = load_symbol!(&go_lib, b"add_struct\0", AddStructFunc);
    let nim_add_struct = load_symbol!(&nim_lib, b"add_struct\0", AddStructFunc);
    let zig_add_struct = load_symbol!(&zig_lib, b"add_struct\0", AddStructFunc);

    let mut group = c.benchmark_group("add_struct");
    let nums = Numbers { a: 10, b: 20 };

    unsafe {
        hs_init(std::ptr::null_mut(), std::ptr::null_mut());
    }
    bench_function!(group, "haskell", || unsafe {
        let mut nums_copy = nums;
        hs_add_struct(&mut nums_copy)
    });
    unsafe {
        hs_exit();
    }
    bench_function!(group, "go", || unsafe { go_add_struct(nums) });
    bench_function!(group, "nim", || unsafe { nim_add_struct(nums) });
    bench_function!(group, "zig", || unsafe { zig_add_struct(nums) });
    bench_function!(group, "rust", || rs_add_struct(&nums));

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(1))
        .output_directory(Path::new("benches_result"));
    targets = bench_basic_addition, bench_struct_addition
}
criterion_main!(benches);
