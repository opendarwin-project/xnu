use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use nix_eval::{Evaluator, NixValue};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(
    name = "xnu-nix-gen",
    about = "Blazing-fast Nix+Ninja build tool for XNU"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "qemu")]
    board: String,

    #[arg(short, long, default_value = "DEVELOPMENT")]
    config: String,

    #[arg(short, long, default_value_t = false)]
    werror: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate build/build.ninja from .nix files
    Generate {
        #[arg(short, long, default_value = "qemu")]
        board: String,
        #[arg(short, long, default_value = "DEVELOPMENT")]
        config: String,
    },
    /// Generate ninja and build target
    Build {
        #[arg(short, long, default_value = "qemu")]
        board: String,
        #[arg(short, long, default_value = "DEVELOPMENT")]
        config: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        ninja_args: Vec<String>,
    },
    /// Build and run under QEMU
    Qemu {
        #[arg(short, long, default_value = "qemu")]
        board: String,
        #[arg(short, long, default_value = "DEVELOPMENT")]
        config: String,
    },
}

fn get_str_attr<'a>(map: &'a HashMap<String, NixValue>, key: &str) -> Result<&'a str> {
    match map.get(key) {
        Some(NixValue::String(s)) => Ok(s.as_str()),
        Some(v) => Err(anyhow!("Expected string for key {}, got {:?}", key, v)),
        None => Err(anyhow!("Missing key {}", key)),
    }
}

fn get_list_strings(val: Option<&NixValue>) -> Vec<String> {
    match val {
        Some(NixValue::List(list)) => list
            .iter()
            .filter_map(|item| match item {
                NixValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn generate_ninja(board: &str, config: &str, werror: bool) -> Result<()> {
    let t0 = std::time::Instant::now();
    let evaluator = Evaluator::new();
    let expr = format!(
        "(import ./default.nix {{ board = \"{}\"; config = \"{}\"; werror = {}; }})",
        board, config, werror
    );

    let val = evaluator
        .evaluate(&expr)
        .context("Failed to evaluate default.nix")?;

    let root_map = match val {
        NixValue::AttributeSet(map) => map,
        _ => return Err(anyhow!("Expected attribute set from default.nix")),
    };

    let tools_map = match root_map.get("tools") {
        Some(NixValue::AttributeSet(map)) => map,
        _ => return Err(anyhow!("Missing tools attribute set")),
    };

    let cc = get_str_attr(tools_map, "cc").unwrap_or("clang");
    let cxx = get_str_attr(tools_map, "cxx").unwrap_or("clang++");
    let libtool = get_str_attr(tools_map, "libtool").unwrap_or("libtool");
    let mig = get_str_attr(tools_map, "mig").unwrap_or("mig");
    let migcom = get_str_attr(tools_map, "migcom").unwrap_or("/usr/libexec/migcom");
    let iig = get_str_attr(tools_map, "iig").unwrap_or("/usr/bin/iig");
    let python = get_str_attr(tools_map, "python").unwrap_or("python3");
    let cargo = get_str_attr(tools_map, "cargo").unwrap_or("cargo");

    fs::create_dir_all("build")?;

    // Generate IOStorageFamily header shims if directory exists
    let iostorage_dir: PathBuf = if let Some(sources_val) = root_map.get("sources") {
        match sources_val {
            NixValue::AttributeSet(sources_map) => {
                if let Some(ios) = sources_map.get("iostorage") {
                    match ios {
                        NixValue::Path(p) => p.clone(),
                        NixValue::String(s) => PathBuf::from(s),
                        NixValue::AttributeSet(attrs) => {
                            if let Some(NixValue::String(s)) = attrs.get("outPath") {
                                PathBuf::from(s)
                            } else if let Some(NixValue::Path(p)) = attrs.get("outPath") {
                                p.clone()
                            } else {
                                PathBuf::from("external/IOStorageFamily")
                            }
                        }
                        _ => PathBuf::from("external/IOStorageFamily"),
                    }
                } else {
                    PathBuf::from("external/IOStorageFamily")
                }
            }
            _ => PathBuf::from("external/IOStorageFamily"),
        }
    } else {
        PathBuf::from("external/IOStorageFamily")
    };
    if iostorage_dir.exists() {
        let shim_dirs = [
            PathBuf::from("build/gen/IOKit/storage"),
            PathBuf::from("build/gen/export/IOKit/storage"),
        ];
        for shim_dir in &shim_dirs {
            fs::create_dir_all(shim_dir)?;
        }
        if let Ok(entries) = fs::read_dir(iostorage_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map_or(false, |ext| ext == "h") {
                    let filename = p.file_name().unwrap();
                    let abs_path = p.canonicalize().unwrap_or(p.clone());
                    let content = format!("#include \"{}\"\n", abs_path.display());
                    for shim_dir in &shim_dirs {
                        let shim_path = shim_dir.join(filename);
                        if let Ok(existing) = fs::read_to_string(&shim_path) {
                            if existing == content {
                                continue;
                            }
                        }
                        let _ = fs::write(shim_path, &content);
                    }
                }
            }
        }
    }

    let mut out = String::new();

    out.push_str("# Generated by xnu-nix-gen (Nix-based build system)\n");
    out.push_str("ninja_required_version = 1.8\n\n");
    out.push_str(&format!("cc = {}\n", cc));
    out.push_str(&format!("cxx = {}\n", cxx));
    out.push_str(&format!("libtool = {}\n", libtool));
    out.push_str(&format!("mig = {}\n", mig));
    out.push_str(&format!("migcom = {}\n", migcom));
    out.push_str(&format!("iig = {}\n", iig));
    out.push_str(&format!("python = {}\n", python));
    out.push_str(&format!("cargo = {}\n\n", cargo));

    // Standard Ninja rules
    out.push_str("rule cc\n  depfile = $out.d\n  deps = gcc\n  command = $cc -MD -MF $out.d $cflags -c $in -o $out\n\n");
    out.push_str("rule cxx\n  depfile = $out.d\n  deps = gcc\n  command = $cxx -MD -MF $out.d $cxxflags -c $in -o $out\n\n");
    out.push_str("rule asm\n  depfile = $out.d\n  deps = gcc\n  command = $cc -MD -MF $out.d $asmflags -c $in -o $out\n\n");
    out.push_str("rule ar\n  command = $libtool -static -o $out $in\n\n");
    out.push_str("rule mig\n  command = $mig -novouchers -migcom $migcom $migflags $in\n\n");
    out.push_str("rule iig\n  command = $iig --def $in --header $header_out --impl $impl_out --framework-name DriverKit -- $iigflags\n\n");
    out.push_str(
        "rule makesyscalls\n  command = $script $outdir $master $makesyscalls $gendir\n\n",
    );
    out.push_str("rule xnu_config\n  command = $python tools/pcons/xnu_config.py --srcroot . --kernel-config $config --format stats --outdir build/gen\n\n");
    out.push_str("rule genassym\n  command = $cc -S -fno-integrated-as $cflags -o $out $in\n\n");
    out.push_str(
        "rule extract_assym\n  command = $python tools/pcons/extract_assym.py $in $out\n\n",
    );
    out.push_str("rule stamp_version\n  command = $python tools/pcons/stamp_version.py --template $in --output $out --config $config --version $version --objroot .\n\n");
    out.push_str("rule cargo_build\n  command = env CARGO_TARGET_DIR=$target_dir $cargo build --manifest-path $manifest $cargo_flags && (cmp -s $artifact $out || cp -f $artifact $out) && touch -c $out\n\n");
    out.push_str("rule link_kernel\n  command = $cc $ldflags -o $out $in\n\n");
    out.push_str(
        "rule kernelcache\n  command = $python tools/pcons/kernelcache.py $in -o $out\n\n",
    );

    // Codegen edges
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("# Code Generation\n");
    out.push_str("# ------------------------------------------------------------\n");

    // XNU Config
    out.push_str(&format!(
        "build build/gen/master_config.h build/gen/meta_features.h build/gen/ioconf.c: xnu_config config/MASTER config/MASTER.arm64 | tools/pcons/xnu_config.py\n  config = {}\n\n",
        config
    ));

    // Syscalls
    out.push_str("build build/syscalls/init_sysent.c build/syscalls/syscalls.c build/syscalls/audit_kevents.c build/syscalls/systrace_args.c build/syscalls/sysproto.h build/syscalls/syscall.h: makesyscalls bsd/kern/syscalls.master bsd/kern/makesyscalls.sh | tools/pcons/run_makesyscalls.sh build/gen/master_config.h\n");
    out.push_str("  script = tools/pcons/run_makesyscalls.sh\n  outdir = build/syscalls\n  master = bsd/kern/syscalls.master\n  makesyscalls = bsd/kern/makesyscalls.sh\n  gendir = build/gen\n\n");

    // Version
    out.push_str(&format!(
        "build build/version/version.c: stamp_version config/version.c.template | tools/pcons/stamp_version.py\n  config = {}\n  version = 24.0.0\n  script = tools/pcons/stamp_version.py\n\n",
        config
    ));

    // MIG Jobs
    let mig_jobs_map = match root_map.get("codegen") {
        Some(NixValue::AttributeSet(cg)) => match cg.get("mig") {
            Some(NixValue::AttributeSet(m)) => match m.get("jobs") {
                Some(NixValue::List(l)) => l.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let mut mig_all_outputs = Vec::new();
    for job_val in &mig_jobs_map {
        if let NixValue::AttributeSet(job) = job_val {
            let defs_file = get_str_attr(job, "defsFile").unwrap_or("");
            let outputs = get_list_strings(job.get("outputs"));
            let flags = get_list_strings(job.get("flags"));
            if !outputs.is_empty() && !defs_file.is_empty() {
                let out_joined = outputs.join(" ");
                out.push_str(&format!(
                    "build {}: mig {} | osfmk/mach/std_types.defs osfmk/mach/mach_types.defs build/gen/master_config.h\n  migflags = {}\n\n",
                    out_joined, defs_file, flags.join(" ")
                ));
                mig_all_outputs.extend(outputs);
            }
        }
    }

    // IIG Jobs
    let iig_jobs_map = match root_map.get("codegen") {
        Some(NixValue::AttributeSet(cg)) => match cg.get("iig") {
            Some(NixValue::AttributeSet(i)) => match i.get("jobs") {
                Some(NixValue::List(l)) => l.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let mut iig_all_outputs = Vec::new();
    for job_val in &iig_jobs_map {
        if let NixValue::AttributeSet(job) = job_val {
            let def_file = get_str_attr(job, "defFile").unwrap_or("");
            let header_out = get_str_attr(job, "headerOut").unwrap_or("");
            let cpp_out = get_str_attr(job, "cppOut").unwrap_or("");
            if !def_file.is_empty() && !cpp_out.is_empty() {
                out.push_str(&format!(
                    "build {} {}: iig {} | build/gen/master_config.h\n  header_out = {}\n  impl_out = {}\n  iigflags = -x c++ -std=gnu++2c -D__IIG=1 -DDRIVERKIT_PRIVATE=1 -DPRIVATE_WIFI_ONLY=1 -Iiokit -Iosfmk -Ibsd -IEXTERNAL_HEADERS -Ibuild/gen -Ibuild/gen/export -Ibuild/syscalls\n\n",
                    cpp_out, header_out, def_file, header_out, cpp_out
                ));
                iig_all_outputs.push(cpp_out.to_string());
                iig_all_outputs.push(header_out.to_string());
            }
        }
    }

    // Assym
    out.push_str("build build/assym/genassym.s: genassym osfmk/arm64/genassym.c | build/gen/master_config.h build/gen/meta_features.h\n");
    let flags_map = match root_map.get("flags") {
        Some(NixValue::AttributeSet(m)) => m,
        _ => return Err(anyhow!("Missing flags")),
    };
    let common_c_flags = get_list_strings(flags_map.get("commonCFlags"));
    let common_incs = get_list_strings(flags_map.get("commonIncludes"));
    let genassym_cflags = format!(
        "{} -DMACH_KERNEL_PRIVATE -DMACH_KERNEL -include gen/meta_features.h {} -Iosfmk/libsa -std=gnu17",
        common_c_flags.join(" "),
        common_incs.join(" ")
    );
    out.push_str(&format!("  cflags = {}\n\n", genassym_cflags));

    out.push_str("build build/assym/assym.s: extract_assym build/assym/genassym.s | tools/pcons/extract_assym.py\n\n");

    // Phony target for all generated headers and code
    let mut header_deps = vec![
        "build/gen/master_config.h".to_string(),
        "build/gen/meta_features.h".to_string(),
        "build/assym/assym.s".to_string(),
        "build/syscalls/sysproto.h".to_string(),
        "build/version/version.c".to_string(),
    ];
    header_deps.extend(mig_all_outputs);
    header_deps.extend(iig_all_outputs);

    out.push_str(&format!(
        "build build/headers_ready: phony {}\n\n",
        header_deps.join(" ")
    ));

    // Component compilation & static libraries
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("# Component Static Libraries\n");
    out.push_str("# ------------------------------------------------------------\n");

    let mut all_archive_targets = Vec::new();
    let components = match root_map.get("components") {
        Some(NixValue::List(l)) => l,
        _ => return Err(anyhow!("Missing components list")),
    };

    let base_implicit_deps = "build/headers_ready";

    for comp_val in components {
        if let NixValue::AttributeSet(comp) = comp_val {
            let name = get_str_attr(comp, "name").unwrap_or("unknown");
            let c_srcs = get_list_strings(comp.get("cSources"));
            let cpp_srcs = get_list_strings(comp.get("cppSources"));
            let s_srcs = get_list_strings(comp.get("sSources"));
            let cflags = get_list_strings(comp.get("cflags")).join(" ");
            let cxxflags = get_list_strings(comp.get("cxxflags")).join(" ");
            let asmflags = get_list_strings(comp.get("asmflags")).join(" ");

            let mut obj_files = Vec::new();

            for src in &c_srcs {
                let clean_src = src.replace('/', "_").replace('.', "_");
                let obj = format!("build/obj/{}/{}.o", name, clean_src);
                out.push_str(&format!(
                    "build {}: cc {} || {}\n  cflags = {}\n",
                    obj, src, base_implicit_deps, cflags
                ));
                obj_files.push(obj);
            }

            for src in &cpp_srcs {
                let clean_src = src.replace('/', "_").replace('.', "_");
                let obj = format!("build/obj/{}/{}.o", name, clean_src);
                out.push_str(&format!(
                    "build {}: cxx {} || {}\n  cxxflags = {}\n",
                    obj, src, base_implicit_deps, cxxflags
                ));
                obj_files.push(obj);
            }

            for src in &s_srcs {
                let clean_src = src.replace('/', "_").replace('.', "_");
                let obj = format!("build/obj/{}/{}.o", name, clean_src);
                out.push_str(&format!(
                    "build {}: asm {} || {}\n  asmflags = {}\n",
                    obj, src, base_implicit_deps, asmflags
                ));
                obj_files.push(obj);
            }

            if !obj_files.is_empty() {
                let archive = format!("build/lib{}.a", name);
                out.push_str(&format!(
                    "\nbuild {}: ar {}\n\n",
                    archive,
                    obj_files.join(" ")
                ));
                all_archive_targets.push(archive);
            }
        }
    }

    // Cargo targets
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("# Cargo Built Subsystems\n");
    out.push_str("# ------------------------------------------------------------\n");

    out.push_str("build build/qemu-boot.elf: cargo_build tools/qemu-boot/Cargo.toml\n");
    out.push_str("  manifest = tools/qemu-boot/Cargo.toml\n  target_dir = build/cargo/qemu_boot\n  cargo_flags = --target aarch64-unknown-none --release\n  artifact = build/cargo/qemu_boot/aarch64-unknown-none/release/qemu-boot\n\n");

    // Kernel Link
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("# Final Kernel Executable and KernelCache\n");
    out.push_str("# ------------------------------------------------------------\n");

    // Ensure libsa (lastkernelconstructor) is strictly the last library linked
    all_archive_targets.sort_by_key(|t| if t.contains("libsa") { 1 } else { 0 });

    let ld_flags = get_list_strings(flags_map.get("ldFlags")).join(" ");
    out.push_str(&format!(
        "build build/mach_kernel: link_kernel {}\n  ldflags = {}\n\n",
        all_archive_targets.join(" "),
        ld_flags
    ));

    // Init binary (Freestanding Userspace executable for mockfs ramdisk)
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("# Freestanding Userspace Init Binary (mockfs)\n");
    out.push_str("# ------------------------------------------------------------\n");
    out.push_str("rule init_cc\n");
    out.push_str("  command = clang -target arm64-apple-macos -ffreestanding -nostdlib -static -e __start $in -o $out\n");
    out.push_str("  description = CC (init) $out\n\n");
    out.push_str("build build/init: init_cc userspace/init/init.c\n\n");

    out.push_str(
        "build build/kernelcache: kernelcache build/mach_kernel | tools/pcons/kernelcache.py\n\n",
    );
    out.push_str("default build/kernelcache build/qemu-boot.elf build/init\n");

    if let Ok(existing) = fs::read_to_string("build/build.ninja") {
        if existing != out {
            fs::write("build/build.ninja", out)?;
        }
    } else {
        fs::write("build/build.ninja", out)?;
    }
    println!(
        "Generated build/build.ninja with rix in {:.2}ms (board={}, config={})",
        t0.elapsed().as_secs_f64() * 1000.0,
        board,
        config
    );

    Ok(())
}

fn run_build(board: &str, config: &str, werror: bool, ninja_args: &[String]) -> Result<()> {
    generate_ninja(board, config, werror)?;
    let status = Command::new("ninja")
        .arg("-f")
        .arg("build/build.ninja")
        .args(ninja_args)
        .status()
        .context("Failed to run ninja")?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }
    Ok(())
}

fn run_qemu(board: &str, config: &str, werror: bool) -> Result<()> {
    run_build(board, config, werror, &[])?;

    let qemu_bin = if Path::new("/Users/theo/.nix-profile/bin/qemu-system-aarch64").exists() {
        "/Users/theo/.nix-profile/bin/qemu-system-aarch64"
    } else {
        "qemu-system-aarch64"
    };

    let mut cmd = Command::new(qemu_bin);
    cmd.args([
        "-M",
        "virt",
        "-cpu",
        "cortex-a53",
        "-m",
        "1024",
        "-smp",
        "2",
        "-nographic",
        "-kernel",
        "build/qemu-boot.elf",
        "-device",
        "loader,force-raw=on,addr=0x48000000,file=build/kernelcache",
    ]);

    if Path::new("build/init").exists() {
        cmd.args([
            "-device",
            "loader,force-raw=on,addr=0x50000000,file=build/init",
        ]);
    }

    if Path::new("build/rootfs.fat32").exists() {
        cmd.args([
            "-drive",
            "file=build/rootfs.fat32,if=none,format=raw,id=hd0",
            "-device",
            "virtio-blk-device,drive=hd0",
        ]);
    }

    println!("Starting QEMU with rix-generated kernelcache...");
    let status = cmd.status().context("Failed to start QEMU")?;
    if !status.success() {
        return Err(anyhow!("QEMU exited with error"));
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate { board, config }) => {
            generate_ninja(&board, &config, cli.werror)?;
        }
        Some(Commands::Build {
            board,
            config,
            ninja_args,
        }) => {
            run_build(&board, &config, cli.werror, &ninja_args)?;
        }
        Some(Commands::Qemu { board, config }) => {
            run_qemu(&board, &config, cli.werror)?;
        }
        None => {
            generate_ninja(&cli.board, &cli.config, cli.werror)?;
        }
    }

    Ok(())
}
