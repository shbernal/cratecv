use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, input, output] if command == "build" => match build(input, output) {
            Ok(()) => ExitCode::SUCCESS,
            Err(why) => {
                eprintln!("{why}");
                for found in why.diagnostics() {
                    eprintln!("  {input}:{found}");
                }
                ExitCode::from(1)
            }
        },
        _ => {
            eprintln!("usage: cratecv build <resume.yaml> <out.pdf>");
            ExitCode::from(2)
        }
    }
}

fn build(input: &str, output: &str) -> Result<(), cratecv::Error> {
    let yaml = std::fs::read_to_string(input)?;
    let resume = cratecv::load(&yaml).map_err(cratecv::Error::Resume)?;
    let document = cratecv::compile(&resume)?;
    let pdf = cratecv::export_pdf(&document)?;
    if let Some(parent) = std::path::Path::new(output).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, pdf)?;
    Ok(())
}
