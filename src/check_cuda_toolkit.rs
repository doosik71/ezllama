use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn run() {
    match cuda_toolkit_version() {
        Some(version) => {
            println!("CUDA Toolkit이 설치되어 있습니다.");
            println!("버전: {version}");
        }
        None => {
            println!("CUDA Toolkit이 설치되어 있지 않습니다.");

            let install_plan = install_plan();
            println!("설치 명령:");
            println!("{}", install_plan.message);

            if let Some(command) = install_plan.command {
                if ask_yes_no("CUDA Toolkit을 설치할까요? [y/N]: ") {
                    match run_install_command(command) {
                        Ok(()) => println!("설치 명령을 실행했습니다."),
                        Err(error) => eprintln!("설치 명령 실행 실패: {error}"),
                    }
                } else {
                    println!("설치를 건너뜁니다.");
                }
            } else {
                println!("자동 실행할 설치 명령이 없어서 설치를 진행하지 않습니다.");
            }
        }
    }
}

fn cuda_toolkit_version() -> Option<String> {
    let output = Command::new("nvcc").arg("--version").output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    parse_cuda_version(&text)
}

fn parse_cuda_version(output: &str) -> Option<String> {
    let release_marker = "release ";
    let start = output.find(release_marker)? + release_marker.len();
    let rest = &output[start..];
    let end = rest.find(',').unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

struct InstallPlan {
    message: &'static str,
    command: Option<&'static str>,
}

fn install_plan() -> InstallPlan {
    if command_exists("apt", &["--version"]) {
        InstallPlan {
            message: "sudo apt update && sudo apt install -y nvidia-cuda-toolkit",
            command: Some("sudo apt update && sudo apt install -y nvidia-cuda-toolkit"),
        }
    } else if command_exists("dnf", &["--version"]) {
        InstallPlan {
            message: "sudo dnf install -y cuda-toolkit",
            command: Some("sudo dnf install -y cuda-toolkit"),
        }
    } else if command_exists("yum", &["--version"]) {
        InstallPlan {
            message: "sudo yum install -y cuda-toolkit",
            command: Some("sudo yum install -y cuda-toolkit"),
        }
    } else if command_exists("pacman", &["-V"]) {
        InstallPlan {
            message: "sudo pacman -S --noconfirm cuda",
            command: Some("sudo pacman -S --noconfirm cuda"),
        }
    } else if command_exists("zypper", &["--version"]) {
        InstallPlan {
            message: "sudo zypper install -y cuda-toolkit",
            command: Some("sudo zypper install -y cuda-toolkit"),
        }
    } else if command_exists("brew", &["--version"]) {
        InstallPlan {
            message: "brew install --cask cuda",
            command: Some("brew install --cask cuda"),
        }
    } else {
        InstallPlan {
            message: "설치 명령을 자동으로 결정할 수 없습니다. 사용 중인 배포판의 CUDA Toolkit 설치 명령을 확인해 주세요.",
            command: None,
        }
    }
}

fn command_exists(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn ask_yes_no(prompt: &str) -> bool {
    print!("{prompt}");
    if io::stdout().flush().is_err() {
        return false;
    }

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "예" | "ㅇ")
}

fn run_install_command(command: &str) -> io::Result<()> {
    let status = Command::new("sh").arg("-c").arg(command).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("종료 코드: {status}"),
        ))
    }
}
