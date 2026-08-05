//! spd-cli: antarmuka terminal untuk backend spd-core.
//!
//! Menjalankan fitur backup/restore dari terminal (tanpa UI), memakai
//! transport mock (untuk tes) atau libusb asli di fase berikutnya.

use std::path::PathBuf;
use std::process::ExitCode;

use app_core::tool::ToolAdapter;
use spd_core::chipdb::ChipConfig;
use spd_core::core::{Flags, Session, DEFAULT_BLK_SIZE};
use spd_core::transport::mock::{part, MockTransport};

const USAGE: &str = "\
spd-cli — tool backup/restore firmware Unisoc (BROM/FDL)

Usage:
  spd-cli <command> [options]

Commands:
  info           Tampilkan info chipset & versi.
  detect         Deteksi perangkat ADB / Fastboot / Unisoc.
  adb-info [sn]  Info perangkat via getprop (serial opsional, default pertama).
  connect        Handshake BROM + boot FDL1/FDL2 (dengan mock).
  list           Ambil daftar partisi.
  dump <name>    Dump satu partisi ke <name>.bin
  dump-all       Dump semua partisi ke folder images/ (mock)
  write <name> <file>
                 Tulis file ke partisi (mock).
  reset          Reset perangkat ke mode normal.
  poweroff       Matikan perangkat.

Options:
  --chip <name>  Pilih chipset (default ums9230).
  --mock         Pakai transport mock (tanpa hardware).

Examples:
  spd-cli info
  spd-cli detect
  spd-cli list --mock
  spd-cli dump boot --mock
";

#[derive(Debug)]
struct Cli {
    command: String,
    args: Vec<String>,
    chip: ChipConfig,
    mock: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[error] {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> spd_core::Result<()> {
    if args.is_empty() || args[0] == "help" || args[0] == "--help" {
        println!("{USAGE}");
        return Ok(());
    }

    let command = args[0].clone();
    let mut chip = ChipConfig::ums9230();
    let mut mock = false;
    let mut rest: Vec<String> = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--chip" => {
                i += 1;
                if i >= args.len() {
                    return Err(spd_core::Error::Other("--chip needs a name".into()));
                }
                chip = match args[i].as_str() {
                    "ums9230" => ChipConfig::ums9230(),
                    other => {
                        return Err(spd_core::Error::Other(format!(
                            "unknown chip: {other}"
                        )))
                    }
                };
            }
            "--mock" => mock = true,
            other => rest.push(other.to_string()),
        }
        i += 1;
    }
    let cli = Cli {
        command,
        args: rest,
        chip,
        mock,
    };

    if cli.command == "info" {
        println!("chip: {}", cli.chip.name);
        println!("exec_addr: 0x{:08x}", cli.chip.exec_addr);
        println!("fdl1_addr: 0x{:08x}", cli.chip.fdl1_addr);
        println!("fdl2_addr: 0x{:08x}", cli.chip.fdl2_addr);
        println!("transport: mock");
        return Ok(());
    }

    if cli.command == "detect" {
        return detect_cmd().map_err(to_spd_err);
    }

    if cli.command == "adb-info" {
        return adb_info_cmd(cli.args.first()).map_err(to_spd_err);
    }

    let t: Box<dyn spd_core::Transport> = if cli.mock {
        Box::new(MockTransport::new(mock_partitions()))
    } else {
        return Err(spd_core::Error::Other(
            "transport libusb asli belum diimplementasikan; pakai --mock".into(),
        ));
    };
    let mut session = Session::new(t, Flags::brom());

    match cli.command.as_str() {
        "info" => {
            // sudah ditangani di atas
            unreachable!()
        }
        "connect" => {
            session.verbose = true;
            let fdl1 = vec![0xAA; 1024];
            let fdl2 = vec![0xBB; 1024];
            let ver = session.boot(
                &fdl1,
                cli.chip.fdl1_addr,
                &fdl2,
                cli.chip.fdl2_addr,
                None,
            )?;
            println!("connected, BROM ver: {ver}");
            Ok(())
        }
        "list" => {
            let parts = session.partition_list()?;
            println!("{:<20} {:>14}", "name", "size");
            for p in &parts {
                println!("{:<20} {:>14}", p.name, p.size);
            }
            Ok(())
        }
        "dump" => {
            let name = cli
                .args
                .first()
                .cloned()
                .ok_or_else(|| spd_core::Error::Other("dump needs a part name".into()))?;
            let parts = session.partition_list()?;
            let size = parts
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.size)
                .ok_or_else(|| {
                    spd_core::Error::Other(format!("partition not found: {name}"))
                })?;
            let fname = format!("{name}.bin");
            let mut f = std::fs::File::create(&fname)?;
            let read = session.dump_partition(&name, 0, size, DEFAULT_BLK_SIZE, &mut f)?;
            println!("dumped {read} bytes to {fname}");
            Ok(())
        }
        "dump-all" => {
            let dir = PathBuf::from("images");
            std::fs::create_dir_all(&dir)?;
            let parts = session.partition_list()?;
            session.dump_all(&parts, |name| {
                std::fs::File::create(dir.join(format!("{name}.bin"))).map_err(Into::into)
            })?;
            println!("dumped {} partitions to images/", parts.len());
            Ok(())
        }
        "write" => {
            if cli.args.len() < 2 {
                return Err(spd_core::Error::Other(
                    "write needs: <name> <file>".into(),
                ));
            }
            let name = &cli.args[0];
            let data = std::fs::read(&cli.args[1])?;
            session.write_partition(name, &data, DEFAULT_BLK_SIZE)?;
            println!("wrote {} bytes to {name}", data.len());
            Ok(())
        }
        "reset" => {
            session.reset()?;
            println!("reset sent");
            Ok(())
        }
        "poweroff" => {
            session.poweroff()?;
            println!("poweroff sent");
            Ok(())
        }
        other => Err(spd_core::Error::Other(format!(
            "unknown command: {other}\n{USAGE}"
        ))),
    }
}

/// Partisi tiruan untuk mode mock.
fn mock_partitions() -> Vec<spd_core::proto::brom::Partition> {
    vec![
        part("prodnv", 64),
        part("miscdata", 1),
        part("boot_a", 64),
        part("boot_b", 64),
        part("nv", 2),
        part("super", 8000),
    ]
}

/// Konversi error app-core menjadi spd-core (CLI memakai satu tipe error).
fn to_spd_err(e: app_core::Error) -> spd_core::Error {
    spd_core::Error::Other(e.to_string())
}

/// Deteksi perangkat: ADB → Fastboot. Unisoc (handshake BROM) fase berikutnya.
fn detect_cmd() -> app_core::Result<()> {
    let adb = adb::AdbAdapter::new(tool_binary("tools/adb/adb", adb::ADB_DEFAULT_BINARY));
    let fastboot = fastboot::FastbootAdapter::new(tool_binary(
        "tools/fastboot/fastboot",
        fastboot::FASTBOOT_DEFAULT_BINARY,
    ));
    let mgr = app_core::DeviceManager::new()
        .with_adb(Box::new(adb))
        .with_fastboot(Box::new(fastboot));

    let s = mgr.scan();
    for w in &s.warnings {
        eprintln!("[warn] {w}");
    }
    println!(
        "devices: {} (scan {} ms)",
        s.devices.len(),
        s.last_scan_ms
    );
    for d in &s.devices {
        println!("  {:<20} {:<14} {}", d.serial, d.state, d.mode.label());
    }
    Ok(())
}

/// Resolusi path binary tool: prefer `tools/<tool>/<tool>` jika ada, fallback ke
/// binary di PATH.
fn tool_binary(rel: &str, fallback: &str) -> String {
    for cand in [format!("{rel}.exe"), rel.to_string()] {
        if std::path::Path::new(&cand).is_file() {
            return cand;
        }
    }
    fallback.to_string()
}

/// Info perangkat ADB via getprop. Tanpa argumen serial, pakai device ADB pertama.
fn adb_info_cmd(serial: Option<&String>) -> app_core::Result<()> {
    let adapter = adb::AdbAdapter::new(tool_binary("tools/adb/adb", adb::ADB_DEFAULT_BINARY));

    let serial = match serial {
        Some(s) => s.clone(),
        None => {
            let devices = adapter.detect()?;
            let serial = devices
                .iter()
                .find(|d| d.state == "device")
                .map(|d| d.serial.clone())
                .ok_or_else(|| app_core::Error::Other("tidak ada device ADB aktif; berikan serial".into()))?;
            println!("using device: {serial}");
            serial
        }
    };

    let info = adapter.device_info(&serial)?;
    println!("serial:          {}", info.serial);
    println!("manufacturer:    {}", fmt_opt(&info.manufacturer));
    println!("model:           {}", fmt_opt(&info.model));
    println!("productName:     {}", fmt_opt(&info.product_name));
    println!("codename:        {}", fmt_opt(&info.codename));
    println!("brand:           {}", fmt_opt(&info.brand));
    println!("androidVersion:  {}", fmt_opt(&info.android_version));
    println!("buildFingerprint:{}", fmt_opt(&info.build_fingerprint));
    println!("securityPatch:   {}", fmt_opt(&info.security_patch));
    println!("sdk:             {}", fmt_opt(&info.sdk));
    Ok(())
}

fn fmt_opt(v: &Option<String>) -> &str {
    v.as_deref().unwrap_or("-")
}
