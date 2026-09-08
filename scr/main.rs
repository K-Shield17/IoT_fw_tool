use std::env;
use std::process::ExitCode;

// binwalk 라이브러리 모듈 (기존 구조 그대로 사용)
use binwalk::{Binwalk, ScanOptions};

fn main() -> ExitCode {
    // 인자 파싱: 오직 펌웨어 파일 경로 하나만 받음
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("사용법: {} <firmware.bin>", args[0]);
        return ExitCode::FAILURE;
    }
    let file_path = &args[1];

    // 1. 기본 스캔
    eprintln!("=== [1] 기본 스캔 ===");
    if let Err(e) = run_basic_scan(file_path) {
        eprintln!("[오류] 기본 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    // 2. -M 로직 (매직/시그니처 상세 분석)
    eprintln!("\n=== [2] 매직/시그니처 상세 분석 ===");
    if let Err(e) = run_magic_scan(file_path) {
        eprintln!("[오류] 매직 스캔 실패: {}", e);
        return ExitCode::FAILURE;
    }

    // 3. -e 로직 (추출)
    eprintln!("\n=== [3] 추출 ===");
    if let Err(e) = run_extract(file_path) {
        eprintln!("[오류] 추출 실패: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// 기본 스캔 (기존 binwalk 로직 재사용)
fn run_basic_scan(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut bw = Binwalk::new(ScanOptions {
        path: file_path.into(),
        extract: false,
        magic: false,
        verbose: false,
        // 기타 옵션들은 기본값 사용
        ..ScanOptions::default()
    });
    bw.scan()?;
    Ok(())
}

/// -M 로직 (매직/시그니처 상세 분석)
fn run_magic_scan(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut bw = Binwalk::new(ScanOptions {
        path: file_path.into(),
        extract: false,
        magic: true,
        verbose: false,
        ..ScanOptions::default()
    });
    bw.scan()?;
    Ok(())
}

/// -e 로직 (추출)
fn run_extract(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut bw = Binwalk::new(ScanOptions {
        path: file_path.into(),
        extract: true,
        magic: false,
        verbose: false,
        ..ScanOptions::default()
    });
    bw.scan()?;
    Ok(())
}
