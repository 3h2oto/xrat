use sha2::{Digest, Sha256};
use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};

pub struct Engines {
    pub xray: PathBuf,
    pub singbox: PathBuf,
    _directory: tempfile::TempDir,
}

struct Pin {
    name: &'static str,
    version: &'static str,
    url: &'static str,
    sha256: &'static str,
}

fn pins() -> [Pin; 2] {
    if cfg!(target_arch = "aarch64") {
        [
            Pin {
                name: "xray",
                version: "26.9.30",
                url: "https://github.com/XTLS/Xray-core/releases/download/v26.9.30/Xray-windows-arm64-v8a.zip",
                sha256: "5cf7ad4fd9aac5aa248ae7e598f3f0a0d0f79a6195089d611ddef9653830f1b0",
            },
            Pin {
                name: "sing-box",
                version: "1.13.21",
                url: "https://github.com/SagerNet/sing-box/releases/download/v1.13.21/sing-box-1.13.21-windows-arm64.zip",
                sha256: "ef752d9bffd6d590dd6886b28819a7ae4efee70b8188f9198757d91570efd554",
            },
        ]
    } else {
        [
            Pin {
                name: "xray",
                version: "26.9.30",
                url: "https://github.com/XTLS/Xray-core/releases/download/v26.9.30/Xray-windows-64.zip",
                sha256: "b17a619343c11b89d8faf36278298856749b15c52277d875d32051b33dcda617",
            },
            Pin {
                name: "sing-box",
                version: "1.13.21",
                url: "https://github.com/SagerNet/sing-box/releases/download/v1.13.21/sing-box-1.13.21-windows-amd64.zip",
                sha256: "a03291793d3a3c6e266447a58140657ac099ff278abf3b8ff678932356a62ced",
            },
        ]
    }
}

fn install(directory: &Path) -> [PathBuf; 2] {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap();
    let mut receipts = Vec::new();
    let binaries = pins().map(|pin| {
        let bytes = client
            .get(pin.url)
            .send()
            .unwrap()
            .error_for_status()
            .unwrap()
            .bytes()
            .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            pin.sha256,
            "{} archive checksum mismatch",
            pin.name
        );
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let name = format!("{}.exe", pin.name);
        let matches = (0..archive.len())
            .filter(|index| {
                archive
                    .by_index(*index)
                    .unwrap()
                    .enclosed_name()
                    .is_some_and(|path| {
                        path.file_name()
                            .is_some_and(|leaf| leaf.to_string_lossy().eq_ignore_ascii_case(&name))
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one {} executable",
            pin.name
        );
        let mut entry = archive.by_index(matches[0]).unwrap();
        assert!(entry.size() > 0 && entry.size() <= 256 * 1024 * 1024);
        let path = directory.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        std::io::copy(&mut entry, &mut file).unwrap();
        file.flush().unwrap();
        drop(file);
        receipts.push(serde_json::json!({
            "engine":pin.name,"version":pin.version,"architecture":std::env::consts::ARCH,
            "source":pin.url,"archive_sha256":pin.sha256,"executable":path,
            "executable_sha256":format!("{:x}", Sha256::digest(std::fs::read(&path).unwrap()))
        }));
        path
    });
    let receipt = std::env::var_os("XRAT_WINDOWS_ENGINE_RECEIPT")
        .map(PathBuf::from)
        .unwrap_or_else(|| directory.join("receipts.json"));
    if let Some(parent) = receipt.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(receipt, serde_json::to_vec_pretty(&receipts).unwrap()).unwrap();
    binaries
}

pub fn paths() -> &'static Engines {
    static ENGINES: OnceLock<Engines> = OnceLock::new();
    ENGINES.get_or_init(|| {
        let directory = tempfile::Builder::new()
            .prefix("xrat engines م ")
            .tempdir()
            .unwrap();
        let [xray, singbox] = install(directory.path());
        Engines {
            xray,
            singbox,
            _directory: directory,
        }
    })
}
