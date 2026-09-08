use std::env;
use std::process::ExitCode;
use std::path::PathBuf;

// binwalk 라이브러리 모듈
use binwalk::Binwalk;

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

    // 2. -M 로직 (Matryoshka - 재귀적 분석)
    eprintln!("\n=== [2] Matryoshka (재귀적 분석) ===");
    if let Err(e) = run_matryoshka(file_path) {
        eprintln!("[오류] Matryoshka 스캔 실패: {}", e);
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

/// 기본 스캔 (옵션 없음)
fn run_basic_scan(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_file = Some(PathBuf::from(file_path));
    let output_dir = Some(PathBuf::from("extractions"));
    
    let bw = Binwalk::configure(
        target_file,
        output_dir,
        None,  // include filters
        None,  // exclude filters
        None,  // custom signatures
        false, // quiet
    )?;
    
    bw.scan()?;
    Ok(())
}

/// -M 로직 (Matryoshka - 재귀적 분석)
fn run_matryoshka(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_file = Some(PathBuf::from(file_path));
    let output_dir = Some(PathBuf::from("extractions"));
    
    let bw = Binwalk::configure(
        target_file,
        output_dir,
        None,  // include filters
        None,  // exclude filters
        None,  // custom signatures
        false, // quiet
    )?;
    
    bw.matryoshka_scan()?;
    Ok(())
}

/// -e 로직 (추출)
fn run_extract(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target_file = Some(PathBuf::from(file_path));
    let output_dir = Some(PathBuf::from("extractions"));
    
    let bw = Binwalk::configure(
        target_file,
        output_dir,
        None,  // include filters
        None,  // exclude filters
        None,  // custom signatures
        false, // quiet
    )?;
    
    bw.extract()?;
    Ok(())
}
