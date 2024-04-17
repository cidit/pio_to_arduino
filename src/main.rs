use clap::Parser;
use core::panic;
use std::{env, fs, io::Write, path::PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct AppArgs {
    project: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = AppArgs::parse();
    let project_dir = if let Some(project) = args.project {
        fs::canonicalize(project)?
    } else {
        env::current_dir()?
    };

    let out_file_name = fs::File::create_new(PathBuf::from(format!(
        "{}.zip",
        project_dir
            .file_name()
            .expect("the name of the current folder should never be empty")
            .to_string_lossy()
    )))?;

    let mut zip = ZipWriter::new(out_file_name);
    let file_options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("libraries.txt", file_options)?;
    let Ok(project_metadata) = fs::read_to_string(project_dir.join("platformio.ini")) else {
        panic!("Not a PIO")
    };
    let libraries: String = project_metadata
        .lines()
        .skip_while(|l| !l.contains("lib_deps"))
        .skip(1)
        .map(|s| s.trim())
        .map(|s| format!("{s}\n"))
        .collect();
    zip.write(libraries.as_bytes())?;

    let src_dir = WalkDir::new(project_dir.join("src"));
    let include_dir = WalkDir::new(project_dir.join("include"));
    for entry in src_dir.into_iter().chain(include_dir.into_iter()) {
        let entry = entry?;
        let path = entry.path();
        if entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name();
        if name.to_string_lossy().to_lowercase().contains("readme") {
            continue;
        }
        let contents = fs::read_to_string(path)?;
        zip.start_file(entry.file_name().to_string_lossy(), file_options)?;
        zip.write(contents.as_bytes())?;
    }

    if let Ok(contents) = fs::read_to_string(project_dir.join("README.md")) {
        zip.start_file("README.md", file_options)?;
        zip.write(contents.as_bytes())?;
    }

    zip.start_file(
        format!(
            r"{}.ino",
            project_dir.file_name().expect("project should have a name").to_string_lossy()
        ),
        file_options,
    )?;
    zip.write("
// TAKEN FROM: <https://github.com/arkhipenko/TaskScheduler/blob/master/examples/Scheduler_example16_Multitab/Scheduler_example16_Multitab.ino>
//
//This file is intentionally left blank.
//
//Arduino IDE plays some dirty tricks on the main sketch .ino file:
//it rearranges #includes, blindly creates forward definitions,
//includes every file in the project that does not have .c or .cpp
//file extension.
//
//Usually it all turns well if you have only one source file and you are either
//inexperienced or really expert C++ Arduino programmer.
//For the folks with the middle ground skills level, when you want
//to split your code into several .cpp files, it is best to leave
//this main sketch empty.
//
//It doesn't matter where you define the void loop() and void setup().
//Just make sure there is exactly one definition of each.
//
//And if you want to use standard Arduino functions
//like digitalWrite or the Serial object - just add #include<Arduino.h>.           
        ".as_bytes())?;

    return Ok(());
}
