use contrast::contrast;
use css_colors::{percent, Color};
use image::imageops::FilterType;
use image::{imageops, ImageBuffer};
use resvg;
use rgb::RGB8;
use sciolyff::interpreter::{html::HTMLOptions, Interpreter};
use std::{
    collections::HashMap,
    default::Default,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
use time::{date, OffsetDateTime};
use usvg::{FitTo, Options, Tree};

fn main() {
    let tournaments_results = get_tournament_info();
    write_result_pages(&tournaments_results);
}

struct TournamentResult {
    interpreter: Interpreter,
    source_file_name: OsString,
    date_added: OffsetDateTime,
    logo_path: PathBuf,
    theme_color: String,
}

fn get_tournament_info() -> Vec<TournamentResult> {
    let mut tournaments = Vec::new();

    let entries = fs::read_dir("results").expect("could not read results dir");
    let logo_info = get_logo_info().expect("could not get logo info");
    for entry in entries {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        println!("Parsing info for {:?}...", path);

        let yaml = fs::read_to_string(&path)
            .expect(&format!("could not read file at {:?}", path));
        let interpreter = Interpreter::from_yaml(&yaml);
        let source_file_name = path.file_name().unwrap().to_os_string();
        let date_added = get_date_added(&source_file_name)
            .expect("could not get date added from git");
        let (logo_path, theme_color) =
            get_logo_path_and_color(&source_file_name, &logo_info)
            .expect("could not find matching logo");

        tournaments.push(TournamentResult {
            interpreter,
            source_file_name,
            date_added,
            logo_path,
            theme_color,
        });
    }

    println!("------------------------------------------------------------");
    println!("Parsing complete.");
    println!("------------------------------------------------------------");
    tournaments
}

fn get_date_added(source_file_name: &OsStr) -> io::Result<OffsetDateTime> {
    let mut path: PathBuf =
        [OsStr::new("results"), source_file_name].iter().collect();
    let mut date = get_date_from_git(&path)?;

    // results were moved from data to results directory
    if date.date() < date!(2020 - 07 - 08) {
        path = [OsStr::new("data"), source_file_name].iter().collect();
        date = get_date_from_git(&path)?;
    }

    Ok(date)
}

fn get_date_from_git(source_file_path: &Path) -> io::Result<OffsetDateTime> {
    let output = Command::new("git")
        .arg("log")
        .arg("--format=%ai")
        .arg("--reverse")
        .arg("--")
        .arg(source_file_path)
        .output()?;
    let date_string = String::from_utf8(output.stdout).unwrap();

    if let Ok(date) = OffsetDateTime::parse(&date_string, "%F %T %z") {
        Ok(date)
    } else {
        println!(
            "Warning: {} not found in git tree; \
            using current time as date added",
            source_file_path.display()
        );
        Ok(OffsetDateTime::now_local())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Logo {
    division: Option<String>,
    minimum_year: u32,
    path: PathBuf,
    theme_color: String,
}

fn get_logo_info() -> io::Result<HashMap<String, Vec<Logo>>> {
    let mut logo_info = HashMap::new();

    let entries = fs::read_dir("public/results/logos")?;
    for entry in entries {
        let path = entry?.path();
        let file_name = path
            .file_stem()
            .unwrap()
            .to_str()
            .expect("logo file name must be valid Unicode");
        let splits = file_name.split('_').collect::<Vec<_>>();

        let minimum_year: u32 = splits[0].parse().unwrap_or(0);
        let start_index = if minimum_year == 0 { 0 } else { 1 };
        let (division, tournament_name) = if splits[splits.len() - 1].len() == 1
        {
            (
                Some(splits[splits.len() - 1].to_string()),
                splits[start_index..(splits.len() - 1)].join("_"),
            )
        } else {
            (
                None,
                splits[start_index..splits.len()].join("_").to_string(),
            )
        };
        let theme_color = get_theme_color(&path);

        let entry = logo_info.entry(tournament_name).or_insert_with(Vec::new);
        entry.push(Logo {
            division,
            minimum_year,
            path,
            theme_color,
        });
    }

    for logos in logo_info.values_mut() {
        logos.sort();
        logos.reverse();
    }

    Ok(logo_info)
}

fn get_logo_path_and_color(
    source_file_name: &OsStr,
    logo_info: &HashMap<String, Vec<Logo>>,
) -> io::Result<(PathBuf, String)> {
    let default_logo_path = PathBuf::from("public/results/logos/default.png");
    let default_theme_color = "#303030".to_string();

    let source_file_str = source_file_name
        .to_str()
        .expect("results file name must be valid Unicode");
    let year: u32 = source_file_str.splitn(2, '-').collect::<Vec<_>>()[0]
        .parse()
        .expect("results file name must start with a year");
    let splits = source_file_str.splitn(2, '_').collect::<Vec<_>>()[1]
        .rsplitn(2, '_')
        .collect::<Vec<_>>();

    let division = splits[0].splitn(2, '.').collect::<Vec<_>>()[0];
    let tournament_name = splits[1];

    let logo_path_and_color = match logo_info.get(tournament_name) {
        Some(logos) => {
            match logos.iter().find(|logo| {
                (logo.division.is_none()
                    || logo.division.as_ref().unwrap() == division)
                    && logo.minimum_year <= year
            }) {
                Some(logo) => (logo.path.clone(), logo.theme_color.clone()),
                None => (default_logo_path, default_theme_color),
            }
        }
        None => (default_logo_path, default_theme_color),
    };

    Ok(logo_path_and_color)
}

fn get_theme_color(logo_path: &Path) -> String {
    let image = if logo_path.extension().unwrap() == "svg" {
        let svg = resvg::render(
            &Tree::from_file(logo_path, &Options::default()).unwrap(),
            FitTo::Original,
            None,
        )
        .unwrap();
        ImageBuffer::from_vec(svg.width(), svg.height(), svg.take()).unwrap()
    } else {
        image::open(logo_path).unwrap().into_rgba()
    };

    let pixel = imageops::resize(&image, 1, 1, FilterType::Triangle).into_raw();

    let mut color = css_colors::rgb(pixel[0], pixel[1], pixel[2]);
    let text_color = RGB8::new(255, 255, 255);

    while contrast::<_, f32>(
        RGB8::new(color.r.as_u8(), color.g.as_u8(), color.b.as_u8()),
        text_color,
    ) < 7.0
    {
        color = color.darken(percent(1));
    }

    color.to_css()
}

fn write_result_pages(tournaments: &[TournamentResult]) {
    fs::create_dir_all("public/results").expect("could not create results dir");

    for tournament in tournaments {
        let mut path = PathBuf::from("public/results");
        path.push(&tournament.source_file_name);
        path.set_extension("html");

        println!("Writing to {:?}...", path);
        fs::write(
            &path,
            tournament.interpreter.to_html(&HTMLOptions {
                color: tournament.theme_color.clone(),
                ..Default::default()
            }),
        )
        .expect(&format!("could not write to path {:?}", path));
    }

    println!("------------------------------------------------------------");
    println!("Results pages complete.");
    println!("------------------------------------------------------------");
}
