use cucumber::{gherkin::Step, given, then, when, World};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;
use tokio::process::Command as AsyncCommand;

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

#[derive(Debug, World)]
#[world(init = DocCheckWorld::new)]
pub struct DocCheckWorld {
    _tmp: TempDir,
    pub work_dir: PathBuf,
    pub checkout_dir: Option<PathBuf>,
    pub last_stdout: String,
    pub last_stderr: String,
    pub last_exit_code: i32,
    background_process: Option<std::process::Child>,
}

impl Drop for DocCheckWorld {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.background_process {
            // Kill the entire process group so grandchildren (e.g. fava) also die.
            #[cfg(unix)]
            {
                let pgid = child.id();
                let _ = Command::new("kill")
                    .args(["-9", &format!("-{pgid}")])
                    .status();
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl DocCheckWorld {
    async fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let work_dir = tmp
            .path()
            .canonicalize()
            .unwrap_or_else(|_| tmp.path().to_path_buf());
        Self {
            _tmp: tmp,
            work_dir,
            checkout_dir: None,
            last_stdout: String::new(),
            last_stderr: String::new(),
            last_exit_code: 0,
            background_process: None,
        }
    }

    fn resolve(&self, path: &str) -> PathBuf {
        self.work_dir.join(path)
    }

    async fn run_args(&mut self, raw: &[String], cwd: &Path, extra_env: &[(&str, &str)]) {
        let bin = PathBuf::from(env!("CARGO_BIN_EXE_hledger-document-check"));
        let (program, args) = if raw.len() >= 2 && raw[0] == "hledger" && raw[1] == "document-check"
        {
            (bin, raw[2..].to_vec())
        } else if !raw.is_empty() && raw[0] == "hledger-document-check" {
            (bin, raw[1..].to_vec())
        } else {
            (PathBuf::from(&raw[0]), raw[1..].to_vec())
        };

        let mut cmd = AsyncCommand::new(program);
        cmd.args(&args).current_dir(cwd);
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let output = cmd.output().await.expect("failed to run command");
        self.last_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        self.last_stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        self.last_exit_code = output.status.code().unwrap_or(-1);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// The gherkin 0.14 crate stores the raw docstring value including the
// content-type hint (e.g. "yaml\n...") and does not strip indentation.
// This function strips the content-type first line and common leading whitespace,
// matching the behaviour of Python behave's `context.text`.
fn docstring(step: &Step) -> String {
    let raw = step.docstring.as_deref().unwrap_or("");
    let lines: Vec<&str> = raw.split('\n').collect();

    // Skip the content-type hint line if it is a single non-empty word.
    let start = if lines
        .first()
        .map(|l| {
            let t = l.trim();
            !t.is_empty() && !t.contains(' ')
        })
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    let content = &lines[start..];

    // Find the minimum indentation of non-empty lines.
    let min_indent = content
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    // Strip common indentation and reassemble.
    let result = content
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Trim trailing newlines so callers control the terminator.
    result.trim_end_matches('\n').to_string()
}

fn interpolate(text: &str, world: &DocCheckWorld) -> String {
    let work_dir = world.work_dir.display().to_string();
    let checkout_dir = world
        .checkout_dir
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    text.replace("{work_dir}", &work_dir)
        .replace("{checkout_dir}", &checkout_dir)
}

fn contains_in_order(text: &str, pattern: &str) -> bool {
    let chunks = ellipsis_chunks(pattern);
    let mut pos = 0;
    for chunk in &chunks {
        if chunk.is_empty() {
            continue;
        }
        match text[pos..].find(chunk.as_str()) {
            Some(idx) => pos += idx + chunk.len(),
            None => return false,
        }
    }
    true
}

fn ellipsis_chunks(pattern: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in pattern.lines() {
        if line.trim() == "..." {
            chunks.push(current.join("\n").trim_matches('\n').to_string());
            current.clear();
        } else {
            current.push(line);
        }
    }
    chunks.push(current.join("\n").trim_matches('\n').to_string());
    chunks
}

fn minimal_pdf(text: &str) -> Vec<u8> {
    let escaped = text
        .replace('\\', r"\\")
        .replace('(', r"\(")
        .replace(')', r"\)");
    let stream = format!("BT\n/F1 12 Tf\n72 720 Td\n({escaped}) Tj\nET");
    let objects: Vec<String> = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".into(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R \
         /Resources << /Font << /F1 5 0 R >> >> >>"
            .into(),
        format!(
            "<< /Length {} >>\nstream\n{stream}\nendstream",
            stream.len()
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    let mut body = "%PDF-1.4\n".to_string();
    let mut offsets = vec![0usize];
    for (i, obj) in objects.iter().enumerate() {
        offsets.push(body.len());
        body.push_str(&format!("{} 0 obj\n{obj}\nendobj\n", i + 1));
    }
    let xref_offset = body.len();
    let xref_entries: Vec<String> = std::iter::once("0000000000 65535 f ".to_string())
        .chain(
            offsets[1..]
                .iter()
                .map(|&off| format!("{off:010} 00000 n ")),
        )
        .collect();
    body.push_str(&format!(
        "xref\n0 {}\n{}\n",
        objects.len() + 1,
        xref_entries.join("\n")
    ));
    body.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
        objects.len() + 1
    ));
    body.into_bytes()
}

fn copy_dir_all(src: &Path, dst: &Path, ignore: &[&str]) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap().flatten() {
        let name = entry.file_name();
        if ignore.contains(&name.to_str().unwrap_or("")) {
            continue;
        }
        let dst_path = dst.join(&name);
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&entry.path(), &dst_path, ignore);
        } else {
            fs::copy(entry.path(), dst_path).unwrap();
        }
    }
}

// ---------------------------------------------------------------------------
// Given steps
// ---------------------------------------------------------------------------

#[given(regex = r#"^a file named "([^"]+)" with content:$"#)]
async fn write_file(world: &mut DocCheckWorld, step: &Step, path: String) {
    let content = docstring(step);
    let target = world.resolve(&path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, format!("{content}\n")).unwrap();
}

#[given("files named:")]
async fn write_files(world: &mut DocCheckWorld, step: &Step) {
    let table = step.table.as_ref().expect("step requires a table");
    let headers = &table.rows[0];
    let path_col = headers.iter().position(|h| h == "path").unwrap();
    let content_col = headers.iter().position(|h| h == "content").unwrap();
    for row in table.rows.iter().skip(1) {
        let path = &row[path_col];
        let content = row[content_col].replace("\\n", "\n");
        let target = world.resolve(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, format!("{content}\n")).unwrap();
    }
}

#[given(regex = r#"^a PDF file named "([^"]+)" with text:$"#)]
async fn write_pdf_file(world: &mut DocCheckWorld, step: &Step, path: String) {
    let text = docstring(step);
    let bytes = minimal_pdf(text.trim());
    let target = world.resolve(&path);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, bytes).unwrap();
}

#[given(regex = r#"^an empty directory named "([^"]+)"$"#)]
async fn create_empty_dir(world: &mut DocCheckWorld, path: String) {
    fs::create_dir_all(world.resolve(&path)).unwrap();
}

#[given("the development dependencies are installed")]
async fn check_dev_deps(_world: &mut DocCheckWorld) {
    for cmd in &["just", "hledger"] {
        let ok = Command::new(cmd)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(ok, "{cmd} is not available");
    }
}

#[given("the optional Fava dependency is installed")]
async fn check_fava(_world: &mut DocCheckWorld) {
    let ok = Command::new("fava")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        ok,
        "fava is not installed; install it with: pip install fava"
    );
}

// ---------------------------------------------------------------------------
// When steps
// ---------------------------------------------------------------------------

#[when(regex = r#"^I run "([^"]+)"$"#)]
async fn run_command(world: &mut DocCheckWorld, command: String) {
    let args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    let cwd = world.work_dir.clone();
    world.run_args(&args, &cwd, &[]).await;
}

#[when("I clone the repository")]
async fn clone_repository(world: &mut DocCheckWorld) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dst = world.work_dir.join("hledger-document-check");
    copy_dir_all(
        src,
        &dst,
        &["target", ".git", ".venv", ".uv-cache", "__pycache__"],
    );
    world.checkout_dir = Some(dst);
}

#[when(regex = r#"^I run `([^`]+)` from the root directory$"#)]
async fn run_from_root(world: &mut DocCheckWorld, command: String) {
    let cwd = world
        .checkout_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    world.run_args(&args, &cwd, &[]).await;
}

#[when("I run a shell command from the root directory:")]
async fn run_shell_command_from_root(world: &mut DocCheckWorld, step: &Step) {
    let cwd = world
        .checkout_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let script = docstring(step);
    // Put the test binary's directory first on PATH so `hledger-document-check`
    // in the script resolves to the compiled test binary.
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_hledger-document-check"));
    let bin_dir = bin
        .parent()
        .unwrap_or(Path::new("."))
        .to_string_lossy()
        .to_string();
    let path_env = format!("{bin_dir}:{}", std::env::var("PATH").unwrap_or_default());
    world
        .run_args(
            &["bash".to_string(), "-c".to_string(), script],
            &cwd,
            &[("PATH", &path_env)],
        )
        .await;
}

#[when(regex = r#"^I start `([^`]+)` from the root directory$"#)]
async fn start_background(world: &mut DocCheckWorld, command: String) {
    use std::os::unix::process::CommandExt as _;
    let cwd = world
        .checkout_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
    let target_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let child = Command::new(&args[0])
        .args(&args[1..])
        .current_dir(&cwd)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("CARGO_TERM_QUIET", "true")
        .process_group(0)
        .spawn()
        .expect("failed to start background process");
    world.background_process = Some(child);
}

// ---------------------------------------------------------------------------
// Then steps
// ---------------------------------------------------------------------------

#[then(regex = r#"^Fava is running at "([^"]+)"$"#)]
async fn fava_is_running(_world: &mut DocCheckWorld, url: String) {
    for _ in 0..120 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let ok = Command::new("curl")
            .args(["-sf", "--max-time", "2", &url])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return;
        }
    }
    panic!("Fava did not start at {url} within 120 seconds");
}

#[then(regex = r"^(?:the command exits with code|the exit code is) (\d+)$")]
async fn exit_code(world: &mut DocCheckWorld, code: i32) {
    assert_eq!(
        world.last_exit_code, code,
        "exit code mismatch\nstdout:\n{}\nstderr:\n{}",
        world.last_stdout, world.last_stderr,
    );
}

#[then(regex = r"^stdout (?:should )?contains?:$")]
async fn stdout_contains(world: &mut DocCheckWorld, step: &Step) {
    let expected = interpolate(&docstring(step), world);
    if expected.contains("...") {
        assert!(
            contains_in_order(&world.last_stdout, &expected),
            "stdout did not contain expected chunks in order:\n{expected}\n\nActual stdout:\n{}",
            world.last_stdout,
        );
    } else {
        assert!(
            world.last_stdout.contains(expected.as_str()),
            "stdout did not contain:\n{expected}\n\nActual stdout:\n{}",
            world.last_stdout,
        );
    }
}

#[then(regex = r"^stderr (?:should )?contains?:$")]
async fn stderr_contains(world: &mut DocCheckWorld, step: &Step) {
    let expected = interpolate(&docstring(step), world);
    assert!(
        world.last_stderr.contains(expected.as_str()),
        "stderr did not contain:\n{expected}\n\nActual stderr:\n{}",
        world.last_stderr,
    );
}

#[then(regex = r"^stdout (?:should not|does not) contains?:$")]
async fn stdout_does_not_contain(world: &mut DocCheckWorld, step: &Step) {
    let unexpected = docstring(step);
    assert!(
        !world.last_stdout.contains(unexpected.as_str()),
        "stdout contained unexpected text:\n{unexpected}\n\nActual stdout:\n{}",
        world.last_stdout,
    );
}

#[then("stdout equals:")]
async fn stdout_equals(world: &mut DocCheckWorld, step: &Step) {
    let expected = interpolate(&format!("{}\n", docstring(step)), world);
    assert_eq!(
        world.last_stdout, expected,
        "stdout did not equal expected\nExpected:\n{expected}\nActual:\n{}",
        world.last_stdout,
    );
}

#[then("I see this output:")]
async fn see_this_output(world: &mut DocCheckWorld, step: &Step) {
    let expected = interpolate(&format!("{}\n", docstring(step)), world);
    assert_eq!(
        world.last_stdout, expected,
        "stdout did not equal expected\nExpected:\n{expected}\nActual:\n{}",
        world.last_stdout,
    );
}

#[then(regex = r#"^the file "([^"]+)" (?:should )?contains? exactly:$"#)]
async fn file_contains_exactly(world: &mut DocCheckWorld, step: &Step, path: String) {
    let expected = interpolate(&format!("{}\n", docstring(step)), world);
    let actual = fs::read_to_string(world.resolve(&path))
        .unwrap_or_else(|_| panic!("file not found: {path}"));
    assert_eq!(
        actual, expected,
        "file {path} did not equal expected\nExpected:\n{expected}\nActual:\n{actual}",
    );
}

#[then(regex = r#"^the file "([^"]+)" contains:$"#)]
async fn file_contains(world: &mut DocCheckWorld, step: &Step, path: String) {
    let expected = interpolate(&docstring(step), world);
    let actual = fs::read_to_string(world.resolve(&path))
        .unwrap_or_else(|_| panic!("file not found: {path}"));
    if expected.contains("...") {
        assert!(
            contains_in_order(&actual, &expected),
            "file {path} did not contain expected chunks in order:\n{expected}\n\nActual:\n{actual}",
        );
    } else {
        assert!(
            actual.contains(expected.as_str()),
            "file {path} did not contain:\n{expected}\n\nActual:\n{actual}",
        );
    }
}

#[then(regex = r#"^the file "([^"]+)" exists$"#)]
async fn file_exists(world: &mut DocCheckWorld, path: String) {
    assert!(
        world.resolve(&path).is_file(),
        "expected file to exist: {path}"
    );
}

#[then(regex = r#"^the file "([^"]+)" does not exist$"#)]
async fn file_does_not_exist(world: &mut DocCheckWorld, path: String) {
    assert!(
        !world.resolve(&path).exists(),
        "expected file not to exist: {path}"
    );
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let features = format!("{}/features", env!("OUT_DIR"));
    DocCheckWorld::run(features).await;
}
